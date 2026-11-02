#!/usr/bin/env python3
"""Fail-closed acceptance decision. A prerelease filename is not qualification."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
from release_check import ROOT, source_id, test_summary, advanced_b_steps, TEST_LIMITS

REQUIRED = {'format','debug-clippy','release-clippy','debug-tests','release-tests','doctests','release-build'}
REQUIRED.update(name for name, _ in advanced_b_steps())


def engineering(path: Path | None, expected_id: str, expected_version: str) -> bool:
    if path is None:return False
    try:
        data=json.loads(path.read_text(encoding='utf-8-sig'))
        if data.get('version') != expected_version or data.get('source_id') != expected_id or data.get('source_id_after') != expected_id or data.get('engineering_gate') != 'PASS':return False
        items={item['step']:item for item in data['steps']}
        if not REQUIRED.issubset(items):return False
        for name,item in items.items():
            if item.get('exit_code') != 0 or item.get('timed_out') or not item.get('passed'):return False
            filename=item['log']
            if Path(filename).name != filename:return False
            log=path.parent/filename
            if hashlib.sha256(log.read_bytes()).hexdigest() != item['log_sha256']:return False
            if name in TEST_LIMITS:
                minimum, ignored = TEST_LIMITS[name]
                if not test_summary(log.read_text(encoding='utf-8',errors='replace'),0,
                                    minimum=minimum,expected_ignored=ignored)['valid']:return False
        return True
    except (OSError,ValueError,KeyError,TypeError):return False


def evidence(path: Path | None, expected_id: str, kind: str) -> bool:
    if path is None:return False
    try:
        data=json.loads(path.read_text(encoding='utf-8-sig'))
        if data.get('source_id') != expected_id or data.get('status') != 'PASS':return False
        if kind=='browser':
            checks=data.get('checks',[])
            return (data.get('source_id_after') == expected_id and len(checks)==2
                    and {x.get('app') for x in checks}=={'localhub','servicedesk'}
                    and all(x.get('status')=='PASS' for x in checks)
                    and data.get('method')=='real browser navigation and fetch, no mocks')
        if kind=='archive':return bool(data.get('source_zip_sha256') and data.get('fresh_debug_tests',0)>=200 and data.get('fresh_release_tests',0)>=200)
        if kind=='msrv':return data.get('compiler','').startswith('rustc 1.74.0 ')
        if kind=='learners':return bool(data.get('observed_tasks') and data.get('unresolved_blockers')==[])
    except (OSError,ValueError,TypeError):return False
    return False


def decide(gates: dict[str,bool]) -> str:
    if not all(gates.get(x,False) for x in ('local_engineering','real_browser','fresh_archive')):
        return 'NOT_QUALIFIED'
    if all(gates.get(x,False) for x in ('windows_native','declared_msrv','learner_acceptance')):
        return 'STABLE_READY'
    return 'RC_READY'


def main() -> int:
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('engineering','browser','archive','windows','msrv','learners'):
        parser.add_argument('--'+name,type=Path)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    identity=source_id();version=(ROOT/'VERSION').read_text().strip()
    gates=dict(local_engineering=engineering(args.engineering,identity,version),
               real_browser=evidence(args.browser,identity,'browser'),fresh_archive=evidence(args.archive,identity,'archive'),
               windows_native=engineering(args.windows,identity,version),
               declared_msrv=evidence(args.msrv,identity,'msrv'),learner_acceptance=evidence(args.learners,identity,'learners'))
    # A Linux report cannot stand in for Windows, even with valid hashes.
    if gates['windows_native']:
        gates['windows_native'] = json.loads(args.windows.read_text(encoding='utf-8-sig')).get('system') == 'Windows'
    report=dict(version=version,source_id=identity,qualification=decide(gates),gates=gates,
                missing=[key for key,value in gates.items() if not value],
                note='Official platform scope: Windows x64 MSVC and Linux x86_64; macOS is not a Stable gate. Historic RC evidence is not inherited after source changes.')
    args.output.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps(report,ensure_ascii=False,indent=2))
    return 0 if report['qualification'] in {'RC_READY','STABLE_READY'} else 2

if __name__=='__main__':raise SystemExit(main())
