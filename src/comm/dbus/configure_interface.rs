use crate::config::system_config;
use log::trace;
use zbus::{fdo, interface};

pub struct ConfigureInterface {}


#[interface(name = "com.canonical.fpgad.configure")]
impl ConfigureInterface {
    async fn get_overlay_control_dir(&self) -> Result<String, fdo::Error> {
        trace!("get_overlay_control_dir called");
        Ok(system_config::overlay_control_dir()?)
    }
    async fn get_firmware_source_dir(&self) -> Result<String, fdo::Error> {
        trace!("get_firmware_source_dir called");
        Ok(system_config::firmware_source_dir()?)
    }

    async fn get_fpga_managers_dir(&self) -> Result<String, fdo::Error> {
        trace!("get_fpga_managers_dir called");
        Ok(system_config::fpga_managers_dir()?)
    }
    async fn set_overlay_control_dir(&self, new_path: &str) -> Result<String, fdo::Error> {
        trace!("set_overlay_control_dir called with prefix: {new_path}");
        system_config::set_overlay_control_dir(new_path)?;
        Ok(format!("overlay_control_dir set to {new_path}"))
    }
    async fn set_firmware_source_dir(&self, new_path: &str) -> Result<String, fdo::Error> {
        trace!("set_firmware_source_dir called with prefix: {new_path}");
        system_config::set_firmware_source_dir(new_path)?;
        Ok(format!("firmware_source_dir set to {new_path}"))
    }

    async fn set_fpga_managers_dir(&self, new_path: &str) -> Result<String, fdo::Error> {
        trace!("set_fpga_managers_dir called with prefix: {new_path}");
        system_config::set_fpga_managers_dir(new_path)?;
        Ok(format!("fpga_managers_dir set to {new_path}"))
    }
}