"""D9-B distribution and audit coverage; Rust behavior has its own two-OS CI."""
import ast
import importlib.util
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-d9-stdlib"


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ModernD9BTests(unittest.TestCase):
    def test_contracts_and_version_scope(self):
        self.assertNotIn("modern-d9-stdlib", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("learner", "reference", "cargo-practice"):
            self.assertIn('channel = "1.100.0"', (BASE / side / "rust-toolchain.toml").read_text(encoding="utf-8"))
        contract = (BASE / "learner/tests/contracts.rs").read_bytes()
        self.assertEqual(contract, (BASE / "reference/tests/contracts.rs").read_bytes())
        self.assertEqual(set(re.findall(r"fn step(\d+)_", contract.decode())), {str(i) for i in range(1, 17)})
        self.assertEqual((BASE / "learner/src/lib.rs").read_text(encoding="utf-8").count("TODO("), 17)
        self.assertNotEqual((BASE / "learner/src/lib.rs").read_bytes(), (BASE / "reference/src/lib.rs").read_bytes())
        index = (ROOT / "course/MODERN_STABLE_FEATURE_INDEX.md").read_text(encoding="utf-8")
        self.assertEqual(re.findall(r"^\| 1\.(\d+)\.0 \|", index, re.M), [str(i) for i in range(75, 101)])
        for n in range(75, 101):
            self.assertIn(f"https://github.com/rust-lang/rust/releases/tag/1.{n}.0", index)
        ast.parse((ROOT / "scripts/modern_d9b_check.py").read_text(encoding="utf-8"))

    def test_packaging_and_no_reference_leak(self):
        pack = load("package.py")
        assets = dict(load("package_learner_kit.py").course_inputs())
        prefix = "projects/modern-d9-stdlib/"
        for path in BASE.rglob("*"):
            if not path.is_file() or {"target", ".git", "__pycache__"}.intersection(path.relative_to(BASE).parts):
                continue
            self.assertTrue(pack.allowed(path), str(path))
            name = path.relative_to(ROOT).as_posix()
            if name.startswith(prefix + "reference/"):
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], path.read_bytes())
        self.assertIn("course/BOOK_D9_STDLIB_CARGO.md", assets)
        self.assertIn(prefix + "cargo-practice/.cargo/shared.toml", assets)
