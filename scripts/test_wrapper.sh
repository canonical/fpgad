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

# Check if the tests component is installed
if [ ! -d "$SNAP_COMPONENTS/test" ]; then
    echo "Tests component not installed. Install with \"snap install fpgad+test.comp\""
    exit 1
fi

# Change to the user's current directory (from outside the snap)
# This allows tests to access files in the working directory
if [ -n "$SNAP_USER_COMMON" ]; then
    # If test data is in SNAP_USER_COMMON, use that
    if [ -d "$SNAP_USER_COMMON/fpgad" ]; then
        cd "$SNAP_USER_COMMON" || return
        echo "Test data found at $SNAP_USER_COMMON/fpgad"
        echo "Changed working directory to: $SNAP_USER_COMMON"
    else
        echo "Test data directory not found at $SNAP_USER_COMMON/fpgad"
    fi
elif [ -n "$SUDO_USER" ] && [ -d "/home/$SUDO_USER/snap/fpgad/common/fpgad" ]; then
    # Running with sudo - use the original user's snap directory
    cd "/home/$SUDO_USER/snap/fpgad/common" || return
    echo "Test data found at /home/$SUDO_USER/snap/fpgad/common/fpgad (via SUDO_USER)"
    echo "Changed working directory to: /home/$SUDO_USER/snap/fpgad/common"
    # Export SNAP_USER_COMMON for tests to use
    export SNAP_USER_COMMON="/home/$SUDO_USER/snap/fpgad/common"
elif [ -n "$SNAP_COMMON" ] && [ -d "$SNAP_COMMON/fpgad" ]; then
    # Fall back to SNAP_COMMON (system-wide)
    cd "$SNAP_COMMON" || return
    echo "Test data found at $SNAP_COMMON/fpgad"
    echo "Changed working directory to: $SNAP_COMMON"
else
    echo "SNAP_USER_COMMON not set and no test data found"
fi

# Add snap_testing to PYTHONPATH so imports work
export PYTHONPATH="$SNAP_COMPONENTS/test:$PYTHONPATH"

# Debug output
echo "SNAP_COMPONENTS: $SNAP_COMPONENTS"
if [ -d "$SNAP_COMPONENTS/dfx-mgr" ]; then
    echo "dfx-mgr component found at: $SNAP_COMPONENTS/dfx-mgr"
    if [ -f "$SNAP_COMPONENTS/dfx-mgr/usr/bin/dfx-mgr-client" ]; then
        echo "dfx-mgr-client found: $SNAP_COMPONENTS/dfx-mgr/usr/bin/dfx-mgr-client"
    else
        echo "WARNING: dfx-mgr component directory exists but dfx-mgr-client not found"
    fi
else
    echo "dfx-mgr component NOT installed (xlnx tests will be skipped)"
fi

# Run the snap tests
exec "$SNAP_COMPONENTS/test/test_snap.sh" "$@"

