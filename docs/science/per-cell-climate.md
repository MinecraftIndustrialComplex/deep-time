# Per-cell climate: the design's constants (not yet built)

Status of this page: `stage 2 (constants checked against sources on 2026-10-02)` for the sources
the project's library could obtain; the rest is marked in the ledger as it stands.

Ledger ids on this page: `CLM-119` to `CLM-146` and `CLM-163` to `CLM-166`, plus the coded rows
the design carries over (CLM-001, CLM-003 to CLM-005, CLM-007 to CLM-010, CLM-013, CLM-014,
CLM-020, CLM-069, CLM-070, CLM-093, CLM-095, CLM-100) (see [claims.md](claims.md)). Today's banded
climate is described in [climate.md](climate.md).

## What you see in the world

Nothing yet. The per-cell climate (`docs/design/p1-26-per-cell-climate.md`, approved by Ben on
2026-10-01) is designed and prototyped but not built. Worlds still use the banded climate, in which
temperature and sea ice are one value per 2-degree latitude band.

When it is built, a player should notice: sea-ice edges that are ragged and move with the seasons,
frozen-ocean biomes that follow currents and coasts; cold continental interiors and mild windward
coasts; deserts inland and on cold west coasts; and a polar continent with an icy interior and
milder coasts. The baked climate layers and the climate API do not change.

## The model in plain words

The climate stage takes a snapshot of the climate every 10 million years of the planet's history.
In the new design each snapshot runs on a coarse latitude–longitude grid of 128 by 64 cells
(2.8 degrees, about 313 km at the equator). Every cell has a land part and a sea part, a 50-m slab
of ocean below the sea, and its own sea ice. Through 72 steps of a model year each cell:

1. absorbs sunlight according to its latitude, the season and its albedo (brighter for snow, ice
   and desert);
2. radiates heat to space as a straight-line function of its temperature, lowered by CO₂;
3. trades heat with its neighbours by diffusion, faster over the tropics than near the poles, and
   faster over land near the equator;
4. has heat carried east or west by a prevailing low-level wind;
5. in the sea part, has heat carried by wind-driven gyres (Stommel's model of the western boundary
   currents), by an overturning "conveyor" that sinks in the narrowest polar-connected basin, and
   by ocean mixing;
6. freezes or melts sea ice from its own heat budget.

Rain comes, once a switch is turned (`climate.rain_model = budget`), from a monthly water-vapour
budget on the same grid: wind carries vapour, the sea and wet soil evaporate it, and it rains out
fast when the column is near saturation. Mountain rain stays on the finer L1 grid (Smith & Barstad),
and its rain-out is taken from the budget, which makes rain shadows. Until the budget beats today's
rain belts on Earth's relief, rain stays on the belts, fed by the new temperatures.

The results are brought down to the 16- or 32-block L1 cells with a coast blend and a winter
inversion term for snowy, dark continental interiors. Between snapshots a cheap one-dimensional
model (one value per latitude band) keeps time for the carbon cycle and the ice, corrected at each
snapshot to the two-dimensional model's zonal means.

## Equations and constants

The design values as they now stand. "Changed" means WP-14a corrected the design document to its
source.

| Ledger id | Quantity | Design value | Units | Source | Status |
|---|---|---|---|---|---|
| CLM-119 | climate grid | 128 × 64 (TransEBM's T42 has the same count) | cells | Ziegler & Rehfeld 2021 p. 2845 | `verified` |
| CLM-120 | surface-dependent diffusivity, D_x(φ) = D_pole + (D_eq − D_pole) cos⁵φ | sea 0.65 → 0.40; land 0.65 → 0.28 N / 0.20 S (**changed**: was constant 0.40 at sea and a sin⁵φ land profile) | W/m²/K | Ziegler & Rehfeld 2021 eq. 2, Table 1; Zhuang et al.'s code | `verified` |
| CLM-121 | east–west diffusion factor | 0.5 (PlaHab: 1/25 below 45°, 1/2,500 above) | – | Ramirez 2024 p. 3 | `calibrated` |
| CLM-122 | land heat capacity (land plus air) | 3 m of water (North et al.: 2.5 m) | m | North, Cahalan & Coakley 1981 p. 118; Ziegler & Rehfeld 2021 Tables 1–2 | `verified` |
| CLM-123 | desert brightening at the top of the atmosphere | +0.08 (**changed** from +0.05) | – | Donohoe & Battisti 2011; Stull 2017 Table 2-5 | `verified` |
| CLM-124 | 850 hPa zonal wind profile | −6, +9, −2 m/s bumps at 12°, 45°, 82° | m/s | none; compared with reanalysis (CLM-164) | `calibrated` |
| CLM-125 | share of wind speed carrying heat, β_T | 0.5 | – | prototype calibration | `calibrated` |
| CLM-126 | mountain blocking transmission | e^(−max(0, z − 1 km)/1 km), off | – | none | `uncited` |
| CLM-127 | ocean eddy diffusivity | 0.8 | W/m²/K | prototype calibration | `calibrated` |
| CLM-128 | Stommel gyres: drag coefficient C_D | 1.2·10⁻³ (**changed** from 1.3·10⁻³); friction r = 1.5 β Δx | – | Large & Yeager 2004 eq. 6a; Stommel 1948 not read | `cited-unverified` |
| CLM-129 | gyres stop at 75° and in zonal channels | 75° | deg | numerics | `game-choice` |
| CLM-130 | gyre heat efficiency γ_o | 1.0 | – | prototype calibration | `calibrated` |
| CLM-131 | overturning strength | 15 Sv (Earth 17.2 Sv) | Sv | McCarthy et al. 2015; Frajka-Williams et al. 2019 | `calibrated` |
| CLM-132 | sinking in the narrowest polar-connected basin | 20–70° | deg | Ferreira et al. 2018 pp. 337, 344 | `verified` |
| CLM-163 | deep-water temperature; upwelling area | 2 °C; sea equatorward of 50° | °C; deg | none | `uncited` |
| CLM-133 | northern ocean heat transport target | 1.5–2 PW at 15–25°N | PW | Trenberth & Caron 2001 Fig. 7 | `verified` |
| CLM-134 | winter inversion I_max | 15 K | K | Serreze et al. 1992 not read; reanalysis (CLM-165) | `from-memory` |
| CLM-135 | sea-ice edge sharpness on L1 | logistic slope 4 | – | look | `game-choice` |
| CLM-136 | flux correction cap | 15 W/m² | W/m² | numerics | `game-choice` |
| CLM-137 | triggered snapshot thresholds | 3 K; 0.15 | K; – | numerics | `game-choice` |
| CLM-138 | Earth's global mean target at 280 ppm | 15 ± 0.5 (Earth 1850–1900: 13.8–14.4) | °C | Rohde & Hausfather 2020 data, computed | `calibrated` |
| CLM-139 | eddy moisture diffusivity | 1.5·10⁶ (GREB 2–8·10⁵) | m²/s | none | `uncited` |
| CLM-140 | three-cell surface meridional wind | 2.5, 1.0, 0.5 m/s | m/s | none; compared with reanalysis (CLM-164) | `uncited` |
| CLM-141 | thermal wind: κ_p and friction r | 60 Pa/K; 10⁻⁵ s⁻¹ | Pa/K; s⁻¹ | Lindzen & Nigam 1987 not read | `from-memory` |
| CLM-142 | bulk evaporation, wind | \|V\| + 3 m/s sea, + 2 m/s land (**changed** from a 3 m/s floor); C_E 1.3·10⁻³ unverified | – | Dommenget & Flöter 2011 eq. 7 | `verified` (form) |
| CLM-143 | soil bucket | β = min(1, S/(0.75 S_max)), S_max 150 mm | mm | Manabe 1969 eqs. 18, 20, p. 743 | `verified` |
| CLM-144 | saturation column vapour | 70 kg/m² · e^(0.065(T − 27)) · scale height 2,500 m | kg/m² | consistency only | `uncited` |
| CLM-145 | rain law | a = 11.4, τ_P = 6.6 days, r_ref 0.7 (**changed** from 15.6 and 8 days) | – | Bretherton et al. 2004 eq. 2 | `verified` |
| CLM-146 | rain suppression over cold seas | e^(0.3 ΔT) | – | none | `uncited` |
| CLM-164 | Earth's zonal-mean low-level winds | see the row | m/s | NCEP/NCAR reanalysis, computed | `verified` |
| CLM-165 | Earth's January inversion over Siberia | 6–9.5 K below 850 hPa | K | NCEP/NCAR reanalysis, computed | `verified` |
| CLM-166 | snowball escape CO₂ to report against | 10⁴–10⁵ ppm | ppm | Hoffman et al. 2017 p. 3 | `verified` |

The main equations, as the design now writes them:

- Sea and land surface: C ∂T/∂t = (1 − α) Q − (A + B T) + exchange + transport. A = 203.3 W/m² and
  B = 2.09 W/m²/K (North, Cahalan & Coakley 1981 p. 93, CLM-003, CLM-004), with A lowered by the
  CO₂ forcing (CLM-069, CLM-070).
- Diffusion: D_face = (1 − f̄) D_ocean(φ) + f̄ D_land(φ), with D_x(φ) = D_x,pole + (D_x,eq −
  D_x,pole) cos⁵ φ and f̄ the land fraction of the two cells sharing the face.
- Rain: P = stab · (W/τ_P) · exp(a (r − r_ref)), with r = W/W_sat the column relative humidity.
- Evaporation: E = ρ_a C_E (|V| + u_t) q_s(T) (1 − r), times the soil factor min(1, S/(0.75 S_max))
  on land.

## Earth against the mod

The per-cell climate is not built, so there is no measurement of the mod. The design's prototype
(scratch code on Earth's relief, not committed) is reported in the design document, Appendix B. The
Earth figures this package established:

| Quantity | Earth | Source of the Earth figure | Design |
|---|---|---|---|
| Global mean surface temperature, 1850–1900 | 13.8 °C (air over sea ice), 14.4 °C (water below) | Berkeley Earth data file, 1951–1980 mean and anomalies, averaged in WP-14a | target 15 ± 0.5 °C |
| Atlantic overturning at 26°N | 17.2 Sv (2004–2012); 17.0 ± 4.4 Sv (2004–2017) | McCarthy et al. 2015; Frajka-Williams et al. 2019, read | 15 Sv |
| Northern ocean heat transport peak | about 2 PW near 15–20°N | Trenberth & Caron 2001 Fig. 7, read by eye | target 1.5–2 PW |
| Atmospheric heat transport peak | 5.0 ± 0.14 PW at 43°N | Trenberth & Caron 2001, abstract | not targeted yet (T9) |
| 850 hPa westerly peak (annual zonal mean) | 4.9 m/s at 45°N; 12.7 m/s at 50°S | NCEP/NCAR reanalysis 1991–2020, computed | 9 m/s at 45° |
| 850 hPa trades | −3.8 m/s at 15°N; −4.5 m/s at 15°S | same | −5.7 m/s at 15° |
| Near-surface trades, meridional | 1.4–1.5 m/s (N); 2.0 m/s (S) | same, σ 0.995 | 2.5 m/s |
| January σ 0.995 minus 850 hPa, Yakutsk | −8.1 K (−17 K against a 6.5 K/km lapse) | same | inversion up to 15 K |
| Snowball terminations | 10⁴–10⁵ ppm CO₂ | Hoffman et al. 2017 | today's 1-D model: about 5.5·10⁵ ppm |

## Simplifications and limits

- The diffusivities of Zhuang et al. and North, Cahalan & Coakley are total transport (atmosphere
  and ocean) in models with one temperature. The design adds its own ocean transport, so those
  values are an upper bound for its atmosphere and must be calibrated down (T9).
- The rain law is a fit over the tropical oceans (Bretherton et al. 2004), applied everywhere.
- The overturning's sinking basin is chosen by width; the real asymmetry is salinity, which the
  model does not carry. Width is one route to it (Ferreira et al. 2018).
- No clouds, no salinity, no sea-ice drift, no coastal upwelling (design §2.6, Q9).
- The wind profiles are symmetric between the hemispheres; Earth's southern westerlies are about
  2.5 times its northern ones at 850 hPa.
- The CO₂ forcing is a fit computed up to 50,000 ppm and used beyond it.

## Game choices

- CLM-129: the gyres stop at 75° and in zonal channels (numerics; the design agent, build log
  2026-09-30, P1-26 design).
- CLM-135: how crisp a sea-ice edge is drawn on L1 (look; same).
- CLM-136, CLM-137: the flux-correction cap and the thresholds for an extra snapshot (numerics;
  same).
- The grid resolution (CLM-119) is Ben's choice (decisions log 2026-10-01, Q4).

## Open questions

- **Earth's mean temperature target** (CLM-138): 15 ± 0.5 °C or Earth's 13.8–14.4 °C? Question for
  Ben in STATUS.
- **CO₂ above 50,000 ppm and the snowball's escape** (CLM-070, CLM-166): question for Ben in STATUS.
- **Köppen on L2** (grid-artifact audit, finding 3): the design does not include it but allows it
  without a format change (design §5.2).
- Not readable in WP-14a, so these stay unverified: Stommel 1948 (the gyre equation and boundary
  width), Lindzen & Nigam 1987 (κ_p's lineage and the friction r), Serreze, Kahl & Schnell 1992
  (the inversion), North, Mengel & Short 1983, Zhuang et al. 2017's paper text (its code was
  read), Murray 1967 (Tetens), Large & Pond 1981.
- C_E = 1.3·10⁻³ (CLM-142) has no source; GREB does not number its C_w.
- K_q (CLM-139) is 2 to 7 times GREB's κ, whose own paper gives two values.
- The 50-m slab is shallower than the sources' 70–96 m (CLM-013); T9 may revisit it against the
  seasonal sea-surface amplitude.
- The deep-water temperature of 2 °C (CLM-163) has no source.

## Sources

- Ziegler2021: read in full (library). Zhuang2017: the paper not obtained; its released code read
  for the constants (GPL-3, nothing copied).
- North1981: read in full (scan with text layer). Ramirez2024, Stassen2019, Dommenget2011,
  Bretherton2004, Manabe1969, Ferreira2018, Trenberth2001, WagnerEisenman2015 (bibliography key
  Wagner2015), DonohoeBattisti2011 (accepted manuscript), McCarthy2015 (accepted manuscript),
  FrajkaWilliams2019, BackBretherton2009, LargeYeager2004, IPCC2001_ch6, Byrne2014, Hoffman2017,
  HeldSoden2006: read in full or in the passages named in the rows.
- Rohde2020: the Berkeley Earth data file read and averaged. Kalnay1996: the NCEP/NCAR reanalysis
  long-term means read and averaged (the paper not fetched).
- Stull2017, Norton1979, Guilloteau2000: textbook, report and lecture notes, read for single values.
- climlab (Rose2018): the MIT source read for its defaults.
- Not obtained: Stommel1948, Lindzen1987, Serreze1992, North1983, Murray1967 (Tetens), Large &
  Pond 1981.
