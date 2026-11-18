"""D0-D6 checks audit and reference contracts, not actual learner mastery."""
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
        # Count alone permits an incorrect ID to replace a real subsection.
        chapter_lengths = {1: 3, 3: 5, 4: 3, 5: 3, 6: 3, 7: 5, 8: 3,
                           9: 3, 10: 3, 11: 3, 12: 6, 13: 4, 14: 5,
                           15: 6, 16: 4, 17: 3, 18: 3, 19: 5, 20: 3}
        expected = {"2", *"ABCDEFG"}
        expected.update(f"{chapter}.{section}"
                        for chapter, length in chapter_lengths.items()
                        for section in range(1, length + 1))
        self.assertEqual(set(identifiers), expected)
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

    def test_development_experience_has_one_title_and_unique_sections(self):
        text = (ROOT / "verification/DEVELOPMENT_EXPERIENCE_D6.md").read_text(encoding="utf-8")
        self.assertEqual(text.count("# RustAppLab · D6 开发经验记录"), 1)
        sections = re.findall(r"^## (\d+)\.", text, re.M)
        self.assertEqual(len(sections), len(set(sections)))

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

    def test_d5_web_server_is_an_isolated_offline_reference_workspace(self):
        base = ROOT / "projects/book-d5-webserver"
        for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                         "webserver-lab/Cargo.toml", "webserver-lab/src/lib.rs",
                         "webserver-lab/src/main.rs", "webserver-lab/tests/cli.rs",
                         "webserver-lab/samples/index.html"):
            with self.subTest(path=relative):
                self.assertTrue((base / relative).is_file())
        manifest = (base / "webserver-lab/Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('rust-version = "1.74"', manifest)
        self.assertIn("publish = false", manifest)
        self.assertNotIn("[dependencies]", manifest)
        workspace = (base / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('"webserver-lab"', workspace)
        root_workspace = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertNotIn('"projects/book-d5-webserver"', root_workspace)

        lesson = (ROOT / "course/BOOK_D5_WEB_SERVER.md").read_text(encoding="utf-8")
        implementation = (base / "webserver-lab/src/lib.rs").read_text(encoding="utf-8")
        for topic in ("20.1", "20.2", "20.3", "Arc<Mutex<Receiver<Message>>>", "Drop"):
            with self.subTest(topic=topic):
                self.assertIn(topic, lesson)
        for contract in ("serve_sequential", "pub fn serve(", "Message::Shutdown",
                         "impl Drop for ThreadPool", "MAX_HEADER_BYTES"):
            with self.subTest(contract=contract):
                self.assertIn(contract, implementation)

    def test_d5_is_in_release_and_declared_msrv_gates(self):
        release = (ROOT / "scripts/release_check.py").read_text(encoding="utf-8")
        msrv = (ROOT / "scripts/msrv_check.py").read_text(encoding="utf-8")
        self.assertIn("*book_d5_steps()", release)
        self.assertIn("plan.extend(book_d5_steps(quality=False))", msrv)

    def test_d6_appendix_reference_workspace_is_isolated_and_executable(self):
        base = ROOT / "projects/book-d6-appendices"
        for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                         "appendix-lab/Cargo.toml", "appendix-lab/src/lib.rs"):
            with self.subTest(path=relative):
                self.assertTrue((base / relative).is_file())
        manifest = (base / "appendix-lab/Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('edition = "2021"', manifest)
        self.assertIn('rust-version = "1.74"', manifest)
        self.assertIn("publish = false", manifest)
        self.assertNotIn("[dependencies]", manifest)
        root_workspace = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertNotIn('"projects/book-d6-appendices"', root_workspace)

        lesson = (ROOT / "course/BOOK_D6_APPENDICES.md").read_text(encoding="utf-8")
        keyword_rows = {}
        token_pattern = chr(96) + r"([^" + chr(96) + r"]+)" + chr(96)
        for line in lesson.splitlines():
            for label in ("Strict", "Reserved", "Weak"):
                if line.startswith(f"| {label} |"):
                    keyword_rows[label] = re.findall(token_pattern, line)
        self.assertEqual(keyword_rows["Strict"], [
            "_", "as", "async", "await", "break", "const", "continue", "crate",
            "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl",
            "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
            "return", "self", "Self", "static", "struct", "super", "trait",
            "true", "type", "unsafe", "use", "where", "while",
        ])
        self.assertEqual(keyword_rows["Reserved"], [
            "abstract", "become", "box", "do", "final", "gen", "macro",
            "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
        ])
        self.assertEqual(keyword_rows["Weak"], [
            "'static", "macro_rules", "raw", "safe", "union",
        ])
        for row in ("| 11 | 位与", "| 12 | 位异或", "| 13 | 位或",
                    "| 14 |", "| 15 |", "| 16 |", "| 17 |", "| 18 |", "| 19 |"):
            with self.subTest(precedence=row):
                self.assertIn(row, lesson)
        implementation = (base / "appendix-lab/src/lib.rs").read_text(encoding="utf-8")
        keywords = (
            "_", "as", "async", "await", "break", "const", "continue", "crate",
            "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl",
            "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
            "return", "self", "Self", "static", "struct", "super", "trait",
            "true", "type", "unsafe", "use", "where", "while", "abstract",
            "become", "box", "do", "final", "gen", "macro", "override", "priv",
            "try", "typeof", "unsized", "virtual", "yield", "'static",
            "macro_rules", "raw", "safe", "union",
        )
        for topic in ("附录 A", "附录 B", "附录 C", "附录 D", "附录 E", "附录 F", "附录 G",
                      "Strict keywords", "Reserved keywords", "Weak keywords",
                      "Edition 2024", "rust-analyzer", "Stable", "Beta", "Nightly",
                      "::", "->", "=>", "@", "#!", "$", *keywords):
            with self.subTest(topic=topic):
                self.assertIn(topic, lesson)
        for contract in ("pub fn r#match", "impl Add for Point",
                         "derive(Debug, Clone, PartialEq, Eq, Hash, Default)",
                         "impl fmt::Display for Task", "f64::NAN", "compile_fail"):
            with self.subTest(contract=contract):
                self.assertIn(contract, implementation)
        self.assertEqual(len(re.findall(r"#\[test\]", implementation)), 11)
        self.assertEqual(len(re.findall(r"compile_fail", implementation)), 2)

    def test_d6_foundations_workspace_covers_remaining_chapter_gaps(self):
        base = ROOT / "projects/book-d6-foundations"
        for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                         "foundation-lab/Cargo.toml", "foundation-lab/src/lib.rs",
                         "foundation-lab/src/main.rs", "foundation-lab/tests/cli.rs"):
            with self.subTest(path=relative):
                self.assertTrue((base / relative).is_file())
        manifest = (base / "foundation-lab/Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('edition = "2021"', manifest)
        self.assertIn('rust-version = "1.74"', manifest)
        self.assertIn("publish = false", manifest)
        self.assertNotIn("[dependencies]", manifest)

        root_workspace = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        members = re.search(r"members\s*=\s*\[(.*?)\]", root_workspace, re.S)
        self.assertIsNotNone(members)
        member_names = re.findall(r'"([^"]+)"', members.group(1))
        self.assertEqual(len(member_names), 17)
        self.assertTrue(all(name.startswith("apps/") for name in member_names))
        self.assertNotIn('"projects/book-d6-foundations"', root_workspace)
        self.assertNotIn('"projects/book-d6-appendices"', root_workspace)

        lesson = (ROOT / "course/BOOK_D6_FOUNDATIONS.md").read_text(encoding="utf-8")
        implementation = (base / "foundation-lab/src/lib.rs").read_text(encoding="utf-8")
        for topic in ("1.2", "猜数字", "3.4", "5.2", "5.3", "6.3", "8.3",
                      "9.1", "9.3", "12.5", "13.3", "13.4"):
            with self.subTest(topic=topic):
                self.assertIn(topic, lesson)
        for contract in ("pub struct Rectangle", "pub fn compare_guess",
                         "pub fn word_frequencies", "pub fn parse_non_empty",
                         "pub fn config_dir_from", "pub fn count_non_empty_lines",
                         "pub fn sum_with_loop", "pub fn sum_with_iterator",
                         "pub fn benchmark_sums", "buffered_line_count_propagates_io_failures",
                         "// This assertion protects a programmer invariant",
                         "/* Use parse_non_empty for user-provided values instead. */"):
            with self.subTest(contract=contract):
                self.assertIn(contract, implementation)
        self.assertEqual(len(re.findall(r"#\[test\]", implementation)), 14)
        cli = (base / "foundation-lab/tests/cli.rs").read_text(encoding="utf-8")
        self.assertEqual(len(re.findall(r"#\[test\]", cli)), 6)
        self.assertEqual(len(re.findall(r"compile_fail", implementation)), 3)


    def test_d6_toolchain_practice_is_shipped_and_keeps_risky_work_isolated(self):
        guide = (ROOT / "course/BOOK_D6_PRACTICAL.md").read_text(encoding="utf-8")
        learner = (ROOT / "scripts/package_learner_kit.py").read_text(encoding="utf-8")
        structural = (ROOT / "scripts/structural_check.py").read_text(encoding="utf-8")
        for term in ("cargo fmt", "cargo clippy", "rust-analyzer", "cargo +1.85.0 fix --edition",
                     'edition = "2024"', "nightly-2026-11-17", "generic_const_exprs",
                     "临时副本", "MSRV", "不计入 Stable"):
            with self.subTest(term=term):
                self.assertIn(term, guide)
        self.assertIn("BOOK_D6_PRACTICAL.md", learner)
        self.assertIn("BOOK_D6_PRACTICAL.md", structural)

    def test_d6_is_registered_in_release_msrv_and_learner_package(self):
        release = (ROOT / "scripts/release_check.py").read_text(encoding="utf-8")
        msrv = (ROOT / "scripts/msrv_check.py").read_text(encoding="utf-8")
        structural = (ROOT / "scripts/structural_check.py").read_text(encoding="utf-8")
        package = (ROOT / "scripts/package.py").read_text(encoding="utf-8")
        learner = (ROOT / "scripts/package_learner_kit.py").read_text(encoding="utf-8")
        self.assertIn("*book_d6_steps()", release)
        self.assertIn("plan.extend(book_d6_steps(quality=False))", msrv)
        self.assertIn("projects/book-d6-appendices", structural)
        self.assertIn("book-d6-appendices", package)
        self.assertIn("BOOK_D6_APPENDICES.md", learner)
        self.assertIn('projects" / "book-d6-appendices', learner)
        self.assertIn("*book_d6_foundations_steps()", release)
        self.assertIn("plan.extend(book_d6_foundations_steps(quality=False))", msrv)
        self.assertIn("projects/book-d6-foundations", structural)
        self.assertIn("book-d6-foundations", package)
        self.assertIn("BOOK_D6_FOUNDATIONS.md", learner)
        self.assertIn('projects" / "book-d6-foundations', learner)


if __name__ == "__main__":
    unittest.main()
