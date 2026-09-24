use std::{collections::HashMap, error::Error, fs, io::ErrorKind, path::Path, time::Duration};

use reqwest::Client;
use serde::{Deserialize, Serialize};

const USD_RATES_URL: &str =
    "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/usd.json";
const FALLBACK_URL: &str = "https://latest.currency-api.pages.dev/v1/currencies/usd.json";

/// A single snapshot of units of each currency per USD.
#[derive(Deserialize, Serialize)]
pub struct UsdRates {
    pub date: String,
    usd: HashMap<String, f64>,
}

impl UsdRates {
    pub async fn load_or_fetch(path: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        match fs::read(path) {
            Ok(bytes) => {
                let rates: Self = serde_json::from_slice(&bytes)?;
                rates.validate()?;
                println!(
                    "Using USD exchange rates dated {} from {}",
                    rates.date,
                    path.display()
                );
                return Ok(rates);
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        for url in [USD_RATES_URL, FALLBACK_URL] {
            let result: Result<Self, Box<dyn Error + Send + Sync>> = async {
                let rates: Self = client
                    .get(url)
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                rates.validate()?;
                Ok(rates)
            }
            .await;

            match result {
                Ok(rates) => {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    let temporary = path.with_extension("json.tmp");
                    fs::write(&temporary, serde_json::to_vec_pretty(&rates)?)?;
                    fs::rename(temporary, path)?;
                    println!(
                        "Saved USD exchange rates dated {} to {}",
                        rates.date,
                        path.display()
                    );
                    return Ok(rates);
                }
                Err(error) => eprintln!("Could not fetch USD rates from {url}: {error}"),
            }
        }

        Err("Could not fetch USD exchange rates from either provider endpoint".into())
    }

    pub fn local_per_usd(&self, currency: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let rate = self
            .usd
            .get(&currency.to_ascii_lowercase())
            .copied()
            .ok_or_else(|| format!("No USD exchange rate for {currency} on {}", self.date))?;

        if !rate.is_finite() || rate <= 0.0 {
            return Err(
                format!("Invalid USD exchange rate for {currency} on {}", self.date).into(),
            );
        }

        Ok(rate)
    }

    fn validate(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.date.is_empty() || self.local_per_usd("USD")? != 1.0 {
            return Err("Invalid USD exchange rate snapshot".into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interprets_rates_as_local_currency_per_usd() {
        let rates: UsdRates =
            serde_json::from_str(r#"{"date":"2026-09-22","usd":{"usd":1,"eur":0.8,"gbp":0}}"#)
                .unwrap();

        rates.validate().unwrap();
        assert_eq!(rates.local_per_usd("EUR").unwrap(), 0.8);
        assert_eq!(rates.local_per_usd("USD").unwrap(), 1.0);
        assert!(rates.local_per_usd("GBP").is_err());
        assert!(rates.local_per_usd("WST").is_err());
    }
}
