#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""PROTOTYPE (docs/design/seamless-wrapping.md): a dev-only copy of the Deep Time jar that loads
without Toroidal World, for the Immersive Portals prototype (TW and Immersive Portals cannot run
together: every join is kicked, docs/research/notes/immersive-portals-pillowcase-prototype.md §5).

  notw_jar.py <deeptime.jar> <out.jar>

Two edits, both in the copy only, never in the shipped jar:
  - META-INF/neoforge.mods.toml: the toroidal_world dependency becomes optional;
  - deeptime.mixins.json: WorldDimensionsShapeMixin (TW's PlanetShape on WorldDimensions.bake) goes.
"""
import json, re, sys, zipfile

src, dst = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(src) as zin, zipfile.ZipFile(dst, "w", zipfile.ZIP_DEFLATED) as zout:
    for item in zin.infolist():
        data = zin.read(item.filename)
        if item.filename == "META-INF/neoforge.mods.toml":
            text = data.decode()
            new, n = re.subn(r'(modId\s*=\s*"toroidal_world"\s*\n\s*type\s*=\s*)"required"', r'\1"optional"', text)
            assert n == 1, "toroidal_world dependency not found"
            data = new.encode()
        elif item.filename == "deeptime.mixins.json":
            cfg = json.loads(data)
            assert "WorldDimensionsShapeMixin" in cfg["mixins"]
            cfg["mixins"].remove("WorldDimensionsShapeMixin")
            data = (json.dumps(cfg, indent=2) + "\n").encode()
        zout.writestr(item, data)
print(f"{dst}: toroidal_world optional, WorldDimensionsShapeMixin removed")
