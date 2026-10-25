#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""floorage_th_read.py label@arm-substring=floorage_th.jsonl ...: pooled tables of floorage_th.py, per arm."""
import json
import sys

CUM = [10, 30, 60, 100, 150, 180, 250]
DC = ["0-330", "330-1000", "1000-2000", ">2000"]
DT = ["0-500", "500-1000", "1000-1500", ">1500"]
TH = ["<8", "8-10", "10-15", "15-25", ">=25"]


def pooled(runs):
    acc = {}
    for r in runs:
        for k, v in r["acc"].items():
            if isinstance(v, int):
                acc[k] = acc.get(k, 0) + v
            else:
                acc[k] = [a + b for a, b in zip(acc.get(k, [0] * len(v)), v)]
    return acc


def row(names, vals, tot):
    return "  ".join(f"{n} {v / max(tot, 1):.3f}" for n, v in zip(names, vals))


def table(label, runs):
    a = pooled(runs)
    oc = a["ocean"][0]
    O = a["record_old"][0]
    U = a["proxy_old_thin"][0]
    print(f"\n=== {label}: {len(runs)} runs, {a['snapshots']} snapshots; ocean {oc}, record hist {a['record_hist_sum'][0]}")
    print("one-sided check, proxy count under the age over the thin floor / record's count under it (must be <= 1):")
    print("  " + "  ".join(f"<{c}: {p / max(rc, 1):.3f}" for c, p, rc in zip(CUM, a["proxy_cum_thin"], a["record_cum"])))
    print(f"  thin floor (< 10 km) share of the ocean {a['thin'][0] / oc:.3f}; proxy negative {a['proxy_negative_thin'][0] / max(a['thin'][0], 1):.4f}"
          " of the thin floor")
    print(f"old floor (> 180 Myr, record) {O / oc:.3f} of the ocean; thin floor proxy-old (upper bound) {U / oc:.3f};"
          f" so at least {(O - U) / max(O, 1):.3f} of the old floor is thick crust (>= 10 km), which is {a['thick_dc'] and sum(a['thick_dc']) / oc:.3f} of the ocean")
    th = a["oc_th"]
    print("ocean by thickness (km):", row(TH, th, oc))
    for key, names in (("dc", DC), ("dco", DC), ("dt", DT)):
        print(f" by {key}:")
        for nm in ("oc", "thin", "thick", "old_thin"):
            v = a[f"{nm}_{key}"]
            print(f"   {nm:8} share of the ocean: " + row(names, v, oc))
    print("  on plates with no trench: ocean", f"{a['oc_notrench'][0] / oc:.3f}", "thick", f"{a['thick_notrench'][0] / oc:.3f}",
          "thin proxy-old", f"{a['old_thin_notrench'][0] / oc:.3f}")
    # Lower bounds on the old floor inside a region R: O minus the most the rest can hold (its thin proxy-old and all
    # its thick crust).
    for key, names in (("dco", DC), ("dc", DC)):
        for j in (1, 2):
            rest_u = sum(a[f"old_thin_{key}"][j:])
            rest_thick = sum(a[f"thick_{key}"][j:])
            lb = max(0, O - rest_u - rest_thick)
            print(f"  old floor within {key} < {['330', '1000', '2000'][j - 1]} km: at least {lb / max(O, 1):.3f} of it"
                  f" (region holds {sum(a[f'oc_{key}'][:j]) / oc:.3f} of the ocean)")
    for nm in ("near", "far"):
        v = a[f"thin_{nm}_cum"]
        tot = v[-1]
        print(f"  thin floor {nm} its own continent (dco {'<=' if nm == 'near' else '>'} 330 km): {tot / oc:.3f} of the ocean;"
              " proxy share under: " + "  ".join(f"<{c} {x / max(tot, 1):.3f}" for c, x in zip(CUM, v[:-1])))


def main():
    for arg in sys.argv[1:]:
        label, path = arg.split("=", 1)
        runs = [json.loads(x) for x in open(path) if x.strip().startswith("{")]
        if "@" in label:
            label, sel = label.split("@", 1)
            runs = [r for r in runs if sel in r["arm"]]
        table(label, runs)


main()
