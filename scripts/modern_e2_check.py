#!/usr/bin/env python3
"""E2 native GUI, actual AccessKit button contracts and captured X11 interactions."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import time
import tomllib
import zipfile

from modern_d8_check import CARGO, RUSTC, ROOT, digest, write
from modern_d11b_check import Runner, install, require, DB_SHA, AUDIT_VERSION

LAB = ROOT / "projects/modern-e2-gui"
CONFIG = '[source.crates-io]\nreplace-with = "e2-vendor"\n[source.e2-vendor]\ndirectory = "vendor"\n'

def snapshot():
    paths=[p for p in LAB.rglob("*") if p.is_file() and
           not {"target",".git","__pycache__","vendor"}.intersection(p.relative_to(LAB).parts)]
    paths.extend([ROOT/"course/BOOK_E2_GUI.md",ROOT/"scripts/modern_d11b_check.py",
                  ROOT/"scripts/modern_d8_check.py",Path(__file__).resolve(),ROOT/"LICENSE"])
    return {p.relative_to(ROOT).as_posix():digest(p) for p in sorted(set(paths))}

def stage(side,path):
    shutil.copytree(LAB/side,path,ignore=shutil.ignore_patterns("target",".git","__pycache__","Cargo.lock","vendor"))
    shutil.copyfile(ROOT/"LICENSE",path/"LICENSE")

def lock_count(lock):
    packages=tomllib.loads(lock.read_text(encoding="utf-8"))["package"]
    for name in ("eframe","egui","egui_kittest"):
        require([p["version"] for p in packages if p["name"]==name]==["0.37.0"],"GUI pin mismatch "+name)
    registry=[p for p in packages if p.get("source")]
    require(all(p.get("checksum") for p in registry),"missing registry checksum")
    return len(registry)

def resolve(e,scratch,report):
    work=scratch/"resolved";stage("reference",work)
    lock=work/"Cargo.lock"
    e.env["CARGO_NET_OFFLINE"]="false"
    committed=LAB/"reference/Cargo.lock"
    require(digest(committed)==digest(LAB/"learner/Cargo.lock"),"committed locks differ")
    require(digest(committed)=="0c547514c781df6d8571cb8dac307c66d671e95d22734e1f3bd098b80b81b210","frozen E2 source lock differs from original audited artifact")
    shutil.copyfile(committed,lock)
    e.execute("fetch-committed",CARGO+["fetch","--locked"],work,timeout=1200)
    require(digest(lock)==digest(committed),"locked fetch changed frozen lock")
    count=lock_count(lock)
    saved=e.output/"Cargo.lock";shutil.copyfile(lock,saved)
    lock_hash=digest(saved);write(e.output/"Cargo.lock.sha256",lock_hash+"  Cargo.lock\n")
    install(e,scratch,"cargo-audit",AUDIT_VERSION)
    db=scratch/"advisory-db";db.mkdir()
    e.execute("db-init",["git","init",db],scratch)
    e.execute("db-fetch",["git","-C",db,"fetch","--depth=1","https://github.com/RustSec/advisory-db.git",DB_SHA],scratch,timeout=600)
    e.execute("db-checkout",["git","-C",db,"checkout","--detach","FETCH_HEAD"],scratch)
    rev,_,_=e.execute("db-revision",["git","-C",db,"rev-parse","HEAD"],scratch)
    require(rev.strip()==DB_SHA,"advisory DB mismatch")
    e.env["CARGO_NET_OFFLINE"]="true"
    out,_,code=e.execute("audit-gui-lock",CARGO+["audit","--no-fetch","--stale","--no-yanked","--db",db,"--file",saved,"--json"],
                         scratch,expected=(0,1),timeout=300)
    data=json.loads(out);write(e.output/"gui-audit.json",json.dumps(data,indent=2,ensure_ascii=False)+"\n")
    vulnerabilities=data["vulnerabilities"]["list"];unsound=data.get("warnings",{}).get("unsound",[])
    report["lock"]=dict(sha256=lock_hash,registry_packages=count,advisory_db=DB_SHA,
                        vulnerabilities=len(vulnerabilities),unsound=len(unsound),
                        warnings=data.get("warnings",{}),yanked_check=False)
    require(code==0 and not vulnerabilities and not unsound,"GUI dependency gate rejected")
    e.env["CARGO_NET_OFFLINE"]="false"
    vendor=scratch/"vendor"
    e.execute("vendor-locked",CARGO+["vendor","--locked","--versioned-dirs","--manifest-path",work/"Cargo.toml",vendor],
              scratch,timeout=1200)
    require(digest(lock)==lock_hash,"vendor altered lock")
    learner=scratch/"learner";stage("learner",learner);shutil.copyfile(saved,learner/"Cargo.lock")
    shutil.move(str(vendor),str(learner/"vendor"));write(learner/".cargo/config.toml",CONFIG)
    archive=e.output/"E2-learner-offline.zip"
    with zipfile.ZipFile(archive,"w",zipfile.ZIP_DEFLATED,compresslevel=6) as z:
        for p in sorted(learner.rglob("*")):
            if p.is_file():
                rel=p.relative_to(learner).as_posix()
                info=zipfile.ZipInfo(rel,(2026,11,26,0,0,0));info.create_system=3;info.external_attr=0o100644<<16
                z.writestr(info,p.read_bytes(),compress_type=zipfile.ZIP_DEFLATED,compresslevel=6)
    with zipfile.ZipFile(archive) as z:
        require(z.testzip() is None,"bad learner ZIP")
        require(z.read("src/lib.rs")== (LAB/"learner/src/lib.rs").read_bytes(),"lib answer leaked")
        require(z.read("src/gui.rs")== (LAB/"learner/src/gui.rs").read_bytes(),"GUI answer leaked")
        require(not any(n.startswith(("reference/","target/")) for n in z.namelist()),"unexpected build/reference files")
    write(archive.with_suffix(".zip.sha256"),digest(archive)+"  "+archive.name+"\n")
    report["archive"]=dict(name=archive.name,sha256=digest(archive),bytes=archive.stat().st_size)

def check_vendor(e,work,home,label):
    out,_,_=e.execute(label+"-metadata",CARGO+["metadata","--frozen","--format-version","1"],work,timeout=300)
    packages=[p for p in json.loads(out)["packages"] if p.get("source")]
    vendor=(work/"vendor").resolve()
    require(packages and all(Path(p["manifest_path"]).resolve().is_relative_to(vendor) for p in packages),"registry escaped vendor")
    entries=[]
    if (home/"registry").exists():
        for p in (home/"registry").rglob("*"):
            if p.is_file():
                name=p.relative_to(home/"registry").as_posix();entries.append(name)
                require(name=="CACHEDIR.TAG" and p.read_bytes().startswith(b"Signature: 8a477f597d28d172789f06886806bc55"),"registry populated: "+name)
    write(e.output/(label+"-vendor.json"),json.dumps(dict(packages=len(packages),registry_files=entries),indent=2)+"\n")

def native(e,work,exe,report):
    # This function executes inside the actual Xvfb display, not a screenshot mock.
    env=e.env.copy();env["LIBGL_ALWAYS_SOFTWARE"]="1"
    # This smoke path explicitly targets the actual X11 server supplied by Xvfb.
    env.pop("WAYLAND_DISPLAY",None)
    env["WINIT_UNIX_BACKEND"]="x11"
    log=e.output/"native-window.txt"
    def x(*args,timeout=10):
        p=subprocess.run(["xdotool",*map(str,args)],env=env,capture_output=True,text=True,timeout=timeout)
        if p.returncode: raise RuntimeError("xdotool failed: "+p.stderr)
        return p.stdout.strip()
    def wait_window(expected):
        until=time.monotonic()+25
        last=""
        while time.monotonic()<until:
            if app.poll() is not None: raise RuntimeError("native app exited before interaction")
            try:
                for wid in x("search","--onlyvisible","--name","^SignalDesk").splitlines():
                    last=x("getwindowname",wid)
                    if expected in last:return wid,last
            except (RuntimeError,subprocess.TimeoutExpired):pass
            time.sleep(.15)
        raise RuntimeError("native title did not reach "+expected+"; last="+last)
    wm=subprocess.Popen(["openbox"],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    try:
        with log.open("wb") as stream:
            app=subprocess.Popen([str(exe),"--english"],cwd=work,env=env,stdout=stream,stderr=stream)
            try:
                observed=[]
                for key,expected,name in [(None,"rows=8 alerts=3 visible=8 mode=all lang=en","01-initial.png"),
                         ("ctrl+f","rows=8 alerts=3 visible=3 mode=alerts lang=en","02-filtered.png"),
                         ("ctrl+l","rows=8 alerts=3 visible=3 mode=alerts lang=zh","03-chinese.png")]:
                    if key:
                        x("windowactivate","--sync",wid)
                        x("key",key)
                    wid,title=wait_window(expected)
                    time.sleep(.4)
                    screenshot=e.output/name
                    subprocess.run(["import","-window",wid,str(screenshot)],env=env,check=True,timeout=15)
                    require(screenshot.read_bytes().startswith(b"\x89PNG\r\n\x1a\n") and screenshot.stat().st_size>1000,"bad native screenshot")
                    observed.append(dict(title=title,image=name,sha256=digest(screenshot)))
                require(len({r["sha256"] for r in observed})==3,"native frames did not change")
                x("windowactivate","--sync",wid);x("key","ctrl+q")
                require(app.wait(timeout=15)==0,"native close failed")
                report["native_gui"]=dict(status="PASS",backend="Xvfb/X11 Mesa software OpenGL",screenshots=observed)
            finally:
                if app.poll() is None:app.kill();app.wait(timeout=10)
    finally:
        wm.terminate()
        try:wm.wait(timeout=5)
        except subprocess.TimeoutExpired:wm.kill();wm.wait(timeout=5)

def replay(e,scratch,report,bundle,native_gui):
    bundle=bundle.resolve()
    expected=bundle.with_suffix(".zip.sha256").read_text(encoding="utf-8").split()[0]
    require(digest(bundle)==expected,"learner ZIP hash mismatch")
    learner=scratch/"搬移 学员 GUI"
    with zipfile.ZipFile(bundle) as z:z.extractall(learner)
    lock=learner/"Cargo.lock";lock_hash=digest(lock);count=lock_count(lock)
    require(lock_hash==digest(LAB/"reference/Cargo.lock")==digest(LAB/"learner/Cargo.lock"),"bundle differs from committed lock")
    reference=scratch/"reference";stage("reference",reference)
    shutil.copyfile(lock,reference/"Cargo.lock")
    shutil.copytree(learner/"vendor",reference/"vendor")
    write(reference/".cargo/config.toml",CONFIG)
    e.env["CARGO_NET_OFFLINE"]="true"
    protected=("tests/contracts.rs","src/main.rs")
    for label,work in (("reference",reference),("learner",learner)):
        home=scratch/("empty-"+label);home.mkdir();e.env["CARGO_HOME"]=str(home)
        require(not list(home.iterdir()),"Cargo home not empty")
        before={p:digest(work/p) for p in protected}
        require(all(before[p]==digest(reference/p) for p in protected),"fixed files differ")
        if label=="reference":
            for mode,flags in (("debug",[]),("release",["--release"])):
                out,_,_=e.execute("reference-"+mode,CARGO+["test","--frozen","--all-targets"]+flags,work,timeout=1800)
                require("5 passed; 0 failed; 0 ignored" in out,"five contracts did not pass")
        else:
            e.execute("learner-compiles",CARGO+["check","--frozen","--all-targets"],work,timeout=1800)
            for n in range(1,5):
                out,_,_=e.execute(f"learner-red-{n}",CARGO+["test","--frozen","--test","contracts",f"step{n}_"],work,expected=(101,),timeout=600)
                require("test result: FAILED" in out,"not a real failing learner contract")
            for p in ("src/lib.rs","src/gui.rs"):shutil.copyfile(reference/p,work/p)
            out,_,_=e.execute("learner-green",CARGO+["test","--frozen","--all-targets"],work,timeout=1200)
            require("5 passed; 0 failed; 0 ignored" in out,"learner contracts not green")
        check_vendor(e,work,home,label)
        require(all(before[p]==digest(work/p) for p in protected) and digest(work/"Cargo.lock")==lock_hash,"fixed inputs modified")
    report["native_gui"]=dict(status="NOT_RUN",reason="interactive Windows desktop not exercised")
    if native_gui:
        out,_,_=e.execute("native-build",CARGO+["build","--frozen","--release","--message-format=json"],learner,timeout=1800)
        exe=None
        for line in out.splitlines():
            data=json.loads(line)
            if data.get("reason")=="compiler-artifact" and data.get("target",{}).get("name")=="e2-signaldesk" and data.get("executable"):
                exe=Path(data["executable"])
        require(exe is not None and exe.is_file(),"no actual native executable")
        native(e,learner,exe,report)
    require(digest(bundle)==expected,"pristine learner bundle changed")
    report["portable"]=dict(archive_sha256=expected,lock_sha256=lock_hash,registry_packages=count,empty_cache_replay="PASS")

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode",choices=("resolve","replay"),required=True)
    parser.add_argument("--output",required=True,type=Path)
    parser.add_argument("--bundle",type=Path)
    parser.add_argument("--native-gui",action="store_true")
    args=parser.parse_args();out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
    before=snapshot()
    report=dict(stage="E2",mode=args.mode,status="FAIL",system=platform.system(),git_commit=os.environ.get("GITHUB_SHA"),
                source_before=before,human_learning="not evaluated")
    try:
        with tempfile.TemporaryDirectory(prefix="rustapplab-e2-") as temp:
            scratch=Path(temp);e=Runner(out,scratch);report["steps"]=e.steps
            e.run("rustc-version",RUSTC+["--version"],scratch,contains="rustc 1.100.0 ")
            if args.mode=="resolve":resolve(e,scratch,report)
            else:
                require(args.bundle is not None,"replay requires bundle")
                replay(e,scratch,report,args.bundle,args.native_gui)
        require(snapshot()==before,"source changed")
        report["status"]="PASS"
    except (OSError,RuntimeError,subprocess.SubprocessError,ValueError,KeyError) as error:
        report["error"]=str(error);print(error,flush=True)
    report["source_after"]=snapshot();report["source_unchanged"]=before==report["source_after"]
    report["files"]={p.name:digest(p) for p in sorted(out.iterdir()) if p.is_file()}
    write(out/"report.json",json.dumps(report,indent=2,ensure_ascii=False)+"\n")
    return 0 if report["status"]=="PASS" and report["source_unchanged"] else 1

if __name__=="__main__":raise SystemExit(main())
