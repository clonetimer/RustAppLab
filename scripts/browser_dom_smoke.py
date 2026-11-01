#!/usr/bin/env python3
"""可选浏览器 DOM 回归：真实 Rust HTTP API + 浏览器 DOM 脚本。

限制：页面使用 set_content + 拦截 fetch 将真实 API JSON 注入；**不等同于**
Chromium 通过网络加载 LocalHub 的端到端浏览器验收。该容器的 Chromium
对 127.0.0.1 导航返回 ERR_BLOCKED_BY_ADMINISTRATOR。

依赖：Python playwright 库 + 本机 Chromium；并需 Rust cargo。
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
from pathlib import Path
from urllib.request import urlopen

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
UI = ROOT / "apps/localhub/src/ui"
OUT = ROOT / "verification"
OUT.mkdir(exist_ok=True)


def get_json(url: str) -> dict:
    with urlopen(url, timeout=5) as response:
        if response.status != 200:
            raise AssertionError(f"HTTP {response.status}: {url}")
        return json.loads(response.read())


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="sfs-browser-dom-") as tmp:
        demo = Path(tmp) / "demo"
        subprocess.run(
            ["cargo", "run", "--locked", "--offline", "-q", "-p", "localhub", "--", "demo", str(demo)],
            cwd=ROOT, check=True, stdout=subprocess.DEVNULL,
        )
        # 测试 XSS 尝试：Rust 输出 JSON，JS 必须 textContent，而不是 innerHTML。
        attack_name = "<img src=x onerror=alert(1)>.txt"
        try:
            (demo / "files" / attack_name).write_text("harmless", encoding="utf-8")
            attack_supported = True
        except OSError:
            attack_supported = False  # Windows 文件系统不允许 <> 文件名
        command = ["cargo", "run", "--locked", "--offline", "-q", "-p", "localhub", "--", "serve", "--port", "0", "--root", str(demo / "files"), "--log", str(demo / "logs/candidate.log")]
        env = os.environ.copy()
        env["NO_PROXY"] = "127.0.0.1,localhost"
        env["no_proxy"] = "127.0.0.1,localhost"
        proc = subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            assert proc.stdout is not None
            boot = proc.stdout.readline()
            match = re.search(r"http://127\.0\.0\.1:(\d+)/", boot)
            if not match:
                raise RuntimeError(f"Rust HTTP startup failed: {boot!r}")
            url = f"http://127.0.0.1:{match.group(1)}"
            files = get_json(url + "/api/files?limit=200")
            logs = get_json(url + "/api/logs")
            assert files["stats"]["files"] > 5, files
            assert logs["records"] == 8 and logs["errors"] == 3, logs
            html = (UI / "index.html").read_text(encoding="utf-8")
            html = html.replace('<link rel="stylesheet" href="/assets/app.css">', "")
            html = html.replace('<script src="/assets/app.js" defer></script>', "")
            css = (UI / "app.css").read_text(encoding="utf-8")
            js = (UI / "app.js").read_text(encoding="utf-8")
            binary = os.getenv("CHROMIUM_PATH") or "/usr/bin/chromium"
            with sync_playwright() as playwright:
                browser = playwright.chromium.launch(
                    headless=True,
                    executable_path=binary,
                    args=["--no-sandbox", "--disable-dev-shm-usage"],
                )
                try:
                    page = browser.new_page(viewport={"width": 1365, "height": 900})
                    errors: list[str] = []
                    page.on("pageerror", lambda exc: errors.append(str(exc)))
                    page.set_content(html, wait_until="domcontentloaded")
                    page.add_style_tag(content=css)
                    page.evaluate("""payload => {
                      window.fetch = async (url) => ({
                        ok: true,
                        json: async () => url.startsWith('/api/files') ? payload.files : payload.logs
                      });
                    }""", {"files": files, "logs": logs})
                    page.evaluate(js)
                    page.wait_for_function("document.querySelector('#status').textContent.startsWith('✓')")
                    assert page.locator("#fileCount").inner_text() == str(files["stats"]["files"])
                    assert page.locator("#errorRate").inner_text() == "37.50%"
                    assert page.locator("#fileRows tr").count() == len(files["files"])
                    if attack_supported:
                        page.locator("#search").fill("<img src")
                        assert page.locator("#fileRows tr:visible").count() == 1
                        assert page.locator("#fileRows img").count() == 0
                    page.locator("#search").fill("non-existent-filter-value")
                    assert page.locator("#fileRows tr:visible").count() == 0
                    page.locator("#search").fill("")
                    assert page.locator("#fileRows tr:visible").count() == len(files["files"])
                    assert not errors, errors
                    page.screenshot(path=OUT / "LOCALHUB_UI_DOM_DESKTOP.png", full_page=True)
                    page.set_viewport_size({"width": 390, "height": 844})
                    assert page.evaluate("document.documentElement.scrollWidth") <= 390
                    page.screenshot(path=OUT / "LOCALHUB_UI_DOM_MOBILE.png", full_page=True)
                    report = (
                        "LocalHub DOM/JS smoke (real Rust HTTP JSON, browser mock fetch)\n"
                        "NOTE: Not a real browser-to-HTTP end-to-end test. Browser localhost navigation blocked by policy.\n"
                        f"Files from HTTP: {files['stats']['files']}; DOM rows: {len(files['files'])}\n"
                        f"Log HTTP: {logs['records']} records, {logs['errors']} errors, 37.50%\n"
                        f"Injection escaped: {attack_supported}; search filter: pass; mobile overflow: no\n"
                        "JavaScript errors: none\n"
                        "Result: PASS (DOM only)\n"
                    )
                    (OUT / "LOCALHUB_UI_DOM_TEST_EVIDENCE.txt").write_text(report, encoding="utf-8")
                    print(report)
                finally:
                    browser.close()
        finally:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)


if __name__ == "__main__":
    main()
