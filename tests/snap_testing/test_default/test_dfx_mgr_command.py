#!/usr/bin/env python3
"""Tests for the dfx-mgr command (passes commands to dfx-mgr-client)."""

import unittest
from common.base_test import FPGATestBase
from common.helpers import is_dfx_mgr_available


class TestDfxMgrCommand(FPGATestBase):
    """Test the dfx-mgr command for Xilinx DFX manager operations."""

    @unittest.skipUnless(
        is_dfx_mgr_available(),
        "dfx-mgr component not installed. Install with: snap install fpgad+dfx-mgr.comp",
    )
    def test_dfx_mgr_list_package(self):
        """Test listing packages via dfx-mgr command."""
        proc = self.run_fpgad(["dfx-mgr", "--", "-listPackage"])
        self.assert_proc_succeeds(proc)

    def test_dfx_mgr_without_component(self):
        """Test dfx-mgr command fails gracefully when component is not installed."""
        if is_dfx_mgr_available():
            self.skipTest("Test only runs when dfx-mgr is NOT available")

        proc = self.run_fpgad(["dfx-mgr", "--", "-listPackage"])
        self.assert_proc_fails(proc)
        self.assertIn("feature not enabled", proc.stderr.lower())

    @unittest.skipUnless(
        is_dfx_mgr_available(),
        "dfx-mgr component not installed. Install with: snap install fpgad+dfx-mgr.comp",
    )
    def test_dfx_mgr_invalid_command(self):
        """Test dfx-mgr with invalid command."""
        proc = self.run_fpgad(["dfx-mgr", "invalidCommand"])
        self.assert_proc_fails(proc)
