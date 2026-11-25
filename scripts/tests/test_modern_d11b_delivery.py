"""D11-B scope and isolation; actual tools have independent CI."""
import ast
import importlib.util
from pathlib import Path
import tomllib
import unittest
import sys

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

    def test_security_remediation_gate_preserves_findings_and_rejects_old_versions(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        try:
            runner = load("modern_d11b_check.py")
        finally:
            sys.path.pop(0)
        clean = {label: dict(vulnerabilities=[], warnings={}, exit_code=0)
                 for label in runner.NETWORK_LOCK_LABELS}
        self.assertEqual(runner.network_gate(clean)["status"], "PASS")
        # A successful tool exit alone cannot qualify an unsound dependency.
        warning = {"advisory": {"id": "RUSTSEC-2026-0386"}}
        clean["d10b-learner"]["warnings"] = {"unsound": [warning]}
        gate = runner.network_gate(clean)
        self.assertEqual(gate["status"], "FAIL")
        self.assertEqual(gate["failed_locks"]["d10b-learner"]["unsound"], ["RUSTSEC-2026-0386"])
        self.assertEqual(clean["d10b-learner"]["warnings"]["unsound"], [warning])
        clean["c1"]["vulnerabilities"] = ["RUSTSEC-2026-0592"]
        clean["c1"]["exit_code"] = 1
        self.assertIn("c1", runner.network_gate(clean)["failed_locks"])

    def test_network_lock_versions_and_old_msrvs(self):
        for project, msrv in (("advanced-c1", "1.85"), ("advanced-c2", "1.85"),
                              ("modern-d10-tokio", "1.100")):
            root = ROOT / "projects" / project
            for name in ("Cargo.toml", "Cargo.lock"):
                self.assertEqual((root / "learner" / name).read_bytes(), (root / "reference" / name).read_bytes())
            manifest = tomllib.loads((root / "reference/Cargo.toml").read_text(encoding="utf-8"))
            self.assertEqual(manifest["package"]["rust-version"], msrv)
            self.assertEqual(manifest["dependencies"]["tokio"]["version"], "=1.53.3")
            packages = tomllib.loads((root / "reference/Cargo.lock").read_text(encoding="utf-8"))["package"]
            by_name = {}
            for package in packages:
                by_name.setdefault(package["name"], []).append(package)
            expected = {"tokio": "1.53.3", "tokio-macros": "2.7.0"}
            if project != "modern-d10-tokio":
                expected.update(url="2.5.9", form_urlencoded="1.2.3")
            for name, version in expected.items():
                self.assertEqual([p["version"] for p in by_name[name]], [version], (project, name))
        probe = tomllib.loads((ROOT / "apps/labquest/ecosystem/Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(probe["dependencies"]["tokio"]["version"], "=1.53.3")
        self.assertEqual(probe["dependencies"]["url"], "=2.5.9")

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
