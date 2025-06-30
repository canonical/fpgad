use crate::config::boot_firmware;
use crate::config::config_files;
use crate::error::FpgadError;
use crate::platforms::platform::Fpga;
use crate::platforms::platform::OverlayHandler;
use crate::platforms::platform::Platform;
use crate::platforms::platform::{platform_for_device, platform_for_known_platform};
use crate::system_io::validate_device_handle;
use log::{info, warn};
use std::path::{Path, PathBuf};

pub fn boot_firmware() -> config_files::BootFirmware {
    let vendor_config =
        config_files::boot_firmware_from_file(&PathBuf::from("/usr/lib/fpgad/config.toml"))
            .unwrap_or(config_files::BootFirmware::default());
    let user_config =
        config_files::boot_firmware_from_file(&PathBuf::from("/etc/fpgad/config.toml"))
            .unwrap_or(config_files::BootFirmware::default());
    user_config.merge(vendor_config)
}

fn load_a_default_bitstream(bitstream: config_files::Bitstream) -> Result<String, FpgadError> {
    validate_device_handle(&bitstream.device_handle)?;
    let platform = platform_for_device(&bitstream.device_handle)?;
    let fpga = platform.fpga(&bitstream.device_handle)?;
    fpga.set_flags(bitstream.flags)?;
    fpga.load_firmware(Path::new(&bitstream.bitstream_path))?;
    Ok(format!(
        "Bitstream {} written to {}\n",
        bitstream.bitstream_path, bitstream.device_handle
    ))
}

fn load_a_default_overlay(overlay: config_files::Overlay) -> Result<String, FpgadError> {
    let platform = platform_for_known_platform(&overlay.platform);
    if let (Some(flags), Some(fpga_handle)) = (overlay.fpga_flags, overlay.device_handle.clone()) {
        validate_device_handle(&fpga_handle)?;
        platform.fpga(&fpga_handle)?.set_flags(flags)?;
    } else if overlay.fpga_flags.is_some() ^ overlay.device_handle.is_some() {
        warn!(
            "Provided default overlay configuration specifies flags or fpga handle but not both. \
            Skipping setting fpga flags. Configuration: {overlay:?}"
        )
    }
    platform
        .overlay_handler(&overlay.overlay_handle)?
        .apply_overlay(overlay.overlay_path.as_ref())?;
    Ok(format!(
        "Overlay {} applied using {}\n",
        overlay.overlay_path, overlay.overlay_handle
    ))
}

pub fn load_defaults() -> Result<String, FpgadError> {
    let mut ret_string = String::new();
    let boot_firmware = boot_firmware::boot_firmware();
    if !boot_firmware.bitstreams.is_empty() {
        for bitstream in boot_firmware.bitstreams {
            ret_string.push_str(&load_a_default_bitstream(bitstream)?);
        }
    } else {
        info!("No default bitstreams found in config files. Skipping.");
        ret_string.push_str("No default bitstreams found in config files. Skipping.\n");
    }
    if !boot_firmware.overlays.is_empty() {
        for overlay in boot_firmware.overlays {
            ret_string.push_str(&load_a_default_overlay(overlay)?);
        }
    } else {
        info!("No default overlays found in config files. Skipping.");
        ret_string.push_str("No default overlays found in config files. Skipping.\n");
    }
    Ok(ret_string)
}
