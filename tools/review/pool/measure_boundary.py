#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `boundary` measure (V1 gap 2, plate boundary organisation, 2026-10-23): how a run's plate
boundaries are organised, against Bird 2003, from one pool run folder, printed as a JSON object on stdout.
It needs no field dump: the plate check's output (`plate.json`), the dynamics record (`rec.json`), the
world globals (`fields/globals.json`) and the stage's summary (`summary.txt`).

    python3 -I measure_boundary.py <run folder>

The boundary mix, shares of the boundary's length over the steps after the first 100 Myr (the dynamics
record's `mix_km`, the classes of `plates::dynamics::boundary_mix`: the motion across the boundary's
normal over two rings of samples, Bird's 20 degree transform window, PLT-310 and PLT-311):

  sub          subducting (ocean-ocean and ocean-continent)        Bird 2003 Table 3: SUB 19.7 %
  spread       spreading (ridge and rift)                          OSR + CRB 36.3 %
  coll         at a collision (continental crust on the subducting side)       CCB 8.8 %
  transform    transform (the motion within 20 degrees of the boundary)       OTF + CTF 28.3 %
  nocontact    converging with no contact yet                      OCB 6.7 %
  oo_of_sub    the ocean-ocean share of the subducting boundary (PLT-370)  about 35 % (Stern & Scholl)
  sub_km, total_km   the subducting and the whole boundary, lattice km per step (one sample spacing per
               boundary sample counts a straight boundary about 1.04 times, the share test in
               plates/shortening.rs, ledger PLT-530: Bird's 51,310 km of SUB is about 53,500 lattice km
               and his 260,487 km about 271,700. Until 2026-10-24 this said 1.84, the model's boundary
               over Bird's, which took the model's boundary to be as long as Earth's)

How the trenches face their convergence (the plates step's own globals over the whole run):

  facing       the oceanic subduction fronts' convergence normal to the front over the full relative
               speed, each weighted by Cortial's factor min(v / v_0, 1) (`plates/arc_front_g_normal_km_myr`
               over `plates/arc_front_g_full_km_myr`, PLT-312's measurement). Bird 2003 Table 3's SUB: an
               area flux of 0.086516 m2/s over 51,310 km is 53.2 mm/yr across the trench against a mean
               relative speed of 62.3 mm/yr, 0.854. (The factor's cap at v_0 lifts the model's ratio a
               little; it is an upper bound on the plain ratio.)
  facing_cont  the same over the fronts under continental overriding crust (the cordilleras' fronts)
  slow_front   the share of the oceanic subduction front converging under 10 mm/yr across it (the
               record's front bands) …
  fast_front   … and at 80 mm/yr or more (where the factor's cap at v_0 = 100 mm/yr can bind)
  across_mm_yr the mean convergence across the oceanic subduction front, mm/yr, from the bands'
               middles (2.5, 7.5, 15, 30, 60, 100); Bird 2003 Table 3's SUB: 53.2

The continental margin and the collisions (the plate check's, `deeptime plate-check`):

  active_margin  the active share of the continental margin (CHK target 0.25 to 0.40), and the stage
               summary's margins line (`deeptime run`'s means over the records after 90 Myr), shares
               of the coast: m_sub at a subduction front, of it m_andean where the continent
               overrides and m_initiated at fronts a subduction initiation made; m_coll in a
               collision belt; m_passive passive
  front_coll_km  the front in collision at any time, km (`plates/collision_front_km_myr` over the run's
               length; Stern & Scholl 2010: 6,000 km of active suture on average, 12,000 km today)
  coll_life_p50, coll_life_p90   collision length, first to last step, Myr (targets 15 to 50, at most 120)
  collisions   collisions per 100 Myr (2 to 6)

The plates (the summary's counters per 100 Myr, the plate check's plates alive):

  plates       plates alive, median over steps (Bird 2003: 52, of which 19 over 0.0377 sr)
  rifts, initiations, reversals   per 100 Myr (Crameri et al. 2020: 13 initiations since 104 Ma, 4 of them
               by polarity reversal)

Version 2 (version 1 without fast_front, across_mm_yr and the margins line).
"""
import json
import sys
from pathlib import Path

SKIP_MYR = 100.0
# The middles of the front bands' convergence across the front (lower edges 0, 5, 10, 20, 40, 80
# mm/yr; the last band is open, 100 taken for it), mm/yr.
MIDDLES = (2.5, 7.5, 15.0, 30.0, 60.0, 100.0)
# Classes of dynamics::mix (ocean_ocean, ocean_continent, collision, ridge, rift, other).
OO, OC, COLL, RIDGE, RIFT, OTHER = range(6)


def read_json(p):
    try:
        return json.loads(Path(p).read_text())
    except (OSError, ValueError):
        return None


def plate_values(path):
    out = {}
    doc = read_json(path)
    if not doc:
        return out
    want = {
        "active_margin_mean": "active_margin",
        "collision_length_p50_myr": "coll_life_p50",
        "collision_length_p90_myr": "coll_life_p90",
        "collisions_per_100myr": "collisions",
        "plates_alive_p50": "plates",
    }
    for m in doc.get("measures", []):
        if m.get("key") in want and m.get("value") is not None:
            out[want[m["key"]]] = float(m["value"])
    return out


def record_values(path):
    rec = read_json(path)
    out = {}
    if not rec:
        return out, None
    d = rec.get("dynamics", rec)
    steps = d.get("steps", [])
    if not steps:
        return out, None
    t0 = d.get("t0_myr", steps[0]["t"])
    dt = d.get("dt_myr", 2.0)
    span = steps[-1]["t"] - t0 if steps[-1]["t"] > t0 else len(steps) * dt
    use = [s for s in steps if s["t"] > t0 + SKIP_MYR]
    mix = [0.0] * 6
    other_conv = 0.0
    slow, fast, across, fronts = 0.0, 0.0, 0.0, 0.0
    for s in use:
        for k, x in enumerate(s.get("mix_km", [0.0] * 6)):
            mix[k] += x
        fb = s.get("front_bands", {})
        other_conv += fb.get("other_converging", 0.0)
        # Normal bands 0, 5, 10, 20, 40, 80 mm/yr; rows ocean-ocean, ocean-continent.
        for row in fb.get("normal", []):
            slow += sum(row[:2])
            fast += sum(row[5:])
            across += sum(x * m for x, m in zip(row, MIDDLES))
            fronts += sum(row)
    total = sum(mix)
    if total > 0:
        n = max(len(use), 1)
        out["sub"] = (mix[OO] + mix[OC]) / total
        out["spread"] = (mix[RIDGE] + mix[RIFT]) / total
        out["coll"] = mix[COLL] / total
        out["transform"] = (mix[OTHER] - other_conv) / total
        out["nocontact"] = other_conv / total
        out["oo_of_sub"] = mix[OO] / max(mix[OO] + mix[OC], 1e-9)
        out["sub_km"] = (mix[OO] + mix[OC]) / n
        out["total_km"] = total / n
    if fronts > 0:
        out["slow_front"] = slow / fronts
        out["fast_front"] = fast / fronts
        out["across_mm_yr"] = across / fronts
    return out, span


def globals_values(path, span):
    doc = read_json(path)
    out = {}
    if not doc:
        return out
    g = doc.get("globals", doc)

    def ratio(a, b):
        x, y = g.get(a), g.get(b)
        return x / y if x is not None and y else None

    out["facing"] = ratio("plates/arc_front_g_normal_km_myr", "plates/arc_front_g_full_km_myr")
    out["facing_cont"] = ratio("plates/arc_front_g_normal_continental_km_myr",
                               "plates/arc_front_g_full_continental_km_myr")
    if span and g.get("plates/collision_front_km_myr") is not None:
        out["front_coll_km"] = g["plates/collision_front_km_myr"] / span
    return out


def summary_values(path, span):
    out = {}
    if not span:
        return out
    try:
        text = Path(path).read_text(errors="replace")
    except OSError:
        return out
    at = text.find("plates_final=")
    if at >= 0:
        line = text[at:].split("\n", 1)[0]
        for kv in line.split():
            k, _, v = kv.partition("=")
            if k in ("rifts", "initiations", "reversals"):
                try:
                    out[k] = 100.0 * float(v) / span
                except ValueError:
                    pass
    at = text.find("margins records=")
    if at >= 0:
        line = text[at:].split("\n", 1)[0]
        kv = dict(x.partition("=")[::2] for x in line.split() if "=" in x)
        try:
            active = float(kv["active_mean"])
            sub = float(kv["subduction_mean"])
            out["m_sub"] = sub
            out["m_andean"] = float(kv["andean_mean"])
            out["m_initiated"] = float(kv["initiated_mean"])
            out["m_coll"] = active - sub
            out["m_passive"] = float(kv["passive_mean"])
        except (KeyError, ValueError):
            pass
    return out


def main():
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    run = Path(sys.argv[1])
    values = plate_values(run / "plate.json")
    rec, span = record_values(run / "rec.json")
    values.update(rec)
    values.update(globals_values(run / "fields" / "globals.json", span))
    values.update(summary_values(run / "summary.txt", span))
    print(json.dumps({k: (None if v is None else float(v)) for k, v in values.items()}, sort_keys=True))


if __name__ == "__main__":
    main()
