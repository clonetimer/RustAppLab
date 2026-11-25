#!/usr/bin/env python3
"""用正式 LabQuest 生成基础与进阶 A/B 的无答案学员包。"""
from __future__ import annotations

import hashlib
import html
import os
import platform
import subprocess
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = (ROOT / "VERSION").read_text(encoding="utf-8").strip()
EXE = Path(os.environ.get("CARGO_TARGET_DIR", str(ROOT / "target"))) / "release" / ("labquest.exe" if platform.system() == "Windows" else "labquest")
OUT = ROOT.parent / f"RustAppLab-{VERSION}-learner-kit.zip"

INTRO = f"""# Rust AppLab {VERSION} · 学员练习包

## 解压后先做哪一个？

| 你现在的位置 | 打开哪里 | 这一节结束应得到什么 |
|---|---|---|
| 还没写过 Rust | [FirstCard/README.md](FirstCard/README.md)，再按跟着做.md 操作 | 随不同姓名和任务变化的任务卡，能解释变量和输出 |
| FirstCard 已完成 | [Studio/unitlab/README.md](Studio/unitlab/README.md)，再打开 START.html | 会计算并拒绝非法输入的单位换算器 |
| 正在做某个 Studio 作品 | 该工程 README.md 和 TASK_CARDS.html | 找到当前要改的函数、检查命令和自己的实验 |
| 已完成基础和独立作品 | Advanced/TermGarden/START.html → Advanced/TaskPocketPro/START_HERE.md | 共享状态、接口设计和业务库复用 |

**这些工程已经生成，不要再次 start，也不要先把十七个工程全部编译一遍。** 每个工程的核心 TODO 都留给你完成，初始测试失败是学习起点。

## FirstCard 之后：先做 UnitLab 的第一个函数

1. 打开 Studio/unitlab/START.html，读第 1 步。
2. 用编辑器打开 Studio/unitlab/src/lib.rs，只修改 celsius_to_fahrenheit 的 TODO(1)，保存。
3. 在 **Studio/unitlab 目录**（不是本包顶层）运行：

```powershell
cargo test --offline --test milestones step1_
```

通过后再做第 2、3 步。三步全部完成后，仍在该目录执行：

```powershell
cargo test --offline
cargo run --offline -- convert -10 c f
cargo run --offline -- convert 25 c m
```

第一条换算应输出 `-10.00 c = 14.00 f`；第二条应拒绝摄氏度转米。换一个自己的数值先预测再运行，在 LEARNING_LOG.md 解释函数的输入、返回值和错误。接着打开 **Studio/quizdeck/README.md**。自动检查不替你判断是否理解。

本地 cargo test 使用包内公开测试；完整平台的固定检查在 source.zip 中。需要平台任务卡、提示或固定验收时，在**完整 RustAppLab 平台根目录**传入这里的实际工程路径：

```powershell
cargo run --locked --offline -p labquest -- learn lesson "D:/练习包/Studio/unitlab" 1
cargo run --locked --offline -p labquest -- learn check "D:/练习包/Studio/unitlab" 1
```

把示例路径换成自己的位置；只有本学员包时不能在它的顶层运行 `-p labquest`。更完整的解释见 [FirstCard 后的学习方法](course/AFTER_FIRSTCARD.md)。

## 顺序与每一段的用途

FirstCard → UnitLab（函数与换算）→ QuizDeck（题库与集合）→ TextScope（文本与借用）→ LedgerLite（持久记录）→ BadgeForge（trait 与输出）→ TermGarden（状态与交互）→ BatchMeter（线程）→ MiniKV（日志与索引）→ NetFetch / ServiceDesk（同步本机 HTTP）→ StoreDesk（SQLite 事务）→ AsyncBoard（Future 与等待）→ TaskPocket / PageMint（独立开发）→ 进阶 A → 进阶 B。

每款 Studio 都有 README.md、TASK_CARDS.html / TASK_CARDS.md、讲义、逐级提示、数据和三步公开测试。只改当前函数；之后留下自己的输入、结果和解释，再读 README 的下一步。

TaskPocket / PageMint 的 CONTRACT.md、DEVELOPMENT_PLAN.md、TEST_PLAN.md、RELEASE_CHECKLIST.md 帮助你独立拆分需求。初始测试中的入口烟测通过不代表业务完成；完整平台 learn lesson/hint/check 支持三个累计检查点。

## 依赖和材料各是什么？

- 需要自己安装 Rust/Cargo 和系统构建工具；Windows x64 MSVC / Linux x86_64 是当前支持范围。安装入口：https://rustup.rs/ 。
- 基础、进阶 A 和 B1–B3 使用本地 path 依赖。StoreDesk 另需系统 SQLite，按 [SQLite 说明](course/SQLITE_SETUP.md) 准备；FirstCard/UnitLab 不需要它。
- B4 的 json-export 使用真实 Serde/serde_json，首次须联网准备固定依赖并写入当前用户的 Cargo 缓存；本包不带 vendor。按 [进阶 B](course/ADVANCED_B.md) 先完成 B1–B3，再使用完整平台的 prepare-json 和第 4 步检查。换机器或缓存位置后可能需要重新准备；准备依赖不等于完成课程。
- 可选现代 D8：在 [Edition 2024 讲义](course/BOOK_D8_EDITION_2024.md) 按步骤复制独立学员工程，需 Rust 1.100.0；不改变原十七个顶层工程与成绩。
- 可选现代 D9-A：完成 D8 后进入 [稳定特性讲义](course/BOOK_D9_STABLE_FEATURES.md)；九项独立实验，不替换十七个经典学员工程。
- C1/C2 请另取对应 learner-offline.zip 并按讲义准备 Rust 1.85.0；本包没有这两门课的学员工程和 vendor。
- projects/book-d* 是可选的参考实验，包含 D1 Cargo、D2 智能指针、D3 面向对象与模式、D4 高级特性、D5 Web Server、D6-A 附录参考实验、D6-B 基础缺口补齐实验。D6-A/D6-B 都是独立参考实验，和其它 Book 实验一样不自动计入个人成绩。course/ 中的概念补课帮助理解当前作品，不是 FirstCard 后必须先做完的新任务。
- MiniKV 的 journal.rs、TermGarden 的 storage.rs、StoreDesk 的 sqlite.rs、AsyncBoard 的 runtime.rs 是已提供的基础设施，任务卡会说明你亲手实现哪一层。

本包有十七个顶层学员工程（FirstCard、十二款 Studio、两款独立作品、进阶 A/B）；参考实验和依赖不额外算作学员作品。测试通过不能代填个人学习记录。每课完成后保留自己的源码与产物，下次直接打开原工程继续。
"""

def add(z: zipfile.ZipFile, relative: str, data: bytes):
    info = zipfile.ZipInfo(relative, date_time=(2026, 10, 28, 0, 0, 0))
    info.create_system = 3
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = 0o100644 << 16
    z.writestr(info, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)

def course_inputs() -> list[tuple[str, bytes]]:
    """Shared course bytes for the real kit and archive-link regression tests."""
    files = [
        "CURRICULUM.md", "AFTER_FIRSTCARD.md", "BOOK_COVERAGE_AUDIT.md", "BOOK_D1_CARGO.md", "BOOK_D2_SMART_POINTERS.md", "BOOK_D3_OOP_PATTERNS.md", "BOOK_D4_ADVANCED_FEATURES.md", "BOOK_D5_WEB_SERVER.md", "BOOK_D6_APPENDICES.md", "BOOK_D6_FOUNDATIONS.md", "BOOK_D6_PRACTICAL.md", "BOOK_D6_LANGUAGE_BASICS.md", "STUDIO_GUIDE.md", "ADVANCED_B.md", "ADVANCED_B_WINDOWS.md", "BOOK_GUIDE.md", "ADVANCED_C1.md", "ADVANCED_C2.md", "USER_TRIAL.md", "SQLITE_SETUP.md", "ADVANCED_A.md",
        "28_independent_product.md", "29_product_release.md",
        "30_second_independent_product.md", "31_transfer_release.md",
        "LEARNER_EVIDENCE_PROTOCOL.md", "LEARNING_LOG_TEMPLATE.md", "BOOK_31_GAPS_REPAIR.md",
        "BOOK_MODERN_COVERAGE_D7.md", "MODERN_RUST_ROADMAP.md", "BOOK_D8_EDITION_2024.md",
        "BOOK_D9_STABLE_FEATURES.md", "MODERN_STABLE_FEATURE_INDEX.md",
        "BEGINNER_WINDOWS_GUIDE.md", "00_first_five_minutes.md", "NETWORK_PREPARATION.md", "LEARNING_NAVIGATOR.md", "RUST_ERROR_LABS.md", "FOUNDATION_PATH.md", "FOUNDATION_NEXT_STEPS.md", "PROGRESS_OVERVIEW.md", "SYSTEMS_PATH.md", "RELIABILITY_PATH.md", "INDEPENDENT_PATH.md",
    ]
    result = [(f"course/{name}", (ROOT / "course" / name).read_bytes()) for name in files]
    for path in sorted((ROOT / "course" / "studio").glob("*.md")):
        result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    for name in ("BOOK_OWNERSHIP_PRACTICE.md", "BOOK_COLLECTIONS_ERRORS.md",
                 "BOOK_GENERICS_LIFETIMES.md", "BOOK_CONCEPT_AUDIT.md"):
        result.append((f"course/{name}", (ROOT / "course" / name).read_bytes()))
    result.append(("verification/concept-audit-rc3.json",
                   (ROOT / "verification/concept-audit-rc3.json").read_bytes()))
    result.append(("verification/concept-repair-20261124.json",
                   (ROOT / "verification/concept-repair-20261124.json").read_bytes()))
    result.append(("scripts/book_d6_check.py", (ROOT / "scripts/book_d6_check.py").read_bytes()))
    # D1 is a runnable reference lab, not an 18th learner exercise. Its code
    # must accompany the docs so offline links do not dangle in learner-kit.
    lab = ROOT / "projects" / "book-d1-cargo"
    for path in sorted(lab.rglob("*")):
        if (path.is_file() and not any(part in {"target", ".git", "__pycache__"}
                                       for part in path.relative_to(lab).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    # D2 is a runnable reference lab, not an additional learner project.
    lab = ROOT / "projects" / "book-d2-smart-pointers"
    for path in sorted(lab.rglob("*")):
        if (path.is_file() and not any(part in {"target", ".git", "__pycache__"}
                                       for part in path.relative_to(lab).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    # D3 is a reference workspace, not an additional learner project.
    lab = ROOT / "projects" / "book-d3-oop-patterns"
    for path in sorted(lab.rglob("*")):
        if (path.is_file() and not any(part in {"target", ".git", "__pycache__"}
                                       for part in path.relative_to(lab).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    # D4 is a reference workspace with a local procedural-macro crate.
    lab = ROOT / "projects" / "book-d4-advanced-features"
    for path in sorted(lab.rglob("*")):
        if (path.is_file() and not any(part in {"target", ".git", "__pycache__"}
                                       for part in path.relative_to(lab).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    # D5 is a runnable reference lab, not an additional learner project.
    # Include the HTML page served by the real TCP server.
    lab = ROOT / "projects" / "book-d5-webserver"
    for path in sorted(lab.rglob("*")):
        if (path.is_file() and not any(part in {"target", ".git", "__pycache__"}
                                       for part in path.relative_to(lab).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs", ".html"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    # D6 is a dependency-free reference lab for appendices A-C.
    lab = ROOT / "projects" / "book-d6-appendices"
    for path in sorted(lab.rglob("*")):
        if (path.is_file() and not any(part in {"target", ".git", "__pycache__"}
                                       for part in path.relative_to(lab).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    # D6-B is a dependency-free reference lab for the remaining basic chapter gaps.
    lab = ROOT / "projects" / "book-d6-foundations"
    for path in sorted(lab.rglob("*")):
        if (path.is_file() and not any(part in {"target", ".git", "__pycache__"}
                                       for part in path.relative_to(lab).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    # D8 is an optional modern exercise. Ship the TODO project, not the solution.
    modern = ROOT / "projects" / "modern-d8-edition"
    result.append((f"projects/modern-d8-edition/README.md", (modern / "README.md").read_bytes()))
    for path in sorted((modern / "learner").rglob("*")):
        if (path.is_file() and not {"target", ".git", "__pycache__"}.intersection(path.relative_to(modern).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    # D9-A ships only the optional learner tree, never diagnostic solutions.
    modern = ROOT / "projects" / "modern-d9-stable"
    result.append(("projects/modern-d9-stable/README.md", (modern / "README.md").read_bytes()))
    for path in sorted((modern / "learner").rglob("*")):
        if (path.is_file() and not {"target", ".git", "__pycache__"}.intersection(path.relative_to(modern).parts)
                and path.suffix in {".toml", ".lock", ".md", ".rs"}):
            result.append((path.relative_to(ROOT).as_posix(), path.read_bytes()))
    return result


def main() -> None:
    if not EXE.is_file():
        raise SystemExit(f"请先 cargo build -p labquest --release，再打包无答案学生项目：{EXE} 不存在")
    with tempfile.TemporaryDirectory(prefix="rustapplab-learner-") as dirname:
        tmp = Path(dirname)
        for name, dirname in (("taskpocket", "TaskPocket"), ("pagemint", "PageMint"), ("beginner", "FirstCard")):
            subprocess.run([str(EXE), "beginner", "start", str(tmp / dirname)] if name == "beginner" else [str(EXE), "app", "init", name, str(tmp / dirname)], check=True,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        (tmp / "Studio").mkdir()
        for product in ("unitlab", "quizdeck", "textscope", "ledgerlite", "badgeforge", "batchmeter", "termgarden", "storedesk", "minikv", "netfetch", "servicedesk", "asyncboard"):
            subprocess.run([str(EXE), "studio", "start", product, str(tmp / "Studio" / product)], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        (tmp / "Advanced").mkdir()
        subprocess.run([str(EXE), "advanced", "start", str(tmp / "Advanced" / "TermGarden")], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        subprocess.run([str(EXE), "advanced-b", "start", str(tmp / "Advanced" / "TaskPocketPro")], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        inputs: list[tuple[str, bytes]] = [("ADVANCED_A.html", (ROOT/"ADVANCED_A.html").read_bytes()), ("ADVANCED_B.html", (ROOT/"ADVANCED_B.html").read_bytes()),("INDEPENDENT_PATH.html", (ROOT/"INDEPENDENT_PATH.html").read_bytes()), ("RELIABILITY_PATH.html", (ROOT/"RELIABILITY_PATH.html").read_bytes()), ("SYSTEMS_PATH.html", (ROOT/"SYSTEMS_PATH.html").read_bytes()), ("START_HERE.md", INTRO.encode("utf-8")), ("SUPPORTED_PLATFORMS.md", (ROOT / "SUPPORTED_PLATFORMS.md").read_bytes())]
        for app in ("TaskPocket", "PageMint", "FirstCard", "Studio", "Advanced"):
            for file in (tmp / app).rglob("*"):
                if file.is_file():
                    rel = (Path(app) / file.relative_to(tmp / app)).as_posix()
                    inputs.append((rel, file.read_bytes()))
        # 学员先看到目标页面，再亲手编写生成器；仅 HTML 预览，无完整 Rust 答案。
        note_files = sorted((tmp / "PageMint" / "samples" / "notes").glob("*.note"))
        articles = []
        for path in note_files:
            lines = path.read_text(encoding="utf-8").splitlines()
            title = html.escape(lines[0].strip(), quote=True)
            paras = "".join(f"<p>{html.escape(line.strip(), quote=True)}</p>" for line in lines[1:] if line.strip())
            articles.append(f'<article data-file="{html.escape(path.name, quote=True)}"><h2>{title}</h2>{paras}</article>')
        preview_head = """<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><title>PageMint</title>
<style>body{margin:0;background:#f3f4f8;color:#253148;font:16px/1.7 system-ui,sans-serif}header,main{max-width:760px;margin:auto;padding:24px}header{padding-top:52px}h1{font-size:34px;margin:0}header p{color:#697386}article{background:white;border-radius:14px;padding:18px 24px;margin-bottom:20px;box-shadow:0 5px 20px #18283613}h2{margin-top:4px;font-size:22px}p{overflow-wrap:anywhere}@media(max-width:700px){header,main{padding:14px}header{padding-top:30px}}</style></head>
<body><header><h1>PageMint · 我的离线笔记</h1><p>完成作品后的视觉预览，不包含任何 Rust 解答。请用自己写的代码生成属于你的网页。</p></header><main>
"""
        preview = preview_head + "\n".join(articles) + "\n</main></body></html>\n"
        inputs.append(("PageMint/GOAL_PREVIEW.html", preview.encode("utf-8")))
        inputs.extend(course_inputs())
        for file in sorted((ROOT / "examples/foundation").glob("*")):
            if file.is_file(): inputs.append(("examples/foundation/" + file.name, file.read_bytes()))
        for file in sorted((ROOT / "examples/systems").glob("*")):
            if file.is_file(): inputs.append(("examples/systems/" + file.name, file.read_bytes()))
        for file in sorted((ROOT / "examples/reliability").glob("*")):
            if file.is_file(): inputs.append(("examples/reliability/" + file.name, file.read_bytes()))
        with zipfile.ZipFile(OUT, "w") as z:
            for name, data in sorted(inputs):
                add(z, name, data)
    with zipfile.ZipFile(OUT) as z:
        names = z.namelist()
        assert z.testzip() is None
        assert len(names) == len(set(names)), "学员包不能包含同名的重复文件"
        assert not any("reference" in item.lower() or "fixture" in item.lower() for item in names)
        for product in ("unitlab", "quizdeck", "textscope", "ledgerlite", "badgeforge", "batchmeter", "termgarden", "storedesk", "minikv", "netfetch", "servicedesk", "asyncboard"):
            assert z.read(f"Studio/{product}/src/lib.rs").decode("utf-8").count("TODO(") == 3
            assert f"Studio/{product}/tests/milestones.rs" in names
        assert "Advanced/TaskPocketPro/taskcore/src/lib.rs" in names
        assert "B1 TODO:" in z.read("Advanced/TaskPocketPro/taskcore/src/lib.rs").decode("utf-8")
        assert "Advanced/TermGarden/DESIGN_REVIEW.md" in names
        assert "todo!" in z.read("Advanced/TermGarden/src/lib.rs").decode("utf-8")
        assert not any("workbench.rs" in n for n in names if n.startswith("Advanced/TermGarden/support"))
        assert "TaskPocket/src/main.rs" in names and "PageMint/src/main.rs" in names
        assert "FirstCard/src/main.rs" in names and "FirstCard/跟着做.md" in names
        assert "PageMint/samples/notes/01-first.note" in names
        assert all("TODO" in z.read(f"{product}/src/main.rs").decode("utf-8") for product in ("TaskPocket", "PageMint", "FirstCard"))
    sha = hashlib.sha256(OUT.read_bytes()).hexdigest()
    (OUT.parent / f"{OUT.name}.sha256").write_text(f"{sha}  {OUT.name}\n", encoding="utf-8")
    print(f"CREATED: {OUT} ({OUT.stat().st_size} bytes, {len(names)} files)")
    print("SHA-256:", sha)


if __name__ == '__main__':
    main()
