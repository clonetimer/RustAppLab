import html
import re
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
class IndependentTeaching(unittest.TestCase):
    def test_both_guides_cover_read_design_test_delivery(self):
        for id in ('taskpocket','pagemint'):
            guide=(ROOT/f'apps/labquest/independent/{id}.md').read_text()
            for token in ('检查点 1','检查点 2','检查点 3','PROJECT_JOURNAL','CONTRACT.md','tests/self_check.rs','RELEASE_CHECKLIST'):
                self.assertIn(token,guide)
    def test_starter_self_tests_do_not_hide_todo(self):
        for id in ('taskpocket','pagemint'):
            src=(ROOT/f'apps/labquest/independent/{id}_self_check.rs').read_text()
            self.assertIn('todo!',src)
            self.assertIn('#[ignore',src)
            self.assertIn('missing_arguments_fail',src)
            self.assertIn('CARGO_BIN_EXE_'+id,src)
    def test_html_is_portable_and_has_matching_headings(self):
        for id in ('taskpocket','pagemint'):
            base=ROOT/'apps/labquest/independent'
            actual=(base/f'{id}.html').read_text()
            md=(base/f'{id}.md').read_text()
            without_code=re.sub(r'```.*?```','',md,flags=re.S)
            expected=re.findall(r'^#{1,2} (.+)$',without_code,re.M)
            heads=[html.unescape(re.sub('<[^>]+>','',value)) for value in re.findall(r'<h[12]>(.*?)</h[12]>',actual)]
            self.assertEqual(heads,expected)
            self.assertIn('name="viewport"',actual)
            self.assertNotIn('<script',actual)
            self.assertNotIn('src="http',actual)
    def test_packaging_includes_the_path(self):
        for f in ('scripts/package.py','scripts/structural_check.py','scripts/package_learner_kit.py'):
            self.assertIn('INDEPENDENT_PATH.html',(ROOT/f).read_text())
    def test_does_not_conflate_smoke_with_learning(self):
        for file in ('course/INDEPENDENT_PATH.md','apps/labquest/independent/TEST_PLAN.md'):
            text=(ROOT/file).read_text()
            self.assertIn('烟测',text)
            self.assertIn('ignore',text)
            self.assertIn('平台',text)
    def test_platform_entry_links_to_the_next_path(self):
        for file in ('START_HERE.html','WORKSHOP.html'):
            self.assertIn('href="INDEPENDENT_PATH.html"',(ROOT/file).read_text())
if __name__=='__main__': unittest.main()
