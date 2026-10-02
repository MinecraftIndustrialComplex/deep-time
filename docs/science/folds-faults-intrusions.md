# Folds, faults and intrusions

Status of this page: `stage 2 (constants checked against sources on 2026-10-02, package WP-8; vertical scale and intrusion sizes decided by Ben the same day)`.

Ledger ids on this page: `DEF-001` to `DEF-074` ([ledger/DEF.md](ledger/DEF.md)). Of these 74
rows, 17 are `verified`, 1 `abstract-only` and 56 `game-choice` (stage 1: 16 `cited-unverified`,
54 `game-choice`, 4 `uncited`).

**Read this first.** The method is a port of Noddy, a kinematic structural-geology program, and the
port matches the original: its constants were found in the original source and its conventions in
the original documentation. The physical rules that decide which structure forms where (thrusts and
folds where the crust shortens, normal faults where it is pulled apart, strike-slip faults where
plates slide past each other) and their dips follow Anderson's theory of faulting, read in his 1905
paper. Fold wavelengths and fold-train lengths come from the Zagros, read in full. Most counts and
sizes are design values, now labelled as such.

**How tall structures are drawn.** Everything else in a rock column is drawn in true metres, 20 m
to a block upwards and 300 m across, so the world is stretched 15 times vertically. Folds and faults
are first shaped at 1:1 in blocks (normal faults at about 60°, thrusts at about 30°, as a geologist
would draw them) and then drawn `vertical_scale` times taller: fold heights and fault throws are
multiplied by it, the sideways movement of a fault is not (DEF-011). Ben chose a setting with the
default 4 on 2026-10-02. At 1 every structure would be 15 times lower in metres than it is wide; at
15 it would be drawn in true metres. At 4, in metres, normal faults dip 21° to 36° and thrusts 5.5°
to 12.6°, and fold heights and fault throws are 4/15 of real: folds about 400 m high on a 16 km
wavelength where Zagros folds are 500 to 1,000 m, and a 38 km normal fault throws the beds about
470 m where a real one throws them about 1 km. True metres wait for one missing piece of physics:
a fold's wavelength should follow the thickness of the layers being folded (Biot), and with every
fold drawn about 16 km long whatever lies under it, true heights pull the granite basement up through
thin cover in keels (open question 3). The ground's share of the structures (fault scarps, rift
valleys) was lowered with the scale, so the terrain is as it was (DEF-054).

Scale used throughout: one block is 300 m across and 20 m up, so 1 km is 3.3 blocks across and
50 blocks up.

## What you see in the world

**Fold belts.** Where continents have collided, and in the mountain belts above subduction zones,
the rock layers are bent into trains of folds: upright anticlines (arches) and synclines (troughs)
with a crest-to-crest distance of about 53 blocks (16 km), half that to twice it, a few crests side
by side and 130 to 1,300 blocks long along the axis. At the default vertical scale a fold rises and
falls a median of about 20 blocks (400 m), its limbs dipping 66° to 77° in blocks (9° to 16° in
metres). Rock laid down after a fold formed lies flat across it.

**Faults.** Thrusts (gently dipping faults that push older rock up over younger) cut the same belts;
normal faults (steep faults whose upper side has slid down) drop tilted blocks along rifts;
strike-slip faults (vertical faults whose sides slid sideways) run along oblique plate margins. A
fault's trace is 15 to 150 km long, a median 38 km (127 blocks). At the default vertical scale normal
faults dip 80° to 85° in blocks and thrusts 55° to 73°, and a median normal fault drops the beds
about 23 blocks (470 m) at its middle. Young faults and folds also shape the ground: fault scarps,
rift valleys and fold ridges up to a few hundred metres high, worn away over 10 Myr.

**Intrusions.** Stocks (small bodies of granite, granodiorite or diorite) rise in collision belts and
arcs; most stop below the ground, and where one crops out only its domed top shows. Basalt dykes,
1.7 to 3 blocks wide at depth (1 to 2 where they reach the ground) and 15 km (50 blocks) long at the
median, cut the rock along rifts. Under every extinct volcano still standing on land a plug of
rhyolite, andesite or basalt 4 to 10 blocks across at depth (2.4 to 6 at the ground) fills the old
conduit. Both are still wider than real ones (a dyke is metres to 150 m, Ship Rock's neck 500 m):
Ben chose sizes between the earlier ones and the narrowest that can be drawn (DEF-048, DEF-049).

On the measured planet the structures cover 48 % of all columns and change the rock of 49 % of land
columns; 65 % of the stable continental interiors (cratons) lie under at least one structure, 16 %
under one younger than 50 Myr.

## The model in plain words

The structure stage runs once per plate step (about 2 Myr), after the plate and surface stages, and
keeps a list of structures that ride on their plates.

1. **Strain from the plates.** Each cell of continental crust gets three strain values from the plate
   model's tectonic regime: shortening (1 in a collision; up to 0.5 in an arc or back-arc that the
   plate model is lifting; up to 0.6 fading to 0 across a 250 km foreland, measured on the plate
   model's Earth-sized sphere), extension (1 in a rift) and shear (where a lifted subduction margin
   converges more than 30° obliquely) (DEF-024 to DEF-029).
2. **New structures.** For each kind the expected number in the step is the sum over cells of
   strain times area times the kind's rate times the step's length; the actual number is drawn from a
   Poisson distribution and each structure is placed in a cell with probability in proportion to its
   strain times area (DEF-022, DEF-023). Shortening makes fold trains and thrusts, extension normal
   faults and dykes, shear strike-slip faults; shortening in collisions and arcs also makes stocks.
3. **Sizes and attitudes.** A fold train's wavelength is drawn about 16 km, its amplitude follows from
   how much the crust there has thickened (DEF-001, DEF-005 to DEF-007), times the vertical scale
   (DEF-011). Folds and thrusts run along
   the plate model's fold trend; thrusts dip under the high ground; normal faults run along the rift
   and dip towards its axis, one in four the other way (DEF-009, DEF-030, DEF-031). Fault length is
   drawn about 40 km, the slip is a fixed share of it (DEF-013, DEF-015 to DEF-018), dips are drawn
   from Anderson's ranges (DEF-019 to DEF-021); then the vertical scale multiplies the throw and
   keeps the heave, which steepens the dip (DEF-011).
4. **Room.** At most three structures of a kind overlap at a place; a new one replaces the oldest
   only if that is at least 30 Myr older (DEF-032, DEF-033).
5. **Riding the plates.** Every structure moves and turns with its plate; one that has had no
   continental crust near it for ten steps has been subducted and is dropped (DEF-035, DEF-037).
6. **At the end.** The extinct volcanoes standing on land get their plugs (DEF-049, DEF-050). Every
   structure becomes an event of the world data, youngest first (DEF-063 to DEF-068, DEF-072).
7. **In the game.** For each column of blocks, the generator undoes the events from youngest to
   oldest: a fold shifts the column up or down by its profile, a fault moves the part above its plane
   back along the slip, an intrusion takes the part inside its ellipsoid. What is left is read from
   the undeformed layers. Rock younger than an event is not moved by it. This is Noddy's method
   (inverse kinematics), with each event given a finite extent on the map (DEF-039, DEF-070).
8. **On the ground.** In the amplifier stage, young structures move the ground as they moved the rock
   (DEF-054 to DEF-062).

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| DEF-001 | fold wavelength | lognormal, median 16, sigma 0.35, clamped 8 to 32 | km | Mouthereau et al. 2007: 15.8 ± 5.3 km, 149 anticlines | `verified` |
| DEF-002 | wavelength and layer thickness | stated, not used: L = 2 pi H (R/6)^(1/3) | - | Schmalholz & Mancktelow 2016, eq. 13 | `verified` |
| DEF-004 | fold length over wavelength | 5 to 12 | - | Alavi 2007: anticlines up to 150 to 200 km, over 10 half wavelengths | `verified` |
| DEF-005 | fold shortening e | 0.08 to 0.30 | fraction | design; Zagros 16 to 30 % (Alavi 2007, Mouthereau et al. 2007) | `game-choice` |
| DEF-007 | fold amplitude | A = X x strength x L x sqrt(e) / pi | blocks | geometry (first order) | `verified` |
| DEF-011 | vertical scale of structures, X | 4 (1 = 1:1 in blocks, 15 = true metres) | - | Ben, decisions log 2026-10-02 | `game-choice` |
| DEF-013 | fault length | lognormal, median 40, sigma 0.5, clamped 15 to 150 | km | design | `game-choice` |
| DEF-015 | displacement and length | D = gamma L | - | Schlische et al. 1996 (0.03, via Lathrop et al. 2022); Elliott 1976 (7 % for thrusts) | `verified` |
| DEF-016 to DEF-018 | slip over length, along the dip in blocks | normal 0.05; thrust 0.08 x (0.5 to 1); strike-slip 0.05 | - | design | `game-choice` |
| DEF-019 | normal fault dip | 55 to 70 | degrees | Anderson 1905; Collettini & Sibson 2001 | `verified` |
| DEF-020 | thrust dip | 20 to 40 | degrees | Anderson 1905; Collettini & Sibson 2001 | `verified` |
| DEF-021 | strike-slip dip | 90 (stored 89.999) | degrees | Anderson 1905 | `verified` |
| DEF-027 | foreland belt width | 250, on the plate model's sphere | km | Alavi 2007: 200 to 300 km | `verified` |
| DEF-030 | thrust dip direction | under the hinterland | - | Alavi 2007: foreland-verging folds | `verified` |
| DEF-042 | stock radius at the world's floor | lognormal, median 4, clamped 2 to 10 | km | design; a stock is exposed over less than 100 km2 (Earle 2019) | `game-choice` |
| DEF-047 | dyke length | lognormal, median 15, clamped 5 to 50 | km | design inside 1 to 2,000 km read | `game-choice` |
| DEF-048 | dyke half-width | 0.85 to 1.5 | blocks | Ben, decisions log 2026-10-02 (real dykes 1 to 150 m) | `game-choice` |
| DEF-049 | plug radius | 8 % of the volcano's radius, clamped 2 to 5 | blocks | Ben, decisions log 2026-10-02 (Ship Rock 500 m across) | `game-choice` |
| DEF-054 | ground's share of a young structure's relief | 0.25 (1 / X at the default) | - | Ben, decisions log 2026-10-01 and 2026-10-02 | `game-choice` |
| DEF-066 | Noddy's rounded constants | pi 3.1415927, 2 pi 6.2831853, ln 2 0.693147, pi 3.1415926535 | - | noddyverse source | `verified` |
| DEF-067 | Noddy's tolerances | 0.001, 0.01 | - | noddyverse source | `verified` |
| DEF-068 | Noddy's table sizes | 628 + 1 samples of 1/100; 128 events; 89.999° | - | noddyverse source | `verified` |

The rest are design rules listed in the ledger. Equations as the code computes them:

    amplitude = X * strength * L * sqrt(e) / pi                              (DEF-007, DEF-011)
    e = e_lo + (clamp(1 - 35,000 m / thickness, e_lo, e_hi) - e_lo) * min(shortening, 1)   (DEF-006)
    s = strength * ratio * length            (thrust: times 0.5 + 0.5 * min(shortening, 1))   (DEF-016, DEF-017)
    dip = atan(X tan d),  slip = s * sqrt(cos^2 d + X^2 sin^2 d)   (d from Anderson's range)   (DEF-011)
    shift of a column in a fold train = w * amplitude * profile(2 pi (b - offset) / L)        (DEF-010)
    w = (1 - a^2)^2 (1 - b^2)^2 inside the window, 0 outside                                  (DEF-039)

where X is `vertical_scale`, L the wavelength in blocks, e the shortening, a and b the position along and across the
window as a fraction of its half-length and half-width, and the profile a sine with period 2 pi.

**The first-order fold.** A sine of amplitude A and wavelength L has arc length L (1 + (pi A / L)^2)
over one wavelength to first order, hence the code's A = L sqrt(e) / pi. Exactly, the amplitudes
drawn shorten the bed by 7.0 % at e 0.08 (limbs 29.5°) and by 20.2 % at e 0.30 (limbs 47.6°): the
first-order form overstates the shortening at the top of the range (DEF-007). All of that is at 1:1
in blocks; drawn X times taller the shortening is (X/15)^2 of it in metres: 0.04 % to 0.13 % at X = 1,
0.5 % to 1.4 % at the default 4, and the full 7 % to 20 % at 15.

**Metres and blocks.** A fault of dip d and slip s at 1:1 in blocks has a heave (horizontal part) of
s cos d blocks and a throw of s sin d. Drawn X times taller it keeps the heave and throws X s sin d
blocks, so in metres the heave is 300 s cos d and the throw 20 X s sin d: the dip in metres is
atan(X tan d / 15). The heave dominates, so the displacements are near real ones in metres (normal
faults 0.017 to 0.029 of their length, thrusts 0.031 to 0.075), and the throws are X/15 of the
true-metre shape's: at the default 4 about a quarter.

## Earth against the mod

Measured on one run: `earthlike_quick_16k`, scattered continents, 500 Myr, seed 1, master 9e84fcf,
2026-10-02, on the test machine (`tools/review/wp8-runs.sh world base`, which runs `deeptime run`,
then `deeptime strata-check` and `deeptime structure-sections`); 6,108 folds and faults and 6,066
intrusions kept. Those figures are at the old stretch 1. The rows marked `x4` were measured at the
default vertical scale 4 on a second pair of runs (2026-10-02, branch of the vertical-scale task, on
the test machine: `earthlike_quick_16k`, supercontinent start, 1000 Myr, seed 1,
`WP8_START=supercontinent WP8_LENGTH=1000 tools/review/wp8-runs.sh world <name>`), the same world
built twice, before (stretch 1, the old intrusion sizes and relief 1) and after; `x1` is its before.

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| Fold wavelength | 15.8 ± 5.3 km (11 to 30 km) | Mouthereau et al. 2007, fig. 5 and §[55], full text | median 52.5 blocks = 15.8 km | base |
| Fold length along the axis | up to 150 to 200 km; often over 100 km | Alavi 2007 p. 1068, Mouthereau et al. 2007 §[7], full text | median 424 blocks = 127 km | base |
| Fold amplitude | 500 to 1,000 m (Zagros, maximum) | Mouthereau et al. 2007 art. p. 5, full text | median 4.9 blocks = 98 m in metres (`x1`); 19.5 blocks = 390 m (`x4`) | base, x1, x4 |
| Fold limb dips | open folds 20° to 35° (interlimb 110° to 140°), near the front about 5° (170°), tight in the inner belt | Alavi 2007 p. 1077, full text | at 1:1 30° to 48° in blocks, 2° to 4° in metres; at the default 4 66° to 77° in blocks, 9° to 16° in metres | arithmetic on DEF-005, DEF-011 |
| Normal fault displacement over length | 0.03 (D = 0.03 L^1.06), typical 0.001 to 0.1 | Lathrop et al. 2022 pp. 2 and 7 (quoting Schlische et al. 1996), full text | 0.017 to 0.029 in metres; throw median 117 m for 38 km at 1:1, about 470 m at 4 | base (median slip 6.6 blocks); x4 (23.1) |
| Thrust displacement over length | about 7 % | Elliott 1976 p. 297, full text | 0.031 to 0.075 in metres; throw median 82 m at 1:1, about 330 m at 4 | base (median slip 8.2 blocks); x4 (16.7) |
| Normal fault dip | over 45°, about 60°; active ruptures 30° to 65° | Anderson 1905 pp. 389, 402; Waldron & Snyder 2020 p. 152; Collettini & Sibson 2001 abstract | at 4: 80° to 85° in blocks, 21° to 36° in metres (at 1:1 55° to 70° and 5° to 10°) | code |
| Thrust dip | under 45°, about 30°; peak 30° ± 5° | Anderson 1905; Collettini & Sibson 2001 abstract | at 4: 55° to 73° in blocks, 5.5° to 12.6° in metres (at 1:1 20° to 40° and 1° to 3°) | code |
| Fold-thrust belt width | 200 to 300 km (Zagros); 60 to 140 km (Subandean, Bolivia) | Alavi 2007 abstract; Mouthereau et al. 2007 §[7]; Hérail et al. pp. 179-180 (poor scan) | 250 km on the plate model's sphere, 102 blocks = 31 km on the map | code (DEF-027) |
| Bed offset at a structure | throws of about 1 km on 40 km faults (from the rows above) | derived | median 2 blocks (40 m), p90 7 (140 m), max 166 (base); median 2, p90 8, max 252 (`x1`); median 8 (160 m), p90 29 (580 m), max 265 (`x4`) | `strata-check`, 20,000 columns |
| Dyke thickness | 2.3 m average (Ship Rock); 1 to 150 m, mean 30 m (Mackenzie giant swarm) | Delaney & Pollard 1981; Bunger et al. 2013, full text | 1.7 to 3 blocks (510 to 900 m) at depth, 1 to 1.8 blocks where they crop out (were 2 to 5 blocks) | code (DEF-048) |
| Dyke length | 1 to 9 km (Ship Rock); over 2,000 km (Mackenzie) | Delaney & Pollard 1981; Bunger et al. 2013 | median 50 blocks = 15 km | base |
| Dyke spacing in a swarm | 6.7 km to 25 km mean (Mackenzie), most continuous ones 35 to 65 km | Bunger et al. 2013 (quoting Hou et al. 2010, Stott & Josey 2009) | not measured (2,633 dykes kept) | base |
| Volcanic neck width | Ship Rock 500 m; small plugs under 30 m; Round Butte diatreme 170 to 190 m | New Mexico Bureau of Geology Ship Rock page; Delaney & Pollard 1981; Latutrie 2020 | 4 to 10 blocks (1.2 to 3 km) at depth, 2.4 to 6 blocks (0.7 to 1.8 km) where they crop out (were 5 to 16 blocks) | code (DEF-049) |
| Stock | exposed over less than 100 km2 | Earle 2019 §3.5 | radius median 13.3 blocks = 4.0 km at the world's floor; outcrops a few per cent of that | base |
| Cratons with old deformation | the basement of every craton is metamorphosed crystalline rock; its sedimentary platform cover is weakly deformed | Earle 2019 §21.1-21.2; Wikipedia "Craton" (secondary) | 65 % of craton cells under a structure, 16 % under one younger than 50 Myr | base |
| Rock columns a geologist would object to | none expected outside real overturned belts | (a check, not an Earth figure) | grade inversions 0.15 % of columns (`x1`), 0.36 % (`x4`; on land 0.10 % and 0.71 %); metamorphic slivers 0.41 % and 0.95 %; plutonic sheets 0.015 % and 0.17 %; malformed columns 0 in both | `strata-check`, 20,000 columns |
| Ground lifted or dropped by young structures | fault scarps and range fronts of hundreds of metres to a few km | (not sourced) | up to +337 m and -106 m, 43,920 cells, land median +1.2 m, identical in `x1` and `x4` (the relief share lowered to 0.25 with the scale) | x1, x4 |

## Simplifications and limits

- **Folds and faults are drawn 4 times taller than their 1:1 shape in blocks**, not in true metres
  (15), while the strata, plutons and terrain are drawn in true metres (DEF-011). See the opening
  section. True metres need wavelengths from layer thickness (next item).
- **Wavelength does not follow the layers.** Buckling theory makes a fold's wavelength proportional
  to the thickness of the stiff layer being folded, and natural folds have wavelengths of 2 to 16
  times it (DEF-002). Here every fold is drawn about 16 km, whatever lies under it: over the thin
  sedimentary cover of most of the planet a real fold would be shorter, and over the 7 to 12 km of
  the Zagros cover it is about right.
- **Belts are narrower than their folds.** Belt widths are measured on the plate model's
  Earth-sized sphere and shrink with the planet (a 250 km foreland is 31 km on the 16k map), while
  the folds in them keep their Earth size, so a foreland belt holds about two fold wavelengths
  across where the Zagros holds about thirteen.
- **No structure older than the run.** The planet starts with undeformed crust, so the basement of
  a craton that no structure reached in the run (35 % of craton cells here) is still flat-layered,
  where on Earth every craton's basement is old, deformed metamorphic rock.
- **Upright folds only**, with horizontal axes and a sine profile; no overturned, box or chevron
  folds, no fault-propagation or fault-bend folds, which the Zagros mostly has (Alavi 2007 p. 1077).
- **Planar faults** with a smooth bell of displacement towards their tips: no listric faults, no
  detachments, no duplexes; a thrust is a single ramp.
- **No sills**; stocks, dykes and plugs are ellipsoids rooted on the world's floor; the large
  batholiths are bodies of the plutons code and are not offset by younger faults.
- **Structures are drawn from strain, not computed from stress.** The rates per area are design
  values; the counts are not compared with any count on Earth.

## Game choices

- Folds and faults drawn 4 times taller than their 1:1 shape in blocks, as a setting (DEF-011): Ben,
  decisions log 2026-10-02 ("Setting, default 4x"), with the metre equivalents and sections drawn at
  1, 4 and 15 in front of him; the ground's share lowered to match (DEF-054). Before: undistorted in
  blocks, an agent's design in P2-01 (build log 2026-10-01).
- Strength, count and size multipliers and the terrain switch: Ben, decisions log 2026-10-01
  (DEF-012, DEF-054, DEF-072).
- Dykes 1.7 to 3 blocks wide and plugs 4 to 10 blocks across at depth, still many times real ones,
  so that they can be seen (DEF-048, DEF-049): Ben, decisions log 2026-10-02 ("Go somewhere in
  between"), between the earlier 2 to 5 and 5 to 16 blocks and the narrowest drawable. The
  narrowest dyke is the narrowest whose outcrop does not break into separate blocks.
- The fold profile is the exact sine where Noddy's default is a 21-point approximation of it
  (DEF-010).
- Every rate, overlap and age rule, the strain weights, the fold-train shape, the stock shape and
  depth and the relief rules: design values of the P2-01 agent (build log 2026-10-01, stages A and
  B), listed one by one in the ledger.

## Open questions

1. **Settled: folds and faults at a vertical scale of 4** (DEF-011; Ben, decisions log 2026-10-02).
   The question compared stretch 1 (today then: bed offsets median 2 blocks, p90 7; grade
   inversions 0.07 % of columns), 4 (offsets 7, p90 26; inversions 0.33 %) and true metres, 15
   (offsets 22, p90 91; inversions 0.78 %; faults near vertical in blocks; the granite basement
   pulled up through thin cover in arcs; the relief term raising ground by up to 5.1 km), on a
   500 Myr scattered-continents world (WP-8). Physics says true metres, but only together with item
   3; the default moves to 15 once that is built.
2. **Settled: intrusion sizes between the old ones and the narrowest drawable** (DEF-048, DEF-049;
   Ben, decisions log 2026-10-02): dykes 1.7 to 3 blocks wide at depth, plugs 2 to 5 blocks in
   radius.
3. **Fold wavelength from layer thickness (Biot), then true metres.** A design would take the
   folded cover's thickness H from the stack at the fold's place and age and draw L about 6.5 H
   (2 to 16 H; Schmalholz & Mancktelow 2016, DEF-002), with the amplitude from the shortening in
   metres. Once that is built, `vertical_scale` can default to true metres (15 at the built-in
   block), the ground's share (DEF-054) lowered with it. Not built (Ben, decisions log 2026-10-02:
   "true metres once fold wavelengths follow layer thickness").
4. **Belt widths on the small planet** (DEF-027): fold belts 31 km wide holding 16 km folds. Either
   the folds shrink with the planet like the belts, or the belts keep their Earth width; a question
   for the plate work, which owns the Earth-normalized lengths.
5. **Sources not obtained**: Anderson 1951 (book), Biot 1961, Sherwin & Chapple 1968, Blanc et al.
   2003, Cowie & Scholz 1992, Kim & Sanderson 2005, Gawthorpe & Leeder 2000, Ernst et al. 1995,
   Jessell & Valenta 1996; Weissel & Karner 1989 abstract only (DEF-059).

## Sources

Read by package WP-8 (2026-10-02) unless noted. Library file names are in the ledger rows.

- `Mouthereau2007`: full text (HAL copy via the web archive). Wavelengths, amplitudes, shortening,
  belt width, buckling.
- `Alavi2007`: full text. Belt width, shortening table, fold lengths and aspect ratios, vergence.
- `McQuarrie2004`: abstract only (the Caltech record is metadata-only). About 20 % shortening.
- `Schmalholz2016`: full text (open access). Dominant wavelength, natural ratios.
- `Biot1961`, `Sherwin1968`, `Blanc2003`: not obtained.
- `Lathrop2022`: full text. Displacement against length, quoting Schlische et al. 1996 and the
  spread.
- `Schlische1996`: abstract only. `Cowie1992a`, `Kim2005`: not obtained.
- `Elliott1976`: full text (Royal Society free copy via the web archive). Thrust displacement 7 %.
- `Anderson1905`: full text (CC0 scan, OCR). The theory and the angles; the code's 1951 book is not
  read.
- `WaldronSnyder2020`: full text (open textbook). Anderson's regimes, dips.
- `CollettiniSibson2001`: abstract only. Dips of active faults.
- `Gawthorpe2000`: not obtained.
- `Herail1990`: full text, a poor OCR scan; the year is the library's key, not read on the scan.
  Subandean widths.
- `DelaneyPollard1981`: full text (USGS). Ship Rock dikes and plugs.
- `Krumbholz2014`: full text (open access). Dyke thickness range and shape.
- `Bunger2013`: full text (author manuscript, HAL). Giant swarm dyke thickness, length, spacing.
- `NMT_Shiprock` (New Mexico Bureau of Geology and Mineral Resources web page), `Latutrie2020` (thesis): neck sizes.
- `Earle2019`: chapter pages §3.5, §21.1, §21.2 (open textbook via the web archive). Stock and
  batholith, cratons.
- `Jessell1981`: full text. Noddy's equations and conventions.
- `Jessell2022`: full text. Noddy's method, speed, licence.
- `Noddyverse (software)`: the source at the pinned commit, read for every constant in DEF-064 to
  DEF-069.
- `Weissel1989`: abstract only (stage 1).
