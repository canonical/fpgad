#!/usr/bin/env python3
"""CLI option tests (--device, --platform, --name)."""

import unittest
from common.base_test import FPGATestBase
from common.helpers import is_dfx_mgr_available


class TestCLIOptions(FPGATestBase):
    """Test CLI options like --device, --platform, and --name."""

    def test_status_with_platform_option_universal(self):
        """Test status command with explicit --platform option set to `universal`."""
        proc = self.run_fpgad(["--platform", "universal", "status"])
        self.assert_in_proc_out("---- DEVICES ----", proc)
        self.assert_in_proc_out("---- OVERLAYS ----", proc)

    @unittest.skipUnless(
        is_dfx_mgr_available(),
        "dfx-mgr component not installed. Install with: snap install fpgad+dfx-mgr.comp",
    )
    def test_status_with_platform_option_xlnx(self):
        """Test status command with explicit --platform option set to `xlnx`."""
        proc = self.run_fpgad(["--platform", "xlnx", "status"])
        self.assert_in_proc_out(
            "#  Accel_type  user_load_type user_load_region Base", proc
        )

    def test_status_with_specific_device_option(self):
        """Test status command with explicit --device option (runs last to allow daemon startup)."""
        proc = self.run_fpgad(["--device", "fpga0", "status"])
        self.assert_proc_succeeds(proc)
