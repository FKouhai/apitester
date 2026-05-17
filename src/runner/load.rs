use std::sync::Arc;

use crate::{config::ValidatedTestConfig, http, stats::Stats};

// Load test runner
pub struct LoadResult {
    pub stats: Stats,
    pub total_reqs: u64,
    pub error_count: u64,
    pub rps: f64,
}

pub async fn run(config: &ValidatedTestConfig) -> Result<LoadResult, anyhow::Error> {
    let now = std::time::Instant::now();
    let load_config = config
        .load
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no load config"))?;
    let client = Arc::new(http::build_client(config)?);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(load_config.concurrency));
    let mut stats = Stats::new();
    let mut i = 0;
    let mut handles = Vec::new();
    let mut error_count: u64 = 0;
    let mut total_reqs: u64 = 0;

    while now.elapsed() < load_config.duration {
        let req_idx = load_config.requests[i % load_config.requests.len()];
        let req = config.requests[req_idx].clone();
        i += 1;
        let client = Arc::clone(&client);
        let sem = Arc::clone(&semaphore);
        let permit = sem.acquire_owned().await.unwrap();
        handles.push(tokio::spawn(async move {
            let _permit = permit;
            http::execute(&req, &client).await
        }));
    }
    for handle in handles {
        match handle.await? {
            Ok(response) => {
                stats.record(response.latency);
                total_reqs += 1;
            }
            Err(_) => {
                total_reqs += 1;
                error_count += 1;
            }
        }
    }
    Ok(LoadResult {
        stats,
        total_reqs,
        error_count,
        rps: total_reqs as f64 / now.elapsed().as_secs_f64(),
    })
}
