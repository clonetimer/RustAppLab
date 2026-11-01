#!/usr/bin/env python3
"""真实本机 TCP/HTTP 联合验收；不依赖公网和第三方 Python 包。"""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
from urllib import request, error
ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--bin-dir',type=Path,default=ROOT/'target'/'release')
parser.add_argument('--output',type=Path)
args=parser.parse_args()
SUFFIX='.exe' if os.name=='nt' else ''
service=(args.bin_dir/f'servicedesk{SUFFIX}').resolve()
fetch=(args.bin_dir/f'netfetch{SUFFIX}').resolve()
assert service.is_file() and fetch.is_file(), '请先 cargo build --release -p servicedesk -p netfetch'
def cli(binary,*params,expected=0):
    result=subprocess.run([str(binary),*map(str,params)],capture_output=True,text=True,encoding="utf-8",timeout=15)
    assert result.returncode==expected,(binary,result.returncode,result.stdout,result.stderr)
    return result
with tempfile.TemporaryDirectory(prefix='applab-network-e2e-') as dirname:
    base=Path(dirname)
    output=args.output or base/'results'
    output.mkdir(parents=True,exist_ok=True)
    db=base/'真实 工单.log'
    cli(service,'init',db)
    with socket.socket() as sock:
        sock.bind(('127.0.0.1',0))
        port=sock.getsockname()[1]
    server=subprocess.Popen([str(service),'serve',str(db),str(port)],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
    try:
        root=f'http://127.0.0.1:{port}'
        for _ in range(100):
            try:
                with request.urlopen(root+'/api/tickets',timeout=.2) as res:
                    assert json.load(res)['count']==0
                break
            except (OSError,error.URLError):
                if server.poll() is not None:raise RuntimeError('服务启动提前退出')
                time.sleep(.04)
        else:raise RuntimeError('HTTP 服务启动超时')
        def post(path,content,allow=False):
            hdr={'Content-Type':'application/x-www-form-urlencoded','X-RustAppLab-Client':'1'} if allow else {'Content-Type':'application/x-www-form-urlencoded'}
            req=request.Request(root+path,content.encode('ascii'),headers=hdr,method='POST')
            return request.urlopen(req,timeout=2)
        try:
            post('/api/tickets','title=blocked')
            raise AssertionError('跨站/无标头写入竟然成功')
        except error.HTTPError as e: assert e.code==403
        with post('/api/tickets','title=%E8%87%AA%E5%B7%B1%E7%9A%84Rust%E5%B7%A5%E5%8D%95',True) as res:
            saved=json.load(res)
            assert saved['tickets'][0]['title']=='自己的Rust工单'
        with request.urlopen(root+'/api/tickets',timeout=2) as res:
            state=json.load(res)
            assert state['count']==1 and not state['tickets'][0]['done']
        downloaded=output/'从网络保存的工单.json'
        cli(fetch,'fetch',root+'/api/tickets',downloaded)
        assert json.loads(downloaded.read_text(encoding='utf-8'))==state
        assert cli(fetch,'fetch',root+'/api/tickets',downloaded,expected=2)
        with post('/api/close','id=1',True) as res:
            assert json.load(res)['tickets'][0]['done']
        # Fetch HTML and JSON into one published bundle.
        manifest=base/'jobs.tsv'
        manifest.write_text(f'index.html\t{root}/\ntickets.json\t{root}/api/tickets\n',encoding='utf-8')
        cli(fetch,'batch',manifest,output/'bundle')
        assert 'ServiceDesk' in (output/'bundle/index.html').read_text(encoding='utf-8')
        assert json.loads((output/'bundle/tickets.json').read_text(encoding='utf-8'))['tickets'][0]['done']
        try:
            request.urlopen(root+'/no-such-route',timeout=2)
            raise AssertionError('404 预期失败')
        except error.HTTPError as e: assert e.code==404
        assert cli(fetch,'fetch',root+'/missing',output/'should-not-exist',expected=2)
        assert not (output/'should-not-exist').exists()
    finally:
        server.terminate()
        try:server.wait(timeout=3)
        except subprocess.TimeoutExpired:server.kill();server.wait(timeout=3)
    persisted=json.loads(cli(service,'list',db).stdout)
    assert persisted['count']==1 and persisted['tickets'][0]['done']
    report={'status':'PASS','requests':'real TCP/HTTP 200/201/403/404, POST create/close, download, batch','count':1,'persisted_after_restart':True,'platform':os.name,'features':['loopback only','synchronous','no external crates']}
    (output/'NETWORK_E2E_EVIDENCE.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print('NETWORK END-TO-END PASS:',json.dumps(report,ensure_ascii=False))
