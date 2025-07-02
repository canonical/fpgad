use crate::platforms::platform::OverlayHandler;
use crate::platforms::platform::Platform;
use crate::platforms::platform::{list_fpga_managers, platform_for_device, read_compatible_string};
use crate::platforms::platform::{platform_for_known_platform, Fpga};
use crate::system_io::validate_device_handle;
use log::{error, trace};
use zbus::{fdo, interface};

pub struct StatusInterface {}

#[interface(name = "com.canonical.fpgad.status")]
impl StatusInterface {
    async fn get_fpga_state(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_state called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        Ok(platform_for_device(device_handle)?
            .fpga(device_handle)?
            .state()?)
    }

    async fn get_fpga_flags(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_flags called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        Ok(platform_for_device(device_handle)?
            .fpga(device_handle)?
            .flags()
            .map(|flags| flags.to_string())?)
    }

    async fn get_overlay_status(
        &self,
        platform_compat_str: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "get_overlay_status called with platform_compat_str: {platform_compat_str} and overlay_handle:\
             {overlay_handle}"
        );
        Ok(platform_for_known_platform(platform_compat_str)
            .overlay_handler(overlay_handle)?
            .status()?)
    }

    async fn get_platform_type(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_platform_type called with device_handle: {device_handle}");
        validate_device_handle(device_handle)?;
        let ret_string = read_compatible_string(device_handle)?;
        Ok(ret_string.to_string())
    }

    async fn get_platform_types(&self) -> Result<String, fdo::Error> {
        trace!("get_platform_types called");
        let mut ret_string = String::new();
        let devices = list_fpga_managers()?;
        for device_handle in devices {
            if let Ok(compat_string) = read_compatible_string(&device_handle) {
                ret_string += format!("{device_handle}:{compat_string}\n").as_str();
            } else {
                error!("Failed to get string for {device_handle}");
                ret_string += format!("{device_handle}:\n").as_str();
            }
        }
        Ok(ret_string)
    }

    async fn get_platform_name(&self, _device_handle: &str) -> Result<String, fdo::Error> {
        todo!()
    }
}
