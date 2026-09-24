use std::error::Error;

use reqwest::Client;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Response {
    data: Option<Value>,
    #[serde(default)]
    errors: Vec<GraphqlError>,
}

#[derive(Deserialize)]
struct GraphqlError {
    message: String,
}

/// Execute a GraphQL document with JSON variables using Bearer authentication.
pub async fn execute<T: DeserializeOwned>(
    client: &Client,
    endpoint: &str,
    token: &str,
    document: &str,
    variables: Value,
) -> Result<T, Box<dyn Error + Send + Sync>> {
    let response: Response = client
        .post(endpoint)
        .bearer_auth(token)
        .json(&json!({ "query": document, "variables": variables }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    response.into_data()
}

impl Response {
    fn into_data<T: DeserializeOwned>(self) -> Result<T, Box<dyn Error + Send + Sync>> {
        if !self.errors.is_empty() {
            let messages = self
                .errors
                .into_iter()
                .map(|error| error.message)
                .collect::<Vec<_>>()
                .join("; ");

            return Err(format!("GraphQL: {messages}").into());
        }

        let data = self.data.ok_or("GraphQL returned no data")?;

        Ok(serde_json::from_value(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_graphql_data() {
        let response: Response =
            serde_json::from_str(r#"{"data":{"entries":[{"title":"Germany"}]}}"#).unwrap();
        let data: Value = response.into_data().unwrap();

        assert_eq!(data["entries"][0]["title"], "Germany");
    }

    #[test]
    fn rejects_graphql_errors_even_when_data_is_present() {
        let response: Response =
            serde_json::from_str(r#"{"data":{},"errors":[{"message":"Not authorized"}]}"#).unwrap();
        let result: Result<Value, _> = response.into_data();

        assert!(result.unwrap_err().to_string().contains("Not authorized"));
    }
}
