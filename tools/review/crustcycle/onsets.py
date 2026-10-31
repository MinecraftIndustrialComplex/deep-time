#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How the model's subduction zones begin, from the onset trace's records (no simulation): onsets.py <arm folder> ...

Reads each run's rec.json: `onsets` (plates/dynamics.rs `OnsetRec`, one per contact onset: a new contact or a polarity
reversal's new onset, with its pair of plates' boundary the step before) and the steps' `front_bands.onset_kind`,
`onset_route` and `onset_buoyant` (the subducting fronts by their contact's onset). Prints, pooled over the runs:

  the onsets per 1,000 Myr by route (plates that came to converge, a reversal, an initiation's pair) and by setting at
  the onset (ocean-ocean, ocean-continent, continent-continent);
  for the onsets of plates that came to converge, ocean-ocean at the onset: the boundary the step before (spreading,
  converging, transform, front, collision, none), how long it had converged, since it last spread, the boundary's
  age, the velocity across it, the two floors' ages there (the subducting plate's and the overriding plate's) and
  their buoyancy against 10 Myr (FRC-005);
  and the subducting fronts' mean length at any time, km, by their contact's onset (ocean-ocean and ocean-continent
  now), the median of the runs.
"""
import json
import statistics
import sys
from pathlib import Path

KINDS = ["spreading", "converging", "transform", "front", "collision", "none", "unknown"]
ROUTES = ["converged", "reversal", "initiation", "unknown"]
BUOYANT = ["both <10", "under <10", "over <10", "neither", "unknown"]
SETTINGS = ["oo", "oc", "cc"]


def runs_of(folder):
    return sorted(p for p in Path(folder).iterdir() if (p / "rec.json").is_file())


def band(x, edges, names):
    for (lo, hi), n in zip(zip(edges, edges[1:] + [float("inf")]), names):
        if lo <= x < hi:
            return n
    return names[-1]


def share_table(title, rows, key, order):
    n = len(rows)
    counts = {}
    for r in rows:
        k = key(r)
        counts[k] = counts.get(k, 0) + 1
    print(f"  {title}: " + "  ".join(f"{k} {counts.get(k, 0) / max(n, 1):.3f}" for k in order))


def main(argv):
    onsets, length_myr, fronts = [], 0.0, {}
    runs = 0
    for folder in argv:
        for r in runs_of(folder):
            rec = json.loads((r / "rec.json").read_text())
            steps = rec.get("steps") or []
            if not steps:
                continue
            runs += 1
            length_myr += float(steps[-1]["t"]) - float(rec.get("t0_myr", steps[0]["t"]))
            for o in rec.get("onsets") or []:
                onsets.append(o)
            acc = {}
            n = 0
            for s in steps:
                fb = s.get("front_bands") or {}
                if "onset_kind" not in fb:
                    continue
                n += 1
                for key, names in (("onset_kind", KINDS), ("onset_route", ROUTES), ("onset_buoyant", BUOYANT)):
                    for si, setting in enumerate(("oo", "oc")):
                        for i, name in enumerate(names):
                            k = (key, setting, name)
                            acc[k] = acc.get(k, 0.0) + fb[key][si][i]
            for k, v in acc.items():
                fronts.setdefault(k, []).append(v / max(n, 1))
    if not onsets:
        print("no onsets recorded (records made before the onset trace)")
        return 1
    per = 1000.0 / max(length_myr / max(runs, 1), 1e-9) / max(runs, 1)
    print(f"# {runs} runs, {len(onsets)} onsets; per 1,000 Myr of one run:")
    for i, name in enumerate(ROUTES[:3]):
        xs = [o for o in onsets if o["route"] == i]
        by = "  ".join(f"{s} {sum(1 for o in xs if o['setting'] == k) * per:.0f}" for k, s in enumerate(SETTINGS))
        print(f"  {name:10s} {len(xs) * per:6.0f}  ({by})")
    for setting in (0, 1):
        new = [o for o in onsets if o["route"] == 0 and o["setting"] == setting]
        print(f"# the onsets of plates that came to converge, {SETTINGS[setting]} at the onset ({len(new)}): shares")
        share_table("the boundary the step before", new, lambda o: KINDS[o["kind"]], KINDS[:6])
        share_table("converged for, Myr", new,
                    lambda o: band(o["converging_myr"], [0, 0.001, 2.001, 10, 30], ["0", "<=2", "2-10", "10-30", ">30"]),
                    ["0", "<=2", "2-10", "10-30", ">30"])
        share_table("since it last spread, Myr", new,
                    lambda o: "never" if o["since_spread_myr"] < 0 else band(
                        o["since_spread_myr"], [0, 2.001, 10, 50], ["<=2", "2-10", "10-50", ">50"]),
                    ["<=2", "2-10", "10-50", ">50", "never"])
        share_table("the boundary's age, Myr", new,
                    lambda o: "new" if o["boundary_age_myr"] < 0 else band(
                        o["boundary_age_myr"], [0, 2.001, 10, 50], ["<=2", "2-10", "10-50", ">50"]),
                    ["<=2", "2-10", "10-50", ">50", "new"])
        share_table("velocity across the step before, mm/yr", new,
                    lambda o: band(o["before"]["normal_mm_yr"], [-1e9, -40, -10, 0, 10],
                                   ["<-40", "-40..-10", "-10..0", "0..10", ">10"]),
                    ["<-40", "-40..-10", "-10..0", "0..10", ">10"])

        def floor(o, plate):
            b = o["before"]
            side = 0 if plate == b["lo"] else 1
            if b["ocean"][side] == 0:
                return "continental" if b["cont"][side] else "none"
            return band(b["age_myr"][side], [0, 6, 12, 24, 55], ["0-6", "6-12", "12-24", "24-55", ">55"])

        names = ["0-6", "6-12", "12-24", "24-55", ">55", "continental", "none"]
        share_table("the subducting plate's floor there, Myr", new, lambda o: floor(o, o["under"]), names)
        share_table("the overriding plate's floor there, Myr", new, lambda o: floor(o, o["over"]), names)
        share_table("buoyancy against 10 Myr", new,
                    lambda o: BUOYANT[buoyant(o)], BUOYANT)
    print("# the subducting fronts by their contact's onset, mean km at any time, median of the runs")
    for key, names in (("onset_kind", KINDS), ("onset_route", ROUTES), ("onset_buoyant", BUOYANT)):
        for setting in ("oo", "oc"):
            vals = [statistics.median(fronts.get((key, setting, n), [0.0])) for n in names]
            tot = max(sum(vals), 1e-9)
            print(f"  {key:13s} {setting}: " + "  ".join(f"{n} {v:6.0f} ({v / tot:.3f})" for n, v in zip(names, vals)))
    return 0


def buoyant(o):
    b = o["before"]
    side = lambda p: 0 if p == b["lo"] else 1
    u, v = side(o["under"]), side(o["over"])
    if b["ocean"][u] == 0:
        return 4
    young = lambda i: b["ocean"][i] > 0 and b["age_myr"][i] < 10.0
    return {(True, True): 0, (True, False): 1, (False, True): 2, (False, False): 3}[(young(u), young(v))]


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
