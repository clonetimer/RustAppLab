#!/usr/bin/env python3
"""Fail-closed acceptance decision. A prerelease filename is not qualification."""
from __future__ import annotations
import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
from release_check import ROOT, source_id, test_summary, advanced_b_steps, advanced_b4_steps, book_d1_steps, book_d2_steps, book_d3_steps, book_d4_steps, TEST_LIMITS

REQUIRED = {'format','debug-clippy','release-clippy','debug-tests','release-tests','doctests','release-build'}
REQUIRED.update(name for name, _ in advanced_b_steps())
REQUIRED.update(name for name, _ in advanced_b4_steps())
REQUIRED.update(name for name, _ in book_d1_steps())
REQUIRED.update(name for name, _ in book_d2_steps())
REQUIRED.update(name for name, _ in book_d3_steps())
REQUIRED.update(name for name, _ in book_d4_steps())


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


def observed_text(value) -> bool:
    """Reject empty template fields, without pretending to grade an explanation."""
    return (isinstance(value, str) and bool(value.strip().strip("_.-…").strip())
            and value.strip().casefold() not in
            {'todo', 'pending', 'placeholder', 'unverified', '待填写', '未评估', '未观察', '未核实'})


def observed_time(value) -> bool:
    try:
        return isinstance(value, str) and datetime.fromisoformat(value.replace('Z', '+00:00')).utcoffset() is not None
    except ValueError:
        return False


def evidence_bytes(root: Path, name, expected_hash) -> bytes:
    """Read only bounded, hashed regular files inside the evidence directory."""
    if (not isinstance(name, str) or not name or '\\' in name or ':' in name
            or Path(name).is_absolute() or '..' in Path(name).parts
            or not isinstance(expected_hash, str) or not re.fullmatch('[0-9a-f]{64}', expected_hash)):
        raise ValueError('invalid evidence path or hash')
    target = root
    for part in Path(name).parts:
        target = target / part
        if target.is_symlink():
            raise ValueError('evidence must not follow links')
    target.resolve().relative_to(root.resolve())
    if not target.is_file() or not 0 < target.stat().st_size <= 4 * 1024 * 1024:
        raise ValueError('missing, empty or oversized evidence')
    with target.open('rb') as source:
        data = source.read(4 * 1024 * 1024 + 1)
    if len(data) > 4 * 1024 * 1024 or hashlib.sha256(data).hexdigest() != expected_hash:
        raise ValueError('changed evidence')
    return data


def learner_evidence(path: Path, data: dict, expected_id: str, expected_version: str | None) -> bool:
    """Validate a human review's completeness/integrity; never infer human mastery."""
    if (not expected_version or type(data.get('schema')) is not int or data['schema'] != 1
            or data.get('version') != expected_version
            or data.get('source_id_after') != expected_id or data.get('unresolved_blockers') != []
            or data.get('method') != 'reviewed-human-observation'
            or not observed_text(data.get('reviewer_id')) or not observed_time(data.get('reviewed_at'))):
        return False
    tasks = data.get('observed_tasks')
    if not isinstance(tasks, list) or not 3 <= len(tasks) <= 100:
        return False
    reviewed_at = datetime.fromisoformat(data['reviewed_at'].replace('Z', '+00:00'))
    routes, sessions = set(), set()
    for task in tasks:
        if not isinstance(task, dict) or task.get('route') not in {'firstcard', 'unitlab', 'independent'}:
            return False
        record = json.loads(evidence_bytes(path.parent, task.get('record'), task.get('sha256')))
        if (not isinstance(record, dict) or type(record.get('schema')) is not int or record['schema'] != 1
                or record.get('kind') != 'human-learner-session' or record.get('route') != task['route']
                or record.get('version') != expected_version or record.get('source_id') != expected_id
                or not observed_time(record.get('observed_at'))):
            return False
        if datetime.fromisoformat(record['observed_at'].replace('Z', '+00:00')) > reviewed_at:
            return False
        fields = ('session_id', 'learner_id', 'observer_id', 'task_id', 'prediction', 'explanation',
                  'error_explanation', 'transfer_task', 'transfer_result')
        if not all(observed_text(record.get(key)) for key in fields):
            return False
        if record['session_id'] in sessions:
            return False
        sessions.add(record['session_id'])
        hints = record.get('hints_used')
        if not isinstance(hints, list) or not all(observed_text(hint) for hint in hints):
            return False
        artifacts = record.get('artifacts')
        if not isinstance(artifacts, list) or not 3 <= len(artifacts) <= 32:
            return False
        kinds, files = set(), set()
        for artifact in artifacts:
            if not isinstance(artifact, dict) or artifact.get('kind') not in {'learner_source', 'command_log', 'transfer_work'}:
                return False
            evidence_bytes(path.parent, artifact.get('path'), artifact.get('sha256'))
            resolved = (path.parent / artifact['path']).resolve()
            if resolved in files:
                return False
            files.add(resolved)
            kinds.add(artifact['kind'])
        if kinds != {'learner_source', 'command_log', 'transfer_work'}:
            return False
        routes.add(task['route'])
    return routes == {'firstcard', 'unitlab', 'independent'}


def evidence(path: Path | None, expected_id: str, kind: str, expected_version: str | None = None) -> bool:
    if path is None:return False
    try:
        data=json.loads(path.read_text(encoding='utf-8-sig'))
        if not isinstance(data, dict):return False
        if data.get('source_id') != expected_id or data.get('status') != 'PASS':return False
        if kind=='browser':
            checks=data.get('checks',[])
            return (data.get('source_id_after') == expected_id and len(checks)==2
                    and {x.get('app') for x in checks}=={'localhub','servicedesk'}
                    and all(x.get('status')=='PASS' for x in checks)
                    and data.get('method')=='real browser navigation and fetch, no mocks')
        if kind=='archive':return bool(data.get('source_zip_sha256') and data.get('fresh_debug_tests',0)>=200 and data.get('fresh_release_tests',0)>=200)
        if kind=='msrv':return data.get('compiler','').startswith('rustc 1.74.0 ')
        if kind=='learners':return learner_evidence(path,data,expected_id,expected_version)
    except (OSError,ValueError,KeyError,TypeError):return False
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
               declared_msrv=evidence(args.msrv,identity,'msrv'),learner_acceptance=evidence(args.learners,identity,'learners',version))
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
