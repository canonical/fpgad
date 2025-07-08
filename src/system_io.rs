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

use crate::config;
use crate::error::FpgadError;
use log::trace;
use std::fs::OpenOptions;
use std::fs::{create_dir_all, remove_dir};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Convenient wrapper for reading the contents of `file_path` to String
pub fn fs_read(file_path: &Path) -> Result<String, FpgadError> {
    trace!("Attempting to read from {file_path:?}");
    let mut buf: String = String::new();
    let result = OpenOptions::new()
        .read(true)
        .open(file_path)
        .and_then(|mut f| f.read_to_string(&mut buf));

    match result {
        Ok(_) => {
            trace!("Reading done");
            Ok(buf)
        }
        Err(e) => Err(FpgadError::IORead {
            file: file_path.into(),
            e,
        }),
    }
}

/// Convenient wrapper for writing `value` to `file_path`
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
        Err(e) => Err(FpgadError::IOWrite {
            data: value.as_ref().to_string(),
            file: file_path.into(),
            e,
        }),
    }
}

/// Convenient wrapper for recursively creating directories up to `path`
pub fn fs_create_dir(path: &Path) -> Result<(), FpgadError> {
    trace!("Attempting to Create '{path:?}'");
    let result = create_dir_all(path);
    match result {
        Ok(_) => {
            trace!("Directory created at {path:?}.");
            Ok(())
        }
        Err(e) => Err(FpgadError::IOCreate {
            file: path.into(),
            e,
        }),
    }
}

/// Convenient wrapper for deleting an "empty" directory - works for overlayfs
pub fn fs_remove_dir(path: &Path) -> Result<(), FpgadError> {
    trace!("Attempting to delete '{path:?}'");
    let result = remove_dir(path);
    match result {
        Ok(_) => {
            trace!("Deleted {path:?}");
            Ok(())
        }
        Err(e) => Err(FpgadError::IODelete {
            file: path.into(),
            e,
        }),
    }
}

pub fn extract_path_and_filename(path: &Path) -> Result<(String, String), FpgadError> {
    // Extract filename
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(FpgadError::Argument(format!(
            "Provided bitstream path {path:?} is missing filename."
        )))?;

    // Extract parent directory
    let base_path = path
        .parent()
        .and_then(|p| p.to_str())
        .ok_or(FpgadError::Argument(format!(
            "Provided bitstream path {path:?} is missing a parent dir."
        )))?;

    Ok((base_path.to_string(), filename.to_string()))
}

/// Helper function to check that a device with given handle does exist.
pub(crate) fn validate_device_handle(device_handle: &str) -> Result<(), FpgadError> {
    if device_handle.is_empty() || !device_handle.is_ascii() {
        return Err(FpgadError::Argument(format!(
            "{device_handle} is invalid name for fpga device.\
                fpga name must be compliant with sysfs rules."
        )));
    }
    let fpga_managers_dir = config::FPGA_MANAGERS_DIR;
    if !PathBuf::from(fpga_managers_dir)
        .join(device_handle)
        .exists()
    {
        return Err(FpgadError::Argument(format!(
            "Device {device_handle} not found."
        )));
    };
    Ok(())
}

#[allow(dead_code)]
pub fn write_firmware_source_dir(new_path: &str) -> Result<(), FpgadError> {
    trace!(
        "Writing fw prefix {} to {}",
        new_path,
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_write(fw_lookup_override, false, new_path)
}

#[allow(dead_code)]
pub fn read_firmware_source_dir() -> Result<String, FpgadError> {
    trace!(
        "Reading fw prefix from {}",
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_read(fw_lookup_override)
}

#[allow(dead_code)]
/// Convenient wrapper for reading contents of a directory
pub fn fs_read_dir(dir: &Path) -> Result<Vec<String>, FpgadError> {
    trace!("Attempting to read directory '{dir:?}'");
    std::fs::read_dir(dir).map_or_else(
        |e| {
            Err(FpgadError::IOReadDir {
                dir: dir.to_owned(),
                e,
            })
        },
        |iter| {
            let ret = iter
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect();
            trace!("Dir reading done.");
            Ok(ret)
        },
    )
}
