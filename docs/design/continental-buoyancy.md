# A continental column's height from its whole buoyancy: design

Status: **designed; being built behind `tectonics.column_buoyancy` (off)** (the low-continents lane's lead, the
coordinator's task of 2026-10-31; branch `continental-buoyancy` from master 0a3ada8c). Ledger PLT-830 to PLT-849.

## Why

The low-continents diagnosis (sea-level.md, "Why base35's continents stand low, against Earth (2026-10-31)") found
base35's continents low because Earth's volume of continental crust covers 0.48 of the planet instead of 0.38: the
larger area holds the sea 371 m over Earth's fill (+516 m of it the area of crust at least 20 km thick), and the same
volume spread wider is thin (31.8 km mean against 36.4; 0.27 of the planet at 30 to 35 km against Earth's 0.08, 0.012
at 40 to 50 km against 0.10). The area is large because no column can hold crust thicker than the column at the sea:
every continental column stands on PLT-281's one Airy line, z = z_0 + (T - 35.1 km) / 7.02 over the datum (z_0 525 m,
the middle of `continent_elevation_m`), which is Earth's at 32 to 34 km and stands 290 to 1,310 m above Earth's at 36
to 44 km, so thick crust stands high and erosion brings it down to the sea's column (33.6 km under base35's sea).
Earth holds its thick crust low on cold, keeled lithosphere and its thin crust high on hot lithosphere: in CRUST1.0
the Basin and Range and Cordillera stand 1,407 to 1,834 m on 30 to 45 km of crust and the cratons 140 to 660 m on 35
to 45 km, with alike crustal densities (m4cold.py); Hyndman & Currie 2011 put a hot back-arc at the sea on about 27
km and a craton on about 38 km, about 1,600 m apart at equal crust.

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
