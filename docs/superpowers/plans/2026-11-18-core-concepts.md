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
- Verified before the later CI artifact fix: initial source ZIP has 792 members / 791 manifest entries with every hash matching; learner ZIP has 359 members / 17 learner projects and every packaged course input byte matches the frozen tree. An earlier learner ZIP predated the final audit wording and was rebuilt before acceptance. The artifact fix requires a refreshed source inventory and archive.
- Whole-batch independent review complete: no Critical/Important/Minor changes required. Reviewer rechecked diagnostic controls, report validation, all 118 audit IDs and current source/learner archives; ready for exact-head remote CI. Raw command logs remain byte-for-byte intact, including their original trailing blank lines and diff context.

## Task 5: Correct artifact selection after CI retry

- [x] Preserve Run 226 first-attempt failure and same-head Windows retry evidence; verify source and logs.
- [x] Reproduce selection of the older, higher-ID duplicate; inspect upstream v4 download code.
- [x] Implement/test deterministic newest-by-created_at current-run selection with no outcome fallback.
- [x] Implement/test exact-ID downloads, SHA256-before-extraction, safe fresh directories and selection trace.
- [x] Wire narrow read permissions, record experience, independently review, refresh actual packages and source inventory.
- [ ] Commit the fix and use a new complete CI run on that exact head before merging; finish main/release gates.

The original Run 226 cannot qualify the corrected source. All 18 producer jobs eventually passed on the original head, but candidate assembly failed because it consumed the first Windows artifact. Final acceptance must name the new head/run rather than reuse the original head's status.

- Verified locally after the artifact fix: full Python suite 169/169 PASS, including 20 selector and 18 downloader regressions. The real 19-record incident snapshot selected 18 producer artifacts; the actual retained Windows success ZIP passed API digest, safe extraction, both report statuses and byte-identical trace checks. Network transfer was substituted with the retained bytes, so remote permissions and downloads remain part of the new CI gate.
- Delta review before commit `2de8dcd76f965f0a846dd633266f3a3c3f14d6e5`: no Critical/Important/contract-required Minor findings. Refreshed source ZIP contains 798 members / 797 manifest entries, all matching current bytes, SHA256 `cd8e57a09cfb140eaf2ae7d297358a3635e0748fc849e99db1c7bb6fe0598ce4`. Learner ZIP remains 359 files / 17 projects, SHA256 `9ee47ca346cb1435c7c6cc9d4b63a6e72e4b325fc64392cd1f94e36548ea21cf`; all current course inputs match. No source or test edits followed that passing suite and review before the commit. Run 227 subsequently exposed the Windows-only fixture issue below.

## Task 6: Preserve hostile ZIP names in cross-platform test fixtures

- [x] Retain Run 227 Windows native artifact and isolate the single 168/169 failure to ZIP test serialization. Reproduce Windows separator normalization independently; no production path bypass was demonstrated.
- [x] Make test ZIP entry names explicit, assert their serialized roundtrip, and reproduce RED/GREEN under the Windows separator boundary without skipping Windows tests.
- [x] Run all Python tests, obtain narrow independent review, record experience and refresh the final source archive.
- [ ] Commit, run full CI on the new head, then finish the original PR/main/release gates. Run 227 cannot qualify the changed fixture.

- Verified: actual Windows failure retained; Windows-separator boundary reproduced RED against the original fixture, GREEN after the fixture-only fix. Full Python suite 169/169 PASS; narrow independent review found no blocker and reran the 18 downloader tests. Production downloader unchanged. Source ZIP refreshed to 799 members / 798 manifest entries, SHA256 `be3360b6fb984eb93858495a79c695b8efc9647fb090c6bdc6467a24f3f1f7af`; all members match current source bytes. Learner inputs and its 359-member / 17-project archive are unchanged from the preceding verified package.

## Task 7: Keep transient selection outside the source identity boundary

- [x] Before candidate assembly, reproduce source_id contamination from selected-artifacts.json at the checkout root. Retain the differential probe; a file outside checkout preserves identity.
- [x] Move the selection path to RUNNER_TEMP, keeping strict source comparison and the copied evidence trace unchanged.
- [ ] Narrow review, refresh source archive and commit, then use the new full CI head. Run 228 cannot qualify this changed workflow.

- Verified after placement change: complete Python suite 169/169 PASS. An independent reviewer reproduced both the root-file contamination and the unchanged identity for runner-temp / nested collected evidence. Source ZIP refreshed to 800 members / 799 manifest entries, SHA256 `5fc60cbf4ba24537615772014791fc8c90efe339a4c9bb81c30db864f79cabf2`; every packaged byte matches the current tree. Source identity is `f8d51cfc5b60212470ce95548a93070b807a92e46f6a9bc4abcdb3f417bfb0e7`. Final commit/run acceptance remains recorded separately in the PR to avoid treating these local checks as remote success.
