#!/usr/bin/env python3
"""Source-bound C1 gate. Dependency preparation is a separate explicit CI step."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
from release_check import source_id
ROOT=Path(__file__).resolve().parents[1]
MANIFEST='projects/advanced-c1/reference/Cargo.toml'

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    before=source_id();steps=[]
    common=['--locked','--offline','--manifest-path',MANIFEST]
    exe=ROOT/'target'/'debug'/('labquest.exe' if os.name=='nt' else 'labquest')
    test_source=(ROOT/'projects/advanced-c1/reference/tests/contracts.rs').read_text(encoding='utf-8')
    expected=test_source.count('#[test]')+test_source.count('#[tokio::test]')
    plan=[('compiler',['rustc','-Vv'],0),('format',['cargo','fmt','--manifest-path',MANIFEST,'--all','--','--check'],0),('clippy',['cargo','clippy']+common+['--all-targets','--','-D','warnings'],0),('debug',['cargo','test']+common+['--all-targets'],expected),('release',['cargo','test']+common+['--release','--all-targets'],expected),('docs',['cargo','test']+common+['--doc'],1),('platform-build',['cargo','build','--locked','--offline','-p','labquest'],0),('platform-lifecycle',[sys.executable,'scripts/c1_e2e.py','--exe',str(exe),'--output',str(out/'lifecycle.json')],0)]
    env=dict(os.environ,CARGO_TARGET_DIR=str(ROOT/'target'),CARGO_TERM_COLOR='never',PYTHONUTF8='1')
    env.pop('CARGO_BUILD_TARGET',None)
    good=True
    for label,command,count in plan:
        log=out/(label+'.txt')
        try:
            with log.open('w',encoding='utf-8') as f:
                f.write('$ '+' '.join(command)+'\n');f.flush()
                code=subprocess.run(command,cwd=ROOT,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=1200).returncode
        except (OSError,subprocess.TimeoutExpired) as e:
            with log.open('a',encoding='utf-8') as f:f.write(str(e))
            code=124
        text=log.read_text(encoding='utf-8',errors='replace');valid=code==0
        suites=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
        if count:valid=valid and sum(int(x[1]) for x in suites)==count and all(x[0]=='ok' and x[2]=='0' and x[3]=='0' for x in suites)
        steps.append({'step':label,'exit_code':code,'passed':valid,'expected_tests':count,'log':log.name,'sha256':hashlib.sha256(log.read_bytes()).hexdigest()})
        print(label,'PASS' if valid else 'FAIL',flush=True)
        if not valid: print(text[-15000:]);good=False;break
    after=source_id();good=good and before==after
    data={'status':'PASS' if good else 'FAIL','source_id':before,'source_id_after':after,'platform':platform.platform(),'steps':steps}
    (out/'c1.json').write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
    return 0 if good else 1
if __name__=='__main__':raise SystemExit(main())
