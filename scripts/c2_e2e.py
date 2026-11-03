#!/usr/bin/env python3
"""Real platform lifecycle; no mocks and no dependency downloads here."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REF = ROOT / 'projects/advanced-c2/reference'

def run(exe, *args, env=None, success=True):
    result = subprocess.run([str(exe), *map(str,args)], cwd=ROOT, env=env,
                            text=True, encoding='utf-8', errors='replace', capture_output=True, timeout=600)
    if (result.returncode == 0) != success:
        diagnostic = ''
        if len(args) >= 3 and args[0] == 'advanced-c2' and args[1] == 'prepare':
            folder = Path(args[2]) / '.c2-prepare'
            logs = sorted(folder.glob('*.txt')) if folder.is_dir() else []
            if logs:
                diagnostic = '\nC2 prepare detail:\n' + logs[-1].read_text(encoding='utf-8', errors='replace')[-12000:]
        raise AssertionError(f'{args}: exit {result.returncode}\n{result.stdout}\n{result.stderr}{diagnostic}')
    return result.stdout + result.stderr

def main():
    p=argparse.ArgumentParser();p.add_argument('--exe',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    exe=a.exe.resolve();checks=[]
    with tempfile.TemporaryDirectory(prefix='c2-platform-') as temp:
        root=Path(temp)/'中文 learner';run(exe,'advanced-c2','start',root)
        starter=(root/'src/lib.rs').read_bytes()
        run(exe,'advanced-c2','start',root,success=False)
        assert (root/'src/lib.rs').read_bytes()==starter
        before=sorted(x.relative_to(root).as_posix() for x in root.rglob('*'))
        run(exe,'learn','reading',root);run(exe,'advanced-c2','next',root)
        assert before==sorted(x.relative_to(root).as_posix() for x in root.rglob('*'))
        checks.append('non-overwriting start and read-only navigation')
        (root/'src/lib.rs').write_text('compile_error!("learner executed in prepare");',encoding='utf-8')
        (root/'Cargo.toml').write_text('invalid manifest',encoding='utf-8')
        (root/'build.rs').write_text('fn main(){ panic!("untrusted build"); }',encoding='utf-8')
        (root/'.cargo').mkdir();(root/'.cargo/config.toml').write_text('invalid config',encoding='utf-8')
        env=dict(os.environ,CARGO_NET_OFFLINE='true')
        run(exe,'advanced-c2','prepare',root,env=env)
        assert not (root/'.c2-reports').exists()
        checks.append('offline preparation uses fixed reference, not learner build inputs')
        (root/'src/lib.rs').write_bytes(starter)
        run(exe,'advanced-c2','check',root,1,env=env,success=False)
        latest=lambda: sorted((root/'.c2-reports').glob('*.txt'))[-1].read_text(encoding='utf-8')
        assert 'passed=0\n' in latest() and 'C2 TODO' in latest()
        checks.append('initial learner is red')
        for path in (REF/'src').glob('*.rs'):shutil.copy2(path,root/'src'/path.name)
        (root/'tests/contracts.rs').write_text('compile_error!("learner tests executed");',encoding='utf-8')
        run(exe,'learn','check',root,'all',env=env)
        assert 'passed=3\n' in latest()
        assert '3/3' in run(exe,'advanced-c2','next',root)
        checks.append('reference passes fixed cumulative tests despite edited manifest/tests/build.rs/config')
        old_reports=len(list((root/'.c2-reports').glob('*.txt')))
        empty=Path(temp)/'empty-cargo';empty.mkdir()
        missing=dict(env,CARGO_HOME=str(empty));missing.pop('RUSTAPPLAB_C2_VENDOR',None)
        run(exe,'advanced-c2','check',root,1,env=missing,success=False)
        assert len(list((root/'.c2-reports').glob('*.txt')))==old_reports
        checks.append('missing dependency environment does not erase learner grade')
        (root/'src/handlers.rs').write_text('compile_error!("fixed infra must be pinned");',encoding='utf-8')
        assert '3/3' in run(exe,'advanced-c2','next',root)
        (root/'src/lib.rs').write_text('compile_error!("changed learner core");',encoding='utf-8')
        assert '过期' in run(exe,'advanced-c2','next',root)
        run(exe,'advanced-c2','check',root,3,env=env,success=False)
        assert 'passed=0\n' in latest()
        checks.append('fixed handler edits ignored; core changes invalidate results')
    a.output.write_text(json.dumps({'status':'PASS','checks':checks},ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps({'status':'PASS','checks':checks},ensure_ascii=False))

if __name__=='__main__':main()
