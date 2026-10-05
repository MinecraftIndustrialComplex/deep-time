#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Prints the geology gate's and the biome rules' census sections of `/deeptime probe biomes` biome maps.

    python3 tools/review/geology_census.py <name>-<profile>-biomemap.json ...
"""
import json
import sys


def find(o, key):
    if isinstance(o, dict):
        if key in o:
            return o[key]
        for v in o.values():
            r = find(v, key)
            if r is not None:
                return r
    return None


for path in sys.argv[1:]:
    d = json.load(open(path))
    print("==", path)
    g = find(d, "geology")
    if g is None:
        print("  no geology section")
    else:
        print("  gated:", g.get("gated_biomes"))
        print("  ground:", g.get("ground"))
        for side in ("plain", "now"):
            s = g[side]
            top = ", ".join(f"{k} {v['columns']}/{v['off_ground']}" for k, v in list(s["by_biome"].items())[:8])
            print(f"  {side}: columns {s['columns']} rock_named {s['rock_named']} off_ground {s['off_ground']} | {top}")
        print("  replaced_by:", g.get("replaced_by"), "holes", g.get("replaced_holes"), "changed", g.get("columns_changed"))
    r = find(d, "rules")
    if isinstance(r, dict):
        print("  rules:", {k: r[k] for k in r if k in ("columns", "rule_columns", "decisions", "exclusive_leaked", "not_candidate", "wrong_class")})
