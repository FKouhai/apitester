use std::time::Duration;
use url::Url;

use super::error::ConfigError;
use super::types::*;

pub fn parse_duration(raw: &str) -> Result<Duration, ConfigError> {
    humantime::parse_duration(raw).map_err(|_| ConfigError::InvalidDuration(raw.to_string()))
}

impl TestConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn validate(self) -> Result<ValidatedTestConfig, ConfigError> {
        let TestConfig {
            base_url,
            requests,
            load,
            timeout,
        } = self;

        let base_url =
            Url::parse(&base_url).map_err(|e| ConfigError::InvalidUrl(base_url.clone(), e))?;

        let timeout = parse_duration(timeout.as_deref().unwrap_or("30s"))?;

        let validated_requests: Vec<ValidatedRequest> = requests
            .into_iter()
            .map(|r| r.validate(&base_url))
            .collect::<Result<_, _>>()?;

        let validated_load = load.map(|l| l.validate(&validated_requests)).transpose()?;

        Ok(ValidatedTestConfig {
            base_url,
            timeout,
            requests: validated_requests,
            load: validated_load,
        })
    }
}

impl RequestConfig {
    fn validate(self, base_url: &Url) -> Result<ValidatedRequest, ConfigError> {
        let url = base_url
            .join(&self.path)
            .map_err(|e| ConfigError::InvalidUrl(self.path.clone(), e))?;

        let validated_assert = self.assert.map(|a| a.validate()).transpose()?;

        Ok(ValidatedRequest {
            name: self.name,
            method: self.method,
            url,
            headers: self.headers,
            body: self.body,
            assert: validated_assert,
        })
    }
}

impl AssertConfig {
    fn validate(self) -> Result<ValidatedAssert, ConfigError> {
        let latency_lt = self.latency_lt.as_deref().map(parse_duration).transpose()?;

        let body_json = self
            .body_json
            .map(|yaml_val| serde_json::to_value(&yaml_val).map_err(ConfigError::from))
            .transpose()?;

        Ok(ValidatedAssert {
            status: self.status,
            headers: self.headers,
            body_contains: self.body_contains,
            body_json,
            latency_lt,
        })
    }
}

impl LoadConfig {
    fn validate(self, requests: &[ValidatedRequest]) -> Result<ValidatedLoadConfig, ConfigError> {
        let indices = match self.requests {
            Some(names) => names
                .into_iter()
                .map(|name| {
                    requests
                        .iter()
                        .position(|r| r.name == name)
                        .ok_or(ConfigError::UnknownLoadRequest(name))
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => (0..requests.len()).collect(),
        };

        Ok(ValidatedLoadConfig {
            concurrency: self.concurrency,
            duration: self.duration,
            ramp_up: self.ramp_up,
            requests: indices,
        })
    }
}
