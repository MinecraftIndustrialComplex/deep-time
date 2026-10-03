# A pillowcase planet from Immersive Portals – CE: throwaway prototype

Run 2026-10-02/03 for Deep Time. Ben asked whether the world could be a seamless "pillowcase" (quincuncial) planet built
from Immersive Portals – CE (IP-CE) global portals instead of a new wrap engine. His words: "I'd honestly rather proper
traversability of the world for players than the ability to build a create / redstone contraption across the seam."

This note follows [`rotated-seam-worlds-prior-art.md`](rotated-seam-worlds-prior-art.md) and
[`world-projection-options.md`](world-projection-options.md). It changes nothing in the Deep Time repo or the pack.

How it was run:
- Everything ran on the test VM, with a real client under Xvfb and Mesa llvmpipe (software GL, about 10 to 25 fps).
- IP-CE was v6.0.10 (tag `v6.0.10`, commit `84eccb5`) on NeoForge 21.1.249.

**Tags.**
- **[R]** I ran it and saw it: a screenshot, a log line or an RCON readout.
- **[C]** I read it in IP-CE's or Deep Time's source.
- **[I]** My inference.
- **[U]** Not verified.

---

## 0. Verdict

**No for MIC's pack as it stands. Yes for players in a small pack, with named breakages.**

- **The idea works for players [R].** Global portals with a 180° rotation make a walkable pillowcase on a 2048 × 1024
  test world.
  - Walking, sprinting and creative flight cross both the slide seam and the half-turn seams.
  - So do items, arrows, minecarts (one carrying a villager), boats and pushed mobs.
  - The view is continuous and the player comes out turned 180°. Nothing rubber-bands.
  - Blocks can be placed and broken through a seam.
- **MIC's pack does not load with IP-CE [R].**
  - It crashes at mod construction against **Petrolpark**, which takes Destroy, MIC Destroy Electric and MIC
    Petrochem with it.
  - With those four removed, it crashes against **CBC Modern Warfare**.
  - With all five removed, the server and the client both load: C2ME, Sodium 0.8.13, Veil, Simple Clouds, Distant
    Horizons 3.3.1, Create, Sable, Cobblemon and Project Atmosphere included.
- **Toroidal World (TW) and IP-CE cannot run together [R].** Every join is kicked with `EncoderException: Failed to
  encode packet 'clientbound/minecraft:player_position'`. Deep Time declares TW required, so **Deep Time cannot run
  with IP-CE without a change**. A two-line scratch edit to the jar was enough to boot it (§5).
- **With Sable present, the pack server crashed four times in about 15 minutes near the seams [R].**
  - Each crash was while ticking a freshly dropped item (seeds, sand, kelp) 30 to 120 blocks beyond a seam plane.
  - The error is a NullPointerException in Sable's `Entity.move` hook: `sable$collisionInfo` is null.
  - IP-CE chains its own collision wrapper onto the same call [C]. The interaction is my inference; I did not run a
    control without IP-CE.
- **Sable ships do not cross a seam [R][C].** IP-CE hands Sable bodies over only through portals into *another*
  dimension, and every seam portal is same-dimension.
- **Distant Horizons: not shown working [U].** In the time available, the LODs did not build and the server kept
  crashing (§7).

**What it would take.**
- MIC would have to drop Destroy (and Petrolpark) and CBC Modern Warfare.
- Deep Time would have to stop requiring TW.
- After that, the blockers are the Sable crash, Sable ships, and Distant Horizons. None is a setting.

---

## 1. What was built

- **Layout (Guyou frame).** The world is the rectangle |x| ≤ HX, |z| ≤ HZ with HX = 2·HZ (test world: HX = 1024).
  - **The short edges** are joined by a slide of 2·HX: portal E at x = +HX sends to x = −HX, and W does the reverse.
  - **Each long edge** is joined to itself by a 180° turn about its midpoint, which is the map (x, z) → (−x, 2·HZ − z).
- **Two layouts tried.**
  - **Six portals:** E, W, and two half-edge portals per long edge, each sending to the other half.
  - **Four portals:** E, W, and **one 2·HX-wide portal per long edge whose destination is its own centre**, turned 180°.
    The two half-edge portals of the six-portal layout are the same isometry, so one portal per edge is enough.
  - **Use the four-portal layout.** It closes the hole at the edge-midpoint turn points (§3).
- **Each portal is made the same way.**
  1. `summon immersive_portals:portal x y z {width, height, axisW, axisH, dimensionTo, destinationX/Y/Z}`.
  2. For the turned seams, add `rotationA:0f, rotationB:0f, rotationC:1f, rotationD:0f`. This is the quaternion (w, x,
     y, z) = (0, 0, 1, 0), a half-turn about the vertical [C, `Portal.java` NBT keys].
  3. Run `portal global convert_normal_portal_to_global_portal` as the portal entity.
  4. Each portal faces inward, so the world is closed and one-way at every seam.
- **Three traps found on the way.**
  1. **Summon coordinates are centred.** Integer summon coordinates land on the block centre (+0.5). The first
     build's planes sat half a block off. Write the coordinates as decimals [R].
  2. **A fresh global portal moves non-player entities only near its centre, until the server restarts.**
     - `GlobalPortalStorage.addPortal` sets `isGlobalPortal` without refreshing the bounding-box cache.
     - So a fresh global portal keeps the "normal portal" box, capped at ±64 blocks around its centre [C].
     - Items 250 blocks below the centre went straight through [R].
     - After a restart the portals load from storage with the full box, and everything crossed [R].
     - A one-line IP-CE fix (`updateCache()` in `addPortal`), or a restart after building the seams, avoids it.
  3. **The conversion needs a loaded chunk.** It silently fails if the portal's chunk is still generating. On the Deep
     Time planet this left four plain, non-global portals in the world [R].
- **The "outside".** The chunks beyond each plane are ordinary world chunks. The portals hide them, but they are
  generated, loaded and ticked like any others. Water, pushed blocks and escaping entities end up there (§2, §3).

---

## 2. Crossing table (vanilla-plus test world unless noted)

| What | Slide seam (E/W) | Half-turn seam (long edge) | Evidence and notes |
|---|---|---|---|
| Player walking | Crosses | Crosses, comes out turned 180° | [R] Positions logged every 0.2 s are monotonic, with no step backwards. Video `vp2-W1-slide.mp4`, `vp2-W2-halfturn.mp4` |
| Sprinting, creative flight | Crosses (Deep Time planet, 22 m/s) | Crosses (22 m/s; flight state kept, yaw turned) | [R] `dt-W1-E-fly.mp4` |
| Item, arrow | Crosses | Crosses, motion reversed | [R] `vp2-S-entities.mp4`, `vp2-E-entities.mp4` |
| Minecart, empty and with a villager rider | Crosses, lands on the far rails | Crosses, lands on the far rails, rider kept | [R] |
| Boat | Crosses | Crosses; motion reversed, **yaw not turned** | [R] Rotation stayed 0 while velocity flipped |
| Mob pushed across (pig, cow, sheep, armour stand) | Crosses | Crosses, but **lands 0.05 block outside the far plane** | [R][C] The landing nudge is the pre-rotation direction (`getRegularEntityTeleportedEyePos`). On a half-turn seam that points outward. If the landing chunk is not ticking, the mob freezes there until a player comes near |
| Mob walking under its own AI | Crosses if its wandering takes it there | Same | [R] A husk wandered across |
| Mob seeing or pathing across | No | No | [R] A husk ignored a survival player 8 blocks away through the seam, then attacked within 8 s when the player stood on its side |
| Snowball | Broke before reaching the plane (inconclusive) | Same | [R] |
| Placing and breaking blocks through the seam | — | Works | [R] An emerald block placed through the half-turn seam landed at the far-side image position; a diamond block on the far side was broken through it |
| Water and lava | — | Flow up to the plane and stop there with a straight edge. They keep flowing in the hidden outside, never across | [R] `vp2-fluids-piston.png` |
| Piston | — | Pushes the block into the hidden outside; it vanishes from view | [R] |
| Redstone, Create machines, conveyors across the seam | No | No | [I] No block adjacency exists across a portal. This matches Ben's acceptance |
| Sable ship | **No**; flew through the plane into the terrain beyond | Not tried | [R] A pushed Sable platform passed x = 8192 untouched. [C] `SableDimensionStackCompat.findCrossedPortal` requires `!portal.getDestDim().equals(level.dimension())` |
| Create train or contraption | Not tested | Not tested | [U] At start-up IP-CE itself logs "Detected a mildly incompatible mod: Create … Sometimes some contraptions will be invisible" [R] |
| Chunk loading across | The client receives chunks through the portal (counts rose from 441 to 588–901). IP-CE caps indirect loading at 8 chunks | Same | [R] Entities that cross into a chunk that is not ticking freeze until a player arrives |
| Maps, waypoints, Antique Atlas, chunk loaders | Not tested | Not tested | [I] They all work in raw coordinates. A map near a seam shows the hidden outside, not the far side |

---

## 3. Turn points, corners and what you see

- **At a turn point you see yourself and the same terrain twice [R].**
  - Walking toward an edge-midpoint turn point, your own avatar walks toward you, mirrored through the point.
  - At the moment of crossing it fills the screen face to face (`m-vp4-W4.png`).
  - Marker pillars appear twice, the second copy turned 180°.
  - The image was stable in every frame captured: no flicker, no missing chunks.
- **Edge midpoints: the six-portal layout leaks [R].** Walking exactly through x = 0 on a long edge, the boundary
  between the two half-edge portals, crossed neither, and the player walked out of the world. The four-portal layout
  fixes this: the same walk, and a diagonal one through the point, turned correctly.
- **Corners leak in both layouts [R].** The corner (±HX, ±HZ) is where the slide seam meets a long edge, and it is the
  other turn point of that edge.
  - A walk aimed exactly at the corner crossed the slide portal on its edge line and came out just past the far
    corner, outside the long-edge portal. The player ended up outside the world.
  - Aimed 0.2 block off, the same walk did the expected double teleport within 0.8 s.
  - The hole is a line of zero width, but players, items and mobs can reach the hidden outside through it.
  - The portals are one-way, so an escaped player can walk back in.
  - A solid wall built just outside the planes would probably plug it. [U] untested.
- **Who sees what in multiplayer [I].** Every player needs IP-CE; the server rejects clients without it by default
  (`serverRejectClientWithoutImmPtl`) [C]. Two players near a turn point would each see the other twice. Only one
  player was tested.

---

## 4. Performance (llvmpipe; absolute numbers are meaningless, only ratios count)

- **The VM was busy.** Its load average ran between 30 and 140, from other jobs, during these runs. Every number is
  noisy.

| Where (vanilla-plus, four portals, render distance 10) | Client fps, three samples |
|---|---|
| Interior, no portal in view | 22 22 24, then 20 19 20 |
| At the long edge, looking at it | 18 18 20 |
| Same spot, looking away | 21 17 19 |
| Near the edge-midpoint turn point, looking at it (two images) | 17 13 15 |
| At a corner, two portals in view | 13 13 12 |

- **Client [R].** Looking through one seam costs about 10–15 % of frame rate. Two portal views (turn point or corner)
  cost 30–40 %. With seams in view the client holds 30–100 % more chunks (441 → 588–901).
- **Server [R].**
  - Vanilla-plus: 6.0–7.0 ms/tick with four global portals, against 3.8–5.2 ms/tick with none, measured on another
    instance at another time. No cost stands out from the noise.
    - [C] Global portals add one bounding-box test per entity per portal per tick
      (`CollisionHelper.updateCollidingPortalForWorld`).
  - Deep Time planet without TW: 7–10 ms/tick, and 19 ms/tick while generating near the pole.
  - Pack without TW: 9.6–19 ms/tick.
  - Pack client frame rate with versus without portals was not measured; the server kept crashing.

---

## 5. Deep Time terrain and Toroidal World

- **TW with IP-CE fails at join, not at boot [R].**
  - Two runs: Deep Time 0.0.1 + TW 0.19.3 + IP-CE + Cloth Config, and later the full pack. Both servers boot.
  - Every client join is kicked: `EncoderException: Failed to encode packet 'clientbound/minecraft:player_position'`.
  - The cause is `NullPointerException: Cannot invoke "ResourceKey.location()"` inside IP-CE's mixin
    `ClientboundPlayerPositionPacket.handler$…$immersive_portals_core$onWrite`. IP-CE writes a dimension key that is
    null on that packet.
  - [I] TW builds or rewrites position packets on a path IP-CE's constructor hook never sees. That matches TW issue
    #42.
- **Where Deep Time assumes TW [C].**
  - `neoforge/src/main/templates/META-INF/neoforge.mods.toml`: `toroidal_world` is `required`. Without TW, FML refuses
    to load.
  - `wrap/PlanetShape.java` is called unconditionally from `mixin/WorldDimensionsShapeMixin.java` on
    `WorldDimensions.bake`. It uses TW's `ShapeDimensions`/`LoopSpans`, so the first bake would fail with
    `NoClassDefFoundError`.
  - `server/DeepTimeCommands.java` has two `/deeptime probe tw|sable` paths that call `ToroidalWorldApi`. They only
    fail when the command is run.
  - The GameTests (`BootGameTests`, `SeamGameTests`, `SableSeamGameTests`, `EditorModelGameTests`) and
    `linkRunMods*` assume TW.
  - The chunk generator, biome source, rasters, `FarTerrain` and the Distant Horizons generator need nothing from TW.
    They fold x themselves (`TerrainSource.wrapX`) and clamp z.
- **The scratch switch.** It was made on a copy of the jar on the VM, not in the repo [R].
  1. `toroidal_world` set to `optional` in the jar's `neoforge.mods.toml`.
  2. `WorldDimensionsShapeMixin` removed from `deeptime.mixins.json`.
- **Results with the switch [R].**
  - With the dh-video planet (C = 16384, 300 m/block) and IP-CE, Deep Time booted, bound its world data and generated
    terrain.
  - The level.dat Nether and End entries that name TW's generator fail to decode and are dropped. Only the overworld
    was tested.
  - The pack also boots with the switch.
  - **What a real change costs [I].** The mods.toml line, a guard or a TW-only class around `PlanetShape`, guards on
    the two probe commands, and TW-aware GameTests: about half a day. That buys a planet with **no** wrap, unless
    IP-CE seams replace it.
- **On the planet [R].**
  - The stored grid is 16384 × 8192 (`manifest.json`: x from −8192, z from −4096), already 2:1.
  - A four-portal seam set at x = ±8192 and z = ±4096 works on real terrain. The east–west crossing was seamless in
    flight, because Deep Time's own x-fold makes both sides identical.
  - **The long edges cannot be reached.** Deep Time's polar storm stops players at 80° (z ≈ ±3641,
    "Polar storm (80.0° S): the world ends at 80.0° S").
  - The long edges are the poles of today's Lambert map, so a half-turn there would not match the terrain anyway [I].
    A pillowcase needs the quincuncial projection of `world-projection-options.md` (Petroff or Gringorten in the
    Guyou frame).

---

## 6. Against MIC's pack: compatibility table

The server and client were copies of the pack's server and client sets (190 mods). Only one change was made to each copy
before its first boot: IP-CE 6.0.10 was added. Removals came afterwards, one crash at a time, and are logged in
`mic-server-bisect.log` and `mic-client-bisect.log`.

| Mod (version) | Result with IP-CE | Evidence |
|---|---|---|
| Petrolpark 1.21.1-1.5.11 | **Hard crash at mod construction.** Petrolpark's access transformer makes `ChunkMap.onChunkReadyToSend` public; IP-CE's `MixinChunkMap_C` has a private `@Overwrite` of it ("cannot reduce visibility of PUBLIC target method") | [R] FML crash report; [R] AT line found in Petrolpark's jar |
| Destroy 0.4.1, MIC Destroy Electric, MIC Petrochem | Removed because they require Petrolpark | [R] |
| CBC Modern Warfare 0.0.6v | **Hard crash at mod construction.** Its `camera.ChunkMapMixin` cannot inject into `ChunkMap.getPlayers`, which IP-CE's `MixinChunkMap_C` merges | [R] |
| Toroidal World 0.19.3 | Loads; **every join kicked** (`player_position` encode NPE) | [R] Twice: minimal set and full pack |
| Deep Time 0.0.1 | Needs TW; with the scratch switch it boots, generates and joins | [R] |
| Sable 2.0.5 (+ Create Aeronautics, Sable ragdoll) | Loads; IP-CE logs "Sable entity tracking compatibility is present". **Ships do not cross same-dimension seams.** **Four server crashes** ticking item entities beyond a seam (NPE `sable$collisionInfo` in Sable's `Entity.move` hook) | [R] Crash reports `mic-crash-sable*.txt`; [C] IP-CE's `MixinEntity_SableCollisionCompat` wraps the same `collide` call |
| C2ME 0.4.0-alpha.0.122 | Loads on server and client. IP-CE warns at start-up: "Chunk loading and portal creation may not work normally" (iPortalTeam issue 1036). No failure was seen in the short run | [R] |
| Sodium 0.8.13 + Sodium Extra | Client loads; IP-CE ships Sodium compat | [R] load; [U] rendering through a seam in the pack was not captured |
| Veil (bundled) | Loads; IP-CE ships Veil compat | [R] load |
| Simple Clouds 0.7.3 | Loads | [R] load; [U] clouds through a portal |
| Distant Horizons 3.3.1 | Loads | [R] load; see §7 |
| Create 6.0.10 | Loads; IP-CE warns "Sometimes some contraptions will be invisible" | [R] log; [U] contraptions and trains not tested |
| Cobblemon 1.8.1 | Loads. The known `#c:tools/brush` join race still happens; it needed one to three attempts | [R] Not caused by IP-CE |
| Project Atmosphere 0.9.1.2 | Loads | [R] |
| Simple Voice Chat 2.6.23 | Loads. On the game port its UDP bind fails, because another pack mod already binds UDP 127.0.0.1 on that port. The server then shuts down about 6 s after start. On game port + 1 it runs. IP-CE warns "Voice may stutter when looking at portal" | [R] |
| Embeddium, Iris, Oculus, Petrolpark-free Destroy | Not in the pack | — |

**IP-CE behaviour worth knowing [C][R].**
- **It contacts the internet at start-up.** It fetches a mod-compatibility list (`checkModInfoFromInternet`, on by
  default). That is where the "mildly incompatible" warnings above come from.
- **It shows a four-page welcome screen** on a client's first start.

---

## 7. Distant Horizons

- **What I saw [R].**
  - Spectating 1,200 blocks from the east seam for three minutes, DH had built no visible LODs. The frame was sky,
    except for a thin vertical slab of terrain where the seam is (`mic-DH-E.png`). That slab is unexplained; it may
    be IP-CE drawing the distant portal.
  - Every closer attempt ended in the Sable crash within seconds.
  - **The DH question is open.**
- **What can be said [I], from how DH and Deep Time work.**
  - **DH draws the raw world in raw coordinates.** It knows nothing of portals. Upstream reports say IP does not
    render DH terrain through portals (three merge requests to DH closed unmerged), so the far view *through* a seam
    ends at the vanilla render distance.
  - **Beyond the slide seams, the raw world is already right.** Deep Time's chunk generator and its own LOD generator
    (`compat/dh/DhLodGenerator` → `FarTerrain.build`) fold x per sample. So the LODs beyond x = ±HX are exactly the
    terrain the portal shows. Far east–west views would look correct in the same place, as long as DH's LODs and
    IP-CE's portal view do not fight in the band where both draw. **[U]** I did not see whether DH's pass is clipped
    by IP-CE's portal stencil or draws over it.
  - **Beyond the long edges, the raw world is wrong today.** The rasters clamp z, so DH would show a smeared edge row
    beyond z = ±HZ.
  - **Deep Time's LOD generator could fill that band cheaply.** Replace the z-clamp with the fold
    (x, z) → (−x, 2·HZ − z), x then wrapped, at the sampling layer. That is `FarTerrain` and `TerrainSource`: a few
    dozen lines.
    - LOD columns are heights and colours, so a 180° turn needs no per-block rotation.
    - The same fold in the chunk generator would make the hidden outside chunks match the far side, which also makes
      corner escapes harmless.
  - **The pole cap must go for any of this to matter.**
  - **Each seam crossing is a teleport to DH.** It jumps 16,384 blocks at a slide seam and to the mirrored point at a
    half-turn seam, so DH re-centres and reloads its LODs. TW's DH integration avoids that for slides; IP-CE does not.

---

## 8. What a player would live with

- **What works.** The player, everything they ride in a vanilla minecart or boat, dropped items, arrows, pushed mobs,
  fliers and builders crossing a seam by hand.
- **What does not.**
  - Fluids, redstone, pistons and all of Create stop at a seam. Ben accepted this.
  - Sable ships do not cross.
  - Mobs do not chase across.
  - Maps and waypoints read raw coordinates.
- **Visual flaws.**
  - Near the four turn points you meet yourself and the landscape twice.
  - Water ends at a straight edge in mid-air.
  - Entities that cross a half-turn seam into an unticked chunk hang 0.05 block outside the world until someone comes
    near.
  - Through a seam, the far side ends at the vanilla render distance. DH's LOD ring would not continue through it.
- **Escapes.** Exact hits on the two corner turn points leak into the hidden outside.
- **Performance.** About 10–15 % client frame time per portal in view, 30–40 % at a turn point or corner. No measurable
  server cost from the portals themselves.
- **Multiplayer.** Every client must run IP-CE. Only one player was tested.
- **In the pack.** Five mods must go, TW must go, and the Sable crash must be fixed before anyone could play near a seam.

---

## 9. What Deep Time would have to change

1. **Drop the TW requirement.** See §5: the mods.toml line, the `PlanetShape` mixin, two probe commands and the
   GameTests. Losing TW also loses everything TW does for Create, Sable, DH and C2ME across the seam. IP-CE replaces
   only the player-facing part.
2. **Change the projection.** It must become the quincuncial one (Guyou frame, Petroff or Gringorten) so that the long
   edges match under a half-turn. This is the rework already described for any quincuncial world in
   `world-projection-options.md` and the design note's §5.3.
3. **Remove the pole cap**, or move it.
4. **Fold the outside.** The chunk generator and `FarTerrain` should answer positions beyond each edge with the folded
   planet: x wraps, and (x, z) → (−x, 2·HZ − z) beyond a long edge.
5. **Install the seams at world creation.**
   - Four global portals, whose chunks must be loaded first.
   - Then a restart, or a patch to IP-CE's `addPortal` (§1).
   - Plus a wall against corner escapes.
6. **Pack-level work outside Deep Time.**
   - Petrolpark/Destroy and CBC Modern Warfare are out, or someone patches IP-CE's `ChunkMap` overwrite.
   - Fix or work around the Sable × IP-CE crash.
   - Accept no Sable crossings.

---

## 10. What I did not verify

- **Create trains and contraptions at a seam.** None was built.
- **The Sable crash's attribution to IP-CE.** I ran no control without IP-CE. The crashes did not recur once on a plain
  restart, and did recur on every later approach.
- **Distant Horizons at a seam**, in any form.
- **Rendering through a seam with Sodium, Veil and Simple Clouds in the pack.** The client loaded, but no seam view was
  captured before the crashes.
- **Multiplayer.** Two players were not tested.
- **Maps, waypoints and chunk loaders** were not tested.
- **The corner wall** was not tested.
- **The half-turn seam on Deep Time terrain** was not walked; the pole cap is in the way.
- **IP-CE's own claims.** Its README claims Sable portal handoff and rotated-portal tests; only the same-dimension
  exclusion was checked.
- **Frame-time numbers** come from software rendering on a loaded VM. They are only good for ratios.

---

## 11. Method and files

- **Test sets.**
  - Vanilla-plus: NeoForge 21.1.249, IP-CE 6.0.10 and Cloth Config, on a superflat world with coloured marker floors at
    the seam sites.
  - The Deep Time sets used the dh-video planet copy. The pack sets were copies of the pack's server and client sets.
- **Driving the client.** A real client under Xvfb (portablemc, offline profile), driven with xdotool. Positions were
  read over RCON every 0.2 s. Frames came from ffmpeg x11grab.
- **Scripts.** They stayed on the VM: `seams.py` (build, delete, list the seams), `walk.py`, `entities.py`, `fps.sh`,
  `modbisect.py`, `clientbisect.py`.
- **Review copies.** Screenshots, clips, crash reports and bisection logs are in `review/out/ip-proto/`, which is not
  committed. The clips:
  - `vp2-W1-slide.mp4`, `vp2-W2-halfturn.mp4`, `vp2-W3-near-turnpoint.mp4`, `vp4-W4-through-turnpoint.mp4`
  - `vp2-W5-corner.mp4` (the escape)
  - `vp2-S-entities.mp4`, `vp2-E-entities.mp4`
  - `dt-W1-E-fly.mp4` (the Deep Time planet)
- **Source read.** IP-CE at `v6.0.10`:
  - `PortalCommand`, `GlobalPortalStorage`, `Portal`, `ServerTeleportationManager`, `CollisionHelper`
  - `compat/sable/SableDimensionStackCompat`, `compat/mixin/sable/MixinEntity_SableCollisionCompat`,
    `mixin/common/collision/MixinEntity`
- **Deep Time.** Its TW coupling was read by a Sonnet sub-agent, and I re-checked `PlanetShape`,
  `WorldDimensionsShapeMixin`, the mods.toml template and `DeepTimeCommands`.
