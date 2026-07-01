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

Issue #8 claimed `parse_g` mishandles `"g"`-format inputs where the seconds
component is omitted but a fractional part is present (`"H:MM:.FF"` or
`"D:H:MM:.FF"`), citing `TimeSpanTests.cs` rows for `"1:2:.3"` →
`TimeSpan(0,1,2,0,300)` and `"1:2:3:.4"` → `TimeSpan(1,2,3,0,400)`. The issue's
own body traces the code and concludes both cases "actually give the right
answer," yet still filed a bug — a self-contradictory finding. Both cases are
already covered by passing tests (`tests/parse_exact.rs`:
`parse_exact_g_hm_with_fraction` and `parse_exact_g_d_hm_with_fraction`) pinned
to the exact cited `TimeSpanTests.cs` rows, and manual tracing of `parse_g`
confirms the empty `sec_s` combined with a `Some` `frac_s` is handled
correctly in both the two-colon and three-colon forms. Do not re-flag "g"/"G"
parsing of empty seconds with a trailing fraction unless a *new*, concretely
failing input is identified (not a restated trace of the existing cases).
