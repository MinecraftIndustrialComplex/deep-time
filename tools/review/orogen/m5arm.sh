#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# The orogen lane (VM, cached pool folders only): the M5 breakdown (tools/review/m5diag: m5fields, m5maps,
# m5table) over one or more arms, each admitted as a light job.
#   m5arm.sh <tag>=<pool argkey folder> [...]   -> /data/ban/deeptime-runs/m5diag/table-<tag>.txt
set -uo pipefail
cd "$(dirname "$0")/../../.."
for pair in "$@"; do
    tag=${pair%%=*} folder=${pair#*=}
    export M5DIAG_RUNS=$folder
    deeptime-admit run --kind light -- tools/review/m5diag/m5fields.sh 4 "fields-$tag"
    deeptime-admit run --kind light -- tools/review/m5diag/m5maps.sh 4 "maps-$tag"
    tools/review/m5diag/m5table.sh "fields-$tag" "maps-$tag" "table-$tag" > /dev/null
    echo "done $tag"
done
