# A continental column's height from its whole buoyancy: design

Status: **designed; being built behind `tectonics.column_buoyancy` (off)** (the low-continents lane's lead, the
coordinator's task of 2026-10-31; branch `continental-buoyancy` from master 0a3ada8c). Ledger PLT-830 to PLT-849.

## Why

The low-continents diagnosis (sea-level.md, "Why base35's continents stand low, against Earth (2026-10-31)", revised
after PR #56's review) found base35's continents low because its crust at least 20 km thick covers 0.48 of the planet
instead of Earth's 0.38, with 10 % more volume than Earth's and 13 % thinner: the larger area holds the sea 292 m over
Earth's fill (means; +492 m of it the area, -68 m the steering's nudge), and the thin crust stands low. The area has
two sources of about equal weight:

- **The start** (the preset): every run starts with 0.4225 of the planet continental in a uniform 35.1 km column
  (PLT-131), 10 % over Earth's area with 6 % more volume (14.8 against 14.0 km planet-mean) and none of Earth's 0.117
  of the planet at 40 km and more. It carries about 0.4 of base35's excess area (0.7 of base34's); on Earth's
  histograms the start's area alone puts the sea 213 m over Earth's fill.
- **The run's thinning** (35.1 to 32.0 km mean), about 0.4: the start's crust is taken under at trenches and
  collisions (its volume kept by the crust budget) while new area grows on ocean floor as island-arc crust at 31.6 km,
  and the start's own crust thins to 33.5 km. Its cause is a hypothesis, not measured: every continental column stands
  on PLT-281's one Airy line, z = z_0 + (T - 35.1 km) / 7.02 over the datum (z_0 525 m, the middle of
  `continent_elevation_m`), which with the sea at the datum is near Earth's at 32 to 34 km and stands 400 to 1,320 m
  above Earth's ice-free crust at 36 to 44 km, so thick crust stands high and may wear to the column at the sea (33.6
  km under base35's sea). The kept volume of the crust budget is the rest (about 0.2 on base35).

Earth holds its thick crust low on cold, keeled lithosphere and its thin crust high on hot lithosphere: in CRUST1.0
the Basin and Range and Cordillera stand 1,407 to 1,834 m on 30 to 45 km of crust and the cratons 140 to 660 m on 35
to 45 km, with alike crustal densities (m4cold.py); Hyndman & Currie 2011 put a hot back-arc at the sea on about 27
km and a craton on about 38 km, about 1,600 m apart at equal crust.

**So the height law and the start's continents are one design.** A height law alone leaves the start's 0.4: on any
craton line a 35.1 km start column stands under a sea at the datum (about 400 m on Hyndman & Currie's, and -350 m on
this design's prototype with the start's L 225 km and K 200 km, `tools/review/buoyancy/law.py`, constants to be
sourced), so the law with the start as it is would drown the start further. This design therefore also sets the
start's continents from Earth's hypsometry and volume under the same law: their area (Earth's crust at least 20 km
thick covers 0.384 of the planet, 14.0 km planet-mean), their thickness distribution (mean 36.4 km, p10 26.6, p50
36.9, p90 44.5) and their lithosphere (cold keeled interiors, hotter margins), so that with the sea at the datum the
start stands where Earth's continents stand. The run's thinning is what the arm tests: if the hypothesis holds, cold
thick crust standing low under the law keeps its thickness and the same volume covers less area.

## Why thermal isostasy (ISO-110) is built and off, and whether that still holds

- **Built** 2026-10-23 with the lithosphere's thermal state (gap 4 stage 1, commits 63f99656 and cce71817; ISO-110 to
  ISO-112, lithosphere-thermal-state.md). Its form adds a term linear in the lithosphere's depth L to PLT-281's line,
  z_th = E (L_ref - L) / (L_cold - L_hot), E = 1,600 m (Hyndman & Currie 2011).
- **Off as an open fault, not a V1 matter** (lithosphere-thermal-state.md "On by default (2026-10-23)"; ISO-110's row):
  the add-on needs a frame, L_ref, the depth at which PLT-281's line stands. The craton frame (225 km) takes the line
  for the craton's, which holds only if the sea stands 938 m over the datum; Hyndman & Currie's frame reads L_ref from
  their two sea-level crusts and so moves with the sea, L_ref = 60 + (S + 629 m) x 165 / 1,600 km, feeding the sea's
  error into every column's height. The page's recommendation: "the physics, not a frame: give a continental column
  the height of its whole lithosphere's buoyancy (the crust's density, the mantle lithosphere's thermal contraction and
  the keel's depletion) against the ridge's column, as Lachenbruch & Morgan (1990) do ... in place of PLT-281's line, so
  the sea follows the columns' volume and no L_ref is needed".
- **Deferred before that** (decisions log 2026-10-20, the coordinator: per-column thermal isostasy "deferred, behind the
  orogen-extent and crust-loss work, since it would only lower the old fifth of the high ground"). That work has landed
  (the orogen's two wedges, PR #50; the crust budget, PR #53).
- **The M5 lane's note on the cold line** (2026-10-25, ISO-110's row): ISO-110 in the datum frame overshoots cold thick
  crust (a craton at 45 to 50 km 965 to 1,678 m against Earth's 340 m), because CRUST1.0's cold platforms rise only 1 to
  27 m per km of crust from 35-40 to 45-55 km against Airy's 142: "the crust past about 40 km acts as rock of about 3,200
  to 3,300 kg/m3". A thermal term alone cannot give that.

So the reason ISO-110 is off still holds for its form (a frame on a median line), and this design removes the frame
rather than choosing one: every column's height is its buoyancy against the ridge's column, whose depth the model
already carries (GDH1's 2,600 m, PLT-352). ISO-110 stays off with the new law on (the switches exclude each other).
The cold line's flatness is a separate term of the crust's own density (below, "The lower crust").
