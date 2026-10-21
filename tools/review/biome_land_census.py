#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Does the biome map agree with the world's sea? (ledger: BAK-058; BIO-170 to BIO-174)

The biome inputs (`climate/continents`, L2) say land or ocean; the world's column rule
(`deeptime_format::water::sea_covers`, Java `Water.core`, step 3) says whether the sea stands on a
column: the ground's top block is under sea level - 1 and the `closed_basin` layer's nearest cell is
not 1. The rule is read here on the world's own raster: the `elevation` layer (the projected
height, no detail term) and `closed_basin` (HYD-105's redraw).

Two counts, both against that rule:
  - **land biome under sea water**: continents at or above the coast band's bottom (the reference's
    band edge 3, -0.19), where the sea stands;
  - **ocean biome on dry ground**: continents under that edge, where no sea stands (a lake, a river,
    a playa and closed ground below sea level are dry here: they are not the sea).
Each is counted on the **L2 nodes** (the biome layer's own cells, where the bake sets the value; the
rule is read at the node's block) and on the **L3 columns** (the biome layer read bilinear at the
column's block, as the game reads it, against the rule on the column: the sub-cell remainder along
a coast is what one L2 cell of 8 blocks cannot place), as km2 and as a share of the world's land
(the columns the sea does not cover). With `--compare BEFORE` it also counts the nodes whose five
biome inputs, wetland flag or Koppen class differ from another bake of the same planet, and how many
of them were mismatched there (the rest is collateral).

    python3 tools/review/biome_land_census.py --world <world-data> [--compare <world-data>] [--json out.json]

Needs numpy, scipy and lz4 (on the VM: nix shell --impure --expr
'(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages
(p: [p.numpy p.scipy p.lz4])' -c python3 ...).
"""

import argparse
import json
import sys
from pathlib import Path

import numpy as np
from scipy import ndimage

sys.path.insert(0, str(Path(__file__).resolve().parent))
import water_census as wc  # noqa: E402

INPUTS = ["temperature", "vegetation", "continents", "erosion", "ridges"]


def load(world: Path):
    man = json.loads((world / "manifest.json").read_text())
    scale = man["scale"]
    bands = man["notes"]["bake"]["climate"]["reference"]["bands"]["continents"]
    edge = float(bands[3])  # the ocean | coast edge of the reference
    r = world / "rasters"
    el, ed = wc.read_dtr(r / "elevation.dtr")
    h = el.astype(np.float32) * ed.get("scale", 1.0) + ed.get("offset", 0.0)
    cb = cbd = None
    p = r / "closed_basin.dtr"
    if p.exists():
        cb, cbd = wc.read_dtr(p)
    co, cod = wc.read_dtr(r / "climate" / "continents.dtr")
    c = co.astype(np.float32) * cod.get("scale", 1.0) + cod.get("offset", 0.0)
    return man, scale, edge, (h, ed["grid"]), (cb, cbd["grid"] if cbd else None), (c, cod["grid"])


def sea_covers(h: np.ndarray, closed: np.ndarray | None, sea_y: int) -> np.ndarray:
    """`water::sea_covers`: the top block ceil(H) - 1 is under sea level - 1, and not closed."""
    ground = np.ceil(h).astype(np.int32) - 1
    sea = ground < sea_y - 1
    if closed is not None:
        sea &= closed == 0
    return sea


def at_blocks(arr: np.ndarray, grid: dict, bx: np.ndarray, bz: np.ndarray) -> np.ndarray:
    """The nearest cell of `arr` (on `grid`) at blocks (bx, bz): x wraps, z clamps."""
    w, hh, r = grid["w"], grid["h"], grid["r"]
    i = np.rint((bx - grid["origin_block_x"]) / r).astype(np.int64) % w
    j = np.clip(np.rint((bz - grid["origin_block_z"]) / r).astype(np.int64), 0, hh - 1)
    return arr[j[:, None], i[None, :]]


def bilinear_at(arr: np.ndarray, grid: dict, bx: np.ndarray, bz: np.ndarray) -> np.ndarray:
    """`arr` read bilinear at blocks (bx, bz) (the game's reading of a climate layer): rows bz,
    columns bx; x wraps, z clamps."""
    w, hh, r = grid["w"], grid["h"], grid["r"]
    u = (bx - grid["origin_block_x"]) / r
    v = np.clip((bz - grid["origin_block_z"]) / r, 0, hh - 1)
    i0 = np.floor(u).astype(np.int64)
    j0 = np.floor(v).astype(np.int64)
    fu = (u - i0).astype(np.float32)
    fv = (v - j0).astype(np.float32)
    j1 = np.minimum(j0 + 1, hh - 1)
    ia, ib = i0 % w, (i0 + 1) % w
    a = arr[j0][:, ia] * (1 - fu)[None, :] + arr[j0][:, ib] * fu[None, :]
    b = arr[j1][:, ia] * (1 - fu)[None, :] + arr[j1][:, ib] * fu[None, :]
    return a * (1 - fv)[:, None] + b * fv[:, None]


def census(world: Path) -> dict:
    man, scale, edge, (h, hg), (cb, cbg), (c, cg) = load(world)
    sea_y = int(scale["sea_level_y"])
    m_h = scale["m_per_block_h"]
    out = {"sea_level_y": sea_y, "continents_edge": edge, "closed_layer": cb is not None}

    # The L2 nodes: the rule read at each node's block.
    bx2 = cg["origin_block_x"] + np.arange(cg["w"], dtype=np.int64) * cg["r"]
    bz2 = cg["origin_block_z"] + np.arange(cg["h"], dtype=np.int64) * cg["r"]
    h2 = at_blocks(h, hg, bx2, bz2)
    cl2 = at_blocks(cb, cbg, bx2, bz2) if cb is not None else None
    sea2 = sea_covers(h2, cl2, sea_y)
    ocean2 = c < edge
    cell2_km2 = (cg["r"] * m_h / 1000.0) ** 2
    land2_km2 = float((~sea2).sum()) * cell2_km2
    a1 = ~ocean2 & sea2
    a2 = ocean2 & ~sea2
    out["l2"] = {
        "cell_km2": cell2_km2,
        "land_km2": land2_km2,
        "land_biome_under_sea": {"cells": int(a1.sum()), "km2": float(a1.sum()) * cell2_km2,
                                 "share_of_land": float(a1.sum()) * cell2_km2 / land2_km2},
        "ocean_biome_on_dry": {"cells": int(a2.sum()), "km2": float(a2.sum()) * cell2_km2,
                               "share_of_land": float(a2.sum()) * cell2_km2 / land2_km2},
    }
    # Where the nodes are: beside the coast (within 2 cells of the other medium) or not.
    near_dry = ndimage.binary_dilation(~sea2, iterations=2)
    near_sea = ndimage.binary_dilation(sea2, iterations=2)
    ground = np.ceil(h2).astype(np.int32) - 1
    out["l2"]["land_biome_under_sea_interior"] = int((a1 & ~near_dry).sum())
    out["l2"]["ocean_biome_on_dry_interior"] = int((a2 & ~near_sea).sum())
    if cl2 is not None:
        out["l2"]["ocean_biome_on_closed_below_sea"] = int((a2 & (cl2 == 1) & (ground < sea_y - 1)).sum())
    out["l2"]["ocean_biome_on_dry_in_the_last_block_under_sea"] = int(
        (a2 & (cl2 != 1 if cl2 is not None else True) & (ground >= sea_y - 1) & (h2 < sea_y)).sum())
    out["l2"]["ocean_biome_on_dry_above_sea"] = int((a2 & (h2 >= sea_y)).sum())
    out["l2"]["land_biome_under_sea_deeper_than_1_block"] = int((a1 & (h2 < sea_y - 2)).sum())

    # The L3 columns: the layer read bilinear at the column, the rule on the column.
    bx3 = hg["origin_block_x"] + np.arange(hg["w"], dtype=np.int64) * hg["r"]
    bz3 = hg["origin_block_z"] + np.arange(hg["h"], dtype=np.int64) * hg["r"]
    cont3 = bilinear_at(c, cg, bx3.astype(np.float64), bz3.astype(np.float64))
    cl3 = None
    if cb is not None:
        cl3 = cb if (cbg["w"], cbg["h"]) == (hg["w"], hg["h"]) else at_blocks(cb, cbg, bx3, bz3)
    sea3 = sea_covers(h, cl3, sea_y)
    cell3_km2 = (hg["r"] * m_h / 1000.0) ** 2
    land3_km2 = float((~sea3).sum()) * cell3_km2
    b1 = (cont3 >= edge) & sea3
    b2 = (cont3 < edge) & ~sea3
    out["l3"] = {
        "cell_km2": cell3_km2,
        "land_km2": land3_km2,
        "land_biome_under_sea": {"cells": int(b1.sum()), "km2": float(b1.sum()) * cell3_km2,
                                 "share_of_land": float(b1.sum()) * cell3_km2 / land3_km2},
        "ocean_biome_on_dry": {"cells": int(b2.sum()), "km2": float(b2.sum()) * cell3_km2,
                               "share_of_land": float(b2.sum()) * cell3_km2 / land3_km2},
    }
    # How far from the rule's coast the mismatched columns lie (blocks): a coast-scale remainder
    # is within a cell of it, a pocket or a closed basin is not.
    dry_dist = ndimage.distance_transform_edt(sea3) * hg["r"]  # sea column -> blocks to dry
    sea_dist = ndimage.distance_transform_edt(~sea3) * hg["r"]  # dry column -> blocks to sea
    for key, m, d in (("land_biome_under_sea", b1, dry_dist), ("ocean_biome_on_dry", b2, sea_dist)):
        if m.any():
            v = d[m]
            out["l3"][key]["blocks_from_the_coast"] = {
                "p50": float(np.percentile(v, 50)), "p90": float(np.percentile(v, 90)),
                "max": float(v.max()),
                "beyond_one_l2_cell": int((v > cg["r"]).sum()),
                "beyond_one_l2_cell_km2": float((v > cg["r"]).sum()) * cell3_km2,
            }
    return out


def compare(a: Path, b: Path) -> dict:
    """Nodes where world `b` differs from `a` in the five inputs, the wetland flag or the class,
    and how many of those `a` had mismatched."""
    _, sa, edge, (ha, hga), (cba, cbga), (ca, cga) = load(a)
    sea_y = int(sa["sea_level_y"])
    bx2 = cga["origin_block_x"] + np.arange(cga["w"], dtype=np.int64) * cga["r"]
    bz2 = cga["origin_block_z"] + np.arange(cga["h"], dtype=np.int64) * cga["r"]
    sea_a = sea_covers(at_blocks(ha, hga, bx2, bz2),
                       at_blocks(cba, cbga, bx2, bz2) if cba is not None else None, sea_y)
    mism_a =((ca >= edge) & sea_a) | ((ca < edge) & ~sea_a)
    out = {"layers": {}}
    changed_any = np.zeros(ca.shape, bool)
    names = [f"climate/{n}" for n in INPUTS] + ["climate/wetland", "climate/koppen_l2"]
    for name in names:
        pa, pb = a / "rasters" / f"{name}.dtr", b / "rasters" / f"{name}.dtr"
        if not (pa.exists() and pb.exists()):
            continue
        xa, _ = wc.read_dtr(pa)
        xb, _ = wc.read_dtr(pb)
        d = xa != xb
        changed_any |= d
        out["layers"][name] = {"cells": int(d.sum()), "in_mismatch": int((d & mism_a).sum())}
    out["changed_cells"] = int(changed_any.sum())
    out["changed_in_before_mismatch"] = int((changed_any & mism_a).sum())
    out["collateral_cells"] = int((changed_any & ~mism_a).sum())
    out["before_mismatched_cells"] = int(mism_a.sum())
    out["before_mismatched_not_changed"] = int((mism_a & ~changed_any).sum())
    out["nodes"] = int(ca.size)
    # What the collateral is: the five inputs only, and how far from a mismatched node.
    near = ndimage.binary_dilation(mism_a, iterations=1)
    out["collateral_beside_a_mismatch"] = int((changed_any & ~mism_a & near).sum())
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--world", required=True, type=Path)
    ap.add_argument("--compare", type=Path, help="another bake of the same planet (the before)")
    ap.add_argument("--json", type=Path)
    a = ap.parse_args()
    res = {"world": str(a.world), "census": census(a.world)}
    if a.compare:
        res["compare_from"] = str(a.compare)
        res["compare"] = compare(a.compare, a.world)
    text = json.dumps(res, indent=1)
    if a.json:
        a.json.write_text(text + "\n")
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
