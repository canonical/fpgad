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

use crate::common::proxies::control_proxy::ControlProxy;
use crate::xilinx_dfx_mgr::{PLATFORM_STRING, setup};
use googletest::prelude::*;
use rstest::*;
use tokio;
use zbus::Connection;
use zbus::Result;

#[gtest]
#[tokio::test]
#[rstest]
#[case::bad_device(
    "dev0",
    0u32,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::no_device(
    "",
    0u32,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::all_good(
    "fpga0",
    0u32,
    ok(contains_substring("Flags set to '0x0' for 'fpga0'"))
)]
#[case::all_good_0x1(
    "fpga0",
    1u32,
    ok(contains_substring("Flags set to '0x1' for 'fpga0'"))
)]
#[case::all_good_0x10(
    "fpga0",
    16u32,
    ok(contains_substring("Flags set to '0x10' for 'fpga0'"))
)]
async fn set_flags_cases<M: for<'a> Matcher<&'a Result<String>>>(
    #[case] device_handle: &str,
    #[case] flags: u32,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let res = proxy
        .set_fpga_flags(PLATFORM_STRING, device_handle, flags)
        .await;
    expect_that!(&res, condition);
}
