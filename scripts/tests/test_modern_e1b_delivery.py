"""E1-B source/learner scope and reuse of the existing TLS/domain contracts."""
import ast
import importlib.util
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-e1-sqlite"

def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

class ModernE1BTests(unittest.TestCase):
    def test_scope_and_fixed_contracts(self):
        self.assertNotIn("modern-e1-sqlite", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference","learner"):
            manifest = tomllib.loads((BASE / side / "Cargo.toml").read_text(encoding="utf-8"))
            self.assertEqual(manifest["package"]["rust-version"], "1.100")
            self.assertEqual(manifest["dependencies"]["rusqlite"]["version"], "=0.43.0")
            self.assertEqual(manifest["dependencies"]["rusqlite"]["features"], ["bundled"])
            self.assertEqual(manifest["dependencies"]["e1-secure-lab"]["path"], "../../modern-e1-tls/reference")
            self.assertEqual(manifest["dependencies"]["servicedesk"]["path"], "../../../apps/servicedesk")
        for name in ("Cargo.toml", "tests/contracts.rs", "tests/support/mod.rs", "src/main.rs"):
            self.assertEqual((BASE / "reference" / name).read_bytes(), (BASE / "learner" / name).read_bytes())
        self.assertEqual((BASE / "learner/src/lib.rs").read_text(encoding="utf-8").count("TODO("), 4)
        self.assertEqual((BASE / "reference/tests/contracts.rs").read_text(encoding="utf-8").count("fn step"), 5)
        self.assertIn("protect_router(app, credentials)", (BASE / "reference/src/lib.rs").read_text(encoding="utf-8"))
        ast.parse((ROOT / "scripts/modern_e1b_check.py").read_text(encoding="utf-8"))

    def test_learner_delivery(self):
        pack = load("package.py")
        assets = dict(load("package_learner_kit.py").course_inputs())
        prefix = "projects/modern-e1-sqlite/"
        for p in BASE.rglob("*"):
            if not p.is_file() or {"target", ".git", "__pycache__"}.intersection(p.relative_to(BASE).parts):
                continue
            self.assertTrue(pack.allowed(p), str(p))
            name = p.relative_to(ROOT).as_posix()
            if name.startswith(prefix + "reference/"):
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], p.read_bytes())
        self.assertFalse(any("modern-e1-tls/reference/" in n or "advanced-c2/reference/" in n for n in assets))
        self.assertIn("course/BOOK_E1_SQLITE.md", assets)
