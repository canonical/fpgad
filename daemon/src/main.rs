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

//! FPGA daemon (fpgad) - System service for managing FPGA devices.
//!
//! This is the main entry point for the fpgad daemon, which provides a DBus service for
//! managing FPGA devices on Linux systems. The daemon:
//! - Exposes two DBus interfaces: `control` and `status`
//! - Manages FPGA bitstream loading through the Linux FPGA subsystem
//! - Handles device tree overlay application and removal
//! - Provides platform-specific FPGA management capabilities
//! - Runs as a system service with appropriate privileges
//!
//! # DBus Service
//!
//! - **Service Name**: `com.canonical.fpgad`
//! - **Status Interface**: `/com/canonical/fpgad/status` - Read-only operations
//! - **Control Interface**: `/com/canonical/fpgad/control` - Write operations
//!
//! # Environment Variables
//!
//! - `RUST_LOG` - Controls logging level (`trace`, `debug`, `info`, `warn`, `error`
//!   or `off`). Defaults to `info`
//!
//! # Architecture
//!
//! The daemon uses a platform abstraction approach that allows different FPGA vendors
//! and platforms to be supported through registered platform implementations. At startup,
//! the daemon:
//! 1. Registers all available platform implementations
//! 2. Creates DBus interface objects
//! 3. Connects to the system DBus and advertises the service
//! 4. Waits indefinitely for incoming DBus requests
//!
//! # Platform Support
//!
//! - **Universal Platform**: Generic FPGA support for standard Linux FPGA subsystem
//! - **Xilinx DFX Manager** (optional, feature-gated): Xilinx-specific softener which uses dfx-mgr
//! - **Additional Platforms**: Can be added via feature flags and custom implementations
//!
//! See the [`platforms`] module for details on the platform abstraction system and
//! [`softeners`] for more information on vendor-specific implementations.

use log::info;
use std::error::Error;
use std::future::pending;
use zbus::connection;
mod error;

mod comm;

mod config;
mod platforms;
#[cfg(feature = "softeners")]
mod softeners;
mod system_io;

#[cfg(feature = "xilinx-dfx-mgr")]
use crate::softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;

use crate::{
    comm::dbus::{control_interface::ControlInterface, status_interface::StatusInterface},
    platforms::universal::UniversalPlatform,
};

/// Register all available FPGA platform implementations.
///
/// This function is called at daemon startup to register platform handlers. Each
/// platform implementation provides vendor or hardware-specific logic for FPGA
/// management operations. Platforms are registered in order of priority, with more
/// specific platforms registered before generic ones.
///
/// # Platform Registration Order
///
/// 1. Xilinx DFX Manager (if feature enabled) - Handles Xilinx-specific devices
/// 2. Universal Platform - Fallback for standard Linux FPGA subsystem devices
fn register_platforms() {
    #[cfg(feature = "xilinx-dfx-mgr")]
    XilinxDfxMgrPlatform::register_platform();
    UniversalPlatform::register_platform();
}

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
