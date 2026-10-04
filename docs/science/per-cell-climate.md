# Per-cell climate

Status of this page: `stage 2 (constants checked against sources on 2026-10-02)` for the sources the
project's library could obtain; the rest is marked in the ledger as it stands. Rewritten on
2026-10-03 for the model as built (P1-26 T1 to T6 and the first part of T9; design document
section 11, "As built"). The Earth comparison below is from the T9 calibration runs on Earth's
relief; the rain budget is built but not calibrated.

Ledger ids on this page: `CLM-119` to `CLM-146`, `CLM-163` to `CLM-168`, `CLM-170`, `CLM-172`,
`CLM-175` to `CLM-179`, plus the rows the build changed (CLM-061, CLM-069, CLM-070 and CLM-071
(retired), CLM-072, CLM-100) and the coded rows the model carries over (CLM-001, CLM-003 to
CLM-005, CLM-007 to CLM-010, CLM-013, CLM-014, CLM-017, CLM-020, CLM-093, CLM-095) (see
[claims.md](claims.md)). CLM-169, CLM-171, CLM-173 and CLM-174 are unused. The banded climate
(`climate.model = banded`) is described in [climate.md](climate.md).

## What you see in the world

Worlds are made with the per-cell climate by default (`climate.model = per_cell`, ledger CLM-175).
Temperature is no longer one value per 2-degree latitude band. Each coarse cell has its own land
and sea temperature, and the sea has its own ice. What a player meets because of that:

- **Sea-ice edges that are ragged**, not one latitude all round the planet. On Earth's relief the
  southern winter ice edge varies by 3.3 degrees of latitude between longitudes (the banded model:
  0.25 degrees). Frozen-ocean biomes follow the edge.
- **Cold continental interiors and milder windward coasts.** The winter spread of land temperature
  along a latitude circle is about twice the banded model's, and the coast-to-interior contrast of
  Eurasia in January is 16 K (banded 6.9 K). Earth's is larger still (see "Earth against the
  mod"): the effect is real but weaker than on Earth.
- **Biome borders that do not step with the 16-block L1 cells.** The Köppen class is decided on
  L2 from the twelve months, each interpolated and lapsed to its own height (decisions log
  2026-10-02).
- **Rain is still the old belts**, fed by the new temperatures. Deserts and wet coasts have not
  yet moved: Mumbai is dry for an Indian monsoon city and the Sahara is wet for a desert (below).
  The moisture budget that should fix this is built (`climate.rain_model = budget`) but is not
  the default, and has not been calibrated.

The baked climate layers and the climate API (mic-climate) are unchanged. Seasonal sea ice is not
baked: only the annual mean (`climate/sea_ice`) is a world layer; the winter maximum and summer
minimum are pipeline and keyframe fields.

## The model in plain words

The climate stage takes a snapshot of the climate every 10 million years of the planet's history,
and in between a cheap one-dimensional model keeps time. A snapshot runs in this order
(`crates/deeptime-core/src/climate/`: `snapshot.rs`, `percell.rs`):

1. **The coarse grid.** L1 is coarsened into blocks of k × k cells (`coarse.rs`, `percell.rs`:
   `climate_grid`). The target is 128 columns by 64 rows (2.8 degrees, about 313 km at the equator);
   k is the largest divisor of L1's width that gives at least that many columns. On a 1024 × 512 L1
   it is exactly 128 × 64; on Earth's 720 × 360 relief k is 5 and the grid 144 × 72 (2.5 degrees).
   Areas, face lengths and distances are exact on the sphere, so every diffusion conserves heat.
2. **The zonal model first**, as before (one value per band): it gives the 2-D model its first
   state, its sea-ice cover and the ITCZ.
3. **The 2-D energy balance** (`ebm2d.rs`). Every cell holds a land tile and a sea tile (each with
   the air above, heat capacity of 3 m of water), a 50-m slab of ocean under the sea tile, and sea
   ice as latent heat. It runs 72 steps of a model year, until the annual means settle. In each
   step a cell:
   - absorbs sunlight by latitude, season and albedo (brighter for snow, ice, ice sheets and, a
     little, desert, which is the previous snapshot's Köppen B land);
   - radiates A + B T less the CO₂ forcing at its own temperature (`co2.rs`);
   - exchanges heat between its land and sea tiles;
   - has its mean temperature diffused to its neighbours, by a diffusivity that falls from equator
     to pole as the fifth power of the cosine of latitude, differently for land and sea, and slower
     east-west than north-south;
   - has the whole row shifted east or west by the prevailing low-level wind, at a share of the
     wind's speed (this carries mild coastal air inland);
   - in the sea part, has slab heat moved by eddy diffusion (not across land), by wind-driven
     gyres (`ocean2d.rs`: Stommel's balance of wind stress, friction and the planet's curvature,
     solved on the real coastlines) and by an overturning conveyor that sinks in the narrowest
     polar-connected basin and upwells over the sea equatorward of 50 degrees;
   - freezes or melts sea ice from its own heat budget.
4. **Rain.** By default (`rain_model = belts`) the belt rain of the banded model, with the ITCZ
   and the sea's departure from its row taken from the 2-D model. Optionally
   (`rain_model = budget`, `moisture.rs`) a monthly water-vapour budget: winds, evaporation from
   sea and a soil bucket, and precipitation that grows with the column's relative humidity; the
   mountain rain of L1 is taken out of it as a sink, which makes rain shadows.
5. **Down to L1** (`percell.rs`): the masked bilinear of the coarse land and sea tiles, a coast
   blend within `coast_km`, and a winter inversion that makes snow-covered, dark, continental
   interiors colder at the surface than the air the model carries. Sea ice is sharpened to a
   crisper edge. Köppen, aridity and the glaciers' equilibrium line follow as before.
6. **The clock** (every 200 kyr) stays one-dimensional. At each snapshot it is corrected to the
   2-D model's zonal means: first a diffusivity per face between bands that carries the 2-D heat
   transport, then a flux per band and per tile that closes the rest (`percell::fit_flux`). A
   snowball or hothouse crossing (3 K of global mean or 0.15 of sea ice since the last snapshot)
   triggers an extra 2-D snapshot, and the 2-D model restarts from the zonal state when the two
   have drawn apart by those amounts.

CO₂ no longer follows a single logarithmic law. The forcing is read from a radiative-convective
model's table (HEXTOR's copy of Kopparapu et al. 2013's) at every tile's own temperature, scaled so
a doubling at 288.15 K gives 3.71 W/m². The ice sheets and Nye's profile take the planet's own
gravity.

## Equations and constants

Values as coded. "Calibrated" rows name what they were calibrated against in the ledger. Run
defaults are in `ClimateParams::default()` and the presets.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| CLM-175 | which climate runs | `per_cell`; `banded` is a setting | – | Ben, decisions log 2026-10-01 | `game-choice` |
| CLM-119 | coarse grid | 128 columns target (k × k L1 blocks); 128 × 64 on L1 1024 × 512, 144 × 72 on Earth's 720 × 360 | cells | Ziegler & Rehfeld 2021 p. 2845 (TransEBM's T42) | `verified` |
| CLM-120 | surface-dependent diffusivity, D_x(φ) = D_pole + (D_eq − D_pole) cos⁵φ | sea 0.65 → 0.40; land 0.65 → 0.28 N / 0.20 S; × CLM-176 | W/m²/K | Ziegler & Rehfeld 2021 eq. 2, Table 1; Zhuang et al.'s code | `verified` |
| CLM-176 | `atmosphere_diffusivity_scale` | 1.3 | – | calibration (global mean 14.1 ± 0.4, winter spread) | `calibrated` |
| CLM-121 | east-west diffusion factor | 0.2 (PlaHab: 1/25 below 45°, 1/2,500 above) | – | Ramirez 2024 p. 3 (for comparison) | `calibrated` |
| CLM-122 | land heat capacity (land plus air), the zonal model's | 3 m of water (North et al.: 2.5 m) | m | North, Cahalan & Coakley 1981 p. 118; Ziegler & Rehfeld 2021 Tables 1–2 | `verified` |
| CLM-013, CLM-014 | slab depth; land depth | 50 m; 3 m | m | none (consistent with the sources, see rows) | `uncited` |
| CLM-123 | `desert_albedo`, desert share of land from the last Köppen | +0.08 | – | Donohoe & Battisti 2011; Stull 2017 Table 2-5 | `verified` |
| CLM-124 | 850 hPa zonal wind profile | −6, +9, −2 m/s bumps at 12°, 45°, 82° | m/s | none; compared with reanalysis (CLM-164) | `calibrated` |
| CLM-125 | `wind_heat_transport` β_T | 0.2 | – | calibration on Earth's relief | `calibrated` |
| CLM-126 | mountain blocking | **not built** | – | none | `uncited`, `not-yet-coded` |
| CLM-127 | ocean eddy diffusivity (CLM-017's value) | 0.3 (the prototype's 0.8 is not used) | W/m²/K | none | `uncited` |
| CLM-128 | Stommel gyres: C_D; friction r | 1.2·10⁻³; 1.5 β Δx | – | Large & Yeager 2004 eq. 6a; Stommel 1948 not read | `cited-unverified` |
| CLM-177 | ρ_sea; ρ_air | 1025; 1.2 | kg/m³ | from memory | `from-memory` |
| CLM-129 | gyres stop at 75° and on rows with no land | 75° | deg | numerics | `game-choice` |
| CLM-130 | `gyre_heat_efficiency` γ_o | 1.0 | – | prototype's January sea surface | `calibrated` |
| CLM-131 | `overturning_sv` | 30 Sv (Earth 17.2 Sv): compensating | Sv | McCarthy et al. 2015; Frajka-Williams et al. 2019 (Earth) | `calibrated` |
| CLM-132 | sinking basin: narrowest of 20–70° that reaches 30° and holds ice-free sea poleward of 50° in late winter | 20–70°, 30°, 50° | deg | Ferreira et al. 2018 pp. 337, 344 | `verified` |
| CLM-163 | deep-water temperature; upwelling area | 2 °C; sea equatorward of 50° | °C; deg | none | `uncited` |
| CLM-133 | northern ocean heat transport target | 1.5–2 PW at 15–25°N; **as built 3.2 PW at 27°N** | PW | Trenberth & Caron 2001 Fig. 7 | `verified` (the target; not met) |
| CLM-134 | winter inversion `winter_inversion_c` I_max; dark below 150 W/m²; interior = land within 1,000 km | 15 K; 150 W/m²; 1,000 km | K | Serreze et al. 1992 not read; reanalysis (CLM-165) | `from-memory` |
| CLM-135 | `sea_ice_edge_sharpness` on L1 | logistic slope 4 | – | look | `game-choice` |
| CLM-136 | flux correction: fitted face D, then flux per band and tile | cap 60 W/m² | W/m²; W/m²/K | numerics | `game-choice` |
| CLM-179 | its numerics | D 0.05–5 W/m²/K where the gradient > 3 K/rad; gain 0.8 B; 3 years × at most 31 trials; stop at 0.1 K RMS, 0.3 K worst | various | numerics | `game-choice` |
| CLM-137 | triggered snapshot; restart of the 2-D model | 3 K; 0.15 | K; – | numerics | `game-choice` |
| CLM-138 | global mean at 280 ppm | target 14.1 ± 0.4; as built 14.0 | °C | Ben, decisions log 2026-10-02; Berkeley Earth (Rohde & Hausfather 2020) | `calibrated` |
| CLM-167 | CO₂ forcing F(C, T) = m [OLR_RC(T, 280 ppm) − OLR_RC(T, C)] | table of 19 T × 91 pCO₂ (190–370 K; 10⁻⁴–10 bar) | W/m² | HEXTOR (Haqq-Misra & Hayworth 2022); Kopparapu et al. 2013 | `verified` |
| CLM-168 | its scale m so that 2 × CO₂ at 288.15 K gives 5.35 ln 2 | 0.903 | – | Myhre et al. 1998 via IPCC 2001 (CLM-069) | `calibrated` |
| CLM-170 | the forcing below the table (about 100 ppm) | logarithm, the table's 100–200 ppm slope | W/m² | none | `game-choice` |
| CLM-172 | `snowball_cloud_w_m2`: extra cloud greenhouse on frozen surfaces | 0 (setting, 0–20) | W/m² | Ben 2026-10-02; Hoffman 2017, Abbot 2012, Hu 2011, Pierrehumbert 2011 | `game-choice` |
| CLM-061 | `KERNEL_EARTH`, the weathering kernel on Earth's 1° relief | 2.475·10⁷ (banded: 2.845·10⁷) | km² m/yr | re-measured, `deeptime climate-dem` | `calibrated` |
| CLM-100 | gravity of the ice | the planet's | m/s² | – | `uncited` |
| CLM-178 | `rain_model` | `belts`; `budget` is built | – | Ben, decisions log 2026-10-01 (Q2) | `game-choice` |
| CLM-139 | eddy moisture diffusivity K_q | 1.5·10⁶ (GREB 2–8·10⁵) | m²/s | none | `uncited` |
| CLM-140 | three-cell surface meridional wind | 2.5, 1.0, 0.5 m/s | m/s | none; compared with reanalysis (CLM-164) | `uncited` |
| CLM-141 | thermal wind: κ_p and friction r | 60 Pa/K; 10⁻⁵ s⁻¹ | Pa/K; s⁻¹ | Lindzen & Nigam 1987 not read | `from-memory` |
| CLM-142 | bulk evaporation, wind | \|V\| + 3 m/s sea, + 2 m/s land; C_E 1.3·10⁻³ unverified | – | Dommenget & Flöter 2011 eq. 7 | `verified` (form) |
| CLM-143 | soil bucket | β = min(1, S/(0.75 S_max)), S_max 150 mm | mm | Manabe 1969 eqs. 18, 20, p. 743 | `verified` |
| CLM-144 | saturation column vapour | 70 kg/m² · e^(0.065(T − 27)) · scale height 2,500 m | kg/m² | consistency only | `uncited` |
| CLM-145 | rain law | a = 11.4, τ_P = 6.6 days, r_ref 0.7 | – | Bretherton et al. 2004 eq. 2 | `verified` |
| CLM-146 | rain suppression over cold seas | e^(0.3 ΔT) | – | none | `uncited` |
| CLM-164, CLM-165 | Earth's low-level winds; Siberian January inversion (validation references) | see the rows | m/s; K | NCEP/NCAR reanalysis, computed | `verified` |
| CLM-166 | snowball escape CO₂ to report against | 10⁴–10⁵ ppm | ppm | Hoffman et al. 2017 p. 3 | `verified` |

The main equations, as the code computes them:

- Land and sea tiles of a cell (f the land share, x the tile exchange, g(h) the sea-ice-dependent
  coupling to the slab, 𝒜 the atmosphere's transport):

  ```text
  C_l ∂T_l/∂t = (1 − α_l) Q − OLR(T_l) − x (1 − f)(T_l − T_o)               + 𝒜
  C_l ∂T_o/∂t = (1 − α_o) Q − OLR(T_o) − x f (T_o − T_l) − g(h)(T_o − T_m)  + 𝒜
  C_m ∂T_m/∂t = g(h)(T_o − T_m) + 𝒪 − (freezing and melting at −1.8 °C)
  OLR(T)      = A + B T − F(C, T) − c_snow · frozen share      A = 203.3 W/m², B = 2.09 W/m²/K
  ```

  Q is the insolation of the latitude and day, α the albedo (CLM-007 to CLM-010 and CLM-123), 𝒜
  acts on the cell mean T̄ = f T_l + (1 − f) T_o and both tiles take its change, 𝒪 is the ocean's
  transport.
- Atmosphere: diffusion in flux form on the sphere with D_face = (1 − f̄) D_sea(φ) + f̄ D_land(φ),
  D_x(φ) = D_x,pole + (D_x,eq − D_x,pole) cos⁵ φ, all × `atmosphere_diffusivity_scale`, the
  east-west faces × `zonal_diffusivity_ratio`; then an exact shift of each row by
  β_T u_a Δt / Δx cells with u_a the 850 hPa profile of CLM-124.
- Gyres: r ∇²ψ + β ∂ψ/∂x = curl τ / ρ₀, with τ = ρ_a C_D |u| u, β = 2Ω cos φ / R and r = 1.5 β Δx,
  ψ = 0 on the coasts, poleward of 75° and on rows without land. The conveyor sinks Ψ
  (`overturning_sv`) in the chosen basin and upwells it evenly over the connected sea equatorward
  of 50°, carrying heat ρ c_p F (T_upwind − T_deep) with T_deep 2 °C.
- Rain, when `rain_model = budget`: ∇·(V W) − ∇·(K_q ∇W) = E − P − S_oro, with
  E = ρ_a C_E (|V| + u_t) q_s(T) (1 − W/W_sat) (times β(S) on land) and
  P = stab (W/τ_P) exp(a (W/W_sat − r_ref)).
- Winter inversion on L1: ΔT = I_max · dark · snow · interior, dark = clamp(1 − Q/150 W/m², 0, 1),
  interior = clamp((f₁₀₀₀ − 0.5)/0.5, 0, 1).
- CO₂ forcing: F(C, T) = m [OLR_RC(T, 280 ppm) − OLR_RC(T, C)], OLR_RC bilinear in T and
  log₁₀ pCO₂ (pCO₂ = C · 10⁻⁶ bar), m = 5.35 ln 2 / [OLR_RC(288.15 K, 280 ppm) −
  OLR_RC(288.15 K, 560 ppm)] = 0.903. Each tile's longwave is linearized about its temperature at
  the step's start.

## Earth against the mod

Run: `deeptime climate-dem` on Earth's relief (GMT `earth_relief_30m_p`, 720 × 360, 0.5 degrees,
resampled onto the planet's grid: Lambert's cylindrical equal-area projection, 720 × 240 cells, a
144 × 48 climate grid; the ice feedback repeated six times), 280 ppm, the calibrated defaults of
CLM-121, CLM-125, CLM-131 and CLM-176, build log 2026-10-03 (T9); measured with
`tools/validation/climate_vs_obs.py`. The banded climate on the same relief is in brackets.
Earth's temperature and spread figures are the design's section 7.1 table: Beck et al. 2023's
monthly temperatures (1961–1990, 0.5 degrees), brought to sea level with 6.5 K/km; the grid
itself is read by the design's scratch script only, and no observed data are in the repository.

| Quantity | Earth | Source of the Earth figure | Design target | The mod (banded) |
|---|---|---|---|---|
| Global mean temperature, 280 ppm | 13.8–14.4 °C (1850–1900) | Berkeley Earth, averaged in WP-14a (CLM-138) | 14.1 ± 0.4 (Ben, 2026-10-02) | 13.8 °C (15.3) |
| Land mean temperature | – | not recorded | none | 7.9 °C (7.8) |
| Glaciated share of land | about 0.10 (Antarctica, Greenland) | `ice_sheet_feedback` comment in params.rs (a round value) | none | 0.095 (0.101) |
| Land sea-level T, along-row std at 30–70°, January north / July south | 6.1 K | design §7.1 (Beck et al. 2023) | ≥ 5.0 K | 4.53 K (2.25) |
| Same, annual | 3.3 K | same | ≥ 2.0 K | 1.92 K (0.64) |
| Southern winter ice edge, std over longitudes | "several degrees" (roughly 55–68°S) | design §1.2, not measured | ≥ 2° | 2.5° (0°) |
| Brest − Vladivostok, January | 20.1 K | design §7.1 | ≥ 12 K | 20.6 K (7.1) |
| Yakutsk, January | −39 °C | design §7.1 | ≤ −28 °C | −29.0 °C (−15.8) |
| Reference cities in the right main group (`climate-dem` list of 24) | – | – | ≥ 21 | 19 (18) |
| Köppen land shares A / B / C / D / E | – | not recorded | none | 0.14 / 0.29 / 0.14 / 0.22 / 0.21 |
| Mumbai annual rain | 3,096 mm | design §1.2 (Beck et al. 2023) | ≥ 1,500 mm (budget) | 623 mm |
| Sahara; Atacama annual rain | 11; 16 mm | design §1.2 | ≤ 80 mm (budget) | about 230; about 180 mm |
| Ocean heat transport, northern peak | 1.5–2 PW at 15–25°N | Trenberth & Caron 2001 (CLM-133) | 1.5–2 PW | **3.1 PW at 27°N** |
| Atmosphere heat transport, northern peak | 5.0 ± 0.14 PW at 43°N | Trenberth & Caron 2001 abstract | none | 3.6 PW at 39°N |
| North Atlantic 55°N 20°W, January / July (sea) | about +9 in January | a round value, no dataset recorded | none | 9.5 / 19.4 °C |

January and July air temperature of the nearest land cell, the lapse rate applied, °C (the July
Earth figures and the rounding of the January ones are approximate and were not read from a
dataset for this page):

| Place | Jan, mod (banded) | Jan, Earth | Jul, mod (banded) | Jul, Earth |
|---|---|---|---|---|
| Brest | 11.3 (4.0) | about 7 | 20.8 (15.0) | about 17 |
| Vladivostok | −9.3 (−3.1) | about −12 | 24.4 (19.5) | about 20 |
| Moscow | −8.2 (−17.7) | about −9 | 16.9 (13.4) | about 18 |
| Yakutsk | −29.0 (−15.8) | about −39 | 15.4 (12.2) | about 19 |

The reference cities that miss their main group: Mumbai and Buenos Aires (rain, still on the belts),
Denver and Lhasa (height), Anchorage. Buenos Aires and Mumbai belong to the rain targets, which wait
for `rain_model = budget`; the others are the temperature model's.

What the table says. The temperature targets are met except two: the along-row spread (4.53 K
winter and 1.92 K annual against 5.0 and 2.0) and the reference cities (19 against 21). Brest −
Vladivostok (20.6 K) matches Earth; Yakutsk (−29.0 °C) passes its target only just and is 10 K
warmer than Earth (on the 0.5° equirectangular grid before Lambert's projection landed it was
−37.7 °C: the equal-area grid's tall high-latitude rows smooth the interior). Brest is 4 K too warm
in January. The ocean's heat transport is about twice Earth's, on purpose (below). The rain rows
are not targets of the default model and are reported only.

## Simplifications and limits

- **The conveyor is a compensating choice, not Earth's ocean.** Earth's Atlantic overturning is
  17 Sv. At 15 Sv the North Atlantic at 55°N was −3.7 °C in January, against about +9; at 30 Sv it is
  +7.4 °C. The price is an ocean heat transport of 3.2 PW at 27°N against 1.5–2 PW, and an
  atmospheric one of 3.5 PW at 30°N against Earth's 5.0 PW at 43°N. The conveyor stands in for the
  atmosphere's stationary eddies (planetary waves), which a model with one temperature per tile
  and a diffusive atmosphere does not have. The atmospheric diffusivity is also above its source's
  (CLM-176, 1.3) for the same reason.
- **The wind carries heat as a shift of the whole row** at a share of the speed (CLM-125); it is a
  parameterization, not a circulation. The wind profile is the same in both hemispheres (Earth's
  southern westerlies are about 2.5 times its northern ones at 850 hPa).
- **No stationary eddies, no clouds beyond the longwave law, no salinity, no sea-ice drift, no
  coastal upwelling, no mountain blocking of the zonal heat transport (CLM-126 is not built).**
  The 20 W/m² of cloud greenhouse that A + B T already holds is implicit (CLM-172).
- **The sinking basin is chosen by width**, as a stand-in for salinity, which the model does not
  carry; Ferreira et al. 2018 name salinity as the proximate cause. One sinking region per planet.
- **The zonal clock and its correction.** The fast clock (every 200 kyr) is one-dimensional. It is
  held on the 2-D model's zonal means by a fitted diffusivity per face and a flux per band and
  tile (CLM-136). It meets 0.3 K in every band except at ice edges, where a band's snowy tile has
  two states (worst miss 0.7–1.2 K on the test planets, RMS 0.13–0.18 K; the global means agree to
  0.02 K). Between snapshots the 2-D structure (the land-sea and east-west pattern) is not
  re-solved except at triggered snapshots (3 K or 0.15 of sea ice).
- **Rain is on the belts by default.** The belt rain is a table by latitude with moisture
  carried along the wind on L1 (CLM-178). The Sahara is about 230 mm and Mumbai 613 mm. The
  moisture budget (CLM-139 to CLM-146) is built and tested on idealized planets (an aquaplanet,
  a monsoon continent) and has not been compared with Earth's rain.
- **The grid is coarse**: 2.8 degrees on L1 1024 × 512, 2.5 degrees on Earth's 720 × 360 relief.
  Everything below that scale comes from the downscaling to L1, not from the model.
- **The rain law is a fit over the tropical oceans** (Bretherton et al. 2004), used everywhere.
- **The gyres see only the annual mean zonal wind stress**, and psi is zero poleward of 75° and
  on rows with no land (the "Drake rule").
- **The desert albedo uses the previous snapshot's Köppen B land**, so it lags the climate by one
  snapshot (10 Myr).
- **The CO₂ forcing** is a cloud-free table scaled to Myhre's all-sky doubling (CLM-168), for
  1 bar of N₂ plus CO₂, saturated, with no CO₂ condensation and no Rayleigh scattering; its error is
  not stated by its source and is roughly ±10 % in forcing at 0.1–1 bar from the radiation and more
  from the assumptions (research note). T is clamped to 190–370 K for F.
- The January and July Earth values for the four cities are approximate.

## Game choices

- CLM-175: the per-cell model is the default, the banded model stays a setting (Ben, decisions log
  2026-10-01).
- CLM-178: rain stays on the belts until the budget beats them on Earth's relief (Ben, 2026-10-01,
  the design's Q2 answer (a)).
- CLM-172: no extra snowball cloud forcing by default; a setting 0–20 W/m² (Ben, 2026-10-02,
  "None by default, a setting").
- CLM-138: the global-mean target of 14.1 ± 0.4 °C (Ben, 2026-10-02).
- CLM-129: the gyres stop at 75° and in zonal channels (numerics; the design agent, build log
  2026-09-30, P1-26 design).
- CLM-135: how crisp a sea-ice edge is drawn on L1 (look; same).
- CLM-136, CLM-179, CLM-137: the flux-correction fit, its cap of 60 W/m² and stops, and the
  thresholds for an extra snapshot (numerics; the design and build agents, build log 2026-09-30 and
  2026-10-03).
- CLM-170: the CO₂ forcing below the table (numerics; the research agent, 2026-10-03).
- The grid resolution (CLM-119) is Ben's choice (decisions log 2026-10-01, Q4).

## Open questions

- **Rain.** The budget (`rain_model = budget`) must beat the belts on Earth's relief (ln P RMSE
  ≤ 0.85, Mumbai and Bangladesh ≥ 1,500 and ≥ 1,200 mm, Sahara and Atacama ≤ 80 mm) before it can
  be the default. Its constants (K_q, κ_p, τ_P, a, the stability factor) are uncalibrated T9
  knobs. T9 has done the temperature knobs only.
- **The ocean heat transport misses its target by 1.2–1.7 PW** (CLM-133). Whether to find the
  missing atmospheric transport (stationary eddies) and bring the conveyor back toward Earth's
  17 Sv is open.
- **Earth's January sea surface in the North Atlantic** (+9 °C) is a round value with no dataset
  behind it on this page (CLM-131).
- **Snowball escape.** The forcing law is replaced (CLM-167), and a tropical-supercontinent test
  planet escapes at 71,054 ppm, inside Hoffman et al.'s 10⁴–10⁵ ppm. What the real 500-Myr runs
  escape at has not been recorded on this page; the research note expects 1–3·10⁵ ppm for a
  Cryogenian Sun, above the proxy range, and names a dusty-ice albedo as the other route.
- Not readable in WP-14a, so these stay unverified: Stommel 1948 (the gyre equation and boundary
  width), Lindzen & Nigam 1987 (κ_p's lineage and the friction r), Serreze, Kahl & Schnell 1992
  (the inversion), North, Mengel & Short 1983, Zhuang et al. 2017's paper text (its code was read),
  Murray 1967 (Tetens), Large & Pond 1981.
- C_E = 1.3·10⁻³ (CLM-142) has no source; GREB does not number its C_w.
- K_q (CLM-139) is 2 to 7 times GREB's κ, whose own paper gives two values.
- The 50-m slab is shallower than the sources' 70–96 m (CLM-013); T9 may revisit it against the
  seasonal sea-surface amplitude.
- The deep-water temperature of 2 °C (CLM-163) has no source.
- ρ_sea and ρ_air (CLM-177) were written from memory.
- T9 has not re-measured the real 500-Myr runs (heights, glaciated share, deposit counts, CO₂ at the
  present) under the calibrated defaults; only `KERNEL_EARTH` was re-measured (CLM-061).

Resolved since the design: Earth's mean temperature target (14.1 ± 0.4 °C, Ben 2026-10-02), the
CO₂ forcing above 50,000 ppm and the snowball escape (HEXTOR table, CLM-167), and the Köppen class
on L2 (Ben 2026-10-02).

## Sources

- Ziegler2021: read in full (library). Zhuang2017: the paper not obtained; its released code read
  for the constants (GPL-3, nothing copied).
- North1981: read in full (scan with text layer). Ramirez2024, Stassen2019, Dommenget2011,
  Bretherton2004, Manabe1969, Ferreira2018, Trenberth2001, WagnerEisenman2015 (bibliography key
  Wagner2015), DonohoeBattisti2011 (accepted manuscript), McCarthy2015 (accepted manuscript),
  FrajkaWilliams2019, BackBretherton2009, LargeYeager2004, IPCC2001_ch6, Byrne2014, Hoffman2017,
  HeldSoden2006: read in full or in the passages named in the rows.
- HaqqMisraHayworth2022 and the HEXTOR software (the OLR table, commit f800e25f, MIT): the paper's
  radiative-transfer section and Appendix A and the table file read (CLM-167); Kopparapu2013: its
  model description and caveats on dense CO₂ read; Abbot2012, Hu2011, Pierrehumbert2011: read for
  the cloud and escape figures of the research note (CLM-172, CLM-166). Myhre1998 itself was not
  read (its 5.35 was read in IPCC 2001 Table 6.2).
- Rohde2020: the Berkeley Earth data file read and averaged. Kalnay1996: the NCEP/NCAR reanalysis
  long-term means read and averaged (the paper not fetched). Beck2023: the observed temperatures of
  the design's section 7.1 table (read by the design's scratch script; not read in this page).
- Stull2017, Norton1979, Guilloteau2000: textbook, report and lecture notes, read for single values.
- climlab (Rose2018): the MIT source read for its defaults.
- Not obtained: Stommel1948, Lindzen1987, Serreze1992, North1983, Murray1967 (Tetens), Large &
  Pond 1981, Abbot 2014, Le Hir et al. 2007 and 2010, Kadoya & Tajika 2019.
