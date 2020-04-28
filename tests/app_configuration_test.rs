use async_std::task;
use azrs_lib::app_configuration::AppConfigurationClient;
use azrs_lib::http::AzRsHttpClientSurf;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[test]
fn it_gets_a_configuration_setting_with_surf_client() {
    let cfg: Config = confy::load_path("azrs.toml").unwrap();

    task::block_on(async {
        let http_client =
            AzRsHttpClientSurf::new(cfg.host, cfg.api_version, cfg.credential, cfg.secret);

        let app_config_client = AppConfigurationClient::new(http_client);
        let response = app_config_client
            .get_configuration_setting("Azure:IntervalSecs", Option::None)
            .await
            .unwrap();

        assert_eq!("3", response.value);
    })
}

#[derive(Serialize, Deserialize)]
struct Config {
    host: String,
    credential: String,
    secret: String,
    api_version: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: String::from(""),
            credential: String::from(""),
            secret: String::from(""),
            api_version: String::from(""),
        }
    }
}

#[test]
fn it_gets_config() {
    let cfg: Config = confy::load_path("azrs.toml").unwrap();

    assert_eq!("1.0", cfg.api_version);
}
