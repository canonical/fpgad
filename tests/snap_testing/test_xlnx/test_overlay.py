#!/usr/bin/env python3
"""Overlay loading tests for xlnx platform."""

import unittest
from pathlib import Path

from common.base_test import FPGATestBase
from common.helpers import (
    BAD_FLAGS,
    set_flags,
    is_dfx_mgr_available,
    is_snap_environment,
    TestData,
    copy_test_data_files,
    cleanup_test_data_files,
    get_test_data_path,
)


@unittest.skipUnless(
    is_dfx_mgr_available(),
    "dfx-mgr component not installed. Install with: snap install fpgad+dfx-mgr.comp",
)
class TestOverlayXlnx(FPGATestBase):
    """Test overlay loading operations with --platform xlnx."""

    PLATFORM = "xlnx"

    def test_load_overlay_local(self):
        """Test loading overlay from local path."""
        test_data_path = get_test_data_path()
        overlay_path = test_data_path / "k26-starter-kits" / "k26_starter_kits.dtbo"
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "overlay", str(overlay_path)]
        )

        self.assert_proc_succeeds(proc)
        self.assert_not_in_proc_out("loaded via ", proc)
        self.assert_in_proc_out("Loaded with slot_handle ", proc)

    @unittest.skipIf(
        is_snap_environment(),
        "Test requires file copy to /lib/firmware, which is not available in confined snap environment",
    )
    def test_load_overlay_lib_firmware(self):
        """Test loading overlay from /lib/firmware."""
        test_data_path = get_test_data_path()
        # Necessary due to bad dtbo content from upstream
        test_file_paths = [
            TestData(
                source=test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin",
                target=Path("/lib/firmware/k26-starter-kits.bit.bin"),
            ),
            TestData(
                source=test_data_path / "k26-starter-kits" / "k26_starter_kits.dtbo",
                target=Path("/lib/firmware/k26-starter-kits.dtbo"),
            ),
        ]
        for file in test_file_paths:
            try:
                copy_test_data_files(file)
            except Exception as e:
                print(f"Failed to copy {file.source} to {file.target}")
                raise e

        overlay_path = Path("/lib/firmware/k26-starter-kits.dtbo")
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "overlay", str(overlay_path)]
        )
        for file in test_file_paths:
            try:
                cleanup_test_data_files(file)
            except Exception as e:
                print(f"Failed to clean up {file.target}")
                raise e

        self.assert_proc_succeeds(proc)
        self.assert_not_in_proc_out("loaded via ", proc)
        self.assert_in_proc_out("Loaded with slot_handle ", proc)

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
        self.assert_not_in_proc_out("loaded via ", proc)
        self.assert_in_proc_out("Loaded with slot_handle ", proc)

    def test_load_overlay_bad_path(self):
        """Test loading overlay from non-existent path fails."""
        overlay_path = Path("/path/does/not/exist")
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "overlay", str(overlay_path)]
        )
        self.assert_proc_fails(proc)
        self.assert_not_in_proc_out("loaded via", proc)
        self.assert_not_in_proc_out("Loaded with slot_handle ", proc)
        self.assert_in_proc_err("FpgadError::Softener:", proc)

    def test_load_overlay_bad_flags(self):
        """Test loading overlay with invalid flags should pass because dfx-mgr resets the flags"""
        set_flags(BAD_FLAGS)
        test_data_path = get_test_data_path()
        overlay_path = test_data_path / "k26-starter-kits" / "k26_starter_kits.dtbo"
        proc = self.run_fpgad(
            ["--platform", self.PLATFORM, "load", "overlay", str(overlay_path)]
        )
        self.assert_proc_succeeds(proc)
        self.assert_not_in_proc_out("loaded via ", proc)
        self.assert_in_proc_out("Loaded with slot_handle ", proc)
