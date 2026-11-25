"""D8 file/packaging isolation; actual Rust runs in the dedicated CI job."""
import ast
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-d8-edition"


class ModernD8DeliveryTests(unittest.TestCase):
    def test_fixed_independent_workspace(self):
        self.assertNotIn("modern-d8-edition", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for rel in ("rust-toolchain.toml", "learner/rust-toolchain.toml"):
            self.assertIn('channel = "1.100.0"', (BASE / rel).read_text(encoding="utf-8"))
        for rel, edition in (("reference/e2021/Cargo.toml", "2021"),
                             ("reference/e2024/Cargo.toml", "2024"),
                             ("learner/Cargo.toml", "2024")):
            manifest = (BASE / rel).read_text(encoding="utf-8")
            self.assertIn(f'edition = "{edition}"', manifest)
            self.assertIn('rust-version = "1.100"', manifest)
            self.assertIn("publish = false", manifest)
        self.assertIn('resolver = "3"', (BASE / "reference/Cargo.toml").read_text(encoding="utf-8"))
        self.assertIn('resolver = "3"', (BASE / "learner/Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual((BASE / "reference/e2024/tests/contracts.rs").read_bytes(),
                         (BASE / "learner/tests/contracts.rs").read_bytes())
        self.assertEqual((BASE / "learner/src/lib.rs").read_text(encoding="utf-8").count("TODO("), 3)
        self.assertNotEqual((BASE / "reference/e2024/src/lib.rs").read_bytes(),
                            (BASE / "learner/src/lib.rs").read_bytes())
        ast.parse((ROOT / "scripts/modern_d8_check.py").read_text(encoding="utf-8"))

    def test_source_and_learner_archives_include_correct_d8_files(self):
        for script in ("package.py", "package_learner_kit.py"):
            spec = importlib.util.spec_from_file_location(script, ROOT / "scripts" / script)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            if script == "package.py":
                for path in BASE.rglob("*"):
                    if path.is_file() and "target" not in path.relative_to(BASE).parts:
                        self.assertTrue(module.allowed(path), str(path))
            else:
                assets = dict(module.course_inputs())
                prefix = "projects/modern-d8-edition/"
                for rel in ("README.md", "learner/Cargo.toml", "learner/Cargo.lock",
                            "learner/rust-toolchain.toml", "learner/src/lib.rs",
                            "learner/tests/contracts.rs", "learner/examples/alarm.rs",
                            "learner/README.md", "learner/LEARNING_LOG.md"):
                    self.assertEqual(assets[prefix + rel], (BASE / rel).read_bytes())
                self.assertIn("course/BOOK_D8_EDITION_2024.md", assets)
                self.assertFalse(any(name.startswith(prefix + "reference/") for name in assets))
