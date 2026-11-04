#!/usr/bin/env python3
"""One-time release assembly; this file is removed before creating the feature tree."""
from pathlib import Path
R=Path.cwd()
def edit(path,old,new):
    p=R/path;s=p.read_text(encoding='utf-8');assert old in s,(path,old);p.write_bytes(s.replace(old,new).encode('utf-8'))
(R/'VERSION').write_bytes(b'0.19.0-rc.1\n')
for name in ('apps/labquest/Cargo.toml','Cargo.lock'):
    edit(name,'0.19.0-dev.2','0.19.0-rc.1')
edit('apps/labquest/src/advanced_c1.rs','C1 已通过 3/3；用自己的回环数据完成练习。C2 Axum 尚未交付。','C1 已通过 3/3；用自己的回环数据完成练习，再学习 C2 Axum 工单服务。')
edit('apps/labquest/src/ecosystem.rs','Tokio/Reqwest/Axum 异步框架版仍未完成。没有真实 Cargo.lock/vendor 就不声称框架依赖已准备好。','C1/C2 异步课程已交付，请使用 advanced-c1/advanced-c2 prepare 或对应离线包。本入口仅保留依赖探针，不是课程准入。')
edit('course/README.md','0.19.0-dev.2','0.19.0-rc.1')
p=R/'course/BOOK_GUIDE.md';s=p.read_text(encoding='utf-8');start=s.index('## 2026-11-03 · C1 异步下载');end=s.index('\n\n',s.index('新增独立 Tokio/Reqwest',start))+2;s=s[:start]+s[end:];s=s.replace('0.19.0-dev.2','0.19.0-rc.1').replace('真实异步网络课程 C 继续按 [课程路线](CURRICULUM.md) 推进。','已交付的真实异步网络课程见 [C1](ADVANCED_C1.md) 与 [C2](ADVANCED_C2.md)，按 [课程路线](CURRICULUM.md) 学习。');p.write_bytes(s.encode('utf-8'))
edit('START_HERE.html','0.18.0-dev.5 · 已有练习不用重建','0.19.0-rc.1 候选版 · 已有练习不用重建')
edit('START_HERE.html','<strong>0.11.1 修复：</strong>已修正 Windows 第 1 步创建目录时误报“上级文件夹不存在”的问题；不用重装 Rust。<br>','<strong>下载与环境：</strong>完整学习使用 source.zip；直接体验使用对应平台 runtime.zip。参考运行包无需Cargo，学员检查仍需Rust。<br>')
edit('START_HERE.html','如果你刚开始学习编程，这一页就是唯一入口。','如果你刚开始学习编程，先从这一页开始。')
edit('START_HERE.html','<div class="track">','<p><a href="README_START_HERE.md">下载包选择、Linux命令与常见问题</a> · <a href="course/USER_TRIAL.md">记录实际试用</a></p><div class="track">')
edit('START_HERE.html','Rust AppLab 0.18.0-dev.5 · 纯离线学习入口。','Rust AppLab 0.19.0-rc.1 · 候选版离线阅读入口。')
edit('START_HERE.html','此开发分支不继承旧 RC 测试资格；新增页面和 Windows 流程需独立验证。','当前版本按同一源码重新验证；Windows本机体验和真人学习仍需实际记录。')
edit('WORKSHOP.html','Tokio非阻塞网络、桌面界面、WebAssembly仍待扩展','C1/C2非阻塞网络已交付；桌面界面与WebAssembly保留为选修方向')
edit('WORKSHOP.html','0.18.0-dev.5 学习导航仍待 Windows 原生复验','0.19.0-rc.1 按当前提交重新执行Windows/Linux准入，真人试用另行记录')
edit('scripts/package_learner_kit.py','INTRO = """# Rust AppLab 0.19.0-dev.2','INTRO = f"""# Rust AppLab {VERSION}')
edit('scripts/package_learner_kit.py','本包不包含已经完成的核心业务参考实现。','C1/C2请另取对应learner-offline.zip；本包不内置这两门课程或其vendor依赖。\n\n本包不包含已经完成的核心业务参考实现。')
edit('scripts/package_learner_kit.py','本教学开发版本需要独立实机验证','本候选版本需要当前源码工程验证和真实试用')
edit('scripts/package_learner_kit.py','    info.compress_type = zipfile.ZIP_DEFLATED','    info.create_system = 3\n    info.compress_type = zipfile.ZIP_DEFLATED')
edit('scripts/package_learner_kit.py','"ADVANCED_C2.md", "SQLITE_SETUP.md"','"ADVANCED_C2.md", "USER_TRIAL.md", "SQLITE_SETUP.md"')
edit('scripts/tests/test_c2_handoff.py','needs: [native, browser, msrv, c1, c2]','needs: [native, browser, msrv, c1, c2, delivery, candidate]')
p=R/'RELEASE_NOTES.md';p.write_bytes(('''# 0.19.0-rc.1 · C2范围候选发布收尾（2026-11-04）

功能冻结在C2，不新增课程。统一首页、入门、已知边界、路线和发布规则；明确B4/C1/C2已交付、参考包/学员包/源码包及环境差异。加入候选交付检查和对应平台运行包，所有当前源码须经完整CI，发布为prerelease而非Stable。真人试用记录保留为空白模板。

修正C1完成提示和旧依赖探针中的过时状态，不改变业务算法、学生合同或已有数据。历史材料自下方开始按当时版本保留，不代表当前范围。

## 历史记录

'''+p.read_text(encoding='utf-8')).encode('utf-8'))
for name in ('.github/workflows/closeout-assemble.yml','closeout_finish.py'):
    (R/name).unlink()
