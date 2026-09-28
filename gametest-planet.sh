#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The dev planet server (plan §9.4): a dedicated server whose overworld is a real Deep Time
# planet with Toroidal World's wrap, where the seam, Sable and pole tests live.
#
#   ./gametest-planet.sh --boot-only                 start, wait for Done, probe column 0 0, stop
#   ./gametest-planet.sh --run columnscontinuous,entitycrossesseam@8185,250,0 [--run ...]
#   ./gametest-planet.sh --run prototypepushback@0,300,3527~16,160   forceload x±16, z±160
#   ./gametest-planet.sh --scenario sable_straddle   hull across the seam survives save + restart
#   ./gametest-planet.sh --size 32768 ...            the 32k preset
#   ./gametest-planet.sh --test-terrain wrap-test    the analytic terrain (the default until P0-10)
#   ./gametest-planet.sh ... -- -PwithMic            extra Gradle arguments after --
#
# Needs the standalone server set (tools/smoke/install.sh standalone), whose jars Gradle links
# into run-planet/mods. Namespace `deeptime_planet`; results come from Deep Time's reporter
# ([gametest-result] / [gametest-summary] lines). Log: run-planet/planet.log.
set -uo pipefail
cd "$(dirname "$0")"
exec python3 tools/planet/run_planet.py "$@"
