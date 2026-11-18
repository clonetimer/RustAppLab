"""Behavior checks for isolated D6 practice; no installed Rust is required here."""
from contextlib import redirect_stderr, redirect_stdout
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/book_d6_check.py"
practice = None
if SCRIPT.is_file():
    spec = importlib.util.spec_from_file_location("book_d6_check", SCRIPT)
    practice = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(practice)


class PracticalChecks(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(practice, "the executable D6 practice runner is missing")
        self.temporary = tempfile.TemporaryDirectory(prefix="D6 test paths ")
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.root = self.base / "source tree"
        self.root.mkdir()
        (self.root / "Cargo.toml").write_text('[workspace]\nmembers = []\n', encoding="utf-8")
        (self.root / "Cargo.lock").write_text('version = 3\n', encoding="utf-8")
        self.original = self.root / "projects/book-d6-appendices"
        shutil.copytree(ROOT / "projects/book-d6-appendices", self.original,
                        ignore=shutil.ignore_patterns("target", "__pycache__"))
        self.source = self.original / "appendix-lab/src/lib.rs"
        self.source_bytes = self.source.read_bytes()
        self.output = self.base / "evidence with spaces"
        self.root_patch = mock.patch.object(practice, "ROOT", self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)
        original_mkdtemp = tempfile.mkdtemp
        self.temp_patch = mock.patch.object(
            practice.tempfile, "mkdtemp",
            side_effect=lambda **kwargs: original_mkdtemp(dir=self.base, **kwargs))
        self.temp_patch.start()
        self.addCleanup(self.temp_patch.stop)
        self.env_patch = mock.patch.dict(os.environ)
        self.env_patch.start()
        self.addCleanup(self.env_patch.stop)
        os.environ.pop("RUSTC_BOOTSTRAP", None)

    def invoke(self, *extra):
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            return practice.main(["--mode", "tools", "--output", str(self.output), *extra])

    def report(self):
        return json.loads((self.output / "report.json").read_text(encoding="utf-8"))

    def runner(self):
        self.output.mkdir()
        workspace = self.base / "isolated workspace"
        (workspace / "lab").mkdir(parents=True)
        return practice.Runner(workspace, self.output, "1.85.0")

    def test_existing_output_is_not_overwritten(self):
        self.output.mkdir()
        sentinel = self.output / "report.json"
        sentinel.write_bytes(b"previous evidence must survive")
        self.assertNotEqual(self.invoke(), 0)
        self.assertEqual(sentinel.read_bytes(), b"previous evidence must survive")
        self.assertEqual(list(self.output.iterdir()), [sentinel])

    def test_output_inside_original_lab_is_rejected_without_writing(self):
        self.output = self.original / "evidence"
        self.assertNotEqual(self.invoke(), 0)
        self.assertFalse(self.output.exists())
        self.assertEqual(self.source.read_bytes(), self.source_bytes)

    def test_output_cannot_create_an_absent_protected_root_file_as_a_directory(self):
        for name in ("VERSION", "rust-toolchain", "rust-toolchain.toml"):
            with self.subTest(name=name):
                protected = self.root / name
                self.assertFalse(protected.exists())
                self.output = protected / "logs"
                with mock.patch.object(practice, "check_tools", return_value=None):
                    self.assertNotEqual(self.invoke(), 0)
                self.assertFalse(protected.exists(), "rejection must happen before mkdir")

    def test_command_preserves_spaces_stderr_exit_code_and_isolated_target(self):
        runner = self.runner()
        command = [sys.executable, "-c",
                   "import os,sys; print(sys.argv[1]); "
                   "print(os.environ['CARGO_TARGET_DIR']); "
                   "print('actual stderr', file=sys.stderr); sys.exit(7)",
                   str(self.base / "an argument with spaces")]
        with self.assertRaises(practice.PracticeFailure):
            runner.run("failure", command)
        step = runner.steps[-1]
        log = (self.output / step["log"]).read_bytes()
        self.assertEqual(step["returncode"], 7)
        self.assertFalse(step["passed"])
        self.assertIn(b"an argument with spaces", log)
        self.assertIn(b"actual stderr", log)
        self.assertIn(str(runner.workspace / "target").encode(), log)
        self.assertEqual(step["sha256"], hashlib.sha256(log).hexdigest())

    def test_failure_saves_report_and_keeps_copy_without_changing_source(self):
        def fail(runner):
            (runner.lab / "appendix-lab/src/lib.rs").write_text("temporary edit", encoding="utf-8")
            runner.run("native-failure", [sys.executable, "-c", "print('native failure'); exit(9)"])

        with mock.patch.object(practice, "check_tools", side_effect=fail):
            self.assertNotEqual(self.invoke(), 0)
        report = self.report()
        self.assertEqual(report["status"], "FAIL")
        self.assertTrue(Path(report["workspace"]).is_dir())
        self.assertEqual(report["steps"][0]["returncode"], 9)
        self.assertIn("native failure", (self.output / report["steps"][0]["log"]).read_text())
        self.assertEqual(self.source.read_bytes(), self.source_bytes)
        self.assertEqual(report["source_snapshot_before"], report["source_snapshot_after"])

    def test_success_cleans_only_its_unique_workspace(self):
        unrelated = self.base / "RustAppLab-D6-Edition-2024"
        unrelated.mkdir()
        (unrelated / "keep.txt").write_text("previous work", encoding="utf-8")
        with mock.patch.object(practice, "check_tools", return_value=None):
            self.assertEqual(self.invoke(), 0)
        report = self.report()
        self.assertEqual(report["status"], "PASS")
        self.assertFalse(Path(report["workspace"]).exists())
        self.assertFalse(report["workspace_retained"])
        self.assertEqual((unrelated / "keep.txt").read_text(), "previous work")
        self.assertTrue((self.output / "report.json").is_file())

    def test_keep_workspace_exposes_only_a_copy_for_ide(self):
        def edit_copy(runner):
            (runner.lab / "appendix-lab/src/lib.rs").write_text("IDE exercise", encoding="utf-8")

        with mock.patch.object(practice, "check_tools", side_effect=edit_copy):
            self.assertEqual(self.invoke("--keep-workspace"), 0)
        report = self.report()
        self.assertTrue(report["workspace_retained"])
        self.assertEqual((Path(report["workspace"]) / "lab/appendix-lab/src/lib.rs").read_text(),
                         "IDE exercise")
        self.assertEqual(self.source.read_bytes(), self.source_bytes)

    def test_new_optional_toolchain_file_fails_isolation_check(self):
        def change_original(_runner):
            (self.root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "nightly"\n')

        with mock.patch.object(practice, "check_tools", side_effect=change_original):
            self.assertNotEqual(self.invoke(), 0)
        report = self.report()
        self.assertEqual(report["status"], "FAIL")
        self.assertIsNone(report["source_snapshot_before"]["rust-toolchain.toml"])
        self.assertIsNotNone(report["source_snapshot_after"]["rust-toolchain.toml"])
        self.assertIn("rust-toolchain.toml", report["changed_source_paths"])
        self.assertTrue(Path(report["workspace"]).exists())

    def test_missing_formatter_fails_with_original_diagnostic_and_keeps_workspace(self):
        def native(command, **_kwargs):
            if command[0] == "rustfmt":
                return subprocess.CompletedProcess(command, 1,
                    b"error: 'rustfmt' is not installed for the toolchain '1.85.0'\n")
            return subprocess.CompletedProcess(command, 0, b"rustc 1.85.0 (test)\nrelease: 1.85.0\n")

        with mock.patch.object(practice.subprocess, "run", side_effect=native):
            self.assertNotEqual(self.invoke(), 0)
        report = self.report()
        self.assertEqual(report["status"], "FAIL")
        self.assertEqual(report["steps"][-1]["returncode"], 1)
        self.assertIn("not installed", (self.output / report["steps"][-1]["log"]).read_text())
        self.assertTrue(Path(report["workspace"]).is_dir())

    def test_dated_nightly_is_required_and_bootstrap_cannot_fake_stable(self):
        for index, channel in enumerate(("nightly", "stable", "nightly-2026-99-42")):
            with self.subTest(channel=channel):
                self.output = self.base / f"invalid-nightly-{index}"
                self.assertNotEqual(self.invoke("--mode", "nightly", "--toolchain", channel), 0)
                self.assertEqual(self.report()["status"], "FAIL")
                self.assertEqual(self.report()["steps"], [])
        self.output = self.base / "bootstrap-blocked"
        with mock.patch.dict(os.environ, {"RUSTC_BOOTSTRAP": "1"}):
            self.assertNotEqual(self.invoke("--mode", "nightly"), 0)
        self.assertIn("RUSTC_BOOTSTRAP", self.report()["error"])
        self.assertEqual(self.report()["steps"], [])

    def test_negative_control_requires_both_failure_code_and_expected_diagnostic(self):
        runner = self.runner()
        for index, (code, message, accepted) in enumerate((
                (0, "E0554", False), (101, "unrelated manifest failure", False),
                (101, "error[E0554]: feature is not allowed", True))):
            command = [sys.executable, "-c", "import sys; print(sys.argv[1]); exit(int(sys.argv[2]))",
                       message, str(code)]
            with self.subTest(code=code, message=message):
                if accepted:
                    runner.run(f"control-{index}", command, expected=101, diagnostic=r"E0554")
                else:
                    with self.assertRaises(practice.PracticeFailure):
                        runner.run(f"control-{index}", command, expected=101, diagnostic=r"E0554")
                self.assertEqual(runner.steps[-1]["passed"], accepted)

    def test_noop_edition_fix_cannot_claim_success_or_switch_manifest(self):
        def native(command, **_kwargs):
            output = b"rustc 1.85.0 (test)\nrelease: 1.85.0\n"
            if "test" in command:
                output = b"test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n"
            return subprocess.CompletedProcess(command, 0, output)

        with mock.patch.object(practice.subprocess, "run", side_effect=native):
            self.assertNotEqual(self.invoke("--mode", "edition"), 0)
        report = self.report()
        manifest = Path(report["workspace"]) / "lab/appendix-lab/Cargo.toml"
        self.assertEqual(report["status"], "FAIL")
        self.assertIn('edition = "2021"', manifest.read_text())
        self.assertEqual(self.source.read_bytes(), self.source_bytes)


if __name__ == "__main__":
    unittest.main()
