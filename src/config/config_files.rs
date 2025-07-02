use crate::config::system_config;
use crate::error::FpgadError;
use crate::system_io::fs_read;
use log::trace;
use serde::Deserialize;
use std::path::Path;
use std::sync::Mutex;

/// This is the top level struct which holds all sections
#[derive(Debug, Deserialize)]
pub(crate) struct TomlConfig {
    system_paths: Option<SystemPaths>,
    boot_firmware: Option<BootFirmware>,
}

/// This is the "defaults" section struct
#[derive(Debug, Deserialize)]
pub(crate) struct SystemPaths {
    overlay_control_dir: Option<String>,
    firmware_source_dir: Option<String>,
    fpga_managers_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BootFirmware {
    pub(crate) bitstreams: Vec<Bitstream>,
    pub(crate) overlays: Vec<Overlay>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Bitstream {
    pub(crate) device_handle: String,
    pub(crate) bitstream_path: String,
    pub(crate) flags: isize,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Overlay {
    pub(crate) platform: String,
    pub(crate) overlay_handle: String,
    pub(crate) overlay_path: String,
    pub(crate) device_handle: Option<String>,
    pub(crate) fpga_flags: Option<isize>,
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

impl BootFirmware {
    pub(crate) fn default() -> BootFirmware {
        BootFirmware {
            bitstreams: Vec::new(),
            overlays: Vec::new(),
        }
    }
}

fn toml_str_to_config(toml_string: &str) -> Result<TomlConfig, FpgadError> {
    let config: TomlConfig = match toml::from_str(toml_string) {
        Ok(config) => config,
        Err(e) => {
            return Err(FpgadError::TomlDe {
                toml_string: toml_string.into(),
                e,
            });
        }
    };
    Ok(config)
}

pub(crate) fn system_paths_config_from_file(file_path: &Path) -> Result<SystemPaths, FpgadError> {
    if !file_path.is_file() {
        return Err(FpgadError::Internal(format!(
            "Config file not found in {file_path:?}"
        )));
    }
    let config = toml_str_to_config(&fs_read(file_path)?)?;

    match config.system_paths {
        Some(system_paths) => Ok(system_paths),
        None => Err(FpgadError::Internal(
            "config file did not contain a `[system_paths]` section.".to_string(),
        )),
    }
}

pub fn boot_firmware_from_file(file_path: &Path) -> Result<BootFirmware, FpgadError> {
    if !file_path.is_file() {
        return Err(FpgadError::Internal(format!(
            "Config file not found in {file_path:?}"
        )));
    }
    let config = toml_str_to_config(&fs_read(file_path)?)?;
    match config.boot_firmware {
        Some(boot_firmware) => Ok(boot_firmware),
        None => Err(FpgadError::Internal(
            "config file did not contain a `[boot_firmware]` section.".to_string(),
        )),
    }
}
