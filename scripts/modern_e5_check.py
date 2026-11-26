#!/usr/bin/env python3
"""Actual CPU science/ML and isolated learner replay; no accuracy claims beyond fixtures."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
LAB = ROOT / "projects/modern-e5-science"
CARGO = ["cargo", "+1.100.0"]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def snapshot():
    return {p.relative_to(LAB).as_posix(): digest(p) for p in LAB.rglob("*")
            if p.is_file() and not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB).parts)}

def check(output):
    output.mkdir(parents=True, exist_ok=False)
    report = dict(stage="E5", status="FAIL", backend="CPU f64", system=platform.system(),
                  machine=platform.machine(), git_commit=os.environ.get("GITHUB_SHA"),
                  human_learning="not evaluated", data="synthetic event groups", steps=[])
    before = snapshot()
    def run(label, command, cwd, env, code=0, contains=None):
        p = subprocess.run(command, cwd=cwd, env=env, capture_output=True,
                           text=True, encoding="utf-8", errors="replace", timeout=180)
        combined = p.stdout + "\n" + p.stderr
        (output / (label + ".txt")).write_text(combined, encoding="utf-8")
        report["steps"].append(dict(name=label, returncode=p.returncode))
        if p.returncode != code or (contains and contains not in combined):
            raise RuntimeError(label + " failed: " + combined[-2000:])
        print(label, "PASS", flush=True)
        return p.stdout
    def verify_demo(stdout):
        fields = dict(line.split("=", 1) for line in stdout.splitlines() if "=" in line)
        assert fields["backend"] == "cpu-f64"
        assert [int(fields[k]) for k in ("train_groups", "test_groups", "train_windows", "test_windows")] == [16, 8, 48, 24]
        loss0, loss1 = float(fields["loss_before"]), float(fields["loss_after"])
        assert abs(loss0 - math.log(2)) < 1e-8 and 0 <= loss1 < loss0
        tp, tn, fp, fn = (int(fields[k]) for k in ("tp", "tn", "fp", "fn"))
        assert tp + fn == tn + fp == 12
        accuracy = float(fields["accuracy"])
        assert abs(accuracy - (tp + tn) / 24) < 1e-10 and accuracy >= 0.9
        n, elapsed = int(fields["inference_count"]), int(fields["elapsed_ns"])
        assert n == 48000 and elapsed > 0
        assert 0 < float(fields["checksum"]) < n
        return fields
    try:
        archive = output / "E5-learner-offline.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as z:
            for p in sorted((LAB / "learner").rglob("*")):
                if p.is_file() and not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB / "learner").parts):
                    z.write(p, p.relative_to(LAB / "learner").as_posix())
        archive_hash = digest(archive)
        (output / "E5-learner-offline.zip.sha256").write_text(archive_hash + "  E5-learner-offline.zip\n", encoding="utf-8")
        with tempfile.TemporaryDirectory(prefix="e5-") as temp:
            scratch = Path(temp)
            ref, learner = scratch / "reference", scratch / "搬移 学员"
            shutil.copytree(LAB / "reference", ref, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
            with zipfile.ZipFile(archive) as z:
                assert z.testzip() is None
                assert z.read("src/lib.rs") == (LAB / "learner/src/lib.rs").read_bytes()
                z.extractall(learner)
            fixed = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src/synthetic.rs", "tests/contracts.rs", "examples/faultscope.rs"]
            fixed_hashes = {p: digest(ref / p) for p in fixed}
            assert fixed_hashes == {p: digest(learner / p) for p in fixed}
            for label, work in (("reference", ref), ("learner", learner)):
                home = scratch / (label + "-cargo")
                home.mkdir()
                assert not list(home.iterdir())
                env = os.environ.copy()
                env.update(CARGO_HOME=str(home), CARGO_NET_OFFLINE="true",
                           CARGO_TARGET_DIR=str(scratch / (label + "-target")))
                run(label + "-compiler", ["rustc", "+1.100.0", "--version"], work, env, contains="rustc 1.100.0")
                if label == "learner":
                    run("learner-compiles", CARGO + ["check", "--frozen", "--all-targets"], work, env)
                    for step in range(1, 5):
                        run(f"learner-red-{step}", CARGO + ["test", "--frozen", "--test", "contracts", f"step{step}_"], work, env, 101, "test result: FAILED")
                    shutil.copyfile(ref / "src/lib.rs", learner / "src/lib.rs")
                for mode, flags in (("debug", []), ("release", ["--release"])):
                    run(label + "-" + mode, CARGO + ["test", "--frozen", "--all-targets"] + flags,
                        work, env, contains="5 passed; 0 failed")
                report[label] = verify_demo(run(label + "-demo", CARGO + ["run", "--frozen", "--release", "--example", "faultscope"], work, env))
                assert fixed_hashes == {p: digest(work / p) for p in fixed}
                registry = home / "registry"
                assert not registry.exists() or all(p.name == "CACHEDIR.TAG" for p in registry.rglob("*") if p.is_file())
            # Timings may differ; compare actual semantic results and numerical losses.
            for key in ("train_groups", "test_groups", "tp", "tn", "fp", "fn", "accuracy", "inference_count"):
                assert report["reference"][key] == report["learner"][key]
            for key in ("loss_before", "loss_after", "checksum"):
                assert math.isclose(float(report["reference"][key]), float(report["learner"][key]), rel_tol=1e-9)
        assert digest(archive) == archive_hash and snapshot() == before
        report.update(status="PASS", learner_zip_sha256=archive_hash, source_unchanged=True)
    except (OSError, ValueError, RuntimeError, AssertionError, KeyError, subprocess.SubprocessError) as error:
        report["error"] = repr(error)
        print("FAIL", repr(error), flush=True)
    finally:
        (output / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return 0 if report["status"] == "PASS" else 1

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raise SystemExit(check(args.output.resolve()))
