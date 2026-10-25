#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""polarity.py <arm folder>: for plates born by subduction initiation, the recorded contacts between the
split and its parent (rec.json `partings` and `collisions`, the only contact records with under/over) and
whether the split is the one going under (as the planted slab, PLT-270) or the parent is.

Prints pooled counts and the reasons the partings give; a contact that never parted or collided is not
in the record, so this is a sample of the pairs' contacts, not all of them."""
import collections
import json
import sys
from pathlib import Path

arm = Path(sys.argv[1])
tot = collections.Counter()
reasons = collections.Counter()
lag = {"agree": [], "reverse": []}
origins = collections.Counter()
for run in sorted(p for p in arm.iterdir() if (p / "rec.json").is_file()):
    r = json.loads((run / "rec.json").read_text())
    plates = {p["id"]: p for p in r["plates"]}
    for p in r["plates"]:
        origins[p.get("origin")] += 1
    for kind in ("partings", "collisions"):
        for c in r.get(kind) or []:
            u, o = c["under"], c["over"]
            pu, po = plates.get(u), plates.get(o)
            for child, par, agree in ((pu, po, True), (po, pu, False)):
                if child is None or par is None:
                    continue
                if child.get("origin") != "subduction_initiation" or child.get("parent") != par["id"]:
                    continue
                key = "agree" if agree else "reverse"
                onset = c.get("contact_onset_myr")
                tot[(kind, key)] += 1
                if kind == "partings":
                    reasons[(key, c.get("reason"))] += 1
                if onset is not None:
                    lag[key].append(onset - child["born_myr"])
print("origins:", dict(origins))
print("pairs (split, parent) in the record:", dict(tot))
print("parting reasons:", dict(reasons))
for k, v in lag.items():
    v = sorted(v)
    if v:
        print(f"{k}: n {len(v)} onset after birth p10 {v[len(v) // 10]:.1f} p50 {v[len(v) // 2]:.1f} p90 {v[9 * len(v) // 10]:.1f} Myr")
