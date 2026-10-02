#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Recount the claims ledger's status table from the ledger files.

    tools/review/claims_counts.py           print the counts table
    tools/review/claims_counts.py --write   replace the table rows in docs/science/claims.md

Several agents edit ledger files at once and each used to hand-edit the totals in claims.md, so the
totals conflicted at every merge. The ledger files are the truth: each row is a table line starting
`| PREFIX-NNN |` whose last cell holds its status marks. The first primary mark found is the row's
status; `pending-rewrite` and `not-yet-coded` are counted as flags beside it; `merged` rows are
counted apart and not in the totals. --write keeps each ledger's Subject cell from the existing
table and rewrites the numbers, including the **all** row. The prose below the table is not touched.
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LEDGER = os.path.join(ROOT, "docs", "science", "ledger")
CLAIMS = os.path.join(ROOT, "docs", "science", "claims.md")
PRIMARY = ["verified", "cited-unverified", "from-memory", "abstract-only", "calibrated", "game-choice", "uncited"]
FLAGS = ["pending-rewrite", "not-yet-coded"]
ROW = re.compile(r"^\|\s*([A-Z]{3})-\d+\s*\|")


def status_cell(line):
    # Split on unescaped pipes; the status is the last non-empty cell.
    cells = [c.strip() for c in re.split(r"(?<!\\)\|", line)]
    cells = [c for c in cells if c]
    return cells[-1] if cells else ""


def count():
    out = {}
    for name in sorted(os.listdir(LEDGER)):
        if not name.endswith(".md"):
            continue
        prefix = name[:-3]
        c = {k: 0 for k in PRIMARY + FLAGS + ["merged", "rows"]}
        seen = set()
        with open(os.path.join(LEDGER, name), encoding="utf-8") as f:
            for line in f:
                m = ROW.match(line)
                if not m or m.group(1) != prefix:
                    continue
                # Two branches adding rows at once have taken the same next id; refuse to count until
                # one is renumbered (and its `// ledger:` comments with it).
                rid = line.split("|")[1].strip()
                if rid in seen:
                    sys.exit(f"claims_counts: {rid} appears twice in ledger/{name}: renumber one")
                seen.add(rid)
                cell = status_cell(line)
                marks = re.findall(r"`([a-z-]+)`", cell) or re.findall(r"[a-z]+(?:-[a-z]+)*", cell)
                if "merged" in marks:
                    c["merged"] += 1
                    continue
                c["rows"] += 1
                primary = next((m for m in marks if m in PRIMARY), None)
                if primary is None:
                    print(f"claims_counts: {prefix}: no status in {line[:60]!r}", file=sys.stderr)
                else:
                    c[primary] += 1
                for fl in FLAGS:
                    if fl in marks:
                        c[fl] += 1
        out[prefix] = c
    return out


def fmt(n):
    return f"{n:,}"


def main():
    counts = count()
    cols = ["rows"] + PRIMARY + FLAGS + ["merged"]
    total = {k: sum(c[k] for c in counts.values()) for k in cols}
    text = open(CLAIMS, encoding="utf-8").read()
    subjects = dict(re.findall(r"^\| \[([A-Z]{3})\]\(ledger/[A-Z]{3}\.md\) \| ([^|]*?) \|", text, re.M))
    lines = []
    for p, c in counts.items():
        lines.append(f"| [{p}](ledger/{p}.md) | {subjects.get(p, '')} | " + " | ".join(str(c[k]) for k in cols) + " |")
    lines.append("| **all** | | " + " | ".join(f"**{fmt(total[k])}**" for k in cols) + " |")
    if "--write" not in sys.argv:
        print("\n".join(lines))
        return
    body = re.compile(r"(?:^\| \[[A-Z]{3}\]\(ledger/[A-Z]{3}\.md\) \|[^\n]*\n)+^\| \*\*all\*\* \|[^\n]*\n", re.M)
    if len(body.findall(text)) != 1:
        sys.exit("claims_counts: could not find exactly one counts table in claims.md")
    text = body.sub(lambda _: "\n".join(lines) + "\n", text)
    open(CLAIMS, "w", encoding="utf-8").write(text)
    print(f"claims_counts: {fmt(total['rows'])} active rows, {total['verified']} verified, {total['merged']} merged")


if __name__ == "__main__":
    main()
