# Hypsometry and calibration targets

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Hypsometry is the distribution of a planet's surface over height: how much is land, how high the land stands, how deep the oceans are. Earth's has two peaks (continents near sea level, ocean floor about 4 km down). The mod's presets are tuned so that an Earth-like run comes out near Earth's figures (about 29 % land, median land height near 430 m, about 10 % of land above 2 km). This page lists every such target, what was tuned to hit it, the Earth figure and where that figure comes from. A calibrated value is honest only if its target is: the targets are ledger rows too.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [HYP](ledger/HYP.md) | hypsometry targets | HYP-001 to HYP-021 (21) |
| [HYP](ledger/HYP.md) | other calibration targets | HYP-022 to HYP-035, HYP-040 to HYP-046 (21) |

Rows by status (the ledger's header): `verified` 1, `cited-unverified` 16, `abstract-only` 2, `calibrated` 16, `game-choice` 1, `uncited` 4.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/ERO.md](extraction-notes/ERO.md).
- Build log: STATUS.md 2026-10-01, "P1-17 hypsometry recalibration"; 2026-09-29 calibration entries.
- Earth relief used: GMT `earth_relief` (SRTM15+, Tozer et al. 2019), at a resolution the documents state three different ways (extraction notes ERO, flag 8).
- Work package in the verification plan: WP-4 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The accepted band for median land height (430 to 640 m) reaches about 50 % above Earth's 427 m.
- Volcano volume is a known contributor to land that stands too high; it was kept as a game choice.
- Since the erosion recalibration of 2026-10-18 the erosion law (K, m, n, κ and S_c: ERO-004 to ERO-007, ERO-013, ERO-023) is set against measured erosion rates (Portenga & Bierman 2011 and Montgomery & Brandon 2002: ERO-171, ERO-172, ERO-179 to ERO-181; n from Ruetenik et al. 2023, ERO-182), no longer against the hypsometry. HYP-001 to HYP-004 stay the reference lines the runs are reported against, as the erosion rates were before; see [erosion-and-sediment.md](erosion-and-sediment.md), "The erosion law against measured rates".

## The continents' hypsometry against Earth's (2026-10-22)

Measured for V1 item 1's third criterion (continental crust under the sea, at most 0.25) on
twelve runs of `earthlike_quick_16k`: master df40ce80 (the "master" column) and, after Ben moved W
to the model sphere on 2026-10-22 (D6), master 10bafdae against the branch that did it (the
"W on the model sphere" column; PLT-400 on in the same change). The full account, the arms and the
sources are in [sea-level.md](sea-level.md), "The continents under the sea and the sea's height at
the end". Earth's figures are HYP-042 to HYP-046 (CRUST1.0's crust at least 20 km thick, the
model's own continent-ocean boundary).

| Final frame, median of 12 | master | W on the model sphere | Earth |
|---|---|---|---|
| Continental crust under the sea | 0.414 | 0.278 | 0.237 to 0.248 at 20 km (0.195 at 25 km; 0.306 by Cogley 1984's contacts) |
| Its surface over the sea, mean / median, m | 381 / 260 | 737 / 572 | 311 / 220 |
| Share 0 to 200 m / 200 to 2,000 m / deeper than 2,000 m under the sea | 0.104 / 0.200 / 0.089 | 0.087 / 0.115 / 0.069 | 0.112 / 0.148 / 0.010 |
| Crust at least 25 km thick above the sea | 0.572 | 0.677 | 0.805 to 0.817 |
| Thickness at which the median surface crosses the sea, km | 35.1 (31.2 at the datum) | 37.0 | 31.0 |
| Land's mean height, m | 1,284 | 1,374 | 797 |
| The sea over the datum, m | 538 | 820 | 0 |

In the datum's frame the continents stood where Earth's do; the sea stands high on them because
the ocean basins are shallow (oceanic-labelled crust at least 10 km thick over 0.325 of the
planet, mostly sedimentary rock: no sediment subducts). The extra drowned crust was on the slope
and below it, in the transition's continental half: W was 200 km on the planet, eight times
Earth's margins on the model sphere. With W at 200 km on the model sphere (the shipped meaning
since 2026-10-22) the share under the sea falls to 0.278 (p < 0.001), the slope share to 0.115,
below Earth's 0.148, and the shelf's to 0.087 (Earth 0.112). The continents are thicker and stand
higher (mean height over the sea 737 m against Earth's 311), and the sea rises with them (820 m
over the datum): the sediment they shed still has no way out of the ocean (sea-level.md).
