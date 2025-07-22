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

use crate::common::proxies::{control_proxy, status_proxy};
use googletest::prelude::*;
use rstest::*;
use zbus::Connection;

#[gtest]
#[tokio::test]
#[rstest]
#[case::load_from_lib_firmware("universal", "fpga0", "/lib/firmware/k26-starter-kits.bit.bin", "")]
async fn cases(
    #[case] platform_handle: &str,
    #[case] device_handle: &str,
    #[case] bitstream_file: &str,
    #[case] fw_lookup: &str,
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let status_proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");

    control_proxy
        .set_fpga_flags(platform_handle, device_handle, 0)
        .await
        .expect("failed to set fpga flags");
    assert_eq!(
        status_proxy
            .get_fpga_flags(platform_handle, device_handle)
            .await
            .expect("failed to get fpga flags"),
        "0",
        "flags not 0 after setting to 0"
    );

    control_proxy
        .write_property("/sys/class/fpga_manager/fpga0/key", "")
        .await
        .expect("failed to reset the encryption key");
    control_proxy
        .write_bitstream_direct(platform_handle, device_handle, bitstream_file, fw_lookup)
        .await
        .expect("failed to write bitstream file");
    let s = status_proxy
        .get_fpga_state(platform_handle, device_handle)
        .await
        .expect("failed to get fpga state");
    assert_eq!(s, "operating", "fpga state not 'operating'");
}
