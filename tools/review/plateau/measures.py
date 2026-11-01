# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau lane: the pool's cached measures of several arms side by side (VM, cached runs, no simulation).

Usage: python3 -I measures.py <label>=<arm folder> [...] [-- <measure> ...]

Reads every run's `measures/<measure>.json` (default guard, coastset, collcoast) and prints, per measure and value, the
median over each arm's runs, one column per arm, with each run's value after it for the first arm against the others'
medians when they differ by more than the arms' spread. Values that are not numbers are skipped.
"""
import json
import statistics
import sys
from pathlib import Path


def main():
    args = sys.argv[1:]
    names = ["guard", "coastset", "collcoast"]
    if "--" in args:
        i = args.index("--")
        args, names = args[:i], args[i + 1:]
    arms = [(a.split("=", 1)[0], Path(a.split("=", 1)[1])) for a in args]
    for m in names:
        vals = {}
        for label, folder in arms:
            per = {}
            for r in sorted(folder.iterdir()):
                f = r / "measures" / f"{m}.json"
                if not (r.is_dir() and f.exists()):
                    continue
                v = json.load(open(f)).get("values", {})
                for k, x in v.items():
                    if isinstance(x, (int, float)) and x == x:
                        per.setdefault(k, []).append(float(x))
            vals[label] = per
        keys = sorted({k for per in vals.values() for k in per})
        print(f"== {m}: median over runs, " + " | ".join(f"{label} (n)" for label, _ in arms))
        for k in keys:
            cells = []
            for label, _ in arms:
                v = vals[label].get(k, [])
                cells.append(f"{statistics.median(v):.4g} ({len(v)})" if v else "-")
            print(f"  {k:32s} " + " | ".join(cells))


if __name__ == "__main__":
    main()
