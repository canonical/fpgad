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

use std::{fs::OpenOptions, io::Read};

use std::io::Write as _;

pub fn fs_read(file_path: &str) -> Result<String, std::io::Error> {
    let mut buf: String = String::new();
    let result = OpenOptions::new()
        .read(true)
        .open(file_path)
        .and_then(|mut f| f.read_to_string(&mut buf));

    // do checks on the data we got if necessary
    match result {
        Ok(_) => Ok(buf),
        Err(e) => Err(e),
    }
}

pub fn fs_write(
    file_path: &str,
    create: bool,
    value: impl AsRef<str>,
) -> Result<(), std::io::Error> {
    OpenOptions::new()
        .create(create)
        .read(false)
        .write(true)
        .open(file_path)
        .and_then(|mut f| write!(f, "{}", value.as_ref()))
}
