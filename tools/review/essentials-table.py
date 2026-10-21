#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The zinc and lead supply floor and the facies fields (review follow-up of the carbonate and
ore-deposit task, 2026-10-20): one row per run of tools/review/carbonate-ensemble.sh, then each
label's median and range.

    python3 tools/review/essentials-table.py <run dir> [label ...]
    python3 tools/review/essentials-table.py --guaranteed <run dir> [label ...]
    python3 tools/review/essentials-table.py --dry <before dir> <after dir> [label ...]

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

`--guaranteed` (the guaranteed deposits in the world as generated, 2026-10-20): per essential the
audit ships, the guaranteed sources holding ore of the material in the world's columns among the
guaranteed sources, the world's land regions (the essential's region size, on the world's own
frame) with a source and with a source holding ore, the spawn's nearest source and nearest source
holding ore, and the access probe's median (deposits and scatter). Needs a build whose audit writes
the `_with_ore` fields (AUDIT_BIN on an older build's worlds).

`--dry` (the top-up counts only sources with a dry centre, 2026-10-21): two run dirs of the same
planets (a label's runs matched by their start and seed; the label may differ, the build before and
the build after), audited by a build that writes the `_dry` fields (AUDIT_BIN). Per planet and
essential, before and after: the sources with ore in the land regions (before: every one, as the
old top-up counted; after: those with a dry centre) per land region, the land regions holding a
dry source with ore, the copies the top-up added and the sources it left out for lying under the
sea (`bake_guaranteed.top_up`, after only), the sources with ore under the sea, and the world's
guaranteed sites with a centre under the sea; then each essential summed over the planets and the
top-up's seconds (`top_up_millis`).
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


def guaranteed_rows(root, labels):
    """One row per (run, essential)."""
    rows = []
    for aj in sorted(root.glob("*.audit.json")):
        stem = aj.name[: -len(".audit.json")]
        label = stem.split("-")[0]
        if labels and label not in labels:
            continue
        a = json.load(open(aj))
        ess = a.get("essentials", {}).get("by_material", {})
        access = (a.get("access") or {}).get("by_material") or {}
        for m, e in sorted(ess.items()):
            g = e.get("regions", {}).get("essential", {})
            acc = access.get(m, {}).get("deposits_and_scatter", {})

            def first(k):
                return (e.get(k) or [None])[0]

            rows.append({
                "run": stem, "label": label, "material": m.split(":")[-1],
                "g": e.get("guaranteed_sources", 0),
                "g_ore": e.get("guaranteed_sources_with_ore"),
                "land": g.get("land_regions"),
                "with": g.get("land_regions_with_a_source"),
                "with_ore": g.get("land_regions_with_a_source_with_ore"),
                "spawn": num(first("spawn_to_nearest_source")),
                "spawn_ore": num(first("spawn_to_nearest_source_with_ore")),
                "p50": num(acc.get("p50")),
                "beyond": acc.get("beyond_radius"),
            })
    return rows


def guaranteed_main(root, labels):
    rows = guaranteed_rows(root, labels)
    keys = ["run", "material", "g_ore/g", "regions (ore)/land", "spawn (ore)", "access p50 (beyond)"]
    print("| " + " | ".join(keys) + " |")
    print("|" + "---|" * len(keys))
    for r in rows:
        print(f"| {r['run']} | {r['material']} | {r['g_ore']}/{r['g']} | "
              f"{r['with']} ({r['with_ore']})/{r['land']} | {fmt(r['spawn'])} ({fmt(r['spawn_ore'])}) | "
              f"{fmt(r['p50'])} ({r['beyond']}) |")
    print()
    by = collections.defaultdict(list)
    for r in rows:
        by[(r["label"], r["material"])].append(r)
    print("| label | material | runs | guaranteed with ore / guaranteed | runs with every land region "
          "holding a source (with ore) | land regions with a source (with ore) / land regions | "
          "spawn to the nearest source, median [range] | the same, holding ore | access p50, median [range] |")
    print("|---|---|---|---|---|---|---|---|---|")

    def med(vals):
        vals = [v for v in vals if v == v]
        if not vals:
            return "-"
        return f"{statistics.median(vals):.4g} [{min(vals):.4g}, {max(vals):.4g}]"

    for (label, m), rs in by.items():
        g = sum(r["g"] or 0 for r in rs)
        go = sum(r["g_ore"] or 0 for r in rs)
        full = sum(r["with"] == r["land"] for r in rs)
        full_ore = sum(r["with_ore"] == r["land"] for r in rs)
        land = sum(r["land"] or 0 for r in rs)
        w = sum(r["with"] or 0 for r in rs)
        wo = sum(r["with_ore"] or 0 for r in rs)
        print(f"| {label} | {m} | {len(rs)} | {go}/{g} | {full} ({full_ore}) of {len(rs)} | "
              f"{w} ({wo})/{land} | {med([r['spawn'] for r in rs])} | "
              f"{med([r['spawn_ore'] for r in rs])} | {med([r['p50'] for r in rs])} |")


def dry_rows(root, labels):
    """{(start-seed, material): row}, one per run of the dir (of the named labels)."""
    rows = {}
    for aj in sorted(root.glob("*.audit.json")):
        stem = aj.name[: -len(".audit.json")]
        label, _, key = stem.partition("-")
        if labels and label not in labels:
            continue
        a = json.load(open(aj))
        ess = a.get("essentials", {}).get("by_material", {})
        bake = a.get("bake_guaranteed") or {}
        topped = bake.get("top_up") or {}
        for m, e in sorted(ess.items()):
            g = e.get("regions", {}).get("essential", {})
            t = topped.get(m, {})
            rows[(key, m.split(":")[-1])] = {
                "land": g.get("land_regions") or 0,
                "all": g.get("ore_sources_in_land_regions"),
                "dry": g.get("dry_ore_sources_in_land_regions"),
                "with_ore": g.get("land_regions_with_a_source_with_ore"),
                "with_dry": g.get("land_regions_with_a_dry_source_with_ore"),
                "under_sea": e.get("sources_with_ore_under_the_sea"),
                "added": t.get("added"),
                "left_out": t.get("sub_sea"),
                "short_by_sea": t.get("short_by_sea"),
                "g_sites": a.get("essentials", {}).get("guaranteed_sites"),
                "g_sea": a.get("essentials", {}).get("guaranteed_sites_under_the_sea"),
                "added_all": bake.get("topped_up"),
                "top_up_ms": bake.get("top_up_millis"),
                "anchor_ms": bake.get("millis"),
            }
    return rows


def dry_main(before_dir, after_dir, labels):
    before = dry_rows(before_dir, labels)
    after = dry_rows(after_dir, labels)
    keys = sorted(set(before) & set(after))
    print("| planet | essential | land regions | sources per land region (before, after) | "
          "regions with a dry source with ore (before, after) | copies added (before, after) | "
          "left out as under the sea (after) | ore sources under the sea (before, after) |")
    print("|---|---|---|---|---|---|---|---|")
    for k in keys:
        b, a = before[k], after[k]
        land = b["land"] or 1

        def per(x):
            return fmt(x / land) if isinstance(x, (int, float)) else "-"

        print(f"| {k[0]} | {k[1]} | {b['land']}, {a['land']} | {per(b['all'])}, {per(a['dry'])} | "
              f"{b['with_dry']}, {a['with_dry']} | {fmt(b['added'])}, {fmt(a['added'])} | "
              f"{fmt(a['left_out'])} | {fmt(b['under_sea'])}, {fmt(a['under_sea'])} |")
    print()
    print("| essential | planets | land regions (before, after) | sources counted (before, after) | "
          "per land region | land regions with a dry source with ore (before, after) | "
          "copies added (before, after) | planets with every land region dry-covered (before, after) | "
          "regions short only for the sea (after) |")
    print("|---|---|---|---|---|---|---|---|---|")
    for m in sorted({k[1] for k in keys}):
        ks = [k for k in keys if k[1] == m]

        def tot(side, f):
            vals = [side[k][f] for k in ks]
            nums = [v for v in vals if isinstance(v, (int, float))]
            return sum(nums) if nums else None

        lb, la = tot(before, "land"), tot(after, "land")
        cb, ca = tot(before, "all"), tot(after, "dry")
        full_b = sum(before[k]["with_dry"] == before[k]["land"] for k in ks)
        full_a = sum(after[k]["with_dry"] == after[k]["land"] for k in ks)
        print(f"| {m} | {len(ks)} | {lb}, {la} | {fmt(cb)}, {fmt(ca)} | "
              f"{fmt(cb / lb) if lb else '-'}, {fmt(ca / la) if la else '-'} | "
              f"{fmt(tot(before, 'with_dry'))}, {fmt(tot(after, 'with_dry'))} | "
              f"{fmt(tot(before, 'added'))}, {fmt(tot(after, 'added'))} | {full_b}, {full_a} | "
              f"{fmt(tot(after, 'short_by_sea'))} |")
    print()
    for name, side in (("before", before), ("after", after)):
        runs = {}
        for k in keys:
            runs.setdefault(k[0], side[k])
        gs = sum(v["g_sites"] or 0 for v in runs.values())
        gw = sum(v["g_sea"] or 0 for v in runs.values())
        ad = sum(v["added_all"] or 0 for v in runs.values())
        tu = [v["top_up_ms"] for v in runs.values() if isinstance(v["top_up_ms"], (int, float))]
        an = [v["anchor_ms"] for v in runs.values() if isinstance(v["anchor_ms"], (int, float))]
        med = statistics.median(tu) if tu else float("nan")
        print(f"{name}: {len(runs)} planets, guaranteed sites {gs}, with a centre under the sea {gw}, "
              f"copies added {ad}, top-up median {med:.0f} ms (sum {sum(tu):.0f}, "
              f"range {min(tu) if tu else float('nan'):.0f} to {max(tu) if tu else float('nan'):.0f}), "
              f"anchor median {statistics.median(an) if an else float('nan'):.0f} ms")


def fmt(v):
    if isinstance(v, float):
        if v != v:
            return "-"
        return f"{v:.3g}"
    return str(v)


def main():
    if sys.argv[1] == "--dry":
        dry_main(pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3]), sys.argv[4:])
        return
    if sys.argv[1] == "--guaranteed":
        guaranteed_main(pathlib.Path(sys.argv[2]), sys.argv[3:])
        return
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
