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
Also: what PLT-700 kept (`noroom_stays`), the founder's and the rate limit's sinks
(`foundered`, `rate_lost`), and the dock tally's splits (PLT-515, PLT-701).
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


def dock(path):
    """The dock tally's front-time (km x Myr) by the overriding side's oceanic share, summed over
    the state-age bins, and its front-time-weighted convergence."""
    bins, w, cw = [0.0] * 4, 0.0, 0.0
    found = False
    with open(f"{path}/summary.txt") as f:
        for line in f:
            m = re.match(r"dock_age=\S+ km_myr=(\S+) means=(\S+)", line)
            if m:
                found = True
                km = [float(x) for x in m.group(1).split(",")]
                means = m.group(2).split(",")
                for i in range(4):
                    bins[i] += km[i]
                s = sum(km)
                c = float(means[4]) if means[4] != "nan" else 0.0
                w += s
                cw += s * c
    return (bins, cw / w if w > 0 else float("nan")) if found else None


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
        "stays": g.get("noroom_stays", 0.0),
        "foundered": g.get("foundered", 0.0),
        "rate_lost": g.get("rate_lost", 0.0),
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
        r["path"] = path
        r["series"] = s
        rows.append(r)
    return rows


COLS = ["crossed", "kept", "loss", "share", "sliver", "capped", "clipped", "upper", "wedge",
        "front_km", "conv_mm_yr", "h_km", "lost_frac", "per_km", "stays", "foundered", "rate_lost"]


def noroom(path):
    """The no-room split by belt class (PLT-701; whole run, km^3): {class: [front_km_myr,
    crossed, sliver, capped, stays]}, or None before the split was printed."""
    out = {}
    with open(f"{path}/summary.txt") as f:
        for line in f:
            m = re.match(r"dock_noroom_(empty|narrow|wide) (\S+)", line)
            if m:
                out[m.group(1)] = [float(x) for x in m.group(2).split(",")]
    return out or None


def main():
    args = sys.argv[1:]
    pool = "/data/ban/deeptime-runs/pool"
    if "--pool" in args:
        i = args.index("--pool")
        pool = args[i + 1]
        del args[i:i + 2]
    series = "--series" in args
    args = [a for a in args if a != "--series"]
    arms, arm_len = {}, {}
    for a in args:
        length, runs = arm_runs(pool, a)
        arms[a] = summarize(length, runs)
        arm_len[a] = length
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
    print("# the dock tally (PLT-515, after the first 100 Myr): share of the collision front-time by the "
          "overriding side's oceanic share within the belt's reach (0-0.25, 0.25-0.5, 0.5-0.75, 0.75-1), "
          "and the front-time's mean convergence (mm/yr)")
    for a, rows in arms.items():
        bins, conv = [0.0] * 4, []
        for r in rows:
            d = dock(r["path"])
            if d is None:
                continue
            for i in range(4):
                bins[i] += d[0][i]
            conv.append(d[1])
        tot = sum(bins)
        if tot > 0:
            print(f"{a:14s} " + " ".join(f"{b / tot:7.3f}" for b in bins) + f"   conv {mean(conv):6.2f}")
    print("# the no-room split by belt class (PLT-701, whole run): share of front-time, and km3/yr "
          "crossed, left over with room (sliver), at the strength height (capped), kept by PLT-700 (stays)")
    for a, rows in arms.items():
        tot = {c: [0.0] * 5 for c in ("empty", "narrow", "wide")}
        n = 0
        for r in rows:
            d = noroom(r["path"])
            if d is None:
                continue
            n += 1
            for c, v in d.items():
                tot[c] = [x + y for x, y in zip(tot[c], v)]
        if n:
            yrs = n * arm_len[a] * 1e6
            ft = sum(v[0] for v in tot.values())
            for c, v in tot.items():
                print(f"{a:14s} {c:7s} front {v[0] / ft if ft > 0 else float('nan'):6.3f}  " + " ".join(
                    f"{k} {x / yrs:7.3f}" for k, x in zip(("crossed", "sliver", "capped", "stays"), v[1:])))
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
