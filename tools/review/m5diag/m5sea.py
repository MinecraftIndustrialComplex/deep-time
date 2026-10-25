#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The drowning diagnosis on cached runs: m5sea.py run <out.jsonl> <arm folder> [<arm folder> ...] | table <name=out.jsonl> ...

After the motion fix (PLT-191) land fell 0.319 to 0.193 and the sea over the datum rose 409 to 687 m (m5-unfreeze against
base14). `run` reads each run's final frame (fields/ on the L1 grid, which is equal-area: rows evenly spaced in sin
latitude) and the record's last step (rec.json), and writes one JSON object per run:

1. the shares: continental crust (plates/crust_type), land (the census's z > 0), the continental crust under the sea
   and the ocean, all as shares of the planet;
2. the sea's budget, an identity on the final frame: with h the plate model's height over the datum
   (plates/elevation_m) and the sea S over the datum (climate/sea_level_m), the water over the floor is
   W = mean over the planet of max(S - h, 0). So S = W / a_o + h_o, where a_o is the share of the planet under the sea
   and h_o the floor's mean height over the datum there; h_o = f_c h_c + (1 - f_c) h_oc, the drowned continental
   crust (share f_c of the ocean, mean height h_c) and the oceanic crust (h_oc). The same with the surface's height
   (surface/elevation_m) as h, to see which height the sea is solved over. A change in S between arms splits exactly
   into the water term and the floor term, and the floor term into its two crusts' shares and heights;
3. the ocean against Earth's: its mean depth S - h_o, the oceanic crust's mean depth S - h_oc, the floor's mean age
   (the record's last `ages` histogram, 10-Myr bins) and its p50, the land ice as metres of water over the planet
   (ice 917 over water 1,000 kg/m3);
4. the continental crust by thickness band (all of it, the drowned part included): the band's share of the
   continental crust, the land share in it, the plate height over the sea p25 / p50 / p75, the census height p50,
   the Airy height over the sea (m4split.airy) p50, the share above 2 km;
5. the oceanic crust by thickness band (oc_bands: share, mean height) and the whole sea floor by the crust's thickness
   under it (wet_bands: share of the sea's area, mean depth below the sea, the continental share in the band), to set
   against CRUST1.0's (m5earthbands.py 2b);
6. the height histograms of the continental and the oceanic crust (hist_cont, hist_oc) and the water layer less land
   ice (water_layer_m: ISO-085's 2,664 m, ice at CLM-210's 917 over 1,028 kg/m3).

`table` prints the medians over the runs, arm beside arm, and splits the change in the sea between the first arm and
each other one by Shapley over four factors (the water, the continental area, the continental heights, the oceanic
heights), each sea refilled from the histograms (fill); and the floor's age alone, by moving the first arm's oceanic
histogram by the change in GDH1's depth over the two age records (PLT-352). Needs numpy only."""
import contextlib
import io
import json
import statistics
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import continents_census as cc  # noqa: E402
from m4split import airy  # noqa: E402

BANDS = [(0, 25), (25, 30), (30, 35), (35, 40), (40, 45), (45, 50), (50, 55), (55, 70), (70, 1e9), (35, 45)]
OC_BANDS = [(0, 6), (6, 8), (8, 10), (10, 15), (15, 25), (25, 1e9)]
WET_BANDS = [(0, 8), (8, 10), (10, 15), (15, 25), (25, 1e9)]
RHO_ICE_OVER_WATER = 917.0 / 1000.0
RHO_ICE, RHO_SEA = 917.0, 1028.0  # ledger: CLM-210 (climate/ice.rs RHO_ICE, RHO_SEA)
WATER_LAYER_M = 2664.0  # ledger: ISO-085 (planet.surface_water_m in earthlike_quick_16k)
AGE_BIN_MYR = 10.0
HIST_EDGES = np.arange(-12000.0, 9000.0 + 1.0, 25.0)


def wq(x, q):
    return float(np.quantile(x, q)) if x.size else None


def gdh1(t):
    """GDH1's depth below the datum for floor of age t Myr, m (PLT-352; Stein & Stein 1992)."""
    return 2600.0 + 365.0 * np.sqrt(t) if t < 20.0 else 5651.0 - 2473.0 * np.exp(-0.0278 * t)


def fill(W, f, hc, ho):
    """The sea S over the datum at which water W (m over the planet) fills every cell below S, on the crusts' height
    histograms (shares of each crust, HIST_EDGES) with the continental share f. No open-sea rule: closed hollows
    fill too, and the steering's offset is not in it (both land in the residual against the run's sea)."""
    c = 0.5 * (HIST_EDGES[1:] + HIST_EDGES[:-1])
    lo, hi = HIST_EDGES[0], HIST_EDGES[-1]
    for _ in range(60):
        s = 0.5 * (lo + hi)
        w = f * (hc * np.maximum(s - c, 0.0)).sum() + (1.0 - f) * (ho * np.maximum(s - c, 0.0)).sum()
        lo, hi = (s, hi) if w < W else (lo, s)
    return 0.5 * (lo + hi)


def factors(rows):
    def norm(k):
        h = np.mean([np.asarray(r[k], float) / max(sum(r[k]), 1) for r in rows], axis=0)
        return h / h.sum()
    return {"W": float(np.mean([r["water_layer_m"] for r in rows])), "f": float(np.mean([r["cont"] for r in rows])),
            "hc": norm("hist_cont"), "ho": norm("hist_oc")}


def shapley(base, arm):
    """Each factor's share of the change in the filled sea, averaged over every order of switching them."""
    import itertools
    keys = ["W", "f", "hc", "ho"]
    part = {k: 0.0 for k in keys}
    perms = list(itertools.permutations(keys))
    for p in perms:
        cur = dict(base)
        s0 = fill(cur["W"], cur["f"], cur["hc"], cur["ho"])
        for k in p:
            cur[k] = arm[k]
            s1 = fill(cur["W"], cur["f"], cur["hc"], cur["ho"])
            part[k] += (s1 - s0) / len(perms)
            s0 = s1
    return part


def budget(h, sea, cont):
    """The sea's identity on one height field h over the datum: S = W / a_o + f_c h_c + (1 - f_c) h_oc."""
    wet = h < sea
    a_o = float(wet.mean())
    W = float(np.where(wet, sea - h, 0.0).mean())
    h_o = float(h[wet].mean())
    dc, do = wet & cont, wet & ~cont
    f_c = float(dc.sum() / max(int(wet.sum()), 1))
    h_c = float(h[dc].mean()) if dc.any() else 0.0
    h_oc = float(h[do].mean()) if do.any() else 0.0
    return {"a_o": a_o, "W_m": W, "W_over_a_o": W / a_o, "h_o": h_o, "f_c": f_c, "h_c": h_c, "h_oc": h_oc,
            "check": W / a_o + h_o - sea, "depth_mean": sea - h_o, "oc_depth_mean": sea - h_oc,
            "dc_depth_mean": sea - h_c}


def one(run):
    with contextlib.redirect_stdout(io.StringIO()):
        f, g, e = cc.load_dump(run / "fields")
    h_, w_, r_ = e["h"], e["w"], e["r"]
    _, ok_r = cc.lat_of_rows(h_, r_, w_ * r_)
    ok = np.repeat(ok_r[:, None], w_, 1)
    sea = float(g["globals"]["climate/sea_level_m"])
    z = (f["elevation"][ok] - cc.SEA_Y) * cc.M_PER_BLOCK_V
    hp = np.nan_to_num(f["plates/elevation_m"][ok])
    hs = np.nan_to_num(f["surface/elevation_m"][ok]) if "surface/elevation_m" in f else None
    T = np.nan_to_num(f["plates/thickness_m"][ok]) / 1000.0
    cont = np.nan_to_num(f["plates/crust_type"][ok]) > 0.5
    ice = np.nan_to_num(f["climate/ice_thickness_m"][ok])
    land = z > 0
    zp = hp - sea
    out = {"run": run.name, "sea_m": sea, "cont": float(cont.mean()), "land": float(land.mean()),
           "land_over_cont": float(land.mean() / max(cont.mean(), 1e-9)),
           "cont_under_sea": float((cont & ~land).sum() / max(int(cont.sum()), 1)),
           "ocean": float((~land).mean()), "ice_water_m": float(ice.mean() * RHO_ICE_OVER_WATER),
           "plate": budget(hp, sea, cont)}
    if hs is not None:
        out["surface"] = budget(hs, sea, cont)
        out["surface_minus_plate_p50_cont"] = wq((hs - hp)[cont], 0.5)
        out["surface_minus_plate_mean_ocean"] = float((hs - hp)[~cont].mean())
    out["T_cont_p10_p50_p90"] = [wq(T[cont], q) for q in (0.1, 0.5, 0.9)]
    out["T_cont_mean"] = float(T[cont].mean())
    bands = {}
    C = max(int(cont.sum()), 1)
    for a, b in BANDS:
        m = cont & (T >= a) & (T < b)
        n = int(m.sum())
        o = {"share": n / C}
        if n:
            o.update({"land": float((m & land).sum() / n), "zp": [wq(zp[m], q) for q in (0.25, 0.5, 0.75)],
                      "z_p50": wq(z[m], 0.5), "airy_p50": wq(airy(T[m] * 1000.0) - sea, 0.5),
                      "above_2km": float((m & (z > 2000)).sum() / n), "T_p50": wq(T[m], 0.5)})
        bands[f"{a:g}-{b:g}"] = o
    out["bands"] = bands
    # The oceanic crust by its thickness (pools included): its share of the oceanic crust and its mean height.
    oc = {}
    O = max(int((~cont).sum()), 1)
    for a, b in OC_BANDS:
        m = ~cont & (T >= a) & (T < b)
        n = int(m.sum())
        oc[f"{a:g}-{b:g}"] = {"share": n / O, "h_mean": float(hp[m].mean()) if n else None}
    out["oc_bands"] = oc
    # The sea's floor by the crust's thickness under it, both kinds (Earth's in m5earthbands.py, 2b): the share of
    # the sea's area, the mean depth below the sea, the continental share of the band.
    wet = hp < sea
    Wn = max(int(wet.sum()), 1)
    wb = {}
    for a, b in WET_BANDS:
        m = wet & (T >= a) & (T < b)
        n = int(m.sum())
        wb[f"{a:g}-{b:g}"] = {"share": n / Wn, "depth": float((sea - hp[m]).mean()) if n else None,
                              "cont": float((m & cont).sum() / n) if n else None}
    out["wet_bands"] = wb
    out["T_oc_p10_p50_p90"] = [wq(T[~cont], q) for q in (0.1, 0.5, 0.9)]
    # The plate heights' histograms by crust (shares of each crust's cells), for the sea's factor split.
    out["hist_cont"] = np.histogram(hp[cont], HIST_EDGES)[0].tolist()
    out["hist_oc"] = np.histogram(hp[~cont], HIST_EDGES)[0].tolist()
    out["water_layer_m"] = WATER_LAYER_M - float(ice.mean()) * RHO_ICE / RHO_SEA
    rec_f = run / "rec.json"
    if rec_f.exists():
        rec = json.loads(rec_f.read_text())
        out["floor_age_p50"] = rec["steps"][-1].get("age_p50")
        if rec.get("ages"):
            a = np.asarray(rec["ages"][-1]["hist"], float)
            mid = (np.arange(a.size) + 0.5) * AGE_BIN_MYR
            out["floor_age_mean"] = float((a * mid).sum() / max(a.sum(), 1e-9))
            out["floor_old180"] = float(a[mid >= 180.0].sum() / max(a.sum(), 1e-9))
            out["floor_gdh1_m"] = float((a * np.array([gdh1(t) for t in mid])).sum() / max(a.sum(), 1e-9))
    return out


def run_cmd(out_path, folders):
    """Each folder may end in @N to keep the seeds 1 to N only (base14's runs that match an 8-run arm's)."""
    with open(out_path, "w") as fo:
        for folder in folders:
            folder, _, top = folder.partition("@")
            for run in sorted(p for p in Path(folder).iterdir() if (p / "fields" / "index.json").exists()):
                if top and int(run.name.rsplit("-", 1)[1]) > int(top):
                    continue
                fo.write(json.dumps(one(run)) + "\n")
                fo.flush()


def med(rows, path):
    vals = []
    for r in rows:
        v = r
        for k in path:
            if isinstance(v, dict):
                v = v.get(k)
            elif isinstance(v, list) and isinstance(k, int) and k < len(v):
                v = v[k]
            else:
                v = None
            if v is None:
                break
        if isinstance(v, (int, float)):
            vals.append(float(v))
    return statistics.median(vals) if vals else float("nan")


def table(args):
    arms = []
    for a in args:
        name, p = a.split("=", 1)
        arms.append((name, [json.loads(x) for x in Path(p).read_text().splitlines() if x.strip()]))
    print("medians over each arm's runs: " + ", ".join(f"{n} {len(r)}" for n, r in arms))

    def line(label, path, fmt="{:.3f}"):
        print(f"  {label:42s}" + "".join(f"{fmt.format(med(r, path)):>12s}" for _, r in arms))

    print(f"  {'':42s}" + "".join(f"{n:>12s}" for n, _ in arms))
    for k in ("sea_m", "cont", "land", "land_over_cont", "cont_under_sea", "ocean", "ice_water_m",
              "floor_age_mean", "floor_age_p50", "floor_age_p50_hist", "T_cont_mean"):
        line(k, [k], "{:.1f}" if k.endswith("_m") or k.startswith(("floor", "T_")) else "{:.3f}")
    for i, q in enumerate(("p10", "p50", "p90")):
        line(f"T_cont {q} km", ["T_cont_p10_p50_p90", i], "{:.1f}")
    for frame in ("plate", "surface"):
        print(f" the sea's identity over the {frame} height: S = W / a_o + f_c h_c + (1 - f_c) h_oc")
        for k in ("a_o", "W_m", "W_over_a_o", "h_o", "f_c", "h_c", "h_oc", "check", "depth_mean",
                  "oc_depth_mean", "dc_depth_mean"):
            line(f"{frame}.{k}", [frame, k], "{:.3f}" if k in ("a_o", "f_c") else "{:.0f}")
    line("surface - plate, continental p50 m", ["surface_minus_plate_p50_cont"], "{:.0f}")
    line("surface - plate, oceanic mean m", ["surface_minus_plate_mean_ocean"], "{:.0f}")
    line("water layer less the land ice m", ["water_layer_m"], "{:.0f}")
    line("floor older than 180 Myr", ["floor_old180"])
    line("GDH1 depth over the floor's ages m", ["floor_gdh1_m"], "{:.0f}")
    for i, q in enumerate(("p10", "p50", "p90")):
        line(f"T_oc {q} km", ["T_oc_p10_p50_p90", i], "{:.1f}")
    print(" the oceanic crust by thickness (pools included): share of it, mean plate height over the datum")
    for a, b in OC_BANDS:
        k = f"{a:g}-{b:g}"
        line(f"  {k} km share", ["oc_bands", k, "share"])
        line(f"  {k} km h mean m", ["oc_bands", k, "h_mean"], "{:.0f}")
    print(" the sea's floor by the crust's thickness under it (both kinds): share of the sea's area, mean depth below"
          " the sea, continental share")
    for a, b in WET_BANDS:
        k = f"{a:g}-{b:g}"
        line(f"  {k} km share", ["wet_bands", k, "share"])
        line(f"  {k} km depth m", ["wet_bands", k, "depth"], "{:.0f}")
        line(f"  {k} km continental", ["wet_bands", k, "cont"])
    fs = [(n, factors(r)) for n, r in arms]
    print(" the sea filled on the arm's mean histograms (every cell below the sea, no steering), against the run mean:")
    for (n, r), (_, fa) in zip(arms, fs):
        s = fill(fa["W"], fa["f"], fa["hc"], fa["ho"])
        mean_s = float(np.mean([x["sea_m"] for x in r]))
        print(f"  {n:12s} filled {s:6.0f} m, run mean {mean_s:6.0f} m, residual (steering, open sea) {mean_s - s:5.0f} m;"
              f" W {fa['W']:.0f} m, cont {fa['f']:.3f}")
    base_n, base_f = fs[0]
    for n, fa in fs[1:]:
        part = shapley(base_f, fa)
        tot = sum(part.values())
        print(f"  {base_n} to {n}: filled sea {tot:+.0f} m = water {part['W']:+.0f} + continental area {part['f']:+.0f}"
              f" + continental heights {part['hc']:+.0f} + oceanic heights {part['ho']:+.0f} (Shapley over the 4 factors)")
        # The floor's age alone: the base's oceanic histogram moved by the change in GDH1's mean depth.
        g0 = float(np.mean([x.get("floor_gdh1_m", np.nan) for x in arms[0][1]]))
        g1 = float(np.mean([x.get("floor_gdh1_m", np.nan) for x in dict(arms)[n]]))
        k = int(round(-(g1 - g0) / (HIST_EDGES[1] - HIST_EDGES[0])))
        ho = np.roll(base_f["ho"], k)
        if k > 0:
            ho[:k] = 0.0
        elif k < 0:
            ho[k:] = 0.0
        s_age = fill(base_f["W"], base_f["f"], base_f["hc"], ho / ho.sum()) - fill(base_f["W"], base_f["f"],
                                                                                     base_f["hc"], base_f["ho"])
        print(f"    the floor's age alone (GDH1 over the age record, {g0:.0f} to {g1:.0f} m deep): {s_age:+.0f} m on the"
              f" base's hypsometry; the oceanic floor's mean height {np.mean([x['plate']['h_oc'] for x in arms[0][1]]):.0f}"
              f" to {np.mean([x['plate']['h_oc'] for x in dict(arms)[n]]):.0f} m")
    print(" continental crust by thickness band: share, land share, plate height over the sea p25/p50/p75, census z p50,"
          " Airy over the sea p50, above 2 km")
    for a, b in BANDS:
        k = f"{a:g}-{b:g}"
        print(f"  {k}")
        line("   share", ["bands", k, "share"])
        line("   land", ["bands", k, "land"])
        for i, q in enumerate(("p25", "p50", "p75")):
            line(f"   plate over the sea {q} m", ["bands", k, "zp", i], "{:.0f}")
        line("   census z p50 m", ["bands", k, "z_p50"], "{:.0f}")
        line("   Airy over the sea p50 m", ["bands", k, "airy_p50"], "{:.0f}")
        line("   above 2 km", ["bands", k, "above_2km"])


if __name__ == "__main__":
    if sys.argv[1] == "run":
        run_cmd(sys.argv[2], sys.argv[3:])
    else:
        table(sys.argv[2:])
