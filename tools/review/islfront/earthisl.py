"""Earth's island subduction front by the model's rule (measurement only, the island-front diagnosis,
2026-10-26).

    earthisl.py <PB2002_steps.dat> <crust1.bnds> <bird.py dir> <out.json>

Bird 2003's SUB steps (PB2002) with their polarity (bird_raster.py's: "/" the right plate subducts,
"\\" the left). For each step the overriding plate's side is sampled on a polar grid about the step's
midpoint (radii to 2,500 km, every 10 degrees of azimuth); a grid point is on the overriding plate
when the nearest boundary step puts it on that plate's side (bird_raster.py's side test). CRUST1.0
(Laske et al. 2013) gives each point's crystalline crust (top of the crystalline layers to the Moho)
and its whole crust (below water and ice to the Moho). A step is "island" at reach D and threshold T
when no point of its overriding plate within D km has crust of at least T km: the crust budget's
rule (`subduction::continent_within`, the overriding plate's continental crust within arc_steps),
with Earth's crust standing for the model's continental kind. Per step: the distance to such crust
through any azimuth on the overriding plate, along the normal into it, and on any plate.
"""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

steps_path, bnds_path, bird_dir, out_path = sys.argv[1:5]
sys.path.insert(0, bird_dir)
import bird  # noqa: E402

R = 6371.0
rows = bird.read_steps(steps_path)

# CRUST1.0: 64,800 cells of 1 degree, latitude 89.5 down to -89.5 outer, longitude -179.5 to 179.5 inner;
# nine boundary elevations (km): top of water, bottom of water, bottom of ice, bottom of sediments 1-3,
# bottom of crystalline 1-3 (the last is the Moho).
B = np.loadtxt(bnds_path)
assert B.shape == (64800, 9), B.shape
CRYST = B[:, 5] - B[:, 8]
WHOLE = B[:, 2] - B[:, 8]
ELEV = B[:, 0]


def cell(lat, lon):
    i = np.clip(np.floor(90.0 - lat).astype(np.int64), 0, 179)
    j = np.clip(np.floor(lon + 180.0).astype(np.int64), 0, 359)
    return i * 360 + j


def ll2v_arr(lon, lat):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], axis=-1)


def v2ll(v):
    lat = np.degrees(np.arcsin(np.clip(v[..., 2], -1.0, 1.0)))
    lon = np.degrees(np.arctan2(v[..., 1], v[..., 0]))
    return lat, lon


# Every step densified at 5 km with its owner, for the side test.
A = ll2v_arr(np.array([r["lon0"] for r in rows]), np.array([r["lat0"] for r in rows]))
Bv = ll2v_arr(np.array([r["lon1"] for r in rows]), np.array([r["lat1"] for r in rows]))
dense, owner = [], []
for k in range(len(rows)):
    d = R * math.acos(max(-1.0, min(1.0, float(A[k] @ Bv[k]))))
    n = max(2, int(d / 5.0) + 2)
    s = np.linspace(0.0, 1.0, n)[:, None]
    q = A[k][None, :] + s * (Bv[k] - A[k])[None, :]
    dense.append(q / np.linalg.norm(q, axis=1)[:, None])
    owner.append(np.full(n, k))
dense = np.concatenate(dense)
owner = np.concatenate(owner)
tree = cKDTree(dense)
CROSS = np.cross(A, Bv)
LEFT = [r["bid"][0:2] for r in rows]
RIGHT = [r["bid"][3:5] for r in rows]
SYM = [r["bid"][2] for r in rows]


def plate_of(q):
    """bird_raster.py's side test: the plate on q's side of its nearest boundary step."""
    _, j = tree.query(q.reshape(-1, 3))
    k = owner[j]
    left = np.einsum("ij,ij->i", q.reshape(-1, 3), CROSS[k]) > 0.0
    names = np.where(left, np.array(LEFT, dtype=object)[k], np.array(RIGHT, dtype=object)[k])
    return names.reshape(q.shape[:-1])


RADII = np.concatenate([np.arange(10.0, 1000.0, 10.0), np.arange(1000.0, 2501.0, 50.0)])
AZ = np.radians(np.arange(0.0, 360.0, 10.0))
THRESH = [10.0, 15.0, 20.0, 25.0, 30.0]
REACH = [0.0, 77.0, 143.0, 154.0, 200.0, 260.0, 400.0, 600.0, 1000.0]

sub = [k for k, r in enumerate(rows) if r["cls"] == "SUB"]
out_steps = []
unpolarised = 0.0
for k in sub:
    r = rows[k]
    sym = SYM[k]
    if sym not in "/\\":
        unpolarised += r["length"]
        continue
    under = RIGHT[k] if sym == "/" else LEFT[k]
    over = LEFT[k] if sym == "/" else RIGHT[k]
    m = A[k] + Bv[k]
    m /= np.linalg.norm(m)
    t = Bv[k] - A[k]
    t -= (t @ m) * m
    t /= np.linalg.norm(t)
    nleft = np.cross(m, t)  # towards the left of the step's direction (cross(a, b) side)
    e1 = nleft if over == LEFT[k] else -nleft
    e2 = np.cross(m, e1)
    th = RADII / R
    dirs = np.cos(AZ)[:, None] * e1[None, :] + np.sin(AZ)[:, None] * e2[None, :]  # (naz, 3)
    q = np.cos(th)[:, None, None] * m[None, None, :] + np.sin(th)[:, None, None] * dirs[None, :, :]
    pl = plate_of(q)  # (nr, naz)
    own = pl == over
    lat, lon = v2ll(q)
    c = cell(lat, lon)
    cr, wh = CRYST[c], WHOLE[c]
    lat0, lon0 = v2ll(m)
    c0 = cell(np.array([lat0]), np.array([lon0]))[0]
    rec = {"k": k, "bid": r["bid"], "over": over, "under": under, "length": r["length"],
           "lat": float(lat0), "lon": float(lon0), "cryst0": float(CRYST[c0]), "whole0": float(WHOLE[c0]),
           "elev0": float(ELEV[c0]), "own_frac_200": float(own[RADII <= 200.0].mean())}
    for T in THRESH:
        for name, field in (("cryst", cr), ("whole", wh)):
            hit = field >= T
            dd_own = np.where(own & hit, RADII[:, None], np.inf).min()
            dd_norm = np.where(own[:, 0] & hit[:, 0], RADII, np.inf).min()
            dd_any = np.where(hit, RADII[:, None], np.inf).min()
            rec[f"{name}{int(T)}_own"] = float(dd_own)
            rec[f"{name}{int(T)}_normal"] = float(dd_norm)
            rec[f"{name}{int(T)}_any"] = float(dd_any)
    # The overriding side's crust along the normal at the arc's distance (150-250 km), its maximum.
    sel = (RADII >= 150.0) & (RADII <= 250.0)
    okn = own[sel, 0]
    rec["arc_cryst_max"] = float(cr[sel, 0][okn].max()) if okn.any() else float("nan")
    rec["arc_whole_max"] = float(wh[sel, 0][okn].max()) if okn.any() else float("nan")
    out_steps.append(rec)

total = sum(s["length"] for s in out_steps)
summary = {"sub_polarised_km": total, "sub_unpolarised_km": unpolarised, "grid": {}}
for T in THRESH:
    for name in ("cryst", "whole"):
        for mode in ("own", "normal", "any"):
            key = f"{name}{int(T)}_{mode}"
            summary["grid"][key] = {str(int(D)): sum(s["length"] for s in out_steps if s[key] > D) for D in REACH}
# Per plate pair (bid without the step's direction), with the island km at the model's rule.
pairs = {}
for s in out_steps:
    key = f"{s['over']}>{s['under']}"
    p = pairs.setdefault(key, {"km": 0.0, "lat": [], "lon": [], "isl_whole15_154": 0.0, "isl_cryst15_154": 0.0,
                               "isl_whole20_154": 0.0, "isl_whole15_260": 0.0, "isl_whole25_154": 0.0})
    p["km"] += s["length"]
    p["lat"].append(s["lat"])
    p["lon"].append(s["lon"])
    for tag, kk, D in (("isl_whole15_154", "whole15_own", 154.0), ("isl_cryst15_154", "cryst15_own", 154.0),
                       ("isl_whole20_154", "whole20_own", 154.0), ("isl_whole15_260", "whole15_own", 260.0),
                       ("isl_whole25_154", "whole25_own", 154.0)):
        if s[kk] > D:
            p[tag] += s["length"]
for p in pairs.values():
    p["lat"] = [min(p["lat"]), max(p["lat"])]
    p["lon"] = [min(p["lon"]), max(p["lon"])]
summary["pairs"] = pairs
Path(out_path).write_text(json.dumps({"summary": summary, "steps": out_steps}))
print(f"SUB polarised {total:.0f} km, unpolarised {unpolarised:.0f} km")
for name in ("whole", "cryst"):
    for mode in ("own", "normal", "any"):
        print(f"== island km by threshold (rows) and reach (columns), {name} crust, {mode}")
        print("     T " + " ".join(f"{int(D):>7}" for D in REACH))
        for T in THRESH:
            g = summary["grid"][f"{name}{int(T)}_{mode}"]
            print(f"  {int(T):>4} " + " ".join(f"{g[str(int(D))]:7.0f}" for D in REACH))
print("== pairs (over>under): km, island km at whole>=15 within 154 km; cryst15/154; whole20/154; whole25/154;"
      " whole15/260; lat, lon ranges")
for key, p in sorted(pairs.items(), key=lambda kv: -kv[1]["km"]):
    print(f"  {key:>7} {p['km']:7.0f} {p['isl_whole15_154']:7.0f} {p['isl_cryst15_154']:7.0f} "
          f"{p['isl_whole20_154']:7.0f} {p['isl_whole25_154']:7.0f} {p['isl_whole15_260']:7.0f}  "
          f"lat {p['lat'][0]:6.1f}..{p['lat'][1]:6.1f} lon {p['lon'][0]:7.1f}..{p['lon'][1]:7.1f}")
