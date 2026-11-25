#!/usr/bin/env python3
"""D10-B: locked vendor, real Tokio contracts and relocated empty-cache learner ZIP."""
from __future__ import annotations

import argparse
import json
import os
import platform
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import zipfile

from modern_d8_check import Evidence, CARGO, RUSTC, TOOLCHAIN, ROOT, digest, write

LAB = ROOT / "projects/modern-d10-tokio"
DIAGNOSTICS = {"stream_pin":"E0277","trait_send":"future cannot be sent between threads safely"}
CONFIG = '[source.crates-io]\nreplace-with = "d10-vendor"\n[source.d10-vendor]\ndirectory = "vendor"\n'


def snapshot():
    paths = [p for p in LAB.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB).parts)]
    paths.extend([ROOT / "course/BOOK_D10_TOKIO_STREAM.md",
                  ROOT / "scripts/modern_d8_check.py", ROOT / "LICENSE",
                  Path(__file__).resolve()])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(paths)}


def copy_pristine(side, destination, vendor):
    shutil.copytree(LAB / side, destination,
                    ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    shutil.copytree(vendor, destination / "vendor")
    write(destination / ".cargo/config.toml", CONFIG)


def pack_learner(stage, archive):
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as output:
        for p in sorted(stage.rglob("*"), key=lambda p: p.relative_to(stage).as_posix()):
            if p.is_file():
                name = p.relative_to(stage).as_posix()
                info = zipfile.ZipInfo(name, date_time=(2026, 11, 25, 0, 0, 0))
                info.create_system = 3
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                output.writestr(info, p.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=6)
    with zipfile.ZipFile(archive) as result:
        if result.testzip() is not None:
            raise RuntimeError("corrupt learner archive")
        names = set(result.namelist())
        if ".cargo/config.toml" not in names or not any(n.endswith("/.cargo-checksum.json") for n in names):
            raise RuntimeError("vendor not included in learner archive")
        if any(n.startswith(("reference/", "probes/", "target/")) for n in names):
            raise RuntimeError("reference or build output leaked into learner archive")
        if result.read("src/lib.rs") != (LAB / "learner/src/lib.rs").read_bytes():
            raise RuntimeError("learner archive source differs from TODO template")
    return digest(archive)


def verify_vendor_resolution(evidence, label, work, home):
    """Frozen resolution must point to the bundled directory source, not a cache."""
    output, _ = evidence.run(label + "-metadata-frozen",
                             CARGO + ["metadata", "--frozen", "--format-version", "1"], work)
    metadata = json.loads(output)
    expected = {(p["name"], p["version"]) for p in
                tomllib.loads((work / "Cargo.lock").read_text(encoding="utf-8"))["package"]
                if p.get("source")}
    registry_packages = [p for p in metadata["packages"] if p.get("source")]
    observed = {(p["name"], p["version"]) for p in registry_packages}
    if not {("tokio", "1.48.0"), ("futures-util", "0.3.34")} <= observed <= expected:
        raise RuntimeError(label + ": unexpected dependency metadata")
    vendor = (work / "vendor").resolve()
    locations = []
    for package in registry_packages:
        manifest = Path(package["manifest_path"]).resolve()
        if not manifest.is_relative_to(vendor) or not manifest.is_file():
            raise RuntimeError(label + ": dependency not resolved from shipped vendor: " + str(manifest))
        locations.append(dict(name=package["name"], version=package["version"],
                              manifest=manifest.relative_to(work.resolve()).as_posix()))
    registry = home / "registry"
    entries = []
    unexpected = []
    if registry.exists():
        for path in sorted(registry.rglob("*")):
            relative = path.relative_to(registry).as_posix()
            if path.is_file():
                # Cargo may create a cache-directory marker even when all
                # dependencies are read from a directory source. It is not a
                # registry index, downloaded crate, or extracted package.
                marker = (relative == "CACHEDIR.TAG" and
                          path.read_bytes().startswith(b"Signature: 8a477f597d28d172789f06886806bc55"))
                entries.append(dict(path=relative, kind="file", bytes=path.stat().st_size,
                                    sha256=digest(path), cache_marker=marker))
                if not marker:
                    unexpected.append(relative)
            elif path.is_dir():
                entries.append(dict(path=relative, kind="directory"))
            else:
                unexpected.append(relative)
    write(evidence.output / (label + "-vendor-resolution.json"),
          json.dumps(dict(registry_entries=entries, vendor_packages=locations,
                          unexpected_registry_entries=unexpected), ensure_ascii=False, indent=2) + "\n")
    if unexpected:
        raise RuntimeError(label + ": frozen replay populated registry data: " + repr(unexpected))
    return len(locations)


def experiments(evidence, scratch, report):
    evidence.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
    evidence.run("cargo-version", CARGO + ["--version"], scratch, contains="cargo 1.100.0 ")
    vendor = scratch / "vendor"
    # Explicit preparation is the only network phase. All compilation below uses
    # the locked local directory source, even with an empty registry cache.
    evidence.env["CARGO_NET_OFFLINE"] = "false"
    evidence.run("vendor-locked", CARGO + ["vendor", "--locked", "--versioned-dirs",
                 "--manifest-path", str(LAB / "reference/Cargo.toml"), str(vendor)], scratch)
    evidence.env["CARGO_NET_OFFLINE"] = "true"
    reference = scratch / "reference"
    stage = scratch / "pristine-learner"
    copy_pristine("reference", reference, vendor)
    copy_pristine("learner", stage, vendor)
    shutil.copyfile(ROOT / "LICENSE", stage / "LICENSE")
    archive = evidence.output / "D10-B-learner-offline.zip"
    archive_hash = pack_learner(stage, archive)
    write(archive.with_suffix(".zip.sha256"), f"{archive_hash}  {archive.name}\n")
    learner = scratch / "搬移 学员 空缓存"
    learner.mkdir()
    with zipfile.ZipFile(archive) as source:
        source.extractall(learner)
    # Separate empty Cargo homes for reference and relocated learner.
    ref_home = scratch / "empty-reference-cargo"
    ref_home.mkdir()
    if any(ref_home.iterdir()):
        raise RuntimeError("reference Cargo home is not initially empty")
    evidence.env["CARGO_HOME"] = str(ref_home)
    common = ["--frozen"]
    evidence.run("reference-debug", CARGO + ["test"] + common + ["--all-targets"],
                 reference, contains="7 passed; 0 failed; 0 ignored")
    evidence.run("reference-release", CARGO + ["test"] + common + ["--release", "--all-targets"],
                 reference, contains="7 passed; 0 failed; 0 ignored")
    for name, diagnostic in DIAGNOSTICS.items():
        example = reference / "examples/diagnostic_probe.rs"
        shutil.copyfile(reference / "probes" / name / "red.rs", example)
        evidence.run("diagnostic-" + name, CARGO + ["check"] + common +
                     ["--example", "diagnostic_probe"], reference, expected=101, stderr=diagnostic)
        shutil.copyfile(reference / "probes" / name / "green.rs", example)
        evidence.run("repair-" + name, CARGO + ["run"] + common +
                     ["--example", "diagnostic_probe"], reference, contains="ok")
        example.unlink()

    reference_vendor_count = verify_vendor_resolution(evidence, "reference", reference, ref_home)
    home = scratch / "empty-learner-cargo"
    home.mkdir()
    if any(home.iterdir()):
        raise RuntimeError("learner Cargo home is not initially empty")
    evidence.env["CARGO_HOME"] = str(home)
    fixed = learner / "tests/contracts.rs"
    fixed_hash = digest(fixed)
    lock_hash = digest(learner / "Cargo.lock")
    if fixed_hash != digest(reference / "tests/contracts.rs"):
        raise RuntimeError("fixed contract mismatch")
    evidence.run("learner-compiles-empty-cache", CARGO + ["check"] + common + ["--all-targets"], learner)
    for number in range(1, 8):
        evidence.run(f"learner-red-{number}", CARGO + ["test"] + common +
                     ["--test", "contracts", f"step{number}_"], learner,
                     expected=101, contains="test result: FAILED")
    shutil.copyfile(reference / "src/lib.rs", learner / "src/lib.rs")
    evidence.run("learner-green", CARGO + ["test"] + common + ["--all-targets"],
                 learner, contains="7 passed; 0 failed; 0 ignored")
    output, _ = evidence.run("cli", CARGO + ["run"] + common + ["--example", "async_workbench"], learner)
    expected = ["joined=(7, 9)", "positive=[2, 3, 4, 5]; sent=3", "ordered=[9, 1, 4]",
                "race=Cancelled", "blocking=42", "sample=Some(31)"]
    if output.strip().splitlines() != expected:
        raise RuntimeError("CLI output differs: " + output)
    if fixed_hash != digest(fixed) or lock_hash != digest(learner / "Cargo.lock"):
        raise RuntimeError("fixed contract or dependency lock changed")
    if archive_hash != digest(archive):
        raise RuntimeError("shipped TODO ZIP was modified")
    learner_vendor_count = verify_vendor_resolution(evidence, "learner", learner, home)
    deps = [p for p in tomllib.loads((LAB / "reference/Cargo.lock").read_text(encoding="utf-8"))["package"]
            if p.get("source")]
    report["portable"] = dict(file=archive.name, sha256=archive_hash,
                              bytes=archive.stat().st_size, registry_dependencies=len(deps),
                              empty_cache_relocated_replay="PASS", fixed_contract_red_then_green=7,
                              reference_vendor_packages=reference_vendor_count,
                              learner_vendor_packages=learner_vendor_count)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(schema=1, stage="D10-B", status="FAIL", toolchain=TOOLCHAIN,
                  git_commit=os.environ.get("GITHUB_SHA"), system=platform.system(),
                  source_before=before, learner_attainment="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-d10b-") as temp:
            scratch = Path(temp)
            evidence = Evidence(output, scratch)
            report["steps"] = evidence.steps
            experiments(evidence, scratch, report)
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
