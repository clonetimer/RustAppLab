#!/usr/bin/env python3
"""Execute the six shipped applications with real input/output. Standard Python only."""
from __future__ import annotations
import argparse, json, os, platform, subprocess, tempfile, xml.etree.ElementTree as ET
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
SUFFIX = '.exe' if os.name == 'nt' else ''
BIN = Path(os.environ.get('CARGO_TARGET_DIR',str(ROOT/'target'))) / 'release'
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output',type=Path)
options=parser.parse_args()
records=[]
def run(app, *args, stdin=None, success=True):
    cmd=[str(BIN/(app+SUFFIX)),*map(str,args)]
    result=subprocess.run(cmd,input=stdin,capture_output=True,text=True,encoding='utf-8',timeout=10)
    assert (result.returncode==0)==success,(app,args,result.returncode,result.stderr)
    records.append({'application':app,'arguments':list(map(str,args)),'exit_code':result.returncode,'stdout':result.stdout,'stderr':result.stderr})
    return result.stdout
with tempfile.TemporaryDirectory(prefix='rustapplab-studio-release-') as tmp:
    t=Path(tmp)
    assert run('unitlab','convert','25','c','f').strip()=='25.00 c = 77.00 f'
    run('unitlab','convert','1','m','f',success=False)
    deck=t/'中文 题库.tsv';deck.write_text('工具\tcargo\r\n2+3\t5\r\n',encoding='utf-8')
    assert 'score=1/2' in run('quizdeck','play',deck,stdin='Cargo\n4\n')
    notes=t/'我的 文本.txt';notes.write_text('Rust 学习\nRust tools\n',encoding='utf-8')
    assert run('textscope','top',notes,'1').strip()=='rust\t2'
    assert run('textscope','find',notes,'Rust')=='1:Rust 学习\n2:Rust tools\n'
    book=t/'生活 账本.tsv'
    for amount in ['0.10','0.20']:
        run('ledgerlite','add',book,'2026-10-28','餐饮',amount)
    assert 'total=0.30' in run('ledgerlite','summary',book)
    before=book.read_bytes();run('ledgerlite','add',book,'2023-02-29','餐饮','99',success=False);assert book.read_bytes()==before
    svg=t/'我的作品.svg';run('badgeforge','make','我的 Rust 作品','从第一行代码，到自己的软件',svg)
    node=ET.fromstring(svg.read_text(encoding='utf-8'));assert node.tag=='{http://www.w3.org/2000/svg}svg'
    assert any(e.text=='我的 Rust 作品' for e in node.iter())
    original=svg.read_bytes();run('badgeforge','make','不要覆盖','测试',svg,success=False);assert svg.read_bytes()==original
    paths=[]
    for i in range(32):
        path=t/f'文本 {i}.txt';path.write_text(f'第{i}份文件\n第二行\n',encoding='utf-8');paths.append(path)
    sequential=run('batchmeter','scan','1',*paths)
    parallel=run('batchmeter','scan','4',*paths)
    assert sequential==parallel and 'files=32' in parallel
    run('batchmeter','scan','2',t/'not-there.txt',success=False)
output={'platform':platform.platform(),'status':'passed','applications':6,'cli_invocations':len(records),'checks':records,'scope':'Real processes/files; not a GUI or Windows-native certification unless executed on that platform.'}
if options.output:
    options.output.write_text(json.dumps(output,ensure_ascii=False,indent=2),encoding='utf-8')
print(f'Release smoke PASS: 6 applications / {len(records)} CLI invocations, exact decimal persistence, SVG XML parse, sequential-parallel equivalence.')
