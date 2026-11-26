"""E1-A source/learner scope; real TLS and dependency resolution in dedicated CI."""
import ast
import hashlib
import importlib.util
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-e1-tls"


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ModernE1ATests(unittest.TestCase):
    def test_fixed_scope_and_contracts(self):
        self.assertNotIn("modern-e1-tls", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference", "learner"):
            manifest = tomllib.loads((BASE / side / "Cargo.toml").read_text(encoding="utf-8"))
            self.assertEqual(manifest["package"]["rust-version"], "1.100")
            self.assertEqual(manifest["dependencies"]["tokio"]["version"], "=1.53.3")
            self.assertEqual(manifest["dependencies"]["rustls"]["version"], "=0.23.46")
            self.assertEqual(manifest["dependencies"]["tokio-rustls"]["version"], "=0.26.6")
            self.assertEqual(manifest["dependencies"]["servicedesk-c2"]["path"], "../../advanced-c2/reference")
        for p in ("Cargo.toml", "Cargo.lock", "tests/contracts.rs", "tests/support/mod.rs", "src/main.rs", "examples/local_demo.rs"):
            self.assertEqual((BASE / "learner" / p).read_bytes(), (BASE / "reference" / p).read_bytes())
        self.assertEqual((BASE / "learner/src/lib.rs").read_text(encoding="utf-8").count("TODO("), 4)
        self.assertNotEqual((BASE / "learner/src/lib.rs").read_bytes(), (BASE / "reference/src/lib.rs").read_bytes())
        self.assertEqual((BASE / "reference/tests/contracts.rs").read_text(encoding="utf-8").count("fn step"), 5)
        frozen = (BASE / "reference/Cargo.lock").read_bytes()
        self.assertEqual(hashlib.sha256(frozen).hexdigest(), "44ef895eb533fcf092d77f52c2488513d9dddb1b8faa0ad034d4cf7615d2c3fd")
        self.assertEqual(len([p for p in tomllib.loads(frozen.decode())["package"] if p.get("source")]), 101)
        self.assertIn("init_store(&journal)", (BASE / "reference/tests/contracts.rs").read_text(encoding="utf-8"))
        ast.parse((ROOT / "scripts/modern_e1a_check.py").read_text(encoding="utf-8"))

    def test_packaging_no_e1_answers_or_c2_answers_in_global_learner(self):
        pack = load("package.py")
        assets = dict(load("package_learner_kit.py").course_inputs())
        for p in BASE.rglob("*"):
            if not p.is_file() or {"target", ".git", "__pycache__"}.intersection(p.relative_to(BASE).parts):
                continue
            self.assertTrue(pack.allowed(p), str(p))
            name = p.relative_to(ROOT).as_posix()
            if "modern-e1-tls/reference/" in name:
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], p.read_bytes())
        self.assertFalse(any("advanced-c2/reference/" in n for n in assets))
        self.assertIn("course/BOOK_E1_TLS_AUTH.md", assets)
