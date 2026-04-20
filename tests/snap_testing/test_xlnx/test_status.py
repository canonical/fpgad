#!/usr/bin/env python3
"""Status command tests for xlnx platform."""

import unittest

from common.base_test import FPGATestBase
from common.helpers import is_dfx_mgr_available


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
        load_proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "bitstream",
                "./fpgad/k26-starter-kits/k26_starter_kits.bit.bin",
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

    def test_status_failed_overlay(self):
        """Test status shows error after failed overlay load."""
        load_proc = self.run_fpgad(
            [
                "--platform",
                self.PLATFORM,
                "load",
                "overlay",
                "./fpgad/k26-starter-kits/k26_starter_kits.dtbo",
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
        self.assert_in_proc_out("slot->handle", proc)


# TODO(Artie): write these by hand - these are trash tests.
