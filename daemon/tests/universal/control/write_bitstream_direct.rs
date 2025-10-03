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
use crate::universal::{PLATFORM_STRING, setup};
use googletest::prelude::*;
use rstest::rstest;
use tokio;
use tokio::time::{Duration, timeout};
use zbus::Connection;

#[gtest]
#[tokio::test]
#[rstest]
#[case::bad_firmware_lookup(
    PLATFORM_STRING,
    "fpga0",
    "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin",
    "/firmware/xilinx/k26-starter-kits/",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_path(
    PLATFORM_STRING,
    "fpga0",
    "this_file_does_not_exist.bit.bin",
    "",
    err(displays_as(contains_substring("FpgadError::IOWrite:")))
)]
async fn should_fail_not_timeout<M: for<'a> Matcher<&'a zbus::Result<String>>>(
    #[case] platform_str: &str,
    #[case] device_hande: &str,
    #[case] bitstream_path_str: &str,
    #[case] firmware_lookup_path_str: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    /* The timeout is required because the xilinx driver returns an error code with
     *  absolute value larger than 4096, (less than -4096) breaking conventions and
     * causing Rust stdlib to panic because it tries to index to the error code:
     * `range start index 18446744073709545980 out of range for slice of length 24`
     */
    // todo: revert the timeout once the error codes from xilinx driver are fixed
    let timeout_result = timeout(
        Duration::from_secs(10),
        proxy.write_bitstream_direct(
            platform_str,
            device_hande,
            bitstream_path_str,
            firmware_lookup_path_str,
        ),
    )
    .await;

    assert_that!(timeout_result, ok(condition))
}

#[gtest]
#[tokio::test]
#[rstest]
#[case::path_with_firmware_lookup(
    PLATFORM_STRING,
    "fpga0",
    "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin",
    "/lib/firmware/xilinx/k26-starter-kits/",
    ok(contains_substring("loaded"))
)]
#[case::no_firmware_lookup(
    PLATFORM_STRING,
    "fpga0",
    "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin",
    "",
    ok(contains_substring("loaded"))
)]
async fn should_pass<M: for<'a> Matcher<&'a zbus::Result<String>>>(
    #[case] platform_str: &str,
    #[case] device_hande: &str,
    #[case] bitstream_path_str: &str,
    #[case] firmware_lookup_path_str: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");

    proxy
        .set_fpga_flags(platform_str, device_hande, 0)
        .await
        .expect("Failed to reset flags during test");

    // todo: revert the timeout once the error codes from xilinx driver are fixed
    let timeout_result = timeout(
        Duration::from_secs(10),
        proxy.write_bitstream_direct(
            platform_str,
            device_hande,
            bitstream_path_str,
            firmware_lookup_path_str,
        ),
    )
    .await;
    assert_that!(timeout_result, ok(condition))
}

#[gtest]
#[tokio::test]
#[rstest]
#[case::bad_bitstream_data(
    PLATFORM_STRING,
    "fpga0",
    "/lib/firmware/xilinx/k24-starter-kits/k24_starter_kits.bit.bin",
    "/lib/firmware/xilinx/k24-starter-kits/"
)]
async fn should_timeout(
    #[case] platform_str: &str,
    #[case] device_hande: &str,
    #[case] bitstream_path_str: &str,
    #[case] firmware_lookup_path_str: &str,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");

    proxy
        .set_fpga_flags(platform_str, device_hande, 0)
        .await
        .expect("Failed to reset flags during test");

    /* The timeout is required because the xilinx driver returns an error code with
     *  absolute value larger than 4096, (less than -4096) breaking conventions and
     * causing Rust stdlib to panic because it tries to index to the error code:
     * `range start index 18446744073709545980 out of range for slice of length 24`
     */
    // todo: revert the timeout once the error codes from xilinx driver are fixed
    let timeout_result = timeout(
        Duration::from_secs(10),
        proxy.write_bitstream_direct(
            platform_str,
            device_hande,
            bitstream_path_str,
            firmware_lookup_path_str,
        ),
    )
    .await;
    if let Ok(res) = timeout_result {
        panic!("Timeout not reached when expecting a timeout to occur: {res:?}");
    }
}
