#!/usr/bin/env python3
"""E1-A: one resolved/audited TLS lock, then actual two-OS frozen learner replay."""
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

LAB = ROOT / "projects/modern-e1-tls"
C2 = ROOT / "projects/advanced-c2/reference"
DOMAIN = ROOT / "apps/servicedesk"
CONFIG = '[source.crates-io]\nreplace-with = "e1-vendor"\n[source.e1-vendor]\ndirectory = "vendor"\n'


def snapshot():
    paths = []
    for base in (LAB, C2, DOMAIN):
        paths.extend(p for p in base.rglob("*") if p.is_file() and not
                     {"target", ".git", "__pycache__", "vendor"}.intersection(p.relative_to(base).parts))
    paths.extend([ROOT / "course/BOOK_E1_TLS_AUTH.md", ROOT / "scripts/modern_d8_check.py",
                  ROOT / "scripts/modern_d11b_check.py", Path(__file__).resolve(), ROOT / "LICENSE"])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(set(paths))}


def stage(side, path):
    shutil.copytree(LAB / side, path, ignore=shutil.ignore_patterns("target", ".git", "__pycache__", "Cargo.lock"))
    manifest = path / "Cargo.toml"
    source = manifest.read_text(encoding="utf-8")
    require(source.count('../../advanced-c2/reference') == 1, "C2 source path changed")
    write(manifest, source.replace('../../advanced-c2/reference', 'support/c2'))
    c2 = path / "support/c2"
    shutil.copytree(C2 / "src", c2 / "src")
    shutil.copytree(C2 / "assets", c2 / "assets")
    manifest = (C2 / "Cargo.toml").read_text(encoding="utf-8")
    require(manifest.count("../../../apps/servicedesk") == 1, "C2 domain source path changed")
    # Drop only C2's standalone test dependencies in this supplied library.
    manifest = manifest.split("[dev-dependencies]")[0].rstrip() + "\n"
    write(c2 / "Cargo.toml", manifest.replace("../../../apps/servicedesk", "../servicedesk"))
    domain = path / "support/servicedesk"
    shutil.copytree(DOMAIN / "src", domain / "src")
    shutil.copytree(DOMAIN / "samples", domain / "samples")
    shutil.copyfile(DOMAIN / "Cargo.toml", domain / "Cargo.toml")
    shutil.copyfile(ROOT / "LICENSE", path / "LICENSE")


def verify_lock(lock):
    packages = tomllib.loads(lock.read_text(encoding="utf-8"))["package"]
    expected = {"tokio": "1.53.3", "axum": "0.8.8", "rustls": "0.23.46",
                "tokio-rustls": "0.26.6", "subtle": "2.6.1", "rcgen": "0.14.10"}
    for name, version in expected.items():
        require([p["version"] for p in packages if p["name"] == name] == [version], "lock pin differs: " + name)
    require(all(p.get("checksum") for p in packages if p.get("source")), "missing registry checksum")
    require(any(p["name"] == "servicedesk-c2" and "source" not in p for p in packages), "C2 is not a path dependency")
    return len([p for p in packages if p.get("source")])


def resolve(e, scratch, report):
    work = scratch / "resolved"
    stage("reference", work)
    lock = work / "Cargo.lock"
    shutil.copyfile(C2 / "Cargo.lock", lock)
    e.env["CARGO_NET_OFFLINE"] = "false"
    # A real Cargo resolution updates the seed; do not hand-author new checksums.
    e.execute("resolve-from-c2-lock", CARGO + ["fetch"], work, timeout=900)
    count = verify_lock(lock)
    saved = e.output / "Cargo.lock"
    shutil.copyfile(lock, saved)
    sha = digest(saved)
    write(e.output / "Cargo.lock.sha256", sha + "  Cargo.lock\n")
    e.execute("fetch-locked", CARGO + ["fetch", "--locked"], work, timeout=900)
    require(digest(lock) == sha, "locked fetch altered resolved lock")
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
    out, _, code = e.execute("audit-tls-lock", CARGO + ["audit", "--no-fetch", "--stale",
                                "--no-yanked", "--db", db, "--file", saved, "--json"],
                                scratch, expected=(0, 1), timeout=300)
    data = json.loads(out)
    write(e.output / "tls-audit.json", json.dumps(data, indent=2, ensure_ascii=False) + "\n")
    vulnerabilities = data["vulnerabilities"]["list"]
    unsound = data.get("warnings", {}).get("unsound", [])
    report["lock"] = dict(sha256=sha, registry_packages=count, advisory_db=DB_SHA,
                          vulnerabilities=len(vulnerabilities), unsound=len(unsound),
                          other_warnings=data.get("warnings", {}), yanked_check=False,
                          initial_resolution="CI generated; freeze in follow-up")
    require(code == 0 and not vulnerabilities and not unsound, "TLS dependency gate rejected; see audit JSON")


def archive_learner(path, output):
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
        for p in sorted(path.rglob("*")):
            if not p.is_file():
                continue
            rel = p.relative_to(path).as_posix()
            info = zipfile.ZipInfo(rel, (2026, 11, 25, 0, 0, 0))
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            z.writestr(info, p.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=6)
    with zipfile.ZipFile(output) as z:
        require(z.testzip() is None, "ZIP integrity")
        require(z.read("src/lib.rs") == (LAB / "learner/src/lib.rs").read_bytes(), "E1 answers leaked or changed")
        require("support/c2/src/lib.rs" in z.namelist(), "missing supplied C2 support")
        require(not any(n.startswith(("reference/", "target/")) for n in z.namelist()), "unexpected reference/build data")
    return digest(output)


def verify_vendor(e, work, home, label, expected):
    out, _, _ = e.execute(label + "-metadata", CARGO + ["metadata", "--frozen", "--format-version", "1"], work)
    pkgs = [p for p in json.loads(out)["packages"] if p.get("source")]
    require(len(pkgs) <= expected and any(p["name"] == "rustls" for p in pkgs), "metadata graph mismatch")
    vendor = (work / "vendor").resolve()
    require(all(Path(p["manifest_path"]).resolve().is_relative_to(vendor) for p in pkgs), "dependency escaped vendor")
    registry = home / "registry"
    entries = []
    if registry.exists():
        for p in sorted(registry.rglob("*")):
            if p.is_file():
                rel = p.relative_to(registry).as_posix()
                entries.append(rel)
                require(rel == "CACHEDIR.TAG" and p.read_bytes().startswith(b"Signature: 8a477f597d28d172789f06886806bc55"),
                        "registry populated: " + rel)
    write(e.output / (label + "-vendor.json"), json.dumps({"packages": len(pkgs), "registry_files": entries,
        "resolved": [{"name": p["name"], "version": p["version"],
                      "manifest": Path(p["manifest_path"]).resolve().relative_to(work.resolve()).as_posix()}
                     for p in pkgs]}, indent=2) + "\n")


def replay(e, scratch, report, lockfile):
    lockfile = lockfile.resolve()
    expected_hash = lockfile.with_suffix(".lock.sha256").read_text(encoding="utf-8").split()[0]
    require(digest(lockfile) == expected_hash, "resolved lock checksum mismatch")
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
    # Archive the pristine learner before injecting reference solutions.
    artifact = e.output / "E1-A-learner-offline.zip"
    archive_hash = archive_learner(learner, artifact)
    write(artifact.with_suffix(".zip.sha256"), archive_hash + "  " + artifact.name + "\n")
    moved = scratch / "搬移 学员 空缓存"
    with zipfile.ZipFile(artifact) as z:
        z.extractall(moved)
    e.env["CARGO_NET_OFFLINE"] = "true"
    for label, work in (("reference", reference), ("learner", moved)):
        home = scratch / ("empty-" + label)
        home.mkdir()
        require(not list(home.iterdir()), "Cargo home is not initially empty")
        e.env["CARGO_HOME"] = str(home)
        fixed = work / "tests/contracts.rs"
        fixed_hash = digest(fixed)
        require(fixed_hash == digest(reference / "tests/contracts.rs"), "contract mismatch")
        if label == "reference":
            for mode, args in (("debug", []), ("release", ["--release"])):
                out, _, _ = e.execute("reference-" + mode, CARGO + ["test", "--frozen", "--all-targets"] + args,
                                       work, timeout=900)
                require("5 passed; 0 failed; 0 ignored" in out, "missing five contracts")
        else:
            e.execute("learner-compiles", CARGO + ["check", "--frozen", "--all-targets"], work, timeout=900)
            for n in range(1, 5):
                out, _, _ = e.execute(f"learner-red-{n}", CARGO + ["test", "--frozen", "--test", "contracts", f"step{n}_"],
                                      work, expected=(101,), timeout=300)
                require("test result: FAILED" in out, "wrong failure, not actual red contract")
            shutil.copyfile(reference / "src/lib.rs", work / "src/lib.rs")
            out, _, _ = e.execute("learner-green", CARGO + ["test", "--frozen", "--all-targets"], work, timeout=900)
            require("5 passed; 0 failed; 0 ignored" in out, "learner fixed contracts did not pass")
            out, _, _ = e.execute("local-demo", CARGO + ["run", "--frozen", "--example", "local_demo"], work, timeout=300)
            require("HTTPS verified; missing=401; reader_write=403; writer_create=201; reader_list=200" in out, "actual HTTPS demo missing")
        verify_vendor(e, work, home, label, count)
        require(digest(fixed) == fixed_hash and digest(work / "Cargo.lock") == expected_hash, "fixed inputs changed")
    require(digest(artifact) == archive_hash, "pristine learner archive changed")
    report["portable"] = dict(artifact=artifact.name, sha256=archive_hash, lock_sha256=expected_hash,
                              registry_packages=count, empty_cache_replay="PASS",
                              supplied_prerequisite="C2 source only; no E1-A answers")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", required=True, choices=("resolve", "replay"))
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--lock-file", type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(stage="E1-A", mode=args.mode, status="FAIL", system=platform.system(),
                  git_commit=os.environ.get("GITHUB_SHA"), source_before=before,
                  human_learning="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-e1a-") as temp:
            scratch = Path(temp)
            e = Runner(out, scratch)
            report["steps"] = e.steps
            e.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
            if args.mode == "resolve":
                resolve(e, scratch, report)
            else:
                require(args.lock_file is not None, "replay requires resolved lock")
                replay(e, scratch, report, args.lock_file)
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
