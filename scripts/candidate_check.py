#!/usr/bin/env python3
"""Build a native reference kit and test the exact extracted candidate artifacts.

This is maintainer tooling, not a new course or a claim of human acceptance.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import urllib.request
import zipfile
from release_check import ROOT, source_id

APPS = ('filequest','loglens','localhub','safesync','labquest','unitlab','quizdeck',
        'textscope','ledgerlite','badgeforge','batchmeter','termgarden','storedesk',
        'minikv','netfetch','servicedesk','asyncboard')
EXAMPLES = {'termgarden-workbench':'workbench','termgarden-ownership-lab':'ownership_lab'}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def runtime_files(target: Path, suffix: str) -> dict[str, Path]:
    files = {f'bin/{name}{suffix}': target/(name+suffix) for name in APPS}
    files.update({f'bin/{alias}{suffix}':target/'examples'/(name+suffix) for alias,name in EXAMPLES.items()})
    files.update({f'bin/{name}{suffix}':target/(name+suffix) for name in ('netfetch-c1','servicedesk-c2')})
    for path in files.values():
        if not path.is_file() or path.is_symlink() or path.stat().st_size == 0:
            raise ValueError(f'missing or invalid native build: {path}')
    return files


def pack_runtime(destination: Path, files: dict[str, Path], info: dict, notices: dict[str, bytes]) -> None:
    content = {name:path.read_bytes() for name,path in files.items()}
    content['LICENSE']=(ROOT/'LICENSE').read_bytes()
    content['BUILD_INFO.json']=(json.dumps(info,ensure_ascii=False,indent=2)+'\n').encode()
    content.update(notices)
    cmd='.\\bin\\' if info['system']=='Windows' else './bin/'
    exe='.exe' if info['system']=='Windows' else ''
    intro=f'''# Rust AppLab {info['version']} · 本机参考运行包

本包是 {info['system']} {info['machine']} 的实际构建，不是源码包或安装器。程序在 bin/。
参考程序可直接运行，LabQuest编译和检查学员源码仍需要安装Rust/Cargo和系统工具链。
Windows结果来自CI runner，不代表你的Win10/11已经验收。Linux需兼容的glibc及系统库，具体构建环境见BUILD_INFO.json；StoreDesk另需系统SQLite。
完整学习请另取同版本source.zip并打开START_HERE.html；B4/C1/C2依赖准备按各课讲义进行。

## 先体验

```text
{cmd}unitlab{exe} convert 25 c f
{cmd}netfetch-c1{exe} demo my-first-download
{cmd}servicedesk-c2{exe} init tickets.db
{cmd}servicedesk-c2{exe} serve tickets.db --port 3000
```

预期温度输出77.00 f；C1生成新下载目录；C2页面为http://127.0.0.1:3000/。
C2重新启动只执行serve，不重复init。已有目录/文件不会被覆盖。
Windows在PowerShell运行，不直接双击后把窗口关闭误认为失败。Linux如解压工具未保留权限，先执行chmod +x bin/*。
这是本机回环教学服务，不提供公网/TLS或账号认证。Ctrl+C请求优雅关闭，没有强制截止时间。

候选版不是Stable。首次使用、源码学习和真人记录见同版本source.zip的README_START_HERE.md及course/USER_TRIAL.md。
THIRD_PARTY保留构建所用锁定依赖的清单与授权文本；完整依赖源码随同一Release中的C1/C2离线源码包提供。
'''
    content['START_RUNTIME.md']=intro.encode()
    content['FILES_SHA256SUMS.txt']=''.join(f'{hashlib.sha256(data).hexdigest()}  {name}\n' for name,data in sorted(content.items())).encode()
    with zipfile.ZipFile(destination,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for name,data in sorted(content.items()):
            entry=zipfile.ZipInfo(name,date_time=(2026,11,4,0,0,0));entry.create_system=3
            entry.external_attr=(0o100755 if name.startswith('bin/') else 0o100644)<<16
            entry.compress_type=zipfile.ZIP_DEFLATED;z.writestr(entry,data)
    with zipfile.ZipFile(destination) as z:
        if z.testzip() is not None: raise ValueError('runtime ZIP CRC failed')


def main() -> int:
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
    logs=out/'logs';logs.mkdir();identity=source_id();version=(ROOT/'VERSION').read_text().strip()
    system=platform.system();machine=platform.machine().lower()
    if system not in ('Windows','Linux') or machine not in ('x86_64','amd64'):
        raise SystemExit('only actual Windows/Linux x86_64 builds are supported')
    suffix='.exe' if system=='Windows' else '';label='windows-x64-msvc' if system=='Windows' else 'linux-x86_64'
    target=ROOT/'target';env=dict(os.environ,CARGO_TARGET_DIR=str(target),CARGO_TERM_COLOR='never',PYTHONUTF8='1')
    env.pop('CARGO_BUILD_TARGET',None)
    steps=[]
    report={'schema':1,'version':version,'source_id':identity,'system':system,'platform':platform.platform(),'status':'RUNNING','steps':steps}

    def run(name: str, command: list, *, cwd: Path=ROOT, expected: int=0, timeout: int=900, custom_env: dict|None=None) -> str:
        path=logs/(name+'.txt')
        result=subprocess.run(list(map(str,command)),cwd=cwd,env=custom_env or env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=timeout)
        path.write_bytes(result.stdout)
        ok=result.returncode==expected
        steps.append({'step':name,'exit_code':result.returncode,'expected_exit_code':expected,'passed':ok,'log':'logs/'+path.name,'sha256':digest(path)})
        text=result.stdout.decode('utf-8',errors='replace')
        if not ok: raise RuntimeError(f'{name}: expected exit {expected}, got {result.returncode}\n{text[-8000:]}')
        print(name,'PASS',flush=True);return text

    try:
        compiler=run('compiler',['rustc','-Vv'])
        run('root-build',['cargo','build','--locked','--offline','--release','--workspace','--all-targets'])
        notices={}
        for course in ('c1','c2'):
            manifest=f'projects/advanced-{course}/reference/Cargo.toml'
            run(course+'-fetch',['cargo','+1.85.0','fetch','--locked','--manifest-path',manifest])
            run(course+'-build',['cargo','+1.85.0','build','--locked','--offline','--release','--manifest-path',manifest])
            metadata=json.loads(run(course+'-licenses',['cargo','+1.85.0','metadata','--locked','--offline','--format-version','1','--manifest-path',manifest]))
            notices[f'THIRD_PARTY/{course}-Cargo.lock']=(ROOT/manifest).with_name('Cargo.lock').read_bytes()
            for package in metadata['packages']:
                if not package.get('source'): continue
                base=Path(package['manifest_path']).parent;prefix=f"THIRD_PARTY/{package['name']}-{package['version']}/"
                notices[prefix+'Cargo.toml']=Path(package['manifest_path']).read_bytes()
                for path in base.iterdir():
                    if path.is_file() and path.name.upper().startswith(('LICENSE','COPYING','NOTICE','UNLICENSE','README')):
                        notices[prefix+path.name]=path.read_bytes()
                custom=package.get('license_file')
                if custom:
                    path=(base/custom).resolve()
                    if path.is_file() and base.resolve() in path.parents: notices[prefix+path.name]=path.read_bytes()
        run('source-package',[sys.executable,'scripts/package.py'])
        run('source-inventory',['git','diff','--exit-code','--','SOURCE_MANIFEST.sha256'])
        source=ROOT.parent/f'RustAppLab-{version}-source.zip';shutil.copy2(source,out/source.name)
        run('fresh-source',[sys.executable,'scripts/fresh_archive_check.py','--source-zip',source,'--output',out/'fresh-archive.json'])
        run('learner-kit',[sys.executable,'scripts/package_learner_kit.py'])
        kit=ROOT.parent/f'RustAppLab-{version}-learner-kit.zip';shutil.copy2(kit,out/kit.name)
        archive=out/f'RustAppLab-{version}-{label}-runtime.zip'
        info={'version':version,'source_id':identity,'commit':os.environ.get('GITHUB_SHA'),'system':system,'machine':machine,'platform':platform.platform(),'libc':platform.libc_ver(),'compiler':compiler,'course_compiler':'Rust 1.85.0','kind':'reference programs; learner assessment needs compiler'}
        pack_runtime(archive,runtime_files(target/'release',suffix),info,notices)
        with tempfile.TemporaryDirectory(prefix='applab-rc-handoff-') as temp:
            fresh=Path(temp)/'中文 runtime';fresh.mkdir()
            with zipfile.ZipFile(archive) as z:
                z.extractall(fresh)
                for line in z.read('FILES_SHA256SUMS.txt').decode().splitlines():
                    sha,name=line.split('  ',1)
                    if digest(fresh/name)!=sha: raise ValueError('runtime checksum mismatch: '+name)
            for path in (fresh/'bin').iterdir(): path.chmod(0o755)
            binary=lambda name:fresh/'bin'/(name+suffix)
            lab=binary('labquest');learner=fresh/'我的 学员'
            student_env=dict(env,CARGO_TARGET_DIR=str(Path(temp)/'student-target'))
            run('reference-conversion',[binary('unitlab'),'convert','25','c','f'],cwd=fresh)
            run('firstcard-create',[lab,'beginner','start',learner],cwd=fresh)
            original=(learner/'src/main.rs').read_bytes()
            run('firstcard-no-overwrite',[lab,'beginner','start',learner],cwd=fresh,expected=2)
            if original!=(learner/'src/main.rs').read_bytes(): raise ValueError('existing learner overwritten')
            run('firstcard-red',[lab,'beginner','check',learner],cwd=fresh,custom_env=student_env,expected=2)
            text=original.decode('utf-8');marker='等待完成第一张任务卡'
            if marker not in text: raise ValueError('starter contract changed')
            (learner/'src/main.rs').write_text(text.replace(marker,'今日任务卡 | {name} | {task}'),encoding='utf-8',newline='\n')
            run('firstcard-green',[lab,'beginner','check',learner],cwd=fresh,custom_env=student_env)
            output=run('firstcard-personal-output',['cargo','run','--offline','--manifest-path',learner/'Cargo.toml','--','小林','完成试用'],cwd=fresh,custom_env=student_env)
            if '小林' not in output or '完成试用' not in output: raise ValueError('personal output missing')
            before={str(p.relative_to(learner)):digest(p) for p in learner.rglob('*') if p.is_file()}
            run('firstcard-readonly-next',[lab,'learn','next',learner],cwd=fresh,custom_env=student_env)
            after={str(p.relative_to(learner)):digest(p) for p in learner.rglob('*') if p.is_file()}
            if before!=after: raise ValueError('navigation changed learner files')
            run('c1-real-demo',[binary('netfetch-c1'),'demo',fresh/'downloads'],cwd=fresh)
            db=fresh/'tickets.db';run('c2-init',[binary('servicedesk-c2'),'init',db],cwd=fresh)
            opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
            for cycle in range(2):
                logfile=logs/f'c2-runtime-server-{cycle}.txt'
                with logfile.open('wb') as stream:
                    process=subprocess.Popen([str(binary('servicedesk-c2')),'serve',str(db),'--port','0'],cwd=fresh,env=env,stdout=stream,stderr=subprocess.STDOUT)
                    try:
                        deadline=time.monotonic()+20;base=None
                        while time.monotonic()<deadline:
                            if process.poll() is not None: raise ValueError('C2 server stopped before readiness')
                            for line in logfile.read_text(encoding='utf-8',errors='replace').splitlines():
                                if line.startswith('LISTENING '):base=line.split(' ',1)[1].rstrip('/')
                            if base:break
                            time.sleep(.1)
                        if base is None:raise TimeoutError('C2 listen address missing')
                        def request(path: str, payload=None):
                            data=None if payload is None else json.dumps(payload).encode()
                            req=urllib.request.Request(base+path,data=data,headers={'X-RustAppLab-Client':'1','Content-Type':'application/json'})
                            with opener.open(req,timeout=10) as response:return json.load(response)
                        if request('/health')['ok'] is not True:raise ValueError('C2 unhealthy')
                        if cycle==0:
                            item=request('/api/tickets',{'title':'候选版实测'});request(f"/api/tickets/{item['id']}/close",{})
                        tickets=request('/api/tickets')
                        if tickets['count']!=1 or tickets['tickets'][0]['done'] is not True:raise ValueError('C2 persisted state missing')
                    finally:
                        if process.poll() is None:
                            process.send_signal(signal.SIGINT) if system=='Linux' else process.terminate()
                        try:process.wait(timeout=10)
                        except subprocess.TimeoutExpired:process.kill();process.wait();raise
                if system=='Linux' and process.returncode!=0:raise ValueError('C2 SIGINT shutdown failed')
                steps.append({'step':f'c2-runtime-restart-{cycle}','passed':True,'log':'logs/'+logfile.name,'sha256':digest(logfile),'stop_method':'SIGINT' if system=='Linux' else 'terminate after committed HTTP response'})
        report['artifacts']=[{'name':p.name,'sha256':digest(p),'bytes':p.stat().st_size} for p in sorted(out.glob('*.zip'))]
        report['source_id_after']=source_id();report['status']='PASS' if report['source_id_after']==identity else 'FAIL'
    except Exception as error:
        report['status']='FAIL';report['error']=f'{type(error).__name__}: {error}';report['source_id_after']=source_id();print(report['error'],flush=True)
    (out/'candidate.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    return 0 if report['status']=='PASS' else 1

if __name__=='__main__':raise SystemExit(main())
