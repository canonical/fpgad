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

use log::error;
use std::path::PathBuf;
use zbus::fdo;

#[derive(Debug, thiserror::Error)]
pub enum FpgadError {
    #[error("FpgadError::Flag: Failed to read flags: {0}")]
    Flag(String),
    #[error("FpgadError::OverlayStatus: Overlay was not applied: {0}")]
    OverlayStatus(String),
    #[error("FpgadError::FPGAState: FPGA state is not as expected: {0}")]
    FPGAState(String),
    #[error("FpgadError::Argument: {0}")]
    Argument(String),
    #[error("FpgadError::IORead: An IO error occurred when reading from {file:?}: {e}")]
    IORead { file: PathBuf, e: std::io::Error },
    #[error("FpgadError::IOWrite: An IO error occurred when writing {data:?} to {file:?}: {e}")]
    IOWrite {
        data: String,
        file: PathBuf,
        e: std::io::Error,
    },
    #[error("FpgadError::IOCreate: An IO error occurred when creating {file:?}: {e}")]
    IOCreate { file: PathBuf, e: std::io::Error },
    #[error("FpgadError::IODelete: An IO error occurred when deleting {file:?}: {e}")]
    IODelete { file: PathBuf, e: std::io::Error },
    #[error("FpgadError::IOReadDir: An IO error occurred when reading directory {dir:?}: {e}")]
    IOReadDir { dir: PathBuf, e: std::io::Error },
    #[cfg(feature = "softeners")]
    #[error("FpgadError::Softener: An error occurred using softener: {0}")]
    Softener(crate::softeners::error::FpgadSoftenerError),
    #[error("FpgadError::Internal: An Internal error occurred: {0}")]
    Internal(String),
}

impl From<FpgadError> for fdo::Error {
    fn from(err: FpgadError) -> Self {
        error!("{err}");
        match err {
            FpgadError::Argument(..) => fdo::Error::InvalidArgs(err.to_string()),
            FpgadError::IORead { .. } => fdo::Error::IOError(err.to_string()),
            FpgadError::IOWrite { .. } => fdo::Error::IOError(err.to_string()),
            FpgadError::IOCreate { .. } => fdo::Error::IOError(err.to_string()),
            FpgadError::IODelete { .. } => fdo::Error::IOError(err.to_string()),
            FpgadError::IOReadDir { .. } => fdo::Error::IOError(err.to_string()),
            _ => fdo::Error::Failed(err.to_string()),
        }
    }
}
