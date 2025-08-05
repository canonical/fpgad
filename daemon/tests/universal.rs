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

pub mod universal {
    pub mod control;
    pub static PLATFORM_STRING: &str = "universal";
    pub mod sequences;
    pub mod status;

    use rstest::*;
    use std::path::Path;

    struct TestData {
        source: &'static str,
        targets: &'static [&'static str],
    }

    fn copy_file(source: &str, target: &str) {
        let exit_status = std::process::Command::new("sudo")
            .arg("cp")
            .arg(source)
            .arg(target)
            .status()
            .unwrap_or_else(|_| panic!("copy_file: failed to execute `sudo cp {source} {target}`"));
        if exit_status.success() {
            println!("copy_file: {source} copied to {target}");
        } else {
            eprintln!(
                "copy_file: failed to copy {} to {}, exit_status: {:?}",
                source,
                target,
                exit_status.code()
            );
        }
    }

    fn copy_test_data_files(test_file: &TestData) {
        for target in test_file.targets.iter() {
            // only copy if not already present
            let f = Path::new(*target);
            if f.exists() {
                println!(
                    "copy_test_data_files: skipping creation: {} exists, ",
                    f.display()
                );
                continue;
            }
            let parent = f.parent().unwrap();
            if !parent.exists() {
                println!(
                    "copy_test_data_files: creating directory {}",
                    parent.display()
                );
                std::fs::create_dir_all(parent).unwrap_or_else(|_| {
                    panic!("copy_test_data_files: Failed to create target directory: {parent:?}")
                });
            }
            copy_file(test_file.source, target);
        }
    }

    fn setup_xilinx_files() {
        let test_files = [
            TestData {
                source: "./daemon/tests/test_data/k26-starter-kits/k26_starter_kits.bit.bin",
                targets: &[
                    "/lib/firmware/k26-starter-kits.bit.bin",
                    "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin",
                ],
            },
            TestData {
                source: "./daemon/tests/test_data/k24-starter-kits/k24_starter_kits.bit.bin",
                targets: &[
                    "/lib/firmware/k24-starter-kits.bit.bin",
                    "/lib/firmware/xilinx/k24-starter-kits/k24_starter_kits.bit.bin",
                ],
            },
            TestData {
                source: "./daemon/tests/test_data/k26-starter-kits/k26_starter_kits.dtbo",
                targets: &[
                    "/lib/firmware/k26-starter-kits.dtbo",
                    "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.dtbo",
                ],
            },
            TestData {
                source: "./daemon/tests/test_data/k24-starter-kits/k24_starter_kits.dtbo",
                targets: &[
                    "/lib/firmware/k24-starter-kits.dtbo",
                    "/lib/firmware/xilinx/k24-starter-kits/k24_starter_kits.dtbo",
                ],
            },
        ];

        for test_file in test_files.iter() {
            copy_test_data_files(test_file);
        }
    }

    /// Use rstest fixtures to copy the necessary files from test_data to where the tests expect them
    ///
    /// At the time of writing, rstest does not officially support "session scoped" tests like
    /// pytest does, but I found this `#[once]` macro in the rstest internal tests and associated
    /// feature request. This may be unstable.
    #[fixture]
    #[once]
    pub fn setup() {
        setup_xilinx_files();
    }
}
pub mod common;
