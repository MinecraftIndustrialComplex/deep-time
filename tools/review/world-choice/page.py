#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The world-shape decision page: one planet (earthlike_quick_16k, seed 1, supercontinent start,
1,000 Myr) baked as Petroff-Guyou and as the cube, side by side in the game.

  page.py --cmp <dir> --shots <review/out> --out <folder>

<dir> is what tools/review/world-choice/compare.sh wrote (compare.json, the world maps); <review/out>
holds the client passes of shots.sh (cmp-pg-near, cmp-cube-near, cmp-pg-far, cmp-cube-far,
cmp-pg-walk) and the cube's editor walk (cube2-walk). Writes <folder>/index.html and <folder>/img/.
Runs on the test VM (Pillow); publish with tools/review/publish.sh world-choice <folder>.
"""
import argparse
import html
import json
import os

from PIL import Image

ap = argparse.ArgumentParser()
ap.add_argument("--cmp", required=True)
ap.add_argument("--shots", required=True)
ap.add_argument("--out", required=True)
ap.add_argument("--gates", default="", help="a JSON file of gate rows [[step, verdict, note], ...]")
a = ap.parse_args()
os.makedirs(f"{a.out}/img", exist_ok=True)
C = json.load(open(f"{a.cmp}/compare.json"))
missing = []


def put(src, name, width=1280, png=False):
    """The picture src, scaled to at most width pixels, as img/<name>; None when it is missing."""
    if not os.path.exists(src):
        missing.append(src)
        return None
    im = Image.open(src)
    im = im.convert("RGB")
    if im.width > width:
        im = im.resize((width, round(im.height * width / im.width)), Image.LANCZOS)
    path = f"img/{name}.{'png' if png else 'jpg'}"
    if png:
        im.save(f"{a.out}/{path}", optimize=True)
    else:
        im.save(f"{a.out}/{path}", quality=84, optimize=True, progressive=True)
    return path


def fig(path, tag, cap="", alt=""):
    if path is None:
        return f'<figure><span class="tag">{tag}</span><p class="cap">(not shot)</p></figure>'
    return (f'<figure><span class="tag">{tag}</span><a href="{path}"><img src="{path}" alt="{html.escape(alt or tag)}" '
            f'loading="lazy"></a>{f"<figcaption class=cap>{cap}</figcaption>" if cap else ""}</figure>')


def f2(v):
    return f"{v:.2f}"


S = C["stats"]
pgl, cul = S["pg16"]["land"], S["cube16"]["land"]
pgp, cup = S["pg16"]["planet"], S["cube16"]["planet"]
s = C["face_blocks"]
d = C["pg_domain"]
pg_w, pg_h = int(d["x_max"] - d["x_min"]), int(d["z_max"] - d["z_min"])
CM = json.load(open(f"{a.cmp}/cube16/deeptime/manifest.json"))
corner_d = [c["land_distance_deg"] for c in CM["projection"].get("cone_points", []) if "land_distance_deg" in c]
corner_txt = (f"at sea, {min(corner_d):.1f}° to {max(corner_d):.1f}° from the nearest land on this planet"
              if corner_d else "at sea")

# ---- pictures ---------------------------------------------------------------------------------
P = {}
P["pg_sat"] = put(f"{a.cmp}/pg16-map/satellite.png", "pg-satellite", 1600)
P["cu_sat"] = put(f"{a.cmp}/cube16-map/satellite.png", "cube-satellite", 1600)
P["pg_prov"] = put(f"{a.cmp}/pg16-map/provinces.png", "pg-provinces", 1600, png=True)
P["cu_prov"] = put(f"{a.cmp}/cube16-map/provinces.png", "cube-provinces", 1600, png=True)
P["pg_globe"] = put(f"{a.shots}/cmp-pg-walk/05-generation-globe.png", "pg-editor-globe")
P["pg_review"] = put(f"{a.shots}/cmp-pg-walk/08-review.png", "pg-editor-review")
P["cu_globe"] = put(f"{a.shots}/cube2-walk/06d-generation-globe-cube.png", "cube-editor-globe")
P["cu_review"] = put(f"{a.shots}/cube2-walk/08-review.png", "cube-editor-review")
wm = json.load(open(f"{a.cmp}/cube16-map/world-map.json"))
seams = []
for v in sorted(wm.get("seam_views", []), key=lambda v: -v["score"])[:4]:
    seams.append((v, put(f"{a.cmp}/cube16-map/{v['file']}", "seam-" + v["file"][:-4], 900, png=True)))


def pair(kind, k):
    return (put(f"{a.shots}/cmp-pg-{kind}/11-vista-{k}.png", f"{kind}-{k}-pg"),
            put(f"{a.shots}/cmp-cube-{kind}/11-vista-{k}.png", f"{kind}-{k}-cube"))


def numbers(v, cube):
    if cube:
        return f"shapes true here (×{f2(v['stretch'])}); ground drawn ×{f2(v['area'])} its true size"
    return f"shapes stretched ×{f2(v['stretch'])} here; area exact (×{f2(v['area'])})"


def pairs(kind, which):
    out = []
    labels = C[kind]["labels"]
    for k in which:
        if k >= len(labels):
            continue
        pg, cu = pair(kind, k)
        vp, vc = C[kind]["pg16"][k], C[kind]["cube16"][k]
        out.append(f'<h3>{html.escape(labels[k][0].upper() + labels[k][1:])}</h3><div class="two">'
                   + fig(pg, "Petroff-Guyou", numbers(vp, False) + f" · camera at x {vp['x']:.0f}, z {vp['z']:.0f}")
                   + fig(cu, "Cube", numbers(vc, True) + f" · camera at x {vc['x']:.0f}, z {vc['z']:.0f}")
                   + "</div>")
    return "\n".join(out)


def idx(kind, start):
    return [k for k, l in enumerate(C[kind]["labels"]) if l.startswith(start)]


gates = json.load(open(a.gates)) if a.gates and os.path.exists(a.gates) else []
gate_rows = "\n".join(
    f'<tr><td data-label="Step">{html.escape(g[0])}</td><td data-label="Result">{html.escape(g[1])}</td>'
    f'<td data-label="What it checked">{html.escape(g[2])}</td></tr>' for g in gates)

same = idx("near", "mountains") + idx("near", "coast") + idx("near", "north pole")
odd = idx("near", "Petroff") + idx("near", "the cube's most")
edges = idx("near", "the cube's corner") + idx("near", "a turning") + idx("near", "the slide") + idx("near", "a straight")

body = f"""
<nav class="top" data-ss-react="off">
  <a class="home" href="https://spacesheep.dev/@benmac/rtsim-hub">All pages</a>
  <a href="#question">The question</a>
  <a href="#numbers">Numbers</a>
  <a href="#places">Same places</a>
  <a href="#edges">Edges</a>
  <a href="#far">Far view</a>
  <a href="#cost">Deleting</a>
  <a href="#decide">Decide</a>
</nav>
<main>
<h1>Cube or Petroff-Guyou</h1>
<p class="dek">One planet (earthlike quick, 16k, seed 1, supercontinent start, 1,000 million years), baked
twice: once on today's Petroff-Guyou world and once on the cube. The same places, shot in the game.
24 October 2026.</p>

<div class="box" id="question" data-ss-id="question">
<p><strong>The question.</strong> The cube now plays: it bakes, generates, has its border, its far
view and its F3 line, and it passed the same start-up checks as Petroff-Guyou. On 4 October you said
you would swap the world shape yourself if the distortion was too bad in play. Should the cube become
the only world, and Petroff-Guyou (with the old Lambert world and the prototype) be deleted?</p>
<p><strong>The short answer from the pictures:</strong> on the cube, mountains and coasts keep their
shape everywhere; on Petroff-Guyou a mountain can be drawn
{f2(pgl['stretch']['max'])} times longer one way than the other, and half of this planet's land is
drawn at least {f2(pgl['stretch']['median'])} times longer one way. The cube pays with ground drawn
bigger near its eight corners (up to {f2(cul['area']['max'])} times on this planet's land), with
borders at 12 edges until the wrap engine joins them, and with north turning as you travel. The
recommendation is to make the cube the only world (the question at the end).</p>
</div>

<section id="glance" data-ss-section="glance">
<h2>The two worlds at a glance</h2>
<table class="stack">
<thead><tr><th></th><th>Petroff-Guyou (today's default)</th><th>Cube</th></tr></thead>
<tbody>
<tr><td>Size at 16k</td><td data-label="Petroff-Guyou">a rectangle of {pg_w:,} × {pg_h:,} blocks
({pg_w * pg_h / 1e6:.1f} million blocks)</td><td data-label="Cube">six square faces of {s:,} blocks in a cross,
{4 * s:,} blocks round ({6 * s * s / 1e6:.1f} million blocks)</td></tr>
<tr><td>Shapes</td><td data-label="Petroff-Guyou">stretched: half of this planet's land at least
×{f2(pgl['stretch']['median'])} longer one way, the worst ×{f2(pgl['stretch']['max'])}</td>
<td data-label="Cube">true everywhere (at most ×{f2(cul['stretch']['max'])})</td></tr>
<tr><td>Area per block</td><td data-label="Petroff-Guyou">the same everywhere</td>
<td data-label="Cube">land drawn ×{f2(cul['area']['min'])} to ×{f2(cul['area']['max'])} its true size;
{cul['area_share_over_1.25'] * 100:.0f} % of the land more than ×1.25</td></tr>
<tr><td>Where you can walk today</td><td data-label="Petroff-Guyou">all the way round east and west;
stopped at the north and south edges, where the two poles lie</td>
<td data-label="Cube">all the way round the four middle faces; onto the north and south faces through
the one middle face they both join; stopped at the 12 outer edges of the cross</td></tr>
<tr><td>With the wrap engine (being built)</td><td data-label="Petroff-Guyou">walk over the poles</td>
<td data-label="Cube">walk over every edge; no border left</td></tr>
<tr><td>North</td><td data-label="Petroff-Guyou">-z near the equator, turning towards the poles near
the edges</td><td data-label="Cube">turns as you travel, differently on each face; F3 shows north's
bearing</td></tr>
<tr><td>Odd points</td><td data-label="Petroff-Guyou">four cone points on the edges (at sea)</td>
<td data-label="Cube">eight corners ({corner_txt}), where three faces meet</td></tr>
</tbody></table>
</section>

<section id="maps" data-ss-section="maps">
<h2>The whole planet, both ways</h2>
<p>The same continents, drawn by each world. Petroff-Guyou puts each hemisphere in a square, side by
side; the cube unfolds into a cross. In the cube's picture the grey lines are the face outlines, the
yellow ones the border for now, red squares the corners and white squares the poles.</p>
<div class="nets">
{fig(P['pg_sat'], "Petroff-Guyou: satellite")}
{fig(P['cu_sat'], "Cube: satellite")}
{fig(P['pg_prov'], "Petroff-Guyou: geologic provinces")}
{fig(P['cu_prov'], "Cube: geologic provinces")}
</div>
<h3>In the planet editor</h3>
<p>What a player sees on Create World (the same preset, a shorter 300-million-year run): the globe and
the final map with the spawn.</p>
<div class="two">
{fig(P['pg_globe'], "Petroff-Guyou: the globe")}
{fig(P['cu_globe'], "Cube: the globe, with its edges and border")}
{fig(P['pg_review'], "Petroff-Guyou: the final map")}
{fig(P['cu_review'], "Cube: the final map")}
</div>
</section>

<section id="numbers" data-ss-section="numbers">
<h2>How wrong each one draws the ground</h2>
<p>Measured on this planet's land, every 16 blocks, from where each world puts every block on the
globe. "Stretch" is how much longer a round hill is drawn one way than the other; "drawn bigger" is
how many blocks a square kilometre gets, against the world's average.</p>
<table class="nums stack">
<thead><tr><th>On land</th><th class="num">Petroff-Guyou</th><th class="num">Cube</th></tr></thead>
<tbody>
<tr><td>Stretch, typical (median)</td><td class="num" data-label="Petroff-Guyou">×{f2(pgl['stretch']['median'])}</td><td class="num" data-label="Cube">×{f2(cul['stretch']['median'])}</td></tr>
<tr><td>Stretch, worst 10 %</td><td class="num" data-label="Petroff-Guyou">×{f2(pgl['stretch']['p90'])} or more</td><td class="num" data-label="Cube">×{f2(cul['stretch']['p90'])}</td></tr>
<tr><td>Stretch, worst</td><td class="num" data-label="Petroff-Guyou">×{f2(pgl['stretch']['max'])}</td><td class="num" data-label="Cube">×{f2(cul['stretch']['max'])}</td></tr>
<tr><td>Land stretched more than ×1.5</td><td class="num" data-label="Petroff-Guyou">{pgl['stretch_share_over_1.5'] * 100:.0f} %</td><td class="num" data-label="Cube">{cul['stretch_share_over_1.5'] * 100:.0f} %</td></tr>
<tr><td>Drawn bigger, smallest to largest</td><td class="num" data-label="Petroff-Guyou">×1 everywhere</td><td class="num" data-label="Cube">×{f2(cul['area']['min'])} to ×{f2(cul['area']['max'])}</td></tr>
<tr><td>Land drawn more than ×1.25</td><td class="num" data-label="Petroff-Guyou">0 %</td><td class="num" data-label="Cube">{cul['area_share_over_1.25'] * 100:.0f} %</td></tr>
<tr><td>Land drawn more than ×1.5</td><td class="num" data-label="Petroff-Guyou">0 %</td><td class="num" data-label="Cube">{cul['area_share_over_1.5'] * 100:.0f} %</td></tr>
</tbody></table>
<p class="cap">Over the whole planet, sea included: Petroff-Guyou's stretch is ×{f2(pgp['stretch']['median'])}
typical and ×{f2(pgp['stretch']['max'])} at worst (Petroff's own figures are 1.78 and 2.67); the cube draws
×{f2(cup['area']['p1'])} to ×{f2(cup['area']['p99'])} on 98 % of the planet and more right at its corners, which lie at sea.
The cube study's worst of 15 planets drew land ×0.79 to ×1.92. This planet goes a little past that,
×{f2(cul['area']['max'])}, on land near a corner: its continents let the turn keep the corners only
{min(corner_d):.1f}° from the nearest coast (a degree is about {4 * s / 360:.0f} blocks on average).</p>
<p>What it means in play. On Petroff-Guyou a round volcano or a crater is an oval wherever the stretch
is high, and a mountain range running one way looks narrower or wider than the same range running the
other way. On the cube everything keeps its shape; near a corner the ground is simply drawn larger, so
a hill there is wider in blocks and its slopes a little gentler, by the square root of the number
above (×{f2(cul['area']['max'] ** 0.5)} across at the very worst).</p>
</section>

<section id="places" data-ss-section="places">
<h2>The same places, side by side</h2>
<p>Each pair looks at the same ground: the camera and what it looks at are the same points on the
planet, put where each world draws them. Shots from the real client with Distant Horizons, the HUD
hidden. Blocks are vanilla stand-ins for the rock (no MIC pack here), the same on both sides.</p>
{pairs("near", same)}
<h3 style="margin-top:36px">Where each world is at its worst</h3>
<p>From above, north up: the land Petroff-Guyou stretches most, and the land the cube draws biggest.</p>
{pairs("near", odd)}
</section>

<section id="edges" data-ss-section="edges">
<h2>Edges and corners: what playing across them feels like</h2>
<p><strong>The four middle faces</strong> wrap round east and west like today's world: you never see
the edges between them, and the step from the last face back to the first (the slide) is as seamless.
<strong>The north and south faces</strong> join one middle face, above and below it, the same way:
every place on the planet, both poles included, can be reached on foot through that face. <strong>The 12 outer
edges</strong> of the cross are where a face meets one that the cross puts somewhere else. Until the
wrap engine joins them, the border stops you one block inside, with a warning on the action bar 24
blocks before; spectators pass. The ground you see past the border is already the right ground,
turned to meet you, so the view is never empty.</p>
<p><strong>North turns.</strong> The cube is turned freely on the planet so that its corners sit at
sea, so no face has north pointing straight up the map. The F3 screen gives each place's face and
north's bearing; the two shots across a turning edge below show the bearing on each side.</p>
<p>Petroff-Guyou has a border too: at its north and south edges, where the poles are, so nobody can
walk over a pole today. Its four cone points are at sea.</p>
{pairs("near", edges)}
<h3>Across the seams, from above</h3>
<p>The bake's close views across the cube's edges, one block a pixel, where they cross the most
relief: the yellow ticks at the rim mark the edge. Ground runs on across it.</p>
<div class="nets">
{"".join(fig(p, html.escape(v['seam']) + f" ({v['kind']})") for v, p in seams)}
</div>
</section>

<section id="far" data-ss-section="far">
<h2>The far view (Distant Horizons)</h2>
<p>High above, after four minutes for Distant Horizons to draw the far terrain.</p>
{pairs("far", range(len(C['far']['labels'])))}
</section>

<section id="gates" data-ss-section="gates">
<h2>Does the cube work end to end?</h2>
<p>The checks Petroff-Guyou passed before it became the default, run on cube worlds this time.</p>
<table class="stack checks">
<thead><tr><th>Step</th><th>Result</th><th>What it checked</th></tr></thead>
<tbody>{gate_rows}</tbody></table>
</section>

<section id="cost" data-ss-section="cost">
<h2>What deleting the old shapes would remove</h2>
<p>The cube plan's section 7.4: when phase 2 lands, delete the Lambert world, Petroff-Guyou and the
quincuncial prototype, so the cube is the only world shape. Counted on master today:</p>
<table class="stack">
<thead><tr><th>Part</th><th>What goes</th><th>Size</th></tr></thead>
<tbody>
<tr><td>Petroff-Guyou</td><td data-label="What goes">the map in Rust, Java and the viewer's JavaScript;
the turn that keeps the cone points at sea; the near-side rule and the wider sampling near the edges;
its parts of the cross-language check; its GameTests; the seamless-world review tools</td><td data-label="Size">
four whole files (1,134 lines) and lines in about 130 more (presets, bake, game, tools)</td></tr>
<tr><td>The Lambert world</td><td data-label="What goes">its level types (planet_lambert, planet_lambert_32k);
the pole cap, the polar storm and the ground past the pole; the Lambert branches in far terrain,
geothermal heat, the magma probe and the terrain readers</td><td data-label="Size">nine whole files
(1,504 lines) and lines in about 70 more</td></tr>
<tr><td>The quincuncial prototype</td><td data-label="What goes">a research crate and its scripts, unused
by the game</td><td data-label="Size">14 files, 3,504 lines</td></tr>
<tr><td>The test planets</td><td data-label="What goes">the GameTests' four baked planets
(the 4k real planet, its caves and two river planets) are Petroff-Guyou bakes: they would be baked
again as cubes, and the tests that stand at fixed places on them (about 56 GameTests in 14 files)
re-pointed</td><td data-label="Size">most of the work of the deletion</td></tr>
<tr><td>Tests</td><td data-label="What goes">the tests of what goes: 26 Rust tests, 7 GameTests (the pole
cap and the Petroff-Guyou world), 8 Java unit tests, and the Petroff-Guyou cases inside shared tests</td>
<td data-label="Size">about 41 whole tests</td></tr>
<tr><td>Kept</td><td data-label="What goes">the research tools (projection survey, sphere grid bench,
mountains and the projection, the world-shape page); the simulation's own Lambert grid until phase 4</td>
<td data-label="Size"></td></tr>
</tbody></table>
<p><strong>The play server.</strong> It runs the Petroff-Guyou world of 22 October on its own build
(tagged <code>play-d5</code>), as planned when the cube plan was made: the plan already keeps it on
that tag, with any hotfix made from the tag, until phase 5 gives it a fresh cube world. Today's
master still reads Petroff-Guyou worlds; after the deletion it would not. So the deletion takes one
thing from the server: a fix made on master could no longer be tried on its world, and would be
copied onto the tag by hand. Its world, its players and its build do not change.</p>
<p><strong>What keeping both would cost.</strong> Every change to the bake, the terrain, the far view,
the border and the tools would keep two world shapes working and tested, about 130 files that name
Petroff-Guyou today; each new feature that touches geometry (the wrap engine, rivers across seams,
the planet API) would be written and tested twice.</p>
</section>

<section id="decide" data-ss-section="decide">
<h2>The decision</h2>
<div class="decision">
<h3>Which world shape stays?</h3>
<p class="ctx">On 22 October you moved the world to a cube and dropped equal area as a strict rule;
on 4 October you said you would swap the shape yourself if distortion felt too bad in play. The
cube is built and checked; what is left is that swap: whether Petroff-Guyou goes, and when.</p>
<ul class="opts">
<li class="rec"><span class="l">A</span><span class="tagrec">Recommended</span><strong>The cube becomes
the only world.</strong> Delete Petroff-Guyou, the Lambert world and the prototype (section 7.4). New
worlds are cubes; the play server keeps its world on its tagged build until phase 5 gives it a cube.
<span class="row"><span class="k">Why</span>true shapes everywhere; one world shape to build the wrap
engine, rivers and the planet API on.</span>
<span class="row"><span class="k">Cost</span>borders at 12 edges until the wrap engine; ground drawn up
to ×{f2(cul['area']['max'])} near corners; north turns.</span></li>
<li><span class="l">B</span><strong>The cube becomes the default, Petroff-Guyou stays selectable</strong>
until the play server moves (phase 5), then delete it.
<span class="row"><span class="k">Cost</span>both shapes kept working and tested meanwhile.</span></li>
<li><span class="l">C</span><strong>Keep Petroff-Guyou as the default</strong>, the cube a choice in the
world-type list, and decide after you have played a cube world.
<span class="row"><span class="k">Cost</span>as B, for longer; the play server's next world would
have to be chosen first.</span></li>
</ul>
</div>
</section>
</main>
<footer>Deep Time, cube plan phase 2 (gate steps and the world-shape material). Bakes: tools/smoke/install.sh dh
--preset earthlike_quick_16k [--cube] --start supercontinent --length-myr 1000; numbers:
tools/review/world-choice/compare.py; shots: tools/client/vm-client.sh with the dh set.</footer>
"""

head = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "head.html")).read()
with open(f"{a.out}/index.html", "w") as f:
    f.write(head + body + "</body>\n</html>\n")
print("missing:", missing)
