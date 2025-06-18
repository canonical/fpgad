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

use std::error::Error;
use std::future::pending;
use std::path::Path;
use zbus::connection;
mod error;
use log::{error, info, trace};

use platforms::{
    platform::{list_fpga_managers, Fpga, Platform},
    universal,
};

mod comm;
use crate::error::FpgadError;
use comm::dbus::interfaces::Greeter;
use universal::UniversalPlatform;

mod platforms;
mod system_io;

pub(crate) fn load_package(
    platform: &mut impl Platform,
    overlay_handle: &str,
    bitstream_path: &Path,
    overlay_source_path: &Path,
) -> Result<(), FpgadError> {
    trace!(
        "Load package called with bitstream_path: {:?} and overlay_path: {:?}",
        bitstream_path, overlay_source_path
    );
    platform.fpga("fpga0").set_flags(0)?;
    platform
        .overlay_handler()
        .set_overlay_fs_path(overlay_handle);
    platform
        .overlay_handler()
        .set_source_path(overlay_source_path)?;

    Ok(platform.overlay_handler().apply_overlay()?)
}

/// Removes the overlay, undoing any extra steps, and then deletes the overlay_handler
fn unload_package(platform: &mut impl Platform) -> Result<(), FpgadError> {
    Ok(platform.overlay_handler().remove_overlay()?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let greeter = Greeter { count: 0 };

    let _conn = connection::Builder::session()?
        .name("com.canonical.fpgad.MyGreeter")?
        .serve_at("/com/canonical/fpgad/MyGreeter", greeter)?
        .build()
        .await?;

    // client will send a request to load bitstream to fpga
    // if no fpga name specified fpgad will try all FPGAs available under /sys/class/fpga_manager/
    // if no platform specified fpgad will use universal_platform for each fpga
    // available, until it successfully loads the bitstream
    //
    for fpga in list_fpga_managers().iter() {
        let mut universal_platform = UniversalPlatform::new();
        info!("Detected {}", universal_platform.fpga(fpga).name());
    }
    trace!("FPGA managers scraped.");
    let mut universal_platform = UniversalPlatform::new();
    trace!("Initializing {}", universal_platform.fpga("fpga0").name());
    let fpga = universal_platform.fpga("fpga0");
    match fpga.get_state() {
        Err(e) => error!("Initialising FPGA failed with error: '{}'", e),
        Ok(val) => info!(
            "{} initialised with initial state of '{}' at time of detection.",
            fpga.name(),
            val
        ),
    };

    let bitstream_path = Path::new("/lib/firmware/k26-starter-kits.bit.bin");
    let dtbo_path = Path::new("/lib/firmware/k26-starter-kits.dtbo");
    let load_result = load_package(&mut universal_platform, "fpga0", bitstream_path, dtbo_path);

    match &load_result {
        Err(e) => {
            error!(
                "Failed to load bitstream using files: '{:?}' for bitstream and '{:?}' for dtbo: '{}'",
                bitstream_path, dtbo_path, e
            );
        }
        Ok(_) => info!("Bitstream appears to be successfully loaded."),
    };

    if load_result.is_ok() {
        info!("Waiting 5s.");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        info!("The wait is over prepare to be unloaded!");
        if unload_package(&mut universal_platform).is_err() {
            error!("Failed to unload bitstream!");
        } else {
            info!(
                "No errors encountered when unloading bitstream.\nWaiting for dbus messages. (ctrl+C to quit)."
            );
        }
    } else {
        error!("Failed to load bitstream!\nWaiting for dbus messages. (ctrl+C to quit).");
    }

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
