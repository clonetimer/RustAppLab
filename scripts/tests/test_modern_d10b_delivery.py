"""D10-B fixed dependency graph, contract/learner separation and packaging."""
import ast
import importlib.util
from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-d10-tokio"


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ModernD10BTests(unittest.TestCase):
    def test_fixed_contracts_and_dependency_checksums(self):
        self.assertNotIn("modern-d10-tokio", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference", "learner"):
            self.assertIn('channel = "1.100.0"', (BASE / side / "rust-toolchain.toml").read_text(encoding="utf-8"))
            manifest = tomllib.loads((BASE / side / "Cargo.toml").read_text(encoding="utf-8"))
            self.assertEqual(manifest["package"]["edition"], "2024")
            self.assertEqual(manifest["dependencies"]["tokio"]["version"], "=1.48.0")
            self.assertEqual(manifest["dependencies"]["futures-util"]["version"], "=0.3.34")
        for path in ("Cargo.lock", "Cargo.toml", "tests/contracts.rs"):
            self.assertEqual((BASE / "reference" / path).read_bytes(), (BASE / "learner" / path).read_bytes())
        old = tomllib.loads((ROOT / "projects/advanced-c1/reference/Cargo.lock").read_text(encoding="utf-8"))
        old_registry = {(p["name"], p["version"], p["source"], p["checksum"])
                        for p in old["package"] if p.get("source")}
        current = tomllib.loads((BASE / "reference/Cargo.lock").read_text(encoding="utf-8"))
        registry = {(p["name"], p["version"], p["source"], p["checksum"])
                    for p in current["package"] if p.get("source")}
        self.assertTrue(registry)
        self.assertTrue(registry.issubset(old_registry))
        contract = (BASE / "learner/tests/contracts.rs").read_text(encoding="utf-8")
        self.assertEqual(set(re.findall(r"fn step(\d+)_", contract)), set("1234567"))
        starter = (BASE / "learner/src/lib.rs").read_text(encoding="utf-8")
        self.assertEqual(starter.count("TODO("), 9)
        self.assertNotEqual(starter, (BASE / "reference/src/lib.rs").read_text(encoding="utf-8"))
        self.assertEqual(len(list((BASE / "reference/probes").glob("*/red.rs"))), 2)
        self.assertEqual(len(list((BASE / "reference/probes").glob("*/green.rs"))), 2)
        ast.parse((ROOT / "scripts/modern_d10b_check.py").read_text(encoding="utf-8"))

    def test_frozen_proof_checks_actual_metadata_and_cache_contents(self):
        runner = (ROOT / "scripts/modern_d10b_check.py").read_text(encoding="utf-8")
        for requirement in ('"metadata", "--frozen"', 'manifest.is_relative_to(vendor)',
                            'unexpected_registry_entries', '"CACHEDIR.TAG"',
                            'verify_vendor_resolution(evidence, "reference"',
                            'verify_vendor_resolution(evidence, "learner"',
                            'initially empty'):
            self.assertIn(requirement, runner)

    def test_source_and_learner_kit(self):
        pack = load("package.py")
        assets = dict(load("package_learner_kit.py").course_inputs())
        prefix = "projects/modern-d10-tokio/"
        for p in BASE.rglob("*"):
            if not p.is_file() or {"target", ".git", "__pycache__"}.intersection(p.relative_to(BASE).parts):
                continue
            self.assertTrue(pack.allowed(p), str(p))
            name = p.relative_to(ROOT).as_posix()
            if name.startswith(prefix + "reference/"):
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], p.read_bytes())
        self.assertIn("course/BOOK_D10_TOKIO_STREAM.md", assets)
