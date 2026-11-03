#!/usr/bin/env python3
"""One-time feature-branch C1 integration; removed before merge."""
from pathlib import Path
import shutil
R=Path(__file__).resolve().parents[1]
def edit(path,old,new,encoding='utf-8'):
 p=R/path;s=p.read_text(encoding=encoding)
 if new in s:return
 assert old in s,(path,old)
 p.write_text(s.replace(old,new),encoding=encoding)
edit('apps/labquest/src/lib.rs','pub mod advanced_b;','pub mod advanced_b;\npub mod advanced_c1;')
edit('apps/labquest/src/lib.rs','        Some("advanced-b") =>','        Some("advanced-c1") => advanced_c1::run(&args[1..])?,\n        Some("advanced-b") =>')
needle='pub fn run(args: &[OsString]) -> Result<(), String> {'
edit('apps/labquest/src/learning.rs',needle,needle+'\n    if args.len() >= 2 && crate::advanced_c1::is_project(Path::new(&args[1])) {\n        return crate::advanced_c1::run(args);\n    }')
edit('apps/labquest/src/learning.rs','真实异步网络课程仍待实现。','C1 真实回环异步下载已提供独立工程；使用 advanced-c1 start/prepare/check。C2 Axum 仍待实现。')
needle='  cargo run --offline -p labquest -- learn reading [工程]'
edit('apps/labquest/src/guide.rs',needle,'  cargo run --offline -p labquest -- advanced-c1 start <新工程>    C1真实异步下载\n  cargo run --offline -p labquest -- advanced-c1 prepare <工程>   显式准备Rust1.85固定依赖\n  cargo run --offline -p labquest -- advanced-c1 check <工程> [1|2|3|all]  累计验收\n'+needle)
for path in ['scripts/package.py','scripts/structural_check.py']:
 edit(path,'parts[:2] == ("projects", "advanced-b")','parts[:2] in (("projects", "advanced-b"), ("projects", "advanced-c1"))')
edit('scripts/release_check.py',"rel.parts[:2] != ('projects', 'advanced-b')","rel.parts[:2] not in {('projects', 'advanced-b'), ('projects', 'advanced-c1')}")
edit('scripts/verify_release_windows.ps1','^projects/advanced-b/','^projects/(advanced-b|advanced-c1)/','utf-8-sig')
for relative in ['README.md','course/CURRICULUM.md','course/BOOK_GUIDE.md']:
 p=R/relative;s=p.read_text(encoding='utf-8')
 if s.startswith('## 2026-11-03 · C1'):continue
 link='course/ADVANCED_C1.md' if relative=='README.md' else 'ADVANCED_C1.md'
 p.write_text(f'## 2026-11-03 · C1 异步下载\n\n新增独立 Tokio/Reqwest 回环 HTTP 下载工程、三阶段学员骨架、LabQuest 固定验收和离线 vendor 分发。基础 MSRV 继续1.74，C1单独使用1.85.0；C2 Axum 尚未交付。参见 [C1 讲义]({link})；具体跨平台结果以当前提交的 CI 为准。下文带旧版本号的内容保留为历史路线记录。\n\n'+s,encoding='utf-8')
for name in ['Cargo.toml','Cargo.lock','src/model.rs','src/main.rs','tests/contracts.rs']:
 target=R/'projects/advanced-c1/learner'/name;target.parent.mkdir(parents=True,exist_ok=True)
 shutil.copyfile(R/'projects/advanced-c1/reference'/name,target)
print('C1 integration and fixed learner materials synchronized; core TODOs preserved.')
