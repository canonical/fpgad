#!/usr/bin/env bash
#
# This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
#
# Copyright 2026 Canonical Ltd.
#
# SPDX-License-Identifier: GPL-3.0-only
#
# fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
#
# fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.
#

# Convenience script to run all FPGA snap tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "Running FPGA Snap Tests"
echo "Working directory: $(pwd)"
echo "=========================================="

# Check if specific test suite is requested
if [ $# -eq 0 ]; then
    echo "Running ALL tests..."
    python3 -m unittest discover -s "$SCRIPT_DIR" -p "test_*.py" -v
elif [ "$1" == "universal" ]; then
    echo "Running UNIVERSAL platform tests..."
    python3 -m unittest discover -s "$SCRIPT_DIR/test_universal" -p "test_*.py" -v
elif [ "$1" == "xlnx" ]; then
    echo "Running xilinx dfx-mgr platform tests..."
    python3 -m unittest discover -s "$SCRIPT_DIR/test_xlnx" -p "test_*.py" -v
elif [ "$1" == "default" ]; then
    echo "Running DEFAULT (no platform) tests..."
    python3 -m unittest discover -s "$SCRIPT_DIR/test_default" -p "test_*.py" -v
else
    echo "Usage: $0 [universal|xlnx|default]"
    echo "  No argument: run all tests"
    echo "  universal: run only --platform universal tests"
    echo "  xlnx: run only --platform xlnx tests"
    echo "  default: run only tests without platform flag"
    exit 1
fi

echo "=========================================="
echo "Tests completed successfully!"
echo "=========================================="

