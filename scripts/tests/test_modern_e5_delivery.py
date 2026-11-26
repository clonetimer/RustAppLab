"""E5 source inventory and learner isolation (actual algorithms in specialist CI)."""
import importlib.util
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-e5-science"
def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod
class ModernE5Tests(unittest.TestCase):
    def test_delivery_and_fixed_contracts(self):
        self.assertNotIn("modern-e5-science", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        assets = dict(load("package_learner_kit.py").course_inputs())
        package = load("package.py")
        for p in BASE.rglob("*"):
            if not p.is_file() or {"target", ".git", "__pycache__"}.intersection(p.relative_to(BASE).parts):
                continue
            self.assertTrue(package.allowed(p), str(p))
            name = p.relative_to(ROOT).as_posix()
            if "/reference/" in name:
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], p.read_bytes())
        for rel in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src/synthetic.rs", "tests/contracts.rs", "examples/faultscope.rs"):
            self.assertEqual((BASE / "reference" / rel).read_bytes(), (BASE / "learner" / rel).read_bytes())
        self.assertEqual((BASE / "learner/src/lib.rs").read_text(encoding="utf-8").count("TODO("), 4)
        self.assertIn("course/BOOK_E5_SCIENCE.md", assets)
