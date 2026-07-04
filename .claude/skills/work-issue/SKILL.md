---
name: work-issue
description: Work a single GitHub issue on chesedo/cs-timespan end-to-end — verify it's real, ask whether to fix it or mark it out of scope, then open a branch + PR for whichever path is chosen. For issues filed by the C# drift-check scanner that turn out to be out of scope, the PR updates the scanner's ignore list instead of the Rust code.
disable-model-invocation: false
allowed-tools: Bash, Read, Edit, Write, Grep, Glob, AskUserQuestion
user-invocable: true
---

# Work Issue

Process exactly one GitHub issue at a time on `chesedo/cs-timespan`. Never batch
multiple issues in one invocation — if the user wants several done, they'll
invoke this once per issue.

## Usage

```
/work-issue <issue-number>
```

## Step 1 — Fetch the issue

```bash
gh issue view <number> --repo chesedo/cs-timespan --json number,title,body,labels,state,url
```

If `state` is not `OPEN`, tell the user and stop — nothing to do.

Note whether the issue carries the `csharp-drift` label. That label means it was
filed automatically by `.github/scripts/drift_check.py` and determines the
out-of-scope path in Step 3.

## Step 2 — Verify the issue is real

Do not trust the issue body at face value — it may describe a gap that's
already been fixed since filing, or (for `csharp-drift` issues) something the
scanner hallucinated. Check, in this order:

1. **Read the current Rust source** the issue references (`src/lib.rs`,
   `src/parse.rs`, `src/fmt.rs`) — grep for the relevant method/constant/format
   specifier. Confirm the gap still exists as described.
2. **If the issue cites specific C# behavior**, fetch the current upstream file
   from `dotnet/runtime` to confirm the citation is still accurate (upstream
   can change). The URLs are in `CSHARP_SOURCES` in `drift_check.py`.
3. **If feasible, prove it empirically** — write a quick throwaway test (or use
   `cargo run --example` / a scratch `#[test]`) that exercises the claimed
   C# behavior against the current Rust code, so you're reporting an observed
   result, not a restated claim.

Classify the issue as one of:
- **CONFIRMED** — gap is real and still present.
- **STALE** — already fixed on `main` since the issue was filed.
- **QUESTIONABLE** — the claim is debatable (e.g. arguably a reasonable Rust-idiomatic
  design choice, or the C# behavior itself is a quirk not worth replicating).
- **FALSE** — the citation doesn't hold up (upstream doesn't say what the issue
  claims, or Rust already handles it correctly).

Report this classification and your reasoning to the user before asking anything.

## Step 3 — Ask what to do

Use `AskUserQuestion` (don't just assume). Offer:
- **Fix it** — only sensible for CONFIRMED issues (steer the user away from this
  for STALE/FALSE, and flag QUESTIONABLE as needing their judgment call).
- **Mark out of scope** — close it without a code fix. Sensible for STALE (no
  scanner change needed, just close), or QUESTIONABLE/FALSE (close AND update
  the scanner, see below).
- **Skip for now** — do nothing further, leave the issue open.

If the user picks "skip," stop here.

## Step 4a — Fix path

1. Make sure `main` is up to date: `git checkout main && git pull`.
2. Branch: `git checkout -b issue-<number>-<short-slug>`.
3. Write the regression test(s) first, against the *unfixed* code. Run just
   that test and confirm it actually fails (or panics) for the reason the
   issue describes — a test that passes vacuously against buggy code isn't
   proving anything. Commit the failing test on its own, e.g.
   `test: add failing regression test for #<number>`.
4. Implement the fix, following this repo's conventions (raw strings
   `r#"..."#` for multiline expected values; no comments unless they explain
   a non-obvious *why*; this crate is unpublished, so breaking API changes
   are fine — don't add compat shims).
5. Run `cargo fmt` (quick local pass), then `nix flake check` — this is what CI
   runs (fmt, clippy with `--all-features -D warnings`, and test with
   `--all-features`), so it must pass clean before proceeding. Confirm the
   regression test now passes.
6. Commit the fix separately from the test commit. Reference the issue in the
   PR body, not the commit subject.
7. Push the branch and open the PR:
   ```bash
   gh pr create --repo chesedo/cs-timespan --title "<title>" --body "$(cat <<'EOF'
   ## Summary
   <what changed and why, tying back to the C# behavior being matched>

   Closes #<number>

   ## Test plan
   - [ ] nix flake check
   EOF
   )"
   ```
8. Report the PR URL to the user.

## Step 4b — Out-of-scope path

**If the issue does NOT have the `csharp-drift` label** (manually filed, not
from the scanner): just confirm with the user, then
`gh issue close <number> --repo chesedo/cs-timespan --comment "<reason>"`. No
scanner change applies — stop here.

**If the issue DOES have the `csharp-drift` label**, closing it alone isn't
enough — the scanner will just re-file an equivalent issue next run. Instead,
close the loop by teaching the scanner:

1. `git checkout main && git pull`.
2. Branch: `git checkout -b drift-ignore-<number>-<short-slug>`.
3. Edit `.github/scripts/drift_ignore.md`: append a new `### <short label>`
   entry describing the *category* of gap (not just this one issue's title) and
   a one-paragraph reason it's not worth flagging. Be specific enough that the
   scanner (an LLM reading this file) can recognize similar-but-differently-worded
   future gaps in the same category.
4. Sanity check: `python3 -m py_compile .github/scripts/drift_check.py`.
5. Commit, push, and open a PR:
   ```bash
   gh pr create --repo chesedo/cs-timespan --title "chore(drift): stop flagging <category>" --body "$(cat <<'EOF'
   ## Summary
   Issue #<number> was reviewed and judged out of scope: <one-line reason>.
   Adds an entry to drift_ignore.md so the scanner stops re-filing this class
   of gap.

   Closes #<number>

   ## Test plan
   - [ ] python3 -m py_compile .github/scripts/drift_check.py
   EOF
   )"
   ```
6. Report the PR URL to the user.

## Step 5 — Address PR review comments

Once a PR is open (from either 4a or 4b), it will typically pick up review
comments — from the Copilot PR reviewer bot, and/or from the repo owner
reviewing directly on GitHub. Don't wait to be asked; check for these and work
through them as part of finishing the issue:

```bash
gh pr view <pr-number> --repo chesedo/cs-timespan --json comments,reviews
gh api repos/chesedo/cs-timespan/pulls/<pr-number>/comments
```

Every comment gets verified before you act on it, regardless of who posted
it — bot or repo owner. Read the actual code it refers to, and if it cites
C# behavior, check that against current upstream rather than trusting the
claim at face value. Being from the owner doesn't exempt a comment from this
check — you can and should disagree with the owner too, the same as with the
bot, if the code/upstream source says otherwise.

Then, per comment:

1. **If you agree**, fix it directly (edit, `cargo fmt`, `nix flake check`,
   commit, push to the same branch), then reply on that specific comment
   thread summarizing what changed:
   ```bash
   gh api repos/chesedo/cs-timespan/pulls/<pr-number>/comments/<comment-id>/replies -f body="<reply>"
   ```
2. **If you disagree**, do not post anything yet — draft the reply text and
   use `AskUserQuestion` to get the user's approval before posting it (or
   before making no change). Only call the `replies` API after they confirm.

## Notes

- Never push directly to `main` or merge the PR yourself — the PR is the
  deliverable; the user merges it.
- If `nix flake check` fails after a fix attempt and you can't resolve it, say
  so plainly rather than opening a broken PR.
- One issue, one branch, one PR. Don't fold unrelated cleanup into the same PR.
