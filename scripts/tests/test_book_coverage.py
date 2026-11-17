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
        implementation = (base / "appendix-lab/src/lib.rs").read_text(encoding="utf-8")
        for topic in ("附录 A", "附录 B", "附录 C", "附录 D", "附录 E", "附录 F", "附录 G",
                      "Strict keywords", "Reserved keywords", "Weak keywords",
                      "Edition 2024", "rust-analyzer", "Stable", "Beta", "Nightly",
                      "`async`", "`await`", "`dyn`", "`abstract`", "`gen`",
                      "`macro`", "`safe`", "`macro_rules`", "`'static`",
                      "`::`", "`->`", "`=>`", "`@`", "`#!`", "`"""D0-D6 checks audit and reference contracts, not actual learner mastery."""
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
        implementation = (base / "appendix-lab/src/lib.rs").read_text(encoding="utf-8")
"):
            with self.subTest(topic=topic):
                self.assertIn(topic, lesson)
        for contract in ("pub fn r#match", "impl Add for Point",
                         "derive(Debug, Clone, PartialEq, Eq, Hash, Default)",
                         "impl fmt::Display for Task", "f64::NAN", "compile_fail"):
            with self.subTest(contract=contract):
                self.assertIn(contract, implementation)
        self.assertEqual(len(re.findall(r"#\[test\]", implementation)), 11)
        self.assertEqual(len(re.findall(r"compile_fail", implementation)), 2)

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




if __name__ == "__main__":
    unittest.main()
