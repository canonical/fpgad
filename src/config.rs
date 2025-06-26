use crate::error::FpgadError;
use crate::system_io::fs_read;
use log::{trace, warn};
use serde::Deserialize;
use std::path::PathBuf;
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

impl SystemConfig {
    fn default() -> SystemConfig {
        SystemConfig {
            config_fs_prefix: Mutex::new(CONFIG_FS_PREFIX.to_string()),
            firmware_prefix: Mutex::new(FW_PREFIX.to_string()),
            sys_fs_prefix: Mutex::new(SYSFS_PREFIX.to_string()),
        }
    }
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
    defaults: Option<DefaultsToml>,
}

/// This is the "defaults" struct
#[derive(Debug, Deserialize)]
struct DefaultsToml {
    config_fs_prefix: Option<String>,
    firmware_prefix: Option<String>,
    sys_fs_prefix: Option<String>,
}

impl From<DefaultsToml> for SystemConfig {
    fn from(value: DefaultsToml) -> Self {
        trace!("User provided config: {value:?}");
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

fn config_from_file() -> Result<SystemConfig, FpgadError> {
    let config_path = PathBuf::from("/etc/fpgad/config.toml");
    if !config_path.is_file() {
        return Err(FpgadError::Internal(format!(
            "Config file not found in {config_path:?}. \
        Using hardcoded defaults"
        )));
    }
    let toml_string = fs_read(&config_path)?;

    let config: TomlConfig = match toml::from_str(&toml_string) {
        Ok(config) => config,
        Err(e) => {
            return Err(FpgadError::TomlDe {
                file: config_path,
                e,
            });
        }
    };
    match config.defaults {
        Some(defaults_toml) => Ok(defaults_toml.into()),
        None => Err(FpgadError::Internal(
            "config file did not contain a `[defaults]` section.".to_string(),
        )),
    }
}

fn init_system_config() -> Mutex<SystemConfig> {
    match config_from_file() {
        Ok(config) => {
            trace!("Successfully loaded config: {config:?}");
            Mutex::new(config)
        }
        Err(e) => {
            warn!("Using hardcoded paths because failed to load config: {e}");
            Mutex::new(SystemConfig::default())
        }
    }
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
