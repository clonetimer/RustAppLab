#!/usr/bin/env python3
"""Actual frozen Cargo replay through the workbench, on prepared offline trees."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import traceback
from urllib.error import HTTPError
from modern_workbench_check import api
from release_check import ROOT
from workbench_process import running_workbench
from modern_d8_check import write
import modern_d10b_check as d10
import modern_e1a_check as e1a
import modern_e1b_check as e1b
import modern_e2_check as e2
import modern_e3_check as e3

COURSES = {
    "modern-d10b": (d10, "ordered=[9, 1, 4]"),
    "modern-e1a": (e1a, "HTTPS verified;"),
    "modern-e1b": (e1b, "HTTPS+SQLite verified;"),
    "modern-e2": (e2, None),
    "modern-e3": (e3, "host fixture only; not a physical timing test"),
}

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def prepare(module, work, log):
    if module is d10:
        shutil.copytree(module.LAB / "learner", work,
                        ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
    else:
        module.stage("learner", work)
        shutil.copyfile(module.LAB / "learner/Cargo.lock", work / "Cargo.lock")
    before = digest(work / "Cargo.lock")
    with log.open("wb") as stream:
        subprocess.run(["cargo", "+1.100.0", "vendor", "--locked", "--versioned-dirs",
                        "vendor"], cwd=work, stdout=stream, stderr=subprocess.STDOUT,
                       check=True, timeout=1200)
    assert digest(work / "Cargo.lock") == before
    write(work / ".cargo/config.toml", module.CONFIG)

def completed(base, job):
    until = time.monotonic() + 1205
    while time.monotonic() < until:
        result = api(base, "/api/job", id=job["id"])
        if result["status"] != "running":
            return result
        time.sleep(.15)
    raise TimeoutError(job)

def operate(base, course, work, step, action="check"):
    return completed(base, api(base, "/api/action", form={
        "id": course, "action": action, "step": step, "args": "", "project_path": str(work)}))

def reject(base, course, work):
    try:
        api(base, "/api/open", form={"id": course, "path": str(work)})
    except HTTPError as exc:
        assert exc.code in (400, 404)
        return
    raise AssertionError("malformed offline project accepted")

def browser_check(base, course, work, output, report):
    from playwright.sync_api import expect, sync_playwright
    with sync_playwright() as pw:
        browser = pw.chromium.launch()
        try:
            page = browser.new_page(viewport={"width": 1440, "height": 1000}, locale="zh-CN")
            page.goto(base + "/", wait_until="domcontentloaded")
            page.locator(f'#course-list [data-course="{course}"]').click()
            expect(page.locator("#create-button")).to_be_hidden()
            page.locator("#project-path").fill(str(work))
            with page.expect_response(lambda r: r.url == base + "/api/open" and r.request.method == "POST") as response:
                page.locator("#open-project-button").click()
            assert response.value.ok, response.value.text()
            expect(page.locator("#source-editor")).to_have_value((work / "src/lib.rs").read_text(encoding="utf-8"))
            with page.expect_response(lambda r: r.url == base + "/api/action" and r.request.method == "POST") as response:
                page.locator("#check-button").click()
            assert response.value.ok
            job = completed(base, response.value.json())
            assert job["exit_code"] == 0 and "test result: ok" in job["output"], job
            expect(page.locator("#console-summary")).to_have_attribute("data-state", "success", timeout=10000)
            page.screenshot(path=str(output / "prepared-browser.png"))
            report["browser"] = {"job": job, "screenshot_sha256": digest(output / "prepared-browser.png")}
        finally:
            browser.close()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--course", choices=COURSES, required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--browser", action="store_true")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {"status": "FAIL", "course": args.course, "git_commit": os.environ.get("GITHUB_SHA"),
              "human_learning": "not evaluated", "vendor_scope": "trusted dependency environment"}
    module, marker = COURSES[args.course]
    exe = ROOT / "target/release" / ("labquest.exe" if os.name == "nt" else "labquest")
    try:
        with tempfile.TemporaryDirectory(prefix="prepared-wb-") as temp:
            tmp = Path(temp)
            stage = tmp / "stage"
            prepare(module, stage, output / "vendor.txt")
            workspace = tmp / "工作区"
            workspace.mkdir()
            work = workspace / "搬移 学员"
            shutil.move(stage, work)
            home = tmp / "empty-cargo"
            home.mkdir()
            env = dict(os.environ, CARGO_HOME=str(home), CARGO_NET_OFFLINE="true")
            with running_workbench([str(exe), "workbench", "--workspace", str(workspace),
                                    "--port", "0"], tmp, output / "server.txt", env) as base:
                card = api(base, "/api/course", id=args.course, step=1)
                assert card["bundle"] and card["project"] is None
                try:
                    api(base, "/api/create", form={"id": args.course})
                except HTTPError as exc:
                    assert exc.code == 400
                else:
                    raise AssertionError("incomplete dependency template created")
                config = work / ".cargo/config.toml"
                data = config.read_bytes()
                config.write_bytes(data + b"\n[build]\n")
                reject(base, args.course, work)
                config.write_bytes(data)
                moved = work / "vendor-moved"
                (work / "vendor").rename(moved)
                reject(base, args.course, work)
                moved.rename(work / "vendor")
                if module in (e1a, e1b):
                    p = work / ("support/c2/src/lib.rs" if module is e1a else "support/e1a/src/lib.rs")
                    data = p.read_bytes()
                    p.write_bytes(data + b"\n// changed prerequisite\n")
                    reject(base, args.course, work)
                    p.write_bytes(data)
                card = api(base, "/api/open", form={"id": args.course, "path": str(work)})
                project = card["project"]
                editable = ["src/lib.rs", "src/gui.rs"] if module is e2 else ["src/lib.rs"]
                assert set(project["files"]) == {*editable, "LEARNING_LOG.md"} and project["history"] is None
                for p in editable:
                    assert (work / p).read_bytes() == (module.LAB / "learner" / p).read_bytes()
                frozen = {}
                for root, dirs, names in os.walk(work):
                    dirs[:] = [d for d in dirs if d not in ("vendor", "target")]
                    for name in names:
                        p = Path(root) / name
                        rel = p.relative_to(work).as_posix()
                        if rel not in [*editable, "LEARNING_LOG.md"]:
                            frozen[rel] = digest(p)
                red = operate(base, args.course, work, 1)
                report["red"] = red
                assert red["exit_code"] == 101 and "test result: FAILED" in red["output"], red
                for p in editable:
                    current = api(base, "/api/file", id=args.course, path=p, project_path=str(work))
                    solution = (module.LAB / "reference" / p).read_text(encoding="utf-8")
                    api(base, "/api/save", form={"id": args.course, "path": p,
                        "revision": current["revision"], "content": solution, "project_path": str(work)})
                assert api(base, "/api/job", id=red["id"])["stale"]
                final = operate(base, args.course, work, card["step_count"])
                report["final"] = final
                assert final["exit_code"] == 0 and "--all-targets" in final["output"] and "--doc" in final["output"], final
                if marker:
                    run = operate(base, args.course, work, 1, "run")
                    report["demo"] = run
                    assert run["exit_code"] == 0 and marker in run["output"], run
                assert all(digest(work / p) == h for p, h in frozen.items())
                registry = home / "registry"
                assert not registry.exists() or all(p.name == "CACHEDIR.TAG" for p in registry.rglob("*") if p.is_file())
                if args.browser:
                    browser_check(base, args.course, work, output, report)
        report["status"] = "PASS"
    except Exception as exc:
        report.update(error=repr(exc), traceback=traceback.format_exc())
        traceback.print_exc()
    finally:
        (output / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False)+"\n", encoding="utf-8")
    return 0 if report["status"] == "PASS" else 1

if __name__ == "__main__":
    raise SystemExit(main())
