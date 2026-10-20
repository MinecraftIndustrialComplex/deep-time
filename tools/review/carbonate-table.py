#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The carbonate and ore-deposit task's tables (2026-10-19): one row per run of
tools/review/carbonate-ensemble.sh, then each label's median and range.

    python3 tools/review/carbonate-table.py <run dir> [label ...]

Per run: the carbonate share of the land's top blocks, of the sea floor's top blocks (and chalk
alone), and of the sedimentary blocks of all columns (strata-check); the surface stage's volumes
(carbonate, pelagic chalk, marine clastics; km3) and the photic sea floor's mean area with its warm
and warm-and-clear parts (10^6 km2); the natural CO2 (end CO2 over the present-day factor), the
global temperature after the factor and before it (the clock's last row, with its degassing and
weathering, 10^12 mol C/yr); then the deposit sites by type with their strata, MVT on carbonate strata and
on oceanic crust, placers on oceanic crust and below the shelf.
"""
import collections
import glob
import json
import pathlib
import statistics
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
CLASSES = ROOT / "neoforge/src/main/resources/data/deeptime/deeptime/rock_class"
SHELF_M = -200.0  # the shelf's edge for the placer count, m (the facies fields' `shelf`)


def classes():
    carb, sed = set(), set()
    for p in glob.glob(str(CLASSES / "*.json")):
        d = json.load(open(p))
        g = d.get("groups", [])
        if "deeptime:rock/carbonate" in g and "deeptime:rock/metamorphic" not in g:
            carb.add(d["class"])
        if "deeptime:rock/sedimentary" in g or "deeptime:rock/unconsolidated" in g:
            sed.add(d["class"])
    return carb, sed


def kv(line):
    return dict(t.split("=", 1) for t in line.split() if "=" in t)


def num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return float("nan")


def log_lines(txt):
    clim, surf, end = {}, {}, []
    for line in txt.read_text(errors="replace").splitlines():
        for part in line.split(": ")[1:] if line.startswith("deeptime ") else [line]:
            part = part.strip()
            if part.startswith("climate_final="):
                clim = kv(part)
            elif part.startswith("surface_final="):
                surf = kv(part)
            elif part.startswith("carbon_series "):
                tok = part.split()[-1]
                t, vals = tok.split(":", 1)
                end = [num(t)] + [num(v) for v in vals.split("/")]
    return clim, surf, end


def share(census, names, key):
    c = census.get("classes", {})
    return sum(c.get(n, {}).get(key, 0.0) for n in names)


def main():
    root = pathlib.Path(sys.argv[1])
    labels = sys.argv[2:]
    carb, sed = classes()
    rows = []
    for txt in sorted(root.glob("*.txt")):
        stem = txt.name[:-4]
        label = stem.split("-")[0]
        if labels and label not in labels:
            continue
        clim, surf, end = log_lines(txt)
        r = {"run": stem, "label": label}
        sj = root / f"{stem}.strata.json"
        if sj.exists():
            s = json.load(open(sj))
            r["land"] = share(s.get("land_columns", {}), carb, "top_share")
            r["sea"] = share(s.get("sea_columns", {}), carb, "top_share")
            r["chalk"] = share(s.get("sea_columns", {}), ["chalk"], "top_share")
            allc = s.get("columns", {})
            sb = share(allc, sed, "block_share")
            r["sed"] = share(allc, carb, "block_share") / sb if sb > 0 else float("nan")
        length = num(surf.get("surface_final")) * 2.0  # steps x 2 Myr
        for k in ["carbonate_km3", "pelagic_km3", "marine_km3"]:
            r[k.split("_")[0]] = num(surf.get(k)) / 1e6
        for k in ["photic_km2_myr", "photic_warm_km2_myr", "photic_warm_clear_km2_myr"]:
            r[k.replace("_km2_myr", "")] = num(surf.get(k)) / length / 1e6 if length > 0 else float("nan")
        fac = num(clim.get("present_co2_factor", 1.0))
        r["co2"] = num(clim.get("co2_ppm")) / fac if fac > 0 else float("nan")
        r["T"] = num(clim.get("ebm_t_global_c"))
        # The clock's last row: natural (before the present-day factor) T and the fluxes (10^12 mol/yr).
        if len(end) > 5:
            r["T_nat"], r["degas"], r["weath"] = end[2], end[4], end[5]
        aj = root / f"{stem}.audit.json"
        r["types"], r["strata"], r["mvt"], r["placer"] = {}, {}, {}, {}
        if aj.exists():
            sites = json.load(open(aj)).get("sites", [])
            by = collections.Counter(x["type"].split(":")[-1] for x in sites)
            r["types"] = dict(by)
            st = collections.defaultdict(collections.Counter)
            for x in sites:
                st[x["type"].split(":")[-1]][x.get("stratum") or "-"] += 1
            r["strata"] = {k: dict(v) for k, v in st.items()}
            mvt = [x for x in sites if x["type"].endswith("mvt_zn_pb")]
            r["mvt"] = {"n": len(mvt), "on_carbonate": sum(x.get("stratum") in carb for x in mvt),
                        "oceanic": sum(bool(x.get("oceanic")) for x in mvt)}
            pl = [x for x in sites if "placer" in x["type"]]
            r["placer"] = {"n": len(pl), "oceanic": sum(bool(x.get("oceanic")) for x in pl),
                           "below_shelf": sum((x.get("ground_m") or 0.0) < SHELF_M for x in pl)}
        rows.append(r)
    cols = ["land", "sea", "chalk", "sed", "carbonate", "pelagic", "marine", "photic",
            "photic_warm", "photic_warm_clear", "co2", "T", "T_nat", "degas", "weath"]
    print("run".ljust(40) + "".join(c[:10].rjust(11) for c in cols))
    for r in rows:
        print(r["run"].ljust(40) + "".join(f"{r.get(c, float('nan')):11.4g}" for c in cols))
    print()
    for label in sorted({r["label"] for r in rows}):
        rs = [r for r in rows if r["label"] == label]
        print(f"{label}: {len(rs)} runs, median [min, max]")
        for c in cols:
            v = sorted(r[c] for r in rs if c in r and r[c] == r[c])
            if v:
                print(f"  {c:18s} {statistics.median(v):10.4g} [{v[0]:.4g}, {v[-1]:.4g}]")
        tys = sorted({t for r in rs for t in r["types"]})
        print("  sites by type, median [min, max]:")
        for t in tys:
            v = sorted(r["types"].get(t, 0) for r in rs)
            hosts = collections.Counter()
            for r in rs:
                hosts.update(r["strata"].get(t, {}))
            top = ", ".join(f"{k} {n}" for k, n in hosts.most_common(5))
            print(f"    {t:34s} {statistics.median(v):6.1f} [{v[0]}, {v[-1]}]  strata: {top}")
        for k in ["mvt", "placer"]:
            agg = collections.Counter()
            for r in rs:
                agg.update(r[k])
            print(f"  {k} (sums over runs): {dict(agg)}")
        print()


if __name__ == "__main__":
    main()
