use crate::error::FpgadError;
use log::{trace, warn};
use std::path::PathBuf;

use crate::config::config_files::{SystemPaths, system_paths_config_from_file};
use std::sync::{Mutex, MutexGuard, OnceLock};

// These are hardcoded backups to prevent crashing and lockups when accessing the config file
// or Mutex values
pub static OVERLAY_CONTROL_DIR: &str = "/sys/kernel/config/device-tree/overlays/";
pub static FIRMWARE_SOURCE_DIR: &str = "/lib/firmware/";
pub static FPGA_MANAGERS_DIR: &str = "/sys/class/fpga_manager/";

#[derive(Debug)]
pub struct SystemConfig {
    pub(crate) overlay_control_dir: Mutex<String>,
    pub(crate) firmware_source_dir: Mutex<String>,
    pub(crate) fpga_managers_dir: Mutex<String>,
}

static CONFIG: OnceLock<Mutex<SystemConfig>> = OnceLock::new();

impl SystemConfig {
    fn overlay_control_dir(&self) -> Result<String, FpgadError> {
        let guard = match self.overlay_control_dir.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(FpgadError::Internal(format!(
                    "Failed when locking overlay_control_dir for read access: {e}"
                )));
            }
        };
        Ok(guard.clone())
    }

    fn firmware_source_dir(&self) -> Result<String, FpgadError> {
        let guard = match self.firmware_source_dir.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(FpgadError::Internal(format!(
                    "Failed when locking firmware_source_dir for read access: {e}"
                )));
            }
        };

        Ok(guard.clone())
    }

    fn fpga_managers_dir(&self) -> Result<String, FpgadError> {
        let guard = match self.fpga_managers_dir.try_lock() {
            Ok(guard) => guard,
            Err(e) => {
                return Err(FpgadError::Internal(format!(
                    "Failed when locking fpga_managers_dir for read access: {e}"
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

pub fn overlay_control_dir() -> Result<String, FpgadError> {
    system_config_guard()?.overlay_control_dir()
}

pub fn firmware_source_dir() -> Result<String, FpgadError> {
    system_config_guard()?.firmware_source_dir()
}

pub fn fpga_managers_dir() -> Result<String, FpgadError> {
    system_config_guard()?.fpga_managers_dir()
}

pub fn set_overlay_control_dir(prefix: String) -> Result<(), FpgadError> {
    let config = system_config_guard()?;

    *config
        .overlay_control_dir
        .lock()
        .map_err(|e| FpgadError::Internal(format!("Failed to lock overlay_control_dir: {e}")))? =
        prefix;

    Ok(())
}

pub fn set_firmware_source_dir(prefix: String) -> Result<(), FpgadError> {
    let config = system_config_guard()?;

    *config
        .firmware_source_dir
        .lock()
        .map_err(|e| FpgadError::Internal(format!("Failed to lock firmware_source_dir: {e}")))? =
        prefix;

    Ok(())
}

pub fn set_fpga_managers_dir(prefix: String) -> Result<(), FpgadError> {
    let config = system_config_guard()?;

    *config
        .fpga_managers_dir
        .lock()
        .map_err(|e| FpgadError::Internal(format!("Failed to lock fpga_managers_dir: {e}")))? =
        prefix;

    Ok(())
}
