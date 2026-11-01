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
TEST_RESULT = re.compile(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;")
CODE_SUFFIXES = {'.rs', '.toml', '.lock', '.py', '.ps1', '.cmd', '.sh', '.html', '.css', '.js', '.yml', '.yaml'}


def source_id(root: Path = ROOT) -> str:
    """Code/build/launcher/UI content; excludes generated evidence, docs and caches."""
    h = hashlib.sha256()
    paths = []
    for path in root.rglob('*'):
        rel = path.relative_to(root)
        if not path.is_file() or any(x in {'target', '.git', '__pycache__'} for x in rel.parts):
            continue
        if rel.parts[0] not in {'apps', 'scripts', '.github'} and len(rel.parts) != 1:
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
    if label in {'debug-tests', 'release-tests'}:
        item['tests'] = test_summary(text, code)
        item['passed'] = item['tests']['valid']
    if label == 'doctests':
        item['tests'] = test_summary(text, code, minimum=2, expected_ignored=0)
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
