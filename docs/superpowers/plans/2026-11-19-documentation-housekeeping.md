# Documentation housekeeping implementation plan

> For agentic workers: use the execution and verification skills; keep historical evidence distinct from current source validation.

**Goal:** Publish the completed learning guidance and make the source repository understandable to people already using it.

**Design:** Keep all active learner entrypoints and project paths stable. Add a maintainer index; move obsolete design narratives, release notes and pre-0.19 verification into clearly named archives. Preserve raw evidence bytes and original conclusions. Root compatibility pages point existing bookmarks toward current material.

**User authorization:** On 2026-11-19 the user said “推送” and asked to organize source documentation and historical/cache clutter. This authorizes the feature-branch publication and reversible organization below; no merge, tag or Release replacement is included.

**Implementation choice:** A navigation-only pass would leave stale “current” reports in place; deleting them would discard useful history. Archive them with explicit indexes and stable learner paths. Keep the currently consumed concept audit and D5/D6 experience paths unchanged.

## Constraints and review focus

- Keep the existing version, courses, tests, launchers and learner work intact.
- Raw historical JSON/TXT/PNG files must match their original Git blobs after moving.
- Both source packaging and structural validation must include the new Markdown destinations.
- Relative links must resolve in an extracted source ZIP, not only in this checkout.
- Old PASS records and plan checkboxes do not certify the current commit or a learner's understanding.
- Build caches and generated reports are distinct from examples, fixtures and users' `my-...` projects; do not mass-delete them.

## Tasks

- [x] Audit current navigation, archive candidates, packaging consumers and CI contracts.
- [x] Publish the completed learning-guidance tree on the authorized branch and create PR #12; GitHub-connected publication preserved the exact local tree.
- [x] Add current maintainer navigation and source-directory explanations.
- [x] Archive old root narratives and release entries, retaining compatibility entrypoints.
- [x] Move pre-0.19 verification as complete groups; add current and historical evidence indexes.
- [x] Include the new docs paths in source packaging and structural validation; keep generated local reports out of tracked evidence.
- [x] Validate archived raw bytes, all changed local links, an actual extracted source ZIP and relevant existing Python tests.
- [x] Obtain one independent review of the housekeeping changes and resolve material findings.
- [ ] Regenerate the canonical source manifest, commit, publish the follow-up and update PR #12.
- [ ] Inspect CI for the final published revision and report its actual result.

## Publication record

The initial normal Git push could not authenticate locally. The authorized GitHub connector created tree `416e22b969dcf5cc556298dcffd8103f567a1c98`, byte-identical to local commit `e905e9b`; published commit `a374ae495659add5f5fa092b44749a6ab5528541` has that same tree and base parent. The original local commit remains on a local backup branch. PR: https://github.com/clonetimer/RustAppLab/pull/12.

Review correction: the frozen v0.19.0-rc.3 release lacks the newly introduced navigation. The three active start pages now identify PR #12 and provide the revision-branch source ZIP, while preserving existing learner directories and frozen release assets.
