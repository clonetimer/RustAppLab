"use strict";
// 永远只用 textContent 写入用户可控的路径或日志字符串，不拼接 HTML。
const byId = (id) => document.getElementById(id);
const print = (id, value) => { byId(id).textContent = String(value); };
const size = (n) => {
  if (n < 1024) return `${n} B`;
  const units = ["KiB", "MiB", "GiB", "TiB", "PiB"];
  let i = -1;
  do { n /= 1024; i++; } while (n >= 1024 && i < units.length - 1);
  return `${n.toFixed(1)} ${units[i]}`;
};

async function request(path) {
  const response = await fetch(path, { cache: "no-store" });
  if (!response.ok) throw new Error(`${path}: HTTP ${response.status}`);
  return response.json();
}
function showFiles(data) {
  print("fileCount", data.stats.files);
  print("directoryCount", data.stats.directories);
  print("totalBytes", size(data.stats.total_bytes));
  print("problemCount", data.stats.problems);
  print("scanRoot", `扫描根目录：${data.root}`);
  byId("extensions").replaceChildren();
  for (const item of data.extensions) {
    const span = document.createElement("span");
    span.textContent = `${item.extension} · ${item.count}`;
    byId("extensions").append(span);
  }
  const rows = [];
  for (const file of data.files) {
    const tr = document.createElement("tr");
    for (const value of [file.path, file.extension, file.bytes]) {
      const td = document.createElement("td");
      td.textContent = String(value);
      tr.append(td);
    }
    tr.lastChild.className = "right";
    rows.push(tr);
  }
  byId("fileRows").replaceChildren(...rows);
  function filter() {
    const query = byId("search").value.toLocaleLowerCase();
    let visible = 0;
    for (const row of rows) {
      const isMatch = row.firstChild.textContent.toLocaleLowerCase().includes(query);
      row.hidden = !isMatch;
      if (isMatch) visible++;
    }
    print("shown", `当前展示 ${visible} / ${data.shown} 条 · ${data.truncated ? "仅返回前 200 条，可通过 API limit 参数查看更多" : "完整快照"}`);
  }
  byId("search").addEventListener("input", filter);
  filter();
}
function showLogs(data) {
  if (!data.enabled) {
    print("logSource", "未配置日志：启动时增加 --log <日志路径> 即可使用此面板。");
    print("records", "未启用");
    print("errorRate", "—");
    return;
  }
  print("logSource", `日志文件：${data.input}`);
  print("records", data.records);
  print("errorRate", `${data.error_rate.toFixed(2)}%`);
  print("logWarning", `ERROR/FATAL ${data.errors} 条；未识别 ${data.skipped} 行。这里是记录级错误占比，不代表请求失败率。`);
  const max = Math.max(1, ...data.levels.map((level) => level.count));
  const levels = byId("levelList");
  levels.replaceChildren();
  for (const item of data.levels) {
    const row = document.createElement("div");
    row.className = "level-row";
    const label = document.createElement("span"); label.textContent = item.level;
    const track = document.createElement("div"); track.className = "track";
    const bar = document.createElement("div"); bar.className = "bar";
    bar.style.width = `${item.count / max * 100}%`;
    track.append(bar);
    const count = document.createElement("b"); count.textContent = item.count;
    row.append(label, track, count);
    levels.append(row);
  }
  const list = byId("topMessages");
  list.replaceChildren();
  for (const item of data.top_messages) {
    const li = document.createElement("li");
    li.textContent = item.message;
    const times = document.createElement("small");
    times.textContent = ` · ${item.count} 次`;
    li.append(times);
    list.append(li);
  }
}
Promise.all([request("/api/files?limit=200"), request("/api/logs")])
  .then(([files, logs]) => {
    showFiles(files);
    showLogs(logs);
    print("status", "✓ 数据来自 LocalHub 本地只读快照");
  })
  .catch((error) => print("status", `读取失败：${error.message}`));
