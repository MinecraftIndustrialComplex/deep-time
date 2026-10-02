# Rotated-seam worlds: has anyone already built a half-turn (quincuncial) planet?

Researched 2026-10-02 for Deep Time, after Ben's "Do a shit ton more research, I feel like someone has likely built
something along these lines already" (decisions log, 2026-10-02). It follows
[`world-projection-options.md`](world-projection-options.md) and §5.3 of
[`../../design/equal-area-projection.md`](../../design/equal-area-projection.md). It changes no code and decides nothing.

**The idea being checked.** A world engine that aliases chunks across a seam with a 180° turn, not only a slide, so
that one Peirce-style square (the "pillowcase", Conway's orbifold 2222, wallpaper group p2) is a single seamless
planet stored once.

**Tags.**
- **[V]** I read the named source myself this session (code, docs, API response).
- **[SA]** read by a Sonnet sub-agent, with the kind of reading it reported; **[SA✓]** means I re-checked that
  specific claim against the source.
- **[I]** my inference or derivation, reasoning given.
- **[M]** from memory, unchecked.
- **[U]** unverified: a snippet, a README claim nobody ran, or a gap.

**What went wrong in the research.** Four sub-agent sweeps were launched. Three reported (other games; academic and
numerical work; Minecraft technical pieces). The fourth, the broad sweep of Minecraft mods, plugins, datapacks and
Bedrock add-ons, **failed on a usage limit and returned nothing**. I redid it myself, more narrowly: about 30
Modrinth API queries, GitHub repository search, and six web searches. YouTube devlogs, Reddit threads, CurseForge-only
projects, PlanetMinecraft, MCPEDL and Discord-announced projects were **not** swept (§7).

---

## 0. The answer in plain words

- **No. Nobody has built this, in Minecraft or in any game I could find.** No mod, plugin, engine or published game
  walks a player across a seam that turns the world 180°, and none uses a quincuncial or orbifold layout for a
  walkable block world.
- **The closest thing is the mod we already depend on.** Toroidal World (TW) has the algebra for it in its source: a
  seam transform with a sign per axis, an orientation named `HALF_TURN`, and public API methods that report the turn.
  The engine refuses to build any shape that uses them. What exists is the vocabulary, not the machinery.
- **Outside games, the half-turn seam is old and solved.** Ocean models run a 180°-folded northern boundary in
  production code, and a 2003 graphics paper states the exact tiling rule. Those give the simulation side
  (the Rust core) a design to copy.
- **Immersive Portals can already rotate at a portal**, so a pillowcase world could be assembled from eight rotated
  portals. A maintained NeoForge 1.21.1 build exists. It cannot show Distant Horizons terrain through portals, and it
  crashes with Toroidal World.
- **So the engine would have to be built**, most cheaply as an extension of Toroidal World (MIT). The parts with no
  precedent anywhere are: rotating a chunk's contents on its way to the client, showing one stored chunk at two
  places on one client near the four turn points, and carrying Create trains across a turning seam.

---

## 1. The most relevant existing projects, ranked

| # | Project | What it gives us | Licence | State | How I know |
|---|---|---|---|---|---|
| 1 | **Toroidal World** (`anyttng/toroidal_world`, Modrinth `toroidal-world`) | The translation-only version of the whole engine, on our exact stack, with Create, Sable, Aeronautics, DH, C2ME and Sodium compat. Its core already has `SeamTransform(xSign, zSign, xShift, zShift)`, `FoldOrientation.HALF_TURN`, `DeckGroupFold`, and API methods `foldOriented`, `preservesHandedness`, `preservesLocalIndices`. None is reachable: `WorldFolds.verifyFoldable` refuses every non-translation shape. | MIT | Very active, one maintainer, pre-1.0; last push 2026-09-29 | [V] source read on branches `dev` and `mc/1.21`, GitHub and Modrinth APIs |
| 2 | **Immersive Portals – CE** (`UpperMoon0/ImmersivePortals`) and upstream Immersive Portals | Portals with an arbitrary rotation, already rendered and teleported through, and convertible to always-loaded "global" portals. A pillowcase could be assembled from commands. | Apache-2.0 | CE: v6.0.10 on 2026-10-01, three releases since 2026-08, 0 stars. Upstream NeoForge port: no release since 2025-06 | [V] repo metadata, README, `gradle.properties`, `build.gradle`; rotation and global-portal commands in `PortalCommand.java` |
| 3 | **Ocean-model "north fold"** (FMS/MOM6, NEMO, Oceananigans, CICE) | A production half-turn seam for grid simulations: exact index maps, sign flips for vectors, zero at the pivot, halo filling. The design for Deep Time's raster kernels. | FMS and MOM6 Apache-2.0; Oceananigans MIT; NEMO CeCILL; CICE own licence | Production code | [SA✓] I re-read FMS `get_fold_index_north` and its licence header; the rest [SA, code read] |
| 4 | **Praun & Hoppe 2003**, "Spherical parametrization and remeshing" | The exact tiling rule in print: "extend the grid by rotating it 180° around the midpoints of boundaries", and the result that separable symmetric filters keep working across it. | Paper; no code released [SA] | Published | [SA✓] quote re-read in the paper text |
| 5 | **HyperRogue** (`zenorogue/hyperrogue`) | The only game where a player walks on orbifold quotients, including cone points on edge centres. Hyperbolic tilings, not a block world. No named 2222 mode found. | GPL-2.0: ideas only, no code | Active (13.1f, 2025-10) | [SA✓] I re-read the `arbiquotient.cpp` option names and the changelog line |
| 6 | **Create `StructureTransform` and Sable `AssemblyTransform`** | The working pattern for rotating block states at runtime, with the known vanilla gap (bells) already patched, and a registry for blocks and block entities that store positions. | Create code MIT; Sable PolyForm Shield (source-available) | Shipping in MIC | [SA, code read] |
| 7 | **Civilization IV "Peirce Quincuncial Projection Map"** script (LPlate2, 2023) | The only quincuncial game map found. It does not solve the seam: it frames the map in impassable peaks. | n/a | Released | [V] resource page fetched |
| 8 | **`Mrcavas/planets`** | A Fabric 1.20.1 prototype of "spherical worlds" that aliases block regions between cube-face islands by redirecting `getBlockState`/`setBlockState`. Its link record has a `rotation` field that nothing uses. | AGPL-3.0: ideas only | 3 commits, 2024-06 to 2025-06, 2 stars | [V] source read |
| 9 | **Cube-planet hobby engines** (Blocky Planet, the Cemetech "Minecraft on a sphere" thread, Seed of Andromeda) | Rotated coordinate transitions at cube edges (90°), described or attempted. Seed of Andromeda's transition table is commented out; it rebuilds the voxel grid on a face change instead. | Seed of Andromeda MIT; others not open | Hobby or abandoned | Cemetech [V]; Blocky Planet and Seed of Andromeda [SA, blog and source read] |
| 10 | **Petroff 2021** code (Zenodo 4661351) | The equal-area square projection itself, closed form. | Archive CC-BY-4.0; `new_projection.py` CC0 [SA] | Published | [SA, code read] |

Not ranked because they add nothing new for a turning seam: Circumnavigate (Fabric, AGPL-3.0, translation only; I
grepped its 128 Java files for rotate, mirror and flip and found nothing relevant [V]), Looping World, Wrap Around
World, TFC Eratosthenes, Planet Border Teleport (all teleports; see `wrapped_planet_worlds.md` §1).

---

## 2. What exists, by family

### 2.1 Toroidal World: the algebra is there, the engine is not [V]

Read from a clone at `dev` (`6cbb46a`, 2026-09-29) and `mc/1.21` (`5ded155`, 2026-09-29), the branch MIC's build comes
from.

- **The transform.** `core/SeamTransform` is `(xSign, zSign, xShift, zShift)` with signs of ±1, composition, inverse
  and powers. `(−1, −1, 2a, 2b)` is a half-turn about the point (a, b).
- **The orientation.** `core/FoldOrientation` has four values: `IDENTITY`, `MIRROR_X`, `MIRROR_Z`, `HALF_TURN`. It
  can already turn a movement delta, an in-block offset and a block face (`applyToDelta`, `applyToBlockOffset`,
  `applyToFace`).
- **The public API anticipates it.** `docs/modding.md`: "`ToroidalShape` carries two flags that describe folds this
  mod does not yet hand out. … They are in the API so that code written now survives the day one lands", and
  "`preservesHandedness()` tells a half turn from a genuine mirror".
- **What the shape model can describe.** `core/FlatShape` holds per-axis bounds, a skew and an optional mirror. Its
  six identifications are RECTANGLE, CYLINDER, MOBIUS, KLEIN, TORUS and LATTICE_TORUS. The constructor rejects other
  combinations as "not one of the five flat identifications". **A half-turn gluing is not among them**: Möbius and
  Klein mirror, they do not turn.
- **What runs.** `WorldFolds.of` always returns the translation-only `WorldLoopTransformer`, after `verifyFoldable`
  refuses anything else ("The wrap engine has no fold for KLEIN yet: its seam reverses the local indices inside a
  chunk"). `DeckGroupFold` (883 lines on `mc/1.21`, 409 on `dev`) is referenced by no other main class on `mc/1.21`;
  only tests construct it.
- **Chunk contents are never touched.** The rewriter `PacketTranslator` registers for chunk packets is
  `chunkPosition`, which moves the packet's x and z [V]. Block states, heightmaps, light, biomes and block entities
  pass through unchanged, and block updates have their position rewritten but not their state [SA, code read].
- **The Sable path refuses any turn.** `compat/sable/SableRigidShift.requireTranslation` throws when the seat's
  orientation is not the identity.
- **Size of what a turn would touch.** On `mc/1.21`: 1,110 main Java files, 59,798 lines. Per-axis fold calls
  (`blockDomain`, `chunkDomain`, `foldBlock`, `foldChunk`, `foldCoord`, `nearestCoord`, `unwrapAround`): 287 call
  sites in 80 main files. Each assumes x and z fold independently, which a half-turn breaks.
- **One copy per client is a stated invariant.** `docs/modding.md`: "A client near a seam is sent one copy of a chunk
  and not the others". View distance is capped at half the world minus a margin
  (`WorldLoopBounds.Looped.maxViewDistance`). §3.3 explains why a half-turn world cannot keep this near its four turn
  points.
- **The author and the project.**
  - Licence MIT, copyright "Vitalik Yarina" [SA]; the GitHub account was renamed (`vitalikyarina` now 404s, the
    description still links to it) [V].
  - 1,045 downloads, 18 followers, a Discord link on the Modrinth page [V, Modrinth API].
  - Issues and pull requests are numbered up to #54. Every pull request is the author's own; no outside
    contribution has ever been opened, so there is no evidence either way on whether one would be accepted [V,
    GitHub API].
  - The maintainer answers issues within days and fixes compat reports quickly (#22, #41, #51) [V].
  - No roadmap, no CONTRIBUTING file, no discussion of rotations in any issue or doc I read [V]. The only signs of
    intent are the unused classes and the API sentence quoted above. [I] The author plans mirror shapes; nothing
    says a half-turn is planned.

### 2.2 Immersive Portals: rotation exists, as a portal [V, SA]

- **Upstream** stores a rotation quaternion on every portal (`Portal.rotation`) and renders the far side from a
  rotated camera. It does not transform chunks [SA, code read].
- **Commands exist** to set a portal's rotation and to turn any portal into a global one:
  `set_portal_rotation`, `rotate_portal_rotation`, `convert_normal_portal_to_global_portal`
  (`PortalCommand.java` lines 341, 498, 514, at upstream commit `aede93a`) [V].
- [I] So a pillowcase is buildable in principle from eight same-dimension portals, one per half-edge of the square,
  each turned 180° about the vertical. Nobody appears to have done it. Its built-in wrapping zone is translation
  only (`wrapped_planet_worlds.md` §2.2).
- **A maintained 1.21.1 NeoForge build now exists**, which the earlier note did not know: Immersive Portals – CE.
  - Created 2026-03-21; releases v6.0.8 (2026-08-29), v6.0.9 (2026-09-15), v6.0.10 (2026-10-01) [V].
  - Its README claims Sable 2.0.5 integration with automated tests for "rotated portals" [V README; **[U]** nobody
    here ran it].
  - It compiles against Sodium 0.5.11 and runs its checks with Sodium 0.8.12 [V `gradle.properties`,
    `build.gradle`]. MIC ships 0.8.13.
- **What it cannot do.**
  - Distant Horizons terrain does not render through portals. Three merge requests adding it to DH were closed
    unmerged [SA, issue read].
  - Toroidal World and Immersive Portals crash together (TW #42, open, no reply) [V].
  - Upstream's `ChunkMap` overwrite conflicts with C2ME-style mixins and Petrolpark (`wrapped_planet_worlds.md`
    §2.3). Whether CE fixed that: **[U]**.

### 2.3 Other Minecraft work [V unless tagged]

My own sweep, narrower than planned (see the note at the top).

- **Nothing on Modrinth rotates at a seam.** Queries for spherical, round, planet, cube planet, world wrap, torus,
  Möbius, Klein bottle, non-Euclidean, looping, globe, curvature, gravity, finite world, border teleport, seam,
  projection and others returned only:
  - the known wrap mods;
  - `round-world`, a shader that bends the picture;
  - Gravity Changer and its forks (MIT), which change which way is down;
  - an Immersive Portals build bundled with gravity changing (Apache-2.0, 2026-06);
  - `planetoid` (LGPL-3.0, 2022), which generates floating spheres;
  - Big Globe, which is a terrain generator;
  - two modpacks that put Immersive Portals to "non-Euclidean" use.
- **GitHub repository search** found `Mrcavas/planets` (row 8 above) and nothing else relevant. That search is weak;
  most queries returned nothing.
- **Web searches** for Möbius or Klein-bottle Minecraft worlds, cube planets with gravity change, sphere mods and
  pole crossings with a turn found no mod. A Minecraft feedback post proposing a cubic world with a 90° gravity shift
  exists [SA, snippet].
- **Build The Earth** uses a modified Dymaxion ("Airocean") projection and leaves its cuts in the ocean as plain
  discontinuities [U, search snippet]. Its licence and projection list were not read.
- **Bedrock**: one search, only a world-border add-on [U].
- **Licence correction to the earlier note.** `wrapped_planet_worlds.md` §1.3 says Circumnavigate's AGPL "matches
  Deep Time's". It does not: Deep Time is LGPL-3.0-or-later, and AGPL code cannot be copied into it.

### 2.4 Other games and engines [SA unless tagged]

- **True 3-D voxel planets avoid the problem.** Space Engineers stores planets in a Cartesian voxel octree and uses
  a cube map only to generate terrain by direction [SA, source read]. No Man's Sky and Dual Universe are believed to
  work the same way [M].
- **Cube-sphere block worlds turn 90° at edges, or dodge it.**
  - Seed of Andromeda (MIT) has a face-transition table with quarter and half turns, all commented out; it deletes
    and rebuilds the voxel grid when the player changes face [SA, source read].
  - Blocky Planet (Unity, 2025, closed source) swaps or flips axes at the 12 edges [SA, dev blog read].
  - A Cemetech hobby project "duplicates known faces into the space" with rotation as the player crosses [V, forum
    thread].
- **Latitude bands.** Dyson Sphere Program shortens its build grid row by row toward the poles, leaving visible
  "fault lines" [SA, mod source and forum].
- **Cylinders with dead poles.** Empyrion, Starbound, Civilization [SA, forum and wiki].
- **Tori presented as globes.** Classic JRPG overworlds [SA, snippet]. Eco was not verified [U].
- **Red Blob Games, "Wraparound square tile maps on a sphere" (2019)** considered Peirce, Guyou and Adams, wrote
  "I've not evaluated these yet", and built a cube instead [V].
- **A Factorio forum post (mrvn, 2016)** proposed crossing a pole to land half a world away "but the tile is 180°
  rotated". Replies called it confusing; no developer answered [V].
- **Non-Euclidean games.** HyperRogue has torus, Klein bottle, Möbius band and projective plane modes, and since
  13.1f a tool that builds orbifold quotients of loaded tilings, with options to block cone points and mirrors
  [SA✓]. Whether it can produce a square-tiling 2222: [U]. Manifold Garden is a 3-torus drawn by instancing copies
  [SA, blog]. Jeff Weeks' Torus Games cover torus and Klein bottle only [SA; partly M].

### 2.5 Academic and numerical work [SA unless tagged]

- **The ocean "north fold".** Tripolar ocean grids glue the top row to itself by `i → N + 1 − i`, which is a
  half-turn about a pivot on the row.
  - FMS `get_fold_index_north`, cell centres: `i → isg + ieg − i`, `j → 2·jeg − j + 1` [SA✓, I re-read lines
    4956–4968 of `mpp_domains_define.inc`].
  - Both velocity components change sign across the fold; scalars do not. Vectors at the pivot are set to zero
    (FMS), or the two sides are averaged (CICE) [SA, code read].
  - The pivot can sit on a cell centre or a cell corner, and each code supports both variants [SA, code read].
  - Oceananigans (MIT) has the shortest readable implementation [SA, code read].
- **Praun & Hoppe 2003** (row 4): the tiling rule, plus the report that an image wavelet coder needed about 100
  lines changed to use it [SA].
- **Orbifold Tutte Embeddings** (Aigerman & Lipman 2015): maps a sphere-like mesh onto the four-cone-point
  pillowcase. MATLAB, "academic use only", no licence file [SA, code read]. A reference for the mathematics only.
- **Cubed-sphere and polyhedral grids** store each seam as (neighbour tile, rotation count, shift): FMS mosaics,
  MITgcm exch2, H3, rHEALPix. S2 avoids tables by going through 3-D coordinates [SA, code and docs read].
- **Petroff 2021 in software.** One panorama viewer by the same author. No game, world generator or Minecraft use
  was found [SA].
- **Spectral solvers on a half-turn domain.** No published code found [SA]. §3.4 gives the workaround.

### 2.6 The technical pieces, one by one [SA, code read, unless tagged]

| Piece | Exists where | Usable how |
|---|---|---|
| Fold a position through a turning seam | TW `SeamTransform`, `FoldOrientation`, `DeckGroupFold` | In our dependency, dormant. Mirrors and skew only; a half-turn generator must be added |
| Turn the face and in-block offset of a click | TW `FoldOrientation.applyToFace`, used in `SeamHit` | Dormant, already written |
| Rotate a block state 180° | Vanilla `BlockState.rotate`, exercised by every rotated structure [M]; Create and Sable both patch the bell | Vanilla call; gaps are modded blocks that do not override `rotate` |
| Rotate a block entity's stored positions | Create `TransformableBlockEntity` and its registries (tracks, stations, signals, pipes, arms); TW's `positions.json` names which NBT keys hold positions | Create's for Create blocks; a general registry must be built |
| Rotate a whole chunk's contents in its packet | **Nowhere found** | Must be built |
| Show one stored chunk at two client positions | **Nowhere in TW** (it guarantees the opposite); Immersive Portals does it by drawing the world twice | Must be built, or avoided (§3.3) |
| Draw stored chunks under a transform | Valkyrien Skies 2 (LGPL-3.0) and Sable, through private Sodium mixins | Design reference only; no public API |
| Carry an entity across with its heading turned | Immersive Portals turns velocity and the player's camera; vanilla teleports take a yaw [M] | Design reference |
| Move a physics ship with a new orientation | Sable `PhysicsPipeline.teleport(body, position, orientation)`, public | Dependency; TW's guard must be relaxed |
| Distant Horizons across a seam | TW mixes into DH's storage and quad-tree, translation only (22 files) | Dependency for translation; a rotated copy is a column re-ordering nobody has written. DH's API has no transform |
| Create trains across a seam | TW has translation folds for tracks and carriages | No precedent for a turn; the train graph is keyed by absolute positions |

---

## 3. What the prior art implies for the design [I]

These are my derivations from the sources above. None has been run in a game.

### 3.1 The fold is a parity rule

Tile the plane with copies of an N × N square. Copy (p, q) is the original turned 180° when p + q is odd:

```
p = floor(i / N);  q = floor(j / N)
a = i − p·N;       b = j − q·N
if (p + q) is odd:  a = N − 1 − a;  b = N − 1 − b      // and vector quantities change sign
```

- I checked the algebra by hand: two half-turns about adjacent edge midpoints compose to the slide (N, N), so the
  rule is consistent.
- A sub-agent derived the same rule and reported that the 5-point Laplacian built from it is symmetric and has the
  predicted spectrum for N = 6 to 9. I did not rerun its script. [SA]
- Its top-edge case is the same map as the ocean models' north fold.

### 3.2 Two frames for the same world

- **Peirce frame.** One square, north pole in the centre. Every edge folds onto itself about its midpoint. The four
  turn points are the edge midpoints.
- **Guyou frame.** The square cut and reassembled as a 2 : 1 rectangle, the two hemispheres side by side. Then x
  wraps by an **ordinary slide**, which is Toroidal World's cylinder as it runs today, and only the two long edges
  fold onto themselves by a half-turn. The four turn points sit at x = 0 and x = L/2 on each long edge.

The Guyou frame needs one new seam type on top of what TW already does. The Peirce frame needs TW's slide on
neither axis.

### 3.3 The four turn points break "one copy per client"

- A half-turn about a point maps a chunk next to that point onto its neighbour on the other side. Standing there, a
  player would see the same stored chunks ahead and behind, turned. Any mob there appears twice. So does the player.
- For a player at P with view distance V, a chunk and its turned image are both in view only if P is within V of
  the turn point (the turn point is the midpoint of the pair, and the midpoint is no farther from P than the
  farther of the two).
- So there are two ways to handle it:
  1. **Keep players at least V from each turn point.** Then TW's one-copy invariant holds everywhere a player can
     stand. Each no-go area is a half-disc of radius V in the square. At a 16,384-block circumference (square side
     about 9,244 blocks):

     | View distance | No-go radius | Share of the planet lost, four areas |
     |---|---|---|
     | 12 chunks | 192 blocks | 0.27 % |
     | 32 chunks | 512 blocks | 1.9 % |

     At 32,768 the shares are a quarter of these.
  2. **Send two copies near the turn points.** Chunk tracking has to work in the client's frame, block updates go
     to both copies, and each entity in range needs a second client-side identity whose interactions map back.
     Nothing in Minecraft's ecosystem does this by packet rewriting; Immersive Portals gets the effect by drawing
     the world a second time.
- Distant Horizons draws far beyond V, and the world is only about 578 chunks wide at 16,384. Its far terrain needs
  turned copies in either case.
- To map chunks onto chunks, each turn point must sit on a chunk corner. In the Peirce frame that means a square
  whose side is an even number of chunks.

### 3.4 The simulation side

- Kernels use the parity rule as their neighbour rule. Scalars carry over; vector components (winds, flow
  directions, slopes) change sign on odd tiles. At a turn point a vector must be zero.
- Separable symmetric filters work unchanged (Praun & Hoppe).
- For the two FFT solvers (flexure, orography), solve on the doubled torus and keep the point-symmetric solution, as
  the design note already proposes, or switch those two to a real-space solver with the fold rule.
- The planet is simulated and stored once. The "one simulation, two copies" work in §5.3 item 3 of the design note
  goes away. The rest of §5.3 stays: latitude is no longer a row, north is not up, and shapes are stretched up to
  about 2.7× near the turn points.

---

## 4. Reuse ledger

**As a dependency (already have, or could add)**
- Toroidal World for every slide seam and all of its mod compat.
- Sable's public teleport with an orientation.
- Create's block and block-entity transform registries, for Create's own blocks.
- Vanilla `BlockState.rotate`.

**By contribution or fork (MIT allows either)**
- A half-turn shape in TW's `FlatShape`, the fold wired into `WorldFolds`, and orientation carried through the 287
  per-axis call sites. Whether the maintainer wants this is unknown; nobody has asked, and no outside pull request
  exists to judge by.

**As a design reference only**
- Ocean-model north folds (Apache-2.0 and MIT code could be ported with attribution; NEMO is CeCILL).
- Praun & Hoppe for filtering across the fold.
- Immersive Portals for turning cameras, velocities and entity renders (Apache-2.0, portable with attribution).
- Valkyrien Skies 2 (LGPL-3.0) and Sable (PolyForm Shield) for drawing chunks under a transform.
- HyperRogue (GPL-2.0), Circumnavigate and `Mrcavas/planets` (AGPL-3.0), WorldEdit's property-by-property rotator
  (GPL-3.0): ideas only, no code.

**Still to be built, with no precedent found**
1. Rotating a chunk's contents on the way to the client: section palettes, heightmaps, light, biomes, block-entity
   positions, block-state facing.
2. The same for single block updates, and the reverse for what the client sends back.
3. Either the no-go areas or the two-copy system of §3.3.
4. Entity crossing with heading, velocity, navigation targets and passengers turned.
5. Sable bodies crossing with a turned orientation and turned velocities.
6. Create contraptions and, hardest, trains.
7. Distant Horizons turned copies.
8. Maps, compasses and every "north" reading. These were already needed for any quincuncial world.
9. In the Rust core: the fold neighbour rule in every kernel, and the per-cell latitude work of §5.3.

---

## 5. Effort estimate

**These are my judgement, not measurements.** Nothing was prototyped. The only measured inputs are the size of
Toroidal World and how long it took.

- **The yardstick.** TW reached translation-only wrapping with MIC's compat list in about eight weeks of one
  maintainer's very fast work: first commits 2026-08-06 and 770 commits by 2026-09-27 per
  `wrapped_planet_worlds.md` (I counted 475 on `dev` alone), and 59,798 lines of main Java on `mc/1.21` [V].
- **A feasibility prototype with Immersive Portals – CE.** Eight rotated global portals around a test square, no
  Deep Time terrain logic. My guess is one to two days, mostly fighting mod conflicts. It would show what standing
  at a turn point looks like and whether "north is not up" is tolerable. It would not show Distant Horizons terrain
  and is not a route to shipping.
- **A feasibility spike on a TW fork.** A half-turn shape, chunk-content rotation and a player walking across, with
  no Create, Sable or DH. My guess is about a week. This is the spike that would replace the guesses below with
  evidence.
- **A full engine at MIC quality, extending TW.**
  - With no-go areas at the turn points: my guess is four to seven weeks.
  - With the two-copy system: six to ten weeks.
  - The largest unknowns are entity duplication, Create trains and Distant Horizons.
  - After that, the fork must keep up with upstream TW, which shipped 12 builds for 1.21.1 in five weeks
    (`wrapped_planet_worlds.md` §1.1).
- **Not included.** The Rust-side projection and climate rework listed in §5.3 of the design note. That work is
  needed for any quincuncial world and was not re-estimated here.

Compared with what was on the table before this research: the "1–2 day feasibility spike" in the decisions log is
realistic only for the Immersive Portals prototype. A spike on the real engine is closer to a week, because chunk
contents have to be rotated and that exists nowhere.

---

## 6. Recommendation

This is for Ben to decide; it outlives the session.

1. **Do not plan on finding this ready-made.** It does not exist. The honest summary is "half the vocabulary, in a
   dependency we already use, and solved mathematics from other fields".
2. **Keep Lambert as the working default**, as `world-projection-options.md` recommends. This research removes one
   of the four costs of a quincuncial world (two copies of the planet). The other three stand: north is not up, four
   turn points, and the climate rework. It adds a multi-week engine.
3. **If Ben wants the half-turn world anyway**, the route is Toroidal World, in this order:
   - Ask the maintainer whether a half-turn shape is wanted upstream. The API text suggests openness to new folds;
     nothing says so for this one.
   - Run the one-week TW-fork spike and judge from screenshots at a seam and at a turn point.
   - Use the Guyou frame (§3.2), so that x keeps TW's existing slide.
   - Start with no-go areas at the turn points, in ocean, and treat the two-copy system as a later upgrade.
4. **Do not build on Immersive Portals.** It loses Distant Horizons across every seam and conflicts with Toroidal
   World. It is useful at most as the cheap look-and-feel prototype.
5. **Whatever is chosen for the world, the simulation can adopt the fold cheaply.** The parity rule and the ocean
   models' sign conventions are enough to write it.

---

## 7. What I could not verify, and what was not covered

- **The Minecraft sweep is thinner than briefed.** The sub-agent for it failed. Not searched: YouTube devlogs,
  Reddit (r/feedthebeast, r/Minecraft, r/admincraft), CurseForge-only projects, PlanetMinecraft, MCPEDL, Discord
  projects, Spigot and Bukkit beyond what `wrapped_planet_worlds.md` already has. A rotated-seam project that lives
  only in those places would have been missed.
- **Immersive Portals – CE.** Its Sable and rotated-portal claims are from the README. Nobody ran it. Whether global
  portals keep working when rotated, and whether CE fixed the C2ME and Petrolpark conflicts, are unknown.
- **Toroidal World's intent.** Inferred from unused code and one paragraph of documentation. The maintainer was not
  contacted.
- **Planet Border Teleport's licence.** Modrinth says LGPL-3.0-or-later [V]; a sub-agent reported AGPL-3.0 from the
  repository. Not resolved.
- **Vanilla facts from memory.** How complete `BlockState.rotate` is for a 180° turn across modded blocks, and the
  exact 1.21.1 teleport signatures, were not checked against the NeoForge sources.
- **Distant Horizons internals, Valkyrien Skies, Sable rendering, Create's transform registries.** Read by a
  sub-agent, not by me.
- **Other games.** Eco, No Man's Sky, Dual Universe, Astroneer, Planet Nomads, Hytale and Vintage Story were not
  verified; a sub-agent ran out of search budget.
- **HyperRogue.** Whether its orbifold tool can build a 2222 on a square tiling was not tested.
- **Engelhardt & Dachsbacher 2008** (octahedron environment maps): not retrieved.
- **The parity rule's numerical test** was run by a sub-agent; I checked the algebra, not the script.
- **Every estimate in §5** is judgement.
- **No game was launched, and nothing was built.**

---

## 8. Method and sources

- **Toroidal World.** Cloned on the test VM; read `core/FlatShape`, `WorldFolds`, `SeamTransform`,
  `FoldOrientation`, `DeckGroupFold`, `DeckGroupLattice`, `DeckTransformation`, `compat/distanthorizons/DhFold`,
  `docs/modding.md`; counted call sites with grep on `mc/1.21`; GitHub API for issues, pull requests and comments;
  Modrinth API for the project and its members. The clone was deleted afterwards.
- **Circumnavigate.** Cloned on the test VM at `5d662fb`; grepped; deleted.
- **Immersive Portals.** Upstream NeoForge port at `aede93a` (a sub-agent's clone, three greps by me);
  `UpperMoon0/ImmersivePortals` through the GitHub API (metadata, releases, README, `gradle.properties`,
  `build.gradle`).
- **`Mrcavas/planets`.** `LICENSE`, `LinkedRegion.kt`, `Region.kt`, `Planets.kt`, three mixins and the commit list,
  through the GitHub API.
- **Modrinth.** `api.modrinth.com/v2/search`, about 30 queries, 15 hits each.
- **Web pages fetched by me.** Red Blob Games `x/1938-square-tiling-of-sphere`; CivFanatics resource 30407; Cemetech
  forum post 280650; Factorio forum post 229894.
- **Sub-agent reports** (Sonnet): other games and engines; academic and numerical work; Minecraft technical pieces.
  Their primary sources include FMS `mpp_domains_define.inc` and `mpp_do_updateV.fh`, NEMO `lbc_nfd_generic.h90`,
  Oceananigans `fill_halo_regions_*zipper.jl`, CICE `ice_boundary.F90`, H3 `faceijk.c`, S2 `s2cell_id.cc`,
  rHEALPix `cell.py`, HyperRogue `euclid.cpp` and `arbiquotient.cpp`, Seed of Andromeda `VoxelSpaceUtils.cpp`,
  Space Engineers cube-map helpers, Create `StructureTransform.java`, Sable `SubLevelAssemblyHelper.java`,
  Valkyrien Skies 2 Sodium mixins, Distant Horizons API and core, Praun & Hoppe 2003, Petroff 2021 and its Zenodo
  archive, McGlynn et al. 2019, Cigolle et al. 2014.
- **Papers cited.**
  - E. Praun & H. Hoppe, "Spherical parametrization and remeshing", ACM TOG 22(3), 2003.
  - M. A. Petroff, "A square equal-area map projection with low angular distortion, minimal cusps, and closed-form
    solutions", ACM TSAS 7(4), 2021, arXiv:2008.13670.
  - N. Aigerman & Y. Lipman, "Orbifold Tutte Embeddings", ACM TOG 34(6), 2015 (code read by a sub-agent; paper not
    read).
  - T. McGlynn et al., "Octahedron-based projections as intermediate representations for computer imaging: TOAST,
    TEA, and more", ApJS 240:22, 2019.
