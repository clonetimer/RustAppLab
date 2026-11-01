#!/usr/bin/env python3
"""Maintainer-only: independent CLI/file-format verification; no third-party Python packages."""
from __future__ import annotations
import argparse, json, os, platform, subprocess, tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def parse_journal(path:Path)->dict[str,str]:
    data=path.read_bytes()
    assert data.startswith(b'RUSTAPPLAB_MINIKV_V1\n') and data.endswith(b'\n')
    state={}
    for line in data.splitlines()[1:]:
        fields=line.decode('ascii').split('\t')
        key=bytes.fromhex(fields[1]).decode('utf-8')
        if fields[0]=='P':
            assert len(fields)==3
            state[key]=bytes.fromhex(fields[2]).decode('utf-8')
        else:
            assert fields[0]=='D' and len(fields)==2 and key in state
            del state[key]
    return state

def run(binary:Path,out:Path)->dict:
    out.mkdir(parents=True,exist_ok=False)
    calls=[]
    def invoke(*args:object,ok:bool=True):
        result=subprocess.run([str(binary),*map(str,args)],capture_output=True,text=True,encoding='utf-8',timeout=20)
        calls.append({'args':list(map(str,args)),'exit':result.returncode,'stdout':result.stdout,'stderr':result.stderr})
        assert (result.returncode==0)==ok, calls[-1]
        return result.stdout
    db=out/'我的 书签.kv'
    invoke('put',db,'book/rust','中文笔记')
    invoke('put',db,'project/next','实现自己的 Iterator')
    assert 'replaced=true' in invoke('put',db,'book/rust','已经理解借用')
    assert invoke('get',db,'book/rust').strip()=='已经理解借用'
    invoke('put',db,'scratch/remove','临时')
    invoke('delete',db,'scratch/remove')
    before=db.read_bytes()
    invoke('delete',db,'absent',ok=False)
    invoke('put',db,' bad','not saved',ok=False)
    assert db.read_bytes()==before
    assert invoke('scan',db,'book/')=='book/rust\t已经理解借用\n'
    report=out/'my-product.tsv'
    invoke('export',db,report)
    saved=report.read_bytes()
    invoke('export',db,report,ok=False)
    assert report.read_bytes()==saved
    expected={'book/rust':'已经理解借用','project/next':'实现自己的 Iterator'}
    assert parse_journal(db)==expected
    absent=out/'missing.kv';invoke('get',absent,'any',ok=False);assert not absent.exists()
    shared=out/'four-workers.kv'
    assert 'records=40' in invoke('share-demo',shared)
    model=parse_journal(shared)
    assert len(model)==40 and set(model)=={f'worker/{w}/{i:02}' for w in range(4) for i in range(10)}
    bad=out/'truncated.kv';bad.write_bytes(db.read_bytes()[:-1]);before=bad.read_bytes()
    invoke('scan',bad,ok=False);assert bad.read_bytes()==before
    assert not list(out.glob('*.lock'))
    result={'version':'0.14.0','platform':platform.platform(),'binary':str(binary),'cli_calls':len(calls),'independent_journal_check':True,'records':expected,'shared_records':len(model),'calls':calls}
    (out/'EVIDENCE.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
    (out/'README.md').write_text('# MiniKV 真实运行产物\n\n这里是参考程序的验证输出，不是学员答案。我的 书签.kv 可用 minikv 读取；my-product.tsv 是导出。four-workers.kv 是四线程实际写入结果。truncated.kv 是刻意构造的损坏反例，请勿当正常数据库使用。EVIDENCE.json 是逐条调用与退出码；Python 独立解析日志验证最终状态。\n',encoding='utf-8')
    return result
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=Path,default=ROOT/'target/release'/('minikv.exe' if os.name=='nt' else 'minikv'));parser.add_argument('--output',type=Path)
    args=parser.parse_args()
    if args.output:
        report=run(args.binary.resolve(),args.output.resolve())
    else:
        with tempfile.TemporaryDirectory(prefix='minikv-smoke-') as folder:
            report=run(args.binary.resolve(),Path(folder)/'outputs')
    print(json.dumps({k:report[k] for k in ['version','platform','cli_calls','independent_journal_check','shared_records']},ensure_ascii=False,indent=2))
