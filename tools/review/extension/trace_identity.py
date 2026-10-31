#!/usr/bin/env python3
"""The rift trace's runs against ext-cum2's own runs of the same seeds: summary.txt and globals.json equal?
# SPDX-License-Identifier: LGPL-3.0-or-later

Run on the VM: python3 trace_identity.py
"""
import hashlib
import json
from pathlib import Path

POOL = Path("/data/ban/deeptime-runs/pool")
TRACE = POOL / "e5329a3bdd42/earthlike_quick_16k-1000/7050a4c4"
CUM2 = POOL / "b6c066edc137/earthlike_quick_16k-1000/0efeeddf"


def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()[:16] if p.exists() else "missing"


for run in ["scattered_continents-1", "scattered_continents-2", "supercontinent-1", "supercontinent-2"]:
    a, b = TRACE / run, CUM2 / run
    out = []
    for f in ["summary.txt", "fields/globals.json"]:
        da, db = digest(a / f), digest(b / f)
        out.append(f"{f} {'same' if da == db else 'DIFFERENT'} ({da} {db})")
    ga, gb = a / "fields/globals.json", b / "fields/globals.json"
    if ga.exists() and gb.exists():
        ja, jb = json.loads(ga.read_text()), json.loads(gb.read_text())
        diff = [k for k in set(ja) | set(jb) if ja.get(k) != jb.get(k)]
        out.append(f"globals differing keys {len(diff)}: {sorted(diff)[:8]}")
    rifts = a / "rifts"
    files = sorted(p.name + ":" + str(p.stat().st_size) for p in rifts.iterdir()) if rifts.exists() else []
    print(run, "|", "; ".join(out), "| trace files", files)
