#!/usr/bin/env python3
"""Maintainer-only HTML rendering. Shipped pages need no Python/JS/network.
Optional tooling dependency: markdown-it-py (not a learner or Rust dependency).
"""
from pathlib import Path
import html
try:
    from markdown_it import MarkdownIt
except ImportError as exc:
    raise SystemExit('Optional maintainer tool missing: python -m pip install markdown-it-py') from exc
ROOT = Path(__file__).resolve().parents[1]
md = MarkdownIt('commonmark', {'html': False}).enable('table')
CSS = '''*{box-sizing:border-box}body{margin:0;background:#f4f7fb;color:#20334d;font:16px/1.85 system-ui,"Microsoft YaHei",sans-serif}header{background:#14273d;color:#fff}header>div,main{max-width:940px;margin:auto;padding:30px 24px}h1{font-size:clamp(25px,4vw,36px);line-height:1.4;margin:10px 0}h2{font-size:24px;line-height:1.5;margin:34px 0 12px;padding-top:12px;border-top:1px solid #d6e2ed}h3{font-size:19px}header p{color:#d7e7f4}article{background:white;border:1px solid #dce5ef;border-radius:14px;padding:12px 28px 28px}p{overflow-wrap:anywhere}code{font:14px/1.65 Consolas,monospace;background:#eaf1f8;color:#194775;padding:2px 4px;border-radius:4px}pre{background:#14273d;color:#edf3fa;padding:16px;border-radius:9px;overflow:auto}pre code{background:none;color:inherit;padding:0;white-space:pre-wrap;overflow-wrap:anywhere}.note{padding:15px;border:1px solid #89b7a4;background:#ebf7f0;color:#245744;border-radius:10px;margin-bottom:20px}table{width:100%;border-collapse:collapse}th,td{border:1px solid #dce5ef;padding:8px;text-align:left}a{color:#14619e}@media(max-width:600px){header>div,main{padding:20px 14px}article{padding:5px 16px 18px}h2{font-size:21px}}'''
import argparse
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--only', nargs='*', help='selected product IDs; omit for all')
args = parser.parse_args()
version = (ROOT/'VERSION').read_text().strip()
for path in sorted((ROOT/'course/studio').glob('*.md')):
    if args.only and path.stem not in args.only: continue
    text = path.read_text(encoding='utf-8')
    first, rest = text.split('\n',1)
    title = html.escape(first.lstrip('# '))
    page = f'''<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title} · 跟着做</title><style>{CSS}</style></head><body><header><div><small>RUST APPLAB / STUDIO / {version}</small><h1>{title}</h1><p>每次只完成一个步骤。读目标、修改指定函数、运行检查，再使用自己的数据。</p></div></header><main><div class="note">代码编辑位置：学员工程 <strong>src/lib.rs</strong>。先完成 TODO(1)，再做 TODO(2)、TODO(3)。<br>分层提示在 HINTS.md，学习记录在 LEARNING_LOG.md。不要修改参考应用源码。</div><article>{md.render(rest)}</article></main></body></html>'''
    (ROOT/f'apps/labquest/studio/{path.stem}_lesson.html').write_text(page,encoding='utf-8')
print('Rendered all offline lesson pages from authoritative Markdown.')
