#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""floorage_age_read.py <floorage_age.py output (JSON lines)>: the tables of where the floor of each age sits.

Per run and pooled over the runs (counts summed over the snapshots from -500 Myr on): the checks, then for each age
band the ocean's count and the share of it in each place, then each place's share of the ocean and of the floor older
than 180 Myr, and the initiation status of the floor older than each gate."""
import json
import sys

BANDS = ["0-10", "10-30", "30-60", "60-100", "100-150", "150-180", "180-250", "250+"]
PLACES = ["margin", "interior", "giant", "notrench", "nofront", "x_interior_rest", "x_neither", "x_giant_notrench",
          "x_margin_notrench"]
GATES = ["150", "117.4"]


def pooled(runs):
    tot = {k: [0] * len(BANDS) for k in ["ocean"] + PLACES}
    st = {g: [0] * 5 for g in GATES}
    el = {g: [] for g in GATES}
    snaps, giants, plates = 0, [], []
    for r in runs:
        for s in r["snaps"]:
            if "ocean" not in s:
                continue
            snaps += 1
            giants.append(s["giant_plates"])
            plates.append(s["plates"])
            for k in tot:
                tot[k] = [a + b for a, b in zip(tot[k], s[k])]
            for g in GATES:
                st[g] = [a + b for a, b in zip(st[g], s[f"status_{g}"])]
                el[g].append(s[f"eligible_plates_{g}"])
    return tot, st, el, snaps, giants, plates


def show(name, runs, names):
    tot, st, el, snaps, giants, plates = pooled(runs)
    if not snaps:
        print(f"{name}: no snapshots")
        return
    print(f"\n== {name}: {snaps} snapshots, plates {sum(plates) / snaps:.1f}, giant plates {sum(giants) / snaps:.2f}")
    oc = tot["ocean"]
    n = sum(oc)
    print(f"{'band':>8} {'ocean':>6} " + " ".join(f"{p[:12]:>12}" for p in PLACES))
    for b, lab in enumerate(BANDS):
        row = [tot[p][b] / max(oc[b], 1) for p in PLACES]
        print(f"{lab:>8} {oc[b] / n:6.3f} " + " ".join(f"{x:12.3f}" for x in row))
    old = sum(oc[6:])
    print(f"{'all':>8} {1.0:6.3f} " + " ".join(f"{sum(tot[p]) / n:12.3f}" for p in PLACES))
    print(f"{'>180':>8} {old / n:6.3f} " + " ".join(f"{sum(tot[p][6:]) / max(old, 1):12.3f}" for p in PLACES))
    print(f"{'>150':>8} {sum(oc[5:]) / n:6.3f} " + " ".join(f"{sum(tot[p][5:]) / max(sum(oc[5:]), 1):12.3f}"
                                                            for p in PLACES))
    for g in GATES:
        s = st[g]
        m = max(sum(s), 1)
        print(f"status >{g} Myr (share of the ocean {sum(s) / n:.3f}): " +
              ", ".join(f"{nm} {x / m:.3f}" for nm, x in zip(names, s)) +
              f"; plates eligible with a seed off the margin {sum(el[g]) / len(el[g]):.2f} per snapshot")


def main():
    runs = [json.loads(line) for line in open(sys.argv[1]) if line.strip()]
    names = runs[0].get("status_names", [])
    bad = []
    for r in runs:
        for s in r.get("snaps", []):
            c = s["check"]
            if c.get("missing") or len(set(c["records"])) != 1 or not c["nan_is_cont"] or \
                    c["rec_ocean"] != c["map_ocean"] or c["rec_old180"] != c["age_old180"]:
                bad.append((r["run"], c))
    print(f"checks: {sum(len(r.get('snaps', [])) for r in runs)} snapshots, {len(bad)} failing")
    for b in bad[:20]:
        print("  FAIL", b)
    for r in runs:
        show(f"{r['arm'].rstrip('/').split('/')[-1]}/{r['run']}", [r], names)
    for start in ("supercontinent", "scattered_continents"):
        show(f"pooled {start}", [r for r in runs if r["run"].startswith(start + "-")], names)
    show("pooled all", runs, names)


main()
