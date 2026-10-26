"""The island front from cached runs' records (measurement only, the island-front diagnosis,
2026-10-26): islrec.py <folder>[@N] ...

Per run, the mean over every step with front bands of the dynamics record's boundary mix
(one lattice spacing a front sample) and its front bands, split as the record splits them:
setting 0 (ocean-ocean: the overriding sample oceanic with no continental crust of its own plate
within the arc's distance, PLT-370; the crust budget's island front) and setting 1. Beside them
the crust budget's front_o / front_c (straight km, PLT-530) as run means and per 100 Myr window
(the cb_globals lines are cumulative at each snapshot).
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
        if "cb_series" in rows[0]:
            ts = sorted(rows[0]["cb_series"])
            print("  crust budget front_o by 50 Myr window: " + " ".join(
                f"{statistics.mean([r['cb_series'].get(t, (0, 0))[0] for r in rows]):.0f}" for t in ts))
            print("  crust budget front_c by 50 Myr window: " + " ".join(
                f"{statistics.mean([r['cb_series'].get(t, (0, 0))[1] for r in rows]):.0f}" for t in ts))
        key = arg.split("@")[0].rstrip("/").split("/")[-3]
        json.dump(rows, open(f"islrec_{key}.json", "w"))  # in the working directory


main()
