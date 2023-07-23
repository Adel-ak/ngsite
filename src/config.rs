use crate::utils::merge_config;
use anyhow::Result;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;

const DEFAULT_CONFIG_STR: &str = include_str!("../default_config.toml");

fn init_config() -> Result<Config> {
    let mut default_config: Value = toml::from_str(DEFAULT_CONFIG_STR)?;

    if let Some(proj_dirs) = ProjectDirs::from_path("Ngsite".into()) {
        let config_dir = proj_dirs.config_dir();

        let config_file = fs::read_to_string(config_dir.join("config.toml"));

        let user_config: Value = match config_file {
            Ok(file) => toml::from_str(&file)?,
            Err(_) => json!({}),
        };

        merge_config(&mut default_config, user_config);

        // Linux:   /home/alice/.config/barapp
        // Windows: C:\Users\Alice\AppData\Roaming\Foo Corp\Bar App
        // macOS:   /Users/Alice/Library/Application Support/com.Foo-Corp.Bar-App
    }

    let config = serde_json::from_value(default_config)?;

    Ok(config)
}

lazy_static! {
    pub(crate) static ref CONFIG: Config = init_config().unwrap();
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Config {
    pub paths: Paths,
    pub ignore_values_in_log: Vec<String>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Paths {
    pub nginx: String,
    pub sites_available: String,
    pub sites_enabled: String,
    pub logs: String,
}
