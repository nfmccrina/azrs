use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Serialize, Deserialize)]
pub struct AzRsConfig {
    pub http_config: HttpConfig,
}

#[derive(Serialize, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub credential: String,
    pub secret: String,
    pub api_version: String,
}

impl Default for AzRsConfig {
    fn default() -> Self {
        Self {
            http_config: HttpConfig {
                host: String::from(""),
                credential: String::from(""),
                secret: String::from(""),
                api_version: String::from(""),
            },
        }
    }
}
