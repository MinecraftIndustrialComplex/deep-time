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
  - an armour stand and an item are carried across, turned.

  The client side was not run (§7.3).
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
| **Create**, kinetic networks | Shafts, cogs and belts read neighbours through `getBlockState`/`getBlockEntity`, so they would connect across a seam. Under a half-turn a shaft's axis direction flips, so its speed sign must flip, and Create's propagation uses position differences between frames [I]. | Block kinetic connections across seams at first (a refusal in `RotationPropagator`); an adapter later. |
| **Create**, contraptions | Carried as entities, yaw turned. Contraption entities hold their own blocks, so the band does not show them; ghosts do. | Phase 4. |
| **Create**, trains | The track graph is keyed by absolute positions and curved tracks store absolute positions; an edge across a seam would join stored positions up to the world's width apart. TW needed its own fold for slides [C]. | Forbid tracks across turning seams at first; a seam-split graph edge later (10–20 days, no precedent). |
| **Create**, fluids and pipes | Same as kinetics. | Same. |
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

RESULTS-PLACEHOLDER

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
