// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
// Copyright 2025 Canonical Ltd.
// SPDX-License-Identifier: GPL-3.0-only
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

/* synchronous functions:
   load, unload overlay, check status output before and after
   set and check flags
   set and check "keys" using write_property
   load bitstream and check fpga state
   set key and read it back then reset it
*/

#[cfg(test)]
mod test_sequences {
    use crate::proxies::{control_proxy, status_proxy};
    use std::path::Path;
    use zbus::Connection;

    static DEVICE_HANDLE: &str = "fpga0";
    static OVERLAY_HANDLE: &str = "fpga0";
    static PLATFORM_HANDLE: &str = "universal";
    static OVERLAY_FILE: &str = "/lib/firmware/k26-starter-kits.dtbo";
    static BITSTREAM_FILE: &str = "/lib/firmware/k26-starter-kits.bit.bin";

    #[tokio::test]
    async fn test_universal_overlay_management() {
        let connection = Connection::system()
            .await
            .expect("failed to create connection");
        let control_proxy = control_proxy::ControlProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        let status_proxy = status_proxy::StatusProxy::new(&connection)
            .await
            .expect("failed to create status proxy");
        for overlay_handle in status_proxy
            .get_overlays()
            .await
            .expect("failed to get overlays")
            .split(",")
        {
            if !overlay_handle.is_empty() {
                control_proxy
                    .remove_overlay(PLATFORM_HANDLE, overlay_handle)
                    .await
                    .unwrap_or_else(|_| panic!("failed to remove {overlay_handle}"));
            }
        }
        control_proxy
            .set_fpga_flags(PLATFORM_HANDLE, DEVICE_HANDLE, 0)
            .await
            .expect("failed to set fpga flags");

        assert_eq!(
            status_proxy
                .get_fpga_flags(PLATFORM_HANDLE, DEVICE_HANDLE)
                .await
                .expect("failed to get fpga flags"),
            "0",
            "flags not set properly"
        );
        control_proxy
            .apply_overlay(PLATFORM_HANDLE, OVERLAY_HANDLE, OVERLAY_FILE, "")
            .await
            .expect("failed to apply overlay");
        let s = status_proxy
            .get_overlay_status(PLATFORM_HANDLE, OVERLAY_HANDLE)
            .await
            .expect("failed to read overlay status");
        let file_name = Path::new(OVERLAY_FILE)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert!(
            s.contains(&file_name),
            "relative path '{file_name}' not found in overlay status: '{s}'"
        );
        assert!(
            s.contains("applied"),
            "state 'applied' not found in overlay status"
        );
    }

    #[tokio::test]
    async fn test_universal_fpga_control() {
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
            .set_fpga_flags(PLATFORM_HANDLE, DEVICE_HANDLE, 0)
            .await
            .expect("failed to set fpga flags");

        assert_eq!(
            status_proxy
                .get_fpga_flags(PLATFORM_HANDLE, DEVICE_HANDLE)
                .await
                .expect("failed to get fpga flags"),
            "0",
            "flags not 0 after setting to 0"
        );
        control_proxy
            .write_bitstream_direct(PLATFORM_HANDLE, OVERLAY_HANDLE, BITSTREAM_FILE, "")
            .await
            .expect("failed to write bitstream file");
        let s = status_proxy
            .get_fpga_state(PLATFORM_HANDLE, OVERLAY_HANDLE)
            .await
            .expect("failed to get fpga state");
        assert_eq!(s, "operating", "fpga state not 'operating'");
    }
}
