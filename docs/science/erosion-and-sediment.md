# Erosion, sediment and sedimentary rock

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Through the whole history, rivers cut into rising land (the stream power law: erosion grows with the water a river carries and with its slope), hillsides creep downslope, rock weathers to soil, and the eroded material is carried off and laid down as sediment on land, in lakes and on the sea floor, where reefs also build limestone. Each deposit is recorded as a layer with a rock type chosen from where and how it was laid (its facies: for example sandstone near shore, shale offshore, coal from peat swamps). This is the stage that makes valleys, plains, deltas, continental shelves and the layered rock under them.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [ERO](ledger/ERO.md) | stream power erosion | ERO-001 to ERO-021 (21) |
| [ERO](ledger/ERO.md) | hillslope diffusion | ERO-022 to ERO-025, ERO-164 to ERO-167 (8) |
| [ERO](ledger/ERO.md) | sediment | ERO-026 to ERO-044 (19) |
| [ERO](ledger/ERO.md) | marine deposition | ERO-045 to ERO-056 (12) |
| [ERO](ledger/ERO.md) | carbonate | ERO-057 to ERO-067 (11) |
| [ERO](ledger/ERO.md) | weathering | ERO-068 to ERO-077 (10) |
| [ERO](ledger/ERO.md) | flow routing | ERO-078 to ERO-085 (8) |
| [ERO](ledger/ERO.md) | facies and lithology | ERO-086 to ERO-163 (78) |

Rows by status: `cited-unverified` 86, `from-memory` 2, `abstract-only` 3, `calibrated` 6, `game-choice` 6, `uncited` 64.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/ERO.md](extraction-notes/ERO.md).
- Research note: `docs/research/notes/surface_processes_climate.md`.
- The implementation follows Braun & Willett 2013 for the implicit stream-power solve; the research note read that paper's abstract only.
- Work package in the verification plan: WP-3 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The erodibility `k_spl` is 2e-9 in the Rust default and 3e-9 in all eight presets; the comment describes the older value (extraction notes ERO, flag 1).
- The weathering constants (50 m/Myr, depth scale 5 m) have no source; the research note says the rate constants "were not retrieved" (flag 2).
- The facies rules are, in the research note's words, "my proposal, not a published scheme that was checked".
- Stage 1 spot checks: the soil-production depth scale of 5 m is 10 to 20 times the 0.25 to 0.5 m that secondary sources give for Heimsath's law (ERO-070, MISMATCH at secondary level); the hillslope diffusivity of 2 m2/yr is 20 times the only primary value read (ERO-023); the Landlab carbonate and marine constants match Landlab's source.
