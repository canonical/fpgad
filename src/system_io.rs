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

use log::trace;
use std::io::Write as _;
use std::path::{Path};

pub fn fs_read(file_path: &Path) -> Result<String, std::io::Error> {
    trace!("Attempting to read from {:?}", file_path);
    let mut buf: String = String::new();
    let result = OpenOptions::new()
        .read(true)
        .open(file_path)
        .and_then(|mut f| f.read_to_string(&mut buf));

    // do checks on the data we got if necessary
    match result {
        Ok(_) => {
            trace!("Read done.");
            Ok(buf)
        }
        Err(e) => Err(e), // TODO: improve error with attempted write path etc
        //TODO: if error is permission error then this should print a hint "are you root/fpga user group"
    }
}

pub fn fs_write(
    file_path: &Path,
    create: bool,
    value: impl AsRef<str>,
) -> Result<(), std::io::Error> {
    trace!(
        "Attempting to write {:?} to {:?}",
        value.as_ref(),
        file_path
    );
    let result = OpenOptions::new()
        .create(create)
        .read(false)
        .write(true)
        .open(file_path)
        .and_then(|mut f| write!(f, "{}", value.as_ref()));
    match result {
        Ok(_) => {
            trace!("Write done.");
            Ok(())
        }
        Err(e) => Err(e), 
        // TODO: improve error with attempted write path etc
        // TODO: if error is permission error then this should print a hint "are you root/fpga user group"
    }
}
