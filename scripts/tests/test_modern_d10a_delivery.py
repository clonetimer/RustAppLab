"""D10-A packaging and scope; actual Rust tests run separately on both platforms."""
import ast
import importlib.util
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-d10-async-core"


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ModernD10ATests(unittest.TestCase):
    def test_contracts_and_fixed_toolchain(self):
        self.assertNotIn("modern-d10-async-core", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference", "learner"):
            self.assertIn('channel = "1.100.0"', (BASE / side / "rust-toolchain.toml").read_text(encoding="utf-8"))
            manifest = (BASE / side / "Cargo.toml").read_text(encoding="utf-8")
            self.assertIn('edition = "2024"', manifest)
            self.assertNotIn("[dependencies]", manifest)
        contract = (BASE / "learner/tests/contracts.rs").read_bytes()
        self.assertEqual(contract, (BASE / "reference/tests/contracts.rs").read_bytes())
        self.assertEqual(set(re.findall(r"fn step(\d+)_", contract.decode())), set("123456"))
        self.assertEqual((BASE / "learner/src/runtime.rs").read_bytes(),
                         (BASE / "reference/src/runtime.rs").read_bytes())
        starter = (BASE / "learner/src/lib.rs").read_text(encoding="utf-8")
        self.assertEqual(starter.count("TODO("), 6)
        self.assertNotEqual(starter, (BASE / "reference/src/lib.rs").read_text(encoding="utf-8"))
        self.assertEqual(len(list((BASE / "reference/probes").glob("*/red.rs"))), 4)
        self.assertEqual(len(list((BASE / "reference/probes").glob("*/green.rs"))), 4)
        ast.parse((ROOT / "scripts/modern_d10a_check.py").read_text(encoding="utf-8"))

    def test_packaging_and_no_reference_leak(self):
        pack = load("package.py")
        assets = dict(load("package_learner_kit.py").course_inputs())
        prefix = "projects/modern-d10-async-core/"
        for path in BASE.rglob("*"):
            if not path.is_file() or {"target", ".git", "__pycache__"}.intersection(path.relative_to(BASE).parts):
                continue
            self.assertTrue(pack.allowed(path), str(path))
            name = path.relative_to(ROOT).as_posix()
            if name.startswith(prefix + "reference/"):
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], path.read_bytes())
        self.assertIn("course/BOOK_D10_ASYNC_CORE.md", assets)
