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

use crate::error::FpgadError;
use crate::platforms::platform::{Fpga, OverlayHandler, Platform};
use log::trace;

use crate::error::FpgadError::ArgumentError;
use crate::platforms::universal_components::universal_fpga::UniversalFPGA;
use crate::platforms::universal_components::universal_overlay_handler::UniversalOverlayHandler;
use std::path::Path;

#[derive(Debug)]
pub struct UniversalPlatform {
    name: &'static str,
    fpga: Option<UniversalFPGA>,
    overlay_handler: Option<UniversalOverlayHandler>,
}

impl UniversalPlatform {
    /// Creates a new [`UniversalPlatform`].
    pub fn new() -> Self {
        trace!("creating new universal_platform");
        UniversalPlatform {
            name: "Universal",
            fpga: None,
            overlay_handler: None,
        }
    }

    pub(crate) fn load_package(
        &mut self,
        bitstream_path: &Path,
        overlay_source_path: &Path,
    ) -> Result<(), FpgadError> {
        println!(
            "Attempting to load {:?}, using overlayfs with {:?}",
            bitstream_path, overlay_source_path
        );
        let overlay_handler = self
            .overlay_handler
            .get_or_insert_with(|| UniversalOverlayHandler::new(overlay_source_path));

        let fpga = self
            .fpga
            .as_mut()
            .ok_or(FpgadError::Internal("FPGA not initialized".into()))?;

        if !bitstream_path.exists() | bitstream_path.is_dir() {
            return Err(ArgumentError(format!(
                "Bitstream file '{:?}' has invalid path. Please ensure the path exists.",
                bitstream_path
            )));
        }

        trace!("overlay handler: {:?}", overlay_handler);
        trace!("FPGA: {:?}", fpga);

        if let Ok(flags) = overlay_handler.get_required_flags() {
            fpga.set_flags(flags)?;
        }

        overlay_handler.prepare_for_load()?;
        overlay_handler.apply_overlay()?;

        fpga.assert_state()?;
        println!("Done loading {:?}.", bitstream_path);
        Ok(())
    }

    /// Removes the overlay, undoing any extra steps, and then deletes the overlay_handler
    pub(crate) fn unload_package(&mut self) -> Result<(), FpgadError> {
        self.overlay_handler.take();
        Ok(())
    }
}

impl Platform for UniversalPlatform {
    /// Returns the `name` of the [`UniversalPlatform`]
    fn name(&self) -> &str {
        self.name
    }
    /// Initialises or get the fpga object called `name`
    fn fpga(&mut self, name: &str) -> &impl Fpga {
        assert!(
            !name.is_empty() && name.is_ascii(),
            "fpga name must be compliant with sysfs rules."
        );

        // Create FPGA if not same or present
        if self.fpga.as_ref().is_none_or(|f| f.name != name) {
            self.fpga = Some(UniversalFPGA::new(name));
        }
        self.fpga.as_ref().unwrap()
    }

    /// Gets the `overlay_handler` associated with this device.
    fn overlay_handler(&mut self, overlay_source_path: &Path) -> &impl OverlayHandler {
        // Create FPGA if not same or present
        if self.overlay_handler.as_ref().is_none() {
            self.overlay_handler = Some(UniversalOverlayHandler::new(overlay_source_path));
        }
        self.overlay_handler.as_ref().unwrap()
    }
}
