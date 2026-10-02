#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Open-loop force diagnostics (docs/design/plate-force-balance.md): what a torque balance would
make of the plate geometry of a run of the present model, without changing the run.

    python3 tools/review/plate-forces.py [--json out.json] <run> [<run> ...]

With `resist`, a drag on motion towards the trench is added to the left side (slab resistance).

<run>.trace holds one `forces` line per step (tools/review/plate-dynamics-trace.patch): per plate
its visible oceanic and continental samples, boundary lengths in samples (slab, slab weighted by
sqrt(age/80 Myr), overriding, ridge, collision, whole edge), the two drag tensors, its present
angular velocity (nrad/yr) and the torque arms of slab pull, slab suction, ridge push and
collision. For each parameter set the balance

    (D_ocean + keel * D_continent) W = slab * T_slab_age + ridge * T_ridge + suction * T_suck

is solved per plate and step, and W is scaled so that the area-weighted RMS speed of all plates is
`RMS` mm/yr. Reported: the spread of plate speeds, speed against the share of a plate's edge that
is slab, oceanic against continental plates, how much the answer jumps from one step to the next,
and how often it is a spin about a pole inside the plate. The same for the run's own speeds.
Standard library only.
"""
import json
import math
import statistics
import sys

R_MM = 6370.0e6  # mm
RMS = 45.0
SETS = {
    "slab only, keel 1": dict(slab=1.0, ridge=0.0, suction=0.0, keel=1.0),
    "slab + ridge 0.1, keel 1": dict(slab=1.0, ridge=0.1, suction=0.0, keel=1.0),
    "slab + ridge 0.1, keel 3": dict(slab=1.0, ridge=0.1, suction=0.0, keel=3.0),
    "slab + ridge 0.1 + suction 0.3, keel 3": dict(slab=1.0, ridge=0.1, suction=0.3, keel=3.0),
    "slab + ridge 0.3 + suction 0.3, keel 3": dict(slab=1.0, ridge=0.3, suction=0.3, keel=3.0),
    # Slab resistance: a drag on the motion towards the trench, per trench length. The number is
    # its size against the basal drag of an average starting plate with a 3,000 km trench.
    "ridge 0.1 + suction 0.3, keel 3, slab resistance 3": dict(slab=1.0, ridge=0.1, suction=0.3, keel=3.0, resist=3.0),
    "ridge 0.1 + suction 0.3, keel 3, slab resistance 10": dict(slab=1.0, ridge=0.1, suction=0.3, keel=3.0, resist=10.0),
    "ridge 0.3 + suction 0.3, keel 3, slab resistance 10": dict(slab=1.0, ridge=0.3, suction=0.3, keel=3.0, resist=10.0),
    # Boundary resistance: a drag per length of plate edge (against a fixed frame here; in the
    # model it would act on the motion relative to the neighbour). The number is its size against
    # the basal drag of an average starting plate.
    "ridge 0.1 + suction 0.3, keel 3, slab resistance 3, edge 1": dict(slab=1.0, ridge=0.1, suction=0.3, keel=3.0, resist=3.0, edge=1.0),
    "ridge 0.1 + suction 0.3, keel 3, slab resistance 3, edge 3": dict(slab=1.0, ridge=0.1, suction=0.3, keel=3.0, resist=3.0, edge=3.0),
    "ridge 0.1 + suction 0.3, keel 3, slab resistance 10, edge 3": dict(slab=1.0, ridge=0.1, suction=0.3, keel=3.0, resist=10.0, edge=3.0),
}


def sym(m):
    a, b, c, d, e, f = m
    return [[a, d, e], [d, b, f], [e, f, c]]


def solve3(m, t):
    (a, b, c), (d, e, f), (g, h, i) = m
    det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    if abs(det) < 1e-12:
        return None
    inv = [[(e * i - f * h), (c * h - b * i), (b * f - c * e)],
           [(f * g - d * i), (a * i - c * g), (c * d - a * f)],
           [(d * h - e * g), (b * g - a * h), (a * e - b * d)]]
    return [sum(inv[r][k] * t[k] for k in range(3)) / det for r in range(3)]


def quad(m, w):
    return sum(w[r] * m[r][k] * w[k] for r in range(3) for k in range(3))


def pct(xs, q):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    k = (len(xs) - 1) * q
    lo, hi = int(math.floor(k)), int(math.ceil(k))
    return xs[lo] + (xs[hi] - xs[lo]) * (k - lo)


def ranks(xs):
    order = sorted(range(len(xs)), key=lambda i: xs[i])
    r = [0.0] * len(xs)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and xs[order[j + 1]] == xs[order[i]]:
            j += 1
        for k in range(i, j + 1):
            r[order[k]] = (i + j) / 2.0
        i = j + 1
    return r


def spearman(xs, ys):
    if len(xs) < 3:
        return float("nan")
    rx, ry = ranks(xs), ranks(ys)
    mx, my = sum(rx) / len(rx), sum(ry) / len(ry)
    sx = math.sqrt(sum((a - mx) ** 2 for a in rx))
    sy = math.sqrt(sum((b - my) ** 2 for b in ry))
    if sx == 0 or sy == 0:
        return float("nan")
    return sum((a - mx) * (b - my) for a, b in zip(rx, ry)) / (sx * sy)


def summarise(rows, prev_key="w"):
    """rows: per plate-step dicts with speed (mm/yr), area, slab share, continental share, spin
    flag, and the relative jump from the plate's last step."""
    sp = [r["speed"] for r in rows]
    big = [r for r in rows if r["area"] >= 0.01]
    out = {
        "plate_steps": len(rows),
        "speed_p10": pct(sp, 0.1), "speed_p50": pct(sp, 0.5), "speed_p90": pct(sp, 0.9),
        "speed_max": max(sp),
        "over_150": sum(s > 150 for s in sp) / len(sp),
        "under_5": sum(s < 5 for s in sp) / len(sp),
        "speed_vs_slab_share_spearman": spearman([r["slab_share"] for r in big], [r["speed"] for r in big]),
        "speed_vs_area_spearman": spearman([r["area"] for r in big], [r["speed"] for r in big]),
        "speed_vs_continental_spearman": spearman([r["cont"] for r in big], [r["speed"] for r in big]),
        "oceanic_p50": pct([r["speed"] for r in big if r["cont"] < 0.2], 0.5),
        "continental_p50": pct([r["speed"] for r in big if r["cont"] > 0.5], 0.5),
        "slab_over_20pct_p50": pct([r["speed"] for r in big if r["slab_share"] >= 0.2], 0.5),
        "slab_under_5pct_p50": pct([r["speed"] for r in big if r["slab_share"] < 0.05], 0.5),
        "spin_share": sum(r["spin"] for r in rows) / len(rows),
    }
    jumps = [r["jump"] for r in rows if r["jump"] is not None]
    out["step_jump_p50"] = pct(jumps, 0.5)
    out["step_jump_p90"] = pct(jumps, 0.9)
    return out


def analyse(run):
    steps = []
    for line in open(run + ".trace"):
        if '"k":"forces"' in line:
            steps.append(json.loads(line))
    out = {"run": run, "steps": len(steps)}
    res = {name: [] for name in list(SETS) + ["the run itself"]}
    last = {name: {} for name in res}
    no_drive = 0
    total = 0
    for s in steps:
        n_all = sum(sum(p["n"]) for p in s["plates"])
        per_set = {name: [] for name in res}
        for p in s["plates"]:
            n = sum(p["n"])
            if n < 30:
                continue
            total += 1
            d_o, d_c = sym(p["drag"][:6]), sym(p["drag"][6:])
            d1 = [[d_o[r][k] + d_c[r][k] for k in range(3)] for r in range(3)]
            norm_c = math.sqrt(sum(x * x for x in p["slab"])) + math.sqrt(sum(x * x for x in p["ridge"]))
            if norm_c == 0:
                no_drive += 1
            base = dict(id=p["id"], area=n / n_all, cont=p["n"][1] / n,
                        slab_share=p["len"][0] / max(1.0, p["len"][5]), d1=d1, n=n)
            # Centroid of the plate from the drag tensor's trace is not available; the spin test
            # uses the share of the motion that the lowest-drag axis carries instead.
            for name, k in SETS.items():
                dm = [[d_o[r][c] + k["keel"] * d_c[r][c] for c in range(3)] for r in range(3)]
                if k.get("edge"):
                    n_ref = n_all / 14.0
                    edge_ref = 2.0 * math.pi * math.sqrt(n_ref / math.pi)
                    boost = k["edge"] * (n_ref / edge_ref) * p["len"][5] / n
                    dm = [[dm[r][c] + boost * d1[r][c] for c in range(3)] for r in range(3)]
                if k.get("resist") and p["len"][0] > 0:
                    # Rank one: the trench's summed arm (exact for a straight trench).
                    cs = k["resist"] * (n_all / 14.0) / 42.0 / p["len"][0]
                    a = p["slab"]
                    dm = [[dm[r][c] + cs * a[r] * a[c] for c in range(3)] for r in range(3)]
                t = [k["slab"] * p["slab_age"][i] + k["ridge"] * p["ridge"][i] + k["suction"] * p["suck"][i]
                     for i in range(3)]
                w = solve3(dm, t)
                if w is None:
                    continue
                per_set[name].append(dict(base, w=w))
            per_set["the run itself"].append(dict(base, w=p["w"]))
        for name, rows in per_set.items():
            if not rows:
                continue
            ms = sum(quad(r["d1"], r["w"]) for r in rows) / sum(r["n"] for r in rows)
            if ms <= 0:
                continue
            # The run's own speeds are kept as they are (nrad/yr -> mm/yr); a balance is scaled.
            scale = R_MM * 1e-9 if name == "the run itself" else RMS / math.sqrt(ms)
            for r in rows:
                w = [x * scale for x in r["w"]]
                speed = math.sqrt(max(0.0, quad(r["d1"], w) / r["n"]))
                # |W| at 90 degrees from the pole against the plate's RMS speed: a plate turning
                # about a pole inside itself has a large |W| for its speed.
                wn = math.sqrt(sum(x * x for x in w))
                spin = speed > 0 and wn / speed > 2.5
                prev = last[name].get(r["id"])
                jump = None
                if prev is not None:
                    pn = math.sqrt(sum(x * x for x in prev))
                    if pn > 0 and wn > 0:
                        jump = math.sqrt(sum((a - b) ** 2 for a, b in zip(w, prev))) / max(pn, wn)
                last[name][r["id"]] = w
                res[name].append(dict(speed=speed, area=r["area"], cont=r["cont"],
                                      slab_share=r["slab_share"], spin=spin, jump=jump))
    out["no_slab_no_ridge_share"] = no_drive / max(1, total)
    for name, rows in res.items():
        if rows:
            out[name] = summarise(rows)
    return out


def main(argv):
    out_json, runs = None, []
    it = iter(argv)
    for a in it:
        if a == "--json":
            out_json = next(it)
        else:
            runs.append(a)
    results = [analyse(r) for r in runs]
    for res in results:
        print(res["run"], "steps", res["steps"], "plates with no slab and no ridge",
              round(res["no_slab_no_ridge_share"], 3))
        for name in list(SETS) + ["the run itself"]:
            if name in res:
                print("  " + name + ": " + " ".join(
                    f"{k}={v:.3g}" for k, v in res[name].items()))
    if out_json:
        json.dump(results, open(out_json, "w"), indent=1)


if __name__ == "__main__":
    main(sys.argv[1:])
