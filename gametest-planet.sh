#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The dev planet server (plan §9.4): a dedicated server whose overworld is a real Deep Time
# planet with Toroidal World's wrap, where the seam, Sable and pole tests live.
#
#   ./gametest-planet.sh --boot-only                 start, wait for Done, probe column 0 0, stop
#   ./gametest-planet.sh --run columnscontinuous,entitycrossesseam@8185,250,0 [--run ...]
#                                                    one test at a time (a /test run cancels any running test)
#   ./gametest-planet.sh --run entitypushedback,hardstopat80,sablehullpushedback,worldinfochannelrequired@0,300,64
#                                                    the pole cap (P1-18; the tests load their polar chunks)
#   ./gametest-planet.sh --scenario sable_straddle   hull across the seam survives save + restart
#   ./gametest-planet.sh --scenario refuse_client    clients without Deep Time are refused (real TCP logins)
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
