#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-27: the plate events of a run (a viewer bundle's events.json) as Wilson-cycle statistics:
subduction onsets by setting and slab age, how soon two plates that opened a ridge start
subducting one under the other (an ocean closing), collisions and reversal-born contacts.

    python3 tools/review/p1-27-events.py <events.json> [<events.json> ...]
"""
import json
import statistics
import sys


def stats(path):
    ev = json.load(open(path))
    kind = lambda e: e["kind"].split(":")[-1]
    p = lambda e, k: e.get("params", {}).get(k)
    t = lambda e: e["t_start"] / 1e6
    onsets = [e for e in ev if kind(e) == "subduction_onset"]
    ridges = [e for e in ev if kind(e) == "ridge_birth"]
    births = [e for e in ev if kind(e) == "plate_birth"]
    span = (max(t(e) for e in ev) - min(t(e) for e in ev)) or 1.0
    out = {"file": path, "span_myr": round(span)}
    by_setting = {}
    for e in onsets:
        by_setting.setdefault(p(e, "setting"), []).append(p(e, "slab_age_myr") or 0.0)
    out["onsets_per_100myr"] = round(100 * len(onsets) / span, 1)
    for s, ages in sorted(by_setting.items()):
        out[f"onsets_{s}"] = len(ages)
        out[f"slab_age_p50_{s}"] = round(statistics.median(ages), 1)
    # Ridge → later subduction between the same two plates: the ocean they opened closing.
    first_sub = {}
    for e in onsets:
        a, b = p(e, "subducting_plate"), p(e, "overriding_plate")
        first_sub.setdefault(frozenset((a, b)), []).append(t(e))
    delays, never = [], 0
    for r in ridges:
        pair = frozenset((p(r, "plate_a"), p(r, "plate_b")))
        later = [x for x in first_sub.get(pair, []) if x > t(r)]
        if later:
            delays.append(min(later) - t(r))
        else:
            never += 1
    out["ridges"] = len(ridges)
    out["ridge_pairs_later_subducting"] = len(delays)
    if delays:
        out["close_delay_p50_myr"] = round(statistics.median(delays), 1)
        out["close_within_50myr"] = sum(d <= 50 for d in delays)
    out["collisions_per_100myr"] = round(
        100 * sum(kind(e) == "collision" for e in ev) / span, 1)
    # Continental margin turned into a plate boundary, by what did it (planet km).
    split = {}
    for e in ev:
        km = p(e, "coast_split_km")
        if km:
            key = kind(e) if kind(e) != "plate_birth" else "birth_" + str(p(e, "origin"))
            split[key] = round(split.get(key, 0) + km)
    out["coast_split_km"] = split
    origins = {}
    for e in births:
        origins[p(e, "origin")] = origins.get(p(e, "origin"), 0) + 1
    out["births"] = origins
    return out


for f in sys.argv[1:]:
    print(json.dumps(stats(f)))
