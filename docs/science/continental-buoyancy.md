# A continental column's height from its whole buoyancy

Status of this page: `stage 2 (constants checked against sources on 2026-10-31)`. Built behind
`tectonics.column_buoyancy`, off by default; not yet run on the pool.

Ledger ids on this page: `PLT-830` to `PLT-836` (see [claims.md](claims.md)); it replaces, where it
is on, PLT-281's line and ISO-110's term.

## What you see in the world

Nothing yet: the switch is off in every preset. With it on, a continent's height follows its
lithosphere as well as its crust. Old, cold interiors with deep keels stand low for their crust:
40 km of crust under a craton stands about 600 m over the datum, where PLT-281's one line puts it at
1,220 m. Crust over a hot, thin lithosphere (a back-arc, a rift, new crust on young floor) stands
high: 35 km of it stands about 2 km up. The start's continents are thicker (35 to 42 km in place of
a uniform 35.1 km), so that they start at the same heights as before.

## The model in plain words

A column floats on the asthenosphere, the convecting mantle under the plates, which has no strength.
Its height over the datum is the height a free asthenosphere would have, plus how much lighter than
the asthenosphere the column is, metre by metre, down to the depth where every column is
asthenosphere (Lachenbruch & Morgan 1990, as Zoback & Mooney 2003 print it).

1. **The crust.** Each of the column's rock pools lies at its depth: the cover (sediment and
   sedimentary rock) on top, then the felsic crust, then the mafic pools at the bottom. Each pool has
   a rock's laboratory density at that depth (Christensen & Mooney 1995), expanded by the column's
   temperature there. Sediment has shale's porosity, filled with sea water. Mafic rock past the
   eclogite pressure weighs partly as eclogite (PLT-278 to PLT-280, as before).
2. **The mantle lithosphere**, from the Moho to the lithosphere's base L. It is mantle rock colder
   than the asthenosphere, so denser: it pulls the column down by its thermal contraction. The
   depleted keel under a craton (K, the thermal state's) is lighter by composition and compensates
   0.4 of its thermal excess (Kaban et al. 2003), not all of it.
3. **The temperature** is the thermal state's steady geotherm (ISO-138): 1,350 °C at L, the crust's
   own heat decaying with depth. L is what the heat-flow law reads (`thermal::sample_l_km`): never
   above the lid's floor, 1.25 times the crust (PLT-820), and for crust with no state yet its floor's
   (PLT-821).
4. **The free asthenosphere's height** comes from the ridge: GDH1's crest under 2,600 m of water
   (PLT-352) is 7 km of ocean crust (PLT-104) at the mantle's temperature. Balanced, the free
   asthenosphere stands 2,580 m under the datum.

The settling (`settle.rs`, ISO-078) moves every continental column to this height each plate step,
under the sea's water where it is drowned and with the surface's flexure, in place of PLT-281's line
plus ISO-110's thermal term. The collapse's thin sheet (PLT-414) reads the same height for its
potential energy. The start's continents take the thickness at which the law holds them at their
start heights (PLT-835).

## Equations and constants

The dry height over the datum:

  z = H_f + (1 / ρ_a) [ Σ_pools ∫ (ρ_a − ρ_p(z) (1 − α T(z))) dz − ρ_m0 α ∫_Moho^L (T_m − T) dz
      + 0.4 ρ_m0 α ∫_Moho^(Moho+K) (T_m − T) dz ]

with ρ_a = ρ_m0 (1 − α T_m) the asthenosphere's density, ρ_p(z) a pool's rock density at depth z at
0 °C, T(z) the geotherm, T_m 1,350 °C, L the lithosphere's base and K the keel's thickness, and
H_f = −2,600 m (ρ_a − ρ_w) / ρ_a − (1 / ρ_a) ∫_0^7km (ρ_a − ρ_ocean (1 − α T_m)) dz. The crust is
integrated by 8-point Gauss-Legendre per pool; the mantle lithosphere in closed form.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| PLT-830 | the mantle's thermal expansion α; its density at 0 °C ρ_m0 | 3.28e-5; 3,330 | 1/K; kg/m³ | Stein & Stein 1992, PSM (read) | `verified` |
| PLT-831 | the keel's compensation of its thermal excess | 0.4 | share | Kaban et al. 2003 (read) | `verified` |
| PLT-832 | the pools' rock densities at 5 km (metamorphic, felsic plutonic, felsic volcanic, mafic volcanic, mafic plutonic) and their increase with depth | 2,746, 2,662, 2,654, 2,878, 2,966; 1.3, 1.25, 1.25, 1.05, 1.05 per km | kg/m³ | Christensen & Mooney 1995, Table 4 (read) | `verified` |
| PLT-833 | sediment: shale's φ_0, c, grain density | 0.5, 5.0e-4, 2,720 | share, 1/m, kg/m³ | Angevine et al. 1990, Fig. 3.11 (secondary, read) | `verified` |
| PLT-834 | the start's column per 40 km: sediment, sedimentary, felsic plutonic, metamorphic, mafic plutonic | 0.1, 2.0, 9.9, 14.4, 13.6 | km | Hacker, Kelemen & Behn 2015, Fig. 1 (read) | `verified` |
| PLT-835 | the start's thickness, from its start height on the state's L and K | 34.8 to 41.9 | km | none: the lane's design | `game-choice` |
| PLT-836 | the law's form and where it replaces PLT-281; H_0 | off; 2.58 | bool; km | Zoback & Mooney 2003 (read) | `verified` |

The constants it reuses: T_m and the geotherm (ISO-138, PLT-467), the lid's floor (PLT-820,
PLT-821), GDH1's ridge (PLT-352), the ocean crust (PLT-104), the eclogite root (PLT-278 to PLT-280),
sea water (ISO-044), the start's L and K (ISO-111, PLT-463).

## Earth against the mod

The unit tests (`plates/buoyancy.rs`), against Earth's own columns; no pool run yet.

| Quantity | Earth | Source of the Earth figure | The law | Run |
|---|---|---|---|---|
| the free asthenosphere's height under sea level, H_0 | 2.78 ± 0.35 km | Zoback & Mooney 2003, eq. (3) (read) | 2.58 km | test |
| a half-space's subsidence from the same constants | 365 √t m (GDH1) | Stein & Stein 1992 (read) | 363 √t m | test |
| ocean floor 1, 10 and 30 Myr old (the law's 7 km column, the model's ocean L) | GDH1 | Stein & Stein 1992 | within 100 m | test |
| the cratons' 40 to 45 km crust (East European, Siberian, Canadian, Australian, Brazilian; CRUST1.0's layers in situ, L 225 km, K 200 km) | 150 to 360 m (rock p50) | CRUST1.0 (Laske et al. 2013), m4cold.py | within 350 m each | test |
| the Basin and Range and Cordillera's 30 to 35 and 35 to 40 km crust (L 60 km) | 1,407 and 1,606 m | CRUST1.0, m4cold.py | within 350 m each | test |
| elevation per km of crust, stable areas at 40 km / the Cordillera at 33 km (their mean crusts; a uniform 2,850 kg/m³ crust) | 141 ± 27 / 131 ± 12 m/km | Hyndman & Currie 2011, pp. 7 to 8 (read) | 126 / 124 m/km | test |
| hot against cold at equal crust (35 km) | about 1,600 m | Hyndman & Currie 2011 (read) | 1,846 m | test |
| the start's column at 38.8 km on a craton / 33 km on a back-arc | 290 m (the cratons' land p50) / 1,407 m | Hacker et al. 2015 Table 1; CRUST1.0 | 456 / within 350 m | test |

The tolerance, 350 m, is Kaban et al.'s error of the cratons' residual topography ("about 0.35 km").
Two of CRUST1.0's bands are missed and not tested: the Canadian shield's 35 to 40 km band (815 m
low) and the Cordillera box's 40 to 45 km band (836 m high: the Colorado Plateau and the Rockies,
not a hot back-arc).

## Simplifications and limits

- The geotherm is the steady conductive one of the thermal state's L; a lithosphere heating or
  cooling carries the steady profile of its current L, not a transient one.
- One rock per pool, with laboratory densities; the crust's density does not change with its
  metamorphic grade except through the eclogite root.
- The keel's compensation is one share for every keel (Kaban et al.'s average); Archean and
  Proterozoic roots differ (their 1.5 to 2.7 % against 0.6 to 1.4 %).
- The ocean keeps GDH1: the law's ocean column matches GDH1 only to 30 Myr, since the model's
  oceanic L grows to its plate's 125 km where GDH1's plate is 95 km.
- The processes' within-step caps (`collapse::toward`) and the collapse's room (PLT-412) still read
  PLT-281's line; the settling moves every column to the law's height at the end of each step.
- The collapse's sheet measures a column against the normal column at `continent_elevation_m`'s
  middle, not against its own lithosphere's normal column.
- No dynamic topography (physics-gap-audit.md).

## Game choices

- The start's thickness (PLT-835): each start continent takes the thickness at which the law holds
  it at the height the preset gives it, on the craton's state the thermal state starts it with. The
  start's area stays the preset's (`ocean_fraction` and `continental_shelf_share`). Ben chose what
  `ocean_fraction` means (decisions log 2026-09-29), not the start's column or the values; the agent
  of the continental-buoyancy lane chose this (build log 2026-10-31).
- Off by default until the pool has run it (the rule: physics goes on once built and correct).

## Open questions

- Hasterok & Chapman 2007's mantle density (3,340 or 3,260 kg/m³) is unresolved; their full texts
  were not obtained, so their thermal isostasy is not used here.
- The start's volume rises with its thicker column (the start's area is kept): whether the start's
  area should be set from Earth's crust at least 20 km thick (0.384 of the planet) is the design's
  second part (docs/design/continental-buoyancy.md).
- The law at equal crust puts hot and cold 1,846 m apart against Hyndman & Currie's ~1,600, within
  the tolerance.

## Sources

- SteinStein1992: pp. 123 to 126, read from the scanned copy's page images.
- Kaban2003: the abstract, introduction, results and conclusions; p. 66's page image checked.
- ChristensenMooney1995: the scanned full text by OCR, Table 4 on page images.
- Angevine1990: chapter 3, Fig. 3.11 on the page image (secondary to Sclater & Christie 1980, not
  read).
- HackerKelemenBehn2015: full text, Fig. 1 on the page image.
- ZobackMooney2003: full text, the equations on page images.
- LachenbruchMorgan1990: the abstract only, a fetch tool's paraphrase; the relation read in Zoback &
  Mooney 2003.
- HyndmanCurrie2011: full text with the supporting material.
- Laske2013 (CRUST1.0): measured through m4cold.py.
- Read and not used: MooneyVidale2003, Lee2014Chapter, Hacker1996, DurrheimMooney1994, OReilly2001,
  LeeLuffiChin2011, SchuttLesher2010, SudholzCopley2025, HasterokChapman2007 (abstracts),
  HasterokChapman2011.
