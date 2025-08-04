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

use fpgad::error::FpgadError;
use fpgad::system_io::fs_read;
use googletest::prelude::*;
use rstest::*;
use std::path::Path;

#[gtest]
#[rstest]
#[case::not_found(
    "bad_input",
    err(displays_as(contains_substring("No such file or directory")))
)]
#[case::is_dir("/etc/", err(displays_as(contains_substring("Is a directory"))))]
#[case::ok("./daemon/tests/test_data/com.canonical.fpgad.conf", ok(anything()))]
fn test_fs_read<M: for<'a> Matcher<&'a std::result::Result<String, FpgadError>>>(
    #[case] path_str: &str,
    #[case] condition: M,
) {
    let r = fs_read(Path::new(path_str));
    expect_that!(r, condition);
}
