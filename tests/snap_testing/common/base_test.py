#!/usr/bin/env python3
"""
Base test class for FPGA snap tests.
"""

import os
import subprocess
import unittest
from pathlib import Path
from subprocess import CompletedProcess
from typing import List

from .helpers import Colors, cleanup_applied_overlays, set_flags


class FPGATestBase(unittest.TestCase):
    """Base test class with common setup, teardown, and assertion methods."""

    def setUp(self):
        """Run before each test."""
        self.cleanup_bitstreams()
        cleanup_applied_overlays()
        self.reset_flags()

    def tearDown(self):
        """Run after each test."""
        self.cleanup_bitstreams()
        cleanup_applied_overlays()

    @classmethod
    def tearDownClass(cls):
        """Run once after all tests in this class are finished."""
        cleanup_applied_overlays()
        set_flags(0)

    def cleanup_bitstreams(self):
        """Remove any loaded bitstreams to ensure clean test state."""
        # Try to remove bitstream - ignore errors if nothing to remove
        try:
            proc = self.run_fpgad(["remove", "bitstream"])
            # Don't assert success - it's OK if there's nothing to remove
            if proc.returncode == 0:
                print(f"{Colors.CYAN}[INFO]{Colors.RESET} Cleaned up loaded bitstream")
        except Exception as e:
            # Silently ignore - we're just trying to clean up
            print(
                f"{Colors.YELLOW}[WARN]{Colors.RESET} Could not cleanup bitstream: {e}"
            )

    # ============================================================
    # ===================== HELPER FUNCTIONS =====================
    # ============================================================

    def assert_proc_succeeds(self, proc, msg=None):
        """Assert that a process completed successfully, including stdout/stderr on failure."""
        if msg is None:
            msg = f"Return code is {proc.returncode} when expecting 0"
        full_msg = (
            f"{msg}\n"
            f"Status code:\t{proc.returncode}\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}"
        )
        self.assertEqual(proc.returncode, 0, full_msg)

    def assert_proc_fails(self, proc, msg=None):
        """Assert that a process failed, including stdout/stderr on failure."""
        if msg is None:
            msg = f"Return code is {proc.returncode} when expecting nonzero"
        full_msg = (
            f"{msg}\n"
            f"Status code:\t{proc.returncode}\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}"
        )
        self.assertNotEqual(proc.returncode, 0, full_msg)

    def assert_in_proc_out(
        self, substring: str, proc: CompletedProcess, msg: str = None
    ):
        """Assert that a substring exists in stdout, including stdout/stderr on failure."""
        if msg is None:
            msg = f"'{substring}' not found in output."
        full_msg = f"{msg}\nstdout:\t{proc.stdout}\nstderr:\t{proc.stderr}"
        self.assertIn(substring, proc.stdout, full_msg)

    def assert_not_in_proc_out(
        self, substring: str, proc: CompletedProcess, msg: str = None
    ):
        """Assert that a substring does not exist in stdout, including stdout/stderr on failure."""
        if msg is None:
            msg = f"Undesired '{substring}' found in output."
        full_msg = f"{msg}\nstdout:\t{proc.stdout}\nstderr:\t{proc.stderr}"
        self.assertNotIn(substring, proc.stdout, full_msg)

    def assert_not_in_proc_err(
        self, substring: str, proc: CompletedProcess, msg: str = None
    ):
        """Assert that a substring does not exist in stdout, including stdout/stderr on failure."""
        if msg is None:
            msg = f"Undesired '{substring}' found in stderr."
        full_msg = f"{msg}\nstdout:\t{proc.stdout}\nstderr:\t{proc.stderr}"
        self.assertNotIn(substring, proc.stderr, full_msg)

    def assert_in_proc_err(
        self, substring: str, proc: CompletedProcess, msg: str = None
    ):
        """Assert that a substring exists in stderr, including stdout/stderr on failure."""
        if msg is None:
            msg = f"'{substring}' not found in stderr."
        full_msg = f"{msg}\nstdout:\t{proc.stdout}\nstderr:\t{proc.stderr}"
        self.assertIn(substring, proc.stderr, full_msg)

    @staticmethod
    def get_fpga0_attribute(attr: str):
        """Get an FPGA manager attribute value."""
        path = Path(f"/sys/class/fpga_manager/fpga0/{attr}")
        with open(path, "r") as f:
            real_attr = f.read().strip()
        return real_attr

    def check_fpga0_attribute(self, attr: str, expected: str):
        """Assert that an FPGA manager attribute contains the expected value."""
        path = Path(f"/sys/class/fpga_manager/fpga0/{attr}")
        with open(path, "r") as f:
            real_attr = f.read().strip()
        self.assertIn(expected, real_attr)

    def reset_flags(self):
        """Reset flags (to zero) using system calls, instead of fpgad."""
        print(f"{Colors.CYAN}[INFO]{Colors.RESET} Resetting fpga0's flags to 0")
        set_flags(0)

    def run_fpgad(self, args: List[str]) -> subprocess.CompletedProcess[str]:
        """
        Run the fpgad cli with provided args as a subprocess.

        :param args: list of arguments to provide to the fpgad cli call
        :return: the completed process object, containing return code and captured output
        """
        # When running in snap, call the CLI binary directly
        # The alias 'fpgad' points to the CLI app, but inside snap we need the actual binary
        if os.getenv("SNAP"):
            cmd = [f"{os.getenv('SNAP')}/bin/fpgad_cli"] + args
        else:
            cmd = ["fpgad"] + args

        print(f"{Colors.CYAN}[INFO]{Colors.RESET} Running: {' '.join(cmd)}")

        proc = subprocess.run(
            cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )

        return proc
