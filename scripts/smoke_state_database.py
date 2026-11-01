#!/usr/bin/env python3
"""Independent, standard-library-only CLI/storage checks. Writes solely inside a new output directory.
Usage: python scripts/smoke_state_database.py [--bin-dir PATH] [--output NEW_DIRECTORY]
Python's sqlite3 independently reads the database written by the Rust program; not a separate engine audit.
"""
from __future__ import annotations
import argparse, json, platform, sqlite3, subprocess, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bin-dir', type=Path, default=ROOT/'target'/'release')
parser.add_argument('--output',type=Path)
a=parser.parse_args()
folder=a.output
if folder is None:
    folder=Path(tempfile.mkdtemp(prefix='rustapplab-013-evidence-'))
else:
    folder.mkdir(parents=True,exist_ok=False)
records=[]
ext='.exe' if platform.system()=='Windows' else ''
def run(app,*args,expect=0,stdin=None):
    cmd=[str(a.bin_dir.resolve()/(app+ext)),*[str(arg) for arg in args]]
    out=subprocess.run(cmd,input=stdin,text=True,encoding='utf-8',capture_output=True,timeout=20)
    records.append({'command':cmd,'returncode':out.returncode,'stdout':out.stdout,'stderr':out.stderr,'expected':expect})
    if out.returncode!=expect:
        raise AssertionError(f'{cmd}: expected {expect}, got {out.returncode}\n{out.stderr}')
    return out
engine=run('storedesk','doctor').stdout.strip()
store=folder/'StoreDesk 中文示例'
run('storedesk','demo',store)
dbpath=store/'inventory.db'
assert dbpath.read_bytes().startswith(b'SQLite format 3\0')
def read():
    with sqlite3.connect(str(dbpath)) as conn:
        return conn.execute('SELECT sku,name,stock FROM items ORDER BY sku').fetchall(),conn.execute('SELECT sku,delta FROM movements ORDER BY id').fetchall()
items,audit=read()
assert [(i[0],i[2]) for i in items]==[('BOOK',4),('PEN',8)]
assert audit==[('PEN',-2),('BOOK',1)]
bad=store/'reject-batch.tsv';bad.write_text('PEN\t-1\nBOOK\t-999\n',encoding='utf-8')
run('storedesk','apply',dbpath,bad,expect=2)
assert read()==(items,audit), 'failed second update must roll back first update and first audit'
name="x'); DROP TABLE items; --"
run('storedesk','add',dbpath,'SAFE',name,2)
assert read()[0][-1]==('SAFE',name,2)
run('storedesk','add',dbpath,'SAFE','not a replacement',99,expect=2)
assert read()[0][-1]==('SAFE',name,2)
run('storedesk','adjust',dbpath,'PEN',-100,expect=2)
assert next(i[2] for i in read()[0] if i[0]=='PEN')==8
run('storedesk','init',dbpath,expect=2)
missing=store/'mistyped.db';run('storedesk','list',missing,expect=2);assert not missing.exists()
run('storedesk','list',dbpath)
run('storedesk','history',dbpath)
with sqlite3.connect(str(dbpath)) as conn:
    quick=conn.execute('PRAGMA integrity_check').fetchall()
    assert quick==[('ok',)]
    foreign=conn.execute('PRAGMA foreign_key_check').fetchall();assert not foreign
    schema=conn.execute('PRAGMA user_version').fetchone()[0]; assert schema==1
    application=conn.execute('PRAGMA application_id').fetchone()[0]; assert application==0x5354444b
    conn.execute("UPDATE items SET stock=17 WHERE sku='PEN'")
# Rust reads a change committed by an independent Python connection, not a cached value.
out=run('storedesk','list',dbpath);assert 'PEN\t学习用笔\t17' in out.stdout
term=folder/'TermGarden 中文示例';run('termgarden','demo',term)
state=term/'garden.tasks';text=state.read_text(encoding='utf-8')
assert text.count('\tdone\t')==2 and text.startswith('TERMGARDEN\t1\n')
badscript=term/'bad-session.txt';badscript.write_text('add MUST NOT BE SAVED\nnot-a-command\n',encoding='utf-8')
before=state.read_bytes();run('termgarden','replay',state,badscript,expect=2);assert state.read_bytes()==before
interactive=term/'自己的任务.tasks'
run('termgarden','play',interactive,'compact',stdin='add 真实终端输入\nx\nq\n')
assert '1\tdone\t真实终端输入' in interactive.read_text(encoding='utf-8')
run('termgarden','play',interactive,stdin='q\n')
# This report does not imply raw-terminal, GUI or native Windows validation.
summary={'platform':platform.platform(),'native_backend':engine,'python_sqlite':sqlite3.sqlite_version,'cli_calls':len(records),'sqlite_header':'verified','schema_version':schema,'application_id':application,'failed_batch_rollback':'inventory and audit unchanged','parameter_binding':'SQL-shaped name stored literally','independent_change_visible_to_rust':True,'integrity_check':quick,'foreign_key_check':foreign,'terminal_script_rollback':True,'terminal_persistence':True,'records':records}
report=folder/'STATE_DATABASE_SMOKE.json';report.write_text(json.dumps(summary,ensure_ascii=False,indent=2),encoding='utf-8')
(folder/'READ_ME.txt').write_text('Rust AppLab 0.13.0 实测演示产物。\nStoreDesk: 初始BOOK=4、PEN=8；测试还增加SAFE名称，再由独立Python连接把PEN改为17，最终库存应为BOOK=4、PEN=17、SAFE=2；原演示INVENTORY_REPORT是初始快照。\nTermGarden: garden.tasks由参考demo生成；自己的任务.tasks由真实stdin交互生成。bad文件为有意的拒绝场景。\n所有数据为教学样例，不是真实用户数据。STATE_DATABASE_SMOKE.json记录每次调用。\n',encoding='utf-8')
print(f'PASS: {len(records)} CLI calls; SQLite independent read/write, rollback and terminal persistence verified.')
print(report)
