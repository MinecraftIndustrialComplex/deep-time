#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
# Extracts the NeoForge + Minecraft sources jar that any Gradle build produces into the
# gitignored libs/neoforge-src/, for grepping (plan §2.2). Re-run after a NeoForge bump.
set -euo pipefail
cd "$(dirname "$0")/../.."
JAR=neoforge/build/moddev/artifacts/neoforge-21.1.249-sources.jar
[ -f "$JAR" ] || { echo "sources.sh: $JAR missing -- run (cd neoforge && nix develop -c ./gradlew createMinecraftArtifacts)" >&2; exit 1; }
rm -rf libs/neoforge-src && mkdir -p libs/neoforge-src
unzip -q -o "$JAR" -d libs/neoforge-src
echo "extracted $(find libs/neoforge-src -name '*.java' | wc -l) sources into libs/neoforge-src"
