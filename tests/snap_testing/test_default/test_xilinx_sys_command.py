#!/usr/bin/env python3
"""Tests for the xlnx-sys command (low-level read/write interface)."""

from common.base_test import FPGATestBase


class TestXlnxSysCommand(FPGATestBase):
    """Test the xlnx-sys command for low-level property access."""

    def test_xlnx_sys_read_property_name(self):
        """Test reading FPGA device name via xlnx-sys read."""
        proc = self.run_fpgad(
            [
                "xlnx-sys",
                "read",
                "read_property",
                "/sys/class/fpga_manager/fpga0/name",
            ]
        )
        self.assert_proc_succeeds(proc)
        self.assertIn("zynqmp", proc.stdout.lower())

    def test_xlnx_sys_read_flags(self):
        """Test reading FPGA flags via xlnx-sys read."""
        proc = self.run_fpgad(["xlnx-sys", "read", "read_flags", "fpga0"])
        self.assert_proc_succeeds(proc)

    def test_xlnx_sys_write_flags(self):
        """Test writing FPGA flags via xlnx-sys write."""
        proc = self.run_fpgad(["xlnx-sys", "write", "write_flags", "fpga0", "0"])
        self.assert_proc_succeeds(proc)

    def test_xlnx_sys_write_property(self):
        """Test writing FPGA property via xlnx-sys write."""
        proc = self.run_fpgad(
            [
                "xlnx-sys",
                "write",
                "write_property",
                "/sys/class/fpga_manager/fpga0/flags",
                "0",
            ]
        )
        self.assert_proc_succeeds(proc)

    def test_xlnx_sys_read_nonexistent_property(self):
        """Test reading non-existent property via xlnx-sys read."""
        proc = self.run_fpgad(
            [
                "xlnx-sys",
                "read",
                "read_property",
                "/sys/class/fpga_manager/fpga0/nonexistent",
            ]
        )
        self.assert_proc_fails(proc)

    def test_xlnx_sys_write_nonexistent_property(self):
        """Test writing to non-existent property via xlnx-sys write."""
        proc = self.run_fpgad(
            [
                "xlnx-sys",
                "write",
                "write_property",
                "/sys/class/fpga_manager/fpga0/nonexistent",
                "value",
            ]
        )
        self.assert_proc_fails(proc)
