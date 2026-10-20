# Extraction notes: the research notes and designs

Written by the stage-1 extraction agent for this subsystem (2026-10-01/02) while it built the ledger. It holds the measured Earth-against-mod numbers found in the build log, the simplifications and game choices the code or documents state, and flags: admissions of unchecked values, contradictions between comments, code and documents, and things that looked wrong. **Nothing here is verified.** A line marked "suspicion" or "my arithmetic" is the extraction agent's own reading, to be checked in stage 2. See [claims.md](../claims.md).
Extracted from the Deep Time repo (W = ``). Quotations are copied from the documents; line numbers are those of the files at the time of reading.

## Earth figures and how they were sourced

Scope: every "Earth is X" figure in documents 6 to 11 (the run accuracy review, the grid-artifact audit, the force-balance design, the collisions-and-welding design, the per-cell climate design and the plate-check baseline). Each table gives the quantity, the value as the document writes it, the source the document names, and how the document marks it (from memory, unverified, not confirmed, sourced, measured on observed data, or not marked).

**The known flags, quoted exactly with file and section** (also repeated in the tables below):

1. File `docs/research/notes/run-review-1a0f969d358.md`, section "Sources" (lines 385-387): "From memory, not checked: Earth's continental crust at about 40 % of its area; land-height shares above 2 and 4 km; mean land height; mountain erosion rates; the sea-ice edge's wander; Hawaii–Emperor length and age; the long-term sea-level range; the Caspian's area; the count of Earth plates above 5 %." Also section 1 "Plate sizes: wrong" (lines 94-95): "(the seven-plate count is sourced; the 5 % reading is from memory)".
2. File `docs/research/notes/grid-artifact-audit.md`, section "Findings", "### 2. Straight rivers (open)" (lines 108-109): "Real rivers are commonly 1.1–1.5 and over (from memory, not sourced)." Repeated in section "Not verified" (line 336): "Earth figures for river sinuosity are from memory."
3. File `docs/design/plate-force-balance.md`, section "3.2 The terms" (lines 116-118): "Gross slab pull is from textbook examples and was not confirmed in Turcotte & Schubert itself." (after "Known sizes, for orientation: slab pull about 1–3 × 10¹³ N/m gross, ridge push 2–3.5 × 10¹² N/m, a high plateau 6–7 × 10¹² N/m."); same section, table row "Basal drag" (line 112): "A factor of 8 is often attributed to Forsyth & Uyeda; not confirmed."; section "3.5 Both sides of the suture, and the foreland basin" (lines 161-162): "Source for a fixed taper: Davis, Suppe & Dahlen 1983, *JGR* 88:1153 (from memory)."; and section "12. Sources" (line 552): "From memory, not checked: Davis, Suppe & Dahlen 1983, *J. Geophys. Res.* 88:1153–1172."

Further flags found while reading (not in the caller's list): plate-collisions-and-welding.md section 3 ("From memory: the plate carrying Pangaea was well under half the planet"; "An orogen-wide mean surface uplift of 0.1–1 mm/yr is from memory"; "No published statistic was found" for plate lifetimes); p1-26-per-cell-climate.md section 9 (six references "† Cited from memory, not re-read in this session") and Appendix A item 6 ("round climatological values for orientation, not part of the scored data").

### Document 6: docs/research/notes/run-review-1a0f969d358.md (accuracy review of workbench run 1a0f969d358)

How the document marks sourcing. Its "Sources" section (lines 362-383) lists the items "Sourced by web search for this review" and then states (lines 385-387), quoted exactly:

> From memory, not checked: Earth's continental crust at about 40 % of its area; land-height shares
> above 2 and 4 km; mean land height; mountain erosion rates; the sea-ice edge's wander; Hawaii–Emperor
> length and age; the long-term sea-level range; the Caspian's area; the count of Earth plates above 5 %.

Other markers in the text: "(the seven-plate count is sourced; the 5 % reading is from memory)" (line 94-95, section 1 "Plate sizes: wrong"); "I have no sourced figure for Earth's turnover rate." (line 103, section 1 "Plate turnover: too fast"); "Earth's is about 27 % (derived: 29 % land on 40 % crust)" (line 164, section 3 "Land fraction"); inline "(from memory)" tags at the lines listed below. Where a figure below has no source named and no marker, the table says "no source named".

| Quantity | Earth value as the document gives it | Source named | Marked by the document as |
|---|---|---|---|
| Median plate speed | "Earth about 40" mm/yr (summary, line 16); "RMS about 40; fastest plates about 85" (table, line 83) | Zahirovic et al. 2015 (Sources) | sourced |
| Plate speed limit | "limit about 200" mm/yr; full convergence rates "20-150" (table, line 80) | Zahirovic et al. 2015 for the 200 limit; "20-150" no source named | limit sourced; 20-150 no source named |
| Spreading at ridges | "East Pacific Rise about 150" mm/yr (table, line 81) | no source named | not marked |
| Convergence at collisions | "India-Eurasia 50 now, up to 190 at 55 Ma" mm/yr (table, line 82) | no source named | not marked |
| Largest plate | Pacific "20 %" of Earth (line 94); 103 million km2 (Sources, line 365) | Wikipedia Pacific plate | sourced |
| Plates above 5 % of area | "seven plates above 5 % of its area" (line 94-95) | arXiv 1011.2752 (seven major plates) | "the seven-plate count is sourced; the 5 % reading is from memory"; also in the from-memory list ("the count of Earth plates above 5 %") |
| Plate lifetime / turnover | "Earth's large plates last hundreds of Myr" (line 103) | none | "I have no sourced figure for Earth's turnover rate." |
| Supercontinent cycle period | "Earth's is quoted from about 400 to 800 Myr" (line 127) | Wikipedia Supercontinent cycle; Nature Rev. Earth Environ. s43017-021-00160-0 | sourced |
| Ocean opening before closing | "100-200 Myr or more"; "the Atlantic has been opening for about 180 Myr" (lines 134-135) | no source named | not marked |
| Flood-basalt / rift link | "Central Atlantic province and the opening of the Atlantic" (line 139) | no source named | not marked |
| Continental crust share | "Earth about 40 %" (line 17); "Earth is about 40 % (from memory)" (line 157) | none | from memory (in the from-memory list) |
| Land fraction | "Earth 29 %" (lines 18, 162) | none named | not marked (not in the from-memory list; used to derive 27 %) |
| Continental crust under water | "Earth's is about 27 % (derived: 29 % land on 40 % crust)" (line 164) | derived from the two figures above | derived |
| Sea-floor age (mean/median) | "mean 64 Myr", Earth 64 Myr (lines 19, 174) | Muller et al. 2008 (mean 64.2 Myr) | sourced (document compares its median 68 Myr with Earth's mean) |
| Oldest large sea-floor areas | "180-200 Myr" (line 175) | Muller et al. 2008 | sourced |
| Sea floor older than 180 / 250 Myr | "close to none" / "one small basin, about 340 Myr" (lines 177-178) | Nature Geoscience ngeo2784 (Herodotus Basin about 340 Myr); "close to none" no source named | 340 Myr sourced; "close to none" not marked |
| Himalaya size | "about 2,400 km long and a few hundred wide" (line 206) | none | not marked |
| Mean land height | "Earth's mean is about 800 m (from memory)" (line 210) | none | from memory |
| Highest point | "Everest is 8.8 km" (line 211) | none | not marked |
| Land above 2 / 4 km | "Earth's figures are about the same (from memory)" (line 212) | none | from memory |
| Mountain erosion rates | "roughly 20 m/Myr (old ranges) to over 1,000 (the Himalaya), from memory" (lines 217-218) | none | from memory |
| Age of Appalachians and Urals | "300-Myr-old" (line 220) | none | not marked |
| Hawaii-Emperor chain | "about 6,000 km over about 80 Myr (from memory)" (line 230-231) | none | from memory |
| Flood-basalt frequency | "Earth averages one per 20-30 Myr, so 33-50" (line 237) | Ernst and Youbi 2017 | sourced |
| Deccan / Siberian Traps size | "the size of the Deccan or Siberian Traps" for 2.0-2.7 million km2 (line 238-239) | none | not marked |
| Sea ice extent | "Earth's is about 12 %" of the ocean (line 251-252); Sources: about 7 % of Earth's surface and 12 % of the oceans | Wikipedia Sea ice | sourced |
| Sea-ice edge wander | "Earth's ice edge wanders by 20 degrees or more with currents and coasts (from memory)" (line 252-253) | none | from memory |
| Land ice today | "Earth today is about 10 %" of land (line 260) | none | not marked |
| Ice-free share of the last 540 Myr | "about 80 % of the time" (line 261-262) | Wikipedia Greenhouse and icehouse Earth | sourced |
| Global mean temperature | "Earth today is about 14-15 degC" (line 263) | none | not marked |
| Climate zones by land area | Table line 284: tropical 19 %, arid 30 %, temperate 13 %, cold 25 %, polar 13 % ("Earth (Peel et al. 2007)") | Peel et al. 2007 | sourced |
| Long-term sea-level range | "about -100 to +200 m (from memory)" (line 289-290) | none | from memory |
| Lake cover | "Earth's lakes cover 3.7 % of ice-free land" (line 301) | Verpoorter et al. 2014 | sourced |
| Caspian Sea area | "0.37 million" km2 (line 303) | none | from memory |

### Document 7: docs/research/notes/grid-artifact-audit.md (grid-artifact audit, P1-23)

This document makes almost no "Earth is X" claims, because it measures the bake against isotropic or random controls rather than against Earth. The one Earth comparison, and the flag, are:

| Quantity | Earth value as the document gives it | Source named | Marked by the document as |
|---|---|---|---|
| River sinuosity | "Real rivers are commonly 1.1-1.5 and over" (compared with the bake's median sinuosity 1.035, 10th percentile 1.010, 90th 1.112 over 128 blocks of path) -- file docs/research/notes/grid-artifact-audit.md, section "Findings", "### 2. Straight rivers (open)", lines 108-109 | none | **"(from memory, not sourced)"**, quoted exactly from lines 108-109: "Real rivers are commonly 1.1–1.5 and over (from memory, not sourced)." Repeated in section "Not verified", line 336: "Earth figures for river sinuosity are from memory." |

Other numbers that look like Earth or geometric figures but are not Earth data: the isotropic control "Within 2 degrees of an axis is 4.4 % for an isotropic field, 2.2 % for each axis alone" (section "Method", orientation test; a geometric identity); "at 67 degrees a round feature is 2.6 x wider than tall in blocks" (Finding 10; the equirectangular projection); the run's planet statistics (land 27.7 %, median land height 715 m, highest point 6,609 m; section "What was fixed") are the bake's own outputs, not Earth values.

### Document 8: docs/design/plate-force-balance.md (force-balance plate motion design)

How the document marks sourcing. Section 12 "Sources" (line 522-526): "Checked online on 2026-10-02 by a sub-agent; most papers are paywalled, so figures are from abstracts or secondary pages unless a link is given. Items it could not confirm are marked in the text." The three flags the caller asked for, quoted exactly:

1. **Gross size of slab pull, not confirmed.** Section "3.2 The terms", after the table, lines 116-118:
   > Known sizes, for orientation: slab pull about 1–3 × 10¹³ N/m gross, ridge push 2–3.5 × 10¹² N/m,
   > a high plateau 6–7 × 10¹² N/m. Gross slab pull is from textbook examples and was not confirmed in
   > Turcotte & Schubert itself.
2. **A factor of 8 for continental drag, not confirmed.** Section "3.2 The terms", table row "Basal drag (resists)", line 112:
   > Continental roots raise the traction 2–5 times (Conrad & Lithgow-Bertelloni 2006, *GRL* 33:L05312, abstract). A factor of 8 is often attributed to Forsyth & Uyeda; not confirmed.
3. **Wedge-taper reference, from memory.** Section "3.5 Both sides of the suture, and the foreland basin", bullet "Incoming side: a wedge", lines 161-162:
   > Source for a fixed taper: Davis, Suppe & Dahlen 1983, *JGR*
   > 88:1153 (from memory).

   Repeated in section "12. Sources", last bullet (line 552): "From memory, not checked: Davis, Suppe & Dahlen 1983, *J. Geophys. Res.* 88:1153–1172."

All "Earth" figures in the document, with the source the document names and its marking:

| Quantity | Earth value as the document gives it | Source named | Marked as |
|---|---|---|---|
| Speed of the fastest and slowest plate groups | "Earth: about 85 and 26-28" mm/yr (sec 1, line 33-34); "about 85 mm/yr for the fastest group and 26-28 for the slowest" (sec 2, line 80-81); acceptance table "slowest group 26-28, median 40, fastest 85" (line 351) | Zahirovic et al. 2015 | sourced (link given; reading mode not stated) |
| Area-weighted RMS plate speed | "about 40" mm/yr (sec 7 table, line 350) | none named in that row (Zahirovic et al. 2015 is the document's speed source) | not marked |
| Slab pull dominance | "The dominant force; an order of magnitude above the rest" (sec 3.2, line 107) | Forsyth and Uyeda 1975 | "abstract" |
| Share of a slab's weight reaching the plate | "Only 8-12 %", "up to twice ridge push" (sec 3.2, line 107) | Schellart 2004 | "abstract read in full" (sec 12) |
| Slab-pull gross size | "about 1-3 x 10^13 N/m gross" (line 116) | none; "from textbook examples" | **"was not confirmed in Turcotte & Schubert itself"** |
| Ridge push | "2-3.5 x 10^12 N/m" (line 116); "about 3 x 10^12 N/m for old plates" (line 110) | Parsons and Richter 1980 (the 3 x 10^12 figure) | not marked beyond the sec 12 blanket |
| Ridge push share of driving force | "under 10 %" (line 110) | Lithgow-Bertelloni and Richards 1998 | not marked beyond the sec 12 blanket |
| High plateau force | "Tibet is quoted at 6-7 x 10^12" N/m; "a high plateau 6-7 x 10^12 N/m" (lines 110, 116) | Molnar and Lyon-Caen 1988 | not marked beyond the sec 12 blanket; the model's own formula gives 6.7 x 10^12 for a 5 km belt on 35 km crust (computed) |
| Subducting vs other plates | "about four times faster" (line 109); acceptance table "oceanic / continental about 3" (line 352) | Conrad and Lithgow-Bertelloni 2002, 2004 for the factor 4 | "from abstracts"; the "about 3" has no source named |
| Speed vs slab age | "Speed follows slab age at the trench" (line 107) | Carlson, Hilde and Uyeda 1983 | not marked beyond the sec 12 blanket |
| India's slowdown | "India slowed from 150 to 40 mm/yr as Tibet thickened" (line 111); "India: 0.27 in 15 Myr" (line 359) | Copley, Avouac and Royer 2010 (first); no source named (second) | not marked beyond the sec 12 blanket |
| Continental-root traction | "2-5 times" (line 112) | Conrad and Lithgow-Bertelloni 2006 | "abstract" |
| Continental drag factor of 8 | "A factor of 8 is often attributed to Forsyth & Uyeda" (line 112) | Forsyth and Uyeda 1975 (attributed) | **"not confirmed"** |
| Slab breakoff time | "10-25 Myr" (line 127) | van Hunen and Allen 2011 | not marked beyond the sec 12 blanket |
| Critical-wedge taper | `wedge_taper_deg` 1.5 degrees "fixed surface slope" (line 158-162) | Davis, Suppe and Dahlen 1983 | **"(from memory)"**, "not checked" |
| Himalayan foreland | "about 450 km wide and 3 to over 7 km deep" (line 172-173) | Wikipedia, Himalayan foreland basin | web page |
| Foredeeps and forebulges | "100-300 km and 2-8 km, forebulges tens to a few hundred metres high" (line 173-175) | DeCelles and Giles 1996 | "via a secondary source" |
| Himalayan wedge | "Himalaya 150-350 km" (line 365) | none named | not marked |
| Foredeep acceptance range | "100-450 km, 2-8 km" (line 364) | none named | not marked |
| Africa and Antarctica drift | "5-25 mm/yr, as Africa and Antarctica do" (line 265) | none named | not marked |
| Speed vs slab-edge share | "3-4" for the ratio; rank correlation "the Forsyth & Uyeda result" (lines 353-354) | Forsyth and Uyeda 1975 (the correlation) | not marked beyond the sec 12 blanket |
| Speed vs plate area | "none (Forsyth & Uyeda)" (line 355) | Forsyth and Uyeda 1975 | not marked beyond the sec 12 blanket |
| Sea-floor age median | "64" Myr (line 360) | none named here (the run review cites Muller et al. 2008) | not marked |
| Old sea floor | "close to none / one small basin" (line 361) | none named here | not marked |
| Supercontinent tenure | "100-300 Myr" (line 362) | none named | not marked |
| Assembly-to-assembly period | "300-500 Myr" (line 363) | none named | not marked |
| Forces needed to break a continent | "a few 10^12 N/m ... widely held to be too small to tear thick, cold continental lithosphere without weakening by magma or heat" (line 396-398) | Buck 2004; Bialas, Buck and Qin 2010 | "from abstracts" |
| Interior extension in supercontinent models | "20-50 MPa" (line 401) | Huang et al. 2019 | not marked beyond the sec 12 blanket |
| Pangaea behaviour | "close to what is believed of Pangaea" (line 410) | none | not marked |

### Document 9: docs/design/plate-collisions-and-welding.md (collision and welding design)

How the document marks sourcing. Section 3 "What Earth does" (line 181-182): "Checked online on 2026-10-01 unless marked 'from memory'. Paywalled papers were confirmed from their abstracts or from secondary pages." Explicit marks in section 3, quoted: "No source gives the largest plate's share through a supercontinent's life. From memory: the plate carrying Pangaea was well under half the planet, with several oceanic plates in the surrounding ocean." (lines 193-195); "Plate lifetimes. No published statistic was found." (line 197); "An orogen-wide mean surface uplift of 0.1–1 mm/yr is from memory." (lines 225-226); "This is the definition of a plate; no source states it as a finding." (line 235). The "Earth" column of the section 2.2 table (lines 84-102) names no source.

| Quantity | Earth value as the document gives it | Source named | Marked as |
|---|---|---|---|
| Largest plate share | "0.20 today"; Pacific 103.3 million km2, "20 % of the Earth" (line 86; line 185) | Bird 2003 (areas from Wikipedia List of tectonic plates) | checked online |
| Number of plates | "52 plates in Bird's model" (line 185) | Bird 2003 | checked online |
| Share covered by the seven largest | "about 92 %" (line 186-187) | Bird 2003; Wikipedia | checked online |
| Plates above 5 % of the planet | "7" (table, line 90) | none named in the table | not marked |
| Steps with one plate above 50 % | "none today" (table, line 87) | none | not marked |
| Large and small plates | "About seven large plates and many small ones for the last 200 Myr" (line 189) | Morra et al. 2013; Mallard et al. 2016 | checked online; "Whether the two groups are distinct populations is disputed" (GSA Today 2018) |
| Largest plate through a supercontinent's life | "well under half the planet" for the plate carrying Pangaea (line 193-195) | none | **"From memory"**; "No source gives the largest plate's share through a supercontinent's life." |
| Plate lifetime | "100-200 Myr for large plates" (table, line 91); "large plates last 100-200 Myr or more, small ones tens of Myr" (line 200-201) | cases: Pacific (born about 190 Ma), Farallon (Lonsdale 2005), Izanagi (gone by about 55 Ma), Kula (about 48 Ma) from Wikipedia | "**No published statistic was found**" (line 197); the 100-200 Myr is an inference from cases |
| Wilson cycle opening time | "an ocean opens for 100-200 Myr" (table, line 93); Iapetus about 600 to 420 Ma; Atlantic "about 175-180 Myr" (lines 208-210) | Wikipedia Iapetus Ocean; Wilson 1966 | checked online (Atlantic age: no source named) |
| Supercontinent cycle period | "about 300-500 Myr" (line 204) | Wikipedia Supercontinent cycle; Nance 2022 | checked online |
| Supercontinent duration | "lasts 100-300 Myr" (line 205-206) | Mitchell et al. 2021 | checked online |
| Pangaea timing | "assembled about 335-325 Ma and began to break up about 200-175 Ma" (line 206-207) | none named | not marked |
| India-Asia collision | "met 59-50 Ma"; "convergence fell from about 150 to about 40 mm/yr between 50 and 35 Ma"; thickening of Tibet the main cause (lines 213-215) | Copley, Avouac and Royer 2010 | checked online (abstract or secondary page) |
| Collision durations | Alleghanian about 325-260 Ma (65 Myr); Uralian about 70-90 Myr (lines 216-217) | Wikipedia pages | web pages |
| Collisions per 100 Myr | "about 3-4 (P1-27's figure)" (table, line 102) | P1-27 (internal) | not marked |
| Uplift per collision | "varies enormously" (table, line 99) | none | not marked |
| Orogen shapes | Himalaya "2,400 km long and 150-350 km wide"; Alps "1,200 x 250 km"; Urals "2,500 km long"; Tibet "2,500 x 1,000 km at a mean above 4,500 m" (lines 219-221); "5-15 times longer than wide" for young belts (line 221; table line 101 "5-15 for young belts") | Wikipedia pages of each | web pages |
| Himalaya lifted area | "Himalaya 2,400 x 150-350 km" (table, line 100) | as above | web page |
| Uplift and exhumation rates | "rock uplift about 5 mm/yr in the Himalaya, 0.5-2 mm/yr in the western Alps"; "more than 1 mm/yr of exhumation in southern Tibet at 17-12 Ma" (lines 223-225) | Nocquet et al. 2016; Tremblay et al. 2015 | checked online |
| Orogen-wide mean surface uplift | "0.1-1 mm/yr" (line 225-226) | none | **"from memory"** |
| Slab-attached plate speed | "3-4 times faster than the others" (line 229-230) | Forsyth and Uyeda 1975 | "from its abstract" |
| Slab breakoff | buoyant continent resists going down; slab can break off (line 232-233) | Davies and von Blanckenburg 1995 | checked online |
| India-Australia plate | "one plate now splitting into India, Capricorn and Australia" (line 236-237) | Royer and Gordon 1997 | checked online |
| Supercontinent heating | "up to about 100 degC" (Coltice 2007); "about 50 degC" (Heron and Lowman 2011); mantle under a large continent "warms for 100-200 Myr" (lines 238-242, 337) | Gurnis 1988; Coltice et al. 2007; Heron and Lowman 2011; Lenardic et al. 2011; Bercovici and Long 2014 | checked online; "The physics is disputed (section 3)" |

### Document 10: docs/design/p1-26-per-cell-climate.md (P1-26 per-cell climate design)

How the document marks sourcing. "Every number below was measured for this document, unless it says 'estimate' or names a source" (line 10). The "observed" columns come from two datasets named in section 1.1 and section 9: Beck et al. 2018 (Koppen-Geiger, 0.5 degrees) and Beck et al. 2023 (monthly T and P, 1961-1990, 0.5 degrees), on GMT's `earth_relief_30m_p` (SRTM15+). Observed temperatures "were brought to sea level with the model's 6.5 degC/km on the same relief" (lines 91-92). The dagger rule (line 1162-1163), quoted: "† Cited from memory, not re-read in this session. The implementing tasks verify the page numbers before code headers cite them." (applies to Ferreira et al. 2018, Lindzen and Nigam 1987, Bretherton, Peters and Back 2004, Manabe 1969, Klein and Hartmann 1993, Serreze, Kahl and Schnell 1992). Appendix A item 6 (lines 1211-1212): "The observed sea-surface temperatures quoted for coastal points ('about 11 degC off Brest') are round climatological values for orientation, not part of the scored data."

| Quantity | Earth value as the document gives it | Source named | Marked as |
|---|---|---|---|
| Variance of land sea-level temperature explained by latitude (eta^2) | annual 0.968, winter 0.942, summer 0.936 ("Earth's 96.8 %", lines 22-23, 111, 1050) | Beck et al. 2023 data on SRTM15+ relief | measured on observed data |
| Along-latitude-circle std of land sea-level temperature | annual 3.29 K, winter 6.06 K, summer 3.16 K ("Earth's 6.1 K", lines 24, 110) | same | measured on observed data |
| Precipitation variability over land | ln P eta^2 0.413, along-row std 0.84 (line 112) | Beck et al. 2023 | measured on observed data |
| Southern winter sea-ice edge | "Antarctic winter edge spans roughly 55-68 degS" (line 114); "several degrees" std (line 1014) | none named | not marked |
| Koppen aggregates | eta^2 0.34, band heterogeneity 0.50, E-W boundaries 0.56 (line 115) | Beck et al. 2018 | measured on observed data |
| Point temperatures, January sea-level T (Brest 7.2, Bergen 0.6, Moscow -8.8, Yakutsk -39.0, Vladivostok -12.9, Seattle 4.5, Boston -2.8, Winnipeg -16.5 degC) | lines 124-125; Yakutsk -39 degC also lines 28, 1016, 1237 | Beck et al. 2023 (lapse-corrected to sea level) | measured on observed data |
| Point annual precipitation (Sahara 11, Atacama 16, Namib 61, Mumbai 3,096, Bangladesh 2,194, SE China 1,765, Kazakh steppe 142, Gobi 112, Amazon 2,234, London 702, Congo 2,156, Central Australia 248 mm) | lines 129-130, 653-665 | Beck et al. 2023 | measured on observed data |
| Brest minus Vladivostok January contrast | "20.1 K" (line 1015) | Beck et al. 2023 | measured on observed data |
| Winter land temperature along-row std and eta^2 on the 2.5 degree grid | "obs 6.0 K", "obs 0.944" (lines 1011, 1237) | Beck et al. 2023 block-averaged | measured on observed data |
| Observed ln P RMSE benchmark | today 1.11, Earth eta^2 0.41 (line 1012) | Beck et al. 2023 | measured |
| Global mean temperature at 280 ppm | "15 +/- 0.5 degC" target (line 1037); EBM 15.26 degC (line 85); 1-D model 15.2 degC (line 1251) | none named | not marked |
| Global albedo | "Earth's global albedo stays 0.30" (line 271) | none named | not marked |
| Surface albedo of the Sahara vs vegetation | "about 0.35 against 0.15 for vegetation" (line 268-269) | none named | not marked |
| Atlantic meridional overturning | "Earth's AMOC is about 17 Sv" (line 369) | none named | not marked |
| Ocean basin widths | "the North Atlantic (47 degrees wide) over the North Pacific (68 degrees)" (line 364) | none named; the explanation of deep-water formation is credited to Ferreira et al. 2018 (dagger) | not marked; Ferreira et al. 2018 "from memory, not re-read" |
| Observed January sea-surface temperatures | "off Brest about 11 degC; off Bergen about 7 degC; off Lisbon about 15 degC" (lines 378-380) | none | **"round climatological values for orientation, not part of the scored data"** (Appendix A item 6) |
| Ocean heat transport | "Trenberth and Caron 2001 put the ocean's at about 2 PW at 15-20 degN" (line 383-384); target "peak 1.5-2 PW at 15-25 degN" (line 1075) | Trenberth and Caron 2001 (no dagger) | cited; reading mode not stated |
| Siberian winter surface inversion | "Siberia's January surface is 10-20 K below 850 hPa" (line 467-468) | Serreze, Kahl and Schnell 1992 (dagger) | **from memory, not re-read** |
| Residence time of atmospheric water vapour | "about 8-10 days" (line 519-520) | none named | not marked |
| Precipitation vs column relative humidity | "tropical P rises 15.6-fold per unit of column relative humidity" (line 585-586) | Bretherton, Peters and Back 2004 (dagger) | **from memory, not re-read** |
| NMS83 land heat capacity | "1.87 x 10^6 J/m2/K, 0.45 m" (line 253-254) | North, Mengel and Short 1983 | cited; reading mode not stated |
| Earth-like diffusivity ratio | PlaHab "1/25" (line 287) | PlaHab (Ramirez 2023) | "not inspected, nothing taken" (section 9) |
| Grid precedents | TransEBM at T42 (128 x 64); GREB 3.75 degrees (lines 189-190) | Ziegler and Rehfeld 2021; Stassen et al. 2019 | cited; reading mode not stated |
| Reference cities in the right main group | "19 / 24" today, target >= 21 / 24 (line 1019) | `climate-dem` list (internal) | internal |

### Document 11: docs/design/plate-check-baseline.md (plate-check baseline on master)

This document is a measurement table. Its only "Earth is X" figure:

| Quantity | Earth value as the document gives it | Source named | Marked as |
|---|---|---|---|
| Slab-attached plate speed divided by other plate speed | "Earth: 3 to 4 (Forsyth and Uyeda)" (reported-only table, row "Slab-attached ÷ other plate speed", in section "Against the targets"); "Slab-attached plates move 1.4 times faster than the rest (Earth: 3 to 4)" (section "What the force-balance measures say today") | Forsyth and Uyeda (no year or journal given in this file) | not marked; no reading note. The same ratio is given in plate-collisions-and-welding.md section 3 as "3-4 times faster ... (Forsyth & Uyeda 1975, ... from its abstract)" and in plate-force-balance.md section 3.2 as "about four times faster" (Conrad and Lithgow-Bertelloni 2002, 2004, "from abstracts"). |

The "Design 'Now'" column is the design document's own measurements (see Document 9), not Earth data. Other columns are measured values of the project's runs.

## Positions the research held that were later overturned

Source: `docs/deeptime-implementation-plan.md` section 1.2 ("Where the documents live, and which one wins"), lines 72-82. The plan says: "The research report predates the decisions. These report positions were overturned and must not be followed:" and lists the following (each "was" is the report's position, "now" is the plan's):

1. Engine language: a pure-Java engine, now Rust over JNI.
2. World scale: 8 m per block, now about 300 m horizontal and 35 m vertical (SCL-002; 20 m vertical until 2026-10-02).
3. Licence: the MIT licence, now LGPL-3.0-or-later.
4. Floating islands: Terralith pass-through overlays, now simulated sky islands.
5. Determinism: bit-for-bit seed determinism, now "stored data only".
6. World shape: a bordered square map, now a wrapped planet.
7. Namespace: the `geohist` namespace, now `deeptime`.
8. Playable area: a window cut out of a larger planet, now the whole planet is the world.
9. History Atlas: a History Atlas item, now the timelapse lives only on the Create World screen.
10. Biome regions: TerraBlender's own region layout by default, now regions follow geography from v1.
11. Server flow: a CLI-first server flow, now a server generates seamlessly on first start (the CLI remains an option).
12. Build range: -64..639, now -352..639.
13. Phase numbering: the report's phase numbering, now the decisions file's Phases 0-3 apply.

Related note in the same paragraph (about a research note, not the report): `wrapped_planet_worlds.md` sizes the world in round chunk counts (1,000 and 2,000 chunks); the decisions fix C at exactly 16,384 and 32,768 blocks (1,024 and 2,048 chunks).

Where these appear in the report (document 1 in the sources DOC): MIT licence and decision 7 ("Licence"), 8 m per block and decisions 1-2 ("World scale", "Build height and world size"), the `geohist` names throughout the architecture and walkthrough, the bordered 4k/8k/16k map, the History Atlas in the walkthrough and phase plan, the pure-Java `geohist-core`, the Terralith Skylands overlay registry (decision 8), TerraBlender's own regions (decision 12), the CLI flow (decision 13), the -64..639 dimension type (decision 2), and Phases 0-4 of the report's phase plan.
