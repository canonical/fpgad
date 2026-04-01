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
Check if version was bumped when Rust source files are modified.

This script compares the version in Cargo.toml between the base branch
and the current branch, ensuring proper semver versioning when Rust
source files have been modified.
"""

import subprocess
import sys
import re
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


def parse_version_from_toml(toml_content: str) -> str:
    """Parse version from Cargo.toml content."""
    try:
        data = tomllib.loads(toml_content)
        version = data.get("workspace", {}).get("package", {}).get("version")

        if not version:
            print("::error::Could not find version in [workspace.package] section")
            sys.exit(1)

        return version
    except Exception as e:
        print(f"::error::Failed to parse TOML: {e}")
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


def main():
    # Get base ref from environment or use 'main' as default
    import os

    base_ref = os.environ.get("GITHUB_BASE_REF", "main")

    print("::group::extract version numbers")
    # Get versions
    old_toml = get_cargo_toml_content(f"origin/{base_ref}")
    old_version = parse_version_from_toml(old_toml)

    new_toml = get_cargo_toml_content()
    new_version = parse_version_from_toml(new_toml)

    print(f"Old version: {old_version}")
    print(f"New version: {new_version}")
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
    else:
        print(f"✅ Version was properly increased: {old_version} → {new_version}")
        print("::endgroup::")


if __name__ == "__main__":
    main()
