"""D7 frozen modern Book mapping: structure, not teaching mastery."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
AUDIT = ROOT / "course/BOOK_MODERN_COVERAGE_D7.md"
OLD = ROOT / "course/BOOK_COVERAGE_AUDIT.md"
BOOK_SHA = "1500248d8f230566e4ec9f27fcbb8fe9e2898ab1"


class ModernBookCoverageTests(unittest.TestCase):
    def test_fixed_book_scope_and_legacy_mapping(self):
        rows = []
        for line in AUDIT.read_text(encoding="utf-8").splitlines():
            if re.match(r"^\| (?:\d+(?:\.\d+)?|[A-G]) \|", line):
                cols = [c.strip() for c in line.split("|")[1:-1]]
                self.assertEqual(len(cols), 7, line)
                rows.append(cols)
        lengths = {1: 3, 3: 5, 4: 3, 5: 3, 6: 3, 7: 5, 8: 3, 9: 3,
                   10: 3, 11: 3, 12: 6, 13: 4, 14: 5, 15: 6, 16: 4,
                   17: 6, 18: 3, 19: 3, 20: 5, 21: 3}
        expected = {"2", *"ABCDEFG"}
        expected.update(f"{ch}.{sec}" for ch, length in lengths.items()
                        for sec in range(1, length + 1))
        self.assertEqual(len(rows), 87)
        self.assertEqual({r[0] for r in rows}, expected)
        old_ids = set(re.findall(r"^\| (\d+(?:\.\d+)?|[A-G]) \|",
                                 OLD.read_text(encoding="utf-8"), re.M))
        self.assertEqual(len(old_ids), 81)
        mapped = [r[2] for r in rows if r[2] != "—"]
        self.assertEqual(len(mapped), 81)
        self.assertEqual(set(mapped), old_ids)
        for ident, official, legacy, status, paths, stage, action in rows:
            with self.subTest(section=ident):
                self.assertIn(f"/blob/{BOOK_SHA}/src/", official)
                if "." in ident:
                    ch, sec = map(int, ident.split("."))
                    self.assertIn(f"/ch{ch:02}-{sec:02}-", official)
                    if ch == 17:
                        self.assertEqual(legacy, "—")
                        self.assertEqual(stage, "D10")
                        self.assertIn(status, {"部分待补", "专项缺口"})
                    else:
                        self.assertEqual(legacy, f"{ch - (ch >= 18)}.{sec}")
                elif ident == "2":
                    self.assertIn("/ch02-00-", official)
                    self.assertEqual(legacy, ident)
                else:
                    self.assertIn(f"/appendix-{ord(ident) - 64:02}-", official)
                    self.assertEqual(legacy, ident)
                self.assertIn(status, {"候选复用", "新版复核", "部分待补", "专项缺口", "资料索引"})
                self.assertTrue(stage and action)
                for path in paths.split("; "):
                    self.assertTrue((ROOT / path.strip(chr(96))).is_file(), path)

    def test_modern_route_does_not_replace_legacy_baselines(self):
        self.assertIn('resolver = "2"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        unitlab = (ROOT / "apps/unitlab/Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('edition = "2021"', unitlab)
        self.assertIn('rust-version = "1.74"', unitlab)
        for course in ("c1", "c2"):
            manifest = (ROOT / f"projects/advanced-{course}/reference/Cargo.toml").read_text(encoding="utf-8")
            self.assertIn('rust-version = "1.85"', manifest)
        self.assertIn("MODERN_RUST_ROADMAP.md", (ROOT / "course/README.md").read_text(encoding="utf-8"))
