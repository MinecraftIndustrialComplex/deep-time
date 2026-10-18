#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The scale comparison of the Phase 1 exit (plan §10.3 "Phase 1 exit test" item 7; decisions file, "Horizontal
scale": about 1:300, to be playtested against about 1:100 and Earth-proportioned).

    python3 tools/review/phase1_scale.py [--only presets,bake,region,render,stats,page] [--scales 100,300,1221]

One planet, three horizontal scales. The circumference stays 32,768 blocks, so a scale changes what a block
is (100 m, 300 m, 1,221 m = 2π × 6,370 km / 32,768, Earth-proportioned) and with it the planet's radius
(521 km, 1,565 km, 6,371 km) and how wide features are in blocks; the vertical scale (35 m per block) is
unchanged. The three bakes share seed, start (scattered continents) and length (500 Myr), and the plate
history does not depend on the scale (it is Earth-normalized), so one window of the planet (2,048 × 2,048
blocks, the same longitude and latitude at every scale) is the same patch of planet each time. For each scale:

  presets  earthlike_quick_32k with scale.m_per_block_h changed       (phase1_vm.py scale-preset)
  bake     deeptime run, three at once through the admission tool     -> <WORK>/h<m>/planet/deeptime
  region   the window: land shares that agree across the scales (within 0.05 of each other, as near 0.6
           as the bakes allow), high relief, off the poles and the seam  (phase1_vm.py scale-region)
  render   standalone server + Chunky over the window, BlueMap, two pictures per scale
           (tools/review/phase1-scale-one.sh, one scale after the other: they share smoke/standalone)
  stats    land share, relief and slope in the window per scale       (phase1_vm.py scale-stats)
  page     review/out/phase1/scale/index.html and section.html (the review page's section)

Everything heavy runs on the test VM through tools/remote/mac.sh (admitted); the pictures and statistics come
back to review/out/phase1/scale/ (not committed). Standard library only.
"""

import argparse
import html
import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MAC = os.path.join(REPO, "tools", "remote", "mac.sh")
WORK = "/Volumes/Graphide/ban/deeptime-runs/phase1/scale"
OUT = os.path.join(REPO, "review", "out", "phase1", "scale")
SCALES = (("1:100", 100), ("1:300", 300), ("Earth-proportioned", 1221))
REGION = 2048
C = 32768


def vm(cmd, flags=(), pulls=(), log=None, kind="auto", name="scale"):
    """Detached on the VM (tools/review/phase1_detach.py), so a dropped ssh session does not lose the job."""
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import phase1_detach
    out, rc, secs = phase1_detach.run(name, cmd, flags=flags, pulls=pulls, kind=kind)
    if log:
        os.makedirs(os.path.dirname(log), exist_ok=True)
        open(log, "w").write(out)
    return rc, out, secs


def chosen(a):
    want = {int(v) for v in a.scales.split(",")}
    return [(lab, m) for lab, m in SCALES if m in want]


def last_json(out):
    for line in reversed(out.splitlines()):
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                return json.loads(line)
            except ValueError:
                pass
    return None


def step_presets(a):
    rc, out, _ = vm(f"python3 tools/review/phase1_vm.py scale-preset --work {WORK}", log=os.path.join(OUT, "logs", "presets.log"))
    print(out.strip()[-600:])
    return rc == 0


def step_bake(a):
    build, out, _ = vm("cargo build --release -q -p deeptime-cli -p deeptime-inspect", log=os.path.join(OUT, "logs", "build.log"), kind="cargo:build", name="scale-build")
    if build:
        print(out[-600:])
        return False

    def one(item):
        lab, m = item
        man = f"{WORK}/h{m}/planet/deeptime/manifest.json"
        cmd = (f"[ -f {man} ] || target/release/deeptime run --preset {WORK}/preset-h{m}.json --start scattered_continents "
               f"--length-myr 500 --threads 8 --out {WORK}/h{m}/planet/deeptime > {WORK}/bake-h{m}.log 2>&1; "
               f"tail -3 {WORK}/bake-h{m}.log 2>/dev/null; [ -f {man} ]")
        rc, out, secs = vm(cmd, log=os.path.join(OUT, "logs", f"bake-h{m}.log"), kind="sim:earthlike_quick_32k:8t", name=f"scale-bake-h{m}")
        print(f"bake {lab}: exit {rc} in {secs:.0f} s")
        return rc == 0

    with ThreadPoolExecutor(max_workers=3) as ex:
        return all(ex.map(one, chosen(a)))


def step_region(a):
    rc, out, _ = vm(f"python3 tools/review/phase1_vm.py scale-region --work {WORK}", flags=["--no-sync"], log=os.path.join(OUT, "logs", "region.log"))
    region = last_json(out)
    print(out.strip()[-400:])
    if rc or not region:
        return False
    os.makedirs(OUT, exist_ok=True)
    json.dump(region, open(os.path.join(OUT, "region.json"), "w"), indent=1)
    return True


def load_region():
    return json.load(open(os.path.join(OUT, "region.json")))


def step_render(a):
    region = load_region()
    ok = True
    for lab, m in chosen(a):
        cmd = f"WORK={WORK} tools/review/phase1-scale-one.sh {m} {region['cx']} {region['cz']} {REGION // 2}"
        rc, out, secs = vm(cmd, flags=["--server"], pulls=[f"review/out/phase1/scale/h{m}-tilt.png", f"review/out/phase1/scale/h{m}-flat.png"],
                           log=os.path.join(OUT, "logs", f"render-h{m}.log"), name=f"scale-render-h{m}")
        print(f"render {lab}: exit {rc} in {secs:.0f} s")
        ok &= rc == 0 and os.path.exists(os.path.join(OUT, f"h{m}-tilt.png"))
    return ok


def step_stats(a):
    rc, out, _ = vm(f"python3 tools/review/phase1_vm.py scale-stats --work {WORK}", flags=["--no-sync"], log=os.path.join(OUT, "logs", "stats.log"))
    print(out.strip()[-1200:])
    # `scale-stats` prints one JSON object per scale (the same as WORK/scale-stats.json): take them from its output.
    stats = []
    agreement = None
    for line in out.splitlines():
        line = line.strip()
        if line.startswith('{"label"'):
            try:
                stats.append(json.loads(line))
            except ValueError:
                pass
        elif line.startswith('{"agreement"'):
            try:
                agreement = json.loads(line)["agreement"]
            except (ValueError, KeyError):
                pass
    if len(stats) < len(chosen(a)):
        return False
    os.makedirs(OUT, exist_ok=True)
    json.dump(stats, open(os.path.join(OUT, "scale-stats.json"), "w"), indent=1)
    if agreement:
        json.dump(agreement, open(os.path.join(OUT, "scale-agreement.json"), "w"), indent=1)
    return rc == 0


def section_html(rel=""):
    """The review page's scale section, from review/out/phase1/scale/ (pictures as <rel>h<m>-<view>.png)."""
    esc = html.escape
    stats = {}
    p = os.path.join(OUT, "scale-stats.json")
    if os.path.exists(p):
        stats = {s["m_per_block_h"]: s for s in json.load(open(p))}
    region = json.load(open(os.path.join(OUT, "region.json"))) if os.path.exists(os.path.join(OUT, "region.json")) else None
    cols = []
    for lab, m in SCALES:
        s = stats.get(float(m))
        have = os.path.exists(os.path.join(OUT, f"h{m}-tilt.png"))
        pics = (f'<a href="{rel}h{m}-tilt.png"><img src="{rel}h{m}-tilt.png" alt="{esc(lab)} from the south, tilted"></a>'
                f'<a href="{rel}h{m}-flat.png"><img src="{rel}h{m}-flat.png" alt="{esc(lab)} from above"></a>') if have else "<p class=\"bad\">not rendered</p>"
        facts = ""
        if s:
            facts = ("<table class=\"facts\">"
                     f"<tr><td>metres per block</td><td>{s['m_per_block_h']:g} horizontal, {s['m_per_block_v']:g} vertical</td></tr>"
                     f"<tr><td>planet radius</td><td>{s['radius_km']:,} km</td></tr>"
                     f"<tr><td>this picture spans</td><td>{s['region_km']:,.0f} km square ({REGION:,} blocks)</td></tr>"
                     f"<tr><td>land share of the window</td><td>{100 * s['land_share']:.1f} %</td></tr>"
                     f"<tr><td>median land slope</td><td>{s['median_slope']:.2f} (95th: {s['p95_slope']:.2f}) blocks of rise per block of run</td></tr>"
                     f"<tr><td>land steeper than 45°</td><td>{100 * s['steep_share']:.1f} %</td></tr>"
                     f"<tr><td>highest ground</td><td>y {s['max_y']} ({(s['max_y'] - 63) * s['m_per_block_v'] / 1000:.1f} km above sea level)</td></tr></table>")
        cols.append(f'<div class="col"><h3>{esc(lab)}</h3>{pics}{facts}</div>')
    note = ""
    if region:
        by = region.get("land_by_scale") or {}
        shares = ", ".join(f"{100 * v:.0f} %" for v in by.values())
        note = (f"Window {REGION:,} × {REGION:,} blocks centred on block {region['cx']:,}, {region['cz']:,}, the same longitude and latitude at every scale, "
                f"chosen so the three land shares agree within {100 * region.get('land_tol', 0.05):.0f} points ({shares}; target "
                f"{100 * region.get('land_target', 0):.0f} %) and the slope statistics compare like with like.")
    return (f'<div class="scale"><p>One planet (seed, start and length equal), simulation C = {C:,} blocks (the Petroff-Guyou world is a little smaller: 26,112 x 13,056 blocks), vertical scale 35 m per block at every scale: only '
            f'the metres a block stands for change. {esc(note)} Wider features take more blocks at 1:100; at Earth proportions a mountain range '
            f'a few hundred kilometres wide is a few hundred blocks and a plain is the size of a Minecraft village.</p>'
            f'<div class="cols">{"".join(cols)}</div></div>')


SECTION_CSS = """
.scale .cols{display:grid;grid-template-columns:repeat(3,1fr);gap:12px}
.scale .col img{width:100%;height:auto;display:block;border:1px solid var(--line);margin-bottom:6px;border-radius:4px}
.scale table.facts{font-size:12.5px;border:none;margin:4px 0}.scale table.facts td{border:none;padding:1px 6px 1px 0}
.scale .bad{color:var(--bad)}
@media (max-width:900px){.scale .cols{grid-template-columns:1fr}}
"""


def step_page(a):
    os.makedirs(OUT, exist_ok=True)
    body = section_html()
    open(os.path.join(OUT, "section.html"), "w").write(body)
    page = (f'<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">'
            f'<title>Deep Time scale comparison</title><style>:root{{--bg:#fbfaf7;--fg:#1d1d1b;--line:#e3e0d8;--bad:#b3261e}}'
            f'body{{background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,sans-serif;margin:24px 16px}}main{{max-width:1300px;margin:0 auto}}{SECTION_CSS}'
            f'</style></head><body><main><h1>Scale comparison: 1:100, 1:300, Earth-proportioned</h1>{body}</main></body></html>')
    open(os.path.join(OUT, "index.html"), "w").write(page)
    print(f"page: {os.path.join(OUT, 'index.html')}")
    return True


STEPS = [("presets", step_presets), ("bake", step_bake), ("region", step_region), ("render", step_render), ("stats", step_stats), ("page", step_page)]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--only")
    ap.add_argument("--scales", default="100,300,1221")
    a = ap.parse_args()
    only = set(a.only.split(",")) if a.only else None
    os.makedirs(os.path.join(OUT, "logs"), exist_ok=True)
    ok = True
    for name, fn in STEPS:
        if only is not None and name not in only:
            continue
        print(f"== {name}", flush=True)
        good = fn(a)
        print(f"   {'ok' if good else 'FAILED'}", flush=True)
        ok &= bool(good)
        if not good:
            break
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
