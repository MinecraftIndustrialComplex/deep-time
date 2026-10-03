#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cold deserts and schist: the GameTests of the three profiles and the biome census, under one hold of
# the test machine's server lock (it is shared, and a wait for each would be hours).
#
#   tools/remote/mac.sh --pack --server -- tools/review/cold-all.sh
set -uo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
tools/review/cold-schist-gates.sh gametest
tools/review/cold-census.sh c-oldmap c-after
