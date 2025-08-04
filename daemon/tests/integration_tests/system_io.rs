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
use fpgad::system_io::{fs_create_dir, fs_read, fs_read_dir, fs_remove_dir, fs_write};
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

#[gtest]
#[rstest]
#[case::not_found(
    "bad_input",
    "write_test",
    err(displays_as(contains_substring("No such file or directory")))
)]
#[case::is_dir(
    "/etc/",
    "write_test",
    err(displays_as(contains_substring("Is a directory")))
)]
fn test_fs_write_no_create<M: for<'a> Matcher<&'a std::result::Result<(), FpgadError>>>(
    #[case] path_str: &str,
    #[case] data: &str,
    #[case] condition: M,
) {
    let r = fs_write(Path::new(path_str), false, data);
    expect_that!(r, condition);
}

#[gtest]
#[rstest]
fn file_write_readback_delete() {
    let file = Path::new("./daemon/tests/test_data/test_file.txt");
    let data = "this is test data\n";
    let create = true;
    let r = fs_write(file, create, data);
    assert_that!(r, ok(anything()));
    assert_that!(file.exists(), eq(true));
    let r = fs_read(file);
    expect_that!(r, ok(eq(data)));
}

#[gtest]
#[rstest]
fn dir_create_and_delete_empty() {
    let dir = Path::new("./daemon/tests/test_data/test_dir");
    let r = fs_create_dir(dir);
    assert_that!(r, ok(anything()));
    let r = fs_remove_dir(dir);
    assert_that!(r, ok(anything()));
    assert_that!(dir.exists(), eq(false));
}

#[gtest]
#[rstest]
fn dir_create_and_delete_not_empty() {
    let dir = Path::new("./daemon/tests/test_data/test_dir");
    let r = fs_create_dir(dir);
    assert_that!(r, ok(anything()));

    let r = fs_write(
        Path::new("./daemon/tests/test_data/test_dir/test_file.txt"),
        true,
        "test_data\n",
    );
    expect_that!(r, ok(anything()));
    let r = fs_remove_dir(dir);
    assert_that!(
        r,
        err(displays_as(contains_substring("Directory not empty")))
    );
    assert_that!(dir.exists(), eq(true));

    let _ = std::fs::remove_dir_all(dir);
    assert_that!(dir.exists(), eq(false));
}

#[gtest]
#[rstest]
fn dir_create_dir_on_filename() {
    let dir = Path::new("./daemon/tests/test_data/com.canonical.fpgad.conf");
    let r = fs_create_dir(dir);
    assert_that!(r, err(displays_as(contains_substring("File exists"))));
}

#[gtest]
#[rstest]
fn test_read_dir_ok() {
    let r = fs_read_dir(Path::new("./daemon/tests/test_data/k26-starter-kits/"));
    assert_that!(
        r,
        ok(contains_each![
            "k26_starter_kits.bit.bin",
            "k26_starter_kits.dtbo",
            "LICENSE",
            "shell.json"
        ])
    );
}

#[gtest]
#[rstest]
fn test_read_dir_fail() {
    let r = fs_read_dir(Path::new("./daemon/tests/test_data/this/dir/doesnt/exist"));
    assert_that!(
        r,
        err(displays_as(contains_substring("No such file or directory")))
    );
}
