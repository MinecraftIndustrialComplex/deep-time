#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""floorage.py <arm folder>...: where the old sea floor sits and what subduction initiation sees there.

Reads each run folder's rec.json, summary.txt and crust-budget maps (maps/t*.bin, maps/plate/t*.bin). Prints one
JSON object per run on stdout. Needs numpy and scipy (the VM's pool numpy env).

The record part (rec.json: the age histograms, the resamplings' consumed and escaped floor, the cohorts' hazard by
age, the subduction front's flux, the plates' lives) is exact.

The maps carry no floor age, so the map part reads a bare oceanic sample's age off its elevation through GDH1 (pools.rs
gdh1_floor_m at the shipped depths -2600 and -5651 m), with a check per snapshot against the record's old180 and age
histogram at the same time. The check FAILS (2026-10-25: the proxy finds 0.14 of the record's old floor on the twin
fl-off-env8, 0.23 on base14), so the map part's age columns ("old", "status_old180", "plates_eligible", "lambda_*")
are not to be used; its areas ("ocean" by distance, plate size, continental share, thickness; front_sub_km) are
exact."""
import json
import math
import re
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

FIELDS = 11
R_KM = 6371.0
TRENCH, ARC, HOT, DIV, SZ, FORE = 2, 4, 8, 16, 64, 128
FRONT_SUB, FRONT_COLL = 256, 512
CLEAR_KM = 1000.0  # PLT-064
MARGIN_KM = 200.0 + 60.0  # PLT-503: arc distance + width, plus one spacing (added per run)
INIT_AGE = 150.0  # PLT-062
RATE = 0.15  # PLT-063, per Myr per A0
A0_SHARE = 1.0 / 14.0
AGE_EDGES = [0, 10, 30, 60, 100, 150, 180, 250, 1e9]
DC_EDGES = [0, 330, 1000, 2000, 1e9]
DT_EDGES = [0, 500, 1000, 1500, 1e9]
DR_EDGES = [0, 500, 1000, 2000, 1e9]
PS_EDGES = [0, 0.02, 0.05, 0.10, 0.20, 1.01]
PC_EDGES = [0, 0.001, 0.1, 0.3, 1.01]
TH_EDGES = [0, 10, 25, 1e9]


def gdh1(t):
    t = np.maximum(t, 0.0)
    return np.where(t < 20.0, 2600.0 + 365.0 * np.sqrt(t), 5651.0 - 2473.0 * np.exp(-0.0278 * t))


def inv_gdh1(z):
    d = -z
    a = np.zeros_like(d)
    knee = 2600.0 + 365.0 * math.sqrt(20.0)
    m1 = (d > 2600.0) & (d < knee)
    a[m1] = ((d[m1] - 2600.0) / 365.0) ** 2
    m2 = (d >= knee) & (d < 5651.0)
    a[m2] = -np.log((5651.0 - d[m2]) / 2473.0) / 0.0278
    a[d >= 5651.0] = 1e4
    return a


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def km(chord):
    return 2.0 * R_KM * np.arcsin(np.minimum(chord / 2.0, 1.0))


def nearest_km(p_from, p_to):
    if len(p_to) == 0:
        return np.full(len(p_from), np.inf)
    d, _ = cKDTree(p_to).query(p_from)
    return km(d)


def band(x, edges):
    return np.clip(np.searchsorted(edges, x, side="right") - 1, 0, len(edges) - 2)


def counts(idx, n):
    return np.bincount(idx, minlength=n).astype(int).tolist()


def hist_bands(hist):
    """The record's 10-Myr bins (41, the last open) into AGE_EDGES' bands."""
    out = [0] * (len(AGE_EDGES) - 1)
    for b, c in enumerate(hist):
        out[int(band(np.array([b * 10.0 + 5.0]), AGE_EDGES)[0])] += c
    return out


def summary_tokens(text):
    out = {}
    for line in text.splitlines():
        if "plates_final=" in line or "margins records=" in line:
            for k, v in re.findall(r"(\w+)=([-0-9.e]+)", line):
                try:
                    out[k] = float(v)
                except ValueError:
                    pass
    return out


def rec_part(r, summ):
    out = {}
    N = r["samples"]
    steps = r["steps"]
    ages = r["ages"]
    late = [s for s in steps if s["t"] >= -500.0]
    # The ages over time, every 50 Myr.
    ser = []
    for a in ages:
        if abs(a["t"] / 50.0 - round(a["t"] / 50.0)) > 1e-6:
            continue
        h = np.array(a["hist"], float)
        n = max(h.sum(), 1.0)
        c = np.arange(len(h)) * 10.0 + 5.0
        ser.append({"t": a["t"], "mean_lb": float((h * c).sum() / n), "p50": a["p50_myr"],
                    "s150": float(h[15:].sum() / n), "s180": float(h[18:].sum() / n), "s250": float(h[25:].sum() / n),
                    "y10": float(h[0] / n), "ocean": int(n)})
    out["age_series"] = ser
    # The resamplings: the steps whose crust-shown record consumed hidden samples. New ocean floor is made only there
    # (ridges.rs: the gaps the plates opened since the last one are filled, ages 0 to the interval).
    res = [i for i, s in enumerate(steps) if (s.get("shown") or {}).get("consumed", 0) > 0]
    res_t = [steps[i]["t"] for i in res]
    out["resampling_t"] = res_t
    gaps = []
    for j in range(1, len(res)):
        i = res[j]
        if steps[i]["t"] < -500.0:
            continue
        made = N - steps[i - 1]["visible"]
        dtr = steps[i]["t"] - steps[res[j - 1]]["t"]
        gaps.append((made, dtr, steps[i]["ocean"]))
    out["resampling_interval_myr"] = float(np.median([g[1] for g in gaps])) if gaps else None
    # Production: the gap floor made per Myr as a share of the visible ocean after the resampling.
    out["prod_share_per_myr"] = float(sum(g[0] for g in gaps) / sum(g[1] * g[2] for g in gaps)) if gaps else None
    # Cohorts: bin b at t into bin b+1 at t+10, pooled over the pairs in the last 500 Myr with no resampling between
    # them (so no floor is made); the open last bin is left out.
    surv_num = np.zeros(41)
    surv_den = np.zeros(41)
    for a, b in zip(ages, ages[1:]):
        if a["t"] < -500.0 or abs(b["t"] - a["t"] - 10.0) > 1e-6:
            continue
        if any(a["t"] < x <= b["t"] + 1e-6 for x in res_t):
            continue
        h1, h2 = np.array(a["hist"], float), np.array(b["hist"], float)
        surv_den[:39] += h1[:39]
        surv_num[:39] += h2[1:40]
    out["bins_den"] = surv_den[:39].tolist()
    out["bins_num"] = surv_num[:39].tolist()
    coh = []
    for lo, hi in [(0, 1), (1, 3), (3, 6), (6, 10), (10, 15), (15, 18), (18, 25), (25, 39)]:
        den, num = surv_den[lo:hi].sum(), surv_num[lo:hi].sum()
        s = num / den if den > 0 else float("nan")
        # Survival over one 10-Myr step, the band's loss per Myr as a share of the band, and the loss in samples/Myr
        # per 10-Myr bin.
        coh.append({"band": f"{lo * 10}-{hi * 10}", "S10": s, "haz_per_myr": (-math.log(s) / 10.0) if s > 0 else None,
                    "den": den, "loss_per_myr": (den - num) / 10.0 / (hi - lo)})
    out["cohorts"] = coh
    # Ocean area over production: the floor's mean life (A / C0), Myr.
    ocean_late = np.mean([s["ocean"] for s in late])
    out["ocean_share_late"] = float(ocean_late / N)
    out["A_over_C0_myr"] = (1.0 / out["prod_share_per_myr"]) if out["prod_share_per_myr"] else None
    # The subduction front's flux: its km by convergence band times the band's middle, km^2/yr (ocean-ocean,
    # ocean-continent), and its km-weighted convergence.
    mids = [2.5, 7.5, 15.0, 30.0, 60.0, 100.0]
    flux, kms = [], []
    for s in late:
        nb = (s.get("front_bands") or {}).get("normal")
        if not nb:
            continue
        km_ = sum(sum(row) for row in nb)
        fl = sum(row[b] * mids[b] for row in nb for b in range(6)) * 1e-6
        flux.append(fl)
        kms.append(km_)
    out["sub_flux_km2_yr"] = float(np.mean(flux)) if flux else None
    out["sub_front_km"] = float(np.mean(kms)) if kms else None
    out["sub_conv_mm_yr"] = float(np.sum(flux) / np.sum(kms) * 1e6) if kms and np.sum(kms) > 0 else None
    out["normal_bands_late"] = (np.mean(np.array([s["front_bands"]["normal"] for s in late if (s.get("front_bands") or {}).get("normal")], float), 0).tolist()
                                if flux else None)
    # Plate speeds, area-weighted, late.
    sp = [sum(p[1] * p[3] for p in s["plates"]) / max(sum(p[1] for p in s["plates"]), 1) for s in late]
    out["speed_aw_mm_yr"] = float(np.mean(sp))
    # Plates and boundaries in the last 500 Myr.
    mix = np.array([s["mix_km"] for s in late], float)
    out["mix_km_late"] = mix.mean(0).tolist()
    out["plates_late"] = float(np.mean([len(s["plates"]) for s in late]))
    out["front_km_late"] = float(np.mean([sum(p[4] for p in s["plates"]) for s in late]))
    inc = [s.get("front_bands", {}).get("incipient", [0, 0]) for s in late]
    out["incipient_km_late"] = np.mean(np.array(inc, float), 0).tolist()
    ben = [s.get("front_bands", {}).get("benioff", [0, 0]) for s in late]
    out["benioff_km_late"] = np.mean(np.array(ben, float), 0).tolist()
    out["old180_late"] = float(np.mean([s["old180"] / max(s["ocean"], 1) for s in late]))
    out["old250_late"] = float(np.mean([s["old250"] / max(s["ocean"], 1) for s in late]))
    # Within the run, over time: the old floor's share against the plates and the subduction front.
    def spearman(x, y):
        x, y = np.argsort(np.argsort(x)), np.argsort(np.argsort(y))
        return float(np.corrcoef(x, y)[0, 1]) if np.std(x) > 0 and np.std(y) > 0 else None
    o180 = np.array([s["old180"] / max(s["ocean"], 1) for s in late])
    npl = np.array([len(s["plates"]) for s in late], float)
    sub = mix[:, 0] + mix[:, 1]
    rid = mix[:, 3]
    out["rho_old180_plates"] = spearman(o180, npl)
    out["rho_old180_sub"] = spearman(o180, sub)
    out["rho_old180_ridge"] = spearman(o180, rid)
    # Lagged: plates 50 Myr earlier (25 steps of 2 Myr) against the old floor now.
    lag = int(round(50.0 / r["dt_myr"]))
    if len(late) > lag + 10:
        out["rho_old180_plates_lag50"] = spearman(o180[lag:], npl[:-lag])
        out["rho_old180_sub_lag50"] = spearman(o180[lag:], sub[:-lag])
    # What the resamplings consumed: the hidden oceanic floor's mean age against the visible floor's.
    cons = []
    for s in steps:
        sh = s.get("shown") or {}
        if sh.get("consumed", 0) > 0 and s["t"] >= -900.0:
            co = sh["consumed"] - sh.get("consumed_cont", 0)
            cons.append({"t": s["t"], "consumed_ocean": co,
                         "consumed_age": sh["consumed_age_myr"] / max(co, 1),
                         "visible_age": sh.get("ocean_age_myr", 0.0) / max(sh.get("ocean", 0), 1),
                         "escaped_ocean": sh.get("escaped_ocean", 0),
                         "escaped_age": sh.get("escaped_age_myr", 0.0) / max(sh.get("escaped_ocean", 0), 1)})
    out["resamplings"] = cons
    # Plate lives.
    lives = r["plates"]
    by = {}
    for p in lives:
        if p["born_myr"] < -990.0 and p["origin"] == "initial":
            continue
        o = p["origin"]
        d = by.setdefault(o, {"n": 0, "life": [], "fate": {}, "share_at_death": []})
        d["n"] += 1
        end = p["died_myr"] if p.get("died_myr") is not None else 0.0
        d["life"].append(end - p["born_myr"])
        f = p.get("fate") or "alive"
        d["fate"][f] = d["fate"].get(f, 0) + 1
        if p.get("share_at_death") is not None:
            d["share_at_death"].append(p["share_at_death"])
    for o, d in by.items():
        life = sorted(d["life"])
        d["life_p50"] = life[len(life) // 2] if life else None
        d["dead_within_20"] = sum(1 for x in life if x <= 20.0) / max(len(life), 1)
        d["share_at_death_p50"] = sorted(d["share_at_death"])[len(d["share_at_death"]) // 2] if d["share_at_death"] else None
        del d["life"], d["share_at_death"]
    out["lives"] = by
    # Each initiated plate over its life: its area at birth, whether a slab front grew on it, the area it lost, and
    # whom it went to.
    track = {}
    for s in steps:
        for p in s["plates"]:
            track.setdefault(int(p[0]), []).append((s["t"], p[1], p[4]))
    ini = []
    for p in lives:
        if p["origin"] != "subduction_initiation" or p["born_myr"] < -900.0:
            continue
        tr = track.get(p["id"], [])
        if not tr:
            ini.append({"born": p["born_myr"], "seen": False, "fate": p.get("fate")})
            continue
        a0 = tr[0][1]
        front20 = max((x[2] for x in tr if x[0] - p["born_myr"] <= 20.0), default=0.0)
        front_any = max(x[2] for x in tr)
        end = tr[-1][1]
        ini.append({"born": p["born_myr"], "seen": True, "a0": a0, "end": end, "peak": max(x[1] for x in tr),
                    "front20": front20, "front_any": front_any, "fate": p.get("fate") or "alive",
                    "into_parent": p.get("into") is not None and p.get("into") == p.get("parent"),
                    "life": (p["died_myr"] if p.get("died_myr") is not None else 0.0) - p["born_myr"]})
    seen = [x for x in ini if x["seen"]]
    out["init_plates"] = {
        "n": len(ini), "seen": len(seen),
        "a0_p50": float(np.median([x["a0"] for x in seen])) if seen else None,
        "front_within20_share": float(np.mean([x["front20"] > 0 for x in seen])) if seen else None,
        "front_ever_share": float(np.mean([x["front_any"] > 0 for x in seen])) if seen else None,
        "left_at_end_p50": float(np.median([x["end"] / max(x["a0"], 1) for x in seen if x["fate"] != "alive"])) if seen else None,
        "absorbed_into_parent": sum(1 for x in seen if x["fate"] == "absorbed" and x["into_parent"]),
        "absorbed": sum(1 for x in seen if x["fate"] == "absorbed"),
        "lost_samples_per_myr": float(sum(max(x["a0"] - x["end"], 0) for x in seen if x["fate"] != "alive") / 900.0),
        "born_samples_per_myr": float(sum(x["a0"] for x in seen) / 900.0),
    }
    t0 = r["t0_myr"]
    births_si = [p["born_myr"] for p in lives if p["origin"] == "subduction_initiation"]
    out["init_per_100myr"] = 100.0 * len(births_si) / (0.0 - t0)
    out["init_late_per_100myr"] = 100.0 * sum(1 for b in births_si if b >= -500.0) / 500.0
    out["births_si"] = births_si
    out["summary"] = {k: summ.get(k) for k in ("initiations", "rifts", "sutures", "collisions", "healed", "births",
                                               "deaths", "plates_final", "resamplings", "passive_mean",
                                               "passive_floor_age_p50_mean", "active_mean", "subduction_mean",
                                               "ocean_age_p50_myr", "ocean_age_p90_myr", "ocean_younger_250")}
    return out


def map_part(run, r, births_si):
    N = r["samples"]
    spacing_km = math.sqrt(4.0 * math.pi / N) * R_KM
    margin_km = MARGIN_KM + spacing_km
    steps = r["steps"]
    step_t = np.array([s["t"] for s in steps])
    ages = r["ages"]
    age_t = np.array([a["t"] for a in ages])
    out = []
    for path in sorted((run / "maps").glob("t*.bin")):
        t = float(path.stem[1:])
        if t < -800.0:
            continue
        a = np.fromfile(path, "<f4").reshape(-1, FIELDS).astype(np.float64)
        n = len(a)
        if n >= N:
            continue  # taken at a resampling: nothing hidden, no front flagged
        pl_path = run / "maps" / "plate" / path.name
        if not pl_path.exists():
            continue
        plate = np.fromfile(pl_path, "<u4").astype(np.int64)
        if len(plate) != n:
            continue
        p = xyz(a[:, 0], a[:, 1])
        z, th = a[:, 2], a[:, 3] / 1000.0
        cont = a[:, 4] > 0.5
        flags = np.rint(a[:, 10]).astype(np.int64)
        oc = ~cont
        trench = (flags & TRENCH) != 0
        div = (flags & DIV) != 0
        fsub = (flags & FRONT_SUB) != 0
        bare = oc & ((flags & (TRENCH | ARC | HOT | SZ | FORE)) == 0) & (th < 10.0)
        age = inv_gdh1(z)
        # The record at this time.
        st = steps[int(np.argmin(np.abs(step_t - t)))]
        ar = ages[int(np.argmin(np.abs(age_t - t)))]
        rec_b = hist_bands(ar["hist"])
        snap = {"t": t, "visible": n, "rec_ocean": st["ocean"], "map_ocean": int(oc.sum()),
                "rec_old180": st["old180"], "rec_bands": rec_b,
                "proxy_old180_all": int((oc & (age > 180)).sum()),
                "proxy_old180_bare": int((bare & (age > 180)).sum()),
                "proxy_bands_all": counts(band(age[oc], AGE_EDGES), len(AGE_EDGES) - 1),
                "proxy_bands_bare": counts(band(age[bare], AGE_EDGES), len(AGE_EDGES) - 1),
                "n_bare": int(bare.sum()), "n_trench": int(trench.sum()), "n_front_sub": int(fsub.sum()),
                "n_div": int(div.sum()), "front_sub_km": float(fsub.sum() * spacing_km)}
        # Distances, km.
        dc = nearest_km(p, p[cont])
        dt = nearest_km(p, p[trench])
        dr = nearest_km(p, p[div])
        dco = np.full(n, np.inf)
        ids = np.unique(plate)
        pshare, pcont, ptrench = {}, {}, {}
        for pid in ids:
            m = plate == pid
            pshare[pid] = m.sum() / N
            pcont[pid] = (m & cont).sum() / max(m.sum(), 1)
            ptrench[pid] = bool((m & trench).any())
            mc = m & cont
            mo = m & oc
            if mc.any() and mo.any():
                dco[mo] = nearest_km(p[mo], p[mc])
        ps = np.array([pshare[x] for x in plate])
        pc = np.array([pcont[x] for x in plate])
        ptr = np.array([ptrench[x] for x in plate])
        # Q2's test per plate: old floor (> 150 Myr) clear of every trench by 1000 km, at least 16 such samples, seeds
        # beyond the plate's own continent by the margin.
        old150c = oc & (age > INIT_AGE) & (dt > CLEAR_KM)
        n150 = {pid: int((old150c & (plate == pid)).sum()) for pid in ids}
        far = {pid: int((old150c & (plate == pid) & (dco > margin_km)).sum()) for pid in ids}
        elig = {pid for pid in ids if n150[pid] >= 16}
        lam = {pid: RATE * n150[pid] / N / A0_SHARE for pid in elig}
        snap["plates"] = len(ids)
        snap["plates_eligible"] = len(elig)
        snap["plates_eligible_far"] = sum(1 for pid in elig if far[pid] > 0)
        snap["lambda_per_myr"] = float(sum(lam.values()))
        snap["lambda_far_per_myr"] = float(sum(v for pid, v in lam.items() if far[pid] > 0))
        snap["births_si_pm25"] = sum(1 for b in births_si if abs(b - t) <= 25.0) / 50.0
        # The old floor (> 180 Myr by the proxy, all oceanic-labelled samples) and every oceanic sample, by place.
        old = oc & (age > 180.0)
        status = np.full(n, -1)
        status[old & (dt <= CLEAR_KM)] = 0  # blocked by a trench within the clearance
        clear_old = old & (dt > CLEAR_KM)
        el = np.array([pid in elig for pid in plate])
        fa = np.array([far.get(pid, 0) > 0 for pid in plate])
        status[clear_old & ~el] = 1  # its plate has under 16 old samples clear of trenches
        status[clear_old & el & ~fa] = 2  # eligible, but no seed beyond the margin: never splits
        status[clear_old & el & fa & (dco <= margin_km)] = 3  # kept on the plate by the margin (PLT-503)
        status[clear_old & el & fa & (dco > margin_km)] = 4  # exposed: can be split off
        snap["status_old180"] = counts(status[old], 5)
        snap["status_names"] = ["trench<=1000", "plate<16", "no_seed", "kept_margin", "exposed"]
        for name, sel in (("old180", old), ("ocean", oc)):
            d = {}
            d["dc"] = counts(band(dc[sel], DC_EDGES), len(DC_EDGES) - 1)
            d["dco"] = counts(band(dco[sel], DC_EDGES), len(DC_EDGES) - 1)
            d["dt"] = counts(band(dt[sel], DT_EDGES), len(DT_EDGES) - 1)
            d["dr"] = counts(band(dr[sel], DR_EDGES), len(DR_EDGES) - 1)
            d["ps"] = counts(band(ps[sel], PS_EDGES), len(PS_EDGES) - 1)
            d["pc"] = counts(band(pc[sel], PC_EDGES), len(PC_EDGES) - 1)
            d["ptrench"] = [int((sel & ~ptr).sum()), int((sel & ptr).sum())]
            d["th"] = counts(band(th[sel], TH_EDGES), len(TH_EDGES) - 1)
            # Cross: distance to the continent by whether a trench lies within the clearance.
            d["dc_x_dt1000"] = [counts(band(dc[sel & (dt <= CLEAR_KM)], DC_EDGES), 4),
                                counts(band(dc[sel & (dt > CLEAR_KM)], DC_EDGES), 4)]
            snap[name] = d
        out.append(snap)
    return out


def main():
    args = sys.argv[1:]
    rec_only = "--rec-only" in args
    for arm in [a for a in args if not a.startswith("--")]:
        for run in sorted(p for p in Path(arm).iterdir() if (p / "rec.json").is_file()):
            r = json.loads((run / "rec.json").read_text())
            summ = summary_tokens((run / "summary.txt").read_text())
            rp = rec_part(r, summ)
            mp = map_part(run, r, rp["births_si"]) if (run / "maps").is_dir() and not rec_only else []
            print(json.dumps({"arm": str(Path(arm)), "run": run.name, "samples": r["samples"], "dt": r["dt_myr"],
                              "rec": rp, "maps": mp}), flush=True)


main()
