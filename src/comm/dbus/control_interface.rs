use crate::platforms::platform::{platform_for_device, platform_for_known_platform};
use crate::system_io::validate_device_handle;
use log::trace;
use std::path::Path;
use zbus::{fdo, interface};

pub struct ControlInterface {}
#[interface(name = "com.canonical.fpgad.control")]
impl ControlInterface {
    async fn set_fpga_flags(
        &self,
        platform_string: &str,
        device_handle: &str,
        flags: u32,
    ) -> Result<String, fdo::Error> {
        trace!("set_fpga_flags called with name: {device_handle} and flags: {flags}");
        validate_device_handle(device_handle)?;
        let platform = match platform_string.is_empty() {
            true => platform_for_device(device_handle)?,
            false => platform_for_known_platform(platform_string)?,
        };
        platform.fpga(device_handle)?.set_flags(flags)?;
        Ok(format!("Flags set to {flags} for {device_handle}"))
    }

    async fn write_bitstream_direct(
        &self,
        platform_string: &str,
        device_handle: &str,
        bitstream_path_str: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "load_firmware called with name: {device_handle} and path_str: {bitstream_path_str}"
        );
        validate_device_handle(device_handle)?;
        let path = Path::new(bitstream_path_str);
        if !path.exists() || path.is_dir() {
            return Err(fdo::Error::InvalidArgs(format!(
                "{bitstream_path_str} is not a valid path to a bitstream file."
            )));
        }
        let platform = match platform_string.is_empty() {
            true => platform_for_device(device_handle)?,
            false => platform_for_known_platform(platform_string)?,
        };
        platform.fpga(device_handle)?.load_firmware(path)?;
        Ok(format!("{bitstream_path_str} loaded to {device_handle}"))
    }

    async fn apply_overlay(
        &self,
        platform_compat_str: &str,
        overlay_handle: &str,
        overlay_source_path: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "apply_overlay called with platform_compat_str:{platform_compat_str}, overlay_handle: \
            {overlay_handle} and overlay_path: {overlay_source_path}",
        );
        let platform = platform_for_known_platform(platform_compat_str)?;
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.apply_overlay(Path::new(overlay_source_path))?;
        Ok(format!(
            "{overlay_source_path} loaded via {overlay_fs_path:?}"
        ))
    }

    async fn remove_overlay(
        &self,
        platform_compat_str: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "remove_overlay called with platform_compat_str: {platform_compat_str} and overlay_handle:\
             {overlay_handle}"
        );
        let platform = platform_for_known_platform(platform_compat_str)?;
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.remove_overlay()?;
        Ok(format!(
            "{overlay_handle} removed by deleting {overlay_fs_path:?}"
        ))
    }
}
