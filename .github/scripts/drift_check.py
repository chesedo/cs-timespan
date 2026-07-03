#!/usr/bin/env python3
"""Compare the Rust cs-timespan implementation against the upstream C# TimeSpan
source/tests and file a GitHub issue for each behavioral gap found.

Requires ANTHROPIC_API_KEY and GH_TOKEN in the environment, and the gh CLI.

Pass --dry-run to run the full scan/verify pipeline without filing anything —
confirmed gaps are printed instead of created as GitHub issues.
"""

import json
import os
import subprocess
import sys
import urllib.request

REPO = "chesedo/cs-timespan"
LABEL = "csharp-drift"
MODEL = "claude-sonnet-5"
MAX_ISSUES_PER_RUN = 15
DRY_RUN = "--dry-run" in sys.argv

RUST_FILES = ["src/lib.rs", "src/parse.rs", "src/fmt.rs"]
IGNORE_FILE = os.path.join(os.path.dirname(__file__), "drift_ignore.md")

CSHARP_SOURCES = {
    "TimeSpan.cs": "https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Private.CoreLib/src/System/TimeSpan.cs",
    "TimeSpanParse.cs": "https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Private.CoreLib/src/System/Globalization/TimeSpanParse.cs",
    "TimeSpanFormat.cs": "https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Private.CoreLib/src/System/Globalization/TimeSpanFormat.cs",
    "TimeSpanTests.cs": "https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Runtime/tests/System.Runtime.Tests/System/TimeSpanTests.cs",
}

SCAN_SYSTEM_PROMPT = """\
You audit a Rust crate (cs-timespan) that intentionally replicates the exact \
parsing/formatting/arithmetic behavior of C# System.TimeSpan, for migration/interop \
use. You are given the current Rust source and the current upstream C# TimeSpan \
source and tests.

List CANDIDATE behavioral gaps worth investigating further: methods, constants, \
format specifiers, parse formats, or edge-case behaviors present in C# that might \
be missing or differently-behaved in Rust. You do NOT need to fully verify each \
one here — a separate, isolated pass will independently confirm or reject every \
candidate you list before anything is filed, so err on the side of listing a \
candidate if you're unsure rather than trying to fully trace it yourself here.

Ignore naming/style differences, internal implementation details, and anything \
that is a deliberate, reasonable design choice for a Rust port (e.g. error types, \
ownership, idiomatic Rust API shape). Only list things that could plausibly cause \
a Rust caller to get a different externally-observable result than the equivalent \
C# call.

Respond with ONLY a JSON array, no prose, no markdown fences. Each element:
{"title": "<short imperative title, under 80 chars>",
 "hint": "<1-3 sentences pointing at the specific C# behavior/citation and the \
Rust code location to compare it against>"}
Return an empty array [] if you find nothing plausible.
"""

VERIFY_SYSTEM_PROMPT = """\
You are independently verifying ONE candidate behavioral gap between a Rust crate \
(cs-timespan, which intentionally replicates C# System.TimeSpan exactly) and the \
upstream C# TimeSpan implementation. You have not seen any other candidates from \
this run, and should not assume the candidate's premise is correct — do your own \
trace of the specific behavior it describes.

Determine whether Rust's current behavior actually diverges from C#'s for this \
specific case. You must be able to state a concrete input where Rust's observable \
result differs from C#'s to confirm a gap. If your own trace concludes Rust's \
behavior already matches C# (even if the candidate's title claims otherwise), you \
MUST reject it — a suggestive title is never sufficient on its own.

Also reject the candidate if it matches an already-accepted-as-intentional \
divergence listed below, even under different wording.

Respond with ONLY a JSON object, no prose, no markdown fences:
- Confirmed: {"confirmed": true, "title": "<short imperative title, under 80 chars>", \
"body": "<markdown explanation citing the relevant C# snippet and the specific \
input/expected/actual values that differ>"}
- Rejected: {"confirmed": false, "reason": "<one sentence why>"}
"""


def fetch(url: str) -> str:
    print(f"Fetching {url} ...", file=sys.stderr)
    with urllib.request.urlopen(url, timeout=30) as resp:
        return resp.read().decode("utf-8")


def read_local(path: str) -> str:
    with open(path, encoding="utf-8") as f:
        return f.read()


def existing_drift_titles() -> set[str]:
    print("Fetching already-filed drift issues ...", file=sys.stderr)
    out = subprocess.run(
        [
            "gh", "issue", "list", "--repo", REPO, "--label", LABEL,
            "--state", "all", "--limit", "200", "--json", "title",
        ],
        capture_output=True, text=True, check=True,
    )
    return {item["title"] for item in json.loads(out.stdout)}


def call_claude(system_prompt: str, user_prompt: str) -> str:
    body = json.dumps({
        "model": MODEL,
        "max_tokens": 8000,
        "system": system_prompt,
        "messages": [{"role": "user", "content": user_prompt}],
    }).encode("utf-8")
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=body,
        headers={
            "content-type": "application/json",
            "x-api-key": os.environ["ANTHROPIC_API_KEY"],
            "anthropic-version": "2023-06-01",
        },
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.loads(resp.read().decode("utf-8"))
    if data.get("stop_reason") == "max_tokens":
        print(
            "Warning: response was cut off (hit max_tokens). The JSON below is "
            "likely incomplete — consider raising max_tokens in call_claude().",
            file=sys.stderr,
        )
    return "".join(block["text"] for block in data["content"] if block["type"] == "text")


def parse_json_response(raw: str, label: str) -> object:
    try:
        return json.loads(raw.strip())
    except json.JSONDecodeError:
        print(f"Claude did not return valid JSON ({label}). Raw response was:", file=sys.stderr)
        print("--- start raw response ---", file=sys.stderr)
        print(raw if raw.strip() else "(empty)", file=sys.stderr)
        print("--- end raw response ---", file=sys.stderr)
        sys.exit(1)


def fail_schema(label: str, data: object) -> None:
    print(f"Claude's response for {label} didn't match the expected schema:", file=sys.stderr)
    print(json.dumps(data), file=sys.stderr)
    sys.exit(1)


def scan_candidates(rust_blob: str, csharp_blob: str, known: set[str], ignore_notes: str) -> list[dict]:
    print("Running scan pass (calling Claude) ...", file=sys.stderr)
    user_prompt = (
        f"Already-filed gaps (do not repeat these, by title):\n"
        f"{json.dumps(sorted(known))}\n\n"
        f"Accepted-as-intentional divergences (do NOT list candidates matching "
        f"these, even under a different title):\n{ignore_notes}\n\n"
        f"--- RUST SOURCE ---\n{rust_blob}\n\n"
        f"--- C# SOURCE/TESTS ---\n{csharp_blob}"
    )
    raw = call_claude(SCAN_SYSTEM_PROMPT, user_prompt)
    data = parse_json_response(raw, "scan")
    if not isinstance(data, list) or not all(
        isinstance(item, dict) and isinstance(item.get("title"), str) for item in data
    ):
        fail_schema("scan (expected a JSON array of {'title': str, ...})", data)
    return data


def verify_candidate(
    candidate: dict, rust_blob: str, csharp_blob: str, ignore_notes: str
) -> dict:
    user_prompt = (
        f"Candidate to verify:\n"
        f"{json.dumps({'title': candidate['title'], 'hint': candidate.get('hint', '')})}\n\n"
        f"Accepted-as-intentional divergences (reject the candidate if it matches "
        f"one of these, even under different wording):\n{ignore_notes}\n\n"
        f"--- RUST SOURCE ---\n{rust_blob}\n\n"
        f"--- C# SOURCE/TESTS ---\n{csharp_blob}"
    )
    raw = call_claude(VERIFY_SYSTEM_PROMPT, user_prompt)
    label = f"verify: {candidate['title']}"
    data = parse_json_response(raw, label)
    if not isinstance(data, dict) or not isinstance(data.get("confirmed"), bool):
        fail_schema(f"{label} (expected an object with a boolean 'confirmed')", data)
    if data["confirmed"] and not (
        isinstance(data.get("title"), str) and isinstance(data.get("body"), str)
    ):
        fail_schema(f"{label} (confirmed=true requires string 'title'/'body')", data)
    return data


def main() -> None:
    print("Reading local Rust source ...", file=sys.stderr)
    rust_blob = "\n\n".join(
        f"// ===== {path} =====\n{read_local(path)}" for path in RUST_FILES
    )
    csharp_blob = "\n\n".join(
        f"// ===== {name} =====\n{fetch(url)}" for name, url in CSHARP_SOURCES.items()
    )
    known = existing_drift_titles()
    ignore_notes = read_local(IGNORE_FILE) if os.path.exists(IGNORE_FILE) else ""

    candidates = scan_candidates(rust_blob, csharp_blob, known, ignore_notes)
    if not candidates:
        print("No candidates found.")
        return
    print(f"Scan found {len(candidates)} candidate(s).", file=sys.stderr)

    filed = 0
    for i, candidate in enumerate(candidates, start=1):
        if filed >= MAX_ISSUES_PER_RUN:
            print(f"Filed {MAX_ISSUES_PER_RUN} issues, stopping for this run.", file=sys.stderr)
            break

        title = candidate["title"].strip()
        if title in known:
            continue

        print(f"[{i}/{len(candidates)}] {title}", file=sys.stderr)
        # Each candidate is verified in its own isolated call — no shared context
        # with the scan pass or with any other candidate — so an earlier finding
        # can't bias whether this one gets confirmed.
        verdict = verify_candidate(candidate, rust_blob, csharp_blob, ignore_notes)

        if not verdict.get("confirmed"):
            print(f"Rejected: {title} ({verdict.get('reason', 'no reason given')})")
            continue

        title = verdict["title"].strip()
        if title in known:
            continue
        body = verdict["body"] + "\n\n---\n_Filed automatically by the C# drift-check workflow._"

        if DRY_RUN:
            print(f"[dry run] Would file: {title}\n{body}\n")
        else:
            subprocess.run(
                [
                    "gh", "issue", "create", "--repo", REPO, "--label", LABEL,
                    "--title", title, "--body", body,
                ],
                check=True,
            )
            print(f"Filed: {title}")
        filed += 1

    if filed == 0:
        print("No drift confirmed after verification.")


if __name__ == "__main__":
    main()
