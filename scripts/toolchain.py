#!/usr/bin/env python3
"""Rebuild the generated toolchain from checksum-pinned upstream archives. No system installs."""

import hashlib
import json
import platform
import shutil
import subprocess
import tarfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise SystemExit("The bundled runtime currently targets Apple Silicon macOS.")
    base = ROOT / "target/toolchain"
    base.mkdir(parents=True, exist_ok=True)
    for name, spec in json.loads((ROOT / "runtime/toolchain.json").read_text()).items():
        archive = base / (name + ".tar.gz")
        if (
            not archive.exists()
            or hashlib.sha256(archive.read_bytes()).hexdigest() != spec["sha256"]
        ):
            pending = archive.with_suffix(".pending")
            with (
                urllib.request.urlopen(spec["url"], timeout=60) as response,
                pending.open("wb") as out,
            ):
                shutil.copyfileobj(response, out)
            if hashlib.sha256(pending.read_bytes()).hexdigest() != spec["sha256"]:
                pending.unlink()
                raise SystemExit("Checksum mismatch: " + name)
            pending.replace(archive)
        staging = base / (name + "-unpack")
        if staging.exists():
            shutil.rmtree(staging)
        staging.mkdir()
        with tarfile.open(archive) as tar:
            tar.extractall(staging, filter="data")
        unpacked = next(staging.iterdir())
        dest = base / name
        if dest.exists():
            shutil.rmtree(dest)
        unpacked.rename(dest)
        staging.rmdir()
    node = base / "node/bin/node"
    subprocess.run([str(node), "--version"], check=True)
    subprocess.run(
        [str(node), str(base / "typescript/lib/tsc.js"), "--version"], check=True
    )
    print("Verified bundled Node and TypeScript ready.")


if __name__ == "__main__":
    main()
