#!/usr/bin/env python3
"""Status command tests for xlnx platform."""

import unittest

from common.base_test import FPGATestBase
from common.helpers import is_dfx_mgr_available, get_test_data_path


@unittest.skipUnless(
    is_dfx_mgr_available(),
    "dfx-mgr component not installed. Install with: snap install fpgad+dfx-mgr.comp",
)
class TestStatusXlnx(FPGATestBase):
    """Test status command with --platform xlnx."""

    PLATFORM = "xlnx"

    def test_status_executes(self):
        """Test status command executes successfully."""
        proc = self.run_fpgad(["--platform", self.PLATFORM, "status"])
        self.assert_proc_succeeds(proc)

    def test_status_with_bitstream(self):
        """Test status shows operating state after loading bitstream."""
        test_data_path = get_test_data_path()
        bitstream_path = str(
            test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin"
        )
        load_proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "bitstream",
                bitstream_path,
            ]
        )
        self.assert_proc_succeeds(
            load_proc, "Failed to load a bitstream before checking status."
        )

        status_proc = self.run_fpgad(["--platform", self.PLATFORM, "status"])
        self.assert_proc_succeeds(status_proc)
        # dfx-mgr returns table with "slot->handle" and bitstream filename
        self.assert_in_proc_out("slot->handle", status_proc)
        self.assert_in_proc_out("k26_starter_kits.bit.bin", status_proc)

        # attempt to clean up
        self.run_fpgad(["--platform", self.PLATFORM, "remove", "bitstream"])

    def test_status_with_overlay(self):
        """Test status shows operating state after loading bitstream."""
        test_data_path = get_test_data_path()
        bitstream_path = str(
            test_data_path / "k26-starter-kits" / "k26_starter_kits.dtbo"
        )
        load_proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "overlay",
                bitstream_path,
            ]
        )
        self.assert_proc_succeeds(
            load_proc, "Failed to load a bitstream before checking status."
        )

        status_proc = self.run_fpgad(["--platform", self.PLATFORM, "status"])
        self.assert_proc_succeeds(status_proc)
        # dfx-mgr returns table with "slot->handle" and bitstream filename
        self.assert_in_proc_out("slot->handle", status_proc)
        self.assert_in_proc_out("k26_starter_kits.bit.bin", status_proc)
        # attempt to clean up
        self.run_fpgad(["--platform", self.PLATFORM, "remove", "overlay"])

    def test_status_failed_overlay(self):
        """Test status shows error after failed overlay load."""
        test_data_path = get_test_data_path()
        overlay_path = str(test_data_path / "fake_overlay.dtbo")
        load_proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "overlay",
                overlay_path,
            ]
        )
        self.assertNotEqual(
            load_proc.returncode,
            0,
            "Overlay load succeeded and therefore test has failed.",
        )

        proc = self.run_fpgad(["--platform", self.PLATFORM, "status"])
        self.assert_proc_succeeds(proc)
        # dfx-mgr returns empty table when nothing is loaded, not "error"
        # Just check it returns successfully
        self.assert_not_in_proc_out("0->0", proc)
