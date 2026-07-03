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
constructs the expected value this way (e.g. `new TimeSpan(24, 0, 0, 0)`),
derive the expected value from the argument positions directly; don't trust a
prose paraphrase that slides the leading integer into the wrong field (e.g.
reading `new TimeSpan(24, 0, 0, 0)` as "24 hours"). Caused issue #9.
