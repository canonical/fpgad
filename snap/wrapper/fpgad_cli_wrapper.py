#!/usr/bin/env python3

import os
import subprocess
import sys


def _print_setup_error(snap_root: str | None, cli_path: str) -> None:
    print("Error: unable to find executable fpgad_cli for this snap.", file=sys.stderr)
    if not snap_root:
        print(
            "- SNAP is not set (this usually means the script is running outside snap runtime).",
            file=sys.stderr,
        )
    print(f"- Expected path: {cli_path}", file=sys.stderr)
    print("", file=sys.stderr)
    print("Install and connect the required pieces:", file=sys.stderr)
    print("  snap install fpgad", file=sys.stderr)
    print("  snap install fpgad+<component_name>", file=sys.stderr)
    print(
        "  snap connect <your_snap>:fpgad-cli-app fpgad:fpgad-cli-content",
        file=sys.stderr,
    )
    print("", file=sys.stderr)
    print(
        "If your local snap instance name differs, adjust the left-hand side of the connect command.",
        file=sys.stderr,
    )


def main() -> int:
    snap_root = os.environ.get("SNAP")
    cli_path = (
        os.path.join(snap_root, "fpgad", "cli", "fpgad_cli")
        if snap_root
        else "$SNAP/fpgad/cli/fpgad_cli"
    )

    if snap_root and os.path.isfile(cli_path) and os.access(cli_path, os.X_OK):
        # Pass through all arguments and return the wrapped command exit status.
        completed = subprocess.run([cli_path, *sys.argv[1:]], check=False)
        return completed.returncode

    _print_setup_error(snap_root, cli_path)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
