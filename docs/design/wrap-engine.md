# A wrap engine of Deep Time's own: half-turn and quarter-turn seams without portals

**Status (2026-10-03).** A design and a dev-only spike, on branch `dev/wrap-engine-spike`. Ben asked
for it in the decisions log on 2026-10-03: "Immersive portals, begin custom wrap engine concurrently."
The production seamless world uses Immersive Portals first ([`seamless-wrapping.md`](seamless-wrapping.md)).
This engine runs alongside that work and covers what portals cannot carry across a seam:
- building and digging next to a seam;
- fluids;
- redstone;
- caves;
- entities' collisions.

Nothing here changes the shipped mod. The spike's code is applied only with
`-Ddeeptime.wrapspike=true` (§7.4).

**Phase 1 (2026-10-04).** Ben chose "Folded access + band (Recommended)" and "TW slides, engine turns
(Recommended)" (decisions log 2026-10-03). The spike is now the production core in package
`wrapengine`, still dev-only: `-Ddeeptime.wrapengine=true`, `./gametest.sh -PwrapEngine`. Its API is
§9, what was built and measured is §10. Where §7 says `wrapspike`, read `wrapengine`.

**The cube (2026-10-22).** Phase 6 of the cube plan: the engine joins the turned conformal cube's 12
turning edges, with Toroidal World keeping the slide. Its design, the corners, the border and the
order of E1 to E8 are §12; the API additions are §9.6.

Read first: [`seamless-wrapping.md`](seamless-wrapping.md) (the gluing tables, what breaks with
portals), [`rotated-seam-worlds-prior-art.md`](../research/notes/rotated-seam-worlds-prior-art.md)
(nobody has built turning seams), [`immersive-portals-pillowcase-prototype.md`](../research/notes/immersive-portals-pillowcase-prototype.md).

**Tags.**
- [R]: run and seen (a GameTest assertion line, a log line).
- [C]: read in source. Minecraft and NeoForge 21.1.249 sources; Toroidal World `mc/1.21` at `5ded155`,
  which is 0.19.3, read by a Sonnet sub-agent with paths cited.
- [I]: inference or judgement.
- [U]: not verified.

## 0. The answer in plain words

- **Recommended architecture: one stored copy, folded access, a replicated band** (§3). Its three parts:
  - **One stored copy.** The server stores each block of the planet once, inside the fundamental
    domain (the Guyou rectangle or the Peirce square).
  - **Folded access.** Every block position that crosses the `Level` API is folded to the block it is,
    and block states are turned on the way through. This covers reads, writes, neighbour updates and
    scheduled ticks. Block logic therefore always runs on stored positions; only its *reads* reach
    across a seam. Fluids, redstone and shape updates see the far side as ordinary neighbours, with no
    code of their own.
  - **A replicated band.** The real chunks beyond each edge, out to a band depth, hold turned copies
    of what they show. The engine writes a copy straight into the chunk sections whenever the stored
    block changes. Anything that reads chunk storage directly then sees the far side unchanged: the
    vanilla client, collisions, light, heightmaps and path-finding regions.
  - **Entities** are carried across the seam line by the seam's motion, with their heading and
    velocity turned.
- **The vanilla client needs nothing to join.** It is sent ordinary chunks, including the band, and
  ordinary teleports. An optional client part removes the one visible flaw: the reload after crossing
  a turning seam (§4.6).
- **The spike shows the core idea works server-side** (§7) [R], in a GameTest with one half-turn seam:
  - a block placed beyond the seam lands on the far side, turned 180°, and the band shows it;
  - it is solid on both sides, and stops an entity at the seam from this side;
  - water crosses and runs on down the far channel, level by level;
  - redstone powers across by wire and through a repeater that faces the seam, and switches off again;
  - an armour stand and an item are carried across, turned;
  - block light reaches across the seam.

  All 34 assertions held, with a control site where nothing crosses. The client side was not run
  (§7.3).
- **Effort.** About 9 to 16 weeks of agent time for the whole engine at MIC quality (§6). About 6 to 9
  weeks without Create trains and kinetic networks across seams, which can stay blocked at first. The
  greatest unknowns are world generation across seams together with C2ME (caves and features), and
  Create.
- **Questions for Ben** are in §8, recommendation first.

---

## 1. What a seam is, in this engine

- **The world is a fundamental domain** (a rectangle or a square) plus a set of **edges**. Each edge
  has a rigid motion of the block grid that takes the strip beyond it to the strip inside some edge.
  The motions are drawn from translations, half-turns and quarter-turns.
- **The layouts:**
  - **Guyou** (the recommended aspect, `seamless-wrapping.md` §2): the rectangle x ∈ [−h, h),
    z ∈ [−h/2, h/2). The east and west edges slide by 2h. The north and south edges each turn half
    about their midpoint: block (x, z) beyond the north edge is block (−x − 1, −h − z − 1). That is
    the block form of the task's (x, z) → (−x, 2b − z), with b = −h/2 and yaw + 180°.
  - **Peirce:** a square of side a whose four edges each turn half about their midpoints.
  - **Quarter-turns** (cube-net layouts, or a projection the survey may find): an edge glued to an
    adjacent edge by a 90° turn about their shared corner.
- **The algebra** (`wrapspike/CellIsometry`, `WrapEdge`, `WrapTopology`):
  - A motion is k quarter-turns followed by a translation. On block cells, the half-turn about the
    grid point (a, b) is (x, z) → (2a − x − 1, 2b − z − 1).
  - `fold(x, z)` applies edge motions until the block is inside. It returns the stored block and the
    composed motion; a corner takes two steps.
  - `images(x, z)` walks the inverse motions to list every band block that shows a stored block.
  - The unit tests (`WrapTopologyTest`, seeded random blocks, properties only) check, for the spike
    seam, Guyou, Peirce and a quarter-turn corner:
    - motions compose and invert;
    - every band image folds back to its stored block, and every band block is listed as an image of
      its stored block;
    - the fold is a **local isometry**. Two neighbouring blocks anywhere in the bands are still
      neighbours after folding, turned the same way. That is exactly what makes the seam invisible to
      fluids, redstone and collisions.
- **Every turn point and edge must lie on a chunk corner or chunk line.** A band chunk is then a turned
  copy of exactly one stored chunk. Guyou with h a multiple of 32 satisfies this (b = 8,192 does), and
  so does Peirce with a side that is a multiple of 32.
- **Turn points (cone points) are where turning seams meet themselves.** Guyou has four: the long
  edges' midpoints and the rectangle's corners. Near one, a player sees the same stored blocks twice,
  once directly and once in the band, as with Immersive Portals. Blocks need nothing special. Entities
  need ghosts to show there (§4.5).

## 2. How Toroidal World does it, and why a turn breaks that

Toroidal World (TW) 0.19.3 is MIT-licensed. MIC ships it, and Deep Time requires it for its x wrap.
It is the best-engineered wrap engine on this stack, and it is translation-only by construction [C]:

- **Two frames.**
  - The server stores one canonical copy and folds every server-side position into bounds.
  - The client lives in an unwrapped, continuous frame and never sees a seam. A per-player mirror of
    the client frame (`engine/seam/ClientPosition`) plus one choke point, `PacketTranslator` (hooked
    on `Connection.send` and `channelRead0`), convert between the two.
  - The client frame is re-based only at login, respawn and dimension change.
- **Chunk-level aliasing.**
  - `LevelMixin` folds the chunk key in `Level.getChunk` and folds positions in `setBlock`,
    `getBlockEntity` and `removeBlockEntity`.
  - `ServerChunkCacheMixin`, `ChunkMapMixin` and `DistanceManagerMixin` fold chunk keys through the
    chunk system. The light engines' node steps are folded too.
  - There is no mixin on `getBlockState` and none for neighbour updates. They work because `pos & 15`
    is unchanged by a whole-chunk translation, so an out-of-bounds position reads the canonical chunk
    correctly.
- **Chunk packets carry the same bytes.** Only the x and z in the header change; sections, light,
  heightmaps and biomes pass through untouched. Block updates, entity positions, sounds, particles,
  block-entity tags (`positions.json`) and records with position fields are moved to the nearest copy.
  The catalogue is 30 clientbound and 11 serverbound packet kinds.
- **The gates.**
  - `WorldFolds.verifyFoldable` refuses any shape whose seam "reverses the local indices inside a
    chunk".
  - `SeamTransform` has per-axis signs but no x/z swap, so it cannot represent a quarter-turn.
  - `FoldOrientation.HALF_TURN` exists, as does `DeckGroupFold` (only tests build it), and the public
    API documents a half-turn orientation. But `SeamSnap` turns no velocity or yaw, mob paths are
    shifted by an integer offset only, and `SableRigidShift.requireTranslation` throws on any turn.
- **Size.** 1,110 main Java files, about 60k lines and 661 mixins. Create alone is 126 mixins and
  Sable 26. The per-axis helpers (`blockDomain`, `foldBlock`, `nearestCoord`, `unwrapAround`, …) have
  roughly 280 call sites, every one assuming x and z fold independently.

**What breaks under a half-turn:**
- `pos & 15` is reversed (x → 15 − x), so chunk aliasing alone reads the wrong block.
- Every chunk payload would have to be re-encoded per client: sections, light, heightmaps, biomes,
  block entities.
- Every orientation (yaw, velocity, block-state facing, facing-bearing NBT) would have to turn.
- So TW's design point, "one copy, client frame, byte-identical chunks", does not carry over. What
  carries over is TW's *catalogue* of everything that holds a position. The recommended design reuses
  that list as a checklist, not TW's mechanism.

## 3. The candidate architectures

There are five options at the core. They differ in where the turn is applied.

| | A. Rotated chunk views | B. Folded access at the `Level` boundary | C. Replicated band | D. Client frame with turned packets (TW-style) | E. Turned rendering on the client |
|---|---|---|---|---|---|
| Idea | Out-of-bounds chunk lookups return a proxy chunk that maps (x, z) → (15 − x, 15 − z) and turns states | Fold the *position* in `getBlockState`, `setBlock`, `getFluidState`, `getBlockEntity`, neighbour updates and scheduled ticks; turn states in and out | Real chunks beyond each edge hold turned copies of what they show, rewritten on every stored change | The client never crosses a seam; the server turns every packet into the client's frame | The client holds stored chunks and draws turned copies beyond the seam |
| Server logic: fluids, redstone, ticks | Through the proxy, only where code goes through `getChunk().getBlockState` | **Yes**: all block logic uses the `Level` API [R, spike] | No: copies are pictures; logic must not run on them | Needs B underneath (aliasing breaks under a turn) | — |
| Direct chunk-storage readers (collision cursor, light engines, heightmaps, packet encoding, random ticks, path-finding regions, DH, Sodium, C2ME) | **No**: sections, palettes and nibble arrays are read directly, so the proxy must be at section level, which means copying | No | **Yes**: they read real copies [R, collisions] | Packets: by re-encoding; collisions and light: need B or C | Client side only |
| Client | Needs packet re-encoding anyway | Unchanged | **Vanilla client works**: the band is ordinary chunks | Rotated chunk encoder per client, all ~40 packet kinds turned, orientations in mod payloads | Deep Sodium and renderer mixins (as Valkyrien Skies 2 and Sable draw ships); client raycasts and collision prediction also need folding |
| Precedent | None | `Mrcavas/planets` redirects `getBlockState`/`setBlockState` between regions (AGPL, ideas only) | Immersive Portals' "hidden outside" is an *unsynced* band; this syncs it | TW, translation only | VS2 and Sable for ships |
| Main risk | Every direct reader of sections | Cost on the hottest call (§5); code that writes `LevelChunk` directly | Keeping copies in sync when the stored chunk is not loaded; copy ticks and spawns | Size: TW needed ~60k lines for translations; rotations add orientation to every rewriter | Fragile across Sodium versions; client-side logic still unfolded |

**What the comparison says:**
- **A is dead.** The direct readers are the majority. A section-level proxy is a copy, which is C.
- **B is necessary under any design.** Under a turn, chunk aliasing cannot give block logic the right
  neighbours. The spike shows B alone is enough for fluids and redstone [R].
- **C is the cheapest way to make every direct reader right.** That includes the vanilla client: no
  packet rewriting, no client mod, no renderer work. It also fixes the Immersive Portals prototype's
  worst bug by construction: collisions against an unseen "hidden outside" (`seamless-wrapping.md`
  §5.3).
- **D is the polished end state for crossings, not a starting point.** The player never sees a
  teleport, and Distant Horizons stays centred. But it is TW's 60k lines plus orientations, and it
  needs B (and C or folded collision and light) on the server anyway. §4.6 gets most of its benefit
  for a fraction of the cost.
- **E is rejected.** It needs a client mod deep in Sodium, and client-side prediction (collisions,
  block outlines, picking) would still see unfolded space.

**Recommendation: B + C, with entity carry at the seam line, and an optional client re-anchor (§4.6)
later.**

## 4. The subsystems: what each touches, and its risks

Mixin targets are Minecraft 1.21.1 / NeoForge 21.1.249 names [C].

### 4.1 Folded block access (B)

- **What the spike already hooks (`mixin/wrapspike/`):**
  - **Level:** `getBlockState`, `getFluidState`, `getBlockEntity` and
    `setBlock(BlockPos, BlockState, int, int)` at HEAD (fold, turn, recurse), plus `setBlock` at RETURN
    (replicate).
  - **BlockStateBase:** `BlockBehaviour.BlockStateBase.handleNeighborChanged`, which re-runs the update
    on the stored block, with the source position moved by the same motion.
  - **LevelTicks:** `LevelTicks.schedule` and `hasScheduledTick`, which keep ticks at the stored
    position. The topology reaches them through a field set at install.
- **What needs nothing:**
  - **Shape updates.** `NeighborUpdater.executeShapeUpdate` reads through `getBlockState` and writes
    through `setBlock`.
  - **Signal queries** (`getSignal`, `hasNeighborSignal`, `getBestNeighborSignal`). These read the
    turned view at the band position, which is isometric to the stored frame.
  - **Fluid spreading.** `FlowingFluid.spread`/`spreadTo` use `level.getBlockState`/`setBlock`.
- **What production adds:**
  - **Block entities:** `removeBlockEntity` and `setBlockEntity`.
  - **Block events:** `blockEvent` for pistons, note blocks and chests.
  - **Effects at block positions:** `levelEvent`, `playSound` and `gameEvent` at band positions.
  - **Explosions:** `Explosion.explode` collects positions and reads and writes through `Level`; the
    blocks are fine, but entity knock-back across the seam needs §4.5's distance rule.
  - **Ticks:** `willTickThisTick` and `clearArea`, which TW also hooks.
  - **`LevelChunk.setBlockState` as the write choke point.** A write into a band chunk that is not a
    replica write is redirected to `Level.setBlock`, folded. This catches mods and tools that write
    chunks directly (TW folds `LevelChunk.setBlockState` too [C]).
  - **Containers:** `Container.stillValidBlockEntity`. A chest opened across a seam has its stored
    position up to the world's width away, and closes at once unless the distance check uses the
    nearest image.
- **Risks:**
  - **Cost.** `getBlockState` is the hottest call in the game; see §5.
  - **Identity.** Block code that keeps a `BlockPos` and later compares it with a block entity's
    `getBlockPos()` sees the band position and the stored position as different. Ordinary blocks do
    not do this; some mods' multiblocks may [I].
  - **WorldGenRegion.** World generation has its own accessor, `WorldGenRegion`; §4.4.

### 4.2 The replicated band (C)

- **The write path (spike, `WrapEngine.replicate` and `writeCopy`).** After a stored block changes,
  for each image:
  - a `LevelChunkSection.setBlockState` with no `onPlace`, `onRemove`, neighbour updates or ticks;
  - then the four heightmaps, the section's light status, the sky-light sources and a light check;
  - then `ServerChunkCache.blockChanged`, as `LevelChunk.setBlockState` does for a real change.

  The client's block-update packet is built through `Level.getBlockState`, which is folded, so it
  carries the same turned state as the copy.
- **The band manager (production).**
  - **Tickets.** A band chunk holds a ticket on its source chunk while it is loaded.
  - **Copy on load.** When both are FULL, the band chunk is rewritten from the source, then resent to
    tracking players. Until then it holds its own generated terrain. Beyond an edge the generator
    already answers with the folded planet (`seamless-wrapping.md` §5.1), so untouched ground matches
    even before the copy.
  - **Saving.** Band chunks may save normally (simplest); they are re-synced on load anyway.
- **Suppressed on band chunks:**
  - random ticks (`ServerLevel.tickChunk`), which would otherwise run the stored block's random tick
    twice through the fold;
  - natural spawning (`NaturalSpawner.spawnForChunk`);
  - block-entity tickers (copies carry no block entity).
- **Block entities in the band.** Copies carry block states, not block entities. The client still
  needs data to draw chests, signs and banners in the band. Production sends
  `ClientboundBlockEntityDataPacket` for band positions, with the tag's position and orientation
  fields turned, driven by a list of position-bearing keys as TW's `positions.json` is.
- **Band depth.** It must cover:
  - an entity's half-width, for collisions;
  - 15 blocks, so block light reaches over;
  - the server's view distance, if crossings should never show the band's outer edge.

  Light and collisions need 2 chunks. Full view needs 8 to 12 chunks [I]. Writes cost one extra
  section write per image. Memory is the chunks a player near a seam loads anyway, since vanilla loads
  that "outside" too (`seamless-wrapping.md` §5.4).
- **Risks:**
  - **Source not loaded.** A band chunk loaded while its source is far away (up to the world's width)
    must wait on the source's ticket. That doubles chunk loading near seams. The doubled set is
    exactly the destination of a crossing, so it is not wasted.
  - **Consistency under concurrency.** C2ME and async light engines touch sections off-thread; replica
    writes must run on the server thread.
  - **Direct chunk writes into the band.** Caught by the choke point in §4.1.

### 4.3 Light, heightmaps, collisions, path-finding regions

- **Read directly from storage, so the band fixes them with no code [I]:**
  - `BlockCollisions` reads `getChunkForCollisions(...).getBlockState`; the spike shows a band copy
    stopping an entity [R].
  - Light engines propagate through neighbour sections.
  - Heightmaps are per chunk.
  - `PathNavigationRegion` snapshots chunks.
- **Light is computed on both sides from the same geometry**, so it agrees across the seam up to the
  band's outer edge [I, not measured].

### 4.4 World generation: caves, carvers, features, structures

- **Terrain is already continuous** through the generator's folded outside (`seamless-wrapping.md`
  §5.1, measured).
- **Noise caves and aquifers are density functions of the raw position.** A cave cut by the seam does
  not continue, because the far side samples the noise at a different point.
  - Fix: make the 3-D noises symmetric under each seam near it. Within a blend width w of an edge with
    motion g, sample f(p) = (1 − s)·N(p) + s·N(g p), with s rising from 0 at depth w to ½ at the edge.
  - That is continuous through the seam and identical on both sides [I]. It is deterministic per
    position and needs no neighbouring chunk, so it is safe under C2ME's compiled density functions
    only if those are wrapped too (TW emits periodic code into C2ME's compiler for the same reason).
  - The blend lowers the noise's variance at the seam; a variance-preserving weight (√ form) avoids
    visible flattening.
- **Carvers, features and structures are seeded per chunk and write up to 8 chunks away.**
  - Continuity needs two things:
    - a folded `WorldGenRegion`, which applies the same §4.1 fold to its `getBlockState`/`setBlock`;
    - folded chunk dependencies (`ChunkMap.getChunkRangeFuture` and the generation step's neighbour
      cache), so a chunk at a seam waits for its folded neighbours.
  - TW does exactly this for translations, with 14 mixins for C2ME's own chunk system [C].
  - Until then, carvers and features that would cross a seam are cut at it. Where that matters, a
    keep-out band of a chunk at the seam hides it.
- **Risk.** The highest-uncertainty part. Cross-region dependencies in chunk generation risk deadlocks,
  and C2ME replaces the chunk system.

### 4.5 Entities: crossing, collisions, ghosts, AI

- **Crossing (spike: `WrapEngine.carryEntities`).**
  - An entity whose position has passed an edge is moved by the motion. Its yaw turns by k·90°,
    velocity by the turn, and head and body rotation too.
  - Production hooks the same places as TW, plus rotation:
    - `ServerLevel.tickNonPassenger` (tail) for entities;
    - `ServerGamePacketListenerImpl.handleMovePlayer` and `handleMoveVehicle` for players and
      vehicles;
    - passengers, leashes and `xo`/`zo`/`xOld`/`zOld` turned.
  - Players get a teleport. With the band, the scene around them is the same, turned.
- **Collisions for entities straddling the seam** come from the band copies [R]. That is the case
  Immersive Portals gets wrong.
- **Ghosts.** An entity within the band's reach of a seam is invisible from the other side, because
  the band copies blocks, not entities.
  - Production sends each client a server-side proxy: `ClientboundAddEntityPacket` with a synthetic
    id, the turned position and yaw, mirrored moves and data.
  - Interactions with a ghost id are mapped back to the real entity.
  - No client mod is needed. Near a turn point an entity appears twice, as in Immersive Portals.
- **AI and path-finding.**
  - Distances (`Entity.distanceTo`, `closerThan`, targeting) should use the nearest image, as TW wraps
    them with `nearestCopy` [C].
  - Paths that run into the band are turned with the entity at the crossing (TW shifts paths by an
    integer [C]; a turn maps nodes by the motion), or simply recomputed.
- **Risks:**
  - **Tracking.** `ChunkMap.TrackedEntity` would track ghosts.
  - **Mod entities.** Contraptions and ships (§4.8) cross under their own rules.

### 4.6 Client and networking

- **Phase 1 needs no packet rewriting and no client mod.**
  - The band reaches the client as ordinary chunks; crossings are vanilla teleports; ghosts are vanilla
    spawn packets.
  - Vanilla clients and clients without Deep Time's client side can join.
- **The one visible flaw: the reload after a turning crossing.**
  - A half-turn sends the player up to the world's width away in raw coordinates, so the client drops
    its chunks and receives the destination's.
  - The server has them loaded already (they are the band's sources), so it is network-bound. It
    would still show void for a second or two [I, not measured].
- **The optional client part: an isometric re-anchor.**
  - On a `SeamCrossing(motion)` packet, the client moves every cached chunk c to g(c), turns its
    contents (section palettes are permuted and their states turned, light nibbles and heightmaps
    permuted, block entities turned), and moves its entities by g.
  - With the band at least as deep as the view distance, every cached chunk lands on a correct chunk:
    a band copy turned back is the stored chunk, and a stored chunk turned is the far side's band.
  - The scene does not change. The render sections rebuild, a frame spike, not a void.
  - This is TW's client-frame benefit (D) at a crossing, without turning every packet forever.
- **Mixin targets for the re-anchor:** `ClientChunkCache`, `ClientLevel`, `LevelRenderer`, and
  Sodium's section manager (MIC ships Sodium 0.8.13).
- **D stays an option** if the re-anchor's frame spike is not good enough.
- **Risks:**
  - **Sodium.** The section rebuild under Sodium is not tested.
  - **Interaction checks.** Interaction-distance checks on the server use raw positions. Clicks in the
    band are within reach of the player in raw coordinates, which is right, but containers' range
    checks are not (§4.1).

### 4.7 Distant Horizons

- **What DH draws.** DH draws LODs from loaded chunks and, beyond them, from Deep Time's own generator
  (`compat/dh/DhLodGenerator` → `FarTerrain`).
- **With the band:**
  - band chunks are real chunks with real copies, so DH builds them in, edits included;
  - beyond the band, `FarTerrain` must answer positions beyond each edge with the folded planet. That
    is the change `seamless-wrapping.md` §7.3 already lists, a few dozen lines;
  - a heightfield needs no per-block turn.
- **On a crossing**, DH re-centres around the destination, whose LODs exist in its store under stored
  positions [I].
- **The shipped TW guard mixin (`compat/dh/TwDhGuard`) is unaffected.**
- **Risks:**
  - DH may build band chunks into its store under their raw keys beyond the world's edges, which wastes
    space [U].
  - A crossing re-centre costs a LOD reload [I].

### 4.8 Compatibility with the MIC pack

| Mod | What crosses, what does not | Work |
|---|---|---|
| **Toroidal World** | Guyou's slide edges are TW's x wrap as shipped. The engine adds only the two turning edges, which TW never sees: z does not loop for TW. Both fold x in the corner bands; the engine folds first, so TW sees in-bounds positions [I]. | Phase 4: coexistence tests; TW's client mirror re-bases after an engine teleport through `teleport()`, which TW hooks [C]. Peirce has no slide edges, so TW would have to go. |
| **Create**, kinetic networks | Shafts, cogs and belts read neighbours through `getBlockState`/`getBlockEntity`, so they would connect across a seam. Under a half-turn a shaft's axis direction flips, so its speed sign must flip, and Create's propagation uses position differences between frames [I]. | **Adapter built** (§11.2): shafts and cogs connect across half- and quarter-turns with the speed sign turned; belts, chain conveyors and gantries are refused [R]. |
| **Create**, contraptions | Carried as entities, yaw turned. Contraption entities hold their own blocks, so the band does not show them; ghosts do. | Assembly beyond a seam is refused with Create's own message, and contraption entities are left out of the carry (§11.5) [R]. Crossing: 11–19 days. |
| **Create**, trains | The track graph is keyed by absolute positions and curved tracks store absolute positions; an edge across a seam would join stored positions up to the world's width apart. TW needed its own fold for slides [C]. | Track within a block of a seam is refused (§11.4) [R]. Crossing through a same-dimension portal edge: 13–22 days for a half-turn. |
| **Create**, fluids and pipes | Pipe searches, pump pressure and the transfer network walk in one block's frame and hand faces to block entities stored in another. | **Adapter built** (§11.3): water pumped across a half-turn reaches a tank on the far side [R]. |
| **Sable** | Bodies cross through Sable's public `PhysicsPipeline.teleport(body, position, orientation)` with the orientation turned. Terrain near a seam comes from the band. | Phase 4. TW's Sable bridge refuses any turn (`requireTranslation`) [C], so the engine owns this for its turning edges. |
| **C2ME** | Server chunk system and compiled density functions. | Phase 2 (§4.4) and band sync off-thread (§4.2). |
| **Sodium** | Renders the band as ordinary chunks. | Only the optional re-anchor touches it. |
| **Petrolpark/Destroy, CBC Modern Warfare** | They crash only with Immersive Portals' `ChunkMap` overwrite (`immersive-portals-pillowcase-prototype.md` §6). The engine overwrites nothing in `ChunkMap`. | Check at Phase 4 [I]. |
| **ScalableLux** | Light engine. | Check that band copies light correctly. |
| **Maps, compasses, waypoints** | Read raw coordinates; near a seam the band shows the right terrain. | Already needed for any quincuncial world (`seamless-wrapping.md` §7.6). |

## 5. Cost on the hot path

- **The cost today.** `Level.getBlockState` runs millions of times a tick. The spike's HEAD injection
  allocates a `CallbackInfoReturnable` per call and walks the edge list.
- **What production does instead:**
  - an inner-box test first, four integer compares: almost every position is far from every seam;
  - one non-volatile topology field;
  - a `@WrapMethod` or a direct redirect in place of a cancellable injection, so the common path
    allocates nothing.
- **What it would cost then.** A few nanoseconds per call [I]. It must be measured on a seam-free
  world before anything ships.

## 6. Recommended plan and effort

Agent working days. These are judgements, not measurements. The spike is the only evidence.

| Phase | What | Days |
|---|---|---|
| 0 | **Spike** (this branch): the algebra, folded access, replica writes, entity carry; GameTests for placement, water, redstone, collisions, crossings | done |
| 1 | **Server core.** Whole-world topologies from the world data's layout (Guyou, Peirce); the fast path (§5); the full fold list (§4.1) with the `LevelChunk` choke point; the band manager (tickets, copy on load, resync, random-tick and spawn suppression, block-entity data to clients); entity carry for players, vehicles, passengers and leashes; container range; GameTests on a whole-world seam dimension | 10–15 |
| 2 | **World generation across seams.** The generator and `FarTerrain` folded beyond every edge; symmetric noise at seams for noise caves and aquifers (C2ME's compiled form included); a folded `WorldGenRegion` and chunk dependencies for carvers, features and structures, or keep-out bands | 8–15 |
| 3 | **Client experience.** Ghost entities; the optional `SeamCrossing` re-anchor (vanilla clients keep the reload); F3, compass and "north"; a real-client walk script (as the Immersive Portals prototype's) | 8–12 |
| 4 | **Pack compatibility.** TW coexistence for Guyou's slides; Distant Horizons; Sable bodies; Create contraptions, plus kinetic and fluid networks blocked or adapted at seams; path-finding and AI distances; C2ME, ScalableLux and Sodium checks | 15–25 |
| 4b | Create trains across turning seams (optional) | 10–20 |
| 5 | **Hardening.** Performance on a seam-free world and at a busy seam; multiplayer; turn points; save and reload; corners | 5–8 |
| | **Total** | **46–75 without trains (about 9–15 weeks); 56–95 with them** |

Compared with the earlier estimate of 6 to 12 weeks on top of Immersive Portals
(`seamless-wrapping.md` §8), this is somewhat higher. That figure did not count world generation
across seams. Parts of Phases 2 and 4 (the generator fold, DH, maps and north) are needed by the
Immersive Portals version anyway, so the engine's own share is about 6 to 9 weeks [I].

**Order.**
1. Phase 1 makes the engine usable on a dev world, with no client mod and with Create's networks
   refused at seams.
2. Phases 2 and 3 make it look right.
3. Phase 4 makes the pack work.
4. Ship behind a world option, Immersive Portals or the engine, until Phase 4 passes the MIC GameTests.

## 7. The spike

### 7.1 What was built (branch `dev/wrap-engine-spike`)

- `neoforge/src/main/java/.../wrapspike/`:
  - `CellIsometry`, `WrapEdge`, `WrapTopology`: the algebra, with factories for the spike seam, Guyou,
    Peirce and a quarter-turn corner;
  - `WrapEngine`: install, replicate, resync, entity carry;
  - `WrapLevel` and `WrapTicks`: duck interfaces;
  - `WrapSpikeGameTests`.
- `neoforge/src/main/java/.../mixin/wrapspike/`: `LevelWrapMixin`, `BlockStateBaseWrapMixin` and
  `LevelTicksWrapMixin`, listed in `deeptime.wrapspike.mixins.json`.
- `neoforge/src/test/java/.../wrapspike/WrapTopologyTest`: the algebra's properties (§1).

### 7.2 The test

`./gametest.sh -PwrapSpike` on the test VM (namespace `deeptime_wrapspike`, `-Ddeeptime.wrapspike=true`).

- **The sites.** Each GameTest builds a 16 × 16 site at x = 4096 + 64k, z0 = 4096, y 100, in the
  GameTest server's flat overworld:
  - the stored side is z ≥ z0;
  - the band is z ∈ [z0 − 8, z0), x ∈ [cx − 8, cx + 8);
  - the glue is the half-turn about (cx, z0): block (x, z) beyond is block (2cx − x − 1, 2z0 − z − 1).
- **`placedAcrossSeamIsTurnedAndSolid`:**
  - stairs facing north set at the band position (cx + 1, z0 − 1), the way a player's click there
    ends up;
  - stairs set inside at (cx − 5, z0), facing east;
  - an armour stand moved one block north at x = cx + 1.5 (where the far side has the stairs) and at
    x = cx + 3.5 (where it is open);
  - a no-gravity item thrown north at 0.4 blocks a tick.
- **`waterFlowsAcrossSeam`:**
  - a source one block inside, in a 1-wide channel A at x = cx + 2 that ends at the seam;
  - channel B at x = cx − 3 is A's continuation across the seam;
  - a control site with the same build and no seam.
- **`redstonePowersAcrossSeam`:**
  - **Line 1:** a redstone block, three wires to the seam, a wire *placed beyond the seam* (which
    lands on the far side), one more wire, and a lamp.
  - **Line 2:** a redstone block, a wire, and a repeater taking input from the south and driving north
    into the seam. Then a wire placed beyond the seam and a lamp.
  - Then the power is removed.

### 7.3 What it showed

**Result: all 3 required tests passed, all 34 assertions OK** [R]. Run: `./gametest.sh -PwrapSpike`
on the test VM, 2026-10-04 02:11 UTC, 36 s wall time, peak server RSS 1.6 GB. The assertion lines are
in `neoforge/build/gametest-standalone.log` (not committed).

| Claim | Measured | |
|---|---|---|
| A block placed beyond the seam lands on the far side, turned | stairs set facing north at band (cx+1, z0−1) → stored at (cx−2, z0) facing **south**; read back through the seam: facing north | [R] |
| The band shows it, turned (what the client is sent) | chunk storage at (cx+1, z0−1) holds stairs facing north; stairs stored facing east at (cx−5, z0) appear in the band at (cx+4, z0−1) facing **west** | [R] |
| It is solid there | collision shape non-empty at both; an armour stand (half-width 0.25) at x = cx+1.5 moving 1 block north stops at z = z0 + **0.2500**, against the far side's stairs | [R] |
| Water flows across | far channel B (cx−3, z0 … z0+6) holds amounts **6, 5, 4, 3, 2, 1, 0** from a source at A (cx+2, z0+1) = 8, A z0 = 7: the run continues level by level across the seam; flow at B z0+2 points away from the seam (+1.0 in z); the band copy beyond A holds water of amount 6; **control site without the seam: B dry in 7 of 7** | [R] |
| Redstone powers across, by wire | wire placed beyond the seam lands at (cx−3, z0) and connects north into the far side; power **15, 14, 13 \| 12, 11** falls by one per block across the seam; the far lamp lights | [R] |
| Through a repeater facing the seam | repeater powered; the wire across from its output at **15**; the far lamp lights; the band copy of the repeater beyond the far side is turned (facing north) and powered | [R] |
| And turns off | redstone blocks removed: both far wires 0, both far lamps off | [R] |
| Entities are carried across, turned | armour stand moved past the seam at x = cx+3.5 → (cx−3.5, z0+0.4), heading turned **180.0°**; a no-gravity item thrown north at x = cx+5.5 → now at x = cx−5.5, z = z0+7.3, moving south | [R] |
| Light crosses | glowstone stored at (cx+6, z0): block light **14** at the far-side block (cx−7, z0) next to its band copy, 13 blocks away through ordinary space (2 at most that way) | [R] |

**What this does not show:**
- **The client.** No client was run. "What the client is sent" means the chunk storage that vanilla
  serializes into chunk and block-update packets. That the client draws the band as it is stored is
  vanilla behaviour [I], not observed here.
- **A whole world.** Only bounded seams on a flat test world were run: no Guyou or Peirce dimension,
  no turn point, no corner where two seams meet. The whole-world topologies are covered only by the
  unit tests.
- **Band management.** No source-chunk tickets, no copy on load, no unloaded sources. Each site copied
  its band once at install (768 blocks).
- **Players.** No player crossing, no ghost entities, no re-anchor.
- **The rest of the pack.** No world generation (caves), no Distant Horizons, no Create, no Sable, no
  Toroidal World on the same level.
- **Performance.** Not measured. The spike's injections allocate on the hottest call (§5).

**First run (for the record).** In the first run (02:07) the item had not moved after 20 ticks: its
freshly forceloaded chunk was not entity-ticking yet. It crossed correctly a few ticks later (the
engine's log line shows it landing at the glued point). The check now waits 120 ticks and the item
moves at 0.2 blocks a tick.

**Licences.** No code was adapted from Toroidal World (MIT) or Immersive Portals (Apache-2.0). TW's
source was read for its design and its catalogue of position-bearing state only. The spike's algebra
is written from the mathematics in §1. So nothing is added to NOTICE.md.

### 7.4 The shipped mod is unchanged

- **The mixins.** `DeepTimeMixinPlugin` applies `mixin.wrapspike.*` only with
  `-Ddeeptime.wrapspike=true`. Without it the mixins are never woven.
- **The runtime.** `WrapEngine.init` registers nothing, and `WrapEngine.topology` returns null.
- **The tests.** The spike's tests are in a namespace that only `-PwrapSpike` enables. The normal
  `./gametest.sh` runs `deeptime` alone.

## 8. Questions for Ben (recommendation first)

1. **Architecture: "one stored copy, folded access, replicated band" (recommended), or Toroidal
   World's client-frame design extended with turns?**
   - The first lets vanilla clients join and needs no packet rewriting; its one flaw is a reload at a
     turning crossing, which an optional client part removes.
   - The second never shows a crossing, but means turning every packet and chunk for every client, on
     top of the same server work.
2. **Keep Toroidal World for Guyou's slide edges (recommended), or let the engine own every edge?**
   - Keeping TW keeps its Create, Sable, C2ME and DH compatibility for the east–west wrap, and the
     engine handles only the two turning edges.
   - Owning every edge drops a 60k-line dependency, but repeats its compatibility work.
3. **Create across turning seams: refuse kinetic, fluid and train connections at seams at first
   (recommended), or build adapters in the first version?**
   - Adapters are the least certain work in the plan, and trains have no precedent.
4. **Continue to Phase 1 now, alongside the Immersive Portals production work (recommended), or stop
   at the spike until the projection survey settles the layout?**
   - The engine takes any layout of translations, half-turns and quarter-turns, so the survey does not
     block Phase 1.

## 9. The engine's API (Phase 1; stable)

Settled 2026-10-04 on branch `worktree-agent-a4445c63094be0023` (Phase 1). Adapters (Create, Sable,
Distant Horizons) and the rest of Deep Time use only what this section lists. A change to it is
announced here first. Package `com.minecraftindustrialcomplex.deeptime.wrapengine` (the spike's
`wrapspike` package is gone; the old `wrap` package is unrelated: Toroidal World's pole cap and
planet shape).

### 9.1 Switching it on

- **The flag.** `-Ddeeptime.wrapengine=true` (`WrapEngine.FLAG`, `WrapEngine.enabled()`). Without it
  the engine's mixins (`mixin.wrapengine.*`, `deeptime.wrapengine.mixins.json`) are not woven,
  `WrapEngine.init()` registers nothing, `topology(level)` is null and `install` throws. The shipped
  default is off.
- **A level.** `WrapEngine.install(ServerLevel, WrapTopology)` on the server thread; `null` removes
  it. The topology must be chunk-aligned (`WrapTopology.isChunkAligned()`). Folded access starts at
  once; loaded band chunks are copied within a few ticks.
- **Tests.** `./gametest.sh -PwrapEngine` runs namespaces `deeptime_wrapengine` and
  `deeptime_wrapbench` with the flag; `./gametest.sh -PwrapBench` runs the benchmark alone without
  it (the baseline).

### 9.2 Topology types (pure integer code, no Minecraft types)

| Type | What it is |
|---|---|
| `CellIsometry(quarterTurns, tx, tz)` | A grid motion: k clockwise quarter-turns (seen from above; one takes north to east, as `Rotation.CLOCKWISE_90`), then a translation. `applyX/applyZ(x, z)` move a block cell (allocation-free), `applyPoint` an entity position, `applyVector` a heading or velocity; `compose`, `inverse`, `yawDegrees()`, `mapsChunksToChunks()`. Factories `translation`, `halfTurnAbout(a, b)`, `quarterTurnAbout(k, a, b)` (about a block corner). |
| `WrapEdge(name, normalX, minSide, coord, spanLo, spanHi, depth, iso)` | One edge: the line `coord` across x (`normalX`) or z, beyond it the low or high side, over the span, `depth` blocks out (the band); `iso` maps beyond → inside. Spans and depth are finite. |
| `WrapTopology(name, edges)` | The edge table. `fold(x, z)` → `Fold(x, z, iso)` (the stored block and the motion band → stored) or null; `foldPoint(x, z)` → `PointFold`; `images(x, z)` (every band block showing a stored block); `inside(x, z)`; `mayFold(x, z)` (the fast-path test, false = certainly not folded); `chunkFold(cx, cz)` → `ChunkFold(sourceX, sourceZ, iso)` or null; `chunkImages(cx, cz)` (motions stored → band); `forEachBandChunk`; `union`. Immutable, thread-safe. |
| `WrapLayouts` | `halfTurnSeam`, `quarterTurnCorner` (bounded, for tests), `guyou(h, depth, slides)`, `peirce(a, depth)`, `cubeCross(s, depth, slides)` and `cubeCrossCorners(s)`. `slides = false` leaves the east–west translation to Toroidal World (Ben, 2026-10-03, "TW slides, engine turns"). |

### 9.3 Queries on a level (`WrapEngine`, static)

| Method | Returns |
|---|---|
| `topology(Level)` | The installed topology or null (always null on the client and without the flag). |
| `fold(Level, BlockPos)` | `Folded(pos, iso)`: the stored position and the motion band → stored, with `toStored()` (the `Rotation` a state takes into the stored frame, for a write) and `toBand()` (for a read); null when the position is not in a band. |
| `canonical(Level, BlockPos)` | The stored position, or the position itself. **Compare canonical positions** when deciding whether two positions are the same block (multiblocks, networks, caches). |
| `isBandChunk(Level, cx, cz)`, `isBandSynced(ServerLevel, cx, cz)` | Whether a chunk is a band chunk; whether its copy is current. |
| `rawState(Level, BlockPos)` | Chunk storage, bypassing the fold (what collisions and the client see). |
| `rotation(iso)`, `apply(iso, BlockPos)`, `turn(iso, Direction)`, `applyPoint(iso, Vec3)`, `applyVector(iso, Vec3)` | Motions on Minecraft types. |
| `stats(ServerLevel)` | The band manager's counters, for logs. |

### 9.4 What folded access guarantees (the behaviour adapters rely on)

At a position `p` in a band, with `f = fold(level, p)`:

- `level.getBlockState(p)` is the stored state turned into the band's frame (`toBand()`).
- `level.getFluidState(p)` is the stored fluid (fluid states do not turn).
- `level.getBlockEntity(p)` is the **stored** block entity; its `getBlockPos()` is `f.pos()`, not `p`.
  `removeBlockEntity(p)` removes it.
- `level.setBlock(p, s, …)` stores `s.rotate(toStored())` at `f.pos()`; the shape and neighbour
  updates run there.
- A neighbour update addressed to `p` (`BlockStateBase.handleNeighborChanged`) runs on the stored
  block at `f.pos()`, with the source position moved by the same motion.
- Scheduled ticks for `p` (`schedule`, `hasScheduledTick`, `willTickThisTick`) are kept at `f.pos()`.
  No scheduled tick, random tick, natural spawn or generation post-processing ever runs in a band
  chunk.
- `LevelChunk.setBlockState` on a band chunk (a mod writing chunk storage) is redirected to the stored
  block, turned (server thread; off it, the write stays and the chunk is re-copied when it reloads).
- A band chunk's storage holds turned copies: read-only pictures. Collisions, light, heightmaps and
  chunk packets read them.
- **Not folded:** `isLoaded`/`hasChunkAt` and `getChunk`/`getChunkAt` (they answer for the band chunk
  itself), `blockEvent`, `levelEvent`, `playSound`, `gameEvent`, explosions' entity knock-back,
  `setBlockEntity` with a band position (never do this), and `Level.getBlockEntity` off the server
  thread (null, as in vanilla).

**Directions across a seam.** Logic at stored position `a` that looks at its neighbour
`q = a.relative(d)` and finds `q` in a band sees the neighbour's state already turned into its own
frame, so most code needs nothing. Code that follows a network into the neighbour and continues from
its stored position must turn its direction too: the neighbour is at `f.pos()` and the direction `d`
becomes `turn(f.iso(), d)` there (for a half-turn, east becomes west). Kinetic speed signs that depend
on axis direction flip under a half-turn.

### 9.5 Entities

- After each server level tick, every root entity (riding nothing) whose position is beyond an edge
  is carried to `foldPoint`: yaw, head and body turned by `iso.yawDegrees()`, velocity by the turn,
  passengers turned with it and re-seated, a mob's path stopped. A player is sent a teleport and its
  motion.
- `WrapCrossingEvent` (NeoForge bus, server thread, cancellable) is posted first: `level()`,
  `entity()`, `iso()`, `from()`, `to()`. Cancel it to carry the entity yourself (contraptions, Sable
  bodies); the engine asks again next tick while the entity is still beyond the edge.

### 9.6 Additions for the cube (2026-10-22, §12)

Additions only; nothing above changes meaning, except that `fold` takes the nearest edge where bands
overlap (no phase-1 layout has overlapping bands).

| Addition | What |
|---|---|
| `WrapCorner(name, images)` | A cone point and its images (block corners) in the topology's frame. |
| `WrapTopology(name, edges, corners, periodX, windowLo)` | The general constructor: with `periodX` > 0 a block outside [windowLo, windowLo + periodX) is reduced into it before folding (Toroidal World's loop). `corners()`, `periodX()`, `nearCorner(x, z)` (within 2 blocks of an image), `translated(dx, dz)` (the same topology moved, for tests). |
| `ChunkFold.split()`, `chunkSources(cx, cz)` | A band chunk that folds by more than one motion (the diagonal of a corner square), and every (source chunk, motion) of a band chunk. |
| `WrapLayouts.cube(layout, depth, slides)` | The cube from `CubeLayout` in Toroidal World's frame; `cubeCross(s, depth, slides)` is `cube(CubeLayout.of(s), …)`, `cubeCrossCorners(s)` its corners. |
| `WrapEngine.nearCorner(level, pos)`, `joinedEdges(level)`, `nearestImage(level, pos, from)` | Queries for adapters and the border: near a cone point; the base names of the joined edges (W3 leaves them out); the band image of a stored position nearest a point. |

## 10. Phase 1 as built (2026-10-04)

Branch `worktree-agent-a4445c63094be0023`. Dev-only: nothing changes without
`-Ddeeptime.wrapengine=true`, and even then a level is unchanged until a topology is installed on it.
The shipped default is unchanged; the normal `./gametest.sh` runs without the engine's mixins.

### 10.1 What was built

- **The edge table** (`CellIsometry`, `WrapEdge`, `WrapTopology`, `WrapLayouts`). Any mix of
  translations, half-turns and quarter-turns, with finite bands. Layouts: Guyou (with or without the
  slides; without leaves the east–west wrap to Toroidal World, as Ben chose), Peirce, the projection
  survey's cube cross net (14 edges: the slide, eight quarter-turn edges, four half-turn edges), and
  the bounded test seams. The unit tests (`WrapTopologyTest`, 11 tests, properties on seeded random
  blocks) check the local isometry on every layout (the cube's away from its eight corners), the
  images, the fast path and the chunk view [R].
- **The fast path** (§5). `WrapTopology.mayFold` is a byte grid over the bands' bounding box, one cell
  per chunk (coarser for huge layouts: at most 2²¹ cells). Outside the box, or in a cell no band
  touches, nothing folds. The read path is `@ModifyVariable` at the head of `Level.getBlockState` (the
  position replaced by a `FoldedPos` only when it folds) and `@ModifyReturnValue` at its return (the
  state turned only for a `FoldedPos`): no callback object, no allocation off the bands. The less
  hot calls (`setBlock`, neighbour updates, tick scheduling) keep cancellable injections.
- **Folded access** (§9.4): `getBlockState`, `getFluidState`, `getBlockEntity`, `removeBlockEntity`,
  `setBlock`, `handleNeighborChanged`, `LevelTicks.schedule`/`hasScheduledTick`/`willTickThisTick`.
- **The choke point**: `LevelChunk.setBlockState` on a band chunk is redirected to the source chunk's
  `setBlockState`, turned; on a stored chunk with images it replicates at its return. So a mod writing
  chunk storage directly is folded too.
- **The band manager** (`BandManager`, one per level):
  - **Tickets mirror levels.** A band chunk at ticket level L ≤ 33 holds a region ticket at L on its
    source chunk (`ChunkTicketLevelUpdatedEvent`, applied at the end of the tick, since the event
    fires inside the distance manager's update). The far side of a seam is as loaded and as ticking
    as the near side.
  - **Copy on load**: a band chunk waits until its source is loaded, then only the blocks that differ
    are rewritten (sections both air are skipped), its generated block entities are dropped and, when
    it was generated just now, its generated mobs removed.
  - **Updates**: every stored change near an edge goes into each loaded band chunk that shows it.
  - **Suppressed in band chunks**: random ticks, weather ticks, natural spawning, scheduled ticks at
    band positions, generation post-processing (which would write the band's generated blocks through
    the fold).
- **Entity crossing**: every root entity beyond an edge after each level tick, yaw, head, body and
  velocity turned, passengers turned and re-seated, a mob's path stopped, a player teleported with its
  motion. `WrapCrossingEvent` lets an adapter take a crossing over.

### 10.2 Evidence [R]

`./gametest.sh -PwrapEngine` on the test VM, 2026-10-04 04:22 UTC: **all 15 required tests passed**
(namespace `deeptime_wrapengine` and the benchmark), 24 s of tests. Every test runs once across a
half-turn seam and once across a quarter-turn corner, with expectations from the topology's motions:

| Test | Half-turn | Quarter-turn |
|---|---|---|
| Building: stairs set facing north beyond the seam | stored on the far side facing south, read back north, the band copy north, solid both sides | stored facing east (a quarter turn), read back north |
| A log stored along x shows in the band | along x (a half-turn keeps axes) | along z |
| Collisions: an armour stand at the seam against the far side's stairs | stops at z = oz + 0.2500 | 0.2500 |
| Entities: an armour stand walks over | carried to the topology's fold, heading +180° | +90° |
| A no-gravity item thrown north | crosses, keeps going along the turned heading, 0.0000 off its line | the same |
| A mock player steps beyond the edge | teleported to the fold, heading +180° | +90° |
| Water: a channel straight across, source one block inside | amounts 6, 5, 4, 3, 2, 1, 0 beyond the seam; far side flows away from it; band copy 6; unglued control dry in 7 of 7 | 6 … 0; flows away |
| Redstone: wire across, then a repeater facing the seam | 15 14 13 \| 12 11, far lamps lit, the repeater's band copy turned (north) and powered, both off again | the same, copy facing west |
| Light: glowstone stored at the seam | block light 14 on the far side, 21 blocks away in ordinary space | 14, 40 blocks away |
| Band consistency: 400 random edits on the stored side, 400 through the band, a direct chunk write into a band chunk (redirected) and into a stored chunk (replicated) | 0 of 7,168 band blocks wrong after each | 0 of 10,752 |
| Unload and reload, band chunk 31 (half) and 16 (quarter) chunks from its source | source edited with the band chunk unloaded (did not load it); band chunk loaded alone: its ticket loaded the source, 0 of 98,304 blocks differ; unloading it released the source; source edited again, band chunk reloaded: 0 of 98,304 differ, the new marks show | the same |

**The first run found a real bug.** A band chunk whose ticket level drops leaves the visible chunk map
(`getChunkNow` is null) and can come back from the pending unloads without a load or an unload event.
Replication skipped it, so it kept stale copies while marked synced (the reload test failed with 3 of
98,304 blocks wrong, exactly the three edits). The band manager now tracks band chunks between their
load and unload events and re-queues the copy of one that is loaded but invisible when a replica
misses it.

**The band does not wait on a forced source in a test.** Forcing a source chunk at level 31 loads its
neighbours across the seam at 32; they are band chunks, so they ticket their sources at 32, one of
which is the stored chunk next to the band chunk under test, whose neighbour (that band chunk) then
loads at 33. That is right (the far side of a loaded place is loaded) and converges (every hop adds a
level), but it would defeat a test of a band chunk loading after its source was edited, so the reload
test edits the source under a border ticket (33).

### 10.3 The getBlockState benchmark [R]

`WrapBenchGameTests`: 16,384 fixed positions in a 2 × 2-chunk patch, four interleaved passes of every
mode, each 2 warm-up and 5 measured rounds of 4,194,304 reads; median over the 20 measured rounds
(p25–p75). Test VM, 2026-10-04, shared with other agents' jobs.

Three JVMs with the engine off (`-PwrapBench`) and three with it on (`-PwrapEngine`, one of them the
full suite), alternated. Each value is one JVM's median; the last column is the median of the three.

| Mode | Run 1 | Run 2 | Run 3 | Median |
|---|---|---|---|---|
| Engine off: baseline | 28.23 | 23.34 | 23.79 | **23.8** |
| Engine woven, no topology on the level | 24.42 | 26.18 | 25.85 | **25.9** |
| Topology installed, patch outside the bands' bounding box (the GameTest sites) | 25.11 | 25.84 | 28.08 | **25.8** |
| Whole-world Guyou (h = 65,536), patch deep inside: the grid cell says no band | 25.59 | 26.69 | 30.19 | **26.7** |
| Every read in a band: fold, read at the stored position, turn | 48.31 | 49.14 | 54.33 | **49.1** |

So off the bands the engine costs about 2 to 3 ns per read (8 to 12 %), less than the spread between
JVMs of the same build (23.3 to 28.2 ns with the engine off). The topology's grid test adds nothing
measurable over the woven null check. A read inside a band costs about twice a plain read; band reads
are a small share of all reads (only blocks within the band's depth beyond an edge). The first,
non-interleaved run ranked the modes by their order in the run (JIT profile changes), which is why the
benchmark interleaves them.

### 10.4 Not in Phase 1 (Phase 2 and later)

- **Whole-world topologies from the world data.** The layout is not chosen (Guyou or the cube cross),
  and the bake still writes Lambert, so nothing installs a topology on a real world yet. The engine
  takes either layout as an edge table (`WrapLayouts`). The cube was chosen (2026-10-22); §12 is the
  plan for it.
- **Block-entity data in the band** (chests, signs, banners drawn by the client there) and container
  range across a seam (`Container.stillValidBlockEntity` with the nearest image). E2, §12.8.
- **Not folded yet**: `blockEvent`, `levelEvent`, `playSound`, `gameEvent`, explosions' entity part,
  `setBlockEntity` with a band position, `LevelTicks.clearArea`, `isLoaded`/`hasChunkAt` (they answer
  for the band chunk, which holds its source loaded).
- **Entities**: leashes across a seam, player-driven vehicles (the client drives them), ghosts
  (entities near a seam are not shown in the band), AI distances by nearest image.
- **Off the server thread**: a band chunk written off it is left and re-copied on reload; C2ME and
  async light are not checked.
- **World generation across seams** (§4.4: symmetric noise, folded `WorldGenRegion`), Distant
  Horizons, Toroidal World on the same level, Sable, Create (another agent, in parallel), the client
  re-anchor, a multiplayer busy-seam measurement.
- **Band chunks save their copies** (simplest, as §4.2 allowed); they are re-copied on load anyway.

## 11. Create across turning seams

**Status (2026-10-04).** Built in parallel with Phase 1, as Ben asked in the decisions log on
2026-10-03: "If it can be done in parallel, do it now, otherwise defer. But we should be doing it."
- **History.** It was first built on the spike's fold, then moved onto Phase 1's API (§9) when that
  landed.
- **When it runs.** Only with `-Ddeeptime.wrapengine=true` and Create installed.
- **Where the code is:**
  - `neoforge/src/main/java/.../compat/create/wrap/`:
    - `SeamFrames` is the only class that touches the engine, through `WrapEngine.fold`, `canonical`,
      `turn`, `rotation`, `topology` and `WrapTopology.images`.
    - `SeamTurns` holds the pure algebra.
    - `KineticSeams`, `SeamRefusals` and `CreateWrapCompat`.
  - The GameTests `CreateSeamGameTests` and `CreateSeamChecks` sit in the `wrapengine` package, next to
    the engine's own tests whose sites they share.
  - `neoforge/src/main/java/.../mixin/wrapengine/compat/create/` holds 15 mixins, listed in
    `deeptime.wrapengine.mixins.json`. `DeepTimeMixinPlugin` applies a mixin in
    `mixin.wrapengine.compat.<modid>` only when the flag is set and that mod is installed.
- **Dev switches:** `-PwrapNoCompat` leaves every adapter mixin out, and `-PwrapCompatSkip=<Mixin>,…`
  leaves the named ones out.

**Create version.** 6.0.10 for NeoForge 1.21.1, as the MIC pack pins it (`pack/mods/create.pw.toml`,
`smoke/mic/server/mods/create-1.21.1-6.0.10.jar`).
- **Licence.** Create's code is MIT; its assets are all rights reserved.
- **What was used.** The source at tag `mc1.21.1-6.0.10` was read to find the hooks. Nothing was copied:
  the mixins call Create's own methods with turned inputs (NOTICE.md).

### 11.1 What a turn does to Create's quantities

**Speed.**
- Create stores a kinetic block's speed as a signed angular velocity about the *positive* direction of
  its rotation axis. `KineticBlockEntity.convertToDirection` flips the sign for a face on the negative
  side [C].
- An angular velocity is an axial vector, but under a proper rotation (a turn about the vertical) it
  turns like an ordinary one: ω′ = Rω.
- So a speed s about +a, seen through a turn R, is s about R(+a). Where R(+a) is a negative direction,
  the far block's stored speed is −s:

| Turn (clockwise from above) | x axis | z axis | y axis |
|---|---|---|---|
| 0° | x, + | z, + | y, + |
| 90° | z, + | x, **−** | y, + |
| 180° | x, **−** | z, **−** | y, + |
| 270° | z, **−** | x, + | y, + |

- **A half-turn** flips the sign of every horizontal speed (shafts and cogs along x or z) and keeps
  every vertical one.
- **A quarter-turn** swaps x and z and flips exactly one of them.
- **The gear ratio does not change.** A cog pair across a seam still reverses, and a large-to-small
  pair is still 2:1. Only the frame of the stored number changes.

**Axes and faces.**
- The folded access already turns block states (`state.rotate`).
- Every *direction* that Create hands from one block entity to another must be turned the same way:
  a pipe connection's side, a pump's front, a gearbox's source face.

**Unit test.** `SeamTurnsTest` checks:
- the table, against the turned unit vectors;
- that the signs compose like the turns;
- on a half-turn seam, Guyou, Peirce, a quarter-turn corner and the cube cross, sampling blocks along
  every edge (corners and turn points included), that a link to a neighbour across the seam is
  accepted from both ends or refused from both;
- that an accepted link's two turns cancel, so a speed sent across and back returns with its own sign
  and Create's propagation cannot flicker;
- that refusals stay under a tenth of the samples.

**Turn points and cube corners.** Three rules keep the adapter's view the same from both ends of a
connection:
- **A block is never its own neighbour.** At a turn point a block can be its own neighbour across the
  seam: it sees itself, turned. No rotation agrees with itself through a quarter-turn, so the adapter
  never connects a block to itself.
- **Neighbours in stored space connect only directly.** Two blocks that are neighbours in stored
  space connect only as such, even where one also sees the other through the seam.
- **A seam link must hold from both ends** (`SeamTurns.seamLink`). Next to a cube corner (a 270° cone
  point) a diagonal seen through the seam is not always seen back that way. The unit test found this
  on the cube cross at (128, 0). Such a link is refused from both ends.

### 11.2 Kinetics [R]

**How Create connects** [C]:
- `RotationPropagator` walks from a block entity to the six faces (and diagonals, for cogs) by
  `pos.relative(face)`.
- `getRotationSpeedModifier(from, to)` decides the connection and the ratio from the two block states
  and `diff = to.getBlockPos() − from.getBlockPos()`.
- `setSource(neighbour.getBlockPos())` stores the source's absolute position.

**What folded access does on its own.**
- The walk reaches the band position, and the Level answers with the far block entity. That block
  entity's `getBlockPos()` is its stored position, up to the world's width away. Its state is in its
  own frame.
- Usually Create then sees no connection.
- Worse, an axis connection only checks that `diff` lies along one axis, never its length. Take two
  parallel x shafts on either side of a north–south seam whose stored positions line up in x: Create
  would call them "connected". So the adapter must never use Create's answer for a pair that is not
  adjacent in stored space.

**The adapter** (`KineticSeams`, `RotationPropagatorSeamMixin`):
- A `@WrapMethod` on `getRotationSpeedModifier` keeps Create's answer for a pair that is adjacent in
  stored space.
- Otherwise it looks for `to`'s band image next to `from` (`SeamFrames.link`, by `images`). The link
  must hold from both ends (§11.1).
- If there is one, it runs Create's method again with `to` as `from` sees it:
  - `to.getBlockPos()` is answered with the band position;
  - `to.getBlockState()` is answered with the state turned back;
  - `getAxisModifier(to, face)` gets the face turned into `to`'s frame.
- The ratio that comes out is in `from`'s frame. It is multiplied by the axis sign (§11.1) of the axis
  `to` is seen with.
- So all of Create's own rules apply unchanged: cogs, large cogs, chain drives, and the modifiers of
  gearboxes and split shafts. So does any addon cog that implements `ICogWheel`.

**A source across the seam.**
- Create stores the source's stored position.
- Every comparison Create makes with it is between stored positions (`propagateMissingSource`,
  `handleRemoved`, `validateKinetics`), so it stays consistent.
- Only `DirectionalShaftHalvesBlockEntity.getSourceFacing` derives a face from it. That covers
  gearboxes, clutches, gearshifts and sequenced gearshifts. A mixin answers it with the face the
  source is seen through.

**Custom connections are refused across a seam.**
- These are the `propagateRotationTo` overrides: belts, chain conveyors, gantry shafts and carriages.
- Their partners are found through stored absolute positions or offsets, which a turn does not carry.
- The connection is not made, and the server logs it.

**Evidence** (`./gametest.sh -PwithMic -PwrapEngine` on the test VM, 2026-10-04, on Phase 1's
engine).
- **Method.** Each build is placed with the same code on a glued site and on a control site with no
  seam, then compared block by block.
- **`createKineticsAcrossHalfTurn`** (site cx = 24576, z0 = 20480):
  - A z shaft line, driven by a creative motor at 16 rpm, runs 5 blocks to the seam and 3 beyond.
  - The far shafts are stored at (24574, 20480…20482) with axis z and speed **+16**. The control's
    speed is −16.
  - A vertical cog line keeps its sign: far −16 / +16, control −16 / +16.
  - An x cog line flips: far +16 / −16, control −16 / +16.
  - Breaking the shaft next to the motor stops all 7 remaining shafts, the far 3 included.
- **`createKineticsAcrossQuarterTurn`** (corner x0 = 25600, z0 = 22528):
  - A z shaft line that crosses the quarter-turn edge lands as an **x** line, stored at
    (25600…25602, 22534). Its speed is +16; the control's is −16.
  - A vertical cog line keeps its sign.
- **All assertions held.**
- **Control run without the adapters** (`-PwrapNoCompat`, same build). The far shafts stand still
  (0 rpm against the control's −16), so the adapters are what carry the rotation across.

### 11.3 Fluids [R]

**How Create moves fluid** [C]:
- Three walks start from one block and move by `pos.relative(face)`, in that block's frame:
  - `FluidPropagator.propagateChangedPipe`, which finds pumps;
  - `PumpBlockEntity.distributePressureTo`, the pump's pressure search;
  - `FluidNetwork`, the transfer network.
- Under folded access each walk continues straight through a seam in unfolded coordinates. Its reads
  and states stay consistent there.
- What breaks is every place where a *face* is handed to a block entity reached through a band
  position. Pipe connections are keyed by face, in the owner's stored frame.

**The adapter** turns that face into the target's frame (`SeamFrames.toStoredFrame`) at each such place:

| Where | What is turned |
|---|---|
| `FluidPropagator.propagateChangedPipe` | the side a pump found across the seam is told to update |
| `FluidPropagator.resetAffectedFluidNetworks` | the connection looked up on a pipe across the seam |
| `FluidPropagator.hasFluidCapability` | the capability side (sided handlers) |
| `PumpBlockEntity.distributePressureTo` | the side pressure is added to |
| `PumpBlockEntity.hasReachedValidEndpoint` | another pump's front; a fluid handler's side |
| `FlowSource.OtherPipe.provideFluid` | the neighbour's outward face |
| `FlowSource.FluidHandler` (its capability cache, `lambda$manageSource$2` in 6.0.10) | the handler's side |
| `FluidNetwork.get` | the connection's face |
| `PipeConnection.determineSource` | called with the owner's band position, which is folded to its stored position first |

**Evidence: `createFluidsAcrossHalfTurn`.**
- The build, in order: a creative tank of water, a pipe, a mechanical pump pushing north, 4 pipes to
  the seam, 2 pipes beyond it, and a fluid tank.
  - A cog and a creative motor drive the pump.
  - The fluid tank is set at a band offset, so it is stored on the far side, at (cx−3, z0+2).
- After 120 ticks the far tank, stored at (25085, 20482), holds **112 mB** of water. The control tank
  holds the same, 112 mB. (On the spike, before Phase 1, both held 104 mB.)
- Without the adapters (`-PwrapNoCompat`) the far tank stays empty: 0 mB after 800 ticks.
- The pipes are Create's ordinary pipe block. Its connections are computed from neighbours read
  through the seam.

### 11.4 Trains

**Assessment** [C, from Create's `content/trains`, read by an Opus sub-agent with paths cited]:

**The graph is absolute.**
- `TrackNodeLocation` is absolute half-block coordinates plus a dimension. `TrackGraph` keys its nodes
  by it.
- A straight `TrackEdge` has no geometry of its own: its length, positions and direction come from its
  two nodes.
- A `BezierConnection` holds absolute positions.
- Edge points (signals, stations) store a node pair and a distance along the edge.
- So an edge whose ends lie on opposite sides of a turning seam would be a chord through the world.
  `canTravelTo` requires a dot product above 7/8, so it would fail at both ends.

**Folded access alone builds a second graph.**
- `ITrackBlock.walkConnectedTracks` reads band blocks, and `TrackPropagator` carries the graph on in
  band coordinates.
- The far track's own propagation builds the mirror copy. The same track then sits in two graphs that
  never merge.
- A train on the band copy is re-aligned from the graph every tick (`Carriage.manageEntities` →
  `alignEntity`), so the engine's carry and the graph would fight.
- That train would not see the far side's stations and signals.

**The precedent is Create's portal tracks.**
- `PortalTrackProvider` maps `findExit(ServerLevel, BlockFace)` to `Exit(ServerLevel, BlockFace)`,
  keyed by portal block. It joins two track ends by a zero-length "inter-dimensional" edge that skips
  the direction check.
- Each side is aligned on its own, and exits facing another way are already handled.
- A carriage straddling a portal has one entity per dimension (`Carriage.entities`, keyed by
  dimension), with cut-off planes and passenger hand-over.
- A turning seam has exactly the portal shape: the inbound end points into the seam, the outbound end
  out of it.
- What blocks a same-dimension portal is that Create treats "same dimension" as "same frame"
  everywhere:
  - `interDimensional` is computed from the dimension alone, on the client too;
  - there is one carriage entity per dimension;
  - about 23 `dimension.equals` checks remain in `content/trains`;
  - `forceNode` keys on the dimension;
  - passenger transfer assumes a change of dimension.

**Estimate: 13–22 agent days for a half-turn**, by the portal route:

| Work | Days |
|---|---|
| Walk filter and band guard | 0.5–1 |
| Binding seam ends and forcing nodes | 1.5–3 |
| Generalising "dimension" to "frame", with client mixins (the client builds track edges itself) | 5–8 |
| Carry exclusion, turned passengers, cut-off and ghosts, chunk loading on both sides | 2–3 |
| Signals, stations, relocation and navigation | 2–3 |
| Tests | 2–4 |

- A quarter-turn adds 2–4 days, and track must be forbidden near turn points (`findPivot` assumes a
  carriage spans at most two frames).
- A native split edge without the portal machinery would take 20–35 days.
- The doc's earlier 10–20 days roughly holds, raised at the low end.

**Prototype: not built.** It does not fit beside Phase 1:
- it needs a client part;
- it needs Create's dimension handling generalised, which should start from Phase 1's final topology
  API.

**Refused now** (both refusals are tested):
- **The track tool** (`TrackPlacementSeamMixin`) refuses a curve or straight run whose box comes within
  a block of a seam. It does so before anything is placed or paid for. Create's own path shows the
  refusal on the action bar with the deny sound (`create.track.deeptime_seam`).
- **Any track block** (`TrackBlockSeamMixin`) within a block of a seam is broken and dropped one tick
  after it is placed, before it joins the graph. That covers track placed by hand, by schematic or by
  the tool.

### 11.5 Contraptions

**Assessment** [C, same sub-agent]:

**Assembly** (`Contraption.searchMovedStructure`, `moveBlock`) reads through the Level, so a structure
reaching beyond a seam would capture the far-side states turned correctly. Three things would still be
wrong:
- captured block-entity NBT is not turned;
- multiblock controllers are subtracted from absolute stored positions;
- glue in the band is not found.

**Disassembly** writes back through `setBlock`, which is folded, but it turns block entities only by
the contraption's own rotation.

**The entities** follow something other than their own position, so the engine's carry would fight
them. (The carry is a `moveTo` plus a yaw turn when an entity's position passes an edge.)
- `ControlledContraptionEntity` (bearings, pistons, pulleys, gantries) follows its anchor. Pistons and
  gantries reset their position every tick.
- `OrientedContraptionEntity` (minecarts, trains) keeps its own yaw and follows its cart or its train's
  graph.

**Collisions** (`ContraptionCollider`) read world blocks through the Level, so they are folded. But
entities on the far side sit at their stored coordinates, so a structure reaching into the band passes
through them.

**Estimate: about 11–19 days.**

| Work | Days |
|---|---|
| Refusals and carry exclusion | 1–2 |
| Structures that reach across while their anchor stays stored: turned block-entity NBT, controllers, glue, ghosts, far-side collisions | 6–10 |
| Minecart contraptions crossing: carried as a unit, yaw and targetYaw turned, couplings by nearest image | 4–7 |

**Done now:**
- **Assembly is refused.** `ContraptionSeamMixin` throws Create's `AssemblyException` with
  `deeptime.wrap.create.refused.contraption` when any captured block lies beyond a seam.
  - The message shows on the controller's goggle tooltip.
  - Every contraption kind assembles through this method, trains included.
- **Contraption entities are left out of the carry.** `CreateWrapCompat` cancels the engine's
  `WrapCrossingEvent` (§9.5) for contraption entities and any vehicle carrying one, so the engine
  leaves them where they are.

**Not refused yet: movement into the band** by an assembled piston or gantry. Its collider sees the far
side's blocks, but empty band space lets it in. The hook is `ContraptionCollider.isCollidingWithWorld`
[C, not built].

### 11.6 Other refusals

- **Belts.** `BeltConnectorItem.canConnect` refuses a belt whose span touches a band position. Belt
  segments find their controller by absolute position.
- **Chain conveyors.** `ChainConveyorBlockEntity.addConnectionTo` refuses one whose span touches a band
  position. Their connections are stored as offsets.
- **How a refusal is reported.** Each refusal:
  - logs one line per place and kind;
  - shows its reason on the action bar of players within 16 blocks (`deeptime.wrap.create.refused.*`).

  Nothing throws, apart from Create's own assembly exception.

### 11.7 What waits for later phases, and why

**The API.** Every adapter reaches the engine through `SeamFrames`, which uses only §9's calls:
`fold`, `canonical`, `turn`, `rotation`, `topology` and `WrapTopology.images`. Phase 1's guarantees
(§9.4) are exactly what the adapters assume:
- a block entity reached through a band position is the stored one, so canonical positions are
  compared;
- a direction followed into a neighbour turns by `turn(f.iso(), d)`.

**Rotating shafts and pipe flow in the band, on the client** (Phase 2, §10.4: block-entity data in the
band).
- The band holds turned block states and no block entities. A client that looks at the band therefore
  sees no turning shaft, no pipe flow and no goggle data across the seam.
- For Create, the band's turned block-entity tag needs three changes:
  - `Speed` multiplied by the axis sign (§11.1);
  - `Source` moved to its band image;
  - pipe connections re-keyed by the turned face.
- The rule for turning the tag is here; sending band block entities is the engine's.

**Loaded band chunks.**
- Create checks `level.isLoaded(neighbour)` before it steps to a neighbour, so a network reaches across
  a seam only while the band chunk there is loaded.
- Phase 1's band manager holds a loaded band chunk's source loaded (§9.4, "Not folded"). The other
  direction needs a test in play: a stored chunk next to a seam, loaded while its band chunk is not
  (§10.4). The tests forceload their boxes.

**Depth of the walk.**
- Pipe searches and pump pressure walk on through the band in unfolded coordinates.
- On a bounded band (the test sites, depth 16), a far-side run longer than the depth is not reached.
- On a whole-world topology (Phase 2) the band is as deep as the layout's depth parameter.

**Cost.**
- The kinetic adapter calls `images`, twice (the link and its way back), only near an edge, and only
  for a pair that is not adjacent in stored space.
- Propagation runs only when blocks change, not every tick.

**MIC with the engine** (found here; not caused by the adapters). With `-PwithMic -PwrapEngine`, 6 of
Phase 1's own tests fail. Without the adapters (`-PwrapNoCompat`) the same 6 fail, together with the 4
Create tests, which then have nothing to carry them:
- **`redstoneAcross{Half,Quarter}Turn`:** the wire placed beyond the seam reads 0 after 13. Both pass
  without MIC. The likely cause is MIC's Alternate Current 1.9.0, which replaces redstone wire
  propagation with its own level access [I, not read]. Its access must be folded.
- **`playerCrosses{Half,Quarter}Turn`:** "Payload kubejs:sync_server_data may not be sent to the
  client!" KubeJS sends a payload to the test's fake player.
- **`bandReload{Half,Quarter}Turn`:** timed out waiting for a band chunk to be copied from its source
  under the MIC set.

**An intermittent boot failure with the engine and MIC.**
- The failure: Registrate reports "Found unused register callbacks" for Create during registration,
  then NeoForge rolls back and crashes on an unbound attribute.
- How often, over 28 boots with the engine on:
  - with the adapters: 4 of 16, all 4 before the merge onto Phase 1 and none of the 8 after it;
  - without them: 1 of 12;
  - never in the plain `-PwithMic` runs seen.
- **The fix already in.** The adapters wire themselves at server start, so they no longer load any
  Create class during mod construction.
- **What remains.** The failure still occurs, rarely, with the adapters left out. It looks like a race
  between parallel mod construction and the engine's or the adapters' mixin weaving [I]. It needs a
  closer look with `-PgametestDebugLog` (Registrate logs the unused callbacks only at debug level).

### 11.8 Running the tests

- **`./gametest.sh -PwithMic -PwrapEngine`**, through `tools/remote/mac.sh`, runs the engine's tests and
  the four Create tests (Create is in the MIC set):
  - `createKineticsAcrossHalfTurn`;
  - `createKineticsAcrossQuarterTurn`;
  - `createFluidsAcrossHalfTurn`;
  - `createRefusalsAtSeam`: a belt, a bearing and a track, each with a control on this side that still
    works.
- **Plain `./gametest.sh -PwrapEngine`:** the Create tests log that they were skipped, and pass.
- **Switches:**
  - `-PwrapNoCompat` runs the engine without the adapters (the control above);
  - `-PwrapCompatSkip=<Mixin>,…` leaves single adapter mixins out.
- **Sites.** The tests use sites 8–11 of the engine tests' grid (x = 20480 + 512 i), with controls at
  z 24576 and 26624. Their glued sites join the engine tests' topology through
  `CreateSeamGameTests.topologies()`.

## 12. The cube (2026-10-22)

Phase 6 of [`cube-world-plan.md`](../plans/cube-world-plan.md) (§5, E1 to E8): the engine joins the
turned conformal cube's seams. Ben scheduled it on 2026-10-22 (Q3, "the wrap engine now"), in
parallel with V1. This section makes §10.4 and the plan's E1 to E8 concrete. It was written before
the code (branch `worktree-agent-ab9924d9a8a834409`); §12.9 says what was built.

### 12.1 Which seams turn, and by how much

The net is phase 0's fold W2 (`worlddata/CubeLayout`, pinned against Rust by the parity fixture):
face side s (a multiple of 256 in a real world, SCL-100), h = s/2, the strip F3, F0, F1, F2 at
z ∈ [−h, h), N above F0 and S below it, period 4s east to west. Its 14 net edges form 7 seams:

| Seam | Net edges | Motion (beyond → planet) | Turn | Pivot |
|---|---|---|---|---|
| east_west | F3-west, F2-east | translation by ±4s | none | none |
| ne | N-east, F1-top | quarter turn | 90° | (h, −h), F0's north-east corner |
| nw | N-west, F3-top | quarter turn | 90° | (−h, −h) |
| se | S-east, F1-bottom | quarter turn | 90° | (h, h) |
| sw | S-west, F3-bottom | quarter turn | 90° | (−h, h) |
| nn | N-top, F2-top | half turn | 180° | (s, −s) |
| ss | S-bottom, F2-bottom | half turn | 180° | (s, s) |

- **Toroidal World keeps the slide** (Ben, 2026-10-03, "TW slides, engine turns"). The engine joins
  the 12 turning edges (`owner` = `wrap_engine` in the manifest).
- **Every motion keeps blocks on blocks, and chunks on chunks.** Each is a grid isometry (k quarter
  turns and a translation). With s a multiple of 256 every pivot sits on a chunk corner, so a band
  chunk shows exactly one stored chunk (except the split chunks of §12.2).
- **The metric across a seam is continuous.** The conformal cube maps two faces onto the sphere so
  that they agree on their shared edge, and the cube's reflection symmetry makes the map's
  derivative match across it: measured, two faces' points on a shared edge are 2.8e-16 rad apart and
  the derivative jumps by at most 0.0034 (`docs/science/seamless-world.md`, "The cube world"). The
  gluing itself is a grid isometry, so a block across a seam is an ordinary neighbour of the same
  size (SCL-184).
- **The planet's rotation and north bearing do not enter the engine.** They are the world's map and
  G9's business. The engine only moves blocks, block states and entities by grid motions.

### 12.2 One geometry, and the topology built from it

- **One geometry in Java.** `WrapLayouts.cube(CubeLayout, depth, slides)` builds the engine's edges
  from `CubeLayout.edges()` and its corners from `CubeLayout.corners()`. The old
  `WrapLayouts.cubeCross` (a shifted copy in the window [0, 4s) with F3 east) is gone;
  `cubeCross(s, depth, slides)` is now a one-line alias of `cube(CubeLayout.of(s), depth, slides)`.
  There is no second copy of the net to drift.
- **From the manifest (phase 2).** Format 0.7 layout_version 2's manifest carries the same table
  (FORMAT.md, "Projection", `edges[]` and `cone_points[]`). Phase 2's reader checks the manifest's
  edges and cone points against `CubeLayout.of(face_blocks)` (name, axis, side, coord, span, iso)
  and refuses a world whose table differs; it then calls `WrapLayouts.cube(CubeLayout.of(face_blocks),
  depth, slides)`. The two sources therefore cannot disagree in play, and the engine needs no second
  parser of the table. The general constructor `WrapTopology(name, edges, corners, periodX,
  windowLo)` takes any edge and corner list, for another layout or a test.
- **The fold rule: the nearest edge.** Phase 1's topologies have bands that never overlap, so it
  folded by the first edge whose band holds a block. The cube's bands overlap in the void squares by
  F0's corners, and there the first edge is wrong: a player walking north off F1 ten blocks from
  F0's corner would be carried back onto F1 by N-east instead of onto N. The engine now uses
  `CubeLayout.fold`'s rule (SCL-108): the edge the block is least deep beyond, ties to the earlier
  edge in the list, so each square splits on its diagonal through the corner. Points (entities) take
  the edge their block takes. For the old layouts nothing changes (no block lies in two bands).
- **Split chunks.** A chunk on such a diagonal folds by two motions. The topology finds them when
  asked: `chunkFold` folds the chunk's four corners, and the chunk is split when they fold by
  different motions (a diagonal is a half-plane boundary, and a half-plane that cuts a square
  separates two of its corners). For them `chunkSources(cx, cz)` lists every (source chunk, motion). The band manager holds a ticket on every
  source of a split chunk and copies it column by column; replication writes a stored change into a
  split chunk only where the fold of the band position is the stored position. `chunkFold` keeps
  answering for the chunk's corner and gains `split()`, so code that uses it as a band test is
  unchanged.
- **Band depth.** D is a multiple of 16 and at most s/2 (so only adjacent bands meet). Production
  default 32 blocks: two chunks, what light and collisions need (§4.2; SCL-185, game-choice);
  `-Ddeeptime.wrapengine.depth` overrides it. GameTests use 16 at s = 256.

### 12.3 Composition with Toroidal World's slide

- **TW's frame.** TW's canonical window is x ∈ [−2s, 2s) (C = 4s), so its seam cuts F2 in half at
  x = 2s: F2's east half lives at x ∈ [−2s, −3s/2). Folded positions the engine hands back must be
  canonical, or TW and the engine would store one block twice.
- **The cut.** `cube` cuts the two edges whose bands straddle the window's end and the two whose
  destinations do:
  - F2-top and F2-bottom are cut at x = 2s; the east piece moves to [−2s, −3s/2) with its motion
    composed with the translation 4s (`F2-top@tw`, `F2-bottom@tw`);
  - N-top and S-bottom are cut at x = 0 by destination: the half with x < 0 lands in F2's east
    half, so its motion is composed with the translation −4s (`N-top@tw`, `S-bottom@tw`).

  Pieces keep their parent's place in the edge order, so ties break as in `CubeLayout`.
- **`slides = false` (production, TW on the level).** The topology has period 4s and window lo −2s.
  A block outside the window (a mod asking before TW aliases it) is reduced into the window before
  the band test, and the translation is composed into the motion; a block in no band is left to TW.
  Band chunks exist only inside the window (TW aliases the rest).
- **`slides = true` (GameTests, and a dev world without TW's loop).** The engine adds two slide
  edges at x = ±2s over F2's rows and their bands (z ∈ [−h − D, h + D)), D deep. A block beyond the
  slide and beyond F2-top slides first and then turns (two steps of the fold).
- **The install decision.** `WrapEngine` asks `ToroidalWorldApi.shapeOf(overworld)` when the server
  has started and hands the shape to `CubeInstall.decide`: a level that loops x with width 4s and
  minimum −2s takes `slides = false` (`TURNS_ONLY`); a level that does not loop takes
  `slides = true` (`WITH_SLIDES`); any other shape (another width, a z loop) is refused (`REFUSE`)
  with an error in the log, and nothing is installed. The decision is a pure function, unit-tested.

### 12.4 The corners

- **Eight cone points of 270°.** Three faces meet at each cube corner, so the angle round it is 270°
  and the holonomy of a loop round it is a quarter turn (SCL-182). No local gluing can make it flat:
  near a corner the band shows one face twice or not at all, whatever the engine does.
- **Images in TW's frame.** n0 (h, −h), n3 (−h, −h), s0 (h, h), s3 (−h, h) once each; n1 (h, −3h)
  and (3h, −h); n2 (−h, −3h) and (−3h, −h); s1 and s2 likewise (CubeLayout's third image at x = 5h is
  −3h in TW's frame).
- **Seam links are refused near them.** `WrapTopology.nearCorner(x, z)` is true within a keep-out of
  K = 2 blocks of any image (the 4 × 4 blocks round it; SCL-183, game-choice: the degenerate
  neighbours are the corner's 2 × 2 blocks and their diagonals). `SeamTurns.seamLink` refuses a link
  when either end, or either image it found, is near a corner, so Create's networks, belts and
  track are refused from both ends alike. Phase 1's refusal (turns that do not cancel) stays as the
  general rule; the keep-out makes it exact at the cube's corners. E4 reuses the same test for
  leashes and ghosts.
- **The convex corners of the net.** At N's and S's outer corners, (±h, −3h) and (±h, 3h), the
  quadrant beyond both edges is in no band of `CubeLayout`, so a diagonal step off N's corner block
  would land in ocean with nothing to carry it back. The engine extends N-east and N-west (S-east,
  S-west) by D past the outer corner and N-top (S-bottom) by D each side; the nearest-edge rule then
  splits the quadrant on its diagonal, and it shows F1 and F2 (or F2 and F3) by the corner's other
  image. `CubeLayout.fold` still answers ocean there (the generator's fill, G5); the band copy
  overwrites that fill within D. Equivalence tests against `CubeLayout.fold` leave these quadrants
  out (SCL-181).
- **The concave corners** (F0's four, and n1, n2, s1, s2's second images on the strip's top and
  bottom lines) are covered twice: two bands meet there. That is the cone point showing one face
  twice; entities follow the nearest edge, and links are refused.

### 12.5 What the border W3 becomes as seams join

- **Until the engine is on, W3 stops everything at 12 border segments** (the turning edges, inset,
  with a warning band; plan §1.3). It ships with phase 2.
- **With the engine on, a joined edge has no border.** `WrapEngine.joinedEdges(level)` names the
  edges of the installed topology (the base names, `@tw` stripped). W3 leaves those segments out:
  no stop, no warning band, no push-back. Installing the cube joins all 12 at once, so on an engine
  world W3 is empty.
- **W4 stays.** An arrival outside the planet (portal, `/tp`, ender pearl) further out than the band
  is folded onto its partner block by W4; one inside the band is carried by the engine on the next
  tick.
- **No slip at the corners.** Entities are carried as soon as their position is beyond an edge, so
  they never go deeper than one tick's movement; the corner squares and the convex quadrants are all
  in bands (§12.4).

### 12.6 E1 to E8: order and gates

| Order | Item | What | Gate |
|---|---|---|---|
| 1 | E1 | The cube topology from the layout, split chunks, corners, TW's slide, the install behind the flag (this branch) | unit tests: the engine's fold equals `CubeLayout.fold` (mod 4s, same turn) in every band for s = 256, 1024, 3840, both modes; GameTests on a quarter seam, a half seam, the slide composed, a corner refusal |
| 2 | E2 | Block-entity data in the band, container range across a seam (this branch; §12.8) | GameTests: a band chunk's packet carries the turned block entities; a re-addressed data packet; changes reach the band's holder; a chest in range across a seam; the sign editor opens at the near image |
| 3 | E4 | Entities: leashes, ghosts, items and projectiles across turns, AI distances by nearest image | GameTests per entity kind at a quarter and a half seam, refused near a corner |
| 4 | E3 | Keeping the band current as players build, on a phase-2 world whose generator already fills the band (G5) | a baked cube world: copying a band chunk of untouched ground changes no block |
| 5 | E5 | The client re-anchor across a turn; G9's north bearing turns with it | a real-client walk on the VM (`vm-client.sh`) across a quarter and a half seam with no reload |
| 6 | E8 | The detail noise and fill: joining a seam changes no terrain | E3's check on every seam of a 1k and a 4k world |
| 7 | E6 | Pack compatibility: TW on the same level, Distant Horizons LODs of band chunks, Sable through `PhysicsPipeline.teleport`, Create trains | `./gametest.sh -PwithMic -PwrapEngine` and a planet server with the MIC set |
| 8 | E7 | Hardening: writes off the server thread, C2ME, save and reload, a busy multiplayer seam | a soak and the benchmark on the VM |

After E7 the default (on for cube worlds, or still dev-only) is Ben's decision.

### 12.7 What depends on phase 2, and how it is tested before

- **From phase 2:** the manifest reader (format 0.7 layout_version 2's `edges[]` and
  `cone_points[]`), cube worlds from the bake (phase 1), the generator's band fill (G5), the border
  W3. E1's install takes its layout from the manifest once the reader exists; until then only from
  `-Ddeeptime.wrapengine.cube=<s>` (dev).
- **Before phase 2:** everything is tested on synthetic topologies from phase 0's net
  (`CubeLayout.of(s)`): unit tests over whole bands, and GameTests on a cube of s = 256 moved to a
  far site of the test world (`WrapTopology.translated`), joined with the phase-1 sites.

### 12.8 Block-entity data in the band (E2)

What §4.2 promised, as it will be built:

- **Chunk packets.** When a band chunk's packet is built (`ClientboundLevelChunkPacketData`), the
  engine appends a block-entity entry for every band position of the chunk whose stored block has a
  block entity: the stored entity's update tag, turned (below), at the band position. The client then
  draws chests, signs and banners there as it does anywhere. Copies still hold no block entity on the
  server.
- **Updates.** A change at a stored position (`ServerChunkCache.blockChanged`, which a block entity's
  `sendBlockUpdated` reaches) is also marked at each of its images, so the band's holder sends it.
  When a holder sends a block entity for a band position (`ChunkHolder.broadcastBlockEntity`), the
  engine sends the stored entity's data addressed to the band position instead of vanilla's packet,
  which carried the stored position.
- **Turning a tag** (`BandTags`). Block states already turn. A short table of position-bearing keys
  says what else does: absolute positions move by the motion stored → band, offsets and direction
  indices turn by its turn. Vanilla needs two rows (an end gateway's `exit_portal`, a brushable
  block's `hit_direction`); Create's keys join in E6, and the table can become a data file then, as
  Toroidal World's position rows are.
- **Container range.** A menu's `stillValid` asks `Player.canInteractWithBlock` with the stored
  position, which can be the far side of the world. When vanilla's answer is no and a topology is
  installed, the engine answers yes if an image of the position is in range.
- **Sign editing.** `ServerPlayer.openTextEdit` sends the stored position; the engine sends the image
  nearest the player when the stored one is out of range, so the editor opens on the sign the player
  sees. The edit comes back addressed to the band position, which folded access resolves.
- **Not in E2:** block events (a chest lid opening in the band), and block-entity renderers that read
  neighbouring block entities through absolute positions in their tags (Create's, E6).

### 12.9 As built

E1 and E2 on branch `worktree-agent-ab9924d9a8a834409` (2026-10-22), still dev-only
(`-Ddeeptime.wrapengine=true`); master's Petroff-Guyou worlds are untouched (nothing installs
without `-Ddeeptime.wrapengine.cube=<s>`).

**E1, the cube topology.**

- `WrapLayouts.cube(CubeLayout, depth, slides)` as §12.2 and §12.3 describe, with the cut pieces
  named `<edge>@tw`; `cubeCross` stays as an alias. `WrapTopology` gained the nearest-edge fold,
  split chunks found from a chunk's corners (`ChunkFold.split()`, `chunkColumns`, `chunkSources`),
  `nearCorner` (keep-out `CORNER_KEEP_OUT` = 2), the period window, and `translated(dx, dz)` for
  test sites. The band manager copies a split chunk column by column from each source and holds a
  ticket on each.
- `CubeInstall` is pure (no Minecraft type, so the unit tests load it): the two properties, the
  default depth 32, and `decide(...)` returning `TURNS_ONLY`, `WITH_SLIDES` or `REFUSE` with a
  reason. The install itself (`WrapEngine.installCube`, on `ServerStartedEvent`) reads Toroidal
  World's shape and installs on the overworld.
- `WrapEngine.joinedEdges`, `nearCorner`, and the `SeamTurns.seamLink` refusal near a corner
  (§9.6, §12.4).

**E2, block entities in the band** (§12.8):

| Piece | What it does |
|---|---|
| `BandTags` | The position-bearing tag table (pure): `exit_portal` moves, `hit_direction` turns; unit-tested |
| `BandBlockEntities` | Which stored entities a band chunk shows, their turned tags, the re-addressed data packet, marking images, the nearest band image |
| `ClientboundLevelChunkPacketDataWrapMixin` | A band chunk's packet lists the stored entities it shows, at band positions |
| `ServerChunkCacheWrapMixin` | `blockChanged` at a stored position also marks its band images |
| `ChunkHolderWrapMixin` | A band holder sends the stored entity's data re-addressed instead of vanilla's packet |
| `BandManager.copy` | After a copy, every entity the chunk shows is marked (a reloaded band chunk that wrote no block still sends its entities' data) |
| `PlayerWrapMixin` | `canInteractWithBlock` is true when the nearest band image is in reach (menus' `stillValid`, the sign editor's keep-alive) |
| `ServerPlayerWrapMixin` | The sign editor and its block update go to the image nearest the player |
| `SignBlockEntityWrapMixin` | Front or back of a sign is judged from the player's position moved through that image (without it, a player at the image edits whichever side faces the far-away stored sign) |
| `ChunkHolderAccessor` | Test access to a holder's marks and its block-entity send |

The access transformer widens `ClientboundLevelChunkPacketData$BlockEntityInfo` and its
constructor, and `ClientboundBlockEntityDataPacket`'s constructor. These lines apply whether the
engine is on or not; they only widen access. Toroidal World's period copies never count as band
images: a block's own copy a period away is TW's business.

**GameTests** (namespace `deeptime_wrapengine`, `./gametest.sh -PwrapEngine`). The cube is
`cube(CubeLayout.of(256), 16, true)` moved to (49152, 20480) and joined with the phase-1 sites in
one topology. Each site's boxes hold its own blocks and the far side's, and no two sites share a
column.

| Test | Site | Checks |
|---|---|---|
| `cubeBuildingAcrossQuarterTurn` | F1-top by (h, −h) | phase 1's building test on a quarter turn of the cube |
| `cubeBuildingAcrossHalfTurn` | F2-top | the same on a half turn |
| `cubeBandConsistentQuarterTurn` | F1-top, farther along | random edits on both sides keep the band a turned copy |
| `cubeBandConsistentAtToroidalSeam` | N-top west of x = 0 | the same where the half turn has the period composed in |
| `cubeSlideComposedWithTurn` | F2's bottom-east corner | a block beyond the slide and the turn folds in two steps |
| `cubeCornerRefusal` | (h, h) and S's outer corner | the keep-out at every image; writes into the split corner square; stands carried off both quadrants |
| `blockEntitiesAcrossHalfTurn`, `blockEntitiesAcrossQuarterTurn`, `cubeBlockEntitiesAcrossQuarterTurn` | plain site 6; F3-top by (−h, −h) | E2's six checks (`WrapBlockEntityGameTests`) |

Results: see STATUS.md's build log for this branch.
