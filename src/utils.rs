use std::path::PathBuf;

use anyhow::Context;
use console::style;
use directories::ProjectDirs;
use log::{debug, info};

use crate::config::Config;

pub trait GetConfig {
    fn get_config_path(&self) -> PathBuf;

    fn read_config(&self) -> anyhow::Result<Config> {
        let config_file = self.get_config_path();
        let config = std::fs::read_to_string(config_file).context("Unable to read config file")?;
        let config = toml::from_str::<Config>(&config).context("Unable to parse config file")?;
        debug!("Config Read {:#?}", config);
        Ok(config)
    }

    fn write_config(&self, config: &Config) -> anyhow::Result<()> {
        let config_file = self.get_config_path();
        let toml = toml::to_string_pretty(&config).context("Unable to serialize config")?;
        std::fs::write(&config_file, toml).context("Unable to write config file")?;
        debug!("Config File Updated at {}", config_file.display());
        Ok(())
    }

    fn save_default_config(&self) -> anyhow::Result<()> {
        let config_file = self.get_config_path();
        if !config_file.exists() {
            if !config_file.parent().unwrap().exists() {
                std::fs::create_dir_all(config_file.parent().unwrap())
                    .context("Unable to create config directory")?;
            }
            let config = Config::default();
            let toml = toml::to_string_pretty(&config).context("Unable to serialize config")?;
            std::fs::write(&config_file, toml).context("Unable to write config file")?;
            info!(
                "{}",
                style(format!(
                    "Default Config File Created at {}",
                    config_file.display()
                ))
                .green()
            );
        }
        Ok(())
    }
}

impl GetConfig for ProjectDirs {
    fn get_config_path(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }
}
