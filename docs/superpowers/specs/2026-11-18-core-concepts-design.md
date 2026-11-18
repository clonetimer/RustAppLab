# Core concepts and IDE entry design

## Intent and authority

Continue the user's approved Rust Book coverage work, record experience, and deliver a coherent batch through feature branch, exact-head CI, merge, checkpoint and immutable RC publication. This follows rc.2 / PR #10; it does not repeat that completed work. The user's repeated instruction to continue supplies execution authority; routine implementation choices are made here without another approval pause.

## Evidence and approach

The 81-row directory audit is complete as a mapping, not as a concept inventory. Read-only audits of the original chapters 4–6 and 8–10 found concrete teaching gaps while also locating relevant examples in TextScope, MiniKV, D2–D4. Reuse those references rather than count them as absent. New small exercises bridge the concepts that are currently only implicit in larger projects.

Considered: (1) only expand the coverage table (cheap but leaves learning gaps); (2) create another full learner application (duplicates the existing 17-project path); (3) extend the existing isolated Foundations reference lab and practical runner. Choose (3), with a scoped concept audit and explicit pointers to existing teaching. The 17 learner projects remain unchanged.

## Deliverables

1. Ownership/borrow/slice and non-Copy struct experiments, with successful examples, intended compiler failures, paired repairs and personal modification questions.
2. Vec/String/HashMap access and mutation experiments, plus explicit match/`?` error propagation and `From` conversion.
3. A concise generic card/default-trait/conditional-impl lab and two-input lifetime examples. Explain elision and borrowed structs without implying lifetimes extend values.
4. A generated VS Code workspace for retained D/E copies, direct project/source paths, a blank editable IDE record, exact jump/diagnose/repair steps, and explicit separation of CLI versus editor extension/server versions.
5. A concept-level inventory for the audited chapters only, identifying existing evidence, additions and remaining partial items. Do not extrapolate its count to the whole book.
6. Updated entry points, actual packages, source manifest, acceptance floors, development experience and an immutable rc.3 release.

## Constraints

- Foundation workspace remains Edition 2021, Rust 1.74, zero third-party dependencies; root learner inventory remains 17.
- New content is reference practice, not automatically credited learner mastery.
- Original rust.bootcss.com numbering is the baseline; correct old translation/API mistakes using primary documentation and actual compilers.
- No external publication from teaching examples, no global editor settings, no automatic extension/toolchain installation, no automatic IDE launch.
- IDE workspace paths are relative so the whole retained directory can move. Any toolchain setting is restricted to this temporary workspace.
- Editable personal IDE records live outside hashed CLI evidence. D remains partial until actual editor operation evidence exists.
- Preserve existing rc.1/rc.2 tags and assets; bump current version to 0.19.0-rc.3 before main integration.
- Failure evidence is retained, default successful temporary work is cleaned, and all new report paths accurately reflect retention.

## Interfaces and file boundaries

- `ownership.rs`: `ReadingNote { title: String, minutes: u32, completed: bool }`; methods `new(String,u32)`, `title(&self)`, `append_to_title(&mut self,&str)`, `into_title(self)`; `first_word(&str)->&str` uses Unicode whitespace; `preview(&str,usize,usize)->Option<&str>` checks byte ranges. Struct update and Option/match constraints are exercised directly in examples.
- `collections.rs`: `select_label(&[String],usize)->Option<&str>`, `annotate_labels(&mut [String],&str)`, `scores_by_name(&[(&str,u32)])->HashMap<String,u32>` (last value wins), `ScoreError::{Io,Parse}`, and `read_score` / `read_score_explicit` over `BufRead` (read first line, trim and parse as u32; EOF is an empty parse error). Provide owned and borrowed key, concatenation, enum-vector examples in runnable documentation.
- `generic_lifetimes.rs`: `Card<T>`, `Summary` required label/default summary, conditional Display implementation, `choose_longer` (UTF-8 byte length, ties choose left), `first_label` with independent second argument, `Excerpt<'a>` and explained method elision.
- Root integrates module declarations, packaging, audit, versions and CI floors. Implementers own separate module/test/guide files and do not modify shared integration files.
- IDE helper extends `scripts/book_d6_check.py`; adds no new runner mode and preserves existing CLI and step contracts. D/E CI exercises retained copies; Nightly remains opt-in.

## Acceptance

Meaningful RED→GREEN for new functions/path behavior; foundation Debug/Release/doc tests on actual 1.74 and 1.85; current stable fmt/Clippy; compiler diagnostic controls; Python suite; real D/E runs with retained/generated paths; source and learner ZIP inspection. Review once as a whole batch, fix concrete issues, then use final-head PR CI and main publication gates. No fabricated IDE or learner PASS.
