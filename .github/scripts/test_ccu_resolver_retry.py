import datetime as dt
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("retry", Path(__file__).with_name("ccu-resolver-retry.py"))
retry = importlib.util.module_from_spec(spec)
spec.loader.exec_module(retry)
NOW = dt.datetime(2026, 9, 25, 12, tzinfo=dt.timezone.utc)


def run(**updates):
    return dict(display_title="CCU conflict resolver #64", status="completed",
                conclusion="failure", created_at="2026-09-25T00:00:00Z",
                updated_at="2026-09-25T01:00:00Z") | updates


class RetryTests(unittest.TestCase):
    def test_transient_failure_recovers_after_cooldown(self):
        self.assertTrue(retry.retry_decision([run()], 64, NOW)[0])

    def test_in_progress_legacy_job_blocks_duplicate(self):
        self.assertFalse(retry.retry_decision([run(display_title="CCU conflict resolver", status="in_progress")], 64, NOW)[0])

    def test_recent_failure_and_success_do_not_retry(self):
        for attempt in [run(updated_at="2026-09-25T11:00:00Z"), run(conclusion="success")]:
            self.assertFalse(retry.retry_decision([attempt], 64, NOW)[0])

    def test_attempt_cap_is_per_issue(self):
        self.assertFalse(retry.retry_decision([run()] * 3, 64, NOW)[0])
        self.assertTrue(retry.retry_decision([run()] * 3, 65, NOW)[0])


if __name__ == "__main__":
    unittest.main()
