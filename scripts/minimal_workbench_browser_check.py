#!/usr/bin/env python3
"""Real Chromium comparison of the previous and minimal workbench appearance."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import traceback

from release_check import ROOT
from workbench_process import running_workbench

BASELINE = "a3659789c1b18e8009eec94c1ba6554e48108635"
FIRST_MINIMAL = "8bb350c215d8153a928fbe167157ba8c6d77a93f"
SECOND_MINIMAL = "b767cff4beb441f1e1cea41479a5a8a1043d5331"
UI = "apps/labquest/src/workbench/ui"
SIZES = ((1440, 900), (1366, 768), (390, 844), (320, 844))


def git_bytes(revision, path):
    return subprocess.check_output(["git", "show", f"{revision}:{UI}/{path}"], cwd=ROOT)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    report = {"status": "FAIL", "baseline": BASELINE, "commit": os.environ.get("GITHUB_SHA"),
              "first_minimal": FIRST_MINIMAL, "second_minimal": SECOND_MINIMAL,
              "comparison": "Same live backend, course and project; each historical HTML/CSS/JS from its frozen Git revision",
              "screenshots": {}, "layout": {}, "human_learning": "not evaluated"}
    exe = ROOT / "target/release" / ("labquest.exe" if os.name == "nt" else "labquest")
    try:
        from playwright.sync_api import expect, sync_playwright
        historical = {side: {name: git_bytes(revision, name)
                             for name in ("app.css", "index.html", "app.js")}
                      for side, revision in (("before", BASELINE), ("v1", FIRST_MINIMAL),
                                             ("v2", SECOND_MINIMAL))}
        report["historical_sha256"] = {side: {name: hashlib.sha256(data).hexdigest()
                                              for name, data in assets.items()}
                                       for side, assets in historical.items()}
        with tempfile.TemporaryDirectory(prefix="minimal-ui-") as temp, sync_playwright() as pw:
            tmp = Path(temp)
            workspace = tmp / "学习工作区"
            workspace.mkdir()
            with running_workbench([str(exe), "workbench", "--workspace", str(workspace), "--port", "0"],
                                    tmp, out / "server.txt") as base:
                browser = pw.chromium.launch()
                try:
                    report["browser_version"] = browser.version
                    setup = browser.new_page(viewport={"width": 1440, "height": 900}, color_scheme="light")
                    setup.goto(base + "/")
                    for course in ("firstcard", "modern-d9b"):
                        setup.locator(f'#course-list [data-course="{course}"]').click()
                        setup.locator("#create-button").click()
                        expect(setup.locator("#source-editor")).not_to_have_value("")
                    setup.close()

                    def shot(page, name):
                        target = out / (name + ".png")
                        page.screenshot(path=str(target), animations="disabled")
                        report["screenshots"][target.name] = hashlib.sha256(target.read_bytes()).hexdigest()

                    def measure(page):
                        return page.evaluate("""() => {
                            const strip = document.querySelector('#step-list');
                            const editor = document.querySelector('#source-editor');
                            const lesson = document.querySelector('#lesson-panel');
                            const rows = new Set([...strip.querySelectorAll('button')]
                                .map(b => Math.round(b.getBoundingClientRect().top)));
                            return { document_width: document.documentElement.scrollWidth,
                                viewport_width: innerWidth,
                                step_rows: rows.size,
                                step_height: strip.clientHeight,
                                step_scroll_width: strip.scrollWidth,
                                step_client_width: strip.clientWidth,
                                visible_objective_rows: [...document.querySelectorAll('.objectives > div')]
                                    .filter(row => getComputedStyle(row).display !== 'none').length,
                                create_visible: getComputedStyle(document.querySelector('#create-button')).display !== 'none',
                                project_form_visible: getComputedStyle(document.querySelector('#project-form')).display !== 'none',
                                editor_width: Math.round(editor.getBoundingClientRect().width),
                                editor_height: Math.round(editor.getBoundingClientRect().height),
                                lesson_width: Math.round(lesson.getBoundingClientRect().width),
                                body_background: getComputedStyle(document.body).backgroundColor,
                                body_gradient: getComputedStyle(document.body).backgroundImage };
                        }""")

                    for side in ("before", "v1", "v2", "after"):
                        page = browser.new_page(viewport={"width": 1440, "height": 900}, color_scheme="light")
                        errors = []
                        page.on("pageerror", lambda error: errors.append(str(error)))
                        if side in historical:
                            assets = historical[side]
                            page.route(base + "/", lambda route: route.fulfill(
                                body=assets["index.html"], content_type="text/html; charset=utf-8"))
                            page.route(base + "/assets/app.css", lambda route: route.fulfill(
                                body=assets["app.css"], content_type="text/css; charset=utf-8"))
                            page.route(base + "/assets/app.js", lambda route: route.fulfill(
                                body=assets["app.js"], content_type="application/javascript; charset=utf-8"))
                        page.goto(base + "/")
                        expect(page.locator("#course-list [data-course]")).to_have_count(33)
                        for course in ("firstcard", "modern-d9b"):
                            page.locator(f'#course-list [data-course="{course}"]').click()
                            editor = page.locator("#source-editor")
                            expect(editor).not_to_have_value("")
                            expect(page.locator("#check-button")).to_be_enabled()
                            for width, height in SIZES:
                                page.set_viewport_size({"width": width, "height": height})
                                page.locator(".masthead").scroll_into_view_if_needed()
                                key = f"{side}-{course}-{width}x{height}"
                                if side == "after":
                                    active_course = page.locator(f'#course-list [data-course="{course}"]')
                                    expect(active_course).to_be_in_viewport(ratio=.8)
                                    expect(page.locator("#create-button")).to_be_hidden()
                                    expect(page.locator("#project-form")).to_be_hidden()
                                    expect(page.locator("#switch-project-button")).to_have_attribute("aria-expanded", "false")
                                    if course == "modern-d9b":
                                        expect(page.locator('#step-list [data-step="1"]')).to_have_attribute(
                                            "aria-label", "第 1 步：inspect_readings：观察 Result")
                                        expect(page.locator("#lesson-goal")).to_be_hidden()
                                        expect(page.locator("#lesson-function")).to_be_hidden()
                                        expect(page.locator("#lesson-concept")).to_be_visible()
                                metrics = measure(page)
                                report["layout"][key] = metrics
                                shot(page, key)
                                if side == "after":
                                    assert metrics["document_width"] <= width + 1, (key, metrics)
                                    assert metrics["body_gradient"] == "none", metrics
                                    assert metrics["step_rows"] == 1, metrics
                                    if width >= 1366:
                                        assert metrics["editor_width"] > metrics["lesson_width"], metrics
                                        expect(editor).to_be_in_viewport(ratio=.25)
                                        expect(page.locator("#console-summary")).to_be_in_viewport()
                                    else:
                                        # The native in-page jump must reveal the editor without
                                        # selecting another course or changing the learner draft.
                                        original = editor.input_value()
                                        page.locator("#jump-to-editor").click()
                                        expect(page.locator("#editor-title")).to_be_in_viewport()
                                        expect(editor).to_be_in_viewport(ratio=.65)
                                        expect(editor).to_have_value(original)
                                        shot(page, key + "-editor")
                                        page.locator("#jump-to-lesson").click()
                                        expect(page.locator("#lesson-title")).to_be_in_viewport()
                                        expect(editor).to_have_value(original)
                            page.set_viewport_size({"width": 1440, "height": 900})
                        if side == "after":
                            # Opening the switch field must not modify the current editor;
                            # rejecting an existing discard prompt must preserve the draft.
                            editor = page.locator("#source-editor")
                            original = editor.input_value()
                            switch = page.locator("#switch-project-button")
                            form = page.locator("#project-form")
                            field = page.locator("#project-path")
                            switch.click()
                            expect(form).to_be_visible()
                            expect(switch).to_have_attribute("aria-expanded", "true")
                            selected_path = field.input_value()
                            editor.fill(original + "\n// unsaved project switch probe")
                            prompts = []
                            def reject_switch(dialog):
                                prompts.append(dialog.message)
                                dialog.dismiss()
                            page.once("dialog", reject_switch)
                            page.locator("#open-project-button").click()
                            assert len(prompts) == 1, prompts
                            expect(editor).to_have_value(original + "\n// unsaved project switch probe")
                            expect(form).to_be_visible()
                            field.fill(selected_path + "-not-selected")
                            switch.click()
                            expect(form).to_be_hidden()
                            expect(field).to_have_value(selected_path)
                            expect(editor).to_have_value(original + "\n// unsaved project switch probe")
                            editor.fill(original)
                            switch.click()
                            with page.expect_response(lambda r: r.url == base + "/api/open" and r.request.method == "POST") as response:
                                page.locator("#open-project-button").click()
                            assert response.value.ok, response.value.text()
                            expect(form).to_be_hidden()
                            expect(editor).to_have_value(original)
                            report["project_switch"] = {"discard_cancel_preserved": True, "reopen_success": True}
                            expect(page.locator("#step-list [data-step]")).to_have_count(17)
                            page.locator('#step-list [data-step="16"]').click()
                            expect(page.locator('#step-list [data-step="16"]')).to_have_attribute("aria-current", "step")
                            expect(page.locator('#step-list [data-step="16"]')).to_be_in_viewport()
                            shot(page, "after-long-course-step16")
                            for width, height in SIZES[2:]:
                                page.set_viewport_size({"width": width, "height": height})
                                expect(page.locator('#course-list [data-course="modern-d9b"]')).to_be_in_viewport(ratio=.8)
                                expect(page.locator('#step-list [data-step="16"]')).to_be_in_viewport(ratio=.8)
                                shot(page, f"after-step16-{width}x{height}")
                            page.set_viewport_size({"width": 1440, "height": 900})
                            page.emulate_media(color_scheme="dark")
                            expect(page.locator("#source-editor")).to_be_visible()
                            metrics = measure(page)
                            report["dark_layout"] = metrics
                            assert metrics["body_background"] != report["layout"]["after-modern-d9b-1440x900"]["body_background"]
                            shot(page, "after-dark")
                        assert not errors, errors
                        page.close()
                    for width, height in SIZES[:2]:
                        suffix = f"modern-d9b-{width}x{height}"
                        before = report["layout"]["before-" + suffix]
                        after = report["layout"]["after-" + suffix]
                        assert before["step_rows"] > after["step_rows"], (before, after)
                        assert after["editor_height"] > before["editor_height"], (before, after)
                        v1 = report["layout"]["v1-" + suffix]
                        assert after["step_scroll_width"] <= after["step_client_width"], after
                        assert after["step_scroll_width"] < v1["step_scroll_width"], (v1, after)
                        assert after["visible_objective_rows"] == 1 < v1["visible_objective_rows"]
                        assert not after["create_visible"] and v1["create_visible"]
                        v2 = report["layout"]["v2-" + suffix]
                        assert v2["project_form_visible"] and not after["project_form_visible"]
                        assert after["editor_height"] >= v2["editor_height"], (v2, after)
                finally:
                    browser.close()
        report["status"] = "PASS"
    except Exception as exc:
        report["error"] = repr(exc)
        report["traceback"] = traceback.format_exc()
        traceback.print_exc()
    finally:
        (out / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return 0 if report["status"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
