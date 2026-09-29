#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Runs every editor knob at both ends of its range on a small planet (settings audit).

The editor catalogue (crates/deeptime-viewer/assets/editor_fields.json) says how far a player may
turn each knob; the core's validation only says what it refuses outright. This script checks that
a knob at either end still gives a planet: for every catalogue field the pipeline preset carries
(numbers at min and max, choices at each option) it runs `deeptime stage <stage>` on a short
history and reads the stage summary lines back. A run fails when the process fails, panics, prints
NaN or inf, leaves the mass-balance residual above 1e-2 km3, or leaves a sink in the surface,
young landscape or amplifier; it is *flagged* when the planet degenerates (almost no land or
almost no sea), which a knob at the end of its range may do on purpose.

    python3 tools/dev/knob_extremes.py --bin target/release/deeptime --out build/knobs \\
        [--preset pipeline_quick_16k] [--length-myr 100] [--start scattered_continents] \\
        [--jobs 3] [--threads 4] [--only id,id] [--skip id,id] [--stage amplify]

The bake's knobs (the `biomes` section: the climate to biome-input mapping) act only when the world
data is written, after the amplify stage. `--stage run` runs `deeptime run` (every stage and the
bake) instead, reads `notes.bake.climate` back from the manifest and checks the five biome
inputs' land shares (finite, adding up to 1); `--preset` may then name a preset file (a small
planet such as crates/deeptime-core/tests/presets/planet_world_4k.json, given a `biomes` section:
a knob is run only where the preset carries its path):

    python3 tools/dev/knob_extremes.py --bin target/release/deeptime --out build/knobs-biomes \\
        --stage run --preset build/planet_world_4k_biomes.json --length-myr 150 --only inland_mid,inland_far,inland_interior,wetland_humidity,wetland_flat_share,wetland_lowland

stdlib only (Python 3.9-safe: the Mac's /usr/bin/python3). Writes <out>/<id>.<end>.log per run and
<out>/summary.json and <out>/summary.md. Exit status 1 when any run failed.
"""

import argparse
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CATALOGUE = os.path.join(REPO, "crates", "deeptime-viewer", "assets", "editor_fields.json")
PRESETS = os.path.join(REPO, "crates", "deeptime-core", "presets", "pipeline")


def get(obj, path):
    for part in path.split("."):
        if isinstance(obj, list):
            try:
                obj = obj[int(part)]
            except (ValueError, IndexError):
                return None
        elif isinstance(obj, dict) and part in obj:
            obj = obj[part]
        else:
            return None
    return obj


def put(obj, path, value):
    parts = path.split(".")
    for part in parts[:-1]:
        obj = obj[int(part)] if isinstance(obj, list) else obj[part]
    last = parts[-1]
    if isinstance(obj, list):
        obj[int(last)] = value
    else:
        obj[last] = value


def metrics(text):
    """The numbers worth reading from a stage's summary lines."""
    out = {}
    for key, pat in [
        ("land_frac", r"hypsometry land_frac_area=([\d.]+)"),
        ("land_p50_m", r"hypsometry .*?land_z_p50_m=(-?\d+)"),
        ("land_p90_m", r"hypsometry .*?land_z_p90_m=(-?\d+)"),
        ("sea_p50_m", r"sea_depth_p50_m=(-?\d+)"),
        ("cont_frac", r"continental_frac=([\d.]+)"),
        ("max_elev_m", r"surface_final=\d+ land_frac=[\d.]+ max_elevation_m=(-?\d+)"),
        ("mass_residual_km3", r"mass_residual_km3=(-?[\d.]+e?-?\d*)"),
        ("t_land_c", r"t_land_c=(-?[\d.]+)"),
        ("precip_land_mm", r"precip_land_mm=(\d+)"),
        ("glaciated", r"glaciated_land_frac=([\d.]+)"),
        ("plutons", r"pluton_bodies=(\d+)"),
        ("plates", r"plates_final=(\d+)"),
        ("sites", r"stage \w+: steps=\d+ t_years=\S+ events=\d+ \(\+\d+\) sites=(\d+)"),
        ("amp_change_m", r"mean_abs_change_m=([\d.]+)"),
        ("eroded_km3", r"amplify_final=.*?eroded_km3=([\d.e+]+)"),
    ]:
        m = re.search(pat, text)
        if m:
            out[key] = float(m.group(1)) if "." in m.group(1) or "e" in m.group(1) else int(m.group(1))
    sinks = [int(x) for x in re.findall(r"(?:surface_final|young_final|amplify_final)=[^\n]*?\bsinks=(\d+)", text)]
    out["sinks"] = sum(sinks) if sinks else None
    m = re.search(r"horizon_check=(\w+)", text)
    out["horizon_check"] = m.group(1) if m else None
    return out


def bake_metrics(world):
    """The biome mapping's own record, from the written world's manifest (`--stage run`)."""
    out = {}
    try:
        with open(os.path.join(world, "manifest.json")) as f:
            notes = json.load(f)["notes"]["bake"]["climate"]
    except (OSError, KeyError, ValueError):
        return out
    out["wetland_cells"] = notes.get("wetland_cells")
    out["land_cells_l2"] = notes.get("land_cells_l2")
    shares = notes.get("land_shares", {})
    out["share_sums_ok"] = all(
        isinstance(v, list) and v and all(isinstance(x, (int, float)) and x == x for x in v) and abs(sum(v) - 1.0) < 0.02
        for v in shares.values()) and len(shares) == 5
    for layer, key in (("erosion", "ero_e6"), ("temperature", "temp_top"), ("vegetation", "veg_top")):
        v = shares.get(layer)
        if isinstance(v, list) and v:
            out[key] = v[6] if layer == "erosion" else v[-1]
    cont = shares.get("continents")
    if isinstance(cont, list):
        out["cont_shares"] = "/".join("%.3f" % x for x in cont)
    out["placement"] = notes.get("placement", {}).get("source")
    return out


def verdict(code, text, m):
    fails, flags = [], []
    if code != 0:
        fails.append("exit %d" % code)
    if re.search(r"panicked|RUST_BACKTRACE", text):
        fails.append("panic")
    if re.search(r"\b(nan|inf)\b", text, re.I) and not re.search(r"\binformation\b", text, re.I):
        # `inf` as a whole word in a summary line means a non-finite value.
        if re.search(r"[=:\s](-?nan|-?inf)\b", text, re.I):
            fails.append("non-finite value")
    if abs(m.get("mass_residual_km3") or 0.0) > 1e-2:
        fails.append("mass residual %g km3" % m["mass_residual_km3"])
    if m.get("sinks"):
        fails.append("%d sinks" % m["sinks"])
    if m.get("horizon_check") not in (None, "ok"):
        fails.append("horizon_check=%s" % m["horizon_check"])
    if code == 0 and not m:
        fails.append("no summary lines")
    if m.get("share_sums_ok") is False:
        fails.append("the biome inputs' land shares are not finite or do not add up to 1")
    if m.get("bake_missing"):
        fails.append("no notes.bake.climate in the manifest")
    lf = m.get("land_frac")
    if lf is not None and lf < 0.05:
        flags.append("almost no land (%.3f)" % lf)
    if lf is not None and lf > 0.75:
        flags.append("almost no sea (land %.3f)" % lf)
    return fails, flags


def run_one(job, args, base_preset):
    fid, end, path, value = job
    preset = json.loads(json.dumps(base_preset))
    put(preset, path, value)
    tag = "%s.%s" % (fid, end)
    pfile = os.path.join(args.out, tag + ".preset.json")
    with open(pfile, "w") as f:
        json.dump(preset, f)
    if args.stage == "run":
        cmd = [args.bin, "run", "--preset", pfile, "--start", args.start,
               "--length-myr", str(args.length_myr), "--threads", str(args.threads),
               "--out", os.path.join(args.out, tag + ".dtcp")]
    else:
        cmd = [args.bin, "stage", args.stage, "--preset", pfile, "--start", args.start,
               "--length-myr", str(args.length_myr), "--threads", str(args.threads),
               "--out", os.path.join(args.out, tag + ".dtcp"), "--work", os.path.join(args.out, tag + ".work")]
    t0 = time.time()
    p = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True)
    secs = time.time() - t0
    text = p.stdout
    with open(os.path.join(args.out, tag + ".log"), "w") as f:
        f.write(text)
    m = metrics(text)
    if args.stage == "run":
        b = bake_metrics(os.path.join(args.out, tag + ".dtcp"))
        m.update(b)
        if not b:
            m["bake_missing"] = True
    # The checkpoint and work folder are large: keep only the log.
    subprocess.run(["rm", "-rf", os.path.join(args.out, tag + ".dtcp"), os.path.join(args.out, tag + ".work")])
    fails, flags = verdict(p.returncode, text, m)
    return {"id": fid, "end": end, "path": path, "value": value, "seconds": round(secs, 1),
            "metrics": m, "fails": fails, "flags": flags}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--preset", default="pipeline_quick_16k")
    ap.add_argument("--length-myr", type=float, default=100.0)
    ap.add_argument("--start", default="scattered_continents")
    ap.add_argument("--stage", default="amplify")
    ap.add_argument("--jobs", type=int, default=3)
    ap.add_argument("--threads", type=int, default=4)
    ap.add_argument("--only", default="")
    ap.add_argument("--skip", default="seed")
    ap.add_argument("--default-run", action="store_true", help="also run the unedited preset")
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)
    args.bin = os.path.abspath(args.bin)
    named = os.path.join(PRESETS, args.preset + ".json")
    base = json.load(open(args.preset if args.preset.endswith(".json") and os.path.exists(args.preset) else named))
    cat = json.load(open(CATALOGUE))
    only = set(x for x in args.only.split(",") if x)
    skip = set(x for x in args.skip.split(",") if x)
    jobs = []
    if args.default_run:
        jobs.append(("default", "-", "seed", base["seed"]))
    for f in cat["fields"]:
        if f.get("phase3") or (only and f["id"] not in only) or f["id"] in skip:
            continue
        cur = get(base, f["path"])
        if cur is None:
            continue
        if f["kind"] == "number":
            for end, v in (("min", f["min"]), ("max", f["max"])):
                jobs.append((f["id"], end, f["path"], int(v) if f.get("integer") else float(v)))
        elif f["kind"] == "choice" and f.get("options"):
            for o in f["options"]:
                jobs.append((f["id"], o, f["path"], o))
    print("%d runs of %s, %g Myr, %d at a time" % (len(jobs), args.stage, args.length_myr, args.jobs), flush=True)
    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = [ex.submit(run_one, j, args, base) for j in jobs]
        for fu in concurrent.futures.as_completed(futs):
            r = fu.result()
            results.append(r)
            print("%-22s %-7s %5.0fs %s %s" % (r["id"], r["end"], r["seconds"],
                  "FAIL " + "; ".join(r["fails"]) if r["fails"] else "ok",
                  ("[" + "; ".join(r["flags"]) + "]") if r["flags"] else ""), flush=True)
    order = {(j[0], j[1]): i for i, j in enumerate(jobs)}
    results.sort(key=lambda r: order[(r["id"], r["end"])])
    with open(os.path.join(args.out, "summary.json"), "w") as f:
        json.dump(results, f, indent=1)
    cols = (["wetland_cells", "cont_shares", "ero_e6", "temp_top", "veg_top", "land_cells_l2"] if args.stage == "run" else []) + ["land_frac", "land_p50_m", "land_p90_m", "sea_p50_m", "cont_frac", "t_land_c", "precip_land_mm",
            "glaciated", "plutons", "plates", "sites", "amp_change_m", "mass_residual_km3"]
    with open(os.path.join(args.out, "summary.md"), "w") as f:
        f.write("| knob | end | value | s | " + " | ".join(cols) + " | verdict |\n")
        f.write("|" + "---|" * (len(cols) + 5) + "\n")
        for r in results:
            cells = [(("%g" % r["metrics"][c]) if isinstance(r["metrics"][c], (int, float)) else str(r["metrics"][c]))
                     if c in r["metrics"] else "" for c in cols]
            v = "FAIL: " + "; ".join(r["fails"]) if r["fails"] else "ok" + (" (" + "; ".join(r["flags"]) + ")" if r["flags"] else "")
            f.write("| %s | %s | %s | %.0f | %s | %s |\n" % (r["id"], r["end"], r["value"], r["seconds"], " | ".join(cells), v))
    bad = [r for r in results if r["fails"]]
    print("%d of %d runs failed" % (len(bad), len(results)))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
