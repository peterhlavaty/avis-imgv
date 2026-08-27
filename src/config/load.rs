//! Finding, reading and creating the configuration file.

use std::{fs, io::ErrorKind, path::PathBuf};

use super::Config;
use crate::{APPLICATION, ORGANIZATION, QUALIFIER};

impl Config {
    pub fn new() -> Config {
        Self::fetch_cfg()
    }

    /// Where the configuration file lives, whether or not it exists yet.
    pub fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|dirs| dirs.config_dir().join("config.json"))
    }

    /// Writes the configuration back out.
    ///
    /// Pretty printed, unlike the one line the viewer used to write on first
    /// run: a file people are now edited from inside the viewer is a file they
    /// will also want to read.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path().ok_or_else(|| {
            std::io::Error::new(ErrorKind::NotFound, "no configuration directory")
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

        fs::write(&path, json)?;
        tracing::info!("Wrote config -> {}", path.display());

        Ok(())
    }

    pub fn fetch_cfg() -> Config {
        let config_dir = match directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        {
            Some(dirs) => dirs.config_dir().to_owned(),
            None => return Config::default(),
        };

        let cfg_path = config_dir.join(PathBuf::from("config.json"));
        tracing::info!("Reading config -> {}", cfg_path.display());

        let config_json = match fs::read_to_string(&cfg_path) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!("Failure reading config file -> {e}");
                let default_cfg = Config::default();

                if e.kind() == ErrorKind::NotFound {
                    tracing::info!("Config file does not exist -> creating default config");
                    let default_cfg_json = match serde_json::to_string(&default_cfg) {
                        Ok(json) => json,
                        Err(e) => {
                            tracing::error!("Failure serializing default cfg -> {e}");
                            return default_cfg;
                        }
                    };

                    if !config_dir.exists() {
                        tracing::info!("Config directory does not exist, creating");
                        if let Err(e) = fs::create_dir_all(&config_dir) {
                            tracing::error!("Failure creating config directory {:?}", e);
                        }
                    }

                    match fs::write(&cfg_path, default_cfg_json) {
                        Ok(_) => {}
                        Err(e) => tracing::error!("Failure writing default config file -> {e}"),
                    };
                }
                return default_cfg;
            }
        };

        let cfg = match serde_json::from_str(&config_json) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::error!("{e}");
                tracing::error!("Failure parsing config json, using defaults");
                Config::default()
            }
        };

        // The whole configuration is one long line, so it stays out of the
        // way unless something needs explaining.
        tracing::debug!(
            "Using config: {}",
            serde_json::to_string(&cfg).unwrap_or_default()
        );

        cfg
    }
}
