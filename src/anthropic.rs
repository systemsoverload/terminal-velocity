use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::errors::Error;

const API_URL: &str = "https://api.anthropic.com/v1/messages";

#[derive(Debug, Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}

// TODO - replace Error:Other with proper errors

pub async fn generate_outline(prompt: &str, api_key: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(api_key).map_err(|e| Error::Other(e.into()))?,
    );
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let msg = format!(
        "Generate a detailed blog post outline for the following topic. Include main sections and key points to cover: {}",
        prompt
    );

    let request = MessageRequest {
        model: "claude-3-haiku-20240307".to_string(),
        max_tokens: 1000,
        messages: vec![Message {
            role: "user".to_string(),
            content: msg,
        }],
    };

    let response = client
        .post(API_URL)
        .headers(headers)
        .json(&request)
        .send()
        .await
        .map_err(|e| Error::Other(e.into()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(Error::Other(format!("API error: {}", error_text).into()));
    }

    let response: MessageResponse = response.json().await.map_err(|e| Error::Other(e.into()))?;

    Ok(response.content[0].text.clone())
}
