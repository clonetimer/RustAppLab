#!/usr/bin/env python3
"""D9-A fixed-toolchain diagnostic pairs, learner red/green, and real CLI replay."""
from __future__ import annotations

import argparse
import json
import os
import platform
from pathlib import Path
import shutil
import subprocess
import tempfile

from modern_d8_check import Evidence, CARGO, RUSTC, TOOLCHAIN, ROOT, digest, write

LAB = ROOT / "projects/modern-d9-stable"
DIAGNOSTICS = {"gat":"E0499","capture":"E0502","upcasting":"E0308","async_trait":"E0038","async_closure":"captured variable cannot escape","guard":"E0004","never":"E0308","allocator":"E0277"}


def snapshot():
    files = [p for p in LAB.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB).parts)]
    files.extend([ROOT / "course/BOOK_D9_STABLE_FEATURES.md",
                  ROOT / "course/MODERN_STABLE_FEATURE_INDEX.md",
                  ROOT / "scripts/modern_d8_check.py", Path(__file__).resolve()])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(files)}


def experiments(evidence, scratch):
    evidence.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
    evidence.run("cargo-version", CARGO + ["--version"], scratch, contains="cargo 1.100.0 ")
    reference, learner = scratch / "reference", scratch / "learner"
    for dest in (reference, learner):
        shutil.copytree(LAB / dest.name, dest, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    common = ["--locked", "--offline"]
    evidence.run("reference-debug", CARGO + ["test"] + common + ["--all-targets"], reference,
                 contains="9 passed")
    evidence.run("reference-release", CARGO + ["test"] + common + ["--release", "--all-targets"],
                 reference, contains="9 passed")
    contract = learner / "tests/contracts.rs"
    contract_digest = digest(contract)
    if contract_digest != digest(reference / "tests/contracts.rs"):
        raise RuntimeError("learner/reference contract mismatch")
    for number in range(1, 10):
        test = f"step{number}_"
        evidence.run(f"learner-red-{number}", CARGO + ["test"] + common +
                     ["--test", "contracts", test], learner, expected=101,
                     contains="test result: FAILED")
    shutil.copyfile(reference / "src/lib.rs", learner / "src/lib.rs")
    evidence.run("learner-green", CARGO + ["test"] + common + ["--all-targets"],
                 learner, contains="9 passed")
    if digest(contract) != contract_digest:
        raise RuntimeError("fixed contract changed")
    for name, args, code, stdout, stderr in (
        ("success", ["7", "0", "3"], 0, "accepted=2; stored=[100, 7, 0, 3]", None),
        ("rollback", ["7", "bad", "3"], 1, None, "batch rejected; retained=[100]"),
        ("empty", [], 1, None, "usage: batch"),
        ("overflow", ["7", "4294967296"], 1, None, "batch rejected; retained=[100]"),
    ):
        evidence.run("cli-" + name, CARGO + ["run"] + common + ["--example", "batch", "--"] + args,
                     learner, expected=code, contains=stdout, stderr=stderr)
    for name, diagnostic in DIAGNOSTICS.items():
        red = reference / "probes" / name / "red.rs"
        evidence.run("diagnostic-" + name, RUSTC + ["--edition=2024", "--emit=metadata",
                     str(red), "-o", str(scratch / (name + ".rmeta"))],
                     scratch, expected=1, stderr=diagnostic)
    for name in (*DIAGNOSTICS, "drop_guard"):
        green = reference / "probes" / name / "green.rs"
        exe = scratch / (name + ("-green.exe" if os.name == "nt" else "-green"))
        evidence.run("repair-build-" + name, RUSTC + ["--edition=2024", str(green),
                     "-o", str(exe)], scratch)
        evidence.run("repair-run-" + name, [exe], scratch, contains="ok")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(schema=1, stage="D9-A", status="FAIL", toolchain=TOOLCHAIN,
                  git_commit=os.environ.get("GITHUB_SHA"), system=platform.system(),
                  source_before=before, learner_attainment="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-d9-") as temp:
            scratch = Path(temp)
            evidence = Evidence(output, scratch)
            report["steps"] = evidence.steps
            experiments(evidence, scratch)
        if snapshot() != before:
            raise RuntimeError("teaching source changed")
        report["status"] = "PASS"
    except (OSError, RuntimeError, subprocess.SubprocessError, ValueError) as error:
        report["error"] = str(error)
        print(error, flush=True)
    report["source_after"] = snapshot()
    report["source_unchanged"] = before == report["source_after"]
    write(output / "report.json", json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    return 0 if report["status"] == "PASS" and report["source_unchanged"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
