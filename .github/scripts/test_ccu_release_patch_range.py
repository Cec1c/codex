import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


spec = importlib.util.spec_from_file_location(
    "ccu_release_patch_range", Path(__file__).with_name("ccu-release-patch-range.py")
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ReleasePatchRangeTests(unittest.TestCase):
    def setUp(self):
        self.previous_cwd = Path.cwd()
        self.directory = tempfile.TemporaryDirectory()
        os.chdir(self.directory.name)
        self.addCleanup(self.cleanup)
        self.git("init", "-b", "main")
        self.git("config", "user.name", "CCU test")
        self.git("config", "user.email", "ccu-test@example.invalid")
        self.git("config", "core.autocrlf", "false")
        self.git("config", "commit.gpgsign", "false")
        self.base = self.commit("codex-rs/tui.rs", "upstream\n")

    def cleanup(self):
        os.chdir(self.previous_cwd)
        self.directory.cleanup()

    def git(self, *args):
        return subprocess.check_output(
            ["git", *args], text=True, stderr=subprocess.PIPE
        ).strip()

    def commit(self, path, contents):
        target = Path(path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(contents, encoding="utf-8")
        self.git("add", "--", path)
        self.git("commit", "-m", "test change")
        return self.git("rev-parse", "HEAD")

    def test_excludes_release_only_hotfixes_and_preserves_patch_order(self):
        self.git("switch", "-c", "upstream-release")
        hotfix = self.commit("codex-rs/tui.rs", "upstream hotfix\n")
        self.git("tag", "rust-v0.153.4", hotfix)
        first = self.commit("codex-rs/tui.rs", "upstream hotfix\nCCU i18n\n")
        second = self.commit("codex-rs/update.rs", "quick update\n")
        self.git("tag", "ccu-rust-v0.153.4-r1", second)
        self.assertEqual(
            module.release_patch_commits("rust-v0.153.4", "ccu-rust-v0.153.4-r1"),
            [first, second],
        )

    def test_rejects_unrelated_base(self):
        source = self.commit("codex-rs/tui.rs", "CCU\n")
        self.git("switch", "-c", "other", self.base)
        unrelated = self.commit("codex-rs/other.rs", "other upstream\n")
        with self.assertRaisesRegex(ValueError, "not based on"):
            module.release_patch_commits(unrelated, source)

    def test_rejects_empty_patch(self):
        with self.assertRaisesRegex(ValueError, "no release patches"):
            module.release_patch_commits(self.base, self.base)

    def test_rejects_disallowed_intermediate_change_even_if_reverted(self):
        self.commit("README.md", "unexpected change\n")
        self.git("rm", "README.md")
        self.git("commit", "-m", "revert disallowed path")
        source = self.commit("codex-rs/tui.rs", "CCU\n")
        with self.assertRaisesRegex(ValueError, "outside the release allowlist"):
            module.release_patch_commits(self.base, source)

    def test_rejects_packaging_only_patch(self):
        source = self.commit(".github/scripts/ccu-package-release.ps1", "package\n")
        with self.assertRaisesRegex(ValueError, "no codex-rs changes"):
            module.release_patch_commits(self.base, source)

    def test_rejects_merge_history(self):
        self.git("switch", "-c", "topic")
        self.commit("codex-rs/other.rs", "feature\n")
        self.git("switch", "main")
        self.commit("codex-rs/tui.rs", "CCU\n")
        self.git("merge", "--no-ff", "topic", "-m", "merge topic")
        with self.assertRaisesRegex(ValueError, "only linear"):
            module.release_patch_commits(self.base, "HEAD")


if __name__ == "__main__":
    unittest.main()
