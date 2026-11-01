#!/usr/bin/env python3
"""使用真正的 release 可执行程序和字节文件，检查 AsyncBoard 的完整 CLI。"""
from __future__ import annotations
import os
import subprocess
import tempfile
import time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
EXE=ROOT/'target/release'/('asyncboard.exe' if os.name=='nt' else 'asyncboard')
if not EXE.is_file():
    raise SystemExit('先运行 cargo build --release -p asyncboard')
checks=[]
def call(*args:str, expect_success=True):
    p=subprocess.run([str(EXE),*map(str,args)],capture_output=True,text=True,encoding="utf-8",timeout=8)
    assert p.returncode == (0 if expect_success else 2),(args,p.stdout,p.stderr)
    checks.append((list(args),p.returncode))
    return p
with tempfile.TemporaryDirectory(prefix='asyncboard-smoke-') as temp:
    root=Path(temp)
    demo=root/'中文 空格 作品'
    start=time.monotonic()
    call('demo',demo)
    elapsed=time.monotonic()-start
    assert elapsed >= 0.085,elapsed
    original=(demo/'RESULTS.tsv').read_text(encoding='utf-8')
    lines=original.strip().splitlines()
    assert len(lines)==4,lines
    assert [line.split('\t')[0] for line in lines[1:]] == ['write','review','read'],lines
    assert (demo/'LEARNING_NOTE.md').is_file()
    call('preview',demo/'jobs.tsv')
    call('demo',demo,expect_success=False)
    assert (demo/'RESULTS.tsv').read_text(encoding='utf-8')==original
    out=demo/'second.tsv'
    call('run',demo/'jobs.tsv',out)
    assert out.read_text(encoding='utf-8')==original
    call('run',demo/'jobs.tsv',out,expect_success=False)
    assert out.read_text(encoding='utf-8')==original
    broken=demo/'broken.tsv'
    broken.write_text('x\t-1\tbad\n',encoding='utf-8')
    failed=demo/'failed.tsv'
    call('run',broken,failed,expect_success=False)
    assert not failed.exists()
print('ASYNCBOARD REAL RELEASE SMOKE PASS:', len(checks),'commands; timed overlap / Unicode paths / no-overwrite / malformed input')
