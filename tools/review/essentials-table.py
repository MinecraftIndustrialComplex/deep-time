#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The zinc and lead supply floor and the facies fields (review follow-up of the carbonate and
ore-deposit task, 2026-10-20): one row per run of tools/review/carbonate-ensemble.sh, then each
label's median and range.

    python3 tools/review/essentials-table.py <run dir> [label ...]

Per run, from `deposit-audit`'s JSON: zinc and lead ore blocks per land chunk (the deposits, the
guaranteed deposits among them, the scatter, all), the land regions holding a source of each
essential (the audit's `essentials`: 8,192-block regions and 2,048-block ones), the distance from
the spawn (the first spawn candidate) and the median candidate to the nearest source, the access
probe's median half-width to gather its need of each (deposits and scatter; points beyond its
radius), and the MVT and SEDEX sites (natural, guaranteed). From the run log's `surface_final` (absent on older
builds) and `strata-check`: black shale laid (10^6 km3), the mean area of the deposit engine's
carbonate-platform and euxinic-basin fields and of source rock richer than plain shale
(10^6 km2), the source-rock TOC summed over the planet's area (`toc_area`, 10^6 km2 x wt %; the
log holds no planet area to make it a mean), and black shale's share of the sea
floor's and the land's top blocks.
"""
import collections
import json
import pathlib
import statistics
import sys

MATERIALS = ["c:zinc", "c:lead"]


def kv(line):
    return dict(t.split("=", 1) for t in line.split() if "=" in t)


def num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return float("nan")


def surface_final(txt):
    surf = {}
    for line in txt.read_text(errors="replace").splitlines():
        for part in line.split(": ")[1:] if line.startswith("deeptime ") else [line]:
            part = part.strip()
            if part.startswith("surface_final="):
                surf = kv(part)
    return surf


def share(census, name):
    return census.get("classes", {}).get(name, {}).get("top_share", 0.0)


def run_row(root, stem):
    r = {"run": stem, "label": stem.split("-")[0]}
    aj = root / f"{stem}.audit.json"
    if aj.exists():
        a = json.load(open(aj))
        by = a.get("planet", {}).get("ore_by_source", {})
        ess = a.get("essentials", {}).get("by_material", {})
        for m in MATERIALS:
            k = m.split(":")[1]
            pl = by.get(m, {}).get("per_land_chunk", {})
            for part in ["deposits", "guaranteed", "scatter", "all"]:
                r[f"{k}_{part}"] = num(pl.get(part))
            e = ess.get(m, {})
            for g in ["essential", "fine"]:
                gr = e.get("regions", {}).get(g, {})
                r[f"{k}_{g}"] = f"{gr.get('land_regions_with_a_source', '?')}/{gr.get('land_regions', '?')}"
            d = [num(x) for x in e.get("spawn_to_nearest_source", []) if x is not None]
            r[f"{k}_spawn"] = d[0] if d else float("nan")
            r[f"{k}_spawn_med"] = statistics.median(d) if d else float("nan")
            r[f"{k}_sources"] = e.get("sources", 0)
            r[f"{k}_guaranteed_sources"] = e.get("guaranteed_sources", 0)
            acc = ((a.get("access") or {}).get("by_material") or {}).get(m, {}).get("deposits_and_scatter", {})
            r[f"{k}_access_p50"] = num(acc.get("p50"))
            r[f"{k}_access_beyond"] = acc.get("beyond_radius", "?")
        sites = a.get("sites", [])
        for t in ["mvt_zn_pb", "sedex_zn_pb"]:
            of = [x for x in sites if x["type"].endswith(t)]
            g = sum(bool(x.get("guaranteed")) for x in of)
            r[t] = f"{len(of) - g}+{g}"
    txt = root / f"{stem}.txt"
    if txt.exists():
        s = surface_final(txt)
        myr = num(s.get("surface_final")) * 2.0  # steps x 2 Myr
        r["black_shale"] = num(s.get("black_shale_km3")) / 1e6
        for k in ["platform", "euxinic", "toc_rich"]:
            r[k] = num(s.get(f"{k}_km2_myr")) / myr / 1e6 if myr > 0 else float("nan")
        r["toc_area"] = num(s.get("toc_km2_myr")) / myr / 1e6 if myr > 0 else float("nan")
    sj = root / f"{stem}.strata.json"
    if sj.exists():
        st = json.load(open(sj))
        r["bs_sea"] = share(st.get("sea_columns", {}), "black_shale")
        r["bs_land"] = share(st.get("land_columns", {}), "black_shale")
    return r


def fmt(v):
    if isinstance(v, float):
        if v != v:
            return "-"
        return f"{v:.3g}"
    return str(v)


def main():
    root = pathlib.Path(sys.argv[1])
    labels = sys.argv[2:]
    rows = []
    for txt in sorted(root.glob("*.audit.json")):
        stem = txt.name[: -len(".audit.json")]
        if labels and stem.split("-")[0] not in labels:
            continue
        rows.append(run_row(root, stem))
    keys = []
    for r in rows:
        keys += [k for k in r if k not in keys]
    print("| " + " | ".join(keys) + " |")
    print("|" + "---|" * len(keys))
    for r in rows:
        print("| " + " | ".join(fmt(r.get(k, "")) for k in keys) + " |")
    by = collections.defaultdict(list)
    for r in rows:
        by[r["label"]].append(r)
    print()
    for label, rs in by.items():
        out = [f"{label} (n={len(rs)}):"]
        for k in keys:
            vals = [r[k] for r in rs if isinstance(r.get(k), float) and r[k] == r[k]]
            if vals:
                out.append(f"{k} {statistics.median(vals):.3g} [{min(vals):.3g}, {max(vals):.3g}]")
        print("  ".join(out))


if __name__ == "__main__":
    main()
