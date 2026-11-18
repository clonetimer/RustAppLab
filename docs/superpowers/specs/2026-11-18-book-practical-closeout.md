# Rust Book practical closeout and RC.2

## Intent and accepted scope

Continue the user's existing Rust Book completion plan and record development experience. The target is the chapter structure at https://rust.bootcss.com/, with usable beginner exercises and reproducible engineering evidence. The user's established workflow is feature branch, full verification, PR, merge, immutable checkpoint and candidate delivery. Stable still requires real learner observations.

## Decisions

- Extend the existing D6 foundation workspace for Cargo basics, compound types, variables and module paths. This avoids a new product or a second learner inventory.
- Replace duplicated manual migration commands with one cross-platform Python runner that creates unique temporary workspaces and retains command logs, source snapshots and migration differences. Keep the guide focused on what learners should predict, change and explain.
- Run tools and Edition migration on Linux and Windows in CI. Nightly remains an explicit optional experiment; its results never replace the Rust 1.74 gate.
- Keep the 81-row directory mapping distinct from detailed topic verification and learner mastery. Correct the module-path mapping and do not invent a percentage for full conceptual coverage.
- Advance the unpublished current source to 0.19.0-rc.2. Main Run 220 failed at publication because rc.1 already belongs to another commit. Preserve that tag and its assets.

## Constraints

Root learner workspace: 17 packages. Foundation/reference baseline: Edition 2021 and Rust 1.74. No registry dependencies for D6. Edition 2024 and Nightly only in disposable copies. No registry publication. No Stable claim or fabricated learner evidence. All CI claims must identify the exact source commit and completed run.

## Alternatives considered

Correcting only the Markdown would leave the command sequence unexecuted by CI. Adding another full learner product would increase setup work without improving the identified concepts. The selected approach reuses the current lab and adds a small execution harness with explicit evidence boundaries.

## Acceptance

True compile success/failure examples for new Rust concepts; Debug, Release, doctest and actual MSRV checks; tools/Edition runner on both supported platforms; optional pinned Nightly with two Stable negative controls; preserved source files; exact 81 identifiers; archived course/script inclusion; fresh full PR and main CI; immutable rc.2 assets with matching source identity and checksums.
