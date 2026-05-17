use std::sync::Arc;

use futures::{StreamExt, stream::FuturesUnordered};

use crate::{
    assert::{CheckResult, check_all},
    config::ValidatedTestConfig,
    http::{self, HttpError, ResponseData},
};

// Functional test runner
//
pub struct RequestResult {
    pub name: String,
    pub response: Result<ResponseData, HttpError>,
    pub assertions: Vec<CheckResult>,
}

pub async fn run(
    config: &ValidatedTestConfig,
    is_parallel: bool,
) -> Result<Vec<RequestResult>, anyhow::Error> {
    let mut results = Vec::new();
    if is_parallel {
        let client = Arc::new(http::build_client(config)?);
        let mut futures = FuturesUnordered::new();
        for req in &config.requests {
            let client = Arc::clone(&client);
            futures.push(async move {
                let res = http::execute(req, &client).await;
                (req.name.clone(), req.assert.clone(), res)
            });
        }
        while let Some((name, assert, res)) = futures.next().await {
            match res {
                Ok(response) => {
                    let assertions = if let Some(a) = &assert {
                        check_all(a, &response)
                    } else {
                        Vec::new()
                    };
                    results.push(RequestResult {
                        name,
                        response: Ok(response),
                        assertions,
                    });
                }
                Err(e) => results.push(RequestResult {
                    name,
                    response: Err(e),
                    assertions: Vec::new(),
                }),
            }
        }
    } else {
        let client = http::build_client(config)?;
        for req in &config.requests {
            let res = http::execute(req, &client).await;
            match res {
                Ok(response) => {
                    let assertions = if let Some(assert) = &req.assert {
                        check_all(assert, &response)
                    } else {
                        Vec::new()
                    };
                    results.push(RequestResult {
                        name: req.name.clone(),
                        response: Ok(response),
                        assertions,
                    });
                }
                Err(e) => {
                    results.push(RequestResult {
                        name: req.name.clone(),
                        response: Err(e),
                        assertions: Vec::new(),
                    });
                }
            }
        }
    }
    Ok(results)
}
