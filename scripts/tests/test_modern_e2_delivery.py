"""E2 independent learner and native GUI evidence contract."""
import ast
import hashlib
import importlib.util
from pathlib import Path
import tomllib
import unittest
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/"projects/modern-e2-gui"
def load(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/"scripts"/name)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
class ModernE2Tests(unittest.TestCase):
    def test_independent_gui_and_fixed_contracts(self):
        self.assertNotIn("modern-e2-gui",(ROOT/"Cargo.toml").read_text(encoding="utf-8"))
        for side in ("reference","learner"):
            manifest=tomllib.loads((BASE/side/"Cargo.toml").read_text(encoding="utf-8"))
            self.assertEqual(manifest["dependencies"]["eframe"]["version"],"=0.37.0")
            self.assertEqual(manifest["dev-dependencies"]["egui_kittest"]["version"],"=0.37.0")
            self.assertEqual(manifest["package"]["rust-version"],"1.100")
        for name in ("Cargo.toml","Cargo.lock","src/main.rs","tests/contracts.rs"):
            self.assertEqual((BASE/"learner"/name).read_bytes(),(BASE/"reference"/name).read_bytes())
        self.assertEqual(sum((BASE/"learner"/name).read_text(encoding="utf-8").count("TODO(") for name in ("src/lib.rs","src/gui.rs")),4)
        self.assertIn("eframe::run_native",(BASE/"reference/src/main.rs").read_text(encoding="utf-8"))
        self.assertIn("Harness::new_ui",(BASE/"reference/tests/contracts.rs").read_text(encoding="utf-8"))
        lock=(BASE/"reference/Cargo.lock").read_bytes()
        self.assertEqual(hashlib.sha256(lock).hexdigest(),"0c547514c781df6d8571cb8dac307c66d671e95d22734e1f3bd098b80b81b210")
        self.assertEqual(sum("source" in p for p in tomllib.loads(lock.decode())["package"]),346)
        workflow=(ROOT/".github/workflows/modern-e2.yml").read_text(encoding="utf-8")
        self.assertIn("libwayland-dev",workflow)
        self.assertIn("pkg-config --exists wayland-client",workflow)
        self.assertIn("dbus-run-session -- xvfb-run",workflow)
        ast.parse((ROOT/"scripts/modern_e2_check.py").read_text(encoding="utf-8"))
    def test_learner_delivery(self):
        pack=load("package.py");assets=dict(load("package_learner_kit.py").course_inputs())
        for p in BASE.rglob("*"):
            if not p.is_file() or {"target",".git","__pycache__","vendor"}.intersection(p.relative_to(BASE).parts):continue
            self.assertTrue(pack.allowed(p),str(p))
            name=p.relative_to(ROOT).as_posix()
            if "modern-e2-gui/reference/" in name:self.assertNotIn(name,assets)
            else:self.assertEqual(assets[name],p.read_bytes())
        self.assertIn("course/BOOK_E2_GUI.md",assets)
