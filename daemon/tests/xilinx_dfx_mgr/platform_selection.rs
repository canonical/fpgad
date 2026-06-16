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

//! Integration tests for platform selection logic with real platform registrations.
//!
//! These tests verify that `match_platform_string` returns the correct concrete platform
//! type (`XilinxDfxMgrPlatform` or `XilinxSysPlatform`) depending on whether the
//! dfx-mgr-client binary is available on the system.
//!
//! Unlike the unit tests in `platform_discovery_tests`, these run in a separate binary
//! and therefore have an isolated `PLATFORM_REGISTRY` — no interference from other tests
//! that register fake platforms.

use fpgad::platforms::platform::{Platform, match_platform_string, register_platform};
use fpgad::platforms::xilinx_sys::XilinxSysPlatform;
use fpgad::softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;
use googletest::prelude::*;
use std::any::Any;

fn setup_integrated_registry() {
    register_platform(
        "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,dfx-mgr,softener",
        || Box::new(XilinxDfxMgrPlatform::new()),
        XilinxDfxMgrPlatform::is_available,
    );
    register_platform(
        "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,xlnx-sys,platform",
        || Box::new(XilinxSysPlatform::new()),
        XilinxSysPlatform::is_available,
    );
}

fn assert_is_dfx_mgr_platform(platform: &dyn Platform) {
    let as_dfx_mgr = (platform as &dyn Any).downcast_ref::<XilinxDfxMgrPlatform>();
    assert_that!(as_dfx_mgr.is_some(), eq(true));
}

fn assert_is_xlnx_sys_platform(platform: &dyn Platform) {
    let as_xlnx_sys = (platform as &dyn Any).downcast_ref::<XilinxSysPlatform>();
    assert_that!(as_xlnx_sys.is_some(), eq(true));
}

#[gtest]
fn test_integrated_platform_selection() {
    setup_integrated_registry();

    let result = match_platform_string("xlnx,zynqmp-pcap-fpga");
    assert_that!(result.is_ok(), eq(true));

    let platform = result.unwrap();
    if XilinxDfxMgrPlatform::is_available() {
        assert_is_dfx_mgr_platform(platform.as_ref());
    } else {
        assert_is_xlnx_sys_platform(platform.as_ref());
    }
}

#[gtest]
fn test_explicit_softener_request_with_real_platforms() {
    setup_integrated_registry();

    let result = match_platform_string("xlnx,softener");

    if XilinxDfxMgrPlatform::is_available() {
        assert_that!(result.is_ok(), eq(true));
        assert_is_dfx_mgr_platform(result.unwrap().as_ref());
    } else {
        // No built-in platform has "softener" in its compat string, so no fallback
        assert_that!(result.is_err(), eq(true));
    }
}

#[gtest]
fn test_xlnx_component_matches_both() {
    setup_integrated_registry();

    let result = match_platform_string("xlnx");
    assert_that!(result.is_ok(), eq(true));

    let platform = result.unwrap();
    if XilinxDfxMgrPlatform::is_available() {
        assert_is_dfx_mgr_platform(platform.as_ref());
    } else {
        assert_is_xlnx_sys_platform(platform.as_ref());
    }
}
