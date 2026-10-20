# Rock classes, metamorphism and intrusions

Status of this page: `stage 2 (constants checked against sources on 2026-10-02, package WP-7)`. Of the 79 active
rows, 11 are `verified`, 1 `abstract-only`, 9 `cited-unverified`, 1 `calibrated`, 54 `game-choice` and 3 `uncited`.
Where a book the code cites (Winter 2010, Yardley 1989, Telford et al. 1990) could not be read, the row says so and
says what was read instead.

Ledger ids on this page: `RCK-001` to `RCK-090` (see [claims.md](claims.md)); the layer order, the traps and the
checks are on [strata.md](strata.md).

## What you see in the world

Most of the rock under a Deep Time landscape is one of 62 named classes in seven families: intrusive igneous (granite,
granodiorite, diorite, gabbro, anorthosite, peridotite), volcanic, clastic sedimentary, chemical and biogenic
sedimentary, metamorphic, special hosts (kimberlite, carbonatite, pegmatite, greisen and the two impact rocks) and
unconsolidated (soil, sand, gravel and the like). Each is drawn with a Minecraft block: the common grey rocks
(shale, mudstone, coal) as stone, the dark foliated series (slate, phyllite, schist, gneiss) as deepslate, and the rest
as the nearest look-alike (calcite for limestone, granite for granite, tuff for eclogite).

Where plates collided, the buried sediments have been heated and turned into slate, schist, gneiss, marble and
quartzite; on a 500 Myr planet two thirds of all columns hold some of this, and 98 % of the columns on land. Under
volcanic arcs, bodies of granodiorite and granite stand as rounded masses 8 to 80 km long (27 to 270 blocks), 2 to 8
km thick, which erosion exposes as granite country; round them, and round the smaller stocks that rise from them,
lies a rim of hornfels in shale, sandstone or volcanic rock, or of skarn in limestone and dolomite, a few hundred metres
to about 2 km wide (tens of blocks deep, a few blocks wide at 16k scale). Rock laid down after the intrusion is not baked.

## The model in plain words

The structure stage runs each step after the surface stage has laid down or stripped sediment.

1. **Heat.** Every continental cell gets a straight-line temperature profile: the climate's mean surface
   temperature (10 degC if none) plus the plate model's surface heat flow divided by a thermal conductivity of
   2.5 W/m/K, times depth. Heat flow above 120 mW/m2 is clamped, so the steepest gradient is 48 degC/km (RCK-028 to RCK-031).
2. **Burial.** While a cell is in a collision (or has a young Himalayan-type orogen), depths are multiplied by the
   thickening of its crust relative to 35 km, at most 2 (RCK-032 to RCK-035).
3. **Metamorphism.** Each layer is tested at the temperature of its mid-depth. If it has reached the formation
   temperature of the next class on its path (shale to slate at 250 degC, phyllite 350, schist 525, gneiss 650;
   limestone to marble, sandstone to quartzite, basalt to greenstone at 300, to amphibolite at 500) it moves up. Classes
   never go back down, so an exhumed belt keeps its grade (RCK-036 to RCK-046).
4. **Basement.** The plate model's rock pools under the loose cover (sediment, sedimentary, volcanic, metamorphic, plutonic)
   have no burial history, so each is stepped along its path by the present geotherm and split at the depths of the
   isotherms. A column never gets cooler downward: a unit under metamorphosed rock takes at least that grade (RCK-050, RCK-051).
5. **Plutons.** Under arcs, and weakly in collision zones, a Poisson number of plutons is emplaced each step. Each is an
   ellipsoid with a roof 2 to 6 km below the surface, a lognormal length (median 25 km), a width of 0.5 to 1 times that, and a
   thickness from the Cruden-McCaffrey power law. They ride their plate, are unroofed by erosion, and are lost if the oceanic
   crust carrying them is subducted (RCK-056 to RCK-067).
6. **Aureoles.** Every pluton and every stock bakes a shell of its host rock: the contact aureole. Its width is 0.3 of the
   intrusion's smallest dimension (a pluton's thickness; a stock's height or width, whichever is less), 1.5 times that where the
   host at the roof is a carbonate (RCK-053, RCK-091). The shell is read band by band down the column at the intrusion's
   centre: skarn where a band is a carbonate, hornfels where it is other soft rock, nothing in crystalline rock, and only rock
   older than the intrusion is baked, so cover laid on an unroofed pluton stays as it was (RCK-054). Each aureole is a body of
   its own in the world data (role `aureole`), written before the intrusions so the intrusion paints over it; a stock's
   aureole is hollow, cut into slabs that stop at the stock's wall, so the stock itself stays intact. A typical 3 km thick
   pluton has a 900 m aureole, about 26 blocks of height at 16k scale and 3 blocks of width.
7. **Blocks.** Each class resolves to a block through the rock tags of the pack, else a vanilla stand-in (RCK-074 to RCK-084).

## Equations and constants

Geotherm and burial (RCK-028):

T(z) = T_s + (q / k) z beta, with q = min(heat flow, 120 mW/m2), k = 2.5 W/m/K, beta = clamp(crust thickness / 35 km, 1, 2)
in collisions and 1 elsewhere.

Pluton thickness (RCK-061):

T = 0.6 D^0.6, with T in km, D = sqrt(L W) in km the diameter of the circle of the pluton's area, L its length and W its width.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| RCK-001 | densities of 26 classes | granite 2650 ... loess 1400 | kg/m3 | Daly, Manger and Clark 1966; Manger 1963 (ranges) | `verified` |
| RCK-085 | halite, gypsum, anhydrite | 2160, 2310, 2950 | kg/m3 | Handbook of Mineralogy | `verified` |
| RCK-086 | nine classes with a secondary or abstract range | rhyolite 2500 ... carbonatite 2900 | kg/m3 | RocFall/PETLAB table (secondary), USGS abstract | `cited-unverified` |
| RCK-087 | seven classes the sources disagree with | peridotite 3300, kimberlite 2800, ironstone 3000, BIF 3400, phosphorite 2700, pumice 700, travertine 2400 | kg/m3 | judgment | `uncited` |
| RCK-015 | carbonate compensation depth | 4500 | m | secondary: Pacific 4.2-4.5 km | `game-choice` |
| RCK-022 | loose sediment shown to | 150 m depth, 20 Myr age | m; Myr | tectonics.js | `game-choice` |
| RCK-028 | geotherm | T = T_s + q z / k beta | degC | Fourier's law (Hacker notes) | `verified` |
| RCK-029 | thermal conductivity | 2.5 | W/m/K | Robertson 1988: 2.1-4.5 | `uncited` |
| RCK-030 | heat-flow clamp | 120 | mW/m2 | game choice; Davies and Davies 2010 | `game-choice` |
| RCK-033, RCK-034 | reference crust, largest burial factor | 35 km, 2 | km | Christensen and Mooney 1995 abstract: mean 41.1 km | `abstract-only`, `game-choice` |
| RCK-039 | shale to slate | 250 | degC | Winter 2010, Yardley 1989 (not read) | `cited-unverified` |
| RCK-040 | quartzite, marble, greenstone | 300 | degC | Stipp 2002; Ferrill 2004; Greenwood and Berman | `cited-unverified` |
| RCK-041 | phyllite | 350 | degC | not read | `cited-unverified` |
| RCK-042 | schist | 525 | degC | Baxter 2002: garnet zone 500-550; Ben, decisions log 2026-10-02 | `game-choice` |
| RCK-043 | amphibolite | 500 | degC | Nelson; Greenwood and Berman | `cited-unverified` |
| RCK-044 | gneiss | 650 | degC | Baxter 2002: sillimanite zone 650-700 | `cited-unverified` |
| RCK-045 | gypsum to anhydrite | 50 | degC | Orti 2012: about 50 degC at 450-500 m; Hardie 1967: 58 degC pure water | `verified` |
| RCK-049 | diagenesis ends | 200 | degC | Nelson | `verified` |
| RCK-053 | contact aureole width | 0.3 x smallest dimension | - | Annen 2017: linear in thickness; 0.02 to 0.5 in its examples | `game-choice` |
| RCK-054 | skarn on carbonate, hornfels elsewhere, band by band | - | - | Meinert 1992; Nelson | `verified` |
| RCK-091 | aureole in a carbonate host | x 1.5 | - | Alta (Cook and Bowman 1994 via Annen 2017); Brenner 2021 from-memory | `game-choice` |
| RCK-056 | pluton rate | 1e-5 | per km2 per Myr | calibrated to the Coastal Batholith (secondary) | `calibrated` |
| RCK-057 | volume exponent | V proportional to D^2.6 | - | Cruden 2008 (a = 0.6) | `verified` |
| RCK-058 | collision granite rate | 0.3 of arc | - | design; Vigneresse and Burg 2003 | `game-choice` |
| RCK-060 | pluton length | median 25, 8 to 80 | km | design; Tuolumne about 1,100 km2 | `game-choice` |
| RCK-061 | pluton thickness | 0.6 D^0.6 | km | Cruden and McCaffrey 2001 (via Cruden 2008) | `verified` |
| RCK-062 | width over length | 0.5 to 1 | - | design; Sierra data 0.17-0.4 for large plutons | `game-choice` |
| RCK-063 | roof depth | 2 to 6 | km | Buddington 1959 (not read) | `cited-unverified` |
| RCK-064, RCK-065 | rock shares | arcs 70/20/10; island arcs 60/40 | % | design | `game-choice` |
| RCK-090 | impact rock definitions | suevite, impact melt | - | Stoeffler and Grieve 2007; French 1998 | `verified` |

## Earth against the mod

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| Mean continental heat flow | 65 mW/m2 (Pollack et al. 1993); 70.9 mW/m2 (Davies and Davies 2010) | Davies and Davies 2010, table 7 and section 3 (read in full) | gradient 26 degC/km at 65 mW/m2 with k = 2.5; the plate model's continental proxy is 55 + 35 exp(-t/60 Myr) | arithmetic; proxy from the P2-14 build-log entry |
| Gradients by setting | 5-10 K/km cold subduction; 20 K/km craton; about 30 K/km Barrovian; about 50 K/km rifts and arcs | Hacker, UCSB GEO 102C part 3; Nelson, Tulane (lecture notes, secondary) | up to 48 degC/km (the 120 mW/m2 clamp at k 2.5); cold subduction (blueschist, eclogite) is not made | arithmetic |
| Garnet zone | 500-550 degC (Glen Clova) | Baxter et al. 2002, abstract | schist at 525 degC (was 450 until 2026-10-02) | - |
| Sillimanite zone | 650-700 degC, about 6 kbar | Baxter et al. 2002 | gneiss at 650 degC | - |
| Thickness of a 56-km pluton | 4.5 km observed (Lucerne); 6.7 km predicted by the law | Cruden 2008, Fig. 2-4 and eq. 1 | 56 km round: 6.7 km | arithmetic; unit test |
| Thickness of a 4.5-km pluton | 1.6 km observed (Gasborn); 1.5 predicted | same | 1.48 km | unit test |
| Median pluton in the mod | - | - | length 25 km, width 0.5-1 of it: D = 21.7 km, thickness 3.8 km (4.1 km before the fix) | arithmetic |
| Coastal Batholith of Peru | length 1,600 km; "up to" 1,000 plutons; 100-37 Ma; width not found | Williams slides; Wikipedia (secondary); the primary papers are closed | 1,034 plutons emplaced in 500 Myr on the whole 16k planet, 230 alive as bodies at the end | Run below |
| Tuolumne Intrusive Complex | about 1,100 km2, 95-85 Ma, 6-10 km deep | Ardill et al. 2020 (full text) | median pluton about 370 km2 (D 21.7 km); roof 2-6 km | - |
| Large Sierra plutons | width / length 0.17 (Lamarck), 0.21 (Mount Givens), 0.4 (Cathedral Peak) | Bateman 1992 (full text); Wikipedia | 0.5-1, mean 0.75 | - |
| Crust thickness | mean 41.1 km; orogens 46.3 km; never over about 75 km | Christensen and Mooney 1995 abstract; Hacker part 14 | 35 km reference; burial factor up to 2 (70 km) | - |
| Columns holding some metamorphic rock | not counted | - | 67.4 % of 20,000 random columns; 97.8 % of land columns | run below |
| Grade inversions, thin metamorphic slivers, plutonic sheets | zero by construction of the check | - | 0.07 % grade inversions, 0.25 % thin metamorphic slivers, 0.05 % plutonic sheets (20,000 columns); none in the stored stacks | run below |

Run below: `earthlike_quick_16k`, start `scattered_continents`, 500 Myr, default seed, 2 threads, `deeptime run` and
`deeptime strata-check --columns 20000`, test machine, 2026-10-02; before is master 9e84fcf and after is this branch (commit 5cb328c3). The pluton thickness fix lowered the surviving pluton bodies by 12 of 230 (5 %) and raised the eroded ones from 616 to 628; nothing else in the run changed (emplaced 1,034, subducted 104, metamorphosed volume 4.058e7 km3, deformed share 0.4805), and `strata-check` is identical (see [strata.md](strata.md)).

### Schist at 525 degC instead of 450 (Ben, 2026-10-02)

Ben answered the first open question of the stage-2 review with "Move schist to 500-550 degC". The coded value is
now 525 degC, the middle of the 500-550 degC in which the garnet of the Barrovian type locality grew (Baxter et al.
2002); amphibolite stays at 500 and gneiss at 650, so the order along the pelitic path is slate 250, phyllite 350,
schist 525, gneiss 650, and the key bands of the strata (RCK-052) are 0, 250, 300, 350, 500, 525 and 650 degC, each
still a formation temperature. Phyllite now covers 350 to 525 degC instead of 350 to 450. A unit test fails on the old
value (`the_grade_order_and_the_key_bands_follow_the_formation_temperatures`).

Measured on `earthlike_quick_16k` (scattered continents, 500 Myr, 2 threads, `strata-check --columns 20000`, test
machine, 2026-10-02), before and after, on two seeds. The worlds are not the same planet before and after (a rock class
changes the erodibility the surface stage reads, so the runs diverge: the shares below move on both seeds by amounts
that are not all the schist's doing); read them as two samples each.

| Share of | Seed 1 before | Seed 1 after | Seed 2 before | Seed 2 after |
|---|---|---|---|---|
| columns holding schist | 35.8 % | 27.4 % | 31.4 % | 14.0 % |
| all rock blocks that are schist | 5.33 % | 5.49 % | 5.80 % | 1.60 % |
| all rock blocks that are phyllite | 0.90 % | 1.31 % | 0.85 % | 1.02 % |
| all rock blocks that are gneiss | 15.6 % | 14.2 % | 13.1 % | 16.8 % |
| columns with a grade inversion | 0.14 % | 0.18 % | 0.08 % | 0.10 % |
| stored cell stacks with a grade inversion | 0 | 0 | 0 | 0 |
| columns without folds and faults with an inversion | 0 | 0 | 0 | 0 |

A third pair on the code as it stood after the plate-force and deposit merges (seed 1, master before and the branch
after, both through the branch's `strata-check`, whose class shares are the 524,288 stored cell stacks; the "before"
inversions come from the master binary's own check, since reading the old world with the new temperatures reports
inversions that the old code never made):

| Share of | Before | After |
|---|---|---|
| stored stacks holding schist / phyllite / gneiss / amphibolite | 37.5 / 16.3 / 57.2 / 1.0 % | 29.1 / 21.7 / 57.6 / 0.8 % |
| all rock blocks that are schist / phyllite / gneiss | 5.0 / 0.87 / 14.3 % | 5.2 / 1.27 / 13.0 % |
| stored stacks with a grade inversion | 0 of 524,288 | 0 of 524,288 |
| land columns with folds and faults applied: inversions | 2 of 4,556 (0.04 %) | 10 of 5,448 (0.18 %) |

Same finding as the first two pairs: schist falls in columns, not in volume, phyllite takes the difference, gneiss does
not move, and the stored stacks, where the rule is that grade rises downwards, stay at zero inversions. The land
columns' ten inversions are gneiss over basalt (5), gneiss over phyllite (2), phyllite over basalt (2) and gneiss over
sandstone (1), all structure-stage thrusts, 0.18 % of land columns.

Schist is made two ways: by the temperature (shale and its kin stepping to schist) and directly, as the metamorphic
pool of a young Himalayan-type orogen (RCK-018), which the threshold does not touch; that is why seed 1's schist volume
did not fall while its column share did. Raising the threshold moved rock from schist into phyllite (and, on seed 2, the
planet's own changes put more into gneiss). The inversions that exist are all made by the structure stage (thrusts and
dykes put a hotter rock over a colder one, as in the Himalaya: Mottram et al. 2014), none by the stored stacks or by the
columns before folds are applied; a wider phyllite band (350 to 525) makes more pairs such as gneiss over phyllite.

## Simplifications and limits

- **Temperature only.** No pressure and no time. Blueschist and eclogite (high pressure, low temperature) are never formed
  because there is no cold-subduction path (RCK-048). Granulite is folded into gneiss.
- **A straight geotherm** with constant conductivity and no radiogenic heat. Real geotherms bend with depth and with
  heat production.
- **Names are textures.** "Gneiss" and "schist" name textures, not temperatures (Nelson). The mod needs a series of
  rocks to show and uses the temperatures at which the textures are usually reached in a Barrovian belt.
- **Granite going to gneiss by heat** is a simplification: an orthogneiss needs non-hydrostatic deformation as well as heat.
  Peridotite goes to serpentinite by hydration, not heating, and the mod never makes it.
- **Pools have no burial history.** The basement is stepped along its path by today's geotherm, not by the heat it once had.
- **One body per intrusive suite.** A pluton in the mod stands for a whole suite; real plutons grow in increments of
  sheets over millions of years (Glazner et al. 2004).
- **Aureoles are a width, not a temperature field.** No isotherm is computed and no peak temperature is stored; the zone
  is one class (skarn or hornfels) across its width, with no isograds inside it, and it exists only on soft hosts.
- **Aureoles are read from the undeformed column at the intrusion's centre** and are not offset by younger faults; a host
  that changes sideways within the aureole takes the class of the centre's host at that depth.
- **Aureole and pluton bodies paint over younger dykes and stocks** that cut them, because the world data paints bodies over
  every structural event. A stock rooted on its own pluton is repainted through that pluton's aureole by a `stock` body; a
  dyke or a much younger stock is not. A format rule "a body with an age paints only older rock" would fix it; it is not
  built (REV-2 follow-up).
- **Plutons are rounder than the big real ones** (width over length 0.5 to 1 against 0.17 to 0.4), and sit shallower
  than the Sierra's 3.5 to 10 km.
- **Nothing reads a class's density.** The 62 densities are loaded from the rock-class files into the Rust table and the
  Java record, and nothing else reads them (a search of the code on 2026-10-02 found no use), so a density that is off
  changes nothing a player sees.
- **Densities are of fresh rock.** Weathered and porous rock is lighter (the New Zealand database runs down to 1500-2000 kg/m3
  for igneous rock); the table is for unweathered crust.
- **One CCD** of 4,500 m for every ocean, the Pacific value.
- **Loose and rock.** Sediment turns to rock at 150 m only for beds under 20 Myr old; real mud and sand stay soft for 1 to 3 km.

## Game choices

The vocabulary, families and groups (RCK-013, the MIC rock-tag README); the metamorphic grade index (RCK-047); continental crust
only (RCK-036); forward-only metamorphism (RCK-038); every pool and path table (RCK-016 to RCK-023, RCK-046); aureole width and
the carbonate factor (RCK-053, RCK-091); pluton shares, length and width (RCK-058 to RCK-065); the stone and deepslate stand-ins (RCK-075 to RCK-079,
decisions log 2026-10-01, Ben), the block resolution order (RCK-080, plan section 7.5). The appearance claims in the
generator comments ("dark grey", "banded") are descriptions, not data (RCK-089).

## Open questions

1. **Pluton width, roof depth, shares and collision rate.** Each is a game choice that real data put somewhere else
   (rows above). STATUS pending questions.
2. **Compaction and lithification depth** (RCK-022, STR-049).
3. **Densities the sources disagree with** (RCK-087): peridotite, kimberlite, ironstone, BIF, phosphorite, pumice, travertine.
4. **Suevite in carbonate targets.** French 1998: suevite has been found only in craters in crystalline silicate targets;
   carbonate-target craters have lithic breccia only. The impact stage (IMP) should be asked whether it makes impact_breccia
   from limestone.
5. **Sources that would settle the rest:** Winter 2010 and Yardley 1989 (formation temperatures), Haederle and Atherton 2002
   (Coastal Batholith width and count), Cruden and McCaffrey 2001 itself, Buddington 1959, Telford 1990 (density table).

## Sources

Read in full or in the relevant part: Cruden 2008 (SKB R-08-138); Daly, Manger and Clark 1966; Manger 1963; Handbook of Mineralogy;
French 1998; Stoeffler and Grieve 2007; Hardie 1967; Orti et al. 2012; Davies and Davies 2010; Meinert 1992 (scan);
Robertson 1988; Baxter et al. 2002; Stipp et al. 2002; Ferrill et al. 2004; Annen 2017; Mottram et al. 2014; Sass et al. 1994;
Bateman 1992; Ducea 2001; Glazner et al. 2004; Greene et al. 2006; Dietl and Koyi 2008; Vigneresse and Burg 2003;
Ardill et al. 2020. Abstract only: Christensen and Mooney 1995; the Brown and Johnson dataset. Secondary: Nelson (Tulane),
Hacker (UCSB), Greenwood and Berman, Wikipedia pages, pyBacktrack and RocFall documentation. Closed and not read: Winter 2010,
Yardley 1989, Spear 1993, Telford et al. 1990, Schon 2015, Cruden 1998, McCaffrey and Petford 1997, Cruden and McCaffrey 2001,
Buddington 1959, Haederle and Atherton 2002, Mukasa 1986, Warren 2016. Details and library keys in
[bibliography.md](bibliography.md), section "Rock classes, metamorphism, plutons and strata: sources read in WP-7".
