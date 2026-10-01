#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-17 hypsometry recalibration: hypsographic curves of runs against Earth's (an HTML page with
inline SVG, no dependencies). Run on the Mac (tools/remote/mac.sh) and pull the folder.

    p1-17-hyps-plot.py --earth build/earth/earth_720.f32 --out <dir> \
        before=<summary-base-500.json> after=<summary-final-500.json> [...]

Each `name=path` is a summary written by tools/review/p1-17-hyps.py; the first is drawn orange,
the second blue (at most two). Earth is the 720 x 360 f32 relief (north first, from 0 degrees E;
GMT earth_relief 30', SRTM15+), cos(lat)-weighted like the CLI's hypsometry line; its figures go
to <dir>/earth.json too.
"""
import array
import html
import json
import math
import os
import sys

W, H = 720, 360


def earth_cells(path: str):
    a = array.array("f")
    with open(path, "rb") as fh:
        a.frombytes(fh.read())
    if sys.byteorder != "little":
        a.byteswap()
    assert len(a) == W * H, len(a)
    cells = []
    for j in range(H):
        lat = 90.0 - (j + 0.5) * 180.0 / H
        w = math.cos(math.radians(lat))
        row = a[j * W:(j + 1) * W]
        cells.extend((float(z), w) for z in row)
    return cells


def quantiles(pairs, qs):
    pairs = sorted(pairs)
    tot = sum(w for _, w in pairs)
    out, acc, k = [], 0.0, 0
    for z, w in pairs:
        acc += w
        while k < len(qs) and acc >= qs[k] * tot:
            out.append(z)
            k += 1
    while len(out) < len(qs):
        out.append(pairs[-1][0])
    return out


def earth_stats(cells):
    tot = sum(w for _, w in cells)
    land = [(z, w) for z, w in cells if z > 0]
    sea = [(-z, w) for z, w in cells if z <= 0]
    la = sum(w for _, w in land)
    sa = sum(w for _, w in sea)
    lq = quantiles(land, [0.1, 0.25, 0.5, 0.75, 0.9])
    bands = [0.0] * 13
    for z, w in cells:
        b = min(max(int(math.floor(z / 1000.0)) + 7, 0), 12)
        bands[b] += w / tot
    curve_desc = sorted(cells, key=lambda x: -x[0])
    curve, acc, k = [], 0.0, 0
    for z, w in curve_desc:
        acc += w
        while k <= 100 and acc >= k / 100.0 * tot:
            curve.append(z)
            k += 1
    while len(curve) < 101:
        curve.append(curve_desc[-1][0])
    return {
        "land_frac_area": la / tot,
        "land_mean_m": sum(z * w for z, w in land) / la,
        "land_z_p10_m": lq[0], "land_z_p25_m": lq[1], "land_z_p50_m": lq[2], "land_z_p75_m": lq[3],
        "land_z_p90_m": lq[4],
        "land_above_1km": sum(w for z, w in land if z > 1000) / la,
        "land_above_2km": sum(w for z, w in land if z > 2000) / la,
        "land_above_3km": sum(w for z, w in land if z > 3000) / la,
        "sea_depth_p50_m": quantiles(sea, [0.5])[0],
        "sea_depth_mean_m": sum(z * w for z, w in sea) / sa,
        "shelf_frac_of_sea": sum(w for z, w in sea if z <= 200) / sa,
        "bands": bands,
        "curve": curve,
    }


def land_curve(curve):
    """Elevations of the land part of a 0..100 % curve against the share of the land (0..1)."""
    land = [z for z in curve if z > 0]
    n = len(land)
    if n < 2:
        return []
    return [(i / (n - 1), z) for i, z in enumerate(land)]


CSS = """
:root{--surface:#fcfcfb;--ink:#0b0b0b;--ink2:#52514e;--muted:#8a8984;--grid:#e6e5e1;--earth:#52514e;
--s1:#eb6834;--s2:#2a78d6;--band1:rgba(235,104,52,.12);--band2:rgba(42,120,214,.12);color-scheme:light}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--surface:#1a1a19;--ink:#fff;--ink2:#c3c2b7;
--muted:#8f8e86;--grid:#33332f;--earth:#c3c2b7;--s1:#d95926;--s2:#3987e5;--band1:rgba(217,89,38,.16);
--band2:rgba(57,135,229,.16);color-scheme:dark}}
:root[data-theme="dark"]{--surface:#1a1a19;--ink:#fff;--ink2:#c3c2b7;--muted:#8f8e86;--grid:#33332f;--earth:#c3c2b7;
--s1:#d95926;--s2:#3987e5;--band1:rgba(217,89,38,.16);--band2:rgba(57,135,229,.16);color-scheme:dark}
body{background:var(--surface);color:var(--ink);font:14px/1.45 system-ui,sans-serif;margin:0;padding:24px 16px}
main{max-width:1100px;margin:0 auto}h1{font-size:22px;margin:0 0 4px}h2{font-size:16px;margin:28px 0 6px}
p{color:var(--ink2);max-width:80ch}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(330px,1fr));gap:16px}
.panel{position:relative}.panel h3{font-size:13px;font-weight:600;margin:0 0 2px;color:var(--ink2)}
svg{display:block;width:100%;height:auto;overflow:visible}.axis text{fill:var(--muted);font-size:11px}
.axis line,.gridl{stroke:var(--grid);stroke-width:1}.zero{stroke:var(--muted);stroke-width:1}
.c{fill:none;stroke-width:2;stroke-linejoin:round;stroke-linecap:round}.earth{stroke:var(--earth)}
.s1{stroke:var(--s1)}.s2{stroke:var(--s2)}.b1{fill:var(--band1)}.b2{fill:var(--band2)}
.legend{display:flex;gap:18px;flex-wrap:wrap;margin:8px 0;color:var(--ink2)}.key{display:inline-block;width:18px;
height:2px;vertical-align:middle;margin-right:6px}.lab{fill:var(--ink2);font-size:11px}
.tip{position:absolute;pointer-events:none;background:var(--surface);border:1px solid var(--grid);padding:6px 8px;
font-size:12px;color:var(--ink);white-space:nowrap;border-radius:4px}.cross{stroke:var(--muted);stroke-width:1}
.tbl{overflow-x:auto}table{border-collapse:collapse;font-size:12px;font-variant-numeric:tabular-nums}
th,td{padding:3px 8px;border-bottom:1px solid var(--grid);text-align:right}th:first-child,td:first-child{text-align:left}
"""

JS = """
document.querySelectorAll('.panel[data-series]').forEach(p=>{
 const s=JSON.parse(p.dataset.series), svg=p.querySelector('svg'), tip=p.querySelector('.tip'),
  cross=svg.querySelector('.cross'), g=JSON.parse(p.dataset.geom);
 svg.addEventListener('mousemove',e=>{const r=svg.getBoundingClientRect(), k=g.w/r.width;
  const x=(e.clientX-r.left)*k; const t=(x-g.l)/(g.w-g.l-g.r); if(t<0||t>1){tip.hidden=true;cross.setAttribute('visibility','hidden');return}
  const lines=s.map(o=>{const i=Math.round(t*(o.v.length-1)); return `${o.n}: ${Math.round(o.v[i]).toLocaleString()} m`});
  tip.innerHTML=`<b>${(t*g.xmax).toFixed(0)} ${g.xunit}</b><br>`+lines.join('<br>'); tip.hidden=false;
  cross.setAttribute('x1',x);cross.setAttribute('x2',x);cross.setAttribute('visibility','visible');
  const pr=p.getBoundingClientRect(); let lx=e.clientX-pr.left+12; if(lx>pr.width-150) lx-=170; tip.style.left=lx+'px';
  tip.style.top=(e.clientY-pr.top+8)+'px'});
 svg.addEventListener('mouseleave',()=>{tip.hidden=true;cross.setAttribute('visibility','hidden')});
});
"""


def panel(title, series, bands=(), ymin=-8000, ymax=6000, xmax=100, xunit="% of the surface",
          land=False):
    """series: [(name, css class, values over 0..1 evenly)]; bands: [(css class, lo, hi)]."""
    w, h, l, r, t, b = 520, 300, 46, 12, 10, 30
    pw, ph = w - l - r, h - t - b

    def X(f):
        return l + f * pw

    def Y(z):
        z = min(max(z, ymin), ymax)
        return t + (ymax - z) / (ymax - ymin) * ph

    out = [f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{html.escape(title)}">']
    step = 2000 if ymax - ymin > 6000 else 1000
    z = math.ceil(ymin / step) * step
    out.append('<g class="axis">')
    while z <= ymax:
        out.append(f'<line class="gridl" x1="{l}" x2="{w - r}" y1="{Y(z):.1f}" y2="{Y(z):.1f}"/>'
                   f'<text x="{l - 6}" y="{Y(z) + 4:.1f}" text-anchor="end">{z / 1000:g} km</text>')
        z += step
    for k in range(0, 101, 20 if not land else 25):
        out.append(f'<text x="{X(k / 100):.1f}" y="{h - 10}" text-anchor="middle">{k * xmax // 100}</text>')
    out.append(f'<text x="{w - r}" y="{h}" text-anchor="end">{xunit}</text></g>')
    if ymin < 0 < ymax:
        out.append(f'<line class="zero" x1="{l}" x2="{w - r}" y1="{Y(0):.1f}" y2="{Y(0):.1f}"/>')
    for cls, lo, hi in bands:
        n = len(lo)
        pts = [f"{X(i / (n - 1)):.1f},{Y(v):.1f}" for i, v in enumerate(hi)]
        pts += [f"{X(i / (n - 1)):.1f},{Y(v):.1f}" for i, v in reversed(list(enumerate(lo)))]
        out.append(f'<polygon class="{cls}" points="{" ".join(pts)}"/>')
    for name, cls, vals in series:
        n = len(vals)
        if n < 2:
            continue
        pts = " ".join(f"{X(i / (n - 1)):.1f},{Y(v):.1f}" for i, v in enumerate(vals))
        out.append(f'<polyline class="c {cls}" points="{pts}"><title>{html.escape(name)}</title></polyline>')
    out.append(f'<line class="cross" x1="0" x2="0" y1="{t}" y2="{h - b}" visibility="hidden"/></svg>')
    data = json.dumps([{"n": n_, "v": [round(v) for v in vals]} for n_, _, vals in series if len(vals) > 1])
    geom = json.dumps({"w": w, "l": l, "r": r, "xmax": xmax, "xunit": xunit})
    return (f'<div class="panel" data-series=\'{html.escape(data)}\' data-geom=\'{geom}\'>'
            f'<h3>{html.escape(title)}</h3>{"".join(out)}<div class="tip" hidden></div></div>')


def resample(pairs, n=101):
    """Land curve [(share, z)] onto n even shares."""
    if not pairs:
        return []
    out = []
    for i in range(n):
        f = i / (n - 1)
        for k in range(1, len(pairs)):
            if pairs[k][0] >= f:
                (f0, z0), (f1, z1) = pairs[k - 1], pairs[k]
                out.append(z0 + (z1 - z0) * ((f - f0) / (f1 - f0) if f1 > f0 else 0))
                break
        else:
            out.append(pairs[-1][1])
    return out


def main() -> int:
    args = sys.argv[1:]
    earth_path = args[args.index("--earth") + 1]
    out_dir = args[args.index("--out") + 1]
    named = [a for a in args if "=" in a and not a.startswith("--")]
    os.makedirs(out_dir, exist_ok=True)
    e = earth_stats(earth_cells(earth_path))
    with open(os.path.join(out_dir, "earth.json"), "w") as fh:
        json.dump(e, fh)
    print("earth " + " ".join(f"{k}={v:.3f}" if isinstance(v, float) else "" for k, v in e.items()
                              if not isinstance(v, list)))
    print("earth bands_km[-7..6]=" + "/".join(f"{b:.3f}" for b in e["bands"]))
    sums = []
    for a in named[:2]:
        name, path = a.split("=", 1)
        with open(path) as fh:
            sums.append((name, json.load(fh)))
    cls = ["s1", "s2"]
    bandcls = ["b1", "b2"]
    keys = sorted({k for _, s in sums for k in s["runs"]})
    # Means and ranges over the runs.
    mean_series, bands, land_series, land_bands = [("Earth", "earth", e["curve"])], [], [], []
    land_series.append(("Earth", "earth", resample(land_curve(e["curve"]))))
    for i, (name, s) in enumerate(sums):
        cs = [v["curve"] for v in s["runs"].values() if "curve" in v]
        if not cs:
            continue
        m = [sum(c[k] for c in cs) / len(cs) for k in range(101)]
        mean_series.append((f"{name} (mean of {len(cs)})", cls[i], m))
        bands.append((bandcls[i], [min(c[k] for c in cs) for k in range(101)],
                      [max(c[k] for c in cs) for k in range(101)]))
        lcs = [resample(land_curve(c)) for c in cs]
        lcs = [c for c in lcs if c]
        lm = [sum(c[k] for c in lcs) / len(lcs) for k in range(101)]
        land_series.append((f"{name} (mean of {len(lcs)})", cls[i], lm))
        land_bands.append((bandcls[i], [min(c[k] for c in lcs) for k in range(101)],
                           [max(c[k] for c in lcs) for k in range(101)]))
    legend = ['<div class="legend"><span><i class="key" style="background:var(--earth)"></i>Earth (SRTM15+, '
              'GMT earth_relief 30′)</span>']
    for i, (name, _) in enumerate(sums):
        legend.append(f'<span><i class="key" style="background:var(--{cls[i]})"></i>{html.escape(name)}</span>')
    legend.append("</div>")
    parts = [f"<meta charset=\"utf-8\"><title>P1-17 hypsometry</title><style>{CSS}</style><main>",
             "<h1>P1-17 hypsometry recalibration</h1>",
             "<p>Hypsographic curves of earthlike_quick_16k planets at the end of their runs against Earth's: the "
             "elevation (relative to sea level) that each share of the surface stands above, true-area weighted. "
             "Bands are the range over the runs. Hover a panel for values.</p>", "".join(legend),
             '<div class="grid">',
             panel("The whole surface: mean of the runs", mean_series, bands),
             panel("Land only: elevation against the share of the land", land_series, land_bands,
                   ymin=0, ymax=5000, xunit="% of the land", land=True),
             "</div><h2>Per run (start, seed)</h2>", "".join(legend), '<div class="grid">']
    for key in keys:
        ser = [("Earth", "earth", e["curve"])]
        for i, (name, s) in enumerate(sums):
            v = s["runs"].get(key, {})
            if "curve" in v:
                ser.append((name, cls[i], v["curve"]))
        parts.append(panel(key.replace("_continents", "").replace("-", ", seed "), ser))
    parts.append("</div><h2>Numbers</h2><div class=\"tbl\"><table><tr><th>run</th><th>land</th><th>median m</th>"
                 "<th>p90 m</th><th>&gt;2 km</th><th>sea median m</th><th>T °C</th><th>glaciated</th>"
                 "<th>Köppen E</th><th>Köppen B</th></tr>")
    parts.append(f"<tr><td>Earth</td><td>{e['land_frac_area']:.3f}</td><td>{e['land_z_p50_m']:.0f}</td>"
                 f"<td>{e['land_z_p90_m']:.0f}</td><td>{e['land_above_2km']:.3f}</td><td>{e['sea_depth_p50_m']:.0f}"
                 "</td><td>14–15</td><td>0.10</td><td>0.13</td><td>0.30</td></tr>")
    for name, s in sums:
        for key in keys:
            v = s["runs"].get(key)
            if not v:
                continue

            def g(k, f):
                return f.format(v[k]) if k in v else "-"
            parts.append(f"<tr><td>{html.escape(name)} {key}</td><td>{g('land_frac_area', '{:.3f}')}</td>"
                         f"<td>{g('land_z_p50_m', '{:.0f}')}</td><td>{g('land_z_p90_m', '{:.0f}')}</td>"
                         f"<td>{g('land_above_2km', '{:.3f}')}</td><td>{g('sea_depth_p50_m', '{:.0f}')}</td>"
                         f"<td>{g('t_surface_c', '{:.1f}')}</td><td>{g('glaciated_land_frac', '{:.3f}')}</td>"
                         f"<td>{g('koppen_E', '{:.2f}')}</td><td>{g('koppen_B', '{:.2f}')}</td></tr>")
    parts.append(f"</table></div></main><script>{JS}</script>")
    with open(os.path.join(out_dir, "index.html"), "w") as fh:
        fh.write("\n".join(parts))
    print(f"wrote {out_dir}/index.html")
    return 0


if __name__ == "__main__":
    sys.exit(main())
