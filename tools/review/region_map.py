#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-10 review bundle: the geologic provinces coloured by the TerraBlender region they were given,
beside the biomes' namespaces over the same grid.

Input: `--map FILE` is the `deeptime-probe/regionmap.json` that `/deeptime probe regions <step>`
writes (one sample per `step` blocks over the whole planet: the baked province id, the region index
the world's table gives it, and the biome the level's biome source places at the surface).

Writes OUT/index.html and OUT/{regions,namespaces}.png (one pixel per sample, times --scale), and a
legend with each region's provinces and land share. No third-party Python packages.

    python3 tools/review/region_map.py --map smoke/terrablender/server/deeptime-probe/regionmap.json \
        --out review/out/p1-10
"""
import argparse
import base64
import colorsys
import hashlib
import html
import json
import struct
import zlib
from pathlib import Path


def png(w, h, rgb):
    raw = b"".join(b"\x00" + bytes(rgb[y * w * 3:(y + 1) * w * 3]) for y in range(h))

    def chunk(tag, data):
        c = struct.pack(">I", len(data)) + tag + data
        return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b""))


def i16(b64):
    raw = base64.b64decode(b64)
    return list(struct.unpack("<%dh" % (len(raw) // 2), raw))


def u16(b64):
    raw = base64.b64decode(b64)
    return list(struct.unpack("<%dH" % (len(raw) // 2), raw))


def name_colour(name):
    if name in ("minecraft:overworld", "minecraft"):
        return (150, 160, 150)
    h = int(hashlib.sha1(name.encode()).hexdigest()[:6], 16) / 0xFFFFFF
    r, g, b = colorsys.hsv_to_rgb(h, 0.65, 0.9)
    return (int(r * 255), int(g * 255), int(b * 255))


def scale_up(w, h, rgb, k):
    if k == 1:
        return w, h, rgb
    out = bytearray()
    for y in range(h):
        row = bytearray()
        for x in range(w):
            row += bytes(rgb[(y * w + x) * 3:(y * w + x) * 3 + 3]) * k
        out += row * k
    return w * k, h * k, out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--map", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--scale", type=int, default=2)
    a = ap.parse_args()
    m = json.load(open(a.map))
    w, h = m["w"], m["h"]
    prov, reg, bio = i16(m["province_i16le"]), i16(m["region_i16le"]), u16(m["biome_u16le"])
    names = m["region_names"]
    palette = m["palette"]
    # TerraBlender's own index order is not the table's: the probe wrote the index, the names are the
    # table's in its order. Colour by index, label by what the table calls it when the lists agree.
    reg_ids = sorted(set(reg))
    reg_label = {i: (names[i] if 0 <= i < len(names) else "region %d" % i) for i in reg_ids}
    regions_img = bytearray(w * h * 3)
    ns_img = bytearray(w * h * 3)
    ns_colour = {"minecraft": (110, 120, 110), "biomesoplenty": (240, 150, 40), "terralith": (60, 150, 230)}
    for i in range(w * h):
        c = name_colour(reg_label.get(reg[i], "?")) if reg[i] >= 0 else (40, 40, 40)
        n = palette[bio[i]].split(":")[0] if bio[i] < len(palette) else "?"
        d = ns_colour.get(n, (200, 60, 200))
        # Province outlines: a sample whose right or lower neighbour is another province.
        edge = (i % w + 1 < w and prov[i] != prov[i + 1]) or (i // w + 1 < h and prov[i] != prov[i + w])
        if edge:
            c = tuple(int(x * 0.45) for x in c)
            d = tuple(int(x * 0.45) for x in d)
        regions_img[i * 3:i * 3 + 3] = bytes(c)
        ns_img[i * 3:i * 3 + 3] = bytes(d)
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    for name, img in (("regions", regions_img), ("namespaces", ns_img)):
        sw, sh, data = scale_up(w, h, img, a.scale)
        (out / (name + ".png")).write_bytes(png(sw, sh, data))
    census = m["census"]
    by_region = {}
    for p, e in census["by_province"].items():
        by_region.setdefault(e["region"], []).append(int(p))
    rows = []
    for r in sorted(by_region):
        c = reg_label.get(r, "region %d" % r)
        sw = "#%02x%02x%02x" % name_colour(c)
        rows.append("<tr><td style='background:%s'>&nbsp;&nbsp;&nbsp;</td><td>%s</td><td>%d provinces</td></tr>"
                    % (sw, html.escape(c), len(by_region[r])))
    page = """<!doctype html><meta charset=utf-8><title>P1-10 regions follow provinces</title>
<style>body{font:14px system-ui;margin:16px;max-width:1200px}img{max-width:100%%;image-rendering:pixelated;border:1px solid #888}
td{padding:2px 8px}</style>
<h1>TerraBlender regions follow geologic provinces</h1>
<p>%s: %d provinces, sampled every %d blocks. Left: each province in the colour of the TerraBlender region it was given
(dark lines are province borders). Right: the namespace of the biome the level places at the surface
(grey vanilla, orange Biomes O' Plenty): a mod biome appears only inside the provinces of its mod's regions.</p>
<table>%s</table>
<h2>Regions</h2><img src="regions.png">
<h2>Biome namespaces</h2><img src="namespaces.png">
<p>Land biomes by namespace: %s</p>
""" % (html.escape(census["level"]), census["provinces"], m["step"], "".join(rows),
       html.escape(json.dumps(census["land_biomes_by_namespace"])))
    (out / "index.html").write_text(page)
    print("wrote", out / "index.html")


if __name__ == "__main__":
    main()
