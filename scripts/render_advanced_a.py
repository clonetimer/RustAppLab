#!/usr/bin/env python3
"""Build a self-contained offline guide; Markdown is the single source."""
from pathlib import Path
import html,re
ROOT=Path(__file__).resolve().parents[1]
def inline(text):
    text=html.escape(text)
    text=re.sub(r'`([^`]+)`',r'<code>\1</code>',text)
    text=re.sub(r'\*\*([^*]+)\*\*',r'<strong>\1</strong>',text)
    text=re.sub(r'\[([^]]+)\]\((https?://[^)]+)\)',r'<a href="\2">\1</a>',text)
    return text
lines=(ROOT/'course/ADVANCED_A.md').read_text().splitlines();parts=[];code=[];in_code=False;table=False;details=False
for line in lines:
    if line.startswith('```'):
        if in_code: parts.append('<pre><code>'+html.escape('\n'.join(code))+'</code></pre>');code=[]
        in_code=not in_code;continue
    if in_code:code.append(line);continue
    if table and not line.startswith('|'):parts.append('</tbody></table></div>');table=False
    if line.startswith('### 提示 '):
        if details:parts.append('</details>')
        parts.append('<details><summary>'+inline(line[4:])+'</summary>');details=True;continue
    if details and line.startswith('#'):parts.append('</details>');details=False
    if line.startswith('|'):
        if re.fullmatch(r'[| :\-]+',line):continue
        if not table:parts.append('<div class="table"><table><tbody>');table=True
        parts.append('<tr>'+''.join('<td>'+inline(x.strip())+'</td>' for x in line.strip('|').split('|'))+'</tr>');continue
    m=re.match(r'^(#{1,3}) (.+)',line)
    if m:
        n=len(m[1]);idattr=' id="a'+m[2][1]+'"' if m[2].startswith(('A1 ','A2 ','A3 ')) else ''
        parts.append(f'<h{n}{idattr}>'+inline(m[2])+f'</h{n}>')
    elif line.strip():parts.append('<p>'+inline(line)+'</p>')
if details:parts.append('</details>')
CSS='''*{box-sizing:border-box}body{margin:0;background:#f3f5f8;color:#18283b;font:17px/1.8 system-ui,"Microsoft YaHei",sans-serif}header{background:#173746;color:white;padding:28px max(20px,calc((100vw - 1020px)/2))}header p{margin:0;color:#d0e5e9}nav{display:flex;gap:12px;flex-wrap:wrap;margin-top:18px}nav a{background:#ffffff18;border:1px solid #ffffff50;color:white;padding:6px 14px;border-radius:8px;text-decoration:none}main{max-width:1080px;margin:auto;padding:28px}h1{font-size:32px;line-height:1.35}h2{margin-top:44px;border-top:2px solid #cddbe0;padding-top:22px}h3{font-size:20px}p{overflow-wrap:anywhere}pre{background:#17232e;color:#f2f5f9;padding:18px;border-radius:10px;overflow:auto;font:14px/1.8 Consolas,monospace}p code,td code{background:#e4ebef;padding:2px 4px;border-radius:3px}.table{overflow:auto}table{width:100%;border-collapse:collapse;font-size:15px;background:white}td{border:1px solid #d5e0e5;padding:10px;min-width:120px}tr:first-child{font-weight:bold;background:#e6f0f2}details{border:1px solid #b7d0da;background:#fff;border-radius:9px;padding:12px 18px;margin:12px 0}summary{cursor:pointer;font-weight:bold;color:#155069}a{color:#006882}footer{margin-top:36px;font-size:14px;color:#546779}@media(max-width:600px){main{padding:14px}h1{font-size:25px}body{font-size:16px}pre{padding:12px}h2{font-size:22px}}'''
out='<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>进阶 A · TermGarden 工作区</title><style>'+CSS+'</style></head><body><header><p>Rust AppLab / 基础阶段之后</p><h1>一份任务 · 多个视图 · 明确的所有权</h1><p>可运行的工作区增强，不是重做基础语法。先体验，再设计，最后独立交付。</p><nav><a href="#a1">A1 共享与借用</a><a href="#a2">A2 编辑与撤销</a><a href="#a3">A3 快照与线程</a></nav></header><main>'+''.join(parts)+'<footer>离线页面，不运行代码、不联网提交、不自动认证学习结果。提示可展开；源讲义：course/ADVANCED_A.md。</footer></main></body></html>'
(ROOT/'ADVANCED_A.html').write_text(out,encoding='utf-8')
print('Rendered ADVANCED_A.html; sections=3, hints=9')
