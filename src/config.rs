use crate::error::FpgadError;
use crate::system_io::fs_read;
use log::{trace, warn};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

// These are hardcoded backups to prevent crashing and lockups when accessing the config file
// or Mutex values
pub static CONFIG_FS_PREFIX: &str = "/sys/kernel/config/device-tree/overlays/";
pub static FW_PREFIX: &str = "/lib/firmware/";
pub static SYSFS_PREFIX: &str = "/sys/class/fpga_manager/";

#[derive(Debug)]
pub struct SystemConfig {
    config_fs_prefix: Mutex<String>,
    firmware_prefix: Mutex<String>,
    sys_fs_prefix: Mutex<String>,
}

static CONFIG: OnceLock<Mutex<SystemConfig>> = OnceLock::new();

impl SystemConfig {
    fn config_fs_prefix(&self) -> Result<String, FpgadError> {
        let guard = match self.config_fs_prefix.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(FpgadError::Internal(format!(
                    "Failed when locking config_fs_prefix for read access: {e}"
                )));
            }
        };
        Ok(guard.clone())
    }

    fn firmware_prefix(&self) -> Result<String, FpgadError> {
        let guard = match self.firmware_prefix.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(FpgadError::Internal(format!(
                    "Failed when locking firmware_prefix for read access: {e}"
                )));
            }
        };

        Ok(guard.clone())
    }

    fn sys_fs_prefix(&self) -> Result<String, FpgadError> {
        let guard = match self.sys_fs_prefix.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(FpgadError::Internal(format!(
                    "Failed when locking sys_fs_prefix for read access: {e}"
                )));
            }
        };
        Ok(guard.clone())
    }
}
/// This is the top level struct which holds all sections
#[derive(Debug, Deserialize)]
struct TomlConfig {
    system_paths: Option<SystemPaths>,
}

/// This is the "defaults" struct
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
    fn default() -> SystemPaths {
        SystemPaths {
            config_fs_prefix: None,
            firmware_prefix: None,
            sys_fs_prefix: None,
        }
    }
    fn merge(self, fallback: SystemPaths) -> SystemPaths {
        SystemPaths {
            config_fs_prefix: self.config_fs_prefix.or(fallback.config_fs_prefix),
            firmware_prefix: self.firmware_prefix.or(fallback.firmware_prefix),
            sys_fs_prefix: self.sys_fs_prefix.or(fallback.sys_fs_prefix),
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

/// User config overrides vendor config and vendor config overrides hardcoded defaults
fn init_system_config() -> Mutex<SystemConfig> {
    let vendor_config = system_paths_config_from_file(&PathBuf::from("/usr/lib/fpgad/config.toml"))
        .unwrap_or_else(|e| {
            warn!("Using hardcoded paths for vendor config because loading config failed: {e}");
            SystemPaths::default()
        });
    let user_config = system_paths_config_from_file(&PathBuf::from("/etc/fpgad/config.toml"))
        .unwrap_or_else(|e| {
            warn!("Using hardcoded paths for user config because loading config failed: {e}");
            SystemPaths::default()
        });
    trace!("Merging user_config: {user_config:?} with vendor_config {vendor_config:?}");
    let merged = user_config.merge(vendor_config);
    let ret = Mutex::new(merged.into());
    trace!("Resulting config: {ret:?}");
    ret
}

pub fn system_config() -> &'static Mutex<SystemConfig> {
    CONFIG.get_or_init(init_system_config)
}

pub fn system_config_guard() -> Result<MutexGuard<'static, SystemConfig>, FpgadError> {
    let guard = match system_config().try_lock() {
        Ok(guard) => guard,
        Err(e) => {
            return Err(FpgadError::Internal(format!(
                "Failed when locking config for read access: {e}"
            )));
        }
    };
    Ok(guard)
}

pub fn config_fs_prefix() -> Result<String, FpgadError> {
    let guard = system_config_guard()?;
    guard.config_fs_prefix()
}

pub fn firmware_prefix() -> Result<String, FpgadError> {
    let guard = system_config_guard()?;
    guard.firmware_prefix()
}

pub fn sys_fs_prefix() -> Result<String, FpgadError> {
    let guard = system_config_guard()?;
    guard.sys_fs_prefix()
}

pub fn set_config_fs_prefix(prefix: String) -> Result<(), FpgadError> {
    let config = system_config_guard()?;

    *config
        .config_fs_prefix
        .lock()
        .map_err(|e| FpgadError::Internal(format!("Failed to lock config_fs_prefix: {e}")))? =
        prefix;

    Ok(())
}

pub fn set_firmware_prefix(prefix: String) -> Result<(), FpgadError> {
    let config = system_config_guard()?;

    *config
        .firmware_prefix
        .lock()
        .map_err(|e| FpgadError::Internal(format!("Failed to lock firmware_prefix: {e}")))? =
        prefix;

    Ok(())
}

pub fn set_sys_fs_prefix(prefix: String) -> Result<(), FpgadError> {
    let config = system_config_guard()?;

    *config
        .sys_fs_prefix
        .lock()
        .map_err(|e| FpgadError::Internal(format!("Failed to lock sys_fs_prefix: {e}")))? = prefix;

    Ok(())
}
