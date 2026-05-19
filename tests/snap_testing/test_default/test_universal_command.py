#!/usr/bin/env python3
"""Tests for the universal command (low-level read/write interface)."""

from common.base_test import FPGATestBase


class TestUniversalCommand(FPGATestBase):
    """Test the universal command for low-level property access."""

    def test_universal_read_property_name(self):
        """Test reading FPGA device name via universal read."""
        proc = self.run_fpgad(
            ["universal", "read", "read_property", "/sys/class/fpga_manager/fpga0/name"]
        )
        self.assert_proc_succeeds(proc)
        self.assertIn("zynqmp", proc.stdout.lower())

    def test_universal_read_flags(self):
        """Test reading FPGA flags via universal read."""
        proc = self.run_fpgad(["universal", "read", "read_flags", "fpga0"])
        self.assert_proc_succeeds(proc)

    def test_universal_write_flags(self):
        """Test writing FPGA flags via universal write."""
        proc = self.run_fpgad(["universal", "write", "write_flags", "fpga0", "0"])
        self.assert_proc_succeeds(proc)

    def test_universal_write_property(self):
        """Test writing FPGA property via universal write."""
        proc = self.run_fpgad(
            [
                "universal",
                "write",
                "write_property",
                "/sys/class/fpga_manager/fpga0/flags",
                "0",
            ]
        )
        self.assert_proc_succeeds(proc)

    def test_universal_read_nonexistent_property(self):
        """Test reading non-existent property via universal read."""
        proc = self.run_fpgad(
            [
                "universal",
                "read",
                "read_property",
                "/sys/class/fpga_manager/fpga0/nonexistent",
            ]
        )
        self.assert_proc_fails(proc)

    def test_universal_write_nonexistent_property(self):
        """Test writing to non-existent property via universal write."""
        proc = self.run_fpgad(
            [
                "universal",
                "write",
                "write_property",
                "/sys/class/fpga_manager/fpga0/nonexistent",
                "value",
            ]
        )
        self.assert_proc_fails(proc)
