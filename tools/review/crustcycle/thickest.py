#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The thickest continental crust on cached pool runs' crust budget maps (no simulation):

    thickest.py <arm folder> [<arm folder> ...]

Per arm, over its 8 runs and every frame (`maps/t*.bin`): the largest continental column, km, and the share of the
visible continental samples (pooled over frames) at or over 60, 70, 80, 90 and 97 km.
"""
import sys
from pathlib import Path

import numpy as np

NF = 11
RUNS = [f"{s}-{n}" for s in ("supercontinent", "scattered_continents") for n in (1, 2, 3, 4)]
LINES = (60, 70, 80, 90, 97)


def main():
    for arm in sys.argv[1:]:
        tops, counts, total = [], np.zeros(len(LINES)), 0
        for r in RUNS:
            maps = Path(arm) / r / "maps"
            for f in sorted(maps.glob("t*.bin")):
                m = np.fromfile(f, "<f4").reshape(-1, NF)
                th = m[m[:, 4] > 0.5, 3].astype(np.float64) / 1000.0
                if th.size == 0:
                    continue
                tops.append(th.max())
                total += th.size
                counts += np.array([(th >= x).sum() for x in LINES])
        print(f"-- {arm}: largest {max(tops):.1f} km, median of the frames' largest {np.median(tops):.1f} km")
        print("   share at or over " + ", ".join(f"{x} km {c / total:.2e}" for x, c in zip(LINES, counts)))


main()
