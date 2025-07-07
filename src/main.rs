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

use log::info;
use std::error::Error;
use std::future::pending;
use zbus::connection;
mod error;

mod comm;

mod config;
mod platforms;
mod system_io;

use crate::comm::dbus::{control_interface::ControlInterface, status_interface::StatusInterface};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

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
