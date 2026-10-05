#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""List a bake's sites of one type (default porphyry) with their bodies, children and distance to spawn.

    p1-15-bodies.py <dump-deposits json> [<spawn x> <spawn z> [type-suffix]]
"""
import json
import math
import sys

d = json.load(open(sys.argv[1]))
c = d["circumference"]
sx = int(sys.argv[2]) if len(sys.argv) > 2 else 0
sz = int(sys.argv[3]) if len(sys.argv) > 3 else 0
suffix = sys.argv[4] if len(sys.argv) > 4 else "porphyry_cu_mo_au"
bodies = {}
for b in d["bodies"]:
    bj = b["body"]
    bodies.setdefault(bj.get("site"), []).append(bj)
sites = d["sites"]
for s in sites:
    if not s["deposit_type"].endswith(suffix):
        continue
    dx = (s["center"][0] - sx) % c
    dx = min(dx, c - dx)
    dist = math.hypot(dx, s["center"][2] - sz)
    print(f"site {s['id']} {s['deposit_type']} centre {s['center']} aabb {s['aabb']} {dist:.0f} from ({sx},{sz})")
    print("   products", [(p["material"], round(p["tonnage_t"] / 1e6, 2), p["role"]) for p in s["products"]])
    for b in bodies.get(s["id"], []):
        print("   body", {k: v for k, v in b.items() if not isinstance(v, (list, dict)) or k in ("aabb", "center")})
    for k in sites:
        if k.get("parent") == s["id"]:
            print("   child", k["id"], k["deposit_type"], k["center"], [(p["material"], round(p["tonnage_t"] / 1e6, 3)) for p in k["products"]])
