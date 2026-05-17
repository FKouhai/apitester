use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct TestConfig {
    pub base_url: String,
    pub requests: Vec<RequestConfig>,
    pub load: Option<LoadConfig>,
    pub timeout: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct RequestConfig {
    pub name: String,
    #[serde(default)]
    pub method: HttpMethod,
    pub path: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub assert: Option<AssertConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct AssertConfig {
    pub status: Option<u16>,
    pub headers: Option<HashMap<String, String>>,
    pub body_contains: Option<String>,
    pub body_json: Option<serde_yaml::Value>,
    pub latency_lt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct LoadConfig {
    pub concurrency: usize,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub ramp_up: Option<Duration>,
    pub requests: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ValidatedTestConfig {
    pub base_url: Url,
    pub timeout: Duration,
    pub requests: Vec<ValidatedRequest>,
    pub load: Option<ValidatedLoadConfig>,
}

#[derive(Debug, Clone)]
pub struct ValidatedRequest {
    pub name: String,
    pub method: HttpMethod,
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub assert: Option<ValidatedAssert>,
}

#[derive(Debug, Clone)]
pub struct ValidatedAssert {
    pub status: Option<u16>,
    pub headers: Option<HashMap<String, String>>,
    pub body_contains: Option<String>,
    pub body_json: Option<serde_json::Value>,
    pub latency_lt: Option<Duration>,
}

#[derive(Debug)]
pub struct ValidatedLoadConfig {
    pub concurrency: usize,
    pub duration: Duration,
    pub ramp_up: Option<Duration>,
    pub requests: Vec<usize>,
}
