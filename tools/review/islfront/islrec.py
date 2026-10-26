"""The island front from cached runs' records (measurement only, the island-front diagnosis,
2026-10-26): islrec.py <folder>[@N] ...

Per run, the mean over every step with front bands of the dynamics record's boundary mix
(one lattice spacing a front sample) and its front bands, split as the record splits them:
setting 0 (ocean-ocean: the overriding sample oceanic with no continental crust of its own plate
within the arc's distance, PLT-370; the crust budget's island front) and setting 1. Beside them
the crust budget's front_o / front_c (straight km, PLT-530) as run means and per 100 Myr window
(the cb_globals lines are cumulative at each snapshot). Records made since PLT-683 (2026-10-26) add the
cross-tabs: the part of the incipient front that is incipient by the slab's thermal floor alone (`slow`,
PLT-681), the incipient front by its convergence band, the subducting front by its sample's convergence
band against its pair of plates' mean (`normal_pair`), the island front by pair origin and band, the
front's turnover per step (`gain`, `loss`, `gain_slab`) and the contacts maturing (`matured`).
"""
import glob
import importlib.util
import json
import os
import re
import statistics
import sys

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    "lossbudget", os.path.join(here, "..", "loss-budget.py")
)
lb = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lb)

MIX = ["oo", "oc", "coll", "ridge", "rift", "other"]
SLAB = [0, 50, 100, 150, 300, 600]
AGE = [0, 6, 12, 24, 50, 100]
NORMAL = [0, 5, 10, 20, 40, 80]
CONT = [0.0, 0.01, 0.05, 0.15, 0.3, 0.5]
COAST = [0, 150, 300, 600, 1200, 2400]
PAIR = ["initiation", "rift", "other"]


def runs(arg):
    folder, _, n = arg.partition("@")
    out = []
    for p in sorted(glob.glob(f"{folder}/*-*/rec.json")):
        seed = int(p.rsplit("/", 2)[1].rsplit("-", 1)[1])
        if n and seed > int(n):
            continue
        out.append(os.path.dirname(p))
    return out


def globals_series(path):
    """[(t, globals dict)] from every cb_globals line of the summary."""
    out = []
    with open(path) as f:
        for line in f:
            m = re.match(r"cb_globals t=(-?[0-9.]+)", line)
            if m:
                out.append((float(m.group(1)), lb.kv(line[m.end():])))
    return out


def add(acc, b):
    """Adds the record's nested lists of numbers into `acc`, key by key."""
    def rec(a, v):
        for i in range(len(a)):
            if isinstance(a[i], list):
                rec(a[i], v[i])
            else:
                a[i] += v[i]
    for k in acc:
        rec(acc[k], b[k])


def scale(acc, f):
    def rec(a):
        return [rec(v) for v in a] if isinstance(a, list) else a * f
    return {k: rec(v) for k, v in acc.items()}


def mean_nested(xs):
    if isinstance(xs[0], list):
        return [mean_nested([x[i] for x in xs]) for i in range(len(xs[0]))]
    return statistics.mean(xs)


def cross_tabs(rows):
    """Run means of the cross-tabs (PLT-683), printed as km per step and shares."""
    rows = [r for r in rows if "x" in r]
    if not rows:
        return
    m = {k: mean_nested([r["x"][k] for r in rows]) for k in rows[0]["x"]}
    print(f"  cross-tabs over {len(rows)} runs (km per step, run means):")
    print("    slow (incipient by the floor alone): island %.0f, continental %.0f" % tuple(m["slow"]))
    for k, name in ((0, "island"), (1, "continental")):
        print(f"    {name} incipient by v_n band: " + "  ".join(
            f"{e}: {v:.0f}" for e, v in zip(NORMAL, m["incipient_normal"][k])))
        t = m["normal_pair"][k]
        print(f"    {name} subducting, rows the sample's v_n band, columns the pair's mean band:")
        print("      " + " ".join(f"{e:>7}" for e in NORMAL))
        for i, e in enumerate(NORMAL):
            print(f"  {e:>4}" + " ".join(f"{v:7.0f}" for v in t[i]))
        u10 = sum(sum(t[i]) for i in range(2))
        u10_fast = sum(sum(t[i][2:]) for i in range(2))
        o10 = sum(sum(t[i]) for i in range(2, 6))
        o10_slow = sum(sum(t[i][:2]) for i in range(2, 6))
        print(f"      under 10 on a pair at 10+: {u10_fast / u10 if u10 else 0:.3f} of {u10:.0f} km;"
              f" at 10+ on a pair under 10: {o10_slow / o10 if o10 else 0:.3f} of {o10:.0f} km")
    for j, name in enumerate(PAIR):
        print(f"    island subducting by origin {name}: " + "  ".join(
            f"{e}: {v:.0f}" for e, v in zip(NORMAL, m["oo_normal_pair"][j])))
    for k, name in ((0, "island"), (1, "continental")):
        print(f"    {name} turnover per step: subducting +{m['gain'][k][1]:.0f} -{m['loss'][k][1]:.0f},"
              f" incipient +{m['gain'][k][0]:.0f} -{m['loss'][k][0]:.0f}; subducting gain by slab band: "
              + "  ".join(f"{e}: {v:.0f}" for e, v in zip(SLAB, m["gain_slab"][k])))
    ev = [e for r in rows for e in r["matured"]]
    rate = statistics.mean([r["matured_per_100myr"] for r in rows])
    if ev:
        print(f"    maturing: {rate:.1f} contacts per 100 Myr; at the step, median front v_n"
              f" {statistics.median([e[0] for e in ev]):.1f} mm/yr, contact age"
              f" {statistics.median([e[1] for e in ev]):.1f} Myr, oceanic front samples"
              f" {statistics.median([e[2] for e in ev]):.0f}")


def one(run):
    d = json.load(open(f"{run}/rec.json"))
    steps = [s for s in d["steps"] if "front_bands" in s]
    n = len(steps)
    o = {"steps": n}
    mix = [0.0] * 6
    acc = {k: [[0.0] * 6, [0.0] * 6] for k in ("slab", "age", "normal")}
    inc = [0.0, 0.0]
    ben = [0.0, 0.0]
    oo = {k: [0.0] * 6 for k in ("oo_over_cont", "oo_under_cont", "oo_coast")}
    pair = [0.0] * 3
    other_conv = 0.0
    xt = {"slow": [0.0, 0.0], "incipient_normal": [[0.0] * 6, [0.0] * 6],
          "normal_pair": [[[0.0] * 6 for _ in range(6)] for _ in range(2)],
          "oo_normal_pair": [[0.0] * 6 for _ in range(3)],
          "gain": [[0.0, 0.0], [0.0, 0.0]], "loss": [[0.0, 0.0], [0.0, 0.0]],
          "gain_slab": [[0.0] * 6, [0.0] * 6]}
    has_x = False
    matured = []
    win = {}
    for s in steps:
        b = s["front_bands"]
        for j in range(6):
            mix[j] += s["mix_km"][j]
        for k in ("slab", "age", "normal"):
            for i in (0, 1):
                for j in range(6):
                    acc[k][i][j] += b[k][i][j]
        for i in (0, 1):
            inc[i] += b.get("incipient", [0, 0])[i]
            ben[i] += b.get("benioff", [0, 0])[i]
        for k in oo:
            for j in range(6):
                oo[k][j] += b.get(k, [0] * 6)[j]
        for j in range(3):
            pair[j] += b.get("oo_pair", [0] * 3)[j]
        other_conv += b.get("other_converging", 0.0)
        if "normal_pair" in b:
            has_x = True
            add(xt, b)
        matured.extend(s.get("matured", []))
        w = int((s["t"] - d["t0_myr"]) // 100)
        x = win.setdefault(w, [0, 0.0, 0.0])
        x[0] += 1
        x[1] += sum(b["slab"][0]) + b.get("incipient", [0, 0])[0]
        x[2] += sum(b["slab"][1]) + b.get("incipient", [0, 0])[1]
    for j, name in enumerate(MIX):
        o["mix_" + name] = mix[j] / n
    o["mix_total"] = sum(mix) / n
    o["other_converging"] = other_conv / n
    for i in (0, 1):
        sub = sum(acc["slab"][i])
        tot = sub + inc[i]
        o[f"front{i}"] = tot / n
        o[f"sub{i}"] = sub / n
        o[f"inc{i}"] = inc[i] / tot if tot else 0.0
        o[f"ben{i}"] = ben[i] / tot if tot else 0.0
        for k in ("slab", "age", "normal"):
            o[f"{k}{i}"] = [x / sub if sub else 0.0 for x in acc[k][i]]
    sub0 = sum(acc["slab"][0])
    for k in oo:
        o[k] = [x / sub0 if sub0 else 0.0 for x in oo[k]]
    o["oo_pair"] = [x / sub0 if sub0 else 0.0 for x in pair]
    o["win_front0"] = {w: x[1] / x[0] for w, x in sorted(win.items())}
    o["win_front1"] = {w: x[2] / x[0] for w, x in sorted(win.items())}
    if has_x:
        o["x"] = scale(xt, 1.0 / n)
        o["matured"] = matured
        span = steps[-1]["t"] - steps[0]["t"] if n > 1 else 0.0
        o["matured_per_100myr"] = 100.0 * len(matured) / span if span else 0.0
    gs = globals_series(f"{run}/summary.txt")
    if gs:
        t_first = d["t0_myr"]
        g = gs[-1][1]
        length = gs[-1][0] - t_first
        o["front_o"] = g.get("front_o", 0.0) / length
        o["front_c"] = g.get("front_c", 0.0) / length
        o["front_straight"] = g.get("front_straight", 0.0) / length
        o["front_young_o"] = g.get("front_young_o", 0.0) / length
        prev_t, prev = t_first, {}
        series = {}
        for t, gg in gs:
            if t <= prev_t:
                continue
            dt = t - prev_t
            series[int((t - t_first) // 1)] = (
                (gg.get("front_o", 0.0) - prev.get("front_o", 0.0)) / dt,
                (gg.get("front_c", 0.0) - prev.get("front_c", 0.0)) / dt,
            )
            prev_t, prev = t, gg
        o["cb_series"] = series
    return o


def show(name, rows, key, edges=None):
    if edges is None:
        xs = [r[key] for r in rows if key in r]
        if not xs:
            return
        print(f"  {key:>16}: mean {statistics.mean(xs):11.4f}  median {statistics.median(xs):11.4f}"
              f"  [{min(xs):.4f}, {max(xs):.4f}]")
    else:
        m = [statistics.mean([r[key][j] for r in rows]) for j in range(len(edges))]
        print(f"  {key:>16}: " + "  ".join(f"{e}: {x:.3f}" for e, x in zip(edges, m)))


def main():
    for arg in sys.argv[1:]:
        rows = [one(r) for r in runs(arg)]
        print(f"== {arg}: {len(rows)} runs, steps {rows[0]['steps'] if rows else 0}")
        for k in ["front_o", "front_c", "front_straight", "front_young_o", "front0", "sub0", "front1",
                  "sub1", "inc0", "inc1", "ben0", "ben1"] + ["mix_" + m for m in MIX] + [
                      "mix_total", "other_converging"]:
            show(arg, rows, k)
        for k, e in (("slab0", SLAB), ("slab1", SLAB), ("age0", AGE), ("age1", AGE), ("normal0", NORMAL),
                     ("normal1", NORMAL), ("oo_over_cont", CONT), ("oo_under_cont", CONT),
                     ("oo_coast", COAST), ("oo_pair", PAIR)):
            show(arg, rows, k, e)
        ws = sorted(rows[0]["win_front0"])
        print("  island front (mix, km) by 100 Myr window: " + " ".join(
            f"{statistics.mean([r['win_front0'].get(w, 0.0) for r in rows]):.0f}" for w in ws))
        print("  continental front (mix, km) by window:   " + " ".join(
            f"{statistics.mean([r['win_front1'].get(w, 0.0) for r in rows]):.0f}" for w in ws))
        cross_tabs(rows)
        if "cb_series" in rows[0]:
            ts = sorted(rows[0]["cb_series"])
            print("  crust budget front_o by 50 Myr window: " + " ".join(
                f"{statistics.mean([r['cb_series'].get(t, (0, 0))[0] for r in rows]):.0f}" for t in ts))
            print("  crust budget front_c by 50 Myr window: " + " ".join(
                f"{statistics.mean([r['cb_series'].get(t, (0, 0))[1] for r in rows]):.0f}" for t in ts))
        key = arg.split("@")[0].rstrip("/").split("/")[-3]
        json.dump(rows, open(f"islrec_{key}.json", "w"))  # in the working directory


main()
