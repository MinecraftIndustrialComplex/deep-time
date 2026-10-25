# M5, the coast with high ground: why it reads three times Earth's (2026-10-25)

The coordinator's M5 diagnosis lane, diagnosis only. No law was changed and no simulation was run.
Everything below comes from the cached pool runs of base12: master 830e231a, pool key 4f5b17199a21, folder
`/data/ban/deeptime-runs/pool/5cf16ecd76cd/earthlike_quick_16k-1000/cd665cef/`, supercontinent
and scattered continents, seeds 1 to 12, 24 runs.

Statistics are medians over the 24 runs, with the IQR in brackets. The two starts are compared by
two-sided Mann-Whitney, and correlations are Spearman over the runs.

Earth is present-day SRTM15+ at 15' (Tozer et al. 2019, the grid of RLC-003). It is put through the
same code on three point sets:
- the census's own Lambert grid, 332,800 cells;
- Fibonacci points of the same count, as in PLT-360;
- 100,000 Fibonacci points, the crust budget maps' count.

Antarctica and Greenland are kept out of Earth's shares (cc's `sheets` box). This is the by-rock
stand-in. Earth's settings come from Bird's PB2002 steps (Bird 2003, PB2002_steps.dat; its readme
was read).

Scripts: `tools/review/m5diag/` (m5lib.py, m5fields.py, m5maps.py, m5earth.py, m5table.py and the
`.sh` runners). The tables are in
[`m5-coast-high-ground-2026-10-25-tables.txt`](m5-coast-high-ground-2026-10-25-tables.txt).

## The answer in short

**Headline.** M5 reads 0.248 [0.211, 0.306] against Earth's 0.084 on the census's own grid. M4 reads
0.128 against 0.055. The two starts do not differ (MW p 0.75 and 0.44).

**Earth's grid figure.** PLT-360's 0.076 is on Fibonacci points. On the census's Lambert grid,
Earth reads 0.084. Q5(a) gives the reason.

**M4's part.** Most of M5's excess goes with M4's level, but not all of it. Set the high-ground
level in each run so that the model has Earth's share of land above it (M4 = 0.055), and M5 reads
0.117 against Earth's 0.084. Raise Earth's level the other way, to the model's share (0.128), and
Earth reads 0.230 against the model's 0.248. So 80 % to 89 % of the excess (0.164) goes with M4's
level, and 11 % to 20 % (0.018 to 0.033) is M5's own.

**The 0.11 target.** With M4 at its own target of 0.07, M5 still reads 0.141. M5 passes 0.11 only
near M4 = 0.047.

**Which high ground goes matters.** 45 % of the model's high ground is under ice, and it lies
inland. Keeping iced land out takes M4 to 0.063 but M5 only to 0.226. If M4 is fixed by removing
inland or iced high ground, M5 stays where it is. M5's high ground is the young collision belts
at convergent coasts.

**Where the excess sits.** It is all at convergent coasts:

| coast setting | excess over Earth |
|---|---|
| within 200 km of a trench | +0.108 |
| within 200 km of a collision | +0.064 |
| elsewhere | +0.001 |

**Landmass geometry is Earth's.** Coast per land and land near the coast match Earth (Q3).

**Measure defects.** The measure has three defects. Fixed together, they take the model from 0.272
to 0.219 and Earth from 0.077 to 0.068, both on Fibonacci points. The model then reads 3.2 times
Earth, so the defects do not explain the excess.

## M5 = s · f_near · M4 / κ

The four factors:
- **κ** is coast points over land points.
- **f_near** is the share of the high ground within 200 km of the sea.
- **s** is qualified coast points per high point within 200 km of the sea.
- **M4** is the share of the land above 2 km.

The table gives medians, so the model's products do not multiply out exactly.

| | M5 | M4 | M5/M4 | f_near | κ | s |
|---|---|---|---|---|---|---|
| model, grid | 0.248 | 0.128 | 1.94 | 0.265 | 0.092 | 0.750 |
| Earth, grid | 0.084 | 0.055 | 1.52 | 0.115 | 0.098 | 1.295 |
| Earth, Fibonacci 332,800 | 0.078 | 0.055 | 1.41 | 0.107 | 0.098 | 1.286 |

The model's high ground is 2.3 times as often near the sea (f_near), but each near-sea high point
qualifies 0.58 times as much coast (s), because it stands in clumps at the shore (Q2). The two
nearly cancel: M5/M4 is 1.28 times Earth's.

At M4 = 0.055, f_near is 0.239 against 0.115 and s is 0.815 against 1.303. M5 at matched M4
follows f_near across the runs (rho +0.59, p 0.002), not κ (−0.07).

Across the runs, M5 tracks:
- M4: rho +0.68 (p 0.0003);
- the young collision part: rho +0.78;
- the trench coast share: rho +0.75 (p < 0.0001).

It does not track M5 at matched M4 (rho −0.04). The collisions per 100 Myr (median 1.60) correlate
with nothing: M5 +0.04, M4 +0.05, the young collision part −0.05, the trench coast share −0.05.

## Q1: the qualified coast by the origin of its nearest qualifying high ground

Classes come from the map nearest each cell (orogeny type and age, rock, kind, arc crust, flags)
and the field's regime. The map at 0 agrees with the fields' orogeny on 99.1 % of the high cells.
"Young" means a last orogeny stamp under 10 Myr.

The first two columns are shares of M5 and M4. The third is what M5 reads with that class's high
ground removed.

| class | share of M5 | share of M4 | M5 without it |
|---|---|---|---|
| collision, young | 0.460 | 0.496 | 0.175 |
| collision, old | 0.136 | 0.143 | 0.226 |
| Andean, young | 0.111 | 0.066 | 0.223 |
| Andean, old | 0.062 | 0.076 | 0.240 |
| no orogeny: other | 0.080 | 0.040 | 0.236 |
| no orogeny: ocean kind | 0.031 | 0.009 | 0.245 |
| no orogeny: arc or hotspot | 0.023 | 0.015 | 0.245 |
| no orogeny: thick, 50.5 km or more | 0.021 | 0.036 | 0.247 |
| no orogeny: parked at T_c | 0.012 | 0.037 | 0.247 |
| no orogeny: rift | 0.011 | 0.001 | 0.244 |

**Young collision by stamp age.** As absolute parts of M5: under 2 Myr 0.087, 2 to 5 Myr 0.018, 5 to
10 Myr 0.011. The belt and wedge are stamped every step while a collision runs (PLT-079), so most of
this is belts of collisions in progress at the final frame. 98.5 % of their qualifying high points
are in the collision regime.

**The other classes' settings:**
- Young Andean high ground stands on arc (0.50) and back-arc (0.42).
- "Other" stands on back-arc (0.38), passive margin (0.27) and craton (0.20).
- The parked untagged columns at T_c are 1.2 % of M5 and 3.7 % of M4. They are not a term for M5.

**M4 has the same order.** Young collision belts are half the land above 2 km.

**The coast by setting.** Model settings are the fields' regime: a trench (regime 2) within 200 km,
else a collision (5), else neither. Earth's are PB2002 steps: SUB, else CCB or OCB, else neither.

| setting | coast share | M5 inside | part of M5 |
|---|---|---|---|
| trench within 200 km, model | 0.244 | 0.570 | 0.141 |
| trench within 200 km, Earth | 0.122 | 0.274 | 0.034 |
| collision within 200 km, model | 0.159 | 0.517 | 0.076 |
| collision within 200 km, Earth | 0.093 | 0.135 | 0.013 |
| neither, model | 0.600 | 0.061 | 0.039 |
| neither, Earth | 0.785 | 0.049 | 0.038 |

**Trench coasts.** The model's 0.141 is mostly young collision high ground (0.081), with old collision
0.019 and young Andean 0.015. So the model's trench coasts carry collision belts behind them, not
only the Andean cordillera.

**Caveat on the settings.** The model's settings are areas (the regime's cells) and Earth's are
lines (the steps), so the model's "within 200 km" reaches somewhat farther. Its shares by setting
are upper bounds against Earth's. The ratio of M5 inside does not depend on that.

## Q2: distance from the qualified coast to its nearest qualifying high point

Shares of the qualified coast, with the qualifying high point's own distance to the sea below them:

| | self (the coast point is high) | 0-50 km | 50-100 | 100-150 | 150-200 |
|---|---|---|---|---|---|
| model | 0.157 | 0.160 | 0.270 | 0.208 | 0.186 |
| Earth, grid | 0.027 | 0.100 | 0.281 | 0.288 | 0.304 |
| model, young collision | 0.239 | 0.162 | 0.258 | 0.181 | 0.149 |
| model, old collision | 0.177 | 0.183 | 0.260 | 0.187 | 0.174 |
| model, young Andean | 0.018 | 0.085 | 0.249 | 0.335 | 0.301 |

| qualifying high point's own distance to the sea | 50 km or less | 50-100 | 100-200 | 200-400 |
|---|---|---|---|---|
| model | 0.167 | 0.470 | 0.332 | 0.022 |
| Earth, grid | 0.044 | 0.346 | 0.536 | 0.075 |

Earth's qualifying high ground stands 100 to 200 km behind the shore. The model's collision belts
reach the shore itself: a sixth of the qualified coast is itself above 2 km, against 3 % on Earth.

The young Andean cordillera has Earth's profile, and Earth's is mostly Andean.

## Q3: coast length and the land near the coast

| | κ (coast / land) | coast length L_100, km per Mkm² of land | land within 50 / 100 / 200 km of the sea |
|---|---|---|---|
| model, grid | 0.092 | 1,524 | 0.062 / 0.152 / 0.303 |
| Earth, grid | 0.098 | 1,565 | 0.069 / 0.157 / 0.292 |
| Earth, Fibonacci 332,800 | 0.098 | 1,668 | 0.088 / 0.167 / 0.293 |

L_100 is (land within 100 km of the sea) / 100 km. The landmasses are as indented and as wide as
Earth's, at the same resolution. Two consequences:
- The narrow landmasses of 2026-10-21 (plates.md "The coastal high ground": 0.45 to 0.56 of the land
  within 200 km of the sea) are gone.
- No part of M5's excess is the coast's geometry. κ and land_within_200 do not track M5 (rho −0.19
  and −0.13).

**Published figure, read.** OECD (2021), *Adapting to a changing climate in the management of
coastal zones*, OECD Publishing (the PDF, read through pdftotext), says that coastal zones "represent
only 20% of global land surface (United Nations, 2017)". Its footnote 3: "This estimate is based on a
definition of coastal zone that includes coastal land up to 100 kilometres from the shoreline."

SRTM15+ through our code reads 0.157 (grid) and 0.167 (Fibonacci) within 100 km. A shoreline at full
resolution, longer than the 39 km grid's, is the likely reason for that gap (not measured). The
model's 0.152 sits with Earth at the same resolution.

**No published figure for 200 km was found and read.** The 0.292 is our own SRTM15+ computation.

## Q4: through the run

The crust budget's maps are 100,000 Fibonacci points and the plate elevation before the surface
stage. The surface stage adds 0.020 to M5 at the end (plate_z M5 0.228 on the grid). Earth on the
same 100,000 points reads M5 0.0715, M4 0.056, M5/M4 1.27, and M5 0.071 at M4 = 0.055.

| Myr | −700 | −500 | −400 | −300 | −200 | −150 | −100 | −50 | 0 |
|---|---|---|---|---|---|---|---|---|---|
| M5 | 0.230 | 0.225 | 0.214 | 0.257 | 0.214 | 0.203 | 0.219 | 0.225 | 0.265 |
| M4 | 0.139 | 0.164 | 0.147 | 0.153 | 0.132 | 0.131 | 0.138 | 0.119 | 0.136 |
| M5/M4 | 1.75 | 1.41 | 1.47 | 1.66 | 1.61 | 1.84 | 1.69 | 1.70 | 2.01 |
| M5 at M4 = 0.055 | 0.106 | 0.098 | 0.089 | 0.096 | 0.097 | 0.108 | 0.107 | 0.118 | 0.128 |
| young collision share of M5 | 0.52 | 0.52 | 0.51 | 0.55 | 0.52 | 0.57 | 0.53 | 0.63 | 0.52 |

M5 is high through the run: about three times Earth's from −700 Myr on. At −900 it reads 0.161,
with M4 at 0.018 before the belts are built.

The part from M4 is there all along. M5's own part grows late. At matched M4, M5 is 0.089 to 0.098
from −500 to −200 Myr, 1.25 to 1.4 times Earth's. It is then 0.107 to 0.118 from −150 to −50 Myr (1.5
to 1.7 times) and 0.128 at the end (1.8 times).

Young collision belts carry about half of M5 at every time. The map at 0 agrees with the fields
(rho 0.95 over the runs, map / field 1.03).

## Q5: the measure

**(a) The Lambert grid against Fibonacci points: a defect.** The census's grid is equal-area
cylindrical, 1024 × 328, with a spacing of 39.1 km and the coast band 1.6 spacings = 62.6 km.
Above about 51° its north-south step exceeds the band, which has two effects:
- Coast facing north or south drops out of the coast. At 60° to 70°, coast per land reads 0.082 on
  the grid against 0.117 on Fibonacci points for Earth.
- Landmasses split into row strips: 357 components by the census's joining rule against 74 when
  joined 8-way on the grid.

The model reads 1.06 times higher on Fibonacci points (0.272) and Earth 0.92 times (0.078). So the
grid's M5 and PLT-360's Earth figure are on different point sets. **Fix:** resample the census to
Fibonacci points by the nearest cell before coast_metrics, as PLT-360's Earth control does. Not
applied.

**(b) The seam: no defect.** No column is a copy of another. The coast and land in the seam's
columns (0 to 3, 1020 to 1023) are within the spread of the rest (coast / mean 0.66 to 0.93, the
mid-planet column 512 0.81), and land flips across the seam are 0.64 of the mean. Nothing is counted
twice.

**(c) Lakes counted as sea: a defect.** Closed sea bodies under 1 Mkm² are 0.4 % of the sea's area
(Earth 0.4 %), but their shores are 10.8 % of the model's coast, and M5 on them is 0.50. Counted as
land:
- the model's M5 reads 0.214 and its M4 0.127;
- Earth's reads 0.076 on the grid.

These closed bodies lie among the belts: half of their shore has high ground within 200 km. A closed
basin below the datum is a lake or a dry depression, not the sea.

**Fix:** count as sea only bodies of 1 Mkm² or more. On Earth's grid that turns two bodies of 0.46
and 0.68 Mkm² into land (sizes of the Caspian and the Black Sea; not identified) and keeps one of
2.49 Mkm² (the Mediterranean's size) as sea. Not applied.

**(d) Ice as rock: no defect in M5; a comparison question for M4.** The census's z is the rock,
exactly (elevation less the surface: 0 m at the median), and the ice is its own field. But 45 % of
the model's high ground has ice on it, while Earth's 0.055 keeps its two ice sheets out by region.

| model, iced land kept out | M4 | M5 |
|---|---|---|
| ice of 1 m or more | 0.063 | 0.226 |
| ice of 100 m or more | 0.071 | 0.230 |

So the iced high ground is inland and barely touches M5. This is the M4 and M6 lanes' question
(like for like: keep iced land out on both sides, or compare bedrock with bedrock, CRUST1.0's
0.047). It is not a fix for M5.

**(e) The straight-line reach: a defect.** 8.8 % of the qualified coast reaches its high ground by a
straight line that crosses the sea (Earth 3.4 %), across a bay or to a facing shore of the same
landmass. Dropped, the model reads 0.224 and Earth 0.081. **Fix:** an over-land reach (the distance
through land cells, or at least a line that stays on land). Not applied.

**The three fixes together.** Fibonacci points, then bodies under 1 Mkm² as land, then no crossing
reach:

| | Fibonacci | + lakes as land | + no crossing |
|---|---|---|---|
| model | 0.272 | 0.239 | 0.219 |
| Earth, no sheets | 0.077 | 0.071 | 0.068 |

The model stays 3.2 times Earth's. M4 is unchanged by the fixes (0.129 against 0.055).

**The 1.6-spacing band against Earth's control.** The band is a resolution choice, and it is the
same on both sides once both are on one point set. On the grid, Earth's control passes the 0.11
target only with M4 at about 0.055.

## The terms behind M5's excess, ranked

The total excess on the grid is 0.164: 0.248 against 0.084.

1. **Young collision belts beside the sea.** These are the stamps under 10 Myr: 0.087 of them are
   in the last plate step, and 98.5 % are in the collision regime. They make 0.114 of M5 (46 %) and
   half of M4. Without them M5 reads 0.175; other high ground still qualifies part of that coast.
   Their high ground reaches the shore (self 0.24).
   - Most of it is within 200 km of a trench (0.081) or of a collision (0.038).
   - Front lane: the collapse and crust stamps (what the belt's stamp builds, how wide, and how close
     to the open side it stands), trace A.
   - The lead: the belt's profile, PLT-243, is `game-choice`. Its uplift is 1 − (d/W)², highest at
     the front, so a front near a coast puts its highest ground at the shore.
   - Fix (sourced physics, a lead, not run): build the belt's surface from the critical taper the
     wedge already uses (PLT-244, Davis, Suppe & Dahlen 1983 `abstract-only`). PLT-192's surface
     slope is 1° to 1.6° on the Lesser Himalayan wedge (DeCelles & Carrapa 2021, read for PLT-192).
     At 1.5° the ground reaches 2 km about 76 km behind the deformation front (computed:
     2 km / tan 1.5°).
   - The earlier finding stands: the crossing per km of front is Earth's, and what is 2.4 to 4.8
     times Earth's is the length of front in collision (plates.md "The crust flux into collision
     fronts").
   - Force balance lane: not indicated. The collision count does not track M5 (rho +0.04).
2. **Too much of the coast is within 200 km of a trench, and high ground is behind it twice as
   often.** The coast share is 0.244 against 0.122. M5 inside is 0.570 against 0.274. The part is
   0.141 against 0.034, so +0.108. Across the runs the trench coast share tracks M5 (rho +0.75).
   - Front lane, subduction margins: where trenches stand against continents. plates.md "The plate
     geometry" found subduction on 34 % of the boundary against Bird's 19.7 %.
   - The high ground behind these trenches is mostly collision belt, so item 1's fix acts here too.
   - Force balance lane: nothing here measures plate speed, so no overlap is shown.
3. **High ground at the shore itself.** Self is 0.157 against 0.027. The qualifying high point is
   within 50 km of the sea 0.167 of the time against 0.044. This is the profile side of item 1 and
   the reason s is low.
   - Front lane: the stamp's profile (PLT-243, item 1).
   - On Earth, 0.59 of the qualified coast reaches its high ground at 100 to 200 km. The model's
     young Andean cordillera does as well (0.64). The young collision belts reach only 0.33, so
     the arc law is not the term.
4. **Closed bodies counted as sea.** −0.034 on the model, −0.008 on Earth. Measure fix (c).
5. **The straight-line reach.** Up to −0.024 on the model, −0.003 on Earth. Measure fix (e).
6. **The grid against Fibonacci points.** It raises the model by 0.024 and lowers Earth by 0.006.
   Measure fix (a); the like-for-like Earth target on the census's own grid is 0.084, not 0.076.

Not terms:
- the landmass geometry (Q3);
- the parked T_c columns (1.2 % of M5);
- the iced high ground (it moves M4, not M5);
- the seam.

## Errors found in docs/science/realism-checks.md (not edited)

1. Section "M5 coast with high ground" says "the coast band is 181 km wide at 40,000 points where the
   census's is about 25 km". The census's band is 1.6 × 39.1 km = 62.6 km, and the pipeline's is
   1.6 × 112.9 km = 181 km. The 16 km in the "Resolution" caveat ("a grid 111 km at the equator
   where the census's is about 16 km") reads like r = 16 blocks; the census's spacing is 39.1 km.
2. The same section's "0.104 by surface at the census's 332,800 points (PLT-360)" is on Fibonacci
   points of the census's count (PLT-360's own row says so). On the census's Lambert grid Earth reads
   0.093 by surface and 0.084 without the sheets.

## Traps

- **The map at 0 (`maps/t0000.bin`) has no flags.** It is a fresh lattice with exactly 100,000
  samples, and every flag is 0 except FORELAND 128. Trench, arc and front flags are only on the
  earlier maps, so read the fields' regime for the final frame.
- **scattered_continents-5's dump has no `plates/collision_mm_yr`.**
- **`plates/collision_mm_yr` is above 0 only on a few high points** (median 0). It marks the front,
  not the belt.
- **PLT-360's Earth figures are on Fibonacci points**, not the census's grid. Compare like with like.

## Commands (VM; tools/review/m5diag/)

- `tools/remote/mac.sh --kind light -- tools/remote/detach.sh start m5fields3 --kind light tools/review/m5diag/m5fields.sh 12 fields3`
  writes `/data/ban/deeptime-runs/m5diag/fields3.jsonl`. It covers the fields of each run's final
  dump at the census's L1 grid.
- `... detach.sh start m5maps --kind light tools/review/m5diag/m5maps.sh` writes `maps.jsonl`, the crust
  budget's maps at ten times.
- `... detach.sh start m5earth4 --kind light tools/review/m5diag/m5earth.sh` writes `earth.json`. It
  reads SRTM15+ 15' at `/data/ban/deeptime-runs/m5diag/earth/earth_relief_15m_p.grd` and Bird
  2003's PB2002_steps.dat at `/data/ban/deeptime-runs/m5diag/earth/PB2002_steps.dat.txt` (its first
  argument overrides that path; downloaded data, not committed).
- `tools/remote/mac.sh --kind light -- tools/review/m5diag/m5table.sh fields3 maps` writes
  `/data/ban/deeptime-runs/m5diag/table.txt`, committed as the tables file above.
