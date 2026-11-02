#!/usr/bin/env python3
"""Verify the declared minimum compiler; never substitute a newer compiler."""
import argparse
import json
from pathlib import Path
import platform
import re
from release_check import ROOT,run_step,source_id,advanced_b_steps

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    identity=source_id()
    report=dict(version=(ROOT/'VERSION').read_text().strip(),source_id=identity,system=platform.system(),status='RUNNING',steps=[])
    tool=run_step('rustc',['rustc','-Vv'],out)
    report['steps'].append(tool)
    matches=re.findall(r'^rustc .*$',(out/'rustc.txt').read_text(encoding='utf-8'),re.M)
    report['compiler']=matches[0] if matches else ''
    if not report['compiler'].startswith('rustc 1.74.0 '):
        report['status']='BLOCKED_ENVIRONMENT';report['reason']='Actual rustc is not the declared minimum 1.74.0; newer compiler results are not MSRV evidence.'
    else:
        plan=[('debug-tests',['cargo','test','--locked','--offline','--workspace','--all-targets','--','--test-threads=2']),
              ('release-tests',['cargo','test','--locked','--offline','--release','--workspace','--all-targets','--','--test-threads=2']),
              ('doctests',['cargo','test','--locked','--offline','--workspace','--doc'])]
        plan.extend(advanced_b_steps(quality=False))
        for label,command in plan:
            item=run_step(label,command,out);report['steps'].append(item)
            if not item['passed']:break
        report['status']='PASS' if len(report['steps'])==len(plan)+1 and all(x['passed'] for x in report['steps']) else 'FAIL'
    report['source_id_after']=source_id()
    if report['source_id_after']!=identity:report['status']='FAIL'
    (out/'msrv.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps(report,ensure_ascii=False,indent=2))
    return 0 if report['status']=='PASS' else 2
if __name__=='__main__':raise SystemExit(main())
