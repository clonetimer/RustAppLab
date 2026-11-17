"""D0/D1 checks audit and reference contracts, not actual learner mastery."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
AUDIT = ROOT / "course/BOOK_COVERAGE_AUDIT.md"
LECTURE = ROOT / "course/BOOK_D1_CARGO.md"


class BookCoverageTests(unittest.TestCase):
    def test_every_book_subsection_has_one_audit_row(self):
        text = AUDIT.read_text(encoding="utf-8")
        rows = []
        for line in text.splitlines():
            if not re.match(r"^\| (?:\d+\.\d+|\d+|[A-G]) \|", line):
                continue
            cols = [x.strip() for x in line.split("|")[1:-1]]
            self.assertEqual(len(cols), 5, line)
            rows.append(cols)
        identifiers = [row[0] for row in rows]
        self.assertEqual(len(rows), 81)
        self.assertEqual(len(set(identifiers)), len(identifiers))
        for item in ("1.1", "2", "14.1", "14.2", "14.3", "14.4", "14.5",
                     "15.2", "18.2", "19.1", "19.5", "20.3", "A", "G"):
            self.assertIn(item, identifiers)
        for ident, title, status, source, reason in rows:
            self.assertIn(status, ("已有实作", "部分", "缺口", "开发中", "资料索引"))
            self.assertTrue(title and reason, ident)
            path = source.strip(chr(96))
            if path != "—":
                with self.subTest(section=ident, path=path):
                    self.assertTrue((ROOT / path).is_file(), "mapped evidence must exist")

    def test_cargo_lecture_has_all_five_topics_and_safe_install(self):
        text = LECTURE.read_text(encoding="utf-8")
        for item in ("14.1", "14.2", "14.3", "14.4", "14.5",
                     "--offline", "--locked", "publish = false", "--root",
                     "cargo applab-map", "cargo package --list"):
            with self.subTest(item=item):
                self.assertIn(item, text)

    def test_isolated_cargo_workspace_and_local_publish_guard(self):
        base = ROOT / "projects/book-d1-cargo"
        for sub in ("book-card", "cargo-applab-map"):
            manifest = (base / sub / "Cargo.toml").read_text(encoding="utf-8")
            self.assertIn("publish = false", manifest)
            self.assertIn('rust-version = "1.74"', manifest)
        workspace = (base / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('[profile.release]', workspace)
        self.assertIn('"book-card", "cargo-applab-map"', workspace)
        parent = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertNotIn('"projects/book-d1-cargo"', parent)


if __name__ == "__main__":
    unittest.main()
