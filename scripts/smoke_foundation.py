#!/usr/bin/env python3
"""Run the foundation lesson's published commands. Not a human-learning study."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bin-dir', type=Path, default=ROOT/'target/release')
    parser.add_argument('--output', type=Path, required=True, help='New JSON file; never overwrite')
    args = parser.parse_args()
    data = ROOT/'examples/foundation'
    notes = (data/'my-notes.txt').read_text(encoding='utf-8')
    tokens = re.findall(r'[^\W_]+', notes, re.UNICODE)
    expected_stats = f'bytes={len(notes.encode("utf-8"))} chars={len(notes)} lines={len(notes.splitlines())} words={len(tokens)}'
    results = []
    def run(app, argv, expected_code=0, expected='', stdin=None):
        exe = args.bin_dir/(app + ('.exe' if os.name == 'nt' else ''))
        proc = subprocess.run([str(exe), *map(str,argv)], input=stdin, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, text=True, encoding='utf-8', timeout=15)
        ok = proc.returncode == expected_code and (not expected or expected in proc.stdout)
        item = dict(application=app, argv=list(map(str,argv)), exit_code=proc.returncode,
                    expected_exit=expected_code, stdout=proc.stdout, stderr=proc.stderr, passed=ok)
        results.append(item)
        if not ok: raise AssertionError(json.dumps(item,ensure_ascii=False))
    try:
        run('unitlab',['convert','-10','c','f'],expected='-10.00 c = 14.00 f')
        run('unitlab',['convert','1500','m','km'],expected='1500.00 m = 1.50 km')
        run('unitlab',['convert','25','c','m'],expected_code=2)
        run('unitlab',['convert','abc','c','f'],expected_code=2)
        run('quizdeck',['check',data/'my-deck.tsv',data/'my-answers.txt'],expected='score=2/3\n复习：Rust 构建工具')
        run('quizdeck',['play',data/'my-deck.tsv'],stdin='5\ncargo\n北京\n',expected='score=3/3')
        run('textscope',['stats',data/'my-notes.txt'],expected=expected_stats)
        run('textscope',['top',data/'my-notes.txt','5'],expected='rust\t2')
        run('textscope',['find',data/'my-notes.txt','Rust'],expected='1:Rust 学习\n2:今天用 Rust 写了 QuizDeck。')
        run('textscope',['find',data/'my-notes.txt',''],expected_code=2)
        status = 'PASS'
    except (AssertionError, OSError, subprocess.TimeoutExpired) as exc:
        status='FAIL'; results.append(dict(error=str(exc),passed=False))
    record = dict(status=status, scope='Automated reference-binary lesson command rehearsal; NOT learner outcome evidence',
                  cases=len(results), results=results)
    with args.output.open('x',encoding='utf-8') as f: json.dump(record,f,ensure_ascii=False,indent=2)
    print(f'{status}: {len(results)} actual CLI commands; evidence: {args.output}')
    return 0 if status=='PASS' else 1
if __name__=='__main__': sys.exit(main())
