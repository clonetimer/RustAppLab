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

INTRO = """# Rust AppLab 0.19.0-dev.2 · 无完整核心答案学员包

本包有十七个顶层学员 Rust 工程：包含 Advanced/TermGarden（所有权）与 Advanced/TaskPocketPro（三 crate 业务库复用与独立 B4 JSON 扩展），保留 FirstCard、十二个 Studio 小作品及两个独立开发项目。

已通过基础课程：先学习 Advanced/TermGarden/START.html，再学习 Advanced/TaskPocketPro/START_HERE.md；不用重复旧关卡。进阶A复用旧TermGarden基础设施，不含进阶核心实现。该工程support目录中的termgarden是依赖，不另计为一个学员作品。

没有编程基础：先打开 FirstCard/跟着做.md。已完成任务卡：先打开 Studio/unitlab/START.html。
在 Studio/unitlab 目录运行 `cargo test --offline --test milestones step1_`，然后只修改 src/lib.rs 的 TODO(1)，保存重试。
UnitLab、QuizDeck、TextScope、LedgerLite、BadgeForge、TermGarden、BatchMeter、MiniKV、NetFetch、ServiceDesk、StoreDesk、AsyncBoard 的 TASK_CARDS.html（可双击打开）与 TASK_CARDS.md 提供逐步预测、三级提示与个人输入实验。每款按1→2→3推进，三步都完成后用自己的数据运行CLI。初始工程故意未实现核心逻辑，测试失败是学习起点。

建议顺序：UnitLab → QuizDeck → TextScope → LedgerLite → BadgeForge；BatchMeter 在理解借用与错误传播之后再学。
完整讲义、分层提示、样例、公开合同测试都在各工程里。基础课程、进阶 A 和 B1–B3 保持纯 path 依赖；B4 的独立 json-export 工作区使用真实 Serde / serde_json，首次需要联网准备依赖缓存。本包不包含 vendor 缓存。NetFetch/ServiceDesk 是同步本机 HTTP 基础版；AsyncBoard 是真实 Future/await 的离线定时器实验，但不是 Tokio 异步网络；StoreDesk 需要系统 SQLite 原生库，先看 course/SQLITE_SETUP.md；其它新手工程不要求 SQLite。

B4 先阅读 course/ADVANCED_B.md 和 course/ADVANCED_B_WINDOWS.md。在完整平台根目录对学员包中 Advanced/TaskPocketPro 的实际路径显式运行 `cargo run --locked --offline -p labquest -- advanced-b prepare-json <工程路径>`；这个子命令联网下载固定依赖，只构建平台参考源码，不运行学员源码。准备成功不是课程通过，之后编辑 JSON 骨架并运行 `cargo run --locked --offline -p labquest -- advanced-b check <工程路径> 4`；显式 `3` 保留原离线合同。缓存位置沿用当前用户 Cargo 配置；换机器、工具链或缓存目录后可能需要重新准备。

TaskPocket / PageMint 新增 START.html、DEVELOPMENT_PLAN.md、TEST_PLAN.md、RELEASE_CHECKLIST.md 和 tests/self_check.rs。初始 cargo test 只有一项入口烟测通过、一项个人测试按设计忽略，不能当成业务完成。完整平台 learn lesson/hint/check 现在也支持它们的三个累计检查点；检查点3仍是原完整合同。

本包不包含已经完成的核心业务参考实现。MiniKV 的 journal.rs、TermGarden 的 storage.rs、StoreDesk 的 sqlite.rs 和 AsyncBoard 的 runtime.rs 是提供给学员的基础设施，不是要求学员先实现的底层作业。CLI 外壳和基础类型已经提供，它不是“从零空文件开发”考核；后续TaskPocket和PageMint才要求更独立地实现。

权威Studio检查和Windows双击向导在完整源码包中：CONTINUE_WINDOWS.cmd。
使用完整平台检查这个包里的工程时，提供该工程的实际路径，例如：
`cargo run --offline -p labquest -- studio check "D:/练习包/Studio/unitlab" 1`
本包本地 `cargo test` 也能独立完成学习；不要删除测试来取得虚假的通过。学习路线与错误练习在 `course/LEARNING_NAVIGATOR.md` 和 `course/RUST_ERROR_LABS.md`。

需要安装 Rust/Cargo。Windows Rust 安装说明：https://rustup.rs/ 。正式声明支持 Windows x64 MSVC 和 Linux x86_64，不声明 macOS；本教学开发版本需要独立实机验证，不继承冻结 RC 的资格。
评测代码只用于自己的可信源码，不是沙箱。每完成一款，请填写 LEARNING_LOG.md，并保存自己的实际产物。
"""

def add(z: zipfile.ZipFile, relative: str, data: bytes):
    info = zipfile.ZipInfo(relative, date_time=(2026, 10, 28, 0, 0, 0))
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = 0o100644 << 16
    z.writestr(info, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)

def course_inputs() -> list[tuple[str, bytes]]:
    """Shared course bytes for the real kit and archive-link regression tests."""
    files = [
        "CURRICULUM.md", "STUDIO_GUIDE.md", "ADVANCED_B.md", "ADVANCED_B_WINDOWS.md", "BOOK_GUIDE.md", "ADVANCED_C1.md", "ADVANCED_C2.md", "SQLITE_SETUP.md", "ADVANCED_A.md",
        "28_independent_product.md", "29_product_release.md",
        "30_second_independent_product.md", "31_transfer_release.md",
        "BEGINNER_WINDOWS_GUIDE.md", "00_first_five_minutes.md", "NETWORK_PREPARATION.md", "LEARNING_NAVIGATOR.md", "RUST_ERROR_LABS.md", "FOUNDATION_PATH.md", "FOUNDATION_NEXT_STEPS.md", "PROGRESS_OVERVIEW.md", "SYSTEMS_PATH.md", "RELIABILITY_PATH.md", "INDEPENDENT_PATH.md",
    ]
    return [(f"course/{name}", (ROOT / "course" / name).read_bytes()) for name in files]


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
        for file in sorted((ROOT / "course/studio").glob("*.md")):
            inputs.append(("course/studio/" + file.name, file.read_bytes()))
        with zipfile.ZipFile(OUT, "w") as z:
            for name, data in sorted(inputs):
                add(z, name, data)
    with zipfile.ZipFile(OUT) as z:
        names = z.namelist()
        assert z.testzip() is None
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
