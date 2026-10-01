#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The deposit audit's real-block checks, one server-lock hold for all of them (the lock is
# contended: one hold, three boots). Runs ON the test machine:
#
#   tools/remote/mac.sh --pack --server --pull review/out/deposit-audit -- \
#       'tools/review/deposit-audit-previews.sh <world-data> <x0,z0,x1,z1>'
#
# 1. the vanilla reference (deposit-audit.sh vanilla);
# 2. the box of <world-data> on the standalone set (Deep Time + Create only);
# 3. the same box on the mic-ev set (the MIC pack plus Excavated Variants, what MIC's deep-time
#    pack branch runs).
# Each preview's ore counts (by material: in deposit bodies, background scatter of claimed
# materials, other mods' features of unclaimed ones; by block id) go to
# review/out/deposit-audit/box-<profile>.json.
set -uo pipefail
cd "$(dirname "$0")/../.."
OUT=review/out/deposit-audit
mkdir -p "$OUT"
world="${1:?world-data folder}"; box="${2:?box x0,z0,x1,z1}"
export DEEPTIME_WORKBENCH_ROOTS="$(dirname "$(realpath "$world")")"

if [ "${SKIP_VANILLA:-0}" != 1 ]; then
    tools/review/deposit-audit.sh vanilla 256 2>&1 | tail -25
fi
for p in ${PROFILES:-standalone mic-ev}; do
    job="deposit-audit-$p"
    rm -rf "smoke/workbench/jobs/$job"
    install=()
    [ -d "smoke/$p/server/mods" ] || install=(--install)
    python3 tools/workbench/preview.py run --lock-held --job "$job" --profile="$p" --size=16384 \
        --box="$box" --world-data="$world" "${install[@]}" 2>&1 | tail -4
    python3 - "smoke/workbench/jobs/$job" "$OUT/box-$p.json" <<'EOF'
import gzip, json, struct, sys, os
job, out = sys.argv[1], sys.argv[2]
f = os.path.join(job, "area-0.dtva.gz")
if not os.path.exists(f):
    print("no export in", job, open(os.path.join(job, "status.json")).read()[:600] if os.path.exists(os.path.join(job, "status.json")) else "")
    sys.exit(0)
d = gzip.decompress(open(f, "rb").read())
n = struct.unpack("<I", d[8:12])[0]
h = json.loads(d[12:12 + n])
doc = {"box": h["box"], "meta": h.get("meta", {}).get("install"), "ore": h["stats"]["ore"],
       "ore_block_ids": sorted(b["id"] for b in h["blocks"] if b.get("cat") == "ore"),
       "categories": h["stats"]["categories"], "claims": h.get("claims"),
       "bodies": [{k: b[k] for k in ("id", "site", "role", "shape", "rock_class", "voxels_in_box", "ore_blocks_in_box")} for b in h["bodies"]]}
json.dump(doc, open(out, "w"), indent=1)
print(job, json.dumps(doc["ore"]))
EOF
done
