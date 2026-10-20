#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Rare-biome shares of a baked planet against a normal world of the same pack (ice spikes report, 2026-10-20).

    tools/review/ice_spikes.py --layers <name>-layers.npy --mapping <climate_mapping.json>
        --params <reference_biomes-parameters.json | Terralith jar> [--census <biomemap.json>]
        [--reference <label>=<reference_biomes.json>]... [--sea-y 63]

Reads the planet's baked biome inputs (`deeptime heightmap` with climate/temperature, vegetation, continents,
erosion, ridges, koppen_l2 or koppen, ice_cover), the world's climate mapping (the five tables the game applies)
and a multi-noise parameter list (the one `/deeptime probe reference_biomes` writes beside its result, or a pack
jar's overworld.json). For every land column it answers the climate search as the game does at the surface
(mapping, then the nearest parameter entry at depth 0, the volcanic biomes left out; rivers, the volcanic layer and
the biome rules are not modelled), and prints:

  * the land's frozen (T0) and dry (H0) shares and the weirdness sign there, by Koppen group
    (EF under ice, EF bare, ET, BWk, other land under ice, the rest);
  * where the ice spikes come from (each group's share of them and its own rate);
  * each watched biome's share of the land against each reference (`/deeptime probe reference_biomes` on a
    server of that pack: its own router and parameter list, several seeds), and the frozen and dry corner's
    ice spikes rate against the reference's.

Land: a column of a land class above the sea whose biome is neither an ocean nor a river (the reference's rule).
With --census (the game's `/deeptime probe biomes` grid of the same world, same step), the emulation's agreement
with the game is printed and the census's own shares are listed too.
"""
import argparse
import base64
import glob
import json
import struct
import zipfile
from collections import Counter

import numpy as np

PL = "data/minecraft/worldgen/multi_noise_biome_source_parameter_list/overworld.json"
KEYS = ("temperature", "humidity", "continentalness", "erosion", "weirdness", "depth")
CODES = ["Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc", "Cfa",
         "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb", "Dfc", "Dfd", "ET", "EF", "sea"]
EF, ET, BWK, SEA = 30, 29, 5, 31
VOLCANIC = {"terralith:yellowstone", "terralith:volcanic_peaks", "terralith:volcanic_crater", "terralith:caldera",
            "terralith:ashen_savanna", "terralith:basalt_cliffs"}
# Vanilla's band edges (OverworldBiomeBuilder): frozen below -0.45, driest humidity below -0.35.
T0_TOP, H0_TOP = -0.45, -0.35
WATCH = ["minecraft:ice_spikes", "minecraft:snowy_plains", "minecraft:snowy_slopes", "minecraft:frozen_peaks",
         "minecraft:jagged_peaks", "minecraft:grove", "minecraft:snowy_taiga", "minecraft:snowy_beach",
         "minecraft:mushroom_fields", "minecraft:cherry_grove", "minecraft:badlands", "minecraft:eroded_badlands",
         "minecraft:wooded_badlands", "minecraft:sunflower_plains", "minecraft:flower_forest", "minecraft:bamboo_jungle",
         "minecraft:sparse_jungle", "minecraft:old_growth_birch_forest", "minecraft:old_growth_spruce_taiga",
         "minecraft:windswept_gravelly_hills", "minecraft:windswept_savanna", "minecraft:meadow", "minecraft:pale_garden"]


def rng(v):
    if isinstance(v, (int, float)):
        return (float(v), float(v))
    if isinstance(v, list):
        return (float(v[0]), float(v[1]))
    return (float(v["min"]), float(v["max"]))


def load_entries(path):
    if path.endswith(".jar"):
        pl = json.loads(zipfile.ZipFile(glob.glob(path)[0]).read(PL))
    else:
        pl = json.load(open(path))
    entries = pl.get("lithostitched:biomes") or pl.get("biomes")
    names, lo, hi, off = [], [], [], []
    for e in entries:
        if e["biome"] in VOLCANIC:
            continue
        p = e["parameters"]
        names.append(e["biome"])
        r = [rng(p[k]) for k in KEYS]
        lo.append([a for a, _ in r])
        hi.append([b for _, b in r])
        off.append(float(p.get("offset", 0)))
    return names, np.array(lo), np.array(hi), np.array(off)


def nearest(pts, lo, hi, off):
    out = np.empty(len(pts), dtype=np.int32)
    for s in range(0, len(pts), 1500):
        p = pts[s:s + 1500, None, :]
        d = np.maximum(0.0, np.maximum(lo[None] - p, p - hi[None]))
        dist = (d * d).sum(axis=2) + off[None] ** 2
        out[s:s + 1500] = dist.argmin(axis=1)
    return out


def table(t):
    """ClimateMapping.Table.apply: piecewise linear, the offset kept beyond the ends; odd on |v| (weirdness)."""
    f, to = np.array(t["from"]), np.array(t["to"])
    odd = t.get("odd", False)

    def apply(v):
        a = np.abs(v) if odd else v
        r = np.interp(a, f, to)
        r = np.where(a <= f[0], to[0] + (a - f[0]), r)
        r = np.where(a >= f[-1], to[-1] + (a - f[-1]), r)
        return np.copysign(r, v) if odd else r
    return apply


def not_land_biome(name):
    return "ocean" in name or "river" in name


def pct(a, b):
    return 100.0 * a / b if b else float("nan")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--layers", required=True)
    ap.add_argument("--mapping", required=True)
    ap.add_argument("--params", required=True)
    ap.add_argument("--census")
    ap.add_argument("--reference", action="append", default=[])
    ap.add_argument("--sea-y", type=float, default=63.0)
    a = ap.parse_args()

    arr = np.load(a.layers)
    meta = json.load(open(a.layers + ".json"))
    ch = {n: i for i, n in enumerate(meta["channels"])}
    rows, cols = arr.shape[:2]

    def get(n):
        return arr[:, :, ch[n]].reshape(-1).astype(np.float64)

    tables = json.load(open(a.mapping))["tables"]
    names, lo, hi, off = load_entries(a.params)
    T, V, C, E, W = (get("climate/" + n) for n in ("temperature", "vegetation", "continents", "erosion", "ridges"))
    kname = "climate/koppen_l2" if "climate/koppen_l2" in ch else "climate/koppen"
    K = np.nan_to_num(get(kname), nan=SEA).round().astype(int)
    K[(K < 0) | (K > SEA)] = SEA
    ice = np.nan_to_num(get("climate/ice_cover"), nan=0.0) > 0 if "climate/ice_cover" in ch else np.zeros(len(K), bool)
    H = get(meta["channels"][0])
    cand = np.nonzero((K != SEA) & (H > a.sea_y))[0]
    t = table(tables["climate/temperature"])(T[cand])
    v = table(tables["climate/vegetation"])(V[cand])
    c = table(tables["climate/continents"])(C[cand])
    e = table(tables["climate/erosion"])(E[cand])
    w = table(tables["climate/ridges"])(W[cand])
    ans = nearest(np.stack([t, v, c, e, w, np.zeros_like(t)], axis=1), lo, hi, off)
    bio = np.array([names[j] for j in ans])
    keep = np.array([not not_land_biome(b) for b in bio])
    idx, bio, t, v, w = cand[keep], bio[keep], t[keep], v[keep], w[keep]
    k, g = K[idx], ice[idx]
    n = len(idx)
    print(f"grid {rows}x{cols} step {meta['step']} ({kname}); land columns {n} "
          f"({len(cand) - n} land-class columns on an ocean or river biome left out); {len(names)} entries")

    if a.census:
        m = json.load(open(a.census))
        if m["w"] * m["h"] == rows * cols:
            cb = struct.unpack("<%dH" % (m["w"] * m["h"]), base64.b64decode(m["biome_u16le"]))
            pal = [p["id"] for p in m["palette"]]
            game = np.array([pal[cb[i]] for i in idx])
            ok = (game == bio) | np.isin(game, list(VOLCANIC))
            print(f"agreement with the census: {ok.mean():.3f}")
            gl = np.array([not not_land_biome(b) for b in game])
            cnt = Counter(game[gl])
            print(f"census land columns {gl.sum()}: ice_spikes {pct(cnt['minecraft:ice_spikes'], gl.sum()):.2f} %, "
                  f"snowy_plains {pct(cnt['minecraft:snowy_plains'], gl.sum()):.2f} %")
        else:
            print(f"census grid {m['w']}x{m['h']} differs from the layers' {cols}x{rows}; not compared")

    t0, h0, pos = t < T0_TOP, v < H0_TOP, w >= 0
    spikes = bio == "minecraft:ice_spikes"
    print(f"\nland: frozen T0 {pct(t0.sum(), n):.1f} %, dry H0 {pct(h0.sum(), n):.1f} %, both {pct((t0 & h0).sum(), n):.1f} %; "
          f"weirdness positive {pct(pos.sum(), n):.1f} % (in T0 and H0: {pct((pos & t0 & h0).sum(), (t0 & h0).sum()):.1f} %)")
    groups = [("EF under ice", (k == EF) & g), ("EF bare", (k == EF) & ~g), ("ET", k == ET), ("BWk", k == BWK),
              ("other under ice", (k != EF) & (k != ET) & (k != BWK) & g)]
    rest = np.ones(n, bool)
    for _, s in groups:
        rest &= ~s
    groups.append(("the rest", rest))
    print(f"\n{'group':16} {'land %':>7} {'T0&H0 %':>8} {'w>0 %':>6} {'spikes %':>8} {'of all spikes %':>15}  top biomes")
    for label, s in groups:
        ns = s.sum()
        if not ns:
            continue
        top = ", ".join(f"{b.split(':')[1]} {pct(x, ns):.0f}" for b, x in Counter(bio[s]).most_common(5))
        print(f"{label:16} {pct(ns, n):7.1f} {pct((s & t0 & h0).sum(), ns):8.1f} {pct((s & pos).sum(), ns):6.1f} "
              f"{pct((s & spikes).sum(), ns):8.1f} {pct((s & spikes).sum(), spikes.sum()):15.1f}  {top}")
    # The ice cap's ice spikes by the slot they come from (pack units: continentalness band, erosion band, sign).
    cap = (k == EF) & g
    cband = np.searchsorted([-0.455, -0.19, -0.11, 0.03, 0.3], c[keep], side="right")
    eband = np.searchsorted([-0.78, -0.375, -0.2225, 0.05, 0.45, 0.55], e[keep], side="right")
    cnames = ["deep", "ocean", "coast", "near", "mid", "far"]
    slots = Counter()
    for ci, ei, p_, sp in zip(cband[cap], eband[cap], pos[cap], spikes[cap]):
        slots[(cnames[ci], f"E{ei}", "+" if p_ else "-")] += np.array([1, int(sp)])
    bands_of = lambda s, b, m: " ".join(f"{nm} {pct((b[s] == i).sum(), s.sum()):.0f}" for i, nm in enumerate(m))
    enames = [f"E{i}" for i in range(7)]
    print(f"\ncontinentalness bands, % of land:   {bands_of(np.ones(n, bool), cband, cnames)}")
    print(f"continentalness bands, % of ice cap: {bands_of(cap, cband, cnames)}")
    print(f"erosion bands, % of land:   {bands_of(np.ones(n, bool), eband, enames)}")
    print(f"erosion bands, % of ice cap: {bands_of(cap, eband, enames)}")
    print("\nice cap by slot (continentalness, erosion band, weirdness sign): columns, of them ice spikes")
    for s_, v_ in sorted(slots.items(), key=lambda kv: -kv[1][0])[:24]:
        print(f"  {s_[0]:5} {s_[1]} {s_[2]}  {v_[0]:6d} {v_[1]:6d}")
    corner = t0 & h0
    print(f"\nice spikes: {pct(spikes.sum(), n):.2f} % of land; of the T0 and H0 land {pct((spikes & corner).sum(), corner.sum()):.1f} %, "
          f"of it with weirdness > 0 {pct((spikes & corner & pos).sum(), (corner & pos).sum()):.1f} %, "
          f"with weirdness < 0 {pct((spikes & corner & ~pos).sum(), (corner & ~pos).sum()):.1f} %")

    refs = []
    for r in a.reference:
        label, path = r.split("=", 1)
        refs.append((label, json.load(open(path))))
    cnt = Counter(bio)
    head = f"\n{'biome':36} {'planet %':>9}" + "".join(f" {lbl + ' %':>12} {'x':>6}" for lbl, _ in refs)
    print(head)
    for b in WATCH:
        line = f"{b:36} {pct(cnt[b], n):9.3f}"
        for _, ref in refs:
            rs = pct(ref["land_biomes"].get(b, 0), ref["land"])
            ratio = pct(cnt[b], n) / rs if rs else float("inf") if cnt[b] else float("nan")
            line += f" {rs:12.3f} {ratio:6.1f}"
        print(line)
    for label, ref in refs:
        rc = ref["frozen_dry_corner"]
        cross = np.array(ref["land_temperature_x_humidity"], dtype=float)
        print(f"{label}: land {ref['land']}; T0 {pct(cross[0].sum(), ref['land']):.1f} %, H0 {pct(cross[:, 0].sum(), ref['land']):.1f} %, "
              f"T0 and H0 {pct(rc['land'], ref['land']):.2f} % (weirdness > 0 {pct(rc['weirdness_positive'], rc['land']):.0f} %); "
              f"ice spikes {pct(rc['biomes'].get('minecraft:ice_spikes', 0), rc['land']):.1f} % of that corner, "
              f"{pct(ref['land_biomes'].get('minecraft:ice_spikes', 0), ref['land']):.3f} % of land")
        if "land_continentalness_bands" in ref:
            cb_ = ref["land_continentalness_bands"]
            print(f"  continentalness bands, % of land: " + " ".join(
                f"{nm} {pct(x, ref['land']):.0f}" for nm, x in zip(["mushroom", "deep", "ocean", "coast", "near", "mid", "far"], cb_)))
            eb_ = ref["land_erosion_bands"]
            print(f"  erosion bands, % of land: " + " ".join(f"E{i} {pct(x, ref['land']):.0f}" for i, x in enumerate(eb_)))
            print("  T0 and H0 corner by slot (continentalness band, erosion band, sign): % of the corner, its ice spikes rate")
            for s_, (cols_, sp_) in sorted(rc.get("slots", {}).items(), key=lambda kv: -kv[1][0])[:20]:
                print(f"    {s_:10} {pct(cols_, rc['land']):6.1f} % {pct(sp_, cols_):6.1f} %")


if __name__ == "__main__":
    main()
