#!/usr/bin/env python3
"""Set command tests for xlnx-sys platform."""

from common.base_test import FPGATestBase


class TestSetXlnxSys(FPGATestBase):
    """Test set command with --platform xlnx-sys."""

    PLATFORM = "xlnx-sys"

    def test_set_flags_nonzero(self):
        """Test setting flags to non-zero value."""
        proc = self.run_fpgad(["--platform", self.PLATFORM, "set", "flags", "20"])
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out(
            "20 written to /sys/class/fpga_manager/fpga0/flags", proc
        )
        self.check_fpga0_attribute("flags", "20")

    def test_set_flags_string(self):
        """Test setting flags with invalid string value fails."""
        proc = self.run_fpgad(["--platform", self.PLATFORM, "set", "flags", "zero"])
        self.assert_proc_fails(proc)
        self.check_fpga0_attribute("flags", "0")

    def test_set_state(self):
        """Test setting state (read-only attribute) fails."""
        old = self.get_fpga0_attribute("state")

        proc = self.run_fpgad(["--platform", self.PLATFORM, "set", "state", "0"])
        self.assert_proc_fails(proc)
        self.check_fpga0_attribute("state", old)

    def test_set_flags_float(self):
        """Test setting flags with float value fails."""
        proc = self.run_fpgad(["--platform", self.PLATFORM, "set", "flags", "0.2"])
        self.assert_proc_fails(proc)
        self.check_fpga0_attribute("flags", "0")

    def test_set_flags_zero(self):
        """Test setting flags to zero."""
        proc = self.run_fpgad(["--platform", self.PLATFORM, "set", "flags", "0"])
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out(
            "0 written to /sys/class/fpga_manager/fpga0/flags", proc
        )
        self.check_fpga0_attribute("flags", "0")
