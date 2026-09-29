#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""One 16 x 16 texture per block id, for review pages (plan Q10's "without EV / with EV" tiles).

    python3 tools/review/block-tiles.py --jars <folder of jars> --vanilla <client-extra jar> \
        --out <folder> <block id or JSON file>...

Block ids come from the arguments; a JSON file contributes every "ns:path" string in it (the ore-hosts
output of deeptime-inspect). For each block: its blockstate's first model (variant "" or the first
variant; multipart: the first part), the model's parent chain, then the first of the textures all,
top, end, up, side, particle, texture, or any; `#` references resolved; the first square frame of that
PNG is written to <out>/<ns>__<path>.png. Jars later in name order override earlier ones and every
jar overrides the vanilla jar, as resource packs do. Blocks it cannot resolve are listed, not fatal.
"""
import argparse
import io
import json
import os
import re
import struct
import sys
import zipfile
import zlib

ap = argparse.ArgumentParser()
ap.add_argument("--jars", required=True)
ap.add_argument("--vanilla", required=True)
ap.add_argument("--out", required=True)
ap.add_argument("blocks", nargs="+")
a = ap.parse_args()

index = {}   # "assets/..." -> ZipFile
zips = [a.vanilla] + sorted(os.path.join(a.jars, f) for f in os.listdir(a.jars) if f.endswith((".jar", ".zip")))
for path in zips:
    try:
        z = zipfile.ZipFile(os.path.realpath(path))
    except (zipfile.BadZipFile, OSError):
        continue
    for n in z.namelist():
        if n.startswith("assets/"):
            index[n] = z


def read(p):
    z = index.get(p)
    return z.read(p) if z else None


def rl(s, kind):
    ns, _, path = s.partition(":") if ":" in s else ("minecraft", ":", s)
    return f"assets/{ns}/{kind}/{path}"


def first_model(block):
    raw = read(rl(block, "blockstates") + ".json")
    if raw is None:
        return None
    d = json.loads(raw)
    if "variants" in d:
        v = d["variants"].get("", next(iter(d["variants"].values()), None))
    else:
        v = next((p["apply"] for p in d.get("multipart", []) if "when" not in p), None) or \
            next((p["apply"] for p in d.get("multipart", [])), None)
    if isinstance(v, list):
        v = v[0] if v else None
    return v.get("model") if v else None


def texture_of(model):
    textures, seen = {}, 0
    while model and seen < 16:
        seen += 1
        raw = read(rl(model, "models") + ".json")
        if raw is None:
            break
        d = json.loads(raw)
        for k, v in d.get("textures", {}).items():
            textures.setdefault(k, v)
        model = d.get("parent")
    for key in ("all", "top", "end", "up", "side", "particle", "texture"):
        if key in textures:
            break
    else:
        key = next(iter(textures), None)
    t = textures.get(key) if key else None
    for _ in range(8):
        if t and t.startswith("#"):
            t = textures.get(t[1:])
    return t


def png_frame(data):
    """The first square frame of a PNG (animated textures are vertical strips), re-encoded."""
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    pos, chunks = 8, []
    while pos < len(data):
        n, typ = struct.unpack(">I4s", data[pos:pos + 8])
        chunks.append((typ, data[pos + 8:pos + 8 + n]))
        pos += 12 + n
    ihdr = chunks[0][1]
    w, h = struct.unpack(">II", ihdr[:8])
    if h <= w:
        return data
    bitdepth, ctype = ihdr[8], ihdr[9]
    if ihdr[12] != 0:   # interlaced: rows are not stored in order; give up on this one
        return None
    ch = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[ctype]
    stride = (w * ch * bitdepth + 7) // 8
    raw = zlib.decompress(b"".join(c for t, c in chunks if t == b"IDAT"))
    # Each row's filter refers only to the row above, so the first w rows stand on their own.
    keep = raw[: (stride + 1) * w]

    def chunk(t, c):
        return struct.pack(">I", len(c)) + t + c + struct.pack(">I", zlib.crc32(t + c) & 0xffffffff)
    new_ihdr = struct.pack(">II", w, w) + ihdr[8:]
    out = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", new_ihdr)
    for t, c in chunks[1:]:
        if t in (b"PLTE", b"tRNS"):
            out += chunk(t, c)
    return out + chunk(b"IDAT", zlib.compress(keep)) + chunk(b"IEND", b"")


blocks = []
for arg in a.blocks:
    if os.path.isfile(arg):
        blocks += re.findall(r'"([a-z0-9_.-]+:[a-z0-9_./-]+)"', open(arg).read())
    else:
        blocks.append(arg)
os.makedirs(a.out, exist_ok=True)
done, missing = 0, []
for b in sorted(set(blocks)):
    if b.startswith(("c:", "deeptime:rock")) or "/" in b.split(":")[1]:
        continue
    model = first_model(b)
    tex = texture_of(model) if model else None
    data = read(rl(tex, "textures") + ".png") if tex else None
    frame = png_frame(data) if data else None
    if frame is None:
        missing.append(b)
        continue
    open(os.path.join(a.out, b.replace(":", "__") + ".png"), "wb").write(frame)
    done += 1
print(f"block-tiles: {done} tiles in {a.out}; unresolved: {len(missing)}"
      + (f" ({', '.join(missing[:12])}{' …' if len(missing) > 12 else ''})" if missing else ""))
