"""Static teaching contracts; native Windows execution is not implied."""
import unittest
from pathlib import Path
from html.parser import HTMLParser
ROOT=Path(__file__).resolve().parents[2]
class AdvancedTests(unittest.TestCase):
    def test_three_lessons_and_nine_hints(self):
        text=(ROOT/'course/ADVANCED_A.md').read_text()
        for i in range(1,4):self.assertIn(f'## A{i} ',text)
        self.assertEqual(text.count('### 提示 '),9)
        for word in ['Rc','RefCell','Weak','FnMut','FnOnce','Send','Sync','DESIGN_REVIEW']:self.assertIn(word,text)
    def test_page_offline_and_links(self):
        text=(ROOT/'ADVANCED_A.html').read_text();self.assertNotIn('<script',text);self.assertEqual(text.count('<details>'),9)
        for i in range(1,4):self.assertIn(f'id="a{i}"',text)
        self.assertIn('name="viewport"',text)
        class Parser(HTMLParser):pass
        Parser().feed(text)
    def test_windows_script_does_not_repeat_stderr_problem(self):
        data=(ROOT/'scripts/advanced_windows.ps1').read_bytes();self.assertTrue(data.startswith(b'\xef\xbb\xbf'))
        text=data.decode('utf-8-sig');self.assertNotIn('2>&1',text);self.assertIn("$ErrorActionPreference = 'Continue'",text)
        self.assertIn('$Code = $LASTEXITCODE',text);self.assertIn('Test-Path -LiteralPath',text);self.assertNotIn('Invoke-Expression',text)
    def test_starter_has_unimplemented_methods_but_no_advanced_core(self):
        text=(ROOT/'apps/labquest/advanced/workbench_starter.rs').read_text();self.assertEqual(text.count('todo!'),11)
        self.assertNotIn('session.undo.push',text);self.assertNotIn('thread::spawn(move || render_with',text)
    def test_package_carries_advanced_entry(self):
        for script in ['package.py','structural_check.py']:
            text=(ROOT/'scripts'/script).read_text();self.assertIn('ADVANCED_A.html',text);self.assertIn('START_ADVANCED_WINDOWS.cmd',text)
        self.assertIn('"Advanced"',(ROOT/'scripts/package_learner_kit.py').read_text())
    def test_knowledge_limits_are_explicit(self):
        text=(ROOT/'course/ADVANCED_A.md').read_text()
        for phrase in ['不声明macOS','第三方依赖','不是恶意代码沙箱','外部','最多32条','不是实时视图']:self.assertIn(phrase,text)
if __name__=='__main__':unittest.main()
