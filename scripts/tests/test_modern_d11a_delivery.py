"""D11-A isolation, fixed contracts, delivery and planned evidence steps."""
import ast
import importlib.util
from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-d11-quality"


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ModernD11ATests(unittest.TestCase):
    def test_fixed_project_and_contracts(self):
        self.assertNotIn("modern-d11-quality", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference", "learner", "tools-practice"):
            self.assertIn('channel = "1.100.0"', (BASE / side / "rust-toolchain.toml").read_text(encoding="utf-8"))
            manifest = tomllib.loads((BASE / side / "Cargo.toml").read_text(encoding="utf-8"))
            self.assertEqual(manifest["package"]["edition"], "2024")
            self.assertNotIn("dependencies", manifest)
        for name in ("Cargo.lock", "Cargo.toml", "tests/contracts.rs", "tests/properties.rs"):
            self.assertEqual((BASE / "reference" / name).read_bytes(), (BASE / "learner" / name).read_bytes())
        contract = (BASE / "learner/tests/contracts.rs").read_text(encoding="utf-8")
        self.assertEqual(set(re.findall(r"fn step(\d+)_", contract)), set("1234"))
        starter = (BASE / "learner/src/lib.rs").read_text(encoding="utf-8")
        self.assertEqual(starter.count("TODO("), 4)
        self.assertNotEqual(starter, (BASE / "reference/src/lib.rs").read_text(encoding="utf-8"))
        runner = (ROOT / "scripts/modern_d11a_check.py").read_text(encoding="utf-8")
        ast.parse(runner)
        for required in ("mutant-", "properties-restored", "format-red", "clippy-red",
                         "release-measure", "gdb-breakpoint", "interactive_ide"):
            self.assertIn(required, runner)

    def test_source_and_learner_kit(self):
        pack = load("package.py")
        assets = dict(load("package_learner_kit.py").course_inputs())
        prefix = "projects/modern-d11-quality/"
        for p in BASE.rglob("*"):
            if not p.is_file() or {"target", ".git", "__pycache__"}.intersection(p.relative_to(BASE).parts):
                continue
            self.assertTrue(pack.allowed(p), str(p))
            name = p.relative_to(ROOT).as_posix()
            if name.startswith(prefix + "reference/"):
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], p.read_bytes())
        self.assertIn("course/BOOK_D11_QUALITY.md", assets)
