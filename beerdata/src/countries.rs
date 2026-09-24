use std::{error::Error, time::Duration};

use reqwest::Client;
use scraper::{Html, Selector};

const COUNTRIES_URL: &str = "https://www.numbeo.com/cost-of-living/";

/// Fetch the country names listed in Numbeo's cost-of-living country selector.
pub async fn fetch() -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let html = client
        .get(COUNTRIES_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let countries = country_names(&html);

    if countries.is_empty() {
        return Err("Numbeo did not provide a country list".into());
    }

    Ok(countries)
}

fn country_names(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("select#country option").expect("valid country selector");

    document
        .select(&selector)
        .filter_map(|option| option.value().attr("value"))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_numbeo_country_options() {
        let html = r#"<select id="country">
            <option value="">---Select Country---</option>
            <option value="Germany">Germany</option>
            <option value="Trinidad And Tobago">Trinidad And Tobago</option>
        </select>"#;

        assert_eq!(country_names(html), ["Germany", "Trinidad And Tobago"]);
    }
}
