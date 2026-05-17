use std::sync::Arc;

use futures::{StreamExt, stream::FuturesUnordered};

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
    let initial = if load_config.ramp_up.is_some() {
        1
    } else {
        load_config.concurrency
    };
    let semaphore = Arc::new(tokio::sync::Semaphore::new(initial));
    let mut stats = Stats::new();
    let mut i = 0;
    let mut handles: FuturesUnordered<
        tokio::task::JoinHandle<Result<http::ResponseData, http::HttpError>>,
    > = FuturesUnordered::new();
    let mut error_count: u64 = 0;
    let mut total_reqs: u64 = 0;
    let mut permits_added: usize = 1;
    loop {
        if let Some(ramp_up) = load_config.ramp_up {
            let current_permits = (now.elapsed().as_secs_f64() / ramp_up.as_secs_f64()).min(1.0)
                * load_config.concurrency as f64;
            let target = current_permits as usize;
            if target > permits_added {
                semaphore.add_permits(target - permits_added);
                permits_added = target;
            }
        }
        let spawning = now.elapsed() < load_config.duration;
        tokio::select! {
            Some(result) = handles.next(), if !handles.is_empty() => {
                match result? {
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
            permit = Arc::clone(&semaphore).acquire_owned(), if spawning => {
                let req_idx = load_config.requests[i % load_config.requests.len()];
                let req = config.requests[req_idx].clone();
                i += 1;
                let client = Arc::clone(&client);
                handles.push(tokio::spawn(async move {
                    let _permit = permit.unwrap();
                    http::execute(&req, &client).await
                }));
            }
            else => break,
        }
    }
    Ok(LoadResult {
        stats,
        total_reqs,
        error_count,
        rps: total_reqs as f64 / now.elapsed().as_secs_f64(),
    })
}
