"""Candidate assembly must reject incomplete or unrelated D/E evidence."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
import assemble_candidate
import book_d6_check


class BookPracticeDeliveryTests(unittest.TestCase):
    def fixture(self, folder):
        snapshot = book_d6_check.source_snapshot(ROOT)
        names = ("rustc-version", "cargo-version", "rustfmt-version", "clippy-version",
                 "rustdoc-version", "original-tests", "edition-2021-probe", "edition-fix",
                 "format", "format-check", "clippy", "debug-tests", "doctests", "release-tests")
        steps = []
        for name in names:
            log = folder / (name + ".txt")
            log.write_bytes(b"recorded command output\n")
            steps.append({"step": name, "passed": True, "returncode": 0,
                          "expected_returncode": 0, "log": log.name,
                          "sha256": hashlib.sha256(log.read_bytes()).hexdigest()})
        evidence = []
        for name in ("manifest-before.txt", "manifest-after.txt", "source-before.txt",
                     "source-after.txt", "changes.diff.txt"):
            file = folder / name
            file.write_bytes(b"recorded before/after evidence\n")
            evidence.append({"file": name, "sha256": hashlib.sha256(file.read_bytes()).hexdigest()})
        data = {"mode": "edition", "toolchain": "1.85.0", "system": "Linux", "status": "PASS",
                "git_commit": "current-commit", "source_unchanged": True, "changed_source_paths": [],
                "source_snapshot_before": snapshot, "source_snapshot_after": snapshot,
                "source_id": book_d6_check.snapshot_id(snapshot),
                "source_id_after": book_d6_check.snapshot_id(snapshot),
                "runner_sha256": hashlib.sha256((ROOT / "scripts/book_d6_check.py").read_bytes()).hexdigest(),
                "steps": steps, "evidence": evidence}
        report = folder / "report.json"
        report.write_text(json.dumps(data), encoding="utf-8")
        return report, data

    def read(self, report):
        self.assertTrue(callable(getattr(assemble_candidate, "read_book_practice_report", None)),
                        "candidate assembly must validate D/E reports")
        return assemble_candidate.read_book_practice_report(
            report, "edition", "Linux", "current-commit")

    def test_complete_source_bound_report_is_accepted(self):
        with tempfile.TemporaryDirectory() as name:
            report, _ = self.fixture(Path(name))
            self.assertEqual(self.read(report)["status"], "PASS")

    def test_wrong_revision_platform_mode_or_runner_is_rejected(self):
        for key, value in (("git_commit", "old-commit"), ("mode", "tools"),
                           ("system", "Windows"), ("runner_sha256", "old-code"),
                           ("source_id_after", "changed-input"), ("source_unchanged", False)):
            with self.subTest(key=key), tempfile.TemporaryDirectory() as name:
                report, data = self.fixture(Path(name))
                data[key] = value
                report.write_text(json.dumps(data), encoding="utf-8")
                with self.assertRaises(ValueError):
                    self.read(report)

    def test_missing_step_and_changed_log_are_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            report, data = self.fixture(Path(name))
            original = data["steps"][:]
            data["steps"] = [step for step in original if step["step"] != "edition-fix"]
            report.write_text(json.dumps(data), encoding="utf-8")
            with self.assertRaises(ValueError):
                self.read(report)
            data["steps"] = original
            report.write_text(json.dumps(data), encoding="utf-8")
            (Path(name) / "edition-fix.txt").write_bytes(b"edited log")
            with self.assertRaises(ValueError):
                self.read(report)

    def test_changed_migration_evidence_is_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            report, _ = self.fixture(Path(name))
            (Path(name) / "changes.diff.txt").write_bytes(b"edited evidence")
            with self.assertRaises(ValueError):
                self.read(report)


if __name__ == "__main__":
    unittest.main()
