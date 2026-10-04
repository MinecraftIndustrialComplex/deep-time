#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Print conformal.py's output compactly."""
import json
import sys

txt = open(sys.argv[1]).read()
dec = json.JSONDecoder()
i = 0
while True:
    j = txt.find("{", i)
    if j < 0:
        break
    o, e = dec.raw_decode(txt[j:])
    i = j + e
    for k, v in o.items():
        print(k, {a: (round(b, 4) if isinstance(b, float) else b) for a, b in v.items()
                  if a not in ("note", "worst_points", "worst_points_mp_bpa")})
        print("  worst", [[round(x, 4) for x in w] for w in v["worst_points"]])
