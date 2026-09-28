#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Generate Deep Time's noise settings, dimension type and presets from vanilla 1.21.1's data.

    tools/dev/gen_worldgen.py [<client-extra jar>]

The output is committed; re-run only after changing this script or bumping Minecraft. The jar
defaults to the one any Gradle build leaves in neoforge/build/moddev/artifacts/.

What it writes (plan §7.1, §7.12):
  data/deeptime/worldgen/noise_settings/planet.json
      Vanilla's overworld settings with Deep Time's router: the build range -352..639, climate
      from `deeptime:raster`, depth from `deeptime:depth`, and vanilla's final_density graph with
      `minecraft:overworld/sloped_cheese` replaced by `deeptime:terrain` and the slides
      re-anchored (bottom -352 -> -328, top 624 -> 640). The cave functions stay references
      (`minecraft:overworld/caves/*`), so a pack's own caves still carve. Vanilla's large ore
      veins are off (decisions file). The surface rule delegates to the pack's
      `minecraft:overworld` settings, resolved when applied (`deeptime:delegate`).
  data/deeptime/worldgen/noise_settings/planet_nocaves.json
      The same without the cave branch and without aquifers, for the GameTest dimensions: the
      density fill then leaves every surface intact, so tests can read heights at NOISE status.
  data/deeptime/dimension_type/planet_overworld.json   vanilla overworld, y -352..639
  data/deeptime/worldgen/world_preset/planet{,_32k}.json and the #minecraft:normal tag entry.
"""
import copy
import json
import os
import sys
import zipfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(REPO, "neoforge", "src", "main", "resources", "data")
JAR = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    REPO, "neoforge/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar")

MIN_Y, HEIGHT = -352, 992
# Vanilla's slides sit 24 blocks above the floor and 64-80 below the 320 top. The bottom slide
# keeps its 24-block span at the new floor; the top slide is moved to the last 16 blocks so
# terrain up to y ~620 is not flattened (cold-read review 2, gap 8).
SLIDES = {(-64, -40): (MIN_Y, MIN_Y + 24), (240, 256): (624, 640)}
TERRAIN_FINAL = {"type": "deeptime:terrain", "mode": "final"}


def load(z, path):
    return json.loads(z.read(path))


def transform(node):
    """Replace sloped_cheese with Deep Time's terrain and re-anchor the slides."""
    if node == "minecraft:overworld/sloped_cheese":
        return copy.deepcopy(TERRAIN_FINAL)
    if isinstance(node, dict):
        out = {k: transform(v) for k, v in node.items()}
        if out.get("type") == "minecraft:y_clamped_gradient":
            key = (out["from_y"], out["to_y"])
            if key not in SLIDES:
                raise SystemExit(f"unexpected y_clamped_gradient {key}")
            out["from_y"], out["to_y"] = SLIDES[key]
        return out
    if isinstance(node, list):
        return [transform(v) for v in node]
    return node


def without_caves(final_density):
    """min(squeeze(... range_choice(sloped_cheese, caves) ...), noodle) -> squeeze(... terrain ...)."""
    squeeze = copy.deepcopy(final_density["argument1"])  # drop the noodle branch

    def strip(node):
        if isinstance(node, dict):
            if node.get("type") == "minecraft:range_choice" and node.get("input") == TERRAIN_FINAL:
                return copy.deepcopy(TERRAIN_FINAL)
            return {k: strip(v) for k, v in node.items()}
        return node

    out = strip(squeeze)
    if "caves" in json.dumps(out):
        raise SystemExit("cave functions survived without_caves()")
    return out


def write(rel, obj):
    path = os.path.join(OUT, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        json.dump(obj, f, indent=2)
        f.write("\n")
    print("wrote", os.path.relpath(path, REPO))


def main():
    with zipfile.ZipFile(JAR) as z:
        ow = load(z, "data/minecraft/worldgen/noise_settings/overworld.json")
        dim = load(z, "data/minecraft/dimension_type/overworld.json")
        normal = load(z, "data/minecraft/worldgen/world_preset/normal.json")

    vr = ow["noise_router"]
    final = transform(vr["final_density"])
    router = {
        # Aquifers as vanilla: sea level and local cave levels.
        "barrier": vr["barrier"],
        "fluid_level_floodedness": vr["fluid_level_floodedness"],
        "fluid_level_spread": vr["fluid_level_spread"],
        "lava": vr["lava"],
        # The five 2D biome inputs come from the simulation (world data) or the analytic test
        # terrain; flat_cache because they do not vary with y.
        **{name: {"type": "minecraft:flat_cache",
                  "argument": {"type": "deeptime:raster", "layer": f"climate/{name}"}}
           for name in ("temperature", "vegetation", "continents", "erosion", "ridges")},
        "depth": {"type": "deeptime:depth"},
        "initial_density_without_jaggedness": {"type": "deeptime:terrain", "mode": "initial"},
        "final_density": final,
        "vein_toggle": 0.0,
        "vein_ridged": 0.0,
        "vein_gap": 0.0,
    }
    planet = {
        "sea_level": ow["sea_level"],
        "disable_mob_generation": ow["disable_mob_generation"],
        "aquifers_enabled": True,
        "ore_veins_enabled": False,
        "legacy_random_source": ow["legacy_random_source"],
        "default_block": ow["default_block"],
        "default_fluid": ow["default_fluid"],
        "noise": {"min_y": MIN_Y, "height": HEIGHT, "size_horizontal": 1, "size_vertical": 2},
        "noise_router": router,
        "spawn_target": ow["spawn_target"],
        "surface_rule": {"type": "deeptime:delegate", "settings": "minecraft:overworld"},
    }
    write("deeptime/worldgen/noise_settings/planet.json", planet)
    nocaves = copy.deepcopy(planet)
    nocaves["noise_router"]["final_density"] = without_caves(final)
    # No aquifers either: a local aquifer next to the sea places barrier stone a few blocks above
    # an ocean floor, which would move the surface the height tests measure. The fluid picker
    # still fills everything below sea level with water.
    nocaves["aquifers_enabled"] = False
    write("deeptime/worldgen/noise_settings/planet_nocaves.json", nocaves)

    dim = dict(dim, min_y=MIN_Y, height=HEIGHT, logical_height=HEIGHT)
    write("deeptime/dimension_type/planet_overworld.json", dim)

    for name, c in (("planet", 16384), ("planet_32k", 32768)):
        dims = copy.deepcopy(normal["dimensions"])
        dims["minecraft:overworld"] = {
            "type": "deeptime:planet_overworld",
            "generator": {
                "type": "deeptime:geologic",
                "biome_source": {"type": "minecraft:multi_noise", "preset": "minecraft:overworld"},
                "settings": "deeptime:planet",
                "world_data": {"folder": "deeptime", "circumference_blocks": c},
            },
        }
        write(f"deeptime/worldgen/world_preset/{name}.json", {"dimensions": dims})
    write("minecraft/tags/worldgen/world_preset/normal.json", {"replace": False, "values": ["deeptime:planet"]})
    write("terrablender/tags/dimension_type/overworld_regions.json",
          {"replace": False, "values": [{"id": "deeptime:planet_overworld", "required": False}]})


if __name__ == "__main__":
    main()
