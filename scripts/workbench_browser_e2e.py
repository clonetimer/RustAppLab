#!/usr/bin/env python3
"""Real Chromium, the embedded workbench, and the learner's own compiled Rust."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from urllib.parse import urlencode, urlparse

from browser_e2e import find_browser
from release_check import ROOT, source_id
from workbench_process import running_workbench
from book_workbench_browser_e2e import run_book_flow

# Immutable comparison for manual runs and future PRs that already contain
# the workbench. The introduction PR uses its actual base revision.
PRE_WORKBENCH = '00ae3f95b6e11bfde63a7b10eca74353149a1bd3'
SIZES = ((1440, 1000), (1366, 768), (390, 844), (320, 844))
BASE_PAGES = {'firstcard': 'START_HERE.html', 'unitlab': 'apps/labquest/studio/unitlab_lesson.html'}
ACTIONS = {
    'firstcard-initial': ('firstcard', 'check', 1, 'failed'),
    'firstcard-pass': ('firstcard', 'check', 1, 'succeeded'),
    'firstcard-personal': ('firstcard', 'run', 1, 'succeeded'),
    'unitlab-initial': ('unitlab', 'check', 1, 'failed'),
    'unitlab-step1': ('unitlab', 'check', 1, 'succeeded'),
    'unitlab-step2': ('unitlab', 'check', 2, 'failed'),
    'unitlab-compiler': ('unitlab', 'check', 1, 'failed'),
    'unitlab-recovered': ('unitlab', 'check', 1, 'succeeded'),
}
STATE_SHOTS = ('pending', 'firstcard-initial', 'firstcard-pass', 'firstcard-personal',
               'unitlab-initial', 'unitlab-step1', 'unitlab-step2', 'unitlab-compiler',
               'collapsed-error', 'external-conflict', 'restored-selection')
OBSERVATIONS = {'all-course-cards', 'full-lesson-and-hints', 'desktop-mobile-layout',
                'save-and-reload', 'existing-project', 'dirty-navigation', 'external-conflict',
                'disconnect-retry', 'book-learning-bridge'}


def screenshot_sizes():
    sizes = {f'{side}-{course}-{w}x{h}.png': (w, h)
             for side in ('before-static', 'after-workbench') for course in BASE_PAGES
             for w, h in SIZES}
    sizes.update({name+'.png': SIZES[0] for name in STATE_SHOTS})
    sizes['mobile-pending.png'] = (320,844)
    sizes['book-link-1440x1000.png'] = (1440, 1000)
    sizes['book-reader-1440x1000.png'] = (1440, 1000)
    sizes['book-reader-320x844.png'] = (320, 844)
    for course in BASE_PAGES:
        for w,h in SIZES[2:]:
            sizes[f'after-workbench-{course}-editor-{w}x{h}.png'] = (w,h)
    return sizes


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def baseline_pages(tmp: Path, revision: str):
    if not re.fullmatch(r'[0-9a-f]{40}', revision) or not revision.strip('0'):
        revision = PRE_WORKBENCH
    present = subprocess.run(['git', 'cat-file', '-e', revision+':apps/labquest/src/workbench.rs'],
                             cwd=ROOT, capture_output=True)
    if present.returncode == 0:
        revision = PRE_WORKBENCH
    checkout = tmp/'old-static'
    checkout.mkdir()
    pages = {}
    for course, source in BASE_PAGES.items():
        data = git('show', f'{revision}:{source}')
        path = checkout / (course+'.html')
        path.write_bytes(data)
        pages[course] = {'source': source, 'sha256': hashlib.sha256(data).hexdigest(), 'url': path.as_uri()}
    return revision, pages


def capture(page, out: Path, report: dict, filename: str):
    dest = out/filename
    page.screenshot(path=str(dest), animations='disabled')
    report['screenshots'][filename] = {'viewport': page.viewport_size,
                                       'sha256': hashlib.sha256(dest.read_bytes()).hexdigest()}


def get_json(page, base, route, **params):
    url = base + route + ('?' + urlencode(params) if params else '')
    response = page.request.get(url)
    assert response.ok, (url, response.text())
    return response.json()


def viewport(page, width, height):
    page.set_viewport_size({'width': width, 'height': height})


def compare_layout(page, expect, out, report, course):
    # Chromium layout metrics measure real document overflow without executing
    # injected JS. Scroll within the actual panes or document is allowed.
    session = page.context.new_cdp_session(page)
    try:
        for width, height in SIZES:
            viewport(page, width, height)
            page.locator('.masthead').scroll_into_view_if_needed()
            expect(page.locator('#lesson-goal')).to_be_visible()
            expect(page.locator('#source-editor')).to_be_editable()
            if width >= 1366:
                for selector in ('#lesson-goal', '#source-editor', '#console-summary'):
                    expect(page.locator(selector)).to_be_in_viewport(ratio=.25)
            span = session.send('Page.getLayoutMetrics')['cssContentSize']['width']
            assert span <= width + 1, (course, width, span)
            capture(page, out, report, f'after-workbench-{course}-{width}x{height}.png')
            if width <= 390:
                page.locator('#source-editor').scroll_into_view_if_needed()
                expect(page.locator('#source-editor')).to_be_in_viewport(ratio=.7)
                capture(page, out, report, f'after-workbench-{course}-editor-{width}x{height}.png')
    finally:
        session.detach()
        viewport(page, *SIZES[0])


def run_flow(page, base, tmp, out, report, expect, exe):
    loc = page.locator
    def choose(course):
        loc(f'#course-list [data-course="{course}"]').click()
    def step(number):
        loc(f'#step-list [data-step="{number}"]').click()
    def course_data(course, number=1):
        return get_json(page, base, '/api/course', id=course, step=number)
    def prose(value):
        # The renderer turns lesson Markdown code spans into <code> elements.
        return re.sub(r'`([^`\n]+)`', r'\1', value)
    def check_card(course, number):
        card = course_data(course, number)
        expect(loc('#step-title')).to_contain_text(card['step_title'])
        expect(loc('#lesson-goal')).to_have_text(prose(card['goal']))
        expect(loc('#lesson-function')).to_have_text(prose(card['function']))
        expect(loc('#lesson-concept')).to_have_text(prose(card['concept']))
        expect(loc('#lesson-prediction')).to_have_text(prose(card['prediction']))
        expect(loc('#lesson-experiment')).to_have_text(prose(card['experiment']))
        expect(loc('#lesson-reflection')).to_have_text(prose(card['reflection']))
        expect(loc('#hint-list details')).to_have_count(3)

    page.goto(base+'/', wait_until='domcontentloaded')
    catalog = get_json(page, base, '/api/catalog')['courses']
    expected = {'firstcard', 'unitlab', 'quizdeck', 'textscope', 'ledgerlite', 'badgeforge', 'termgarden', 'batchmeter', 'minikv', 'netfetch', 'servicedesk', 'storedesk', 'asyncboard', 'book-d1', 'book-d2', 'book-d3', 'book-d4', 'book-d5', 'book-d6', 'book-foundations'}
    assert {c['id'] for c in catalog} == expected and len(catalog) == len(expected)
    expect(loc('#course-list [data-course]')).to_have_count(len(expected))
    for item in catalog:
        course = item['id']
        choose(course)
        expect(loc('#step-list [data-step]')).to_have_count(len(item['steps']))
        for number in range(1, len(item['steps'])+1):
            step(number)
            check_card(course, number)
    report['observations'].append('all-course-cards')

    # The Book bridge is real server data, not a decorative link.
    choose('unitlab')
    bridge = course_data('unitlab', 1)['reading']
    assert bridge['topics'] and bridge['version_note']
    expect(loc('#book-reading')).to_be_visible()
    loc('#book-reading summary').click()
    expect(loc('#book-reading-topics a').first).to_have_attribute(
        'href', re.compile(r'^https://(rust\.bootcss\.com|doc\.rust-lang\.org)/'))
    capture(page, out, report, 'book-link-1440x1000.png')

    # Local lessons remain readable without selecting a project or changing code.
    loc('#book-open').click()
    expect(loc('#book-dialog')).to_be_visible()
    loc('#book-select').select_option('d1')
    expect(loc('#book-text')).to_contain_text('Cargo')
    loc('#book-select').select_option('gap31')
    expect(loc('#book-text')).to_contain_text('31 项')
    loc('#book-select').select_option('evidence')
    expect(loc('#book-text')).to_contain_text('学员')
    capture(page, out, report, 'book-reader-1440x1000.png')
    viewport(page, 320, 844)
    expect(loc('#book-close')).to_be_visible()
    capture(page, out, report, 'book-reader-320x844.png')
    loc('#book-close').click()
    viewport(page, *SIZES[0])
    report['observations'].append('book-learning-bridge')

    choose('firstcard')
    step(1)
    loc('#lesson-open').click()
    expect(loc('#lesson-dialog')).to_be_visible()
    expect(page.frame_locator('#lesson-frame').locator('body')).to_contain_text('任务卡')
    loc('#lesson-dialog button').click()
    for detail in loc('#hint-list details').all():
        detail.locator('summary').click()
        expect(detail).to_have_attribute('open', '')
    report['observations'].append('full-lesson-and-hints')
    choose('unitlab')
    choose('firstcard')
    loc('#create-button').click()
    editor = loc('#source-editor')
    expect(editor).to_have_value(re.compile('等待完成第一张任务卡'))
    initial = editor.input_value()
    project = Path(course_data('firstcard')['project']['path'])
    compare_layout(page, expect, out, report, 'firstcard')

    def action(name, *, pending=False, mobile_pending=False):
        course, kind, number, result = ACTIONS[name]
        button = loc('#'+kind+'-button')
        with page.expect_response(lambda r: r.url == base+'/api/action' and r.request.method == 'POST') as response:
            button.click()
        assert response.value.ok, response.value.text()
        started = response.value.json()
        assert started['status'] == 'running' and started['action'] == kind
        if pending or mobile_pending:
            expect(button).to_be_disabled()
            expect(button).to_be_in_viewport()
            expect(loc('#console-summary')).to_have_attribute('data-state', 'pending')
            capture(page, out, report, 'mobile-pending.png' if mobile_pending else 'pending.png')
            if mobile_pending:
                viewport(page, *SIZES[0])
        expect(loc('#console-summary')).to_have_attribute(
            'data-state', 'success' if result=='succeeded' else 'error', timeout=180_000)
        job = get_json(page, base, '/api/job', id=started['id'])
        assert (job['course_id'], job['action'], job['step'], job['status']) == ACTIONS[name], job
        assert job['stale'] is False and (job['exit_code']==0)==(result=='succeeded'), job
        assert isinstance(job['exit_code'], int), job
        console = loc('#console-output').inner_text()
        # The visible dock must carry the actual backend output.
        assert job['output'].strip() in console, (job, console)
        report['actions'][name] = job
        if name in STATE_SHOTS:
            capture(page, out, report, name+'.png')
        return job

    action('firstcard-initial', pending=True)
    fixed = initial.replace('println!("等待完成第一张任务卡");', 'println!("今日任务卡 | {name} | {task}");')
    assert fixed != initial
    editor.fill(fixed)
    expect(loc('#save-status')).to_have_attribute('data-state', 'dirty')
    loc('#save-button').click()
    expect(loc('#save-status')).to_have_attribute('data-state', 'saved')
    assert (project/'src/main.rs').read_text(encoding='utf-8') == fixed
    page.reload()
    expect(editor).to_have_value(fixed)
    report['observations'].append('save-and-reload')
    action('firstcard-pass')
    loc('#run-settings summary').click()
    loc('#run-args').fill('林 果\n整理桌面')
    loc('#run-settings summary').click()
    personal = action('firstcard-personal')
    assert '今日任务卡 | 林 果 | 整理桌面' in personal['output']
    # Import a real project created outside the browser with an older-style
    # document set. Opening must not regenerate the source or deleted README.
    imported = tmp/'已有 UnitLab 中文'
    subprocess.run([str(exe), 'studio', 'start', 'unitlab', str(imported)], cwd=tmp, check=True,
                   capture_output=True, timeout=30)
    (imported/'README.md').unlink()
    sourcefile = imported/'src/lib.rs'
    starter = sourcefile.read_text(encoding='utf-8')
    choose('unitlab')
    loc('#project-path').fill(str(imported))
    loc('#open-project-button').click()
    expect(editor).to_have_value(starter)
    assert not (imported/'README.md').exists() and sourcefile.read_text(encoding='utf-8') == starter
    report['observations'].append('existing-project')
    compare_layout(page, expect, out, report, 'unitlab')
    report['observations'].append('desktop-mobile-layout')
    viewport(page, 320, 844)
    action('unitlab-initial', mobile_pending=True)
    unit = starter.replace('let _ = celsius;\n    0.0', 'celsius * 9.0 / 5.0 + 32.0')
    assert unit != starter
    editor.fill(unit)
    # Checking a dirty draft must save it first.
    action('unitlab-step1')
    assert sourcefile.read_text(encoding='utf-8') == unit
    step(2)
    action('unitlab-step2')
    history = course_data('unitlab', 2)['project']['history']
    assert history and history['same_source'] and history['passed_steps'] == 1, history
    page.reload()
    check_card('unitlab', 2)
    expect(editor).to_have_value(unit)
    capture(page, out, report, 'restored-selection.png')
    step(1)
    broken = unit + '\nfn rustapplab_broken( {\n'
    editor.fill(broken)
    failed = action('unitlab-compiler')
    assert 'unclosed delimiter' in failed['output'] and 'src/lib.rs' in failed['output'].replace('\\', '/')
    loc('#console-toggle').click()
    expect(loc('#console-panel')).to_have_attribute('data-mode', 'collapsed')
    expect(loc('#console-summary')).to_have_attribute('data-state', 'error')
    expect(loc('#console-summary')).to_be_visible()
    capture(page, out, report, 'collapsed-error.png')
    editor.fill(unit)
    action('unitlab-recovered')
    # Retain a dirty editor when a learner cancels navigation.
    draft = unit+'\n// 浏览器里的草稿\n'
    editor.fill(draft)
    dialogs = []
    def dismiss(dialog):
        dialogs.append(dialog.message)
        dialog.dismiss()
    page.once('dialog', dismiss)
    choose('firstcard')
    assert dialogs, 'dirty navigation did not ask before discarding the draft'
    expect(editor).to_have_value(draft)
    report['observations'].append('dirty-navigation')
    external = unit+'\n// 另一个编辑器的修改\n'
    sourcefile.write_text(external, encoding='utf-8', newline='\n')
    with page.expect_response(lambda r: r.url == base+'/api/save') as conflict:
        loc('#save-button').click()
    assert conflict.value.status == 409
    expect(loc('#reload-file-button')).to_be_visible()
    expect(editor).to_have_value(draft)
    assert sourcefile.read_text(encoding='utf-8') == external
    capture(page, out, report, 'external-conflict.png')
    page.once('dialog', lambda d: d.accept())
    loc('#reload-file-button').click()
    expect(editor).to_have_value(external)
    report['observations'].append('external-conflict')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--exe', type=Path, default=ROOT/'target/release'/('labquest.exe' if os.name=='nt' else 'labquest'))
    parser.add_argument('--baseline', default=os.environ.get('WORKBENCH_BASE_REVISION') or PRE_WORKBENCH)
    parser.add_argument('--browser-path')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    exe = args.exe.resolve()
    report = {'schema': 1, 'source_id': source_id(), 'status': 'RUNNING', 'actions': {},
              'screenshots': {}, 'observations': [], 'javascript_errors': [],
              'comparison': 'before-static = original START_HERE / UnitLab lesson; after-workbench = real embedded learning interface'}
    try:
        from playwright.sync_api import expect, sync_playwright
        with tempfile.TemporaryDirectory(prefix='workbench-e2e-') as name, sync_playwright() as pw:
            tmp = Path(name)
            rev, pages = baseline_pages(tmp, args.baseline)
            report['baseline_revision'] = rev
            report['baseline_pages'] = {k: {p: v[p] for p in ('source', 'sha256')} for k,v in pages.items()}
            browser = pw.chromium.launch(headless=True, executable_path=find_browser(args.browser_path))
            try:
                page = browser.new_page(viewport={'width':1440,'height':1000}, locale='zh-CN')
                report['browser_version'] = browser.version
                page.on('pageerror', lambda e: report['javascript_errors'].append(str(e)))
                for course, old in pages.items():
                    page.goto(old['url'], wait_until='load')
                    expect(page.locator('h1').first).to_be_visible()
                    for w,h in SIZES:
                        viewport(page, w, h)
                        capture(page, out, report, f'before-static-{course}-{w}x{h}.png')
                viewport(page, *SIZES[0])
                workspace = tmp/'学习 路线'
                workspace.mkdir()
                cmd = [str(exe), 'workbench', '--workspace', str(workspace), '--port']
                with running_workbench([*cmd,'0'], tmp, out/'server.txt') as base:
                    run_flow(page, base, tmp, out, report, expect, exe)
                    report['book_practice'] = run_book_flow(page, base, tmp, out, expect)
                # Refuse a real connection while preserving the page. Restart
                # the real server on the same URL before retrying navigation.
                page.locator('#course-list [data-course="quizdeck"]').click()
                expect(page.locator('#retry-button')).to_be_visible(timeout=15_000)
                with running_workbench([*cmd,str(urlparse(base).port)], tmp, out/'restarted-server.txt'):
                    page.locator('#retry-button').click()
                    expect(page.locator('#lesson-goal')).to_have_text(
                        get_json(page, base, '/api/course', id='quizdeck', step=1)['goal'].replace('`', ''))
                report['observations'].append('disconnect-retry')
            finally:
                browser.close()
        assert not report['javascript_errors'], report['javascript_errors']
        assert set(report['screenshots']) == set(screenshot_sizes())
        assert set(report['observations']) == OBSERVATIONS
        assert set(report['actions']) == set(ACTIONS)
        report['source_id_after'] = source_id()
        assert report['source_id'] == report['source_id_after'], 'source changed while browser was running'
        report['status'] = 'PASS'
    except Exception as exc:
        import traceback
        report['error'] = f'{type(exc).__name__}: {exc}'
        report['source_id_after'] = source_id()
        report['status'] = 'FAIL'
        traceback.print_exc()
    finally:
        (out/'workbench.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    return int(report['status'] != 'PASS')


if __name__ == '__main__':
    raise SystemExit(main())
