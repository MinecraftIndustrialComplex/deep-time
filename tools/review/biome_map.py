#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-09 review bundle: the Köppen map beside the biome map, per profile.

Inputs, per profile (`--profile NAME=DIR`, repeatable): DIR holds
  biomemap.json          `/deeptime probe biomes <step>` (the whole planet: every column's Köppen
                         class from the world data and the biome the game's biome source places
                         at the terrain surface, plus the census per Köppen family)
  inspect/*.json|png     optional: `deeptime-inspect biomes` over Chunky-generated areas (the
                         biomes the chunks actually store, with the Köppen map of the same area)

Writes OUT/index.html, OUT/<profile>/{koppen,biomes}.png (one pixel per sample), the inspect
images, and OUT/<profile>/cells.bin (Köppen class + biome index per sample, for the page's hover
readout). No third-party Python packages.

    python3 tools/review/biome_map.py --out review/out/p1-09 \
        --profile standalone=review/out/p1-09-data/standalone --profile mic=review/out/p1-09-data/mic
"""
import argparse
import base64
import html
import json
import shutil
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

KOPPEN_COLOURS = [
    "#0000ff", "#0078ff", "#46aafa", "#7fbfff", "#ff0000", "#ff9696", "#f5a500", "#ffdc64",
    "#ffff00", "#c8c800", "#969600", "#96ff96", "#64c864", "#329632", "#c8ff50", "#64ff50",
    "#32c800", "#ff00ff", "#c800c8", "#963296", "#966496", "#aaafff", "#5a78dc", "#4b50b4",
    "#320087", "#00ffff", "#37c8ff", "#007d7d", "#00465f", "#b2b2b2", "#666666", "#0b1d3a",
]
KOPPEN_NAMES = {
    "Af": "tropical rainforest", "Am": "tropical monsoon", "Aw": "tropical savanna (dry winter)",
    "As": "tropical savanna (dry summer)", "BWh": "hot desert", "BWk": "cold desert",
    "BSh": "hot steppe", "BSk": "cold steppe", "Cfb": "oceanic", "Cfa": "humid subtropical",
    "Csa": "Mediterranean, hot summer", "Csb": "Mediterranean, warm summer",
    "Dfc": "subarctic", "Dfb": "humid continental", "ET": "tundra", "EF": "ice cap", "sea": "sea",
}

# The conventional biome-map palette for vanilla (the same table as deeptime-inspect's render.rs).
BIOME_COLOURS = {
    "ocean": "#000070", "deep_ocean": "#000030", "warm_ocean": "#0000ac", "lukewarm_ocean": "#000090",
    "deep_lukewarm_ocean": "#000040", "cold_ocean": "#202070", "deep_cold_ocean": "#202038",
    "frozen_ocean": "#7070d6", "deep_frozen_ocean": "#404090", "plains": "#8db360",
    "sunflower_plains": "#b5db88", "snowy_plains": "#ffffff", "ice_spikes": "#b4dcdc", "desert": "#fa9418",
    "swamp": "#07f9b2", "mangrove_swamp": "#67352b", "forest": "#056621", "flower_forest": "#2d8e49",
    "birch_forest": "#307444", "old_growth_birch_forest": "#589c6c", "dark_forest": "#40511a",
    "taiga": "#0b6659", "old_growth_pine_taiga": "#596651", "old_growth_spruce_taiga": "#818e79",
    "snowy_taiga": "#31554a", "savanna": "#bdb25f", "savanna_plateau": "#a79d64",
    "windswept_savanna": "#e5da87", "windswept_hills": "#606060", "windswept_gravelly_hills": "#888888",
    "windswept_forest": "#507050", "jungle": "#537b09", "sparse_jungle": "#628b17", "bamboo_jungle": "#768e14",
    "badlands": "#d94515", "eroded_badlands": "#ff6d3d", "wooded_badlands": "#b09765", "meadow": "#60a445",
    "cherry_grove": "#ffa0d0", "grove": "#47875a", "snowy_slopes": "#c4c4c4", "frozen_peaks": "#a0a0ff",
    "jagged_peaks": "#dcdcdc", "stony_peaks": "#7b8f74", "river": "#0000ff", "frozen_river": "#a0a0ff",
    "beach": "#fade55", "snowy_beach": "#faf0c0", "stony_shore": "#a2a284", "mushroom_fields": "#ff00ff",
    "dripstone_caves": "#7b6d52", "lush_caves": "#2c6d17",
}


def hexrgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def biome_rgb(bid):
    if bid.startswith("minecraft:") and bid[10:] in BIOME_COLOURS:
        return hexrgb(BIOME_COLOURS[bid[10:]])
    h = 0x811C9DC5
    for b in bid.encode():
        h = ((h ^ b) * 0x01000193) & 0xFFFFFFFF
    ch = lambda s: 64 + ((h >> s) & 0xFF) * 160 // 255
    return (ch(0), ch(8), ch(16))


def write_png(path, w, h, rgb_rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rgb_rows)

    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
    data += chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    path.write_bytes(data)


def profile_section(name, src, out):
    m = json.loads((src / "biomemap.json").read_text())
    w, h = m["w"], m["h"]
    bio = struct.unpack(f"<{w * h}H", base64.b64decode(m["biome_u16le"]))
    kop = base64.b64decode(m["koppen_u8"])
    hgt = struct.unpack(f"<{w * h}h", base64.b64decode(m["height_i16le"]))
    pal = [p["id"] for p in m["palette"]]
    prgb = [biome_rgb(b) for b in pal]
    krgb = [hexrgb(c) for c in KOPPEN_COLOURS]
    sea = m.get("sea_level", 63)
    d = out / name
    d.mkdir(parents=True, exist_ok=True)
    # North (−z) up: row 0 is z0, the north edge.
    krows, brows = [], []
    for j in range(h):
        kr, br = bytearray(), bytearray()
        for i in range(w):
            c = j * w + i
            k = kop[c]
            kc = krgb[k] if k < 32 else (0, 0, 0)
            bc = prgb[bio[c]]
            if k == 31 and hgt[c] > sea:
                kc = (40, 40, 40)   # land the Köppen map calls sea (coast mismatch between L1 and the terrain)
            kr += bytes(kc)
            br += bytes(bc)
        krows.append(kr)
        brows.append(br)
    write_png(d / "koppen.png", w, h, krows)
    write_png(d / "biomes.png", w, h, brows)
    (d / "cells.bin").write_bytes(bytes(kop) + struct.pack(f"<{w * h}H", *bio))
    census = m["census"]
    # Legend: biomes by land share.
    counts = {}
    for c in range(w * h):
        if kop[c] != 31 and hgt[c] > sea:
            counts[bio[c]] = counts.get(bio[c], 0) + 1
    land = sum(counts.values()) or 1
    legend = sorted(counts.items(), key=lambda kv: -kv[1])
    inspect = []
    idir = src / "inspect"
    if idir.is_dir():
        (d / "inspect").mkdir(exist_ok=True)
        for js in sorted(idir.glob("*.json")):
            v = json.loads(js.read_text())
            if v.get("command") != "biomes":
                continue
            imgs = []
            for im in v["images"]:
                a = im["area"]
                pair = []
                for key in ("koppen_png", "png"):
                    if key in im:
                        p = Path(im[key])
                        local = idir / p.name
                        if local.exists():
                            shutil.copy(local, d / "inspect" / p.name)
                            pair.append(f"{name}/inspect/{p.name}")
                imgs.append((a, pair))
            inspect.append((js.stem, v, imgs))
    return {
        "name": name, "w": w, "h": h, "step": m["step"], "x0": m["x0"], "z0": m["z0"], "palette": pal,
        "census": census, "legend": legend, "land": land, "inspect": inspect,
    }


CSS = """
:root{--bg:#f7f7f5;--fg:#16161a;--muted:#6b6a66;--card:#fff;--line:#e3e2de;--accent:#3b5bdb}
@media (prefers-color-scheme:dark){:root:not([data-theme=light]){--bg:#15161a;--fg:#ececef;--muted:#9a9aa3;--card:#1e1f24;--line:#2d2e35;--accent:#8ea2ff}}
body{margin:0;background:var(--bg);color:var(--fg);font:14px/1.5 system-ui,sans-serif}
main{max-width:1400px;margin:0 auto;padding:24px 16px 64px}
h1{font-size:22px;margin:0 0 4px}h2{font-size:18px;margin:32px 0 8px}h3{font-size:15px;margin:20px 0 6px}
p{max-width:80ch}.muted{color:var(--muted)}
.pair{display:grid;grid-template-columns:1fr 1fr;gap:12px}
@media (max-width:800px){.pair{grid-template-columns:1fr}}
figure{margin:0;background:var(--card);border:1px solid var(--line);border-radius:8px;padding:8px}
figure img{width:100%;image-rendering:pixelated;display:block;border-radius:4px}
figcaption{font-size:12px;color:var(--muted);margin-top:4px}
table{border-collapse:collapse;font-size:13px;background:var(--card)}
td,th{border:1px solid var(--line);padding:3px 8px;text-align:left;vertical-align:top}
th{font-weight:600}.num{text-align:right;font-variant-numeric:tabular-nums}
.sw{display:inline-block;width:12px;height:12px;border-radius:2px;vertical-align:-2px;margin-right:6px;border:1px solid #0003}
.legend{columns:3 260px;font-size:12px}.legend div{break-inside:avoid}
.readout{font:12px ui-monospace,monospace;min-height:1.5em;color:var(--muted)}
.scroll{overflow-x:auto}
"""


def page(sections, out):
    kleg = "".join(
        f'<div><span class="sw" style="background:{c}"></span>{k}</div>'
        for k, c in zip(json.loads(json.dumps([
            "Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc", "Cfa",
            "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb", "Dfc", "Dfd",
            "ET", "EF", "sea"])), KOPPEN_COLOURS))
    parts = []
    meta = {}
    for s in sections:
        n = s["name"]
        meta[n] = {k: s[k] for k in ("w", "h", "step", "x0", "z0", "palette")}
        c = s["census"]
        rows = "".join(
            f"<tr><td>{html.escape(k)}</td><td>{html.escape(v['classes'])}</td><td class=num>{v['columns']}</td>"
            f"<td class=num>{'' if v['share'] is None else f'{100 * v['share']:.1f} %'}</td>"
            f"<td>{html.escape(' '.join(v['tags']))}</td></tr>"
            for k, v in c["checks"].items())
        byk = "".join(
            f"<tr><td><span class=sw style='background:{KOPPEN_COLOURS[i]}'></span>{html.escape(k)}</td>"
            f"<td>{html.escape(KOPPEN_NAMES.get(k, ''))}</td><td class=num>{v['columns']}</td>"
            f"<td>{html.escape(', '.join(v['top']))}</td></tr>"
            for i, (k, v) in sorted(((["Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb",
                                          "Cwc", "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd",
                                          "Dfa", "Dfb", "Dfc", "Dfd", "ET", "EF", "sea"].index(k), (k, v))
                                        for k, v in c["by_koppen"].items())))
        leg = "".join(
            f'<div><span class="sw" style="background:#%02x%02x%02x"></span>{html.escape(s["palette"][b])} '
            f'<span class=muted>{100 * n_ / s["land"]:.1f} %</span></div>' % biome_rgb(s["palette"][b])
            for b, n_ in s["legend"][:60])
        insp = ""
        for stem, v, imgs in s["inspect"]:
            figs = ""
            for a, pair in imgs:
                cap = f"x {a['x0']}…{a['x1']}, z {a['z0']}…{a['z1']}"
                figs += "<div class=pair>" + "".join(
                    f"<figure><img src='{p}' alt=''><figcaption>{'Köppen (world data)' if 'koppen' in p else 'biomes stored in the chunks'}: {cap}</figcaption></figure>"
                    for p in pair) + "</div>"
            bk = "".join(
                f"<tr><td>{html.escape(k)}</td><td class=num>{x['columns']}</td><td>{html.escape(', '.join(f'{b} {n_}' for b, n_ in x['biomes'][:6]))}</td></tr>"
                for k, x in v.get("by_koppen", {}).items())
            insp += (f"<h3>Chunky area ({html.escape(stem)}): the chunks' own biomes</h3>{figs}"
                     f"<div class=scroll><table><tr><th>Köppen</th><th>columns</th><th>biomes stored there</th></tr>{bk}</table></div>")
        parts.append(f"""
<h2>{html.escape(n)}</h2>
<p class=muted>Whole planet, one sample every {s['step']} blocks ({s['w']} × {s['h']}); north up, the seam at the left/right edges.
Biomes from the game's own biome source at the terrain surface (what chunk generation and <code>/locate biome</code> use), through this world's climate mapping: {html.escape(c.get('mapping', ''))}.
Hover a map for the class and biome.</p>
<div class=pair>
<figure><img id="k-{n}" src="{n}/koppen.png" alt="Köppen map"><figcaption>Köppen–Geiger class (world data, L1); dark grey = land the L1 map calls sea</figcaption></figure>
<figure><img id="b-{n}" src="{n}/biomes.png" alt="Biome map"><figcaption>Surface biome ({len(s['palette'])} biomes)</figcaption></figure>
</div>
<div class=readout id="r-{n}"></div>
<h3>Köppen families and their biomes' tags</h3>
<div class=scroll><table><tr><th>family</th><th>classes</th><th>columns</th><th>tagged</th><th>any of</th></tr>{rows}</table></div>
<h3>The commonest biomes of each Köppen class</h3>
<div class=scroll><table><tr><th>class</th><th></th><th>columns</th><th>biomes</th></tr>{byk}</table></div>
<h3>Biome legend (share of land samples)</h3><div class=legend>{leg}</div>
{insp}
""")
    doc = f"""<!doctype html><html lang=en><head><meta charset=utf-8><meta name=viewport content="width=device-width,initial-scale=1">
<title>P1-09 Köppen vs biomes</title><style>{CSS}</style></head><body><main>
<h1>Deep Time P1-09: biomes follow the simulated climate</h1>
<p class=muted>A real baked planet (<code>earthlike_quick_16k</code>, scattered continents, 500 Myr). Left: the Köppen–Geiger map the climate stage computed. Right: the biome the pack places there, standalone (vanilla biomes) and in MIC (Terralith). The climate mapping (plan §7.2) turns physical climate into vanilla's five biome inputs so that deserts land in BW, jungle in Af, taiga in Dfc and snow in ET/EF.</p>
<details><summary>Köppen legend</summary><div class=legend>{kleg}</div></details>
{''.join(parts)}
</main>
<script>
const META={json.dumps(meta)};
const KC={json.dumps(["Af","Am","Aw","As","BWh","BWk","BSh","BSk","Csa","Csb","Csc","Cwa","Cwb","Cwc","Cfa","Cfb","Cfc","Dsa","Dsb","Dsc","Dsd","Dwa","Dwb","Dwc","Dwd","Dfa","Dfb","Dfc","Dfd","ET","EF","sea"])};
for (const [n,m] of Object.entries(META)) {{
  let cells=null;
  fetch(n+'/cells.bin').then(r=>r.arrayBuffer()).then(b=>{{cells=b;}}).catch(()=>{{}});
  const r=document.getElementById('r-'+n);
  for (const id of ['k-'+n,'b-'+n]) {{
    const img=document.getElementById(id);
    img.addEventListener('mousemove',e=>{{
      const b=img.getBoundingClientRect();
      const i=Math.floor((e.clientX-b.left)/b.width*m.w), j=Math.floor((e.clientY-b.top)/b.height*m.h);
      if(i<0||j<0||i>=m.w||j>=m.h) return;
      const x=m.x0+i*m.step, z=m.z0+j*m.step;
      let t='x '+x+', z '+z;
      if(cells){{const c=j*m.w+i; const k=new Uint8Array(cells,0,m.w*m.h)[c]; const bi=new DataView(cells).getUint16(m.w*m.h+2*c,true); t+='  ·  '+KC[k]+'  ·  '+m.palette[bi];}}
      r.textContent=t;
    }});
  }}
}}
</script></body></html>"""
    (out / "index.html").write_text(doc)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--profile", action="append", required=True, metavar="NAME=DIR")
    a = ap.parse_args()
    a.out.mkdir(parents=True, exist_ok=True)
    secs = []
    for p in a.profile:
        n, d = p.split("=", 1)
        secs.append(profile_section(n, Path(d), a.out))
    page(secs, a.out)
    total = sum(f.stat().st_size for f in a.out.rglob("*") if f.is_file())
    print(f"wrote {a.out / 'index.html'} ({total / 1e6:.1f} MB)")


if __name__ == "__main__":
    main()
