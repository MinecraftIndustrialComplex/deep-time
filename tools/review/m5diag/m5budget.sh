#!/usr/bin/env bash
# The drowning diagnosis: the continental crust budget (tools/review/continent-budget.py) of an arm's cached runs
# that ran with DEEPTIME_CRUST_BUDGET=1 (VM): m5budget.sh <arm folder> [max seed]
# Where the continental area comes from and goes, process by process (parts and moves), and the thickness bands.
set -uo pipefail
cd "$(dirname "$0")/../../.."
top=${2:-999}
runs=()
for s in "$1"/*-*/summary.txt; do
  seed=$(basename "$(dirname "$s")")
  seed=${seed##*-}
  [ "$seed" -le "$top" ] && grep -q '^cb_t' "$s" && runs+=("$s")
done
echo "${#runs[@]} runs with a crust budget"
[ "${#runs[@]}" -gt 0 ] && python3 -I tools/review/continent-budget.py "${runs[@]}"
