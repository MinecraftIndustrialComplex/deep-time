#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Plate dynamics of a run (docs/design/plate-collisions-and-welding.md): how large the largest
plate is, how long plates live and how they die, how much land one step adds, and the shape of
what each collision lifted.

    python3 tools/review/plate-dynamics.py [--label L] [--json out.json] <run> [<run> ...]

<run> is the path of a run of tools/review/plate-dynamics.sh without its extension: it reads
<run>.trace (the per-step and per-collision lines of tools/review/plate-dynamics-trace.patch) and
<run>.events.json (the viewer bundle's event list). Standard library only.

Shares are of the visible L0 samples (equal areas), so they are shares of the planet. Lengths are
Earth-normalized km (the plate model's sphere, radius 6,370 km). A set's extent is taken from its
second moments: length = sqrt(12 * l1) * R and width = sqrt(12 * l2) * R (exact for a uniform
strip), elongation = length / width. Earth's collision belts are 5-15 (the Himalaya is about
2,400 km by 250-400 km, the Urals 2,500 by 150, the Alps 1,200 by 200).
"""
import json
import math
import statistics
import sys

R_KM = 6370.0


def eig3(m):
    """Eigenvalues (descending) of the symmetric matrix [xx, yy, zz, xy, xz, yz]."""
    a, b, c, d, e, f = m
    p1 = d * d + e * e + f * f
    q = (a + b + c) / 3.0
    p2 = (a - q) ** 2 + (b - q) ** 2 + (c - q) ** 2 + 2.0 * p1
    if p2 < 1e-300:
        return [q, q, q]
    p = math.sqrt(p2 / 6.0)
    ba, bb, bc, bd, be, bf = (a - q) / p, (b - q) / p, (c - q) / p, d / p, e / p, f / p
    det = ba * (bb * bc - bf * bf) - bd * (bd * bc - bf * be) + be * (bd * bf - bb * be)
    r = max(-1.0, min(1.0, det / 2.0))
    phi = math.acos(r) / 3.0
    l1 = q + 2.0 * p * math.cos(phi)
    l3 = q + 2.0 * p * math.cos(phi + 2.0 * math.pi / 3.0)
    return [l1, 3.0 * q - l1 - l3, l3]


def extent(mom):
    """(samples, length km, width km, elongation) of a traced moment list, or None."""
    n = mom[0]
    if n < 6:
        return None
    mu = [x / n for x in mom[1:4]]
    xx, yy, zz, xy, xz, yz = [x / n for x in mom[4:10]]
    cov = [xx - mu[0] ** 2, yy - mu[1] ** 2, zz - mu[2] ** 2,
           xy - mu[0] * mu[1], xz - mu[0] * mu[2], yz - mu[1] * mu[2]]
    l1, l2, _ = eig3(cov)
    if l2 <= 0.0:
        return None
    return n, math.sqrt(12 * l1) * R_KM, math.sqrt(12 * l2) * R_KM, math.sqrt(l1 / l2)


def pct(xs, q):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    k = (len(xs) - 1) * q
    lo, hi = int(math.floor(k)), int(math.ceil(k))
    return xs[lo] + (xs[hi] - xs[lo]) * (k - lo)


def med(xs):
    return statistics.median(xs) if xs else float("nan")


def analyse(run):
    steps, collisions = [], []
    for line in open(run + ".trace"):
        d = json.loads(line)
        if d["k"] == "step":
            steps.append(d)
        elif d["k"] == "collision":
            collisions.append(d)
    ev = json.load(open(run + ".events.json"))
    kind = lambda e: e["kind"].split(":")[-1]
    par = lambda e, k: e.get("params", {}).get(k)
    tm = lambda e: e["t_start"] / 1e6
    out = {"run": run, "steps": len(steps)}
    t0, t1 = steps[0]["t"], steps[-1]["t"]
    dt = (t1 - t0) / max(1, len(steps) - 1)
    out["span_myr"] = round(t1 - t0 + dt)

    # 1. Plate sizes.
    largest, big5, alive, n_cont = [], [], [], []
    for s in steps:
        vis = sum(p[1] for p in s["plates"])
        shares = sorted((p[1] / vis for p in s["plates"]), reverse=True)
        largest.append(shares[0])
        big5.append(sum(x >= 0.05 for x in shares))
        alive.append(len(shares))
    out["largest_p50"] = med(largest)
    out["largest_max"] = max(largest)
    for th in (0.2, 0.3, 0.5, 0.7):
        out[f"largest_over_{int(th * 100)}"] = sum(x > th for x in largest) / len(largest)
    out["plates_over_5pct_p50"] = med(big5)
    out["alive_min"], out["alive_p50"], out["alive_max"] = min(alive), med(alive), max(alive)
    out["largest_series_50myr"] = [round(largest[i], 2) for i in
                                   range(0, len(largest), max(1, round(50 / dt)))]

    # 2. Lifetimes and fates.
    born, origin, parent = {}, {}, {}
    for s in steps[:1]:
        for p in s["plates"]:
            born[p[0]], origin[p[0]] = t0 - dt, "initial"
    for e in ev:
        if kind(e) == "plate_birth":
            pid = par(e, "plate")
            born[pid], origin[pid], parent[pid] = tm(e), par(e, "origin"), par(e, "parent")
    died, fate, into = {}, {}, {}
    for e in ev:
        if kind(e) == "plate_death":
            pid = par(e, "plate")
            died[pid], fate[pid], into[pid] = tm(e), par(e, "fate"), par(e, "absorbed_by")
    out["births"] = sum(1 for p in born if origin[p] != "initial")
    out["deaths"] = len(died)
    lives = [died[p] - born[p] for p in died if p in born]
    out["life_p50_myr"] = med(lives)
    out["dead_within_20myr"] = sum(x <= 20 for x in lives) / max(1, len(lives))
    fates = {}
    for p in died:
        fates[fate[p]] = fates.get(fate[p], 0) + 1
    out["fates"] = fates
    for o in ("rift", "subduction_initiation", "initial"):
        ids = [p for p in born if origin[p] == o]
        dead = [p for p in ids if p in died]
        ls = [died[p] - born[p] for p in dead]
        # Survival to 50 Myr among plates born at least 50 Myr before the end (no censoring).
        old = [p for p in ids if born[p] <= t1 - 50]
        surv = [p for p in old if p not in died or died[p] - born[p] > 50]
        out[o] = {
            "born": len(ids), "dead": len(dead), "life_p50_myr": med(ls),
            "survive_50myr": len(surv) / len(old) if old else float("nan"),
            "sutured": sum(fate[p] == "sutured" for p in dead),
            "sutured_to_parent": sum(fate[p] == "sutured" and into[p] == parent.get(p) for p in dead),
        }
    # Who takes a rift-born plate that is welded: the plate holding its parent's crust then
    # (plate ids change hands at every weld, so parents are followed through them), a sibling
    # of the same rift, or another plate.
    succ = {}

    def holder(p):
        while p in succ:
            p = succ[p]
        return p

    back = {"parent": [], "sibling": [], "other": []}
    for t, p in sorted((died[p], p) for p in died):
        if origin.get(p) == "rift" and fate[p] == "sutured":
            sibs = {holder(q) for q in born
                    if q != p and origin[q] == "rift" and born[q] == born[p] and parent[q] == parent[p]}
            who = ("parent" if into[p] == holder(parent[p])
                   else "sibling" if into[p] in sibs else "other")
            back[who].append(t - born[p])
        if into[p] is not None:
            succ[p] = into[p]
    out["rift_born_welded_to"] = {k: len(v) for k, v in back.items()}
    out["rift_born_welded_to_life_p50"] = {k: med(v) for k, v in back.items() if v}
    # After a rift splits a plate covering more than half the planet: how long until one plate
    # covers more than half again.
    t_index = {s["t"]: i for i, s in enumerate(steps)}
    regrow = []
    for e in ev:
        i = t_index.get(tm(e)) if kind(e) == "rift_onset" else None
        if i and largest[i - 1] > 0.5 and largest[i] <= 0.5:
            j = i
            while j < len(steps) and largest[j] <= 0.5:
                j += 1
            if j < len(steps):
                regrow.append((j - i) * dt)
    out["megaplate_regrows_myr_p50"] = med(regrow)
    out["megaplate_regrows_n"] = len(regrow)
    # Plates alive at the end: how old.
    ages = [t1 - born[p[0]] for p in steps[-1]["plates"] if p[0] in born]
    out["age_of_living_p50_myr"] = med(ages)
    out["rifts"] = sum(kind(e) == "rift_onset" for e in ev)
    out["rift_parent_over_half"] = sum(
        kind(e) == "rift_onset" and (par(e, "area_km2") or 0) > 0.5 * 4 * math.pi * 782.2784 ** 2
        for e in ev)

    # 3. Land per step.
    # Of all L0 samples (the visible ones change at every resampling; a gap between plates is sea).
    n_all = max(s["visible"] for s in steps)
    land = [s["land"] / n_all for s in steps]
    jumps = [(land[i] - land[i - 1], steps[i]["t"]) for i in range(1, len(land))]
    col_t = {}
    for c in collisions:
        col_t.setdefault(c["t"], []).append(c)
    # The first 10 Myr are the start-up (the surface settling on the initial crust), kept apart.
    out["land_p50"] = med(land)
    out["land_gain_max_startup"] = max(j for j, t in jumps if t < t0 + 10)
    jumps = [(j, t) for j, t in jumps if t >= t0 + 10]
    out["land_gain_max"] = max(j for j, _ in jumps)
    out["land_gain_p99"] = pct([j for j, _ in jumps], 0.99)
    out["land_gain_over_1pct"] = sum(j > 0.01 for j, _ in jumps)
    big_t = {c["t"] for c in collisions if c["frac"] >= 0.1}
    out["land_gain_p50_collision_steps"] = med([j for j, t in jumps if t in big_t])
    out["land_gain_p50_other_steps"] = med([j for j, t in jumps if t not in big_t])
    out["land_gain_max_other_steps"] = max([j for j, t in jumps if t not in big_t], default=float("nan"))
    top = sorted(jumps, reverse=True)[:5]
    out["land_gain_top5"] = [(round(j, 4), t, len(col_t.get(t, []))) for j, t in top]
    out["top20_gains_with_collision"] = sum(
        t in col_t for _, t in sorted(jumps, reverse=True)[:20])

    # 4. Collisions.
    big = [c for c in collisions if c["frac"] >= 0.1]
    out["collisions"] = len(big)
    out["terrane_transfers"] = len(collisions) - len(big)
    out["sutures"] = sum(c["sutured"] for c in collisions)
    lifted = [c["m10"][0] / n_all for c in big]
    out["lifted_share_p50"], out["lifted_share_max"] = med(lifted), max(lifted, default=float("nan"))
    em = [c["emerged"][0] / n_all for c in big]
    out["emerged_share_p50"], out["emerged_share_max"] = med(em), max(em, default=float("nan"))
    out["uplift_m_values"] = sorted({round(c["dz_max"]) for c in big})
    out["radius_km_p50"] = med([c["r_km"] for c in big])
    for key in ("m10", "m100", "core", "seeds"):
        ex = [extent(c[key]) for c in big]
        ex = [x for x in ex if x]
        out[f"{key}_elongation_p50"] = med([x[3] for x in ex])
        out[f"{key}_elongation_p90"] = pct([x[3] for x in ex], 0.9)
        out[f"{key}_length_km_p50"] = med([x[1] for x in ex])
        out[f"{key}_width_km_p50"] = med([x[2] for x in ex])
    out["elongation_over_3"] = sum(
        1 for c in big if (extent(c["m100"]) or (0, 0, 0, 0))[3] >= 3.0) / max(1, len(big))
    out["contact_age_p50_myr"] = med([c["t"] - c["onset"] for c in big])
    out["convergence_p50_mm_yr"] = med([c["conv"] for c in big])
    # Real overlap against the counted one: the continental crust actually under the other plate
    # (seed samples x a sample's area) over the contact's length, against `depth_km`.
    a_km2 = 4 * math.pi * R_KM ** 2 / n_all
    real = []
    for c in big:
        x = extent(c["seeds"])
        if x:
            real.append(x[0] * a_km2 / x[1])
    out["real_overlap_km_p50"] = med(real)
    # The shortening since the contact began (or since its last collision): the continental
    # crust that went under, counted once per sample, over the contact's length.
    short = []
    for c in big:
        x = extent(c["seeds"])
        if x and "cont_in" in c:
            short.append(c["cont_in"] * a_km2 / x[1])
    out["real_shortening_km_p50"] = med(short)
    out["real_shortening_km_p90"] = pct(short, 0.9)
    out["contact_age_p90_myr"] = pct([c["t"] - c["onset"] for c in big], 0.9)
    out["suture_dying_plate_larger"] = sum(
        c["sutured"] and c["under_samples"] > c["over_samples"] for c in collisions)
    out["counted_depth_km_p50"] = med([c["depth_km"] for c in big])
    out["under_age_at_suture_p50_myr"] = med([c["t"] - c["under_born"] for c in collisions if c["sutured"]])
    out["suture_onto_parent"] = sum(c["sutured"] and c["under_parent"] == c["over"] for c in collisions)
    # What a suture moves: the welded plate's share of the planet.
    moved = [c["under_samples"] / n_all for c in collisions if c["sutured"]]
    out["suture_plate_share_p50"], out["suture_plate_share_max"] = med(moved), max(moved, default=float("nan"))
    return out


def fmt(v):
    if isinstance(v, float):
        return f"{v:.3g}"
    return json.dumps(v) if isinstance(v, (dict, list)) else str(v)


def main(argv):
    label, out_json, runs = "", None, []
    it = iter(argv)
    for a in it:
        if a == "--label":
            label = next(it)
        elif a == "--json":
            out_json = next(it)
        else:
            runs.append(a)
    results = []
    for r in runs:
        try:
            results.append(analyse(r))
        except (OSError, ValueError, KeyError, IndexError) as e:
            print(f"skipped {r}: {e}", file=sys.stderr)
    for res in results:
        print("run " + " ".join(f"{k}={fmt(v)}" for k, v in res.items()))
    if len(results) > 1:
        keys = [k for k, v in results[0].items() if isinstance(v, (int, float)) and k != "steps"]
        line = [f"{label or 'all'} n={len(results)}"]
        for k in keys:
            xs = [r[k] for r in results if not (isinstance(r[k], float) and math.isnan(r[k]))]
            if xs:
                line.append(f"{k}={sum(xs) / len(xs):.3g}[{min(xs):.3g},{max(xs):.3g}]")
        print("mean " + " ".join(line))
    if out_json:
        json.dump(results, open(out_json, "w"), indent=1)


if __name__ == "__main__":
    main(sys.argv[1:])
