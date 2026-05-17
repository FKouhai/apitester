// Assertion engine

use crate::{config::ValidatedAssert, http::ResponseData};
use serde_json;

#[derive(Debug)]
pub struct CheckResult {
    pub passed: bool,
    pub rule_name: String,
    pub message: String,
}

pub fn check_all(assert: &ValidatedAssert, response: &ResponseData) -> Vec<CheckResult> {
    let mut results = Vec::new();
    if let Some(expected) = assert.status {
        results.push(CheckResult {
            passed: response.status == expected,
            rule_name: "Status".into(),
            message: format!("expected {}, got {}", expected, response.status),
        });
    }

    if let Some(expected) = &assert.body_contains {
        results.push(CheckResult {
            passed: response.body.contains(expected),
            rule_name: "Body Contains".into(),
            message: format!("Expected body to contain '{}'", expected),
        });
    }
    if let Some(expected) = assert.latency_lt {
        results.push(CheckResult {
            passed: response.latency < expected,
            rule_name: "Latency".into(),
            message: format!(
                "Expected latency to be less than {:?} but was {:?}",
                expected, response.latency
            ),
        });
    }
    if let Some(headers) = &assert.headers {
        for (k, v) in headers {
            results.push(CheckResult {
                passed: response.headers.get(k) == Some(v),
                rule_name: format!("Header '{}'", k),
                message: format!("Expected header '{}' to be '{}'", k, v),
            });
        }
    }
    if let Some(expected) = &assert.body_json {
        match serde_json::from_str::<serde_json::Value>(&response.body) {
            Ok(parsed) => {
                let (passed, message) = if let Some(map) = expected.as_object() {
                    let failed: Vec<String> = map
                        .iter()
                        .filter(|(k, v)| parsed.get(*k) != Some(v))
                        .map(|(k, _)| k.clone())
                        .collect();
                    if failed.is_empty() {
                        (true, "all expected keys matched".to_string())
                    } else {
                        (false, format!("keys did not match: {}", failed.join(", ")))
                    }
                } else if parsed == *expected {
                    (true, "body matched".to_string())
                } else {
                    (false, format!("expected {}, got {}", expected, parsed))
                };
                results.push(CheckResult {
                    passed,
                    rule_name: "Body JSON".into(),
                    message,
                });
            }
            Err(_) => {
                results.push(CheckResult {
                    passed: false,
                    rule_name: "Body JSON".into(),
                    message: "Failed to parse body as JSON".into(),
                });
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use super::*;
    #[test]
    fn status_check_passes() {
        let assert = ValidatedAssert {
            status: Some(200),
            headers: None,
            body_contains: None,
            body_json: None,
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::new(),
            body: "Ok".into(),
            latency: Duration::from_millis(100),
        };
        let results = check_all(&assert, &response_data);
        assert!(results[0].passed);
    }

    #[test]
    fn status_check_failed() {
        let assert = ValidatedAssert {
            status: Some(200),
            headers: None,
            body_contains: None,
            body_json: None,
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 404,
            headers: HashMap::new(),
            body: "Ok".into(),
            latency: Duration::from_millis(100),
        };
        let results = check_all(&assert, &response_data);
        assert!(!results[0].passed);
    }
    #[test]
    fn body_contains_passes() {
        let assert = ValidatedAssert {
            status: Some(200),
            headers: None,
            body_contains: Some("Ok".into()),
            body_json: None,
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::new(),
            body: "Ok".into(),
            latency: Duration::from_millis(100),
        };
        let results = check_all(&assert, &response_data);
        assert!(results[0].passed);
    }
    #[test]
    fn body_contains_failed() {
        let assert = ValidatedAssert {
            status: None,
            headers: None,
            body_contains: Some("Ok".into()),
            body_json: None,
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 502,
            headers: HashMap::new(),
            body: "Error".into(),
            latency: Duration::from_millis(100),
        };
        let results = check_all(&assert, &response_data);
        assert!(!results[0].passed);
    }
    #[test]
    fn latency_lt_passes() {
        let assert = ValidatedAssert {
            status: None,
            headers: None,
            body_contains: None,
            body_json: None,
            latency_lt: Some(Duration::from_millis(100)),
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::new(),
            body: "Ok".into(),
            latency: Duration::from_millis(50),
        };
        let results = check_all(&assert, &response_data);
        assert!(results[0].passed);
    }
    #[test]
    fn latency_lt_fails() {
        let assert = ValidatedAssert {
            status: None,
            headers: None,
            body_contains: None,
            body_json: None,
            latency_lt: Some(Duration::from_millis(100)),
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::new(),
            body: "Ok".into(),
            latency: Duration::from_millis(500),
        };
        let results = check_all(&assert, &response_data);
        assert!(!results[0].passed);
    }
    #[test]
    fn header_check_passes() {
        let assert = ValidatedAssert {
            status: None,
            headers: Some(HashMap::from([(
                "content-type".to_string(),
                "application/json".to_string(),
            )])),
            body_contains: None,
            body_json: None,
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: "Ok".into(),
            latency: Duration::from_millis(50),
        };
        let results = check_all(&assert, &response_data);
        assert!(results[0].passed);
    }
    #[test]
    fn header_check_failed() {
        let assert = ValidatedAssert {
            status: None,
            headers: Some(HashMap::from([(
                "content-type".to_string(),
                "application/json".to_string(),
            )])),
            body_contains: None,
            body_json: None,
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::from([("content-type".to_string(), "application/xml".to_string())]),
            body: "Ok".into(),
            latency: Duration::from_millis(50),
        };
        let results = check_all(&assert, &response_data);
        assert!(!results[0].passed);
    }
    #[test]
    fn header_check_missing_key() {
        let assert = ValidatedAssert {
            status: None,
            headers: Some(HashMap::from([(
                "content-type".to_string(),
                "application/json".to_string(),
            )])),
            body_contains: None,
            body_json: None,
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::new(),
            body: "Ok".into(),
            latency: Duration::from_millis(50),
        };
        let results = check_all(&assert, &response_data);
        assert!(!results[0].passed);
    }
    #[test]
    fn body_json_passes() {
        let assert = ValidatedAssert {
            status: None,
            headers: None,
            body_contains: None,
            body_json: Some(serde_json::json!({
                "status": "ok",
                "count": 42
            })),
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: r#"{"status":"ok","count":42}"#.into(),
            latency: Duration::from_millis(50),
        };
        let results = check_all(&assert, &response_data);
        assert!(results[0].passed);
    }
    #[test]
    fn body_json_passes_subset() {
        let assert = ValidatedAssert {
            status: None,
            headers: None,
            body_contains: None,
            body_json: Some(serde_json::json!({
                "status": "ok",
                "count": 42
            })),
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: r#"{"status":"ok","count":42, "response":"hello","proto":"mtls"}"#.into(),
            latency: Duration::from_millis(50),
        };
        let results = check_all(&assert, &response_data);
        assert!(results[0].passed);
    }
    #[test]
    fn body_json_fails() {
        let assert = ValidatedAssert {
            status: None,
            headers: None,
            body_contains: None,
            body_json: Some(serde_json::json!({
                "status": "ok",
                "count": 42
            })),
            latency_lt: None,
        };
        let response_data = ResponseData {
            status: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: r#"{"status":"not ok","count":0}"#.into(),
            latency: Duration::from_millis(50),
        };
        let results = check_all(&assert, &response_data);
        assert!(!results[0].passed);
    }
}
