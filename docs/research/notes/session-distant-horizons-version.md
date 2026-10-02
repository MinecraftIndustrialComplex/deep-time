# Distant Horizons: which version and API work with Deep Time on NeoForge 1.21.1

Date: 2026-09-28 (the plan to feed Distant Horizons) and 2026-10-01 to 2026-10-02 (the version
finding); all from the transcript's UTC timestamps.
Status: the coordinator's assessment, recorded from a working session. Not a literature review.

## Question

Distant Horizons (DH) is a mod that draws far-away terrain as low-detail blocks ("levels of detail",
LODs) beyond the normal view distance. Deep Time wants to give DH far terrain straight from its baked
world data, so that distant areas appear without generating chunks. Which DH versions and which part
of DH's API can Deep Time use on NeoForge 1.21.1, and does it conflict with the world-wrapping mod
Toroidal World (TW), which Deep Time depends on?

## Assessment

Plan, 2026-09-28. The coordinator proposed feeding DH through its world-generator interface
(`IDhApiWorldGenerator`) so that the whole planet gets far-view terrain without generating chunks,
including across the east-west wrap. The owner agreed to do it in a later phase (Phase 3). An early
research report noted that TW already supports DH and is pinned to nearly the versions of the target
modpack (MIC): TW compiles against DH 3.3.1 while MIC ships 3.3.0.

Work, 2026-10-01. On the owner's request the work was pulled forward and given to a sub-agent. Its
final report, relayed by the coordinator the same day and again on 2026-10-02, said the following.

- Far terrain works with DH 3.3.0, the version MIC ships, on a dedicated server. It is on whenever DH
  is installed and off with a system property.
- Versions and API: DH 3.3.0 and 3.3.1 are API 7.1.0, and 3.3.3 is API 7.2.0. A line in the plan
  saying "API 4.0.0 renamed methods" is stale. The members Deep Time uses are identical in all
  three versions: registering a generator through `DhApi.worldGenOverrides` and implementing
  `IDhApiWorldGenerator.generateLod(...)`, returning data sources.
- A packaging fault in DH 3.3.0: its jar renames every NeoForge class with a `_neoforge` suffix,
  including the API, and the unsuffixed `DhApi` in that jar is a dead Fabric copy. A mod compiled
  against the published API would register into a dead object. DH 3.3.1 fixed this. Deep Time
  therefore binds to DH by name at run time, with a dynamic proxy and method handles, and a test
  opens the real 3.3.0 and 3.3.3 jars to prove it.
- Conflict with Toroidal World 0.19.3: TW wraps DH's generator binding and throws for any generator
  that is not DH's own on a wrapped level. With DH 3.3.0, TW's DH mixins target the unsuffixed
  classes that NeoForge never loads, so they do nothing and Deep Time's generator registers. With
  DH 3.3.1 and later the mixins apply and the generator is refused, and DH falls back to real chunks.
  The coordinator's summary: updating DH in the pack would turn far terrain off. It also noted that
  DH 3.3.3's Chunky hook crashes the boot in the test setup.
- Wrapping itself needs nothing from TW, because every sample folds x. Past the pole edge the data
  clamp as real chunks do.
- Measured on a loaded 32-core x86 test server: one section of far terrain takes about 9.6 to 11.1
  ms single-threaded (detail 0), 37 to 44 ms inside DH during pre-generation and 52 ms in the full
  pack. A DH pre-generation of 257 sections finished with no failures. Against the chunks generated
  afterwards, water and biome matched in nearly all columns and ground height was within one block in
  75 to 94 % over four planets. The top block was right 85 to 96 % of the time on vanilla biomes but
  only about 55 % in MIC, because the Terralith biome mod has no entry in the lookup table.
- Not verified: how it looks. No client with a graphics card ran. Far terrain has no trees, caves or
  deposits. Singleplayer and DH sending LODs to a connected client are untested.

The coordinator's advice to the owner: keep MIC on DH 3.3.0 and ask TW's author for a way to declare
a seam-aware generator before any DH update. On 2026-10-02 it described the feature as working on the
server side only, tested by server-side GameTests, and tied to the DH version MIC ships.

## What was sourced and what was not

- The version and API facts: a sub-agent (Opus model) checked them with `javap` and a decompile of the
  real DH 3.3.0, 3.3.1 and 3.3.3 jars and of TW's code, then ran smoke tests on a server. The
  coordinator relayed the report and did not repeat the checks. The results are recorded in
  `STATUS.md` (build log for P3-04, and the pending-questions row).
- "TW compiles against 3.3.1, MIC ships 3.3.0": the wrapped-planet research note,
  `docs/research/notes/wrapped_planet_worlds.md`, which reads the pack's DH pin.
- Timings and match rates: measurements by the sub-agent in this repository, on one loaded machine
  and a few planets.
- The statement that DH's API is LGPL-3.0 is in the plan and came from earlier research; no
  licence text was read in this session.
- Nothing on how DH looks was observed.

## What was decided

On 2026-10-01 Ben asked for the Distant Horizons work to run in parallel with the rest, and the
decisions log (`docs/deeptime-design-decisions.md`) records that P3-04 was pulled forward. The four
questions the agent left are listed in `STATUS.md` as pending and had not been put to the owner when
the transcript ends: (1) keep MIC on DH 3.3.0 and ask TW's author (recommended), add a Deep Time
mixin into TW's guard, or accept that a DH update turns far terrain off; (2) a Terralith surface table
for the 55 % top-block match; (3) trees; (4) whether the on/off switch becomes a setting. No version
decision was made.

## Open

- All four questions above.
- How far terrain looks in a real client, including the hand-over to real chunks.
- Behaviour with DH versions after 3.3.3, and with TW versions after 0.19.3.
