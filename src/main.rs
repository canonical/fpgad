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

use log::{error, info};
use std::error::Error;
use std::future::pending;
use zbus::connection;
mod error;

mod comm;

mod config;
mod platforms;
mod system_io;

use crate::comm::dbus::interfaces::{ConfigureInterface, ControlInterface, StatusInterface};
use crate::config::system_config;

/// Tries to set the system's firmware search path to the value which is stored in the
/// system_config, should be done immediately after initialising the config.
/// If the system_config's fw_prefix is not different to the hardcoded default,
/// then nothing happens.
/// On error, no changes stick and the failure is logged.
fn set_fw_lookup_on_startup() {
    let prefix: String = match system_config::firmware_source_dir() {
        Ok(pfx) => pfx, // weird pattern necessary to release the lock on config
        Err(e) => {
            error!("Failed to get firmware prefix after config init on startup: {e}");
            return;
        }
    };

    if prefix != system_config::FIRMWARE_SOURCE_DIR {
        match system_config::set_firmware_source_dir(&prefix) {
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
    let _ = system_config::system_config();
    set_fw_lookup_on_startup();

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
    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
