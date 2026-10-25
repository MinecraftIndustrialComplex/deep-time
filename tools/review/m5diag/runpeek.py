#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""runpeek.py <arm folder>: the runs of a pool arm, one run's files, its map times and its fields."""
import json
import sys
from pathlib import Path

d = Path(sys.argv[1])
runs = sorted(p.name for p in d.iterdir() if p.is_dir())
print("runs", runs)
r = d / runs[0]
print("files", sorted(p.name for p in r.iterdir()))
if (r / "maps").exists():
    print("maps", sorted(p.name for p in (r / "maps").iterdir()))
idx = json.loads((r / "fields" / "index.json").read_text())
print("fields", sorted(idx))
s = (r / "summary.txt").read_text().splitlines()
print("summary lines", len(s))
for line in s[:40]:
    print("  ", line[:240])
