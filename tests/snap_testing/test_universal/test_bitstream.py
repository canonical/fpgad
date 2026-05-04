#!/usr/bin/env python3
"""Bitstream loading tests for universal platform."""

import unittest
from pathlib import Path
from common.base_test import FPGATestBase
from common.helpers import (
    BAD_FLAGS,
    set_flags,
    is_snap_environment,
    TestData,
    copy_test_data_files,
    cleanup_test_data_files,
    get_test_data_path,
)


class TestBitstreamUniversal(FPGATestBase):
    """Test bitstream loading operations with --platform universal."""

    PLATFORM = "universal"

    def test_load_bitstream_local(self):
        """Test loading bitstream from local relative path."""
        test_data_path = get_test_data_path()
        path_str = str(test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin")
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "bitstream", path_str]
        )
        self.assert_proc_succeeds(proc)
        self.assertIn("loaded to 'fpga0' using firmware lookup path", proc.stdout)

    def test_load_bitstream_home_fullpath(self):
        """Test loading bitstream from full absolute path."""
        test_data_path = get_test_data_path()
        path = test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin"
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "bitstream", str(path.resolve())]
        )
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded to 'fpga0' using firmware lookup path", proc)

    @unittest.skipIf(
        is_snap_environment(),
        "Test requires file copy to /lib/firmware, which is not available in confined snap environment",
    )
    def test_load_bitstream_lib_firmware(self):
        """Test loading bitstream from /lib/firmware."""
        test_data_path = get_test_data_path()
        test_file_paths = TestData(
            source=test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin",
            target=Path("/lib/firmware/k26-starter-kits.bit.bin"),
        )
        try:
            copy_test_data_files(test_file_paths)
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e

        proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "bitstream",
                str(test_file_paths.target),
            ]
        )

        try:
            cleanup_test_data_files(test_file_paths)
        except Exception as e:
            print(f"Failed to clean up {test_file_paths.target}")
            raise e

        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded to 'fpga0' using firmware lookup path", proc)

    @unittest.skipIf(
        is_snap_environment(),
        "Test requires file copy to /lib/firmware, which is not available in confined snap environment",
    )
    def test_load_bitstream_lib_firmware_xilinx(self):
        """Test loading bitstream from /lib/firmware/xilinx subdirectory."""
        test_data_path = get_test_data_path()
        test_file_paths = TestData(
            source=test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin",
            target=Path(
                "/lib/firmware/xilinx/k26_starter_kits/k26-starter-kits.bit.bin"
            ),
        )
        try:
            copy_test_data_files(test_file_paths)
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e

        proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "bitstream",
                str(test_file_paths.target),
            ]
        )

        try:
            cleanup_test_data_files(test_file_paths)
        except Exception as e:
            print(f"Failed to clean up {test_file_paths.target}")
            raise e

        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded to 'fpga0' using firmware lookup path", proc)

    def test_load_bitstream_path_not_exist(self):
        """Test loading bitstream from non-existent path fails."""
        proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "bitstream",
                "/this/path/is/fake.bit.bin",
            ]
        )
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("FpgadError::IOWrite:", proc)

    def test_load_bitstream_containing_dir(self):
        """Test loading bitstream with directory path fails."""
        test_data_path = get_test_data_path()
        path = test_data_path / "k26-starter-kits"
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "bitstream", str(path)]
        )
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("FpgadError::IOWrite:", proc)

    def test_load_bitstream_bad_flags(self):
        """Test loading bitstream with invalid flags fails."""
        test_data_path = get_test_data_path()
        path = test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin"
        set_flags(BAD_FLAGS)
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "bitstream", str(path)]
        )
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("FpgadError::IOWrite:", proc)
