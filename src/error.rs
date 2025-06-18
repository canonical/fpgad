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

#[derive(Debug, thiserror::Error)]
pub enum FpgadError {
    #[error("Failed to read flags: {0}")]
    FlagError(String),
    #[error("Overlay was not applied: {0}")]
    OverlayStatusError(String),
    #[error("FPGA state is not as expected: {0}")]
    FPGAStateError(String),
    #[error("ArgumentError: {0}")]
    ArgumentError(String),
    #[error("An IO error occurred: {0}")]
    IOError(String),
    #[error("An Internal error occurred: {0}")]
    InternalError(String),
}
