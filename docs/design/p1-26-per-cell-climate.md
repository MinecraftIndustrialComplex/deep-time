# P1-26 Per-cell climate — design

Status: **approved by Ben 2026-10-01** (decisions log, "Approve as recommended"). Nothing here is
built yet. Constants checked against their sources before the build (science stage 2, WP-14a,
2026-10-02): each change is marked "WP-14a" with its source, and the ledger rows are CLM-119 to
CLM-146 and CLM-163 onward (`docs/science/ledger/CLM.md`; page `docs/science/per-cell-climate.md`).
The build starts after
the climate agent's ice-sheet branch (`worktree-agent-adc924860a859865c-ice`) lands, because both
change `crates/deeptime-core/src/climate/`. Ben asked for it on 2026-09-30 (decisions log: "I feel
like climate in general should probably be per cell as opposed to completely banded across a
single lattitude"; answer "Design doc first"). It replaces the per-cell sea-ice item queued the same
day.

Every number below was measured for this document, unless it says "estimate" or names a source. The
measurements were run on the M6 Mac under scratch folder `/Volumes/Graphide/ban/deeptime-runs/p1-26-design/`,
which is deleted after this task. Appendix A lists the commands, and Appendix B the prototype.

---

## Summary (two minutes)

**The problem, measured.** Today the climate's backbone is one temperature per 2° latitude band
(`ebm.rs`) and one reference rainfall per belt latitude (`belts.rs`). Per-cell effects only nudge
them. What that does:

- **Temperature.** On a real 500-Myr `earthlike_quick_16k` run, latitude explains 99.6 % of the
  variance of land temperature at sea level. On Earth it explains 96.8 %. Along a latitude circle,
  winter land temperature varies by 3.1 K, against Earth's 6.1 K.
- **Sea ice** is one value per band. Its southern edge varies by 0.23° all round the planet.
- **The polar continent** is uniformly ET or EF.
- **On Earth's relief** the model gets:
  - Yakutsk's January at −16 °C (observed −39 °C);
  - Mumbai at 531 mm a year (observed 3,096 mm);
  - the Sahara at 265 mm (observed 11 mm);
  - Köppen agreement of 35 % exact and 65 % by main group.

**The design.**
1. **A seasonal 2-D energy balance on a 128 × 64 grid (2.8°).**
   - Each cell has a land tile, a sea tile, a slab ocean with its own sea ice, and its own albedo.
   - Heat moves by surface-dependent diffusion and by the prevailing wind.
   - In the ocean, heat moves by Stommel gyres built from the real coastlines, plus an overturning
     conveyor that sinks in the narrowest polar-connected basin.
2. **Rain from a monthly moisture budget** on the same grid: monsoon winds, evaporation, a soil
   bucket, and Smith & Barstad mountain rain on L1. It is built behind a switch: today's belt rain,
   fed by the new temperatures, stays the default until the budget beats it on Earth's relief.
3. **Downscaling to L1** with a coast blend and a winter-inversion term. **The baked layers and the
   climate API (mic-climate) do not change.**
4. **The fast carbon/ice clock stays 1-D.** It is flux-corrected to the 2-D zonal means at every
   snapshot. Snowball and hothouse crossings trigger an extra 2-D snapshot.

**Prototype evidence** (scratch code, Earth's relief at 2.5°, Appendix B):
- **Winter land temperature beats today's model.** The error against observations is 4.9 K
  (today 7.2 K). Along latitude circles it varies by 5.4 K (today 3.3 K, Earth 6.0 K).
- **It conserves and is cheap.** Energy and water close, at about 0.13 s of one core per model year.
- **A plain diffusive 2-D EBM was not enough:** its winter variation stayed at 3.2 K. The conveyor,
  the wind-carried heat and the inversion made the difference.
- **Rain is the risk.** The budget gets the deserts (Sahara 32 mm), but its overall error is still
  worse than today's belts.

**What you will see.**
- Ragged, seasonal sea-ice edges, with frozen-ocean biomes that follow currents and coasts.
- Cold continental interiors and mild windward coasts.
- Deserts inland and on cold west coasts.
- A polar continent with an icy interior and milder coasts.
- A satellite view that no longer needs its band-hiding blur.

**Cost.** About 0.3–1.2 s per snapshot, which adds about 15–60 s to a 500-Myr quick run, and about
10 MB of memory.

**Plan.** There are eleven tasks: six Opus, five Sonnet. Three run in parallel once the shared coarse
grid exists, and the build starts after the ice branch lands. Section 10 lists eleven open
questions, each with a recommendation.

---

## 1. The current model, measured

### 1.1 What was run

- **Real run.** `deeptime stage climate --preset earthlike_quick_16k --start scattered_continents
  --length-myr 500` on master 424431b:
  - L1 is 1024 × 512, with 250 plate steps and 51 snapshots;
  - it ended at CO₂ 344 ppm, with 19.7 % of the land glaciated and Köppen land shares A 0.10 /
    B 0.20 / C 0.15 / D 0.15 / E 0.41;
  - the fields were dumped with `deeptime dump-fields`.
- **Earth relief.** One snapshot of the same model at 720 × 360, made by a scratch copy of
  `deeptime climate-dem` that also dumps the twelve months. The relief is GMT's `earth_relief_30m_p`
  (SRTM15+), and the run repeats the snapshot six times with the ice-sheet feedback, as
  `climate-dem` does. The EBM came out at 15.26 °C, with net TOA −7·10⁻⁵ W/m² and 10.0 % of the
  land glaciated.
- **Observed.**
  - Köppen–Geiger: Beck et al. 2018, present, 0.5°.
  - Monthly air temperature and precipitation, 1961–1990: Beck et al. 2023, 0.5°.

  Both are CC BY 4.0 and used for validation only. Observed temperatures were brought to sea level
  with the model's 6.5 °C/km on the same relief.

**Metrics.** Everything is area-weighted.
- **η²**: the share of a field's variance over land explained by 2° latitude bands. It is the
  correlation ratio, 1 for a purely banded field.
- **Along-row std**: the mean, over rows at 30–70°, of the standard deviation along each row's land
  cells. It shows how much a field varies along a latitude circle.
- **Ice edge**: for each longitude, the most equatorward latitude whose sea ice ≥ 0.5.
- **Köppen**:
  - η² is the share-weighted mean of η² over the class indicators;
  - "band heterogeneity" is the share of each band's land outside the band's modal class;
  - "E–W boundaries" is the share of class changes between north–south neighbours.
- **Winter** is January north of the equator and July south of it.

### 1.2 Results

| Metric | Current model, real run | Current model, Earth relief | Earth observed |
|---|---|---|---|
| Land T at sea level, along-row std: annual / winter / summer | 0.87 / 3.09 / 1.85 K | 0.90 / 3.37 / 1.90 K | **3.29 / 6.06 / 3.16 K** |
| Same, η² (variance explained by latitude): annual / winter / summer | 0.996 / 0.978 / 0.961 | 0.996 / 0.974 / 0.967 | **0.968 / 0.942 / 0.936** |
| ln precipitation over land: η² / along-row std (0–70°) | 0.451 / 0.72 | 0.753 / 0.37 | **0.413 / 0.84** |
| Sea ice: η² over the sea | 0.998 | 1 (one value per band by construction) | — |
| Southern ice edge: spread over longitudes (std; min…max) | 0.23° (70.5…71.9°S) | 0.00° (69.2°S everywhere) | Antarctic winter edge spans roughly 55–68°S |
| Köppen: η² / band heterogeneity / E–W boundaries | 0.27 / 0.54 / 0.53 | 0.40 / 0.43 / 0.57 | 0.34 / 0.50 / 0.56 |
| Köppen vs Beck 2018 (exact class / main group, by land area) | — | 34.6 % / 64.9 % | — |
| Monthly T at sea level over land: RMSE / bias vs observed | — | 6.6 K / −0.9 K | — |
| ln annual precipitation over land: RMSE vs observed | — | 1.11 (a factor of 3) | — |

**Points** (January sea-level T, °C; annual precipitation, mm). Model on Earth's relief vs observed:

| | Brest | Bergen | Moscow | Yakutsk | Vladivostok | Seattle | Boston | Winnipeg |
|---|---|---|---|---|---|---|---|---|
| Model | 4.9 | −5.4 | −16.8 | −16.5 | −1.9 | 3.1 | 0.4 | −9.0 |
| Observed | 7.2 | 0.6 | −8.8 | **−39.0** | **−12.9** | 4.5 | −2.8 | −16.5 |

| | Sahara | Atacama | Namib | Mumbai | Bangladesh | SE China | Kazakh steppe | Gobi | Amazon | London |
|---|---|---|---|---|---|---|---|---|---|---|
| Model | 265 | 154 | 233 | 531 | 278 | 408 | 758 | 431 | 1,311 | 1,080 |
| Observed | 11 | 16 | 61 | **3,096** | **2,194** | **1,765** | 142 | 112 | 2,234 | 702 |

**What the maps show** (looked at as quick-look PNGs):
- **Temperature.** On Earth's relief the January isotherms run straight east–west across Eurasia
  and North America. The observed map has its cold pole over eastern Siberia, a warm western
  Europe, and isotherms that bend north–south along the coasts.
- **Real run.**
  - The north polar continent is a uniform grey of ET and EF.
  - The subtropical continent carries a ruler-straight BWh stripe with straight BSh bands either
    side. These come from the belt rain table times the interior moisture floor.
  - The sea-ice edge is a staircase of 2° bands at about 70°N and 70°S.
- **Köppen aggregates.** By the aggregate Köppen metrics the classes are not much more banded than
  Earth's: η² 0.27–0.40 against 0.34. Precipitation, elevation and the coast distance already vary
  per cell. What looks striped is the **temperature-set boundaries**: E/D (warmest month 10 °C),
  ET/EF, D/C (coldest month 0 °C) and A/C. These follow the straight isotherms, so the metric to
  fix is temperature and sea ice, with rain second.

### 1.3 The causes in the code

Line numbers are at master 424431b; the ice branch moves some of them.

1. **One temperature per band.** `EbmState` holds `t_ocean`, `t_land`, `t_mixed` and `ice_j_m2` per
   band (`ebm.rs:73–81`). Diffusion acts on the band mean (`ebm.rs:310–330`).
   `snapshot::compute` interpolates the bands to rows (`snapshot.rs:246–255`). A land cell's month
   is then `t_o[j][m] + cont·(t_l[j][m] − t_o[j][m]) + mar·dcar` (`snapshot.rs:461`): **the same two
   numbers for the whole row**, blended by a continentality between 0 and 1. A continent that fills
   a band has no interior–coast contrast beyond that blend, and every continent at a latitude gets
   the same "land" temperature.
2. **Rain from a latitude table.** `base = belt_precip(φ′) × subsidence × q × …`
   (`snapshot.rs:470`, table at `belts.rs:108`). φ′ shifts per column only by the ITCZ, which is
   smoothed over 1,500 km. Far inland the moisture march falls to its floor (`moisture_floor` 0.3),
   so interiors take the table's latitude profile: straight BW/BS stripes.
3. **Row marches.** Moisture, maritime influence and the gyre offset are marched along each row
   (`lapse.rs`) and blurred over 300 km (`snapshot.rs:401`). Air never moves north–south, so a
   monsoon can only come from the ITCZ shift.
4. **Sea ice per band.** `sea_ice[c] = er.sea_ice[band]` (`climate/mod.rs:1101`).
5. **Between snapshots the reference is per row.** `tm = refs.t_mean[j] + d_mean[c] − Γz`
   (`climate/mod.rs:961`). The fine clock's equilibrium-line shift is per band (`clock.rs:350`), and
   so is the ice share it hands the ice (`climate/mod.rs:1051, 1076`).
6. **The satellite view hides the bands.** It blurs temperatures over ±3 rows and sea ice over
   5 × 17 cells (`satellite.rs:717, 729`).
7. **Gyres are a prescription.** `sst_anomaly` (`belts.rs:304`) puts fixed ±3–6 °C offsets on coasts
   by basin position. They are not energy-conserving, they cannot move sea ice, and they are the
   only reason western Europe is mild.

---

## 2. Temperature on the grid

### 2.1 The grid

**The climate grid is a regular latitude–longitude grid of `climate.grid_columns` × half that cells,
128 × 64 by default (2.8125°, 313 km at the equator).** It is built from L1 by block means: L1 is
1024 or 2048 wide on every built-in preset, so a coarse cell is exactly 8 × 8 or 16 × 16 L1 cells.
On other circumferences the block is `L1 width / columns` rounded, as the ice sheet's grid does.

Why this grid:
- **Planetary scale.** Like the rest of the planetary-scale climate, the grid is Earth-normalized:
  the same 2.8° on every preset, so a 16k and a 32k planet get the same climate.
- **Precedent.** TransEBM (Ziegler & Rehfeld 2021), the current descendant of North, Mengel & Short
  1983, uses this same resolution (T42, 128 × 64; a Gaussian grid, where ours is regular: Ziegler &
  Rehfeld 2021, p. 2845). GREB uses 3.75° × 3.75° (Stassen et al. 2019, p. 426).
- **Reuse.**
  - L1 is equirectangular, so coarsening and refining are block means and bilinear interpolation.
  - The ice branch's `climate/sheet.rs` already has that machinery (`sheet::Grid::{new, coarsen,
    refine}`, about 256 columns). Moved to a shared `climate/coarse.rs`, both use it.
- **Fast numerics.** Rows and columns give tridiagonal (ADI) implicit diffusion and an exact zonal
  advection shift (§2.8).
- **Poles.** The polar rows lie beyond the soft cap (±75–80°). Implicit diffusion is stable there,
  and the gyres are not solved poleward of 75° (§2.6).

**Alternatives considered:**
- **2° (180 × 90):** about 2× the cost for coasts that L1 refines anyway.
- **4° (90 × 45):** half the cost, but too coarse for narrow seas and land bridges, e.g. a Red Sea or
  a Panama.
- **The L0 Fibonacci samples:** 100,000–250,000 points, far too many. A separate coarse Fibonacci
  mesh would be uniform and pole-free, but needs its own remapping and does not suit row
  advection or the sheet grid.

The 4° option stays a setting for slow machines (open question Q4).

### 2.2 State

Per coarse cell:

| Symbol | What | From |
|---|---|---|
| f | land fraction | L1 land mask, block mean |
| s_g | share of the land under ice sheets and glaciers | the ice fields (`climate/ice_thickness_m` ≥ 1 m), previous step |
| d | share of the land that is desert (Köppen B) | previous snapshot |
| z̄ | mean height of the land above the sea | L1 |
| T_l | land surface and the air above it, °C | prognostic |
| T_o | sea surface and the air above it, °C | prognostic |
| T_m | mixed-layer slab, °C | prognostic |
| E_i | sea ice, as the latent heat it holds, J/m² | prognostic |
| basin | which ocean basin the cell's sea belongs to | coastline, per snapshot |

These are exactly today's four band variables, now per cell. The physics of each tile, the
sea-ice thermodynamics and the constants carry over from `ebm.rs` unchanged, which keeps the 1-D
model's tested Earth calibration as the starting point.

### 2.3 Equations

Per cell, with Q the daily-mean insolation of the cell's latitude (`insolation.rs`: tilt, star,
orbit), A and B the CO₂-adjusted longwave, and C_l, C_m the heat capacities:

```
land:   C_l ∂T_l/∂t = (1 − α_l) Q − (A + B T_l) − x (1 − f)(T_l − T_o)                 + 𝒜_l
sea:    C_l ∂T_o/∂t = (1 − α_o) Q − (A + B T_o) − x f (T_o − T_l) − g(h)(T_o − T_m)    + 𝒜_o
slab:   C_m ∂T_m/∂t = g(h)(T_o − T_m) + 𝒪 − (freezing / melting at −1.8 °C, into E_i)
```

- x = `land_ocean_exchange_w_m2_k` (10 W/m²/K) is the land–sea air exchange inside a cell.
- g(h) = 30 W/m²/K (open water) blended with the ice conductance k/h by cover, as today.
- 𝒜 is the atmosphere's heat transport, applied, as today, to the cell mean
  T̄ = f T_l + (1 − f) T_o with heat capacity C_l, so both tiles receive the same change:

```
C_l ∂T̄/∂t |_atm = ∇·(D ∇T̄)  −  β_T C_l u_a(φ′, month) ∂T̄/∂x
```

- 𝒪 is the ocean's heat transport (§2.6).

**Heat capacities.** C_l = `land_depth_m` (3 m of water), C_m = `ocean_depth_m` (50 m), sea ice as
today (k = 2.2 W/m/K, full cover at 0.5 m, conduction tapered above 10 m). In the prototype a land
capacity of 1.87·10⁶ J/m²/K (0.45 m) fit worse: winter RMSE 8.6 K against 6.3 K with 3 m (run B
against run A). It stays a setting with today's 3 m as the default.

*WP-14a (sources read):* the 1.87·10⁶ J/m²/K is the land **surface** alone in Zhuang et al. 2017's
model, which adds an atmospheric column of 4.76·10⁶ to every surface type: its land column is
6.63·10⁶ J/m²/K (1.6 m of water), TransEBM's tuned one 2.66·10⁶ (0.64 m) (Ziegler & Rehfeld 2021,
Tables 1 and 2). Run B therefore tested a land column without its air, not the lineage's value.
North, Cahalan & Coakley 1981 give the "ideal land" earth-atmosphere capacity as C_L = 0.16 B years
with B = 2.09 W/m²/K, that is 1.06·10⁷ J/m²/K or 2.5 m of water, and the ocean mixed layer as
4.7 B years, 3.1·10⁸ J/m²/K or 74 m (p. 118, list of symbols). The air column alone is
c_p p_s / g ≈ 1.0·10⁷ J/m²/K (2.5 m). Today's 3 m for land plus air is consistent with North et al.;
the 50-m slab is shallower than their 74 m, Zhuang et al.'s 70 m and TransEBM's 96 m (Ziegler &
Rehfeld 2021, p. 2846). Both stay as they are (calibrated, CLM-013, CLM-014, CLM-122); T9 may
revisit the slab depth against the seasonal SST amplitude.

### 2.4 Albedo per cell

```
α_l = a₀ + a₂P₂(sin φ) + Δα_desert·d·(1 − snow)          (ice-free)
α_l → a_i  by max(snow(T_l), s_g)                        (snow: today's smooth step at T_f ± 3 °C)
α_o = a₀ + a₂P₂(sin φ)  → a_i  by the sea-ice cover min(1, h / 0.5 m)
```

- a₀, a₂ and a_i are today's 0.33, 0.25 and 0.62. As in North, Cahalan & Coakley 1981, the a₂ term
  carries the zenith-angle and cloud dependence.
- **Desert brightening** Δα_desert defaults to +0.05 in planetary terms. Sahara surfaces are about
  0.35 against 0.15 for vegetation, but the atmosphere halves the contrast at the top of the
  atmosphere. The desert share d is lagged by one snapshot (a Charney-type feedback), and a₀ is
  re-fitted so Earth's global albedo stays 0.30.
- **Ice sheets brighten their own cell**, not a whole band. A Laurentide-sized sheet now cools North
  America, not the whole of 50–70°N.
- **No explicit clouds in v1.** The stratocumulus decks over cold eastern seas (Klein & Hartmann
  1993, low cloud against lower-tropospheric stability) are a follow-up switch (open question Q9).

### 2.5 Atmospheric heat transport

**Diffusion depends on the surface**, following the NMS83 lineage (Zhuang, North & Stevens 2017, as
tabulated in Ziegler & Rehfeld 2021):
- D_face = (1 − f̄) D_ocean + f̄ D_land(φ);
- D_land(φ) = D_eq + (D_pole − D_eq)·|sin φ|⁵;
- defaults D_ocean 0.40, D_eq 0.65, D_pole 0.28 (north) / 0.20 (south), in W/m²/K as today;
- f̄ is the mean land fraction of the two cells sharing the face.

A zonal factor `climate.zonal_diffusivity_ratio` (default 0.5, calibrated) multiplies the
east–west faces. PlaHab (Ramirez 2024, PSJ 5:2) uses D_lon/D_lat = 0.02/0.5 = 1/25 for Earth
equatorward of 45° and 0.0002/0.5 = 1/2,500 poleward of it, on 30° longitude bands, with the same
1/cos²φ flux form (p. 3, eq. 1; read WP-14a). Our 0.5 is a calibrated value about 12 times
PlaHab's low-latitude ratio, not a sourced one. In the prototype, 0.1–0.5 added zonal variety but
needed the ocean terms to put it in the right places (Appendix B).

The diffusion is in flux form on the sphere:
- **Conductances**: east–west faces D·Δφ/(cos φ·Δλ); north–south faces D·cos φ_face·Δλ/Δφ.
- **Cell area**: Δλ (sin φ_N − sin φ_S).

A zonally uniform planet therefore reproduces the 1-D model exactly, which is a unit test (T2).

**The prevailing wind carries heat zonally.** u_a is a low-level (about 850 hPa) zonal-mean wind in
belt latitude φ′ = φ − shift(φ, ITCZ, month), using `belts::shift` as today:

```
u_a(φ′) = −6·b(|φ′|; 12°, 18°) + 9·b(|φ′|; 45°, 28°) − 2·b(|φ′|; 82°, 10°)  m/s
```

- b is `belts::bump`. The profile gives trades to about 25°, westerlies peaking at 45° and still
  blowing at 65°, and weak polar easterlies.
- *WP-14a check against Earth* (NCEP/NCAR reanalysis, 1991–2020 monthly long-term means, annual
  zonal mean at 850 hPa; computed WP-14a): westerlies peak at 4.9 m/s at 45°N and 12.7 m/s at 50°S
  (the profile: 9 m/s at 45°, between the two); trades −3.8 m/s at 15°N and −4.5 m/s at 15°S (the
  profile: −5.7 to −5.9 m/s at 10–15°, about 1.4 times stronger); 2.4 m/s at 60°N and 9.3 m/s at
  60°S (the profile: 4.6); at 80°N the mean is +1.0 m/s, so the profile's polar easterlies
  (−1.8 m/s) are an Antarctic-coast feature (−2.2 m/s at 70°S), not a northern one. The profile is
  a hemispheric average, not a fit; T9 may give it the two hemispheres' strengths.
- The surface regimes of `belts.rs` end the westerlies at 60°. Used for heat, they made the
  prototype advect *easterly* across Siberia in January.
- β_T (`climate.wind_heat_transport`, default 0.5, calibrated) is the share of the wind's speed at
  which the boundary layer's heat moves.
- u_a is constant along a row, so the advection is non-divergent and conserves exactly. It leaves
  the zonal mean untouched, so the 1-D clock never needs to know about it. It gives the coast
  asymmetry: mild west coasts under westerlies, continental east coasts.
- **Results.** With this profile, the prototype's Moscow went from −15 to −7 °C (observed −9) while
  Yakutsk cooled from −16 to −23 °C (−34 °C with the inversion; observed −39). With the surface
  regimes the two had come out alike (Appendix B, runs F and L).

**Mountain blocking** (option, `climate.mountain_blocking`, default off until calibrated). The row's
*anomaly* T̄ − ⟨T̄⟩_row is advected upwind instead, with each east–west face transmitting
`e^(−max(0, z_face − 1 km)/1 km)`. Here z_face is the larger of the two cells' land-weighted mean
heights. Mild Pacific air then pools west of a Rockies or an Andes.
- **Conservation.** Advecting the anomaly keeps a uniform row uniform, and the row's fluxes
  telescope, so it is still conservative.
- **Cost.** It needs explicit sub-steps (Courant ≤ 0.9, about 5–20 per step).
- **Prototype.** It raised winter variation from 5.4 to 6.6 K (Earth 6.0) but also the winter RMSE
  by 1 K. Calibration decides.

### 2.6 Ocean heat transport by basin

Three terms act on the slab T_m (𝒪 above), each conservative in flux form:

1. **Eddy diffusion** through the sea's share of each face, as today's 1-D `ocean_diffusivity_w_m2_k`,
   now in 2-D. A face between two cells counts only the smaller sea share, so there is no flux
   across land. Today's 0.3 W/m²/K; the prototype's better runs used 0.8 (calibrated).

2. **Stommel gyres** (Stommel 1948; implemented from the paper).
   - **Equation.** The wind-driven transport streamfunction ψ (m³/s) solves
     `r ∇²ψ + β ∂ψ/∂x = curl τ / ρ₀` on the sea, with ψ = 0 on coasts. τ is the annual-mean
     belt-wind stress ρ_a C_D |u| u with C_D 1.3·10⁻³, and β = 2Ω cos φ / a.
   - **Friction.** r = β·1.5·Δx gives a western boundary layer about 1.5 cells wide.
   - **Grid.** ψ lives on cell corners, so the face transports are exact differences of ψ and are
     non-divergent whatever ψ is.
   - **Where ψ = 0:**
     - poleward of 75°;
     - on latitude lines whose rows hold no land at all. This is the "Drake rule": an open channel
       has no Sverdrup closure, so it carries no gyre.
   - **Solver.** Gauss–Seidel sweeping east to west, the direction the Sverdrup balance integrates.
     The prototype took 0.01–0.03 s and about 300 sweeps.
   - **Heat.** The slab is advected by γ_o·ψ-transports. γ_o (`climate.gyre_heat_efficiency`,
     default 1.0, calibrated) relates the 50-m slab's temperature to the wind-driven layer's.
   - **What comes out.** On Earth's coastlines the prototype's subtropical gyres turn clockwise in
     the North Atlantic and the North Pacific, and |ψ| peaks at 19–22 Sv.
     - **Western intensification.** At 30°N the northward flow sits in the westernmost two or three
       cells: 6.7, 3.6 and 2.1 Sv per 2.5° column at 77.5–72.5°W.
     - **Return flow.** It is a broad 1 Sv per column across the east.

     The four gyre effects today prescribed as `sst_*` offsets come out of the basin geometry:
     - warm western boundaries;
     - cold eastern boundaries in the subtropics;
     - the subpolar gyres' cold western and warmer eastern sides.

3. **An overturning conveyor** (new; the prototype showed it is needed).
   - **Sinking.** In each snapshot, one sinking region per planet sits in the **narrowest
     polar-connected basin**: the basin of 20–70° latitude, in either hemisphere, with the smallest
     mean true width that has sea poleward of 50° free of ice in late winter. On Earth this picks
     the North Atlantic (47° wide) over the North Pacific (68°). Basin width is one of the leading
     explanations of why deep water forms in the Atlantic and not the Pacific (Ferreira et al.
     2018), and a model without salinity can use it.
   - **Upwelling.** Spread evenly over all sea equatorward of 50°.
   - **Flow.** A potential flow carries the transport Ψ (`climate.overturning_sv`, default 15 Sv;
     Earth's AMOC is about 17 Sv) from upwelling to sinking.
   - **Heat.** Flux = ρc_p·U·(T_upwind − T_deep), T_deep = 2 °C. This is conservative in flux form
     even though the flow has sources and sinks: heat is taken up where upwelled water warms and
     given back where the surface water cools on its way to sinking.
   - **Results.** On Earth's relief the prototype ran with the same settings without and with a
     15-Sv conveyor (runs E and F):

     | January sea surface | No conveyor | 15-Sv conveyor | Observed |
     |---|---|---|---|
     | off Brest | 4.3 °C | 9.2 °C | about 11 °C |
     | off Bergen | −3.7 °C | 0.4 °C | about 7 °C |
     | off Lisbon | 11.2 °C | 15.2 °C | about 15 °C |
     | off Seattle (the Pacific is untouched) | 9.5 °C | 10.0 °C | — |

     - Northern ocean heat transport peaks at 1.6 PW near 20–25°N, up from 0.7 PW. Trenberth &
       Caron 2001 put the ocean's at about 2 PW at 15–20°N.
     - 25 Sv gives 2.2 PW and 11 °C off Brest (run I).
     - With the alternative rule (the basin of largest E − P) the conveyor sank in the North
       Pacific, which is why the width rule is the default.
   - The sinking region and Ψ go into the run's notes and the events (§5.6).

**Not modelled:** sea-ice drift, salinity, the deep ocean's own heat store (it has a fixed T_deep),
coastal Ekman upwelling. Upwelling's dry coasts come from the cold eastern boundary and the stability
term in §3.4. These are open question Q9.

### 2.7 Sea ice per cell

Per cell, exactly today's thermodynamics:
- the slab holds at −1.8 °C while ice grows into E_i;
- ice melts from the top when the surface thaws;
- cover = min(1, h/0.5 m) brightens the sea tile and insulates it by k/h.

The seasonal cycle comes for free from the 72 steps a year. What is new is that **each cell freezes
from its own monthly heat budget**: a sea next to a cold continent, or downstream of the conveyor's
warm water, freezes or stays open on its own. The snapshot keeps the twelve monthly covers, and L1
receives the annual mean, the winter maximum and the summer minimum (§2.9).

**In the prototype** (best run, L):
- Sea ice over the sea has η² 0.89, against 0.998 today.
- The March northern edge ranges over 54–86°N (std 5.7°).
- **The Antarctic ice is far too small.** In September only 37 of 144 longitudes carry ice of 50 %
  or more (71–76°S, std 2.5°), and the plain-diffusion baseline reached only 61–64°S everywhere.
  The Southern Ocean has no gyres (the Drake rule), and the stronger ocean mixing warms it.

Calibration targets the Antarctic edge explicitly (§7.1).

### 2.8 Numerical scheme

One model year has 72 steps (5.07 days, `ebm_steps_per_year`). A step is:

1. **Local terms, implicit.** The 3 × 3 solve per cell of today's `ebm.rs`: radiation, OLR,
   land–sea exchange, surface–slab coupling. Cells run in parallel.
2. **Atmospheric diffusion of T̄, implicit, ADI** (Lie splitting).
   - **x sweep**: one cyclic tridiagonal solve per row (Sherman–Morrison).
   - **y sweep**: one tridiagonal per column.
   - Both run in parallel over lines.
   - Conductances are symmetric, so each sweep conserves Σ A·T̄ exactly. Implicit, so the polar rows
     (cells 8 km wide) need no time-step limit.
3. **Zonal advection of T̄.** An exact shift of s = β_T·u_a·Δt/Δx cells per row: its integer part
   moves whole cells; the fraction is donor-cell. Conservative and monotone. Numerical diffusion is
   at most Δx²/(8Δt) ≈ 3·10⁴ m²/s, against D's 2·10⁶ m²/s.
4. **Ocean.**
   - Slab diffusion, ADI as in 2.
   - Gyre + conveyor transport, explicit upwind, sub-stepped to Courant ≤ 0.5. The prototype needed
     3–6 sub-steps per step once the gyres stopped at 75°, against 372 with them reaching the
     poles.
5. **Freezing and melting** at the slab (heat moves to E_i and back; none is lost).

**Spin-up.**
- **First snapshot.** Each cell's tiles start from the 1-D model's band state at its latitude, which
  is converged by the existing carbon-balance search. It runs ≤ `ebm_spinup_years` (40), stopping
  once no cell's annual mean moves 0.005 K and no ice store 2 cm (today's criterion). The prototype
  converged in 17–20 years from a cruder start.
- **Later snapshots.** They warm-start from the previous snapshot's state, carried with the plates
  like the anomalies, and run ≤ `ebm_warm_years` (6).

**Determinism.** Line solves and per-cell work are independent. Flux accumulation in the sub-steps is
done per face, then gathered per cell in a fixed order. The result is the same on any thread count,
which is a test.

**Energy closure** is tested at snapshot level:
- |ASR − OLR| < 0.05 W/m² at convergence. The prototype gave 0.002–0.010.
- With radiation off, Σ heat changes by < 10⁻⁹ over a year.

### 2.9 Mapping to L1's monthly fields

For each month m and L1 cell c (rows north first), the steps are:

1. **Coarse tiles to L1.**
   - T_l* is the **masked bilinear** of the coarse land tiles: bilinear weights over the 2 × 2
     surrounding coarse cells, keeping only cells with f > 0 and renormalizing.
   - T_o* is the same over cells with f < 1.
   - If none of the four has the medium, the other's value is used (a tiny island takes the sea).
2. **Land cell:**
   `t_sl = T_l* + s_c·(T_o* − T_l*) − ΔT_inv`.
   - s_c is today's `sea_near` within `coast_km` (250 Earth km), reused. It moderates coasts
     finer than a coarse cell.
   - **ΔT_inv** is the winter surface inversion: snow-covered interiors in the dark season sit
     below the boundary-layer temperature the energy balance carries. Siberia's January surface is
     10–20 K below 850 hPa (Serreze, Kahl & Schnell 1992). The formula is
     `ΔT_inv = I_max·dark(m)·snow(T_l*)·interior(c)`, where:
     - dark = clamp(1 − Q_m/150 W/m², 0, 1);
     - snow is the albedo's snow factor;
     - interior = clamp((f_1000 km − 0.5)/0.5, 0, 1), with f_1000 km the land fraction within about
       1,000 km;
     - I_max = 15 K by default (`climate.winter_inversion_c`).
   - *WP-14a check.* Serreze, Kahl & Schnell 1992 could not be obtained, so the "10–20 K" stays
     from memory. The NCEP/NCAR reanalysis 1991–2020 monthly long-term means (Kalnay et al. 1996,
     NOAA PSL, public domain; computed WP-14a) put January air at the lowest model level (σ 0.995)
     8.1 K below 850 hPa at Yakutsk's grid point and 9.5 K below at Verkhoyansk's, against 9.4 K
     *above* at Brest's. Measured from what a 6.5 K/km lapse would give over the 1.3–1.5 km to
     850 hPa (about 9 K warmer at the surface), the inversion deficit is 17–19 K, and the
     reanalysis level is itself 7 K warmer than Yakutsk's station (−32 against −39 °C). The 15-K
     default sits inside that range.
   - The term is diagnostic. It does not enter the energy balance, so closure holds.
   - In the prototype, 15 K took Yakutsk from −23 to −34 °C (observed −39). It left Moscow alone,
     because Moscow's tile is above the snow threshold once the westerly heat reaches it. Winter
     RMSE fell from 5.9 to 4.9 K.
3. **Sea cell:** `t_sl = T_o*`.
4. **Sea ice.** `cover_L1(m)` is the masked bilinear of the coarse monthly covers, sharpened by a
   logistic of slope `climate.sea_ice_edge_sharpness` (default 4). An L1 sea cell's cover is thus
   its own number, edges are crisp, and they follow the coarse pattern.
5. **Then exactly as today:**
   - `t_mean`, `t_warm` and `t_cold` from `t_sl − Γ·max(0, h − sea)`;
   - Köppen from the twelve months;
   - aridity (Thornthwaite);
   - the ELA from the summer `t_sl` and the annual precipitation.

**Fields written:**

| Field | Change |
|---|---|
| `climate/month/t_sl_c_01…12`, `climate/month/p_mm_01…12` (last snapshot) | same names, same meaning; per-cell values |
| `climate/{t_mean_c, t_warm_c, t_cold_c, precip_mm, aridity, koppen, ela_m}` | same |
| `climate/sea_ice` (annual mean, 0–1, 0 on land) | same name; now per cell |
| `climate/sea_ice_max`, `climate/sea_ice_min` | **new** pipeline fields: winter maximum and summer minimum monthly cover |
| `climate/anomaly/*` (carried between snapshots) | same; the reference becomes the 1-D clock's corrected band value (§4.1) |

The month convention (month 1 starts at solar longitude 280°) and FORMAT.md "The climate at a
block" stay as they are.

### 2.10 Sources for this section

North, Mengel & Short 1983 (seasonal EBM with explicit continents); Zhuang, North & Stevens 2017 and
Ziegler & Rehfeld 2021 (TransEBM; the NMS83 constants and surface-dependent D); North & Kim 2017
(*Energy Balance Climate Models*, ch. 8, 2-D EBMs); North, Cahalan & Coakley 1981 (A, B, D, the P₂
albedo); climlab (Rose 2018; today's port); Stommel 1948 (gyres); Rose & Marshall 2009 (ocean heat
transport and sea ice in EBMs; their wind-curl diffusion is an alternative to our gyres); Ferreira
et al. 2018 (Atlantic–Pacific asymmetry of deep-water formation); Trenberth & Caron 2001 (heat
transports); Serreze, Kahl & Schnell 1992 (Arctic inversions); Wagner & Eisenman 2015 (sea ice in
EBMs). Licences are in §9.

---

## 3. Rain from a moisture budget

### 3.1 The idea

The climate grid also carries the column water vapour W (kg/m²). The vapour's residence time is
about 8–10 days, much shorter than a month, so **each month's W is the steady state** of that
month's evaporation, transport and precipitation. One steady solve per month per snapshot gives the
rain, conserves water exactly at convergence, and needs no time-step limit, polar rows included.

```
∇·(V W) − ∇·(K_q ∇W) = E(W) − P(W) − S_oro
```

V is the month's low-level wind, K_q the eddy moisture diffusivity (1.5·10⁶ m²/s), and S_oro the
mountain rain taken out on L1 (§3.6).

### 3.2 Winds

V = belt wind + thermal wind.

1. **Belt wind** in belt latitude φ′ = φ − shift(φ, ITCZ_column(m), hemi(m)).
   - **ITCZ.** The thermal equator of the 2-D model's own T̄: the soft maximum over |φ| ≤ 35°,
     clamped to ±`itcz_max_shift_deg` and smoothed along the row. This is today's rule in
     `belts::itcz`, now fed by per-cell temperatures. Over a large summer continent it swings up to
     22°, and the trades that cross the equator turn into monsoon westerlies (today's
     `regime_weights`).
   - **Zonal component**: today's regime winds.
   - **Meridional component**: the three-cell surface flow, which sets where moisture converges:
     ```
     v = −2.5·sin(π|φ′|/30)·sgn φ′          |φ′| < 30°   (trades toward the ITCZ)
         +1.0·sin(π(|φ′|−30)/30)·sgn φ′     30–60°       (Ferrel, poleward)
         −0.5·sin(π(|φ′|−60)/30)·sgn φ′     > 60°        (polar, equatorward)
     ```
     It converges at the ITCZ and at 60° and diverges at 30° and the poles. That gives the wet and
     dry belts **from the budget**, replacing the `belt_precip` table.
     *WP-14a check* (NCEP/NCAR reanalysis 1991–2020, annual zonal mean at σ 0.995, computed): the
     trades blow equatorward at 1.4–1.5 m/s at 15–20°N and 2.0 m/s at 10–15°S (the profile: 2.5);
     the Ferrel branch poleward at 0.7 m/s at 50°N and 2.3 m/s at 50°S (the profile: 1.0 at 45°).
     Within a factor of two everywhere, hemispherically averaged; a T9 knob, not a sourced value.
2. **Thermal wind** (monsoon lows and winter highs).
   - **Pressure.** The departure of T̄ from its row mean, smoothed over two coarse cells, makes a
     pressure anomaly p′ = −κ_p T̄′. κ_p is 60 Pa/K (ρ g H/T with H ≈ 1.5 km).
   - **Wind.** The linear friction–Coriolis balance of Lindzen & Nigam 1987 turns it into wind:
     ```
     r u − f v = −p′_x/ρ,   r v + f u = −p′_y/ρ   ⇒   u = −(r p′_x + f p′_y)/(ρ(r² + f²)),  v = −(r p′_y − f p′_x)/(ρ(r² + f²))
     ```
     with r = 10⁻⁵ s⁻¹.
   - **Behaviour.** It is cross-isobaric near the equator and almost geostrophic in mid-latitudes.
     A summer continent 10 K warmer than its row gets about 5 m/s of inflow: an onshore monsoon.
     A winter continent gets outflow: a dry winter monsoon.
   - *WP-14a.* Lindzen & Nigam 1987 could not be obtained; their friction constant is unread, so
     r = 10⁻⁵ s⁻¹ stays the design's own value. κ_p = ρgH/T is the hydrostatic surface-pressure
     change of a layer of depth H warmed by T′ (1.2 × 9.81 × 1,500 / 290 = 61 Pa/K, computed).
     Back & Bretherton 2009 (J. Climate 22, 4182–4196; read in full, the authors' copy) say
     Lindzen & Nigam needed a layer "extending up to 700 hPa" (about 3 km) and an ad hoc back
     pressure, and that a mixed layer to 850 hPa (about 1.5 km) is more realistic (pp. 4183 and
     4195): the design's H is that more realistic depth.

### 3.3 Evaporation

Bulk formula per tile (GREB's form, Stassen et al. 2019):
`E = ρ_a C_E max(|V|, 3 m/s) q_s(T_tile) (1 − RH)`, where:
- RH = W/W_sat;
- C_E = 1.3·10⁻³;
- q_s is from Tetens;
- the sea tile has the factor (1 − ice cover);
- the land tile has the bucket factor β(S) of Manabe 1969: β = min(1, S/(0.75 S_max)), S_max
  150 mm, with S stepped month by month by P − E − runoff and cycled until the year repeats.

**Recycling** (interiors kept moist by their own evaporation) comes from the bucket. It replaces
today's `recycling` and `moisture_floor` constants.

W_sat = 70 kg/m² · e^(0.065(T̄ − 27)) · (1 − f + f·e^(−z̄/2,500 m)). The last factor is the vapour
scale height over high land, so a Tibet holds a fifth of the sea-level water. It replaces
`plateau_m`.

### 3.4 Condensation and precipitation

```
P = stab · (W/τ_P) · exp(a (RH − r_ref)),    τ_P = 6.6 days, a = 11.4, r_ref = 0.7
```

- **The exponential on RH** is Bretherton, Peters & Back 2004's observed relation over the tropical
  oceans. It is written relative to W, so P scales with the air's water as Clausius–Clapeyron
  demands.
  - *WP-14a (source read):* the paper gives two fits. The daily one is P = exp[15.6 (r − 0.603)]
    mm/day (eq. 1, p. 1519); the **monthly-mean** one is P = exp[11.4 (r − 0.522)] mm/day (eq. 2,
    p. 1521). 15.6 is an e-folding rate, not "15.6-fold per unit of RH". This budget solves monthly
    means, so it takes the monthly rate, a = 11.4 (was 15.6), and τ_P = 6.6 days (was 8), which
    makes the law equal Bretherton's monthly fit at their typical convecting column (W_sat = 72 mm,
    p. 1521) and r = 0.7: 50.4 mm / 6.6 days = 7.6 mm/day = exp(11.4 × 0.178). The prototype ran
    15.6 and 8 days; T9 calibrates `precip_rh_slope` from here.
- **Typical rates** (a = 11.4, τ_P = 6.6 days): an ITCZ column (W 50, RH 0.75) rains about
  13 mm/day; a subtropical high (W 25, RH 0.45) about 0.2 mm/day; a mid-latitude column (W 15,
  RH 0.7) about 2 mm/day.
- **Convergence.** Convergent winds pile W up until it rains: the ITCZ, the polar front, monsoon
  inflow. Divergence dries: the subtropical highs. This plays the role of GREB's ω term.
- **Subsidence on the west sides of continents.**
  - Over sea colder than its row's sea mean, rain is suppressed:
    stab = e^(0.3·min(0, T_o − ⟨T_o⟩_row)), so 5 K colder gives ×0.22.
  - The cold eastern-boundary seas come from the gyres (§2.6). Together they make the cold-current
    deserts (Atacama, Namib, Baja) on the east sides of the subtropical highs.
  - Today's `subsidence_factor` (basin position) stays as a modulator on the east side of basins
    until calibration shows the stability term alone suffices.

### 3.5 Numerical scheme, stability, conservation

- **Discretization.** Upwind flux form on the climate grid, with a two-point diffusive flux. The
  linearized system is an M-matrix: upwind inflows are off-diagonal and negative, and outflow,
  diffusion and ∂P/∂W − ∂E/∂W > 0 sit on the diagonal. So Newton–Gauss–Seidel iterations converge
  monotonically and keep W ≥ 0.
- **Solver: zonal line relaxation.** Each row is solved implicitly with its neighbours held: one
  Newton step of a cyclic tridiagonal. Rows sweep north to south, then back.
  - The prototype converged to 10⁻⁸ kg/m²/s in about 60 sweeps per month, 0.2–0.3 s single-threaded
    for all twelve months.
  - Point Gauss–Seidel needed 3,000 sweeps, because the polar rows are stiff.
  - The months are independent, so they run in parallel.
- **Conservation.** At convergence Σ A·E = Σ A·P + Σ A·S_oro exactly, up to the tolerance. The
  prototype's global imbalance was 2–3·10⁻⁶ of the evaporation, and it is a test.
- **Coupling.** The moisture budget reads the energy balance's monthly temperatures, but its
  latent heat does not feed back: D in §2.5 already includes the latent part of the transport, as
  in the 1-D model. A moist energy balance (Siler, Roe & Armour 2018) is a later option (Q10). One-way
  coupling keeps the energy balance's calibration and the clock's 1-D model consistent.

### 3.6 Mountain rain and rain shadows

**Smith & Barstad stays on L1, unchanged** (`orographic.rs`, the MIT port: one FFT per wind
direction over the rows it reaches).
- **Directions.** Instead of eight belt regimes, the eight directions are the eight compass
  directions of the coarse wind V (bilinear to L1). Each L1 cell weights the two directions either
  side of its wind by angle, scaled by its speed over the direction's reference speed. The cost is
  the same eight FFTs as today.
- **Rain at L1.**
  `P_L1 = max(P_min, P*_bg + q·P_SB)` on the windward side, and
  `P*_bg·max(oro_lee_floor, 1 + q·P_SB/P*_bg)` in the lee. Here:
  - P*_bg is the coarse rain, bilinear;
  - q = RH*/r_ref is the coarse humidity, bilinear, standing in for today's marched moisture;
  - P_SB keeps its Clausius–Clapeyron temperature factor.
- **Shadows.** Per coarse cell, the windward excess (the mean over its L1 land of max(0, q·P_SB))
  is the sink S_oro in §3.1. The moisture budget then carries less water past the range. This is
  the large-scale rain shadow (a Patagonia, a Great Basin), and it replaces today's barrier march.
  The fine-scale shadow right behind a ridge is SB's own lee term.
- **Iteration.** Fixed point, budget → SB → sink → budget. Two passes suffice because the sink is a
  small share of the budget.

With `rain_model = budget`, `lapse.rs`'s moisture march and its constants (`moisture_decay_km`,
`moisture_floor`, `recycling`, `ocean_recharge_km`, `barrier_recover_km`, `vapour_scale_m`,
`air_mixing_km`, `plateau_*`, `onshore`) are unused. `belts` keeps them (§3.8, §5.7). The march's
*temperature* role (maritime influence) retires with per-cell temperature in both modes: the 2-D
advection and the coast blend replace it.

### 3.7 Does it give the right deserts and wet coasts?

The prototype has everything in §3.2–3.5, including the ITCZ per column, the thermal wind, cold-sea
stability and the bucket. It lacks the SB coupling and `subsidence_factor`, and it was not
calibrated. On Earth's relief at 2.5° (runs L and N: without and with the bucket):

| Annual P, mm | Prototype, no bucket | Prototype, bucket | Today | Observed |
|---|---|---|---|---|
| Sahara 23°N 10°E | 563 | **32** | 265 | 11 |
| Kazakh steppe | 331 | **170** | 758 | 142 |
| Central Australia | 353 | **255** | 311 | 248 |
| Gobi | 122 | 8 | 431 | 112 |
| London | 826 | 986 | 1,080 | 702 |
| Amazon | 1,377 | 1,182 | 1,311 | 2,234 |
| Congo | 941 | 544 | 924 | 2,156 |
| Mumbai | 1,213 | 604 | 531 | 3,096 |
| Bangladesh | 7 | 2 | 278 | 2,194 |
| SE China | 365 | 184 | 408 | 1,765 |
| Atacama coast | 1,391 | 1,227 | 154 | 16 |
| Namib | 1,361 | 966 | 233 | 61 |

- **What works:**
  - **Water closes**, to 3–4·10⁻⁶ of the evaporation.
  - **Variety is Earth-like or more.** Latitude explains 40–43 % of ln P over land, against 41 %
    observed and 75 % today. The along-row std of ln P is 1.1–1.5, against 0.84 observed and 0.37
    today.
  - **The ITCZ excursion over continents works.** It took Mumbai from 200 to 1,200 mm before the
    bucket.
  - **The bucket makes the big deserts.** Dry soil stops recycling: the Sahara at 32 mm.
- **What does not work yet.** On the 2.5° grid the ln P RMSE against observed is 1.7–2.0, worse than
  today's 1.05. Three errors dominate it, and each has a named fix to try in calibration (§7.5):
  1. **Too dry**: monsoon Asia, the Congo and the Gobi. The bucket's dry state is self-sustaining
     wherever the prototype's moisture supply is weak.
     - Bangladesh's 2.5° cell holds the Himalayan front, so the plateau factor cut its W_sat. The
       fix is the cell's *lowland* height for W_sat, with SB doing the mountains.
     - Other fixes to try are a larger κ_p and a lower r_ref.
  2. **Too wet**: the cold-current coasts.
     - These coastal cells are half sea, and the sea half's evaporation rains on the land half.
     - The Andes do not shadow the Atacama without SB.
     - Fixes to try: SB's sink, a stronger stability term, and evaporation from the sea tile kept
       out of the land tile's rain.
  3. **Gyre-cold upwelling** is not strong enough yet (Q9).

### 3.8 Building rain behind a switch

Because of §3.7, `climate.rain_model` has two values:

- **`belts`** (the first default): today's rain scheme, but fed by the 2-D model:
  - the ITCZ from the 2-D T̄ per column;
  - ΔSST from the 2-D sea tile's departure from its row mean, instead of the prescribed gyre
    offsets;
  - basin position from the same coastline as the gyres.

  Today's Earth skill (ln P RMSE 1.05–1.11) is kept while temperature and sea ice go per cell.
- **`budget`**: §3.1–3.6. It becomes the default when, on Earth's relief:
  - its ln P RMSE is below the belts' value;
  - its Köppen main-group agreement is at least the belts' value;
  - the §7.1 desert and monsoon points are met.

T4 builds it; T9 calibrates it and flips the default.

---

## 4. What stays banded

### 4.1 The fast clock keeps the 1-D model, flux-corrected to the 2-D zonal means

The fine clock (`clock.rs`) steps carbon, the EBM and the ice every 100 kyr (200 kyr on the ice
branch). That is 2,500–5,000 steps per 500 Myr with up to 10 EBM years each, plus 1,500 steps in
the 3-Myr orbital window. The 2-D model costs about 0.1 s of one core per model year, so running it
there would add 40–80 minutes single-threaded, or 10–20 minutes on all cores. **The clock keeps the 90-band 1-D EBM.** Two changes make it consistent with the
2-D climate:

1. **A per-band flux correction F_b.**
   - **Fit.** At every 2-D snapshot, F_b (W/m², one per band) is fitted so the 1-D model's annual
     band means of land T and sea T match the 2-D model's zonal means of the same tiles (area
     weighted by tile). The update is F_b ← F_b + λ_b (T2D − T1D), with λ_b ≈ B + 2D/Δφ², for about
     10 iterations of 6 warm-started 1-D years. That costs about 40 ms per snapshot, at about 1 ms
     per 1-D year.
   - **Use.** F_b is added to the 1-D model's absorbed sunlight until the next snapshot. It stands
     for everything 2-D the 1-D model cannot see: gyres, the conveyor, the land distribution within
     the band, the cell-local ice albedo.
   - **Guard.** |F_b| is capped at 15 W/m², and the run's notes report its range.
   - **Expected size.** With the 2-D model's zonal-mean structure the prototype's zonal RMS against
     the uncorrected 1-D model was 0.18 K with plain diffusion and 1.5–4.8 K with the
     surface-dependent D and the ocean terms. So F_b will be a few W/m², mostly at high latitude.
2. **Pattern scaling between snapshots.** This is today's mechanism, with a new source.
   - **Anomaly.** At a snapshot, each L1 cell's anomaly is its 2-D downscaled value minus the
     corrected 1-D band reference:
     `d_mean = t_mean + Γz − refs.t_mean[row]`, and likewise for warm, cold and ln P.
   - **Carry.** It is carried with the plates.
   - **Between snapshots.** The value is `refs(clock now) + anomaly`.
   - **Sea ice between snapshots** moves with the band's sea-ice change relative to the snapshot: a
     cell's monthly cover is shifted by the band's cover change, then clamped.
   - **Code.** `climate/mod.rs`'s "advect" path keeps its shape; only the snapshot's source changes.

### 4.2 Snowball and hothouse triggers

- **Triggers stay as they are.** The clock's 1-D model decides them: sea-ice cover ≥ 0.95
  (`snowball_on_frac`) and global mean > 25 °C (`hothouse_c`). With F_b the 1-D model starts from
  the 2-D model's zonal state, so it runs away at about the forcing at which the 2-D model would.
- **New:**
  - **Triggered snapshots.** When the clock's global mean moves by more than 3 K, or its sea-ice
    cover by more than 0.15, against the last snapshot, the stage takes a 2-D snapshot at the end
    of the plate step instead of waiting up to 10 Myr. This refreshes F_b, the ice albedo per cell
    and the pattern through a snowball's onset and escape. Cost is one extra snapshot per crossing.
  - **A test.** On the existing snowball scenario (tropical supercontinent, weathering ramp), the
    2-D model run at the onset CO₂ must also freeze over, and run at the escape CO₂ must thaw.
  - **The escape CO₂ is reported against Earth's (WP-14a).** The clock's CO₂ forcing, which this
    design keeps, is Byrne & Goldblatt 2014's fit F = 5.32 ln(C/C₀) + 0.39 [ln(C/C₀)]², stated for
    200–10,000 ppm and computed by them over 100–50,000 ppm (Table 2 and abstract; read WP-14a),
    blended from Myhre's logarithm above 1,000 ppm (CLM-069 to CLM-071). Above 50,000 ppm it is
    an extrapolation with no source behind it. Today's snowball scenario escaped at about
    550,000 ppm (0.55 bar), where Hoffman et al. 2017 expect terminations at 10⁴ to 10⁵ ppm
    (p. 3 of 43; read WP-14a) and note that cloud-free models do not deglaciate at any feasible
    CO₂ while cloud condensate adds ≥ 10 W/m² (p. 10 of 43). T6 records the 2-D model's escape
    CO₂ in the run's notes beside 10⁴–10⁵ ppm. What to do above 50,000 ppm (cap the forcing, find a radiative
    calculation that covers 0.1–1 bar, or add the snowball's cloud and dust forcing) is an open
    question for Ben (STATUS "Pending questions", WP-14a); until then the forcing law is unchanged.
- **The ice-albedo feedback.**
  - In the 2-D model it is per cell: snow by tile temperature, sea ice by cover, sheets by s_g.
  - In the clock it is per band, as today.

### 4.3 The orbital window

The last 3 Myr keep the clock's 2-kyr steps, the 1-D model and the per-band equilibrium-line shift
δ_b. The ice branch's sheet already forces its coarse grid with `ela0_c + δ[band]`: the snapshot's
per-cell ELA plus the band shift. With per-cell climate, ela0_c becomes the 2-D snapshot's
downscaled ELA, so ice sheets grow where the 2-D climate is coldest and snowiest. That is a
Laurentide over a cold Canada-like region, not a band.

Inside the window an option, `climate.orbital_snapshot_kyr`, re-runs the 2-D snapshot every so
often:
- default 0, none;
- 100 kyr gives 30 more snapshots and about +10–30 s;
- it lets the sheets' own albedo cool their region in 2-D.

This is open question Q6.

### 4.4 Glacio-eustasy

Unchanged: sea level follows the ice volume of the ice fields (`ice.rs`), whatever made the ice.
Per-cell climate changes *where* ice grows, and so the volume. `ice_reference_km3` (the calibrated
present ice) must be re-measured on the calibration runs (T9).

### 4.5 Weathering (per-cell runoff)

`kernel_of` already sums each land cell's runoff (Fu's Budyko curve on the cell's P and T) and
temperature into per-band kernels. The clock scales each band by its temperature change, and the
ice branch adds a per-band supply limit from erosion. **This code does not change; its inputs
improve.**
- Dry interiors and cold-current coasts weather less.
- Monsoon coasts weather more.

The Earth normalisation `KERNEL_EARTH` (2.845·10⁷, measured with `climate-dem`) must be
re-measured on Earth's relief with the new climate (T9). Otherwise the carbon cycle's balance CO₂
drifts. Using the bucket's runoff instead of Fu's curve is open question Q11. Fu stays, so the
climate and the surface stage use one runoff.

### 4.6 How it fits the ice branch

The branch (`458c15c`) adds:
- `climate/sheet.rs`: shallow-ice flow on a coarse grid of about 256 columns, the elevation
  feedback, isostasy and calving;
- supply-limited weathering;
- the 200-kyr clock.

This design depends on it as follows:
- **One coarse-grid type.** `sheet::Grid` moves to `climate/coarse.rs` and gains masked bilinear
  refinement and unit-sphere areas. The sheet keeps its 256 columns; the climate uses 128.
- **The sheet's forcing** reads `ela0_c` (now per-cell 2-D) and `precip0_c` (now the moisture
  budget's) unchanged in form.
- **Ice albedo.** The sheet's ice share per *coarse climate cell* feeds the 2-D model's s_g. Its
  per-band share keeps feeding the clock, as it does on the branch.
- **No change** to supply limits, calving, isostasy or the clock step.
- **The ice takes the planet's gravity (WP-14a; science stage 1 fix 17).** `ice.rs` uses a fixed
  `G = 9.81` in Nye's profile H = √(2τ₀L/(ρ_i g)), and the branch's `sheet.rs` imports the same
  constant into its shallow-ice rate factor 2A(ρ_i g)ⁿ/(n + 2). Both read `planet.gravity_m_s2`
  instead, passed in by T6, which touches the sheet anyway. Test: on a planet with g = 3.71 m/s²
  the Nye thickness at the same L and τ₀ is √(9.81/3.71) = 1.63 times Earth's, and the sheet's flux
  for the same geometry is (3.71/9.81)³ = 0.054 times. Earth's presets carry 9.81, so Earth-like
  worlds do not change. Glacio-eustasy and the ice's isostatic depression (ρ_i/ρ_m) do not depend
  on g.

---

## 5. Downstream effects

### 5.1 World data and the climate API: unchanged

The baked layers are exactly today's:
- `climate/t_mean_c`, `climate/p_mm` and `climate/koppen`;
- the twelve `climate/month/t_sl_c_NN`, with the lapse rate taken out;
- the twelve `climate/month/p_share_NN`;
- `climate/ice_cover`;
- the five biome inputs.

`notes.bake.climate.monthly` keeps its keys.

**FORMAT.md "The climate at a block" does not change**, so Rust's `ClimateWorld`, Java's
`ClimateLayers` and `DeepTimeClimate` (`API_VERSION` 1) stay bit-for-bit as they are. mic-climate
needs no change. `deeptime parity` and `ClimateParityTest` re-run as regression checks only.

The format version stays 0.2. The new `climate/sea_ice_{max,min}` are pipeline and keyframe fields,
not baked. Baking sea ice for an in-game seasonal freeze is a later choice (Q8).

### 5.2 Biome mapping and placement

`bake/biomes.rs` reads the Köppen class, the warmest and coldest months, the aridity, the glacier
flag and, for the sea, the annual surface temperature, with sea ice below −2 °C → frozen ocean. None
of these change in meaning, so no code changes. Effects:
- Frozen-ocean biomes follow the per-cell sea temperature. **The in-game frozen-ocean edge stops
  being a straight line.**
- Taiga, tundra and snowy biomes follow the new isotherms.
- Deserts move to interiors and cold coasts.
- The placement table (`placement.json`, pack-overridable) needs no change. Its per-class bands stay
  valid.

**Köppen on L2 (open, WP-14a note).** The grid-artifact audit (build log 2026-10-01, P1-23,
finding 3; STATUS "Pending questions") recommends deciding the Köppen class on L2 inside P1-26, so
biome borders stop stepping with the 16-block L1 cells. This design does not include it: §2.9 and
T5 classify on L1 "exactly as today". It accommodates it without a format change. The class is a
function of the twelve monthly temperatures and precipitations, which §2.9 already produces from
smooth fields (masked bilinear of the coarse tiles, the coast blend, the inversion), and the bake
already holds the twelve `month/t_sl_c_NN` and `month/p_share_NN` layers. A step in
`bake/biomes.rs` can interpolate those to each L2 cell, lapse to the L2 height and classify there,
then set the band coordinates per L2 cell; the baked L1 `climate/koppen` stays for the climate API.
If Ben takes the audit's recommendation it becomes a T5/T10 item (Sonnet; acceptance: the audit's
share of biome-input change faces within 2° of an axis falls from 24 % toward the 4.4 % of an
isotropic field).

### 5.3 The `climateConsistent` GameTest floors

The floors are per Köppen family shares of columns whose biome carries the matching tag:
A 0.6, BW 0.65, BS 0.42, C 0.75, D 0.9, Dsub 0.85, E 0.9, sea 0.95, BW:desert 0.45, Af:jungle 0.5,
Dfc:taiga 0.33, E:snow 0.72. They were set on `test_planet_real` "about 0.1 under the lower of
standalone and MIC".
- **What may change.** The biome logic is unchanged, but the test planet's climate is new, so its
  columns per class change. Families with few columns move most: BW had 17 columns at the first
  calibration.
- **Procedure.** Re-measure on the merged tree in the coordinator's batch, which runs GameTests
  once per merge batch (2026-09-30), and re-base any floor the census misses with the same rule:
  0.1 under the lower of standalone and MIC, documented in `BiomeGameTests`. A drop of more than
  0.15 in a family with over 100 columns is treated as a bug in the climate, not a floor to lower.

### 5.4 Surface weathering and runoff

`surface/climate.rs` reads `climate/t_mean_c` and `climate/precip_mm` and derives PET, Fu's ET,
runoff and aridity. The interface is unchanged. Rain moves from belts to monsoon coasts and
windward slopes, so **erosion and the calibrated heights move**. P1-17's targets (land median
0.6–1.1 km, p90, glaciated share 3–18 %) must be re-checked on the calibration runs (T10). If
heights drift, retune erosion, not the climate.

### 5.5 Deposits and other readers of the climate

- The deposit engine's frames read `climate/{t_mean_c, precip_mm}` and the surface's facies
  (evaporite, coal, carbonate platform), which depend on aridity and temperature.
- `structure` reads `t_mean` for its surface temperature.
- Evaporites and coal follow the new deserts and wet lowlands.
- The four families' counts must be re-checked (T10): the 1-km survivor formula and USGS densities
  are unchanged, so the counts move only through the geography of facies.

### 5.6 The satellite view, keyframes and events

- **Satellite view.** The band-hiding blurs go: ±3 rows for temperature and 5 × 17 cells for sea
  ice become 1 × 1 and 1 × 3 (`satellite.rs:717–731`). Sea ice draws per cell, and the new layers
  can draw winter and summer ice.
- **Keyframes.** `climate/sea_ice` stays a layer, and `climate/sea_ice_max` and
  `climate/sea_ice_min` are added. Optional, per Q7: the month's coarse wind as a vector layer.
- **Events and notes.** The conveyor's sinking region (lat, lon, Ψ) at each snapshot goes into
  `notes.climate`. When it moves between basins, the stage writes an event `deeptime:ocean_overturning_shift`
  (EVENTS.md). It is cheap and it gives "why here" for a mild coast.

### 5.7 Workbench and editor settings

World-shaping knobs are settings (standing rule, 2026-09-29). Defaults reproduce the calibrated
model. Paths are under the preset's `climate` section, edited in `editor_fields.json`, Java's
`EditorFields`, the lang file and `AdvancedSettingsTest`, with the built-in presets synced by
`tools/dev/sync_builtin_presets.py`.

**New:**

| Setting (path) | Default | Editor | What it does |
|---|---|---|---|
| Climate model (`climate.model`) | `per_cell` | Advanced, a choice | `per_cell` or `banded` (today's, kept for comparison and old presets) |
| Climate grid (`climate.grid_columns`) | 128 | Advanced | 90 (4°, faster) / 128 / 180 (2°) |
| Wind-carried heat (`climate.wind_heat_transport`) | 0.5 (calibrated) | Advanced | how far the prevailing wind carries coastal mildness inland; 0 = none |
| Zonal heat mixing (`climate.zonal_diffusivity_ratio`) | 0.5 (calibrated) | Advanced | east–west against north–south heat diffusion |
| Ocean gyres (`climate.gyre_heat_efficiency`) | 1.0 (calibrated) | Advanced | how much heat the wind-driven currents carry; 0 = none |
| Ocean mixing (`climate.ocean_diffusivity_w_m2_k`, exists, not yet in the editor) | 0.8 (today 0.3; calibrated) | Advanced | eddy heat transport in the sea |
| Ocean overturning (`climate.overturning_sv`) | 15 Sv | Advanced, stops Off 0 / Earth 15 / Strong 25 | the deep-water conveyor that warms one subpolar sea; 0 = off |
| Winter inversions (`climate.winter_inversion_c`) | 15 °C | Advanced | how much colder snow-covered interiors get in the dark season than the air above them |
| Monsoon strength (`climate.thermal_wind_pa_k`) | 60 Pa/K | Advanced (`budget`) | how strongly hot continents draw in sea air in summer (and push it out in winter) |
| Rain efficiency (`climate.precip_rh_slope`) | 11.4 (WP-14a; was 15.6) | Advanced (`budget`) | how sharply rain grows with humidity: higher makes wet places wetter and dry places drier |
| Soil water (`climate.bucket_mm`) | 150 mm | Advanced (`budget`) | how much rain the land stores and re-evaporates (inland reach of moisture) |
| Sea-ice edges (`climate.sea_ice_edge_sharpness`) | 4 | Advanced | 1 = soft, coarse edges; higher = crisp L1 edges |
| Orbital-window 2-D snapshots (`climate.orbital_snapshot_kyr`) | 0 | Advanced (Q6) | extra 2-D climates inside the ice-age window |

**Kept:** tilt, star, CO₂ settings, lapse rate, `monsoon_shift` (`itcz_max_shift_deg`, still the
ITCZ's reach), `mountain_rain` (`oro_hours_per_day`), snowline, ice-age settings.

**Rain model** (`climate.rain_model`): `belts` (first default) or `budget` (§3.8), Advanced. The
rain settings follow it:
- `rain_reach` (`moisture_decay_km`) keeps its meaning under `belts`. Under `budget` the editor shows
  Soil water (`bucket_mm`) under the same id and hint ("How far sea air carries its moisture over
  land").
- Monsoon strength and Rain efficiency apply under `budget` only.

`ice_sheets` (`ice_sheet_feedback`) stays, now per cell. Mountain blocking
(`climate.mountain_blocking`, 0–1, default 0 until calibrated) joins the Advanced list.

**Retired under `per_cell`, still read under `banded`:**
- `maritime_decay_km`, a temperature role that the 2-D advection replaces;
- the four `sst_*` offsets, which the gyres and the conveyor replace.

`moisture_floor`, `recycling`, `ocean_recharge_km`, `air_mixing_km`, `vapour_scale_m`,
`barrier_recover_km`, `plateau_*` and `onshore` stay in use under `rain_model = belts`. All keys
stay valid in presets, as `deny_unknown_fields` requires, and their doc comments say where they
apply.

---

## 6. Cost

### 6.1 Measured

| What | Measured | Where |
|---|---|---|
| Today's climate stage, 500-Myr `earthlike_quick_16k` | 186 s of an 864-s run: clock 76.5, snapshots 38.5 (51 × 0.75 s), ice 31.4, topography 20.8, advection 16.0 | loaded M6, load average 80–95, 12 cores |
| Today's snapshot on Earth's relief, 720 × 360 | first 0.82 s, then 0.46 s each: cells 246 ms, orographic 196 ms, belts 61 ms, **1-D EBM 1.7 ms** | same |
| Today's snapshot on a quiet Mac, 1024 × 512 (2026-09-29 workbench run, older climate code) | 0.11 s per snapshot (5.6 s / 52) | quiet M6 |
| Prototype 2-D EBM, 144 × 72 (10,368 cells), one thread, plain diffusion | **0.10 s CPU per model year**: 18 years in 2.2 s of user time, incl. the moisture | loaded M6 (wall times 3–30× higher from contention) |
| Same, all §2 terms (surface-dependent D, blocking advection with sub-steps, gyres + conveyor at 6 sub-steps) | **0.13 s CPU per model year**: 3.07 s of user time in all, 0.83 s of it the moisture, for 17 model years | same |
| Prototype moisture budget, 12 months, line relaxation | 0.2–0.3 s (630–890 sweeps); with the bucket's three passes over the year 0.66–0.83 s (about 2,000 sweeps), one thread | same |
| Prototype Stommel solve | 0.01–0.03 s | same |
| Prototype peak memory (maximum resident set) | 26 MB | same |

### 6.2 Estimate per snapshot and per run (quiet M6, 12 threads)

Scaling rules:
- 128 × 64 is 0.79× the prototype's cells.
- Line solves and per-cell work parallelize about 4–6× (64–128 independent lines; the shared Mac
  showed no clean multi-thread numbers).
- The twelve moisture months run in parallel.

| Part | Quick (L1 1024 × 512) | Full (L1 2048 × 1024) |
|---|---|---|
| 2-D EBM, warm snapshot, ≤ 6 years | 0.08–0.5 s | same (the climate grid does not scale with L1) |
| Moisture, 2 passes with SB | 0.05–0.6 s | same |
| SB orographic, 8 directions (as today) | 0.01–0.2 s | 0.04–0.8 s |
| Downscaling to L1 (masked bilinear, inversion, Köppen, ELA) | 0.1–0.3 s | 0.4–1.2 s |
| 1-D flux-correction fit | 0.04 s | 0.04 s |
| **Per snapshot** | **0.3–1.2 s** (today 0.11 quiet / 0.75 loaded) | **0.5–2.4 s** |
| First snapshot (spin-up ≤ 20 years) | +0.4–2 s | +0.4–2 s |
| **500 Myr (52 snapshots + about 2 triggered)** | **+15–60 s** | **+25–120 s** |

The fine clock is unchanged: 76.5 s loaded today, and the ice branch's 200-kyr step halves it.
Against the time budgets (quick about 5 min, full about 15 min on a 6–8-core PC), the quick preset's
climate grows by roughly a quarter of today's whole run at the top of the range.

**Two cheaper knobs** if P1-17's benchmark gate says so:
- `climate.grid_columns` 90 halves the 2-D cost.
- `snapshot_every_myr` 20 halves the snapshot count.

The calibration task (T9) measures the real numbers on a quiet Mac. It also decides the default
`ebm_warm_years` for the 2-D model; 3 may suffice, since the slab adjusts in months.

**Memory:**
- The climate grid's state and monthly buffers are about 7 MB.
- L1 gains two f32 fields (`sea_ice_{max,min}`): 4 MB at quick, 16 MB at full.
- The estimate in `climate::estimate` adds 8 B per L1 cell.

---

## 7. Validation and calibration

### 7.1 Earth's relief against observations

**Tool.** `deeptime climate-dem` gains `--dump <dir>`: the twelve monthly t_sl and P, the coarse
fields and the sea ice. A validation script, `tools/validation/climate_vs_obs.py`, compares them
with:
- Beck et al. 2018 Köppen (present, 0.5°);
- Beck et al. 2023 monthly T and P, 1961–1990 and 1991–2020, 0.5°.

The script (re-implementing this document's metrics) downloads the data into a gitignored cache.
**Nothing observed is committed or shipped.** Both datasets are CC BY 4.0, which would allow it, but
the plan keeps validation data out of the repo. Optional extras are HadISST sea ice (Open Government
Licence) and GPCP precipitation (NOAA, public).

**Targets on Earth's relief.** Each is a test in the script, printed by the calibration task:

| Metric | Today | Observed | Target |
|---|---|---|---|
| Land T at sea level, along-row std, winter / annual | 3.4 / 0.9 K | 6.1 / 3.3 K | ≥ 5.0 / ≥ 2.0 K |
| Same, η² winter / annual | 0.974 / 0.996 | 0.942 / 0.968 | ≤ 0.960 / ≤ 0.985 |
| Monthly T over land, RMSE vs observed | 6.6 K | — | ≤ 5.0 K |
| Winter land T on the 2.5° grid (block means): RMSE / along-row std / η² | 7.2 K / 3.3 K / 0.975 (prototype run L: 4.9 / 5.4 / 0.949) | — / 6.0 K / 0.944 | ≤ 5.0 K / ≥ 5.0 K / ≤ 0.955 |
| ln annual P over land, RMSE / η² | 1.11 / 0.75 | — / 0.41 | ≤ 0.85 / ≤ 0.55 |
| Köppen vs Beck: exact / main group | 34.6 % / 64.9 % | — | ≥ 38 % / ≥ 68 % with `belts` rain; ≥ 42 % / ≥ 70 % for `budget` |
| Southern winter ice edge, std over longitudes | 0.0° | several degrees | ≥ 2° |
| Brest − Vladivostok January contrast | 6.8 K | 20.1 K | ≥ 12 K |
| Yakutsk, January | −16.5 °C | −39 °C | ≤ −28 °C |
| Mumbai / Bangladesh annual P | 531 / 278 mm | 3,096 / 2,194 | ≥ 1,500 / ≥ 1,200 mm |
| Sahara / Atacama annual P | 265 / 154 mm | 11 / 16 | ≤ 80 / ≤ 80 mm |
| Reference cities in the right main group (`climate-dem` list) | 19 / 24 | — | ≥ 21 / 24 |

The temperature, sea-ice and contrast rows apply to the first per-cell default, which keeps today's
rain on the new temperatures (`rain_model = belts`). The rain rows are the bar the moisture budget
must clear to become the default (§3.8): ln P, Mumbai and Bangladesh, and the Sahara and Atacama.
Under `belts` they are reported but are not gates.

### 7.2 Zonal means against the 1-D model

- **Uncorrected.** Both models run on the same planet, with the 1-D model taking the zonal land
  fractions. The zonal annual means are printed. This is a diagnostic: the prototype's RMS was
  0.18 K for plain diffusion and 1.5–4.8 K with the full model.
- **Corrected.** After the F_b fit:
  - |ΔT| < 0.3 K in every band for land and for sea;
  - the clock's global mean within 0.2 K of the 2-D model's;
  - |F_b| ≤ 15 W/m².

  A test (T6).
- **Earth.** The 2-D global mean stays 15 ± 0.5 °C at 280 ppm, as the 1-D model is today: a0 is
  re-fitted if needed.
  - *WP-14a:* Earth's own pre-industrial mean is cooler. Berkeley Earth gives 14.10 °C for
    1951–1980 (air temperature over sea ice; 14.70 °C with the water below the ice) and an
    1850–1900 anomaly of −0.31 K against that base, so 13.8 °C (14.4 °C) for 1850–1900 (Rohde &
    Hausfather 2020; the data file's header and monthly anomalies, read and averaged WP-14a). The
    15 °C is the mean of North, Cahalan & Coakley 1981's models with their northern-hemisphere
    coefficients (15.0 °C for an ice-free planet, p. 93; 14.97 °C for the diffusive model,
    p. 97). Whether T9 targets 15 °C (today's
    calibration) or
    Earth's 13.8–14.4 °C is a question for Ben (STATUS "Pending questions", WP-14a); the target
    stays 15 ± 0.5 °C until he answers.

### 7.3 Closure

- **Energy.** |ASR − OLR| < 0.05 W/m² at every converged snapshot, in the run's notes. The heat
  test (radiation off) holds to 10⁻⁹ per year.
- **Water.** |ΣE − ΣP − ΣS_oro| / ΣE < 10⁻⁴ every month.
- **Ocean.** The gyre transports are non-divergent to round-off. The conveyor's Σ sources equal
  Σ sinks. Σ 𝒪 = 0 to 10⁻⁹.

### 7.4 When it counts as stripe-free

Stripe-free means **Earth-like zonal variety**, not zero banding: Earth is 96.8 % latitude in its
annual land temperature. On the two real runs (`earthlike_quick_16k`, scattered and supercontinent,
500 Myr) and on Earth's relief:

1. The along-row std of land sea-level T in winter is ≥ 4.5 K on the real runs (today 3.1 K) and
   ≥ 5.0 K on Earth's relief.
2. Sea ice over the sea has η² ≤ 0.90 (today 0.998), and the southern and northern ice edges each
   have a std over longitudes ≥ 1.5° (today 0.23°).
3. The polar continent of the scattered run (17 % of the land) has a winter along-row std ≥ 4 K
   (today 2.2 K) and at least three Köppen classes over ≥ 5 % of its land each (today two: ET,
   EF).
4. **Visual check** on the review page (T10). In the satellite view, Köppen and January temperature
   of both runs and Earth's relief, side by side with today's, no class boundary or ice edge follows
   one grid row for more than 15° of longitude. The script reports the longest run of a boundary
   along a single row.
5. The workbench's satellite view looks right without the band blur.

### 7.5 Calibration procedure (T9)

The order is chosen so each knob is tuned on what it controls most, on Earth's relief:

1. **Global and zonal energy:** a₀ and D_ocean/D_eq/D_pole against the 1-D model's Earth zonal
   means and 15 °C.
2. **Ocean:**
   - γ_o and Ψ against the January sea surface of the North Atlantic and North Pacific;
   - the northern ocean heat transport (peak 1.5–2 PW at 15–25°N);
   - the March and September ice edges.
3. **Land temperature pattern:** β_T, `zonal_diffusivity_ratio` and I_max against the January
   along-row std, the Brest–Vladivostok contrast and Yakutsk, keeping the July RMSE from worsening.
4. **Rain:** κ_p, τ_P, a and the stability factor against the monsoon points, the desert points and
   ln P RMSE; then SB's `oro_hours_per_day`.
5. **Re-measure the Earth references** `KERNEL_EARTH` and `ice_reference_km3`, and re-run the real
   planets: heights, glaciated share, deposit counts, CO₂ at the present (§5.4–5.5).

Each step records its numbers in STATUS.md's build log. Tolerances are the implementer's (CLAUDE.md:
"calibrated tolerances are yours"); the targets in 7.1 are Ben's to approve with this document.

---

## 8. Implementation plan

Order and parallelism:

```
T1 coarse grid ──┬─ T2 2-D EBM ───┐
                 ├─ T3 ocean ─────┼─ T5 snapshot integration ─ T6 clock coupling ─ T9 calibration ─ T10 downstream checks
                 └─ T4 moisture ──┘        │                                        │
                                           ├─ T7 satellite/keyframes (parallel)     └─ T11 docs (alongside)
                                           └─ T8 editor settings (parallel)
```

All tasks start after the ice branch lands on master. Each runs the cargo gates on the Mac:
- `cargo test --workspace`, clippy `-D warnings`, fmt, deny;
- `./gradlew build test` where Java is touched.

GameTests run once per merge batch by the coordinator (2026-09-30 rule).

| # | Task | Model | Acceptance (all through `tools/remote/mac.sh`) |
|---|---|---|---|
| T1 | **Coarse grid** `climate/coarse.rs`: move `sheet::Grid` here; add unit-sphere areas, face geometry, masked bilinear refine, coarsen with a pick mask | **Opus** (shared with the ice sheet, data layout) | Σ cell areas = 4π to 10⁻¹²; coarsen then refine of a constant is that constant; masked refine never reads the other medium unless none of 4 has it; the sheet's tests unchanged and green |
| T2 | **2-D EBM** `climate/ebm2d.rs`: tiles, albedo, surface-dependent D, ADI, zonal advection, slab diffusion, sea ice, spin-up and warm start, monthly outputs | **Opus** | Radiation off: heat conserved to 10⁻⁹ over a year; a zonally uniform planet equals `ebm.rs` band by band to 10⁻⁶; Earth's land at 280 ppm: global mean 15 ± 0.5 °C, net TOA < 0.05 W/m²; same result on 1 and 3 threads; ≤ 0.15 s per model year on one core at 128 × 64 |
| T3 | **Ocean** `climate/ocean2d.rs`: Stommel ψ on corners (Drake and 75° rules), conveyor (basin finder, width rule, potential flow), sub-stepped upwind transport | **Opus** | Rectangular NH basin: clockwise gyre, western-boundary current ≤ 2 cells wide and ≥ 3× the interior speed; face transports non-divergent to round-off; Σ 𝒪 = 0 to 10⁻⁹; Earth's coastlines: sinking basin = North Atlantic, ψ 10–40 Sv |
| T4 | **Moisture** `climate/moisture.rs`: belt + ITCZ + thermal winds, evaporation, bucket, Bretherton precipitation with stability, line-relaxation solve, SB sink hook, lowland W_sat | **Opus** | Water closes to 10⁻⁴ every month; W ≥ 0; aquaplanet: P max at the ITCZ, minimum at 25–30°; a 10-K-warm summer continent gets onshore inflow and ≥ 2× its winter rain; ≤ 1 s for 12 months × 3 bucket passes at 128 × 64 on one core. Not the default yet (§3.8) |
| T5 | **Snapshot integration**: `snapshot::compute` runs T2→T3 + downscaling when `climate.model = per_cell`; rain from `rain_model` (`belts` fed by the 2-D ITCZ and sea-tile ΔSST, or T4's budget + SB coupling); `banded` keeps today's code; new fields `sea_ice_{max,min}`; `climate-dem --dump` | Sonnet (well specified once T2–T4 exist) | `per_cell` (both rain models) and `banded` pass the existing climate tests (thresholds of per-cell metrics in T9); coupled pipeline resumes identically on 1 vs 3 threads; baked layers identical in *names and descriptors* to today's (FORMAT unchanged); `ColumnParityTest`/`ClimateParityTest` green |
| T6 | **Clock coupling**: F_b fit, pattern scaling of temperature, precipitation and sea ice between snapshots, triggered snapshots, the sheet's per-cell ELA and s_g | **Opus** (snowball physics) | After a snapshot the 1-D model's band means match the 2-D zonal means to 0.3 K; the snowball scenario test still snowballs and escapes, and the 2-D model freezes at the onset CO₂; a triggered snapshot fires at the onset and the escape; checkpoint resume identical |
| T7 | **Satellite view and keyframes**: drop the band blurs, draw per-cell ice, add `sea_ice_{max,min}` layers and ramps | Sonnet | Viewer bundle shows the new layers; `satellite.rs` tests updated; no row-aligned stripes in the rendered present-day frame (the T10 script's check) |
| T8 | **Editor settings** (§5.7): `editor_fields.json`, lang, `EditorFields`, `AdvancedSettingsTest`, presets synced, share-code mapping of `rain_reach` | Sonnet | Gradle tests green; every new key round-trips; presets equal the defaults; the workbench shows the fields |
| T9 | **Calibration** (§7.5) on Earth's relief + the two real runs; `tools/validation/climate_vs_obs.py`; re-measure `KERNEL_EARTH`, `ice_reference_km3`; costs on a quiet Mac; flip `rain_model` to `budget` when §3.8's bar is met | **Opus** (many coupled knobs) | Every §7.1 target met or explained in STATUS with Ben told; §7.2–7.4 met; measured seconds per snapshot and per run recorded; the rain-model decision recorded with its numbers |
| T10 | **Downstream checks**: P1-09 census and `climateConsistent` floors (coordinator batch), P1-17 heights, deposit counts, review page (private spacesheep) | Sonnet | Floors re-based by the documented rule or the climate fixed; heights within P1-17's targets; review page published and linked from the hub |
| T11 | **Docs**: plan §8.3 "as built", FORMAT.md note that the climate layers are unchanged and `sea_ice_{max,min}` are pipeline fields, NOTICE.md (no new ported code expected; equations cited in headers), EVENTS.md (`ocean_overturning_shift`) | Sonnet | Docs match the code |

T2, T3 and T4 can run in parallel once T1 lands. Agree the slab interface first:
`Ocean::step(&mut tm, &mut ice, dt)`, plus the monthly temperature arrays T4 reads. T7 and T8 can
run in parallel with T6. T9 iterates with T2–T4 owners if a target needs a model change.

---

## 9. References and licences

The project is LGPL-3.0-or-later.
- **Code** may be ported only from MIT, BSD, Apache-2.0 or CC BY sources, with NOTICE.md entries.
- **GPL code** is never ported or copied. GPL tools run only as unshipped oracles in
  `tools/validation/`.
- **Equations** from papers are free to implement.

This design ports no new code. Every new piece is implemented from equations.

| Source | What we take | Licence (checked 2026-09-30) |
|---|---|---|
| North, Mengel & Short 1983, JGR 88(C11) 6576, doi:10.1029/JC088iC11p06576 | seasonal EBM with explicit continents | paper; equations only |
| Zhuang, North & Stevens 2017, SoftwareX 6, 198 (2-D EBM, full multigrid) | lineage, constants | repository licence not established; equations only |
| Ziegler & Rehfeld 2021, GMD 14, 2843 (TransEBM; Table 1 constants) | constants, 128 × 64 grid precedent | paper CC BY 4.0; TransEBM code GPL-3 (oracle only) |
| North & Kim 2017, *Energy Balance Climate Models*, Wiley | 2-D EBM methods (ch. 8) | book; equations only |
| North, Cahalan & Coakley 1981, Rev. Geophys. 19, 91 | A, B, D, P₂ albedo (in use) | paper |
| climlab (Rose 2018, JOSS 3, 659) | today's EBM port | MIT (in NOTICE.md) |
| fastscape-lem/orographic-precipitation (Smith & Barstad 2004, JAS 61, 1377) | SB, today's port, unchanged | MIT (in NOTICE.md) |
| Stommel 1948, Trans. AGU 29, 202 | gyre equation | paper |
| Rose & Marshall 2009, JAS 66, 2828 | ocean heat transport in EBMs (alternative closure) | paper |
| Ferreira et al. 2018, Annu. Rev. Earth Planet. Sci. 46, 327 ("Atlantic–Pacific asymmetry in deep water formation") † | basin width selects the sinking basin | paper |
| Trenberth & Caron 2001, J. Climate 14, 3433 | heat transport targets (read WP-14a: Fig. 7, ocean about 2 PW near 15–20°N; atmosphere 5.0 PW at 43°N) | paper |
| Lindzen & Nigam 1987, JAS 44, 2418 † (not obtainable in WP-14a; read through Back & Bretherton 2009, J. Climate 22, 4182) | linear frictional surface wind from temperature gradients | paper |
| Dommenget & Flöter 2011, Clim. Dyn. 37, 2143; Stassen, Dommenget & Loveday 2019, GMD 12, 425 (GREB) | bulk evaporation form, precipitation-on-RH-and-convergence idea | paper CC BY; **code has no licence** (github.com/christianstassen/greb-official: no LICENSE file; Zenodo "Other (Open)"); equations only |
| Fanning & Weaver 1996, JGR 101(D10), 15111; Weaver et al. 2001 (UVic EMBM) | energy–moisture balance atmosphere precedent | papers; code licence unverifiable (site unreachable, forks unlicensed); equations only |
| Bretherton, Peters & Back 2004, J. Climate 17, 1517 (read WP-14a) | P against column relative humidity (eq. 2, the monthly fit) | paper |
| Manabe 1969, Mon. Weather Rev. 97, 739 (read WP-14a) | soil bucket (eqs. 18 and 20, p. 743) | paper |
| Siler, Roe & Armour 2018, J. Climate 31, 7481; Bischoff & Schneider 2014, J. Climate 27, 4937 | moist EBM and energy-flux equator (later option, Q10) | papers |
| Wagner & Eisenman 2015, J. Climate 28, 3998 (read WP-14a) | sea ice in EBMs (Table 1: k = 2 W/m/K, pure ice) | paper; code (eisenman-group) has no licence; equations only |
| Klein & Hartmann 1993, J. Climate 6, 1587 † | low cloud vs stability (follow-up, Q9) | paper |
| Serreze, Kahl & Schnell 1992, J. Climate 5, 615 † (not obtainable in WP-14a; the inversion size checked against NCEP/NCAR reanalysis instead, §2.9) | Arctic winter inversions | paper |
| PlaHab (Ramirez 2024, PSJ 5:2, doi:10.3847/PSJ/ad0729) | zonal/meridional diffusivity ratio precedent (paper read WP-14a) | data and code CC BY 4.0 on UCF STARS (code not inspected, nothing taken) |
| **Not used:** ExoPlaSim (GPL-2.0), PlaSim (treated as GPL), Isca (GPL-3.0), CLIMBER-X (GPL-3.0), OpenLEM/LFPM (GPL-3.0-or-later), SPEEDY (non-commercial grant) | — | do not port; oracles at most |

† Cited from memory, not re-read in this session. The implementing tasks verify the page numbers
before code headers cite them. WP-14a (2026-10-02) read the unmarked ones it could obtain; what it
could not read is listed in `docs/science/per-cell-climate.md`, "Open questions".

**Validation data** (never committed or shipped):
- Beck et al. 2018, Sci. Data 5, 180214, Köppen maps: CC BY 4.0, figshare 6396959.
- Beck et al. 2023, Sci. Data 10, 724, monthly climatologies 0.5°: CC BY 4.0, figshare 21789074.
- GMT `earth_relief` (SRTM15+, Tozer et al. 2019), via the GMT data server.
- ETOPO 2022 (NOAA, US-government work) as an alternative relief.
- HadISST (Open Government Licence).
- GPCP v2.3 (NOAA).
- **Not:** WorldClim (non-commercial, no redistribution) and Peel et al. 2007's map files
  (CC BY-NC-SA). Peel's classification *rules* are formulae and are in use.

---

## 10. Open questions for Ben

| # | Question | Options | Recommended |
|---|---|---|---|
| Q1 | Approve this design to build in Phase 1 after the ice branch lands? | (a) Yes as written; (b) yes, without the overturning conveyor (gyres only; in the prototype the winter seas off Europe-like coasts were 4–5 °C colder without it); (c) yes, but temperature and sea ice only, rain stays on today's belts permanently | **(a)** |
| Q2 | How does rain move to the moisture budget? | (a) Staged: today's belt rain runs on the per-cell temperatures first; the budget becomes the default once it beats the belts on Earth's relief (§3.8); (b) the budget is the default from the start, calibrated in T9; (c) keep the belts, drop the budget | **(a)**: the prototype's rain is not yet better than today's (ln P RMSE 1.7–2.0 against 1.05) |
| Q3 | Keep the old banded climate as a setting? | (a) Keep `climate.model = banded` for comparison and old presets; (b) remove it once per-cell is calibrated | **(a)** until Phase 1 exit, then decide |
| Q4 | Climate grid default | (a) 128 × 64 (2.8°); (b) 90 × 45 (4°, about half the cost); (c) 180 × 90 (2°, about 2×) | **(a)**, with (b) as the fallback if P1-17's benchmark gate needs it |
| Q5 | How is the overturning's sinking sea chosen? | (a) The narrowest polar-connected basin (geometry; picks the North Atlantic on Earth); (b) the basin of largest evaporation minus rain (picked the North Pacific in the prototype); (c) a setting with a map pick in the editor | **(a)** |
| Q6 | Re-run the 2-D climate inside the 3-Myr ice-age window? | (a) No, pattern scaling only (default 0); (b) every 100 kyr (+10–30 s) so ice sheets cool their own region | **(a)** now; revisit if LGM ice looks banded |
| Q7 | Show winds in the workbench? | (a) Add a monthly wind-arrow layer (coarse grid); (b) no | **(a)**, cheap and it explains monsoons |
| Q8 | Seasonal sea ice in the game | (a) Keep only the biome's frozen ocean (annual mean, today); (b) bake winter-maximum sea ice so a pass lays ice blocks (format 0.3) | **(a)** for Phase 1 |
| Q9 | Extra ocean–cloud physics: stratocumulus brightening over cold seas, coastal upwelling | (a) Later, if calibration misses the cold-coast deserts; (b) now | **(a)** |
| Q10 | Latent heat in the energy balance (a moist EBM: rain heats the air where it falls) | (a) No: dry D includes latent transport, as today; (b) yes, recalibrate D | **(a)** for v1 |
| Q11 | Runoff for weathering and the surface | (a) Keep Fu's Budyko curve on the new P and T (one runoff for climate and surface); (b) use the moisture budget's bucket runoff | **(a)** |

## Appendix A. How the measurements were made

All runs were on the M6 Mac, through `tools/remote/mac.sh` for the build and plain ssh for the
scratch tools. Everything sits under `/Volumes/Graphide/ban/deeptime-runs/p1-26-design/`, which is
deleted after this task. The Mac was heavily shared: load average 70–95 on 12 cores, and it
rebooted once mid-task. So wall times are inflated and CPU (user) times are quoted.

1. `cargo build --release -p deeptime-cli`, then
   `deeptime stage climate --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --out …/q16-scattered.dtcp`.
   It took 864 s; the timings are in §6.1.
2. `deeptime dump-fields --in …/q16-scattered.dtcp --fields surface/elevation_m,climate/{t_mean_c,precip_mm,koppen,t_warm_c,t_cold_c,aridity,glacier,sea_ice,ela_m,ice_thickness_m},climate/month/{t_sl_c,p_mm}_01..12`.
3. Earth's relief: GMT `earth_relief_30m_p.grd`, turned into 720 × 360 f32 (north first, from 0°E).
   A scratch crate (`p126-scratch dem`) reproduces `climate-dem` (six snapshots with the ice
   feedback) with `monthly: true` and dumps every field.
4. Observations: Beck 2018 `Beck_KG_V1_present_0p5.tif`; Beck 2023
   `climate_data_0p5.zip:1961_1990/ensemble_mean_0p5.nc`.
5. Metrics: a scratch numpy script (`analyse.py`, area-weighted η², along-row std, ice edges, Köppen
   statistics, agreement, RMSEs, point values). Quick-look PNGs were made with a second script.
6. The observed sea-surface temperatures quoted for coastal points ("about 11 °C off Brest") are
   round climatological values for orientation, not part of the scored data.

## Appendix B. The prototype (scratch, not committed)

`p126-scratch proto` is about 1,100 lines of Rust outside the repository. It depends on
`deeptime-core` for insolation, the belt winds and the EBM constants. It implements, behind
environment switches:
- §2 on Earth's relief at 2.5° (144 × 72; 120 × 60 also run): tiles, ADI diffusion with an optional
  surface-dependent D and zonal factor, zonal shift advection on the surface regimes or the
  low-level profile, optional mountain blocking, slab diffusion, Stommel gyres (ψ on corners, Drake
  and 75° rules), the conveyor (narrow-basin or E − P rule, potential flow), sea ice, and the winter
  inversion as a diagnostic;
- §3's monthly moisture budget: belt + ITCZ + thermal winds, bulk evaporation with a fixed land
  factor or the soil bucket, Bretherton precipitation with cold-sea stability, and line
  relaxation.

It has no SB coupling and no L1 downscaling: temperatures are the coarse land tile, so coasts finer
than 2.5° are not scored. It was not calibrated. Its numbers show what each ingredient does, not the
final skill.

**Winter land temperature on Earth's relief**, 2.5° grid. Winter is January north and July south.
"Obs" is the observation block-averaged to the same grid; numbers in brackets include the winter
inversion. Today's model, block-averaged the same way, scores 3.3 K / 0.975 / 7.2 K (summer RMSE
6.1 K).

| Run | Ingredients | Along-row std (obs 6.0 K) | η² (obs 0.944) | RMSE vs obs | Jan land tile: Moscow / Yakutsk / Winnipeg, °C (obs −9 / −39 / −17) |
|---|---|---|---|---|---|
| baseline | isotropic D, surface-regime advection β_T 0.15, gyres γ_o 0.4, D_ocean 0.3 | 3.2 K | 0.978 | 7.4 K | −17.0 / −16.9 / −6.3 |
| dz 0.1 | baseline, zonal D × 0.1 | 4.6 | 0.957 | 7.9 | −19.6 / −17.6 / −9.4 |
| A | surface-dependent D (NMS83 lineage) | 3.1 (4.4) | 0.985 (0.977) | 6.3 (8.0) | −20.5 / −21.8 (−33.3) / −8.9 |
| C | A + D_ocean 0.8, γ_o 1.0, β_T 0.3 | 3.7 (4.9) | 0.972 (0.962) | 6.9 (6.6) | −17.7 / −17.4 (−28.9) / −4.0 |
| D | C + zonal D × 0.3, C_l 0.45 m | 5.1 (6.3) | 0.938 (0.931) | 10.4 (10.4) | −22.5 / −17.5 (−29.0) / −10.7 |
| E | C + ITCZ, stability (rain only) | 3.7 (4.9) | 0.972 (0.962) | 6.9 (6.6) | −17.7 / −17.4 / −4.0 |
| F | E + **conveyor 15 Sv** (sinks in the North Atlantic) | 4.0 (5.3) | 0.963 (0.953) | 6.9 (6.1) | −15.3 / −15.8 (−27.3) / −2.9 |
| J | F + **low-level advection profile** | 4.1 (5.5) | 0.962 (0.949) | 6.0 (5.3) | −10.3 / −22.1 (−33.6) / −4.2 |
| **L** | J with β_T 0.5, zonal D × 0.5 | 4.1 (**5.4**) | 0.961 (**0.949**) | 5.9 (**4.9**) | **−7.2 / −22.7 (−34.2)** / −1.6 |
| M | L + mountain blocking + bucket | 5.2 (6.6) | 0.936 (0.927) | 6.6 (5.9) | −7.2 / −22.9 (−34.4) / −1.5 |

**Other figures:**
- Global means were 14.9–15.5 °C in every run, against 15.2 °C for the 1-D model.
- Net TOA was ≤ 0.010 W/m², and water closed to 2–4·10⁻⁶.
- The Stommel gyres reached 19–22 Sv.
- Northern ocean heat transport peaked at 0.3–1.0 PW without the conveyor, 1.5–1.6 PW with 15 Sv
  and 2.2 PW with 25 Sv.
- Summer RMSE was 4.9–5.6 K in every run but D (6.4 K), against today's 6.1 K.

The precipitation runs are in §3.7.

**What is still wrong in the best run (L):**
- **Winnipeg is too warm**: −2 °C against −17 °C observed. Pacific air crosses the Rockies, and no
  Arctic outflow reaches it. Mountain blocking (run M) and the stationary-wave trough it lacks are
  calibration and follow-up items.
- **Coastal points are cold on the land tile.** Brest's land tile is −2.5 °C against 7.2 °C
  observed, while its sea tile is 9.1 °C. L1's coast blend (§2.9) fixes points like these; the
  prototype does not downscale.

**Lessons that shaped the design:**
1. **A diffusive 2-D EBM alone keeps temperatures banded.** Winter zonal std stayed at about 3 K,
   like today's model. Diffusion and the land–sea exchange smooth zonal contrasts as well as
   meridional ones.
2. **Zonal variety can be raised with a smaller zonal D** (run D reaches Earth's 6 K), **but it lands
   in the wrong places** (RMSE 10 K): western Europe was frozen. Location needs the ocean and the
   wind.
3. **The conveyor puts the warmth where it belongs.** With sinking chosen by basin width (the
   E − P rule picked the North Pacific), the January sea off Brest went from 4.3 to 9.2 °C, and
   ocean heat transport reached Earth's order (1.6 PW).
4. **The westerly heat transport must use a low-level profile reaching 65–70°.** The surface regimes
   end the westerlies at 60°, and shifted south in January they advected heat westward across
   Siberia, so Moscow and Yakutsk came out alike. The profile takes Moscow from −15 to −7 °C and
   Yakutsk from −16 to −23 °C.
5. **Interior winter cold needs the inversion term,** gated by snow, darkness and continentality. It
   takes Yakutsk to −34 °C and leaves Moscow alone once Moscow is warm enough.
6. **The moisture budget is cheap and closes, but rain is the risk.**
   - It gives Earth-like variety, the ITCZ excursion gives monsoon rain, and the bucket gives the big
     deserts.
   - Its overall error is still above today's belts, and the bucket's dry state can lock in where
     the moisture supply is weak.
   - Hence §3.8's switch.
7. **Gyres must stop at 75° and in open channels.** Otherwise tiny polar cells force about 370
   ocean sub-steps per step, and the Southern Ocean grows a spurious 90-Sv gyre.
