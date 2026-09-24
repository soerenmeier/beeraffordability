use std::{error::Error, time::Duration};

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::gql;

const SAVE_COUNTRY: &str = include_str!("graphql/save_country.graphql");

#[derive(Deserialize)]
struct CountryEntry {
    title: String,
}

#[derive(Deserialize)]
struct SavedCountry {
    #[serde(rename = "save_countries_country_Entry")]
    entry: Option<CountryEntry>,
}

/// Create each country in Craft's Countries section.
pub async fn push(
    endpoint: &str,
    token: &str,
    countries: &[String],
) -> Result<usize, Box<dyn Error + Send + Sync>> {
    if countries.is_empty() {
        return Err("countries.json contains no countries".into());
    }

    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;

    for country in countries {
        let saved: SavedCountry = gql::execute(
            &client,
            endpoint,
            token,
            SAVE_COUNTRY,
            json!({ "title": country }),
        )
        .await
        .map_err(|error| format!("Saving {country}: {error}"))?;

        if saved
            .entry
            .as_ref()
            .is_none_or(|entry| &entry.title != country)
        {
            return Err(format!("Craft did not save country {country}").into());
        }
    }

    Ok(countries.len())
}
