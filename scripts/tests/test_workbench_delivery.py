"""Keep the maintainer evidence and learner distribution contracts independent."""
import json
from pathlib import Path
import tempfile
import unittest
import zipfile
import sys

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
import assemble_candidate
import candidate_check
import package
import workbench_browser_e2e as browser


class WorkbenchDeliveryTests(unittest.TestCase):
    def test_distribution_contains_source_assets_and_native_launcher(self):
        for source in ('START_WORKBENCH_WINDOWS.cmd', 'START_WORKBENCH.sh',
                       'apps/labquest/src/workbench/ui/index.html',
                       'apps/labquest/src/workbench/ui/app.js',
                       'apps/labquest/src/workbench/ui/app.css',
                       'scripts/tests/workbench_ui_state.cjs'):
            self.assertTrue(package.allowed(package.ROOT/source), source)
        for system, launcher in (('Windows','START_WORKBENCH_WINDOWS.cmd'),
                                 ('Linux','START_WORKBENCH.sh')):
            with self.subTest(system=system), tempfile.TemporaryDirectory() as tmp:
                path = Path(tmp)/'runtime.zip'
                candidate_check.pack_runtime(path, {}, {'system':system,'machine':'x64','version':'test'}, {})
                with zipfile.ZipFile(path) as archive:
                    self.assertEqual(archive.read(launcher), (package.ROOT/launcher).read_bytes())
                    self.assertIn(launcher, archive.read('FILES_SHA256SUMS.txt').decode())
                    self.assertIn(launcher, archive.read('START_RUNTIME.md').decode())
                    if system == 'Linux':
                        self.assertTrue((archive.getinfo(launcher).external_attr >> 16) & 0o111)

    def test_new_browser_evidence_rejects_wrong_revision_or_incomplete_flow(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'workbench.json'
            for after, message in (('old-source', 'wrong or changed source'),
                                   ('current', 'incomplete workbench')):
                path.write_text(json.dumps({'source_id':'current', 'source_id_after':after,
                                            'status':'PASS', 'checks':['localhub','servicedesk']}))
                with self.assertRaisesRegex(ValueError, message):
                    assemble_candidate.read_workbench_report(path,'current')

    def test_static_baseline_uses_pre_workbench_git_objects(self):
        # Source ZIPs used by fresh-archive qualification need no git history.
        if not (package.ROOT/'.git').exists():
            self.skipTest('source archive has no git history')
        import subprocess
        exists = subprocess.run(['git','cat-file','-e', browser.PRE_WORKBENCH+'^{commit}'],
                                cwd=package.ROOT, capture_output=True)
        if exists.returncode != 0:
            self.skipTest('shallow checkout; browser/evidence jobs fetch baseline history')
        with tempfile.TemporaryDirectory() as tmp:
            _, pages = browser.baseline_pages(Path(tmp), browser.PRE_WORKBENCH)
            self.assertEqual(set(pages), {'firstcard','unitlab'})
            for course, item in pages.items():
                self.assertEqual(item['source'], browser.BASE_PAGES[course])
                self.assertEqual(len(item['sha256']), 64)


if __name__ == '__main__':
    unittest.main()
