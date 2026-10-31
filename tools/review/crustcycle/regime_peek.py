#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""A run's final frame (fields/, the L1 grid): the regime codes' shares and the land and sea by them (no simulation).

    regime_peek.py <run folder> ...

Reads `regime.f32`, `plates__crust_type.f32` and `elevation.f32` (blocks over the sea level, continents_census's
scale) and prints, per regime code (plates/state.rs `regime`: 0 ocean basin, 1 ridge, 2 trench, 3 arc, 4 back-arc,
5 and up the rest), its share of the cells, of the land and of the sea. Whether the final frame still holds the
fronts' codes (the crust-budget map t0000 is the final resampling's lattice, with no TRENCH flags) decides whether a
drainage reading on it can tell active coasts from passive ones.
"""
import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import continents_census as cc  # noqa: E402


def main(argv):
    for arg in argv:
        f, _g, e = cc.load_dump(Path(arg) / "fields")
        reg = f["regime"].astype(np.int64)
        z = (f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V
        land = z > 0
        codes, counts = np.unique(reg, return_counts=True)
        print(f"== {arg}: {reg.size} cells; land {land.mean():.3f}")
        for c, k in zip(codes, counts):
            m = reg == c
            print(f"  regime {c:3d}: cells {k / reg.size:.4f}, of the land {(m & land).sum() / max(land.sum(), 1):.4f},"
                  f" of the sea {(m & ~land).sum() / max((~land).sum(), 1):.4f}")
        _ = json


if __name__ == "__main__":
    main(sys.argv[1:])
