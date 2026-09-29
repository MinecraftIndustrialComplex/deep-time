# Wrapped planet worlds on NeoForge 1.21.1: prior art, mechanisms, and a recommendation for Deep Time

Researched 2026-09-27 for Deep Time: a NeoForge 1.21.1 mod whose Rust core simulates a planet's history and plays it as a finite
world. Planned sizes: a circumference C of 16k blocks (singleplayer) or 32k (servers), pole to pole C/2, and about 300 m per block.

**Method.** I cloned the sources at the commits named below, read Modrinth API metadata, and used `gh api` for GitHub issues.
Bytecode was checked with `javap` against the pack's jars in `mods/mic-climate/smoke/server/mods/`, and I read the pack's current
`pack/mods/*.pw.toml`. A Sonnet sub-agent covered space mods, server plugins, CurseForge and game prior art; section 1.4 and
section 5.2 rely on its report and carry its sources. I launched no game and ran no build.

**Tags.** **[V]** means verified from the cited source. **[I]** means my inference, with the reasoning given. Every [V] names
its source inline. Clones sit in the session scratchpad (`r9/`) and are not kept.

---

## 0. Summary

- **[V] There is now a seamless wrap mod for exactly this stack: Toroidal World (TW).** It is MIT-licensed, runs on NeoForge and
  Fabric, and has had 1.21.1 builds since 2026-08-24; the newest is 0.19.3, released today. It does not use portals or
  teleports. It rewrites every position the server sends so that each client holds the copy of the world nearest to it, and it
  folds distances everywhere else. Its supported-mods list names Create, Sable, Create Aeronautics, C2ME, Sodium, Lithium,
  Distant Horizons and Tectonic. Its build pins the same versions MIC runs: Create 6.0.10, Sable 2.0.5, Aeronautics 1.3.2 and
  Sodium 0.8.13, with C2ME 0.4.0-alpha.0.120 (minimum .116). It is also very young: 840 downloads, 2 GitHub stars, one
  maintainer, and it is pre-1.0 with no world-compatibility promise between versions.
- **[V] Immersive Portals (IP) is effectively unmaintained on NeoForge 1.21.1, and it conflicts with MIC.** The last release is
  6.0.7 from 2025-06-18, and the Fabric upstream repo is archived. Users report crashes with Sodium 0.8.12 and 0.8.13 (MIC runs
  0.8.13) and a mixin conflict with Sable. Petrolpark, which MIC ships for Destroy, touches the `ChunkMap` method that IP
  overwrites. IP also crashes when combined with TW.
- **[V] The other options are teleport-at-border mods.** Looping World and Circumnavigate are Fabric-only. TFC Eratosthenes is
  tied to TerraFirmaCraft. Planet Border Teleport is a 3-class server mod. The Wrap Around World datapack also teleports. None of
  them handles physics ships. Space mods (Ad Astra, Northstar, Stellaris, Cosmic Horizons and others) keep planets as ordinary
  unbounded dimensions.
- **[V] Sable exposes a public teleport, `PhysicsPipeline.teleport(body, position, orientation)`.** Its Rapier implementation
  sets only the pose, so velocity carries over.
- **[V + I] Poles: a sphere-correct pole crossing cannot be seamless in a flat block grid.** Joining the north edge to itself
  shifted by C/2 is a *glide reflection*, which mirrors whatever passes through it. IP mirrors refuse to teleport, and TW's
  Sable code throws on a reflecting seam (both [V]). The alternatives are a 180° rotation, which joins the wrong longitudes, or
  sawtooth portal strips. Either way the pole needs caps, a non-seamless teleport, or a cube-sphere built from rotated portals.
- **Recommendation.**
  1. East–west: make TW a dependency and run a **cylinder** shape (X loops, Z does not), registered through TW's public
     `ShapeModule` API, with Deep Time's generator producing X-periodic terrain.
  2. Poles: **polar caps with a soft barrier** at about ±75–80°.
  3. Optionally later: a **non-seamless "pole transit"** teleport that shifts x by C/2 and turns the heading 180°. It works for
     players and Sable ships through public APIs.
  4. Keep a teleport-at-seam implementation as the fallback if TW fails the spike.
  5. Do not use Immersive Portals.

---

## 1. Existing approaches

### 1.1 Overview

Download counts and dates come from `https://api.modrinth.com/v2/project/<slug>` and `/version` on 2026-09-27.

| Project | Mechanism | Loaders / MC | NeoForge 1.21.1? | Licence | Downloads | Status |
|---|---|---|---|---|---|---|
| Toroidal World (`toroidal-world`) | Seamless: coordinate folding plus packet rewriting; torus or cylinder | NeoForge + Fabric; 1.21.1, 26.1.2, 26.2, 26.3 | **Yes**, 0.19.3+mc1.21.1 (2026-09-27); 12 builds for 1.21.1 since 2026-08-24 | MIT | 840 | Very active (770 commits since 2026-08-06); pre-1.0 |
| Immersive Portals for (Neo)Forge (`immersive-portals-neoforge`) | Seamless see-through global portals; wrap zone = 4 portals (torus) | Forge/NeoForge 1.18.2–1.21.1 | Yes, 6.0.7 (2025-06-18) | Apache-2.0 | 366,524 | No updates since 2025-06; authors "moved on" (see 2.1) |
| Immersive Portals (Fabric, `immersiveportals`) | same | Fabric ≤1.21.1 | n/a | Apache-2.0 | 1,788,631 | Repo archived |
| IPSable (fork, GitHub `r2smith141/IPSable`) | IP fork with Sable sub-level transit | NeoForge 1.21.1 | Yes, v0.5.0 prerelease (2026-08-05) | Apache-2.0 | 612 (latest release assets) | Experimental |
| Circumnavigate (`circumnavigate`) | Seamless: server/client coordinate translation, chunks tiled | **Fabric only**; 1.20.4, 1.21.1 | **No** | AGPL-3.0-only | 3,604 | Alpha; 0.7 (2026-05-21) |
| Looping World (`looping-world`) | Teleports the player at the vanilla world border | Fabric; 1.19.2–1.21.1 | No | MIT | 534 | One release (2026-01-24) |
| TFC Eratosthenes (`tfc-eratosthenes`) | Teleports the player (and vehicle) at the 180° meridian; pushes back at the poles; requires TFC | Forge 1.20.1, NeoForge 1.21.1 | Yes, 4.0.0 (2026-08-14) | GPL-3.0-or-later | 1,227 | Active |
| Planet – Border teleport (`planet-border-teleport`) | Server-side teleport of any non-passenger entity at configured X/Z borders | NeoForge 1.21.1 | Yes, 1.0.0 | LGPL-3.0-or-later | 143 | One release (2026-07-27) |
| Wrap Around World (`wrap-around-world`) | Datapack teleport | Datapack; 1.21.x | Listed 1.0 | MIT | 69 | One release (2026-06-23) |
| Immersive Portals Paperized | Server-side portal views (Paper) | Paper/Folia | n/a | Zlib | 1,110 | Alpha; the sub-agent found no wrap feature (entity pass-through is future work) |
| Chunky Border | A border only, no wrap | many | 1.2.18 (2024-06) | GPL-3.0 | 369,765 | For comparison only |

Sources: the Modrinth API for each slug. The per-mod sections below have more detail.

### 1.2 Toroidal World (details in section 6)

- **[V] Mechanism.** From `docs/how-it-works.md` on `anyttng/toroidal_world` (commit `3c1f3a30`, branch `mc/1.21`):
  - The loop width is stored in the dimension's chunk generator.
  - Vanilla noise is resized so that a whole number of cells fits into one lap.
  - Distance queries use the nearest copy.
  - An entity crossing the border is moved to the far side in the same tick, together with its vehicle, velocity and path.
  - Everything sent to a client is rewritten to the copy nearest that client: chunks, entity positions, block updates and
    waypoints.
- **[V] The client "believes the world is infinite."** Bounds reach the client separately, through `ToroidalWorldClientApi`
  (`docs/modding.md`).
- **[V] Shapes.** It offers a torus or a cylinder. A cylinder loops one axis and leaves the other to "the vanilla world border"
  (`docs/modding.md`, `LoopSpans`).
- **[V] Sizes.**
  - Presets run from 32 to 512 chunks.
  - Custom sizes go up to `WorldBorder.MAX_CENTER_COORDINATE`, per `core/WorldLoopSizes.java`.
  - The overworld must be at least 16 chunks wide.
  - The Nether width must equal the overworld width divided by the portal scale exactly (`docs/dedicated-server.md`).
  - C = 16k (1000 chunks) and C = 32k (2000 chunks) are both legal. [I]: both divide evenly by 8 for a 1:8 Nether.
- **[V] Every player needs the mod** (`docs/how-it-works.md`).
- **[V] Known issues** (GitHub issues):
  - #51 (2026-09-26): a Sable contraption vanished when crossing the seam on 0.18.1. The maintainer says it is fixed in the next
    update.
  - #42 (open): it crashes with Immersive Portals.
  - #43 (open): it does not work with Northbound.
  - #41: Distant Horizons 3.3 would not draw LODs beyond the seam. The maintainer says this is fixed in the next update.
  - #36: flat terrain and seam artefacts when combined with MTChunkGeneration, Tectonic and Streams Reflowing. The maintainer
    says this is fixed.
  - #20: entity collision bounce. The maintainer says this is fixed.

### 1.3 Circumnavigate

- **[V] Mechanism.** From the wiki `Principles.md` (`FamroFexl/Circumnavigate` wiki; repo at `5d662fb`):
  - Limits are per dimension and counted in chunks.
  - The client's coordinate space extends forever, while the server translates each position to the unwrapped copy closest to
    the client.
  - Client render distance is limited to the width of one tile.
  - This is the same core idea as TW, and it came first.
- **[V] It ships a curvature shader that needs Iris.** It supports vanilla clients. Server-side mods are "considered unsupported
  by default" (wiki `Compatibility.md`).
- **[V] It is Fabric-only, alpha, and AGPL-3.0.** The README says "No stable or expected functionality or future support is
  guaranteed." Its future work lists structure and feature generation across the seam, and map items.
- [I] It is of no direct use on NeoForge. Its AGPL licence matches Deep Time's, so its code could be studied or borrowed from.

### 1.4 Teleport mods and plugins

- **[V] TFC Eratosthenes** (`trygve55/tfc_eratosthenes` at `017f0e9`, `PlayerEventHandler.java`):
  - It checks every 23 ticks whether the player has passed the meridian. The half-circumference it tests against depends on
    latitude, because it uses Craster parabolic or Equal Earth projections.
  - For a mounted player it unrides all passengers, teleports the root vehicle and then each passenger, and re-mounts them
    (`passThe180MeridianForVehicle`, lines 208–215).
  - Poles are a pushback wall ("You have reached The North Pole", lines 36–48).
  - Only players, and whatever they ride, wrap.
- **[V] Planet – Border teleport.**
  - README: X and Z borders, ±326 by default.
  - Bytecode of `PlanetWorldEvents` (javap): it skips passengers (`isPassenger`), calls vanilla
    `Entity.teleportTo(ServerLevel,…,Set,yaw,pitch)`, and has configurable yaw offsets for each crossing direction
    (`YAW_NORTH_TO_SOUTH`, and so on).
  - It is torus-style: north maps to the south edge, not to x + C/2.
- **[V] Looping World** (`neuromuser/looping-world` at `bf208e1`): it loops over online players only and calls
  `player.teleport(...)` when they touch the vanilla world border (`LoopingWorld.java`, lines 33–79).
- **[V] Bukkit-era plugins (sub-agent).**
  - WorldWrapper (dev.bukkit.org/projects/worldwrapper, last file 2021) wraps only players, plus entities that are ridden or
    riding.
  - WorldWrap (2012) is dead.
  - The Forge port of WorldBorder advertises "wrap-around teleportation" (github.com/RoyCurtis/WorldBorder-Forge).
  - A 2023 SpigotMC thread found no modern wrap plugin
    (spigotmc.org/threads/are-there-any-world-wrap-or-wraparound-plugins-for-spigot.604352/).
  - "World Is Small" (curseforge.com/minecraft/mc-mods/world-is-small; Forge 1.20.1, 245 downloads) depends on Immersive
    Portals.
- **[V] Space mods (sub-agent).** Ad Astra (neoforge 1.21.1, 5.7M downloads, custom Terrarium licence), Northstar Redux (MIT,
  NeoForge 1.21.1), Stellaris (CC-BY-NC-SA, NeoForge 1.21.1) and Cosmic Horizons (Forge 1.20.1 only; renders planets as spheres
  from orbit): none documents finite or wrapping surfaces. **[I]** Nobody grepped their generators, but all of them add ordinary
  dimensions. Sources: `api.modrinth.com/v2/project/{ad-astra, northstar-redux, stellaris, cosmic-horizons-cosmos}` and
  cosmic-mod.github.io.

---

## 2. Immersive Portals in depth

### 2.1 Status on NeoForge 1.21.1

- **[V] Official build.** `immersive-portals-neoforge` 6.0.7 for NeoForge 1.21.1, published 2025-06-18 (Modrinth versions API).
  Repo `iPortalTeam/ImmersivePortalsModForNeo` at `aede93a` ("v6.0.7", 2025-06-18) has had no pushes since.
- **[V] Upstream is abandoned.** `gh api repos/qouteall/ImmersivePortalsMod` returns `archived: true`. On NeoForge PR #71, a
  commenter says the main authors of both the Fabric version and the NeoForge port "have both moved on to other things". Issue
  #73 (2026-07) asks for a successor.
- **[V] Community forks and patches.**
  - IPSable (r2smith141/IPSable) is an Apache-2.0 fork for Sable, Aeronautics and Create. v0.5.0 is a prerelease; the README
    lists "Performance (Sub Level stuttering)" and "DH + Voxy Support" as limitations. It ships a patched Rapier native that is
    **Windows x86_64 only**, per `build.gradle`'s `sable_rapier_x86_64_windows.dll`; without it you lose aperture contact
    clipping. It compiles against Sodium 0.8.12.
  - Immersive Portals Continued (Fabric, 2025-02).
  - Immersive Portals Compat Patches (jmcmahon1999/immptl-compat) patches only AE2 and Mekanism.
  - Immersive Portals × Sable Bridge (ARR). Its README says "Sable physics objects cannot go through portals" and that it is
    unsuitable for Sable 2.0.
  - Valkyrien Skies × IP compat 2.0.0 (1,049 downloads).

### 2.2 How the wrapping world works

- **[V] One command, four portals.**
  - `/portal global create_inward_wrapping <p1> <p2>` (and `create_outward_wrapping`) creates **four** `WorldWrappingPortal`
    global portals, one each for NORTH, SOUTH, WEST and EAST.
  - Each spans the full world height, from `McHelper.getMinY` to `getMaxYExclusive` (`WorldWrappingPortal.invokeAddWrappingZone`,
    lines 207–250). The online docs still say y 0–256, which is out of date.
  - A zone counts as valid only with all four portals (`WrappingZone.isValid`).
  - So the built-in command always makes a **torus**. [I] A cylinder means deleting the N/S pair or creating portals through
    the API.
- **[V] Global portals are always loaded and can be very large.**
- **[V] Chunk tracking.** The server tracks which chunks and entities each player can see through portals and loads them
  gradually, nearest first.
- **[V] Rendering.** The client holds several worlds and re-renders the world recursively for each visible portal using the
  stencil buffer. The docs call portal rendering expensive.
- **[V] Teleport timing.** The client teleports when the camera crosses, before a frame, and can teleport twice in one frame at
  wrapping corners.
- Sources for the last four points: qouteall.fun/immptl/wiki/Implementation-Details and qouteall.fun/immptl/wiki/Commands-Reference.

### 2.3 Compatibility with MIC's mods

- **Sodium.** [V] Issues #66 and #67 (0.8.12-alpha.2 / 0.8.13-alpha: crash on world creation or main menu) and #74 (0.8.12).
  Open PR #71 fixes the Sodium 0.8.x API change (`OcclusionCuller.Visitor` → `RenderSectionVisitor`) and is unmerged. MIC ships
  **Sodium 0.8.13** (`pack/mods/sodium.pw.toml`), so upstream IP is incompatible with MIC as shipped.
- **Sable / Aeronautics.**
  - [V] #64 reports a `@Redirect` conflict: IP's `MixinEntity.redirectHandleCollisions` is skipped because Sable's
    `EntityMixin.sable$collideRedirect` already redirects it. With both loaded, one user reports "broke all sable collision"
    plus network-protocol kicks.
  - [V] IPSable exists to fix this and is a prerelease.
- **Create.** [V] #38: moving parts of contraptions sometimes fail to render, and contraption collision disappears. [I] Create
  trains crossing an IP portal: I found no report either way. Create's own portal support is a separate mechanism (2.4).
- **C2ME.**
  - [V] #52: IP overwrites `ChunkMap#onChunkReadyToSend` to abort early, which breaks every mod that mixes into that method. A
    commenter adds "petrolpark's library" to the affected list.
  - [V] The MIC jar `petrolpark-1.21.1-1.5.6.jar` has an access-transformer line making `ChunkMap.onChunkReadyToSend` public.
    Whether petrolpark also injects there, I did not check.
  - [I] C2ME replaces the chunk system that IP rewrites for tracking. No issue confirms either way, so treat it as high risk.
- **Distant Horizons.**
  - [V] PR #71 also claims DH fixes and is unmerged. A commenter says DH merged IP support into DH's main branch (merge request
    93); I did not verify that.
  - [V] IPSable lists DH support as missing.
- **Toroidal World.** [V] TW #42: a crash on entering a toroidal world with IP installed.
- **Valkyrien Skies** is not in MIC. [V] A third-party bridge exists.

### 2.4 Can IP build a sphere-correct pole portal?

- **[V] Rotation is supported, reflection-teleport is not.** IP supports rotation transforms (`set_portal_rotation_along`,
  `rotate_portal_rotation`) and huge global portals. `Mirror` is a separate `Portal` subclass whose `canTeleportEntity` returns
  `false` and which calls `setTeleportable(false)` every tick (`portal/Mirror.java`, lines 12–35). The docs say: "Mirrors
  cannot be entered, because the player would have to be mirrored."
- **[I, geometry] A rigid portal covering the whole north edge with a 180° yaw rotation maps (x, z) → (2x₀ − x, 2z_N − z).**
  - That gives the right destination longitude (x + C/2) only at x = x₀ ± C/4.
  - At x₀ and x₀ + C/2 it sends you back to where you came from.
  - Everywhere else you arrive at the wrong longitude.
- **[I] The correct map is a glide reflection.** (x, z) → (x + C/2, 2z_N − z) has determinant −1, and IP cannot teleport
  through a reflection.
- **[I] Tiling the edge with many short portals, each rotated 180°, gives a sawtooth.** Longitude runs backwards inside each
  portal and jumps between portals.
- **Answer: no.** See section 5 for why no mod can.

---

## 3. Teleporting at the border (x = C → x = 0)

### 3.1 What breaks with a naive teleport

- **Vehicles and passengers.**
  - [V] TFC Eratosthenes has to unride, teleport each entity, then re-mount (1.4).
  - [V] WorldWrapper only moves ridden entities (1.4).
  - [I] Every other entity type must be scanned each tick as well. PBT does this for all non-passenger entities.
- **Projectiles.** [I] Anything in flight must be moved with its velocity preserved. Otherwise it flies out past the border.
  Examples: CBC shells, TaCZ bullets, arrows.
- **Leashed mobs.** [I] Vanilla drops a leash once the holder is more than about 10 blocks away, and after a teleport the holder
  is C blocks away. The leashed pair must move together.
- **Create contraptions and trains.**
  - [I] Contraption entities can be teleported as entities, but a contraption straddling the seam cannot exist.
  - [I] Trains run on the track graph, not as free entities.
  - [V] Create 6.0.10 has `com.simibubi.create.api.contraption.train.PortalTrackProvider` (javap on
    `create-1.21.1-6.0.10.jar`). It is a registry from `Block` to `findExit(ServerLevel, BlockFace) → Exit`, used for Nether
    portal tracks.
  - [I] A "seam portal" block at x = C−1 paired with one at x = 0 could reuse it. Whether a same-dimension exit is accepted
    needs a test.
- **Aeronautics / Sable ships.** See 3.2. [I] Players standing on the deck, the client's copy of the sub-level, and physics
  chunk tickets at the destination all have to move in the same tick.
- **Redstone, fluids, belts, pipes, kinetic networks.** [I] Blocks at x = C−1 and x = 0 are not neighbours, so no network
  crosses the seam. Chunk loaders on one side do not load the other.
- **Multiplayer sync.** [I] A player at x = C−5 and another at x = 5 are about C apart for tracking, sound, voice chat and mob
  targeting, so they cannot see or hear each other.
- **Seam hitch.** [I] The client drops its chunks and asks for new ones at the destination, which shows as pop-in. A fast jet
  makes this worse.
- **Maps.** [I] Vanilla maps and Antique Atlas simply end at the border. That is acceptable, but a map will not wrap.

### 3.2 Moving a Sable sub-level instantly

- **[V] The API** (Sable repo `ryanhcode/sable` at `6966d29`, tag 2.0.5; also javap on the MIC jar `sable-neoforge-1.21.1-2.0.5.jar`):
  - `PhysicsPipeline.teleport(PhysicsPipelineBody body, Vector3dc position, Quaterniondc orientation)` is public API
    (`common/.../api/physics/PhysicsPipeline.java`, lines 143–149).
  - The wrapper `RigidBodyHandle.teleport(position, orientation)` sits at `RigidBodyHandle.java:196`.
  - The command `/sable teleport <targets> <destination>` is in `SableSubLevelCommands.java`, lines 58–171.
- **[V] Velocity is preserved.** The Rapier implementation (`RapierPhysicsPipeline.java`, lines 433–441) calls the native
  `teleportObject`, then writes `logicalPose`. The Rust side (`sable_rapier/src/main/rust/rapier/src/lib.rs`, lines 1161–1186)
  calls only `rb.set_position(pose, true)`, which in Rapier leaves the body's velocity unchanged.
- **[V] Velocity can be changed.** `getLinearVelocity`, `getAngularVelocity` and `addLinearAndAngularVelocity` are public
  (`RigidBodyHandle.java`, lines 144–186), so a pole transit can rotate the velocity 180°.
- **[V] TW already does this at the seam.** It wraps `SubLevelPhysicsSystem.updatePose`, re-seats the read-back pose, and calls
  `pipeline.teleport(body, target, pose.orientation())` (`compat/sable/SablePoseFold.java`, line 84, and
  `mixin/SubLevelPhysicsSystemMixin.java`). Constraint-connected bodies move as one group (`SableConstraintGraph`).
- **[V] Sable stores sub-level blocks in a plot grid near the world edge.** `SubLevelContainer.DEFAULT_ORIGIN = 10000` plots,
  about 30M blocks. It patches `WorldBorder.isWithinBounds` so plot chunks count as inside
  (`mixin/world_border/WorldBorderMixin.java`). **[I]** Any wrap or fold code must exempt the plot grid; TW's `SableChunkKeys`
  appears to do this. **[I]** Rapier bodies collide with terrain colliders, not with the world border, so a vanilla border will
  not stop a ship. A teleport mod has to catch ships itself.
- **Licence.** [V] Sable is PolyForm Shield 1.0.0 (`LICENSE.md`), which is source-available. Calling its API is fine.
  [I] Redistributing a modified Sable, or patched natives as IPSable does, is not.

### 3.3 Hiding the seam in a teleport design

- **Real "ghost" chunks.** [I] With an X-periodic generator and a border at C, the vanilla server still loads and generates chunks
  past the border within view distance. The ghost therefore draws correctly with vanilla and Sodium. The costs:
  - These are real, separate chunks: extra disk, extra generation, mob spawns.
  - They carry none of the players' builds.
  - The border blocks interaction, but explosions and fluids can still change them.
- **Distant Horizons.**
  - [V] DH's API has `IDhApiWorldGenerator`, which returns `VANILLA_CHUNKS`, `API_CHUNKS` or `API_DATA_SOURCES` through
    `generateLod(...)`, and `IDhApiWorldGeneratorOverrideRegister.registerWorldGeneratorOverride(IDhApiLevelWrapper, generator)`.
    Checked with javap on `DistantHorizons-3.2.0-b-1.21.1-fabric-neoforge.jar`.
  - [I] Deep Time could feed LODs straight from its rasters (and periodic beyond C). DH builds LODs from real chunks once they
    exist, so builds near x ≈ 0 would never appear in the LODs beyond C.
- **[I] Only a TW or Circumnavigate style fold makes builds visible across the seam.**

---

## 4. Rendering across the seam without portals

- **[I] A periodic generator alone, with no wrap mod.** Chunks beyond C are real and independent. Vanilla, Sodium and DH draw
  them correctly because the terrain matches. Structures, trees and features match only if their placement is also periodic.
  Player builds are never mirrored. This is 3.3's ghost-chunk design.
- **[V] With TW.**
  - The server never stores chunks outside the canonical span.
  - A chunk "on the far side of the border arrives at the coordinates right next to you" (`how-it-works.md`), so builds show on
    both sides.
  - DH is listed as "Supported", and the store description says the far side shows "out to half the world width".
  - Folding DH is done by mixins into DH internals: `compat/distanthorizons/mixin/*`, including `FullDataSourceV2RepoMixin`,
    `LodQuadTreeMixin` and `WorldGeneratorInjectorMixin`.
  - TW compiles against DH 3.3.1. MIC ships DH 3.3.0 (`pack/mods/distanthorizons.pw.toml`); the smoke copy has 3.2.0-b.
    [I] Internal mixins make the exact DH version matter.
- **[V] TW and foreign generators.**
  - TW rebuilds only generators whose class is exactly `NoiseBasedChunkGenerator` or `FlatLevelSource`
    (`engine/gen/ShapedDimensions.isRebuildable`, line 244). It swaps those for its `LoopedChunkGenerator`, which also stamps its
    fold into the structure state.
  - Any other generator is "stamped" instead (`ChunkGeneratorMixin` / `ShapeStamp`): it gets the shape, folding and crossings,
    but TW does not touch its noise ("Only the terrain does not repeat", `how-it-works.md`).
  - [I] Deep Time's `NoiseBasedChunkGenerator` subclass would be stamped. Its raster periodicity supplies the repeat, so that
    combination is correct.
  - [I] Open questions for the spike:
    - Structures and features straddling the seam: TW only installs the structure-state transformer for its own generator.
    - Whether TW's `TerrainCeiling` and floating-crumb sweep leave Deep Time's settings alone. They key on vanilla density
      functions, per `TerrainCeiling.LADDER_BASE_BLOCKS` / `VANILLA_NAMESPACE` and `GenerationProbe.sweepCrumbs`'s javadoc.
    - Whether TW's thread-bound fold (its `NoiseBasedChunkGeneratorMixin` wraps `doFill`, `doCreateBiomes` and
      `iterateNoiseColumn`) changes what Deep Time's density functions read.
- **[I] An alternative integration.** If Deep Time instead used a vanilla `NoiseBasedChunkGenerator` with custom raster density
  functions (option (c) in `minecraft_integration_architecture.md`), TW would rebuild it into `LoopedChunkGenerator`. That
  includes the structure fold, but it also brings TW's noise-grid rewriting.

---

## 5. Poles

### 5.1 Why a seamless sphere-correct pole crossing cannot exist in a flat grid [I, standard topology]

- **The chart.** An equirectangular block world is a cylinder, [0, C) × [z_S, z_N]. On a real sphere each polar edge collapses to
  a single point. In Minecraft the polar row is still C blocks long, so the edge has to be glued to something.
- **Option 1, antipodal gluing: (x, z_N+d) ≡ (x + C/2, z_N − d).**
  - This is correct for navigation: fly north, arrive at longitude +180°, now heading south.
  - It is a **glide reflection** (determinant −1). A band glued this way is a Möbius band (cross-cap), and with both poles
    treated the same way the whole surface is a Klein bottle.
  - Anything crossing it comes out mirrored.
  - [V] IP refuses to teleport through mirrors (2.4). TW's API reserves `preservesHandedness()` and `decomposesPerAxis()` for
    such folds but "the engine refuses to build a fold that answers false" (`docs/modding.md`). Its Sable compat throws "a
    reflection is not a rigid-body rotation" (`compat/sable/SableRigidShift.java`).
- **Option 2, reflection gluing: (x, z_N+d) ≡ (2x₀ − x, z_N − d).**
  - This is a 180° rotation, orientation-preserving and rigid, and topologically it caps the cylinder into a real sphere.
  - It joins the *wrong* longitudes, except at two meridians (2.4).
- **Option 3, torus: north edge joined to the south edge.**
  - [V] Supported natively by TW (torus preset) and IP (wrap zone).
  - It is geographically wrong: you cross the north pole into the southern polar region.
  - The simulation's rasters are not periodic in latitude, so the generator would need a blend band.
- **Option 4, non-seamless pole transit.** Teleport to (x + C/2, mirrored z) and rotate yaw and velocity 180°.
  - This is rigid, so it works for players and Sable ships (3.2).
  - Only the continuous view across the pole is lost.
  - [V] This is the RPG-Maker and GameDev technique the sub-agent found (forums.rpgmakerweb.com/threads/globe-shaped-world-map.56500/;
    gamedev.net/forums/topic/634996-the-intriguing-problem-with-map-wrapping/).
- **Option 5, cube-sphere.**
  - Six square faces, each C/4 on a side (8,000 blocks at C = 32k).
  - Every edge join is a 90° or 180° rotation, so every seam is rigid and orientation-preserving.
  - [I] It is feasible only with portal-style seams. That means IP, which conflicts with MIC, or a custom rotated fold, which TW
    does not support. Each of the 8 corners has a 90° angle deficit where three faces meet.

### 5.2 How games handle it (sub-agent, with its sources)

- **Cylinder with impassable polar ice caps.** Civilization V and VI (civilization.fandom.com/wiki/Map_(Civ6)); EU4 loops
  east–west with no polar caps (eu4.paradoxwikis.com/Map).
- **Torus option.** Civilization IV (forums.civfanatics.com/threads/world-wrap.280304/).
- **Cylinders.** Starbound (en.wikipedia.org/wiki/Starbound).
- **No wrap.** Dwarf Fortress (dwarffortresswiki.org, Advanced world generation) and Veloren (book.veloren.net/players/faq.html).
- **Cube-sphere or quad-sphere.**
  - Kerbal Space Program's PQS (wiki.kerbalspaceprogram.com/wiki/Tutorial:Making_Planets).
  - Space Engineers' six-face planet maps (spaceengineers.wiki.gg/wiki/Modding/Tutorials/Creating_a_Planet).
  - Seed of Andromeda's voxel "inflated cube" (voxel.wiki/wiki/round-planets/).
- **Goldberg / hex-sphere Civ-likes.** These remain prototypes (en.wikipedia.org/wiki/Goldberg_polyhedron).
- **Equal-area projections.** Civ map-makers use them to fight polar stretch
  (forums.civfanatics.com/threads/a-new-earth-map-making-challenge.657636/). TFC Eratosthenes uses Craster parabolic and Equal
  Earth with a latitude-dependent wrap width (1.4).

### 5.3 Projection distortion [I, arithmetic]

- **East–west stretch.** In equirectangular, a block spans 300·cos φ m east–west and 300 m north–south, so features stretch by
  sec φ:

  | Latitude | 60° | 70° | 75° | 80° | 85° |
  |---|---|---|---|---|---|
  | sec φ | 2.0 | 2.9 | 3.9 | 5.8 | 11.5 |

  At C = 32k, one block of z is 180°/16,000 = 0.01125° of latitude. Caps at ±75° remove about 1,333 blocks at each end.
- **Mitigations.**
  - Caps: Civ style.
  - Latitude-dependent wrap width: TFC Eratosthenes. [I] It is incompatible with a chunk-aligned constant span such as TW's,
    because the seam becomes ragged.
  - Cube-sphere: the gnomonic cube has a centre-to-corner area ratio of 3√3 ≈ 5.2, and the equiangular cube about 1.30. Both
    come from the area elements (1+x²+y²)^(−3/2) and (1+t₁²)(1+t₂²)/(1+t₁²+t₂²)^(3/2) evaluated at the corner.
  - Sampling the simulation at the true latitude and longitude of each column. This keeps geology consistent and only stretches
    east–west near the poles.

---

## 6. Recommendation for MIC

### 6.1 Approach

1. **East–west: Toroidal World, cylinder shape, X looping.**
   - Deep Time registers its own shape through the public API:
     `ShapeModule.of(deep_time:planet, …, ShapeDimensions.withSpans(dimensions, LoopSpans.ofWidth(Axis.X, C/16), …))`.
     This is `docs/modding.md` → "Declaring a world shape".
   - For C = 32k the spans are −1000 … 1000 chunks.
   - Deep Time's rasters wrap in longitude so that chunk −1000 continues seamlessly from chunk 999.
   - Deep Time keeps its own generator, which TW stamps (4).
   - Depend on `com.toroidalworld:toroidal-world-api` `compileOnly`, and declare TW as an optional or required dependency.
   - [V] MIT is compatible with AGPL-3.0 use.
2. **Poles: caps.** Put a soft barrier at about ±75–80°: rising turbulence and headwind for Sable ships and a pushback for
   entities, in the style of Eratosthenes. Z is not looped. [I] TW's cylinder leaves Z unbounded, so Deep Time owns the Z limit
   and the generator outside it (ice shelf or void).
3. **Optional later: a pole transit** (5.1, option 4), as a clearly marked jump rather than a seam.
4. **Fallback if TW fails the spike: Deep Time's own teleport seam.**
   - Scan all entities every tick and move vehicles together with their passengers.
   - Move Sable bodies with `PhysicsPipeline.teleport`.
   - Use real ghost chunks from the periodic generator and DH LODs from rasters beyond C.
   - Accept that nothing crosses the seam.
   - [I] Deep Time could instead fork TW under MIT if TW is abandoned.
5. **Do not use Immersive Portals** (2.1–2.3).

### 6.2 Risks

- **TW is young.** It is seven weeks old, has one maintainer and 2 stars, and is pre-1.0 with no promise that worlds survive
  upgrades. It is also very intrusive: 1,315 Java files and dozens of mixins into the internals of Create, Sable, Aeronautics,
  C2ME, DH and Sodium. Every MIC update to those mods can break it. [V] for the counts and versions; [I] for the fragility.
- **Version drift today.** [V] TW builds against:
  - DH 3.3.1, while MIC ships 3.3.0.
  - C2ME .120 (minimum .116), while MIC ships .122.
  - NeoForge 21.1.248 as its minimum, while MIC ships 21.1.249.

  [I] The minimums pass, but the internal mixins are exact-version sensitive.
- **Ships crossing the seam were broken until very recently.** #51 was fixed "in next update", and 0.19.x is the first release
  that could contain the fix. Treat ship crossing as unverified.
- **Mods TW does not list** (untested [I]):
  - Simple Voice Chat: its own UDP positions are not rewritten, so positional audio probably breaks near the seam.
  - Antique Atlas 4 and Surveyor: the client-side unwrapped coordinates may record duplicate terrain past C.
  - Create Big Cannons and TaCZ projectiles; Create Radar; cc_sable / CC: Tweaked (world positions inside custom payloads);
    Chunky; Serene Seasons and Project Atmosphere (per-position climate); Petrolpark and Destroy.
  - TW's `PacketRewriters` and `positions.json` exist for exactly these gaps, but each needs a patch.
- **Every client needs TW, and every player's client runs TW's coordinate rewriting.** [V] `how-it-works.md`.
- **Poles** (section 5). Players may expect to fly over the pole; the caps must be explained in game.
- **Performance at 2,000 chunks of width.** [I] The loop itself costs nothing, but the per-packet rewriting is new code on the
  hot network path.

### 6.3 First technical spike

Use a Prism instance with MIC 2.1.3 plus TW 0.19.3 (NeoForge 21.1.249) and no game-content changes.

1. **Boot.** Create a cylinder world with a custom preset of 2,000 chunks, then one of 1,000. Check the log for mixin
   failures; C2ME .122, DH 3.3.0, Sodium 0.8.13 and petrolpark are the likely suspects.
2. **Stub Deep Time generator.** Write a `NoiseBasedChunkGenerator` subclass with an X-periodic test raster, plus a
   `ShapeModule`. Confirm that:
   - TW stamps it rather than rebuilding it;
   - the terrain is continuous at chunk −1000 / 999;
   - there is no terrain ceiling, crumb sweep or climate compression;
   - structures and trees that straddle the seam are intact.
3. **Aeronautics jet at full speed** across the seam, repeated about 20 times. Check:
   - hull continuity, passengers staying on deck, propellers and ropes and constraints;
   - the client hitch;
   - destination chunk tickets;
   - save and quit while the ship straddles the seam, then reload.
4. **Create.** Run a train across the seam. Test a bearing or elevator contraption straddling it, and belts and shafts that span
   it.
5. **DH.** Confirm LODs past the seam at 32k (the #41 fix) and check the far side for player builds.
6. **Multiplayer.** Put two players on opposite sides of the seam. Check entity tracking and sound, and test Simple Voice Chat
   and Antique Atlas behaviour.
7. **Chunky.** Pre-generate the canonical span and check that nothing is stored outside it.
8. **Pole barrier prototype.** Push a Sable ship back through `RigidBodyHandle.addLinearAndAngularVelocity`, and prototype a
   pole transit with `teleport` plus a 180° rotation of the velocity.
9. **Fallback measurements** (only if TW fails): time a plain `PhysicsPipeline.teleport` seam hop for a jet, and measure the
   client pop-in.

---

## 7. What crossing the seam does, read from TW 0.19.3's code (added 2026-09-28)

Added 2026-09-28 because the seams review page did not say where the wrap is or what a player experiences when crossing it.
Sections 1.2 and 4 relied on TW's docs; this section checks each claim against the shipped code.

**Method.** I decompiled the inner jar `libs/ref/twx/META-INF/jarjar/toroidal_world-neoforge-0.19.3+mc1.21.1.jar` (sha256
`e4537c2de1d05d54…`) with CFR 0.152 and read its mixin configs (`toroidal_world.mixins.json`, 289 core mixins plus 11 client
mixins). I re-read `docs/how-it-works.md`, `docs/modding.md` and `DESCRIPTION.md` on branch `mc/1.21` as of 2026-09-28. Vanilla
behaviour comes from the NeoForge 21.1.249 sources (`libs/neoforge-src/`). Class names below are TW's
(`com.toroidalworld.…`) unless marked vanilla or Deep Time. No client was run, so everything here is what the code does, not
something anyone has watched. Numbers are for C = 16,384; at 32k, double them.

### 7.1 Where the seam is

- **[V] It is one north–south line at x = −C/2 ≡ +C/2: x = −8,192 ≡ 8,192 at 16k, and ±16,384 at 32k.**
  - `LoopSpans.ofWidth(X, C/16)` (Deep Time `wrap/PlanetShape.java`) builds `WorldLoopBounds.Looped.ofWidth(w)` with
    `minChunk = −w/2` and `maxChunk = w/2`. That is chunks −512 … 511 and blocks [−8,192, 8,192).
  - `Looped.isOver(x)` is `x < minBlock || x ≥ maxBlock`, so block 8,191 is the last stored column and block −8,192 the first.
  - The line runs the full z length: Z is not looped (`LoopSpans.NONE` on Z), and the pole cap at ±80° is Deep Time's.
- **[V] Nothing else sits on the seam.**
  - Deep Time sets no world border (no border code in `neoforge/src/main/java`), so the vanilla border applies: centre 0,0,
    diameter 59,999,968.
  - TW's `WorldBorderMixin` measures the border through the seam (`isWithinBounds`, `getDistanceToBorder`, `clampToBounds`,
    `getCollisionShape`), and a border wider than the loop never blocks x.
- **[V] The seam is not placed away from spawn on purpose.** x = 0 is half a lap from the seam (longitude 180° in Deep Time's
  λ = 360°·(x + C/2)/C). Spawn, however, is the best-ranked candidate in `spawn/candidates.json`. The core's ranking measures
  east–west distance with `wrapped_dx` (`crates/deeptime-core/src/deposits/spawn.rs`) and does not avoid the seam
  (`server/SpawnHandler.java`). [I] A spawn near the seam is possible.

### 7.2 Two coordinate systems: the server's and each client's

- **[V] The server keeps one copy, x ∈ [−8,192, 8,192).** Every server-side position is folded back into it, where "fold"
  means adding or subtracting C until the coordinate is inside.
- **[V] Each client keeps its own unfolded x, which is never folded while you stay in one dimension.**
  - The server records this per connection as a "mirror": `engine/seam/ClientPosition` (`record Mirror(x, z, space, …)`), held
    by `ServerGamePacketListenerImplMixin`.
  - Walking east from 8,190 across the seam, the client's own x goes 8,191, 8,192, 8,193 … and keeps growing. After one full
    lap east it is about 16,384 + where you started, and it keeps growing lap after lap.
  - modding.md: "A client level's own engine believes the world is infinite" and "Client coordinates near the seam run whole
    world widths away from the server's".
- **[V] The mirror is re-based to the server's folded position at exactly three moments.** "Re-based" means reset to the
  folded coordinate: the client's x jumps back into [−8,192, 8,192).
  - Login: `ServerGamePacketListenerImplMixin.toroidal$seedMirror`, at the TAIL of the listener's constructor.
  - Respawn: `PlayerListMixin.toroidal$rebaseMirrorOnRespawn`.
  - Dimension change: `PacketTranslator.playerPosition`. When the mirror does not describe the current dimension, it calls
    `rebase` and sends the position packet unchanged.

  All three happen behind a loading screen anyway. Nothing re-bases a client that has walked many laps.
- **[V + I] The limit on laps is far away.**
  - [V] Vanilla's `ServerGamePacketListenerImpl.clampHorizontal` clamps a reported x to ±3.0E7.
  - [I] Reaching that needs about 1,830 laps at 16k (916 at 32k) without a relog, death or portal, so it is not a practical
    limit.
- **[V] F3 shows the folded coordinates, plus the unfolded ones on extra lines.**
  - `mixin/DebugEntryPositionMixin`, a `@ModifyReturnValue` on `DebugScreenOverlay.getGameInformation`, rewrites the "XYZ:",
    "Block:" and "Chunk:" lines with `transformer.fold(…)`. The Chunk line's region file becomes the stored one.
  - It inserts "Unwrapped XYZ", "Unwrapped Block" and "Unwrapped Chunk" lines holding the client's raw coordinates.
  - Crossing east at the seam, F3's XYZ therefore jumps from 8,191.9 to −8,191.9, while "Unwrapped XYZ" reads 8,192.1.
  - The right-hand "Targeted Block"/"Targeted Fluid" lines (`getSystemInformation`) are not hooked. [I] They show the client's
    raw coordinates.

### 7.3 A player walking across

- **[V] The client sends its unfolded position. The server moves the player back into the span and tells the client nothing.**
  - `ServerGamePacketListenerImplMixin.toroidal$continuousX`/`Z` (a `@WrapOperation` on `clampHorizontal` in
    `handleMovePlayer`) records the reported x into the mirror (`MirrorWriter.PLAYER_MOVE`).
  - It then hands vanilla the copy of that x nearest the player's current server x (`WrapDomain.unwrapAround`), and does the
    same for `firstGoodX`/`lastGoodX`. Vanilla's "moved too quickly" and "moved wrongly" checks therefore see an ordinary
    small step.
  - `toroidal$wrapIntoBounds` at `handleMovePlayer` RETURN: if the new server position is past the edge, `SeamSnap.withPassengers`
    moves the player (and any passengers) by ±C and resets the good positions. It then calls `ServerChunkCache.move(player)`.
  - No `ClientboundPlayerPositionPacket` is sent, so there is no teleport and no loading screen. The client never learns it
    was moved.
- **[V] Chunks do not reload at the crossing.**
  - The server's chunk view centre jumps from chunk 511 to −512. `ChunkTrackingViewMixin.difference` walks both views in folded
    chunk space and reports only the chunks that really enter or leave, the same one-column strip as any other step.
  - `ChunkTrackingViewPositionedMixin` makes `contains`/`forEach` wrap.
  - Every chunk packet is placed at the copy nearest the client's mirror:
    - `PacketTranslator.chunkPosition` for chunk data and light;
    - `chunkCacheCenter` for the view centre;
    - `forgetChunk` for unloads, via `TranslationContext.toClient(ChunkPos)`, which is `transformer.nearestCopy(anchor, pos)`.
  - Stored chunk −512 therefore arrives at client chunk 512, beside 511.
- **[V] A client is sent one copy of each chunk, never two.**
  - modding.md: "A client near a seam is sent one copy of a chunk and not the others".
  - TW caps view and simulation distance at `chunkWidth/2 − 3` (`WorldLoopBounds.Looped.maxViewDistance`, applied in
    `PlayerListMixin`, `ChunkMapMixin`, `TranslationContext`). That is 509 chunks at 16k and 1,021 at 32k.
  - [I] Vanilla's 32-chunk maximum is far below that cap, so it never binds, and you can never see the same place twice or
    your own back.
- **[V] Teleports (`/tp`, and anything else that goes through `ServerGamePacketListenerImpl.teleport`) land on the copy
  nearest the client.**
  - `ServerGamePacketListenerImplMixin.toroidal$wrapTeleportX`/`Z` folds the destination on the server.
  - `PacketTranslator.playerPosition` sends the client `ClientPosition.destinationOf(…)`: an absolute destination is unwrapped
    around the mirror, and a relative one is mirror + folded delta.
  - Before and after the teleport, `toroidal$dropChunksBeforeTeleport`/`toroidal$resendChunksAfterTeleport` resend only the
    chunks whose nearest copy flips.

### 7.4 Everything else that crosses

- **[V] Non-player entities (mobs, items, arrows, snowballs, minecarts, boats) are folded at the end of their own tick,
  however they got past the edge.**
  - The fold is `ServerLevelMixin.toroidal$wrapEntityIntoBounds`, at `ServerLevel.tickNonPassenger` TAIL, through
    `SeamSnap.withPassengers`.
  - It sets the position, shifts `xo`/`xOld` (so the next frame interpolates smoothly), shifts a mob's navigation, move and
    look targets by the same ±C, and recurses into passengers.
  - An entity that joins the level past the edge is folded on `PersistentEntitySectionManager.addEntity`
    (`EntitySectionManagerMixin`).
  - This contradicts, or at least narrows, STATUS.md's P0-06 note that "`setPos` is a teleport and stays unfolded". [I] That
    holds only until the entity's next tick. The test probably read the position before then.
- **[V] What the players watching it see.**
  - The fold moves the entity about 16,384 blocks on the server, which is more than a relative-move packet can carry.
  - Vanilla's `ServerEntity.sendChanges` therefore sends an absolute `ClientboundTeleportEntityPacket` (delta outside
    ±32,767 in 1/4096 units).
  - TW's `PacketTranslator.teleportEntity` rewrites that packet to the copy nearest each watcher, so on each client the entity
    lands where it was heading. `addEntity`, `moveVehicle`, sounds, particles, block events and explosions go through the same
    translator.
- **[V] Vehicles and riders.**
  - A player driving a boat or horse: `handleMoveVehicle` is wrapped the same way (`toroidal$vehicleContinuousX`/`Z`,
    `toroidal$wrapVehicleIntoBounds`), and the vehicle is folded with all its passengers.
  - A player riding something they do not steer, such as a minecart: the vehicle is folded in its own tick, and the rider's
    mirror is re-seated next to its old value (`MirrorWriter.PASSENGER`).
- **[V] Sable ships are folded by the centroid of their constraint group.**
  - `compat/sable/SablePoseFold.reseat` runs from `SubLevelPhysicsSystemMixin` on `updatePose`. When the group's centre is
    past the edge, every body in it is moved with `PhysicsPipeline.teleport`.
  - Any velocity the teleport dropped is added back, `lastNetworkedPose` is shifted, and the collision sections are
    re-uploaded.
  - A ship straddling the seam therefore stays stored on the side its centre is on.
  - Client snapshots are re-seated by `ClientboundSableSnapshotDualPacketMixin`/`ClientboundStartTrackingSubLevelPacketMixin`.
  - [V] Deep Time's `jetCrossesSeam` passed 20 of 20 on the planet server (STATUS.md P0-06). [unverified] How the hull looks on
    a client at the crossing.
- **[V] Two players on opposite sides.** Take A at server x = 8,100 and B at −8,100, 184 blocks apart through the seam.
  - Entity tracking measures that distance through the seam (`TrackedEntityMixin.updatePlayer` wraps the `subtract`).
  - A's client gets B at the copy of −8,100 nearest A's mirror. If A's mirror is 8,100, that is 8,284.
  - B's client gets A at −8,284, if B's mirror is −8,100.
  - Each sees the other 184 blocks away, at different client coordinates. Sounds use `PlayerListMixin.broadcast`, which sends
    each listener the nearest copy.

### 7.5 Coordinates in commands, maps, compasses and spawn points

- **[V] Commands use server (folded) coordinates.**
  - `CommandCoordinatesMixin` refuses an absolute x or z outside the span with "%s is outside this world — it must be between
    %s and %s" (`SeamCommandErrors`, `argument.toroidal_world.pos.outside_world`). At 16k the span is −8192 to 8191.
  - Relative `~` coordinates are allowed past the edge and folded (7.3).
  - [I] The numbers on F3's XYZ line, which are folded, can be typed into `/tp`, while "Unwrapped" numbers past the span are
    refused.
  - `/fill`, `/fillbiome`, `/clone` and Create's glue command refuse a box that crosses the seam with "That area crosses the
    world seam and could mean either side of it — run the command once per side" (`commands.toroidal_world.region.across_seam`).
  - `/locate` measures its distance through the seam (`LocateCommandMixin`).
- **[V] Compasses and lodestones point the short way.**
  - `CompassAngleStateMixin` points the needle at the copy of the target nearest the holder.
  - `minecraft:lodestone_tracker` is a known position component (modding.md), read "seated" on the client.
- **[V] Maps.**
  - `MapItemMixin.update` measures the holder from the nearest copy of the map's centre.
  - `MapItemSavedDataMixin` places markers and banners at the nearest copy.
  - [I] Server block reads are folded (`LevelMixin.getChunk`, `ServerChunkCacheMixin`), so a map next to the seam fills in
    both sides.
- **[V] Stored spawn points are inside the span.** World spawn goes through `ServerLevelMixin.setDefaultSpawnPos`, and beds
  and anchors through `ServerPlayerMixin.setRespawnPosition`; both go through `SeamRespawnData.insideBounds`.
  `ClientAnchorSync` keeps re-sending the client the nearest copy of the spawn, border centre and chunk-cache centre as it moves.
- **[V] Going all the way round.**
  - `engine/seam/circumnavigation/SeamTravel` adds each tick's folded x step per dimension. When the running total reaches ±C,
    it grants the advancement "Back Where You Started" ("Travel a full lap around the world") and subtracts C.
  - It counts net travel: 8,000 blocks east then 8,000 west is no lap.
  - Nothing else happens at a lap. The client's x has grown by C, the server's is where it started, and the terrain is the
    same blocks.

### 7.6 What this changes in earlier text

- Section 1.2 and the seams page said players are "moved when their movement packet crosses". **[V]** That is right, with
  two additions: the client is not told, and F3 shows the folded number anyway.
- The seams page's "What a player sees" view was right that chunks arrive beside you at shifted positions. It said nothing
  about F3 (folded plus "Unwrapped" lines), about re-basing on login, respawn and dimension change, or about commands
  refusing out-of-span coordinates.
- STATUS.md's "`setPos` is a teleport and stays unfolded" is too strong (7.4).
- Still unverified without a client: whether the crossing is visually seamless (no hitch or pop-in), how contraptions and
  ship hulls render at the seam, and Simple Voice Chat, Antique Atlas and other unlisted mods near the seam (6.2).

---

## Sources

- **Toroidal World.**
  - Code: github.com/anyttng/toroidal_world, branch `mc/1.21` at `3c1f3a30`. Docs: `README.md`, `DESCRIPTION.md`,
    `docs/how-it-works.md`, `docs/modding.md`, `docs/supported-mods.md`, `docs/dedicated-server.md`. Build pins:
    `gradle.properties`. Files: `common/src/main/java/com/toroidalworld/{engine/gen/LoopedChunkGenerator.java,
    engine/gen/ShapedDimensions.java, core/ShapedChunkGenerator.java, core/WorldLoopSizes.java,
    mixin/NoiseBasedChunkGeneratorMixin.java, api/v1/gen/GenerationProbe.java, compat/sable/SableRigidShift.java,
    compat/sable/SablePoseFold.java, compat/sable/mixin/SubLevelPhysicsSystemMixin.java}`.
  - Issues: #20, #36, #41, #42, #43 and #51 (`gh api repos/anyttng/toroidal_world/issues/<n>`).
  - Modrinth: api.modrinth.com/v2/project/toroidal-world.
- **Immersive Portals.**
  - Code: github.com/iPortalTeam/ImmersivePortalsModForNeo at `aede93a`. Files: `src/main/java/qouteall/imm_ptl/core/{portal/global_portals/WorldWrappingPortal.java,
    portal/Mirror.java, commands/PortalCommand.java}` and `gradle.properties`.
  - Issues and PRs: #38, #52, #56, #64, #66, #67, #71, #73 and #74.
  - Upstream archived: github.com/qouteall/ImmersivePortalsMod.
  - Docs: qouteall.fun/immptl/wiki/Commands-Reference and qouteall.fun/immptl/wiki/Implementation-Details.
  - Modrinth: api.modrinth.com/v2/project/{immersive-portals-neoforge, immersiveportals, immersive-portals-continued,
    immersive-portals-compat, immersive_portal_sable_bridge, valkyrien-skies-iportal-compat, immersiveportalspaperized}.
- **IP forks and patches.** github.com/r2smith141/IPSable at `f5e0471d` (`README.md`, `build.gradle`, `gradle.properties`,
  releases); github.com/rebbyIf/ImmersivePortalSableBridge (`README.md`); github.com/jmcmahon1999/immptl-compat (README).
- **Sable.** github.com/ryanhcode/sable at `6966d29` (2.0.5). Files: `common/.../api/physics/PhysicsPipeline.java`,
  `api/physics/handle/RigidBodyHandle.java`, `sable_rapier/.../RapierPhysicsPipeline.java`,
  `sable_rapier/src/main/rust/rapier/src/lib.rs`, `api/sublevel/SubLevelContainer.java`,
  `mixin/world_border/WorldBorderMixin.java`, `command/SableSubLevelCommands.java` and `LICENSE.md`. Jar check: javap on the
  MIC `sable-neoforge-1.21.1-2.0.5.jar`.
- **Create.** javap of `PortalTrackProvider` in `create-1.21.1-6.0.10.jar`.
- **Distant Horizons.** javap of `IDhApiWorldGenerator`, `EDhApiWorldGeneratorReturnType` and
  `IDhApiWorldGeneratorOverrideRegister` from `DistantHorizons-3.2.0-b-1.21.1-fabric-neoforge.jar`.
- **Petrolpark.** `META-INF/accesstransformer.cfg` in `petrolpark-1.21.1-1.5.6.jar`.
- **Circumnavigate.** github.com/FamroFexl/Circumnavigate at `5d662fb`; wiki pages `Principles.md` and `Compatibility.md`;
  api.modrinth.com/v2/project/circumnavigate.
- **TFC Eratosthenes.** github.com/trygve55/tfc_eratosthenes at `017f0e9`, branch 1.21.1:
  `src/main/java/net/trygve55/eratosthenes/PlayerEventHandler.java` and `README.md`.
- **Looping World.** github.com/neuromuser/looping-world at `bf208e1`.
- **Planet – Border teleport.** github.com/Anch0vu/Planet---Border-teleport (`README.md`) and the jar
  `planet-1.0.0.jar` (javap).
- **MIC pack versions.** `pack/pack.toml` and `pack/mods/{sodium,distanthorizons,c2me-neoforge,sable,create,create-aeronautics}.pw.toml`.
- **Earlier notes.** `aeronautics_rust_bridge.md` (Sable native loading; `StaticPhysicsPipeline.teleport` is a pass-through)
  and `minecraft_integration_architecture.md` (generator options).
