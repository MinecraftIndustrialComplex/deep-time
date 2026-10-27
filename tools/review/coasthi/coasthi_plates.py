#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Which plate the coast's high ground stands on (coast-high lane, diagnosis only):
coasthi_plates.py <out.jsonl> <run folder> [...]

For each run: the census's coast and high ground (coasthi.py's reach with sources, on the final
fields' Fibonacci points); each point takes the plate id, the island-arc and continental-arc magma
and the thickness of the nearest crust-budget map sample at the end (maps/t0000.bin and
maps/plate/t0000.bin, the same time as the fields); the dynamics record's collision states open at
the end (partings with reason "open") name each collision's overriding and incoming plate. A
qualified coast point is then split by the plate of the high point that qualifies it: the
overriding plate of an open collision (the belt's side), its incoming plate (the wedge's side),
both, or neither; by that plate's size and continental area, its origin (rift, subduction
initiation, fission, initial) and age; by whether the high crust is island-arc crust (1 km or more
of island-arc magma, more than of continental-arc magma, crust_budget.rs `origin`); and by which
plate the sea in front of the coast belongs to (the high point's own plate, the collision partner,
or another). Needs numpy, scipy."""
import json
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import coasthi  # noqa: E402
import continents_census as cc  # noqa: E402

SAMPLE_KM2 = cc.EARTH_KM2 / 100_000


def frac(mask, sel):
    return float((mask & sel).sum() / max(sel.sum(), 1))


def run_one(run):
    run = Path(run)
    f, _g, e = cc.load_dump(run / "fields")
    h, w, r = e["h"], e["w"], e["r"]
    frow, fcol, fp = cc.fib_sample(h, w, r)
    _lat, ok_r = cc.lat_of_rows(h, r, w * r)
    sp = float(np.sqrt(cc.EARTH_KM2 / (int(ok_r.sum()) * w)))
    z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[frow, fcol].astype(float)
    reg = np.rint(np.nan_to_num(f["regime"][frow, fcol], nan=0.0)).astype(int)
    oro = np.rint(np.nan_to_num(f["plates/orogeny_type"][frow, fcol], nan=0.0)).astype(int)
    geo = cc.coast_geometry(fp, z, sp)
    G = coasthi.Graph(fp, geo["ground"], sp)
    reach, src = G.reach(geo["hi"], geo["coast"])
    q = geo["coast"] & (reach <= coasthi.REACH)
    coast, hi = geo["coast"], geo["hi"]
    m = np.fromfile(run / "maps" / "t0000.bin", "<f4").reshape(-1, cc.MAP_FIELDS).astype(np.float64)
    pid = np.fromfile(run / "maps" / "plate" / "t0000.bin", "<u4").astype(np.int64)
    mp = cc.xyz(m[:, 0], m[:, 1])
    _d, k = cKDTree(mp).query(fp)
    plate = pid[k]
    arc_oc, arc_co = m[k, 7], m[k, 8]
    island = (arc_oc >= 1000.0) & (arc_oc > arc_co)
    kind_m = m[:, 4] > 0.5
    rec = json.loads((run / "rec.json").read_text())
    t_end = rec["steps"][-1]["t"]
    open_ = [p for p in rec["partings"] if p["reason"] == "open" and p["end_myr"] >= t_end - 1e-6]
    over_ids = {p["over"] for p in open_}
    under_ids = {p["under"] for p in open_}
    partners = {}
    for p in open_:
        partners.setdefault(p["over"], set()).add(p["under"])
        partners.setdefault(p["under"], set()).add(p["over"])
    pinfo = {p["id"]: p for p in rec["plates"]}
    ids, counts = np.unique(pid, return_counts=True)
    size = dict(zip(ids.tolist(), counts.tolist()))
    cids, ccounts = np.unique(pid[kind_m], return_counts=True)
    csize = dict(zip(cids.tolist(), ccounts.tolist()))
    qi = np.nonzero(q)[0]
    j = src[qi]
    pj = plate[j]
    side = np.array([3 if (a in over_ids and a in under_ids) else 1 if a in over_ids else 2 if a in under_ids else 0
                     for a in pj.tolist()])
    # The sea in front of the coast: the plate of the nearest world-ocean point.
    ocean = geo["ocean"]
    oi = np.nonzero(ocean)[0]
    _d2, ko = cKDTree(fp[oi]).query(fp[qi])
    psea = plate[oi[ko]]
    sea_rel = np.array([0 if s == a else 1 if s in partners.get(a, ()) else 2 for s, a in zip(psea.tolist(), pj.tolist())])
    csz = np.array([csize.get(a, 0) for a in pj.tolist()]) * SAMPLE_KM2 / 1e6  # Mkm2 of continental crust
    origin = np.array([pinfo.get(a, {}).get("origin", "?") for a in pj.tolist()])
    age = np.array([t_end - (pinfo.get(a, {}).get("born_myr") if pinfo.get(a, {}).get("born_myr") is not None
                             else rec["t0_myr"]) for a in pj.tolist()])
    n_coast = max(int(coast.sum()), 1)
    out = {"run": str(run), "M5": float(q.sum() / n_coast), "open_collisions": len(open_)}
    names = ("none", "over", "under", "both")
    out["q_side"] = {nm: float((side == s).mean()) for s, nm in enumerate(names)}
    # The same split of all the high ground (M4's).
    hidx = np.nonzero(hi)[0]
    hside = np.array([3 if (a in over_ids and a in under_ids) else 1 if a in over_ids else 2 if a in under_ids else 0
                      for a in plate[hidx].tolist()])
    out["hi_side"] = {nm: float((hside == s).mean()) for s, nm in enumerate(names)}
    out["q_sea"] = {nm: float((sea_rel == s).mean()) for s, nm in enumerate(("own_plate", "partner", "other"))}
    for s, nm in enumerate(names):
        sel = side == s
        out[f"q_sea_{nm}"] = {n2: frac(sea_rel == s2, sel) for s2, n2 in enumerate(("own_plate", "partner", "other"))}
    bins = [0, 1, 3, 10, 30, 1e9]
    out["q_by_cont_Mkm2"] = coasthi.hist(csz, bins)
    out["cont_edges"] = "<1, 1-3, 3-10, 10-30, >=30 Mkm2 of continental crust on the high point's plate"
    hcsz = np.array([csize.get(a, 0) for a in plate[hidx].tolist()]) * SAMPLE_KM2 / 1e6
    out["hi_by_cont_Mkm2"] = coasthi.hist(hcsz, bins)
    for s, nm in ((1, "over"), (2, "under")):
        sel = side == s
        out[f"q_{nm}_by_cont_Mkm2"] = coasthi.hist(csz[sel], bins) if sel.any() else None
    ORIG = ("rift", "subduction_initiation", "fission", "initial", "back_arc", "margin_propagation")
    out["q_by_origin"] = {o: float((origin == o).mean()) for o in ORIG}
    hor = np.array([pinfo.get(a, {}).get("origin", "?") for a in plate[np.nonzero(hi)[0]].tolist()])
    out["hi_by_origin"] = {o: float((hor == o).mean()) for o in ORIG}
    out["q_by_plate_age"] = coasthi.hist(age, [0, 20, 50, 100, 300, 1e9])
    out["age_edges"] = "<20, 20-50, 50-100, 100-300, >=300 Myr"
    out["q_island_arc"] = float(island[j].mean()) if qi.size else None
    out["hi_island_arc"] = float(island[hi].mean()) if hi.any() else None
    out["q_island_arc_by_side"] = {nm: frac(island[j], side == s) for s, nm in enumerate(names)}
    out["q_arc_oc_med_m"] = float(np.median(arc_oc[j])) if qi.size else None
    out["q_himalayan"] = float((oro[j] == 2).mean()) if qi.size else None
    out["q_coll_regime"] = float((reg[j] == 5).mean()) if qi.size else None
    # The fronts of the open collisions on the overriding side: an overriding plate's map samples with
    # a sample of its incoming plate within 1.6 lattice spacings. Each high point on an overriding plate
    # takes its distance to its plate's fronts, and each qualified coast point the same distance to the
    # fronts of its high point's plate: a coast farther from the front than its high point by half the
    # reach stands behind the belt (the overriding plate's far side), else along it (the front's ends).
    mtree = cKDTree(mp)
    lat_sp = float(np.sqrt(cc.EARTH_KM2 / mp.shape[0]))
    front_pts = {}
    for o in open_:
        a, b = o["over"], o["under"]
        oa = np.nonzero(pid == a)[0]
        if oa.size == 0:
            continue
        nb = mtree.query_ball_point(mp[oa], cc.chord(1.6 * lat_sp))
        isf = np.fromiter((bool(np.any(pid[np.asarray(x, np.int64)] == b)) for x in nb), bool, oa.size)
        if isf.any():
            front_pts.setdefault(a, []).append(oa[isf])
    ftree = {a: cKDTree(mp[np.concatenate(v)]) for a, v in front_pts.items()}

    def d_front(points, plates):
        d = np.full(points.shape[0], np.nan)
        for a, t in ftree.items():
            sel = plates == a
            if sel.any():
                d[sel] = cc.arc_km(t.query(points[sel])[0])
        return d

    oc_tree = cKDTree(mp[~kind_m])
    d_oc = lambda pts: cc.arc_km(oc_tree.query(pts)[0])  # noqa: E731
    on_over = (side == 1) | (side == 3)
    dfj = d_front(fp[j], pj)
    dfc = d_front(fp[qi], pj)
    has_f = on_over & np.isfinite(dfj)
    behind = has_f & (dfc > dfj + 0.5 * reach[qi])
    out["q_over_with_front"] = float(has_f.mean()) if qi.size else None
    out["q_behind_belt"] = float(behind.mean()) if qi.size else None
    out["q_along_belt"] = float((has_f & ~behind).mean()) if qi.size else None
    out["q_over_dfront_src_p50"] = float(np.median(dfj[has_f])) if has_f.any() else None
    out["q_over_dfront_coast_p50"] = float(np.median(dfc[has_f])) if has_f.any() else None
    hover = np.isin(plate[hidx], list(ftree.keys()))
    dfh = d_front(fp[hidx], plate[hidx])
    okh = hover & np.isfinite(dfh)
    out["hi_over_dfront_p50"] = float(np.median(dfh[okh])) if okh.any() else None
    out["hi_over_dfront_p90"] = float(np.percentile(dfh[okh], 90)) if okh.any() else None
    out["hi_over_dfront_hist"] = coasthi.hist(dfh[okh], [0, 100, 200, 400, 800, 1e9]) if okh.any() else None
    out["dfront_edges"] = "<100, 100-200, 200-400, 400-800, >=800 km from the open collision's front"
    # Within the belt's zone (collision_belt_km, PLT-241: 400 km) of an open front or beyond it, and
    # whether the high ground's last stamp is a young (under 10 Myr) collision orogeny: beyond the
    # zone that is the overriding plate's interior shortening (PLT-652) or another front's.
    age = f["plates/orogeny_age_myr"][frow, fcol].astype(float)
    young_him = (oro == 2) & (np.nan_to_num(age, nan=1e9) < 10.0)
    belt_km = 400.0
    out["q_over_in_belt"] = float((has_f & (dfj <= belt_km)).mean()) if qi.size else None
    out["q_over_beyond_belt"] = float((has_f & (dfj > belt_km)).mean()) if qi.size else None
    out["q_over_beyond_belt_young_him"] = float((has_f & (dfj > belt_km) & young_him[j]).mean()) if qi.size else None
    out["q_over_in_belt_young_him"] = float((has_f & (dfj <= belt_km) & young_him[j]).mean()) if qi.size else None
    out["q_young_him"] = float(young_him[j].mean()) if qi.size else None
    hy = young_him[hidx]
    out["hi_over_in_belt"] = float((okh & (dfh <= belt_km)).mean()) if hidx.size else None
    out["hi_over_beyond_belt"] = float((okh & (dfh > belt_km)).mean()) if hidx.size else None
    out["hi_over_beyond_belt_young_him"] = float((okh & (dfh > belt_km) & hy).mean()) if hidx.size else None
    out["hi_young_him"] = float(hy.mean()) if hidx.size else None
    # The qualified coast against any open front (either side) and the sea in front of it: the crust
    # (map kind) and depth of its nearest world-ocean point, and 100 and 200 km farther out.
    under_pts = []
    for o in open_:
        ib = np.nonzero(pid == o["under"])[0]
        if ib.size:
            nb = mtree.query_ball_point(mp[ib], cc.chord(1.6 * lat_sp))
            isf = np.fromiter((bool(np.any(pid[np.asarray(x, np.int64)] == o["over"])) for x in nb), bool, ib.size)
            under_pts.append(ib[isf])
    allf = [v for vs in front_pts.values() for v in vs] + under_pts
    ci_all = np.nonzero(coast)[0]
    if allf and qi.size:
        fp_all = np.concatenate(allf)
        ft = cKDTree(mp[fp_all])
        dq = cc.arc_km(ft.query(fp[qi])[0])
        dcall = cc.arc_km(ft.query(fp[ci_all])[0])
        out["q_coast_dfront_hist"] = coasthi.hist(dq, [0, 100, 200, 400, 800, 1e9])
        out["coast_dfront_hist"] = coasthi.hist(dcall, [0, 100, 200, 400, 800, 1e9])
        out["coast_dfront_edges"] = "<100, 100-200, 200-400, 400-800, >=800 km from any open collision front"
        out["rate_coast_near_front"] = float((dq <= 200).sum() / max((dcall <= 200).sum(), 1))
        out["rate_coast_far_front"] = float((dq > 200).sum() / max((dcall > 200).sum(), 1))
    out["q_faces_continental"] = float(kind_m[k[oi[ko]]].mean()) if qi.size else None
    out["q_faces_depth_p50"] = float(np.median(z[oi[ko]])) if qi.size else None
    _d3, ko_all = cKDTree(fp[oi]).query(fp[ci_all])
    out["coast_faces_continental"] = float(kind_m[k[oi[ko_all]]].mean()) if ci_all.size else None
    ftree_all = cKDTree(fp)

    def offshore(cidx, oidx, km):
        a, b = fp[cidx], fp[oidx]
        t = b - a * (a * b).sum(1)[:, None]
        t /= np.maximum(np.linalg.norm(t, axis=1), 1e-12)[:, None]
        ang = km / cc.EARTH_R_KM
        nn = ftree_all.query(a * np.cos(ang) + t * np.sin(ang))[1]
        return z[nn], kind_m[k[nn]]

    if qi.size:
        for km in (100.0, 200.0):
            zq, kq = offshore(qi, oi[ko], km)
            za, ka = offshore(ci_all, oi[ko_all], km)
            out[f"q_depth_{km:.0f}km_p50"] = float(np.median(zq))
            out[f"coast_depth_{km:.0f}km_p50"] = float(np.median(za))
            out[f"q_cont_{km:.0f}km"] = float(kq.mean())
            out[f"coast_cont_{km:.0f}km"] = float(ka.mean())
            if allf:
                nr = dq <= 200.0
                out[f"q_nearfront_depth_{km:.0f}km_p50"] = float(np.median(zq[nr])) if nr.any() else None
                out[f"q_nearfront_cont_{km:.0f}km"] = float(kq[nr].mean()) if nr.any() else None
                out[f"q_farfront_depth_{km:.0f}km_p50"] = float(np.median(zq[~nr])) if (~nr).any() else None
                out[f"q_farfront_cont_{km:.0f}km"] = float(kq[~nr].mean()) if (~nr).any() else None
    # The high ground by its distance to any plate boundary (map samples with another plate's sample
    # within 1.6 lattice spacings) and its regime: collision-regime ground more than 400 km from every
    # boundary is the overriding plates' interior shortening (PLT-652, stamped as the belt is) or a
    # stale stamp; within 400 km it is a front's belt or wedge.
    nb_all = mtree.query_ball_point(mp, cc.chord(1.6 * lat_sp))
    is_b = np.fromiter((bool(np.any(pid[np.asarray(x, np.int64)] != pid[i])) for i, x in enumerate(nb_all)), bool,
                       mp.shape[0])
    btree = cKDTree(mp[is_b])
    db_j = cc.arc_km(btree.query(fp[j])[0]) if qi.size else np.zeros(0)
    db_h = cc.arc_km(btree.query(fp[hidx])[0]) if hidx.size else np.zeros(0)
    for nm, dd, rr, yy in (("q", db_j, reg[j], young_him[j]), ("hi", db_h, reg[hidx], young_him[hidx])):
        if dd.size == 0:
            continue
        coll = rr == 5
        out[f"{nm}_coll_far_from_boundary"] = float((coll & (dd > 400)).mean())
        out[f"{nm}_coll_near_boundary"] = float((coll & (dd <= 400)).mean())
        out[f"{nm}_young_him_far_from_boundary"] = float((yy & (dd > 400)).mean())
        out[f"{nm}_dboundary_hist"] = coasthi.hist(dd, [0, 100, 200, 400, 800, 1e9])
    # A partition of the qualified coast by its high ground: collision regime within 400 km of a plate
    # boundary (a front's belt or wedge), collision regime beyond (interior), Andean, Himalayan out of
    # the collision regime (an old belt), no orogeny. For the first, which side of the belt the coast
    # stands on: nearer the boundary than its high point (the front's side) or farther (behind it).
    if qi.size:
        db_c = cc.arc_km(btree.query(fp[qi])[0])
        rj, oj = reg[j], oro[j]
        part = np.where(rj == 5, np.where(db_j <= 400, 0, 1), np.where(oj == 1, 2, np.where(oj == 2, 3, 4)))
        for kk, nm in enumerate(("coll_belt", "coll_interior", "andean", "him_old_belt", "none")):
            out[f"part_{nm}"] = float((part == kk).mean())
        cb = part == 0
        out["coll_belt_front_side"] = float((cb & (db_c < db_j)).mean())
        out["coll_belt_back_side"] = float((cb & (db_c >= db_j)).mean())
        out["coll_belt_coast_dboundary_p50"] = float(np.median(db_c[cb])) if cb.any() else None
        out["coll_belt_src_dboundary_p50"] = float(np.median(db_j[cb])) if cb.any() else None
        out["coll_belt_src_island_arc"] = float(island[j][cb].mean()) if cb.any() else None
        out["coll_belt_self"] = float(hi[qi][cb].mean()) if cb.any() else None
        for o in ORIG:
            out[f"coll_belt_on_{o}"] = float((cb & (origin == o)).mean())
    # The incoming plates' foreland: their samples within 300 km of their open fronts (map z is the
    # plate model's elevation over the datum, so less the run's last sea level), continental share,
    # elevation and thickness; and the same of the overriding side's first 300 km.
    sea_m = rec["steps"][-1].get("sea_m") or 0.0
    fl_z, fl_c, fl_t, ov_z = [], [], [], []
    for o, ib in zip(open_, under_pts):
        if ib.size == 0:
            continue
        sel = np.nonzero(pid == o["under"])[0]
        d = cc.arc_km(cKDTree(mp[ib]).query(mp[sel])[0])
        s3 = sel[d <= 300.0]
        fl_z.append(m[s3, 2] - sea_m)
        fl_c.append(kind_m[s3])
        fl_t.append(m[s3, 3] / 1000.0)
        so = np.nonzero(pid == o["over"])[0]
        if o["over"] in front_pts:
            do = cc.arc_km(cKDTree(mp[np.concatenate(front_pts[o["over"]])]).query(mp[so])[0])
            ov_z.append(m[so[do <= 300.0], 2] - sea_m)
    if fl_z:
        fz, fc, ft_ = np.concatenate(fl_z), np.concatenate(fl_c), np.concatenate(fl_t)
        out["foreland_cont"] = float(fc.mean())
        out["foreland_land"] = float((fz > 0).mean())
        out["foreland_cont_land"] = float((fz[fc] > 0).mean()) if fc.any() else None
        out["foreland_cont_z_p50"] = float(np.median(fz[fc])) if fc.any() else None
        out["foreland_cont_thick_p50"] = float(np.median(ft_[fc])) if fc.any() else None
    if ov_z:
        oz = np.concatenate(ov_z)
        out["overside_land"] = float((oz > 0).mean())
        out["overside_z_p50"] = float(np.median(oz))
    out["q_src_d_oceanic_p50"] = float(np.median(d_oc(fp[j]))) if qi.size else None
    out["hi_d_oceanic_p50"] = float(np.median(d_oc(fp[hidx]))) if hidx.size else None
    out["hi_d_oceanic_hist"] = coasthi.hist(d_oc(fp[hidx]), [0, 100, 200, 400, 800, 1e9]) if hidx.size else None
    out["d_oceanic_edges"] = "<100, 100-200, 200-400, 400-800, >=800 km to the nearest oceanic crust"
    # Counterfactuals on the final frame (what each part of the high ground buys, not a model):
    # M5 with the high ground of a class lowered to just under 2 km, and with the incoming plates'
    # drowned continental crust within 300 km of their open fronts raised to 1 m (a foreland filled
    # to the sea, as Earth's forelands are).
    def m5_of(zz):
        g2 = cc.coast_geometry(fp, zz, sp)
        return float(g2["has"].sum() / max(g2["coast"].sum(), 1))

    db_all = cc.arc_km(btree.query(fp)[0])
    for nm, mask in (("no_coll_belt", (reg == 5) & (db_all <= 400)), ("no_coll_interior", (reg == 5) & (db_all > 400)),
                     ("no_coll", reg == 5), ("no_young_him", young_him), ("no_andean", oro == 1),
                     ("no_coll_on_sliver_plates", (reg == 5) & np.isin(
                         np.array([pinfo.get(a, {}).get("origin", "?") for a in plate.tolist()]),
                         ("back_arc", "subduction_initiation"))),
                     ("no_island_arc_hi", island)):
        zz = z.copy()
        zz[mask & (zz > 2000)] = 1999.0
        out[f"cf_{nm}"] = m5_of(zz)
    if under_pts:
        fl = np.zeros(z.size, bool)
        for o, ib in zip(open_, under_pts):
            if ib.size == 0:
                continue
            dfl = cc.arc_km(cKDTree(mp[ib]).query(fp)[0])
            fl |= (plate == o["under"]) & (dfl <= 300.0) & kind_m[k] & (z <= 0)
        zz = z.copy()
        zz[fl] = 1.0
        out["cf_foreland_filled"] = m5_of(zz)
        out["cf_foreland_pts"] = int(fl.sum())
    # Open collisions: the overriding plate's continental area against the incoming plate's.
    out["open"] = [{"over": p["over"], "under": p["under"], "front_km": p["front_km"],
                    "over_cont_Mkm2": csize.get(p["over"], 0) * SAMPLE_KM2 / 1e6,
                    "under_cont_Mkm2": csize.get(p["under"], 0) * SAMPLE_KM2 / 1e6,
                    "over_origin": pinfo.get(p["over"], {}).get("origin"),
                    "over_age": t_end - (pinfo.get(p["over"], {}).get("born_myr") or rec["t0_myr"]),
                    "q_pts_over": int(((side == 1) | (side == 3))[pj == p["over"]].sum()) if qi.size else 0,
                    "start_myr": p["start_myr"], "peak_height_m": p["peak_height_m"]} for p in open_]
    return out


def main():
    with open(sys.argv[1], "a") as fh:
        for run in sys.argv[2:]:
            o = run_one(run)
            fh.write(json.dumps(o) + "\n")
            fh.flush()
            print(f"{Path(run).name}: M5 {o['M5']:.3f} side {o['q_side']} sea {o['q_sea']} island {o['q_island_arc']}",
                  flush=True)
    print("COASTHI_PLATES_DONE")


if __name__ == "__main__":
    main()
