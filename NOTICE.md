# Third-party notices

Deep Time is licensed LGPL-3.0-or-later. Code ported or adapted from other projects is listed here
with its licence and the Deep Time files that carry it. Every such file also names its source in its
header comment. GPL-licensed tools are used only as unshipped test oracles under `tools/validation/`
and contribute no code.

| Source | Licence | Used for | Deep Time files |
|---|---|---|---|
| [tectonics.js](https://github.com/davidson16807/tectonics.js) by Carl Davidson — `noncompiled/models/lithosphere/RockColumn.js` and `Crust.js` (commit ce371fa) | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | The plate model's rock pools: the seven pools, the conserved felsic family, `Crust.overlap` accretion, the lithification and metamorphosis thresholds, top-down erosion (adapted: metres instead of kg/m², L0 samples instead of rasters, rules rewritten) | `crates/deeptime-core/src/plates/pools.rs`, `crates/deeptime-core/src/plates/state.rs` (`pool`) |

## Build-time terms

- cargo-xwin downloads the Microsoft C runtime and Windows SDK to link the Windows natives. Using it
  accepts Microsoft's licence for them (https://go.microsoft.com/fwlink/?LinkId=2086102), which the
  project owner accepted on 2026-09-28.
