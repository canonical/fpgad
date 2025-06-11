// This file is part of fpgad, an application to manage FPGA subsystem together with devicetree and kernel modules.
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

use crate::{error::FpgadError, system_io::fs_read};
use log::trace;

use super::platform::{Fpga, OverlayHandler, Platform};

#[derive(Debug)]
pub struct UniversalPlatform {
    name: &'static str,
    fpga: Option<UniversalFPGA>,
    overlay_handler: Option<UniversalOverlayHandler>,
}

impl UniversalPlatform {
    /// Creates a new [`UniversalPlatform`].
    pub(crate) fn new() -> Self {
        trace!("creating new UniversalPlatform");
        UniversalPlatform {
            name: "Universal",
            fpga: None,
            overlay_handler: None,
        }
    }
}

impl Platform for UniversalPlatform {
    /// Returns the `name` of the [`UniversalPlatform`]
    fn name(&self) -> &str {
        self.name
    }

    fn fpga(&mut self, name: &str) -> &impl Fpga {
        assert!(
            !name.is_empty() && name.is_ascii(),
            "fpga name must be compliant with sysfs rules."
        );

        // Create FPGA if not same or present
        if self.fpga.as_ref().is_none_or(|f| f.name != name) {
            self.fpga = Some(UniversalFPGA::new(name.to_string()));
        }
        self.fpga.as_ref().unwrap()
    }

    fn overlay_handler(&self) -> &impl OverlayHandler {
        self.overlay_handler.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub struct UniversalOverlayHandler {}

impl OverlayHandler for UniversalOverlayHandler {
    fn apply_devicetree(&self) -> bool {
        todo!()
    }

    fn unapply_devicetree(&self) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct UniversalFPGA {
    name: String,
}

impl UniversalFPGA {
    pub(crate) fn new(name: String) -> Self {
        UniversalFPGA { name }
    }
}

impl Fpga for UniversalFPGA {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> Result<String, FpgadError> {
        trace!("reading /sys/class/fpga_manager/{}/state", self.name);
        match fs_read(&format!("/sys/class/fpga_manager/{}/state", self.name)) {
            Ok(val) => Ok(val),
            Err(e) => Err(FpgadError::Io(e)),
        }
    }

    fn load_bitstream(&self) -> bool {
        todo!()
    }

    fn unload_bitstream(&self) -> bool {
        todo!()
    }
}
