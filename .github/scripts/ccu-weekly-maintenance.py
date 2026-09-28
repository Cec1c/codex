"""Read-only maintenance digest; use one Actions failure notification per seven days."""

import base64
import datetime as dt
import json
import os
from pathlib import Path
import re
import subprocess
import sys

REMINDER_WORKFLOW = "ccu-weekly-maintenance.yml"
INTERVAL = dt.timedelta(days=7)
FAILURES = {"failure", "timed_out", "startup_failure", "action_required"}


def github_api(path):
    result = subprocess.run(
        ["gh", "api", path],
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=45,
        creationflags=0x08000000 if os.name == "nt" else 0,
    )
    return json.loads(result.stdout)


def reminder_due(runs, now, run_id, run_attempt):
    if run_attempt > 1:
        return False, "A rerun cannot send another reminder."
    for run in runs:
        if str(run["id"]) == str(run_id) or run["conclusion"] not in FAILURES:
            continue
        # Completion time is conservative when a previous job was delayed.
        sent = dt.datetime.fromisoformat(run["updated_at"].replace("Z", "+00:00"))
        if now - sent < INTERVAL:
            return False, f"A reminder already ran within seven days: {run['html_url']}"
    return True, "No failed reminder within the last seven days."


def stable_version(tag, fork=False):
    pattern = (
        r"ccu-rust-v(\d+)\.(\d+)\.(\d+)-r\d+" if fork else r"rust-v(\d+)\.(\d+)\.(\d+)"
    )
    match = re.fullmatch(pattern, tag)
    if not match:
        raise ValueError(f"Unexpected stable release tag: {tag}")
    return tuple(map(int, match.groups()))


def collect_findings(api, repo, manager_repo):
    upstream = api("repos/openai/codex/releases/latest")
    fork = api(f"repos/{repo}/releases/latest")
    findings = []
    if stable_version(upstream["tag_name"]) > stable_version(
        fork["tag_name"], fork=True
    ):
        findings.append(
            f"Upstream {upstream['tag_name']} is newer than the published fork "
            f"{fork['tag_name']}: {upstream['html_url']}"
        )

    manifest_file = api(
        f"repos/{manager_repo}/contents/release-channels/stable.json?ref=main"
    )
    manifest = json.loads(base64.b64decode(manifest_file["content"]))
    if manifest["release"]["releaseTag"] != fork["tag_name"]:
        findings.append(
            f"The manager channel still points to {manifest['release']['releaseTag']}; "
            f"the published fork is {fork['tag_name']}. Sync manually when ready."
        )

    issues = api(
        f"repos/{repo}/issues?state=open&labels=ccu-sync-conflict&per_page=100"
    )
    conflicts = []
    for issue in issues:
        if "pull_request" in issue:
            continue
        target = re.search(r"ccu-rust-v\d+\.\d+\.\d+-r\d+", issue["title"])
        if target:
            version = stable_version(target[0], fork=True)
            published = stable_version(fork["tag_name"], fork=True)
            revision = int(target[0].rsplit("-r", 1)[1])
            published_revision = int(fork["tag_name"].rsplit("-r", 1)[1])
            if version < published or (
                version == published and revision <= published_revision
            ):
                continue
        conflicts.append(issue)
    if conflicts:
        links = ", ".join(issue["html_url"] for issue in conflicts[:5])
        findings.append(f"Unresolved upstream replay conflicts: {links}")

    workflows = [
        (repo, "ccu-i18n-release.yml"),
        (repo, "ccu-conflict-resolver.lock.yml"),
        (repo, "ccu-i18n-ci.yml"),
        (manager_repo, "sync-fork-channel.yml"),
        (manager_repo, "release.yml"),
        (manager_repo, "ci.yml"),
    ]
    for repository, workflow in workflows:
        runs = api(
            f"repos/{repository}/actions/workflows/{workflow}/runs?branch=main&per_page=20"
        )["workflow_runs"]
        latest = next((run for run in runs if run["conclusion"] != "skipped"), None)
        if (
            latest
            and latest["status"] == "completed"
            and latest["conclusion"] in FAILURES
        ):
            findings.append(
                f"Latest {repository}/{workflow}: {latest['conclusion']} - {latest['html_url']}"
            )
    return findings


def evaluate(api, repo, manager_repo, now, run_id, run_attempt, dry_run):
    due, reason = False, "Read-only preview; no notification requested."
    if not dry_run:
        if run_attempt > 1:
            return {
                "notify": False,
                "reason": "A rerun cannot send another reminder.",
                "findings": [],
                "dry_run": False,
            }
        try:
            page = 1
            while True:
                history = api(
                    f"repos/{repo}/actions/workflows/{REMINDER_WORKFLOW}/runs?per_page=100&page={page}"
                )["workflow_runs"]
                attempts = list(history)
                for run in history:
                    if str(run["id"]) == str(run_id):
                        continue
                    # A successful rerun replaces the run's conclusion, but must
                    # not erase the original reminder's seven-day cooldown.
                    for attempt in range(1, run.get("run_attempt", 1)):
                        attempts.append(
                            api(
                                f"repos/{repo}/actions/runs/{run['id']}/attempts/{attempt}"
                            )
                        )
                due, reason = reminder_due(attempts, now, run_id, run_attempt)
                if not due or len(history) < 100:
                    break
                page += 1
        except (subprocess.SubprocessError, ValueError, KeyError):
            return {
                "notify": False,
                "reason": "Skipped: unable to verify notification history. No reminder is sent without checking the seven-day interval.",
                "findings": [],
                "dry_run": False,
            }
        if not due:
            return {"notify": False, "reason": reason, "findings": [], "dry_run": False}
    try:
        findings = collect_findings(api, repo, manager_repo)
    except (subprocess.SubprocessError, ValueError, KeyError) as error:
        findings = [
            f"Maintenance check could not complete ({type(error).__name__}). Check GitHub API availability and read permissions."
        ]
    return {
        "notify": bool(findings) and due and not dry_run,
        "reason": reason if findings else "No pending maintenance found.",
        "findings": findings,
        "dry_run": dry_run,
    }


def render_report(report):
    lines = [
        "# CCU weekly maintenance reminder",
        "",
        "Updates are manual. This check does not modify issues, branches, or releases, "
        "and never starts a build or conflict resolver.",
        "",
        report["reason"],
        "",
    ]
    lines.extend(f"- {finding}" for finding in report["findings"])
    if report["dry_run"]:
        lines.extend(["", "Preview only: no maintenance notification requested."])
    lines.extend(
        [
            "",
            "When ready, run `CCU i18n release` in the fork, then run "
            "`Sync fork release channel` in the manager after the fork release succeeds.",
            "",
            "A due reminder intentionally marks only this weekly workflow as failed, "
            "using your existing GitHub Actions failure-email settings. This means "
            "maintenance is pending, not that an automatic upgrade was attempted. "
            "Manual release and CI failures retain their normal notifications.",
        ]
    )
    return "\n".join(lines) + "\n"


def main():
    dry_run = os.environ.get("CCU_REMINDER_DRY_RUN", "true") == "true"
    report = evaluate(
        github_api,
        os.environ.get("GITHUB_REPOSITORY", "Cec1c/codex"),
        "Cec1c/codex-cli-ultra",
        dt.datetime.now(dt.timezone.utc),
        os.environ.get("GITHUB_RUN_ID", "local"),
        int(os.environ.get("GITHUB_RUN_ATTEMPT", "1")),
        dry_run,
    )
    summary = render_report(report)
    print(summary)
    if summary_path := os.environ.get("GITHUB_STEP_SUMMARY"):
        with Path(summary_path).open("a", encoding="utf-8") as stream:
            stream.write(summary)
    if report["notify"]:
        print(
            "::error::Weekly maintenance reminder: see the run summary. Automatic updates are disabled."
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
