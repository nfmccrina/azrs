use crate::http::AzRsHttpClient;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

#[derive(Deserialize, Debug)]
pub struct ConfigurationSetting {
    pub key: String,
    pub label: Option<String>,
    pub content_type: Option<String>,
    pub value: String,
    pub last_modified: String,
    pub locked: bool,
    pub tags: HashMap<String, String>,
    pub etag: String,
}

pub struct AppConfigurationClient {
    http_client: Box<dyn AzRsHttpClient>,
}

impl AppConfigurationClient {
    pub fn new(http_client: impl AzRsHttpClient + 'static) -> Self {
        AppConfigurationClient {
            http_client: Box::new(http_client),
        }
    }

    pub async fn get_configuration_setting(
        &self,
        key: &str,
        label: Option<String>,
    ) -> Result<ConfigurationSetting, Box<dyn Error>> {
        let path = String::from("/kv/") + key;
        Ok(serde_json::from_str(
            &self
                .http_client
                .get(
                    &path,
                    match label {
                        Some(l) => Some(vec![(String::from("label"), l)]),
                        None => Option::None,
                    },
                )
                .await?,
        )?)
    }
}
