use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

use crate::config::{HttpMethod, ValidatedRequest, ValidatedTestConfig};

// HTTP client builder and request execution
pub struct ResponseData {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
    pub latency: Duration,
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("request timed out")]
    Timeout,
    #[error("unsupported HTTP method: {0}")]
    UnsupportedMethod(String),
}

pub fn build_client(config: &ValidatedTestConfig) -> Result<reqwest::Client, reqwest::Error> {
    Client::builder().timeout(config.timeout).build()
}

pub async fn execute(
    req: &ValidatedRequest,
    client: &reqwest::Client,
) -> Result<ResponseData, HttpError> {
    let mut builder = match &req.method {
        HttpMethod::Get => client.get(req.url.clone()),
        HttpMethod::Post => client.post(req.url.clone()),
        HttpMethod::Put => client.put(req.url.clone()),
        HttpMethod::Delete => client.delete(req.url.clone()),
        HttpMethod::Patch => client.patch(req.url.clone()),
        HttpMethod::Head => client.head(req.url.clone()),
    };

    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }
    if let Some(body) = &req.body {
        builder = builder.body(body.clone());
    }
    let start = std::time::Instant::now();
    let response = builder.send().await?;
    let latency = start.elapsed();
    let status = response.status().as_u16();
    let headers: std::collections::HashMap<_, _> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let body = response.text().await?;

    Ok(ResponseData {
        status,
        headers,
        body,
        latency,
    })
}
