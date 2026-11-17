# RustAppLab · D5 开发经验记录

Date: 2026-11-17  
Scope: Rust Book chapter 20.1–20.3, PR #8

## 1. Source manifest is a release artifact, not a hand-maintained checklist

The first PR CI run failed at the structural gate on both Linux and Windows. The C1/C2 portable distribution checks also failed at their source-inventory comparison. The D5 code had not yet reached the native Cargo compile/test stage in those jobs.

Root cause: `SOURCE_MANIFEST.sha256` contained hashes that did not match the final bytes of several modified course files and the new D5 workspace. The source-package job regenerates the manifest and deliberately runs `git diff --exit-code -- SOURCE_MANIFEST.sha256`; any stale or incorrectly calculated entry is therefore a hard failure.

**Rule:** finish all source, documentation, packaging and test edits first; then regenerate the manifest using the repository's authoritative `python scripts/package.py` path. Do not hand-edit hashes or reimplement SHA-256 in a maintenance script. Verify the generated manifest is clean in a second run before interpreting downstream delivery failures as product regressions.

## 2. Classify CI failures by the first failed gate

A top-level failed job does not prove that Rust compilation or runtime behavior failed.

For this run:
- `tooling-tests` passed on both native platforms.
- `structure` failed because it checks every tracked source hash against `SOURCE_MANIFEST.sha256`.
- C1/C2 portable source inventory failed because packaging regenerated a different manifest.
- Browser smoke and the standalone C1/C2 stable / 1.85.0 jobs passed.

**Rule:** inspect the first failed step and its captured evidence before changing application code. If structural validation fails, repair the inventory and rerun the pipeline; do not treat downstream jobs as independent D5 functional failures. The native engineering evidence file and source-inventory diff are the primary diagnostic artifacts.

## 3. Keep Rust Book reference labs isolated from learner products

D5 lives under `projects/book-d5-webserver/` as a separate Workspace:
- Edition 2021, declared Rust 1.74 MSRV.
- No registry dependencies; verification uses `--locked --offline`.
- `publish = false`.
- It is excluded from the root Workspace and the frozen 17-package learner product set.

This preserves the existing release boundary while allowing the book-derived lab to be built, linted, tested, documented and packaged independently. The source archive and learner kit must both include the lesson, workspace manifests, Rust sources, tests and served HTML sample.

## 4. Make concurrency tests deterministic

Worker-pool tests should coordinate with channels, a Condvar or a barrier. Do not assert a particular request completion order and do not rely on arbitrary sleeps to prove parallelism.

For shutdown, enqueue all worker termination messages before joining any worker. Joining immediately after sending one shutdown message can deadlock if a different worker consumes that message. The pool's Drop contract is: queued jobs drain, each worker receives a shutdown message, then all worker handles are joined.

## 5. Respect TCP stream boundaries

TCP does not preserve application write or packet boundaries. A CRLF terminator may arrive in separate reads, so a buffer fragment must never be treated as a complete HTTP line. The request reader now accumulates one bounded line through LF with `read_until`, applies both per-line and total-header limits, and has a one-byte-at-a-time reader regression test. Keep this test when changing the socket reader; it protects a real stream-framing edge case without depending on packet timing.

## 6. State the HTTP teaching subset explicitly

This lab is a learning implementation, not a replacement for a production HTTP stack. Its supported subset and exclusions must be stated in the lesson and README: HTTP/1.1 GET, bounded headers, no request body, no keep-alive, no TLS and no production-grade backpressure. Parser limits and unsupported inputs need explicit tests so the implementation does not imply broader protocol support than it provides.

## 7. Keep release and MSRV gates in sync

Each isolated reference workspace must be registered in both `scripts/release_check.py` and `scripts/msrv_check.py`. The native release gate alone proves behavior on the installed stable compiler; it does not prove the declared minimum Rust version. Add a regression assertion that the workspace appears in both plans whenever a new D-stage workspace is introduced.

## 8. D5 acceptance sequence

1. Run the D5 isolated Workspace format, Clippy, debug tests, release tests and doctests.
2. Run the structural gate and tooling tests.
3. Regenerate `SOURCE_MANIFEST.sha256` with `scripts/package.py`; require a clean second run.
4. Build and inspect both source and learner-kit archives, including the D5 HTML sample and local relative links.
5. Require Linux, Windows and Rust 1.74 CI evidence before marking D5 complete.
6. Keep the PR draft until all required gates pass; do not merge based only on a successful subset.

This record distinguishes verified failures from unverified stages. A structural-gate failure is not a Rust runtime failure, and a green packaging step is not evidence that a learner has mastered the chapter.
