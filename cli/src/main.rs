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

//! This is FPGAd's commandline interface (CLI) . Due to strict confinement of the snap, this can only be used from a terminal or from a script which is not part of another snap. It is a useful helper for one-off control of the FPGA device or testing, and serves as an example implementation for the DBus interface.
//!
//! ```text
//!Usage: [snap run] fpgad [OPTIONS] <COMMAND>
//!
//! OPTIONs:
//!   -h, --help            Print help
//!       --handle <DEVICE_HANDLE>  fpga device `HANDLE` to be used for the operations.
//!                        Default value for this option is calculated in runtime
//!                        and the application picks the first available fpga device
//!                        in the system (under `/sys/class/fpga_manager/`)
//!
//! COMMANDs:
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

#[derive(Parser, Debug)]
#[command(name = "fpga")]
#[command(bin_name = "fpga")]
struct Cli {
    /// fpga device `HANDLE` to be used for the operations.
    /// Default value for this option is calculated in runtime and the application
    /// picks the first available fpga in the system (under /sys/class/fpga_manager)
    #[arg(long = "handle")]
    handle: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum LoadSubcommand {
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

#[derive(Subcommand, Debug)]
enum RemoveSubcommand {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();
    debug!("parsed cli command with {cli:?}");
    let result = match cli.command {
        Commands::Load { command } => load_handler(&cli.handle, &command).await,
        Commands::Remove { command } => remove_handler(&cli.handle, &command).await,
        Commands::Set { attribute, value } => set_handler(&cli.handle, &attribute, &value).await,
        Commands::Status => status_handler(&cli.handle).await,
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
