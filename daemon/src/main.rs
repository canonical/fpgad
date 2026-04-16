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

use fpgad::comm::dbus::{control_interface::ControlInterface, status_interface::StatusInterface};
use fpgad::register_platforms;
use log::info;
use std::error::Error;
use std::future::pending;
use zbus::connection;

/// Main entry point for the fpgad daemon.
///
/// Initializes the daemon by:
/// 1. Setting up logging via `env_logger` (defaults to "info" level)
/// 2. Registering platform implementations
/// 3. Creating DBus interface instances
/// 4. Connecting to the system DBus and advertising the service
/// 5. Running indefinitely to serve DBus requests
///
/// # Returns: `Result<(), Box<dyn Error>>`
/// * `Ok(())` - Never returns under normal operation (runs until terminated)
/// * `Err(Box<dyn Error>)` - Initialization error (DBus connection failed, etc.)
///
/// # Environment Variables
///
/// - `RUST_LOG` - Controls logging level (`trace`, `debug`, `info`, `warn`, `error`
///   or `off`). Defaults to `info`
///
/// # Examples
///
/// ```bash
/// # Run with default logging (info level)
/// fpgad
///
/// # Run with debug logging
/// RUST_LOG=debug fpgad
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    register_platforms();

    // Upon load, the daemon will search each fpga device and determine what platform it is
    // based on its name in /sys/class/fpga_manager/{device}/name
    let status_interface = StatusInterface {};
    let control_interface = ControlInterface {};

    let _conn = connection::Builder::system()?
        .name("com.canonical.fpgad")?
        .serve_at("/com/canonical/fpgad/status", status_interface)?
        .serve_at("/com/canonical/fpgad/control", control_interface)?
        .build()
        .await?;

    info!("Started com.canonical.fpgad dbus service");
    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
