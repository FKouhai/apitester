use std::collections::HashMap;
use std::time::Duration;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use apitester::config::{
    HttpMethod, ValidatedAssert, ValidatedLoadConfig, ValidatedRequest, ValidatedTestConfig,
};
use apitester::runner::load as load_runner;
use apitester::runner::test as test_runner;

fn make_config(
    server_uri: &str,
    requests: Vec<ValidatedRequest>,
    load: Option<ValidatedLoadConfig>,
) -> ValidatedTestConfig {
    ValidatedTestConfig {
        base_url: Url::parse(server_uri).unwrap(),
        timeout: Duration::from_secs(5),
        requests,
        load,
    }
}

fn make_get_request(name: &str, url: Url, assert: Option<ValidatedAssert>) -> ValidatedRequest {
    ValidatedRequest {
        name: name.to_string(),
        method: HttpMethod::Get,
        url,
        headers: HashMap::new(),
        body: None,
        assert,
    }
}

// --- test runner ---

#[tokio::test]
async fn test_sequential_passes_all_assertions() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let url = Url::parse(&format!("{}/health", server.uri())).unwrap();
    let request = make_get_request(
        "health",
        url,
        Some(ValidatedAssert {
            status: Some(200),
            headers: None,
            body_contains: Some("ok".to_string()),
            body_json: None,
            latency_lt: None,
        }),
    );

    let config = make_config(&server.uri(), vec![request], None);
    let results = test_runner::run(&config, false).await.unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].response.is_ok());
    assert!(results[0].assertions.iter().all(|a| a.passed));
}

#[tokio::test]
async fn test_sequential_fails_status_assertion() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/missing"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let url = Url::parse(&format!("{}/missing", server.uri())).unwrap();
    let request = make_get_request(
        "missing",
        url,
        Some(ValidatedAssert {
            status: Some(200),
            headers: None,
            body_contains: None,
            body_json: None,
            latency_lt: None,
        }),
    );

    let config = make_config(&server.uri(), vec![request], None);
    let results = test_runner::run(&config, false).await.unwrap();

    let status_check = results[0]
        .assertions
        .iter()
        .find(|a| a.rule_name == "Status")
        .unwrap();
    assert!(!status_check.passed);
}

#[tokio::test]
async fn test_sequential_no_assertions_produces_empty_vec() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let url = Url::parse(&format!("{}/ping", server.uri())).unwrap();
    let config = make_config(
        &server.uri(),
        vec![make_get_request("ping", url, None)],
        None,
    );
    let results = test_runner::run(&config, false).await.unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].assertions.is_empty());
    assert!(results[0].response.is_ok());
}

#[tokio::test]
async fn test_sequential_multiple_requests() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let requests = (0..3)
        .map(|i| {
            let url = Url::parse(&format!("{}/req{}", server.uri(), i)).unwrap();
            make_get_request(&format!("req-{}", i), url, None)
        })
        .collect();

    let config = make_config(&server.uri(), requests, None);
    let results = test_runner::run(&config, false).await.unwrap();

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.response.is_ok()));
}

#[tokio::test]
async fn test_parallel_all_succeed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("hello"))
        .mount(&server)
        .await;

    let requests = (0..5)
        .map(|i| {
            let url = Url::parse(&format!("{}/ep{}", server.uri(), i)).unwrap();
            make_get_request(
                &format!("req-{}", i),
                url,
                Some(ValidatedAssert {
                    status: Some(200),
                    headers: None,
                    body_contains: None,
                    body_json: None,
                    latency_lt: None,
                }),
            )
        })
        .collect();

    let config = make_config(&server.uri(), requests, None);
    let results = test_runner::run(&config, true).await.unwrap();

    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|r| r.response.is_ok()));
    assert!(
        results
            .iter()
            .all(|r| r.assertions.iter().all(|a| a.passed))
    );
}

#[tokio::test]
async fn test_body_contains_assertion() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/greet"))
        .respond_with(ResponseTemplate::new(200).set_body_string("hello world"))
        .mount(&server)
        .await;

    let url = Url::parse(&format!("{}/greet", server.uri())).unwrap();
    let request = make_get_request(
        "greet",
        url,
        Some(ValidatedAssert {
            status: None,
            headers: None,
            body_contains: Some("hello".to_string()),
            body_json: None,
            latency_lt: None,
        }),
    );

    let config = make_config(&server.uri(), vec![request], None);
    let results = test_runner::run(&config, false).await.unwrap();

    let check = results[0]
        .assertions
        .iter()
        .find(|a| a.rule_name == "Body Contains")
        .unwrap();
    assert!(check.passed);
}

// --- load runner ---

#[tokio::test]
async fn test_load_runs_for_duration_and_collects_stats() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/load"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let url = Url::parse(&format!("{}/load", server.uri())).unwrap();
    let config = make_config(
        &server.uri(),
        vec![make_get_request("load-req", url, None)],
        Some(ValidatedLoadConfig {
            concurrency: 2,
            duration: Duration::from_millis(300),
            ramp_up: None,
            requests: vec![0],
        }),
    );

    let result = load_runner::run(&config).await.unwrap();

    assert!(result.total_reqs > 0);
    assert_eq!(result.error_count, 0);
    assert!(result.rps > 0.0);
}

#[tokio::test]
async fn test_load_respects_concurrency() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(50)))
        .mount(&server)
        .await;

    let url = Url::parse(&format!("{}/slow", server.uri())).unwrap();
    let config = make_config(
        &server.uri(),
        vec![make_get_request("slow-req", url, None)],
        Some(ValidatedLoadConfig {
            concurrency: 3,
            duration: Duration::from_millis(300),
            ramp_up: None,
            requests: vec![0],
        }),
    );

    let result = load_runner::run(&config).await.unwrap();
    assert!(result.total_reqs > 0);
}

#[tokio::test]
async fn test_load_fails_without_load_config() {
    let server = MockServer::start().await;
    let url = Url::parse(&format!("{}/any", server.uri())).unwrap();
    let config = make_config(
        &server.uri(),
        vec![make_get_request("req", url, None)],
        None,
    );

    let result = load_runner::run(&config).await;
    assert!(result.is_err());
}
