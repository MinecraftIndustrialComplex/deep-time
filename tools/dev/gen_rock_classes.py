#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Generate Deep Time's 60 rock-class definitions (plan §5.4.1) from the table below.

    tools/dev/gen_rock_classes.py [<output dir>]

The output is committed; re-run after editing the table. It goes to
neoforge/src/main/resources/data/deeptime/deeptime/rock_class/<class>.json (the datapack registry
`deeptime:rock_class`). Stale *.json files there are deleted first, and nothing is written unless
the whole table validates.

The vocabulary (class names, families, groups, stand-ins) is MIC's rock-tag README,
/home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/README.md: CLASSES follows
its Classes table in order, GROUPS its Groups table. `vanilla_fallback` is the class's vanilla member
there, or the last entry of its fallback list.

density_kg_m3 is a typical dry bulk density (bulk density of the loose material for unconsolidated
classes). erodibility and diffusivity are relative multipliers on the stream-power K and the hillslope
diffusivity, with generic crystalline basement (granite, gneiss: what "unclassified crust" means) at
1.0. P1-12 set them; P1-07 reviewed them against the rock map and meridian cross-sections of the
coupled pipeline (STATUS.md, P1-07); every file says where each value came from.

P1-07 adds two optional keys that the structure stage reads (crates/deeptime-core/src/structure/):
`lithifies_to` on unconsolidated classes (what loose material and buried regolith become once
buried below the lithification depth or older than the lithification age) and `loose_equivalent` on
the clastic rocks the facies rules lay down (how the young, shallow part of such a deposit is shown:
sandstone's top as sand). The metamorphic path is the prograde regional sequence the stage steps a
class along; the temperature of each step is the structure stage's (metamorphism.rs).

P1-28 adds `stone_use` (see STONE_USE below): whether a class's blocks are tool-grade stone, building stone
or neither. It is the class's own data (rock_class/<class>.json), and the generator derives from it the
two item tags that gate vanilla's stone recipes, `minecraft:stone_tool_materials` and
`minecraft:stone_crafting_materials` (data/minecraft/tags/item/, `replace: false`).

`facies_rule` names the setting that forms a class. The Phase 1 core forms these: carbonate_platform,
dolomitisation, evaporite_basin, euxinic_basin, marine_shelf_mud, floodplain, alluvial_fan,
peat_swamp (surface/facies.rs), pedogenesis, deep_weathering, tropical_weathering
(surface/weathering.rs), regional_metamorphism (structure/metamorphism.rs), arc_plutonism, batholith,
contact_aureole, contact_metasomatism (structure/plutons.rs). The others name settings later stages
may simulate; the core does not read the key.
"""
import glob
import json
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
    REPO, "neoforge", "src", "main", "resources", "data", "deeptime", "deeptime", "rock_class")

# The data root the item tags go to: the one the rock_class folder sits in, unless it was moved.
DATA = os.path.dirname(os.path.dirname(os.path.dirname(OUT)))

# README families, in its order.
FAMILIES = (
    "intrusive_igneous",     # Intrusive igneous
    "volcanic",              # Volcanic
    "clastic_sedimentary",   # Clastic sedimentary
    "chemical_sedimentary",  # Chemical and biogenic sedimentary
    "metamorphic",           # Metamorphic
    "special_host",          # Special hosts
    "unconsolidated",        # Unconsolidated and regolith
)

# README Groups table, in its order; "#x" is another group. `any` is left out of the files.
GROUPS = (
    ("intrusive", "granite granodiorite diorite gabbro anorthosite peridotite"),
    ("volcanic", "rhyolite andesite basalt tuff obsidian scoria pumice"),
    ("special", "kimberlite carbonatite pegmatite greisen"),
    ("igneous", "#intrusive #volcanic kimberlite carbonatite pegmatite"),
    ("clastic", "conglomerate diamictite sandstone red_bed mudstone shale black_shale"),
    ("chemical", "limestone dolostone chalk chert travertine halite gypsum anhydrite coal ironstone "
                 "banded_iron_formation phosphorite"),
    ("sedimentary", "#clastic #chemical"),
    ("metamorphic", "slate phyllite schist blueschist greenstone gneiss quartzite marble amphibolite "
                    "eclogite hornfels skarn serpentinite greisen"),
    ("unconsolidated", "soil saprolite laterite bauxite gravel sand clay mud till loess volcanic_ash"),
    ("felsic", "granite granodiorite rhyolite pegmatite greisen"),
    ("intermediate", "diorite andesite"),
    ("mafic", "gabbro anorthosite basalt scoria amphibolite greenstone"),
    ("ultramafic", "peridotite serpentinite kimberlite"),
    ("carbonate", "limestone dolostone chalk travertine marble"),
    ("evaporite", "halite gypsum anhydrite"),
    ("iron_formation", "ironstone banded_iron_formation"),
    ("any", "#igneous #sedimentary #metamorphic #special #unconsolidated #unclassified_crust"),
)
EXCLUDED_GROUPS = {"any"}
NON_CLASS_MEMBERS = {"unclassified_crust"}  # a filler tag, not a class

# README "Create ore stones" table: the class of each Create ore stone and its metal.
CREATE_METAL_HOSTS = {"banded_iron_formation": "iron", "ironstone": "gold",
                      "serpentinite": "copper", "dolostone": "zinc"}

FILLER = {"minecraft:stone", "minecraft:deepslate"}
FILLER_EXCEPTIONS = {("coal", "minecraft:stone")}  # README "Coal stand-in (decided)"

REFERENCE_CLASSES = {"granite", "gneiss"}

TUNED = ("set in P1-12; reviewed in P1-07 against the rock map and meridian cross-sections of "
         "pipeline_quick_16k and kept (STATUS.md, P1-07)")
DENSITY_TYPICAL = ("typical value, cf. the rock-density tables of Telford, Geldart & Sheriff 1990 "
                   "(Applied Geophysics, 2nd ed.) and Schön 2015 (Physical Properties of Rocks, 2nd ed.)")
K_BEDROCK = ("bedrock ranked by rock strength: river incision scales roughly with tensile strength^-2 "
             "(Sklar & Dietrich 2001, Geology 29(12):1087–1090) and lithologic K contrasts span orders of "
             "magnitude (Stock & Montgomery 1999, JGR 104(B3):4983–4993); the strength-based contrast is "
             "compressed to its square root, so bedrock spans about 0.3–4")
K_LOOSE = ("loose sediment, ranked above all bedrock (about 4–10) by ease of entrainment: silt and fine "
           "sand move at the lowest flow speeds, cohesive clay and coarse gravel need more "
           "(Hjulström 1935, Bull. Geol. Inst. Univ. Uppsala 25)")
D_BASIS = ("hillslope transport coefficient D of Roering, Kirchner & Dietrich 1999 (WRR 35(3):853–870), "
           "used as a linear creep multiplier and scaled by how readily the rock makes mobile soil "
           "(soil production function, Heimsath et al. 1997, Nature 388:358–361): loose and friable "
           "classes 2–5, soft sedimentary 1.2–2, crystalline 0.5–1")
VF_MEMBER = "vanilla member (MIC rock-tag README, Classes table)"
VF_FALLBACK = "last fallback entry (MIC rock-tag README, Fallbacks table)"

PELITE = ["slate", "phyllite", "schist", "gneiss"]

# P1-07: what loose material becomes once buried (structure/columns.rs). Buried regolith (the
# palaeosols of the strata log) lithifies too: soil and saprolite to mudstone (palaeosol claystone),
# laterite to ironstone. Bauxite stays bauxite (buried bauxites keep their class), and there is no
# entry for classes without a loose state.
LITHIFIES_TO = {
    "soil": "mudstone", "saprolite": "mudstone", "laterite": "ironstone",
    "gravel": "conglomerate", "sand": "sandstone", "clay": "mudstone", "mud": "shale",
    "till": "diamictite", "loess": "mudstone", "volcanic_ash": "tuff",
}
# P1-07: how the young, shallow part of a clastic deposit is shown (structure/columns.rs): the
# facies rules lay down the rock class, and sediment still within the lithification depth and age is
# its loose equivalent. Red beds show as sand (the sand class holds red sand).
LOOSE_EQUIVALENT = {
    "conglomerate": "gravel", "diamictite": "till", "sandstone": "sand", "red_bed": "sand", "mudstone": "clay",
    "shale": "mud", "black_shale": "mud", "tuff": "volcanic_ash",
}
MAFIC = ["greenstone", "amphibolite"]

# One row per class, in the README's Classes-table order.
#   rho / rho_src: density and, where it is not a textbook typical value, why (else DENSITY_TYPICAL)
#   k / k_why, d / d_why: erodibility and diffusivity with the judgment behind them
#   path: prograde regional-metamorphism sequence; wx: weathers_to (temperate), None to omit
#   facies: facies_rule, None to omit; vf / vf_from: vanilla_fallback and "member" or "fallback"
CLASSES = [
    # ---- Intrusive igneous
    dict(name="granite", family="intrusive_igneous", rho=2650,
         k=1.0, k_why=None, d=1.0, d_why=None,
         path=["gneiss"], wx="saprolite", facies="batholith", tags=[],
         vf="minecraft:granite", vf_from="member",
         notes="Granite, monzogranite, leucogranite and syenite. With gneiss, the reference class "
               "(erodibility and diffusivity 1.0) for generic crystalline crust."),
    dict(name="granodiorite", family="intrusive_igneous", rho=2720,
         k=1.0, k_why="mechanically like granite", d=1.0, d_why="weathers to grus like granite",
         path=["gneiss"], wx="saprolite", facies="arc_plutonism", tags=["ore_host"],
         vf="minecraft:diorite", vf_from="fallback",
         notes="Granodiorite and tonalite, including the stocks that host porphyry copper deposits."),
    dict(name="diorite", family="intrusive_igneous", rho=2850,
         k=0.8, k_why="finer, less quartz-rich and tougher than granite",
         d=0.9, d_why="makes less grus than granite",
         path=["gneiss"], wx="saprolite", facies="arc_plutonism", tags=[],
         vf="minecraft:diorite", vf_from="member",
         notes="Diorite and monzodiorite."),
    dict(name="gabbro", family="intrusive_igneous", rho=3000,
         k=0.6, k_why="tough interlocking pyroxene and plagioclase",
         d=0.8, d_why="massive crystalline rock with thin soils",
         path=MAFIC, wx="saprolite", facies=None, tags=[],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Gabbro, norite and dolerite/diabase: coarse intrusive basalt. Forms in ocean crust, "
               "layered intrusions and sills, so no single facies rule."),
    dict(name="anorthosite", family="intrusive_igneous", rho=2730,
         rho_src="judgment from mineralogy: almost all plagioclase (about 2620–2760 kg/m³)",
         k=0.9, k_why="coarse plagioclase rock, close to granite",
         d=0.9, d_why="coarse crystalline rock, close to granite",
         path=["gneiss"], wx="saprolite", facies="layered_intrusion", tags=["ore_host"],
         vf="minecraft:calcite", vf_from="fallback",
         notes="Anorthosite, host of Fe-Ti-V oxide ores in layered intrusions and massif anorthosites. "
               "Metamorphosed anorthosite is taken as anorthositic gneiss."),
    dict(name="peridotite", family="intrusive_igneous", rho=3300,
         k=0.8, k_why="strong when fresh, but serpentinises and weathers quickly",
         d=0.8, d_why="massive crystalline rock",
         path=["serpentinite"], wx="saprolite", facies="mantle_exhumation", tags=["ore_host"],
         vf="minecraft:tuff", vf_from="fallback",
         notes="Peridotite, dunite, harzburgite, pyroxenite and komatiite cumulates: upper mantle and "
               "ultramafic cumulates (Cr, Ni, PGE hosts). Hydrates to serpentinite."),
    # ---- Volcanic
    dict(name="rhyolite", family="volcanic", rho=2500,
         k=0.7, k_why="fine-grained siliceous lava, stronger than granite",
         d=0.7, d_why="fine siliceous rock that makes thin soils",
         path=["gneiss"], wx="saprolite", facies="silicic_volcanism", tags=[],
         vf="minecraft:granite", vf_from="fallback",
         notes="Rhyolite, dacite, felsic porphyry and ignimbrite matrix: the volcanic equivalent of granite."),
    dict(name="andesite", family="volcanic", rho=2600,
         k=0.8, k_why="fine-grained lava, stronger than granite but jointed",
         d=0.8, d_why="fine-grained lava",
         path=MAFIC, wx="saprolite", facies="arc_volcanism", tags=[],
         vf="minecraft:andesite", vf_from="member",
         notes="Andesite and other arc lavas."),
    dict(name="basalt", family="volcanic", rho=2900,
         k=0.7, k_why="dense, strong lava; columnar joints let rivers pluck it",
         d=0.8, d_why="fine-grained lava",
         path=MAFIC, wx="saprolite", facies=None, tags=[],
         vf="minecraft:smooth_basalt", vf_from="member",
         notes="Basalt, MORB, flood basalt, pillow lava and komatiite flows. Erupts at ridges, plumes and "
               "arcs, so no single facies rule."),
    dict(name="tuff", family="volcanic", rho=1900,
         rho_src="judgment: a mid value for dry tuff, whose density varies widely with porosity and welding",
         k=2.5, k_why="weak, porous consolidated ash",
         d=1.8, d_why="soft rock that breaks down into thick mobile regolith",
         path=["schist", "gneiss"], wx="clay", facies="explosive_volcanism", tags=["porous", "friable"],
         vf="minecraft:tuff", vf_from="member",
         notes="Tuff, welded ignimbrite and volcaniclastic rock: consolidated ash (loose ash is "
               "volcanic_ash). Glassy ash alters readily to clay."),
    dict(name="obsidian", family="volcanic", rho=2400,
         rho_src="judgment: density of rhyolitic glass",
         k=0.5, k_why="massive, strong glass that weathers slowly",
         d=0.5, d_why="weathers very slowly and makes little mobile soil",
         path=["gneiss"], wx="clay", facies="silicic_lava_flow", tags=["glassy"],
         vf="minecraft:obsidian", vf_from="member",
         notes="Obsidian and other volcanic glass. Glass devitrifies over geological time, so old "
               "obsidian is rare."),
    dict(name="scoria", family="volcanic", rho=1500,
         rho_src="judgment: highly vesicular basaltic rock, far below solid basalt",
         k=3.0, k_why="vesicular, weakly welded cinder",
         d=2.0, d_why="loose, cindery surface",
         path=MAFIC, wx="clay", facies="cinder_cone", tags=["porous", "friable"],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Red and black scoria and cinder."),
    dict(name="pumice", family="volcanic", rho=700,
         rho_src="judgment: pumice is often lighter than water",
         k=3.5, k_why="frothy glass, very weak",
         d=2.2, d_why="very weak, breaks down readily",
         path=["schist", "gneiss"], wx="clay", facies="plinian_eruption",
         tags=["glassy", "porous", "friable"],
         vf="minecraft:diorite", vf_from="fallback",
         notes="Pumice: frothy silicic glass from explosive eruptions."),
    # ---- Clastic sedimentary
    dict(name="conglomerate", family="clastic_sedimentary", rho=2550,
         rho_src="judgment: between sandstone and its pebbles",
         k=1.3, k_why="strength set by matrix and cement; the clasts resist",
         d=1.2, d_why="cemented sediment that sheds gravel",
         path=["quartzite"], wx="gravel", facies="alluvial_fan", tags=["reservoir", "ore_host"],
         vf="minecraft:cobblestone", vf_from="fallback",
         notes="Conglomerate, breccia and quartz-pebble conglomerate (host of Witwatersrand-type "
               "gold-uranium palaeoplacers). Metaconglomerate is taken as quartzite."),
    # Ben, 2026-09-30 ("Add diamictite"): lithified till, so buried glacial deposits (the deep-time
    # glaciers', the ice ages', a snowball Earth's) stop turning into conglomerate.
    dict(name="diamictite", family="clastic_sedimentary", rho=2600,
         rho_src="judgment: a compact, poorly sorted mud-matrix rock with dense clasts; slightly above "
                 "conglomerate",
         k=1.2, k_why="strength set by the indurated mud matrix; the clasts of every size resist",
         d=1.1, d_why="massive, unbedded rock that sheds a bouldery, clayey regolith",
         path=PELITE, wx="soil", facies="glacial", tags=[],
         vf="minecraft:cobbled_deepslate", vf_from="fallback",
         tuned="set with the glacial landforms (2026-09-30), between conglomerate and mudstone",
         notes="Diamictite and tillite: lithified till and glaciomarine diamict, clasts of every size "
               "floating in a mud matrix (the Neoproterozoic snowball diamictites, the Dwyka Group). "
               "Its muddy matrix takes the pelitic metamorphic path. Stand-in: cobbled deepslate, a "
               "dark matrix with lighter fragments of mixed size and shape, unlike conglomerate's "
               "clast-supported cobblestone."),
    dict(name="sandstone", family="clastic_sedimentary", rho=2350,
         k=1.5, k_why="weaker than granite; strongly dependent on its cement",
         d=1.3, d_why="soft sedimentary rock that sheds sand",
         path=["quartzite"], wx="sand", facies=None, tags=["reservoir"],
         vf="minecraft:sandstone", vf_from="member",
         notes="Sandstone, quartz arenite, arkose and greywacke/turbidite sand. Laid down by rivers, "
               "wind, waves and turbidity currents, so no single facies rule."),
    dict(name="red_bed", family="clastic_sedimentary", rho=2400,
         rho_src="judgment: a mix of sandstone and mudstone",
         k=1.8, k_why="weakly cemented continental sandstone and mudstone",
         d=1.5, d_why="soft sedimentary rock",
         path=["quartzite"], wx="sand", facies="floodplain",
         tags=["reservoir", "oxidised", "ore_host"],
         vf="minecraft:red_sandstone", vf_from="member",
         notes="Continental red beds: hematite-stained sandstone and mudstone of oxidising floodplains "
               "and dunes; host of sediment-hosted copper."),
    dict(name="mudstone", family="clastic_sedimentary", rho=2350,
         k=3.0, k_why="weak mudrock that slakes on wetting and drying",
         d=1.8, d_why="soft mudrock that makes thick clay-rich soil",
         path=PELITE, wx="clay", facies=None, tags=["seal"],
         vf="minecraft:packed_mud", vf_from="fallback",
         notes="Mudstone, siltstone and claystone. Laid down in lakes, floodplains and offshore, so no "
               "single facies rule."),
    dict(name="shale", family="clastic_sedimentary", rho=2450,
         k=2.5, k_why="weak, fissile mudrock",
         d=1.6, d_why="soft mudrock",
         path=PELITE, wx="clay", facies="marine_shelf_mud", tags=["seal"],
         vf="minecraft:polished_tuff", vf_from="fallback",
         notes="Grey shale and argillite: fissile offshore mudrock."),
    dict(name="black_shale", family="clastic_sedimentary", rho=2400,
         rho_src="plan §5.4.1 example value; organic matter puts it below grey shale",
         k=1.6, k_why="plan §5.4.1 example value; below grey shale because many organic-rich "
                      "shales are quartz-rich and brittle",
         d=1.2, d_why="plan §5.4.1 example value",
         path=PELITE, wx="clay", facies="euxinic_basin", tags=["source_rock", "reduced"],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Organic-rich (euxinic) black shale, the classic petroleum source rock. Matches the plan's "
               "§5.4.1 example row."),
    # ---- Chemical and biogenic sedimentary
    dict(name="limestone", family="chemical_sedimentary", rho=2600,
         k=1.2, k_why="mechanically close to granite; solution adds to it in humid climates",
         d=0.8, d_why="dissolves rather than creeping; thin residual soils",
         path=["marble"], wx="clay", facies="carbonate_platform", tags=["reservoir", "karst"],
         vf="minecraft:calcite", vf_from="fallback",
         notes="Limestone and reef carbonate. Weathers by solution, leaving residual clay."),
    dict(name="dolostone", family="chemical_sedimentary", rho=2750,
         k=1.1, k_why="slightly stronger and less soluble than limestone",
         d=0.8, d_why="dissolves rather than creeping, like limestone",
         path=["marble"], wx="clay", facies="dolomitisation",
         tags=["reservoir", "karst", "ore_host", "create_metal_host:zinc"],
         vf="minecraft:dripstone_block", vf_from="fallback",
         notes="Dolostone, host of Mississippi Valley-type and SEDEX zinc-lead deposits. Create's "
               "asurine (zinc) is its member."),
    dict(name="chalk", family="chemical_sedimentary", rho=1900,
         rho_src="judgment: porous chalk sits far below calcite's 2710 kg/m³",
         k=3.0, k_why="soft, porous, weakly cemented carbonate",
         d=1.8, d_why="soft, frost-shattered rock under soil-mantled slopes",
         path=["marble"], wx="clay", facies="pelagic_carbonate",
         tags=["reservoir", "porous", "friable"],
         vf="minecraft:calcite", vf_from="fallback",
         notes="Chalk: pelagic coccolith ooze. Weathers to clay-with-flints."),
    dict(name="chert", family="chemical_sedimentary", rho=2600,
         rho_src="judgment: microcrystalline quartz with a little porosity, just under quartz's 2650 kg/m³",
         k=0.3, k_why="microcrystalline quartz, among the most resistant rocks",
         d=0.5, d_why="breaks into hard fragments, makes little soil",
         path=["quartzite"], wx="gravel", facies="siliceous_ooze", tags=["ore_host"],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Chert, flint, jasper and exhalite (the siliceous horizon over volcanogenic "
               "massive-sulphide ores). Metachert is taken as quartzite."),
    dict(name="travertine", family="chemical_sedimentary", rho=2400,
         rho_src="judgment: porous calcite rock",
         k=1.6, k_why="porous carbonate, weaker than limestone",
         d=1.2, d_why="soft, porous carbonate",
         path=["marble"], wx="clay", facies="spring_carbonate", tags=["porous"],
         vf="minecraft:dripstone_block", vf_from="member",
         notes="Travertine, tufa and speleothem flowstone: carbonate precipitated from springs and cave water."),
    dict(name="halite", family="chemical_sedimentary", rho=2160,
         rho_src="mineral density of halite",
         k=4.0, k_why="weak and highly soluble; top of the bedrock range",
         d=1.5, d_why="flows and dissolves; rarely soil-mantled",
         path=[], wx=None, facies="evaporite_basin", tags=["soluble", "seal"],
         vf="minecraft:calcite", vf_from="fallback",
         notes="Rock salt and potash (sylvite) beds. Dissolves at the surface, leaving nothing to weather "
               "to; flows and rises in diapirs at depth."),
    dict(name="gypsum", family="chemical_sedimentary", rho=2310,
         rho_src="mineral density of gypsum",
         k=3.5, k_why="soft (Mohs 2) and soluble",
         d=1.6, d_why="soft and soluble",
         path=["anhydrite"], wx=None, facies="evaporite_basin", tags=["soluble", "seal"],
         vf="minecraft:calcite", vf_from="fallback",
         notes="Gypsum and alabaster. Dehydrates to anhydrite on burial and dissolves at the surface."),
    dict(name="anhydrite", family="chemical_sedimentary", rho=2950,
         rho_src="close to the mineral density of anhydrite",
         k=1.8, k_why="mechanically stronger than gypsum, but soluble and hydrates to gypsum at the surface",
         d=1.2, d_why="harder than gypsum but still soluble",
         path=[], wx="gypsum", facies="evaporite_basin", tags=["soluble", "seal"],
         vf="minecraft:diorite", vf_from="fallback",
         notes="Anhydrite. Hydrates back to gypsum near the surface."),
    dict(name="coal", family="chemical_sedimentary", rho=1300,
         k=3.0, k_why="weak, cleated, brittle",
         d=2.0, d_why="friable, breaks into small fragments",
         path=[], wx=None, facies="peat_swamp", tags=["source_rock", "reduced", "friable"],
         vf="minecraft:stone", vf_from="fallback",
         notes="Coal from lignite to anthracite; rank rises within the class, so no metamorphic path. Its "
               "vanilla fallback is stone, the one allowed filler exception."),
    dict(name="ironstone", family="chemical_sedimentary", rho=3000,
         rho_src="judgment: iron-oxide-rich oolitic sedimentary rock",
         k=1.3, k_why="moderately strong ferruginous rock",
         d=1.0, d_why="moderately strong ferruginous rock",
         path=[], wx="clay", facies="ironstone_shelf",
         tags=["fe_rich", "oxidised", "ore_host", "create_metal_host:gold"],
         vf="minecraft:red_sandstone", vf_from="fallback",
         notes="Phanerozoic oolitic ironstone and gossan. Create's ochrum (gold) is its member."),
    dict(name="banded_iron_formation", family="chemical_sedimentary", rho=3400,
         rho_src="judgment: chert bands (about 2650 kg/m³) alternating with hematite/magnetite "
                 "(about 5200 kg/m³), roughly a third iron oxide by volume",
         k=0.4, k_why="hard chert and iron-oxide laminae that hold up ridges",
         d=0.5, d_why="very resistant, makes little soil",
         path=[], wx="gravel", facies="ferruginous_ocean",
         tags=["fe_rich", "ore_host", "create_metal_host:iron"],
         vf="minecraft:granite", vf_from="fallback",
         notes="Banded iron formation, jaspilite and granular iron formation, the world's main iron ore. "
               "Create's crimsite (iron) is its member; metamorphosed BIF stays in the class."),
    dict(name="phosphorite", family="chemical_sedimentary", rho=2700,
         rho_src="judgment: carbonate-fluorapatite pellets (about 3100 kg/m³) in a porous sandy or muddy matrix",
         k=1.8, k_why="soft pelletal sediment",
         d=1.5, d_why="soft sedimentary rock",
         path=[], wx="clay", facies="upwelling_shelf", tags=["ore_host"],
         vf="minecraft:packed_mud", vf_from="fallback",
         notes="Phosphorite, U-bearing marine phosphate formed where upwelling feeds the shelf."),
    # ---- Metamorphic
    dict(name="slate", family="metamorphic", rho=2780,
         k=0.9, k_why="strong and fine, though it parts on cleavage",
         d=0.8, d_why="makes thin, platy rubble",
         path=["phyllite", "schist", "gneiss"], wx="clay", facies="regional_metamorphism",
         tags=["foliated"],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Slate: the lowest grade of the pelite sequence (slate, phyllite, schist, gneiss)."),
    dict(name="phyllite", family="metamorphic", rho=2740,
         rho_src="judgment: between slate and schist",
         k=1.4, k_why="micaceous, parts easily on its foliation",
         d=1.0, d_why="micaceous, weathers faster than slate",
         path=["schist", "gneiss"], wx="saprolite", facies="regional_metamorphism", tags=["foliated"],
         vf="minecraft:polished_tuff", vf_from="fallback",
         notes="Phyllite: one grade above slate."),
    dict(name="schist", family="metamorphic", rho=2750,
         k=1.3, k_why="mica-rich foliation lowers its strength",
         d=1.1, d_why="micaceous, weathers faster than gneiss",
         path=["gneiss"], wx="saprolite", facies="regional_metamorphism", tags=["foliated"],
         vf="minecraft:tuff", vf_from="fallback",
         notes="Mica schist and pelitic schist."),
    dict(name="blueschist", family="metamorphic", rho=3100,
         rho_src="judgment from mineralogy: glaucophane, lawsonite and epidote are denser than their "
                 "greenschist equivalents",
         k=0.7, k_why="dense, tough high-pressure rock",
         d=0.7, d_why="dense crystalline rock",
         path=["eclogite"], wx="saprolite", facies="subduction_high_p", tags=["foliated"],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Blueschist: high-pressure, low-temperature subduction facies; becomes eclogite with depth."),
    dict(name="greenstone", family="metamorphic", rho=2900,
         rho_src="judgment: metabasalt, close to basalt",
         k=0.8, k_why="tough metabasalt",
         d=0.8, d_why="massive crystalline rock",
         path=["amphibolite"], wx="saprolite", facies="regional_metamorphism",
         tags=["foliated", "ore_host"],
         vf="minecraft:tuff", vf_from="fallback",
         notes="Greenstone, greenschist and metabasalt; host of greenstone-belt (orogenic) gold."),
    dict(name="gneiss", family="metamorphic", rho=2700,
         k=1.0, k_why=None, d=1.0, d_why=None,
         path=[], wx="saprolite", facies="regional_metamorphism", tags=["foliated"],
         vf="minecraft:granite", vf_from="fallback",
         notes="Gneiss, orthogneiss, migmatite and granulite: the top of the regional sequence. With "
               "granite, the reference class (erodibility and diffusivity 1.0)."),
    dict(name="quartzite", family="metamorphic", rho=2640,
         rho_src="close to the mineral density of quartz",
         k=0.3, k_why="recrystallised quartz, among the most resistant rocks",
         d=0.5, d_why="barely weathers chemically; thin, sandy soils",
         path=[], wx="sand", facies="regional_metamorphism", tags=[],
         vf="minecraft:calcite", vf_from="fallback",
         notes="Quartzite: metamorphosed quartz sandstone, among the hardest rocks at the surface."),
    dict(name="marble", family="metamorphic", rho=2700,
         rho_src="close to the mineral density of calcite",
         k=1.2, k_why="mechanically close to limestone; soluble",
         d=0.8, d_why="dissolves rather than creeping, like limestone",
         path=[], wx="clay", facies="regional_metamorphism", tags=["karst"],
         vf="minecraft:calcite", vf_from="member",
         notes="Marble: metamorphosed limestone or dolostone."),
    dict(name="amphibolite", family="metamorphic", rho=3000,
         k=0.6, k_why="tough hornblende-plagioclase rock",
         d=0.7, d_why="dense crystalline rock",
         path=[], wx="saprolite", facies="regional_metamorphism", tags=["foliated"],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Amphibolite: amphibolite-facies metabasalt."),
    dict(name="eclogite", family="metamorphic", rho=3400,
         k=0.35, k_why="very dense, strong garnet-omphacite rock",
         d=0.5, d_why="very strong, makes little soil",
         path=[], wx="saprolite", facies="subduction_high_p", tags=[],
         vf="minecraft:tuff", vf_from="fallback",
         notes="Eclogite: the highest-pressure metabasalt, from subducted oceanic crust."),
    dict(name="hornfels", family="metamorphic", rho=2800,
         rho_src="judgment: dense, fine-grained baked mudrock, above its shale protolith",
         k=0.35, k_why="fine, massive, very hard contact rock",
         d=0.5, d_why="very hard; breaks into angular scree",
         path=[], wx="gravel", facies="contact_aureole", tags=[],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Hornfels: fine, hard rock baked in a contact aureole."),
    dict(name="skarn", family="metamorphic", rho=3200,
         rho_src="judgment from mineralogy: garnet and pyroxene with some calcite",
         k=0.6, k_why="tough garnet-pyroxene rock",
         d=0.6, d_why="tough crystalline rock",
         path=[], wx="saprolite", facies="contact_metasomatism", tags=["ore_host"],
         vf="minecraft:granite", vf_from="fallback",
         notes="Skarn and tactite: calc-silicate rock where magma meets carbonate; hosts tungsten, copper, "
               "iron, zinc and gold skarn ores."),
    dict(name="serpentinite", family="metamorphic", rho=2600,
         k=1.8, k_why="weak, sheared and slippery; landslide-prone",
         d=1.5, d_why="weak, sheared rock prone to slow earthflows",
         path=[], wx="clay", facies="ophiolite", tags=["ore_host", "create_metal_host:copper"],
         vf="minecraft:tuff", vf_from="fallback",
         notes="Serpentinite and the ophiolite mantle section. Create's veridium (copper) is its member."),
    # ---- Special hosts
    dict(name="kimberlite", family="special_host", rho=2800,
         rho_src="judgment: fresh hypabyssal kimberlite; weathered and volcaniclastic kimberlite is lighter",
         k=2.0, k_why="weathers fast; pipes erode into depressions",
         d=1.5, d_why="weathers quickly to clay-rich 'yellow ground'",
         path=[], wx="clay", facies="kimberlite_pipe", tags=["ore_host"],
         vf="minecraft:smooth_basalt", vf_from="fallback",
         notes="Kimberlite and lamproite pipes, the primary source of diamonds."),
    dict(name="carbonatite", family="special_host", rho=2900,
         rho_src="judgment from mineralogy: calcite and dolomite with apatite and magnetite",
         k=1.3, k_why="carbonate rock, weathers like marble",
         d=1.0, d_why="carbonate rock under deep residual regolith",
         path=[], wx="saprolite", facies="alkaline_complex", tags=["ore_host"],
         vf="minecraft:dripstone_block", vf_from="fallback",
         notes="Carbonatite, ijolite and fenite of alkaline complexes; host of rare-earth and niobium ores."),
    dict(name="pegmatite", family="special_host", rho=2620,
         rho_src="judgment: quartz and feldspar, like granite without its mafic minerals",
         k=1.1, k_why="very coarse; cleavage and grain boundaries weaken it",
         d=1.0, d_why="coarse granitic rock, like granite",
         path=["gneiss"], wx="saprolite", facies="pegmatite_field", tags=["ore_host"],
         vf="minecraft:granite", vf_from="fallback",
         notes="Granitic pegmatite (LCT and NYF), host of lithium, tantalum, tin and rare-element ores."),
    dict(name="greisen", family="special_host", rho=2750,
         rho_src="judgment from mineralogy: quartz and muscovite with topaz",
         k=1.0, k_why="quartz-mica rock, close to granite",
         d=0.9, d_why="quartz-rich altered granite",
         path=[], wx="saprolite", facies="granite_cupola", tags=["ore_host"],
         vf="minecraft:diorite", vf_from="fallback",
         notes="Greisen: quartz-mica altered granite cupola hosting tin and tungsten. Also in the "
               "metamorphic group (metasomatised granite)."),
    # ---- Unconsolidated and regolith
    dict(name="soil", family="unconsolidated", rho=1300,
         rho_src="judgment: typical dry bulk density of mineral topsoil",
         k=6.0, k_why="loose, though roots and clay give it some cohesion",
         d=4.0, d_why="the mobile layer itself",
         path=[], wx=None, facies="pedogenesis", tags=[],
         vf="minecraft:dirt", vf_from="member",
         notes="Soil and subsoil."),
    dict(name="saprolite", family="unconsolidated", rho=1600,
         rho_src="judgment: between soil and fresh rock",
         k=4.5, k_why="rotted in place but keeps some rock fabric",
         d=3.0, d_why="friable weathered rock that feeds the soil",
         path=[], wx="soil", facies="deep_weathering", tags=["friable"],
         vf="minecraft:coarse_dirt", vf_from="fallback",
         notes="Saprolite: rock weathered in place that keeps its fabric; the weathering product of most "
               "crystalline classes."),
    dict(name="laterite", family="unconsolidated", rho=2000,
         rho_src="judgment: iron-oxide-rich regolith; indurated ferricrete is denser",
         k=4.0, k_why="ferricrete crusts cap mesas; the least erodible regolith",
         d=2.0, d_why="partly indurated regolith",
         path=[], wx=None, facies="tropical_weathering", tags=["fe_rich", "oxidised", "ore_host"],
         vf="minecraft:red_sand", vf_from="fallback",
         notes="Laterite, ferricrete and Ni-Co laterite: iron-rich tropical regolith."),
    dict(name="bauxite", family="unconsolidated", rho=1900,
         rho_src="judgment: porous gibbsite-rich regolith",
         k=4.5, k_why="earthy to pisolitic, partly indurated",
         d=2.5, d_why="earthy regolith",
         path=[], wx=None, facies="tropical_weathering", tags=["oxidised", "ore_host"],
         vf="minecraft:red_sand", vf_from="fallback",
         notes="Bauxite: aluminium-rich tropical regolith, the aluminium ore."),
    dict(name="gravel", family="unconsolidated", rho=1800,
         rho_src="judgment: typical dry bulk density of loose gravel",
         k=5.0, k_why="coarse clasts need strong flows to move",
         d=3.0, d_why="loose, coarse sediment",
         path=["quartzite"], wx=None, facies="fluvial_channel", tags=["porous", "ore_host"],
         vf="minecraft:gravel", vf_from="member",
         notes="Gravel, alluvium and placer gravel. Lithifies to conglomerate on burial; its "
               "metamorphic path is conglomerate's."),
    dict(name="sand", family="unconsolidated", rho=1600,
         rho_src="judgment: loose quartz sand, about 40% porosity",
         k=9.0, k_why="cohesionless; moves at the lowest flow speeds",
         d=4.5, d_why="cohesionless, highly mobile",
         path=["quartzite"], wx=None, facies=None, tags=["porous", "ore_host"],
         vf="minecraft:sand", vf_from="member",
         notes="Sand, dune sand and heavy-mineral beach sand; laid down by rivers, wind and waves, so no "
               "single facies rule. Lithifies to sandstone on burial; its metamorphic path is sandstone's."),
    dict(name="clay", family="unconsolidated", rho=1600,
         rho_src="judgment: dry bulk density of compacted clay",
         k=5.0, k_why="cohesive: needs stronger flows than silt or fine sand",
         d=3.0, d_why="creeps readily when wet",
         path=PELITE, wx=None, facies=None, tags=["seal"],
         vf="minecraft:clay", vf_from="member",
         notes="Clay and kaolin, from weathering or quiet water, so no single facies rule. Lithifies to "
               "mudstone on burial; its metamorphic path is mudstone's."),
    dict(name="mud", family="unconsolidated", rho=1500,
         rho_src="judgment: bulk density of fresh, water-saturated mud",
         k=7.0, k_why="soft, water-rich, weakly cohesive",
         d=4.0, d_why="soft, water-rich, flows easily",
         path=PELITE, wx=None, facies=None, tags=["reduced"],
         vf="minecraft:mud", vf_from="member",
         notes="Mud of tidal flats, lakes and the sea floor, so no single facies rule. Lithifies to "
               "shale on burial; its metamorphic path is the pelites'."),
    dict(name="till", family="unconsolidated", rho=2000,
         rho_src="judgment: dense, overconsolidated lodgement till",
         k=4.5, k_why="dense, bouldery, overconsolidated diamict",
         d=2.5, d_why="dense but loose-textured sediment",
         path=PELITE, wx="soil", facies="glacial", tags=[],
         vf="minecraft:coarse_dirt", vf_from="member",
         notes="Glacial till and diamict. Lithifies to diamictite on burial (2026-09-30; it was "
               "conglomerate before), and heated it follows diamictite's pelitic path."),
    dict(name="loess", family="unconsolidated", rho=1400,
         rho_src="judgment: loess is characteristically light and open-structured",
         k=10.0, k_why="cohesionless silt that gullies readily; top of the range",
         d=4.0, d_why="cohesionless silt",
         path=PELITE, wx="soil", facies="aeolian", tags=["porous", "friable"],
         vf="minecraft:smooth_sandstone", vf_from="fallback",
         notes="Loess: wind-blown silt, parent of fertile steppe soils. Lithifies to siltstone "
               "(mudstone), whose metamorphic path it takes."),
    dict(name="volcanic_ash", family="unconsolidated", rho=1000,
         rho_src="judgment: loose tephra",
         k=10.0, k_why="loose, fine, light tephra; top of the range",
         d=5.0, d_why="loose, fine, light tephra",
         path=["schist", "gneiss"], wx="clay", facies="tephra_fall",
         tags=["glassy", "porous", "friable"],
         vf="minecraft:tuff", vf_from="fallback",
         notes="Unconsolidated volcanic ash and tephra; lithifies to tuff, whose metamorphic path it takes."),
]

# P1-28 (Ben, decisions log 2026-10-01): which rock is fit to make stone tools and furnaces from.
# Vanilla's recipes read two item tags that hold only cobblestone, cobbled deepslate and blackstone:
#   minecraft:stone_tool_materials      stone axe, hoe, pickaxe, shovel, sword (and repairing them)
#   minecraft:stone_crafting_materials  furnace, brewing stand
# so a player standing on granite or shale could not make a stone pickaxe or a furnace. (Everything
# else that wants cobblestone names the block itself: piston, dispenser, dropper, lever, observer,
# the cobblestone slabs. River, beach and till cobbles, CobblePass, supply those.) Each class gets
# a stone_use:
#   tool   hard, tough rock that holds an edge: Mohs hardness about 5.5+ and compressive strength
#          above ~100 MPa, massive or only weakly foliated, insoluble. In both tags.
#   build  solid, workable rock that is fit for walls and furnaces but would not hold an edge: soft
#          carbonates (Mohs 3), sandstone (hard grains, weak cement), foliated metamorphic rock that
#          splits along its planes, coarse crystals that cleave. In stone_crafting_materials only.
#   none   soft, crumbly, porous, soluble or loose: shale and the mudrocks, chalk, evaporites, coal,
#          pumice, scoria, most tuff, kimberlite, every unconsolidated class. In neither tag.
# The tier is the class's; the tags are made of ITEMS, so the generator maps classes to the blocks
# Deep Time itself places (the member tags in data/deeptime/tags/block/rock and each class's
# vanilla_fallback) and then to the items those drop. An item takes the tier of the class that owns
# it (the class whose member tag lists it); an item that is only ever a stand-in takes the lowest
# tier of the classes that stand in with it, because it may be standing in for the softer one.
# Items that serve classes of different tiers are printed by the generator; a pack fixes them by
# giving the hard classes stand-ins of their own. Cobblestone, cobbled deepslate and blackstone stay
# in the tags as vanilla has them (the tags are `replace: false`, so packs can add to them).
STONE_USE = {
    # name: (tier, reason)
    "granite": ("tool", "massive crystalline quartz-feldspar rock, Mohs 6, 150-250 MPa: the standard hard rock"),
    "granodiorite": ("tool", "as granite: Mohs 6, 100-250 MPa"),
    "diorite": ("tool", "tough interlocking plagioclase and hornblende, 150-250 MPa; ground-stone axes were made of it"),
    "gabbro": ("tool", "coarse intrusive basalt, among the toughest rocks (200-300 MPa); dolerite is a classic axe stone"),
    "anorthosite": ("tool", "almost all plagioclase (Mohs 6), coarse but strong, 150-250 MPa"),
    "peridotite": ("tool", "fresh olivine rock is hard (Mohs 6.5-7) and dense; it softens only where it serpentinises"),
    "rhyolite": ("tool", "fine-grained, siliceous and conchoidal: knapped and ground for tools worldwide"),
    "andesite": ("tool", "fine-grained, strong (100-250 MPa) lava; a common axe and mill stone"),
    "basalt": ("tool", "dense fine-grained lava, 150-300 MPa; the commonest ground-stone axe rock"),
    "tuff": ("none", "consolidated ash is porous and weak (5-30 MPa) and weathers to clay; welded ignimbrite is the exception and is not told apart"),
    "obsidian": ("tool", "volcanic glass: the sharpest edge there is (Mohs 5-5.5), brittle but a premier knapping stone"),
    "scoria": ("none", "vesicular, weakly welded cinder; crumbles under load"),
    "pumice": ("none", "frothy glass, lighter than water, abrades to powder"),
    "conglomerate": ("tool", "cemented pebbles of hard rock; its stand-in is cobblestone, which vanilla already counts"),
    "diamictite": ("tool", "lithified till, well cemented and clast-rich; its stand-in is cobbled deepslate, which vanilla already counts"),
    "sandstone": ("build", "quartz grains (Mohs 7) in a cement of varying strength: a classic walling stone, not a toolstone"),
    "red_bed": ("build", "hematite-cemented sandstone and mudstone: as sandstone, weaker"),
    "mudstone": ("none", "compacted mud: weak (10-40 MPa), slakes in water"),
    "shale": ("none", "splits into flakes along its bedding; weak and fissile"),
    "black_shale": ("none", "organic-rich shale: weaker still, and it burns"),
    "limestone": ("build", "calcite, Mohs 3, 50-150 MPa: scratches with a knife and takes no edge, but it is the commonest sedimentary rock and the standard walling and hearth stone, so a limestone world must still be able to build a furnace"),
    "dolostone": ("build", "dolomite, Mohs 3.5-4: harder and less soluble than limestone, still no toolstone"),
    "chalk": ("none", "soft, porous, friable micrite (1-10 MPa); it marks, it does not build"),
    "chert": ("tool", "microcrystalline silica (Mohs 7) with a conchoidal fracture: flint, the toolstone of the Stone Age"),
    "travertine": ("build", "dense banded carbonate quarried for the Colosseum; Mohs 3-4"),
    "halite": ("none", "water-soluble, Mohs 2.5"),
    "gypsum": ("none", "Mohs 2, scratches with a fingernail, soluble"),
    "anhydrite": ("none", "Mohs 3.5, hydrates to gypsum and dissolves"),
    "coal": ("none", "a fuel, not a rock to build with: Mohs 1-2.5 and brittle"),
    "ironstone": ("build", "iron-cemented sediment, hard but patchy and bedded: walls and hearths"),
    "banded_iron_formation": ("tool", "jasper and chert bands with hematite: extremely hard and tough (Mohs 6-7)"),
    "phosphorite": ("none", "soft, pelletal marine phosphate (crumbly)"),
    "slate": ("build", "splits into thin plates: roofing and paving, but it shatters rather than flakes to an edge"),
    "phyllite": ("build", "foliated with a silky sheen; splits like slate and is weaker"),
    "schist": ("build", "mica-rich and foliated: splits along the mica and crumbles in a wall, but it is solid bedrock"),
    "blueschist": ("tool", "dense, tough high-pressure metabasite (glaucophane, Mohs 6); with greenstone and amphibolite, a toolstone family"),
    "greenstone": ("tool", "tough metabasalt, the Neolithic axe stone of Langdale and Mount William"),
    "gneiss": ("tool", "banded, hard feldspar and quartz rock, 100-200 MPa; it cleaves only along weak bands"),
    "quartzite": ("tool", "fused quartz grains (Mohs 7), the toughest of the common rocks and a classic knapping stone"),
    "marble": ("build", "recrystallised calcite, Mohs 3: takes a polish, holds no edge"),
    "amphibolite": ("tool", "dense, tough metabasalt (hornblende, Mohs 5-6); ground-stone axes worldwide"),
    "eclogite": ("tool", "garnet and omphacite (Mohs 6-7), extremely dense and hard"),
    "hornfels": ("tool", "baked, massive, fine-grained and conchoidal: knapped into tools; the hardest contact rock"),
    "skarn": ("build", "patchy garnet and pyroxene (hard) in calcite (soft) with ore minerals: too uneven to knap"),
    "serpentinite": ("build", "Mohs 3-4 and slippery, carved rather than flaked; a solid building stone"),
    "kimberlite": ("none", "serpentinised and clay-altered 'blue ground' that weathers to a crumbling soil"),
    "carbonatite": ("build", "igneous carbonate, Mohs 3: solid but soft"),
    "pegmatite": ("build", "huge feldspar and quartz crystals that cleave: strong in a wall, shatters when knapped"),
    "greisen": ("build", "quartz-mica rock: hard quartz but flaky mica; fit for walls"),
    "soil": ("none", "loose"),
    "saprolite": ("none", "rotted rock that crumbles in the hand"),
    "laterite": ("none", "loose to earthy regolith (hardened ferricrete is not told apart)"),
    "bauxite": ("none", "earthy to pisolitic regolith"),
    "gravel": ("none", "loose"),
    "sand": ("none", "loose"),
    "clay": ("none", "loose"),
    "mud": ("none", "loose"),
    "till": ("none", "loose"),
    "loess": ("none", "loose silt"),
    "volcanic_ash": ("none", "loose"),
}
STONE_TIERS = ("none", "build", "tool")   # lowest to highest
VANILLA_STONE_ITEMS = ("minecraft:cobblestone", "minecraft:cobbled_deepslate", "minecraft:blackstone")

SNAKE = re.compile(r"^[a-z][a-z0-9_]*$")
TAG = re.compile(r"^[a-z][a-z0-9_]*(:[a-z][a-z0-9_]*)?$")


def expand_groups():
    """Group name -> set of classes it contains, following #group references."""
    table = dict(GROUPS)
    memo = {}

    def members(group, seen=()):
        if group in memo:
            return memo[group]
        if group in seen:
            raise SystemExit(f"group cycle through {group}")
        out = set()
        for m in table[group].split():
            if m.startswith("#"):
                ref = m[1:]
                if ref in NON_CLASS_MEMBERS:
                    continue
                if ref not in table:
                    raise SystemExit(f"group {group}: unknown group {m}")
                out |= members(ref, seen + (group,))
            else:
                out.add(m)
        memo[group] = out
        return out

    return {g: members(g) for g, _ in GROUPS}


CLASS_FAMILY = {r["name"]: r["family"] for r in CLASSES}


def validate(groups):
    errors = []
    for table, what in ((LITHIFIES_TO, "LITHIFIES_TO"), (LOOSE_EQUIVALENT, "LOOSE_EQUIVALENT")):
        for k in table:
            if k not in CLASS_FAMILY:
                errors.append(f"{what}: unknown class {k}")
    names = [r["name"] for r in CLASSES]
    known = set(names)
    if len(CLASSES) != 60:
        errors.append(f"expected 60 classes, got {len(CLASSES)}")
    if len(known) != len(names):
        errors.append("duplicate class names")
    if len(GROUPS) != 17:
        errors.append(f"expected 17 group tags, got {len(GROUPS)}")
    for g, members in groups.items():
        for m in members - known:
            errors.append(f"group {g}: unknown class {m}")
    last_family = 0
    for r in CLASSES:
        n = r["name"]
        if not SNAKE.match(n):
            errors.append(f"{n}: name is not snake_case")
        if r["family"] not in FAMILIES:
            errors.append(f"{n}: unknown family {r['family']}")
        else:
            i = FAMILIES.index(r["family"])
            if i < last_family:
                errors.append(f"{n}: out of README family order")
            last_family = i
        if not class_groups(n, groups):
            errors.append(f"{n}: in no group")
        if not (isinstance(r["rho"], int) and r["rho"] > 0 and r["rho"] % 10 == 0):
            errors.append(f"{n}: density must be a positive multiple of 10")
        for key in ("k", "d"):
            if not (isinstance(r[key], float) and r[key] > 0):
                errors.append(f"{n}: {key} must be a float > 0")
        path = r["path"]
        if len(set(path)) != len(path) or n in path:
            errors.append(f"{n}: metamorphic_path repeats a class or contains itself")
        for p in path:
            if p not in known:
                errors.append(f"{n}: metamorphic_path names unknown class {p}")
        if r["wx"] is not None and (r["wx"] not in known or r["wx"] == n):
            errors.append(f"{n}: bad weathers_to {r['wx']}")
        lith = LITHIFIES_TO.get(n)
        if lith is not None and (lith not in known or r["family"] != "unconsolidated"
                                 or CLASS_FAMILY.get(lith) == "unconsolidated"):
            errors.append(f"{n}: lithifies_to {lith} must take a loose class to a rock class")
        loose = LOOSE_EQUIVALENT.get(n)
        if loose is not None and (CLASS_FAMILY.get(loose) != "unconsolidated"
                                  or r["family"] == "unconsolidated"):
            errors.append(f"{n}: loose_equivalent {loose} must take a rock class to a loose class")
        if r["facies"] is not None and not SNAKE.match(r["facies"]):
            errors.append(f"{n}: facies_rule is not snake_case")
        for t in r["tags"]:
            if not TAG.match(t):
                errors.append(f"{n}: bad tag {t}")
        if len(set(r["tags"])) != len(r["tags"]):
            errors.append(f"{n}: duplicate tags")
        metals = [t.split(":", 1)[1] for t in r["tags"] if t.startswith("create_metal_host:")]
        want = [CREATE_METAL_HOSTS[n]] if n in CREATE_METAL_HOSTS else []
        if metals != want:
            errors.append(f"{n}: create_metal_host tags {metals}, README says {want}")
        vf = r["vf"]
        if not vf.startswith("minecraft:"):
            errors.append(f"{n}: vanilla_fallback {vf} is not in the minecraft namespace")
        if vf in FILLER and (n, vf) not in FILLER_EXCEPTIONS:
            errors.append(f"{n}: vanilla_fallback {vf} is filler")
        if r["vf_from"] not in ("member", "fallback"):
            errors.append(f"{n}: vf_from must be member or fallback")
        for ident in [vf] + class_groups(n, groups):
            if "terracotta" in ident:
                errors.append(f"{n}: id {ident} contains terracotta")
        if not r["notes"].strip():
            errors.append(f"{n}: empty notes")
    for n in STONE_USE:
        if n not in known:
            errors.append(f"STONE_USE: unknown class {n}")
    for r in CLASSES:
        n = r["name"]
        use = STONE_USE.get(n)
        if use is None:
            errors.append(f"{n}: no STONE_USE row")
        elif use[0] not in STONE_TIERS or not use[1].strip():
            errors.append(f"{n}: STONE_USE needs a tier of {STONE_TIERS} and a reason")
        members = block_members(n)
        if r["vf_from"] == "member" and r["vf"] not in members:
            errors.append(f"{n}: vanilla_fallback {r['vf']} is said to be a member but is not in its member tag")
    if errors:
        raise SystemExit("rock-class table invalid:\n  " + "\n  ".join(errors))


MEMBER_TAGS = os.path.join(REPO, "neoforge", "src", "main", "resources", "data", "deeptime", "tags", "block", "rock")


def block_members(name):
    """The blocks Deep Time's own member tag `deeptime:rock/<name>` lists (a pack adds more)."""
    path = os.path.join(MEMBER_TAGS, name + ".json")
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as f:
        return [v for v in json.load(f)["values"] if isinstance(v, str) and not v.startswith("#")]


def stone_items():
    """Item id -> tier, and the items serving classes of more than one tier (item -> {class: tier}).

    The blocks Deep Time itself places are each class's member tag and its vanilla_fallback; the
    item a block drops has the block's id for all of them. See STONE_USE for the ownership rule."""
    owners, riders = {}, {}
    for r in CLASSES:
        n = r["name"]
        tier = STONE_USE[n][0]
        members = block_members(n)
        for m in members:
            owners.setdefault(m, {})[n] = tier
        if r["vf"] not in members:
            riders.setdefault(r["vf"], {})[n] = tier
    tiers = {}
    for item, by in owners.items():
        tiers[item] = max(by.values(), key=STONE_TIERS.index)
    for item, by in riders.items():
        if item not in owners:
            tiers[item] = min(by.values(), key=STONE_TIERS.index)
    shared = {}
    for item in set(owners) | set(riders):
        by = dict(owners.get(item, {}))
        by.update(riders.get(item, {}))
        if len(set(by.values())) > 1:
            shared[item] = by
    return tiers, shared


def item_tag(comment, items):
    return {"replace": False, "_comment": comment, "values": sorted(items)}


def class_groups(name, groups):
    return [f"deeptime:rock/{g}" for g, _ in GROUPS
            if g not in EXCLUDED_GROUPS and name in groups[g]]


def sources(r):
    n = r["name"]
    tuned = r.get("tuned", TUNED)
    rho = f"{r['rho_src'] if r.get('rho_src') else DENSITY_TYPICAL}; rounded to 10 kg/m³; {tuned}"
    if n in REFERENCE_CLASSES:
        k = f"reference: generic crystalline basement (granite, gneiss) is 1.0; {K_BEDROCK}; {tuned}"
        d = f"reference: generic crystalline basement (granite, gneiss) is 1.0; {D_BASIS}; {tuned}"
    else:
        basis = K_LOOSE if r["family"] == "unconsolidated" else K_BEDROCK
        k = f"judgment, relative to granite: {r['k_why']}; {basis}; {tuned}"
        d = f"judgment, relative to granite: {r['d_why']}; {D_BASIS}; {tuned}"
    vf = VF_MEMBER if r["vf_from"] == "member" else VF_FALLBACK
    if (n, r["vf"]) in FILLER_EXCEPTIONS:
        vf += "; stone is the one allowed filler exception (Ben's pick 2026-09-27)"
    return {"density_kg_m3": rho, "erodibility": k, "diffusivity": d, "vanilla_fallback": vf,
            "stone_use": f"judgment (P1-28, Ben's hardness tiers 2026-10-01): {STONE_USE[n][1]}"}


def document(r, groups):
    doc = {
        "class": r["name"],
        "family": r["family"],
        "groups": class_groups(r["name"], groups),
        "density_kg_m3": r["rho"],
        "erodibility": r["k"],
        "diffusivity": r["d"],
        "metamorphic_path": list(r["path"]),
    }
    if r["wx"] is not None:
        doc["weathers_to"] = r["wx"]
    if r["name"] in LITHIFIES_TO:
        doc["lithifies_to"] = LITHIFIES_TO[r["name"]]
    if r["name"] in LOOSE_EQUIVALENT:
        doc["loose_equivalent"] = LOOSE_EQUIVALENT[r["name"]]
    if r["facies"] is not None:
        doc["facies_rule"] = r["facies"]
    doc["tags"] = list(r["tags"])
    doc["vanilla_fallback"] = r["vf"]
    doc["stone_use"] = STONE_USE[r["name"]][0]
    doc["sources"] = sources(r)
    doc["notes"] = r["notes"]
    return doc


def main():
    groups = expand_groups()
    validate(groups)
    docs = [document(r, groups) for r in CLASSES]
    os.makedirs(OUT, exist_ok=True)
    for stale in glob.glob(os.path.join(OUT, "*.json")):
        os.remove(stale)
    for doc in docs:
        with open(os.path.join(OUT, doc["class"] + ".json"), "w", encoding="utf-8") as f:
            f.write(json.dumps(doc, indent=2, ensure_ascii=False))
            f.write("\n")
    print(f"wrote {len(docs)} rock classes to {os.path.relpath(OUT, REPO)}")
    write_stone_tags()


def write_stone_tags():
    """data/minecraft/tags/item/stone_{tool,crafting}_materials.json from the classes' stone_use."""
    tiers, shared = stone_items()
    tool = [i for i, t in tiers.items() if t == "tool" and i not in VANILLA_STONE_ITEMS]
    build = [i for i, t in tiers.items() if t == "build"]
    by_tier = {t: sorted(n for n, (u, _) in STONE_USE.items() if u == t) for t in STONE_TIERS}
    note = ("generated by tools/dev/gen_rock_classes.py from each rock class's stone_use (P1-28); "
            "replace is false, so packs add their own stones")
    tags = {
        "stone_tool_materials": item_tag(
            note + ". Hard rock classes (vanilla already lists cobblestone, cobbled deepslate and blackstone): " + ", ".join(by_tier["tool"]), tool),
        "stone_crafting_materials": item_tag(
            note + ". The tool items, plus solid rock that is fit for building but not for tools: "
            + ", ".join(by_tier["build"]), tool + build),
    }
    folder = os.path.join(DATA, "minecraft", "tags", "item")
    os.makedirs(folder, exist_ok=True)
    for name, tag in tags.items():
        with open(os.path.join(folder, name + ".json"), "w", encoding="utf-8") as f:
            f.write(json.dumps(tag, indent=2, ensure_ascii=False))
            f.write("\n")
    print(f"wrote {len(tags)} item tags to {os.path.relpath(folder, REPO)}: "
          f"{len(tool)} tool items, {len(build)} building-only items")
    print("stone_use: " + "; ".join(f"{t} {len(by_tier[t])}" for t in reversed(STONE_TIERS)))
    for item, by in sorted(shared.items()):
        print(f"  shared item {item} is {tiers.get(item, 'none')}: "
              + ", ".join(f"{n} ({t})" for n, t in sorted(by.items())))
    vf = {r["name"]: r["vf"] for r in CLASSES}
    riders = [n for n in by_tier["tool"]
              if not any(tiers.get(b) == "tool" for b in block_members(n) + [vf[n]])]
    print("hard classes whose vanilla-only blocks are not tool-grade: " + ", ".join(riders))


if __name__ == "__main__":
    main()
