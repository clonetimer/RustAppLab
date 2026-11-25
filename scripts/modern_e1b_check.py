#!/usr/bin/env python3
"""E1-B actual SQLite/TLS, same resolved audited lock and relocated learner replay."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import tomllib
import zipfile

from modern_d8_check import CARGO, RUSTC, ROOT, digest, write
from modern_d11b_check import Runner, install, require, DB_SHA, AUDIT_VERSION
from modern_e1a_check import stage as stage_e1a, verify_vendor, snapshot as e1a_snapshot, LAB as E1A

LAB = ROOT / "projects/modern-e1-sqlite"
CONFIG = '[source.crates-io]\nreplace-with = "e1b-vendor"\n[source.e1b-vendor]\ndirectory = "vendor"\n'


def snapshot():
    current = e1a_snapshot()
    paths = [p for p in LAB.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__", "vendor"}.intersection(p.relative_to(LAB).parts)]
    paths.extend([ROOT / "course/BOOK_E1_SQLITE.md", Path(__file__).resolve()])
    current.update({p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(paths)})
    return current


def stage(side, destination):
    shutil.copytree(LAB / side, destination,
                    ignore=shutil.ignore_patterns("target", ".git", "__pycache__", "Cargo.lock"))
    manifest = destination / "Cargo.toml"
    text = manifest.read_text(encoding="utf-8")
    require(text.count("../../modern-e1-tls/reference") == 1 and
            text.count("../../../apps/servicedesk") == 1, "prerequisite path changed")
    write(manifest, text.replace("../../modern-e1-tls/reference", "support/e1a")
          .replace("../../../apps/servicedesk", "support/e1a/support/servicedesk"))
    support = destination / "support/e1a"
    stage_e1a("reference", support)
    # Prior completed course only as a library. No E1-B answers.
    for folder in ("tests", "examples"):
        shutil.rmtree(support / folder)
    (support / "src/main.rs").unlink()
    shutil.copyfile(ROOT / "LICENSE", destination / "LICENSE")


def verify_lock(lock):
    packages = tomllib.loads(lock.read_text(encoding="utf-8"))["package"]
    expected = {"tokio": "1.53.3", "axum": "0.8.8", "rustls": "0.23.46",
                "tokio-rustls": "0.26.6", "rusqlite": "0.43.0", "rcgen": "0.14.10"}
    for name, version in expected.items():
        require([p["version"] for p in packages if p["name"] == name] == [version], "wrong version: " + name)
    registry = [p for p in packages if p.get("source")]
    require(all(p.get("checksum") for p in registry), "missing registry checksum")
    require(any(p["name"] == "libsqlite3-sys" for p in registry), "no real native SQLite dependency")
    for name in ("e1-secure-lab", "servicedesk", "servicedesk-c2"):
        require(any(p["name"] == name and "source" not in p for p in packages), "missing local prerequisite")
    return len(registry)


def resolve(e, scratch, report):
    work = scratch / "resolved"
    stage("reference", work)
    lock = work / "Cargo.lock"
    require(digest(E1A / "reference/Cargo.lock") == digest(E1A / "learner/Cargo.lock"),
            "E1-A prerequisite locks differ")
    e.env["CARGO_NET_OFFLINE"] = "false"
    if (LAB / "reference/Cargo.lock").is_file():
        require(digest(LAB / "reference/Cargo.lock") == digest(LAB / "learner/Cargo.lock"), "E1-B locks differ")
        shutil.copyfile(LAB / "reference/Cargo.lock", lock)
        e.execute("fetch-frozen-lock", CARGO + ["fetch", "--locked"], work, timeout=900)
    else:
        shutil.copyfile(E1A / "reference/Cargo.lock", lock)
        e.execute("resolve-from-e1a-lock", CARGO + ["fetch"], work, timeout=900)
    count = verify_lock(lock)
    saved = e.output / "Cargo.lock"
    shutil.copyfile(lock, saved)
    sha = digest(saved)
    write(e.output / "Cargo.lock.sha256", sha + "  Cargo.lock\n")
    e.execute("fetch-locked", CARGO + ["fetch", "--locked"], work, timeout=900)
    require(digest(lock) == sha, "locked fetch modified lock")
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
    out, _, code = e.execute("audit-sqlite-lock", CARGO + ["audit", "--no-fetch", "--stale",
                              "--no-yanked", "--db", db, "--file", saved, "--json"],
                              scratch, expected=(0, 1), timeout=300)
    data = json.loads(out)
    write(e.output / "sqlite-audit.json", json.dumps(data, ensure_ascii=False, indent=2) + "\n")
    vulnerabilities = data["vulnerabilities"]["list"]
    unsound = data.get("warnings", {}).get("unsound", [])
    report["lock"] = dict(sha256=sha, registry_packages=count, advisory_db=DB_SHA,
                          vulnerabilities=len(vulnerabilities), unsound=len(unsound),
                          warnings=data.get("warnings", {}), yanked_check=False)
    require(code == 0 and not vulnerabilities and not unsound, "SQLite/TLS dependency gate rejected")


def archive_learner(path, output):
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
        for p in sorted(path.rglob("*")):
            if not p.is_file():
                continue
            name = p.relative_to(path).as_posix()
            info = zipfile.ZipInfo(name, (2026, 11, 25, 0, 0, 0))
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            z.writestr(info, p.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=6)
    with zipfile.ZipFile(output) as z:
        require(z.testzip() is None, "ZIP integrity")
        require(z.read("src/lib.rs") == (LAB / "learner/src/lib.rs").read_bytes(), "E1-B solution leaked")
        require("support/e1a/src/lib.rs" in z.namelist() and
                "support/e1a/support/servicedesk/src/lib.rs" in z.namelist(), "missing completed prerequisites")
        require(not any(n.startswith(("reference/", "target/")) for n in z.namelist()), "unexpected reference/build files")
    return digest(output)


def replay(e, scratch, report, lockfile):
    lockfile = lockfile.resolve()
    expected_hash = lockfile.with_suffix(".lock.sha256").read_text(encoding="utf-8").split()[0]
    require(digest(lockfile) == expected_hash, "resolved lock hash mismatch")
    if (LAB / "reference/Cargo.lock").is_file():
        require(expected_hash == digest(LAB / "reference/Cargo.lock") == digest(LAB / "learner/Cargo.lock"),
                "artifact differs from committed lock")
    count = verify_lock(lockfile)
    reference, learner = scratch / "reference", scratch / "learner-template"
    stage("reference", reference)
    stage("learner", learner)
    for work in (reference, learner):
        shutil.copyfile(lockfile, work / "Cargo.lock")
    vendor = scratch / "vendor"
    e.env["CARGO_NET_OFFLINE"] = "false"
    e.execute("vendor-locked", CARGO + ["vendor", "--locked", "--versioned-dirs",
              "--manifest-path", reference / "Cargo.toml", vendor], scratch, timeout=1200)
    for work in (reference, learner):
        shutil.copytree(vendor, work / "vendor")
        write(work / ".cargo/config.toml", CONFIG)
    archive = e.output / "E1-B-learner-offline.zip"
    archive_hash = archive_learner(learner, archive)
    write(archive.with_suffix(".zip.sha256"), archive_hash + "  " + archive.name + "\n")
    moved = scratch / "搬移 SQLite 学员 空缓存"
    with zipfile.ZipFile(archive) as z:
        z.extractall(moved)
    e.env["CARGO_NET_OFFLINE"] = "true"
    for label, work in (("reference", reference), ("learner", moved)):
        home = scratch / ("empty-" + label)
        home.mkdir()
        require(not list(home.iterdir()), "Cargo home not empty")
        e.env["CARGO_HOME"] = str(home)
        fixed = work / "tests/contracts.rs"
        fixed_hash = digest(fixed)
        require(fixed_hash == digest(reference / "tests/contracts.rs"), "fixed contracts differ")
        if label == "reference":
            for mode, args in (("debug", []), ("release", ["--release"])):
                out, _, _ = e.execute("reference-" + mode, CARGO + ["test", "--frozen", "--all-targets"] + args,
                                       work, timeout=1200)
                require("5 passed; 0 failed; 0 ignored" in out, "missing five actual contracts")
        else:
            e.execute("learner-compiles", CARGO + ["check", "--frozen", "--all-targets"], work, timeout=1200)
            for number in range(1,5):
                out, _, _ = e.execute(f"learner-red-{number}", CARGO + ["test", "--frozen", "--test", "contracts", f"step{number}_"],
                                      work, expected=(101,), timeout=300)
                require("test result: FAILED" in out, "wrong failure, not actual red contract")
            shutil.copyfile(reference / "src/lib.rs", work / "src/lib.rs")
            out, _, _ = e.execute("learner-green", CARGO + ["test", "--frozen", "--all-targets"], work, timeout=1200)
            require("5 passed; 0 failed; 0 ignored" in out, "learner contracts not green")
            out, _, _ = e.execute("local-demo", CARGO + ["run", "--frozen", "--example", "local_demo"], work, timeout=300)
            require("HTTPS+SQLite verified; denied=403; created=201; closed=200; reopened=1; audit=2" in out,
                    "actual SQLite demo missing")
            events = [json.loads(line) for line in out.splitlines() if line.startswith("{")]
            require(len(events) == 3 and {event["status"] for event in events} == {403, 201, 200},
                    "missing actual access events")
            database = scratch / "cli-new.sqlite"
            e.execute("cli-init", CARGO + ["run", "--frozen", "--", "init", database], work)
            require(database.read_bytes().startswith(b"SQLite format 3\x00"), "not an actual SQLite DB")
            before = digest(database)
            e.execute("cli-init-refuses-overwrite", CARGO + ["run", "--frozen", "--", "init", database],
                      work, expected=(1,))
            require(digest(database) == before, "existing DB overwritten")
        verify_vendor(e, work, home, label, count)
        require(digest(fixed) == fixed_hash and digest(work / "Cargo.lock") == expected_hash, "fixed inputs modified")
    require(digest(archive) == archive_hash, "pristine learner archive modified")
    report["portable"] = dict(artifact=archive.name, sha256=archive_hash, lock_sha256=expected_hash,
                              registry_packages=count, empty_cache_replay="PASS",
                              supplied_prerequisites="E1-A/C2; no E1-B answer")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", required=True, choices=("resolve", "replay"))
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--lock-file", type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(stage="E1-B", mode=args.mode, status="FAIL", system=platform.system(),
                  git_commit=os.environ.get("GITHUB_SHA"), source_before=before, human_learning="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-e1b-") as temp:
            scratch = Path(temp)
            evidence = Runner(out, scratch)
            report["steps"] = evidence.steps
            evidence.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
            if args.mode == "resolve":
                resolve(evidence, scratch, report)
            else:
                require(args.lock_file is not None, "replay needs resolved lock")
                replay(evidence, scratch, report, args.lock_file)
        require(snapshot() == before, "source changed")
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
