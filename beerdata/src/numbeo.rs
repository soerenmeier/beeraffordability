use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
    io::ErrorKind,
    path::Path,
    time::Duration,
};

use reqwest::Client;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const ACTOR_RUNS_URL: &str =
    "https://api.apify.com/v2/acts/logiover~numbeo-cost-of-living-scrape/runs";

#[derive(Debug, PartialEq, Serialize)]
pub struct CountryPrice {
    pub country: String,
    /// Average grocery-store price for a 0.5 liter bottle in the local currency.
    pub price: f64,
    pub currency: String,
    /// Average net monthly salary from the same country-level Numbeo dataset.
    pub monthly_salary: Option<f64>,
}

#[derive(Deserialize)]
struct RunResponse {
    data: Run,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Run {
    id: String,
    status: String,
    status_message: Option<String>,
    default_dataset_id: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PriceRow {
    #[serde(rename = "type")]
    kind: Option<String>,
    country: Option<String>,
    category_group: Option<String>,
    item_name: Option<String>,
    avg_price: Option<f64>,
    lowest_price: Option<f64>,
    highest_price: Option<f64>,
    currency: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct ApifyCache {
    countries: Vec<String>,
    completed: Vec<String>,
    imported_runs: Vec<String>,
    #[serde(default)]
    active_run_id: Option<String>,
    rows: Vec<Value>,
}

/// Fetch national-average domestic bottled-beer prices and net monthly salaries.
///
/// `api_key` is an Apify API token. This starts a billable actor run that emits
/// approximately 55 dataset items per country, even though only beer is returned.
/// Cached rows and active runs are resumed automatically; `resume_run_id` can
/// import an older run that predates the cache.
pub async fn domestic_beer(
    api_key: &str,
    countries: &[String],
    cache_path: &Path,
    cached_only: bool,
    resume_run_id: Option<&str>,
) -> Result<Vec<CountryPrice>, Box<dyn Error + Send + Sync>> {
    if countries.is_empty() {
        return Err("No countries provided for the Numbeo scrape".into());
    }

    let mut cache = ApifyCache::load(cache_path, countries)?;
    let client = Client::builder().timeout(Duration::from_secs(75)).build()?;
    let mut recovered = false;

    if cache.rows.is_empty() && cache.active_run_id.is_none() {
        if let Some(run_id) = cache.imported_runs.last().cloned() {
            println!("Recovering dataset from previous Apify run {run_id} (no new paid run)...");
            let previous: RunResponse = client
                .get(format!("https://api.apify.com/v2/actor-runs/{run_id}"))
                .bearer_auth(api_key)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            let rows = dataset_rows(&client, api_key, &previous.data.default_dataset_id).await?;

            if rows.is_empty() {
                return Err(format!(
                    "Apify run {run_id} has no dataset rows; no new run was started"
                )
                .into());
            }

            println!("Recovered {} raw Apify rows", rows.len());
            cache.record(&run_id, rows, None);
            cache.save(cache_path)?;
            recovered = true;
        }
    }

    if let Some(run_id) = resume_run_id {
        if !cache.imported_runs.iter().any(|id| id == run_id) {
            let previous: RunResponse = client
                .get(format!("https://api.apify.com/v2/actor-runs/{run_id}"))
                .bearer_auth(api_key)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            if !matches!(
                previous.data.status.as_str(),
                "SUCCEEDED" | "FAILED" | "TIMED-OUT" | "ABORTED"
            ) {
                return Err(format!("Apify run {run_id} has not finished yet").into());
            }

            let rows = dataset_rows(&client, api_key, &previous.data.default_dataset_id).await?;
            let count = rows.len();
            cache.record(&previous.data.id, rows, None);
            cache.save(cache_path)?;
            println!(
                "Imported {count} rows from Apify run {run_id} into {}",
                cache_path.display()
            );
        }
    }

    let pending = cache.pending();
    println!(
        "Apify cache: {} of {} countries already covered ({})",
        countries.len() - pending.len(),
        countries.len(),
        cache_path.display()
    );

    if pending.is_empty() && cache.active_run_id.is_none() {
        return cached_prices(cache);
    }

    if cached_only && cache.active_run_id.is_none() {
        println!(
            "Using {} cached Apify rows; no new run will be started",
            cache.rows.len()
        );
        return cached_prices(cache);
    }

    if recovered {
        return Err(format!(
            "Recovered {} raw rows to {}. {} countries have matching beer prices; first row fields: {}. No new paid run was started. Use --cached-only to upload available prices, or rerun sync to request missing countries",
            cache.rows.len(),
            cache_path.display(),
            countries.len() - pending.len(),
            sample_fields(&cache.rows)
        )
        .into());
    }

    if !cache.rows.is_empty() && cache.completed.is_empty() && cache.active_run_id.is_none() {
        return Err(format!(
            "Apify cache {} contains {} raw rows, but none matched a domestic beer price. First row fields: {}. No new paid run was started",
            cache_path.display(),
            cache.rows.len(),
            sample_fields(&cache.rows)
        )
        .into());
    }

    let mut run: RunResponse = if let Some(run_id) = &cache.active_run_id {
        println!("Resuming Apify run {run_id} from {}", cache_path.display());
        client
            .get(format!("https://api.apify.com/v2/actor-runs/{run_id}"))
            .bearer_auth(api_key)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?
    } else {
        let started: RunResponse = client
            .post(ACTOR_RUNS_URL)
            .bearer_auth(api_key)
            .query(&[("timeout", "3600")])
            .json(&json!({
                "cities": [],
                "countries": &pending,
                "scrapeCategories": ["cost-of-living"],
                "scrapeRankings": false,
                "proxyConfiguration": {
                    "useApifyProxy": true,
                    "apifyProxyGroups": ["RESIDENTIAL"]
                }
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        println!(
            "Apify run {} started (https://console.apify.com/actors/runs/{})",
            started.data.id, started.data.id
        );
        cache.active_run_id = Some(started.data.id.clone());
        cache.save(cache_path)?;

        started
    };

    loop {
        match run.data.status.as_str() {
            "SUCCEEDED" => {
                println!("Apify run {} succeeded", run.data.id);
                break;
            }
            "FAILED" | "TIMED-OUT" | "ABORTED" => break,
            _ => {
                println!(
                    "Apify run {}: {} - {}",
                    run.data.id,
                    run.data.status,
                    run.data
                        .status_message
                        .as_deref()
                        .unwrap_or("waiting for progress")
                );
            }
        }

        run = client
            .get(format!(
                "https://api.apify.com/v2/actor-runs/{}",
                run.data.id
            ))
            .bearer_auth(api_key)
            .query(&[("waitForFinish", "15")])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
    }

    let rows = dataset_rows(&client, api_key, &run.data.default_dataset_id)
        .await
        .map_err(|error| {
            format!(
                "Apify run {} ended with {}; could not retrieve its dataset: {error}",
                run.data.id, run.data.status
            )
        })?;
    println!("Apify returned {} dataset rows", rows.len());

    if run.data.status == "SUCCEEDED" && rows.is_empty() {
        return Err(format!(
            "Apify run {} succeeded but returned no dataset rows",
            run.data.id
        )
        .into());
    }
    cache.record(
        &run.data.id,
        rows,
        (run.data.status == "SUCCEEDED").then_some(&pending),
    );
    cache.active_run_id = None;
    cache.save(cache_path)?;

    if cached_only {
        println!(
            "Using {} cached Apify rows; no new run was started",
            cache.rows.len()
        );
        return cached_prices(cache);
    }

    if run.data.status != "SUCCEEDED" {
        return Err(format!(
            "Apify run {} ended with {}: {}. Saved {} rows to {}; {} countries still need scraping. Check https://console.apify.com/actors/runs/{}",
            run.data.id,
            run.data.status,
            run.data.status_message.as_deref().unwrap_or("no details"),
            cache.rows.len(),
            cache_path.display(),
            cache.pending().len(),
            run.data.id
        )
        .into());
    }

    cached_prices(cache)
}

impl ApifyCache {
    fn load(path: &Path, countries: &[String]) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(Self {
                    countries: countries.to_vec(),
                    completed: Vec::new(),
                    imported_runs: Vec::new(),
                    active_run_id: None,
                    rows: Vec::new(),
                });
            }
            Err(error) => return Err(error.into()),
        };
        let mut cache: Self = serde_json::from_slice(&bytes)?;

        if cache.countries != countries {
            return Err(format!(
                "Apify cache {} is for a different country list; archive it before starting a new scrape",
                path.display()
            )
            .into());
        }

        cache.update_completed(None);

        Ok(cache)
    }

    fn pending(&self) -> Vec<String> {
        let completed: HashSet<&str> = self.completed.iter().map(String::as_str).collect();

        self.countries
            .iter()
            .filter(|country| !completed.contains(country.as_str()))
            .cloned()
            .collect()
    }

    fn record(&mut self, run_id: &str, rows: Vec<Value>, completed_request: Option<&[String]>) {
        self.rows.extend(rows);
        self.update_completed(completed_request);

        if !self.imported_runs.iter().any(|id| id == run_id) {
            self.imported_runs.push(run_id.to_owned());
        }
    }

    fn update_completed(&mut self, completed_request: Option<&[String]>) {
        let known: HashSet<&str> = self.countries.iter().map(String::as_str).collect();
        let mut completed: HashSet<String> = self.completed.iter().cloned().collect();

        for raw in &self.rows {
            let Ok(row) = serde_json::from_value::<PriceRow>(raw.clone()) else {
                continue;
            };
            let Some(country) = row.country.as_deref() else {
                continue;
            };

            let country = numbeo_country(country);
            if known.contains(country) && is_usable_beer_row(&row) {
                completed.insert(country.to_owned());
            }
        }

        if let Some(requested) = completed_request {
            completed.extend(requested.iter().cloned());
        }

        self.completed = self
            .countries
            .iter()
            .filter(|country| completed.contains(*country))
            .cloned()
            .collect();
    }

    fn save(&self, path: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        fs::rename(temporary, path)?;

        Ok(())
    }
}

async fn dataset_rows(
    client: &Client,
    api_key: &str,
    dataset_id: &str,
) -> Result<Vec<Value>, Box<dyn Error + Send + Sync>> {
    Ok(client
        .get(format!(
            "https://api.apify.com/v2/datasets/{dataset_id}/items"
        ))
        .bearer_auth(api_key)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

fn cached_prices(cache: ApifyCache) -> Result<Vec<CountryPrice>, Box<dyn Error + Send + Sync>> {
    let row_count = cache.rows.len();
    let fields = sample_fields(&cache.rows);
    let rows: Vec<PriceRow> = cache
        .rows
        .into_iter()
        .filter_map(|row| serde_json::from_value(row).ok())
        .collect();
    let rejected = rows
        .iter()
        .filter(|row| is_beer_row(row) && !is_usable_beer_row(row))
        .count();
    if rejected > 0 {
        println!("Skipped {rejected} bottled-beer prices inconsistent with their reported ranges");
    }
    let prices = beer_prices(rows);
    let missing_salaries = prices
        .iter()
        .filter(|price| price.monthly_salary.is_none())
        .count();
    if missing_salaries > 0 {
        println!("{missing_salaries} countries have a beer price but no matching Numbeo salary");
    }

    if prices.is_empty() {
        return Err(format!(
            "Cached {} Apify rows, but none matched a domestic beer price. First row fields: {}",
            row_count, fields
        )
        .into());
    }

    let known: HashSet<&str> = cache.countries.iter().map(String::as_str).collect();
    if let Some(price) = prices
        .iter()
        .find(|price| !known.contains(price.country.as_str()))
    {
        return Err(format!("Numbeo country {} is not in countries.json", price.country).into());
    }

    Ok(prices)
}

fn sample_fields(rows: &[Value]) -> String {
    rows.first()
        .and_then(Value::as_object)
        .map(|row| row.keys().cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_else(|| "not an object".to_owned())
}

fn numbeo_country(country: &str) -> &str {
    country
        .strip_prefix("Cost of Living in ")
        .unwrap_or(country)
}

fn is_beer_row(row: &PriceRow) -> bool {
    row.kind.as_deref() == Some("country_price")
        && row.category_group.as_deref() == Some("Markets")
        && row
            .item_name
            .as_deref()
            .is_some_and(|name| name.eq_ignore_ascii_case("Domestic Beer (0.5 Liter Bottle)"))
}

fn is_usable_beer_row(row: &PriceRow) -> bool {
    let Some(price) = row.avg_price else {
        return false;
    };

    is_beer_row(row)
        && price.is_finite()
        && price > 0.0
        && row
            .currency
            .as_deref()
            .is_some_and(|currency| !currency.is_empty())
        && row.lowest_price.is_none_or(|lowest| price >= lowest)
        && row.highest_price.is_none_or(|highest| price <= highest)
}

fn beer_prices(rows: Vec<PriceRow>) -> Vec<CountryPrice> {
    let salaries: HashMap<(String, String), f64> = rows
        .iter()
        .filter(|row| {
            row.kind.as_deref() == Some("country_price")
                && row.category_group.as_deref() == Some("Salaries And Financing")
                && row.item_name.as_deref().is_some_and(|name| {
                    name.eq_ignore_ascii_case("Average Monthly Net Salary (After Tax)")
                })
                && row
                    .avg_price
                    .is_some_and(|salary| salary.is_finite() && salary > 0.0)
        })
        .filter_map(|row| {
            let country = numbeo_country(row.country.as_deref()?);
            let currency = row.currency.as_deref()?;
            Some(((country.to_owned(), currency.to_owned()), row.avg_price?))
        })
        .collect();

    rows.into_iter()
        .filter(is_usable_beer_row)
        .filter_map(|row| {
            let country = numbeo_country(row.country.as_deref()?).to_owned();
            let currency = row.currency?;
            let monthly_salary = salaries.get(&(country.clone(), currency.clone())).copied();

            Some(CountryPrice {
                country,
                price: row.avg_price?,
                currency,
                monthly_salary,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_empty_countries_before_starting_actor() {
        assert!(
            domestic_beer("unused", &[], Path::new("tmp/unused.json"), false, None)
                .await
                .is_err()
        );
    }

    #[test]
    fn loads_cache_from_before_active_runs_were_saved() {
        let cache: ApifyCache =
            serde_json::from_str(r#"{"countries":[],"completed":[],"imported_runs":[],"rows":[]}"#)
                .unwrap();

        assert!(cache.active_run_id.is_none());
    }

    #[tokio::test]
    async fn caches_partial_results_and_only_retries_uncovered_countries() {
        let countries = vec!["Germany".to_owned(), "France".to_owned()];
        let mut cache = ApifyCache {
            countries: countries.clone(),
            completed: Vec::new(),
            imported_runs: Vec::new(),
            active_run_id: Some("aborted-run".to_owned()),
            rows: Vec::new(),
        };
        let rows = serde_json::from_str(r#"[
            {"type":"country_price","country":"Germany","categoryGroup":"Markets","itemName":"Domestic Beer (0.5 Liter Bottle)","avgPrice":0.99,"currency":"EUR"},
            {"type":"country_price","country":"France","categoryGroup":"Markets","itemName":"Milk","avgPrice":1.20,"currency":"EUR"},
            {"other":"unrecognized actor field"}
        ]"#).unwrap();

        cache.record("aborted-run", rows, None);
        assert_eq!(cache.pending(), ["France"]);
        assert_eq!(cache.rows.len(), 3);
        assert_eq!(cache.rows[2]["other"], "unrecognized actor field");

        let path = std::path::PathBuf::from(format!(
            "tmp/apify-test-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        cache.save(&path).unwrap();
        let mut loaded = ApifyCache::load(&path, &countries).unwrap();
        assert_eq!(loaded.active_run_id.as_deref(), Some("aborted-run"));
        let wrong_countries = ["Germany".to_owned()];
        assert!(ApifyCache::load(&path, &wrong_countries).is_err());

        loaded.active_run_id = None;
        loaded.save(&path).unwrap();
        let partial = domestic_beer("unused", &countries, &path, true, None)
            .await
            .unwrap();
        assert_eq!(partial.len(), 1);
        fs::remove_file(&path).unwrap();

        loaded.record("finished-run", Vec::new(), Some(&["France".to_owned()]));
        assert!(loaded.pending().is_empty());
        assert_eq!(cached_prices(loaded).unwrap().len(), 1);
    }

    #[test]
    fn maps_numbeo_page_titles_and_rejects_inconsistent_prices() {
        let raw: Vec<Value> = serde_json::from_str(r#"[
            {"type":"country_price","country":"Cost of Living in Brazil","categoryGroup":"Markets","itemName":"Domestic Beer (0.5 Liter Bottle)","avgPrice":6.81,"lowestPrice":4,"highestPrice":12,"currency":"BRL"},
            {"type":"country_price","country":"Cost of Living in Bhutan","categoryGroup":"Markets","itemName":"Domestic Beer (0.5 Liter Bottle)","avgPrice":0.77,"lowestPrice":35,"highestPrice":136.68,"currency":"BTN"}
        ]"#).unwrap();
        let mut cache = ApifyCache {
            countries: vec!["Brazil".to_owned(), "Bhutan".to_owned()],
            completed: Vec::new(),
            imported_runs: Vec::new(),
            active_run_id: None,
            rows: Vec::new(),
        };

        cache.record("aborted-run", raw, None);
        assert_eq!(cache.pending(), ["Bhutan"]);
        assert_eq!(
            cached_prices(cache).unwrap(),
            vec![CountryPrice {
                country: "Brazil".to_owned(),
                price: 6.81,
                currency: "BRL".to_owned(),
                monthly_salary: None,
            }]
        );
    }

    #[test]
    fn selects_grocery_beer_from_country_prices_only() {
        let rows = serde_json::from_str(r#"[
            {"type":"country_price","country":"Germany","categoryGroup":"Markets","itemName":"Domestic Beer (0.5 Liter Bottle)","avgPrice":0.99,"currency":"EUR"},
            {"type":"country_price","country":"Germany","categoryGroup":"Salaries And Financing","itemName":"Average Monthly Net Salary (After Tax)","avgPrice":2932.68,"currency":"EUR"},
            {"type":"country_price","country":"Germany","categoryGroup":"Salaries And Financing","itemName":"Average Monthly Net Salary (After Tax)","avgPrice":4000,"currency":"USD"},
            {"type":"city_price","country":"Iceland","categoryGroup":"Salaries And Financing","itemName":"Average Monthly Net Salary (After Tax)","avgPrice":3000,"currency":"ISK"},
            {"type":"country_price","country":"Germany","categoryGroup":"Restaurants","itemName":"Domestic Beer (0.5 Liter Bottle)","avgPrice":4.50,"currency":"EUR"},
            {"type":"city_price","city":"Berlin","categoryGroup":"Markets","itemName":"Domestic Beer (0.5 Liter Bottle)","avgPrice":1.25,"currency":"EUR"},
            {"type":"country_price","country":"Iceland","categoryGroup":"Markets","itemName":"Domestic Beer (0.5 Liter Bottle)","avgPrice":null,"currency":"ISK"}
        ]"#).unwrap();

        assert_eq!(
            beer_prices(rows),
            vec![CountryPrice {
                country: "Germany".to_owned(),
                price: 0.99,
                currency: "EUR".to_owned(),
                monthly_salary: Some(2932.68),
            }]
        );
    }
}
