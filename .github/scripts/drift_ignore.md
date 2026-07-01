<!--
Accepted-as-intentional divergences from C# TimeSpan.

Each entry below was reviewed by a human and judged NOT a real gap (deliberate
design choice, C# quirk not worth replicating, or a duplicate/near-duplicate of
something already tracked). The drift-check scanner is told to skip anything
matching these categories, so add an entry here whenever an issue filed by the
scanner turns out to be out of scope, instead of just closing the issue.

Format: one "### <short label>" heading per entry, with a one-paragraph reason.
-->

### Misread `TimeSpan(days, hours, minutes, seconds)` constructor argument order

Watch for gaps whose "expected" value comes from a C# test data row that
constructs a `TimeSpan` via the 4-arg constructor, e.g.
`new TimeSpan(24, 0, 0, 0)`, and whose write-up glosses that value in prose as
"24 hours" or otherwise slides the leading integer into the wrong field. The
constructor signature is `TimeSpan(int days, int hours, int minutes, int
seconds)` — the first argument is always **days**, not hours — so
`new TimeSpan(24, 0, 0, 0)` is 24 days, not "24 hours promoted to 1 day."
Before filing, re-derive the expected value directly from the constructor
argument positions (or the literal tick count already given alongside it in
the test data) rather than trusting an inline English paraphrase of what the
constructor call "means." Issue #9 was filed this way: it claimed lenient
parsing of `"24:00:00"` should overflow-promote to 1 day, but the cited
`new TimeSpan(24, 0, 0, 0)` is actually 24 days, and Rust's
`TimeSpan::parse("24:00:00")` already returns `24 * TICKS_PER_DAY`
(20736000000000), matching C# tick-for-tick.
