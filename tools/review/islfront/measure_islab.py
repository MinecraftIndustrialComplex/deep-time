#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `islab` measure (the M5 lane, 2026-10-26; measurement only): the island and continental
subduction fronts, their speed and turnover, and the plates' speeds by slab, for the island-front fixes
A (the incipient front's net resistance, PLT-680) and B (the slab's thermal floor, PLT-681), from a run's
`rec.json` and `plate.json`, printed as a JSON object on stdout.

    python3 -I measure_islab.py <run folder>

Fronts are the dynamics record's boundary mix (`front_bands`, one lattice spacing a front sample, the
current geometry, PLT-582), means over every step with front bands. Setting 0 is the island front
(ocean-ocean: no continental crust of the overriding plate within the arc's distance, PLT-370), setting
1 the continental front. A front is subducting or incipient (PLT-610, and with PLT-681 on, a sample
converging under 10 mm/yr).

  isl_km, cont_km        the front, km per step (subducting plus incipient)
  isl_sub_km, cont_sub_km   its subducting part
  isl_inc, cont_inc      incipient share of the front
  isl_slow, cont_slow    share of the front incipient by the thermal floor alone (`slow`, PLT-683)
  isl_sub_u10, cont_sub_u10   share of the subducting front converging under 10 mm/yr (sample v_n)
  sub_u10_fastpair       of the subducting front converging under 10 mm/yr (both settings), the share
                         on a pair of plates whose mean convergence is 10 or more (`normal_pair`,
                         PLT-683): what a per-sample floor cuts that a per-pair one would keep
  sub_o10_slowpair       of the subducting front converging at 10 or more, the share on a pair whose
                         mean is under 10: what a per-pair floor cuts that a per-sample one keeps
  sub_slowpair           share of the subducting front on pairs whose mean is under 10
  isl_gain_sub, isl_loss_sub, isl_gain_inc, isl_loss_inc   the island front's turnover, km per step
                         (`gain`, `loss`); cont_* the same for the continental front
  gain_sub_longslab      share of the subducting front's gain on pairs whose slab is already 100 km
                         or more (`gain_slab`)
  matured_per_100myr     contacts whose slab passed the critical convergence (`matured`), per 100 Myr
  matured_vn_p50, matured_age_p50   their oceanic front's mean convergence (mm/yr) and the contact's age
                         (Myr) at that step, medians over the run's events
  speed_slab, speed_noslab, speed_slab_ratio   plate.json: median speed of plates with a slab on 20 % of
                         their boundary or more / less, and their ratio (the plate check's records)
  speed_slab_over_free   plate.json: plates with 20 % or more of the edge on slab over those under 5 %
  speed_rms, speed_p50, speed_p10, slab_share   plate.json: the area-weighted RMS speed, the median and
                         10th percentile of plates of 30+ samples, the mean slab share of the boundary

The cross-tab columns (slow, normal_pair, gain, loss, gain_slab, matured) are absent from records made
before PLT-683 and print as null there.
"""
import json
import statistics
import sys

BANDS = 6
U10 = 2  # NORMAL_BANDS_MM_YR [0, 5, 10, 20, 40, 80]: bands 0 and 1 are under 10 mm/yr
LONG = 2  # SLAB_BANDS_KM [0, 50, 100, 150, 300, 600]: bands 2 and up are 100 km or more


def ratio(a, b):
    return a / b if b else None


def main():
    run = sys.argv[1]
    d = json.load(open(f"{run}/rec.json"))
    steps = [s for s in d["steps"] if "front_bands" in s]
    n = max(len(steps), 1)
    o = {}
    tot = [0.0, 0.0]
    sub = [0.0, 0.0]
    inc = [0.0, 0.0]
    slow = [0.0, 0.0]
    u10 = [0.0, 0.0]
    has_x = False
    u10_fast = 0.0
    u10_all = 0.0
    o10_slow = 0.0
    o10_all = 0.0
    slowpair = 0.0
    gain = [[0.0, 0.0], [0.0, 0.0]]
    loss = [[0.0, 0.0], [0.0, 0.0]]
    gslab_long = 0.0
    gslab_all = 0.0
    matured = []
    for s in steps:
        b = s["front_bands"]
        for k in (0, 1):
            sk = sum(b["slab"][k])
            ik = b.get("incipient", [0.0, 0.0])[k]
            sub[k] += sk
            inc[k] += ik
            tot[k] += sk + ik
            u10[k] += sum(b["normal"][k][:U10])
        if "normal_pair" in b:
            has_x = True
            for k in (0, 1):
                slow[k] += b["slow"][k]
                m = b["normal_pair"][k]
                for i in range(BANDS):
                    for j in range(BANDS):
                        x = m[i][j]
                        if i < U10:
                            u10_all += x
                            if j >= U10:
                                u10_fast += x
                        else:
                            o10_all += x
                            if j < U10:
                                o10_slow += x
                        if j < U10:
                            slowpair += x
                for st in (0, 1):
                    gain[k][st] += b["gain"][k][st]
                    loss[k][st] += b["loss"][k][st]
                gslab_all += sum(b["gain_slab"][k])
                gslab_long += sum(b["gain_slab"][k][LONG:])
        matured.extend(s.get("matured", []))
    for k, name in ((0, "isl"), (1, "cont")):
        o[f"{name}_km"] = tot[k] / n
        o[f"{name}_sub_km"] = sub[k] / n
        o[f"{name}_inc"] = ratio(inc[k], tot[k])
        o[f"{name}_sub_u10"] = ratio(u10[k], sub[k])
        o[f"{name}_slow"] = ratio(slow[k], tot[k]) if has_x else None
        for st, sname in ((1, "sub"), (0, "inc")):
            o[f"{name}_gain_{sname}"] = gain[k][st] / n if has_x else None
            o[f"{name}_loss_{sname}"] = loss[k][st] / n if has_x else None
    o["sub_u10_fastpair"] = ratio(u10_fast, u10_all) if has_x else None
    o["sub_o10_slowpair"] = ratio(o10_slow, o10_all) if has_x else None
    o["sub_slowpair"] = ratio(slowpair, u10_all + o10_all) if has_x else None
    o["gain_sub_longslab"] = ratio(gslab_long, gslab_all) if has_x else None
    span = (steps[-1]["t"] - steps[0]["t"]) if len(steps) > 1 else 0.0
    o["matured_per_100myr"] = (100.0 * len(matured) / span if span else None) if has_x else None
    o["matured_vn_p50"] = statistics.median([m[0] for m in matured]) if matured else None
    o["matured_age_p50"] = statistics.median([m[1] for m in matured]) if matured else None
    try:
        p = {m["key"]: m.get("value") for m in json.load(open(f"{run}/plate.json"))["measures"]}
    except (OSError, KeyError, ValueError):
        p = {}
    for key, src in (("speed_slab", "speed_slab_mm_yr"), ("speed_noslab", "speed_noslab_mm_yr"),
                     ("speed_slab_ratio", "speed_slab_ratio"),
                     ("speed_slab_over_free", "speed_slab_over_free"), ("speed_rms", "speed_rms_mm_yr"),
                     ("speed_p50", "speed_big_p50_mm_yr"), ("speed_p10", "speed_big_p10_mm_yr"),
                     ("slab_share", "slab_share_mean")):
        o[key] = p.get(src)
    print(json.dumps(o))


main()
