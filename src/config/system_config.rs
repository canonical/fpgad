use crate::error::FpgadError;
use log::{error, trace, warn};
use std::path::{Path, PathBuf};

use crate::config::config_files::{SystemPaths, system_paths_config_from_file};
use crate::system_io::fs_write;
use std::sync::{Mutex, MutexGuard, OnceLock};

// These are hardcoded backups to prevent crashing and lockups when accessing the config file
// or Mutex values
pub static CONFIG_FS_PREFIX: &str = "/sys/kernel/config/device-tree/overlays/";
pub static FW_PREFIX: &str = "/lib/firmware/";
pub static SYSFS_PREFIX: &str = "/sys/class/fpga_manager/";
pub static FW_LOOKUP_PATH: &str = "/sys/module/firmware_class/parameters/path";

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

fn write_fw_prefix(prefix: &str) -> Result<(), FpgadError> {
    trace!("Writing fw prefix {prefix} to {FW_LOOKUP_PATH}");
    let fw_lookup_override = Path::new(FW_LOOKUP_PATH);
    fs_write(fw_lookup_override, false, prefix)
}

pub fn set_firmware_prefix(prefix: &str) -> Result<(), FpgadError> {
    let config = system_config_guard()?;
    let old_prefix = config.firmware_prefix()?;
    write_fw_prefix(prefix)?;

    *config.firmware_prefix.lock().map_err(|e| {
        if let Err(reset_err) = write_fw_prefix(&old_prefix) {
            error!("Failed to reset firmware_prefix: {reset_err}");
        }
        FpgadError::Internal(format!("Failed to lock firmware_prefix: {e}"))
    })? = prefix.to_owned();
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
