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

//! Xilinx DFX Manager overlay handler implementation.
//!
//! This module provides the [`XilinxDfxMgrOverlayHandler`] struct, which implements the
//! [`OverlayHandler`] trait for managing device tree overlays on Xilinx FPGA devices
//! using the dfx-mgr backend.
//!
//! # Key Features
//!
//! - **Slot-based Management**: Uses dfx-mgr's slot system for overlay tracking and removal
//!
//! # Overlay Loading Process
//!
//! 1. Parse the .dtbo file to extract the `firmware-name` property
//! 2. Locate the bitstream file in the same directory as the .dtbo
//! 3. Call `dfx-mgr-client -o <dtbo_path> -b <bitstream_path>` to load both
//!
//! This ensures the bitstream and overlay are applied together, which is required
//! as a temporary workaround while `-load` is not supported due to snap confinement limitations
//!
//! # Examples
//!
//! ```rust,no_run
//! # use daemon::platforms::platform::platform_for_known_platform;
//! # use std::path::Path;
//! # fn example() -> Result<(), daemon::error::FpgadError> {
//! let platform = platform_for_known_platform("xlnx,zynqmp-pcap-fpga")?;
//! let handler = platform.overlay_handler("my_overlay")?;
//! handler.apply_overlay(Path::new("/lib/firmware/design.dtbo"), Path::new(""))?;
//! # Ok(())
//! # }
//! ```

use crate::error::FpgadError;
use crate::platforms::platform::OverlayHandler;
use crate::softeners::error::FpgadSoftenerError;
use crate::softeners::{xilinx_dfx_mgr, xilinx_dfx_mgr_helpers};
use crate::system_io;
use log::{debug, trace};
use std::option::Option;
use std::path::Path;

/// Xilinx DFX Manager overlay handler implementation.
///
/// This struct provides overlay management for Xilinx FPGA devices using the
/// dfx-mgr backend. Unlike the universal overlay handler, it doesn't directly
/// manage configfs overlay directories since dfx-mgr handles that internally.
///
/// # Implementation Notes
///
/// The overlay handle parameter is currently unused since dfx-mgr manages
/// overlay lifecycle internally through its slot system.
pub struct XilinxDfxMgrOverlayHandler {}

impl XilinxDfxMgrOverlayHandler {
    /// Create a new XilinxDfxMgrOverlayHandler instance.
    ///
    /// # Arguments
    ///
    /// * `_overlay_handle` - Overlay handle (currently unused, slot management WIP)
    ///
    /// # Returns: `Self`
    /// * New XilinxDfxMgrOverlayHandler instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use daemon::softeners::xilinx_dfx_mgr_overlay_handler::XilinxDfxMgrOverlayHandler;
    ///
    /// let handler = XilinxDfxMgrOverlayHandler::new("");
    /// ```
    pub(crate) fn new(_: &str) -> Self {
        XilinxDfxMgrOverlayHandler {}
    }
}
impl OverlayHandler for XilinxDfxMgrOverlayHandler {
    fn apply_overlay(&self, source_path: &Path, lookup_path: &Path) -> Result<String, FpgadError> {
        trace!(
            "apply_overlay called with source_path='{}', lookup_path='{}'",
            source_path.display(),
            lookup_path.display()
        );
        let (parent_dir, _) = system_io::extract_path_and_filename(source_path)?;
        // Extract firmware-name from the .dtbo file using fdtdump
        let firmware_name = xilinx_dfx_mgr_helpers::extract_firmware_name(source_path)?;
        debug!("Extracted firmware-name='{}' from dtbo", firmware_name);

        // Construct the bitstream path in the lookup_path
        let bitstream_path = parent_dir.join(&firmware_name);

        // Verify the bitstream file exists
        if !bitstream_path.exists() {
            return Err(FpgadSoftenerError::DfxMgr(format!(
                "Bitstream file '{}' not found in lookup path '{}'",
                firmware_name,
                parent_dir.display()
            ))
            .into());
        }

        trace!("Found bitstream at '{}'", bitstream_path.display());

        // Call dfx-mgr with -o <source_path> -b <bitstream_path>
        xilinx_dfx_mgr::load_overlay(&bitstream_path, source_path).map_err(|e| e.into())
    }

    fn remove_overlay(&self, slot_handle: Option<&str>) -> Result<String, FpgadError> {
        Ok(xilinx_dfx_mgr::remove(slot_handle)?)
    }

    fn required_flags(&self) -> Result<isize, FpgadError> {
        todo!()
    }

    fn status(&self) -> Result<String, FpgadError> {
        Ok(xilinx_dfx_mgr::list_package()?)
    }

    fn overlay_fs_path(&self) -> Result<&Path, FpgadError> {
        Err(FpgadSoftenerError::DfxMgr("Not Applicable".to_string()).into())
    }
}
