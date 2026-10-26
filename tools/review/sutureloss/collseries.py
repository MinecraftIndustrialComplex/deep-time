#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The collision fronts' crust loss, decomposed, from the pool's cached crust budgets.

  python3 -I tools/review/sutureloss/collseries.py <arm> [<arm> ...] [--pool DIR] [--series]

Reads every run of each arm (pool/arms/<arm>.json) and, from the crust budget's `cb_globals`
lines (cumulative over the run, a snapshot every 50 Myr), prints in km^3/yr over the run:
the crust crossing the collision fronts (`crossed`), what stayed (`kept`: belts, wedges and the
overriding plates' interiors), and the loss (crossed - kept) split into PLT-242's share (the
rest), the belt's no room with room left (`noroom_sliver`), with none (`noroom_capped`) and the
ceilings' clip; then the loss as a product, front length x convergence x crust thickness x lost
fraction, and between the first arm and each other the change's log shares of those factors.
`--series` adds each 50-Myr interval's front, convergence and losses (mean over the runs).
"""
import json
import math
import re
import sys

NUM = r"(-?[0-9.]+(?:e[+-]?[0-9]+)?|NaN|nan|inf|-inf)"


def arm_runs(pool, arm):
    d = json.load(open(f"{pool}/arms/{arm}.json"))
    base = f"{pool}/{d['key']}/{d['preset']}-{d['length']}/{d['argkey']}"
    return d["length"], [(f"{s}-{n}", f"{base}/{s}-{n}") for s, n in d["runs"]]


def snaps(path):
    out = []
    with open(f"{path}/summary.txt") as f:
        for line in f:
            m = re.match(r"cb_globals t=(-?[0-9.]+)", line)
            if m:
                g = {
                    k: float(v)
                    for k, v in re.findall(r"([A-Za-z0-9_/]+)=" + NUM + r"(?=\s|$)", line[m.end():])
                }
                out.append((float(m.group(1)), g))
    return out


def parts(g):
    c, k = g.get("crossed", 0.0), g.get("kept", 0.0)
    sl, ca, cl = (g.get(f"noroom_{x}", 0.0) for x in ("sliver", "capped", "clipped"))
    return {
        "crossed": c,
        "kept": k,
        "loss": c - k,
        "share": c - k - sl - ca - cl,
        "sliver": sl,
        "capped": ca,
        "clipped": cl,
        "upper": g.get("upper", 0.0),
        "wedge": g.get("kept_wedge", 0.0),
        "area": g.get("crossed_km2", 0.0),
        "front": g.get("front_km_myr", 0.0),
        "toe": g.get("taper_toe", 0.0),
    }


def mean(xs):
    xs = [x for x in xs if x == x]
    return sum(xs) / len(xs) if xs else float("nan")


def summarize(length, runs):
    rows = []
    for name, path in runs:
        s = snaps(path)
        if not s:
            continue
        p = parts(s[-1][1])
        yrs = length * 1e6
        r = {k: v / yrs for k, v in p.items() if k not in ("area", "front")}
        r["front_km"] = p["front"] / length
        r["conv_mm_yr"] = p["area"] / p["front"] if p["front"] > 0 else float("nan")
        r["h_km"] = p["crossed"] / p["area"] if p["area"] > 0 else float("nan")
        r["lost_frac"] = p["loss"] / p["crossed"] if p["crossed"] > 0 else float("nan")
        r["per_km"] = p["loss"] / p["front"] if p["front"] > 0 else float("nan")
        r["name"] = name
        r["series"] = s
        rows.append(r)
    return rows


COLS = ["crossed", "kept", "loss", "share", "sliver", "capped", "clipped", "upper", "wedge",
        "front_km", "conv_mm_yr", "h_km", "lost_frac", "per_km"]


def main():
    args = sys.argv[1:]
    pool = "/data/ban/deeptime-runs/pool"
    if "--pool" in args:
        i = args.index("--pool")
        pool = args[i + 1]
        del args[i:i + 2]
    series = "--series" in args
    args = [a for a in args if a != "--series"]
    arms = {}
    for a in args:
        length, runs = arm_runs(pool, a)
        arms[a] = summarize(length, runs)
    print("# collision fronts, mean over each arm's runs (km3/yr; front km at any time; km3/km/Myr)")
    print(f"{'arm':14s} {'n':>3s} " + " ".join(f"{c:>9s}" for c in COLS))
    for a, rows in arms.items():
        print(f"{a:14s} {len(rows):3d} " + " ".join(f"{mean([r[c] for r in rows]):9.3f}" for c in COLS))
    for a, rows in arms.items():
        for start in ("supercontinent", "scattered_continents"):
            sub = [r for r in rows if r["name"].startswith(start)]
            if sub:
                label = a[:9] + "/" + start[:4]
                print(f"{label:14s} {len(sub):3d} " + " ".join(f"{mean([r[c] for r in sub]):9.3f}" for c in COLS))
    names = list(arms)
    if len(names) > 1:
        b = arms[names[0]]
        mb = {c: mean([r[c] for r in b]) for c in COLS}
        for a in names[1:]:
            ma = {c: mean([r[c] for r in arms[a]]) for c in COLS}
            tot = math.log(ma["loss"] / mb["loss"])
            print(f"# {names[0]} -> {a}: loss x{ma['loss'] / mb['loss']:.3f} (log {tot:.3f}); log shares of the change:")
            for c in ("front_km", "conv_mm_yr", "h_km", "lost_frac"):
                x = math.log(ma[c] / mb[c])
                print(f"   {c:12s} x{ma[c] / mb[c]:.3f}  log {x:+.3f}  share {x / tot:+.2f}")
            print("   by part (km3/yr): " + ", ".join(
                f"{c} {mb[c]:.3f} -> {ma[c]:.3f} ({ma[c] - mb[c]:+.3f})" for c in ("share", "sliver", "capped", "clipped")))
            print("   as shares of crossed: " + ", ".join(
                f"{c} {mb[c] / mb['crossed']:.4f} -> {ma[c] / ma['crossed']:.4f}" for c in ("share", "sliver", "capped", "upper", "wedge")))
    if series:
        for a, rows in arms.items():
            print(f"# {a}: each 50-Myr interval, mean over runs: front km, conv mm/yr, crossed, share, sliver, "
                  "capped (km3/yr), sliver/crossed, capped/crossed")
            n = min(len(r["series"]) for r in rows)
            for i in range(1, n):
                vals = []
                for r in rows:
                    (t0, g0), (t1, g1) = r["series"][i - 1], r["series"][i]
                    p0, p1 = parts(g0), parts(g1)
                    dt = t1 - t0
                    d = {k: p1[k] - p0[k] for k in p1}
                    vals.append((
                        d["front"] / dt,
                        d["area"] / d["front"] if d["front"] > 0 else float("nan"),
                        d["crossed"] / dt / 1e6,
                        d["share"] / dt / 1e6,
                        d["sliver"] / dt / 1e6,
                        d["capped"] / dt / 1e6,
                        d["sliver"] / d["crossed"] if d["crossed"] > 0 else float("nan"),
                        d["capped"] / d["crossed"] if d["crossed"] > 0 else float("nan"),
                    ))
                t = rows[0]["series"][i][0]
                print(f"   t={t:7.0f} " + " ".join(f"{mean([v[j] for v in vals]):9.3f}" for j in range(8)))


if __name__ == "__main__":
    main()
