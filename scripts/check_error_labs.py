#!/usr/bin/env python3
"""Execute the ownership/borrowing examples as actual Rust compiler experiments."""
from __future__ import annotations
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOC = ROOT / 'course' / 'RUST_ERROR_LABS.md'
EXPECTED = ['E0382','E0502','E0308','E0597']

def verify() -> tuple[int, int]:
    text = DOC.read_text(encoding='utf-8')
    blocks = re.findall(r'^```(rust(?:,compile_fail)?)\n(.*?)^```\s*$', text, re.M | re.S)
    if len(blocks) != 8 or [tag.endswith('compile_fail') for tag, _ in blocks] != [True, False] * 4:
        raise RuntimeError(f'expected 4 failing and 4 corrected Rust examples, got {len(blocks)}')
    with tempfile.TemporaryDirectory(prefix='rust-error-labs-') as tmp:
        folder=Path(tmp)
        for index,(kind,source) in enumerate(blocks):
            file=folder / f'lab{index}.rs'
            binary=folder / f"lab{index}{'.exe' if sys.platform == 'win32' else ''}"
            file.write_text(source,encoding='utf-8')
            result=subprocess.run(['rustc','--edition','2021',str(file),'-o',str(binary)],capture_output=True,text=True,encoding='utf-8',timeout=20)
            if kind=='rust,compile_fail':
                expected=EXPECTED[index//2]
                if result.returncode == 0 or not re.search(rf'^error\[{expected}\]:',result.stderr,re.M):
                    raise RuntimeError(f'Lab {index//2+1} must fail with {expected}; exit={result.returncode}; stderr={result.stderr[-500:]}')
                print(f'[EXPECTED FAIL] {expected}')
            else:
                if result.returncode != 0:
                    raise RuntimeError(f'Corrected lab {index//2+1} did not compile: {result.stderr[-500:]}')
                run=subprocess.run([str(binary)],capture_output=True,text=True,encoding='utf-8',timeout=8)
                if run.returncode != 0:
                    raise RuntimeError(f'Corrected lab {index//2+1} did not run: {run.stderr[-500:]}')
                print(f'[PASS] corrected lab {index//2+1}')
    return 4,4

if __name__ == '__main__':
    try:
        failing,correct=verify()
    except (RuntimeError,OSError,subprocess.TimeoutExpired) as exc:
        print(f'FAIL: {exc}',file=sys.stderr)
        sys.exit(1)
    print(f'PASS: {failing} intentional compiler errors; {correct} corrected Rust programs compiled and ran')
