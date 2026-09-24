use std::{
    collections::{HashMap, HashSet},
    error::Error,
    time::Duration,
};

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::{fx::UsdRates, gql, numbeo::CountryPrice};

/// Assume a 40-hour work week when estimating hourly pay from net monthly salary.
const HOURS_PER_MONTH: f64 = 40.0 * 52.0 / 12.0;

const COUNTRIES_WITH_IDS: &str = include_str!("graphql/countries_with_ids.graphql");
const DATAPOINTS_FOR_QUARTER: &str = include_str!("graphql/datapoints_for_quarter.graphql");
const SAVE_DATAPOINT: &str = include_str!("graphql/save_datapoint.graphql");

#[derive(Deserialize)]
struct CountryEntries {
    entries: Vec<CountryEntry>,
}

#[derive(Deserialize)]
struct CountryEntry {
    id: String,
    title: String,
}

#[derive(Deserialize)]
struct DatapointEntries {
    entries: Vec<DatapointRelations>,
}

#[derive(Deserialize)]
struct DatapointRelations {
    id: String,
    quarter: Vec<RelatedEntry>,
    country: Vec<RelatedEntry>,
}

#[derive(Deserialize)]
struct RelatedEntry {
    id: String,
}

#[derive(Deserialize)]
struct SavedDatapoint {
    #[serde(rename = "save_datapoints_datapoint_Entry")]
    entry: Option<DatapointEntry>,
}

#[derive(Deserialize)]
struct DatapointEntry {
    id: String,
}

pub struct CraftState {
    country_ids: HashMap<String, i32>,
    existing: HashSet<i32>,
}

#[derive(Debug)]
struct Datapoint<'a> {
    source: &'a CountryPrice,
    country_id: i32,
    price_usd: f64,
    wage_local: f64,
    wage_usd: f64,
}

/// Check Craft country and quarter datapoint queries before starting a paid scrape.
pub async fn prepare(
    endpoint: &str,
    token: &str,
    quarter_id: i32,
) -> Result<CraftState, Box<dyn Error + Send + Sync>> {
    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;
    let data: CountryEntries =
        gql::execute(&client, endpoint, token, COUNTRIES_WITH_IDS, json!({})).await?;
    let mut country_ids = HashMap::new();

    for entry in data.entries {
        let id = entry.id.parse::<i32>()?;

        if country_ids.insert(entry.title.clone(), id).is_some() {
            return Err(format!("Multiple Craft countries named {}", entry.title).into());
        }
    }

    println!("Found {} Craft countries", country_ids.len());
    let data: DatapointEntries = gql::execute(
        &client,
        endpoint,
        token,
        DATAPOINTS_FOR_QUARTER,
        json!({ "quarterId": [quarter_id] }),
    )
    .await?;
    let existing = existing_country_ids(data, quarter_id)?;
    println!(
        "Found {} existing Craft datapoints for quarter {quarter_id}",
        existing.len()
    );

    Ok(CraftState {
        country_ids,
        existing,
    })
}

/// Match Numbeo prices to Craft countries and create missing datapoints for one quarter.
pub async fn push(
    endpoint: &str,
    token: &str,
    quarter_id: i32,
    prices: &[CountryPrice],
    rates: &UsdRates,
    craft: &CraftState,
) -> Result<(usize, usize, usize), Box<dyn Error + Send + Sync>> {
    if prices.is_empty() {
        return Err("No Numbeo prices to sync".into());
    }

    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;
    let matched = match_prices(prices, &craft.country_ids, rates)?;
    let missing_salaries = prices.len() - matched.len();
    let pending = pending_datapoints(matched, &craft.existing);
    let skipped = prices.len() - missing_salaries - pending.len();
    let count = pending.len();

    println!(
        "Skipping {skipped} already saved or duplicate country datapoints for quarter {quarter_id}"
    );

    for (index, datapoint) in pending.into_iter().enumerate() {
        let saved: SavedDatapoint = gql::execute(
            &client,
            endpoint,
            token,
            SAVE_DATAPOINT,
            json!({
                "quarterId": [quarter_id],
                "countryId": [datapoint.country_id],
                "price": datapoint.price_usd,
                "wage": datapoint.wage_usd,
                "localCurrency": datapoint.source.currency,
                "localPrice": datapoint.source.price,
                "localWage": datapoint.wage_local
            }),
        )
        .await
        .map_err(|error| format!("Saving datapoint for {}: {error}", datapoint.source.country))?;

        if saved.entry.as_ref().is_none_or(|entry| entry.id.is_empty()) {
            return Err(format!(
                "Craft did not save datapoint for {}",
                datapoint.source.country
            )
            .into());
        }

        if (index + 1) % 10 == 0 || index + 1 == count {
            println!("Saved {}/{} Craft datapoints", index + 1, count);
        }
    }

    Ok((count, skipped, missing_salaries))
}

fn existing_country_ids(
    data: DatapointEntries,
    quarter_id: i32,
) -> Result<HashSet<i32>, Box<dyn Error + Send + Sync>> {
    let mut existing = HashSet::new();

    for entry in data.entries {
        let [quarter] = entry.quarter.as_slice() else {
            return Err(format!(
                "Craft datapoint {} does not have exactly one quarter",
                entry.id
            )
            .into());
        };
        let [country] = entry.country.as_slice() else {
            return Err(format!(
                "Craft datapoint {} does not have exactly one country",
                entry.id
            )
            .into());
        };

        if quarter.id.parse::<i32>()? != quarter_id {
            return Err(
                format!("Craft returned datapoint {} from another quarter", entry.id).into(),
            );
        }

        existing.insert(country.id.parse()?);
    }

    Ok(existing)
}

fn pending_datapoints<'a>(
    matched: Vec<Datapoint<'a>>,
    existing: &HashSet<i32>,
) -> Vec<Datapoint<'a>> {
    let mut seen = existing.clone();

    matched
        .into_iter()
        .filter(|datapoint| seen.insert(datapoint.country_id))
        .collect()
}

fn match_prices<'a>(
    prices: &'a [CountryPrice],
    country_ids: &HashMap<String, i32>,
    rates: &UsdRates,
) -> Result<Vec<Datapoint<'a>>, Box<dyn Error + Send + Sync>> {
    let mut matched = Vec::new();

    for price in prices {
        let country_id = country_ids
            .get(&price.country)
            .copied()
            .ok_or_else(|| format!("No Craft country matches {}", price.country))?;
        let Some(monthly_salary) = price.monthly_salary else {
            continue;
        };

        let local_per_usd = rates.local_per_usd(&price.currency)?;
        let wage_local = monthly_salary / HOURS_PER_MONTH;
        let price_usd = price.price / local_per_usd;
        let wage_usd = wage_local / local_per_usd;

        if !price_usd.is_finite() || !wage_usd.is_finite() || wage_usd <= 0.0 {
            return Err(format!("Invalid converted price or wage for {}", price.country).into());
        }

        matched.push(Datapoint {
            source: price,
            country_id,
            price_usd,
            wage_local,
            wage_usd,
        });
    }

    Ok(matched)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_local_price_and_monthly_net_salary_to_usd_hourly() {
        let prices = vec![CountryPrice {
            country: "Germany".to_owned(),
            price: 0.99,
            currency: "EUR".to_owned(),
            monthly_salary: Some(2932.68),
        }];
        let ids = HashMap::from([("Germany".to_owned(), 42)]);
        let rates: UsdRates =
            serde_json::from_str(r#"{"date":"2026-09-22","usd":{"usd":1,"eur":0.8}}"#).unwrap();

        let matched = match_prices(&prices, &ids, &rates).unwrap();
        let datapoint = &matched[0];

        assert_eq!(datapoint.country_id, 42);
        assert!((datapoint.price_usd - 0.99 / 0.8).abs() < 1e-10);
        assert!((datapoint.wage_local - 2932.68 / HOURS_PER_MONTH).abs() < 1e-10);
        assert!((datapoint.wage_usd - datapoint.wage_local / 0.8).abs() < 1e-10);
        assert!(
            (datapoint.price_usd / datapoint.wage_usd * 60.0 - 0.99 / datapoint.wage_local * 60.0)
                .abs()
                < 1e-10
        );
        assert!(
            match_prices(&prices, &HashMap::new(), &rates)
                .unwrap_err()
                .to_string()
                .contains("Germany")
        );
    }

    #[test]
    fn skips_saved_datapoints_duplicate_prices_and_missing_salaries() {
        let data: DatapointEntries = serde_json::from_value(json!({
            "entries": [{
                "id": "10",
                "quarter": [{ "id": "4" }],
                "country": [{ "id": "2" }]
            }]
        }))
        .unwrap();
        let existing = existing_country_ids(data, 4).unwrap();
        let prices = [
            CountryPrice {
                country: "Germany".to_owned(),
                price: 0.99,
                currency: "EUR".to_owned(),
                monthly_salary: Some(2932.68),
            },
            CountryPrice {
                country: "France".to_owned(),
                price: 1.25,
                currency: "EUR".to_owned(),
                monthly_salary: Some(2000.0),
            },
            CountryPrice {
                country: "Samoa".to_owned(),
                price: 3.0,
                currency: "WST".to_owned(),
                monthly_salary: None,
            },
        ];
        let rates: UsdRates =
            serde_json::from_str(r#"{"date":"2026-09-22","usd":{"usd":1,"eur":0.8}}"#).unwrap();
        let ids = HashMap::from([
            ("Germany".to_owned(), 2),
            ("France".to_owned(), 3),
            ("Samoa".to_owned(), 4),
        ]);
        let matched = match_prices(&prices, &ids, &rates).unwrap();
        assert_eq!(matched.len(), 2);

        let france_twice = match_prices(&prices[1..2], &ids, &rates).unwrap();
        let pending =
            pending_datapoints(matched.into_iter().chain(france_twice).collect(), &existing);

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].country_id, 3);
        assert!(
            pending_datapoints(
                match_prices(&prices[0..1], &ids, &rates).unwrap(),
                &existing
            )
            .is_empty()
        );
    }

    #[test]
    fn refuses_to_upload_when_existing_datapoints_cannot_be_identified() {
        let data: DatapointEntries = serde_json::from_value(json!({
            "entries": [{
                "id": "10",
                "quarter": [{ "id": "5" }],
                "country": [{ "id": "2" }]
            }]
        }))
        .unwrap();

        assert!(existing_country_ids(data, 4).is_err());

        let data: DatapointEntries = serde_json::from_value(json!({
            "entries": [{
                "id": "10",
                "quarter": [{ "id": "4" }],
                "country": []
            }]
        }))
        .unwrap();

        assert!(existing_country_ids(data, 4).is_err());
    }
}
