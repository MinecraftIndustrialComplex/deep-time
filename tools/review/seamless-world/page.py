#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The seamless world's review page (SW-1): the evidence run's two worlds side by side, the layout
the wrap engine reads, the projection pass's record and the gates.

    python3 tools/review/seamless-world/page.py <evidence dir> <out dir> [gates.json]

<evidence dir> is what tools/review/seamless-evidence.sh wrote, pulled back (summary.json,
pg-map/, lambert-map/); the page and its pictures go to <out dir>, ready for
tools/review/publish.sh seamless-world <out dir>.
"""
import html
import json
import shutil
import sys
from pathlib import Path

ev, out = Path(sys.argv[1]), Path(sys.argv[2])
gates = json.loads(Path(sys.argv[3]).read_text()) if len(sys.argv) > 3 else {}
out.mkdir(parents=True, exist_ok=True)
for w in ("pg", "lambert"):
    for f in ("satellite.png", "provinces.png"):
        src = ev / f"{w}-map" / f
        if src.exists():
            shutil.copy(src, out / f"{w}-{f}")
s = json.loads((ev / "summary.json").read_text())
wm = json.loads((ev / "pg-map" / "world-map.json").read_text())
bp = s["pg"]["bake_projection"]
lam = bp["lambda_c"]
pa = bp["pole_anisotropy"]
sh = bp["shares"]
proj = wm.get("projection", {})


def f(v, n=3):
    return f"{v:.{n}f}"


marks = "".join(
    f'<circle cx="{m["px"][0] / 2048 * 100:.3f}%" cy="{m["px"][1] / 1024 * 100:.3f}%" r="7" '
    f'class="{"pole" if "pole" in m["what"] else "cone"}"><title>{html.escape(m["what"])} ({m["x"]}, {m["z"]})</title></circle>'
    for m in wm["marks"])
edges = "".join(
    f"<tr><td>{html.escape(e['name'])}</td><td>{e['kind']}</td><td>{e['axis']} = {e['coord']}</td>"
    f"<td>[{', '.join(map(str, e['iso']))}]</td><td>{e.get('pivot', '—')}</td><td>{e['owner']}</td></tr>"
    for e in proj.get("edges", []))
cones = "".join(
    f"<tr><td>{html.escape(c['name'])}</td><td>{f(c['lat_deg'], 2)}°</td><td>{f(c['lon_deg'], 2)}°</td>"
    f"<td>{f(d, 1)}°</td></tr>"
    for c, d in zip(proj.get("cone_points", []), lam["cone_distances_deg"]))
gate_rows = "".join(f"<tr><td>{html.escape(k)}</td><td>{html.escape(str(v))}</td></tr>" for k, v in gates.items())

page = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Deep Time: the seamless world</title>
<meta name="description" content="The Petroff-Guyou world in the format, the bake and the game (SW-1): one quick 16k planet as the new world and as the Lambert world, the layout the wrap engine reads, the bake's record and the gates.">
<style>
:root {{ --bg:#0f1216; --fg:#e8eaed; --mute:#9aa3ad; --card:#181d23; --line:#2a313a; --acc:#7cc4ff; --pole:#ffd166; --cone:#ff6b6b; }}
@media (prefers-color-scheme: light) {{ :root {{ --bg:#f7f8fa; --fg:#14181d; --mute:#5b6570; --card:#fff; --line:#dde2e8; --acc:#0b62b5; }} }}
body {{ margin:0; background:var(--bg); color:var(--fg); font:16px/1.55 system-ui,-apple-system,Segoe UI,Roboto,sans-serif; }}
main {{ max-width:1180px; margin:0 auto; padding:24px 16px 64px; }}
h1 {{ font-size:1.9rem; margin:.2em 0 .1em; }} h2 {{ margin-top:2.2em; font-size:1.3rem; }}
.lede {{ color:var(--mute); max-width:70ch; }}
.map {{ position:relative; border-radius:8px; overflow:hidden; border:1px solid var(--line); }}
.map img {{ display:block; width:100%; height:auto; }}
.map svg {{ position:absolute; inset:0; width:100%; height:100%; }}
.map circle.cone {{ fill:none; stroke:var(--cone); stroke-width:3; }} .map circle.pole {{ fill:var(--pole); stroke:#000; stroke-width:1; }}
.map line {{ stroke:#fff; stroke-dasharray:6 6; stroke-width:1.5; opacity:.6; }}
.cap {{ color:var(--mute); font-size:.9rem; margin:.4em 0 1.4em; }}
.grid {{ display:grid; grid-template-columns:repeat(auto-fit,minmax(220px,1fr)); gap:12px; }}
.stat {{ background:var(--card); border:1px solid var(--line); border-radius:8px; padding:12px 14px; }}
.stat b {{ display:block; font-size:1.5rem; }} .stat span {{ color:var(--mute); font-size:.85rem; }}
table {{ border-collapse:collapse; width:100%; font-size:.92rem; }} td,th {{ border-bottom:1px solid var(--line); padding:6px 8px; text-align:left; }}
.wrap {{ overflow-x:auto; }}
code {{ background:var(--card); padding:1px 5px; border-radius:4px; }}
</style></head><body><main>
<h1>The seamless world</h1>
<p class="lede">Petroff's equal-area map in the Guyou 2&#8239;:&#8239;1 layout is now the default world (format 0.5).
Below, one quick 16k planet ({s['pg']['scale']['circumference_blocks']:,} blocks round) baked twice: as the new world
and as the Lambert world it replaces. The simulation is the same; only the bake's last pass differs.</p>

<h2>The new world: {bp['world_blocks'][0]:,} × {bp['world_blocks'][1]:,} blocks</h2>
<div class="map"><img src="pg-satellite.png" alt="Satellite view of the Petroff-Guyou world">
<svg viewBox="0 0 2048 1024" preserveAspectRatio="none"><line x1="1024" y1="0" x2="1024" y2="1024"/><line x1="0" y1="512" x2="2048" y2="512"/></svg>
<svg>{marks}</svg></div>
<p class="cap">Red rings: the four cone points (each corner is one point, drawn at both ends). Yellow dots: the poles, on the
north and south edges a quarter of the way in. Dashed: the line between the two hemisphere squares and the equator.
East and west wrap (Toroidal World); the north edge is glued onto itself by a half-turn about its midpoint, and so is the
south edge. Until the wrap engine joins those two edges a temporary border stops players a block inside them.</p>

<div class="grid">
<div class="stat"><b>{f(sh['land_map'], 4)}</b><span>land share on the map (planet: {f(sh['land_true'], 4)})</span></div>
<div class="stat"><b>{f(sh['ice_cover_map'], 4)}</b><span>ice share on the map (planet: {f(sh['ice_cover_true'], 4)})</span></div>
<div class="stat"><b>{f(pa['poles_tangential_over_radial'], 2)}</b><span>polar streaking, 80–88°: tangential over radial slope (point sampling: {f(pa['point_sampled_poles'], 1)}; mid-latitudes {f(pa['mid_latitudes_tangential_over_radial'], 2)}; 1 = no preferred direction)</span></div>
<div class="stat"><b>{f(pa.get('subpolar_tangential_over_radial', float('nan')), 2)}</b><span>the same at 70–80° (point sampling: {f(pa.get('point_sampled_subpolar', float('nan')), 1)})</span></div>
<div class="stat"><b>{f(bp['lambda_c_deg'], 2)}°</b><span>λc chosen; nearest land to a cone point {f(lam['min_distance_to_land_deg'], 1)}°</span></div>
<div class="stat"><b>{f(bp['m_per_block_h'], 1)} m</b><span>true size of a block (Lambert world: 300 m)</span></div>
<div class="stat"><b>{bp['millis']['total'] / 1000:.1f} s</b><span>projection pass</span></div>
</div>

<h2>The Lambert world (still a setting)</h2>
<div class="map"><img src="lambert-satellite.png" alt="Satellite view of the same planet on the Lambert world"></div>
<p class="cap">The same planet as a Lambert world (<code>scale.projection = lambert</code>, level type
<code>deeptime:planet_lambert</code>): stretched east–west towards the poles, closed off by the pole cap at 80°.</p>

<h2>The layout the wrap engine reads</h2>
<p>Every world's <code>manifest.json</code> carries a <code>projection</code> block (FORMAT.md "The world's layout",
<code>layout_version</code> 1). Each edge says what lies beyond it, as the engine's <code>CellIsometry</code>
<code>[quarter turns, tx, tz]</code>, and who joins it.</p>
<div class="wrap"><table><tr><th>edge</th><th>kind</th><th>line</th><th>iso</th><th>pivot</th><th>owner</th></tr>{edges}</table></div>
<h3>Cone points and the λc search</h3>
<p>The bake tries every quarter degree of λc and keeps the one whose four cone points lie farthest from land. On this
planet {f(lam['share_all_at_sea'] * 100, 1)} % of the candidates put all four at sea and none puts all four 3° from land.</p>
<div class="wrap"><table><tr><th>cone point</th><th>latitude</th><th>longitude</th><th>to land</th></tr>{cones}</table></div>

<h2>One projection, three languages</h2>
<p>Rust (the bake), Java (the game) and JavaScript (the workbench) run the projection on the same portable
trigonometry and give the same bits: Java's <code>ProjectionParityTest</code> and the pages'
<code>tools/viewer/projection-check.js</code> (5,280 values) reproduce every value of the Rust fixture.</p>

<h2>Gates</h2>
<div class="wrap"><table>{gate_rows or '<tr><td>pending</td><td></td></tr>'}</table></div>
</main></body></html>
"""
(out / "index.html").write_text(page)
print(out / "index.html")
