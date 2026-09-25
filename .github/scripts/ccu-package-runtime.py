"""Overlay CCU executables on the checksum-verified upstream runtime resources."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import tarfile
import zipfile


def assemble(
    archive, checksums, binary, host, output, upstream_version, version, target
):
    source_target = target.replace("-unknown-linux-gnu", "-unknown-linux-musl")
    expected_name = f"codex-package-{source_target}.tar.gz"
    if archive.name != expected_name:
        raise ValueError(f"Expected upstream archive {expected_name}")
    digests = [
        line.split()[0]
        for line in checksums.read_text().splitlines()
        if len(line.split()) == 2 and line.split()[1].lstrip("*") == archive.name
    ]
    with archive.open("rb") as stream:
        hasher = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
        digest = hasher.hexdigest()
    if digests != [digest]:
        raise ValueError("Upstream package checksum mismatch or missing checksum")
    windows = "windows" in target
    suffix = ".exe" if windows else ""
    required = {f"codex-path/rg{suffix}"}
    if windows:
        required.update(
            {
                "codex-resources/codex-command-runner.exe",
                "codex-resources/codex-windows-sandbox-setup.exe",
            }
        )
    elif "linux" in target:
        required.add("codex-resources/bwrap")

    with tarfile.open(archive, "r:gz") as source:
        resources = {}
        seen = set()
        for member in source.getmembers():
            path = PurePosixPath(member.name)
            if path.is_absolute() or ".." in path.parts or "\\" in member.name:
                raise ValueError(f"Unsafe upstream path: {member.name}")
            name = path.as_posix()
            if name.casefold() in seen:
                raise ValueError(f"Duplicate upstream path: {name}")
            seen.add(name.casefold())
            if path.parts and path.parts[0] in {"codex-resources", "codex-path"}:
                if member.isdir():
                    continue
                if not member.isfile():
                    raise ValueError(f"Unsupported upstream resource type: {name}")
                resources[name] = member
        metadata = json.load(source.extractfile("codex-package.json"))
        if (
            metadata.get("version"),
            metadata.get("target"),
            metadata.get("variant"),
        ) != (upstream_version, source_target, "codex"):
            raise ValueError(
                "Upstream package metadata does not match requested release"
            )
        if missing := required - resources.keys():
            raise ValueError(f"Incomplete upstream runtime: {sorted(missing)}")
        if not windows and any(not resources[name].mode & 0o111 for name in required):
            raise ValueError("Upstream runtime helper is not executable")
        metadata.update(version=version, target=target)
        with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as destination:

            def write(name, stream, executable):
                info = zipfile.ZipInfo(f"package/{name}")
                info.create_system = 3
                info.external_attr = (0o100755 if executable else 0o100644) << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                with destination.open(info, "w", force_zip64=True) as dest:
                    shutil.copyfileobj(stream, dest)

            for name, path in (
                (f"bin/codex{suffix}", binary),
                (f"bin/codex-code-mode-host{suffix}", host),
            ):
                with path.open("rb") as stream:
                    write(name, stream, True)
            for name, member in sorted(resources.items()):
                with source.extractfile(member) as stream:
                    write(name, stream, bool(member.mode & 0o111))
            destination.writestr(
                "package/codex-package.json", json.dumps(metadata, indent=2) + "\n"
            )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("archive", "checksums", "binary", "host", "output"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    for name in ("upstream-version", "version", "target"):
        parser.add_argument(f"--{name}", required=True)
    assemble(**vars(parser.parse_args()))
