#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Ice spikes report (Ben, 2026-10-20: "like 20x more ice spike biomes than I'd usually see"): a baked planet's
# rare-biome shares against a normal world of the same pack. Scratch: /Volumes/Graphide/ban/deeptime-runs/icespikes/.
#
#   tools/remote/mac.sh --kind cargo:build -- tools/review/ice-spikes.sh planet <name> <preset.json> [run args]
#       builds this tree's release CLI and runs that preset (a play world's own preset.json) again, 8 threads,
#       detached through the admission tool, into <name>-world/ (log <name>.log); then `dump` it
#   Before and after: `--placement-patch /Volumes/Graphide/ban/deeptime-runs/icespikes/no-ef-floor.json` (written
#   by every call) bakes the planet without EF's continentalness floor (BIO-161), as before 2026-10-20.
#   tools/remote/mac.sh --kind cargo:build -- tools/review/ice-spikes.sh rebake <from name> <name>
#       builds this tree's release CLI, copies <from name>-world/ to <name>-world/ and resumes it to its own stop
#       (only the run's tail and the bake run again), detached; the same planet baked by this tree
#   tools/remote/mac.sh --no-sync -- tools/review/ice-spikes.sh dump <name> <world-data folder>
#       the baked biome inputs, Koppen classes and ice cover every 16 blocks on the census's grid
#       (deeptime heightmap; mac.sh admits it) into <name>-layers.npy
#   tools/remote/mac.sh [--pack] -- tools/review/ice-spikes.sh census <standalone|mic> <name> <world-data folder>
#       installs the profile's smoke set (16k), boots it on a copy of that world data and runs
#       `/deeptime probe biomes 16` and `/deeptime probe reference_biomes` (the pack's own router and parameter
#       list, 4 seeds, 400,000 blocks square every 512); writes <name>-<profile>-biomemap.json,
#       <profile>-reference.json, <profile>-reference-parameters.json and <name>-<profile>-climate_mapping.json
#   tools/remote/mac.sh --no-sync -- tools/review/ice-spikes.sh shares <name> <profile>
#       tools/review/ice_spikes.py over them, against both references when both exist
#   tools/remote/mac.sh --kind cargo:build -- tools/review/ice-spikes.sh testplanet <name> <tree|binary> [run args]
#       the GameTests' 4k planet (planet_world_4k) baked by this tree or a saved CLI, dumped into <name>-layers.npy
#   tools/remote/mac.sh --kind sim:planet_world_4k:8t -- tools/review/ice-spikes.sh testplanets <prefix>
#       `testplanet` four times with this tree (8 threads): <prefix>-{before,after}-{std,mic}, mic with Terralith's
#       table as gametest.sh -PwithMic bakes it, before without EF's continentalness floor
#   tools/remote/mac.sh --no-sync -- tools/review/ice-spikes.sh emulate <ice_spikes.py arguments>
#   tools/remote/mac.sh --no-sync -- tools/review/ice-spikes.sh clean
set -euo pipefail

RUNS=/Volumes/Graphide/ban/deeptime-runs/icespikes
PYENV='(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy])'
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"
printf '%s\n' '{"schema": 1, "id": "no-ef-floor", "patch": {"classes": {"EF": {"continentalness": null}}}}' > "$RUNS/no-ef-floor.json"

case "${1:-}" in
    planet)
        name="${2:?name}"; preset="${3:?preset.json}"; shift 3
        CARGO_BUILD_JOBS=8 cargo build --release -q -p deeptime-cli
        cp target/release/deeptime "$RUNS/deeptime-$name"
        rm -rf "$RUNS/$name-world"
        setsid nohup bash -c 'deeptime-admit run --kind sim:earthlike_quick_16k:8t --label "icespikes-$1" -- \
            "$2/deeptime-$1" run --preset "$3" --threads 8 --out "$2/$1-world" "${@:5}" && "$4" dump "$1" "$2/$1-world"' \
            _ "$name" "$RUNS" "$preset" "$0" "$@" > "$RUNS/$name.log" 2>&1 < /dev/null &
        echo "started pid $! -> $RUNS/$name.log"
        ;;
    rebake)
        from="${2:?from name}"; name="${3:?name}"
        CARGO_BUILD_JOBS=8 cargo build --release -q -p deeptime-cli
        cp target/release/deeptime "$RUNS/deeptime-$name"
        rm -rf "$RUNS/$name-world"
        cp -a "$RUNS/$from-world" "$RUNS/$name-world"
        setsid nohup deeptime-admit run --kind sim:earthlike_quick_16k:8t --label "icespikes-$name" -- \
            "$RUNS/deeptime-$name" resume --out "$RUNS/$name-world" --threads 8 \
            > "$RUNS/$name.log" 2>&1 < /dev/null &
        echo "started pid $! -> $RUNS/$name.log"
        ;;
    dump)
        name="${2:?name}"; world="${3:?world-data folder}"
        bin="$REPO/target/release/deeptime"
        [ -x "$bin" ] || CARGO_BUILD_JOBS=8 cargo build --release -q -p deeptime-cli
        # The census's grid (BiomeCensus: step 16, x0 = -C/2 + 8, z0 = -zp + 8; zp the projection's z edge, C/4 on
        # Petroff-Guyou, the pole C/2pi on Lambert).
        read -r c zp < <(python3 -c 'import json, math, sys; m = json.load(open(sys.argv[1])); c = m["scale"]["circumference_blocks"]
d = m.get("projection", {}).get("domain")
print(c, int(d["z_max"]) if d else int(c / (2 * math.pi)))' "$world/manifest.json")
        x0=$(( -c / 2 + 8 )); x1=$(( x0 + c ))
        z0=$(( -zp + 8 )); z1=$(( z0 + 2 * zp ))
        "$bin" heightmap --world-data "$world" --box="$x0,$z0,$x1,$z1" --step 16 \
            --layers climate/temperature,climate/vegetation,climate/continents,climate/erosion,climate/ridges,climate/koppen_l2,climate/koppen,climate/ice_cover,climate/wetland,climate/t_mean_c,climate/p_mm \
            --out "$RUNS/$name-layers.npy"
        ls -la "$RUNS/$name-layers.npy" "$RUNS/$name-layers.npy.json"
        ;;
    census)
        profile="${2:?profile}"; name="${3:?name}"; world="${4:?world-data folder}"
        [ -d "$world" ] || { echo "no world data at $world" >&2; exit 1; }
        tools/smoke/install.sh "$profile" --size 16384
        # The world data's own level type (install.sh without a preset picks Lambert): Petroff-Guyou worlds
        # are deeptime:planet (13,312 blocks round), Lambert ones deeptime:planet_lambert (16,384).
        level_type=$(python3 -c 'import json,sys; m = json.load(open(sys.argv[1])); c = m["scale"]["circumference_blocks"]
pg = m.get("projection", {}).get("kind") == "petroff_guyou"
print("deeptime:planet" + ("" if pg else "_lambert") + ("" if c <= 16384 else "_32k"))' "$world/manifest.json")
        server="smoke/$profile/server"
        rm -rf "$server/planet"
        mkdir -p "$server/planet"
        cp -R "$world" "$server/planet/deeptime"
        rm -rf "$server/planet/deeptime/epochs"
        # The server runs in a private copy of its set: the probes write where the mod is told to.
        PROBE="$RUNS/probe-$profile-$name"; rm -rf "$PROBE"; mkdir -p "$PROBE"
        DEEPTIME_PROBE_DIR="$PROBE" python3 tools/smoke/run.py --profile "$profile" --level-type "$level_type" --timeout 3000 \
            --rcon "deeptime probe biomes 16" --rcon "deeptime probe reference_biomes" || true
        cp "$PROBE/biomemap.json" "$RUNS/$name-$profile-biomemap.json"
        cp "$PROBE/reference_biomes.json" "$RUNS/$profile-reference.json"
        cp "$PROBE/reference_biomes-parameters.json" "$RUNS/$profile-reference-parameters.json"
        cp "$server/planet/deeptime/climate_mapping.json" "$RUNS/$name-$profile-climate_mapping.json" 2>/dev/null || true
        ;;
    testplanet)
        # The GameTests' 4k planet (gametest.sh: planet_world_4k, Terralith's table with MIC), baked by <binary>
        # (`tree`: this tree's release CLI) with any further `deeptime run` arguments, then dumped like `dump`.
        name="${2:?name}"; bin="${3:?binary or tree}"; shift 3
        if [ "$bin" = tree ]; then
            CARGO_BUILD_JOBS=8 cargo build --release -q -p deeptime-cli
            bin="$RUNS/deeptime-tree"; cp target/release/deeptime "$bin"
        fi
        rm -rf "$RUNS/$name-world"
        "$bin" run --preset crates/deeptime-core/tests/presets/planet_world_4k.json "$@" --out "$RUNS/$name-world" \
            > "$RUNS/$name.log" 2>&1
        "$0" dump "$name" "$RUNS/$name-world"
        ;;
    testplanets)
        prefix="${2:?prefix}"
        terralith=crates/deeptime-core/data/climate_mapping/packs/terralith.json
        "$0" testplanet "$prefix-after-std" tree --threads 8
        "$0" testplanet "$prefix-before-std" "$RUNS/deeptime-tree" --threads 8 --placement-patch "$RUNS/no-ef-floor.json"
        "$0" testplanet "$prefix-after-mic" "$RUNS/deeptime-tree" --threads 8 --placement-patch "$terralith"
        "$0" testplanet "$prefix-before-mic" "$RUNS/deeptime-tree" --threads 8 --placement-patch "$RUNS/no-ef-floor.json" \
            --placement-patch "$terralith"
        ;;
    emulate)
        # Any arguments of tools/review/ice_spikes.py.
        shift
        nix shell --impure --expr "$PYENV" -c python3 tools/review/ice_spikes.py "$@"
        ;;
    shares)
        name="${2:?name}"; profile="${3:?profile}"
        refs=()
        for p in standalone mic; do
            [ -e "$RUNS/$p-reference.json" ] && refs+=(--reference "$p=$RUNS/$p-reference.json")
        done
        census=()
        [ -e "$RUNS/$name-$profile-biomemap.json" ] && census=(--census "$RUNS/$name-$profile-biomemap.json")
        nix shell --impure --expr "$PYENV" -c python3 tools/review/ice_spikes.py --layers "$RUNS/$name-layers.npy" \
            --mapping "$RUNS/$name-$profile-climate_mapping.json" --params "$RUNS/$profile-reference-parameters.json" \
            "${census[@]}" "${refs[@]}"
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *)
        sed -n '3,31p' "$0"; exit 2
        ;;
esac
