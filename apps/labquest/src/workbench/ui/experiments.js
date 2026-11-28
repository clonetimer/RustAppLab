'use strict';

const RUST_ERROR_GUIDES = Object.freeze({
  E0382: ['值被移动后再次使用', '检查所有权移交的位置，判断只读借用或调整使用顺序是否满足需求。', 'ch04-01-what-is-ownership.html', 'ownership'],
  E0502: ['可变借用与共享借用冲突', '定位两个引用实际使用的区间，检查修改是否发生在最后一次读取之前。', 'ch04-02-references-and-borrowing.html', 'ownership'],
  E0499: ['存在重叠的可变借用', '同一数据不能同时由两个活跃的可变引用修改。', 'ch04-02-references-and-borrowing.html', 'ownership'],
  E0308: ['类型不匹配', '代码给出的值与这里需要的类型不同。检查参数、分支、返回值，以及最后一行是否多写了分号。', 'ch03-02-data-types.html', 'd6b'],
  E0277: ['Trait 约束未满足', '核对类型实现的 trait 和调用要求的 bound。', 'ch10-02-traits.html', 'generics'],
  E0599: ['方法或关联项不可用', '当前值可能没有这个方法。先确认变量类型和方法名称，再检查是否需要导入 trait。', 'ch10-02-traits.html', 'generics'],
  E0596: ['当前绑定不能被可变借用', '检查 mut 绑定与函数接收的引用类型。', 'ch04-02-references-and-borrowing.html', 'ownership'],
  E0425: ['名称不存在', 'Rust 找不到这个变量或函数。检查拼写、是否已定义，以及定义是否在当前代码块内。', 'ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html', 'd6b'],
  E0432: ['无法解析导入', '核对 use 中模块路径与公开成员的名称。', 'ch07-04-bringing-paths-into-scope-with-the-use-keyword.html', 'd1'],
  E0433: ['模块或 crate 路径无法解析', '核对模块声明、依赖与路径起点。', 'ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html', 'd1'],
  E0597: ['借用对象存活时间不足', '被引用的值必须在最后一次使用引用时仍然存在。', 'ch10-03-lifetime-syntax.html', 'generics'],
});
const experimentState = { code: null, running: false, token: 0, predictions: Object.create(null) };

function isExperimentRunning() { return experimentState.running; }

function compilerDiagnostics(output) {
  const clean = (output || '').replace(/\u001b\[[0-9;]*m/g, '');
  const codes = [...new Set([...clean.matchAll(/^\s*error\[(E\d{4})\]:/gm)].map(match => match[1]))];
  return codes.filter(code => RUST_ERROR_GUIDES[code]).slice(0, 3).map(code => {
    const [title, explanation, chapter, lesson] = RUST_ERROR_GUIDES[code];
    return { code, title, explanation, url: `https://doc.rust-lang.org/book/${chapter}`, lesson, experiment: code };
  });
}

// Do not infer a passing state or parse arbitrary successful program output.
// Use only a failed, current check supplied by the caller.
function teachingFailure(card, job) {
  if (!card || job?.status !== 'failed') return null;
  if (job.exit_code === null) {
    return { title: '本次操作未能完成', reason: '没有获得正常退出结果，暂时不能判断代码是否满足要求。',
      action: '查看下方真实输出中的启动、环境或超时提示，再重试。' };
  }
  if (job.action !== 'check') {
    return { title: '程序运行未成功', reason: '这次运行返回失败；可能与输入、代码或环境有关。',
      action: '核对本次输入及下方输出，再运行；运行结果不代表课程检查。' };
  }
  const clean = String(job.output || '').replace(/\u001b\[[0-9;]*m/g, '');
  const lines = clean.split(/\r?\n/);
  const index = lines.findIndex(line => /^\s*error(?:\[E\d{4}\])?:\s*\S/.test(line));
  if (index >= 0) {
    const diagnostic = lines[index].match(/^\s*error(?:\[(E\d{4})\])?:\s*(.+)$/);
    const code = diagnostic?.[1];
    const message = diagnostic?.[2] || '';
    const guide = code && RUST_ERROR_GUIDES[code];
    // The checker can compile a temporary copy. Only map an unambiguous
    // compiler location to a source path actually editable in this project.
    let location = '';
    for (const line of lines.slice(index + 1, index + 9)) {
      if (/^\s*error(?:\[E\d{4}\])?:/.test(line)) break;
      const at = line.match(/^\s*-->\s+(.+):(\d+):(\d+)\s*$/);
      if (!at) continue;
      const path = at[1].replace(/\\/g, '/');
      const candidates = (card.project?.files || []).filter(file =>
        path === file || path.endsWith('/' + file));
      if (candidates.length === 1) location = candidates[0] + ' · 第 ' + at[2] + ' 行，第 ' + at[3] + ' 列';
      break;
    }
    if (guide) return { title: '先修复：' + guide[0] + '（' + code + '）', reason: guide[1], location,
      action: '核对第一处错误，按任务卡修改自己的源码，再检查本步。' };
    if (/unclosed delimiter|mismatched closing delimiter|unexpected closing delimiter/.test(message)) {
      return { title: '先修复：括号不匹配', reason: '括号、花括号或方括号没有正确配对，编译器无法读懂这段代码。',
        location, action: '从标出的代码附近检查成对的括号，再检查本步。' };
    }
    return { title: '先查看第一条错误', reason: '输出包含错误诊断，但工作台尚不能可靠解释这一类错误。',
      location, action: '阅读下方原始错误及其上下文；不要仅凭退出码重装 Rust。' };
  }
  if (card.id === 'firstcard' && /^\s*期待：/m.test(clean) && /^\s*实际：/m.test(clean)) {
    return { title: '输出还不符合任务要求', reason: '检查器已运行任务卡，并发现实际输出与期待结果不同。',
      action: '对照下方“期待 / 实际”，检查任务卡中的 println! 是否使用输入变量，再检查本步。' };
  }
  if (/学习反馈：第 \d+ 步已经进入功能检查|^test result: FAILED\b/m.test(clean)) {
    return { title: '代码还未满足检查要求', reason: '本次输出包含功能检查失败的信息，不代表 Rust 安装失败。',
      action: '查看下方失败的检查或断言，对照当前任务卡，修改后再检查。' };
  }
  return { title: '检查未通过', reason: '现有输出不足以可靠判断是代码、输入还是运行环境问题。',
    action: '先阅读下方真实输出中的第一处失败信息，再选择对应的修正方法。' };
}

function renderFailureGuide(card, job) {
  if (!Object.prototype.hasOwnProperty.call(ui, 'lessonFailure')) return;
  const guide = teachingFailure(card, job);
  ui.lessonFailure.hidden = !guide;
  if (!guide) return;
  text(ui.lessonFailureTitle, guide.title);
  text(ui.lessonFailureReason, guide.reason);
  text(ui.lessonFailureAction, guide.action);
  ui.lessonFailureLocation.hidden = !guide.location;
  text(ui.lessonFailureLocation, guide.location || '');
}

function renderCompilerHelp(job) {
  if (!ui.compilerHelp || typeof document.createElement !== 'function') return;
  const guides = job?.status === 'failed' ? compilerDiagnostics(job.output) : [];
  ui.compilerHelp.hidden = guides.length === 0;
  ui.compilerHelpList.replaceChildren();
  for (const guide of guides) {
    const item = document.createElement('li');
    const description = document.createElement('p');
    description.textContent = `${guide.code} · ${guide.title}：${guide.explanation}`;
    const local = button('本地概念解释', 'errorTopic', guide.lesson);
    const experiment = button('运行失败／修复实验', 'errorExperiment', guide.experiment);
    local.className = experiment.className = 'text-button';
    item.append(description, bookLink('官方章节', guide.url), local, experiment);
    ui.compilerHelpList.append(item);
  }
}

function experimentControls() {
  const disabled = experimentState.running || Boolean(state.busy || state.pending) || !experimentState.code;
  ui.experimentFailure.disabled = disabled;
  ui.experimentRepair.disabled = disabled;
  ui.experimentSelect.disabled = experimentState.running;
  ui.experimentPrediction.readOnly = experimentState.running;
}

async function openExperiment(code = 'E0382') {
  if (!RUST_ERROR_GUIDES[code]) return;
  if (ui.bookDialog.open) ui.bookDialog.close();
  if (!ui.experimentDialog.open) ui.experimentDialog.showModal();
  if (experimentState.running) return;
  const token = ++experimentState.token;
  experimentState.code = null;
  ui.experimentSelect.value = code;
  text(ui.experimentStatus, '正在读取对照源码…');
  text(ui.experimentOutput, '');
  text(ui.experimentBad, ''); text(ui.experimentGood, '');
  experimentControls();
  try {
    const material = await api('/api/experiment', { query: { code } });
    if (token !== experimentState.token) return;
    if (material.code !== code || typeof material.failure !== 'string' || typeof material.repair !== 'string') throw new Error('实验内容不完整，请重新打开。');
    experimentState.code = code;
    text(ui.experimentBad, material.failure);
    text(ui.experimentGood, material.repair);
    text(ui.experimentQuestion, material.question);
    ui.experimentPrediction.value = experimentState.predictions[code] || '';
    text(ui.experimentStatus, `${code} · ${material.title}。先预测，再选择一次实际编译。`);
  } catch (error) {
    if (token === experimentState.token) text(ui.experimentStatus, error.message);
  }
  experimentControls();
}

function validateExperimentJob(job, code, variant, id) {
  if (!job || job.code !== code || job.variant !== variant || !Number.isInteger(job.id)
      || (id !== undefined && job.id !== id)
      || !['running', 'completed', 'error'].includes(job.status) || typeof job.output !== 'string'
      || (job.status !== 'running' && typeof job.matched !== 'boolean')) {
    throw new Error('实验返回了不匹配的结果，请重新运行。');
  }
  return job;
}

async function runExperiment(variant) {
  if (experimentState.running || state.busy || state.pending || !experimentState.code) return;
  const code = experimentState.code;
  experimentState.running = true;
  experimentControls(); renderControls();
  const label = variant === 'failure' ? '失败对照' : '修复对照';
  text(ui.experimentStatus, `正在编译${label}…`);
  text(ui.experimentOutput, '');
  try {
    let job = validateExperimentJob(await api('/api/experiment', { form: { code, variant } }), code, variant);
    const id = job.id;
    const deadline = Date.now() + 45000;
    while (job.status === 'running') {
      text(ui.experimentOutput, job.output);
      if (Date.now() > deadline) throw new Error('实验等待超时。结果尚未确认，请查看本地服务窗口后重试。');
      await new Promise(resolve => setTimeout(resolve, 250));
      job = validateExperimentJob(await api('/api/experiment-job', { query: { id } }), code, variant, id);
    }
    text(ui.experimentOutput, job.output);
    const detail = job.matched
      ? variant === 'failure' ? `观察到了预期的 ${code}，编译退出码 ${job.compiler_exit_code}` : `修复示例编译与运行通过，退出码 ${job.program_exit_code}`
      : '结果未符合预期，请查看真实输出';
    text(ui.experimentStatus, detail);
    ui.experimentStatus.dataset.state = job.matched ? 'success' : 'error';
  } catch (error) {
    text(ui.experimentStatus, error.message);
    ui.experimentStatus.dataset.state = 'error';
  } finally {
    experimentState.running = false;
    experimentControls(); renderControls();
  }
}

function initializeExperiments() {
  for (const [name, id] of Object.entries({ experimentDialog: 'experiment-dialog', experimentSelect: 'experiment-select', experimentClose: 'experiment-close', experimentStatus: 'experiment-status', experimentBad: 'experiment-bad', experimentGood: 'experiment-good', experimentFailure: 'experiment-failure', experimentRepair: 'experiment-repair', experimentQuestion: 'experiment-question', experimentPrediction: 'experiment-prediction', experimentOutput: 'experiment-output' })) ui[name] = document.getElementById(id);
  ui.experimentSelect.replaceChildren(...Object.entries(RUST_ERROR_GUIDES).map(([code, guide]) => new Option(`${code} · ${guide[0]}`, code)));
  ui.experimentSelect.addEventListener('change', () => { void openExperiment(ui.experimentSelect.value); });
  ui.experimentClose.addEventListener('click', () => ui.experimentDialog.close());
  ui.experimentFailure.addEventListener('click', () => { void runExperiment('failure'); });
  ui.experimentRepair.addEventListener('click', () => { void runExperiment('repair'); });
  ui.experimentPrediction.addEventListener('input', () => { if (experimentState.code) experimentState.predictions[experimentState.code] = ui.experimentPrediction.value; });
  document.getElementById('book-error-experiments').addEventListener('click', () => { void openExperiment(); });
  ui.compilerHelpList.addEventListener('click', event => {
    const local = event.target.closest('[data-error-topic]');
    const experiment = event.target.closest('[data-error-experiment]');
    if (local) openBook(local.dataset.errorTopic);
    if (experiment) void openExperiment(experiment.dataset.errorExperiment);
  });
}
