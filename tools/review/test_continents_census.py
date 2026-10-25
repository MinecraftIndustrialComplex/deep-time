#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Fixture tests of M5's three measure fixes (RLC-052; docs/research/notes/m5-coast-high-ground-2026-10-25.md):

  1. a closed body of sea is land, not the sea (a lake's shore is not a coast);
  2. the 200 km are over land (a strait wider than 1.6 spacings blocks the reach);
  3. the census grid is sampled on Fibonacci points (the grid loses the coast facing north or south
     above about 51 degrees).

    python3 -I tools/review/test_continents_census.py        (needs numpy and scipy)

Each fixture is a patch of the census's own Fibonacci point set (332,800 points, 39.1 km apart) cut
to a few thousand points around a centre, with the heights drawn in the patch's east/north km.
"""
import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE / "pool"))

import continents_census as cc  # noqa: E402
import measure_v1census as mv  # noqa: E402

N = 332_800
SP = float(np.sqrt(cc.EARTH_KM2 / N))


def patch(lat0, lon0, half_km):
    """The census's Fibonacci points within `half_km` of (lat0, lon0): (unit vectors, east km, north km)."""
    lat, lon = cc.fib_points(N)
    p = cc.xyz(lat, lon)
    c = cc.xyz(np.array([lat0]), np.array([lon0]))[0]
    east = np.cross([0.0, 0.0, 1.0], c)
    east /= np.linalg.norm(east)
    north = np.cross(c, east)
    sel = p @ c > np.cos(half_km / cc.EARTH_R_KM)
    p = p[sel]
    return p, cc.EARTH_R_KM * (p @ east), cc.EARTH_R_KM * (p @ north)


class Lakes(unittest.TestCase):
    def test_a_lakes_shore_is_not_a_coast(self):
        # An ocean south of y = -600 km; a closed basin (disc, 120 km, under the datum) 800 km inland
        # with high ground beside it. The version-1 census counts the lake's shore as coast with
        # high ground; the world ocean's coast has none within 200 km.
        p, x, y = patch(0.0, 0.0, 1300.0)
        z = np.full(x.size, 500.0)
        z[y < -600.0] = -1000.0
        z[np.hypot(x, y - 200.0) < 120.0] = -100.0
        z[np.hypot(x - 200.0, y - 200.0) < 60.0] = 3000.0
        old = cc.coast_metrics(p, z, SP, closed_as_land=False, over_land=False)
        new = cc.coast_metrics(p, z, SP)
        self.assertGreater(old["coast_hi_points"], 10)
        self.assertEqual(new["coast_hi_points"], 0)
        self.assertEqual(new["coast_with_hi"], 0.0)
        # The coast of the new census is the ocean's strip alone: the lake's rim is gone from it.
        lake_rim = (np.hypot(x, y - 200.0) < 120.0 + 1.6 * SP)
        geo = cc.coast_geometry(p, z, SP)
        self.assertFalse((geo["coast"] & lake_rim).any())
        self.assertTrue(geo["coast"][(y > -600.0) & (y < -600.0 + 0.8 * SP)].all())
        self.assertLess(new["coast_points"], old["coast_points"])

    def test_the_connected_sea_stays_the_sea(self):
        # A gulf open to the ocean through a 90 km mouth is the sea: its shore is a coast.
        p, x, y = patch(0.0, 0.0, 1300.0)
        z = np.full(x.size, 500.0)
        z[y < -600.0] = -1000.0
        z[(np.abs(x) < 45.0) & (y < 0.0)] = -1000.0  # a channel to a gulf
        z[(np.hypot(x, y - 100.0) < 150.0)] = -1000.0
        sea = z <= 0
        ocean = cc.world_ocean(p, sea, SP)
        self.assertTrue((ocean == sea).all())

    def test_only_the_largest_body_is_the_world_ocean(self):
        # Documented limit: two seas that do not join are two bodies; the smaller is a closed basin
        # whatever its size (the world ocean is the connected one, not a size cut).
        p, x, y = patch(0.0, 0.0, 1300.0)
        z = np.full(x.size, 500.0)
        z[x < -700.0] = -1000.0
        z[(x > 400.0) & (np.abs(y) < 500.0)] = -1000.0
        sea = z <= 0
        ocean = cc.world_ocean(p, sea, SP)
        self.assertTrue(ocean[x < -700.0].all())
        self.assertFalse(ocean[(x > 400.0) & (np.abs(y) < 500.0)].any())


class Reach(unittest.TestCase):
    def strait(self):
        # Ocean north of y = 300 km and a 120 km channel (x within 60 km) south from it to y = -500 km;
        # land elsewhere, joined round the channel's south end; high ground on the east bank.
        p, x, y = patch(0.0, 0.0, 1500.0)
        z = np.full(x.size, 500.0)
        z[y > 300.0] = -1000.0
        z[(np.abs(x) < 60.0) & (y > -500.0)] = -1000.0
        z[(x >= 60.0) & (x <= 160.0) & (np.abs(y) <= 100.0)] = 3000.0
        return p, x, y, z

    def test_a_reach_across_a_strait_is_blocked(self):
        p, x, y, z = self.strait()
        west = (x < -64.0) & (x > -90.0) & (np.abs(y) < 50.0)
        old = cc.coast_geometry(p, z, SP, closed_as_land=False, over_land=False)
        new = cc.coast_geometry(p, z, SP)
        # The fixture separates the two rules: the west bank's coast is 125 to 155 km from the
        # east bank's high ground in a straight line, on one landmass joined round the south end.
        self.assertTrue(old["coast"][west].all())
        self.assertTrue(old["has"][west].all())
        self.assertFalse(new["has"][west].any())
        self.assertTrue(np.isinf(new["reach"][west]).all())
        # The east bank (the high ground itself, and its shore) still counts.
        east = (x >= 60.0) & (x <= 120.0) & (np.abs(y) <= 100.0)
        self.assertTrue(new["has"][east & new["coast"]].all())
        self.assertTrue(new["coast"][east].any())

    def test_a_reach_over_land_is_kept_and_is_the_straight_run(self):
        # Ocean north of y = 300 km, a band of high ground 120 to 180 km behind the shore: every coast
        # point reaches it over land, by the straight run, so the lattice's path stretch does not
        # shorten the reach (the path alone would add up to 15 % to every distance).
        p, x, y = patch(0.0, 0.0, 1500.0)
        z = np.full(x.size, 500.0)
        z[y > 300.0] = -1000.0
        z[(y >= 120.0) & (y <= 180.0) & (np.abs(x) < 500.0)] = 3000.0
        new = cc.coast_geometry(p, z, SP)
        old = cc.coast_geometry(p, z, SP, closed_as_land=False, over_land=False)
        sel = new["coast"] & (np.abs(x) < 400.0)
        self.assertGreater(sel.sum(), 20)
        self.assertTrue(old["has"][sel].all())
        self.assertTrue(new["has"][sel].all())
        np.testing.assert_allclose(new["reach"][sel], old["reach"][sel], rtol=1e-9)

    def test_a_bay_with_a_way_round_over_land_is_not_a_wall(self):
        # A notch of sea 80 km wide and 45 km deep in the shore: the coast on its west side has high ground
        # across it 110 km away in a straight line over the water, and 150 km round the notch over land.
        p, x, y = patch(0.0, 0.0, 1500.0)
        z = np.full(x.size, 500.0)
        z[y > 300.0] = -1000.0
        west = np.zeros(x.size, bool)
        for x0 in (-800.0, -400.0, 0.0, 400.0, 800.0):  # five notches along the shore
            z[(np.abs(x - x0) < 40.0) & (y > 255.0)] = -1000.0
            z[(x - x0 >= 46.0) & (x - x0 <= 150.0) & (y >= 200.0) & (y <= 330.0)] = 3000.0
            west |= (x - x0 < -46.0) & (x - x0 > -85.0) & (y > 265.0) & (y < 300.0)
        new = cc.coast_geometry(p, z, SP)
        self.assertTrue(new["coast"][west].all())
        self.assertGreater(west.sum(), 5)
        self.assertTrue(new["has"][west].all())
        self.assertTrue((new["reach"][west] > 110.0).all())


def grid_dump(folder, h=328, w=1024, r=16):
    """A census-grid dump (equal-area cylindrical, as `deeptime dump-fields`): sea everywhere south of
    70 degrees north, a polar landmass north of it with a ring of high ground at 70.8 to 71.6 degrees."""
    lat_r, _ok = cc.lat_of_rows(h, r, w * r)
    lat = np.repeat(lat_r[:, None], w, 1)
    z = np.where(lat > 70.0, 500.0, -1000.0)
    z[(lat > 70.8) & (lat < 71.6)] = 3000.0
    elev = (z / cc.M_PER_BLOCK_V + cc.SEA_Y).astype("<f4")
    idx = {}
    for name, a in (("elevation", elev), ("plates/crust_type", np.zeros_like(elev)),
                    ("climate/ice_thickness_m", np.zeros_like(elev))):
        fn = name.replace("/", "_") + ".f32"
        a.astype("<f4").tofile(Path(folder) / fn)
        idx[name] = {"file": fn, "h": h, "w": w, "r": r}
    (Path(folder) / "index.json").write_text(json.dumps(idx))
    (Path(folder) / "globals.json").write_text(json.dumps({"globals": {}}))


class Grid(unittest.TestCase):
    def test_the_nearest_cell_of_each_point(self):
        h, w, r = 328, 1024, 16
        row, col, p = cc.fib_sample(h, w, r)
        lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
        self.assertEqual(row.size, int(ok_r.sum()) * w)
        self.assertTrue(ok_r[row].all())
        plat = np.degrees(np.arcsin(p[:, 2]))
        big_r = w * r / (2 * np.pi)
        # equal-area rows: the cell's sin(latitude) is within half a row of the point's (the two polar
        # caps beyond the outermost rows, 0.3 % of the sphere each, are the edge row's)
        err = np.abs(np.sin(np.radians(lat_r[row])) - np.sin(np.radians(plat)))
        inner = np.abs(np.sin(np.radians(plat))) < 0.99
        self.assertLessEqual(float(err[inner].max()), 0.5 * r / big_r + 1e-9)
        self.assertLessEqual(float(err.max()), 1.0 * r / big_r + 1e-9)
        plon = np.degrees(np.arctan2(p[:, 1], p[:, 0])) % 360.0
        dlon = np.abs((plon - 360.0 * col / w + 180.0) % 360.0 - 180.0)
        self.assertLessEqual(float(dlon.max()), 180.0 / w + 1e-6)

    def test_a_high_latitude_coast_is_lost_on_the_grid_and_kept_on_fibonacci_points(self):
        with tempfile.TemporaryDirectory() as d:
            grid_dump(d)
            f, _g, e = cc.load_dump(d)
            h, w, r = e["h"], e["w"], e["r"]
            lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
            lat = np.repeat(lat_r[:, None], w, 1)
            ok = np.repeat(ok_r[:, None], w, 1)
            lon = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
            z = (f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V
            sp = float(np.sqrt(cc.EARTH_KM2 / ok.sum()))
            # The grid's north-south step at 70 degrees is wider than the coast band: no coast at all.
            self.assertGreater(sp / np.cos(np.radians(70.0)), 1.6 * sp)
            grid = cc.coast_metrics(cc.xyz(lat[ok], lon[ok]), z[ok], sp, closed_as_land=False, over_land=False)
            self.assertEqual(grid["coast_points"], 0)
            # The measure on the same field: version 1's value is 0, the new one sees the coast.
            v = mv.census(d)
            self.assertEqual(v["coast_hi_old"], 0.0)
            self.assertGreater(v["coast_hi"], 0.9)
            self.assertGreater(v["coast_hi_fib"], 0.9)
            # The coast is the ring's length (2 pi R cos(70) = 13,690 km) times the band, less the half
            # spacing the nearest sea point stands off the shore.
            row, col, fp = cc.fib_sample(h, w, r)
            fib = cc.coast_metrics(fp, z[row, col], sp)
            band_points = 2 * np.pi * cc.EARTH_R_KM * np.cos(np.radians(70.0)) * 1.6 * sp / sp ** 2
            self.assertGreater(fib["coast_points"], 0.5 * band_points)
            self.assertLess(fib["coast_points"], 1.1 * band_points)


if __name__ == "__main__":
    unittest.main()
