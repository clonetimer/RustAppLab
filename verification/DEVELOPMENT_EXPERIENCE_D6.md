# RustAppLab · D6 开发经验记录

Date: 2026-11-17; reviewed 2026-11-18

Scope: D6-A/B references, practical verification, beginner gaps and immutable RC.2 delivery

## 1. Keep reference labs isolated from learner products

D6-A and D6-B are reference workspaces, not additional learner projects. They remain outside the root Cargo workspace and do not change the frozen 17-package learner set. Each lab declares Edition 2021, Rust 1.74 MSRV, `publish = false`, and no registry dependencies.

**Rule:** a coverage reference lab must be independently buildable and testable without silently changing the learner product inventory. Register every reference workspace in structural validation, release checks, declared-MSRV checks, source packaging, learner-kit packaging, POSIX/PowerShell verification, and regression tests.

## 2. Make interactive examples deterministic at the test boundary

The guessing game uses an injected comparison function and a small time-seeded xorshift helper only for the interactive demonstration. Unit tests exercise fixed secret/guess pairs; they never wait for wall-clock randomness or assert a particular random output.

The time-seeded helper is explicitly documented as non-cryptographic. Do not present a small PRNG as suitable for security-sensitive randomness. When a future lesson requires a production RNG, introduce the `rand` crate only with a pinned lockfile and a reproducible offline dependency strategy.

## 3. Do not mutate process environment in unit tests

The config-path lesson uses `config_dir_from(Option<&OsStr>)`. The CLI reads environment variables, but the core path decision is a pure function. This avoids tests racing over global process environment and makes Windows/POSIX path behavior easier to isolate.

**Rule:** inject environment-derived values into pure functions; reserve direct `std::env` access for the application boundary.

## 4. Demonstrate panic versus Result with an explicit decision boundary

Ordinary invalid input is represented as `Result`; panic is reserved for a broken programmer invariant. A panic test may use `catch_unwind` to observe behavior, but the lesson must not imply that catching arbitrary panics is a general recovery strategy.

## 5. Benchmark equivalent work and never assert a performance winner

Loop and iterator versions receive the same slice and repeat count, and the test checks equal results only. Timing is an opt-in release-mode observation using `Instant` and `black_box`; no CI test asserts that one form is faster.

**Rule:** CI validates semantic equivalence and that the benchmark runs. Performance claims require repeated measurements, release builds, controlled inputs, and a report of variability—not a single wall-clock assertion.

## 6. Preserve I/O errors while using iterator adapters

The buffered line-count example uses `BufRead::lines` with `try_fold`. Each line read can fail, and `?` propagates the error. Do not convert an I/O error into an empty line or silently skip it unless the application contract explicitly permits that behavior.

## 7. Keep Appendix keyword and symbol references edition-aware

D6-A now includes the complete current Strict/Reserved/Weak keyword sets and explains the Edition 2018/2024 changes. The older Chinese book is retained as the audit chapter baseline, while current token classification is linked to the Rust Reference.

**Rule:** never copy a historical keyword table forward without checking the current Rust Reference and Edition Guide. Distinguish current syntax from historical Edition behavior.

## 8. Treat documentation tables and generated lesson text as tested deliverables

A previous programmatic Markdown edit duplicated the document body at the macro-metavariable symbol `$` and broke a backtick pair. The later experience record repeated that corruption. The 2026-11-18 audit reproduced the defect with a unique-title/section test before reconstructing this section.

**Rule:** use literal replacements or structured patches when text contains `$`, backticks or regular-expression replacement tokens. After a generated edit, review the diff and check unique headings, table completeness and links. Keyword presence cannot detect an entire duplicated document.

## 9. Apply the isolated workspace's exact stable rustfmt output

D6-B passed the source/structural/tooling checks and Linux/Rust 1.74 compilation, but the first native CI attempt stopped at the D6-B formatting gate. The diff was limited to rustfmt layout in the library assertion/error constructors and CLI test process handling. The workspace was formatted to current stable rustfmt and the source manifest refreshed before the complete cross-platform rerun.

**Rule:** every isolated workspace must be checked with its exact manifest command. A root-workspace `cargo fmt --all -- --check` does not include reference workspaces excluded from the root workspace. Formatting is a real gate, not a cosmetic afterthought.


## 10. Treat D6 as one acceptance batch, and verify the actual archives

The final D6 acceptance must cover more than compiling the reference crate. Run 215 validated Linux and Windows native engineering, browser, C1/C2 regressions, offline portable archives, delivery relocation, and the actual Rust 1.74 MSRV. D6-B's exact evidence is 14 library unit tests + 6 CLI integration tests + 2 doctests, all passing in Debug/Release where applicable.

The assembled candidate was opened and inspected independently:
- The source archive contains 651 manifest-tracked files; every entry exists and every SHA-256 matches.
- The learner kit contains both D6-A and D6-B lesson/reference files.
- All 17 expected learner projects remain present; D6 reference workspaces are not counted as new learner projects.
- All 12 outer candidate file hashes match the release checksum list.

**Rule:** inspect the actual assembled ZIPs, not just the packaging script's success message. Count the learner projects by the published package contract, verify the source manifest against extracted bytes, and check that reference labs are included without changing the learner-project inventory.

## 11. Separate implementation coverage from learner mastery

At the PR #10 starting point, the audit recorded 77 items with related implementations, 3 partial items and 1 reference index across 81 rows. This was a directory-mapping inventory. The 2026-11-18 review found missing beginner objectives underneath some of those mapped rows. Author-side reproducibility and learner mastery must be assessed independently: missing learner submissions alone do not show that a course implementation is missing.

**Rule:** report implementation evidence and learner evidence separately. CI, doctests, package integrity, and browser tests prove engineering contracts; learner mastery still requires the learner to complete exercises and explain the results.

## 12. Audit concepts behind the directory labels

The 81 original identifiers were checked against the actual old Chinese Book directory. A count and a few sample IDs were insufficient: the regression now compares the full ID set. The previous 7.3 mapping said Cargo path dependencies even though that chapter teaches Rust module paths. The review also found missing explicit variable/shadowing and compound-type practice.

**Rule:** retain three separate claims: topic mapping, reproducible reference behavior, and learner mastery. Do not report 77/81 as conceptual coverage, and do not interpret zero rows labeled gap as proof that no knowledge is missing. A detailed remaining-concept count stays unquantified until a full concept audit exists.

## 13. Reproduce setup on a genuinely minimal toolchain

On a newly installed Rust 1.85.0 minimal profile, the old guide's cargo fmt command failed with cargo-fmt not installed. Clippy was also not part of that profile. Installing components onto stable while invoking unqualified commands could select a different directory override.

**Rule:** install every required component explicitly and select the same toolchain in every command. The new guide pins 1.85.0 for tools/Edition practice. The runner records actual compiler/tool versions; invoking an installed executable does not prove a learner completed IDE navigation or diagnostics.

## 14. Fix under the old Edition, then change the manifest

The appendix overview and the practical guide previously contradicted each other: the overview changed edition before cargo fix. The corrected order follows the official Edition Guide: establish a baseline, run fix while the old edition is active, then change edition and the temporary MSRV, format and test.

A temporary gen identifier deliberately needs a rewrite to r#gen. This creates observable migration evidence instead of accepting a no-op command as a convincing exercise. Keep the before/after manifest and source diff. The actual project remains Edition 2021 / Rust 1.74.

Reference: https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html

## 15. Temporary work must not erase previous learner evidence

The earlier PowerShell and POSIX examples reused fixed temporary directory names and recursively deleted those paths. A native command could fail without stopping the PowerShell sequence, and the final cleanup could remove the failed workspace before the learner saved evidence.

**Rule:** create one unique workspace per run, refuse an existing evidence destination, stop on a failed command, and retain the failed copy. Save logs and before/after evidence outside the disposable workspace before cleanup. Hash the original reference sources and root manifests before and after; absence of rust-toolchain files is also a state to preserve. The learner explicitly requests retention for IDE practice.

## 16. Verify why a negative example failed

The new array examples were checked directly with rustc as well as rustdoc: incorrect length and element type produced E0308; their corrected versions compiled. Variable/privacy examples produced E0384, E0308, E0070, E0603 and E0616. Missing imports must not accidentally make a compile-fail example look valid.

The Nightly exercise uses two Stable controls: one checks E0554 with the feature gate, and one removes that gate and checks the generic constant-expression rejection. A feature that later stabilizes should cause this exercise to need review. Tool installation failure or an arbitrary nonzero exit is not a passing negative test.

## 17. Build directories can leak through extension filters

After running the foundation lab locally, course_inputs included target incremental .lock files. An extension allow-list alone had treated compiler output as course source. A regression created real generated .rs and .lock files in a unique target child and reproduced the leak before the fix.

**Rule:** exclude target, .git and cache directories while enumerating every reference lab. Inspect the actual ZIP member names, contents and local links after building, not only in a clean checkout. Ship the practical runner with its guide in both source and learner archives.

## 18. Keep version identities and publication immutable

Main Run 220 used commit 34e39ebab056e1f893486cc1713e43f9efc33fa4. Its 17 verification/candidate jobs and checkpoint passed, but Publish immutable RC assets failed at Existing release tag points to another commit. Version v0.19.0-rc.1 already points to fed4a2d37a72de88dfb353474544efde9bdd537a and was published on 2026-11-04.

**Rule:** advance the current candidate to rc.2 and update VERSION, LabQuest/Cargo.lock and active download instructions together. Keep historical notes and evidence intact. Never force-move a release tag or overwrite old assets to turn a failed publication green. Engineering checks passing does not imply that a new downloadable release exists.

Evidence: https://github.com/clonetimer/RustAppLab/actions/runs/44107176066

## 19. Bind claims to the actual revision and evidence scope

Run 222 passed for the earlier practical-guide-only revision, but it did not execute migration or Nightly. New D/E Linux and Windows jobs execute the runner, and the candidate waits for those jobs. Nightly remains opt-in and never replaces the actual Rust 1.74 gate.

The runner's source snapshot has a narrow, documented scope: root configuration and the appendix lab inputs. It is not interchangeable with the full release source_id. Preserve exact commands, exit codes and log digests; verify the expected input snapshot when assembling a candidate.

## 20. Keep beginner tasks small and explain the result

Cargo vocabulary is now attached to the actual foundation package, executable and import names. Arrays and tuples use three scores; variables and modules use a small study plan. New exercises ask learners to predict, run, modify and explain rather than just copy a command.

The local foundation batch passed Rust 1.85.0 and actual Rust 1.74.0 Debug/Release with 29 runtime tests and 15 doctests. These are reference checks, not 44 completed learner objectives. Cross-platform and final candidate evidence must come from the current PR/main revision. Personal learning records remain unassessed until submitted.

## 21. Update acceptance floors when adding executable exercises

Independent review found that the native/MSRV gates still accepted the old foundation counts of 20 runtime tests and 2 doctests. The new batch actually contains 29 runtime tests and 15 doctests. Without updated floors, losing the new integration tests could still pass the old threshold.

A new run_step regression then revealed a second issue: the log includes its command line, and an unanchored test-summary regex counted a printed summary inside that command as another suite. A report of 20 tests was counted as 40. The regression failed with the old parser, then passed after matching only summary lines at line start in multiline mode. Floors are now 29 Debug, 29 Release and 15 doctests, with zero ignored tests.

**Rule:** keep acceptance thresholds aligned with the actual exercise set, and verify the evidence parser against command echoes and missing suites. A gate's own regression must prove it rejects the old incomplete batch, not merely compare constants. Exact final CI and artifact checks still decide publication.
