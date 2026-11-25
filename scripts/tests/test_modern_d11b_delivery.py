"""D11-B scope and isolation; actual tools have independent CI."""
import ast
import importlib.util
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-d11-safety"


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ModernD11BTests(unittest.TestCase):
    def test_contracts_and_specialist_isolation(self):
        self.assertNotIn("modern-d11-safety", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference", "learner"):
            manifest = tomllib.loads((BASE / side / "Cargo.toml").read_text(encoding="utf-8"))
            self.assertEqual(manifest["package"]["edition"], "2024")
            self.assertNotIn("dependencies", manifest)
            self.assertIn('channel = "1.100.0"', (BASE / side / "rust-toolchain.toml").read_text(encoding="utf-8"))
        self.assertEqual((BASE / "reference/tests/contracts.rs").read_bytes(),
                         (BASE / "learner/tests/contracts.rs").read_bytes())
        self.assertEqual((BASE / "learner/src/lib.rs").read_text(encoding="utf-8").count("TODO("), 2)
        self.assertFalse((BASE / "learner/examples").exists())
        for probe in (BASE / "miri-practice/probes").glob("*.rs"):
            self.assertIn("MIRI-ONLY", probe.read_text(encoding="utf-8"))
        fuzz = tomllib.loads((BASE / "fuzz-practice/fuzz/Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(fuzz["dependencies"]["libfuzzer-sys"], "=0.4.13")
        self.assertEqual(fuzz["dependencies"]["d11-quality-lab"]["path"], "..")
        audit = tomllib.loads((BASE / "audit-practice/Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(audit["dependencies"]["regex"], "=1.5.4")
        runner = (ROOT / "scripts/modern_d11b_check.py").read_text(encoding="utf-8")
        ast.parse(runner)
        for text in ("Undefined Behavior", "fuzz-mutant", "fuzz-counterexample.bin",
                     "fuzz-replay-corrected", "RUSTSEC-2022-0013", "fixture_built_or_run=False",
                     '"--no-fetch"', '"--no-yanked"', "source_unchanged"):
            self.assertIn(text, runner)

    def test_packaging(self):
        pack = load("package.py")
        assets = dict(load("package_learner_kit.py").course_inputs())
        prefix = "projects/modern-d11-safety/"
        for p in BASE.rglob("*"):
            if not p.is_file() or {"target", ".git", "__pycache__"}.intersection(p.relative_to(BASE).parts):
                continue
            self.assertTrue(pack.allowed(p), str(p))
            name = p.relative_to(ROOT).as_posix()
            if name.startswith(prefix + "reference/"):
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], p.read_bytes())
        self.assertIn("course/BOOK_D11_SAFETY.md", assets)
