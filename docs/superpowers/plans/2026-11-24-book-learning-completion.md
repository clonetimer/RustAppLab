# Rust Book learning completion

**Goal:** Complete the user's P1/P2 priorities on existing PR #14 and make P3 observation usable without inventing learner results.

**Authority:** User's priority table and established feature branch → CI → merge workflow. Continue `feat/book-learning-bridge-20261123` from `288d09f`; do not recreate previously completed work. Use the existing workbench specification for safety and platform requirements; its earlier non-goal of course expansion is superseded only by the user's explicit D1–D6 integration request.

**Architecture:** Reuse the native Rust workbench and real compiler/checker. Embed the existing D1–D6 projects for offline copies; reuse the current editor, optimistic saves and bounded jobs. Add searchable knowledge navigation and diagnostic-linked isolated compiler experiments. No new learner runtime or Rust dependency.

## Constraints and decisions

- Rust 1.74 remains the baseline; C1/C2 and D6 Edition/toolchain migration keep their documented 1.85 requirements.
- The 31 IDs are the historical six-chapter/118-concept sample, not the entire Book. Preserve the rc3 audit and add dated evidence.
- Successful reference tests, successful learner code, and observed understanding are different results. No invented learner scores or Stable status.
- Create private learner copies; never overwrite reference projects or existing work. Preserve source revisions, stale result handling, bounded files, exact Origin and literal process arguments.
- D5 server and D6 library use real tests through the browser; do not start an unbounded server in the generic run action. Toolchain migration/nightly/publish are still explicit advanced operations outside the one-click check.
- Parallel ownership: root UI/routes/error experiments; backend agent `book_projects.rs`, `model.rs`, `jobs.rs`; concept agent foundations source/tests and 31-item teaching/audit; evidence agent learner logs/protocol/packaging/legacy evidence gate. Integration and review stay with root.

## Tasks

- [x] P1: Search concepts, open current official Book and local lesson, navigate to corresponding editable project; compiler failures link to relevant explanation and executable failure/repair pair. Meaningful JS and native experiment tests, then real browser verification.
- [x] P2 concepts: Supply 31 explicit concept explanations, predictions, runnable successes/failures/repairs and transfer questions; verify the four original gaps including abort in a separate process and actual backtrace contrast.
- [x] P2 projects: Append seven existing Book workspaces to the course model. Copy embedded sources, edit nested Rust files, run real all-target and doc tests, retain actual exit codes and stale-source semantics. Test wrong project, manifest/test edits, traversal/symlink rejection and actual compile failure/recovery.
- [x] P3: Make learning logs available from FirstCard onward; supply concrete human observation/transfer tasks and package them. Keep unobserved fields unassessed; reject placeholder evidence in the legacy release gate.
- [ ] Integrate: Refresh source manifest, format/check/test locally, inspect desktop/mobile screenshots, independent review, push same PR, verify exact-head cross-platform CI, merge and verify main under existing authorization.

## Review focus

1. Course switches and knowledge links must preserve unsaved drafts.
2. Intentional compiler failure must match its intended error; infrastructure failure cannot count as a successful experiment.
3. Book source edits invalidate current checks; immutable tests/manifests cannot be edited through the browser.
4. Runtime package works without a source checkout; practice requires an installed compiler exactly as before.
5. A complete-looking log or a successful test cannot be promoted into real learner evidence or a mastery score.

## Evidence and progress

- Inspected latest remote CI `45228436579`: browser passes; two native jobs stop at import ordering in `book_gap4.rs`. Existing local baseline: Node 11/11, Python 173/173.
- Fresh task checkout has no Rust initially; installing a task-local Rust 1.85 compiler for native verification.

- Implemented all four product tasks. Local proof: JavaScript 14/14; Python 185/185; integrated LabQuest 209 tests before the final API/reference-contract regressions, those focused regressions also pass. All seven embedded Book copies execute real tests without source-tree paths. Foundations Debug 56, Release 56, doctests 134; 25 deliberate failures match their exact compiler codes.
- Independent product review found two important defects: D6-A could be emptied to zero tests, and learner-kit reference directories lacked protection. Added fixed independent behavior tests and package-layout recognition, each red→green. A dialog-control stale-disabled defect also has a production-JS red→green regression.
- Ruling: keep opening existing Book projects read-only. Copies with outdated immutable contracts get an actionable refusal and must migrate source into a fresh copy; do not silently add test files while opening.
- Ruling: compiler comparisons use a separate bounded temporary job store and cannot change course checks or learner files. The browser prevents overlapping normal actions; the backend may perform one bounded isolated comparison alongside a course job.
- Real browser launch is blocked by this environment's socket policy. No bypass or fabricated screenshot; the CI Chromium job executes the existing flow plus the new Book/diagnostic flow. Exact-head CI, screenshots and merge remain pending.
