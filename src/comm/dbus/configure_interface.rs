use crate::config;
use crate::error::FpgadError;
use crate::system_io::{fs_read, fs_write};
use log::trace;
use std::path::Path;
use zbus::{fdo, interface};

pub struct ConfigureInterface {}

fn write_firmware_source_dir(new_path: &str) -> Result<(), FpgadError> {
    trace!(
        "Writing fw prefix {} to {}",
        new_path,
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_write(fw_lookup_override, false, new_path)
}

fn read_firmware_source_dir() -> Result<String, FpgadError> {
    trace!(
        "Reading fw prefix from {}",
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_read(fw_lookup_override)
}

pub fn set_firmware_source_dir(new_path: &str) -> Result<(), FpgadError> {
    // TODO: checks for exist?
    write_firmware_source_dir(new_path)
}
#[interface(name = "com.canonical.fpgad.configure")]
impl ConfigureInterface {
    async fn get_overlay_control_dir(&self) -> Result<String, fdo::Error> {
        trace!("get_overlay_control_dir called");
        Ok(config::OVERLAY_CONTROL_DIR.to_string())
    }
    async fn get_firmware_source_dir(&self) -> Result<String, fdo::Error> {
        trace!("get_firmware_source_dir called");
        Ok(read_firmware_source_dir()?)
    }
    async fn get_fpga_managers_dir(&self) -> Result<String, fdo::Error> {
        trace!("get_fpga_managers_dir called");
        Ok(config::FPGA_MANAGERS_DIR.to_string())
    }

    async fn set_firmware_source_dir(&self, new_path: &str) -> Result<String, fdo::Error> {
        trace!("set_firmware_source_dir called with prefix: {new_path}");
        set_firmware_source_dir(new_path)?;
        Ok(format!("firmware_source_dir set to {new_path}"))
    }
}
