#!/usr/bin/env python3
"""Run the actual advanced CLI and inspect persisted files independently. No student claim."""
from pathlib import Path
import argparse,hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
target=Path(os.environ.get('CARGO_TARGET_DIR',str(ROOT/'target')))/'release'/'examples'
exe=target/('workbench.exe' if os.name=='nt' else 'workbench');lab=target/('ownership_lab.exe' if os.name=='nt' else 'ownership_lab')
calls=[]
def run(binary,args,stdin=None,ok=True):
 r=subprocess.run([str(binary),*map(str,args)],input=stdin,text=True,capture_output=True,timeout=20)
 idx=len(calls)+1;(out/f'call-{idx}.txt').write_text(r.stdout+'\nSTDERR\n'+r.stderr)
 calls.append(dict(command=[str(binary),*map(str,args)],exit_code=r.returncode,expected_success=ok))
 assert (r.returncode==0)==ok,(r.stdout,r.stderr)
 return r

def records(path):
 lines=path.read_text(encoding='utf-8').splitlines();assert lines[0]=='TERMGARDEN\t1'
 return [line.split('\t') for line in lines[1:]]
run(exe,['demo',out/'demo']);before=(out/'demo/garden.tasks').read_bytes()
run(exe,['demo',out/'demo'],ok=False);assert before==(out/'demo/garden.tasks').read_bytes()
r=run(lab,[]);assert r.stdout.count('=true')==4
one=out/'before';two=out/'after';db=out/'my tasks 中文.tasks'
commands=f'add 我的任务\nadd 已完成任务\nx\nreport {one}\nprefix [今天]\nreport {two}\nundo\nsave\nwait\nq\n'
run(exe,['play',db],commands)
assert records(db)==[['1','open','我的任务'],['2','done','已完成任务']]
assert records(one/'snapshot.tasks')==records(db)
assert records(two/'snapshot.tasks')==[['1','open','[今天]我的任务'],['2','done','已完成任务']]
assert (one/'board.txt').read_text().startswith('revision=3\n')
assert (two/'board.txt').read_text().startswith('revision=4\n')
assert '1/2 completed' in (two/'summary.txt').read_text()
r=run(exe,['play',db],'q\n');assert '我的任务' in r.stdout
h=hashlib.sha256((one/'snapshot.tasks').read_bytes()).hexdigest()
r=run(exe,['play',db],f'report {one}\nq\n');assert '未执行' in r.stderr
assert hashlib.sha256((one/'snapshot.tasks').read_bytes()).hexdigest()==h
run(exe,['play',out/'not-saved.tasks'],'add 未保存\nq\n');assert not (out/'not-saved.tasks').exists()
r=run(exe,['play',db],'unknown\nq\n');assert '未执行' in r.stderr
(out/'RESULT.json').write_text(json.dumps(dict(status='PASS',calls=calls,independent_checks=['batch undo','read persisted tasks','old snapshot vs current data','background reports','no overwrite','no implicit save','weak/cycle runtime experiments'],source='maintainer reference implementation, not student work'),ensure_ascii=False,indent=2))
print('Advanced CLI PASS:',len(calls),'calls; persisted records and snapshots checked independently')
