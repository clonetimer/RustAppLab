"""Synthetic, temporary validator fixtures; never actual learner evidence."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import release_decision


class LearnerEvidenceTests(unittest.TestCase):
    def write_json(self, path, data):
        path.write_text(json.dumps(data), encoding="utf-8")

    def fixture(self, root):
        """Exercise integrity validation with clearly labelled synthetic bytes."""
        tasks = []
        for route in ("firstcard", "unitlab", "independent"):
            artifacts = []
            for kind in ("learner_source", "command_log", "transfer_work"):
                artifact = root / f"{route}-{kind}.txt"
                artifact.write_text(f"SYNTHETIC TEST FIXTURE: {route} {kind}\n", encoding="utf-8")
                artifacts.append({"kind": kind, "path": artifact.name,
                                  "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest()})
            record = {
                "schema": 1, "kind": "human-learner-session", "version": "test-only",
                "source_id": "fixture", "route": route, "session_id": f"fixture-{route}",
                "learner_id": "fixture-learner", "observer_id": "fixture-observer",
                "observed_at": "2026-11-24T09:00:00+00:00", "task_id": f"fixture-{route}-a",
                "prediction": "Synthetic prediction text for validator testing.",
                "explanation": "Synthetic explanation text for validator testing.",
                "error_explanation": "Synthetic error analysis for validator testing.",
                "transfer_task": "Synthetic changed task for validator testing.",
                "transfer_result": "Synthetic observed result for validator testing.",
                "hints_used": [], "artifacts": artifacts,
            }
            path = root / f"{route}.json"
            self.write_json(path, record)
            tasks.append({"route": route, "record": path.name,
                          "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
        report = {
            "schema": 1, "version": "test-only", "source_id": "fixture",
            "source_id_after": "fixture", "status": "PASS",
            "method": "reviewed-human-observation", "reviewer_id": "fixture-reviewer",
            "reviewed_at": "2026-11-24T10:00:00+00:00", "unresolved_blockers": [],
            "observed_tasks": tasks,
        }
        path = root / "learners.json"
        self.write_json(path, report)
        return path, report

    def accepted(self, path):
        return release_decision.evidence(path, "fixture", "learners", "test-only")

    def test_legacy_placeholder_pass_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "learners.json"
            for tasks in ("TODO", ["placeholder"], {"task": "unverified"}):
                with self.subTest(tasks=tasks):
                    self.write_json(path, {"source_id": "fixture", "status": "PASS",
                                           "observed_tasks": tasks, "unresolved_blockers": []})
                    self.assertFalse(release_decision.evidence(path, "fixture", "learners"))

    def test_complete_bound_fixture_has_valid_structure_and_integrity(self):
        with tempfile.TemporaryDirectory() as tmp:
            path, _ = self.fixture(Path(tmp))
            self.assertTrue(self.accepted(path))

    def test_wrong_identity_unreviewed_missing_routes_and_blockers_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for key, value in (("schema", True), ("version", "old"), ("source_id_after", "old"),
                               ("method", "automated-test"), ("reviewer_id", "TODO"),
                               ("reviewed_at", "not-a-date"), ("reviewed_at", "2026-11-24T08:00:00+00:00"),
                               ("unresolved_blockers", ["unresolved"]),
                               ("observed_tasks", [])):
                with self.subTest(field=key):
                    path, report = self.fixture(root)
                    report[key] = value
                    self.write_json(path, report)
                    self.assertFalse(self.accepted(path))
            path, report = self.fixture(root)
            report["observed_tasks"] = report["observed_tasks"][:2]
            self.write_json(path, report)
            self.assertFalse(self.accepted(path))

    def test_session_metadata_and_human_responses_cannot_be_left_blank(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for key, value in (("schema", True), ("kind", "automated-replay"), ("source_id", "old"),
                               ("version", "old"), ("route", "unitlab"),
                               ("observer_id", ""), ("session_id", "____"),
                               ("observed_at", "2026-11-24"), ("prediction", "待填写"),
                               ("explanation", "未评估"), ("explanation", "\u3000"),
                               ("transfer_result", "TODO"),
                               ("hints_used", None), ("artifacts", [])):
                with self.subTest(field=key):
                    path, report = self.fixture(root)
                    entry = report["observed_tasks"][0]
                    record_path = root / entry["record"]
                    record = json.loads(record_path.read_text())
                    record[key] = value
                    self.write_json(record_path, record)
                    entry["sha256"] = hashlib.sha256(record_path.read_bytes()).hexdigest()
                    self.write_json(path, report)
                    self.assertFalse(self.accepted(path))

    def test_missing_or_modified_session_and_artifact_bytes_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in ("firstcard.json", "firstcard-learner_source.txt",
                         "firstcard-command_log.txt", "firstcard-transfer_work.txt"):
                for mode in ("changed", "missing"):
                    with self.subTest(name=name, mode=mode):
                        path, _ = self.fixture(root)
                        target = root / name
                        if mode == "missing":
                            target.unlink()
                        else:
                            target.write_text("changed", encoding="utf-8")
                        self.assertFalse(self.accepted(path))

    def test_record_paths_cannot_escape_the_evidence_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in ("../outside.json", "/tmp/outside.json", r"..\outside.json"):
                with self.subTest(name=name):
                    path, report = self.fixture(root)
                    report["observed_tasks"][0]["record"] = name
                    self.write_json(path, report)
                    self.assertFalse(self.accepted(path))

    def test_repeated_artifact_cannot_impersonate_three_evidence_types(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path, report = self.fixture(root)
            entry = report["observed_tasks"][0]
            record_path = root / entry["record"]
            record = json.loads(record_path.read_text())
            original = record["artifacts"][0]
            for index, artifact in enumerate(record["artifacts"][1:], 1):
                artifact["path"] = "./" * index + original["path"]
                artifact["sha256"] = original["sha256"]
            self.write_json(record_path, record)
            entry["sha256"] = hashlib.sha256(record_path.read_bytes()).hexdigest()
            self.write_json(path, report)
            self.assertFalse(self.accepted(path))

    def test_artifact_cannot_follow_a_symbolic_link(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path, _ = self.fixture(root)
            artifact = root / "firstcard-command_log.txt"
            saved = root / "saved-log.txt"
            artifact.rename(saved)
            try:
                artifact.symlink_to(saved)
            except OSError:
                self.skipTest("symlinks not available for this account")
            self.assertFalse(self.accepted(path))

    def test_missing_learner_records_still_leave_engineering_at_rc(self):
        gates = {name: True for name in ("local_engineering", "real_browser", "fresh_archive",
                                        "windows_native", "declared_msrv")}
        gates["learner_acceptance"] = False
        self.assertEqual(release_decision.decide(gates), "RC_READY")


if __name__ == "__main__":
    unittest.main()
