//! 将真实的扫描数据转成一个可离线打开的 HTML 应用页面。
use crate::scanner::{format_size, ScanSummary};
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;

const STYLE: &str = r#"
:root { color-scheme: light; --ink:#17243b; --muted:#5d6b7d; --line:#e6ebf2; --navy:#172b4d; --blue:#2463d9; --paper:#f3f6fb; }
* { box-sizing:border-box; }
body { margin:0; background:var(--paper); color:var(--ink); font:15px/1.6 system-ui, -apple-system, 'Segoe UI', sans-serif; }
header { background:linear-gradient(115deg, #11294d, #2556a2); color:white; padding:42px max(24px,calc((100% - 1120px)/2)); }
header .tag { opacity:.8; font-size:12px; letter-spacing:.13em; text-transform:uppercase; }
header h1 { margin:9px 0 5px; font-size:clamp(26px, 5vw, 40px); line-height:1.2; }
header .path { color:#c9def9; word-break:break-all; }
main { max-width:1160px; margin:0 auto; padding:28px 20px 56px; }
.cards { display:grid; grid-template-columns:repeat(4,minmax(0,1fr)); gap:14px; margin-bottom:22px; }
.card, section { border:1px solid var(--line); border-radius:16px; background:white; box-shadow:0 4px 14px rgba(29,49,83,.035); }
.card { padding:16px 19px; }
.card .k { color:var(--muted); font-size:13px; }
.card .v { margin-top:5px; font-size:25px; font-weight:750; letter-spacing:-.02em; }
section { padding:22px 24px; margin-bottom:20px; }
section h2 { font-size:18px; margin:0 0 16px; }
.sub { color:var(--muted); font-size:13px; margin:-8px 0 16px; }
.two { display:grid; grid-template-columns:1fr 1fr; gap:20px; }
.bar-row { display:grid; grid-template-columns:minmax(65px,130px) minmax(0,1fr) 50px; align-items:center; gap:12px; margin:12px 0; }
.bar-name { overflow:hidden; white-space:nowrap; text-overflow:ellipsis; font-size:13px; }
.track { height:11px; background:#edf3fd; border-radius:30px; overflow:hidden; }
.fill { height:100%; background:linear-gradient(90deg,#3a82ea,#54b4b4); border-radius:30px; }
.count { text-align:right; color:var(--muted); font-variant-numeric:tabular-nums; }
.table-wrap { overflow:auto; max-height:540px; border:1px solid var(--line); border-radius:10px; }
table { border-collapse:collapse; width:100%; text-align:left; }
tr[hidden] { display:none; }
th, td { border-bottom:1px solid var(--line); padding:10px 14px; font-size:13px; }
th { position:sticky; top:0; background:#f6f9fe; color:#51627b; font-weight:650; }
td.path { font-family:ui-monospace, SFMono-Regular, Consolas, monospace; overflow-wrap:anywhere; }
td.size { text-align:right; white-space:nowrap; font-variant-numeric:tabular-nums; }
input { display:block; width:min(420px,100%); border:1px solid #b8c9e0; outline-color:var(--blue); border-radius:9px; font:inherit; padding:10px 14px; margin:8px 0 16px; }
.pill { background:#eaf1ff; border-radius:6px; padding:2px 7px; font-size:12px; }
.note { color:var(--muted); font-size:13px; }
.warn { background:#fff8eb; border-left:3px solid #e5a536; padding:11px 14px; border-radius:6px; overflow-wrap:anywhere; }
footer { text-align:center; color:var(--muted); font-size:12px; margin-top:30px; }
@media(max-width:760px) { .cards { grid-template-columns:repeat(2,1fr); } .two { grid-template-columns:1fr; } section { padding:18px; } }
"#;

const SCRIPT: &str = r#"
(function () {
  const field = document.getElementById('search');
  const rows = Array.from(document.querySelectorAll('#files tbody tr'));
  const count = document.getElementById('visible');
  if (!field || !count) return;
  function filter() {
    const text = field.value.toLocaleLowerCase();
    let visible = 0;
    for (const row of rows) {
      const matched = row.textContent.toLocaleLowerCase().includes(text);
      row.hidden = !matched;
      if (matched) visible++;
    }
    count.textContent = String(visible);
  }
  field.addEventListener('input', filter);
  filter();
})();
"#;

pub fn escape_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(ch),
        }
    }
    result
}

pub fn render(summary: &ScanSummary, limit: usize) -> String {
    let mut html = String::with_capacity(36_000);
    let root = escape_html(&summary.root.to_string_lossy());
    write!(
        html,
        "<!doctype html><html lang=\"zh-CN\"><head><meta charset=\"utf-8\">\
        <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
        <title>FileQuest · 本地目录报告</title><style>{STYLE}</style></head><body>\
        <header><div class=\"tag\">Rust AppLab / FileQuest 0.1</div><h1>本地文件雷达</h1>\
        <div class=\"path\">{root}</div></header><main>"
    )
    .expect("写入 String 不会失败");
    write!(
        html,
        "<div class=\"cards\">\
        <div class=\"card\"><div class=\"k\">文件总数</div><div class=\"v\">{}</div></div>\
        <div class=\"card\"><div class=\"k\">总文件体积</div><div class=\"v\">{}</div></div>\
        <div class=\"card\"><div class=\"k\">已扫描子目录</div><div class=\"v\">{}</div></div>\
        <div class=\"card\"><div class=\"k\">访问异常</div><div class=\"v\">{}</div></div></div>",
        summary.files.len(),
        format_size(summary.total_bytes()),
        summary.directories,
        summary.problems.len(),
    )
    .expect("写入 String 不会失败");

    html.push_str("<div class=\"two\"><section><h2>文件类型分布</h2><p class=\"sub\">按文件数量统计（不是体积）</p>");
    let counts = summary.extension_counts();
    let max_count = counts.first().map_or(1, |entry| entry.1.max(1));
    for (ext, count) in counts.iter().take(8) {
        let percent = (*count as f64) * 100.0 / (max_count as f64);
        write!(
            html,
            "<div class=\"bar-row\"><span class=\"bar-name\" title=\"{}\">{}</span>\
            <div class=\"track\"><div class=\"fill\" style=\"width:{percent:.1}%\"></div></div>\
            <span class=\"count\">{count}</span></div>",
            escape_html(ext),
            escape_html(ext)
        )
        .expect("写入 String 不会失败");
    }
    if counts.is_empty() {
        html.push_str("<p class=\"note\">该目录没有可展示的普通文件。</p>");
    }
    html.push_str("</section><section><h2>体积最大的文件</h2><p class=\"sub\">用于定位大体积产物</p><div class=\"table-wrap\"><table><thead><tr><th>路径</th><th>大小</th></tr></thead><tbody>");
    for record in summary.largest_files(8) {
        write!(
            html,
            "<tr><td class=\"path\">{}</td><td class=\"size\">{}</td></tr>",
            escape_html(&record.relative_path.to_string_lossy()),
            format_size(record.bytes)
        )
        .expect("写入 String 不会失败");
    }
    html.push_str("</tbody></table></div></section></div>");

    let shown = summary.files.len().min(limit);
    write!(
        html,
        "<section><h2>文件清单</h2><p class=\"sub\">显示前 {shown} / {} 个文件（按路径排序）；\
        搜索只覆盖当前显示的文件。可通过命令行参数 --limit 调整。</p>\
        <label for=\"search\">按路径或扩展名筛选</label>\
        <input id=\"search\" placeholder=\"例如：src、rs、logs\" autocomplete=\"off\">\
        <div class=\"note\">当前匹配 <strong id=\"visible\">{shown}</strong> 条</div>\
        <div class=\"table-wrap\"><table id=\"files\"><thead><tr>\
        <th>相对路径</th><th>类型</th><th>大小</th></tr></thead><tbody>",
        summary.files.len()
    )
    .expect("写入 String 不会失败");
    for file in summary.files.iter().take(limit) {
        write!(
            html,
            "<tr><td class=\"path\">{}</td><td><span class=\"pill\">{}</span></td><td class=\"size\">{}</td></tr>",
            escape_html(&file.relative_path.to_string_lossy()),
            escape_html(&file.extension),
            format_size(file.bytes)
        )
        .expect("写入 String 不会失败");
    }
    html.push_str("</tbody></table></div></section>");

    write!(
        html,
        "<section><h2>扫描边界与提示</h2><p class=\"note\">跳过的构建/依赖目录：{}；\
        跳过的符号链接：{}。本工具不会跟随符号链接，不读取文件内容，\
        不删除、移动或修改被扫描的文件。默认忽略常见构建目录，可添加 --all 扫描。</p>",
        summary.ignored_directories, summary.skipped_symlinks,
    )
    .expect("写入 String 不会失败");
    for problem in summary.problems.iter().take(12) {
        write!(html, "<p class=\"warn\">{}</p>", escape_html(problem))
            .expect("写入 String 不会失败");
    }
    if summary.problems.len() > 12 {
        html.push_str("<p class=\"note\">更多异常请使用命令行 scan 查看。</p>");
    }
    html.push_str(
        "</section><footer>FileQuest · 完全离线的静态报告 · 数据保存在你本机</footer></main>",
    );
    write!(html, "<script>{SCRIPT}</script></body></html>").expect("写入 String 不会失败");
    html
}

/// 默认只创建新报告，不覆盖既有文件；--force 才允许覆盖常规文件。
pub fn write_report(path: &Path, html: &str, force: bool) -> io::Result<()> {
    if force
        && matches!(
            fs::symlink_metadata(path),
            Ok(metadata) if !metadata.file_type().is_file()
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--force 只能覆盖已有的普通文件，不能覆盖链接或目录",
        ));
    }
    let mut options = OpenOptions::new();
    options.write(true);
    if force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(path)?;
    file.write_all(html.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_is_escaped() {
        assert_eq!(
            escape_html("<script a='\"&\">"),
            "&lt;script a=&#39;&quot;&amp;&quot;&gt;"
        );
    }
}
