#!/usr/bin/env python3
# This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
#
# Copyright 2025 Canonical Ltd.
#
# SPDX-License-Identifier: GPL-3.0-only
#
# fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
#
# fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

"""
Version policy checks for Cargo.toml.

Supported modes:
1) bump (default):
    Ensure [workspace.package].version increased compared to the base branch.
2) validate:
    Validate a target version and enforce lockstep between
    [workspace.package].version and [workspace.dependencies].fpgad_macros.version.
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

import tomllib


def run_git_command(cmd: list[str]) -> str:
    """Run a git command and return the output."""
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def get_cargo_toml_content(ref: str | None = None) -> str:
    """Get Cargo.toml content from a specific ref or current working tree."""
    if ref:
        try:
            return run_git_command(["git", "show", f"{ref}:Cargo.toml"])
        except subprocess.CalledProcessError as e:
            print(f"::error::Failed to get Cargo.toml from {ref}: {e}")
            sys.exit(1)
    else:
        cargo_path = Path("Cargo.toml")
        if not cargo_path.exists():
            print("::error::Cargo.toml not found in current directory")
            sys.exit(1)
        return cargo_path.read_text()


def parse_toml(toml_content: str) -> dict:
    """Parse Cargo.toml content into a dictionary."""
    try:
        return tomllib.loads(toml_content)
    except Exception as e:
        print(f"::error::Failed to parse TOML: {e}")
        sys.exit(1)


def get_workspace_version(toml_data: dict) -> str:
    """Return [workspace.package].version."""
    version = toml_data.get("workspace", {}).get("package", {}).get("version")
    if not version:
        print("::error::Could not find version in [workspace.package] section")
        sys.exit(1)
    return version


def get_fpgad_macros_dependency_version(toml_data: dict) -> str:
    """Return [workspace.dependencies].fpgad_macros.version."""
    dependency = (
        toml_data.get("workspace", {}).get("dependencies", {}).get("fpgad_macros")
    )

    if not dependency:
        print("::error::Could not find [workspace.dependencies].fpgad_macros")
        sys.exit(1)

    if isinstance(dependency, str):
        # String-form dependency declaration, e.g. fpgad_macros = "0.1.1"
        return dependency

    if isinstance(dependency, dict):
        version = dependency.get("version")
        if not version:
            print(
                "::error::Could not find version in [workspace.dependencies].fpgad_macros"
            )
            sys.exit(1)
        return version

    print("::error::Invalid format for [workspace.dependencies].fpgad_macros")
    sys.exit(1)


def parse_semver(version: str) -> tuple[int, int, int]:
    """
    Parse a semantic version string.

    Returns (major, minor, patch) as integers.
    Ignores pre-release and build metadata.
    """
    # Extract major.minor.patch (ignore pre-release and build metadata)
    match = re.match(r"^(\d+)\.(\d+)\.(\d+)", version)
    if not match:
        print(f"::error::Invalid semver format: {version}")
        sys.exit(1)

    return int(match.group(1)), int(match.group(2)), int(match.group(3))


def compare_versions(old_version: str, new_version: str) -> bool:
    """
    Compare two semantic versions.

    Returns True if new_version > old_version, False otherwise.
    """
    old_major, old_minor, old_patch = parse_semver(old_version)
    new_major, new_minor, new_patch = parse_semver(new_version)

    if new_major > old_major:
        return True
    elif new_major == old_major and new_minor > old_minor:
        return True
    elif new_major == old_major and new_minor == old_minor and new_patch > old_patch:
        return True

    return False


def enforce_version_lock(
    workspace_version: str, macros_dependency_version: str
) -> None:
    """Ensure workspace and fpgad_macros dependency versions stay in lockstep."""
    if workspace_version != macros_dependency_version:
        print(
            "::error::Version lock mismatch!%0A"
            f"[workspace.package].version: {workspace_version}%0A"
            f"[workspace.dependencies].fpgad_macros.version: {macros_dependency_version}%0A"
            "%0A"
            "These versions must always match.%0A"
            "Please update root Cargo.toml so both versions are identical.%0A"
        )
        sys.exit(1)


def validate_mode(expected_version: str, ref: str | None) -> None:
    """Validate expected version and lockstep constraints."""
    toml_content = get_cargo_toml_content(ref)
    toml_data = parse_toml(toml_content)

    workspace_version = get_workspace_version(toml_data)
    macros_dependency_version = get_fpgad_macros_dependency_version(toml_data)

    # Validate both strings are semver-like and lockstep.
    parse_semver(workspace_version)
    parse_semver(macros_dependency_version)

    enforce_version_lock(workspace_version, macros_dependency_version)

    if workspace_version != expected_version:
        print(
            "::error::Requested version does not match Cargo.toml.%0A"
            f"Requested version: {expected_version}%0A"
            f"[workspace.package].version: {workspace_version}%0A"
        )
        sys.exit(1)

    print(
        "✅ Version validation passed: "
        f"requested={expected_version}, workspace={workspace_version}, "
        f"fpgad_macros dependency={macros_dependency_version}"
    )


def bump_mode() -> None:
    """Ensure workspace version was bumped when Rust sources changed."""
    # Get base ref from environment or use 'main' as default
    base_ref = os.environ.get("GITHUB_BASE_REF", "main")

    print("::group::extract version numbers")
    # Get versions
    old_toml = get_cargo_toml_content(f"origin/{base_ref}")
    old_data = parse_toml(old_toml)
    old_version = get_workspace_version(old_data)

    new_toml = get_cargo_toml_content()
    new_data = parse_toml(new_toml)
    new_version = get_workspace_version(new_data)
    new_macros_dependency_version = get_fpgad_macros_dependency_version(new_data)

    print(f"Old version: {old_version}")
    print(f"New version: {new_version}")
    print(
        "New [workspace.dependencies].fpgad_macros.version: "
        f"{new_macros_dependency_version}"
    )
    print("::endgroup::")

    print("::group::validate version lock")
    enforce_version_lock(new_version, new_macros_dependency_version)
    print(
        "✅ Version lock is valid: "
        f"workspace={new_version}, fpgad_macros dependency={new_macros_dependency_version}"
    )
    print("::endgroup::")

    print("::group::compare versions using semver")
    version_increased = compare_versions(old_version, new_version)

    if not version_increased:
        error_msg = (
            f"::error::Version was not increased!%0A"
            f"Old version: {old_version}%0A"
            f"New version: {new_version}%0A"
            f"%0A"
            f"Rust source files were modified but the version was not bumped in Cargo.toml.%0A"
            f"Please update the version in the [workspace.package] section of the root Cargo.toml.%0A"
            f"Version must follow semver and be greater than {old_version}.%0A"
            f"%0A"
        )
        print(error_msg)
        print("::endgroup::")  # necessary to avoid never-ending group
        sys.exit(1)

    print(f"✅ Version was properly increased: {old_version} → {new_version}")
    print("::endgroup::")


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments while preserving backward compatibility."""
    parser = argparse.ArgumentParser(description="Cargo.toml version policy checks")
    subparsers = parser.add_subparsers(dest="command")

    validate_parser = subparsers.add_parser(
        "validate",
        help="validate requested version and enforce workspace/fpgad_macros lockstep",
    )
    validate_parser.add_argument(
        "--expected-version",
        required=True,
        help="expected release version (for example: 0.1.1)",
    )
    validate_parser.add_argument(
        "--ref",
        required=False,
        help="optional git ref to read Cargo.toml from (for example commit SHA)",
    )

    # No-argument invocation remains the default bump check.
    return parser.parse_args()


def main():
    args = parse_args()

    if args.command == "validate":
        validate_mode(args.expected_version, args.ref)
        return

    bump_mode()


if __name__ == "__main__":
    main()
