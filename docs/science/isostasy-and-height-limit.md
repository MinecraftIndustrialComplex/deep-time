# Isostasy, subsidence, sea level and the height limit

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The crust floats on the denser mantle: thick or light crust stands high, thin or dense crust sits low, and loading or unloading it (with sediment, ice or erosion) makes it sink or rise. This is isostasy. Ocean floor also sinks as it cools with age. Mountains cannot grow without limit, because crust that stands too high spreads sideways under its own weight; in the mod that limit comes from a crustal strength and the planet's gravity instead of a fixed cap (6.8 km at Earth's gravity with the default strength). Sea level moves with the volume of land ice.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [ISO](ledger/ISO.md) | isostasy | ISO-001 to ISO-010 (10) |
| [ISO](ledger/ISO.md) | thermal subsidence | ISO-011 to ISO-017 (7) |
| [ISO](ledger/ISO.md) | crust spreading and the height limit | ISO-018 to ISO-032 (15) |
| [ISO](ledger/ISO.md) | sea level | ISO-033 to ISO-039 (7) |
| [ISO](ledger/ISO.md) | lithospheric flexure (its own page: [flexure.md](flexure.md)) | ISO-040 to ISO-069 (30) |
| [ISO](ledger/ISO.md) | the continent's settling on the plate model (written up in [plates.md](plates.md), "Mountains eat the world", 2026-10-20) | ISO-078 (1) |
| [ISO](ledger/ISO.md) | the sedimentary cover's pore water in the column's height (written up in [rivers-and-lakes.md](rivers-and-lakes.md), "Lakes against Earth", 2026-10-20) | ISO-080 to ISO-082 (3) |

Rows by status (70 active, 2 `merged`): `verified` 3, `cited-unverified` 9, `abstract-only` 8, `calibrated` 6, `game-choice` 14, `uncited` 30.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/ERO.md](extraction-notes/ERO.md).
- Session note: `docs/research/notes/session-mountain-height-limit.md`.
- Build log: STATUS.md 2026-10-01, "Terrain above the build limit, stage 1" and "The height limit, stage 2".
- Work package in the verification plan: WP-4 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- Three different crust densities are in use (2,650, 2,700 and 2,800 kg/m3) in different places, none cited (extraction notes ERO, flag 9).
- The shipped ocean floor is 0.3 to 0.6 km shallower than the GDH1 age-depth curve it is named after (flag 5).
- The sea floor could reach below the world's bottom (y -412 to -472 against a floor of y -352). Settled by Ben on 2026-10-02 ("Lower min_y to -512 (Recommended)"): the floor is y -512, under the plate model's nominal trench (y -437 at the default scale). Measured on the quick 16k planet: the finished sea floor reaches y -528, so the old floor cut 69,011 L3 cells flat (and 147,824 more sat in the bottom 24-block slide) where -512 cuts 33 (81 in the slide), up to 16 blocks deep. The bake records `lowest_y`, `cells_below_min_y` and `cells_in_floor_slide`. A bottom squeeze for settings deeper than the floor (the recommended option's net) is not built; it is a pending question.
- A sedimentary cover is porous and its pores hold water, so a metre of sediment weighs less than a metre of crust and floats less of its own thickness (ISO-080 to ISO-082, 2026-10-20). Before, the column's height counted the cover as crust at 2,830 kg/m3, so a closed basin filled with sediment only after about seven times its depth had come in; with Athy's porosity law (Sclater & Christie's shaly sandstone) it fills with about 3.6 times its depth on the normal column. The cover is the SEDIMENT and SEDIMENTARY pools together, compacting as one column under its own thickness, so new sediment on an old cover lands on rock already squeezed (a fresh metre nets 0.47 m of height on a bare column and 0.31 m on the normal cover).
