//! 离线可打开的案件进度板。静态 HTML 仅展示最新一次 status 的真实结果。
use crate::{case, checks::Step};
use std::path::{Path, PathBuf};

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn render(steps: &[Step]) -> String {
    let passed = steps.iter().filter(|step| step.passed).count();
    let mut output = format!(
        r#"<!doctype html><html lang="zh-CN"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>LabQuest · CASE 01</title>
<style>
:root{{--bg:#0d1728;--panel:#182740;--line:#2a3c58;--ink:#f4f7ff;--muted:#abbdd5;--blue:#72a9ff;--green:#65dfae;--orange:#ffcb73}}
*{{box-sizing:border-box}}body{{margin:0;background:var(--bg);color:var(--ink);font:16px/1.6 system-ui,sans-serif}}
main{{max-width:920px;margin:0 auto;padding:clamp(18px,4vw,48px)}}h1{{font-size:clamp(28px,5vw,42px);margin:10px 0 5px}}
p{{color:var(--muted)}}.tag{{font-size:12px;letter-spacing:.15em;color:var(--blue);font-weight:750}}
.hero{{display:flex;gap:20px;align-items:center;justify-content:space-between;flex-wrap:wrap}}.number{{font-size:40px;font-weight:800;color:var(--green)}}
.track{{height:12px;background:#293853;border-radius:100px;overflow:hidden;margin:20px 0 24px}}.fill{{height:100%;width:{:.1}%;background:linear-gradient(90deg,var(--blue),var(--green))}}
.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(270px,1fr));gap:12px}}
.card{{background:var(--panel);padding:18px;border:1px solid var(--line);border-radius:14px}}
.card.ok{{border-color:#337e66}}.status{{color:var(--orange);font-size:12px;font-weight:750}}.ok .status{{color:var(--green)}}
.card h2{{font-size:18px;margin:6px 0}}.card p{{margin:8px 0 0;font-size:14px}}
.links{{display:flex;gap:10px;flex-wrap:wrap;margin:24px 0}}a{{color:var(--blue)}}.links a{{padding:8px 12px;background:var(--panel);border-radius:9px}}
footer{{font-size:13px;color:var(--muted);margin-top:22px}}
</style></head><body><main><div class="tag">RUST APPLAB / CASE 01 / OFFLINE</div>
<div class="hero"><div><h1>网关超时事故调查</h1><p>从文件现场到可复算的软件修复证据</p></div><div><div class="number">{passed}/{total}</div><small>已通过的真实验收项</small></div></div>
<div class="track" role="progressbar" aria-label="完成进度" aria-valuemin="0" aria-valuemax="{total}" aria-valuenow="{passed}"><div class="fill"></div></div>
<div class="grid">"#,
        100.0 * passed as f64 / steps.len().max(1) as f64,
        total = steps.len()
    );
    for step in steps {
        let class = if step.passed { "card ok" } else { "card" };
        let label = if step.passed {
            "已验证"
        } else {
            "待行动"
        };
        output.push_str(&format!(
            "<section class=\"{class}\"><div class=\"status\">{label} · {:02}</div><h2>{}</h2><p>{}</p></section>",
            step.number, escape(step.name), escape(&step.feedback)
        ));
    }
    output.push_str(r#"</div><nav class="links"><a href="inventory.html">文件地图</a><a href="incident.md">事故记录</a><a href="current.md">修复报告</a><a href="PROGRAM_PROOF.txt">代码运行证据</a><a href="CASE_REPORT.md">最终报告</a></nav>
<footer>这是离线静态快照，不会自动刷新。每完成一项后重新运行 labquest board &lt;案例目录&gt; 更新页面。不要把教学模拟的错误比例当作真实服务可用率。</footer></main></body></html>"#);
    output
}

pub fn write(root: &Path, steps: &[Step]) -> Result<PathBuf, String> {
    case::ensure_case(root)?;
    let path = root.join("deliverables/CASE_BOARD.html");
    case::write_owned(&path, &render(steps))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn board_escapes_feedback() {
        let html = render(&[Step {
            number: 1,
            name: "<alert>",
            passed: false,
            feedback: "\"><script>alert(1)</script>".into(),
        }]);
        assert!(html.contains("&lt;alert&gt;"));
        assert!(!html.contains("<script>"));
    }
}
