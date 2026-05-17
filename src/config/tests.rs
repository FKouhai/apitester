use std::time::Duration;

use super::error::ConfigError;
use super::types::*;

#[test]
fn parses_basic_yaml() {
    let yaml = r#"
base_url: https://api.example.com
timeout: 10s
requests:
  - name: health
    method: GET
    path: /health
    assert:
      status: 200
"#;
    let config: TestConfig = serde_yaml::from_str(yaml).unwrap();
    let validated = config.validate().unwrap();
    assert_eq!(validated.base_url.as_str(), "https://api.example.com/");
    assert_eq!(validated.timeout, Duration::from_secs(10));
    assert_eq!(validated.requests.len(), 1);
    assert_eq!(validated.requests[0].name, "health");
    assert_eq!(
        validated.requests[0].url.as_str(),
        "https://api.example.com/health"
    );
}

#[test]
fn parses_fixture_file() {
    let path = format!("{}/tests/fixtures/basic.yaml", env!("CARGO_MANIFEST_DIR"));
    let config = TestConfig::from_file(&path).unwrap();
    let validated = config.validate().unwrap();
    assert!(validated.requests[0].url.as_str().ends_with("/health"));
}

#[test]
fn defaults_method_to_get() {
    let yaml = r#"
base_url: https://example.com
requests:
  - name: x
    path: /x
"#;
    let config: TestConfig = serde_yaml::from_str(yaml).unwrap();
    let validated = config.validate().unwrap();
    assert!(matches!(validated.requests[0].method, HttpMethod::Get));
}

#[test]
fn defaults_timeout_to_30s() {
    let yaml = r#"
base_url: https://example.com
requests:
  - name: x
    path: /x
"#;
    let config: TestConfig = serde_yaml::from_str(yaml).unwrap();
    let validated = config.validate().unwrap();
    assert_eq!(validated.timeout, Duration::from_secs(30));
}

#[test]
fn rejects_unknown_fields() {
    let yaml = "base_url: https://x.com\nrequests: []\nfoobar: true";
    let err = serde_yaml::from_str::<TestConfig>(yaml).unwrap_err();
    assert!(err.to_string().contains("foobar"));
}

#[test]
fn rejects_invalid_url() {
    let yaml = r#"
base_url: not-a-url
requests:
  - name: x
    path: /x
"#;
    let config: TestConfig = serde_yaml::from_str(yaml).unwrap();
    let err = config.validate().unwrap_err();
    assert!(matches!(err, ConfigError::InvalidUrl(..)));
}

#[test]
fn rejects_unknown_load_request() {
    let yaml = r#"
base_url: https://x.com
requests:
  - name: a
    path: /a
load:
  concurrency: 1
  duration: 5s
  requests:
    - nonexistent
"#;
    let config: TestConfig = serde_yaml::from_str(yaml).unwrap();
    let err = config.validate().unwrap_err();
    assert!(matches!(err, ConfigError::UnknownLoadRequest(..)));
    assert!(err.to_string().contains("nonexistent"));
}

#[test]
fn load_defaults_to_all_requests() {
    let yaml = r#"
base_url: https://x.com
requests:
  - name: a
    path: /a
  - name: b
    path: /b
load:
  concurrency: 1
  duration: 5s
"#;
    let config: TestConfig = serde_yaml::from_str(yaml).unwrap();
    let validated = config.validate().unwrap();
    assert_eq!(validated.load.unwrap().requests, vec![0, 1]);
}

#[test]
fn parses_latency_assertion() {
    let yaml = r#"
base_url: https://example.com
requests:
  - name: fast
    path: /fast
    assert:
      latency_lt: 500ms
"#;
    let config: TestConfig = serde_yaml::from_str(yaml).unwrap();
    let validated = config.validate().unwrap();
    let assert = validated.requests[0].assert.as_ref().unwrap();
    assert_eq!(assert.latency_lt, Some(Duration::from_millis(500)));
}

#[test]
fn parses_body_json_assertion() {
    let yaml = r#"
base_url: https://example.com
requests:
  - name: check
    path: /check
    assert:
      body_json:
        status: ok
        count: 42
"#;
    let config: TestConfig = serde_yaml::from_str(yaml).unwrap();
    let validated = config.validate().unwrap();
    let assert = validated.requests[0].assert.as_ref().unwrap();
    let json = assert.body_json.as_ref().unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["count"], 42);
}
