#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Candidate places for the Final world map's review shots: volcanoes, fjords, river mouths.

    python3 tools/workbench/find_places.py PORT RUN [volcanoes|fjords|mouths|all]

Asks a running `deeptime watch` on 127.0.0.1:PORT (a private one, never Ben's on 8642) about the
finished run RUN and prints `?run=RUN&world=<layer>/<x>/<z>/<blocks per px>` links to open in the
page. Nothing is claimed until you have looked: open each link and check the shot shows the thing.

- volcanoes: the run's own events (`deeptime:volcano_*` with a standing height) with their block.
- fjords: a lattice of `world/sample` calls (step 24 blocks) over 45..80 degrees of latitude for sea
  points more than 8 blocks below sea level with land at least FJORD_WALL (25) blocks high within FJORD_REACH (36) blocks
  on both sides of an axis: a narrow deep arm between high ground, ranked by the lower side.
- mouths: the river network's largest mouths in the sea (rivers/rivers.dtrv, read from --world).
Standard library only.
"""
import json
import struct
import sys
import urllib.request


def get(url):
    return json.load(urllib.request.urlopen(url))


def volcanoes(base, c):
    ev = get(base + "/state.json").get("events", [])
    out = []
    for e in ev:
        k = e.get("kind", "")
        p = e.get("place") or {}
        if p.get("type") != "point" or not k.startswith("deeptime:volcano_"):
            continue
        prm = e.get("params", {})
        h = prm.get("height_m") or prm.get("tallest_m") or 0
        if k.endswith("stratovolcano") or prm.get("standing", 0) or prm.get("live", 0):
            out.append((h, k, round(p["lon"] / 360 * c - c / 2), round(-p["lat"] / 360 * c), e.get("t_start")))
    for h, k, x, z, t in sorted(out, reverse=True)[:10]:
        print(f"volcano {k[17:]:14} {h:6.0f} m  elev/{x}/{z}/1   sat/{x}/{z}/0.5")
    if not out:
        print("no standing volcano in the run's events")


def sample(base, x, z):
    return get(f"{base}/world/sample?x={x}&z={z}")


def fjords(base, info):
    c, sea = info["circumference"], info["sea_level_y"]
    import os
    step, reach = 24, int(os.environ.get("FJORD_REACH", 36))
    wall = float(os.environ.get("FJORD_WALL", 25))
    found = []
    for hemi in (-1, 1):
        for zi in range(int(45 / 360 * c), int(80 / 360 * c), step):
            z = hemi * zi
            for x in range(-c // 2, c // 2, step):
                s = sample(base, x, z)
                if s["surface"] != "sea" or s["terrain_y"] > sea - 8:
                    continue
                for dx, dz in ((reach, 0), (0, reach)):
                    a, b = sample(base, x - dx, z - dz), sample(base, x + dx, z + dz)
                    low = min(a["terrain_y"], b["terrain_y"])
                    if a["surface"] != "sea" and b["surface"] != "sea" and low > sea + wall:
                        found.append((low - sea, x, z))
    found.sort(reverse=True)
    seen = []
    for h, x, z in found:
        if all(abs(x - sx) + abs(z - sz) > 200 for sx, sz in seen):
            seen.append((x, z))
            print(f"fjord candidate walls {h:5.0f} blocks  elev/{x}/{z}/0.5   sat/{x}/{z}/2")
        if len(seen) >= 10:
            break
    if not found:
        print("no fjord-like arm found on the lattice")


def mouths(world):
    b = open(world + "/rivers/rivers.dtrv", "rb").read()
    c, _t, _tx, _tz, nv, nk, _e = struct.unpack_from("<7I", b, 16)
    # Layout 2: 52 fixed bytes, 20-byte chains (FORMAT.md "Rivers").
    secs = [struct.unpack_from("<QII", b, 68 + 16 * i) for i in range(4)]
    out = []
    for k in range(nk):
        first, count, mouth = struct.unpack_from("<IIB", b, secs[1][0] + 20 * k)
        x, z, _w, hw, _d, q = struct.unpack_from("<6f", b, secs[0][0] + 24 * (first + count - 1))
        if mouth == 0:
            out.append((q, x, z, hw))
    for q, x, z, hw in sorted(out, reverse=True)[:8]:
        print(f"river mouth q {q:6.0f} m3/s half-width {hw:4.1f}  sat/{x:.0f}/{z:.0f}/2   water/{x:.0f}/{z:.0f}/1")


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    port, run = sys.argv[1], sys.argv[2]
    what = sys.argv[3] if len(sys.argv) > 3 else "all"
    base = f"http://127.0.0.1:{port}/api/runs/{run}"
    info = get(base + "/world.json")
    if not info.get("available"):
        print("no world:", info.get("reason"))
        return 1
    if what in ("volcanoes", "all"):
        volcanoes(base, info["circumference"])
    if what in ("fjords", "all"):
        fjords(base, info)
    if what in ("mouths", "all"):
        import os
        w = os.environ.get("DEEPTIME_WORLD")
        if w:
            mouths(w)
        elif what == "mouths":
            print("set DEEPTIME_WORLD to the run's world folder")
    return 0


if __name__ == "__main__":
    sys.exit(main())
