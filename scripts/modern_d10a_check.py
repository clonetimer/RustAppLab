#!/usr/bin/env python3
"""D10-A actual fixed-toolchain contracts, diagnostic pairs and CLI replay."""
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

LAB = ROOT / "projects/modern-d10-async-core"
DIAGNOSTICS = {"pin_unpin":"E0277","heterogeneous":"E0308","borrowed_task":"E0373","non_send":"future cannot be sent between threads safely"}


def snapshot():
    files = [p for p in LAB.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB).parts)]
    files.extend([ROOT / "course/BOOK_D10_ASYNC_CORE.md",
                  ROOT / "scripts/modern_d8_check.py", Path(__file__).resolve()])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(files)}


def experiments(evidence, scratch):
    evidence.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
    evidence.run("cargo-version", CARGO + ["--version"], scratch, contains="cargo 1.100.0 ")
    reference, learner = scratch / "reference", scratch / "learner"
    for dest in (reference, learner):
        shutil.copytree(LAB / dest.name, dest, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    common = ["--locked", "--offline"]
    evidence.run("reference-debug", CARGO + ["test"] + common + ["--all-targets"],
                 reference, contains="6 passed")
    evidence.run("reference-release", CARGO + ["test"] + common + ["--release", "--all-targets"],
                 reference, contains="6 passed")
    protected = ("tests/contracts.rs", "src/runtime.rs")
    before = {p: digest(learner / p) for p in protected}
    if any(digest(reference / p) != before[p] for p in protected):
        raise RuntimeError("fixed contracts or runtime differ")
    for number in range(1, 7):
        evidence.run(f"learner-red-{number}", CARGO + ["test"] + common +
                     ["--test", "contracts", f"step{number}_"], learner,
                     expected=101, contains="test result: FAILED")
    shutil.copyfile(reference / "src/lib.rs", learner / "src/lib.rs")
    evidence.run("learner-green", CARGO + ["test"] + common + ["--all-targets"],
                 learner, contains="6 passed")
    if any(digest(learner / p) != before[p] for p in protected):
        raise RuntimeError("fixed learner files changed")
    output, _ = evidence.run("cli", CARGO + ["run"] + common +
                             ["--example", "async_workbench"], learner)
    expected = ["joined=(7, 9); polls=2", "ordered=[8, 3, 5]",
                "race=Right(2)", "signal=Some(42)", "pinned=31"]
    if output.strip().splitlines() != expected:
        raise RuntimeError("actual CLI output differs: " + output)
    for name, diagnostic in DIAGNOSTICS.items():
        red = reference / "probes" / name / "red.rs"
        evidence.run("diagnostic-" + name, RUSTC + ["--edition=2024", "--emit=metadata",
                     str(red), "-o", str(scratch / (name + ".rmeta"))],
                     scratch, expected=1, stderr=diagnostic)
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
    report = dict(schema=1, stage="D10-A", status="FAIL", toolchain=TOOLCHAIN,
                  git_commit=os.environ.get("GITHUB_SHA"), system=platform.system(),
                  source_before=before, learner_attainment="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-d10a-") as temp:
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
