#!/usr/bin/env python3
"""Maintainer rehearsal of the real four-app course path, not learner certification.
Only loopback HTTP; all output goes into a new directory. No third-party Python packages.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import time
from typing import Iterator
from urllib import error, parse, request

ROOT = Path(__file__).resolve().parents[1]
APPS = ("servicedesk", "netfetch", "batchmeter", "minikv")


def checked(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def inspect_snapshots(before: bytes, after: bytes, titles: list[str]) -> None:
    """Check actual content and identity, not just exit codes or file existence."""
    a, b = json.loads(before), json.loads(after)
    checked(a["count"] == b["count"] == len(titles), "snapshot counts differ")
    checked([x["title"] for x in a["tickets"]] == titles, "before titles differ")
    checked([x["title"] for x in b["tickets"]] == titles, "after titles differ")
    checked([x["id"] for x in a["tickets"]] == [1, 2], "unexpected ticket IDs")
    checked([x["id"] for x in b["tickets"]] == [1, 2], "ticket IDs changed")
    checked([x["done"] for x in a["tickets"]] == [False, False], "bad initial state")
    checked([x["done"] for x in b["tickets"]] == [True, False], "bad close state")


def inspect_statistics(text: str, files: list[Path]) -> None:
    rows = text.splitlines()
    checked(len(rows) == len(files) + 1, "unexpected number of stats rows")
    for row, path in zip(rows[:-1], files):
        name, size, lines = row.rsplit("\t", 2)
        data = path.read_bytes()
        checked(Path(name) == path, "file order or path changed")
        checked(int(size) == len(data), "byte size mismatch")
        # This fixture is single-line JSON, so Rust lines() and splitlines() agree.
        checked(int(lines) == len(data.decode("utf-8").splitlines()), "line count mismatch")
    checked(rows[-1] == f"files={len(files)} bytes={sum(p.stat().st_size for p in files)} lines={len(files)}", "bad totals")


def run(bin_dir: Path, out: Path) -> dict:
    binaries = {name: (bin_dir / (name + (".exe" if os.name == "nt" else ""))).resolve() for name in APPS}
    missing = [str(p) for p in binaries.values() if not p.is_file()]
    if missing:
        raise FileNotFoundError("Build these Rust programs first: " + ", ".join(missing))
    out.mkdir(exist_ok=False)  # Refuse to alter a prior learner artifact, even on failure.
    out = out.resolve()
    calls: list[dict] = []
    events: list[dict] = []
    report = {"status": "RUNNING", "kind": "maintainer-rehearsal-not-student-record", "platform": os.name,
              "applications": {name: {"path": str(p), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for name, p in binaries.items()},
              "calls": calls, "http_events": events}
    opener = request.build_opener(request.ProxyHandler({}))

    def cli(app: str, *args: object, expected: int = 0) -> str:
        command = [str(binaries[app]), *map(str, args)]
        result = subprocess.run(command, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=20)
        item = {"app": app, "args": list(map(str, args)), "exit": result.returncode,
                "stdout": result.stdout, "stderr": result.stderr}
        calls.append(item)
        checked(result.returncode == expected, f"Unexpected exit: {item}")
        return result.stdout

    def http(base: str, path: str, fields: dict | None = None) -> dict:
        body = parse.urlencode(fields).encode("ascii") if fields is not None else None
        req = request.Request(base + path, data=body, headers={"Content-Type": "application/x-www-form-urlencoded", "X-RustAppLab-Client": "1"})
        with opener.open(req, timeout=3) as response:
            data = json.load(response)
            events.append({"path": path, "method": "POST" if fields is not None else "GET", "status": response.status, "data": data})
            return data

    @contextmanager
    def server(journal: Path, number: int) -> Iterator[str]:
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        with (out / f"server-{number}.log").open("wb") as log:
            proc = subprocess.Popen([str(binaries["servicedesk"]), "serve", str(journal), str(port)],
                                    stdout=log, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL)
            try:
                base = f"http://127.0.0.1:{port}"
                deadline = time.monotonic() + 8
                while True:
                    if proc.poll() is not None:
                        raise RuntimeError(f"Service exited before ready; inspect server-{number}.log")
                    try:
                        http(base, "/api/tickets")
                        break
                    except (OSError, error.URLError):
                        if time.monotonic() >= deadline:
                            raise TimeoutError("Service did not become ready")
                        time.sleep(.05)
                yield base
            finally:
                if proc.poll() is None:
                    proc.terminate()
                    try:
                        proc.wait(timeout=4)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                        proc.wait(timeout=4)

    try:
        journal = out / "tickets.log"
        before, after = out / "before.json", out / "after.json"
        titles = ["解释 channel 为什么一直等待", '给自己的 "Rust <档案>" 加回归测试']
        cli("servicedesk", "init", journal)
        with server(journal, 1) as base:
            for title in titles:
                http(base, "/api/tickets", {"title": title})
            cli("netfetch", "fetch", base + "/api/tickets", before)
            http(base, "/api/close", {"id": 1})
            cli("netfetch", "fetch", base + "/api/tickets", after)
            inspect_snapshots(before.read_bytes(), after.read_bytes(), titles)
            saved = before.read_bytes()
            cli("netfetch", "fetch", base + "/api/tickets", before, expected=2)
            checked(before.read_bytes() == saved, "existing snapshot was modified")
            absent = out / "should-not-exist.json"
            cli("netfetch", "fetch", base + "/missing", absent, expected=2)
            checked(not absent.exists(), "failed request left a file")
        # A newly launched server, not just a CLI read, proves replay across process restarts.
        with server(journal, 2) as base:
            restarted = http(base, "/api/tickets")
            checked(restarted == json.loads(after.read_bytes()), "state changed after server restart")
        one = cli("batchmeter", "scan", 1, before, after)
        four = cli("batchmeter", "scan", 4, before, after)
        checked(one == four, "thread-count change altered output")
        inspect_statistics(one, [before, after])
        (out / "statistics.txt").write_text(one, encoding="utf-8")
        db = out / "index.kv"
        for key, value in [("snapshot/before", "before.json"), ("snapshot/after", "after.json"), ("course/next", "AsyncBoard")]:
            cli("minikv", "put", db, key, value)
        checked(cli("minikv", "get", db, "snapshot/after").strip() == "after.json", "index not persisted")
        scanned = cli("minikv", "scan", db, "snapshot/")
        checked(scanned == "snapshot/after\tafter.json\nsnapshot/before\tbefore.json\n", "prefix scan wrong")
        cli("minikv", "export", db, out / "index.tsv")
        index = (out / "index.tsv").read_bytes()
        cli("minikv", "export", db, out / "index.tsv", expected=2)
        checked(index == (out / "index.tsv").read_bytes(), "export overwrote an existing file")
        report.update(status="PASS", cli_calls=len(calls), server_launches=2, checks=[
            "real HTTP create and close", "decoded JSON and title identity", "restart replay", "no overwrite",
            "404 creates no output", "ordered equal stats at workers=1/4", "independent byte/line totals", "persistent prefix index"])
        (out / "README.md").write_text("# 个人工单档案 · 维护者真实演练\n\n这里是参考程序实际运行生成的文件，不是真人学员成绩。\n"
                                       "before/after.json 为真实 HTTP 下载；statistics.txt 为 Rust 文件统计；index.kv/tsv 是持久索引。\n"
                                       "ServiceDesk 已实际停止并重新启动，状态保持一致。工具检查不证明学员已独立理解并发。\n", encoding="utf-8")
        return report
    except Exception as exc:
        report.update(status="FAIL", error=f"{type(exc).__name__}: {exc}")
        raise
    finally:
        (out / "SYSTEMS_REHEARSAL.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin-dir", type=Path, default=ROOT / "target" / "release")
    parser.add_argument("--output", type=Path, required=True, help="new directory; will not overwrite")
    args = parser.parse_args()
    try:
        result = run(args.bin_dir, args.output)
    except (OSError, AssertionError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        parser.exit(1, f"SYSTEMS REHEARSAL FAILED: {exc}\n")
    print(f"SYSTEMS REHEARSAL PASS: {result['cli_calls']} real CLI calls; artifacts at {args.output}")

if __name__ == "__main__":
    main()
