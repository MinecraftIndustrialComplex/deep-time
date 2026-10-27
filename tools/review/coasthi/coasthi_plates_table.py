#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""coasthi_plates.py's output over an arm: coasthi_plates_table.py <plates.jsonl>. Each share of the
qualified coast as the median over the runs and pooled (weighted by each run's M5, so a share of
all the arm's coast with high ground); the open collisions' sizes pooled."""
import json
import sys

import numpy as np

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from coasthi_table import flatten  # noqa: E402


def main():
    rows = [json.loads(line) for line in open(sys.argv[1]) if line.strip()]
    flat = [flatten({k: v for k, v in r.items() if k != "open"}) for r in rows]
    m5 = np.array([r["M5"] for r in rows])
    keys = []
    for r in flat:
        for k in r:
            if k not in keys:
                keys.append(k)
    print(f"{len(rows)} runs; M5 median {np.median(m5):.4f}; columns: median over runs, pooled (weighted by M5)")
    for k in keys:
        v = np.array([np.nan if r.get(k) is None else float(r[k]) for r in flat])
        if np.all(np.isnan(v)):
            continue
        ok = np.isfinite(v)
        pooled = float((v[ok] * m5[ok]).sum() / max(m5[ok].sum(), 1e-12))
        print(f"{k:40s} {np.nanmedian(v):9.4f} {pooled:9.4f}")
    opens = [o for r in rows for o in r["open"]]
    print(f"\nopen collisions: {len(opens)} in {len(rows)} runs")
    oc = np.array([o["over_cont_Mkm2"] for o in opens])
    uc = np.array([o["under_cont_Mkm2"] for o in opens])
    fk = np.array([o["front_km"] for o in opens])
    qo = np.array([o["q_pts_over"] for o in opens])
    for nm, v in (("over cont Mkm2", oc), ("under cont Mkm2", uc), ("front km", fk)):
        print(f"  {nm:16s} p10 {np.percentile(v, 10):8.2f} p50 {np.median(v):8.2f} p90 {np.percentile(v, 90):8.2f}")
    small = uc < 3.0
    print(f"  incoming continental crust under 3 Mkm2: {small.mean():.3f} of the open collisions,"
          f" {fk[small].sum() / fk.sum():.3f} of their front length")
    print(f"  incoming smaller than overriding: {(uc < oc).mean():.3f}; front-weighted {fk[uc < oc].sum() / fk.sum():.3f}")
    from collections import Counter
    print("  overriding plate origin:", dict(Counter(o["over_origin"] for o in opens)))


if __name__ == "__main__":
    main()
