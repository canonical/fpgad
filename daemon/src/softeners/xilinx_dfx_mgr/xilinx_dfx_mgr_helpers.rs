// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
//
// Copyright 2026 Canonical Ltd.
//
// SPDX-License-Identifier: GPL-3.0-only
//
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
//
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

//! Helper functions for Xilinx DFX Manager operations.
//!
//! This module provides utility functions for working with Xilinx device tree overlay
//! files and extracting information needed for dfx-mgr operations.
//!
//! # Key Functions
//!
//! - [`get_dfx_mgr_client_path`] - Locates and validates the dfx-mgr-client binary path
//! - [`run_dfx_mgr`] - Executes dfx-mgr-client synchronously with given arguments
//! - [`extract_firmware_name`] - Parses .dtbo files to extract the firmware-name property
//!
//! # Device Tree Parsing
//!
//! The module uses the `fdt` crate to parse flattened device tree binary (.dtbo) files
//! and extract properties. This is essential for coordinating bitstream and overlay loading
//! in the Xilinx dfx-mgr workflow.
//!
//! # Examples
//!
//! ```rust,no_run
//! # use std::path::Path;
//! # use daemon::softeners::xilinx_dfx_mgr_helpers::extract_firmware_name;
//! # fn example() -> Result<(), daemon::error::FpgadError> {
//! let firmware = extract_firmware_name(Path::new("/lib/firmware/design.dtbo"))?;
//! println!("Bitstream file: {}", firmware);
//! # Ok(())
//! # }
//! ```

use crate::error::FpgadError;
use crate::softeners::error::FpgadSoftenerError;
use log::trace;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Get the path to the dfx-mgr-client binary and verify it exists.
///
/// Checks for dfx-mgr-client in the following order:
/// 1. In snap component: `$SNAP_COMPONENTS/dfx-mgr/usr/bin/dfx-mgr-client`
/// 2. In system files: `/usr/bin/dfx-mgr-client` when `$SNAP_COMPONENTS` is not set
///
/// # Returns
/// * `Ok(String)` - The path to dfx-mgr-client if it exists
/// * `Err(FpgadSoftenerError)` - If dfx-mgr-client cannot be found or doesn't exist
///
/// # Examples
///
/// ```rust,no_run
/// # use daemon::softeners::xilinx_dfx_mgr::xilinx_dfx_mgr_helpers::get_dfx_mgr_client_path;
/// # fn example() -> Result<(), daemon::softeners::error::FpgadSoftenerError> {
/// let path = get_dfx_mgr_client_path()?;
/// println!("dfx-mgr-client is at: {}", path);
/// # Ok(())
/// # }
/// ```
pub fn get_dfx_mgr_client_path() -> Result<String, FpgadSoftenerError> {
    // Check for dfx-mgr-client in snap component first, then fall back to system files
    let dfx_mgr_client_path = if let Ok(snap_components) = env::var("SNAP_COMPONENTS") {
        let path = format!("{}/dfx-mgr/usr/bin/dfx-mgr-client", snap_components);
        if !Path::new(&path).exists() {
            return Err(FpgadSoftenerError::DfxMgr(format!(
                "dfx-mgr-client not found at '{path}'.\n\n\
            To enable Xilinx DFX Manager support, install the dfx-mgr component:\n\n\
                `sudo snap install fpgad+dfx-mgr.comp --dangerous`\n\n\
            Or run the CLI using the `universal` platform (no dfx-mgr required):\n\n\
                `fpgad --platform=universal <command>`\n\n\
            If you are calling the daemon over DBus manually \n\n \
                set the `platform_string` to `universal` instead."
            )));
        }
        path
    } else {
        let path = String::from("/usr/bin/dfx-mgr-client");
        if !Path::new(&path).exists() {
            return Err(FpgadSoftenerError::DfxMgr(format!(
                "dfx-mgr-client not found on system at '{path}'.\n\n\
            To enable Xilinx DFX Manager support, install the dfx-mgr binaries e.g.\n\n\
                `sudo apt install dfx-mgr`\n\n\
            Or run the CLI using the `universal` platform (no dfx-mgr required):\n\n\
                `fpgad --platform=universal <command>`\n\n\
            If you are calling the daemon over DBus manually \n\n \
                set the `platform_string` to `universal` instead."
            )));
        }
        path
    };

    Ok(dfx_mgr_client_path)
}

/// Run dfx-mgr-client with the given arguments (synchronous version).
///
/// # Arguments
///
/// * `args` - Command line arguments to pass to dfx-mgr-client
///
/// # Returns
/// * `Ok(String)` - stdout from dfx-mgr-client on success
/// * `Err(FpgadSoftenerError)` - Formatted stdout, stderr and exit status from dfx-mgr-client if dfx-mgr-client
///   execution fails of string explaining that dfx-mgr-client cannot be found
///
/// # Examples
///
/// ```rust,no_run
/// # use daemon::softeners::xilinx_dfx_mgr::xilinx_dfx_mgr_helpers::run_dfx_mgr;
/// # fn example() -> Result<(), daemon::softeners::error::FpgadSoftenerError> {
/// let output = run_dfx_mgr(&["-listPackage"])?;
/// println!("Packages: {}", output);
/// # Ok(())
/// # }
/// ```
pub fn run_dfx_mgr(args: &[&str]) -> Result<String, FpgadSoftenerError> {
    let dfx_mgr_client_path = get_dfx_mgr_client_path()?;

    trace!("Calling dfx-mgr-client with args {:#?}", args);
    let output = Command::new(&dfx_mgr_client_path)
        .args(args)
        .output()
        .map_err(|e| {
            FpgadSoftenerError::DfxMgr(format!("dfx-mgr-client failed to produce output:\n{e}"))
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(FpgadSoftenerError::DfxMgr(format!(
            "dfx-mgr-client failed.\n{}\nStdout:\n{:?}\nStderr:\n{:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Extract the firmware-name property from a device tree overlay (.dtbo) file
///
/// # Arguments
///
/// * `dtbo_path` - Path to the .dtbo file
///
/// # Returns: `Result<String, FpgadError>`
/// * `Ok(String)` - The firmware name extracted from the dtbo
/// * `Err(FpgadError)` - If the dtbo file cannot be read or firmware-name is not found
pub fn extract_firmware_name(dtbo_path: &Path) -> Result<String, FpgadError> {
    trace!("Extracting firmware-name from '{}'", dtbo_path.display());

    // Read the dtbo file
    let dtb_data = fs::read(dtbo_path).map_err(|e| {
        FpgadSoftenerError::DfxMgr(format!(
            "Failed to read dtbo file '{}': {}",
            dtbo_path.display(),
            e
        ))
    })?;

    // Parse the device tree
    let fdt = fdt::Fdt::new(&dtb_data).map_err(|e| {
        FpgadSoftenerError::DfxMgr(format!(
            "Failed to parse dtbo file '{}': {:?}",
            dtbo_path.display(),
            e
        ))
    })?;

    // Search for firmware-name property in all nodes
    for node in fdt.all_nodes() {
        if let Some(firmware_name_prop) = node.property("firmware-name") {
            // The property value is a null-terminated string, extract it
            let value = firmware_name_prop.value;

            // Find the null terminator or use the entire value
            let end = value.iter().position(|&b| b == 0).unwrap_or(value.len());
            let firmware_name = std::str::from_utf8(&value[..end]).map_err(|e| {
                FpgadSoftenerError::DfxMgr(format!(
                    "Failed to parse firmware-name as UTF-8 string: {}",
                    e
                ))
            })?;

            trace!("Found firmware-name='{}' in dtbo", firmware_name);
            return Ok(firmware_name.to_string());
        }
    }

    Err(FpgadSoftenerError::DfxMgr(format!(
        "`firmware-name` property not found in dtbo file '{}'",
        dtbo_path.display()
    ))
    .into())
}

#[cfg(test)]
mod tests {
    use crate::softeners::xilinx_dfx_mgr::xilinx_dfx_mgr_helpers::extract_firmware_name;
    use googletest::prelude::*;
    use std::path::PathBuf;

    #[gtest]
    fn test_extract_firmware_name_k26() {
        let test_dtbo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/test_data/k26-starter-kits/k26_starter_kits.dtbo");
        if !test_dtbo.exists() {
            println!("SKIP: test data not found at {}", test_dtbo.display());
            return;
        }
        let result = extract_firmware_name(&test_dtbo);
        assert_that!(result, ok(eq("k26_starter_kits.bit.bin")));
    }

    #[gtest]
    fn test_extract_firmware_name_k24() {
        let test_dtbo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/test_data/k24-starter-kits/k24_starter_kits.dtbo");
        if !test_dtbo.exists() {
            println!("SKIP: test data not found at {}", test_dtbo.display());
            return;
        }
        let result = extract_firmware_name(&test_dtbo);
        assert_that!(result, ok(eq("k24_starter_kits.bit.bin")));
    }
}
