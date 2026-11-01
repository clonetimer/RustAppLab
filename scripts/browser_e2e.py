#!/usr/bin/env python3
"""Real browser -> local Rust HTTP -> files. No mock fetch or set_content fallback.

Maintainer dependency: Playwright and a browser (not needed for learning Rust).
Exit 0 = both real UI flows passed; 2 = environment blocked; 1 = application/test failure.
"""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path
import platform
import re
import shutil
import socket
import subprocess
import tempfile
import time
from urllib.request import urlopen
from release_check import source_id

ROOT = Path(__file__).resolve().parents[1]


def progress(stage: str, message: str) -> None:
    """Visible stage markers, flushed so PowerShell can surface progress promptly."""
    print(f'[browser:{stage}] {message}', flush=True)



def port() -> int:
    with socket.socket() as sock:
        sock.bind(('127.0.0.1',0))
        return sock.getsockname()[1]


def start(exe: Path, args: list[str], url: str) -> subprocess.Popen:
    proc = subprocess.Popen([str(exe),*args],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    for _ in range(100):
        try:
            with urlopen(url,timeout=.3) as response:
                assert response.status == 200
            return proc
        except OSError:
            if proc.poll() is not None:
                raise RuntimeError('Rust HTTP server exited during startup')
            time.sleep(.05)
    stop(proc)
    raise RuntimeError('Rust HTTP server startup timed out')


def stop(proc: subprocess.Popen) -> None:
    proc.terminate()
    try:proc.wait(timeout=4)
    except subprocess.TimeoutExpired:proc.kill();proc.wait(timeout=4)


def find_browser(explicit: str | None) -> str | None:
    """Find a real installed browser without changing its security policy."""
    candidates = [explicit, os.environ.get('CHROMIUM_PATH'), os.environ.get('CHROME_PATH'),
                  os.environ.get('EDGE_PATH')]
    for command in ('chromium', 'chromium-browser', 'google-chrome', 'chrome', 'msedge'):
        candidates.append(shutil.which(command))
    if os.name == 'nt':
        roots = [os.environ.get('PROGRAMFILES'), os.environ.get('PROGRAMFILES(X86)'),
                 os.environ.get('LOCALAPPDATA')]
        relatives = [r'Google\Chrome\Application\chrome.exe',
                     r'Microsoft\Edge\Application\msedge.exe',
                     r'Chromium\Application\chrome.exe']
        for base in roots:
            if base:
                candidates.extend(str(Path(base) / rel) for rel in relatives)
    for candidate in candidates:
        if candidate and Path(candidate).is_file():
            return str(Path(candidate).resolve())
    return None


def main() -> int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--browser-path',help='An installed Chromium-compatible browser; policy is not modified')
    args=parser.parse_args();out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
    report=dict(version=(ROOT/'VERSION').read_text().strip(),platform=platform.platform(),source_id=source_id(),
                method='real browser navigation and fetch, no mocks',status='RUNNING',checks=[])
    try:
        progress('setup', 'Importing Playwright and finding browser')
        from playwright.sync_api import sync_playwright, expect
        binary=find_browser(args.browser_path)
        with tempfile.TemporaryDirectory(prefix='applab-browser-') as temp, sync_playwright() as pw:
            tmp=Path(temp);suffix='.exe' if os.name=='nt' else '';bins=ROOT/'target/release'
            options=dict(headless=True)
            if binary:options['executable_path']=binary
            if os.name!='nt':options['args']=['--no-sandbox','--disable-dev-shm-usage']
            progress('launch', f'Launching Chromium: {binary or "Playwright-managed"}')
            browser=pw.chromium.launch(**options)
            progress('launch', 'Browser launched')
            report['browser_version']=browser.version
            try:
                for app in ['localhub','servicedesk']:
                    progress(app, 'Preparing real Rust demo and HTTP server')
                    proc=None
                    page=browser.new_page(viewport=dict(width=1280,height=900))
                    errors=[];dialogs=[]
                    page.on('pageerror',lambda err: errors.append(str(err)))
                    page.on('dialog',lambda dialog: (dialogs.append(dialog.message),dialog.dismiss()))
                    try:
                        demo=tmp/app
                        subprocess.run([str(bins/(app+suffix)),'demo',str(demo)],check=True,capture_output=True,timeout=20)
                        p=port();url=f'http://127.0.0.1:{p}'
                        if app=='localhub':
                            cmd=['serve','--root',str(demo/'files'),'--log',str(demo/'logs/candidate.log'),'--port',str(p)]
                            health='/api/health'
                        else:
                            cmd=['serve',str(demo/'tickets.log'),str(p)];health='/api/tickets'
                        proc=start(bins/(app+suffix),cmd,url+health)
                        progress(app, 'HTTP health check OK; navigating browser to actual localhost page')
                        response=page.goto(url+'/',wait_until='networkidle',timeout=15000)
                        assert response is not None and response.status==200
                        if app=='localhub':
                            expect(page.locator('#status')).to_have_text(re.compile(r'^✓'), timeout=10000)
                            assert page.locator('#errorRate').inner_text()=='37.50%'
                            rows=page.locator('#fileRows tr:visible').count();assert rows>0
                            page.locator('#search').fill('no-match-rc-qualification')
                            assert page.locator('#fileRows tr:visible').count()==0
                            page.locator('#search').fill('')
                            assert page.locator('#fileRows tr:visible').count()==rows
                        else:
                            expect(page.locator('#count')).to_have_text('(2)', timeout=10000)
                            title='RC 中文 <img src=x onerror=alert(1)>'
                            page.locator('#title').fill(title)
                            page.locator('#new button').click()
                            expect(page.locator('#count')).to_have_text('(3)', timeout=10000)
                            row=page.locator('#tickets li').filter(has_text=title)
                            assert row.count()==1 and row.locator('img').count()==0
                            row.locator('button').click()
                            expect(page.locator('#tickets li.closed')).to_have_count(2, timeout=10000)
                            # Restart the actual Rust server and reload, not a cached DOM.
                            stop(proc);proc=None
                            p=port();url=f'http://127.0.0.1:{p}'
                            proc=start(bins/(app+suffix),['serve',str(demo/'tickets.log'),str(p)],url+'/api/tickets')
                            page.goto(url+'/',wait_until='networkidle',timeout=15000)
                            expect(page.locator('#tickets li.closed')).to_have_count(2, timeout=10000)
                            assert page.locator('#tickets li').filter(has_text=title).count()==1
                        progress(app, 'UI assertions passed; writing desktop and mobile screenshots')
                        page.screenshot(path=str(out/(app+'-desktop.png')),full_page=True)
                        page.set_viewport_size(dict(width=390,height=844))
                        # Use Playwright's DOM box API instead of page.evaluate: ServiceDesk
                        # forbids unsafe-eval under its Content Security Policy.
                        # This checks primary layout panels; horizontal overflow
                        # elsewhere is not claimed to be exhaustively measured.
                        for selector in ('body', 'main'):
                            element = page.locator(selector)
                            if element.count():
                                rect = element.bounding_box()
                                assert rect is not None
                                assert rect['x'] >= -1 and rect['x'] + rect['width'] <= 391, (selector, rect)
                        page.screenshot(path=str(out/(app+'-mobile.png')),full_page=True)
                        assert not errors and not dialogs,(errors,dialogs)
                        report['checks'].append(dict(app=app,status='PASS',javascript_errors=errors))
                        progress(app, 'PASS')
                    except Exception as exc:
                        message=str(exc)
                        state='BLOCKED_ENVIRONMENT' if 'ERR_BLOCKED_BY_ADMINISTRATOR' in message else 'FAIL'
                        report['checks'].append(dict(app=app,status=state,error=message))
                        progress(app, f'{state}: {message[:200]}')
                    finally:
                        page.close()
                        if proc is not None:stop(proc)
            finally:browser.close()
        report['status']='PASS' if len(report['checks'])==2 and all(x['status']=='PASS' for x in report['checks']) else ('FAIL' if any(x['status']=='FAIL' for x in report['checks']) else 'BLOCKED_ENVIRONMENT')
    except Exception as exc:
        report['status']='BLOCKED_ENVIRONMENT'
        report['environment_error']=str(exc)
    progress('finish', f"Summary: {report['status']}")
    report['source_id_after']=source_id()
    if report['source_id_after'] != report['source_id']:report['status']='FAIL'
    (out/'browser.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps(report,ensure_ascii=False,indent=2))
    return 0 if report['status']=='PASS' else (2 if report['status']=='BLOCKED_ENVIRONMENT' else 1)

if __name__=='__main__':raise SystemExit(main())
