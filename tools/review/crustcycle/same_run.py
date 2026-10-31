#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Whether a measurement-only branch's runs are bit for bit its base's (no simulation):

    same_run.py <base arm folder> <trace arm folder> [field ...]

For each run folder the two arm folders share (start-seed), compares the hashes of the run's saved state (state.json,
every file but fields/globals.json, which a new global changes) and the step records (rec.json) with the named
front_bands fields taken out of every step (the fields the trace adds). Prints one line a run: the state files that
differ and whether the records match.
"""
import json
import sys
from pathlib import Path


def leaves(x, p="", out=None):
    out = {} if out is None else out
    if isinstance(x, dict):
        for k, v in x.items():
            leaves(v, f"{p}/{k}", out)
    elif isinstance(x, list):
        for i, v in enumerate(x):
            leaves(v, f"{p}[{i}]", out)
    else:
        out[p] = x
    return out


def record(run, drop):
    rec = json.loads((run / "rec.json").read_text())
    for s in rec.get("steps") or []:
        fb = s.get("front_bands")
        if isinstance(fb, dict):
            for k in drop:
                fb.pop(k, None)
    return rec


def main(argv):
    base, trace, drop = Path(argv[0]), Path(argv[1]), argv[2:]
    names = sorted(p.name for p in trace.iterdir() if (p / "rec.json").is_file() and (base / p.name / "rec.json").is_file())
    if not names:
        print("no run in both folders")
        return 1
    bad = 0
    for name in names:
        a, b = base / name, trace / name
        fa = leaves(json.loads((a / "state.json").read_text()))
        fb = leaves(json.loads((b / "state.json").read_text()))
        diff = sorted(k for k in set(fa) | set(fb) if fa.get(k) != fb.get(k) and "globals.json" not in k)
        ra, rb = record(a, drop), record(b, drop)
        same_rec = ra == rb
        bad += bool(diff) or not same_rec
        print(f"{name}: state files differing {len(diff)} {diff[:6]}; records equal without {drop}: {same_rec}")
        if not same_rec:
            la, lb = leaves(ra), leaves(rb)
            keys = sorted(k for k in set(la) | set(lb) if la.get(k) != lb.get(k))
            print(f"  record leaves differing {len(keys)}: {keys[:6]}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
