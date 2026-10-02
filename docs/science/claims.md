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

2,069 rows.

| Ledger | Subject | Rows | `cited-unverified` | `from-memory` | `abstract-only` | `calibrated` | `game-choice` | `uncited` | of which `pending-rewrite` | of which `not-yet-coded` |
|---|---|---|---|---|---|---|---|---|---|---|
| [PLT](ledger/PLT.md) | plate tectonics | 212 | 51 | 6 | 20 | 19 | 63 | 53 | 183 | 29 |
| [CLM](ledger/CLM.md) | climate physics | 146 | 43 | 5 | 0 | 13 | 23 | 62 | 118 | 28 |
| [MOI](ledger/MOI.md) | moisture, winds, seasons | 76 | 20 | 0 | 0 | 0 | 10 | 46 | 76 | 0 |
| [BIO](ledger/BIO.md) | Köppen classes and biomes | 99 | 24 | 0 | 0 | 1 | 66 | 8 | 0 | 0 |
| [ERO](ledger/ERO.md) | erosion, sediment, facies | 167 | 86 | 2 | 3 | 6 | 6 | 64 | 0 | 0 |
| [ISO](ledger/ISO.md) | isostasy, subsidence, height limit, sea level | 39 | 8 | 0 | 0 | 6 | 6 | 19 | 0 | 0 |
| [HYP](ledger/HYP.md) | hypsometry and calibration targets | 34 | 10 | 0 | 1 | 18 | 1 | 4 | 0 | 0 |
| [HYD](ledger/HYD.md) | rivers, lakes, runoff | 76 | 14 | 3 | 0 | 1 | 36 | 22 | 0 | 0 |
| [YNG](ledger/YNG.md) | the young landscape | 21 | 1 | 0 | 0 | 0 | 9 | 11 | 0 | 0 |
| [AMP](ledger/AMP.md) | the amplifier | 71 | 8 | 0 | 0 | 1 | 53 | 9 | 0 | 0 |
| [GLA](ledger/GLA.md) | glacial erosion and landforms | 66 | 20 | 1 | 0 | 1 | 26 | 18 | 0 | 0 |
| [VOL](ledger/VOL.md) | volcanoes | 73 | 1 | 0 | 0 | 1 | 16 | 55 | 0 | 0 |
| [MAG](ledger/MAG.md) | magma and lava | 42 | 3 | 0 | 0 | 0 | 31 | 8 | 0 | 0 |
| [IMP](ledger/IMP.md) | meteor impacts | 141 | 9 | 6 | 6 | 0 | 65 | 12 | 0 | 0 |
| [RCK](ledger/RCK.md) | rock classes, metamorphism, plutons | 84 | 18 | 0 | 0 | 1 | 26 | 39 | 0 | 0 |
| [STR](ledger/STR.md) | strata, traps, hosts | 48 | 4 | 0 | 0 | 2 | 37 | 5 | 0 | 0 |
| [DEF](ledger/DEF.md) | folds, faults, intrusions | 74 | 16 | 0 | 0 | 0 | 54 | 4 | 0 | 0 |
| [DEP](ledger/DEP.md) | deposit models, counts, sizes | 204 | 88 | 0 | 23 | 0 | 61 | 32 | 0 | 0 |
| [ORE](ledger/ORE.md) | deposit formation, coal, petroleum, placers | 257 | 82 | 2 | 10 | 0 | 57 | 106 | 0 | 0 |
| [PRV](ledger/PRV.md) | geologic provinces | 27 | 5 | 0 | 0 | 4 | 6 | 12 | 0 | 0 |
| [SCL](ledger/SCL.md) | world scale, planet constants | 51 | 6 | 0 | 0 | 1 | 37 | 7 | 0 | 0 |
| [BAK](ledger/BAK.md) | the bake, satellite view | 40 | 0 | 0 | 0 | 1 | 34 | 5 | 0 | 0 |
| [POL](ledger/POL.md) | the pole cap | 21 | 0 | 0 | 0 | 0 | 18 | 3 | 0 | 0 |
| [GEO](ledger/GEO.md) | geothermal features | 0 | – | – | – | – | – | – | – | – |
| **all** | | **2,069** | **522** | **53** | **60** | **76** | **723** | **635** | **377** | **57** |

No row is `verified`. In plain terms: 723 rows (35 %) are deliberate game or implementation
choices; 635 (31 %) state something about the physical world with no source at all; 522 (25 %)
name a source that nobody is recorded as having read; 113 (5 %) were written from memory or from
an abstract and say so; 76 (4 %) were tuned to a target.

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
| `pending-rewrite` | Added to rows of today's plate motion and today's banded climate, which are about to be replaced. Stage 2 does not verify these. |
| `not-yet-coded` | The constant exists only in an approved design. Verify it before it is built. |

Rules for changing the ledger (see `CLAUDE.md`, "Science documentation is part of done"):

- A row becomes `verified` only when someone has fetched the source and read the value. Write
  what was read: "Collins et al. 2005, eq. 21, p. 823, full text, 2026-10-05".
- If the source gives a different value, the code is corrected, with a test, and the row records
  the old value, the new value and the source.
- If no source can be found, the row stays `uncited` and its feature page says so. A value may be
  kept without a source; it may not be presented as established.
- When a value changes in code, its row changes in the same commit.

## Supporting files

- [bibliography.md](bibliography.md): the sources, with full references and how each has been read.
- [spot-checks.md](spot-checks.md): what the stage 1 bibliography pass found when it tried to read
  403 values in their sources: 176 matched, 134 matched in part, 19 did not match, 74 could not be
  read. Results there are leads for stage 2, with the passage quoted and the level of evidence; none
  of them changed a status.
- `extraction-notes/`: per subsystem, the measured Earth-against-mod numbers in the build log,
  the stated simplifications, and flags (contradictions between comment, code and document).
- [extraction-notes/preset-coverage.md](extraction-notes/preset-coverage.md): every key of the
  default preset with its value and owning subsystem, and how the other presets differ.

## Known gaps in the ledger itself

- **Two agents per large subsystem did not read each other's rows.** Some claims are ledgered
  twice under different ids (the ocean age-depth law in PLT and ISO; rock erodibility in ERO and RCK;
  coal and petroleum sizes in DEP and ORE; the equilibrium-line rule in CLM, GLA and HYD). Stage 2
  merges them: keep the lower id, mark the other "see".
- **Not read line by line:** test modules; `plates/dynamics.rs`, `geometry.rs`, `noise.rs`,
  `state.rs`; `structure/deform/lattice.rs` and parts of `bake_horizons.rs`; the internals of the
  diffusion solver and flow routing; the Java twins of Rust code; `surface/detail.rs` beyond its
  constants; editor hint texts (`crates/deeptime-viewer/assets/editor_fields.json`) except where an
  agent happened on an Earth claim. Hints are shown to players and state Earth figures: they need
  their own sweep.
- **Only the default preset was read in full** (`earthlike_quick_16k`). The other presets were
  diffed against it; the test presets were not ledgered.
- **Validation ranges** (the bounds a setting is allowed to take) are recorded inside the value
  cell, not as rows of their own.
- **Geothermal features** have no rows: they are not on master.
- Rows were extracted by Sonnet sub-agents and checked for form, counts and private details by the
  stage 1 lead. The lead read the impacts and volcano rows in full and sampled the rest. A wrong
  "as coded" value is possible; the code wins.

## Verification plan for stage 2

### Found in stage 1: fix first

These came out of the spot checks and the extraction flags. None was fixed (stage 1 changes no
code). "Evidence" says how strong the finding is.

| # | Ledger | What is wrong | Evidence |
|---|---|---|---|
| 1 | IMP-042 | Complex crater depth is coded 0.294 D^0.301 km and cites Collins et al. 2005, eq. 28. That equation is 0.4 D^0.3 km. Craters over 3.2 km are about 27 % too shallow. | primary full text, re-read by the stage 1 lead **Fixed in stage 2 (WP-1, 2026-10-02):** code, comment or hint corrected, with a test where a value changed; see [ledger/IMP.md](ledger/IMP.md). |
| 2 | ORE-001 and following; `surface/fields.rs`, `surface/mod.rs` | The time windows are not applied on real histories: the banded-iron, oxygenated-ocean and land-plant clocks are hard-coded. No BIF ever forms; coal and sediment-hosted zinc-lead are not limited in time. | code reading by the extraction agent; the deposit audit found no BIF on either planet |
| 3 | DEP (the `richness_mult` setting) | The richness setting is documented as scaling grade, and grade never reaches a block. | deposit audit, build log 2026-10-01 |
| 4 | BIO-002 | Köppen: the code tests the polar group before the dry group. Peel et al. 2007 says "the B climates must be identified first". Cold dry land is classed tundra or ice cap instead of desert or steppe. | primary full text (sub-agent) |
| 5 | HYD-030 | Holdridge biotemperature counts temperatures above 30 °C as zero as well as those below 0 °C. The code applies only the lower limit, with Holdridge's coefficient. Evaporation is overestimated in hot deserts. | primary full text, scan (sub-agent) |
| 6 | ERO-070 | Soil production depth scale is 5 m. Secondary sources give 0.25 to 0.5 m for Heimsath's law. A possible factor of 10 to 20. | secondary only; Heimsath et al. 1997 not readable |
| 7 | ERO-022, ERO-049 | The comment says diffusivity between two cells is averaged "as Landlab's LinearDiffuser". Landlab takes the maximum. | Landlab source (sub-agent) |
| 8 | ERO-003; `SurfaceParams::default` | `k_spl` is 2e-9 in the Rust default and its comment, 3e-9 in all eight presets. | code reading |
| 9 | IMP-018 | Editor hint: "one crater of 20 km or more per 180 million km² per year". The coded rate is one per 1.8 × 10¹⁴ km² per year: a factor of a million. | arithmetic **Fixed in stage 2 (WP-1, 2026-10-02):** code, comment or hint corrected, with a test where a value changed; see [ledger/IMP.md](ledger/IMP.md). |
| 10 | IMP-038, IMP-054, IMP-062 | Equation numbers of Collins et al. 2005 cited in comments are wrong (rim height is eq. 48, ejecta eq. 47, melt eq. 30). Values are right. | primary full text (sub-agent) **Fixed in stage 2 (WP-1, 2026-10-02):** code, comment or hint corrected, with a test where a value changed; see [ledger/IMP.md](ledger/IMP.md). |
| 11 | VOL-039 | The 60 km spacing of arc volcanoes cites de Bremond d'Ars et al. 1995, whose abstract says spacing is random with no characteristic value. | primary abstract |
| 12 | VOL-016, VOL-026, VOL-029, VOL-048 | Comments disagree with the code they sit on (flank angle, crater size, caldera "across" against radius, subsidence rate). | arithmetic by the extraction agent |
| 13 | DEF-016 to DEF-018 | Fault slip ratios are coded 0.05, 0.08, 0.05 beside a cited 0.03. | secondary (a review quoting Schlische) |
| 14 | DEF-019, DEF-020 | Fault dips from Anderson 1951 are applied in block space, which is stretched 15 times vertically: real dips become about 6.6° and 2.2°. A design question, not a typo. | arithmetic from the build log's own formula |
| 15 | DEF-042, RCK-061 | A "stock" is coded larger than the definition quoted beside it; the pluton thickness law cited to Cruden 1998 is not in that paper's abstract. | secondary; primary abstract |
| 16 | HYP-010 | The ocean's most common depth is stated as 3 to 4 km; NOAA gives about 4.3 km and the project's own Earth table peaks in the 4 to 5 km band. | agency page (sub-agent) |
| 17 | CLM (ice thickness) | Land ice uses a hard-coded gravity of 9.81 m/s² and ignores the planet's gravity setting. In code that is `pending-rewrite`; carry the fix into the new climate. | code reading |
| 18 | PLT, plate-check and the force-balance design | "Slab-attached plates move 3 to 4 times faster" is attributed to Forsyth & Uyeda 1975; the statement found is in Conrad & Lithgow-Bertelloni 2004. Correct the attribution before the force balance is calibrated to it. | primary abstracts |
| 19 | SCL (`pole_cap_deg`), preset | The preset's pole-cap latitude is read by nothing; the game uses its own value from the world preset file. | code reading |
| 20 | SCL-039 and the timeline range | A run may be 4,600 Myr long while the planet's age defaults to 4.57 Gyr. | arithmetic |
| 21 | DEP (MVT and sedex time gate) | The 2.02 Ga limit comes from Leach et al. 2010 in Economic Geology, not from the USGS report of the same authors and year that is also cited; that paper does not support putting MVT deposits on the same clock as sedex. Moot until fix 2. | primary abstract |
| 22 | DEP (supergene copper) | The data file says Sillitoe 2005's "3 to 9 Myr" was not found; the abstract has it ("at least 0.5 m.y. but typically minima of 3 to 9 m.y."). The engine uses only the 0.5. The 100 mm/yr rainfall floor is stricter than the sources. | primary abstract |
| 23 | DEP ("Circular 1178 p. 15") | The 1 km assessment depth is on printed page 5. The rule itself is right. | primary full text (sub-agent) |
| 24 | MOI-073, CLM (weathering) | The runoff sensitivity 0.038 per K is GEOCARB II's (Berner 1994); the cited Berner & Kothavala 2001 uses 0.045 and 0.025. | primary full text (sub-agent) |
| 25 | GLA-002 | Glacial erosion constant 1e-5 with exponent 1; the published pairs are 1e-4 with exponent 1 and 1e-5 with exponent 2. Glacial erosion may be ten times too slow, or the exponent wrong. | primary abstract and a citing paper |
| 26 | AMP-034, AMP-038 | The thermal slope values attributed to Schott et al. are read differently from Schott's shader: 0.9 and 1.4 there are absolute tangents, not multipliers of 0.57. | Schott's shader source (sub-agent) |

Further mismatches and partial results, with quotations, are in [spot-checks.md](spot-checks.md);
further flags are in `extraction-notes/`.

### Flags known before stage 1, and where each now stands

| Flag | Where it stands |
|---|---|
| The impacts agent wrote the Collins, Melosh & Marcus 2005 constants from memory | The paper was read. Ten constants match; the complex-crater depth law does not (fix 1); three equation numbers are wrong (fix 10). 34 IMP rows stay `from-memory` until stage 2 confirms each. |
| It doubted the complex-crater depth law (0.294 D^0.301 km against 0.4 D^0.3) | The doubt was right: the paper has 0.4 D^0.3. |
| It fitted the ocean rule to an abstract | Davison & Collins 2007 was read in full. The three numbers match; the paper's own limit is a depth-to-diameter ratio of about 8, against the code's fitted 6.3 (IMP-065 to IMP-067 stay `abstract-only`). |
| The force-balance design could not confirm the gross size of slab pull | Still unconfirmed. Supported only as "of order 10¹³ N/m" by a 2024 paper citing Turcotte & Schubert; the book was not read (WP-13a). |
| ... a factor of 8 for continental drag | Not in Forsyth & Uyeda's abstract, which says only that drag is "stronger under the continents". The paper's body was not readable (WP-13a). |
| ... a wedge-taper reference | Davis, Suppe & Dahlen 1983 exists and the reference is right; no angle is in its abstract. A secondary summary gives tapers from 2.5°, so the design's 1.5° is doubtful (WP-13a). |
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
design: gross slab pull, the factor of 8 for continental drag, and the wedge taper, none confirmed.

### Work packages

Counts exclude nothing: they are all rows of the ledgers named. "Paywalled" lists what stage 1
could not read and stage 2 needs. Suggested model follows the project rule: Sonnet 5.5 for
well-specified checking and fixing, Opus where a wrong value forces a design or calibration decision.

| WP | Ledgers | Rows | Sources needed | Paywalled or unread | Model | Notes |
|---|---|---|---|---|---|---|
| WP-0 | all | – | a way to read PDF text on the test machine; a de-duplicated bibliography | – | Sonnet 5.5 | Do first. Stage 1 lost most of its time to PDFs the fetch tool cannot read and a search budget that ran out. Merge duplicate ledger rows and bibliography keys. |
| WP-1 | IMP | 141 | Collins et al. 2005 (in hand), Davison & Collins 2007 (in hand), Grieve 1987 (in hand) | Grieve & Shoemaker 1994 (book), Pike 1980 and 1985, Alexopoulos & McKinnon 1994, Herrick et al. 1997, Fernandes & Dietrich 1997, Roering et al. 1999, Neukum et al. 2001, Bland & Artemieva 2006 | Sonnet 5.5 | Fix 1, 9, 10 with tests. Add a comparison with the Earth Impact Database. |
| WP-2 | VOL, MAG | 115 | a morphometric database of volcanoes; Global Volcanism Program; USGS | de Bremond d'Ars et al. 1995, Scaillet et al. 2008, Szabo & Moore 1986, Bryan & Ernst 2008, Self et al. on Deccan flows | Opus | Nearly all uncited: the work is finding sources and deciding which design values to keep as `game-choice`. Fix 11, 12. |
| WP-3 | ERO | 167 | Landlab and fastscapelib source (in hand) | Braun & Willett 2013, Yuan et al. 2019, Stock & Montgomery 1999, Whipple & Tucker 1999, Harel et al. 2016, Heimsath et al. 1997, Roering et al. 1999, Sklar & Dietrich 2001, Bosscher & Schlager 1992 | Opus | Fix 6, 7, 8. Any change needs recalibration: run with WP-4, after the surface-stage work now in flight lands. |
| WP-4 | ISO, HYP | 73 | ETOPO1 hypsographic table (NOAA), GMT relief | Stein & Stein 1992, Parsons & Sclater 1977, Molnar & Lyon-Caen 1988, Clark & Royden 2000, Cogley 1984 | Opus | Fix 16. Settle one crust density. Re-derive Earth's hypsometry figures from one named dataset. |
| WP-5 | HYD, YNG | 97 | Leopold & Maddock 1953 (in hand), Holdridge 1967 (in hand), Barnes et al. 2014 (in hand), Messager et al. 2016, Verpoorter et al. 2014 | Zhang et al. 2004, Fu 1981, Schumm 1963, Porter 2001, Hock 2003 | Sonnet 5.5 | Fix 5. Unify the two runoff laws. Replace the from-memory sinuosity figure. |
| WP-6 | AMP, GLA | 137 | Ohmura et al. 1992 (in hand), Schott et al. 2024 shaders (in hand), Hergarten 2021 (in hand) | Hallet et al. 1996, Harbor 1992, Humphrey & Raymond 1994, Cuffey & Paterson 2010, Porter 2001, Herman et al. 2015, Egholm et al. 2009 | Sonnet 5.5 | AMP is mostly `game-choice`: confirm each is labelled so. GLA: the erosion law's constant and exponent. |
| WP-7 | RCK, STR | 132 | a petrology text for facies boundaries; a rock-density table | Winter 2010, Yardley 1989, Telford et al. 1990, Schön 2015, Cruden 1998, Hjulström 1935 | Sonnet 5.5 | Fix 15. 41 rock densities and all erodibilities are unsourced judgement: label them. |
| WP-8 | DEF | 74 | Noddy source (in hand, MIT snapshot) | Schlische et al. 1996, Kim & Sanderson 2005, Cowie & Scholz 1992, Anderson 1951 | Opus | Fix 13, 14. The vertical-stretch question for dips and amplitudes is a design decision for Ben. |
| WP-9 | DEP | 204 | USGS reports (open: OFR 2008-1155, Bulletin 1693, SIR 2010-5070, Circular 1178, OFR 2009-1252) | Singer et al. 2005 (Economic Geology), Singer & Menzie 2010 (book), Sillitoe 2005 | Sonnet 5.5 | Fix 3. Tables are open but are PDFs: needs WP-0. |
| WP-10 | ORE | 257 | USGS reports (open), KGS Bulletin 240 (in hand) | Waples 1980 full text, Garnett & Bassett 2005, Leach et al. 2005, Bekker et al. 2010, Heckel 1986 | Opus | Fix 2 (a real bug). Redo the coal size and density derivation from sources. Same code directory as WP-9: run them one after the other or as one agent. |
| WP-11 | BIO | 99 | Peel et al. 2007 (in hand) | UNEP 1997 atlas (book), Thornthwaite 1948 | Sonnet 5.5 | Fix 4. Mostly `game-choice` rows to confirm. |
| WP-12 | SCL, BAK, PRV, POL | 139 | NASA Earth Fact Sheet (in hand), CRUST 2.0 (in hand) | the USGS "Geologic provinces" map page (gone; an archive copy exists), Mallard et al. 2016, Kesler & Wilkinson 2008, Gough 1981 | Sonnet 5.5 | Fix 19, 20. Identify the province map properly or drop the citation. |
| WP-13a | PLT `not-yet-coded` | 29 | – | Forsyth & Uyeda 1975, Turcotte & Schubert, Conrad & Lithgow-Bertelloni 2004 and 2006, Davis et al. 1983, Molnar & Lyon-Caen 1988, Stein & Stein 1992 | Opus | **Before the force balance is built.** Fix 18. Confirm slab pull, continental drag, wedge taper, suction share. |
| WP-13b | PLT rest | 183 | Cortial et al. 2019 (in hand: every Table 3 constant matches) | Zahirovic et al. 2015, Bradley 2008, Stern 2002 | Sonnet 5.5 | **Deferred** until the plate rewrite lands; then ledger the new code and verify. |
| WP-14a | CLM `not-yet-coded` | 28 | Byrne & Goldblatt 2014 (in hand), Hoffman et al. 2017 (in hand), climlab source (in hand) | the six references the design marks "from memory" (all exist; five unread) | Opus | **Before the per-cell climate is built.** Fix 17 in the new code. |
| WP-14b | CLM rest, MOI | 194 | – | North, Cahalan & Coakley 1981 (scan), Walker et al. 1981, Berner 1994 (scan), Berger 1978, Smith & Barstad 2004 | Sonnet 5.5 | **Deferred** until the climate rewrite lands. |
| WP-15 | editor hints and Earth reference figures | not ledgered | NASA, NOAA, USGS fact pages | – | Sonnet 5.5 | Sweep `editor_fields.json` and every "Earth is X" in comments into one table with one source each. |

What can run in parallel: WP-1, WP-2, WP-5, WP-6, WP-7, WP-8, WP-11, WP-12, WP-13a, WP-14a and
WP-15 touch separate code and can run together once WP-0 is done. WP-3 and WP-4 share the
calibration and should be one effort, started after the surface-stage changes now in flight have
landed. WP-9 and WP-10 share the deposits code. WP-13b and WP-14b wait for the rewrites.

Each work package ends with: rows marked `verified`, corrected, or left `uncited` with a note; the
code corrected with a test where a value was wrong; the feature page written from
[_template.md](_template.md); the bibliography entries updated with what was read.
