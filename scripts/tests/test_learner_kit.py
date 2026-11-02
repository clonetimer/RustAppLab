"""Verify the actual B4 archive assets without building a distribution."""
import ast
import importlib.util
from pathlib import Path
import posixpath
import re
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/package_learner_kit.py'


class LearnerKitTests(unittest.TestCase):
    def assets(self):
        # An importable entrypoint prevents a tooling test from generating a kit.
        tree = ast.parse(SCRIPT.read_text(encoding='utf-8'))
        self.assertTrue(any(isinstance(node, ast.FunctionDef) and node.name == 'main'
                            for node in tree.body), 'Packaging needs a guarded main entrypoint')
        spec = importlib.util.spec_from_file_location('learner_kit', SCRIPT)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_b4_entry_links_resolve_inside_real_zip(self):
        assets = self.assets()
        with tempfile.TemporaryDirectory() as temp:
            archive = Path(temp)/'assets.zip'
            with zipfile.ZipFile(archive, 'w') as output:
                assets.add(output, 'ADVANCED_B.html', (ROOT/'ADVANCED_B.html').read_bytes())
                for name, data in assets.course_inputs():
                    assets.add(output, name, data)
            with zipfile.ZipFile(archive) as output:
                self.assertIsNone(output.testzip())
                names = set(output.namelist())
                for source in ('ADVANCED_B.html', 'course/ADVANCED_B.md', 'course/ADVANCED_B_WINDOWS.md', 'course/BOOK_GUIDE.md'):
                    self.assertIn(source, names)
                    text = output.read(source).decode('utf-8')
                    links = re.findall(r'href="([^"]+)"', text) if source.endswith('.html') else re.findall(r'(?<!!)\[[^\]]+\]\(([^)]+)\)', text)
                    for target in links:
                        if target.startswith(('https://', 'http://', '#')):
                            continue
                        resolved = posixpath.normpath(posixpath.join(posixpath.dirname(source), target.split('#', 1)[0]))
                        self.assertIn(resolved, names, f'{source} links to missing archive asset {target}')

    def test_intro_explains_separate_b4_dependency_preparation(self):
        intro = self.assets().INTRO
        self.assertNotIn('不需要下载外部 Rust crate', intro)
        for phrase in ('B1–B3', 'B4', 'prepare-json', '联网', '缓存'):
            self.assertIn(phrase, intro)


if __name__ == '__main__':
    unittest.main()
