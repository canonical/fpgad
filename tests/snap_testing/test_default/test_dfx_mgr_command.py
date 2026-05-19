#!/usr/bin/env python3
"""Tests for the dfx-mgr command (passes commands to dfx-mgr-client)."""

import unittest
from common.base_test import FPGATestBase
from common.helpers import (
    is_dfx_mgr_available,
    get_test_data_path,
)


class TestDfxMgrCommand(FPGATestBase):
    """Test the dfx-mgr command for Xilinx DFX manager operations."""

    @unittest.skipUnless(
        is_dfx_mgr_available(),
        "dfx-mgr component not installed. Install with: snap install fpgad+dfx-mgr.comp",
    )
    def test_dfx_mgr_list_package(self):
        """Test listing packages via dfx-mgr command."""
        proc = self.run_fpgad(["dfx-mgr", "-listPackage"])
        self.assert_proc_succeeds(proc)

    def test_dfx_mgr_without_component(self):
        """Test dfx-mgr command fails gracefully when component is not installed."""
        if is_dfx_mgr_available():
            self.skipTest("Test only runs when dfx-mgr is NOT available")

        proc = self.run_fpgad(["dfx-mgr", "-listPackage"])
        self.assert_proc_fails(proc)
        self.assertIn("dfx-mgr-client not found", proc.stderr.lower())

    def test_multi_param(self):
        """Test loading bitstream with overlay from full absolute path to check multiple arguments work."""
        test_data_path = get_test_data_path()
        bit_path = test_data_path / "k26-starter-kits" / "k26_starter_kits.bit.bin"
        o_lay_path = test_data_path / "k26-starter-kits" / "k26_starter_kits.dtbo"
        proc = self.run_fpgad(
            ["dfx-mgr", "-b", str(bit_path.resolve()), "-o", str(o_lay_path.resolve())]
        )
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("Loaded with slot_handle ", proc)

    @unittest.skipUnless(
        is_dfx_mgr_available(),
        "dfx-mgr component not installed. Install with: snap install fpgad+dfx-mgr.comp",
    )
    def test_dfx_mgr_invalid_command(self):
        """Test dfx-mgr with invalid command."""
        proc = self.run_fpgad(["dfx-mgr", "invalidCommand"])
        self.assert_proc_fails(proc)
