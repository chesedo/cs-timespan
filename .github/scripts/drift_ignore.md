<!--
Accepted-as-intentional divergences from C# TimeSpan.

Each entry below was reviewed by a human and judged NOT a real gap (deliberate
design choice, C# quirk not worth replicating, or a duplicate/near-duplicate of
something already tracked). The drift-check scanner is told to skip anything
matching these categories, so add an entry here whenever an issue filed by the
scanner turns out to be out of scope, instead of just closing the issue.

Format: one "### <short label>" heading per entry, with a one-paragraph reason.
-->

### `TimeSpan(days, hours, minutes, seconds)` constructor argument order

The 4-arg C# constructor is `TimeSpan(int days, int hours, int minutes, int
seconds)` — the first argument is **days**, not hours. When a cited test row
constructs the expected value this way, derive it from the argument positions
directly: `new TimeSpan(24, 0, 0, 0)` is 24 days, not "24 hours." Caused issue
#9.

### "g"/"G" parse: empty-seconds-with-fraction segments (e.g. "1:2:.3", "1:2:3:.4")

Already covered by passing tests `parse_exact_g_hm_with_fraction` and
`parse_exact_g_d_hm_with_fraction` in `tests/parse_exact.rs`, pinned to the
cited `TimeSpanTests.cs` rows. Don't re-flag this unless a new, concretely
failing input is identified. Caused issue #8.
