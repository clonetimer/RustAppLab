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
            'projects/advanced-b/reference/json-export/Cargo.lock',
            'projects/advanced-b/learner/json-export/src/lib.rs',
            'projects/advanced-b/learner/taskcore/tests/b1.rs',
            'projects/advanced-b/learner/Cargo.lock',
            'projects/advanced-b/ADVANCED_B.md',
            'projects/book-d2-smart-pointers/smart-pointer-lab/src/lib.rs',
            'projects/book-d3-oop-patterns/oop-pattern-lab/src/lib.rs',
            'projects/book-d4-advanced-features/advanced-lab/src/lib.rs',
            'projects/book-d4-advanced-features/d4-proc-macros/src/lib.rs',
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


class B4GateTests(unittest.TestCase):
    def test_explicit_complete_independent_workspace_matrix(self):
        full = release.advanced_b4_steps()
        basic = release.advanced_b4_steps(quality=False)
        self.assertEqual(len(full), 10)
        self.assertEqual(len(basic), 7)
        self.assertEqual(len({name for name, _ in full}), 10)
        for name, command in full:
            self.assertTrue(name.startswith('advanced-b4-'))
            self.assertIn(name, release_decision.REQUIRED)
            if name == 'advanced-b4-platform-e2e':
                self.assertEqual(command, ['cargo','test','--locked','--offline','-p','labquest','--features','b4-integration','--test','advanced_b_json_e2e','--','--test-threads=1'])
                self.assertEqual(release.TEST_LIMITS[name], (5, 0))
            else:
                self.assertIn('projects/advanced-b/reference/json-export/Cargo.toml', command)
                if command[1] != 'fmt':
                    self.assertIn('--locked', command)
                    self.assertIn('--offline', command)
        for profile in ('debug', 'release'):
            for config in ('default', 'strict'):
                name = f'advanced-b4-{profile}-{config}'
                self.assertEqual(release.TEST_LIMITS[name], (20, 0))
                self.assertIn(name, dict(basic))
        for config in ('default', 'strict'):
            self.assertEqual(release.TEST_LIMITS['advanced-b4-doctests-'+config], (1, 0))

    def test_b4_zero_tests_or_ignored_cannot_qualify(self):
        for label in ('advanced-b4-debug-default', 'advanced-b4-doctests-strict', 'advanced-b4-platform-e2e'):
            for output in ('', 'test result: ok. 0 passed; 0 failed; 0 ignored;', 'test result: ok. 30 passed; 0 failed; 1 ignored;'):
                with self.subTest(label=label, output=output), tempfile.TemporaryDirectory() as temp:
                    result = release.run_step(label, [sys.executable, '-c', f'print({output!r})'], Path(temp))
                    self.assertFalse(result['passed'])

    def test_book_d1_workspace_has_install_and_package_checks(self):
        full = release.book_d1_steps()
        basic = release.book_d1_steps(quality=False)
        self.assertEqual([name for name, _ in full],
                         ['book-d1-format', 'book-d1-clippy', 'book-d1-tests',
                          'book-d1-doctests', 'book-d1-install-smoke', 'book-d1-package-list'])
        self.assertEqual([name for name, _ in basic],
                         ['book-d1-tests', 'book-d1-doctests', 'book-d1-install-smoke'])
        self.assertEqual(release.TEST_LIMITS['book-d1-tests'], (10, 0))
        self.assertEqual(release.TEST_LIMITS['book-d1-doctests'], (1, 0))
        for name in ('book-d1-format', 'book-d1-clippy', 'book-d1-tests',
                     'book-d1-doctests', 'book-d1-install-smoke', 'book-d1-package-list'):
            self.assertIn(name, release_decision.REQUIRED)

    def test_book_d2_workspace_has_explicit_release_and_msrv_matrix(self):
        full = release.book_d2_steps()
        basic = release.book_d2_steps(quality=False)
        self.assertEqual([name for name, _ in full],
                         ['book-d2-format', 'book-d2-clippy', 'book-d2-tests', 'book-d2-doctests'])
        self.assertEqual([name for name, _ in basic],
                         ['book-d2-tests', 'book-d2-doctests'])
        self.assertEqual(release.TEST_LIMITS['book-d2-tests'], (26, 0))
        self.assertEqual(release.TEST_LIMITS['book-d2-doctests'], (1, 0))
        for name, command in full:
            self.assertTrue(name.startswith('book-d2-'))
            self.assertIn('projects/book-d2-smart-pointers/Cargo.toml', command)
            if command[1] != 'fmt':
                self.assertIn('--locked', command)
                self.assertIn('--offline', command)

    def test_book_d3_workspace_has_explicit_release_and_msrv_matrix(self):
        full = release.book_d3_steps()
        basic = release.book_d3_steps(quality=False)
        self.assertEqual([name for name, _ in full],
                         ['book-d3-format', 'book-d3-clippy', 'book-d3-tests', 'book-d3-doctests'])
        self.assertEqual([name for name, _ in basic],
                         ['book-d3-tests', 'book-d3-doctests'])
        self.assertEqual(release.TEST_LIMITS['book-d3-tests'], (44, 0))
        self.assertEqual(release.TEST_LIMITS['book-d3-doctests'], (1, 0))
        for name, command in full:
            self.assertTrue(name.startswith('book-d3-'))
            self.assertIn('projects/book-d3-oop-patterns/Cargo.toml', command)
            if command[1] != 'fmt':
                self.assertIn('--locked', command)
                self.assertIn('--offline', command)

    def test_book_d4_workspace_has_explicit_release_and_msrv_matrix(self):
        full = release.book_d4_steps()
        basic = release.book_d4_steps(quality=False)
        self.assertEqual([name for name, _ in full],
                         ['book-d4-format', 'book-d4-clippy', 'book-d4-tests', 'book-d4-doctests'])
        self.assertEqual([name for name, _ in basic],
                         ['book-d4-tests', 'book-d4-doctests'])
        self.assertEqual(release.TEST_LIMITS['book-d4-tests'], (39, 0))
        self.assertEqual(release.TEST_LIMITS['book-d4-doctests'], (2, 0))
        for name, command in full:
            self.assertTrue(name.startswith('book-d4-'))
            self.assertIn('projects/book-d4-advanced-features/Cargo.toml', command)
            if command[1] != 'fmt':
                self.assertIn('--locked', command)
                self.assertIn('--offline', command)

    def test_book_d5_d6_workspaces_have_explicit_release_and_msrv_matrix(self):
        expected = {
            'book_d5_steps': (
                ['book-d5-format', 'book-d5-clippy', 'book-d5-tests',
                 'book-d5-release-tests', 'book-d5-doctests'],
                ['book-d5-tests', 'book-d5-release-tests', 'book-d5-doctests'],
                'projects/book-d5-webserver/Cargo.toml',
            ),
            'book_d6_steps': (
                ['book-d6-format', 'book-d6-clippy', 'book-d6-tests',
                 'book-d6-release-tests', 'book-d6-doctests'],
                ['book-d6-tests', 'book-d6-release-tests', 'book-d6-doctests'],
                'projects/book-d6-appendices/Cargo.toml',
            ),
        }
        for function, (full_names, basic_names, manifest) in expected.items():
            full = getattr(release, function)()
            basic = getattr(release, function)(quality=False)
            self.assertEqual([name for name, _ in full], full_names)
            self.assertEqual([name for name, _ in basic], basic_names)
            for name, command in full:
                self.assertIn(manifest, command)
                if command[1] != 'fmt':
                    self.assertIn('--locked', command)
                    self.assertIn('--offline', command)
        self.assertEqual(release.TEST_LIMITS['book-d5-tests'], (20, 0))
        self.assertEqual(release.TEST_LIMITS['book-d5-release-tests'], (20, 0))
        self.assertEqual(release.TEST_LIMITS['book-d5-doctests'], (1, 0))
        self.assertEqual(release.TEST_LIMITS['book-d6-tests'], (11, 0))
        self.assertEqual(release.TEST_LIMITS['book-d6-release-tests'], (11, 0))
        self.assertEqual(release.TEST_LIMITS['book-d6-doctests'], (3, 0))

    def test_book_d6_foundations_workspace_has_explicit_release_and_msrv_matrix(self):
        full = release.book_d6_foundations_steps()
        basic = release.book_d6_foundations_steps(quality=False)
        self.assertEqual([name for name, _ in full], [
            'book-d6-foundations-format', 'book-d6-foundations-clippy',
            'book-d6-foundations-tests', 'book-d6-foundations-release-tests',
            'book-d6-foundations-doctests',
        ])
        self.assertEqual([name for name, _ in basic], [
            'book-d6-foundations-tests', 'book-d6-foundations-release-tests',
            'book-d6-foundations-doctests',
        ])
        for name, command in full:
            self.assertIn('projects/book-d6-foundations/Cargo.toml', command)
            if command[1] != 'fmt':
                self.assertIn('--locked', command)
                self.assertIn('--offline', command)
        self.assertEqual(release.TEST_LIMITS['book-d6-foundations-tests'], (20, 0))
        self.assertEqual(release.TEST_LIMITS['book-d6-foundations-release-tests'], (20, 0))
        self.assertEqual(release.TEST_LIMITS['book-d6-foundations-doctests'], (2, 0))

    def test_all_gate_entrypoints_include_book_d3(self):
        for relative in ('scripts/release_check.py', 'scripts/msrv_check.py'):
            text = (release.ROOT/relative).read_text()
            if relative.endswith('/release_check.py'):
                for name in ('book_d1_steps()', 'book_d2_steps()', 'book_d3_steps()', 'book_d4_steps()', 'book_d5_steps()', 'book_d6_steps()', 'book_d6_foundations_steps()'):
                    self.assertIn('*' + name, text)
            else:
                for name in ('book_d1_steps(quality=False)', 'book_d2_steps(quality=False)',
                             'book_d3_steps(quality=False)', 'book_d4_steps(quality=False)',
                             'book_d5_steps(quality=False)', 'book_d6_steps(quality=False)',
                             'book_d6_foundations_steps(quality=False)'):
                    self.assertIn(name, text)
        for relative in ('scripts/verify.sh', 'scripts/verify.ps1', 'scripts/verify_release_windows.ps1'):
            text = (release.ROOT/relative).read_text(encoding='utf-8-sig')
            self.assertIn('projects/advanced-b/reference/json-export/Cargo.toml', text)
            self.assertIn('projects/book-d1-cargo/Cargo.toml', text)
            self.assertIn('projects/book-d2-smart-pointers/Cargo.toml', text)
            self.assertIn('projects/book-d3-oop-patterns/Cargo.toml', text)
            self.assertIn('projects/book-d4-advanced-features/Cargo.toml', text)
            self.assertIn('projects/book-d5-webserver/Cargo.toml', text)
            self.assertIn('projects/book-d6-appendices/Cargo.toml', text)
            self.assertIn('projects/book-d6-foundations/Cargo.toml', text)
            self.assertIn('advanced_b_json_e2e', text)
            self.assertNotIn('cargo fetch', text)
        for relative in ('scripts/verify.ps1', 'scripts/verify_release_windows.ps1'):
            self.assertTrue((release.ROOT/relative).read_bytes().startswith(b'\xef\xbb\xbf'))


if __name__ == '__main__':
    unittest.main()
