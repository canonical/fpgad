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

//! FPGA CLI (fpgad_cli) - Command-line interface for managing FPGA devices.
//!
//! This is FPGAd's command-line interface (CLI) for interacting with the fpgad daemon via DBus.
//! Due to strict confinement of the snap, this can only be used from a terminal or from a script
//! which is not part of another snap. It is a useful helper for one-off control of the FPGA
//! device or testing, and serves as an example implementation for the DBus interface.
//!
//! # Application Flow
//!
//! When invoked, the CLI:
//! 1. Initializes logging via `env_logger` (controlled by `RUST_LOG` environment variable)
//! 2. Parses command-line arguments into the [`Cli`] structure using clap
//! 3. Dispatches to the appropriate handler based on the command:
//!    - [`load_handler`](load::load_handler) for loading bitstreams/overlays
//!    - [`remove_handler`](remove::remove_handler) for removing bitstreams/overlays
//!    - [`set_handler`](set::set_handler) for setting FPGA attributes
//!    - [`status_handler`](status::status_handler) for querying device status
//! 4. Prints success messages or logs errors and exits with appropriate status code
//!
//! All operations communicate with the fpgad daemon over DBus and are executed asynchronously
//! using tokio.
//!
//! # Environment Variables
//!
//! - `RUST_LOG` - Controls logging level (`trace`, `debug`, `info`, `warn`, `error`, or `off`)
//!
//! # Common Concepts
//!
//! The following concepts are shared across all CLI submodules ([`load`], [`remove`], [`set`], [`status`]).
//!
//! ## Device Handles
//! [Device Handles]: #device-handles
//!
//! A "device handle" refers to the name of an FPGA device as it appears in
//! `/sys/class/fpga_manager/`. Common examples include:
//! - `fpga0` - The first FPGA device
//! - `fpga1` - The second FPGA device (if multiple FPGAs are present)
//!
//! These handles uniquely identify FPGA devices in the system and are used throughout
//! the CLI to specify which device to operate on.
//!
//! ## Overlay Handles
//! [Overlay Handles]: #overlay-handles
//!
//! An "overlay handle" refers to the name of a device tree overlay as it appears in
//! `/sys/kernel/config/device-tree/overlays/`. Common examples include:
//! - `overlay0` - A generic overlay name
//! - `fpga-design` - A custom overlay name specified during loading
//!
//! These handles are used to identify and manage loaded device tree overlays. When loading
//! an overlay, you can specify a custom handle or let the system choose one based on the
//! device handle.
//!
//! ## Error Handling
//! [Error Handling]: #error-handling
//!
//! All CLI functions communicate with the fpgad daemon via DBus and return
//! `Result<String, zbus::Error>` (or variants with `Vec<String>` or `HashMap<String, String>`).
//!
//! When the fpgad daemon returns an application-level error (not a DBus communication
//! error), the error will be of type `zbus::Error::Failure` and the error message will
//! begin with `FpgadError::<variant>:` followed by the error details. For example:
//! ```text
//! FpgadError::Argument: Device fpga0 not found.
//! FpgadError::IOWrite: Failed to write bitstream: Permission denied
//! FpgadError::IORead: Failed to read state: No such file or directory
//! ```
//!
//! This allows callers to distinguish between:
//! - **DBus communication errors** - Problems connecting to or communicating with the daemon
//! - **Application errors** - Errors from the daemon itself (prefixed with `FpgadError::`)
//!
//! # Usage
//!
//! ```text
//! Usage: [snap run] fpgad [OPTIONS] <COMMAND>
//!
//! OPTIONS:
//!   -h, --help            Print help
//!       --handle <DEVICE_HANDLE>  fpga device `HANDLE` to be used for the operations.
//!                        Default value for this option is calculated in runtime
//!                        and the application picks the first available fpga device
//!                        in the system (under `/sys/class/fpga_manager/`)
//!
//! COMMANDS:
//! ├── load                Load a bitstream or overlay
//! │   ├── overlay <FILE> [--handle <OVERLAY_HANDLE>]
//! │   │       Load overlay (.dtbo) into the system using the default OVERLAY_HANDLE
//! │   │           (either the provided DEVICE_HANDLE or "overlay0") or provide
//! │   │       --handle: to name the overlay directory
//! │   └── bitstream <FILE>
//! │           Load bitstream (e.g. `.bit.bin` file) into the FPGA
//! │
//! ├── set <ATTRIBUTE> <VALUE>
//! │       Set an attribute/flag under `/sys/class/fpga_manager/<DEVICE_HANDLE>/<ATTRIBUTE>`
//! │
//! ├── status [--handle <DEVICE_HANDLE>]
//! │       Show FPGA status (all devices and overlays) or provide
//! │       --handle: for a specific device status
//! │
//! └── remove              Remove an overlay or bitstream
//!     ├── overlay [--handle <OVERLAY_HANDLE>]
//!     │       Removes the first overlay found (call repeatedly to remove all) or provide
//!     │       --handle: to remove overlay previously loaded with given OVERLAY_HANDLE
//!     └── bitstream
//!             Remove active bitstream from FPGA (bitstream removal is vendor specific)
//! ```
//!
//! ### Loading
//!
//! ```shell
//! fpgad [--handle=<device_handle>] load ( (overlay <file> [--handle=<handle>]) | (bitstream <file>) )
//! ```
//!
//! ### Removing
//!
//! ```shell
//! fpgad [--handle=<device_handle>] remove ( ( overlay <HANDLE> ) | ( bitstream ) )
//! ```
//!
//! ### Set
//!
//! ```shell
//! fpgad [--handle=<device_handle>] set ATTRIBUTE VALUE
//! ```
//!
//! ### Status
//!
//! ```shell
//! fpgad [--handle=<device_handle>] status
//! ```
//!
//! ## examples (for testing)
//!
//! ### Load
//!
//! ```shell
//! sudo ./target/debug/cli load bitstream /lib/firmware/k26-starter-kits.bit.bin
//! sudo ./target/debug/cli --handle=fpga0 load bitstream /lib/firmware/k26-starter-kits.bit.bin
//!
//! sudo ./target/debug/cli load overlay /lib/firmware/k26-starter-kits.dtbo
//! sudo ./target/debug/cli load overlay /lib/firmware/k26-starter-kits.dtbo --handle=overlay_handle
//! sudo ./target/debug/cli --handle=fpga0 load overlay /lib/firmware/k26-starter-kits.dtbo --handle=overlay_handle
//! ```
//!
//! ### Remove
//!
//! ```shell
//! sudo ./target/debug/cli --handle=fpga0 remove overlay
//! sudo ./target/debug/cli --handle=fpga0 remove overlay --handle=overlay_handle
//! ```
//!
//! ### Set
//!
//! ```shell
//! sudo ./target/debug/cli set flags 0
//! sudo ./target/debug/cli --handle=fpga0 set flags 0
//! ```
//!
//! ### Status
//!
//! ```shell
//! ./target/debug/cli status
//! ./target/debug/cli --handle=fpga0 status
//! ```

mod proxies;

pub mod load;

pub mod remove;

pub mod status;

pub mod set;

use clap::{Parser, Subcommand};

/// Command-line interface structure for FPGA management operations.
///
/// This structure represents the top-level CLI interface for interacting with FPGA devices
/// through the fpgad daemon's DBus interface. It provides a unified interface for loading
/// bitstreams and overlays, querying device status, setting attributes, and removing
/// loaded components.
///
/// # Examples
///
/// ```bash
/// # Load a bitstream
/// fpgad load bitstream /lib/firmware/design.bit.bin
///
/// # Check status of all FPGA devices
/// fpgad status
///
/// # Load an overlay with a specific handle
/// fpgad load overlay /lib/firmware/overlay.dtbo --handle=my_overlay
/// ```
#[derive(Parser, Debug)]
#[command(name = "fpga")]
#[command(bin_name = "fpga")]
pub struct Cli {
    /// fpga device `HANDLE` to be used for the operations.
    /// Default value for this option is calculated in runtime and the application
    /// picks the first available fpga in the system (under /sys/class/fpga_manager)
    #[arg(long = "handle")]
    pub handle: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands for loading FPGA components.
///
/// This enum defines the types of components that can be loaded onto an FPGA device:
/// - **Overlay**: Device tree overlays (.dtbo files) that describe hardware configuration
/// - **Bitstream**: FPGA configuration bitstreams (.bit.bin files) containing the actual FPGA design
///
/// Device tree overlays are typically loaded before or after bitstreams to properly configure
/// the kernel's view of the FPGA's hardware interfaces and peripherals.
#[derive(Subcommand, Debug)]
pub enum LoadSubcommand {
    /// Load overlay into the system
    Overlay {
        /// Overlay `FILE` to be loaded (typically .dtbo)
        file: String,

        /// `HANDLE` for the overlay directory which will be created
        /// under "/sys/kernel/config/device-tree/overlays"
        #[arg(long = "handle")]
        handle: Option<String>,
    },
    /// Load bitstream into the system
    Bitstream {
        /// Bitstream `FILE` to be loaded (typically .bit.bin)
        file: String,
    },
}

/// Subcommands for removing FPGA components.
///
/// This enum defines the types of components that can be removed from an FPGA device:
/// - **Overlay**: Removes a device tree overlay by its handle.
/// - **Bitstream**: Intended to remove the currently loaded FPGA bitstream (vendor-specific
///   operation)
///
/// Removing overlays is important for proper cleanup when reconfiguring the FPGA.
/// Bitstream removal support depends on the FPGA vendor and platform capabilities.
#[derive(Subcommand, Debug)]
pub enum RemoveSubcommand {
    /// Remove overlay with the `HANDLE` provided
    Overlay {
        /// `HANDLE` is the handle that is given during `load` operation
        /// it is different than device_handle which is being used for platform
        /// detection logic.
        #[arg(long = "handle")]
        handle: Option<String>,
    },
    /// Remove bitstream loaded in given `HANDLE` to fpga command
    Bitstream,
}

/// Top-level commands supported by the CLI.
///
/// This enum represents all the primary operations available through the fpgad CLI:
/// - **Load**: Load bitstreams or device tree overlays onto the FPGA
/// - **Set**: Configure FPGA attributes and flags (e.g., programming flags)
/// - **Status**: Query the current state of FPGA devices and loaded overlays
/// - **Remove**: Unload bitstreams or device tree overlays from the FPGA
///
/// Each command communicates with the fpgad daemon via DBus to perform privileged
/// operations on FPGA devices managed through the Linux kernel's FPGA subsystem.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Load a bitstream or an overlay for the given device handle
    Load {
        #[command(subcommand)]
        command: LoadSubcommand,
    },
    /// Set an option (e.g. flags) to a specific value for a given device handle
    Set { attribute: String, value: String },
    /// Get the status information for the given device handle
    Status,
    /// Remove bitstream or an overlay
    Remove {
        #[command(subcommand)]
        command: RemoveSubcommand,
    },
}
