#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Cube phase 2, Ben's world-shape decision: the real client's shots of the two kept worlds, one pass
# after the other (vm-client.sh's game dir is shared within a checkout): the planet editor on
# Petroff-Guyou (the same settings as the cube walk cube2-walk: earthlike_quick_16k, 300 Myr), then
# each kept world opened with the dh set (Distant Horizons) and shot at compare.py's vistas, near
# (settle 45 s) and far (settle 240 s, DH's far terrain). Every client run admits itself.
#   tools/remote/mac.sh -- tools/remote/detach.sh start cube2-shots bash tools/review/world-choice/shots.sh [steps...]
# steps: walk pg-near cube-near pg-far cube-far (default all, in that order); pg-edges, cube-edges
# shoot the near pass again from vista 9 (the edges) into its folder
set -uo pipefail
cd "$(dirname "$0")/../../.."
CMP=${CMP:-/Volumes/Graphide/ban/deeptime-runs/cube2-cmp}
STEPS=("$@"); [ ${#STEPS[@]} -gt 0 ] || STEPS=(walk pg-near cube-near pg-far cube-far)
BUILT=0
build() { if [ "$BUILT" = 1 ]; then echo --no-build; fi; }
for s in "${STEPS[@]}"; do
  case "$s" in
    walk)
      tools/client/vm-client.sh --name cmp-pg-walk --no-guidance --no-create --prop worldtype=deeptime:planet
      echo "SHOTS walk rc=$?"
      ;;
    pg-near|cube-near|pg-far|cube-far)
      w="${s%-*}16"; kind="${s#*-}"
      settle=45; [ "$kind" = far ] && settle=240
      V=$(cat "$CMP/vistas-$w-$kind.txt")
      # The dh set's mods are linked by the Gradle step: the first dh pass builds.
      tools/client/vm-client.sh --name "cmp-$s" --mods dh --no-guidance $(build) --save "$CMP/$w" \
        --prop "open=$w" --prop "settle=$settle" --prop "vista=$V"
      echo "SHOTS $s rc=$?"
      BUILT=1
      ;;
    pg-edges|cube-edges)
      # The near pass again from vista 9 (the edges, three with F3) into the near folder.
      w="${s%-*}16"
      V=$(cat "$CMP/vistas-$w-near.txt")
      tools/client/vm-client.sh --name "cmp-$s" --mods dh --no-guidance $(build) --save "$CMP/$w" \
        --prop "open=$w" --prop settle=45 --prop vistafrom=9 --prop "vista=$V"
      rc=$?
      cp review/out/cmp-"$s"/11-vista-*.png "review/out/cmp-${s%-*}-near/"
      echo "SHOTS $s rc=$rc"
      BUILT=1
      ;;
    *) echo "unknown step $s"; exit 2 ;;
  esac
done
echo "shots done"
