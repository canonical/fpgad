use crate::platforms::platform::Fpga;
use crate::platforms::platform::OverlayHandler;
use crate::platforms::platform::Platform;
use crate::platforms::platform::new_platform;
use crate::system_io::validate_device_handle;
use log::trace;
use zbus::{fdo, interface};

pub struct StatusInterface {}

#[interface(name = "com.canonical.fpgad.status")]
impl StatusInterface {
    async fn get_fpga_state(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_state called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        Ok(new_platform(device_handle).fpga(device_handle)?.state()?)
    }

    async fn get_fpga_flags(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_flags called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        Ok(new_platform(device_handle)
            .fpga(device_handle)?
            .flags()
            .map(|flags| flags.to_string())?)
    }

    async fn get_overlay_status(
        &self,
        device_handle: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "get_overlay_status called with device_handle: {device_handle} and overlay_handle:\
             {overlay_handle}"
        );
        validate_device_handle(device_handle)?;
        Ok(new_platform(device_handle)
            .overlay_handler(overlay_handle)?
            .status()?)
    }
}
