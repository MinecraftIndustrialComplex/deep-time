#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Arms against a base on the M5 diagnosis's fields (VM): m5compare.py <base fields> <name>=<fields> ...

Every M5 and M4 here is the version 1 rule (m5fields.py). The base keeps only the runs whose
(start, seed) the first arm has, so a 24-run base reads against an 8-run arm on matched seeds.
Prints medians, and Mann-Whitney p against the base, of M5, M4, each class's part of M5 by the
nearest qualifying high ground and of M4 (absolute shares of the land), and M5 without the young
collision class."""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from m5fields import CLASSES  # noqa: E402
from scipy.stats import mannwhitneyu  # noqa: E402
import numpy as np  # noqa: E402


def load(p):
    return [json.loads(x) for x in Path(p).read_text().splitlines() if x.strip()]


def main():
    base = load(sys.argv[1])
    arms = [(a.split("=", 1)[0], load(a.split("=", 1)[1])) for a in sys.argv[2:]]
    keep = {(r["start"], r["seed"]) for r in arms[0][1]} if arms else None
    base = [r for r in base if keep is None or (r["start"], r["seed"]) in keep]
    cols = [("base", base)] + arms
    rows = [("M5", lambda r: r["M5"]), ("M4", lambda r: r["M4"])]
    rows += [(f"M5 by {k}", lambda r, k=k: r["m5_by_nearest"][k]) for k in CLASSES]
    rows += [("M5 without coll_young", lambda r: r["m5_without"]["coll_young"])]
    rows += [(f"M4 by {k}", lambda r, k=k: r["m4_by"][k]) for k in CLASSES]
    print("version 1 rule; runs " + ", ".join(f"{n} {len(rs)}" for n, rs in cols))
    print(f"{'':<24}" + "".join(f"{n:>22}" for n, _ in cols))
    for name, get in rows:
        b = [get(r) for r in base]
        cells = []
        for n, rs in cols:
            v = [get(r) for r in rs]
            m = float(np.median(v)) if v else float("nan")
            if n == "base":
                cells.append(f"{m:>22.4f}")
            else:
                try:
                    p = mannwhitneyu(b, v).pvalue
                except ValueError:
                    p = float("nan")
                cells.append(f"{m:>12.4f} (p {p:.2f})")
        print(f"{name:<24}" + "".join(cells))


if __name__ == "__main__":
    main()
