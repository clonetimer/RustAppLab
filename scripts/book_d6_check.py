#!/usr/bin/env python3
"""Run D6 tools, Edition migration, or opt-in Nightly practice in a unique copy.

Install the requested toolchain/components explicitly before running this script.
PASS describes these commands only; IDE use and learner understanding are manual.
"""
from __future__ import annotations

import argparse
from datetime import date, datetime, timezone
import difflib
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time


ROOT = Path(__file__).resolve().parents[1]
LAB = Path("projects/book-d6-appendices")
COMMON = ("--locked", "--offline", "--workspace")

NIGHTLY_SOURCE = """#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

pub struct Square<const N: usize>
where
    [(); N * N]:,
{
    cells: [u8; N * N],
}

impl<const N: usize> Square<N>
where
    [(); N * N]:,
{
    pub fn zeroed() -> Self { Self { cells: [0; N * N] } }
    pub fn cells(&self) -> &[u8] { &self.cells }
}

#[cfg(test)]
mod tests {
    use super::Square;

    #[test]
    fn const_expression_sizes_storage() {
        assert_eq!(Square::<3>::zeroed().cells().len(), 9);
    }
}
"""


class PracticeFailure(RuntimeError):
    """A command or an evidence requirement did not pass."""


def source_snapshot(root: Path) -> dict:
    """Include absent toolchain files and additions/removals in the source lab."""
    paths = {root / name for name in
             ("VERSION", "Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml")}
    paths.update(path for path in (root / LAB).rglob("*")
                 if path.is_file() and not any(part in {"target", ".git", "__pycache__"}
                                               for part in path.relative_to(root / LAB).parts))
    result = {}
    for path in sorted(paths):
        key = path.relative_to(root).as_posix()
        if path.is_file():
            result[key] = hashlib.sha256(path.read_bytes()).hexdigest()
        else:
            result[key] = "NOT_A_FILE" if path.exists() else None
    return result


def snapshot_id(snapshot: dict) -> str:
    data = json.dumps(snapshot, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(data).hexdigest()


class Runner:
    def __init__(self, workspace: Path, output: Path, toolchain: str):
        self.workspace, self.output, self.toolchain = workspace, output, toolchain
        self.lab = workspace / "lab"
        self.manifest = self.lab / "appendix-lab/Cargo.toml"
        self.source = self.lab / "appendix-lab/src/lib.rs"
        self.steps = []
        self.before = {}
        self.env = os.environ.copy()
        self.env.pop("CARGO_BUILD_TARGET", None)
        self.env.update(CARGO_TARGET_DIR=str(workspace / "target"), CARGO_TERM_COLOR="never",
                        RUST_BACKTRACE="0")

    def run(self, label: str, command: list[str], *, expected: int = 0,
            diagnostic: str | None = None, minimum_tests: int | None = None) -> str:
        """Preserve raw combined stdout/stderr; never infer success from the last line."""
        print(f"[{label}] {' '.join(command)}", flush=True)
        start = time.monotonic()
        timed_out = False
        try:
            result = subprocess.run(command, cwd=self.lab, env=self.env,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    timeout=180, check=False)
            code, raw = result.returncode, result.stdout
        except subprocess.TimeoutExpired as error:
            code, raw, timed_out = 124, error.stdout or b"", True
        except OSError as error:
            code, raw = 127, (str(error) + "\n").encode("utf-8")
        log = self.output / (label + ".txt")
        log.write_bytes(raw)
        text = raw.decode("utf-8", errors="replace")
        passed = code == expected and not timed_out
        if diagnostic:
            passed = passed and re.search(diagnostic, text, re.IGNORECASE) is not None
        counts = None
        if minimum_tests is not None:
            suites = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
            counts = {"passed": sum(int(row[1]) for row in suites),
                      "failed": sum(int(row[2]) for row in suites),
                      "ignored": sum(int(row[3]) for row in suites)}
            passed = (passed and bool(suites) and counts["passed"] >= minimum_tests
                      and counts["failed"] == counts["ignored"] == 0
                      and all(row[0] == "ok" for row in suites))
        step = {"step": label, "argv": command, "cwd": str(self.lab), "returncode": code,
                "expected_returncode": expected, "passed": passed, "timed_out": timed_out,
                "seconds": round(time.monotonic() - start, 3), "log": log.name,
                "sha256": hashlib.sha256(raw).hexdigest()}
        if diagnostic:
            step["required_diagnostic"] = diagnostic
        if counts is not None:
            step["tests"] = counts
        self.steps.append(step)
        if not passed:
            raise PracticeFailure(f"{label} failed (exit {code}, expected {expected}); see {log}")
        return text

    def cargo(self, label: str, command: str, *arguments: str,
              toolchain: str | None = None, **checks) -> str:
        return self.run(label, ["cargo", "+" + (toolchain or self.toolchain), command,
                               "--manifest-path", str(self.manifest), *arguments], **checks)

    def capture_before(self):
        self.before = {"manifest": self.manifest.read_text(encoding="utf-8"),
                       "source": self.source.read_text(encoding="utf-8")}

    def save_evidence(self):
        diffs = []
        for label, path in (("manifest", self.manifest), ("source", self.source)):
            if label not in self.before or not path.is_file():
                continue
            before, after = self.before[label], path.read_text(encoding="utf-8")
            for state, content in (("before", before), ("after", after)):
                (self.output / f"{label}-{state}.txt").write_bytes(content.encode("utf-8"))
            diffs.extend(difflib.unified_diff(before.splitlines(keepends=True),
                                             after.splitlines(keepends=True),
                                             fromfile=f"before/{path.name}", tofile=f"after/{path.name}"))
        (self.output / "changes.diff.txt").write_bytes("".join(diffs).encode("utf-8"))


def compiler(runner: Runner, toolchain: str, *, nightly: bool = False,
             label: str = "rustc-version") -> tuple[int, int, int]:
    output = runner.run(label, ["rustc", "+" + toolchain, "-Vv"])
    match = re.search(r"^release: (\d+)\.(\d+)\.(\d+)([^\s]*)", output, re.MULTILINE)
    if not match or (nightly and "nightly" not in match[4]) or (not nightly and match[4]):
        raise PracticeFailure(f"{toolchain} did not identify as the required {'Nightly' if nightly else 'Stable'} compiler")
    return tuple(int(match[index]) for index in (1, 2, 3))


def quality(runner: Runner, *, minimum_tests: int = 11, docs: bool = False):
    runner.cargo("format", "fmt", "--all")
    runner.cargo("format-check", "fmt", "--all", "--", "--check")
    runner.cargo("clippy", "clippy", *COMMON, "--all-targets", "--", "-D", "warnings")
    runner.cargo("debug-tests", "test", *COMMON, "--all-targets", minimum_tests=minimum_tests)
    runner.cargo("doctests", "test", *COMMON, "--doc", minimum_tests=3)
    runner.cargo("release-tests", "test", *COMMON, "--all-targets", "--release",
                 minimum_tests=minimum_tests)
    if docs:
        runner.cargo("documentation", "doc", *COMMON, "--no-deps")
        if not (runner.workspace / "target/doc/book_d6_appendix_lab/index.html").is_file():
            raise PracticeFailure("cargo doc did not produce the expected crate documentation")


def tool_versions(runner: Runner, *, analyzer: bool = False) -> tuple[int, int, int]:
    version = compiler(runner, runner.toolchain)
    for label, command in (
            ("cargo-version", ["cargo", "+" + runner.toolchain, "--version"]),
            ("rustfmt-version", ["rustfmt", "+" + runner.toolchain, "--version"]),
            ("clippy-version", ["cargo", "+" + runner.toolchain, "clippy", "--version"]),
            ("rustdoc-version", ["rustdoc", "+" + runner.toolchain, "--version"])):
        runner.run(label, command)
    if analyzer:
        runner.run("rust-analyzer-version", ["rustup", "run", runner.toolchain, "rust-analyzer", "--version"])
    return version


def check_tools(runner: Runner):
    tool_versions(runner, analyzer=True)
    probe = "\npub fn d6_unused_mut_probe() -> u8 {\n    let mut value = 7_u8;\n    value.saturating_add(1)\n}\n"
    source = runner.source.read_text(encoding="utf-8")
    marker = "\n#[cfg(test)]\nmod tests {"
    if source.count(marker) != 1:
        raise PracticeFailure("cannot place the tools probe before the original test module")
    runner.source.write_bytes(source.replace(marker, probe + marker, 1).encode("utf-8"))
    runner.capture_before()
    runner.cargo("unused-mut-diagnostic", "clippy", *COMMON, "--all-targets", "--", "-D", "warnings",
                 expected=101, diagnostic=r"unused[_-]mut|does not need to be mutable")
    runner.cargo("repair-unused-mut", "fix", *COMMON, "--all-targets", "--allow-no-vcs")
    if "let mut value = 7_u8;" in runner.source.read_text(encoding="utf-8"):
        raise PracticeFailure("cargo fix did not repair the unused_mut probe")
    quality(runner, docs=True)


def check_edition(runner: Runner):
    if tool_versions(runner) < (1, 85, 0):
        raise PracticeFailure("Edition 2024 requires Rust 1.85.0 or newer")
    runner.cargo("original-tests", "test", *COMMON, "--all-targets", minimum_tests=11)
    probe = """
pub fn gen() -> u32 { 2024 }

#[cfg(test)]
mod d6_edition_probe {
    #[test]
    fn preserves_reserved_identifier() {
        assert_eq!(super::gen(), 2024);
    }
}
"""
    runner.source.write_bytes(runner.source.read_bytes() + probe.encode("utf-8"))
    runner.capture_before()
    runner.cargo("edition-2021-probe", "test", *COMMON, "--all-targets", minimum_tests=12)
    runner.cargo("edition-fix", "fix", *COMMON, "--all-targets", "--edition", "--allow-no-vcs")
    fixed = runner.source.read_text(encoding="utf-8")
    if "fn r#gen()" not in fixed or "super::r#gen()" not in fixed:
        raise PracticeFailure("cargo fix --edition did not rewrite the gen definition and call")
    manifest = runner.manifest.read_text(encoding="utf-8")
    if any(manifest.splitlines().count(field) != 1 for field in
           ('edition = "2021"', 'rust-version = "1.74"')):
        raise PracticeFailure("the copy must still use Edition 2021 / Rust 1.74 until cargo fix finishes")
    manifest = manifest.replace('edition = "2021"', 'edition = "2024"', 1)
    manifest = manifest.replace('rust-version = "1.74"', 'rust-version = "1.85"', 1)
    runner.manifest.write_bytes(manifest.encode("utf-8"))
    quality(runner, minimum_tests=12)


def check_nightly(runner: Runner, stable_toolchain: str):
    runner.manifest = runner.lab / "Cargo.toml"
    runner.source = runner.lab / "src/lib.rs"
    runner.manifest.write_text('[package]\nname = "nightly-isolation-lab"\nversion = "0.1.0"\n'
                               'edition = "2021"\npublish = false\n\n[workspace]\n', encoding="utf-8")
    runner.source.write_bytes(NIGHTLY_SOURCE.encode("utf-8"))
    runner.capture_before()
    compiler(runner, runner.toolchain, nightly=True)
    compiler(runner, stable_toolchain, label="stable-rustc-version")
    runner.cargo("nightly-lockfile", "generate-lockfile", "--offline")
    runner.cargo("nightly-tests", "test", *COMMON, "--all-targets", minimum_tests=1)
    runner.cargo("stable-gate-rejected", "check", *COMMON, "--all-targets",
                 toolchain=stable_toolchain, expected=101, diagnostic=r"E0554")
    ungated = NIGHTLY_SOURCE.replace("#![feature(generic_const_exprs)]\n", "", 1)
    (runner.output / "source-without-feature-gate.txt").write_bytes(ungated.encode("utf-8"))
    runner.source.write_bytes(ungated.encode("utf-8"))
    runner.cargo("stable-expression-rejected", "check", *COMMON, "--all-targets",
                 toolchain=stable_toolchain, expected=101,
                 diagnostic=r"generic parameters may not be used in const operations")
    runner.source.write_bytes(NIGHTLY_SOURCE.encode("utf-8"))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", required=True, choices=("tools", "edition", "nightly"))
    parser.add_argument("--toolchain", help="default: 1.85.0, or nightly-2026-11-17 for Nightly")
    parser.add_argument("--stable-toolchain", default="1.85.0")
    parser.add_argument("--output", type=Path, required=True, help="new evidence directory; never overwritten")
    parser.add_argument("--keep-workspace", action="store_true", help="retain a successful copy for IDE practice")
    args = parser.parse_args(argv)
    toolchain = args.toolchain or ("nightly-2026-11-17" if args.mode == "nightly" else "1.85.0")
    output = args.output.resolve()
    source_lab = (ROOT / LAB).resolve()
    try:
        if output == source_lab or source_lab in output.parents:
            raise PracticeFailure("evidence output must be outside the original reference lab")
        output.mkdir(parents=True, exist_ok=False)
    except (OSError, PracticeFailure) as error:
        print(f"D6 refused: {error}", file=sys.stderr)
        return 2

    report = {"schema": 1, "mode": args.mode, "toolchain": toolchain,
              "stable_toolchain": args.stable_toolchain, "system": platform.system(),
              "git_commit": os.environ.get("GITHUB_SHA"),
              "runner_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "started_at": datetime.now(timezone.utc).isoformat(), "status": "FAIL",
              "scope": "D6 command practice; IDE use and learner mastery are not automated",
              "snapshot_scope": "VERSION, root manifests/toolchain files and projects/book-d6-appendices",
              "workspace": None, "workspace_retained": False, "steps": [], "error": None}
    before = source_snapshot(ROOT)
    report.update(source_snapshot_before=before, source_id=snapshot_id(before))
    runner = None
    try:
        if os.environ.get("RUSTC_BOOTSTRAP"):
            raise PracticeFailure("unset RUSTC_BOOTSTRAP; it invalidates the Stable/Nightly controls")
        if args.mode == "nightly":
            if not re.fullmatch(r"nightly-\d{4}-\d{2}-\d{2}", toolchain):
                raise PracticeFailure("Nightly requires a dated nightly-YYYY-MM-DD toolchain")
            date.fromisoformat(toolchain.removeprefix("nightly-"))
        workspace = Path(tempfile.mkdtemp(prefix="RustAppLab-D6-"))
        report["workspace"] = str(workspace)
        runner = Runner(workspace, output, toolchain)
        report["steps"] = runner.steps
        if args.mode == "nightly":
            (runner.lab / "src").mkdir(parents=True)
            check_nightly(runner, args.stable_toolchain)
        else:
            shutil.copytree(source_lab, runner.lab,
                            ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
            runner.capture_before()
            {"tools": check_tools, "edition": check_edition}[args.mode](runner)
        report["status"] = "PASS"
    except Exception as error:
        report["error"] = str(error)
    finally:
        try:
            if runner:
                runner.save_evidence()
            after = source_snapshot(ROOT)
            changed = sorted(key for key in before.keys() | after.keys() if before.get(key) != after.get(key))
            report.update(source_snapshot_after=after, source_id_after=snapshot_id(after),
                          changed_source_paths=changed, source_unchanged=not changed)
            if changed:
                report["status"] = "FAIL"
                report["error"] = "Original source changed during practice: " + ", ".join(changed)
            report["evidence"] = [{"file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
                                  for path in sorted(output.iterdir()) if path.is_file()]
            report["workspace_retained"] = bool(runner and runner.workspace.exists())
            report_path = output / "report.json"
            report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            if report["status"] == "PASS" and runner and not args.keep_workspace:
                shutil.rmtree(runner.workspace)
                report["workspace_retained"] = False
                report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        except Exception as error:
            report["status"], report["error"] = "FAIL", str(error)
            (output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{report['status']}: {args.mode}; report: {output / 'report.json'}")
    if report["workspace_retained"]:
        print(f"Retained workspace: {report['workspace']}")
    if report["error"]:
        print(report["error"], file=sys.stderr)
    return 0 if report["status"] == "PASS" else 2


if __name__ == "__main__":
    raise SystemExit(main())
