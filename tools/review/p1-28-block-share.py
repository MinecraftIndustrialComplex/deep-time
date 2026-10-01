#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The share of placed rock that is plain stone or deepslate, from generated boxes (P1-28).

    tools/review/p1-28-block-share.py area-0.dtva.gz [area-1.dtva.gz ...] [--top 64]

Reads the `deeptime-inspect export-area` files `tools/workbench/preview-area.sh` writes (the real
generator's blocks, one box each, area.rs documents the format) and counts, over the rock and
crust blocks only (not air, water, soil, plants, ore or lava):

- the blocks within `--top` of each column's terrain top (the highest block that is not air or
  water), and
- every block of the column from the terrain top down,

and prints the share that is `minecraft:stone` or `minecraft:deepslate` (the category `crust`),
with the commonest blocks. Standard library only.
"""
import argparse
import array
import collections
import gzip
import json
import struct
import sys


def load(path):
    d = gzip.open(path).read() if path.endswith(".gz") else open(path, "rb").read()
    header_len = struct.unpack_from("<I", d, 8)[0]
    h = json.loads(d[12:12 + header_len])
    blob = 12 + header_len
    blob += (-blob) % 4

    def section(name):
        s = h["sections"][name]
        a = array.array("H")
        a.frombytes(d[blob + s["offset"]: blob + s["offset"] + s["count"] * 2])
        return a

    return h, section("run_counts"), section("run_values"), section("run_lengths")


def count(path, top):
    h, counts, values, lengths = load(path)
    blocks = h["blocks"]
    cat = [blocks[e[0]]["cat"] for e in h["entries"]]
    ids = [blocks[e[0]]["id"] for e in h["entries"]]
    near, whole = collections.Counter(), collections.Counter()
    k = 0
    for c in range(h["width"] * h["depth"]):
        runs = []
        y = h["y0"]
        for _ in range(counts[c]):
            runs.append((y, lengths[k], values[k]))
            y += lengths[k]
            k += 1
        # The terrain top: the highest block that is neither air nor water.
        ground = [r for r in runs if cat[r[2]] not in ("air", "water")]
        if not ground:
            continue
        top_y = ground[-1][0] + ground[-1][1] - 1
        for start, n, v in runs:
            if cat[v] not in ("rock", "crust"):
                continue
            whole[ids[v]] += n
            hi, lo = min(start + n - 1, top_y), max(start, top_y - top + 1)
            if hi >= lo:
                near[ids[v]] += hi - lo + 1
    return near, whole


def report(label, tally):
    total = sum(tally.values())
    crust = sum(n for b, n in tally.items() if b in ("minecraft:stone", "minecraft:deepslate"))
    top = ", ".join(f"{b.split(':')[1]} {n / total:.1%}" for b, n in tally.most_common(8)) if total else "-"
    print(f"{label:<22}{total:>11,} blocks   stone or deepslate {crust / total if total else 0:.1%}   ({top})")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="+")
    ap.add_argument("--top", type=int, default=64)
    a = ap.parse_args()
    near, whole = collections.Counter(), collections.Counter()
    for f in a.files:
        n, w = count(f, a.top)
        near.update(n)
        whole.update(w)
    print(f"{len(a.files)} box(es):")
    report(f"top {a.top} blocks", near)
    report("whole column", whole)


if __name__ == "__main__":
    sys.exit(main())
