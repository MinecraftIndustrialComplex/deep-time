# Third-party notices

Deep Time is licensed LGPL-3.0-or-later. Code ported or adapted from other projects is listed here
with its licence and the Deep Time files that carry it. Every such file also names its source in its
header comment. GPL-licensed tools are used only as unshipped test oracles under `tools/validation/`
and contribute no code.

| Source | Licence | Used for | Deep Time files |
|---|---|---|---|
| [tectonics.js](https://github.com/davidson16807/tectonics.js) by Carl Davidson — `noncompiled/models/lithosphere/RockColumn.js` and `Crust.js` (commit ce371fa) | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | The plate model's rock pools: the seven pools, the conserved felsic family, `Crust.overlap` accretion, the lithification and metamorphosis thresholds, top-down erosion (adapted: metres instead of kg/m², L0 samples instead of rasters, rules rewritten) | `crates/deeptime-core/src/plates/pools.rs`, `crates/deeptime-core/src/plates/state.rs` (`pool`) |

## Data

- **U.S. Geological Survey grade–tonnage models and reports** (public domain, works of the U.S.
  Government; attribution kept as good practice): OFR 2008-1155 (porphyry copper), OFR 2009-1252
  (sediment-hosted Zn–Pb), Bulletin 1693 models 31a, 32a/b, 34a and 39a, SIR 2010-5070-A, -B and -N,
  Circular 891 (coal), OFR 95-831, PP 1625-A, PP 1698, OFR 88-450-G, OFR 94-220, OFR 2003-253,
  DDS-60 and DDS-69-B. Digitised in P1-14, with the table or figure and page of every number, into
  `crates/deeptime-core/data/usgs/*.json`; the deposit types in
  `neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/` copy them. Facts cited from
  journal articles (Waples 1980, Butterworth et al. 2016, Hoggard et al. 2020, Sillitoe 2005, …)
  are numbers, not redistributed text.

## Build-time terms

- cargo-xwin downloads the Microsoft C runtime and Windows SDK to link the Windows natives. Using it
  accepts Microsoft's licence for them (https://go.microsoft.com/fwlink/?LinkId=2086102), which the
  project owner accepted on 2026-09-28.
