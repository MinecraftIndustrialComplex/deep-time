#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The coast with high ground (V1's M5, v1census version 2) taken apart, on a pool arm's cached runs
and on Earth, with the census's own geometry (tools/review/continents_census.py coast_geometry: the
world ocean, the reach over land, Fibonacci points of the grid's count). Diagnosis only (coast-high
lane, 2026-10-26).

    coasthi.py model <out.jsonl> <run folder> [<run folder> ...]
    coasthi.py earth <out.json> <earth_relief_15m_p.grd> [PB2002_steps.dat]

What it adds to the census: for every coast point that counts, the high point that qualifies it
(the nearest by the census's reach: the straight run over ground, else the path over ground), so
the qualified coast can be split by what that high ground is and where it stands; the reach by
class of high ground (M5 with one class's high ground removed); the coast by its setting (a trench,
arc or back-arc within 200 km; else a collision belt; else neither; Earth: PB2002 steps SUB, then
CCB or OCB); the coast by the size of its landmass; the high ground in connected pieces (joined
within 1.6 spacings) by size and by how far inland each reaches; and the factors
M5 = s * f_near * M4 / kappa (kappa coast over land, f_near the high ground within 200 km of the
ocean, s qualified coast points per such high point).

Earth is present-day SRTM15+ at 15' (Tozer et al. 2019) with the Antarctica and Greenland box kept
out of the shares (the realism checks' by-rock stand-in, RLC-055). Needs numpy, scipy (and h5py for
Earth).
"""
import json
import sys
from pathlib import Path

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import dijkstra
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import continents_census as cc  # noqa: E402

REACH = 200.0
LEVEL = 2000.0
# Classes of the high ground in the model: orogeny type and the time since its last stamp.
HCLS = ("him_young", "him_old", "and_young", "and_old", "none")
YOUNG_MYR = 10.0
# Regime codes (plates/state.rs regime): the coast's setting and the high point's own regime.
SUBD = (2, 3, 4)
COLL = (5,)
REG_NAMES = ("ocean_basin", "ridge", "trench", "arc", "back_arc", "collision", "rift", "hotspot", "craton",
             "passive_margin")


class Graph:
    """The census's over-land graph for one planet: ground points joined within 1.6 spacings."""

    def __init__(self, p, ground, sp):
        self.p, self.ground, self.sp = p, ground, sp
        self.gi = np.nonzero(ground)[0]
        pg = p[self.gi]
        pairs = cKDTree(pg).query_pairs(cc.chord(1.6 * sp), output_type="ndarray")
        wt = cc.arc_km(np.linalg.norm(pg[pairs[:, 0]] - pg[pairs[:, 1]], axis=1))
        self.g = coo_matrix((wt, (pairs[:, 0], pairs[:, 1])), shape=(self.gi.size, self.gi.size)).tocsr()
        self.tree = cKDTree(p)
        # The landmass (ground component) of each point, for the coast's landmass size.
        from scipy.sparse.csgraph import connected_components
        _, lab = connected_components(self.g, directed=False)
        self.comp = np.full(p.shape[0], -1)
        self.comp[self.gi] = lab

    def reach(self, src_mask, coast):
        """coast_geometry's reach from the high points in src_mask, with the qualifying point."""
        n = self.p.shape[0]
        reach = np.full(n, np.inf)
        src = np.full(n, -1, np.int64)
        gl = np.nonzero(src_mask[self.gi])[0]
        if gl.size == 0:
            return reach, src
        d, _pred, s = dijkstra(self.g, directed=False, indices=gl, limit=REACH, min_only=True,
                               return_predecessors=True)
        reach[self.gi] = d
        ok = s >= 0
        src[self.gi[ok]] = self.gi[s[ok]]
        ci, hidx = np.nonzero(coast)[0], np.nonzero(src_mask)[0]
        near = cKDTree(self.p[hidx]).query_ball_point(self.p[ci], cc.chord(REACH))
        counts = np.fromiter((len(q) for q in near), np.int64, ci.size)
        if counts.sum():
            pa_i = np.repeat(ci, counts)
            pb_i = hidx[np.concatenate([np.asarray(q, np.int64) for q in near if len(q)])]
            for lo in range(0, pa_i.size, 400_000):
                a, b = pa_i[lo:lo + 400_000], pb_i[lo:lo + 400_000]
                pa, pb = self.p[a], self.p[b]
                clear = np.ones(a.size, bool)
                for t in np.linspace(0.1, 0.9, 9):
                    m = pa * (1 - t) + pb * t
                    m /= np.linalg.norm(m, axis=1)[:, None]
                    clear &= self.ground[self.tree.query(m)[1]]
                dd = cc.arc_km(np.linalg.norm(pa - pb, axis=1))
                k = clear & (dd <= REACH)
                a, b, dd = a[k], b[k], dd[k]
                if a.size == 0:
                    continue
                o = np.lexsort((dd, a))
                a, b, dd = a[o], b[o], dd[o]
                first = np.r_[True, a[1:] != a[:-1]]
                a, b, dd = a[first], b[first], dd[first]
                better = dd < reach[a]
                reach[a[better]] = dd[better]
                src[a[better]] = b[better]
        return reach, src


def hist(x, edges, w=None):
    h, _ = np.histogram(x, bins=edges, weights=w)
    return (h / max(h.sum(), 1e-12)).round(4).tolist()


def analyse(p, z, sp, keep, hcls=None, ncls=0, reg=None, sub_d=None, coll_d=None, extra=None):
    """Everything for one planet. keep: the points that count in the shares (Earth: not the sheets).
    hcls: class index of each point's high ground (model); reg: regime code per point (model);
    sub_d, coll_d: km to the nearest subduction / collision boundary (Earth, PB2002)."""
    geo = cc.coast_geometry(p, z, sp)
    ground, coast, hi, d_sea, has = geo["ground"], geo["coast"], geo["hi"], geo["d_sea"], geo["has"]
    land = z > 0
    G = Graph(p, ground, sp)
    reach, src = G.reach(hi, coast)
    q = coast & (reach <= REACH)
    assert np.array_equal(q, has), "the reach with sources differs from the census's"
    C = coast & keep
    Q = q & keep
    L = land & keep
    H = hi & keep
    n_c, n_q = max(int(C.sum()), 1), max(int(Q.sum()), 1)
    near = d_sea <= REACH
    out = {"M5": Q.sum() / n_c, "M4": H.sum() / max(L.sum(), 1), "kappa": C.sum() / max(L.sum(), 1),
           "f_near": (H & near).sum() / max(H.sum(), 1), "s": Q.sum() / max((H & near).sum(), 1),
           "coast_pts": int(C.sum()), "q_pts": int(Q.sum()), "hi_pts": int(H.sum()), "land_pts": int(L.sum())}
    out["ratio"] = out["M5"] / max(out["M4"], 1e-9)
    # The land's hypsometry: shares of the land by height over the sea, and its median.
    out["land_z_hist"] = hist(z[L], [0, 200, 500, 1000, 2000, 1e9])
    out["land_z_edges"] = "0-200, 200-500, 500-1000, 1000-2000, >2000 m"
    out["land_z_p50"] = float(np.median(z[L])) if L.any() else None
    # The land within 200 km of the sea by height: how high the coastal belt of land stands.
    ln = L & near
    out["near_land_z_hist"] = hist(z[ln], [0, 200, 500, 1000, 2000, 1e9])
    # The sea in front: the depth 100 km out from each coast point, along the great circle through its
    # nearest world-ocean point (a shelf reads shallow, a margin without one deep).
    oi = np.nonzero(geo["ocean"])[0]
    if oi.size:
        otree = cKDTree(p[oi])
        for nm, sel in (("q", Q), ("coast", C)):
            idx = np.nonzero(sel)[0]
            if idx.size == 0:
                continue
            b = p[oi[otree.query(p[idx])[1]]]
            a = p[idx]
            t = b - a * (a * b).sum(1)[:, None]
            t /= np.maximum(np.linalg.norm(t, axis=1), 1e-12)[:, None]
            ang = 100.0 / cc.EARTH_R_KM
            zz = z[G.tree.query(a * np.cos(ang) + t * np.sin(ang))[1]]
            out[f"{nm}_depth100_p50"] = float(np.median(zz))
            out[f"{nm}_depth100_deeper1km"] = float((zz < -1000).mean())
    qi = np.nonzero(Q)[0]
    j = src[qi]
    # The reach of the qualified coast and the qualifying point's own distance to the sea.
    out["reach_hist"] = hist(np.where(hi[qi], -1.0, reach[qi]), [-2, 0, 50, 100, 150, 200.1])
    out["reach_hist_edges"] = "self, 0-50, 50-100, 100-150, 150-200 km"
    out["src_dsea_hist"] = hist(d_sea[j], [-1, 50, 100, 200, 400, 1e9])
    out["src_dsea_edges"] = "<=50, 50-100, 100-200, 200-400, >400 km"
    out["reach_med"] = float(np.median(reach[qi])) if qi.size else None
    out["src_dsea_med"] = float(np.median(d_sea[j])) if qi.size else None
    out["src_z_med"] = float(np.median(z[j])) if qi.size else None
    # The high ground in connected pieces: size, how far inland it reaches, and the coast it serves.
    comp_h = cc.components(p, hi, sp)
    pieces = []
    if hi.any():
        lab = comp_h[hi]
        hidx = np.nonzero(hi)[0]
        nlab = lab.max() + 1
        size = np.bincount(lab, minlength=nlab)
        dmax = np.zeros(nlab)
        np.maximum.at(dmax, lab, d_sea[hidx])
        dmin = np.full(nlab, np.inf)
        np.minimum.at(dmin, lab, d_sea[hidx])
        served = np.bincount(comp_h[j], minlength=nlab) if qi.size else np.zeros(nlab)
        keep_h = np.bincount(lab, weights=keep[hidx].astype(float), minlength=nlab)
        km2 = cc.EARTH_KM2 / p.shape[0]
        area = size * km2 / 1e6  # Mkm2
        for lo_, hi_, nm in ((0, 0.02, "lt0.02"), (0.02, 0.1, "0.02_0.1"), (0.1, 0.5, "0.1_0.5"),
                             (0.5, 1e9, "ge0.5")):
            m = (area >= lo_) & (area < hi_)
            pieces.append({"bin_Mkm2": nm, "hi_share": float(keep_h[m].sum() / max(keep_h.sum(), 1)),
                           "q_share": float(served[m].sum() / max(served.sum(), 1)),
                           "n": int(m.sum()),
                           "inland_share": float(keep_h[m & (dmin > REACH)].sum() / max(keep_h.sum(), 1))})
        # Pieces by how far their farthest point stands from the sea.
        for lo_, hi_, nm in ((0, 100, "reach_lt100"), (100, 200, "reach_100_200"), (200, 400, "reach_200_400"),
                             (400, 1e9, "reach_ge400")):
            m = (dmax >= lo_) & (dmax < hi_)
            pieces.append({"bin_dmax_km": nm, "hi_share": float(keep_h[m].sum() / max(keep_h.sum(), 1)),
                           "q_share": float(served[m].sum() / max(served.sum(), 1)), "n": int(m.sum())})
        out["hi_pieces_n"] = int(nlab)
        out["hi_piece_area_med_Mkm2"] = float(np.median(area))
        # The largest piece's share of the high ground and of the qualified coast.
        k = int(np.argmax(keep_h))
        out["hi_largest_share"] = float(keep_h[k] / max(keep_h.sum(), 1))
        out["q_by_largest"] = float(served[k] / max(served.sum(), 1))
    out["pieces"] = pieces
    # The coast by the size of its landmass.
    lm = G.comp
    lm_sz = np.bincount(lm[ground & (lm >= 0)], minlength=lm.max() + 1) * cc.EARTH_KM2 / p.shape[0] / 1e6
    csz = lm_sz[lm[np.nonzero(C)[0]]]
    qsz = lm_sz[lm[qi]]
    bins = [0, 0.1, 1.0, 10.0, 1e9]
    out["coast_by_landmass"] = hist(csz, bins)
    out["q_by_landmass"] = hist(qsz, bins)
    out["landmass_edges"] = "<0.1, 0.1-1, 1-10, >=10 Mkm2"
    out["rate_by_landmass"] = [float(((qsz >= a) & (qsz < b)).sum() / max(((csz >= a) & (csz < b)).sum(), 1))
                               for a, b in zip(bins[:-1], bins[1:])]
    # The coast by setting.
    ci = np.nonzero(C)[0]
    if reg is not None:
        sub_m = np.isin(reg, SUBD)
        coll_m = np.isin(reg, COLL)
        dsub = np.full(p.shape[0], np.inf)
        dcol = np.full(p.shape[0], np.inf)
        if sub_m.any():
            dsub[ci] = cc.arc_km(cKDTree(p[sub_m]).query(p[ci])[0])
        if coll_m.any():
            dcol[ci] = cc.arc_km(cKDTree(p[coll_m]).query(p[ci])[0])
    else:
        dsub, dcol = sub_d, coll_d
    if dsub is not None:
        sett = np.where(dsub[ci] <= REACH, 0, np.where(dcol[ci] <= REACH, 1, 2))
        qc = q[ci]
        out["setting"] = {nm: {"coast": float((sett == k).mean()), "rate": float(qc[sett == k].mean()) if (sett == k).any()
                               else None, "part": float((qc & (sett == k)).sum() / n_c)}
                          for k, nm in enumerate(("subduction", "collision", "neither"))}
    # Model only: what the high ground is.
    if hcls is not None:
        jc = hcls[j]
        out["q_by_class"] = {nm: float((jc == k).mean()) for k, nm in enumerate(HCLS)}
        out["hi_by_class"] = {nm: float((H & (hcls == k)).sum() / max(H.sum(), 1)) for k, nm in enumerate(HCLS)}
        # M5 with one class's high ground removed (by the same reach), and with only it.
        rk = []
        for k in range(ncls):
            r_k, _ = G.reach(hi & (hcls == k), coast)
            rk.append(r_k)
        rk = np.array(rk)
        out["m5_without"] = {}
        out["m5_only"] = {}
        for k, nm in enumerate(HCLS):
            other = np.delete(rk, k, axis=0).min(0) if ncls > 1 else np.full(p.shape[0], np.inf)
            out["m5_without"][nm] = float((C & (other <= REACH)).sum() / n_c)
            out["m5_only"][nm] = float((C & (rk[k] <= REACH)).sum() / n_c)
        # The reach histogram per class of the qualifying ground.
        out["reach_hist_by_class"] = {nm: hist(np.where(hi[qi[jc == k]], -1.0, reach[qi[jc == k]]),
                                               [-2, 0, 50, 100, 150, 200.1]) for k, nm in enumerate(HCLS)}
    # How far the high ground stands from the deep ocean floor (deeper than 2,500 m below the sea, a
    # stand-in for oceanic crust that reads the same on both planets).
    deep = z < -2500.0
    if deep.any():
        dtree = cKDTree(p[deep])
        dd_h = cc.arc_km(dtree.query(p[H])[0]) if H.any() else np.zeros(0)
        dd_q = cc.arc_km(dtree.query(p[j])[0]) if qi.size else np.zeros(0)
        out["hi_ddeep_hist"] = hist(dd_h, [0, 100, 200, 400, 800, 1e9])
        out["q_src_ddeep_hist"] = hist(dd_q, [0, 100, 200, 400, 800, 1e9])
        out["ddeep_edges"] = "<100, 100-200, 200-400, 400-800, >=800 km to sea floor deeper than 2.5 km"
        out["hi_ddeep_p50"] = float(np.median(dd_h)) if dd_h.size else None
        out["q_src_ddeep_p50"] = float(np.median(dd_q)) if dd_q.size else None
    # f_near by kind of high ground: the model's collision regime against the rest; Earth's high ground
    # within 500 km of a PB2002 collision step (CCB, OCB), else of a subduction step, against the rest.
    if reg is not None:
        kinds = (("collision", reg == 5), ("other", reg != 5))
    elif coll_d is not None:
        kinds = (("collision", coll_d <= 500.0), ("subduction", (coll_d > 500.0) & (sub_d <= 500.0)),
                 ("other", (coll_d > 500.0) & (sub_d > 500.0)))
    else:
        kinds = ()
    for nm, km in kinds:
        hk = H & km
        out[f"f_near_{nm}"] = float((hk & near).sum() / max(hk.sum(), 1))
        out[f"hi_share_{nm}"] = float(hk.sum() / max(H.sum(), 1))
        out[f"hi_near_of_land_{nm}"] = float((hk & near).sum() / max(L.sum(), 1))
    if reg is not None:
        rj = reg[j]
        out["q_by_src_regime"] = {REG_NAMES[k]: float((rj == k).mean()) for k in range(10)}
        out["hi_by_regime"] = {REG_NAMES[k]: float((H & (reg == k)).sum() / max(H.sum(), 1)) for k in range(10)}
    if extra:
        for nm, v in extra.items():
            out[f"src_{nm}_med"] = float(np.nanmedian(v[j])) if qi.size else None
            out[f"hi_{nm}_med"] = float(np.nanmedian(v[H])) if H.any() else None
            out[f"coast_{nm}_med"] = float(np.nanmedian(v[C])) if C.any() else None
    return out


def model_run(run):
    f, _g, e = cc.load_dump(Path(run) / "fields")
    h, w, r = e["h"], e["w"], e["r"]
    frow, fcol, fp = cc.fib_sample(h, w, r)
    _lat, ok_r = cc.lat_of_rows(h, r, w * r)
    ncell = int(ok_r.sum()) * w
    sp = float(np.sqrt(cc.EARTH_KM2 / ncell))
    z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[frow, fcol].astype(float)
    oro = np.rint(np.nan_to_num(f["plates/orogeny_type"][frow, fcol], nan=0.0)).astype(int)
    age = f["plates/orogeny_age_myr"][frow, fcol].astype(float)
    reg = np.rint(np.nan_to_num(f["regime"][frow, fcol], nan=0.0)).astype(int)
    young = np.nan_to_num(age, nan=1e9) < YOUNG_MYR
    hcls = np.where(oro == 2, np.where(young, 0, 1), np.where(oro == 1, np.where(young, 2, 3), 4))
    thick = f["plates/thickness_m"][frow, fcol].astype(float) / 1000.0
    ice = np.nan_to_num(f["climate/ice_thickness_m"][frow, fcol].astype(float))
    platez = f["plates/elevation_m"][frow, fcol].astype(float)
    surfz = f["surface/elevation_m"][frow, fcol].astype(float)
    extra = {"thick_km": thick, "ice_m": ice, "oro_age": age, "plate_minus_surface_m": platez - surfz}
    keep = np.ones(z.size, bool)
    out = analyse(fp, z, sp, keep, hcls=hcls, ncls=len(HCLS), reg=reg, extra=extra)
    out["run"] = str(run)
    return out


def earth(grd, steps_path):
    sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "m5diag"))
    import m5earth  # noqa: E402
    zg, la, lo = m5earth.load(grd)
    n = 332_800
    plat, plon = cc.fib_points(n)
    z, plon_s = m5earth.sample(zg, la, lo, plat, plon)
    p = cc.xyz(plat, plon)
    sp = float(np.sqrt(cc.EARTH_KM2 / n))
    sheets = m5earth.sheets_of(plat, plon_s)
    sub_d = coll_d = None
    if steps_path:
        st = m5earth.load_steps(steps_path)
        allp = np.arange(n)
        sub_d = np.full(n, np.inf)
        coll_d = np.full(n, np.inf)
        sub_d[allp] = cc.arc_km(cKDTree(st["SUB"]).query(p)[0])
        coll_d[allp] = cc.arc_km(cKDTree(np.concatenate([st[k] for k in ("CCB", "OCB") if k in st])).query(p)[0])
    res = {"no_sheets": analyse(p, z.astype(float), sp, ~sheets, sub_d=sub_d, coll_d=coll_d),
           "all": analyse(p, z.astype(float), sp, np.ones(n, bool), sub_d=sub_d, coll_d=coll_d)}
    return res


def paleodem(paths):
    """The PaleoDEM maps the realism checks read (RLC-051), sampled as earth-paleodem.py samples them
    (the node at or after each point's latitude and longitude), with and without the ice-sheet box."""
    import importlib.util
    spec = importlib.util.spec_from_file_location("earth_paleodem",
                                                  Path(__file__).resolve().parents[1] / "earth-paleodem.py")
    ep = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(ep)
    n = 332_800
    plat, plon = cc.fib_points(n)
    p = cc.xyz(plat, plon)
    sp = float(np.sqrt(cc.EARTH_KM2 / n))
    sheets = cc.ice_sheets(plat, plon)
    res = {}
    for path in paths:
        z, lat, lon = ep.read_grid(path)
        zp = z[np.clip(np.searchsorted(lat, plat), 0, lat.size - 1),
               np.clip(np.searchsorted(lon, plon), 0, lon.size - 1)].astype(float)
        name = Path(path).name
        # The 0 Ma map carries Antarctica's bedrock (the South Pole reads 40 m, Vostok -40 m, the
        # Wilkes basin -760 m) but Greenland's ice surface (its summit 3,300 m): the by-rock reading
        # keeps Greenland alone out.
        greenland = sheets & (plat > 0)
        res[name] = {"all": analyse(p, zp, sp, np.ones(n, bool)), "no_sheets": analyse(p, zp, sp, ~sheets),
                     "no_greenland": analyse(p, zp, sp, ~greenland)}
        for k, v in res[name].items():
            print(f"{name} {k}: M5 {v['M5']:.4f} M4 {v['M4']:.4f} f_near {v['f_near']:.3f} s {v['s']:.3f}"
                  f" kappa {v['kappa']:.4f}", flush=True)
    return res


def main():
    mode, out = sys.argv[1], Path(sys.argv[2])
    if mode == "model":
        with out.open("a") as fh:
            for run in sys.argv[3:]:
                res = model_run(run)
                fh.write(json.dumps(res) + "\n")
                fh.flush()
                print(f"{Path(run).name}: M5 {res['M5']:.3f} M4 {res['M4']:.3f} f_near {res['f_near']:.3f}"
                      f" s {res['s']:.3f} kappa {res['kappa']:.4f}", flush=True)
    elif mode == "earth":
        res = earth(sys.argv[3], sys.argv[4] if len(sys.argv) > 4 else None)
        out.write_text(json.dumps(res, indent=1))
        for k, v in res.items():
            print(f"{k}: M5 {v['M5']:.4f} M4 {v['M4']:.4f} f_near {v['f_near']:.3f} s {v['s']:.3f} kappa {v['kappa']:.4f}")
    elif mode == "paleodem":
        out.write_text(json.dumps(paleodem(sys.argv[3:]), indent=1))
    print("COASTHI_DONE")


if __name__ == "__main__":
    main()
