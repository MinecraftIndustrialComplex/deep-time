#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The cone budget's books per run (no simulation): cone_books.py <run> ...

Per run, km^3 Earth-normalized over the run (not a year): what the hotspots' cones handed (the
volcano stage's `volcanoes/hotspot_cones_handed_km3`), what the tails netted by id
(`plates/hotspot_cones_by_id_km3`), what dead tails left unnetted
(`plates/hotspot_cones_unnetted_km3`), the rest (handed less netted less unnetted: what the live
hotspots' records still hold at the end, and the last frame's), and the tails' rock.
"""
import json
import math
import os
import sys


def g(run):
    with open(os.path.join(run, "fields", "globals.json")) as f:
        d = json.load(f)
    return d.get("globals", d)


rows = []
for run in sys.argv[1:]:
    x = g(run)
    get = lambda k: x.get(k, math.nan)
    handed = get("volcanoes/hotspot_cones_handed_km3")
    by_id = get("plates/hotspot_cones_by_id_km3")
    un = get("plates/hotspot_cones_unnetted_km3")
    rock = get("plates/hotspot_tail_rock_km3")
    built = get("volcanoes/hotspot_cones_built_km3")
    kernel = get("plates/hotspot_cones_km3")
    rest = handed - by_id - un
    rows.append((os.path.basename(run.rstrip("/")), handed, by_id, un, rest, rock, built, kernel))
print(f"{'run':24s} {'handed':>10s} {'by id':>10s} {'unnetted':>10s} {'rest':>10s} {'rest/handed':>11s} {'by id/rock':>10s} {'built':>10s} {'kernel':>8s}")
for r in rows:
    name, handed, by_id, un, rest, rock, built, kernel = r
    print(f"{name:24s} {handed:10.1f} {by_id:10.1f} {un:10.2f} {rest:10.2f} {rest / handed:11.5f} {by_id / rock:10.4f} {built:10.1f} {kernel:8.3f}")
n = len(rows)
mean = lambda k: sum(r[k] for r in rows) / n
print(f"{'mean of ' + str(n):24s} {mean(1):10.1f} {mean(2):10.1f} {mean(3):10.2f} {mean(4):10.2f} {mean(4) / mean(1):11.5f} {mean(2) / mean(5):10.4f} {mean(6):10.1f} {mean(7):8.3f}")
print("rest range", min(r[4] for r in rows), max(r[4] for r in rows))
print("unnetted range", min(r[3] for r in rows), max(r[3] for r in rows))
print("by id / rock range", min(r[2] / r[5] for r in rows), max(r[2] / r[5] for r in rows))
