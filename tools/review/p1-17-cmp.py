#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Compare two Deep Time world-data folders (P1-17: a speed-up must leave the same world).

Every file is compared by SHA-256, except manifest.json, whose JSON is compared without the fields
that record time or the memory accounting (wall times, creation time, and the history's
`reserved*`, `spill*` and `frames_on_disk` figures: when frames go to disk follows the allocator's
count, which another thread schedule changes, while the frames themselves are the same). A differing manifest
prints the paths of the keys that differ.

    tools/review/p1-17-cmp.py <world A> <world B>      exit 0 when they are the same
"""
import hashlib
import json
import os
import sys

DROP = {"millis", "created_utc", "elapsed_ms", "timings", "passes", "detail_ms", "sink_ms",
        "sample_ms", "reserved_peak_bytes", "reserved_base_peak_bytes", "spill_read_seconds",
        "spill_write_seconds"}


def dropped(k):
    return k in DROP or k.startswith("spill") or k.startswith("reserved") or k == "frames_on_disk" \
        or k == "in_memory_bytes"


def strip(v):
    if isinstance(v, dict):
        return {k: strip(x) for k, x in v.items() if not dropped(k)}
    if isinstance(v, list):
        return [strip(x) for x in v]
    if isinstance(v, str) and v.startswith("{") and "millis" in v:
        try:
            return strip(json.loads(v))
        except ValueError:
            return v
    return v


def diff_paths(a, b, path=""):
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            yield from diff_paths(a.get(k), b.get(k), f"{path}.{k}")
    elif isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        for i, (x, y) in enumerate(zip(a, b)):
            yield from diff_paths(x, y, f"{path}[{i}]")
    elif a != b:
        yield f"{path}: {str(a)[:80]} != {str(b)[:80]}"


def files(root):
    out = {}
    for d, _, fs in os.walk(root):
        for f in fs:
            p = os.path.join(d, f)
            out[os.path.relpath(p, root)] = p
    return out


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main():
    a, b = files(sys.argv[1]), files(sys.argv[2])
    bad = 0
    for k in sorted(set(a) | set(b)):
        if k not in a or k not in b:
            print("ONLY IN", "A" if k in a else "B", k)
            bad += 1
            continue
        if os.path.basename(k) == "manifest.json":
            ja, jb = (strip(json.load(open(x))) for x in (a[k], b[k]))
            if ja != jb:
                print("DIFF", k)
                for line in list(diff_paths(ja, jb))[:10]:
                    print("   ", line)
                bad += 1
            continue
        if sha(a[k]) != sha(b[k]):
            print("DIFF", k)
            bad += 1
    print(f"{len(a)} files, {bad} differ")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
