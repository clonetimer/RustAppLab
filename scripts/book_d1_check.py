#!/usr/bin/env python3
"""Real offline Cargo extension installation smoke, isolated from the user's Cargo home."""
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "projects" / "book-d1-cargo" / "cargo-applab-map"


def run(args: list[str], env: dict[str, str], expect_ok: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, cwd=ROOT, env=env, text=True, encoding="utf-8", errors="replace",
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=120)
    if (result.returncode == 0) != expect_ok:
        raise AssertionError(f"unexpected return: {args!r}\n{result.stdout}\n{result.stderr}")
    return result


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="rustapplab-d1-") as folder:
        root = Path(folder)
        install = root / "install"
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(root / "target")
        run(["cargo", "install", "--locked", "--offline", "--path", str(PACKAGE),
             "--root", str(install)], env)
        executable = install / "bin" / ("cargo-applab-map.exe" if os.name == "nt" else "cargo-applab-map")
        if not executable.is_file():
            raise AssertionError("cargo install did not place the expected binary in isolated root")
        direct = run([str(executable), "profiles"], env)
        if not direct.stdout.startswith("14.1 "):
            raise AssertionError(f"direct invocation mismatch: {direct.stdout!r}")
        env["PATH"] = str(install / "bin") + os.pathsep + env.get("PATH", "")
        cargo = run(["cargo", "applab-map", "workspace"], env)
        if not cargo.stdout.startswith("14.3 "):
            raise AssertionError(f"Cargo extension not discovered: {cargo.stdout!r}")
        error = run(["cargo", "applab-map", "unknown"], env, expect_ok=False)
        if error.stdout or "未知主题" not in error.stderr:
            raise AssertionError("invalid Cargo extension invocation must fail without stdout")
        print("BOOK D1 INSTALL PASS: scoped root, direct binary, Cargo extension and error path")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
