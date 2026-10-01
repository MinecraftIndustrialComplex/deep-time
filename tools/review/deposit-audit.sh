#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The deposit audit (Ben, 2026-10-01, "Audit all types, then tune"): how deposits come out in
# blocks, beside the background scatter and a vanilla overworld. Runs ON the test machine, one
# step per call; re-run it after any tuning of deposits or scatter.
#
#   tools/remote/mac.sh          -- 'tools/review/deposit-audit.sh audit <world-data> [name] [deposit-audit args…]'
#   tools/remote/mac.sh --server -- 'tools/review/deposit-audit.sh vanilla [radius-blocks]'
#
# audit    `deeptime deposit-audit` on a baked world-data folder (a run's `world/`, a save's
#          `deeptime/`): the table on stdout, every site's row in review/out/deposit-audit/<name>.json.
#          No Minecraft server: the bake is evaluated as the mod's deposit and scatter features do
#          (crates/deeptime-cli/src/deposit_audit.rs says how, and how close that is to real chunks).
# vanilla  The vanilla reference: the standalone server set with level-type minecraft:normal (a
#          plain 1.21.1 overworld; Deep Time acts only in its own worlds), a Chunky square of the
#          given radius (default 256 blocks: 1,024 chunks) around 0,0, then every ore block in it
#          counted by `deeptime-inspect blocks`. Prints ore blocks per chunk by block id and by
#          material, and writes review/out/deposit-audit/vanilla.json. The figures in
#          VANILLA_PER_CHUNK (deposit_audit.rs) come from this step.
set -euo pipefail
cd "$(dirname "$0")/../.."
OUT=review/out/deposit-audit
mkdir -p "$OUT"
step="${1:?usage: deposit-audit.sh <audit <world-data> [name] [args…] | vanilla [radius]>}"
shift

case "$step" in
audit)
    world="${1:?audit needs a world-data folder}"; shift
    name="${1:-$(basename "$(dirname "$world")")}"; [ $# -gt 0 ] && shift
    cargo build -q --release -p deeptime-cli
    ./target/release/deeptime deposit-audit --world-data "$world" --json "$OUT/$name.json" "$@" | tee "$OUT/$name.md" ;;
vanilla)
    radius="${1:-256}"
    seed="${DEPOSIT_AUDIT_SEED:-20261001}"
    tools/smoke/install.sh standalone --seed "$seed" 2>&1 | tail -3
    set +e
    python3 tools/smoke/run.py --profile standalone --fresh --level-type minecraft:normal --chunky "$radius" 2>&1 \
        | grep -E '^\s*\[(PASS|FAIL| -- )\]|^(PASS|FAIL)|chunky|Done' | cut -c1-200
    set -e
    cargo build -q --release -p deeptime-inspect
    ./target/release/deeptime-inspect blocks --world smoke/standalone/server/planet \
        --area="-$radius,-$radius,$radius,$radius" > "$OUT/vanilla-blocks.txt"
    python3 - "$OUT/vanilla-blocks.txt" "$radius" "$seed" "$OUT/vanilla.json" <<'EOF'
import json, re, sys
text, radius, seed, out = open(sys.argv[1]).read(), int(sys.argv[2]), sys.argv[3], sys.argv[4]
chunks = (2 * radius // 16) ** 2
blocks = {}
try:
    doc = json.loads(text)
    src = doc.get("blocks", doc) if isinstance(doc, dict) else {}
    for k, v in src.items():
        if isinstance(v, (int, float)):
            blocks[k] = int(v)
except ValueError:
    for m in re.finditer(r"([a-z0-9_.-]+:[a-z0-9_/.-]+)\D+?(\d+)", text):
        blocks[m.group(1)] = int(m.group(2))
ores = {k: v for k, v in blocks.items() if k.endswith("_ore") or k.endswith("ancient_debris")}
by_material = {}
for k, v in ores.items():
    name = k.split(":")[1].removeprefix("deepslate_").removeprefix("nether_").removesuffix("_ore")
    by_material[name] = by_material.get(name, 0) + v
doc = {"level_type": "minecraft:normal", "seed": seed, "radius_blocks": radius, "chunks": chunks,
       "ore_blocks": ores, "per_chunk_by_block": {k: round(v / chunks, 3) for k, v in sorted(ores.items())},
       "per_chunk_by_material": {k: round(v / chunks, 3) for k, v in sorted(by_material.items())}}
json.dump(doc, open(out, "w"), indent=1)
print(f"vanilla 1.21.1 overworld, seed {seed}, {chunks} chunks around 0,0:")
for k, v in sorted(by_material.items()):
    print(f"  {k:10s} {v:8d} blocks  {v / chunks:8.2f} per chunk")
EOF
    ;;
*) echo "unknown step $step" >&2; exit 2 ;;
esac
