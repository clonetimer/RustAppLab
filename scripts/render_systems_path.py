#!/usr/bin/env python3
"""Maintainer renderer; shipped HTML needs no Python or network. Optional: markdown-it-py."""
from pathlib import Path
from markdown_it import MarkdownIt
ROOT = Path(__file__).resolve().parents[1]
md = MarkdownIt('commonmark', {'html': False}).enable('table')
body = md.render((ROOT/'course/SYSTEMS_PATH.md').read_text(encoding='utf-8'))
# Keep tables scrollable without forcing the whole mobile viewport to scroll sideways.
body = body.replace('<table>', '<div class="table-scroll"><table>').replace('</table>', '</table></div>')
page = '''<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>个人工单档案 · Rust AppLab</title><style>
*{box-sizing:border-box}body{margin:0;background:#f4f7fb;color:#23364d;font:16px/1.85 system-ui,"Microsoft YaHei",sans-serif}header{background:#17324d;color:white}header>div,main{max-width:960px;margin:auto;padding:28px 24px}h1{font-size:clamp(26px,4vw,37px);line-height:1.35}h2{margin-top:40px;padding-top:16px;border-top:1px solid #d9e3ed;font-size:24px;line-height:1.45}h3{font-size:19px}article{background:white;border:1px solid #d9e3ed;border-radius:15px;padding:16px 28px}p,li{overflow-wrap:anywhere}code{font:14px/1.7 Consolas,monospace;background:#edf3f7;padding:2px 4px;border-radius:4px;overflow-wrap:anywhere}pre{background:#17324d;color:white;border-radius:9px;padding:16px;white-space:pre-wrap;overflow-wrap:anywhere}pre code{background:none;padding:0}a{color:#16639c}.table-scroll{overflow-x:auto}table{border-collapse:collapse;min-width:520px;width:100%}td,th{border:1px solid #d9e3ed;padding:10px;vertical-align:top;text-align:left}header p{margin:0}.stamp{font-size:13px;letter-spacing:.14em}@media(max-width:550px){header>div,main{padding:20px 14px}article{padding:12px 16px}h2{font-size:21px}}</style></head><body><header><div><span class="stamp">RUST APPLAB / SYSTEMS PATH / 0.18.0-dev.5</span><p>四款自己的软件 · 一份可保存的个人工单档案</p></div></header><main><article>'''+body+'''</article></main></body></html>'''
(ROOT/'SYSTEMS_PATH.html').write_text(page,encoding='utf-8')
print('Rendered SYSTEMS_PATH.html from course/SYSTEMS_PATH.md')
