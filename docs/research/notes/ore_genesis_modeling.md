# Ore genesis modeling for a geology-driven worldgen mod (deposits, sizing, mineral systems, preservation, game schemas)

Research date 2026-09-27. Scope: what a simulated geologic history must track to place ore, fuel, evaporite and gem deposits; how to size and space them stochastically; reusable tools, data and rule systems; how games and Minecraft mods model ore today; a proposed data-driven deposit schema. Tectonic and surface-process engines are treated as inputs only.

Method note: the session's WebSearch budget was exhausted early, so sources were retrieved directly: OpenAlex/Semantic Scholar abstracts (DOI links below), the USGS Publications Warehouse API and report PDFs (full text parsed locally), USGS Bulletin 1693 full text (47 descriptive models parsed), the Create Ore Excavation 1.6.8 jar shipped in the MIC pack, and sub-agent fetches of GitHub sources and wikis. Wherever only a title could be verified, a claim is marked "(title-level)". Abbreviations: B1693 m17 = USGS Bulletin 1693 (Cox & Singer eds. 1986) model 17, full text at https://pubs.usgs.gov/bul/1693/report.pdf. "AV100" = Economic Geology 100th Anniversary Volume (2005) review chapters.

## Q1. Deposit catalog by setting and process: what the simulation must track for each class

### Takeaway
Each major deposit class is controlled by a small, well-documented set of ingredients: tectonic regime, a specific magma or fluid source, a host or trap lithology, a depth and temperature window, and often a time window set by Earth's evolving redox state or the supercontinent cycle. Surface deposits add climate, drainage and landscape stability. A simulation that records per-cell tectonic regime history, magma type and oxidation, lithology stack, fault architecture, basin fill, heat flow, paleolatitude and climate, sea level, and a burial/exhumation ledger can place every class below with published, citable rules.

### Cited Findings

**A. Arc and granite-related magmatic-hydrothermal systems**

| Class | Setting | Hosts | Age dependence | Depth / T window | Geometry | Associated & indicator features | Weathering / surface expression | Sources |
|---|---|---|---|---|---|---|---|---|
| Porphyry Cu±Mo±Au | Convergent margins (continental and island arcs). The largest deposits sit in compressive arcs with thickened crust and active uplift. Since ~20 Ma the Pacific giants coincide with subduction of aseismic ridges, seamount chains or plateaus, and flat slabs. | Tonalite–monzogranite or syenitic porphyry stocks intruding granitic, volcanic or calcareous sedimentary rocks | "Mainly Mesozoic and Cenozoic, but may be any age". More than half of the 25 largest formed in three Cenozoic windows. Precambrian porphyries are scarce, probably removed by exhumation. | Parental plutons at 5–15 km paleodepth feed stocks more than 3 km tall. Fluids cool from >700 to <250 °C. Porphyry Cu-Au bodies are emplaced at 1–2 km. | Stockwork veinlets and disseminations in zoned alteration shells, up to 4 km³ of altered rock. Systems cluster in belts hundreds of km long that stay active for ≥10 Myr. | Zoned alteration (sodic-calcic, potassic, phyllic, argillic, propylitic). A lithocap with high-sulfidation epithermal ore lies above; Cu-Au skarn, distal Zn-Pb skarn, carbonate-replacement and sediment-hosted Au sit peripherally. Geochemistry: Cu-Mo-Au-Ag-W-B-Sr at the core, Pb-Zn-Au-As-Sb outward. | Leached cap stained with red-streak hematitic limonite. Green and blue Cu carbonates and silicates. A supergene chalcocite blanket forms at the (paleo)water table. Rutile in residual soil. | [Sillitoe 2010](https://doi.org/10.2113/gsecongeo.105.1.3); [B1693 m17, m20c](https://pubs.usgs.gov/bul/1693/report.pdf); [Cooke et al. 2005](https://doi.org/10.2113/gsecongeo.100.5.801); [Seedorff et al. 2005](https://doi.org/10.5382/av100.10); [Kesler & Wilkinson 2006](https://doi.org/10.2113/gsecongeo.101.5.919) |
| Epithermal Au-Ag (high-, intermediate-, low-sulfidation) | Subaerial volcanic arcs at convergent margins, plus intra-arc, back-arc and post-collisional rifts. Creede-type veins need saline fluids from evaporite or seawater-bearing basement. | Coeval volcanic rocks and basement. The highest grades are in steep veins. Bulk-tonnage ore is in breccias and leached rock. | Mostly Tertiary and younger because shallow deposits are preferentially lost to erosion. Rare Archean examples exist. | Depth <1.5 km below the water table, temperature <300 °C. High-sulfidation disseminated Au lies within the uppermost ~500 m of lithocaps. | Veins, stockworks, breccias, disseminations. Districts cover <10 to ~200 km². Deposit sizes range from 10s of kt to >1 Gt ore at 0.1 to >30 g/t Au. | Low/intermediate-sulfidation: quartz-adularia-illite-calcite, banded crustiform veins. High-sulfidation: vuggy quartz and alunite with enargite. Au-As-Sb-Hg-Tl anomalies high in the system. | Bleached country rock with jarosite, alunite, limonite and kaolinite. Supergene processes can raise grade. | [Simmons et al. 2005](https://doi.org/10.5382/av100.16); [USGS SIR 2010-5070-Q](https://pubs.usgs.gov/publication/sir20105070Q); [Sillitoe 2010](https://doi.org/10.2113/gsecongeo.105.1.3); [B1693 m25a–d](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Skarns (Fe, Au, Cu, Zn, W, Mo, Sn) | Mostly adjacent to plutons in limestone-bearing sequences. Also along faults and shear zones, on the seafloor, and in deep metamorphic terranes. Cu skarn: continental-margin late-orogenic magmatism. Fe skarn: oceanic arcs, Andean arcs, rifted margins. | Carbonate or calcareous clastic rocks at intrusive contacts | Mainly Mesozoic–Tertiary, but may be any age | Cu skarns form in a relatively shallow environment. W skarns sit around coarse equigranular batholiths with large high-T aureoles. | Zoned: proximal garnet, distal pyroxene, then wollastonite or sulfide at the marble front | Pluton chemistry predicts the metal: Fe and Au skarns with Mg-rich, reduced plutons; Cu, Zn and Mo with oxidized plutons; Sn with evolved reduced granites. Magnetic anomalies. | Cu carbonates and Fe-rich gossan. Magnetite float. Calc-silicate pebbles in streams point to covered skarns. | [Meinert et al. 2005](https://doi.org/10.5382/av100.11); [B1693 m18b, m18d, m14b](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Sn-W greisen, veins and Sn porphyry | Foldbelts and accreted margins with late- to post-orogenic granites. Cu-Au forms in the main arc, while Sn forms after large-scale intracrustal melting in the back arc. | Specialized S-type, ilmenite-series (reduced) leucogranites, SiO₂ >73%, K₂O >4%, with topaz, fluorite, tourmaline and beryl. Pelitic wallrocks. | Paleozoic–Mesozoic most common; any age possible | 200–500 °C, from a few hundred m to 6 km depth (average ~2 km) | Greisen and stockwork in granite cupolas; veins; zoned systems with inner Sn-W-Cu-As-Bi and an outer Pb-Zn-Ag-Sb-Au-U halo | Magnetic susceptibility separates barren magnetite-series from fertile ilmenite-series granites. Cassiterite, topaz and tourmaline appear in stream sediment. | Granite reddened near greisen. Rich tin placers form within a few km of the source, and placers supplied at least half of all tin ever mined. | [Lehmann 2020](https://doi.org/10.1016/j.lithos.2020.105756); [Kesler & Wilkinson 2013](https://doi.org/10.1144/sp393.3); [B1693 m15b, m15c](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Porphyry Mo (arc-related and Climax-type) | Arc-related: differentiated calc-alkaline granitoids. Climax-type: extensional zones in cratons far from margins, on thick crust, late in the cycle. | Climax-type: granite-rhyolite with >75% SiO₂ and high Rb, Y, Nb | Climax examples are mainly mid-Tertiary | Hypabyssal | Veinlet and fracture-controlled. Arc-related ore runs 0.03–0.22% Mo at large tonnage. | Mo-Sn-W-Rb anomalies above ore; Pb-Zn-F-U halo up to several km away | Yellow ferrimolybdite stains | [USGS SIR 2010-5070-D](https://pubs.usgs.gov/publication/sir20105070D); [B1693 m16](https://pubs.usgs.gov/bul/1693/report.pdf) |

**B. Seafloor and basin-brine systems**

| Class | Setting | Hosts | Age dependence | Depth / T | Geometry | Associated & indicator features | Weathering | Sources |
|---|---|---|---|---|---|---|---|---|
| VMS (VHMS) | Extensional phases of convergent systems. Five lithostratigraphic types: bimodal-mafic (rifted oceanic arcs), mafic (primitive back-arcs and ophiolites, e.g. Cyprus), pelite-mafic (mature back-arcs, Besshi), bimodal-felsic (rifted continental arcs) and siliciclastic-felsic (mature continental back-arcs, Iberia). Modern high-temperature vent sites: 65% at mid-ocean ridges, 22% in back-arcs, 12% on arcs. | Submarine volcanic rocks with or without sediments. Kuroko types are associated with organic mudstone. | 3.55 Ga to active today. Peaks at 2740–2690, 1910–1840, 510–460 and 390–355 Ma track supercontinent assembly (Kenorland, Nuna, Gondwana, Pangea). VMS does not form during breakup. Sulfate (barite) content rises with atmospheric oxidation. | At or near the seafloor. Basalt-derived fluids reach 350–400 °C. | Massive-sulfide lens (>60% sulfide) above a stringer or stockwork feeder. Sizes range from pods <1 t to Rio Tinto at 1.5 Gt. Felsic-type deposits have a median area of 0.02 km² and a median alteration footprint of 3.2 km². | Cu-Zn in mafic types, with Pb added in felsic types. Hanging-wall exhalites (chert, Mn, Eu, Tl). Gahnite in streams. | Massive limonite gossans; gold in stream sediment | [Franklin et al. 2005](https://doi.org/10.5382/av100.17); [Hannington et al. 2005](https://doi.org/10.5382/av100.06); [Huston et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.571); [USGS SIR 2010-5070-C](https://pubs.usgs.gov/publication/sir20105070C); [USGS OFR 2009-1034](https://pubs.usgs.gov/of/2009/1034/); [B1693 m24a, m28a](https://pubs.usgs.gov/bul/1693/report.pdf) |
| SEDEX / clastic-dominated (CD) Pb-Zn | Passive margins, back-arcs, continental rifts and sag basins. Epicratonic embayments with synsedimentary growth faults (half-grabens); small third-order basins act as traps. Forms during supercontinent breakup. 85% of sediment-hosted base-metal deposits, including all giants (>10 Mt metal), lie within 200 km of the transition from thick to thin lithosphere. | Euxinic black shale, siltstone, chert, dolostone and turbidites, with local evaporites | None before 2.02 Ga (needs an oxidized hydrosphere). Major pulse 1.85–1.58 Ga, then 530–300 Ma. | Synsedimentary to early diagenetic, from basinal brines | Stratiform laminated sulfide. Zoned laterally Cu→Pb→Zn→Ba away from the feeder. Host basins contain >10 Mt Zn+Pb. | Exhalative chert, barite, hematite; NH₃ anomalies | Large gossans with Pb-Zn-Cu carbonates, sulfates and silicates | [Leach et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.593); [Hoggard et al. 2020](https://doi.org/10.1038/s41561-020-0593-2); [Huston et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.571); [USGS SIR 2010-5070-N](https://pubs.usgs.gov/publication/sir20105070N); [B1693 m31a](https://pubs.usgs.gov/bul/1693/report.pdf); [Large et al. 2005](https://doi.org/10.5382/av100.28) |
| MVT Pb-Zn | Platform carbonates, typically on passive margins, later tapped by basin brines driven by orogenic uplift. Deposits sit on reefs over basement highs and at the margins of clastic basins. | Dolostone and limestone | First appear at 2.02 Ga. Maximum abundance during Pangea assembly (Devonian–Carboniferous); a new era began after Pangea breakup. Needs evaporites and oxidized brines. | Pine Point model: 60–100 °C brines flowing at 1–5 m/yr for 0.5–5 Myr, driven by topography. In Southeast Missouri the ore window lasted only 100s of kyr. | Open-space fill, replacement and breccia bodies. Zoned Cu(-Ni-Co)→Pb→Zn upward. Trace mineralization extends across ~1,000 km. | Regional Pb-Zn in insoluble residues; background Zn 20 ppm, Pb 9 ppm | Zinc silicate and carbonate (nonsulfide) ore in the oxidation zone | [Leach et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.593); [Garven 1985](https://doi.org/10.2113/gsecongeo.80.2.307); [Bethke 1986](https://doi.org/10.2113/gsecongeo.81.2.233); [Appold & Garven 1999](https://doi.org/10.2113/gsecongeo.94.6.913); [Cathles & Adams 2005](https://doi.org/10.5382/av100.05); [B1693 m32a, m32b](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Sediment-hosted stratiform Cu (Kupferschiefer, Copperbelt) | Failed rifts that became intracratonic basins at low latitude during supercontinent breakup. Synrift red beds are overlain by marine or lacustrine sediments and evaporites. Hydrologically closed basins let the process run for 10s–100s of Myr. | Red beds with an overlying reduced shale or dolomite; sabkha and evaporite beds | Paleoproterozoic onward, after red beds appear. Supergiants (>24 Mt Cu) occur in only three basins (Udokan, Katanga, Zechstein). | Moderate-temperature oxidized saline brines crossing a reduction front | Disseminated, stratabound. About 20% of world Cu and the leading Co source. | Cu-Ag-(Co), low Au; weak radioactivity | Surface exposures may be fully leached; chalcocite enrichment down-dip | [Hitzman et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.627); [Hitzman et al. 2005](https://doi.org/10.5382/av100.19); [USGS SIR 2010-5070-M](https://pubs.usgs.gov/publication/sir20105070M); [B1693 m30b](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Unconformity U | Intracratonic basins on the flanks of Archean domes: an unconformity between graphitic metapelite basement and younger unmetamorphosed quartz arenite; stable since the Mesoproterozoic | Metamorphosed carbonaceous pelites plus quartz arenites | 1.8–1.2 Ga. High-grade large deposits formed essentially in the late Paleoproterozoic to early Mesoproterozoic. Oxidized U-bearing fluids exist only after ~2.2 Ga. | Diagenetic-hydrothermal. Near-100 wt% U accumulations formed where oxidized brines met methane seeps (Athabasca). | Breccia fill, veins and disseminations at the unconformity and on basement faults | Coarse uraninite and pitchblende; radioactivity | Secondary U minerals (autunite, torbernite, uranophane) | [B1693 m37a](https://pubs.usgs.gov/bul/1693/report.pdf); [Cuney 2010](https://doi.org/10.2113/gsecongeo.105.3.553); [Cathles & Adams 2005](https://doi.org/10.5382/av100.05) |
| Sandstone / roll-front U | Stable platforms, foreland-interior basins and coastal plains beside major uplifts; fluvial channels and braided streams. U comes from felsic volcanic ash or eroding felsic plutons. | Feldspathic or tuffaceous sandstone between mudstones | Devonian and younger; roll-fronts mainly Tertiary. This family developed essentially only after ~0.45 Ga, once land plants supplied reductants. | Groundwater-driven, near surface. In Texas a semi-arid climate favored weathering of the ash source; the humid east lacked economic deposits. | Crescent-shaped rolls at the oxidized–reduced interface that cut across bedding; tabular lenses around plant debris | Plant debris or migrating H₂S/hydrocarbons act as reductants. Alteration zoning from oxidized to reduced sandstone. | Yellow carnotite bloom | [B1693 m30c](https://pubs.usgs.gov/bul/1693/report.pdf); [Cuney 2010](https://doi.org/10.2113/gsecongeo.105.3.553); [USGS 2017 Texas roll-type U](https://pubs.usgs.gov/publication/70179186); [USGS Bulletin 2141](https://pubs.usgs.gov/publication/b2141) |

**C. Orogenic, craton-margin and pegmatite systems**

| Class | Setting | Hosts | Age dependence | Depth / T | Geometry | Associated & indicator features | Weathering | Sources |
|---|---|---|---|---|---|---|---|---|
| Orogenic Au (lode, including greenstone and Homestake types) | Accretionary and collisional orogens: continental-margin mobile belts and accreted margins. Forms synchronously with the late stages of orogeny, on faults produced by regional compression. | Greenstones, turbidites, BIF, Fe-rich mafic rocks, porphyry stocks. Metamorphism is typically greenschist facies. | Pulses at 2.8–2.55 Ga, 2.1–1.8 Ga and 600–50 Ma. Precambrian examples are more abundant than exhumation alone predicts. | Epizonal, mesozonal and hypozonal subtypes defined by P–T conditions (numeric depths not verified; see Gaps). Deformation regenerates permeability; metamorphic devolatilization supplies the fluid. | Quartz veins and saddle reefs. Deposits cluster into camps spaced every 30–50 km along narrow belts, with Pareto (power-law) size distributions. | Arsenic is the best pathfinder (Ag-Pb-Zn-Cu also). Quartz chips in soil. | Gold recoverable from soil by panning. Placers: 15,500 t Au in Cordilleran orogens. | [Goldfarb et al. 2005](https://doi.org/10.5382/av100.14); [Robert et al. 2005](https://doi.org/10.5382/av100.30); [S.F. Cox 2005](https://doi.org/10.5382/av100.04); [Kesler & Wilkinson 2006](https://doi.org/10.2113/gsecongeo.101.5.919); [B1693 m36a, m36b](https://pubs.usgs.gov/bul/1693/report.pdf) |
| IOCG (Olympic Dam, Candelaria) | The largest are intracratonic, within ~100 km of Archean or Paleoproterozoic craton margins or other lithospheric boundaries. They formed 100–200 Myr after supercontinent assembly in an anorogenic setting, with mantle underplating or plumes. Phanerozoic examples sit in extensional or transtensional arc segments. | Granite and volcanic basement; breccias; pervasive alkali (Na, K) metasomatism | Giant IOCGs are mainly a Precambrian phenomenon | Magmatic-hydrothermal, from highly saline fluids | Structurally controlled breccias. Alteration zones are regional in scale relative to ore. | Abundant low-Ti magnetite and hematite; LREE, U, Co, F, P; gravity and magnetic highs | Near-surface deposits are preserved above buoyant subcontinental lithospheric mantle | [Groves et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.641); [Williams et al. 2005](https://doi.org/10.5382/av100.13); [B1693 m29b](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Carlin-type Au (sediment-hosted disseminated) | Passive-margin sequence followed by compressional orogenies that built steep conduits, shallow low-dipping traps and reactive calcareous hosts | Pyritic calcareous sedimentary rocks | Nevada deposits formed ~42–36 Ma | Shallow | Disseminated, stratabound | Fine-grained, refractory Au | Oxidation makes ore amenable to processing | [Cline et al. 2005](https://doi.org/10.5382/av100.15) |
| Pegmatites: LCT (Li-Cs-Ta, plus Sn, Be and gems) and NYF (Nb-Y-F, plus REE) | LCT: orogenic hinterlands, mostly collisional, emplaced late in orogeny along pre-existing structures near evolved peraluminous S-type granites. NYF: from A- and I-type granites. | Metasupracrustal rocks at upper greenschist to lower amphibolite facies | Age peaks match supercontinent assembly: LCT maxima at 2638, 1800, 962, 529, 485, 371, 309 and 274 Ma. Li and Ta resources concentrate in Archean and Phanerozoic rocks (preservation). | Crystallize at ~350–550 °C in days to years | Dikes and sills with giant crystals (spodumene 14 m, beryl 18 m, K-feldspar 49 m) | Spodumene, petalite, lepidolite, pollucite, columbite-tantalite, cassiterite, beryl, tourmaline, gemstones | — | [USGS SIR 2010-5070-O](https://pubs.usgs.gov/publication/sir20105070O); [Černý & Ercit 2005](https://doi.org/10.2113/gscanmin.43.6.2005); [McCauley & Bradley 2014](https://doi.org/10.3749/canmin.52.2.183) |
| Carbonatite and peralkaline REE-Nb-P | Continental shields and rift lineaments such as the East African Rift | Carbonatite, pyroxenite, ijolite, fenite | Complexes intrude Precambrian shields but may themselves be much younger | Magmatic to metasomatic | Disseminated and banded | Radiometric, magnetic and gravity highs; LREE (La, Ce, Pr, Nd) | Goethite-rich soils enriched in P, Nb and REE; lateritic P-Nb deposits | [B1693 m10](https://pubs.usgs.gov/bul/1693/report.pdf); [USGS SIR 2010-5070-J](https://pubs.usgs.gov/publication/sir20105070J); [Freyssinet et al. 2005](https://doi.org/10.5382/av100.21) |

**D. Orthomagmatic (magmatic segregation) systems**

| Class | Setting | Hosts | Age dependence | Geometry | Associated & indicator features | Weathering | Sources |
|---|---|---|---|---|---|---|---|
| Ni-Cu-(PGE) sulfide | Mantle-derived magmas that reach sulfide saturation, usually by assimilating S-rich crust. The ideal site is a mantle plume intersecting a continental rift containing S-rich sediments. Komatiite types occur in greenstone belts; Duluth-type in rifts. | Base of mafic and ultramafic bodies. Komatiites and tholeiitic picrites (hot, low viscosity) are the most capable. | Komatiite class 2.7–1.9 Ga; flood-basalt class 1.1–0.25 Ga; Mg-basalt class Archean to present. Most deposits are Archean–Paleoproterozoic. | Massive sulfide at the base grading upward to net-textured and then disseminated | Sulfur from pyritic shale or anhydrite (non-magmatic S isotopes); Ni/Cu ~1/3 at Duluth | Gossans (Pd, Ir retained), laterites | [Barnes & Lightfoot 2005](https://doi.org/10.5382/av100.08); [Arndt et al. 2005](https://doi.org/10.5382/av100.02); [Naldrett 2010](https://doi.org/10.2113/gsecongeo.105.3.669); [USGS SIR 2010-5070-I](https://pubs.usgs.gov/publication/sir20105070I); [B1693 m5a, m6a](https://pubs.usgs.gov/bul/1693/report.pdf); [Griffin et al. 2013 (title-level: continental-root control)](https://doi.org/10.1038/ngeo1954) |
| Stratiform chromite and PGE reefs (layered intrusions) | Large layered mafic-ultramafic intrusions in cratonic shields | Norite, pyroxenite, dunite, anorthosite | 75% of known PGE sit in three Neoarchean–Paleoproterozoic intrusions (Bushveld, Great Dyke, Stillwater) | Laterally continuous seams of uniform grade and thickness, the most continuous of any orebody | PGE concentrate near the level where crystallization switches between magma types | Reef is hard to see on weathered surfaces | [Naldrett 2010](https://doi.org/10.2113/gsecongeo.105.3.669); [Cawthorn et al. 2005](https://doi.org/10.5382/av100.09); [USGS SIR 2010-5070-E](https://pubs.usgs.gov/publication/sir20105070E); [B1693 m1, m2b](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Podiform chromite (ophiolites) | Ultramafic section of ophiolites formed at mid-ocean ridge, off-ridge and supra-subduction settings, later exposed in accreted terranes | Deformed, commonly serpentinized dunite and harzburgite near the cumulate–tectonite contact | Phanerozoic | Massive to disseminated pods | Chromite; PGE (Rh, Ir, Ru, Pd, Pt) | Highly resistant to weathering (placer and eluvial chromite) | [B1693 m8a](https://pubs.usgs.gov/bul/1693/report.pdf); [USGS SIR 2012-5157](https://pubs.usgs.gov/publication/sir20125157) |
| Kimberlite and lamproite diamonds | Cratons stable since the Early Proterozoic. Pipes do not correlate with orogenies; they follow epeirogenic warping, doming and basement fracture zones. 99% of economic diamonds come from subcontinental lithospheric mantle. Archean cratons carry ~250–350 km thick keels, versus <150 km for Proterozoic and younger lithosphere. | Kimberlite diatremes; olivine lamproite | Most productive pipes are 80–100, 250 and 1,000–1,100 Ma old. Diamonds have reached the surface since ≥2.82 Ga. | Pipes (median outcrop 12 ha). Mined kimberlites carry 0.1–0.6 ppm diamond. | Pyrope and G10 garnet, Mg-ilmenite (absent in lamproite), chromite, phlogopite; Ni-Nb anomalies | Pipes weather rapidly into topographic depressions and shed diamonds into placers | [B1693 m12](https://pubs.usgs.gov/bul/1693/report.pdf); [Gurney et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.689); [Gurney et al. 2005](https://doi.org/10.5382/av100.07); [Kerrich et al. 2005](https://doi.org/10.5382/av100.33); [USGS diamond pipe models 1992](https://pubs.usgs.gov/publication/70016991) |
| Fe-Ti(-V) oxide (massif anorthosite) | Proterozoic massif anorthosite suites (Grenville, Rogaland) | Anorthosite, norite | Proterozoic | Discordant ilmenite bodies, rarely >300 Mt at 10–45% TiO₂ | Ilmenite, hemo-ilmenite | — | [USGS SIR 2010-5070-K](https://pubs.usgs.gov/publication/sir20105070K) |

**E. Chemical and biogenic sediments and fuels**

| Class | Setting and controls | Age dependence | Geometry | Associated / indicators | Weathering | Sources |
|---|---|---|---|---|---|---|
| Banded / granular iron formation (Algoma, Superior) | Algoma type is interlayered with submarine volcanics in greenstone belts. The larger Superior type lies on passive-margin shelves and intracratonic basins, without direct volcanic links. The acme needed a large hydrothermal Fe supply, big continental shelves and a stratified ocean. | Acme ~2.65–1.85 Ga. Superior type mostly 2.0±0.2 Ga; Algoma mostly Archean. BIF (and paleoplacer U) do not reappear in younger basins; they are redox-restricted. | Centimetre-banded chert and Fe-oxide, silicate, carbonate or sulfide layers | Commonly interlayered with quartzite, shale and dolomite | Silica leaching converts it to high-grade supergene hematite ore; channel iron deposits form in Tertiary paleochannels | [Bekker et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.467); [Clout & Simonson 2005](https://doi.org/10.5382/av100.20); [B1693 m28b, m34a](https://pubs.usgs.gov/bul/1693/report.pdf); [Groves et al. 2005](https://doi.org/10.1144/gsl.sp.2005.248.01.04); [Freyssinet et al. 2005](https://doi.org/10.5382/av100.21) |
| Quartz-pebble-conglomerate Au-U (Witwatersrand) | Fault-bounded foreland or retroarc basins fed by eroding granite-greenstone terranes; alluvial fans | 3.1–2.2 Ga (Central Rand basin 2.90–2.84 Ga). Detrital pyrite and uraninite survive only under a reducing atmosphere; younger analogues contain Fe oxides. | Reefs on unconformities, cm to m thick and 100s of km² in area | Pyrite, uraninite, bitumen, heavy minerals | Residual gold. The deposit type holds ~40% of all gold ever produced. | [B1693 m29a](https://pubs.usgs.gov/bul/1693/report.pdf); [USGS SIR 2010-5070-P](https://pubs.usgs.gov/publication/sir20105070P); [Frimmel et al. 2005](https://doi.org/10.5382/av100.23); [Cuney 2010](https://doi.org/10.2113/gsecongeo.105.3.553) |
| Evaporites (halite, potash) | Arid to hyperarid climate. Hydrographically restricted basins with periodic seawater replenishment and little siliciclastic input. Potash basins lie between 15° and 45° N/S. Basin type (rift, foreland, sag, transform) matters less than climate. The sequence runs red beds → carbonate → anhydrite/gypsum → halite → potash, in cycles from cm to 100s of m. | Potash basins are Neoproterozoic or younger; half are Middle–Late Devonian, Permian or Paleogene–Neogene. Mineralogy tracks seawater chemistry oscillating between KCl-type and MgSO₄-type seas. | Stratabound beds, or halokinetic salt domes and diapirs. Basins hold tens of Mt to >100 Gt K₂O. Elk Point basin: ~1.2 million km², with potash in the top 100 m of the Prairie Evaporite. | Sylvite, carnallite, polyhalite; bromine | Dissolution at the surface; USGS assessment tracts stop at ≤3 km depth | [USGS SIR 2010-5090-S](https://pubs.usgs.gov/publication/sir20105090S) (full text); [USGS SIR 2010-5090-CC](https://pubs.usgs.gov/publication/sir20105090CC); [Warren 2010 (title-level)](https://doi.org/10.1016/j.earscirev.2009.11.004) |
| Phosphorite | Upwelling type: marine basins well connected to the open sea with high plankton productivity. Warm-current type: basins on the flanks of rising domes and at river mouths. Both mostly between 40° N and 40° S; most deposits formed at low latitude. In a rifting sequence, phosphorite is the late-stage product (volcanism early, evaporites intermediate). | Upwelling type Precambrian–Miocene; warm-current type Early Cretaceous–Pliocene. Deposition is sporadic, as continents drift into low latitudes. | Pellets, nodules, phosphatized shell and bone | P, N, F, C, U; anomalously radioactive | Limonite, goethite; lateritic phosphate over carbonates | [B1693 m34c, m34d](https://pubs.usgs.gov/bul/1693/report.pdf); [Cook & McElhinny 1979](https://doi.org/10.2113/gsecongeo.74.2.315) |
| Coal | Coal is preserved only where the rise in accommodation roughly equals the peat production rate. For mires, base level is the groundwater table, which in paralic settings is set by sea level and the precipitation/evaporation ratio. Below a threshold accommodation rate no continuous peat forms; excess accommodation drowns mires. | Needs land plants. The Carboniferous coal peak reflects environmental and tectonic conditions, not a delay in lignin-degrading fungi. High accumulation rates continue wherever conditions permit. | Laterally continuous seams (paralic mid-lowstand and highstand tracts) or thicker isolated seams (transgressive turnarounds). Peat compacts to coal about 11:1 in a Utah case. | Rank follows burial temperature history (vitrinite reflectance; see Q4) | — | [Bohacs & Suter 1997](https://doi.org/10.1306/3b05c3fc-172a-11d7-8645000102c1865d); [Ryer & Langer 1980](https://doi.org/10.1306/212f7b44-2b24-11d7-8648000102c1865d); [Nelsen et al. 2016](https://doi.org/10.1073/pnas.1517943113) |
| Petroleum system | Required elements: mature source rock, migration pathway, reservoir, trap and seal, with correct relative timing of generation, migration and accumulation. The critical moment is the maximum burial of the source and the time of highest trapping probability. Catagenesis (oil generation, then secondary cracking to gas at higher T) spans ~50–150 °C. Source effectiveness depends on age, paleolatitude, structural form and biota. | Six intervals covering one-third of Phanerozoic time sourced >90% of discovered reserves: Silurian 9%, Upper Devonian–Tournaisian 8%, Pennsylvanian–Lower Permian 8%, Upper Jurassic 25%, middle Cretaceous 29%, Oligocene–Miocene 12.5%. About 70% of reserves were generated since the Coniacian and about 50% since the Oligocene. | Structural or stratigraphic traps; salt diapirs make traps | — | Erosion or over-maturation destroys accumulations (inferred from the preservation clause of the critical-moment definition) | [SLB: petroleum system](https://glossary.slb.com/en/terms/p/petroleum_system); [SLB: critical moment](https://glossary.slb.com/en/terms/c/critical_moment); [SLB: catagenesis](https://glossary.slb.com/en/terms/c/catagenesis); [Klemme & Ulmishek 1991](https://doi.org/10.1306/0c9b2a47-1710-11d7-8645000102c1865d); [Magoon & Dow 1994](https://doi.org/10.1306/m60585) |

**F. Supergene and residual (weathering) deposits**

| Class | Controls | Age / timing | Geometry & zoning | Sources |
|---|---|---|---|---|
| Supergene Cu enrichment (porphyry and related) | Leaching, oxidation and chalcocite enrichment to depths of several hundred m. Needs vertically extensive, fractured orebodies with high pyrite/Cu-sulfide ratios and non-reactive (advanced argillic or sericitic) wallrock. Needs uninterrupted supergene activity for ≥0.5 Myr, typically 3–9 Myr. Surface uplift must lower the water table, in hot semiarid to pluvial climates, with erosion that does not outpace supergene processes. Hydroxychlorides indicate hyperarid conditions. | Andean and SW USA Cenozoic provinces; >50% of world-mined Cu came from these supergene-rich provinces (2005) | Hematitic leached cap over mature enrichment, goethitic over protore, jarositic over pyritic ore. The oxide zone (chrysocolla) overlies a chalcocite blanket at 1.5 to >2% Cu (2–3× hypogene). Exotic oxide-Cu deposits form in piedmont gravels downslope. | [Sillitoe 2005](https://doi.org/10.5382/av100.22); [B1693 m17](https://pubs.usgs.gov/bul/1693/report.pdf); [Cooke et al. 2005](https://doi.org/10.2113/gsecongeo.100.5.801); [Hartley & Rice 2005 (title-level)](https://doi.org/10.1007/s00126-005-0017-7) |
| Ni-Co laterite | Deep weathering of ultramafic rock (peridotite, dunite, serpentinite) under humid tropical conditions, typically within 26° of the equator. High chemical and low physical erosion. Uplift is needed to expose the ultramafic source (ophiolite emplacement at convergent margins) or a stable craton. The richest ore (>3% Ni) formed where oxide regolith was uplifted and Ni leached down into saprolite silicates. | Profiles can develop in as little as ~1 Myr. Of total laterite tonnage, 55% was weathered Miocene–present, 33% Cretaceous–mid-Tertiary, 12% Mesozoic. | Ferricrete / limonite (Ni-goethite) / smectite (poor drainage only) / saprolite (garnierite, hydrous Mg-silicates) / protolith. Oxide, hydrous-silicate and clay types. Laterites supply >60% of global Ni (2013). | [B1693 m38a](https://pubs.usgs.gov/bul/1693/report.pdf); [USGS SIR 2010-5070-H](https://pubs.usgs.gov/publication/sir20105070H); [Butt & Cluzel 2013](https://doi.org/10.2113/gselements.9.2.123); [USGS OFR 2011-1058](https://pubs.usgs.gov/of/2011/1058/); [Freyssinet et al. 2005](https://doi.org/10.5382/av100.21) |
| Bauxite (laterite and karst types) | Laterite type: aluminous silicate rocks on well-drained plateaus in warm-to-hot wet climates, in tectonically stable areas. Karst type: residual or transported material on carbonates in wet tropics, on stable land protected from erosion. Orthobauxite forms in one weathering phase; metabauxite after a shift to drier conditions; cryptobauxite under a clay cover (Amazonian). | Laterite type mainly Cenozoic and still forming; karst type Paleozoic–Cenozoic | Pisolitic, nodular or earthy blankets; Al-Ga signature | [B1693 m38b, m38c](https://pubs.usgs.gov/bul/1693/report.pdf); [Freyssinet et al. 2005](https://doi.org/10.5382/av100.21) |
| Supergene Fe, lateritic Au, P-Nb | Residual Fe ore on BIF under lateritic weathering. Au is residual in savanna climates, degraded and upper-saprolite-enriched in humid rainforest, and remobilized deeper by acid saline groundwater in semi-arid climates. P-Nb laterites form over carbonatite or phosphatic carbonate. | Cenozoic paleoclimate history | Profile-position depends on climate history | [Freyssinet et al. 2005](https://doi.org/10.5382/av100.21) |

**G. Placers (clastic concentration of resistant heavy minerals)**

| Class | Controls | Age | Geometry / indicators | Sources |
|---|---|---|---|---|
| General | Placers form within geomorphic features and concentrate dense detrital minerals, almost always by water. Development is started by tectonic activity plus climate and sea-level change. Existing placers are predominantly Cenozoic, eroded from Mesozoic and older sources. Production has shifted to mineral sands (ilmenite, rutile, zircon). | Phanerozoic, mostly Cenozoic | Classified by geomorphic setting | [Garnett & Bassett 2005](https://doi.org/10.5382/av100.25) |
| Au-PGE placers | High-energy alluvial settings where gradients flatten (inside meanders, below rapids and falls, behind boulders); surf-winnowed beaches. Tertiary conglomerates along major faults; long-eroded shields; high terraces. | Cenozoic; older placers are rarely preserved | Flattened, flaky gold. Nugget Ag content falls with distance from source. Heavy-mineral suite: magnetite, chromite, ilmenite, zircon, garnet, rutile. | [B1693 m39a, m39b](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Cassiterite placers | Stream gradients within a critical range: fast enough to separate by gravity, not so fast the channel is swept clean. Offshore placers in drowned valleys and strandlines. Sources are evolved granites. | Late Tertiary–Holocene common | Cassiterite coarsens toward the source; euhedral crystals mean the source is close. Usually within a few km of it. | [B1693 m39e](https://pubs.usgs.gov/bul/1693/report.pdf); [Lehmann 2020](https://doi.org/10.1016/j.lithos.2020.105756) |
| Diamond placers | Streams draining kimberlite areas, plus beach deposits, on stable cratons | Tertiary–Quaternary | Can lie up to 1,000 km from the source. Pyrope, Mg-ilmenite and phlogopite point to pipes. | [B1693 m39d](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Heavy-mineral sands (Ti, Zr, REE) | Sourced from deeply weathered high-grade metamorphic (sillimanite grade or higher) and igneous terranes, delivered by rivers, then sorted by waves, tides, longshore drift and wind in the upper swash zone and dunes. Sea-level transgressions and regressions control deposition and preservation. Stable craton margins. | Mostly Paleogene–Quaternary | Laminated lenses several m to tens of m thick; "shoestring" bodies parallel to the shoreline. Minerals: ilmenite, rutile, leucoxene, zircon, monazite, garnet, sillimanite/kyanite, staurolite. Fe leaching from ilmenite enriches residually. | [USGS SIR 2010-5070-L](https://pubs.usgs.gov/publication/sir20105070L); [B1693 m39c](https://pubs.usgs.gov/bul/1693/report.pdf) |

**H. Gemstones**

| Class | Controls | Sources |
|---|---|---|
| Corundum (ruby, sapphire) | Primary magmatic: xenocrysts in continental alkali basalt (E Australia, S Vietnam), lamprophyre and syenite. Metamorphic: marble-hosted ruby (Mogok) and mafic-ultramafic hosts (Montepuez). Metamorphic-metasomatic: desilicated pegmatites (plumasite), skarns, shear-zone biotite schist. Distribution is tied to collision, rift and subduction geodynamics: the Pan-African collision belt (E Africa, Madagascar, India, Sri Lanka) and the Himalayan orogeny (45 Ma–present) for Asian marble rubies. Most sapphires and rubies are now mined from placers; metamorphic-terrane placers (Sri Lanka, Madagascar) give the best sapphire. | [Giuliani & Groat 2019](https://doi.org/10.5741/gems.55.4.464) ([PDF](https://www.gia.edu/doc/WN19-geology-of-corundum-and-emerald-gem-deposits.pdf)) |
| Emerald | Colombian type: veins in Lower Cretaceous black shale of the Eastern Cordillera; evaporites may have supplied saline fluids along major faults. Veins are high in Be, Na and Mg. Schist type: pegmatites or quartz veins intruding mafic-ultramafic rock, with metasomatism producing emerald-bearing biotite schist and plumasite. | [Giuliani & Groat 2019](https://doi.org/10.5741/gems.55.4.464); [B1693 m31c](https://pubs.usgs.gov/bul/1693/report.pdf) |
| Pegmatite gems (beryl, tourmaline) and diamond | Pegmatite gems come from LCT pegmatites; diamonds from kimberlite, lamproite and placers (tables C, D, G) | [USGS SIR 2010-5070-O](https://pubs.usgs.gov/publication/sir20105070O); [B1693 m12](https://pubs.usgs.gov/bul/1693/report.pdf) |

**Secular "switches" a sim clock must know** (all cited):
- **Great Oxidation Event.** Redox-sensitive types are confined to the early Precambrian and never recur: paleoplacer U, BIF, BIF-associated Mn carbonates ([Groves et al. 2005](https://doi.org/10.1144/gsl.sp.2005.248.01.04)). Uranium deposit types step through 4.55–3.2 Ga (none), 3.1–2.2 Ga (U placers), 2.2–0.45 Ga (redox deposits, including unconformity type) and 0.45 Ga–present (plant-reductant sandstone and roll-front types) ([Cuney 2010](https://doi.org/10.2113/gsecongeo.105.3.553)). Pb-Zn in sediments first appears at 2.02 Ga ([Leach et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.593)).
- **Supercontinent cycle.** It has operated since ~2.8 Ga: Kenorland ~2.7 Ga, Columbia ~1.8 Ga, Rodinia ~1 Ga, Pangea ~0.3 Ga. Komatiitic ocean plateaus declined at ~2.6 Ga ([Kerrich et al. 2005](https://doi.org/10.5382/av100.33)). VMS peaks with assembly ([Huston et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.571)), and so do pegmatites ([McCauley & Bradley 2014](https://doi.org/10.3749/canmin.52.2.183)). IOCG giants form 100–200 Myr after assembly ([Groves et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.641)). Supergiant sediment-hosted Cu forms during breakup at low latitude ([Hitzman et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.627)).
- **Cooling Earth.** Komatiite Ni is confined to 2.7–1.9 Ga ([Naldrett 2010](https://doi.org/10.2113/gsecongeo.105.3.669)). Buoyant Archean subcontinental lithospheric mantle helps preserve old deposits ([Groves et al. 2005](https://doi.org/10.1144/gsl.sp.2005.248.01.04)).
- **Biology.** Land plants underpin coal and sandstone U ([Cuney 2010](https://doi.org/10.2113/gsecongeo.105.3.553); [Nelsen et al. 2016](https://doi.org/10.1073/pnas.1517943113)). Biota and paleolatitude control petroleum source rocks ([Klemme & Ulmishek 1991](https://doi.org/10.1306/0c9b2a47-1710-11d7-8645000102c1865d)).
- **Deposit types are time-bound.** Secular analysis works as a quick favorability filter by host-rock age. The recorded age distribution reflects uneven preservation, data gaps, plate-motion contingency and long-term Earth change ([Goldfarb, Bradley & Leach 2010](https://doi.org/10.2113/gsecongeo.105.3.459); see also [Meyer 1988](https://doi.org/10.1146/annurev.ea.16.050188.001051)).

### Inferences
- **Minimum sim state for ore placement.** The tables above collapse into roughly 20 fields per cell and time step:
  - tectonic regime (MOR, back-arc, oceanic arc, continental arc, collision/orogen, rift, passive margin, foreland, intracratonic sag, craton interior, plume/LIP);
  - magma flux by type (calc-alkaline arc, alkaline, S-type/reduced, A-type, komatiite/picrite, tholeiite/flood basalt, kimberlite, carbonatite) with a magma oxidation flag (magnetite- vs ilmenite-series);
  - crust thickness and lithosphere (keel) thickness, plus distance to thick–thin lithosphere boundaries;
  - convergence rate, obliquity, slab age and a flat-slab/ridge-subduction flag;
  - heat flow, metamorphic grade and P–T path;
  - fault density and reactivation;
  - basin type and fill stack (carbonate platform, black shale, red beds, sandstone, evaporite);
  - organic carbon;
  - brine flow driver (topographic gradient) and redox of aquifers;
  - global clocks (atmospheric O₂, land plants, seawater chemistry);
  - paleolatitude and climate (temperature, precipitation, aridity), upwelling;
  - sea level;
  - an exhumation/burial ledger (uplift, erosion, water-table depth).
- **Many classes are "event plus context" rules**, so the sim does not need to model ore chemistry:
  - porphyry = arc magma pulse × oxidized × thick or compressive crust;
  - MVT = orogenic uplift beside a carbonate platform with evaporite brines;
  - laterite = exposed ultramafic × humid tropics × low erosion × ≥1 Myr.
- **Nearly every endogenic deposit has a downstream "child":** porphyry → supergene blanket and exotic Cu; tin granite → cassiterite placer; kimberlite → diamond placer; ophiolite → chromite placer and Ni laterite; orogenic gold → gold placer; BIF → hematite ore. A single deposit registry with parent–child links would let erosion generate placers and supergene zones consistently.

### Gaps
- Numeric depth ranges for orogenic gold subtypes (epizonal, mesozonal, hypozonal) could not be verified: the abstracts of [Groves et al. 1998](https://doi.org/10.1016/s0169-1368(97)00012-7) and [Goldfarb et al. 2001](https://doi.org/10.1016/s0169-1368(01)00016-6) were unavailable, and Goldfarb et al. 2005 names the subtypes without giving depths in its abstract.
- Diamond stability depth and temperature were not retrieved as a number; the GIA diamond-geology PDF link returned HTML. The cited facts cover craton age, keel thickness and 99% lithospheric sourcing.
- Details of Warren 2010, Hartley & Rice 2005, Hagemann 2016, Diaz-Rodriguez 2021, Griffin 2013 and Yanites & Kesler 2015 were unavailable (no abstract in OpenAlex or Semantic Scholar, paywalled) and are cited at title level only.
- Not covered by retrieved sources: opal, turquoise, jade/nephrite, lithium brines (only a 2026 USGS map report was found), geothermal and hydrothermal Mn, and seafloor Mn nodules.
- Unresolved conflict on nickel's source split. [Butt & Cluzel 2013](https://doi.org/10.2113/gselements.9.2.123) state laterites supply "over 60% of global nickel supply". [USGS SIR 2010-5070-I (2014)](https://pubs.usgs.gov/publication/sir20105070I) states magmatic sulfides account for "approximately 60 percent of the world's nickel production". The two probably measure different things (supply or resources vs mine production, and different years). This does not affect placement rules.

## Q2. Quantitative sizing: USGS deposit models, grade–tonnage, the three-part method, open datasets, and stochastic placement

### Takeaway
USGS practice gives a ready recipe for stochastic deposits: (1) permissive tracts drawn from geology, (2) a count of deposits per tract from deposit-density models, and (3) tonnage and grades drawn from grade–tonnage distributions, which are roughly lognormal with Pareto tails, combined by Monte Carlo. The data are US public domain. For a game, the recipe maps directly onto "favorability map → Poisson or clustered count → lognormal size and grade → geometry". The one essential adjustment is a scale factor: a 10,000-block world at 1 m per block is only ~100 km², where real densities would give almost no deposits.

### Cited Findings
- **Three-part method.** It has been used since 1975 (27 areas, 5×10⁶ km² by 1993). Step 1 delineates areas by the deposit types geology permits. Step 2 estimates metal and ore characteristics from grade–tonnage models. Step 3 estimates the number of undiscovered deposits. Permissive boundaries are drawn so that the probability of a deposit outside them is below 1 in 100,000 to 1,000,000 ([Singer 1993](https://pubs.usgs.gov/publication/70018261), [DOI](https://doi.org/10.1007/bf02272804)). Monte Carlo simulation combines the pieces ([Root, Menzie & Scott 1992](https://doi.org/10.1007/bf01782266)).
- **Current USGS tooling.**
  - MapMark4 (R) with MapMark4GUI ([USGS TM 7-C18](https://pubs.usgs.gov/publication/tm7C18)); tract aggregation under independence, total dependence or a user correlation matrix ([ATA GUI, TM 7-C21](https://pubs.usgs.gov/publication/tm7C21)).
  - A 2025 modified MapMark4 adds multivariate correlated grade–tonnage simulation and an empirical resampling mode. Previously, tonnage and grades were simulated independently, which gives "geologically unrealistic" results where they correlate ([USGS 2025](https://pubs.usgs.gov/publication/70266112)).
  - A 2026 USGS study compares global "pooled" deposit data with local "tract" data across 16 past assessments ([SIR 2026-5031](https://pubs.usgs.gov/publication/sir20265031)).
- **Descriptive models.** Bulletin 1693 remains the base catalogue (1986; each model lists rock types, age range, depositional environment, tectonic setting, ore controls, weathering, geochemical signature and examples; [B1693](https://pubs.usgs.gov/bul/1693/report.pdf)). Its grade–tonnage plots are figures only; the medians are not in the text layer. Updates run through [SIR 2010-5070 chapters A–R](https://pubs.usgs.gov/publication/sir20105070B) (MVT, porphyry Cu, VMS, porphyry Mo, stratiform chromite, Be, Co-Cu-Au, Ni-Co laterite, Ni-Cu-PGE, REE, Fe-Ti, heavy-mineral sands, sediment-hosted Cu, sedex, LCT pegmatite, QPC Au, epithermal, alkalic epithermal).
- **Deposit density.** Porphyry Cu densities range from 2 to 128 deposits per 100,000 km² of exposed permissive rock in 19 well-explored control areas. 90% of control areas hold ≥4, 50% hold ≥15, and 10% hold ≥35 per 100,000 km². Density is not related to age or depth but is inversely related to permissive area. Deposits closer than 2 km are merged into one. For a 76,000 km² Antarctic tract the regression predicts 90% / 50% / 10% chances of at least 4 / 11 / 34 undiscovered deposits ([Singer, Berger & Menzie 2005](https://doi.org/10.2113/gsecongeo.100.3.491)). A general density treatment is in [Singer 2008 (title-level)](https://doi.org/10.1007/s11004-007-9127-3). A Cuban arc check gave a three-part estimate of 3.2 deposits against 3.9 from the density model ([USGS Sierra Maestra](https://pubs.usgs.gov/publication/70212544)).
- **Size-distribution shape.**
  - Most deposits of a type follow a lognormal distribution, while the largest follow a Pareto (hyperbolic) tail ([Agterberg 1995](https://doi.org/10.1080/00206819509465388)).
  - Deposits above the median size hold over 96% of each metal, and the largest 10% hold 47–79%. "World-class" deposits (top 10% by contained metal) hold >86% of gold and similar shares of Ag, Cu, Zn and Pb. They contain at least 100 t Au, 2,400 t Ag, 2 Mt Cu, 1.7 Mt Zn or 1 Mt Pb ([Singer 1995](https://doi.org/10.2113/gsecongeo.90.1.88)).
  - Orogenic-gold camps are spaced every 30–50 km along belts, with Pareto size distributions ([Robert et al. 2005](https://doi.org/10.5382/av100.30)).
  - Deposit sizes within provinces follow power laws ([Hronsky & Groves 2008](https://doi.org/10.1080/08120090701581356)).
- **Grade–tonnage values retrieved.** USGS convention: "90th percentile" means 90% of deposits are at least that large.

| Type | Tonnage P90 / P50 / P10 | Grades (P50, with other percentiles where given) | n | Source |
|---|---|---|---|---|
| Porphyry Cu (all) | 33 / 240 / 1,500 Mt (figure intercepts) | Cu 0.24 / 0.44 / 0.80%. Au grade correlates with tonnage (r = −0.28); Cu does not (r = 0.1, not significant). | 422 | [USGS OFR 2008-1155](https://pubs.usgs.gov/of/2008/1155/) |
| VMS, felsic | 0.15 / 3 / 36 Mt | Cu 1.2%, Zn 3.2%, Pb 0.42%, Au 0.4 g/t, Ag 25 g/t. Median age 390 Ma. Median deposit area 0.02 km². | 421 | [USGS OFR 2009-1034](https://pubs.usgs.gov/of/2009/1034/) |
| VMS, bimodal-mafic | 0.14 / 1.9 / 31 Mt | Cu 1.4%, Zn 1.7%, Au 0.24 g/t | 272 | [USGS OFR 2009-1034](https://pubs.usgs.gov/of/2009/1034/) |
| Ni-Co laterite | 9.0 / 56 / 290 Mt | Ni 0.83 / 1.3 / 1.8%; Co 0.00 / 0.04 / 0.11%. Tonnage correlates with deposit area. | 117 | [USGS OFR 2011-1058](https://pubs.usgs.gov/of/2011/1058/) |
| Podiform chromite (major / minor / banded) | Medians 11,000 / 100 / 650 t | Mean Cr₂O₃ 45 / 43 / 42%. Deposits separated by a 100 m rule. | 619 | [USGS SIR 2012-5157](https://pubs.usgs.gov/publication/sir20125157) |
| Diamond kimberlite pipes | Median 26 Mt | 0.25 ct/t; median stone 0.07 ct; 67% industrial. P(pipe is workable) ≈ 0.005. log₁₀ tonnes = 6.5 + log₁₀ outcrop ha (median outcrop 12 ha). log₁₀ largest diamond (ct) = −1.5 + 0.54 log₁₀ tonnes. | — | [USGS 1992](https://pubs.usgs.gov/publication/70016991) |
| Li pegmatites | Median 21.2 Mt (29.8 Mt for Archean hosts, 6.5 Mt for Paleo–Mesoproterozoic hosts) | 1.12% Li₂O. Cs and Rb median tonnage 7.6 and 6.3 Mt. | 73 | [USGS 2026](https://pubs.usgs.gov/publication/70276446) |
| Granitic tin (bedrock) | Average 58,000 t contained Sn; top 5 deposits = 46% of resource | — | 301 | [Kesler & Wilkinson 2013](https://doi.org/10.1144/sp393.3) |
| Epithermal Au-Ag | 10s of kt to >1 Gt | Au 0.1 to >30 g/t; Ag <1 to 1000s g/t | ~100 | [USGS SIR 2010-5070-Q](https://pubs.usgs.gov/publication/sir20105070Q) |
| Arc porphyry Mo | Large tonnage | 0.03–0.22% Mo | — | [SIR 2010-5070-D](https://pubs.usgs.gov/publication/sir20105070D) |
| Fe-Ti (anorthosite) | Rarely >300 Mt | 10–45% TiO₂ | — | [SIR 2010-5070-K](https://pubs.usgs.gov/publication/sir20105070K) |
| Sediment-hosted Cu (supergiant) | >24 Mt contained Cu | — | — | [Hitzman et al. 2005](https://doi.org/10.5382/av100.19) |
| Sedex basins | >10 Mt Zn+Pb per basin | — | — | [SIR 2010-5070-N](https://pubs.usgs.gov/publication/sir20105070N) |
| Potash basins | 10s of Mt to >100 Gt K₂O | Assessment tract criteria: ≥1 m thick, ≥4% K₂O, ≤3 km depth | >900 sites | [SIR 2010-5090-S](https://pubs.usgs.gov/publication/sir20105090S); [SIR 2010-5090-CC](https://pubs.usgs.gov/publication/sir20105090CC) |

- **Spatial "same deposit" rules** usable as minimum separations: 2 km for porphyry Cu ([Singer et al. 2005](https://doi.org/10.2113/gsecongeo.100.3.491)) and sediment-hosted Cu ([USGS OFR 2003-107](https://pubs.usgs.gov/publication/ofr2003107)); 100 m for podiform chromite ([SIR 2012-5157](https://pubs.usgs.gov/publication/sir20125157)); 500 m for Co-Cu-Au ([SIR 2010-5070-G](https://pubs.usgs.gov/publication/sir20105070G)).
- **Area–tonnage relations.** Deposit surface area and tonnage are positively related in all 10 deposit types analysed, and larger deposits are discovered earlier ([USGS 2021](https://pubs.usgs.gov/publication/70229519)). Kimberlite tonnage ∝ outcrop area ([USGS 1992](https://pubs.usgs.gov/publication/70016991)). Laterite tonnage correlates with area ([OFR 2011-1058](https://pubs.usgs.gov/of/2011/1058/)).
- **Global-scale anchors.**
  - Tectonic-diffusion modelling estimates ~125,895 porphyry Cu deposits formed in the Phanerozoic, ~47,789 remain in the crust (1.7×10¹¹ t Cu), and about two-thirds were recycled by uplift and erosion ([Kesler & Wilkinson 2008](https://doi.org/10.1130/g24317a.1)).
  - About 14,520 granitic tin deposits remain in the crust ([Kesler & Wilkinson 2013](https://doi.org/10.1144/sp393.3)).
  - Mean undiscovered Cu is ≥3,500 Mt worldwide (porphyry ≥3,100 Mt, sediment-hosted ≥420 Mt) against 2,100 Mt identified, across 236 tracts in 11 regions ([USGS SIR 2018-5160](https://pubs.usgs.gov/publication/sir20185160)).
- **Datasets and licences** (sub-agent verification, plus coordinator corrections checked here):
  - USGS MRDS is US public domain and has not been systematically updated since 2011; its record count is unverified ([mrdata.usgs.gov/mrds](https://mrdata.usgs.gov/mrds)). USMIN reached v10.0 in 2023 ([mrdata.usgs.gov/usmin](https://mrdata.usgs.gov/usmin)).
  - The USGS grade–tonnage databases (OFR 2008-1155, 2009-1034, 2009-1252, 2011-1058, 2003-107; SIR 2012-5157) are US public domain, distributed as FileMaker/Excel/text ([OFR 2009-1034](https://pubs.usgs.gov/of/2009/1034/), [OFR 2009-1252](https://pubs.usgs.gov/of/2009/1252/)).
  - Live USGS grade–tonnage model pages exist for sediment-hosted Cu, carbonatite Nb/REE, podiform chromite and sediment-hosted Au, all public domain ([sedcu](https://mrdata.usgs.gov/sedcu/), [carbonatite](https://mrdata.usgs.gov/carbonatite/), [podchrome](https://mrdata.usgs.gov/podchrome/), [sedau](https://mrdata.usgs.gov/sedau/); all returned HTTP 200 here).
  - MapMark4 (the R package that runs three-part assessments) is mirrored on GitHub with `License: CC0` in its DESCRIPTION file ([USGS-R/MapMark4](https://github.com/USGS-R/MapMark4)).
  - EMINERS v3.0 (OFR 2004-1344) is a C++ program built on Microsoft Visual C++ 6.0 and MFC. It "works only on Microsoft Windows XP or newer". The page describes no batch or scripting interface, which settles the conflicting sub-agent reports in favour of "Windows GUI program" ([USGS OFR 2004-1344](https://pubs.usgs.gov/of/2004/1344/)).
  - Macrostrat (rock units, ages, lithology; 33,903 units, 2.5M map polygons) is CC BY 4.0 ([macrostrat.org](https://macrostrat.org)).
  - Geoscience Australia's default licence is CC BY 4.0; OZMIN's current URL and status are unverified ([GA copyright](https://www.ga.gov.au/copyright)).
  - The USGS 2017 critical-minerals global dataset is on ScienceBase ([item](https://www.sciencebase.gov/catalog/item/594d3c8ee4b062508e39b332)), as are the USGS/GA/GSC Critical Minerals Mapping Initiative data ([CMMI item](https://www.sciencebase.gov/catalog/item/686ecb93d4be020e5c0a6c07)).
  - Mudd & Jowitt's global Cu dataset is [Mudd & Jowitt 2018, "Growing Global Copper Resources, Reserves and Production", Econ. Geol. 113(6):1235–1267](https://doi.org/10.5382/econgeo.2018.4590) (checked on Crossref; it replaces an earlier 2013 citation). Their global Ni data are in [Econ. Geol. 2014](https://doi.org/10.2113/econgeo.109.7.1813). Both are journal-published, with no blanket open licence.
  - GTK, BGS, GEOROC and OZMIN licences are unverified.

### Inferences
- **Recommended placement pipeline** (per deposit type T), mirroring the three-part method:
  1. The rule engine (Q3) produces a time-integrated favorability map F_T(x) in [0,1]. Permissive tracts are the connected regions where F_T exceeds a threshold, which keeps the USGS "~zero probability outside" property.
  2. Count: N ~ Poisson(ρ_T · A_tract · mean(F_T) · k_world), where ρ_T comes from density models (porphyry P50 ≈ 15 per 100,000 km²) and k_world is the world-compression factor.
  3. Positions: a clustered point process (Neyman-Scott/Thomas) with parent camps spaced to match observed camp spacing (30–50 km for orogenic gold, scaled), a minimum separation equal to the USGS same-deposit rule, and sampling weighted by F_T.
  4. Tonnage: a lognormal fitted to P90/P50/P10, with μ = ln P50 and σ = (ln P10 − ln P90)/2.563. Porphyry Cu gives σ ≈ 1.49. A Pareto tail above P10 is optional.
  5. Grades: correlated lognormals via a Gaussian copula, using published correlations (porphyry Au vs tonnage r = −0.28).
  6. Footprint: from the area–tonnage relations.
  7. Realize geometry (Q7), then apply preservation (Q5).
- **Scale mismatch is the central balance problem.** At 1 m per block, a 10,000 × 10,000 world is 100 km², so real porphyry density gives ~0.015 deposits. A median porphyry (240 Mt at ~2.6 t/m³ ≈ 9×10⁷ m³, roughly a 450 m cube) would dwarf the playable vertical range. The mod therefore needs separate multipliers for count per area (×10³–10⁴) and size (×10⁻²–10⁻⁴), applied per type so relative rarity and size ranks survive. Keeping the lognormal shape but compressing σ gives "mostly ordinary, occasionally huge" deposits without world-eating giants.
- **Heavy tails are gameplay-relevant.** If the top 10% of deposits hold most of the metal, a world naturally gets a few "legendary" deposits worth an expedition. A cap or a guaranteed number of giants per world could control variance.
- Grade–tonnage tables can ship as plain JSON percentiles derived from public-domain USGS data, with attribution kept as good practice even though it is not required.

### Gaps
- Bulletin 1693's per-model medians are only in scanned figures, and a sediment-hosted Zn-Pb (OFR 2009-1252) median could not be parsed reliably. Medians for MVT, sedex, orogenic gold, epithermal subtypes, IOCG, Sn greisen, placers, bauxite and phosphate were not extracted. They exist in the cited USGS reports, or as figures in Bulletin 1693, and could be digitized in a follow-up.
- Deposit-density regressions beyond porphyry Cu (Singer 2008) were not retrieved; the abstract was unavailable.
- MRDS record count and the OZMIN, GTK and BGS licence terms are unverified.

## Q3. Mineral-systems approach, prospectivity tools, and a mineral-system rule engine on simulated fields

### Takeaway
The mineral-systems approach frames a deposit as the coincidence in space and time of (1) metal and fluid sources, (2) an energy driver, (3) lithospheric architecture or pathways, and (4) physico-chemical traps, followed by preservation. It is the natural shape for a sim rule engine: evaluate each component as a fuzzy score from simulated fields at each time step, and combine them multiplicatively. Prospectivity toolkits (weights-of-evidence, fuzzy overlay, ML) can be used offline to calibrate weights on real data. Several published regional forward models give concrete thresholds or process physics worth simplifying.

### Cited Findings
- **Four-component definition.** A mineral system needs four components operating efficiently and coincidentally: (1) sources of metals and fluids, (2) energy sources to drive fluids, (3) active crustal and mantle-lithosphere architecture as pathways, and (4) physico-chemical gradients where metals deposit. Geoscience Australia's knowledge-driven, continent-scale IOCG prospectivity method translates these components into mappable criteria and tests weighting uncertainty by Monte Carlo ([Skirrow et al. 2019](https://doi.org/10.1016/j.oregeorev.2019.103011)).
- **Translation steps.** McCuaig et al. propose four steps: critical processes → constituent processes → targeting elements → mappable targeting criteria. Targets should be ranked at the critical-process level. Criteria are scale-dependent. Uncertainty comes from data resolution, how well a proxy reflects the element, how well the element reflects the process, and whether the process is actually critical ([McCuaig, Beresford & Hronsky 2010](https://doi.org/10.1016/j.oregeorev.2010.05.008)).
- **Multiplicative ranking.** Target ranking is best done multiplicatively, requiring all essential processes. Deposit sizes are power-law distributed within provinces, which supports "Elephant Country" (go where giants are) versus "First Mover" strategies. Hierarchical (progressive area reduction) and Venn-diagram (GIS overlay) approaches both exist ([Hronsky & Groves 2008](https://doi.org/10.1080/08120090701581356)).
- **Empirical vs conceptual prospectivity.** Empirical (data-driven) methods relate known deposits to surrounding features; conceptual (knowledge-driven) methods suit areas with few known deposits ([Knox-Robinson & Wyborn 1997](https://doi.org/10.1080/08120099708728326)). Weights-of-evidence is the classic data-driven method ([Bonham-Carter, Agterberg & Wright 1990](https://doi.org/10.4095/128059)).
- **Further framing papers:** [Hagemann, Lisitsin & Huston 2016 (title-level)](https://doi.org/10.1016/j.oregeorev.2015.12.012); [Huston et al. 2016, "Tectono-metallogenic systems" (title-level)](https://doi.org/10.1016/j.oregeorev.2015.09.005).
- **Software** (sub-agent, licences checked via GitHub API):
  - EIS Toolkit is a Python library for prospectivity mapping, EUPL-1.2, active 2025 ([GitHub](https://github.com/GispoCoding/eis_toolkit)). The EIS QGIS plugin (v1.0.6, "comprehensive mineral prospectivity mapping and analysis framework") has no LICENSE file at the repo root, but its README states "Licensed under GPL 2 or later" with a GPL v2 badge. That reconciles the two conflicting sub-agent checks: GPL-2.0-or-later, declared in the README only ([README](https://github.com/GispoCoding/eis_qgis_plugin)).
  - ArcSDM (GTK fork of the USGS/GSC tool; weights-of-evidence, logistic regression, fuzzy overlay plus ML classifiers) needs ArcGIS Pro, has no licence file, and was active 2026 ([GitHub](https://github.com/gtkfi/ArcSDM)).
- **Forward and regional models to simplify:**
  - **Porphyry magmatism from plate kinematics.** Random-forest and SVM classifiers on reconstructed South American subduction found favorable conditions at ~100 km/Myr convergence, ~15° obliquity, subducting-plate age ~25–70 Myr, and a location >2,000 km from the subducting trench boundary ([Butterworth et al. 2016](https://doi.org/10.1002/2016tc004289)). Follow-up: [Diaz-Rodriguez, Müller & Chandra 2021 (title-level)](https://doi.org/10.1016/j.oregeorev.2021.104300).
  - **Porphyry triggers and fertility.** Giant porphyries relate to ridge, seamount or plateau subduction, flat slabs, crustal thickening, uplift and erosion ([Cooke et al. 2005](https://doi.org/10.2113/gsecongeo.100.5.801)). They need hot, hydrous, oxidized, S-rich arc magmas, with ascent controlled by crustal stress ([Richards 2003](https://doi.org/10.2113/gsecongeo.98.8.1515)). Flat-slab subduction builds a Cu-Au inventory in the main arc; later back-arc crustal melting of reduced crust produces Sn-W ([Lehmann 2020](https://doi.org/10.1016/j.lithos.2020.105756)). Numerical models show porphyry ore shells stabilize at the boundary between lithostatic magmatic up-flow and hydrostatic meteoric convection ([Weis, Driesner & Heinrich 2012](https://doi.org/10.1126/science.1225009)).
  - **Lithospheric architecture.** 85% of sediment-hosted base metals lie within 200 km of thick–thin lithosphere transitions; rifting there gives more subsidence and lower basal heat flow ([Hoggard et al. 2020](https://doi.org/10.1038/s41561-020-0593-2)). Giant IOCGs sit within ~100 km of craton margins ([Groves et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.641)). Magmatic ores show continental-root control ([Griffin et al. 2013, title-level](https://doi.org/10.1038/ngeo1954)).
  - **Basin hydrology (MVT).** Numerical flow, heat and solute models of topography-driven brines reproduce Pine Point (1–5 m/yr, 60–100 °C, 0.5–5 Myr) ([Garven 1985](https://doi.org/10.2113/gsecongeo.80.2.307)). Compaction-driven flow is too slow and cools before reaching the district; uplift-driven flow works ([Bethke 1986](https://doi.org/10.2113/gsecongeo.81.2.233)). In Southeast Missouri, uplift-driven flow during the Ouachita orogeny gave only a short (100s of kyr) ore window before meteoric freshening ([Appold & Garven 1999](https://doi.org/10.2113/gsecongeo.94.6.913)).
  - **Other fluid drivers.** Upper-crustal flow is driven by precipitation, sediment deposition, compaction, petroleum generation, evaporative brines and internal heat ([Cathles & Adams 2005](https://doi.org/10.5382/av100.05)). At depth, ongoing deformation regenerates permeability and metamorphic devolatilization produces fluid ([S.F. Cox 2005](https://doi.org/10.5382/av100.04)).
  - **Ni-Cu-PGE recipe:** a plume (large magma volume) plus a rift (easy access) plus S-rich sediments ([Barnes & Lightfoot 2005](https://doi.org/10.5382/av100.08)).
  - **Sediment-hosted Cu:** time available (10s–100s of Myr in closed basins) may be the key to supergiants ([Hitzman et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.627)).
  - **Limits of full physics.** Rigorous coupled thermal-hydraulic-mechanical-chemical modelling of ore systems still challenges computing ([Ingebritsen & Appold 2012](https://doi.org/10.2113/econgeo.107.4.559)).
  - **Exhumation as a statistical forward model.** Deposits diffuse vertically through the crust by burial, uplift and erosion; this explains age-frequency distributions and yields endowment estimates ([Kesler & Wilkinson 2006](https://doi.org/10.2113/gsecongeo.101.5.919), [2008](https://doi.org/10.1130/g24317a.1)).

### Inferences
- **Rule engine design.** Each deposit type becomes a mineral system with five scored components: Source, Driver, Pathway, Trap and Preservation.
  - Each component is a membership function (piecewise-linear curve) over sim fields. Examples: convergence rate peaking near 100 km/Myr; slab age in 25–70 Myr; distance to lithosphere step under 200 km.
  - Components combine by product or fuzzy minimum, so any missing critical process zeroes the score (the multiplicative logic). Fuzzy "OR" (maximum) handles alternatives within one component.
  - Scores are integrated over time within the type's formation window and gated by global clocks (post-GOE, land plants).
  - Each step should record which component limited the score, so the visualizer can show why an area is barren.
- **Calibration offline.** Run a weights-of-evidence or logistic fit (EIS Toolkit, Python) on real Earth layers against real deposit locations (MRDS, USGS grade–tonnage databases) to tune the membership curves, then ship only the fitted curves as data. EUPL copyleft applies to toolkit code, not to derived parameters; this is a licence interpretation to confirm.
- **Event-driven types** (MVT flushes, kimberlite eruptions, VMS pulses, supergene cycles) fit better as discrete events emitted by the tectonic and surface engines than as continuous scores: "orogenic uplift began beside a carbonate platform with evaporite brines" becomes one MVT district event with a short duration.
- A "why here" explanation per deposit (component scores, time of formation, erosion since) doubles as in-game lore for prospecting tools.

### Gaps
- Wyborn, Heinrich & Jaques (1994) could not be retrieved; the source–pathway–trap framing above rests on Skirrow et al. 2019, McCuaig et al. 2010 and Knox-Robinson & Wyborn 1997.
- The Diaz-Rodriguez 2021 method details, the Hagemann 2016 content and the ArcSDM method list were not verified.
- No published "whole-planet forward model of ore formation from simulated tectonics" was found; the sub-agent's games survey also found no shipped game doing this.

## Q4. Thermodynamic and petrologic tools: offline lookup-table generators or overkill?

### Takeaway
They are overkill as runtime components and marginal even offline for ore placement: deposit rules and USGS models already encode what matters at game fidelity. They are useful for a few offline tables: metamorphic facies and mineral assemblage versus P–T (for host-rock "skins" and assemblage labels), and perhaps melt fraction or magma-type classification. Most are Fortran, C or Python command-line tools with academic or copyleft terms; none ships a Java library. Petroleum and coal maturity is the exception: simple kinetic or time-temperature indices (TTI, Easy%Ro) are cheap enough to run inside the sim.

### Cited Findings
Licence and status details come from sub-agent checks; runtime per point was unverified for every tool.


| Tool | What / batch capability | Licence (as verified) | JVM embedding | Source |
|---|---|---|---|---|
| Perple_X (Connolly, ETH) | Phase equilibria and pseudosections; Fortran CLI; batch-capable | No formal licence found; free download with source | Subprocess or offline only | [perplex.ethz.ch](https://www.perplex.ethz.ch) |
| THERMOCALC (Powell & Holland) | Phase-equilibria CLI | "Freely available for academic research" (site wording) | CLI only | [hpxeosandthermocalc.org](https://hpxeosandthermocalc.org) |
| MELTS family (rhyolite-MELTS, pMELTS, alphaMELTS; PetThermoTools) | Magma crystallization; alphaMELTS is "a scripted version of MELTS" | alphaMELTS 2 is AGPL-3.0 (LICENSE file checked). The separate PetThermoTools mirror shows NOASSERTION. | Subprocess only | [melts.ofm-research.org](https://melts.ofm-research.org); [magmasource/alphaMELTS](https://github.com/magmasource/alphaMELTS) |
| ENKI ThermoEngine | Python/C thermodynamic engine | Unverified (no canonical licensed repo found) | Unconfirmed | (sub-agent) |
| BurnMan | Python mantle-mineral physics | GPL-2.0 | No | [GitHub](https://github.com/geodynamics/burnman) |
| MAGEMin (Riel et al. 2022) | Fast Gibbs-energy minimization built for batch runs; C core, Julia wrapper | GPL-3.0 | C core, JNI-plausible | [GitHub](https://github.com/ComputationalThermodynamics/MAGEMin) |
| Theriak-Domino | Phase diagrams, Fortran CLI | Free for non-commercial use (site wording) | CLI only | [titan.minpet.unibas.ch](https://titan.minpet.unibas.ch/minpet/theriak) |
| PHREEQC | Aqueous and fluid–rock geochemistry; explicit batch build; IPhreeqc library | US public domain | Most plausible JNI target (C API) | [USGS PHREEQC](https://www.usgs.gov/software/phreeqc-version-3) |
| Reaktoro | Chemical equilibrium and kinetics, C++/Python | LGPL-2.1 | C++ core, JNI-plausible | [GitHub](https://github.com/reaktoro/reaktoro) |
| SUPCRTBL | Aqueous thermodynamic database; SupPhreeqc converts it to PHREEQC format | Public repo with no LICENSE file, so all rights reserved (coordinator check; raw LICENSE probes on main/master also returned 404 here) | — | [Zimmer et al. 2016](https://doi.org/10.1016/j.cageo.2016.02.013); [HydrogeoIU/SUPCRTBL](https://github.com/HydrogeoIU/SUPCRTBL) |
| PyBasin (Luijendijk) | 1D burial and thermal history calibrated to vitrinite reflectance and thermochronology | LGPL-3.0 | Python | [GitHub](https://github.com/ElcoLuijendijk/pybasin) |

- **Lopatin TTI.** TTI = Σ Δtₙ·2ⁿ, with the reaction rate doubling per 10 °C interval. Oil generation begins at TTI ≈ 15 and peaks at TTI ≈ 75 ([Waples 1980](https://doi.org/10.1306/2f9193d2-16ce-11d7-8645000102c1865d)).
- **Easy%Ro.** Parallel first-order reactions with distributed activation energies, integrated along T(t) and mapped to vitrinite reflectance ([Sweeney & Burnham 1990](https://doi.org/10.1306/0c9b251f-1710-11d7-8645000102c1865d)); the parameter table was not re-extracted.
- **Catagenesis** spans ~50–150 °C ([SLB glossary](https://glossary.slb.com/en/terms/c/catagenesis)).
- **Commercial basin tools.** Trinity is proprietary (ZetaWare; sub-agent). BasinMod (with BasinMod Risk and BasinMod 2-D) is proprietary software of Platte River Associates, Boulder, CO, listed on its product site ([platte.com](https://www.platte.com/)); this corrects a sub-agent note that attributed it to Ikon Science/SLB. PetroMod is SLB's (coordinator-resolved; no URL retrieved here).
- **Why full physics is out of reach:** coupled thermal-hydraulic-mechanical-chemical ore modelling remains computationally challenging ([Ingebritsen & Appold 2012](https://doi.org/10.2113/econgeo.107.4.559)).

### Inferences
- **Worth doing offline:**
  - A facies/assemblage grid (P 0–1.5 GPa × T 200–900 °C for a handful of bulk compositions: pelite, basalt, granite, peridotite, carbonate) from MAGEMin or Perple_X, shipped as a small table. Use it to label metamorphic grade (greenschist/amphibolite, used by orogenic-gold and pegmatite rules) and to choose host-rock blocks (slate→schist→gneiss, marble, serpentinite).
  - Optionally, a few PHREEQC solubility-versus-temperature curves to justify trap-efficiency curves.
- **Run in-sim instead:** TTI (one exponential per layer per step) or an Easy%Ro-style kinetic index for every sedimentary layer. This gives oil and gas windows, coal rank and the petroleum "critical moment" cheaply.
- **Avoid as dependencies:** GPL-licensed code bundled into a public mod (BurnMan, MAGEMin) raises licence-compatibility questions. Tables generated offline avoid bundling, though whether outputs are unencumbered is a licence interpretation to confirm per tool.

### Gaps
- No verified per-point runtimes.
- ThermoEngine and GEMS licence terms are unverified. SUPCRTBL has no LICENSE file (all rights reserved).
- No published example was found of a game or procedural generator using these tools for worldgen.

## Q5. Preservation and exposure: how uplift and erosion history decides what survives and what is exposed

### Takeaway
Preservation is as important as formation. The observed age distribution of shallow deposits (epithermal, porphyry) is set largely by exhumation. Two-thirds of Phanerozoic porphyry Cu has already been eroded, and erosion converts primary deposits into placers and supergene blankets. A per-deposit burial/exhumation ledger, compared against each type's formation depth, reproduces the classic patterns: epithermal-over-porphyry "telescoping" by erosion level, supergene blankets where uplift is moderate and the climate right, placers downstream of eroded sources, and old deposits surviving mainly on buoyant cratons.

### Cited Findings
- **Exhumation sets age distributions.** The temporal distribution of ore deposits is controlled largely by exhumation (burial, uplift, erosion). Rates derived from modal ages and emplacement depths are 167 m/Myr for epithermal, 158 m/Myr for porphyry Cu and 63 m/Myr for orogenic gold deposits. Scarce Precambrian epithermal and porphyry deposits were probably removed; Archean and Paleoproterozoic orogenic gold is more abundant than exhumation alone predicts ([Kesler & Wilkinson 2006](https://doi.org/10.2113/gsecongeo.101.5.919)).
- **Tectonic diffusion.** Of ~125,895 Phanerozoic porphyries, only ~47,789 remain at various crustal depths; about two-thirds of deposited Cu was recycled by uplift and erosion; deposits above 3.3 km are within reach of mining ([Kesler & Wilkinson 2008](https://doi.org/10.1130/g24317a.1)). Tin deposits (average formation depth ~2 km) were treated the same way ([Kesler & Wilkinson 2013](https://doi.org/10.1144/sp393.3)). Climate imprints on porphyry exhumation patterns ([Yanites & Kesler 2015, title-level](https://doi.org/10.1038/ngeo2429)).
- **Vertical telescoping of porphyry systems.** Lithocaps can exceed 1 km in thickness if not eroded. High-sulfidation disseminated Au sits in the uppermost ~500 m, above porphyry ore, above 5–15 km plutons ([Sillitoe 2010](https://doi.org/10.2113/gsecongeo.105.1.3)). Porphyry Cu needs "uplift and erosion to expose subvolcanic rocks" ([B1693 m17](https://pubs.usgs.gov/bul/1693/report.pdf)). Most porphyries are Phanerozoic, typically Cenozoic, which reflects preservation in young rocks ([Seedorff et al. 2005](https://doi.org/10.5382/av100.10)). Most epithermal deposits are Cenozoic because shallow deposits are lost in unstable regions ([USGS SIR 2010-5070-Q](https://pubs.usgs.gov/publication/sir20105070Q)).
- **Supergene windows.** Uplift must lower water tables. Activity must last ≥0.5 Myr (typically 3–9 Myr). Erosion must not outpace supergene processes. A natural cycle exposes deeper potassic zones to in-situ oxidation without enrichment. Laterally flowing Cu solutions form exotic deposits in piedmont gravels ([Sillitoe 2005](https://doi.org/10.5382/av100.22)). Formation and preservation of supergene blankets matter for high-grade giants ([Cooke et al. 2005](https://doi.org/10.2113/gsecongeo.100.5.801)).
- **Laterites and bauxite.** They need low physical erosion plus uplift to expose the protolith ([B1693 m38a](https://pubs.usgs.gov/bul/1693/report.pdf)), or stable land protected from erosion ([B1693 m38b, m38c](https://pubs.usgs.gov/bul/1693/report.pdf)). A laterite profile can form in ~1 Myr ([SIR 2010-5070-H](https://pubs.usgs.gov/publication/sir20105070H)). Most laterite tonnage has Miocene–present weathering ages ([OFR 2011-1058](https://pubs.usgs.gov/of/2011/1058/)). The richest Ni formed where uplift let Ni leach down into saprolite ([Butt & Cluzel 2013](https://doi.org/10.2113/gselements.9.2.123)).
- **Placers from eroded sources.** Placers are Cenozoic-dominated, formed by tectonics plus climate and sea level ([Garnett & Bassett 2005](https://doi.org/10.5382/av100.25)). Older Au placers are rarely preserved ([B1693 m39a](https://pubs.usgs.gov/bul/1693/report.pdf)). Cassiterite placers lie within a few km of their source ([Lehmann 2020](https://doi.org/10.1016/j.lithos.2020.105756)); diamonds travel up to 1,000 km ([B1693 m39d](https://pubs.usgs.gov/bul/1693/report.pdf)). Heavy-mineral sands depend on sea-level transgression and regression ([SIR 2010-5070-L](https://pubs.usgs.gov/publication/sir20105070L)). Witwatersrand ores needed "later moderate uplift and erosion to remove Phanerozoic strata and retain Early Proterozoic rocks" ([B1693 m29a](https://pubs.usgs.gov/bul/1693/report.pdf)).
- **Cratons as preservers.** Formation and preservation became progressively decoupled as buoyant Archean subcontinental lithospheric mantle gave way to thinner, denser Phanerozoic lithospheric mantle ([Groves et al. 2005](https://doi.org/10.1144/gsl.sp.2005.248.01.04)). Olympic Dam–type near-surface IOCGs survived above buoyant, refractory lithosphere that resisted delamination and uplift ([Groves et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.641)).
- **Other preservation effects.**
  - Li and Ta pegmatite resources cluster in Archean and Phanerozoic rocks, interpreted as orogenic-belt preservation like orogenic gold ([McCauley & Bradley 2014](https://doi.org/10.3749/canmin.52.2.183)).
  - Near-surface U types (high-level hydrothermal, calcrete) are known only from the last period because of low preservation potential ([Cuney 2010](https://doi.org/10.2113/gsecongeo.105.3.553)).
  - Most sediment-hosted Pb-Zn has probably been destroyed by recycling ([Leach et al. 2010](https://doi.org/10.2113/gsecongeo.105.3.593)).
  - Sulfide oxidation above the water table is bacterially mediated; hyperarid climates freeze supergene profiles, marked by hydroxychloride minerals ([Sillitoe 2005](https://doi.org/10.5382/av100.22)).

### Inferences
- **Exhumation ledger per deposit.** Store formation depth z_f, formation time t_f and a vertical extent (top, bottom). Accumulate burial B(t) and erosion E(t) from the surface-process engine at the deposit's column. Present depth of top = z_f + B − E. Then:
  - (a) E > z_f + vertical extent: the deposit is fully eroded. Spawn placer children downstream, weighted by resistant minerals (Au, PGE, cassiterite, diamond, chromite, corundum, zircon/ilmenite/rutile) and transport length (Sn a few km, diamond up to 1,000 km).
  - (b) Partly eroded: truncate the geometry. A porphyry column shows lithocap/epithermal at shallow erosion, porphyry ore at moderate erosion and barren pluton roots at deep erosion.
  - (c) Top within a few hundred m of the surface, with uplift lowering the water table, an eligible climate and ≥0.5–9 Myr of stability: build a supergene profile (leached cap, oxide zone, enrichment blanket at 2–3× grade).
  - (d) Buried beyond a depth cutoff (the USGS uses 1–3 km for assessments): the deposit exists but is inaccessible at world scale unless shallow.
- **Order-of-magnitude check.** At 150–170 m/Myr, a 1 km-deep epithermal system is removed in roughly 6–7 Myr. Epithermal and supergene deposits in the game should cluster in young, active belts, while old cratons should expose deep-level types (orogenic gold, pegmatites, layered intrusions, BIF, kimberlite roots).
- **Minecraft vertical mapping.** The real formation depth window must be mapped onto a compressed Y range. The vertical ordering (placers and laterites at the surface; supergene and epithermal shallow; porphyry, skarn and VMS mid-depth; orogenic gold, pegmatite and layered intrusion deeper) should be preserved, rather than absolute depths.

### Gaps
- Yanites & Kesler 2015 (the climate–exhumation link) and Hartley & Rice 2005 (Andean supergene climate controls) could only be cited at title level.
- No quantitative rules were found for placer concentration efficiency per unit of eroded source; they would need Garnett & Bassett's full text or fluvial sorting literature.

## Q6. How games and Minecraft mods model ore geology (data schemas) and what players liked or disliked

### Takeaway
Existing systems fall into three families:
1. **Host-rock-aware procedural veins** (Dwarf Fortress, TerraFirmaCraft, Vintage Story, Geolosys, GregTech). Ore is tied to rock type or layer, found with prospecting tools and surface indicators; TFC pre-bakes one ore block per ore × rock × grade.
2. **Abstract "resource fields" for automated extraction** (IE mineral veins, Immersive Petroleum polygon reservoirs, Create Ore Excavation grid veins, GT bedrock veins, Factorio patches, Satisfactory nodes).
3. **Vanilla-style scatter** plus noise-driven large veins.

None derives ore from a simulated geologic history. The most common complaint across mods is "I searched for hours and found nothing". Players respond with map overlays, vein finders, cheat tools and requests for infinite-vein toggles. Persisted ore metadata has caused performance problems.

### Cited Findings

**Minecraft mod schemas** (sub-agent, verified from source files at the branches noted)

| System (version checked) | Definition format & key fields | Placement algorithm | Discovery tools | Source |
|---|---|---|---|---|
| TerraFirmaCraft (1.21.x branch; a 1.20.x branch also exists) | Vein configured features `tfc:cluster_vein` (+`size`), `tfc:disc_vein` (+`size`,`height`), `tfc:pipe_vein` (+`height`,`radius`,`min/max_skew`,`min/max_slant`,`sign`), `tfc:kaolin_disc_vein` (fields unverified). Shared `VeinConfig`: `blocks` (replace host rock → weighted `with` list), `indicator` {`depth`,`rarity`,`underground_rarity`,`underground_count`,`blocks`}, `rarity`, `density`, `min_y`, `max_y`, `project`, `project_offset`, `random_name`, `near_lava`. | Standard features. `rarity` is an inverse per-chunk chance; `density` is a per-block keep chance; only listed host rocks are replaced. The rock category enum (igneous extrusive/intrusive, metamorphic, sedimentary) covers 20 rocks. `RockLayerSettings` is a probabilistic tree keyed by terrain type (ocean_floor/land/volcanic/uplift) and layer index. | Prospector's pick: radius 12, cooldown 10 ticks. False-negative chance falls from 0.30 (tool tier 0) to 0 (tier 5). Counts in a 25³ cube are bucketed TRACES <10 / SMALL <20 / MEDIUM <40 / LARGE <80 / VERY_LARGE ≥80. | [VeinConfig.java](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/world/feature/vein/VeinConfig.java); [Indicator.java](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/world/feature/vein/Indicator.java); [RockLayerSettings.java](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/world/settings/RockLayerSettings.java); [PropickItem.java](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/common/items/PropickItem.java) |
| TFC ore blocks & grades | One block per ore × rock. 12 graded ores (native Cu, Au, Ag, hematite, cassiterite, bismuthinite, garnierite, malachite, magnetite, limonite, sphalerite, tetrahedrite) come as POOR/NORMAL/RICH blocks, weighted per vein (e.g. 35/40/25 for a normal vein, 15/25/60 for a rich one). Veins are emitted by the Python datagen `resources/world_gen.py`, e.g. a `ruby_marble_belt` disc vein. | — | — | [Ore.java](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/common/blocks/rock/Ore.java); [normal_native_gold.json](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/resources/data/tfc/worldgen/configured_feature/vein/normal_native_gold.json); [world_gen.py](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/resources/world_gen.py) |
| GregTech CEu Modern (1.21 branch; a 1.20.1 branch also exists) | `GTOreDefinition`: `cluster_size` (IntProvider), `density`, `weight`, `layer`, `dimension_filter`, `height_range`, `discard_chance_on_air_exposure`, `biomes`, `weight_modifier`, `generator`, `indicators`. Generators: standard (`targets` block/deep_block/nether_block), layered (`layer_patterns`), veined (vanilla-style `ore_blocks`, `rare_blocks`, `filler_block`, `veininess_threshold` 0.4, richness 0.1–0.3, `rare_block_chance` 0.02), dike, geode, classic, cuboid, noop. | Bypasses vanilla placed features. Veins sit on a grid (`oreVeinGridSize` 3 chunks, `oreVeinRandomOffset` 12 blocks) with weighted type selection and a chunk cache. Vanilla ore generation and large veins are removed by default. | Surface indicator generator (`block`, `radius` 1–32, `density`). Bedrock ore veins (`weight`, `size`, `yield`, `depletion_amount`, `depletion_chance`, `depleted_yield`, `materials`) and bedrock fluid veins; `bedrockOreDistance` 16 chunks. KubeJS `GTOreVeinEvents` add/modify/remove. | [GTOreDefinition.java](https://raw.githubusercontent.com/GregTechCEu/GregTech-Modern/1.21/src/main/java/com/gregtechceu/gtceu/api/data/worldgen/GTOreDefinition.java); [ConfigHolder.java](https://raw.githubusercontent.com/GregTechCEu/GregTech-Modern/1.21/src/main/java/com/gregtechceu/gtceu/config/ConfigHolder.java); [BedrockOreDefinition.java](https://raw.githubusercontent.com/GregTechCEu/GregTech-Modern/1.21/src/main/java/com/gregtechceu/gtceu/api/data/worldgen/bedrockore/BedrockOreDefinition.java) |
| Geolosys (1.21.1 branch, NeoForge 21.1.125) | Deposit JSON `{type, config}`. Types `geolosys:deposit_dense`, `deposit_sparse`, `deposit_dike`, `deposit_top_layer` (plus a `LayerDeposit` class). Fields: `yMin`, `yMax`, `size`/`baseRadius`/`radius`+`depth`, `spread`, `density`, `chanceForSample`, `biomeTag`, `blocks` (a `"default"` list plus per-host-block overrides of `{block, chance}`, where `null` = air), `samples` (surface marker blocks), `blockStateMatchers`, `generationWeight` (JSON key; `genWt` is only the Java name). | Once per chunk (with a chunk-skip chance), `NUMBER_PLUTONS_PER_CHUNK` weighted deposits are placed; blocks spilling into neighbour chunks are queued. | ProPick scans a configurable cuboid ahead of the clicked face, then the chunk column. The repo was archived on 2026-01-19 and moved to git.oitsjustjose.com. | [DepositFeature.java](https://raw.githubusercontent.com/oitsjustjose/Geolosys/1.21.1/src/main/java/com/oitsjustjose/geolosys/common/world/feature/DepositFeature.java); [kimberlite.json](https://raw.githubusercontent.com/oitsjustjose/Geolosys/1.21.1/src/main/resources/data/geolosys/deposits/ore/kimberlite.json); [repo](https://github.com/oitsjustjose/Geolosys) |
| Immersive Engineering (1.21.1) | `immersiveengineering:mineral_mix` recipe: `ores` [{`chance`, `output` tag or id, optional NeoForge `conditions`}], `spoils`, `weight`, `fail_chance`, `biome_predicates` (OR-of-AND biome-tag lists), `sample_background`. Outputs reference common tags, e.g. `{"tag":"c:ores/lead"}`. | Once per chunk: the column of maximum absolute Perlin noise becomes a vein centre with radius 12 + rand(32) blocks, skipped if its circle overlaps an existing vein (as read by the sub-agent; overlapping veins are blended by saturation). The mineral is weighted-picked by biome. Fail chance rises centre→edge (smoothstep). Depletion counter against `mineralVeinYield` (config key unverified). | The core sample lists each intersecting vein's mineral, depletion and saturation. The 2020 rewrite brought positioned radius veins, weighted core samples, admin commands and villager-sold vein maps. | [MineralMixSerializer.java](https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/main/java/blusunrize/immersiveengineering/common/crafting/serializers/MineralMixSerializer.java); [ExcavatorHandler.java](https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/api/java/blusunrize/immersiveengineering/api/excavator/ExcavatorHandler.java); [PR #4052](https://github.com/BluSunrize/ImmersiveEngineering/pull/4052) |
| Immersive Petroleum (TwistedGate fork; 1.20.1 and 1.21.1 builds) | `immersivepetroleum:reservoirs`: `name`, `fluid`, `fluidminimum`, `fluidcapacity` (mB), `fluidtrace` (residual regeneration), `equilibrium`, `weight`, `dimensions`/`biomes` {`isBlacklist`,`list`}. The default oil reservoir holds 2.5–32.5 million mB at weight 40. | A global Perlin field (scale 0.015625) is thresholded at \|n\| > 2/3; qualifying columns are flood-filled and traced into irregular polygon "islands" (point-in-polygon at query time). Pressure falls from 100% to 0% as the amount drops from full to half capacity; flow is 15–2500 mB/t. | Seismic survey block scans 65×65 columns into a map overlay; survey result item. | [ReservoirSerializer.java](https://raw.githubusercontent.com/TwistedGate/ImmersivePetroleum/1.21.1/src/main/java/flaxbeard/immersivepetroleum/common/crafting/serializers/ReservoirSerializer.java); [ReservoirPolygon.java](https://raw.githubusercontent.com/TwistedGate/ImmersivePetroleum/1.21.1/src/main/java/flaxbeard/immersivepetroleum/api/reservoir/ReservoirPolygon.java); [Modrinth API](https://api.modrinth.com/v2/project/immersivepetroleum) |

**Create Ore Excavation 1.6.8 (in the MIC pack; read directly from the jar)**
- Vein definitions are recipes of type `createoreexcavation:vein` (data path `data/createoreexcavation/recipe/ore_vein_type/*.json`). Fields: `name` (JSON text component), `icon` (item stack), `placement` {`spacing`, `separation`, `salt`} (a structure-style random-spread grid), `amountMultiplierMin` and `amountMultiplierMax`, `biomeWhitelist` (biome tag, e.g. `minecraft:is_overworld`), `finite` (`"default"`) and `priority`. Examples: copper spacing 128, separation 8, amount ×10–30; diamond spacing 256, separation 64, amount ×0.5–2 ([COE jar, local: createoreexcavation-1.21-1.6.8.jar](https://github.com/tom5454/Create-Ore-Excavation)).
- Extraction recipes: `createoreexcavation:drilling` (`drill` item or tag, `output` list with optional `chance`, optional `fluid` input {`amount`, `ingredient`}, `stress`, `ticks`, `veinId`, `priority`) and `createoreexcavation:extracting` for fluid veins (e.g. water at 500 mB per 20 ticks). The netherite vein needs lava 1000 mB and 2048 stress, and yields ancient debris at 0.2 chance ([COE jar](https://github.com/tom5454/Create-Ore-Excavation)).
- Server config defaults (read from bytecode): `finiteAmountBase` = 1000; `defaultInfinite` = true; `maxExtractorsPerVein` = 0 (unlimited); vein finder `veinFinderNear` = 1 chunk, `veinFinderFar` accuracy 25, `veinFinderCd` 100 ticks. The config notes more veins can be added with KubeJS ([COE jar](https://github.com/tom5454/Create-Ore-Excavation); [README#kubejs](https://github.com/tom5454/Create-Ore-Excavation#kubejs)).

**NeoForge 1.21.1 common tags** (from `Tags.java` on the 1.21.1 branch):
- Ore tags: `c:ores` plus `c:ores/{coal,copper,diamond,emerald,gold,iron,lapis,netherite_scrap,quartz,redstone}`. Only vanilla materials have constants; modded materials use the same convention, e.g. `c:ores/lead` in IE recipes.
- Host-ground tags, exactly three each: `c:ores_in_ground/{stone,deepslate,netherrack}` and `c:ore_bearing_ground/{stone,deepslate,netherrack}`.
- Drop-rate tags: `c:ore_rates/{sparse,singular,dense}`.
- Raw and storage tags: `c:raw_materials` with `/{copper,gold,iron}`, and `c:storage_blocks/raw_{copper,gold,iron}`.
- No docs page enumerating these ore tags was found; the general tag docs cover only the `c:` convention.

Sources: [Tags.java](https://raw.githubusercontent.com/neoforged/NeoForge/1.21.1/src/main/java/net/neoforged/neoforge/common/Tags.java); [NeoForge tag docs](https://docs.neoforged.net/docs/resources/server/tags/).

**Ore-variant mods**
- **Excavated Variants** (default branch 1.21.x) is data-driven. It uses four config types (ground types, ores, stones, modifiers) under `globaldata/<ns>/excavated_variants/`. At runtime it cross-joins every ore × stone pair that shares a ground type and lacks a hand-authored block, then generates the missing variant (block, assets, loot, tags). Modifiers use a filter DSL (`and/or/not`, `ground_type@`, `stone@`, `ore@`, `mod@`, `block@`). Toggles include `attempt_worldgen_replacement` and `add_conversion_recipes`. The exact MC patch pin is unverified ([README](https://raw.githubusercontent.com/lukebemishprojects/ExcavatedVariants/1.21.x/README.md)).
- **Unearthed** (cainthetrain) has first-party NeoForge builds only for 1.21.4 and 1.21.8. An unofficial 1.21.1 port exists ([Modrinth](https://api.modrinth.com/v2/project/unearthed-cainthetrain); [port](https://github.com/Chunchunmaru3D/UnEarthed-1.21.1-neoforge-port)).

**Other games** (sub-agent; flags carried over)
- **Dwarf Fortress.**
  - `[ENVIRONMENT:<class>:<inclusion>:<frequency>]` ties a mineral to a rock class; `[ENVIRONMENT_SPEC:<mineral>:<inclusion>:<frequency>]` ties it to one specific stone. Documented classes: ALL_STONE, IGNEOUS_ALL, IGNEOUS_INTRUSIVE, IGNEOUS_EXTRUSIVE, SOIL, SOIL_OCEAN, SOIL_SAND, METAMORPHIC, SEDIMENTARY, ALLUVIAL.
  - Inclusions: VEIN (sinuous streak, ~100 stones), CLUSTER (large ovoid, ~750 stones), CLUSTER_SMALL (1–9 tiles) and CLUSTER_ONE ([DF wiki: inorganic tokens](https://dwarffortresswiki.org/index.php/Inorganic_material_definition_token); [Vein](https://dwarffortresswiki.org/index.php/Vein)).
  - Layer eligibility comes from `[SEDIMENTARY]`, `[IGNEOUS_INTRUSIVE]`, `[IGNEOUS_EXTRUSIVE]`, `[METAMORPHIC]`, `[SOIL]` and `[AQUIFER]` on the stone. `METAL_ORE:<metal>:<pct>` is rolled four times per ore stone.
  - Examples: hematite `[ENVIRONMENT:SEDIMENTARY:VEIN:100][ENVIRONMENT:IGNEOUS_EXTRUSIVE:VEIN:100]…[METAL_ORE:IRON:100]` ([raw](https://dwarffortresswiki.org/index.php/DF2014:Hematite/raw)); cassiterite `[ENVIRONMENT:ALLUVIAL:CLUSTER_SMALL:100][ENVIRONMENT_SPEC:GRANITE:VEIN:100]` ([raw](https://dwarffortresswiki.org/index.php/Cassiterite/raw)).
  - Region biome gates the layer stones: soil, then sedimentary (80%) or extrusive, then metamorphic, then intrusive deepest (with gems such as kimberlite diamonds) ([Layer](https://dwarffortresswiki.org/index.php/Layer); [Biome](https://dwarffortresswiki.org/index.php/Biome)).
  - SEDIMENTARY_OCEAN_* classes are unverified.
- **Vintage Story.**
  - Deposit JSON has `code`, `triesPerChunk` and `generator`, plus `attributes` {`inblock`, `placeblock` (e.g. `game:sand-{rock}`, one variant per host rock), `radius`, `yPosRel`, `thickness`}, each a distribution {`dist` gaussian/uniform/invexp, `avg`, `var`} ([VS modding: simple worldgen](https://wiki.vintagestory.at/Modding:Content_Tutorial_Simple_Worldgen)).
  - Generator families: FollowSurface, FollowSurfaceBelow, FollowSeaLevel, Anywhere and ChildDeposit. ChildDeposit spawns only inside a parent body, e.g. gold and silver in quartz, sylvite in halite ([Ore Deposits](https://wiki.vintagestory.at/Ore_Deposits/en)).
  - Strata come from geologic-province noise with igneous, metamorphic and sedimentary groups ([Terrain Generation](https://wiki.vintagestory.at/Terrain_Generation)).
  - Prospecting pick: Density mode samples 3 rocks within 16 blocks and reports qualitative per-mille odds without a location; Node mode scans radius 6 (8 on the Exploration playstyle), shows the direction and size of the deposit, and detects child deposits ([Guide:Prospecting](https://wiki.vintagestory.at/Guide:Prospecting); [Prospecting pick](https://wiki.vintagestory.at/index.php/Prospecting_pick)).
- **Factorio.** `AutoplaceSpecification` has `probability_expression`, `richness_expression`, `control` and `tile_restriction` ([Lua API](https://lua-api.factorio.com/latest/types/AutoplaceSpecification.html)). Frequency, size and richness sliders act independently. The starting area guarantees coal, iron, copper and stone patches ([Map generator](https://wiki.factorio.com/Map_generator)). Richness grows with distance from spawn, per community sources ([forum](https://forums.factorio.com/viewtopic.php?t=40210)). Version 2.0 unifies placement on spot-noise expressions ([FFF-390](https://factorio.com/blog/post/fff-390)).
- **Satisfactory.** Nodes are hand-placed with fixed locations. Purity is Impure ×0.5, Normal ×1 or Pure ×2 extraction; resource wells use satellite nodes of mixed purity ([Resource Node](https://satisfactory.wiki.gg/wiki/Resource_Node); [Resource Well](https://satisfactory.wiki.gg/wiki/Resource_Well)).
- **Minecraft vanilla 1.18+.**
  - `minecraft:ore` config: `size` (0–64), `discard_chance_on_air_exposure` and `targets` (rule test → state), wrapped in placed features with `count`, `height_range` (uniform/trapezoid), `in_square` and `biome` ([Ore (feature)](https://minecraft.wiki/w/Ore_(feature)); [Placed feature](https://minecraft.wiki/w/Placed_feature)).
  - Large ore veins come from noise-router density functions `vein_toggle`, `vein_ridged` and `vein_gap`: copper with granite at Y 0–50, iron with tuff at Y −60 to −8, plus a 2% raw-ore-block chance ([Ore vein](https://minecraft.wiki/w/Ore_vein); [Noise router](https://minecraft.wiki/w/Noise_router)).
- **RimWorld and Oxygen Not Included.** A ground-penetrating scanner creates deep deposits that a deep drill extracts; a long-range mineral scanner finds surface lumps ([Deep drill](https://rimworldwiki.com/wiki/Deep_drill); [GPS](https://rimworldwiki.com/wiki/Ground-penetrating_scanner)). ONI geysers are renewable nodes, tunable with Geotuners ([Geyser](https://oxygennotincluded.wiki.gg/wiki/Geyser)).
- **Simulated-geology ore in games.** None found in a shipped game. ProcGenesis advertises tectonics, erosion and ore views but documents no algorithm ([procgenesis.com](https://procgenesis.com/)). Gleba and World Orogen simulate rock and tectonics without ores ([Gleba](https://calandiel.itch.io/gleba); [Orogen](https://www.orogen.studio/)). Azgaar's generator has no geology or ore model ([discussion #1091](https://github.com/Azgaar/Fantasy-Map-Generator/discussions/1091)).

**What players liked and disliked** (sub-agent; Reddit was blocked from this environment and WebSearch was exhausted, so evidence leans on GitHub issues, PRs, wikis and changelogs, and likes are thin)
- **"Searched and found nothing" is the dominant complaint.**
  - IE: "I mined at level 10 and 14 for about an hour and found nothing" ([IE #2229, 2017](https://github.com/BluSunrize/ImmersiveEngineering/issues/2229)).
  - Immersive Petroleum: reservoirs "supposed to be 50% but we have searched far and wide… nothing comes back" ([IP #181, 2020](https://github.com/Flaxbeard/ImmersivePetroleum/issues/181)).
  - COE: "I've spent 3 hours straight flying in the nether trying to find [ancient debris]". The same user found rebalancing through KubeJS inaccessible ([COE #85, 2024](https://github.com/tom5454/Create-Ore-Excavation/issues/85)).
- **Players ask for an infinite escape hatch.** An IP request wanted parity with IE's `excavator_depletion = -1` for a "static oil station… which pumps infinite amount of oil" on bordered servers ([IP #96, 2018](https://github.com/Flaxbeard/ImmersivePetroleum/issues/96)). COE ships infinite veins by default (above).
- **Tools and map integration get built when prospecting is unclear.**
  - A cheat tool exists purely to find Geolosys veins ([GeolosysHax](https://github.com/TheAirBlow/GeolosysHax)).
  - GTCEu merged FTB Chunks ore-prospector compatibility ([PR #2642](https://github.com/GregTechCEu/GregTech-Modern/pull/2642)) and sends chat messages when new veins are prospected ([CHANGELOG](https://github.com/GregTechCEu/GregTech-Modern/blob/1.20.1/CHANGELOG.md)).
  - GT 1.12's prospector map was reworked for off-by-one chunk errors, JourneyMap/VoxelMap/Xaero waypoint export, and vein grouping ([GT PR #1194](https://github.com/GregTechCEu/GregTech/pull/1194)).
  - A COE fork adds "Multi-chunk Perlin-shaped ore deposits… distance-driven richness, Xaero map overlay, per-player reveal policies" ([create-ore-excavation-deposits](https://github.com/RandomCodeGuys/create-ore-excavation-deposits)).
  - TFC's field guide carries a Precision Prospecting addon section ([Field Guide](https://terrafirmacraft.github.io/Field-Guide/en_us/precisionprospecting/index.html)).
- **Performance traps.**
  - Geolosys ores had a mining delay on servers ([#82](https://github.com/oitsjustjose/Geolosys/issues/82)).
  - A `geolosys.dat` file grew past 50 MB, causing ~20 s freezes during chunk generation ([#158](https://github.com/oitsjustjose/Geolosys/issues/158)).
  - GTCEu's KubeJS bedrock-vein `.biomes()` call failed ([#3114](https://github.com/GregTechCEu/GregTech-Modern/issues/3114)).
- **Opacity.**
  - Dwarf Fortress: "You won't be told which kinds of metals are present" at embark. Players rely on proxies ("Shallow metals… and Flux") and DFHack `embark-tools` ([DF wiki: Embark](https://dwarffortresswiki.org/index.php/DF2014:Embark)).
  - Vintage Story: a developer stated that density readings "are position based", pushing back on a mod that displayed them as chunk-aligned ([Prospecting pick](https://wiki.vintagestory.at/index.php/Prospecting_pick)).
- **Author design direction.**
  - IE moved from chunk-based to positioned, radius-based veins ([PR #4052](https://github.com/BluSunrize/ImmersiveEngineering/pull/4052)).
  - GT 1.12 adapted Gregicality's spatial fluid-vein system, generating veins in 8×8-chunk regions with some intentionally empty ([GT PR #191](https://github.com/GregTechCEu/GregTech/pull/191)).
  - Mojang tuned 1.18 large veins publicly across snapshots ("smaller and more clumped", then "slightly rarer…", then "slightly larger and spawn more frequently"). Diamonds peak near Y −64 and gold near Y −16 ([Ore vein](https://minecraft.wiki/w/Ore_vein); [Java Edition 1.18](https://minecraft.wiki/w/Java_Edition_1.18)).

### Inferences
- **Common template across systems:**
  - host-rock replace→with tables (TFC `blocks`, Geolosys per-host `blocks`, VS `{rock}` templating, DF ENVIRONMENT/ENVIRONMENT_SPEC);
  - a rarity or weight knob;
  - a Y window;
  - size and density knobs;
  - biome and dimension filters;
  - indicators (TFC, GT, Geolosys `samples`);
  - grade tiers (TFC poor/normal/rich, Satisfactory purity);
  - yield and depletion (IE, IP, GT bedrock, COE finite);
  - a spatial layout primitive (GT grid, COE spacing/separation, IE noise maxima, IP noise-threshold polygons).

  A geology mod can keep every one of these but drive them from the sim rather than noise.
- **Worth adopting:**
  - TFC's per-rock ore blocks with grades, which make realism visible in the block itself.
  - VS ChildDeposit, which maps directly onto real mineral associations (Au in quartz veins, sylvite in halite, supergene zones in porphyries).
  - GT and Geolosys surface indicators, and IE core samples as a readable assay.
  - IP polygon reservoirs, but traced from the sim's actual trap closures (anticlines, salt domes) instead of Perlin noise.
- **Addressing the top complaint.** Make geology itself the clue, so players learn "arc volcanics near an old intrusion = copper". Add visible gossans and alteration halos, and stream-sediment or float sampling that points upstream toward the source (as real exploration works). Give prospecting results confidence levels and export them to map mods (Xaero, JourneyMap, FTB Chunks). Always ship config levers: an infinite toggle, richness and frequency.
- **Performance.** Store a compact per-region deposit registry (parameters plus seed) and regenerate block detail deterministically at chunk generation, rather than persisting per-block metadata (the Geolosys `.dat` lesson).
- **COE in MIC.** COE assigns veins from a salted spacing/separation grid, not coordinates, so sim deposits cannot be pinned to COE veins through data alone. A geology mod needs a COE-side hook (mixin or API) to report the sim deposit as the chunk's vein, or its own deep-drilling extraction.

### Gaps
- Direct player sentiment from Reddit, CurseForge comments and the Bay 12 and TFC forums was unreachable (Reddit blocked, search budget exhausted). TFC and GT "likes" are essentially missing. TFC's rationale for its 1.18+ vein rewrite and community reactions to Mojang's 1.18 changes were not found.
- Unverified details: TFC `kaolin_disc_vein` fields; GT `GTLayerPattern` fields (the sub-agent's KubeJS example was composed from builder methods, not copied from the repo); the IE `mineralVeinYield` config key; the Excavated Variants MC patch pin; a Geolosys `deposit_layer` example; the Unearthed source repo.
- Whether COE exposes a KubeJS or Java API for per-chunk vein assignment (not just new vein types) was not checked.

## Q7. Proposed data-driven deposit-type schema (real and fantasy ores), mapping to Minecraft/NeoForge conventions, and balance levers

### Takeaway
A deposit type should be a datapack JSON with nine field groups:
1. identity and products;
2. genesis as mineral-system components, with predicates over named sim fields so fantasy fields can be plugged in;
3. windows (age, depth, temperature);
4. sizing (density, lognormal grade–tonnage, correlations, spacing and clustering);
5. geometry and zoning, including child deposits;
6. indicators;
7. weathering, supergene and placer transforms;
8. preservation;
9. Minecraft mapping and balance.

Materials resolve through common `c:` tags to existing ore blocks where present, and otherwise generate per-host-stone variants; Excavated Variants' ground-type model is a compatible pattern. Balance runs through explicit count, size and richness multipliers, background scatter, starter guarantees, finite/infinite toggles and prospecting fidelity, all borrowed from systems players already know.

### Cited Findings
Conventions and precedents the schema builds on:
- NeoForge 1.21.1 defines `c:ores/<material>` conventions, `c:ores_in_ground/{stone,deepslate,netherrack}`, `c:ore_bearing_ground/{stone,deepslate,netherrack}`, `c:ore_rates/{sparse,singular,dense}` and `c:raw_materials/<material>`. Constants exist only for vanilla materials ([Tags.java](https://raw.githubusercontent.com/neoforged/NeoForge/1.21.1/src/main/java/net/neoforged/neoforge/common/Tags.java)).
- Other mods consume these tags. IE mineral mixes output `{"tag":"c:ores/lead"}` and can skip empty tags with NeoForge conditions ([galena.json](https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/generated/resources/data/immersiveengineering/recipe/mineral/galena.json); [MineralMixSerializer.java](https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/main/java/blusunrize/immersiveengineering/common/crafting/serializers/MineralMixSerializer.java)).
- **One ore block per host stone.** TFC authors every ore × rock (× grade) block explicitly ([Ore.java](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/common/blocks/rock/Ore.java)). Excavated Variants generates missing ore × stone variants at runtime from ground-type groupings ([README](https://raw.githubusercontent.com/lukebemishprojects/ExcavatedVariants/1.21.x/README.md)). Vintage Story templates `placeblock` per host rock (`game:sand-{rock}`) ([VS modding](https://wiki.vintagestory.at/Modding:Content_Tutorial_Simple_Worldgen)).
- **Rock-class predicates and nesting.** DF binds minerals to rock classes (ENVIRONMENT) or specific stones (ENVIRONMENT_SPEC) with an inclusion shape and frequency ([DF tokens](https://dwarffortresswiki.org/index.php/Inorganic_material_definition_token)). VS ChildDeposit places deposits inside parent bodies ([VS Ore Deposits](https://wiki.vintagestory.at/Ore_Deposits/en)).
- **Balance levers in the wild:**
  - Factorio: independent frequency, size and richness sliders plus a starting-area guarantee ([Map generator](https://wiki.factorio.com/Map_generator)).
  - Satisfactory: purity ×0.5/×1/×2 ([Resource Node](https://satisfactory.wiki.gg/wiki/Resource_Node)).
  - Vanilla: `discard_chance_on_air_exposure`, `count` and `height_range` ([Ore (feature)](https://minecraft.wiki/w/Ore_(feature))).
  - COE: `amountMultiplierMin/Max`, spacing/separation, `finiteAmountBase` 1000, `defaultInfinite` true ([repo](https://github.com/tom5454/Create-Ore-Excavation); local jar).
  - TFC: per-vein grade weights and a prospecting false-negative chance that falls with tool tier ([PropickItem.java](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.21.x/src/main/java/net/dries007/tfc/common/items/PropickItem.java)).
  - VS: Density vs Node prospecting modes ([Guide:Prospecting](https://wiki.vintagestory.at/Guide:Prospecting)).
  - IE: edge fail-chance and depletion ([ExcavatorHandler.java](https://raw.githubusercontent.com/BluSunrize/ImmersiveEngineering/1.21.1/src/api/java/blusunrize/immersiveengineering/api/excavator/ExcavatorHandler.java)).
  - IP: pressure-dependent flow ([ReservoirPolygon.java](https://raw.githubusercontent.com/TwistedGate/ImmersivePetroleum/1.21.1/src/main/java/flaxbeard/immersivepetroleum/api/reservoir/ReservoirPolygon.java)).
- **Player-demand signals the schema should serve:** confident discovery signals, map integration, infinite toggles, low persistence overhead ([IE #2229](https://github.com/BluSunrize/ImmersiveEngineering/issues/2229); [IP #96](https://github.com/Flaxbeard/ImmersivePetroleum/issues/96); [GT PR #1194](https://github.com/GregTechCEu/GregTech/pull/1194); [Geolosys #158](https://github.com/oitsjustjose/Geolosys/issues/158)).
- **Real-type sizing inputs** come from the USGS values in Q2, e.g. porphyry Cu tonnage 33/240/1,500 Mt and Cu 0.24/0.44/0.80% ([OFR 2008-1155](https://pubs.usgs.gov/of/2008/1155/)), and porphyry density P50 ≥15 per 100,000 km² ([Singer et al. 2005](https://doi.org/10.2113/gsecongeo.100.3.491)).

### Inferences
The proposal:


**Field groups.** "Sim fields" are named scalars or enums published per cell and time step by the tectonic, surface and climate engines, or registered by other mods and datapacks.

| Group | Proposed fields | Precedent |
|---|---|---|
| Identity | `id`, `family` (magmatic_hydrothermal, orthomagmatic, seafloor, basin_brine, metamorphic, sedimentary_chemical, residual, placer, fuel, fantasy), `display`, `lore_key` | — |
| Products | `products[]`: {`material` (resolved via `c:ores/<m>`, `c:raw_materials/<m>`), `grade_key`, `role` (primary/byproduct/trace), `block_policy` (reuse → generate → fallback)} | IE tag outputs |
| Genesis: components | `components`: {`source`, `driver`, `pathway`, `trap`}, each a list of terms {`field`, `curve` (piecewise-linear points) or `range`, `weight`}, combined by `combine`: `product`, `min` or `geometric_mean` | Skirrow et al. four components; Hronsky & Groves multiplicative ranking |
| Genesis: settings | `settings[]`: {`regime` (enum or tag, e.g. `#geo:regime/continental_arc`), `weight`} and `forbid_regimes[]` | DF ENVIRONMENT classes, made tectonic |
| Clocks & age | `clock_gates` (e.g. `geo:atmo_o2 > geo:goe`, `geo:land_plants`), `age_window_ma`, optional `age_weight_curve` for secular peaks | Cuney; Leach; Klemme & Ulmishek |
| Host lithology | `hosts`: {`any_of`/`all_of` rock tags, e.g. `#geo:rock/carbonate`}, `trap_contacts` (e.g. intrusive × carbonate) | TFC replace-lists; DF ENVIRONMENT_SPEC |
| Windows | `depth_window_m`, `temp_window_c`, optional `pressure_window_mpa`, `duration_myr` | USGS/AV100 depth and T data |
| Sizing | `density_per_1e5km2` {p90,p50,p10}; `tonnage_mt` {dist: lognormal, p90,p50,p10, pareto_tail_above}; `grades` {key: {dist,p90,p50,p10}}; `correlations` [[a,b,r]]; `min_separation_m`; `clustering` {model: thomas, camp_spacing_km, cluster_radius_km}; `area_from_tonnage` (regression) | USGS three-part method; Singer; Robert camp spacing |
| Geometry | `shape` (stockwork_shell, lens, pipe, vein_swarm, stratiform_sheet, blanket, disseminated_halo, placer_channel, strandline, pod, dike); `orientation` (follow_fault, follow_bedding, vertical, follow_basal_contact); `zoning[]` {zone, depth_rel, products, density_mult}; `children[]` {type_id, relation (inside, above, contact_carbonate, above_watertable, downstream)} | VS ChildDeposit; porphyry and VMS zoning |
| Indicators | `alteration_halo` {blocks, radius_mult}; `gossan` {block, requires_exposure}; `stream_sediment`/`float` {material, decay_km}; `geophysics` {magnetic, gravity, radiometric} as tool hints | GT/Geolosys indicators; B1693 "Weathering" and "Geochemical signature" fields |
| Weathering | `supergene` {requires: climate curve, water_table_drop_m, duration_myr; zones: leached_cap, oxide, enrichment {grade_mult}}; `oxide_products`; `resistates` (minerals that survive into placers) | Sillitoe 2005; laterite and bauxite models |
| Preservation | `formation_depth_m`, `vertical_extent_m`, `erosion_behavior` (truncate, destroy, spawn_placer), `placer_transport_km` | Kesler & Wilkinson; B1693 placers |
| Minecraft mapping | `per_host_stone`, `grades` (poor/normal/rich) or `ore_rates` (sparse/singular/dense), `raw_item`, `tags_add[]`, `ev_ground_type` (Excavated Variants), `y_compression` | NeoForge `c:` tags; TFC grades; EV |
| Balance | `count_mult`, `size_mult`, `richness_mult`, `background_scatter` {density, hosts}, `starter` {within_blocks, min_count}, `extraction` {finite, amount_base, infinite_toggle}, `prospecting` {detect_radius, false_negative, reveal: map_overlay} | Factorio, COE, TFC, VS, IE |

**Example A: real porphyry Cu-Mo-Au (abridged)**
```json
{
  "id": "geo:porphyry_cu_mo_au",
  "family": "magmatic_hydrothermal",
  "products": [
    {"material": "copper", "grade_key": "cu_pct", "role": "primary"},
    {"material": "molybdenum", "grade_key": "mo_pct", "role": "byproduct"},
    {"material": "gold", "grade_key": "au_gpt", "role": "byproduct"}
  ],
  "settings": [
    {"regime": "#geo:regime/continental_arc", "weight": 1.0},
    {"regime": "#geo:regime/oceanic_arc", "weight": 0.7}
  ],
  "components": {
    "source": [
      {"field": "geo:arc_magma_flux", "curve": [[0, 0], [0.3, 0.5], [1, 1]]},
      {"field": "geo:magma_oxidation", "curve": [[0.4, 0], [0.6, 1]]}
    ],
    "driver": [{"field": "geo:intrusion_depth_m", "curve": [[500, 0], [1500, 1], [4000, 1], [6000, 0]]}],
    "pathway": [
      {"field": "geo:fault_density", "curve": [[0, 0.2], [0.5, 1]]},
      {"field": "geo:convergence_rate_km_myr", "curve": [[30, 0.3], [100, 1], [160, 0.6]]}
    ],
    "trap": [{"field": "geo:crust_thickness_km", "curve": [[25, 0.3], [45, 1], [70, 1]]}],
    "combine": "product"
  },
  "hosts": {"any_of": ["#geo:rock/intermediate_intrusive", "#geo:rock/volcanic", "#geo:rock/carbonate"]},
  "depth_window_m": [1000, 6000],
  "temp_window_c": [250, 700],
  "sizing": {
    "density_per_1e5km2": {"p90": 4, "p50": 15, "p10": 35},
    "tonnage_mt": {"dist": "lognormal", "p90": 33, "p50": 240, "p10": 1500},
    "grades": {
      "cu_pct": {"dist": "lognormal", "p90": 0.24, "p50": 0.44, "p10": 0.80},
      "au_gpt": {"dist": "lognormal", "p50": 0.2}
    },
    "correlations": [["tonnage_mt", "au_gpt", -0.28]],
    "min_separation_m": 2000,
    "clustering": {"model": "thomas", "camp_spacing_km": 40}
  },
  "geometry": {
    "shape": "stockwork_shell",
    "zoning": [
      {"zone": "potassic_core", "products": ["copper", "molybdenum"], "density_mult": 1.0},
      {"zone": "phyllic", "products": ["copper"], "density_mult": 0.5},
      {"zone": "propylitic_halo", "products": [], "density_mult": 0.0}
    ],
    "children": [
      {"type_id": "geo:epithermal_hs", "relation": "above"},
      {"type_id": "geo:skarn_cu", "relation": "contact_carbonate"},
      {"type_id": "geo:supergene_cu_blanket", "relation": "above_watertable"}
    ]
  },
  "indicators": {
    "alteration_halo": {"blocks": ["geo:phyllic_altered_rock"], "radius_mult": 3},
    "gossan": {"block": "geo:hematitic_gossan"},
    "stream_sediment": {"material": "copper", "decay_km": 5}
  },
  "preservation": {"formation_depth_m": 2500, "vertical_extent_m": 1500, "erosion_behavior": "truncate"},
  "minecraft": {"per_host_stone": true, "grades": ["poor", "normal", "rich"]},
  "balance": {"count_mult": 2000, "size_mult": 0.001, "richness_mult": 1.0}
}
```
The cited values are the settings, depth window, P90/P50/P10 figures, correlation and 2 km separation (Q1/Q2). The Au P50, curve shapes, camp spacing for porphyries and all multipliers are illustrative.

**Example B: fantasy "mythril" in a "mana-rich ultramafic" setting (abridged)**
```json
{
  "id": "mymod:mythril_orthomagmatic",
  "family": "fantasy",
  "requires_fields": {
    "mymod:mana_flux": {
      "provider": "mymod:leyline_field",
      "fallback": {"noise": "simplex", "scale_km": 200, "coupled_to": "geo:plume_flux", "coupling": 0.6}
    }
  },
  "settings": [
    {"regime": "#geo:regime/plume_lip", "weight": 1.0},
    {"regime": "#geo:regime/ophiolite_obduction", "weight": 0.5}
  ],
  "components": {
    "source": [
      {"field": "geo:magma_type", "range": ["komatiite", "picrite"]},
      {"field": "mymod:mana_flux", "curve": [[0.5, 0], [0.8, 1]]}
    ],
    "driver": [{"field": "geo:mantle_potential_temp_c", "curve": [[1350, 0], [1500, 1]]}],
    "pathway": [{"field": "geo:rift_fault_density", "curve": [[0.1, 0], [0.4, 1]]}],
    "trap": [{"field": "geo:host_sulfur_content", "curve": [[0, 0.2], [1, 1]]}],
    "combine": "product"
  },
  "age_window_ma": [4000, 0],
  "hosts": {"any_of": ["#geo:rock/ultramafic", "#geo:rock/serpentinite"]},
  "sizing": {
    "analog": "geo:ni_cu_pge_komatiite",
    "tonnage_mt": {"dist": "lognormal", "p90": 0.05, "p50": 0.5, "p10": 5},
    "grades": {"mythril_ppm": {"dist": "lognormal", "p50": 3}}
  },
  "geometry": {
    "shape": "lens",
    "orientation": "follow_basal_contact",
    "zoning": [
      {"zone": "basal_massive", "density_mult": 1.0},
      {"zone": "net_textured", "density_mult": 0.5}
    ]
  },
  "indicators": {
    "gossan": {"block": "mymod:glimmering_gossan"},
    "stream_sediment": {"material": "mythril", "decay_km": 20}
  },
  "weathering": {"resistates": ["mythril"]},
  "preservation": {"erosion_behavior": "spawn_placer", "placer_transport_km": 50},
  "products": [{"material": "mythril", "grade_key": "mythril_ppm", "role": "primary"}],
  "minecraft": {"per_host_stone": true, "tags_add": ["c:ores/mythril"], "raw_item": "c:raw_materials/mythril"}
}
```
The fantasy setting is ordinary geology plus one registered custom field with a fallback. A world without the provider mod still places mythril plausibly, coupled to plume flux. Sizing borrows a real analog (komatiite Ni-Cu), so rarity stays believable. If another mod already provides mythril ore, its blocks are reused through the tag. The numbers are illustrative.

**Mapping rules**
1. Resolve each `material` to `c:ores/<m>`. For vanilla grounds, reuse an existing block that is in both `c:ores/<m>` and `c:ores_in_ground/<ground>`. Otherwise generate `<ns>:<host>_<m>_ore`, or delegate to Excavated Variants by publishing host stones as EV "stones" in a ground type and materials as EV "ores".
2. Map host lithology classes to blocks through tags (`#geo:rock/granite` → minecraft:granite, a Create stone, or any stone-mod block). The sim decides lithology, the tag decides the concrete block, so stone and biome mods coexist.
3. Map grade to both ore-block density inside the body and grade tiers (poor/normal/rich), and use `c:ore_rates` for drop multiplicity.
4. Compress Y by type order rather than absolute depth (Q5).
5. Emit extraction-field data (COE-like vein records, IP-like reservoirs) from the same deposit registry, so drilling and fluid mods agree with the visible geology.

**Balance levers** (defaults to tune)
- **Per-type and global multipliers**: `count_mult`, `size_mult`, `richness_mult`, as in Factorio's sliders.
- **`background_scatter`**: low-density single-block or small-cluster occurrences of common ores (Fe, Cu, coal, Zn for Create) in any eligible host, like DF CLUSTER_SMALL/ONE. This prevents "found nothing".
- **`starter` guarantee**: at least one shallow, exposed deposit of each essential early material within N blocks of spawn, as Factorio does.
- **Legendary deposits**: cap or guarantee a number of heavy-tail giants per world (Singer's world-class share).
- **Extraction**: finite deposits with an infinite toggle, as COE and IE players request. Pressure or depletion curves for fluids (IP).
- **Prospecting fidelity by tool tier**: radius, false negatives and resolution (TFC, VS), plus map-overlay export (GT PR #1194).
- **World-size compression**: `count_mult` and `size_mult` scaled to world size, since the real porphyry density (~15 per 100,000 km²) gives almost none in a 100 km² world (Q2).

### Gaps
- No cross-mod standard exists for a deposit-type definition, and none of the reviewed mods expresses tectonic or temporal predicates. The schema is a proposal and has not been tested against real sim output.
- NeoForge defines host-ground tags only for stone, deepslate and netherrack. No documented convention was found for tagging ores in other host stones, so the mod must pick its own namespace or rely on Excavated Variants' ground types.
- Excavated Variants' exact 1.21.1 compatibility and any API for programmatic (non-datapack) registration were not verified.
- It is unknown whether COE exposes a per-chunk vein-assignment API.
