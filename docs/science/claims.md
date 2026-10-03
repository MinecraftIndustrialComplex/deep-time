# The claims ledger

One row for every scientific claim the code makes: every physical constant, empirical law,
scaling relation, threshold, rate, calibration target and "Earth is X" comparison, in Rust and Java
code, in preset defaults and in data files. A preset default counts: it is a claim about what an
Earth-like planet is.

The ledger was built on 2026-10-01 and 2026-10-02 (stage 1) by reading the code. **Stage 1 checked
almost nothing against sources and changed no code.** Its purpose is to make the unchecked visible,
row by row, so that stage 2 can check each row and fix what is wrong.

The rows are in `ledger/`, one file per subsystem, because one file would be too large to open.

## Counts

2,275 active rows in 29 ledgers (P1-10 added PRV-028 to PRV-031 and re-marked PRV rows, 2026-10-02; plate-motion task 2 added PLT-222 to PLT-227 the same day; force-balance task 5 added the ledger FRC, 27 rows, and CHK-058 and CHK-059; the table is recounted by tools/review/claims_counts.py and the prose further down may lag it). The ledgers hold 2,433 ids: stage 1 left 2,069 rows; stage 2 package WP-0
(2026-10-02) retired 193 of them as duplicates (`merged`, not counted) and added 301 rows for code stage 1
had not covered (51 in GEO, 74 in PRS, 84 in HNT, 35 in CRE, 57 in CHK). The flexure feature (ISO-040 to
ISO-067, added to master the same day by its own agent, and ISO-068, ISO-069 from its follow-up) is included.

| Ledger | Subject | Rows | `verified` | `cited-unverified` | `from-memory` | `abstract-only` | `calibrated` | `game-choice` | `uncited` | of which `pending-rewrite` | of which `not-yet-coded` | `merged` (not counted) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| [AMP](ledger/AMP.md) | the amplifier | 73 | 0 | 8 | 0 | 0 | 1 | 55 | 9 | 0 | 0 | 0 |
| [BAK](ledger/BAK.md) | the bake, satellite view | 42 | 0 | 0 | 0 | 0 | 1 | 36 | 5 | 0 | 0 | 0 |
| [BIO](ledger/BIO.md) | Köppen classes and biomes | 103 | 24 | 2 | 0 | 3 | 1 | 72 | 1 | 0 | 0 | 0 |
| [CHK](ledger/CHK.md) | plate-check target ranges | 59 | 19 | 3 | 1 | 7 | 0 | 11 | 18 | 3 | 4 | 0 |
| [CLM](ledger/CLM.md) | climate physics | 138 | 23 | 24 | 2 | 0 | 14 | 25 | 50 | 106 | 32 | 12 |
| [CRE](ledger/CRE.md) | core engine: plate geometry, noise, plate state, deformation lattice | 35 | 0 | 8 | 0 | 0 | 0 | 0 | 27 | 1 | 0 | 0 |
| [DEF](ledger/DEF.md) | folds, faults, intrusions | 74 | 17 | 0 | 0 | 1 | 0 | 56 | 0 | 0 | 0 | 0 |
| [DEP](ledger/DEP.md) | deposit models, counts, sizes | 165 | 54 | 2 | 0 | 20 | 0 | 62 | 27 | 0 | 0 | 40 |
| [ERO](ledger/ERO.md) | erosion, sediment, facies | 159 | 0 | 83 | 2 | 4 | 6 | 6 | 58 | 0 | 0 | 11 |
| [FRC](ledger/FRC.md) | plate forces (the force balance, recorded) | 27 | 3 | 0 | 10 | 1 | 0 | 13 | 0 | 0 | 0 | 0 |
| [GEO](ledger/GEO.md) | geothermal features | 61 | 4 | 0 | 0 | 0 | 0 | 54 | 3 | 0 | 0 | 0 |
| [GLA](ledger/GLA.md) | glacial erosion and landforms | 64 | 0 | 20 | 1 | 0 | 1 | 25 | 17 | 0 | 0 | 2 |
| [HNT](ledger/HNT.md) | editor hint texts | 93 | 14 | 12 | 2 | 4 | 7 | 39 | 15 | 0 | 0 | 0 |
| [HYD](ledger/HYD.md) | rivers, lakes, runoff | 75 | 0 | 14 | 3 | 0 | 1 | 36 | 21 | 0 | 0 | 1 |
| [HYP](ledger/HYP.md) | hypsometry and calibration targets | 34 | 0 | 10 | 0 | 1 | 18 | 1 | 4 | 0 | 0 | 0 |
| [IMP](ledger/IMP.md) | meteor impacts | 135 | 43 | 9 | 6 | 6 | 0 | 56 | 15 | 0 | 0 | 6 |
| [ISO](ledger/ISO.md) | isostasy, flexure, subsidence, height limit, sea level | 67 | 3 | 8 | 0 | 6 | 6 | 14 | 30 | 0 | 0 | 2 |
| [MAG](ledger/MAG.md) | magma and lava | 57 | 2 | 1 | 0 | 1 | 0 | 52 | 1 | 0 | 0 | 0 |
| [MOI](ledger/MOI.md) | moisture, winds, seasons | 70 | 0 | 17 | 0 | 0 | 0 | 8 | 45 | 70 | 0 | 6 |
| [ORE](ledger/ORE.md) | deposit formation, coal, petroleum, placers | 191 | 25 | 17 | 0 | 17 | 0 | 63 | 69 | 0 | 0 | 75 |
| [PLT](ledger/PLT.md) | plate tectonics | 209 | 17 | 45 | 4 | 15 | 12 | 70 | 46 | 162 | 15 | 20 |
| [POL](ledger/POL.md) | the pole cap | 21 | 0 | 0 | 0 | 0 | 0 | 18 | 3 | 0 | 0 | 0 |
| [PRS](ledger/PRS.md) | presets other than earthlike_quick_16k; test and world presets | 74 | 0 | 9 | 0 | 1 | 2 | 53 | 9 | 15 | 0 | 0 |
| [PRV](ledger/PRV.md) | geologic provinces | 31 | 0 | 0 | 0 | 5 | 4 | 10 | 12 | 0 | 0 | 0 |
| [RCK](ledger/RCK.md) | rock classes, metamorphism, plutons | 79 | 11 | 8 | 0 | 1 | 1 | 55 | 3 | 0 | 0 | 11 |
| [SCL](ledger/SCL.md) | world scale, planet constants | 50 | 0 | 6 | 0 | 0 | 1 | 37 | 6 | 0 | 0 | 1 |
| [STR](ledger/STR.md) | strata, traps, hosts | 46 | 1 | 0 | 0 | 1 | 2 | 42 | 0 | 0 | 0 | 4 |
| [VOL](ledger/VOL.md) | volcanoes | 74 | 0 | 0 | 0 | 1 | 2 | 47 | 24 | 0 | 0 | 0 |
| [YNG](ledger/YNG.md) | the young landscape | 19 | 0 | 0 | 0 | 0 | 0 | 9 | 10 | 0 | 0 | 2 |
| **all** | | **2,325** | **260** | **306** | **31** | **95** | **80** | **1,025** | **528** | **357** | **51** | **193** |

72 rows are `verified` (counted from the ledger files after WP-2 merged WP-15/16 and the plate-check boundary mix, 2026-10-02, WP-13a, the volcano decisions and WP-8): PLT-213 and PLT-218 (plate-check boundary mix and arc heights), ISO-041, ISO-042
and ISO-048 (the flexure agent), MAG-028 and MAG-029 (WP-2), one GEO row and thirteen HNT rows (WP-15, WP-16, WP-2 and the volcano decisions),
15 PLT and 19 CHK rows (WP-13a: the force-balance constants and the plate-check targets) and 17 DEF rows (WP-8: folds,
faults and the Noddy port).
In plain terms, of the 2,231 active rows: 881 (39 %) are deliberate game or implementation choices; 601 (27 %)
state something about the physical world with no source at all; 475 (21 %) name a source that nobody is recorded
as having read; 123 (6 %) were written from memory or from an abstract and say so; 79 (4 %) were tuned to a target;
72 (3 %) were read in their sources.

A low `from-memory` count does not mean a subsystem was written with its sources open. It means
its agent did not say. The impacts agent was candid, and so has 34 such rows; most `cited-unverified`
and `uncited` rows elsewhere were very probably written the same way.

## How to read a row

| Column | Meaning |
|---|---|
| id | `PREFIX-NNN`. Stable: an id is never reused. New rows take the next number. Code comments refer to it (`// ledger: IMP-012`). |
| feature | The feature within the subsystem. |
| what the code does | One sentence in plain words. |
| where | File and symbol. No line numbers: they rot. |
| value or equation as coded | Read from the code, not from a paper or a document. |
| units | SI or the code's own unit. |
| source cited | The citation exactly as written, or `none cited`. |
| citation written in | Code comment, build log entry, design document, research note, or plan section. For a `game-choice`, who decided and when. |
| status | One mark from the table below, then `pending-rewrite` or `not-yet-coded` where they apply, then any note. |

| Status | Meaning |
|---|---|
| `verified` | The source was fetched and the value read in it. The row says what was read, where, by whom and when. |
| `cited-unverified` | A source is named. Nothing records that anyone read the value in it. |
| `from-memory` | The code, the build log or a document says the value was written from memory or could not be checked. |
| `abstract-only` | Only the abstract or a secondary summary was read. |
| `calibrated` | Tuned until a run matched a target. The row names the target and the source of the target's Earth figure. |
| `game-choice` | Deliberately not physical. The row says why and who decided: Ben (with the date in `docs/deeptime-design-decisions.md`) or an agent (with the build-log entry). |
| `uncited` | No source anywhere. |
| `merged` | Retired in stage 2 (WP-0) as a duplicate of another row. The row keeps its id and its `where`, says which row owns the claim, and is not counted. Verify the owner, not the retired row. |
| `pending-rewrite` | Added to rows of today's plate motion and today's banded climate, which are about to be replaced. Stage 2 does not verify these. |
| `not-yet-coded` | The constant exists only in an approved design. Verify it before it is built. |

Rules for duplicates (WP-0). One claim, one row: when two rows state the same parameter, law or threshold
at the same place in the code or data, one row owns it and the other is retired with `merged` and a pointer.
The owner is the row that states the claim most completely; a composite row that only lists parameters with
their own rows is retired into those rows; a `pending-rewrite` row never owns a claim that a non-pending row
also states; ties go to the lower id number. Ids are never renumbered or reused. A retired row's extra
information (a citation, a flag) is added to the owner's text, marked "Absorbed in stage 2". Statuses were
not changed by merging; where the two rows disagreed, the disagreement is listed under "Duplicate rows
merged in WP-0" below for the verifying package to settle.

Rules for changing the ledger (see `CLAUDE.md`, "Science documentation is part of done"):

- A row becomes `verified` only when someone has fetched the source and read the value. Write
  what was read: "Collins et al. 2005, eq. 21, p. 823, full text, 2026-10-05".
- If the source gives a different value, the code is corrected, with a test, and the row records
  the old value, the new value and the source.
- If no source can be found, the row stays `uncited` and its feature page says so. A value may be
  kept without a source; it may not be presented as established.
- When a value changes in code, its row changes in the same commit.

## Supporting files

- [bibliography.md](bibliography.md): the sources, one entry per work, with full references, how each has been read, and whether the project's source library (kept off the repository, not distributed) holds a readable copy and under which file name.
- [spot-checks.md](spot-checks.md): what the stage 1 bibliography pass found when it tried to read
  403 values in their sources: 176 matched, 134 matched in part, 19 did not match, 74 could not be
  read. Results there are leads for stage 2, with the passage quoted and the level of evidence; none
  of them changed a status.
- `extraction-notes/`: per subsystem, the measured Earth-against-mod numbers in the build log,
  the stated simplifications, and flags (contradictions between comment, code and document).
- [extraction-notes/preset-coverage.md](extraction-notes/preset-coverage.md): every key of the
  default preset with its value and owning subsystem, and how the other presets differ.

## Known gaps in the ledger itself

What WP-0 (2026-10-02) closed: the duplicate rows (see "Duplicate rows merged in WP-0" below), the geothermal
features (GEO), the presets other than `earthlike_quick_16k`, the test presets and the in-game world presets
(PRS), the editor hint texts (HNT), `plates/geometry.rs`, `noise.rs`, `state.rs` and
`structure/deform/lattice.rs` (CRE) and the `plate-check` target ranges (CHK).

Still open:

- **Not read line by line:** test modules; `plates/dynamics.rs`; parts of `bake_horizons.rs`; the
  internals of the diffusion solver and flow routing; the Java twins of Rust code; `surface/detail.rs`
  beyond its constants. The four shipped essentials files
  (`neoforge/src/main/resources/data/deeptime/essentials/*.json`) have no row (flag 36 below).
- **Hint texts:** HNT does not repeat the hints already ledgered in PLT, HYD and IMP (PLT-203 to
  PLT-208, HYD-061, HYD-065, IMP-016, IMP-022, IMP-071, IMP-083); hints that only restate a default are
  not rows.
- **Validation ranges** (the bounds a setting is allowed to take) are recorded inside the value
  cell, not as rows of their own.
- **Rows with status disagreements** left by merging are listed under "Status disagreements" below.
- Rows were extracted by Sonnet sub-agents and checked for form, counts and private details by the
  stage 1 lead and by the WP-0 lead. A wrong "as coded" value is possible; the code wins.

## Duplicate rows merged in WP-0

Stage 1's agents did not read each other's rows, so the same claim was ledgered twice. WP-0 compared 216
candidate pairs (found by shared code symbols and by text similarity), had them judged by four Sonnet
agents, reconciled the four sets of decisions (the DEP/ORE and ERO/RCK families were judged twice and
disagreed in six places; the lead settled each by keeping the granular rows) and retired **193 rows**
under the rules in "How to read a row". Ids were not renumbered; each retired row has status `merged`,
keeps its `where`, and says which row owns the claim. The retired row's extra content (a citation, a flag)
was appended to the owner's text as "Absorbed in stage 2".

By ledger pair (the retired row's ledger, then the ledger of the row that now owns the claim; pairs with
fewer than three rows are grouped):

| Retired in | Owner in | Rows | What |
|---|---|---|---|
| ORE | DEP (or DEP and ORE) | 61 | the deposit data files described twice: coal, petroleum, BIF, MVT, sedex, porphyry, supergene, placer sizes, geometry and zoning, host weights, size and richness multipliers |
| DEP | ORE (or DEP) | 40 | the same families, retired the other way where the ORE row was the granular one |
| PLT | ISO | 12 | the ocean age-depth law (PLT-092, PLT-094 to PLT-096), oceanic heat flow, isostatic factor, crust strength, flow and strength height, uplift scale |
| ORE | STR | 10 | trap closure scores written in both the structure and the deposit ledgers |
| CLM | SCL | 6 | planet constants (axial tilt, star flux, day length, gravity, star age, snapshot interval) |
| PLT | HYP | 5 | target land share inputs: ocean fraction, shelf share, continent elevation, arc build-up, uplift width |
| RCK | ERO | 5 | rock-class erodibility and diffusivity |
| CLM | GLA | 4 | the equilibrium-line rule: CLM-113, CLM-114, CLM-116, CLM-117 into GLA-036 to GLA-039 |
| ERO, IMP | ORE | 4 + 4 | fields and sizing inputs shared with the deposit ledger |
| PLT | SCL | 3 | model radius, time step, continental share cap |
| RCK | MAG | 3 | magma chamber depth, lava-lake share, which volcanoes get a chamber |
| all other pairs | | 36 | planet and scale constants (IMP-131, MOI-049, ISO-029, ISO-030, ...), the lapse rate (MOI-051 into GLA-038), RCK composites into ERO and IMP, spawn rules, and singletons |

The four families that `claims.md` named as duplicated, in the table: the ocean age-depth law (PLT into
ISO), rock erodibility (RCK into ERO and IMP), coal and petroleum (DEP and ORE, both directions) and the
equilibrium-line rule (CLM and MOI into GLA; HYD-039 stays because it re-states one figure of the
relation, not the fit).

Three composite rows were retired into many owners: RCK-007 and RCK-011 (diffusivity and erodibility per
rock class) into the 54 per-class rows of ERO and IMP, and RCK-008 into the loose-sediment rows.

### Status disagreements between a retired row and its owner

Merging did not change any status. Where the two rows disagreed, the owner keeps its status and the
verifying package settles it:

| Retired | Owner | Disagreement |
|---|---|---|
| CLM-114 | GLA-037 | CLM-114 said `cited-unverified`; GLA-037 is `from-memory` (STATUS 2026-10-01 admits Ohmura et al. 1992 was cited from memory). |
| MOI-051 | GLA-038 | MOI-051 was `cited-unverified`, GLA-038 `uncited`; both rest on "the standard atmosphere's" 6.5 degC/km with no document named. |
| MOI-073 | CLM-079 | MOI-073 cited Berner and Kothavala 2001 (fix 24); CLM-079 says none cited. |
| CLM-081, MOI-074 | SCL-038 | They cite plan section 8.3; SCL-038 says none cited. |
| ORE-036 | DEP-026 | `abstract-only` against `cited-unverified` for the same Cooke et al. 2005 citation. |
| ORE-099 | DEP-041 | `abstract-only` against `cited-unverified` for Sillitoe 2005 and Sillitoe and McKee 1996. |
| ORE-071 | DEP-044 | ORE-071 cites SIR 2010-5070-B p. 25; DEP-044 says none cited and is `game-choice`. |
| ORE-043 | IMP-105, IMP-106 | ORE-043 is `cited-unverified` (Grieve and Masaitis 1994; Naldrett 2004); IMP-105 says none cited for the thresholds, IMP-106 carries the citations. |
| IMP-107, IMP-117 | ORE-044, ORE-041 | `game-choice` (thresholds are design) against `cited-unverified` with the same citation. |
| DEP-118, DEP-142 | ORE-119 to ORE-124, ORE-163 | The retired rows named sources (USGS PP 1625-A, Averitt 1975, OFR 2011-1296; OFR 2003-253 p. 8); the owners say none cited. |
| GLA-034 | ERO-016 | GLA-034 carried `game-choice` with provenance; ERO-016 is `uncited`. |
| ERO-151, ERO-152 | IMP-085, IMP-086, IMP-089, IMP-090 | ERO rows cited the four-paper scheme (Sklar and Dietrich 2001; Stock and Montgomery 1999; Roering et al. 1999; Heimsath et al. 1997) as `cited-unverified`; the IMP rows call the value `uncited`. |
| RCK-005, RCK-007, RCK-009, RCK-011, RCK-012 | ERO-101 to ERO-163 | RCK rows said `uncited` ("judgment, relative to granite"); the ERO per-class rows carry the scheme citations as `cited-unverified`, and ERO-102 and ERO-139 call the reference 1.0 a normalisation (`game-choice`). |
| PLT-034 | HYP-015 | PLT-034 cites Cortial (r_s 1,800 km) for the default; HYP-015 says none cited. |
| PLT-123 | ISO-020 | PLT-123 listed design references that exist only in the build log; the owner's "none cited" stays true for the code. |

## Verification plan for stage 2

### Found in stage 1: fix first

These came out of the spot checks and the extraction flags. None was fixed (stage 1 changes no
code). "Evidence" says how strong the finding is.

| # | Ledger | What is wrong | Evidence |
|---|---|---|---|
| 1 | IMP-042 | Complex crater depth is coded 0.294 D^0.301 km and cites Collins et al. 2005, eq. 28. That equation is 0.4 D^0.3 km. Craters over 3.2 km are about 27 % too shallow. | primary full text, re-read by the stage 1 lead **Fixed in stage 2 (WP-1, 2026-10-02):** code, comment or hint corrected, with a test where a value changed; see [ledger/IMP.md](ledger/IMP.md). |
| 2 | ORE-001 and following; `surface/fields.rs`, `surface/mod.rs` | The time windows are not applied on real histories: the banded-iron, oxygenated-ocean and land-plant clocks are hard-coded. No BIF ever forms; coal and sediment-hosted zinc-lead are not limited in time. | code reading by the extraction agent; the deposit audit found no BIF on either planet |
| 3 | DEP (the `richness_mult` setting) | The richness setting is documented as scaling grade, and grade never reaches a block. | deposit audit, build log 2026-10-01 |
| 4 | BIO-002 | Köppen: the code tests the polar group before the dry group. Peel et al. 2007 says "the B climates must be identified first". Cold dry land is classed tundra or ice cap instead of desert or steppe. **FIXED in WP-11 (2026-10-02), with a test.** | primary full text (sub-agent) |
| 5 | HYD-030 | Holdridge biotemperature counts temperatures above 30 °C as zero as well as those below 0 °C. The code applies only the lower limit, with Holdridge's coefficient. Evaporation is overestimated in hot deserts. | primary full text, scan (sub-agent) |
| 6 | ERO-070 | Soil production depth scale is 5 m. Secondary sources give 0.25 to 0.5 m for Heimsath's law. A possible factor of 10 to 20. | secondary only; Heimsath et al. 1997 not readable |
| 7 | ERO-022, ERO-049 | The comment says diffusivity between two cells is averaged "as Landlab's LinearDiffuser". Landlab takes the maximum. | Landlab source (sub-agent) |
| 8 | ERO-003; `SurfaceParams::default` | `k_spl` is 2e-9 in the Rust default and its comment, 3e-9 in all eight presets. | code reading |
| 9 | IMP-018 | Editor hint: "one crater of 20 km or more per 180 million km² per year". The coded rate is one per 1.8 × 10¹⁴ km² per year: a factor of a million. | arithmetic **Fixed in stage 2 (WP-1, 2026-10-02):** code, comment or hint corrected, with a test where a value changed; see [ledger/IMP.md](ledger/IMP.md). |
| 10 | IMP-038, IMP-054, IMP-062 | Equation numbers of Collins et al. 2005 cited in comments are wrong (rim height is eq. 48, ejecta eq. 47, melt eq. 30). Values are right. | primary full text (sub-agent) **Fixed in stage 2 (WP-1, 2026-10-02):** code, comment or hint corrected, with a test where a value changed; see [ledger/IMP.md](ledger/IMP.md). |
| 11 | VOL-039 | The 60 km spacing of arc volcanoes cites de Bremond d'Ars et al. 1995, whose abstract says spacing is random with no characteristic value. | primary abstract **Settled in WP-2 (2026-10-02):** the citation now quotes the abstract; the mean spacings are from Hildreth 2007 (27 to 87 km); 60 km is labelled a game choice. |
| 12 | VOL-016, VOL-026, VOL-029, VOL-048 | Comments disagree with the code they sit on (flank angle, crater size, caldera "across" against radius, subsidence rate). | arithmetic by the extraction agent **Settled in WP-2 (2026-10-02):** comments rewritten from the arithmetic and the sources read; the caldera comment's numbers were radii, so flag 4's reading was wrong; the subsidence rate was a question for Ben, answered 2026-10-02: the default is now 0 (it counted flexure and cooling twice). |
| 13 | DEF-016 to DEF-018 | Fault slip ratios are coded 0.05, 0.08, 0.05 beside a cited 0.03. | secondary (a review quoting Schlische) **Settled in WP-8 (2026-10-02):** not an error. The ratios are slips along the dip in block space; their horizontal part is true in metres, so in metres the displacements are 0.017 to 0.029 of the length for normal faults and 0.031 to 0.075 for thrusts, against 0.03 (Schlische, via Lathrop et al. 2022, read in full) and 7 % (Elliott 1976, read in full). What is short is the throw, which is flag 14's question. |
| 14 | DEF-019, DEF-020 | Fault dips from Anderson 1951 are applied in block space, which is stretched 15 times vertically: real dips become about 6.6° and 2.2°. A design question, not a typo. | arithmetic from the build log's own formula **WP-8 (2026-10-02):** the angles are verified (Anderson 1905, read in full; Collettini & Sibson 2001, abstract). Not a units error: the code says on purpose that folds and faults are drawn undistorted in blocks (DEF-011). Physics favours true metres; put to Ben with sections drawn three ways. **Settled 2026-10-02** (Ben: "Setting, default 4x"): a setting, default 4x on the 20 m block, held as the share of true metres `true_height_share` 4/15 so it survives a change of block height (dips 21° to 36° and 5.5° to 12.6° in metres at any block height; 80° to 85° and 55° to 73° in blocks of 20 m), the scarp term lowered to match; true metres once fold wavelengths follow layer thickness (DEF-011). |
| 15 | DEF-042, RCK-061 | A "stock" is coded larger than the definition quoted beside it; the pluton thickness law cited to Cruden 1998 is not in that paper's abstract. | secondary; primary abstract **DEF-042 settled in WP-8 (2026-10-02):** no contradiction. The definition is about the exposed area (Earle 2019, read); the coded radius is the body's half-width at the world's floor, and a stock's outcrop is only its cupola, a few per cent of that. The comment is corrected; no value changed. RCK-061 is WP-7's. **RCK-061 settled in WP-7 (2026-10-02):** T = 0.6 D^0.6 km is the fit of Cruden and McCaffrey 2001 (read in Cruden 2008, SKB R-08-138), with D the diameter of the circle of equal area; Cruden 1998 gives no equation. The code now uses D instead of the long axis. |
| 16 | HYP-010 | The ocean's most common depth is stated as 3 to 4 km; NOAA gives about 4.3 km and the project's own Earth table peaks in the 4 to 5 km band. | agency page (sub-agent) |
| 17 | CLM (ice thickness) | Land ice uses a hard-coded gravity of 9.81 m/s² and ignores the planet's gravity setting. In code that is `pending-rewrite`; carry the fix into the new climate. | code reading **WP-14a (2026-10-02):** written into the per-cell design (section 4.6, task T6) with a test; the unmerged ice branch's `sheet.rs` imports the same constant, so T6 fixes both. No code changed now (the code is about to be replaced and the branch is not merged). |
| 18 | PLT, plate-check and the force-balance design | "Slab-attached plates move 3 to 4 times faster" is attributed to Forsyth & Uyeda 1975; the statement found is in Conrad & Lithgow-Bertelloni 2004. Correct the attribution before the force balance is calibrated to it. | primary abstracts **Settled in WP-13a (2026-10-02):** Forsyth & Uyeda 1975 was read in full. They give slab-attached plates at 6-9 cm/yr and the others at 0-4 cm/yr (p. 178), and their Table 1 gives a ratio of means of 3.8; the words "~4 times faster" are Conrad & Lithgow-Bertelloni 2004's. The designs, PLT-162 and CHK-056 now say so. |
| 19 | SCL (`pole_cap_deg`), preset | The preset's pole-cap latitude is read by nothing; the game uses its own value from the world preset file. | code reading |
| 20 | SCL-039 and the timeline range | A run may be 4,600 Myr long while the planet's age defaults to 4.57 Gyr. | arithmetic |
| 21 | DEP (MVT and sedex time gate) | The 2.02 Ga limit comes from Leach et al. 2010 in Economic Geology, not from the USGS report of the same authors and year that is also cited; that paper does not support putting MVT deposits on the same clock as sedex. Moot until fix 2. | primary abstract; **Stage 2 (WP-9/10):** the ledger now names the Economic Geology paper (ORE-014); the shared gate is unchanged and listed as an open question. |
| 22 | DEP (supergene copper) | The data file says Sillitoe 2005's "3 to 9 Myr" was not found; the abstract has it ("at least 0.5 m.y. but typically minima of 3 to 9 m.y."). The engine uses only the 0.5. The 100 mm/yr rainfall floor is stricter than the sources. | primary abstract; **Stage 2 (WP-9/10): data note corrected**; the 100 mm/yr floor and the 0.5 Myr minimum are open questions on [deposits.md](deposits.md). |
| 23 | DEP ("Circular 1178 p. 15") | The 1 km assessment depth is on printed page 5. The rule itself is right. | primary full text (sub-agent); **Stage 2 (WP-9/10): corrected** in the porphyry type note and the ledger. |
| 24 | MOI-073, CLM (weathering) | The runoff sensitivity 0.038 per K is GEOCARB II's (Berner 1994); the cited Berner & Kothavala 2001 uses 0.045 and 0.025. | primary full text (sub-agent) |
| 25 | GLA-002 | Glacial erosion constant 1e-5 with exponent 1; the published pairs are 1e-4 with exponent 1 and 1e-5 with exponent 2. Glacial erosion may be ten times too slow, or the exponent wrong. | primary abstract and a citing paper |
| 26 | AMP-034, AMP-038 | The thermal slope values attributed to Schott et al. are read differently from Schott's shader: 0.9 and 1.4 there are absolute tangents, not multipliers of 0.57. | Schott's shader source (sub-agent) |

### Found in WP-0 while reading code stage 1 had not covered

Read from code and documents by the extraction agents; nothing here was checked against a source, and none was fixed (WP-0 changes no code). They continue the numbering above.

| # | Ledger | What is wrong or unsettled | Evidence |
|---|---|---|---|
| 27 | HNT-017, HNT-019 (and the comment in `volcanoes/params.rs`) | The hints say blocks are "15 times taller than wide". It is the other way round: a block is 300 m wide and 20 m tall, so heights are drawn 15 times larger against widths. | arithmetic from the scale rows **Fixed in WP-15: the hints and the code comments now say heights are stretched 15 times against widths.** **WP-2 (2026-10-02):** in the volcano files (`params.rs`, `shape.rs`, `Magma.java`) and the volcano_flanks hint the merged text is WP-2's "heights are drawn 15 times larger than widths (a block is 300 m across and 20 m tall)". |
| 28 | HNT-057 | The ice-sheet hint says 50 kPa "matches Antarctica". `ice.rs` and CLM-102 give about 32 kPa for Antarctica and about 80 kPa for Greenland. | code comment against hint **Fixed in WP-15: the hint now says 32 kPa fits Antarctica and 80 kPa Greenland, and that the default 50 was chosen to meet targets.** |
| 29 | HNT-062, HNT-063 | The caldera hints give Crater Lake's radius as about 4 km (the code comment says 4 to 5 km across) and Yellowstone's as 20 km, while the preset's largest caldera radius is 9 km. | code against hint **Fixed in WP-15 (hints): Crater Lake 8 to 10 km across (radius 4 to 5 km), Yellowstone 48 by 72 km across; the preset's 9 km maximum is a design range (WP-2).** **WP-2 (2026-10-02):** the caldera_radius_max hint now uses the primary USGS figure, Yellowstone a basin 45 by 85 km (Christiansen 2001), with Long Valley 16 by 32 km. |
| 30 | HNT (volcano_size, shield_height_high, volcano_density, arc_spacing) | Shield heights: hints say "up to 6 km" and "Hawaii: 5 to 9 km", the coded range is 3.5 to 6.5 km. Arc spacing "real arcs: 50 to 70 km" repeats the 60 km that VOL-039 shows is not in the cited paper. | code against hint; fix 11 **Fixed in WP-15 (hints): shield and stratovolcano heights and Mauna Loa stated; "Earth's spacing" removed. The spacing figure itself stays with fix 11 (WP-2).** **WP-2 (2026-10-02):** the volcano_size hint adds Earth's median stratovolcano height (1.02 km, Grosse et al. 2014); arc_spacing gives Hildreth 2007's mean spacings of large centres, 27 to 87 km. |
| 31 | HNT (ejecta_reach, feature_deposits, star_age, ocean_fraction) | "1/27 at 3 radii" while the coded taper is zero there; "1 is the true deposit size" while each deposit type enlarges its body 6 to 12 times (and richness reaches no blocks, fix 3); "a younger star is fainter" while the Gough law is normalised to 1 at the end of the history; the ocean-fraction slider minimum 0.31 is not the real bound and the presets ship 0.7, not the 0.71 the hint calls Earth. | code against hint **Fixed in WP-15 (hints): ejecta taper, deposit drawing size, star age and ocean fraction now say what the code does. The richness setting still reaches no block (fix 3, WP-9).** |
| 32 | HNT (river_width, river_depth, pet_per_degree, day_length, lake_freeze, thrust_slip) | Unit and wording problems: "blocks" for quantities that are blocks per root of discharge; PET per degree without "per year"; "one rotation" for a solar day; "ice more than half the year" where a symmetric cycle gives exactly half; "about 11 blocks" is the maximum of the slip law. | arithmetic **Fixed in WP-15: units and wording corrected.** |
| 33 | PLT-204 to PLT-208 (hints, already ledgered) | `cordillera_width_km` is a half-width in the code but the hint treats it as the width of the range; the "Abyssal plain depth" label sets `oceanic_floor_elevation_m`, not the unused `abyssal_elevation_m`; the hint uses 180 Myr for two different things. | code against hint **Part fixed in WP-15: the arc_range_width hint now says the setting is a gaussian scale. The other plate hints wait for the plate rewrite (WP-13).** |
| 34 | PRS-022, PRS-035 to PRS-042 | `planet_world_4k.json` says it uses "the calibrated parameters of pipeline_quick_16k" but twelve keys differ (older tectonic and erosion values kept on purpose, not stated in the preset). The 4,096-block test planet is about 196 km in radius and has Earth's gravity (PRS-029). | preset diff |
| 35 | PRS-001 to PRS-003, HYP-019 | The full presets take uplift times 0.8 on top of the quick presets' times 0.7; HYP-019 describes one factor of 0.7. The 32k presets reuse the 16k tuning; only `quick_32k` was run, once. | preset diff; build log |
| 36 | PRS-057 to PRS-059 | The four shipped essentials files (`neoforge/src/main/resources/data/deeptime/essentials/*.json`: iron, coal, copper, gold) have no ledger row, and the test essentials differ from them in region size, exposure and spawn limit. | grep of the ledgers |
| 37 | PRS-063, PRS-067 to PRS-069 | The circumference, the floor (-512; -352 until 2026-10-02), the top density slide (624 to 640), the lava-lake anchor and height are written as literals in the world preset, the noise settings, the dimension type and the Rust `scale`; no check ties the copies together. | code reading |
| 38 | CHK-056, CHK-025 to CHK-027 | The plate-check text says "Earth: 3 to 4" for slab-attached plates with no author; the baseline design adds Forsyth and Uyeda; the force-balance design itself attributes "about four times" to Conrad and Lithgow-Bertelloni 2002 and 2004. This is fix 18 seen from the check. | design documents **Settled in WP-13a:** as fix 18; CHK-025 to CHK-027 now carry Earth figures read or computed from the sources (3.0 to 4; 0.71; -0.34). |
| 39 | CHK (rift window, land gain, active-margin mean) | Hard-coded literals disagree with the labels: the rift-born plate measures use 50.0 instead of `rift_window_myr`; the land gain uses 10.0 instead of `start_up_myr`; the active-margin mean uses 90.0 under a label that says 100 Myr (also PLT-160). | code reading **Part settled in WP-13a:** the active-margin label now says 90 Myr, as the code and its doc comment do. The two literals (50.0 for `rift_window_myr` in `rift_alive_50_pct` and `rift_welded_back`; 10.0 for `start_up_myr` in the land gain) equal their parameters' defaults and were left: changing them only matters under `--param`, and the force-balance task 5 is editing the same file. Open for that task or a follow-up. |
| 40 | CHK-019, CHK-055, CHK-021, CHK-036, CHK-037, CHK-039, CHK-040 | Targets that do not match their Earth figure: a 230 Myr ceiling written for the end-of-run ocean-floor age is applied to a run mean of per-step 90th percentiles; "at most 20 mm/yr" for the 10th percentile is below the 26 to 28 mm/yr the design gives for the slowest group; a period ceiling of 600 Myr beside an Earth range of 300 to 500; a target defined at 20 Myr after onset on an Earth figure "in 15 Myr"; two targets restricted in the design to "a belt above 3 km" carry no such condition. | design against code **WP-13a:** CHK-021 settled (today's plates' 10th percentile is 11 mm/yr; the 26-28 were category medians); CHK-036 changed to 400-800 Myr (Mitchell et al. 2021); CHK-037 settled (0.27 over 15 Myr, and India's rate has stayed low since); CHK-019 and CHK-055 stand (Earth's 90th percentile is 133 Myr, the ceiling 230: question in STATUS.md); CHK-039 and CHK-040's "belt above 3 km" is for task 3 to build into the measure. |
| 41 | PLT-143 to PLT-161, CHK-001 to CHK-019 | The collision targets are all marked `pending-rewrite`, but the force-balance design (section 7) says they stand as the acceptance test of the rewrite. | design against ledger **Settled in WP-13a:** `pending-rewrite` removed from PLT-143 to PLT-161. |
| 42 | CRE-014 | `pool::CONSERVED = 5` is documented as the "felsic-family" pools but covers pools 0 to 4, which include sediment, sedimentary and metamorphic rock. | code reading |
| 43 | CRE-034 | The plan says the column rule and the Noddy port agree to 1e-5 block; the code counts a point as a side flip above 1e-3 block when it lies within 1e-6 of a fault plane and leaves flipped points out of the worst case. | plan against code |
| 44 | GEO-014, GEO-048 to GEO-050, GEO-017, GEO-035, GEO-046 | Geothermal hints and format notes disagree with the code: "one to four" fields per volcano against a formula that gives 0 to 7; "2 to 5 blocks across" is the pool radius (4 to 10 across); FORMAT.md says a tenth of a terrace is stained (code: 7 %) and a field is drawn from 3 blocks above sea level (code: 5). | code against documents **Fixed in WP-16: hints, FORMAT.md and comments now agree with the code.** |
| 45 | GEO-012, GEO-026, GEO-040, GEO-041 | The plan says heat comes "from the heat flow already modelled"; the code uses only active volcanoes. Nothing in code, hints or decisions mentions geysers, boiling or temperatures, and spring water is plain water; the plan's epithermal cinnabar is a 6 % stain. | code against plan **Part fixed in WP-16: geysers, heat sources and cinnabar are now stated plainly in the hints, FORMAT.md and the page; the plan's wording is a decision for Ben (Pending).** **Ben, 2026-10-02 (decisions log, "Export heat flow"): the bake now exports the plate model's heat flow and fields also stand on hot continental ground (GEO-052 to GEO-056).** |
| 46 | GEO-007, GEO-019, GEO-020 | The 400 mm rainfall threshold has no source; the 3 degC rule is a vanilla freezing rule, not geology. | none cited **Part fixed in WP-16: the 400 mm and 3 degC rules are marked `game-choice` with the qualitative sources found (NPS, Paulson and Ingebritsen 1991); no source gives either number.** **Ben, 2026-10-02 (decisions log, "Keep all three"): the 400 mm rain test, no pools under 3 degC and pools of 4 to 10 blocks stay as owner-confirmed game choices.** |
| 47 | SCL-009 (absorbing ISO-030) | Sea level 63 is called an agent choice in SCL-009 and Ben's design choice in the retired ISO-030, whose text SCL-009 now carries. | ledger rows |
| 48 | master `STATUS.md` | The P2-14 build-log entry lost its heading and first paragraphs in the merge to master; its remaining bullets sit at the end of the P1-10 entry. | read by the GEO extraction agent **Fixed in WP-16: the heading and first bullets are restored (reconstructed; the original text survives nowhere in the history).** |

Further mismatches and partial results, with quotations, are in [spot-checks.md](spot-checks.md);
further flags are in `extraction-notes/`.

### Flags known before stage 1, and where each now stands

| Flag | Where it stands |
|---|---|
| The impacts agent wrote the Collins, Melosh & Marcus 2005 constants from memory | The paper was read. Ten constants match; the complex-crater depth law does not (fix 1); three equation numbers are wrong (fix 10). 34 IMP rows stay `from-memory` until stage 2 confirms each. |
| It doubted the complex-crater depth law (0.294 D^0.301 km against 0.4 D^0.3) | The doubt was right: the paper has 0.4 D^0.3. |
| It fitted the ocean rule to an abstract | Davison & Collins 2007 was read in full. The three numbers match; the paper's own limit is a depth-to-diameter ratio of about 8, against the code's fitted 6.3 (IMP-065 to IMP-067 stay `abstract-only`). |
| The force-balance design could not confirm the gross size of slab pull | **WP-13a:** of order 10¹³ N/m. Faccenna et al. 2012's eq. 1 (the Turcotte & Schubert law with Carlson et al.'s heating factor, read in full) gives 1.4 × 10¹³ N/m for 80 Myr floor and 0.7–2.0 × 10¹³ for 20–150 Myr (computed); the design's "1–3 × 10¹³ gross" was from memory and is replaced (PLT-183). Turcotte & Schubert itself was not read. |
| ... a factor of 8 for continental drag | **WP-13a:** confirmed in the body, which is free on the publisher's site: drag under the continents "is approximately eight times as strong as the mantle drag under the oceanic portion of the plates" (p. 189, Table 6). The design's `keel_drag` 3 follows Conrad & Lithgow-Bertelloni 2006's 2–5 (read in full) and is unchanged (PLT-188). |
| ... a wedge-taper reference | **WP-13a:** the 1.5° is the wedge's surface slope, not its taper (surface slope plus basal dip). The Nepal Himalaya's frontal (Lesser Himalayan) wedge has surface slopes of 1–1.6° and a total taper of 6–7.5° (DeCelles & Carrapa 2021, read in full); the secondary "tapers from 2.5°" are total tapers. Davis, Suppe & Dahlen 1983's abstract supports a wedge that grows at constant taper. The value stands; the setting should be named a slope (PLT-192). |
| A river-sinuosity comparison in the grid audit is from memory | Still unsourced. Leopold & Wolman 1957 give 1.5 as the conventional threshold for a meandering river; nothing read supports "commonly 1.1 to 1.5 and over" (HYD-027, WP-5). |
| The accuracy review marks some Earth figures as from memory | Its list is quoted in [extraction-notes/DOC.md](extraction-notes/DOC.md). Checked so far: continental crust about 41 % of area (Cogley 1984, abstract: matches); Hawaii–Emperor 6,200 km and 81 Myr (secondary: matches); mean land height about 800 m (the older figure; unresolved); the others unread (WP-15). |

### Risk ranking: the unverified values that most shape the worlds

Ranked by how much a wrong value would change what a player sees, among code that is not about to
be rewritten.

1. **Stream power erosion: K, m, n** (ERO-003 to ERO-008). Tuned, with no external source. They set
   all relief, every valley and the land's height. Tied to the hypsometry targets, so any change
   means recalibration.
2. **Hillslope diffusivity and soil production** (ERO-023: 2 m²/yr, uncited, 20 times the one
   primary value read; ERO-070: see fix 6). They set how smooth the land is.
3. **The ocean age-depth law** (ISO-011 to ISO-017; duplicated in PLT). The shipped law is 0.3 to
   0.6 km shallower than the GDH1 curve it is named after; Stein & Stein 1992 could not be read.
   It sets ocean depth, sea level and land fraction.
4. **The height limit** (ISO-018 to ISO-032): a crustal strength of 140 MPa with no source gives
   the 6.8 km ceiling for mountains and volcanoes.
5. **Crust and mantle densities** (ISO-001, ISO-008, ISO-009, PLT-121 and following): 2,650, 2,700
   and 2,800 kg/m³ are used in different places. One literature value read is 2,830.
6. **The hypsometry targets themselves** (HYP-001 to HYP-021): the Earth figures everything is
   tuned to (land 0.289, median land height 427 m, land above 2 km 0.102) come from the project's
   own reading of a relief grid whose resolution is stated three ways. Earth's mean land height
   and the 427 m median were not confirmed.
7. **Deposit counts for coal, petroleum and the families without a density model** (DEP-117 to
   DEP-144, ORE-108 to ORE-206, and the placeholder densities): coal field sizes and densities are
   an agent's judgement on basin areas marked "NOT verified" (one of which, the Illinois Basin, is
   less than half a secondary source's figure); petroleum field size is one Niger Delta unit applied
   everywhere; about 25 sizing inputs are placeholders. These set how much coal and oil a world has.
   By contrast the metal families' grade and tonnage tables were read in stage 1 and match the USGS
   reports.
8. **Glaciation** (GLA-037, CLM-117; GLA-002): the equilibrium-line rule is used outside its data
   and with a different temperature than its source defines; the erosion constant is paired with
   the wrong exponent. They set how much of the cold land is ice-carved.
9. **Volcano sizes and rates** (VOL-016 to VOL-052): all design values. Volcanoes raise the share
   of high land far above Earth's.
10. **The impact rate and the crater diffusivity** (IMP-001, IMP-003, IMP-074): the rate's source is
    unread; the diffusivity decides how long craters last.

In code about to be rewritten, to be carried into the new designs: the CO₂ forcing fit used a
hundred times beyond its stated range; snowball escape at 0.55 bar against a published 0.01 to
0.1 bar; the energy-balance constants as a mix of two climlab classes and a paper that could not be
read; the weathering runoff coefficient from an older GEOCARB than the one cited. And in the plate
design: gross slab pull, the factor of 8 for continental drag, and the wedge taper, none confirmed
until WP-13a, which settled all three (see the flags above); its open item is the slab suction share,
which is not the quantity its sources give.

### Work packages

Row counts are the active rows of the ledgers named, after de-duplication (WP-0). "Sources readable" are the works cited by those rows that the source library now holds as text or as a scan, "not readable" the rest (the detailed lists are in "Sources in the library, by work package" below). Suggested model follows the project rule: Sonnet 5.5 for
well-specified checking and fixing, Opus where a wrong value forces a design or calibration decision.

| WP | Ledgers | Rows now | Sources readable (text / scan) | Sources not readable | Model | Notes |
|---|---|---|---|---|---|---|
| WP-0 | all | – | – | – | Sonnet 5.5 | **Done** (2026-10-02): the source library, the de-duplicated bibliography and ledger, the stage 1 coverage gaps (PRS, HNT, CRE, CHK, GEO). This table is its output. |
| WP-1 | IMP | 135 | 5 / 6 | 33 | Sonnet 5.5 | Fix 1, 9, 10 with tests (HNT-018 is the hint for fix 9). Add a comparison with the Earth Impact Database. Wood and Head 1976, Pike 1980, Moore 1974 and Gault and Wedekind 1978 are page scans with machine OCR text; Grieve and Cintala 1992 could not be obtained. |
| WP-2 | VOL, MAG | 115 | 0 / 0 | 1 | Opus | Nearly all uncited: the work is finding sources and deciding which design values to keep as `game-choice`. Fix 11, 12, 30. |
| WP-3 | ERO | 156 | 3 / 0 | 11 | Opus | Fix 6, 7, 8. Any change needs recalibration: run with WP-4, after the surface-stage work now in flight lands. ERO now owns the rock-class erodibility rows that RCK also stated. |
| WP-4 | ISO, HYP | 99 | 7 / 0 | 26 | Opus | Fix 16. Settle one crust density. Re-derive Earth's hypsometry figures from one named dataset. ISO now owns the ocean age-depth law that PLT also stated. |
| WP-5 | HYD, YNG | 94 | 12 / 0 | 8 | Sonnet 5.5 | Fix 5. Unify the two runoff laws. Replace the from-memory sinuosity figure. |
| WP-6 | AMP, GLA | 135 | 8 / 0 | 17 | Sonnet 5.5 | AMP is mostly `game-choice`: confirm each is labelled so. GLA: the erosion law's constant and exponent (fix 25). GLA now owns the equilibrium-line rule that CLM also stated. |
| WP-7 | RCK, STR | 117 | 1 / 0 | 16 | Sonnet 5.5 | Fix 15. 41 rock densities and all erodibilities are unsourced judgement: label them. STR now owns the trap closure scores that ORE also stated. |
| WP-8 | DEF | 74 | 6 / 0 | 13 | Opus | **Done** (2026-10-02): 17 rows `verified`, 1 `abstract-only`, 56 `game-choice`; flags 13 and 15 (DEF part) settled without a code change; flag 14 put to Ben with pictures; no value in the code changed. |
| WP-9 | DEP | 164 | 24 / 0 | 34 | Sonnet 5.5 | Fix 3. Tables are open but are PDFs: the library now holds them as text. DEP and ORE kept the granular rows; check which of the two owns a row before verifying it. |
| WP-10 | ORE | 182 | 19 / 0 | 43 | Opus | Fix 2 (a real bug). Redo the coal size and density derivation from sources. Same code directory as WP-9: run them one after the other or as one agent. |
| WP-11 | BIO | 100 | 6 / 0 | 3 | Sonnet 5.5 | **Done** (2026-10-02, branch worktree-agent-ab02ca251f753b0a4): fix 4 made, plus the Cs/Cw tie rule and Tmon10; see biomes.md. Holdridge's 30 degC cut-off (fix 5) confirmed in the source and left to WP-5. |
| WP-12 | SCL, BAK, PRV, POL, PRS | 212 | 4 / 0 | 10 | Sonnet 5.5 | Fix 19, 20, 34, 35, 36, 37. Identify the province map properly or drop the citation. PRS (presets other than the default, test and world presets) is new here. |
| WP-13a | PLT `not-yet-coded`, CHK | 86 | 5 / 0 | 27 | Opus | **Before the force balance is built.** Fix 18, 38 to 41. Confirm slab pull, continental drag, wedge taper, suction share. CHK (the plate-check target ranges) is new here. |
| WP-13b | PLT rest, CRE | 198 | 3 / 0 | 23 | Sonnet 5.5 | **Deferred** until the plate rewrite lands; then ledger the new code and verify. CRE (geometry, noise, plate state, deformation lattice) is new; `plates/dynamics.rs` is still not covered. |
| WP-14a | CLM `not-yet-coded` | 28 | 4 / 0 | 11 | Opus | **Done** (2026-10-02): the design's constants read against their sources; the design document corrected where a source disagreed (surface-dependent D, desert albedo, drag, evaporation wind, rain law); 12 rows `verified`, CLM-163 to CLM-166 added; eleven carried-over coded rows verified; fix 17 written into the design (T6). See [per-cell-climate.md](per-cell-climate.md). |
| WP-14b | CLM rest, MOI | 176 | 13 / 0 | 19 | Sonnet 5.5 | **Deferred** until the climate rewrite lands. |
| WP-15 | HNT (and the hint rows in PLT, HYD, IMP) | 84 | 0 / 0 | 0 | Sonnet 5.5 | The hint texts are now ledgered (HNT, 84 rows). Remaining: one table of Earth figures with one source each; fix 27 to 33. |
| WP-16 | GEO | 51 | 0 / 0 | 0 | Sonnet 5.5 | New. The geothermal features (51 rows, all `game-choice` or `uncited`): find sources for the rainfall threshold and decide what is a game choice; fix 44 to 46. |

What can run in parallel: WP-1, WP-2, WP-5, WP-6, WP-7, WP-8, WP-11, WP-12, WP-13a, WP-14a and
WP-15 touch separate code and can run together once WP-0 is done. WP-3 and WP-4 share the
calibration and should be one effort, started after the surface-stage changes now in flight have
landed. WP-9 and WP-10 share the deposits code. WP-13b and WP-14b wait for the rewrites.

Each work package ends with: rows marked `verified`, corrected, or left `uncited` with a note; the
code corrected with a test where a value was wrong; the feature page written from
[_template.md](_template.md); the bibliography entries updated with what was read.

### Sources in the library, by work package

Produced by WP-0 (2026-10-02). The library is a folder on the test machine with one sub-folder per bibliography key (`<key>/`), holding the saved PDF or page and a text extraction beside it; the file names are those named here and in [bibliography.md](bibliography.md). It is not distributed. A work is listed under every package whose rows cite it. "Readable" means a text layer or text page exists; a fetch does not verify anything (see the status rules). Works the code cites that no ledger row ties to a package are listed in the bibliography only.

#### WP-1: 135 rows; 44 sources

- **Readable in the library** (text file names are in the folder named by the key): `Collins2005` (collins2005.txt); `DavisonCollins2007` (dc2007.txt); `Gilbert1893` (Gilbert1893.vol12.txt); `Grieve1987` (grieve1987.txt); `Osinski2013` (Osinski2013.txt).
- **Fetched but a scan** (no text layer; read by eye or OCR): `GaultWedekind1978`; `McKinnonSchenk1985`; `Moore1974`; `Pike1980`; `Pike1985`; `WoodHead1976`.
- **Landing page only** (the paper itself was not obtained): `GrieveShoemaker1994` (book); `Heimsath1997` (paywalled).
- **Not readable** : `Abramov2004` (paywalled, no substitute found); `AlexopoulosMcKinnon1994` (paywalled, no substitute found); `Ames1998` (paywalled, no substitute found); `Bland2006` (paywalled, no substitute found); `ChapmanMcKinnon1986` (book); `Culling1960` (paywalled, no substitute found); `Donofrio1998` (not found); `FassettThomson2014` (paywalled, no substitute found); `Fernandes1997` (paywalled, no substitute found); `Grieve1994` (paywalled, no substitute found); `Grieve2005` (paywalled, no substitute found); `GrieveCintala1992` (not found); `GrieveDenceRobertson1977` (book); `GrieveStoffler1991` (paywalled, no substitute found); `Herrick1997 (not in the sources file; it is the real source of Collins et al.'s eq. 28)` (book); `LeFeuvre2011` (open copy); `Lightfoot2016` (book); `Masaitis1998` (paywalled, no substitute found); `McGetchin1973` (paywalled, no substitute found); `Melosh1989` (book); `Naldrett2004` (book); `Neukum2001` (paywalled, no substitute found); `PierazzoVickery1997` (paywalled, no substitute found); `Pike1977` (book); `PikeSpudis1987` (paywalled, no substitute found); `Roering1999` (paywalled, no substitute found); `SchmidtHousen1987` (paywalled, no substitute found); `Shoemaker1962` (book); `Sklar2001` (paywalled, no substitute found); `Soderblom1970` (paywalled, no substitute found); `Stock1999` (paywalled, no substitute found).

#### WP-2: 115 rows; 1 source

- **Not readable** : `Bremond1995` (paywalled, no substitute found).
- **Added and read in WP-2 (2026-10-02)** (text files in the folder named by the key): `Grosse2014` (Grosse2014.txt); `ClagueSherrod2014` (ClagueSherrod2014.txt); `Christiansen2001` (Christiansen2001.txt); `USGS-MaunaLoa` (USGS-MaunaLoa.txt); `USGS-ActiveVolcanoesFAQ` (USGS-ActiveVolcanoesFAQ.txt); and, already in the library, `Hildreth2007`, `Poland2014`, `Courtillot2003a`. Abstracts only: `Bremond1995` (free to read, refused to scripts), `Rasmussen2022`, `ChaussardAmelung2014`, `TenBrink1991`. Not obtained: `Lev2019`; `ClagueDalrymple1987` was fetched but is a scan without text.

#### WP-3: 156 rows; 14 sources

- **Readable in the library** (text file names are in the folder named by the key): `Barnes2014` (Barnes2014.txt); `Landlab` (Landlab.txt); `Wikipedia-Laterite` (Wikipedia-Laterite.txt).
- **Landing page only** (the paper itself was not obtained): `Braun2013` (paywalled); `Heimsath1997` (paywalled); `Schott2024` (open copy); `Yuan2019` (open copy).
- **Not readable** : `Ahnert1976` (book); `Bosscher1992` (paywalled, no substitute found); `Harel2016` (open copy); `Hjulstrom1935` (book); `Roering1999` (paywalled, no substitute found); `Sklar2001` (paywalled, no substitute found); `Stock1999` (paywalled, no substitute found).

#### WP-4: 99 rows; 33 sources

- **Readable in the library** (text file names are in the folder named by the key): `Calmant1990` (Calmant1990.txt); `CharetteSmith2010` (CharetteSmith2010.txt); `Cortial2019` (Cortial2019.txt); `Jain2024` (Jain2024.txt); `Peel2007` (Peel2007.txt); `Tzathas2024` (Tzathas2024.txt); `Wickert2016` (Wickert2016.txt).
- **Landing page only** (the paper itself was not obtained): `Braun2013` (paywalled).
- **Not readable** : `Beaumont2001` (paywalled, no substitute found); `Bird1991` (paywalled, no substitute found); `BrotchieSilvester1969` (paywalled, no substitute found); `BurovDiament1995` (paywalled, no substitute found); `ClarkRoyden2000` (paywalled, no substitute found); `Cogley1984` (paywalled, no substitute found); `EnglandMcKenzie1982` (paywalled, no substitute found); `Fastscapelib` (open access); `KarnerWatts1983` (paywalled, no substitute found); `Lu2024` (paywalled, no substitute found); `LyonCaenMolnar1983` (paywalled, no substitute found); `MolnarLyonCaen1988` (book); `ParsonsSclater1977` (paywalled, no substitute found); `PerezGussinyeWatts2005` (paywalled, no substitute found); `Rey2001` (paywalled, no substitute found); `Royden1997` (paywalled, no substitute found); `Shepard` (not found); `SteinStein1992` (paywalled, no substitute found); `Tozer2019` (open access); `TurcotteSchubert_Geodynamics` (book); `Watts1978` (paywalled, no substitute found); `WattsTenBrink1989` (paywalled, no substitute found); `Wessel1993` (paywalled, no substitute found); `Wessel2019-GMT` (other); `WieczerkowskiMitrovicaWolf1999` (other).

#### WP-5: 94 rows; 20 sources

- **Readable in the library** (text file names are in the folder named by the key): `Barnes2014` (Barnes2014.txt); `Barnes2014b` (Barnes2014b.txt); `Barnes2021` (Barnes2021.txt); `Holdridge1967` (Holdridge1967.txt); `Leopold1953` (Leopold1953.txt); `Leopold1957` (Leopold1957.txt); `Lugo1999` (Lugo1999.txt); `Messager2016` (Messager2016.txt); `Ohmura1992` (Ohmura1992.txt); `Peel2007` (Peel2007.txt); `Zhao2022` (Zhao2022.txt); `Zomer2022` (Zomer2022.txt).
- **Landing page only** (the paper itself was not obtained): `Downing2006` (other); `Schott2024` (open copy); `Schumm1963` (other).
- **Not readable** : `Fu1981` (not found); `Garbrecht1997` (paywalled, no substitute found); `MiddletonThomas1997` (book); `Verpoorter2014` (open access); `Zhang2004` (other).

#### WP-6: 135 rows; 25 sources

- **Readable in the library** (text file names are in the folder named by the key): `Alley1998` (Alley1998.txt); `Barnes2014` (Barnes2014.txt); `Hergarten2021` (Hergarten2021.txt); `Humphrey1994` (Humphrey1994.txt); `Ohmura1992` (Ohmura1992.txt); `Sugden1978` (Sugden1978.txt); `Tzathas2024` (Tzathas2024.txt); `WorldbuildingPasta` (WorldbuildingPasta.txt).
- **Landing page only** (the paper itself was not obtained): `goSPL` (other); `Porter1989` (other); `Schott2024` (open copy); `Svensson1959` (open access).
- **Not readable** : `Alley2003` (paywalled, no substitute found); `Cuffey2010` (book); `Egholm2009` (paywalled, no substitute found); `gosplDocs` (open access); `Hallet1996` (paywalled, no substitute found); `Harbor1992` (paywalled, no substitute found); `ICAO1993` (not found); `Minder2010` (other); `Mitchell2015` (paywalled, no substitute found); `Oerlemans2011` (book); `Porter2001` (paywalled, no substitute found); `Rolland2003` (other); `Sugden1976` (book).

#### WP-7: 117 rows; 17 sources

- **Readable in the library** (text file names are in the folder named by the key): `Alavi2007` (Alavi2007.txt).
- **Landing page only** (the paper itself was not obtained): `Heimsath1997` (paywalled); `tectonics.js (software)` (open access); `Telford1990` (book).
- **Not readable** : `Bateman1979` (paywalled, no substitute found); `Buddington1959` (paywalled, no substitute found); `Cruden1998` (paywalled, no substitute found); `Doust1990` (book); `Haederle2002` (paywalled, no substitute found); `Hudec2007` (paywalled, no substitute found); `Mukasa1986` (paywalled, no substitute found); `Pollack1993` (paywalled, no substitute found); `Roering1999` (paywalled, no substitute found); `Schon2015` (book); `Warren2016` (book); `Winter2010` (book); `Yardley1989` (book).

#### WP-8: 74 rows; 19 sources

- **Readable in the library** (text file names are in the folder named by the key): `Alavi2007` (Alavi2007.txt); `Jessell1981` (Jessell1981.txt); `Jessell2022` (Jessell2022.txt); `Lathrop2022` (Lathrop2022.txt); `NoddyOriginal (software, other releases)` (NoddyOriginal_software_other_releases.txt); `Wellmann2016` (Wellmann2016.txt).
- **Landing page only** (the paper itself was not obtained): `Jessell1996` (open copy); `Noddyverse (software)` (open access).
- **Not readable** : `Anderson1951` (book); `Biot1961` (paywalled, no substitute found); `Blanc2003` (paywalled, no substitute found); `Cowie1992a` (paywalled, no substitute found); `Gawthorpe2000` (open access); `Kim2005` (paywalled, no substitute found); `McQuarrie2004` (open copy); `Mouthereau2007` (paywalled, no substitute found); `Schlische1996` (paywalled, no substitute found); `Sherwin1968` (paywalled, no substitute found); `Weissel1989` (paywalled, no substitute found).
- **Added and read in WP-8 (2026-10-02)** (text files in the folder named by the key): `Mouthereau2007` (Mouthereau2007_raw.txt, HAL copy); `Schmalholz2016`; `Elliott1976`; `Anderson1905` (OCR of a scan); `WaldronSnyder2020`; `DelaneyPollard1981`; `Krumbholz2014`; `Bunger2013` (author manuscript); `Herail1990` (poor OCR); `Latutrie2020`; `Earle2019` (chapter pages); `NMT_Shiprock` and `WikipediaCraton` (web pages); and, already in the library, `Alavi2007`, `Lathrop2022` (Lathrop2022_raw.txt), `Jessell1981`, `Jessell2022`. Abstracts only: `McQuarrie2004`, `Schlische1996`, `CollettiniSibson2001`. Not obtained: `Anderson1951`, `Biot1961`, `Sherwin1968`, `Blanc2003`, `Cowie1992a`, `Kim2005`, `Gawthorpe2000`, `Jessell1996`, `Weissel1989`. The Noddy source was read at the pinned commit from a tarball on the test machine.

#### WP-9: 164 rows; 58 sources

- **Readable in the library** (text file names are in the folder named by the key): `Acklam` (acklam3.txt); `Blackman2021` (Blackman2021.txt); `Cannon1986` (Cannon1986.txt); `Cannon1995` (ofr95831.txt); `Cox1986` (b1693.txt); `duBray1995` (ofr95831.txt); `Ellis1999` (pp1625PN.txt, pp1625PG.txt, pp1625PS.txt); `Emsbo2016` (sir5070n.txt); `Hormann1993` (Hormann1993.txt); `John2010` (sir5070b.txt); `Leach2010a` (sir5070a.txt); `MenzieMosier1986` (MenzieMosier1986.txt); `MosierBriskey1986` (MosierBriskey1986.txt); `MosierSinger1986` (MosierSinger1986.txt); `NMRA1998data` (nmra.zip, nmra_tracts.txt, nmra_types.txt); `OrrisBliss1985` (OrrisBliss1985.txt); `OrrisBliss1986` (OrrisBliss1986.txt); `Schmidt2007` (sir5039.txt); `Singer2001` (pp1640a.txt); `Singer2008a` (of1155.txt, PorCuTX2008.txt); `Singer2009` (of1252.txt); `Tuttle2000` (au719211.txt, reg7.zip); `USGS2000a` (c1178.txt); `Wood1983` (circ891.txt).
- **Landing page only** (the paper itself was not obtained): `Cooke2005` (open copy); `Singer1993` (paywalled); `Singer2005` (paywalled); `Singer2008b` (paywalled).
- **Not readable** : `AbramowitzStegun1964` (book); `Agterberg1995` (paywalled, no substitute found); `Bethke1990` (paywalled, no substitute found); `Bliss1992` (open copy); `Bohacs1997` (paywalled, no substitute found); `Bradley2003` (paywalled, no substitute found); `Clark1990` (paywalled, no substitute found); `Garven1984` (paywalled, no substitute found); `Garven1985` (paywalled, no substitute found); `Hronsky2008` (paywalled, no substitute found); `Kesler2006` (paywalled, no substitute found); `Kesler2008` (paywalled, no substitute found); `Leach2001` (paywalled, no substitute found); `Leach2005` (paywalled, no substitute found); `Leach2010b` (paywalled, no substitute found); `Marsaglia1964` (paywalled, no substitute found); `McCuaig2010` (paywalled, no substitute found); `Mote2001` (paywalled, no substitute found); `Nelsen2016` (open access); `Oliver1986` (paywalled, no substitute found); `Richards2003` (paywalled, no substitute found); `Robert2005` (paywalled, no substitute found); `Root1992` (paywalled, no substitute found); `Seedorff2005` (paywalled, no substitute found); `Sillitoe2005` (paywalled, no substitute found); `Sillitoe2010` (paywalled, no substitute found); `SillitoeMcKee1996` (paywalled, no substitute found); `Singer1995` (paywalled, no substitute found); `Skirrow2019` (paywalled, no substitute found); `Titley1995` (book).

#### WP-10: 182 rows; 62 sources

- **Readable in the library** (text file names are in the folder named by the key): `Averitt1975` (Averitt1975.txt); `Cox1986` (b1693.txt); `Dubiel2003` (Dubiel2003.txt); `Ellis1999` (pp1625PN.txt, pp1625PG.txt, pp1625PS.txt); `Emsbo2016` (sir5070n.txt); `Hatch2002` (Hatch2002.txt); `Hoggard2020` (Hoggard2020.txt); `John2010` (sir5070b.txt); `Leach2010a` (sir5070a.txt); `Moore2003` (ofr03253.txt); `Newell1997` (kgs.txt); `Nuccio1994` (Nuccio1994.txt); `Nuccio2003` (Nuccio2003.txt); `Osinski2013` (Osinski2013.txt); `Ruppert2014` (Ruppert2014.txt); `SAIMM-coalfields` (SAIMM-coalfields.txt); `Spencer1988` (Spencer1988.txt); `Tuttle2000` (au719211.txt, reg7.zip); `Wood1983` (circ891.txt).
- **Landing page only** (the paper itself was not obtained): `BAP2018` (open access); `Cooke2005` (open copy); `GABowen` (open access); `Lopatin1971 (related; not cited by the code)` (not found); `USGS2000b` (open access); `Waples1980` (paywalled).
- **Not readable** : `Abramov2004` (paywalled, no substitute found); `Ames1998` (paywalled, no substitute found); `Basuki2002` (paywalled, no substitute found); `Bekker2010` (paywalled, no substitute found); `Bethke1990` (paywalled, no substitute found); `Bohacs1997` (paywalled, no substitute found); `Bradley2003` (paywalled, no substitute found); `Butterworth2016` (paywalled, no substitute found); `Dyman2002` (paywalled, no substitute found); `Garnett2005` (paywalled, no substitute found); `Garven1984` (paywalled, no substitute found); `Garven1985` (paywalled, no substitute found); `Grieve1994` (paywalled, no substitute found); `Grieve2005` (paywalled, no substitute found); `Gumsley2017` (open access); `Hampson1999` (paywalled, no substitute found); `Heckel1986` (paywalled, no substitute found); `Higley2005` (open access); `Kesler2006` (paywalled, no substitute found); `Kesler2008` (paywalled, no substitute found); `Leach2001` (paywalled, no substitute found); `Leach2005` (paywalled, no substitute found); `Leach2010b` (paywalled, no substitute found); `Lehmann2020` (paywalled, no substitute found); `Lightfoot2016` (book); `Magoon1994` (book); `Masaitis1998` (paywalled, no substitute found); `Nelsen2016` (open access); `NMET-Jharia` (not found); `Rasbury1998` (paywalled, no substitute found); `Richards2003` (paywalled, no substitute found); `Robert2005` (paywalled, no substitute found); `RuhrCounts` (not found); `Ryer1980` (paywalled, no substitute found); `SilesiaMDPI` (not found); `Thomas1949` (paywalled, no substitute found); `Trippi2011` (open access).

#### WP-11: 99 rows; 9 sources

- **Readable in the library** (text file names are in the folder named by the key): `Felzenszwalb2012` (Felzenszwalb2012.txt); `ODonnell2012` (ODonnell2012.txt); `Ohmura1992` (Ohmura1992.txt); `Peel2007` (Peel2007.txt); `WikipediaKoppen` (WikipediaKoppen.txt); `Zomer2022` (Zomer2022.txt).
- **Not readable** : `Fick2017` (other); `MiddletonThomas1997` (book); `Thornthwaite1948` (paywalled, no substitute found).

#### WP-12: 212 rows; 14 sources

- **Readable in the library** (text file names are in the folder named by the key): `Coltice2019` (Coltice2019.txt); `Cortial2019` (Cortial2019.txt); `Gonzalez2010` (Gonzalez2010.txt); `Peel2007` (Peel2007.txt).
- **Landing page only** (the paper itself was not obtained): `Mallard2016` (paywalled); `USGS-geologic-provinces-map (no author, no year; key: USGSprov)` (open copy).
- **Not readable** : `Connelly2012 (age of the solar system)` (other); `Gough1981` (open copy); `Keinert2015` (paywalled, no substitute found); `Kesler2008` (paywalled, no substitute found); `Koerner2004 (treeline)` (other); `NASA-EarthFactSheet` (open access); `Swinbank2006` (other); `UNEP1992 (aridity classes)` (book).

#### WP-13a: 86 rows; 32 sources

- **Readable in the library** (text file names are in the folder named by the key): `Coltice2019` (Coltice2019.txt); `DeCellesGiles1996` (DeCellesGiles1996.txt); `Nance2022` (Nance2022.txt); `Nocquet2016` (Nocquet2016.txt); `Tremblay2015` (Tremblay2015.txt).
- **Added in WP-13a** (2026-10-02, lawful open copies): `ForsythUyeda1975` (publisher's free PDF); `LithgowBertelloniRichards1998` and `ConradLB2006` (UCL Discovery); `Zahirovic2015` (accepted manuscript, Wollongong; the landing page kept as Zahirovic2015_landing.txt); `Mitchell2021` (accepted manuscript, Curtin; landing page kept as Mitchell2021_landing.txt); `Bird2003` (author's site); `FaccennaBecker2012` (author's preprint); `DeCellesCarrapa2021` (NSF public access copy); `vanHunen2011EGU` (EGU abstract); `Seton2020_earth_age` (the GMT server's 6 arc-minute sea-floor age grid). Still not readable: Schellart 2004, Conrad & Lithgow-Bertelloni 2002 and 2004, Davis, Suppe & Dahlen 1983, Dahlen 1990, Carlson et al. 1983, Copley et al. 2010 (Wiley and Durham refuse scripted access), van Hunen & Allen 2011, Parsons & Richter 1980, Turcotte & Schubert.
- **Landing page only** (the paper itself was not obtained): `Huang2019` (open copy); `Mitchell2021` (paywalled); `Zahirovic2015` (paywalled).
- **Not readable** : `BercoviciLong2014` (paywalled, no substitute found); `Bialas2010` (paywalled, no substitute found); `Buck2004` (book); `Carlson1983` (paywalled, no substitute found); `Coltice2007` (paywalled, no substitute found); `ConradLB2002` (paywalled, no substitute found); `ConradLB2004` (paywalled, no substitute found); `ConradLB2006` (open copy); `Copley2010` (paywalled, no substitute found); `DalZilio2018` (paywalled, no substitute found); `DaviesvonBlanckenburg1995` (paywalled, no substitute found); `DavisSuppeDahlen1983` (paywalled, no substitute found); `ForsythUyeda1975` (paywalled, no substitute found); `Gurnis1988` (paywalled, no substitute found); `HeronLowman2011` (paywalled, no substitute found); `Lenardic2011` (paywalled, no substitute found); `LithgowBertelloniRichards1998` (paywalled, no substitute found); `MolnarLyonCaen1988` (book); `ParsonsRichter1980` (paywalled, no substitute found); `RoyerGordon1997` (paywalled, no substitute found); `Schellart2004` (paywalled, no substitute found); `TurcotteSchubert_Geodynamics` (book); `vanHunenAllen2011` (paywalled, no substitute found); `Zhang2018` (open access).

#### WP-13b: 198 rows; 26 sources

- **Readable in the library** (text file names are in the folder named by the key): `CharetteSmith2010` (CharetteSmith2010.txt); `Cortial2019` (Cortial2019.txt); `Nance2022` (Nance2022.txt).
- **Landing page only** (the paper itself was not obtained): `Mallard2016` (paywalled); `Mitchell2021` (paywalled); `Zahirovic2015` (paywalled).
- **Not readable** : `Bird2003` (open access); `Bradley2008` (paywalled, no substitute found); `Cogley1984` (paywalled, no substitute found); `Copley2010` (paywalled, no substitute found); `Davidson_tectonicsjs` (other); `ForsythUyeda1975` (paywalled, no substitute found); `Lallemand_EncMarineGeosciences` (not found); `Lonsdale2005` (paywalled, no substitute found); `MercierdeLepinay2016` (paywalled, no substitute found); `Morra2013` (paywalled, no substitute found); `ParsonsSclater1977` (paywalled, no substitute found); `Schellart2008` (paywalled, no substitute found); `Shepard1963` (book); `Shoemake1992` (book); `SteinStein1992` (paywalled, no substitute found); `Stern2002` (paywalled, no substitute found); `Stern2004` (paywalled, no substitute found); `SternGerya2018` (paywalled, no substitute found); `Tozer2019` (open access); `Wilson1966` (paywalled, no substitute found).

#### WP-14a: 28 rows; 15 sources

- **Readable in the library** (text file names are in the folder named by the key): `Ramirez2024` (Ramirez2024.txt); `Rohde2020` (Rohde2020.txt); `Stassen2019` (Stassen2019.txt); `Ziegler2021` (Ziegler2021.txt).
- **Added in WP-14a** (2026-10-02, lawful open copies): `Bretherton2004` (co-author's copy); `Manabe1969` (GFDL); `Trenberth2001` (Zenodo deposit); `Ferreira2018` (NSF public access repository); `Dommenget2011` (author's copy); `DonohoeBattisti2011` (accepted manuscript, co-author's page); `WagnerEisenman2015` (author's site); `McCarthy2015` (NERC accepted manuscript); `FrajkaWilliams2019` (open access); `BackBretherton2009` (author's copy); `LargeYeager2004` (NCAR technical note); `IPCC2001_ch6` (IPCC); `climlab_src` (MIT source); `Zhuang2017` (the release's manual only); `Stull2017`, `Norton1979`, `Bierwirth2009`, `Guilloteau2000`; `BerkeleyEarth_LandOcean` (data file); `NCEP_R1_ltm` (NOAA PSL reanalysis long-term means). Still not readable: `Lindzen1987`, `Serreze1992`, `Stommel1948` (an open copy exists only on a course page re-hosting the publisher's PDF), `North1983`, `Murray1967` (Tetens), Large & Pond 1981, the Zhuang et al. 2017 paper itself (Elsevier refuses scripted access).

#### WP-14b: 176 rows; 32 sources

- **Readable in the library** (text file names are in the folder named by the key): `Bergman2004` (Bergman2004.txt); `BernerKothavala2001` (BernerKothavala2001.txt); `Byrne2014` (Byrne2014.txt); `Felzenszwalb2012` (Felzenszwalb2012.txt); `Fretwell2013` (Fretwell2013.txt); `HeldSoden2006` (HeldSoden2006.txt); `Hoffman2017` (Hoffman2017.txt); `Morlighem2017` (Morlighem2017.txt); `North1981` (North1981.txt); `Nye1952` (Nye1952.txt); `Rohde2020` (Rohde2020.txt); `Rose2018` (Rose2018.txt); `WorldbuildingPasta` (WorldbuildingPasta.txt).
- **Landing page only** (the paper itself was not obtained): `Archer2009` (open copy); `FastscapeLEM` (open access).
- **Not readable** : `Berger1978` (paywalled, no substitute found); `Berner1994` (open access); `Bird2003` (open access); `Cuffey2010` (book); `Gough1981` (open copy); `Hoffman1998` (paywalled, no substitute found); `ICAO1993` (not found); `Imbrie1980` (paywalled, no substitute found); `Kopp2011` (paywalled, no substitute found); `Laskar2004` (open access); `Myhre1998` (paywalled, no substitute found); `Rowley2002` (paywalled, no substitute found); `Self2006` (open copy); `SmithBarstad2004` (other); `Thornthwaite1948` (paywalled, no substitute found); `Walker1981` (paywalled, no substitute found); `Willmott1985` (paywalled, no substitute found).

#### WP-15: 84 rows; 0 sources


#### WP-16: 51 rows; 0 sources


#### Not tied to a ledger row (context sources): 37

