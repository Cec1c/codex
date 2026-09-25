import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location("runtime", Path(__file__).with_name("ccu-package-runtime.py"))
runtime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runtime)


class RuntimePackageTests(unittest.TestCase):
    def package(self, root, target, *, omit=None, extra=None):
        upstream = target.replace("-unknown-linux-gnu", "-unknown-linux-musl")
        suffix = ".exe" if "windows" in target else ""
        files = {
            "codex-package.json": json.dumps({"version": "0.157.0", "target": upstream,
                "variant": "codex", "entrypoint": f"bin/codex{suffix}"}).encode(),
            f"bin/codex{suffix}": b"official",
            f"codex-path/rg{suffix}": b"ripgrep",
            "codex-resources/voice/NOTICE.md": b"license retained",
        }
        if "windows" in target:
            files.update({"codex-resources/codex-command-runner.exe": b"runner",
                          "codex-resources/codex-windows-sandbox-setup.exe": b"setup"})
        if "linux" in target:
            files["codex-resources/bwrap"] = b"sandbox"
        files.pop(omit, None)
        files.update(extra or {})
        archive = root / f"codex-package-{upstream}.tar.gz"
        with tarfile.open(archive, "w:gz") as tar:
            for name, content in files.items():
                info = tarfile.TarInfo(name)
                info.size, info.mode = len(content), 0o755
                tar.addfile(info, io.BytesIO(content))
        sums = root / "SHA256SUMS"
        sums.write_text(f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n")
        binary, host = root / "fork", root / "host"
        binary.write_bytes(b"CCU")
        host.write_bytes(b"CCU host")
        return archive, sums, binary, host, root / "runtime.zip", "0.157.0", "0.157.0-ccu.i18n.1", target

    def test_five_platform_packages_keep_resources_and_executable_modes(self):
        for target in ("x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu",
                       "aarch64-unknown-linux-gnu", "x86_64-apple-darwin", "aarch64-apple-darwin"):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                args = self.package(Path(directory), target)
                runtime.assemble(*args)
                with zipfile.ZipFile(args[4]) as result:
                    suffix = ".exe" if "windows" in target else ""
                    self.assertEqual(result.read(f"package/bin/codex{suffix}"), b"CCU")
                    self.assertEqual(result.read("package/codex-resources/voice/NOTICE.md"), b"license retained")
                    metadata = json.loads(result.read("package/codex-package.json"))
                    self.assertEqual((metadata["version"], metadata["target"]), (args[6], target))
                    self.assertEqual(result.getinfo(f"package/codex-path/rg{suffix}").external_attr >> 16, 0o100755)

    def test_rejects_checksum_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.package(Path(directory), "x86_64-pc-windows-msvc")
            args[1].write_text("0" * 64 + "  " + args[0].name)
            with self.assertRaisesRegex(ValueError, "checksum"):
                runtime.assemble(*args)

    def test_rejects_missing_sandbox_helper(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.package(Path(directory), "x86_64-pc-windows-msvc",
                omit="codex-resources/codex-windows-sandbox-setup.exe")
            with self.assertRaisesRegex(ValueError, "Incomplete"):
                runtime.assemble(*args)

    def test_rejects_path_traversal(self):
        with tempfile.TemporaryDirectory() as directory:
            args = self.package(Path(directory), "x86_64-unknown-linux-gnu",
                extra={"codex-resources/../../escape": b"bad"})
            with self.assertRaisesRegex(ValueError, "Unsafe"):
                runtime.assemble(*args)


if __name__ == "__main__":
    unittest.main()
