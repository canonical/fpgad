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

/// The driver-decided location of fpga_manager objects. Typically `/sys/class/fpga_manager/`.
pub static FPGA_MANAGERS_DIR: &str = "/sys/class/fpga_manager/";

/// The default location of the device-tree overlay directory within the config filesystem. Typically `/sys/kernel/config/device-tree/overlays/`, but often remounted to `/config/device-tree/overlays/`.
pub static OVERLAY_CONTROL_DIR: &str = "/sys/kernel/config/device-tree/overlays/";

/// The path of the file which can be written to in order to add an additional firmware search path
/// to the system. Typically `/sys/module/firmware_class/parameters/path`.
/// See
/// [https://docs.kernel.org/driver-api/firmware/fw_search_path.html](https://docs.kernel.org/driver-api/firmware/fw_search_path.html)
/// for more information
pub static FIRMWARE_LOC_CONTROL_PATH: &str = "/sys/module/firmware_class/parameters/path";
