"""Align replayed workspace versions without upgrading locked external dependencies."""

import argparse
from pathlib import Path
import subprocess


def sync_release_lock(workspace: Path, *, check: bool = False) -> None:
    # Published patches can carry the previous release's workspace versions.
    # --workspace keeps registry/git dependencies at their existing locked versions.
    workspace = workspace.resolve()
    command = [
        "cargo",
        "update",
        "--workspace",
        "--manifest-path",
        str(workspace / "Cargo.toml"),
    ]
    if check:
        command.append("--locked")
    subprocess.run(command, cwd=workspace, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workspace", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        sync_release_lock(args.workspace, check=args.check)
    except subprocess.CalledProcessError as error:
        raise SystemExit(error.returncode) from error
