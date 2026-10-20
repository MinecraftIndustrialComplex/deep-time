#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Excavated Variants' generated assets, from a real client (plan Q10).
#
#   tools/client/ev-assets.sh [--no-build] [--timeout S] [--lease-wait S]     (on the laptop)
#
#   --no-build    skip the sync and the Gradle step (the Mac copy is built, the ev-client set linked)
#   --timeout S   the client's limit, default 1500 s (EV_ASSETS_TIMEOUT sets the default)
#   --lease-wait  how long to wait for the Mac's desktop (graphide's M6 lock), default 600 s
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
# It is a client run like tools/client/mac-screenshots.sh's, and shares its launch: (1) mac.sh syncs
# the checkout, installs the ev-client set if the Mac has none (tools/smoke/install.sh ev-client) and
# runs `./gradlew -PrunSet=ev-client classes gametestClasses createClientLaunchScript
# linkRunModsClient`; (2) graphide's m6lock.sh is copied to the Mac; (3) `mac.sh --no-sync --server`
# runs tools/client/ev-assets-mac.sh there, as a server job of the VM's admission tool (memory: 16 GB
# shared), which takes graphide's `drive` lease, renews it while the client runs, starts the client
# in ban's desktop session (sudo -n launchctl asuser, the window server macOS gives no process
# started from sshd) and kills only its own client on any failure (tools/client/mac-client-lib.sh);
# (4) mac.sh --pull brings review/out/q10/ev-assets/ back. Ctrl-C stops the Mac side too. Exit 75:
# the desktop stayed busy; exit 4: nobody is logged in at the Mac.
set -euo pipefail
cd "$(dirname "$0")/../.."
REPO="$(git rev-parse --show-toplevel)"

BUILD=1; PASS=(); TIMEOUT="${EV_ASSETS_TIMEOUT:-1500}"
while [ $# -gt 0 ]; do
    case "$1" in
        --no-build) BUILD=0; shift ;;
        --timeout) TIMEOUT="$2"; shift 2 ;;
        --lease-wait) PASS+=("$1" "$2"); shift 2 ;;
        -h|--help) sed -n '2,/^set -euo/p' "$0" | sed 's/^# \{0,1\}//' | sed '$d'; exit 0 ;;
        *) echo "ev-assets: unknown argument $1 (see --help)" >&2; exit 2 ;;
    esac
done
[[ "$TIMEOUT" =~ ^[0-9]+$ ]] || { echo "ev-assets: --timeout is seconds (got '$TIMEOUT')" >&2; exit 2; }

TOOL=ev-assets
source tools/client/mac-client-laptop-lib.sh
mcl_check_m6lock

if [ "$BUILD" = 1 ]; then
    tools/remote/mac.sh -- 'set -eo pipefail; [ -f smoke/ev-client/install.json ] || tools/smoke/install.sh ev-client 2>&1 | tail -3; cd neoforge && ./gradlew --console=plain -q -PrunSet=ev-client classes gametestClasses createClientLaunchScript linkRunModsClient'
fi
mcl_push_m6lock
mcl_stop_on_interrupt

set +e
tools/remote/mac.sh --no-sync --server --pull review/out/q10/ev-assets -- \
    "tools/client/ev-assets-mac.sh --m6lock '$CACHE/m6lock.sh' --timeout $TIMEOUT ${PASS[*]:-}"
status=$?
set -e
trap - INT TERM
echo "ev-assets: exit $status; output review/out/q10/ev-assets/ (summary.json, pack/, times.log, client.log, runner.log)"
exit "$status"
