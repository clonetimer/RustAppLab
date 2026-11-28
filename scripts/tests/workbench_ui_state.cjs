const assert = require('node:assert/strict');
const { existsSync, readFileSync } = require('node:fs');
const { resolve } = require('node:path');
const { test } = require('node:test');
const vm = require('node:vm');

const app = resolve(__dirname, '../../apps/labquest/src/workbench/ui/app.js');

function loadApp(fetch = () => assert.fail('unexpected request'), bindings = {}) {
  assert.ok(existsSync(app), 'expected the offline workbench script to exist');
  const context = vm.createContext({
    document: { addEventListener() {} },
    fetch,
    URLSearchParams,
    AbortController,
    setTimeout,
    clearTimeout,
    ...bindings,
  });
  for (const asset of ['knowledge.js', 'experiments.js', 'app.js']) {
    const path = resolve(__dirname, '../../apps/labquest/src/workbench/ui', asset);
    vm.runInContext(readFileSync(path, 'utf8'), context, { filename: path });
  }
  return context;
}

test('API requests send literal arguments and the workbench header', async () => {
  const calls = [];
  const context = loadApp(async (url, options) => {
    calls.push({ url, options });
    return { ok: true, json: async () => ({ id: 381 }) };
  });
  const job = await context.api('/api/action', { form: {
    id: 'firstcard', action: 'run', step: 1, args: ' 梨 花 \n写 Rust & 去散步',
  } });
  await context.api('/api/job', { query: { id: job.id } });
  assert.deepEqual(calls.map(call => call.url), ['/api/action', '/api/job?id=381']);
  assert.equal(calls[0].options.body.get('args'), ' 梨 花 \n写 Rust & 去散步');
  assert.equal(calls[0].options.method, 'POST');
  assert.equal(calls[0].options.headers['Content-Type'], 'application/x-www-form-urlencoded;charset=UTF-8');
  assert.ok(calls.every(call => call.options.headers['X-RustAppLab'] === 'workbench'));
});

test('save conflict keeps the server status and human-readable error', async () => {
  const context = loadApp(async () => ({
    ok: false, status: 409, json: async () => ({ error: '文件在外部修改过' }),
  }));
  await assert.rejects(
    context.api('/api/save', { form: { id: 'unitlab', path: 'src/lib.rs', content: 'draft', revision: '1' } }),
    { status: 409, message: '文件在外部修改过' },
  );
});

test('file requests stay bound to the project shown in the browser', async () => {
  const context = loadApp(async url => ({ ok: true, json: async () => url }));
  const projectPath = 'D:\\学习 Rust\\换算练习';
  const request = await context.getFile({ id: 'unitlab', project: { path: projectPath } }, 'src/lib.rs');
  const url = new URL(request, 'http://127.0.0.1:7777');
  assert.equal(url.pathname, '/api/file');
  assert.equal(url.searchParams.get('id'), 'unitlab');
  assert.equal(url.searchParams.get('path'), 'src/lib.rs');
  assert.equal(url.searchParams.get('project_path'), projectPath);
});

const card = {
  id: 'unitlab', step: 1, step_count: 3, next_course: 'quizdeck',
  project: { source_digest: 'new', suggested_step: 2 },
};
const check = {
  id: 41, course_id: 'unitlab', action: 'check', step: 1, status: 'succeeded',
  output: 'real output', exit_code: 0, source_digest: 'new', stale: false,
};

test('focused route keeps catalog order without crossing learning stages', () => {
  const { courseFocusIds } = loadApp();
  const courses = [
    { id: 'firstcard', kind: 'studio' }, { id: 'unitlab', kind: 'studio' },
    { id: 'quizdeck', kind: 'studio' }, { id: 'book-d1', kind: 'book' },
    { id: 'book-d2', kind: 'book' }, { id: 'modern-d8', kind: 'book' },
    { id: 'modern-d9a', kind: 'book' },
  ];
  const focus = id => Array.from(courseFocusIds(courses, id));
  assert.deepEqual(focus('firstcard'), ['firstcard', 'unitlab', 'quizdeck']);
  assert.deepEqual(focus('quizdeck'), ['quizdeck']);
  assert.deepEqual(focus('book-d1'), ['book-d1', 'book-d2']);
  assert.deepEqual(focus('book-d2'), ['book-d2']);
  assert.deepEqual(focus('modern-d8'), ['modern-d8', 'modern-d9a']);
  assert.deepEqual(focus('missing'), ['firstcard', 'unitlab', 'quizdeck']);
  assert.deepEqual(Array.from(courseFocusIds([], 'firstcard')), []);
});

test('foundation previews distinguish full software examples from current-step recaps', () => {
  const { foundationPreview, lessonRecap } = loadApp();
  assert.equal(foundationPreview('firstcard').output, '今日任务卡 | 小林 | 写 Rust');
  assert.equal(foundationPreview('unitlab').output, '25.00 c = 77.00 f');
  assert.match(foundationPreview('unitlab').note, /全部 3 步/);
  assert.match(foundationPreview('quizdeck').output, /score=2\/3/);
  assert.equal(foundationPreview('modern-d8'), null);
  assert.equal(foundationPreview('constructor'), null);
  assert.match(lessonRecap({ id: 'firstcard', step: 1 }), /没有独立实现参数解析/);
  assert.match(lessonRecap({ id: 'unitlab', step: 1 }), /还未完成/);
  assert.match(lessonRecap({ id: 'quizdeck', step: 3 }), /检查答案数量/);
  assert.equal(lessonRecap({ id: 'book-d1', step: 1, concept: 'Cargo 工作区' }), 'Cargo 工作区');
});

test('a run or a stale check leaves the course without a current check', () => {
  const { nextDestination } = loadApp();
  for (const result of [
    { ...check, action: 'run' },
    { ...check, stale: true },
    { ...check, source_digest: 'previous' },
    { ...check, course_id: 'firstcard' },
    { ...check, status: 'failed', exit_code: 1 },
  ]) {
    assert.equal(nextDestination(card, result, false), null);
  }
});

test('fresh cumulative checks offer the next stage or course for the selected stage', () => {
  const { nextDestination } = loadApp();
  assert.equal(nextDestination(card, check, false).step, 2);
  assert.equal(nextDestination(card, check, true), null);
  assert.equal(nextDestination({ ...card, step: 2 }, check, false), null);
  assert.equal(nextDestination({ ...card, step: 3 }, { ...check, step: 3 }, false).course, 'quizdeck');
});

test('restored lessons use only a matching cumulative successful history', () => {
  const { restoredLessonPass } = loadApp();
  const classic = { ...card, kind: 'studio', project: { ...card.project, history: {
    passed_steps: 1, upto_step: 1, same_source: true, compilation_failed: false,
  } } };
  assert.equal(restoredLessonPass(classic, null, false, false), true);
  assert.equal(restoredLessonPass(classic, check, false, false), false);
  assert.equal(restoredLessonPass(classic, null, true, false), false);
  assert.equal(restoredLessonPass(classic, null, false, true), false);
  assert.equal(restoredLessonPass({ ...classic, step: 2 }, null, false, false), false);
  assert.equal(restoredLessonPass({ ...classic, id: 'modern-d8' }, null, false, false), false);
  assert.equal(restoredLessonPass({ ...classic, kind: 'book' }, null, false, false), false);
  for (const history of [{ same_source: false }, { compilation_failed: true }, { upto_step: 2 }]) {
    const project = { ...classic.project, history: { ...classic.project.history, ...history } };
    assert.equal(restoredLessonPass({ ...classic, project }, null, false, false), false);
  }
});

test('navigation can recommend a later step only within the verified scope', () => {
  const { nextDestination } = loadApp();
  const withSuggestion = { ...card, project: { ...card.project, suggested_step: 3 } };
  assert.equal(nextDestination(withSuggestion, { ...check, step: 2 }, false).step, 3);
  assert.equal(nextDestination(withSuggestion, check, false).step, 2);
});

function primeEditor(context, original, draft = '保存版本') {
  Object.assign(context, { original, draft });
  vm.runInContext(`
    const elements = {};
    Object.setPrototypeOf(ui, new Proxy(elements, {
      get(target, key) {
        return target[key] ??= {
          dataset: {}, querySelectorAll: () => [], setAttribute() {}, replaceChildren() {}, add() {},
        };
      },
    }));
    state.catalog = { courses: [{ id: 'unitlab', default_file: 'src/lib.rs' }] };
    state.card = original;
    state.file = { path: 'src/lib.rs', content: '保存版本', revision: 'A' };
    ui.editor.value = draft;
  `, context);
}

test('changing steps cannot silently discard a draft when another tab opened a different project', async () => {
  const original = { ...card, project: { ...card.project, path: '/projects/A' } };
  const replacement = {
    ...card, step: 2, title: 'UnitLab', step_title: '枚举与匹配', goal: '目标', function: 'convert',
    concept: '枚举', prediction: '预测', experiment: '实验', reflection: '反思', hints: [], run_args: [],
    project: { ...card.project, path: '/projects/B', files: ['src/lib.rs'] },
  };
  const requests = [];
  const confirmations = [];
  const context = loadApp(async url => {
    requests.push(url);
    return { ok: true, json: async () => url.startsWith('/api/course') ? replacement : {
      path: 'src/lib.rs', content: '另一工程的源码', revision: 'B',
    } };
  }, {
    window: { confirm: message => { confirmations.push(message); return false; } },
    Option: class {},
  });
  primeEditor(context, original, '还没有保存的草稿');
  await context.selectCourse('unitlab', 2);
  assert.equal(confirmations.length, 1);
  assert.equal(vm.runInContext('state.card.project.path', context), '/projects/A');
  assert.equal(vm.runInContext('state.card.step', context), 1);
  assert.equal(vm.runInContext('ui.editor.value', context), '还没有保存的草稿');
  assert.equal(vm.runInContext('dirty()', context), true);
  assert.deepEqual(requests, ['/api/course?id=unitlab&step=2']);
});

test('opening an externally edited source file makes old check results stale', async () => {
  const original = { ...card, project: {
    ...card.project, path: '/projects/A',
    history: { passed_steps: 1, upto_step: 1, same_source: true, compilation_failed: false },
  } };
  const context = loadApp(async request => {
    const path = new URL(request, 'http://127.0.0.1:7777').searchParams.get('path');
    const file = path === 'LEARNING_LOG.md'
      ? { path, content: '日志', revision: 'notes', source_digest: 'new' }
      : { path, content: '外部改坏的 Rust', revision: 'B', source_digest: 'edited' };
    return { ok: true, json: async () => file };
  }, { check });
  primeEditor(context, original);
  vm.runInContext('state.lastCheck = check; state.output = { job: check }; renderControls()', context);
  assert.equal(vm.runInContext('ui.consoleSummary.dataset.state', context), 'success');

  await context.selectFile('LEARNING_LOG.md');
  assert.equal(vm.runInContext('ui.next.hidden', context), false);
  await context.selectFile('src/lib.rs');
  assert.equal(vm.runInContext('ui.editor.value', context), '外部改坏的 Rust');
  assert.equal(vm.runInContext('ui.consoleSummary.dataset.state', context), 'warning');
  assert.equal(vm.runInContext('ui.next.hidden', context), true);
  assert.match(vm.runInContext('ui.history.textContent', context), /重新检查/);
});

test('a check of newer disk content never verifies an older open editor buffer', async () => {
  const original = { ...card, project: { ...card.project, path: '/projects/A' } };
  const context = loadApp(async url => ({ ok: true, json: async () => url.startsWith('/api/course')
    ? structuredClone(original)
    : { path: 'src/lib.rs', content: '磁盘上刚修好的版本', revision: 'B', source_digest: 'new' },
  }));
  // The card already knows the latest project digest after a step change,
  // but the open textarea still holds an earlier file revision.
  primeEditor(context, original);
  await context.finishJob(check);
  vm.runInContext('renderControls()', context);
  assert.equal(vm.runInContext('ui.editor.value', context), '保存版本');
  assert.equal(vm.runInContext('ui.consoleSummary.dataset.state', context), 'warning');
  assert.equal(vm.runInContext('ui.reloadFile.hidden', context), false);
  assert.equal(vm.runInContext('ui.next.hidden', context), true);
});

test('returning to a workbench tab refreshes old success without replacing the open buffer', async () => {
  const original = { ...card, project: { ...card.project, path: '/projects/A' } };
  const requests = [];
  const context = loadApp(async url => {
    requests.push(url);
    return { ok: true, json: async () => ({
      path: 'src/lib.rs', content: '另一标签保存的坏代码', revision: 'B', source_digest: 'edited',
    }) };
  }, { check });
  primeEditor(context, original);
  vm.runInContext('state.lastCheck = check; state.output = { job: check }; renderControls()', context);
  assert.equal(vm.runInContext('ui.consoleSummary.dataset.state', context), 'success');
  assert.equal(vm.runInContext('ui.next.hidden', context), false);

  await context.recheckOnFocus?.();
  assert.equal(vm.runInContext('ui.consoleSummary.dataset.state', context), 'warning');
  assert.equal(vm.runInContext('ui.next.hidden', context), true);
  assert.equal(vm.runInContext('ui.reloadFile.hidden', context), false);
  assert.equal(vm.runInContext('ui.editor.value', context), '保存版本');
  assert.equal(requests.length, 1);
  assert.match(requests[0], /^\/api\/file\?/);
});

test('teaching feedback distinguishes behavior, real errors, and unknown failures', () => {
  const { teachingFailure } = loadApp();
  const lesson = { id: 'firstcard', project: { files: ['src/main.rs'] } };
  const failure = { action: 'check', status: 'failed', exit_code: 2 };
  const guide = output => teachingFailure(lesson, { ...failure, output });
  assert.equal(guide('期待：正确\n实际：占位').title, '输出还不符合任务要求');
  const typed = guide('error[E0308]: mismatched types\n --> C:\\tmp\\src\\main.rs:4:9\n');
  assert.match(typed.title, /E0308/);
  assert.equal(typed.location, 'src/main.rs · 第 4 行，第 9 列');
  assert.match(typed.reason, /类型/);
  assert.match(guide('error: unclosed delimiter\n --> src/main.rs:3:5').title, /括号/);
  assert.equal(guide('error[E9999]: unknown\n --> other.rs:2:1').location, '');
  assert.match(guide('test result: FAILED. 0 passed').title, /检查要求/);
  assert.match(guide('random output').reason, /不足以/);
  assert.match(teachingFailure(lesson, { ...failure, exit_code: null }).title, /未能完成/);
  assert.equal(teachingFailure(lesson, { ...failure, action: 'run', output: 'error[E0308]: example' }).title, '程序运行未成功');
  assert.equal(teachingFailure(lesson, { ...failure, status: 'succeeded' }), null);
});

test('FirstCard reload trusts only a current server check, not a stored completion flag', async () => {
  const original = { ...card, id: 'firstcard', step_count: 1, next_course: 'unitlab',
    project: { path: '/projects/A', source_digest: 'new' } };
  const saved = { id: 'firstcard', projectPath: '/projects/A', checkJob: 41 };
  const first = { ...check, course_id: 'firstcard' };
  for (const [job, expected] of [
    [first, true], [{ ...first, stale: true }, false],
    [{ ...first, source_digest: 'different' }, false],
    [{ ...first, action: 'run' }, false],
    [{ ...first, exit_code: 2, status: 'failed' }, false],
    [{ ...first, id: 42 }, false],
  ]) {
    const context = loadApp(async url => {
      assert.equal(url, '/api/job?id=41');
      return { ok: true, json: async () => job };
    });
    primeEditor(context, original);
    assert.equal(await context.restoreFirstCardCheck(saved), expected);
  }
  const expired = loadApp(async () => ({ ok: false, status: 404, json: async () => ({ error: 'missing' }) }));
  primeEditor(expired, original);
  assert.equal(await expired.restoreFirstCardCheck(saved), false);
  const wrongProject = loadApp();
  primeEditor(wrongProject, original);
  assert.equal(await wrongProject.restoreFirstCardCheck({ ...saved, projectPath: '/projects/B' }), false);
});

test('book navigation has a bounded allowlist and diagnostics never pretend to fix code', () => {
  const context = loadApp();
  const docs = context.compilerDiagnostics([
    'error[E0382]: borrow of moved value',
    'error[E0502]: cannot borrow as mutable',
    'error[E0382]: borrow of moved value',
    'error[E9999]: unknown code',
  ].join('\n'));
  assert.deepEqual(Array.from(docs, item => item.code), ['E0382', 'E0502']);
  assert.ok(docs.every(item => item.url.startsWith('https://doc.rust-lang.org/')));
  assert.equal(context.compilerDiagnostics('test failed: assertion failed').length, 0);
  assert.equal(context.compilerDiagnostics('error[E9999] unknown').length, 0);
});

test('compiler help ignores error-like text in program output and links supported diagnostics to experiments', () => {
  const context = loadApp();
  assert.equal(context.compilerDiagnostics('printed E0382 in my program').length, 0);
  assert.equal(context.compilerDiagnostics('error[E0425]: missing variable\n  | println!("E0382");').length, 1);
  const docs = context.compilerDiagnostics('error[E0597]: borrowed value does not live long enough');
  assert.equal(docs[0].experiment, 'E0597');
  assert.equal(docs[0].lesson, 'generics');
});

test('knowledge navigation searches Chinese terms, Rust names and compiler codes', () => {
  const context = loadApp();
  assert.ok(context.knowledgeMatches('HashMap').some(item => item.id === 'collections'));
  assert.ok(context.knowledgeMatches('借用').some(item => item.id === 'ownership'));
  assert.ok(context.knowledgeMatches('E0502').some(item => item.id === 'ownership'));
  assert.equal(context.knowledgeMatches('does-not-exist').length, 0);
  assert.equal(context.bookCourseFor('generics'), 'book-foundations');
  assert.equal(context.bookCourseFor('d6a'), 'book-d6');
  assert.equal(context.bookCourseFor('evidence'), null);
});

test('knowledge and experiment actions unlock when a project check finishes', () => {
  const context = loadApp();
  primeEditor(context, { ...card, project: { ...card.project, path: '/projects/A' } });
  vm.runInContext(`
    state.busy = 'check';
    experimentState.code = 'E0382';
    ui.bookPractice.disabled = true;
    experimentControls();
  `, context);
  assert.equal(vm.runInContext('ui.experimentFailure.disabled', context), true);
  vm.runInContext("state.busy = null; renderControls()", context);
  assert.equal(vm.runInContext('ui.bookPractice.disabled', context), false);
  assert.equal(vm.runInContext('ui.experimentFailure.disabled', context), false);
  assert.equal(vm.runInContext('ui.experimentRepair.disabled', context), false);
});

test('modern step-local checks cannot imply a different step passed', () => {
  const { nextDestination } = loadApp();
  const modern = { ...card, id: 'modern-e5', step_count: 5, next_course: null };
  const result = { ...check, course_id: modern.id, step: 3 };
  assert.equal(nextDestination(modern, result, false), null);
  assert.equal(nextDestination({ ...modern, step: 3 }, result, false).step, 4);
  assert.equal(nextDestination({ ...modern, step: 5 }, { ...result, step: 5 }, false).experiment, true);
});
