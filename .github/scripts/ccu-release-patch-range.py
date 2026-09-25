"""Validate and list only the CCU patches above a published release's upstream tag."""

import subprocess
import sys


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


def release_patch_commits(base: str, source: str) -> list[str]:
    base = git("rev-parse", "--verify", f"{base}^{{commit}}")
    source = git("rev-parse", "--verify", f"{source}^{{commit}}")
    if subprocess.run(["git", "merge-base", "--is-ancestor", base, source]).returncode:
        raise ValueError("Published patch source is not based on its upstream tag")
    revision_range = f"{base}..{source}"
    if git("rev-list", "--merges", revision_range):
        raise ValueError("Published patch source must contain only linear release patches")
    commits = git("rev-list", "--reverse", revision_range).splitlines()
    if not commits:
        raise ValueError("Published patch source contains no release patches")
    # Check each commit, not just the net diff, because each commit is replayed.
    for commit in commits:
        paths = git("diff-tree", "--no-commit-id", "--name-only", "-r", commit).splitlines()
        invalid = [
            path
            for path in paths
            if not path.startswith("codex-rs/")
            and path != ".github/scripts/ccu-package-release.ps1"
            and path != ".github/scripts/ccu-package-runtime.py"
        ]
        if invalid:
            raise ValueError(f"Published patch changes paths outside the release allowlist: {invalid}")
    if subprocess.run(["git", "diff", "--quiet", base, source, "--", "codex-rs"]).returncode == 0:
        raise ValueError("Published patch source contains no codex-rs changes")
    return commits


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("Usage: ccu-release-patch-range.py UPSTREAM_COMMIT FORK_COMMIT")
    try:
        print("\n".join(release_patch_commits(sys.argv[1], sys.argv[2])))
    except (ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
