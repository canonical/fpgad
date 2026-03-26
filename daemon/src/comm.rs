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

//! Communication interfaces for the fpgad daemon.
//!
//! This module provides communication interfaces that allow external clients to
//! interact with the fpgad daemon. Currently, the only communication method is
//! DBus, which provides a system bus service for privileged FPGA operations.
//!
//! # Submodules
//!
//! - [`dbus`] - DBus interface implementation for the daemon
//!
//! # Architecture
//!
//! The daemon exposes two DBus interfaces:
//! - **Control Interface** - Write operations (loading, applying, removing)
//! - **Status Interface** - Read-only operations (querying state, listing devices)
//!
//! For more information, see the [`dbus`] module documentation.

pub mod dbus;
