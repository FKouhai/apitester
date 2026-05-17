use std::sync::Arc;

use futures::{StreamExt, stream::FuturesUnordered};

use crate::{
    assert::{CheckResult, check_all},
    config::{ValidatedAssert, ValidatedTestConfig},
    http::{self, HttpError, ResponseData},
};

// Functional test runner
//
pub struct RequestResult {
    pub name: String,
    pub response: Result<ResponseData, HttpError>,
    pub assertions: Vec<CheckResult>,
}

fn into_request_result(
    name: String,
    assert: Option<&ValidatedAssert>,
    res: Result<ResponseData, HttpError>,
) -> RequestResult {
    match res {
        Ok(response) => {
            let assertions = assert.map_or_else(Vec::new, |a| check_all(a, &response));
            RequestResult {
                name,
                response: Ok(response),
                assertions,
            }
        }
        Err(e) => RequestResult {
            name,
            response: Err(e),
            assertions: Vec::new(),
        },
    }
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
            results.push(into_request_result(name, assert.as_ref(), res));
        }
    } else {
        let client = http::build_client(config)?;
        for req in &config.requests {
            let res = http::execute(req, &client).await;
            results.push(into_request_result(
                req.name.clone(),
                req.assert.as_ref(),
                res,
            ));
        }
    }
    Ok(results)
}
