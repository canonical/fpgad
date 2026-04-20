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

//! This is FPGAd's commandline interface (CLI) . Due to strict confinement of the snap, this can
//! only be used from a terminal or from a script which is not part of another snap.
//! It is a useful helper for one-off control of the FPGA device or testing, and serves as an
//! example implementation for the DBus interface.
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
//!   -h, --help                      Print help
//!   -p, --platform <PLATFORM>       Platform override string (bypasses platform detection logic).
//!                                   When provided, this platform string is passed directly to the
//!                                   daemon instead of auto-detecting from the device handle.
//!                                   Examples: "universal", "xlnx,zynqmp-pcap-fpga"
//!   -d, --device <DEVICE_HANDLE>    FPGA device handle to be used for the operations.
//!                                   Default value is calculated at runtime - the application
//!                                   picks the first available FPGA device in the system
//!                                   (under `/sys/class/fpga_manager/`).
//!                                   Examples: "fpga0", "fpga1"
//!
//! SUBCOMMAND OPTIONS:
//!   -n, --name <OVERLAY_NAME>       (Used with load/remove overlay subcommands)
//!                                   Name for the overlay directory in configfs
//!                                   (under `/sys/kernel/config/device-tree/overlays/`).
//!                                   If not provided, defaults to the device handle or "overlay0".
//!
//! COMMANDS:
//! ├── load                Load a bitstream or overlay
//! │   ├── overlay <FILE> [--name <OVERLAY_HANDLE> --platform <PLATFORM>]
//! │   │       Load overlay (.dtbo) into the system using the default OVERLAY_HANDLE
//! │   │           (either the provided DEVICE_HANDLE or "overlay0") or provide
//! │   │       --name: to name the overlay directory
//! │   └── bitstream <FILE> [--platform <PLATFORM>]
//! │           Load bitstream (e.g. `.bit.bin` file) into the FPGA
//! │
//! ├── set <ATTRIBUTE> <VALUE>
//! │       Set an attribute/flag under `/sys/class/fpga_manager/<DEVICE_HANDLE>/<ATTRIBUTE>`
//! │
//! ├── status [--device <DEVICE_HANDLE> --platform <PLATFORM>]
//! │       Show FPGA status (all devices and overlays) or provide
//! │       --device: for a specific device status
//! │
//! └── remove              Remove an overlay or bitstream
//!     ├── overlay [--name <OVERLAY_HANDLE> --platform <PLATFORM>]
//!     │       Removes the first overlay found (call repeatedly to remove all) or provide
//!     │       --name: to remove overlay previously loaded with given OVERLAY_HANDLE
//!     └── bitstream [--name <BITSTREAM_HANDLE> --platform <PLATFORM>]
//!             Remove active bitstream from FPGA (bitstream removal is vendor specific)
//! ```
//!
//! ### Loading
//!
//! ```shell
//! fpgad [--device=<device_handle>] [--platform=<platform>] load ( (overlay <file> [--name=<overlay_name>]) | (bitstream <file>) )
//! ```
//!
//! ### Removing
//!
//! ```shell
//! fpgad [--device=<device_handle>] [--platform=<platform>] remove ( ( overlay [--name=<overlay_name>] ) | ( bitstream ) )
//! ```
//!
//! ### Set
//!
//! ```shell
//! fpgad [--device=<device_handle>] set ATTRIBUTE VALUE
//! ```
//!
//! ### Status
//!
//! ```shell
//! fpgad [--device=<device_handle>] [--platform=<platform>] status
//! ```
//!
//! ## examples (for testing)
//!
//! ### Load
//!
//! ```shell
//! sudo ./target/debug/cli load bitstream /lib/firmware/k26-starter-kits.bit.bin
//! sudo ./target/debug/cli --device=fpga0 load bitstream /lib/firmware/k26-starter-kits.bit.bin
//! sudo ./target/debug/cli --platform=universal load bitstream /lib/firmware/k26-starter-kits.bit.bin
//! sudo ./target/debug/cli --platform=xlnx load bitstream /lib/firmware/k26-starter-kits.bit.bin
//!
//! sudo ./target/debug/cli load overlay /lib/firmware/k26-starter-kits.dtbo
//! sudo ./target/debug/cli load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
//! sudo ./target/debug/cli --device=fpga0 load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
//! sudo ./target/debug/cli --platform=universal load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
//! sudo ./target/debug/cli --platform=xlnx --device=fpga0 load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
//! ```
//!
//! ### Remove
//!
//! ```shell
//! sudo ./target/debug/cli --device=fpga0 remove overlay
//! sudo ./target/debug/cli --device=fpga0 remove overlay --name=overlay_handle
//! ```
//!
//! ### Set
//!
//! ```shell
//! sudo ./target/debug/cli set flags 0
//! sudo ./target/debug/cli --device=fpga0 set flags 0
//! ```
//!
//! ### Status
//!
//! ```shell
//! ./target/debug/cli status
//! ./target/debug/cli --device=fpga0 status
//! ```

mod proxies;

mod load;

mod remove;

mod status;

mod set;

use crate::load::load_handler;
use crate::remove::remove_handler;
use crate::set::set_handler;
use crate::status::status_handler;
use clap::{Parser, Subcommand};
use log::{debug, error};
use std::error::Error;

/// Command-line interface structure for FPGA management operations.
///
/// This structure represents the top-level CLI interface for interacting with FPGA devices
/// through the fpgad daemon's DBus interface. It provides a unified interface for loading
/// bitstreams and overlays, querying device status, setting attributes, and removing
/// loaded components.
///
/// # Examples
///
/// ```shell
///
/// # Load a bitstream
/// fpgad load bitstream /lib/firmware/design.bit.bin
///
/// # Check status of all FPGA devices
/// fpgad status
///
/// # Load an overlay with a specific name
/// fpgad load overlay /lib/firmware/overlay.dtbo --name=my_overlay
///
/// ```
#[derive(Parser, Debug)]
#[command(name = "fpga")]
#[command(bin_name = "fpga")]
struct Cli {
    /// Platform override string (bypasses platform detection logic).
    /// When provided, this platform string is passed directly to the daemon
    /// instead of auto-detecting from the device handle.
    /// Examples: "universal", "xlnx,zynqmp-pcap-fpga"
    #[arg(short = 'p', long = "platform")]
    platform: Option<String>,

    /// FPGA `device` handle to be used for the operations.
    /// Default value is calculated at runtime - the application picks the first
    /// available FPGA device in the system (under /sys/class/fpga_manager/).
    /// Examples: "fpga0", "fpga1"
    #[arg(short = 'd', long = "device")]
    device: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

/// Subcommands for loading FPGA components.
///
/// This enum defines the types of components that can be loaded onto an FPGA device:
/// - **Overlay**: Device tree overlays (.dtbo files) that describe hardware configuration
/// - **Bitstream**: FPGA configuration bitstreams (.bit.bin files) containing the actual FPGA design
///
/// Device tree overlays are typically loaded before or after bitstreams to properly configure
/// the kernel's view of the FPGA's hardware interfaces and peripherals.
///
/// # Examples
///
/// ```shell
/// # Load a bitstream
/// fpgad load bitstream [-d=<DEVICE_HANDLE> -p=<COMPAT_STR>] /lib/firmware/design.bit.bin
///
/// # Load an overlay with a custom name
/// fpgad load overlay [-d=<DEVICE_HANDLE> -p=<COMPAT_STR>] /lib/firmware/overlay.dtbo [-n=my_overlay]
/// ```
#[derive(Subcommand, Debug)]
enum LoadSubcommand {
    /// Load overlay into the system
    Overlay {
        /// Overlay `FILE` to be loaded (typically .dtbo)
        file: String,

        /// Name for the overlay directory which will be created
        /// under "/sys/kernel/config/device-tree/overlays/".
        /// If not provided, defaults to the device handle or "overlay0".
        #[arg(short = 'n', long = "name")]
        name: Option<String>,
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
/// - **Overlay**: Removes a device tree overlay by its name.
/// - **Bitstream**: Intended to remove the currently loaded FPGA bitstream (vendor-specific
///   operation that may use slot identifiers on platforms like DFX Manager)
///
/// Removing overlays is important for proper cleanup when reconfiguring the FPGA.
/// Bitstream removal support depends on the FPGA vendor and platform capabilities.
///
/// # Examples
///
/// ```shell
/// # Remove the first overlay found
/// fpgad remove overlay
///
/// # Remove a specific overlay by name
/// fpgad [-d=<DEVICE_HANDLE>] [-p=<COMPAT_STR>] remove overlay -n=my_overlay
///
/// # Remove a bitstream, if supported
/// fpgad [-d=<DEVICE_HANDLE>] [-p=<COMPAT_STR>] remove bitstream -n=0 # for dfx-mgr slot 0
/// ```
#[derive(Subcommand, Debug)]
enum RemoveSubcommand {
    /// Remove overlay with the name provided
    Overlay {
        /// Name of the overlay to remove (as given during `load` operation).
        /// If not provided, removes the first overlay found in the system.
        /// This is different from device_handle which is used for platform detection.
        #[arg(short = 'n', long = "name")]
        name: Option<String>,
    },
    /// Remove bitstream loaded in the given device
    Bitstream {
        /// Handle/identifier for the bitstream to remove.
        /// For DFX Manager platforms, this can be a slot ID.
        /// Use empty string "" to remove the latest bitstream.
        #[arg(long = "handle")]
        handle: Option<String>,
    },
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
///
/// # Examples
///
/// ```shell
/// # Load a bitstream to a specific device
/// fpgad --device=fpga0 load bitstream /lib/firmware/design.bit.bin
///
/// # Load an overlay with platform override
/// fpgad --platform=universal load overlay /lib/firmware/overlay.dtbo --name=my_overlay
///
/// # Set flags for a device
/// fpgad --device=fpga0 set flags 0
///
/// # Get status for all devices
/// fpgad status
///
/// # Remove an overlay by name
/// fpgad remove overlay --name=my_overlay
/// ```
#[derive(Subcommand, Debug)]
enum Commands {
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

/// Main entry point for the FPGA CLI application.
///
/// Initializes the environment logger, parses command-line arguments, and dispatches
/// to the appropriate handler based on the specified command. All operations are
/// performed asynchronously using tokio.
///
/// # Flow
///
/// 1. Initializes logging via `env_logger`
/// 2. Parses CLI arguments into the `Cli` structure
/// 3. Dispatches to the appropriate handler:
///    - `load_handler` for loading bitstreams/overlays
///    - `remove_handler` for removing bitstreams/overlays
///    - `set_handler` for setting FPGA attributes
///    - `status_handler` for querying device status
/// 4. Prints success messages or logs errors
///
/// # Returns
///
/// * `Ok(())` - Command executed successfully
/// * `Err(Box<dyn Error>)` - An error occurred during command execution
///
/// # Examples
///
/// The application is typically invoked from the command line:
///
/// ```bash
/// # Load a bitstream
/// fpgad load bitstream /lib/firmware/design.bit.bin
///
/// # Query status
/// fpgad status
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();
    debug!("parsed cli command with {cli:#?}");
    let result = match cli.command {
        Commands::Load { command } => load_handler(&cli.platform, &cli.device, &command).await,
        Commands::Remove { command } => remove_handler(&cli.platform, &cli.device, &command).await,
        Commands::Set { attribute, value } => {
            set_handler(&cli.platform, &cli.device, &attribute, &value).await
        }
        Commands::Status => status_handler(&cli.platform, &cli.device).await,
    };
    match result {
        Ok(msg) => {
            println!("{msg}");
            Ok(())
        }
        Err(e) => {
            error!("{e}");
            Err(e.into())
        }
    }
}
