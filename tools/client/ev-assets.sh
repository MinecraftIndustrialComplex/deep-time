#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Excavated Variants' generated assets, from a real client (plan Q10).
#
#   tools/client/ev-assets.sh            (on the laptop)
#
# EV registers an ore-in-stone block for every ore and stone it knows; Dynamic Asset Generator (DAG)
# makes their blockstates, models and textures when a CLIENT loads its resources. No jar holds them
# and a dedicated server never makes them, so BlueMap (and deeptime-inspect's colour table) has
# nothing to draw EV's blocks with. This boots the NeoForge dev client with the ev-client set
# (Deep Time, Toroidal World, EV, DAG, Create: every ore and stone EV knows in the MIC pack) on the
# Mac's desktop session with DAG's full cache and resource timing on, waits for the block atlas,
# stops the client and turns DAG's cache into a resource pack:
#   review/out/q10/ev-assets/pack/        pack.mcmeta + assets/ (BLUEMAP_PACKS, `colors --jars`)
#   review/out/q10/ev-assets/times.log    DAG's per-resource generation times
#   review/out/q10/ev-assets/summary.json counts, the summed times, the client's load timeline
#
# The desktop is shared with graphide's agents (CLAUDE.md): the run holds a "drive" lease of
# graphide's M6 lock (utilities/scripts/mactest/m6lock.sh, streamed over ssh) and the Deep Time
# server lock (mac.sh --server: memory). macOS gives a GUI app started from sshd no window server,
# so the Mac side starts the client through `launchctl asuser` (tools/client/ev-assets-mac.sh).
set -euo pipefail
REPO="$(git rev-parse --show-toplevel)"
HOST="${DEEPTIME_MAC:-ban@mac-mini}"
M6LOCK="${M6LOCK:-/home/greencheetah/Projects/graphide/graphide/utilities/scripts/mactest/m6lock.sh}"
HOLDER="deeptime-ev-assets-$(basename "$REPO")"
[ -f "$M6LOCK" ] || { echo "ev-assets.sh: no m6lock.sh at $M6LOCK (set M6LOCK)" >&2; exit 1; }

lease() { ssh -o BatchMode=yes -o ConnectTimeout=15 "$HOST" "bash -s -- $*" < "$M6LOCK"; }
if ! lease acquire drive "$HOLDER" "'Deep Time Q10: a Minecraft client makes Excavated Variants textures'" 2400; then
    echo "ev-assets.sh: the Mac's desktop is busy (m6lock above); try again later" >&2
    exit 75
fi
trap 'lease release drive "$HOLDER" >/dev/null 2>&1 || true' EXIT
"$REPO/tools/remote/mac.sh" --server --pull review/out/q10/ev-assets -- 'tools/client/ev-assets-mac.sh'
