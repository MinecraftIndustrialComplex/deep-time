#!/usr/bin/env python3
"""Bare ocean floor's height over GDH1 by its distance to the nearest thick ocean crust, split by that crust's kind
(arc crust or other: tails', heads', margins'), at one frame of cached pool runs. No simulation.

    bare_lift.py [--t T] <arm folder> [<arm folder> ...]
"""
import sys
from pathlib import Path

import numpy as np

NF = 11
NP = 9
HOTSPOT = 8
R_KM = 6371.0
RUNS = [f"{s}-{n}" for s in ("supercontinent", "scattered_continents") for n in (1, 2, 3, 4)]
DBINS = [(0, 200), (200, 400), (400, 800), (800, 1e9)]


def gdh1_depth(age):
    t = np.maximum(np.nan_to_num(age, nan=0.0), 0.0)
    return np.where(t < 20.0, 2600.0 + 365.0 * np.sqrt(t), 5651.0 - 2473.0 * np.exp(-0.0278 * t))


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], axis=1)


def nearest(src, dst):
    """For each src point, the great-circle distance (km) to the nearest dst point and its index."""
    if len(dst) == 0:
        return np.full(len(src), np.inf), np.zeros(len(src), np.int64)
    best = np.full(len(src), -2.0)
    idx = np.zeros(len(src), np.int64)
    for a in range(0, len(src), 2000):
        d = src[a:a + 2000] @ dst.T
        j = d.argmax(axis=1)
        best[a:a + 2000] = d[np.arange(len(j)), j]
        idx[a:a + 2000] = j
    return R_KM * np.arccos(np.clip(best, -1.0, 1.0)), idx


def one(run, t):
    m = np.fromfile(run / "maps" / f"t{t}.bin", "<f4").reshape(-1, NF).astype(np.float64)
    age = np.fromfile(run / "maps" / "age" / f"t{t}.bin", "<f4").astype(np.float64)
    pools = np.fromfile(run / "maps" / "pools" / f"t{t}.bin", "<f4").reshape(-1, NP).astype(np.float64)
    lat, lon, z, th, kind, arc_o, flags = m[:, 0], m[:, 1], m[:, 2], m[:, 3], m[:, 4], m[:, 7], m[:, 10]
    flags = flags.astype(np.int64)
    oc = kind < 0.5
    bare = oc & (th < 7200.0)
    thick = oc & (th >= 7200.0)
    res = z + gdh1_depth(age)
    p = xyz(lat, lon)
    bi = np.flatnonzero(bare)
    ti = np.flatnonzero(thick)
    arc_t = arc_o[ti] > 0.5
    d_any, j_any = nearest(p[bi], p[ti])
    d_arc, _ = nearest(p[bi], p[ti[arc_t]])
    d_oth, _ = nearest(p[bi], p[ti[~arc_t]])
    out = {}
    r = res[bi]
    for lo, hi in DBINS:
        for tag, d in (("any", d_any), ("arc", d_arc), ("oth", d_oth)):
            sel = (d >= lo) & (d < hi)
            out[f"{tag}{lo}_n"] = float(sel.sum()) / max(len(bi), 1)
            out[f"{tag}{lo}_res"] = float(r[sel].mean()) if sel.any() else float("nan")
    # Near any thick crust (under 200 km): the bare samples' own sediment and extra igneous rock, the neighbour's
    # thickness and its height over GDH1, and the hotspot flag.
    near = d_any < 200.0
    nb = bi[near]
    nt = ti[j_any[near]]
    out["near_sed"] = float((pools[nb, 0] + pools[nb, 1]).mean()) if near.any() else float("nan")
    out["near_extra"] = float((th[nb] - 7000.0 - pools[nb, 0] - pools[nb, 1]).mean()) if near.any() else float("nan")
    out["near_hot"] = float(((flags[nb] & HOTSPOT) != 0).mean()) if near.any() else float("nan")
    out["near_age"] = float(age[nb].mean()) if near.any() else float("nan")
    out["nb_th"] = float(th[nt].mean()) if near.any() else float("nan")
    out["nb_res"] = float(res[nt].mean()) if near.any() else float("nan")
    if near.sum() > 10:
        x, y = res[nt], res[nb]
        out["slope"] = float(np.polyfit(x, y, 1)[0])
    else:
        out["slope"] = float("nan")
    # The plates: each bare sample's distance to the nearest sample of another plate, and whether the nearest thick
    # crust is on its own plate.
    plate = np.fromfile(run / "maps" / "plate" / f"t{t}.bin", "<u4").astype(np.int64)
    d_edge = np.full(len(bi), np.inf)
    for q in np.unique(plate[bi]):
        on = plate[bi] == q
        other = np.flatnonzero(plate != q)
        d_edge[on], _ = nearest(p[bi[on]], p[other])
    same = plate[bi] == plate[ti[j_any]]
    for tag, sel in (("edge", d_edge < 400.0), ("inner", d_edge >= 400.0)):
        for nt_tag, nsel in (("near", near), ("all", np.ones(len(bi), bool))):
            k = sel & nsel
            out[f"{tag}_{nt_tag}_n"] = float(k.sum()) / max(len(bi), 1)
            out[f"{tag}_{nt_tag}_res"] = float(r[k].mean()) if k.any() else float("nan")
    out["near_same_plate"] = float(same[near].mean()) if near.any() else float("nan")
    for tag, k in (("near_same", near & same), ("near_other", near & ~same)):
        out[f"{tag}_res"] = float(r[k].mean()) if k.any() else float("nan")
    # The resampling's blend (ridges.rs `interpolate`): heights over all three corners of the sample's own plate, the
    # rock over its class. Bare floor beside its own plate's continental crust, beside its own plate's thick ocean
    # crust only, and beside neither (under 200 km).
    ci = np.flatnonzero(~oc)
    d_cs = np.full(len(bi), np.inf)
    d_ts = np.full(len(bi), np.inf)
    for q in np.unique(plate[bi]):
        on = plate[bi] == q
        cq = ci[plate[ci] == q]
        tq = ti[plate[ti] == q]
        d_cs[on], _ = nearest(p[bi[on]], p[cq])
        d_ts[on], _ = nearest(p[bi[on]], p[tq])
    by_cont = d_cs < 200.0
    by_thick = (d_ts < 200.0) & ~by_cont
    neither = ~by_cont & ~by_thick
    for tag, k in (("bycont", by_cont), ("bythick", by_thick), ("neither", neither)):
        out[f"{tag}_n"] = float(k.sum()) / max(len(bi), 1)
        out[f"{tag}_res"] = float(r[k].mean()) if k.any() else float("nan")
        out[f"{tag}_age"] = float(age[bi[k]].mean()) if k.any() else float("nan")
    # The same within one age band (60 to 200 Myr), so the classes' ages do not stand in for them.
    band = (age[bi] >= 60.0) & (age[bi] < 200.0)
    for tag, k in (("bycont", by_cont), ("bythick", by_thick), ("neither", neither)):
        kk = k & band
        out[f"{tag}_old_res"] = float(r[kk].mean()) if kk.any() else float("nan")
    subf = (flags[bi] & 256) != 0
    out["bare_sub_n"] = float(subf.mean())
    out["bare_sub_res"] = float(r[subf].mean()) if subf.any() else float("nan")
    far = d_any >= 800.0
    out["far_sed"] = float((pools[bi[far], 0] + pools[bi[far], 1]).mean()) if far.any() else float("nan")
    out["far_age"] = float(age[bi[far]].mean()) if far.any() else float("nan")
    return out


def fmt(v):
    if v != v:
        return "nan"
    return f"{v:.3f}" if abs(v) < 10 else f"{v:.0f}"


def main():
    args = sys.argv[1:]
    t = "0000"
    if args[:1] == ["--t"]:
        t, args = args[1], args[2:]
    for arm in args:
        rows = [one(Path(arm) / r, t) for r in RUNS if (Path(arm) / r / "maps" / f"t{t}.bin").exists()]
        keys = list(rows[0])
        print(f"-- {arm} t{t} runs {len(rows)}")
        med = {k: float(np.nanmedian([x[k] for x in rows])) for k in keys}
        for k in keys:
            print(f"{k} {fmt(med[k])}")


main()
