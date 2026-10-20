#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The land-shape task's table (build log 2026-10-20): per ensemble of
tools/review/force-balance-ensemble.sh run with DEEPTIME_CRUST_BUDGET=1, the end of each run's
crust budget (land, land above 2 km, coast p50, the land by the origin of its crust, the thickness
bands, the flow's substeps) and climate (Koppen E share of the land, CO2 and the natural CO2, the
CO2 before the present-day nudge's factor), with the mean over the runs and each run (seeds pair up
across groups).

    python3 tools/review/land-shape.py <len> <seeds,> <starts,> name=<dir>[:<label>] ...
"""
import re
import sys
from pathlib import Path

ORIGINS = ["island_arc", "collision", "andean", "other"]


def last_line(text, tag):
    lines = [l for l in text.splitlines() if l.startswith(tag + " ")]
    return lines[-1] if lines else None


def kv(line):
    return {k: float(v) for k, v in re.findall(r"(\w+)=(-?[0-9.]+(?:e[-+]?[0-9]+)?|nan)", line)}


def read(path):
    text = Path(path).read_text(errors="replace")
    out = {}
    m = re.search(r"koppen_land=A([0-9.]+)/B([0-9.]+)/C([0-9.]+)/D([0-9.]+)/E([0-9.]+)", text)
    if m:
        out["E"] = float(m.group(5))
    for key in ["co2_ppm", "present_co2_factor", "t_surface_c"]:
        m = re.search(rf"\b{key}=(-?[0-9.]+)", text)
        if m:
            out[key] = float(m.group(1))
    if "co2_ppm" in out and out.get("present_co2_factor", 0) > 0:
        out["co2_natural"] = out["co2_ppm"] / out["present_co2_factor"]
    t = last_line(text, "cb_t")
    if t:
        d = kv(t)
        for k in ["land", "above_2km", "coast_p50_km", "cont", "dry", "mean_t_km"]:
            out[k] = d.get(k, float("nan"))
    o = last_line(text, "cb_origin")
    if o:
        for name in ORIGINS:
            m = re.search(rf"{name}=\[land=([0-9.]+) above_2km=([0-9.]+) coast_p50_km=(-?[0-9.]+|NaN|nan)\]", o)
            if m:
                out[f"{name}_land"] = float(m.group(1))
                out[f"{name}_hi"] = float(m.group(2))
                out[f"{name}_coast"] = float(m.group(3)) if m.group(3).lower() != "nan" else float("nan")
        hi = sum(out.get(f"{n}_land", 0) * out.get(f"{n}_hi", 0) for n in ORIGINS)
        for n in ORIGINS:
            out[f"{n}_hi_share"] = out.get(f"{n}_land", 0) * out.get(f"{n}_hi", 0) / hi if hi > 0 else float("nan")
    b = last_line(text, "cb_bins")
    if b:
        for lo, hi_, a, z50 in re.findall(r"\[([0-9.]+)-([0-9.]+)km a=([0-9.]+) v=[0-9.]+ z10/50/90=-?[0-9]+/(-?[0-9]+)/", b):
            out[f"band{int(float(lo))}"] = float(a)
            out[f"band{int(float(lo))}_z50"] = float(z50)
    g = last_line(text, "cb_globals")
    if g:
        d = kv(g)
        for k in ["flow_steps", "flow_substeps", "flow_short_myr", "flow_edge", "channel_flow"]:
            if k in d:
                out[k] = d[k]
    return out


def fmt(v, k):
    if v != v:
        return "nan"
    if k in ("co2_ppm", "co2_natural", "coast_p50_km") or k.endswith("_coast") or k.endswith("_z50") or k.startswith("flow"):
        return f"{v:.0f}"
    if k in ("t_surface_c", "mean_t_km", "present_co2_factor"):
        return f"{v:.2f}"
    return f"{v:.3f}"


def main():
    length, seeds, starts = sys.argv[1], sys.argv[2].split(","), sys.argv[3].split(",")
    groups = []
    for a in sys.argv[4:]:
        name, spec = a.split("=", 1)
        folder, label = (spec.split(":", 1) + ["r"])[:2] if ":" in spec else (spec, "r")
        groups.append((name, folder, label))
    keys = ["land", "above_2km", "coast_p50_km", "cont", "dry", "mean_t_km", "E", "co2_ppm", "co2_natural",
            "present_co2_factor", "t_surface_c"]
    keys += [f"{n}_{s}" for n in ORIGINS for s in ("land", "hi", "coast", "hi_share")]
    keys += [f"band{e}" for e in (0, 20, 25, 30, 35, 40, 45, 55, 70)]
    keys += ["flow_steps", "flow_substeps", "flow_short_myr", "flow_edge"]
    for start in starts:
        print(f"\n== {start}, {length} Myr, seeds {','.join(seeds)}")
        rows = {}
        for name, folder, label in groups:
            runs = []
            for seed in seeds:
                f = Path(folder) / f"{label}-{start}-{seed}-{length}.txt"
                runs.append(read(f) if f.exists() else {})
            rows[name] = runs
        head = f"{'':24}" + "".join(f"{n:>34}" for n, _, _ in groups)
        print(head)
        for k in keys:
            cells = []
            for name, _, _ in groups:
                vals = [r.get(k, float("nan")) for r in rows[name]]
                ok = [v for v in vals if v == v]
                mean = sum(ok) / len(ok) if ok else float("nan")
                cells.append(f"{fmt(mean, k)} [{' '.join(fmt(v, k) for v in vals)}]")
            if all(c.startswith("nan [") and set(c[5:-1].split()) <= {"nan"} for c in cells):
                continue
            print(f"{k:24}" + "".join(f"{c:>34}" for c in cells))


if __name__ == "__main__":
    main()
