#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P2-01: the review page of a planet's folds and faults, from what tools/review/p2-01-report.sh
gathered (review/out/p2-01/: sections/, relief.png, report.txt).

    python3 tools/review/p2-01-page.py review/out/p2-01 review/out/p2-01-page

Writes <out>/index.html with the pictures beside it; publish with
`tools/review/publish.sh p2-01-structure <out>`.
"""
import html
import json
import shutil
import sys
from pathlib import Path


def main() -> None:
    src, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    report = {}
    for line in (src / "report.txt").read_text().splitlines():
        if " = " in line:
            k, v = line.split(" = ", 1)
            try:
                report[k.strip()] = json.loads(v)
            except ValueError:
                pass
    sections = json.loads((src / "sections/sections.json").read_text())
    for f in (src / "sections").glob("*.png"):
        shutil.copy(f, out / f.name)
    shutil.copy(src / "relief.png", out / "relief.png")

    st = report.get("deformation.structures", {})
    lat = report.get("deformation.lattice", {})
    chk = report.get("strata-check.structure", {})
    rel = report.get("amplify.structure_relief", {})
    hyp = {k.split("[")[1].rstrip("]"): v for k, v in report.items() if k.startswith("hypsometry[")}
    diff = report.get("difference", {})
    traps = report.get("deformation.traps", {})
    pct = lambda x: f"{100 * x:.1f} %"
    rows = []
    for name, h in hyp.items():
        rows.append(
            f"<tr><td>{'with the relief' if not rows else 'relief off'}</td><td>{h['land_share']:.4f}</td><td>{h['land_median_m']:.0f}</td>"
            f"<td>{h['land_p10_m']:.0f}</td><td>{h['land_p90_m']:.0f}</td><td>{h['land_p99_m']:.0f}</td>"
            f"<td>{h['land_mean_m']:.0f}</td></tr>")
    parts = [
        "<!doctype html><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'>",
        "<meta name='description' content='Folds, faults and intrusions of a Deep Time planet: sections before and after, the terrain with and without their relief, and the numbers.'>",
        "<title>Deep Time: folds and faults (P2-01)</title>",
        "<style>body{font:15px/1.5 system-ui,sans-serif;margin:16px;background:#14161b;color:#e6e6e6}"
        "img{max-width:100%;display:block;margin:4px 0 14px}.px{image-rendering:pixelated}h2{margin:28px 0 4px}"
        "p,li{max-width:72em;color:#c3c8d2}table{border-collapse:collapse;margin:8px 0}td,th{border:1px solid #444;padding:3px 9px;text-align:right}"
        "td:first-child,th:first-child{text-align:left}.k{display:inline-block;margin:2px 10px 2px 0}"
        ".s{display:inline-block;width:12px;height:12px;vertical-align:-1px;margin-right:4px}</style>",
        "<h1>Folds and faults</h1>",
        "<p>One planet (the quick 16,384-block preset, scattered continents, 500 million years). "
        f"It keeps {st.get('fold_train', 0):,} fold trains, {st.get('thrust', 0):,} thrusts, {st.get('normal_fault', 0):,} normal faults, "
        f"{st.get('strike_slip_fault', 0):,} strike-slip faults, {st.get('stock', 0):,} stocks, {st.get('dyke', 0):,} dykes and "
        f"{st.get('plug', 0):,} volcanic plugs. They change {pct(chk.get('land_changed_share', 0))} of land columns; "
        f"no column in 20,000 is malformed.</p>",
        "<h2>Sections</h2>",
        "<p>Each section is drawn twice in block space (one block is one square): the layers as they were laid down, "
        "then as the folds, faults and intrusions left them. Red lines are fault planes where they cut rock older than the fault. "
        "Rock younger than a structure lies undisturbed across it.</p>",
    ]
    for s in sections["sections"]:
        about = s.get("about") or {}
        when = ""
        if about:
            end = "today" if abs(about["to_myr"]) < 1 else f"{-about['to_myr']:.0f} million years ago"
            when = f" Its structures formed from {-about['from_myr']:.0f} million years ago to {end}."
        parts.append(
            f"<h3>{html.escape(s['title'])}</h3><p>{s['blocks']:.0f} blocks ({s['km']:.0f} km) from block "
            f"({s['from'][0]:.0f}, {s['from'][1]:.0f}) to ({s['to'][0]:.0f}, {s['to'][1]:.0f}), y {s['y0']} to {s['y1']}. "
            f"The structures change {s['columns_changed']} of its {s['columns']} columns.{when}</p>"
            f"<p>As laid down:</p><img class='px' src='{s['plain']}' alt='{html.escape(s['title'])}, before'>"
            f"<p>With the structures:</p><img class='px' src='{s['structure']}' alt='{html.escape(s['title'])}, after'>")
    parts.append("<p>" + "".join(
        f"<span class='k'><span class='s' style='background:{l['colour']}'></span>{html.escape(l['class'])}</span>"
        for l in sections["legend"]) + "</p>")
    parts += [
        "<h2>Terrain</h2>",
        f"<p>{rel.get('structures', 0)} young structures still show on the ground: fault scarps, the dropped blocks of rifts "
        "and ridges over fresh folds. The picture is the place where the terrain changes most, without and with them.</p>",
        "<img src='relief.png' alt='Hillshades without and with the structures, and their difference'>",
        "<table><tr><th>Baked terrain</th><th>land share</th><th>median m</th><th>p10</th><th>p90</th><th>p99</th><th>mean</th></tr>"
        + "".join(rows) + "</table>",
        f"<p>(The second row is the same planet with the relief setting at 0.) {pct(diff.get('cells_a_block_or_more_apart', 0))} of land "
        f"differs by a block or more, by {diff.get('mean_abs_m_where_apart', 0):.0f} m on average there.</p>",
        "<h2>Numbers</h2><ul>",
        f"<li>Columns the structures act on (L2 lattice): {pct(lat.get('deformed_frac', 0))}; crossed by a fault plane: "
        f"{pct(lat.get('faulted_frac', 0))}; the most a bed is shifted: {lat.get('max_shift_blocks', 0):.0f} blocks.</li>",
        f"<li>A bed's offset in the changed columns: median {chk.get('bed_offset_blocks', {}).get('median', 0)} blocks, "
        f"90th percentile {chk.get('bed_offset_blocks', {}).get('p90', 0)}.</li>",
        f"<li>Columns that meet a bed twice (a thrust carried it over itself): {chk.get('bed_repeated_columns', 0)} of 20,000; "
        f"columns through an intrusion: {chk.get('intruded_columns', 0)}.</li>",
        f"<li>Continental cells with structural closure of a half or more (petroleum traps): "
        f"{pct(traps.get('world', {}).get('closure_at_least_half', 0))} from the world's own folds and faults; "
        f"{pct(traps.get('regime_rule_block_km', {}).get('closure_at_least_half', 0))} by the old rule.</li>",
        "</ul>",
    ]
    (out / "index.html").write_text("\n".join(parts))
    print(out / "index.html")


if __name__ == "__main__":
    main()
