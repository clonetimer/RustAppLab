"""RC closeout regressions: current status, handoff and release identity."""
from pathlib import Path
import importlib.util
import sys
import tempfile
import unittest
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts'))

class CloseoutTests(unittest.TestCase):
    def test_current_version_is_one_candidate_not_three_development_versions(self):
        version=(ROOT/'VERSION').read_text().strip()
        self.assertEqual(version, '0.19.0-rc.1')
        for name in ('apps/labquest/Cargo.toml', 'Cargo.lock'):
            self.assertIn('version = "'+version+'"', (ROOT/name).read_text())
        for name in ('README.md','README_START_HERE.md','KNOWN_ISSUES.md','RELEASE_POLICY.md','ROADMAP.md'):
            self.assertIn(version, (ROOT/name).read_text(encoding='utf-8').splitlines()[0], name)

    def test_active_entrypoints_do_not_claim_delivered_courses_are_missing(self):
        paths=('README.md','README_START_HERE.md','RELEASE_POLICY.md','KNOWN_ISSUES.md','SUPPORTED_PLATFORMS.md',
               'ROADMAP.md','course/CURRICULUM.md','course/BOOK_GUIDE.md','course/NETWORK_PREPARATION.md',
               'apps/labquest/src/advanced_c1.rs','apps/labquest/src/ecosystem.rs','WORKSHOP.html')
        stale=('C2 Axum 尚未交付','Tokio/Reqwest/Axum、桌面 GUI 与 WASM 课程仍未完成',
               'Serde/serde_json 课程待实现','Tokio/Reqwest/Axum 异步框架版仍未完成',
               'Tokio/Axum 真实异步网络课程 C 仍待交付','C（真实异步网络）尚待完成',
               'Tokio非阻塞网络、桌面界面、WebAssembly仍待扩展',
               'C 成熟异步网络仍待完成', 'Axum 异步未实现', '异步目标尚未完成')
        for name in paths:
            text=(ROOT/name).read_text(encoding='utf-8')
            for item in stale:
                with self.subTest(file=name, phrase=item): self.assertNotIn(item,text)

    def test_handoff_distinguishes_experience_learning_and_human_acceptance(self):
        text=(ROOT/'README_START_HERE.md').read_text(encoding='utf-8')
        for item in ('runtime','learner-kit','source.zip','1.85.0','MSVC','TODO','--frozen','已有工程'):
            self.assertIn(item,text)
        self.assertTrue((ROOT/'course/USER_TRIAL.md').is_file())
        policy=(ROOT/'RELEASE_POLICY.md').read_text(encoding='utf-8')
        self.assertIn('真人',policy)
        self.assertIn('Stable',policy)
        self.assertIn('候选',policy)

    def test_learner_package_title_uses_current_version(self):
        import package_learner_kit
        self.assertIn((ROOT/'VERSION').read_text().strip(),package_learner_kit.INTRO.splitlines()[0])
        self.assertIn('C1',package_learner_kit.INTRO)
        self.assertIn('C2',package_learner_kit.INTRO)

    def test_reference_runtime_packaging_and_candidate_gate_exist(self):
        for name in ('scripts/candidate_check.py','scripts/assemble_candidate.py','.github/workflows/delivery.yml'):
            self.assertTrue((ROOT/name).is_file(),name)
        ci=(ROOT/'.github/workflows/ci.yml').read_text(encoding='utf-8')
        self.assertIn('needs: [native, browser, msrv, c1, c2, delivery]',ci)
        self.assertIn('--prerelease',ci)
        self.assertIn('--latest=false',ci)
        self.assertIn("github.ref == 'refs/heads/main'",ci)
        self.assertNotIn('--clobber',ci)

    def test_candidate_requires_source_bound_reports_and_complete_assets(self):
        path=ROOT/'scripts/assemble_candidate.py'
        self.assertTrue(path.is_file(),'candidate verifier missing')
        spec=importlib.util.spec_from_file_location('candidate_assembly_test',path)
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises((ValueError,FileNotFoundError)):
                module.collect_assets(Path(tmp),Path(tmp)/'out')

class DeliveryIntegrityTests(unittest.TestCase):
    def test_missing_native_binaries_are_rejected(self):
        from candidate_check import runtime_files
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(ValueError): runtime_files(Path(temp), '.exe')

    def test_evidence_paths_cannot_escape_artifact(self):
        from assemble_candidate import safe_child
        with tempfile.TemporaryDirectory() as temp:
            for name in ('../outside', '/etc/hosts', 'nested/../../escape', r'..\\outside'):
                with self.subTest(name=name), self.assertRaises(ValueError):
                    safe_child(Path(temp),name)

    def test_modified_logs_and_wrong_source_cannot_pass(self):
        import hashlib,json
        from assemble_candidate import read_report
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); log=root/'command.txt';log.write_bytes(b'actual run')
            report={'source_id':'current','source_id_after':'current','status':'PASS',
                    'steps':[{'step':'demo','passed':True,'log':'command.txt','sha256':hashlib.sha256(log.read_bytes()).hexdigest()}]}
            path=root/'result.json';path.write_text(json.dumps(report))
            self.assertEqual(read_report(path,'current')['status'],'PASS')
            with self.assertRaises(ValueError):read_report(path,'wrong')
            log.write_bytes(b'changed output')
            with self.assertRaises(ValueError):read_report(path,'current')

if __name__=='__main__': unittest.main()
