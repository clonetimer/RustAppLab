"""Rehearsal contract checks. These are not Windows-native or learner-effectiveness tests."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("systems_path", ROOT / "scripts/smoke_systems_path.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class SystemsPathTests(unittest.TestCase):
    def test_snapshot_validation_rejects_false_success(self):
        titles = ['中文', 'quote "<>&']
        before = dict(count=2, tickets=[dict(id=i+1, title=t, done=False) for i,t in enumerate(titles)])
        after = json.loads(json.dumps(before))
        after['tickets'][0]['done'] = True
        module.inspect_snapshots(json.dumps(before).encode(), json.dumps(after).encode(), titles)
        with self.assertRaises(AssertionError):
            module.inspect_snapshots(json.dumps(before).encode(), json.dumps(before).encode(), titles)
        after['tickets'][1]['title'] = 'invented'
        with self.assertRaises(AssertionError):
            module.inspect_snapshots(json.dumps(before).encode(), json.dumps(after).encode(), titles)

    def test_stats_are_verified_independently_in_bytes_and_order(self):
        with tempfile.TemporaryDirectory() as tmp:
            files = [Path(tmp)/'中文.json', Path(tmp)/'second.json']
            for file, data in zip(files, ['{"标题":"中"}', '{}']): file.write_text(data, encoding='utf-8')
            rows = ''.join(f'{p}\t{len(p.read_bytes())}\t1\n' for p in files)
            report = rows + f'files=2 bytes={sum(len(p.read_bytes()) for p in files)} lines=2\n'
            module.inspect_statistics(report, files)
            with self.assertRaises(AssertionError): module.inspect_statistics(report, list(reversed(files)))
            with self.assertRaises(AssertionError): module.inspect_statistics(report.replace('lines=2','lines=1'), files)

    def test_missing_program_does_not_create_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            out=Path(tmp)/'must-not-exist'
            with self.assertRaises(FileNotFoundError): module.run(Path(tmp), out)
            self.assertFalse(out.exists())

    def test_course_and_template_distinguish_references_from_learners(self):
        text=(ROOT/'course/SYSTEMS_PATH.md').read_text(encoding='utf-8')
        for term in ['窗口 A','窗口 B','不能算作独立完成','create_new','Content-Length','串行实现也可能通过']:
            self.assertIn(term,text)
        template=(ROOT/'examples/systems/LEARNING_LOG_TEMPLATE.md').read_text(encoding='utf-8')
        self.assertIn('未做',template)
        self.assertNotIn('验收通过',template)

if __name__ == '__main__': unittest.main()
