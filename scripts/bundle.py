#!/usr/bin/env python3
"""Build a Keiko-compatible signed macOS release. Never touches installed applications."""

import hashlib
import os
import platform
import plistlib
import shutil
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
os.chdir(ROOT)


def run(*args, **kwargs):
    subprocess.run(args, check=True, **kwargs)


run("python3", "scripts/toolchain.py")
version = tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
env = os.environ.copy()
env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + f" --remap-path-prefix={Path.home()}=~"
run("cargo", "build", "--release", "--locked", env=env)
binary = Path("target/release/typedojo")
if str(Path.home()).encode() in binary.read_bytes():
    raise SystemExit("Release rejected: builder home path found in binary")
base = Path("target/bundle")
app = base / "TypeDojo.app"
if app.exists():
    shutil.rmtree(app)  # exclusively generated output inside this repository
macos = app / "Contents/MacOS"
resources = app / "Contents/Resources"
macos.mkdir(parents=True)
resources.mkdir()
licenses = resources / "Licenses"
licenses.mkdir()
for license_file in list(Path("assets/fonts").glob("OFL-*.txt")) + list(
    Path("assets/licenses").glob("*.txt")
):
    shutil.copy2(license_file, licenses / license_file.name)
shutil.copy2("ATTRIBUTION.md", resources / "ATTRIBUTION.md")
node = resources / "node/bin"
node.mkdir(parents=True)
shutil.copy2("target/toolchain/node/bin/node", node / "node")
shutil.copytree("target/toolchain/typescript", resources / "typescript")
shutil.copy2("target/toolchain/node/LICENSE", licenses / "LICENSE-Node.txt")
shutil.copy2(
    "target/toolchain/typescript/LICENSE.txt", licenses / "LICENSE-TypeScript.txt"
)
shutil.copy2("runtime/toolchain.json", resources / "toolchain.json")
run("codesign", "--force", "--sign", "-", str(node / "node"))
shutil.copy2(binary, macos / "typedojo")
info = dict(
    CFBundleName="TypeDojo",
    CFBundleDisplayName="TypeDojo",
    CFBundleIdentifier="dev.bujosa.typedojo",
    CFBundleVersion=version,
    CFBundleShortVersionString=version,
    CFBundleExecutable="typedojo",
    CFBundleIconFile="TypeDojo",
    CFBundlePackageType="APPL",
    LSApplicationCategoryType="public.app-category.education",
    LSMinimumSystemVersion="13.5",
    NSHighResolutionCapable=True,
)
(app / "Contents/Info.plist").write_bytes(plistlib.dumps(info))
run("swift", "scripts/icon.swift", str(base / "icon.png"))
iconset = base / "TypeDojo.iconset"
iconset.mkdir(exist_ok=True)
for size in (16, 32, 128, 256, 512):
    for scale in (1, 2):
        suffix = "@2x" if scale == 2 else ""
        run(
            "sips",
            "-z",
            str(size * scale),
            str(size * scale),
            str(base / "icon.png"),
            "--out",
            str(iconset / f"icon_{size}x{size}{suffix}.png"),
            stdout=subprocess.DEVNULL,
        )
run("iconutil", "-c", "icns", str(iconset), "-o", str(resources / "TypeDojo.icns"))
run("codesign", "--force", "--sign", "-", str(app))
run("codesign", "--verify", "--deep", "--strict", str(app))
out = Path("target/release-assets")
out.mkdir(exist_ok=True)
arch = {"aarch64": "arm64"}.get(platform.machine(), platform.machine())
archive = out / f"TypeDojo-v{version}-macos-{arch}.zip"
run("ditto", "-c", "-k", "--keepParent", str(app), str(archive))
checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
(out / "SHA256SUMS.txt").write_text(f"{checksum}  {archive.name}\n")
print(f"Prepared {archive} and SHA256SUMS.txt (ad-hoc signed, not notarized).")
