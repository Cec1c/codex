"""Bound scheduled resolver retries without starting overlapping model jobs."""

import datetime as dt
import json
import subprocess
import sys

MAX_ATTEMPTS = 3
COOLDOWN = dt.timedelta(hours=6)


def retry_decision(runs, issue_number, now):
    # Also covers jobs started before issue-specific run names were introduced.
    if any(run["status"] != "completed" for run in runs):
        return False, "a resolver is still queued or running"
    title = f"CCU conflict resolver #{issue_number}"
    attempts = [run for run in runs if run.get("display_title") == title]
    if len(attempts) >= MAX_ATTEMPTS:
        return False, f"issue #{issue_number} reached {MAX_ATTEMPTS} attempts; manual retry required"
    if not attempts:
        return True, "no tracked attempt for this issue"
    latest = max(attempts, key=lambda run: run["created_at"])
    if latest["conclusion"] not in {"failure", "timed_out", "cancelled", "startup_failure"}:
        return False, "last resolver did not report a retryable failure; inspect its result"
    ended = dt.datetime.fromisoformat(latest["updated_at"].replace("Z", "+00:00"))
    if now - ended < COOLDOWN:
        return False, "failed resolver is within the six-hour cooldown"
    return True, f"retrying completed failure ({len(attempts)}/{MAX_ATTEMPTS} attempts used)"


def main():
    repo, issue = sys.argv[1:]
    if not issue.isdecimal():
        raise ValueError("issue must be a positive number")
    result = subprocess.check_output([
        "gh", "api", "--paginate", "--slurp",
        f"repos/{repo}/actions/workflows/ccu-conflict-resolver.lock.yml/runs?per_page=100",
    ], text=True)
    runs = [run for page in json.loads(result) for run in page["workflow_runs"]]
    allowed, reason = retry_decision(runs, issue, dt.datetime.now(dt.timezone.utc))
    print(reason)
    return 0 if allowed else 1


if __name__ == "__main__":
    sys.exit(main())
