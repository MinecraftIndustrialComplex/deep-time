#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Whether two pool runs made the same world: their `state.json` per-file hashes and `rec.json`.

  python3 -I tools/review/uplift/samestate.py <run folder A> <run folder B> [...]

Each argument after the first is compared with the first. Prints the files that differ, the
files only one run has, and whether `rec.json` is the same (the measurement traces' check:
a trace's world should be its base's bit for bit, its globals aside).
"""
import hashlib
import json
import os
import sys


def files(run):
    with open(os.path.join(run, "state.json")) as f:
        return json.load(f)["files"]


def rec(run):
    p = os.path.join(run, "rec.json")
    if not os.path.exists(p):
        return None
    with open(p, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def main(argv):
    if len(argv) < 2:
        raise SystemExit(__doc__)
    a = argv[0]
    fa, ra = files(a), rec(a)
    for b in argv[1:]:
        fb, rb = files(b), rec(b)
        diff = sorted(k for k in fa.keys() & fb.keys() if fa[k] != fb[k])
        only_a = sorted(fa.keys() - fb.keys())
        only_b = sorted(fb.keys() - fa.keys())
        same = len(fa.keys() & fb.keys()) - len(diff)
        print(f"{a}\n  vs {b}")
        print(f"  same files {same}, differing {len(diff)}: {', '.join(diff) or '-'}")
        print(f"  only in the first: {', '.join(only_a) or '-'}")
        print(f"  only in the second: {', '.join(only_b) or '-'}")
        print(f"  rec.json {'identical' if ra == rb and ra is not None else 'DIFFERS'}")


if __name__ == "__main__":
    main(sys.argv[1:])
