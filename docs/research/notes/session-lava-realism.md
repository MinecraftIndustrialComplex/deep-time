# How realistic lava should be, and the lava floor as a game convention

Date: 2026-10-01 (transcript UTC timestamps, 17:05 to 22:34).
Status: the coordinator's assessment, recorded from a working session. Not a literature review.

## Question

Where does molten rock exist on a real planet, and where should lava exist in a generated Deep Time
world? In particular, should the layer of lava at the bottom of the world, which vanilla Minecraft
has, stay or go? The question arose after a bug in which lava flooded caves and deep seabeds far
above the world's bottom.

## Assessment

The bug. Vanilla Minecraft hard-codes two lava depths for a world whose floor is at y -64. One
fills every open block below y -54 (the floor plus 10) with lava. The other lets a cave lake at
y -10 or lower (the floor plus 54) be lava. Deep Time's world floor is at y -352, so the first rule
flooded about 300 blocks of depth: every cave and the bottom of every deep sea. A sub-agent
measured one coastal preview box: 2,152,868 lava blocks, 4,020 sea columns standing directly on
lava, and 5,810 lava faces touching water. The default fix measures both depths from the world's
floor: lava below y -342 (floor plus 10), lava lakes only in aquifer cells within 54 blocks of the
floor, and lava kept 8 blocks below any seabed that reaches the floor. After the fix the same box
held 8,267 lava blocks, nearly all at the floor, and none under the sea. Both depths are settings;
0 turns either off. A block is 20 m tall in the default preset, so the floor is about 8.3 km below
sea level and the lava zone is the bottom 64 blocks of a world about 990 blocks tall.

The realism answer. Asked what the most realistic way to have lava is, the coordinator said lava
should exist only under and inside active volcanoes and nowhere else. It listed where molten rock
really is:

- Under active volcanoes: magma chambers a few kilometres down with a conduit to the crater. At the
  game's scale that is roughly 100 to 500 blocks, which fits inside the world's depth.
- In craters during activity, as a lava lake or as flows. Persistent lava lakes are rare; Earth has
  only a handful at any time.
- Along spreading ridges and rifts: shallow magma under the axis, mostly under the sea.
- Over hotspots such as Hawaii or Iceland.

And where it is not: under ordinary land (rock 8 km down is hot, around 200 to 250 degrees C, but
solid), under dead volcanoes (the chamber has frozen into granite-type rock, which the simulation
already produces), and in random caves. So a lava layer across the whole planet near the bottom of
the world is a game convention, not geology.

The cost it named is playability. In a fully realistic version lava becomes a regional resource;
some starts would be thousands of blocks from any volcano, and lava matters for obsidian, Nether
portals and fuel. Vanilla assumes a player can always dig down to lava. It offered a middle
option: volcano lava plus the deep floor kept as a convention, 300 or more blocks down, reachable
only by digging to the bottom. It also said that, at 400 or more blocks of digging, the floor is a
weak guarantee anyway. In the same discussion it said the small random lava lakes and springs that
still appear at any depth come from ordinary world features, not from the floor rule.

Later measurement in the repository (volcano-lava work, build log in `STATUS.md`): on the 16k quick
planet, 123 active volcanoes, 100 of them on land and 16 with crater lakes; a land point is a median
658 blocks (90th percentile 1,309) from the nearest active volcano on land.

## What was sourced and what was not

- The cause of the flood and all lava counts: a sub-agent's diagnosis and measurements in this
  repository (preview boxes of the owner's runs, a new GameTest that fails when vanilla's rule is put
  back, and the code in `LavaRules.java` and `LavaFluids.java`). The coordinator read the code
  before explaining the exact rule.
- Every statement about real lava (where it occurs, chamber depths, "a handful" of persistent lava
  lakes, 200 to 250 degrees C at 8 km, frozen chambers becoming granite-type rock): no source was
  consulted; this is the model's own knowledge and has not been checked.
- The 100 to 500 block chamber depth is the coordinator's conversion at the game's scale, not a
  sourced figure.
- The distance-to-volcano figures: measured by the volcano-lava sub-agent on one planet.

## What was decided

On 2026-10-01 Ben approved the volcano model: magma chambers, conduits and crater lakes at active
volcanoes, produced from the bake, and the random lava-lake features and springs removed behind a
setting. Whether the lava floor at the bottom of the world stays was not decided; it stays as a
setting meanwhile. Both are in the 2026-10-01 row "Lava flood fixed ..." of
`docs/deeptime-design-decisions.md`. The open floor question and the measured distance to lava are
a pending-question row in `STATUS.md`.

## Open

- Keep or remove the lava floor. This is undecided in the repo.
- The real-world facts above are unchecked. Chamber depth, the number of persistent lava lakes and
  the 200 to 250 degree figure need a source before they go into a science page.
- Whether a median distance of 658 blocks to the nearest volcano is acceptable for players, and
  whether lava for obsidian and portals needs a guarantee (for example a floor or a dedicated
  source) in packs that rely on it.
- Lava at spreading ridges and rifts is not produced yet, because the bake does not export them
  (`STATUS.md`). What the target pack's early recipes need lava for was not checked.
- Which feature or mod produces the leftover small lava lakes near the surface was not traced by the
  first agent; the volcano work removes the random features behind a setting.
