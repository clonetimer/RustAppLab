#!/usr/bin/env python3
"""D9-B actual standard-library contracts, Cargo diagnostics and loopback registry."""
from __future__ import annotations

import argparse
from datetime import datetime, timedelta, timezone
import functools
import hashlib
import http.server
import io
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import tempfile
import threading
import tomllib

from modern_d8_check import Evidence, CARGO, RUSTC, TOOLCHAIN, ROOT, digest, write

LAB = ROOT / "projects/modern-d9-stdlib"


def snapshot():
    files = [p for p in LAB.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB).parts)]
    files.extend([ROOT / "course/BOOK_D9_STDLIB_CARGO.md",
                  ROOT / "course/MODERN_STABLE_FEATURE_INDEX.md",
                  ROOT / "scripts/modern_d8_check.py", Path(__file__).resolve()])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(files)}


def stdlib(evidence, scratch):
    reference, learner = scratch / "reference", scratch / "learner"
    for dest in (reference, learner):
        shutil.copytree(LAB / dest.name, dest, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    common = ["--locked", "--offline"]
    evidence.run("reference-debug", CARGO + ["test"] + common + ["--all-targets"],
                 reference, contains="16 passed")
    evidence.run("reference-release", CARGO + ["test"] + common + ["--release", "--all-targets"],
                 reference, contains="16 passed")
    contract = learner / "tests/contracts.rs"
    before = digest(contract)
    if before != digest(reference / "tests/contracts.rs"):
        raise RuntimeError("learner/reference contract mismatch")
    for number in range(1, 17):
        evidence.run(f"learner-red-{number}", CARGO + ["test"] + common +
                     ["--test", "contracts", f"step{number}_"], learner,
                     expected=101, contains="test result: FAILED")
    shutil.copyfile(reference / "src/lib.rs", learner / "src/lib.rs")
    evidence.run("learner-green", CARGO + ["test"] + common + ["--all-targets"],
                 learner, contains="16 passed")
    if digest(contract) != before:
        raise RuntimeError("learner contract changed")
    for name, args, code, out, err in (
        ("telemetry", ["7", "bad", "120", "0"], 0,
         "good=3; bad=1; outliers=[120]; changes=[-7]", None),
        ("empty", [], 1, None, "usage: telemetry"),
        ("boundary", ["4294967295", "bad", "0"], 0,
         "good=2; bad=1; outliers=[2147483647]; changes=[]", None),
    ):
        evidence.run("cli-" + name, CARGO + ["run"] + common +
                     ["--example", "telemetry", "--"] + args, learner,
                     expected=code, contains=out, stderr=err)


def cargo_practice(evidence, scratch):
    work = scratch / "cargo-practice"
    shutil.copytree(LAB / work.name, work, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    manifest, main = work / "Cargo.toml", work / "src/main.rs"
    evidence.run("cargo-lock-v4", CARGO + ["generate-lockfile", "--offline"], work)
    lock = work / "Cargo.lock"
    if tomllib.loads(lock.read_text(encoding="utf-8"))["version"] != 4:
        raise RuntimeError("expected lockfile v4")
    shutil.copyfile(lock, evidence.output / "cargo-lock-v4.toml.txt")
    check = CARGO + ["check", "--locked", "--offline", "--all-targets"]
    evidence.run("cargo-unused-warn", check, work, stderr="unused dependency")
    write(manifest, manifest.read_text(encoding="utf-8") +
          '\n[lints.cargo]\nunused_dependencies = "deny"\n')
    evidence.run("cargo-unused-deny", check, work, expected=101, stderr="unused dependency")
    code = main.read_text(encoding="utf-8").replace(
        '    println!("mode={}", env!("D9_MODE"));',
        '    println!("mode={}", env!("D9_MODE"));\n    println!("helper={}", d9_helper::label());')
    write(main, code)
    evidence.run("cargo-unused-fixed", check, work, stderr="unused variable")
    strict = check + ["--config", 'build.warnings="deny"']
    evidence.run("cargo-rust-warning-deny", strict, work, expected=101, stderr="unused variable")
    write(main, code.replace("    let unused_value = 7;\n", ""))
    evidence.run("cargo-rust-warning-fixed", strict, work)
    output, _ = evidence.run("cargo-debug-profile-include",
                             CARGO + ["run", "--locked", "--offline", "--profile", "debug"], work)
    if output.strip().splitlines() != ["mode=included", "helper=ready"]:
        raise RuntimeError("include/debug profile output differs: " + output)
    write(work / ".cargo/shared.toml", '[env]\nD9_MODE = { value = "changed", force = true }\n')
    evidence.run("cargo-include-changed",
                 CARGO + ["run", "--locked", "--offline", "--profile", "debug"], work,
                 contains="mode=changed")


def crate_archive(directory, version):
    name = "d9age"
    prefix = f"{name}-{version}"
    target = directory / f"{prefix}.crate"
    directory.mkdir(parents=True, exist_ok=True)
    data = {
        "Cargo.toml": '[package]\nname = "d9age"\nversion = "' + version +
                      '"\nedition = "2024"\nrust-version = "1.100"\nlicense = "MIT"\n',
        "src/lib.rs": "pub fn version() -> &'static str { " + json.dumps(version) + " }\n",
    }
    with tarfile.open(target, "w:gz") as tar:
        for relative, content in data.items():
            raw = content.encode("utf-8")
            info = tarfile.TarInfo(f"{prefix}/{relative}")
            info.size = len(raw)
            info.mode = 0o644
            tar.addfile(info, io.BytesIO(raw))
    return digest(target)


def cooldown(evidence, scratch):
    registry = scratch / "registry"
    registry.mkdir()
    requests = []
    now = datetime.now(timezone.utc)
    versions = [("1.0.0", now - timedelta(days=30)), ("1.1.0", now - timedelta(hours=1))]
    entries = []
    for version, published in versions:
        entries.append(dict(name="d9age", vers=version, deps=[], features={}, yanked=False,
                            cksum=crate_archive(registry / "crates", version),
                            rust_version="1.100", pubtime=published.strftime("%Y-%m-%dT%H:%M:%SZ")))
    write(registry / "d9/ag/d9age", "".join(json.dumps(entry) + "\n" for entry in entries))
    write(evidence.output / "registry-index.jsonl.txt", "".join(json.dumps(entry) + "\n" for entry in entries))

    class Handler(http.server.SimpleHTTPRequestHandler):
        def log_message(self, format, *args):
            requests.append(self.path)

    factory = functools.partial(Handler, directory=str(registry))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), factory)
    origin = f"http://127.0.0.1:{server.server_port}"
    write(registry / "config.json", json.dumps({"dl": origin + "/crates/{crate}-{version}.crate"}) + "\n")
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    prior = evidence.env.copy()
    try:
        # Only explicit synthetic alternate registry and its downloads use HTTP;
        # a fresh Cargo home contains no registry credentials or global config.
        for key in list(evidence.env):
            if (key.lower() in {"http_proxy", "https_proxy", "all_proxy", "no_proxy"}
                    or key.startswith(("CARGO_REGISTRIES_", "CARGO_REGISTRY_", "CARGO_HTTP_",
                                       "CARGO_NET_", "CARGO_RESOLVER_"))):
                evidence.env.pop(key, None)
        evidence.env["CARGO_HOME"] = str(scratch / "registry-cargo-home")
        evidence.env["CARGO_NET_OFFLINE"] = "false"
        evidence.env["CARGO_NET_RETRY"] = "0"
        evidence.env["NO_PROXY"] = "127.0.0.1"
        def config(work, age):
            write(work / ".cargo/config.toml",
                  '[registries.d9-probe]\nindex = "sparse+' + origin + '/"\nmin-publish-age = "' + age +
                  '"\n[resolver]\nincompatible-publish-age = "deny"\n')
        def consumer(name, age):
            work = scratch / name
            write(work / "Cargo.toml",
                  '[package]\nname = "d9-cooldown-consumer"\nversion = "0.1.0"\nedition = "2024"\n'
                  'rust-version = "1.100"\npublish = false\n[workspace]\nresolver = "3"\n'
                  '[dependencies]\nd9age = { version = "1", registry = "d9-probe" }\n')
            write(work / "src/main.rs", 'fn main() { println!("{}", d9age::version()); }\n')
            config(work, age)
            return work
        def verify(work, label, expected):
            lock = work / "Cargo.lock"
            selected = [p["version"] for p in tomllib.loads(lock.read_text(encoding="utf-8"))["package"]
                        if p["name"] == "d9age"]
            if selected != [expected]:
                raise RuntimeError(f"{label}: expected {expected}, got {selected}")
            shutil.copyfile(lock, evidence.output / (label + ".lock.txt"))
            out, _ = evidence.run(label + "-run", CARGO + ["run", "--locked"], work)
            if out.strip() != expected:
                raise RuntimeError(f"{label}: runtime version {out!r}")
        open_work = consumer("no-cooldown", "0")
        evidence.run("cooldown-disabled-lock", CARGO + ["generate-lockfile"], open_work)
        verify(open_work, "cooldown-disabled", "1.1.0")
        restricted = consumer("three-day-cooldown", "3 days")
        evidence.run("cooldown-enabled-lock", CARGO + ["generate-lockfile"], restricted)
        verify(restricted, "cooldown-enabled", "1.0.0")
        config(open_work, "3 days")
        verify(open_work, "cooldown-existing-lock", "1.1.0")
        if not any("/d9/ag/d9age" in path for path in requests):
            raise RuntimeError("local sparse index was not requested")
    finally:
        evidence.env = prior
        server.shutdown()
        server.server_close()
        thread.join()
        write(evidence.output / "loopback-requests.json", json.dumps(requests, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(schema=1, stage="D9-B", status="FAIL", toolchain=TOOLCHAIN,
                  git_commit=os.environ.get("GITHUB_SHA"), system=platform.system(),
                  source_before=before, learner_attainment="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-d9b-") as temp:
            scratch = Path(temp)
            evidence = Evidence(output, scratch)
            report["steps"] = evidence.steps
            evidence.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
            evidence.run("cargo-version", CARGO + ["--version"], scratch, contains="cargo 1.100.0 ")
            stdlib(evidence, scratch)
            cargo_practice(evidence, scratch)
            cooldown(evidence, scratch)
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
