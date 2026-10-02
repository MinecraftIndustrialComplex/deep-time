#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P2-01: how dense a planet's structures are, per `p2-01-planet.sh world` run.

    python3 tools/review/p2-01-dense.py <runs-folder> <name>...

One line a run: events, the lattice's candidates and segments, the Noddy check's longest history
and how many sampled columns needed a chain of histories, and the strata-check's structure census.
"""
import json
import sys
from pathlib import Path

runs = Path(sys.argv[1])
for name in sys.argv[2:]:
    m = json.loads((runs / f"{name}-world" / "manifest.json").read_text())
    note = m["notes"]["structure"]
    note = json.loads(note) if isinstance(note, str) else note
    d = note["deformation"]
    bake = m["notes"]["bake"]
    bake = json.loads(bake) if isinstance(bake, str) else bake
    lat, chk = d["lattice"], d["noddy_check"]
    s = json.loads((runs / f"{name}-strata.json").read_text())
    st = s["structure"]
    print(json.dumps({
        "run": name,
        "events": bake["structure"]["events"],
        "structures": d["structures"],
        "mean_candidates": round(lat["mean_candidates"], 2),
        "max_candidates": lat["max_candidates"],
        "max_segments": lat["max_segments"],
        "max_shift_blocks": round(lat["max_shift_blocks"], 1),
        "max_heave_blocks": round(lat["max_heave_blocks"], 1),
        "margin_blocks": lat["margin_blocks"],
        "noddy": {k: chk[k] for k in ["columns", "columns_with_events", "points", "worst", "side_flips",
                                      "max_history", "long_histories"]},
        "strata_check": {k: st[k] for k in ["changed_share", "land_changed_share", "malformed_columns",
                                            "bed_offset_blocks", "bed_repeated_columns", "intruded_columns"]},
        "findings_share": s["columns"]["share"],
    }))
