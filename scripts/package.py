#!/usr/bin/env python3
"""可复现源码包（不包含编译缓存、用户演示目录或个人文件）。"""
from __future__ import annotations

import hashlib
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = (ROOT / "VERSION").read_text(encoding="utf-8").strip()
OUTPUT = ROOT.parent / f"RustAppLab-{VERSION}-source.zip"
MANIFEST = ROOT / "SOURCE_MANIFEST.sha256"
SKIP = {".git", "target", "__pycache__", "filequest-playground", "loglens-playground", "localhub-playground", "safesync-playground", "labquest-playground", "labquest-second-case"}
ROOT_ALLOWED = {
    "VERSION", "RELEASE_POLICY.md", "KNOWN_ISSUES.md", "VERIFY_RELEASE_WINDOWS.cmd", "VERIFY_BROWSER_WINDOWS.cmd", "Cargo.toml", "Cargo.lock", ".gitignore", ".gitattributes", "LICENSE", "PROJECT_DESIGN.md", "README.md",
    "README_START_HERE.md", "REFERENCE_ANALYSIS.md", "RELEASE_NOTES.md", "ROADMAP.md",
    "START_HERE.html", "SYSTEMS_PATH.html", "RELIABILITY_PATH.html", "INDEPENDENT_PATH.html", "ADVANCED_A.html", "ADVANCED_B.html", "START_ADVANCED_B_WINDOWS.cmd", "START_ADVANCED_WINDOWS.cmd", "START_WINDOWS.cmd", "WORKSHOP.html", "CONTINUE_WINDOWS.cmd", "PREPARE_NETWORK_WINDOWS.cmd", "LEARN_NEXT_WINDOWS.cmd", "SUPPORTED_PLATFORMS.md",
}


def allowed(path: Path) -> bool:
    parts = path.relative_to(ROOT).parts
    if len(parts) == 1:
        return parts[0] in ROOT_ALLOWED
    if parts[0] == "apps" and len(parts) >= 3 and parts[1] in ('filequest', 'loglens', 'localhub', 'safesync', 'labquest', 'unitlab', 'quizdeck', 'textscope', 'ledgerlite', 'badgeforge', 'batchmeter', 'termgarden', 'storedesk', 'minikv', 'netfetch', 'servicedesk', 'asyncboard'):
        return path.suffix in (".rs", ".toml", ".md", ".html", ".css", ".js", ".tsv", ".txt", ".svg", ".ps1")
    if parts[0] == "examples":
        return path.suffix in (".svg", ".txt", ".md", ".html", ".json", ".kv", ".tsv")
    if len(parts) >= 3 and parts[:2] in (("projects", "advanced-b"), ("projects", "advanced-c1")):
        return path.suffix in (".rs", ".toml", ".md", ".lock") or path.name == "Cargo.lock"
    if parts[0] == "course":
        return path.suffix == ".md"
    if parts[0] == "scripts":
        return path.suffix in (".py", ".sh", ".ps1")
    if parts[0] == "verification":
        return path.suffix in (".md", ".txt", ".json") or path.name in ("LOCALHUB_UI_DOM_DESKTOP.png", "LOCALHUB_UI_DOM_MOBILE.png", "STUDIO_DESKTOP.png", "STUDIO_MOBILE.png", "MINIKV_WORKSHOP_DESKTOP.png", "MINIKV_LESSON_MOBILE.png")
    if len(parts) >= 3 and parts[:2] == (".github", "workflows"):
        return path.suffix in (".yml", ".yaml")
    return False


def enumerate_files() -> list[Path]:
    return sorted(
        (p for p in ROOT.rglob("*") if p.is_file() and allowed(p)
         and not any(part in SKIP for part in p.relative_to(ROOT).parts)),
        key=lambda p: p.relative_to(ROOT).as_posix(),
    )


source_files = enumerate_files()
MANIFEST.write_text("".join(
    f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(ROOT).as_posix()}\n"
    for path in source_files
), encoding="utf-8")
subprocess.run([sys.executable, str(ROOT / "scripts/structural_check.py")], check=True)

with zipfile.ZipFile(OUTPUT, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
    for path in [*source_files, MANIFEST]:
        item = zipfile.ZipInfo(f"RustAppLab/{path.relative_to(ROOT).as_posix()}",
                               date_time=(2026, 10, 28, 0, 0, 0))
        item.compress_type = zipfile.ZIP_DEFLATED
        item.external_attr = (0o100755 if path.suffix == ".sh" else 0o100644) << 16
        archive.writestr(item, path.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
with zipfile.ZipFile(OUTPUT, "r") as archive:
    assert archive.testzip() is None
    assert len(archive.namelist()) == len(source_files) + 1

sha = hashlib.sha256(OUTPUT.read_bytes()).hexdigest()
(OUTPUT.parent / f"{OUTPUT.name}.sha256").write_text(f"{sha}  {OUTPUT.name}\n", encoding="utf-8")
print(f"CREATED: {OUTPUT} ({OUTPUT.stat().st_size} bytes, {len(source_files)+1} files)")
print(f"SHA-256: {sha}")
