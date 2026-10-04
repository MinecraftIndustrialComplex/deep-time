# Upwind rain

Status of this page: `stage 2 (constants checked against sources on 2026-10-04)` for Hergarten &
Robl 2022's lengths; the rest is marked in the ledger as it stands. Written on 2026-10-04 for
P1-26's T9 (Ben, decisions log 2026-10-04: "S&B + upwind moisture budget (Recommended)").

Ledger ids on this page: `CLM-190` to `CLM-199` (see [claims.md](claims.md)), with the rows it
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

<!-- DEFAULT-PENDING -->

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
   - vapour condenses into cloud water, faster the more of its reference water W_ref the column
     holds (the condensation time τ_c is set so that a column holding W_ref rains P_ref);
   - cloud water falls out as rain within τ_f, and a share β of it evaporates back into vapour;
   - land gives back to the air the share of its rain that evaporates again (Fu's Budyko curve on
     the cell's last annual rain and potential evaporation);
   - mountains take out what Smith & Barstad's model rains on their windward slopes (the
     snapshot's second pass);
   - the wind (the per-cell budget's: belt, three-cell and thermal winds, CLM-140 and CLM-141, at
     a share of their speed) carries both kinds, and an eddy diffusivity spreads them sideways.
   The month is solved to its steady state (the vapour lives about a week). Every face's flux
   leaves one cell and enters another, so the water closes exactly: what evaporates equals what
   rains plus what the mountains take.
4. **The sea's marks.** The same flow carries two tracers of where the vapour evaporated: the sea
   basin position (east or west flank) and the sea surface's departure from its row. The land's
   own evaporation carries neither, so the marks fade inland. They set step 1's flank and stability
   over land; the snapshot runs the budget twice, the second time with the first's marks and
   mountain sink.
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
| CLM-178 | `rain_model` | <!-- DEFAULT-VALUE --> | – | Ben, decisions log 2026-10-01 (Q2) and 2026-10-04 | `game-choice` |
| CLM-198 | the budget's equations (LFPM's two species on any mesh, steady per month) | see below | – | Hergarten & Robl 2022 eqs. 10–11, 42–43 | `verified` (form) |
| CLM-193 | cloud water's fallout time τ_f | 2,500 s (L_f = 25 km at 10 m/s) | s | Hergarten & Robl 2022 section 7 (L_f); 10 m/s the code's | `verified` (L_f) |
| CLM-194 | re-evaporation β | 18.05 (from L_c = L_f = 25 km, L_l = 500 km) | – | Hergarten & Robl 2022 section 7, eq. 27 | `verified` |
| CLM-190 | eddy diffusivity of the vapour and cloud water | <!-- K --> | m²/s | none (LFPM's L_d = 25 km is for a mountain range) | `calibrated` |
| CLM-191 | `rain_wind_share`: the transport's share of the wind's speed | <!-- S --> | – | none | `calibrated` |
| CLM-195 | `rain_recycling`: share of the land's evapotranspiration returned to the air | <!-- R --> | – | none | `calibrated` |
| CLM-196 | `sea_surface_rh`: the near-surface humidity over the sea at the reference column | <!-- H --> | – | none | `calibrated` |
| CLM-197 | the land's recycled share before any rain is known | 0.6 | – | about Earth's land ET/P (from memory) | `from-memory` |
| CLM-199 | the sea's marks: basin position and sea-surface departure carried as tracers of the vapour | mixing ratios | – | none | `game-choice` |
| CLM-145 | how fast condensation grows with the column's water | <!-- A --> | – | Bretherton et al. 2004 (11.4) | <!-- A-STATUS --> |
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
k_s the mountains' sink per unit vapour, b = `precip_rh_slope` × r_ref, h_s = `sea_surface_rh`.
The discretization is first-order upwind in flux form with a two-point diffusive flux, solved by
block Gauss–Seidel (a 2 × 2 Newton solve per cell) in the flow's upwind-to-downwind order.

On L1: P_L1 = P_circ(L1) · hydro · bilinear(P/P_ref) + Smith & Barstad at W_v/W_ref, with
P_circ = P_belt(φ′) · subsidence(basin_air, φ′) · e^(0.07 ΔSST_air) · plateau(z) (MOI-062).

<!-- NUMBERS-PENDING -->

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
