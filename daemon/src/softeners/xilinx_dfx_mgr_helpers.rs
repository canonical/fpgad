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
use std::fs;
use std::path::Path;

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
    use crate::softeners::xilinx_dfx_mgr_helpers::extract_firmware_name;
    use std::path::PathBuf;

    #[test]
    fn test_extract_firmware_name_k26() {
        let test_dtbo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/test_data/k26-starter-kits/k26_starter_kits.dtbo");

        if test_dtbo.exists() {
            let result = extract_firmware_name(&test_dtbo);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "k26-starter-kits.bit.bin");
        }
    }

    #[test]
    fn test_extract_firmware_name_k24() {
        let test_dtbo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/test_data/k24-starter-kits/k24_starter_kits.dtbo");

        if test_dtbo.exists() {
            let result = extract_firmware_name(&test_dtbo);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "k24-starter-kits.bit.bin");
        }
    }
}
