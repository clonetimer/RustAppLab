#!/usr/bin/env python3
"""仅在已完成本机 Linux x86_64 release 构建后，打包十七个 Rust 可执行程序。"""
from __future__ import annotations

import hashlib
import os
import platform
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APPS = ('filequest', 'loglens', 'localhub', 'safesync', 'labquest', 'unitlab', 'quizdeck', 'textscope', 'ledgerlite', 'badgeforge', 'batchmeter', 'termgarden', 'storedesk', 'minikv', 'netfetch', 'servicedesk', 'asyncboard')
VERSION = (ROOT / "VERSION").read_text(encoding="utf-8").strip()
if platform.system() != "Linux" or platform.machine().lower() not in ("x86_64", "amd64"):
    raise SystemExit("只支持 Linux x86_64 本机打包；不伪造其他操作系统发行包")
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", str(ROOT / "target"))) / "release"
EXAMPLES = {"termgarden-workbench": "workbench", "termgarden-ownership-lab": "ownership_lab"}
OUT = ROOT.parent / f"RustAppLab-{VERSION}-linux-x86_64-release.zip"
for app in APPS:
    if not (TARGET / app).is_file():
        raise SystemExit(f"缺少编译产物 {TARGET / app}，先 cargo build --release --workspace")

instructions = """Rust AppLab 0.19.0-dev.1 · Linux x86_64 已构建程序

保留十七个主程序，另附两个TermGarden进阶示例可执行文件。它们在提供的 Linux Rust 1.99.1 环境构建；无需 Cargo 即可运行参考程序。StoreDesk 另需系统 libsqlite3.so.0；未把系统共享库打入此包。
这不是 Windows/macOS 二进制，不含联网服务、安装器或自动更新功能。

在解压后的目录：
  ./filequest help
  ./filequest demo fq-demo
  ./loglens demo log-demo
  ./localhub demo hub-demo
  ./localhub serve --root hub-demo/files --log hub-demo/logs/candidate.log
  # 再从本机浏览器访问 http://127.0.0.1:8765/
  ./safesync demo safe-demo
  ./safesync verify safe-demo/backup
  ./safesync restore safe-demo/backup safe-demo/recovered-again
  ./unitlab convert 25 c f
  ./quizdeck demo
  ./textscope demo
  ./ledgerlite demo
  ./badgeforge make "My Rust" "Created by me" new-badge.svg
  ./batchmeter help
  ./termgarden-workbench demo advanced-demo
  ./termgarden-workbench play my-tasks.tasks
  ./termgarden-ownership-lab
  ./labquest advanced start my-workbench
  ./labquest advanced check my-workbench 1
  ./termgarden demo new-garden-demo
  ./termgarden play garden.tasks
  ./minikv demo new-kv-demo
  ./minikv share-demo shared-demo.kv
  ./storedesk doctor
  ./storedesk demo new-store-demo
  ./netfetch demo new-network-download
  ./asyncboard demo new-async-demo
  ./asyncboard preview new-async-demo/jobs.tsv
  ./servicedesk demo new-service-demo
  ./servicedesk serve new-service-demo/tickets.log 8766
  ./labquest studio start unitlab my-unitlab
  ./labquest studio check my-unitlab 1
  ./labquest beginner start my-first-card
  # 修改 my-first-card/src/main.rs 中 println! 的那一行
  ./labquest beginner check my-first-card
  ./labquest start case01
  ./labquest next case01
  ./labquest practice
  ./labquest predict 400 6
  ./labquest board case01
  ./labquest app init my-taskpocket
  ./labquest app hint 1
  ./labquest app catalog
  ./labquest app init pagemint my-pagemint
  ./labquest app hint pagemint 1
  ./labquest learn lesson my-pagemint 1
  ./labquest learn check my-pagemint 1
  # 打开 case01/deliverables/CASE_BOARD.html

注意：labquest 的 test-app / app check / beginner check / studio check 都需要本机安装 Rust/Cargo；app init 可创建源码骨架。
请阅读生成的 CONTRACT.md，亲手完成 TaskPocket 或 PageMint 的独立软件逻辑后再运行相应的 app check。
不要对真实资料做破坏测试。SafeSync 默认拒绝已有路径，并不保留元数据/ACL。
项目教学、源码、安全边界见 RustAppLab-0.19.0-dev.1-source.zip；当前不声明 macOS 支持。
"""
with zipfile.ZipFile(OUT, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zipf:
    hashes = []
    for app in APPS:
        binary = (TARGET / app).read_bytes()
        info = zipfile.ZipInfo(app, date_time=(2026, 10, 28, 0, 0, 0))
        info.external_attr = 0o100755 << 16
        info.compress_type = zipfile.ZIP_DEFLATED
        zipf.writestr(info, binary, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
        hashes.append(f"{hashlib.sha256(binary).hexdigest()}  {app}\n")
    for name, example in EXAMPLES.items():
        binary = (TARGET / "examples" / example).read_bytes()
        info = zipfile.ZipInfo(name, date_time=(2026,10,31,0,0,0))
        info.external_attr = 0o100755 << 16
        zipf.writestr(info,binary,compress_type=zipfile.ZIP_DEFLATED,compresslevel=9)
        hashes.append(f"{hashlib.sha256(binary).hexdigest()}  {name}\n")
    for filename, data in (
        ("README_BINARY_RELEASE.txt", instructions.encode("utf-8")),
        ("BINARIES_SHA256SUMS.txt", "".join(hashes).encode("utf-8")),
        ("LICENSE", (ROOT / "LICENSE").read_bytes()),
    ):
        info = zipfile.ZipInfo(filename, date_time=(2026, 10, 28, 0, 0, 0))
        info.external_attr = 0o100644 << 16
        info.compress_type = zipfile.ZIP_DEFLATED
        zipf.writestr(info, data, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
with zipfile.ZipFile(OUT) as check:
    assert check.testzip() is None
    assert len(check.namelist()) == len(APPS) + len(EXAMPLES) + 3
sha = hashlib.sha256(OUT.read_bytes()).hexdigest()
(OUT.parent / f"{OUT.name}.sha256").write_text(f"{sha}  {OUT.name}\n", encoding="utf-8")
print(f"Linux binaries ZIP: {OUT} ({OUT.stat().st_size} bytes)")
print("SHA-256:", sha)
