# The coast with high ground on base33: realistic, artifact or missing physics? (2026-10-30)

Diagnosis only: no law changed, no simulation run. Ben asked whether the excess coastal high ground
has a physically backed, realistic explanation, or is missing physics or an unrealistic sim, and
asked for a Fable advisor to investigate (decisions log 2026-10-30). The coordinator ran the M5
note's breakdown on base33; the advisor, read-only, measured base33's cached maps and records. The
coordinator checked the advisor's central numbers against the run files (marked **checked** below);
the rest is the advisor's, as it reported it.

Runs: base33 (master 2abbc567, pool key 31c7c70bd7c7, `earthlike_quick_16k-1000/cd665cef`, both
starts, seeds 1 to 12, 24 runs; the reversal behind the overriding block, PLT-737, on). Earth:
SRTM15+ as in the M5 note (docs/research/notes/m5-coast-high-ground-2026-10-25.md).

## 1. Where the excess sits (the M5 note's definitions, the census grid; **checked**)

`tools/review/m5diag/` (m5fields.sh, m5maps.sh, m5table.sh with `M5DIAG_RUNS` on base33's folder;
the table at `/data/ban/deeptime-runs/m5diag/table-base33.txt` on the VM). Medians over the runs:

| | base33 | Earth (grid) |
|---|---|---|
| M5, coast with high ground | 0.1325 | 0.0840 |
| M4, land above 2 km | 0.0372 | 0.0552 |
| M5 / M4 | 3.66 | 1.52 |
| f_near (high ground near the coast) | 0.374 | 0.115 |
| land within 200 km of the coast | 0.335 | 0.292 |
| coast within 200 km of a trench: share / high / part of M5 | 0.171 / 0.415 / 0.073 | 0.122 / 0.274 / 0.034 |
| of that part: young / old collision, young / old Andean | 0.033 / 0.016, 0.009 / 0.003 | |
| coast within 200 km of a collision: share / high / part | 0.039 / 0.528 / 0.022 | 0.093 / 0.135 / 0.013 |
| other coast: share / high / part | 0.788 / 0.048 / 0.040 | 0.785 / 0.049 / 0.038 |

The passive coasts are Earth's; the active coast's total share is near Earth's; the model has less
high ground than Earth, and most of the excess (+0.040 of +0.049) sits at trench coasts whose high
ground is collision orogens, not Andean arcs. On base12 (the M5 note) the trench coast share was
0.244; it is 0.171 now.

## 2. The advisor's measurements (base33's maps 50 Myr before each run's end, its records)

Scripts: `/data/ban/deeptime-cache/advisor/` on the VM (adv_maps.py, adv_coll.py, adv_arc.py,
adv_run.sh, their outputs); not kept in the repo.

- **Sea level masks the excess** (**checked**, adv_maps-base33.txt): M5 at the run's own sea 0.126;
  100 m lower 0.143; 300 m lower 0.169; 600 m lower 0.212; at the datum 0.231; at the sea that gives
  Earth's land share 0.131. Earth at its sea +100 / +300 / +600 m: 0.080 / 0.106 / 0.138. Flooded
  continental lowland adds low coast; the coast cells that qualify barely change.
- **The high ground is coastal whatever the sea** (**checked**): the distance of land above 2 km to
  the sea, quartiles and p90, 151 / 255 / 411 / 586 km against Earth's 378 / 980 / 1,371 / 1,618 km;
  beyond 400 km 0.26 against 0.74.
- **Hypsometry** (**checked**): land in 0-0.5, 0.5-1, 1-2, 2-3, 3-4 and over 4 km, 0.571, 0.294,
  0.095, 0.025, 0.015, 0.0013 against Earth's 0.607, 0.195, 0.142, 0.027, 0.010, 0.0195: no
  Tibet-class band (p99 3.3 against 4.7 km), too much land at 0.5 to 1 km.
- **The crust a collision keeps stays on the overriding plate** (**checked**, supercontinent-1's
  `fields/globals.json`): `plates/collision_kept_km3` 1.073e10, of it `collision_wedge_km3` (the
  wedge on the incoming plate) 3.94e8, 3.7 %, and `collision_upper_km3` (the interior) 6.68e8.
- **Collisions** (rec.json, after the first 100 Myr): 55.7 collision states and 25.9 partings per
  100 Myr per run; duration quartiles 0 / 2 / 6 Myr, p90 24; terranes under 50 samples 0.54 of the
  states and 0.13 of the crossed area; 1,000 samples or more 0.07 of the states, 0.52 of the area,
  a median 42 Myr (the Earth-like ones). Shortening p50 160 km, p90 665 km. (Some "share" columns of
  the advisor's table count fields that are always present and read near 1.0; nothing below uses
  them.)
- **Plates born**: 233 a run, 0.32 back-arc slivers, 0.23 rift fragments, 0.21 fission, 0.18
  initiations.
- **Freeboard** (**checked** against docs/science/sea-level.md): the surface crosses the sea at
  35.1 km of crust against Earth's 31.0 km (CRUST1.0), so a normal-thickness continent stands under
  the sea and land is thickened crust.

## 3. The advisor's split of the excess (+0.049 over Earth; inferred, not measured)

1. **Realistic in kind, about +0.01 to 0.015**: belts at active margins (the Cordillera, Japan,
   Taiwan, Papua New Guinea); the trench coast share alone at Earth's rate gives +0.013, an upper
   bound (the model's settings are areas, Earth's lines).
2. **Artifact of where the orogen goes, about +0.02 to 0.025**: 96 % of the kept crust on the
   overriding plate, while Earth's doubly vergent orogens hold more of their mass in the pro-wedge
   on the lower plate (Willett, Beaumont & Fullsack 1993; Naylor & Sinclair 2008, as the ledger
   reads them); on a narrow overriding strip the belt reaches its back coast. Taiwan's upper plate,
   the Coastal Range, stands 0 to 1.68 km (Lai et al. 2022) against these coasts' median 3.4 km.
3. **Missing physics, about +0.01**: small terranes. A fragment under 50 samples docks for a median
   0 to 2 Myr and 0.95 of it is stacked as thickness (PLT-242's share, read for large sutures); on
   Earth such fragments accrete as area or subduct.
4. **The measure**: small (the three defects are fixed in v1census2).

Why the two-wedge arm (PLT-730, off) raised coast_hi, in the advisor's reading: the pro-wedge rises
out of the sea because the normal column stands under it, so the crest stands between two seas; a
freeboard symptom, not a refutation of Willett et al. 1993.

## 4. Other problems, by effect on the planet (the advisor's ranking)

1. No Tibet-class plateau (land over 4 km 0.0013 against 0.0195): large collisions shorten about
   600 km, the strength height stands about 4.5 km over the sea, and high plateaus wear at 63 m/Myr
   against Tibet's under 10 (erosion-and-sediment.md, ERO-174).
2. Freeboard (above).
3. Collision states lasting a step; 55.7 transfers and welds per 100 Myr (1.2 to 3.7 times
   Macdonald et al. 2019's 15 to 46 sutures per 100 Myr, RLC-073, a loose comparison; the advisor's
   82 added the 25.9 partings, which dock nothing: PR #48's review).
4. Continental arcs 1.1 to 1.3 km against the Andes' 4 km (magmatic addition without Andean
   shortening).

## 5. The order (Ben, 2026-10-30: "Freeboard, then orogen (Recommended)")

1. Freeboard: the sea budget, so a normal-thickness continent stands above the sea (Earth's
   shoreline at about 31 km of crust): sediment stock, floor age, mean continental thickness
   against Seton et al. 2020, GlobSed and CRUST1.0 as sea-level.md cites them.
2. The collision fill as a doubly vergent orogen with the pro-wedge on the incoming plate (the
   two-wedge arm again, on dry margins).
3. Small-terrane docking.

The advisor's expectation, stated before any of it is built: the first two likely raise the coast
with high ground before anything lowers it, because they take away the flooding that masks it.
