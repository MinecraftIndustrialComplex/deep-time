# Upwind rain

Status of this page: `stage 2 (constants checked against sources on 2026-10-04)` for Hergarten &
Robl 2022's lengths and equations; the rest is marked in the ledger as it stands. Written on
2026-10-04 and 2026-10-05 for P1-26's T9 (Ben, decisions log 2026-10-04: "S&B + upwind moisture
budget (Recommended)"), calibrated under Ben's validation policy of 2026-10-05 (physics from
sources; Earth's own relief and CO₂ must give Earth's land rain, about 750 mm).

Ledger ids on this page: `CLM-190` to `CLM-200` (see [claims.md](claims.md)), with the rows it
shares: CLM-139 to CLM-145 (the winds, evaporation and saturation of the per-cell moisture
budget), CLM-178 (which rain runs), MOI-008, MOI-022, MOI-048 and MOI-063 (the circulation's rain),
HYD-029 and HYD-034 (potential evaporation and Fu's curve). The per-cell climate around it is in
[per-cell-climate.md](per-cell-climate.md), the banded climate and the belts in
[climate.md](climate.md).

## What you see in the world

With `climate.rain_model = upwind` a world's rain is the wind's: air that has crossed a continent
has rained most of its water out, so continental interiors downwind of the coasts are drier, wet
coasts are the ones the wind blows onto from the sea, and a mountain range takes its water out
of the air before the land behind it (Smith & Barstad's windward rain, now subtracted from the
air the wind carries on). Deserts under the subtropical highs' eastern flanks (an Atacama, a
Namib) stay dry because the air that reaches them came off the cold eastern edge of a sea basin.
The rain's fine pattern (the belts of latitude, the highs' flanks, the cold seas, mountains and
plateaus) is the same as with the belts; what changes is how much of it the air can still deliver
where it arrives.

The upwind rain is a setting (`climate.rain_model = upwind`); the default is still the belts
(CLM-178), because on Earth's relief the upwind rain meets Earth's land rain but not the design's
bar on the rain's pattern (below). Ben decides the default (STATUS, pending questions).

## The model in plain words

Every climate snapshot (every 10 million years of the planet's history, `climate/snapshot.rs`)
runs, after the per-cell energy balance has given every coarse cell its monthly land and sea
temperatures and sea ice:

1. **The circulation's rain on L1.** For every L1 cell and month: the belt rain of its belt
   latitude (the rain over an open ocean at that latitude, MOI-008), drier under the subtropical
   highs' eastern flanks and wetter under their western flanks (MOI-022), drier over sea colder
   than its row (MOI-063) and on high plateaus (MOI-048). This is the rain a column holding its
   reference water would give. Over land the flank and the sea temperature are the ones the air
   carries from the sea it evaporated from (step 4), not a march along the row.
2. **Its mean over each coarse cell** (the per-cell climate's grid, about 2.5 to 2.8 degrees),
   times the hydrological cycle's scaling with the global temperature (Held & Soden 2006), is the
   cell's reference rain P_ref.
3. **The budget** (`climate/upwind.rs`). Per coarse cell and month the wind carries two kinds of
   water, vapour and cloud water, as in Hergarten & Robl 2022's linear feedback precipitation
   model (LFPM):
   - the open sea evaporates by the bulk formula into the vapour;
   - vapour condenses into cloud water in proportion to the vapour, as in LFPM (the condensation
     time τ_c is set so that a column holding its reference water W_ref rains P_ref; a setting,
     `rain_rh_slope`, can make it grow faster with the column's humidity, off by default);
   - cloud water falls out as rain within τ_f, and a share β of it evaporates back into vapour;
   - land gives back to the air all it evaporates again (Fu's Budyko curve on the cell's last
     annual rain and potential evaporation; `rain_recycling` 1);
   - mountains take out what Smith & Barstad's model rains on their windward slopes (the
     snapshot's second pass);
   - the wind (the per-cell budget's: belt, three-cell and thermal winds, CLM-140 and CLM-141, at
     a share of their speed) carries both kinds, and an eddy diffusivity spreads them sideways.
   The month is solved to its steady state (the vapour lives about a week). Every face's flux
   leaves one cell and enters another, so the water closes exactly: what evaporates equals what
   rains plus what the mountains take.
4. **The sea's marks and the highs' sides.** The same flow carries a tracer of the sea surface the
   vapour evaporated from (its departure from its row); the land's own evaporation carries none,
   so it fades inland and sets step 1's stability over land. Which side of a subtropical high a
   place lies under (east: sinking, dry; west: wetter) is the place's own: the basin position of
   the sea it borders, carried over the land by a smoothing of about 1,000 km on the climate mesh
   (`flank_position`, CLM-192). The snapshot runs the budget twice, the second time with the first's
   marks and mountain sink.
5. **Back on L1.** Each L1 cell's rain is its own circulation rain (step 1) times its coarse cell's
   P/P_ref (bilinear between cells), plus Smith & Barstad's windward rain (or minus its lee drying)
   at the air's moisture W_v/W_ref. So a coast keeps its fine detail and an interior gets what the
   air brought.

The budget only sees cells and the faces between them (two cells, a length, the distance between
their centres and the face's normal in each cell's east and north), never rows or columns: a cube
or HEALPix climate grid only has to list its faces (`Mesh`).

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| CLM-178 | `rain_model` | `belts` (`upwind` a setting) | – | Ben, decisions log 2026-10-01 (Q2) and 2026-10-04 | `game-choice` |
| CLM-198 | the budget's equations (LFPM's two species on any mesh, steady per month) | see below | – | Hergarten & Robl 2022 eqs. 10–11, 42–43 | `verified` (form) |
| CLM-193 | cloud water's fallout time τ_f | 2,500 s (L_f = 25 km at 10 m/s) | s | Hergarten & Robl 2022 section 7 (L_f); 10 m/s the code's | `verified` (L_f) |
| CLM-194 | re-evaporation β | 18.05 (from L_c = L_f = 25 km, L_l = 500 km) | – | Hergarten & Robl 2022 section 7, eq. 27 | `verified` |
| CLM-200 | `rain_rh_slope` a: condensation's growth with the column's humidity | 0 (LFPM's linear condensation) | – | Hergarten & Robl 2022 eq. 10 | `verified` (form) |
| CLM-195 | `rain_recycling`: share of the land's evapotranspiration returned to the air | 1 | – | the water balance | `game-choice` (physical) |
| CLM-191 | `rain_wind_share`: the transport's share of the wind's speed | 0.3 | – | calibration to Earth's land rain | `calibrated` |
| CLM-190 | eddy diffusivity of the vapour and cloud water | 5·10⁵ | m²/s | none | `uncited` |
| CLM-196 | `sea_surface_rh`: the near-surface humidity over the sea at the reference column | 0.8 | – | about Earth's marine surface humidity | `from-memory` |
| CLM-192 | the highs' reach over land; fade below a quarter of sea | 1,000 km; 0.25 | m | about the Rossby radius | `from-memory` |
| CLM-197 | the land's recycled share before any rain is known | 0.6 | – | about Earth's land ET/P | `from-memory` |
| CLM-199 | the sea-surface departure carried as a tracer of the vapour | mixing ratio | °C | none | `game-choice` |
| MOI-022 | `west_flank_wet`: the wetting under a high's western side | 0.4 | – | none | `uncited` |
| CLM-142, CLM-144 | bulk evaporation; saturation column water | as the moisture budget | – | see per-cell-climate.md | as there |

The budget per coarse cell and month, in steady state:

```text
∇·(V W_v) − ∇·(K ∇W_v) = E_sea(W_v) + ε P + β k_c W_c − C(W_v) − k_s W_v
∇·(V W_c) − ∇·(K ∇W_c) = C(W_v) − β k_c W_c − W_c/τ_f
P = W_c/τ_f,   C(W) = k_c W e^(b (W/W_ref − 1)),   k_c = 1/(W_ref/P_ref − β τ_f)
E_sea = ρ_a C_E (|V| + u_t) q_s(T_sea) (1 − h_s W_v/W_ref) · open sea share
W_ref = r_ref W_sat(T̄, z_low)
```

W_v and W_c are the columns' vapour and cloud water (kg/m²), V the low-level wind times
`rain_wind_share`, K the eddy diffusivity, ε = `rain_recycling` × ET/P × the cell's land share (ET/P
from Fu's curve with ω 2.6 on the cell's annual rain and 60 mm/yr per °C potential evaporation),
k_s the mountains' sink per unit vapour, b = `rain_rh_slope` × r_ref (0: linear), h_s = `sea_surface_rh`.
The discretization is first-order upwind in flux form with a two-point diffusive flux, solved by
block Gauss–Seidel (a 2 × 2 Newton solve per cell) in the flow's upwind-to-downwind order.

On L1: P_L1 = P_circ(L1) · hydro · bilinear(P/P_ref) + Smith & Barstad at W_v/W_ref, with
P_circ = P_belt(φ′) · subsidence(a, φ′) · e^(0.07 ΔSST_air) · plateau(z) (MOI-062), with a the
place's side of the highs (CLM-192) and ΔSST_air the departure the vapour carries (CLM-199).

## Earth against the mod

Runs: `deeptime climate-dem --dem earth720.f32 --width 720 --preset <json> --dump` on Earth's relief
(GMT `earth_relief_30m_p`, resampled onto the planet's grid: 720 × 240 cells, a 144 × 48 climate
grid), 280 ppm, each scored by `tools/validation/climate_vs_obs.py --obs` against Beck et al.
2018's Köppen classes and Beck et al. 2023's 1961–1990 monthly temperature and rain (0.5°, CC BY
4.0, validation only, never in the repository); 2026-10-05, branch `worktree-agent-a16a907f3e5afef3d`
(build log 2026-10-05). The rain is compared on the land both call land.

| Quantity | Earth (Beck) | Belts (default) | Upwind, 2026-10-04 trial | Upwind, calibrated |
|---|---|---|---|---|
| Land rain, mm | 773 | 532 | 801 | **764** |
| The planet's mean rain (land and sea), mm (reference line) | about 990 (GPCP, from memory) | 764 | – | 876 |
| ln annual rain over land, RMSE (design bar: below the belts') | – | **1.135** | 1.190 | 1.217 |
| ln annual rain, bias | – | −0.05 | +0.11 | +0.15 |
| Köppen, exact class / main group (design bar: at least the belts') | – | **33.8 % / 63.7 %** | 30.6 % / 58.8 % | 28.6 % / 57.8 % |
| Köppen land shares A / B / C / D / E | 0.19 / 0.32 / 0.11 / 0.23 / 0.15 | 0.14 / 0.29 / 0.14 / 0.22 / 0.21 | 0.16 / 0.21 / 0.21 / 0.20 / 0.21 | 0.17 / 0.18 / 0.24 / 0.20 / 0.21 |
| Reference cities in the right main group (target 21 of 24); at the stations' heights | – | 19; 20 | 16; – | 16; 17 |
| Mumbai / Dhaka, mm (targets ≥ 1,500 / ≥ 1,200) | 3,096 / 2,194 | 623 / 245 | 849 / 185 | 942 / 295 |
| Sahara / Atacama, mm (targets ≤ 80) | 11 / 16 | 244 / 179 | 271 / 660 | 425 / 427 |
| Monthly land temperature, RMSE | – | 6.10 K | 5.93 K | 5.88 K |

Regional land rain, mm (Earth; belts; upwind calibrated): Sahara 25; 280; 507. Amazon 2,416;
1,072; 1,848. Congo 1,771; 1,077; 2,081. India 1,125; 413; 565. South-east China 1,530; 305; 365.
Western Europe 897; 1,076; 1,266. Siberia 430; 421; 116. Central Asia 239; 564; 700. Interior
Australia 262; 329; 427. Eastern USA 1,185; 432; 529. Interior western USA 339; 437; 718. Sahel
554; 864; 2,099.

The 2026-10-04 trial used `rain_recycling` 0.6 (four tenths of the land's evaporation lost) and a
humidity e-folding of 1; the defaults it started from (recycling 1, the budget's slope 11.4,
transport 0.5) gave 910 mm and an ln RMSE of 1.48. The calibration kept the water physical (all
evaporation returned, LFPM's linear condensation) and set the one free scale, the transport's
share of the wind, to Earth's land rain: 0.3 → 764 mm, 0.4 → 867, 0.5 → about 945.

What the table says. The upwind rain is the only model that gives Earth its rain (the belts are
31 % dry over land), and its deserts and monsoon coasts are the belts' errors in the other
direction: it is too wet in the Sahara, the Sahel, central Asia and the interior West, and too dry
where Earth's summer monsoons bring the sea's air in (India, China, the eastern USA) and in
Siberia. Both stem from the circulation's rain (P_ref), which neither model makes monsoonal, and
from the winds: in July the per-cell winds blow from the north-east over Bangladesh, because the
model's Tibet is cold (glaciated at Lhasa) instead of a summer heat source. The rain's pattern
misses the design's section 3.8 bar on both counts, so the belts stay the default.

Tried and dropped (Earth's relief, 2026-10-04 and 2026-10-05): the summer flow round the highs
(2 to 7 m/s poleward on a basin's western side; the eastern USA 350 → 379 mm, nothing else moved:
the rain there is limited by the circulation's rain, not by the vapour supply); Bretherton's
monthly e-folding 11.4 (Dhaka 24 mm: with the winds from the wrong side it dries the monsoon
coasts); a stronger thermal wind (κ 120 and 240 Pa/K: the Congo 5,400 mm); the ITCZ's largest
shift 16° instead of 22° (no change); a wetter western flank (`west_flank_wet` 2: south-east China
325 → 410 mm, main group +0.7 points).

**The surface lapse rate.** At 5.5 instead of 6.5 K/km (`lapse_c_per_km`; observed near-surface
lapse rates on mountain slopes are about 5 to 5.5 K/km, Rolland 2003 and Minder et al. 2010, only
their abstracts read) the cities in the right main group go 19 → 20 (belts) and 16 → 18 (upwind),
the Köppen main group 63.7 → 65.3 % (belts), Lhasa from EF to ET. It changes every block's
temperature in the game, so it is a question for Ben, not a change made here.

**12 planets** (`tools/review/p1-17-hyps.sh 500 "1 2 3" "scattered_continents supercontinent"`,
`earthlike_quick_16k`, 500 Myr to the deposits stage, each run admitted as a 2-thread simulation;
mean [min, max] of 6 runs, belts → upwind): land 0.184 [0.150, 0.199] → 0.166 [0.130, 0.207];
land median height 411 → 359 m, p90 1,788 → 1,678 m, above 2 km 0.083 → 0.063; global mean 14.7 →
14.5 °C; glaciated land 0.070 → 0.108; sea level 38 → 33 m; Köppen E 0.20 → 0.24, B 0.28 → 0.20.
The wetter continents erode faster: about a tenth less land and lower mountains.

**The winter spread.** The design's target (land temperature at sea level, January north and July
south, along-row standard deviation at 30–70°: at least 5.0 K, against Earth's "6.1 K") cannot be
reproduced from Beck et al. 2023 with the documented method: on Beck's own 0.5° grid it is 4.24 K,
on the model's grid 3.89 K (rows weighted by their span of latitude: 4.25 K), both brought to sea
level with 6.5 K/km on GMT's relief. The model gives 4.50–4.53 K on the same grid (4.91 K span
weighted), already more than Earth's; at the ground's height 6.86 K against Earth's 5.43 K. The
design's 6.1 K came from a scratch script that is not in the repository. Ben is asked to restate
the target relative to the observed value on the same grid (STATUS, pending questions).



## Sources

- `HergartenRobl2022`: Hergarten, S. and Robl, J. (2022), The linear feedback precipitation model
  (LFPM 1.0), Geosci. Model Dev. 15, 2063–2084. Read on 2026-10-04 (the article page, WebFetch):
  equations 10–11 (vapour and cloud water with condensation, fallout and re-evaporation β),
  equation 27 (L_l L_s = L_c L_f), equations 42–43 (evapotranspiration returns ε of the rain),
  section 7's lengths (L_c = L_f = 25 km, L_l = 500 km, L_d = 25 km). Written from the paper, not
  from OpenLEM's code (GPL).
- `Bretherton2004`: the rain's e-folding in column relative humidity (CLM-145), read in WP-14a.
- `SmithBarstad2004`: the mountain rain (`orographic.rs`), unchanged.
- `Fu1981` and `Zhang2004`: the Budyko curve (HYD-034), as the surface stage uses it.
- `Beck2018`, `Beck2023`: the observed Köppen classes and monthly temperature and rain the
  calibration compares with (validation only, never shipped).
- `Rolland2003`, `Minder2010`: observed near-surface lapse rates (abstracts only; the lapse-rate
  question, not used by the code).

## Simplifications and limits

- The circulation's rain (P_ref) comes from the belts' latitude table and the highs' flanks; the
  budget scales it by the vapour that arrives but cannot make a monsoon where the belts have none.
- One wind per month: the vapour's transport is the month's mean low-level wind at 0.3 of its
  speed, with an eddy diffusivity standing in for the weather that moves water across it.
- The budget runs on the coarse climate grid (2.5° on Earth); the L1 rain keeps its fine pattern
  from the circulation and the mountains, and takes the coarse cell's amount.

## Game choices

- `rain_wind_share` 0.3 is the one value set to Earth's land rain (CLM-191).
- The highs' reach over land (1,000 km) and its fade below a quarter of sea (CLM-192).

## Open questions

- Which rain is the default (CLM-178): the belts (better pattern, 31 % too dry) or the upwind
  rain (Earth's amount, worse pattern, a tenth less land on the planets).
- The winter-spread target's observed value (above).
- The surface lapse rate (above).
- The budget rain (`rain_model = budget`, not this page's) makes one cell of about 30 m a year on
  the idealised test planet (`climate::tests`).
