"""C2 distribution regressions: portable text bytes, hidden Cargo files, source binding."""
import ast
import hashlib
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class C2DistributionTests(unittest.TestCase):
    def test_source_manifest_is_utf8_lf_without_platform_newline_translation(self):
        script = ROOT / 'scripts/package.py'
        tree = ast.parse(script.read_text(encoding='utf-8'))
        self.assertTrue(any(isinstance(node, ast.FunctionDef) and node.name == 'main'
                            for node in tree.body), 'Importing packaging must not rebuild artifacts')
        module = load('package')
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / '中文.rs'
            source.write_bytes(b'line one\nline two\n')
            manifest = root / 'manifest.sha256'
            # Windows translates LF through text writers; checksum bytes must bypass that.
            with patch.object(Path, 'write_text', side_effect=AssertionError('platform text writer used')):
                module.write_manifest([source], root, manifest)
            expected = f'{hashlib.sha256(source.read_bytes()).hexdigest()}  中文.rs\n'.encode('utf-8')
            self.assertEqual(manifest.read_bytes(), expected)
            self.assertNotIn(b'\r', manifest.read_bytes())

    def test_portable_archive_preserves_hidden_cargo_configuration_and_checksums(self):
        module = load('package_c2')
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'stage'
            (root / '.cargo').mkdir(parents=True)
            (root / '.cargo/config.toml').write_bytes(b'[source.crates-io]\n')
            (root / 'vendor/example').mkdir(parents=True)
            (root / 'vendor/example/.cargo-checksum.json').write_bytes(b'{}')
            (root / 'vendor/example/LICENSE').write_bytes(b'license')
            archive = Path(temp) / 'portable.zip'
            digest = module.pack(root, archive)
            with zipfile.ZipFile(archive) as z:
                self.assertEqual(z.read('.cargo/config.toml'), b'[source.crates-io]\n')
                self.assertEqual(z.read('vendor/example/.cargo-checksum.json'), b'{}')
                self.assertEqual(z.read('vendor/example/LICENSE'), b'license')
            self.assertEqual(digest, hashlib.sha256(archive.read_bytes()).hexdigest())

    def test_c2_source_changes_invalidate_fingerprint_but_build_output_does_not(self):
        from release_check import source_id
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / 'projects/advanced-c2/learner/src/lib.rs'
            source.parent.mkdir(parents=True)
            source.write_bytes(b'// before\n')
            before = source_id(root)
            source.write_bytes(b'// after\n')
            after = source_id(root)
            self.assertNotEqual(before, after)
            generated = root / 'projects/advanced-c2/reference/target/generated.rs'
            generated.parent.mkdir(parents=True)
            generated.write_bytes(b'// build output\n')
            self.assertEqual(after, source_id(root))


    def test_offline_learner_has_the_platform_marker_but_reference_does_not(self):
        module = load('package_c2')
        with tempfile.TemporaryDirectory() as temp:
            for kind in ('reference', 'learner'):
                root = Path(temp) / kind
                root.mkdir()
                module.write_kit_configuration(root, kind)
                self.assertEqual((root / '.cargo/config.toml').read_bytes(), module.CONFIG.encode('utf-8'))
                marker = root / '.applab-c2'
                if kind == 'learner':
                    self.assertEqual(marker.read_bytes(), b'RustAppLab C2 v1\n')
                else:
                    self.assertFalse(marker.exists())

    def test_portable_zip_host_metadata_does_not_change_package_bytes(self):
        module = load('package_c2')
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'stage'
            (root / '.cargo').mkdir(parents=True)
            (root / '.cargo/config.toml').write_bytes(b'config\n')
            (root / 'vendor/example').mkdir(parents=True)
            (root / 'vendor/example/.cargo-checksum.json').write_bytes(b'{}')
            (root / 'Z.txt').write_bytes(b'upper')
            (root / 'a.txt').write_bytes(b'lower')
            linux, windows = Path(temp) / 'linux.zip', Path(temp) / 'windows.zip'
            with patch.object(sys, 'platform', 'linux'):
                module.pack(root, linux)
            with patch.object(sys, 'platform', 'win32'):
                module.pack(root, windows)
            self.assertEqual(linux.read_bytes(), windows.read_bytes())
            with zipfile.ZipFile(windows) as archive:
                self.assertEqual(archive.namelist(), sorted(archive.namelist()))
                self.assertTrue(all(item.create_system == 3 for item in archive.infolist()))


if __name__ == '__main__':
    unittest.main()
