# RustAppLab Browser Learning Workbench Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans and scoped parallel agents for disjoint files. Record progress and decisions in this plan. The existing user authorization covers implementing the discussed learning-workflow improvement and the established feature-branch → CI → merge workflow.

**Goal:** Read, edit, save, run and genuinely check FirstCard plus 12 Studio projects in one native local browser workbench.

**Architecture:** Embed offline assets in `labquest`; add a bounded loopback HTTP adapter, project model and asynchronous checker jobs. Reuse the existing task cards, project generators and authoritative CLI/checkers.

**Tech Stack:** Rust 1.74-compatible standard library, existing workspace crates, plain HTML/CSS/JavaScript, existing maintainer-only Playwright.

**Spec:** `docs/superpowers/specs/2026-11-23-browser-learning-workbench-design.md`

## Global Constraints

- Rust MSRV 1.74; no new learner Python, Node, WSL or third-party Rust dependency.
- FirstCard and all 12 Studio courses; preserve route and existing checker semantics, without new prerequisite gates.
- Only trusted local loopback use; enforce same-origin mutations, bounded input/output and selected-project file access.
- Never overwrite an existing learner project or silently overwrite externally changed source.
- Ordinary runs and old reports must not manufacture a current PASS.
- Work in `feat/browser-learning-workbench-20261123`; keep changes out of CppAppLab.

## Review Focus

- An old project lacks recent generated docs: open and teach from embedded canonical course data without rewriting source.
- Source changes externally or during a check: conflict on save, retain draft, and mark old results stale.
- A failed Cargo/toolchain launch: clear busy state and show an actionable real error, keeping source intact.
- Windows paths contain spaces/Chinese characters: pass literal argument vectors and preserve paths through forms/JSON.
- Short/mobile viewport: show real tutorial prose, visible output affordance and readable pending controls without horizontal page overflow.

## Task 1: Existing-course model and learner files

**Owner:** domain agent. **Files:** create `apps/labquest/src/workbench/model.rs`; minimal readonly helpers in `learning.rs` / `studio.rs` only as necessary.

**Interfaces:** `Workspace` owns selected project mapping; `new(PathBuf)`, `catalog_json`, `course_json(id,step)`, `lesson_html(id)`, `create(id)`, `open(id,path)`, `file_json(id,path)`, `save(id,path,content,revision)`, `project_path(id)`, `source_digest(id)`, and `validate_step(id,step)`. Functions return `Result<..., ApiError>` where `ApiError {status:u16,message:String}` lives in parent `workbench`; JSON string escaping is exported as `model::json_string` for other workbench modules.

- [x] Add focused tests for real catalog/cards and project identification, non-overwrite, Unicode path, conflict and symlink/traversal cases.
- [x] Implement model and reuse existing navigation/fingerprint logic; expose history as history, not a fresh check.
- [x] Verify relevant tests and review data contracts against the spec.

## Task 2: Local server and real asynchronous operations

**Owner:** root. **Files:** create `workbench.rs`, `workbench/http.rs`, `workbench/jobs.rs`; wire `lib.rs` and `guide.rs`.

- [x] First write/run a black-box CLI test proving the workbench entry is absent at baseline.
- [x] Implement bounded strict HTTP/forms, exact Host/Origin policy, offline assets and same-origin API dispatch.
- [x] Run real existing CLI checks and literal-argument learner runs in one active job; retain raw output, result and source revision.
- [x] Test pending responsiveness, repeated action rejection, actual check failure→success, ordinary run separation and bad HTTP/file inputs.

## Task 3: Integrated learning interface

**Owner:** UI agent. **Files:** create `apps/labquest/src/workbench/ui/index.html`, `app.css`, `app.js`.

- [x] Implement the exact HTTP contract and render existing cards/steps with full-lesson access.
- [x] Build compact desktop/mobile layout, actual editable files, dirty/save state, existing-project path entry and clear pending/retry feedback.
- [x] Keep check/run outcomes separate, retain errors while output is collapsed, and guard async selection/save races and draft loss.
- [ ] Verify syntax and use actual browser acceptance evidence for layout and behavior.

## Task 4: Launch, packaging and browser evidence

**Owner:** delivery/test agent. **Files:** native workbench integration tests, `scripts/workbench_browser_e2e.py`, new Windows/Linux entry scripts, `.github/workflows/ci.yml`, packaging/assembly checks as required.

- [x] Extend the existing browser job with real labquest-backed learner flows and untouched baseline-page screenshots.
- [ ] Verify FirstCard/UnitLab actual failures, edits, saves, real PASS, run, reload, existing project reuse and external-save conflict.
- [ ] Capture 1440×1000, 1366×768, 390×844 and 320×844; verify actual visible prose/editor/dock geometry and controls.
- [x] Package embedded assets and obvious launchers into source/runtime artifacts; retain old terminal entry.

## Task 5: Documentation, integration and delivery

**Owner:** root.

- [x] Update learner entry documents, exact new workbench scope, operational limits and development experience.
- [x] Refresh `SOURCE_MANIFEST.sha256` after all source changes.
- [x] Run local available baseline and relevant regression suites; independent whole-change review.
- [ ] Push branch, inspect exact-head CI and real screenshots, fix concrete defects, then merge and verify main.
- [ ] Deliver user-visible screenshots and direct startup/update instructions; state actual coverage and remaining existing course paths accurately.

## Progress and rulings

- Baseline inspected: `00ae3f95b6e11bfde63a7b10eca74353149a1bd3`; no open PRs or local edits in the inspection checkout.
- Isolated worktree created at `RustAppLab-workbench` on the feature branch.
- Ruling: use scoped parallel ownership for the domain model, UI and delivery tests while root owns transport/jobs — independent files and a written HTTP contract reduce turnaround without mixing edits.
- Ruling: integrate the complete FirstCard/Studio family now; keep independent and advanced course execution at their established entries — their contracts and tools are different, and the user asked to improve the current learning experience rather than expand curricula.

- Local native baseline: the unchanged LabQuest test suite passes with Rust 1.85 after exposing the host SQLite runtime through a task-local linker symlink; no project workaround or system package change.
- New CLI black-box test failed at baseline because `workbench` did not exist, then passed with the actual native server. Current native flows cover FirstCard/UnitLab red→green, existing Unicode project paths, conflict, busy state and stale results.
- Model regression reproduced identical starter files in two projects accepting an old revision; project-bound revisions/digests now reject the write. File reads also return the digest of the same snapshot.
- Independent review found dirty-draft loss on cross-tab same-course step navigation and current PASS persisting after loading externally changed source; production-JavaScript regressions now cover both, same-project metadata refresh before a check, and external changes on return to the browser. File reads return a same-snapshot digest; job completion always checks the editor file revision and idle focus verification preserves drafts.
- Release identity: use new 0.19.0-rc.4; the tag was absent before work. Existing main publishing creates immutable version tags, so rc.3 must not be reused.
- Local Rust 1.85 strict Clippy exposed two baseline warnings (SHA hex operator grouping and a redundant return binding). Applied behavior-preserving parentheses/direct return fixes to satisfy the existing warning gate; no checker requirements changed.
- Actual Chromium screenshots, Windows native packaging and Rust 1.74 remain CI gates; local native checks do not substitute for them.

- Local integrated LabQuest suite: 192 passed before the additional same-snapshot digest regression; current focused workbench library suite: 21 passed. Python discovery: 172 passed. Strict whole-workspace Clippy and format checks passed on Rust 1.85. UI state regressions are also required by the real-browser CI job.

- Independent final review: no outstanding Critical/Important findings after production-JavaScript checks of cross-tab draft preservation, stale results on file load/metadata refresh, focus revalidation, and discarded late focus responses after changing projects. Exact-head browser/MSRV/Windows evidence is still pending CI.
