# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau lane: how much of the high ground drains to a closed depression at a run's end (VM, cached runs).

Usage: python3 -I closed.py <arm folder> [<arm folder> ...]

Internal drainage needs a depression for a basin's water to end in. On the final L1 fields (1024 x 328, x wraps),
for the surface's elevation (`surface/elevation_m`, after the last step's erosion and fill) and for the plate model's
(`plates/elevation_m`, its Airy heights interpolated, before the surface's offset): a priority flood from the open
sea (cells at or under the sea level, `climate/sea_level_m`), as the surface stage's (`pflood.rs`, ERO-080), then the
steepest descent on the filled surface. A cell drains to a depression when its path reaches a cell the flood raised
by more than 1 m (a lake of the flood's kind). Printed, medians over the runs: the share of the land above 2 km, and
of its flat part (3 x 3 relief under 500 m), that drains to a depression, the share that is the depression's own
floor, the number of such depressions holding high ground, and the largest one's catchment and lake in cells (a cell
is 4.8 x 4.8 km on the map, 23 km2; 66 times that Earth-normalized). The water balance that would close a basin is
not computed (the dump has no precipitation): this is the most the internal-drainage law could reach at that frame.
"""
import heapq
import json
import statistics
import sys
from pathlib import Path

import numpy as np

LAKE_M = 1.0


def field(f, idx, name):
    e = idx[name]
    return np.fromfile(f / e["file"], "<f4").reshape(e["h"], e["w"]).astype(np.float64)


def neighbours(h, w):
    out = []
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            if dy == 0 and dx == 0:
                continue
            out.append((dy, dx))
    return out


def flood(z, sea):
    h, w = z.shape
    filled = np.full(z.shape, np.inf)
    seen = np.zeros(z.shape, bool)
    heap = []
    sea_m = z <= sea
    for j, i in zip(*np.nonzero(sea_m)):
        filled[j, i] = sea
        seen[j, i] = True
    # Seed the flood from the sea cells that touch land.
    nb = neighbours(h, w)
    for j, i in zip(*np.nonzero(sea_m)):
        for dy, dx in nb:
            jj, ii = j + dy, (i + dx) % w
            if 0 <= jj < h and not seen[jj, ii]:
                heap.append((max(z[jj, ii], sea), jj, ii))
                seen[jj, ii] = True
    heapq.heapify(heap)
    while heap:
        f, j, i = heapq.heappop(heap)
        filled[j, i] = f
        for dy, dx in nb:
            jj, ii = j + dy, (i + dx) % w
            if 0 <= jj < h and not seen[jj, ii]:
                seen[jj, ii] = True
                heapq.heappush(heap, (max(z[jj, ii], np.nextafter(f, np.inf)), jj, ii))
    # Land no sea reaches (none on these planets) keeps its own height.
    filled[~seen] = z[~seen]
    return filled


def drains_to_lake(z, filled, sea):
    """Per cell: the id of the depression its steepest-descent path on the filled surface ends in (-1: the sea)."""
    h, w = z.shape
    lake = ((filled - z) > LAKE_M) & (z > sea)
    # Lakes: connected lake cells at one fill level form one depression (label by flood fill on equal level).
    lid = np.full(z.shape, -1, np.int64)
    n = 0
    for j, i in zip(*np.nonzero(lake)):
        if lid[j, i] >= 0:
            continue
        stack = [(j, i)]
        lid[j, i] = n
        lvl = filled[j, i]
        while stack:
            a, b = stack.pop()
            for dy, dx in neighbours(h, w):
                aa, bb = a + dy, (b + dx) % w
                if 0 <= aa < h and lake[aa, bb] and lid[aa, bb] < 0 and abs(filled[aa, bb] - lvl) < 1.0:
                    lid[aa, bb] = n
                    stack.append((aa, bb))
        n += 1
    # Receivers on the filled surface (steepest descent, diagonals at sqrt 2).
    best = np.zeros(z.shape)
    rcv = np.arange(h * w).reshape(h, w)
    for dy, dx in neighbours(h, w):
        sh = np.roll(np.roll(filled, -dy, axis=0), -dx, axis=1)
        if dy == 1:
            sh[-1, :] = np.inf
        if dy == -1:
            sh[0, :] = np.inf
        s = (filled - sh) / (np.sqrt(2.0) if dx and dy else 1.0)
        jj = (np.arange(h)[:, None] + dy) % h
        ii = (np.arange(w)[None, :] + dx) % w
        idx = jj * w + ii
        better = s > best
        best = np.where(better, s, best)
        rcv = np.where(better, idx, rcv)
    flat = filled.ravel()
    order = np.argsort(flat, kind="stable")
    target = lid.ravel().copy()
    r = rcv.ravel()
    for c in order:
        if target[c] < 0 and r[c] != c:
            target[c] = target[r[c]]
    return target.reshape(h, w), lid, n


def run(r):
    f = r / "fields"
    idx = json.load(open(f / "index.json"))
    g = json.load(open(f / "globals.json"))
    g = g.get("globals", g)
    sea = float(g.get("climate/sea_level_m", 0.0))
    out = {}
    for tag, name in (("surface", "surface/elevation_m"), ("plates", "plates/elevation_m")):
        if name not in idx:
            continue
        z = field(f, idx, name)
        zs = z - sea
        hi = zs > 2000.0
        pad = np.pad(zs, 1, mode="edge")
        win = np.stack([pad[1 + dy:pad.shape[0] - 1 + dy, 1 + dx:pad.shape[1] - 1 + dx]
                        for dy in (-1, 0, 1) for dx in (-1, 0, 1)])
        flat = hi & ((win.max(0) - win.min(0)) < 500.0)
        filled = flood(z, sea)
        target, lid, n = drains_to_lake(z, filled, sea)
        closed = target >= 0
        for label, m in (("hi", hi), ("flat", flat)):
            k = int(m.sum())
            if k == 0:
                continue
            out[f"{tag}_{label}_n"] = k
            out[f"{tag}_{label}_to_depression"] = float((closed & m).sum() / k)
            out[f"{tag}_{label}_in_depression"] = float(((lid >= 0) & m).sum() / k)
        ids = np.unique(target[closed & hi])
        out[f"{tag}_depressions_with_high_ground"] = len(ids)
        if len(ids):
            sizes = [(int((target == i).sum()), int((lid == i).sum())) for i in ids]
            sizes.sort(reverse=True)
            out[f"{tag}_largest_catchment_cells"] = sizes[0][0]
            out[f"{tag}_largest_lake_cells"] = sizes[0][1]
        out[f"{tag}_land_to_depression"] = float((closed & (zs > 0)).sum() / max(int((zs > 0).sum()), 1))
    return out


def main():
    for arm in sys.argv[1:]:
        rows = []
        for r in sorted(Path(arm).iterdir()):
            if (r / "fields" / "index.json").exists():
                rows.append(run(r))
        print(f"== closed {arm} ({len(rows)} runs)")
        for key in sorted({k for r in rows for k in r}):
            v = [r[key] for r in rows if key in r]
            print(f"  median {key:42s} {statistics.median(v):10.4f}  (n {len(v)}, min {min(v):.4f}, max {max(v):.4f})")


if __name__ == "__main__":
    main()
