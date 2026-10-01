#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-27: the before/after evidence page (review/out/p1-27/evidence/index.html, not published):
the active and passive shares of the continental margins over each run (from the `margins` line's
series), the run summaries side by side, and the same keyframes of the regime, elevation,
provinces, satellite and volcano layers before and after.

    python3 tools/review/p1-27-page.py review/out/p1-27/evidence "<starts>"
"""
import html
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
starts = sys.argv[2].split()
LAYERS = ["regime", "elevation", "satellite", "provinces", "volcanoes_map"]
KEEP = {
    "margins records": ["active_mean", "passive_mean", "regime_active_mean", "passive_floor_age_p50_mean"],
    "provinces shield": ["shield", "platform", "orogen", "basin", "extended", "edge_density", "speck_share"],
    "hypsometry": ["land_frac_area", "land_z_p50_m", "land_above_2km"],
    "volcanoes_final": ["arc_base_p50_mean_m", "arc_base_low_share_mean", "active_strato"],
    "plates_final": ["plates_final", "ocean_age_p90_myr", "ocean_younger_250", "collisions", "initiations"],
}


def summary(text):
    out = {}
    for tag, keys in KEEP.items():
        line = next((l for l in text.splitlines() if tag in l), "")
        for k in keys:
            m = re.search(r"(?:^|\s)" + re.escape(k) + r"=([-0-9.eE+]+)", line)
            if m:
                out[k] = float(m.group(1))
    m = re.search(r"series\(t:active/passive\)=(\S+)", text)
    series = []
    if m:
        for item in m.group(1).split(","):
            t, ap = item.split(":")
            a, p = ap.split("/")
            series.append((float(t), float(a), float(p)))
    return out, series


def chart(runs):
    w, h, pad = 640, 220, 36
    ts = [t for _, s in runs for t, _, _ in s] or [0, 1]
    t0, t1 = min(ts), max(ts)
    x = lambda t: pad + (w - 2 * pad) * (t - t0) / max(t1 - t0, 1e-9)
    y = lambda v: h - pad - (h - 2 * pad) * v
    parts = [f'<svg viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img">']
    for v in (0, 0.25, 0.5, 0.75, 1.0):
        parts.append(f'<line x1="{pad}" x2="{w - pad}" y1="{y(v)}" y2="{y(v)}" class="grid"/>'
                     f'<text x="4" y="{y(v) + 4}" class="tick">{v:.2f}</text>')
    parts.append(f'<line x1="{pad}" x2="{w - pad}" y1="{y(0.3)}" y2="{y(0.3)}" class="earth"/>'
                 f'<text x="{w - pad - 150}" y="{y(0.3) - 4}" class="tick">Earth ≈ 0.3 active</text>')
    for label, s in runs:
        if not s:
            continue
        pts = " ".join(f"{x(t):.1f},{y(a):.1f}" for t, a, _ in s)
        parts.append(f'<polyline points="{pts}" class="{label}"/>')
    parts.append(f'<text x="{pad}" y="{h - 8}" class="tick">{t0:.0f} Myr</text>'
                 f'<text x="{w - pad - 40}" y="{h - 8}" class="tick">{t1:.0f} Myr</text></svg>')
    return "".join(parts)


body = []
for start in starts:
    runs, sums = [], {}
    for label in ("before", "after"):
        f = root / f"{label}-{start}.txt"
        if not f.exists():
            continue
        s, series = summary(f.read_text())
        sums[label] = s
        runs.append((label, series))
    body.append(f"<h2>{html.escape(start)}</h2>")
    body.append("<p>Share of the continental margin that is active (a subduction front within "
                "400 km, or a collision belt), every 10 Myr: <span class='before'>before</span> "
                "and <span class='after'>after</span>.</p>")
    body.append(chart(runs))
    keys = sorted({k for s in sums.values() for k in s}, key=lambda k: [k2 for ks in KEEP.values() for k2 in ks].index(k))
    body.append("<table><tr><th></th><th>before</th><th>after</th></tr>")
    for k in keys:
        cells = "".join(f"<td>{sums.get(l, {}).get(k, float('nan')):.3g}</td>" for l in ("before", "after"))
        body.append(f"<tr><th>{html.escape(k)}</th>{cells}</tr>")
    body.append("</table>")
    frames = sorted({int(m.group(1)) for p in (root / f"after-{start}" / "keyframes").glob("kf-*-regime.png")
                     for m in [re.match(r"kf-(\d+)-", p.name)] if m})
    for n in frames:
        body.append(f"<h3>{html.escape(start)}, keyframe {n}</h3><div class='grid2'>")
        for layer in LAYERS:
            for label in ("before", "after"):
                src = f"{label}-{start}/keyframes/kf-{n}-{layer}.png"
                if (root / src).exists():
                    body.append(f"<figure><img src='{src}' loading='lazy' alt=''><figcaption>{label} · {layer}</figcaption></figure>")
        body.append("</div>")

page = f"""<!doctype html><meta charset="utf-8"><title>P1-27 margins</title>
<style>
:root {{ color-scheme: light dark; --bg:#fff; --fg:#222; --muted:#888; }}
@media (prefers-color-scheme: dark) {{ :root {{ --bg:#16181d; --fg:#e6e6e6; --muted:#999; }} }}
body {{ background:var(--bg); color:var(--fg); font:14px system-ui, sans-serif; margin:0; padding:16px; }}
.grid2 {{ display:grid; grid-template-columns:repeat(2, minmax(0, 1fr)); gap:8px; }}
figure {{ margin:0; }} img {{ width:100%; image-rendering:pixelated; }}
figcaption, .tick {{ color:var(--muted); font-size:12px; fill:var(--muted); }}
svg .grid {{ stroke:var(--muted); stroke-opacity:.3; }} svg .earth {{ stroke:#2ca02c; stroke-dasharray:4 3; }}
svg polyline {{ fill:none; stroke-width:2; }} svg .before {{ stroke:#d62728; }} svg .after {{ stroke:#1f77b4; }}
span.before {{ color:#d62728; }} span.after {{ color:#1f77b4; }}
table {{ border-collapse:collapse; margin:8px 0 16px; }} td, th {{ padding:2px 10px; text-align:right; }}
</style>
<h1>P1-27: plate margins and coherent continents</h1>
<p>earthlike_quick_16k, seed 1; before = master's plate model with the P1-27 instruments, after = this branch.</p>
{''.join(body)}
"""
(root / "index.html").write_text(page)
print(root / "index.html")
