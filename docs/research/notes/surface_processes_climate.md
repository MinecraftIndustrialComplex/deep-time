# Surface processes, stratigraphy, hydrology and climate tooling for a simulated-geologic-history Minecraft world

*Scope: the surface half of the history (fluvial and hillslope erosion, weathering and regolith, sediment routing and stratigraphy, rivers, lakes and deltas, glaciation, sea level, climate). Tectonics and ore genesis appear only as interfaces, for example "takes an uplift-rate field". Repo licence, activity and release data came from the GitHub and PyPI APIs and from the Inria GitLab API on 2026-09-27. Runtimes are quoted from the papers and docs cited; nothing heavy was run locally.*

## 1. Landscape evolution models (LEMs): processes, outputs, speed, language, licence, maintenance

### Takeaway
The modern LEM core fits in an O(n) implicit step: stream-power incision plus sediment deposition, flow routing with depression handling, and implicit diffusion. It is well documented and portable. All of the best-maintained implementations, however, are either GPL-3.0 (fastscapelib, fastscapelib-fortran, goSPL, RichDEM, TopoToolbox v3) or Python/PETSc/MPI stacks that cannot run inside a Minecraft JVM. The MIT/BSD options (Landlab, the fastscape Python front-end, orographic-precipitation, Sequence) work as reference designs, but the fastscape front-end calls GPL Fortran and Landlab's PriorityFloodFlowRouter wraps GPL RichDEM. Measured single-thread CPU cost for an implicit stream-power step with depression handling is about 1 µs per cell per step.

### Cited Findings

#### FastScape algorithm (Braun & Willett 2013)
- The algorithm is O(n) and implicit in time. The abstract says it is efficient enough for inversions needing tens of thousands of runs at very high discretisation (about 10^8 nodes). — [Braun & Willett 2013 (Geomorphology 180–181) abstract via ResearchGate](https://www.researchgate.net/publication/236741975_A_very_efficient_On_implicit_and_parallel_method_to_solve_the_stream_power_equation_governing_fluvial_incision_and_landscape_evolution)
- Yuan et al. (2019) extended it to Davy & Lague erosion–deposition. Their solver is also O(N) and implicit, unconditionally stable at large time steps, and predicts foreland-basin stratigraphy from erosion and transport efficiencies. — [Yuan et al. 2019, JGR Earth Surface](https://doi.org/10.1029/2018JF004867)
- Braun (2023) published an implicit algorithm for the threshold stream-power incision model. — [Braun 2023, JGR Earth Surface](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2023JF007140)

#### fastscapelib (C++ plus Python)
- **Licence and packaging.** GPL-3.0. A C++ header-only library with Python bindings built on pybind11 (>=2.12,<3) and scikit-build-core, using xtensor containers. — [GitHub repo](https://github.com/fastscape-lem/fastscapelib); [install docs](https://github.com/fastscape-lem/fastscapelib/blob/HEAD/doc/source/install.md); [pyproject.toml](https://github.com/fastscape-lem/fastscapelib/blob/HEAD/pyproject.toml); [PyPI](https://pypi.org/project/fastscapelib/)
- **Releases.** v0.2.0 (2023-10-09) was a complete rewrite. It added profile, raster (with diagonal options) and triangular-mesh grids, periodic boundaries, single- and multiple-direction flow routing, and priority-flood plus basin-graph depression resolution. v0.3.0 (2025-06-25) added a HEALPix global grid (conda-forge only, not on Windows), user-defined "flow kernels" (Numba-JIT in Python), and parallel execution of kernels and flow routers. v0.4.0 is unreleased. — [release notes](https://fastscapelib.readthedocs.io/en/latest/release_notes.html)
- **Components.** Eroders are `spl.hpp` (stream power law) and `diffusion_adi.hpp`. Flow code is `flow_router`, `sink_resolver`, `basin_graph`, `flow_kernel` and `algo/pflood.hpp`. Grids are profile, raster, trimesh and HEALPix. — [repo tree](https://github.com/fastscape-lem/fastscapelib/tree/HEAD/include/fastscapelib)
- **Sink resolvers.** `PFloodSinkResolver` updates only the topography. `MSTSinkResolver` (the Cordonnier et al. 2019 basin-graph approach) requires single-flow routing. — [flow guide](https://github.com/fastscape-lem/fastscapelib/blob/HEAD/doc/source/guide_flow.md)
- **No marine or sediment modules yet.** The docs say: "We plan to add more processes in the future, like the continental and marine sediment transport / deposition modules available in the Fastscapelib-fortran". — [eroders guide](https://github.com/fastscape-lem/fastscapelib/blob/HEAD/doc/source/guide_eroders.md)
- **Activity.** Last push 2026-09-07; 46 stars. Its stated philosophy is "write algorithms once, reuse it everywhere". — [GitHub](https://github.com/fastscape-lem/fastscapelib); [why fastscapelib](https://github.com/fastscape-lem/fastscapelib/blob/HEAD/doc/source/why_fastscapelib.md)

#### fastscapelib-fortran (the legacy engine with the marine and stratigraphy features)
- **Status and licence.** The README says: "this repository is in maintenance-only mode, further development … is happening in … fastscapelib". Licence GPLv3; last release v2.8.4 (2023-09-29); last push 2024-11-07. — [README](https://github.com/fastscape-lem/fastscapelib-fortran)
- **Processes.** An enriched stream-power law with a sediment deposition term G (Yuan 2019), hillslope diffusion Kd, and marine silt–sand transport with separate porosity and transport coefficients. `FastScape_Strati` writes stratigraphic horizons to VTK. It also has spectral elastic flexure, horizontal advection (`FastScape_Set_V`), and lake geometry and depth (`FastScape_Copy_Lake_Depth`). — [fastscapelib-fortran docs](https://fastscape.org/fastscapelib-fortran/)

#### fastscape (Python, xarray-simlab components)
- **Licence and dependencies.** BSD-3-Clause; latest release 0.1.0 (2023-09-25). It uses fastscapelib-fortran (GPL) for execution and xarray-simlab as its framework. — [GitHub](https://github.com/fastscape-lem/fastscape)
- **xarray-simlab.** BSD-3; last release 0.5.0 (2021-01-26); last push 2021-11-04. — [GitHub](https://github.com/xarray-contrib/xarray-simlab)
- **Process classes (read from source).**
  - `SingleFlowRouter` and `MultipleFlowRouter`, `DrainageArea`.
  - `StreamPowerChannel`, `DifferentialStreamPowerChannel`, and `StreamPowerChannelTD` / `DifferentialStreamPowerChannelTD` (transport and deposition via the G coefficient).
  - `LinearDiffusion` and `DifferentialLinearDiffusion` (bedrock versus soil).
  - `Sea`.
  - `MarineSedimentTransport`: a silt/sand ratio, per-class porosity with an exponential e-folding compaction depth, per-class diffusivity, and an active-layer depth.
  - `StratigraphicHorizons`: horizons track the surface until a "freeze time", after which only tectonics and erosion modify them.
  - `UniformSedimentLayer` (soil or regolith).
  - Tectonics: `BlockUplift`, `TwoBlocksUplift`, `HorizontalAdvection`.
  - Isostasy: local isostasy variants and `Flexure`.
  - Initial conditions: `FlatSurface`, `Escarpment`, `BareRockSurface`.
  - Sources: [processes/](https://github.com/fastscape-lem/fastscape/tree/HEAD/fastscape/processes); [main.py](https://github.com/fastscape-lem/fastscape/blob/HEAD/fastscape/processes/main.py); [marine.py](https://github.com/fastscape-lem/fastscape/blob/HEAD/fastscape/processes/marine.py); [channel.py](https://github.com/fastscape-lem/fastscape/blob/HEAD/fastscape/processes/channel.py)
- **No orographic precipitation in core fastscape.** A separate MIT package, `orographic-precipitation`, implements the Smith & Barstad (2004) linear theory. It is numpy-only, with an optional fastscape extension; release 1.0 (2020-06-24), last push 2024-08-12. — [GitHub](https://github.com/fastscape-lem/orographic-precipitation)

#### goSPL (Salles et al.), with Badlands and eSCAPE as predecessors
- **Licence, stack and status.** GPL-3.0; Python on PETSc, petsc4py and MPI; outputs HDF5 for ParaView, with post-processing to VTK and netCDF including "stratigraphic records". — [JOSS 2020](https://www.theoj.org/joss-papers/joss.02804/10.21105.joss.02804.pdf). Release v2026.7.14 (2026-07-10); last push 2026-09-23. — [GitHub](https://github.com/Geodels/gospl); [PyPI](https://pypi.org/project/gospl/)
- **Processes in the JOSS paper.** Multiple-flow-direction stream power with implicit parallel drainage area; inland deposition in depressions by priority-flood; dual-lithology (coarse and fine) marine deposition by diffusion; hillslope processes; compaction. It is forced by deforming plate tectonics (horizontal and vertical displacement), paleotopography, paleoclimate, and sea level. — [JOSS](https://www.theoj.org/joss-papers/joss.02804/10.21105.joss.02804.pdf)
- **2026 additions in the tech guide.** Source: [tech guide index](https://gospl.readthedocs.io/en/latest/tech_guide/index.html).
  - A "diagnostic glacial-erosion model". The ELA mass balance is routed downhill to an ice discharge. Ice thickness, sliding, abrasion, till and moraine deposition, meltwater and ice load are then derived in one linear solve. A true SIA solve was implemented and then removed because it "diverged on real continental runs" and "over-thickens km-scale continental ice" at long time steps. — [ice.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/ice.rst)
  - A water table plus a generic duricrust. Crust precipitates where the water table sits in the capillary fringe, resists erosion, and produces relief inversion and "stacked, buried and re-exhumed duricrusts". — [groundwater.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/groundwater.rst)
  - A provenance post-processor that attributes deposited sediment to source-rock types per basin and per node, with transport distance and an optional copper-fertility layer. — [provenance.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/provenance.rst)
- **Scaling (goSPL's own measurements on NCI Gadi).** Source: [hpc_scaling.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/user_guide/hpc_scaling.rst).
  - A 10 km global mesh (5.9 M nodes) "cannot start below ~16 ranks" and needs about 6.6 GB on the heaviest rank.
  - The capped benchmark ran 521 s at 16 ranks and 44 s at 288 ranks.
  - A 5 km mesh (23.7 M nodes) needs at least 96 ranks and about 24 GB on rank 0.
  - The number of timed steps is capped by the harness; the step count behind the table is not stated. — [scaling harness README](https://github.com/Geodels/gospl/blob/HEAD/scripts/scaling/README.md)
- **Headline application.** Global landscape evolution over 100 Myr, assimilating paleoelevation and paleoclimate reconstructions; it quantifies sediment flux and stratigraphic architecture. — [Salles et al. 2023, Science (abstract)](https://doi.org/10.1126/science.add2541)
- **Badlands (goSPL's predecessor).**
  - Licence conflict: GitHub reports LGPL-3.0 and PyPI metadata reports GNU GPL. Release v2.3.1 (2026-05-01). — [GitHub](https://github.com/badlands-model/badlands); [PyPI](https://pypi.org/project/badlands/)
  - Mesh and processes: a TIN mesh; stream power and transport capacity; linear and nonlinear diffusion; wave-induced longshore drift; marine deposition; reef and carbonate-platform growth, pelagic growth; turbidity currents; flexure; sea level.
  - Strata record erodibility and grain size (strataMesh, carbMesh, eroMesh). The docs point to goSPL as successor. — [Badlands docs](https://badlands.readthedocs.io/en/latest/)
- **eSCAPE.** Not present in the Geodels GitHub org today (repos: bioLEC, gospl, gospl-global-workflows, paleoPhysiography, gospl-container, paleoReef, goSPL-examples, isoFlex; all GPL-3.0). — [Geodels org](https://github.com/Geodels). The goSPL JOSS paper describes goSPL as adding horizontal advection and multiple lithologies with compaction over eSCAPE. — [JOSS](https://www.theoj.org/joss-papers/joss.02804/10.21105.joss.02804.pdf)

#### Landlab
- **Licence and status.** MIT; v2.11.0 (2026); last push 2026-09-21. — [GitHub](https://github.com/landlab/landlab); [PyPI](https://pypi.org/project/landlab/)
- **Component directories in the repo.** Source: [src/landlab/components](https://github.com/landlab/landlab/tree/HEAD/src/landlab/components)
  - **Flow and hydrology:** flow_accum, flow_director, flow_router, priority_flood_flow_router, lake_fill, sink_fill, depression_finder, hand_calculator, groundwater, overland_flow, river_flow_dynamics, tidal_flow.
  - **Fluvial:** stream_power, space, erosion_deposition, detachment_ltd_erosion, area_slope_transporter, gravel_bedrock_eroder, gravel_river_transporter, lateral_erosion, network_sediment_transporter.
  - **Hillslope and weathering:** diffusion, depth_dependent_diffusion, depth_dependent_taylor_soil_creep, nonlinear_diffusion, taylor_nonlinear_hillslope_flux, transport_length_diffusion, weathering, threshold_eroder, bedrock_landslider, landslides, mass_wasting_runout.
  - **Lithology, tectonics and tracers:** lithology, flexure, gflex, normal_fault, tectonics, concentration_tracker.
  - **Marine and deltas:** marine_sediment_transport, carbonate, geoenthalpy_delta.
  - **Climate and ecology:** spatial_precip, uniform_precip, pet, radiation, vegetation_dynamics, species_evolution, fire_generator.
  - **There is no glacier and no karst component.**
- **Component details (from source).**
  - `PriorityFloodFlowRouter` "is a wrapper of the RichDEM package", which is GPL-3.0. — [source](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/priority_flood_flow_router/priority_flood_flow_router.py)
  - `LakeMapperBarnes` implements Barnes et al. (2014) lake filling and routing. — [source](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/lake_fill/lake_fill_barnes.py)
  - `LithoLayers` / `Lithology` give a 3D representation of material with per-rock-type attributes such as age and erodibility; material is removed by erosion or added by deposition. — [litholayers.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/lithology/litholayers.py); [lithology.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/lithology/lithology.py)
- **Sequence is not a Landlab component.** "Sequence" is a separate 2D (profile) sequence-stratigraphic model built on Landlab. — [Sequence docs](https://sequence.readthedocs.io/en/develop/). Package `sequence-model`: MIT, v0.6.0 (2023-12-20). — [PyPI](https://pypi.org/project/sequence-model/)

#### Other LEM and topographic-analysis codes
- **TopoToolbox v3** (all GPL-3.0). — [libtopotoolbox](https://github.com/TopoToolbox/libtopotoolbox); [pytopotoolbox](https://github.com/TopoToolbox/pytopotoolbox); [topotoolbox3](https://github.com/TopoToolbox/topotoolbox3)
  - `libtopotoolbox` is a C library with weekly releases; latest tag "2026-W38" (2026-09-21).
  - `pytopotoolbox` is at v0.0.12 (2026-06-01).
  - MATLAB `topotoolbox3` released 2026-09-16.
  - The older TopoToolbox 2 includes a `STREAMobj/fastscape.m` method. — [TopoToolbox 2 file](https://github.com/wschwanghart/topotoolbox/blob/master/@STREAMobj/fastscape.m)
- **RichDEM.** GPL-3.0, C++ with Python. Last release v2.3.1 (2022-02-11); last push 2024-06-24; the PyPI package is 0.3.4 from 2018, so maintenance looks dormant. It implements parallel priority-flood and flow accumulation "for trillion cell" DEMs (Barnes 2016, 2017). — [GitHub README](https://github.com/r-barnes/richdem); [PyPI](https://pypi.org/project/richdem/)
- **OpenLEM (Hergarten).** A single C++ source file with a Python interface, GPL-2.0-or-later, "continuously developed since 2012" with "no full documentation". It includes the shared stream-power model, LFPM orographic precipitation, a hillslope extension (2023) and semi-implicit diffusion. — [OpenLEM](http://hergarten.at/openlem/)
- **DAGGER and scabbard (Gailleton).** DAGGER is a C++ (originally header-only) GPL-3.0 graph backend; scabbard is an MIT Python front-end exposing graphflood, CHONK and trackscape/popscape. — [DAGGER](https://github.com/bgailleton/DAGGER); [scabbard](https://github.com/bgailleton/scabbard)
- **CAESAR-Lisflood.** Only forks are active.
  - The original `tcoulthard/caesar-lisflood` has no licence file and a last push in 2017.
  - `HAIL-CAESAR` is a GPL-3.0 C++ HPC variant, pushed 2025-08-12.
  - `GeospatialResearch/CAESAR-Lisflood` is GPL-3.0, pushed 2026-09-21.
  - Sources: [HAIL-CAESAR](https://github.com/dvalters/HAIL-CAESAR); [GeospatialResearch fork](https://github.com/GeospatialResearch/CAESAR-Lisflood); [tcoulthard repo](https://github.com/tcoulthard/caesar-lisflood)

#### Measured speeds (the anchors for all budgets below)
- **CPU implicit stream power (Tzathas et al. 2024).** Python+Numba on an Intel Xeon E5-2650 v4, fixed domain, grid spacing 50/25/12 m. Source: [Tzathas et al. 2024 author PDF](https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf)

  | Grid | Simulation (Cordonnier 2016 style) | Iterations | Analytical multigrid | Fixed-point |
  |---|---|---|---|---|
  | 512² | 55.5 s | 230 | 1.79 s | 7.85 s |
  | 1024² | 556 s | 460 | 8.18 s | 70.9 s |
  | 2048² | 4799 s | 980 | 33.65 s | 539 s |

  The simulation's time step had to shrink in proportion to cell size to avoid artifacts.
- **CPU versus GPU (FastFlow, Jain et al. 2024).** Source: [FastFlow author PDF](https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf)
  - A CPU implementation of Cordonnier 2016 needs 2.6 s for 10 implicit iterations (700 kyr) on 512²; FastFlow on GPU takes 0.1 s.
  - A 2048² terrain with 300 kyr (200 iterations) of fluvial erosion, deposition, and rivers and lakes takes under 10 s.
  - The implicit scheme allowed a 20× larger time step (20 kyr versus 1 kyr). A 10 Myr landscape at 512² took 7.2 s explicit versus 0.5 s implicit.
  - Flow plus depression routing runs "in under 55ms on terrains up to 4096²" on an RTX A6000.
- **Depression resolution at 16384² (Cordonnier et al. 2019).** About 100–400 s per call depending on the number of local minima. Their O(n) MST method took roughly 85–120 s per step inside a FastScape run; implementations were C++, compared against RichDEM v2.2.9. — [Cordonnier et al. 2019, ESurf (Fig. 5)](https://esurf.copernicus.org/articles/7/549/2019/esurf-7-549-2019.pdf)

### Inferences
- **Planning cost.** From Tzathas's table (for example 556 s / 460 iterations / 1024² ≈ 1.15 µs per cell per iteration) and FastFlow's CPU figure (≈1 µs per cell per iteration at 512²), a single-threaded CPU implicit stream-power step with depression handling costs about 1 µs per cell. A well-written Java port should land within a small factor of that. This is an assumption; no Java benchmark was found.
- **fastscapelib is the cleanest algorithm reference, but its code cannot be copied into a non-GPL mod.**
  - A GPL-3.0 mod could link it through JNI, but that means shipping native binaries per OS and architecture (see section 7).
  - The marine, stratigraphy and lake features Ben needs live in the maintenance-only fastscapelib-fortran (GPL) or its BSD Python wrapper. So the practical path is to reimplement from the papers (Braun & Willett 2013; Yuan 2019; Cordonnier 2019) and use fastscape and Landlab offline as validation oracles.
- **goSPL is the closest scientific match to "simulate a world's surface history with stratigraphy, glaciation, provenance and sea level".** Its own docs, however, show it is an MPI/HPC code (at least 16 ranks and 6.6 GB/rank at 10 km global resolution). It is an offline reference design, not an embeddable engine.
- **Landlab's MIT licence makes its component code the safest thing to read line by line when porting.** Useful components include `ExponentialWeatherer`, `LithoLayers`, `CarbonateProducer`, `SimpleSubmarineDiffuser`, `HeightAboveDrainageCalculator`, `ThresholdEroder` and `ConcentrationTracker`. Avoid its `PriorityFloodFlowRouter`, which is RichDEM/GPL.

### Gaps
- CHILD could not be assessed. The CSDMS model pages refused connections (ECONNREFUSED) and no CHILD repository turned up on GitHub. Its licence, language status and maintenance are unverified.
- No published fastscapelib (C++) timings at 1k², 4k² or 16k² were found. The repo contains benchmark sources but no published results. The per-cell figures above come from Tzathas, FastFlow and Cordonnier 2019.
- Landlab component throughput (for example FastscapeEroder or SPACE at 1k²) was not found in the sources reached.
- goSPL's scaling table does not state how many model steps the timed runs contain, so its seconds cannot be converted to a per-step cost.

## 2. Computer-graphics research that made erosion fast or game-friendly, and whether code exists

### Takeaway
Graphics research has already produced the key "game-speed" ideas:
- uplift-driven stream power on a stream graph (Cordonnier 2016);
- hydrology-first vector terrain that can be evaluated at any resolution (Génevaux 2013);
- GPU stream power (Schott 2023; FastFlow 2024);
- analytical stream-power solutions that skip time stepping (Tzathas 2024);
- multi-scale erosion amplification (Schott 2024), glaciers (Argudo 2020; Cordonnier 2023), dunes (Paris 2019), meanders (Paris 2023) and caves (Paris 2021).

Much of the code is MIT (the LIRIS/Adobe group: Schott, Paris, Argudo). The Inria "landscapes" codes (Tzathas 2024, FastFlow, glacial erosion) are research-only and non-commercial, so only their papers can be used.

### Cited Findings
- **Cordonnier et al. 2016 (CGF / Eurographics).** The "first method in computer graphics that combines uplift and hydraulic erosion". A user-painted uplift map drives stream-power erosion on a stream graph, which is converted to a DEM by blending landform feature kernels, generating "large realistic terrains at a low computational cost". — [DOI 10.1111/cgf.12820 (abstract via OpenAlex)](https://doi.org/10.1111/cgf.12820). The CPU cost of that simulation style is about 2.6 s per 10 implicit iterations at 512². — [FastFlow PDF](https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf)
- **Cordonnier et al. 2017, "Sculpting Mountains" (TVCG).** A layered crust model with volume-preserving folding under plate collision produces a volumetric uplift map. Erosion and uplift are simulated jointly, and "the stratigraphy allows us to render folded strata on eroded cliffs". — [DOI 10.1109/TVCG.2017.2689022](https://doi.org/10.1109/TVCG.2017.2689022)
- **Génevaux et al. 2013 (SIGGRAPH), hydrology-first.** A hierarchical drainage network is built as a geometric graph, then watersheds and river types are derived. Terrain is a construction tree of procedural terrain and river patches that is "both analytic and continuous and can be rendered by using varying levels of detail". — [DOI 10.1145/2461912.2461996](https://doi.org/10.1145/2461912.2461996)
- **Schott et al. 2023 (ACM TOG 42(5)).**
  - **Method.** Modelling happens in the "uplift domain"; stream-power erosion is simulated interactively with a "fast yet accurate approximation of drainage area and flow routing". Tools include copy-paste, warping for "folds and faults", elevation constraints, and inverse modelling of uplift from a DEM. — [project page](https://aparis69.github.io/public_html/projects/schott2023_Uplift.html)
  - **Code.** MIT, C++/CMake on Windows and Linux. — [StreamPowerErosion](https://github.com/H-Schott/StreamPowerErosion)
  - **WebGPU port.** A port called "TerrainX" (BSD-3) reports about 40 fps "even with 4.5k size textures" on an RTX 3060 Laptop GPU. — [WebGPU port](https://github.com/GPU-Gang/WebGPU-Erosion-Simulation)
  - **Criticism.** The FastFlow authors say Schott's GPU routing needs about √n iterations (the length of the longest river). It fails for pre-existing depressions and under noisy uplift, where it underestimates drainage and erosion. — [FastFlow PDF](https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf)
  - **Time-step limit.** Tzathas notes that explicit GPU schemes bound the time step, and that without depression breaching water "is trapped within the depressions". — [Tzathas PDF](https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf)
- **Tzathas, Gailleton, Steer & Cordonnier 2024 (CGF / Eurographics 2024).**
  - **Method.** Analytical stream-power solutions along the stream tree, accelerated by multigrid. Hillslopes use a steady-state approximation through Hack's law. Thermal erosion uses a critical slope of atan 30° (0.57).
  - **Speed.** 1024² takes 8.18 s versus 556 s for simulation (Python+Numba, single Xeon). The multigrid needs "one level of down/up-sampling" per resolution doubling, so cost is "almost linear".
  - **Code.** At gitlab.inria.fr/landscapes/analytical-terrains.
  - Source: [Tzathas PDF](https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf)
  - **Licence.** Inria's research licence: "may be used 'non-commercially', i.e., for research and/or evaluation purposes only" and "THE USER CANNOT USE, EXPLOIT OR DISTRIBUTE THE SOFTWARE FOR COMMERCIAL PURPOSES". — [analytical-terrains repo (LICENSE.md)](https://gitlab.inria.fr/landscapes/analytical-terrains)
- **FastFlow (Jain, Kerbl, Gain, Finley & Cordonnier; CGF / Pacific Graphics 2024).**
  - **Algorithm.** GPU flow routing in O(log n) iterations and depression routing in O(log² n), using a depression MST. It is 5× faster for flow routing and 34–52× for depression routing than prior work at 1024², and the whole solve runs "in under 55ms on terrains up to 4096²" (RTX A6000).
  - **Limits.** Single flow direction only (no MFD). A semi-implicit deposition scheme "comes at the cost of some material loss".
  - **Code.** PyTorch plus CUDA at gitlab.inria.fr/landscapes/fastflow, under the same Inria research-only licence.
  - Sources: [FastFlow PDF](https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf); [fastflow repo (LICENSE.md)](https://gitlab.inria.fr/landscapes/fastflow)
- **Jain, Benes & Cordonnier 2024, debris flow (ACM TOG 43(4)).** Debris-flow erosion dominates steep, low-drainage areas near ridges, where slope-reducing erosion yields "unrealistically uniform slopes". The method uses a unified GPU erosion and deposition algorithm and approximate GPU routing out of newly formed depressions. — [DOI 10.1145/3658213](https://doi.org/10.1145/3658213); [HAL author PDF](https://hal.science/hal-04574826v1/file/2024_Siggraph_Debris_Flow_Author_Version.pdf)
- **Schott et al. 2024, "Terrain Amplification using Multi-scale Erosion" (ACM TOG 43(4), SIGGRAPH 2024).** Amplifies a low-resolution terrain into a high-resolution, hydrologically consistent one, using fast approximations of thermal erosion, stream-power erosion and deposition at several scales. — [DOI 10.1145/3658200](https://doi.org/10.1145/3658200). The code is MIT. It "requires at least OpenGl 4.3 for compute shaders", and its recommended workflow alternates erosion, thermal and deposition passes with ×2 upsampling. — [MultiScaleErosion README](https://github.com/H-Schott/MultiScaleErosion)
- **Paris et al. 2019, "Terrain Amplification with Implicit 3D Features" (ACM TOG).** Arches and overhangs are built as implicit construction trees, placed by Poisson sampling and "stratified erosion and invasion percolation". MIT code. — [project page](https://aparis69.github.io/public_html/projects/paris2019_3D.html); [GitHub](https://github.com/aparis69/Implicit-Volumetric-Terrains)
- **Paris et al. 2019, "Desertscape Simulation" (CGF / Pacific Graphics 2019).** An aeolian simulation producing barchan, longitudinal and anchored dunes, plus bedrock abrasion. The surface wind is computed from a high-altitude wind field and the relief. The code has an MIT licence and a replicability stamp. — [project page](https://aparis69.github.io/public_html/projects/paris2019_Deserts.html); [GitHub](https://github.com/aparis69/Desertscapes-Simulation)
- **Argudo et al. 2020, glaciers (ACM TOG, SIGGRAPH Asia 2020).** A Shallow Ice Approximation (SIA) simulation plus procedural amplification. It is multiresolution and simulates "formation and evolution of glaciers over hundreds of years" interactively; crevasses, moraines, seracs, ogives and icefalls are synthesised procedurally. — [DOI 10.1145/3414685.3417855](https://doi.org/10.1145/3414685.3417855). The MIT code is C++ with Qt 5.15, OpenGL 4.3 and GLEW, tested on a GTX 1060. — [oargudo/glaciers](https://github.com/oargudo/glaciers)
- **Cordonnier et al. 2023, "Forming Terrains by Glacial Erosion" (ACM TOG 42(4)).** Uses a "deep learning-based estimation of high-order ice flows" and a multi-scale advection scheme across glacial and interglacial cycles. It produces U-shaped and hanging valleys, fjords and glacial lakes. — [DOI 10.1145/3592422](https://doi.org/10.1145/3592422). The code (gitlab.inria.fr/landscapes/glacial-erosion) is under an Inria research-only licence. — [glacial-erosion repo (license.txt)](https://gitlab.inria.fr/landscapes/glacial-erosion)
- **Paris et al. 2023, meandering rivers (ACM TOG, SIGGRAPH Asia 2023).** A vector-based migration model with cutoffs forming oxbow lakes, and avulsions, running "at interactive rates". MIT code. — [project page](https://aparis69.github.io/public_html/projects/paris2023_Meanders.html); [GitHub](https://github.com/aparis69/Meandering-rivers)

### Inferences
- **The most Minecraft-compatible pattern combines two layers.**
  - First, a coarse physics layer: uplift-driven stream power on a grid or stream graph (Cordonnier 2016, or Braun & Willett run on a coarse grid).
  - Second, a resolution-independent evaluation layer: vector rivers and landform kernels (Génevaux 2013, Cordonnier 2016), or analytical elevation along the stream tree (Tzathas 2024), queried per chunk at block resolution.
  - This avoids a global block-resolution pass.
- **Tzathas-style analytical solutions** turn "final landscape for a given uplift history" into seconds rather than minutes. Their code can't be redistributed, so Ben would reimplement from the paper.
- **GPU paths (FastFlow, Schott 2023/2024, Argudo 2020) need OpenGL 4.3 compute shaders.**
  - macOS stops at OpenGL 4.1 (see section 7), so every GPU stage needs a CPU fallback.
  - FastFlow's code is research-only in any case.
  - Schott's MIT shaders are the only GPU stream-power and amplification code that can legally be ported into a public mod.

### Gaps
- Per-iteration timings for Schott 2023 and Schott 2024, Argudo 2020 and Cordonnier 2023 could not be extracted. The HAL preprints sit behind an anti-bot wall (Anubis), and ACM and Wiley returned HTML to scripted downloads.
- No released code was found for Cordonnier 2016, Cordonnier 2017, Génevaux 2013 or the Jain 2024 debris-flow paper. Absence was not exhaustively confirmed, because the web search budget was exhausted mid-task.
- The Galin et al. terrain-modelling survey and 2025–2026 graphics follow-ups beyond those listed could not be searched, for the same reason.

## 3. Weathering and regolith (soil production, saprolite and laterite, karst, talus): inputs, outputs, and visible game features

### Takeaway
Quantitative, portable models exist:
- exponential soil production;
- Braun (2016) weathering-front propagation driven by groundwater flow;
- goSPL's 2026 water-table plus duricrust scheme, which produces relief inversion and buried crusts;
- threshold or thermal slopes for talus.

These can drive visible block features: regolith depth, saprolite thickness, laterite or duricrust caps, and scree aprons. Karst is the weak spot. No LEM surveyed has a karst-dissolution component; the only usable code is a graphics cave-network synthesiser (MIT).

### Cited Findings
- **Landlab `ExponentialWeatherer`.** Soil production `w0 * exp(-soil__depth / w_star)` "in the style of Ahnert (1976)". `ExponentialWeathererIntegrated` uses the analytical integral over a time step. — [exponential_weathering.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/weathering/exponential_weathering.py); [exponential_weathering_integrated.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/weathering/exponential_weathering_integrated.py). The canonical empirical reference is Heimsath et al. 1997, "The soil production function and landscape equilibrium". — [DOI 10.1038/41056](https://doi.org/10.1038/41056)
- **Braun et al. 2016, regolith by chemical weathering (JGR Earth Surface).**
  - **Mechanism.** The weathering front advances "linearly proportional to the pore fluid velocity", which comes from the computed water-table geometry.
  - **Where regolith is thickest** (beneath highs or lows) depends on a dimensionless number combining slope², hydraulic conductivity and precipitation, "independent of the chemical weathering rate".
  - **In orogens,** regolith existence and thickness depend on the ratio of the erosion timescale to the weathering timescale.
  - **In anorogenic areas,** regolith thickness increases (the OpenAlex abstract is truncated at that point).
  - Source: [DOI 10.1002/2016JF003914 (abstract via OpenAlex)](https://doi.org/10.1002/2016JF003914)
- **goSPL water table and "generic duricrust" (2026).** A shallow groundwater head is solved each step from infiltrated rainfall. Where water-table depth lies in a capillary fringe, "an indurated crust precipitates and resists erosion, producing relief inversion and — with stratigraphy on — stacked, buried and re-exhumed duricrusts". It runs after flow accumulation and before erosion. — [groundwater.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/groundwater.rst). The same tech guide lists continental soil and regolith production and weathering. — [tech guide index](https://gospl.readthedocs.io/en/latest/tech_guide/index.html)
- **Laterite and bauxite formation conditions.**
  - "high temperatures and heavy rainfall alternating with dry periods".
  - Favoured on "low topographical reliefs of gentle crests and plateaus".
  - Bauxite forms from granitic or iron-poor rocks; iron-rich laterite from mafic rocks.
  - Caps can be about 30 m thick; profiles run soil → laterite → saprolite → bedrock.
  - An active lateritization period ran about 35 to 1.5 Ma.
  - Source: [Wikipedia: Laterite](https://en.wikipedia.org/wiki/Laterite)
- **Talus and scree.**
  - Landlab `ThresholdEroder` caps slopes at a threshold Sc and "assumes material to dissolve away". — [threshold_eroder.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/threshold_eroder/threshold_eroder.py)
  - Tzathas 2024 adds thermal erosion above a critical slope of atan 30°. — [Tzathas PDF](https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf)
  - Schott 2024 includes a thermal pass in its amplification workflow. — [MultiScaleErosion](https://github.com/H-Schott/MultiScaleErosion)
  - Landlab also ships `BedrockLandslider` and `MassWastingRunout`. — [components dir](https://github.com/landlab/landlab/tree/HEAD/src/landlab/components)
- **Karst.** Landlab's components include no karst or dissolution process. — [components dir](https://github.com/landlab/landlab/tree/HEAD/src/landlab/components). Paris et al. 2021 instead synthesise "geologically coherent cave networks" procedurally, outside any LEM.
  - Skeletons come from a gridless anisotropic shortest path using inlets and outlets, faults, inception horizons, fractures and permeability contrasts; conduits are signed-distance-function (SDF) construction trees. — [project page](https://aparis69.github.io/public_html/projects/paris2021_Karsts.html)
  - The released MIT code was "recoded … to make sure it is free to use", so results and timings may differ from the paper. — [Karst-Synthesis](https://github.com/aparis69/Karst-Synthesis)
- **Climate-scale chemical weathering (CO₂ feedback).** ExoPlaSim offers `co2weathering` (carbon–silicate weathering), with an `erosionsupplylimit` cap. Its docs note that weathering experiments usually "estimate the average weathering rate for a given climate in order to interpolate timescales between climates, rather than modelling changes in CO2 over time directly". — [ExoPlaSim source docstring](https://github.com/alphaparrot/ExoPlaSim/blob/HEAD/exoplasim/__init__.py)

### Inferences
- **Mapping weathering to visible features.** These rules are straightforward to run per column on the coarse LEM grid.

  | Model output or condition | Visible feature |
  |---|---|
  | Soil/regolith depth H (exponential production minus hillslope transport) | Dirt, coarse-dirt or gravel thickness |
  | Weathering-front depth (Braun 2016) | Saprolite layer: clay or "rotten" stone variant |
  | Duricrust flag where tropical wet–dry climate, low relief and long exposure coincide | Laterite or bauxite cap blocks: red-sandstone- or terracotta-like, or a mod ore block |
  | Slopes above critical after thermal erosion | Scree or gravel aprons |
  | Carbonate lithology × runoff × time, with caves from a Paris-2021-style skeleton | Karst |

- **The goSPL design doc is the only surveyed implementation of duricrust with burial and exhumption.** It is GPL, so reimplement from its described algorithm rather than its code.

### Gaps
- No quantitative LEM-coupled karst-dissolution model, and no saprolite or laterite rate law with coefficients, was retrieved. Searching stopped when the budget ran out, and the Braun 2016 full text (coefficients) was not accessible.
- The rate constants for Landlab `w0` and `w_star` or Heimsath's soil production were not retrieved. The Heimsath abstract was unavailable.

## 4. Sedimentation and stratigraphy: which tools record dated, facies-bearing layers usable as block layers

### Takeaway
goSPL, Badlands and fastscape/fastscapelib-fortran all record stratigraphy, but at different fidelity:
- **goSPL:** per-node layers with deposition elevation, thickness and porosity, dual lithology, compaction by rock-type trends, and provenance.
- **Badlands:** strata with erodibility and grain size, plus reef or carbonate and turbidites.
- **fastscape:** frozen horizons plus a silt/sand marine layer.

Landlab offers the most portable building blocks (MIT): `LithoLayers` for 3D rock-type stacks, `CarbonateProducer`, a submarine diffuser, a delta component, and tracers. None outputs "sandstone/shale/limestone/evaporite/coal" facies directly. Those have to come from rules on grain-size fraction, water depth, carbonate production and climate.

### Cited Findings
- **goSPL stratigraphy.**
  - Each node's layers store "elevation at time of deposition", "thickness", and "porosity … computed at centre of each layer"; erosion, transport and deposition are recorded at chosen steps.
  - Compaction follows published trends for "sandstone, shale, carbonate".
  - Source: [strat.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/strat.rst)
  - Dual-lithology (coarse and fine) routing. — [JOSS](https://www.theoj.org/joss-papers/joss.02804/10.21105.joss.02804.pdf)
  - Provenance attribution per basin and per pixel to source-rock types. — [provenance.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/provenance.rst)
  - Priority-flood returns, for each depression, "its volume, its spill over node and a basin unique identifier". — [dep.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/dep.rst)
- **Badlands stratigraphy.** Stratal layers record erodibility and grain size; there is reef and carbonate-platform growth, pelagic deposition, turbidity currents and wave-driven longshore drift. — [Badlands docs](https://badlands.readthedocs.io/en/latest/)
- **fastscape and fastscapelib-fortran.**
  - `StratigraphicHorizons` uses freeze times per horizon. `MarineSedimentTransport` handles a silt/sand fraction with porosity and compaction. `StreamPowerChannelTD` handles continental deposition, and `UniformSedimentLayer` handles soil or regolith. — [fastscape processes source](https://github.com/fastscape-lem/fastscape/tree/HEAD/fastscape/processes)
  - `FastScape_Strati` writes horizons to VTK. — [fastscapelib-fortran docs](https://fastscape.org/fastscapelib-fortran/)
- **Landlab building blocks.**
  - `LithoLayers` / `Lithology`: rock types carry attributes such as age and erodibility; deposition adds material. — [litholayers.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/lithology/litholayers.py)
  - `CarbonateProducer`: the Bosscher & Schlager (1992) light-limited growth law `G = Gm tanh(I0 e^{-kz}/Ik)`. — [carbonate_producer.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/carbonate/carbonate_producer.py)
  - `SimpleSubmarineDiffuser`: water-depth-dependent diffusivity with a wave-base decay. — [simple_submarine_diffuser.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/marine_sediment_transport/simple_submarine_diffuser.py)
  - `GeoEnthalpyDelta`: topset and foreset delta deposition by an enthalpy formulation. — [geoenthalpy_delta.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/geoenthalpy_delta/geoenthalpy_delta.py)
  - `ConcentrationTrackerForSpace`: mass-balance tracking of any sediment property, usable for provenance. — [concentration_tracker_for_space.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/concentration_tracker/concentration_tracker_for_space.py)
- **Sequence.** A 2D profile model with fluvial and marine transport, sea-level change, compaction, local or flexural isostasy, and subsidence or uplift. MIT, v0.6.0 (2023-12). — [Sequence docs](https://sequence.readthedocs.io/en/develop/); [PyPI](https://pypi.org/project/sequence-model/)
- **SedFlux.** "A stratigraphic basin-filling model", in C, MIT; the repo was last pushed 2025-09-11. — [mcflugen/sedflux](https://github.com/mcflugen/sedflux)
- **pyDeltaRCM.** A reduced-complexity delta model using weighted random walks of water parcels. MIT, v2.3.0 (2026-07-30), published in JOSS. — [GitHub](https://github.com/DeltaRCM/pyDeltaRCM); [PyPI](https://pypi.org/project/pyDeltaRCM/)
- **Yuan et al. 2019.** Foreland-basin stratigraphy emerges from the efficiency of erosion and of sediment transport in an O(N) implicit scheme. — [DOI 10.1029/2018JF004867](https://doi.org/10.1029/2018JF004867)

### Inferences
- **Record per coarse-grid column, not per block.** An event log of (age, thickness, facies id, provenance id) is enough. Facies is decided at deposition time by rules:
  - fluvial or deltaic coarse fraction → sandstone or conglomerate;
  - fine fraction offshore or in lakes → shale or mudstone;
  - carbonate production above wave base and within the photic zone (the Landlab law) → limestone;
  - arid climate plus a closed basin or restricted sea → evaporite;
  - humid, low-gradient, subsiding swamp with high vegetation → coal;
  - duricrust flag → laterite.
- **Merge adjacent events with the same facies** to bound memory. For example, 512² columns × 200 events × 8 bytes ≈ 420 MB before merging, versus about 105 MB at 256². That is arithmetic, not a measured figure.
- **Block-layer placement.** At block resolution, interpolate layer tops between coarse columns (bilinear or kernel), add noise-scale thickness variation, and cut by the final erosion surface. Folds and faults from the tectonics module then act on these surfaces, as in Cordonnier 2017's "folded strata on eroded cliffs".

### Gaps
- No retrieved tool outputs evaporite or coal facies directly. Those facies rules are my proposal, not a published scheme that was checked.
- pyDeltaRCM's stratigraphy-recording features were not verified beyond the README.

## 5. Climate: simplified GCMs versus parametric fields for biomes, and cheap paleoclimate through geologic time

### Takeaway
Real GCMs are hours per equilibrium climate on a laptop (ExoPlaSim) or tens of seconds to minutes per simulated year (SpeedyWeather). Each needs Fortran/MPI or Julia and carries GPL-2.0 or EUPL-1.2, so they belong offline, for precomputed reference states or calibration. In-mod climate should be parametric and composable:
1. latitude insolation and an energy-balance model with ice-albedo feedback;
2. circulation-cell wind belts and ITCZ;
3. lapse rate;
4. linear orographic precipitation (Smith & Barstad; LFPM), costing O(N) to O(N log N) per snapshot;
5. Köppen classification;
6. ELA-based glaciation and a prescribed eustatic sea-level curve.

### Cited Findings
- **ExoPlaSim (Paradise et al., MNRAS 2022).** Sources: [arXiv 2107.07685](https://arxiv.org/abs/2107.07685); [GitHub](https://github.com/alphaparrot/ExoPlaSim); [PyPI](https://pypi.org/project/exoplasim/); [docs](https://exoplasim.readthedocs.io/en/latest/)
  - **Cluster speed.** T21 (32×64), 10 layers, 45-minute time step: 30–60 s of wall time per simulated year on 8–32 threads.
  - **Laptop speed.** A 2017 i7-7700HQ laptop with 4 MPI threads takes 3.75 min per year (5.5 min with a 30-minute step).
  - **Higher resolutions.** T42: about 15–16 min per year on 16 threads. T170: about 60 h per year on 32 threads.
  - **Spin-up.** T21 aquaplanets need 50–200 years to reach energy balance.
  - **Licence and packaging.** GPL-2.0 (GitHub, PyPI); v3.4.2 on PyPI (2025-01-02); pushed 2026-09-19. It requires gfortran/gcc and optionally MPI and netCDF; Windows only via WSL.
  - **Options.** Custom `landmap`/`topomap` (.sra files), a mixed-layer ocean (default 50 m), glaciers (persistent snow above `mindepth` becomes glacier; optional initial ice sheets), and `co2weathering`. — [ExoPlaSim __init__.py docstrings](https://github.com/alphaparrot/ExoPlaSim/blob/HEAD/exoplasim/__init__.py)
  - **PlaSim.** The paper describes it as able "to model a year of climate in under a minute of wall-time". — [arXiv 2107.07685](https://arxiv.org/abs/2107.07685). The upstream mirror HartmutBorth/PLASIM shows no detected licence and was last pushed in 2019. — [GitHub](https://github.com/HartmutBorth/PLASIM)
- **Worldbuilding Pasta's ExoPlaSim workflow for fictional worlds.** Source: [Worldbuilding Pasta ExoPlaSim guide](https://worldbuildingpasta.blogspot.com/2021/11/an-apple-pie-from-scratch-part-vi.html)
  - T21 takes "10 minutes to over an hour for 10 simulated years on typical laptops".
  - Run with `runtobalance()` (minimum 10 and maximum 100 years), then classify Köppen from monthly `tas`, `ts` and `pr` with `koppenpasta.py`.
  - Recommends at least T42 for publishable maps.
  - Known biases: high latitudes are too cold (no ocean heat transport) and ice sheets are hard to form.
- **SpeedyWeather.jl.** Sources: [README](https://github.com/SpeedyWeather/SpeedyWeather.jl); [LICENSE](https://github.com/SpeedyWeather/SpeedyWeather.jl/blob/main/LICENSE); [JOSS 2024](https://www.theoj.org/joss-papers/joss.06323/10.21105.joss.06323.pdf)
  - **Speed.** About 400 km (T31), 8 layers: 2300 simulated years per day (about 40 s per year) on one M4 core, and about 500–600 simulated years per day on older Intel MacBooks. Fortran SPEEDY is reported at 240.
  - **Model components.** A slab ocean, thermodynamic sea ice, and a 2-layer land bucket model (soil temperature, moisture, vegetation, snow).
  - **GPU claims conflict inside the README.** The feature list says "typically 5-50x faster"; the benchmark section says GPU support is "a work in progress" with no expected gain at low resolution.
  - **Licence changed.** EUPL-1.2 since a 2026-03-02 commit ("EUPL license for SpeedyWeather+submodules (#992)"). Release v0.22.1 (2026-08-21).
  - **Design.** A Julia reinvention of SPEEDY with modular parameterizations. — [JOSS 2024](https://www.theoj.org/joss-papers/joss.06323/10.21105.joss.06323.pdf)
- **Isca.** Fortran with a Python front end, GPL-3.0. Options range from Held-Suarez to grey, two-band and multi-band radiation, with idealized or realistic continents and topography. It needs gfortran and netcdf-fortran through conda environments. Last tagged release v1.0 (2018), but pushed 2026-08-13. — [Isca README](https://github.com/ExeClim/Isca)
- **climlab.** MIT, v0.9.2 (2026-03-18). Moist and dry energy-balance models (seasonal and annual-mean), radiative-convective columns with RRTMG, and 1D advection-diffusion solvers. The Fortran parts were moved to companion packages. — [README](https://github.com/climlab/climlab); [PyPI](https://pypi.org/project/climlab/)
- **Smith & Barstad (2004) linear orographic precipitation.** Two steady advection equations (cloud water and hydrometeors) solved by Fourier transform into "a single formula relating terrain and precipitation". MIT Python implementation. — [orographic-precipitation](https://github.com/fastscape-lem/orographic-precipitation)
- **LFPM (Hergarten & Robl 2022, GMD).** A linear-feedback precipitation model built for LEMs. It is linear-time and costs about 2.4× the simplest stream-power step, or 2.2× the shared stream-power model. OpenLEM's implementation uses periodic boundary conditions in the y direction by default, and advection runs along one grid axis. — [LFPM paper](https://gmd.copernicus.org/articles/15/2063/2022/gmd-15-2063-2022.pdf). It is implemented in OpenLEM (GPL-2.0+). — [OpenLEM](http://hergarten.at/openlem/)
- **Köppen–Geiger rules.** Inputs are 12 monthly mean temperatures and precipitations. Source: [Wikipedia: Köppen climate classification](https://en.wikipedia.org/wiki/K%C3%B6ppen_climate_classification)
  - A: every month ≥18 °C.
  - B: aridity threshold of 20 × mean annual temperature plus 280, 140 or 0 depending on the share of precipitation in the high-sun half.
  - C and D: split at a coldest month of 0 °C (or −3 °C), with at least one month above 10 °C.
  - E: warmest month below 10 °C; ET versus EF split at 0 °C.
  - Second and third letters: f/m/w/s, a/b/c/d, h/k.
- **Parametric hand method (Worldbuilding Pasta Part VIb).** Source: [Worldbuilding Pasta Part VIb](https://worldbuildingpasta.blogspot.com/2020/05/an-apple-pie-from-scratch-part-vib.html)
  - Wet belts at 0° and 60°, dry belts at 30° and the poles.
  - Subtropical highs at 25–35°; the ITCZ follows the thermal equator seasonally.
  - Gyre currents warm western coasts by up to 15 °C and cool eastern coasts by 10 °C around 60° latitude.
  - A lapse rate of 4.46 °C per km is quoted from Clima-Sim.
  - Rain reaches about 2000 km inland from warm coasts, is blocked by relief of at least 1000 m, peaks at 1–1.5 km relief on windward slopes, and stays dry above 4 km.
- **Game-oriented generators with simple climate.**
  - Undiscovered Worlds (GPL-3.0, C++) generates Köppen zones, rivers, lakes (including salt lakes), deltas and glaciers, with regional maps at about 1 px per 1 km. — [Undiscovered Worlds blog](https://undiscoveredworlds.blogspot.com/2019/01/what-is-undiscovered-worlds.html); [Classic repo](https://github.com/JonathanCRH/Undiscovered_Worlds_Classic)
  - Worldengine (MIT, Python) uses "plate simulations, erosion, rain shadows, Holdridge life zones". — [worldengine](https://github.com/Mindwerks/worldengine)
- **Cheap glaciation from temperature (goSPL).** `ELA(x) = z(x) + (T(x) − T_ELA)/Γ`, run "once per paleo-climate snapshot" to drive the diagnostic ice model. — [ela_from_temperature.py](https://github.com/Geodels/gospl/blob/HEAD/gospl/tools/ela_from_temperature.py)
- **Sea level in LEMs.** Sea level is an external forcing: goSPL takes "sea-level fluctuations" — [JOSS](https://www.theoj.org/joss-papers/joss.02804/10.21105.joss.02804.pdf) — and fastscape has a `Sea` process. — [marine.py](https://github.com/fastscape-lem/fastscape/blob/HEAD/fastscape/processes/marine.py)

### Inferences
- **A GCM per geologic snapshot is out of reach in minutes.** ExoPlaSim at T21 needs 50–200 years × 3.75 min per year, which is about 3–12 h per equilibrium state. SpeedyWeather needs about 40–170 s per year plus spin-up.
- **Recommended in-mod pipeline for each snapshot, every 1–10 Myr:**
  1. A zonal EBM (climlab-style diffusive EBM with an ice-albedo feedback; port the equations) for the latitudinal temperature profile, including obliquity and CO₂ or solar forcing.
  2. Continentality and lapse-rate corrections on the coarse DEM.
  3. Wind belts from the cell model.
  4. LFPM or Smith–Barstad precipitation, run along each belt's wind direction.
  5. Köppen classification, then the ELA for glaciers.
  6. A sea-level curve set as a user preset.
  
  Because LFPM costs about 2.4 stream-power steps, the whole climate update should cost the same order as one or two erosion steps, which is negligible (inference from the LFPM cost ratio).
- **Offline GCMs as a lookup table.** ExoPlaSim or SpeedyWeather could build a small table of "climate response to continent layout and CO₂" to calibrate the parametric model. This mirrors ExoPlaSim's own practice of interpolating between equilibrium climates.
- **Biome mods need climate as parameters.** Köppen class plus mean temperature and precipitation map naturally onto vanilla-style temperature/humidity parameters. That is an inference; the exact mapping belongs to the biome-integration researcher.

### Gaps
- ROCKE-3D was not researched (search budget exhausted); no runtime figures are given.
- Isca and PlaSim laptop runtimes for a small planet were not found; only ExoPlaSim's (PlaSim-derived) numbers are available.
- No cited source was obtained for cheap ocean-current parametrisations beyond the Worldbuilding Pasta rules of thumb, or for published Phanerozoic sea-level curves (Haq, Miller) as data.
- Whether SpeedyWeather accepts arbitrary custom orography and land-sea masks for a synthetic planet was not verified.

## 6. Hydrology at game resolution: rivers, lakes, deltas, floodplains, waterfalls, aquifers, and cost at 1 block over 16k×16k

### Takeaway
The algorithms are mature and O(n):
- priority-flood (Barnes 2014, 2016) or the MST basin graph (Cordonnier 2019) for depressions;
- Fill–Spill–Merge (MIT) for real lake levels given runoff;
- HAND for floodplains;
- vector meander and delta models for planform detail.

But a single global pass at 1 block over 16k² (268 M cells) costs about 100 s of CPU for depression routing alone, plus several GB of arrays. Full 2D hydrodynamics (GraphFlood) is hours to days. Game-resolution water should therefore come from coarse-grid routing (4–16 blocks per cell) plus vector carving and per-chunk local refinement.

### Cited Findings
- **Priority-Flood (Barnes et al. 2014).** The general variant is O(n log n) for floating-point DEMs, with an O(m log m) improved variant (m ≤ n), which was on average 16.8% faster. It was tested on about 8×10⁹ cells of 3 m Minnesota LiDAR across 44 counties (average about 2×10⁸ cells each) on a Xeon X5560. — [Barnes et al. 2014, arXiv 1511.04463](https://arxiv.org/abs/1511.04463)
- **Parallel priority-flood and flow accumulation for trillion-cell DEMs (Barnes 2016, 2017).** Distributed, tile-based. — [RichDEM README](https://github.com/r-barnes/richdem). goSPL uses the Barnes (2016) three-phase approach: per-tile flood, then a spillover graph, then a serial master solve. — [dep.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/dep.rst)
- **Depression cost at 16384² (268 M cells).** About 100–400 s per call depending on the number of minima, and about 85–120 s per step for the O(n) MST method inside a FastScape run. — [Cordonnier et al. 2019](https://esurf.copernicus.org/articles/7/549/2019/esurf-7-549-2019.pdf)
- **GPU routing.** Flow plus depression routing takes under 55 ms up to 4096²; single-flow direction only; research-only code. — [FastFlow PDF](https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf)
- **Lakes with water balance (Fill–Spill–Merge, Barnes, Callaghan & Wickert 2021).** Runoff "fills depressions, which then overflow and spill into their neighbors. If both a depression and its neighbor fill, they merge". It is 90–2600× faster than Jacobi iteration. — [ESurf 2021](https://esurf.copernicus.org/articles/9/105/2021/). MIT C++ code. — [GitHub](https://github.com/r-barnes/Barnes2020-FillSpillMerge). Landlab has `LakeMapperBarnes`. — [source](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/lake_fill/lake_fill_barnes.py)
- **Floodplains (HAND).** Landlab `HeightAboveDrainageCalculator` implements Nobre et al. (2011) height above the nearest drainage. — [hand_calculator.py](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/hand_calculator/hand_calculator.py)
- **2D hydrodynamics (GraphFlood 1.0, Gailleton et al. 2024).** A stationary shallow-water approximation on the flow DAG with Manning friction, about 10× faster than River.lab. It solves rivers "in just seconds for a million-pixel DEM". An 83-million-pixel DEM on a 32 GB laptop needed about 20 h (100 s per iteration). Single-threaded. — [GraphFlood ESurf 2024](https://esurf.copernicus.org/articles/12/1295/2024/esurf-12-1295-2024.pdf)
- **Planform detail.** Meanders with oxbow cutoffs and avulsions at interactive rates (MIT). — [Paris 2023](https://aparis69.github.io/public_html/projects/paris2023_Meanders.html). Deltas: pyDeltaRCM (MIT) — [GitHub](https://github.com/DeltaRCM/pyDeltaRCM) — and Landlab `GeoEnthalpyDelta`. — [source](https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/geoenthalpy_delta/geoenthalpy_delta.py)
- **Aquifers.**
  - Vanilla Java 1.18+ aquifers fill cave space from 16×12×16 cells, each with a random center, and lava can appear at or below Y = −10. In datapack world generation, the `aquifers` object can be omitted to disable them, per the current wiki. — [Minecraft Wiki: Aquifer](https://minecraft.wiki/w/Aquifer)
  - Geologically driven water tables exist in goSPL — [groundwater.rst](https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/groundwater.rst) — and Landlab (`groundwater`). — [components dir](https://github.com/landlab/landlab/tree/HEAD/src/landlab/components)
- **Vertical budget.** Custom dimensions allow `min_y` from −2032 to 2031 and `height` up to 4064, both multiples of 16; the default overworld is −64 to 319. — [Minecraft Wiki: Dimension type](https://minecraft.wiki/w/Dimension_type)

### Inferences
- **Memory and time for a block-resolution global pass.** At 16k² = 268 M cells, one float32 field is about 1.07 GB. Elevation, receivers, stack order and area together are about 4.3 GB (arithmetic). That is too much for a typical modpack JVM heap alongside around 190 mods. Depression resolution alone is about 100 s single-threaded (Cordonnier 2019 anchor).
- **Recommended design.**
  1. Route flow on the final LEM grid at 4–16 blocks per cell (1024²–4096² for a 16k world). That is 1–16 M cells, or about 0.4–6 s of depression resolution by linear scaling of the Cordonnier anchor.
  2. Extract rivers as vectors with discharge.
  3. Compute lake levels with Fill–Spill–Merge using the climate runoff.
  4. Compute floodplains with HAND.
  5. Carve channel cross-sections per chunk from vectors, with width and depth from discharge by hydraulic geometry.
  6. Place waterfalls where a river crosses a knickpoint, which in the LEM comes from `LithoLayers`-style erodibility contrasts.
  7. Place deltas and meanders from the vector models.
- **Vanilla aquifers can stay** for cave water once the base terrain comes from the sim. A goSPL-like water-table depth could modulate their levels; whether datapack noise settings expose enough control in 1.21.1 needs checking.
- **The height limit forces a decision.** 4064 blocks maximum and 384 by default mean kilometre-scale relief needs either vertical scaling or a horizontal scale above 1 m per block. Stream-power and diffusion parameters are unit-dependent, so the simulation should run in physical units and map to blocks at the end.

### Gaps
- No priority-flood benchmark specifically at 4k² in a JVM language was found.
- Vanilla 1.21.1 aquifer datapack schema details (as opposed to the current wiki text) were not verified.
- No sourced hydraulic-geometry coefficients (width and depth versus discharge) were gathered.

## 7. Verdict: embeddable libraries versus reference implementations to port versus offline-only, and a realistic compute budget

### Takeaway
Nothing in this space can be dropped into a NeoForge 1.21.1 (Java 21) mod as-is. Every mature engine is Python/Fortran/PETSc/Julia or GPL/EUPL/research-licensed C++. The realistic plan is a pure-Java port of a compact, paper-documented core:
- Braun–Willett / Yuan implicit stream power with deposition;
- priority-flood or MST sinks;
- ADI or implicit diffusion;
- exponential weathering;
- LithoLayers-style stratigraphy;
- a parametric climate (EBM plus LFPM or Smith–Barstad plus Köppen plus ELA).

Permissive (MIT/BSD) code from Landlab and the LIRIS graphics group can be read and ported directly. GPL and Inria codes should be used only from their papers, or offline for validation. On a gaming PC this supports a few minutes of deep-time history on a 256²–512² grid, then a young-landscape pass at 1024²–2048², then per-chunk block amplification.

### Cited Findings
- **Licence facts that constrain bundling:**
  - **GPL-3.0:** fastscapelib, fastscapelib-fortran, goSPL, RichDEM, TopoToolbox v3, Isca, IGM, Undiscovered Worlds, DAGGER.
  - **GPL-2.0:** ExoPlaSim. **GPL-2.0-or-later:** OpenLEM.
  - **Badlands:** LGPL-3.0 on GitHub versus GPL in PyPI metadata.
  - **EUPL-1.2:** SpeedyWeather.
  - **MIT or BSD:** Landlab, fastscape (Python), orographic-precipitation, Sequence, SedFlux, pyDeltaRCM, climlab, Fill–Spill–Merge, the Schott, Argudo and Paris codes, worldengine, scabbard, and the WebGPU port.
  - **Research-only, non-commercial:** Inria analytical-terrains, fastflow and glacial-erosion.
  - Sources: the GitHub and PyPI pages cited in sections 1–6; [IGM](https://github.com/instructed-glacier-model/igm); [Inria licence text](https://gitlab.inria.fr/landscapes/fastflow)
- **GPL linking rule.** From the GPL FAQ: "If a library is released under the GPL … any software which uses it has to be under the GPL or a GPL-compatible license … because the program actually links to the library". — [GPL FAQ #IfLibraryIsGPL](https://www.gnu.org/licenses/gpl-faq.html#IfLibraryIsGPL)
- **GPL separate-program exception.** Programs invoked by simple fork/exec "without intimate communication" are separate programs. — [GPL FAQ #GPLPlugins](https://www.gnu.org/licenses/gpl-faq.html#GPLPlugins)
- **GPL and proprietary systems.** "You cannot incorporate GPL-covered software in a proprietary system". — [GPL FAQ #GPLInProprietarySystem](https://www.gnu.org/licenses/gpl-faq.html#GPLInProprietarySystem)
- **EUPL-1.2 compatibility.** EUPL-1.2 lists GPLv2 and v3, AGPLv3, LGPL, MPL, EPL, CeCILL and OSL as compatible downstream licences. The matrix notes there is "no specific case law" on whether linking creates a derivative work. — [EUPL compatibility matrix](https://interoperable-europe.ec.europa.eu/collection/eupl/matrix-eupl-compatible-open-source-licences)
- **Java native interop.** The Foreign Function & Memory API became final only in JDK 22 (JEP 454); in JDK 21 it was the preview JEP 442. — [JEP 454](https://openjdk.org/jeps/454)
- **GPU limits.** Compute shaders arrived in OpenGL 4.3; macOS supports only OpenGL 4.1, and Apple deprecated OpenGL in 2018. — [Wikipedia: OpenGL](https://en.wikipedia.org/wiki/OpenGL). The MIT GPU codes need OpenGL 4.3. — [MultiScaleErosion](https://github.com/H-Schott/MultiScaleErosion); [glaciers](https://github.com/oargudo/glaciers)
- **Speed anchors** (details and sources in sections 1–6):
  - About 1 µs per cell per step for CPU implicit stream power with routing (Tzathas; FastFlow).
  - Analytical multigrid: 8.2 s at 1024² and 34 s at 2048².
  - GPU implicit: 10 Myr at 512² in 0.5 s (research code).
  - About 100 s per depression solve at 16384² (Cordonnier 2019).
  - goSPL needs at least 16 MPI ranks at 10 km global resolution.
  - ExoPlaSim T21 takes 3.75 min per year on a laptop and needs 50–200 years to equilibrate.
  - SpeedyWeather T31 takes 40–170 s per year.
  - GraphFlood: 83 Mpx took about 20 h.

### Inferences
- **Embeddable as-is:** effectively none for the core. The only native candidates are small permissive C++ units (Fill–Spill–Merge, MIT) called via JNI. JNI means per-OS, per-architecture binaries (Windows, Linux and macOS on x64 and arm64), and FFM would need `--enable-preview` on Java 21. A pure-Java port is simpler.
- **Port from papers (licence-clean):**
  - Braun & Willett 2013 stream-power law;
  - Yuan 2019 erosion–deposition;
  - Cordonnier 2019 MST sink resolver, or plain Barnes 2014 priority-flood;
  - implicit or ADI diffusion;
  - Tzathas 2024 analytical solver as an optional fast final-landscape mode;
  - FastFlow ideas for an optional GL 4.3 compute path (not its code);
  - Braun 2016 regolith front;
  - goSPL's documented duricrust, diagnostic-ice and ELA schemes (from the docs, not the code);
  - LFPM and Smith & Barstad.
- **Port from permissive code:**
  - Landlab `ExponentialWeatherer`, `LithoLayers`, `CarbonateProducer`, `SimpleSubmarineDiffuser`, HAND, `ThresholdEroder`, `ConcentrationTracker`;
  - orographic-precipitation (MIT, FFT);
  - climlab EBM equations;
  - Fill–Spill–Merge;
  - Schott 2024 multi-scale erosion shaders for amplification, with a CPU fallback;
  - Argudo glaciers (SIA);
  - Paris desert dunes, meanders and karst networks.
- **Offline-only** (development-time calibration, validation, or optional external tooling via fork/exec):
  - goSPL, Badlands, fastscape/fastscapelib, Landlab full models;
  - ExoPlaSim, PlaSim, Isca, SpeedyWeather;
  - IGM, GraphFlood, CAESAR-Lisflood, TopoToolbox, RichDEM, FastFlow.
- **Realistic compute budget.** Single-thread times use about 1 µs per cell per step. The "8 threads" figures assume 3–4× effective speedup, which is not measured.

  | Stage | Assumptions | Single-thread | 8 threads |
  |---|---|---|---|
  | A. Deep time, 300 Myr at 256² | Δt = 100 kyr, 3000 implicit steps | ≈3.3 min | ≈1 min |
  | A. Deep time, 300 Myr at 512² | as above | ≈13 min | ≈3–4 min |
  | A. Deep time at 512² | Δt = 250–500 kyr (unconditionally stable; FastFlow shows an implicit 20× Δt matches explicit) | ≈3–5 min | — |
  | B. Young landscape at 1024² | last 2–10 Myr, about 100–200 steps | ≈2–4 min | — |
  | B. Young landscape at 2048² | as above | ≈8–15 min | — |
  | B. Analytical alternative | Tzathas-style | 8 s (1024²), 34 s (2048²) | — |
  | C. Climate | 30–300 parametric snapshots at 128²–512² | well under a minute in total | — |
  | D. Hydrology | on the stage-B grid (1–4 M cells) | about 1–6 s per solve | — |
  | E. Block detail | per chunk, on demand, amplification plus carving from vectors | amortised over world generation | — |

  - Stage A also includes weathering, hillslope diffusion, marine diffusion and stratigraphy logging.
  - Stage C runs EBM, winds, LFPM, Köppen and ELA per snapshot.
  - Stage D covers routing, Fill–Spill–Merge lakes and HAND.

  Total: about 5–15 minutes on a 6–8-core gaming CPU for a 256²–512² history plus a 1024² young landscape. 2048² final detail or finer history grids push past 15 minutes unless the analytical solver or a GL 4.3 compute path (Windows and Linux only) is used.
- **Visualization fits naturally.** Stage A emits a height and facies raster per step, which is what a Dwarf-Fortress-style timelapse needs. Sampling one frame per 1–5 Myr means 60–300 frames of 256²–512², which is cheap.
- **Validate the Java core offline.** Compare hypsometry and chi or steepness statistics against fastscape or Landlab with identical parameters, as Tzathas did against simulation.

### Gaps
- No measured Java or JVM throughput for these kernels. The "within a small factor of C++/Numba" assumption and the multithread speedup are unverified.
- The effect of very large implicit steps (≥250 kyr) on stratigraphic fidelity and river-network stability over hundreds of Myr was not quantified by any source found.
- The GPL-in-Minecraft question (a GPL mod linking a proprietary game) was not researched. Ben would need a licence decision if any GPL code were bundled.
