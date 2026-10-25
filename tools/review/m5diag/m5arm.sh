#!/usr/bin/env bash
# M5 diagnosis of one arm (VM, cached pool folders only): m5arm.sh <run folder> <name> [workers]
# <run folder> is <pool>/<key>/<preset>-<length>/<argkey>. Writes fields-<name>.jsonl, maps-<name>.jsonl
# and table-<name>.txt under /data/ban/deeptime-runs/m5diag. Every M5 they hold is the version 1 rule
# (the census grid, closed depressions as sea, the reach straight across); v1census version 2 (RLC-055) differs.
set -uo pipefail
cd "$(dirname "$0")/../../.."
export M5DIAG_RUNS="$1"
w="${3:-3}"
tools/review/m5diag/m5fields.sh "$w" "fields-$2"
tools/review/m5diag/m5maps.sh "$w" "maps-$2"
tools/review/m5diag/m5table.sh "fields-$2" "maps-$2" "table-$2" > /dev/null
echo "M5ARM_DONE $2"
