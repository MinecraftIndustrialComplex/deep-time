# Folds, faults and intrusions

Status of this page: `stage 2 (constants checked against sources on 2026-10-02, package WP-8)`.

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

One thing is not settled and is a question for Ben (STATUS.md, "Pending questions"). Everything
else in a rock column is drawn in true metres, 20 m to a block upwards and 300 m across, so the world
is stretched 15 times vertically; folds and faults alone are drawn undistorted in blocks. In blocks
they look as a geologist would draw them (normal faults at about 60°, thrusts at about 30°). In
metres every one of them is 15 times too low: the faults dip 5° to 10° and 1° to 3°, folds are about
100 m high on a 16 km wavelength where Zagros folds are 500 to 1,000 m, and a normal fault 38 km long
throws the beds about 120 m where a real one throws them about 1 km.

Scale used throughout: one block is 300 m across and 20 m up, so 1 km is 3.3 blocks across and
50 blocks up.

## What you see in the world

**Fold belts.** Where continents have collided, and in the mountain belts above subduction zones,
the rock layers are bent into trains of folds: upright anticlines (arches) and synclines (troughs)
with a crest-to-crest distance of about 53 blocks (16 km), half that to twice it, a few crests side
by side and 130 to 1,300 blocks long along the axis. In blocks a fold is a gentle to moderate bend,
its limbs at 30° to 48°, rising and falling a median 5 blocks. Rock laid down after a fold formed
lies flat across it.

**Faults.** Thrusts (gently dipping faults that push older rock up over younger) cut the same belts;
normal faults (steep faults whose upper side has slid down) drop tilted blocks along rifts;
strike-slip faults (vertical faults whose sides slid sideways) run along oblique plate margins. A
fault's trace is 15 to 150 km long, a median 38 km (127 blocks). Where it cuts the layers the beds
are offset by a median 2 blocks and at most about 170. Young faults and folds also shape the ground:
fault scarps, rift valleys and fold ridges, worn away over 10 Myr.

**Intrusions.** Stocks (small bodies of granite, granodiorite or diorite) rise in collision belts and
arcs; most stop below the ground, and where one crops out only its domed top shows. Basalt dykes,
2 to 5 blocks wide and 15 km (50 blocks) long at the median, cut the rock along rifts. Under every
extinct volcano still standing on land a plug of rhyolite, andesite or basalt 5 to 16 blocks across
fills the old conduit.

On the measured planet the structures cover 48 % of all columns and change the rock of 49 % of land columns; 65 % of
the stable continental interiors (cratons) lie under at least one structure, 16 % under one younger
than 50 Myr.

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
   how much the crust there has thickened (DEF-001, DEF-005 to DEF-007). Folds and thrusts run along
   the plate model's fold trend; thrusts dip under the high ground; normal faults run along the rift
   and dip towards its axis, one in four the other way (DEF-009, DEF-030, DEF-031). Fault length is
   drawn about 40 km, the slip is a fixed share of it (DEF-013, DEF-015 to DEF-018), dips are drawn
   from Anderson's ranges (DEF-019 to DEF-021).
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
| DEF-007 | fold amplitude | A = strength x L x sqrt(e) / pi | blocks | geometry (first order) | `verified` |
| DEF-011 | vertical scale of structures | undistorted in blocks | - | open question to Ben | `game-choice` |
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
| DEF-048 | dyke half-width | 1 to 2.5 | blocks | design (real dykes 1 to 150 m) | `game-choice` |
| DEF-049 | plug radius | 8 % of the volcano's radius, clamped 2.5 to 8 | blocks | design (Ship Rock 500 m across) | `game-choice` |
| DEF-066 | Noddy's rounded constants | pi 3.1415927, 2 pi 6.2831853, ln 2 0.693147, pi 3.1415926535 | - | noddyverse source | `verified` |
| DEF-067 | Noddy's tolerances | 0.001, 0.01 | - | noddyverse source | `verified` |
| DEF-068 | Noddy's table sizes | 628 + 1 samples of 1/100; 128 events; 89.999° | - | noddyverse source | `verified` |

The rest are design rules listed in the ledger. Equations as the code computes them:

    amplitude = strength * L * sqrt(e) / pi                                  (DEF-007)
    e = e_lo + (clamp(1 - 35,000 m / thickness, e_lo, e_hi) - e_lo) * min(shortening, 1)   (DEF-006)
    slip = strength * ratio * length         (thrust: times 0.5 + 0.5 * min(shortening, 1))   (DEF-016, DEF-017)
    shift of a column in a fold train = w * amplitude * profile(2 pi (b - offset) / L)        (DEF-010)
    w = (1 - a^2)^2 (1 - b^2)^2 inside the window, 0 outside                                  (DEF-039)

where L is the wavelength in blocks, e the shortening, a and b the position along and across the
window as a fraction of its half-length and half-width, and the profile a sine with period 2 pi.

**The first-order fold.** A sine of amplitude A and wavelength L has arc length L (1 + (pi A / L)^2)
over one wavelength to first order, hence the code's A = L sqrt(e) / pi. Exactly, the amplitudes
drawn shorten the bed by 7.0 % at e 0.08 (limbs 29.5°) and by 20.2 % at e 0.30 (limbs 47.6°): the
first-order form overstates the shortening at the top of the range (DEF-007). All of that is in
blocks; in metres the same folds shorten the beds by 0.04 % to 0.13 %.

**Metres and blocks.** A dip d drawn in blocks is atan(tan d / 15) in metres. A slip s along the dip
in blocks has a heave (horizontal part) of 300 s cos d metres and a throw (vertical part) of 20 s
sin d metres. Because the heave dominates, the displacements are near real ones in metres (normal
faults 0.017 to 0.029 of their length, thrusts 0.031 to 0.075), while the throws are a ninth to a
fifteenth of real ones.

## Earth against the mod

Measured on one run: `earthlike_quick_16k`, scattered continents, 500 Myr, seed 1, master 9e84fcf,
2026-10-02, on the test machine (`tools/review/wp8-runs.sh world base`, which runs `deeptime run`,
then `deeptime strata-check` and `deeptime structure-sections`); 6,108 folds and faults and 6,066
intrusions kept.

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| Fold wavelength | 15.8 ± 5.3 km (11 to 30 km) | Mouthereau et al. 2007, fig. 5 and §[55], full text | median 52.5 blocks = 15.8 km | base |
| Fold length along the axis | up to 150 to 200 km; often over 100 km | Alavi 2007 p. 1068, Mouthereau et al. 2007 §[7], full text | median 424 blocks = 127 km | base |
| Fold amplitude | 500 to 1,000 m (Zagros, maximum) | Mouthereau et al. 2007 art. p. 5, full text | median 5.1 blocks = 103 m in metres | base |
| Fold limb dips | open folds 20° to 35° (interlimb 110° to 140°), near the front about 5° (170°), tight in the inner belt | Alavi 2007 p. 1077, full text | 30° to 48° in blocks; 2° to 4° in metres | arithmetic on DEF-005 |
| Normal fault displacement over length | 0.03 (D = 0.03 L^1.06), typical 0.001 to 0.1 | Lathrop et al. 2022 pp. 2 and 7 (quoting Schlische et al. 1996), full text | 0.017 to 0.029 in metres; throw median 117 m for 38 km | base (median slip 6.6 blocks) |
| Thrust displacement over length | about 7 % | Elliott 1976 p. 297, full text | 0.031 to 0.075 in metres; throw median 82 m | base (median slip 8.2 blocks) |
| Normal fault dip | over 45°, about 60°; active ruptures 30° to 65° | Anderson 1905 pp. 389, 402; Waldron & Snyder 2020 p. 152; Collettini & Sibson 2001 abstract | 55° to 70° in blocks, 5° to 10° in metres | code |
| Thrust dip | under 45°, about 30°; peak 30° ± 5° | Anderson 1905; Collettini & Sibson 2001 abstract | 20° to 40° in blocks, 1° to 3° in metres | code |
| Fold-thrust belt width | 200 to 300 km (Zagros); 60 to 140 km (Subandean, Bolivia) | Alavi 2007 abstract; Mouthereau et al. 2007 §[7]; Hérail et al. pp. 179-180 (poor scan) | 250 km on the plate model's sphere, 102 blocks = 31 km on the map | code (DEF-027) |
| Bed offset at a structure | throws of about 1 km on 40 km faults (from the rows above) | derived | median 2 blocks (40 m), p90 7 (140 m), max 166 | base, `strata-check`, 20,000 columns |
| Dyke thickness | 2.3 m average (Ship Rock); 1 to 150 m, mean 30 m (Mackenzie giant swarm) | Delaney & Pollard 1981; Bunger et al. 2013, full text | 2 to 5 blocks, 600 to 1,500 m | code (DEF-048) |
| Dyke length | 1 to 9 km (Ship Rock); over 2,000 km (Mackenzie) | Delaney & Pollard 1981; Bunger et al. 2013 | median 50 blocks = 15 km | base |
| Dyke spacing in a swarm | 6.7 km to 25 km mean (Mackenzie), most continuous ones 35 to 65 km | Bunger et al. 2013 (quoting Hou et al. 2010, Stott & Josey 2009) | not measured (2,633 dykes kept) | base |
| Volcanic neck width | Ship Rock 500 m; small plugs under 30 m; Round Butte diatreme 170 to 190 m | New Mexico Bureau of Geology Ship Rock page; Delaney & Pollard 1981; Latutrie 2020 | 5 to 16 blocks, 1.5 to 4.8 km | code (DEF-049) |
| Stock | exposed over less than 100 km2 | Earle 2019 §3.5 | radius median 13.3 blocks = 4.0 km at the world's floor; outcrops a few per cent of that | base |
| Cratons with old deformation | the basement of every craton is metamorphosed crystalline rock; its sedimentary platform cover is weakly deformed | Earle 2019 §21.1-21.2; Wikipedia "Craton" (secondary) | 65 % of craton cells under a structure, 16 % under one younger than 50 Myr | base |

## Simplifications and limits

- **Folds and faults are drawn undistorted in blocks** while the strata, plutons and terrain are
  drawn in true metres (DEF-011). See the opening section and "Open questions" 1.
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

- Folds and faults drawn undistorted in blocks (DEF-011): an agent's design in P2-01 (build log
  2026-10-01), accepted by Ben as "true size" with the defaults on 2026-10-01; the metre
  equivalents are now put to him.
- Strength, count and size multipliers and the terrain switch: Ben, decisions log 2026-10-01
  (DEF-012, DEF-054, DEF-072).
- Dykes 2 to 5 blocks wide and plugs 5 to 16 blocks across, many times real ones, so that they can
  be seen (DEF-048, DEF-049); put to Ben.
- The fold profile is the exact sine where Noddy's default is a 21-point approximation of it
  (DEF-010).
- Every rate, overlap and age rule, the strain weights, the fold-train shape, the stock shape and
  depth and the relief rules: design values of the P2-01 agent (build log 2026-10-01, stages A and
  B), listed one by one in the ledger.

## Open questions

1. **Should folds and faults be drawn at the world's own vertical stretch?** (DEF-011; STATUS.md
   "Pending questions", with pictures under `review/out/wp8/`.) The same world was run three times
   with only this changed. Today (stretch 1): bed offsets median 2 blocks (p90 7), folds 5 blocks
   high, grade inversions in 0.07 % of columns. Stretch 4: offsets 7 (p90 26), folds 21 blocks,
   normal faults 80° to 85° and thrusts 56° to 74° in blocks, inversions 0.33 %. True metres
   (stretch 15): offsets 22 (p90 91), folds 77 blocks, faults near vertical in blocks, inversions
   0.78 %, and with 16 km folds over thin sedimentary cover the granite basement is pulled up in
   keels through the cover in arcs. Physics says true metres, but only together with wavelengths
   taken from the thickness of the layers being folded (Biot), which is not built; the relief term
   would also need lowering (with true metres it raised ground by up to 5.1 km).
2. **Intrusions larger than life** (DEF-048, DEF-049): dykes are 4 to 1,500 times real widths and
   plugs 3 to 10 times Ship Rock. The narrowest drawable (dykes 1 to 2 blocks, plugs 2 to 6 blocks)
   would be closer.
3. **Fold amplitude from layer thickness.** A design would take the folded cover's thickness H from
   the stack at the fold's place and age and draw L about 6.5 H (2 to 16), with the amplitude from
   the shortening in metres. Not built.
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
