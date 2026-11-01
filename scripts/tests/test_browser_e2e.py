import importlib.util
from pathlib import Path
import tempfile
import unittest
import sys

SCRIPT = Path(__file__).resolve().parents[1] / "browser_e2e.py"
sys.path.insert(0, str(SCRIPT.parent))
spec = importlib.util.spec_from_file_location("browser_e2e", SCRIPT)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class BrowserCspCompatibilityTests(unittest.TestCase):
    def test_browser_test_does_not_use_page_script_string_eval(self):
        source = SCRIPT.read_text(encoding="utf-8")
        self.assertNotIn("page.wait_for_function(", source)
        self.assertNotIn("page.evaluate(", source)
        self.assertIn("expect(page.locator('#count')).to_have_text('(2)'", source)
        self.assertIn("expect(page.locator('#tickets li.closed')).to_have_count(2", source)

class BrowserDiscoveryTests(unittest.TestCase):
    def test_explicit_existing_browser_is_used(self):
        with tempfile.TemporaryDirectory() as temp:
            browser = Path(temp) / ("browser.exe" if __import__('os').name == 'nt' else "browser")
            browser.write_bytes(b"test browser placeholder")
            self.assertEqual(module.find_browser(str(browser)), str(browser.resolve()))

    def test_missing_explicit_path_does_not_become_fake_browser(self):
        # Other installed real browsers may still be found; the function must only return
        # an existing file and never synthesize a path or disable browser policy.
        result = module.find_browser("/definitely/missing/rustapplab-browser")
        self.assertTrue(result is None or Path(result).is_file())

if __name__ == "__main__":
    unittest.main()
