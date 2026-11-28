'use strict';

const selectionKey = 'rustapplab.workbench.selection';
const state = {
  catalog: null,
  card: null,
  file: null,
  busy: null,
  pending: null,
  retry: null,
  output: null,
  lastCheck: null,
  historyChanged: false,
  saveIssue: null,
  needsFileReload: false,
  checkingFocus: false,
  connection: 'pending',
};
const ui = {};

function errorWithStatus(message, status) {
  return Object.assign(new Error(message), { status });
}

async function api(route, { query, form } = {}) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 10000);
  const headers = { 'X-RustAppLab': 'workbench' };
  const options = { headers, cache: 'no-store', signal: controller.signal };
  if (form) {
    options.method = 'POST';
    headers['Content-Type'] = 'application/x-www-form-urlencoded;charset=UTF-8';
    options.body = new URLSearchParams(form);
  }
  try {
    const response = await fetch(query ? `${route}?${new URLSearchParams(query)}` : route, options);
    state.connection = 'success';
    const data = await response.json();
    if (!response.ok) {
      throw errorWithStatus(data.error || `本地服务返回 HTTP ${response.status}。`, response.status);
    }
    return data;
  } catch (error) {
    if (error.status) throw error;
    if (error instanceof SyntaxError) {
      throw errorWithStatus('本地服务返回了无法读取的数据。', 502);
    }
    state.connection = 'error';
    throw errorWithStatus('连接本地工作台失败。请确认启动窗口仍在运行，再重试。', 0);
  } finally {
    clearTimeout(timeout);
  }
}

function nextDestination(card, check, changed) {
  if (!card?.project || changed || !check || check.action !== 'check' || check.stale
      || check.status !== 'succeeded' || check.course_id !== card.id || check.step < card.step
      || check.source_digest !== card.project.source_digest) {
    return null;
  }
  // Modern checks select one contract, not a cumulative Studio assessment.
  if (card.id.startsWith('modern-') && check.step !== card.step) return null;
  if (card.step < card.step_count) {
    const suggested = card.project.suggested_step;
    const withinScope = Number.isInteger(suggested) && suggested > card.step
      && suggested <= Math.min(check.step + 1, card.step_count);
    return { step: withinScope ? suggested : card.step + 1 };
  }
  return card.next_course ? { course: card.next_course } : { experiment: true };
}

function text(element, value) {
  if (element.textContent !== value) element.textContent = value;
}

function notice(message, tone = 'neutral') {
  ui.status.dataset.state = tone;
  text(ui.status, message);
}

function dirty() {
  return Boolean(state.file && ui.editor.value !== state.file.content);
}

function sourceChanged() {
  return state.needsFileReload === 'unverified'
    || ((state.needsFileReload || dirty()) && state.file?.path !== 'LEARNING_LOG.md');
}

function allowDiscard(message = '当前文件有未保存的修改。确定放弃这些修改吗？') {
  return !dirty() || window.confirm(message);
}

function readSelection() {
  try {
    return JSON.parse(localStorage.getItem(selectionKey)) || {};
  } catch {
    return {};
  }
}

function rememberSelection() {
  const { card, file } = state;
  if (!card) return;
  try {
    localStorage.setItem(selectionKey, JSON.stringify({
      id: card.id,
      step: card.step,
      file: file?.path,
      projectPath: card.project?.path || ui.projectPath.value,
    }));
  } catch {
    // Local storage is optional in private or restricted browser profiles.
  }
}

function button(label, dataName, value) {
  const element = document.createElement('button');
  element.type = 'button';
  element.textContent = label;
  element.dataset[dataName] = value;
  return element;
}

function courseKind(course) {
  return course.id.startsWith('modern-') ? 'modern' : (course.kind || 'studio');
}

// Use catalog order, not inferred completion. Don't recommend an unrelated
// learning stage just because a group ends.
function courseFocusIds(courses, currentId) {
  if (!courses.length) return [];
  const found = courses.findIndex(course => course.id === currentId);
  const index = found < 0 ? 0 : found;
  const current = courses[index];
  return [current.id, ...courses.slice(index + 1)
    .filter(course => courseKind(course) === courseKind(current))
    .slice(0, 2).map(course => course.id)];
}

let courseNavigation = null;

function buildCourses() {
  const courses = state.catalog.courses;
  const controls = new Map();
  const groups = new Map();
  const labels = { studio: '基础课程', book: 'Rust 专项练习', modern: '进阶课程' };
  for (const [kind, label] of Object.entries(labels)) {
    const details = document.createElement('details');
    details.className = 'course-disclosure';
    details.dataset.courseGroup = kind;
    const summary = document.createElement('summary');
    summary.textContent = label;
    const list = document.createElement('div');
    list.className = 'course-group-list';
    details.append(summary, list);
    groups.set(kind, { details, list });
  }
  for (const course of courses) {
    const [name, ...subtitle] = course.title.split(' · ');
    const control = button('', 'course', course.id);
    control.className = 'course-link';
    control.disabled = Boolean(state.busy);
    const label = document.createElement('strong');
    label.textContent = name;
    control.append(label);
    if (subtitle.length) {
      const detail = document.createElement('small');
      detail.textContent = subtitle.join(' · ');
      control.append(detail);
    }
    controls.set(course.id, control);
  }
  courseNavigation = { controls, groups, current: null };
  ui.courseGroups.replaceChildren(...Array.from(groups.values(), group => group.details));
  ui.courseFocus.hidden = !courses.length;
  ui.courseCatalog.hidden = !courses.length;
  refreshCourseNavigation(state.card?.id || courses[0]?.id);
}

function refreshCourseNavigation(id) {
  if (!courseNavigation || courseNavigation.current === id) return;
  const { controls, groups } = courseNavigation;
  const courses = state.catalog.courses;
  const [current, ...upcoming] = courseFocusIds(courses, id);
  if (!current) return;
  const pinned = new Set([current, ...upcoming]);
  // There is exactly one control per course. Keep IDs, event delegation and
  // the existing dirty-navigation guard; only move controls between sections.
  ui.courseCurrent.replaceChildren(controls.get(current));
  ui.courseUpcoming.replaceChildren(...upcoming.map(id => controls.get(id)));
  ui.courseUpcomingSection.hidden = !upcoming.length;
  for (const [kind, group] of groups) {
    const remaining = courses.filter(course => courseKind(course) === kind && !pinned.has(course.id));
    group.list.replaceChildren(...remaining.map(course => controls.get(course.id)));
    group.details.hidden = !remaining.length;
    group.details.open = false;
  }
  courseNavigation.current = current;
  ui.courseList.scrollTop = 0;
}

// A stored result is usable only for the cumulative classic Studio contract.
// Modern and Book tests have no equivalent persisted per-step history.
function restoredLessonPass(card, check, changed, historyChanged) {
  const history = card?.project?.history;
  return Boolean(card?.kind === 'studio' && !card.id.startsWith('modern-')
    && !check && !changed && !historyChanged && history?.same_source
    && !history.compilation_failed && history.upto_step === history.passed_steps
    && history.passed_steps >= card.step);
}

function renderCard(previous) {
  const { card } = state;
  const course = state.catalog.courses.find(item => item.id === card.id);
  refreshCourseNavigation(card.id);
  text(ui.courseTitle, card.title);
  const path = card.project?.path;
  text(ui.projectState, path ? '练习已打开' : (card.bundle ? '需先准备课程包' : '尚未开始'));
  if (Object.prototype.hasOwnProperty.call(ui, 'activeProjectPath')) {
    ui.activeProjectPath.hidden = !path;
    text(ui.activeProjectPath, path ? '当前项目：' + path : '');
  }
  ui.projectState.dataset.open = String(Boolean(path));
  ui.projectState.title = path || (card.bundle ? '只打开通过验收的离线学员包；完整讲义有准备说明' : '新项目将创建在启动工作台时指定的工作区');
  ui.stepList.hidden = false;
  ui.lessonLoading.hidden = true;
  ui.lessonContent.hidden = false;
  if (previous?.id !== card.id) {
    ui.stepList.replaceChildren(...course.steps.map((title, index) => {
      const control = button(title, 'step', index + 1);
      control.disabled = Boolean(state.busy);
      const ordinal = document.createElement('span');
      ordinal.className = 'step-number';
      ordinal.textContent = index + 1;
      ordinal.setAttribute('aria-hidden', 'true');
      control.prepend(ordinal);
      return control;
    }));
  }
  if (previous?.id !== card.id || previous.step !== card.step) {
    text(ui.stepTitle, card.step_title);
    text(ui.goal, card.goal);
    text(ui.target, card.function);
    text(ui.concept, card.concept);
    renderReading(card.reading);
    text(ui.prediction, card.prediction);
    text(ui.experiment, card.experiment);
    text(ui.reflection, card.reflection);
    ui.hints.replaceChildren(...card.hints.map((hint, index) => {
      const details = document.createElement('details');
      const summary = document.createElement('summary');
      summary.dataset.hint = index + 1;
      summary.textContent = `提示 ${index + 1}`;
      const description = document.createElement('p');
      description.textContent = hint;
      details.append(summary, description);
      return details;
    }));
    if (Object.prototype.hasOwnProperty.call(ui, 'lessonPractice')) {
      ui.lessonHints.open = false;
      ui.lessonPractice.open = false;
      ui.lessonProgress.dataset.phase = '';
    }
    ui.lessonContent.scrollTop = 0;
    ui.runArgs.value = card.run_args.join('\n');
    ui.runSettings.open = false;
  }
  document.title = `${card.title} · RustAppLab`;
}

function renderFiles(requestedPath) {
  const files = state.card.project?.files || [];
  ui.fileSelect.replaceChildren();
  if (files.length === 0) {
    ui.fileSelect.add(new Option('打开项目后选择文件', ''));
  } else {
    files.forEach(path => ui.fileSelect.add(new Option(path, path)));
    ui.fileSelect.value = requestedPath || files[0];
  }
}

function installFile(file, previousDigest = state.card.project.source_digest) {
  if (file.source_digest) {
    if (file.source_digest !== previousDigest) state.historyChanged = true;
    state.card.project.source_digest = file.source_digest;
  }
  ui.editor.value = file.content;
  state.file = { ...file, content: ui.editor.value };
  state.saveIssue = null;
  state.needsFileReload = false;
  ui.fileSelect.value = file.path;
  ui.editor.scrollTop = 0;
  ui.editor.scrollLeft = 0;
  rememberSelection();
}

function getCourse(id, step) {
  return api('/api/course', { query: { id, step } });
}

function getFile(card, path) {
  return api('/api/file', { query: { id: card.id, path, project_path: card.project.path } });
}

async function inspectFileRevision(card, file) {
  try {
    const disk = await getFile(card, file.path);
    return { changed: disk.revision !== file.revision, digest: disk.source_digest };
  } catch (error) {
    return { error };
  }
}

function reconcileFileRevision(card, inspection) {
  const digestChanged = inspection.digest && inspection.digest !== card.project.source_digest;
  if (digestChanged) {
    card.project.source_digest = inspection.digest;
    state.historyChanged = true;
  }
  state.needsFileReload = inspection.error ? 'unverified' : inspection.changed;
  return digestChanged;
}

async function recheckOnFocus() {
  const { card, file } = state;
  if (state.checkingFocus || state.busy || state.pending || !card?.project || !file) return;
  state.checkingFocus = true;
  const inspection = await inspectFileRevision(card, file);
  state.checkingFocus = false;
  if (state.busy || state.pending || state.card !== card || state.file !== file) return;

  const digestChanged = reconcileFileRevision(card, inspection);
  if (inspection.error) {
    notice(`暂时无法核对当前文件。 ${inspection.error.message}`, 'warning');
  } else if (state.needsFileReload) {
    notice('磁盘上的文件已变化，编辑器内容已保留。可重读磁盘版本。', 'warning');
  } else if (digestChanged) {
    notice('工程源码或运行输入已变化，请重新检查。', 'warning');
  }
  renderControls();
}

function renderHistory() {
  const { card } = state;
  const history = card?.project?.history;
  ui.history.hidden = !history;
  if (!history) return;
  const record = history.compilation_failed
    ? `上次检查在第 ${history.upto_step} 步的检查范围内编译失败`
    : `历史检查 · ${history.passed_steps} / ${history.upto_step} 步通过`;
  const freshness = history.same_source && !state.historyChanged && !sourceChanged()
    ? '记录中的 Rust 源码一致'
    : '源码或输入已有变动，请重新检查';
  text(ui.history, `${record} · ${freshness}`);
}

function renderNext() {
  const { card, lastCheck } = state;
  const changed = sourceChanged();
  let destination = nextDestination(card, lastCheck, changed);
  const restored = Boolean(state.file) && restoredLessonPass(card, lastCheck, changed, state.historyChanged);
  if (!destination && restored) {
    destination = card.step < card.step_count ? { step: card.step + 1 }
      : card.next_course ? { course: card.next_course } : { experiment: true };
  }
  const history = card?.project?.history;
  const suggestedStep = card?.project?.suggested_step;
  const historySuggestion = !destination && history?.same_source && !state.historyChanged
    && !sourceChanged() && suggestedStep >= 1 && suggestedStep <= card.step_count
    && suggestedStep !== card.step;
  if (historySuggestion) destination = { step: suggestedStep };
  ui.next.hidden = !destination;
  if (!destination) return;
  if (destination.step) {
    const verb = historySuggestion ? '查看建议' : '进入';
    text(ui.next, `${verb}第 ${destination.step} 步 →`);
  } else if (destination.course) {
    const title = state.catalog.courses.find(item => item.id === destination.course)?.title;
    text(ui.next, `下一课 · ${title?.split(' · ')[0] || destination.course} →`);
  } else {
    text(ui.next, '做个人实验 →');
  }
  ui.next.onclick = async () => {
    if (state.busy || state.pending) return;
    if (destination.experiment) {
      if (Object.prototype.hasOwnProperty.call(ui, 'lessonPractice')) ui.lessonPractice.open = true;
      ui.experimentTitle.focus();
      return;
    }
    const id = destination.course || card.id;
    const step = destination.step || 1;
    await selectCourse(id, step);
    if (state.card.id === id && state.card.step === step) ui.stepTitle.focus();
  };
}

// Use one real form for both opening and switching a project. The empty editor
// is a lesson entry, not a disabled development environment.
function renderLessonEntry(card, project) {
  if (!Object.prototype.hasOwnProperty.call(ui, 'startPanel')) return;
  ui.editorPanel.dataset.ready = String(project);
  ui.startPanel.hidden = project;
  ui.projectSwitchSlot.hidden = !project;
  text(ui.editorTitle, project ? '代码编辑器' : '开始学习');
  text(ui.projectSwitchSummary, project ? '切换已有项目' : '打开已有项目');
  const target = project ? ui.projectSwitchSlot : ui.startPanel.querySelector('.start-card');
  if (ui.projectSwitch.parentElement !== target) {
    target.append(ui.projectSwitch);
    ui.projectSwitch.open = !project && Boolean(card?.bundle);
  }
  const course = card?.id || '';
  if (ui.projectSwitch.dataset.course !== course) {
    ui.projectSwitch.dataset.course = course;
    ui.projectSwitch.open = !project && Boolean(card?.bundle);
  }
  text(ui.startTitle, card?.bundle ? '打开本课的练习' : '准备开始本课');
  text(ui.startDescription, card?.bundle
    ? '请先解压 ' + card.bundle + '，再打开解压后的个人练习文件夹。准备方法见完整讲义。'
    : card ? '点击开始，工作台会自动准备练习代码。' : '正在读取课程…');
  ui.console.hidden = !project && !state.output;
  if (Object.prototype.hasOwnProperty.call(ui, 'coursePreview')) {
    const preview = foundationPreview(card?.id);
    ui.coursePreview.hidden = project || !preview;
    if (preview) {
      text(ui.coursePreviewDescription, preview.description);
      text(ui.coursePreviewOutput, preview.output);
      text(ui.coursePreviewNote, preview.note);
    }
  }
}

// These three previews are taken from the existing learner contracts and
// reference CLI demonstrations, not from the learner's own execution.
// UnitLab and QuizDeck need all three steps before their full demo works.
const foundationPreviews = Object.freeze({
  firstcard: {
    description: '输入姓名和任务，生成自己的任务卡。',
    output: '今日任务卡 | 小林 | 写 Rust',
    note: '修改一行代码即可完成。',
  },
  unitlab: {
    description: '输入数字与单位，得到换算结果。',
    output: '25.00 c = 77.00 f',
    note: '完成全部 3 步后，才能使用完整换算器。',
  },
  quizdeck: {
    description: '批改自己的题库，显示得分与错题。',
    output: 'score=2/3\n复习：打印宏',
    note: '完成全部 3 步后，才能使用完整题库工具。',
  },
});

function foundationPreview(id) {
  return Object.prototype.hasOwnProperty.call(foundationPreviews, id) ? foundationPreviews[id] : null;
}

function lessonRecap(card) {
  if (!card) return '';
  const explanations = {
    firstcard: ['用变量参与格式化输出，让姓名和任务随输入变化。参数读取代码已经提供，本步没有独立实现参数解析。'],
    unitlab: [
      '用函数参数参与计算，返回浮点结果而不是打印固定数字。此时完整换算器还未完成。',
      '用枚举与 match 区分单位组合，用 Result 表达成功或失败。',
      '解析用户文本、传播错误，再格式化计算结果，把前面的函数连接成软件。',
    ],
    quizdeck: [
      '借用字符串，去掉首尾空白，再比较完整答案；相等不是包含。',
      '把题库文本解析为拥有自己的 String 的 Card，再保存到 Vec。',
      '先检查答案数量，再逐题比较，统计得分与错题。',
    ],
  };
  const explanation = Object.prototype.hasOwnProperty.call(explanations, card.id)
    ? explanations[card.id][card.step - 1] : '';
  return explanation || card.concept || '';
}

// The task card reveals guidance from actual project and check state.
function renderLearningProgress(card, locked) {
  if (!Object.prototype.hasOwnProperty.call(ui, 'lessonProgress') || !card) return;
  const changed = sourceChanged();
  const check = state.lastCheck;
  const passed = Boolean(nextDestination(card, check, changed))
    || (Boolean(state.file) && restoredLessonPass(card, check, changed, state.historyChanged));
  const failed = check?.action === 'check' && check.status === 'failed'
    && check.course_id === card.id && check.step === card.step && !check.stale && !changed
    && check.source_digest === card.project?.source_digest;
  const phase = !card.project ? 'start' : locked ? 'busy' : !state.file ? 'file' : passed ? 'passed' : failed ? 'retry' : 'edit';
  if (Object.prototype.hasOwnProperty.call(ui, 'lessonRecap')) {
    // A recap is conditional on a verified current code result. Never turn
    // a run, dirty source or older output into a learning achievement.
    const showRecap = phase === 'passed';
    ui.lessonRecap.hidden = !showRecap;
    ui.lessonConceptRow.hidden = showRecap;
    if (showRecap) text(ui.lessonRecapText, lessonRecap(card));
  }
  const guidance = {
    start: card.bundle ? '先准备并打开本课练习' : '先开始本课',
    busy: '正在处理，请查看操作状态',
    file: '打开练习文件后继续',
    passed: card.can_run === false ? '代码检查通过 · 完成练习与复盘' : '代码检查通过 · 试一组自己的输入',
    retry: '检查未通过 · 查看输出与提示',
    edit: '修改代码，再检查本步',
  };
  text(ui.lessonProgress, '第 ' + card.step + ' / ' + card.step_count + ' 步 · ' + guidance[phase]);
  if (ui.lessonProgress.dataset.phase !== phase) {
    if (phase === 'passed') ui.lessonPractice.open = true;
    if (phase === 'retry') ui.lessonHints.open = true;
    ui.lessonProgress.dataset.phase = phase;
  }
}

function renderControls() {
  const { busy, pending, card, file, retry, saveIssue } = state;
  const locked = Boolean(busy || pending || isExperimentRunning());
  const project = Boolean(card?.project);
  const changed = dirty();
  const saving = busy === 'save' || (busy && ui.save.dataset.saving === 'true');
  const disconnected = state.connection === 'error';
  ui.engine.dataset.state = state.connection;
  text(ui.engine, disconnected ? '连接中断' : state.connection === 'success' ? '本地工作台已连接' : '正在连接');

  for (const control of ui.courseList.querySelectorAll('button')) {
    control.disabled = locked;
    control.setAttribute('aria-current', control.dataset.course === card?.id ? 'true' : 'false');
  }
  for (const control of ui.stepList.querySelectorAll('button')) {
    control.disabled = locked;
    control.setAttribute('aria-current', Number(control.dataset.step) === card?.step ? 'step' : 'false');
  }

  renderLessonEntry(card, project);
  renderLearningProgress(card, locked);
  ui.projectPath.disabled = locked || !card;
  ui.openProject.disabled = locked || !card;
  ui.create.hidden = Boolean(card?.bundle);
  ui.create.disabled = locked || !card || project || Boolean(card?.bundle);
  ui.lessonOpen.disabled = !card;
  ui.fileSelect.disabled = locked || !project;
  ui.editor.readOnly = locked || !file;
  ui.runArgs.disabled = locked || !project || !file;
  ui.save.disabled = locked || !file || !changed || saveIssue?.status === 409;
  ui.check.disabled = locked || !file || saveIssue?.status === 409;
  ui.run.disabled = locked || !file || card?.can_run === false || saveIssue?.status === 409;
  ui.run.hidden = card?.can_run === false;
  ui.runSettings.hidden = card?.can_run === false;
  if (ui.learningLog) ui.learningLog.disabled = locked;
  if (ui.bookPractice) ui.bookPractice.disabled = locked;
  if (ui.experimentFailure) experimentControls();
  ui.check.title = card?.id?.startsWith('modern-') ? 'Rust 1.100.0：本步固定合同，最后一步全部测试；首次需自行安装工具链' : card?.kind === 'book' ? '执行当前专题的真实 Cargo 测试和文档测试；不等于理解评分' : '检查当前步骤的实际行为';
  ui.next.disabled = locked;
  ui.retry.hidden = !retry;
  ui.retry.disabled = Boolean(busy);
  ui.stopWaiting.hidden = !pending || Boolean(busy);
  ui.reloadFile.hidden = saveIssue?.status !== 409 && !state.needsFileReload;
  ui.reloadFile.disabled = locked;
  if (retry) text(ui.retry, retry.label);

  const checking = busy === 'check' || pending?.action === 'check';
  const running = busy === 'run' || pending?.action === 'run';
  ui.check.setAttribute('aria-busy', String(checking));
  ui.run.setAttribute('aria-busy', String(running));
  text(ui.create, busy === 'create' ? '正在准备…' : '开始本课');
  text(ui.openProject, busy === 'open' ? '正在打开…' : '打开项目');
  text(ui.save, saving ? '正在保存…' : '保存');
  text(ui.check, checking ? '正在检查…' : card?.id?.startsWith('modern-') ? '检查本步' : card?.kind === 'book' ? '验证专题' : '检查本步');
  text(ui.run, running ? '正在运行…' : '运行程序');
  if (saving) {
    ui.saveStatus.dataset.state = 'pending';
    text(ui.saveStatus, '正在保存');
  } else if (saveIssue) {
    ui.saveStatus.dataset.state = saveIssue.status === 409 ? 'warning' : 'error';
    text(ui.saveStatus, saveIssue.status === 409 ? '文件有冲突，草稿已保留' : '保存失败');
  } else if (state.needsFileReload) {
    ui.saveStatus.dataset.state = 'warning';
    text(ui.saveStatus, state.needsFileReload === 'unverified' ? '暂时无法核对文件版本' : '文件已在外部修改');
  } else if (file) {
    ui.saveStatus.dataset.state = changed ? 'dirty' : 'saved';
    text(ui.saveStatus, changed ? '有未保存的修改' : '已保存');
  } else {
    ui.saveStatus.dataset.state = 'idle';
    text(ui.saveStatus, '尚未打开文件');
  }
  renderNext();
  renderHistory();
  renderOutput();
}

function setConsoleMode(mode) {
  ui.console.dataset.mode = mode;
  ui.consoleOutput.hidden = mode === 'collapsed';
  ui.consoleToggle.setAttribute('aria-expanded', String(mode !== 'collapsed'));
  ui.consoleSize.setAttribute('aria-pressed', String(mode === 'large'));
  text(ui.consoleToggle, mode === 'collapsed' ? '展开' : '折叠');
  text(ui.consoleSize, mode === 'large' ? '还原' : '放大');
}

function renderOutput() {
  const { output } = state;
  let summary = output?.summary || '运行与检查的输出显示在这里';
  let tone = output?.tone || 'neutral';
  if (output?.job) {
    const job = output.job;
    const label = job.action === 'check' ? (state.card.kind === 'book' ? '专题代码测试' : `第 ${job.step} 步检查`) : '程序运行';
    const old = job.stale || job.source_digest !== state.card.project.source_digest || sourceChanged();
    if (output.paused) {
      summary = `${label} · 连接中断，等待继续查询`;
      tone = 'error';
    } else if (job.status === 'running') {
      summary = `${label}进行中…`;
      tone = 'pending';
    } else if (job.status === 'failed') {
      summary = `${label}失败 · ${job.exit_code === null ? '无法执行，请查看输出' : `退出码 ${job.exit_code}`}`;
      if (old) summary += ' · 当时的文件版本';
      tone = 'error';
    } else if (old) {
      summary = `${label}已完成 · 当时的文件版本，请重新${job.action === 'check' ? '检查' : '运行'}`;
      tone = 'warning';
    } else {
      summary = job.action === 'check' ? `${label}通过` : `${label}结束 · 退出码 ${job.exit_code}`;
      tone = 'success';
    }
  }
  ui.console.dataset.phase = output ? 'active' : 'idle';
  ui.consoleSummary.dataset.state = tone;
  text(ui.consoleSummary, summary);
  text(ui.consoleOutput, output?.job?.output ?? output?.details ?? '');
  renderCompilerHelp(output?.job);
}

// Navigation, saving and polling all use the same operation lock. A failed
// poll keeps its job ID until the learner retries or explicitly stops waiting.
async function perform(key, message, task, retry = null) {
  if (state.busy || (state.pending && state.pending.action !== key)) return;
  state.busy = key;
  state.retry = null;
  notice(message, 'pending');
  renderControls();
  try {
    await task();
  } catch (error) {
    if (!state.retry) state.retry = retry;
    notice(error.message, 'error');
  } finally {
    state.busy = null;
    renderControls();
  }
}

function repeat(label, invoke) {
  return { label, invoke };
}

async function applyCard(card, { file, projectPath, forceFile = false } = {}) {
  const previous = state.card;
  const changedProject = previous?.id !== card.id || previous?.project?.path !== card.project?.path;
  state.card = card;
  if (changedProject || forceFile) {
    state.file = null;
    state.lastCheck = null;
    state.output = null;
    state.saveIssue = null;
    state.historyChanged = false;
    state.needsFileReload = false;
    ui.editor.value = '';
    ui.projectPath.value = card.project?.path || projectPath || '';
  } else if (card.project?.source_digest !== previous.project?.source_digest) {
    state.historyChanged = true;
  }
  renderCard(previous);
  if (!changedProject && !forceFile) {
    rememberSelection();
    return;
  }

  const course = state.catalog.courses.find(item => item.id === card.id);
  const files = card.project?.files || [];
  const requested = [file, course.default_file].find(path => files.includes(path)) || files[0];
  renderFiles(requested);
  rememberSelection();
  if (!requested) return;
  try {
    installFile(await getFile(card, requested));
  } catch (error) {
    state.retry = repeat('重读文件', () => selectFile(requested));
    throw error;
  }
}

function readyNotice() {
  if (state.card.project) {
    notice(state.file ? '' : '请选择一个文件开始编辑。');
  } else {
    notice('');
  }
}

async function start() {
  await perform('boot', '正在读取本地课程…', async () => {
    state.catalog = await api('/api/catalog');
    buildCourses();
    const saved = readSelection();
    const course = state.catalog.courses.find(item => item.id === saved.id) || state.catalog.courses[0];
    const step = Number.isInteger(saved.step) && saved.step >= 1 && saved.step <= course.steps.length
      && saved.id === course.id ? saved.step : 1;
    const card = await getCourse(course.id, step);
    await applyCard(card, saved.id === course.id ? saved : {});
    readyNotice();
  }, repeat('重连', start));
}

async function selectCourse(id, step) {
  if (state.busy || state.pending) return;
  if (id === state.card?.id && step === state.card.step) return;
  const previous = state.card;
  const sameCourse = id === previous?.id;
  if (!sameCourse && !allowDiscard()) return;
  await perform('course', '正在读取任务卡…', async () => {
    const card = await getCourse(id, step);
    if (sameCourse && card.project?.path !== previous.project?.path
        && !allowDiscard('当前课程的项目已在另一标签页切换。确定放弃当前文件的草稿并切换项目吗？')) {
      notice('仍保留当前草稿。请核对项目路径，切换工程前可先复制草稿。', 'warning');
      return;
    }
    await applyCard(card);
    readyNotice();
  }, repeat('重读任务卡', () => selectCourse(id, step)));
}

async function changeProject(action) {
  if (state.busy || state.pending || !state.card || !allowDiscard()) return;
  const path = ui.projectPath.value;
  if (action === 'open' && !path.trim()) {
    notice('请输入已有学习项目的目录。', 'warning');
    ui.projectPath.focus();
    return;
  }
  let opened = false;
  await perform(action, action === 'create' ? '正在准备练习…' : '正在打开已有项目…', async () => {
    const form = { id: state.card.id };
    if (action === 'open') form.path = path;
    const card = await api(`/api/${action}`, { form });
    await applyCard(card, { forceFile: true });
    opened = true;
    notice('练习已准备好，按任务卡修改代码。', 'success');
  });
  if (opened && state.file) ui.editor.focus();
}

async function selectFile(path, reload = false) {
  if (state.busy || state.pending || !state.card?.project) return;
  if (state.file) ui.fileSelect.value = state.file.path;
  if (!path || (!reload && path === state.file?.path) || !allowDiscard()) return;
  await perform(reload ? 'reload' : 'file', '正在读取文件…', async () => {
    const { id, step, project } = state.card;
    if (reload) {
      const card = await getCourse(id, step);
      if (card.project?.path !== project.path) {
        throw errorWithStatus('当前课程的项目已切换。请用「打开项目」重新打开原目录后重读。', 409);
      }
      const file = await getFile(card, path);
      state.card = card;
      installFile(file, project.source_digest);
    } else {
      installFile(await getFile(state.card, path));
    }
    notice(reload ? '已重读磁盘版本。' : '文件已打开。', 'success');
  }, repeat('重读文件', () => selectFile(path, reload)));
}

async function saveFile() {
  if (!dirty()) return;
  const { file, card } = state;
  const content = ui.editor.value;
  state.saveIssue = null;
  ui.save.dataset.saving = 'true';
  renderControls();
  try {
    const saved = await api('/api/save', { form: {
      id: card.id, path: file.path, content, revision: file.revision, project_path: card.project.path,
    } });
    state.file = { ...file, content, revision: saved.revision };
    if (card.project.source_digest !== saved.source_digest) state.historyChanged = true;
    card.project.source_digest = saved.source_digest;
    state.needsFileReload = false;
  } catch (error) {
    state.saveIssue = error;
    if (error.status === 409) {
      throw errorWithStatus(`${error.message} 草稿仍在编辑器中，可以先复制，再选择重读磁盘版本。`, 409);
    }
    throw error;
  } finally {
    ui.save.dataset.saving = 'false';
  }
}

async function saveCurrent() {
  if (ui.save.disabled) return;
  await perform('save', '正在保存文件…', async () => {
    await saveFile();
    notice('文件已保存。', 'success');
  });
}

function readJob(job, expected) {
  if (job.course_id !== expected.id || job.action !== expected.action || job.step !== expected.step
      || (expected.jobId !== undefined && job.id !== expected.jobId)
      || !['running', 'succeeded', 'failed'].includes(job.status)) {
    throw errorWithStatus('本地工作台返回了不同任务的结果，请重新检查或运行。', 502);
  }
  return job;
}

async function finishJob(job) {
  const { id, step, project } = state.card;
  state.pending = null;
  state.output = { job };
  if (job.action === 'check') state.lastCheck = job;
  let refreshIssue;
  try {
    const latest = await getCourse(id, step);
    if (latest.project?.path === project.path) {
      // A previous card refresh may know a newer digest than the open file.
      // Always verify the editor's revision while retaining its contents.
      const inspection = await inspectFileRevision(latest, state.file);
      reconcileFileRevision(latest, inspection);
      refreshIssue = inspection.error;
      if (latest.project.source_digest !== project.source_digest) state.historyChanged = true;
      state.card = latest;
      if (job.action === 'check' && !job.stale && job.exit_code !== null
          && job.source_digest === latest.project.source_digest) {
        state.historyChanged = false;
      }
    } else {
      state.needsFileReload = 'unverified';
    }
  } catch (error) {
    refreshIssue = error;
    state.needsFileReload = 'unverified';
  }
  if (refreshIssue) {
    notice(`已收到本次输出。项目状态暂时无法刷新。 ${refreshIssue.message}`, 'warning');
  } else if (job.status === 'failed') {
    notice(job.action === 'check' ? '先看输出中的错误或未满足的要求，修改后再检查。' : '运行失败，请查看输出并修正。', 'error');
  } else if (job.stale || sourceChanged() || job.source_digest !== state.card.project.source_digest) {
    notice('文件版本有变化，查看输出后可重新检查或运行。', 'warning');
  } else {
    notice(job.action === 'check' ? '本步代码检查通过。试一组自己的输入，再继续学习。' : '运行结束。可更换输入，再观察输出变化。', 'success');
  }
}

async function pollJob(job) {
  const expected = { id: job.course_id, action: job.action, step: job.step, jobId: job.id };
  while (job.status === 'running') {
    state.pending = job;
    state.output = { job };
    renderControls();
    await new Promise(resolve => setTimeout(resolve, 450));
    try {
      job = readJob(await api('/api/job', { query: { id: expected.jobId } }), expected);
    } catch (error) {
      if (error.status === 0 || error.status >= 500) {
        state.output = { job, paused: true };
        state.retry = repeat('继续查询结果', () => resumeJob(job));
      } else {
        state.pending = null;
        state.output = { summary: '无法查询本次结果', tone: 'error', details: error.message };
      }
      throw error;
    }
  }
  await finishJob(job);
}

async function resumeJob(job) {
  await perform(job.action, '正在重新查询本次结果…', async () => {
    if (ui.console.dataset.mode === 'collapsed') setConsoleMode('normal');
    await pollJob(job);
  });
}

async function startJob(action) {
  const control = action === 'check' ? ui.check : ui.run;
  if (control.disabled) return;
  const { id, step, project } = state.card;
  const isCheck = action === 'check';
  const label = isCheck ? (state.card.kind === 'book' ? '专题测试' : `第 ${step} 步检查`) : '程序运行';
  const args = isCheck ? '' : ui.runArgs.value;
  state.output = { summary: `${label}准备中…`, tone: 'pending' };
  if (ui.console.dataset.mode === 'collapsed') setConsoleMode('normal');
  if (window.matchMedia('(max-width: 600px)').matches) {
    window.requestAnimationFrame(() => {
      if (state.busy === action && control.getBoundingClientRect().bottom > ui.console.getBoundingClientRect().top) {
        control.scrollIntoView({ block: 'center' });
      }
    });
  }
  await perform(action, `正在准备${label}…`, async () => {
    let job;
    try {
      await saveFile();
      const form = { id, action, step, args, project_path: project.path };
      job = readJob(await api('/api/action', { form }), { id, action, step });
    } catch (error) {
      state.output = { summary: `未开始${label}`, tone: 'error', details: error.message };
      throw error;
    }
    notice(`${label}使用当前磁盘中的文件。`, 'pending');
    await pollJob(job);
  });
}

function initialize() {
  const ids = {
    engine: 'engine-status', courseList: 'course-list',
    courseFocus: 'course-focus', courseCurrent: 'course-current', courseUpcoming: 'course-upcoming',
    courseUpcomingSection: 'course-upcoming-section', courseCatalog: 'course-catalog', courseGroups: 'course-groups',
    courseTitle: 'course-title', projectState: 'project-state', projectPath: 'project-path',
    projectForm: 'project-form', create: 'create-button', openProject: 'open-project-button',
    startPanel: 'start-panel', startTitle: 'start-title', startDescription: 'start-description',
    editorPanel: 'editor-panel', editorTitle: 'editor-title', projectSwitch: 'project-switch',
    projectSwitchSlot: 'project-switch-slot', projectSwitchSummary: 'project-switch-summary',
    stepList: 'step-list', lessonOpen: 'lesson-open', lessonClose: 'lesson-close', lessonDialog: 'lesson-dialog',
    lessonFrame: 'lesson-frame', lessonContent: 'lesson-content', lessonLoading: 'lesson-loading',
    stepTitle: 'step-title', goal: 'lesson-goal', target: 'lesson-function', concept: 'lesson-concept',
    lessonProgress: 'lesson-progress', lessonHints: 'lesson-hints', lessonPractice: 'lesson-practice',
    lessonRecap: 'lesson-recap', lessonRecapText: 'lesson-recap-text', lessonConceptRow: 'lesson-concept-row',
    coursePreview: 'course-preview', coursePreviewDescription: 'course-preview-description',
    coursePreviewOutput: 'course-preview-output', coursePreviewNote: 'course-preview-note',
    activeProjectPath: 'active-project-path',
    prediction: 'lesson-prediction', hints: 'hint-list', experiment: 'lesson-experiment',
    reading: 'book-reading', readingTopics: 'book-reading-topics',
    readingQuestion: 'book-reading-question', readingVersion: 'book-reading-version',
    experimentTitle: 'experiment-title', reflection: 'lesson-reflection', fileSelect: 'file-select',
    saveStatus: 'save-status', reloadFile: 'reload-file-button', editor: 'source-editor',
    save: 'save-button', run: 'run-button', check: 'check-button', runSettings: 'run-settings',
    runArgs: 'run-args', status: 'status-message', history: 'history-summary',
    retry: 'retry-button', stopWaiting: 'stop-waiting-button', next: 'next-button',
    console: 'console-panel', consoleOutput: 'console-output', consoleSummary: 'console-summary',
    consoleSize: 'console-size', consoleToggle: 'console-toggle',
    compilerHelp: 'compiler-help', compilerHelpList: 'compiler-help-list',
    bookOpen: 'book-open', bookClose: 'book-close', bookDialog: 'book-dialog',
    bookSelect: 'book-select', bookStatus: 'book-status', bookText: 'book-text',
  };
  for (const [name, id] of Object.entries(ids)) ui[name] = document.getElementById(id);

  ui.courseList.addEventListener('click', async event => {
    const control = event.target.closest('[data-course]');
    if (!control) return;
    const id = control.dataset.course;
    const changed = id !== state.card?.id;
    await selectCourse(id, 1);
    if (changed && state.card?.id === id) control.focus({ preventScroll: true });
  });
  ui.stepList.addEventListener('click', event => {
    const control = event.target.closest('[data-step]');
    if (control) void selectCourse(state.card.id, Number(control.dataset.step));
  });
  ui.create.addEventListener('click', () => { void changeProject('create'); });
  initializeKnowledge();
  initializeExperiments();
  ui.projectForm.addEventListener('submit', event => {
    event.preventDefault();
    void changeProject('open');
  });
  ui.lessonOpen.addEventListener('click', () => {
    if (state.card.lesson_topic) { openBook(state.card.lesson_topic); return; }
    ui.lessonFrame.src = `/api/lesson?${new URLSearchParams({ id: state.card.id })}`;
    ui.lessonDialog.showModal();
  });
  ui.lessonClose.addEventListener('click', () => ui.lessonDialog.close());
  ui.save.addEventListener('click', () => { void saveCurrent(); });
  ui.run.addEventListener('click', () => { void startJob('run'); });
  ui.check.addEventListener('click', () => { void startJob('check'); });
  ui.fileSelect.addEventListener('change', () => { void selectFile(ui.fileSelect.value); });
  ui.reloadFile.addEventListener('click', () => { void selectFile(state.file.path, true); });
  ui.retry.addEventListener('click', () => { void state.retry?.invoke(); });
  ui.stopWaiting.addEventListener('click', () => {
    state.pending = null;
    state.retry = null;
    state.output = {
      summary: '已停止等待本次输出', tone: 'warning', details: state.output.job.output,
    };
    notice('可以继续编辑。若本地任务仍在执行，等它结束后再启动新任务。', 'warning');
    renderControls();
  });
  ui.consoleToggle.addEventListener('click', () => {
    setConsoleMode(ui.console.dataset.mode === 'collapsed' ? 'normal' : 'collapsed');
  });
  ui.consoleSize.addEventListener('click', () => {
    setConsoleMode(ui.console.dataset.mode === 'large' ? 'normal' : 'large');
  });

  ui.editor.addEventListener('input', renderControls);
  let allowTab = false;
  ui.editor.addEventListener('blur', () => { allowTab = false; });
  ui.editor.addEventListener('keydown', event => {
    if (event.key === 'Escape') {
      allowTab = true;
      return;
    }
    if (event.key === 'Tab' && !event.shiftKey && !allowTab && !ui.editor.readOnly
        && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      ui.editor.setRangeText('    ', ui.editor.selectionStart, ui.editor.selectionEnd, 'end');
      renderControls();
    }
    allowTab = false;
  });
  document.addEventListener('keydown', event => {
    if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === 's') {
      event.preventDefault();
      void saveCurrent();
    }
  });
  window.addEventListener('beforeunload', event => {
    if (dirty()) {
      event.preventDefault();
      event.returnValue = '';
    }
  });
  window.addEventListener('focus', () => { void recheckOnFocus(); });
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') void recheckOnFocus();
  });
  setConsoleMode('normal');
  void start();
}

document.addEventListener('DOMContentLoaded', initialize);
