#!/usr/bin/env python3
"""Build two portable C1 source kits, then replay the ZIPs with an empty Cargo cache."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import zipfile
from release_check import ROOT, source_id

CONFIG='[source.crates-io]\nreplace-with = "c1-vendor"\n[source.c1-vendor]\ndirectory = "vendor"\n'

def write_kit_configuration(stage: Path, kind: str) -> None:
    """An unpacked learner is a platform-recognized project, not just source files."""
    if kind not in ('reference', 'learner'):
        raise ValueError('unknown C1 kit kind')
    (stage / '.cargo').mkdir(exist_ok=True)
    (stage / '.cargo/config.toml').write_bytes(CONFIG.encode('utf-8'))
    if kind == 'learner':
        (stage / '.applab-c1').write_bytes(b'RustAppLab C1 v1\n')


def run(command,cwd,log,env=None,expect=0):
    with log.open('w',encoding='utf-8') as stream:
        stream.write('$ '+' '.join(map(str,command))+'\n');stream.flush()
        result=subprocess.run(list(map(str,command)),cwd=cwd,env=env,stdout=stream,stderr=subprocess.STDOUT,timeout=900)
    if result.returncode!=expect:raise RuntimeError(log.read_text(encoding='utf-8',errors='replace')[-12000:])
    return log.read_text(encoding='utf-8',errors='replace')

def pack(source,output):
    with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for path in sorted(source.rglob('*'), key=lambda p: p.relative_to(source).as_posix()):
            if path.is_file():
                info=zipfile.ZipInfo(path.relative_to(source).as_posix(),date_time=(2026,11,3,0,0,0))
                info.create_system=3;info.compress_type=zipfile.ZIP_DEFLATED;info.external_attr=0o100644<<16
                z.writestr(info,path.read_bytes())
    with zipfile.ZipFile(output) as z:
        assert z.testzip() is None
        assert '.cargo/config.toml' in z.namelist()
        assert any(n.endswith('/.cargo-checksum.json') for n in z.namelist())
    return hashlib.sha256(output.read_bytes()).hexdigest()

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);before=source_id();artifacts=[]
    with tempfile.TemporaryDirectory(prefix='c1-package-') as temp:
        temp=Path(temp);vendor=temp/'vendor'
        run(['cargo','+1.85.0','vendor','--locked','--versioned-dirs','--manifest-path',ROOT/'projects/advanced-c1/reference/Cargo.toml',vendor],ROOT,out/'vendor.txt')
        dependencies=[x for x in tomllib.loads((ROOT/'projects/advanced-c1/reference/Cargo.lock').read_text())['package'] if x.get('source')]
        platform_target=temp/'platform-target'
        run(['cargo','+1.85.0','build','--locked','--offline','-p','labquest','--target-dir',platform_target],ROOT,out/'platform-build.txt')
        platform_exe=platform_target/'debug'/('labquest.exe' if os.name=='nt' else 'labquest')
        for kind in ('reference','learner'):
            stage=temp/('stage-'+kind)
            shutil.copytree(ROOT/'projects/advanced-c1'/kind,stage,ignore=shutil.ignore_patterns('target'))
            shutil.copytree(vendor,stage/'vendor')
            write_kit_configuration(stage, kind)
            shutil.copy2(ROOT/'LICENSE',stage/'LICENSE');shutil.copy2(ROOT/'course/ADVANCED_C1.md',stage/'START_HERE.md')
            archive=out/f'C1-{kind}-offline.zip';digest=pack(stage,archive)
            archive.with_suffix('.zip.sha256').write_bytes(f'{digest}  {archive.name}\n'.encode('utf-8'))
            moved=temp/('搬移 空缓存 '+kind);moved.mkdir()
            with zipfile.ZipFile(archive) as z:z.extractall(moved)
            home=temp/('cargo-empty-'+kind);home.mkdir()
            env=dict(os.environ,CARGO_HOME=str(home),CARGO_NET_OFFLINE='true',CARGO_TERM_COLOR='never',CARGO_TARGET_DIR=str(temp/('target-'+kind)))
            env.pop('CARGO_BUILD_TARGET',None);env.pop('RUSTAPPLAB_C1_VENDOR',None)
            if kind=='reference':
                text=run(['cargo','+1.85.0','test','--frozen','--all-targets'],moved,out/(kind+'-replay.txt'),env)
                test_source=(ROOT/'projects/advanced-c1/reference/tests/contracts.rs').read_text(encoding='utf-8')
                expected=test_source.count('#[test]')+test_source.count('#[tokio::test]')
                assert f'{expected} passed; 0 failed; 0 ignored;' in text
                run(['cargo','+1.85.0','test','--frozen','--doc'],moved,out/(kind+'-docs.txt'),env)
            else:
                run(['cargo','+1.85.0','check','--frozen','--all-targets'],moved,out/(kind+'-compiles.txt'),env)
                text=run(['cargo','+1.85.0','test','--frozen','--test','contracts','c1_1_'],moved,out/(kind+'-red.txt'),env,101)
                assert 'C1 TODO' in text and 'test result: FAILED' in text
                assert (moved/'src/lib.rs').read_text(encoding='utf-8').count('TODO(')==3
                assert not (moved/'src/transport.rs').exists() and not (moved/'src/queue.rs').exists()
                assert (moved/'.applab-c1').read_bytes()==b'RustAppLab C1 v1\n'
                platform_env=dict(env,RUSTAPPLAB_C1_VENDOR=str(moved/'vendor'))
                run([platform_exe,'advanced-c1','next',moved],ROOT,out/'learner-navigation.txt',platform_env)
                run([platform_exe,'advanced-c1','check',moved,'1'],ROOT,out/'learner-fixed-red.txt',platform_env,2)
                latest=lambda: sorted((moved/'.c1-reports').glob('*.txt'))[-1].read_text(encoding='utf-8')
                assert 'passed=0\n' in latest() and 'C1 TODO' in latest()
                # Only the extracted test copy is completed; the shipped learner ZIP stays TODO-only.
                for file in (ROOT/'projects/advanced-c1/reference/src').glob('*.rs'):
                    shutil.copy2(file,moved/'src'/file.name)
                run([platform_exe,'advanced-c1','check',moved,'all'],ROOT,out/'learner-fixed-green.txt',platform_env)
                assert 'passed=3\n' in latest()
                assert hashlib.sha256(archive.read_bytes()).hexdigest()==digest

            assert not (home/'registry').exists(), 'offline replay used a registry cache'
            item={'file':archive.name,'sha256':digest,'bytes':archive.stat().st_size,'registry_dependencies':len(dependencies),'empty_cache_relocated_replay':'PASS'}
            if kind=='learner':item['platform_fixed_assessment']='RED_THEN_GREEN'
            artifacts.append(item)
    after=source_id();assert before==after
    (out/'portable.json').write_text(json.dumps({'status':'PASS','source_id':before,'source_id_after':after,'artifacts':artifacts},ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps(artifacts,ensure_ascii=False))
if __name__=='__main__':main()
