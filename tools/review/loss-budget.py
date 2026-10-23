#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continental crust's additions and losses against Stern & Scholl 2010, Table 1.

  python3 tools/review/loss-budget.py <length Myr> <summary.txt> ... [--label NAME]

Reads each run's crust budget (`DEEPTIME_CRUST_BUDGET=1`: the last `cb_t`, `cb_parts`,
`cb_moves` and `cb_globals` lines, cumulative over the run) and prints, in km^3 a year over the
whole run (Earth-normalized; 1 AU = 1 km^3/yr), Stern & Scholl's rows with the model's own
number beside Earth's, per run and as the mean [lowest, highest] over the runs.

Stern & Scholl 2010 (Int. Geol. Rev. 52:1-31, full text, Table 1 p. 12 and pp. 11-19):
  additions  intra-oceanic arcs 1.5, continental arcs 1.0, crust-suturing zones 0.2,
             rifts / hotspots / volcanic rifted margins 0.5; total 3.2
  losses     ocean margins: sediment subduction 1.1, subduction erosion 1.4 (about 60 km^3/km/Myr
             over 43,000 km of margin: 25 sediment, 35 forearc, after Scholl & von Huene 2009);
             crust-suturing zones: sediment 0.1, erosion 0.3, deep subduction of continental
             crust 0.3 (115 km^3/km/Myr over 6,000 km long-term); foundering unknown; total > 3.2
  lengths    intra-oceanic arcs 14,500 km (100 km^3/km/Myr), continental arcs 27,500 km (35)

The model's rows:
  island arcs        arc magma on oceanic crust (`arc_o`); it turns continental when the arc
                     matures (PLT-047), with the ocean floor under it (row "ocean floor")
  continental arcs   arc magma on continental crust (`arc_c`; the cones counted inside)
  hotspots           the hotspots' own magma (`hotspot_magma`)
  ocean floor        the ocean floor's own crust (7 km a sample) made continental with the crust
                     on it (`cb_moves` o2c, every part), less what stays under it as its basement
                     and is not counted as continental crust (`o2c_b_km`, PLT-331, since
                     2026-10-20; runs before it have none, so the whole 7 km)
  pelagic            felsic rock the plate step's surface makes (`cb_parts` felsic, part
                     `plate_surface`: the pelagic sediment, `pelagic_m_per_myr`)
  sediment subd.     the share of the slab's sediment and sedimentary rock a resampling sends to the
                     mantle (`sed_subducted`, PLT-440, since 2026-10-22; the rest accretes to the
                     column on top, `accreted`, of it ocean floor's: accreted - accreted_continent)
  subd. erosion      the forearc crust erosive fronts take to the mantle (`forearc_c` continental,
                     `forearc_o` the rest; PLT-441, since 2026-10-22; 0 with the rule off)
  collision          crust a collision took across its front less the share stacked (crossed - kept),
                     split (since 2026-10-21, PLT-401) into the crust found no room (`noroom_sliver`:
                     left over in a belt with room in it; `noroom_capped`: the belt at the strength
                     height; `noroom_clipped`: the ceilings) and the rest, PLT-242's share's part
                     (whatever the share and PLT-400, the parts add up to crossed - kept)
  foundering         crust beyond `crust_max_km` sunk from the column's bottom (`foundered`)
  rate cap           crust over the thickening ceiling with no room, sent down (`rate_lost`)
  dropped            crust a resampling drops (`dropped` + `mafic_dropped`)
and the felsic rock made or destroyed per part (`cb_parts` felsic), the flows between the
continental and oceanic labels (`cb_parts` vol by part) and the net change of the continental
volume (visible and hidden) over the run.

The collision fronts' lengths and rates (PLT-401; runs from 2026-10-21 on, NaN before): the front
in collision at any time (`front_km_myr` over the run's length), the convergence across it
(`crossed_km2` / `front_km_myr`), the crust crossing (`crossed` / `crossed_km2`), the area crossing
a year, the loss per km of front and Myr, the share kept, and the crossing crust above the normal
column (`crossed_above`), raised by a collision within 30 Myr (`crossed_recent`) and stacked by the
same collision (`crossed_restack`, PLT-400). Earth (plates.md "The crust flux into collision
fronts"): 6,000 km of active suture on average and 12,000 km today, 115 km^3/km/Myr (Stern &
Scholl 2010, p. 18-19); the continental convergent boundary 23,003 km at 18.7 mm/yr across, 0.43
km^2 a year (Bird 2003, Table 3).
"""
import re
import sys

NUM = r"(-?[0-9.]+(?:e[+-]?[0-9]+)?|NaN|nan|inf|-inf)"
PLANET_KM2 = 5.1007e8  # Earth's surface, km^2: the budget's shares are of the planet.
OCEAN_CRUST_KM = 7.0  # the ocean floor's own crust (`transition::oceanic_m`), km


def kv(text):
    return {k: float(v) for k, v in re.findall(r"([A-Za-z0-9_/]+)=" + NUM + r"(?=[\s\]]|$)", text)}


def read(path):
    last = {}
    first_t = None
    with open(path) as f:
        for line in f:
            m = re.match(r"(cb_t|cb_parts|cb_moves|cb_globals) t=(-?[0-9.]+)", line)
            if not m:
                continue
            kind, body = m.group(1), line[m.end():]
            if kind == "cb_t":
                d = kv(body)
                if first_t is None:
                    first_t = d
                last["t"] = d
            elif kind == "cb_parts":
                last["parts"] = {
                    name: {p: float(x) for p, x in re.findall(r"(\w+):" + NUM, inner)}
                    for name, inner in re.findall(r"(\w+)=\[([^\]]*)\]", body)
                }
            elif kind == "cb_moves":
                last["moves"] = {
                    name: kv(inner) for name, inner in re.findall(r"(\w+)=\[([^\]]*)\]", body)
                }
            else:
                last["globals"] = kv(body)
    last["first"] = first_t
    return last


def budget(run, length):
    yrs = length * 1.0e6
    g = run["globals"]
    parts = run["parts"]
    moves = run["moves"]
    per = lambda km3: km3 / yrs  # a global (km^3 over the run) in km^3/yr
    part = lambda stock, name: parts.get(stock, {}).get(name, 0.0) * 1.0e6 / yrs
    o2c_km3 = sum(m.get("o2c", 0.0) * PLANET_KM2 * m.get("o2c_t_km", 0.0) for m in moves.values())
    o2c_floor = sum(
        m.get("o2c", 0.0) * PLANET_KM2 * max(OCEAN_CRUST_KM - m.get("o2c_b_km", 0.0), 0.0)
        for m in moves.values()
    )
    o2c_basement = sum(m.get("o2c", 0.0) * PLANET_KM2 * m.get("o2c_b_km", 0.0) for m in moves.values())
    out = {
        "island_arcs": per(g.get("arc_o", 0.0)),
        "continental_arcs": per(g.get("arc_c", 0.0)),
        "hotspots": per(g.get("hotspot_magma", 0.0)),
        "made_continental": o2c_km3 / yrs,
        "ocean_floor": o2c_floor / yrs,
        "basement": o2c_basement / yrs,
        "pelagic": part("felsic", "plate_surface"),
        "accreted_ocean": per(g.get("accreted", 0.0) - g.get("accreted_continent", 0.0)),
        "sed_subducted": per(g.get("sed_subducted", 0.0)),
        "forearc": per(g.get("forearc_c", 0.0) + g.get("forearc_o", 0.0)),
        "forearc_c": per(g.get("forearc_c", 0.0)),
        "erosive_km": (g.get("forearc_front_c", 0.0) + g.get("forearc_front_o", 0.0)) / length,
        "collision": per(g.get("crossed", 0.0) - g.get("kept", 0.0)),
        "foundering": per(g.get("foundered", 0.0)),
        "rate_cap": per(g.get("rate_lost", 0.0)),
        "dropped": per(g.get("dropped", 0.0) + g.get("mafic_dropped", 0.0)),
        "cordillera": per(g.get("cordillera", 0.0)),
        "crossed": per(g.get("crossed", 0.0)),
        "erosion_off": -part("vol", "erosion"),
        "transition_off": -part("vol", "transition"),
        "front_o_km": g.get("front_o", 0.0) / length,
        "front_c_km": g.get("front_c", 0.0) / length,
        "g_c_full": g.get("front_gc_full", float("nan")) / max(g.get("front_c", 0.0), 1e-9),
        "g_c_normal": g.get("front_gc_normal", float("nan")) / max(g.get("front_c", 0.0), 1e-9),
        "g_full": g.get("front_g_full", 0.0) / max(g.get("front_o", 0.0) + g.get("front_c", 0.0), 1e-9),
        "g_normal": g.get("front_g_normal", 0.0) / max(g.get("front_o", 0.0) + g.get("front_c", 0.0), 1e-9),
    }
    # The collision fronts (PLT-401): NaN for runs from before the globals existed.
    nan = float("nan")
    crossed, kept = g.get("crossed", 0.0), g.get("kept", 0.0)
    area, fkm = g.get("crossed_km2", nan), g.get("front_km_myr", nan)
    noroom = [g.get(f"noroom_{k}", nan) for k in ("sliver", "capped", "clipped")]
    out["noroom_sliver"], out["noroom_capped"], out["noroom_clipped"] = (per(x) for x in noroom)
    out["coll_share"] = per(crossed - kept - sum(noroom))
    out["front_coll_km"] = fkm / length
    out["conv_coll"] = area / fkm if fkm > 0 else nan
    out["h_coll"] = crossed / area if area > 0 else nan
    out["area_coll"] = area / yrs
    out["coll_per_km"] = (crossed - kept) / fkm if fkm > 0 else nan
    out["kept_frac"] = kept / crossed if crossed > 0 else nan
    for key, name in (("above_frac", "crossed_above"), ("recent_frac", "crossed_recent"),
                      ("restack_frac", "crossed_restack")):
        out[key] = g.get(name, nan) / crossed if crossed > 0 else nan
    out["wedge_frac"] = g.get("kept_wedge", nan) / kept if kept > 0 else nan
    out["arc_o_per_km"] = g.get("arc_o", 0.0) / max(g.get("front_o", 0.0), 1e-9)
    out["arc_c_per_km"] = g.get("arc_c", 0.0) / max(g.get("front_c", 0.0), 1e-9)
    out["cord_per_km"] = g.get("cordillera", 0.0) / max(g.get("front_c", 0.0), 1e-9)
    t0, t1 = run["first"], run["t"]
    vol = lambda t: t.get("vol_km3", 0.0) + t.get("vol_hidden_km3", 0.0)
    out["net"] = (vol(t1) - vol(t0)) / yrs
    out["cont_end"] = t1.get("cont", float("nan"))
    out["felsic_parts"] = {k: v * 1.0e6 / yrs for k, v in parts.get("felsic", {}).items()}
    out["felsic_ocean_parts"] = {k: v * 1.0e6 / yrs for k, v in parts.get("felsic_ocean", {}).items()}
    out["vol_parts"] = {
        k: (parts.get("vol", {}).get(k, 0.0) + parts.get("vol_hidden", {}).get(k, 0.0)) * 1.0e6 / yrs
        for k in set(parts.get("vol", {})) | set(parts.get("vol_hidden", {}))
    }
    return out


ROWS = [
    ("additions", None, None),
    ("intra-oceanic (island) arcs", "island_arcs", 1.5),
    ("continental arcs", "continental_arcs", 1.0),
    ("crust-suturing zones", None, 0.2),
    ("rifts, hotspots, rifted margins", "hotspots", 0.5),
    ("model only: ocean floor's own crust made continental", "ocean_floor", 0.0),
    ("model only: pelagic sediment made", "pelagic", 0.0),
    ("losses", None, None),
    ("ocean margins: sediment subduction (PLT-440)", "sed_subducted", 1.1),
    ("ocean margins: subduction erosion (PLT-441)", "forearc", 1.4),
    ("  of it continental crust", "forearc_c", float("nan")),
    ("crust-suturing zones (all three)", "collision", 0.7),
    ("  of it PLT-242's share", "coll_share", float("nan")),
    ("  of it no room: left over in a belt with room", "noroom_sliver", float("nan")),
    ("  of it no room: the belt at the strength height", "noroom_capped", float("nan")),
    ("  of it no room: clipped by the ceilings", "noroom_clipped", float("nan")),
    ("foundering", "foundering", float("nan")),
    ("model only: rate cap sent down", "rate_cap", 0.0),
    ("model only: resampling drops", "dropped", 0.0),
    ("flows (moved, not made or lost)", None, None),
    ("ocean-floor felsic rock accreted at resamplings", "accreted_ocean", float("nan")),
    ("off the continents by the surface stage", "erosion_off", float("nan")),
    ("off the continents by the transition", "transition_off", float("nan")),
    ("crust made continental (o2c x thickness)", "made_continental", float("nan")),
    ("ocean floor kept under it as basement, not counted (PLT-331)", "basement", float("nan")),
    ("crossed collision fronts", "crossed", float("nan")),
    ("shortened into cordilleras", "cordillera", float("nan")),
    ("net change of the continental volume", "net", 0.0),
]

LENGTHS = [
    ("island-arc front, km (Earth 14,500)", "front_o_km", "{:.0f}"),
    ("continental-arc front, km (Earth 27,500)", "front_c_km", "{:.0f}"),
    ("erosive fronts, km (Clift & Vannucchi 2004: 22,925 of 40,225)", "erosive_km", "{:.0f}"),
    ("island arcs per km of front, km3/km/Myr (Earth ~100 at full rate)", "arc_o_per_km", "{:.1f}"),
    ("continental arcs per km, km3/km/Myr (Earth 35)", "arc_c_per_km", "{:.1f}"),
    ("cordillera per km of continental front, km3/km/Myr", "cord_per_km", "{:.1f}"),
    ("mean g, all oceanic fronts, full speed", "g_full", "{:.3f}"),
    ("mean g, all oceanic fronts, normal", "g_normal", "{:.3f}"),
    ("mean g, continental fronts, full speed", "g_c_full", "{:.3f}"),
    ("mean g, continental fronts, normal", "g_c_normal", "{:.3f}"),
    ("collision front at any time, km (Earth 6,000 to 12,000 of suture)", "front_coll_km", "{:.0f}"),
    ("convergence across it, mm/yr (Bird's CCB 18.7)", "conv_coll", "{:.1f}"),
    ("crust crossing it, km", "h_coll", "{:.1f}"),
    ("area crossing, km2/yr (Bird's CCB 0.43)", "area_coll", "{:.3f}"),
    ("collision loss per km of front, km3/km/Myr (Earth ~115)", "coll_per_km", "{:.0f}"),
    ("kept / crossed", "kept_frac", "{:.3f}"),
    ("crossing crust above the normal column, share", "above_frac", "{:.3f}"),
    ("  raised by a collision within 30 Myr, share", "recent_frac", "{:.3f}"),
    ("  stacked by the same collision (PLT-400), share", "restack_frac", "{:.3f}"),
    ("wedge / kept", "wedge_frac", "{:.3f}"),
    ("continental crust at the end, share of the planet", "cont_end", "{:.3f}"),
]


def stats(xs):
    xs = [x for x in xs if x == x]
    if not xs:
        return float("nan"), float("nan"), float("nan")
    return sum(xs) / len(xs), min(xs), max(xs)


def main():
    args = sys.argv[1:]
    label = None
    if "--label" in args:
        i = args.index("--label")
        label = args[i + 1]
        del args[i : i + 2]
    length = float(args[0])
    runs = []
    for p in args[1:]:
        r = read(p)
        if "globals" not in r:
            print(f"# {p}: no crust budget", file=sys.stderr)
            continue
        runs.append((p, budget(r, length)))
    if not runs:
        sys.exit("no runs with a crust budget")
    print(f"# loss budget{' ' + label if label else ''}: {len(runs)} runs, km3/yr over {length:.0f} Myr")
    print(f"{'row':58s} {'model mean [min, max]':>26s} {'Earth':>6s}")
    for name, key, earth in ROWS:
        if key is None and earth is None:
            print(f"-- {name}")
            continue
        if key is None:
            print(f"{name:58s} {'0 (no rule)':>26s} {earth:6.2f}")
            continue
        m, lo, hi = stats([b[key] for _, b in runs])
        e = "" if earth != earth else f"{earth:6.2f}"
        print(f"{name:58s} {m:8.2f} [{lo:6.2f}, {hi:6.2f}]  {e:>6s}")
    print("-- lengths and rates")
    for name, key, fmt in LENGTHS:
        m, lo, hi = stats([b[key] for _, b in runs])
        print(f"{name:58s} {fmt.format(m):>8s} [{fmt.format(lo)}, {fmt.format(hi)}]")
    for title, key in [
        ("felsic rock made (+) or destroyed (-) by part", "felsic_parts"),
        ("felsic rock onto (+) or off (-) the ocean floor by part", "felsic_ocean_parts"),
        ("continental volume (visible + hidden) by part", "vol_parts"),
    ]:
        names = sorted({k for _, b in runs for k in b[key]})
        if not names:
            continue
        print(f"-- {title}, km3/yr")
        for n in names:
            m, lo, hi = stats([b[key].get(n, 0.0) for _, b in runs])
            if abs(m) < 0.005 and abs(lo) < 0.005 and abs(hi) < 0.005:
                continue
            print(f"   {n:55s} {m:8.2f} [{lo:6.2f}, {hi:6.2f}]")
    if len(runs) > 1:
        print("-- per run: island arcs, continental arcs, ocean floor, pelagic, collision, foundering, rate cap, net")
        for p, b in runs:
            keys = ["island_arcs", "continental_arcs", "ocean_floor", "pelagic", "collision", "foundering", "rate_cap", "net"]
            print(f"   {p.rsplit('/', 1)[-1]:45s} " + " ".join(f"{b[k]:6.2f}" for k in keys))


if __name__ == "__main__":
    main()
