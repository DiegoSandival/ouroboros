use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub data_path: PathBuf,
    pub max_records: u32,
    #[serde(default)]
    pub sync_writes: bool,
}

impl Config {
    pub const DEFAULT_FILE_NAME: &'static str = "ouroboros.toml";

    pub fn load_default() -> Result<Self> {
        let config_path = env::current_dir()?.join(Self::DEFAULT_FILE_NAME);
        Self::from_path(config_path)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::ConfigNotFound(path.to_path_buf()));
        }

        let raw = fs::read_to_string(path)?;
        let mut config: Self = toml::from_str(&raw)?;

        if config.max_records == 0 {
            return Err(Error::InvalidConfig("max_records must be greater than zero"));
        }

        if config.data_path.as_os_str().is_empty() {
            return Err(Error::InvalidConfig("data_path must not be empty"));
        }

        if config.data_path.is_relative() {
            let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
            config.data_path = base_dir.join(&config.data_path);
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::Config;

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ouroboros-config-{unique}"));
        fs::create_dir_all(&path).expect("temporary directory should be created");
        path
    }

    #[test]
    fn resolve_relative_data_path_from_config_file() {
        let directory = temp_dir();
        let config_path = directory.join(Config::DEFAULT_FILE_NAME);
        fs::write(&config_path, "data_path = \"ring.db\"\nmax_records = 8\n")
            .expect("config file should be written");

        let config = Config::from_path(&config_path).expect("config should load");
        assert_eq!(config.max_records, 8);
        assert_eq!(config.data_path, directory.join("ring.db"));
        assert!(!config.sync_writes);

        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }

    #[test]
    fn read_sync_writes_when_provided() {
        let directory = temp_dir();
        let config_path = directory.join(Config::DEFAULT_FILE_NAME);
        fs::write(
            &config_path,
            "data_path = \"ring.db\"\nmax_records = 8\nsync_writes = true\n",
        )
        .expect("config file should be written");

        let config = Config::from_path(&config_path).expect("config should load");
        assert!(config.sync_writes);

        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }
}