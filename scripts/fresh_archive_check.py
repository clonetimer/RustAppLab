#!/usr/bin/env python3
"""Extract the exact source ZIP to a fresh directory and run real offline checks.

This report is source-bound and intended for the release_decision.py archive gate.
It refuses path traversal, validates SOURCE_MANIFEST through structural_check,
and records the hash of the exact archive tested.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import subprocess
import sys
import tempfile
import time
import zipfile


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-zip', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True, help='New JSON report path')
    args = parser.parse_args()
    archive = args.source_zip.resolve()
    output = args.output.resolve()
    if not archive.is_file():
        parser.error(f'source ZIP not found: {archive}')
    if output.exists():
        parser.error(f'report already exists; choose a new path: {output}')
    output.parent.mkdir(parents=True, exist_ok=True)
    report = {
        'schema': 1, 'status': 'RUNNING', 'version': None, 'source_id': None,
        'source_id_after': None, 'system': platform.system(), 'platform': platform.platform(),
        'source_zip': archive.name, 'source_zip_sha256': sha256(archive),
        'fresh_debug_tests': 0, 'fresh_release_tests': 0, 'doctests': 0, 'steps': [],
    }
    with tempfile.TemporaryDirectory(prefix='rustapplab-fresh-archive-') as tmp:
        temp = Path(tmp)
        try:
            with zipfile.ZipFile(archive) as zf:
                bad = zf.testzip()
                if bad:
                    raise RuntimeError(f'ZIP CRC failure: {bad}')
                names = zf.namelist()
                for name in names:
                    part = PurePosixPath(name)
                    if part.is_absolute() or '..' in part.parts or not part.parts or part.parts[0] != 'RustAppLab':
                        raise RuntimeError(f'unsafe or unexpected archive path: {name}')
                zf.extractall(temp)
            root = temp / 'RustAppLab'
            if not (root / 'Cargo.toml').is_file():
                raise RuntimeError('archive missing RustAppLab/Cargo.toml')
            sys.path.insert(0, str(root / 'scripts'))
            from release_check import source_id, test_summary
            report['version'] = (root / 'VERSION').read_text(encoding='utf-8').strip()
            report['source_id'] = source_id(root)
            plan = [
                ('structure', [sys.executable, 'scripts/structural_check.py'], 'structure'),
                ('debug-tests', ['cargo', 'test', '--locked', '--offline', '--workspace', '--all-targets', '--', '--test-threads=2'], 'debug'),
                ('release-tests', ['cargo', 'test', '--locked', '--offline', '--release', '--workspace', '--all-targets', '--', '--test-threads=2'], 'release'),
                ('doctests', ['cargo', 'test', '--locked', '--offline', '--workspace', '--doc'], 'docs'),
            ]
            passed_all = True
            for label, command, kind in plan:
                log = output.parent / f'{output.stem}-{label}.txt'
                start = time.monotonic()
                env = os.environ.copy()
                env.pop('CARGO_BUILD_TARGET', None)
                env['CARGO_TARGET_DIR'] = str(root / 'target')
                env['CARGO_TERM_COLOR'] = 'never'
                try:
                    result = subprocess.run(command, cwd=root, env=env, stdout=subprocess.PIPE,
                                            stderr=subprocess.STDOUT, timeout=600, check=False)
                    code = result.returncode
                    data = result.stdout
                except subprocess.TimeoutExpired as exc:
                    code = 124
                    data = (exc.stdout or b'') + b'\nTIMEOUT after 600 seconds\n'
                log.write_bytes(data)
                text = data.decode('utf-8', errors='replace')
                item = {'step': label, 'command': command, 'exit_code': code,
                        'seconds': round(time.monotonic() - start, 3), 'log': log.name,
                        'log_sha256': sha256(log), 'passed': code == 0}
                if kind in ('debug', 'release'):
                    summary = test_summary(text, code)
                    item['tests'] = summary
                    item['passed'] = summary['valid']
                    report['fresh_debug_tests' if kind == 'debug' else 'fresh_release_tests'] = summary['passed'] if summary['valid'] else 0
                elif kind == 'docs':
                    summary = test_summary(text, code, minimum=2, expected_ignored=0)
                    item['tests'] = summary
                    item['passed'] = summary['valid']
                    report['doctests'] = summary['passed'] if summary['valid'] else 0
                report['steps'].append(item)
                if not item['passed']:
                    passed_all = False
                    break
            report['source_id_after'] = source_id(root)
            report['status'] = 'PASS' if passed_all and len(report['steps']) == len(plan) and report['source_id_after'] == report['source_id'] and report['fresh_debug_tests'] >= 200 and report['fresh_release_tests'] >= 200 and report['doctests'] >= 2 else 'FAIL'
        except Exception as exc:
            report['status'] = 'FAIL'
            report['error'] = f'{type(exc).__name__}: {exc}'
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if report['status'] == 'PASS' else 1

if __name__ == '__main__':
    raise SystemExit(main())
