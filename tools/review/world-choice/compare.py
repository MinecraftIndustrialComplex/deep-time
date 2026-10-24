#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Cube phase 2, Ben's world-shape decision: one preset and seed baked as Petroff-Guyou (pg16) and
as the cube (cube16), compared.

  compare.py box  <cmp> <world>   the heightmap box of a world (x0,z0,x1,z1, its whole domain)
  compare.py plan <cmp>           the distortion of each world (from `deeptime heightmap --scale`
                                  at step 16), the places to shoot, and the client's vista strings

The plan writes <cmp>/compare.json (the numbers and places) and <cmp>/vistas-<world>-<near|far>.txt
(ScreenshotDriver's vista= property). A place is a lat/lon; each world puts the camera and the
target where its own projection draws them, so both views look at the same ground.
"""
import json
import math
import sys

import numpy as np
from scipy import ndimage

STEP = 16
WORLDS = ("pg16", "cube16")


def manifest(cmp, w):
    with open(f"{cmp}/{w}/deeptime/manifest.json") as f:
        return json.load(f)


def box(cmp, w):
    d = manifest(cmp, w)["projection"]["domain"]
    x0, x1 = math.floor(d["x_min"]), math.ceil(d["x_max"])
    z0, z1 = math.floor(d["z_min"]), math.ceil(d["z_max"]) + 1
    return x0, z0, x1, z1


def units(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], -1)


def wpct(v, w, q):
    """Weighted percentile q (0..100) of v."""
    o = np.argsort(v)
    v, w = v[o], w[o]
    c = np.cumsum(w)
    return float(np.interp(q / 100.0 * c[-1], c, v))


class World:
    def __init__(self, cmp, name):
        self.name = name
        a = np.load(f"{cmp}/{name}-hm.npy")
        with open(f"{cmp}/{name}-hm.npy.json") as f:
            meta = json.load(f)
        ch = meta["channels"]
        self.step = meta["step"]
        self.x0, self.z0, self.x1, self.z1 = meta["box"]
        g = lambda n: a[..., ch.index(n)].astype(np.float64)
        self.H, self.E = g("surface"), g("elevation")
        self.lat, self.lon, self.mpb = g("lat_deg"), g("lon_deg"), g("m_per_block")
        man = manifest(cmp, name)
        self.man = man
        self.sea = float(man["scale"]["sea_level_y"])
        self.proj = man["projection"]
        self.cube = self.proj["kind"] == "cube"
        rows, cols = self.H.shape
        self.xs = self.x0 + self.step * np.arange(cols)
        self.zs = self.z0 + self.step * np.arange(rows)
        self.X, self.Z = np.meshgrid(self.xs, self.zs)
        self.U = units(self.lat, self.lon)
        fin = np.isfinite(self.lat) & np.isfinite(self.lon) & np.isfinite(self.H)
        if self.cube:
            self.s = int(self.proj["face_blocks"])
            h = self.s / 2
            X, Z = self.X, self.Z
            band = (np.abs(Z) <= h) & (X >= -3 * h) & (X < 5 * h)
            column = (np.abs(X) <= h) & (np.abs(Z) <= 3 * h)
            self.dom = fin & (band | column)
            # Blocks to the nearest place off the six faces (the net's empty squares), so a place's
            # views stay clear of the turning edges.
            self.edge_dist = ndimage.distance_transform_edt(band | column) * self.step
        else:
            self.dom = fin
            d = self.proj["domain"]
            self.edge_dist = np.minimum(self.Z - d["z_min"], d["z_max"] - self.Z)
        self.N = self.blocks()
        self.jacobian()
        self.land = self.dom & (self.E >= self.sea)

    def blocks(self):
        if self.cube:
            return 6.0 * self.s * self.s
        d = self.proj["domain"]
        return (d["x_max"] - d["x_min"]) * (d["z_max"] - d["z_min"])

    def jacobian(self):
        """dU/dx and dU/dz per block at each sample (central differences), where both one-sided
        differences agree (no seam, crease or cone between the neighbours)."""
        U, st = self.U, self.step
        ok = self.dom.copy()
        Jx = np.full(U.shape, np.nan)
        Jz = np.full(U.shape, np.nan)
        f = U[:, 2:] - U[:, 1:-1]
        b = U[:, 1:-1] - U[:, :-2]
        Jx[:, 1:-1] = (f + b) / (2 * st)
        agree_x = np.zeros(ok.shape, bool)
        agree_x[:, 1:-1] = (np.linalg.norm(f - b, axis=-1) < 0.25 * np.linalg.norm(f + b, axis=-1) / 2) \
            & self.dom[:, 2:] & self.dom[:, :-2]
        f = U[2:] - U[1:-1]
        b = U[1:-1] - U[:-2]
        Jz[1:-1] = (f + b) / (2 * st)
        agree_z = np.zeros(ok.shape, bool)
        agree_z[1:-1] = (np.linalg.norm(f - b, axis=-1) < 0.25 * np.linalg.norm(f + b, axis=-1) / 2) \
            & self.dom[2:] & self.dom[:-2]
        self.jok = ok & agree_x & agree_z
        self.Jx, self.Jz = Jx, Jz
        a = np.einsum("...i,...i", Jx, Jx)
        bb = np.einsum("...i,...i", Jx, Jz)
        c = np.einsum("...i,...i", Jz, Jz)
        tr, det = a + c, a * c - bb * bb
        disc = np.sqrt(np.maximum(tr * tr / 4 - det, 0))
        s1 = np.sqrt(np.maximum(tr / 2 + disc, 0))
        s2 = np.sqrt(np.maximum(tr / 2 - disc, 0))
        with np.errstate(divide="ignore", invalid="ignore"):
            self.stretch = s1 / s2
            # Blocks per unit of planet area over the world's mean: how much larger than its true
            # size the world draws the ground there.
            self.area = 4 * math.pi / (s1 * s2 * self.N)
        self.patch = s1 * s2 * st * st  # planet area (steradians) a sample stands for

    def index(self, x, z):
        if self.cube:
            p = 4 * self.s
            x = (x + 1.5 * self.s) % p - 1.5 * self.s
        else:
            d = self.proj["domain"]
            w = d["x_max"] - d["x_min"]
            x = (x - d["x_min"]) % w + d["x_min"]
        j = int(round((x - self.x0) / self.step))
        i = int(round((z - self.z0) / self.step))
        i = min(max(i, 0), self.H.shape[0] - 1)
        j = min(max(j, 0), self.H.shape[1] - 1)
        return i, j

    def height(self, x, z):
        i, j = self.index(x, z)
        return max(self.H[i, j], self.sea) if np.isfinite(self.H[i, j]) else self.sea

    def ground(self, x, z):
        i, j = self.index(x, z)
        return self.H[i, j]

    def latlon(self, x, z):
        i, j = self.index(x, z)
        return float(self.lat[i, j]), float(self.lon[i, j])

    def at(self, u, mask=None):
        """The block of the planet point u (a unit vector): the nearest usable sample, then a
        Newton step through its Jacobian."""
        m = self.jok if mask is None else (self.jok & mask)
        dots = np.where(m, self.U @ u, -2.0)
        i, j = np.unravel_index(np.argmax(dots), dots.shape)
        x, z = float(self.X[i, j]), float(self.Z[i, j])
        J = np.stack([self.Jx[i, j], self.Jz[i, j]], -1)
        d = u - self.U[i, j]
        delta, *_ = np.linalg.lstsq(J, d - np.dot(d, self.U[i, j]) * self.U[i, j], rcond=None)
        if np.hypot(*delta) < 2 * self.step:
            x, z = x + delta[0], z + delta[1]
        return x, z, (i, j)

    def toward(self, ij, u_from, u_to):
        """The block step from u_from towards u_to: the great circle's tangent at u_from times
        its angle, through the Jacobian at sample ij."""
        i, j = ij
        J = np.stack([self.Jx[i, j], self.Jz[i, j]], -1)
        c = float(np.clip(np.dot(u_from, u_to), -1, 1))
        t = u_to - c * u_from
        n = np.linalg.norm(t)
        if n < 1e-12:
            return np.zeros(2)
        delta, *_ = np.linalg.lstsq(J, t / n * math.acos(c), rcond=None)
        return delta

    def north(self, ij, u):
        i, j = ij
        J = np.stack([self.Jx[i, j], self.Jz[i, j]], -1)
        lat = math.asin(max(-1.0, min(1.0, u[2])))
        lon = math.atan2(u[1], u[0])
        n = np.array([-math.sin(lat) * math.cos(lon), -math.sin(lat) * math.sin(lon), math.cos(lat)])
        d, *_ = np.linalg.lstsq(J, n, rcond=None)
        return d / np.linalg.norm(d)


def yaw_of(d):
    """Minecraft's yaw facing block direction d = (dx, dz): the look vector is (-sin yaw, cos yaw)."""
    return math.degrees(math.atan2(-d[0], d[1]))


def ll(u):
    return math.degrees(math.asin(max(-1.0, min(1.0, u[2])))), math.degrees(math.atan2(u[1], u[0])) % 360


def unit_ll(lat, lon):
    return units(np.array(lat), np.array(lon))


def view(w, cam, tgt, mode, lift=45.0, y=None, pitch=None, mask=None):
    """A vista in world w: stand above the planet point cam, look at the planet point tgt."""
    cx, cz, ij = w.at(cam, mask)
    delta = w.toward(ij, cam, tgt)
    D = float(np.hypot(*delta))
    n = max(2, int(D / 8))
    top = max(w.height(cx + delta[0] * k / n, cz + delta[1] * k / n) for k in range(n + 1))
    yt = w.height(cx + delta[0], cz + delta[1]) + 5
    yc = top + lift if y is None else y
    p = math.degrees(math.atan2(yc - yt, D)) if pitch is None else pitch
    p = min(max(p, 4.0), 60.0)
    return dict(x=round(cx, 1), y=round(yc, 1), z=round(cz, 1), yaw=round(yaw_of(delta), 1), pitch=round(p, 1),
                mode=mode, dist=round(D, 1), target=[round(cx + delta[0], 1), round(cz + delta[1], 1)],
                stretch=round(float(w.stretch[ij]), 3), area=round(float(w.area[ij]), 3),
                m_per_block=round(float(w.mpb[ij]), 1), face_blocks=w.s if w.cube else None)


def overhead(w, at, mode=0, r=120, lift=170, mask=None):
    cx, cz, ij = w.at(at, mask)
    nd = w.north(ij, at)
    top = max(w.height(cx + dx, cz + dz) for dx in range(-r, r + 1, 16) for dz in range(-r, r + 1, 16))
    return dict(x=round(cx, 1), y=round(top + lift, 1), z=round(cz, 1), yaw=round(yaw_of(nd), 1), pitch=90.0,
                mode=mode, dist=0, target=[round(cx, 1), round(cz, 1)],
                stretch=round(float(w.stretch[ij]), 3), area=round(float(w.area[ij]), 3),
                m_per_block=round(float(w.mpb[ij]), 1), face_blocks=w.s if w.cube else None)


def stats(w):
    out = {}
    for what, m in (("planet", w.jok), ("land", w.jok & w.land)):
        v = {}
        wt = w.patch[m]
        for k, arr in (("stretch", w.stretch), ("area", w.area)):
            a = arr[m]
            v[k] = {"min": float(a.min()), "p1": wpct(a, wt, 1), "median": wpct(a, wt, 50), "p90": wpct(a, wt, 90),
                    "p99": wpct(a, wt, 99), "max": float(a.max())}
        a = w.area[m]
        v["area_share_over_1.25"] = float(wt[a > 1.25].sum() / wt.sum())
        v["area_share_over_1.5"] = float(wt[a > 1.5].sum() / wt.sum())
        s = w.stretch[m]
        v["stretch_share_over_1.25"] = float(wt[s > 1.25].sum() / wt.sum())
        v["stretch_share_over_1.5"] = float(wt[s > 1.5].sum() / wt.sum())
        v["stretch_share_over_2"] = float(wt[s > 2.0].sum() / wt.sum())
        v["samples"] = int(m.sum())
        out[what] = v
    land = w.land & w.dom
    out["land_share_of_blocks"] = float(land.sum() / w.dom.sum())
    out["blocks"] = w.N
    return out


def pick(score, mask):
    s = np.where(mask, score, -np.inf)
    i, j = np.unravel_index(np.argmax(s), s.shape)
    return (i, j) if np.isfinite(s[i, j]) else None


def plan(cmp):
    W = {n: World(cmp, n) for n in WORLDS}
    pg, cu = W["pg16"], W["cube16"]
    h = cu.s / 2
    res = {"stats": {n: stats(w) for n, w in W.items()}, "places": {}}

    # How far a planet point lies from the trouble spots of the other world: PG's edges and cone
    # points, the cube's turning edges.
    def pg_clear(u, margin=400):
        x, z, ij = pg.at(u)
        if pg.edge_dist[ij] < margin:
            return False
        for cp in pg.proj["cone_points"]:
            for im in cp["images"]:
                if math.hypot(x - im[0], z - im[1]) < margin + 200:
                    return False
        return True

    def cube_clear(u, margin=400):
        x, z, ij = cu.at(u)
        return cu.edge_dist[ij] >= margin

    def best(world, score, mask, clear_other, n=60):
        s = np.where(mask & world.jok, score, -np.inf).ravel()
        for k in np.argsort(-s)[:n]:
            if not np.isfinite(s[k]):
                break
            ij = np.unravel_index(k, world.H.shape)
            u = world.U[ij]
            if clear_other(u):
                return ij, u
        return None, None

    relief = ndimage.maximum_filter(np.where(cu.dom, cu.H, -1e9), size=9) \
        - ndimage.minimum_filter(np.where(cu.dom, cu.H, 1e9), size=9)
    places = {}
    # High-latitude mountains: the highest ground between 55 and 78 degrees.
    m = cu.land & (np.abs(cu.lat) >= 55) & (np.abs(cu.lat) <= 78) & (cu.edge_dist >= 500)
    ij, u = best(cu, cu.H, m, pg_clear)
    places["mountains"] = u
    # A coast: land beside the sea within 3 samples, mid latitudes, the most relief nearby.
    sea_near = ndimage.maximum_filter((~cu.land & cu.dom).astype(np.uint8), size=7).astype(bool)
    m = cu.land & sea_near & (np.abs(cu.lat) >= 25) & (np.abs(cu.lat) <= 52) & (cu.edge_dist >= 500)
    ij, u = best(cu, relief, m, pg_clear)
    places["coast"] = u
    # PG's most stretched land (not at its edges, where the border stands).
    m = pg.land & (pg.edge_dist >= 200)
    ij, u = best(pg, pg.stretch, m, lambda u: cube_clear(u, 200))
    places["pg_stretch"] = u
    # The cube's most enlarged land (clear of the turning edges).
    m = cu.land & (cu.edge_dist >= 200)
    ij, u = best(cu, cu.area, m, lambda u: pg_clear(u, 200))
    places["cube_area"] = u

    def cube_pt(x, z):
        return unit_ll(*cu.latlon(x, z))

    # The cube's corner: F0's north-east corner (N, F0 and F1 meet).
    corner = cube_pt(h, -h)
    corner_cam = cube_pt(h - 120, -h + 120)
    corner_far = cube_pt(h - 520, -h + 520)
    # A turning edge, both of its images: the seam view with the most relief across it.
    with open(f"{cmp}/cube16-map/world-map.json") as f:
        wm = json.load(f)
    owner = {e["name"]: e["owner"] for e in cu.proj["edges"]}
    # The bake's seam views across the 12 turning edges (both sides owned by the wrap engine, so
    # bordered today), not the slide or the direct seams.
    turning = [v for v in wm.get("seam_views", []) if v["kind"] == "mountain"
               and all(owner.get(e) == "wrap_engine" for e in v["seam"].split("|"))]
    turning.sort(key=lambda v: -v["score"])
    edge_views = []
    if turning:
        v = turning[0]
        cx, cz = v["centre"]
        res["edge_seam"] = v
        # The two images of that point: the inward side of each is on a face, the other side off.
        u0 = cube_pt(cx, cz)
        imgs = []
        far = np.hypot(cu.X - cx, cu.Z - cz) > 400
        for (x, z) in ((cx, cz), cu.at(u0, far)[:2]):
            for d in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                i1, j1 = cu.index(x + 110 * d[0], z + 110 * d[1])
                i2, j2 = cu.index(x - 110 * d[0], z - 110 * d[1])
                if cu.dom[i1, j1] and not cu.dom[i2, j2]:
                    imgs.append(((x, z), d))
                    break
        for (x, z), d in imgs:
            edge_views.append((cube_pt(x + 110 * d[0], z + 110 * d[1]), cube_pt(x - 60 * d[0], z - 60 * d[1]),
                               (round(x), round(z))))
    res["edge_images"] = [e[2] for e in edge_views]
    # The slide (F2's east edge, x = 5h, glued to F3's west edge by the east-west wrap): where it
    # crosses the most relief.
    zs = np.arange(-h + 200, h - 200, 16)
    sl = max(zs, key=lambda z: cu.ground(5 * h - 24, z) + cu.ground(-3 * h + 24, z))
    slide_cam, slide_tgt = cube_pt(5 * h - 110, sl), cube_pt(-3 * h + 60, sl)
    # A straight edge (F0 | N, z = -h): the most relief where it crosses.
    xs = np.arange(-h + 200, h - 200, 16)
    st = max(xs, key=lambda x: cu.ground(x, -h + 24) + cu.ground(x, -h - 24))
    straight_cam, straight_tgt = cube_pt(st, -h + 110), cube_pt(st, -h - 60)
    # The north pole: on PG at its border (the north edge), on the cube in face N's middle. The
    # camera stands south of it as PG draws it (inside PG's world), at the same point on the cube.
    npole = np.array([0.0, 0.0, 1.0])
    pp = next(p for p in pg.proj["poles"] if p["name"].lower().startswith("n"))
    px, pz = pp["images"][0]
    pole_cam = unit_ll(*pg.latlon(px, pz + 150))
    pole_far = unit_ll(*pg.latlon(px, pz + 650))
    pole_top = unit_ll(*pg.latlon(px, pz + 64))

    def toward_pole(u):
        lat, lon = ll(u)
        return unit_ll(lat + (6 if lat > 0 else -6), lon)

    def away_equator(u, deg):
        lat, lon = ll(u)
        return unit_ll(lat - math.copysign(deg, lat), lon)

    near, far = {n: [] for n in WORLDS}, {n: [] for n in WORLDS}
    labels_near, labels_far = [], []

    def add(label, fn_near):
        labels_near.append(label)
        for n, w in W.items():
            near[n].append(fn_near(w))

    def add_far(label, fn):
        labels_far.append(label)
        for n, w in W.items():
            far[n].append(fn(w))

    mt, co = places["mountains"], places["coast"]
    if mt is not None:
        # About 3 degrees (130 blocks) equatorward, looking poleward at the peak.
        add("mountains: looking poleward", lambda w: view(w, away_equator(mt, 3.0), mt, 0))
        add("mountains: from above, north up", lambda w: overhead(w, mt))
    if co is not None:
        add("coast: looking poleward", lambda w: view(w, away_equator(co, 3.0), co, 0))
        add("coast: from above, north up", lambda w: overhead(w, co))
    add("north pole: looking at it from the south", lambda w: view(w, pole_cam, npole, 0, lift=60))
    add("north pole: from above, 60 blocks (as Petroff-Guyou draws it) south of it, north up",
        lambda w: overhead(w, pole_top))
    if places["pg_stretch"] is not None:
        add("Petroff-Guyou's most stretched land: from above, north up", lambda w: overhead(w, places["pg_stretch"]))
    if places["cube_area"] is not None:
        add("the cube's most enlarged land: from above, north up", lambda w: overhead(w, places["cube_area"]))
    add("the cube's corner (F0's north-east): looking at it", lambda w: view(w, corner_cam, corner, 0, lift=50))
    seam_name = res.get("edge_seam", {}).get("seam", "")
    for k, (cam, tgt, img) in enumerate(edge_views):
        side = seam_name.split("|")[k] if k < len(seam_name.split("|")) else f"image {k + 1}"
        add(f"a turning edge ({seam_name.replace('|', ' to ')}), from the {side.split('-')[0]} side at "
            f"x {img[0]}, z {img[1]}: looking across, F3 shown",
            lambda w, cam=cam, tgt=tgt: view(w, cam, tgt, 1, lift=40))
    add("the slide (F2 to F3): looking east across it, F3 shown", lambda w: view(w, slide_cam, slide_tgt, 1, lift=40))
    add("a straight edge (F0 to N): looking north across it", lambda w: view(w, straight_cam, straight_tgt, 0, lift=40))

    if mt is not None:
        add_far("far view: the mountains from 12 degrees equatorward, high",
                lambda w: view(w, away_equator(mt, 12.0), mt, 0, y=480.0, pitch=22.0))
    # The cube's far camera for the pole stands on face N, as far from the pole on the planet as
    # Petroff-Guyou's, towards F0 (+z): Petroff-Guyou's own point lies across a turning edge from it.
    pn = next(p for p in cu.proj["poles"] if p["name"] == "north")["images"][0]
    ang = math.acos(float(np.clip(np.dot(pole_far, npole), -1, 1)))
    zf = min(np.arange(pn[1], -h - 200, 16),
             key=lambda z: abs(math.acos(float(np.clip(np.dot(cube_pt(pn[0], z), npole), -1, 1))) - ang))
    pole_far_cube = cube_pt(pn[0], zf)
    add_far("far view: the north pole from the south, high",
            lambda w: view(w, pole_far_cube if w.cube else pole_far, npole, 0, y=480.0, pitch=22.0))
    add_far("far view: the cube's corner from inside F0, high",
            lambda w: view(w, corner_far, corner, 0, y=480.0, pitch=22.0))

    for n in WORLDS:
        for kind, vs in (("near", near[n]), ("far", far[n])):
            with open(f"{cmp}/vistas-{n}-{kind}.txt", "w") as f:
                f.write(";".join(f"{v['x']},{v['y']},{v['z']},{v['yaw']},{v['pitch']},{v['mode']}" for v in vs))
    res["near"] = {"labels": labels_near, **near}
    res["far"] = {"labels": labels_far, **far}
    res["places"] = {k: (list(ll(u)) if u is not None else None) for k, u in places.items()}
    res["corner"] = list(ll(corner))
    res["pole_cam"] = list(ll(pole_cam))
    res["slide_z"] = float(sl)
    res["straight_x"] = float(st)
    res["face_blocks"] = cu.s
    res["pg_domain"] = pg.proj["domain"]
    res["pg_cone_points"] = pg.proj["cone_points"]
    with open(f"{cmp}/compare.json", "w") as f:
        json.dump(res, f, indent=1, default=float)
    print(json.dumps(res["stats"], indent=1))
    for k, lab in enumerate(labels_near):
        print(f"near {k}: {lab}")
        for n in WORLDS:
            print(f"   {n}: {near[n][k]}")
    for k, lab in enumerate(labels_far):
        print(f"far {k}: {lab}")
        for n in WORLDS:
            print(f"   {n}: {far[n][k]}")


if __name__ == "__main__":
    if sys.argv[1] == "box":
        print("%d,%d,%d,%d" % box(sys.argv[2], sys.argv[3]))
    elif sys.argv[1] == "plan":
        plan(sys.argv[2])
    else:
        sys.exit(__doc__)
