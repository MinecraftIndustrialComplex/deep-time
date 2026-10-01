# What a professional geologist says about game geology: Gneiss Name's videos, mapped to Deep Time

Research note, 2026-10-01. Not a decision; nothing here changes the plan until Ben says so.

**Method.** The channel was found by web search. English captions were pulled with
`yt-dlp --skip-download --write-auto-subs --write-subs` (no video or audio downloaded). Fifteen
videos were read in full. Most of his own captions are human-quality; two older ones (Sediment 2022,
Iron Ore 2024) are visibly machine-translated and garble words ("tough" = tuff, "Archaic" = Archean,
"Nice" = Gneiss). Those are corrected below where the meaning is clear and flagged where it is not.

**Reading the labels.** **His:** is what he says or clearly implies (paraphrased; short quoted
phrases carry a rough timestamp). **Inference:** is mine, about Deep Time. Where he gives a number it
is his, from the captions, not independently checked.

---

## 1. Channel and video list

**Channel.** Gneiss Name, `@gneissname` (https://www.youtube.com/@gneissname), site gneiss.name.
Confirmed with Ben. The speaker is a working geologist (analytical geology at a university; electron
microscopes and mass specs; field work in Michigan, Australia, Madagascar) who plays Minecraft in
vanilla with command blocks, data packs and display entities. In a Feb 2026 video he says he is
"over 256" thousand subscribers. His build for the long-running series is a downloadable
vanilla-Minecraft "Geology of Minecraft" world, plus a Geologic Timeline Project where one block is
one million years (4,567 blocks for Earth's history, with dioramas per era).

Everything of his that is about game geology, newest first. Dates are upload dates.

| # | Title | URL | Date | Why it matters to us |
|---|---|---|---|---|
| 1 | Geologic Review: Vintage Story, Part 1 (rocks, sediment, soils, world gen) | https://www.youtube.com/watch?v=b4e7mTRB7qg | 2025-12-15 | The richest source: a full critique of province-based strata |
| 2 | Geologic Review: Vintage Story, Part 2 (ores, structures, caves, prospecting) | https://www.youtube.com/watch?v=2D8y68Zf7Uo | 2026-03-25 | Ore settings, prospecting, salt domes, kimberlite, craters, cave-ins |
| 3 | Geologic Review: Hytale, Early Access | https://www.youtube.com/watch?v=qnnEknpbAB0 | 2026-01-26 | Strata by zone, cliffs, glaciers, rivers, rock inventory |
| 4 | Real mountains in Minecraft | https://www.youtube.com/watch?v=PCZyPyAwQ5E | 2025-08-20 | Mountain types, scale, slope, volcanoes |
| 5 | Geology of Minecraft: Iron Ore, Facts and Fiction | https://www.youtube.com/watch?v=OfHLsqaHJoo | 2024-12-12 | Ore distribution vs reality, iron "veins", banded iron formations |
| 6 | How does Minecraft ore compare to Earth | https://www.youtube.com/watch?v=Id0yCfaoDsw | 2024-01-01 | Ore abundance vs real annual production, definition of ore |
| 7 | Geology of Minecraft: Caves! | https://www.youtube.com/watch?v=1tQumBj-lSE | 2023-04-25 | Cave types, speleothems, karst, water table |
| 8 | Geology of Minecraft: Igneous Rocks | https://www.youtube.com/watch?v=H5AVWM-T054 | 2022-07-30 | Igneous blobs, tuff misplacement, cooling and crystal size |
| 9 | Geology of Minecraft: Episode 5, Sediment | https://www.youtube.com/watch?v=0MwsU3CeRaM | 2022-02-05 | Grain sizes, sorting by transport (captions garbled) |
| 10 | Sulfur Caves, A Quick Look (1.21.x snapshot) | https://www.youtube.com/watch?v=Col0ApUB68E | 2026-04-10 | Minecraft's new biome vs real sulfur and thermal systems |
| 11 | Minecraft Maps Are Strange | https://www.youtube.com/watch?v=4dEx8D1mjso | 2025-10-01 | Hillshade and topo-line mapping (a little relevant) |
| 12 | Geologic Review: Ruri Rocks S1 (anime) | https://www.youtube.com/watch?v=qjtzBLA24aI | 2026-06-25 | One relevant point: no geologic maps shown |
| 13 | Minecraft Water is Strange | https://www.youtube.com/watch?v=em2Y4Rm2yiM | 2026-02-23 | Rendering internals only; no geology (read, nothing used) |

Appearances on other channels (counted, because they carry his views):

| Title | Channel | URL | Date | Note |
|---|---|---|---|---|
| Fixing Minecraft with Real Geology, ft. Gneiss Name | Amos Redstone | https://www.youtube.com/watch?v=AwkgAM9c7M8 | 2024-11-22 | He is the guest for the first third (sand, gravel, soil, weathering); the later proposals (freeze-thaw, wind, chemical weathering mechanics) are Amos's, "consulting on the ideas" |
| Talking Geology, Science Communication, and Minecraft with @gneissname | Quantum Mic | https://www.youtube.com/watch?v=pRxXRb1kRGI | 2026-01-02 | Interview; background, plus his number-one misunderstood concept: scale of geologic time |

**Not retrieved.** "Geology of Minecraft: Coal and Diamonds"
(https://www.youtube.com/watch?v=JivStfUWOwo). YouTube answered the subtitle request with HTTP 429 on
three attempts spaced a minute apart while every other video succeeded. The interview (above) says
the video compares how long Earth took to make its coal to how fast we burn it, and notes about half
of Earth's total coal production is already used. Re-run later with
`nix run nixpkgs#yt-dlp -- --skip-download --write-auto-subs --sub-langs en <url>` to complete it.
Also not read: the short or older episodes (Geologic Materials 1.1, Stone, Tuff Talk, "Why care about
Geology") and the Minecraft Education / color-theory videos.

**Other games he has covered.** In the channel list I could see (about 80 uploads, not exhaustive)
the only games reviewed for geology are Vintage Story (2 parts), Hytale, and the Minecraft sulfur
caves snapshot. He says in Part 2 that he has "zero experience" with TerraFirmaCraft; no Dwarf
Fortress, Valheim, or others appear. A search for a different channel ("The Minecraft Historian",
"The Geologic History of Minecraft") turned up a student's cross-section interpretation that only
mentions Gneiss Name in passing; it is not him and is not used here (one hearsay line: that student
says Gneiss Name avoids doing a "geologic history of Minecraft" because of sealed dirt pockets,
which matches his own gripes below).

---

## 2. His stance, in short

**His:** he does not want a one-to-one recreation of geology. "A little inconsistency with reality
is fine or even good" (VS1 0:00-1:30); real life can be boring and a game's job is to make it
interesting; he repeatedly calls many game choices fine "for gameplay balance and diversity". What
he objects to most is geology that is *incoherent* (rock where it could not be, features that
contradict their own labels), not simplification. He rates a game by whether its geology tells a
consistent story a geologist could read. His strongest praise goes to Vintage Story: "an incredible
job representing geology and the mechanics surrounding it" (VS2 54:17), with provinces, soils, sediment
variety, mantle, panning/prospecting, cave-ins.

**Inference:** Deep Time is the thing he keeps wishing these games were: strata come from a simulated
history, so most of his *structural* complaints (rock where it should not be, draped layers, no
rivers) are impossible by construction. What it will not get for free is the *decorative* layer on
top of the simulation (cave contents, soil, indicator features, small structures, block-level
detail), which is where his critique now lands. Section 5 is that gap.

---

## 3. Recurring criticisms and praise, by theme

### 3.1 Rock types and where they occur

- **His, criticism (all games):** the supply of igneous and metamorphic rocks is thin. Hytale has
  basalt as its only named igneous rock (no granite, diorite, gabbro, andesite, obsidian, phonolite,
  trachyte; HT 19:50). Vintage Story lacks diorite, dacite, granodiorite and most plutonic rocks, has only slate and phyllite
  from the shale-to-gneiss series (no schist or gneiss), and tuff/scoria exist only in creative (VS1
  8:02-10:57, 18:32). Minecraft lacks everything above slate.
- **His, praise:** Vintage Story's sedimentary coverage is "exceptionally good" (limestone, chalk,
  bauxite, evaporites, plus conglomerate, shale, claystone, chert); its sediment ladder (cracked
  rock, boulders, stones, gravel, sand), and different sand textures for marine and land settings
  ("zero notes", VS1 13:38). He singles out kimberlite as "a great inclusion" and the meteorite
  rock suevite as a delight.
- **His, criticism (Minecraft):** tuff appears as lens-shaped blobs deep in deepslate and in
  "worm-like" iron-vein shapes. Tuff is volcanic ash: surface layers near volcanoes, not buried
  conduits (Igneous 3:52; Iron 5:12-5:54). Minecraft's granite, andesite and diorite all sit above
  y 0 as identical lens ("American football") shaped blobs, and below y 0 the same shapes are tuff
  (Igneous 2:21-4:27). He accepts this "for gameplay", as humans need access to the blocks.
- **His, crafting recipes:** granite from diorite plus quartz, etc. "doesn't make sense in real
  life" but "there is a little bit of truth" (granite is quartz-rich, Igneous 5:31).
- **His, texture requests:** shale and phyllite textures should show visible layering (foliation);
  conglomerate should show clasts; claystone should be smooth (VS1 14:25-14:50, 18:32). Kimberlite
  has one texture and tiles badly (VS2 39:27). Ore textures are the same for every host rock and
  every grade, and zinc on granite is hard to see (VS2 14:57, 18:26).
- **His, bedrock:** in real geology "bedrock" is the first solid rock under you, not an
  impervious floor. Vintage Story calls its bottom "mantle", which he scores +10 and calculates is not
  far from correct if the crust is scaled with the planet (VS1 5:42-6:07).

### 3.2 Strata and layering

- **His, the core complaint (VS1 22:59-32:30):** Vintage Story's six USGS-style provinces
  (shield, platform, orogenic, basin, large igneous, extended crust) are a good idea, but layers are
  placed "top down" so sedimentary rock is draped over underlying topography. Real sediments fill
  low areas as roughly horizontal sheets; the way it is drawn looks as if folded rock was eroded flat.
  Basalt floods are the same: they should fill lows, not drape. He suggests a second, hidden
  "subsurface topography" noise map (seed + 1) for the igneous floor, many more thin sedimentary
  layers, and not entirely horizontal ones.
- **His, composition tweaks:** shield = felsic plutonic only (his idea: pink, grey and white granite
  variants); no metamorphics in platforms; orogenic provinces should be metamorphic-rich (10 to 50
  layers, sedimentary down to 10); a landform generated before the province can put a sinkhole or a
  limestone feature in a shield (fixable by swapping the landform).
- **His, Minecraft:** "stone everywhere with patches and then a horizontal line separating the stone
  from the deep shale" (the captions' wording for deepslate). He wants rock type to vary by biome and
  depth (Iron 21:17-21:35).
- **His, Hytale:** praises that each zone has its own deep stratigraphy, with a basalt sill layer,
  "volcanic rock" gabbro-like roof, shale on top in the mountain zone and a red/white sandstone
  stack in the desert: "not just a surface coating" (HT 15:29). Criticises the straight, unblended
  seams where zones meet; caves dead-end at the boundary (HT 7:36).
- **His, sediment sorting (2022):** sediment is sorted by transport: gravel drops out first at
  bends, then sand, then clay in still water; grains round with distance (Sediment 7:04-11:29). That is
  why clay, sand and gravel sit in rivers and swamps. (Garbled captions; the content is standard
  sedimentology.)

### 3.3 Ore deposits and their settings

- **His, definition:** an ore is rock that holds a wanted element at high enough concentration to pay
  for mining and refining, and that changes with demand (Ore 1:18-2:04). In a game he is fine with
  "if the player is mining it, it's an ore".
- **His, criticism of Minecraft 1.18 distribution:** depth histograms (coal peaks y96, copper 48, iron
  16, lapis 0, gold -16, diamond and redstone deepen) are a game-design curve; real ores do not
  depend on depth (Ore 2:46-3:11). Iron generates in three overlapping groups (small veins to y72,
  large at y 80-384 peaking at 232, large at -4 to 56 peaking at 16) and 62 percent of the iron sits
  above y 70 on mountains; "real mountains don't have more iron as you climb" (Iron 4:08).
  "Fine, for game design".
- **His, scale of abundance:** the 128x128 excavation gave about 2x Earth's whole annual gold
  production, copper filled the hole from -58 to 100, diamonds were about 75 years of world production
  (Ore 8:57-14:19). The annual iron mined would be a cube hundreds of blocks on a side; coal more than
  1,800. **Inference:** a good argument for Deep Time's finite, tonnage-sized sites (decisions log
  2026-09-28 on COE vein size) when tonnage translates to blocks.
- **His, iron "veins":** the big noise-shaped iron structures in deepslate are filled with tuff and
  raw iron. "Vein" has a specific meaning (planar crack filled by fluid-grown minerals; amethyst-style
  growth mirrors) and those are not veins. If they are magma conduits, then use smooth basalt, or add
  an ultramafic rock (dunite, harzburgite); anything except tuff (Iron 5:12-8:42). Calcite veins in
  Minecraft are closer to the real thing.
- **His, real iron:** it comes from banded iron formations (BIF): 1.5-3.8 Ga, none forming today,
  thousands of metres thick, product of cyanobacterial oxygen rusting dissolved iron out of the
  oceans for about 2 billion years; peak 2.5 Ga (Iron 14:18-20:57). Not meant for the game "unless
  they were only found in a rare biome, maybe an ancient taiga". There is no native iron; meteorites
  are iron-nickel (HT 25:20). He suggests limonite only near the surface and under peat (bog iron) as
  in VS (VS2 24:08), and would add an optional steel/iron split.
- **His, Vintage Story ore specifics (VS2 11:03-31:42):** praises that grades (poor to bountiful) exist,
  that native copper occurs in basalt (highest grade) and malachite in carbonates, tin in granite
  (pegmatites), flint nodules in chalk/limestone and fire clay under coal measures. Complains of ores
  placed in rocks where they do not form: bismuth in igneous and metamorphic rocks, nickel (pentlandite) in
  any igneous rock when it should be peridotite and massive-sulfide settings, ilmenite in metamorphic rock,
  cinnabar in metamorphics, galena with the highest grades in chert/shale, emerald in limestone
  (should be pegmatites), olivine in vugs ("green quartz"), quartz-vug size (fist-sized in life).
  He wants sulfur at volcanic vents (extended crust, large igneous provinces), lapis lazuli with
  contact metamorphism, bauxite in hot humid regions, uranium graded toward sedimentary rocks, titanium and tin placers,
  limonite as bog iron. Most ore in real life is placer or hydrothermal; "most of Earth's economic
  deposits are placer deposits" (VS2 16:58).
- **His, mining returns:** you get the ore but real mining also yields gangue and tailings; ore blocks
  and refined blocks having the same volume ignores volume loss (Ore 5:18). Fortune is "better
  extraction", not more material (Ore 6:00).
- **His, special structures (VS2):** praises salt domes (diapirs, "most surprising"), kimberlites
  (with his fix: overlying rocks flat, pipe running down to the mantle, mostly in shields), meteorite
  craters with suevite and meteoric iron ("I love that these are in the game"), ruins built from the local
  stone, termite mounds made of local bedrock (a fieldwork trick). Salt domes must start in a
  sedimentary layer and grow *up*, not down; salt should be more layered (evaporite beds).

### 3.4 Caves and karst

- **His, Minecraft caves (Caves!):** praises dripstone as real karst vocabulary (stalactites,
  stalagmites, flowstone, "speleothem" is the general term); Lush caves have only a loose analogue
  (cave life is eyeless fish; plants only near entrances) but stepped clay pools resemble shelfstone or
  stepped travertine at a thermal pool (Caves 0:54-2:31); copper in dripstone caves "I can't think of
  anything to justify" (Caves 3:01); noise columns keep ores and dirt in them, he would have replaced
  them with one rock type.
- **His, dripstone growth/mechanics:** game growth is absurdly fast (about 73 m per Minecraft year
  vs a record of 40 mm per year), lava dripping into cauldrons makes new matter and would cool to
  obsidian or basalt, mud-to-clay by dripstone is "a little bit of truth" (Caves 4:34-6:12).
- **His, rock vs cave:** solution caves form in carbonates and evaporites, not slate. Deepslate caves
  (and any cave in non-soluble rock) are inaccurate: "slate... is not dissolvable by a weak acid"
  (Caves 14:20). Yet in VS and Hytale he says caves in every rock are acceptable "because caves are
  fun"; what bothers him more is speleothems of the wrong rock (sandstone or slate stalactites in
  Hytale; any-rock speleothems in VS). Speleothems are almost always carbonate; every speleothem
  should be limestone, which would also ease VS's limestone scarcity (VS2 32:03-34:09).
- **His, big three cave types and ravines:** karst (solution), glacial (meltwater under ice; "ice caves"
  are a different thing), lava tubes. Karst caves form at the water table, in multiple levels as it
  falls, roof collapse makes sinkholes, a mature karst terrain collapses into valleys with the hills
  where there was no cave (Caves 13:27-21:52). A Minecraft ravine is a "narrow valley cut by a river"
  in life; Minecraft's look like opened fractures.
- **His, sulfur caves:** more realistic if only in aquifers (though worse for exploration); real sulfur caves are acid
  (sulfuric) dissolution caves, forming within thousands of years; wall minerals are gypsum rather than
  sulfur or cinnabar; thermal pools are acid and coloured by thermophile bacteria; Minecraft's
  sulfur biome shares only the name (Sulfur 1:09-5:20).
- **His, other cave contents:** praises Vintage Story saltpeter films in caves but notes real saltpeter
  needs bat guano; thermal pools over magma bodies in extended crust (and travertine, new in 1.22);
  suggests temperature rising near pools; underground temperature should be flat for the first
  15 m and then +25 degC per km, while VS does that in about 160 m (VS2 35:15-36:55). Salt-filled
  caves below Y12 are "for some reason". Glow-worm pockets are sealed with no food source.
- **His, sinkhole dirt:** dirt pockets sealed inside rock "just doesn't happen" (hearsay in the
  Minecraft Historian video; his Iron video says "almost as much sense as bags of earth underground").

### 3.5 Mountains and tectonics

- **His, scale:** Minecraft mountains are about 255 m high, a tenth of Australia's highest peak
  and "the world is actually quite flat" at the real 1:10,000 scale (Mountains 0:00-1:30). A real
  226 m hill (Rib Mountain, Wisconsin, 2 Ga quartzite) at 1:1 "doesn't give me mountain vibes": slope
  is too shallow. Minecraft mountains are "incredibly steep compared to real life, but they kind of
  need it to pull off looking like impressive mountains" (Mountains 14:20). He does not want Minecraft
  to match real topography.
- **His, mountain types:** fault block (Tetons, abrupt rise, no foothills, flat sediment fans in front
  from erosion), dome, volcanic (stratovolcano, shield), fold (Himalaya, still growing; top of
  Everest is marine limestone) (Mountains 6:09-11:30). Mountain height has three definitions
  (elevation, distance from core, prominence).
- **His, scale of time:** the interview: "the scale of time" is the most misunderstood idea;
  Appalachians are worn down, Rib Mountain is the heart of a vanished range (Interview 27:16-28:30).
- **His, Hytale:** praises genuine vertical cliffs; says Minecraft "really doesn't" have inland cliffs
  ("almost very gradual transitions"); praises climbing for traversal (HT 18:33-19:24). Mountains with
  marble in them are "cool" because metamorphism accompanies mountain building (HT 6:10).
- **His, VS provinces:** orogenic provinces that are flat plains are possible (worn mountains), but
  mountains sitting on 10 layers of metamorphic rock are thin; the crust bends under mountains
  creating basins (VS1 27:16-28:22). Oceanic crust is missing: he wants the ophiolite sequence (peridotite,
  gabbro, sheeted dikes, pillow basalt, sediment) and mid-ocean ridges, plus diving suits (VS1 30:32).

### 3.6 Rivers and erosion

- **His, rivers:** Vintage Story has none (ponds and rivulets) and the dev told him it is on the
  roadmap; "rivers are a difficult problem when you have no idea where the closest ocean or pond
  will be" and need watersheds (VS1 33:01). Hytale has no true rivers but he rates its sand bars,
  meanders, dry river beds that connect across zones, and rivers running under glaciers
  (HT 17:23-18:33).
- **His, sediment and weathering (Amos collab and Sediment):** clay is the end of a weathering
  ladder; he would like soil-by-biome types; Minecraft's moss and mud/clay are real weathering
  analogs. A freeze-thaw ladder to gravel, wind to sand, chemical weathering to dirt were Amos's mechanics
  (Fixing 5:34-10:19).
- **His, glaciers and rivers:** "you can actually make out different glaciers and how they merge"
  on his 1:33,000 map (Mountains 10:46); glacier into iceberg breakups in Hytale are "incredibly cool".
- **His, soil:** soils are climate, organics, relief, parent material, time (CORPT); praises VS soils
  (forest floor poor, prairie rich, terra preta must be made) and asks for types by climate and
  underlying rock (VS1 20:24-22:14); Minecraft's dirt is best read as silt; podzol is a real soil
  type (Fixing 2:44).

### 3.7 Volcanoes

- **His:** Minecraft has implicit inactive volcanoes in the central depressions of its mountains,
  and he wants "a special biome with a big lava pool in the middle" (Mountains 8:55-9:16). Stratovolcanoes
  are andesitic, steep and part of arcs; shield volcanoes are basaltic, low and wide (Mountains 11:44).
  Ferns are among the first plants after eruptions.
- **His, lava interactions:** lava over water makes basalt (stone/cobble) in VS and Minecraft; he
  says both results should really be obsidian or volcanic glass, and that obsidian has granite's
  chemistry, basalt's is very different (VS2 49:52; HT 11:35). Lava tubes appear as a cave type but he does not see them implemented.
- **His, Hytale volcano zone:** basalt as sill, "cooled magma" replacing obsidian is a fine idea
  ("cooled lava" would be a better name); volcanoes that open into the magma chamber are "totally
  unrealistic but very cool" (HT 13:41-15:06). Ash should become actual tuff.

### 3.8 Scale and time

- **His:** games compress scale. He builds 1:1 mountains in 2,048-high custom dimensions to see what
  real relief looks like (Mountains 5:00). Vintage
  Story's equator-to-pole distance is 10,000 blocks vs 10 million metres, a 1,000x compression, and
  its mantle would be 500 blocks down if scaled (VS1 5:42).
- **His, geologic time:** a block = 1 Myr timeline; geologists' "slow" is millions of years, and
  granite crystals take several million years to cool (Igneous 8:27-10:02). Interview: scale of time is
  his number-one misunderstood concept, and he wants a video that counts uranium atoms per block of granite.
- **His, comparisons are the hook:** he makes geology tangible by scaling real quantities to blocks
  (annual iron as a 588-block cube; copper filling the hole; Washington Monument at 1:1).

### 3.9 Biomes versus geology

- **His:** biomes should respond to the rock below (VS1 34:52: "being pedantic, I would add some
  input from the lithic material below it"). Hytale's "fens" imply carbonate bedrock under wetlands
  (HT 4:50) so the stone there reads as dolostone/limestone; he explicitly wants Hytale's nondescript
  "stone" renamed dolostone. Vintage Story's biome system (each plant has temperature, rain, forest
  ranges; no hard biome lines) is praised, with a wish for more mid-sized plants.
- **His, Hytale surface features:** "they don't really connect into the underlying geology" (chalk cliffs
  that sit on top of terrain with grass under them, HT 5:52). Zone boundaries are sharp lines.

### 3.10 Gameplay

- **His, prospecting (VS2 41:55-47:09):** panning in VS ignores where you are (real gold pans come from
  upstream sources); prospecting pick density mode reports the noise map value at a spot, not
  presence, so results confuse; node search gives the exact six-block neighbourhood. He wants a middle
  mode ("fine, medium, coarse") that reports actual presence in the current chunk or similar, and a
  map shortcut for every ore. "I am all for rarity... but it could be very frustrating" if you cannot find
  something even when you understand the system. Modern real prospecting is regional geology first,
  then geophysics, then sampling, then drilling.
- **His, tool tiers:** ore tier requirements have no relation to mineral hardness; "that's just there
  for gameplay progression". Rock hardness would dominate. Cave-ins with rock strength are impressive;
  he would re-rank them: sedimentary and foliated low-grade metamorphic weakest, then limestone,
  marble, fine then coarse igneous, with chert probably strongest.
- **His, progression:** "You can't have people... grab some bog iron clay, and have iron tools in no
  time" (VS2 54:17), the tech-tier problem; likes the in-game guide ("you shouldn't need a wiki").
- **His, outcrop marking:** Hytale marks cave entrances in deserts with stone outcrops (HT 6:50).
- **His, geologic maps:** Ruri Rocks shows plenty of topographic maps but "they don't talk about or show a
  single geologic map in the entire season"; rough geologic maps exist for the whole Earth and are
  where you start (Ruri 8:40-9:00). In the Maps video he explains hillshaded relief with a
  north-west light source and notes Zelda uses elevation tint plus topo lines.
- **His, other:** liking rocks is "not childish"; collect responsibly. Not game-relevant.

---

## 4. Mapping to Deep Time

Status key: **Does** = built (task id where known). **Planned** = in the plan/STATUS task list.
**Not planned** = no mention found in the plan, STATUS or decisions (checked with keyword searches,
not a full read). **Pack** = belongs to MIC or the pack, since Deep Time ships no blocks of its own.
Statuses are *inference*; the points are **His**.

| # | His point | Source | Deep Time | Notes |
|---|---|---|---|---|
| 1 | Layers should fill topography as sheets, not drape over it | VS1 25:11-26:53 | **Does** (P1-04, P1-07, P1-08), **Planned** (P2-01) | Strata log is a depositional record on the simulated surface; folds, faults and unconformities come with the Noddy evaluator (P2-01). A depositional column in a basin and a thin one on a shield falls out of the simulation |
| 2 | Provinces (shield, platform, orogen, basin, LIP, extended) drive rock | VS1 22:59 | **Does** (P1-03, P1-07, P1-27), **Planned** (P1-10) | `meta/province` exists; P1-10 maps USGS provinces to TerraBlender regions. Flood-basalt layer is in P1-21 |
| 3 | A stone/deepslate horizontal line is boring; vary rock by biome and depth | Iron 21:17 | **Does** (P1-07, P1-12) | 60 rock classes from simulation; vanilla blobs removed (Q4). Deepslate stays only as "unclassified" |
| 4 | More igneous and metamorphic types (gneiss, schist, eclogite, migmatite, etc.) | VS1, HT 19:50 | **Does** | Rock-tag vocabulary has gneiss, schist, blueschist, greenstone, eclogite, amphibolite, hornfels, skarn, quartzite, kimberlite, carbonatite, pegmatite, saprolite, laterite, bauxite, loess, gypsum, halite. Block variety depends on the pack's stand-ins |
| 5 | Tuff is ash: surface layers near volcanoes, not buried blobs | Igneous, Iron | **Does** (Q4 removes tuff blobs; P1-21 volcanoes) | Whether the simulation places explicit ash-fall (tephra) layers near volcanoes is not clear in the plan; grep found no "tephra"/"ash layer". See idea 9 |
| 6 | Granite/andesite/diorite as identical lenses | Igneous 2:21 | **Does** (P1-07 plutons as bodies), **Planned** (P2-01) | Plutons are ellipsoid bodies under arcs; exhumed by erosion. No lens clones |
| 7 | Sealed dirt/gravel/clay pockets in rock make no sense | Iron 6:14; hearsay | **Not planned** (the opposite: §7.7 keeps loose pockets) | The plan keeps vanilla dirt/gravel/clay pockets. See idea 2 |
| 8 | Ores must sit in the right host rock, not depth curves | Ore 2:46; VS2 | **Does** (P1-14, P1-15) | Mineral-systems engine: favourability from fields, host rock via classes, USGS sizes. 1.18 depth histograms replaced by sites; vanilla ore veins off |
| 9 | Banded iron formations, 2.5 Ga, none today | Iron 14:18; Interview | **Does** (P1-14) | BIF family with a secular clock (2.65-1.85 Ga acme; inherited cratonic basement if timeline starts later). Needs a long enough timeline |
| 10 | Placers come from an upstream source; panning should use that | VS2 16:58, 42:18 | **Does** (P1-14 placer children along drainage), **Pack** (panning) | Site registry has `parent` for placer to source |
| 11 | Supergene, gossan, weathered caps | VS2 24:08 | **Does** (P1-14 supergene; gossan in geometry), **Planned** (P2-05) | |
| 12 | Kimberlite pipes: flat overlying rock, deep pipe, shield only | VS2 39:27 | **Planned** (P2-10 kimberlite diamonds) | His rules (cratons, pipe to depth, overlying strata not draped) are good acceptance tests |
| 13 | Salt diapirs rise up from a source layer | VS2 38:03 | **Planned** (P2-11) | Plus oil traps; halite and gypsum classes exist |
| 14 | Meteorite craters with suevite and meteoric iron | VS2 40:10 | **Planned** (P2-08) | Plan has breccia and impact melt; no `suevite` class in the 60-class list (checked); see idea 9 |
| 15 | Bauxite/laterite in hot humid regions | VS2 50:55 | **Planned** (P2-05, P2-10) | Classes exist now |
| 16 | Volcanoes: stratovolcano with caldera, shield, flood basalt, ferns | Mountains 8:25-12:02 | **Does** (P1-21) | Arc stratovolcanoes, calderas, hotspot shields, flood basalt layer; "ferns first" is biome-level (P1-09) |
| 17 | Fold, fault-block, volcanic, dome mountains | Mountains 6:09 | **Does** (folds via collisions P1-03/P1-27, volcanoes P1-21), **Planned** (P2-09 grabens/fault scarps) | Dome/laccolith mountains not named; erosion of a pluton could produce one |
| 18 | Foothills vs abrupt fronts (Tetons) | Mountains 6:13 | **Does** (erosion sim P1-04/P1-06) | Fault-block fronts need P2-09 fault scarps |
| 19 | Old ranges worn down (Appalachians, Rib Mountain) | Interview 27:16 | **Does** (P1-03, P1-04, P1-17 hypsometry) | Long timeline + erosion; calibration is open (P1-17) |
| 20 | Minecraft mountains are steep; that is OK | Mountains 14:20 | **Does** | 15x vertical exaggeration is by design (300 m / 20 m per block); drama slider (P1-06) |
| 21 | Real vertical cliffs are missing from Minecraft | HT 18:33 | **Does** (P1-06 cliff bands on hard rock), **Planned** (P2: crags/overhangs) | Talus/scree deferred (STATUS P1-06) |
| 22 | Rivers need watersheds | VS1 33:01 | **Does** (P1-22 rivers and lakes) | Hydrology routed on the simulated surface |
| 23 | Dry river beds, meanders, sand bars, rivers under glaciers | HT 17:23 | **Planned** (P2-03: meanders, deltas, evaporation-limited lakes), **Not planned** (dry wadis) | Dry beds would fall out of discharge < some threshold; see idea 12 |
| 24 | Glaciers visible and flowing; iceberg breakup | Mountains 10:46; HT 8:41 | **Does** (P1-05 ice sheets, ice branch), glacial erosion till class | Calving icebergs are a pack/biome matter |
| 25 | Caves should be karst in carbonates, at the water table, multi-level | Caves 13:27-21:52 | **Planned** (P2-04 Karst-Synthesis port; lithology and water table) | Vanilla caves kept as baseline so every region stays playable (matches his "caves are fun") |
| 26 | Speleothems only in carbonate caves; all of them limestone | VS2 32:03; HT 19:24 | **Not planned** | See idea 1 |
| 27 | Lava tubes, glacial caves, sinkholes/cenotes, karst towers, flooded cave levels | Caves 14:48-21:52 | **Not planned** (P2-04 is karst only) | See idea 6 |
| 28 | No hard seams between zones: caves and strata should continue across boundaries | HT 7:36 | **Does** (single continuous simulation, no zones) | Caves use one noise basis; seam handled by Toroidal World (P0-06) |
| 29 | Sulfur caves are acid dissolution caves, gypsum minerals | Sulfur 1:09 | **Not planned** | Mojang's new biome; at most a pack concern |
| 30 | Hot springs, travertine, thermal pools over magma, sulfur and cinnabar at vents | VS2 35:36; Sulfur | **Not planned** | See idea 5 |
| 31 | Temperature with depth: flat 15 m, +25 degC/km | VS2 36:55 | **Not planned** (heat flow exists in metamorphism, P1-07) | Could feed mic-climate's underground temperature |
| 32 | Soil from climate + parent rock + relief + time | VS1 20:24; Fixing | **Planned** (P2-05 regolith/saprolite/laterite), partial (loess, regolith thickness) | Soil *type* by parent rock is not named. See idea 4 |
| 33 | Biomes should respond to underlying rock | VS1 34:52; HT 4:50 | **Planned** (P1-10 provinces -> regions), otherwise **Not planned** | Climate-driven biomes today (P1-09). See idea 4 |
| 34 | Ruins/structures built from local stone | VS2 41:15 | **Not planned** (**Pack**) | See idea 8 |
| 35 | Ores in zoned, shaped bodies; grades visible; rich vs poor | VS2 14:57 | **Does** (P1-14 zoned geometry, grade per site), **Pack** (visual grade) | Textures per grade are a resource-pack job |
| 36 | Ore abundance should respect real production scale | Ore 8:57 | **Does** (P1-14 lognormal tonnages, finite) | Block-per-tonne policy is the open lever (COE decision, log 2026-09-28) |
| 37 | Prospecting: presence mode at chunk scale, geophysics, surface indicators | VS2 43:01-47:09 | **Planned** (MIC prospecting plan; `DeepTimeSites` API) | See idea 3 |
| 38 | Rock hardness should govern tier, cave-ins by rock strength | VS2 47:09, 49:09 | **Not planned** (**Pack**) | Deep Time could export a strength tag per class; idea 10 |
| 39 | Mantle/base of crust shown; oceanic crust as ophiolite sequence | VS1 5:42, 30:32 | **Partial** (oceanic crust age and subduction are simulated, P1-03) | Ocean-floor column and base-of-world rock not described. Idea 7 |
| 40 | Geologic maps are the starting point of exploration | Ruri 8:40 | **Does** (viewer provinces view; satellite-style view, log 2026-09-30), **Not planned** (in-game map item; the History Atlas item was dropped) | Idea 11 is a small viewer/BlueMap layer |
| 41 | Weathering ladder cobble > gravel > sand > dirt as mechanics | Fixing 5:34 | **Pack** | Out of scope for a worldgen mod |
| 42 | Native copper in flood basalt, tin in pegmatite granite, emerald in pegmatite | VS2 14:36, 16:39, 30:58 | **Not planned** (copper in basalt, emerald/tin pegmatites) | Plan has pegmatite class but no deposit types for them. Idea 9 |
| 43 | Bog iron / limonite near surface under peat | VS2 24:08 | **Not planned** | Idea 9; interesting for MIC pre-industrial iron |
| 44 | Underclay (fire clay) beneath coal seams | VS2 8:32 | **Not planned** | Tiny addition to the coal facies; idea 9 |
| 45 | Evaporite beds, playa basins, layered salt | VS2 27:02, 37:17 | **Partial** (halite/gypsum/anhydrite classes; evaporites feed MVT brines); P2-09 loess and ergs | Playa/salt-flat landform not named. Idea 9 |
| 46 | Stromatolites, fossil-bearing limestone | Iron 18:17; VS1 15:13 | **Not planned** | A decoration item; low |

---

## 5. New ideas worth adopting, ranked

Ranked by how much geological credibility they buy a player or a geologist versus what they cost.
All are **Inference**, grounded in his points above; sizes are rough (S = days, M = about a week or
two of one engineer, L = more).

1. **Geology-gated cave contents (S-M, on top of P2-04).** Place dripstone/speleothems and flowstone
   only in carbonate or evaporite caves, and let a cave's contents follow its host rock: wet/dry zone
   above and below the P2-04 water table, flooded lower levels, calcite crusts on non-carbonate
   caves. *Why:* it is his single most repeated cave complaint (wrong-rock speleothems), and it is
   the cheap half of a fix he accepts for vanilla's "caves in every rock" (that part stays for
   playability). *Where:* a placed-feature filter driven by the column's rock class, using the pack's
   dripstone.
2. **Make underground "loose" pockets geological or remove them (S).** Decide on the vanilla dirt,
   gravel and clay blobs that §7.7 currently keeps: keep them only as sinkhole fills in karst,
   paleosol or regolith beds at unconformities, and gravel/clay lenses inside sedimentary strata;
   drop them in crystalline rock. *Why:* "bags of earth underground" is a recurring gripe and costs
   almost nothing to fix once host rock is known.
3. **Feed the prospecting plan from his critique (M, MIC side).** Give prospectors three tiers:
   regional (province/geology map), chunk-scale presence ("fine, medium, coarse"), and local; tie
   panning to the real upstream placer source (the site `parent` link); expose surface indicators from
   P1-14's geometry (gossan caps, alteration halos, iron stain, outcrop marks like Hytale's cave
   outcrops); give rich/poor grades visibly distinct textures (pack). *Why:* the one gameplay
   system he critiqued in detail, and Deep Time already stores the data.
4. **Lithology-aware soils and biome nudges (M).** Soil type from climate, parent rock, relief and age
   (his CORPT framing): e.g. podzol/coarse dirt on sandstone and granite, red laterite soils on warm
   wet old surfaces, loess on dust deposits, calcareous grassland and karst pavement on limestone,
   serpentine barrens on peridotite, fens over carbonate. *Why:* his "biomes should respond to rock"
   point (VS1, Hytale fens). *Where:* P2-05 weathering blocks plus P1-10 region steering; needs the
   biome/surface-rule side of the generator, not new blocks.
5. **Geothermal and hydrothermal surface features (M).** A heat-flow raster (already used for
   metamorphism) marks rifts and young volcanic fields; place hot springs and travertine terraces,
   fumaroles with native sulfur, cinnabar in epithermal settings; a warm-ground temperature signal for
   mic_climate. *Why:* he lists these as real, rare and distinctive, and a pack with temperature mods
   (mic-climate connector) can use them as warm refuges. Ties to P2-10's epithermal family.
6. **More cave kinds: lava tubes, sinkholes and cenotes, karst towers, glacial caves (M each).** Lava
   tubes in the flood-basalt layer and young basaltic shields (P1-21 provides the flows), sinkholes
   where P2-04 produces a weak roof, karst towers or cockpit terrain where carbonate and climate favour,
   glacial caves under P1-05 ice. *Why:* his "big three" caves, and sinkholes are his own childhood
   example; they are the visible payoff of the karst port.
7. **Ocean-floor column and base-of-world rock (S-M).** Under oceanic crust: sediment, pillow
   basalt, sheeted dikes (basalt), gabbro, peridotite; and a bottom layer that reads as the top of the
   mantle instead of an arbitrary stone. *Why:* his one missing province in VS1 and a "+10 points" for
   showing the mantle. Check first what P1-07 already puts under oceanic crust (class list has all
   the rocks).
8. **Local-stone hook for structures (S in Deep Time, M in the pack).** Expose the rock class (and a
   simple "nearest outcrop" lookup) so the pack's villages and ruins use the local bedrock or stone
   from the region, not a biome palette. *Why:* he calls this a "nice touch" in VS and uses it as a field
   tool. Deep Time would just provide `rockAt()` in the Java API; the placement change is a MIC mixin.
9. **Data-only backlog of small rocks and deposit types (S each).** All are JSON deposit types or
   rock classes riding existing machinery: `suevite`/impactite and `travertine` classes; ash-fall
   (tephra) tuff layers around the P1-21 volcanoes; native copper in flood basalt (Keweenaw type); pegmatite
   tin and beryl/emerald; magmatic Ni-Cu sulfide (Eagle-type) and peridotite nickel; bog iron/limonite
   under peat; underclay beneath coal; playa/evaporite lake beds with layered salt; sulfur at volcanic
   vents. *Why:* each is a place he named where VS gets the host wrong and a geologist would expect it
   right.
10. **Rock-strength tag (S).** A per-class `deeptime:strength` table (sedimentary and foliated
    metamorphics weak, chert and fine-grained igneous strong) for pack mechanics such as cave-ins and
    mining time. His ranking in VS2 49:09 is a reasonable seed. *Why:* he wants gameplay hardness to follow
    rock, not ore tier. Low priority; pure data.
11. **Geologic map layer (S).** Add a lithology/province map to the HTML viewer and a BlueMap overlay, in the
    USGS colours the VS noise map already uses. Do not add an in-game item (the History Atlas is
    out by decision). *Why:* his "geologists start from a geologic map" point; also a good
    verification view for P1-23's artifact audit.
12. **Ephemeral rivers and dry beds (S-M, with P2-03).** Where discharge is intermittent (arid
    climate, high infiltration), render the channel dry with a sandy floor and sand bars. *Why:* he
    praises it in Hytale; P1-22 already knows discharge and P1-26 will know seasons.
13. **Comparison scaling for the player (S).** In `/deeptime why` or the viewer, convert a site's
    tonnage to "years of world output" or blocks. *Why:* his videos work because of this
    comparison, and it communicates why Deep Time's deposits are finite.

---

## 6. Praised in other games: things to borrow

All **His** praise, with the borrowing as **Inference**.

- **Vintage Story: province-keyed strata with a stated rule per province** (VS1 22:59). We already go
  further (simulated); borrow the *visible* provinces as a map and as TerraBlender regions (P1-10),
  and his corrections (flat sheets; felsic shield; metamorphic orogens).
- **Vintage Story: sediment as a full ladder, with marine vs land sand variants.** Borrow as a
  texture/pack matter and as a surface rule: marine sands under shallow seas, land sands on dunes.
- **Vintage Story: soils with a real recipe (CORPT), plus player-made terra preta.** Borrow soil
  from rock + climate (idea 4).
- **Vintage Story: thermal pools over magma; saltpeter films in caves; salt domes; kimberlite pipes;
  meteor craters.** Borrow as P2-08/P2-10/P2-11 plus idea 5; each is a site-registry feature with a
  "Why here?" chain, which is richer than VS's noise maps.
- **Vintage Story: ruins built from local stone, termite mounds of local bedrock.** Idea 8.
- **Vintage Story: prospecting in two modes (density, node search), in-game guide, cave-ins by rock
  strength, panning by density.** Borrow the split and fix the confusing part he names (presence mode).
- **Vintage Story: gradually thinner coal grades with depth and fire clay beneath coal** (VS2 8:32-10:21).
  Matches our coal-by-burial ranking; add underclay (idea 9).
- **Vintage Story: ore texture overlays, grade shown by name.** Pack matter; he would add grade-specific art.
- **Hytale: deep, zone-specific stratigraphy.** We have that by simulation; borrow *blend* at
  boundaries (which his critique says Hytale lacks) as a test: no sharp seams between provinces.
- **Hytale: real cliffs and climbing; boulder fields; rivers under glaciers; glaciers calving to
  icebergs; dry river beds that connect across zones; outcrops that mark caves; flooded caves with life;
  coral near warm volcanic rock.** Our cliff bands (P1-06), rivers (P1-22) and ice branch cover the
  big three; dry beds (idea 12) and cave marking via surface outcrop (idea 3) are cheap.
- **Hytale: cooled lava replacing obsidian in lava-water interaction.** A pack/mechanics matter.
- **Minecraft: dripstone and mud-to-clay "has a little bit of truth"; stepped clay pools; amethyst
  geodes; calcite veins "closer to a real vein".** Keep these as vanilla flavour; ensure our
  placement reuses them in the right hosts (idea 1).
- **Minecraft (in his opinion): mountain steepness is needed for impressive-looking mountains, and
  height is deliberately limited.** Deep Time's 15x vertical exaggeration is the principled version
  of the same trade.

---

## 7. Caveats

- This is one person's opinion (a good one). He disagrees with himself in places ("caves in any rock
  are fine" and "speleothems of the wrong rock bother me more"). Treat his "fine for gameplay" notes
  as permission, not as bugs to fix.
- Timestamps are from caption lines and can drift a few seconds. Several figures in the Sediment and
  Iron Ore captions are machine-translated and might be off; sedimentology and iron-geology content
  matches standard facts, so I kept it.
- The status column is my reading of the plan, STATUS.md task table and decisions file, with keyword
  searches. "Not planned" means no mention found, not a considered rejection. Check P2-04, P2-08,
  P2-10 and P2-11 text before turning any idea into a task.
- Two YouTube items are not his and are labelled above: the Amos Redstone video (where only the
  first third is his) and the Quantum Mic interview.
- Coal and Diamonds is missing (HTTP 429 on subtitles).
