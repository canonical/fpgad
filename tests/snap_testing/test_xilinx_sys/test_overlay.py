#!/usr/bin/env python3
"""Overlay loading tests for xlnx-sys platform."""

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


class TestOverlayXlnxSys(FPGATestBase):
    """Test overlay loading operations with --platform xlnx-sys."""

    PLATFORM = "xlnx-sys"

    @unittest.skipIf(
        is_snap_environment(),
        "Test requires file copy, which is not available in confined snap environment",
    )
    def test_load_overlay_local(self):
        """Test loading overlay from local path."""
        test_data_path = get_test_data_path()
        # Necessary due to bad dtbo content from upstream
        test_file_paths = TestData(
            source=test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin",
            target=test_data_path / "k26-starter-kits" / "k26-starter-kits.bit.bin",
        )

        try:
            copy_test_data_files(test_file_paths)
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e
        overlay_path = test_data_path / "k26-starter-kits" / "k26_starter_kits.dtbo"
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "overlay", str(overlay_path)]
        )
        try:
            cleanup_test_data_files(test_file_paths)
        except Exception as e:
            print(f"Failed to clean up {test_file_paths.target}")
            raise e

        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded to", proc)

    def test_load_overlay_full_path(self):
        """Test loading overlay with full absolute path."""
        test_data_path = get_test_data_path()
        overlay_path = test_data_path / "k26-starter-kits" / "k26_starter_kits.dtbo"
        proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "overlay",
                str(overlay_path.resolve()),
            ]
        )
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded to", proc)

    def test_load_overlay_bad_path(self):
        """Test loading overlay from non-existent path fails."""
        overlay_path = Path("/path/does/not/exist")
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "overlay", str(overlay_path)]
        )
        self.assert_proc_fails(proc)
        self.assert_not_in_proc_out("loaded to", proc)
        self.assert_in_proc_err("FpgadError::OverlayStatus:", proc)

    def test_load_overlay_bad_flags(self):
        """Test loading overlay with invalid flags fails."""
        set_flags(BAD_FLAGS)
        test_data_path = get_test_data_path()
        overlay_path = test_data_path / "k26-starter-kits" / "k26_starter_kits.dtbo"
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "overlay", str(overlay_path)]
        )
        self.assert_proc_fails(proc)
        self.assert_not_in_proc_out("loaded to", proc)
        self.assert_in_proc_err("FpgadError::OverlayStatus:", proc)
