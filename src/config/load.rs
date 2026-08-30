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
        // What was not understood on the way in is not there to write back
        // out, and writing anyway would make the loss permanent.
        if self.partial {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "part of the configuration file could not be read, so it is not being written over",
            ));
        }

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

        let cfg = Self::from_json(&config_json);

        // The whole configuration is one long line, so it stays out of the
        // way unless something needs explaining.
        tracing::debug!(
            "Using config: {}",
            serde_json::to_string(&cfg).unwrap_or_default()
        );

        cfg
    }

    /// Reads a configuration document one section at a time.
    ///
    /// Section by section rather than in one go, because one section the
    /// viewer cannot make sense of — a key renamed between versions, a number
    /// where a string belongs, a hand edit with a comma missing — used to
    /// discard the whole file and hand back the defaults for everything. A
    /// section that is simply absent is not a problem: it is what a file
    /// written by an older build looks like, and the defaults are the right
    /// answer.
    pub fn from_json(document: &str) -> Config {
        let map: serde_json::Map<String, serde_json::Value> = match serde_json::from_str(document) {
            Ok(map) => map,
            Err(e) => {
                tracing::error!("The configuration file could not be read at all: {e}");
                return Config {
                    partial: true,
                    ..Config::default()
                };
            }
        };

        let mut partial = false;

        Config {
            image_view: section(&map, "image_view", &mut partial),
            grid_view: section(&map, "grid_view", &mut partial),
            general: section(&map, "general", &mut partial),
            cache: section(&map, "cache", &mut partial),
            slideshow: section(&map, "slideshow", &mut partial),
            tags: section(&map, "tags", &mut partial),
            raw: section(&map, "raw", &mut partial),
            partial,
        }
    }
}

/// One section of the document, or its defaults when it cannot be read.
fn section<T: serde::de::DeserializeOwned + Default>(
    map: &serde_json::Map<String, serde_json::Value>,
    name: &str,
    partial: &mut bool,
) -> T {
    let Some(value) = map.get(name) else {
        return T::default();
    };

    match serde_json::from_value(value.clone()) {
        Ok(parsed) => parsed,
        Err(e) => {
            tracing::error!("Ignoring the \"{name}\" section of the configuration: {e}");
            *partial = true;
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::GridViewConfig;
    use super::*;

    #[test]
    fn a_complete_file_is_read_whole() {
        let json = serde_json::to_string(&Config::default()).unwrap();
        let cfg = Config::from_json(&json);

        assert!(!cfg.partial);
    }

    /// A file written by an older build has sections the newer one added.
    #[test]
    fn a_missing_section_costs_nothing() {
        let cfg = Config::from_json(r#"{"general": {}}"#);

        assert!(!cfg.partial);
        assert_eq!(
            cfg.grid_view.images_per_row,
            GridViewConfig::default().images_per_row
        );
    }

    /// The case that used to discard the file: one section the viewer cannot
    /// make sense of.
    #[test]
    fn one_bad_section_costs_only_that_section() {
        let cfg = Config::from_json(
            r#"{"grid_view": "not an object", "general": {"text_scaling": 2.5}}"#,
        );

        assert!(cfg.partial);
        assert_eq!(cfg.general.text_scaling, 2.5);
        assert_eq!(
            cfg.grid_view.images_per_row,
            GridViewConfig::default().images_per_row
        );
    }

    #[test]
    fn a_file_that_was_not_understood_is_never_written_back() {
        let cfg = Config {
            partial: true,
            ..Config::default()
        };

        assert!(cfg.save().is_err());
    }

    #[test]
    fn a_document_that_is_not_json_at_all_is_partial() {
        assert!(Config::from_json("not json").partial);
    }
}
