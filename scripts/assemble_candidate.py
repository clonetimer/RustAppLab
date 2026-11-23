#!/usr/bin/env python3
"""Collect only verified, source-bound RC assets from this workflow's artifacts."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import shutil
import struct
import subprocess
import re
import zipfile
from release_check import ROOT, source_id
import release_decision
import book_d6_check
import workbench_browser_e2e as workbench


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def safe_child(folder: Path, name: str) -> Path:
    relative=PurePosixPath(name)
    if relative.is_absolute() or '..' in relative.parts or '\\' in name:
        raise ValueError('unsafe evidence path: '+name)
    path=(folder/name).resolve()
    if folder.resolve() not in path.parents: raise ValueError('evidence escaped its directory')
    return path


def read_report(path: Path, identity: str, *, engineering=False) -> dict:
    data=json.loads(path.read_text(encoding='utf-8-sig'))
    if data.get('source_id')!=identity or data.get('source_id_after')!=identity:
        raise ValueError(f'wrong or changed source: {path}')
    field='engineering_gate' if engineering else 'status'
    if data.get(field)!='PASS':raise ValueError(f'failed report: {path}')
    for step in data.get('steps',[]):
        if step.get('passed') is not True or step.get('timed_out'):
            raise ValueError(f'failed step in {path}: {step.get("step")}')
        log=safe_child(path.parent,step['log'])
        expected=step.get('log_sha256',step.get('sha256'))
        if not expected or sha256(log)!=expected:raise ValueError('log digest mismatch: '+str(log))
    return data


def read_book_practice_report(path: Path, mode: str, system: str, commit: str | None) -> dict:
    """D/E inputs have their own scope, separate from the full release source ID."""
    expected_snapshot = book_d6_check.source_snapshot(ROOT)
    data = read_report(path, book_d6_check.snapshot_id(expected_snapshot))
    if (data.get('mode') != mode or data.get('system') != system
            or data.get('toolchain') != '1.85.0'
            or (commit is not None and data.get('git_commit') != commit)
            or data.get('runner_sha256') != sha256(ROOT/'scripts/book_d6_check.py')):
        raise ValueError('wrong practice revision/platform/runner: '+str(path))
    if (data.get('source_snapshot_before') != expected_snapshot
            or data.get('source_snapshot_after') != expected_snapshot
            or data.get('source_unchanged') is not True
            or data.get('changed_source_paths') != []):
        raise ValueError('practice source snapshot changed: '+str(path))
    required = {'rustc-version', 'cargo-version', 'rustfmt-version', 'clippy-version',
                'rustdoc-version', 'format', 'format-check', 'clippy',
                'debug-tests', 'doctests', 'release-tests'}
    required |= ({'rust-analyzer-version', 'unused-mut-diagnostic', 'repair-unused-mut', 'documentation'}
                 if mode == 'tools' else {'original-tests', 'edition-2021-probe', 'edition-fix'})
    names = [step['step'] for step in data.get('steps', [])]
    if len(names) != len(set(names)) or not required.issubset(names):
        raise ValueError('practice command evidence incomplete: '+str(path))
    for step in data['steps']:
        expected = 101 if step['step'] == 'unused-mut-diagnostic' else 0
        if step.get('returncode') != expected or step.get('expected_returncode') != expected:
            raise ValueError('unexpected practice exit code: '+str(path))
    evidence = data.get('evidence', [])
    expected_files = {'manifest-before.txt', 'manifest-after.txt', 'source-before.txt',
                      'source-after.txt', 'changes.diff.txt', 'ide-workspace.json'}
    if not expected_files.issubset({item['file'] for item in evidence}):
        raise ValueError('practice before/after evidence incomplete: '+str(path))
    for item in evidence:
        if sha256(safe_child(path.parent, item['file'])) != item.get('sha256'):
            raise ValueError('practice evidence digest mismatch: '+str(path))
    ide = json.loads((path.parent/'ide-workspace.json').read_text(encoding='utf-8'))
    settings = {
        'rust-analyzer.cargo.extraEnv': {'RUSTUP_TOOLCHAIN': '1.85.0'},
        'rust-analyzer.cargo.extraArgs': ['--locked', '--offline'],
        'rust-analyzer.cargo.buildScripts.enable': False,
        'rust-analyzer.procMacro.enable': False,
    }
    folders = ide.get('folders') if isinstance(ide, dict) else None
    if (not isinstance(ide, dict) or set(ide) != {'folders', 'extensions', 'settings'}
            or not isinstance(folders, list) or len(folders) != 1
            or not isinstance(folders[0], dict) or folders[0].get('path') != 'lab'
            or not set(folders[0]).issubset({'name', 'path'})
            or ide.get('extensions') != {'recommendations': ['rust-lang.rust-analyzer']}
            or ide.get('settings') != settings):
        raise ValueError('wrong or incomplete IDE workspace evidence: '+str(path))
    # CI retains D/E copies and records their actual paths. Interpret paths using
    # the producing platform, not the host reading the downloaded report.
    path_type = PureWindowsPath if system == 'Windows' else PurePosixPath
    workspace = data.get('workspace')
    if (data.get('workspace_retained') is not True or not isinstance(workspace, str)
            or not path_type(workspace).is_absolute() or '..' in path_type(workspace).parts):
        raise ValueError('IDE practice copy was not retained: '+str(path))
    base = path_type(workspace)
    live_paths = {'project_directory': base/'lab',
                  'ide_workspace_file': base/'D6-IDE.code-workspace',
                  'practice_source': base/'lab'/'appendix-lab'/'src'/'lib.rs'}
    if mode == 'tools':
        live_paths['documentation_index'] = base/'target'/'doc'/'book_d6_appendix_lab'/'index.html'
    elif data.get('documentation_index') is not None:
        raise ValueError('Edition report advertises documentation it did not build')
    for name, expected_path in live_paths.items():
        actual = data.get(name)
        if not isinstance(actual, str) or path_type(actual) != expected_path:
            raise ValueError('wrong retained IDE path '+name+': '+str(path))
    return data


def read_workbench_report(path: Path, identity: str) -> dict:
    """Require real completed jobs, current runner inputs and intact PNG evidence."""
    data = read_report(path, identity)
    if (data.get('schema') != 1 or not data.get('browser_version')
            or data.get('javascript_errors') != []
            or set(data.get('observations', [])) != workbench.OBSERVATIONS):
        raise ValueError('incomplete workbench browser observation: '+str(path))
    jobs = data.get('actions', {})
    if set(jobs) != set(workbench.ACTIONS):
        raise ValueError('missing workbench action: '+str(path))
    seen_ids = set()
    for name, expected in workbench.ACTIONS.items():
        job = jobs[name]
        if (tuple(job.get(key) for key in ('course_id','action','step','status')) != expected
                or type(job.get('id')) is not int or job['id'] in seen_ids
                or type(job.get('exit_code')) is not int
                or (job['exit_code'] == 0) != (expected[-1] == 'succeeded')
                or job.get('stale') is not False or not job.get('source_digest')
                or not job.get('output')):
            raise ValueError('invalid workbench job: '+name)
        seen_ids.add(job['id'])
    revision = data.get('baseline_revision', '')
    if not re.fullmatch(r'[0-9a-f]{40}', revision):
        raise ValueError('missing static baseline revision')
    if subprocess.run(['git','cat-file','-e',revision+':apps/labquest/src/workbench.rs'],
                      cwd=ROOT, capture_output=True).returncode == 0:
        raise ValueError('static baseline already contains the workbench')
    for course, repo_path in workbench.BASE_PAGES.items():
        content = subprocess.check_output(['git','show',revision+':'+repo_path], cwd=ROOT)
        expected = {'source': repo_path, 'sha256': hashlib.sha256(content).hexdigest()}
        if data.get('baseline_pages', {}).get(course) != expected:
            raise ValueError('incorrect before-page evidence: '+course)
    expected_shots = workbench.screenshot_sizes()
    screenshots = data.get('screenshots', {})
    if set(screenshots) != set(expected_shots):
        raise ValueError('incomplete workbench screenshot inventory')
    for name, (width,height) in expected_shots.items():
        shot = safe_child(path.parent, name)
        raw = shot.read_bytes()
        if (screenshots[name].get('sha256') != hashlib.sha256(raw).hexdigest()
                or screenshots[name].get('viewport') != {'width':width,'height':height}
                or raw[:8] != b'\x89PNG\r\n\x1a\n' or raw[12:16] != b'IHDR'
                or raw[16:24] != struct.pack('>II', width, height)):
            raise ValueError('workbench screenshot bytes/viewport mismatch: '+name)
    return data


def collect_assets(inputs: Path, output: Path) -> dict:
    identity=source_id();version=(ROOT/'VERSION').read_text().strip()
    if '-rc.' not in version: raise ValueError('only candidate versions may use this publisher')
    linux=inputs/'native-ubuntu-latest/native-evidence/engineering.json'
    windows=inputs/'native-windows-latest/native-evidence/engineering.json'
    browser=inputs/'real-browser/browser.json'
    msrv=inputs/'declared-msrv/msrv.json'
    checked=[]
    def checked_report(path,**kw):
        result=read_report(path,identity,**kw);checked.append(str(path.relative_to(inputs)));return result
    for path in (linux,windows):
        checked_report(path,engineering=True)
        if not release_decision.engineering(path,identity,version):raise ValueError('native qualification missing: '+str(path))
    checked_report(inputs/'native-windows-latest/windows-launcher-evidence/engineering.json',engineering=True)
    checked_report(msrv);checked_report(browser)
    workbench_path = inputs/'real-browser/workbench/workbench.json'
    read_workbench_report(workbench_path, identity)
    checked.append(str(workbench_path.relative_to(inputs)))
    for runner, system in (('ubuntu-latest', 'Linux'), ('windows-latest', 'Windows')):
        for mode in ('tools', 'edition'):
            path = inputs/f'book-practice-{runner}'/mode/'report.json'
            read_book_practice_report(path, mode, system, os.environ.get('GITHUB_SHA'))
            checked.append(str(path.relative_to(inputs)))
    for kind,path in (('msrv',msrv),('browser',browser)):
        if not release_decision.evidence(path,identity,kind):raise ValueError('invalid '+kind+' report')
    c2browser=json.loads((inputs/'real-browser/c2-browser.json').read_text(encoding='utf-8'))
    if c2browser.get('status')!='PASS' or len(c2browser.get('checks',[]))!=3:raise ValueError('C2 real browser evidence missing')
    for course in ('c1','c2'):
        for matrix in ('ubuntu-latest-1.85.0','ubuntu-latest-stable','windows-latest-stable'):
            folder=inputs/f'{course}-{matrix}'
            data=checked_report(folder/(course+'.json'))
            if len(data.get('steps',[]))!=8:raise ValueError('incomplete course gate')
            life=json.loads((folder/'lifecycle.json').read_text(encoding='utf-8'))
            if life.get('status')!='PASS' or len(life.get('checks',[]))!=6:raise ValueError('incomplete learner lifecycle')
    asset_sources={}
    def add_file(path: Path):
        if not path.is_file():raise FileNotFoundError(path)
        previous=asset_sources.get(path.name)
        if previous and sha256(previous)!=sha256(path):raise ValueError('different cross-platform bytes: '+path.name)
        asset_sources[path.name]=path
    for course in ('c1','c2'):
        for system in ('ubuntu-latest','windows-latest'):
            folder=inputs/f'{course}-portable-{system}';data=checked_report(folder/'portable.json')
            if len(data.get('artifacts',[]))!=2:raise ValueError('missing portable variants')
            for item in data['artifacts']:
                path=safe_child(folder,item['file'])
                if item.get('empty_cache_relocated_replay')!='PASS' or sha256(path)!=item['sha256']:
                    raise ValueError('portable replay/hash failed')
                if 'learner' in path.name and item.get('platform_fixed_assessment')!='RED_THEN_GREEN':
                    raise ValueError('portable learner fixed assessment missing')
                add_file(path)
            add_file(folder/f'RustAppLab-{version}-source.zip')
    for system,os_name in (('ubuntu-latest','Linux'),('windows-latest','Windows')):
        folder=inputs/f'delivery-{system}';data=checked_report(folder/'candidate.json')
        if data.get('version')!=version or data.get('system')!=os_name:raise ValueError('wrong native delivery version/platform')
        required={'workbench-relocated-launcher','firstcard-create','firstcard-no-overwrite','firstcard-red','firstcard-green','firstcard-personal-output','firstcard-readonly-next','c1-real-demo','c2-runtime-restart-0','c2-runtime-restart-1'}
        if not required.issubset({step['step'] for step in data['steps']}):raise ValueError('first-use path incomplete')
        fresh=checked_report(folder/'fresh-archive.json')
        source=folder/f'RustAppLab-{version}-source.zip'
        if fresh.get('source_zip_sha256')!=sha256(source) or not release_decision.evidence(folder/'fresh-archive.json',identity,'archive'):
            raise ValueError('source archive replay missing')
        for item in data['artifacts']:
            path=safe_child(folder,item['name'])
            if sha256(path)!=item['sha256'] or path.stat().st_size!=item['bytes']:raise ValueError('native artifact hash mismatch')
            add_file(path)
    expected={f'RustAppLab-{version}-source.zip',f'RustAppLab-{version}-learner-kit.zip',
              f'RustAppLab-{version}-linux-x86_64-runtime.zip',f'RustAppLab-{version}-windows-x64-msvc-runtime.zip',
              'C1-reference-offline.zip','C1-learner-offline.zip','C2-reference-offline.zip','C2-learner-offline.zip'}
    if set(asset_sources)!=expected:raise ValueError('candidate asset set is incomplete or unexpected')
    if output.exists():raise ValueError('output already exists; never overwrite candidate assets')
    output.mkdir(parents=True)
    for path in asset_sources.values():shutil.copy2(path,output/path.name)
    shutil.copy2(ROOT/'README_START_HERE.md',output/'QUICK_START.md')
    shutil.copy2(ROOT/'course/USER_TRIAL.md',output/'USER_TRIAL.md')
    with zipfile.ZipFile(output/'VERIFICATION_EVIDENCE.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for path in sorted(inputs.rglob('*')):
            if path.is_file() and path.suffix.lower() in ('.json','.txt','.log','.png'):
                z.write(path,path.relative_to(inputs).as_posix())
    gates=dict(local_engineering=True,windows_native=True,real_browser=True,fresh_archive=True,declared_msrv=True,learner_acceptance=False)
    qualification=release_decision.decide(gates)
    if qualification!='RC_READY':raise ValueError('unexpected qualification')
    files=[{'name':p.name,'bytes':p.stat().st_size,'sha256':sha256(p)} for p in sorted(output.iterdir()) if p.is_file()]
    result={'schema':1,'version':version,'commit':os.environ.get('GITHUB_SHA'),'source_id':identity,'qualification':qualification,'prerelease':True,'human_observation':'PENDING — no fabricated learner acceptance','gates':gates,'additional_gates':{'c1':True,'c2':True,'native_runtime_and_first_use':True,'workbench_browser':True,'book_tools_and_edition':True,'book_ide_entry_structure':True},'checked_reports':checked,'files':files}
    (output/'RELEASE_MANIFEST.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    hashes=''.join(f'{sha256(p)}  {p.name}\n' for p in sorted(output.iterdir()) if p.is_file())
    (output/'SHA256SUMS.txt').write_bytes(hashes.encode())
    return result


def main() -> int:
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    result=collect_assets(a.input.resolve(),a.output.resolve());print(json.dumps(result,ensure_ascii=False,indent=2));return 0

if __name__=='__main__':raise SystemExit(main())
