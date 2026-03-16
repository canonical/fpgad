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

use crate::error::FpgadError;
use crate::platforms::platform::OverlayHandler;
use crate::softeners::error::FpgadSoftenerError;
use crate::softeners::xilinx_dfx_mgr;
use std::option::Option;

use std::path::Path;

pub struct XilinxDfxMgrOverlayHandler {}

impl XilinxDfxMgrOverlayHandler {
    pub(crate) fn new(_: &str) -> Self {
        XilinxDfxMgrOverlayHandler {}
    }
}
impl OverlayHandler for XilinxDfxMgrOverlayHandler {
    fn apply_overlay(
        &self,
        _source_path: &Path,
        _lookup_path: &Path,
    ) -> Result<String, FpgadError> {
        // todo: search for bitstream in overlay dir
        //  xilinx_dfx_mgr::load_overlay(bitstream_path, _source_path)
        todo!()
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
