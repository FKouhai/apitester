use colored::Colorize;
use serde_json::json;

use crate::runner::{load::LoadResult, test::RequestResult};

pub fn print_test_results(results: &[RequestResult], format: &str) {
    println!("{}", format_test_results(results, format));
}

pub fn print_load_results(result: &LoadResult, format: &str) {
    println!("{}", format_load_results(result, format));
}

pub fn format_test_results(results: &[RequestResult], format: &str) -> String {
    match format {
        "json" => format_json(results),
        "tap" => format_tap(results),
        "terminal" => format_terminal(results),
        _ => format!("unknown format {}", format),
    }
}

pub fn format_load_results(result: &LoadResult, format: &str) -> String {
    match format {
        "json" => format_load_json(result),
        "tap" => format_load_tap(result),
        "terminal" => format_load_terminal(result),
        _ => format!("unknown format {}", format),
    }
}

fn format_load_terminal(result: &LoadResult) -> String {
    format!(
        "\n{}\n  total requests: {}\n  errors:         {}\n  rps:            {:.2}\n  p50:            {:?}\n  p90:            {:?}\n  p99:            {:?}",
        "Load Test Results".bold(),
        result.total_reqs,
        result.error_count,
        result.rps,
        result.stats.p50(),
        result.stats.p90(),
        result.stats.p99(),
    )
}

fn format_load_json(result: &LoadResult) -> String {
    let output = json!({
        "total_requests": result.total_reqs,
        "errors": result.error_count,
        "rps": result.rps,
        "p50_ms": result.stats.p50().as_millis(),
        "p90_ms": result.stats.p90().as_millis(),
        "p99_ms": result.stats.p99().as_millis(),
    });
    serde_json::to_string_pretty(&output).unwrap()
}

fn format_load_tap(result: &LoadResult) -> String {
    if result.error_count == 0 {
        format!(
            "TAP version 14\n1..1\nok 1 - load test ({} requests, {:.2} rps)",
            result.total_reqs, result.rps
        )
    } else {
        format!(
            "TAP version 14\n1..1\nnot ok 1 - load test ({} errors out of {} requests)",
            result.error_count, result.total_reqs
        )
    }
}

fn format_terminal(results: &[RequestResult]) -> String {
    let mut out = String::new();
    for result in results {
        out.push_str(&format!("\n{}\n", result.name.bold()));
        match &result.response {
            Err(e) => {
                out.push_str(&format!(
                    "  {} network error: {}\n",
                    "ERROR".red().bold(),
                    e
                ));
            }
            Ok(response) => {
                out.push_str(&format!("  duration: {:?}\n", response.latency));
                if result.assertions.is_empty() {
                    out.push_str(&format!("  {} (no assertions)\n", "OK".cyan()));
                } else {
                    for check in &result.assertions {
                        let label = if check.passed {
                            "PASS".green().bold()
                        } else {
                            "FAIL".red().bold()
                        };
                        out.push_str(&format!(
                            "  {} [{}] {}\n",
                            label, check.rule_name, check.message
                        ));
                    }
                }
            }
        }
    }
    out
}

fn format_json(results: &[RequestResult]) -> String {
    let output: Vec<_> = results
        .iter()
        .map(|r| {
            let (status, latency_ms, assertions) = match &r.response {
                Ok(resp) => (
                    resp.status,
                    resp.latency.as_millis(),
                    r.assertions
                        .iter()
                        .map(|a| {
                            json!({
                                "rule": a.rule_name,
                                "passed": a.passed,
                                "message": a.message,
                            })
                        })
                        .collect::<Vec<_>>(),
                ),
                Err(_) => (0, 0, vec![]),
            };
            let error = r.response.as_ref().err().map(|e| e.to_string());
            json!({
                "name": r.name,
                "status": status,
                "latency_ms": latency_ms,
                "error": error,
                "assertions": assertions,
            })
        })
        .collect();
    serde_json::to_string_pretty(&output).unwrap()
}

fn format_tap(results: &[RequestResult]) -> String {
    let mut out = format!("TAP version 14\n1..{}\n", results.len());
    for (i, result) in results.iter().enumerate() {
        let n = i + 1;
        let all_passed = result.response.is_ok() && result.assertions.iter().all(|a| a.passed);
        if all_passed {
            out.push_str(&format!("ok {} - {}\n", n, result.name));
        } else {
            out.push_str(&format!("not ok {} - {}\n", n, result.name));
            for check in &result.assertions {
                if !check.passed {
                    out.push_str(&format!(
                        "  # FAIL [{}] {}\n",
                        check.rule_name, check.message
                    ));
                }
            }
            if let Err(e) = &result.response {
                out.push_str(&format!("  # ERROR {}\n", e));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use super::*;
    use crate::{assert::CheckResult, http::ResponseData, runner::load::LoadResult, stats::Stats};

    fn make_response(status: u16) -> ResponseData {
        ResponseData {
            status,
            headers: HashMap::new(),
            body: "ok".into(),
            latency: Duration::from_millis(42),
        }
    }

    fn make_result(name: &str, status: u16, assertions: Vec<CheckResult>) -> RequestResult {
        RequestResult {
            name: name.to_string(),
            response: Ok(make_response(status)),
            assertions,
        }
    }

    fn passing_check() -> CheckResult {
        CheckResult {
            passed: true,
            rule_name: "Status".into(),
            message: "expected 200, got 200".into(),
        }
    }

    fn failing_check() -> CheckResult {
        CheckResult {
            passed: false,
            rule_name: "Status".into(),
            message: "expected 200, got 404".into(),
        }
    }

    // --- terminal ---

    #[test]
    fn terminal_shows_request_name() {
        colored::control::set_override(false);
        let results = vec![make_result("my request", 200, vec![])];
        let out = format_test_results(&results, "terminal");
        assert!(out.contains("my request"));
    }

    #[test]
    fn terminal_shows_pass_for_passing_assertion() {
        colored::control::set_override(false);
        let results = vec![make_result("req", 200, vec![passing_check()])];
        let out = format_test_results(&results, "terminal");
        assert!(out.contains("PASS"));
    }

    #[test]
    fn terminal_shows_fail_for_failing_assertion() {
        colored::control::set_override(false);
        let results = vec![make_result("req", 404, vec![failing_check()])];
        let out = format_test_results(&results, "terminal");
        assert!(out.contains("FAIL"));
    }

    #[test]
    fn terminal_shows_no_assertions_when_empty() {
        colored::control::set_override(false);
        let results = vec![make_result("req", 200, vec![])];
        let out = format_test_results(&results, "terminal");
        assert!(out.contains("no assertions"));
    }

    // --- json ---

    #[test]
    fn json_output_is_valid_json() {
        let results = vec![make_result("req", 200, vec![passing_check()])];
        let out = format_test_results(&results, "json");
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed[0]["name"], "req");
        assert_eq!(parsed[0]["status"], 200);
    }

    #[test]
    fn json_includes_assertions() {
        let results = vec![make_result("req", 200, vec![passing_check()])];
        let out = format_test_results(&results, "json");
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed[0]["assertions"][0]["passed"], true);
    }

    #[test]
    fn json_empty_results_produces_empty_array() {
        let out = format_test_results(&[], "json");
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed, serde_json::json!([]));
    }

    // --- tap ---

    #[test]
    fn tap_starts_with_version_line() {
        let results = vec![make_result("req", 200, vec![passing_check()])];
        let out = format_test_results(&results, "tap");
        assert!(out.starts_with("TAP version 14"));
    }

    #[test]
    fn tap_ok_for_passing_result() {
        let results = vec![make_result("req", 200, vec![passing_check()])];
        let out = format_test_results(&results, "tap");
        assert!(out.contains("ok 1 - req"));
    }

    #[test]
    fn tap_not_ok_for_failing_result() {
        let results = vec![make_result("req", 404, vec![failing_check()])];
        let out = format_test_results(&results, "tap");
        assert!(out.contains("not ok 1 - req"));
    }

    #[test]
    fn tap_plan_line_matches_result_count() {
        let results = vec![
            make_result("a", 200, vec![]),
            make_result("b", 200, vec![]),
            make_result("c", 200, vec![]),
        ];
        let out = format_test_results(&results, "tap");
        assert!(out.contains("1..3"));
    }

    // --- unknown format ---

    #[test]
    fn unknown_format_returns_error_message() {
        let results = vec![make_result("req", 200, vec![])];
        let out = format_test_results(&results, "xml");
        assert!(out.contains("unknown format"));
    }

    // --- load results ---

    fn make_load_result(total: u64, errors: u64) -> LoadResult {
        LoadResult {
            stats: Stats::new(),
            total_reqs: total,
            error_count: errors,
            rps: total as f64 / 5.0,
        }
    }

    #[test]
    fn load_json_is_valid_json() {
        let result = make_load_result(100, 0);
        let out = format_load_results(&result, "json");
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["total_requests"], 100);
        assert_eq!(parsed["errors"], 0);
    }

    #[test]
    fn load_tap_ok_when_no_errors() {
        let result = make_load_result(100, 0);
        let out = format_load_results(&result, "tap");
        assert!(out.contains("ok 1"));
    }

    #[test]
    fn load_tap_not_ok_when_errors() {
        let result = make_load_result(100, 10);
        let out = format_load_results(&result, "tap");
        assert!(out.contains("not ok 1"));
        assert!(out.contains("10 errors"));
    }

    #[test]
    fn load_terminal_shows_stats() {
        colored::control::set_override(false);
        let result = make_load_result(50, 2);
        let out = format_load_results(&result, "terminal");
        assert!(out.contains("total requests: 50"));
        assert!(out.contains("errors:         2"));
    }
}
