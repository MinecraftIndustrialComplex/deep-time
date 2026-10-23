#!/usr/bin/env python3
"""Where two pool runs' per-step records part: the first record that differs, in which keys, and
how many records differ in each key over the run.

    python3 -I tools/review/pool/rec_firstdiff.py <run folder A> <run folder B>

A run folder is a pool run's (`.../<source key>/<preset>-<length>/<argkey>/<start>-<seed>`); the
records are its `rec.json`, written by `deeptime plate-check --record`. Reads only, so it is a
light job (run it with `mac.sh --no-sync --kind shell:command -- python3 -I ...` on the VM). The
records carry the plates' per-step numbers (land, sea_m, continental, plates, forces, collisions,
...), not the samples' own fields: two runs whose records are equal can still differ elsewhere.
"""
import json
import sys


def load(folder):
    with open(f"{folder}/rec.json") as f:
        return json.load(f)["steps"]


def short(v):
    text = str(v)
    return text if len(text) <= 200 else text[:200] + "..."


def main(argv):
    if len(argv) != 3:
        print(__doc__)
        return 2
    a, b = load(argv[1]), load(argv[2])
    print(f"records: {len(a)} and {len(b)}")
    counts = {}
    first = None
    for i, (x, y) in enumerate(zip(a, b)):
        keys = [k for k in x if x[k] != y.get(k)]
        for k in keys:
            counts[k] = counts.get(k, 0) + 1
        if keys and first is None:
            first = i
            print(f"first differing record {i} (t = {x.get('t')} Myr): {keys}")
            for k in keys[:6]:
                print(f"  {k}: A {short(x[k])} | B {short(y[k])}")
    if first is None:
        print("no differing record")
        return 0
    print("records differing, by key:", dict(sorted(counts.items(), key=lambda kv: -kv[1])))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
