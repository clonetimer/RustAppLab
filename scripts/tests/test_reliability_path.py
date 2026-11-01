"""Checks for teaching material and its independent validators; no platform claims."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('reliability_path',ROOT/'scripts/smoke_reliability_path.py')
m=importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

class ReliabilityTests(unittest.TestCase):
    def test_partial_commit_and_orphan_audit_are_rejected(self):
        before={'items':[['PEN','笔',10]],'movements':[]}
        m.assert_rollback(before,json.loads(json.dumps(before)))
        for after in [{'items':[['PEN','笔',8]],'movements':[]},
                      {'items':before['items'],'movements':[[1,'PEN',-2]]}]:
            with self.assertRaises(AssertionError): m.assert_rollback(before,after)
    def test_equal_delay_order_is_not_a_contract(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'equal.tsv'
            expected={'a':(40,'甲'),'b':(40,'乙')}
            for rows in ['a\t40\t甲\nb\t40\t乙\n','b\t40\t乙\na\t40\t甲\n']:
                p.write_text('id\twait_ms\tnote\n'+rows,encoding='utf-8')
                self.assertEqual(set(m.inspect_report(p,expected)),{'a','b'})
    def test_invented_or_duplicate_completion_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'bad.tsv'
            for rows in ['a\t40\t甲\na\t40\t甲\n','a\t41\t甲\nb\t40\t乙\n','a\t40\t甲\n']:
                p.write_text('id\twait_ms\tnote\n'+rows,encoding='utf-8')
                with self.assertRaises(AssertionError): m.inspect_report(p,{'a':(40,'甲'),'b':(40,'乙')})
    def test_missing_binary_and_missing_db_do_not_create_files(self):
        with tempfile.TemporaryDirectory() as d:
            out=Path(d)/'new'
            with self.assertRaises(FileNotFoundError): m.run(Path(d),out)
            self.assertFalse(out.exists())
            db=Path(d)/'absent.db'
            with self.assertRaises(Exception): m.snapshot(db)
            self.assertFalse(db.exists())
    def test_samples_and_curriculum_are_truthful(self):
        text=(ROOT/'course/RELIABILITY_PATH.md').read_text(encoding='utf-8')
        for term in ['协作式超时','配置','不能算作独立完成','不声明 macOS','平台根目录','完整路径','不是断电']:
            self.assertIn(term,text)
        base=ROOT/'examples/reliability'
        self.assertEqual((base/'rollback.tsv').read_text().splitlines(),['PEN\t-2','BOOK\t-99'])
        self.assertIn('未做',(base/'LEARNING_LOG_TEMPLATE.md').read_text())
    def test_html_is_shipped_offline_not_a_server(self):
        text=(ROOT/'RELIABILITY_PATH.html').read_text()
        self.assertIn('name="viewport"',text)
        self.assertNotIn('<script',text)
        self.assertNotIn('src="http',text)
        self.assertIn('wake_lab',text)
if __name__=='__main__': unittest.main()
