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
  their buoyancy against 10 Myr (FRC-005); and, of those whose boundary the step before was spreading or absent, the
  younger plate's age and origin and the plates' relation (rec.json's plate lives);
  and the subducting fronts' mean length at any time, km, by their contact's onset (ocean-ocean and ocean-continent
  now), the median of the runs.

With the resampling trace's records (`resamples`, the onsets' `since_resample_myr`, `prov` and outcome) it also prints
the resamplings (how many, their spacing, the share of new samples by how the resampling made them: a gap of the gap
fill, PLT-502; a one-way slab gap, PLT-542; an orphan pocket; contested by two footprints; given another plate than the
old sample nearest it), and the onsets by time since the last resampling: how many, how their first front samples were
made (both sides), how many passed incipient and when, their slab's longest, and the front they made over the run
(ocean-ocean and ocean-continent km x Myr, as a mean length over the run).

With the overlap trace's records (the onsets' `overlap`, plates/dynamics.rs `OnsetOverlap`) it prints, by band and by
setting and route, the two plates' motion across the overlap at the onset: the overlapping samples converging, moving
along the boundary (PLT-311's transform, or at rest), parting or with no normal; the pair's own reading; and what the
onsets whose pair does not converge make. The pair's reading was the samples' normals summed at the overlap's
centroid in the records made before PR #55's review fixes (2026-10-31), and is the samples' mean convergence since
(PLT-810, PLT-811), so `pair_conv_mm_yr` equals `conv_mm_yr` in the newer records.

Signs: the boundary the step before (`before.normal_mm_yr`, dynamics.rs `PairBound`) is negative converging, positive
parting; the overlap's convergence (`overlap.conv_mm_yr`, `pair_conv_mm_yr`, subduction.rs `OverlapMotion`) is
positive converging.
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
    onsets, length_myr, fronts, resamples, step_ts = [], 0.0, {}, [], []
    runs = 0
    for folder in argv:
        for r in runs_of(folder):
            rec = json.loads((r / "rec.json").read_text())
            steps = rec.get("steps") or []
            if not steps:
                continue
            runs += 1
            length_myr += float(steps[-1]["t"]) - float(rec.get("t0_myr", steps[0]["t"]))
            lives = {int(x["id"]): x for x in rec.get("plates") or []}
            resamples.append(rec.get("resamples") or [])
            step_ts.append([float(s["t"]) for s in steps])
            for o in rec.get("onsets") or []:
                lu, lo = lives.get(o["under"], {}), lives.get(o["over"], {})
                o["lives"] = (lu, lo)
                pu, po = lu.get("parent"), lo.get("parent")
                o["relation"] = ("parent" if pu == o["over"] or po == o["under"]
                                 else "siblings" if pu is not None and pu == po else "other")
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
        # The plates' lives at the onset: the younger plate's age and how it was born, by the boundary the step before.
        for kind in (0, 5):
            sub = [o for o in new if o["kind"] == kind]
            if not sub:
                continue
            print(f"  of those with the boundary {KINDS[kind]} the step before ({len(sub)}):")
            share_table("    the younger plate's age, Myr", sub,
                        lambda o: band(young_age(o), [0, 2.001, 10, 50], ["<=2", "2-10", "10-50", ">50"]),
                        ["<=2", "2-10", "10-50", ">50"])
            share_table("    the younger plate's origin", sub, lambda o: young_origin(o),
                        ["rift", "fission", "back_arc", "subduction_initiation", "margin_propagation", "initial",
                         "other"])
            share_table("    the plates' relation", sub, lambda o: o.get("relation", "?"), ["parent", "siblings", "other"])
    if any(resamples):
        resampling_report(onsets, resamples, length_myr, step_ts)
    if any((o.get("overlap") or {}).get("n", 0) > 0 for o in onsets):
        overlap_report(onsets, length_myr)
    print("# the subducting fronts by their contact's onset, mean km at any time, median of the runs")
    for key, names in (("onset_kind", KINDS), ("onset_route", ROUTES), ("onset_buoyant", BUOYANT)):
        for setting in ("oo", "oc"):
            vals = [statistics.median(fronts.get((key, setting, n), [0.0])) for n in names]
            tot = max(sum(vals), 1e-9)
            print(f"  {key:13s} {setting}: " + "  ".join(f"{n} {v:6.0f} ({v / tot:.3f})" for n, v in zip(names, vals)))
    return 0


PROV = ["gap", "slab gap", "orphan", "contested", "moved"]


OVERLAP = ["converging", "transform", "parting", "no normal"]


def overlap_lines(sub, length_myr):
    # The overlap trace (OnsetRec.overlap): the plates' motion across the overlap at the onset, by sample and by
    # the pair's own reading, and what the onsets whose pair does not converge make.
    ov = [o for o in sub if (o.get("overlap") or {}).get("n", 0) > 0]
    if not ov:
        return
    n = sum(o["overlap"]["n"] for o in ov)
    by = [sum(o["overlap"][k.replace(" ", "_")] for o in ov) / n for k in OVERLAP]
    pair = [sum(1 for o in ov if o["overlap"]["pair_class"] == i) / len(ov) for i in range(len(OVERLAP))]
    rest = [o for o in ov if o["overlap"]["pair_class"] != 0]
    km = [sum(o["front_km_myr"][k] for o in rest) / max(length_myr, 1e-9) for k in (0, 1)]
    passed = sum(1 for o in rest if o["subducting_after_myr"] >= 0) / max(len(rest), 1)
    print(f"    the overlap at the onset ({len(ov)} of {len(sub)} read): samples " + "  ".join(
        f"{k} {v:.3f}" for k, v in zip(OVERLAP, by)) + "; the pair " + "  ".join(
        f"{k} {v:.3f}" for k, v in zip(OVERLAP, pair)) + f"; convergence median"
          f" {statistics.median([o['overlap']['conv_mm_yr'] for o in ov]):.1f} mm/yr (samples), "
          f"{statistics.median([o['overlap']['pair_conv_mm_yr'] for o in ov]):.1f} (the pair)")
    print(f"    of them with the pair not converging: passed incipient {passed:.3f}; front made, mean km over the run:"
          f" oo {km[0]:.0f}, oc {km[1]:.0f}")
    # The samples' mean convergence (`conv_mm_yr`, PLT-811's reading since PR #55's fixes), positive, by the pair's
    # class as recorded: in records made before the fixes, how many the mean would have called converging.
    by = [sum(1 for o in ov if o["overlap"]["pair_class"] == i and o["overlap"]["conv_mm_yr"] > 0.0)
          / max(sum(1 for o in ov if o["overlap"]["pair_class"] == i), 1) for i in range(len(OVERLAP))]
    print(f"    the samples' mean convergence positive: all {sum(1 for o in ov if o['overlap']['conv_mm_yr'] > 0.0) / len(ov):.3f};"
          f" by the recorded class " + "  ".join(f"{k} {v:.3f}" for k, v in zip(OVERLAP, by)))


def overlap_report(onsets, length_myr):
    print("# the overlap trace: onsets by setting at the onset and route, all times")
    for setting in (0, 1):
        for route, name in enumerate(ROUTES[:3]):
            sub = [o for o in onsets if o["setting"] == setting and o["route"] == route]
            if not sub:
                continue
            print(f"  {SETTINGS[setting]} {name} ({len(sub)})")
            overlap_lines(sub, length_myr)


def since_of(t, res_ts):
    # The time since the last resampling at or before t (-1 before any), as the onset trace records it.
    last = [r for r in res_ts if r <= t + 1e-9]
    return t - last[-1] if last else -1.0


def resampling_report(onsets, resamples, length_myr, step_ts):
    n_res = sum(len(r) for r in resamples)
    gaps = [b["t"] - a["t"] for r in resamples for a, b in zip(r, r[1:])]
    tot = [0] * len(PROV)
    samples = 0
    for r in resamples:
        for x in r:
            samples += x["samples"]
            for i, c in enumerate(x["bits"]):
                tot[i] += c
    print(f"# resamplings: {n_res} over {len(resamples)} runs, spacing median {statistics.median(gaps) if gaps else 0:.0f}"
          f" Myr; new samples by how they were made: " + "  ".join(f"{n} {c / max(samples, 1):.4f}" for n, c in
                                                                   zip(PROV, tot)))
    traced = [o for o in onsets if "since_resample_myr" in o and o.get("prov")]
    if not traced:
        return
    bands = [("its own step", lambda x: 0 <= x < 1e-6), ("the step after", lambda x: 1e-6 <= x <= 2.001),
             ("2-10 Myr after", lambda x: 2.001 < x <= 10), ("over 10 Myr after", lambda x: x > 10),
             ("before any", lambda x: x < 0)]
    # The chance rate: the share of the runs' steps in each band (an onset on a step drawn at random).
    sinces = [since_of(t, [x["t"] for x in r]) for ts, r in zip(step_ts, resamples) for t in ts]
    dts = [b - a for ts in step_ts for a, b in zip(ts, ts[1:])]
    print(f"# steps: {len(sinces)} over the runs, step median {statistics.median(dts) if dts else 0:g} Myr; share of"
          f" steps by time since the last resampling: " + "  ".join(
              f"{name} {sum(1 for x in sinces if test(x)) / max(len(sinces), 1):.3f}" for name, test in bands))
    near = sorted({round(x, 6) for x in sinces if 0 <= x <= 6.001})
    for setting in (0, 1):
        sub = [o for o in traced if o["setting"] == setting]
        print(f"# onsets {SETTINGS[setting]} at the onset by the exact time since the last resampling (up to 6 Myr):"
              f" share of the onsets / share of the steps: " + "  ".join(
                  f"{v:g} {sum(1 for o in sub if abs(o['since_resample_myr'] - v) < 1e-6) / max(len(sub), 1):.3f}"
                  f"/{sum(1 for x in sinces if abs(x - v) < 1e-6) / max(len(sinces), 1):.3f}" for v in near))
    for setting in (0, 1):
        print(f"# onsets {SETTINGS[setting]} at the onset, by time since the last resampling (all routes)")
        for name, test in bands:
            sub = [o for o in traced if o["setting"] == setting and test(o["since_resample_myr"])]
            if not sub:
                continue
            n = sum(o["prov"]["n"] for o in sub)
            over = [sum(o["prov"]["over_bits"][i] for o in sub) / max(n, 1) for i in range(len(PROV))]
            under = [sum(o["prov"]["under_bits"][i] for o in sub) / max(n, 1) for i in range(len(PROV))]
            hidden = sum(o["prov"]["under_hidden"] for o in sub) / max(n, 1)
            passed = [o for o in sub if o["subducting_after_myr"] >= 0]
            km = [sum(o["front_km_myr"][k] for o in sub) / max(length_myr, 1e-9) for k in (0, 1)]
            print(f"  {name:18s} {len(sub):4d} onsets ({len(sub) / len(traced):.3f}); passed incipient"
                  f" {len(passed) / len(sub):.3f}, after median"
                  f" {statistics.median([o['subducting_after_myr'] for o in passed]) if passed else -1:.0f} Myr;"
                  f" longest slab median {statistics.median([o['max_slab_km'] for o in sub]):.0f} km;"
                  f" front made, mean km over the run: oo {km[0]:.0f}, oc {km[1]:.0f}")
            print("    front samples' provenance, overriding side: " + "  ".join(
                f"{p} {v:.3f}" for p, v in zip(PROV, over)) + f"; subducting side: " + "  ".join(
                f"{p} {v:.3f}" for p, v in zip(PROV, under)) + f" (gone under {hidden:.3f})")
            # A front sample needs a converging, non-transform neighbour of the subducting plate
            # (dynamics.rs `front_index`), so an onset with none had no converging front that step.
            none = sum(1 for o in sub if o["prov"]["n"] == 0)
            idle = [o for o in sub if o["prov"]["n"] == 0]
            km_idle = [sum(o["front_km_myr"][k] for o in idle) / max(length_myr, 1e-9) for k in (0, 1)]
            passed_idle = sum(1 for o in idle if o["subducting_after_myr"] >= 0) / max(len(idle), 1)
            print(f"    at the onset: no converging front sample {none / len(sub):.3f}, front samples median"
                  f" {statistics.median([o['prov']['n'] for o in sub]):.0f}; relative speed median"
                  f" {statistics.median([o['convergence_mm_yr'] for o in sub]):.0f} mm/yr; the boundary the step"
                  f" before: " + "  ".join(
                      f"{k} {sum(1 for o in sub if KINDS[o['kind']] == k) / len(sub):.3f}" for k in KINDS[:6]))
            print(f"    of them with no converging front sample at the onset: passed incipient {passed_idle:.3f};"
                  f" front made, mean km over the run: oo {km_idle[0]:.0f}, oc {km_idle[1]:.0f}")
            overlap_lines(sub, length_myr)


def young_age(o):
    born = [float(x.get("born_myr", -1e9)) for x in o["lives"] if x]
    return o["t"] - max(born) if born else 1e9


def young_origin(o):
    known = [x for x in o["lives"] if x]
    if not known:
        return "other"
    x = max(known, key=lambda x: float(x.get("born_myr", -1e9)))
    return x.get("origin", "other")


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
