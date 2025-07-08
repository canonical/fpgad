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

pub static FIRMWARE_SOURCE_DIR: &str = "/lib/firmware/";
pub static FPGA_MANAGERS_DIR: &str = "/sys/class/fpga_manager/";
pub static OVERLAY_CONTROL_DIR: &str = "/sys/kernel/config/device-tree/overlays/";
pub static FIRMWARE_LOC_CONTROL_PATH: &str = "/sys/module/firmware_class/parameters/path";
