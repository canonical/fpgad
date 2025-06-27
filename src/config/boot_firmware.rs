use crate::config::config_files::{BootFirmware, boot_firmware_from_file};
use std::path::PathBuf;

pub fn boot_firmware() -> BootFirmware {
    let vendor_config = boot_firmware_from_file(&PathBuf::from("/usr/lib/fpgad/config.toml"))
        .unwrap_or(BootFirmware::default());
    let user_config = boot_firmware_from_file(&PathBuf::from("/etc/fpgad/config.toml"))
        .unwrap_or(BootFirmware::default());
    user_config.merge(vendor_config)
}
