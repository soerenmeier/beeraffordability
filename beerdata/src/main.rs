mod countries;
mod fx;
mod gql;
mod numbeo;
mod push_countries;
mod sync_datapoints;

use std::{error::Error, fs};

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Download Numbeo's country list to countries.json.
    Countries,
    /// Fetch Numbeo beer prices and net salaries, then upload USD datapoints.
    Sync {
        #[arg(long)]
        endpoint_url: String,
        #[arg(long)]
        endpoint_token: String,
        #[arg(long)]
        apify_api_token: String,
        /// Use available cached prices without starting a new Apify run.
        #[arg(long)]
        cached_only: bool,
        /// Import an older run that predates the cache (not needed for cached runs).
        #[arg(long)]
        resume_apify_run_id: Option<String>,
        #[arg(long)]
        quarter_id: i32,
    },
    /// Create country entries from countries.json in Craft.
    PushCountries {
        #[arg(long)]
        endpoint_url: String,
        #[arg(long)]
        endpoint_token: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Args::parse();

    match args.cmd {
        Cmd::Countries => {
            let countries = countries::fetch().await?;
            let json = serde_json::to_string_pretty(&countries)?;

            fs::write("countries.json", json)?;
            println!("Saved {} countries to countries.json", countries.len());
        }
        Cmd::Sync {
            endpoint_url,
            endpoint_token,
            apify_api_token,
            cached_only,
            resume_apify_run_id,
            quarter_id,
        } => {
            sync(
                &endpoint_url,
                &endpoint_token,
                &apify_api_token,
                cached_only,
                resume_apify_run_id.as_deref(),
                quarter_id,
            )
            .await?
        }
        Cmd::PushCountries {
            endpoint_url,
            endpoint_token,
        } => {
            let countries: Vec<String> =
                serde_json::from_str(&fs::read_to_string("countries.json")?)?;
            let created = push_countries::push(&endpoint_url, &endpoint_token, &countries).await?;

            println!("Created {created} countries");
        }
    }

    Ok(())
}

async fn sync(
    endpoint_url: &str,
    endpoint_token: &str,
    apify_api_token: &str,
    cached_only: bool,
    resume_apify_run_id: Option<&str>,
    quarter_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let countries: Vec<String> = serde_json::from_str(&fs::read_to_string("countries.json")?)?;
    println!("Checking Craft countries and existing datapoints before scraping...");
    let craft = sync_datapoints::prepare(endpoint_url, endpoint_token, quarter_id).await?;

    let rates_path = format!("tmp/fx-{quarter_id}.json");
    let rates = fx::UsdRates::load_or_fetch(std::path::Path::new(&rates_path)).await?;

    let cache_path = format!("tmp/apify-{quarter_id}.json");
    println!(
        "Loading Apify cache from {cache_path} for {} countries...",
        countries.len()
    );
    let prices = numbeo::domestic_beer(
        apify_api_token,
        &countries,
        std::path::Path::new(&cache_path),
        cached_only,
        resume_apify_run_id,
    )
    .await?;
    let json = serde_json::to_string_pretty(&prices)?;

    fs::write("numbeo.json", json)?;
    println!(
        "Saved {} country prices and salaries to numbeo.json",
        prices.len()
    );

    println!("Matching Numbeo data with Craft countries and uploading USD datapoints...");
    let (created, skipped, missing_salaries) = sync_datapoints::push(
        endpoint_url,
        endpoint_token,
        quarter_id,
        &prices,
        &rates,
        &craft,
    )
    .await?;
    println!(
        "Created {created} USD datapoints for quarter {quarter_id}, skipped {skipped} existing or duplicate entries and {missing_salaries} countries without a Numbeo salary"
    );

    Ok(())
}
