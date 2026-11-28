"""Start and stop our own real workbench for maintainer acceptance checks."""
from __future__ import annotations
from contextlib import contextmanager
from pathlib import Path
import os
import re
import subprocess
import time
import urllib.request


@contextmanager
def running_workbench(command, cwd: Path, logfile: Path, env=None):
    """Listen on a chosen or ephemeral port; wait for both the URL and health."""
    with logfile.open('wb') as stream:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=stream,
                                   stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL)
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        try:
            end = time.monotonic() + 30
            while time.monotonic() < end:
                output = logfile.read_text(encoding='utf-8', errors='replace')
                if process.poll() is not None:
                    raise RuntimeError(f'workbench exited ({process.returncode}): {output}')
                match = re.search(r'^RUSTAPPLAB_WORKBENCH_URL=(http://127\.0\.0\.1:\d+)/\r?$', output, re.M)
                if match:
                    with opener.open(match[1] + '/api/health', timeout=5) as response:
                        if response.status != 200:
                            raise RuntimeError('workbench health check failed')
                    yield match[1]
                    return
                time.sleep(.05)
            raise TimeoutError('workbench did not announce its URL: ' + output)
        finally:
            if process.poll() is None:
                if os.name == 'nt':
                    # A .cmd launcher owns a child labquest.exe. Stop the whole
                    # test-owned tree so Windows releases the extracted files.
                    subprocess.run(['taskkill', '/pid', str(process.pid), '/t', '/f'],
                                   capture_output=True, timeout=15, check=False)
                else:
                    process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)



def choose_workbench_course(page, course):
    """Select via the visible navigation, expanding its category if necessary."""
    # A previous course click can be loading asynchronously. Wait for the
    # current course's real operation lock to clear before reading visibility:
    # otherwise navigation may move the target into a closed group mid-click.
    from playwright.sync_api import expect
    expect(page.locator("#course-current [data-course]")).to_be_enabled()
    control = page.locator(f'#course-list [data-course="{course}"]')
    control.wait_for(state="attached")
    if not control.is_visible():
        # Only the optional category is collapsed. Use its visible summary,
        # not injected JS or forced clicks on a hidden learner control.
        group = control.locator("xpath=ancestor::details[1]")
        group.locator("summary").click()
    control.click()
