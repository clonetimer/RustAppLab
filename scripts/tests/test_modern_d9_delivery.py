"""D9-A packaging/contract scope; real Rust runs separately on both platforms."""
import ast
import importlib.util
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-d9-stable"


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ModernD9DeliveryTests(unittest.TestCase):
    def test_independent_and_fixed_contracts(self):
        self.assertNotIn("modern-d9-stable", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference", "learner"):
            self.assertIn('channel = "1.100.0"', (BASE / side / "rust-toolchain.toml").read_text(encoding="utf-8"))
            cargo = (BASE / side / "Cargo.toml").read_text(encoding="utf-8")
            for text in ('edition = "2024"', 'rust-version = "1.100"', 'resolver = "3"', 'publish = false'):
                self.assertIn(text, cargo)
            self.assertNotIn("[dependencies]", cargo)
        contract = BASE / "learner/tests/contracts.rs"
        self.assertEqual(contract.read_bytes(), (BASE / "reference/tests/contracts.rs").read_bytes())
        self.assertEqual(set(re.findall(r"fn step(\d+)_", contract.read_text(encoding="utf-8"))),
                         {str(i) for i in range(1, 10)})
        self.assertEqual((BASE / "learner/src/lib.rs").read_text(encoding="utf-8").count("TODO("), 9)
        self.assertNotEqual((BASE / "learner/src/lib.rs").read_bytes(), (BASE / "reference/src/lib.rs").read_bytes())
        self.assertEqual(len(list((BASE / "reference/probes").glob("*/red.rs"))), 8)
        self.assertEqual(len(list((BASE / "reference/probes").glob("*/green.rs"))), 9)
        ast.parse((ROOT / "scripts/modern_d9_check.py").read_text(encoding="utf-8"))

    def test_version_index_and_delivery(self):
        index = (ROOT / "course/MODERN_STABLE_FEATURE_INDEX.md").read_text(encoding="utf-8")
        self.assertEqual(re.findall(r"^\| 1\.(\d+)\.0 \|", index, re.M),
                         [str(i) for i in range(75, 101)])
        pack = load("package.py")
        for p in BASE.rglob("*"):
            if p.is_file() and not {"target", "__pycache__", ".git"}.intersection(p.relative_to(BASE).parts):
                self.assertTrue(pack.allowed(p), str(p))
        assets = dict(load("package_learner_kit.py").course_inputs())
        prefix = "projects/modern-d9-stable/"
        for p in (BASE / "learner").rglob("*"):
            if p.is_file() and not {"target", "__pycache__", ".git"}.intersection(p.relative_to(BASE).parts):
                self.assertEqual(assets[p.relative_to(ROOT).as_posix()], p.read_bytes())
        self.assertFalse(any(name.startswith(prefix + "reference/") for name in assets))
        for doc in ("course/BOOK_D9_STABLE_FEATURES.md", "course/MODERN_STABLE_FEATURE_INDEX.md",
                    prefix + "README.md"):
            self.assertIn(doc, assets)
