#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The end-of-run CO2 hunt's tables (2026-10-19): one row per run of tools/review/co2-ensemble.sh.

    python3 tools/review/co2-table.py <run dir> [label ...]

For each run: the end CO2 and the present-day factor, so the natural (unnudged) end CO2; the
global and surface temperatures; the land share (surface_final, in-game cells); the land above
2 km; the land under ice sheets at the natural end and after the factor; the carbon cycle's
fluxes at the end (degassing, weathering, in 10^12 mol C/yr; ridge and subduction, km2/yr). Then
each label's median and range, and the CO2 history (ppm every 100 Myr, natural: the series is
the clock's, the factor applies at the end only).
"""
import json
import pathlib
import statistics
import sys


def kv(line):
    out = {}
    for tok in line.split():
        if "=" in tok:
            k, v = tok.split("=", 1)
            out[k] = v
    return out


def num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return float("nan")


def parse(txt):
    clim, surf, series = {}, {}, []
    for line in txt.read_text(errors="replace").splitlines():
        for part in line.split(": ")[1:] if line.startswith("deeptime ") else [line]:
            part = part.strip()
            if part.startswith("climate_final="):
                clim = kv(part)
            elif part.startswith("surface_final="):
                surf = kv(part)
            elif part.startswith("carbon_series "):
                series = []
                for tok in part.split()[1:]:
                    t, vals = tok.split(":", 1)
                    series.append([num(t)] + [num(v) for v in vals.split("/")])
    return clim, surf, series


def main():
    root = pathlib.Path(sys.argv[1])
    labels = sys.argv[2:]
    rows = []
    for txt in sorted(root.glob("*.txt")):
        label = txt.name.split("-")[0]
        if labels and label not in labels:
            continue
        clim, surf, series = parse(txt)
        cj = txt.with_suffix(".climate.json")
        pres = {}
        if cj.exists():
            try:
                pres = json.loads(cj.read_text()).get("present", {})
            except json.JSONDecodeError:
                pass
        co2 = num(clim.get("co2_ppm"))
        fac = num(clim.get("present_co2_factor", pres.get("co2_factor", 1.0)))
        end = series[-1] if series else [float("nan")] * 17
        rows.append({
            "run": txt.stem,
            "label": label,
            "co2": co2,
            "factor": fac,
            "natural": co2 / fac if fac == fac and fac > 0 else float("nan"),
            "t_global": num(clim.get("ebm_t_global_c")),
            "t_surface": num(clim.get("t_surface_c")),
            "land": num(surf.get("land_frac")),
            "above2": num(clim.get("land_above_2km")),
            "nat_sheet": num(clim.get("present_natural_sheet_frac")),
            "sheet": num(clim.get("present_sheet_frac")),
            "reached": clim.get("present_reached", "?"),
            "degas": end[4],
            "weath": end[5],
            "ridge": end[15],
            "subd": end[16],
            "series": series,
            "trials": pres.get("trial_rows", {}).get("rows"),
            "by_height": pres.get("sheets_by_height"),
        })
    cols = ["co2", "factor", "natural", "t_global", "t_surface", "land", "above2", "nat_sheet",
            "sheet", "degas", "weath", "ridge", "subd"]
    print("run".ljust(44) + "".join(c.rjust(10) for c in cols) + "  reached")
    for r in rows:
        print(r["run"].ljust(44) + "".join(f"{r[c]:10.3f}" if abs(r[c]) < 100 else f"{r[c]:10.0f}"
                                          for c in cols) + "  " + r["reached"])
    print()
    for label in sorted({r["label"] for r in rows}):
        rs = [r for r in rows if r["label"] == label]
        print(f"{label}: {len(rs)} runs, median [min, max]")
        for c in cols:
            v = sorted(r[c] for r in rs if r[c] == r[c])
            if v:
                print(f"  {c:10s} {statistics.median(v):10.3f} [{v[0]:.3f}, {v[-1]:.3f}]")
    print()
    print("CO2 history, natural ppm (the clock's series), every 100 Myr from the start:")
    for r in rows:
        s = r["series"]
        if not s:
            continue
        t0 = s[0][0]
        picks, nxt = [], t0
        for row in s:
            if row[0] >= nxt - 1e-6:
                picks.append(row)
                nxt = (row[0] // 100.0) * 100.0 + 100.0
        if picks[-1] is not s[-1]:
            picks.append(s[-1])
        print("  " + r["run"].ljust(44) + " ".join(f"{p[0]:.0f}:{p[1]:.0f}" for p in picks))
    print()
    print("The carbon cycle every 100 Myr: t:co2 ppm/T global/degassing/weathering (10^12 mol/yr)"
          "/land share/land above 2 km/kernel_rel/ridge km2/yr:")
    for r in rows:
        s = r["series"]
        if not s:
            continue
        picks, nxt = [], s[0][0]
        for row in s:
            if row[0] >= nxt - 1e-6:
                picks.append(row)
                nxt = (row[0] // 100.0) * 100.0 + 100.0
        if picks[-1] is not s[-1]:
            picks.append(s[-1])
        print("  " + r["run"])
        print("    " + " ".join(f"{p[0]:.0f}:{p[1]:.0f}/{p[2]:.1f}/{p[4]:.2f}/{p[5]:.2f}/{p[13]:.3f}/{p[12]:.3f}"
                                f"/{p[9]:.2f}/{p[15]:.2f}" for p in picks))
    for r in rows:
        if r["trials"]:
            print(f"  trials {r['run']}: " + " ".join(
                f"x{t[0]:.2f}->sheet{t[1]:.3f}/T{t[4]:.1f}" for t in r["trials"]))
        if r["by_height"]:
            b = r["by_height"]
            print(f"  sheets by height {r['run']}: land " + "/".join(f"{x:.3f}" for x in b["natural_land_share"])
                  + " natural sheet " + "/".join(f"{x:.3f}" for x in b["natural_sheet_share"])
                  + " chosen sheet " + "/".join(f"{x:.3f}" for x in b["sheet_share"]))


if __name__ == "__main__":
    main()
