#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Sea-floor spikes study (2026-10-20): where two calibration runs' whole-world heightmaps differ.

    compare.py <study dir> <base tag> <tag> ...

Reads <study dir>/cal-<tag>/hm.npy (calib.sh) and prints, for each tag against the base: the
share of samples below sea level, the mean change of H on land and at sea, and how many samples
went from land to sea and back, with the latitude band they sit in.
"""

import json
import sys

import numpy as np


def main():
    study, tags = sys.argv[1], sys.argv[2:]
    hs = {}
    for t in tags:
        a = np.load(f"{study}/cal-{t}/hm.npy", mmap_mode="r")
        meta = json.load(open(f"{study}/cal-{t}/hm.npy.json"))
        ch = {n: i for i, n in enumerate(meta["channels"])}
        H = np.asarray(a[:, :, ch["surface"]], dtype=np.float32)
        hs[t] = H
        info = {k: v for k, v in meta.items() if k != "channels"}
        print(t, json.dumps(info))
        print(f"  H<63 {np.mean(H < 63):.4f}  min {H.min():.1f} max {H.max():.1f}")
    b = tags[0]
    rows = hs[b].shape[0]
    for t in tags[1:]:
        d = hs[t] - hs[b]
        land = hs[b] >= 63
        l2s = land & (hs[t] < 63)
        s2l = ~land & (hs[t] >= 63)
        print(f"{t} - {b}: mean {d.mean():.3f} blocks, |d|>1 on {np.mean(np.abs(d) > 1):.4f}, "
              f"land->sea {l2s.mean():.4f}, sea->land {s2l.mean():.4f}")
        print(f"  on land: mean {d[land].mean():.3f}, p1/p50/p99 {np.percentile(d[land], [1, 50, 99])}")
        print(f"  at sea:  mean {d[~land].mean():.3f}, p1/p50/p99 {np.percentile(d[~land], [1, 50, 99])}")
        # How far from the sea the land changed (blocks; the map's samples are `step` apart).
        from scipy import ndimage
        dist = ndimage.distance_transform_edt(land) * json.load(open(f"{study}/cal-{b}/hm.npy.json"))["step"]
        big = land & (np.abs(d) > 2)
        for lo, hi in ((0, 16), (16, 64), (64, 256), (256, 1e9)):
            ring = land & (dist > lo) & (dist <= hi)
            print(f"  land {lo}-{hi:g} blocks from the sea: {ring.sum() / land.sum():.3f} of the land, "
                  f"{big[ring].sum() / max(big.sum(), 1):.3f} of the |d|>2 samples, |d|>2 on {big[ring].mean():.4f} of it")
        band = np.arange(rows) * 10 // rows
        for k in range(10):
            m = band == k
            lm = land[m]
            print(f"  rows {k}/10: land->sea {l2s[m].mean():.4f} sea->land {s2l[m].mean():.4f} "
                  f"mean d at sea {d[m][~lm].mean():.2f}, land share {lm.mean():.3f}, "
                  f"on land mean |d| {np.abs(d[m][lm]).mean() if lm.any() else 0:.3f} "
                  f"|d|>2 {np.mean(np.abs(d[m][lm]) > 2) if lm.any() else 0:.4f}")


if __name__ == "__main__":
    main()
