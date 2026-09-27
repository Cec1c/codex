import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location(
    "ccu_release_lock", Path(__file__).with_name("ccu-sync-release-lock.py")
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ReleaseLockTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        offline = patch.dict(os.environ, {"CARGO_NET_OFFLINE": "true"})
        offline.start()
        self.addCleanup(offline.stop)
        self.manifest = self.root / "Cargo.toml"
        self.manifest.write_text(
            '[workspace]\nmembers = ["app", "pinned"]\nresolver = "2"\n'
            '[workspace.package]\nversion = "0.157.0"\n',
            encoding="utf-8",
        )
        for name, version in [("app", "version.workspace = true"),
                              ("pinned", 'version = "1.2.3"')]:
            crate = self.root / name
            (crate / "src").mkdir(parents=True)
            (crate / "src/lib.rs").write_text("", encoding="utf-8")
            (crate / "Cargo.toml").write_text(
                f'[package]\nname = "{name}"\n{version}\nedition = "2021"\n',
                encoding="utf-8",
            )
        subprocess.run(
            ["cargo", "generate-lockfile", "--manifest-path", str(self.manifest)],
            check=True,
        )
        self.lock = self.root / "Cargo.lock"
        self.original = self.lock.read_bytes()

    def bump_version(self, old, new):
        self.manifest.write_text(
            self.manifest.read_text(encoding="utf-8").replace(old, new),
            encoding="utf-8",
        )

    def test_repairs_successive_upstream_bumps_and_preserves_pinned_package(self):
        expected = self.original
        for old, new in [("0.157.0", "0.157.1"), ("0.157.1", "0.157.2")]:
            with self.subTest(version=new):
                self.bump_version(old, new)
                module.sync_release_lock(self.root)
                expected = expected.replace(old.encode(), new.encode())
                self.assertEqual(self.lock.read_bytes(), expected)
                module.sync_release_lock(self.root, check=True)

    def test_prepared_branch_check_rejects_stale_lock_without_modifying_it(self):
        self.bump_version("0.157.0", "0.157.1")
        with self.assertRaises(subprocess.CalledProcessError):
            module.sync_release_lock(self.root, check=True)
        self.assertEqual(self.lock.read_bytes(), self.original)

    def test_repeated_sync_is_a_noop(self):
        module.sync_release_lock(self.root)
        module.sync_release_lock(self.root, check=True)
        self.assertEqual(self.lock.read_bytes(), self.original)

    def test_invalid_manifest_fails_without_rewriting_lock(self):
        self.manifest.write_text("[workspace\n", encoding="utf-8")
        with self.assertRaises(subprocess.CalledProcessError):
            module.sync_release_lock(self.root)
        self.assertEqual(self.lock.read_bytes(), self.original)


if __name__ == "__main__":
    unittest.main()
