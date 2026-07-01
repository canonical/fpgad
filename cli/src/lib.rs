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
//!                                   Examples: "xlnx-sys", "xlnx,zynqmp-pcap-fpga"
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
//! sudo ./target/debug/cli --platform=xlnx-sys load bitstream /lib/firmware/k26-starter-kits.bit.bin
//! sudo ./target/debug/cli --platform=xlnx-sys load bitstream /lib/firmware/k26-starter-kits.bit.bin
//!
//! sudo ./target/debug/cli load overlay /lib/firmware/k26-starter-kits.dtbo
//! sudo ./target/debug/cli load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
//! sudo ./target/debug/cli --device=fpga0 load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
//! sudo ./target/debug/cli --platform=xlnx-sys load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
//! sudo ./target/debug/cli --platform=xlnx-sys --device=fpga0 load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
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

// (xlnx and dfx-mgr subcommands are documented on their respective enum variants below)
// TODO: move this line of docs somewhere useful
mod proxies;

pub mod load;

pub mod remove;

pub mod status;

pub mod set;

pub mod xlnx_sys;

pub mod dfx_mgr;

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
#[command(name = "fpgad")]
#[command(bin_name = "fpgad")]
pub struct Cli {
    /// Platform override string (bypasses platform detection logic).
    /// When provided, this platform string is passed directly to the daemon
    /// instead of auto-detecting from the device handle.
    /// Examples: "xlnx-sys", "xlnx,zynqmp-pcap-fpga"
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

impl Cli {
    /// Returns the platform override string, if provided.
    pub fn platform(&self) -> Option<&String> {
        self.platform.as_ref()
    }

    /// Returns the device handle, if provided.
    pub fn device(&self) -> Option<&String> {
        self.device.as_ref()
    }

    /// Returns a reference to the command.
    pub fn command(&self) -> &Commands {
        &self.command
    }
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
pub enum LoadSubcommand {
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
pub enum RemoveSubcommand {
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

/// Subcommands for the xlnx-sys platform interface.
///
/// Provides direct access to the daemon's `xlnx_sys` read/write DBus methods,
/// allowing low-level control of FPGA manager sysfs properties and flags.
///
/// # Valid `sub_cmd` values
///
/// **Read** (`fpgad xlnx-sys read <sub_cmd> <path>`):
///
/// | `sub_cmd` | `path` | Description |
/// |-----------|--------|-------------|
/// | `read_flags` | Device handle or full sysfs path to flags, e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags` | Read the current programming flags |
/// | `read_property` | Full sysfs path e.g. `/sys/class/fpga_manager/fpga0/name` | Read a sysfs property string |
///
/// **Write** (`fpgad xlnx-sys write <sub_cmd> <path> <value>`):
///
/// | `sub_cmd` | `path` | `value` | Description |
/// |-----------|--------|---------|-------------|
/// | `write_flags` | Device handle or full sysfs path to flags, e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags` | Hex `u32` with or without `0x` prefix (e.g. `0x20` or `20`, both = 32) | Set FPGA programming flags |
/// | `write_property` | Full sysfs path | String payload | Write a string to a sysfs property |
/// | `write_property_bytes` | Full sysfs path | Hex byte string | Write raw bytes to a sysfs property |
///
/// # Examples
///
/// ```shell
/// fpgad xlnx-sys read read_flags fpga0
/// fpgad xlnx-sys read read_property /sys/class/fpga_manager/fpga0/name
/// fpgad xlnx-sys write write_flags fpga0 0x20
/// fpgad xlnx-sys write write_property /sys/class/fpga_manager/fpga0/key VALUE
/// fpgad xlnx-sys write write_property_bytes /sys/class/fpga_manager/fpga0/key deadbeef
/// ```
#[derive(Subcommand, Debug)]
pub enum XlnxSysSubcommand {
    /// Read an FPGA property using the xlnx interface
    Read {
        /// Read operation to perform: `read_flags` or `read_property`.
        ///
        /// * `read_flags` — `path` is a device handle or full sysfs path to flags, e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags`.
        ///
        /// * `read_property` — `path` is the full sysfs path, e.g. `/sys/class/fpga_manager/fpga0/name`.
        ///
        /// See: <https://docs.rs/fpgad/latest/fpgad/platforms/xlnx_sys/enum.ReadSubCommand.html>
        sub_cmd: String,
        /// For `read_flags`: device handle or full sysfs path, e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags`.
        ///
        /// For `read_property`: full sysfs path, e.g. `/sys/class/fpga_manager/fpga0/name`.
        path: String,
    },
    /// Write an FPGA property using the xlnx_sys interface
    Write {
        /// Write operation to perform: `write_flags`, `write_property`, or `write_property_bytes`.
        ///
        /// * `write_flags` — `path` is a device handle or full sysfs path to flags, e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags`; `value` is a hex `u32` with or without `0x` prefix
        ///   (e.g. `0x20` or `20`, both = decimal 32).
        ///
        /// * `write_property` — `path` is a full sysfs path; `value` is a string payload.
        ///
        /// * `write_property_bytes` — `path` is a full sysfs path; `value` is a hex byte string, e.g. `deadbeef`.
        ///
        /// See: <https://docs.rs/fpgad/latest/fpgad/platforms/xlnx_sys/enum.WriteSubCommand.html>
        sub_cmd: String,
        /// For `write_flags`: device handle or full sysfs path to flags, e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags`.
        ///
        /// For `write_property` / `write_property_bytes`: full sysfs path under
        /// `/sys/class/fpga_manager/`.
        path: String,
        /// Value to write.
        ///
        /// For `write_flags`: hex `u32` with or without `0x` prefix (e.g. `0x20` or `20`, both = 32).
        ///
        /// For `write_property`: string payload.
        ///
        /// For `write_property_bytes`: hex-encoded byte string, e.g. `deadbeef`.
        value: String,
    },
}

/// Top-level commands supported by the CLI.
///
/// This enum represents all the primary operations available through the fpgad CLI:
/// - **Load**: Load bitstreams or device tree overlays onto the FPGA
/// - **Set**: Configure FPGA attributes and flags (e.g., programming flags)
/// - **Status**: Query the current state of FPGA devices and loaded overlays
/// - **Remove**: Unload bitstreams or device tree overlays from the FPGA
/// - **XlnxSys**: Low-level read/write access to FPGA manager properties via the xlnx_sys interface
/// - **DfxMgr**: Pass commands directly to `dfx-mgr-client` (requires dfx-mgr component)
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
/// fpgad --platform=xlnx-sys load overlay /lib/firmware/overlay.dtbo --name=my_overlay
///
/// # Set flags for a device
/// fpgad --device=fpga0 set flags 0
///
/// # Get status for all devices
/// fpgad status
///
/// # Remove an overlay by name
/// fpgad remove overlay --name=my_overlay
///
/// # Read FPGA flags via xlnx_sys interface
/// fpgad xlnx-sys read read_flags fpga0
///
/// # Write flags via xlnx_sys interface
/// fpgad xlnx-sys write write_flags fpga0 0x20
///
/// # Invoke dfx-mgr-client
/// fpgad dfx-mgr "-listPackage"
/// ```
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
    /// Low-level read/write access to FPGA manager properties (xlnx_sys platform interface)
    XlnxSys {
        #[command(subcommand)]
        command: XlnxSysSubcommand,
    },
    /// Pass a command directly to `dfx-mgr-client` (requires the `dfx-mgr` snap component).
    ///
    /// This is a thin passthrough to the Xilinx DFX Manager client binary.  The arguments are
    /// forwarded verbatim to `dfx-mgr-client`, so any flag or option that the tool accepts can be
    /// used here.
    ///
    /// The `dfx-mgr` component must be installed:
    /// ```shell
    /// sudo snap install fpgad+dfx-mgr
    /// ```
    ///
    /// For a full list of `dfx-mgr-client` commands and options, see the upstream project:
    /// <https://github.com/Xilinx/dfx-mgr>
    ///
    /// # Common commands
    ///
    /// | Example | Description |
    /// |---------|-------------|
    /// | `fpgad dfx-mgr -listPackage` | List all available acceleration packages |
    /// | `fpgad dfx-mgr -listSlot`    | List all FPGA slots and their current state |
    /// | `fpgad dfx-mgr -load 0 <package_name>` | Load a package into slot 0 |
    /// | `fpgad dfx-mgr -remove 0`   | Remove the package loaded in slot 0 |
    ///
    /// # Notes
    ///
    /// * Arguments that begin with `-` (such as `-listPackage`) are supported directly — no `--`
    ///   separator is required.
    /// * Multiple tokens are accepted: `fpgad dfx-mgr -load 0 my_design`
    DfxMgr {
        /// One or more arguments to pass to `dfx-mgr-client`.
        ///
        /// Tokens beginning with `-` are accepted without needing a `--` separator, so commands
        /// like `-listPackage` or `-load 0 my_design` work naturally:
        ///
        /// ```shell
        /// fpgad dfx-mgr -listPackage
        /// fpgad dfx-mgr -load 0 my_design
        /// ```
        #[arg(allow_hyphen_values = true, num_args = 1.., value_name = "CMD")]
        cmd: Vec<String>,
    },
    /// Generate a shell completion script and print it to stdout.
    ///
    /// This is primarily used at packaging time (the snap wires the generated bash
    /// script up via the `completer` keyword), but it can also be sourced manually:
    ///
    /// ```shell
    /// # Enable completions for the current shell session
    /// source <(fpgad completions bash)
    ///
    /// # Or install them permanently for the current user
    /// fpgad completions bash > ~/.local/share/bash-completion/completions/fpgad
    /// ```
    Completions {
        /// Shell to generate the completion script for (e.g. `bash`, `zsh`, `fish`).
        shell: clap_complete::Shell,
    },
}
