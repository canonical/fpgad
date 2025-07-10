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

mod status;

mod load;

mod remove;

use crate::load::load_handler;
use crate::remove::remove_handler;
use crate::status::status_handler;
use clap::{Parser, Subcommand, arg, command};
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
    /// Get the status information for the given device handle
    Status,
    /// Load a bitstream or an overlay for the given device handle
    Load {
        #[command(subcommand)]
        command: LoadSubcommand,
    },
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
        Commands::Status => status_handler(&cli.handle).await,
        Commands::Load { command } => load_handler(&cli.handle, &command).await,
        Commands::Remove { command } => remove_handler(&cli.handle, &command).await,
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
