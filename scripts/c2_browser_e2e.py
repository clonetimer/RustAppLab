#!/usr/bin/env python3
"""Real Chromium end-to-end test for the C2 Axum ServiceDesk UI."""
import argparse
import json
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time
import urllib.request

from playwright.sync_api import sync_playwright


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--exe",type=Path,required=True)
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    exe=args.exe.resolve()
    if not exe.is_file():
        raise FileNotFoundError(exe)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    checks=[]
    with tempfile.TemporaryDirectory(prefix="c2-browser-") as tmp:
        root=Path(tmp)
        db=root/"中文 tickets.db"
        init=subprocess.run([str(exe),"init",str(db)],capture_output=True,text=True,timeout=10)
        assert init.returncode==0,init.stderr
        with socket.socket() as sock:
            sock.bind(("127.0.0.1",0))
            port=sock.getsockname()[1]
        log=(root/"server.log").open("w",encoding="utf-8")
        process=subprocess.Popen([str(exe),"serve",str(db),"--port",str(port)],
            stdout=log,stderr=subprocess.STDOUT,cwd=root)
        base=f"http://127.0.0.1:{port}"
        try:
            deadline=time.monotonic()+15
            while time.monotonic()<deadline:
                if process.poll() is not None:
                    raise AssertionError((root/"server.log").read_text(encoding="utf-8"))
                try:
                    with urllib.request.urlopen(base+"/health",timeout=1) as response:
                        health=json.loads(response.read())
                    if health.get("ok") is True: break
                except Exception:
                    time.sleep(.1)
            else: raise AssertionError("C2 HTTP server did not become ready")
            checks.append("real loopback server and health endpoint")
            with sync_playwright() as playwright:
                browser=playwright.chromium.launch()
                page=browser.new_page()
                errors=[]
                page.on("pageerror",lambda error:errors.append(str(error)))
                page.goto(base+"/",wait_until="networkidle")
                assert page.get_by_role("heading",name="ServiceDesk · 异步工单台").is_visible()
                title='<img src=x onerror=alert(1)> safe title'
                page.locator("#title").fill(title)
                page.get_by_role("button",name="创建").click()
                row=page.locator("#tickets li").filter(has_text=title)
                row.wait_for(state="visible",timeout=5000)
                assert row.locator("img").count()==0,"ticket title was parsed as HTML"
                assert row.get_by_role("button",name="关闭").is_visible()
                checks.append("create via browser JSON request; untrusted title rendered as text")
                row.get_by_role("button",name="关闭").click()
                page.get_by_text("已关闭工单 #1").wait_for()
                assert row.get_by_text("（已关闭）").is_visible()
                page.reload(wait_until="networkidle")
                assert page.locator("#tickets li").filter(has_text=title).get_by_text("（已关闭）").is_visible()
                assert page.locator("#tickets li button").count()==0
                assert not errors,errors
                checks.append("close, reload and persistent UI state; no JavaScript errors")
                browser.close()
        finally:
            if process.poll() is None:
                process.send_signal(signal.SIGINT)
                try: process.wait(timeout=8)
                except subprocess.TimeoutExpired:
                    process.kill();process.wait()
            log.close()
        assert process.returncode==0, (root/"server.log").read_text(encoding="utf-8")
    args.output.write_text(json.dumps({"status":"PASS","checks":checks},ensure_ascii=False,indent=2),encoding="utf-8")
    print(json.dumps({"status":"PASS","checks":checks},ensure_ascii=False))


if __name__=="__main__":main()
