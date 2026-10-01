#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# P1-27: cordillera calibration runs (coupled pipeline), one block of lines per variant:
#   tools/review/p1-27-calib.sh <start> <seed> <length> label:path=value,... ...
set -euo pipefail
START=$1; SEED=$2; LEN=$3; shift 3
R=/Volumes/Graphide/ban/deeptime-runs/p1-27/calib
for v in "$@"; do
    label=${v%%:*}; sets=()
    if [ "$label" != "$v" ]; then
        IFS=',' read -ra kv <<< "${v#*:}"
        for x in "${kv[@]}"; do sets+=(--set "$x"); done
    fi
    tools/review/p1-27-pipeline.sh "$R" "$label" "$START" "$SEED" "$LEN" -- "${sets[@]}"
done
