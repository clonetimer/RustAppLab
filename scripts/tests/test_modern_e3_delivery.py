"""E3 project isolation, actual ARM configuration and protected distribution."""
import ast
import hashlib
import importlib.util
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "projects/modern-e3-embedded"


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ModernE3Tests(unittest.TestCase):
    def test_no_std_real_arm_and_same_contracts(self):
        self.assertNotIn("modern-e3-embedded", (ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference", "learner"):
            manifest = tomllib.loads((BASE / side / "Cargo.toml").read_text(encoding="utf-8"))
            self.assertEqual(manifest["package"]["rust-version"], "1.100")
            self.assertEqual(manifest["dependencies"]["embedded-hal"], "=1.0.0")
            target = manifest["target"]['cfg(all(target_arch = "arm", target_os = "none"))']["dependencies"]
            self.assertEqual(target["cortex-m"], "=0.7.7")
            self.assertEqual(target["cortex-m-rt"], "=0.7.5")
            self.assertEqual(target["cortex-m-semihosting"], "=0.5.0")
            self.assertIn("#![no_std]", (BASE / side / "src/lib.rs").read_text(encoding="utf-8"))
            firmware = (BASE / side / "src/bin/firmware.rs").read_text(encoding="utf-8")
            self.assertIn("#![no_main]", firmware)
            self.assertIn("#[exception]", firmware)
            self.assertIn("enable_interrupt()", firmware)
        for p in ("Cargo.toml", "build.rs", "memory.x", "tests/contracts.rs",
                  "src/sim.rs", "src/bin/firmware.rs", "examples/host_demo.rs"):
            self.assertEqual((BASE / "reference" / p).read_bytes(), (BASE / "learner" / p).read_bytes())
        self.assertEqual((BASE / "learner/src/lib.rs").read_text(encoding="utf-8").count("TODO("), 5)
        self.assertEqual((BASE / "reference/tests/contracts.rs").read_text(encoding="utf-8").count("fn step"), 5)
        lock = (BASE / "reference/Cargo.lock").read_bytes()
        self.assertEqual(lock, (BASE / "learner/Cargo.lock").read_bytes())
        self.assertEqual(hashlib.sha256(lock).hexdigest(), "db2e90bc00344d3d33fa853f1a513cbb03780c078d491a08ca932293aa22909a")
        self.assertEqual(sum("source" in p for p in tomllib.loads(lock.decode())["package"]), 20)
        ast.parse((ROOT / "scripts/modern_e3_check.py").read_text(encoding="utf-8"))

    def test_source_and_learner_kit(self):
        pack = load("package.py")
        assets = dict(load("package_learner_kit.py").course_inputs())
        for p in BASE.rglob("*"):
            if not p.is_file() or {"target", ".git", "__pycache__", "vendor"}.intersection(p.relative_to(BASE).parts):
                continue
            self.assertTrue(pack.allowed(p), str(p))
            name = p.relative_to(ROOT).as_posix()
            if "modern-e3-embedded/reference/" in name:
                self.assertNotIn(name, assets)
            else:
                self.assertEqual(assets[name], p.read_bytes())
        self.assertIn("course/BOOK_E3_EMBEDDED.md", assets)
