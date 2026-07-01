# Agent instructions

## Publishing status

This crate has **not been published** to crates.io and has no released versions.
Breaking API changes are acceptable — do not add backwards-compatibility shims or
deprecation layers. Just change the API and update all call sites.

That said, the priority is always matching C# `System.TimeSpan` behavior exactly.
If closing a gap requires a breaking change, make it — don't preserve a
C#-incompatible API just for stability. Don't bump the crate version for this —
version stays at `0.1.0` until the crate is actually published; only bump it
once it is.

## C# source citations

When implementing a fix that mirrors specific C# `TimeSpan` behavior, cite the
upstream source file and line number(s), when possible (e.g. `// TimeSpan.cs#L338`).
This makes it fast to re-verify the citation later if upstream changes, without
re-deriving which lines the Rust code was based on.

Prefer placing the citation in the test that exercises the behavior rather than
inline in the implementation — put it directly above the test function itself
(not scattered inside the test body). Only cite in `src/*.rs` when no test
covers the specific behavior being mirrored.

The same applies when a Rust test is duplicating a specific C# test case (e.g.
from `TimeSpanTests.cs`): cite the file and line(s) of the test being mirrored,
directly above the test function, so it's clear which upstream case is being
reproduced and easy to check if that case changes.

## Nix

The flake (`flake.nix`) is the source of truth for checks — it's what CI
(`.github/workflows/ci.yml`) runs via `nix flake check`, covering `cargo fmt
--check`, `cargo clippy --all-features -- -D warnings`, and `cargo test
--all-features`.

- Run the full check suite before opening a PR: `nix flake check`
- For a faster inner loop while iterating, `cargo fmt` / `cargo clippy
  --all-features` / `cargo test --all-features` directly are fine, but always
  confirm with `nix flake check` before considering work done — it's the exact
  gate CI enforces, including `--all-features`, which plain `cargo clippy`/`cargo
  test` skip by default (missing it can hide `chrono`-feature-gated issues).
- `nix develop` drops into a devShell with the pinned Rust toolchain if `cargo`
  isn't otherwise available.

## Corrections

Whenever the user corrects an approach or decision, consider whether it should be
recorded here so the same mistake isn't repeated in future sessions.

If the correction is about something a skill (`.claude/skills/*/SKILL.md`) did
or instructed, ask the user whether the skill file itself should be updated to
reflect the correction, so the same mistake isn't repeated the next time that
skill runs.
