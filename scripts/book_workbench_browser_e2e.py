#!/usr/bin/env python3
"""Real browser acceptance for local Book learning, learner copies and diagnostics.

The existing workbench runner may import ``run_book_flow``. This module also runs
independently against a freshly created workspace; every action is a real UI
click and every result is checked against the corresponding server job.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile
import time
from urllib.parse import urlencode

from browser_e2e import find_browser
from release_check import ROOT
from workbench_process import running_workbench

BOOK_IDS = ('book-d1', 'book-d2', 'book-d3', 'book-d4', 'book-d5', 'book-d6',
            'book-foundations')
STUDIO_IDS = {'firstcard', 'unitlab', 'quizdeck', 'textscope', 'ledgerlite',
              'badgeforge', 'termgarden', 'batchmeter', 'minikv', 'netfetch',
              'servicedesk', 'storedesk', 'asyncboard'}
SIZES = ((1440, 1000), (390, 844), (320, 844))


def get_json(page, base, route, **params):
    url = base + route + ('?' + urlencode(params) if params else '')
    response = page.request.get(url)
    assert response.ok, (url, response.status, response.text())
    return response.json()


def completed_job(page, base, route, job_id, timeout=185):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = get_json(page, base, route, id=job_id)
        if result['status'] != 'running':
            return result
        time.sleep(.05)
    raise TimeoutError(f'{route} job {job_id} did not finish within {timeout}s')


def run_book_flow(page, base, tmp, out, expect):
    """Exercise the actual UI; write independent evidence below out/book-practice.

    ``tmp`` is the existing runner's temporary root. No repository source file is
    edited: files resolved from the API must be inside that temporary root.
    """
    dest = Path(out) / 'book-practice'
    dest.mkdir(parents=True, exist_ok=False)
    tmp = Path(tmp).resolve()
    report = {'schema': 1, 'status': 'RUNNING', 'checks': {}, 'experiments': {},
              'projects': {}, 'screenshots': {}, 'observations': []}
    loc = page.locator

    def card(course, step=1):
        return get_json(page, base, '/api/course', id=course, step=step)

    def capture(name):
        path = dest / name
        page.screenshot(path=str(path), animations='disabled')
        report['screenshots'][name] = {
            'viewport': page.viewport_size,
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        }

    def views(name, focus):
        session = page.context.new_cdp_session(page)
        try:
            for width, height in SIZES:
                page.set_viewport_size({'width': width, 'height': height})
                loc(focus).scroll_into_view_if_needed()
                expect(loc(focus)).to_be_in_viewport()
                actual = session.send('Page.getLayoutMetrics')['cssContentSize']['width']
                assert actual <= width + 1, (name, width, actual)
                capture(f'{name}-{width}x{height}.png')
        finally:
            session.detach()
            page.set_viewport_size({'width': 1440, 'height': 1000})

    def choose(course):
        loc(f'#course-list [data-course="{course}"]').click()
        expect(loc('#course-title')).to_have_text(card(course)['title'])
        expect(loc('#step-list [data-step]')).to_have_count(3)

    def create(course):
        current = card(course)
        assert current['kind'] == 'book' and current['project'] is None, current
        with page.expect_response(lambda response: response.url == base + '/api/create'
                                  and response.request.method == 'POST') as response:
            loc('#create-button').click()
        assert response.value.ok, response.value.text()
        created = response.value.json()
        project = created['project']
        root = Path(project['path']).resolve()
        assert root.is_relative_to(tmp), ('learner project escaped test workspace', root)
        assert project['history'] is None, project
        assert 'LEARNING_LOG.md' in project['files'], project
        assert not any(path.endswith('Cargo.toml') or '/tests/' in path
                       for path in project['files']), project
        default = next(item['default_file'] for item in catalog if item['id'] == course)
        expect(loc('#file-select')).to_have_value(default)
        expect(loc('#source-editor')).to_have_value((root / default).read_text(encoding='utf-8'))
        if created['can_run']:
            expect(loc('#run-button')).to_be_visible()
        else:
            expect(loc('#run-button')).to_be_hidden()
        report['projects'][course] = {'path': str(root), 'default_file': default,
                                     'editable_files': project['files'],
                                     'source_digest': project['source_digest']}
        return created

    def check(course, name, *, expected='succeeded', step=1):
        with page.expect_response(lambda response: response.url == base + '/api/action'
                                  and response.request.method == 'POST') as response:
            loc('#check-button').click()
        assert response.value.ok, response.value.text()
        started = response.value.json()
        assert started['course_id'] == course and started['action'] == 'check', started
        job = completed_job(page, base, '/api/job', started['id'])
        assert (job['course_id'], job['action'], job['step'], job['status']) == (
            course, 'check', step, expected), job
        assert isinstance(job['exit_code'], int), job
        assert (job['exit_code'] == 0) == (expected == 'succeeded'), job
        assert job['stale'] is False, job
        expect(loc('#console-output')).to_contain_text(job['output'].strip(), timeout=10_000)
        expect(loc('#console-summary')).to_have_attribute(
            'data-state', 'success' if expected == 'succeeded' else 'error')
        assert '--locked' in job['output'] and '--offline' in job['output'], job
        if expected == 'succeeded':
            assert 'test result: ok' in job['output'] and 'Doc-tests' in job['output'], job
        assert card(course, step)['project']['history'] is None
        report['checks'][name] = job
        return job

    def open_topic(topic):
        loc('#book-open').click()
        expect(loc('#book-dialog')).to_be_visible()
        loc('#knowledge-search').fill(topic)
        control = loc(f'#knowledge-results [data-topic="{topic}"]')
        expect(control).to_be_visible()
        control.click()
        expect(loc('#book-select')).to_have_value(topic)
        material = get_json(page, base, '/api/book', id=topic)
        expect(loc('#book-status')).to_have_text(material['title'])
        expect(loc('#book-text')).not_to_be_empty()

    def experiment(variant):
        with page.expect_response(lambda response: response.url == base + '/api/experiment'
                                  and response.request.method == 'POST') as response:
            loc('#experiment-' + variant).click()
        assert response.value.ok, response.value.text()
        started = response.value.json()
        assert started['code'] == 'E0382' and started['variant'] == variant, started
        job = completed_job(page, base, '/api/experiment-job', started['id'], timeout=45)
        assert (job['id'], job['code'], job['variant'], job['status'], job['matched']) == (
            started['id'], 'E0382', variant, 'completed', True), job
        if variant == 'failure':
            assert isinstance(job['compiler_exit_code'], int) and job['compiler_exit_code'] != 0, job
            assert job['program_exit_code'] is None, job
            assert re.search(r'^error\[E0382\]:', job['output'], re.M), job
        else:
            assert job['compiler_exit_code'] == 0 and job['program_exit_code'] == 0, job
        expect(loc('#experiment-output')).to_have_text(job['output'], timeout=10_000)
        expect(loc('#experiment-status')).to_have_attribute('data-state', 'success')
        report['experiments'][variant] = job
        return job

    try:
        page.goto(base + '/', wait_until='domcontentloaded')
        page.set_viewport_size({'width': 1440, 'height': 1000})
        catalog = get_json(page, base, '/api/catalog')['courses']
        expected = STUDIO_IDS | set(BOOK_IDS) | {"modern-d8", "modern-d9a", "modern-d9b", "modern-d10a", "modern-d11a", "modern-e4", "modern-e5"}
        assert {item['id'] for item in catalog} == expected and len(catalog) == len(expected)
        expect(loc('#course-list [data-course]')).to_have_count(len(expected))
        report['catalog_ids'] = [item['id'] for item in catalog]

        choose('book-d1')
        first = create('book-d1')
        editor = loc('#source-editor')
        original = editor.input_value()
        draft = original + '\n// 自动验收草稿：取消导航时应保留。\n'
        editor.fill(draft)
        expect(loc('#save-status')).to_have_attribute('data-state', 'dirty')
        loc('#book-open').click()
        loc('#knowledge-search').fill('HashMap')
        expect(loc('#knowledge-results [data-topic="collections"]')).to_be_visible()
        loc('#knowledge-results [data-topic="collections"]').click()
        expect(loc('#book-select')).to_have_value('collections')
        expect(loc('#book-text')).to_contain_text('HashMap')
        expect(loc('#book-text pre code').first).to_be_visible()
        views('knowledge', '#knowledge-search')
        canceled = []

        def dismiss(dialog):
            canceled.append(dialog.message)
            dialog.dismiss()

        page.once('dialog', dismiss)
        loc('#book-practice').click()
        assert canceled, 'knowledge-to-project navigation discarded a dirty draft without confirmation'
        expect(loc('#book-dialog')).to_be_visible()
        expect(loc('#course-title')).to_have_text(first['title'])
        assert editor.input_value() == draft
        loc('#book-close').click()
        editor.fill(original)
        open_topic('d1')
        loc('#book-practice').click()
        expect(loc('#book-dialog')).to_be_hidden()
        expect(loc('#course-title')).to_have_text(first['title'])
        check('book-d1', 'book-d1')
        report['observations'].extend(['knowledge-search-and-rendering', 'dirty-draft-preserved',
                                       'knowledge-opens-learner-project'])

        for course in BOOK_IDS[1:]:
            topic = next(item['lesson_topic'] for item in catalog if item['id'] == course)
            open_topic(topic)
            loc('#book-practice').click()
            expect(loc('#book-dialog')).to_be_hidden()
            expect(loc('#course-title')).to_have_text(card(course)['title'])
            create(course)
            check(course, course)
        report['observations'].append('all-seven-book-workspaces-checked')

        foundation = report['projects']['book-foundations']
        source_path = Path(foundation['path']) / foundation['default_file']
        original = editor.input_value()
        before = card('book-foundations')['project']['source_digest']
        loc('#step-list [data-step="2"]').click()
        expect(loc('#step-title')).to_contain_text(card('book-foundations', 2)['step_title'])
        broken = original + '\nfn browser_acceptance_broken( {\n'
        editor.fill(broken)
        failed = check('book-foundations', 'foundation-syntax-failure', expected='failed', step=2)
        assert 'unclosed delimiter' in failed['output'], failed
        assert foundation['default_file'] in failed['output'].replace('\\', '/'), failed
        assert source_path.read_text(encoding='utf-8') == broken
        capture('foundation-syntax-failure-1440x1000.png')
        editor.fill(original)
        repaired = check('book-foundations', 'foundation-restored', step=2)
        assert repaired['source_digest'] == before
        assert source_path.read_text(encoding='utf-8') == original
        report['observations'].append('learner-source-save-failure-repair')

        loc('#learning-log-button').click()
        expect(loc('#file-select')).to_have_value('LEARNING_LOG.md')
        expect(editor).to_have_value(re.compile('预测'))
        note = editor.input_value() + '\n## 自动化验收记录\n本记录验证保存功能，不是对真实学员掌握程度的观察。\n'
        editor.fill(note)
        loc('#save-button').click()
        expect(loc('#save-status')).to_have_attribute('data-state', 'saved')
        assert (Path(foundation['path']) / 'LEARNING_LOG.md').read_text(encoding='utf-8') == note
        assert card('book-foundations', 2)['project']['source_digest'] == before
        loc('#file-select').select_option(foundation['default_file'])
        expect(editor).to_have_value(original)
        views('project', '#source-editor')
        report['observations'].append('learning-log-saved-without-fabricated-completion')

        loc('#book-open').click()
        loc('#book-error-experiments').click()
        expect(loc('#experiment-dialog')).to_be_visible()
        loc('#experiment-select').select_option('E0382')
        expect(loc('#experiment-status')).to_contain_text('E0382')
        loc('#experiment-prediction').fill('自动验收预测：title 被移入 Vec 后再次读取触发 E0382；修复后编译和断言通过。')
        failure = experiment('failure')
        capture('experiment-failure-1440x1000.png')
        repair = experiment('repair')
        assert failure['id'] != repair['id']
        views('experiment-repaired', '#experiment-status')
        loc('#experiment-close').click()
        expect(editor).to_have_value(original)
        assert source_path.read_text(encoding='utf-8') == original
        assert card('book-foundations', 2)['project']['source_digest'] == before
        report['observations'].append('isolated-compiler-failure-and-repair')
        report['status'] = 'PASS'
        return report
    except Exception as exc:
        report['status'] = 'FAIL'
        report['error'] = f'{type(exc).__name__}: {exc}'
        try:
            capture('failure.png')
        except Exception:
            pass
        raise
    finally:
        (dest / 'book-practice.json').write_text(
            json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        page.set_viewport_size({'width': 1440, 'height': 1000})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target' / 'release' /
                        ('labquest.exe' if os.name == 'nt' else 'labquest'))
    parser.add_argument('--browser-path')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    from playwright.sync_api import expect, sync_playwright
    with tempfile.TemporaryDirectory(prefix='book-browser-') as directory, sync_playwright() as pw:
        tmp = Path(directory)
        workspace = tmp / '学员 Book 实验'
        workspace.mkdir()
        command = [str(args.exe.resolve()), 'workbench', '--workspace', str(workspace), '--port', '0']
        with running_workbench(command, tmp, out / 'server.txt') as base:
            browser = pw.chromium.launch(headless=True, executable_path=find_browser(args.browser_path))
            try:
                page = browser.new_page(viewport={'width': 1440, 'height': 1000}, locale='zh-CN')
                errors = []
                page.on('pageerror', lambda error: errors.append(str(error)))
                result = run_book_flow(page, base, tmp, out, expect)
                assert not errors, errors
                print(json.dumps({'status': result['status'], 'checks': len(result['checks']),
                                  'screenshots': len(result['screenshots'])}, ensure_ascii=False))
            finally:
                browser.close()
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
