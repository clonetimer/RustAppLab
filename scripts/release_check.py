#!/usr/bin/env python3
"""Reproducible local release checks, with source-bound logs. Python >=3.10.

This is engineering evidence on the host that actually runs it, NOT a Windows,
MSRV, browser or teaching-effectiveness certificate. No networking is needed.
"""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
IGNORED_CHALLENGES = 15
TEST_RESULT = re.compile(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", re.MULTILINE)
CODE_SUFFIXES = {'.rs', '.toml', '.lock', '.py', '.ps1', '.cmd', '.sh', '.html', '.css', '.js', '.cjs', '.yml', '.yaml',
                 '.md', '.tsv', '.txt', '.json', '.kv', '.svg'}
BOOK_D1_MANIFEST = 'projects/book-d1-cargo/Cargo.toml'
BOOK_D2_MANIFEST = 'projects/book-d2-smart-pointers/Cargo.toml'
BOOK_D3_MANIFEST = 'projects/book-d3-oop-patterns/Cargo.toml'
BOOK_D4_MANIFEST = 'projects/book-d4-advanced-features/Cargo.toml'
BOOK_D5_MANIFEST = 'projects/book-d5-webserver/Cargo.toml'
BOOK_D6_MANIFEST = 'projects/book-d6-appendices/Cargo.toml'
BOOK_D6_FOUNDATIONS_MANIFEST = 'projects/book-d6-foundations/Cargo.toml'
ADVANCED_B4_MANIFEST = 'projects/advanced-b/reference/json-export/Cargo.toml'
ADVANCED_B_MANIFEST = 'projects/advanced-b/reference/Cargo.toml'
TEST_LIMITS = {
    'debug-tests': (200, 15), 'release-tests': (200, 15), 'doctests': (2, 0),
    'advanced-b-debug-default': (12, 0), 'advanced-b-debug-strict': (12, 0),
    'advanced-b-release-default': (12, 0), 'advanced-b-release-strict': (12, 0),
    'advanced-b-no-default': (12, 0),
    'advanced-b4-debug-default': (20, 0), 'advanced-b4-debug-strict': (20, 0),
    'advanced-b4-release-default': (20, 0), 'advanced-b4-release-strict': (20, 0),
    'advanced-b4-doctests-default': (1, 0), 'advanced-b4-doctests-strict': (1, 0),
    'advanced-b4-platform-e2e': (5, 0),
    'advanced-b-doctests-default': (1, 0), 'advanced-b-doctests-strict': (1, 0),
    'book-d1-tests': (10, 0), 'book-d1-doctests': (1, 0),
    'book-d2-tests': (26, 0), 'book-d2-doctests': (1, 0),
    'book-d3-tests': (44, 0), 'book-d3-doctests': (1, 0),
    'book-d4-tests': (39, 0), 'book-d4-doctests': (2, 0),
    'book-d5-tests': (20, 0), 'book-d5-release-tests': (20, 0), 'book-d5-doctests': (1, 0),
    'book-d6-tests': (11, 0), 'book-d6-release-tests': (11, 0), 'book-d6-doctests': (3, 0),
    'book-d6-foundations-tests': (48, 0), 'book-d6-foundations-release-tests': (48, 0),
    'book-d6-foundations-doctests': (76, 0),
}


def advanced_b_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """Explicitly cover the separate reference workspace; never run learner TODOs."""
    common = ['--locked', '--offline', '--manifest-path', ADVANCED_B_MANIFEST, '--workspace']
    plan = []
    if quality:
        plan.append(('advanced-b-format', ['cargo', 'fmt', '--manifest-path', ADVANCED_B_MANIFEST,
                                           '--all', '--', '--check']))
        for name, features in [('default', []), ('strict', ['--all-features'])]:
            plan.append(('advanced-b-clippy-' + name,
                         ['cargo', 'clippy'] + common + ['--all-targets'] + features + ['--', '-D', 'warnings']))
    for profile, flags in [('debug', []), ('release', ['--release'])]:
        for name, features in [('default', []), ('strict', ['--all-features'])]:
            plan.append((f'advanced-b-{profile}-{name}',
                         ['cargo', 'test'] + common + flags + ['--all-targets'] + features + ['--', '--test-threads=2']))
    plan.append(('advanced-b-no-default', ['cargo', 'test'] + common +
                 ['--all-targets', '--no-default-features', '--', '--test-threads=2']))
    for name, features in [('default', []), ('strict', ['--all-features'])]:
        plan.append(('advanced-b-doctests-' + name, ['cargo', 'test'] + common + features + ['--doc']))
    return plan


def advanced_b4_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """B4 is a separately prepared workspace; every command here stays offline."""
    common = ['--locked', '--offline', '--manifest-path', ADVANCED_B4_MANIFEST, '--workspace']
    configs = [('default', []), ('strict', ['--all-features'])]
    plan = []
    if quality:
        plan.append(('advanced-b4-format', ['cargo', 'fmt', '--manifest-path', ADVANCED_B4_MANIFEST,
                                           '--all', '--', '--check']))
        for name, features in configs:
            plan.append(('advanced-b4-clippy-' + name,
                         ['cargo', 'clippy'] + common + ['--all-targets'] + features + ['--', '-D', 'warnings']))
    for profile, flags in [('debug', []), ('release', ['--release'])]:
        for name, features in configs:
            plan.append((f'advanced-b4-{profile}-{name}',
                         ['cargo', 'test'] + common + flags + ['--all-targets'] + features + ['--', '--test-threads=2']))
    for name, features in configs:
        plan.append(('advanced-b4-doctests-' + name, ['cargo', 'test'] + common + features + ['--doc']))
    plan.append(('advanced-b4-platform-e2e', ['cargo', 'test', '--locked', '--offline', '-p', 'labquest',
                 '--features', 'b4-integration', '--test', 'advanced_b_json_e2e', '--', '--test-threads=1']))
    return plan


def book_d1_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """Book D1 stays independent of the frozen 17-package root workspace."""
    common = ['--locked', '--offline', '--manifest-path', BOOK_D1_MANIFEST, '--workspace']
    steps = []
    if quality:
        steps.extend([
            ('book-d1-format', ['cargo', 'fmt', '--manifest-path', BOOK_D1_MANIFEST,
                                '--all', '--', '--check']),
            ('book-d1-clippy', ['cargo', 'clippy'] + common +
             ['--all-targets', '--', '-D', 'warnings']),
        ])
    steps.extend([
        ('book-d1-tests', ['cargo', 'test'] + common + ['--all-targets']),
        ('book-d1-doctests', ['cargo', 'test'] + common + ['--doc']),
        ('book-d1-install-smoke', [sys.executable, 'scripts/book_d1_check.py']),
    ])
    if quality:
        steps.append(('book-d1-package-list', ['cargo', 'package', '--list', '--allow-dirty',
                      '--manifest-path', 'projects/book-d1-cargo/book-card/Cargo.toml']))
    return steps


def book_d2_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """Book D2 is a separate, dependency-free smart-pointer workspace."""
    common = ['--locked', '--offline', '--manifest-path', BOOK_D2_MANIFEST, '--workspace']
    steps = []
    if quality:
        steps.extend([
            ('book-d2-format', ['cargo', 'fmt', '--manifest-path', BOOK_D2_MANIFEST,
                                '--all', '--', '--check']),
            ('book-d2-clippy', ['cargo', 'clippy'] + common +
             ['--all-targets', '--', '-D', 'warnings']),
        ])
    steps.extend([
        ('book-d2-tests', ['cargo', 'test'] + common + ['--all-targets']),
        ('book-d2-doctests', ['cargo', 'test'] + common + ['--doc']),
    ])
    return steps


def book_d3_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """Book D3 is an isolated object-model and pattern-matching workspace."""
    common = ['--locked', '--offline', '--manifest-path', BOOK_D3_MANIFEST, '--workspace']
    steps = []
    if quality:
        steps.extend([
            ('book-d3-format', ['cargo', 'fmt', '--manifest-path', BOOK_D3_MANIFEST,
                                '--all', '--', '--check']),
            ('book-d3-clippy', ['cargo', 'clippy'] + common +
             ['--all-targets', '--', '-D', 'warnings']),
        ])
    steps.extend([
        ('book-d3-tests', ['cargo', 'test'] + common + ['--all-targets']),
        ('book-d3-doctests', ['cargo', 'test'] + common + ['--doc']),
    ])
    return steps


def book_d4_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """Book D4 is an isolated advanced-features workspace with a proc-macro crate."""
    common = ['--locked', '--offline', '--manifest-path', BOOK_D4_MANIFEST, '--workspace']
    steps = []
    if quality:
        steps.extend([
            ('book-d4-format', ['cargo', 'fmt', '--manifest-path', BOOK_D4_MANIFEST,
                                '--all', '--', '--check']),
            ('book-d4-clippy', ['cargo', 'clippy'] + common +
             ['--all-targets', '--', '-D', 'warnings']),
        ])
    steps.extend([
        ('book-d4-tests', ['cargo', 'test'] + common + ['--all-targets']),
        ('book-d4-doctests', ['cargo', 'test'] + common + ['--doc']),
    ])
    return steps


def book_d5_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """Book D5 is an isolated, dependency-free TCP/HTTP and worker-pool workspace."""
    common = ['--locked', '--offline', '--manifest-path', BOOK_D5_MANIFEST, '--workspace']
    steps = []
    if quality:
        steps.extend([
            ('book-d5-format', ['cargo', 'fmt', '--manifest-path', BOOK_D5_MANIFEST,
                                '--all', '--', '--check']),
            ('book-d5-clippy', ['cargo', 'clippy'] + common +
             ['--all-targets', '--', '-D', 'warnings']),
        ])
    steps.extend([
        ('book-d5-tests', ['cargo', 'test'] + common +
         ['--all-targets', '--', '--test-threads=2']),
        ('book-d5-release-tests', ['cargo', 'test'] + common +
         ['--all-targets', '--release', '--', '--test-threads=2']),
        ('book-d5-doctests', ['cargo', 'test'] + common + ['--doc']),
    ])
    return steps



def book_d6_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """Book D6 is an isolated, dependency-free appendix reference lab."""
    common = ['--locked', '--offline', '--manifest-path', BOOK_D6_MANIFEST, '--workspace']
    steps = []
    if quality:
        steps.extend([
            ('book-d6-format', ['cargo', 'fmt', '--manifest-path', BOOK_D6_MANIFEST,
                                '--all', '--', '--check']),
            ('book-d6-clippy', ['cargo', 'clippy'] + common +
             ['--all-targets', '--', '-D', 'warnings']),
        ])
    steps.extend([
        ('book-d6-tests', ['cargo', 'test'] + common + ['--all-targets']),
        ('book-d6-release-tests', ['cargo', 'test'] + common +
         ['--all-targets', '--release']),
        ('book-d6-doctests', ['cargo', 'test'] + common + ['--doc']),
    ])
    return steps



def book_d6_foundations_steps(quality: bool = True) -> list[tuple[str, list[str]]]:
    """Book D6-B is an isolated, dependency-free foundations-gap workspace."""
    common = ['--locked', '--offline', '--manifest-path', BOOK_D6_FOUNDATIONS_MANIFEST, '--workspace']
    steps = []
    if quality:
        steps.extend([
            ('book-d6-foundations-format', ['cargo', 'fmt', '--manifest-path', BOOK_D6_FOUNDATIONS_MANIFEST,
                                            '--all', '--', '--check']),
            ('book-d6-foundations-clippy', ['cargo', 'clippy'] + common +
             ['--all-targets', '--', '-D', 'warnings']),
        ])
    steps.extend([
        ('book-d6-foundations-tests', ['cargo', 'test'] + common + ['--all-targets']),
        ('book-d6-foundations-release-tests', ['cargo', 'test'] + common +
         ['--all-targets', '--release']),
        ('book-d6-foundations-doctests', ['cargo', 'test'] + common + ['--doc']),
    ])
    return steps


def source_id(root: Path = ROOT) -> str:
    """Code, build and lesson inputs; excludes generated evidence and build caches."""
    h = hashlib.sha256()
    paths = []
    for path in root.rglob('*'):
        rel = path.relative_to(root)
        if not path.is_file() or any(x in {'target', '.git', '__pycache__'} for x in rel.parts):
            continue
        if (rel.parts[0] not in {'apps', 'scripts', '.github', 'course', 'examples'}
                and rel.parts[:2] not in {('projects', 'advanced-b'), ('projects', 'advanced-c1'), ('projects', 'advanced-c2'), ('projects', 'book-d1-cargo'), ('projects', 'book-d2-smart-pointers'), ('projects', 'book-d3-oop-patterns'), ('projects', 'book-d4-advanced-features'), ('projects', 'book-d5-webserver'), ('projects', 'book-d6-appendices'), ('projects', 'book-d6-foundations'), ('projects', 'modern-d8-edition')} and len(rel.parts) != 1):
            continue
        if path.suffix in CODE_SUFFIXES or rel.as_posix() == 'VERSION':
            paths.append(path)
    for p in sorted(paths, key=lambda p: p.relative_to(root).as_posix()):
        h.update(p.relative_to(root).as_posix().encode('utf-8') + b'\0')
        h.update(hashlib.sha256(p.read_bytes()).digest())
    return h.hexdigest()


def test_summary(text: str, exit_code: int, minimum: int = 200, expected_ignored: int = 15) -> dict:
    results = TEST_RESULT.findall(text)
    data = dict(suites=len(results), passed=sum(int(x[1]) for x in results),
                failed=sum(int(x[2]) for x in results), ignored=sum(int(x[3]) for x in results))
    data['valid'] = bool(exit_code == 0 and results and data['passed'] >= minimum
                         and data['failed'] == 0 and data['ignored'] == expected_ignored
                         and all(x[0] == 'ok' for x in results))
    return data


def run_step(label: str, command: list[str], output: Path, timeout: int = 600) -> dict:
    log = output / (label + '.txt')
    env = os.environ.copy()
    env.pop('CARGO_BUILD_TARGET',None)
    env.update(CARGO_TARGET_DIR=str(ROOT/'target'), RUST_BACKTRACE='0', PYTHONUTF8='1', PYTHONIOENCODING='utf-8', CARGO_TERM_COLOR='never')
    start = time.monotonic()
    timed_out = False
    with log.open('wb') as handle:
        handle.write(('$ ' + ' '.join(command) + '\n').encode('utf-8'))
        handle.flush()
        try:
            proc = subprocess.Popen(command, cwd=ROOT, env=env, stdout=handle, stderr=subprocess.STDOUT,
                                    start_new_session=os.name != 'nt',
                                    creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if os.name == 'nt' else 0)
            try:
                code = proc.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                timed_out = True
                if os.name == 'nt':
                    subprocess.run(['taskkill', '/PID', str(proc.pid), '/T', '/F'], stdout=handle, stderr=handle, timeout=15, check=False)
                else:
                    os.killpg(proc.pid, signal.SIGKILL)
                proc.wait(timeout=15)
                code = 124
        except OSError as exc:
            handle.write(str(exc).encode('utf-8'))
            code = 127
    text = log.read_text(encoding='utf-8', errors='replace')
    item = dict(step=label, command=command, exit_code=code, timed_out=timed_out,
                seconds=round(time.monotonic()-start, 3), log=log.name,
                log_sha256=hashlib.sha256(log.read_bytes()).hexdigest(), passed=code == 0)
    if label in TEST_LIMITS:
        minimum, ignored = TEST_LIMITS[label]
        item['tests'] = test_summary(text, code, minimum=minimum, expected_ignored=ignored)
        item['passed'] = item['tests']['valid']
    return item


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='New directory, never overwrites earlier evidence')
    args = parser.parse_args()
    stamp = dt.datetime.now(dt.timezone.utc).strftime('%Y%m%dT%H%M%SZ')
    out = (args.output or ROOT.parent / ('qualification-' + platform.system().lower() + '-' + stamp)).resolve()
    out.mkdir(parents=True, exist_ok=False)
    start_id = source_id()
    report = dict(schema=1, version=(ROOT/'VERSION').read_text().strip(),
                  generated_utc=stamp, platform=platform.platform(), system=platform.system(), machine=platform.machine(),
                  source_id=start_id, engineering_gate='RUNNING', steps=[],
                  scope='Native local engineering tests only; not whole-course or all-platform certification',
                  unchecked=['browser HTTP navigation', 'other operating systems', 'MSRV unless this compiler is 1.74.0', 'real learners'])
    cargo = ['cargo']
    locked = ['--locked', '--offline', '--workspace', '--all-targets']
    plan = [
        ('rustc', ['rustc','-Vv']), ('cargo',['cargo','-V']),
        ('tooling-tests',[sys.executable,'-m','unittest','discover','-s','scripts/tests','-v']),
        ('structure',[sys.executable, 'scripts/structural_check.py']),
        ('format',cargo+['fmt','--all','--','--check']),
        ('debug-clippy',cargo+['clippy']+locked+['--','-D','warnings']),
        ('release-clippy',cargo+['clippy','--release']+locked+['--','-D','warnings']),
        ('debug-tests',cargo+['test']+locked+['--','--test-threads=2']),
        ('release-tests',cargo+['test','--release']+locked+['--','--test-threads=2']),
        ('doctests',cargo+['test','--locked','--offline','--workspace','--doc']),
        ('release-build',cargo+['build','--locked','--offline','--workspace','--release']),
        *advanced_b_steps(),
        *advanced_b4_steps(),
        *book_d1_steps(),
        *book_d2_steps(),
        *book_d3_steps(),
        *book_d4_steps(),
        *book_d5_steps(),
        *book_d6_steps(),
        *book_d6_foundations_steps(),
        ('studio-smoke',[sys.executable,'scripts/smoke_studio_release.py','--output',str(out/'studio-smoke.json')]),
        ('database-smoke',[sys.executable,'scripts/smoke_state_database.py','--output',str(out/'database-products')]),
        ('minikv-smoke',[sys.executable,'scripts/smoke_minikv.py','--output',str(out/'minikv-products')]),
        ('network-smoke',[sys.executable,'scripts/smoke_network.py','--output',str(out/'network-products')]),
        ('async-smoke',[sys.executable,'scripts/smoke_asyncboard.py']),
        ('backup-hash',[sys.executable,'scripts/bench_safesync.py','--files','64','--mib','8','--output',str(out/'backup-check.json')]),
    ]
    for label, command in plan:
        print(f'[{label}]', flush=True)
        item = run_step(label, command, out)
        report['steps'].append(item)
        (out/'engineering.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
        print(('PASS' if item['passed'] else 'FAIL')+' '+label, flush=True)
        if not item['passed']:
            break
    report['source_id_after'] = source_id()
    report['engineering_gate'] = 'PASS' if (len(report['steps']) == len(plan)
        and all(x['passed'] for x in report['steps']) and start_id == report['source_id_after']) else 'FAIL'
    report['completed_steps'] = len(report['steps'])
    report['planned_steps'] = len(plan)
    (out/'engineering.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print(report['engineering_gate'], str(out/'engineering.json'), flush=True)
    return 0 if report['engineering_gate'] == 'PASS' else 1


if __name__ == '__main__':
    raise SystemExit(main())
