#!/usr/bin/env python3
"""C2 feature-branch integration; copies locked teaching infrastructure and C2 grader.

Run once after reference contracts are green. It never copies reference lib.rs
into learner lib.rs. Integration remains reviewable as a single git commit.
"""
from pathlib import Path
import shutil
import subprocess

R = Path(__file__).resolve().parents[1]
REF = R / "projects/advanced-c2/reference"
LEARNER = R / "projects/advanced-c2/learner"

def edit(relative: str, old: str, new: str, encoding: str = "utf-8") -> None:
    path=R/relative
    text=path.read_text(encoding=encoding)
    if new in text: return
    if text.count(old)!=1:
        raise AssertionError(f"{relative}: expected exactly one anchor {old!r}, found {text.count(old)}")
    path.write_bytes(text.replace(old,new).encode(encoding))

def make_grader() -> None:
    source=(R/"apps/labquest/src/advanced_c1.rs").read_text(encoding="utf-8")
    source=source.replace("C1","C2").replace("c1","c2")
    start=source.index("const REFERENCE:")
    end=source.index("const STARTER:",start)
    source=source[:start]+'''const REFERENCE: &str = include_str!("../../../projects/advanced-c2/reference/src/lib.rs");
const HANDLERS: &str = include_str!("../../../projects/advanced-c2/reference/src/handlers.rs");
const HTML: &str = include_str!("../../../projects/advanced-c2/reference/assets/index.html");
const JS: &str = include_str!("../../../projects/advanced-c2/reference/assets/client.js");
const CSS: &str = include_str!("../../../projects/advanced-c2/reference/assets/app.css");
const DOMAIN_MANIFEST: &str = include_str!("../../../apps/servicedesk/Cargo.toml");
const DOMAIN: &str = include_str!("../../../apps/servicedesk/src/lib.rs");
'''+source[end:]
    start=source.index("    if reference {",source.index("fn install("))
    end=source.index("    Ok(())\n}\npub fn init",start)
    source=source[:start]+'''    write(&root.join("Cargo.toml"), &MANIFEST.replace("../../../apps/servicedesk", "support/servicedesk"))?;
    for (path, content) in [
        ("src/handlers.rs", HANDLERS), ("assets/index.html", HTML),
        ("assets/client.js", JS), ("assets/app.css", CSS),
        ("support/servicedesk/Cargo.toml", DOMAIN_MANIFEST),
        ("support/servicedesk/src/lib.rs", DOMAIN)
    ] { write(&root.join(path), content)?; }
    write(&root.join("src/lib.rs"), if reference { REFERENCE } else { STARTER })?;
'''+source[end:]
    anchor='    walk(&root.join("src"), &root.join("src"), &mut files, 0)?;'
    if anchor not in source: raise AssertionError("missing walk")
    source=source.replace(anchor,anchor+'''
    // Fixed infrastructure is never used as submitted learner code.
    for name in ["main.rs", "model.rs", "handlers.rs"] { files.remove(name); }
''',1)
    source=source.replace(' || !files.contains_key("main.rs")','')
    old='    fs::remove_dir_all(temp.0.join("src")).map_err(|e| e.to_string())?;'
    assert old in source
    source=source.replace(old,'    fs::remove_file(temp.0.join("src/lib.rs")).map_err(|e| e.to_string())?;')
    old='for text in ["C2 Report v1\\n", MANIFEST, LOCK, TESTS]'
    new='for text in ["C2 Report v1\\n", MANIFEST, LOCK, TESTS, MODEL, MAIN, HANDLERS, HTML, JS, CSS, DOMAIN_MANIFEST, DOMAIN]'
    assert old in source
    source=source.replace(old,new)
    (R/"apps/labquest/src/advanced_c2.rs").write_bytes(source.encode("utf-8"))

def sync_learner() -> None:
    for name in ("Cargo.toml","Cargo.lock","src/model.rs","src/main.rs","src/handlers.rs","tests/contracts.rs","assets/index.html","assets/client.js","assets/app.css"):
        path=LEARNER/name
        path.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(REF/name,path)
    student=(LEARNER/"src/lib.rs").read_text(encoding="utf-8")
    assert student.count("TODO(")==3 and "runtime.spawn_blocking(" not in student

def main():
    make_grader()
    sync_learner()
    edit("apps/labquest/src/lib.rs","pub mod advanced_c1;","pub mod advanced_c1;\npub mod advanced_c2;")
    edit("apps/labquest/src/lib.rs",'        Some("advanced-c1") => advanced_c1::run(&args[1..])?,',
         '        Some("advanced-c2") => advanced_c2::run(&args[1..])?,\n        Some("advanced-c1") => advanced_c1::run(&args[1..])?,')
    anchor='    if args.len() >= 2 && crate::advanced_c1::is_project(Path::new(&args[1])) {'
    edit("apps/labquest/src/learning.rs",anchor,
         '    if args.len() >= 2 && crate::advanced_c2::is_project(Path::new(&args[1])) {\n        return crate::advanced_c2::run(args);\n    }\n'+anchor)
    edit("apps/labquest/src/learning.rs","C2 Axum 仍待实现。",
         "C2 Axum 真实异步工单服务已提供独立学员工程；使用 advanced-c2 start/prepare/check。")
    edit("apps/labquest/src/guide.rs",
         "  cargo run --offline -p labquest -- advanced-c1 start",
         "  cargo run --offline -p labquest -- advanced-c2 start <新工程>    C2 Axum工单服务\n"
         "  cargo run --offline -p labquest -- advanced-c2 prepare <工程>   显式准备固定依赖\n"
         "  cargo run --offline -p labquest -- advanced-c2 check <工程> [1|2|3|all]  累计验收\n"
         "  cargo run --offline -p labquest -- advanced-c1 start")
    for name in ("scripts/package.py","scripts/structural_check.py"):
        edit(name,'("projects", "advanced-c1"))','("projects", "advanced-c1"), ("projects", "advanced-c2"))')
        edit(name,'path.suffix in (".rs", ".toml", ".md", ".lock")','path.suffix in (".rs", ".toml", ".md", ".lock", ".html", ".css", ".js")')
    edit("scripts/release_check.py","('projects', 'advanced-c1')}",
         "('projects', 'advanced-c1'), ('projects', 'advanced-c2')}")
    edit("scripts/verify_release_windows.ps1",
         "advanced-b|advanced-c1","advanced-b|advanced-c1|advanced-c2",encoding="utf-8-sig")
    edit("scripts/package_learner_kit.py",'"ADVANCED_C1.md",','"ADVANCED_C1.md", "ADVANCED_C2.md",')
    for relative in ("README.md","ROADMAP.md","course/CURRICULUM.md","course/BOOK_GUIDE.md"):
        path=R/relative
        source=path.read_text(encoding="utf-8")
        if not source.startswith("## C2 Axum"):
            link=("course/ADVANCED_C2.md" if relative in ("README.md","ROADMAP.md") else "ADVANCED_C2.md")
            path.write_bytes(("## C2 Axum · 新增异步工单课程\n\n独立 Axum/Tokio 工作区、固定 ServiceDesk 业务日志、三阶段学员接口与回归测试。"
                              f"见 [C2 指南]({link})；当前提交的实际支持范围以 CI 结果为准。"
                              "原同步版本、C1 和基础 Rust1.74 保留。\n\n"+source).encode("utf-8"))
    subprocess.run(["cargo","fmt","--manifest-path",str(REF/"Cargo.toml"),"--all"],check=True)
    subprocess.run(["cargo","fmt","--manifest-path",str(LEARNER/"Cargo.toml"),"--all"],check=True)
    subprocess.run(["cargo","fmt","--all"],cwd=R,check=True)
    subprocess.run(["python","scripts/package.py"],cwd=R,check=True)
    print("C2 learner, LabQuest routes, source manifest and core files synchronized.")

if __name__=="__main__":
    main()
