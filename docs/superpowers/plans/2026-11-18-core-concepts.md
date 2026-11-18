# Core concepts and IDE entry implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans for the owned task. Keep file ownership disjoint and record actual test results.

**Goal:** Fill verified concept-level teaching gaps and make the retained IDE exercise directly usable, then ship one accepted RC batch.

**Architecture:** Extend the existing isolated Foundations lab with three modules. Extend the D/E runner with temporary workspace artifacts while keeping all shared integration edits with the root agent.

**Tech Stack:** Rust Edition 2021 / 1.74 minimum, Python 3.10+, Markdown, existing GitHub Actions matrix.

**Spec:** `docs/superpowers/specs/2026-11-18-core-concepts-design.md`

## Global Constraints

- Foundation workspace remains Edition 2021, Rust 1.74, zero third-party dependencies; root learner inventory remains 17.
- No learner mastery or IDE PASS is inferred from CLI/reference tests.
- Do not install tools/extensions, launch IDEs, write personal editor settings, overwrite evidence or mutate original reference source during experiments.
- Keep existing rc.1/rc.2 immutable; current delivery is 0.19.0-rc.3.
- Independent agents edit only assigned files; root owns `lib.rs`, versions, source inventory, CI, packaging and audit integration.

## Review Focus

- UTF-8 byte ranges, empty text and reversed/out-of-bound ranges return the documented values without accidental panics.
- Active borrows are genuinely used after conflicting operations; compile-fail examples fail for the intended rule and have successful controls.
- I/O errors remain distinguishable from parse errors; `?` and explicit match preserve identical first-line behavior.
- IDE paths with spaces and moved retained directories still resolve; successful cleanup leaves no advertised live entry, and failure retains recoverable evidence.
- Old translation claims and scoped concept totals do not become whole-book completeness or learner achievement claims.

## Task 1: Ownership, borrowing, struct receivers and slices

Files: create `projects/book-d6-foundations/foundation-lab/src/ownership.rs`, `tests/ownership.rs` under the same crate, and `course/BOOK_OWNERSHIP_PRACTICE.md`.

Interfaces: implement the exact ownership API in the spec; root adds `pub mod ownership;`. Do not edit shared files.

- [x] Write tests for clone independence, mutation then consuming title, moved fields/Copy remainder, Unicode/empty words and valid/invalid UTF-8 byte windows. Run before implementation and observe the missing API or TODO failure.
- [x] Implement minimal behavior; add paired successful/compile-fail doctests for moves, shared mutation, active mutable borrows, NLL, dangling return, struct update, Option type and exhaustive match. Record intended error codes.
- [x] Write a beginner guide with predict → run → change → explain, real paths/commands and links to existing D3/D4 examples where reuse is appropriate.
- [x] Verify scoped tests and doctests on 1.74 and 1.85; report commands/counts and error controls.

## Task 2: Collections and error propagation

Files: create `projects/book-d6-foundations/foundation-lab/src/collections.rs`, `tests/collections.rs` under the same crate, and `course/BOOK_COLLECTIONS_ERRORS.md`.

Interfaces: implement the exact collections/error API in the spec; root adds `pub mod collections;`. `ScoreError` implements Display/Error and From for I/O and parse errors.

- [x] Write failing behavioral tests for missing indices, Unicode mutation, duplicate key replacement, successful/invalid/EOF first lines and a real failing reader boundary.
- [x] Implement APIs and runnable examples for Vec mutation/enum values, String ownership (`+`/`format!`), UTF-8 indexing/slicing, map ownership/access/entry, explicit match versus `?`.
- [x] Explain `get` returns `Option<&V>` and `or_insert` returns `&mut V`; do not copy outdated translation signatures.
- [x] Verify focused tests/doctests and report actual results; hand remaining concept inventory to root.

## Task 3: IDE entry and honest records

Files: modify only `scripts/book_d6_check.py`, `scripts/tests/test_book_d6_practical.py`, `course/BOOK_D6_PRACTICAL.md`.

Interfaces: preserve modes/steps and existing report fields. Add direct retained project/source/workspace paths, generated `D6-IDE.code-workspace` with relative `lab`, workspace-only cargo toolchain/offline settings, and a blank editable record in the temporary workspace. No automatic editor launch or installation.

- [x] Tests first: relative paths survive a moved workspace with spaces; report success-retained / success-cleaned / failure-retained states correctly; original protected files stay unchanged; editable records are outside evidence hashes.
- [x] Generate entry for copied tools/edition workspaces, preserve Nightly behavior, print useful paths only if retained, keep generated record status NOT_RUN.
- [x] Add concrete Task definition jump and E0308 edit/repair instructions, explicit rust-src preparation, version-layer distinctions and troubleshooting.
- [x] Run focused Python tests and real tools/edition retained runs. Report actual paths/version/hash outcomes, not GUI acceptance.

## Task 4: Generics, lifetimes, integration and release

Files: new `src/generic_lifetimes.rs` / `tests/generic_lifetimes.rs` in Foundations and `course/BOOK_GENERICS_LIFETIMES.md`; root-owned audit, README/entrypoints, package/structural scripts, test floors, VERSION/Cargo files, release notes, experience and source manifest.

- [x] Write tests then implement Card/Summary/conditional impl and lifetime functions. Add two-type generic and failed-bound examples, elision explanations and borrowed struct controls.
- [x] Integrate module declarations and all guides into real learner/source archives; incorporate the audited concept inventories without extrapolation.
- [x] Record rc.2 acceptance and this batch's lessons; bump current files to rc.3 without rewriting historical evidence.
- [x] Raise foundation test floors to actual verified counts; run full affected suites, stable formatting/Clippy, 1.74/1.85, real D/E/G and actual ZIP checks.
- [ ] Whole-batch independent review; fix concrete findings with evidence; commit and open PR; wait exact-head full CI, inspect candidate, merge; verify main CI, checkpoint, immutable rc.3 and all published asset digests.

## Execution ledger

- Baseline: main `ed255fe31ec1be2faba04ffb720660d13938e6a6`, rc.2, no open PR, clean checkout; Python baseline 126/126 PASS.
- Ruling: use separate sibling worktree `/workspace/scratch/e7b5b7306cab/RustAppLab-concepts`; preserve the clean main checkout.
- Ruling: parallel work is limited to disjoint task files; root alone integrates shared files, avoiding overlapping module/CI/version edits.
- Ruling: actual VS Code operation remains unassessed because this environment has no verified editor installation; generated entry and CLI evidence do not change D to complete.

- Verified: full Foundations matrix on Rust 1.74.0 and 1.85.0, each Debug 48 / Release 48 / doctest 76, no failures or ignored tests. Stable isolated-workspace fmt and Clippy also passed.
- Verified: independent scoped audit retains 118 stable targets; teaching 38 → 87, with 25 partial / 2 related / 4 gap remaining. Counts apply only to chapters 4–6 and 8–10, not the whole book.
- Task 1/2 independent review: no critical/important findings; guide fixes create the examples directory before the probe and update both assertions after changing the sample input.
- Verified: final local D/E/G reruns passed with unchanged source snapshots; D/E retained entry bytes match hashed `ide-workspace.json`, blank record remains NOT_RUN, and the candidate validator accepted both actual Linux reports. Evidence is in `verification/rc3-core-concepts/`.
- Verified: fresh independent review reproduced all 24 intended compile-fail diagnostic controls on both Rust versions (48/48); no unrelated error codes.
- Verified: all Python regression suites passed, 131/131; actual learner archive contains 359 members, with 17 top-level learner projects unchanged. Final source archive and full remote CI remain separate gates.
- Verified: final source ZIP has 792 members / 791 manifest entries with every hash matching; final learner ZIP has 359 members / 17 learner projects and every packaged course input byte matches the frozen tree. An earlier learner ZIP predated the final audit wording and was rebuilt before acceptance.
- Whole-batch independent review complete: no Critical/Important/Minor changes required. Reviewer rechecked diagnostic controls, report validation, all 118 audit IDs and current source/learner archives; ready for exact-head remote CI. Raw command logs remain byte-for-byte intact, including their original trailing blank lines and diff context.
