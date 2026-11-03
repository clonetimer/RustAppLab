"""Regression checks for the C2 handoff and non-mutating CI."""
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parents[2]
class C2HandoffTests(unittest.TestCase):
    def test_bootstrap_generators_are_not_shipped(self):
        for name in ("scripts/c2_sync.py", ".github/workflows/c2-sync.yml", ".github/workflows/c2-bootstrap.yml", ".github/workflows/c2-green.yml"):
            self.assertFalse((ROOT/name).exists(), name)
    def test_only_main_pushes_and_prs_run_the_full_gate(self):
        text=(ROOT/".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertTrue("branches: [main]" in text)
        self.assertFalse("branches: [main, feat/" in text)
        self.assertTrue("needs: [native, browser, msrv, c1, c2]" in text)
    def test_completion_does_not_repeat_the_old_roadmap(self):
        text=(ROOT/"apps/labquest/src/advanced_c2.rs").read_text(encoding="utf-8")
        self.assertFalse("C2 Axum 尚未交付" in text)
        self.assertTrue("stage == 1 && !cfg!(unix)" in text)
    def test_learner_fixed_assets_match_reference(self):
        base=ROOT/"projects/advanced-c2"
        for name in ("Cargo.toml", "Cargo.lock", "src/main.rs", "src/model.rs", "src/handlers.rs", "tests/contracts.rs", "assets/index.html", "assets/client.js", "assets/app.css"):
            self.assertEqual((base/"reference"/name).read_bytes(), (base/"learner"/name).read_bytes(), name)
        starter=(base/"learner/src/lib.rs").read_text(encoding="utf-8")
        self.assertEqual(starter.count("TODO("), 3)
        self.assertNotIn("persist_checked", starter)
if __name__ == "__main__": unittest.main()
