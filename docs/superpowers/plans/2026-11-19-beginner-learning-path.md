# Beginner learning path Implementation Plan

> **For agentic workers:** Apply the existing execution and review workflow; complete the documentation and generated guidance as one integrated delivery.

**Goal:** A learner who completes FirstCard can identify what they practiced, verify it with their own input, and start the next appropriate task without guessing a command or directory.

**Architecture:** Keep the existing course sequence and assessment contracts. Make `course/AFTER_FIRSTCARD.md` the FirstCard-to-UnitLab bridge; use the same progression in entry documents, application READMEs, and generated learner instructions. Reuse existing task-card metadata for generated Studio README content.

**Tech Stack:** Markdown, existing static HTML, Rust LabQuest content generation, existing PowerShell entry points.

**Spec:** User feedback on 2026-11-19: READMEs are too terse; after FirstCard a beginner cannot tell what to do or what they learned. This is a correction to the existing learning experience, not a new curriculum.

## Constraints and review focus

- Preserve FirstCard → UnitLab → QuizDeck → TextScope → LedgerLite → BadgeForge → TermGarden → BatchMeter → MiniKV → NetFetch → ServiceDesk → StoreDesk → AsyncBoard → independent projects.
- Preserve learner code, existing completion records, and all assessment requirements.
- Label command working directories; provide executable commands rather than CLI fragments.
- Distinguish reference programs, learner TODOs, optional Book labs, and data examples.
- Describe observable abilities and transfer exercises; automated checks never certify understanding.
- Keep Windows/Linux, Rust version, SQLite and C1/C2 dependency requirements accurate.
- Check old learner folders, names with spaces/Chinese, next-command copy/paste, generated file portability, and changed-source progress.

## Tasks

- [x] Root: rewrite the front-door route, add the concrete FirstCard bridge, resolve the conflicting legacy FileQuest handoff, and align the Windows course menu.
- [x] App documentation worker: improve all 17 app READMEs using real code, commands, step contracts, outcomes and next actions.
- [x] Project documentation worker: improve advanced/reference/example entry documents, preserving their different roles and prerequisites.
- [x] Generated guidance worker: add portable FirstCard/Studio READMEs and actionable completion guidance using existing course data; retain old records.
- [x] Run existing document/link checks and relevant Rust beginner/learning/Studio tests; walk FirstCard → UnitLab → QuizDeck with documented commands.
- [x] Independently review the integrated diff, correct concrete issues, and record development experience in verification/README_LEARNING_HANDOFF_20261119.md.
- [x] Publish one reviewable PR. The initial automatic approval review rejected publication before authorization; the user explicitly authorized it on 2026-11-19. The exact reviewed tree is published as commit a374ae4 in PR #12. Later checks and housekeeping are tracked by the follow-up plan and PR.

Review corrections: remove duplicate archive members; explain and run personal tests separately; make first-failure hints executable with quoted paths; preserve old-project entry fallbacks and the existing Tokio boundary. All 113 relevant Rust tests and 24 Python checks passed after correction. Format and actual learner-package checks passed; Rust 1.85 Clippy warnings in two unchanged baseline files, Windows native execution and browser screenshots remain explicitly unverified or blocked as recorded.

Documentation edits need no new snapshot test framework. Meaningful generation/handoff regressions belong in the existing integration tests. Release-wide CI remains the repository's existing gate; do not turn this documentation correction into a new release project.
