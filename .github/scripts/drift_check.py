#!/usr/bin/env python3
"""Compare the Rust cs-timespan implementation against the upstream C# TimeSpan
source/tests and file a GitHub issue for each behavioral gap found.

Requires ANTHROPIC_API_KEY and GH_TOKEN in the environment, and the gh CLI.
"""

import json
import os
import subprocess
import sys
import urllib.request

REPO = "chesedo/timespan-rs"
LABEL = "csharp-drift"
MODEL = "claude-sonnet-4-6"
MAX_ISSUES_PER_RUN = 15

RUST_FILES = ["src/lib.rs", "src/parse.rs", "src/fmt.rs"]

CSHARP_SOURCES = {
    "TimeSpan.cs": "https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Private.CoreLib/src/System/TimeSpan.cs",
    "TimeSpanParse.cs": "https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Private.CoreLib/src/System/TimeSpanParse.cs",
    "TimeSpanFormat.cs": "https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Private.CoreLib/src/System/TimeSpanFormat.cs",
    "TimeSpanTests.cs": "https://raw.githubusercontent.com/dotnet/runtime/main/src/libraries/System.Runtime/tests/System.Runtime.Tests/System/TimeSpanTests.cs",
}

SYSTEM_PROMPT = """\
You audit a Rust crate (cs-timespan) that intentionally replicates the exact \
parsing/formatting/arithmetic behavior of C# System.TimeSpan, for migration/interop \
use. You are given the current Rust source and the current upstream C# TimeSpan \
source and tests. Find concrete behavioral gaps: methods, constants, format \
specifiers, parse formats, or edge-case behaviors present in C# but missing or \
differently-behaved in Rust.

Ignore naming/style differences, internal implementation details, and anything \
that is a deliberate, reasonable design choice for a Rust port (e.g. error types, \
ownership, idiomatic Rust API shape). Only report things that would cause a Rust \
caller to get a different externally-observable result than the equivalent C# call.

Respond with ONLY a JSON array, no prose, no markdown fences. Each element:
{"title": "<short imperative title, under 80 chars>",
 "body": "<markdown explanation citing the relevant C# snippet and what's missing \
or different in Rust>"}
Return an empty array [] if you find nothing.
"""


def fetch(url: str) -> str:
    with urllib.request.urlopen(url, timeout=30) as resp:
        return resp.read().decode("utf-8")


def read_local(path: str) -> str:
    with open(path, encoding="utf-8") as f:
        return f.read()


def existing_drift_titles() -> set[str]:
    out = subprocess.run(
        [
            "gh", "issue", "list", "--repo", REPO, "--label", LABEL,
            "--state", "all", "--limit", "200", "--json", "title",
        ],
        capture_output=True, text=True, check=True,
    )
    return {item["title"] for item in json.loads(out.stdout)}


def call_claude(user_prompt: str) -> str:
    body = json.dumps({
        "model": MODEL,
        "max_tokens": 8000,
        "system": SYSTEM_PROMPT,
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
    return "".join(block["text"] for block in data["content"] if block["type"] == "text")


def main() -> None:
    rust_blob = "\n\n".join(
        f"// ===== {path} =====\n{read_local(path)}" for path in RUST_FILES
    )
    csharp_blob = "\n\n".join(
        f"// ===== {name} =====\n{fetch(url)}" for name, url in CSHARP_SOURCES.items()
    )
    known = existing_drift_titles()

    user_prompt = (
        f"Already-filed gaps (do not repeat these, by title):\n"
        f"{json.dumps(sorted(known))}\n\n"
        f"--- RUST SOURCE ---\n{rust_blob}\n\n"
        f"--- C# SOURCE/TESTS ---\n{csharp_blob}"
    )

    raw = call_claude(user_prompt).strip()
    try:
        gaps = json.loads(raw)
    except json.JSONDecodeError:
        print("Claude did not return valid JSON:", file=sys.stderr)
        print(raw, file=sys.stderr)
        sys.exit(1)

    if not gaps:
        print("No drift found.")
        return

    if len(gaps) > MAX_ISSUES_PER_RUN:
        print(
            f"Found {len(gaps)} gaps, capping at {MAX_ISSUES_PER_RUN} for this run.",
            file=sys.stderr,
        )
        gaps = gaps[:MAX_ISSUES_PER_RUN]

    for gap in gaps:
        title = gap["title"].strip()
        if title in known:
            continue
        body = gap["body"] + "\n\n---\n_Filed automatically by the C# drift-check workflow._"
        subprocess.run(
            [
                "gh", "issue", "create", "--repo", REPO, "--label", LABEL,
                "--title", title, "--body", body,
            ],
            check=True,
        )
        print(f"Filed: {title}")


if __name__ == "__main__":
    main()
