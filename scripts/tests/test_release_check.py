import importlib.util
from pathlib import Path
import tempfile
import unittest
import sys

spec = importlib.util.spec_from_file_location('release_check', Path(__file__).resolve().parents[1]/'release_check.py')
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)

class SummaryTests(unittest.TestCase):
    def test_accepts_real_unfiltered_summary(self):
        text='test result: ok. 210 passed; 0 failed; 15 ignored;\n'
        self.assertTrue(release.test_summary(text,0)['valid'])
    def test_timeout_cannot_be_misreported_as_pass(self):
        self.assertFalse(release.test_summary('test result: ok. 210 passed; 0 failed; 15 ignored;',124)['valid'])
    def test_filtering_all_tests_cannot_be_a_release_pass(self):
        self.assertFalse(release.test_summary('test result: ok. 0 passed; 0 failed; 15 ignored;',0)['valid'])
    def test_missing_or_failed_summary_is_not_a_pass(self):
        for text in ('', 'test result: FAILED. 210 passed; 1 failed; 15 ignored;'):
            self.assertFalse(release.test_summary(text,0)['valid'])
    def test_new_ignored_tests_fail_closed(self):
        self.assertFalse(release.test_summary('test result: ok. 210 passed; 0 failed; 16 ignored;',0)['valid'])
    def test_distinct_suites_sum_not_overwrite(self):
        text='test result: ok. 110 passed; 0 failed; 15 ignored;\ntest result: ok. 110 passed; 0 failed; 0 ignored;'
        self.assertEqual(release.test_summary(text,0)['passed'],220)
        self.assertTrue(release.test_summary(text,0)['valid'])
    def test_source_fingerprint_ignores_results_but_detects_code(self):
        with tempfile.TemporaryDirectory() as temp:
            r=Path(temp);(r/'apps/a/src').mkdir(parents=True);(r/'verification').mkdir()
            p=r/'apps/a/src/lib.rs';p.write_text('pub fn f() {}')
            before=release.source_id(r)
            (r/'verification/result.json').write_text('{}')
            self.assertEqual(before,release.source_id(r))
            p.write_text('pub fn f() { panic!() }')
            self.assertNotEqual(before,release.source_id(r))

    def test_source_fingerprint_covers_separate_workspace_and_embedded_lessons(self):
        # Removing projects/course or non-Rust embedded inputs from the fingerprint
        # would let evidence for old learner contracts qualify changed materials.
        paths = (
            'projects/advanced-b/reference/taskcore/src/lib.rs',
            'projects/advanced-b/learner/taskcore/tests/b1.rs',
            'projects/advanced-b/learner/Cargo.lock',
            'projects/advanced-b/ADVANCED_B.md',
            'course/RUST_ERROR_LABS.md',
            'apps/labquest/studio/README.md',
            'examples/foundation/tasks.tsv',
        )
        for relative in paths:
            with self.subTest(path=relative), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                target = root / relative
                target.parent.mkdir(parents=True)
                target.write_text('before', encoding='utf-8')
                before = release.source_id(root)
                target.write_text('after', encoding='utf-8')
                self.assertNotEqual(before, release.source_id(root))

    def test_source_fingerprint_ignores_nested_builds_and_unshipped_student_projects(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            before = release.source_id(root)
            for relative in ('projects/advanced-b/reference/target/generated.rs',
                             'projects/my-personal-app/src/main.rs',
                             'verification/current/evidence.json'):
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text('generated', encoding='utf-8')
            self.assertEqual(before, release.source_id(root))

    def test_advanced_workspace_requires_real_test_and_doctest_summaries(self):
        cases = (
            ('advanced-b-debug-default', '', False),
            ('advanced-b-debug-default', 'test result: ok. 0 passed; 0 failed; 0 ignored;', False),
            ('advanced-b-debug-strict', 'test result: ok. 12 passed; 0 failed; 1 ignored;', False),
            ('advanced-b-release-default', 'test result: FAILED. 12 passed; 1 failed; 0 ignored;', False),
            ('advanced-b-release-strict', 'test result: ok. 12 passed; 0 failed; 0 ignored;', True),
            ('advanced-b-doctests-default', 'test result: ok. 0 passed; 0 failed; 0 ignored;', False),
            ('advanced-b-doctests-strict', 'test result: ok. 1 passed; 0 failed; 0 ignored;', True),
        )
        for label, output, expected in cases:
            with self.subTest(step=label, output=output), tempfile.TemporaryDirectory() as temp:
                result = release.run_step(label, [sys.executable, '-c', f'print({output!r})'], Path(temp))
                self.assertEqual(result['passed'], expected)


# The release label alone must never promote an incomplete candidate.
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import release_decision
class DecisionTests(unittest.TestCase):
    def test_missing_browser_blocks_rc(self):
        self.assertEqual(release_decision.decide(dict(local_engineering=True,fresh_archive=True,real_browser=False)),'NOT_QUALIFIED')
    def test_complete_local_browser_archive_can_enter_rc(self):
        self.assertEqual(release_decision.decide(dict(local_engineering=True,fresh_archive=True,real_browser=True)),'RC_READY')
    def test_native_platforms_and_msrv_needed_for_stable(self):
        gates={x:True for x in ['local_engineering','fresh_archive','real_browser','windows_native','declared_msrv','learner_acceptance']}
        self.assertEqual(release_decision.decide(gates),'STABLE_READY')
        gates['macos_native']=False  # 不声明 macOS，不阻塞稳定资格
        self.assertEqual(release_decision.decide(gates),'STABLE_READY')
        gates['windows_native']=False
        self.assertEqual(release_decision.decide(gates),'RC_READY')
    def test_wrong_source_evidence_is_rejected(self):
        import json
        with tempfile.TemporaryDirectory() as temp:
            p=Path(temp)/'bad.json';p.write_text(json.dumps(dict(engineering_gate='PASS',version='rc',source_id='old',source_id_after='old',steps=[])))
            self.assertFalse(release_decision.engineering(p,'new','rc'))


class WindowsEntryTests(unittest.TestCase):
    def test_cmd_launchers_have_no_unintended_control_bytes(self):
        for script in release.ROOT.glob('*.cmd'):
            with self.subTest(script=script.name):
                bad=[x for x in script.read_bytes() if x < 32 and x not in (9,10,13)]
                self.assertEqual(bad, [], 'Escaped Python strings must not introduce vertical tabs into Windows paths')

    def test_cmd_powershell_targets_exist(self):
        import re
        found=set()
        for script in release.ROOT.glob('*.cmd'):
            for target in re.findall(r'-File\s+"%~dp0([^"\r\n]+)"',script.read_text(encoding='utf-8-sig')):
                found.add(script.name)
                with self.subTest(script=script.name,target=target):
                    self.assertTrue((release.ROOT/target.replace('\\','/')).is_file())
        self.assertEqual(found, {'START_WINDOWS.cmd','CONTINUE_WINDOWS.cmd','VERIFY_RELEASE_WINDOWS.cmd','VERIFY_BROWSER_WINDOWS.cmd','LEARN_NEXT_WINDOWS.cmd','START_ADVANCED_WINDOWS.cmd'})
        browser_runner=(release.ROOT/'scripts/verify_browser_windows.ps1').read_text(encoding='utf-8-sig')
        self.assertIn("$BrowserArgs += @('--browser-path', $Browser)", browser_runner)
        self.assertIn("$BrowserCode = Invoke-LoggedProcess 'browser-e2e' $Python.exe $BrowserArgs", browser_runner)
        self.assertNotIn("$Args += @('--browser-path', $Browser)", browser_runner)

if __name__=='__main__':unittest.main()
