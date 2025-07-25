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
use crate::platforms::universal_components::universal_fpga::UniversalFPGA;
use crate::platforms::universal_components::universal_overlay_handler::UniversalOverlayHandler;
use fpgad_macros::platform;
use log::trace;
use std::sync::OnceLock;

#[platform(compat_string = "universal")]
#[derive(Debug)]
pub struct UniversalPlatform {
    fpga: OnceLock<UniversalFPGA>,
    overlay_handler: OnceLock<UniversalOverlayHandler>,
}

impl UniversalPlatform {
    /// Creates a new [`UniversalPlatform`].
    pub fn new() -> Self {
        trace!("creating new universal_platform");
        UniversalPlatform {
            fpga: OnceLock::new(),
            overlay_handler: OnceLock::new(),
        }
    }
}

impl Platform for UniversalPlatform {
    /// Initialises or get the fpga object called `name`
    fn fpga(&self, device_handle: &str) -> Result<&dyn Fpga, FpgadError> {
        Ok(self.fpga.get_or_init(|| UniversalFPGA::new(device_handle)))
    }

    /// Gets the `overlay_handler` associated with this device.
    fn overlay_handler(&self, overlay_handle: &str) -> Result<&(dyn OverlayHandler), FpgadError> {
        // TODO: replace the return type of UniversalOverlayHandler to Result and use
        // get_or_try_init instead here when stable:
        // https://github.com/rust-lang/rust/issues/121641
        let handler = self
            .overlay_handler
            .get_or_init(|| UniversalOverlayHandler::new(overlay_handle));

        // NOTE: This will fail if the constructor fails.
        let parent_path = handler.overlay_fs_path()?.parent().ok_or_else(|| {
            FpgadError::Argument(format!(
                "The path {:?} has no parent directory.",
                handler.overlay_fs_path()
            ))
        })?;

        if !parent_path.exists() {
            return Err(FpgadError::Argument(format!(
                "The overlayfs path {parent_path:?} doesn't seem to exist."
            )));
        }
        Ok(handler)
    }
}
