"""The island front from the crust budget's maps (measurement only, the island-front diagnosis,
2026-10-26).

    islmap.py <out.jsonl> <every Myr> <run folder> [<run folder> ...]

Per run, the snapshots after the first 100 Myr every <every> Myr, except those taken at a
resampling. On each: the visible samples' Delaunay triangulation, the subduction front (the
map's 256 flag on the overriding side, past incipient, as the boundary mix classed it), and its
island part by the crust budget's rule (no continental sample of the overriding plate within
2 lattice steps through that plate, `subduction::continent_within` with arc_steps 2), the same
rule at other reaches (path km through the plate), the front's lengths (raw Voronoi, Bird's
divider at 77 and 380 km, and on one lattice, `measure_boundgeom.k7`), and the island front's
Voronoi length split by what it is (plates, pieces, both sides, distances, ages, pairs, chains).
One JSON row per run on <out.jsonl>.
"""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import connected_components, dijkstra
from scipy.spatial import ConvexHull, cKDTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pool"))
import measure_boundgeom as mg  # noqa: E402

R = 6371.0
SUB = 256
COLL = 512
POOL_FIELDS = 9
BASEMENT = 7
AREA_SR = [0.0, 0.0377, 0.1, 0.5, 1.0]
CONT = [0.0, 0.01, 0.05, 0.15, 0.3, 0.5]
HOPS = [3, 4, 5, 9, 17]  # lower edges; past the last or none in the last bin
PATH_KM = [0.0, 200.0, 260.0, 400.0, 600.0, 1000.0]
EUCL_KM = [0.0, 200.0, 400.0, 600.0, 1000.0, 2000.0]
AGE = [0.0, 10.0, 30.0, 60.0, 100.0]
PAIR_KM = [0.0, 500.0, 1500.0, 3000.0, 6000.0]
CHAIN_KM = [0.0, 150.0, 300.0, 600.0, 1200.0, 2400.0]
REACH_KM = [143.0, 200.0, 260.0, 400.0, 600.0, 1000.0]


def band(x, edges):
    """Index of the band of each x among the lower edges (NaN and inf to the last band)."""
    x = np.asarray(x, dtype=np.float64)
    i = np.searchsorted(np.asarray(edges, dtype=np.float64), np.where(np.isfinite(x), x, 1e300),
                        side="right") - 1
    return np.clip(i, 0, len(edges) - 1)


def tri_geometry(xyz):
    hull = ConvexHull(xyz)
    tri, nbr = hull.simplices, hull.neighbors
    m = len(tri)
    a, b, c = xyz[tri[:, 0]], xyz[tri[:, 1]], xyz[tri[:, 2]]
    cc = np.cross(b - a, c - a)
    cc /= np.linalg.norm(cc, axis=1)[:, None]
    cc[np.einsum("ij,ij->i", cc, a) < 0.0] *= -1.0
    t_idx = np.repeat(np.arange(m), 3)
    k = np.tile(np.arange(3), m)
    u = tri[t_idx, (k + 1) % 3]
    v = tri[t_idx, (k + 2) % 3]
    t2 = nbr[t_idx, k]
    once = t_idx < t2
    eu, ev, e1, e2 = u[once], v[once], t_idx[once], t2[once]
    dots = np.clip(np.einsum("ij,ij->i", cc[e1], cc[e2]), -1.0, 1.0)
    vlen = R * np.arccos(dots)
    elen = R * np.arccos(np.clip(np.einsum("ij,ij->i", xyz[eu], xyz[ev]), -1.0, 1.0))
    return eu, ev, e1, e2, vlen, elen, cc


def chains(mask, e1, e2, cc):
    """The runs of the masked Voronoi edges: paths between nodes of degree other than 2 in the
    masked subgraph, closed loops too. Returns [(raw km, 77 km, 380 km)] per path."""
    idx = np.nonzero(mask)[0]
    if len(idx) == 0:
        return []
    a, b = e1[idx].tolist(), e2[idx].tolist()
    adj = {}
    for j, (x, y) in enumerate(zip(a, b)):
        adj.setdefault(x, []).append(j)
        adj.setdefault(y, []).append(j)
    seen = [False] * len(idx)
    out = []

    def walk(n0, j):
        nodes = [n0]
        n = n0
        while True:
            seen[j] = True
            nxt = b[j] if a[j] == n else a[j]
            nodes.append(nxt)
            nb = adj[nxt]
            if len(nb) != 2:
                break
            jn = nb[0] if nb[1] == j else nb[1]
            if seen[jn]:
                break
            n, j = nxt, jn
        return nodes

    paths = []
    for n0, js in adj.items():
        if len(js) != 2:
            for j in js:
                if not seen[j]:
                    paths.append(walk(n0, j))
    for j in range(len(idx)):
        if not seen[j]:
            paths.append(walk(a[j], j))
    for nodes in paths:
        P = [tuple(cc[n]) for n in nodes]
        raw = sum(R * mg.ang(P[i], P[i + 1]) for i in range(len(P) - 1))
        out.append((raw, mg.divider(P, 77.0), mg.divider(P, 380.0)))
    return out


def plate_stats(plate, kind):
    n = len(plate)
    ids, inv, counts = np.unique(plate, return_inverse=True, return_counts=True)
    cont = np.bincount(inv, weights=kind.astype(np.float64), minlength=len(ids))
    area_sr = counts * 4.0 * math.pi / n
    return ids, inv, area_sr, cont / counts, cont > 0


def front_halves(flags, plate, eu, ev, vlen, bit):
    """The front's half-edges (overriding sample o, the other sample u, Voronoi km, edge index):
    a cross-plate edge with the bit on o, kept only towards o's main other plate (the one with the
    most Voronoi length about o)."""
    x = plate[eu] != plate[ev]
    o_list, u_list, l_list, e_list = [], [], [], []
    for a, b in ((eu, ev), (ev, eu)):
        s = x & ((flags[a] & bit) != 0)
        o_list.append(a[s])
        u_list.append(b[s])
        l_list.append(vlen[s])
        e_list.append(np.nonzero(s)[0])
    o = np.concatenate(o_list)
    u = np.concatenate(u_list)
    ln = np.concatenate(l_list)
    ei = np.concatenate(e_list)
    if len(o) == 0:
        return o, u, ln, ei
    key = o.astype(np.int64) * (int(plate.max()) + 1) + plate[u]
    uk, kinv = np.unique(key, return_inverse=True)
    tot = np.bincount(kinv, weights=ln)
    ko = uk // (int(plate.max()) + 1)
    order = np.lexsort((-tot, ko))
    best = {}
    for j in order.tolist():
        best.setdefault(int(ko[j]), int(uk[j]))
    keep = np.array([best[int(oo)] == int(kk) for oo, kk in zip(o.tolist(), key.tolist())], dtype=bool)
    return o[keep], u[keep], ln[keep], ei[keep]


def path_dist(xyz, plate, eu, ev, elen, sources, limit, hops=False):
    """Distance through each sample's own plate (km, or lattice steps with `hops`) to the nearest
    source sample of that plate; inf past `limit` or with none."""
    n = len(plate)
    same = plate[eu] == plate[ev]
    w = np.ones(int(same.sum())) if hops else np.maximum(elen[same], 1e-6)
    g = coo_matrix((np.concatenate([w, w]),
                    (np.concatenate([eu[same], ev[same]]), np.concatenate([ev[same], eu[same]]))),
                   shape=(n, n)).tocsr()
    src = np.nonzero(sources)[0]
    if len(src) == 0:
        return np.full(n, np.inf)
    return dijkstra(g, directed=False, indices=src, min_only=True, limit=limit)


def largest_piece(plate, eu, ev):
    n = len(plate)
    same = plate[eu] == plate[ev]
    g = coo_matrix((np.ones(int(same.sum())), (eu[same], ev[same])), shape=(n, n))
    nc, lab = connected_components(g, directed=False)
    size = np.bincount(lab, minlength=nc)
    best = {}
    first = np.zeros(nc, dtype=np.int64)
    first[lab[::-1]] = np.arange(n)[::-1]
    cp = plate[first]
    for ci in np.argsort(-size, kind="stable").tolist():
        best.setdefault(int(cp[ci]), ci)
    return np.isin(lab, list(best.values()))


def add(acc, name, bins, weights, nb):
    v = np.bincount(bins, weights=weights, minlength=nb)[:nb]
    a = acc.setdefault(name, [0.0] * nb)
    for i in range(nb):
        a[i] += float(v[i])


_FIB = {}


def fib_geometry(n):
    if n not in _FIB:
        lat = mg.fib(n)
        _FIB[n] = (lat, tri_geometry(lat))
    return _FIB[n]


def lengths(flags, plate, island_bool, geom):
    """Raw, 77 km and 380 km lengths of the subduction front and of its island part on one map."""
    eu, ev, e1, e2, vlen, elen, cc = geom
    o, u, ln, ei = front_halves(flags, plate, eu, ev, vlen, SUB)
    m_sub = np.zeros(len(eu), dtype=bool)
    m_sub[ei] = True
    m_isl = np.zeros(len(eu), dtype=bool)
    m_isl[ei[island_bool[o]]] = True
    out = {}
    for name, m in (("sub", m_sub), ("isl", m_isl), ("cnt", m_sub & ~m_isl)):
        cs = chains(m, e1, e2, cc)
        out[name + "_raw"] = sum(c[0] for c in cs)
        out[name + "_77"] = sum(c[1] for c in cs)
        out[name + "_380"] = sum(c[2] for c in cs)
    return out, (o, u, ln, ei, m_isl)


def snapshot(xyz, plate, flags, kind, group, age, basement, spacing_km, acc, continent_any=True):
    """Measures one map into acc (sums) and returns the per-snapshot numbers."""
    geom = tri_geometry(xyz)
    eu, ev, e1, e2, vlen, elen, cc = geom
    hops = path_dist(xyz, plate, eu, ev, elen, kind, 40, hops=True)
    pkm = path_dist(xyz, plate, eu, ev, elen, kind, 3000.0)
    island = hops > 2
    snap, (o, u, ln, ei, m_isl) = lengths(flags, plate, island, geom)
    S = (flags & SUB) != 0
    snap["n_sub"] = int(S.sum())
    snap["n_isl"] = int((S & island).sum())
    snap["n_isl_km"] = snap["n_isl"] * spacing_km
    snap["n_sub_km"] = snap["n_sub"] * spacing_km
    # The island front at other reaches (path km through the overriding plate).
    for r in REACH_KM:
        far = island if r <= 143.0 else (pkm > r)
        snap["isl_reach%d_n_km" % int(r)] = int((S & far).sum()) * spacing_km
    ids, inv, area_sr, cshare, hascont = plate_stats(plate, kind)
    isl = island[o]
    oi, ui, li = o[isl], u[isl], ln[isl]
    cn = ~isl
    oc, uc, lc = o[cn], u[cn], ln[cn]
    snap["isl_vor"] = float(li.sum())
    snap["cnt_vor"] = float(lc.sum())
    for tag, oo, uu, ll in (("isl", oi, ui, li), ("cnt", oc, uc, lc)):
        add(acc, tag + "_over_area", band(area_sr[inv[oo]], AREA_SR), ll, len(AREA_SR))
        add(acc, tag + "_under_area", band(area_sr[inv[uu]], AREA_SR), ll, len(AREA_SR))
        add(acc, tag + "_over_cont", band(cshare[inv[oo]], CONT), ll, len(CONT))
        add(acc, tag + "_under_cont", band(cshare[inv[uu]], CONT), ll, len(CONT))
        add(acc, tag + "_under_kind", kind[uu].astype(np.int64), ll, 2)
        add(acc, tag + "_mutual", ((flags[uu] & (SUB | COLL)) != 0).astype(np.int64), ll, 2)
        add(acc, tag + "_sub_age", band(age[uu], AGE) if age is not None else np.zeros(len(uu), np.int64),
            ll, len(AGE))
    main = largest_piece(plate, eu, ev)
    add(acc, "isl_frag_over", (~main[oi]).astype(np.int64), li, 2)
    add(acc, "isl_frag_under", (~main[ui]).astype(np.int64), li, 2)
    add(acc, "cnt_frag_over", (~main[oc]).astype(np.int64), lc, 2)
    add(acc, "isl_hops", band(hops[oi], HOPS), li, len(HOPS))
    add(acc, "isl_path_km", band(pkm[oi], PATH_KM), li, len(PATH_KM))
    add(acc, "isl_over_hascont", hascont[inv[oi]].astype(np.int64), li, 2)
    if age is not None:
        add(acc, "isl_over_age", band(age[oi], AGE), li, len(AGE))
    if basement is not None:
        native = kind & ~(basement > 0.0)
        made = kind & (basement > 0.0)
        nkm = path_dist(xyz, plate, eu, ev, elen, native, 3000.0)
        add(acc, "isl_native_path_km", band(nkm[oi], PATH_KM), li, len(PATH_KM))
        add(acc, "cnt_cont_made", made[oc].astype(np.int64), lc, 2)
        # The continental front whose nearest own-plate continent within 2 steps is all made from
        # ocean floor (a matured arc): no native continent within 2 steps.
        nh = path_dist(xyz, plate, eu, ev, elen, native, 40, hops=True)
        add(acc, "cnt_native_within2", (nh[oc] <= 2).astype(np.int64), lc, 2)
        snap["n_isl_native2_km"] = int((S & (nh > 2)).sum()) * spacing_km
        for r in (200.0, 260.0, 400.0):
            snap[f"isl_native_reach{int(r)}_n_km"] = int((S & (nkm > r)).sum()) * spacing_km
    if continent_any:
        tree = cKDTree(xyz[kind]) if kind.any() else None
        if tree is not None:
            d, _ = tree.query(xyz[oi])
            add(acc, "isl_any_cont_km", band(R * 2.0 * np.arcsin(np.clip(d / 2.0, 0, 1)), EUCL_KM), li,
                len(EUCL_KM))
    # Pairs: the island front's Voronoi km per (over, under) plate pair in this snapshot.
    if len(oi):
        key = plate[oi].astype(np.int64) * (int(plate.max()) + 1) + plate[ui]
        uk, kinv = np.unique(key, return_inverse=True)
        tot = np.bincount(kinv, weights=li)
        add(acc, "isl_pair_km", band(tot[kinv], PAIR_KM), li, len(PAIR_KM))
        snap["isl_pairs"] = int(len(uk))
        top = np.sort(tot)[::-1]
        snap["isl_top1"] = float(top[0] / tot.sum())
        snap["isl_top5"] = float(top[:5].sum() / tot.sum())
    # Chains of the island front, by their length at 77 km.
    cs = chains(m_isl, e1, e2, cc)
    if cs:
        l77 = np.array([c[1] for c in cs])
        add(acc, "isl_chain77", band(l77, CHAIN_KM), l77, len(CHAIN_KM))
        snap["isl_chains"] = len(cs)
    # One lattice: the cleaned map on the 100,000-point Fibonacci lattice (k7).
    cleaned, frag = mg.clean(xyz, plate, group)
    snap["frag_samples"] = float(frag["frag_small"] + frag["frag_mid"] + frag["frag_big"])
    lat, p7, f7, k7_island = mg.k7(xyz, cleaned, flags & ~COLL, island)
    lat_, geom7 = fib_geometry(len(lat))
    l7, _ = lengths(f7, p7, k7_island, geom7)
    for k, v in l7.items():
        snap[k + "_k7"] = v
    return snap


def read_extra(maps, name, n):
    age = None
    ap = maps / "age" / name
    if ap.exists():
        a = np.fromfile(ap, "<f4").astype(np.float64)
        if len(a) == n:
            age = a
    basement = None
    pp = maps / "pools" / name
    if pp.exists():
        b = np.fromfile(pp, "<f4").reshape(-1, POOL_FIELDS)
        if len(b) == n:
            # Crust made continental during the run (a matured arc): it keeps the ocean floor's
            # formation time (start crust has formed_myr 0) and the ocean crust under it as its
            # basement (PLT-331; start crust carries at most traces). A float: 1 made, 0 native.
            made = (b[:, BASEMENT + 1] != 0.0) | (b[:, BASEMENT] > 1000.0)
            basement = made.astype(np.float64)
    return age, basement


def record_at(rec, t):
    best = None
    for s in rec["steps"]:
        if best is None or abs(s["t"] - t) < abs(best["t"] - t):
            best = s
    return best


def run_one(run, every):
    run = Path(run)
    maps = run / "maps"
    rec = json.loads((run / "rec.json").read_text())
    n_all = int(rec.get("samples", 0))
    spacing_km = R * math.sqrt(4.0 * math.pi / n_all)
    files = sorted(maps.glob("t*.bin"), key=lambda f: float(f.stem[1:]))
    t0 = rec["t0_myr"]
    acc = {}
    snaps = []
    for f in files:
        t = float(f.stem[1:])
        if t <= t0 + 100.0 or round(t - t0) % every:
            continue
        snap = mg.read_snapshot(f, maps / "plate" / f.name)
        if snap is None:
            continue
        xyz, plate, flags, kind, group = snap
        if len(plate) >= n_all:
            continue
        age, basement = read_extra(maps, f.name, len(plate))
        s = snapshot(xyz, plate, flags, kind, group, age, basement, spacing_km, acc)
        r = record_at(rec, t)
        s["t"] = t
        s["rec_t"] = r["t"]
        s["rec_oo"] = r["mix_km"][0]
        s["rec_oc"] = r["mix_km"][1]
        snaps.append(s)
        print(f"{run.name} t={t:.0f} isl n {s['n_isl_km']:.0f} rec {s['rec_oo']:.0f} isl77 {s['isl_77']:.0f} "
              f"k7 {s['isl_77_k7']:.0f}", file=sys.stderr, flush=True)
    return {"run": str(run), "spacing_km": spacing_km, "acc": acc, "snaps": snaps}


def main():
    out, every = Path(sys.argv[1]), int(sys.argv[2])
    # A pool folder also holds <run>.lock folders; keep only finished runs with maps.
    runs = [r for r in sys.argv[3:] if (Path(r) / "rec.json").is_file() and (Path(r) / "maps").is_dir()]
    from multiprocessing import Pool
    with Pool(int(__import__("os").environ.get("ISL_PROCS", "2"))) as pool, out.open("a") as fh:
        for row in pool.imap_unordered(_job, [(r, every) for r in runs]):
            fh.write(json.dumps(row) + "\n")
            fh.flush()


def _job(args):
    return run_one(*args)


if __name__ == "__main__":
    main()
