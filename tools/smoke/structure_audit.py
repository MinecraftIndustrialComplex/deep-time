#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Audit of the worldgen structures a jar set defines, against Deep Time's burial rule
(StructureBurial.covers; docs/science/world-scale-and-bake.md, "Structures keep to the ground").

Reads data/<ns>/worldgen/structure/*.json from every jar in a folder (and the jars nested under
META-INF/jarjar), plus any --extra jar (the vanilla resources jar), and prints one row per structure:
type, step, terrain adaptation, start height, heightmap projection, and what the rule does with it.
Run it on the test VM (a jar sweep): tools/remote/mac.sh -- 'python3 tools/smoke/structure_audit.py
smoke/mic/server/mods --extra neoforge/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar'.

The classification is StructureBurial.covers on the structure's JSON:
  covered      the rule moves a start of it down when its top comes within `cover` of the ground: a
               minecraft:stronghold (any step), the ordinary minecraft:mineshaft, and any structure in
               an underground step (underground_structures, underground_decoration, strongholds) with a
               start_height and no project_start_to_heightmap (trial chambers, ancient cities, ...)
  follows      underground step but the structure follows the ground: buried treasure, the mesa
               mineshaft, a start projected to a heightmap (trail ruins): never touched
  unknown      underground step, a type without a start_height that the rule does not know (Ice and
               Fire's dragon caves, the Nether's fortress and fossils): never touched
  gap          NOT covered, though it places by an absolute height below ground (a start_height whose
               whole range is under y 63, no heightmap projection) in a step the rule does not look at:
               Terralith's underground/* sites sit in surface_structures
  other-step   not covered and not underground by its height (listed only when it has a start_height)
"""
import argparse
import io
import json
import sys
import zipfile
from pathlib import Path

UNDERGROUND = {"underground_structures", "underground_decoration", "strongholds"}
FOLLOWS_TYPES = {"minecraft:buried_treasure"}


def height_text(h):
    if h is None:
        return "-"
    if isinstance(h, dict):
        t = h.get("type", "")
        body = {k: v for k, v in h.items() if k != "type"}
        return (t.split(":")[-1] + " " + json.dumps(body, separators=(",", ":"))).strip()
    return json.dumps(h, separators=(",", ":"))


def top_of(h):
    """The highest absolute start y a start_height can give, or None when it is not absolute."""
    if h is None:
        return None
    if isinstance(h, dict):
        if "absolute" in h:
            return h["absolute"]
        if "max_inclusive" in h:
            return top_of(h["max_inclusive"])
        return None
    return None


def classify(doc):
    typ = doc.get("type", "?")
    step = doc.get("step", "?")
    if typ == "minecraft:stronghold":
        return "covered"
    if step not in UNDERGROUND:
        top = top_of(doc.get("start_height"))
        if top is not None and top < 63 and not doc.get("project_start_to_heightmap"):
            return "gap"
        return "other-step"
    if typ in FOLLOWS_TYPES or doc.get("project_start_to_heightmap"):
        return "follows"
    if typ == "minecraft:mineshaft":
        return "covered" if doc.get("mineshaft_type") == "normal" else "follows"
    if "start_height" in doc:
        return "covered"
    return "unknown"


def scan_zip(zf, label, rows):
    for name in zf.namelist():
        parts = name.split("/")
        if len(parts) >= 5 and parts[0] == "data" and parts[2] == "worldgen" and parts[3] == "structure" and name.endswith(".json"):
            ns = parts[1]
            sid = ns + ":" + "/".join(parts[4:])[:-5]
            try:
                doc = json.loads(zf.read(name))
            except Exception as e:  # noqa: BLE001
                rows.append((sid, "unreadable: %s" % e, "?", "?", "-", "-", "?", label))
                continue
            rows.append((sid, doc.get("type", "?"), doc.get("step", "?"), doc.get("terrain_adaptation", "none"),
                         height_text(doc.get("start_height")), doc.get("project_start_to_heightmap", "-"),
                         classify(doc), label))
        elif name.startswith("META-INF/jarjar/") and name.endswith(".jar"):
            try:
                scan_zip(zipfile.ZipFile(io.BytesIO(zf.read(name))), label + "!" + name.split("/")[-1], rows)
            except zipfile.BadZipFile:
                pass


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("jars", help="folder of mod jars")
    ap.add_argument("--extra", action="append", default=[], help="more jars (the vanilla resources jar)")
    ap.add_argument("--all", action="store_true", help="also list the other-step structures that have no start_height")
    args = ap.parse_args()
    rows = []
    paths = sorted(Path(args.jars).glob("*.jar")) + [Path(p) for p in args.extra]
    for p in paths:
        try:
            with zipfile.ZipFile(p) as zf:
                scan_zip(zf, p.name, rows)
        except zipfile.BadZipFile:
            print("skipped (not a zip): %s" % p, file=sys.stderr)
    rows.sort(key=lambda r: (r[6], r[0]))
    print("| structure | type | step | adaptation | start_height | heightmap | rule | jar |")
    print("|---|---|---|---|---|---|---|---|")
    counts = {}
    for r in rows:
        counts[r[6]] = counts.get(r[6], 0) + 1
        if r[6] == "other-step" and not args.all and r[4] == "-":
            continue
        print("| " + " | ".join(str(c) for c in r) + " |")
    print()
    print("jars scanned: %d; structures: %d; %s" % (len(paths), len(rows), ", ".join("%s %d" % kv for kv in sorted(counts.items()))))


if __name__ == "__main__":
    main()
