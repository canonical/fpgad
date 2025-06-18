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

use crate::error::FpgadError;
use log::trace;
use std::fs::{create_dir_all, remove_dir};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::{fs::OpenOptions, io::Read};

pub fn fs_read(file_path: &Path) -> Result<String, FpgadError> {
    trace!("Attempting to read from {:?}", file_path);
    let mut buf: String = String::new();
    let result = OpenOptions::new()
        .read(true)
        .open(file_path)
        .and_then(|mut f| f.read_to_string(&mut buf));

    // do checks on the data we got if necessary
    match result {
        Ok(_) => Ok(buf),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(FpgadError::IOError(format!(
                "File {:?} not found: {}",
                file_path, e
            ))),
            ErrorKind::PermissionDenied => Err(FpgadError::IOError(format!(
                "Read permission denied for file {:?}: {}",
                file_path, e
            ))),
            _ => Err(FpgadError::IOError(format!("{}", e))),
        },
    }
}

pub fn fs_write(file_path: &Path, create: bool, value: impl AsRef<str>) -> Result<(), FpgadError> {
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
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(FpgadError::IOError(format!(
                "File {:?} not found: {}",
                file_path, e
            ))),
            ErrorKind::PermissionDenied => Err(FpgadError::IOError(format!(
                "Read permission denied for file {:?}: {}",
                file_path, e
            ))),
            _ => Err(FpgadError::IOError(format!("{}", e))),
        },
    }
}

pub fn fs_create_dir(path: &Path) -> Result<(), FpgadError> {
    trace!("Attempting to Create '{:?}'", path);
    let result = create_dir_all(path);
    match result {
        Ok(_) => {
            trace!("Directory created at {:?}.", path);
            Ok(())
        }
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(FpgadError::IOError(format!(
                "Read permission denied when creating directory {:?}: {}",
                path, e
            ))),
            ErrorKind::NotFound => Err(FpgadError::IOError(format!(
                "Attempted to create a directory but the base path could not be found {:?}: {}",
                path, e
            ))),
            _ => Err(FpgadError::IOError(format!("{}", e))),
        },
    }
}

pub fn fs_remove_dir(path: &Path) -> Result<(), FpgadError> {
    trace!("Attempting to delete '{:?}'", path);
    let result = remove_dir(path);
    match result {
        Ok(_) => {
            trace!("Deleted {:?}", path);
            Ok(())
        }
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(FpgadError::IOError(format!(
                "Read permission denied when deleting directory {:?}: {}",
                path, e
            ))),
            ErrorKind::DirectoryNotEmpty => Err(FpgadError::IOError(format!(
                "Attempted to delete a directory it is not empty {:?}: {}",
                path, e
            ))),
            ErrorKind::NotFound => Err(FpgadError::IOError(format!(
                "Attempted to delete a directory that does not exist {:?}: {}",
                path, e
            ))),
            _ => Err(FpgadError::IOError(format!("{}", e))),
        },
    }
}
