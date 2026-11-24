"""The repair ledger must cover the frozen audit, with real executable anchors."""
import hashlib
import json
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
BASELINE = ROOT / "verification/concept-audit-rc3.json"
REPAIRS = ROOT / "verification/concept-repair-20261124.json"
SOURCE = ROOT / "projects/book-d6-foundations/foundation-lab/src/concept_repairs.rs"


class ConceptRepairContract(unittest.TestCase):
    def ledger(self):
        self.assertTrue(REPAIRS.is_file(), "31 repairs need a separate evidence ledger")
        return json.loads(REPAIRS.read_text(encoding="utf-8"))

    def test_exact_historical_targets_and_titles_are_preserved(self):
        baseline = json.loads(BASELINE.read_text(encoding="utf-8"))
        repair = self.ledger()
        self.assertEqual(repair["baseline_sha256"], hashlib.sha256(BASELINE.read_bytes()).hexdigest())
        self.assertEqual(repair["scope"]["sampled_concept_count"], 118)
        self.assertEqual(repair["scope"]["chapters"], [4, 5, 6, 8, 9, 10])
        expected = {c["id"]: c for c in baseline["concepts"] if c["id"] in baseline["remaining_ids"]}
        entries = repair["concepts"]
        self.assertEqual(len(entries), 31)
        self.assertEqual({c["id"] for c in entries}, set(expected))
        for entry in entries:
            with self.subTest(concept=entry["id"]):
                self.assertEqual(entry["title"], expected[entry["id"]]["title"])
                self.assertEqual(entry["baseline_status"], expected[entry["id"]]["current_status"])
                self.assertEqual(entry["source_url"], expected[entry["id"]]["source_url"])
        self.assertEqual(repair["learner_attainment"], "未评估")

    def test_every_repair_has_the_six_parts_and_an_executable_section(self):
        repair = self.ledger()
        self.assertTrue(SOURCE.is_file(), "repair claims require the Rust examples")
        source = SOURCE.read_text(encoding="utf-8")
        sections = re.split(r"(?m)^//! ## ([^\n ]+)\s*\n", source)
        sections = dict(zip(sections[1::2], sections[2::2]))
        lesson = (ROOT / "course/BOOK_31_GAPS_REPAIR.md").read_text(encoding="utf-8")
        for entry in repair["concepts"]:
            with self.subTest(concept=entry["id"]):
                self.assertIn(entry["id"], sections)
                self.assertIn(f"### {entry['id']}", lesson)
                for key in ("explanation", "prediction", "success", "failure", "repair", "transfer"):
                    self.assertTrue(entry["teaching_chain"][key].strip(), key)
                evidence = entry["evidence"]
                self.assertTrue(evidence)
                for item in evidence:
                    self.assertTrue((ROOT / item["path"]).is_file(), item)
                    self.assertIn(item["kind"], ("doctest", "compile_fail", "integration_test"))
                self.assertIn("//! ```", sections[entry["id"]])
                self.assertTrue(any("failure" in i["roles"] for i in evidence))
                self.assertTrue(any("success" in i["roles"] for i in evidence))


if __name__ == "__main__":
    unittest.main()
