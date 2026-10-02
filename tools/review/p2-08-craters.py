#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Review pictures of a run's impact craters (P2-08, surface/impacts).

Reads the baked crater list (<world>/impacts/craters.json) and the run summary
(manifest.json notes.impacts), picks craters to look at (the youngest that shows on land, the
oldest that still shows on land, the largest that shows, a buried one, one worn to its roots)
and draws for each a hill-shaded map of the terrain the game reads (`deeptime heightmap`, blocks
at 1:1) with the rim circle, and an east-west section through its centre beside the fresh
profile's depth. Writes <out>/<name>.png, craters.json (the picks and the counts) and prints the
counts.

    python3 tools/review/p2-08-craters.py --world-data <run> --deeptime target/release/deeptime \\
        --out review/out/p2-08

Needs numpy and matplotlib (on the test machine: nix shell --impure --expr
'(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages
(p: [p.numpy p.matplotlib])' -c python3 …).
"""

import argparse
import json
import subprocess
import tempfile
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
from matplotlib.colors import LightSource  # noqa: E402


def heightmap(deeptime: str, world: Path, box: list[int]) -> np.ndarray:
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "h.npy"
        subprocess.run(
            [deeptime, "heightmap", "--world-data", str(world), "--box",
             ",".join(str(v) for v in box), "--layers", "water_level", "--out", str(out)],
            check=True, stdout=subprocess.DEVNULL)
        return np.load(out)


def draw(c: dict, title: str, h: np.ndarray, box: list[int], sea_y: float, m_h: float, m_v: float,
         path: Path) -> dict:
    surface, water = h[:, :, 0], h[:, :, 3]
    x0, z0, x1, z1 = box
    radius = c["diameter_m"] / 2 / m_h
    fig, (ax, sec) = plt.subplots(1, 2, figsize=(13, 6), gridspec_kw={"width_ratios": [1, 1.15]})
    ls = LightSource(azdeg=315, altdeg=40)
    lo, hi = float(np.percentile(surface, 1)), float(np.percentile(surface, 99))
    rgb = ls.shade(surface, cmap=matplotlib.colormaps["gist_earth"], vmin=lo - 0.3 * (hi - lo),
                   vmax=hi + 0.2 * (hi - lo), vert_exag=1.0, dx=1.0, dy=1.0, blend_mode="soft")
    wet = water > surface + 0.5
    rgb[wet] = 0.55 * rgb[wet] + 0.45 * np.array([0.25, 0.45, 0.8, 1.0])
    ax.imshow(rgb, extent=[x0, x1, z1, z0], interpolation="nearest")
    th = np.linspace(0, 2 * np.pi, 200)
    ax.plot(c["x"] + radius * np.cos(th), c["z"] + radius * np.sin(th), color="white", lw=0.8, ls=":")
    ax.set_xlabel("block x")
    ax.set_ylabel("block z")
    ax.set_title(f"{title}: crater {c['id']}, {c['diameter_m'] / 1000:.1f} km ({2 * radius:.0f} blocks), "
                 f"{c['morphology']}, {c['age_myr']:.1f} Myr, {c['fate']}", fontsize=10)
    row = int(round(c["z"])) - z0
    row = min(max(row, 0), surface.shape[0] - 1)
    xs = np.arange(x0, x1)
    sec.fill_between(xs, surface[row], lo - 5, color="#8a7a66")
    wl = np.where(wet[row], water[row], np.nan)
    sec.fill_between(xs, wl, surface[row], where=wet[row], color="#4f7fd0", alpha=0.7)
    sec.plot(xs, surface[row], color="black", lw=0.8)
    for sx in (c["x"] - radius, c["x"] + radius):
        sec.axvline(sx, color="gray", lw=0.6, ls=":")
    sec.set_aspect("equal")
    sec.set_xlabel("block x (section through the centre, 1 block = 1 block)")
    sec.set_ylabel("block y")
    centre = float(surface[row, min(max(int(round(c["x"])) - x0, 0), surface.shape[1] - 1)])
    ring = [float(surface[min(max(int(round(c["z"] + radius * np.sin(a))) - z0, 0), surface.shape[0] - 1),
                          min(max(int(round(c["x"] + radius * np.cos(a))) - x0, 0), surface.shape[1] - 1)])
            for a in np.linspace(0, 2 * np.pi, 32, endpoint=False)]
    rim = float(np.mean(ring))
    sec.set_title(f"fresh depth {c['depth_m'] / m_v:.1f} blocks, simulated relief left {c['relief_m'] / m_v:.1f}; "
                  f"in the terrain: rim y {rim:.1f}, centre y {centre:.1f} ({rim - centre:.1f} blocks)", fontsize=9)
    fig.tight_layout()
    fig.savefig(path, dpi=110)
    plt.close(fig)
    return {"rim_y": rim, "centre_y": centre, "bowl_blocks": rim - centre, "sea_y": sea_y,
            "wet_share": float(wet.mean())}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--world-data", required=True, type=Path)
    ap.add_argument("--deeptime", default="target/release/deeptime")
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args()
    a.out.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((a.world_data / "manifest.json").read_text())
    scale = manifest["scale"]
    m_h, m_v, sea_y = scale["m_per_block_h"], scale["m_per_block_v"], scale["sea_level_y"]
    circ = scale["circumference_blocks"]
    note = manifest.get("notes", {}).get("impacts", {})
    if isinstance(note, str):
        note = json.loads(note)
    craters = json.loads((a.world_data / "impacts" / "craters.json").read_text())["craters"]

    def land(c: dict) -> bool:
        box = [int(c["x"]) - 1, int(c["z"]) - 1, int(c["x"]) + 1, int(c["z"]) + 1]
        return float(heightmap(a.deeptime, a.world_data, box)[0, 0, 0]) > sea_y + 1

    clear = [c for c in craters if abs(c["z"]) < circ / 4 - 3 * c["diameter_m"] / m_h - 32]
    showing = [c for c in clear if c["fate"] in ("fresh", "degraded")]
    showing_land = [c for c in showing if land(c)]
    picks: list[tuple[str, dict]] = []

    def pick(name: str, pool: list[dict], key) -> None:
        pool = [c for c in pool if all(c["id"] != p["id"] for _, p in picks)]
        if pool:
            picks.append((name, sorted(pool, key=key)[0]))

    pick("young", [c for c in showing_land if c["fate"] == "fresh"], lambda c: c["age_myr"])
    pick("old", showing_land, lambda c: -c["age_myr"])
    pick("largest", showing, lambda c: -c["diameter_m"])
    pick("young-complex", [c for c in showing_land if c["morphology"] != "simple"], lambda c: c["age_myr"])
    pick("buried", [c for c in clear if c["fate"] == "buried"], lambda c: -c["diameter_m"])
    pick("roots", [c for c in clear if c["fate"] == "eroded_to_roots"], lambda c: -c["diameter_m"])
    out = {"counts": note.get("counts"), "size_classes": note.get("size_classes"), "present": note.get("present"),
           "formed": note.get("formed"), "visible": note.get("visible"),
           "visible_on_continents": note.get("visible_on_continents"), "catalogue": len(craters),
           "showing": len(showing), "showing_on_land": len(showing_land), "views": []}
    for name, c in picks:
        half = int(max(48, 2.2 * c["diameter_m"] / 2 / m_h))
        box = [int(c["x"]) - half, int(c["z"]) - half, int(c["x"]) + half, int(c["z"]) + half]
        h = heightmap(a.deeptime, a.world_data, box)
        seen = draw(c, name, h, box, sea_y, m_h, m_v, a.out / f"{name}.png")
        out["views"].append({"name": name, "crater": c, "box": box, **seen})
        print(f"{name}: crater {c['id']} {c['diameter_m'] / 1000:.1f} km {c['morphology']} {c['age_myr']:.1f} Myr "
              f"{c['fate']} relief {c['relief_m']:.0f} m of {c['depth_m']:.0f} m; terrain bowl {seen['bowl_blocks']:.1f} blocks")
    (a.out / "craters.json").write_text(json.dumps(out, indent=1))
    ages = sorted(c["age_myr"] for c in showing)
    print(f"catalogue={len(craters)} showing={len(showing)} showing_on_land={len(showing_land)} "
          f"oldest_showing_myr={ages[-1] if ages else 0} formed={note.get('formed')} "
          f"formed_by_size={note.get('counts', {}).get('formed')} size_classes={note.get('size_classes')}")
    print(f"present={json.dumps(note.get('present'))}")


if __name__ == "__main__":
    main()
