# D6 practical reference evidence · Linux · 2026-11-18

Historical rc.2 evidence; the original reports and logs below are preserved. Current rc.3 command evidence and retained IDE entry snapshots are in [rc3-core-concepts](../rc3-core-concepts/README.md).

These are actual local command runs for the D6 tools, Edition migration and optional Nightly experiments. They bind the runner's SHA-256 and the explicitly listed input snapshot. They are not a full-release source fingerprint, Windows validation, IDE interaction record or learner completion record. The local reports have no GitHub workflow commit; final cross-platform acceptance comes from the exact PR/main run.

- Tools: Rust 1.85.0 with rustfmt, Clippy, rust-docs and rust-analyzer; 15 steps, Debug 11 / doc 3 / Release 11.
- Edition: Rust 1.85.0; 14 steps, old Edition baseline 11, migration probe 12, migrated Debug 12 / doc 3 / Release 12. The gen definition and call are actually rewritten to r#gen.
- Nightly: nightly-2026-11-17; 6 steps, one test passes, Stable 1.85 rejects the feature gate with E0554 and rejects the expression again without the gate.

All three reports have PASS and unchanged original-source snapshots. Command logs, before/after source and manifest, and differences are retained. Successful temporary workspaces were removed only after evidence was saved. Each report lists evidence hashes; verify those bytes before relying on a copied report.

Reproduce from the repository root with the components installed as documented in course/BOOK_D6_PRACTICAL.md. Use a new output directory each time:

    python scripts/book_d6_check.py --mode tools --toolchain 1.85.0 --output <new-tools-directory>
    python scripts/book_d6_check.py --mode edition --toolchain 1.85.0 --output <new-edition-directory>
    python scripts/book_d6_check.py --mode nightly --toolchain nightly-2026-11-17 --stable-toolchain 1.85.0 --output <new-nightly-directory>

Each mode's report.json contains argv, exit codes, log names and digests. Nightly remains opt-in and cannot replace the stable/MSRV gates.
