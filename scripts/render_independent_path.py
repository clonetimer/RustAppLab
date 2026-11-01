#!/usr/bin/env python3
"""Maintainer renderer; delivered pages are fully offline and need no Python."""
from pathlib import Path
import html
from markdown_it import MarkdownIt
ROOT = Path(__file__).resolve().parents[1]
STYLE = '''*{box-sizing:border-box}body{margin:0;background:#f3f6fa;color:#25354c;font:16px/1.85 system-ui,"Microsoft YaHei",sans-serif}header{background:#172f4c;color:white}header>div,main{max-width:980px;margin:auto;padding:24px}header a{color:#d0eaff}h1{font-size:clamp(26px,4vw,36px);line-height:1.4}h2{margin-top:38px;padding-top:18px;border-top:1px solid #d8e1ee;line-height:1.5;font-size:24px}h3{font-size:19px}article{background:white;border:1px solid #d8e1ee;border-radius:16px;padding:20px 28px}p,li{overflow-wrap:anywhere}pre{background:#172f4c;color:white;padding:16px;border-radius:9px;white-space:pre-wrap;overflow-wrap:anywhere}code{font:14px/1.75 Consolas,monospace;background:#ecf1f7;padding:2px 4px;border-radius:3px}pre code{background:none;padding:0}.table-scroll{overflow:auto}table{border-collapse:collapse;width:100%;min-width:520px}td,th{padding:10px;border:1px solid #d8e1ee;text-align:left;vertical-align:top}a{color:#135b9a}nav{display:flex;flex-wrap:wrap;gap:16px}.note{border-left:4px solid #bc711a;padding:10px 16px;background:#fff7e8}footer{max-width:980px;margin:auto;padding:24px;color:#516078}@media(max-width:560px){header>div,main{padding:16px 12px}article{padding:12px 16px}h2{font-size:21px}}'''
def page(title, source, nav=''):
    body=MarkdownIt('commonmark', {'html': False}).enable('table').render(source)
    body=body.replace('<table>','<div class="table-scroll"><table>').replace('</table>','</table></div>')
    return '<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>'+html.escape(title)+'</title><style>'+STYLE+'</style></head><body><header><div><p>RUST APPLAB · INDEPENDENT PRODUCTS · 0.18.0-dev.6</p><nav>'+nav+'</nav></div></header><main><article>'+body+'</article></main><footer>行为检查 ≠ 独立设计能力认证。保留自己的源码、测试、产物与解释。</footer></body></html>'
def main():
    folder=ROOT/'apps/labquest/independent'
    for name in ('taskpocket','pagemint'):
        (folder/f'{name}.html').write_text(page(name, (folder/f'{name}.md').read_text(),'<a href="CONTRACT.md">需求合同</a><a href="TEST_PLAN.md">自己的测试</a><a href="RELEASE_CHECKLIST.md">交付清单</a>'),encoding='utf-8')
    source=ROOT/'course/INDEPENDENT_PATH.md'
    if source.exists():
        (ROOT/'INDEPENDENT_PATH.html').write_text(page('独立开发：TaskPocket → PageMint', source.read_text(),'<a href="START_HERE.html">新手入口</a><a href="RELIABILITY_PATH.html">上一段：事务与等待</a>'),encoding='utf-8')
    print('Rendered independent product guides')
if __name__ == '__main__': main()
