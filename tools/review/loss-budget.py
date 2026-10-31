#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continental crust's additions and losses against Stern & Scholl 2010, Table 1.

  python3 tools/review/loss-budget.py <length Myr> <summary.txt> ... [--label NAME] [--from T]

With --from T (Myr before the end, e.g. -500) the window from T to the end, from the
cumulative lines printed every 50 Myr (the length is then -T).

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
  island arcs        arc magma at the island arcs' law (`arc_o`: on oceanic crust, and with
                     `island_arc_own_crust`, PLT-760, on the crust an intra-oceanic arc made) and the
                     felsic cones it counted inside its rate (`arc_cones_o`, PLT-761, since
                     2026-10-31; before, the cones were left out of both arc rows); it turns
                     continental when the arc matures (PLT-047), with the ocean floor under it (row
                     "ocean floor")
  continental arcs   arc magma at the continental arcs' law (`arc_c`) and its felsic cones
                     (`arc_cones_c`)
  hotspots           on continental crust: the hotspots' own basalt and their LIPs there
                     (`hotspot_magma_c`) and the cones they counted inside (`hotspot_cones_c`), since
                     2026-10-31; before, all of the hotspots' own basalt (`hotspot_magma`), ocean
                     floor's included and the cones left out (row "model only: hotspots, all")
  ocean floor        the ocean floor's own crust (7 km a sample) made continental with the crust
                     on it (`cb_moves` o2c, every part), less what stays under it as its basement
                     and is not counted as continental crust (`o2c_b_km`, PLT-331, since
                     2026-10-20; runs before it have none, so the whole 7 km)
  pelagic            felsic rock the plate step's surface makes (`cb_parts` felsic, part
                     `plate_surface`: the pelagic sediment, `pelagic_m_per_myr`,
                     until 2026-10-25, when PLT-103 was retired)
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


def read(path, since=None):
    """The last crust-budget lines of a run; with `since` (Myr, e.g. -500) the cumulative lines
    less those printed at that time (the window from it to the end; `cb_moves`' thicknesses are
    the run's to then, so the "made continental" rows are approximate in a window)."""
    last = {}
    first_t = None
    at = {}
    with open(path) as f:
        for line in f:
            m = re.match(r"(cb_t|cb_parts|cb_moves|cb_globals) t=(-?[0-9.]+)", line)
            if not m:
                continue
            kind, body = m.group(1), line[m.end():]
            if since is not None and abs(float(m.group(2)) - since) < 1e-6:
                at[kind] = body
            if kind == "cb_t":
                d = kv(body)
                if first_t is None or (since is not None and abs(float(m.group(2)) - since) < 1e-6):
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
    if since is not None and "cb_globals" in at and "globals" in last:
        g0 = kv(at["cb_globals"])
        last["globals"] = {k: v - g0.get(k, 0.0) for k, v in last["globals"].items()}
        p0 = {
            name: {p: float(x) for p, x in re.findall(r"(\w+):" + NUM, inner)}
            for name, inner in re.findall(r"(\w+)=\[([^\]]*)\]", at.get("cb_parts", ""))
        }
        last["parts"] = {
            name: {p: x - p0.get(name, {}).get(p, 0.0) for p, x in d.items()}
            for name, d in last["parts"].items()
        }
        m0 = {name: kv(inner) for name, inner in re.findall(r"(\w+)=\[([^\]]*)\]", at.get("cb_moves", ""))}
        for name, d in last["moves"].items():
            for key in ("o2c", "c2o", "hide", "surface", "drown", "emerge"):
                if key in d:
                    d[key] -= m0.get(name, {}).get(key, 0.0)
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
    nan = float("nan")
    split = "hotspot_magma_c" in g  # runs from 2026-10-31 on (PLT-761)
    out = {
        "island_arcs": per(g.get("arc_o", 0.0) + g.get("arc_cones_o", 0.0)),
        "continental_arcs": per(g.get("arc_c", 0.0) + g.get("arc_cones_c", 0.0)),
        "suture_arcs": per(g.get("arc_s", 0.0)),
        "arc_cones": per(g.get("arc_cones_o", nan) + g.get("arc_cones_c", nan)),
        "hotspots": per(
            g.get("hotspot_magma_c", 0.0)
            + g.get("hotspot_cones_c", 0.0)
            # The hotspot lane (PLT-795, PLT-799): the LIPs' plateau beyond the plate model's dome
            # on continental crust (the volcano stage lays it), and the volcanic margins.
            + g.get("lip_c", 0.0)
            - g.get("lip_dome_c", 0.0)
            + g.get("vrm", 0.0)
        )
        if split
        else per(g.get("hotspot_magma", 0.0)),
        "lip_c": per(g.get("lip_c", nan)),
        "lip_all": per(g.get("lip", nan)),
        "vrm": per(g.get("vrm", nan)),
        "tails_c": per(g.get("hotspot_magma_c", 0.0) + g.get("hotspot_cones_c", 0.0) - g.get("lip_dome_c", 0.0)),
        "hotspots_all": per(g.get("hotspot_magma", 0.0) + g.get("hotspot_cones", 0.0)),
        "cones_all": per(g.get("cone_felsic", 0.0) + g.get("cone_mafic", 0.0)),
        "sed_trench": per(g.get("sed_trench", nan)),
        "sed_one_way": per(g.get("sed_trench", nan) - g.get("sed_trench_c", nan) - g.get("sed_trench_o", nan)),
        "made_continental": o2c_km3 / yrs,
        "ocean_floor": o2c_floor / yrs,
        "basement": o2c_basement / yrs,
        "pelagic": part("felsic", "plate_surface"),
        "accreted_ocean": per(g.get("accreted", 0.0) - g.get("accreted_continent", 0.0)),
        "sed_subducted": per(g.get("sed_subducted", 0.0)),
        "forearc": per(g.get("forearc_c", 0.0) + g.get("forearc_o", 0.0)),
        "forearc_c": per(g.get("forearc_c", 0.0)),
        "forearc_short": per(g.get("forearc_short", nan)),
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
    # By the arcs' setting at onset (PLT-760, runs from 2026-10-31 on): the intra-oceanic arcs'
    # front past incipient and the island law's rock (cones included) over it.
    band = g.get("arc_band", nan)
    fronts = g.get("front_o", 0.0) + g.get("front_c", 0.0)
    out["band_frac"] = band / fronts if fronts > 0 else nan
    arcs_rock = g.get("arc_o", 0.0) + g.get("arc_c", 0.0) + g.get("arc_cones_o", 0.0) + g.get("arc_cones_c", 0.0)
    out["arc_per_band"] = arcs_rock / band if band > 0 else nan
    # Per km of the oceanic fronts (front_o + front_c, km x Myr): the trench's input and what goes
    # down, against Clift et al. 2009's Table 1 (40,225 km: 1,768,761 km3/Myr delivered, 1,648,091
    # subducted; 44.0 and 41.0 km3/km/Myr) and Stern & Scholl's 25 (Scholl & von Huene 2009 as
    # they quote it, over ~43,000 km); the erosion per km of erosive front (Clift et al. 2009: 1,331,600
    # km3/Myr over the 19 erosive margins' 22,925 km, 58.1).
    out["sed_in_per_km"] = g.get("sed_trench", nan) / fronts if fronts > 0 else nan
    out["sed_down_per_km"] = g.get("sed_subducted", nan) / fronts if fronts > 0 else nan
    efront = g.get("forearc_front_c", 0.0) + g.get("forearc_front_o", 0.0)
    out["erosion_per_km"] = (g.get("forearc_c", 0.0) + g.get("forearc_o", 0.0)) / efront if efront > 0 else nan
    fio = g.get("front_io", nan)
    out["front_io_km"] = fio / length
    out["arc_io_per_km"] = (g.get("arc_o", 0.0) + g.get("arc_cones_o", 0.0)) / fio if fio > 0 else nan
    out["v_n_o"] = g.get("front_n_o", nan) / max(g.get("front_o", 0.0), 1e-9)
    out["v_n_c"] = g.get("front_n_c", nan) / max(g.get("front_c", 0.0), 1e-9)
    out["cord_per_km"] = g.get("cordillera", 0.0) / max(g.get("front_c", 0.0), 1e-9)
    # The hotspot lane (PLT-795, PLT-799): plume heads' provinces per 100 Myr, the mean
    # province, and the volcanic margins' gap points.
    nlip = g.get("lip_n", nan)
    out["lip_per_100myr"] = nlip / length * 100.0
    out["lip_mean_km3"] = g.get("lip", nan) / nlip if nlip > 0 else nan
    out["vrm_n"] = g.get("vrm_n", nan)
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
    ("crust-suturing zones (arc magma at collision fronts, PLT-763)", "suture_arcs", 0.2),
    ("rifts, hotspots, rifted margins (on continental crust)", "hotspots", 0.5),
    ("  model only: the arcs' felsic cones, in both arc rows", "arc_cones", float("nan")),
    ("  of it the tails (basalt and cones, LIP domes out)", "tails_c", float("nan")),
    ("  of it LIPs on continental crust, whole (PLT-795)", "lip_c", float("nan")),
    ("  of it volcanic rifted margins (PLT-799)", "vrm", float("nan")),
    ("  model only: LIPs, all, whole (PLT-795)", "lip_all", float("nan")),
    ("  model only: hotspots, all (their basalt and cones)", "hotspots_all", float("nan")),
    ("  model only: the volcano stage's cones, all", "cones_all", float("nan")),
    ("model only: ocean floor's own crust made continental", "ocean_floor", 0.0),
    ("model only: pelagic sediment made", "pelagic", 0.0),
    ("losses", None, None),
    ("ocean margins: sediment subduction (PLT-440, PLT-660)", "sed_subducted", 1.1),
    ("  model only: the floor's sediment reaching the trenches", "sed_trench", float("nan")),
    ("  model only: of it the one-way slab's (all subducted)", "sed_one_way", float("nan")),
    ("ocean margins: subduction erosion (PLT-441)", "forearc", 1.4),
    ("  of it continental crust", "forearc_c", float("nan")),
    ("  model only: asked and not there (the band short)", "forearc_short", float("nan")),
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
    ("mean v_n at island-arc fronts, mm/yr (PLT-666: 86.6)", "v_n_o", "{:.1f}"),
    ("mean v_n at continental-arc fronts, mm/yr (PLT-666: 58.6)", "v_n_c", "{:.1f}"),
    ("intra-oceanic arcs' front past incipient, km (PLT-760)", "front_io_km", "{:.0f}"),
    ("island law's rock per km of it, km3/km/Myr", "arc_io_per_km", "{:.1f}"),
    ("cordillera per km of continental front, km3/km/Myr", "cord_per_km", "{:.1f}"),
    ("plume heads' LIPs per 100 Myr (PLT-795)", "lip_per_100myr", "{:.2f}"),
    ("mean LIP, km3 (whole)", "lip_mean_km3", "{:.3g}"),
    ("volcanic-margin gap points over the run (PLT-799)", "vrm_n", "{:.0f}"),
    ("trench input per km of front, km3/km/Myr (Clift et al. 2009: 44.0)", "sed_in_per_km", "{:.1f}"),
    ("sediment subducted per km, km3/km/Myr (Clift 2009: 41.0; S&S: 25)", "sed_down_per_km", "{:.1f}"),
    ("erosion per km of erosive front, km3/km/Myr (Clift 2009: 58.1)", "erosion_per_km", "{:.1f}"),
    ("arc band / arc fronts (the fronts with an arc, PLT-554)", "band_frac", "{:.3f}"),
    ("arcs' rock per km of band, km3/km/Myr (Earth 2.5 AU / 42,000 km = 60)", "arc_per_band", "{:.1f}"),
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
    since = None
    if "--from" in args:
        i = args.index("--from")
        since = float(args[i + 1])
        del args[i : i + 2]
    length = float(args[0])
    if since is not None:
        length = -since
    runs = []
    for p in args[1:]:
        r = read(p, since)
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
