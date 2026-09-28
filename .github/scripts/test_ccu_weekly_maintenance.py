import base64
import datetime as dt
import importlib.util
import json
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    "maintenance", Path(__file__).with_name("ccu-weekly-maintenance.py")
)
maintenance = importlib.util.module_from_spec(spec)
spec.loader.exec_module(maintenance)
NOW = dt.datetime(2026, 9, 28, 2, 17, tzinfo=dt.timezone.utc)


def run(run_id=1, conclusion="failure", ended=None):
    return {
        "id": run_id,
        "status": "completed",
        "conclusion": conclusion,
        "updated_at": (ended or NOW - dt.timedelta(days=1)).isoformat(),
        "html_url": f"https://github.com/test/runs/{run_id}",
    }


class MaintenanceTests(unittest.TestCase):
    def test_cooldown_boundary_and_current_run(self):
        for elapsed, expected in [
            (dt.timedelta(days=7) - dt.timedelta(seconds=1), False),
            (dt.timedelta(days=7), True),
            (dt.timedelta(days=8), True),
        ]:
            with self.subTest(elapsed=elapsed):
                history = [run(ended=NOW - elapsed), run(run_id=99)]
                self.assertEqual(
                    maintenance.reminder_due(history, NOW, 99, 1)[0], expected
                )

    def test_successful_previews_do_not_delay_a_reminder(self):
        self.assertTrue(
            maintenance.reminder_due([run(conclusion="success")], NOW, 99, 1)[0]
        )

    def test_rerun_never_reads_or_notifies_even_when_history_is_unavailable(self):
        def fail_api(path):
            self.fail(f"Rerun unexpectedly accessed {path}")

        report = maintenance.evaluate(
            fail_api, "test/fork", "test/manager", NOW, 99, 2, False
        )
        self.assertFalse(report["notify"])

    def test_cooldown_skips_all_release_checks(self):
        paths = []

        def api(path):
            paths.append(path)
            return {"workflow_runs": [run()]}

        report = maintenance.evaluate(
            api, "test/fork", "test/manager", NOW, 99, 1, False
        )
        self.assertFalse(report["notify"])
        self.assertEqual(len(paths), 1)

    def test_unavailable_history_suppresses_duplicate_notifications(self):
        def api(path):
            raise subprocess.CalledProcessError(1, ["gh", "api", path])

        report = maintenance.evaluate(
            api, "test/fork", "test/manager", NOW, 99, 1, False
        )
        self.assertFalse(report["notify"])
        self.assertIn("unable to verify", report["reason"])

    def test_successful_rerun_does_not_erase_original_failure_cooldown(self):
        def api(path):
            if "/attempts/1" in path:
                return run()
            return {"workflow_runs": [run(conclusion="success") | {"run_attempt": 2}]}

        report = maintenance.evaluate(
            api, "test/fork", "test/manager", NOW, 99, 1, False
        )
        self.assertFalse(report["notify"])

    def test_snapshot_aggregates_releases_conflicts_and_latest_failures(self):
        def api(path):
            if path == "repos/openai/codex/releases/latest":
                return {
                    "tag_name": "rust-v0.158.0",
                    "html_url": "https://upstream/release",
                }
            if path.endswith("/releases/latest"):
                return {"tag_name": "ccu-rust-v0.157.1-r1"}
            if "/contents/" in path:
                content = json.dumps(
                    {"release": {"releaseTag": "ccu-rust-v0.157.0-r1"}}
                )
                return {"content": base64.b64encode(content.encode()).decode()}
            if "/issues?" in path:
                return [
                    {
                        "title": "[CCU sync] ccu-rust-v0.158.0-r1 needs manual rebase",
                        "html_url": "https://fork/issues/65",
                    },
                    {
                        "title": "[CCU sync] ccu-rust-v0.157.1-r1 needs manual rebase",
                        "html_url": "https://fork/issues/old",
                    },
                ]
            if "ccu-i18n-release.yml" in path:
                return {"workflow_runs": [run()]}
            return {"workflow_runs": [run(conclusion="success"), run()]}

        findings = maintenance.collect_findings(api, "test/fork", "test/manager")
        self.assertEqual(len(findings), 4)
        self.assertIn("rust-v0.158.0", findings[0])
        self.assertIn("0.157.0-r1", findings[1])
        self.assertIn("https://fork/issues/65", findings[2])
        self.assertNotIn("https://fork/issues/old", findings[2])
        self.assertIn("ccu-i18n-release.yml", findings[3])

    def test_history_pagination_preserves_cooldown_after_many_previews(self):
        paths = []

        def api(path):
            paths.append(path)
            runs = (
                [run(conclusion="success")] * 100
                if path.endswith("&page=1")
                else [run()]
            )
            return {"workflow_runs": runs}

        report = maintenance.evaluate(
            api, "test/fork", "test/manager", NOW, 99, 1, False
        )
        self.assertFalse(report["notify"])
        self.assertEqual(len(paths), 2)

    def test_no_pending_work_is_silent_and_preview_does_not_fail(self):
        for dry_run, findings, expected in [
            (False, [], False),
            (False, ["Pending update"], True),
            (True, ["Pending update"], False),
        ]:
            with self.subTest(dry_run=dry_run, findings=findings):
                with patch.object(
                    maintenance, "collect_findings", return_value=findings
                ):
                    report = maintenance.evaluate(
                        lambda path: {"workflow_runs": []},
                        "test/fork",
                        "test/manager",
                        NOW,
                        99,
                        1,
                        dry_run,
                    )
                self.assertEqual(report["notify"], expected)

    def test_collection_error_uses_only_the_due_reminder(self):
        with patch.object(
            maintenance, "collect_findings", side_effect=ValueError("bad tag")
        ):
            report = maintenance.evaluate(
                lambda path: {"workflow_runs": []},
                "test/fork",
                "test/manager",
                NOW,
                99,
                1,
                False,
            )
        self.assertTrue(report["notify"])
        self.assertEqual(len(report["findings"]), 1)
        self.assertIn("ValueError", report["findings"][0])

    def test_semantic_version_order_and_invalid_tag(self):
        self.assertGreater(
            maintenance.stable_version("rust-v0.100.0"),
            maintenance.stable_version("rust-v0.99.9"),
        )
        with self.assertRaises(ValueError):
            maintenance.stable_version("rust-v0.159.0-alpha.1")


if __name__ == "__main__":
    unittest.main()
