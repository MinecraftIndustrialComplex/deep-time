# Strata

Status of this page: `stage 2 (checked against sources on 2026-10-02, package WP-7)`. Of the 46 active rows,
1 is `verified`, 1 `abstract-only`, 2 `calibrated` and 42 `game-choice`. Nothing in this subsystem is a
physical law that the code could get wrong in a number: almost everything is a rule of how the game
draws layers. The page says which parts agree with geology, which do not, and where the code departs on
purpose.

Ledger ids on this page: `STR-001` to `STR-050` (see [claims.md](claims.md)). The rock classes themselves
and the metamorphism that changes them are on [rock-and-metamorphism.md](rock-and-metamorphism.md).

## What you see in the world

Dig down anywhere and the rock comes in horizontal beds in the order they were laid down, youngest on top.
Each bed is at least one block (20 m) thick, a bed of regional metamorphic rock at least eight blocks
(160 m) and a bed of plutonic rock at least 16 (320 m). Beds run on across the landscape for kilometres and
thin out where the sea or the river that laid them did not reach; where one rock grades into another
sideways the two interleave in tongues. Under a belt of folded rock the beds are bent, cut by faults and
repeated, and the same bed can be met twice going down a thrust. Gaps in the record show as a bed lying on
much older rock. Slate, schist and gneiss lie under unmetamorphosed cover, never the other way round, and
metamorphic grade rises as you go down. Granite and its relatives are not beds but bodies: they cut across
the beds (see the other page).

The strata are drawn tall. One block is 300 m across and 20 m deep, so every layer is drawn 15 times thicker
against its width than on Earth. The world is 992 blocks (19.8 km) from the floor at y -352 to the ceiling at
y 639.

## The model in plain words

1. **The log.** Through the whole history the surface stage keeps, for every cell, a list of layers: what was
   laid down, how thick, when, and where its sediment came from. It is a stack of rock classes with ages, not
   a chemical record. Each step deposits on top; erosion strips from the top. The thickness of a layer is
   the thickness it was laid down with. Nothing compacts it (STR-049).
2. **The column.** At the end of the run, and in each saved frame, the structure stage builds a rock column
   for each map cell from the top down: the regolith, then the strata log with the classes that regional
   metamorphism left in it, then the rock pools of the plate model as basement (sediment, sedimentary,
   volcanic, metamorphic, plutonic), then the mantle. Young shallow beds are shown as their loose form
   (sand, mud), the rest as rock (RCK-022).
3. **The order.** Every unit gets a stratigraphic key, a 16-bit number that grows downward and orders units
   identically in every column. The key space is built grade first: the regolith, then one band per
   metamorphic grade (unmetamorphosed, then from 250, 300, 350, 450, 500 and 650 degC), then the mantle.
   Inside a band the log's layers go by age and the basement pools in erosion order (STR-018 to STR-021).
   Ordered by key, a column can only get hotter downward.
4. **Settling.** The bake smooths the layers sideways (a Gaussian of about 10 cells, STR-011 to STR-014),
   merges anything thinner than its class minimum into the bed under it, and keeps 32 horizons a cell
   (STR-001, STR-005 to STR-008).
5. **The column rule.** The game does not read a cell's stack but the blend of the 16 cells round the block
   column, matched by key, so a bed present in every cell interpolates smoothly and a bed present in some
   pinches out (STR-015 to STR-017).
6. **Structures.** Fold, fault, dyke and plug events (P2-01, see
   [folds-faults-intrusions.md](folds-faults-intrusions.md)) displace the keys' beds in the column rule. Rock
   laid down after a structure formed is not touched by it, which is how unconformities appear (STR-022).
7. **Traps and hosts.** For the oil and gas engine each cell gets a trap score from 0 to 1, the largest of
   several rules (folds, thrusts, faults, salt, growth faults, drape). For the ore engine each rock class gets
   the share it takes of the top 2,000 m (STR-025 to STR-043).
8. **The checks.** `deeptime strata-check` counts the columns a geologist would object to (STR-044 to STR-048).

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| STR-001 | horizons per cell | 32 (2 to 255) | count | plan section 5.7.4 | `game-choice` |
| STR-003 | build range | y -352 to 639, sea level y 63 | blocks | design decisions file | `game-choice` |
| STR-004 | strata thickness multiplier | m_per_block_v / feature_scale.strata; preset 1 | - | decisions log 2026-09-28 | `game-choice` |
| STR-005 | thinnest horizon | 1 block (20 m) | blocks | build log P1-07 | `game-choice` |
| STR-006 | thinnest metamorphic horizon in a stack | 8 blocks (160 m) | blocks | build log 2026-10-01 | `game-choice` |
| STR-007 | thinnest plutonic horizon in a stack | 16 blocks (320 m) | blocks | build log 2026-10-01 | `game-choice` |
| STR-009 | rock cover over a deposit ends at a non-sedimentary horizon at least | 3 blocks | blocks | decisions log 2026-09-30 | `game-choice` |
| STR-011 | lateral smoothing | Gaussian sigma 10 cells (0 to 16) | L1 cells | decisions log 2026-09-30 | `calibrated` |
| STR-012 | smoothing taper | Gaussian of 1.5 sigma lowered to zero at the window edge | - | P1-23 | `calibrated` |
| STR-014 | ridge filled by the thickest unit within | 16 blocks | blocks | decisions log 2026-10-01 | `game-choice` |
| STR-015 | column rule | 4 x 4 cells, uniform cubic B-spline weights | - | format design | `game-choice` |
| STR-016, STR-017 | tongue band and minor share | 64 keys; 0.15 | keys; fraction | build log 2026-09-29 | `game-choice` |
| STR-018 | key space | regolith 0-4095, seven grade bands, mantle 61440-65535 | keys | build log 2026-10-01 | `game-choice` |
| STR-025 | sediment a closure needs | 500 | m | design | `game-choice` |
| STR-026, STR-027 | salt dome | halite at least 100 m under at least 1,000 m; score min(1, t / 200) x min(1, overburden / 1000) | m | design (Hudec and Jackson 2007 not read) | `game-choice` |
| STR-028, STR-029 | growth fault | sediment over 1,500 m; score 0.8 x min(1, (s - g) / g) | m | design | `game-choice` |
| STR-030 | fold-thrust belt reach | 300; score min(1, s / 2000 m) x (1 - d / 300 km) | km | Alavi 2007: average width about 300 km | `verified` |
| STR-032, STR-033 | rift fault blocks, drape | 0.6 x min(1, s / 1000 m); 0.1 x min(1, s / 1000 m) | - | design | `game-choice` |
| STR-034 to STR-038 | shares of folds and faults | fold train min(a / L / 0.1, 1); thrust 0.8 (foot wall 0.4); normal 0.6; strike-slip 0.3; edge fade min(1, 2w) | - | design | `game-choice` |
| STR-041 | ore window | 2,000 | m | design | `game-choice` |
| STR-044 to STR-048 | realism checks | zero grade inversions; slivers under 8 blocks; plutonic sheets under 16 blocks | - | build log 2026-10-01 | `game-choice` |
| STR-049 | compaction | none: a layer keeps its deposited thickness | - | - | `game-choice` |
| STR-050 | time in a one-block horizon | 20 m / accumulation rate | Myr | secondary | `abstract-only` |

The trap closure of a cell is the largest of the rules, clamped to 0 to 1 (STR-040):

closure = max over rules of: min(1, sediment / (4 x 500 m)) x share (folds and faults, STR-034);
min(1, halite / 200 m) x min(1, overburden / 1,000 m) (salt); 0.8 x min(1, (sediment - 1,500 m) / 1,500 m)
(growth faults); 0.6 x min(1, sediment / 1,000 m) (rift blocks); 0.1 x min(1, sediment / 1,000 m) (drape).

## Earth against the mod

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| Width of the Zagros fold-thrust belt | average about 300 km, in a 200 to 300 km wide zone of deformation | Alavi 2007, Am. J. Sci. 307, abstract and text (read in full) | `fold_belt_km` 300 | the setting; no run needed |
| Thickness of a deposited shale layer after burial | 60 % of its deposited thickness at 1 km, 43 % at 3 km (sand 82 % and 65 %, chalk 46 % and 33 %) | worked from Sclater and Christie 1980 parameters as tabulated in the pyBacktrack documentation (secondary) | 100 % at any depth (STR-049) | arithmetic: phi = phi0 exp(-z/c), thickness ratio (1 - phi0) / (1 - phi) |
| Porosity of mud and sand at 150 m | shale 0.58, sand 0.47 (still soft) | same | rock below 150 m if older than 20 Myr (RCK-022) | arithmetic |
| Accumulation rate of pelagic sediment | calcareous ooze 3-50 m/Myr, siliceous ooze 2-10, red clay 1-5; platform carbonate up to about 1,000 m/Myr | Rothwell 2005 via Wikipedia; Schlager 1981 via Wikipedia (secondary) | a one-block bed is 20 m, so 0.4-20 Myr of pelagic sediment | arithmetic |
| Niger Delta wedge | a clastic wedge 12 km thick over 75,000 km2, growth faults detached on overpressured shale | Sanuade and Kaka 2017, Geol. Q. 61(1) (secondary review) | growth-fault threshold 1,500 m (no source) | not measured |
| Dip of beds under lowland | none given | - | median 18.6 degrees in block space, 1.3 degrees real-equivalent at sigma 10 | `dips.py` on 1,500 columns, `earthlike_quick_16k`, scattered, 500 Myr, seed 1, 2026-09-30 (build log) |
| Columns with a grade inversion, a thin metamorphic sliver, a plutonic sheet | none counted; inverted sequences are real but rare in a given column | - | WP-7_BEFORE_AFTER_STRATA | see below |

Strata-check on a 16k quick planet (preset `earthlike_quick_16k`, start `scattered_continents`, 500 Myr, 2 threads,
seed default, `deeptime run` then `deeptime strata-check --world-data … --columns 20000`, test machine, 2026-10-02;
before is master 9e84fcf, after is this branch with the pluton thickness fix):

STRATA_CHECK_TABLE

## Simplifications and limits

- **No compaction.** A layer keeps its deposited thickness at any depth. Real mud halves by 3 km of burial. The
  strata are already drawn enlarged, so this changes the proportion of sediment against crystalline rock, not
  the look of a column. It is not a decision anyone recorded (STR-049).
- **No sedimentary structures.** A bed is one rock class with one age span. There is no bedding thickness
  distribution, no cross-bedding, no turbidite sequence: a one-block bed stands for 0.4 to 20 Myr of pelagic
  sediment and as little as 20 kyr of platform growth (STR-050).
- **Unconformities are implicit.** Rock younger than a structure is not deformed by it. A hiatus has no record,
  so the gap is only visible as a jump in age and grade (STR-022).
- **Bodies are painted where they are.** A fault younger than a pluton does not offset it: plutons, alteration
  halos and gossans are bodies painted over the horizons.
- **The cover depth of a deposit reads the undeformed stack,** not the folded one (P2-01 stage B, left open).
- **Grade order is a rule of the look.** In real orogens inverted sequences occur (ophiolite soles, thrust
  zones, the Himalaya), and metamorphic bodies a few hundred metres thick are real. The rule keeps cover on
  basement and grade rising downward, which is the Barrovian default (STR-044).
- **One belt width for traps** (`fold_belt_km` 300) and another for the fold engine's foreland (250, DEF-027),
  both meant for the Zagros; the data say about 300 km.
- **Closure scores are design numbers.** Trap types are standard (anticlines, faults, tilted fault blocks, salt
  domes; secondary source); the weights are not measured.

## Game choices

Most of this page: the number of horizons (STR-001), the build range (STR-003, decided by Ben, design decisions
file), the one-block minimum (STR-005), the 8 and 16 block minimums (STR-006, STR-007), the smoothing and its
taper (STR-011 to STR-014, Ben 2026-09-30 and 2026-10-01), the column rule and tongues (STR-015 to STR-017, Ben
2026-09-29), the key space (STR-018 to STR-021), the coal cover cap (STR-009, Ben 2026-09-30), and the trap and
host weights (STR-025 to STR-043, agent design values).

## Open questions

1. **Compaction.** Should buried layers thin with depth (STATUS pending questions)? It would reduce sediment
   against basement by tens of per cent and make deep sedimentary piles shorter.
2. **Salt and growth-fault thresholds** have no source; Hudec and Jackson 2007 and Doust and Omatsola 1990 are
   paywalled. A reader with access could check the 100 m, 1,000 m and 1,500 m values.
3. **The Subandean belt width** (100 to 200 km in the stage-1 comment) was not read.
4. **Wording of the stub:** the stage-1 stub said the mod "compacts" layers. It does not.

## Sources

Read in full: Alavi 2007 (library `Alavi2007`); Mottram et al. 2014 (inverted sequences, `Mottram2014`);
Cruden 2008 (SKB R-08-138, for the plutonic thickness used in STR-007, `Cruden2008_SKB`).
Read in part or second hand: Hacker's lecture notes (ophiolite sole thickness, `Hacker_UCSB_geo102C`); Nelson's
lecture notes (`Nelson_Tulane_EENS212`); Manger 1963 (porosity, `Manger1963`); pyBacktrack documentation
(Sclater and Christie parameters, no saved copy); Sanuade and Kaka 2017 (no saved copy); Hudec and Jackson 2007
(quoted by a blog; not read). Not read: Doust and Omatsola 1990, Sadler 1981, Schlager 1981, Miall.
