# Deep Time

Deep Time (`deeptime`) is a NeoForge 1.21.1 mod that simulates a planet's geologic history and then
lets you play that planet. A Rust engine moves tectonic plates on a sphere, erodes and deposits the
land, runs a parametric climate, forms rock strata and ore deposits, and records what happened in one
shared event log. The result is written into the world folder, and a Java adapter turns it into a
Minecraft overworld: terrain, climate-driven biomes (vanilla, Terralith, TerraBlender mods), strata
built from the pack's own stone blocks, and ore deposits that exist where the geology put them. The
world wraps east–west like a real planet (through Toroidal World) and has soft polar caps.

Deep Time ships no blocks of its own. It is built for the MIC modpack first and stays usable by any
pack.

**Status:** pre-alpha, Phase 0 (the bridge). See `STATUS.md`.

## Building

Everything runs inside the Nix dev shell:

```sh
nix develop -c cargo test --workspace               # Rust core, CLI, viewer
cd neoforge && nix develop -c ./gradlew build       # the mod jar (builds the host native library)
./gametest.sh                                       # headless GameTests
```

The standalone CLI runs any simulation stage without Minecraft and writes an HTML viewer:

```sh
nix develop -c cargo run -p deeptime-cli -- synth --kind wrap-test --size 16384 --out /tmp/w --viewer /tmp/v
```

## Documentation

- `docs/deeptime-design-decisions.md` — the design decisions (authoritative)
- `docs/deeptime-implementation-plan.md` — architecture, data model and the phase plan
- `docs/research/` — the research report and notes behind the design

## Licence

LGPL-3.0-or-later (`COPYING.LESSER`, which builds on `COPYING`). Third-party attributions are in
`NOTICE.md`.
