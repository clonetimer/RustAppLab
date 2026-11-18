"""Download selected CI artifact IDs without reselection or unsafe extraction."""
import contextlib
import copy
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import warnings
import zipfile


HELPER = Path(__file__).resolve().parents[1] / 'download_ci_artifacts.py'
RUN_ID = 44250858396
TOKEN = 'test-token-never-log'


def archive_bytes(entries):
    result = io.BytesIO()
    with warnings.catch_warnings():
        warnings.simplefilter('ignore', UserWarning)
        with zipfile.ZipFile(result, 'w', compression=zipfile.ZIP_STORED) as archive:
            for name, contents in entries:
                archive.writestr(name, contents)
    return result.getvalue()


def metadata(identifier, name, payload):
    return {
        'id': identifier,
        'node_id': f'node-{identifier}',
        'name': name,
        'size_in_bytes': len(payload),
        'url': f'https://api.github.com/repos/owner/repo/actions/artifacts/{identifier}',
        'archive_download_url': f'https://api.github.com/repos/owner/repo/actions/artifacts/{identifier}/zip',
        'expired': False,
        'created_at': '2026-11-18T06:43:41Z',
        'updated_at': '2026-11-18T06:43:41Z',
        'expires_at': '2026-12-18T06:43:41Z',
        'digest': 'sha256:' + hashlib.sha256(payload).hexdigest(),
        'workflow_run': {
            'id': RUN_ID,
            'repository_id': 100,
            'head_repository_id': 100,
            'head_branch': 'fix/ci-artifacts',
            'head_sha': 'a' * 40,
        },
    }


class DownloadTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(HELPER.is_file(), 'The exact-ID artifact downloader is not implemented.')
        spec = importlib.util.spec_from_file_location('download_ci_artifacts', HELPER)
        self.downloader = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.downloader)
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.serial = 0
        self.payload = archive_bytes([('candidate.json', b'{"status":"PASS"}\n'),
                                      ('logs/', b''), ('logs/step.log', b'verified\n')])
        self.selection = {
            'schema': 1,
            'repository': 'owner/repo',
            'run_id': RUN_ID,
            'artifacts': [metadata(14404059996, 'delivery-windows-latest', self.payload)],
        }

    def run_cli(self, selection=None, downloads=None, output=None, token=TOKEN):
        self.serial += 1
        raw = selection if isinstance(selection, bytes) else json.dumps(
            self.selection if selection is None else selection, indent=2).encode() + b'\n'
        source = self.root / f'selection-{self.serial}.json'
        source.write_bytes(raw)
        output = output or self.root / f'download-{self.serial}'
        downloads = {14404059996: self.payload} if downloads is None else downloads
        calls = []

        def gh(arguments, **kwargs):
            calls.append((arguments, kwargs))
            identifier = int(arguments[-1].split('/')[-2])
            result = downloads[identifier]
            if isinstance(result, Exception):
                raise result
            kwargs['stdout'].write(result)
            return subprocess.CompletedProcess(arguments, 0)

        stdout, stderr = io.StringIO(), io.StringIO()
        with patch.dict(os.environ, {'GH_TOKEN': token}), \
                patch.object(self.downloader.subprocess, 'run', side_effect=gh), \
                contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            result = self.downloader.main(['--selection', str(source), '--output', str(output)])
        return result, output, stdout.getvalue(), stderr.getvalue(), calls

    def assert_rejected(self, selection=None, downloads=None, *, before_download=False):
        result, output, stdout, stderr, calls = self.run_cli(selection, downloads)
        self.assertNotEqual(result, 0)
        self.assertFalse(output.exists())
        self.assertEqual(stdout, '')
        self.assertTrue(stderr)
        self.assertNotIn(TOKEN, stderr)
        if before_download:
            self.assertEqual(calls, [])
        return stderr, calls

    def test_downloads_exact_selected_ids_and_extracts_real_zip_bytes(self):
        native = archive_bytes([('native-evidence/engineering.json', b'{"status":"PASS"}')])
        self.selection['artifacts'].append(metadata(23, 'native-ubuntu-latest', native))
        result, output, _, stderr, calls = self.run_cli(
            downloads={14404059996: self.payload, 23: native})
        self.assertEqual(result, 0, stderr)
        self.assertEqual(stderr, '')
        self.assertEqual((output / 'delivery-windows-latest/candidate.json').read_bytes(),
                         b'{"status":"PASS"}\n')
        self.assertEqual((output / 'delivery-windows-latest/logs/step.log').read_bytes(), b'verified\n')
        self.assertEqual((output / 'native-ubuntu-latest/native-evidence/engineering.json').read_bytes(),
                         b'{"status":"PASS"}')
        self.assertEqual([call[0] for call in calls], [
            ['gh', 'api', '--hostname', 'github.com',
             'repos/owner/repo/actions/artifacts/14404059996/zip'],
            ['gh', 'api', '--hostname', 'github.com',
             'repos/owner/repo/actions/artifacts/23/zip'],
        ])
        for arguments, kwargs in calls:
            self.assertNotIn(TOKEN, str(arguments))
            self.assertNotIn('env', kwargs)  # GH_TOKEN stays inherited, outside argv.
            self.assertIs(kwargs.get('check'), True)
            self.assertEqual(kwargs.get('stderr'), subprocess.PIPE)
        self.assertEqual({path.name for path in output.iterdir()},
                         {'delivery-windows-latest', 'native-ubuntu-latest', 'ARTIFACT_SELECTION.json'})

    def test_preserves_exact_selection_bytes_with_complete_metadata(self):
        self.selection['artifacts'][0]['uninterpreted_metadata'] = {'evidence': '原始信息'}
        raw = json.dumps(self.selection, ensure_ascii=False, indent=4).encode('utf-8') + b'\n\n'
        result, output, _, stderr, _ = self.run_cli(raw)
        self.assertEqual(result, 0, stderr)
        self.assertEqual((output / 'ARTIFACT_SELECTION.json').read_bytes(), raw)

    def test_digest_is_checked_before_zip_parsing_or_extraction(self):
        stderr, calls = self.assert_rejected(downloads={14404059996: b'not a ZIP archive'})
        self.assertIn('digest', stderr.lower())
        self.assertEqual(len(calls), 1)

    def test_matching_digest_does_not_accept_corrupt_crc(self):
        damaged = self.payload.replace(b'verified\n', b'corrupt!\n')
        self.assertNotEqual(damaged, self.payload)
        self.selection['artifacts'][0]['digest'] = 'sha256:' + hashlib.sha256(damaged).hexdigest()
        self.assert_rejected(downloads={14404059996: damaged})

    def test_matching_digest_does_not_accept_non_zip_data(self):
        payload = b'not a ZIP archive'
        self.selection['artifacts'][0] = metadata(14404059996, 'delivery-windows-latest', payload)
        self.assert_rejected(downloads={14404059996: payload})

    def test_duplicate_archive_members_are_rejected(self):
        for entries in ([('same.txt', b'first'), ('same.txt', b'second')],
                        [('same/', b''), ('same', b'file')]):
            payload = archive_bytes(entries)
            self.selection['artifacts'][0] = metadata(14404059996, 'delivery-windows-latest', payload)
            with self.subTest(entries=entries):
                self.assert_rejected(downloads={14404059996: payload})

    def test_unsafe_archive_paths_cannot_escape_output(self):
        for name in ('../escaped.txt', '/escaped.txt', 'nested/../../escaped.txt',
                     r'nested\escaped.txt', r'\server\escaped.txt', 'C:/escaped.txt',
                     'C:escaped.txt', '//server/share/escaped.txt', './escaped.txt',
                     'nested/./escaped.txt', 'nested//escaped.txt'):
            payload = archive_bytes([(name, b'unsafe')])
            self.selection['artifacts'][0] = metadata(14404059996, 'delivery-windows-latest', payload)
            with self.subTest(name=name):
                self.assert_rejected(downloads={14404059996: payload})
                self.assertFalse((self.root / 'escaped.txt').exists())

    def test_symlinks_and_special_files_are_rejected(self):
        for mode in (stat.S_IFLNK | 0o777, stat.S_IFIFO | 0o644):
            member = zipfile.ZipInfo('link')
            member.create_system = 3
            member.external_attr = mode << 16
            payload = archive_bytes([(member, b'../escaped.txt')])
            self.selection['artifacts'][0] = metadata(14404059996, 'delivery-windows-latest', payload)
            with self.subTest(mode=mode):
                self.assert_rejected(downloads={14404059996: payload})

    def test_archive_file_directory_conflict_is_rejected(self):
        payload = archive_bytes([('file', b'contents'), ('file/nested.txt', b'nested')])
        self.selection['artifacts'][0] = metadata(14404059996, 'delivery-windows-latest', payload)
        self.assert_rejected(downloads={14404059996: payload})

    def test_invalid_wrapper_metadata_is_rejected_before_download(self):
        fields = {
            'schema': [None, True, 0, 2, '1'],
            'repository': [None, 1, '', '../repo', 'owner/repo/other', 'owner/repo?x=1',
                           'https://github.com/owner/repo', 'owner/..', 'owner/repo\n'],
            'run_id': [None, True, 0, -1, '226', 1.5],
            'artifacts': [None, [], {}, 'artifacts', [None], [[]]],
        }
        for field, values in fields.items():
            for value in values:
                selected = copy.deepcopy(self.selection)
                selected[field] = value
                with self.subTest(field=field, value=value):
                    self.assert_rejected(selected, before_download=True)
            selected = copy.deepcopy(self.selection)
            del selected[field]
            with self.subTest(missing=field):
                self.assert_rejected(selected, before_download=True)

    def test_invalid_artifact_metadata_is_rejected_before_download(self):
        fields = {
            'id': [None, True, 0, -1, '1', 1.5],
            'name': [None, 1, '', ' ', '.', '..', '../artifact', '/artifact',
                     r'parent\artifact', 'C:artifact', 'ARTIFACT_SELECTION.json', 'name\n'],
            'digest': [None, 1, '', 'sha256:' + 'a' * 63, 'sha256:' + 'g' * 64,
                       'md5:' + 'a' * 64, 'sha256:' + 'a' * 64 + '\n'],
            'expired': [None, 0, 1, True, 'false'],
            'created_at': [None, 1, '', 'not-a-date', '2026-11-18',
                           '2026-11-18T06:43:41', '2026-02-30T06:43:41Z'],
            'workflow_run': [None, [], {}, {'id': True}, {'id': str(RUN_ID)}, {'id': RUN_ID + 1}],
        }
        for field, values in fields.items():
            for value in values:
                selected = copy.deepcopy(self.selection)
                selected['artifacts'][0][field] = value
                with self.subTest(field=field, value=value):
                    self.assert_rejected(selected, before_download=True)
            selected = copy.deepcopy(self.selection)
            del selected['artifacts'][0][field]
            with self.subTest(missing=field):
                self.assert_rejected(selected, before_download=True)

    def test_duplicate_artifact_ids_or_names_are_rejected_before_download(self):
        for identifier, name in ((14404059996, 'other'), (23, 'delivery-windows-latest')):
            selected = copy.deepcopy(self.selection)
            selected['artifacts'].append(metadata(identifier, name, self.payload))
            with self.subTest(identifier=identifier, name=name):
                self.assert_rejected(selected, before_download=True)

    def test_invalid_json_is_rejected_before_download(self):
        for raw in (b'not-json', b'\xff', b'[]', b'null'):
            with self.subTest(raw=raw):
                self.assert_rejected(raw, before_download=True)

    def test_existing_output_directory_and_file_are_never_overwritten(self):
        for kind in ('directory', 'file', 'broken-symlink'):
            output = self.root / kind
            if kind == 'directory':
                output.mkdir()
                (output / 'keep.txt').write_bytes(b'keep')
            elif kind == 'file':
                output.write_bytes(b'keep')
            else:
                output.symlink_to(self.root / 'missing-target')
            with self.subTest(kind=kind):
                result, _, _, stderr, calls = self.run_cli(output=output)
                self.assertNotEqual(result, 0)
                self.assertTrue(stderr)
                self.assertEqual(calls, [])
                if kind == 'directory':
                    self.assertEqual((output / 'keep.txt').read_bytes(), b'keep')
                elif kind == 'file':
                    self.assertEqual(output.read_bytes(), b'keep')
                else:
                    self.assertTrue(output.is_symlink())

    def test_failed_download_stops_without_trying_another_artifact(self):
        self.selection['artifacts'].append(metadata(23, 'native-ubuntu-latest', self.payload))
        failure = subprocess.CalledProcessError(403, ['gh'], stderr=TOKEN.encode())
        stderr, calls = self.assert_rejected(downloads={14404059996: failure, 23: self.payload})
        self.assertEqual(len(calls), 1)
        self.assertIn('14404059996', stderr)

    def test_later_download_failure_removes_incomplete_output(self):
        self.selection['artifacts'].append(metadata(23, 'native-ubuntu-latest', self.payload))
        failure = subprocess.CalledProcessError(1, ['gh'], stderr=TOKEN.encode())
        _, calls = self.assert_rejected(downloads={14404059996: self.payload, 23: failure})
        self.assertEqual(len(calls), 2)

    def test_missing_token_fails_before_download(self):
        result, output, _, stderr, calls = self.run_cli(token='')
        self.assertNotEqual(result, 0)
        self.assertFalse(output.exists())
        self.assertIn('GH_TOKEN', stderr)
        self.assertEqual(calls, [])

    def test_missing_gh_returns_nonzero_without_disclosing_exception_data(self):
        self.assert_rejected(downloads={14404059996: FileNotFoundError(TOKEN)})


if __name__ == '__main__':
    unittest.main()
