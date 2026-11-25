#!/usr/bin/env python3
"""D11-A fixed-contract replay, actual tools, mutation checks, measurement and GDB."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile

from modern_d8_check import Evidence, CARGO, RUSTC, TOOLCHAIN, ROOT, digest, write

LAB = ROOT / "projects/modern-d11-quality"


def snapshot():
    files = [p for p in LAB.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB).parts)]
    files.extend([ROOT / "course/BOOK_D11_QUALITY.md",
                  ROOT / "scripts/modern_d8_check.py", Path(__file__).resolve()])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(files)}


def copy(side, scratch):
    dest = scratch / side
    shutil.copytree(LAB / side, dest,
                    ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    return dest


def mutate(original, old, new):
    if original.count(old) != 1:
        raise RuntimeError("mutation target not unique")
    return original.replace(old, new, 1)


def experiments(evidence, scratch, report, with_gdb):
    evidence.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
    evidence.run("cargo-version", CARGO + ["--version"], scratch, contains="cargo 1.100.0 ")
    common = ["--locked", "--offline"]
    reference = copy("reference", scratch)
    learner = copy("learner", scratch)
    practice = copy("tools-practice", scratch)
    for name in ("contracts.rs", "properties.rs"):
        if digest(reference / "tests" / name) != digest(learner / "tests" / name):
            raise RuntimeError("fixed tests differ: " + name)
    for mode, args in (("debug", []), ("release", ["--release"])):
        output, _ = evidence.run("reference-" + mode, CARGO + ["test"] + common + args +
                                 ["--all-targets"], reference)
        for count in (4, 3):
            if f"{count} passed; 0 failed; 0 ignored" not in output:
                raise RuntimeError("missing full contract/property suite")
    evidence.run("learner-compiles", CARGO + ["check"] + common + ["--all-targets"], learner)
    for number in range(1, 5):
        evidence.run(f"learner-red-{number}", CARGO + ["test"] + common +
                     ["--test", "contracts", f"step{number}_"], learner,
                     expected=101, contains="test result: FAILED")
    protected = {name: digest(learner / "tests" / name) for name in ("contracts.rs", "properties.rs")}
    shutil.copyfile(reference / "src/lib.rs", learner / "src/lib.rs")
    evidence.run("learner-green-contracts", CARGO + ["test"] + common +
                 ["--test", "contracts"], learner, contains="4 passed; 0 failed; 0 ignored")
    evidence.run("learner-green-properties", CARGO + ["test"] + common +
                 ["--test", "properties"], learner, contains="3 passed; 0 failed; 0 ignored")
    if any(digest(learner / "tests" / name) != value for name, value in protected.items()):
        raise RuntimeError("fixed tests modified")
    evidence.run("cli-valid", CARGO + ["run"] + common + ["--example", "frame", "--", "温度"],
                 learner, contains="decoded_bytes=6; text=温度")
    evidence.run("cli-corrupt", CARGO + ["run"] + common +
                 ["--example", "frame", "--", "温度", "corrupt"], learner,
                 expected=1, stderr="decode rejected: Checksum")

    # Independently compiled mutants must be rejected by an actual property assertion,
    # not by a compilation error. Only the temporary reference copy is changed.
    source = reference / "src/lib.rs"
    original = source.read_text(encoding="utf-8")
    mutants = (
        ("checksum", "if frame[2 + length] != checksum(payload) {",
         "if false {", "roundtrip_and_single_bit_faults"),
        ("trailing", "if frame.len() != length + 3 {",
         "if frame.len() < length + 3 {", "roundtrip_and_single_bit_faults"),
        ("threshold", "if value >= threshold {",
         "if value > threshold {", "implementations_agree_with_histogram_oracle"),
    )
    try:
        for name, old, new, test in mutants:
            write(source, mutate(original, old, new))
            evidence.run("mutant-" + name, CARGO + ["test"] + common +
                         ["--test", "properties", test], reference,
                         expected=101, contains="test result: FAILED")
    finally:
        write(source, original)
    evidence.run("properties-restored", CARGO + ["test"] + common +
                 ["--test", "properties"], reference, contains="3 passed; 0 failed; 0 ignored")

    evidence.run("rustfmt-version", CARGO + ["fmt", "--version"], practice)
    evidence.run("clippy-version", CARGO + ["clippy", "--version"], practice)
    evidence.run("format-red", CARGO + ["fmt", "--", "--check"], practice,
                 expected=1, contains="Diff")
    evidence.run("format-repair", CARGO + ["fmt"], practice)
    evidence.run("format-green", CARGO + ["fmt", "--", "--check"], practice)
    clippy = CARGO + ["clippy"] + common + ["--all-targets", "--", "-D", "warnings"]
    evidence.run("clippy-red", clippy, practice, expected=101, stderr="needless_return")
    file = practice / "src/lib.rs"
    fixed = mutate(file.read_text(encoding="utf-8"), "return value > 10;", "value > 10")
    write(file, fixed)
    evidence.run("clippy-green", clippy, practice)
    evidence.run("tools-behavior", CARGO + ["test"] + common, practice,
                 contains="1 passed; 0 failed; 0 ignored")
    evidence.run("reference-clippy", CARGO + ["clippy"] + common + ["--lib", "--", "-D", "warnings"],
                 reference)

    output, _ = evidence.run("release-measure", CARGO + ["run"] + common +
                             ["--release", "--example", "measure", "--", "1000"], learner)
    lines = output.strip().splitlines()
    if len(lines) != 3 or lines[0] != "size=4096; rounds=1000; samples=5; result=2064000":
        raise RuntimeError("measurement contract differs")
    timings = {}
    for line, key in zip(lines[1:], ("loop_median_ns", "iterator_median_ns")):
        match = re.fullmatch(key + r"=(\d+)", line)
        if not match or int(match.group(1)) <= 0:
            raise RuntimeError("missing positive measurement: " + line)
        timings[key] = int(match.group(1))
    report["measurement"] = dict(size=4096, rounds=1000, samples=5, **timings,
                                 interpretation="observed runner only; no speedup assertion")

    report["debugger"] = dict(status="NOT_RUN", reason="interactive IDE not exercised")
    if with_gdb:
        evidence.run("gdb-version", ["gdb", "--version"], learner, contains="GNU gdb")
        output, _ = evidence.run("debug-build-json", CARGO + ["build"] + common +
                                 ["--example", "debug_probe", "--message-format=json"], learner)
        executable = None
        for line in output.splitlines():
            message = json.loads(line)
            if message.get("reason") == "compiler-artifact" and message.get("target", {}).get("name") == "debug_probe":
                executable = message.get("executable")
        if not executable or not Path(executable).is_file():
            raise RuntimeError("no actual Cargo debugger executable")
        output, _ = evidence.run("gdb-breakpoint", ["gdb", "--batch", "--nx",
                                 "-ex", "set pagination off",
                                 "-ex", "break debug_probe::diagnose_frame",
                                 "-ex", "run", "-ex", "info args", "-ex", "bt",
                                 "-ex", "continue", "--args", executable],
                                 learner)
        if not all(re.search(pattern, output) for pattern in (
                r"Breakpoint 1, .*diagnose_frame", r"expected\s*=\s*42",
                r"actual\s*=\s*43", r"#0\s+.*diagnose_frame", r"checksum_matches=false")):
            raise RuntimeError("debugger did not actually stop with expected arguments/stack")
        report["debugger"] = dict(status="PASS", mode="GDB batch breakpoint, arguments, stack",
                                  interactive_ide="NOT_RUN")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--gdb", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(schema=1, stage="D11-A", status="FAIL", toolchain=TOOLCHAIN,
                  git_commit=os.environ.get("GITHUB_SHA"), system=platform.system(),
                  source_before=before, learner_attainment="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-d11a-") as temp:
            scratch = Path(temp)
            evidence = Evidence(output, scratch)
            report["steps"] = evidence.steps
            experiments(evidence, scratch, report, args.gdb)
        if snapshot() != before:
            raise RuntimeError("teaching source changed")
        report["status"] = "PASS"
    except (OSError, RuntimeError, subprocess.SubprocessError, ValueError) as error:
        report["error"] = str(error)
        print(error, flush=True)
    report["source_after"] = snapshot()
    report["source_unchanged"] = before == report["source_after"]
    report["files"] = {p.name: digest(p) for p in sorted(output.iterdir()) if p.is_file()}
    write(output / "report.json", json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    return 0 if report["status"] == "PASS" and report["source_unchanged"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
