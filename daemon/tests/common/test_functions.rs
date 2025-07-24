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

use googletest::prelude::*;
use zbus::Result;

pub fn compare_result(res: &Result<String>, exp: &Result<String>) {
    match (res, exp) {
        (Ok(res_s), Ok(exp_s)) => {
            assert_that!(
                res_s.to_string(),
                contains_substring(exp_s.clone()),
                "Mismatched error signature"
            );
        }
        (Err(res_err), Err(exp_err)) => {
            assert_that!(
                res_err.to_string(),
                contains_substring(exp_err.to_string()),
                "Mismatched error signature"
            );
        }
        (res, exp) => {
            panic!("Result mismatch: got {res:?}, expected {exp:?}");
        }
    }
}
