use crate::error::FpgadError;
use log::{trace, warn};
use std::path::PathBuf;

use crate::config::config_files::{SystemPaths, system_paths_config_from_file};
use std::sync::{Mutex, MutexGuard, OnceLock};

// These are hardcoded backups to prevent crashing and lockups when accessing the config file
// or Mutex values
pub static CONFIG_FS_PREFIX: &str = "/sys/kernel/config/device-tree/overlays/";
pub static FW_PREFIX: &str = "/lib/firmware/";
pub static SYSFS_PREFIX: &str = "/sys/class/fpga_manager/";

#[derive(Debug)]
pub struct SystemConfig {
    pub(crate) config_fs_prefix: Mutex<String>,
    pub(crate) firmware_prefix: Mutex<String>,
    pub(crate) sys_fs_prefix: Mutex<String>,
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
