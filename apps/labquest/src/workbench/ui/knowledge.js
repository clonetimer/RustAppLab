'use strict';

// Navigation metadata only. Canonical lessons and project sources come from the binary.
const KNOWLEDGE_TOPICS = Object.freeze([
  { id: 'ownership', title: '所有权、借用与结构体', terms: 'String str Copy Clone 移动 mut 字段 初始化 元组 类单元 多共享引用 切片 E0382 E0502 E0499 E0596', course: 'book-foundations', chapter: 'ch04-00-understanding-ownership.html' },
  { id: 'collections', title: '集合、字符串与错误', terms: 'Vec Drop HashMap zip collect BuildHasher RandomState UTF-8 panic unwind abort backtrace unwrap expect Result ErrorKind', course: 'book-foundations', chapter: 'ch08-00-common-collections.html' },
  { id: 'generics', title: '泛型、Trait 与生命周期', terms: '单态化 孤儿规则 newtype Ord PartialOrd NaN blanket impl lifetime E0277 E0599 E0597', course: 'book-foundations', chapter: 'ch10-00-generics.html' },
  { id: 'd1', title: 'D1 · Cargo 与工程组织', terms: 'workspace feature 工作空间 package crate publish 配置 E0432 E0433', course: 'book-d1', chapter: 'ch14-00-more-about-cargo.html' },
  { id: 'd2', title: 'D2 · 智能指针', terms: 'Box Rc RefCell Weak Deref Drop 引用循环 内部可变性', course: 'book-d2', chapter: 'ch15-00-smart-pointers.html' },
  { id: 'd3', title: 'D3 · 对象与模式', terms: '面向对象 trait object dyn 模式 match if let 穷尽性 状态模式', course: 'book-d3', chapter: 'ch18-00-oop.html' },
  { id: 'd4', title: 'D4 · 高级 Rust', terms: 'unsafe 原始指针 关联类型 macro 宏 过程宏 函数指针 闭包', course: 'book-d4', chapter: 'ch20-00-advanced-features.html' },
  { id: 'd5', title: 'D5 · 多线程 Web Server', terms: '并发 thread 线程池 HTTP 有界 优雅关闭 shutdown channel mpsc', course: 'book-d5', chapter: 'ch21-00-final-project-a-web-server.html' },
  { id: 'd6a', title: 'D6-A · 附录与工具', terms: '关键字 derive rustfmt clippy rustdoc rustup Edition nightly 工具链', course: 'book-d6', chapter: 'appendix-00.html' },
  { id: 'd6b', title: 'D6-B · 基础概念综合', terms: '变量 函数 分支 循环 模块 测试 泛型 所有权 字符串', course: 'book-foundations', chapter: 'ch03-00-common-programming-concepts.html' },
  { id: "d8", title: "D8 · Edition 2024 告警筛选", terms: "Edition 2024 let chains gen macro 编译器 工具链", course: "modern-d8", chapter: null },
  { id: "d9a", title: "D9-A · 现代语言特性", terms: "GAT RPITIT precise capture async closure never upcast DropGuard allocator", course: "modern-d9a", chapter: null },
  { id: "d9b", title: "D9-B · 现代标准库", terms: "Result inspect LazyLock RwLock downgrade array_windows CString bit_width Cargo", course: "modern-d9b", chapter: null },
  { id: "d10a", title: "D10-A · 异步核心", terms: "Future Poll Pending Waker Pin Unpin join race signal executor", course: "modern-d10a", chapter: null },
  { id: "d11a", title: "D11-A · 工程质量", terms: "测试 frame checksum Clippy rustfmt GDB benchmark property", course: "modern-d11a", chapter: null },
  { id: "e4", title: "E4 · C ABI 与 WebAssembly", terms: "C ABI FFI unsafe repr C WASM WebAssembly DLL so 指针", course: "modern-e4", chapter: null },
  { id: "e5", title: "E5 · 科学计算与机器学习", terms: "Welford logistic regression 逻辑回归 CPU 机器学习 holdout confusion matrix 数据泄漏", course: "modern-e5", chapter: null },
  { id: "d7", title: "D7 · 新版 Rust Book 映射", terms: "新版 Book 覆盖 审计", course: null, chapter: null },
  { id: "d10b", title: "D10-B · Tokio 与 Stream", terms: "Tokio Stream async select cancel", course: "modern-d10b", chapter: null },
  { id: "d11b", title: "D11-B · 安全与模糊测试", terms: "Miri libFuzzer RustSec unsafe fuzz", course: "modern-d11b", chapter: null },
  { id: "e1a", title: "E1-A · HTTPS 与授权", terms: "TLS HTTPS 认证 授权", course: "modern-e1a", chapter: null },
  { id: "e1b", title: "E1-B · SQLite 与可观测性", terms: "SQLite 事务 日志 监测", course: "modern-e1b", chapter: null },
  { id: "e2", title: "E2 · 原生 GUI", terms: "egui eframe GUI 桌面 双语", course: "modern-e2", chapter: null },
  { id: "e3", title: "E3 · 嵌入式", terms: "no_std HAL Cortex-M QEMU SysTick", course: "modern-e3", chapter: null },
  { id: 'gap31', title: '31 项概念 · 对照与迁移', terms: '4.1 4.2 4.3 5.1 5.2 5.3 6.1 6.3 抽样 补课 对照 实验', course: 'book-foundations', chapter: 'ch00-00-introduction.html' },
  { id: 'errors', title: '编译错误 · 失败与修复', terms: '报错 E0382 E0502 E0308 E0499 E0597 E0596 E0277 E0599 E0425 E0432 E0433', course: 'book-foundations', chapter: 'ch04-02-references-and-borrowing.html' },
  { id: 'evidence', title: '记录理解与迁移', terms: '学员 预测 提示 记录 学习效果 自己输入 解释 复测 观察', course: null, chapter: null },
]);
let bookRequest = 0;

function knowledgeMatches(query = '') {
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  return KNOWLEDGE_TOPICS.filter(topic => terms.every(term => `${topic.id} ${topic.title} ${topic.terms}`.toLocaleLowerCase().includes(term)));
}

function bookCourseFor(topic) {
  return KNOWLEDGE_TOPICS.find(item => item.id === topic)?.course || null;
}

function allowedBookURL(url) {
  return /^https:\/\/(rust\.bootcss\.com|doc\.rust-lang\.org)\//.test(url || '');
}

function localTopicForURL(url) {
  if (/ch04-|ch05-/.test(url)) return 'ownership';
  if (/ch08-|ch09-/.test(url)) return 'collections';
  if (/ch10-/.test(url)) return 'generics';
  if (/ch01-03|ch07-|ch14-|cargo\/reference/.test(url)) return 'd1';
  if (/ch15-/.test(url)) return 'd2';
  if (/ch16-|ch21-/.test(url)) return 'd5';
  return 'd6b';
}

function bookLink(caption, url) {
  const link = document.createElement('a');
  link.textContent = caption;
  link.href = url;
  link.target = '_blank';
  link.rel = 'noopener noreferrer';
  link.referrerPolicy = 'no-referrer';
  return link;
}

// Text-only inline parser: raw HTML, images and arbitrary URLs are never executed.
function markdownInline(parent, value) {
  const pattern = /(`[^`]+`|\*\*[^*]+\*\*|\[([^\]]+)\]\(([^)]+)\))/g;
  let offset = 0;
  for (const match of value.matchAll(pattern)) {
    parent.append(document.createTextNode(value.slice(offset, match.index)));
    const part = match[0];
    if (part.startsWith('`') || part.startsWith('**')) {
      const strong = part.startsWith('**');
      const node = document.createElement(strong ? 'strong' : 'code');
      node.textContent = part.slice(strong ? 2 : 1, strong ? -2 : -1);
      parent.append(node);
    } else if (allowedBookURL(match[3])) {
      parent.append(bookLink(match[2], match[3]));
    } else {
      parent.append(document.createTextNode(match[2]));
    }
    offset = match.index + part.length;
  }
  parent.append(document.createTextNode(value.slice(offset)));
}

function markdownCells(line) {
  const cells = [];
  let value = '', code = false, escaped = false;
  for (const ch of line.trim().replace(/^\|/, '').replace(/\|$/, '')) {
    if (ch === '|' && !code && !escaped) { cells.push(value.trim()); value = ''; }
    else { value += ch; if (ch === '`' && !escaped) code = !code; }
    escaped = ch === '\\' && !escaped;
  }
  cells.push(value.trim());
  return cells;
}

function renderMarkdown(container, source) {
  container.replaceChildren();
  const lines = source.replace(/\r\n/g, '\n').split('\n');
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) { i++; continue; }
    if (/^\s*```/.test(line)) {
      const pre = document.createElement('pre');
      const code = document.createElement('code');
      const collected = [];
      for (i++; i < lines.length && !/^\s*```/.test(lines[i]); i++) collected.push(lines[i]);
      code.textContent = collected.join('\n'); pre.append(code); container.append(pre); i++; continue;
    }
    if (line.includes('|') && i + 1 < lines.length && /^\s*\|?[\s:|-]+\|[\s:|-]*$/.test(lines[i + 1])) {
      const wrap = document.createElement('div'); wrap.className = 'lesson-table';
      const table = document.createElement('table');
      const head = document.createElement('thead'); const row = document.createElement('tr');
      for (const value of markdownCells(line)) { const cell = document.createElement('th'); markdownInline(cell, value); row.append(cell); }
      head.append(row); table.append(head); i += 2;
      const body = document.createElement('tbody');
      while (i < lines.length && lines[i].trim().startsWith('|')) {
        const row = document.createElement('tr');
        for (const value of markdownCells(lines[i++])) { const cell = document.createElement('td'); markdownInline(cell, value); row.append(cell); }
        body.append(row);
      }
      table.append(body); wrap.append(table); container.append(wrap); continue;
    }
    const heading = /^(#{1,6})\s+(.+)$/.exec(line);
    if (heading) {
      const node = document.createElement(`h${Math.min(heading[1].length + 1, 6)}`);
      markdownInline(node, heading[2]); container.append(node); i++; continue;
    }
    if (/^\s*(?:[-*]|\d+[.)])\s+/.test(line)) {
      const list = document.createElement(/^\s*\d/.test(line) ? 'ol' : 'ul');
      while (i < lines.length && /^\s*(?:[-*]|\d+[.)])\s+/.test(lines[i])) {
        const item = document.createElement('li'); markdownInline(item, lines[i++].replace(/^\s*(?:[-*]|\d+[.)])\s+/, '')); list.append(item);
      }
      container.append(list); continue;
    }
    const p = document.createElement('p'); markdownInline(p, line.replace(/^>\s*/, '')); container.append(p); i++;
  }
}

function renderKnowledgeResults() {
  const found = knowledgeMatches(ui.knowledgeSearch.value);
  ui.knowledgeResults.replaceChildren();
  for (const item of found) {
    const control = button(item.title, 'topic', item.id);
    control.className = 'knowledge-link';
    control.setAttribute('aria-current', item.id === ui.bookSelect.value ? 'true' : 'false');
    ui.knowledgeResults.append(control);
  }
  text(ui.knowledgeCount, found.length ? `${found.length} 个相关主题` : '没有匹配主题；试试 Rust 名称或错误编号');
}

async function loadBook() {
  const id = ui.bookSelect.value;
  const request = ++bookRequest;
  text(ui.bookStatus, '正在读取离线讲义…');
  ui.bookText.replaceChildren();
  const topic = KNOWLEDGE_TOPICS.find(item => item.id === id);
  ui.bookPractice.hidden = !topic?.course;
  ui.bookPractice.disabled = Boolean(state.busy || state.pending || isExperimentRunning());
  ui.bookOfficial.hidden = !topic?.chapter;
  if (topic?.chapter) ui.bookOfficial.href = `https://doc.rust-lang.org/book/${topic.chapter}`;
  renderKnowledgeResults();
  try {
    const material = await api('/api/book', { query: { id } });
    if (request !== bookRequest) return;
    renderMarkdown(ui.bookText, material.markdown);
    text(ui.bookStatus, material.title);
    ui.bookText.scrollTop = 0;
  } catch (error) {
    if (request === bookRequest) text(ui.bookStatus, error.message);
  }
}

function openBook(topic = 'ownership') {
  if (!KNOWLEDGE_TOPICS.some(item => item.id === topic)) return;
  ui.bookSelect.value = topic;
  ui.knowledgeSearch.value = '';
  if (!ui.bookDialog.open) ui.bookDialog.showModal();
  void loadBook();
}

function renderReading(reading) {
  if (!ui.reading || typeof document.createElement !== 'function') return;
  const entries = reading?.topics?.length ? reading.topics : [];
  ui.reading.hidden = entries.length === 0;
  ui.readingTopics.replaceChildren();
  for (const topic of entries) {
    const item = document.createElement('li');
    const label = document.createElement('strong'); label.textContent = topic.title; item.append(label);
    for (const [caption, url] of [['中文旧版', topic.chinese], ['当前官方', topic.official]]) {
      if (allowedBookURL(url)) item.append(bookLink(caption, url));
    }
    const local = button('本地讲义与实验', 'topic', localTopicForURL(topic.official));
    local.className = 'text-button'; item.append(local); ui.readingTopics.append(item);
  }
  text(ui.readingQuestion, reading ? '思考：' + reading.question : '');
  text(ui.readingVersion, reading?.version_note || '');
}

function initializeKnowledge() {
  for (const [name, id] of Object.entries({ knowledgeSearch: 'knowledge-search', knowledgeResults: 'knowledge-results', knowledgeCount: 'knowledge-count', bookPractice: 'book-practice', bookOfficial: 'book-official', learningLog: 'learning-log-button' })) ui[name] = document.getElementById(id);
  ui.bookSelect.replaceChildren(...KNOWLEDGE_TOPICS.map(topic => new Option(topic.title, topic.id)));
  ui.bookOpen.addEventListener('click', () => openBook(state.card?.lesson_topic || 'ownership'));
  ui.bookClose.addEventListener('click', () => ui.bookDialog.close());
  ui.bookSelect.addEventListener('change', () => { void loadBook(); });
  ui.knowledgeSearch.addEventListener('input', renderKnowledgeResults);
  ui.knowledgeResults.addEventListener('click', event => {
    const control = event.target.closest('[data-topic]');
    if (control) { ui.bookSelect.value = control.dataset.topic; void loadBook(); }
  });
  ui.readingTopics.addEventListener('click', event => {
    const control = event.target.closest('[data-topic]');
    if (control) openBook(control.dataset.topic);
  });
  ui.bookPractice.addEventListener('click', async () => {
    if (state.busy || state.pending || isExperimentRunning()) return;
    const course = bookCourseFor(ui.bookSelect.value);
    if (!course) return;
    await selectCourse(course, 1);
    if (state.card?.id === course) ui.bookDialog.close();
  });
  ui.learningLog.addEventListener('click', async () => {
    if (state.busy || state.pending || isExperimentRunning()) return;
    if (state.card?.project?.files.includes('LEARNING_LOG.md')) await selectFile('LEARNING_LOG.md');
    else openBook('evidence');
  });
}
