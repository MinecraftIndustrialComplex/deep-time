#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-26 rain review page: Earth's relief under each rain model against Beck et al.'s observations.

    python3 tools/review/p1-26-rain-page.py OUT OBS_DIR label=DUMP_DIR[,METRICS_JSON] ... \
        [--ensemble ENSEMBLE_SUMMARY.txt] [--note NOTE.html]

DUMP_DIR is a `deeptime climate-dem --dump` folder, METRICS_JSON the line
`tools/validation/climate_vs_obs.py --obs` printed for it, OBS_DIR the observed grids that script
reads (Beck 2018 Köppen, Beck 2023 monthly T and P at 0.5°; CC BY 4.0, validation only: the
page shows them resampled onto the model's grid, nothing else of them). Writes OUT/index.html and
its PNGs (numpy and Pillow only). The page goes up with tools/review/publish.sh.
"""
import html
import json
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "validation"))
import climate_vs_obs as cvo  # noqa: E402

# Beck et al. 2018's legend colours, by class (display only).
KG_RGB = {
    "Af": (0, 0, 255), "Am": (0, 120, 255), "Aw": (70, 170, 250),
    "BWh": (255, 0, 0), "BWk": (255, 150, 150), "BSh": (245, 165, 0), "BSk": (255, 220, 100),
    "Csa": (255, 255, 0), "Csb": (200, 200, 0), "Csc": (150, 150, 0),
    "Cwa": (150, 255, 150), "Cwb": (100, 200, 100), "Cwc": (50, 150, 50),
    "Cfa": (200, 255, 80), "Cfb": (100, 255, 80), "Cfc": (50, 200, 0),
    "Dsa": (255, 0, 255), "Dsb": (200, 0, 200), "Dsc": (150, 50, 150), "Dsd": (150, 100, 150),
    "Dwa": (170, 175, 255), "Dwb": (90, 120, 220), "Dwc": (75, 80, 180), "Dwd": (50, 0, 135),
    "Dfa": (0, 255, 255), "Dfb": (55, 200, 255), "Dfc": (0, 125, 125), "Dfd": (0, 70, 95),
    "ET": (178, 178, 178), "EF": (102, 102, 102),
}
SEA_RGB = (225, 232, 240)
NODATA_RGB = (255, 255, 255)

# Annual precipitation: log scale from 30 to 4,000 mm, dry brown through green to deep blue.
P_STOPS = [(30, (140, 81, 10)), (100, (216, 179, 101)), (250, (246, 232, 195)), (500, (199, 234, 229)),
           (1000, (90, 180, 172)), (2000, (1, 102, 94)), (4000, (0, 40, 90))]


def ramp(v, stops):
    x = np.log(np.clip(v, stops[0][0], stops[-1][0]))
    xs = np.log([s[0] for s in stops])
    out = np.zeros(v.shape + (3,), np.uint8)
    for k in range(3):
        out[..., k] = np.interp(x, xs, [s[1][k] for s in stops]).astype(np.uint8)
    return out


def diverging(d, lim=1.5):
    """ln(model/observed): brown (too dry) through white to blue (too wet)."""
    t = np.clip(d / lim, -1, 1)
    out = np.empty(d.shape + (3,), np.float64)
    dry, wet, mid = np.array([140, 81, 10]), np.array([1, 80, 160]), np.array([248, 248, 248])
    neg = t < 0
    out[neg] = mid + (dry - mid) * (-t[neg])[:, None]
    out[~neg] = mid + (wet - mid) * t[~neg][:, None]
    return out.astype(np.uint8)


def save(rgb, path, scale=1):
    im = Image.fromarray(rgb, "RGB")
    if scale != 1:
        im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
    im.save(path, optimize=True)


def load(d):
    meta = json.load(open(os.path.join(d, "meta.json")))
    w, h = meta["w"], meta["h"]
    rd = lambda n: np.fromfile(os.path.join(d, n + ".f32"), dtype="<f4").reshape(h, w)
    k = rd("koppen").astype(int)
    names = np.array(meta["koppen_codes"], dtype=object)[np.clip(k, 0, len(meta["koppen_codes"]) - 1)]
    return meta, w, h, rd("precip"), names, k != cvo.SEA


def obs_on(meta, w, h, od):
    kg, _t, p = cvo.load_obs(od)
    lat1 = np.array(meta["lat"], dtype=float)
    lon = (np.arange(w) + 0.5) / w * 360.0
    lon = np.where(lon >= 180.0, lon - 360.0, lon)
    io = np.clip(((lon + 180.0) / 0.5).astype(int), 0, 719)
    jo = np.clip(((90.0 - lat1) / 0.5).astype(int), 0, 359)
    kgo = kg[jo][:, io]
    po = np.nansum(p[:, jo][:, :, io], axis=0)
    names = np.array([""] + cvo.BECK_CODES, dtype=object)[np.clip(kgo, 0, 30)]
    return po, names, kgo > 0


def kg_rgb(names, land):
    out = np.empty(names.shape + (3,), np.uint8)
    out[:] = SEA_RGB
    for n, c in KG_RGB.items():
        out[land & (names == n)] = c
    return out


def p_rgb(p, land):
    out = ramp(np.maximum(p, 1.0), P_STOPS)
    out[~land] = SEA_RGB
    return out


def legend_html():
    cells = "".join(
        f'<span class="kg"><i style="background:rgb{c}"></i>{n}</span>' for n, c in KG_RGB.items()
    )
    ps = "".join(
        f'<span class="kg"><i style="background:rgb{c}"></i>{v:,}</span>' for v, c in P_STOPS
    )
    return f'<p class="legend">Köppen: {cells}</p><p class="legend">Annual rain, mm: {ps}</p>'


ROWS = [
    ("Earth's land rain on the common land, mm (observed {obs:.0f})", "land_p_mm_on_obs", "{:.0f}"),
    ("ln annual rain over land, RMSE against observed (target ≤ 0.85)", "ln_p_rmse", "{:.2f}"),
    ("ln annual rain, bias", "ln_p_bias", "{:+.2f}"),
    ("ln annual rain, η² by latitude (observed {eta:.2f})", "eta2_ln_precip", "{:.2f}"),
    ("Köppen vs Beck 2018: exact class", "koppen_exact", "{:.1%}"),
    ("Köppen vs Beck 2018: main group", "koppen_group", "{:.1%}"),
    ("Reference cities in the right main group, of 24 (target ≥ 21)", "places_group", "{}"),
    ("Mumbai, mm (observed 3,096; target ≥ 1,500)", "mumbai_mm", "{:.0f}"),
    ("Dhaka, mm (observed 2,194; target ≥ 1,200)", "dhaka_mm", "{:.0f}"),
    ("Sahara 20–28° N, mm (observed 11; target ≤ 80)", "sahara_mm", "{:.0f}"),
    ("Atacama, mm (observed 16; target ≤ 80)", "atacama_mm", "{:.0f}"),
    ("Winter along-row spread of land T at sea level, K (observed {obs_std:.2f} on this grid)",
     "along_row_std_winter", "{:.2f}"),
    ("Monthly land T, RMSE against observed, K", "t_month_rmse", "{:.2f}"),
    ("Global mean, °C (target 14.1 ± 0.4)", "t_global_c", "{:.2f}"),
]


def main():
    args = sys.argv[1:]
    ens, note = None, None
    if "--ensemble" in args:
        i = args.index("--ensemble")
        ens = args[i + 1]
        del args[i : i + 2]
    if "--note" in args:
        i = args.index("--note")
        note = args[i + 1]
        del args[i : i + 2]
    out, od, runs = args[0], args[1], args[2:]
    os.makedirs(out, exist_ok=True)
    loaded = []
    for r in runs:
        label, rest = r.split("=", 1)
        parts = rest.split(",")
        met = json.load(open(parts[1])) if len(parts) > 1 else {}
        loaded.append((label, load(parts[0]), met))
    meta, w, h, _, _, land0 = loaded[0][1]
    po, ko, lando = obs_on(meta, w, h, od)
    save(kg_rgb(ko, land0 & lando), os.path.join(out, "koppen-observed.png"))
    save(p_rgb(po, land0 & lando), os.path.join(out, "rain-observed.png"))
    figs = []
    for label, (m, _w, _h, p, names, land), met in loaded:
        both = land & lando
        save(kg_rgb(names, land), os.path.join(out, f"koppen-{label}.png"))
        save(p_rgb(p, land), os.path.join(out, f"rain-{label}.png"))
        d = np.log(np.maximum(p, 1.0)) - np.log(np.maximum(po, 1.0))
        rgb = diverging(d)
        rgb[~both] = SEA_RGB
        save(rgb, os.path.join(out, f"rain-ratio-{label}.png"))
        figs.append(label)
    # The numbers table.
    first = loaded[0][2]
    head = "".join(f"<th>{html.escape(l)}</th>" for l, _, _ in loaded)
    body = []
    for text, key, fmt in ROWS:
        t = text.format(obs=first.get("obs_land_p_mm", float("nan")),
                        eta=first.get("eta2_ln_p_obs", float("nan")),
                        obs_std=first.get("along_row_std_winter_obs", float("nan")))
        cells = "".join(
            f"<td>{fmt.format(met[key]) if met.get(key) is not None else '–'}</td>" for _, _, met in loaded
        )
        body.append(f"<tr><th>{html.escape(t)}</th>{cells}</tr>")
    regions = sorted({n for _, _, met in loaded for n in (met.get("regions") or {})})
    reg_rows = []
    for n in regions:
        obs = next((met["regions"][n][1] for _, _, met in loaded if n in (met.get("regions") or {})), None)
        cells = "".join(
            f"<td>{(met.get('regions') or {}).get(n, ['–'])[0]}</td>" for _, _, met in loaded
        )
        reg_rows.append(f"<tr><th>{n.replace('_', ' ')}</th><td>{obs}</td>{cells}</tr>")
    sections = []
    for label in figs:
        sections.append(
            f'<h3>{html.escape(label)}</h3><div class="row">'
            f'<figure><img src="koppen-{label}.png"><figcaption>Köppen</figcaption></figure>'
            f'<figure><img src="rain-{label}.png"><figcaption>Annual rain</figcaption></figure>'
            f'<figure><img src="rain-ratio-{label}.png"><figcaption>ln(model / observed) rain: '
            f'brown too dry, blue too wet (±1.5 = ×4.5)</figcaption></figure></div>'
        )
    ens_html = ""
    if ens and os.path.exists(ens):
        ens_html = f"<h2>12-run ensemble (planets, not Earth)</h2><pre>{html.escape(open(ens).read())}</pre>"
    note_html = open(note).read() if note and os.path.exists(note) else ""
    proj = meta.get("projection") or "rows from north to south, as the run's meta.json lists their latitudes"
    page = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Deep Time rain review</title>
<meta name="description" content="P1-26 rain on Earth's relief: Smith and Barstad with an upwind moisture budget against the belts and Beck et al.'s observations.">
<style>
:root {{ --bg:#fbfbfa; --fg:#1d1d1b; --mut:#666; --line:#ddd; }}
@media (prefers-color-scheme: dark) {{ :root {{ --bg:#161616; --fg:#eee; --mut:#aaa; --line:#333; }} }}
body {{ background:var(--bg); color:var(--fg); font:15px/1.5 system-ui, sans-serif; margin:0 auto; max-width:1500px; padding:16px; }}
table {{ border-collapse:collapse; margin:8px 0 20px; }}
th, td {{ border-bottom:1px solid var(--line); padding:4px 10px; text-align:right; }}
th:first-child {{ text-align:left; font-weight:normal; }}
.row {{ display:flex; flex-wrap:wrap; gap:12px; }}
figure {{ margin:0; flex:1 1 440px; }}
figure img {{ width:100%; image-rendering:pixelated; border:1px solid var(--line); }}
figcaption, .legend {{ color:var(--mut); font-size:13px; }}
.kg {{ display:inline-block; margin-right:8px; white-space:nowrap; }}
.kg i {{ display:inline-block; width:12px; height:12px; margin-right:3px; vertical-align:-1px; }}
pre {{ overflow-x:auto; font-size:12px; }}
</style></head><body>
<h1>Deep Time rain review (P1-26 T9 and T10)</h1>
<p>Earth's relief (GMT earth_relief_30m_p) run through <code>deeptime climate-dem</code> on the planet's
grid ({w} × {h} cells, {proj}), each rain model against Beck et al. 2018's Köppen classes and Beck et al. 2023's
1961–1990 monthly temperature and rain (CC BY 4.0), resampled onto the same grid. The maps are drawn on that
grid as it is, one pixel per cell.</p>
{note_html}
<h2>Numbers</h2>
<table><tr><th></th>{head}</tr>{''.join(body)}</table>
<h2>Regional land rain, mm (observed, then each model)</h2>
<table><tr><th></th><th>observed</th>{head}</tr>{''.join(reg_rows)}</table>
<h2>Maps</h2>
{legend_html()}
<h3>Observed</h3><div class="row">
<figure><img src="koppen-observed.png"><figcaption>Köppen, Beck et al. 2018</figcaption></figure>
<figure><img src="rain-observed.png"><figcaption>Annual rain, Beck et al. 2023 (1961–1990)</figcaption></figure></div>
{''.join(sections)}
{ens_html}
</body></html>
"""
    open(os.path.join(out, "index.html"), "w").write(page)
    print(os.path.join(out, "index.html"))


if __name__ == "__main__":
    main()
