# Sources cited by the research notes and designs

This is the second part of the bibliography: every source cited in the research report, the
science-related research notes and the design documents, grouped by the document that cites it, **as
that document writes it**. It was extracted in stage 1 (2026-10-02) by a sub-agent that read the
eleven documents in full. The references here were not resolved, de-duplicated or checked: titles,
years and links are the documents' own. The `read:` field records what the document itself says
about how the source was read; "not stated" is common.

Sources the code relies on are in [bibliography.md](bibliography.md), with full references. Where a
source appears in both, the entry there is the one to trust.

**Stage 2, WP-0 (2026-10-02).** Each mention below now ends with `[= Key]` when it is the same work as an entry of
[bibliography.md](bibliography.md) (171 of the 676 mentions; the key is the entry's heading there, where the
library status and the reference are) or `[unresolved]` when it is not (505: tools, mods, blogs, web pages,
and papers that no ledger row relies on). Matching was by DOI, URL or authors, year and title, strictly: a
different edition, a software port of a paper, or a different paper by the same authors is left unresolved. The
mentions themselves were not changed.

The engineering notes (Minecraft integration, mod compatibility, wrapped worlds, runtime packaging,
the Rust bridge, player demand) cite mod sources, jars and web pages, not science; their sources
are in the notes themselves and are not repeated here. See [`../research/README.md`](../research/README.md).

Extracted from the eleven research documents of the Deep Time repo (W = ``). Nothing here is invented: citations, URLs and reading notes are copied from the documents. Sources are grouped under one heading per document; a source cited by two documents is listed under each (de-duplication comes later).

Each bullet gives: the citation as the document writes it; the URL or DOI if the document gives one ("no link" otherwise); the document sections and the claim it supports; and `read:` how the document says the source was read ("fetched", "abstract only", "secondary source", "from memory", "not accessible", "web search (fetch not stated)", "title-level", or "not stated"). Each document's heading carries a reading-method note quoting what its methods or caveats section says. Where a document gives only a short form (for example "Singer et al. 2005"), the short form is kept; the long title or journal is added only when the document supplies it, or when it appears in the URL or DOI the document gives (marked as such).

Documents:
1. docs/research/geologic-history-worldgen-report.md
2. docs/research/notes/tectonics_engines.md
3. docs/research/notes/surface_processes_climate.md
4. docs/research/notes/ore_genesis_modeling.md
5. docs/research/notes/geologist_youtuber_feedback.md
6. docs/research/notes/run-review-1a0f969d358.md
7. docs/research/notes/grid-artifact-audit.md
8. docs/design/plate-force-balance.md
9. docs/design/plate-collisions-and-welding.md
10. docs/design/p1-26-per-cell-climate.md
11. docs/design/plate-check-baseline.md

## 1. docs/research/geologic-history-worldgen-report.md

Reading-method note for this document: the report has no methods or caveats section for literature. It says in passing that line counts were "counted for this report" (tokei 14.0.0 on shallow clones, 2026-09-27, commits listed in the "Ledger note"), that mod jars were "decompiled in the research" or checked "with javap", and that some links were "reasoned from source, not observed". Every bullet below says "not stated" unless the report says otherwise. Section names are the report's own headings.

### Geoscience tools and models (section "Nothing scientific embeds, but three published models carry the engine"; also "Reuse ledger")

- GPlates / pyGPlates 1.0.0 -- https://github.com/GPlates/GPlates ; https://www.gplates.org/docs/pygplates/ -- cited in "Nothing scientific embeds" for: reconstruction engine that only moves plates along supplied rotations; also in the reuse ledger (GPlates, pyGPlates, GPlately, GPL-2.0, offline validation and `.rot`/`.gpml` export) -- read: not stated. [unresolved]
- pygplates on PyPI -- https://pypi.org/project/pygplates/ -- "Nothing scientific embeds": GPL-2.0-only, about 239 MB installed with native GL and glib dependencies -- read: not stated. [unresolved]
- GPlately (GPL-2.0) -- no link -- reuse ledger row "Plate-kinematics validation": offline only -- read: not stated. [unresolved]
- Worldbuilding Pasta, "An Apple Pie from Scratch" Part V -- https://worldbuildingpasta.blogspot.com/2020/06/an-apple-pie-from-scratch-part-v.html -- "Nothing scientific embeds": worldbuilders hand-edit rotation files in 10-50 Myr steps, "more art than science" -- read: not stated. [unresolved]
- Worldbuilding Pasta, "An Apple Pie from Scratch" Part VIIc -- https://worldbuildingpasta.blogspot.com/2023/03/an-apple-pie-from-scratch-part-viic.html -- same claim -- read: not stated. [unresolved]
- Coltice et al. 2019 (StagYY mantle-convection run) -- https://europepmc.org/article/PMC/PMC6821462 -- "Nothing scientific embeds": nine months and 390,200 time steps for 1.5 Gyr; also decision 5 ("Coltice's run used a Pangea-like start") -- read: not stated. [= Coltice2019]
- goSPL scaling (hpc_scaling.rst) -- https://github.com/Geodels/gospl/blob/HEAD/docs/user_guide/hpc_scaling.rst -- "Nothing scientific embeds": cannot start a 10 km global mesh below about 16 MPI ranks -- read: not stated. [unresolved]
- goSPL (GPL-3.0) landscape-evolution code -- no separate link here -- "Nothing scientific embeds": GPL and HPC-bound -- read: not stated. [unresolved]
- analytical-terrains licence (Inria) -- https://gitlab.inria.fr/landscapes/analytical-terrains -- "Nothing scientific embeds": licensed for non-commercial research only -- read: not stated. [unresolved]
- ExoPlaSim -- https://arxiv.org/abs/2107.07685 -- "Nothing scientific embeds": 3.75 minutes per simulated year, 50-200 years spin-up, 3-12 hours per equilibrium climate; reuse ledger: GCM calibration table, GPL-2.0, offline -- read: not stated. [unresolved]
- Cortial, Peytavie, Galin and Guerin 2019, "Procedural Tectonic Planets" -- https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf -- "Nothing scientific embeds": blueprint for the plate driver (Fibonacci sphere samples, subduction, collision with terrane transfer above 300 km interpenetration, rifting into 2-4 plates, appendix constants: 2 Myr steps, 6,370 km radius, 100 mm/yr cap, 0.28 s per step at 100,000 samples on a 2016 i7-6700K, user study 97 votes to 47, reviewed by a tectonics specialist); decisions 5 and the architecture section (uplift terms) -- read: not stated (figures quoted from the paper). [= Cortial2019]
- Cortial re-implementations (three GitHub ports, no licence) -- choosealicense "no permission" page https://choosealicense.com/no-permission/ -- "Nothing scientific embeds": no permission to reuse; reuse ledger "partial Cortial core 793 lines C++" -- read: not stated. [unresolved]
- ProceduralPlanets (AurelienBesnier) at commit 81ad60e -- https://github.com/AurelienBesnier/ProceduralPlanets -- Ledger note: line count of the partial Cortial re-implementation (793 lines) -- read: counted with tokei on a shallow clone. [unresolved]
- PlaTec (Mindwerks plate-tectonics, LGPL) -- https://github.com/Mindwerks/plate-tectonics ; https://github.com/Mindwerks/plate-tectonics/blob/main/src/platecapi.hpp -- "Nothing scientific embeds": maintained embeddable engine, 512x512 world in 10.3 s in the research session, flat, unitless, power-of-two, outputs height, plate id and crust age; reuse ledger (3,407 lines C++, PlaTec/PyPlatec prototype only); Ledger note commit 2a27c4f -- read: run in the research session and counted with tokei. [unresolved]
- tectonics.js, RockColumn.js (davidson16807, CC BY 4.0) -- https://github.com/davidson16807/tectonics.js/blob/master/noncompiled/models/lithosphere/RockColumn.js -- "Nothing scientific embeds": seven rock-mass pools per cell, portable with attribution; reuse ledger (908 lines JS); Ledger note commit e43927c -- read: counted with tokei; otherwise not stated. [= Davidson_tectonicsjs]
- tectonics.cpp (CC BY-NC-SA 4.0) -- https://github.com/davidson16807/tectonics.cpp -- "Nothing scientific embeds" and decision 7: must stay reading material -- read: not stated. [unresolved]
- Mallard et al. 2016 -- https://web.gps.caltech.edu/~jstock/161/PapersToRead/nature17992-Mallard.pdf -- "Nothing scientific embeds": calibration target, plate-size split near 10^7.6 km2 -- read: not stated. [= Mallard2016]
- tectonics.js v2 post (2017-9-16-version2.md) -- https://github.com/davidson16807/tectonics.js/blob/master/_posts/2017-9-16-version2.md -- "Nothing scientific embeds": ocean crust mostly younger than about 250 Myr -- read: not stated. [unresolved]
- Jessell et al. 2022, Noddyverse (ESSD) -- https://essd.copernicus.org/articles/14/381/2022/ -- "Nothing scientific embeds": Noddy ordered event history evaluated pointwise by unravelling displacement equations -- read: not stated. [= Jessell2022]
- Wellmann et al. 2016 (GMD) -- https://gmd.copernicus.org/articles/9/1019/2016/ -- same Noddy kinematic-evaluation claim -- read: not stated. [= Wellmann2016]
- noddy_qt (Loop3D) -- https://github.com/Loop3D/noddy_qt -- Noddy built 200^3-voxel models in under 15 s on one Xeon Gold 6254; C source MIT -- read: not stated. [unresolved]
- noddyverse (Loop3D) at commit 6a21b23 -- https://github.com/Loop3D/noddyverse -- same; `unEvents.c` 1,226 lines holding unfold/unfault/undyke/unplug/unbury/reverseEvents; ledger 2,024 lines in `unEvents.c`, `taste.c`, `matrix.c`; `ellFault.c` 126 lines -- read: counted for this report. [= Noddyverse (software)]
- Nemez, Minecraft CPU performance testing (page 8) -- https://nemez.net/posts/20240414-minecraft-cpu-performance-testing/page-8/ -- per-chunk budget 20-70 ms from measured pregeneration rates (183 chunks/s on 4 threads) -- read: not stated. [unresolved]
- ishland C2ME benchmark gist -- https://gist.github.com/ishland/9601a033afad4be5b17ffaed859577fd -- per-chunk budget (137 chunks/s on 10 threads) -- read: not stated. [unresolved]
- Grose et al. 2021, LoopStructural (GMD) -- https://gmd.copernicus.org/articles/14/3915/2021/ -- "Nothing scientific embeds": about 8 minutes per model, stays offline; reuse ledger: LoopStructural (MIT) offline only -- read: not stated. [unresolved]
- GemPy (EUPL-1.2) -- no link -- "Nothing scientific embeds" and reuse ledger: offline only -- read: not stated. [unresolved]
- Tzathas et al. 2024, Analytical Terrains (EG) -- https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf -- surface processes: 556 s for 460 iterations at 1024^2 (about 1 us per cell per step); final landscape in 8 s at 1024^2; code is Inria research-only; ledger "Fast final landscape" -- read: not stated. [= Tzathas2024]
- FastFlow (PG2024 author version; no authors named in the report) -- https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf -- surface processes cost figure -- read: not stated. [= Jain2024]
- Braun & Willett 2013 (link slug: "A very efficient O(n) implicit and parallel method to solve the stream power equation governing fluvial incision and landscape evolution") -- https://www.researchgate.net/publication/236741975_A_very_efficient_On_implicit_and_parallel_method_to_solve_the_stream_power_equation_governing_fluvial_incision_and_landscape_evolution -- implicit stream-power method; ledger -- read: not stated. [= Braun2013]
- Yuan et al. (deposition term; "Yuan 2019" in ledger) -- no link -- surface processes and ledger: deposition term -- read: not stated. [= Yuan2019]
- Barnes (priority-flood, "Barnes 2014" in ledger) -- no link -- surface processes and ledger: priority-flood -- read: not stated. [= Barnes2014]
- fastscapelib (GPL-3.0) at commit b85cc6b -- https://github.com/fastscape-lem/fastscapelib -- stream-power, ADI-diffusion, priority-flood files 393, 381, 181 lines; 5,549 code lines total; offline oracle -- read: counted for this report. [= Fastscapelib]
- Landlab (MIT) at commit 7c60bd6 -- https://github.com/landlab/landlab -- soil production, LithoLayers, Bosscher-Schlager carbonate growth, submarine diffusion, HAND floodplains, threshold slopes; ledger line counts 142, 192, 273, 216, 178, 206 (1,207 total) -- read: counted for this report. [= Landlab]
- goSPL groundwater technical guide -- https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/groundwater.rst -- water-table duricrust scheme (laterite, bauxite caps with relief inversion); ledger "Duricrust and laterite, diagnostic ice, ELA" -- read: not stated. [unresolved]
- Laterite (Wikipedia) -- https://en.wikipedia.org/wiki/Laterite -- block mapping for laterite/bauxite caps -- read: not stated. [= Wikipedia-Laterite]
- Karst-Synthesis (Paris et al., MIT) -- https://github.com/aparis69/Karst-Synthesis -- cave-network synthesiser, karst gap -- read: not stated. [unresolved]
- Paris et al. 2019, 2021, 2023 (meanders, dunes, karst) -- no links -- reuse ledger "Meanders, dunes, karst": MIT, not counted -- read: not stated. [unresolved]
- Cordonnier et al. 2019, Fill-Spill-Merge (ESurf) -- https://esurf.copernicus.org/articles/7/549/2019/esurf-7-549-2019.pdf -- lakes from a 4-16-block routing grid -- read: not stated. [unresolved]
- Fill-Spill-Merge code (Barnes2020-FillSpillMerge, MIT) at 1c499ea, with depression hierarchy at 72bb3f5 -- https://github.com/r-barnes/Barnes2020-FillSpillMerge -- ledger 1,077 + 1,514 lines C++ (2,591) -- read: counted for this report. [unresolved]
- climlab (MIT) at commit fcbde37 -- https://github.com/climlab/climlab -- zonal energy-balance model with ice-albedo feedback; 801-line EBM module -- read: counted for this report. [unresolved]
- orographic-precipitation (fastscape-lem, MIT) at baf8a0c -- https://github.com/fastscape-lem/orographic-precipitation -- Smith-Barstad linear orographic precipitation, 96-line Python module -- read: counted for this report. [= FastscapeLEM]
- Smith and Barstad (linear orographic precipitation) -- no separate citation -- climate paragraph -- read: not stated. [unresolved]
- LFPM (GMD 2022) -- https://gmd.copernicus.org/articles/15/2063/2022/gmd-15-2063-2022.pdf -- orographic precipitation at about 2.4 stream-power steps' cost -- read: not stated. [unresolved]
- Koppen climate classification (Wikipedia) -- https://en.wikipedia.org/wiki/K%C3%B6ppen_climate_classification -- Koppen classification from monthly T and P -- read: not stated. [= WikipediaKoppen]
- SpeedyWeather (EUPL-1.2) -- no link -- offline GCM calibration -- read: not stated. [unresolved]

### Ore genesis and deposit sizing (section "Nothing scientific embeds", ore paragraph; "Ores and stones" table; decision 2)

- ProcGenesis -- https://procgenesis.com/ -- advertises ore views but documents no algorithm -- read: not stated. [unresolved]
- Skirrow et al. 2019, mineral systems (Ore Geology Reviews) -- https://doi.org/10.1016/j.oregeorev.2019.103011 -- mineral-systems approach, multiplicative ranking; ledger "Mineral-systems rule engine" -- read: not stated. [= Skirrow2019]
- Hronsky and Groves 2008 -- https://doi.org/10.1080/08120090701581356 -- mineral-systems approach; ledger -- read: not stated. [= Hronsky2008]
- McCuaig et al. 2010 -- no link -- ledger "Mineral-systems rule engine and deposit schema" -- read: not stated. [= McCuaig2010]
- Butterworth et al. 2016, Tectonics (per DOI) -- https://doi.org/10.1002/2016tc004289 -- porphyry favourability near 100 km/Myr convergence, ~15 degrees obliquity, 25-70 Myr slab age -- read: not stated. [= Butterworth2016]
- Hoggard et al. 2020, Nature Geoscience -- https://doi.org/10.1038/s41561-020-0593-2 -- 85% of sediment-hosted base metals within 200 km of a thick-thin lithosphere step -- read: not stated. [= Hoggard2020]
- Singer 1993 (USGS) -- https://pubs.usgs.gov/publication/70018261 -- USGS three-part method, grade-tonnage -- read: not stated. [= Singer1993]
- Singer et al. 2005 -- https://doi.org/10.2113/gsecongeo.100.3.491 -- porphyry tonnage percentiles 33, 240, 1,500 Mt at P90/P50/P10 (sigma about 1.49) -- read: not stated. [= Singer2005]
- USGS OFR 2008-1155 -- https://pubs.usgs.gov/of/2008/1155/ -- grade-tonnage/deposit data for sizing -- read: not stated. [= Singer2008a]
- Kesler and Wilkinson 2006, Economic Geology -- https://doi.org/10.2113/gsecongeo.101.5.919 -- exhumation about 158-167 m/Myr for porphyry and epithermal deposits -- read: not stated. [= Kesler2006]
- Sillitoe 2005 -- https://doi.org/10.5382/av100.22 -- epithermal exhumation; supergene zones at 2-3x primary grade (walkthrough step 9) -- read: not stated. [= Sillitoe2005]
- Bekker et al. 2010 -- https://doi.org/10.2113/gsecongeo.105.3.467 -- banded iron formation peak 2.65-1.85 Ga; iron row in the table -- read: not stated. [= Bekker2010]
- Cuney 2010, Economic Geology -- https://doi.org/10.2113/gsecongeo.105.3.553 -- sandstone uranium needs land plants; uranium row (1.8-1.2 Ga basin; 3.1-2.2 Ga placer) -- read: not stated. [unresolved]
- Nelsen et al. 2016, PNAS -- https://doi.org/10.1073/pnas.1517943113 -- coal needs land plants -- read: not stated. [= Nelsen2016]
- Perple_X, MAGEMin (GPL-3.0), PHREEQC -- no links -- thermodynamic codes, offline tables only; ledger "Metamorphic facies table" -- read: not stated. [unresolved]
- Waples 1980 (Lopatin TTI) -- https://doi.org/10.1306/2f9193d2-16ce-11d7-8645000102c1865d -- oil generation starts near TTI 15, peaks near 75; ledger -- read: not stated. [= Waples1980]
- Easy%Ro -- no link -- ledger "Petroleum maturity" -- read: not stated. [unresolved]
- MapMark4 (CC0) -- no link -- ledger "Grade-tonnage and density data": method reference -- read: not stated. [unresolved]
- USGS OFR/SIR databases; USGS MRDS -- no links -- ledger: public-domain data; calibration against MRDS -- read: not stated. [unresolved]
- EIS Toolkit (EUPL-1.2) -- no link -- ledger "Prospectivity weight calibration": offline -- read: not stated. [unresolved]
- USGS Bulletin 1693 ("B1693") -- https://pubs.usgs.gov/bul/1693/report.pdf -- diamond row (kimberlite pipes, placers up to 1,000 km downstream); ledger rule-engine reference -- read: not stated. [= Cox1986]
- Sillitoe 2010, Economic Geology -- https://doi.org/10.2113/gsecongeo.105.1.3 -- copper row: porphyry Cu-Mo-Au, VMS -- read: not stated. [= Sillitoe2010]
- Hitzman et al. 2010, Economic Geology -- https://doi.org/10.2113/gsecongeo.105.3.627 -- copper row: sediment-hosted stratiform Cu -- read: not stated. [unresolved]
- Leach et al. 2010, Economic Geology -- https://doi.org/10.2113/gsecongeo.105.3.593 -- zinc-lead row: MVT and SEDEX, younger than 2.02 Ga -- read: not stated. [= Leach2010b]
- Goldfarb et al. 2005 -- https://doi.org/10.5382/av100.14 -- gold row: orogenic lode, epithermal, placers -- read: not stated. [unresolved]
- Lehmann 2020, Lithos -- https://doi.org/10.1016/j.lithos.2020.105756 -- tungsten row: Sn-W greisen from reduced S-type leucogranite -- read: not stated. [= Lehmann2020]
- Bohacs and Suter 1997 -- https://doi.org/10.1306/3b05c3fc-172a-11d7-8645000102c1865d -- coal row: subsidence vs peat production -- read: not stated. [= Bohacs1997]
- Magoon & Dow 1994 -- https://doi.org/10.1306/m60585 -- crude-oil row: petroleum system -- read: not stated. [= Magoon1994]
- Giuliani and Groat 2019, Gems & Gemology -- https://doi.org/10.5741/gems.55.4.464 -- emerald row: Colombian black-shale-vein type -- read: not stated. [unresolved]
- Kesler and Wilkinson 2008, Geology -- https://doi.org/10.1130/g24317a.1 -- decision 2: deposits above 3.3 km are within mining reach -- read: not stated. [= Kesler2008]

### Minecraft bridge: game and mod sources (sections "Subclass the vanilla noise generator...", "Ores and stones...", "Runtime cost...")

NeoForge 21.1.249 sources jar (path (a local path)) -- cited for source-level claims; the report says the source "settles which one works". Read: source read (not further described).

- ChunkPyramid.java -- baked chunks pass through loading pyramid -- read: source read. [unresolved]
- ChunkSerializer.java -- NeoForge reports baked chunks as not new -- read: source read. [unresolved]
- ChunkGenerationTask.java -- partial bake resumes at an intermediate status -- read: source read. [unresolved]
- NoiseBasedChunkGenerator.java -- non-final; codec holds biome source and settings -- read: source read. [unresolved]
- ChunkMap.java -- builds RandomState from generatorSettings() -- read: source read. [unresolved]
- NoiseRouter.java -- 15-field noise router -- read: source read. [unresolved]
- RandomState.java -- custom density functions receive neither seed nor level -- read: source read. [unresolved]
- SurfaceSystem.java -- surface rules repaint only default_block -- read: source read. [unresolved]
- ServerChunkCache.java -- chunk system never consults the world border -- read: source read. [unresolved]
- LevelEvent.java -- cancellable CreateSpawnPosition -- read: source read. [unresolved]
- DimensionType.java -- build range up to 4,064 blocks -- read: source read. [unresolved]
- WorldCreationContext.java -- worldgenLoadContext() frozen registries -- read: source read. [unresolved]
- BiomeModifiers.java -- neoforge:remove_features -- read: source read. [unresolved]
- OreVeinifier.java -- vanilla large ore veins hard-coded copper-in-granite and iron-in-tuff -- read: source read. [unresolved]
- RegisterPresetEditorsEvent.java -- walkthrough step 1 -- read: source read. [unresolved]
- DedicatedServerProperties.java -- level-type=geohist:geologic on dedicated servers -- read: source read. [unresolved]
- TerraFirmaCraft TFCChunkGenerator (commit 8aa7f96) -- https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/8aa7f96cae0e7b10c63bf5b183f42b14f693c76f/src/main/java/net/dries007/tfc/world/TFCChunkGenerator.java -- dummy NoiseBasedChunkGenerator, empty getBaseColumn -- read: source read. [unresolved]
- World Preview TerraFirmaCraft -- https://modrinth.com/mod/world-preview-terrafirmacraft -- TFC generator breaks World Preview cross-section -- read: not stated. [unresolved]
- TerraFirmaCraft issue #2764 -- https://github.com/TerraFirmaCraft/TerraFirmaCraft/issues/2764 -- open Distant Horizons crash report -- read: not stated. [unresolved]
- TerraFirmaCraft repository at 16b1b8f (world package 25,152 lines Java; vein package 579 lines) -- https://github.com/TerraFirmaCraft/TerraFirmaCraft -- ledger -- read: counted for this report. [unresolved]
- ReTerraForged (Campione01, MIT) at ba35ca7 -- https://github.com/Campione01/ReterraForged -- proves the "keep vanilla generator, change inputs" route; LICENSE file is MIT; 1,432 + 957 (379 TerraBlender) + 2,507 lines; 22,283-line codebase -- read: LICENSE file and code read, counted with tokei. [unresolved]
- ReTerraForged CellSampler.java -- https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/world/worldgen/densityfunction/CellSampler.java -- density-function types, lazy Supplier<WorldLookup> -- read: source read. [unresolved]
- ReTerraForged MixinRandomState.java -- https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/mixin/MixinRandomState.java -- seed capture -- read: source read. [unresolved]
- ReTerraForged PresetEditorPage.java -- https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/client/gui/screen/presetconfig/PresetEditorPage.java -- preset screen with live DynamicTexture preview -- read: source read. [unresolved]
- TerraScribe (Sammasr, MIT) -- https://github.com/Sammasr/terrascribe -- pre-alpha, about 2,100 lines, last commit 2026-05-10 -- read: not stated. [unresolved]
- C2ME McToAst.java -- https://github.com/RelativityMC/C2ME-fabric/blob/ver/1.21.1/c2me-opts-dfc/src/main/java/com/ishland/c2me/opts/dfc/common/ast/McToAst.java -- C2ME compiler runs unknown density-function types as delegates -- read: source read. [unresolved]
- Tectonic (Modrinth, 3.0.26) -- https://modrinth.com/mod/tectonic -- NoiseBasedChunkGeneratorMixin forces lava level on settings key path `overworld`; WorldCreationUiStateMixin; 325 KB "Terratonic" noise-settings file; overlay.terratonic idiom; 16.5 million downloads -- read: jar decompiled in the research. [unresolved]
- Biomes O' Plenty (Modrinth) -- https://modrinth.com/mod/biomes-o-plenty -- hard-depends on TerraBlender; 36.4 million downloads -- read: not stated. [unresolved]
- Lithostitched (Modrinth) -- https://modrinth.com/mod/lithostitched -- Regions Unexplored depends on it; scopes modifiers by LevelStem -- read: not stated. [unresolved]
- TerraBlender 4.1.0.8 (Glitchfiend) -- https://github.com/Glitchfiend/TerraBlender -- applies to a dimension when three conditions hold (DimensionType tag, NoiseBasedChunkGenerator, MultiNoiseBiomeSource) -- read: decompiled. [unresolved]
- Aether dimension JSON -- https://raw.githubusercontent.com/The-Aether-Team/The-Aether/1.21.1-develop/src/generated/resources/data/aether/dimension/the_aether.json -- plain minecraft:noise dimension -- read: not stated. [unresolved]
- Deeper and Darker Otherside dimension JSON -- https://raw.githubusercontent.com/KyaniteMods/DeeperAndDarker/neoforge-1.21/src/generated/resources/data/deeperdarker/dimension/otherside.json -- same -- read: not stated. [unresolved]
- Alex's Caves MultiNoiseBiomeSourceMixin -- https://raw.githubusercontent.com/AlexModGuy/AlexsCaves/1.21.1/src/main/java/com/github/alexmodguy/alexscaves/mixin/MultiNoiseBiomeSourceMixin.java -- hooks getNoiseBiome; 1.21.1 port unreleased source branch -- read: source read. [unresolved]
- Terralith (Modrinth, 2.6.2) -- https://modrinth.com/mod/terralith -- Skylands terrain spline (+1.25 at y 200-230, continentalness -0.745 to -0.455 etc.), four terralith:skylands_* biome entries; 23.3 million downloads -- read: jar parsed in the research. [unresolved]
- FloatingIslands-Datapack (Youmiel) -- https://github.com/Youmiel/FloatingIslands-Datapack -- datapacks that overwrite noise_settings/overworld.json do not apply -- read: not stated. [unresolved]
- NeoForge Tags.java (1.21.1) -- https://raw.githubusercontent.com/neoforged/NeoForge/1.21.1/src/main/java/net/neoforged/neoforge/common/Tags.java -- c:ores convention; host-ground tags only for stone, deepslate, netherrack -- read: source read. [unresolved]
- Excavated Variants 4.3.1 README -- https://raw.githubusercontent.com/lukebemishprojects/ExcavatedVariants/1.21.x/README.md -- generates missing ore x stone variants -- read: not stated. [unresolved]
- misode/mcmeta stone_ore_replaceables.json -- https://raw.githubusercontent.com/misode/mcmeta/1.21.1-data/data/minecraft/tags/block/stone_ore_replaceables.json -- vanilla ore host tag -- read: not stated. [unresolved]
- Create Ore Excavation 1.6.8 (tom5454) -- https://github.com/tom5454/Create-Ore-Excavation -- vein recipes on a salted grid; `OreVeinGenerator.pick(LevelChunk)`, `OreData.setRecipe`, `setRandomMul` -- read: `javap` on the pack's own jar (finiteAmountBase interaction "was not decompiled"). [unresolved]
- Create: Diesel Generators 1.3.15 (Modrinth) -- https://modrinth.com/mod/create-diesel-generators -- `OilChunksSavedData.getBaseOilAmount`, `CDGKubeJSPlugin.calculateOilChunks`, 7,000,000 base -- read: `javap` on the pack's own jar. [unresolved]
- TFC PropickItem.java -- https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/common/items/PropickItem.java -- decision 10: false-negative chance 0.30 at tier 0 to 0 at tier 5 -- read: source read. [unresolved]
- Immersive Petroleum issue #96 -- https://github.com/Flaxbeard/ImmersivePetroleum/issues/96 -- decision 10: infinite-toggle request -- read: not stated. [unresolved]
- Region file format (Minecraft Wiki) -- https://minecraft.wiki/w/Region_file_format -- 1,048,576 chunks, about 5-30 GiB -- read: not stated. [unresolved]
- Chunky wiki (World borders; Pregeneration) -- https://github.com/pop4959/Chunky/wiki/World-borders ; https://github.com/pop4959/Chunky/wiki/Pregeneration -- worldborder pregeneration; servers pregenerate for hours -- read: not stated. [unresolved]
- Distant Horizons IDhApiWorldGenerator API doc -- https://distant-horizons-team.gitlab.io/distant-horizons/com/seibel/distanthorizons/api/interfaces/override/worldGenerator/IDhApiWorldGenerator.html -- LODs from rasters -- read: not stated. [unresolved]

### Market and community evidence (section "The premise holds...")

- Modrinth: Terralith, Tectonic, Biomes O' Plenty (see above); Geolosys -- https://modrinth.com/mod/geolosys -- 21,911 downloads, stalled at 1.20.1; TerraFirmaCraft -- https://modrinth.com/mod/terrafirmacraft -- 719,575 downloads; Immersive Geology 25,459 and GeoCraft 235 (no links) -- download counts at the research date -- read: not stated. [unresolved]
- TerraForged (CurseForge) -- https://www.curseforge.com/minecraft/mc-mods/terraforged -- about 11.7 million downloads, frozen on 1.18.2 -- read: not stated. [unresolved]
- Realistic Terrain Generation (CurseForge) -- https://www.curseforge.com/minecraft/mc-mods/realistic-terrain-generation -- 12.5 million+, frozen on 1.12.2 -- read: not stated. [unresolved]
- FreeTerraForged (Modrinth) -- https://modrinth.com/mod/freeterraforged -- 795 downloads -- read: not stated. [unresolved]
- Game World Observer, Dwarf Fortress sales -- https://gameworldobserver.com/2024/04/04/dwarf-fortress-sales-800k-steam-1-million-wishlists -- over a million Steam copies -- read: not stated. [unresolved]
- WN Hub -- https://wnhub.io/news/stores-and-publishing/item-47594 -- VG Insights estimate about $19.9 million gross -- read: not stated. [unresolved]
- Dwarf Fortress wiki, World generation -- https://www.dwarffortresswiki.org/index.php/World_generation -- history "happened for a reason" -- read: not stated. [unresolved]
- Steam, WorldBox -- https://store.steampowered.com/app/1206560/WorldBox__God_Simulator/ -- 96% positive across 29,730 reviews -- read: not stated. [unresolved]
- Terra-Titan (Planet Minecraft) -- https://www.planetminecraft.com/project/terra-titan-1-20-1-21-massive-and-realistic-30-000-x-30-000-map-survival-friendly-mmorpg-focused-java-edition/ -- 30,000-block realistic map -- read: not stated. [unresolved]
- Terra-Mar (Planet Minecraft) -- https://www.planetminecraft.com/project/terra-mar-1-20-massive-and-realistic-40-000-x-40-000-map-survival-friendly-mmorpg-focused-java-edition/ -- 40,000-block map -- read: not stated. [unresolved]
- EarthMC -- https://earthmc.net/ -- Earth-map geopolitics server since 2016 -- read: not stated. [unresolved]
- TopG listing (Geopolitical servers) -- https://topg.org/minecraft-servers/type/Geopolitical/ -- Terradivisa ore-territory loop -- read: not stated. [unresolved]
- World Preview (Modrinth) -- https://modrinth.com/mod/world-preview -- 1,992,325 downloads, stale since 2024 -- read: not stated. [unresolved]
- Genesis World Previewer/Pregenerator (Modrinth) -- https://modrinth.com/mod/genesis-world-previewer-pregenerator -- 2026 successor; walkthrough step 4 "Genesis's trick" -- read: not stated. [unresolved]
- Minecraft Forum, tectonic-plates mod request -- https://www.minecraftforum.net/forums/mapping-and-modding-java-edition/minecraft-mods/requests-ideas-for-mods/1328878-idea-minecraft-tectonic-plates-mod-volcanoes-cave -- "impossible to code" -- read: not stated. [unresolved]
- British Geological Survey, GB geology Minecraft world -- https://www.bgs.ac.uk/discovering-geology/maps-and-resources/maps/gb-geology-minecraft-world/ -- education channel -- read: not stated. [unresolved]
- OTG Continued (Modrinth) -- https://modrinth.com/mod/open-terrain-generator-continued -- fork "due to issues with Mojang" -- read: not stated. [unresolved]
- Info Toast, "OpenTerrainGenerator: the death of an 11 year old Minecraft mod" -- https://infotoast.org/site/index.php/2022/03/13/openterraingenerator-the-death-of-an-11-year-old-minecraft-mod/ -- terrain mods die at version jumps -- read: not stated. [unresolved]
- Minecraft Wiki, Density function -- https://minecraft.wiki/w/Density_function -- 26.3 snapshots removed flat_cache, cache_2d, cache_all_in_cell, shifted_noise, renamed cache_once -- read: not stated. [unresolved]
- FTB forum, "TerraFirmaCraft opinions" -- https://forum.feed-the-beast.com/threads/terrafirmacraft-opinions.51111/ -- "thousands of blocks to find a nickel vein" -- read: not stated. [unresolved]
- Immersive Engineering issue #2229 -- https://github.com/BluSunrize/ImmersiveEngineering/issues/2229 -- "mined ... for about an hour and found nothing" -- read: not stated. [unresolved]
- Create Ore Excavation issue #85 -- https://github.com/tom5454/Create-Ore-Excavation/issues/85 -- "3 hours straight flying in the nether" -- read: not stated. [unresolved]
- Geolosys issue #158 -- https://github.com/oitsjustjose/Geolosys/issues/158 -- per-block ore data froze servers about 20 s past 50 MB -- read: not stated. [unresolved]
- Geolosys repository at 544f54b -- https://github.com/oitsjustjose/Geolosys -- ledger: 956 lines deposit API and feature -- read: counted for this report. [unresolved]
- Undiscovered Worlds Classic (JonathanCRH) at b31f2ed -- https://github.com/JonathanCRH/Undiscovered_Worlds_Classic -- 56,338 lines, a complete single-author world engine -- read: counted for this report. [unresolved]
- StreamPowerErosion (H-Schott, Schott et al. 2023, MIT) at 8a0c017 -- https://github.com/H-Schott/StreamPowerErosion -- 214-line GLSL kernel; optional GPU path -- read: counted for this report. [unresolved]

### Architecture and runtime (section "A Minecraft-free core...", decisions 3, 4, 7, 13)

- JEP 306 (strict floating point) -- https://openjdk.org/jeps/306 -- bit-identical results with fixed operation order -- read: not stated. [unresolved]
- arXiv 2408.05148 -- https://arxiv.org/pdf/2408.05148 -- vectorised/native/GPU code reorders sums -- read: not stated. [unresolved]
- JEP 448 (Vector API, incubator) -- https://openjdk.org/jeps/448 -- needs launch flag -- read: not stated. [unresolved]
- JEP 442 (FFM API, preview on Java 21) -- https://openjdk.org/jeps/442 -- MemorySegment avoided -- read: not stated. [unresolved]
- Create: Power Grid issue #457 (patryk3211) -- https://github.com/patryk3211/PowerGrid/issues/457 -- native backend SIGILL on QEMU virtual CPU, no macOS binary -- read: not stated. [unresolved]
- OpenGL (Wikipedia) -- https://en.wikipedia.org/wiki/OpenGL -- macOS stops at OpenGL 4.1, compute shaders need 4.3 -- read: not stated. [unresolved]
- Prism Launcher RAM defaults (BisectHosting help) -- https://help.bisecthosting.com/hc/en-us/articles/50075520613787-How-to-Allocate-More-RAM-in-the-Prism-Launcher -- launcher defaults 2-4 GB -- read: not stated. [unresolved]
- OuiHeberg modded server guide -- https://www.ouiheberg.com/en/blog/minecraft-modded-server-guide -- 200-mod packs want 10-16 GB -- read: not stated. [unresolved]
- GNU GPL FAQ, GPLPluginsInNF -- https://www.gnu.org/licenses/gpl-faq.html#GPLPluginsInNF -- decision 7: GPL plug-in combined with non-free program -- read: not stated. [unresolved]
- JOML, EJML, ojAlgo -- no links -- ledger "Vector maths, optional linear algebra" -- read: not stated. [unresolved]
- RCON (wiki.vg) -- https://wiki.vg/RCON -- decision 13: 1,446-byte commands -- read: not stated. [unresolved]
- Ledger-referenced mods without separate links: Terralith ReStoned, Regions Unexplored, Oh The Biomes We've Gone, Distant Horizons, C2ME, Chunky (links above), Create, KubeJS, Jade -- named as ecosystem facts -- read: not stated. [unresolved]
## 2. docs/research/notes/tectonics_engines.md

Reading-method note for this document: header says "Research date 2026-09-27. Licence, release and commit data were read from the GitHub and PyPI APIs that day. Two quick experiments were run in this session: a PyPlatec benchmark and an in-memory pyGPlates synthetic-plate test" (scripts `bench_platec.py`, `test_pygplates.py`). It also says "this session's WebSearch budget ran out partway through, so discovery of post-2019 graphics papers is incomplete". Each question (Q1-Q6) ends with a "Gaps" list naming what could not be read; those are applied below. Sections cited as Q1 (reconstruction tools), Q2 (forward geodynamic codes), Q3 (graphics/procedural tectonics), Q4 (3D structural modelling), Q5 (scale), Q6 (verdict). Sources whose text is quoted in "Cited Findings" are recorded as "fetched (quoted)"; the document does not use the word "fetched", so this is my reading of its method statement.

### Q1. Kinematic and reconstruction tools

- GPlates / PyGPlates repository (GPlates/GPlates) -- https://github.com/GPlates/GPlates -- Q1: description, GPL-2.0-only (COPYING notice quoted), last pushed 2026-09-26, 152 stars -- read: fetched (quoted; metadata from GitHub API). [unresolved]
- GPlates releases -- https://github.com/GPlates/GPlates/releases -- Q1: GPlates 2.5 (2024-04-15), PyGPlates 1.0.0 (2025-03-28) -- read: fetched (API). [unresolved]
- pygplates on PyPI -- https://pypi.org/project/pygplates/ -- Q1: 1.0.0 uploaded 2025-03-26, wheels cp38-cp313, 31-73 MB; install footprint measured -- read: fetched (PyPI JSON) and installed and run in the session (pyGPlates 1.0.0, Python 3.12, i7-1185G7). [unresolved]
- pyGPlates documentation -- https://www.gplates.org/docs/pygplates/ -- Q1: what pyGPlates 1.0 computes (reconstruction, topologies, rotations, velocities) -- read: fetched (quoted). [unresolved]
- pygplates.Feature documentation -- https://www.gplates.org/docs/pygplates/generated/pygplates.Feature.html -- Q1: factory methods incl. `create_total_reconstruction_sequence()` -- read: fetched (quoted). [unresolved]
- pygplates.RotationModel documentation -- https://www.gplates.org/docs/pygplates/generated/pygplates.RotationModel.html -- Q1: RotationModel accepts in-memory rotation features -- read: fetched (quoted). [unresolved]
- GPlately README (GPlates/gplately) -- https://github.com/GPlates/gplately -- Q1: capabilities, GPL-2.0, 95 stars, pushed 2026-09-21; method adapted from Williams et al. 2021 -- read: fetched (quoted). [unresolved]
- GPlately v2.0.0 release notes -- https://github.com/GPlates/gplately/releases/tag/v2.0.0 -- Q1: v2.0.0 (2025-07-14) changes -- read: fetched. [unresolved]
- gplately on PyPI -- https://pypi.org/project/gplately/ -- Q1: dependency list (PyPI 2.0.0, Python >=3.9) -- read: fetched. [unresolved]
- GPlately SeafloorGrid notebook -- https://gplates.github.io/gplately/latest/notebook-html/10-SeafloorGrids.html -- Q1: SeafloorGrid method (icosahedral point mesh, ridge_time_step 1 Myr, ridge_sampling 0.5 degrees) and cost -- read: fetched (quoted). [unresolved]
- Williams et al. 2021, Geoscience Frontiers 12(2):769-780 -- no link -- Q1: source of the SeafloorGrid method -- read: secondary source (named in the GPlately README). [unresolved]
- PlateTectonicTools (EarthByte) -- https://github.com/EarthByte/PlateTectonicTools ; https://pypi.org/project/PlateTectonicTools/ -- Q1: ptt functions, GPL-2.0, last PyPI release 0.4.1 on 2022-11-16, README "only in its initial stages" -- read: fetched (quoted; GitHub API). [unresolved]
- Mather et al., "Deep time spatio-temporal data analysis using pyGPlates with PlateTectonicTools and GPlately", Geoscience Data Journal -- https://doi.org/10.1002/gdj3.185 -- Q1: reference paper for the stack -- read: not stated (cited, not quoted). [unresolved]
- plate-model-manager -- https://github.com/GPlates/plate-model-manager ; https://pypi.org/project/plate-model-manager/ -- Q1: "like Conda for plate models", GPL-2.0, v1.3.2 uploaded 2026-05-24 -- read: fetched. [unresolved]
- Worldbuilding Pasta, "Part V Supplement: Tectonics: Using GPlates" (2020) -- https://worldbuildingpasta.blogspot.com/2020/06/an-apple-pie-from-scratch-part-v.html -- Q1: hand-edited .rot workflow, 50/10 Myr steps, starts at 2000 Ma; Q3/Q6 comparisons -- read: fetched (quoted). [unresolved]
- Worldbuilding Pasta, "Part VIIc: Constructing Global Terrain" (2023) -- https://worldbuildingpasta.blogspot.com/2023/03/an-apple-pie-from-scratch-part-viic.html -- Q1 and Q5: hand-painted elevations (25 m units to 8,100 m), 17,300x8,650 px, about 2 km/pixel, "more art than science", "7-8000 m should be sufficient" -- read: fetched (quoted). [unresolved]
- Mallard et al. 2016, Nature 535:140 -- https://web.gps.caltech.edu/~jstock/161/PapersToRead/nature17992-Mallard.pdf ; https://www.nature.com/articles/nature17992 -- Q1: plate polygons built manually in GPlates; Q2: StagYY, Ra 10^6, 45 km, yield stresses 100-250 MPa (plate-like 100-350 MPa), 100 MPa model more than 100 plates, plate-size split around 10^7.6 km2; Q6 validation targets -- read: fetched (quoted, PDF). [= Mallard2016]
- GPL FAQ #GPLPluginsInNF -- https://www.gnu.org/licenses/gpl-faq.html#GPLPluginsInNF -- Q1 and Q6: plug-in combined with nonfree program -- read: fetched (quoted). [unresolved]
- GPL FAQ #GPLIncompatibleLibs -- https://www.gnu.org/licenses/gpl-faq.html#GPLIncompatibleLibs -- Q1 and Q6: only copyright holders can authorise an exception -- read: fetched (quoted). [unresolved]
- GPL FAQ #MereAggregation -- https://www.gnu.org/licenses/gpl-faq.html#MereAggregation -- Q1: separate programs communicating by pipes/sockets -- read: fetched (quoted). [unresolved]
- GPL FAQ #v2v3Compatibility -- https://www.gnu.org/licenses/gpl-faq.html#v2v3Compatibility -- Q1: GPLv2 and GPLv3 not compatible -- read: fetched (quoted). [unresolved]

### Q2. Forward geodynamic codes

- Coltice et al. 2019, "What drives tectonic plates?", Science Advances 5:eaax4295 -- https://europepmc.org/article/PMC/PMC6821462 ; https://www.science.org/doi/10.1126/sciadv.aax4295 -- Q2: StagYY yin-yang grid, Ra 10^7, 61 MPa yield stress, 23 km average resolution (about 50,000,000 cells), 1,500 Ma in nine months and 390,200 time steps; ten lower-vigour models Ra 10^6 at 45 km; surface speeds about 4 cm/yr -- read: fetched (quoted, Europe PMC full text). Gap: "Neither the Coltice supplement nor the Mallard main text reports wall-clock time for the low-Ra (10^6, 45 km) runs." [= Coltice2019]
- ASPECT (geodynamics/aspect) -- https://github.com/geodynamics/aspect ; https://github.com/geodynamics/aspect/releases -- Q2: GPL v2 or later, v3.1.0 released 2026-09-19, 311 stars -- read: fetched (API). [unresolved]
- Underworld 2 (underworldcode/underworld2) -- https://github.com/underworldcode/underworld2 -- Q2: LGPL-3, v2.17.3 released 2026-05-14 -- read: fetched (quoted). [unresolved]
- Underworld 3 (underworldcode/underworld3) -- https://github.com/underworldcode/underworld3 -- Q2: LGPL-3, v3.0.0 2026-03-22, v3.1.0 2026-07-08 -- read: fetched. [unresolved]
- LaMEM (UniMainzGeo/LaMEM) -- https://github.com/UniMainzGeo/LaMEM -- Q2: MIT, v3.2.0 released 2026-09-22 -- read: fetched (quoted). [unresolved]
- CitcomS (geodynamics/citcoms) -- https://github.com/geodynamics/citcoms -- Q2: GPL-2.0, last push 2025-09-12 -- read: fetched. [unresolved]
- Rayleigh (geodynamics/Rayleigh) -- https://github.com/geodynamics/Rayleigh -- Q2: GPL-3.0, Fortran, pushed 2026-09-20 -- read: fetched. [unresolved]
- pTatin3D (richardhaslam/pTatin3d) -- https://github.com/richardhaslam/pTatin3d -- Q2: GPL-3.0 copy, last pushed 2021 -- read: fetched. [unresolved]
- ptatin-gene (laetitialp) -- https://github.com/laetitialp/ptatin-gene -- Q2: 2026 GPL-3.0 fork -- read: fetched. [unresolved]
- pTatinIO.jl (boriskaus) -- https://github.com/boriskaus/pTatinIO.jl -- Q2: pTatin3D related -- read: fetched (listed). [unresolved]
- tectonics.js version-2 post -- https://github.com/davidson16807/tectonics.js/blob/master/_posts/2017-9-16-version2.md -- Q2 and Q3: crust age to density to "subductability", velocity smoothed by diffusion, flood-fill plate segmentation; critique of random plates and Earth age distribution -- read: fetched (quoted). [unresolved]
- StagYY -- no link -- Q2: "The StagYY licence and distribution terms are not documented in the sources read; it was only 'provided' by its author" -- read: not accessible as a source in itself (named only through Coltice and Mallard). [unresolved]

### Q3. Game and graphics procedural tectonics

- Cortial, Peytavie, Galin and Guerin 2019, "Procedural Tectonic Planets", Computer Graphics Forum 38(2), Eurographics 2019 -- https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf ; https://doi.org/10.1111/cgf.13614 -- Q3, Q5, Q6: whole model (Table 1 per-sample parameters, Appendix A constants, Table 2 timings on an i7-6700K, 97 votes to 47 user study, stated limitations, state-of-the-art summary of strata encodings) -- read: fetched (quoted; full PDF). [= Cortial2019]
- robertoost/RNGalaxy -- https://github.com/robertoost/RNGalaxy -- Q3: Unity/C# Cortial implementation, unlicensed, 8 stars -- read: fetched (GitHub). [unresolved]
- AurelienBesnier/ProceduralPlanets -- https://github.com/AurelienBesnier/ProceduralPlanets -- Q3: C++20/Qt5/CGAL Cortial implementation, unlicensed -- read: fetched (GitHub). [unresolved]
- gubace/Procedural-Tectonic-Planets -- https://github.com/gubace/Procedural-Tectonic-Planets -- Q3: OpenGL student project citing the Cortial PDF, 2025-12 -- read: fetched (GitHub). [unresolved]
- choosealicense.com, "no permission" -- https://choosealicense.com/no-permission/ -- Q3: unlicensed code means no permission -- read: fetched (quoted). [unresolved]
- Viitanen 2012 thesis (Helsinki Metropolia; title only as the Semantic Scholar URL slug "Physically-Based-Terrain-Generation-:-Procedural-...") -- https://www.semanticscholar.org/paper/Physically-Based-Terrain-Generation-:-Procedural-Viitanen/7a7496190f5ba295d1c78ad3a73fe28398582939 ; PDF https://www.theseus.fi/bitstream/handle/10024/40422/Viitanen_Lauri_2012_03_30.pdf -- Q3: PlaTec description -- read: abstract only ("This description comes from the abstract as indexed; the PDF itself returned HTTP 403"); thesis runtime figures not verified. [unresolved]
- PlaTec / plate-tectonics (Mindwerks) -- https://github.com/Mindwerks/plate-tectonics -- Q3, Q6: LGPL (2.1+ in headers, LGPL-3.0 per GitHub), v1.4.3 released 2025-11-18, 65 commits since 2025-01-01 -- read: fetched; benchmarked in the session (PyPlatec 512x512: 1,042 steps, 10.3 s; 1024x1024: 1,144 steps, 36.9 s; peak RSS about 195 MB). [unresolved]
- platecapi.hpp -- https://github.com/Mindwerks/plate-tectonics/blob/main/src/platecapi.hpp -- Q3: C API outputs height, plate index and crust age only -- read: fetched (quoted). [unresolved]
- lithosphere.hpp -- https://github.com/Mindwerks/plate-tectonics/blob/main/src/lithosphere.hpp -- Q3 and Q5: square power-of-two map, "5000-13000 collisions" per step in 512x512 -- read: fetched (quoted). [unresolved]
- PyPlatec on PyPI -- https://pypi.org/project/PyPlatec/ -- Q3: LGPL-3.0-or-later, wheels 53 KB to 0.65 MB -- read: fetched; benchmarked. [unresolved]
- Mindwerks/pyplatec (archived) -- https://github.com/Mindwerks/pyplatec -- Q3: bindings moved in-repo -- read: fetched. [unresolved]
- WorldEngine (Mindwerks) -- https://github.com/Mindwerks/worldengine ; https://pypi.org/project/worldengine/ -- Q3: plate simulation + erosion + Holdridge life zones, MIT, 1,071 stars, v0.20.0 2025-11-18 -- read: fetched (quoted). [unresolved]
- worldengine-java -- https://github.com/Mindwerks/worldengine-java -- Q3: Java loader, MIT, last pushed 2015 -- read: fetched. [unresolved]
- tectonics.js (davidson16807) -- https://github.com/davidson16807/tectonics.js -- Q3: CC-BY-4.0, 233 stars, last push 2020-08-22 -- read: fetched. [= tectonics.js (software)]
- tectonics.js RockColumn.js -- https://github.com/davidson16807/tectonics.js/blob/master/noncompiled/models/lithosphere/RockColumn.js -- Q3: per-cell rock pools (sediment, sedimentary, metamorphic, felsic/mafic plutonic/volcanic, age) -- read: fetched (quoted). [unresolved]
- tectonics.js Crust.js -- https://github.com/davidson16807/tectonics.js/blob/master/noncompiled/models/lithosphere/Crust.js -- Q3: Float32 rasters per pool; felsic "sial", mafic "sima" comments -- read: fetched (quoted). [unresolved]
- tectonics.cpp (davidson16807) -- https://github.com/davidson16807/tectonics.cpp -- Q3: header-only C++ library, CC BY-NC-SA 4.0 since May 2025, 100+ commits since 2025-01-01, last pushed 2026-07-29 -- read: fetched (quoted). [unresolved]
- tectonics.cpp inc/model/rock/README.md -- https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/README.md -- Q3: rock hierarchy -- read: fetched (quoted). [unresolved]
- tectonics.cpp stratum README -- https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/stratum/README.md -- Q3 and Q4: bedrock/sediment/sedimentary strata -- read: fetched (quoted). [unresolved]
- tectonics.cpp mineral README -- https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/mineral/README.md -- Q3: minerals, grain-size classes, half-byte bins -- read: fetched (quoted). [unresolved]
- tectonics.cpp designing-stratigraphic-model.md -- https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/designing-stratigraphic-model.md -- Q3 and Q5: 12 km depth (Kola borehole), "Minecraft level, 1m blocks", 16 mineral pools, 40k cells -- read: fetched (quoted). [unresolved]
- tectonics.cpp crust README and crust/ directory -- https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/crust/README.md ; https://github.com/davidson16807/tectonics.cpp/tree/master/inc/model/rock/crust -- Q3: crust types, Airy isostatic displacement headers -- read: fetched (quoted). [unresolved]
- tectonics.cpp thoughts-on-future-design.md -- https://github.com/davidson16807/tectonics.cpp/blob/master/inc/model/rock/crust/thoughts-on-future-design.md -- Q5: mountain thickness grid-resolution dependence -- read: fetched (quoted). [unresolved]
- Undiscovered Worlds Classic (JonathanCRH) README -- https://github.com/JonathanCRH/Undiscovered_Worlds_Classic -- Q3 and Q5: GPL-3.0, regional maps about 1 pixel to 1 km, mountains from a separate peak map -- read: fetched (quoted). [unresolved]
- Undiscovered Worlds (new globe version) -- https://github.com/JonathanCRH/Undiscovered_Worlds -- Q3: "early experimental", same seed not always same planet -- read: fetched (quoted). [unresolved]
- lukabergs/UndiscoveredWorlds fork -- https://github.com/lukabergs/UndiscoveredWorlds -- Q3: adds plate-tectonics module mirroring PlaTec -- read: fetched. [unresolved]
- Red Blob Games, planet generation (2018) -- https://www.redblobgames.com/x/1843-planet-generation/ -- Q3: single-step Voronoi plates "following the approach of Andy Gainey and others" -- read: fetched (quoted). [unresolved]
- Experilous (Andy Gainey, 2014 planet generator) -- no link -- Q3 Gaps: "refused connections on 2026-09-27, and the archive.org fetch was blocked" -- read: not accessible; known only indirectly via Red Blob Games. [unresolved]
- FioDev/Procedural-Tectonics -- https://github.com/FioDev/Procedural-Tectonics -- Q3: Unity 2023, no licence, 400-1,000 simulated years -- read: fetched. [unresolved]
- Cordonnier et al. 2016, CGF 35(2) (title not given in the document; it quotes the abstract) -- https://doi.org/10.1111/cgf.12820 ; abstract via https://api.openalex.org/works/doi:10.1111/cgf.12820 -- Q3: uplift map to stream-power terrain -- read: abstract only (via OpenAlex). Gap: code availability unknown. [unresolved]
- Cordonnier et al. 2018, "Sculpting Mountains", IEEE TVCG -- https://doi.org/10.1109/TVCG.2017.2689022 -- Q3: folding crust model, volumetric uplift map, folded strata -- read: not stated (quoted lines read as abstract text). Gap: code availability unknown. [unresolved]
- Borg, Gain, Guerin, Peytavie, Cani, Galin and Cordonnier, CGF 45(2), 2026, "Authoring Terrestrial Planets with Diffusion Models" -- https://onlinelibrary.wiley.com/doi/10.1111/cgf.70390 -- Q3: generative heightmaps, no tectonic simulation -- read: not stated (described, not quoted). [unresolved]
- Zechmann and Hlavacs 2025, arXiv 2510.24764 -- https://arxiv.org/abs/2510.24764 -- Q3: two Godot planet generators, neither tectonic -- read: not stated. [unresolved]
- H-Schott/StreamPowerErosion -- https://github.com/H-Schott/StreamPowerErosion -- Q3 and Q6: "Large-Scale Stream Power Erosion", MIT, pushed 2026-09-17 -- read: fetched. [unresolved]
- fastscapelib -- https://github.com/fastscape-lem/fastscapelib -- Q3 and Q6: GPL-3.0 C++/Python -- read: fetched. [unresolved]
- Cortial et al. 2020, "Real-Time Hyper-Amplification of Planets" -- no link -- Q3 Gaps: "exists (seen in a ResearchGate listing), but I did not read its content" -- read: not accessible / not read. [unresolved]
- Galin et al. 2019, "A Review of Digital Terrain Modeling" -- no link -- Q3 Gaps: "blocked by HAL's anti-bot page and the abstract was not in Semantic Scholar, so it is not quoted" -- read: not accessible. [unresolved]

### Q4. 3D structural and lithology modelling

- Jessell et al. 2022, "Into the Noddyverse", ESSD 14:381 -- https://essd.copernicus.org/articles/14/381/2022/ -- Q4 and Q5: Noddy point evaluation by unravelling kinematics, event vocabulary (STRAT, FOLD, FAULT, UNC, DYKE, PLUG, SHEAR ZONE, TILT), 4x4x4 km at 20 m (200^3), under 15 s on Xeon Gold 6254, 1 million models in about a week on 20 cores -- read: fetched (quoted). [= Jessell2022]
- Wellmann et al. 2016, pynoddy 1.0, GMD 9:1019 -- https://gmd.copernicus.org/articles/9/1019/2016/ -- Q4: Noddy voxel evaluation in reverse chronological order, discontinuity code, "seconds to minutes on a single core" -- read: fetched (quoted). [= Wellmann2016]
- Loop3D/noddy_qt -- https://github.com/Loop3D/noddy_qt -- Q4: MIT, pushed 2026-09-24, Qt5 port, "None of the calculation code ... changed by the port", random 5-event generator; lineage Imperial 1981-82, Monash 1994, UWA -- read: fetched (quoted). [unresolved]
- Loop3D/noddyverse -- https://github.com/Loop3D/noddyverse -- Q4: MIT, C code to generate random Noddy models, command-line source -- read: fetched (quoted). [= Noddyverse (software)]
- cgre-aachen/pynoddy -- https://github.com/cgre-aachen/pynoddy ; https://pypi.org/project/pynoddy/ -- Q4: GPL-2.0, last PyPI release 1.1 (2017-05-10) -- read: fetched. [unresolved]
- Jessell 1981; Jessell and Valenta 1996 (Noddy displacement equations) -- no links -- Q4 Gaps: "were not read. Porting needs the MIT C source" -- read: not read. [= Jessell1981; = Jessell1996]
- Grose et al. 2021, LoopStructural, GMD 14:3915 -- https://gmd.copernicus.org/articles/14/3915/2021/ -- Q4: implicit interpolation, case study (15 faults; about 1 min map2loop, 8 min model, 3 min rendering on an i7/32 GB laptop) -- read: fetched (quoted). [unresolved]
- Loop3D/LoopStructural -- https://github.com/Loop3D/LoopStructural ; https://pypi.org/project/LoopStructural/ -- Q4: MIT, 290 stars, v1.8.2 on 2026-09-27 -- read: fetched. [unresolved]
- Loop3D/map2loop -- https://github.com/Loop3D/map2loop -- Q4: standalone repo archived -- read: fetched. [unresolved]
- de la Varga et al. 2019, GemPy, GMD 12:1 -- https://gmd.copernicus.org/articles/12/1/2019/ -- Q4: universal cokriging, Theano -- read: fetched. [unresolved]
- gempy-project/gempy -- https://github.com/gempy-project/gempy ; https://pypi.org/project/gempy/ -- Q4: EUPL-1.2, 1,344 stars, PyPI 2026.0.3 -- read: fetched. Gap: no GemPy per-point timing on commodity hardware. [unresolved]
- Fraters et al. 2019, Geodynamic World Builder, Solid Earth 10:1785 -- https://se.copernicus.org/articles/10/1785/2019/ -- Q4: JSON plate-tectonic features, per-point query, LGPLv2.1, about 36 s for a 560,000-element ASPECT model on 480 MPI processes -- read: fetched (quoted). [unresolved]
- GeodynamicWorldBuilder/WorldBuilder -- https://github.com/GeodynamicWorldBuilder/WorldBuilder -- Q4: C++14 and CMake, v1.1.0 (March 2026) -- read: fetched. [unresolved]
- Landlab litholayers.py and lithology.py -- https://github.com/landlab/landlab/blob/master/src/landlab/components/lithology/litholayers.py ; https://github.com/landlab/landlab/blob/master/src/landlab/components/lithology/lithology.py -- Q4: Lithology/LithoLayers representation, MIT -- read: fetched (quoted). [unresolved]
- Salles et al. 2020, goSPL, JOSS -- https://joss.theoj.org/papers/10.21105/joss.02804 -- Q4: dual-lithology sediment routing, horizontal advection -- read: fetched (quoted). [unresolved]
- goSPL (Geodels/gospl) -- https://github.com/Geodels/gospl ; https://pypi.org/project/gospl/ -- Q4: GPL-3.0, PyPI 2026.7.14 -- read: fetched. Gap: "details of goSPL's stratigraphic layer model ... were not verified; the docs landing page did not show them". [unresolved]

### Q5. Scale

- Minecraft Wiki, Dimension type -- https://minecraft.wiki/w/Dimension_type -- Q5: min_y between -2032 and 2031, height 16-4064, overworld -64/384 -- read: fetched (quoted). [unresolved]
- Sources re-cited in Q5 (Cortial 2019, Undiscovered Worlds Classic, Worldbuilding Pasta VIIc, tectonics.cpp design notes, lithosphere.hpp, PyPlatec, Noddyverse ESSD): see Q3 and Q4 above. [unresolved]

### Q6. Verdict

- FSF, "The LGPL and Java" -- https://www.gnu.org/licenses/lgpl-java.html -- Q6: LGPL libraries may be linked to proprietary applications -- read: fetched (quoted). [unresolved]
- Q6 licence table re-cites: GPlates, aspect, underworld2, LaMEM, noddy_qt, tectonics.cpp (links as above) -- read: as in Q1-Q4. Q6 Gaps: "Whether GPL/LGPL obligations bite for Minecraft mods ... has, to my knowledge, never been litigated. The above reflects FSF interpretations, not legal advice." [unresolved]
## 3. docs/research/notes/surface_processes_climate.md

Reading-method note for this document: the scope line says "Repo licence, activity and release data came from the GitHub and PyPI APIs and from the Inria GitLab API on 2026-09-27. Runtimes are quoted from the papers and docs cited; nothing heavy was run locally." The "Gaps" lists say: HAL preprints "sit behind an anti-bot wall (Anubis), and ACM and Wiley returned HTML to scripted downloads"; the web search budget was exhausted mid-task; CSDMS model pages "refused connections (ECONNREFUSED)". Where a bullet is quoted from a README, docs page or source file, I record it as "fetched (quoted)"; where the document says "abstract" I record "abstract only". Section numbers are the document's own (1 to 7).

### 1. Landscape evolution models

- Braun and Willett 2013, Geomorphology 180-181 -- https://www.researchgate.net/publication/236741975_A_very_efficient_On_implicit_and_parallel_method_to_solve_the_stream_power_equation_governing_fluvial_incision_and_landscape_evolution -- sec 1 and 7: O(n) implicit stream-power algorithm, about 10^8 nodes -- read: abstract only (via ResearchGate). [= Braun2013]
- Yuan et al. 2019, JGR Earth Surface -- https://doi.org/10.1029/2018JF004867 -- sec 1 and 4: O(N) implicit erosion-deposition (Davy and Lague), foreland-basin stratigraphy -- read: not stated. [= Yuan2019]
- Braun 2023, JGR Earth Surface -- https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2023JF007140 -- sec 1: implicit algorithm for the threshold stream-power incision model -- read: not stated (Wiley returned HTML to scripted downloads per sec 2 Gaps). [unresolved]
- fastscapelib (fastscape-lem) -- https://github.com/fastscape-lem/fastscapelib ; install docs https://github.com/fastscape-lem/fastscapelib/blob/HEAD/doc/source/install.md ; pyproject https://github.com/fastscape-lem/fastscapelib/blob/HEAD/pyproject.toml ; PyPI https://pypi.org/project/fastscapelib/ -- sec 1: GPL-3.0, header-only C++, pybind11, xtensor, last push 2026-09-07, 46 stars -- read: fetched (API and docs). [= Fastscapelib]
- fastscapelib release notes -- https://fastscapelib.readthedocs.io/en/latest/release_notes.html -- sec 1: v0.2.0 (2023-10-09), v0.3.0 (2025-06-25), v0.4.0 unreleased -- read: fetched. [unresolved]
- fastscapelib include tree -- https://github.com/fastscape-lem/fastscapelib/tree/HEAD/include/fastscapelib -- sec 1: components (spl.hpp, diffusion_adi.hpp, flow code, grids) -- read: fetched. [unresolved]
- fastscapelib guide_flow.md -- https://github.com/fastscape-lem/fastscapelib/blob/HEAD/doc/source/guide_flow.md -- sec 1: PFlood and MST sink resolvers -- read: fetched (quoted). [unresolved]
- fastscapelib guide_eroders.md -- https://github.com/fastscape-lem/fastscapelib/blob/HEAD/doc/source/guide_eroders.md -- sec 1: no marine or sediment modules yet -- read: fetched (quoted). [unresolved]
- fastscapelib why_fastscapelib.md -- https://github.com/fastscape-lem/fastscapelib/blob/HEAD/doc/source/why_fastscapelib.md -- sec 1: "write algorithms once, reuse it everywhere" -- read: fetched (quoted). [unresolved]
- fastscapelib-fortran -- https://github.com/fastscape-lem/fastscapelib-fortran ; docs https://fastscape.org/fastscapelib-fortran/ -- sec 1 and 4: maintenance-only, GPLv3, v2.8.4 (2023-09-29), deposition term G (Yuan 2019), marine transport, FastScape_Strati VTK horizons -- read: fetched (quoted README). [unresolved]
- fastscape (Python) -- https://github.com/fastscape-lem/fastscape ; processes/ https://github.com/fastscape-lem/fastscape/tree/HEAD/fastscape/processes ; main.py https://github.com/fastscape-lem/fastscape/blob/HEAD/fastscape/processes/main.py ; marine.py https://github.com/fastscape-lem/fastscape/blob/HEAD/fastscape/processes/marine.py ; channel.py https://github.com/fastscape-lem/fastscape/blob/HEAD/fastscape/processes/channel.py -- sec 1, 4, 5: BSD-3, v0.1.0 (2023-09-25), process classes, `Sea` process -- read: "read from source". [unresolved]
- xarray-simlab -- https://github.com/xarray-contrib/xarray-simlab -- sec 1: BSD-3, last release 0.5.0 (2021-01-26) -- read: fetched (API). [unresolved]
- orographic-precipitation (fastscape-lem) -- https://github.com/fastscape-lem/orographic-precipitation -- sec 1 and 5: MIT, Smith and Barstad (2004) linear theory, release 1.0 (2020-06-24) -- read: fetched. [= FastscapeLEM]
- Smith and Barstad 2004 -- no link -- sec 5: linear orographic precipitation, Fourier transform "single formula" -- read: secondary source (via the orographic-precipitation repo). [= SmithBarstad2004]
- goSPL JOSS paper (Salles et al. 2020) -- https://www.theoj.org/joss-papers/joss.02804/10.21105.joss.02804.pdf -- sec 1, 4, 5: GPL-3.0 stack, processes (MFD stream power, priority-flood deposition, dual lithology, compaction), forcing with plate tectonics and sea level -- read: fetched (PDF). [unresolved]
- goSPL repository and PyPI -- https://github.com/Geodels/gospl ; https://pypi.org/project/gospl/ -- sec 1: v2026.7.14 (2026-07-10), last push 2026-09-23 -- read: fetched (API). [unresolved]
- goSPL tech guide index -- https://gospl.readthedocs.io/en/latest/tech_guide/index.html -- sec 1 and 3: 2026 additions; soil and regolith production -- read: fetched. [unresolved]
- goSPL ice.rst -- https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/ice.rst -- sec 1: diagnostic glacial-erosion model, SIA removed because it "diverged on real continental runs" -- read: fetched (quoted). [= goSPL]
- goSPL groundwater.rst -- https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/groundwater.rst -- sec 1, 3, 6: water table plus generic duricrust, relief inversion -- read: fetched (quoted). [unresolved]
- goSPL provenance.rst -- https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/provenance.rst -- sec 1 and 4: provenance post-processor -- read: fetched. [unresolved]
- goSPL hpc_scaling.rst -- https://github.com/Geodels/gospl/blob/HEAD/docs/user_guide/hpc_scaling.rst -- sec 1 and 7: 10 km global mesh (5.9 M nodes) cannot start below about 16 ranks, 6.6 GB on heaviest rank, 521 s at 16 ranks and 44 s at 288 ranks, 5 km mesh needs 96 ranks and 24 GB -- read: fetched (quoted; goSPL's own measurements on NCI Gadi). Gap: the number of timed steps is not stated. [unresolved]
- goSPL scaling harness README -- https://github.com/Geodels/gospl/blob/HEAD/scripts/scaling/README.md -- sec 1: step count capped by harness -- read: fetched. [unresolved]
- Salles et al. 2023, Science, global landscape evolution over 100 Myr -- https://doi.org/10.1126/science.add2541 -- sec 1: headline application -- read: abstract only. [unresolved]
- Badlands -- https://github.com/badlands-model/badlands ; https://pypi.org/project/badlands/ ; docs https://badlands.readthedocs.io/en/latest/ -- sec 1 and 4: licence conflict (LGPL-3.0 on GitHub, GPL in PyPI metadata), v2.3.1 (2026-05-01), strata record erodibility and grain size -- read: fetched. [unresolved]
- Geodels GitHub organisation (eSCAPE not present) -- https://github.com/Geodels -- sec 1: repo list, all GPL-3.0 -- read: fetched. [unresolved]
- eSCAPE -- no link -- sec 1: predecessor of goSPL; "Not present in the Geodels GitHub org today" -- read: not accessible as a repo (described via the goSPL JOSS paper). [unresolved]
- Landlab -- https://github.com/landlab/landlab ; https://pypi.org/project/landlab/ ; components https://github.com/landlab/landlab/tree/HEAD/src/landlab/components -- sec 1, 3, 4, 6: MIT, v2.11.0 (2026), component directories; no glacier or karst component -- read: fetched (directory listing). [= Landlab]
- Landlab priority_flood_flow_router.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/priority_flood_flow_router/priority_flood_flow_router.py -- sec 1: wrapper of RichDEM (GPL-3.0) -- read: source read. [unresolved]
- Landlab lake_fill_barnes.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/lake_fill/lake_fill_barnes.py -- sec 1 and 6: LakeMapperBarnes -- read: source read. [unresolved]
- Landlab litholayers.py and lithology.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/lithology/litholayers.py ; https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/lithology/lithology.py -- sec 1 and 4: LithoLayers/Lithology -- read: source read. [unresolved]
- Sequence (Landlab-based) -- https://sequence.readthedocs.io/en/develop/ ; https://pypi.org/project/sequence-model/ -- sec 1 and 4: 2D profile sequence-stratigraphic model, MIT, v0.6.0 (2023-12-20) -- read: fetched. [unresolved]
- TopoToolbox v3: libtopotoolbox, pytopotoolbox, topotoolbox3 -- https://github.com/TopoToolbox/libtopotoolbox ; https://github.com/TopoToolbox/pytopotoolbox ; https://github.com/TopoToolbox/topotoolbox3 -- sec 1: GPL-3.0, tag 2026-W38, v0.0.12, MATLAB release 2026-09-16 -- read: fetched (API). [unresolved]
- TopoToolbox 2 STREAMobj/fastscape.m -- https://github.com/wschwanghart/topotoolbox/blob/master/@STREAMobj/fastscape.m -- sec 1 -- read: fetched (listed). [unresolved]
- RichDEM -- https://github.com/r-barnes/richdem ; https://pypi.org/project/richdem/ -- sec 1 and 6: GPL-3.0, v2.3.1 (2022-02-11), PyPI 0.3.4 from 2018, parallel priority-flood for trillion-cell DEMs (Barnes 2016, 2017) -- read: fetched (quoted README). [unresolved]
- OpenLEM (Hergarten) -- http://hergarten.at/openlem/ -- sec 1 and 5: single C++ file, GPL-2.0-or-later, "continuously developed since 2012", LFPM implementation -- read: fetched (quoted). [unresolved]
- DAGGER and scabbard (Gailleton) -- https://github.com/bgailleton/DAGGER ; https://github.com/bgailleton/scabbard -- sec 1: GPL-3.0 C++ graph backend; MIT Python front-end -- read: fetched. [unresolved]
- CAESAR-Lisflood: HAIL-CAESAR https://github.com/dvalters/HAIL-CAESAR ; GeospatialResearch fork https://github.com/GeospatialResearch/CAESAR-Lisflood ; original https://github.com/tcoulthard/caesar-lisflood -- sec 1: original has no licence file, last push 2017; forks GPL-3.0 -- read: fetched (API). [unresolved]
- Tzathas, Gailleton, Steer and Cordonnier 2024, "Analytical Terrains" (CGF / Eurographics 2024) -- https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf -- sec 1, 2, 3, 7: speed table (512^2 55.5 s, 1024^2 556 s / 460 iterations, 2048^2 4799 s; analytical 1.79, 8.18, 33.65 s), thermal erosion atan 30 degrees, Python+Numba on a Xeon E5-2650 v4 -- read: fetched (author PDF, quoted). [= Tzathas2024]
- Inria analytical-terrains repo LICENSE.md -- https://gitlab.inria.fr/landscapes/analytical-terrains -- sec 2: research-only, "THE USER CANNOT USE, EXPLOIT OR DISTRIBUTE THE SOFTWARE FOR COMMERCIAL PURPOSES" -- read: fetched (quoted; Inria GitLab API). [unresolved]
- FastFlow (Jain, Kerbl, Gain, Finley and Cordonnier; CGF / Pacific Graphics 2024) -- https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf -- sec 1, 2, 6: CPU Cordonnier 2016 needs 2.6 s per 10 implicit iterations at 512^2, GPU 0.1 s, 20x larger time step, under 55 ms up to 4096^2 on an RTX A6000, critique of Schott 2023 -- read: fetched (author PDF, quoted). [= Jain2024]
- fastflow repo LICENSE.md -- https://gitlab.inria.fr/landscapes/fastflow -- sec 2 and 7: same Inria research-only licence -- read: fetched (quoted). [unresolved]
- Cordonnier et al. 2019, ESurf 7:549 (Fig. 5) -- https://esurf.copernicus.org/articles/7/549/2019/esurf-7-549-2019.pdf -- sec 1, 6, 7: depression resolution at 16384^2 about 100-400 s, O(n) MST method 85-120 s per step, compared against RichDEM v2.2.9 -- read: fetched (quoted). [unresolved]

### 2. Computer-graphics research

- Cordonnier et al. 2016, CGF / Eurographics -- https://doi.org/10.1111/cgf.12820 (abstract via https://api.openalex.org/works/doi:10.1111/cgf.12820) -- sec 2: uplift map to stream-power erosion on a stream graph; "first method in computer graphics that combines uplift and hydraulic erosion" -- read: abstract only (via OpenAlex). Gap: no released code found. [unresolved]
- Cordonnier et al. 2017, "Sculpting Mountains", TVCG -- https://doi.org/10.1109/TVCG.2017.2689022 -- sec 2 and 4: layered crust, volumetric uplift map, folded strata on eroded cliffs -- read: not stated (quoted). Gap: no released code found. [unresolved]
- Genevaux et al. 2013, SIGGRAPH -- https://doi.org/10.1145/2461912.2461996 -- sec 2: hydrology-first hierarchical drainage network, analytic terrain -- read: not stated (quoted). Gap: no released code found. [unresolved]
- Schott et al. 2023, ACM TOG 42(5), uplift-domain stream power -- project page https://aparis69.github.io/public_html/projects/schott2023_Uplift.html ; code https://github.com/H-Schott/StreamPowerErosion -- sec 2: uplift domain, MIT C++/CMake -- read: project page fetched; per-iteration timings could not be extracted (sec 2 Gaps). [unresolved]
- WebGPU port ("TerrainX", BSD-3) -- https://github.com/GPU-Gang/WebGPU-Erosion-Simulation -- sec 2: about 40 fps with 4.5k textures on an RTX 3060 Laptop GPU -- read: fetched. [unresolved]
- Jain, Benes and Cordonnier 2024, debris flow, ACM TOG 43(4) -- https://doi.org/10.1145/3658213 ; HAL author PDF https://hal.science/hal-04574826v1/file/2024_Siggraph_Debris_Flow_Author_Version.pdf -- sec 2: debris-flow erosion on steep low-drainage areas -- read: not stated (HAL behind Anubis per Gaps). Gap: no released code found. [unresolved]
- Schott et al. 2024, "Terrain Amplification using Multi-scale Erosion", ACM TOG 43(4) -- https://doi.org/10.1145/3658200 ; code https://github.com/H-Schott/MultiScaleErosion -- sec 2, 3, 7: multi-scale amplification, MIT, requires OpenGL 4.3 -- read: README fetched (quoted); timings not extracted. [= Schott2024]
- Paris et al. 2019, "Terrain Amplification with Implicit 3D Features", ACM TOG -- https://aparis69.github.io/public_html/projects/paris2019_3D.html ; https://github.com/aparis69/Implicit-Volumetric-Terrains -- sec 2: arches and overhangs, MIT -- read: project page fetched. [unresolved]
- Paris et al. 2019, "Desertscape Simulation", CGF / Pacific Graphics -- https://aparis69.github.io/public_html/projects/paris2019_Deserts.html ; https://github.com/aparis69/Desertscapes-Simulation -- sec 2: aeolian dunes, MIT, replicability stamp -- read: project page fetched. [unresolved]
- Argudo et al. 2020, glaciers, ACM TOG (SIGGRAPH Asia 2020) -- https://doi.org/10.1145/3414685.3417855 ; code https://github.com/oargudo/glaciers -- sec 2 and 7: SIA plus procedural amplification, MIT, Qt 5.15 and OpenGL 4.3 -- read: README fetched; timings not extracted. [unresolved]
- Cordonnier et al. 2023, "Forming Terrains by Glacial Erosion", ACM TOG 42(4) -- https://doi.org/10.1145/3592422 ; code https://gitlab.inria.fr/landscapes/glacial-erosion -- sec 2: deep-learning ice-flow estimation; code research-only (license.txt) -- read: not stated; timings not extracted. [unresolved]
- Paris et al. 2023, meandering rivers, ACM TOG (SIGGRAPH Asia 2023) -- https://aparis69.github.io/public_html/projects/paris2023_Meanders.html ; https://github.com/aparis69/Meandering-rivers -- sec 2 and 6: vector meander migration, oxbow cutoffs, avulsions, MIT -- read: project page fetched. [unresolved]
- Galin et al. terrain-modelling survey (and 2025-2026 graphics follow-ups) -- no link -- sec 2 Gaps: "could not be searched, for the same reason" -- read: not accessible / not searched. [unresolved]

### 3. Weathering and regolith

- Landlab ExponentialWeatherer and ExponentialWeathererIntegrated -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/weathering/exponential_weathering.py ; https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/weathering/exponential_weathering_integrated.py -- sec 3: `w0 * exp(-soil__depth / w_star)` "in the style of Ahnert (1976)" -- read: source read. [= Landlab]
- Ahnert 1976 -- no link -- sec 3: named in the Landlab docstring -- read: secondary source (not accessed). [= Ahnert1976]
- Heimsath et al. 1997, "The soil production function and landscape equilibrium" -- https://doi.org/10.1038/41056 -- sec 3: canonical empirical reference for soil production -- read: not accessible ("The Heimsath abstract was unavailable"); rate constants not retrieved. [= Heimsath1997]
- Braun et al. 2016, regolith by chemical weathering, JGR Earth Surface -- https://doi.org/10.1002/2016JF003914 (abstract via OpenAlex) -- sec 3 and 7: weathering front advances proportional to pore fluid velocity; thickness controlled by dimensionless number; erosion/weathering timescale ratio in orogens -- read: abstract only, truncated ("the Braun 2016 full text (coefficients) was not accessible"). [= Braun2016]
- Laterite (Wikipedia) -- https://en.wikipedia.org/wiki/Laterite -- sec 3: formation conditions, bauxite vs iron-rich laterite, caps about 30 m, lateritization about 35 to 1.5 Ma -- read: fetched (quoted). [= Wikipedia-Laterite]
- Landlab threshold_eroder.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/threshold_eroder/threshold_eroder.py -- sec 3: slope threshold Sc -- read: source read. [= Landlab]
- Paris et al. 2021, cave networks -- https://aparis69.github.io/public_html/projects/paris2021_Karsts.html ; https://github.com/aparis69/Karst-Synthesis -- sec 3: anisotropic shortest-path skeletons, SDF construction trees; MIT code "recoded ... to make sure it is free to use", results and timings may differ -- read: project page and README fetched. [unresolved]
- ExoPlaSim source docstrings (exoplasim/__init__.py) -- https://github.com/alphaparrot/ExoPlaSim/blob/HEAD/exoplasim/__init__.py -- sec 3 and 5: `co2weathering`, `erosionsupplylimit`, landmap/topomap options -- read: source read (quoted). [unresolved]

### 4. Sedimentation and stratigraphy

- goSPL strat.rst -- https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/strat.rst -- sec 4: per-node layer elevation, thickness, porosity; compaction trends -- read: fetched (quoted). [unresolved]
- goSPL dep.rst -- https://github.com/Geodels/gospl/blob/HEAD/docs/tech_guide/dep.rst -- sec 4 and 6: priority-flood returns volume, spill-over node, basin id; Barnes (2016) three-phase approach -- read: fetched (quoted). [unresolved]
- Landlab carbonate_producer.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/carbonate/carbonate_producer.py -- sec 4: Bosscher and Schlager (1992) light-limited growth law -- read: source read. [= Landlab]
- Bosscher and Schlager 1992 -- no link -- sec 4: growth law `G = Gm tanh(I0 e^{-kz}/Ik)` -- read: secondary source (via the Landlab docstring). [= Bosscher1992]
- Landlab simple_submarine_diffuser.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/marine_sediment_transport/simple_submarine_diffuser.py -- sec 4 -- read: source read. [= Landlab]
- Landlab geoenthalpy_delta.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/geoenthalpy_delta/geoenthalpy_delta.py -- sec 4 and 6 -- read: source read. [unresolved]
- Landlab concentration_tracker_for_space.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/concentration_tracker/concentration_tracker_for_space.py -- sec 4 -- read: source read. [unresolved]
- SedFlux (mcflugen) -- https://github.com/mcflugen/sedflux -- sec 4: stratigraphic basin-filling model in C, MIT, last push 2025-09-11 -- read: fetched. [unresolved]
- pyDeltaRCM -- https://github.com/DeltaRCM/pyDeltaRCM ; https://pypi.org/project/pyDeltaRCM/ -- sec 4 and 6: reduced-complexity delta model, MIT, v2.3.0 (2026-07-30), JOSS -- read: README only (Gap: "stratigraphy-recording features were not verified beyond the README"). [unresolved]
- Facies rules (sandstone, shale, limestone, evaporite, coal, laterite) -- no source -- sec 4 Gaps: "my proposal, not a published scheme that was checked" -- read: n/a (author proposal). [unresolved]

### 5. Climate

- ExoPlaSim (Paradise et al., MNRAS 2022) -- https://arxiv.org/abs/2107.07685 ; https://github.com/alphaparrot/ExoPlaSim ; https://pypi.org/project/exoplasim/ ; https://exoplasim.readthedocs.io/en/latest/ -- sec 5 and 7: T21 speeds (30-60 s per year on cluster; 3.75 min per year on a 2017 i7-7700HQ laptop with 4 MPI threads), T42 15-16 min per year, T170 about 60 h per year, 50-200 years spin-up, GPL-2.0, v3.4.2 (2025-01-02) -- read: fetched (quoted). [unresolved]
- PLASIM mirror (HartmutBorth/PLASIM) -- https://github.com/HartmutBorth/PLASIM -- sec 5: no detected licence, last pushed 2019 -- read: fetched (API). [unresolved]
- Worldbuilding Pasta ExoPlaSim guide (Part VI) -- https://worldbuildingpasta.blogspot.com/2021/11/an-apple-pie-from-scratch-part-vi.html -- sec 5: T21 "10 minutes to over an hour for 10 simulated years", `runtobalance()`, `koppenpasta.py`, biases -- read: fetched (quoted). [unresolved]
- SpeedyWeather.jl -- README https://github.com/SpeedyWeather/SpeedyWeather.jl ; LICENSE https://github.com/SpeedyWeather/SpeedyWeather.jl/blob/main/LICENSE ; JOSS 2024 https://www.theoj.org/joss-papers/joss.06323/10.21105.joss.06323.pdf -- sec 5 and 7: about 40 s per year on one M4 core at T31, EUPL-1.2 since 2026-03-02, v0.22.1 (2026-08-21), conflicting GPU claims -- read: fetched (quoted). [unresolved]
- SPEEDY (Fortran) -- no link -- sec 5: reported at 240 simulated years per day -- read: secondary source (via SpeedyWeather README). [unresolved]
- Isca (ExeClim) -- https://github.com/ExeClim/Isca -- sec 5: GPL-3.0, Held-Suarez to multi-band radiation, last tag v1.0 (2018) -- read: fetched (README). Gap: laptop runtimes not found. [unresolved]
- climlab -- https://github.com/climlab/climlab ; https://pypi.org/project/climlab/ -- sec 5: MIT, v0.9.2 (2026-03-18), EBM models -- read: fetched. [unresolved]
- LFPM (Hergarten and Robl 2022, GMD) -- https://gmd.copernicus.org/articles/15/2063/2022/gmd-15-2063-2022.pdf -- sec 5 and 7: linear-time, about 2.4x the simplest stream-power step (2.2x the shared stream-power model) -- read: fetched (quoted). [unresolved]
- Koppen climate classification (Wikipedia) -- https://en.wikipedia.org/wiki/K%C3%B6ppen_climate_classification -- sec 5: Koppen-Geiger rules -- read: fetched (quoted). [= WikipediaKoppen]
- Worldbuilding Pasta Part VIb -- https://worldbuildingpasta.blogspot.com/2020/05/an-apple-pie-from-scratch-part-vib.html -- sec 5: parametric hand method (wet/dry belts, gyres, lapse rate 4.46 C per km quoted from Clima-Sim, rain inland 2000 km) -- read: fetched (quoted). [= WorldbuildingPasta]
- Clima-Sim -- no link -- sec 5: source of the 4.46 C/km lapse rate in Part VIb -- read: secondary source. [unresolved]
- Undiscovered Worlds blog -- https://undiscoveredworlds.blogspot.com/2019/01/what-is-undiscovered-worlds.html ; Classic repo https://github.com/JonathanCRH/Undiscovered_Worlds_Classic -- sec 5: Koppen zones, rivers, lakes, deltas, glaciers; GPL-3.0 C++ -- read: fetched. [unresolved]
- worldengine -- https://github.com/Mindwerks/worldengine -- sec 5: "plate simulations, erosion, rain shadows, Holdridge life zones", MIT -- read: fetched (quoted). [unresolved]
- goSPL ela_from_temperature.py -- https://github.com/Geodels/gospl/blob/HEAD/gospl/tools/ela_from_temperature.py -- sec 5: `ELA(x) = z(x) + (T(x) - T_ELA)/Gamma` per paleo-climate snapshot -- read: source read (quoted). [= gosplDocs]
- ROCKE-3D -- no link -- sec 5 Gaps: "was not researched (search budget exhausted)" -- read: not accessible / not researched. [unresolved]
- Haq and Miller sea-level curves -- no link -- sec 5 Gaps: "No cited source was obtained ... for published Phanerozoic sea-level curves (Haq, Miller) as data" -- read: not obtained. [unresolved]
- CHILD (CSDMS) -- no link -- sec 1 Gaps: "could not be assessed. The CSDMS model pages refused connections (ECONNREFUSED) and no CHILD repository turned up on GitHub" -- read: not accessible. [unresolved]

### 6. Hydrology

- Barnes et al. 2014, Priority-Flood, arXiv 1511.04463 -- https://arxiv.org/abs/1511.04463 -- sec 6: O(n log n) general variant, improved variant 16.8% faster, tested on about 8x10^9 cells of Minnesota LiDAR -- read: fetched (quoted). [= Barnes2014]
- Barnes 2016, 2017 (parallel priority-flood and flow accumulation for trillion-cell DEMs) -- no separate link -- sec 6 -- read: secondary source (via RichDEM README and goSPL dep.rst). [unresolved]
- Barnes, Callaghan and Wickert 2021, Fill-Spill-Merge, ESurf 9:105 -- https://esurf.copernicus.org/articles/9/105/2021/ ; code https://github.com/r-barnes/Barnes2020-FillSpillMerge -- sec 6: runoff fills, spills and merges depressions; 90-2600x faster than Jacobi iteration; MIT C++ -- read: fetched (quoted). [= Barnes2021]
- Landlab hand_calculator.py -- https://github.com/landlab/landlab/blob/HEAD/src/landlab/components/hand_calculator/hand_calculator.py -- sec 6: HAND, Nobre et al. (2011) -- read: source read. [unresolved]
- Nobre et al. 2011 -- no link -- sec 6: height above the nearest drainage -- read: secondary source. [unresolved]
- GraphFlood 1.0 (Gailleton et al. 2024), ESurf 12:1295 -- https://esurf.copernicus.org/articles/12/1295/2024/esurf-12-1295-2024.pdf -- sec 6 and 7: stationary shallow-water on the flow DAG, about 10x faster than River.lab, 83 Mpx DEM about 20 h on a 32 GB laptop -- read: fetched (quoted). [unresolved]
- Minecraft Wiki, Aquifer -- https://minecraft.wiki/w/Aquifer -- sec 6: aquifers from 16x12x16 cells, lava at or below Y = -10, `aquifers` object may be omitted -- read: fetched (quoted). Gap: 1.21.1 aquifer datapack schema "not verified" (wiki text is current, not 1.21.1). [unresolved]
- Minecraft Wiki, Dimension type -- https://minecraft.wiki/w/Dimension_type -- sec 6: min_y -2032..2031, height up to 4064 -- read: fetched. [unresolved]

### 7. Licences and verdict

- IGM (instructed-glacier-model) -- https://github.com/instructed-glacier-model/igm -- sec 7: GPL-3.0 licence list -- read: fetched (licence only). [unresolved]
- Inria licence text (fastflow LICENSE.md) -- https://gitlab.inria.fr/landscapes/fastflow -- sec 7 -- read: fetched. [unresolved]
- GPL FAQ #IfLibraryIsGPL -- https://www.gnu.org/licenses/gpl-faq.html#IfLibraryIsGPL -- sec 7: linking to a GPL library -- read: fetched (quoted). [unresolved]
- GPL FAQ #GPLPlugins -- https://www.gnu.org/licenses/gpl-faq.html#GPLPlugins -- sec 7: fork/exec separate programs -- read: fetched (quoted). [unresolved]
- GPL FAQ #GPLInProprietarySystem -- https://www.gnu.org/licenses/gpl-faq.html#GPLInProprietarySystem -- sec 7 -- read: fetched (quoted). [unresolved]
- EUPL compatibility matrix -- https://interoperable-europe.ec.europa.eu/collection/eupl/matrix-eupl-compatible-open-source-licences -- sec 7: EUPL-1.2 compatible downstream licences, "no specific case law" -- read: fetched (quoted). [unresolved]
- JEP 454 (FFM API final in JDK 22) -- https://openjdk.org/jeps/454 -- sec 7: FFM in JDK 21 was preview JEP 442 -- read: fetched. [unresolved]
- OpenGL (Wikipedia) -- https://en.wikipedia.org/wiki/OpenGL -- sec 2 and 7: compute shaders in OpenGL 4.3, macOS stops at 4.1, deprecated 2018 -- read: fetched. [unresolved]
- Sec 7 Gap: "The GPL-in-Minecraft question (a GPL mod linking a proprietary game) was not researched." [unresolved]
## 4. docs/research/notes/ore_genesis_modeling.md

Reading-method note for this document (its own "Method note"): "the session's WebSearch budget was exhausted early, so sources were retrieved directly: OpenAlex/Semantic Scholar abstracts (DOI links below), the USGS Publications Warehouse API and report PDFs (full text parsed locally), USGS Bulletin 1693 full text (47 descriptive models parsed), the Create Ore Excavation 1.6.8 jar shipped in the MIC pack, and sub-agent fetches of GitHub sources and wikis. Wherever only a title could be verified, a claim is marked '(title-level)'." Abbreviations: "B1693 m17" = USGS Bulletin 1693 (Cox and Singer eds. 1986) model 17, https://pubs.usgs.gov/bul/1693/report.pdf; "AV100" = Economic Geology 100th Anniversary Volume (2005). Unless a bullet says otherwise, a DOI-linked paper is recorded as "abstract only (OpenAlex/Semantic Scholar, per the Method note)" and a B1693 model as "full text parsed". The document's own Gaps lists are applied: papers whose abstract was unavailable are marked "title-level" or "not accessible". Sections are Q1 to Q7 as in the document; subsections of Q1 are lettered A to H (tables) as in the document.

### Q1. Deposit catalog by setting and process

Q1 table A (arc and granite-related systems)
- Sillitoe 2010 -- https://doi.org/10.2113/gsecongeo.105.1.3 -- Q1 A (porphyry Cu, epithermal), also table rows in the report; porphyry/epithermal settings, depths, exhumation -- read: abstract only. [= Sillitoe2010]
- USGS Bulletin 1693 (Cox and Singer eds. 1986), models m17, m20c, m25a-d, m18b, m18d, m14b, m15b, m15c, m16, m24a, m28a, m31a, m32a, m32b, m30b, m37a, m30c, m36a, m36b, m29b, m10, m5a, m6a, m1, m2b, m8a, m12, m28b, m34a, m29a, m34c, m34d, m38a, m38b, m38c, m39a-e, m31c -- https://pubs.usgs.gov/bul/1693/report.pdf -- Q1 A-H and elsewhere: descriptive deposit models (settings, age, hosts, indicators, weathering) -- read: full text parsed ("47 descriptive models parsed"). [= Cox1986]
- Cooke et al. 2005 -- https://doi.org/10.2113/gsecongeo.100.5.801 -- Q1 A and F: porphyry Cu, Pacific giants / supergene -- read: abstract only. [= Cooke2005]
- Seedorff et al. 2005 (AV100) -- https://doi.org/10.5382/av100.10 -- Q1 A: porphyry deposits -- read: abstract only. [= Seedorff2005]
- Kesler and Wilkinson 2006 -- https://doi.org/10.2113/gsecongeo.101.5.919 -- Q1 A, C: exhumation, preservation -- read: abstract only. [= Kesler2006]
- Simmons et al. 2005 (AV100) -- https://doi.org/10.5382/av100.16 -- Q1 A: epithermal Au-Ag -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-Q -- https://pubs.usgs.gov/publication/sir20105070Q -- Q1 A: epithermal Au-Ag -- read: USGS Publications Warehouse (not further stated). [unresolved]
- Meinert et al. 2005 (AV100) -- https://doi.org/10.5382/av100.11 -- Q1 A: skarns -- read: abstract only. [unresolved]
- Lehmann 2020 -- https://doi.org/10.1016/j.lithos.2020.105756 -- Q1 A and G: Sn-W granites, cassiterite placers -- read: abstract only. [= Lehmann2020]
- Kesler and Wilkinson 2013 -- https://doi.org/10.1144/sp393.3 -- Q1 A: Sn-W -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-D -- https://pubs.usgs.gov/publication/sir20105070D -- Q1 A: porphyry Mo -- read: USGS Publications Warehouse. [unresolved]

Q1 table B (seafloor and basin-brine systems)
- Franklin et al. 2005 (AV100) -- https://doi.org/10.5382/av100.17 -- Q1 B: VMS types -- read: abstract only. [unresolved]
- Hannington et al. 2005 (AV100) -- https://doi.org/10.5382/av100.06 -- Q1 B: modern vent sites (65% MOR, 22% back-arc, 12% arc) -- read: abstract only. [unresolved]
- Huston et al. 2010 -- https://doi.org/10.2113/gsecongeo.105.3.571 -- Q1 B and secular switches: VMS peaks with supercontinent assembly; SEDEX/CD ages -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-C -- https://pubs.usgs.gov/publication/sir20105070C -- Q1 B: VMS -- read: USGS Publications Warehouse. [unresolved]
- USGS OFR 2009-1034 -- https://pubs.usgs.gov/of/2009/1034/ -- Q1 B: VMS (felsic-type footprint statistics) -- read: USGS Publications Warehouse. [unresolved]
- Leach et al. 2010 -- https://doi.org/10.2113/gsecongeo.105.3.593 -- Q1 B and switches: SEDEX/CD and MVT Pb-Zn, none before 2.02 Ga -- read: abstract only. [= Leach2010b]
- Hoggard et al. 2020 -- https://doi.org/10.1038/s41561-020-0593-2 -- Q1 B: 85% of sediment-hosted base-metal deposits within 200 km of a thick-thin lithosphere transition -- read: abstract only. [= Hoggard2020]
- USGS SIR 2010-5070-N -- https://pubs.usgs.gov/publication/sir20105070N -- Q1 B: SEDEX -- read: USGS Publications Warehouse. [= Emsbo2016]
- Large et al. 2005 (AV100) -- https://doi.org/10.5382/av100.28 -- Q1 B: SEDEX -- read: abstract only. [unresolved]
- Garven 1985 -- https://doi.org/10.2113/gsecongeo.80.2.307 -- Q1 B: MVT brine flow -- read: abstract only. [= Garven1985]
- Bethke 1986 -- https://doi.org/10.2113/gsecongeo.81.2.233 -- Q1 B: MVT -- read: abstract only. [unresolved]
- Appold and Garven 1999 -- https://doi.org/10.2113/gsecongeo.94.6.913 -- Q1 B: MVT (Southeast Missouri ore window of 100s of kyr) -- read: abstract only. [unresolved]
- Cathles and Adams 2005 (AV100) -- https://doi.org/10.5382/av100.05 -- Q1 B: MVT and unconformity U fluid flow -- read: abstract only. [unresolved]
- Hitzman et al. 2010 -- https://doi.org/10.2113/gsecongeo.105.3.627 -- Q1 B and switches: sediment-hosted stratiform Cu -- read: abstract only. [unresolved]
- Hitzman et al. 2005 (AV100) -- https://doi.org/10.5382/av100.19 -- Q1 B: sediment-hosted stratiform Cu -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-M -- https://pubs.usgs.gov/publication/sir20105070M -- Q1 B: sediment-hosted Cu -- read: USGS Publications Warehouse. [unresolved]
- Cuney 2010 -- https://doi.org/10.2113/gsecongeo.105.3.553 -- Q1 B, E, switches: unconformity and roll-front U, uranium age steps (3.1-2.2 Ga, 2.2-0.45 Ga, 0.45 Ga-present), land plants -- read: abstract only. [unresolved]
- USGS 2017 Texas roll-type U -- https://pubs.usgs.gov/publication/70179186 -- Q1 B: roll-front U (Texas climate) -- read: USGS Publications Warehouse. [unresolved]
- USGS Bulletin 2141 -- https://pubs.usgs.gov/publication/b2141 -- Q1 B: roll-front U -- read: USGS Publications Warehouse. [unresolved]

Q1 table C (orogenic, craton-margin, pegmatite)
- Goldfarb et al. 2005 (AV100) -- https://doi.org/10.5382/av100.14 -- Q1 C: orogenic Au -- read: abstract only (names subtypes "without giving depths in its abstract"). [unresolved]
- Robert et al. 2005 (AV100) -- https://doi.org/10.5382/av100.30 -- Q1 C: orogenic Au -- read: abstract only. [= Robert2005]
- S.F. Cox 2005 (AV100) -- https://doi.org/10.5382/av100.04 -- Q1 C: orogenic Au, permeability -- read: abstract only. [unresolved]
- Groves et al. 2010 -- https://doi.org/10.2113/gsecongeo.105.3.641 -- Q1 C and switches: IOCG giants 100-200 Myr after assembly -- read: abstract only. [unresolved]
- Williams et al. 2005 (AV100) -- https://doi.org/10.5382/av100.13 -- Q1 C: IOCG -- read: abstract only. [unresolved]
- Cline et al. 2005 (AV100) -- https://doi.org/10.5382/av100.15 -- Q1 C: Carlin-type Au -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-O -- https://pubs.usgs.gov/publication/sir20105070O -- Q1 C and H: pegmatites -- read: USGS Publications Warehouse. [unresolved]
- Cerny and Ercit 2005 -- https://doi.org/10.2113/gscanmin.43.6.2005 -- Q1 C: pegmatite classification -- read: abstract only. [unresolved]
- McCauley and Bradley 2014 -- https://doi.org/10.3749/canmin.52.2.183 -- Q1 C and switches: pegmatite age peaks match supercontinent assembly -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-J -- https://pubs.usgs.gov/publication/sir20105070J -- Q1 C: carbonatite REE-Nb-P -- read: USGS Publications Warehouse. [unresolved]
- Freyssinet et al. 2005 (AV100) -- https://doi.org/10.5382/av100.21 -- Q1 C, E, F: lateritic and supergene deposits -- read: abstract only. [unresolved]

Q1 table D (orthomagmatic)
- Barnes and Lightfoot 2005 (AV100) -- https://doi.org/10.5382/av100.08 -- Q1 D: Ni-Cu-PGE sulfide -- read: abstract only. [unresolved]
- Arndt et al. 2005 (AV100) -- https://doi.org/10.5382/av100.02 -- Q1 D: komatiite Ni -- read: abstract only. [unresolved]
- Naldrett 2010 -- https://doi.org/10.2113/gsecongeo.105.3.669 -- Q1 D and switches: Ni-Cu-PGE, komatiite Ni 2.7-1.9 Ga, PGE reefs -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-I -- https://pubs.usgs.gov/publication/sir20105070I -- Q1 D and Gaps: magmatic Ni sulfides, "approximately 60 percent of the world's nickel production" -- read: USGS Publications Warehouse. [unresolved]
- Griffin et al. 2013 (title-level: continental-root control) -- https://doi.org/10.1038/ngeo1954 -- Q1 D -- read: title-level only (Gaps). [unresolved]
- Cawthorn et al. 2005 (AV100) -- https://doi.org/10.5382/av100.09 -- Q1 D: stratiform chromite and PGE reefs -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-E -- https://pubs.usgs.gov/publication/sir20105070E -- Q1 D: PGE layered intrusions -- read: USGS Publications Warehouse. [unresolved]
- USGS SIR 2012-5157 -- https://pubs.usgs.gov/publication/sir20125157 -- Q1 D: podiform chromite -- read: USGS Publications Warehouse. [unresolved]
- Gurney et al. 2010 -- https://doi.org/10.2113/gsecongeo.105.3.689 -- Q1 D: diamonds -- read: abstract only. [unresolved]
- Gurney et al. 2005 (AV100) -- https://doi.org/10.5382/av100.07 -- Q1 D: diamonds -- read: abstract only. [unresolved]
- Kerrich et al. 2005 (AV100) -- https://doi.org/10.5382/av100.33 -- Q1 D and switches: supercontinent cycle since ~2.8 Ga -- read: abstract only. [unresolved]
- USGS diamond pipe models 1992 -- https://pubs.usgs.gov/publication/70016991 -- Q1 D: kimberlite/lamproite -- read: USGS Publications Warehouse. [unresolved]
- USGS SIR 2010-5070-K -- https://pubs.usgs.gov/publication/sir20105070K -- Q1 D: Fe-Ti(-V) oxide -- read: USGS Publications Warehouse. [unresolved]
- GIA diamond-geology PDF (link not given) -- Q1 Gaps: "the GIA diamond-geology PDF link returned HTML" -- read: not accessible. [unresolved]

Q1 table E (chemical and biogenic sediments, fuels)
- Bekker et al. 2010 -- https://doi.org/10.2113/gsecongeo.105.3.467 -- Q1 E and switches: BIF, acme 2.65-1.85 Ga -- read: abstract only. [= Bekker2010]
- Clout and Simonson 2005 (AV100) -- https://doi.org/10.5382/av100.20 -- Q1 E: BIF -- read: abstract only. [unresolved]
- Groves et al. 2005 (Geological Society Special Publication 248) -- https://doi.org/10.1144/gsl.sp.2005.248.01.04 -- Q1 E and switches: redox-restricted deposit types, buoyant Archean SCLM -- read: abstract only. [unresolved]
- Frimmel et al. 2005 (AV100) -- https://doi.org/10.5382/av100.23 -- Q1 E: Witwatersrand quartz-pebble conglomerate -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-P -- https://pubs.usgs.gov/publication/sir20105070P -- Q1 E: quartz-pebble conglomerate Au-U -- read: USGS Publications Warehouse. [unresolved]
- USGS SIR 2010-5090-S -- https://pubs.usgs.gov/publication/sir20105090S -- Q1 E: potash evaporites (Elk Point basin, 15-45 degrees N/S) -- read: full text. [unresolved]
- USGS SIR 2010-5090-CC -- https://pubs.usgs.gov/publication/sir20105090CC -- Q1 E: evaporites -- read: USGS Publications Warehouse. [unresolved]
- Warren 2010 -- https://doi.org/10.1016/j.earscirev.2009.11.004 -- Q1 E: evaporites -- read: title-level only. [unresolved]
- Cook and McElhinny 1979 -- https://doi.org/10.2113/gsecongeo.74.2.315 -- Q1 E: phosphorite -- read: abstract only. [unresolved]
- Bohacs and Suter 1997 -- https://doi.org/10.1306/3b05c3fc-172a-11d7-8645000102c1865d -- Q1 E: coal accommodation vs peat production -- read: abstract only. [= Bohacs1997]
- Ryer and Langer 1980 -- https://doi.org/10.1306/212f7b44-2b24-11d7-8648000102c1865d -- Q1 E: coal (Utah peat compaction 11:1) -- read: abstract only. [= Ryer1980]
- Nelsen et al. 2016 -- https://doi.org/10.1073/pnas.1517943113 -- Q1 E and switches: land plants, Carboniferous coal peak -- read: abstract only. [= Nelsen2016]
- SLB glossary: petroleum system, critical moment, catagenesis -- https://glossary.slb.com/en/terms/p/petroleum_system ; https://glossary.slb.com/en/terms/c/critical_moment ; https://glossary.slb.com/en/terms/c/catagenesis -- Q1 E: petroleum system elements, critical moment, 50-150 C catagenesis -- read: fetched (glossary pages). [unresolved]
- Klemme and Ulmishek 1991 -- https://doi.org/10.1306/0c9b2a47-1710-11d7-8645000102c1865d -- Q1 E and switches: six source-rock intervals (Silurian 9%, ... middle Cretaceous 29%) -- read: abstract only. [unresolved]
- Magoon and Dow 1994 -- https://doi.org/10.1306/m60585 -- Q1 E: petroleum system -- read: abstract only. [= Magoon1994]

Q1 table F (supergene and residual)
- Sillitoe 2005 (AV100) -- https://doi.org/10.5382/av100.22 -- Q1 F: supergene Cu enrichment (3-9 Myr, chalcocite blanket 2-3x hypogene) -- read: abstract only. [= Sillitoe2005]
- Hartley and Rice 2005 (title-level) -- https://doi.org/10.1007/s00126-005-0017-7 -- Q1 F -- read: title-level only. [unresolved]
- Butt and Cluzel 2013 -- https://doi.org/10.2113/gselements.9.2.123 -- Q1 F and Gaps: Ni-Co laterite, "over 60% of global nickel supply" -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-H -- https://pubs.usgs.gov/publication/sir20105070H -- Q1 F: Ni-Co laterite -- read: USGS Publications Warehouse. [unresolved]
- USGS OFR 2011-1058 -- https://pubs.usgs.gov/of/2011/1058/ -- Q1 F: Ni laterite -- read: USGS Publications Warehouse. [unresolved]

Q1 table G (placers) and H (gemstones)
- Garnett and Bassett 2005 (AV100) -- https://doi.org/10.5382/av100.25 -- Q1 G: placers, mostly Cenozoic -- read: abstract only. [= Garnett2005]
- USGS SIR 2010-5070-L -- https://pubs.usgs.gov/publication/sir20105070L -- Q1 G: heavy-mineral sands -- read: USGS Publications Warehouse. [unresolved]
- Giuliani and Groat 2019 -- https://doi.org/10.5741/gems.55.4.464 ; PDF https://www.gia.edu/doc/WN19-geology-of-corundum-and-emerald-gem-deposits.pdf -- Q1 H: corundum and emerald deposit types -- read: abstract only (PDF link listed). [unresolved]

Q1 secular switches and Gaps
- Goldfarb, Bradley and Leach 2010 -- https://doi.org/10.2113/gsecongeo.105.3.459 -- Q1 switches: deposit types time-bound; uneven preservation -- read: abstract only. [unresolved]
- Meyer 1988 -- https://doi.org/10.1146/annurev.ea.16.050188.001051 -- Q1 switches: "see also" -- read: not stated. [unresolved]
- Groves et al. 1998 -- https://doi.org/10.1016/s0169-1368(97)00012-7 -- Q1 Gaps: orogenic Au depth subtypes -- read: not accessible ("abstract ... unavailable"). [unresolved]
- Goldfarb et al. 2001 -- https://doi.org/10.1016/s0169-1368(01)00016-6 -- Q1 Gaps -- read: not accessible ("abstract ... unavailable"). [unresolved]
- Hagemann 2016, Diaz-Rodriguez 2021, Yanites and Kesler 2015 -- no links given in Q1 -- Q1 Gaps: "unavailable (no abstract in OpenAlex or Semantic Scholar, paywalled) and are cited at title level only" -- read: title-level only. [unresolved]
- Not covered by retrieved sources (Q1 Gaps): opal, turquoise, jade/nephrite, lithium brines (only a 2026 USGS map report found), geothermal and hydrothermal Mn, seafloor Mn nodules. [unresolved]
### Q2. Quantitative sizing (USGS deposit models, grade-tonnage, three-part method, datasets)

- Singer 1993 (USGS, three-part method) -- https://pubs.usgs.gov/publication/70018261 ; DOI https://doi.org/10.1007/bf02272804 -- Q2: method in use since 1975 (27 areas, 5x10^6 km2 by 1993), permissive-boundary probability below 1 in 100,000 to 1,000,000 -- read: abstract only (USGS record / DOI). [= Singer1993]
- Root, Menzie and Scott 1992 -- https://doi.org/10.1007/bf01782266 -- Q2: Monte Carlo combination -- read: abstract only. [= Root1992]
- USGS TM 7-C18 (MapMark4 and MapMark4GUI) -- https://pubs.usgs.gov/publication/tm7C18 -- Q2: current USGS tooling -- read: USGS Publications Warehouse. [unresolved]
- USGS TM 7-C21 (ATA GUI, tract aggregation) -- https://pubs.usgs.gov/publication/tm7C21 -- Q2 -- read: USGS Publications Warehouse. [unresolved]
- USGS 2025 (modified MapMark4, correlated grade-tonnage simulation) -- https://pubs.usgs.gov/publication/70266112 -- Q2 -- read: USGS Publications Warehouse. [unresolved]
- USGS SIR 2026-5031 (pooled vs tract data, 16 assessments) -- https://pubs.usgs.gov/publication/sir20265031 -- Q2 -- read: USGS Publications Warehouse. [unresolved]
- USGS Bulletin 1693 (grade-tonnage plots) -- https://pubs.usgs.gov/bul/1693/report.pdf -- Q2: base catalogue; "grade-tonnage plots are figures only; the medians are not in the text layer" -- read: full text parsed (medians not extractable). [= Cox1986]
- USGS SIR 2010-5070 chapters A-R (index entry B) -- https://pubs.usgs.gov/publication/sir20105070B -- Q2: updated deposit-model chapters (MVT, porphyry Cu, VMS, porphyry Mo, stratiform chromite, Be, Co-Cu-Au, Ni-Co laterite, Ni-Cu-PGE, REE, Fe-Ti, heavy-mineral sands, sediment-hosted Cu, sedex, LCT pegmatite, QPC Au, epithermal, alkalic epithermal) -- read: USGS Publications Warehouse. [unresolved]
- Singer, Berger and Menzie 2005 (porphyry Cu densities; the report calls the first author Singer et al.) -- https://doi.org/10.2113/gsecongeo.100.3.491 -- Q2 and report: 2-128 deposits per 100,000 km2, P90 >= 4, P50 >= 15, P10 >= 35; 2 km merge rule -- read: abstract only. [= Singer2005]
- Singer 2008 (general density treatment; title-level) -- https://doi.org/10.1007/s11004-007-9127-3 -- Q2 -- read: title-level only; Gaps: "the abstract was unavailable". [= Singer2008b]
- USGS Sierra Maestra (Cuban arc check, 3.2 vs 3.9 deposits) -- https://pubs.usgs.gov/publication/70212544 -- Q2 -- read: USGS Publications Warehouse. [unresolved]
- Agterberg 1995 -- https://doi.org/10.1080/00206819509465388 -- Q2: lognormal bulk with Pareto tail -- read: abstract only. [= Agterberg1995]
- Singer 1995 -- https://doi.org/10.2113/gsecongeo.90.1.88 -- Q2: world-class deposits hold >86% of gold; thresholds 100 t Au, 2,400 t Ag, 2 Mt Cu, 1.7 Mt Zn, 1 Mt Pb -- read: abstract only. [= Singer1995]
- Robert et al. 2005 -- https://doi.org/10.5382/av100.30 -- Q2 and Q1: orogenic-gold camps spaced 30-50 km, Pareto size distributions -- read: abstract only. [= Robert2005]
- Hronsky and Groves 2008 -- https://doi.org/10.1080/08120090701581356 -- Q2 and Q3: power-law deposit sizes within provinces; multiplicative ranking, "Elephant Country" vs "First Mover" -- read: abstract only. [= Hronsky2008]
- USGS OFR 2008-1155 (porphyry Cu grade-tonnage, n = 422: tonnage 33/240/1,500 Mt; Cu 0.24/0.44/0.80%; Au-tonnage r = -0.28) -- https://pubs.usgs.gov/of/2008/1155/ -- Q2 table -- read: full text parsed ("figure intercepts" for tonnage). [= Singer2008a]
- USGS OFR 2009-1034 (VMS, felsic n = 421; bimodal-mafic n = 272) -- https://pubs.usgs.gov/of/2009/1034/ -- Q2 table -- read: full text parsed. [unresolved]
- USGS OFR 2011-1058 (Ni-Co laterite, n = 117: 9.0/56/290 Mt) -- https://pubs.usgs.gov/of/2011/1058/ -- Q2 table; Q5 -- read: full text parsed. [unresolved]
- USGS SIR 2012-5157 (podiform chromite, n = 619) -- https://pubs.usgs.gov/publication/sir20125157 -- Q2 table; 100 m same-deposit rule -- read: full text parsed. [unresolved]
- USGS 1992 (diamond kimberlite pipe models; median 26 Mt, 0.25 ct/t) -- https://pubs.usgs.gov/publication/70016991 -- Q2 table -- read: USGS Publications Warehouse. [unresolved]
- USGS 2026 (Li pegmatites, n = 73; median 21.2 Mt, 1.12% Li2O) -- https://pubs.usgs.gov/publication/70276446 -- Q2 table -- read: USGS Publications Warehouse. [unresolved]
- Kesler and Wilkinson 2013 (granitic tin, n = 301; about 14,520 deposits remain) -- https://doi.org/10.1144/sp393.3 -- Q2 and Q5 -- read: abstract only. [unresolved]
- USGS SIR 2010-5070-Q, -D, -K, -N; Hitzman et al. 2005 (https://doi.org/10.5382/av100.19); SIR 2010-5090-S and -CC (https://pubs.usgs.gov/publication/sir20105090S ; https://pubs.usgs.gov/publication/sir20105090CC) -- Q2 table rows (epithermal, arc porphyry Mo, Fe-Ti, sedex basins, sediment-hosted Cu supergiant, potash basins with tract criteria >= 1 m, >= 4% K2O, <= 3 km, >900 sites) -- read: USGS Publications Warehouse; SIR 2010-5090-S full text. [unresolved]
- USGS OFR 2003-107 (sediment-hosted Cu; 2 km same-deposit rule) -- https://pubs.usgs.gov/publication/ofr2003107 -- Q2 -- read: USGS Publications Warehouse. [unresolved]
- USGS SIR 2010-5070-G (Co-Cu-Au; 500 m rule) -- https://pubs.usgs.gov/publication/sir20105070G -- Q2 -- read: USGS Publications Warehouse. [unresolved]
- USGS 2021 (area-tonnage relations in 10 deposit types) -- https://pubs.usgs.gov/publication/70229519 -- Q2 -- read: USGS Publications Warehouse. [unresolved]
- Kesler and Wilkinson 2008 -- https://doi.org/10.1130/g24317a.1 -- Q2 and Q5: about 125,895 porphyry Cu deposits formed in the Phanerozoic, 47,789 remain, two-thirds recycled, deposits above 3.3 km within mining reach -- read: abstract only. [= Kesler2008]
- USGS SIR 2018-5160 (undiscovered Cu, 236 tracts, 11 regions) -- https://pubs.usgs.gov/publication/sir20185160 -- Q2 -- read: USGS Publications Warehouse. [unresolved]
- USGS MRDS -- https://mrdata.usgs.gov/mrds -- Q2: public domain, not systematically updated since 2011, record count unverified -- read: fetched; count not verified. [unresolved]
- USMIN -- https://mrdata.usgs.gov/usmin -- Q2: v10.0 in 2023 -- read: fetched. [unresolved]
- USGS OFR 2009-1252 (sediment-hosted Zn-Pb) -- https://pubs.usgs.gov/of/2009/1252/ -- Q2: dataset; Gaps: median "could not be parsed reliably" -- read: partially parsed. [= Singer2009]
- USGS grade-tonnage model pages: sedcu, carbonatite, podchrome, sedau -- https://mrdata.usgs.gov/sedcu/ ; https://mrdata.usgs.gov/carbonatite/ ; https://mrdata.usgs.gov/podchrome/ ; https://mrdata.usgs.gov/sedau/ -- Q2: public domain, "all returned HTTP 200 here" -- read: fetched (status checked). [unresolved]
- USGS-R/MapMark4 -- https://github.com/USGS-R/MapMark4 -- Q2: `License: CC0` in DESCRIPTION -- read: fetched (sub-agent plus coordinator check). [unresolved]
- USGS OFR 2004-1344, EMINERS v3.0 -- https://pubs.usgs.gov/of/2004/1344/ -- Q2: Windows GUI program, Visual C++ 6.0/MFC -- read: fetched (page; conflicting sub-agent reports reconciled). [unresolved]
- Macrostrat -- https://macrostrat.org -- Q2: 33,903 units, 2.5M map polygons, CC BY 4.0 -- read: fetched (sub-agent). [unresolved]
- Geoscience Australia copyright page -- https://www.ga.gov.au/copyright -- Q2: default CC BY 4.0; OZMIN URL and status unverified -- read: fetched; OZMIN unverified. [unresolved]
- ScienceBase: USGS 2017 critical-minerals global dataset https://www.sciencebase.gov/catalog/item/594d3c8ee4b062508e39b332 ; CMMI https://www.sciencebase.gov/catalog/item/686ecb93d4be020e5c0a6c07 -- Q2 -- read: not stated. [unresolved]
- Mudd and Jowitt 2018, "Growing Global Copper Resources, Reserves and Production", Econ. Geol. 113(6):1235-1267 -- https://doi.org/10.5382/econgeo.2018.4590 -- Q2: global Cu dataset ("checked on Crossref; it replaces an earlier 2013 citation") -- read: metadata checked on Crossref. [unresolved]
- Mudd and Jowitt, global Ni data, Econ. Geol. 2014 -- https://doi.org/10.2113/econgeo.109.7.1813 -- Q2 -- read: not stated. [unresolved]
- GTK, BGS, GEOROC and OZMIN licences -- no links -- Q2 and Gaps: "unverified" -- read: not accessible / unverified. [unresolved]

### Q3. Mineral-systems approach and prospectivity

- Skirrow et al. 2019 -- https://doi.org/10.1016/j.oregeorev.2019.103011 -- Q3: four-component mineral-system definition, GA continent-scale IOCG prospectivity, Monte Carlo of weights -- read: abstract only. [= Skirrow2019]
- McCuaig, Beresford and Hronsky 2010 -- https://doi.org/10.1016/j.oregeorev.2010.05.008 -- Q3: critical processes to mappable criteria, four steps -- read: abstract only. [= McCuaig2010]
- Hronsky and Groves 2008 -- https://doi.org/10.1080/08120090701581356 -- Q3 -- read: abstract only (see Q2). [= Hronsky2008]
- Knox-Robinson and Wyborn 1997 -- https://doi.org/10.1080/08120099708728326 -- Q3: empirical vs conceptual prospectivity -- read: abstract only. [unresolved]
- Bonham-Carter, Agterberg and Wright 1990 -- https://doi.org/10.4095/128059 -- Q3: weights-of-evidence -- read: abstract only. [unresolved]
- Hagemann, Lisitsin and Huston 2016 (title-level) -- https://doi.org/10.1016/j.oregeorev.2015.12.012 -- Q3 -- read: title-level only. [unresolved]
- Huston et al. 2016, "Tectono-metallogenic systems" (title-level) -- https://doi.org/10.1016/j.oregeorev.2015.09.005 -- Q3 -- read: title-level only. [unresolved]
- EIS Toolkit (GispoCoding) -- https://github.com/GispoCoding/eis_toolkit -- Q3: Python prospectivity library, EUPL-1.2, active 2025 -- read: fetched (licence via GitHub API; sub-agent). [unresolved]
- EIS QGIS plugin -- https://github.com/GispoCoding/eis_qgis_plugin -- Q3: v1.0.6, no LICENSE file, README says "Licensed under GPL 2 or later" -- read: fetched (README; two conflicting sub-agent checks reconciled). [unresolved]
- ArcSDM (gtkfi) -- https://github.com/gtkfi/ArcSDM -- Q3: needs ArcGIS Pro, no licence file, active 2026 -- read: fetched; Gaps: "ArcSDM method list ... not verified". [unresolved]
- Butterworth et al. 2016 -- https://doi.org/10.1002/2016tc004289 -- Q3: random forest/SVM on South American subduction, ~100 km/Myr, ~15 degrees, slab age ~25-70 Myr, >2,000 km from trench -- read: abstract only. [= Butterworth2016]
- Diaz-Rodriguez, Muller and Chandra 2021 (title-level) -- https://doi.org/10.1016/j.oregeorev.2021.104300 -- Q3 -- read: title-level only; Gaps: method details not verified. [unresolved]
- Richards 2003 -- https://doi.org/10.2113/gsecongeo.98.8.1515 -- Q3: porphyry magmas hot, hydrous, oxidized, S-rich -- read: abstract only. [= Richards2003]
- Weis, Driesner and Heinrich 2012 -- https://doi.org/10.1126/science.1225009 -- Q3: porphyry ore shell stabilizes at the lithostatic/hydrostatic boundary -- read: abstract only. [unresolved]
- Ingebritsen and Appold 2012 -- https://doi.org/10.2113/econgeo.107.4.559 -- Q3 and Q4: coupled thermal-hydraulic-mechanical-chemical modelling still challenges computing -- read: abstract only. [unresolved]
- Re-cited in Q3 (see Q1/Q2 entries): Cooke et al. 2005, Lehmann 2020, Hoggard et al. 2020, Groves et al. 2010, Griffin et al. 2013 (title-level), Garven 1985, Bethke 1986, Appold and Garven 1999, Cathles and Adams 2005, S.F. Cox 2005, Barnes and Lightfoot 2005, Hitzman et al. 2010, Kesler and Wilkinson 2006 and 2008. [unresolved]
- Wyborn, Heinrich and Jaques 1994 -- no link -- Q3 Gaps: "could not be retrieved" -- read: not accessible. [unresolved]
- Whole-planet ore forward model / games survey -- Q3 Gaps: none found. [unresolved]

### Q4. Thermodynamic and petrologic tools

Q4 note: "Licence and status details come from sub-agent checks; runtime per point was unverified for every tool."
- Perple_X (Connolly, ETH) -- https://www.perplex.ethz.ch -- Q4: Fortran CLI; "No formal licence found; free download with source" -- read: sub-agent check. [unresolved]
- THERMOCALC (Powell and Holland) -- https://hpxeosandthermocalc.org -- Q4: "Freely available for academic research" -- read: sub-agent check (site wording). [unresolved]
- MELTS family: melts.ofm-research.org https://melts.ofm-research.org ; alphaMELTS https://github.com/magmasource/alphaMELTS -- Q4: alphaMELTS 2 AGPL-3.0 (LICENSE file checked); PetThermoTools mirror shows NOASSERTION -- read: sub-agent check. [unresolved]
- ENKI ThermoEngine -- no link -- Q4: licence "Unverified (no canonical licensed repo found)" -- read: not accessible / unverified. [unresolved]
- BurnMan -- https://github.com/geodynamics/burnman -- Q4: GPL-2.0 -- read: sub-agent check. [unresolved]
- MAGEMin (Riel et al. 2022) -- https://github.com/ComputationalThermodynamics/MAGEMin -- Q4: GPL-3.0, C core, Julia wrapper -- read: sub-agent check. [unresolved]
- Theriak-Domino -- https://titan.minpet.unibas.ch/minpet/theriak -- Q4: "Free for non-commercial use (site wording)" -- read: sub-agent check. [unresolved]
- PHREEQC (USGS) -- https://www.usgs.gov/software/phreeqc-version-3 -- Q4: US public domain, IPhreeqc library -- read: sub-agent check. [unresolved]
- Reaktoro -- https://github.com/reaktoro/reaktoro -- Q4: LGPL-2.1 -- read: sub-agent check. [unresolved]
- SUPCRTBL (Zimmer et al. 2016) -- https://doi.org/10.1016/j.cageo.2016.02.013 ; https://github.com/HydrogeoIU/SUPCRTBL -- Q4: no LICENSE file so all rights reserved ("coordinator check; raw LICENSE probes on main/master also returned 404 here") -- read: fetched (GitHub probe). [unresolved]
- PyBasin (Luijendijk) -- https://github.com/ElcoLuijendijk/pybasin -- Q4: 1D burial/thermal history, LGPL-3.0 -- read: sub-agent check. [unresolved]
- Waples 1980 (Lopatin TTI) -- https://doi.org/10.1306/2f9193d2-16ce-11d7-8645000102c1865d -- Q4: TTI = sum of dt_n * 2^n, oil generation starts near TTI 15, peaks near 75 -- read: abstract only. [unresolved]
- Sweeney and Burnham 1990 (Easy%Ro) -- https://doi.org/10.1306/0c9b251f-1710-11d7-8645000102c1865d -- Q4: parallel first-order reactions, distributed activation energies -- read: abstract only ("the parameter table was not re-extracted"). [unresolved]
- SLB glossary catagenesis -- https://glossary.slb.com/en/terms/c/catagenesis -- Q4 -- read: fetched. [unresolved]
- Trinity (ZetaWare) -- no link -- Q4: proprietary -- read: sub-agent only. [unresolved]
- BasinMod (Platte River Associates) -- https://www.platte.com/ -- Q4: proprietary, "corrects a sub-agent note that attributed it to Ikon Science/SLB" -- read: fetched (product site). [unresolved]
- PetroMod (SLB) -- no URL -- Q4: "coordinator-resolved; no URL retrieved here" -- read: not stated. [unresolved]
- GEMS -- no link -- Q4 Gaps: licence terms unverified -- read: unverified. [unresolved]

### Q5. Preservation and exposure

- Kesler and Wilkinson 2006 (https://doi.org/10.2113/gsecongeo.101.5.919) -- Q5: exhumation rates 167 m/Myr epithermal, 158 m/Myr porphyry Cu, 63 m/Myr orogenic gold -- read: abstract only. [= Kesler2006]
- Kesler and Wilkinson 2008 (https://doi.org/10.1130/g24317a.1) and 2013 (https://doi.org/10.1144/sp393.3) -- Q5: tectonic diffusion; tin average formation depth ~2 km -- read: abstract only. [= Kesler2008]
- Yanites and Kesler 2015 (title-level) -- https://doi.org/10.1038/ngeo2429 -- Q5: climate imprint on porphyry exhumation -- read: title-level only. [unresolved]
- Q5 re-cites (see Q1 entries): Sillitoe 2010, B1693 m17, m38a, m38b, m38c, m39a, m39d, m29a, Seedorff et al. 2005, SIR 2010-5070-Q, Sillitoe 2005, Cooke et al. 2005, SIR 2010-5070-H, OFR 2011-1058, Butt and Cluzel 2013, Garnett and Bassett 2005, Lehmann 2020, SIR 2010-5070-L, Groves et al. 2005 and 2010, McCauley and Bradley 2014, Cuney 2010, Leach et al. 2010. [unresolved]
- Hartley and Rice 2005 (title-level) -- https://doi.org/10.1007/s00126-005-0017-7 -- Q5 and Gaps: Andean supergene climate controls -- read: title-level only. [unresolved]
- Q5 Gaps: "No quantitative rules were found for placer concentration efficiency per unit of eroded source." [unresolved]

### Q6. How games and Minecraft mods model ore geology

Q6 note: the mod-schema table is "(sub-agent, verified from source files at the branches noted)"; "Other games (sub-agent; flags carried over)"; for player sentiment "Reddit was blocked from this environment and WebSearch was exhausted, so evidence leans on GitHub issues, PRs, wikis and changelogs, and likes are thin".
- TerraFirmaCraft 1.21.x raw sources: VeinConfig.java https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/world/feature/vein/VeinConfig.java ; Indicator.java https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/world/feature/vein/Indicator.java ; RockLayerSettings.java https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/world/settings/RockLayerSettings.java ; PropickItem.java https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/common/items/PropickItem.java ; Ore.java https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/common/blocks/rock/Ore.java ; normal_native_gold.json https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/resources/data/tfc/worldgen/configured_feature/vein/normal_native_gold.json ; world_gen.py https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/resources/world_gen.py -- Q6: vein configured features, indicators, rock layers, prospector's pick (radius 12, false-negative 0.30 at tier 0 to 0 at tier 5), graded ore blocks -- read: source files read (sub-agent). Unverified: `kaolin_disc_vein` fields. [unresolved]
- GregTech CEu Modern 1.21 branch: GTOreDefinition.java https://raw.githubusercontent.com/GregTechCEu/GregTech-Modern/1.21/src/main/java/com/gregtechceu/gtceu/api/data/worldgen/GTOreDefinition.java ; ConfigHolder.java https://raw.githubusercontent.com/GregTechCEu/GregTech-Modern/1.21/src/main/java/com/gregtechceu/gtceu/config/ConfigHolder.java ; BedrockOreDefinition.java https://raw.githubusercontent.com/GregTechCEu/GregTech-Modern/1.21/src/main/java/com/gregtechceu/gtceu/api/data/worldgen/bedrockore/BedrockOreDefinition.java -- Q6 -- read: source files read (sub-agent). Unverified: `GTLayerPattern` fields (KubeJS example "composed from builder methods, not copied from the repo"). [unresolved]
- Geolosys 1.21.1 branch: DepositFeature.java https://raw.githubusercontent.com/oitsjustjose/Geolosys/1.21.1/src/main/java/com/oitsjustjose/geolosys/common/world/feature/DepositFeature.java ; kimberlite.json https://raw.githubusercontent.com/oitsjustjose/Geolosys/1.21.1/src/main/resources/data/geolosys/deposits/ore/kimberlite.json ; repo https://github.com/oitsjustjose/Geolosys (archived 2026-01-19) -- Q6 -- read: source files read (sub-agent). Unverified: a `deposit_layer` example. [unresolved]
- Immersive Engineering 1.21.1: MineralMixSerializer.java https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/main/java/blusunrize/immersiveengineering/common/crafting/serializers/MineralMixSerializer.java ; ExcavatorHandler.java https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/api/java/blusunrize/immersiveengineering/api/excavator/ExcavatorHandler.java ; PR #4052 https://github.com/BluSunrize/ImmersiveEngineering/pull/4052 ; galena.json https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/generated/resources/data/immersiveengineering/recipe/mineral/galena.json (Q7) -- Q6 and Q7: mineral_mix recipe, vein centre algorithm -- read: source files read (sub-agent). Unverified: `mineralVeinYield` config key. [unresolved]
- Immersive Petroleum (TwistedGate fork): ReservoirSerializer.java https://raw.githubusercontent.com/TwistedGate/ImmersivePetroleum/1.21.1/src/main/java/flaxbeard/immersivepetroleum/common/crafting/serializers/ReservoirSerializer.java ; ReservoirPolygon.java https://raw.githubusercontent.com/TwistedGate/ImmersivePetroleum/1.21.1/src/main/java/flaxbeard/immersivepetroleum/api/reservoir/ReservoirPolygon.java ; Modrinth API https://api.modrinth.com/v2/project/immersivepetroleum -- Q6: reservoir fields, Perlin threshold polygons -- read: source files read (sub-agent). [unresolved]
- Create Ore Excavation 1.6.8 (MIC pack jar createoreexcavation-1.21-1.6.8.jar; README https://github.com/tom5454/Create-Ore-Excavation ; https://github.com/tom5454/Create-Ore-Excavation#kubejs) -- Q6: vein recipe schema (spacing/separation/salt, amount multipliers), drilling/extracting recipes, config defaults `finiteAmountBase` = 1000, `defaultInfinite` = true -- read: "read directly from the jar" (config defaults "read from bytecode"). Gap: "Whether COE exposes a KubeJS or Java API for per-chunk vein assignment ... was not checked." [unresolved]
- NeoForge Tags.java (1.21.1) -- https://raw.githubusercontent.com/neoforged/NeoForge/1.21.1/src/main/java/net/neoforged/neoforge/common/Tags.java ; NeoForge tag docs https://docs.neoforged.net/docs/resources/server/tags/ -- Q6, Q7: c:ores tags, host-ground tags, ore_rates, raw materials -- read: source read; docs page "cover only the c: convention". [unresolved]
- Excavated Variants README -- https://raw.githubusercontent.com/lukebemishprojects/ExcavatedVariants/1.21.x/README.md -- Q6: ground types, ores, stones, modifiers, runtime cross-join -- read: fetched; MC patch pin unverified. [unresolved]
- Unearthed (cainthetrain) -- https://api.modrinth.com/v2/project/unearthed-cainthetrain ; port https://github.com/Chunchunmaru3D/UnEarthed-1.21.1-neoforge-port -- Q6: first-party NeoForge only for 1.21.4 and 1.21.8 -- read: fetched (API); source repo unverified. [unresolved]
- Dwarf Fortress wiki: inorganic tokens https://dwarffortresswiki.org/index.php/Inorganic_material_definition_token ; Vein https://dwarffortresswiki.org/index.php/Vein ; hematite raw https://dwarffortresswiki.org/index.php/DF2014:Hematite/raw ; cassiterite raw https://dwarffortresswiki.org/index.php/Cassiterite/raw ; Layer https://dwarffortresswiki.org/index.php/Layer ; Biome https://dwarffortresswiki.org/index.php/Biome ; Embark https://dwarffortresswiki.org/index.php/DF2014:Embark -- Q6 -- read: sub-agent; "SEDIMENTARY_OCEAN_* classes are unverified". [unresolved]
- Vintage Story wiki: Simple Worldgen https://wiki.vintagestory.at/Modding:Content_Tutorial_Simple_Worldgen ; Ore Deposits https://wiki.vintagestory.at/Ore_Deposits/en ; Terrain Generation https://wiki.vintagestory.at/Terrain_Generation ; Guide:Prospecting https://wiki.vintagestory.at/Guide:Prospecting ; Prospecting pick https://wiki.vintagestory.at/index.php/Prospecting_pick -- Q6 -- read: sub-agent. [unresolved]
- Factorio: Lua API AutoplaceSpecification https://lua-api.factorio.com/latest/types/AutoplaceSpecification.html ; Map generator https://wiki.factorio.com/Map_generator ; forum https://forums.factorio.com/viewtopic.php?t=40210 ; FFF-390 https://factorio.com/blog/post/fff-390 -- Q6 (richness vs distance "per community sources") -- read: sub-agent. [unresolved]
- Satisfactory wiki: Resource Node https://satisfactory.wiki.gg/wiki/Resource_Node ; Resource Well https://satisfactory.wiki.gg/wiki/Resource_Well -- Q6 -- read: sub-agent. [unresolved]
- Minecraft Wiki: Ore (feature) https://minecraft.wiki/w/Ore_(feature) ; Placed feature https://minecraft.wiki/w/Placed_feature ; Ore vein https://minecraft.wiki/w/Ore_vein ; Noise router https://minecraft.wiki/w/Noise_router ; Java Edition 1.18 https://minecraft.wiki/w/Java_Edition_1.18 -- Q6: ore feature config, large ore veins (copper/granite Y 0-50, iron/tuff Y -60 to -8, 2% raw-ore-block chance), tuning across snapshots -- read: sub-agent. [unresolved]
- RimWorld wiki: Deep drill https://rimworldwiki.com/wiki/Deep_drill ; Ground-penetrating scanner https://rimworldwiki.com/wiki/Ground-penetrating_scanner ; ONI Geyser https://oxygennotincluded.wiki.gg/wiki/Geyser -- Q6 -- read: sub-agent. [unresolved]
- ProcGenesis -- https://procgenesis.com/ -- Q6: advertises tectonics, erosion and ore views, documents no algorithm -- read: sub-agent. [unresolved]
- Gleba (calandiel) -- https://calandiel.itch.io/gleba ; World Orogen -- https://www.orogen.studio/ -- Q6: rock and tectonics without ores -- read: sub-agent. [unresolved]
- Azgaar Fantasy Map Generator discussion #1091 -- https://github.com/Azgaar/Fantasy-Map-Generator/discussions/1091 -- Q6: no geology or ore model -- read: sub-agent. [unresolved]
- Player-sentiment issues and PRs: IE #2229 https://github.com/BluSunrize/ImmersiveEngineering/issues/2229 ; IP #181 https://github.com/Flaxbeard/ImmersivePetroleum/issues/181 ; COE #85 https://github.com/tom5454/Create-Ore-Excavation/issues/85 ; IP #96 https://github.com/Flaxbeard/ImmersivePetroleum/issues/96 ; GeolosysHax https://github.com/TheAirBlow/GeolosysHax ; GTCEu PR #2642 https://github.com/GregTechCEu/GregTech-Modern/pull/2642 ; GTCEu CHANGELOG https://github.com/GregTechCEu/GregTech-Modern/blob/1.20.1/CHANGELOG.md ; GT PR #1194 https://github.com/GregTechCEu/GregTech/pull/1194 ; create-ore-excavation-deposits https://github.com/RandomCodeGuys/create-ore-excavation-deposits ; TFC Field Guide Precision Prospecting https://terrafirmacraft.github.io/Field-Guide/en_us/precisionprospecting/index.html ; Geolosys #82 https://github.com/oitsjustjose/Geolosys/issues/82 ; Geolosys #158 https://github.com/oitsjustjose/Geolosys/issues/158 ; GTCEu #3114 https://github.com/GregTechCEu/GregTech-Modern/issues/3114 ; GT PR #191 https://github.com/GregTechCEu/GregTech/pull/191 -- Q6 "What players liked and disliked": complaints, infinite-vein requests, tooling, performance traps, design direction -- read: sub-agent (GitHub issues/PRs/changelogs). Gaps: "Direct player sentiment from Reddit, CurseForge comments and the Bay 12 and TFC forums was unreachable ... TFC and GT 'likes' are essentially missing." [unresolved]

### Q7. Proposed data-driven deposit-type schema

Q7 re-cites sources already listed under Q1, Q2 and Q6; the Q7 "Cited Findings" cite them as follows. Examples A (porphyry Cu-Mo-Au) and B (fantasy mythril) in Q7 are JSON proposals: the document says "The cited values are the settings, depth window, P90/P50/P10 figures, correlation and 2 km separation (Q1/Q2). The Au P50, curve shapes, camp spacing for porphyries and all multipliers are illustrative", and for Example B "The numbers are illustrative."
- NeoForge Tags.java (1.21.1) -- https://raw.githubusercontent.com/neoforged/NeoForge/1.21.1/src/main/java/net/neoforged/neoforge/common/Tags.java -- Q7: c:ores, c:ores_in_ground, c:ore_bearing_ground, c:ore_rates, c:raw_materials conventions -- read: source read. [unresolved]
- IE galena.json and MineralMixSerializer.java -- https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/generated/resources/data/immersiveengineering/recipe/mineral/galena.json ; https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/main/java/blusunrize/immersiveengineering/common/crafting/serializers/MineralMixSerializer.java -- Q7: other mods consume `c:ores/lead` -- read: source files read. [unresolved]
- TFC Ore.java; Excavated Variants README; Vintage Story Simple Worldgen; DF inorganic tokens; VS Ore Deposits; Factorio Map generator; Satisfactory Resource Node; Minecraft Wiki Ore (feature); COE repo and local jar; TFC PropickItem.java; VS Guide:Prospecting; IE ExcavatorHandler.java; IP ReservoirPolygon.java -- Q7 "balance levers in the wild" and "one ore block per host stone" -- read: as in Q6. [unresolved]
- IE #2229, IP #96, GT PR #1194, Geolosys #158 -- Q7: player-demand signals -- read: as in Q6. [unresolved]
- USGS OFR 2008-1155 and Singer et al. 2005 -- Q7: real-type sizing inputs (33/240/1,500 Mt; Cu 0.24/0.44/0.80%; P50 >= 15 per 100,000 km2) -- read: as in Q2. [unresolved]
- Skirrow et al. (four components), Hronsky and Groves (multiplicative ranking), Cuney, Leach, Klemme and Ulmishek (clocks), Kesler and Wilkinson (preservation), Sillitoe 2005 (supergene), B1693 (weathering and geochemical signature fields), Robert (camp spacing) -- Q7 "Precedent" column of the field-group table -- read: as in Q1-Q3. [unresolved]
- Q7 Gaps: no cross-mod deposit-type standard exists; "The schema is a proposal and has not been tested against real sim output"; NeoForge host-ground tags only for stone, deepslate, netherrack; Excavated Variants' 1.21.1 compatibility and any API for programmatic registration "were not verified"; "It is unknown whether COE exposes a per-chunk vein-assignment API." [unresolved]

## 5. docs/research/notes/geologist_youtuber_feedback.md

Reading-method note for this document (its own "Method"): "The channel was found by web search. English captions were pulled with `yt-dlp --skip-download --write-auto-subs --write-subs` (no video or audio downloaded). Fifteen videos were read in full. Most of his own captions are human-quality; two older ones (Sediment 2022, Iron Ore 2024) are visibly machine-translated and garble words." "Where he gives a number it is his, from the captions, not independently checked." The document uses short codes for the videos (VS1, VS2, HT, Mountains, Iron, Ore, Caves, Igneous, Sediment, Sulfur, Maps, Ruri, Water, Fixing, Interview) with caption timestamps. All sources here are YouTube videos by the geologist "Gneiss Name" (channel @gneissname) unless noted. The document says the claims are the speaker's ("His:") and that the Deep Time mapping is the author's inference; none of the speaker's numbers were independently verified.

### Channel and videos (section 1)

- Gneiss Name channel (@gneissname) -- https://www.youtube.com/@gneissname ; site gneiss.name -- sec 1: identity and credentials of the speaker ("Confirmed with Ben"); "over 256 thousand subscribers" in a Feb 2026 video -- read: channel list viewed ("about 80 uploads, not exhaustive"). [unresolved]
- "Geologic Review: Vintage Story, Part 1 (rocks, sediment, soils, world gen)" (VS1), 2025-12-15 -- https://www.youtube.com/watch?v=b4e7mTRB7qg -- sec 3.1, 3.2, 3.6, 3.9, 4, 6: rock supply, province-based strata critique (layers drape instead of filling lows), soils (CORPT), mantle, ophiolite -- read: captions read in full. [unresolved]
- "Geologic Review: Vintage Story, Part 2 (ores, structures, caves, prospecting)" (VS2), 2026-03-25 -- https://www.youtube.com/watch?v=2D8y68Zf7Uo -- sec 3.3, 3.4, 3.10, 4, 6: ore host-rock complaints, salt domes, kimberlite, craters, prospecting modes, cave-ins, thermal pools -- read: captions read in full. [unresolved]
- "Geologic Review: Hytale, Early Access" (HT), 2026-01-26 -- https://www.youtube.com/watch?v=qnnEknpbAB0 -- sec 3.1, 3.2, 3.5, 3.6, 3.7, 3.9: zone strata, cliffs, glaciers, rivers, volcano zone -- read: captions read in full. [unresolved]
- "Real mountains in Minecraft" (Mountains), 2025-08-20 -- https://www.youtube.com/watch?v=PCZyPyAwQ5E -- sec 3.5, 3.7, 3.8: mountain types, scale (255 m, 1:10,000), slope, volcanoes, 1:33,000 map -- read: captions read in full. [unresolved]
- "Geology of Minecraft: Iron Ore, Facts and Fiction" (Iron), 2024-12-12 -- https://www.youtube.com/watch?v=OfHLsqaHJoo -- sec 3.1, 3.3: iron generation groups (62 percent above y 70), tuff "veins", banded iron formations (1.5-3.8 Ga) -- read: captions read in full; "visibly machine-translated" (garbles "tuff" as "tough", "Archean" as "Archaic"). [unresolved]
- "How does Minecraft ore compare to Earth" (Ore), 2024-01-01 -- https://www.youtube.com/watch?v=Id0yCfaoDsw -- sec 3.3: ore definition, 1.18 depth histograms, 128x128 excavation vs annual production (about 2x Earth's annual gold, diamonds about 75 years of production) -- read: captions read in full. [unresolved]
- "Geology of Minecraft: Caves!" (Caves), 2023-04-25 -- https://www.youtube.com/watch?v=1tQumBj-lSE -- sec 3.4: speleothems, karst, water table, cave types, dripstone growth (about 73 m per Minecraft year vs a 40 mm per year record) -- read: captions read in full. [unresolved]
- "Geology of Minecraft: Igneous Rocks" (Igneous), 2022-07-30 -- https://www.youtube.com/watch?v=H5AVWM-T054 -- sec 3.1, 3.8: igneous blobs, tuff misplacement, cooling and crystal size -- read: captions read in full. [unresolved]
- "Geology of Minecraft: Episode 5, Sediment" (Sediment), 2022-02-05 -- https://www.youtube.com/watch?v=0MwsU3CeRaM -- sec 3.2, 3.6: grain-size sorting by transport -- read: captions read in full; "visibly machine-translated"; "the content is standard sedimentology". [unresolved]
- "Sulfur Caves, A Quick Look (1.21.x snapshot)" (Sulfur), 2026-04-10 -- https://www.youtube.com/watch?v=Col0ApUB68E -- sec 3.4: real sulfur caves are sulfuric-acid dissolution caves with gypsum -- read: captions read in full. [unresolved]
- "Minecraft Maps Are Strange" (Maps), 2025-10-01 -- https://www.youtube.com/watch?v=4dEx8D1mjso -- sec 3.10: hillshade (north-west light), topo lines -- read: captions read in full. [unresolved]
- "Geologic Review: Ruri Rocks S1 (anime)" (Ruri), 2026-06-25 -- https://www.youtube.com/watch?v=qjtzBLA24aI -- sec 3.10: no geologic maps shown -- read: captions read in full. [unresolved]
- "Minecraft Water is Strange" (Water), 2026-02-23 -- https://www.youtube.com/watch?v=em2Y4Rm2yiM -- sec 1: rendering internals only, "read, nothing used" -- read: captions read in full. [unresolved]
- "Fixing Minecraft with Real Geology, ft. Gneiss Name" (Fixing), Amos Redstone channel, 2024-11-22 -- https://www.youtube.com/watch?v=AwkgAM9c7M8 -- sec 3.1, 3.6: first third is Gneiss Name (sand, gravel, soil, weathering); later proposals are Amos's -- read: captions read in full; only the first third attributed to him. [unresolved]
- "Talking Geology, Science Communication, and Minecraft with @gneissname" (Interview), Quantum Mic channel, 2026-01-02 -- https://www.youtube.com/watch?v=pRxXRb1kRGI -- sec 1, 3.5, 3.8: background; scale of geologic time as most misunderstood concept; also describes the Coal and Diamonds video -- read: captions read in full. [unresolved]
- "Geology of Minecraft: Coal and Diamonds" -- https://www.youtube.com/watch?v=JivStfUWOwo -- sec 1 "Not retrieved": content known only from the interview (compares how long Earth took to make coal to how fast it is burned; about half of Earth's total coal already used) -- read: not accessible ("YouTube answered the subtitle request with HTTP 429 on three attempts ... Re-run later"). [unresolved]
- Short or older episodes (Geologic Materials 1.1, Stone, Tuff Talk, "Why care about Geology") and Minecraft Education / color-theory videos -- no URLs -- sec 1: "Also not read" -- read: not read. [unresolved]
- "The Minecraft Historian" / "The Geologic History of Minecraft" (a different channel; student cross-section interpretation that mentions Gneiss Name in passing) -- no URL -- sec 1 and 3.4: one hearsay line (that the student says Gneiss Name avoids a "geologic history of Minecraft" because of sealed dirt pockets) -- read: secondary source (turned up in a web search; "not him and is not used here" except the hearsay line). [unresolved]
- Gneiss Name's downloadable "Geology of Minecraft" world and Geologic Timeline Project (1 block = 1 Myr, 4,567 blocks) -- no URL -- sec 1 -- read: described by the speaker (secondary). [unresolved]
- yt-dlp (software; `nix run nixpkgs#yt-dlp -- --skip-download --write-auto-subs --sub-langs en <url>`) -- no URL -- sec 1 method: caption extraction tool -- read: used. [unresolved]
- Real-world facts quoted by the speaker and passed through unchecked (document says they are "his, from the captions, not independently checked"): Rib Mountain, Wisconsin (226 m, 2 Ga quartzite); Tetons fault-block range; Everest summit marine limestone; BIF (1.5-3.8 Ga, peak 2.5 Ga); underground temperature +25 degC per km after 15 m; Kola-style comparisons; USGS-style provinces in Vintage Story (shield, platform, orogenic, basin, large igneous, extended crust). [unresolved]

## 6. docs/research/notes/run-review-1a0f969d358.md

Reading-method note for this document: it states under "Sources" that the listed items were "Sourced by web search for this review", and separately "From memory, not checked: Earth's continental crust at about 40 % of its area; land-height shares above 2 and 4 km; mean land height; mountain erosion rates; the sea-ice edge's wander; Hawaii-Emperor length and age; the long-term sea-level range; the Caspian's area; the count of Earth plates above 5 %." It does not say whether each web page was fetched in full or only seen in search results, so the read status below is "web search (fetch not stated)". The run's own measurements (frames, events.jsonl, status lines) are internal data, not literature, and are not listed here. Section numbers are the document's (sec 1 plate kinematics, 2 supercontinent cycle, 3 continents, 4 oceans, 5 mountains, 6 volcanism, 7 climate and ice, 8 rivers and lakes).

- Pacific plate (Wikipedia) -- https://en.wikipedia.org/wiki/Pacific_plate -- Sources; sec 1: Pacific plate 103 million km2, about 20 % of Earth -- read: web search (fetch not stated). [unresolved]
- arXiv 1011.2752 -- https://arxiv.org/pdf/1011.2752 -- Sources; sec 1: seven major plates -- read: web search (fetch not stated). Title and authors not given. [unresolved]
- Muller et al. 2008, sea-floor age -- https://agupubs.onlinelibrary.wiley.com/doi/10.1029/2007gc001743 -- Sources; sec 4: mean sea-floor age 64.2 Myr, oldest large areas 180-200 Myr -- read: web search (fetch not stated). [unresolved]
- Herodotus Basin about 340 Myr (Nature Geoscience) -- https://www.nature.com/articles/ngeo2784 -- Sources; sec 4: one small basin about 340 Myr -- read: web search (fetch not stated). Authors and title not given. [unresolved]
- Zahirovic et al. 2015, tectonic speed limits in plate reconstructions -- https://earthbyte.org/Resources/Pdf/Zahirovic_2015_Tectonic_speed_limits_plate_reconstructions.pdf -- Sources; sec 1: median RMS plate speed about 40 mm/yr over 200 Myr, fastest plates about 85, limit about 200 -- read: web search (fetch not stated). [= Zahirovic2015]
- Supercontinent cycle (Wikipedia) -- https://en.wikipedia.org/wiki/Supercontinent_cycle -- Sources; sec 2: cycle length (quoted "from about 400 to 800 Myr"), Rodinia and Pangaea dates -- read: web search (fetch not stated). [unresolved]
- Nature Reviews Earth & Environment article (s43017-021-00160-0) -- https://www.nature.com/articles/s43017-021-00160-0 -- Sources; sec 2: supercontinent cycle length and dates -- read: web search (fetch not stated). Title and authors not given. [= Mitchell2021]
- Peel et al. 2007, Koppen-Geiger climate zones by land area (HESS 11:1633) -- https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf -- Sources; sec 7 table: Earth tropical 19 %, arid 30 %, temperate 13 %, cold 25 %, polar 13 % of land -- read: web search (fetch not stated). [= Peel2007]
- Sea ice (Wikipedia) -- https://en.wikipedia.org/wiki/Sea_ice -- Sources; sec 7: sea ice about 7 % of Earth's surface and 12 % of the oceans -- read: web search (fetch not stated). [unresolved]
- Verpoorter et al. 2014, global lakes (GRL) -- https://agupubs.onlinelibrary.wiley.com/doi/10.1002/2014GL060641 -- Sources; sec 8: lakes 3.7 % of non-glaciated land -- read: web search (fetch not stated). [= Verpoorter2014]
- Ernst and Youbi 2017 (file name "Ernst and Youbi 2017 PPP"), flood basalts -- http://www.largeigneousprovinces.org/sites/default/files2/Ernst%20and%20Youbi%202017%20PPP.PDF -- Sources; sec 6: one flood basalt per 20-30 Myr -- read: web search (fetch not stated). [unresolved]
- Greenhouse and icehouse Earth (Wikipedia) -- https://en.wikipedia.org/wiki/Greenhouse_and_icehouse_Earth -- Sources; sec 7: greenhouse for about 80 % of the last 540 Myr -- read: web search (fetch not stated). [unresolved]
- In-repo references (not scientific sources): docs/deeptime-implementation-plan.md section 8, STATUS.md (P1-27 entry), docs/design/p1-26-per-cell-climate.md, crates/deeptime-core/src/plates/normalise.rs, crates/deeptime-core/presets/earthlike_quick_16k.json. [unresolved]
- Earth figures used in the text with no source named in the Sources list (see the Earth figures DOC): East Pacific Rise about 150 mm/yr; India-Eurasia 50 mm/yr now and up to 190 at 55 Ma; Earth land fraction 29 %; Atlantic opening for about 180 Myr and oceans open 100-200 Myr or more; Himalaya about 2,400 km long; Everest 8.8 km; Deccan/Siberian Traps size; land ice about 10 % of land; present global mean temperature 14-15 C; Appalachians and Urals about 300 Myr old. [unresolved]

## 7. docs/research/notes/grid-artifact-audit.md

Reading-method note for this document: it is a measurement audit of the project's own bake, not a literature review, so it cites almost no outside sources. It keeps three kinds of statement apart: "measured", "seen" (pictures), "inferred" (reading of the code); and its "Not verified" section says "Earth figures for river sinuosity are from memory." Only techniques and named methods are cited; none has a URL, a year or a reading note in the document.

- Fairfield and Leymarie, "rho-8" stochastic flow routing -- no link, year or title -- Option B.1 (rivers and valleys along eight directions): proposed fix, "choose each cell's receiver between the two neighbours that bracket the true downslope direction, by a hash of the cell in proportion to the angle (Fairfield and Leymarie's rho-8)" -- read: not stated. [unresolved]
- D8 single-direction flow routing -- no citation -- Findings 1, 2, 8: cause of eight-direction rivers, valleys and dams -- read: not stated (named technique). [unresolved]
- Koppen climate classes -- no citation -- Finding 3 and Option C: where the class is decided -- read: not stated (named scheme). [unresolved]
- Catmull-Rom interpolation (clamped), cubic B-spline, value noise with smoothstep fade, 3x3 chamfer distance versus exact Euclidean distance transform, Gaussian smoothing -- no citations -- Findings 6, 7, 8, 9: algorithms whose artefacts were measured -- read: not stated (named techniques). [unresolved]
- Vanilla Minecraft biome blur (4-block cells) -- no citation -- Option C.3 and "Not verified": may soften a 16-block step -- read: not stated; the document says "Nothing was seen in the game". [unresolved]
- Orientation, phase, spectrum and outlier tests (a4 = mean cos 4 theta, a8 = mean cos 8 theta, robust z-score) -- defined in "Method" as the audit's own estimators; no external source -- read: n/a. [unresolved]
- Earth river sinuosity -- no source -- Finding 2: "Real rivers are commonly 1.1-1.5 and over (from memory, not sourced)" -- read: from memory (see the Earth figures DOC). [unresolved]
- Tools and in-repo references (not scientific sources): `tools/review/grid-audit.sh`, `tools/review/grid_audit.py`, `deeptime heightmap --columns`, `surface/spl.rs`, `surface/young.rs`, `bake/biomes.rs`, `amplify/resample.rs`, `deeptime-format/src/detail.rs`, `structure/bake_horizons.rs`. [unresolved]

## 8. docs/design/plate-force-balance.md

Reading-method note for this document: its section 12 "Sources" says "Checked online on 2026-10-02 by a sub-agent; most papers are paywalled, so figures are from abstracts or secondary pages unless a link is given. Items it could not confirm are marked in the text." The text marks sources with "abstract", "from abstracts", "via a secondary source", "not confirmed" and "from memory". The document also marks its own statements "measured", "read" (from the code) or "inferred". Sections cited: 2 (how plates move today), 3.1-3.5 (force model), 4 (open-loop test), 5 (options), 7 (acceptance targets), 8 (the mega-plate), 12 (Sources).

- Forsyth and Uyeda 1975, Geophys. J. R. Astron. Soc. 43:163-200 -- https://academic.oup.com/gji/article/43/1/163/586101 -- sec 3.1, 3.2, 7: plates have no inertia so torques sum to zero; slab pull dominant force, an order of magnitude above the rest; slab resistance "terminal velocity"; boundary (transform) resistance; speed-vs-slab-share and speed-vs-area results -- read: abstract ("Forsyth & Uyeda 1975, abstract", sec 3.2). The attribution of a factor of 8 for continental basal drag to this paper is "not confirmed". [= ForsythUyeda1975]
- Schellart 2004, Geophys. Res. Lett. 31:L07611 -- no link -- sec 3.2: only 8-12 % of a slab's weight reaches the plate, up to twice ridge push -- read: "abstract read in full" (sec 12). [= Schellart2004]
- Carlson, Hilde and Uyeda 1983, Geophys. Res. Lett. 10:297-300 -- no link -- sec 3.2: plate speed follows slab age at the trench -- read: not stated beyond the sec 12 blanket (paywalled, abstracts or secondary pages). [= Carlson1983]
- Conrad and Lithgow-Bertelloni 2002, Science 298:207-209; 2004, J. Geophys. Res. 109:B10407 -- no links -- sec 3.2: slab pull and suction explain subducting plates being about four times faster than others -- read: "from abstracts" (sec 3.2). [= ConradLB2002; = ConradLB2004]
- Conrad and Lithgow-Bertelloni 2006, Geophys. Res. Lett. 33:L05312 -- no link -- sec 3.2: continental roots raise basal traction 2-5 times -- read: abstract (sec 3.2). [= ConradLB2006]
- Lithgow-Bertelloni and Richards 1998, Rev. Geophys. 36:27-78 -- no link -- sec 3.2: ridge push is under 10 % of the driving force -- read: not stated beyond the sec 12 blanket. [= LithgowBertelloniRichards1998]
- Parsons and Richter 1980, Earth Planet. Sci. Lett. 51:445-450 -- no link -- sec 3.2: ridge push about 3 x 10^12 N/m for old plates -- read: not stated beyond the sec 12 blanket. [= ParsonsRichter1980]
- Molnar and Lyon-Caen 1988, GSA Special Paper 218:179-207 -- no link -- sec 3.2: Tibet quoted at 6-7 x 10^12 N/m -- read: not stated beyond the sec 12 blanket. [= MolnarLyonCaen1988]
- Copley, Avouac and Royer 2010, J. Geophys. Res. 115:B03410 -- no link -- sec 3.2 and 7: India slowed from 150 to 40 mm/yr as Tibet thickened -- read: not stated beyond the sec 12 blanket. [= Copley2010]
- Turcotte and Schubert, Geodynamics -- no year or link -- sec 3.2 (gross slab pull), 3.5 (line-load flexure profile, alpha = (4D/(delta rho g))^(1/4)): textbook -- read: "Gross slab pull is from textbook examples and was not confirmed in Turcotte & Schubert itself" (sec 3.2). Flexure formula is attributed to the book without a stated reading. [= TurcotteSchubert_Geodynamics]
- Zahirovic, Muller, Seton and Flament 2015, Earth Planet. Sci. Lett. 418:40-52 -- https://www.earthbyte.org/tectonic-speed-limits-from-plate-kinematic-reconstructions/ -- sec 1, 2, 7: Earth's spread about 85 mm/yr for the fastest group and 26-28 for the slowest; median 40 -- read: link given; reading mode not stated beyond the sec 12 blanket. [= Zahirovic2015]
- van Hunen and Allen 2011, Earth Planet. Sci. Lett. 302:27-37 -- no link -- sec 3.3: slab breakoff 10-25 Myr -- read: not stated beyond the sec 12 blanket. [= vanHunenAllen2011]
- Davis, Suppe and Dahlen 1983, J. Geophys. Res. 88:1153-1172 -- no link -- sec 3.5: source for a fixed critical-wedge taper (`wedge_taper_deg` 1.5 degrees) -- read: **from memory, not checked** (sec 3.5 "(from memory)"; sec 12 "From memory, not checked"). [= DavisSuppeDahlen1983]
- Himalayan foreland basin (Wikipedia) -- https://en.wikipedia.org/wiki/Himalayan_foreland_basin -- sec 3.5: foreland about 450 km wide and 3 to over 7 km deep -- read: web page (fetch not stated). [unresolved]
- DeCelles and Giles 1996, Basin Research 8:105-123 -- no link -- sec 3.5 and sec 12: foredeeps of 100-300 km and 2-8 km, forebulges tens to a few hundred metres -- read: "via a secondary source" (sec 3.5). [= DeCellesGiles1996]
- Coltice, Husson, Faccenna and Arnould 2019, Sci. Adv. 5:eaax4295 -- https://pmc.ncbi.nlm.nih.gov/articles/PMC6821462/ -- sec 12: quoted "Slabs pull plates at rapid rates and tear continents apart, with keels of continents only slowing down their drift when they are not attached to a subducting plate." -- read: fetched (quoted). [= Coltice2019]
- Bercovici and Long 2014, Geophys. Res. Lett. 41:6659-6666 -- no link -- sec 8: slab rollback alone can disperse a supercontinent -- read: not stated beyond the sec 12 blanket. [= BercoviciLong2014]
- Dal Zilio, Faccenda and Capitanio 2018, Tectonophysics 746:312-324 -- no link -- sec 8: rim subduction can supply the tension -- read: not stated beyond the sec 12 blanket. [= DalZilio2018]
- Zhang et al. 2018, Geosci. Front. 9:997-1007 -- no link -- sec 8: upwelling under the supercontinent dominates, old orogens are weak lines -- read: not stated beyond the sec 12 blanket. [= Zhang2018]
- Huang et al. 2019, Geochem. Geophys. Geosyst. 20:4830-4848 -- https://par.nsf.gov/biblio/10127684-modeling-inception-supercontinent-breakup-stress-state-importance-orogens -- sec 8: 20-50 MPa of extension in the interior -- read: link given; reading mode not stated. [= Huang2019]
- Buck 2004, in Karner et al. (eds), Rheology and Deformation of the Lithosphere at Continental Margins, 1-30 -- no link -- sec 8: forces too small to tear thick cold continental lithosphere without weakening -- read: "from abstracts" (sec 8). [= Buck2004]
- Bialas, Buck and Qin 2010, Earth Planet. Sci. Lett. -- no link -- sec 8 and sec 12: same claim -- read: "from abstracts" (sec 8); "volume not confirmed" (sec 12). [= Bialas2010]
- Internal tools (not scientific sources): `tools/review/plate-dynamics-trace.patch`, `tools/review/plate-forces.py` (the open-loop test of sec 4), `plates/kinematics.rs` and other code files named in sec 2 and 6. [unresolved]

## 9. docs/design/plate-collisions-and-welding.md

Reading-method note for this document: section 3 "What Earth does" says "Checked online on 2026-10-01 unless marked 'from memory'. Paywalled papers were confirmed from their abstracts or from secondary pages." The document marks its own statements "measured" (from runs), "read" (from the code) or "inferred". Sections cited: 2.2 (the three faults, with an "Earth" column), 3 (What Earth does), 4 (options).

- Bird 2003, Geochem. Geophys. Geosyst. 4(3):1027 (52 plates) -- no link; areas from https://en.wikipedia.org/wiki/List_of_tectonic_plates -- sec 3 "Plate sizes": 52 plates, Pacific 103.3 million km2 (20 %), seven largest cover about 92 % -- read: checked online 2026-10-01; areas from a Wikipedia page (paper reading mode not stated). [= Bird2003]
- Morra et al. 2013, EPSL 373:93-101 -- no link -- sec 3: about seven large plates and many small ones for the last 200 Myr -- read: checked online (abstract or secondary page; per the sec 3 blanket statement). [= Morra2013]
- Mallard et al. 2016, Nature 535:140-143 -- no link -- sec 3: slabs set the layout of the large plates -- read: checked online (per blanket statement). [= Mallard2016]
- GSA Today, June 2018, "Broken sheets: on the numbers and areas of tectonic plates" (title from URL slug) -- https://www.geosociety.org/gsa-today/june-2018/broken-sheetson-the-numbers-and-areas-of-tectonic-plates -- sec 3: whether large and small plates are distinct populations is disputed -- read: link given (web page). [unresolved]
- Pangaea-carrying plate share -- no source -- sec 3: "No source gives the largest plate's share through a supercontinent's life. From memory: the plate carrying Pangaea was well under half the planet, with several oceanic plates in the surrounding ocean." -- read: from memory. [unresolved]
- Lonsdale 2005, Tectonophysics 404:237-264 -- no link -- sec 3 "Plate lifetimes": Farallon plate split about 23 Ma -- read: checked online (per blanket statement). [= Lonsdale2005]
- Wikipedia: Pacific plate, Izanagi plate, Kula plate -- https://en.wikipedia.org/wiki/Pacific_plate (and /Izanagi_plate, /Kula_plate) -- sec 3: Pacific born about 190 Ma and still exists; Izanagi gone by about 55 Ma, Kula about 48 Ma -- read: web pages. The document says "No published statistic was found" for plate lifetimes. [unresolved]
- Wikipedia: Supercontinent cycle -- https://en.wikipedia.org/wiki/Supercontinent_cycle -- sec 3: period about 300-500 Myr -- read: web page. [unresolved]
- Nance 2022 -- https://pmc.ncbi.nlm.nih.gov/articles/PMC9796656/ -- sec 3: supercontinent cycle period -- read: web page (PMC). Title and journal not given. [= Nance2022]
- Mitchell et al. 2021, Nat. Rev. Earth Environ. 2:358-374 -- no link -- sec 3: a supercontinent lasts 100-300 Myr -- read: checked online (per blanket statement). Pangaea dates (assembled about 335-325 Ma, break-up about 200-175 Ma) have no separate source named. [= Mitchell2021]
- Wikipedia: Iapetus Ocean -- https://en.wikipedia.org/wiki/Iapetus_Ocean -- sec 3: Iapetus open from about 600 to 420 Ma -- read: web page. [unresolved]
- J. T. Wilson 1966, Nature 211:676-681 -- no link -- sec 3: Wilson cycle -- read: not stated (cited with the Atlantic opening age of "about 175-180 Myr", for which no other source is named). [= Wilson1966]
- Copley, Avouac and Royer 2010, J. Geophys. Res. 115:B03410 -- no link -- sec 3 "Collisions": India-Asia met 59-50 Ma, convergence fell from about 150 to about 40 mm/yr between 50 and 35 Ma, thickening of Tibet the main cause -- read: checked online (abstract or secondary page; per blanket statement). [= Copley2010]
- Wikipedia: Alleghanian orogeny, Uralian orogeny -- https://en.wikipedia.org/wiki/Alleghanian_orogeny ; /Uralian_orogeny -- sec 3: Alleghanian about 325-260 Ma (65 Myr), Uralian about 70-90 Myr -- read: web pages. [unresolved]
- Wikipedia pages for the Himalaya, Alps, Urals and Tibet (no URLs) -- sec 3: Himalaya 2,400 km by 150-350 km; Alps 1,200 by 250 km; Urals 2,500 km; Tibet 2,500 by 1,000 km, mean above 4,500 m -- read: web pages ("Wikipedia pages of each"). [unresolved]
- Nocquet et al. 2016, Sci. Rep. 6:28404 -- no link -- sec 3 "Rates": rock uplift about 5 mm/yr in the Himalaya, 0.5-2 mm/yr in the western Alps -- read: checked online (per blanket statement). [= Nocquet2016]
- Tremblay et al. 2015, PNAS 112:12030 -- no link -- sec 3: more than 1 mm/yr exhumation in southern Tibet at 17-12 Ma -- read: checked online (per blanket statement). [= Tremblay2015]
- Orogen-wide mean surface uplift 0.1-1 mm/yr -- no source -- sec 3: "is from memory" -- read: from memory. [unresolved]
- Forsyth and Uyeda 1975, Geophys. J. R. Astron. Soc. 43:163-200 -- no link -- sec 3 "What happens to the plates": slab pull the main driving force; slab-attached plates move 3-4 times faster; continents add drag -- read: "from its abstract". [= ForsythUyeda1975]
- Davies and von Blanckenburg 1995, EPSL 129:85-102 -- no link -- sec 3: buoyant continent resists going down, slab can break off -- read: checked online (per blanket statement). [= DaviesvonBlanckenburg1995]
- Royer and Gordon 1997, Science 277:1268-1274 -- no link -- sec 3: India-Capricorn-Australia splitting -- read: checked online (per blanket statement). [= RoyerGordon1997]
- Definition of a plate (two plates are one when they stop moving relative to each other) -- no source -- sec 3: "This is the definition of a plate; no source states it as a finding." -- read: n/a. [unresolved]
- Gurnis 1988, Nature 332:695-699; Coltice et al. 2007, Geology 35:391-394 (up to about 100 degC) -- no links -- sec 3 and 4.3: insulation heats the mantle under supercontinents -- read: checked online (per blanket statement). [= Gurnis1988; = Coltice2007]
- Heron and Lowman 2011 (about 50 degC); Lenardic et al. 2011 -- no links or journals -- sec 3: heating small -- read: checked online (per blanket statement). [= HeronLowman2011; = Lenardic2011]
- Bercovici and Long 2014, Geophys. Res. Lett. 41:6659-6666 -- no link -- sec 3: cause is subduction around the supercontinent -- read: checked online (per blanket statement). [= BercoviciLong2014]
- Cortial et al. (2019; named "Cortial") -- no citation here -- sec 2.3, 4.1, 4.2: terrane transfer, the 300 km threshold, the weak h(z) uplift term, "Cortial's" rules -- read: not stated in this document. [= Cortial2019]
- Internal tools (not scientific sources): `tools/review/plate-dynamics.sh`, `plate-dynamics.py`, `plate-dynamics-trace.patch`, `deeptime plate-check`. [unresolved]

## 10. docs/design/p1-26-per-cell-climate.md

Reading-method note for this document: it says "Every number below was measured for this document, unless it says 'estimate' or names a source" (line 10). Section 9 "References and licences" lists the sources with the licence "checked 2026-09-30" and marks with a dagger: "† Cited from memory, not re-read in this session. The implementing tasks verify the page numbers before code headers cite them." (lines 1162-1163). Other reading notes in the table: PlaHab "not inspected, nothing taken"; UVic EMBM "code licence unverifiable (site unreachable, forks unlicensed)"; Zhuang et al. "repository licence not established". The document says its design "ports no new code. Every new piece is implemented from equations" and that GPL tools are oracles only. Sections cited: 1 (current model), 2 (temperature on the grid; 2.10 sources), 3 (moisture budget), 4 (what stays banded), 7 (validation), 9 (references), Appendix A (how measured), Appendix B (prototype).

Papers and books (section 9 table unless noted; the dagger marks "cited from memory, not re-read")
- North, Mengel and Short 1983, JGR 88(C11) 6576, doi:10.1029/JC088iC11p06576 -- no URL -- sec 2.1, 2.3, 2.5, 2.10, 9: seasonal EBM with explicit continents; NMS83 constants and surface-dependent diffusivity; land heat capacity 1.87x10^6 J/m2/K (0.45 m) -- read: not stated ("paper; equations only"). [= North1983]
- Zhuang, North and Stevens 2017, SoftwareX 6, 198 -- no URL -- sec 2.5, 2.10, 9: 2-D EBM lineage and constants "as tabulated in Ziegler and Rehfeld 2021" -- read: not stated; "repository licence not established; equations only". [= Zhuang2017]
- Ziegler and Rehfeld 2021, GMD 14, 2843 (TransEBM; Table 1 constants) -- no URL -- sec 2.1, 2.5, 9: 128 x 64 (T42) grid precedent; constants -- read: not stated; paper CC BY 4.0, TransEBM code GPL-3 (oracle only). [= Ziegler2021]
- North and Kim 2017, "Energy Balance Climate Models", Wiley (ch. 8) -- no URL -- sec 2.10, 9: 2-D EBM methods -- read: not stated. [= NorthKim2017]
- North, Cahalan and Coakley 1981, Rev. Geophys. 19, 91 -- no URL -- sec 2.4, 2.10, 9: A, B, D, P2 albedo "in use" -- read: not stated. [= North1981]
- climlab (Rose 2018, JOSS 3, 659) -- no URL -- sec 2.2, 2.10, 9: today's EBM port, MIT -- read: not stated. [= Rose2018]
- fastscape-lem/orographic-precipitation (Smith and Barstad 2004, JAS 61, 1377) -- no URL -- sec 3.6, 9: Smith and Barstad orographic rain port, MIT -- read: not stated. [unresolved]
- Stommel 1948, Trans. AGU 29, 202 -- no URL -- sec 2.6, 9: wind-driven gyre equation "implemented from the paper" -- read: not stated. [= Stommel1948]
- Rose and Marshall 2009, JAS 66, 2828 -- no URL -- sec 2.10, 9: ocean heat transport and sea ice in EBMs (alternative closure) -- read: not stated. [= RoseMarshall2009]
- Ferreira et al. 2018, Annu. Rev. Earth Planet. Sci. 46, 327, "Atlantic-Pacific asymmetry in deep water formation" -- no URL -- sec 2.6, 9 (marked with a dagger): basin width selects the sinking basin -- read: **from memory, not re-read**. [= Ferreira2018]
- Trenberth and Caron 2001, J. Climate 14, 3433 -- no URL -- sec 2.6, 9: ocean heat transport targets (ocean about 2 PW at 15-20 N) -- read: not stated. [= Trenberth2001]
- Lindzen and Nigam 1987, JAS 44, 2418 -- no URL -- sec 3.2, 9 (dagger): linear frictional surface wind from temperature gradients -- read: **from memory, not re-read**. [= Lindzen1987]
- Dommenget and Floter 2011, Clim. Dyn. 37, 2143; Stassen, Dommenget and Loveday 2019, GMD 12, 425 (GREB) -- no URL -- sec 3.3, 9: bulk evaporation form, precipitation on RH and convergence -- read: not stated; paper CC BY; code github.com/christianstassen/greb-official has no LICENSE file (Zenodo "Other (Open)"); equations only. [= Dommenget2011; = Stassen2019]
- Fanning and Weaver 1996, JGR 101(D10), 15111; Weaver et al. 2001 (UVic EMBM) -- no URL -- sec 9: energy-moisture balance atmosphere precedent -- read: not stated; "code licence unverifiable (site unreachable, forks unlicensed)". [= Fanning1996; = Weaver2001]
- Bretherton, Peters and Back 2004, J. Climate 17, 1517 -- no URL -- sec 3.4, 9 (dagger): precipitation vs column relative humidity, 15.6-fold per unit RH -- read: **from memory, not re-read**. [= Bretherton2004]
- Manabe 1969, Mon. Weather Rev. 97, 739 -- no URL -- sec 3.3, 9 (dagger): soil bucket -- read: **from memory, not re-read**. [= Manabe1969]
- Siler, Roe and Armour 2018, J. Climate 31, 7481; Bischoff and Schneider 2014, J. Climate 27, 4937 -- no URL -- sec 3.5, 9: moist EBM and energy-flux equator (later option Q10) -- read: not stated. [= Siler2018; = Bischoff2014]
- Wagner and Eisenman 2015, J. Climate 28, 3998 -- no URL -- sec 2.10, 9: sea ice in EBMs -- read: not stated; code (eisenman-group) has no licence. [= Wagner2015]
- Klein and Hartmann 1993, J. Climate 6, 1587 -- no URL -- sec 2.4, 9 (dagger): low cloud vs lower-tropospheric stability -- read: **from memory, not re-read**. [= Klein1993]
- Serreze, Kahl and Schnell 1992, J. Climate 5, 615 -- no URL -- sec 2.9, 9 (dagger): Arctic winter inversions; "Siberia's January surface is 10-20 K below 850 hPa" -- read: **from memory, not re-read**. [= Serreze1992]
- PlaHab (Ramirez 2023, PSJ) -- no URL (data and code CC BY 4.0 on UCF STARS) -- sec 2.5, 9: zonal/meridional diffusivity ratio precedent (1/25) -- read: "not inspected, nothing taken". [unresolved]
- Not used (licence reasons): ExoPlaSim (GPL-2.0), PlaSim (treated as GPL), Isca (GPL-3.0), CLIMBER-X (GPL-3.0), OpenLEM/LFPM (GPL-3.0-or-later), SPEEDY (non-commercial grant) -- sec 9 -- read: not stated. [unresolved]

Named methods with no citation in the document
- Charney-type desert-albedo feedback (sec 2.4), Thornthwaite aridity (sec 2.9), Fu's Budyko curve for runoff (sec 4.5, 7), Tetens saturation formula (sec 3.3), Koppen classification (throughout), Smith and Barstad orographic rain (see above), Sherman-Morrison cyclic tridiagonal solve, ADI (Lie splitting), donor-cell advection, Newton-Gauss-Seidel line relaxation (sec 2.8, 3.5) -- read: not stated (named techniques). [unresolved]

Validation data and datasets (section 9 "Validation data"; section 1.1; Appendix A)
- Beck et al. 2018, Sci. Data 5, 180214, Koppen-Geiger maps, present, 0.5 degrees (file `Beck_KG_V1_present_0p5.tif`) -- figshare 6396959 -- sec 1.1, 7.1, 9, App. A: "observed" Koppen classes; CC BY 4.0, "used for validation only", not committed -- read: dataset used (downloaded by the validation script into a gitignored cache; run on the Mac). [= Beck2018]
- Beck et al. 2023, Sci. Data 10, 724, monthly temperature and precipitation climatologies 0.5 degrees, 1961-1990 and 1991-2020 (`climate_data_0p5.zip:1961_1990/ensemble_mean_0p5.nc`) -- figshare 21789074 -- sec 1.1, 7.1, 9, App. A: "observed" monthly T and P (Brest, Yakutsk, Mumbai, Sahara and the other point values) -- read: dataset used; observed temperatures "brought to sea level with the model's 6.5 C/km on the same relief". [= Beck2023]
- GMT `earth_relief_30m_p` (SRTM15+, Tozer et al. 2019), via the GMT data server -- no URL -- sec 1.1, 9, App. A: Earth's relief at 720 x 360 (30 arc-minute) for the validation run -- read: dataset used. [= Tozer2019]
- ETOPO 2022 (NOAA) -- no URL -- sec 9: alternative relief -- read: not used in the measurements ("as an alternative"). [unresolved]
- HadISST (Open Government Licence) -- no URL -- sec 7.1, 9: optional sea-ice extra -- read: not used ("Optional extras"). [unresolved]
- GPCP v2.3 (NOAA; "GPCP precipitation (NOAA, public)") -- no URL -- sec 7.1, 9: optional precipitation extra -- read: not used. [unresolved]
- WorldClim -- no URL -- sec 9 "Not": non-commercial, no redistribution -- read: not used. [unresolved]
- Peel et al. 2007 map files (CC BY-NC-SA) -- no URL -- sec 9 "Not": "Peel's classification rules are formulae and are in use" -- read: map files not used; rules used. [unresolved]
- Observed sea-surface temperatures at coastal points ("about 11 C off Brest", "about 7 C off Bergen", "about 15 C off Lisbon") -- no source -- sec 2.6 and App. A item 6: "round climatological values for orientation, not part of the scored data" -- read: not stated (rounded from general knowledge). [unresolved]
- Internal tools (not scientific sources): `deeptime climate-dem`, `deeptime dump-fields`, scratch crate `p126-scratch` (prototype, about 1,100 lines of Rust, not committed), `tools/validation/climate_vs_obs.py`. [unresolved]

## 11. docs/design/plate-check-baseline.md

Reading-method note for this document: it is the measured baseline of the `deeptime plate-check` tool on the project's own runs, so it cites one outside source, in a table cell and a closing paragraph, with no reading note.

- Forsyth and Uyeda (1975; the year is given in the other design documents, not here) -- no link -- section "Against the targets" (reported-only table row "Slab-attached / other plate speed ... Earth: 3 to 4 (Forsyth and Uyeda)") and section "What the force-balance measures say today" ("Slab-attached plates move 1.4 times faster than the rest (Earth: 3 to 4)") -- claim: Earth's ratio of slab-attached to other plate speed -- read: not stated. [= ForsythUyeda1975]
- Internal references (not scientific sources): `crates/deeptime-core/src/plates/check.rs` (`MEASURES`), `plates/dynamics.rs`, `subduction::Pull::front_km`, `PlateStep::boundary_km`, `tools/review/plate-check.sh`, `docs/design/plate-collisions-and-welding.md` (the design's "Now" figures). [unresolved]
