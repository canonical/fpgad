#!/usr/bin/env python3
"""dfx-mgr passthrough command tests for xlnx platform."""

import unittest

from common.base_test import FPGATestBase
from common.helpers import is_dfx_mgr_available


@unittest.skipUnless(
    is_dfx_mgr_available(),
    "dfx-mgr component not installed. Install with: snap install fpgad+dfx-mgr.comp",
)
class TestDfxMgrPassthrough(FPGATestBase):
    """Test dfx-mgr passthrough command with security validation."""

    def test_dfx_mgr_list_package(self):
        """Test dfx-mgr -listPackage command executes successfully."""
        proc = self.run_fpgad(["dfx-mgr", "-listPackage"])
        self.assert_proc_succeeds(proc)

    # ============================================================
    # ============== SECURITY VALIDATION TESTS ===================
    # ============================================================

    def test_dfx_mgr_blocks_semicolon_command_chain(self):
        """Test that semicolon command chaining is blocked."""
        proc = self.run_fpgad(["dfx-mgr", "-listPackage;rm", "-rf", "/"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_ampersand_background(self):
        """Test that ampersand background command injection is blocked."""
        proc = self.run_fpgad(
            ["dfx-mgr", "-listPackage", "&", "sudo", "rm", "-rf", "/"]
        )
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_pipe(self):
        """Test that pipe command injection is blocked."""
        proc = self.run_fpgad(["dfx-mgr", "-listPackage", "|", "grep", "secret"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_dollar_variable(self):
        """Test that dollar sign variable expansion is blocked."""
        proc = self.run_fpgad(["dfx-mgr", "-load", "$HOME"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_backtick_substitution(self):
        """Test that backtick command substitution is blocked."""
        proc = self.run_fpgad(["dfx-mgr", "`whoami`"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_redirect_output(self):
        """Test that output redirection is blocked."""
        proc = self.run_fpgad(["dfx-mgr", "-listPackage", ">", "/tmp/output"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_redirect_input(self):
        """Test that input redirection is blocked."""
        proc = self.run_fpgad(["dfx-mgr", "-listPackage", "<", "/etc/passwd"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_glob_pattern(self):
        """Test that glob patterns are blocked."""
        proc = self.run_fpgad(["dfx-mgr", "-load", "*"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_double_quotes(self):
        """Test that double quotes are blocked."""
        proc = self.run_fpgad(["dfx-mgr", '"malicious"'])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_single_quotes(self):
        """Test that single quotes are blocked."""
        proc = self.run_fpgad(["dfx-mgr", "'malicious'"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_handles_newline_as_whitespace(self):
        """Test that newlines in arguments are treated as whitespace by the CLI.

        The CLI uses split_whitespace() which treats \\n as a delimiter.
        This is safe because:
        1. It splits "-list\\nPackage" into ["-list", "Package"]
        2. Each resulting argument is validated by the daemon
        3. If the split produces dangerous patterns, they're caught (e.g., "cmd\\n& rm" → ["cmd", "&", "rm"])

        This test documents that newlines cause splitting, not rejection.
        The daemon still blocks newlines within individual arguments as a defense-in-depth measure.
        """
        # The CLI will split this into ["-list", "Package"], both of which are valid
        # (though the command may fail because dfx-mgr doesn't have this exact flag)
        proc = self.run_fpgad(["dfx-mgr", "-list\nPackage"])
        # Should not fail due to validation error
        # May fail because the flag is invalid, but that's a different error
        if proc.returncode != 0:
            self.assert_not_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_dangerous_chars_after_newline_split(self):
        """Test that dangerous characters are caught even after newline splitting.

        If someone tries to inject commands using newlines, e.g., "-listPackage\\n& rm -rf /",
        the CLI will split it into multiple arguments, and the daemon will still catch
        the dangerous characters like '&'.

        This demonstrates defense-in-depth: newline splitting doesn't bypass validation.
        """
        # This will be split into ["-listPackage", "&", "rm", "-rf", "/"]
        # The "&" should be caught as a dangerous character
        proc = self.run_fpgad(["dfx-mgr", "-listPackage\n&", "rm", "-rf", "/"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_backslash(self):
        """Test that backslash is blocked."""
        proc = self.run_fpgad(["dfx-mgr", "-list\\Package"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_subshell_parentheses(self):
        """Test that parentheses for subshells are blocked."""
        proc = self.run_fpgad(["dfx-mgr", "(ls)"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_dollar_parentheses(self):
        """Test that dollar-parentheses command substitution is blocked."""
        proc = self.run_fpgad(["dfx-mgr", "$(whoami)"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_curly_braces(self):
        """Test that curly braces are blocked."""
        proc = self.run_fpgad(["dfx-mgr", "{test}"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_square_brackets(self):
        """Test that square brackets are blocked."""
        proc = self.run_fpgad(["dfx-mgr", "[test]"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_question_mark(self):
        """Test that question marks are blocked."""
        proc = self.run_fpgad(["dfx-mgr", "-load", "test?"])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_blocks_too_long_argument(self):
        """Test that excessively long arguments are blocked."""
        long_arg = "a" * 1025  # Over the 1024 character limit
        proc = self.run_fpgad(["dfx-mgr", "-load", long_arg])
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("too long", proc)

    def test_dfx_mgr_allows_valid_long_argument(self):
        """Test that arguments at the length boundary are accepted."""
        # This will fail because the package doesn't exist, but should pass validation
        boundary_arg = "a" * 1024  # Exactly 1024 characters
        proc = self.run_fpgad(["dfx-mgr", "-load", "0", boundary_arg])
        # Should fail because package doesn't exist, not because of validation
        self.assert_proc_fails(proc)
        # Should NOT contain validation errors
        self.assert_not_in_proc_out("dangerous character", proc)
        self.assert_not_in_proc_out("too long", proc)

    def test_dfx_mgr_allows_at_sign_in_arguments(self):
        """Test that @ sign is allowed in arguments (e.g., for version strings)."""
        proc = self.run_fpgad(["dfx-mgr", "-load", "0", "package@version"])
        # Should fail because package doesn't exist, not because of validation
        self.assert_proc_fails(proc)
        # Should NOT contain validation errors
        self.assert_not_in_proc_out("dangerous character", proc)
        self.assert_not_in_proc_out("invalid characters", proc)

        proc = self.run_fpgad(["dfx-mgr", "-list@Package"])
        # May or may not succeed depending on if this is a valid flag
        # But should not fail on validation
        if proc.returncode != 0:
            stderr_lower = proc.stderr.lower()
            if (
                "dangerous character" in stderr_lower
                or "invalid characters" in stderr_lower
            ):
                self.fail("@ sign was incorrectly blocked by validation")

    def test_dfx_mgr_allows_special_characters_for_international_users(self):
        """Test that non-dangerous special characters are allowed (for international users)."""
        # These will fail because packages don't exist, but should pass validation
        special_names = ["design#1", "package+variant", "file=test", "name%value"]

        for name in special_names:
            proc = self.run_fpgad(["dfx-mgr", "-load", "0", name])
            # Should fail because package doesn't exist, not because of validation
            self.assert_proc_fails(proc)
            # Should NOT contain validation errors
            stderr_lower = proc.stderr.lower()
            if "dangerous character" in stderr_lower:
                self.fail(
                    f"Non-dangerous character in '{name}' was incorrectly flagged as dangerous"
                )
            if "invalid characters" in stderr_lower:
                self.fail(
                    f"Valid special character in '{name}' was incorrectly flagged as invalid"
                )

    def test_dfx_mgr_allows_valid_package_names(self):
        """Test that valid package naming patterns are accepted."""
        # These will fail because packages don't exist, but should pass validation
        valid_names = [
            "my_design",
            "my-package",
            "design_v2.0",
            "package-with-dashes_and_underscores.v1",
        ]

        for name in valid_names:
            proc = self.run_fpgad(["dfx-mgr", "-load", "0", name])
            # Should fail because package doesn't exist, not because of validation
            self.assert_proc_fails(proc)
            # Should NOT contain validation errors (check both stdout and stderr)
            if (
                "dangerous character" in proc.stdout
                or "dangerous character" in proc.stderr
            ):
                self.fail(
                    f"Valid package name '{name}' was incorrectly flagged as dangerous"
                )
            if (
                "invalid characters" in proc.stdout
                or "invalid characters" in proc.stderr
            ):
                self.fail(
                    f"Valid package name '{name}' was incorrectly flagged as having invalid characters"
                )

    def test_dfx_mgr_allows_valid_characters(self):
        """Test that valid character patterns are accepted."""
        # Test with underscore in flag name
        proc = self.run_fpgad(["dfx-mgr", "-list_Package"])
        # May or may not succeed depending on if this is a real dfx-mgr flag
        # But should not fail on validation
        if proc.returncode != 0:
            self.assert_not_in_proc_err("invalid characters", proc)
            self.assert_not_in_proc_err("dangerous character", proc)

    def test_dfx_mgr_allows_paths_with_slashes(self):
        """Test that file paths with forward slashes are accepted."""
        # This will fail because the file doesn't exist, but should pass validation
        proc = self.run_fpgad(["dfx-mgr", "-load", "0", "/path/to/package"])
        self.assert_proc_fails(proc)
        # Should NOT contain validation errors
        self.assert_not_in_proc_out("dangerous character", proc)
        self.assert_not_in_proc_out("invalid characters", proc)

    def test_dfx_mgr_allows_paths_with_colons(self):
        """Test that paths with colons are accepted (for edge cases like container paths)."""
        # This will fail because the path doesn't exist, but should pass validation
        proc = self.run_fpgad(["dfx-mgr", "-load", "0", "/path:with:colons"])
        self.assert_proc_fails(proc)
        # Should NOT contain validation errors
        self.assert_not_in_proc_out("dangerous character", proc)
        self.assert_not_in_proc_out("invalid characters", proc)

    def test_dfx_mgr_real_world_attack_example(self):
        """Test the specific attack pattern mentioned in the issue."""
        # Original attack: fpgad_cli dfx-mgr -listPackage & sudo rm -rf / --no-preserve-root
        proc = self.run_fpgad(
            [
                "dfx-mgr",
                "-listPackage",
                "&",
                "sudo",
                "rm",
                "-rf",
                "/",
                "--no-preserve-root",
            ]
        )
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("dangerous character", proc)
        # Ensure the command never reached the shell
        self.assert_in_proc_err("FpgadError::Argument:", proc)


if __name__ == "__main__":
    unittest.main()
