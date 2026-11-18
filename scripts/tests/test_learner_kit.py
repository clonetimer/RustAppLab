"""Verify the actual B4 archive assets without building a distribution."""
import ast
import importlib.util
from pathlib import Path
import posixpath
import re
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/package_learner_kit.py'


class LearnerKitTests(unittest.TestCase):
    def assets(self):
        # An importable entrypoint prevents a tooling test from generating a kit.
        tree = ast.parse(SCRIPT.read_text(encoding='utf-8'))
        self.assertTrue(any(isinstance(node, ast.FunctionDef) and node.name == 'main'
                            for node in tree.body), 'Packaging needs a guarded main entrypoint')
        spec = importlib.util.spec_from_file_location('learner_kit', SCRIPT)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_b4_entry_links_resolve_inside_real_zip(self):
        assets = self.assets()
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp)/'assets.zip'
            with zipfile.ZipFile(archive, 'w') as output:
                assets.add(output, 'ADVANCED_B.html', (ROOT/'ADVANCED_B.html').read_bytes())
                for name, data in assets.course_inputs():
                    assets.add(output, name, data)
            with zipfile.ZipFile(archive) as output:
                self.assertIsNone(output.testzip())
                names = set(output.namelist())
                for source in ('ADVANCED_B.html', 'course/ADVANCED_B.md', 'course/ADVANCED_B_WINDOWS.md', 'course/BOOK_GUIDE.md', 'course/BOOK_D1_CARGO.md', 'projects/book-d1-cargo/README.md', 'course/BOOK_D2_SMART_POINTERS.md', 'projects/book-d2-smart-pointers/README.md', 'course/BOOK_D3_OOP_PATTERNS.md', 'projects/book-d3-oop-patterns/README.md', 'course/BOOK_D4_ADVANCED_FEATURES.md', 'projects/book-d4-advanced-features/README.md'):
                    self.assertIn(source, names)
                    text = output.read(source).decode('utf-8')
                    links = re.findall(r'href="([^"]+)"', text) if source.endswith('.html') else re.findall(r'(?<!!)\[[^\]]+\]\(([^)]+)\)', text)
                    for target in links:
                        if target.startswith(('https://', 'http://', '#')):
                            continue
                        resolved = posixpath.normpath(posixpath.join(posixpath.dirname(source), target.split('#', 1)[0]))
                        self.assertIn(resolved, names, f'{source} links to missing archive asset {target}')

    def test_d1_is_shipped_as_a_reference_not_an_extra_student_project(self):
        assets = dict(self.assets().course_inputs())
        for path in ("course/BOOK_COVERAGE_AUDIT.md", "course/BOOK_D1_CARGO.md",
                     "projects/book-d1-cargo/Cargo.toml",
                     "projects/book-d1-cargo/Cargo.lock",
                     "projects/book-d1-cargo/book-card/src/lib.rs",
                     "projects/book-d1-cargo/cargo-applab-map/src/main.rs"):
            self.assertIn(path, assets)
        self.assertIn("publish = false", assets["projects/book-d1-cargo/book-card/Cargo.toml"].decode())

    def test_d6_practice_guide_has_runner_and_local_links_in_archive(self):
        assets = self.assets()
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp) / "practice.zip"
            with zipfile.ZipFile(archive, "w") as output:
                for name, data in assets.course_inputs():
                    assets.add(output, name, data)
            with zipfile.ZipFile(archive) as output:
                names = set(output.namelist())
                self.assertIn("scripts/book_d6_check.py", names)
                for source in ("course/BOOK_D6_PRACTICAL.md", "course/BOOK_D6_FOUNDATIONS.md",
                               "course/BOOK_D6_LANGUAGE_BASICS.md"):
                    self.assertIn(source, names)
                    for target in re.findall(r'(?<!!)\[[^\]]+\]\(([^)]+)\)',
                                             output.read(source).decode("utf-8")):
                        if target.startswith(("https://", "http://", "#")):
                            continue
                        resolved = posixpath.normpath(posixpath.join(
                            posixpath.dirname(source), target.split("#", 1)[0]))
                        self.assertIn(resolved, names, f"{source}: missing {target}")

    def test_reference_lab_build_output_never_enters_learner_kit(self):
        target = ROOT / "projects/book-d6-foundations/target"
        target.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="package-probe-", dir=target) as temp:
            (Path(temp) / "generated.rs").write_text("// build artifact\n", encoding="utf-8")
            (Path(temp) / "incremental.lock").write_text("build lock\n", encoding="utf-8")
            names = [name for name, _ in self.assets().course_inputs()]
            leaked = [name for name in names if "/target/" in name]
            self.assertEqual(leaked, [], "compiled output must not be shipped as course source")

    def test_d2_is_shipped_as_a_reference_not_an_extra_student_project(self):
        assets = dict(self.assets().course_inputs())
        for path in ("course/BOOK_D2_SMART_POINTERS.md",
                     "projects/book-d2-smart-pointers/Cargo.toml",
                     "projects/book-d2-smart-pointers/Cargo.lock",
                     "projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs",
                     "projects/book-d2-smart-pointers/smart-pointer-lab/src/main.rs",
                     "projects/book-d2-smart-pointers/smart-pointer-lab/tests/cli.rs"):
            self.assertIn(path, assets)
        self.assertIn("publish = false",
                      assets["projects/book-d2-smart-pointers/smart-pointer-lab/Cargo.toml"].decode())

    def test_d3_is_shipped_as_a_reference_not_an_extra_student_project(self):
        assets = dict(self.assets().course_inputs())
        for path in ("course/BOOK_D3_OOP_PATTERNS.md",
                     "projects/book-d3-oop-patterns/Cargo.toml",
                     "projects/book-d3-oop-patterns/Cargo.lock",
                     "projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs",
                     "projects/book-d3-oop-patterns/oop-pattern-lab/src/main.rs",
                     "projects/book-d3-oop-patterns/oop-pattern-lab/tests/cli.rs"):
            self.assertIn(path, assets)
        self.assertIn("publish = false",
                      assets["projects/book-d3-oop-patterns/oop-pattern-lab/Cargo.toml"].decode())

    def test_d4_advanced_features_workspace_is_shipped_as_reference(self):
        assets = dict(self.assets().course_inputs())
        for path in ("course/BOOK_D4_ADVANCED_FEATURES.md",
                     "projects/book-d4-advanced-features/Cargo.toml",
                     "projects/book-d4-advanced-features/Cargo.lock",
                     "projects/book-d4-advanced-features/advanced-lab/src/lib.rs",
                     "projects/book-d4-advanced-features/advanced-lab/src/main.rs",
                     "projects/book-d4-advanced-features/advanced-lab/tests/cli.rs",
                     "projects/book-d4-advanced-features/d4-proc-macros/src/lib.rs"):
            self.assertIn(path, assets)
        self.assertIn("publish = false",
                      assets["projects/book-d4-advanced-features/advanced-lab/Cargo.toml"].decode())
        self.assertIn("proc-macro = true",
                      assets["projects/book-d4-advanced-features/d4-proc-macros/Cargo.toml"].decode())

    def test_d6_appendices_workspace_is_shipped_as_reference(self):
        assets = dict(self.assets().course_inputs())
        for path in ("course/BOOK_D6_APPENDICES.md",
                     "projects/book-d6-appendices/Cargo.toml",
                     "projects/book-d6-appendices/Cargo.lock",
                     "projects/book-d6-appendices/README.md",
                     "projects/book-d6-appendices/appendix-lab/Cargo.toml",
                     "projects/book-d6-appendices/appendix-lab/src/lib.rs"):
            self.assertIn(path, assets)
        self.assertIn("publish = false",
                      assets["projects/book-d6-appendices/appendix-lab/Cargo.toml"].decode())
        intro = self.assets().INTRO
        self.assertIn("D6-A 附录参考实验", intro)
        self.assertIn("D6-A/D6-B 都是独立参考实验", intro)

    def test_d6_foundations_workspace_is_shipped_as_reference(self):
        assets = dict(self.assets().course_inputs())
        for path in ("course/BOOK_D6_FOUNDATIONS.md",
                     "projects/book-d6-foundations/Cargo.toml",
                     "projects/book-d6-foundations/Cargo.lock",
                     "projects/book-d6-foundations/README.md",
                     "projects/book-d6-foundations/foundation-lab/Cargo.toml",
                     "projects/book-d6-foundations/foundation-lab/src/lib.rs",
                     "projects/book-d6-foundations/foundation-lab/src/main.rs",
                     "projects/book-d6-foundations/foundation-lab/tests/cli.rs"):
            self.assertIn(path, assets)
        self.assertIn("publish = false",
                      assets["projects/book-d6-foundations/foundation-lab/Cargo.toml"].decode())
        intro = self.assets().INTRO
        self.assertIn("D6-B 基础缺口补齐实验", intro)

    def test_intro_explains_separate_b4_dependency_preparation(self):
        intro = self.assets().INTRO
        self.assertNotIn('不需要下载外部 Rust crate', intro)
        for phrase in ('B1–B3', 'B4', 'prepare-json', '联网', '缓存'):
            self.assertIn(phrase, intro)


if __name__ == '__main__':
    unittest.main()
