use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("invalid URL '{0}': {1}")]
    InvalidUrl(String, #[source] url::ParseError),
    #[error("invalid duration '{0}'")]
    InvalidDuration(String),
    #[error("duplicate request name: '{0}'")]
    DuplicateRequestName(String),
    #[error("load references unknown request: '{0}'")]
    UnknownLoadRequest(String),
    #[error("invalid body_json value: {0}")]
    InvalidBodyJson(#[from] serde_json::Error),
}
