#!/usr/bin/env python3
"""D11-B: real Miri, libFuzzer and RustSec evidence, isolated from stable learner source."""
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
import tomllib

from modern_d8_check import Evidence, CARGO, RUSTC, TOOLCHAIN, ROOT, digest, write

LAB = ROOT / "projects/modern-d11-safety"
QUALITY = ROOT / "projects/modern-d11-quality/reference"
NIGHTLY = "nightly-2026-11-17"
NCARGO = ["cargo", "+" + NIGHTLY]
FUZZ_VERSION = "0.13.2"
AUDIT_VERSION = "0.23.0"
DB_SHA = "b9b8a3c2cb1f2a3c76ab17a551193568c9f8b7a4"
ADVISORY = "RUSTSEC-2022-0013"
LOCKS = {
    "root": ROOT / "Cargo.lock",
    "c1": ROOT / "projects/advanced-c1/reference/Cargo.lock",
    "c2": ROOT / "projects/advanced-c2/reference/Cargo.lock",
    "d10b": ROOT / "projects/modern-d10-tokio/reference/Cargo.lock",
    "d11a": QUALITY / "Cargo.lock",
}


class Runner(Evidence):
    def execute(self, name, command, cwd, expected=(0,), timeout=240):
        command = [str(x) for x in command]
        result = subprocess.run(command, cwd=cwd, env=self.env, capture_output=True, timeout=timeout)
        out = result.stdout.decode("utf-8", errors="replace").replace("\r\n", "\n")
        err = result.stderr.decode("utf-8", errors="replace").replace("\r\n", "\n")
        path = self.output / (name + ".txt")
        write(path, json.dumps(command, ensure_ascii=False) + "\n" + out + "\n--- stderr ---\n" + err)
        passed = result.returncode in expected
        self.steps.append(dict(step=name, command=command, returncode=result.returncode,
                               expected_returncodes=list(expected), log=path.name,
                               sha256=digest(path), passed=passed))
        print(f"{name}: {'PASS' if passed else 'FAIL'} ({result.returncode})", flush=True)
        if not passed:
            raise RuntimeError(f"{name}: wrong exit status; see {path}")
        return out, err, result.returncode


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def snapshot():
    paths = [p for base in (LAB, QUALITY) for p in base.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__", "vendor", "corpus", "artifacts"}.intersection(p.relative_to(base).parts)]
    paths.extend(LOCKS.values())
    paths.extend([ROOT / "course/BOOK_D11_SAFETY.md",
                  ROOT / "scripts/modern_d8_check.py", Path(__file__).resolve()])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(set(paths))}


def copy(side, scratch):
    dest = scratch / side
    shutil.copytree(LAB / side, dest, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    return dest


def stable(e, scratch, report):
    e.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
    reference, learner = copy("reference", scratch), copy("learner", scratch)
    fixed = learner / "tests/contracts.rs"
    before = digest(fixed)
    require(before == digest(reference / "tests/contracts.rs"), "fixed contracts differ")
    for label, args in (("debug", []), ("release", ["--release"])):
        e.run("reference-" + label, CARGO + ["test", "--locked", "--offline"] + args,
              reference, contains="2 passed; 0 failed; 0 ignored")
    for i in (1, 2):
        e.run(f"learner-red-{i}", CARGO + ["test", "--locked", "--offline",
              "--test", "contracts", f"step{i}_"], learner, expected=101, contains="test result: FAILED")
    shutil.copyfile(reference / "src/lib.rs", learner / "src/lib.rs")
    e.run("learner-green", CARGO + ["test", "--locked", "--offline"],
          learner, contains="2 passed; 0 failed; 0 ignored")
    require(before == digest(fixed), "fixed contract was modified")


def miri(e, scratch, report):
    e.env["CARGO_NET_OFFLINE"] = "false"
    e.execute("nightly-install", ["rustup", "toolchain", "install", NIGHTLY, "--profile", "minimal",
              "--component", "miri", "--component", "rust-src"], scratch, timeout=900)
    e.run("nightly-version", ["rustc", "+" + NIGHTLY, "--version"], scratch, contains="-nightly")
    e.run("miri-version", NCARGO + ["miri", "--version"], scratch, contains="miri")
    ref = copy("reference", scratch)
    e.execute("miri-setup", NCARGO + ["miri", "setup"], ref, timeout=900)
    e.env["CARGO_NET_OFFLINE"] = "true"
    e.run("miri-fixed-contracts", NCARGO + ["miri", "test", "--locked", "--offline",
          "--test", "contracts"], ref, contains="2 passed; 0 failed; 0 ignored")
    examples = ref / "examples"
    examples.mkdir(exist_ok=True)
    command = NCARGO + ["miri", "run", "--locked", "--offline", "--example", "miri_probe"]
    cases = {"out_of_bounds": r"out.of.bounds|beyond the end|dangling pointer",
             "uninitialized": r"uninit|uninitialized"}
    for case, pattern in cases.items():
        target = examples / "miri_probe.rs"
        shutil.copyfile(LAB / "miri-practice/probes" / (case + ".rs"), target)
        out, err, _ = e.execute("miri-red-" + case, command, ref, expected=(1, 101))
        require("Undefined Behavior" in err and re.search(pattern, err, re.I),
                case + ": wrong Miri failure; not an actual matching UB diagnostic")
        shutil.copyfile(ref / "probes" / (case + ".rs"), target)
        e.run("miri-green-" + case, command, ref, contains="ok")
        target.unlink()
    report["miri"] = dict(negative_cases=2, corrected_cases=2, native_ub_executed=False,
                          toolchain=NIGHTLY, scope="Ubuntu; exercised paths only")


def install(e, scratch, name, version):
    e.env["CARGO_NET_OFFLINE"] = "false"
    prefix = scratch / "tools"
    e.execute(name + "-install", CARGO + ["install", name, "--version", version,
              "--locked", "--root", prefix], scratch, timeout=1500)
    e.env["PATH"] = str(prefix / "bin") + os.pathsep + e.env["PATH"]


def fuzz(e, scratch, report):
    e.env["CARGO_NET_OFFLINE"] = "false"
    e.execute("nightly-install", ["rustup", "toolchain", "install", NIGHTLY,
              "--profile", "minimal"], scratch, timeout=900)
    e.run("nightly-version", ["rustc", "+" + NIGHTLY, "--version"], scratch, contains="-nightly")
    install(e, scratch, "cargo-fuzz", FUZZ_VERSION)
    e.run("cargo-fuzz-version", NCARGO + ["fuzz", "--version"], scratch, contains=FUZZ_VERSION)
    work = scratch / "codec"
    shutil.copytree(QUALITY, work, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    shutil.copytree(LAB / "fuzz-practice/fuzz", work / "fuzz")
    manifest = work / "fuzz/Cargo.toml"
    e.execute("fuzz-lock", NCARGO + ["generate-lockfile", "--manifest-path", manifest], work)
    e.execute("fuzz-fetch", NCARGO + ["fetch", "--locked", "--manifest-path", manifest], work, timeout=600)
    lock = work / "fuzz/Cargo.lock"
    lock_hash = digest(lock)
    shutil.copyfile(lock, e.output / "fuzz-Cargo.lock.txt")
    e.env["CARGO_NET_OFFLINE"] = "true"
    corpus = work / "fuzz/corpus/frame"
    corpus.mkdir(parents=True)
    (corpus / "payload").write_bytes(b"temperature")
    (corpus / "frame").write_bytes(bytes([0xa5, 2, 7, 9, 12]))
    budget = ["--", "-runs=4096", "-seed=20261125", "-max_len=128"]
    base = NCARGO + ["fuzz", "run", "frame"]
    _, err, _ = e.execute("fuzz-baseline", base + budget, work, timeout=900)
    require("DONE" in err and re.search(r"cov:\s*\d+", err), "missing actual libFuzzer coverage/DONE log")
    code = work / "src/lib.rs"
    original = code.read_text(encoding="utf-8")
    mutation = "if frame[2 + length] != checksum(payload) {"
    require(original.count(mutation) == 1, "mutation target changed")
    try:
        write(code, original.replace(mutation, "if false {", 1))
        _, error, _ = e.execute("fuzz-mutant", base + ["--", "-runs=256", "-seed=20261125", "-max_len=128"],
                                work, expected=(1, 77, 101), timeout=900)
        require("single-bit checksum mutation accepted" in error,
                "mutant did not fail the intended property")
        crashes = sorted((work / "fuzz/artifacts/frame").glob("crash-*"))
        require(bool(crashes), "no actual libFuzzer crash artifact")
        artifact = e.output / "fuzz-counterexample.bin"
        shutil.copyfile(crashes[0], artifact)
    finally:
        write(code, original)
    e.execute("fuzz-replay-corrected", base + [artifact, "--", "-runs=1"], work, timeout=900)
    _, err, _ = e.execute("fuzz-restored", base + budget, work, timeout=900)
    require("DONE" in err and re.search(r"cov:\s*\d+", err), "missing restored fuzz run")
    require(lock_hash == digest(lock), "fuzz lockfile changed after preparation")
    report["fuzz"] = dict(baseline_budget=4096, mutant_budget=256, restored_budget=4096,
                          counterexample=artifact.name, counterexample_sha256=digest(artifact),
                          lock_sha256=lock_hash, actual_counterexample_replay="PASS",
                          scope="finite libFuzzer budget; not a proof of safety")


def audit(e, scratch, report):
    install(e, scratch, "cargo-audit", AUDIT_VERSION)
    e.run("cargo-audit-version", CARGO + ["audit", "--version"], scratch, contains=AUDIT_VERSION)
    db = scratch / "advisory-db"
    db.mkdir()
    e.execute("db-init", ["git", "init", db], scratch)
    e.execute("db-fetch", ["git", "-C", db, "fetch", "--depth=1",
              "https://github.com/RustSec/advisory-db.git", DB_SHA], scratch, timeout=600)
    e.execute("db-checkout", ["git", "-C", db, "checkout", "--detach", "FETCH_HEAD"], scratch)
    rev, _, _ = e.execute("db-revision", ["git", "-C", db, "rev-parse", "HEAD"], scratch)
    require(rev.strip() == DB_SHA, "advisory database revision mismatch")
    fixture = copy("audit-practice", scratch)
    manifest = fixture / "Cargo.toml"
    original = manifest.read_text(encoding="utf-8")
    lock = fixture / "Cargo.lock"
    histories = {}
    for label, version in (("historical", "1.5.4"), ("patched", "1.5.5")):
        write(manifest, original.replace('regex = "=1.5.4"', f'regex = "={version}"'))
        if lock.exists():
            lock.unlink()
        e.execute("resolve-" + label, CARGO + ["generate-lockfile"], fixture, timeout=300)
        package = [p for p in tomllib.loads(lock.read_text(encoding="utf-8"))["package"]
                   if p["name"] == "regex"]
        require(len(package) == 1 and package[0]["version"] == version,
                label + ": wrong historical dependency")
        saved = e.output / (label + "-Cargo.lock.txt")
        shutil.copyfile(lock, saved)
        histories[label] = saved
    # Only audit already existing lockfiles, from scratch outside any scanned project.
    # Disable yanked-index queries for this fixed snapshot exercise, not advisories.
    e.env["CARGO_NET_OFFLINE"] = "true"
    findings = {}
    for label, file in {**histories, **LOCKS}.items():
        before = digest(file)
        out, _, code = e.execute("audit-" + label, CARGO + ["audit", "--no-fetch", "--stale",
                                "--no-yanked", "--db", db, "--file", file, "--json"],
                                scratch, expected=(0, 1), timeout=300)
        data = json.loads(out)
        vulnerabilities = data["vulnerabilities"]["list"]
        ids = [v["advisory"]["id"] for v in vulnerabilities]
        require(digest(file) == before, label + ": lockfile modified")
        write(e.output / (label + "-audit.json"), json.dumps(data, ensure_ascii=False, indent=2) + "\n")
        findings[label] = dict(vulnerabilities=ids, exit_code=code,
                               lock_sha256=before, warnings=data.get("warnings", {}))
        if label == "historical":
            require(code == 1 and ADVISORY in ids, "historical advisory not detected")
        elif label == "patched":
            require(ADVISORY not in ids, "target advisory still present after version change")
    report["audit"] = dict(database_revision=DB_SHA, tool_version=AUDIT_VERSION,
                           target_advisory=ADVISORY, fixture_built_or_run=False,
                           yanked_check=False, findings=findings,
                           project_findings_require_review=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", required=True, choices=("stable", "miri", "fuzz", "audit"))
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(schema=1, stage="D11-B", mode=args.mode, status="FAIL",
                  git_commit=os.environ.get("GITHUB_SHA"), system=platform.system(),
                  stable_toolchain=TOOLCHAIN, source_before=before,
                  learner_attainment="not evaluated")
    try:
        if args.mode in {"miri", "fuzz", "audit"}:
            require(platform.system() == "Linux" and platform.machine() == "x86_64",
                    "automated specialist lab supports Linux x86_64 only")
        with tempfile.TemporaryDirectory(prefix="rustapplab-d11b-") as temp:
            scratch = Path(temp)
            e = Runner(out, scratch)
            report["steps"] = e.steps
            {"stable": stable, "miri": miri, "fuzz": fuzz, "audit": audit}[args.mode](e, scratch, report)
        require(snapshot() == before, "teaching source or existing dependency lock changed")
        report["status"] = "PASS"
    except (OSError, RuntimeError, subprocess.SubprocessError, ValueError, KeyError) as error:
        report["error"] = str(error)
        print(error, flush=True)
    report["source_after"] = snapshot()
    report["source_unchanged"] = before == report["source_after"]
    report["files"] = {p.name: digest(p) for p in sorted(out.iterdir()) if p.is_file()}
    write(out / "report.json", json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    return 0 if report["status"] == "PASS" and report["source_unchanged"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
