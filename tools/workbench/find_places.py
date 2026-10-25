#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Candidate places for the Final world map's review shots: volcanoes and river mouths.

    DEEPTIME_WORLD=<the run's world folder> python3 tools/workbench/find_places.py PORT RUN [volcanoes|mouths|all]

Asks a running `deeptime watch` on 127.0.0.1:PORT (a private one, never Ben's on 8642) about the
finished run RUN and prints `?run=RUN&world=<layer>/<x>/<z>/<blocks per px>` links to open in the
page. Nothing is claimed until you have looked: open each link and check the shot shows the thing.

- volcanoes: the run's own events (`deeptime:volcano_*` with a standing height). An event holds a
  latitude and longitude only, and a cube world has no closed form from those to a block (its map is
  a table and the manifest carries the turn), so the block comes from the Rust evaluator:
  `deeptime inspect-column --world-data $DEEPTIME_WORLD --points - --latlon` (the binary is
  $DEEPTIME_BIN, else `deeptime` on the PATH).
- mouths: the river network's largest mouths in the sea (rivers/rivers.dtrv, read from --world).

The fjord scan this tool once had is gone with the plate carree: it walked a lattice over a latitude
band, and a latitude band is no z range on the cube's net (cube plan 7.4).
Standard library only.
"""
import json
import os
import shutil
import struct
import subprocess
import sys
import urllib.request


def get(url):
    return json.load(urllib.request.urlopen(url))


def blocks_of(world, points):
    """The blocks (x, z) of (latitude, longitude) points on the cube world at `world`, one call to the
    Rust evaluator (WorldProjection::block_of, the game's own placement)."""
    exe = os.environ.get("DEEPTIME_BIN") or shutil.which("deeptime")
    if not exe:
        sys.exit("no deeptime binary: set DEEPTIME_BIN (or put `deeptime` on the PATH)")
    text = "".join(f"{lat!r} {lon!r}\n" for lat, lon in points)
    p = subprocess.run([exe, "inspect-column", "--world-data", world, "--points", "-", "--latlon"],
                       input=text.encode(), capture_output=True, timeout=120)
    if p.returncode != 0:
        sys.exit("deeptime inspect-column: " + p.stderr.decode(errors="replace").strip()[-400:])
    cols = [json.loads(line) for line in p.stdout.decode().splitlines() if line.strip()]
    if len(cols) != len(points):
        sys.exit(f"deeptime inspect-column placed {len(cols)} of {len(points)} points")
    return [(int(c["x"]), int(c["z"])) for c in cols]


def volcanoes(base, world):
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
            out.append((h, k, p["lat"], p["lon"], e.get("t_start")))
    out = sorted(out, reverse=True)[:10]
    for (h, k, _lat, _lon, _t), (x, z) in zip(out, blocks_of(world, [(o[2], o[3]) for o in out]) if out else []):
        print(f"volcano {k[17:]:14} {h:6.0f} m  elev/{x}/{z}/1   sat/{x}/{z}/0.5")
    if not out:
        print("no standing volcano in the run's events")


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
    w = os.environ.get("DEEPTIME_WORLD")
    if what in ("volcanoes", "all"):
        if w:
            volcanoes(base, w)
        else:
            print("set DEEPTIME_WORLD to the run's world folder (a volcano's block comes from its map)")
    if what in ("mouths", "all"):
        if w:
            mouths(w)
        elif what == "mouths":
            print("set DEEPTIME_WORLD to the run's world folder")
    return 0


if __name__ == "__main__":
    sys.exit(main())
