#!/usr/bin/env python3
"""Execute real Rust/C/WASM contracts; never mutate the original learner."""
import argparse
import functools
import hashlib
import http.server
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import zipfile

ROOT = Path(__file__).resolve().parents[1]
LAB = ROOT / "projects/modern-e4-interop"
CARGO = ["cargo", "+1.100.0"]
def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def check(output, browser):
    output.mkdir(parents=True, exist_ok=False)
    report = {"stage": "E4", "status": "FAIL", "git_commit": os.environ.get("GITHUB_SHA"), "steps": [], "human_learning": "not evaluated"}
    def run(label, cmd, cwd, env, code=0, contains=None):
        completed = subprocess.run([str(x) for x in cmd], cwd=cwd, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=600)
        combined = completed.stdout + "\n" + completed.stderr
        (output / (label + ".txt")).write_text(combined, encoding="utf-8")
        report["steps"].append({"name": label, "returncode": completed.returncode})
        if completed.returncode != code or (contains and contains not in combined):
            raise RuntimeError(label + " failed; see original log")
        print(label, "PASS", flush=True)
        return combined
    def capture_sources():
        return {p.relative_to(LAB).as_posix(): digest(p) for p in LAB.rglob("*") if p.is_file() and not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB).parts)}
    before = capture_sources()
    try:
        archive = output / "E4-learner-offline.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as z:
            for p in sorted((LAB / "learner").rglob("*")):
                if p.is_file() and not {"target", ".git", "__pycache__"}.intersection(p.relative_to(LAB / "learner").parts):
                    z.write(p, p.relative_to(LAB / "learner").as_posix())
        archive_hash = digest(archive)
        with tempfile.TemporaryDirectory(prefix="e4-") as temp:
            scratch = Path(temp)
            ref, learner = scratch / "reference", scratch / "搬移 学员"
            shutil.copytree(LAB / "reference", ref, ignore=shutil.ignore_patterns("target", ".git", "__pycache__"))
            learner.mkdir()
            with zipfile.ZipFile(archive) as z:
                assert z.testzip() is None
                assert z.read("src/lib.rs") == (LAB / "learner/src/lib.rs").read_bytes()
                z.extractall(learner)
            fixed = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "tests/contracts.rs", "c/client.c", "c/signal_bridge.h", "web/index.html", "web/app.js"]
            assert all(digest(ref / p) == digest(learner / p) for p in fixed)
            env = os.environ.copy()
            for label, work in (("reference", ref), ("learner", learner)):
                home = scratch / (label + "-cargo")
                home.mkdir()
                assert not list(home.iterdir())
                target = scratch / (label + "-target")
                env.update(CARGO_HOME=str(home), CARGO_TARGET_DIR=str(target), CARGO_NET_OFFLINE="true")
                # Preserve rustup shims before changing Cargo home.
                run(label + "-compiler", ["rustc", "+1.100.0", "--version"], work, env, contains="rustc 1.100.0")
                if label == "learner":
                    run("learner-compiles", CARGO + ["check", "--frozen", "--all-targets"], work, env)
                    for step in range(1, 4):
                        run(f"learner-red-{step}", CARGO + ["test", "--frozen", "--test", "contracts", f"step{step}_"], work, env, 101, "test result: FAILED")
                    shutil.copyfile(ref / "src/lib.rs", work / "src/lib.rs")
                for mode, flags in (("debug", []), ("release", ["--release"])):
                    run(label + "-" + mode, CARGO + ["test", "--frozen", "--all-targets"] + flags, work, env, contains="3 passed; 0 failed")
                run(label + "-native-build", CARGO + ["build", "--frozen", "--release", "--lib"], work, env)
                release = target / "release"
                shutil.copyfile(work / "c/client.c", release / "client.c")
                shutil.copyfile(work / "c/signal_bridge.h", release / "signal_bridge.h")
                if os.name == "nt":
                    lib = release / "e4_signal_bridge.dll"
                    import_lib = release / "e4_signal_bridge.dll.lib"
                    if not import_lib.is_file():
                        matches = list(release.glob("*e4_signal_bridge*.lib"))
                        assert len(matches) == 1, matches
                        import_lib = matches[0]
                    exe = release / "c-client.exe"
                    run(label + "-c-link", ["cl", "/nologo", "/W4", "/WX", "client.c", "/Fe:c-client.exe", "/link", import_lib], release, env)
                else:
                    lib = release / "libe4_signal_bridge.so"
                    exe = release / "c-client"
                    run(label + "-c-link", ["cc", "-std=c11", "-Wall", "-Wextra", "-Werror", "client.c", "-L.", "-le4_signal_bridge", "-Wl,-rpath,$ORIGIN", "-o", exe], release, env)
                assert lib.is_file()
                run(label + "-c-execute", [exe], release, env, contains="E4_C_ABI_PASS")
                run(label + "-wasm-build", CARGO + ["build", "--frozen", "--release", "--target", "wasm32-unknown-unknown", "--lib"], work, env)
                wasm = target / "wasm32-unknown-unknown/release/e4_signal_bridge.wasm"
                assert wasm.read_bytes()[:8] == b"\x00asm\x01\x00\x00\x00"
                node = """const fs=require('fs'),assert=require('assert/strict');(async()=>{const {instance}=await WebAssembly.instantiate(fs.readFileSync(process.argv[1]),{});const e=instance.exports;assert.equal(e.e4_abi_version(),1);assert.deepEqual([-4,10,19,20,31].map(x=>e.e4_classify(x,10,20)),[0,1,1,2,2]);assert.deepEqual([-4,10,19,20,31].map(x=>e.e4_classify(x,10,19)),[0,1,2,2,2]);assert.equal(e.e4_classify(0,20,20),-1);assert.equal(e.e4_classify(100001,10,20),-1);assert.equal(e.e4_summarize,undefined);console.log('E4_WASM_PASS')})().catch(e=>{console.error(e);process.exit(1)})"""
                run(label + "-wasm-node", ["node", "-e", node, wasm], work, env, contains="E4_WASM_PASS")
                report[label] = {"native_sha256": digest(lib), "wasm_sha256": digest(wasm), "c_exe_sha256": digest(exe)}
                assert all(digest(ref / p) == digest(work / p) for p in fixed)
                registry = home / "registry"
                assert not registry.exists() or all(p.name == "CACHEDIR.TAG" for p in registry.rglob("*") if p.is_file())
                if label == "learner" and browser:
                    shutil.copyfile(wasm, work / "web/e4_signal_bridge.wasm")
                    from playwright.sync_api import sync_playwright
                    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(work / "web"))
                    with http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
                        thread = threading.Thread(target=server.serve_forever, daemon=True)
                        thread.start()
                        try:
                            with sync_playwright() as playwright:
                                chromium = playwright.chromium.launch()
                                page = chromium.new_page(viewport={"width": 1000, "height": 780})
                                page.goto(f"http://127.0.0.1:{server.server_port}/")
                                page.wait_for_function("document.querySelector('#status').textContent.includes('ABI 1')")
                                page.locator("#analyze").click()
                                assert page.locator("#result").inner_text() == "低: 1 | 正常: 2 | 高: 2"
                                page.screenshot(path=str(output / "wasm-default.png"))
                                page.locator("#high").fill("19")
                                page.locator("#analyze").click()
                                assert page.locator("#result").inner_text() == "低: 1 | 正常: 1 | 高: 3"
                                page.screenshot(path=str(output / "wasm-transfer.png"))
                                page.locator("#high").fill("10")
                                page.locator("#analyze").click()
                                assert page.locator("#result").inner_text().startswith("错误：")
                                chromium.close()
                        finally:
                            server.shutdown()
                            thread.join(timeout=5)
                    report["browser"] = {"status": "PASS", "screenshots": {name: digest(output / name) for name in ("wasm-default.png", "wasm-transfer.png")}}
            assert digest(archive) == archive_hash
        assert before == capture_sources()
        report.update(status="PASS", learner_zip_sha256=archive_hash, source_unchanged=True)
    except (OSError, RuntimeError, ValueError, AssertionError, subprocess.SubprocessError) as error:
        report["error"] = str(error)
        print("FAIL", repr(error), flush=True)
    finally:
        (output / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return 0 if report["status"] == "PASS" else 1

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--browser", action="store_true")
    args = parser.parse_args()
    raise SystemExit(check(args.output.resolve(), args.browser))
