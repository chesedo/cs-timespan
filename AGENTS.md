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

## Never fabricate data

If a fetch, read, or lookup fails or comes back empty (404, empty body,
timeout), say so and stop — never invent plausible-looking content to fill
the gap. This applies especially to C# source: don't reconstruct a function
from memory/guesswork and present it as fetched.

Same rule if the source IS reachable but something's off: missing code, or
C# behavior that contradicts its own docs. Say so explicitly instead of
picking one side quietly.

(A prior session's C# source fetch 404'd — wrong URL — got back an explicit
"response body was not retrieved," and fabricated a `TryTimeToTicks` function
body anyway, inventing a C# behavior that doesn't exist. That shipped as a
real bug plus two wrong tests, and survived a later citation-audit pass
uncaught. See commit `2165f39`.)

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

### Verify citations before trusting them

A citation is a claim, not a proof. Before relying on one — reading an
existing one or adding a new one — confirm both:
1. It's reachable: fetch the URL/file and line, don't assume it exists.
2. It supports the claim: the cited C# actually says what the comment/test
   claims, and the Rust code actually behaves that way. Prove it empirically
   where feasible (a throwaway test).

When auditing citations in bulk, have a subagent independently re-verify each
one against live upstream source rather than trusting an earlier summary —
a wrong claim, once written up, tends to get carried forward unchecked.

## Test coverage parity

When porting a C# test suite's data-driven test (`[Theory]`/`[MemberData]`/
`[InlineData]`), match **all** of the upstream cases at minimum — not just a
representative sample. This includes combinatorially-generated data sets (e.g.
a C# helper that loops over N values to yield every pairwise combination):
port every generated row, not a hand-picked subset, even if that means the
Rust test file ends up large. If a case is genuinely infeasible to port
(e.g. it depends on a C#-only construct with no Rust equivalent), state that
explicitly in a comment rather than silently omitting it.

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
