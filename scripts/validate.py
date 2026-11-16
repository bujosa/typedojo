#!/usr/bin/env python3
"""Repository gates using the pinned app-owned runtime; no learner data or installed apps."""

import os
import tempfile
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
for command in [
    ["cargo", "fmt", "--check"],
    ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
    ["cargo", "test", "--locked"],
    ["cargo", "run", "--locked", "--", "--verify-completion"],
    ["cargo", "run", "--locked", "--", "--verify-art"],
    ["cargo", "run", "--locked", "--", "--verify-content"],
    ["cargo", "run", "--locked", "--", "--verify-learning"],
    ["target/toolchain/node/bin/node", "tools/format-content.mjs"],
]:
    print("Running:", " ".join(command), flush=True)
    env = os.environ.copy()
    env["TYPEDOJO_HOME"] = tempfile.mkdtemp(prefix="dojo-validation-")
    subprocess.run(command, env=env, cwd=ROOT, check=True)
print("All local gates passed.")
