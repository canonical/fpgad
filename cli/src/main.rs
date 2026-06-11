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

use clap::Parser;
use fpgad_cli::{
    Cli, Commands, dfx_mgr::dfx_mgr_handler, load::load_handler, remove::remove_handler,
    set::set_handler, status::status_handler, xlnx_sys::xlnx_sys_handler,
};
use log::{debug, error};
use std::error::Error;

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
///    - `xlnx_sys_handler` for low-level property read/write via the xlnx_sys interface
///    - `dfx_mgr_handler` for passing commands to dfx-mgr-client
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
    let result = match cli.command() {
        Commands::Load { command } => load_handler(cli.platform(), cli.device(), command).await,
        Commands::Remove { command } => remove_handler(cli.platform(), cli.device(), command).await,
        Commands::Set { attribute, value } => {
            set_handler(cli.platform(), cli.device(), attribute, value).await
        }
        Commands::Status => status_handler(cli.platform(), cli.device()).await,
        Commands::XlnxSys { command } => xlnx_sys_handler(command).await,
        Commands::DfxMgr { cmd } => dfx_mgr_handler(cmd).await,
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
