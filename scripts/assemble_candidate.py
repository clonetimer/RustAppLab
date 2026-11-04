#!/usr/bin/env python3
"""Collect only verified, source-bound RC assets from this workflow's artifacts."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import zipfile
from release_check import ROOT, source_id
import release_decision


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
        required={'firstcard-create','firstcard-no-overwrite','firstcard-red','firstcard-green','firstcard-personal-output','firstcard-readonly-next','c1-real-demo','c2-runtime-restart-0','c2-runtime-restart-1'}
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
    result={'schema':1,'version':version,'commit':os.environ.get('GITHUB_SHA'),'source_id':identity,'qualification':qualification,'prerelease':True,'human_observation':'PENDING — no fabricated learner acceptance','gates':gates,'additional_gates':{'c1':True,'c2':True,'native_runtime_and_first_use':True},'checked_reports':checked,'files':files}
    (output/'RELEASE_MANIFEST.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    hashes=''.join(f'{sha256(p)}  {p.name}\n' for p in sorted(output.iterdir()) if p.is_file())
    (output/'SHA256SUMS.txt').write_bytes(hashes.encode())
    return result


def main() -> int:
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--input',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    result=collect_assets(a.input.resolve(),a.output.resolve());print(json.dumps(result,ensure_ascii=False,indent=2));return 0

if __name__=='__main__':raise SystemExit(main())
