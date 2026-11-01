#!/usr/bin/env python3
"""Maintainer rehearsal with real binaries and independent sqlite3 reads, not learner evidence."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]

def snapshot(db: Path) -> dict:
    # Read-only: a misspelled path cannot silently create an empty database.
    con = sqlite3.connect(db.resolve().as_uri() + '?mode=ro', uri=True)
    try:
        return {
            'items': [list(row) for row in con.execute('SELECT sku,name,stock FROM items ORDER BY sku')],
            'movements': [list(row) for row in con.execute('SELECT id,sku,delta FROM movements ORDER BY id')],
        }
    finally:
        con.close()

def assert_rollback(before: dict, after: dict) -> None:
    assert before == after, 'A failed batch changed logical stock or audit history'

def inspect_report(path: Path, expected: dict[str, tuple[int, str]]) -> list[str]:
    rows = path.read_text(encoding='utf-8').splitlines()
    assert rows and rows[0] == 'id\twait_ms\tnote', 'Bad TSV header'
    seen: dict[str, tuple[int, str]] = {}
    order = []
    for row in rows[1:]:
        fields = row.split('\t')
        assert len(fields) == 3 and fields[0] not in seen, 'Missing/duplicate/corrupt completion'
        seen[fields[0]] = (int(fields[1]), fields[2])
        order.append(fields[0])
    assert seen == expected, 'Completion identity or content mismatch'
    # Equal-delay completion order intentionally not asserted; wait_ms is configuration.
    return order

def run(bin_dir: Path, output: Path) -> dict:
    programs = {p: bin_dir.resolve()/(p + ('.exe' if os.name == 'nt' else '')) for p in ('storedesk','asyncboard')}
    for p in programs.values():
        if not p.is_file(): raise FileNotFoundError(p)
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    for p in (ROOT/'examples/reliability').iterdir():
        if p.is_file(): shutil.copy2(p, output/p.name)
    steps = []
    def call(program: str, *args, fail: bool = False):
        cmd = [str(programs[program]), *map(str,args)]
        now = time.monotonic()
        p = subprocess.run(cmd, capture_output=True, text=True, encoding='utf-8', timeout=30)
        elapsed = time.monotonic()-now
        n = len(steps)+1
        (output/f'{n:02d}-{program}.stdout.txt').write_text(p.stdout,encoding='utf-8')
        (output/f'{n:02d}-{program}.stderr.txt').write_text(p.stderr,encoding='utf-8')
        steps.append(dict(command=cmd, exit_code=p.returncode, expected_exit=2 if fail else 0, elapsed_seconds=elapsed))
        assert p.returncode == (2 if fail else 0), (cmd,p.stdout,p.stderr)
        return p
    db = output/'inventory.db'
    call('storedesk','doctor')
    call('storedesk','init',db)
    call('storedesk','add',db,'PEN',"Alice's 钢笔",10)
    call('storedesk','add',db,'BOOK','我的 Rust 练习本',3)
    call('storedesk','list',db)
    call('storedesk','history',db)
    before = snapshot(db)
    assert [r[2] for r in before['items']] == [3,10] and before['movements'] == []
    call('storedesk','apply',db,output/'rollback.tsv',fail=True)
    after = snapshot(db)
    assert_rollback(before,after)
    call('storedesk','list',db)
    call('storedesk','history',db)
    call('storedesk','apply',db,output/'success.tsv')
    success = snapshot(db)
    assert [r[2] for r in success['items']] == [4,8]
    assert [r[1:] for r in success['movements']] == [['PEN',-2],['BOOK',1]]
    call('storedesk','apply',db,output/'repeated-sku.tsv')
    final = snapshot(db)
    assert [r[2] for r in final['items']] == [4,5]
    assert [r[1:] for r in final['movements']] == [['PEN',-2],['BOOK',1],['PEN',-1],['PEN',-2]]
    call('storedesk','add',db,'PEN','不应覆盖',99,fail=True)
    assert_rollback(final,snapshot(db))
    call('storedesk','list',db)
    call('storedesk','history',db)
    call('asyncboard','preview',output/'jobs.tsv')
    report = output/'RESULTS.tsv'
    call('asyncboard','run',output/'jobs.tsv',report)
    expected = {'read':(180,'复习事务闭包'),'write':(30,'记录失败的原因'),'review':(80,'解释 Pending 与 Waker')}
    order = inspect_report(report,expected)
    digest = hashlib.sha256(report.read_bytes()).hexdigest()
    call('asyncboard','run',output/'jobs.tsv',report,fail=True)
    assert hashlib.sha256(report.read_bytes()).hexdigest() == digest
    equal = output/'EQUAL.tsv'
    call('asyncboard','run',output/'equal-delay.tsv',equal)
    equal_order = inspect_report(equal,{'first':(40,'同延时甲'),'second':(40,'同延时乙')})
    bad = output/'SHOULD_NOT_EXIST.tsv'
    call('asyncboard','run',output/'duplicate-jobs.tsv',bad,fail=True)
    assert not bad.exists()
    result = dict(status='PASS', kind='reference_application_rehearsal_not_student_observation',
        commands=steps, command_count=len(steps), initial=before, rollback=after, success=success, final=final,
        report_order_observed=order, equal_delay_order_observed=equal_order,
        sqlite_readback='Independent Python sqlite3, read-only connection',
        scope='No cross-application transaction; no power-loss or Windows-native claim')
    (output/'REHEARSAL.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
    return result

def main() -> None:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bin-dir',type=Path,default=ROOT/'target/release')
    parser.add_argument('--out',type=Path,required=True,help='new, non-existing directory')
    args=parser.parse_args()
    result=run(args.bin_dir,args.out)
    print(f"RELIABILITY REHEARSAL PASS: {result['command_count']} real CLI calls; independent SQLite readback; report integrity")
if __name__=='__main__': main()
