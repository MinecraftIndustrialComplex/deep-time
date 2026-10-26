#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plate check's measures of two pool arms side by side (medians over the runs, and fails).

  python3 -I tools/review/sutureloss/platecmp.py <base arm> <arm> [--pool DIR] [--grep WORD]...

Reads each run's plate.json (`deeptime plate-check` on that run) and prints, per measure, the
median of each arm's runs, the target, and how many runs fail it.
"""
import json
import statistics
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from collseries import arm_runs  # noqa: E402


def read(pool, arm):
    _, runs = arm_runs(pool, arm)
    out = {}
    for _, path in runs:
        try:
            d = json.load(open(f"{path}/plate.json"))
        except OSError:
            continue
        for m in d["measures"]:
            v = m.get("value")
            e = out.setdefault(m["key"], {"label": m["label"], "lo": m.get("lo"), "hi": m.get("hi"), "v": [], "fail": 0})
            if v is not None:
                e["v"].append(v)
            e["fail"] += m.get("status") == "FAIL"
    return out


def main():
    args = sys.argv[1:]
    pool = "/data/ban/deeptime-runs/pool"
    greps = []
    while "--pool" in args or "--grep" in args:
        for flag in ("--pool", "--grep"):
            if flag in args:
                i = args.index(flag)
                if flag == "--pool":
                    pool = args[i + 1]
                else:
                    greps.append(args[i + 1].lower())
                del args[i:i + 2]
    a, b = read(pool, args[0]), read(pool, args[1])
    print(f"{'measure':70s} {'target':>14s} {args[0]:>12s} {'fails':>5s} {args[1]:>12s} {'fails':>5s}")
    for k, e in a.items():
        if greps and not any(g in (k + e["label"]).lower() for g in greps):
            continue
        f = b.get(k, {"v": [], "fail": 0})
        med = lambda xs: statistics.median(xs) if xs else float("nan")
        tgt = f"{e['lo'] if e['lo'] is not None else ''}-{e['hi'] if e['hi'] is not None else ''}"
        print(f"{e['label'][:70]:70s} {tgt:>14s} {med(e['v']):12.3f} {e['fail']:5d} {med(f['v']):12.3f} {f['fail']:5d}")


if __name__ == "__main__":
    main()
