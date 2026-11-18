# Rust Book Practical Closeout Implementation Plan

> For agentic workers: use the execution and verification workflows task by task. The user has already requested continued implementation; this is the execution record for the existing plan.

**Goal:** Make the remaining D6 practice reproducible, repair known beginner gaps, record lessons, and deliver a verified RC.2.

**Architecture:** Reuse the isolated foundation and appendix labs. A Python runner executes tools, migration and optional Nightly in unique temporary copies; the ordinary root and MSRV remain unchanged.

**Tech Stack:** Rust 1.74 / 1.85 / pinned Nightly, Python standard library, GitHub Actions on Linux and Windows.

**Spec:** ../specs/2026-11-18-book-practical-closeout.md

## Global Constraints

- Root workspace: 17 packages, Edition 2021, declared Rust 1.74 for foundations.
- D6: no registry dependencies, publish disabled.
- Nightly is opt-in, never a replacement for Stable/MSRV evidence.
- Preserve existing release tags, assets and historical reports.
- Topic mapping, executable behavior and learner mastery are separate claims.

## Review Focus

- A clean minimal toolchain lacks fmt/clippy; installation instructions must name all components.
- Paths with spaces, repeated output paths and failed commands must not lose previous evidence.
- Migration must run cargo fix under the old edition before changing the manifest.
- A Nightly gate alone does not prove a feature still needs Nightly; test Stable without the gate too.
- Course files and runner must exist in actual archives, and publication must use a new version.

## Tasks

- [x] 1. Extend foundation-lab and its lessons with Cargo, tuple/array, variable and module experiments. Verify new behavior and compiler errors with real Rust 1.85 and 1.74.
- [x] 2. Implement scripts/book_d6_check.py and behavior tests. Verify safe output handling, failure records and source preservation. Run tools, Edition migration and pinned Nightly for real.
- [x] 3. Update the practical guide, exact directory audit, experience record, packaging and CI. Validate actual source/learner archives and all Python tooling tests.
- [ ] 4. Advance current identities to rc.2, refresh source manifest, run independent review and full PR CI, then merge and verify main checkpoint/publication and released checksums.

## Execution ledger

- Baseline: PR #10 head 6a5827f4e3ce6d009836a7d2beaa9c9285ada24a; local 106 Python tests PASS. Fresh isolated checkout on the existing feature branch.
- Reproduced: minimal Rust 1.85 fails cargo fmt because cargo-fmt is not installed. Install the named components before executing the guide.
- Source review: development experience section 8 contains a duplicated body; reconstruct the text and verify unique section headings.
- Ruling: include the independently found variable/module teaching gaps in the existing D6 batch; they share the same lab and add no learner products.
- Ruling: use rc.2 because Run 220's publication error proves rc.1 is immutable and already assigned. Do not weaken the tag protection.

- Tasks 1–3 local verification: 125 Python tests PASS; foundation Debug/Release 29 and doctests 15 on Rust 1.85/1.74; current stable format and foundation Debug PASS; practical tools/edition/nightly 15/14/6 steps PASS with unchanged scoped inputs.
- Archive check: source 716 entries (715 exact manifest hashes), learner 348 entries and 17 expected learner projects; no target/cache members.
- Review fix: update foundation floors to 29/29/15 and prevent command echoes from duplicating test-summary counts; regression RED→GREEN and release-check suite 26/26.
- Review clarification: D1 current-version wording and exact IDE/lab/doc paths corrected.
- Remote integration remains gated by exact-head PR CI, immutable main checkpoint and rc.2 publication; the final run URLs and merge SHA will be recorded on PR #10.
