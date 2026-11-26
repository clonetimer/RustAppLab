#!/usr/bin/env python3
"""E3 real no_std HAL contracts, ARM ELF builds and QEMU interrupt execution."""
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
import zipfile

from modern_d8_check import CARGO, RUSTC, ROOT, digest, write
from modern_d11b_check import Runner, install, require, DB_SHA, AUDIT_VERSION

LAB = ROOT / "projects/modern-e3-embedded"
TARGET = "thumbv7m-none-eabi"
CONFIG = '[source.crates-io]\nreplace-with = "e3-vendor"\n[source.e3-vendor]\ndirectory = "vendor"\n'


def snapshot():
    paths = [p for p in LAB.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__", "vendor"}.intersection(p.relative_to(LAB).parts)]
    paths.extend([ROOT / "course/BOOK_E3_EMBEDDED.md", ROOT / "scripts/modern_d8_check.py",
                  ROOT / "scripts/modern_d11b_check.py", Path(__file__).resolve(), ROOT / "LICENSE"])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(set(paths))}


def stage(side, destination):
    shutil.copytree(LAB / side, destination,
                    ignore=shutil.ignore_patterns("target", ".git", "__pycache__", "vendor", "Cargo.lock"))
    shutil.copyfile(ROOT / "LICENSE", destination / "LICENSE")


def lock_count(lock):
    packages = tomllib.loads(lock.read_text(encoding="utf-8"))["package"]
    expected = {"cortex-m": "0.7.7", "cortex-m-rt": "0.7.5", "cortex-m-semihosting": "0.5.0"}
    for name, version in expected.items():
        require([p["version"] for p in packages if p["name"] == name] == [version], "pin mismatch " + name)
    require("1.0.0" in [p["version"] for p in packages if p["name"] == "embedded-hal"], "missing HAL 1.0")
    require(any(p["name"] == "e3-embedded-lab" and not p.get("source") for p in packages), "missing local core")
    registry = [p for p in packages if p.get("source")]
    require(all(p.get("checksum") for p in registry), "missing registry checksum")
    return len(registry)


def resolve(e, scratch, report):
    work = scratch / "resolved"
    stage("reference", work)
    lock = work / "Cargo.lock"
    e.env["CARGO_NET_OFFLINE"] = "false"
    committed = LAB / "reference/Cargo.lock"
    require(digest(committed) == digest(LAB / "learner/Cargo.lock") == "db2e90bc00344d3d33fa853f1a513cbb03780c078d491a08ca932293aa22909a", "frozen E3 lock differs from audited artifact")
    shutil.copyfile(committed, lock)
    e.execute("fetch-committed", CARGO + ["fetch", "--locked"], work, timeout=600)
    require(digest(committed) == digest(lock), "committed lock changed")
    count = lock_count(lock)
    saved = e.output / "Cargo.lock"
    shutil.copyfile(lock, saved)
    lock_hash = digest(saved)
    write(e.output / "Cargo.lock.sha256", lock_hash + "  Cargo.lock\n")
    install(e, scratch, "cargo-audit", AUDIT_VERSION)
    db = scratch / "advisory-db"
    db.mkdir()
    e.execute("db-init", ["git", "init", db], scratch)
    e.execute("db-fetch", ["git", "-C", db, "fetch", "--depth=1",
              "https://github.com/RustSec/advisory-db.git", DB_SHA], scratch, timeout=600)
    e.execute("db-checkout", ["git", "-C", db, "checkout", "--detach", "FETCH_HEAD"], scratch)
    rev, _, _ = e.execute("db-revision", ["git", "-C", db, "rev-parse", "HEAD"], scratch)
    require(rev.strip() == DB_SHA, "advisory DB mismatch")
    e.env["CARGO_NET_OFFLINE"] = "true"
    out, _, code = e.execute("audit-embedded-lock", CARGO + ["audit", "--no-fetch", "--stale",
                              "--no-yanked", "--db", db, "--file", saved, "--json"],
                              scratch, expected=(0, 1), timeout=300)
    data = json.loads(out)
    write(e.output / "embedded-audit.json", json.dumps(data, ensure_ascii=False, indent=2) + "\n")
    vulnerabilities = data["vulnerabilities"]["list"]
    unsound = data.get("warnings", {}).get("unsound", [])
    report["lock"] = dict(sha256=lock_hash, registry_packages=count, advisory_db=DB_SHA,
                          vulnerabilities=len(vulnerabilities), unsound=len(unsound),
                          warnings=data.get("warnings", {}), yanked_check=False)
    require(code == 0 and not vulnerabilities and not unsound, "embedded dependency gate rejected")
    e.env["CARGO_NET_OFFLINE"] = "false"
    vendor = scratch / "vendor"
    e.execute("vendor-locked", CARGO + ["vendor", "--locked", "--versioned-dirs",
              "--manifest-path", work / "Cargo.toml", vendor], scratch, timeout=600)
    require(digest(lock) == lock_hash, "vendor changed resolved lock")
    learner = scratch / "learner"
    stage("learner", learner)
    shutil.copyfile(saved, learner / "Cargo.lock")
    shutil.move(str(vendor), str(learner / "vendor"))
    write(learner / ".cargo/config.toml", CONFIG)
    archive = e.output / "E3-learner-offline.zip"
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
        for p in sorted(learner.rglob("*")):
            if p.is_file():
                rel = p.relative_to(learner).as_posix()
                info = zipfile.ZipInfo(rel, (2026, 11, 26, 0, 0, 0))
                info.create_system = 3
                info.external_attr = 0o100644 << 16
                z.writestr(info, p.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=6)
    with zipfile.ZipFile(archive) as z:
        require(z.testzip() is None, "bad learner ZIP")
        require(z.read("src/lib.rs") == (LAB / "learner/src/lib.rs").read_bytes(), "reference solution leaked")
        require("memory.x" in z.namelist() and "src/bin/firmware.rs" in z.namelist(), "missing real firmware sources")
        require(not any(n.startswith(("reference/", "target/")) for n in z.namelist()), "unexpected files")
    archive_hash = digest(archive)
    write(archive.with_suffix(".zip.sha256"), archive_hash + "  " + archive.name + "\n")
    report["archive"] = dict(name=archive.name, sha256=archive_hash, bytes=archive.stat().st_size)


def verify_vendor(e, work, home, label):
    out, _, _ = e.execute(label + "-metadata", CARGO + ["metadata", "--frozen", "--format-version", "1"],
                         work, timeout=300)
    packages = [p for p in json.loads(out)["packages"] if p.get("source")]
    vendor = (work / "vendor").resolve()
    require(packages and all(Path(p["manifest_path"]).resolve().is_relative_to(vendor) for p in packages),
            "registry package escaped vendor")
    entries = []
    if (home / "registry").exists():
        for p in sorted((home / "registry").rglob("*")):
            if p.is_file():
                name = p.relative_to(home / "registry").as_posix()
                entries.append(name)
                require(name == "CACHEDIR.TAG" and
                        p.read_bytes().startswith(b"Signature: 8a477f597d28d172789f06886806bc55"), "registry populated")
    write(e.output / (label + "-vendor.json"), json.dumps(dict(packages=len(packages), registry_files=entries),
          indent=2) + "\n")


def run_qemu(e, work, elf, label):
    command = ["qemu-system-arm", "-M", "lm3s6965evb", "-cpu", "cortex-m3",
               "-nographic", "-monitor", "none", "-serial", "none",
               "-semihosting-config", "enable=on,target=native", "-kernel", str(elf)]
    path = e.output / (label + "-qemu.txt")
    try:
        result = subprocess.run(command, cwd=work, env=e.env, capture_output=True, timeout=30)
    except subprocess.TimeoutExpired as error:
        write(path, json.dumps(command) + "\nTIMEOUT\n" +
              (error.stdout or b"").decode("utf-8", errors="replace") + "\n--- stderr ---\n" +
              (error.stderr or b"").decode("utf-8", errors="replace"))
        raise RuntimeError("actual QEMU timed out; see " + str(path)) from error
    out = result.stdout.decode("utf-8", errors="replace").replace("\r\n", "\n")
    err = result.stderr.decode("utf-8", errors="replace").replace("\r\n", "\n")
    combined = out + "\n" + err
    write(path, json.dumps(command) + "\n" + out + "\n--- stderr ---\n" + err)
    e.steps.append(dict(step=label+"-qemu", command=command, returncode=result.returncode,
                        log=path.name, sha256=digest(path), passed=False))
    require(result.returncode == 0 and "E3_QEMU_PASS" in combined and "E3_PANIC" not in combined,
            "actual QEMU did not complete successfully")
    ticks = re.findall(r"timer_irq_count=(\d+)", combined)
    rows = re.findall(r"sample=(\d+) scheduled=(\d+) raw=(\d+) enabled=(\d+) missed=(\d+)", combined)
    require(len(ticks) == 1 and len(rows) == 4, "missing real IRQ trace")
    values = [[int(x) for x in row] for row in rows]
    require([v[0] for v in values] == list(range(4)), "unexpected sample sequence")
    require([v[2] for v in values] == [1500, 2100, 1800, 1400], "wrong raw trace")
    require([v[3] for v in values] == [0, 1, 1, 0], "wrong control trace")
    previous = 0
    for row in values:
        require(row[1] == previous + 2 * (row[4] + 1), "periodic phase/missed deadline mismatch")
        previous = row[1]
    require(int(ticks[0]) >= previous >= 8, "inconsistent actual interrupt count")
    require("simulated_interlock=latched output_command=low" in combined, "missing fail-closed fixture")
    e.steps[-1]["passed"] = True
    print(label + "-qemu: PASS (actual ARM execution)", flush=True)
    return dict(status="PASS", interrupts=int(ticks[0]), samples=values, log=path.name)


def firmware(e, work, label, with_qemu):
    out, _, _ = e.execute(label + "-arm-build", CARGO + ["build", "--frozen", "--release", "--target", TARGET,
                          "--features", "firmware", "--bin", "firmware", "--message-format=json"],
                          work, timeout=600)
    exe = None
    for line in out.splitlines():
        item = json.loads(line)
        if item.get("reason") == "compiler-artifact" and item.get("target", {}).get("name") == "firmware" and item.get("executable"):
            exe = Path(item["executable"])
    require(exe is not None and exe.is_file(), "no actual ARM executable")
    blob = exe.read_bytes()
    require(len(blob) >= 52 and blob[:6] == b"\x7fELF\x01\x01" and
            int.from_bytes(blob[16:18], "little") == 2 and
            int.from_bytes(blob[18:20], "little") == 40, "not a 32-bit little-endian ARM executable ELF")
    saved = e.output / (label + "-firmware.elf")
    shutil.copyfile(exe, saved)
    result = dict(target=TARGET, elf=saved.name, sha256=digest(saved), file_bytes=saved.stat().st_size,
                  machine="ARM", qemu=dict(status="NOT_RUN", reason="QEMU automated on Ubuntu only"))
    if with_qemu:
        result["qemu"] = run_qemu(e, work, saved, label)
    return result


def replay(e, scratch, report, bundle, with_qemu):
    bundle = bundle.resolve()
    expected = bundle.with_suffix(".zip.sha256").read_text(encoding="utf-8").split()[0]
    require(digest(bundle) == expected, "learner archive hash mismatch")
    learner = scratch / "搬移 嵌入式学员"
    with zipfile.ZipFile(bundle) as z:
        require(z.testzip() is None, "bad learner archive")
        z.extractall(learner)
    lock = learner / "Cargo.lock"
    lock_hash = digest(lock)
    count = lock_count(lock)
    committed = LAB / "reference/Cargo.lock"
    require(lock_hash == digest(committed) == digest(LAB / "learner/Cargo.lock") == "db2e90bc00344d3d33fa853f1a513cbb03780c078d491a08ca932293aa22909a", "bundle differs from committed lock")
    reference = scratch / "reference"
    stage("reference", reference)
    shutil.copyfile(lock, reference / "Cargo.lock")
    shutil.copytree(learner / "vendor", reference / "vendor")
    write(reference / ".cargo/config.toml", CONFIG)
    e.env["CARGO_NET_OFFLINE"] = "true"
    e.execute("installed-arm-target", ["rustup", "target", "list", "--installed", "--toolchain", "1.100.0"], scratch)
    if with_qemu:
        require(platform.system() == "Linux", "QEMU proof is scoped to Linux")
        e.execute("qemu-version", ["qemu-system-arm", "--version"], scratch)
    protected = ("tests/contracts.rs", "src/bin/firmware.rs", "src/sim.rs", "build.rs", "memory.x")
    results = {}
    for label, work in (("reference", reference), ("learner", learner)):
        home = scratch / ("empty-" + label)
        home.mkdir()
        require(not list(home.iterdir()), "Cargo home not empty")
        e.env["CARGO_HOME"] = str(home)
        fixed = {p: digest(work / p) for p in protected}
        require(all(fixed[p] == digest(reference / p) for p in protected), "fixed files differ")
        if label == "reference":
            for mode, flags in (("debug", []), ("release", ["--release"])):
                out, _, _ = e.execute("reference-" + mode, CARGO + ["test", "--frozen", "--all-targets"] + flags,
                                       work, timeout=600)
                require("5 passed; 0 failed; 0 ignored" in out, "five real host contracts did not pass")
        else:
            e.execute("learner-compiles", CARGO + ["check", "--frozen", "--all-targets"], work, timeout=600)
            for n in range(1, 5):
                out, _, _ = e.execute(f"learner-red-{n}", CARGO + ["test", "--frozen", "--test", "contracts", f"step{n}_"],
                                      work, expected=(101,), timeout=300)
                require("test result: FAILED" in out, "not an actual RED contract")
            shutil.copyfile(reference / "src/lib.rs", work / "src/lib.rs")
            out, _, _ = e.execute("learner-green", CARGO + ["test", "--frozen", "--all-targets"], work, timeout=600)
            require("5 passed; 0 failed; 0 ignored" in out, "learner fixed contract failed")
            out, _, _ = e.execute("host-demo", CARGO + ["run", "--frozen", "--example", "host_demo"], work)
            require(out.strip().splitlines() == [
                "scheduled=2 raw=1500 enabled=0 missed=0",
                "scheduled=4 raw=2100 enabled=1 missed=0",
                "scheduled=8 raw=1800 enabled=1 missed=1",
                "scheduled=10 raw=1400 enabled=0 missed=0",
                "host fixture only; not a physical timing test"], "host integration trace differs")
        results[label] = firmware(e, work, label, with_qemu)
        verify_vendor(e, work, home, label)
        require(all(fixed[p] == digest(work / p) for p in protected) and digest(work / "Cargo.lock") == lock_hash,
                "fixed teaching inputs changed")
    require(digest(bundle) == expected, "pristine learner archive changed")
    report["firmware"] = results
    report["portable"] = dict(archive_sha256=expected, lock_sha256=lock_hash, registry_packages=count,
                              empty_cache_replay="PASS")
    report["physical_hardware"] = "NOT_EVALUATED"
    report["hard_real_time"] = "NOT_EVALUATED"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("resolve", "replay"), required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--bundle", type=Path)
    parser.add_argument("--qemu", action="store_true")
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(stage="E3", mode=args.mode, status="FAIL", system=platform.system(),
                  git_commit=os.environ.get("GITHUB_SHA"), source_before=before, human_learning="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-e3-") as temp:
            scratch = Path(temp)
            e = Runner(out, scratch)
            report["steps"] = e.steps
            e.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
            if args.mode == "resolve":
                resolve(e, scratch, report)
            else:
                require(args.bundle is not None, "replay requires bundle")
                replay(e, scratch, report, args.bundle, args.qemu)
        require(snapshot() == before, "teaching source changed")
        report["status"] = "PASS"
    except (OSError, RuntimeError, subprocess.SubprocessError, ValueError, KeyError) as error:
        report["error"] = str(error)
        print(error, flush=True)
    report["source_after"] = snapshot()
    report["source_unchanged"] = before == report["source_after"]
    report["files"] = {p.name: digest(p) for p in sorted(out.iterdir()) if p.is_file()}
    write(out / "report.json", json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    return 0 if report["status"] == "PASS" and report["source_unchanged"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
