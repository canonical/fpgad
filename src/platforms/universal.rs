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

use crate::platforms::platform::{Fpga, OverlayHandler, Platform};
use log::trace;

use crate::platforms::universal_components::universal_fpga::UniversalFPGA;
use crate::platforms::universal_components::universal_overlay_handler::UniversalOverlayHandler;

#[derive(Debug)]
pub struct UniversalPlatform {
    platform_type: &'static str,
    fpga: Option<UniversalFPGA>,
    overlay_handler: Option<UniversalOverlayHandler>,
}

impl UniversalPlatform {
    /// Creates a new [`UniversalPlatform`].
    pub fn new() -> Self {
        trace!("creating new universal_platform");
        UniversalPlatform {
            platform_type: "Universal",
            fpga: None,
            overlay_handler: None,
        }
    }
}

impl Platform for UniversalPlatform {
    /// Returns the `name` of the [`UniversalPlatform`]
    fn platform_type(&self) -> &str {
        self.platform_type
    }
    /// Initialises or get the fpga object called `name`
    fn fpga(&mut self, name: &str) -> &impl Fpga {
        assert!(
            !name.is_empty() && name.is_ascii(),
            "fpga name must be compliant with sysfs rules."
        );

        // Create FPGA if not same or present
        if self.fpga.as_ref().is_none_or(|f| f.device_handle != name) {
            self.fpga = Some(UniversalFPGA::new(name));
        }
        self.fpga.as_ref().unwrap()
    }

    /// Gets the `overlay_handler` associated with this device.
    fn overlay_handler(&mut self) -> &mut dyn OverlayHandler {
        // Create overlay handler if not same or present
        if self.overlay_handler.as_ref().is_none() {
            self.overlay_handler = Some(UniversalOverlayHandler::new());
        }
        self.overlay_handler.as_mut().unwrap()
    }
}
