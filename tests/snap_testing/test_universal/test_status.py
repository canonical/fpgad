#!/usr/bin/env python3
"""Status command tests for universal platform."""

from common.base_test import FPGATestBase


class TestStatusUniversal(FPGATestBase):
    """Test status command with --platform universal."""

    PLATFORM = "universal"

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
        self.assert_in_proc_out("operating", status_proc)


# TODO(Artie): Add back missing tests.
