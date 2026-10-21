# The plateau's warmth (the mass-elevation effect)

Status of this page: `stage 2 (constants checked against sources on 2026-10-21)`. The law is
fitted to station data read on 2026-10-21; its shape is read in full in the literature, its
numbers come from the fit (CLM-226 is `calibrated`, not `verified`).

Ledger ids on this page: `CLM-225` to `CLM-231` and `MOI-085` (see [claims.md](claims.md));
`CLM-220`, the Earth reference that asked for the term, was rewritten the same day.

## What you see in the world

Broad high ground is warmer than a mountain of the same height. A plateau 4 to 5 km up and
hundreds of kilometres wide has summers above freezing: grass, shrubs and tundra, not an ice cap.
On Earth's own relief the model's Tibetan Plateau above 3 km had a warmest month of −3.5 °C
(median) and was 64 % glacier before this term; now its warmest month is 6.0 °C and it holds no
ice sheet. The Altiplano and the Ethiopian Highlands warm by 2 to 4 K in their summer. A lone peak
gets little of this warmth and keeps its glaciers, and the great ice sheets keep their cold: ice
heats no air.

In the dark months the effect turns round. A high plateau in winter is a little colder than the
free air at its height, so the warmth is a summer one, and the ice line on a plateau moves up
while its winters stay hard.

On generated planets the same term cuts the white patches on 4 to 5 km plateaus at 25 to 45
degrees that Ben's run of 2026-10-21 showed ([plates.md](plates.md), "White patches, drowned
continents and coastal high ground"). It does not remove them all, and it does not change how much
high ground a planet has: that is the plateau-extent task's.

## The model in plain words

The term lives in the climate snapshot, where each land cell's twelve monthly temperatures are
made (`climate/snapshot.rs`, the land fields and the cells loop; the GPU path in `climate/gpu.rs`
does the same sums).

1. **The base height** (CLM-225). Take the ground's height above the sea, z. Set the sea to 0,
   and set to 0 every cell under at least 1 m of land ice from the previous step (CLM-228). Average
   that over a box of half-width 150 Earth km, run twice (about a 300 km wide bell). Cap the result
   at the cell's own height. This is Ẑ: the height of the broad, bare high ground the cell stands
   on. A peak takes its surroundings' base; a valley beside a plateau keeps its own, lower height;
   an ice sheet's interior and the sea get 0.
2. **The month's sunlight** (CLM-226, CLM-227). For each row and month, q̂ is the month's daily-mean
   sunlight at the top of the atmosphere (the same `insolation::monthly` the climate uses, with
   the planet's S₀ and obliquity) divided by the planet's mean, S₀/4. It is clamped to 0.60 to
   1.42, the range the law was fitted over.
3. **The warmth.** k = K (q̂ − Q₀), with K = 2.43·10⁻³ K/m and Q₀ = 0.792 (about 270 W/m² on
   Earth). The land cell's month temperature at sea level becomes the lowland's plus k·Ẑ. Then
   the lapse rate takes the ground's height off as before: t = t_sl − Γ z, Γ = 6.5 K/km (GLA-038).
   The orographic rain's moisture factor stays on the lowland air's temperature.
4. **Who sees it.** Everything downstream reads t or t_sl: the glaciers' equilibrium line (the
   warmest month), the Köppen class and aridity, the weathering kernel (the annual mean, so the
   CO₂ the planet settles at), the anomalies carried between snapshots (MOI-052), the bake's
   twelve monthly layers `climate/month/t_sl_c_NN`, which the Java side lapses to each block's
   height (MOI-085), and the L2 Köppen.

Ice and warmth feed each other: an iced plateau gets no warmth, so it can stay iced (see
"Simplifications and limits").

## Equations and constants

Per land cell c in row j and month m:

    Ẑ_c   = max(0, min(z_c, B²_r[z′]_c)),  z′ = z on bare land, 0 at sea and under ≥ 1 m of ice
    q̂_jm  = Q_TOA(φ_j, m) / (S₀/4)
    k_jm  = K · (clamp(q̂_jm, q_min, q_max) − Q₀)
    T_sl  = T_sl,lowland + k_jm · Ẑ_c
    T     = T_sl − Γ z_c

B²_r is a box average of half-width r cells applied twice (`surface::grid::box_blur`, rows
wrapping, columns mirrored at the poles); r is 150 Earth km in the grid's cells.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| CLM-225 | footprint of the base height (`MEE_FOOTPRINT_KM`) | 150 (two passes) | Earth km | swept against CLM-229 | `calibrated` |
| CLM-226 | warmth per metre and unit sunlight (`MEE_K_PER_M`); its zero (`MEE_Q0`) | 2.43·10⁻³; 0.792 | K/m; – | least squares on CLM-229 | `calibrated` |
| CLM-227 | the relative sunlight's clamp (`MEE_Q_MIN`, `MEE_Q_MAX`) | 0.60; 1.42 | – | the fitted range | `game-choice` |
| CLM-228 | ice that heats no air (`MEE_ICE_M`) | 1.0 | m | the per-cell climate's ice test | `game-choice` |
| CLM-229 | the stations' warmth over the free air (the fit's target) | table below | K | 1991–2020 normals; NCEP/NCAR R1 | `abstract-only` |
| CLM-230 | the law's shape in the literature | – | – | Han et al. 2018, 2018b, 2020; Yao & Zhang 2013; Pepin & Seidel 2005; others | `verified` |
| CLM-231 | the cold the lapse rate still leaves (not coded) | – | K/km | R1 against lowland normals | `abstract-only` |
| MOI-085 | what the bake's monthly sea-level temperatures hold | t_sl + k·Ẑ on land | °C | – | `game-choice` |

**The fit** (CLM-226). A sub-agent read each plateau station's 1991–2020 monthly normals and
computed the free air at the station's height over the nearby lowlands from NCEP/NCAR Reanalysis 1
(R1). The station minus the free air is the observed warmth; it ranges with the lowland column
chosen, and the fit takes the midpoint. Ẑ and q̂ come from Earth's relief on the model's grid
(720 × 240 cells of 55.6 km). Least squares on 28 station-months of 12 stations (the warmest
month, the annual mean, and the coldest month where it was read) gives K = 2.432·10⁻³ K/m and
Q₀ = 0.792 with an rms of 1.28 K. Lhasa (a valley station, warmer than the open plateau) and
Pikes Peak (a lone summit with low-confidence normals) are left out. Script:
`tools/validation/mee_station_fit.py`.

| Station | Height, m | Ẑ, m | Warmth over the free air, warmest month / year / coldest month, K | Fitted, warmest month, K |
|---|---|---|---|---|
| Nagqu (31.5° N) | 4,507 | 4,780 | 5.35 / 1.8 / −2.2 | 7.2 |
| Tingri (28.6° N) | 4,300 | 3,805 | 6.9 / 4.25 / 1.45 | 5.7 |
| Shiquanhe (32.5° N) | 4,279 | 4,633 | 9.15 / 2.2 / – | 7.0 |
| Madoi (34.9° N) | 4,272 | 4,131 | 4.2 / 3.15 / – | 6.3 |
| El Alto (16.5° S) | 4,058 | 2,970 | 4.85 / 3.4 / 0.9 | – |
| Puno (15.8° S) | 3,825 | 3,107 | 5.2 / 3.9 / 1.5 | – |
| Juliaca (15.5° S) | 3,838 | – | 4.9 / 3.05 / – | – |
| Oruro (18.0° S) | 3,735 | 3,059 | 5.8 / 3.35 / – | – |
| Leadville (39.3° N) | 3,095 | 2,278 | 1.5 / −1.1 / – | – |
| Climax (39.4° N) | 3,457 | – | 3.0 / −0.25 / – | – |
| Alamosa (37.4° N) | 2,299 | 2,220 | 2.2 / −0.75 / – | – |
| Addis Ababa (9.0° N) | 2,355 | 1,872 | 2.25 / 2.05 / – | – |

The fitted column is k·Ẑ for Tibet's July (q̂ 1.41 to 1.42). Mean residuals, model minus observed:
Tibet −0.10 K (10 station-months), the Altiplano −0.63 K (10), the Rockies +1.51 K (6), Addis Ababa
about −0.1 K (2). The footprint was swept with the fit redone each time: 56, 111, 150, 222, 333 and
500 km give rms 1.19, 1.20, 1.28, 1.37, 1.52 and 1.63 K.

## Earth against the mod

Run: `deeptime climate-dem` on Earth's relief (GMT `earth_relief_30m_p`, 720 × 360, on the model's
720 × 240 equal-area grid; the ice feedback over five snapshots), 280 ppm, defaults; scored with
`tools/validation/climate_vs_obs.py` and `tools/validation/mee_station_fit.py`; 2026-10-21. "Before"
is master at c790d1f3, "after" this branch.

| Quantity | Earth | Source of the Earth figure | Before | After |
|---|---|---|---|---|
| Global mean temperature, 280 ppm | 14.1 ± 0.4 °C (target) | CLM-138 (Ben, 2026-10-02) | 14.41 °C | 14.56 °C |
| Land mean temperature | – | – | 8.67 °C | 9.24 °C |
| Glaciated share of land | about 0.10 | params.rs comment (a round value) | 0.094 | 0.081 |
| Tibet (27–40° N, 75–103° E) above 3 km: glacier share | small (mountain glaciers only) | from memory | 0.643 | 0 |
| Same: warmest month, median of cells | Nagqu 10.0 °C at 4,507 m | CLM-229 | −3.5 °C | 6.0 °C |
| Same: annual mean, mean of cells | Nagqu −0.1 °C | CLM-229 | −13.1 °C | −8.7 °C |
| Greenland: glacier share of its land | about 0.8 | from memory | 0.497 | 0.483 |
| Antarctica: glacier share of its land | nearly all | from memory | 0.883 | 0.881 |
| Reference cities in the right main group, cells; station heights | 21 of 24 (target) | CLM-138's checks | 21; 22 | 21; 23 |
| Köppen land shares A / B / C / D / E | – | – | 0.17 / 0.16 / 0.24 / 0.23 / 0.20 | 0.18 / 0.17 / 0.23 / 0.25 / 0.18 |
| Weathering kernel (`KERNEL_EARTH`, CLM-061) | – | – | 5.039·10⁷ km²·m/yr | 5.147·10⁷ km²·m/yr |
| Along-row spread of land sea-level T, winter; annual | 3.23 K and 2.0 K (targets) | design §7.1 | 4.48; 1.92 K | 4.38; 1.73 K |
| Brest − Vladivostok, January; Yakutsk, January | 20.1 K; −39 °C | design §7.1 | 20.5 K; −28.2 °C | 19.8 K; −27.6 °C |

Warmest month at each station's own height (the model's cell temperature at sea level, lapsed by
6.5 K/km to the station), °C:

| Station | Observed | Before | After |
|---|---|---|---|
| Nagqu | 10.0 | −2.5 | 8.0 |
| Tingri | 12.6 | −0.9 | 7.8 |
| Shiquanhe | 15.0 | −0.9 | 8.9 |
| Madoi | 8.4 | −0.9 | 8.1 |
| El Alto | 9.9 | 3.1 | 7.0 |
| Puno | 11.4 | 4.1 | 8.2 |
| Oruro | 12.5 | 4.9 | 9.1 |
| Leadville | 12.8 | 3.0 | 6.5 |
| Climax | 11.9 | 0.7 | 4.2 |
| Alamosa | 18.5 | 9.1 | 12.5 |
| Addis Ababa | 18.8 | 14.0 | 16.4 |

Nagqu's January is −24.3 °C (−24.4 before) against −11.3 observed.

What the tables say. The plateau summers moved most of the way to Earth's, and Tibet's ice cap,
which Earth does not have, is gone. The rest of the gap is not this term's: the model's free air
over a plateau is too cold (CLM-231, below). The global mean warms by 0.15 K, nearly all from
Tibet's lost ice (about 1.1 % of the land at 30 to 38° N, now dark ground); it is now 0.06 K above
CLM-138's band (see "Open questions"). The glaciated share drops from 0.094 to 0.081 for the same
reason; the shortfall against Earth's 0.10 was there before and lies in Greenland, whose share is
0.48 against about 0.8. Lhasa's cell is now ET (tundra); at its station height it is Dsc, in the
D group like Earth's Dwb, which is the extra city at station heights.

### On generated planets

QUICK_PLANETS_PENDING

## Simplifications and limits

- **The law is fitted, not derived.** It is linear in the base height and in the month's
  sunlight, with one coefficient for every plateau. Han et al. 2020 find the distance to the
  massif's edge explains 23 % of the effect's variance inside Tibet; the model's blur gives the
  edge a lower Ẑ, which stands in for that only in part.
- **One baseline.** The fit uses the free air of R1 over nearby lowlands. Other studies use other
  baselines (Han et al. 2018: lowland stations extrapolated with NCEP lapse rates, MODIS land
  surface temperature on the plateau) and get an annual effect of 11.5 °C in Tibet's core, against
  about 2 K at Nagqu here. Han's MBE law (annual MEE = 0.0019 MBE + 0.584) is therefore not used.
- **The model's free air is too cold over plateaus** (CLM-231). Γ = 6.5 K/km everywhere, while R1's
  lapse from the lowland surface to 4.4 km is 5.3 to 6.1 K/km in summer and about 4.5 K/km over
  China's lowlands in the year. The plateau stations remain 0.3 (Madoi) to 6 K (Shiquanhe) too cold
  in summer, the Rockies about 6 K, and Nagqu 13 K too cold in January. A lapse rate that follows
  the season or the free air would move every mountain's ice line, not only the plateaus', and is
  left for its own task.
- **The Rockies get more warmth than observed** (+1.5 K mean residual): their stations sit in
  ranges and parks narrower than the footprint. They are still about 6 K too cold in summer
  because of the lapse rate.
- **Ice and warmth feed each other.** Ground under ice counts at 0, so a plateau that iced over in
  a cold spell gets no warmth until its ice has gone, and the climate must warm further to clear
  it than to keep it clear (hysteresis). The ice mask is the previous step's. Without the mask the
  law shrank Earth's ice sheets (Greenland 0.497 → 0.437, Antarctica 0.883 → 0.870), which is the
  wrong sign.
- **The bake carries the cell's warmth to every block of the cell** (MOI-085). The snapshot caps Ẑ
  at the cell's own height; the Java side lapses `t_sl_c` at each block's height but cannot cap,
  since Ẑ is not in the bake. A valley floor well below its cell's Ẑ is therefore a little warm
  (in Tibet's July, about 1.5 K per km below Ẑ). Real valleys on plateaus are warmer still (Lhasa,
  CLM-229), so this is left; capping would need a format change.
- **Between snapshots** the warmth is frozen at the last snapshot's (the carried anomalies,
  MOI-052), as everything else of the per-cell climate is.
- **Earth's orbit only in the fit.** q̂ uses the planet's own obliquity and S₀, so a planet with a
  larger tilt gets stronger summer warmth and stronger winter cooling; the clamp holds the law to
  the fitted range.

## Game choices

- **The clamp** (CLM-227): q̂ is held to 0.60 to 1.42, the span of the fitted station-months
  (Shiquanhe's January 0.65 to its July 1.416), so polar summers (q̂ about 1.55 in the solstice
  month at the pole) and polar nights do not extrapolate. It moves the fit by less than 0.01 K rms.
  Agent, build log 2026-10-21.
- **The ice threshold** (CLM-228): 1 m, the per-cell climate's own test for an ice-covered cell, so
  both see the same ice. Agent, build log 2026-10-21.
- **The footprint** (CLM-225) is 150 km although 56 km fits the stations 0.09 K better: 56 km is a
  single cell of Earth's grid, so a lone peak would get a plateau's warmth, which the literature
  says it does not (CLM-230). Agent, build log 2026-10-21.
- **No settings.** The constants are in code (`climate/lapse.rs`), not in the preset: they are
  Earth's physics, fitted once.

## Open questions

- **The global mean is 14.56 °C, 0.06 K above CLM-138's 14.1 ± 0.4.** The calibration (CLM-121,
  CLM-125, CLM-131, CLM-176) was tuned with Tibet's spurious ice cap reflecting sunlight. Retuning
  `atmosphere_diffusivity_scale` (CLM-176) would bring it back, at the cost of moving every
  planet's climate again and re-measuring the kernel; left for the coordinator and Ben.
- **The lapse rate** (CLM-231) is the larger remaining error on plateaus and mountains and is
  recommended as its own task.
- **The winter sign.** Against R1 Nagqu's January is 2.2 K colder than the free air (and Pepin &
  Seidel 2005 find surface sites cool against R1 in winter); against the Sichuan Basin's air at
  4,500 m Yao & Zhang 2013 find the plateau 1 to 2 °C warmer in the cold months. The law follows
  R1. Yanai, Li & Song 1992 also find the air just above the plateau warmer than the surrounding
  air at the same level in winter.
- **Monthly shape.** Han et al. 2018b put the interior's peak in August and its minimum in January
  and February; the law peaks with the sunlight (June or July). Their per-month slopes are in a
  figure only.
- **The station data** were read in Wikipedia's copies of the national normals; several station
  latitudes are from memory, and El Alto's table is labelled 1991–2020 but cites a 1961–1990
  source (CLM-229).

## Sources

- `Han2018MEE`, `Han2018bMEE`, `Han2020MEE` (J. Mt. Sci.): read in full (the publisher's PDFs).
- `Zhao2015Treeline`: read in full. `Han2012MBE`, `Zhao2014MEE`: abstracts.
- `YaoZhang2013` (PLoS ONE): read in full (the tables are images). `YaoZhang2015`: the abstract,
  verbatim.
- `He2016Andes`: the abstract (through OpenAlex) and a summary of the full text.
- `PepinSeidel2005`: read in full. `Yanai1992`: read in full. `Flohn1968`: read in full (no number
  taken). `Pan2021`: read in full (a cross-check only).
- `Kalnay1996`: the R1 long-term means used for the free air (data read, the paper not).
- `StationNormals2026`: the stations' 1991–2020 normals in Wikipedia's climate boxes (copies of
  the CMA's, SENAMHI's and NOAA's tables).
