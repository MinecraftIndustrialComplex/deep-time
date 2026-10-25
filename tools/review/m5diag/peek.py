#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""peek.py <run>: the count of each flag bit in a few of the run's maps (are the front flags there?)."""
import sys
from pathlib import Path

import numpy as np

run = Path(sys.argv[1])
for t in ("t-500", "t-300", "t-050", "t0000"):
    a = np.fromfile(run / "maps" / f"{t}.bin", "<f4").reshape(-1, 11)
    fl = a[:, 10].astype(np.int64)
    print(t, a.shape[0], {b: int(((fl & b) != 0).sum()) for b in (1, 2, 4, 8, 16, 32, 64, 128, 256, 512)})
