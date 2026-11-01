"""Static packaging/launcher checks; do not claim native PowerShell execution."""
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
class LearningSessionTests(unittest.TestCase):
    def test_ps51_encoding_and_literal_paths(self):
        path=ROOT/'scripts/learning_session.ps1'
        self.assertTrue(path.read_bytes().startswith(b'\xef\xbb\xbf'))
        text=path.read_text(encoding='utf-8-sig')
        self.assertIn('Resolve-Path -LiteralPath $Project',text)
        self.assertIn('Set-Location -LiteralPath $Root',text)
        self.assertNotIn('Invoke-Expression',text)
    def test_native_stderr_is_not_captured_under_stop(self):
        text=(ROOT/'scripts/learning_session.ps1').read_text(encoding='utf-8-sig')
        code='\n'.join(x for x in text.splitlines() if not x.strip().startswith('#'))
        self.assertNotIn('2>&1',code)
        self.assertIn("$ErrorActionPreference = 'Continue'",code)
        self.assertIn('$Code = $LASTEXITCODE',code)
        self.assertIn('finally { $ErrorActionPreference = $Saved }',code)
    def test_cmd_dispatches_to_shipped_script(self):
        text=(ROOT/'LEARN_NEXT_WINDOWS.cmd').read_text()
        self.assertIn('DisableDelayedExpansion',text)
        self.assertIn('"%~dp0scripts\\learning_session.ps1"',text)
        self.assertNotIn('set /p',text)
    def test_entry_does_not_overwrite_or_delete_learner_sources(self):
        text=(ROOT/'scripts/learning_session.ps1').read_text(encoding='utf-8-sig')
        for dangerous in ['Remove-Item','Set-Content','WriteAllText','studio start','beginner start']:
            self.assertNotIn(dangerous,text)
        self.assertIn("'C'",text)
        self.assertIn("'P'",text)
        self.assertIn("'overview'",text)
        self.assertIn("@('learn', 'check', $Path)",text)
    def test_sample_data_contract(self):
        base=ROOT/'examples/foundation'
        deck=[line.split('\t') for line in (base/'my-deck.tsv').read_text().splitlines()]
        answers=(base/'my-answers.txt').read_text().splitlines()
        self.assertEqual(len(deck),3)
        self.assertEqual(sum(a.strip().lower()==b.strip().lower() for (_,a),b in zip(deck,answers)),2)
        lines=(base/'my-notes.txt').read_text().splitlines()
        self.assertEqual([i for i,l in enumerate(lines,1) if 'Rust' in l],[1,2])
if __name__=='__main__': unittest.main()
