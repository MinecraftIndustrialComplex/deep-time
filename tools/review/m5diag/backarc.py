"""The back-arc and front-length diagnosis on a pool arm's cached runs (front lane, 2026-10-25).

    BAKEY=<pool key[,key]> [BAARM=<preset-length/argkey>] python3 -I backarc.py <out.jsonl> <workers> [run,run,...]

Per run, at the final frame (maps/t0000 is a resampling, so it has no front flags; the fronts are
the census grid's trench cells):
1. Kinematics at the fronts: each trench cell (fields' regime 2, the subducting side), its map
   sample (u) and the nearest map sample of another plate within 300 km (o, the overriding side);
   the trench normal n (tangent at o, from u into the upper plate), the upper plate's absolute
   trench-normal speed Vup = v_o . (-n) (Heuret & Lallemand 2005's sign: positive toward the
   trench), the subducting plate's Vsub = v_u . n and the convergence Vc = Vup + Vsub. Velocities
   from rec.json's prev_omega with the model_km scale fitted to steps[-1].plates' speed_mm_yr at
   the visible-sample centroid. SUB: the trench cell's crust is oceanic (else coll). The SUB front
   split by the upper plate's crust (oc: continental crust of o within 200 km of the front, else
   oo) and by Vup bands; its contacts (plate pairs) and their lengths.
2. The grid: each cell's plate and its nearest SUB front point on the same plate (d_km); the
   back-arc regime (4) by d and by its front's Vup band; the high ground (land above 2 km) by
   m4split's classes and the context: behind a SUB front within 800 km (by Vup band), else within
   800 km of a collision front point, else none.
3. The record: per step after -900 Myr, the boundary mix, the front bands (slab, normal speed,
   contact age, incipient, Benioff), the plate count.
"""
import contextlib
import io
import json
import os
import sys
from multiprocessing import Pool
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
from m5lib import cc  # noqa: E402
from m4split import CLASSES, classify  # noqa: E402

POOL = Path("/data/ban/deeptime-runs/pool")
KEY = os.environ["BAKEY"]
ARM = os.environ.get("BAARM", "earthlike_quick_16k-1000/cd665cef")
FIELDS = 11
REG_TR, REG_BA = 2, 4
VUP_EDGES = (-20.0, -5.0, 5.0, 20.0)
VUP_NAMES = ("ret_fast", "ret", "neutral", "adv", "adv_fast")
D_EDGES = (200.0, 440.0, 800.0)
D_NAMES = ("d0_200", "d200_440", "d440_800", "d800p")
CTX = VUP_NAMES + ("coll", "none")


def vband(v):
    return np.searchsorted(np.array(VUP_EDGES), v, side="right")


def dband(d):
    return np.searchsorted(np.array(D_EDGES), d, side="right")


def align(p_cells, kind_cells, a):
    """m5f.py's: the map's frame against the grid's, by the nearest sample's crust kind."""
    rng = np.random.default_rng(0)
    pick = rng.choice(p_cells.shape[0], 20000, replace=False)
    best = None
    for ls in (1, -1):
        for lsign in (1, -1):
            for off in (0.0, 180.0, 90.0, -90.0):
                q = cc.xyz(ls * a[:, 0], lsign * a[:, 1] + off)
                _, j = cKDTree(q).query(p_cells[pick])
                agree = float(((a[j, 4] > 0.5) == kind_cells[pick]).mean())
                if best is None or agree > best[0]:
                    best = (agree, ls, lsign, off)
    return best


def pct(x, qs=(10, 25, 50, 75, 90)):
    x = np.asarray(x)
    return {f"p{q}": float(np.percentile(x, q)) for q in qs} if x.size else None


def pair_fronts(G, q, qm, qt, pid, plate):
    """Each trench cell's plate pair: o, the local plate with the most arc and back-arc cells
    (regime 3, 4) within 300 km (else the nearest plate other than the cell's own), u the nearest
    other local plate; o_map o's nearest sample; n the tangent direction from u's local samples'
    centroid to o's (model frame, at o_map)."""
    tr = np.nonzero(G["reg"] == REG_TR)[0]
    arc_cells = np.nonzero(np.isin(G["reg"], (3, 4)))[0]
    loc = qt.query_ball_point(G["p"][tr], cc.chord(300.0))
    arcloc = cKDTree(G["p"][arc_cells]).query_ball_point(G["p"][tr], cc.chord(300.0))
    keep, os_, us, om_, ns, by_arc = [], [], [], [], [], []
    for t in range(tr.size):
        ids = np.asarray(loc[t], dtype=np.int64)
        if ids.size < 3:
            continue
        pl = pid[ids]
        ups = np.unique(pl)
        if ups.size < 2:
            continue
        ap = plate[arc_cells[np.asarray(arcloc[t], dtype=np.int64)]] if len(arcloc[t]) else np.zeros(0, np.int64)
        score = np.array([(ap == x).sum() for x in ups])
        dist = np.linalg.norm(q[ids] - G["p"][tr[t]], axis=1)
        if score.max() > 0:
            o = ups[np.argmax(score)]
            by_arc.append(True)
        else:
            own = plate[tr[t]]
            other = pl != own
            if not other.any():
                continue
            o = pl[other][np.argmin(dist[other])]
            by_arc.append(False)
        mo = pl == o
        mu = ~mo
        u = pl[mu][np.argmin(dist[mu])]
        mu = pl == u
        o_map = ids[mo][np.argmin(dist[mo])]
        co, cu = qm[ids[mo]].mean(0), qm[ids[mu]].mean(0)
        x = qm[o_map]
        dv = co - cu
        nv = dv - (dv @ x) * x
        nn_ = np.linalg.norm(nv)
        if nn_ < 1e-12:
            by_arc.pop()
            continue
        keep.append(t)
        os_.append(o)
        us.append(u)
        om_.append(o_map)
        ns.append(nv / nn_)
    keep = np.array(keep, dtype=np.int64)
    return {"tr": tr[keep], "o": np.array(os_), "u": np.array(us), "o_map": np.array(om_, dtype=np.int64),
            "n": np.array(ns), "by_arc": np.array(by_arc, bool),
            "own_is_u": plate[tr[keep]] == np.array(us), "paired": float(keep.size / max(tr.size, 1))}


def kinematics(a, pid, rec, P, sub):
    """The fronts' kinematics in the model frame (the map's own lat/lon)."""
    qm = cc.xyz(a[:, 0], a[:, 1])
    om = {int(i): np.array(w, float) for i, w in rec["prev_omega"]}
    ratios = []
    for row in rec["steps"][-1]["plates"]:
        i, spd = int(row[0]), row[3]
        m = pid == i
        if i not in om or m.sum() < 50 or spd <= 0.5:
            continue
        c = qm[m].sum(0)
        c /= np.linalg.norm(c)
        s = np.linalg.norm(np.cross(om[i], c))
        if s > 0:
            ratios.append(spd / s)
    K = float(np.median(ratios))
    spread = float(np.percentile(ratios, 90) / np.percentile(ratios, 10)) if len(ratios) > 3 else None
    W = np.zeros((pid.max() + 1, 3))
    for i, w in om.items():
        if i <= pid.max():
            W[i] = w
    o, u, o_map, nvec = P["o"], P["u"], P["o_map"], P["n"]
    qi = qm[o_map]
    vo = np.cross(W[o], qi) * K
    vu = np.cross(W[u], qi) * K
    vup = (vo * -nvec).sum(1)
    vsub = (vu * nvec).sum(1)
    rel = vu - vo
    vpar = np.linalg.norm(rel - (rel * nvec).sum(1)[:, None] * nvec, axis=1)
    # The upper plate's continental crust within 200 km (mix_km's ocean-continent rule, PLT-370).
    cont = a[:, 4] > 0.5
    nb = cKDTree(qm).query_ball_point(qi, cc.chord(200.0))
    oc = np.array([bool(np.any(cont[j] & (pid[j] == oo))) for j, oo in zip(nb, o)], dtype=bool)
    # The lithosphere's net rotation (t0000 is a resampling: equal-area samples, all visible):
    # w_net = 3/(8 pi) * integral of r x v dA, and Vup, Vsub in that no-net-rotation frame.
    wp = W[pid]
    w_net = 1.5 * (wp - (qm * wp).sum(1)[:, None] * qm).sum(0) / len(qm)
    vo_n = np.cross(W[o] - w_net, qi) * K
    vu_n = np.cross(W[u] - w_net, qi) * K
    return {"K": K, "K_spread_p90_p10": spread, "n_plates_fit": len(ratios), "sub": sub,
            "o": o, "u": u, "vup": vup, "vsub": vsub, "vc": vup + vsub, "vpar": vpar, "oc": oc,
            "o_map": o_map, "net_rot_mm_yr": float(np.linalg.norm(w_net) * K),
            "vup_nnr": (vo_n * -nvec).sum(1), "vsub_nnr": (vu_n * nvec).sum(1)}


def kin_summary(k, sub_km):
    m0 = k["sub"]
    out = {"K_model_km": k["K"] / 1.0e6, "K_spread_p90_p10": k["K_spread_p90_p10"],
           "n_plates_fit": k["n_plates_fit"], "trench_cells": int(m0.size), "sub_cells": int(m0.sum())}
    for nm, m in (("all", m0), ("oo", m0 & ~k["oc"]), ("oc", m0 & k["oc"]), ("coll", ~m0)):
        if not m.any():
            continue
        vb = vband(k["vup"][m])
        out[nm] = {"n": int(m.sum()), "share": float(m.sum() / max(m0.sum(), 1)),
                   "vup": pct(k["vup"][m]), "vsub": pct(k["vsub"][m]),
                   "vc": pct(k["vc"][m]), "vpar": pct(k["vpar"][m]),
                   "vup_bands": {VUP_NAMES[b]: float((vb == b).mean()) for b in range(len(VUP_NAMES))},
                   "vc_neg": float((k["vc"][m] < 0).mean()),
                   "vup_gt_vsub": float((k["vup"][m] > k["vsub"][m]).mean()),
                   "vup_nnr": pct(k["vup_nnr"][m]), "vsub_nnr": pct(k["vsub_nnr"][m]),
                   "vup_nnr_bands": {VUP_NAMES[b]: float((vband(k["vup_nnr"][m]) == b).mean())
                                     for b in range(len(VUP_NAMES))},
                   "vup_neg": float((k["vup"][m] < 0).mean()),
                   "vup_nnr_neg": float((k["vup_nnr"][m] < 0).mean())}
    out["net_rot_mm_yr"] = k["net_rot_mm_yr"]
    # The SUB front's contacts: plate pairs and their lengths (cell shares of the record's SUB km).
    pairs, cnt = np.unique(np.stack([k["o"][m0], k["u"][m0]], 1), axis=0, return_counts=True)
    lens = cnt / cnt.sum() * sub_km
    out["contacts"] = int(pairs.shape[0])
    out["contacts_ge500km"] = int((lens >= 500).sum())
    out["contact_km"] = pct(lens, (25, 50, 75, 90))
    out["contact_km_max"] = float(lens.max())
    out["contact_km_top5_share"] = float(np.sort(lens)[::-1][:5].sum() / lens.sum())
    out["upper_plates"] = int(np.unique(k["o"][m0]).size)
    out["lower_plates"] = int(np.unique(k["u"][m0]).size)
    return out


def load_grid(run):
    with contextlib.redirect_stdout(io.StringIO()):
        f, g, e = cc.load_dump(run / "fields")
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat2 = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon2 = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    G = {"p": cc.xyz(lat2[ok], lon2[ok]), "ncell": int(ok.sum())}
    G["cell_km2"] = cc.EARTH_KM2 / G["ncell"]
    G["z"] = (f["elevation"][ok] - cc.SEA_Y) * cc.M_PER_BLOCK_V
    G["T"] = np.nan_to_num(f["plates/thickness_m"][ok])
    G["cont"] = np.nan_to_num(f["plates/crust_type"][ok]) > 0.5
    oro = np.rint(np.nan_to_num(f["plates/orogeny_type"][ok])).astype(int)
    age = np.nan_to_num(f["plates/orogeny_age_myr"][ok], nan=1e9)
    G["reg"] = np.rint(np.nan_to_num(f["regime"][ok])).astype(int)
    G["land"] = G["z"] > 0
    G["hi"] = G["land"] & (G["z"] > 2000)
    G["L"] = max(int(G["land"].sum()), 1)
    G["cls"] = classify(oro, age, G["T"] / 1000.0, G["cont"], G["reg"])
    return G


def grid_part(G, q, plate, k, sub_km):
    p, ncell, cell_km2, T, cont, reg = (G[x] for x in ("p", "ncell", "cell_km2", "T", "cont", "reg"))
    land, hi, L, cls = G["land"], G["hi"], G["L"], G["cls"]
    out = {"cells": ncell}
    # Each cell's nearest SUB front point (the overriding side's sample) on its own plate.
    sub = k["sub"]
    fo, o = k["o_map"][sub], k["o"][sub]
    vup_s = k["vup"][sub]
    d = np.full(ncell, np.inf)
    jf = np.full(ncell, -1)
    for pl in np.unique(o):
        sel = np.nonzero(o == pl)[0]
        cells = np.nonzero(plate == pl)[0]
        if not cells.size:
            continue
        dc, jc = cKDTree(q[fo[sel]]).query(p[cells])
        d[cells] = cc.arc_km(dc)
        jf[cells] = sel[jc]
    behind = d <= 800.0
    vb_cell = np.where(behind, vband(vup_s[np.maximum(jf, 0)]), -1)
    oc_cell = behind & k["oc"][sub][np.maximum(jf, 0)]  # behind an ocean-continent front (else ocean-ocean)
    fc = k["o_map"][~sub]
    dcoll = cc.arc_km(cKDTree(q[fc]).query(p)[0]) if fc.size else np.full(ncell, np.inf)
    ctx = np.where(behind, vb_cell, np.where(dcoll <= 800.0, len(VUP_NAMES), len(VUP_NAMES) + 1))
    # The back-arc regime.
    ba = reg == REG_BA
    nba = max(int(ba.sum()), 1)
    out["backarc"] = {
        "share_planet": float(ba.sum() / ncell), "oceanic": float((ba & ~cont).sum() / nba),
        "area_over_sub_km": float(ba.sum() * cell_km2 / max(sub_km, 1)),
        "own_plate_front_le800": float((ba & behind).sum() / nba),
        "d_km": pct(d[ba & behind]),
        "by_d": {D_NAMES[b]: float((ba & (dband(d) == b)).sum() / nba) for b in range(4)},
        "by_vup": {VUP_NAMES[b]: float((ba & (vb_cell == b)).sum() / nba) for b in range(5)},
        "land": float((ba & land).sum() / nba),
        "hi": float((ba & hi).sum() / nba),
        "cont_land": float((ba & cont & land).sum() / max(int((ba & cont).sum()), 1)),
        "oce_land": float((ba & ~cont & land).sum() / max(int((ba & ~cont).sum()), 1)),
    }
    # Behind the fronts (all cells within 800 km on the upper plate) by the front's Vup band.
    beh = {}
    for b in range(5):
        m = behind & (vb_cell == b)
        mc = m & cont
        beh[VUP_NAMES[b]] = {"cells_share_planet": float(m.sum() / ncell),
                             "cont": float(mc.sum() / max(m.sum(), 1)),
                             "land": float((m & land).sum() / max(m.sum(), 1)),
                             "hi": float((m & hi).sum() / max(m.sum(), 1)),
                             "T_cont_p50_km": float(np.median(T[mc]) / 1000.0) if mc.any() else None,
                             "hi_of_land_L": float((m & hi).sum() / L)}
    out["behind"] = beh
    # The high ground by class and context, as shares of the land (M4's).
    hg = {}
    for i, c in enumerate(CLASSES):
        m = hi & (cls == i)
        if not m.any():
            continue
        row = {"m4": float(m.sum() / L)}
        for j, cn in enumerate(CTX):
            row[cn] = float((m & (ctx == j)).sum() / L)
        for b in range(4):
            row[D_NAMES[b]] = float((m & (dband(d) == b)).sum() / L)
        row["dcoll_le800"] = float((m & (dcoll <= 800.0)).sum() / L)
        row["beh_oo"] = float((m & behind & ~oc_cell).sum() / L)
        row["beh_oc"] = float((m & oc_cell).sum() / L)
        hg[c] = row
    out["hi_by_class"] = hg
    out["M4"] = float(hi.sum() / L)
    out["land_ctx"] = {cn: float((land & (ctx == j)).sum() / L) for j, cn in enumerate(CTX)}
    out["land_beh_oo"] = float((land & behind & ~oc_cell).sum() / L)
    out["land_beh_oc"] = float((land & oc_cell).sum() / L)
    return out


def record_part(rec):
    st = [s for s in rec["steps"] if s["t"] >= -900.0]
    mix = np.array([s["mix_km"] for s in st])
    out = {"steps": len(st), "mix_km_mean": mix.mean(0).tolist(), "mix_km_last": rec["steps"][-1]["mix_km"]}
    fb = [s["front_bands"] for s in st]
    for key in ("slab", "normal", "age"):
        out[f"{key}_mean"] = np.array([b[key] for b in fb]).mean(0).tolist()
    for key in ("incipient", "benioff"):
        out[f"{key}_mean"] = np.array([b.get(key, [0, 0]) for b in fb]).mean(0).tolist()
    for key in ("oo_over_cont", "oo_under_cont", "oo_coast", "oo_pair"):
        out[f"{key}_mean"] = np.array([b.get(key, [0] * (3 if key == "oo_pair" else 6)) for b in fb]).mean(0).tolist()
    out["other_converging_mean"] = float(np.mean([b["other_converging"] for b in fb]))
    out["plates_mean"] = float(np.mean([len(s["plates"]) for s in st]))
    out["plates_last"] = len(rec["steps"][-1]["plates"])
    return out


def one(run):
    a = np.fromfile(run / "maps" / "t0000.bin", "<f4").reshape(-1, FIELDS).astype(np.float64)
    pid = np.fromfile(run / "maps" / "plate" / "t0000.bin", "<u4").astype(np.int64)
    rec = json.loads((run / "rec.json").read_text())
    G = load_grid(run)
    agree, ls, lsign, off = align(G["p"], G["cont"], a)
    q = cc.xyz(ls * a[:, 0], lsign * a[:, 1] + off)
    qt = cKDTree(q)
    _, nn = qt.query(G["p"])
    plate = pid[nn]
    # The trench cells (regime 2, the subducting side) and their plate pairs.
    P = pair_fronts(G, q, cc.xyz(a[:, 0], a[:, 1]), qt, pid, plate)
    sub = ~G["cont"][P["tr"]]
    k = kinematics(a, pid, rec, P, sub)
    mix = rec["steps"][-1]["mix_km"]
    sub_km = float(mix[0] + mix[1])
    out = {"run": run.name, "map_align": agree, "trench_paired": P["paired"], "sub_km_last": sub_km,
           "pair_by_arc": float(P["by_arc"].mean()), "own_is_u": float(P["own_is_u"].mean()),
           "kin": kin_summary(k, sub_km), "sea_m": rec["steps"][-1]["sea_m"]}
    conv = sub & (k["vc"] > 0)
    kc = dict(k, sub=conv)
    out["kin_conv"] = kin_summary(kc, sub_km)
    out["grid"] = grid_part(G, q, plate, k, sub_km)
    out["rec"] = record_part(rec)
    return out


def main():
    dest = Path(sys.argv[1])
    workers = int(sys.argv[2]) if len(sys.argv) > 2 else 4
    # BAKEY may list several keys (an alias's cached seeds): the first key's run of a name wins.
    seen = {}
    for key in KEY.split(","):
        for p in sorted((POOL / key / ARM).glob("*-*")):
            if (p / "maps" / "t0000.bin").exists() and p.name not in seen:
                seen[p.name] = p
    runs = sorted(seen.values(), key=lambda p: p.name)
    if len(sys.argv) > 3:
        runs = [p for p in runs if p.name in sys.argv[3].split(",")]
    print(f"{len(runs)} runs", flush=True)
    with Pool(workers) as pool, dest.open("w") as fh:
        for res in pool.imap_unordered(one, runs):
            fh.write(json.dumps(res) + "\n")
            fh.flush()
            kk = res["kin"]
            print(f"{res['run']}: K {kk['K_model_km']:.0f} spread {kk['K_spread_p90_p10']} sub cells {kk['sub_cells']}"
                  f" vup p50 {kk['all']['vup']['p50']:.1f} ba width {res['grid']['backarc']['area_over_sub_km']:.0f}",
                  flush=True)
    print("BA_DONE")


if __name__ == "__main__":
    main()
