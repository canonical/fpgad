use crate::config::system_config;
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
    overlay_control_dir: Option<String>,
    firmware_source_dir: Option<String>,
    fpga_managers_dir: Option<String>,
}

impl From<SystemPaths> for system_config::SystemConfig {
    fn from(value: SystemPaths) -> Self {
        trace!("Creating Config (with Mutex) from {value:?}");
        system_config::SystemConfig {
            overlay_control_dir: Mutex::new(value.overlay_control_dir.unwrap_or_else(|| {
                trace!("No overlay_control_dir provided. Using hardcoded value.");
                system_config::OVERLAY_CONTROL_DIR.to_string()
            })),
            firmware_source_dir: Mutex::new(value.firmware_source_dir.unwrap_or_else(|| {
                trace!("No firmware_source_dir provided. Using hardcoded value.");
                system_config::FIRMWARE_SOURCE_DIR.to_string()
            })),
            fpga_managers_dir: Mutex::new(value.fpga_managers_dir.unwrap_or_else(|| {
                trace!("No fpga_managers_dir provided. Using hardcoded value.");
                system_config::FPGA_MANAGERS_DIR.to_string()
            })),
        }
    }
}

impl SystemPaths {
    pub(crate) fn default() -> SystemPaths {
        SystemPaths {
            overlay_control_dir: None,
            firmware_source_dir: None,
            fpga_managers_dir: None,
        }
    }
    pub(crate) fn merge(self, fallback: SystemPaths) -> SystemPaths {
        SystemPaths {
            overlay_control_dir: self.overlay_control_dir.or(fallback.overlay_control_dir),
            firmware_source_dir: self.firmware_source_dir.or(fallback.firmware_source_dir),
            fpga_managers_dir: self.fpga_managers_dir.or(fallback.fpga_managers_dir),
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
