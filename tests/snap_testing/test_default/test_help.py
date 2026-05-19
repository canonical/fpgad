#!/usr/bin/env python3
"""Help command tests (platform-independent)."""

from common.base_test import FPGATestBase


class TestHelp(FPGATestBase):
    """Test help command functionality."""

    def test_help_main(self):
        """Test main help command."""
        proc = self.run_fpgad(["help"])
        self.assert_proc_succeeds(proc)

    def test_help_main_as_flag(self):
        """Test help as --help flag."""
        proc = self.run_fpgad(["--help"])
        self.assert_proc_succeeds(proc)

    def test_help_set(self):
        """Test help for set command."""
        proc = self.run_fpgad(["help", "set"])
        self.assert_proc_succeeds(proc)

    def test_help_remove(self):
        """Test help for remove command."""
        proc = self.run_fpgad(["help", "remove"])
        self.assert_proc_succeeds(proc)

    def test_help_remove_overlay(self):
        """Test help for remove overlay subcommand."""
        proc = self.run_fpgad(["help", "remove", "overlay"])
        self.assert_proc_succeeds(proc)

    def test_help_remove_bitstream(self):
        """Test help for remove bitstream subcommand."""
        proc = self.run_fpgad(["help", "remove", "bitstream"])
        self.assert_proc_succeeds(proc)

    def test_help_load(self):
        """Test help for load command."""
        proc = self.run_fpgad(["help", "load"])
        self.assert_proc_succeeds(proc)

    def test_help_load_bitstream(self):
        """Test help for load bitstream subcommand."""
        proc = self.run_fpgad(["help", "load", "bitstream"])
        self.assert_proc_succeeds(proc)

    def test_help_load_overlay(self):
        """Test help for load overlay subcommand."""
        proc = self.run_fpgad(["help", "load", "overlay"])
        self.assert_proc_succeeds(proc)

    def test_help_universal(self):
        """Test help for universal command."""
        proc = self.run_fpgad(["help", "universal"])
        self.assert_proc_succeeds(proc)

    def test_help_universal_read(self):
        """Test help for universal read subcommand."""
        proc = self.run_fpgad(["help", "universal", "read"])
        self.assert_proc_succeeds(proc)

    def test_help_universal_write(self):
        """Test help for universal write subcommand."""
        proc = self.run_fpgad(["help", "universal", "write"])
        self.assert_proc_succeeds(proc)

    def test_help_dfx_mgr(self):
        """Test help for dfx-mgr command."""
        proc = self.run_fpgad(["help", "dfx-mgr"])
        self.assert_proc_succeeds(proc)
