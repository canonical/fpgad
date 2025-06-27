use crate::config::system_config::{CONFIG_FS_PREFIX, FW_PREFIX, SYSFS_PREFIX, SystemConfig};
use crate::error::FpgadError;
use crate::system_io::fs_read;
use log::trace;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Mutex;

/// This is the top level struct which holds all sections
#[derive(Debug, Deserialize)]
pub(crate) struct TomlConfig {
    defaults: Option<SystemPaths>,
}

/// This is the "defaults" section struct
#[derive(Debug, Deserialize)]
pub(crate) struct SystemPaths {
    config_fs_prefix: Option<String>,
    firmware_prefix: Option<String>,
    sys_fs_prefix: Option<String>,
}

impl From<SystemPaths> for SystemConfig {
    fn from(value: SystemPaths) -> Self {
        trace!("Creating Config (with Mutex) from {value:?}");
        SystemConfig {
            config_fs_prefix: Mutex::new(value.config_fs_prefix.unwrap_or_else(|| {
                trace!("No config_fs_prefix provided. Using hardcoded value.");
                CONFIG_FS_PREFIX.to_string()
            })),
            firmware_prefix: Mutex::new(value.firmware_prefix.unwrap_or_else(|| {
                trace!("No firmware_prefix provided. Using hardcoded value.");
                FW_PREFIX.to_string()
            })),
            sys_fs_prefix: Mutex::new(value.sys_fs_prefix.unwrap_or_else(|| {
                trace!("No sys_fs_prefix provided. Using hardcoded value.");
                SYSFS_PREFIX.to_string()
            })),
        }
    }
}

impl SystemPaths {
    pub(crate) fn default() -> SystemPaths {
        SystemPaths {
            config_fs_prefix: None,
            firmware_prefix: None,
            sys_fs_prefix: None,
        }
    }
    pub(crate) fn merge(self, fallback: SystemPaths) -> SystemPaths {
        SystemPaths {
            config_fs_prefix: self.config_fs_prefix.or(fallback.config_fs_prefix),
            firmware_prefix: self.firmware_prefix.or(fallback.firmware_prefix),
            sys_fs_prefix: self.sys_fs_prefix.or(fallback.sys_fs_prefix),
        }
    }
}

pub(crate) fn system_paths_config_from_file(
    file_path: &PathBuf,
) -> Result<SystemPaths, FpgadError> {
    if !file_path.is_file() {
        return Err(FpgadError::Internal(format!(
            "Config file not found in {file_path:?}"
        )));
    }
    let toml_string = fs_read(file_path)?;

    let config: TomlConfig = match toml::from_str(&toml_string) {
        Ok(config) => config,
        Err(e) => {
            return Err(FpgadError::TomlDe {
                toml_string: toml_string.clone(),
                e,
            });
        }
    };
    match config.defaults {
        Some(defaults_toml) => Ok(defaults_toml),
        None => Err(FpgadError::Internal(
            "config file did not contain a `[defaults]` section.".to_string(),
        )),
    }
}
