// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
//
// Copyright 2025 Canonical Ltd.
//
// SPDX-License-Identifier: GPL-3.0-only
//
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
//
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

use crate::platforms::platform::Fpga;
use log::{error, info, trace};
use std::error::Error;
use std::future::pending;
use std::path::Path;
use zbus::connection;
mod error;

mod comm;

mod config;
mod platforms;
mod system_io;

use crate::comm::dbus::interfaces::{ConfigureInterface, ControlInterface, StatusInterface};
use crate::config::boot_firmware::boot_firmware;
use crate::config::system_config::{
    FW_PREFIX, firmware_prefix, set_firmware_prefix, system_config,
};
use crate::error::FpgadError;
use crate::platforms::platform::new_platform;
use crate::platforms::platform::{OverlayHandler, Platform};
use crate::system_io::validate_device_handle;

fn load_defaults() -> Result<(), FpgadError> {
    let boot_firmware = boot_firmware();
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
        let platform = new_platform(device_handle);
        let fpga = platform.fpga(device_handle)?;
        if let Some(flags) = boot_firmware.default_fpga_flags {
            fpga.set_flags(flags)?
        }
        fpga.load_firmware(path)?;
        info!("{bitstream_path_str} loaded to {device_handle}")
    } else {
        info!("Not enough information provided in order to load a bitstream on startup")
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

        let platform = new_platform(device_handle);
        if let Some(flags) = boot_firmware.default_fpga_flags {
            platform.fpga(device_handle)?.set_flags(flags)?
        }
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.apply_overlay(Path::new(overlay_source_path))?;
        info!("{overlay_source_path} loaded via {overlay_fs_path:?}")
    } else {
        info!("Not enough information provided in order to apply an overlay on startup")
    }
    Ok(())
}
/// Tries to set the system's firmware search path to the value which is stored in the
/// system_config, should be done immediately after initialising the config.
/// If the system_config's fw_prefix is not different to the hardcoded default,
/// then nothing happens.
/// On error, no changes stick and the failure is logged.
fn set_fw_lookup_on_startup() {
    let prefix: String = match firmware_prefix() {
        Ok(pfx) => pfx, // weird pattern necessary to release the lock on config
        Err(e) => {
            error!("Failed to get firmware prefix after config init on startup: {e}");
            return;
        }
    };

    if prefix != FW_PREFIX {
        match set_firmware_prefix(&prefix) {
            Ok(_) => {
                info!("System's firmware lookup prefix was successfully set.");
            }
            Err(e) => {
                error!("Failed to set firmware lookup prefix on startup: {e}")
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    // call to initialise
    let _ = system_config();
    set_fw_lookup_on_startup();

    if let Err(e) = load_defaults() {
        error!("Failed to apply the specified bitstreams during startup: {e}");
    }

    // Upon load, the daemon will search each fpga device and determine what platform it is
    // based on its name in /sys/class/fpga_manager/{device}/name
    let status_interface = StatusInterface {};
    let control_interface = ControlInterface {};
    let configure_interface = ConfigureInterface {};

    let _conn = connection::Builder::system()?
        .name("com.canonical.fpgad")?
        .serve_at("/com/canonical/fpgad/status", status_interface)?
        .serve_at("/com/canonical/fpgad/control", control_interface)?
        .serve_at("/com/canonical/fpgad/configure", configure_interface)?
        .build()
        .await?;

    info!("Started com.canonical.fpgad dbus service");
    if let Err(e) = sd_notify::notify(true, &[sd_notify::NotifyState::Ready]) {
        eprintln!("Failed to notify systemd: {e}");
    }
    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
