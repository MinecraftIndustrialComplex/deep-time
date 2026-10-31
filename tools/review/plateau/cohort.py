# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau lane: where a collision plateau's crust goes, erosion against the plates' thinning (VM, cached runs).

Usage: python3 -I cohort.py <arm folder> [<arm folder> ...]      (MODE=chain for the largest event per run only)

From the crust-budget maps every 50 Myr (`maps/t<Myr>.bin`, MAP_FIELDS, and `maps/pools/t<Myr>.bin`, POOL_FIELDS of
`plates/crust_budget.rs`) and the sea of the dynamics record's nearest step (`rec.json`).

A collision stamps its belt's samples with the step's time (orogeny 2, `orogeny_age` = t - stamp), and the stamp
stays until another orogeny stamps the sample again, so the samples stamped at one step are one body of crust from
snapshot to snapshot, whatever their plate does in between. The high ground's cohort at t: every continental sample
with orogeny 2 whose stamp is one of the stamps of the samples standing more than HI m over the sea at t (default
3,000); the same cohort at t + 50 is every continental sample with orogeny 2 and one of those stamps then.

The two ways the model takes such crust away leave different marks in its rock pools: the surface's erosion strips the
column from the top (`pools::strip`, EROSION_ORDER: sediment, sedimentary, felsic and mafic volcanic, metamorphic,
felsic plutonic; never the mafic plutonic lower crust), while the plates' collapse (`plates/collapse.rs`: "the rock
moved is the collapsing column's own mix, thinned in proportion") and the lower-crustal flow move the column's own
mix. So the mafic plutonic pool falls only by the plates' share: K = ln(MP_t / MP_t+50) of the cohort's mean, the
plates' thinning over the 50 Myr about K x the mean thickness, and the rest of the thickness lost the top's (erosion
less deposition). The founder (crust over 70 km) also takes the bottom first and would count with the plates.
Limits: a resampling's blend mixes a cohort's samples with their neighbours'; crust a cohort sample gains from
another (the flow) counts against its loss. Means per sample, so the cohort's samples restamped by a later orogeny
leave it (n_t+50 / n_t says how many stayed).
"""
import json
import os
import statistics
import sys
from pathlib import Path

import numpy as np

F = 11  # lat, lon, z, thickness, kind, orogeny, orogeny_age, arc_oc, arc_co, group, flags
P = 9  # sediment, sedimentary, metamorphic, felsic_plutonic, felsic_volcanic, mafic_volcanic, mafic_plutonic, basement, formed
POOLS = ["sed", "sedim", "meta", "fplut", "fvolc", "mvolc", "mplut"]
HI = float(os.environ.get("HI", "3000"))
MODE = os.environ.get("MODE", "pairs")


def seas_of(run):
    rec = json.load(open(run / "rec.json"))
    return {round(s["t"]): float(s.get("sea_m") or 0.0) for s in rec.get("steps", [])}


def near(seas, t):
    return seas[min(seas, key=lambda x: abs(x - t))]


def load(run, t):
    name = f"t{t:04.0f}.bin" if t >= 0 else f"t{t:04.0f}.bin"
    a = run / "maps" / name
    p = run / "maps" / "pools" / name
    if not a.exists() or not p.exists():
        return None
    m = np.fromfile(a, "<f4").reshape(-1, F).astype(np.float64)
    q = np.fromfile(p, "<f4").reshape(-1, P).astype(np.float64)
    if len(m) != len(q):
        return None
    return m, q


def stamps(m, t):
    return np.rint(np.nan_to_num(t - m[:, 6], nan=1e9) * 10.0).astype(np.int64)


def cohort(m, t, keys):
    coll = (m[:, 4] > 0.5) & (m[:, 5] == 2)
    return coll & np.isin(stamps(m, t), keys)


def stats(m, q, sel, sea):
    n = int(sel.sum())
    if n == 0:
        return None
    z = m[sel, 2] - sea
    out = dict(n=n, T=float(m[sel, 3].mean()), z=float(z.mean()), hi=float((z > HI).mean()),
               hi4=float((z > 4000).mean()))
    for k, name in enumerate(POOLS):
        out[name] = float(q[sel, k].mean())
    return out


def budget(a, b, dt):
    """The cohort's thinning between two snapshots: the plates' (in proportion), the top's (the rest), m/Myr."""
    if not a or not b or a["mplut"] <= 0 or b["mplut"] <= 0:
        return None
    k = np.log(a["mplut"] / b["mplut"])
    kf = np.log(a["fplut"] / b["fplut"]) if a["fplut"] > 0 and b["fplut"] > 0 else float("nan")
    tm = 0.5 * (a["T"] + b["T"])
    plates = k * tm
    top = (a["T"] - b["T"]) - plates
    return dict(plates=plates / dt, top=top / dt, total=(a["T"] - b["T"]) / dt, dz=(b["z"] - a["z"]) / dt,
                k_mp=k, k_fp=kf, kept=b["n"] / a["n"],
                lost_top={name: (a[name] - b[name]) / dt for name in POOLS[:6]})


def times(run):
    ts = []
    for p in (run / "maps").glob("t*.bin"):
        try:
            ts.append(int(round(float(p.stem[1:]))))
        except ValueError:
            pass
    return sorted(ts)


def run_pairs(run):
    seas = seas_of(run)
    ts = [t for t in times(run) if t >= -950]
    rows = []
    cache = {}

    def get(t):
        if t not in cache:
            cache.clear() if len(cache) > 1 else None
            cache[t] = load(run, t)
        return cache[t]

    for t1 in ts:
        t2 = t1 + 50
        if t2 not in ts:
            continue
        x1, x2 = get(t1), get(t2)
        if x1 is None or x2 is None:
            continue
        (m1, q1), (m2, q2) = x1, x2
        s1, s2 = near(seas, t1), near(seas, t2)
        hi = (m1[:, 4] > 0.5) & (m1[:, 5] == 2) & (m1[:, 2] - s1 > HI)
        if hi.sum() < 20:
            continue
        age = m1[hi, 6]
        keys = np.unique(stamps(m1, t1)[hi])
        a = stats(m1, q1, cohort(m1, t1, keys), s1)
        b = stats(m2, q2, cohort(m2, t2, keys), s2)
        bud = budget(a, b, 50.0)
        if bud:
            rows.append(dict(t=t1, n_hi=int(hi.sum()), young=float((age < 10.0).mean()), a=a, b=b, **bud))
            if os.environ.get("VERBOSE"):
                print(f"    {run.name} {t1}..{t2}: high {int(hi.sum())} (young {rows[-1]['young']:.2f}) kept "
                      f"{bud['kept']:.2f}; T {a['T'] / 1000:.1f} -> {b['T'] / 1000:.1f} km, z {a['z']:.0f} -> {b['z']:.0f} m; "
                      f"plates {bud['plates']:.0f} top {bud['top']:.0f} m/Myr")
    return rows


def chain(run):
    """The largest event above 4 km: its cohort at t*, t* + 50 and t* + 100."""
    seas = seas_of(run)
    best = None
    for t in [t for t in times(run) if t >= -950]:
        x = load(run, t)
        if x is None:
            continue
        m, _ = x
        hi4 = (m[:, 4] > 0.5) & (m[:, 2] - near(seas, t) > 4000)
        if best is None or hi4.sum() > best[1]:
            best = (t, int(hi4.sum()))
    if not best or best[1] < 20:
        return None
    t0 = best[0]
    m0, q0 = load(run, t0)
    s0 = near(seas, t0)
    hi4 = (m0[:, 4] > 0.5) & (m0[:, 5] == 2) & (m0[:, 2] - s0 > 4000)
    keys = np.unique(stamps(m0, t0)[hi4])
    out = [(t0, best[1], float((m0[hi4, 6] < 10).mean()), stats(m0, q0, cohort(m0, t0, keys), s0))]
    for dt in (50, 100):
        x = load(run, t0 + dt)
        if x is None:
            break
        m, q = x
        out.append((t0 + dt, None, None, stats(m, q, cohort(m, t0 + dt, keys), near(seas, t0 + dt))))
    return out


def fmt(s):
    return (f"n {s['n']:5d} T {s['T'] / 1000:5.1f} km z {s['z']:6.0f} m above {HI / 1000:.0f} km {s['hi']:.2f} above 4 km "
            f"{s['hi4']:.2f} pools(km) " + "/".join(f"{s[k] / 1000:.1f}" for k in POOLS))


def main():
    for arm in sys.argv[1:]:
        runs = [r for r in sorted(Path(arm).iterdir()) if r.is_dir() and (r / "maps").is_dir()]
        print(f"== {arm} ({len(runs)} runs) HI {HI:.0f} m")
        if MODE == "chain":
            for r in runs:
                c = chain(r)
                if not c:
                    print(f"  {r.name}: no event above 4 km")
                    continue
                t0, n4, young, s = c[0]
                print(f"  {r.name}: largest above 4 km at t {t0} ({n4} samples, stamped under 10 Myr ago {young:.2f})")
                for t, _, _, s in c:
                    print(f"    t {t:5d} {fmt(s) if s else 'empty'}")
                for (ta, _, _, sa), (tb, _, _, sb) in zip(c, c[1:]):
                    b = budget(sa, sb, float(tb - ta))
                    if b:
                        print(f"    {ta}..{tb}: thinning m/Myr total {b['total']:.0f} = plates (in proportion) "
                              f"{b['plates']:.0f} + top {b['top']:.0f}; surface {b['dz']:+.1f} m/Myr; cohort kept "
                              f"{b['kept']:.2f}; K mafic {b['k_mp']:.3f} felsic plutonic {b['k_fp']:.3f}")
            continue
        allrows = []
        for r in runs:
            rows = run_pairs(r)
            allrows += rows
            if rows:
                w = sum(x["n_hi"] for x in rows)
                pl = sum(x["n_hi"] * x["plates"] for x in rows) / w
                tp = sum(x["n_hi"] * x["top"] for x in rows) / w
                print(f"  {r.name:24s} pairs {len(rows):2d} high-ground weight {w:6d}: plates {pl:6.1f} top {tp:6.1f} m/Myr")
        if not allrows:
            continue
        w = sum(x["n_hi"] for x in allrows)

        def wmean(key, rows=allrows, w=w):
            return sum(x["n_hi"] * x[key] for x in rows) / w

        print(f"  all pairs {len(allrows)}, weighted by the high ground at t: thinning {wmean('total'):.1f} m/Myr = plates "
              f"{wmean('plates'):.1f} + top {wmean('top'):.1f}; surface {wmean('dz'):+.1f} m/Myr; kept {wmean('kept'):.2f}")
        for lo, hi in ((0.0, 0.25), (0.25, 0.75), (0.75, 1.01)):
            sub = [x for x in allrows if lo <= x["young"] < hi]
            if sub:
                ws = sum(x["n_hi"] for x in sub)
                print(f"    high ground stamped under 10 Myr ago {lo:.2f}..{min(hi, 1):.2f}: pairs {len(sub)}, weight {ws}: "
                      f"plates {sum(x['n_hi'] * x['plates'] for x in sub) / ws:.1f} top "
                      f"{sum(x['n_hi'] * x['top'] for x in sub) / ws:.1f} m/Myr")
        clean = [x for x in allrows if x["kept"] >= 0.8]
        if clean:
            wc = sum(x["n_hi"] for x in clean)
            print(f"    cohorts that kept 0.8 of their samples or more: pairs {len(clean)}, weight {wc}: plates "
                  f"{sum(x['n_hi'] * x['plates'] for x in clean) / wc:.1f} top {sum(x['n_hi'] * x['top'] for x in clean) / wc:.1f} "
                  f"m/Myr, surface {sum(x['n_hi'] * x['dz'] for x in clean) / wc:+.1f} m/Myr")
        tops = {k: sum(x["n_hi"] * x["lost_top"][k] for x in allrows) / w for k in POOLS[:6]}
        print("    pools' mean loss, m/Myr: " + ", ".join(f"{k} {v:.1f}" for k, v in tops.items()))
        kd = [x["k_fp"] - x["k_mp"] for x in allrows if x["k_fp"] == x["k_fp"]]
        if kd:
            print(f"    K felsic plutonic - K mafic plutonic: median {statistics.median(kd):+.3f} (0 if the top never "
                  f"reached the felsic plutonic pool)")


if __name__ == "__main__":
    main()
