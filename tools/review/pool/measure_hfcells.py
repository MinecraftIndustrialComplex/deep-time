#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `hfcells` measure: the continental heat flow of a run's final frame (PLT-137, PLT-467,
PLT-469), from one pool run folder, printed as a JSON object on stdout.

    python3 -I measure_hfcells.py <run folder>

From the final frame's dump (`fields/`, the L1 grid): `plates/continental_heat_flow_mw_m2` (the heat
flow interpolated over a cell's continental corners alone, NaN on oceanic crust and in gaps) and
`plates/crust_type` (1 continental). The proxy gives continental crust at least 55 mW/m2
(55 + 35 e^(-t/60), PLT-137), so a cell under it is crust that kept the sea floor's heat flow: the fault
the final pass (`tectonics.heat_flow_refresh`, PLT-469) fixes.

  cont_cells        L1 cells of continental crust with a continental heat flow
  under55_cells     of those, cells under 54.9 mW/m2 (the bake test's floor, 0.1 under the law's 55)
  under55_share     the same as a share of cont_cells
  q_min, q_p01, q_median, q_mean   the continental heat flow's minimum, 1st percentile, median and mean, mW/m2
  q_max             and its maximum
  over150_cells, over150_share   cells over 150 mW/m2 (the top of Earth's regional continental heat
                    flows, 100 to 150: PLT-823) and their share of cont_cells
  over200_cells, over200_share   cells over 200 mW/m2 (above it Lachenbruch et al. 1985 found only
                    geothermal fields) and their share
  all_q_min         the minimum of `plates/heat_flow_mw_m2` over cells whose crust type is continental
                    (the field the structure stage and the surface's geotherm read)

Version 2 (2026-10-31, the continental lid's floor, PLT-820: the shares over 150 and 200 added).
"""
import json
import sys
from pathlib import Path

import numpy as np

FLOOR = 54.9


def main():
    d = Path(sys.argv[1]) / "fields"
    idx = json.loads((d / "index.json").read_text())

    def get(name):
        e = idx[name]
        return np.fromfile(d / e["file"], "<f4").reshape(e["h"], e["w"]).astype(np.float64)

    kind = get("plates/crust_type")
    q = get("plates/continental_heat_flow_mw_m2")
    q_all = get("plates/heat_flow_mw_m2")
    cont = (kind > 0.5) & np.isfinite(q)
    v = q[cont]
    n = max(int(v.size), 1)
    ca = (kind > 0.5) & np.isfinite(q_all)
    out = {
        "cont_cells": float(v.size),
        "under55_cells": float((v < FLOOR).sum()),
        "under55_share": float((v < FLOOR).sum()) / n,
        "q_min": float(v.min()) if v.size else float("nan"),
        "q_p01": float(np.percentile(v, 1)) if v.size else float("nan"),
        "q_median": float(np.median(v)) if v.size else float("nan"),
        "q_mean": float(v.mean()) if v.size else float("nan"),
        "q_max": float(v.max()) if v.size else float("nan"),
        "over150_cells": float((v > 150.0).sum()),
        "over150_share": float((v > 150.0).sum()) / n,
        "over200_cells": float((v > 200.0).sum()),
        "over200_share": float((v > 200.0).sum()) / n,
        "all_q_min": float(q_all[ca].min()) if ca.any() else float("nan"),
    }
    print(json.dumps(out, sort_keys=True))


if __name__ == "__main__":
    main()
