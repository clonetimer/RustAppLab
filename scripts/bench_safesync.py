#!/usr/bin/env python3
"""SafeSync 真实字节复制/校验/恢复基准，不以固定速度阈值判定通过。

使用：先 cargo build --release -p safesync，再 python3 scripts/bench_safesync.py
仅使用 Python 标准库；临时数据在函数结束时销毁，用户源文件不参与测试。
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import random
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MIB = 1024 * 1024


def hash_file(path: Path) -> str:
    dig = hashlib.sha256()
    with path.open("rb") as input_file:
        while chunk := input_file.read(MIB):
            dig.update(chunk)
    return dig.hexdigest()


def run_step(binary: Path, *args: Path | str) -> tuple[float, str]:
    start = time.perf_counter()
    process = subprocess.run(
        [str(binary), *(str(arg) for arg in args)],
        capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=240,
    )
    elapsed = time.perf_counter() - start
    if process.returncode:
        raise RuntimeError(f"命令失败 {args}: code={process.returncode}, stderr={process.stderr[-1000:]}")
    return round(elapsed, 5), process.stdout.strip()


def main() -> None:
    parser = argparse.ArgumentParser(description="SafeSync 文件级真实基准（同机结果才有可比性）")
    default_binary = ROOT / "target/release" / ("safesync.exe" if os.name == "nt" else "safesync")
    parser.add_argument("--binary", type=Path, default=default_binary)
    parser.add_argument("--files", type=int, default=64)
    parser.add_argument("--mib", type=int, default=32)
    parser.add_argument("--output", type=Path, default=None)
    args = parser.parse_args()
    if not 1 <= args.files <= 5000 or not 1 <= args.mib <= 512:
        parser.error("--files 应为 1..5000，--mib 应为 1..512")
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"缺少可执行程序：{binary}；先 cargo build --release -p safesync")
    rng = random.Random(20261028)
    result = {
        "project": "Rust AppLab 0.6.0",
        "binary": str(binary),
        "binary_sha256": hash_file(binary),
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "os": platform.platform(),
        "python": platform.python_version(),
        "cpu_count": os.cpu_count(),
        "files": args.files,
        "logical_bytes": args.mib * MIB,
        "dataset": "pseudo-random deterministic seed 20261028; local temporary directory",
    }
    with tempfile.TemporaryDirectory(prefix="rust-applab-benchmark-") as temp:
        base = Path(temp)
        source = base / "source"
        source.mkdir()
        total = args.mib * MIB
        file_sizes = [total // args.files + (1 if i < total % args.files else 0) for i in range(args.files)]
        for i, length in enumerate(file_sizes):
            path = source / f"test-{i:05d}.bin"
            with path.open("wb") as output:
                while length:
                    part = min(length, MIB)
                    output.write(rng.randbytes(part))
                    length -= part
        snapshot, restored = base / "snapshot", base / "restored"
        backup_s, backup_stdout = run_step(binary, "backup", source, snapshot)
        verify_s, verify_stdout = run_step(binary, "verify", snapshot)
        restore_s, restore_stdout = run_step(binary, "restore", snapshot, restored)
        # 独立于 Rust 备份程序，用 Python hashlib 校验每份真实数据。
        start = time.perf_counter()
        for path in sorted(source.iterdir()):
            original_digest = hash_file(path)
            if original_digest != hash_file(snapshot / "data" / path.name):
                raise RuntimeError(f"备份内容不匹配: {path.name}")
            if original_digest != hash_file(restored / path.name):
                raise RuntimeError(f"恢复内容不匹配: {path.name}")
        crosscheck_s = round(time.perf_counter() - start, 5)
        result.update({
            "backup_seconds": backup_s,
            "verify_seconds": verify_s,
            "restore_seconds": restore_s,
            "independent_sha256_seconds": crosscheck_s,
            "backup_mib_per_second": round(args.mib / backup_s, 3) if backup_s else None,
            "verify_mib_per_second": round(args.mib / verify_s, 3) if verify_s else None,
            "restore_mib_per_second": round(args.mib / restore_s, 3) if restore_s else None,
            "all_content_equal": True,
            "backup_stdout": backup_stdout,
            "verify_stdout": verify_stdout,
            "restore_stdout": restore_stdout,
            "benchmark_limits": [
                "Wall time includes disk cache and directory traversal; not a controlled cold-cache benchmark",
                "No wall-power-loss, network storage, Windows/macOS or concurrent modification test",
                "Timing not guaranteed reproducible outside the recorded environment",
            ],
        })
    out = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(out, encoding="utf-8")
        print(f"记录已写入 {args.output}")
    print(out)


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
        print(f"Benchmark FAILED: {error}", file=sys.stderr)
        sys.exit(1)
