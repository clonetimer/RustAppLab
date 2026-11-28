#!/usr/bin/env python3
"""Real workbench HTTP and Chromium checks for modern learner projects."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile
import time
import traceback
from urllib.parse import urlencode
from urllib.request import Request, ProxyHandler, build_opener

from release_check import ROOT
from workbench_process import choose_workbench_course, running_workbench

COURSES = {
    "modern-d8": "modern-d8-edition",
    "modern-d9a": "modern-d9-stable",
    "modern-d9b": "modern-d9-stdlib",
    "modern-d10a": "modern-d10-async-core",
    "modern-d11a": "modern-d11-quality",
    "modern-e4": "modern-e4-interop",
    "modern-e5": "modern-e5-science",
    "modern-d11b": "modern-d11-safety"
}
OPENER = build_opener(ProxyHandler({}))

def api(base, route, form=None, **query):
    url = base + route + ("?" + urlencode(query) if query else "")
    data = None if form is None else urlencode(form).encode()
    headers = {"Origin": base, "X-RustAppLab": "workbench",
               "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8"}
    with OPENER.open(Request(url, data=data, headers=headers), timeout=15) as response:
        return json.loads(response.read())

def done(base, job):
    until = time.monotonic() + 185
    while time.monotonic() < until:
        result = api(base, "/api/job", id=job["id"])
        if result["status"] != "running":
            return result
        time.sleep(0.05)
    raise TimeoutError(job)

def check(base, course, project, step):
    return done(base, api(base, "/api/action", form={"id": course, "action": "check",
                    "step": step, "args": "", "project_path": project}))

def browser_flow(base, output, report):
    from playwright.sync_api import expect, sync_playwright
    with sync_playwright() as pw:
        browser = pw.chromium.launch()
        try:
            page = browser.new_page(viewport={"width": 1440, "height": 1000}, locale="zh-CN")
            page.goto(base + "/", wait_until="domcontentloaded")
            choose_workbench_course(page, "modern-e5")
            editor = page.locator("#source-editor")
            expect(editor).not_to_have_value("")
            original = editor.input_value()
            # The API acceptance already repaired this private copy. Exercise the actual
            # UI with an authentic compiler error and recovery on the same file.
            def click_check(expected):
                with page.expect_response(lambda r: r.url == base + "/api/action" and
                                          r.request.method == "POST") as response:
                    page.locator("#check-button").click()
                assert response.value.ok, response.value.text()
                job = done(base, response.value.json())
                assert job["status"] == expected and not job["stale"], job
                expect(page.locator("#console-summary")).to_have_attribute(
                    "data-state", "success" if expected == "succeeded" else "error", timeout=10000)
                return job
            editor.fill(original + '\ncompile_error!("modern browser recovery probe");\n')
            red = click_check("failed")
            assert "modern browser recovery probe" in red["output"]
            page.screenshot(path=str(output / "modern-browser-red.png"))
            editor.fill(original)
            green = click_check("succeeded")
            assert "test result: ok" in green["output"]
            page.screenshot(path=str(output / "modern-browser-green.png"))
            page.locator("#book-open").click()
            page.locator("#knowledge-search").fill("Welford")
            expect(page.locator('#knowledge-results [data-topic="e5"]')).to_be_visible()
            page.locator('#knowledge-results [data-topic="e5"]').click()
            expect(page.locator("#book-text")).to_contain_text("Welford")
            expect(page.locator("#book-practice")).to_be_visible()
            page.screenshot(path=str(output / "modern-browser-lesson.png"))
            page.locator("#book-close").click()
            page.set_viewport_size({"width": 390, "height": 844})
            editor.scroll_into_view_if_needed()
            expect(editor).to_be_in_viewport()
            session = page.context.new_cdp_session(page)
            assert session.send("Page.getLayoutMetrics")["cssContentSize"]["width"] <= 391
            session.detach()
            page.screenshot(path=str(output / "modern-browser-mobile.png"))
            report["browser"] = {"red": red, "green": green, "screenshots": {
                p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                for p in output.glob("modern-browser-*.png")}}
        finally:
            browser.close()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--browser", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {"status": "FAIL", "git_commit": os.environ.get("GITHUB_SHA"),
              "human_learning": "not evaluated", "projects": {}}
    exe = ROOT / "target/release" / ("labquest.exe" if os.name == "nt" else "labquest")
    try:
        with tempfile.TemporaryDirectory(prefix="modern-workbench-") as directory:
            tmp = Path(directory)
            workspace = tmp / "个人 现代 Rust"
            workspace.mkdir()
            with running_workbench([str(exe), "workbench", "--workspace", str(workspace),
                                    "--port", "0"], tmp, output / "server.txt") as base:
                catalog = {item["id"]: item for item in api(base, "/api/catalog")["courses"]}
                assert set(COURSES) <= set(catalog) and len(catalog) == 33
                for course, folder in COURSES.items():
                    report["current_course"] = course
                    print(course, "START", flush=True)
                    card = api(base, "/api/create", form={"id": course})
                    project = card["project"]
                    assert Path(project["path"]).resolve().is_relative_to(workspace.resolve())
                    assert set(project["files"]) == {"src/lib.rs", "LEARNING_LOG.md"}
                    assert project["history"] is None
                    root = Path(project["path"])
                    template = ROOT / "projects" / folder / "learner"
                    source = root / "src/lib.rs"
                    assert source.read_bytes() == (template / "src/lib.rs").read_bytes()
                    frozen = {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                              for p in root.rglob("*") if p.is_file() and
                              p not in (source, root / "LEARNING_LOG.md")}
                    reds = []
                    for step in range(1, len(catalog[course]["steps"])):
                        red = check(base, course, str(root), step)
                        assert red["exit_code"] == 101 and "test result: FAILED" in red["output"], red
                        assert "+1.100.0" in red["output"] and "--frozen" in red["output"]
                        reds.append(red)
                    report["projects"][course] = {"red": reds}
                    current = api(base, "/api/file", id=course, path="src/lib.rs", project_path=str(root))
                    # D8 is a two-edition reference workspace; the learner matches e2024.
                    reference = ROOT / "projects" / folder / "reference"
                    if course == "modern-d8":
                        reference /= "e2024"
                    solved_path = reference / "src/lib.rs"
                    report["reference_path"] = solved_path.relative_to(ROOT).as_posix()
                    solved = solved_path.read_text(encoding="utf-8")
                    api(base, "/api/save", form={"id": course, "path": "src/lib.rs",
                        "revision": current["revision"], "content": solved, "project_path": str(root)})
                    assert all(api(base, "/api/job", id=red["id"])["stale"] for red in reds)
                    greens = []
                    for step in range(1, len(catalog[course]["steps"])):
                        green = check(base, course, str(root), step)
                        assert green["exit_code"] == 0 and "test result: ok" in green["output"], green
                        assert "running 0 tests" not in green["output"], green
                        assert f"step{step}_" in green["output"] and not green["stale"]
                        greens.append(green)
                    final = check(base, course, str(root), len(catalog[course]["steps"]))
                    assert final["exit_code"] == 0 and "--all-targets" in final["output"], final
                    assert "--doc" in final["output"] and not final["stale"]
                    assert all(hashlib.sha256((root / p).read_bytes()).hexdigest() == h
                               for p, h in frozen.items())
                    assert (template / "src/lib.rs").read_bytes() != source.read_bytes()
                    report["projects"][course] = {"red": reds, "green": greens, "final": final}
                    print(course, "PASS", flush=True)
                if args.browser:
                    browser_flow(base, output, report)
        report["status"] = "PASS"
    except Exception as exc:
        report["error"] = repr(exc)
        report["traceback"] = traceback.format_exc()
        print("FAIL", repr(exc), flush=True)
        traceback.print_exc()
    finally:
        (output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return 0 if report["status"] == "PASS" else 1

if __name__ == "__main__":
    raise SystemExit(main())
