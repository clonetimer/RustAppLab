#!/usr/bin/env python3
"""D8 real fixed-toolchain experiments; no learner-source or repository mutation."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
LAB = ROOT / "projects/modern-d8-edition"
TOOLCHAIN = "1.100.0"
CARGO = ["cargo", "+" + TOOLCHAIN]
RUSTC = ["rustc", "+" + TOOLCHAIN]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def snapshot() -> dict[str, str]:
    files = [p for p in LAB.rglob("*") if p.is_file() and
             not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB).parts)]
    files.extend([ROOT / "course/BOOK_D8_EDITION_2024.md", Path(__file__).resolve()])
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(files)}


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8", newline="\n")


class Evidence:
    def __init__(self, output: Path, scratch: Path):
        self.output = output
        self.steps: list[dict] = []
        self.env = os.environ.copy()
        for key in list(self.env):
            if (key.startswith(("CARGO_RESOLVER_", "CARGO_BUILD_", "CARGO_TARGET_",
                                "CARGO_PROFILE_")) or key in
                    {"RUSTC", "RUSTDOC", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
                     "RUSTUP_TOOLCHAIN", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"}):
                self.env.pop(key, None)
        self.env["CARGO_HOME"] = str(scratch / "cargo-home")
        self.env["CARGO_NET_OFFLINE"] = "true"

    def run(self, name, command, cwd, expected=0, contains=None, stderr=None):
        command = [str(x) for x in command]
        process = subprocess.run(command, cwd=cwd, env=self.env,
                                 capture_output=True, timeout=180)
        out = process.stdout.decode("utf-8", errors="replace").replace("\r\n", "\n")
        err = process.stderr.decode("utf-8", errors="replace").replace("\r\n", "\n")
        log = self.output / (name + ".txt")
        write(log, json.dumps(command, ensure_ascii=False) + "\n" + out + "\n--- stderr ---\n" + err)
        passed = (process.returncode == expected and
                  (contains is None or contains in out) and
                  (stderr is None or stderr in err))
        self.steps.append(dict(step=name, command=command, returncode=process.returncode,
                               expected_returncode=expected, log=log.name,
                               sha256=digest(log), passed=passed))
        print(f"{name}: {'PASS' if passed else 'FAIL'} ({process.returncode})", flush=True)
        if not passed:
            raise RuntimeError(f"{name}: unexpected compiler or program behavior; see {log}")
        return out, err


def compile_pair(evidence: Evidence, scratch: Path, name: str, code: str,
                 fail_edition: str, diagnostic: str, corrected: str) -> None:
    """Compile both original editions, then execute the repaired 2024 program."""
    original = scratch / (name + ".rs")
    fixed = scratch / (name + "_fixed.rs")
    write(original, code)
    write(fixed, corrected)
    for edition in ("2021", "2024"):
        exe = scratch / (name + edition + (".exe" if os.name == "nt" else ""))
        if edition == fail_edition:
            evidence.run(name + "-" + edition + "-red",
                         RUSTC + ["--edition=" + edition, str(original), "-o", str(exe)],
                         scratch, expected=1, stderr=diagnostic)
        else:
            evidence.run(name + "-" + edition + "-build",
                         RUSTC + ["--edition=" + edition, str(original), "-o", str(exe)], scratch)
            evidence.run(name + "-" + edition + "-run", [exe], scratch, contains="ok")
    exe = scratch / (name + "-fixed" + (".exe" if os.name == "nt" else ""))
    evidence.run(name + "-fixed-build",
                 RUSTC + ["--edition=2024", str(fixed), "-o", str(exe)], scratch)
    evidence.run(name + "-fixed-run", [exe], scratch, contains="ok")


def migration(evidence: Evidence, scratch: Path) -> None:
    work = scratch / "migration"
    manifest = work / "Cargo.toml"
    old = '[package]\nname = "d8-migrate"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n'
    write(manifest, old)
    main = work / "src/main.rs"
    write(main, 'fn main() { let gen = 7; println!("{gen}"); }\n')
    evidence.run("migration-lock", CARGO + ["generate-lockfile", "--offline"], work)
    evidence.run("migration-before", CARGO + ["run", "--locked", "--offline"], work, contains="7")
    evidence.run("migration-fix", CARGO + ["fix", "--edition", "--locked", "--offline",
                                           "--allow-no-vcs"], work)
    repaired = main.read_text(encoding="utf-8")
    if "r#gen" not in repaired:
        raise RuntimeError("cargo fix did not repair the reserved identifier")
    write(evidence.output / "migration-before.rs.txt", 'fn main() { let gen = 7; println!("{gen}"); }\n')
    write(evidence.output / "migration-after.rs.txt", repaired)
    write(manifest, old.replace('edition = "2021"', 'edition = "2024"'))
    evidence.run("migration-after", CARGO + ["run", "--locked", "--offline"], work, contains="7")


def resolver(evidence: Evidence, scratch: Path) -> None:
    for version, selected in (("2", "1.1.0"), ("3", "1.0.0")):
        work = scratch / ("resolver-" + version)
        write(work / "Cargo.toml", '[package]\nname = "d8-resolver-consumer"\n'
              'version = "0.1.0"\nedition = "2024"\nrust-version = "1.100"\npublish = false\n'
              '[workspace]\nresolver = "' + version + '"\n'
              '[dependencies]\nd8-resolver-probe = "1"\n')
        write(work / "src/main.rs", 'fn main() { println!("{}", d8_resolver_probe::version()); }\n')
        write(work / ".cargo/config.toml", '[source.crates-io]\nreplace-with = "d8-local"\n'
              '[source.d8-local]\ndirectory = "vendor"\n')
        # Synthetic directory source, no external crate/download. Version 1.1
        # intentionally requires a future compiler; it is not a published crate.
        for dep_version, msrv in (("1.0.0", "1.100"), ("1.1.0", "1.101")):
            dep = work / "vendor" / ("d8-resolver-probe-" + dep_version)
            write(dep / "Cargo.toml", '[package]\nname = "d8-resolver-probe"\nversion = "'
                  + dep_version + '"\nedition = "2024"\nrust-version = "' + msrv + '"\n')
            write(dep / "src/lib.rs", 'pub fn version() -> &\'static str { "' + dep_version + '" }\n')
            hashes = {p: digest(dep / p) for p in ("Cargo.toml", "src/lib.rs")}
            checksum = hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest()
            write(dep / ".cargo-checksum.json", json.dumps({"files": hashes, "package": checksum}))
        evidence.run("resolver-" + version + "-lock", CARGO + ["generate-lockfile", "--offline"], work)
        lock = work / "Cargo.lock"
        parsed = tomllib.loads(lock.read_text(encoding="utf-8"))
        actual = [p["version"] for p in parsed["package"] if p["name"] == "d8-resolver-probe"]
        if actual != [selected]:
            raise RuntimeError(f"resolver {version}: expected {selected}, got {actual}")
        shutil.copyfile(lock, evidence.output / ("resolver-" + version + ".lock.txt"))
        if version == "2":
            evidence.run("resolver-2-msrv-rejection", CARGO + ["check", "--locked", "--offline"],
                         work, expected=101, stderr="1.101")
        else:
            evidence.run("resolver-3-compatible-run", CARGO + ["run", "--locked", "--offline"],
                         work, contains="1.0.0")


def experiments(evidence: Evidence, scratch: Path) -> None:
    evidence.run("rustc-version", RUSTC + ["--version"], scratch, contains="rustc 1.100.0 ")
    evidence.run("cargo-version", CARGO + ["--version"], scratch, contains="cargo 1.100.0 ")
    reference = scratch / "reference"
    learner = scratch / "learner"
    for name, dest in (("reference", reference), ("learner", learner)):
        shutil.copytree(LAB / name, dest, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    common = ["--locked", "--offline"]
    evidence.run("reference-debug", CARGO + ["test"] + common + ["--workspace", "--all-targets"], reference)
    evidence.run("reference-release", CARGO + ["test"] + common + ["--release", "--workspace", "--all-targets"], reference)
    for edition, order, kind in (("2021", "else,drop", "const"), ("2024", "drop,else", "expr")):
        out, _ = evidence.run("edition-" + edition, CARGO + ["run"] + common +
                              ["-p", "d8-edition" + edition], reference)
        if out.strip().splitlines() != ["drop=" + order, "macro=" + kind]:
            raise RuntimeError(f"edition {edition}: unexpected observation {out!r}")
    # Each learner task must independently fail before solution injection.
    for step in ("step1_", "step2_", "step3_"):
        evidence.run("learner-red-" + step, CARGO + ["test"] + common +
                     ["--test", "contracts", step], learner, expected=101,
                     contains="test result: FAILED")
    if (learner / "tests/contracts.rs").read_bytes() != (reference / "e2024/tests/contracts.rs").read_bytes():
        raise RuntimeError("learner and reference contracts diverge")
    # Copy the solution only into this temporary learner project; keep fixed tests.
    shutil.copyfile(reference / "e2024/src/lib.rs", learner / "src/lib.rs")
    evidence.run("learner-green", CARGO + ["test"] + common + ["--all-targets"], learner)
    for name, args, code, expected in (
        ("alarm", ["45", "40"], 0, "alarm:45"),
        ("boundary", ["40", "40"], 0, "alarm:40"),
        ("missing", ["missing", "40"], 0, "no alarm"),
        ("negative", ["-1", "-2"], 0, "no alarm"),
        ("invalid", ["bad", "40"], 1, None),
    ):
        evidence.run("cli-" + name, CARGO + ["run"] + common +
                     ["--example", "alarm", "--"] + args, learner, expected=code,
                     contains=expected, stderr="invalid reading" if code else None)
    compile_pair(evidence, scratch, "gen_identifier",
                 'fn main() { let gen = 7; assert_eq!(gen, 7); println!("ok"); }\n',
                 "2024", "reserved keyword",
                 'fn main() { let r#gen = 7; assert_eq!(r#gen, 7); println!("ok"); }\n')
    let_chain = 'fn main() { if let Some(x) = Some(7) && x > 0 { println!("ok"); } }\n'
    compile_pair(evidence, scratch, "let_chain", let_chain, "2021", "let chains", let_chain)
    compile_pair(evidence, scratch, "extern_block",
                 'extern "C" { fn d8_unused_external_symbol(); }\nfn main() { println!("ok"); }\n',
                 "2024", "extern blocks must be unsafe",
                 'unsafe extern "C" { fn d8_unused_external_symbol(); }\nfn main() { println!("ok"); }\n')
    migration(evidence, scratch)
    resolver(evidence, scratch)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = snapshot()
    report = dict(schema=1, status="FAIL", toolchain=TOOLCHAIN,
                  git_commit=os.environ.get("GITHUB_SHA"), system=platform.system(),
                  source_before=before, learner_attainment="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-d8-") as temporary:
            scratch = Path(temporary)
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
    report["source_unchanged"] = report["source_before"] == report["source_after"]
    write(output / "report.json", json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    return 0 if report["status"] == "PASS" and report["source_unchanged"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
