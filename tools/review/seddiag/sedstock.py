#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Where the sea's sediment lies, and how much of it the floor's age explains:
sedstock.py <arm folder>[@N] [map name, default t0000.bin | all]

The sediment-excess diagnosis (2026-10-25: the sea holds 1.02e9 km3 of sediment and sedimentary rock on m5-blendoff
against GlobSed's 3.37e8). Reads each run's crust-budget maps (`DEEPTIME_CRUST_BUDGET_MAPS`, plates/crust_budget.rs):
maps/t*.bin (11 f32: lat, lon, z, T, kind, orogeny, its age, island-arc magma, continental-arc magma, group, flags),
maps/age/t*.bin (f32 floor age, NaN on continental crust) and maps/pools/t*.bin (9 f32: the 7 pools, the basement,
formed_myr), all in visible-sample order, and the run's sea level at the map's time from rec.json.

The measure is m5cont.py's "sea's sediment": the sediment and sedimentary pools (pools 0 and 1) of every sample below
the sea, both kinds of crust (GlobSed's grid covers the margins too). Volumes are Earth-normalised km3 (planet-mean m
x 5.1007e5). It is split
  - by crust kind, and the oceanic samples by their crust's thickness (the M4 lane's bands, m5thickoc.py);
  - by floor age (maps/age);
  - by distance to the nearest continental sample of any plate (km on the 6,370 km sphere);
  - into the plates' pelagic rain (5 m/Myr x age, PLT-103, capped at the pile) and the rest (terrigenous: the surface
    stage's deposits and what resamplings accreted).
Counterfactuals (measurement only):
  - "earth ages": the model's mean pile per oceanic sample in 10-Myr age bins, weighted by the steady-state
    triangular area-age distribution dA/dt ~ (1 - t/t_m) with t_m = 3 x the mean age given (default Seton et al.
    2020's 64.2 Myr), over the model's own wet oceanic area. The ratio to the model's oceanic total is the part the
    age distribution explains (with the model's thickness-age relation kept).
  - "compacted": each wet sample's pile compacted with an exponential porosity-depth law phi = phi0 exp(-z/lam)
    from a deposited porosity phi0 (the model never compacts: pools.rs diagenesis moves sediment to sedimentary rock
    metre for metre, and GlobSed's thickness is the compacted pile in place). phi0 and lam on the command line:
    PHI0=0.63 LAM_KM=1.96 (shale, Sclater & Christie 1980, from memory until read).
Medians over runs. Needs numpy and scipy."""
import json
import os
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

MAP, POOLS = 11, 9
R_KM = 6370.0
KM3_PER_M = 5.1007e5  # planet-mean m to km3 on Earth's 5.1007e8 km2
PELAGIC = 5.0  # m/Myr, PLT-103
EARTH_MEAN_AGE = float(os.environ.get("EARTH_MEAN_AGE", "64.2"))  # Seton et al. 2020 (to check)
PHI0 = float(os.environ.get("PHI0", "0.63"))
LAM_KM = float(os.environ.get("LAM_KM", "1.96"))
THICK = [(0, 8), (8, 10), (10, 15), (15, 25), (25, 1e9)]
AGES = [(0, 20), (20, 60), (60, 120), (120, 180), (180, 300), (300, 1e9)]
DIST = [(0, 100), (100, 200), (200, 500), (500, 1000), (1000, 1e9)]
PILE = [(0, 0.5), (0.5, 1), (1, 2), (2, 5), (5, 10), (10, 1e9)]


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def sea_at(run, t):
    steps = json.loads((run / "rec.json").read_text()).get("steps") or []
    best = min(steps, key=lambda s: abs(float(s["t"]) - t))
    return float(best["sea_m"]), float(best["t"])


def compacted_km(d_km):
    """Height a pile deposited d_km thick at porosity PHI0 stands at once compacted: the solid it holds,
    d (1 - phi0), equals the integral of 1 - phi0 exp(-z/lam) from 0 to H (Newton's method, vectorised)."""
    s = d_km * (1.0 - PHI0)
    h = np.maximum(s, 1e-9)
    for _ in range(40):
        f = h - PHI0 * LAM_KM * (1.0 - np.exp(-h / LAM_KM)) - s
        h = np.maximum(h - f / (1.0 - PHI0 * np.exp(-h / LAM_KM)), 1e-12)
    return np.where(d_km > 0, h, 0.0)


def one(run, name):
    t = float(name[1:-4])
    a = np.fromfile(run / "maps" / name, "<f4").reshape(-1, MAP).astype(np.float64)
    age = np.fromfile(run / "maps" / "age" / name, "<f4").astype(np.float64)
    pp = np.fromfile(run / "maps" / "pools" / name, "<f4").reshape(-1, POOLS).astype(np.float64)
    n = len(a)
    assert len(age) == n and len(pp) == n, (run, name)
    sea, t_rec = sea_at(run, t)
    z, cont = a[:, 2], a[:, 4] > 0.5
    oc = ~cont
    wet = z < sea
    pile = np.maximum(pp[:, 0], 0) + np.maximum(pp[:, 1], 0)
    T = pp[:, :7].sum(1)
    agef = np.where(oc, np.nan_to_num(age, nan=0.0), np.nan)
    p = xyz(a[:, 0], a[:, 1])
    dist = np.zeros(n)
    if cont.any():
        d, _ = cKDTree(p[cont]).query(p[oc])
        dist[oc] = 2 * R_KM * np.arcsin(np.minimum(d / 2, 1.0))
    pel = np.where(oc, np.minimum(pile, PELAGIC * np.nan_to_num(agef)), 0.0)
    km3 = lambda m: float(m.sum()) / n * KM3_PER_M
    w = wet
    out = {"t": t, "t_rec": t_rec, "sea_m": sea, "wet": float(w.mean()), "oc": float(oc.mean()),
           "sea_sed": km3(pile * w), "sea_sed_oc": km3(pile * (w & oc)), "sea_sed_cont": km3(pile * (w & cont)),
           "dry_sed": km3(pile * ~w), "meta_wet_oc": km3(pp[:, 2] * (w & oc)),
           "pel": km3(pel * w), "terr_oc": km3((pile - pel) * (w & oc)),
           "age_mean": float(np.nanmean(agef[oc])), "age_old180": float((agef[oc] > 180).mean()),
           "age_mean_wet": float(np.nanmean(agef[oc & w]))}
    for lo, hi in THICK:
        m = w & oc & (T >= lo * 1e3) & (T < hi * 1e3)
        out[f"T{lo}"] = km3(pile * m)
        out[f"T{lo}:n"] = float(m.sum()) / max(int((w & oc).sum()), 1)
        out[f"T{lo}:pile"] = float(pile[m].mean()) if m.any() else float("nan")
    for lo, hi in AGES:
        m = w & oc & (agef >= lo) & (agef < hi)
        out[f"A{lo}"] = km3(pile * m)
        out[f"A{lo}:n"] = float(m.sum()) / max(int((w & oc).sum()), 1)
        out[f"A{lo}:pile"] = float(pile[m].mean()) if m.any() else float("nan")
        out[f"A{lo}:terr"] = km3((pile - pel) * m)
    for lo, hi in DIST:
        m = w & oc & (dist >= lo) & (dist < hi)
        out[f"D{lo}"] = km3(pile * m)
        out[f"D{lo}:n"] = float(m.sum()) / max(int((w & oc).sum()), 1)
        out[f"D{lo}:pile"] = float(pile[m].mean()) if m.any() else float("nan")
    for lo, hi in PILE:
        m = w & (pile >= lo * 1e3) & (pile < hi * 1e3)
        out[f"P{lo}"] = km3(pile * m)
        out[f"P{lo}:n"] = float(m.sum()) / max(int(w.sum()), 1)
    # The age counterfactual: the model's pile-by-age curve on Earth's steady-state area-age distribution.
    tm = 3.0 * EARTH_MEAN_AGE
    edges = np.arange(0.0, max(float(np.nanmax(agef[oc])) + 10.0, tm + 10.0), 10.0)
    mo = w & oc
    idx = np.clip(np.digitize(agef[mo], edges) - 1, 0, len(edges) - 2)
    sums = np.bincount(idx, weights=pile[mo], minlength=len(edges) - 1)
    cnt = np.bincount(idx, minlength=len(edges) - 1)
    mean_bin = np.where(cnt > 0, sums / np.maximum(cnt, 1), np.nan)
    mid = 0.5 * (edges[:-1] + edges[1:])
    wt = np.clip(1.0 - mid / tm, 0.0, None)
    ok = np.isfinite(mean_bin) & (wt > 0)
    earth_mean_pile = float((mean_bin[ok] * wt[ok]).sum() / wt[ok].sum())
    out["earth_ages_oc"] = earth_mean_pile * float(mo.sum()) / n * KM3_PER_M
    out["earth_ages_ratio"] = out["earth_ages_oc"] / max(out["sea_sed_oc"], 1e-9)
    out["pel_earth_ages"] = PELAGIC * EARTH_MEAN_AGE * float(mo.sum()) / n * KM3_PER_M
    # Compaction: the pile as it would stand compacted in place.
    comp = compacted_km(pile / 1e3) * 1e3
    out["sea_sed_compacted"] = km3(comp * w)
    out["compacted_ratio"] = out["sea_sed_compacted"] / max(out["sea_sed"], 1e-9)
    # Thick oceanic-labelled crust (15 km and more): its share of the sea's sediment, its arc magma.
    m = w & oc & (T >= 15e3)
    out["thick_iarc_km"] = float(a[m, 7].mean() / 1e3) if m.any() else float("nan")
    out["thick_meta_km"] = float(pp[m, 2].mean() / 1e3) if m.any() else float("nan")
    out["thick_felsic_km"] = float((pp[m, 3] + pp[m, 4]).mean() / 1e3) if m.any() else float("nan")
    out["thick_dist_p50"] = float(np.median(dist[m])) if m.any() else float("nan")
    out["thick_age_p50"] = float(np.nanmedian(agef[m])) if m.any() else float("nan")
    return out


def table(rows, name):
    keys = rows[0][1].keys()
    md = {k: statistics.median(r[k] for _, r in rows if not (isinstance(r[k], float) and np.isnan(r[k])))
          if any(not (isinstance(r[k], float) and np.isnan(r[k])) for _, r in rows) else float("nan") for k in keys}
    lo = {k: min(r[k] for _, r in rows) for k in ("sea_sed",)}
    hi = {k: max(r[k] for _, r in rows) for k in ("sea_sed",)}
    print(f"{len(rows)} runs, map {name} (t {md['t']:.0f}); Earth-normalised km3, medians")
    print(f"  sea {md['sea_m']:.0f} m over the datum; wet {md['wet']:.3f}, oceanic {md['oc']:.3f} of the planet;"
          f" floor age mean {md['age_mean']:.1f} Myr (wet {md['age_mean_wet']:.1f}), older than 180 {md['age_old180']:.3f}")
    print(f"  the sea's sediment {md['sea_sed']:.3g} [{lo['sea_sed']:.3g}, {hi['sea_sed']:.3g}]: on oceanic crust"
          f" {md['sea_sed_oc']:.3g}, on continental crust {md['sea_sed_cont']:.3g}; above the sea {md['dry_sed']:.3g};"
          f" metamorphic on wet oceanic {md['meta_wet_oc']:.3g}")
    print(f"  on oceanic crust: the plates' pelagic rain (5 m/Myr x age) {md['pel']:.3g}, the rest (terrigenous)"
          f" {md['terr_oc']:.3g}")
    print("  wet oceanic by crust thickness, km: " + "; ".join(
        f"{lo}-{hi:g} share {md[f'T{lo}:n']:.3f} pile {md[f'T{lo}:pile']:.0f} m {md[f'T{lo}']:.3g}" for lo, hi in THICK))
    print("  wet oceanic by floor age, Myr: " + "; ".join(
        f"{lo}-{hi:g} share {md[f'A{lo}:n']:.3f} pile {md[f'A{lo}:pile']:.0f} m {md[f'A{lo}']:.3g} (terr {md[f'A{lo}:terr']:.3g})"
        for lo, hi in AGES))
    print("  wet oceanic by distance to a continent, km: " + "; ".join(
        f"{lo}-{hi:g} share {md[f'D{lo}:n']:.3f} pile {md[f'D{lo}:pile']:.0f} m {md[f'D{lo}']:.3g}" for lo, hi in DIST))
    print("  wet samples by their own pile, km: " + "; ".join(
        f"{lo}-{hi:g} share {md[f'P{lo}:n']:.3f} {md[f'P{lo}']:.3g}" for lo, hi in PILE))
    print(f"  earth ages (triangular, mean {EARTH_MEAN_AGE} Myr): oceanic {md['earth_ages_oc']:.3g}, ratio"
          f" {md['earth_ages_ratio']:.3f}; pelagic rain at that mean age {md['pel_earth_ages']:.3g}")
    print(f"  compacted (phi0 {PHI0}, lambda {LAM_KM} km): {md['sea_sed_compacted']:.3g}, ratio {md['compacted_ratio']:.3f}")
    print(f"  oceanic crust 15 km and more: island-arc magma {md['thick_iarc_km']:.1f} km, metamorphic"
          f" {md['thick_meta_km']:.1f} km, felsic {md['thick_felsic_km']:.1f} km, distance p50 {md['thick_dist_p50']:.0f} km,"
          f" floor age p50 {md['thick_age_p50']:.0f} Myr")
    return md


def main():
    folder, _, top = sys.argv[1].partition("@")
    which = sys.argv[2] if len(sys.argv) > 2 else "t0000.bin"
    runs = sorted(q for q in Path(folder).iterdir() if (q / "maps" / "pools").is_dir() and (q / "maps" / "age").is_dir())
    if top:
        runs = [r for r in runs if int(r.name.rsplit("-", 1)[1]) <= int(top)]
    if not runs:
        print("no runs with maps/pools and maps/age")
        return
    names = sorted((p.name for p in (runs[0] / "maps" / "pools").glob("t*.bin")), key=lambda s: float(s[1:-4])) \
        if which == "all" else [which]
    series = []
    for name in names:
        rows = [(r.name, one(r, name)) for r in runs if (r / "maps" / "pools" / name).exists()]
        if which == "all":
            md = {k: statistics.median(x[k] for _, x in rows) for k in
                  ("t", "sea_m", "sea_sed", "sea_sed_oc", "sea_sed_cont", "pel", "terr_oc", "age_mean", "meta_wet_oc",
                   "dry_sed", "oc")}
            series.append(md)
        else:
            print("runs: " + ", ".join(r for r, _ in rows))
            table(rows, name)
    if series:
        print(f"{len(runs)} runs; the sea's sediment over time, Earth-normalised km3 (medians), and its net rate km3/yr")
        print("  t Myr | sea m | oceanic | sea's sediment | on oceanic | on continental | pelagic | terrigenous |"
              " metamorphic wet oc | above sea | age mean | rate")
        prev = None
        for s in series:
            rate = (s["sea_sed"] - prev["sea_sed"]) / ((s["t"] - prev["t"]) * 1e6) if prev else float("nan")
            print(f"  {s['t']:6.0f} | {s['sea_m']:5.0f} | {s['oc']:.3f} | {s['sea_sed']:.3g} | {s['sea_sed_oc']:.3g} |"
                  f" {s['sea_sed_cont']:.3g} | {s['pel']:.3g} | {s['terr_oc']:.3g} | {s['meta_wet_oc']:.3g} |"
                  f" {s['dry_sed']:.3g} | {s['age_mean']:.0f} | {rate:.2f}")
            prev = s


if __name__ == "__main__":
    main()
