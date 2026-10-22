# Third-party notices

Deep Time is licensed LGPL-3.0-or-later. Code ported or adapted from other projects is listed here
with its licence and the Deep Time files that carry it. Every such file also names its source in its
header comment. GPL-licensed tools are used only as unshipped test oracles under `tools/validation/`
and contribute no code.

| Source | Licence | Used for | Deep Time files |
|---|---|---|---|
| [Landlab](https://github.com/landlab/landlab) (Hobley et al. 2017; Barnhart et al. 2020) — components `ExponentialWeatherer` / `ExponentialWeathererIntegrated` (`src/landlab/components/weathering/`), `Lithology` / `LithoLayers` (`src/landlab/components/lithology/`), `ThresholdEroder` (`src/landlab/components/threshold_eroder/`), `SimpleSubmarineDiffuser` (`src/landlab/components/marine_sediment_transport/`) and `CarbonateProducer` (`src/landlab/components/carbonate/`), read on 2026-09-28 | [MIT](https://github.com/landlab/landlab/blob/master/LICENSE.md), Copyright (c) The Landlab Team | The surface stage's equations and defaults (P1-04): soil production w₀·e^(−H/H*) and its integrated form; the strata log's layer model (layers bottom to top, same-class deposits combined, erosion from the top, surface properties from the top layer); the critical slope S(S > S_c) = S_c; the depth-dependent marine diffusivity D₀·f₁·f₂ with the wave base and tidal range; Bosscher & Schlager's G = G_m·tanh(I₀e^(−kz)/I_k) with Landlab's defaults and smooth Heaviside (adapted: climate factors, landslide rock routed as sediment, carbonate capped at sea level, a K cap with merges, base ages and provenance) | `crates/deeptime-core/src/surface/{weathering.rs, strata_log.rs, spl.rs, marine.rs, carbonate.rs}` |
| [tectonics.js](https://github.com/davidson16807/tectonics.js) by Carl Davidson — `noncompiled/models/lithosphere/RockColumn.js` and `Crust.js` (commit ce371fa) | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | The plate model's rock pools: the seven pools, the conserved felsic family, `Crust.overlap` accretion, the lithification and metamorphosis thresholds, top-down erosion (adapted: metres instead of kg/m², L0 samples instead of rasters, rules rewritten) | `crates/deeptime-core/src/plates/pools.rs`, `crates/deeptime-core/src/plates/state.rs` (`pool`) |
| [orographic-precipitation](https://github.com/fastscape-lem/orographic-precipitation) by Raphael Lange (fastscape-lem) — `orographic_precipitation/orographic_precipitation.py`, `compute_orographic_precip` (commit baf8a0c), and the defaults of `fastscape_ext.py` | [MIT](https://github.com/fastscape-lem/orographic-precipitation/blob/master/LICENSE), Copyright (c) 2020 Raphael Lange | The climate's orographic precipitation (P1-05): Smith & Barstad 2004's Fourier-space transfer function with the original's clipping of the vertical wavenumber terms, the mm/h conversion and the default τ_c, τ_f, N_m, H_w and c_w (adapted: x wraps with rows zero-padded only, each axis's own length in the wavenumbers, the wind as grid components, the planet's Coriolis parameter, rustfft) | `crates/deeptime-core/src/climate/orographic.rs` |
| [climlab](https://github.com/climlab/climlab) by Brian Rose — `climlab/model/ebm.py` (`EBM`, `EBM_seasonal`), with the processes it assembles (`AplusBT`, `StepFunctionAlbedo`, `MeridionalHeatDiffusion`, `DailyInsolation`), read on 2026-09-28 (commit fcbde37) | [MIT](https://github.com/climlab/climlab/blob/main/LICENSE), Copyright (c) 2017 Brian E. J. Rose | The climate's energy balance (P1-05): the diffusive EBM equation C ∂T/∂t = (1−α)Q − (A + BT) + (1/cos φ)∂/∂φ(cos φ D ∂T/∂φ), the P₂ albedo with an ice step, the daily-mean insolation formula and climlab's default constants (ported as equations; adapted: a land and a slab-ocean surface per band, a smoothed ice step, sea-ice insulation, ice sheets, implicit time stepping, a circular orbit) | `crates/deeptime-core/src/climate/{ebm.rs, insolation.rs}` |
| [HEXTOR](https://github.com/BlueMarbleSpace/hextor) by Blue Marble Space (Haqq-Misra & Hayworth 2022, PSJ) — the OLR table `model/radiation/radiation_N2_CO2_Sun.h5` (Kopparapu et al. 2013's radiative-convective model, 1 bar N₂ + CO₂, T 190–370 K, pCO₂ 10⁻⁴–10 bar), commit f800e25f, read on 2026-10-03 | [MIT](https://github.com/BlueMarbleSpace/hextor/blob/main/LICENSE), Copyright (c) 2020 Blue Marble Space | The climate's CO₂ forcing at every concentration and temperature (P1-26 T6): the table's 1,729 OLR values shipped as data (the duplicated column dropped, mW/m² turned into W/m²), interpolated as HEXTOR's `getOLR` does (linear in T and log₁₀ pCO₂), scaled to Myhre's all-sky doubling | `crates/deeptime-core/src/climate/{co2.rs, co2_table.rs}` |
| [MultiScaleErosion](https://github.com/H-Schott/MultiScaleErosion) by H-Schott (Schott et al. 2024, "Terrain Amplification using Multi-scale Erosion", ACM TOG 43(4)) — `data/shaders/{erosion,thermal,deposition}.glsl` and the "Result" sequences of `src/code/src/main.cpp` (commit 964a3d5), read on 2026-09-29 | [MIT](https://github.com/H-Schott/MultiScaleErosion/blob/main/LICENSE), Copyright (c) 2023 Schott | The amplifier's passes (P1-06): the multiple-flow drainage area (a cell sends its area to each lower neighbour in proportion to slope^p and takes what its neighbours send, one hop per iteration), the stream-power lowering k·A^m·S^n clamped to the receiver's height, the thermal pass (a neighbour more than tan θ below takes matter, more than tan θ above gives it; the threshold angle carries smooth noise) and the deposition pass (sediment travels along the same weights and settles where the flow's capacity is below the load), and the workflow erosion → thermal → deposition alternated with ×2 upsampling (ported as equations, not line by line; adapted: SI units and an incision length at a reference area and slope, a coarse-level drainage-area prior refined instead of started from zero, pairwise excess-proportional thermal transfer, deposition of the eroded load with a half-gap cap, Priority-Flood repair, tiles with halos on the CPU) | `crates/deeptime-core/src/surface/{amplify.rs, amplify/kernels.rs, thermal.rs}` |
| [Noddy](https://github.com/Loop3D/noddyverse) as published in Loop3D/noddyverse (Jessell & Valenta 1996; Jessell et al. 2022, Earth Syst. Sci. Data 14:381–392) — `src/unEvents.c`, `src/taste.c`, `src/matrix.c`, and the helpers those three call: `src/utils.c` (`convrt`, `rotset`, `irotate`, `matmlt`, `paxis`, `plane`, `polcar`), `src/ellFault.c`, `src/proflib.c` (`calcProfile`), `src/which.c` (`whichRock`, `whichLayer`), the orientation set-up of each event in `src/getEvent.c` and the constants of `src/nodStruc.h` (commit 6a21b2374516f8506c19cabcbe3ead00be9cc45d) | [MIT](https://github.com/Loop3D/noddyverse/blob/main/LICENSE), Copyright (c) 2021 Loop3D (the code is Mark Jessell's, Imperial College 1981–82, Monash University 1994, UWA; `matrix.c` (c) Ewen Christopher and Monash University 1994) | The fold-and-fault engine (P2-01): the inverse kinematics of folds, faults (translation, rotation, elliptical, curved, ring), shear zones, kinks, unconformities, dykes, plugs, tilts and homogeneous strain, the sequence codes and `taste`, the layer lookup, and the 4 × 4 transformation matrices (ported function by function with the original's arithmetic and constants, checked bit for bit against the C by `tools/validation/noddy/`; adapted: no global state, an exact sine beside the sampled profile, `rotset`'s out-of-range `acos` clamped, `rotateX` fixed; not ported: DXF surfaces, imported blocks, Biot folds, alteration zones, geophysics) | `crates/deeptime-noddy/src/*.rs` (the reference fixture `crates/deeptime-noddy/tests/data/reference.txt` is the original's output) |
| Petroff, "A Square Equal-area Map Projection with Low Angular Distortion, Minimal Cusps, and Closed-form Solutions" (ACM TSAS 7(4):18, 2021), supplement `new_projection.py` (`new_projection`, `new_projection_inverse`), Zenodo [10.5281/zenodo.4661351](https://doi.org/10.5281/zenodo.4661351), downloaded 2026-10-04 | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) in the file's header (the archive's record says CC BY 4.0; credit is given either way), Matthew A. Petroff | The seamless world's projection: Petroff's square equal-area map, forward and inverse, as scalar code (adapted: the inverse's quadrant rule gives each boundary ray to one side; 0/0 cases as branches; arguments clamped to their functions' domains; on Deep Time's portable trigonometry so Rust, Java and JS agree bit for bit) | `crates/deeptime-format/src/petroff.rs`, `neoforge/src/main/java/com/minecraftindustrialcomplex/deeptime/worlddata/Petroff.java`, `crates/deeptime-viewer/assets/projection.js` |

## Shared code from another repository

- **mic-viewer** (MinecraftIndustrialComplex/mic-viewer, public since 2026-10-21), MPL-2.0, by the same author as
  Deep Time: the shared world viewer's core (an HTTP server that answers loopback names only, the
  tile cache, the slippy map, the marks and their budget), shared with mic-rtsim's world viewer
  (mic-rtsim VIEWER.md; decisions log 2026-10-21). Deep Time depends on it unmodified, as a git
  dependency at the commit pinned in `Cargo.toml` (no copy in this repository since 2026-10-21,
  decisions log); its files and its licence (`LICENSE` in its repository) keep their MPL-2.0
  terms. Its server, tile cache and map core began as Deep Time's own workbench code
  (`watch/http.rs`, `watch/tiles.rs`, `deeptime-viewer`'s PNG encoder and `assets/worldmap.js`),
  which the author relicensed under MPL-2.0 for it. Deep Time links it (`crates/deeptime-cli`,
  `crates/deeptime-tiles`) and the workbench serves its `map.js`.

## APIs called (nothing ported, nothing shipped)

- **Create** 6.0.10 by the Create Team / the Creators of Create
  (https://github.com/Creators-of-Create/Create), code MIT, assets all rights reserved. The wrap
  engine's Create adapters (`neoforge/src/main/java/.../compat/create/wrap/` and
  `.../mixin/wrapengine/compat/create/`, dev-only behind `-Ddeeptime.wrapengine=true`) are mixins into
  Create's own classes (`RotationPropagator`, `DirectionalShaftHalvesBlockEntity`, `FluidPropagator`,
  `PumpBlockEntity`, `PipeConnection`, `FlowSource`, `FluidNetwork`, `BeltConnectorItem`,
  `ChainConveyorBlockEntity`, `TrackPlacement`, `TrackBlock`, `Contraption`) that call Create's code
  with turned inputs; they are compiled against the Create jar (and the Ponder, Flywheel and
  Registrate jars it nests), which is never included in Deep Time's jar. Create's source at tag
  `mc1.21.1-6.0.10` was read to find the hook points; no code was copied or adapted, and no asset is
  used.

- **Distant Horizons** by James Seibel and the Distant Horizons team
  (https://gitlab.com/distant-horizons-team/distant-horizons), LGPL-3.0. When it is installed, Deep
  Time registers a world generator through its public API (`com.seibel.distanthorizons.api`:
  `DhApi.worldGenOverrides`, `IDhApiWorldGenerator.generateLod`, `IDhApiFullDataSource`,
  `DhApiTerrainDataPoint`, `IDhApiWrapperFactory`, `IDhApiWorldProxy`, `IDhApiTerrainDataRepo`), so
  its far terrain comes from Deep Time's world data (P3-04). The API is bound by name at run time
  (`neoforge/src/main/java/.../compat/dh/DhApiBinding.java`); no Distant Horizons class is compiled
  against, copied or included in Deep Time's jar. What the API expects of a generator (the y of a
  data point counted from the level's bottom, air up to the top of the level, the generation step
  of a finished column) was read from the class files of Distant Horizons 3.3.0, 3.3.1 and 3.3.3
  with `javap` and a decompiler; no code was taken. The tests download the 3.3.0 and 3.3.3 jars
  from Modrinth and open them (`DhApiBindingTest`); they are never redistributed.

## Data

- **U.S. Geological Survey grade–tonnage models and reports** (public domain, works of the U.S.
  Government; attribution kept as good practice): OFR 2008-1155 (porphyry copper), OFR 2009-1252
  (sediment-hosted Zn–Pb), Bulletin 1693 models 31a, 32a/b, 34a and 39a, SIR 2010-5070-A, -B and -N,
  Circular 891 (coal), OFR 95-831, PP 1625-A, PP 1698, OFR 88-450-G, OFR 94-220, OFR 2003-253,
  DDS-60 and DDS-69-B. Digitised in P1-14, with the table or figure and page of every number, into
  `crates/deeptime-core/data/usgs/*.json`; the deposit types in
  `neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/` copy them. Facts cited from
  journal articles (Waples 1980, Butterworth et al. 2016, Hoggard et al. 2020, Sillitoe 2005, …)
  are numbers, not redistributed text.

## Build-time terms

- cargo-xwin downloads the Microsoft C runtime and Windows SDK to link the Windows natives. Using it
  accepts Microsoft's licence for them (https://go.microsoft.com/fwlink/?LinkId=2086102), which the
  project owner accepted on 2026-09-28.
