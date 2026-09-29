#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Turn Dynamic Asset Generator's cache from an ev-assets client run into a resource pack (plan Q10).

    python3 tools/client/ev-assets-pack.py review/out/q10/ev-assets

Reads <dir>/cache (a copy of the client's run/.cache/dynamic_asset_generator with DAG's full_cache on)
and <dir>/times.log, writes <dir>/pack/ and <dir>/summary.json (merging what ev-assets-mac.sh put
there). DAG 6.1.2 keeps two kinds of cache:
  cache/<mod>/assets/<ns>/...                      blockstates, models, lang, an atlas definition
  cache/dynamic_asset_generator/tex_sources__<atlas>/<ns>/textures/...   the sprites it painted
                                                   (Excavated Variants' ore-in-stone textures)
The pack holds both at assets/<ns>/..., except the atlas definition: its sprite sources are DAG's
own type, which only DAG understands, and the sprites are already files, which BlueMap and
`deeptime-inspect colors` read from the vanilla atlas's block/ directory.
times.log has one line per generated resource: "<cache> <ns>:<path> <microseconds>" (DAG's
ResourceUtils: System.nanoTime() difference / 1000).
"""
import json
import os
import re
import shutil
import sys

out = sys.argv[1]
cache = os.path.join(out, "cache")
pack = os.path.join(out, "pack")
shutil.rmtree(pack, ignore_errors=True)
counts = {}


def put(src, ns, rel):
    dest = os.path.join(pack, "assets", ns, rel)
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    shutil.copy2(src, dest)
    kind = rel.split(os.sep)[0]
    counts[kind] = counts.get(kind, 0) + 1


for root, _, files in os.walk(cache):
    parts = os.path.relpath(root, cache).split(os.sep)
    for f in files:
        src = os.path.join(root, f)
        if "assets" in parts:
            sub = parts[parts.index("assets") + 1:]
            if len(sub) >= 2 and sub[1] == "atlases":
                continue
            if sub:
                put(src, sub[0], os.path.join(*sub[1:], f))
        else:
            tex = next((i for i, p in enumerate(parts) if p.startswith("tex_sources__")), None)
            if tex is not None and len(parts) > tex + 1:
                put(src, parts[tex + 1], os.path.join(*parts[tex + 2:], f))
with open(os.path.join(pack, "pack.mcmeta"), "w") as fh:
    json.dump({"pack": {"pack_format": 34, "description": "Excavated Variants assets painted by Dynamic Asset "
                        "Generator on a client (Deep Time Q10 renders)"}}, fh)

by_kind_us, by_kind_n = {}, {}
tl = os.path.join(out, "times.log")
if os.path.exists(tl):
    for line in open(tl, errors="replace"):
        m = re.match(r"(\S+) (\S+?):([a-z_]+)/\S* (\d+)$", line.strip())
        if m:
            k = m.group(3)
            by_kind_us[k] = by_kind_us.get(k, 0) + int(m.group(4))
            by_kind_n[k] = by_kind_n.get(k, 0) + 1
sp = os.path.join(out, "summary.json")
summary = json.load(open(sp)) if os.path.exists(sp) else {}
summary.update({
    "pack_files_by_kind": counts,
    "pack_files": sum(counts.values()),
    "generated_by_kind": by_kind_n,
    "generation_ms_by_kind": {k: round(v / 1000, 1) for k, v in by_kind_us.items()},
    "generation_ms_total": round(sum(by_kind_us.values()) / 1000, 1),
    "generation_note": "sum of DAG's per-resource times (times.log, microseconds); resources may be made on several threads",
})
summary.pop("times_log_lines", None)
summary.pop("times_log_total_ms", None)
summary.pop("assets_copied", None)
summary.pop("assets_by_kind", None)
json.dump(summary, open(sp, "w"), indent=1)
print(json.dumps({k: v for k, v in summary.items() if k != "ev_and_dag_log_lines"}))
