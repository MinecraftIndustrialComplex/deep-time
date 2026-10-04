#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-24 review page: the faster-quick-mode candidates side by side, with their run times.

    python3 tools/review/p1-24-page.py <runs dir> <views dir> label=run-name:"description" ...

Reads each run's result.json (tools/review/p1-24-profile.sh) and the pictures and views.json of
tools/review/p1-24-views.py in <views dir>, and writes <views dir>/index.html (self-contained
apart from the PNGs beside it). Publish with tools/review/publish.sh p1-24 <views dir>.
"""

import html
import json
import sys
from pathlib import Path

VIEWS = [("screen", "Create World screen, last frame"), ("planet", "Whole planet (a sample every 16 blocks)"),
         ("ground", "Most rugged 1024 x 1024 blocks"), ("coast", "Most coastal 1024 x 1024 blocks")]


def main() -> int:
    runs, out = Path(sys.argv[1]), Path(sys.argv[2])
    specs = []
    for s in sys.argv[3:]:
        label, rest = s.split("=", 1)
        name, _, desc = rest.partition(":")
        specs.append((label, name, desc))
    views = json.loads((out / "views.json").read_text())
    base_wall = None
    rows, cards = [], []
    for label, name, desc in specs:
        r = json.loads((runs / name / "result.json").read_text())
        wall = r["wall_s"]
        base_wall = base_wall or wall
        v = views.get(label, {})
        g, c = v.get("ground", {}), v.get("coast", {})
        rows.append(f"<tr><td><b>{html.escape(label)}</b></td><td>{html.escape(desc)}</td>"
                    f"<td class=n>{wall:.0f}</td><td class=n>{base_wall / wall:.2f}×</td>"
                    f"<td class=n>{r.get('max_rss_gib')}</td><td class=n>{r.get('world_bytes', 0) / 1e6:.0f}</td>"
                    f"<td class=n>{v.get('land_share', '')}</td><td class=n>{g.get('relief_blocks', '')}</td>"
                    f"<td class=n>{g.get('valley_cells', '')}</td><td class=n>{c.get('valley_cells', '')}</td></tr>")
        imgs = "".join(
            f"<figure><a href='{label}-{k}.png'><img loading=lazy src='{label}-{k}.png' alt='{html.escape(t)}'></a>"
            f"<figcaption>{html.escape(t)}</figcaption></figure>"
            for k, t in VIEWS if (out / f"{label}-{k}.png").exists())
        cards.append(f"<section><h2>{html.escape(label)} <small>{html.escape(desc)} · {wall:.0f} s</small></h2>"
                     f"<div class=grid>{imgs}</div></section>")
    page = f"""<!doctype html><html lang=en><head><meta charset=utf-8>
<meta name=viewport content="width=device-width,initial-scale=1">
<title>Faster Create World</title>
<meta name="description" content="P1-24: what each faster quick mode candidate costs in planet quality, on a mid-range PC profile.">
<style>
:root{{--bg:#fafafa;--fg:#1c1c1f;--muted:#666;--card:#fff;--line:#ddd;--accent:#4c5fd7}}
@media (prefers-color-scheme:dark){{:root:not([data-theme=light]){{--bg:#15161a;--fg:#ececef;--muted:#9a9aa3;--card:#1e1f24;--line:#2d2e35;--accent:#8ea2ff}}}}
:root[data-theme=dark]{{--bg:#15161a;--fg:#ececef;--muted:#9a9aa3;--card:#1e1f24;--line:#2d2e35;--accent:#8ea2ff}}
body{{background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,sans-serif;margin:0 auto;max-width:1400px;padding:16px}}
h1{{margin:.2em 0}} h2{{font-size:1.1em;margin:1.2em 0 .4em}} small{{color:var(--muted);font-weight:normal}}
p{{max-width:75ch}} .wrap{{overflow-x:auto}}
table{{border-collapse:collapse;background:var(--card)}} td,th{{border:1px solid var(--line);padding:4px 8px;text-align:left}}
td.n{{text-align:right;font-variant-numeric:tabular-nums}}
.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:10px}}
figure{{margin:0;background:var(--card);border:1px solid var(--line);padding:6px}} img{{width:100%;height:auto;display:block}}
figcaption{{color:var(--muted);font-size:.85em}}
</style></head><body>
<h1>Faster Create World: the quick-mode candidates</h1>
<p>Each run is earthlike_quick_16k, scattered continents, 500 Myr, on the mid-range PC profile (6 cores / 12 threads,
11 Rayon threads, 6 GiB), with one change. A coarser grid or a longer step changes the whole history, so each candidate is a
different planet: compare the character (coast detail, ranges, valleys), not the places. The ground and coast windows are each
planet's own most rugged and most coastal 1024-block squares. Design note: <code>docs/design/faster-create-world.md</code>.</p>
<div class=wrap><table><tr><th>candidate</th><th>change</th><th>wall s</th><th>speed-up</th><th>max RSS GiB</th><th>world MB</th>
<th>land share</th><th>rugged window relief (blocks)</th><th>valley cells, rugged</th><th>valley cells, coast</th></tr>
{''.join(rows)}</table></div>
{''.join(cards)}
</body></html>"""
    (out / "index.html").write_text(page)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
