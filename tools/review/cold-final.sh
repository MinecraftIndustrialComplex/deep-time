#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Final gates of the cold-desert and schist branch under one hold of the server lock: the three
# GameTest profiles (tools/review/cold-schist-gates.sh gametest).
#   tools/remote/mac.sh --pack -- tools/review/detached-server.sh <log> tools/review/cold-final.sh
set -uo pipefail
cd "$(cd "$(dirname "$0")/../.." && pwd)"
tools/review/cold-schist-gates.sh gametest
