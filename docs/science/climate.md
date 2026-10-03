# Climate

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md). The per-cell climate, the default since P1-26 (2026-10-03), has its own page: [per-cell-climate.md](per-cell-climate.md).

Climate decides temperature, rain, ice and sea level through the planet's history, and with them erosion, glaciers, coal and the biomes of the finished world. Today's code solves an energy balance (sunlight in, heat out, heat carried poleward) in latitude bands, adds wind and rain belts, rain shadows behind mountains, a carbon cycle with a slowly brightening sun, orbital cycles that pace ice ages, land ice and sea ice. Because it works in bands, climate follows latitude far too closely (the design measured latitude explaining 99.6 % of land temperature variance, against 96.8 % on Earth). **This banded model is no longer the default.** Since P1-26 (built 2026-10-02 to 2026-10-03; design `docs/design/p1-26-per-cell-climate.md`, section 11 "As built") a snapshot runs a per-cell, seasonal, two-dimensional energy balance with a slab ocean, wind-driven gyres, an overturning conveyor and, optionally, a moisture budget, and `climate.model` defaults to `per_cell`. **See [per-cell-climate.md](per-cell-climate.md)** for what it does, its constants and how it compares with Earth. The banded model is what this page's older rows describe: it stays as `climate.model = banded`, it is the zonal model that gives the per-cell model its first state, and the fine clock (every 200 kyr) is still one-dimensional, corrected at each snapshot to the 2-D zonal means. Its rows are marked `pending-rewrite`; the per-cell rows (CLM-119 onward) describe code and carry no such mark, except CLM-126 (mountain blocking), which is not built.

The constants of the per-cell design were checked against their sources before the build (WP-14a, 2026-10-02) and the built model's constants are in [per-cell-climate.md](per-cell-climate.md).

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [CLM](ledger/CLM.md) | energy balance | CLM-001 to CLM-026, CLM-119 to CLM-134, CLM-138, CLM-163 to CLM-165, CLM-172, CLM-175 to CLM-178 (51) |
| [CLM](ledger/CLM.md) | insolation and orbit | CLM-027 to CLM-044 (18) |
| [CLM](ledger/CLM.md) | carbon cycle (CLM-069 to CLM-073 are the CO₂ forcing's old law and its anchors; the law is CLM-167) | CLM-045 to CLM-080, CLM-167, CLM-168, CLM-170 (39) |
| [CLM](ledger/CLM.md) | deep-time clock | CLM-081 to CLM-091, CLM-136, CLM-137, CLM-166, CLM-179 (15) |
| [CLM](ledger/CLM.md) | land ice and sea ice | CLM-092 to CLM-112, CLM-135 (22) |
| [CLM](ledger/CLM.md) | mountain glaciers | CLM-113 to CLM-118 (6) |
| [CLM](ledger/CLM.md) | moisture budget (built behind `rain_model = budget`, not the default) | CLM-139 to CLM-146 (8) |
| [CLM](ledger/CLM.md) | ice branch: the flowing-sheet switch, present-day climate, supply-limited and seafloor weathering (merged 2026-10-02) | CLM-147 to CLM-162 (16) |
| [GLA](ledger/GLA.md) | the flowing ice sheet (`climate/sheet.rs`, off by default) and the ice mass balance for the hydrology; on the glacial page | GLA-067 to GLA-076 (10) |
| [MOI](ledger/MOI.md) | wind and rain belts | MOI-001 to MOI-033 (33) |
| [MOI](ledger/MOI.md) | orographic rain | MOI-034 to MOI-050 (17) |
| [MOI](ledger/MOI.md) | lapse rate | MOI-051 to MOI-053 (3) |
| [MOI](ledger/MOI.md) | seasons | MOI-054 to MOI-060 (7) |
| [MOI](ledger/MOI.md) | climate snapshot | MOI-061 to MOI-076 (16) |

Rows by status (CLM and MOI, 2026-10-03, counted by hand from the two ledger files; `merged` rows not counted): `verified` 23, `cited-unverified` 50, `from-memory` 3, `calibrated` 16, `game-choice` 41, `uncited` 98; `pending-rewrite` 182 (the status mark only); `not-yet-coded` 1 (CLM-126).

## The ice branch, merged 2026-10-02

What the merge added to the banded model (build log 2026-10-01 and 2026-10-02, "Climate ice branch"):

- **Flowing ice sheets** (`climate/sheet.rs`, GLA-067 to GLA-076): shallow-ice flow with Glen's law on a coarse grid, mass balance on the ice surface against the equilibrium line (the elevation feedback), the bed sinking under the ice, calving where the pressed-down bed lies below the sea. **Off by default and in every built-in preset** (`climate.ice_dynamics`, CLM-147; Ben, decisions log 2026-10-02): driven by the banded climate they overgrow. Off, the land ice is the model from before the branch (Nye's equilibrium profile with Imbrie's lag in the ice-age window), and the flexure's ice load (`climate/ice_thickness_m`, ISO rows) is that ice.
- **Supply-limited weathering** (CLM-152 to CLM-157) and **seafloor weathering** (CLM-158 to CLM-162): on by default.
- **The present-day climate** (`climate.present_climate`, CLM-148 to CLM-151): temperate by default.
- **The fine clock's step** is 200 kyr (CLM-082).
- **For the hydrology:** `climate/ice_balance_m_yr`, the land ice's surface mass balance (GLA-076), beside the ice thickness and extent.

Measured on `earthlike_quick_16k` (seeds 1 and 2; scattered start 500 Myr, supercontinent 1000 Myr; final state; build log 2026-10-02): runs on one build are reproducible to every printed digit. With the branch's carbon-cycle settings set back to master's, the merged build with the sheets off printed master's figures exactly. With the defaults (sheets off) the new weathering keeps more CO2 (scattered 113 and 271 ppm against master's 88 and 119; supercontinent 471 and 475 against 237 and 374), so the land ice falls from 0.14 to 0.10 (scattered) and from 0.11 to 0.07 (supercontinent) of the land. Earth today is about 0.10. With the sheets on, the land ice is 0.15 in all four runs, but only because the temperate present raises CO2 to 650 to 1,726 ppm (15 to 18 degC). The sea ice is then 0.03 to 0.07 of the ocean, against 0.07 to 0.14 with them off. Without the present-day nudge, three 500-Myr planets ended with 47 to 52 % of the land under ice (build log 2026-10-01).

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/CLM.md](extraction-notes/CLM.md), [extraction-notes/MOI.md](extraction-notes/MOI.md).
- Design: `docs/design/p1-26-per-cell-climate.md` (its section 2 measures the faults of the banded model; its section 9 lists references, six of them marked "cited from memory").
- Session notes: `docs/research/notes/session-polar-climate-bands.md`.
- Work package in the verification plan: WP-14 (deferred until the rewrite lands) ([claims.md](claims.md#verification-plan-for-stage-2)).

## The CO₂ forcing

One law at every concentration and temperature, since P1-26 T6 (2026-10-03; `climate/co2.rs`, ledger CLM-167, CLM-168, CLM-170, CLM-172; the sources are in the research note `docs/research/notes/high-co2-forcing-and-snowball-escape.md`):

```text
F(C, T) = m · [OLR_RC(T, 280 ppm) − OLR_RC(T, C)]        m = 0.903
OLR(T)  = A + B T − F(C, T) − c_snow · frozen share       A = 203.3 W/m², B = 2.09 W/m²/K
```

- OLR_RC is the clear-sky outgoing longwave of a radiative-convective model (Kopparapu et al. 2013: 1 bar of N₂ plus CO₂, moist adiabat, saturated, CO₂ collision-induced absorption), as HEXTOR (Haqq-Misra & Hayworth 2022, MIT licence, commit f800e25f) tabulates it: 19 temperatures (190 to 370 K) by 91 CO₂ partial pressures (10⁻⁴ to 10 bar), interpolated linearly in T and in log₁₀ pCO₂ with pCO₂ = C · 10⁻⁶ bar. It is each tile's own surface temperature that is looked up, so the greenhouse of a snowball's cold tropics and of a hothouse differ.
- m scales the clear-sky table so that doubling CO₂ at 288.15 K gives Myhre et al. 1998's all-sky 5.35 ln 2 = 3.71 W/m². F is zero at 280 ppm at every temperature, so Earth's calibration is unchanged. Below the table (about 100 ppm) the forcing continues as a logarithm with the table's own slope between about 100 and 200 ppm.
- It replaced the Myhre law below 1,000 ppm blended into Byrne & Goldblatt 2014's quadratic fit above 2,000 ppm (CLM-069 to CLM-071), which was stated only to 10,000 ppm (computed to 50,000) and was used to 1,000,000 ppm without temperature dependence. Byrne & Goldblatt's values are the cross-check: at 288 K the table (× m) gives 22.7 W/m² at 10⁴ ppm against their 24.1, and 40.4 against 38.1 at 5·10⁴ ppm.
- A + B T already carries about 20 W/m² of longwave cloud greenhouse at snowball temperatures. Extra cloud forcing on frozen surfaces is a setting (`climate.snowball_cloud_w_m2`, default 0, range 0 to 20 W/m²; Ben, decisions log 2026-10-02).
- Not modelled: CO₂ condensation, Rayleigh scattering of a dense CO₂ atmosphere, clouds' state dependence. The source states no error for the table; Kopparapu et al. say the method tends to overestimate the greenhouse effect of dense CO₂ atmospheres. The research note puts it at about ±10 % in forcing at 0.1 to 1 bar from the radiation alone.
- Expected effect: the escape CO₂ of a snowball falls (the note's 1-D stand-in: about 1.9·10⁵ ppm at today's Sun against 3.3·10⁵ with the old law; the real clock's escapes are listed in each run's notes beside Hoffman et al. 2017's 10⁴ to 10⁵ ppm, CLM-166).

## Known before the page is written

- Land ice thickness used a hard-coded gravity of 9.81 m/s2; since P1-26 T6 it takes the planet's gravity (CLM-100).
- Earth-like runs end near 210 to 220 ppm CO2, not the 280 ppm the carbon cycle is balanced for (flag 7).
- Ice sheets overgrow on the banded climate (47 to 52 % of the land under ice on three 500-Myr Earth-like planets, build log 2026-10-01 "Climate ice branch"), so the flowing sheets (`climate.ice_dynamics`, CLM-147, GLA-067 to GLA-076) are off by default and in every built-in preset (decisions log 2026-10-02, "Merge with ice sheets off"); they are turned on once the per-cell climate (P1-26) fixes the cold bias. See "The ice branch, merged 2026-10-02" below.
- Stage 1 spot checks (before P1-26): the CO2 forcing fit (Byrne & Goldblatt 2014) was stated valid for 200 to 10,000 ppm and used up to 1,000,000 ppm (replaced, see "The CO₂ forcing"); a snowball state ended at 0.55 bar of CO2 where Hoffman and others 2017 give 0.01 to 0.1 bar (the new law lowers it, not yet to that range); the banded model's Earth mean temperature of 15.6 degC is above any long-term mean (Berkeley Earth: 14.1 degC for 1951 to 1980; the per-cell model is calibrated to 14.1 ± 0.4 degC, CLM-138).
- The energy-balance constants are a mix: A, B and D attributed to North, Cahalan & Coakley 1981 (not readable: a scan), albedos from two different climlab classes that climlab does not combine.
