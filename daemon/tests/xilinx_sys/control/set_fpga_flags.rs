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
use crate::xilinx_sys::setup;
use googletest::prelude::*;
use rstest::*;
use tokio;
use zbus::Connection;
use zbus::Result;

#[gtest]
#[tokio::test]
#[rstest]
#[case::no_device_handle("", "0", err(displays_as(contains_substring("FpgadError::Argument:"))))]
#[case::bad_device_handle(
    "dev0",
    "0",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_typo_path(
    "/sy/class/fpga_manager/",
    "0",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_short_path(
    "/sys/class/fpga_manager/",
    "0",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
// Path is under fpga_manager but doesn't end with /flags — must now be rejected
#[case::bad_not_full_path_no_flags(
    "/sys/class/fpga_manager/fpga0/",
    "0",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_handle_in_path(
    "/sys/class/fpga_manager/no-dev/flags",
    "0",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
// --- valid device handles / paths ---
#[case::all_good_full_path(
    "/sys/class/fpga_manager/fpga0/flags",
    "0",
    ok(contains_substring("Flags set to '0x0' for 'fpga0'"))
)]
#[case::all_good("fpga0", "0", ok(contains_substring("Flags set to '0x0' for 'fpga0'")))]
// --- hex string format variants (all target device "fpga0") ---
// bare hex without prefix
#[case::hex_bare_zero("fpga0", "0", ok(contains_substring("Flags set to '0x0'")))]
#[case::hex_bare_uppercase("fpga0", "FF", ok(contains_substring("Flags set to '0xFF'")))]
#[case::hex_bare_lowercase("fpga0", "ff", ok(contains_substring("Flags set to '0xFF'")))]
#[case::hex_bare_mixed("fpga0", "20", ok(contains_substring("Flags set to '0x20'")))]
// with 0x prefix
#[case::hex_0x_zero("fpga0", "0x0", ok(contains_substring("Flags set to '0x0'")))]
#[case::hex_0x_uppercase_prefix("fpga0", "0xFF", ok(contains_substring("Flags set to '0xFF'")))]
#[case::hex_0x_lowercase_prefix("fpga0", "0xff", ok(contains_substring("Flags set to '0xFF'")))]
// with 0X prefix (capital X)
#[case::hex_0x_capital_x_prefix("fpga0", "0X20", ok(contains_substring("Flags set to '0x20'")))]
// max u32 value
#[case::max_u32_hex(
    "fpga0",
    "FFFFFFFF",
    ok(contains_substring("Flags set to '0xFFFFFFFF'"))
)]
#[case::max_u32_hex_prefixed(
    "fpga0",
    "0xFFFFFFFF",
    ok(contains_substring("Flags set to '0xFFFFFFFF'"))
)]
// with surrounding whitespace (should be trimmed)
#[case::hex_with_whitespace("fpga0", " 0x20 ", ok(contains_substring("Flags set to '0x20'")))]
// --- invalid flag values ---
// decimal-only string that overflows a hex u32
#[case::decimal_string_too_large(
    "fpga0",
    "4294967295",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
// 9 hex digits — overflows u32
#[case::hex_overflow(
    "fpga0",
    "100000000",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::invalid_chars(
    "fpga0",
    "xyz",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
async fn cases<M: for<'a> Matcher<&'a Result<String>>>(
    #[case] device_hande: &str,
    #[case] flags: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let res = proxy.xlnx_sys("write_flags", device_hande, flags).await;
    expect_that!(&res, condition)
}
