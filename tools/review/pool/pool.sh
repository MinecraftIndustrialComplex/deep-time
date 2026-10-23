#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The laptop's entry to the shared test-run pool (tools/review/pool/README.md; Ben, 2026-10-22: "If both
# deltas and continent spread need to do a world gen to test they should just merge their test."). Every
# heavy step runs on the test VM through tools/remote/mac.sh and the VM's admission tool; this script
# computes the source key (git only), starts the VM-side orchestrator detached, and waits for it.
#
#   tools/review/pool/pool.sh run <arm> [--key K] [--preset P] [--length Myr] [--starts a,b] [--seeds 1,2,3]
#                                [--threads T] [--set k=v]... [+ENV=v]... [--measures a,b] [--keep-ck]
#                                [--retry] [--rerun]
#   tools/review/pool/pool.sh table --base <arm> --arm <arm> [--arm <arm> ...] [--measure v1census]
#   tools/review/pool/pool.sh measure [--arm <arm>]... [--measures a,b]
#   tools/review/pool/pool.sh alias <new key|HEAD> <old key> --check-seed S [--preset P] [--length Myr]
#                                [--start a] [--threads T] [--set k=v]... [--all-arms]
#   tools/review/pool/pool.sh combine <name> <branch> [<branch> ...]      (combine.sh)
#   tools/review/pool/pool.sh forget <name>          delete the throwaway branch pool/<name>
#   tools/review/pool/pool.sh key [--ref REF]        this checkout's source key (refuses when dirty)
#   tools/review/pool/pool.sh arms | status | gc [--days N] [--apply] | keep <key> [reason] [--remove]
#
# `run` without --key uses this checkout's HEAD: the files the key covers must be committed. Arguments are
# plain words (mac.sh joins them into one line): no spaces or shell characters. Output is the VM job's log.
set -euo pipefail
REPO=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
cd "$REPO"
MAC=tools/remote/mac.sh
POOLPY=tools/review/pool/pool.py

die() { echo "pool.sh: $*" >&2; exit 2; }
usage() { sed -n '/^#   tools/,/^# plain words/p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//' >&2; exit 2; }

check_words() {
    local a
    for a in "$@"; do
        [[ "$a" =~ ^[A-Za-z0-9_.,=:+/@%-]+$ ]] || die "argument '$a' has a space or a shell character; mac.sh passes its arguments as one line, so use plain words"
    done
}

key_of() { python3 -I tools/review/pool/srckey.py key "$@"; }

# A quick look or a small change on the pool (the VM's admission tool is not asked: kind light).
light() {
    check_words "$@"
    "$MAC" --kind light -- python3 "$POOLPY" "$@"
}

# A long job: the orchestrator runs detached on the VM (it survives this session) and every heavy step it
# takes is admitted on its own. We wait on its log, 9 minutes at a time.
detached() {
    local what=$1; shift
    check_words "$@"
    local job rc tries=0
    job="pool-$what-$(date +%s)-$$"
    "$MAC" --kind light -- tools/remote/detach.sh start "$job" --kind light python3 "$POOLPY" "$@"
    echo "pool.sh: VM job $job (log target/detached/$job.log in the VM copy of $(basename "$REPO"))" >&2
    while :; do
        rc=0
        "$MAC" --no-sync --kind light -- tools/remote/detach.sh wait "$job" 540 200 || rc=$?
        case "$rc" in
            3) continue ;;
            255) tries=$((tries + 1)); [ "$tries" -le 20 ] || die "lost the VM $tries times; the job $job keeps running there"; sleep 15; continue ;;
            *) return "$rc" ;;
        esac
    done
}

provenance() {
    PROV=(--branch "$(git rev-parse --abbrev-ref HEAD | tr -c 'A-Za-z0-9._/@+\n-' '_')" --commit "$(git rev-parse --short=10 HEAD)")
}

verb=${1:-}
[ -n "$verb" ] || usage
shift
case "$verb" in
    key) key_of "$@" ;;
    run)
        arm=${1:-}; [ -n "$arm" ] && [ "${arm#-}" = "$arm" ] || die "run needs an arm name first: pool.sh run <arm> [options]"
        shift
        key=""; rest=()
        while [ $# -gt 0 ]; do
            case "$1" in
                --key) key=$2; shift 2 ;;
                *) rest+=("$1"); shift ;;
            esac
        done
        [ -n "$key" ] || key=$(key_of)
        provenance
        detached "$arm" run --key "$key" --arm "$arm" "${PROV[@]}" "${rest[@]}"
        ;;
    build)
        # The binary of a key (this checkout's HEAD unless --key), which combine.sh uses; `run` builds by itself.
        key=""; rest=()
        while [ $# -gt 0 ]; do
            case "$1" in
                --key) key=$2; shift 2 ;;
                *) rest+=("$1"); shift ;;
            esac
        done
        [ -n "$key" ] || key=$(key_of)
        provenance
        detached "build-$key" build --key "$key" "${PROV[@]}" "${rest[@]}"
        ;;
    alias)
        new=${1:-}; old=${2:-}; [ -n "$new" ] && [ -n "$old" ] || die "alias needs <new key|HEAD> <old key> --check-seed S"
        shift 2
        if [ "$new" = HEAD ] || [ "$new" = . ]; then new=$(key_of); fi
        provenance
        detached "alias-$new" alias "$new" "$old" "${PROV[@]}" "$@"
        ;;
    measure) detached measure measure "$@" ;;
    table | arms | status | gc | keep) light "$verb" "$@" ;;
    combine) exec tools/review/pool/combine.sh "$@" ;;
    forget)
        name=${1:-}; [ -n "$name" ] || die "forget needs the name given to combine"
        [[ "$name" =~ ^[A-Za-z0-9_.-]+$ ]] || die "bad name"
        git rev-parse --verify -q "refs/heads/pool/$name" > /dev/null || die "no branch pool/$name"
        git branch -D "pool/$name"
        echo "pool.sh: the pool's runs under that key stay until pool.sh gc (or pool.sh keep <key>)" >&2
        ;;
    -h | --help | help) usage ;;
    *) die "unknown verb '$verb' (run, table, measure, alias, combine, forget, key, arms, status, gc, keep)" ;;
esac
