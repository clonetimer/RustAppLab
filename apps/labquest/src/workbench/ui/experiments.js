'use strict';

const RUST_ERROR_GUIDES = Object.freeze({
  E0382: ['值被移动后再次使用', '检查所有权移交的位置，判断只读借用或调整使用顺序是否满足需求。', 'ch04-01-what-is-ownership.html', 'ownership'],
  E0502: ['可变借用与共享借用冲突', '定位两个引用实际使用的区间，检查修改是否发生在最后一次读取之前。', 'ch04-02-references-and-borrowing.html', 'ownership'],
  E0499: ['存在重叠的可变借用', '同一数据不能同时由两个活跃的可变引用修改。', 'ch04-02-references-and-borrowing.html', 'ownership'],
  E0308: ['类型不匹配', '对照期望和实际类型，检查参数、分支与返回表达式。', 'ch03-02-data-types.html', 'd6b'],
  E0277: ['Trait 约束未满足', '核对类型实现的 trait 和调用要求的 bound。', 'ch10-02-traits.html', 'generics'],
  E0599: ['方法或关联项不可用', '核对接收者类型、方法名称和 trait 是否已导入。', 'ch10-02-traits.html', 'generics'],
  E0596: ['当前绑定不能被可变借用', '检查 mut 绑定与函数接收的引用类型。', 'ch04-02-references-and-borrowing.html', 'ownership'],
  E0425: ['名称不存在', '核对拼写和该绑定所在的作用域。', 'ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html', 'd6b'],
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
