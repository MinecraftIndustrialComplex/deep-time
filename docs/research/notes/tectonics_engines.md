# Tectonic / lithosphere-evolution engines and event-history → 3D lithology tools (for a NeoForge 1.21.1 geologic-history worldgen mod)

Research date 2026-09-27. Licence, release and commit data were read from the GitHub and PyPI APIs that day. Two quick experiments were run in this session: a PyPlatec benchmark and an in-memory pyGPlates synthetic-plate test. Their scripts are in `/tmp/claude-1000/-home-greencheetah-Projects-mic/f981c0d3-5144-49ec-8686-2946d755872c/scratchpad/r1/` (`bench_platec.py`, `test_pygplates.py`). Note: this session's WebSearch budget ran out partway through, so discovery of post-2019 graphics papers is incomplete (see Gaps under Q3).

## Q1. Kinematic/reconstruction tools (GPlates, pyGPlates, GPlately, PlateTectonicTools): what they compute, pip status, can they drive a synthetic planet, and how worldbuilders use them

### Takeaway
pyGPlates is now pip-installable: 1.0.0 shipped on PyPI in March 2025 with wheels for CPython 3.8–3.13 on Linux, macOS and Windows. Its API can build a fictional planet's plates and rotation model entirely in memory; I verified this in this session. But it is a reconstruction engine, not a simulator. It moves the plates you give it along the rotations you give it, and computes velocities, resolved topologies and deforming networks. Deciding what the plates do (rifting, subduction initiation, collision, new ocean crust) is still your job, and worldbuilders do all of it by hand. The whole EarthByte/GPlates stack is GPL-2.0-only and roughly 240 MB of Python plus native code. That makes it an offline authoring and validation tool, or a design template to reimplement in Java. It cannot be linked into a Minecraft mod.

### Cited Findings
**GPlates and pyGPlates (core)**
- GPlates is "a program for visualising and manipulating plate tectonic reconstructions"; "PyGPlates is a Python package containing the core plate reconstruction functionality in GPlates". Both live in one C++ repo, last pushed 2026-09-26, with 152 stars. — [GPlates/GPlates](https://github.com/GPlates/GPlates)
- Licence: GPL version 2 only. The COPYING notice says "the only valid and applicable version of the GPL, as far as the GPlates program is concerned, is _this_ particular version of the license (version 2, dated June 1991, not any later version)". The same text is the licence field of pyGPlates on PyPI. — [GPlates repo](https://github.com/GPlates/GPlates), [pygplates on PyPI](https://pypi.org/project/pygplates/)
- Releases: GPlates 2.5 (2024-04-15) and PyGPlates 1.0.0 (2025-03-28, GitHub release). No GPlates 3.x release is listed. — [GPlates releases](https://github.com/GPlates/GPlates/releases)
- pip status: `pygplates` 1.0.0 on PyPI (uploaded 2025-03-26; only 1.0.0rc1 and 1.0.0 are listed), `requires_python >=3.8`. Wheels exist for cp38–cp313 on manylinux x86_64/aarch64, macOS x86_64/arm64 and win_amd64. Wheel sizes are about 31 MB (Windows) to 73 MB (Linux). — [PyPI JSON / project page](https://pypi.org/project/pygplates/)
- What pyGPlates 1.0 computes, per its documentation:
  - Reconstruction: reconstructing regular features, motion paths and flowlines, plus reverse reconstruction.
  - Topologies: resolving topological plate boundaries and topological networks (deforming), rigid blocks, and strain rates in the triangulation.
  - Rotations: finite rotations, plate circuit paths, net rotation, crossover detection and synchronisation, equivalent total and stage rotations.
  - Velocities, including in dynamic plates.
  - Installation via conda or pip. — [pyGPlates docs](https://www.gplates.org/docs/pygplates/)
- Programmatic feature creation. `pygplates.Feature` has factory methods `create_reconstructable_feature()`, `create_topological_feature()`, `create_topological_network_feature()`, `create_tectonic_section()`, `create_flowline()`, `create_motion_path()` and `create_total_reconstruction_sequence()`. The last one builds rotation sequences, i.e. the contents of a `.rot` file. — [pygplates.Feature](https://www.gplates.org/docs/pygplates/generated/pygplates.Feature.html)
- `pygplates.RotationModel(rotation_features, …)` is created "from rotation feature collection(s) and/or rotation filename(s)". It accepts "a rotation feature collection, or rotation filename, or rotation feature, or sequence of rotation features", so no files are needed. — [pygplates.RotationModel](https://www.gplates.org/docs/pygplates/generated/pygplates.RotationModel.html)
- **Measured in this session:** pyGPlates 1.0.0 on Python 3.12, Linux, i7-1185G7 laptop.
  - Setup: I built a synthetic plate 101 (a 30°×30° polygon) and a synthetic total-reconstruction sequence (Euler pole at 60°N 20°E, 1°/Myr, samples every 10 Myr from 0 to 200 Ma), entirely in memory.
  - Reconstructed centroid: (15.1, 15.0) at 0 Ma, (20.7, 51.2) at 50 Ma, (39.1, 85.7) at 100 Ma.
  - Velocity at (15°N, 15°E) at 50 Ma: 7.88 cm/yr.
  - Timing: import took 0.51 s; the whole script took 0.53 s.
  - Installed footprint: 78 MB `pygplates/` plus 161 MB `pygplates.libs/` (about 239 MB).
  - On NixOS the Linux wheel also needed system `libstdc++`, `zlib`, `libGL`, `expat` and `glib` on the library path before it would import. — software from [pygplates on PyPI](https://pypi.org/project/pygplates/)

**GPlately, PlateTectonicTools, plate-model-manager**
- GPlately "enables the reconstruction of data through deep geologic time (such as points, lines, polygons, and rasters), the interrogation of plate kinematic information (plate velocities, rates of subduction and seafloor spreading), the rapid comparison of multiple plate motion models, and the plotting of reconstructed output data on maps". Its tools are "parallel-safe"; it installs via pip, conda or Docker. It can also make motion paths and flowlines. — [GPlates/gplately README](https://github.com/GPlates/gplately)
- GPlately licence and activity: GPL-2.0, 95 stars, pushed 2026-09-21. v2.0.0 (2025-07-14) changes:
  - Dropped Python 3.5–3.8 and deprecated `gplately.pygplates` (users now import `pygplates` directly).
  - Added the plate-model-manager, a CLI, PyGMT plotting and Docker images. — [GitHub](https://github.com/GPlates/gplately), [v2.0.0 release notes](https://github.com/GPlates/gplately/releases/tag/v2.0.0)
- GPlately dependencies (PyPI 2.0.0, Python ≥3.9): numpy, scipy, shapely, matplotlib, cartopy, pooch, tqdm, netcdf4, rasterio, geopandas, plate-model-manager, pyyaml, pygmt, rioxarray, pygplates≥1.0.0. — [gplately on PyPI](https://pypi.org/project/gplately/)
- Seafloor age grids (GPlately `SeafloorGrid` / `TopologySeafloorGrid`):
  - Inputs: a rotation model, topology features and continental polygons.
  - Method: at the maximum time an "icosahedral point mesh fills the ocean basin". At each time step new points emerge from mid-ocean ridges (`ridge_time_step` default 1 Myr, `ridge_sampling` default 0.5°). Points are moved with `ReconstructByTopologies`, and grids are produced at 0.1–1° spacing.
  - Cost: "gridding can take a while (depending on the number of CPUs used)". `nprocs` defaults to all CPUs but one, and runs can resume from checkpoints.
  - Outputs: netCDF `SEAFLOOR_AGE` and `SPREADING_RATE` grids. — [GPlately SeafloorGrid notebook](https://gplates.github.io/gplately/latest/notebook-html/10-SeafloorGrids.html)
  - The method is adapted from Williams et al. 2021 (Geoscience Frontiers 12(2):769–780). — [GPlately README](https://github.com/GPlates/gplately)
- PlateTectonicTools (`ptt`) is "a collection of common plate tectonic functionality … primarily built on top of the pyGPlates Python library". Its notebooks compute average seafloor spreading rates of mid-ocean ridges and subducting rates of paleo rasters along subduction zones, and predict CO2 from age and bottom-water temperature. The README still says it "is only in its initial stages". — [EarthByte/PlateTectonicTools](https://github.com/EarthByte/PlateTectonicTools)
- PTT status: GPL-2.0; last PyPI release 0.4.1 on 2022-11-16; no default-branch commits since 2025-01-01 (GitHub API). — [PyPI](https://pypi.org/project/PlateTectonicTools/), [GitHub](https://github.com/EarthByte/PlateTectonicTools)
- Reference paper for the stack: Mather et al., "Deep time spatio-temporal data analysis using pyGPlates with PlateTectonicTools and GPlately", Geoscience Data Journal. — [doi:10.1002/gdj3.185](https://doi.org/10.1002/gdj3.185)
- plate-model-manager is "like Conda for plate models" (GPL-2.0; version 1.3.2 uploaded 2026-05-24). — [GitHub](https://github.com/GPlates/plate-model-manager), [PyPI](https://pypi.org/project/plate-model-manager/)

**Use on non-Earth planets**
- Worldbuilding Pasta, "Part V Supplement: Tectonics: Using GPlates" (2020):
  - Direction: the history is run "from the past forwards", unlike GPlates' usual backward reconstruction.
  - Rotation file: a hand-edited `.rot` with one line per plate position per time (plate ID, time, pole lat/lon, angle, conjugate plate, comment). Plate ID 0 is the fixed axis and 1 is for static reference features.
  - Features: cratons, continents, ocean crust polygons, rifts, subduction zones, island arcs and flowlines.
  - Time steps: 50 Myr normally, 10 Myr during complex collisions, starting at 2000 Ma. Speeds are checked with the Kinematics tool.
  - Everything is manual: adding subduction zones and arcs each step, splitting plates by editing vertices, tracing ocean-crust polygons, and handling collisions and accreted terranes.
  - The post does not mention pyGPlates or any scripting. — [worldbuildingpasta Part V supplement](https://worldbuildingpasta.blogspot.com/2020/06/an-apple-pie-from-scratch-part-v.html)
- Worldbuilding Pasta, "Part VIIc: Constructing Global Terrain" (2023):
  - Turning the GPlates history into terrain is manual: elevations are hand-painted in Paint.net/GIMP on an exponential scale in 25 m units, up to 8,100 m.
  - Maps are 3,600×1,800 px, reprojected to regional Hammer projections at 17,300×8,650 px (about 2 km/pixel). Erosion uses Wilbur or goSPL; the goSPL part is flagged as a work in progress.
  - The author calls it "more art than science". — [worldbuildingpasta Part VIIc](https://worldbuildingpasta.blogspot.com/2023/03/an-apple-pie-from-scratch-part-viic.html)
- Research precedent on synthetic planets: Mallard et al. 2016 took plate layouts from 3D convection models and "manually build plate polygons using GPlates through a careful analysis of the surface velocity, horizontal divergence, viscosity, synthetic sea-floor age and temperature field". They note "the precise determination of the plate layout for one snapshot is very time-consuming". — [Mallard et al. 2016, Nature 535:140 (PDF)](https://web.gps.caltech.edu/~jstock/161/PapersToRead/nature17992-Mallard.pdf)

**What the licences mean for a mod**
- GPL FAQ, plug-ins for a nonfree program: if the GPL plug-in and the nonfree main program "form a single combined program … that combination … would violate the GPL", unless the plug-in's licence adds an exception. — [GPL FAQ #GPLPluginsInNF](https://www.gnu.org/licenses/gpl-faq.html#GPLPluginsInNF)
- GPL FAQ, exceptions for other people's code: "if you want to use parts of other GPL-covered programs by other authors in your code, you cannot authorize the exception for them. You have to get the approval of the copyright holders". — [GPL FAQ #GPLIncompatibleLibs](https://www.gnu.org/licenses/gpl-faq.html#GPLIncompatibleLibs)
- GPL FAQ, separate programs: "pipes, sockets and command-line arguments are communication mechanisms normally used between two separate programs". Intimate exchange of complex internal data structures could still make them one program. — [GPL FAQ #MereAggregation](https://www.gnu.org/licenses/gpl-faq.html#MereAggregation)
- GPL FAQ, v2 vs v3: GPLv2 and GPLv3 "are not compatible"; code under "version 2 or later" is compatible with GPLv3. — [GPL FAQ #v2v3Compatibility](https://www.gnu.org/licenses/gpl-faq.html#v2v3Compatibility)

### Inferences
- **pyGPlates can drive a fictional planet**, because nothing in it is Earth-specific: plate IDs, Euler poles, polygons and topologies are just data. It supplies none of the "brain", though:
  - where plates go;
  - when a new ridge or trench appears;
  - collision detection and suturing;
  - creating ocean-crust polygons;
  - updating topologies as boundaries migrate.
  
  A procedural generator would have to write all of that (the worldbuilder does it by hand in the WBP workflow). Topologies are the hardest part to generate procedurally, because the boundary sections must stay consistent through time.
- **Rigid-plate kinematics are cheap.** The in-memory test ran in milliseconds. The expensive parts are data management and optional gridding such as SeafloorGrid. Reimplementing finite rotations, stage poles, plate hierarchies and velocity as ω×r in Java is small and well-defined work.
- **The mod cannot link it.** GPL-2.0-only plus non-free Minecraft means neither pyGPlates nor any GPlates code can be linked into the mod jar: an exception would have to come from the University of Sydney, NGU and Caltech. Calling it as a separate process is normally a separate program, but it would make every player install Python, the ~240 MB wheel and system GL/glib libraries (GPlately adds cartopy, rasterio, geopandas and pygmt).
- **Recommended roles for this stack:**
  - A dev-time or designer-time tool: prototype and validate a synthetic plate history, and check plate velocities or seafloor-age statistics.
  - An **export target**: the mod could write `.rot`/`.gpml` for its generated worlds so geology-minded players can open them in GPlates. Reading and writing a file format does not combine programs.
  - A conceptual template to reimplement in Java.
- SeafloorGrid's approach (seed points at ridges, track them with plate motion, grid their ages) is essentially the same idea as Cortial et al.'s ridge resampling (Q3). Either is a sound reference algorithm for computing oceanic crust age in a Java driver.

### Gaps
- I found no public tool that *procedurally* generates GPlates rotation files and topologies for a fictional planet; only manual workflows turned up. The WebSearch budget was exhausted before a broader search.
- GPlates desktop 2.5 install size, and whether a GPlates 3.0 is planned, were not checked.
- I found no runtime figures for a full SeafloorGrid run.

## Q2. Forward geodynamic codes (ASPECT, Underworld 2/3, LaMEM, CitcomS, pTatin3D, Rayleigh, StagYY): can any produce plate-like tectonics on a synthetic planet in minutes on a gaming PC, and could a very low-resolution run be a driver?

### Takeaway
No. The published 3D spherical runs that produce Earth-like plates (both StagYY) needed a supercomputer:
- Coltice et al. 2019: 23 km average resolution, about 50 million cells, 1.5 Gyr simulated over nine months and 390,200 time steps.
- Mallard et al. 2016: 45 km resolution, and still limited by compute to a Rayleigh number 10–50× below Earth's.

All the open codes are PETSc/deal.II-class HPC software with copyleft or permissive licences but huge footprints. A "very low-resolution" run cheap enough for a player would not resolve plate-boundary localisation (inference). These codes are useful for parameter ranges and statistical validation, not as runtime drivers.

### Cited Findings
- **Coltice et al. 2019 ("What drives tectonic plates?", Science Advances 5:eaax4295), StagYY on a yin-yang grid.**
  - High-vigour model: Rayleigh number 10^7; surface yield stress 61 MPa; a "14-km weak top layer in oceanic areas"; continental rafts; two antipodal 500-km-thick dense basal provinces; Pangea-like start.
  - "The resolution is 23 km on average (about 50,000,000 volumetric cells) and is refined to a radial resolution of <10 km in boundary layers."
  - "We obtained a 1500–million year (Ma) duration dynamic evolution of this system (after nine full months and 390,200 time steps on a parallel supercomputer). For computational reasons, other parameter combinations could not be tested."
  - Ten further lower-vigour models used "a Rayleigh number of 10^6 and an average resolution of 45 km". — [Coltice et al. 2019 full text (Europe PMC)](https://europepmc.org/article/PMC/PMC6821462); [Science Advances](https://www.science.org/doi/10.1126/sciadv.aax4295)
- **Mallard et al. 2016 (Nature 535:140–143), StagYY, 3D spherical, Boussinesq.**
  - Ra 10^6 "is 10–50 times lower than is expected for Earth, and produces a top boundary layer that is 300-km thick. We were limited to this value of Ra because of the computational power required to solve for convection with large viscosity variations."
  - "The average resolution is 45 km laterally and vertically for all of the models."
  - Yield stresses tested: 100, 150, 200 and 250 MPa; "yield stress values that produce plate-like behaviour are between 100 MPa and 350 MPa".
  - The 100 MPa model had more than 100 plates.
  - Simplifications: no continents or chemical piles.
  - Earth's plate size split (small vs large plates) is around 10^7.6 km².
  - StagYY was "provided" by co-author P. J. Tackley. — [Mallard et al. 2016 PDF](https://web.gps.caltech.edu/~jstock/161/PapersToRead/nature17992-Mallard.pdf); [Nature](https://www.nature.com/articles/nature17992)
- **ASPECT** (C++ parallel finite elements): GPL "version 2 of the License, or (at your option) any later version". v3.1.0 released 2026-09-19 (v3.0.0 was 2024-12-11); 311 stars. A Docker image exists and dependencies are covered in the manual's installation section. — [geodynamics/aspect](https://github.com/geodynamics/aspect), [releases](https://github.com/geodynamics/aspect/releases)
- **Underworld 2** is "a parallel, particle-in-cell, finite element code for Geodynamics". Its LICENSE.md says "All Underworld source code is released under the LGPL-3" (examples are CC BY 4.0). v2.17.3 was released 2026-05-14. — [underworldcode/underworld2](https://github.com/underworldcode/underworld2)
- **Underworld 3** has the same LGPL-3 licence text. v3.0.0 came out 2026-03-22 and v3.1.0 on 2026-07-08. It installs via pixi and has binder demos. — [underworldcode/underworld3](https://github.com/underworldcode/underworld3)
- **LaMEM** is MIT-licensed ("Copyright (c) 2011-2026 JGU Mainz, Boris Kaus, Anton Popov"). It is a 3D parallel marker-in-cell staggered finite-difference code using PETSc solvers, also installable through LaMEM.jl. v3.2.0 was released 2026-09-22. — [UniMainzGeo/LaMEM](https://github.com/UniMainzGeo/LaMEM)
- **CitcomS**: GPL-2.0, "a finite element code designed to solve compressible thermochemical convection problems relevant to Earth's mantle". Last push 2025-09-12, with only 2 commits since 2025-01-01. — [geodynamics/citcoms](https://github.com/geodynamics/citcoms)
- **Rayleigh**: GPL-3.0, Fortran, described as "Pseudo-spectral MHD"; pushed 2026-09-20. — [geodynamics/Rayleigh](https://github.com/geodynamics/Rayleigh)
- **pTatin3D**: a GPL-3.0 copy exists on GitHub (last pushed 2021), plus a 2026 GPL-3.0 fork (`ptatin-gene`). It is described as "a EU Flagship code to simulate geodynamic processes". — [richardhaslam/pTatin3d](https://github.com/richardhaslam/pTatin3d), [laetitialp/ptatin-gene](https://github.com/laetitialp/ptatin-gene), [boriskaus/pTatinIO.jl](https://github.com/boriskaus/pTatinIO.jl)
- **A cheap "physics-flavoured" alternative, tectonics.js v2.** Crust age → density → "subductability". The velocity field is smoothed with "the diffusion equation" to "approximate the 2d motion of the mantle". Plates are identified by "repeatedly performing the Flood Fill algorithm" (image segmentation of the velocity field), and each segment's angular velocity is its average over the mask. — [tectonics.js version-2 post](https://github.com/davidson16807/tectonics.js/blob/master/_posts/2017-9-16-version2.md)

### Inferences
- **Order of magnitude.** The mantle is about 9×10^11 km³, so the "low-resolution" research setup (45 km cells) is roughly 10^7 cells. Coltice's 23 km run (about 5×10^7 cells) took 9 months on a supercomputer.
- **What "minutes on a gaming PC" would take.** You would need about 10^3–10^5× less work: cells of roughly 150–250 km, and far fewer time steps. At that resolution the grid is coarser than the lithosphere, which is about 100 km thick. Mallard already reports a 300-km-thick lid at Ra 10^6. Localised weak plate boundaries (tens of km wide) would not form, and you would get a few sluggish, blobby "plates", not Earth-like tectonics with a distribution of 7 large plates plus fractal small plates.
- **Output would still need processing.** Plates have to be extracted from the flow field: Mallard traced them by hand in GPlates, and tectonics.js uses flood-fill segmentation. Continents, arcs and crust age all need extra bookkeeping.
- **Best use of forward codes for this mod:**
  - A source of *targets* to calibrate a procedural driver against, e.g. plate-size frequency (7 large plus fractal small, split near 10^7.6 km²), typical surface speeds (~4 cm/yr in Coltice), and the yield-stress window.
  - Offline generation of synthetic velocity-field libraries (research-grade effort) whose distilled kinematics (pole sequences) could ship as data. Check the licensing of any such derived datasets.
- LaMEM (MIT) is the only permissively licensed full solver. Even so, it depends on PETSc and parallel infrastructure, and nothing about it suits running inside a Minecraft JVM.

### Gaps
- Neither the Coltice supplement nor the Mallard main text reports wall-clock time for the low-Ra (10^6, 45 km) runs.
- I did not find any published plate-like convection run on consumer hardware at minute scale (search budget exhausted). GPU Stokes solvers were not investigated.
- The StagYY licence and distribution terms are not documented in the sources read; it was only "provided" by its author.

## Q3. Game/graphics procedural tectonics (PlaTec/pyplatec, WorldEngine, tectonics.js, Cortial et al. 2019 and implementations, Undiscovered Worlds, Experilous, Cordonnier et al. 2016/2018, 2020–2026 work): what they compute, and which track rock type, age and strata

### Takeaway
Cortial et al. 2019 ("Procedural Tectonic Planets") is the most complete and geologically reviewed procedural model:
- a sphere sampled at 60k–500k points;
- each sample carries crust type, thickness, elevation, oceanic age, ridge direction, orogeny type/age and fold direction;
- subduction, collision/terrane accretion, seafloor generation and Poisson-triggered rifting;
- 0.19–1.9 s per 2-Myr step on a 2016 desktop.

It has no official code, and the three GitHub re-implementations have no licence.

PlaTec (LGPL C++ with Python bindings, revived in 2025) is the only maintained embeddable library. It is 2D, unitless and outputs only height, plate IDs and crust age; I measured 10 s for a 512² world and 37 s for 1024². Only tectonics.js (7 rock-mass pools per cell, CC BY 4.0) and tectonics.cpp (strata × 16 minerals, now CC BY-NC-SA) track rock types.

Cordonnier 2016 (uplift map → stream-power terrain) and 2018 (folded, layered crust under collision with visible folded strata) are the key precedents for tectonics feeding terrain and strata. Undiscovered Worlds treats mountains and volcanoes as separate feature maps rather than simulating plates; a 2026 fork vendors PlaTec. Experilous-style generators, known here via Red Blob Games, apply single-step plate-boundary rules. Both are heuristic.

### Cited Findings
**Cortial, Peytavie, Galin & Guérin 2019 (Computer Graphics Forum 38(2), Eurographics 2019)**
- **Model.** Plates are point-sampled crust on a sphere, sampled "on Fibonacci sampling of the sphere" and triangulated. The processes are subduction (including forced subduction of terranes), continental collision, oceanic crust generation and plate rifting.
- **Per-sample parameters (their Table 1):** crust type (oceanic or continental), thickness e, elevation z, oceanic crust age a_o, local ridge direction r, orogeny type o (subduction/Andean vs collision/Himalayan), orogeny age a_c, and local fold direction f.
- **Control.** The user steers plate motion, can trigger rifting and "may pause the simulation at any time". — [Cortial et al. 2019 PDF](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf); [doi:10.1111/cgf.13614](https://doi.org/10.1111/cgf.13614)
- **Process details.**
  - Two converging oceanic plates "always lead to the subduction of the older plate". Subduction uplift of the overriding plate is u = u0·f(distance to front)·g(relative speed)·h(elevation).
  - Collisions are discrete events: when interpenetration exceeds a threshold ("300 km in our implementation"), a terrane is detached and attached to the other plate.
  - Rifting splits a plate into 2–4 sub-plates as a Poisson process.
  - Oceanic crust is generated between diverging plates, with a "global resampling every 10–60 iterations".
  - Every step applies continental erosion, oceanic dampening and trench sediment accretion. — [same](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf)
- **Constants (Appendix A):**
  - Time step δt = 2 My; planet radius R = 6,370 km.
  - Elevations: highest ridge −1 km, abyssal plains −6 km, trenches −10 km, highest continental altitude 10 km.
  - Subduction distance 1,800 km; collision distance 4,200 km; maximum plate speed 100 mm/yr.
  - Rates: oceanic damping 0.04 mm/yr, continental erosion 0.03 mm/yr, sediment accretion 0.3 mm/yr, subduction uplift 0.6 mm/yr. — [same](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf)
- **Performance** (C++, Intel i7-6700K at 4 GHz, 16 GB, GTX 1080). Crust on the CPU, with elevation multithreaded; amplification on the GPU; rendering at 37–145 Hz. Default: "500 000 point samples … for a 6 370 km radius planet (≈ 35 km resolution)". Table 2, average seconds per step (40 plates, land ratio 0.3):

  | Resolution (samples) | Regular step (subduction + collision + elevation) | Oceanic crust generation | Plate rifting |
  |---|---|---|---|
  | 102 km (60k) | 0.19 s | 0.58 s | 0.23 s |
  | 78 km (100k) | 0.28 s | 1.22 s | 0.21 s |
  | 50 km (250k) | 1.24 s | 3.58 s | 1.50 s |
  | 35 km (500k) | 1.90 s | 13.10 s | 7.70 s |

  Crust generation and rifting run "every 20 to 120 My, depending on maximum plate speed". Elsewhere the authors say "complex planets can be modeled in a few minutes" and "the crust model has a resolution ranging from 50 km to 500 km". — [same](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf)
- **Amplification.** The coarse crust drives either procedural landforms (e.g. 3D Gabor noise oriented by ridge direction, to create transform-fault fabric) or exemplar-based amplification from SRTM90 tiles chosen by orogeny type and age and aligned with fold direction (the paper says 19 exemplar sets). — [same](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf)
- **Validation.** In a 2-alternative user study the model received 97 votes against 47 for a fractal model (p < 0.4%), and a tectonics expert reviewed it (N. Coltice is thanked). — [same](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf)
- **Stated limitations:**
  - rifting "progressively tears the crust";
  - "more forced subduction events than expected";
  - no passive margins or shallow shelves;
  - no hot spots ("could be easily implemented … as specific sample points drifting across plates");
  - no drainage networks. — [same](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf)
- **Community implementations, none licensed:**
  - robertoost/RNGalaxy: Unity/C#, "A procedural tectonic planet simulator", Fibonacci sampling plus KD-tree, 8 stars, last push 2021. — [GitHub](https://github.com/robertoost/RNGalaxy)
  - AurelienBesnier/ProceduralPlanets: C++20/Qt5/CGAL, 2023. — [GitHub](https://github.com/AurelienBesnier/ProceduralPlanets)
  - gubace/Procedural-Tectonic-Planets: an OpenGL student project citing the Cortial PDF, 2025-12. — [GitHub](https://github.com/gubace/Procedural-Tectonic-Planets)
- **No licence means no permission.** Code with no licence "generally means you have no permission from the creators of the software to use, modify, or share the software". — [choosealicense.com/no-permission](https://choosealicense.com/no-permission/)

**PlaTec / plate-tectonics / PyPlatec and WorldEngine**
- Viitanen 2012 thesis (Helsinki Metropolia): PlaTec is a 2D terrain generator that uses a simplified plate-tectonics model to make heightmaps in real time. It starts from a flat fractal map, splits it into plates and moves them. Continental collisions build mountain belts; oceanic crust collisions give coastal ranges or island chains. This description comes from the abstract as indexed; the PDF itself returned HTTP 403. — [Semantic Scholar listing](https://www.semanticscholar.org/paper/Physically-Based-Terrain-Generation-:-Procedural-Viitanen/7a7496190f5ba295d1c78ad3a73fe28398582939); [thesis PDF URL](https://www.theseus.fi/bitstream/handle/10024/40422/Viitanen_Lauri_2012_03_30.pdf)
- C API:
  - `platec_api_create(seed, width, height, sea_level, erosion_period, folding_ratio, aggr_overlap_abs, aggr_overlap_rel, cycle_count, num_plates)`;
  - `platec_api_step`, `platec_api_is_finished`;
  - `platec_api_get_heightmap`, `platec_api_get_platesmap`, `platec_api_get_agemap`;
  - per-plate velocity unit vectors.
  
  So it outputs height, plate index and **crust age** only, with no rock types. — [platecapi.hpp](https://github.com/Mindwerks/plate-tectonics/blob/main/src/platecapi.hpp)
- The lithosphere header requires a "square height map" whose side is "a power of two" and notes that "In a 512*512 map that means 5000-13000 collisions" per step. — [lithosphere.hpp](https://github.com/Mindwerks/plate-tectonics/blob/main/src/lithosphere.hpp)
- The licence is LGPL in every source, but the version varies:
  - file headers say LGPL "version 2.1 of the License, or (at your option) any later version";
  - GitHub reports LGPL-3.0;
  - PyPI `PyPlatec` says LGPL-3.0-or-later. — [Mindwerks/plate-tectonics](https://github.com/Mindwerks/plate-tectonics), [PyPlatec on PyPI](https://pypi.org/project/PyPlatec/)
- Maintenance:
  - plate-tectonics v1.4.3 was released 2025-11-18, with 65 commits since 2025-01-01. The Python bindings now live in-repo (`pybindings/`), and the old `Mindwerks/pyplatec` repo is archived.
  - PyPlatec 1.4.3 wheels are tiny: about 0.62–0.65 MB on Linux, 131 KB on macOS and 53 KB on Windows. — [plate-tectonics](https://github.com/Mindwerks/plate-tectonics), [pyplatec (archived)](https://github.com/Mindwerks/pyplatec), [PyPI](https://pypi.org/project/PyPlatec/)
- **Measured in this session:** PyPlatec 1.4.3 on CPython 3.12, i7-1185G7 laptop, single thread. Parameters: seed 3, sea_level 0.65, erosion_period 60, folding_ratio 0.02, aggr_overlap_abs 1,000,000, aggr_overlap_rel 0.33, 2 cycles, 10 plates.

  | Map size | Steps | Wall time | Height range (unitless) |
  |---|---|---|---|
  | 512×512 | 1,042 | 10.3 s | 0.04–18.9 |
  | 1024×1024 | 1,144 | 36.9 s | 0.01–16.6 |

  Peak RSS was about 195 MB. — software from [PyPlatec on PyPI](https://pypi.org/project/PyPlatec/)
- **WorldEngine.** "Worlds are generated using plate simulations, erosion, rain shadows, Holdridge life zones model and plenty of other phenomenons", using pyplatec "to get proper mountain chains".
  - MIT licence, 1,071 stars.
  - v0.20.0 was released 2025-11-18, the first release since v0.19.0 in 2016; 34 commits since 2025-01-01; supports Python 3.9–3.14.
  - Worlds are saved as protobuf or HDF5, and a Java loader exists (`worldengine-java`, MIT, last pushed 2015).
  - The README example is 2048×2048 with 25 plates, noting "it could take a few minutes". — [Mindwerks/worldengine](https://github.com/Mindwerks/worldengine), [PyPI](https://pypi.org/project/worldengine/), [worldengine-java](https://github.com/Mindwerks/worldengine-java)

**tectonics.js and tectonics.cpp (Carl Davidson)**
- tectonics.js is "3d plate tectonics in your web browser": CC-BY-4.0, 233 stars, last push 2020-08-22. — [davidson16807/tectonics.js](https://github.com/davidson16807/tectonics.js)
- Per-cell rock model (`RockColumn`): `displacement`, `sediment`, `sedimentary`, `metamorphic`, `felsic_plutonic`, `felsic_volcanic`, `mafic_volcanic`, `mafic_plutonic` and `age`. The felsic family is a "conserved" set of pools; mafic rock and age are not conserved. — [RockColumn.js](https://github.com/davidson16807/tectonics.js/blob/master/noncompiled/models/lithosphere/RockColumn.js)
- `Crust` stores these as Float32 rasters, one per pool (structure-of-arrays). Its comments say:
  - "felsic" is "the buoyant, unsubductable igneous component … AKA 'sial'";
  - "mafic" is "the denser, subductable igneous component … AKA 'sima'";
  - mafic "never undergoes conversion to the other rock types" because "it's not performant", among other reasons. — [Crust.js](https://github.com/davidson16807/tectonics.js/blob/master/noncompiled/models/lithosphere/Crust.js)
- Davidson's critique of random plates: generating "a series of plates at random … is how its done in virtually every other non-academic tectonics simulator I've seen, and *it's wrong*. There is simply no way you can reproduce Earth's age distribution this way". — [version-2 post](https://github.com/davidson16807/tectonics.js/blob/master/_posts/2017-9-16-version2.md)
- tectonics.cpp is "a header-only C++ library that's intended to supplant the backend to tectonics.js". Licence: "As of completing major milestones in May 2025, permission to use all code going forward is provided under the Creative Commons Attribution-NonCommercial-Sharealike 4.0 license … Commercial licensing is also available by prior agreement". The repo had 100 or more commits since 2025-01-01 and was last pushed 2026-07-29. — [davidson16807/tectonics.cpp](https://github.com/davidson16807/tectonics.cpp)
- tectonics.cpp rock model:
  - A hierarchy: density tower → crust → plate → formation / column → stratum → particle → clast → grain → mineral. "Columns and strata have exactly one representation within model state, albeit at a very coarse resolution." — [inc/model/rock/README.md](https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/README.md)
  - Strata exist only where they serve a functional distinction: a "bedrock" layer "only created upon the rifting of plates", a "sediment" layer and a "sedimentary" layer. — [stratum/README.md](https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/stratum/README.md)
  - Minerals track mass, weathering, and grain-size classes (amorphous, extrusive, intrusive, crystalline), stored in half-byte bins. — [mineral/README.md](https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/mineral/README.md)
  - The design note sets the maximum depth of interest at 12 km (the Kola borehole), says "Lowest acceptable precision for a layer is probably Minecraft level, 1m blocks", uses 16 mineral mass pools, and plans for "upwards of 40k cells per grid". — [designing-stratigraphic-model.md](https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/designing-stratigraphic-model.md)
  - Crust types include `blocks`, "floats mapping to layers of constant height … like Minecraft blocks"; Airy isostatic displacement headers are included. — [crust/README.md](https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/crust/README.md), [crust/ directory](https://github.com/davidson16807/tectonics.cpp/tree/master/inc/model/rock/crust)

**Undiscovered Worlds, Experilous-style, and small projects**
- Undiscovered Worlds (Jonathan Hill), GPL-3.0:
  - The classic version makes global maps and regional maps at "approximately 1 pixel to 1 km".
  - It "does not treat mountain ranges as normal elevation". Mountains come from a separate map of peak elevation above surrounding land (50 m increments), and volcanoes from a separate map (shield vs strato, active vs extinct).
  - Imported maps are 2048×1025. — [Undiscovered_Worlds_Classic README](https://github.com/JonathanCRH/Undiscovered_Worlds_Classic)
  - The new globe version is "early experimental": there are no regional maps yet, and "the same seed does not always produce the same planet". It has had no commits since 2025-01-01. — [JonathanCRH/Undiscovered_Worlds](https://github.com/JonathanCRH/Undiscovered_Worlds)
- The fork `lukabergs/UndiscoveredWorlds` (0 stars, active 2026-09) adds "an internal plate-tectonics module". Its `src/simulations/geology/plate_tectonics/` files (bounds, geometry, heightmap, lithosphere, mass, movement) mirror the PlaTec library. — [lukabergs/UndiscoveredWorlds](https://github.com/lukabergs/UndiscoveredWorlds)
- Red Blob Games' planet generator (2018) is "a one-week experiment" that follows "the approach of Andy Gainey and others":
  - N random Voronoi regions (10–50) become plates, each with a random direction vector;
  - boundary cells are classified by relative motion, with Δdistance < −0.75 giving mountains;
  - it is "not time-evolved", i.e. single-step. — [redblobgames.com/x/1843-planet-generation](https://www.redblobgames.com/x/1843-planet-generation/)
- FioDev/Procedural-Tectonics (Unity, 2023, no licence) simulates "families of grouped tectonic plates". By default two plates collide over "400–1,000 simulated 'years'". — [GitHub](https://github.com/FioDev/Procedural-Tectonics)

**Cordonnier et al.: tectonic uplift feeding terrain, and folded strata**
- Cordonnier et al. 2016 (CGF 35(2)): "Given a user‐painted uplift map, we generate a stream graph … Our approach relies on the stream power equation … By combining crust uplift and stream power erosion we generate large realistic terrains at a low computational cost", then blends landform kernels into a DEM. — [doi:10.1111/cgf.12820](https://doi.org/10.1111/cgf.12820) (abstract via [OpenAlex](https://api.openalex.org/works/doi:10.1111/cgf.12820))
- Cordonnier et al. 2018 ("Sculpting Mountains", IEEE TVCG):
  - "user is provided hands-on control on the shape and motion of tectonic plates, represented using a new geologically-inspired model for the Earth crust";
  - "The model captures their volume preserving and complex folding behaviors under collision";
  - "It generates a volumetric uplift map representing the growth rate of subsurface layers";
  - "The stratigraphy allows us to render folded strata on eroded cliffs". — [doi:10.1109/TVCG.2017.2689022](https://doi.org/10.1109/TVCG.2017.2689022)
- Cortial et al. summarise the graphics state of the art: subsurface strata are "encoded as a cell-based grid of layered stacks, with different thicknesses and material properties for the layers of each cell-specific stack … or volumetric models". Cordonnier et al. "approximate the lithosphere tectonics to reproduce folding and faulting effects in the crust, combined with hydraulic erosion". — [Cortial et al. 2019](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf)

**2020–2026 work found**
- Borg, Gain, Guérin, Peytavie, Cani, Galin & Cordonnier (CGF 45(2), 2026, "Authoring Terrestrial Planets with Diffusion Models"):
  - a generative model trained on Earth and Mars data infers heightmaps from user sketches of coarse elevation, landcover, temperature and precipitation on a globe;
  - it conditions on an extracted global river network;
  - plate tectonics is cited as motivation, and there is no tectonic simulation. — [Wiley](https://onlinelibrary.wiley.com/doi/10.1111/cgf.70390)
- Zechmann & Hlavacs 2025 (arXiv 2510.24764) compare two Godot planet generators, FBM Perlin noise and "Minecraft-inspired layered noise" (continentalness, erosion, peaks & valleys, spline remapping). Neither involves tectonics. — [arXiv:2510.24764](https://arxiv.org/abs/2510.24764)
- Surface-process tools that accept an uplift field:
  - H-Schott/StreamPowerErosion: "Large-Scale Stream Power Erosion", MIT, C++, pushed 2026-09-17. — [GitHub](https://github.com/H-Schott/StreamPowerErosion)
  - fastscapelib: "efficient algorithms for landscape evolution modeling", GPL-3.0, C++/Python. — [GitHub](https://github.com/fastscape-lem/fastscapelib)

### Inferences
- **Who tracks what:**

  | Tool | Tracks | Does not track |
  |---|---|---|
  | PlaTec / WorldEngine | height, plate ID, crust age | rock types |
  | Cortial 2019 | crust type, thickness, oceanic age, orogeny type/age, fold direction (a structural-fabric proxy) | rock types, strata |
  | tectonics.js | 7 rock-mass pools + age per cell | stratigraphic order |
  | tectonics.cpp | ordered functional strata (bedrock, sediment, sedimentary) × 16 minerals, with grain size and weathering | a deep stratigraphic stack |
  | Cordonnier 2018 | folded layered stratigraphy in a 3D uplift volume | — |
  | GPlates | plate IDs, feature types, seafloor age via GPlately | lithology |

  **No existing engine produces a block-scale 3D lithology.** Every one of them stops at 2D fields or coarse columns, and the Q4 tools must bridge the gap.
- **Blueprint for the mod's planet-scale history: Cortial et al.** It is scientifically reviewed, well parameterised and fast enough.
  - Derived from their Table 2: 500 Myr at 2 Myr per step is 250 steps.
  - At 100k samples (78 km): 250 × 0.28 s ≈ 70 s, plus 4–25 crust-generation/rifting events at about 1.4 s each (≈6–36 s), so roughly **1.3–1.8 min** on a 2016 desktop CPU.
  - At 500k samples (35 km): 250 × 1.9 s ≈ 8 min, plus 4–25 × 20.8 s (≈1.4–8.7 min), so roughly **9–17 min**.
  
  A well-optimised multithreaded Java port on a modern gaming CPU should land in the same range. That fits a "watch your world form" timelapse.
- **PlaTec is the only drop-in engine.** It is fast (seconds for 512²), LGPL and tiny. But it is flat and unitless, its crust has no composition, and it has no spherical geometry. It could provide a quick prototype of timelapse visuals or a regional heightfield, not credible geology.
- **Rock-inventory ideas: tectonics.js and tectonics.cpp.** Conserved felsic pools versus recyclable mafic crust, and functional strata with mineral bins.
  - tectonics.js's CC BY 4.0 allows porting with attribution.
  - tectonics.cpp's CC BY-NC-SA 4.0 bars commercial use and is share-alike; it is not an OSI licence and conflicts with typical mod licences. Treat it as reading material only, unless a commercial or other licence is negotiated with the author.
- **The graphics field has converged on "coarse tectonic parameters → local landform selection/amplification"** (Cortial) and "uplift field → erosion" (Cordonnier 2016, Schott/StreamPowerErosion). These define the interface the tectonic driver must emit for the surface-process team:
  - uplift/subsidence rate fields;
  - orogeny type/age;
  - fold/structural trend;
  - rock erodibility per layer.

### Gaps
- The Experilous site (Andy Gainey's 2014 planet generator) refused connections on 2026-09-27, and the archive.org fetch was blocked, so its method and licence were not verified directly (only indirectly via Red Blob Games).
- The PlaTec thesis PDF returned 403, so thesis runtime figures were not verified; the benchmark above stands in for them.
- Cortial et al. 2020, "Real-Time Hyper-Amplification of Planets", exists (seen in a ResearchGate listing), but I did not read its content.
- Galin et al. 2019, "A Review of Digital Terrain Modeling", was blocked by HAL's anti-bot page and the abstract was not in Semantic Scholar, so it is not quoted.
- Code availability for Cordonnier 2016 and 2018 is unknown.
- Discovery of 2020–2026 SIGGRAPH/Eurographics/JCGT papers on procedural plate tectonics is incomplete because the WebSearch budget was exhausted. Beyond Borg et al. 2026 (which is generative, not tectonic) I found none, but absence is not established.

## Q4. 3D structural and lithology modelling from event histories (Noddy/pynoddy, LoopStructural, GemPy, plus how tectonics.js, goSPL and Landlab LithoLayers represent layered rock): can any produce a queryable rock-type-at-(x,y,z) fast enough for world generation?

### Takeaway
Yes: the Noddy kinematic approach. You write an ordered history (base stratigraphy, then tilts, folds, faults, unconformities, dykes, plugs and shear zones). Rock at any point is found by "unravelling" the displacement equations in reverse chronological order.
- It is per-point and needs no global solve, so it is naturally lazy and chunkable.
- It generated 200³ voxels, geophysics included, in under 15 s on one server CPU.
- Its modernised C source (Qt port, 2026) is MIT-licensed, so it can be ported to Java.

LoopStructural (MIT) and GemPy (EUPL-1.2) are data-driven implicit interpolators. They take minutes per model in Python and need observations, so they suit offline authoring only. The Geodynamic World Builder (LGPL-2.1, C++ with a C API) is a per-point query library for plate-scale features (slabs, plates, faults). Landlab LithoLayers (MIT) and goSPL (GPL-3) show how surface-process codes represent layered rock.

### Cited Findings
**Noddy family**
- **How Noddy evaluates a point.** "The 3D geological models are calculated by taking the current x, y, z position of a point and unravelling the kinematics (using idealised displacement equations) until we get back to the time when the infinitesimal volume of rock was formed, whether defined by the initial stratigraphy, the time of formation of a stratigraphy above an unconformity or an intrusive event." — [Jessell et al. 2022, "Into the Noddyverse", ESSD 14:381](https://essd.copernicus.org/articles/14/381/2022/)
- **Event vocabulary.** STRAT (only once, first), FOLD, FAULT, UNC (unconformity), DYKE, PLUG, SHEAR ZONE and TILT, "which can be superimposed in any order". Table 1 lists the free parameters per event; for a fault these are position, displacement, azimuth, inclination and pitch. — [same](https://essd.copernicus.org/articles/14/381/2022/)
- **Resolution and extent.** "Any subset of the geology can be calculated for any sub-volume of an infinite Cartesian space … at an arbitrary resolution". The Noddyverse uses a 4 × 4 × 4 km volume sampled with 20 m voxels (200³). — [same](https://essd.copernicus.org/articles/14/381/2022/)
- **Speed.**
  - "rapid, taking under 15 s to generate 200 × 200 × 200 voxel models with both geological and geophysical representations combined using an Intel® Xeon® Gold 6254 CPU at 3.10 GHz";
  - 1 million random 5-event models "were built in around a week on a computer using 20 processor cores";
  - "Noddy can be thought of as a kinematic, semantic, implicit modelling scheme". — [same](https://essd.copernicus.org/articles/14/381/2022/)
- **How pynoddy describes Noddy.**
  - Each voxel is computed independently "using the Eulerian (inverse) form of the defining Lagrangian displacement equations, and applying them in reverse chronological order (i.e. starting with the most recent deformation event)".
  - New lithologies are created by "unconformities, dykes, and plugs".
  - A "discontinuity code" records which discontinuous events (faults, unconformities, dykes, plugs) affected each voxel.
  - Displacement equations are "kept as simple as possible, and superimposed deformation events are combined to produce structural complexity".
  - A single kinematic model is "typically very fast (in the order of seconds to minutes on a single core)". — [Wellmann et al. 2016, pynoddy 1.0, GMD 9:1019](https://gmd.copernicus.org/articles/9/1019/2016/)
- **Noddy Qt port** (Loop3D/noddy_qt):
  - MIT licence, pushed 2026-09-24.
  - It "builds a 3D geological model from a user-specified sequence of tectonic/depositional events (folds, faults, unconformities, dykes, plugs, ...), voxelises the result into a block model, and forward-calculates gravity and magnetic anomalies".
  - The original GUI used the commercial XVT toolkit and now runs on Qt5; "None of the calculation code (event kinematics, voxelisation, geophysics forward modelling) were … changed by the port".
  - New feature: a "Random 5-event history generator".
  - Lineage: Imperial College 1981–82 → Monash 1994 → UWA. — [Loop3D/noddy_qt](https://github.com/Loop3D/noddy_qt)
- Loop3D/noddyverse (MIT) holds the "C Code to generate random Noddy models", including "the source code for a command line version of Noddy". The model files moved to Australia's NCI after CloudStor closed. — [Loop3D/noddyverse](https://github.com/Loop3D/noddyverse)
- pynoddy is GPL-2.0. Its last PyPI release is 1.1 (2017-05-10), and it has had only 1 commit since 2025-01-01. — [cgre-aachen/pynoddy](https://github.com/cgre-aachen/pynoddy), [PyPI](https://pypi.org/project/pynoddy/)

**Implicit modellers**
- **LoopStructural** ("time-aware geological modelling"):
  - Implicit scalar fields are interpolated with piecewise-linear (tetrahedral) or finite-difference schemes, solved as sparse least-squares problems (conjugate gradient by default, or the pyamg multigrid solver).
  - Fault kinematics can be "added into the implicit description".
  - It has time-aware fold interpolation and unconformities. — [Grose et al. 2021, GMD 14:3915](https://gmd.copernicus.org/articles/14/3915/2021/)
- LoopStructural case study (15 faults; fault fields on a 50,000-element grid, stratigraphy on 500,000 elements; "workstation laptop with an i7 processor and 32gb of RAM"):
  - map2loop processing takes about 1 min;
  - building the implicit model about 8 min;
  - rendering surfaces on a 200 × 200 × 100 grid about 3 min. — [same](https://gmd.copernicus.org/articles/14/3915/2021/)
- LoopStructural status: MIT licence, 290 stars, v1.8.2 released 2026-09-27 (same day as a `map2loop-v3.5.0` tag in its releases). The standalone Loop3D/map2loop repo is archived. — [Loop3D/LoopStructural](https://github.com/Loop3D/LoopStructural), [PyPI](https://pypi.org/project/LoopStructural/), [Loop3D/map2loop](https://github.com/Loop3D/map2loop)
- **GemPy**: implicit modelling by universal cokriging of interface points and orientation (gradient) data, originally built on Theano for CPU/GPU execution. — [de la Varga et al. 2019, GMD 12:1](https://gmd.copernicus.org/articles/12/1/2019/)
- GemPy status: EUPL-1.2, 1,344 stars; PyPI 2026.0.3 (2026-06-18); pre-release 2026.1.0a3 (2026-07-11). — [gempy-project/gempy](https://github.com/gempy-project/gempy), [PyPI](https://pypi.org/project/gempy/)

**Plate-scale 3D query library**
- **Geodynamic World Builder (GWB).**
  - A JSON file "centered around plate tectonic terminology" defines continental plates, oceanic plates (age-dependent plate-model temperature), subducting slabs (trench line plus depth segments with dips and thicknesses), faults (including listric ones) and mantle layers.
  - Features overlay hierarchically, with later ones overwriting earlier ones.
  - It returns "temperature and/or composition" at any queried point; composition is a label plus a 0–1 fraction.
  - C++, with C and Fortran wrappers: "the C wrapper enables calling the GWB from almost any other language".
  - Works in Cartesian and spherical geometry; licence LGPLv2.1.
  - Benchmark: about 36 s to set up initial conditions for a 560,000-element ASPECT model on 480 MPI processes (≈0.7 % of the first time step). — [Fraters et al. 2019, Solid Earth 10:1785](https://se.copernicus.org/articles/10/1785/2019/)
- GWB status: requires "a C++14 compiler and CMake"; the current release is v1.1.0 (March 2026); pushed 2026-09-16. — [GeodynamicWorldBuilder/WorldBuilder](https://github.com/GeodynamicWorldBuilder/WorldBuilder)

**How surface-process codes represent layered rock**
- **Landlab `Lithology` / `LithoLayers`** (MIT):
  - "a three dimensional representation of material … Material can be removed through erosion or added to through deposition. Rock types can have multiple attributes (e.g. age, erodability …)".
  - `LithoLayers` is "constructed by specifying a series of depths below the surface, an anchor point, a series of rock type ids, and the functional form of a surface". — [litholayers.py](https://github.com/landlab/landlab/blob/master/src/landlab/components/lithology/litholayers.py), [lithology.py](https://github.com/landlab/landlab/blob/master/src/landlab/components/lithology/lithology.py)
- **goSPL** (GPL-3.0):
  - it simulates "dual-lithology (coarse and fine) sediment routing and stratigraphic history forced with deforming plate tectonics, paleotopographies and paleoclimate reconstructions";
  - unlike its predecessor it handles "horizontal advection" and "multiple lithologies and associated compaction";
  - it is Python with PETSc/MPI, runs on an unstructured spherical mesh and writes HDF5 output.
  - The latest PyPI release is 2026.7.14 (Python ≥3.11). — [Salles et al. 2020, JOSS](https://joss.theoj.org/papers/10.21105/joss.02804), [Geodels/gospl](https://github.com/Geodels/gospl), [PyPI](https://pypi.org/project/gospl/)
- tectonics.js per-cell rock pools and tectonics.cpp functional strata: see Q3. — [RockColumn.js](https://github.com/davidson16807/tectonics.js/blob/master/noncompiled/models/lithosphere/RockColumn.js), [tectonics.cpp stratum README](https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/stratum/README.md)

### Inferences
- **Speed.** Noddy's published figure (under 15 s for 8×10^6 voxels, geophysics included) is an upper bound of about 1.9 µs per voxel.
  - A full 16×16×384 Minecraft chunk column (98,304 blocks) would take under about 0.18 s single-threaded at that rate.
  - A Java port that skips geophysics, keeps histories short (5–20 events), caches per-chunk event transforms and evaluates coarsely (restored coordinates on a lattice, exact evaluation only where discontinuity codes differ) should run well inside worldgen budgets.
  - This is an estimate; no benchmark without geophysics was found.
- **Scale match.** Noddy's native domain (a km-scale cube with 20 m voxels) is about the size of a Minecraft world, and its event vocabulary maps onto Minecraft-visible geology:
  - folded and faulted strata;
  - unconformable sedimentary cover;
  - dykes (vein and ore hosts);
  - plugs (stocks, batholith tops, volcanic necks).
  
  Plate-scale engines are 100–1,000× coarser. The natural architecture is **plate-scale history → regional structural event list → per-block Noddy-style evaluation**.
- **Translating plate-scale events into local kinematic events** (design inference):
  - collision → folds + thrust faults + uplift/tilt + metamorphism;
  - rifting → normal faults + tilted blocks + unconformity + syn-rift sediments;
  - subduction arc → plutons (plugs) + dykes + volcanic strata;
  - hotspot → plug/dyke swarm + lava strata;
  - passive margin or basin → STRAT/UNC packages with thicknesses from subsidence;
  - erosion surfaces → unconformities.
- **Metamorphic grade.** Noddy has no P–T. But inverse kinematics already gives each point's restored position at every event time, so peak burial depth and heat (from collision thickening or pluton proximity) can be tracked per point and mapped to a grade (slate → schist → gneiss). This is a design extension, not an existing feature.
- **Implicit modellers are the wrong tool here.** They fit observations (contacts, orientations), and a synthetic world has none. With 8-minute builds (LoopStructural) and dense kriging (GemPy), they only make sense offline, e.g. if a future feature lets users draw geological maps to fit.
- **Licences for reuse:**
  - Noddy C (MIT): port freely with attribution.
  - LoopStructural (MIT): its fault-displacement profiles and fold-frame maths can be ported.
  - pynoddy (GPL-2.0) and goSPL (GPL-3.0): do not copy code.
  - GemPy (EUPL-1.2): copyleft.
  - GWB (LGPL-2.1): usable via a separately linked native library, or port its feature concepts. Its per-point-query idea (plates, slabs and faults defined by 2D surface geometry plus depth segments) is a good model for deep structure below the Noddy-scale layer.
- **Layer representation for the surface-process interface.** Landlab-style "depth-below-surface + anchor + functional surface + rock IDs with attributes" is a compact way to hand layered rock and erodibility to erosion code. Noddy-style restored coordinates can supply the same thing implicitly.

### Gaps
- Noddy's exact displacement equations (Jessell 1981; Jessell & Valenta 1996) were not read. Porting needs the MIT C source.
- No published Noddy timing without geophysics, and no per-event cost breakdown.
- No GemPy or GWB per-point timing on commodity hardware.
- The details of goSPL's stratigraphic layer model (layers per node, stored attributes) were not verified; the docs landing page did not show them.

## Q5. Scale: what compressions do existing procedural tectonic generators use, and what goes wrong with each?

### Takeaway
Existing tectonic generators do not compress scale. They simulate Earth-sized planets at 35–500 km resolution and then either amplify detail from exemplars or noise (Cortial) or zoom to about 1–2 km per pixel for regional maps (Undiscovered Worlds, Worldbuilding Pasta). PlaTec is unitless.

A whole 30 km Minecraft world is about one Cortial sample (~1,000 km²). Default vertical range is 384 blocks against Earth's ~20 km of relief. Custom dimension types can raise the build height to 4,064 blocks.

Every single-factor compression (a 1:1 window, horizontal squeeze, or a tiny planet) breaks something: visible geologic variety, slopes and dips, or physical meaning. The credible path is multi-scale decoupling (inference).

### Cited Findings
- **Minecraft vertical limits.**
  - Overworld default: `min_y` −64, `height` 384.
  - Custom dimension types: `min_y` "between -2032 and 2031 and be a multiple of 16"; `height` "between 16 and 4064 and be a multiple of 16"; and "min_y + height - 1 … cannot be greater than 2031". — [Minecraft Wiki: Dimension type](https://minecraft.wiki/w/Dimension_type)
- **Cortial et al.:**
  - planet radius 6,370 km;
  - 500k samples ≈ 35 km resolution (60k samples ≈ 102 km), with crust model resolution "ranging from 50 km to 500 km";
  - relief from trenches at −10 km to a maximum continental altitude of 10 km;
  - subduction influence 1,800 km and collision influence 4,200 km;
  - detail is added by amplification (procedural noise or SRTM90 exemplars) rather than by simulation. — [Cortial et al. 2019](https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf)
- **Undiscovered Worlds:** a global map plus regional maps "at a scale of approximately 1 pixel to 1 km". — [Undiscovered_Worlds_Classic README](https://github.com/JonathanCRH/Undiscovered_Worlds_Classic)
- **Worldbuilding Pasta:** about 2 km/pixel at 17,300 × 8,650 px, and "at this lateral resolution we won't see the peak itself so a range up to something like 7-8000 m should be sufficient". — [worldbuildingpasta Part VIIc](https://worldbuildingpasta.blogspot.com/2023/03/an-apple-pie-from-scratch-part-viic.html)
- **tectonics.cpp design:** stratigraphy matters down to about 12 km; "Lowest acceptable precision for a layer is probably Minecraft level, 1m blocks"; "upwards of 40k cells per grid". — [designing-stratigraphic-model.md](https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/designing-stratigraphic-model.md)
- **Resolution sensitivity:** Davidson notes that redefining plate boundaries where crust overlaps would make "the thickness of mountain ranges … highly dependant on grid resolution". — [thoughts-on-future-design.md](https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/crust/thoughts-on-future-design.md)
- **PlaTec:** square power-of-two maps with unitless heights (0.01–18.9 in this session's runs). — [lithosphere.hpp](https://github.com/Mindwerks/plate-tectonics/blob/main/src/lithosphere.hpp), [PyPlatec](https://pypi.org/project/PyPlatec/)
- **Noddy domain:** 4 × 4 × 4 km volumes at 20 m voxels. — [Noddyverse ESSD](https://essd.copernicus.org/articles/14/381/2022/)

### Inferences
- **Arithmetic.**
  - Earth's surface (5.1×10^8 km²) split into 500k samples is about 1,000 km² per sample. A 30 × 30 km Minecraft world (900 km²) is **about one tectonic sample**; a 10 × 10 km world is about 0.1 sample.
  - Vertically, fitting 20 km of relief (−10 to +10 km) into 384 blocks needs about 52× compression; into 4,064 blocks, about 5×.
- **(a) True 1:1 "window onto a big planet".** Geology stays correct, but the playable map shows a fragment of one belt (a forearc, part of an orogen, one rift shoulder).
  - The plate history mostly becomes context the player never sees.
  - Mountains are capped at 384 m unless a custom dimension height is used; a real alpine range needs 3–5 km, i.e. the full 4,064-block limit and its costs in memory, lighting and generation.
  - The timelapse would show a planet on which the play area is a dot.
- **(b) Horizontal compression** (1 block = h metres with h ≫ 1) while vertical stays 1 block = v m. Slopes in block space scale by h/v.
  - With h = 100 and v = 1, a 30° fault or bedding dip becomes about 89°, and every hillside becomes a cliff.
  - If both h and v are 100 m, relief fits but each block is a 100 m cube: trees, rivers and ore bodies are out of scale with the player, and strata thinner than 100 m vanish.
  - Isostatic and flexural wavelengths and arc–trench distances (hundreds of km) shrink to a few hundred blocks and read as a caricature.
- **(c) Shrinking the planet** (a whole-planet sphere a few tens of km across).
  - Lithosphere thickness (~100 km), mantle depth and plate speeds do not scale, so the "physics" becomes purely stylistic.
  - Parameters such as Cortial's 1,800 km subduction influence must be rescaled arbitrarily, which loses the scientific grounding the mod is supposed to have.
- **(d) Recommended: multi-scale decoupling**, as in Cortial's amplification and Undiscovered Worlds' regional zoom:
  1. Run the planet-scale history coarsely (50–100 km samples; this drives the timelapse and chooses the window).
  2. Pick a window straddling interesting boundaries (e.g. a suture plus an arc).
  3. Derive a regional event list (Q4) and evaluate structure at true 1:1 horizontal scale in the upper crust.
  4. Spend the vertical budget unevenly: modest relief compression (about 2–5×, helped by a taller custom dimension), and strong compression of the deep crust into the bottom tens of blocks, so batholith roots, gneiss and ophiolites still appear where erosion or faulting brings them up.
  
  Exact ratios need playtesting. No published voxel-game precedent was found.

### Gaps
- I found no published analysis of scale compression for tectonics-driven terrain in voxel games (the WebSearch budget ran out before a targeted search).
- Dwarf Fortress and other world-gen timelapse games' scale conventions were not researched.

## Q6. Verdict: embeddable libraries vs algorithms to reimplement vs offline-only tools, and what a minimal-but-credible from-scratch driver must contain

### Takeaway
Nothing heavy should be embedded. The mod should implement two things natively in Java:
- a **Cortial-style spherical kinematic plate driver**, enriched with tectonics.js-style rock inventories and an event log;
- a **Noddy-style event-history evaluator** that returns lithology at (x, y, z) lazily per chunk.

PlaTec (LGPL) is the only ready-made embeddable engine, and it is too crude (2D, unitless, age-only) to be the core. GPlates/pyGPlates/GPlately, the geodynamic solvers, LoopStructural/GemPy and goSPL belong offline: calibration, designer tooling, or export and interop targets.

### Cited Findings
- LGPL with Java: "libraries licensed under the GNU Lesser General Public License (LGPL) may be linked to proprietary applications", whereas an application linking a GPL library "must be licensed under the GPL". — [FSF, "The LGPL and Java"](https://www.gnu.org/licenses/lgpl-java.html)
- A GPL plug-in forming a single combined program with a nonfree main program violates the GPL without an exception, and only the copyright holders can grant one. — [GPL FAQ #GPLPluginsInNF](https://www.gnu.org/licenses/gpl-faq.html#GPLPluginsInNF), [#GPLIncompatibleLibs](https://www.gnu.org/licenses/gpl-faq.html#GPLIncompatibleLibs)
- Licences confirmed above:

  | Licence | Projects |
  |---|---|
  | GPL-2.0-only | GPlates / pyGPlates |
  | GPL-2.0 | GPlately, PTT, CitcomS, pynoddy |
  | GPL-2.0-or-later | ASPECT |
  | GPL-3.0 | goSPL, fastscapelib, Rayleigh, Undiscovered Worlds, pTatin3D copies |
  | LGPL-3 | Underworld 2/3 |
  | LGPL-2.1 | GWB |
  | LGPL (2.1+ in headers; 3.0 per GitHub/PyPI) | PlaTec |
  | MIT | Noddy Qt/Noddyverse, LoopStructural, LaMEM, Landlab, WorldEngine, StreamPowerErosion |
  | EUPL-1.2 | GemPy |
  | CC BY 4.0 | tectonics.js |
  | CC BY-NC-SA 4.0 | tectonics.cpp |
  | No licence | RNGalaxy, ProceduralPlanets, gubace, FioDev |

  — sources as cited in Q1–Q4 (e.g. [GPlates](https://github.com/GPlates/GPlates), [aspect](https://github.com/geodynamics/aspect), [underworld2](https://github.com/underworldcode/underworld2), [LaMEM](https://github.com/UniMainzGeo/LaMEM), [noddy_qt](https://github.com/Loop3D/noddy_qt), [tectonics.cpp](https://github.com/davidson16807/tectonics.cpp))

### Inferences
**Classification for a NeoForge 1.21.1 / Java 21 mod distributed to the public:**

| Option | Class | Why |
|---|---|---|
| Cortial et al. 2019 | **Algorithm to reimplement (core)** | Complete, reviewed process set; per-sample state; ~1–17 min per 500 Myr depending on resolution; interactive, so it suits a timelapse. No licensed code exists, so write from the paper. |
| Noddy kinematics (Jessell) | **Algorithm/code to port (core 3D)** | Per-point inverse evaluation, MIT C source, domain matches Minecraft scale; lithology is chunk-lazy. |
| tectonics.js rock pools and plate segmentation | Ideas/code to port | CC BY 4.0 (attribution). Rock-mass bookkeeping (felsic conserved vs mafic recycled); flood-fill plate segmentation if plates should emerge from a force field. |
| tectonics.cpp strata/minerals | Read-only inspiration | CC BY-NC-SA 4.0 blocks reuse in a public (possibly monetised) mod without a separate licence. |
| PlaTec / PyPlatec | Embeddable (JNI native lib, or port) but not core | LGPL, tiny and fast, but a 2D unitless square (power-of-two) grid with height/plate/age only. Needs per-OS natives (Windows/macOS/Linux, x64/arm64) if used via JNI. A Java port is easy and must stay LGPL. |
| GWB | Optional embeddable (LGPL, C API) or concept source | Per-point composition for slabs, plates and faults at depth. Overkill for blocks; useful model for deep structure. |
| Landlab LithoLayers | Concept (MIT) | Layer spec for handing lithology and erodibility to erosion code. |
| GPlates / pyGPlates / GPlately / PTT | **Offline / interop** | GPL-2.0-only; ~240 MB with native GL/glib deps; great for validation, designer authoring, and `.rot`/`.gpml` export for players. |
| ASPECT, Underworld 2/3, LaMEM, CitcomS, pTatin3D, Rayleigh, StagYY | **Offline research only** | HPC cost (months on supercomputers for Earth-like plates). Use published statistics to calibrate. |
| LoopStructural, GemPy | Offline only | Data-fitting implicit modelling, minutes per model in Python. |
| goSPL, Fastscape, StreamPowerErosion | Surface-process team's domain | They consume uplift fields and lithology from this driver. |

**Minimal-but-credible from-scratch tectonic driver (design sketch; numbers anchored to the cited sources):**
1. **Domain.** A coarse sphere with Fibonacci-sampled points (60k–250k, about 50–100 km) and a triangulation, plus a user or seed choice of a regional window that becomes the bordered Minecraft map. A flat bounded region is simpler, but plates entering or leaving the edges then need synthetic boundary plates; the sphere avoids that.
2. **Rigid-plate kinematics.**
   - Plate IDs; per-plate Euler pole and angular speed; velocity v = ω × r; finite and stage rotations (GPlates concepts).
   - Speed cap about 100 mm/yr; time step 1–2 Myr (Cortial uses 2 Myr, GPlately's ridge step is 1 Myr).
   - Slab pull steers plates toward their trenches (Cortial).
   - Optionally, plate motions chosen from a density/"subductability" field (tectonics.js v2), so that old oceanic crust drives plates instead of pure randomness.
3. **Per-sample crust state** (Cortial Table 1): type, thickness, elevation, oceanic age, ridge direction, orogeny type and age, fold/structural trend. Add:
   - a rock inventory (tectonics.js-style pools: sediment, sedimentary, metamorphic, felsic/mafic × plutonic/volcanic);
   - a compact **event log** (time, event type, parameters, e.g. "thrusted at 212 Ma, trend 040°, shortening X") for the structural stage.
4. **Boundary processes:**
   - Divergence: new oceanic crust, age 0, ridge direction; periodic resampling every 10–60 steps.
   - Subduction: the older plate or the oceanic plate goes down; uplift profile over about 1,800 km scaled by distance, relative speed and elevation; volcanic arc; trench sediment accretion.
   - Continent–continent collision: terrane detach and attach when interpenetration exceeds about 300 km; thickening; Himalayan-type orogeny.
   - Rifting: a Poisson process biased to large continental plates, splitting into 2–4 sub-plates.
   - Hotspots: fixed mantle-frame points (Cortial's suggested extension).
   - Transforms.
5. **Vertical balance.** Airy isostasy for continents (tectonics.cpp includes an Airy displacement model); ocean depth from age (subsidence as in Cortial's aging); simple rate-constant erosion and damping at this scale (Cortial's rates of about 0.03–0.3 mm/yr). Uplift and subsidence *rates* are exported as the interface to the surface-process and erosion teams (Cordonnier 2016-style input).
6. **Validation targets.** Plate size–frequency (a few large plates plus fractal small ones, split near 10^7.6 km²; Mallard); ocean crust mostly younger than about 250 Myr (Davidson's critique of random plates); plausible hypsometry. Designers can optionally check synthetic histories by exporting to GPlates.
7. **Bridge to blocks.**
   - Inside the window, turn the event log into a Noddy-style history: STRAT per basin, then TILT/FOLD/FAULT/UNC/DYKE/PLUG/SHEAR events with parameters drawn from each event's trend, magnitude and timing.
   - Evaluate lithology(x, y, z) per block by reverse-chronological inverse displacement; derive metamorphic grade from each point's restored burial history.
   - Leave hooks where ore-genesis rules key off event types (arc plutons, rift basins, sutures, hydrothermal dykes); ore genesis itself is another team's scope.
8. **Performance and UX.** Store the planet history as snapshots per N steps for the timelapse (PlaTec's step API and Cortial's pause-anytime design show per-step rendering is natural). Keep the global simulation deterministic from the seed. Keep the per-chunk lithology query side-effect free so chunks generate in parallel.

### Gaps
- Whether GPL/LGPL obligations bite for Minecraft mods (plug-ins to a proprietary game) has, to my knowledge, never been litigated. The above reflects FSF interpretations, not legal advice.
- No end-to-end prototype of the Cortial→Noddy bridge was found anywhere, so its visual credibility at block scale is unproven.
- Cortial-model runtimes in Java and at a 1–2 Myr time step on current hardware are extrapolated from 2016 C++ figures, not measured.
