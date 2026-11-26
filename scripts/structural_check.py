#!/usr/bin/env python3
"""只检查 Rust AppLab 交付结构、文档链接和演示数据契约。

注意：本脚本不能编译 Rust，也不代替 cargo test / clippy / rustfmt。
"""
from __future__ import annotations

import ast
import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
problems: list[str] = []


def require(condition: bool, message: str) -> None:
    if not condition:
        problems.append(message)


# The manifest tracks distributable source, not local student case folders or build artifacts.
ROOT_ALLOWED = {
    "VERSION", "RELEASE_POLICY.md", "KNOWN_ISSUES.md", "VERIFY_RELEASE_WINDOWS.cmd", "VERIFY_BROWSER_WINDOWS.cmd", "Cargo.toml", "Cargo.lock", ".gitignore", ".gitattributes", "LICENSE", "PROJECT_DESIGN.md",
    "README.md", "README_START_HERE.md", "REFERENCE_ANALYSIS.md",
    "START_WORKBENCH_WINDOWS.cmd", "START_WORKBENCH.sh",
    "RELEASE_NOTES.md", "ROADMAP.md", "START_HERE.html", "SYSTEMS_PATH.html", "RELIABILITY_PATH.html", "INDEPENDENT_PATH.html", "ADVANCED_A.html", "ADVANCED_B.html", "START_ADVANCED_B_WINDOWS.cmd", "START_ADVANCED_WINDOWS.cmd", "START_WINDOWS.cmd", "WORKSHOP.html", "CONTINUE_WINDOWS.cmd", "PREPARE_NETWORK_WINDOWS.cmd", "LEARN_NEXT_WINDOWS.cmd", "SUPPORTED_PLATFORMS.md", "VERIFY_BROWSER_WINDOWS.cmd",
}
APPS = ('filequest', 'loglens', 'localhub', 'safesync', 'labquest', 'unitlab', 'quizdeck', 'textscope', 'ledgerlite', 'badgeforge', 'batchmeter', 'termgarden', 'storedesk', 'minikv', 'netfetch', 'servicedesk', 'asyncboard')

for relative in ('course/BOOK_OWNERSHIP_PRACTICE.md', 'course/BOOK_COLLECTIONS_ERRORS.md',
                 'course/BOOK_GENERICS_LIFETIMES.md', 'course/BOOK_CONCEPT_AUDIT.md',
                 'verification/concept-audit-rc3.json'):
    require((ROOT / relative).is_file(), f'核心概念教学材料缺失：{relative}')
for module in ('ownership', 'collections', 'generic_lifetimes'):
    for area in ('src', 'tests'):
        relative = f'projects/book-d6-foundations/foundation-lab/{area}/{module}.rs'
        require((ROOT / relative).is_file(), f'核心概念实验缺失：{relative}')


def allowed(path: Path) -> bool:
    parts = path.relative_to(ROOT).parts
    if len(parts) == 1:
        return parts[0] in ROOT_ALLOWED
    if parts[0] == "apps" and len(parts) >= 3 and parts[1] in APPS:
        return path.suffix in (".rs", ".toml", ".md", ".html", ".css", ".js", ".tsv", ".txt", ".svg", ".ps1")
    if parts[0] == "examples":
        return path.suffix in (".svg", ".txt", ".md", ".html", ".json", ".kv", ".tsv")
    if len(parts) >= 3 and parts[:2] in (("projects", "advanced-b"), ("projects", "advanced-c1"), ("projects", "advanced-c2"), ("projects", "book-d1-cargo"), ("projects", "book-d2-smart-pointers"), ("projects", "book-d3-oop-patterns"), ("projects", "book-d4-advanced-features"), ("projects", "book-d5-webserver"), ("projects", "book-d6-appendices"), ("projects", "book-d6-foundations"), ("projects", "modern-d8-edition"), ("projects", "modern-d9-stable"), ("projects", "modern-d9-stdlib"), ("projects", "modern-d10-async-core"), ("projects", "modern-d10-tokio"), ("projects", "modern-d11-quality"), ("projects", "modern-d11-safety"), ("projects", "modern-e1-tls"), ("projects", "modern-e1-sqlite"), ("projects", "modern-e3-embedded")):
        return path.suffix in (".rs", ".toml", ".md", ".lock", ".html", ".css", ".js") or path.name in ("Cargo.lock", "memory.x")
    if parts[0] in ("course", "docs"):
        return path.suffix == ".md"
    if parts[0] == "scripts":
        return path.suffix in (".py", ".sh", ".ps1", ".cjs")
    if parts[0] == "verification":
        return path.suffix in (".md", ".txt", ".json") or path.name in (
            "LOCALHUB_UI_DOM_DESKTOP.png", "LOCALHUB_UI_DOM_MOBILE.png", "STUDIO_DESKTOP.png", "STUDIO_MOBILE.png", "MINIKV_WORKSHOP_DESKTOP.png", "MINIKV_LESSON_MOBILE.png"
        )
    if len(parts) >= 3 and parts[:2] == (".github", "workflows"):
        return path.suffix in (".yml", ".yaml")
    return False


def files_under_root() -> list[Path]:
    return sorted(
        (path for path in ROOT.rglob("*") if path.is_file() and allowed(path)
         and not any(part in (".git", "target", "__pycache__")
                     for part in path.relative_to(ROOT).parts)),
        key=lambda item: item.relative_to(ROOT).as_posix(),
    )


workspace = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
require(all(f'"apps/{app}"' in workspace for app in APPS), "workspace 未收录全部 package")
require('default-members = ["apps/filequest"]' in workspace, "默认包必须为 FileQuest")
for app in ("filequest", "loglens", "localhub", "safesync"):
    for relative in ("Cargo.toml", "src/main.rs", "src/lib.rs", "src/cli.rs", "tests/acceptance.rs", "tests/cli_smoke.rs", "tests/challenges.rs"):
        require((ROOT / "apps" / app / relative).is_file(), f"缺少 {app}/{relative}")

for relative in ("ADVANCED_B.html", "START_ADVANCED_B_WINDOWS.cmd", "course/ADVANCED_B.md", "apps/labquest/src/advanced_b.rs", "apps/labquest/tests/advanced_b_e2e.rs", "projects/advanced-b/ADVANCED_B.md", "projects/advanced-b/learner/Cargo.toml", "projects/advanced-b/learner/taskcore/src/lib.rs", "projects/advanced-b/learner/taskreport/src/main.rs", "projects/advanced-b/reference/taskcore/src/lib.rs", "projects/advanced-b/reference/taskreport/src/main.rs"):
    require((ROOT / relative).is_file(), f"进阶 B 交付缺失：{relative}")

# Book D1 is additive, never a root workspace member.
for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                 "book-card/Cargo.toml", "book-card/src/lib.rs",
                 "cargo-applab-map/Cargo.toml", "cargo-applab-map/src/main.rs",
                 "cargo-applab-map/tests/cli.rs"):
    require((ROOT / "projects/book-d1-cargo" / relative).is_file(),
            f"Book D1 缺少独立课程工程：{relative}")
for relative in ("course/BOOK_COVERAGE_AUDIT.md", "course/BOOK_D1_CARGO.md",
                 "scripts/book_d1_check.py", "scripts/tests/test_book_coverage.py"):
    require((ROOT / relative).is_file(), f"Book D0/D1 审计或门禁缺失：{relative}")

# Book D2 is a separate smart-pointer workspace, with runnable reference tests.
for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                 "smart-pointer-lab/Cargo.toml", "smart-pointer-lab/src/lib.rs",
                 "smart-pointer-lab/src/main.rs", "smart-pointer-lab/tests/cli.rs"):
    require((ROOT / "projects/book-d2-smart-pointers" / relative).is_file(),
            f"Book D2 缺少独立课程工程：{relative}")
for relative in ("course/BOOK_D2_SMART_POINTERS.md",):
    require((ROOT / relative).is_file(), f"Book D2 讲义缺失：{relative}")

# Book D3 is a separate OOP and pattern-matching workspace.
for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                 "oop-pattern-lab/Cargo.toml", "oop-pattern-lab/src/lib.rs",
                 "oop-pattern-lab/src/main.rs", "oop-pattern-lab/tests/cli.rs"):
    require((ROOT / "projects/book-d3-oop-patterns" / relative).is_file(),
            f"Book D3 缺少独立课程工程：{relative}")
require((ROOT / "course/BOOK_D3_OOP_PATTERNS.md").is_file(),
        "Book D3 讲义缺失")

# Book D4 is a separate advanced-features workspace with a local proc-macro crate.
for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                 "advanced-lab/Cargo.toml", "advanced-lab/src/lib.rs",
                 "advanced-lab/src/main.rs", "advanced-lab/tests/cli.rs",
                 "d4-proc-macros/Cargo.toml", "d4-proc-macros/src/lib.rs"):
    require((ROOT / "projects/book-d4-advanced-features" / relative).is_file(),
            f"Book D4 缺少独立课程工程：{relative}")
require((ROOT / "course/BOOK_D4_ADVANCED_FEATURES.md").is_file(),
        "Book D4 讲义缺失")

# Book D5 is an isolated synchronous TCP/HTTP and worker-pool workspace.
for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                 "webserver-lab/Cargo.toml", "webserver-lab/src/lib.rs",
                 "webserver-lab/src/main.rs", "webserver-lab/tests/cli.rs",
                 "webserver-lab/samples/index.html"):
    require((ROOT / "projects/book-d5-webserver" / relative).is_file(),
            f"Book D5 缺少独立课程工程：{relative}")
require((ROOT / "course/BOOK_D5_WEB_SERVER.md").is_file(),
        "Book D5 讲义缺失")

# Book D6 is an isolated, dependency-free appendix reference workspace.
for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                 "appendix-lab/Cargo.toml", "appendix-lab/src/lib.rs"):
    require((ROOT / "projects/book-d6-appendices" / relative).is_file(),
            f"Book D6 缺少独立课程工程：{relative}")
require((ROOT / "course/BOOK_D6_APPENDICES.md").is_file(),
        "Book D6 附录讲义缺失")
require((ROOT / "course/BOOK_D6_PRACTICAL.md").is_file(),
        "Book D6 D/E/G 实操指南缺失")
for relative in ("course/BOOK_D6_LANGUAGE_BASICS.md", "scripts/book_d6_check.py",
                 "projects/book-d6-foundations/foundation-lab/src/language_basics.rs",
                 "projects/book-d6-foundations/foundation-lab/src/language_basics/catalog.rs",
                 "projects/book-d6-foundations/foundation-lab/tests/compound_types.rs",
                 "projects/book-d6-foundations/foundation-lab/tests/language_basics.rs"):
    require((ROOT / relative).is_file(), f"Book D6 实操补齐缺失：{relative}")

# Book D6-B is an isolated, dependency-free foundations-gap workspace.
for relative in ("Cargo.toml", "Cargo.lock", "README.md",
                 "foundation-lab/Cargo.toml", "foundation-lab/src/lib.rs",
                 "foundation-lab/src/main.rs", "foundation-lab/tests/cli.rs"):
    require((ROOT / "projects/book-d6-foundations" / relative).is_file(),
            f"Book D6-B 缺少独立课程工程：{relative}")
require((ROOT / "course/BOOK_D6_FOUNDATIONS.md").is_file(),
        "Book D6-B 基础知识缺口讲义缺失")

# B4 is a separate locked workspace; both shipped teaching variants are required.
for variant in ('reference', 'learner'):
    for relative in ('Cargo.toml', 'Cargo.lock', 'src/lib.rs', 'src/main.rs', 'tests/b4_api.rs', 'tests/b4_cli.rs'):
        target = ROOT / 'projects' / 'advanced-b' / variant / 'json-export' / relative
        require(target.is_file(), f"B4 交付缺失：{target.relative_to(ROOT)}")
for relative in ('apps/labquest/src/advanced_b_json.rs', 'apps/labquest/tests/advanced_b_json_e2e.rs', 'scripts/tag_checkpoint.py', 'scripts/tests/test_tag_checkpoint.py'):
    require((ROOT / relative).is_file(), f"B4 工具或平台文件缺失：{relative}")

for relative in ("apps/labquest/Cargo.toml", "apps/labquest/src/main.rs", "apps/labquest/src/lib.rs",
                 "apps/labquest/src/case.rs", "apps/labquest/src/proof.rs", "apps/labquest/src/checks.rs",
                 "apps/labquest/src/board.rs", "apps/labquest/src/coach.rs", "apps/labquest/src/capstone.rs",
                 "apps/labquest/tests/case_e2e.rs", "apps/labquest/tests/capstone_e2e.rs",
                 "apps/labquest/tests/fixtures/taskpocket_reference.rs",
                 "apps/labquest/src/transfer.rs", "apps/labquest/tests/transfer_e2e.rs",
                 "apps/labquest/tests/transfer_cli.rs",
                 "apps/labquest/tests/fixtures/pagemint_reference.rs"):
    require((ROOT / relative).is_file(), f"缺少 LabQuest 文件：{relative}")

for asset in ("START_HERE.html", "SYSTEMS_PATH.html", "RELIABILITY_PATH.html", "INDEPENDENT_PATH.html", "ADVANCED_A.html", "ADVANCED_B.html", "START_ADVANCED_B_WINDOWS.cmd", "START_ADVANCED_WINDOWS.cmd", "START_WINDOWS.cmd", "WORKSHOP.html", "CONTINUE_WINDOWS.cmd", "PREPARE_NETWORK_WINDOWS.cmd", "LEARN_NEXT_WINDOWS.cmd", "SUPPORTED_PLATFORMS.md", "VERIFY_BROWSER_WINDOWS.cmd", "scripts/verify_browser_windows.ps1", "scripts/start_windows.ps1",
              "scripts/windows_beginner_smoke.ps1", "course/BEGINNER_WINDOWS_GUIDE.md",
              "apps/labquest/src/beginner.rs", "apps/labquest/tests/beginner_e2e.rs"):
    require((ROOT / asset).is_file(), f"新手入口文件缺失：{asset}")
require("beginner start" in (ROOT / "apps/labquest/src/guide.rs").read_text(encoding="utf-8"), "新手 CLI 未进入帮助")

for app in ("unitlab", "quizdeck", "textscope", "ledgerlite", "badgeforge", "batchmeter", "termgarden", "storedesk", "minikv", "netfetch", "servicedesk", "asyncboard"):
    for rel in (f"apps/{app}/Cargo.toml", f"apps/{app}/src/lib.rs", f"apps/{app}/src/main.rs", f"apps/{app}/tests/milestones.rs", f"course/studio/{app}.md", f"apps/labquest/studio/{app}_starter.rs"):
        require((ROOT / rel).is_file(), f"缺少 Studio 交付文件：{rel}")
    require((ROOT / f"apps/labquest/studio/{app}_starter.rs").read_text(encoding="utf-8").count("TODO(") == 3, f"{app} 必须有3个学员任务")

for chapter in range(32):
    require(len(list((ROOT / "course").glob(f"{chapter:02}_*.md"))) == 1, f"教程 {chapter:02} 必须恰好有一章")

for markdown in files_under_root():
    if markdown.suffix != ".md":
        continue
    text = markdown.read_text(encoding="utf-8")
    for target in re.findall(r"(?<!!)\[[^\]]+\]\(([^)]+)\)", text):
        if target.startswith(("https://", "http://", "mailto:", "#")):
            continue
        local = target.split("#", 1)[0]
        if local and not (markdown.parent / local).exists():
            problems.append(f"文档链接不存在：{markdown.relative_to(ROOT)} -> {target}")

for asset in ("index.html", "app.css", "app.js"):
    require((ROOT / "apps" / "localhub" / "src" / "ui" / asset).is_file(), f"本地网页资源缺失：{asset}")

# 教学挑战源码有意标记 ignored：不执行，避免误把学生任务算为当前功能。
for app, expected in (("filequest", 5), ("loglens", 4), ("localhub", 3), ("safesync", 3)):
    source = (ROOT / "apps" / app / "tests/challenges.rs").read_text(encoding="utf-8")
    actual = len(re.findall(r"#\[ignore\s*=", source))
    require(actual == expected, f"{app} 挑战数量异常：预期 {expected}，实际 {actual}")

# 只静态解释自包含的 concat! 字符串（不是解释/测试 Rust 逻辑）。
demo = (ROOT / "apps/loglens/src/demo.rs").read_text(encoding="utf-8")
examples = {}
for name in ("BASELINE", "CANDIDATE"):
    block = re.search(rf"const {name}: &str = concat!\((.*?)\);", demo, re.S)
    if not block:
        problems.append(f"缺少日志样例 {name}")
        continue
    lines = re.findall(r'^\s*("(?:[^"\\]|\\.)*")\s*,\s*$', block.group(1), re.M)
    value = "".join(ast.literal_eval(line) for line in lines)
    examples[name] = value.splitlines()

if len(examples) == 2:
    before = examples["BASELINE"]
    after = examples["CANDIDATE"]
    level = lambda line: line.split()[1] if len(line.split()) >= 2 and "T" in line.split()[0] else "(invalid)"
    require(len(before) == 8 and sum(level(line) in ("ERROR", "FATAL") for line in before) == 1,
            "基线演示数据应为 8 条记录、1 条错误")
    require(len(after) == 9 and sum(level(line) in ("ERROR", "FATAL") for line in after) == 3
            and after[-1] == "stack trace: line 42", "新版本演示数据应为 8 条记录、3 条错误、1 条未知文本")

manifest = ROOT / "SOURCE_MANIFEST.sha256"
if manifest.is_file():
    lines = manifest.read_text(encoding="utf-8").splitlines()
    indexed = {}
    for line in lines:
        match = re.fullmatch(r"([a-f0-9]{64})  (.+)", line)
        if not match:
            problems.append("SOURCE_MANIFEST.sha256 含无效格式行")
            continue
        indexed[match.group(2)] = match.group(1)
    actual = {}
    for path in files_under_root():
        rel = path.relative_to(ROOT).as_posix()
        actual[rel] = hashlib.sha256(path.read_bytes()).hexdigest()
    for key in sorted(set(actual) | set(indexed)):
        require(actual.get(key) == indexed.get(key), f"文件清单缺失或校验不符：{key}")

if problems:
    print(f"STRUCTURE CHECK FAILED: {len(problems)} issues")
    for item in problems:
        print("  -", item)
    sys.exit(1)
print("STRUCTURE CHECK PASSED: 17 packages, twelve Studio lessons, beginner flow, Book D0-D6 reference labs, 32 legacy chapters, links, challenges, demo contract, manifest")
print("NOTE: this is NOT a Rust compile, formatting, Clippy, or runtime test.")
