use crate::config::boot_firmware;
use crate::config::config_files::{boot_firmware_from_file, BootFirmware};
use crate::error::FpgadError;
use crate::platforms::platform::new_platform;
use crate::platforms::platform::Fpga;
use crate::platforms::platform::OverlayHandler;
use crate::platforms::platform::Platform;
use crate::system_io::validate_device_handle;
use log::{info, trace};
use std::path::{Path, PathBuf};

pub fn boot_firmware() -> BootFirmware {
    let vendor_config = boot_firmware_from_file(&PathBuf::from("/usr/lib/fpgad/config.toml"))
        .unwrap_or(BootFirmware::default());
    let user_config = boot_firmware_from_file(&PathBuf::from("/etc/fpgad/config.toml"))
        .unwrap_or(BootFirmware::default());
    user_config.merge(vendor_config)
}

pub fn load_defaults() -> Result<String, FpgadError> {
    let mut ret_string = String::new();
    let boot_firmware = boot_firmware::boot_firmware();
    if let (Some(bitstream_path_str), Some(device_handle)) = (
        boot_firmware.default_bitstream.as_ref(),
        boot_firmware.default_device_handle.as_ref(),
    ) {
        trace!(
            "Default bitstream provided. Attempting to load {bitstream_path_str} to {device_handle}"
        );
        validate_device_handle(device_handle)?;

        let path = Path::new(bitstream_path_str);
        if !path.exists() || path.is_dir() {
            return Err(FpgadError::Argument(format!(
                "the provided default bitstream path '{bitstream_path_str}' is not a valid path to \
                a bitstream file.",
            )));
        }
        let platform = new_platform(device_handle)?;
        let fpga = platform.fpga(device_handle)?;
        if let Some(flags) = boot_firmware.default_fpga_flags {
            fpga.set_flags(flags)?
        }
        fpga.load_firmware(path)?;
        info!("{bitstream_path_str} loaded to {device_handle}");
        ret_string.push_str(&format!("{bitstream_path_str} loaded to {device_handle}\n"));
    } else {
        info!("Not enough information provided in order to load a bitstream on startup");
        ret_string
            .push_str("Not enough information provided in order to load a bitstream on startup\n");
    }
    if let (Some(overlay_source_path), Some(device_handle), Some(overlay_handle)) = (
        boot_firmware.default_overlay.as_ref(),
        boot_firmware.default_device_handle.as_ref(),
        boot_firmware.default_overlay_handle.as_ref(),
    ) {
        trace!(
            "Default overlay provided. Attempting to load {overlay_source_path} to {device_handle} \
            with overlay_handle {overlay_handle}"
        );
        validate_device_handle(device_handle)?;

        let platform = new_platform(device_handle)?;
        if let Some(flags) = boot_firmware.default_fpga_flags {
            platform.fpga(device_handle)?.set_flags(flags)?
        }
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.apply_overlay(Path::new(overlay_source_path))?;
        info!("{overlay_source_path} loaded via {overlay_fs_path:?}");
        ret_string.push_str(&format!(
            "{overlay_source_path} loaded via {overlay_fs_path:?}\n"
        ));
    } else {
        info!("Not enough information provided in order to apply an overlay on startup");
        ret_string
            .push_str("Not enough information provided in order to apply an overlay on startup\n");
    }
    Ok(ret_string)
}
