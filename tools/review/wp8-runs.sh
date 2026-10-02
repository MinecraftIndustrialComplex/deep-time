#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Science stage 2, WP-8 (folds, faults, intrusions): measurement runs on the test machine,
# detached. Scratch in /Volumes/Graphide/ban/deeptime-runs/wp8/ (deleted when the package lands).
#
#   tools/remote/mac.sh -- tools/review/wp8-runs.sh world <name> [VAR=VALUE] [--set PATH=VALUE]...
#       builds the release CLI, copies it as deeptime-<name>, runs earthlike_quick_16k
#       (scattered_continents, 500 Myr, 3 threads) into <name>-world/, then strata-check
#       (<name>-strata.json) and structure-sections (<name>-sections/); log <name>.log.
#       A VAR=VALUE third argument is put in the run's environment. WP8_RUNS, WP8_START and
#       WP8_LENGTH override the scratch folder, the start and the length (the vertical-scale task:
#       WP8_RUNS=…/def-scale WP8_START=supercontinent WP8_LENGTH=1000).
#   tools/remote/mac.sh -- tools/review/wp8-runs.sh gates     (fmt, clippy, deny, cargo test, gradlew)
#   tools/remote/mac.sh --no-sync -- tools/review/wp8-runs.sh status
#   tools/remote/mac.sh --no-sync -- tools/review/wp8-runs.sh clean
set -euo pipefail

RUNS="${WP8_RUNS:-/Volumes/Graphide/ban/deeptime-runs/wp8}"
START="${WP8_START:-scattered_continents}"
LENGTH="${WP8_LENGTH:-500}"
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$RUNS"
cd "$REPO"

case "${1:-}" in
    world)
        name="$2"; shift 2
        envs=""
        case "${1:-}" in *=*) case "$1" in --*) ;; *) envs="$1"; shift ;; esac ;; esac
        sets=""; for a in "$@"; do sets="$sets '$a'"; done
        setsid nohup sh -c "cargo build --release -p deeptime-cli \
            && cp target/release/deeptime '$RUNS/deeptime-$name' \
            && rm -rf '$RUNS/$name-world' '$RUNS/$name-sections' \
            && env $envs /usr/bin/time -v '$RUNS/deeptime-$name' run --preset earthlike_quick_16k \
               --start '$START' --length-myr '$LENGTH' --threads 3 --out '$RUNS/$name-world' $sets \
            && '$RUNS/deeptime-$name' strata-check --world-data '$RUNS/$name-world' > '$RUNS/$name-strata.json' \
            && '$RUNS/deeptime-$name' structure-sections --world-data '$RUNS/$name-world' \
               --out '$RUNS/$name-sections' --half 300 --px 2 \
            && echo WP8-DONE" > "$RUNS/$name.log" 2>&1 < /dev/null &
        echo "started pid $! -> $RUNS/$name.log"
        ;;
    gates)
        # The package's gates, in the foreground: fmt, clippy, deny, the workspace tests, Gradle.
        set +e
        cargo fmt --all --check; echo "fmt=$?"
        cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -3; echo "clippy=${PIPESTATUS[0]}"
        cargo deny check licenses bans 2>&1 | tail -3; echo "deny=${PIPESTATUS[0]}"
        cargo test --workspace --no-fail-fast 2>&1 | grep -E "^test result|FAILED|panicked"; echo "test=${PIPESTATUS[0]}"
        (cd neoforge && ./gradlew build test -q 2>&1 | tail -5; echo "gradle=${PIPESTATUS[0]}"; \
            python3 ../tools/ci/junit-summary.py build/test-results/test 2>&1 | tail -3)
        ;;
    status)
        for f in "$RUNS"/*.log; do
            [ -e "$f" ] || continue
            echo "== $f"; tail -n "${2:-4}" "$f"
        done
        ;;
    clean)
        rm -rf "$RUNS"
        ;;
    *)
        sed -n 2,14p "$0"; exit 2
        ;;
esac
