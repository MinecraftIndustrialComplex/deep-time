# Stage 1 spot checks

While the bibliography was being resolved (2026-10-02), each sub-agent tried to read the values
that matter most in their sources. This file holds what they found, value by value, with the passage
quoted and the level of evidence. It is a first pass and a guide for stage 2, **not a verification**:
no ledger row was marked `verified` on the strength of it.

How to read a result:

- `MATCH`: the fetched text gives the coded value.
- `MISMATCH`: the fetched text gives a different value, or the source does not contain what is
  attributed to it. These lead the verification plan in [claims.md](claims.md).
- `PARTIAL`: something was confirmed and something was not, or the value is right and its use is
  doubtful.
- `COULD NOT READ`: paywall, scan without text, blocked download.

**Evidence level** matters as much as the result. "Primary full text" is the paper itself.
"Primary abstract" is only its abstract. "Secondary source" is a citing paper, an agency page or an
encyclopedia. "Search snippet" is a search engine's summary and is weak. A `MATCH` at a secondary
level is a lead, not a confirmation. Lines marked "my arithmetic" are the sub-agent's own
calculation. The sub-agents were told never to judge a value from memory; where one of them recalls
something, it says so.

One result was re-read by the stage 1 lead in the extracted text of the paper: Collins, Melosh &
Marcus 2005, equation 28 (ledger IMP-042), a mismatch.

Totals: MATCH 176, PARTIAL 134, MISMATCH 19, COULD NOT READ 74.

| Subsystem | MATCH | PARTIAL | MISMATCH | COULD NOT READ |
|---|---|---|---|---|
| Meteor impacts (IMP) | 10 | 15 | 1 | 6 |
| Volcanoes and magma (VOL, MAG) | 4 | 10 | 3 | 4 |
| Erosion, isostasy and hypsometry (ERO, ISO, HYP) | 12 | 4 | 2 | 8 |
| Rivers, lakes and the young landscape (HYD, YNG) | 10 | 7 | 1 | 4 |
| The amplifier and glacial landforms (AMP, GLA) | 11 | 7 | 1 | 7 |
| Rock, strata and structure (RCK, STR, DEF) | 8 | 15 | 2 | 8 |
| Deposit models (DEP) | 35 | 10 | 2 | 4 |
| Deposit formation, coal, petroleum, placers (ORE) | 16 | 14 | 1 | 11 |
| Moisture, Köppen classes and biomes (MOI, BIO) | 18 | 6 | 2 | 2 |
| Scale, provinces, pole cap and bake (SCL, PRV, POL, BAK) | 9 | 15 | 0 | 4 |
| Plate tectonics (PLT) | 28 | 14 | 1 | 10 |
| Climate physics (CLM) | 15 | 17 | 3 | 6 |

---

## Meteor impacts (IMP)

Results: MATCH 10, PARTIAL 15, MISMATCH 1, COULD NOT READ 6.

Quotes from Collins et al. 2005 come from the text I extracted from the PDF at https://spiral.imperial.ac.uk/bitstreams/8065368c-096c-4787-aab7-cbaf1a7d7277/download ("C2005"). "Reassembled" means the equation's pieces were legible in the extraction and I put them in order; where a glyph (a square root, pi, theta) was lost I say so.

- IMP-030: coded D_tc = 1.161 (rho_i/rho_t)^(1/3) L^0.78 v^0.44 g^-0.22 (sin theta)^(1/3). C2005 eq. 21*, p. 823, reassembled: D_tc = 1.161 (rho_i/rho_t)^(1/3) L^0.78 v_i^0.44 g_E^-0.22 sin^(1/3) theta; prose: "the diameter of the transient crater D_tc (in m) as measured at the pre-impact target surface", "This equation applies for impacts into solid rock targets where gravity is the predominant arresting influence in crater growth, which is the case for all terrestrial impacts larger than a couple of hundred meters in diameter", "For impacts into water, the constant 1.161 must be replaced by 1.365", "In reality, these constants are not known to three decimal places; the values quoted serve as a best estimate within a range of 0.8 to 1.5." (C2005 URL above)
  Result: MATCH. Notes: the paper's g is Earth's gravity g_E (the code substitutes the planet's g, the usual Schmidt-Housen reading, but that is the code's extension). The paper credits "Holsapple and Schmidt (1982), Schmidt and Housen (1987), and Gault (1974)", not Gault & Wedekind 1978 or Chapman & McKinnon 1986. The validity statement "larger than a couple of hundred meters" is relevant to the 2-km minimum crater, which my arithmetic puts at an impactor about 84 m wide.
  Evidence level: primary full text

- IMP-031: coded D = 1.25 D_tc. C2005 eq. 22*, p. 824: "D_fr = 1.25 D_tc (22*)", "for a simple crater" (extracted: "D fr  1.25D tc (22*)").
  Result: MATCH
  Evidence level: primary full text

- IMP-032: coded D = 1.17 D_tc^1.13 / D_c^0.13. C2005 eq. 27*, p. 824, reassembled: D_fr = 1.17 D_tc^1.13 / D_c^0.13; prose: "established by McKinnon and Schenk (1985), which lies intermediate between the estimates of Croft (1985) and Holsapple (1993). In this equation, D_c is the diameter at which the transition from simple to complex crater occurs (taken to be 3.2 km on Earth); both D_tc and D_fr are in km".
  Result: MATCH
  Evidence level: primary full text

- IMP-033: coded final = max(1.25 D_tc, 1.17 D_tc^1.13/D_c^0.13). C2005 p. 824: "If the transient crater diameter is greater than 2.56 km, we apply Equation 27 to determine the final crater diameter and report that a 'complex' crater is formed; otherwise, we apply Equation 22 and report that a 'simple' crater is formed."
  Result: PARTIAL. The paper switches on D_tc > 2.56 km (= 3.2/1.25), so the final diameter jumps from 3.2 km (eq. 22) to about 2.91 km (eq. 27 at D_tc = 2.56 km, my arithmetic) and the complex law only takes over above that. The code instead takes the larger of the two, so the complex law applies only above D_tc about 1.66 D_c. The two agree away from the transition.
  Evidence level: primary full text

- IMP-034: coded transition 3.2 km (cited to Grieve 1987). C2005 pp. 823-824: "For craters smaller than ~3.2 km in diameter on Earth (classified by Dence [1965] as 'simple' based on their intuitive morphology)" and "(taken to be 3.2 km on Earth)". Grieve 1987 (OCR, the page headed "248 GRIEVE"): "At diameters above ~2 km in sedimentary targets and ~4 km in crystalline targets, terrestrial impact structures have a complex form". I found no "3.2" in the Grieve 1987 OCR text.
  Result: PARTIAL. The value 3.2 km is Collins et al.'s; the citation to Grieve 1987 does not support the single number.
  Evidence level: primary full text (Collins 2005; Grieve 1987 via OCR, so a missed digit is possible)

- IMP-035: coded 2 km sedimentary, 4 km crystalline (Grieve 1987). Source says (Grieve 1987 OCR): "At diameters above -2 km in sedimentary targets and -4 km in crystalline targets, terrestrial impact structures have a complex form (Figure 2)." (https://www.wipp.energy.gov/library/cra/2009_cra/references/Others/Grieve_1987_Terrestrial_Impact_Structures_ERMS241228.pdf)
  Result: MATCH
  Evidence level: primary full text

- IMP-036: coded transition proportional to 1/g (D_c = 3200 x 9.81/g), cited to Pike 1980 and Melosh 1989. C2005 gives no gravity scaling of D_c; Pike 1980 not fetched.
  Result: COULD NOT READ (Pike 1980 and Melosh 1989 not online in a readable form; ADS scans carry no text layer)
  Evidence level: none

- IMP-037: coded d_tc = D_tc/(2 sqrt 2). C2005 Fig. 3 caption: "we assume that D_tc = 2 [sqrt 2] d_tc" (the root sign was not extracted), eq. 25*, and eq. 46 reassembled "V_tc = pi D_tc^3 / (16 sqrt 2)" with the text "assume that the transient crater is a paraboloid with a depth to diameter ratio of 1:2 [sqrt 2]"; cited there to "Dence et al. (1977)". For a paraboloid V = pi D^2 d / 8, so d = D/(2 sqrt 2) gives the V_tc printed.
  Result: MATCH (the radical sign is inferred from the internal consistency of eq. 46, not read directly). The code cites no equation number at the line.
  Evidence level: primary full text (with inference)

- IMP-038: coded h_rim = 0.07 D_tc^4 / D^3, "eq. 25". C2005 eq. 48*, p. 829, reassembled: h_fr = 0.07 D_tc^4 / D_fr^3, "found by inserting r = D_fr/2 into Equation 31 [sic; the extracted text says 31, the ejecta law is eq. 47]". The paper also says rim heights of complex craters are "significantly smaller than the scaled rim heights of simple craters".
  Result: PARTIAL. Value matches; the code's equation number is wrong (25 is d_tc; the rim height is 48).
  Evidence level: primary full text

- IMP-039, 040, 041: coded V_br = 0.032 D^3 (eqs. 24 and 26); t_br = 2.8 V_br (d_tc + h)/(d_tc D^2); depth = d_tc + h_rim - t_br. C2005 eq. 23*: "V_br = 0.032 D_fr^3", "an approximate relationship based on estimates of unbulked breccia-lens volumes at Meteor Crater and Brent Crater (Grieve and Garvin 1984)"; eq. 24* reassembled: t_br = 2.8 V_br (d_tc + h_fr) / (d_tc D_fr^2), "Assuming that the top surface of the breccia lens is parabolic and that the brecciation process increases the bulk volume of this material by 10%"; eq. 26*: "d_fr = d_tc + h_fr - t_br".
  Result: MATCH on values. The V_br law is eq. 23 (code groups it under eqs. 24 and 26). The layout of eq. 24 is reassembled from fragments.
  Evidence level: primary full text

- IMP-042: coded depth = 0.294 D^0.301 km for complex craters, "eq. 28". C2005 p. 824: "To estimate the average depth d_fr (in km) from the rim to floor of a complex crater of rim-to-rim diameter D_fr (in km), we use the depth-to-diameter relationship of Herrick et al. (1997) for venusian craters: d_fr = 0.4 D_fr^0.3 (28*)", and "The similarity in surface gravity between Earth and Venus as well as the large number of fresh complex craters on Venus makes this relationship more reliable than that based on the limited and erosion-affected data for terrestrial complex craters (Pike 1980; Grieve and Therriault 2004)."
  Result: MISMATCH. The paper gives d_fr = 0.4 D_fr^0.3 (km). The coded 0.294 D^0.301 is not in the paper. At 20 km the paper law gives 0.98 km and the coded law 0.72 km (my arithmetic; ratio 0.735). I could not trace where 0.294 D^0.301 comes from. Two machine-written search snippets (not verified) attributed it variously to Pike (1980) and to Herrick et al. (1997); neither is reliable. (Note: Grieve 1987, p. 248, OCR, gives other simple-crater depth laws "d_a = 0.14 D^1.02 (Pike 1980)" and a "d_t = 0.29 D^..." law "this work", whose exponent is garbled; this is a different law.)
  Evidence level: primary full text

- IMP-062: coded V_melt = 8.9e-12 E sin theta, "eq. 12". C2005 p. 824-825: "V_m = 8.9 x 10^-12 E" (vertical, for epsilon_m = 5.2 MJ/kg and density 2700 kg/m3) and eq. 30*: "V_m = 8.9 x 10^-12 E sin theta (30*)"; "This expression works well for all geologic materials except ice. In this case, V_m is about ten times larger than for rock"; "for impact angles of ~15 degrees or less, Equation 30 probably overestimates the volume of impact melt produced by a factor of ~2". The exponent's minus sign was confirmed from the PDF font map (U+2212).
  Result: PARTIAL. Value and form match; equation number is 30 (eq. 12 is in the atmospheric-entry section). Sources named by the paper: O'Keefe & Ahrens 1982b, Pierazzo et al. 1997, Pierazzo & Melosh 2000, Grieve & Cintala 1992.
  Evidence level: primary full text

- IMP-063: coded sheet = 4 V_melt / (pi D_tc^2), eq. 31. C2005 eq. 31*, p. 825: "t_m = 4V_m / [pi] D_tc^2 (31*)" with the prose "we estimate the average thickness of this sheet t_m as the ratio of the melt volume to the area of a circle equal in diameter to the transient crater" (the pi glyph was not extracted).
  Result: MATCH
  Evidence level: primary full text

- IMP-054: coded t(r) = D_tc^4 / (112 max(r, rim)^3), "eq. 32", McGetchin 1973. C2005 eq. 47*, p. 828, reassembled: t_e = D_tc^4 / (112 r^3); derivation: h_tr = D_tc/14.1 and a falloff with "one over the distance from the crater rim cubed". Prose: "The power of 3 is a good approximation of data from explosion experiments (McGetchin et al. 1973)... which show that the power can vary between 2.5 and 3.5." Inserting r = D_fr/2 gives eq. 48's 0.07.
  Result: PARTIAL. Value matches; the equation number is 47 (eq. 32 is the fireball radius R_f = 0.002 E^(1/3)). The paper's r is distance from the crater centre and the paper only reports ejecta outside the final rim.
  Evidence level: primary full text

- IMP-020: coded speed 20 km/s as "the Earth average for asteroids" (Collins et al.; Le Feuvre & Wieczorek 2011). C2005 p. 819: asteroids "typically collide with the Earth's atmosphere at velocities of 12-20 km s^-1 (Bottke et al. 1994)"; the entry speed range is "between 11 and 72 km s^-1"; the worked examples (Table 6) use 20 km/s for all three impacts and a 17 km/s airburst case. The program has no default speed.
  Result: PARTIAL. 20 km/s is the top of Collins et al.'s asteroid range and their example value, not stated as an average. Le Feuvre & Wieczorek 2011 not read.
  Evidence level: primary full text (Collins); Le Feuvre COULD NOT READ

- IMP-021, 022: coded impactor density 3000 (stony), 7800 (iron), about 1000 (comet). C2005 p. 819: asteroids "made of rock (rho_i ~2000-3000 kg m^-3; Hilton 2002) or iron (rho_i ~8000 kg m^-3...)"; comets "rho_i ~500-1500 kg m^-3 (Chapman and Brandt 2004)".
  Result: PARTIAL. All three are inside or at the edge of the paper's ranges (3000 is the top of the stony range; iron 7800 vs about 8000). The code cites nothing for them.
  Evidence level: primary full text

- IMP-023: coded crust density 2700 (2500-2750 for sedimentary to crystalline). C2005 p. 818: "sedimentary rock, for which we assign a target density of rho_t = 2500 kg m^-3, crystalline rock (rho_t = 2750 kg m^-3), or a marine target... a target density of rho_t = 2700 kg m^-3 for the rock layer below"; the melt formula assumes "an impactor and target density of 2700 kg m^-3".
  Result: PARTIAL. 2700 appears only as the rock below water and in the melt formula; the paper's two land targets are 2500 and 2750.
  Evidence level: primary full text

- IMP-024: coded 45 degrees, "the most probable" (Gilbert 1893; Shoemaker 1962). C2005 pp. 817-818 (Introduction): "The most likely impact angle is 45 [degrees] (Shoemaker 1962)."
  Result: MATCH (value and Shoemaker attribution; the sin 2 theta d theta density is not stated in Collins et al., and Gilbert 1893 is not cited by them).
  Evidence level: primary full text

- IMP-064: coded check values: a roughly 40-m iron body makes about 1.2 km (Barringer); build log 1-km stony asteroid about 15 km. C2005 Table 6 (examples): 40-m iron (8000 kg/m3, 20 km/s, 50 percent velocity loss in the atmosphere, sedimentary target 2500): "Final crater diameter (km) Equations 21 and 22 or 27: 1.2 (Simple)"; 1.75-km stony (2700, 20 km/s, crystalline 2750, 45 degrees): "23.7 (Complex)"; 18-km stony: "186 (Complex)". My arithmetic with the code's laws (eqs. 21, 22, 27, D_c 3.2 km) on those inputs gives about 1.18 km (taking 10 km/s after the 50 percent loss), 23.8 km and 185.6 km.
  Result: MATCH. The paper's Barringer-like case is 20 km/s slowed to about half, not the 13 km/s in the unit-test comment. No 1-km stony case is given.
  Evidence level: primary full text (Table 6) plus my own arithmetic

- IMP-001: coded 5.6e-15 craters of D >= 20 km per km2 per year (+/- 2.8 unused), Grieve & Shoemaker 1994. The chapter could not be read. Grieve 1987 (OCR, cratering-rate section; page not captured) quotes the older estimate: "Average terrestrial cratering rate estimates have been calculated from well-dated, relatively young (< 120 Myr) large craters on the North American and European cratons (5.4 +/- 2.7 x 10^-15 km^-2 yr^-1 for D >= 20 km, n = 7; Grieve 1984)" (OCR rendered "5.412.7 x 10-l5 km-2 yr-l for D+) 20 km").
  Result: PARTIAL. The same quantity, 5.4 +/- 2.7 (Grieve 1984, as restated in 1987), is within 4 percent of the coded 5.6 +/- 2.8; whether 5.6 +/- 2.8 is the 1994 value is not confirmed.
  Evidence level: secondary source (Grieve 1987 quoting Grieve 1984, OCR)

- IMP-003, 004: coded slope b = 1.8, cumulative N(>=D) proportional to D^-1.8, extended below 20 km. Grieve 1987 (OCR, section on the known record; page not captured): "For large diameters the distribution approximates the relation N proportional to D^-1.8, where N is cumulative number" (the sign was lost in OCR) and the Fig. 8 caption: "Note departure from N ~ D^-1.8 at smaller diameters, which indicates a shortage of known craters due to crater recognition and retention problems."
  Result: PARTIAL. 1.8 above about 20 km is supported (as Grieve's 1987 reading of the known record); extending it below 20 km is the code's inference, in line with Grieve's remark that the known record is short of small craters, and is not itself in the source.
  Evidence level: primary full text of a different paper (Grieve 1987, OCR)

- IMP-065, 067: coded three numbers: sea removes two thirds of craters under 1 km, a third at about 30 km, hardly any over 100 km. Source says (abstract of Davison & Collins 2007): "We find that the oceans reduce the number of craters smaller than 1 km in diameter by about two-thirds, the number of craters ~30 km in diameter by about one-third, and that for craters larger than ~100 km in diameter, the oceans have little effect." (https://spiral.imperial.ac.uk/server/api/core/bitstreams/ba3d1b52-99e7-4439-926d-7a38cdfb3e7c/content)
  Result: MATCH. "Little effect" over 100 km agrees with the module comment; the sea_floor_factor comment's "a tenth at 100 km" is the code's own figure, not the paper's. The abstract also says oceans win above about 12 km: "Above a diameter of ~12 km, more craters occur on the ocean floor than on land". Note the paper's three numbers are about the whole crater population (oceans' share of area, depth distribution), not a single depth ratio.
  Evidence level: primary full text

- IMP-066: coded cutoff 6.3 impactor diameters of water. Source says (Davison & Collins 2007, Methods): "we kept the resolution constant and varied the water thickness from 0-8 times the impactor diameter (at water thicknesses in excess of ~8 L, no crater forms on the seafloor)"; Results: "for the simple crater case ... the final crater diameter decreases linearly with increasing R until no crater is formed in the seafloor"; "performing a linear regression [on R <= 3 or 4 points]... (3) [equation not extracted] ... R^2 ... 0.92"; Conclusions: "no crater is formed on the seafloor above R = 8"; and "for R > 4, no crater occurs on the seafloor" (Shuvalov 2002, as quoted). Table 2 (first column set, the 100-m impactor) lists a normalised diameter 0.103 at R = 6; the paper marks some Table 2 entries as resurge features rather than craters, and I could not tell from the extraction which. R is water depth over impactor diameter.
  Result: PARTIAL. The paper gives its own linear relation (eq. 3, R^2 = 0.92, fitted at small R) and a no-crater limit near R = 8; the code's 6.3 comes from fitting the three summary numbers, as the comment says, and is not a number in the paper.
  Evidence level: primary full text

- IMP-068: coded 3.8-km mean ocean depth and 71 percent ocean cover. Davison & Collins 2007 (introduction): "Oceans cover 70% of the Earth"; the paper also gives the maximum ocean depth as about 7 km. NOAA: "The average depth of the ocean is about 3,682 meters (12,080 feet)." (https://oceanservice.noaa.gov/facts/oceandepth.html; the percentage page returned 404.)
  Result: PARTIAL. 3.8 km vs 3.68 km; 71 vs 70 percent (rounding level). The code cites nothing for either.
  Evidence level: primary full text (Davison & Collins, cover) and secondary source (NOAA, depth)

- IMP-014: coded claim that the atmosphere breaks up most stony bodies that would make craters under 2 km (Bland & Artemieva 2006). Davison & Collins 2007: "stony impactors much smaller than 100 m in diameter are significantly affected by atmospheric entry (broken up and/or decelerated) and will probably not form a single large impact crater."
  Result: PARTIAL. Supports the cut-off idea at about 100 m (a 2-km crater needs about an 84 m stony impactor by my arithmetic); Bland & Artemieva 2006 itself was not read.
  Evidence level: primary full text (Davison & Collins); Bland & Artemieva COULD NOT READ (paywalled)

- IMP-016: coded "300 km is the largest crater known on Earth (Vredefort, 2.02 Gyr)". Wikipedia (secondary), infobox: "170-300 km (110-190 mi) (estimated former crater diameter)", age "2023 Ma, +/- 4 Ma" (https://en.wikipedia.org/wiki/Vredefort_impact_structure).
  Result: PARTIAL. 300 km is the upper end of a 170-300 km range; the age agrees with 2.02 Gyr. The Earth Impact Database page itself could not be fetched.
  Evidence level: secondary source (Wikipedia)

- IMP-083: coded fade times (2-km bowl in about 10 Myr, 20-km crater in about 1 Gyr), Culling 1960 and Fassett & Thomson 2014. Fassett & Thomson abstract (OpenAlex reconstruction): "The average diffusivity calculate[d] over the past 3 Ga is ~5.5 m2/Myr. With this diffusivity, after 3 Ga, a 1 km diameter crater reduced to approximately ~52% of its depth and 300 m only ~7% of depth".
  Result: PARTIAL. The lunar diffusivity (5.5 m2/Myr = 5.5e-6 m2/yr) is about 1800 times smaller than the code's Earth value 0.01 m2/yr, so the paper does not support the fade times; they follow from the code's own diffusivity. The paper confirms the method (diffusion model on crater profiles), not the number.
  Evidence level: primary abstract (via OpenAlex)

- IMP-074: coded diffusivity 0.01 m2/yr, "soil-mantled hillslopes measure 0.003 to 0.05 m2/yr" (Fernandes & Dietrich 1997; Roering et al. 1999).
  Result: COULD NOT READ (both papers paywalled; Wiley and AGU pages returned 403 or were not tried)
  Evidence level: none

- IMP-072: coded Gaussian of variance 2 kappa t (Culling 1960; Soderblom 1970; Fassett & Thomson 2014).
  Result: COULD NOT READ (Culling 1960 and Soderblom 1970 paywalled; Fassett & Thomson abstract confirms a diffusion model for crater degradation only)
  Evidence level: primary abstract (Fassett & Thomson only)

- IMP-045: coded peak-ring onset 40 km on Venus (Alexopoulos & McKinnon 1994).
  Result: COULD NOT READ (GeoScienceWorld returned 403)
  Evidence level: none

- IMP-048: coded central-peak diameter 0.22 D (Pike 1985).
  Result: COULD NOT READ (ADS scan is image-only, JBIG2, no text layer)
  Evidence level: none

- IMP-005: coded rate constant within a factor of 2 for the last about 3 Gyr (Neukum et al. 2001).
  Result: COULD NOT READ (Springer paywall; Crossref metadata only)
  Evidence level: none

## Volcanoes and magma (VOL, MAG)

Results: MATCH 4, PARTIAL 10, MISMATCH 3, COULD NOT READ 4.

Earth figures with no source in the repo were looked up; the figure each source gives is recorded below. All Wikipedia figures are "secondary source".

#### Citation and the code's 'flags' (earth/VOL.md flags 1 to 5)

- VOL-039: coded arc spacing 60 km (50-70 km "real fronts"). Source says (abstract of Bremond1995): "Volcano spacing follows a random distribution" and the Gamma distribution corresponds "to randomly generated points"; also "no characteristic spacing". (Crossref and OpenAlex abstracts via WebFetch; Wiley page 403.)
  Result: PARTIAL (the paper's abstract says there is no characteristic spacing; it gives no 50-70 km figure. The full text may tabulate mean spacing; not read)
  Evidence level: primary abstract

- Flag 1 / VOL-016: coded profile (1 - rho)^1.5, comment "upper flanks ~20 degrees at 2.5 km over R = 14 km"; the earth file computes the profile's steepest slope as 15 degrees. Source for real Fuji says: "Upper flanks display 31 degrees-35 degrees angles" and "Fuji's main crater is 780 m in diameter and 240 m deep" (Wikipedia Mount Fuji, which cites Britannica for height and a geological cross-section for the crater).
  Result: MISMATCH of the comment against a real stratovolcano (Fuji's upper flanks are about 31-35 degrees, steeper than both the comment's 20 degrees and the formula's 15 degrees; the arithmetic of flag 1 is a separate matter and stands). The code's cones are far gentler than Fuji by design at the default flank setting.
  Evidence level: secondary source (Wikipedia)

- Flag 2 / VOL-017: coded shield "flanks under ~10 degrees". Source says: Mauna Loa "slopes are about 12 degrees at their steepest, a consequence of its extremely fluid lava" (Wikipedia Mauna Loa).
  Result: PARTIAL (real steepest slope of the biggest shield is about 12 degrees, above the comment's 10; the formula gives about 14 degrees for the tallest shield before the flank setting)
  Evidence level: secondary source (Wikipedia)

- Flag 3 / VOL-048: coded island subsidence 25 m/Myr, cap 1,500 m; comment "Hawaiian islands sink ~2 km over ~30 Myr" (= 67 m/Myr). Source says: "Estimated average subsidence of the site northwest of Hawaii is 2.7 mm/yr" (Szabo and Moore 1986, abstract via OpenAlex). 2.7 mm/yr is 2,700 m/Myr. Wikipedia Mauna Kea: the volcano "continues to slip and flatten under its own weight at a rate of less than 0.01 in (0.2 mm) per year".
  Result: MISMATCH in scale. The young, actively loaded island of Hawaii sinks about 2.7 mm/yr, 100 times the coded 25 m/Myr and 40 times the comment's 67 m/Myr. The comment's "2 km over 30 Myr" was not found in any source. The fast phase is a loading-flexure effect on young islands; old islands sink much more slowly and no figure for them was read.
  Evidence level: primary abstract (Szabo and Moore 1986); secondary (Wikipedia, Mauna Kea 0.2 mm/yr)

- Flag 4 / VOL-029: coded caldera radius 3-9 km (6-18 km across); comment "Crater Lake 4-5, Long Valley 8-16, Yellowstone 30-45 across". Sources say: Crater Lake "5 by 6 miles (8.0 by 9.7 km) across" (Wikipedia Crater Lake; collapse "around 7,700 (+-150) years ago"); "The 16 x 32 km (20 x 10 mi) Long Valley caldera" formed about 760,000 years ago (USGS Long Valley page, fetched); Yellowstone Caldera "about 34 miles (55 km) by 45 miles (72 km) wide" (Wikipedia Yellowstone hotspot, citing Global Volcanism Program).
  Result: MISMATCH of the comment's wording. The comment's numbers are semi-axes (radii), not widths: Crater Lake radius is 4.0-4.9 km, Long Valley 8 and 16 km, and the comment's Yellowstone 30-45 sits close to the real semi-axes (27-36 km). The real widths are 8-10, 16-32 and 55-72 km. The coded radius range 3-9 km (and the 0.6 cap) therefore holds Crater Lake only; Long Valley's long axis (radius 16 km) and Yellowstone (radius 27-36 km) are far outside it.
  Evidence level: secondary source (Wikipedia, with GVP cited for Yellowstone); primary agency page for Long Valley

- Flag 5 / VOL-026: coded summit crater radius at least 1 km (2 km across); comment "real ones are 0.3-1 km". Sources say: Fuji main crater "780 m in diameter and 240 m deep" (Wikipedia Mount Fuji); Cotopaxi "800 m x 550 m wide crater which is 250 m deep" (Wikipedia Cotopaxi); Mount St. Helens 1980 crater "2 to 3 km wide and 635 m deep" (Wikipedia Mount St. Helens).
  Result: PARTIAL (summit craters of stratovolcanoes run 0.5-1 km across, 2-3 km for a collapse crater; the code's 2-4 km across for a cone is a collapse-crater size, as flag 5 says)
  Evidence level: secondary source (Wikipedia)

#### Volcano sizes (VOL-016 to VOL-023, VOL-032)

- VOL-019: coded cone height 1.2-3.0 km; comment: Fuji 3.8 km a.s.l., Cotopaxi about 2.9 km above the Andean plateau, Shasta about 3 km above its base. Sources say: Fuji "3,776.24 m" (Wikipedia, citing Britannica); Cotopaxi 5,897 m, rising "from a highland plain of about 3,800 m" (Wikipedia Cotopaxi), i.e. about 2.1 km above its plain; Shasta "towers nearly 9,761 feet (3,000 m) above its surroundings" (Wikipedia Mount Shasta); Rainier "more than 13,000 feet (4,000 m) above its surroundings" (Wikipedia Mount Rainier).
  Result: PARTIAL (Fuji and Shasta match; Cotopaxi is about 2.1 km above its plateau, not 2.9 km; Rainier at 4 km relief is above the code's 3 km maximum)
  Evidence level: secondary source (Wikipedia)

- VOL-020: coded basal radius 14 km at 3 km height; comment: Fuji about 18 km, Rainier about 10, Shasta about 14. Source says: Cotopaxi "a width at its base of about 23 km" (Wikipedia Cotopaxi). No source with a base width for Fuji, Rainier or Shasta was found (the Fuji, Rainier and Shasta pages gave none).
  Result: COULD NOT READ for Fuji, Rainier, Shasta (no figure found). Cotopaxi's 23 km width (radius 11.5 km on a 2.1-km cone) is in the code's range.
  Evidence level: secondary source (Wikipedia)

- VOL-021: coded shield height 3.5-6.5 km; comment: Hawaii 5-9 km above the sea floor, Reunion about 7, Galapagos 3-4. Source says: Mauna Loa "(5,000 m) to the sea floor" below sea level and "4,170 m subaerial height", total "9,170 m (30,085 ft)"; Mauna Kea "prominence from under the ocean is 30,610 ft (9,330 m)" (Wikipedia Mauna Loa and Mauna Kea; USGS Mauna Loa page: "an additional 5 km" below sea level, 4,169 m summit). Reunion: the Wikipedia page gave Piton des Neiges 3,070 m but no height above the sea floor. Galapagos not fetched.
  Result: PARTIAL (the 9 km top of the Hawaii range is confirmed; Reunion and Galapagos not read)
  Evidence level: secondary source (Wikipedia) and primary agency page (USGS, for Mauna Loa)

- VOL-022: coded shield radius 40 km at 6.5 km; comment: Mauna Loa about 60, Mauna Kea about 40. Source says: Mauna Loa "covers a land area of 5,271 km2 and spans a maximum width of 120 km" (Wikipedia); USGS: the subaerial part extends "about 120 km from the southern tip of the island to the summit caldera". The subaerial part stands on the sea floor with a much larger submarine footprint; no Mauna Kea width found.
  Result: PARTIAL (60 km radius for Mauna Loa matches the subaerial width; the full base is larger; Mauna Kea not found)
  Evidence level: secondary source (Wikipedia) and primary agency page (USGS)

- VOL-023: coded silicic centre 0.9 km tall, 35 km radius; comment: Yellowstone plateau about 1 km above the Snake River Plain over about 70 km. Source says: nothing about the plateau's height above the plain (Wikipedia Yellowstone Caldera and hotspot pages give caldera 55 x 72 km; "highest point 2,805 m").
  Result: COULD NOT READ (no figure found for the plateau elevation difference; a 70-km span is consistent with a 55 x 72 km caldera)
  Evidence level: secondary source (Wikipedia)

- VOL-032: coded ceiling 6.8 km; comment: Ojos del Salado 6.9 km, Mauna Kea 4.2 km. Sources say: Ojos del Salado "6,893 m" (also 6,879 and 6,887 m cited; a Padua GPS survey 6,900 +-5 m), "the world's highest volcano" (Wikipedia Ojos del Salado, citing GVP); Mauna Kea "13,803 ft (4,207 m)" (Wikipedia).
  Result: MATCH
  Evidence level: secondary source (Wikipedia)

- VOL-005: coded silicic crust threshold 42 km; comment "the Central Andes' ignimbrite flare-up sits on 60-70 km crust; normal crust is ~35". Source says: "the crust thickened to 60-70 kilometres" during the flare-up and the complex formed "over 70-kilometre-thick" crust (Wikipedia Altiplano-Puna volcanic complex).
  Result: MATCH for the 60-70 km; the 35 km "normal" figure not read
  Evidence level: secondary source (Wikipedia)

- VOL-007: coded hotspot spacing 70 km; comment "Hawaiian volcanoes line up ~40-80 km apart". Not looked up (no source fetched).
  Result: COULD NOT READ (not fetched)
  Evidence level: none

#### Hotspot track, lifetimes, chambers, lava lakes, flood basalts

- Run-review hotspot reference (earth file flag 12): "Hawaii-Emperor is about 6,000 km over about 80 Myr (from memory)". Source says: the chain "stretches about 6,200 km", the oldest confirmed Emperor age "is 81 million years, for Detroit Seamount. However, Meiji Seamount...is likely somewhat older", plate motion "around 60 mm per year" for 81-43 Myr, bend at "approximately 47 million years ago" (Wikipedia Hawaiian-Emperor seamount chain, citing Sharp and Clague 2006 and Tarduno et al.).
  Result: MATCH (6,200 km over 81 Myr; 6,200/81 = 76 mm/yr by simple division, the page gives 60 mm/yr for the Emperor leg)
  Evidence level: secondary source (Wikipedia)

- Stratovolcano lifetime: the code stands 2-8 Myr for a "centre" against "a real cone's 0.1 to 1 Myr". Sources say: Rainier's present cone is "more than 500,000 years old" with earlier deposits "more than 840,000 years old"; Shasta "About 593,000 years ago, andesitic lavas erupted in what is now Mount Shasta's western flank"; Fuji's modern Shin-Fuji "around 10,000 years ago" (Wikipedia pages).
  Result: PARTIAL (cone ages of 0.01-0.6 Myr sit in the claimed 0.1-1 Myr band; no review of stratovolcano lifetimes was read; Hildreth 2007's abstract has none)
  Evidence level: secondary source (Wikipedia)

- MAG-002: coded chamber top 3 km; comment "Kilauea's about 2-4 km, Mount St Helens' from 5 km, Vesuvius' shallow one about 4-5 km", "real shallow chambers sit 2-6 km". Sources say: Kilauea deformation from "a volumetric source at 1.2 km depth" and "a sill-shaped source located at a depth of 3.5 km" (Przeor et al. 2022, abstract relayed by Europe PMC); St Helens "two chambers below the volcano: one approximately 3-7 miles (5-12 km) below the surface, the other about 7-25 miles (12-40 km)" (Wikipedia Mount St. Helens); Vesuvius reservoirs "migrated upward from depths of 7-8 km to 3-4 km between the AD 79 and AD 472 eruptions" (Scaillet et al. 2008, abstract relayed by Europe PMC).
  Result: PARTIAL (Kilauea 1.2-3.5 km vs coded 2-4; St Helens 5 km upward matches; Vesuvius shallow chamber 3-4 km vs coded 4-5)
  Evidence level: primary abstract (Przeor, Scaillet); secondary (Wikipedia, St Helens)

- MAG-020: coded "about eight of some 1,400 active volcanoes" with persistent lava lakes. Sources say: "There are about 1,350 potentially active volcanoes worldwide, aside from the continuous belts of volcanoes on the ocean floor" (USGS FAQ, fetched); persistent or near-persistent lava lakes listed in Wikipedia Lava lake: Erebus, Erta Ale, Kilauea, Masaya, Mount Michael, Nyiragongo, Yasur, Tofua (eight names), with "Only a few volcanoes have hosted persistent or near-persistent lava lakes during recent decades"; Ambrym's lakes "disappeared after a large eruption in December 2018".
  Result: MATCH on the order (8 listed of 1,350); the code's 1,400 is a rounded-up USGS 1,350, and the count of eight is a Wikipedia list, not a count from a review
  Evidence level: primary agency page (USGS, count); secondary (Wikipedia, list)

- VOL-052 flood basalt size and thickness: coded pile 1,500 m, radius 900 km (2.5 million km2 as a disc); run-review says "the size of the Deccan or Siberian Traps", 2.0-2.7 million km2. Sources say: Deccan "cover an area of about 500,000 square kilometres" now, "Originally...may have covered about 1.5 million km2", thickness "more than about 2 kilometres", volume about 1 million km3 (Wikipedia Deccan Traps); Siberian Traps "about 7 million km2" and "around 4 million km3" of basaltic rock, eruptions "roughly two million years" (Wikipedia Siberian Traps, citing Ivanov et al. 2013); Wikipedia Large igneous province (citing Ross et al. 2005) gives Deccan 0.5-0.8 million km2 and Siberian 1.5-3.9 million km2.
  Result: PARTIAL (the coded area of 2.5 million km2 is 1.7-5 times the Deccan's original or present area and inside the range quoted for the Siberian Traps; thickness 1.5 km is in line with the Deccan's "more than about 2 km"; the same page set quotes Siberian area as 1.5-3.9 and as 7 million km2, so the sources disagree)
  Evidence level: secondary source (Wikipedia)

- Flood-basalt frequency, "Earth averages one per 20-30 Myr, so 33-50" per 1,000 Myr (earth file). Source says: "The 16 traps reviewed" run from the Columbia basalts "at ~16 Ma to possible Siberian remnants at ~360 Ma", and "The sequence of LIP ages appears random" (Courtillot and Renne 2003, abstract). Sixteen traps in about 345 Myr is one per about 22 Myr (my arithmetic on their numbers, not a figure in the paper).
  Result: PARTIAL (consistent with 20-30 Myr, derived by division; the paper itself does not state a recurrence interval in the abstract; the run's 17 per 1,000 Myr is a third below this rate)
  Evidence level: primary abstract

- VOL-036: coded step 80 m; comment "Deccan flows are 10-100 m thick". Source says: nothing on individual flow thickness was found (Wikipedia Deccan Traps: "more than about 2 kilometres" total; Self et al. 2007 abstract unavailable).
  Result: COULD NOT READ
  Evidence level: none

#### Not bibliographic
Minecraft 1.21.1 classes (NoiseBasedChunkGenerator.createFluidPicker, Aquifer.*), Terralith and vanilla features, the deposit engine `arc_magma_flux`, `plates/hotspots.rs` (cites Cortial et al. 2019 s7.5, ledgered by the hotspot agent), the decisions-log entries, `docs/research/notes/*` and FORMAT.md, and the build-log run ids. Named volcanoes listed in the sources file (Fuji, Cotopaxi, Shasta, Rainier, Mauna Loa, Mauna Kea, Yellowstone, Crater Lake, Long Valley, Ojos del Salado, Kilauea, St Helens, Vesuvius, Deccan) are examples, handled under Part 2.

## Erosion, isostasy and hypsometry (ERO, ISO, HYP)

Results: MATCH 12, PARTIAL 4, MISMATCH 2, COULD NOT READ 8.

Labels S1 to S27 are used in Part 1. Ledger ids in the first column.

- S1 (ERO-001, ERO-079, HYP-030, ERO-078): coded implicit (backward Euler) solve in stack order, cited to Braun & Willett 2013. Source says: could not read the paper. Secondary: fastscapelib `spl.hpp`: "It numerically solves the Stream Power Law (SPL) using an implicit finite difference scheme 1st order in space and time. The method is detailed in Braun and Willet's (2013) and has been slightly adapted." (https://raw.githubusercontent.com/fastscape-lem/fastscapelib/main/include/fastscapelib/eroders/spl.hpp); Landlab `fastscape_stream_power.py` line 23: "This class uses the Braun-Willett Fastscape approach to calculate the" (and line 31: "(see Braun & Willett 2013, Appendix B for further details)").
  Result: PARTIAL (the scheme and the paper are as attributed according to two software packages; the paper's equations, the stack order and the analytic steady state of HYP-030 were not read; COULD NOT READ for the paper itself, ScienceDirect 403)
  Evidence level: secondary source (fastscapelib header, Landlab docstring)

- S2 (ERO-003, ERO-004, ERO-005, ERO-008): coded K 2e-9 (Rust default) and 3e-9 (all presets), m = 1, n = 2, units m^(1-2m)/yr. Source says: Tzathas et al. 2024, Table 1: "Fluvial erosion k 2 x 10^-5 m^(1-2m) y^-1", "Drainage exponent m 0.4" (the PDF text reads "Fluvial erosion k 2 10 5 m 1 2 m y 1"; "Drainage exponent m 0 : 4"); fastscapelib has no default K, m or n (only `tolerance = 1e-3`); Landlab FastscapeEroder defaults `K_sp=0.001, m_sp=0.5, n_sp=1.0` (https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf; fastscapelib spl.hpp; Landlab fastscape_stream_power.py lines 194-196).
  Result: PARTIAL (the unit convention m^(1-2m)/yr is the one Tzathas uses and is independent of n; the numeric values 2e-9 and 3e-9 have no external source and are tuned; the ranges of K, m, n in Whipple & Tucker 1999 and Stock & Montgomery 1999 could not be read, paywalled)
  Evidence level: primary full text (Tzathas, units and example values); software source (defaults)

- S3 (ERO-002, ERO-009): coded G = 1 (Yuan deposition coefficient, default on). Source says: only the first abstract sentence was reachable: "The stream power law model has been widely used to represent erosion by rivers but does not take into account the role played by sediment in modulating erosion and deposition rates." (https://publishup.uni-potsdam.de/frontdoor/index/index/docId/49084). The equation dh/dt = U - K A^m S^n + (G/A) Q_in and the values of G used in the paper were not seen.
  Result: COULD NOT READ (Wiley paywall; HAL behind Anubis; Potsdam page is metadata only)
  Evidence level: search snippet (abstract sentence, page content)

- S4 (ERO-006): coded n = 2, with the comment "worldwide 10Be denudation rates favour n > 1, Harel, Mudd & Attal 2016". Source says: not read. Only the title was verified: "Global analysis of the stream power law parameters based on worldwide 10Be denudation rates" (Crossref).
  Result: COULD NOT READ (Elsevier paywall; the Edinburgh author manuscript returns 403 to WebFetch and curl)
  Evidence level: title only

- S5 (ERO-069, ERO-070, ERO-068, ERO-099): coded soil production w0 = 50 m/Myr (50 mm/kyr) at 1,000 mm/yr rain and 15 C, decay depth H* = 5 m. Source says: Heimsath et al. 1997 abstract (verbatim): "Second, we calculate an exponential decline of soil production rates with increasing soil depth from measurements of the in situ produced cosmogenic 10Be and 26Al concentrations in bedrock sampled under soils of different depths." The abstract gives no numbers (https://www.nature.com/articles/41056, abstract). The parameters were found only in secondary sources: Wikipedia "Soil production function": "The value of P0 was found to range from 0.08 to 2.0 mm/yr for sites in northern California, and 0.05-0.14 mm/yr for sites in southeastern Australia. Meanwhile values of k do not vary significantly, ranging from 2 to 4 m-1." (equation de/dt = P0 exp[-kh]) (https://en.wikipedia.org/wiki/Soil_production_function). That is a decay depth 1/k of 0.25 to 0.5 m. The SERC teaching vignette "The humped soil production function" (https://serc.carleton.edu/vignettes/collection/37905.html, retrieved via WebFetch) gives, for what the fetch summary calls the "Tin Camp Creek, Australia findings", "SPR = 47e^-0.02*H" (SPR in m/Ma, H in cm), which is w0 = 47 m/Myr with a decay depth of 50 cm = 0.5 m; a second vignette (collection/37906) says "the soil production function defined by these data show a maximum soil production rate of about 200 m/Ma".
  Result: MISMATCH (secondary sources give a decay depth of about 0.25 to 0.5 m, the code uses 5.0 m, a factor of 10 to 20 too deep; w0 = 50 m/Myr is inside the range the secondary sources give, roughly 47 to 2,000 m/Myr, so that half is plausible). Caveat: the numeric evidence is secondary (Wikipedia and teaching vignettes), not the paper's text, which could not be read; Wikipedia's ranges pool several sites and papers, and the 47 m/Myr, 0.5 m example is an Australian site, so none of these is necessarily the 1997 paper's own fit. Consequence for the code: H* = 5 m means soil production is nearly flat down to several metres, so bare rock turns to regolith far faster than the field data imply once cover exists; the effect on land elevations depends on how much regolith is stripped each step.
  Evidence level: secondary source (Wikipedia; SERC vignette); primary abstract (no numbers)

- S6 (ERO-068): coded H(t+dt) = H* ln(exp(H/H*) + w0 dt/H*), "Landlab ExponentialWeathererIntegrated". Source says (Landlab `exponential_weathering_integrated.py`): signature `soil_production__maximum_rate=1.0, soil_production__decay_depth=1.0, soil_production__expansion_factor=1.0`; "w = w_0 exp(-d / d_star)"; code (lines 199-210) `self._soil_prod_rate[...] = self._w0 * np.exp(...)` and `self._soil_prod_total[...] = self._wstar * np.log(self._fexp * self._soil_prod_rate[...] * dt / self._wstar + 1)` (arguments spread over several lines), i.e. w0 exp(-d/d*) integrated over dt. `exponential_weathering.py`: "Uses exponential soil production function in the style of Ahnert (1976)."
  Result: MATCH on the equation (with expansion factor 1 the code's closed form is algebraically the same increment: H + H* ln(1 + (w0 dt/H*) e^(-H/H*))). Landlab's numeric defaults are 1.0 and 1.0 (placeholders), so Landlab provides no value for ERO-069 or ERO-070.
  Evidence level: primary full text (source code)

- S7 (ERO-057, ERO-062, ERO-058 to ERO-061): coded G = G_m tanh(I0 exp(-k z)/I_k) x smooth_heaviside(z, tidal range), depth floored at -2 x tidal range; G_m 10 mm/yr, k 0.1 per m, I0 2,000, I_k 400. Source says (Landlab `carbonate_producer.py`, raw file): "def __init__(self, grid, max_carbonate_production_rate=0.01, extinction_coefficient=0.1, surface_light=2000.0, saturating_light=400.0, tidal_range=0.0,)"; docstring "G = G_m \tanh (I_0 e^{-kz} / I_k)"; "self._carb_prod_rate *= smooth_heaviside(self._depth, width=self.tidal_range)"; "self._depth.clip(min=-2.0 * self._tidal_range - _EPSILON, out=self._depth)"; `smooth_heaviside`: "np.divide(1.0, (1.0 + np.exp(-2.0 / width * np.asarray(x))), ...)"; docstring: "Bosscher and Schlager (1992) suggest plausible values or ranges for these parameters as follows: G_m = 10 to 15 mm/y, I_0 = 2,000 to 2,250 micro Einsteins per square meter per second in the tropics, I_k = 50 to 450 micro Einsteins per square meter per second, and k = 0.04 to 0.16 m^-1 (corresponding to a decay depth of 6.25 to 16 m). The default values used in this component are based on these estimates." (https://raw.githubusercontent.com/landlab/landlab/master/src/landlab/components/carbonate/carbonate_producer.py)
  Result: MATCH for all five numbers (0.01 m/yr = 10 mm/yr; 0.1; 2000; 400; units micro Einstein per m2 per s) and for the logistic of width tidal range and the depth floor. Notes: Landlab's carbonate tidal_range default is 0.0, whereas the mod uses 2.0 for both components (a Deep Time choice, taken from the submarine diffuser); the ranges in the docstring are Landlab's reading of Bosscher & Schlager, not verified against that paper.
  Evidence level: primary full text (source code); secondary for Bosscher & Schlager's ranges (via Landlab)

- S8 (ERO-045): coded D = D0 f1 f2 + land (1 - f1), f1 = (tanh(z / tidal range) + 1)/2, f2 = exp(-(z - wave_base)/wave_base) for z > wave_base else 1. Source says (Landlab `simple_submarine_diffuser.py`): docstring "D(h) = D_0 f_1(h) f_2(h)"; "f_1(h) = \exp( -(h - h_w) / h_w)" for h above the wave base depth; "f_2(h) = (\tanh ( -h / R_t) + 1) / 2" (docstring sign in elevation convention); code: `df = (np.tanh(self._inverse_tidal_range * water_depth) + 1.0) / 2.0` and `k[deep_water] *= np.exp(-(self._depth[deep_water] - self._wave_base) / self._wave_base)`; `_TINY_DIFFUSIVITY = 1.0e-20` on land (https://raw.githubusercontent.com/landlab/landlab/master/src/landlab/components/marine_sediment_transport/simple_submarine_diffuser.py)
  Result: MATCH (same two factors; Landlab names the exponential f_1 and the tanh f_2, the mod swaps the labels, and the mod adds the land term).
  Evidence level: primary full text (source code)

- S9 (ERO-046): coded D0 100 m2/yr. Source says: "shallow_water_diffusivity=100.0" and docstring "Diffusivity coefficient for shallow water (m2 / y) (default 100)".
  Result: MATCH
  Evidence level: primary full text (source code)

- S10 (ERO-047, ERO-048): coded wave base 60 m, tidal range 2 m. Source says: "wave_base=60.0", "tidal_range=2.0"; docstring "Wave base (m) (default 60)" and "Tidal range (m) (default 2)" (same file as S8).
  Result: MATCH
  Evidence level: primary full text (source code)

- S11 (ERO-022, ERO-049): coded faces take the arithmetic mean of the two cells' diffusivities, "as Landlab's `LinearDiffuser` maps node diffusivities to links" (diffusion.rs module doc). Source says (Landlab `diffusion/diffusion.py`, line 420): `kd_links = self._grid.map_max_of_link_nodes_to_link(self._kd)`; docstring line 156: "mapped to links using an upwind scheme in the simple case." (https://raw.githubusercontent.com/landlab/landlab/master/src/landlab/components/diffusion/diffusion.py)
  Result: MISMATCH (Landlab takes the maximum of the two nodes, an upwind choice; the mod uses the arithmetic mean and says it copies Landlab). With the maximum, a coastal land cell would feel the full marine D0, not about D0/2; the ERO-049 observation ("the sea erodes a coastal land cell at about D0/2") is a property of the mod's mean, not of Landlab. SimpleSubmarineDiffuser inherits from LinearDiffuser, so the same rule applies there. Version caveat: master as of 2026-10-01.
  Evidence level: primary full text (source code)

- S12 (ERO-012, ERO-027, ERO-013): coded critical slope S_c cut-back S(S > S_c) = S_c, rock removed becomes river sediment (Landlab lets it dissolve), S_c = 0.35. Source says (Landlab `threshold_eroder.py`): "Threshold eroder that cuts off slopes at a given threshold slope (Sc) and assumes material to dissolve away"; "S(S>Sc) = Sc"; `def __init__(self, grid, slope_crit=1.0)`. Tzathas 2024: "where s c = atan(30 deg) is the critical slope" and Table 1 "Critical slope s c 0.57".
  Result: MATCH for the law and for "dissolve away" (so the mod's change to conserve mass is a real change). The value 0.35 is not Landlab's (default 1.0) and not Tzathas's (0.57 = atan 30 deg); it is uncited and tuned.
  Evidence level: primary full text (Landlab source; Tzathas text)

- S13 (ERO-023): coded hillslope diffusivity kappa 2.0 m2/yr for crystalline basement; the brief gives a typical 0.001 to 0.01 m2/yr. Source says (Tzathas 2024, Table 1): "Hillslope erosion k h 0.1 m2 y-1" (a terrain-generation parameter, not a field measurement). Roering et al. 1999 and Fernandes & Dietrich 1997 could not be read.
  Result: PARTIAL (the code value is uncited and 20 times the one value in a primary text that was readable; the literature range in the brief is not confirmed by a source I could read; the mod's cell is 4.8 km, which may justify a larger effective value but no document says so)
  Evidence level: primary full text (Tzathas, one value); the field ranges COULD NOT READ

- S14 (ERO-080): coded Priority-Flood+epsilon (Algorithms 3 and 4, pits on a FIFO queue), next representable value above the neighbour. Source says (Barnes et al., arXiv 1511.04463): "Algorithm 3 Priority-Flood+epsilon: This variation of the Improved Priority-Flood ensures that all filled depressions have surfaces with a height difference of at least epsilon between any two consecutive cells ..."; "Algorithm 2 Improved Priority-Flood: This improvement to the Priority-Flood uses a plain queue to speed-up the filling of depressions"; "Algorithm 4 Priority-Flood+FlowDirs: This variation of Priority-Flood is based on the work of Metz et al. ... determines flow directions for all cells"; the algorithm uses `NextAfter(DEM(c), 1)`. (The epsilon symbol is lost in the PDF text extraction; the words are verbatim apart from it.) (https://arxiv.org/pdf/1511.04463)
  Result: MATCH (epsilon variant on a plain queue, and the flow-direction variant; NextAfter matches the f64 next_up). Preprint numbering; the journal's may differ.
  Evidence level: primary full text (preprint)

- S15 (ISO-011, ISO-012, ISO-013, ISO-017, ISO-016): coded GDH1 depths 4.2 / 5.0 / 5.5 km at 20 / 50 / 100 Myr quoted from the build log; floor -5,600 m; GDH1 rate 0.215 mm/yr; heat flow 510/sqrt(age) mW/m2. Source says: the Nature abstract was reachable only for its first sentence: "Variations in sea-floor depth and heat flow with age provide the main constraints on the thermal structure and evolution of the oceanic lithosphere." (https://www.nature.com/articles/359123a0, abstract). No equation or coefficient reachable.
  Result: COULD NOT READ (Nature paywall; no open copy; Hasterok 2013 and Grose & Afonso 2013, which reprint GDH1, are also closed)
  Evidence level: search snippet (first abstract sentence)

- S16 (ISO-014): coded ridge crest depth -2,500 m, "Parsons & Sclater 1977 (2.5 km)".
  Source says: not read.
  Result: COULD NOT READ (AGU paywall, no open copy found)
  Evidence level: none

- S17 (HYP-007): coded mean sea depth 3,683 m on GMT, "Charette & Smith give 3,682 m, ocean 361.84 M km2 = 70.9 %". Source says: "is analysis resulted in a mean ocean depth value of 3682 m."; Table 1 last row "3682.2 361.84 1.3324 This study" (mean depth m, area 10^6 km2, volume 10^9 km3); Table 2 "Mean or Total 3682.22 361.841 1.33238" (https://tos.org/oceanography/assets/docs/23-2_charette.pdf, text pulled from the PDF).
  Result: MATCH on 3,682 m and 361.84 M km2. The 70.9 % is the project's own division by Earth's surface area (361.84 / 510.07 = 70.9 %), which the paper does not state; the 510 M km2 figure is not from this paper.
  Evidence level: primary full text

- S18 (HYP-001, HYP-006, HYP-010, HYP-009): coded Earth land share 0.289, sea median depth 4,062 m, "ocean mode -4 to -3 km", land mode 0 to +1 km. Sources: (a) Wikipedia "Earth": "The remaining 29.2% of Earth's crust is land" and land 148,940,000 km2, water 361,132,000 km2 (the infobox cites Pidwirny 2006); (b) NOAA ETOPO1 note (Eakins & Sharman 2012): "The histogram reveals two primary elevation groupings: the continents, several hundred meters above sea level, and the oceanic abyssal plains, roughly 4,300 meters below sea level."; (c) NOAA "World Ocean Volumes" table: Pacific Ocean area 161,760,000 km2 = 44.7 % of ocean area, volume 660,000,000 km3 = 49.4 %, avg depth 4080 m (from which the world ocean is about 362 M km2 and about 1.34 billion km3, my arithmetic, not stated in the table). (https://en.wikipedia.org/wiki/Earth; https://www.ncei.noaa.gov/sites/default/files/2023-01/Hypsographic%20Curve%20of%20Earth%E2%80%99s%20Surface%20from%20ETOPO1.pdf; https://www.ncei.noaa.gov/sites/default/files/2023-01/World%20Ocean%20Volumes.pdf)
  Result: land share 0.289 against 0.292: MATCH within 0.003 (secondary source; different dataset). Ocean mode: MISMATCH (the ETOPO1 histogram's abyssal mode is about 4.3 km below sea level, i.e. in the -5 to -4 km band; the ledger's own 13 Earth bands (HYP-009) have their peak, 0.229, in the -5 to -4 km band, not -4 to -3 km (0.170); the stated "-4 to -3 km" in HYP-010 is off by one band). Sea median 4,062 m: COULD NOT READ (ETOPO1 note has a mode, not a median; the median is the project's own computation on SRTM15+ and is not independently sourced). Land mode 0 to 1 km: consistent with the project's band table (0.216 in the 0 to +1 km band) and with "several hundred meters".
  Evidence level: secondary source (Wikipedia, land share); primary full text (NOAA, ocean mode)

- S19 (HYP-005, HYP-025, HYP-004, HYP-003, HYP-002): coded land mean elevation 784 m (textbook about 840 m), land above 2 km 0.102, median 427 m, p90 2,022 m; Koppen land shares B 30.2, D 24.6, A 19.0, C 13.4, E 12.8 %. Source says: none of these were found in a source I could read. Peel et al. 2007 HTML abstract: "BWh (14.2%, Hot desert) followed by Aw (11.5%, Tropical savannah)". The NOAA hypsographic note's Table 1 is an image and was not readable.
  Result: COULD NOT READ for all (the elevation percentiles are the project's computations on SRTM15+ and are not published as such; the Koppen group shares were not located in the text extracted from the Peel PDF, which is only partly decodable)
  Evidence level: primary abstract (Peel, climate-type shares only)

- S20 (HYP-028, HYP-029): coded fastscapelib 0.3.0 as an oracle; K 2e-6, m 0.5, n 1, U 1 mm/yr, "fastscapelib defaults". Source says: `FG& flow_graph, K&& k_coef, double area_exp, double slope_exp, double tolerance = 1e-3` (K, area exponent and slope exponent are required arguments, only the Newton tolerance has a default); header: "The method is detailed in Braun and Willet's (2013) and has been slightly adapted."; LICENSE: "GNU GENERAL PUBLIC LICENSE Version 3, 29 June 2007".
  Result: MATCH (GPL-3.0 as stated); PARTIAL on "defaults": there are no fastscapelib K, m, n defaults to compare, the oracle values are the project's own and the 256 x 256, dx 4.8 km, K 2e-6 are not copied from the library.
  Evidence level: primary full text (source)

- S21 (ERO-073, ERO-074, ERO-075): coded laterite rule "hot and wet with dry seasons", bauxite on felsic parent in very wet climate. Source says (Wikipedia "Laterite"): "intensive and prolonged weathering of the underlying parent rock, usually under conditions of high temperatures and heavy rainfall alternating with dry periods"; "An essential feature for the formation of laterite is the repetition of wet and dry seasons."; "bauxites are derived from granitic igneous rock and other iron-poor rocks" (https://en.wikipedia.org/wiki/Laterite).
  Result: MATCH on the qualitative conditions (the numbers T > 20 C, P > 1,200 mm, slope < 0.01 are the project's "design values")
  Evidence level: secondary source (Wikipedia)

- S22 (ISO-001, ISO-008, ISO-009, ISO-010): coded crust 2,700, mantle 3,300 kg/m3 (Airy ratio 0.8; factor 5.5), crust 2,800 in the spread law. Source says (Wikipedia "Isostasy", citing Kearey, Klepeis & Vine 2009, Global Tectonics, 3rd ed., pp. 42-45): "where rho m is the density of the mantle (ca. 3,300 kg m-3) and rho c is the density of the crust (ca. 2,750 kg m-3)"; root for positive topography "b 1 ~ 5 h 1" (https://en.wikipedia.org/wiki/Isostasy). Wikipedia "Earth" lists continental crust density "2.2-2.9" g/cm3.
  Result: PARTIAL (3,300 matches; crust 2,700 vs 2,750 in the source and 2,800 in spread.rs, a spread within the quoted 2.2 to 2.9 g/cm3; the arithmetic 2,700/3,300 = 0.818 and 3,300/(3,300-2,700) = 5.5 is the project's and is consistent; the source's root ratio about 5 corresponds to rho c / (rho m - rho c) = 2,750/550 = 5.0, a different quantity from the mod's 5.5, which is the crust removed per unit of surface lowering)
  Evidence level: secondary source (Wikipedia, citing Kearey et al. 2009)

- S23 (ISO-020, ISO-021, ISO-028): coded strength height h_s = S/(rho_c g) with S = 140 MPa, rho_c 2,800: 5.1 km at 9.81 m/s2, 13.4 km at 0.38 g, 2.5 km at 2 g; volcano ceiling 6.8 km. Source says: no source for 140 MPa or for "Tibet stands near 5 km" was found or read; Molnar & Lyon-Caen 1988, England & McKenzie 1982 and Clark & Royden 2000 are paywalled. Arithmetic check by me: 140e6 / (2800 x 9.81) = 5.10 km; 140e6 / (2800 x 0.38 x 9.81) = 13.4 km; 140e6 / (2800 x 2 x 9.81) = 2.55 km. The arithmetic is mine, not a source.
  Result: MATCH on the arithmetic (derived); COULD NOT READ for the 140 MPa value and for the existence of such a limit in the cited papers
  Evidence level: none for the value; derived for the arithmetic

- S24 (ISO-004): coded isostasy radius 50 km called "a flexural wavelength". Source says: Wikipedia "Flexural rigidity": effective elastic thickness "(~5-100 km)" without a citation; no flexural wavelength figure was found in a readable source.
  Result: COULD NOT READ (Watts 2001, the standard reference, is a book; no open source for continental flexural wavelengths was reached)
  Evidence level: secondary source (Wikipedia, elastic thickness only)

- S25 (ISO-018, ISO-019, ISO-022, ISO-032): coded diffusivity D = D0 min(x, X)^3 (cube of the excess over the strength height) after Clark & Royden 2000 and Royden et al. 1997; D0 range 10^2 to 10^5 km2/Myr; Earth's ceiling about 9 km. Source says: none read (Geology and Science paywalls, 403).
  Result: COULD NOT READ
  Evidence level: none

- S26 (ERO-097, ERO-098): coded erodibility rank by tensile strength^-2 (Sklar & Dietrich 2001), K spans orders of magnitude (Stock & Montgomery 1999), loose sediment ranked 4 to 10 by entrainment (Hjulstroem 1935). Source says for Hjulstroem (Wikipedia "Hjulstroem curve"): "The Hjulstrom curve shows that sand particles of a size around 0.1 mm require the lowest stream velocity to erode."; "For cohesive sediment, mostly clay but also silt, the erosion velocity increases with decreasing grain size, as the cohesive forces are relatively more important when the particles get smaller." (https://en.wikipedia.org/wiki/Hjulstr%C3%B6m_curve). Sklar & Dietrich and Stock & Montgomery not read.
  Result: Hjulstroem: PARTIAL (the qualitative ordering, fine sand easiest, cohesive clay harder, coarse gravel harder, is supported; the curve is about flow velocity for entrainment and the multipliers 4 to 10 on K are the project's own, so the paper does not support the numbers). Sklar & Dietrich and Stock & Montgomery: COULD NOT READ.
  Evidence level: secondary source (Wikipedia); none for the other two

- S27 (HYP-031): coded cost model about 1 microsecond per cell per step (556 s / 460 iterations / 1024^2 about 1.15 microseconds), derived from Tzathas et al. 2024 and FastFlow. Source says (Tzathas, Table 2): "Resolution Multigrid Fixed-Point Simulation Optim. ... 1024 8.18 s 70.89 s 556.42 s 4.10 s"; text: "the number of iterations increases in both the fixed point (43, 82, 156 iterations) and the simulation (230, 460, 980 iterations)" for the three resolutions; "We used an Intel Xeon E5-2650 v4 CPU with 64 GB RAM to compute all the results and timings" (https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf). 556.42 / 460 / 1024^2 = 1.15 microseconds (my arithmetic).
  Result: MATCH (the 556 s and the 460 iterations at 1024 are as quoted; the paper does not say whether the simulation timing is single-threaded, and a Python/numba code is not the mod's Rust solver, so "about 1 microsecond" is a transfer by assumption, as the note itself says)
  Evidence level: primary full text

- S28 (ERO-007, ERO-023, ERO-028): checks on whether porosity or compaction is coded (Athy 1930, Sclater & Christie 1980): the ledger states "no porosity or compaction is modelled in the surface stage" (ERO-028) and "no porosity or compaction" under simplifications; no ledger row cites Athy or Sclater & Christie.
  Result: COULD NOT READ (nothing to check; those two works are not cited; listed in Part 1 for completeness)
  Evidence level: none

## Rivers, lakes and the young landscape (HYD, YNG)

Results: MATCH 10, PARTIAL 7, MISMATCH 1, COULD NOT READ 4.

- HYD-001: coded width exponent 0.5 (width ∝ Q^0.5). Source says: "In the width-discharge graph, line AO-BO ha[s] a slope upward to the right equal to 0.5, or in the expression w=aQ^b, b=0.5, which is representative of the increase of width with discharge downstream." (OCR text layer of https://pubs.usgs.gov/pp/0252/report.pdf, text on Figure 19; the OCR renders the symbol b as "6" in places). The figure's legend reads "CHANGE DOWNSTREAM FOR DISCHARGE OF GIVEN FREQ[UENCY]".
  Result: MATCH. Nuance: this is the paper's representative downstream value for "discharge of given frequency" (not specifically mean annual discharge), taken from average conditions in the rivers studied. The at-a-station width exponent is 0.26.
  Evidence level: primary full text

- HYD-002: coded depth exponent 0.4. Source says: "In the depth-discharge graph line, A1-B1 has a slope f=0.4, and A1-C1 has a slope f=0.4. In the velocity-discharge graph, line m=0.1 for the downstream direction, and m=0.34 at a station." (same passage). The paragraph introducing the three equations gives station averages for 20 cross sections in the Great Plains and Southwest, "biased toward semiarid conditions": b=0.26, f=0.40, m=0.34 (OCR: "6-0.26 /=0.40 7n=0.34").
  Result: MATCH (downstream f = 0.4; the downstream velocity exponent is 0.1, so b+f+m = 1.0). The project does not use the velocity exponent.
  Evidence level: primary full text

- HYD-027: coded "Real rivers are commonly 1.1 to 1.5 and over". Source says (Leopold & Wolman 1957): "The reaches which have been called meanders are those in which the sinuosity, the ratio of thalweg length to valley length, is equal to or greater than 1.5. This value is an arbitrary one but in our experience where the sinuosity is 1.5 or greater, one would readily agree that the stream is a true meander. Many channels which appear to the eye to be very tortuous actually have rather low sinuosity ratings." and "straight channels, those with sinuosities less than 1.5, occur throughout the range of slopes" (OCR of https://pubs.usgs.gov/pp/0282b/report.pdf, section on Figure 46). Also "Valley Creek ... Its sinuosity (ratio of thalweg length to valley length) is practically 1.0."
  Result: PARTIAL. The conventional 1.5 threshold for a meandering reach is sourced (Leopold & Wolman 1957, stated as arbitrary). A source for "typical" sinuosity of real rivers as 1.1 to 1.5 was NOT found: Schumm 1963 (abstract) has pattern classes by sinuosity but no numbers in the abstract, and a secondary page (Wikipedia "Sinuosity", no source named) lists "1.05 <= SI < 1.25 winding, 1.25 <= SI < 1.50 twisty, 1.50 <= SI meandering". The mod's median 1.035 over 128 blocks should be compared with a threshold, not with a "typical" range. Note too that sinuosity depends on the measurement length (the audit used 128 blocks).
  Evidence level: primary full text (threshold); secondary source (Wikipedia, for the class bands)

- HYD-029: coded 60 mm/degC (source value 58.93). Source says: "the mean annual potential evapotranspiration in mm. at any site may be determined by multiplying the mean annual biotemperature by the factor 58.93." (Holdridge, Life Zone Ecology, scan OCR, https://app.ingemmet.gob.pe/biblioteca/pdf/Amb-56.pdf). Lugo et al. 1999: "The PETR was derived as (T bio * 58.93)/Pann (Holdridge, 1967)."
  Result: MATCH (58.93 is Holdridge's factor; the code rounds to 60, a 1.8 % difference). Note the scan's own text attributes the factor to Holdridge's 1959-1960 work ("found from his plant formation diagram that the same value ... could be determined ... by simply multiplying the mean annual bio-temperature in degrees C by the factor 58.93") and gives it again in the book; I could not confirm the edition year from the scan.
  Evidence level: primary full text (scan OCR); second primary-literature use in Lugo et al. 1999

- HYD-030: coded biotemperature "the year's mean of the temperature with time below freezing counted as 0 degC ... no upper cut-off is applied". Source says: "Recently, more detailed study in the outer tropics has indicated the desirability of eliminating high temperatures also in the calculation of the biotemperature. ... indicate that this should be close to 30 C. Thus, biotemperature is an average of the temperature between 0 and 30 C relative to the total period, which on the life zone chart is one year." and "The range of temperatures within which vegetative growth occurs is estimated to lie between 0 Celsius as a minimum and 30 C. as a maximum." (Holdridge, scan OCR). Lugo et al. 1999: "net primary productivity ceases at temperatures below 0 C and above 30 C".
  Result: MISMATCH. Holdridge 1967's biotemperature zeroes temperatures above 30 degC as well as below 0 degC; Climate::thaw applies only the lower limit. The 58.93 factor is defined against that two-sided biotemperature, so using it with a one-sided one overestimates PET wherever monthly means exceed 30 degC (hot deserts). For the sinusoid between warmest and coldest month the effect appears only above 30 degC.
  Evidence level: primary full text (scan OCR) plus Lugo et al. 1999

- HYD-014: coded "Priority-Flood, Algorithm 3 with the pit queue of Algorithm 4". Source says (author preprint, algorithm captions): "Algorithm 3 Priority-Flood+epsilon: This variation of the Improved Priority-Flood ensures that all filled depressions have surfaces with a height difference of at least epsilon ..."; its pseudocode begins "1: Let Open be a priority queue 2: Let Pit be a plain queue". "Algorithm 4 Priority-Flood+FlowDirs: This variation of Priority-Flood is based on the work of Metz et al. [2011] determines flow directions for all cells ... A plain queue cannot be used to speed this variation up because all the elevation information inside the depressions is needed." (https://richard.science/sci/2014_depressions.pdf, preprint; Algorithm 2 is "Improved Priority-Flood", which introduces the plain queue.) The ASCII glyph for epsilon was lost in extraction.
  Result: PARTIAL. Algorithm 3 (Priority-Flood+epsilon) does contain the pit queue itself, so "Algorithm 3 with the pit queue" is right; the words "of Algorithm 4" are wrong, since Algorithm 4 is the flow-direction variant that explicitly has no pit queue (the pit queue comes from Algorithm 2). Preprint numbering is assumed to equal the journal's; I could not check the published version.
  Evidence level: primary full text (author preprint)

- HYD-015: coded "mask = 2 x (steps to nearest outlet) + (flat's largest distance from higher ground - the cell's own); receiver is the same-height neighbour with the lowest mask". Source says: "The maximal height of the cell's flat ... is added to this value, thereby inverting the gradient away from higher terrain. Finally, the cell's value in FlatMask is always incremented by 2 I. This is the twice the gradient towards lower terrain and enforces unambiguous drainage from the flat without the iterative applications required by G&M" (https://richard.science/sci/2014_flats.pdf). Abstract: "Flow is modeled by superimposing a gradient away from higher terrain with a gradient towards lower terrain".
  Result: MATCH on the combination rule. (The receiver rule, lowest mask among same-height neighbours, I did not read.)
  Evidence level: primary full text (author preprint)

- HYD-049 / HYD-050: coded Fill-Spill-Merge's idea (research note speed claim "90-2600x faster than Jacobi iteration"). Source says: "In these case studies, the algorithm runs 90-2600 times faster (with a reduction in compute time of 2000-63 000 times) than the commonly used Jacobi iteration and produces a more accurate output." and "Runoff fills depressions, which then overflow and spill into their neighbors. If both a depression and its neighbor fill, they merge." (abstract, https://doi.org/10.5194/esurf-9-105-2021).
  Result: MATCH on the claim as quoted by the research note (and the mechanism the code takes the idea from). The row's code is an equilibrium solve, not the paper's volume bookkeeping, as the ledger says.
  Evidence level: primary abstract (Crossref record)

- HYD-034: coded Fu's Budyko curve, ET = P(1 + r - (1 + r^w)^(1/w)), w = 2.6. Source says: not read. The Zhang et al. 2004 abstract (OpenAlex) describes "a rational function approach" and a "dryness index", with no mention of Fu or omega; Fu 1981 was not found.
  Result: COULD NOT READ (Fu 1981 not found; Zhang 2004 paywalled, abstract does not show omega). The equation form is the standard one-parameter Fu curve but I will not call it verified.
  Evidence level: primary abstract (Zhang 2004), insufficient

- HYD-039: coded "Ohmura figure about -2 degC" (summer temperature at the equilibrium line for little accumulation). Source says (as returned by the summariser; my prompt had suggested the equation): "P = a + bT + cT2, whereby a = 645, b = 296 and c = 9, and P and T are in mm w.e. and degC, respectively." and "the characteristic climate at glacier equilibrium lines can be described using the summer 3 months' temperature in a free atmosphere, annual total precipitation, and the sum of global and long-wave net radiation" (abstract, Crossref). Solving 645 + 296 T + 9 T^2 = 100 mm gives T = -1.96 degC (my arithmetic from the quoted equation).
  Result: PARTIAL. The -2 degC follows from the equation at 100 mm of precipitation, so the project's recalled number is reproduced. But T in Ohmura is the free-atmosphere mean of the three summer months, not the surface warmest-month mean the degree-day rule uses, and the paper's standard error is 200 mm w.e. (summariser) so the comparison "-1 against -2" is within the scatter; the paper's data do not reach as low as 100 mm (ledger BIO/MOI notes lowest ELAs a few hundred mm).
  Evidence level: primary abstract; equation from a summariser (secondary until re-read)

- HYD-040: coded base 200 mm tuned so lakes average ~650 mm; "1,500 +- 150 km3 a year without the Caspian, about 650 mm over 2.3 million km2". Source says: "the annual VE of global lakes (excluding the Caspian Sea) from 1985 to 2018 is estimated to be 1500 +- 150 km3", "The average annual Ao for global lakes from 1985 to 2018 is 1.47 x 10^6 km2, which is about 63% of As (2.34 x 10^6 km2), indicating that a large portion of the global lake area is covered by ice." and (Methods) "HydroLAKES dataset, which contains 1,427,688 water bodies that are 0.1 km2 or larger. In this study, we included all of these water bodies except for the Caspian Sea." (Zhao et al. 2022, PDF text layer.)
  Result: PARTIAL. 1500 +- 150 km3 and the Caspian exclusion are in the paper. "650 mm" and "2.3 million km2" are not stated in it: they are the project's division 1,500 km3 / 2.34 million km2 = 641 mm per year per unit of total (ice-covered or not) lake area, and the 2.34 million km2 includes reservoirs (natural and artificial, 1.42 million water bodies of at least 0.1 km2). Per unit of ice-free open water (1.47 million km2) the same volume is 1,020 mm. Which of the two the "200 mm base + 60 mm per degC" should reproduce is therefore a modelling choice (the base is described as including sublimation, which fits the per-total-area figure). Also Zhao's total lake area is 1.57 % of global land area (same paper), not 1.8 %.
  Evidence level: primary full text

- HYD-043: coded "most of Earth's lake area is in cold climates at 6-8 degC of mean temperature above freezing". Source says: "In the region above 40N, lakes contribute 62% of global Ao, but generate 46% of global VE." and "The majority (83%) of the VE in this region occurs in the period from June to November" (Zhao et al. 2022). Arithmetic from those shares: 0.46 x 1,500 km3 over 0.62 x 1.47 million km2 = about 760 mm per unit of ice-free open-water area above 40N.
  Result: PARTIAL. The paper supports "most open-water lake area is at high latitude"; the "6-8 degC" figure has no source in it and I found none elsewhere.
  Evidence level: primary full text (shares); the 6-8 degC could not be found

- HYD-041 / lake evaporation by latitude: coded 200 + 60 x T+ mm a year (200 at 0 degC, 1,100 at 15, 2,000 at 30). Source says: no table of open-water evaporation rate by latitude is in the Zhao text I read (the only rate statement is "For lakes with Elake < 1500 mm year-1 ... For lakes with Elake > 1500 mm year-1 ..."). 
  Result: COULD NOT READ a rate-by-latitude source. Candidates not read: Wang et al. 2018 (closed), Zhao et al. 2022 supplementary figures.
  Evidence level: none

- HYD-044: coded "Polar lakes lose 150-350 mm a year". Source says: nothing found.
  Result: COULD NOT READ (no source found). The value decides the polar-lake case (ledger flag 14).
  Evidence level: none

- HYD-037: coded degree-day melt 4 mm per degC per day ("real snow melts 3 to 8"). Source says: nothing found.
  Result: COULD NOT READ (no source found; candidate Hock 2003, closed access, abstract unavailable).
  Evidence level: none

- HYD-045: coded ARID_INDEX 0.2 as "UNEP's arid and hyper-arid". Source says: UNEP classes in Zomer et al. 2022: "<0.03 Hyper Arid; 0.03-0.2 Arid; 0.2-0.5 Semi-Arid; 0.5-0.65 Dry sub-humid; >0.65 Humid", attributed to "UNEP. World atlas of desertification - Second Edition (United Nations Environment Program, 1997)". The Wikipedia article on the aridity index gives the 1992 first-edition limits as hyperarid below 0.05, arid 0.05-0.20, semi-arid 0.20-0.50, dry subhumid 0.50-0.65.
  Result: MATCH for the 0.2 boundary (arid or hyper-arid is below 0.2 in both editions). The index in UNEP is precipitation over reference evapotranspiration (FAO Penman-Monteith in Zomer et al.), while the code divides by a biotemperature-based PET.
  Evidence level: secondary source (Zomer et al. 2022 via PMC summariser; Wikipedia)

- HYD-046: coded polar when the warmest month is below 10 degC ("Koppen's E"). Source says: Table 1, "E Polar T hot < 10; T - Tundra T hot > 0; F - Frost T hot <= 0" (Peel et al. 2007; the <= glyph is lost in extraction but the F row is "T hot  0").
  Result: MATCH.
  Evidence level: primary full text

- HYD-062 / HYD-063 (Verpoorter): coded 3.7 % of non-glaciated land, 117 million lakes over 0.002 km2, 5.0 million km2. Source says: "GLOWABO contains geographic and morphometric information for approximately 117 million lakes with a combined surface area of about 5 x 10^6 km2, representing 3.7% of Earth's nonglaciated land area." (abstract; "all lakes greater than 0.002 km2").
  Result: MATCH.
  Evidence level: primary abstract

- HYD-063 (Messager): coded 1.8 % of land, 1.42 million lakes of 10 ha and more, 2.67 million km2. Source says: "HydroLAKES distinguishes 1.43 million individual polygons of natural lakes and human-made reservoirs with a surface area of at least 10 ha. Natural lakes cover a total of 2.67 x 10^6 km2" and "or 1.8% of global land area" (PMC summariser); abstract: "1.42 million natural lakes covering 2.67 million km2". It also says Verpoorter's higher estimates "may indicate a possible inflation of global lake estimates".
  Result: MATCH. (The 1.8 % denominator is "global land area"; Verpoorter's is non-glaciated land, so they are not on the same basis. Downing 2006: "304 million lakes; 4.2 million km2" of natural lakes, and "about 4.6 million km2 of the earth's continental 'land' surface (>3%)" covered by water including impoundments; abstract only.)
  Evidence level: primary full text via summariser (second source for the numbers: PMC abstract via EuropePMC); Downing primary abstract

- HYD-064: coded Caspian Sea 371,000-390,000 km2. Source says: Wikipedia "It covers a surface area of 371,000 km2 (excluding the highly saline lagoon of Garabogazkol to its east)", "Its gross coverage is 386,400 km2" and infobox "389,000 km2" (citing van der Leeden); a search snippet gave 371,000 (Guinness World Records) and 378,400 km2 (Encyclopaedia Iranica, estimated 1986). The summariser reported "377.0" in the table of Messager et al. 2016 but with unclear units.
  Result: PARTIAL. The coded range spans the published figures (371,000 excluding the Garabogazkol lagoon, about 386,000-389,000 including it); I found no primary scientific source. Note the water level changes the area by tens of thousands of km2 over decades.
  Evidence level: secondary source (Wikipedia, Encyclopaedia Iranica snippet)

- YNG-010: coded 500 m snowline drop (Porter 1989). Source says: "Mountain glaciers showed a snowline depression of approximately 500 meters." (abstract, OpenAlex reconstruction), for "average Quaternary glacial conditions".
  Result: MATCH for 500 m. The "about 900 m at the last glacial maximum" attributed to Porter 2001 could not be read (Porter 2001 is about tropical snowline depression; abstract unavailable).
  Evidence level: primary abstract

- YNG-015: coded detail noise "in the style of Schott et al. 2024". Source says: abstract: "amplifies low-resolution terrain inputs into detailed, hydrologically consistent outputs by employing multi-scale erosion techniques" with "fast approximations of thermal, stream power, and deposition erosion simulations performed at varying scales".
  Result: PARTIAL. The paper exists as cited and is about erosion-based amplification; the coded detail layer is plain value noise with no erosion, so "in the style of" applies to the amplifier (outside this ledger), not to the 12 % / 6-block noise term. The 0.12 coefficient is not from the paper.
  Evidence level: primary abstract

---

## The amplifier and glacial landforms (AMP, GLA)

Results: MATCH 11, PARTIAL 7, MISMATCH 1, COULD NOT READ 7.

#### The Ohmura relation (GLA-037, GLA-031, GLA-039; earth/AMP.md flag 1)

- GLA-037: coded P = 645 + 296 T + 9 T^2 (P in mm, T in degC), fitted at the ELAs of 70 glaciers. Source says: "The best-fit polynomial regression curve for the 70 glaciers under consideration is P = a + bT + cT2, whereby a = 645, b = 296 and c = 9, and Ρ and Τ are in mm w.e. and °C, respectively." and the abstract: "The relationships between temperature, precipitation and radiation on glacier equilibrium lines are investigated, using 70 glaciers for which the mass balance and meteorological observations have been carried out for sufficiently long periods." (Cambridge page, fetched twice; the second fetch returned the same equation sentence.) (https://www.cambridge.org/core/journals/journal-of-glaciology/article/climate-at-the-equilibrium-line-of-glaciers/8DCF1C24E32A6781CC365853535CD16F)
  Result: MATCH on the coefficients and the 70 glaciers. This clears the "from memory" admission for the polynomial itself.
  Evidence level: primary full text (page text via WebFetch)

- GLA-037 (definition of T, and standard error): coded T = summer temperature from "the mean of the three warmest consecutive months" in the model's climate. Source says: T is "the mean temperature of the summer months, June, July and August (December, January and February for the Southern Hemisphere) in the free atmosphere at the equivalent altitude as the ELA" and "The standard error of estimate is 200 mm w.e." (same page).
  Result: PARTIAL (the fit is for fixed JJA or DJF means of free-atmosphere air at the ELA's altitude, not the warmest three consecutive months; the coded solve is exact algebra of the cited polynomial; with a 200 mm w.e. standard error, the fit cannot support values near P = 0 where it gives -2.35 degC)
  Evidence level: primary full text (page text via WebFetch)

- GLA-031 / earth flag 1: the project's note that the fit "is extrapolated below its data (its lowest ELAs sit at a few hundred mm)". Source says: "The paper does not explicitly state the lowest and highest precipitation or temperature values across the 70 glaciers. Table 3 contains this data but is not fully readable in the provided text." (WebFetch reply for the same page.)
  Result: COULD NOT READ (the data table was not readable; the range of the 70 glaciers is therefore unconfirmed. The code's polynomial root, -2.35 degC at P = 0, follows from the published coefficients: 645 - 296 x 2.35 + 9 x 2.35^2 = about -1.)
  Evidence level: primary full text (page text, table missing)

#### The glacial erosion law and its constant (GLA-001 to GLA-003)

- GLA-001/003: coded E = K_g u_s^l with l = 1 and "E about 1e-4 * u_s under the surging Variegated Glacier". Source says (abstract, reconstructed from OpenAlex): "Sediment output was proportional to sliding, with an erosion rate of approximately 1.0 x 10^-4. Over a 20-year cycle, about 0.3 m of bedrock was eroded, with roughly two-thirds occurring during the 2-year peak surge period." (Humphrey and Raymond 1994.) Cook et al. 2020 (PMC7005307): "Values for K_G and l were first obtained empirically for Variegated Glacier, Alaska...When augmented with data from two other temperate glaciers, this indicated a linear relationship between erosion and sliding speed (i.e. l = 1)".
  Result: MATCH
  Evidence level: primary abstract (Humphrey and Raymond); primary full text (Cook et al., confirming)

- GLA-002 (the coded constant): coded K_g = 1e-5 with l = 1, with the comment that 1e-4 "overstates the long-term rate". Source says: Cook et al. 2020: "the dimensionless erosion rate factor, K_G, is 1 x 10-4 (when erosion rate is expressed in m a-1)", "the same as that indicated by most previous field estimates" and "is the value used most commonly in LEM studies"; their own fit is "l = 0.65 (or 0.69 including Meserve Glacier outlier)". Prasicek et al. 2018 (via PMC, Table 1 as relayed): linear law "K_g = 1 x 10-4 (dimensionless), with exponent l = 1, attributed to Humphrey and Raymond (1994)" and nonlinear law "K_g = 1 x 10-5 m-1 year, with exponent l = 2, cited to Herman et al. (2015)".
  Result: PARTIAL, and a likely pairing error. Published pairs are (1e-4, l = 1) and (1e-5 m^-1 yr, l = 2). The code pairs 1e-5 with l = 1, which is neither; it is a tenfold reduction of the l = 1 value, which the code states as a deliberate calibration but which no source gives. The units of the l = 2 constant differ, so 1e-5 with l = 1 should not be read as Herman et al.'s value.
  Evidence level: primary full text (Cook et al. 2020 and Prasicek et al. 2018, full text on PMC)

- GLA-002 (rates the constant was calibrated against): coded rates "0.01 mm/yr polar, 0.1 Norway, about 1 Swiss Alps, 10 to 100 south-east Alaska" attributed to Hallet et al. 1996. Source says: could not read Hallet et al. 1996. Cook et al. 2020: "The most rapid erosion is achieved at temperate glaciers with high mean annual precipitation" with erosion rates ">10 mm/year exclusively in Alaska (except Argentière Glacier)" (as relayed).
  Result: COULD NOT READ for the quoted list. The Alaska end is consistent with Cook et al., which is a different compilation.
  Evidence level: primary full text (Cook et al. 2020, on a different dataset); Hallet et al. not read

- GLA-003 (Harbor's l = 2): coded "Harbor used 2". Source says (Harbor 1992 abstract): "With an erosion law dependent on basal velocity, the model predicts rapid transformation of V-shaped cross section into a recognizably glacial form over time period on order of 10,000 yr eventual development, quasi-parabolic section." The exponent is not in the abstract. Cook et al. 2020 mention "empirically derived maximum estimates of 2.02" and "2.34-2.62" with citation numbers I did not resolve.
  Result: COULD NOT READ for l = 2 (the abstract confirms a velocity-dependent law and a quasi-parabolic result, which supports GLA-054 qualitatively)
  Evidence level: primary abstract

- GLA-010/011: coded Glen's law n = 3 and A = 2.4e-24 Pa^-3 s^-1 for temperate ice (Cuffey and Paterson 2010, "section 8"). Source says: Prasicek et al. 2018: "the deformation factor at the ELA where ice temperature is assumed to be just below the melting point", "2.4 x 10-24 Pa-3 s-1 (from Table 1), cited to Cuffey and Paterson (2010)" (as relayed).
  Result: MATCH on the value as quoted by a second paper. The book was not read, and the "section 8" location was not confirmed.
  Evidence level: secondary source (Prasicek et al. 2018)

- GLA-012: coded sliding law u_s = B tau_b^3 with B = 30 m/yr per (100 kPa)^3. Source says: Prasicek et al. 2018 give a sliding factor "1.7 x 10-19 Pa-3 m2 s-1 at the ELA" cited to Headley et al. (2012), in a different parameterisation; no source gives the coded B.
  Result: COULD NOT READ (no source for B; units differ and I did not convert)
  Evidence level: secondary source

- GLA-018: coded supercooling ratio 1.5 ("eroding beds settle about 1.5 times as steep as the ice surface"). Source says (Alley et al. 2003 abstract): "We find that the long profiles of beds of highly erosive glaciers tend towards steady-state angles opposed to and slightly more than 50 per cent steeper than the overlying ice-air surface slopes".
  Result: MATCH for 1.5 (the abstract says "slightly more than 50 per cent"). The "theory 1.2 to 1.7" attributed to Alley et al. 1998 is not in that paper's abstract: COULD NOT READ.
  Evidence level: primary abstract

#### Flux routing, glacier shape, ELA drop

- GLA-009: coded ice flux routed down the D8 drainage tree, cited to Hergarten 2021; goSPL cited for comparison. Source says: Hergarten's glacial law "E = K_g A_i^m S^n" (Eq. 19), "K_g is the glacial erodibility, A_i the catchment-size equivalent of ice flux, m, n exponents, S slope" (as relayed); goSPL ice.rst: "The ice margin H >= 0 is a free-boundary (obstacle) problem, on which an implicit F(H)=0 thickness solve diverged on real continental runs; and an ice-dynamics solve is stiff and over-thickens km-scale continental ice at coarse resolution".
  Result: MATCH for the goSPL "diverged on real continental runs" statement. PARTIAL for Hergarten: his erosion law is a stream-power law of ice flux and slope, not the sliding-speed law (E = K_g u_s^l) the code uses; the code takes from it the idea of routing flux down a drainage network.
  Evidence level: primary full text (Hergarten page and goSPL ice.rst, relayed)

- GLA-022: coded glacier width = H / 0.3 ("a design value of 0.3"); Hergarten cited for thickness proportional to width. Source says: "w ~ A^alpha, where alpha = eta xi ~ 0.30" (Eq. 12) and "w = A_i^0.3" (the parameterisation used in the experiments); "h ~ w" with "a thickness-to-width ratio of 0.05 in nondimensional coordinates" (as relayed).
  Result: PARTIAL, with a suspected mix-up. Hergarten's 0.3 is the exponent of width against flux or area, not a thickness-to-width ratio. The thickness-to-width ratio he uses is 0.05 in nondimensional units, and that is not comparable to the code's H/w = 0.3. I do not know whether the code's 0.3 came from there; the code calls it a design value. Aletsch (Wikipedia): "a maximum of 905 metres" thick at Konkordiaplatz and "a width of approximately 1.5 km" south of it, giving 0.6, as the earth file says (the thickness maximum and the 1.5 km width are at different places).
  Evidence level: primary full text (Hergarten, relayed); secondary (Wikipedia Aletsch)

- GLA-030: coded ELA drop 500 m (Porter 1989: "snowlines some 500 m lower than today on Quaternary average"); about 900 m at the last glacial maximum (Porter 2001). Source says (Porter 1989 abstract): "mountain glacier snowlines depressed 'some 500 m.'" under average Quaternary conditions. Porter 2001 abstract was not available.
  Result: MATCH for 500 m (Porter 1989). COULD NOT READ for 900 m; and the paper in question is about the tropics, so using it as a global LGM figure is untested.
  Evidence level: primary abstract (Porter 1989)

- GLA-057: coded confined ice only; areal scouring under the sheet (Sugden 1978). Source says (abstract): "Landscapes of areal scouring are found to be associated with zones of basal melting and occur beneath much of ice-sheet centre"; "Selective linear erosion are common on uplands near eastern periphery".
  Result: MATCH
  Evidence level: primary abstract

- GLA-054/055: coded parabola exponent b about 2, Svensson fitted b = 2.0 to 2.2. Source says: abstract not available (OpenAlex null); Harbor's abstract gives "quasi-parabolic section".
  Result: COULD NOT READ for Svensson's b = 2.0-2.2
  Evidence level: none (primary abstract of Harbor, qualitative only)

#### The buzzsaw relief (GLA-062, GLA-064; flag 7)

- GLA-062: coded relief above cirque floors "rarely over 600 m" (Mitchell and Humphries 2015); coded `max_ridge_raise_m` 150. Source says (abstract, confirmed by the word positions in OpenAlex's index): "In addition, cirque relief above floors averages 346 +- 107 m, and is rarely >600 m. In regions where ice-free cirques are abundant, cirque relief is independent of modern precipitation rates, tectonic regime, and relative peak height, limiting peaks within a fixed distance above QA-ELA" and "glacial cirque erosion excavates a zone centered on QA-ELA, while climate-driven weathering processes on adjacent slopes limit peaks to a few hundred meters above this zone." (https://doi.org/10.1130/g36180.1 via https://api.openalex.org/works/doi:10.1130/g36180.1)
  Result: MATCH for the 600 m quote. The mean is 346 +- 107 m, well above the coded 150 m for ridge raising, but the 150 m is a different quantity (a raise cap, not a cirque relief). The paper's relief is cirque headwall relief above cirque floors, not the buzzsaw's cut of peaks above the ELA.
  Evidence level: primary abstract

- GLA-064: coded buzzsaw cuts mean height towards the ELA (Egholm et al. 2009). Source says (abstract): "we further demonstrate how a combination of erosional destruction of topography above the snowline by glacier-sliding and commensurate isostatic landscape uplift caused by erosional unloading can explain observations of maximum mountain height by driving elevations towards an altitude window just below the snowline".
  Result: MATCH
  Evidence level: primary abstract

#### The amplifier (Schott 2024 shaders)

- AMP-034 (earth/AMP.md flag 4): coded `thermal_slope` 0.6; comment "Schott: tan 30 deg = 0.57". Source says (thermal.glsl, fetched twice with the same result): `uniform float tanThresholdAngle = 0.57f; uniform bool noisifiedAngle = true; uniform float noise_min = 0.9f; uniform float noise_max = 1.4f; uniform float noiseWavelength = 0.0023f;` and, in main(): `float tanAngle = tanThresholdAngle; if (noisifiedAngle) { vec3 p = Point(ivec2(x, y)); float t = snoise(p * noiseWavelength) * 0.5f + 0.5f; tanAngle = mix(noise_min, noise_max, t); }`. (https://raw.githubusercontent.com/H-Schott/MultiScaleErosion/main/data/shaders/thermal.glsl)
  Result: PARTIAL. 0.57 is the shader's `tanThresholdAngle`, the value used only when `noisifiedAngle` is false. With `noisifiedAngle` true (the shader's default), the 0.57 is not used at all and the slope threshold is a noise value between 0.9 and 1.4 (about 42 to 54 degrees). The comment "tan 30 deg = 0.57" is therefore the non-default branch; the default Schott threshold is a much steeper 0.9-1.4. Whether the paper's figures used the shader's defaults I could not read, since the C++ wrapper showed no angle lines and the UI code was not fetched. The coded 0.6 sits near the non-default 0.57 and far below the noisified default.
  Evidence level: primary full text (shader source, fetched twice)

- AMP-038: coded critical slope varied by plus or minus 25 percent ("Schott: noisifiedAngle x0.9 to 1.4"). Source says: as above, `tanAngle = mix(noise_min, noise_max, t)` with noise_min 0.9 and noise_max 1.4.
  Result: MISMATCH of interpretation. In the shader, 0.9 and 1.4 are the tangent values themselves (replacing 0.57), not multipliers of a base value. As a multiplier of 0.57 they would give 0.51 to 0.80, which is not what the shader does.
  Evidence level: primary full text (shader source)

- AMP-006: coded flow power 2.0; "Schott: 1.3". Source says (erosion.glsl): `uniform float flow_p = 1.3f;`
  Result: MATCH for the cited Schott value (the coded 2.0 is the project's tuned choice; stale 1.5 in kernels.rs remains an internal inconsistency)
  Evidence level: primary full text (shader source)

- AMP-028: coded slope cap 0.5; "Schott caps the slope at 1 (min(S,1))". Source says (erosion.glsl): `uniform float p_sa = 0.8f; uniform float p_sl = 2.f;` and `spe = pow(stream, p_sa) * clamp(pow(steepest_slope, p_sl), 0., 1.);` with `k = 0.0005f`.
  Result: PARTIAL. The cap is on the slope term S^2 (clamped to 1), which is equivalent to capping S at 1; so "min(S,1)" is right for S but the shader clamps the power S^2 not S. The exponents p_sa = 0.8 and p_sl = 2 in the code comments match.
  Evidence level: primary full text (shader source)

- AMP-016: coded flood epsilon 0.01 m (Barnes, Lehman and Mulla 2014, "Priority-Flood+epsilon" for the method). Source says (abstract): "determine flow directions through either incremental elevation changes or depression carving. In the case of incremental elevation changes, the algorithm includes safety checks not present in prior works."
  Result: PARTIAL (the method is in the paper; the coded 0.01 m is the project's choice; the paper's own epsilon name could not be read)
  Evidence level: primary abstract

- AMP-034 (Tzathas): the note "critical slope atan 30 deg (0.57)" attributed to Tzathas 2024. Source says: could not read (Inria PDF returned binary; Wiley and EG library 403).
  Result: COULD NOT READ
  Evidence level: none (search snippet for the title only)

- AMP-001 (reference check): the claim that the code follows Schott's workflow "erosion, thermal, deposition with x2 upsampling". Source says (README): "The suggested sequence is: 'erosion -> thermal -> deposition' with alternating x2 upsampling for optimal amplification results".
  Result: MATCH
  Evidence level: primary full text (GitHub README, relayed)

#### Earth reference figures with no source in the repo

- Sinuosity ("Real rivers are commonly 1.1-1.5 and over", audit finding 2, marked from memory). Source says: sinuosity classes "SI <1.05: almost straight; 1.05 <= SI <1.25: winding; 1.25 <= SI <1.50: twisty; 1.50 <= SI: meandering" (Mueller 1968, Annals of the Association of American Geographers 58(2): 371-385, via Wikipedia Sinuosity).
  Result: PARTIAL (the classes support "1.1-1.5 and over" for winding, twisty and meandering rivers, but not that rivers are commonly so; the amplifier's median of 1.035 falls in "almost straight")
  Evidence level: secondary source (Wikipedia, citing Mueller 1968)

- East Antarctica "ice at 20 to 50 mm/yr water equivalent", the Earth summit "5 km + 3-4 km", the Hallet rates, Oerlemans' beta = 0.005, Porter 2001's 900 m, Svensson's b: not found or not looked up this pass (see the results above for the ones tried).
  Result: COULD NOT READ
  Evidence level: none

## Rock, strata and structure (RCK, STR, DEF)

Results: MATCH 8, PARTIAL 15, MISMATCH 2, COULD NOT READ 8.

Format: ledger id, coded value, what the source says, result, evidence level.

- RCK-039/RCK-040 (slate 250 degC; greenschist-facies rocks 300 degC): coded 250 and 300 degC. Source says: "Low-grade metamorphism takes place at temperatures between about 200 to 320oC, and relatively low pressure. High-grade metamorphism takes place at temperatures greater than 320oC" (Nelson, Tulane EENS 212, https://www2.tulane.edu/~sanelson/eens212/metaclassification&facies.htm); and for greenschist facies "typically 300-450 °C ... and 2-10 kilobars" (Wikipedia "Greenschist", citing Encyclopaedia Britannica; https://en.wikipedia.org/wiki/Greenschist).
  Result: PARTIAL. Slate at 250 sits inside the 200-320 degC low-grade band; greenschist onset at 300 is at the cold edge of the 300-450 range. Winter ch. 21/25 themselves were not readable, so the "textbook boundary" is not confirmed in the cited book. No pressure or depth is coded, and the facies pressure ranges are 2-10 kbar for greenschist.
  Evidence level: secondary source (Tulane course notes; Wikipedia citing Britannica)
- RCK-041/RCK-042 (phyllite 350; schist 450, "garnet zone"): coded 350 and 450 degC. Source says: no source fetched states phyllite or schist formation temperatures; the Tulane page defines slate, phyllite, schist by foliation only ("If a rock has a slatey cleavage as its foliation, it is termed a slate, if it has a phyllitic foliation, it is termed a phyllite, ...") and states no temperatures for them.
  Result: COULD NOT READ (no figure for phyllite or schist found; rock names are textural, not temperature-defined, so any single temperature is a game rule)
  Evidence level: secondary source (Tulane notes)
- RCK-043 (amphibolite 500 degC): coded 500 degC. Source says: "Amphibolite facies conditions are experienced at temperatures in excess of 500 °C and pressures less than 1.2 GPa" (https://en.wikipedia.org/wiki/Amphibolite, no reference cited in the article).
  Result: MATCH (500 degC is the lower edge; the Wikipedia statement carries no citation)
  Evidence level: secondary source (Wikipedia, uncited statement)
- RCK-044 (gneiss 650 degC, upper amphibolite, sillimanite and migmatite): coded 650 degC. Source says: "The granulite facies is determined by the lower temperature boundary of 700 ± 50 °C" and "pressure range of 2-15 kb" (https://en.wikipedia.org/wiki/Granulite).
  Result: PARTIAL. 650 degC is within the 700 ± 50 lower boundary of granulite facies, at its low end; it is the upper-amphibolite end, so consistent. Gneiss is a texture, not a facies, and forms across amphibolite to granulite facies. The migmatite claim was not checked.
  Evidence level: secondary source (Wikipedia)
- RCK-045 (gypsum to anhydrite 50 degC, Warren 2016 ch. 7): coded 50 degC. Source says: not fetched (book, no online text).
  Result: COULD NOT READ (book not online)
  Evidence level: none
- RCK-028/RCK-030 (geothermal gradient; gradient cap 120/2.5 = 48 degC/km): coded T = T_s + q z / k with k = 2.5 (m K), heat flow cap 120 mW/m2, i.e. a maximum of 48 degC/km (derived by the code). Source says: "temperature rises with depth at a rate of about 25-30 °C/km ... near the surface in the continental crust" (https://en.wikipedia.org/wiki/Geothermal_gradient, citing IPCC 2008, Fridleifsson et al.); mean continental heat flow "65 ... mW m-2" (Pollack, Hurter & Johnson 1993 abstract, via OpenAlex https://api.openalex.org/works/doi:10.1029/93RG01249); "A geothermal gradient of around 30o/km, expected in areas undergoing an orogenic event, would produce a succession of facies from zeolite to prehnite pumpellyite to greenschist to amphibolite to granulite"; "A low geothermal gradient of around 10o/km ... zeolite to blueschist to eclogite" (Nelson, Tulane); granulites form "at high thermal gradients of greater than 30 °C/km" (Wikipedia "Granulite").
  Result: PARTIAL. The mean continental heat flow of 65 mW/m2 over k = 2.5 gives 26 degC/km, which agrees with 25-30 degC/km. The 48 degC/km ceiling is above the quoted orogenic 30 degC/km but not contradicted. The value k = 2.5 (m K) and the 120 mW/m2 cap were not found in any source fetched.
  Evidence level: secondary source (Wikipedia, Tulane notes) plus primary abstract (Pollack 1993)
- RCK-061 (pluton thickness T = 0.6 L^0.6 km): coded T = 0.6 x L^0.6 with L in km (the median 25 km gives T about 4.1 km; the maximum 80 km gives about 8.3 km, derived). Source says (Cruden 1998 abstract): "Granites in crystalline terranes and continental magmatic arcs tend to be circular to elliptical in map view, varying in width from about 3 to 100 km"; "large (10-100 km wide), thin (≤3 km thick) granites can be emplaced quickly (100 to 1 Ma)" (https://api.openalex.org/works/doi:10.1144/gsjgs.155.5.0853).
  Result: MISMATCH against the abstract's stated thickness. Cruden's abstract says granites 10-100 km wide are at most 3 km thick; the code gives about 4.1 km at 25 km and about 8.3 km at 80 km. The abstract contains no equation, so the 0.6 L^0.6 form could not be found in the cited paper at all. (The full text was not read.)
  Evidence level: primary abstract
- RCK-060 (Tuolumne about 1,100 km2, 45 x 25 km, median pluton L 25 km): coded as stated. Source says: the Bateman & Chappell abstract describes a "concentrically zoned plutonic sequence ... progressively younger and more felsic inward"; no area or size is given (OpenAlex summary); Cruden's abstract gives pluton widths of "about 3 to 100 km".
  Result: COULD NOT READ for the Tuolumne area and dimensions; the 8-80 km range is consistent with Cruden's 3-100 km widths.
  Evidence level: primary abstract (no size in it)
- RCK-056 (Coastal Batholith: about 1,000 plutons, 1,600 x 60 km, about 60 Myr, rate 1e-4 per km2 per Myr): coded as stated. Source says: "The batholith runs a length of ca. 1600 km" and "a group of hundreds, if not thousands, of individual plutons" (https://en.wikipedia.org/wiki/Coastal_Batholith_of_Peru, citing Haederle & Atherton 2002); Mukasa 1986 reports zircon ages that "range from 188 to 37 m.y." for the sampled plutons (abstract, https://api.openalex.org/works?search=...; the earlier search result also gave 105-62 Ma for the Cretaceous main phase, a search snippet); a search snippet described it as "2000 km long ... approximately 1000 plutons" (not a source I could name).
  Result: PARTIAL. The 1,600 km length and a pluton count of the order of a thousand are supported; the 60 km width and the 60 Myr duration are not found (the age span in the abstracts is 188-37 Ma overall, or about 105-62 Ma for the main Cretaceous phase, so 60 Myr is plausible but not confirmed). Arithmetic from the code's own numbers: 1000 / (1600 x 60 x 60) = 1.7e-4, not 1e-4.
  Evidence level: secondary source (Wikipedia) and primary abstract (Mukasa 1986), plus a search snippet
- RCK-063 (pluton roof depth 2-6 km, Buddington 1959): coded 2 to 6 km. Source says: not found; the Buddington record has no abstract.
  Result: COULD NOT READ (no abstract; paywalled)
  Evidence level: none
- RCK-006 (river incision scales with tensile strength to the power -2): coded "river incision scales roughly with tensile strength^-2". Source says: "Experiments also reveal that rock resistance to fluvial erosion scales with the 'square of tensile strength.'" (Sklar & Dietrich 2001 abstract, https://api.openalex.org/works/doi:10.1130/0091-7613(2001)029<1087:SARSCO>2.0.CO;2).
  Result: MATCH
  Evidence level: primary abstract
- RCK-007 (bedrock erodibility span about 0.3-4, i.e. a compressed contrast): coded bedrock multipliers 0.3 (quartzite) to 4.0 (halite), granite 1.0, mudstone 3.0. Source says: "rock erodibility varies substantially, from granite and metamorphic rocks at 10^-7 to 10^-6 m^0.2 yr^-1 to volcaniclastic rocks and mudstones at 10^-5 to 10^-2 m^0.2 yr^-1" (Stock & Montgomery 1999 abstract, https://api.openalex.org/works/doi:10.1029/98JB02139).
  Result: PARTIAL. The paper does support contrasts of several orders of magnitude (mudstone/volcaniclastic K 10 to 100,000 times granite K). The code deliberately compresses these to a factor of 3 between granite and mudstone, so the coded values are a game design choice, not what the paper measures. The ordering (crystalline low, mudstone high) agrees.
  Evidence level: primary abstract
- RCK-008 (loose-class erodibility 4-10, ranked from Hjulström 1935: "silt and fine sand move at the lowest flow speeds, cohesive clay and coarse gravel need more"): coded laterite 4.0 up to loess and volcanic ash 10.0; clay 5.0, sand 9.0, mud 7.0. Source says: the curve shows "critical erosion velocity in cm/s as a function of particle size in mm"; for cohesive sediments "the erosion velocity increases with decreasing [grain size]" because cohesive forces dominate at small sizes; the article "contains no discussion of bedrock erosion" (https://en.wikipedia.org/wiki/Hjulstr%C3%B6m_curve, which quotes Hjulström's citation).
  Result: PARTIAL. Hjulström supports a qualitative ranking of loose grains (clay and coarse gravel harder to entrain than sand, which agrees with clay 5.0 and gravel 5.0 below sand 9.0), but the curve is a threshold velocity for grains, not a bedrock-incision coefficient, and it does not give multipliers 4-10. The code's wording that "silt and fine sand" move at the lowest speeds is not what the curve says: the minimum is for sand-sized grains and silt needs more because of cohesion (per the Wikipedia paraphrase; the 1935 text was not read). So mud 7.0 against sand 9.0 is in the right direction, but silt-sized loess at 10.0 (equal to ash) is not supported.
  Evidence level: secondary source (Wikipedia)
- RCK-010 (hillslope creep coefficient D multipliers; Roering 1999, Heimsath 1997): coded D multipliers 0.5-5. Source says: Roering et al. demonstrate "a diffusive transport law depending nonlinearly on hillslope gradient" which "approximates linear diffusion at low gradients" (abstract, https://api.openalex.org/works?search=Evidence for nonlinear ... Roering); Heimsath 1997 abstract not available (metadata only).
  Result: PARTIAL. The papers establish a hillslope diffusivity and a depth-dependent soil production function; neither gives lithology-dependent multipliers. The coded multipliers are judgment, as the code says.
  Evidence level: primary abstract (Roering); metadata only (Heimsath)
- RCK-001 (rock densities, 20 classes, from Telford/Schon): coded granite 2650, basalt 2900, etc. Source says: "The average density of granite is between 2.65 and 2.75 g/cm3" (https://en.wikipedia.org/wiki/Granite, citing "Rock Types and Specific Gravities", EduMine) and "The average density of basalt is 2.9 g/cm3 ... granite's typical density of 2.7 g/cm3" (https://en.wikipedia.org/wiki/Basalt, citing Philpotts & Ague 2009 p. 22).
  Result: PARTIAL. Granite 2650 and basalt 2900 agree with these two secondary values. The Telford and Schon tables could not be read (books; the archive.org lending copy returned 401), and Wikipedia gave no density for gabbro, peridotite, eclogite, sandstone or gneiss, so the other 18 values are unchecked. 41 of 62 densities have no source in the code.
  Evidence level: secondary source (Wikipedia citing EduMine and Philpotts & Ague)
- RCK-015 (carbonate compensation depth 4500 m): coded CCD_M = 4500 m. Source says: Pacific "About 4200-4500 m, except beneath the equatorial upwelling zone, where the CCD is about 5000 m"; Atlantic "approximately 5000 m"; Indian Ocean "approximately 4300 m" (https://en.wikipedia.org/wiki/Carbonate_compensation_depth, uncited in the article).
  Result: PARTIAL. 4500 m is within the Pacific range and below the Atlantic (5000 m) and Indian (4300 m) figures; a single global value of 4.5 km is the middle of the 4.2-5.0 km range.
  Evidence level: secondary source (Wikipedia)
- RCK-022 (loose sediment turns to rock below 2.2 MPa overburden, about 150 m): coded 150 m, 20 Myr. Source says: the tectonics.js repository front page describes "a 3d planet simulator in your web browser" and does not mention lithification or overburden (https://github.com/davidson16807/tectonics.js). Arithmetic on the code's own numbers: 2.2 MPa / (150 m x 9.81 m/s2) = 1,495 kg/m3, i.e. a bulk density near that of water-saturated loose sediment.
  Result: COULD NOT READ (the value would be in the source code, not read)
  Evidence level: none
- RCK-069/RCK-071 (magma chamber depths; crater lava lakes "about eight of some 1,400 active volcanoes"): coded chamber depth 3 km; lava lake share 0.25. Source says: Wikipedia "Lava lake" lists eight examples of volcanoes with persistent or near-persistent lava lakes (Erebus, Erta Ale, Kilauea, Masaya, Mount Michael, Nyiragongo, Yasur, Tofua) and gives no count of active volcanoes (https://en.wikipedia.org/wiki/Lava_lake). Chamber depths for Kilauea, St Helens, Vesuvius were not checked.
  Result: PARTIAL (the "eight" agrees; the 1,400 and the chamber depths were not found)
  Evidence level: secondary source (Wikipedia)
- DEF-001 (fold wavelength 16 km median, "15.8 ± 5.3 km", Mouthereau 2007): coded median 16 km, range 8-32 km. Source says: "Homogeneous fold wavelengths (15.8 ± 5.3 km), tectono-sedimentary evidence for simultaneous fold growth in past 5.5 ± 2.5 Ma, drainage network organization, and homogeneous peak differential stresses (40 ± 15 MPa) together point to buckling as the dominant process responsible for cover folding" (Mouthereau et al. 2007 abstract, https://api.openalex.org/works/doi:10.1029/2007TC002098).
  Result: MATCH (the quoted phrase is in the abstract verbatim; the 8-32 km clamp is a design value, as the code says)
  Evidence level: primary abstract
- DEF-005 (fold shortening, Zagros 16-19 %, and about 20 %): coded fold_shortening [0.08, 0.30]. Source says: "A shortening 65-78 km (16-19%) is estimated across the Zagros" (Mouthereau et al. 2007 abstract, same URL); Alavi 2007: "Shortening estimates vary regionally, with minimum values ranging from 16-30 percent across different sectors" (abstract, https://api.openalex.org/works/doi:10.2475/09.2007.02). McQuarrie 2004 "about 20 %": not read.
  Result: MATCH for Mouthereau's 16-19 %; PARTIAL for the coded range (Alavi's 16-30 % supports an upper value of 0.30; the lower 0.08 for forelands is a design value); McQuarrie's figure COULD NOT READ.
  Evidence level: primary abstract
- DEF-002 (buckling wavelength follows stiff-layer thickness; Biot 1961, tested by Sherwin & Chapple 1968): Source says: Sherwin & Chapple evaluated Biot's theory "by examining 818 fold wavelengths from six geological sites containing quartz veins in slate and phyllite", finding "viscosity ratio values ... too low for the Biot theory ... to apply directly" (abstract summary, https://api.openalex.org/works/doi:10.2475/ajs.266.3.167). Biot 1961 abstract not available.
  Result: PARTIAL. "Tested on natural folds" is right; the test found the simple theory did not fit directly. The code does not use the relation anyway.
  Evidence level: primary abstract (summary), metadata only for Biot
- DEF-004 (fold train length 5-12 times wavelength, Blanc et al. 2003): coded [5.0, 12.0]. Source says: the abstract reports a balanced cross-section with about 49 km shortening over 5 Myr (about 10 mm/yr) and several decollement horizons; no fold length-to-wavelength ratio (https://api.openalex.org/works/doi:10.1144/0016-764902-110).
  Result: COULD NOT READ (abstract has no ratio; full text paywalled)
  Evidence level: primary abstract
- DEF-013 (fault trace length, linkage of segments of some tens of km, Gawthorpe & Leeder 2000): Source says: not found; OpenAlex had the record but no abstract.
  Result: COULD NOT READ
  Evidence level: none
- DEF-015 (D = gamma L with gamma about 0.03 for normal faults): coded as the cited gamma 0.03. Source says: "The best fit curve through the compiled data is arithmetically defined by D = 0.03L^1.06" (Lathrop et al. 2022 on Schlische et al. 1996, https://www.frontiersin.org/journals/earth-science/articles/10.3389/feart.2022.907543/full); Schlische abstract: displacement "scales linearly with fault length", with "no significant change in the geometric and scaling properties between very small and substantially larger faults" (abstract summary, https://api.openalex.org/works/doi:10.1130/0091-7613(1996)024<0683:GASROA>2.3.CO;2); Cowie & Scholz abstract: "a linear d-L relationship is most supported", with a "possible cross-over phenomenon between small and large faults" unresolved (https://api.openalex.org/works/doi:10.1016/0191-8141(92)90066-6); Lathrop et al. on the spread: "Reported values of c range from 0.0001 to 1" and "although they typically fall between 0.001 and 0.1".
  Result: PARTIAL. 0.03 is Schlische's coefficient as reported by a review (the exponent is 1.06, not exactly 1); the exact 0.03 is not in the abstract itself. The "order of magnitude" spread is confirmed by the review, not by Kim & Sanderson, whose paper was not read.
  Evidence level: secondary source (Lathrop et al. 2022, open access) plus primary abstracts
- DEF-016/DEF-017/DEF-018 (slip ratios 0.05 normal, 0.08 x 0.5-1.0 thrust, 0.05 strike-slip): coded 0.05, 0.08 (effective 0.04-0.08), 0.05. Source says: as above, gamma of the source is 0.03 for normal faults; typical gamma 0.001-0.1.
  Result: MISMATCH against the cited value (coded 0.05 is 1.7 times 0.03; 0.08 is 2.7 times; 0.05 is 1.7 times), yet all lie inside the 0.001-0.1 range the review gives as typical. The code labels them design values, and the cited 0.03 is specific to normal faults, so no source was found for the thrust or strike-slip coefficients.
  Evidence level: secondary source (Lathrop et al. 2022)
- DEF-019/DEF-020 (normal faults dip about 60 degrees, thrusts about 30 degrees, Anderson 1951): coded normal 55-70, thrust 20-40 degrees. Source says: "a normal fault would form close to 60°, a reverse fault close to 30°, and strike-slip fault close to 90° degrees using the basic Andersonian Fault Theory" (GeoExpro, https://geoexpro.com/whos-andy-and-what-did-he-know-about-faults/); "Gravity regimes are predicted to have a 60° dip, while thrust regimes have a dip angle of 30°" (https://en.wikipedia.org/wiki/Anderson%27s_theory_of_faulting, which cites no 1951 work); a search snippet gives 60-65 degrees for friction coefficients of 0.6-0.85.
  Result: MATCH on the numbers as Andersonian theory predictions; the book itself was not read, and the 60/30 figures are consequences of Coulomb friction of about 0.6, not statements made as such in the book. The ranges 55-70 and 20-40 enclose them. The dips are applied in a world drawn 15 times taller than wide (ledger flag 7), so the shown dip is not the real dip.
  Evidence level: secondary source (GeoExpro, Wikipedia)
- STR-030/DEF-027 (Zagros Simple Folded Belt about 250 km wide; Subandean 100-200 km; code 300 km and 250 km): coded fold_belt_km 300 (STR-030), foreland_km 250 (DEF-027). Source says: "the approximately 200-300 km wide deformation zone" (Alavi 2007 abstract, https://api.openalex.org/works/doi:10.2475/09.2007.02). Subandean width: not found.
  Result: PARTIAL. 250 km is the middle of 200-300 km; 300 km (STR-030) is the upper end, not the quoted 250. The Subandean 100-200 km was not found.
  Evidence level: primary abstract
- DEF-042 (a stock crops out over less than 100 km2, radius under 5.6 km; coded radius 2-10 km): coded radius [4.0 median, 2.0, 10.0] km; long half-axis x 1 to 1.6. Source says: "a stock is an igneous intrusion that has a surface exposure of less than 100 square kilometres" (https://en.wikipedia.org/wiki/Stock_(geology), citing Gabler et al. 2009 Physical Geography and Whittow 1984 Dictionary of Physical Geography); a batholith is "larger than 100 km2" (https://en.wikipedia.org/wiki/Batholith). Arithmetic: a circle of radius 5.6 km is 98.5 km2; a radius of 10 km is 314 km2, and an ellipse of 10 x 16 km is 503 km2.
  Result: MATCH for the definition (100 km2, radius 5.6 km); MISMATCH of the coded range against its own definition (radii above 5.6 km give stocks of 100-500 km2, which are batholiths by that definition), as the earth-file flag 1 stated.
  Evidence level: secondary source (Wikipedia) plus derived arithmetic
- DEF-059 (rift flanks rise by flexure when the hanging wall unloads; Weissel & Karner 1989; coded footwall_share 0.2): Source says: "the uplift of rift flanks results from mechanical unloading" and isostatic rebound with finite flexural strength (abstract summary, https://api.openalex.org/works/doi:10.1029/JB094iB10p13919).
  Result: PARTIAL (the mechanism is confirmed; 0.2 is a design value and is not in the abstract)
  Evidence level: primary abstract (summary)
- STR-026/STR-028 (salt rises once overburden dense enough; Niger delta growth faults): Source says: Hudec & Jackson 2007 is a review of salt tectonics (no abstract in the record); Doust & Omatsola 1990 not found.
  Result: COULD NOT READ
  Evidence level: none
- DEF-063 (Noddy port licence: "MIT License, Copyright (c) 2021 Loop3D; code is Mark Jessell's"): coded licence MIT; the mod ports it under LGPL. Source says: Loop3D/noddyverse LICENSE title line "MIT License", second line "Copyright (c) 2021 Loop3D" (fetched from raw.githubusercontent.com at the pinned commit 6a21b23...); Jessell et al. 2022: "All codes and data are released under the MIT License." and "The modified Noddy software source code (in C language) is made available through a GitHub repository with DOI 10.5281/zenodo.4589883" (https://essd.copernicus.org/articles/14/381/2022/); noddy_qt: "the project is distributed under the MIT License" and "acknowledges Noddy's original development at Imperial College, University of London (1981-2), subsequent updates at Monash University (first released 1994), and latest changes at The University of Western Australia" (https://github.com/Loop3D/noddy_qt). But other releases of the same program differ: the SourceForge project "Noddy" (developer mjessell, registered 17 Nov 2009, last update 23 Jul 2016) lists "License: GNU General Public License version 3.0 (GPLv3)" (https://sourceforge.net/projects/noddy/); pynoddy's repository is under GPL-2.0 and its README says "Mark Jessell is currently cleaning the Noddy core library in order to adjust our code to a more permissive license" (https://github.com/flohorovicic/pynoddy); and tectonique.net says "As of May 2009 this software is no longer subject to a commercial release, and hence is freely distributable", with three conditions (do not misrepresent the origin; mark altered versions plainly; do not remove this license notice) (https://tectonique.net/noddy/).
  Result: MATCH for the repository the code names (MIT, Copyright (c) 2021 Loop3D). PARTIAL for the wider question: the original Noddy has been distributed under at least three licences (a zlib-style free-distribution notice, GPLv3 on SourceForge, GPL-2.0 in pynoddy) before the 2021-22 MIT release by the author's group, and the author states the MIT release in a peer-reviewed paper. An MIT-licensed source can be ported into an LGPL work provided the MIT notice and copyright are kept, and the "Copyright (c) Ewen Christopher and Monash University 1994" matrix.c notice was not seen in anything fetched. The MIT claim rests on the noddyverse snapshot only; the GPL releases are older.
  Evidence level: primary full text (LICENSE file, ESSD article page) plus secondary pages for the other releases
- DEF-073 (Noddy cost about 1.9 microseconds per voxel; "200^3-voxel models, geophysics included, in under 15 s on one Xeon Gold 6254"): Source says: "taking under 15 s to generate 200x200x200 voxel models with both geological and geophysical representations combined using an Intel Xeon Gold 6254 CPU at 3.10 GHz" (Jessell et al. 2022, https://essd.copernicus.org/articles/14/381/2022/). Arithmetic: 15 s / 8,000,000 voxels = 1.9 microseconds per voxel.
  Result: MATCH
  Evidence level: primary full text (article page)
- DEF-010/DEF-038/DEF-066 (Noddy's sine table of 629 values, 3.1415927 constants, fault geometry; Jessell & Valenta 1996): Source says: not read (the chapter was not fetched; the source files are in the noddyverse repository, whose unEvents.c header was fetched and showed `#define TOLERANCE 0.001`, which matches DEF-067).
  Result: PARTIAL (TOLERANCE 0.001 in unEvents.c confirmed; the other constants were not fetched)
  Evidence level: primary full text (unEvents.c first 40 lines only)

## Deposit models (DEP)

Results: MATCH 35, PARTIAL 10, MISMATCH 2, COULD NOT READ 4.

Numbers below are quoted from what I fetched and saw. "Report" means the PDF text layer unless I say "scan" (Bulletin 1693 figure pages were decoded from images and read by eye).

#### Porphyry copper (OFR 2008-1155, Singer 2005)

- S1 (DEP-001): coded p90 33, p50 240, p10 1500 Mt, n = 422. Source says: Table 1, "General Cu ... Tons 422 1,500 240 33" under the headers "10th percentile / 50th percentile / 90th percentile of deposits"; and Fig. 2 "Tonnage model for all porphyry copper deposits, n = 422" labelled "33 ... 240 ... 1,500" (https://pubs.usgs.gov/of/2008/1155/of2008-1155.pdf).
  Result: MATCH
  Evidence level: primary full text

- S2 (DEP-002; earth flag 4): coded Cu p90 0.24, p50 0.44, p10 0.75 (and "0.80 is a figure tick"). Source says: Table 1 "Cu grade 422 0.75 0.44 0.24"; Fig. 3 labels "0.44 0.75 / 0.24" and the axis ticks "0.13 0.80 2.0 / 0.08 0.50 1.3 / 0.32 / 0.2". So 0.80 is an axis tick, and the research note's 0.80 was wrong (same URL).
  Result: MATCH
  Evidence level: primary full text

- S3 (DEP-003, 004, 005): coded Mo p10 0.03, p50 0.002, p90 0.0; Au 0.5 / 0.05 / 0.0; Ag 3.8 / 0.0 / 0.0; presence 228, 256, 172 of 422. Source says: Table 1 "Mo grade 422 0.03 0.002 0.0", "Au grade 422 0.5 0.05 0.0", "Ag grade 422 3.8 0.0 0.0"; text "Grades not available (always for by-products) are treated as zero". Presence counts are not printed. My recount of PorCuTX2008.txt (https://pubs.usgs.gov/of/2008/1155/data/PorCuTX2008.txt, 427 records with a tonnage, not 422) gives positive Mo 232, Au 249, Ag 176.
  Result: PARTIAL (percentiles match; presence counts 228/256/172 not reproduced: 232/249/176 on the file I downloaded, which has 5 more tonnage records than the report's 422)
  Evidence level: primary full text (table) and primary data file (recount, crude parse)

- S4 (DEP-006 to 009): coded r(ton, Cu) = 0.1 not significant; r(ton, Au) = -0.28, n = 256; r(Mo, Au) = -0.16, n = 148; r(Au, Ag) = 0.26, no n. Source says: "a low positive correlation (r = 0.1) that is not significant at the one percent level ... Tonnage is correlated with gold grade (r = -0.28**, n = 256) and gold is correlated with molybdenum (r = -0.16*, n = 148) and with silver (r = 0.26**)" (same URL).
  Result: MATCH
  Evidence level: primary full text

- S5 (DEP-029, DEP-030; earth flag 14): coded subtype tonnage 20c 34/200/1200 (n 115), 17 30/250/1400 (n 256), 21a 48/280/4800 (n 51), and the data file doubting 4800 ("may be a typo, Fig. 17 reads about 3800"). Source says: Table 1 "Cu-Mo (21a) Tons 51 4,800 280 48"; Fig. 17 labels "280 4,800 48". The subtype rule: "porphyry Cu-Au (type 20c) if Au/Mo greater than or equal to 30, porphyry Cu-Mo (type 21a) if Au/Mo less than or equal to 3 ... porphyry Cu (type 17) otherwise" (same URL).
  Result: MATCH (4,800 is printed in both the table and the figure; no typo; the data-file doubt is unfounded)
  Evidence level: primary full text

- S6 (DEP-013): coded min separation 2,000 m. Source says: "All mineralized rock or alteration within two (2) kilometers was combined into one deposit" (OFR 2008-1155 Rules applied); SIR 2010-5070-B repeats it; the Singer 2005 abstract: "deposits that have mineralization or alteration separated by less than an arbitrary but consistent distance, 2 km for porphyry copper deposits, are combined into one deposit".
  Result: MATCH
  Evidence level: primary full text (OFR, SIR) and primary abstract (Singer 2005)

- S7 (DEP-011): coded density p90 4, p50 15, p10 35 per 100,000 km2. Source says: "Porphyry copper deposit densities vary from 2 to 128 deposits per 100,000 km2 of exposed permissive rock ... Ninety percent of the control areas have densities of four or more deposits, 50 percent have densities of 15 or more deposits, and 10 percent have densities of 35 or more deposits per 100,000 km2" (abstract, https://pubs.usgs.gov/publication/70197200).
  Result: MATCH (the numbers; the unit is deposits and prospects per 100,000 km2 of EXPOSED permissive rock in 19 control areas, see S9)
  Evidence level: primary abstract

- S8 (DEP-034): coded min 2, max 128. Source says the same abstract sentence ("vary from 2 to 128").
  Result: MATCH
  Evidence level: primary abstract

- S9 (DEP-153; earth flag 5): coded: density applied to the whole permissive tract area. Source says: "Only 286 deposits and prospects that have more than half of the deposit not covered by postmineral rocks, sediments, or ice were counted ... Porphyry copper deposit density is inversely related to the exposed area of permissive rock. The linear regression line and confidence limits constructed with the 19 control areas can be used to estimate the number of undiscovered deposits, given the size of a permissive area. ... we estimate a 90 percent chance of at least four, a 50 percent chance of at least 11, and a 10 percent chance of at least 34 undiscovered porphyry copper deposits ... in a permissive area of about 76,000 km2" (same abstract). Arithmetic on that example (mine): 4, 11, 34 deposits in 0.76 x 100,000 km2 = 5.3, 14.5, 44.7 per 100,000 km2.
  Result: PARTIAL (the density counts deposits plus prospects per exposed permissive rock and falls with tract size; the engine applies a flat 15 or 17.8 to the total tract area. At the paper's own 76,000 km2 example the median is 14.5 per 100,000 km2, so the flat value is the right size for tracts of 40,000-82,000 km2, but not size-dependent. The regression coefficients are not in the abstract)
  Evidence level: primary abstract

- S10 (DEP-147): coded split-lognormal mean 17.8 from 4/15/35. Source says: no mean is given in the abstract. My arithmetic with the split lognormal (sigma_lo = ln(15/4)/1.2816 = 1.031, sigma_hi = ln(35/15)/1.2816 = 0.661; mean = 15 x (e^(0.532) x Phi(-1.031) + e^(0.2185) x Phi(0.661)) = 15 x (1.702 x 0.1512 + 1.244 x 0.7457) = 17.8) reproduces 17.8.
  Result: PARTIAL (17.8 is a project construct derived from the three published percentiles; it is not a number in Singer et al. 2005. For comparison, the PP 1640-A abstract says of its podiform chromite densities "highly skewed distribution, suggesting that mean estimates can be misleading"; that is a different deposit type, but the same caution applies to a mean taken from a skewed density distribution)
  Evidence level: secondary (my arithmetic on a primary abstract)

- S11 (DEP-011, 147; the task's Singer 2008 / Singer and Menzie 2010 question): coded density from the 2005 paper. Source says: Singer 2008b abstract, "For podiform chromite, porphyry copper, and volcanogenic massive sulfide deposit types, the size of tract that geologically could contain the deposits is an excellent predictor of the total number of deposits" (https://pubs.usgs.gov/publication/70197201); search snippet "deposit densities of 0.14 Area^-0.61 for porphyry copper". Applying that equation (units unseen; if density is deposits per km2 and area in km2) to 76,000 km2 gives 14.7 per 100,000 km2, matching the 2005 example (14.5).
  Result: PARTIAL (the area dependence is confirmed; the equation itself is a snippet only; the book Singer and Menzie 2010 could not be read)
  Evidence level: primary abstract (Singer 2008b); search snippet (equation)

- S12 (DEP-154, 171; earth flag 6): coded "Circular 1178 p. 15: USGS assessments count deposits to 1 km; a deposit counts if any part is within 1 km". Source says: "Estimates of the undiscovered resources were made to a depth of 1 km beneath the surface. ... Because some mineral deposits have vertical extents of more than 1 km, the rule was adopted such that if any part of a deposit was judged to be within 1 km of the surface, then the whole deposit was counted. ... Although direct mineral exploration is seldom conducted below 1 km, deposits may be explored and developed up to 3 km deep once they have been discovered at lesser depths." The passage sits under the page header "METHOD OF ASSESSMENT 5" (https://pubs.usgs.gov/circ/c1178/c1178.pdf). In the contents, p. 15 falls in the discussion of zinc.
  Result: MISMATCH on the page (the rule is on printed p. 5, not p. 15; the rule itself matches; the research note's "1-3 km" mixes the 1 km rule with the 3 km development remark)
  Evidence level: primary full text

- S13 (DEP-041; earth flag 10): coded min_stable_myr 0.5 attributed to Sillitoe 2005; data file says "typically 3-9 Myr" was not found. Source says: "Mature oxidation and enrichment are promoted by the following: uninterrupted supergene activity for at least 0.5 m.y. but typically minima of 3 to 9 m.y.; tectonically or isostatically induced surface uplift ...; and hot, semiarid to pluvial climates so long as erosion rates remain in balance with rather than outpacing supergene processes" (Crossref abstract of doi 10.5382/av100.22).
  Result: PARTIAL (0.5 is the stated minimum; "3 to 9 m.y." IS in Sillitoe 2005, so the data file's statement that it was not found is wrong; the engine uses the lower bound only)
  Evidence level: primary abstract

- S14 (DEP-043): coded grade_mult [2.0, 3.0]. Source says: "Enriched grades may attain 1.5 to >2 percent Cu, commonly two or three times the hypogene tenor" and "Enrichment, by factors of three or even more" (Sillitoe 2005 abstract); SIR 2010-5070-B: "Copper grades of enriched copper sulfide zones are up to 3 times those of underlying hypogene ore ... in Andean PCDs (Sillitoe, 2005). Enrichment factors in some southwestern North American PCDs approach 8" (https://pubs.usgs.gov/sir/2010/5070/b/pdf/SIR10-5070B.pdf).
  Result: MATCH (2-3x is the common range; the abstract also gives "or even more", and 8 for SW North America)
  Evidence level: primary abstract and primary full text

- S15 (DEP-040; earth flag 9): coded precip_mm [100, 1500]. Source says: SIR 2010-5070-B, "Precipitation for this period, inferred to be 10 cm/year or more, permitted significant circulation of meteoric water ... (Clark and others, 1990)"; Sillitoe 2005 abstract: "hot, semiarid to pluvial climates" and "prevalence of hydroxychlorides in northern Chile testifies to exceptionally arid supergene conditions".
  Result: PARTIAL (the 10 cm/yr is a statement about one stage of the central Andes, not a minimum for enrichment; Sillitoe 2005 gives semiarid and, for oxidation, arid conditions. The lower bound is therefore stricter than the sources)
  Evidence level: primary full text and primary abstract

- S16 (DEP-048): coded 0.4-6.2 Myr per deposit, onset at least 11 Myr after porphyry, active 34-14 Ma (at least 20 Myr). Source says: "K/Ar ages ranging from about 34 to 14 Ma ... a minimum of 20 m.y. Supergene activity at individual deposits lasted for at least 0.4 to 6.2 m.y. ... Supergene activity commenced no less than 11 m.y. after generation of each porphyry copper deposit" (Sillitoe and McKee 1996 abstract, via OpenAlex).
  Result: MATCH
  Evidence level: primary abstract

- S17 (DEP-018, 035, 036, 047): coded vertical extent of hypogene ore 1250 m (midpoint); enriched ore tens of Mt to >1500 Mt at 0.4-1.7 percent Cu, at most 750 m thick. Source says: "The vertical extent of hypogene copper ore in porphyry copper deposits is generally less than or equal to 1 to 1.5 km"; "Enriched copper sulfide ores are as much as 750 m thick but most are tens to several hundred meters thick and consist of tens of megatons to more than 1.5 gigatons at 0.4 to 1.7 percent copper" (SIR 2010-5070-B).
  Result: MATCH (the 1250 midpoint is the project's own)
  Evidence level: primary full text

- S18 (DEP-032): coded median ore area 0.6 km2, sulfide 3.7, alteration 5.1; log Cu = 5.67 + 0.796 log sulfide area, r = 0.56. Source says: SIR 2010-5070-B "range from 0.02 to 28 km2 with a median area of 0.6 km2 ... altered rock ... median area of 5.1 km2 ... median value of 3.7 km2"; OFR 2008-1155 Fig. 26 "r = 0.56** log Cu = 5.67 + 0.796 log sulfide area".
  Result: MATCH (read in the text, not in Table E1 itself)
  Evidence level: primary full text

- S19 (DEP-017, 031, 033): coded emplacement depth medians 1.2 / 1.5 / 2.3 km and placer share 4.1-9.5 percent, digitised from OFR 2008-1155 Figs. 24 and 23. Source says: the figure text carries only axis ticks ("1.0 1.6 2.5 4.0 0.63 0.4 0.25" for Fig. 24).
  Result: COULD NOT READ (vector figures, no renderer available here; the medians are not printed in the text)
  Evidence level: none

#### BIF, placer, MVT, sedex grade-tonnage (Bulletin 1693 figures decoded from the scan; Singer 2009; SIR A)

- S20 (DEP-062, 063): coded BIF tonnage 11 / 170 / 2400 Mt, Fe 30 / 53 / 66 percent, n = 66. Source says: Fig. 172 (p. 229) "Tonnages of Algoma Fe and Superior Fe deposits", n = 66, arrows labelled 11, 170, 2400 (million tonnes); Fig. 173 (p. 230) iron grade arrows 30, 53, 66, n = 66 (https://pubs.usgs.gov/bul/1693/report.pdf). OFR 95-831: "median deposit size is 170 million tonnes (Mosier and Singer, 1986)".
  Result: MATCH
  Evidence level: primary full text (scan, read by eye) and secondary (OFR 95-831)

- S21 (DEP-050, 051, 052, 053, 054, 055): coded placer tonnage 0.022 / 1.1 / 50 Mt (n 65), Au 0.084 / 0.2 / 0.48 g/t, Ag p10 0.036, r(ton, Au) = -0.35, r(Au, Ag) = 0.66 (n 16), 2.0 t/m3, 1.6 km rule. Source says: Fig. 196 (p. 263) arrows 0.022, 1.1, 50, n = 65; Fig. 197 (p. 264) Au 0.084, 0.2, 0.48 and Ag 0.036; text: "deposit volume and grades have been converted to metric tons and grams per metric ton using 2.0 metric tons per cubic meter, the average density of wet sand and gravel. Gold grade is correlated with tonnage (r = -0.35) and with silver grade (r = 0.66, n = 16)"; "grade and tonnage figures are for districts or for placer operations within one mile (1.6 km) of one another" (same URL).
  Result: MATCH
  Evidence level: primary full text

- S22 (DEP-074 to 078): coded CAam tonnage 0.93 / 10 / 86 Mt (n 132), Zn 1.7 / 5.1 / 12, Pb 0.19 / 1.5 / 5.0, Ag p10 37; r values. Source says: Table 2 "CAam Tons 132 86.0 10.0 0.93 / Zn grade 132 12.0 5.1 1.7 / Pb grade 132 5.0 1.5 0.19 / Ag grade 132 37.0 0.0 0.0"; text "Tonnage is correlated with zinc grade (r = -0.30**, n=128) in the CAam ... Zinc grade is correlated with lead grade in the CAam group (r = 0.16**, n=117) ... Lead grade is correlated with silver grade in the CAam group (r = 0.45**, n=56)" (https://pubs.usgs.gov/of/2009/1252/of2009-1252_text.pdf).
  Result: MATCH
  Evidence level: primary full text

- S23 (DEP-102 to 105): coded SHam 5 / 25 / 300 Mt (n 25), Zn 2.3 / 8.0 / 14.0, Pb 0.88 / 2.1 / 5.3, Ag 0 / 21 / 71, Cu p10 1.0, r(Zn, Pb) = 0.72. Source says: Table 2 "SHam Tons 25 300.0 25.0 5.0 / Zn grade 25 14.0 8.0 2.3 / Pb grade 25 5.3 2.1 0.88 / Cu grade 25 1.0 0.0 0.0 / Ag grade 25 71.0 21.0 0.0"; "in the SHam group (r = 0.72**, n=25)" (same URL).
  Result: MATCH
  Evidence level: primary full text

- S24 (DEP-101): coded Bulletin 1693 32a/32b tonnage 2.2 / 35 / 540 Mt (n 20), Zn 1.4 / 4 / 12, Pb p50 0.87, p10 3.6. Source says: Fig. 168 (p. 225) arrows 2.2, 35, 540, n = 20; Fig. 169 Zn 1.4, 4, 12; Fig. 170 (p. 226) Pb 0.87 and 3.6; Fig. 171 Ag 0.48 and 19 (same Bulletin URL).
  Result: MATCH
  Evidence level: primary full text (scan)

- S25 (DEP-116; earth flag 14): coded 31a tonnage 1.7 / 15 / 130 Mt (n 45), Zn 5.6, Pb 2.8. Source says: Fig. 159 (p. 213) arrows 1.7, 15, 130, n = 45; Fig. 160 Zn 2.4, 5.6, 13; Fig. 161 Pb 1.0, 2.8, 7.7; Fig. 162 Ag 30 and 160; Fig. 163 Cu 0.28 (same Bulletin URL). OFR 95-831: "Most deposits are 1.7 (90th percentile) to 130 (10th percentile) million tons; median deposit size is 15 million tons (Menzie and Mosier, 1986)". SIR 2010-5070-N instead writes "Most economic sedex deposits are between 1.4 and 120 Mt; the median deposit size is 15 Mt".
  Result: MATCH (the printed intercepts are 1.7 and 130; SIR 2010-5070-N's 1.4 and 120 is a loose restatement, not the table)
  Evidence level: primary full text (scan) and secondary (OFR 95-831, SIR N)

- S26 (DEP-100, 081, 090): coded SIR 2010-5070-A MVT-only 0.65 / 7.0 / 42.4 Mt, Zn 6.0, Pb 1.9, fluids 75-200 C (50-250), 10-30 wt percent NaCl eq., Zn:Pb about 10:1, Pine Point 60 ore bodies in 1,600 km2, economic mineralisation "a few to several tens of meters". Source says: Table 1 "Size (Mt) 0.06 0.65 1.90 7.00 18.1 42.4 394 113 ... Pb (%) ... 1.90 ... Zn (%) ... 6.00"; "Fluid inclusion temperatures in MVT deposits range from about 50 C to 250 C; however, most of the temperatures are between 75 C and 200 C"; "typically 10 to 30 wt. percent NaCl equivalent"; "about a 10:1 mass ratio of Zn to Pb metal"; "These individual ore breccias total 60 deposits in a 1,600 km2 district"; "economic mineralization is usually on the order of a few to several tens of meters" (https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf).
  Result: MATCH. Related difference the code already knows: MVT-only 10th/50th/90th columns 0.65 / 7.0 / 42.4 against the CAam proxy the engine uses, 0.93 / 10 / 86, so the engine's large end is twice the MVT-only figure.
  Evidence level: primary full text

- S27 (DEP-091): coded brine_flow_km 600. Source says: "Neoproterozoic and Phanerozoic platform carbonate sequences located less than ~600 km inboard of Phanerozoic orogenic belts ... have the potential" (SIR 2010-5070-A, under Geological Assessment Guides Essential for Inclusion in Permissive Tracts). The model also says extension "can extend several hundred kilometers into the foreland".
  Result: MATCH
  Evidence level: primary full text

#### Densities (sedex, placer), SEDEX and brine facts

- S28 (DEP-107; earth flags 6, 15): coded sedex 12 NMRA tracts, 207,508 km2, about 2 / 10 / 26 undiscovered deposits, density 1 / 5 / 12.5 per 100,000 km2. Source says: I summed the attribute tables of the three sedex shapefiles in the NMRA download (Sedimentary_exhalative_Zn, Sedimentary_exhalative_ZnPb, ak/Sedex_ZnPb; unique tract and model pairs): "assessed tracts (with estimates): n=12 area=207508 sum_p90=2 sum_p50=10 sum_p10=26"; 23 sedex tracts are listed in all, 561,881 km2, and 11 of them (354,373 km2, including EC02 at 220,100 km2) have no estimate (https://mrdata.usgs.gov/nmra/nmra.zip). 2/207,508, 10/207,508, 26/207,508 x 100,000 = 0.96 / 4.8 / 12.5.
  Result: MATCH (exact reproduction; the density is a sum of tract percentiles over the tracts that have estimates, 37 percent of the listed sedex tract area, so it is not a published density)
  Evidence level: primary data (USGS dataset)

- S29 (DEP-056, 061; DEP-051): coded placer "no USGS placer density exists" (density 1 / 1 / 1 placeholder); Bay RMP tract PLACER1 1 / 2 / 7 undiscovered deposits in 55,350 km2. Source says: SIR 2007-5039 tract table row "PLACER1 55,350 Placer Au 39a 1 2 7 3.1 1.1 0.2 gpt Au" (https://pubs.usgs.gov/sir/2007/5039/pdf/sir20075039.pdf). PP 1640-A abstract: "Placer gold deposits in two locations in Alaska have deposit densities of 0.0054 and 0.0061 deposit per square kilometer" (https://pubs.usgs.gov/pp/p1640a/p1640A.pdf).
  Result: MISMATCH on the claim that no USGS placer density exists: PP 1640-A gives 0.0054 and 0.0061 per km2 (540 and 610 per 100,000 km2) for placer gold, a different unit (district counts, 1.6 km rule) from the tract estimates (2 per 55,350 km2 = 3.6 per 100,000 km2). The type is a child type and the density is unused, so no effect on the engine. PLACER1 row itself MATCHES.
  Evidence level: primary full text

- S30 (DEP-080, 065): coded MVT and BIF density placeholders; PP 1640-A Table 1 has no Zn-Pb or iron formation. Source says: the PP 1640-A abstract lists 13 types and no Zn-Pb or iron formation; it does list "Porphyry copper deposits in Nevada and Arizona have deposit densities of 0.00015 and 0.0012 deposit per square kilometer" (15 and 120 per 100,000 km2).
  Result: MATCH (no MVT or BIF density; the two porphyry figures sit inside the 2-128 range of Singer 2005)
  Evidence level: primary abstract (Table 1 body not read)

- S31 (DEP-110, 116; earth flag 14): coded sedex brines 100-200 C, 17-30 wt percent TDS, leaching window 150-250 C, basin fill at least 3 km, rimming platforms 100,000 to 1,000,000 km2, ore density note 2.2 g/cm3 is sandstone. Source says: SIR 2010-5070-N "sedex deposits form from 100 to 200 C brines with salinities between 10 and 30 wt. percent total dissolved solids"; "Saline brine (>17 percent TDS) is necessary to form a sedex deposit"; "a temperature interval of burial of approximately 150-250 C"; "at least 3 km of sediment fill above a crystalline basement"; "rimmed by unusually large (10^5 and 10^6 square kilometers) shallow-water epeiric carbonate platforms"; "typical sandstone bulk density of 2.2 grams per cubic centimeter" (https://pubs.usgs.gov/sir/2010/5070/n/sir20105070n.pdf).
  Result: PARTIAL (all but salinity match; "17-30" is a hybrid of the observed 10-30 and the permissive minimum of more than 17 percent)
  Evidence level: primary full text

- S32 (DEP-111): coded 85 percent within 200 km of a thick-thin lithosphere step. Source says: "85% of sediment-hosted base metals, including all giant deposits (>10 megatonnes of metal), occur within 200 kilometres of the transition between thick and thin lithosphere" (abstract via Semantic Scholar).
  Result: MATCH
  Evidence level: primary abstract

#### Genesis rules and timing

- S33 (DEP-086, 112, 188): coded MVT and sedex cannot form before 2.02 Ga, cited as "Leach et al. 2010" and attached to SIR 2010-5070-A. Source says: SIR 2010-5070-A has no 2.02 Ga statement (it reports MVT in Proterozoic rocks, 9.0 percent of MVT zinc, and one Neoarchean deposit, Pering-Bushy Park). The Economic Geology paper Leach et al. 2010 (105:593-625) says: "The emergence of CD and MVT deposits in the rock record between 2.02 Ga, the age of the earliest known deposit of these ores, and 1.85 to 1.58 Ga ... Although MVT and CD deposits appeared for the first time in Earth history at 2.02 Ga, only CD deposits were important repositories for Pb and Zn in sediments between the Great Oxygenation Event, until after the second oxidation of the atmosphere in the late Neoproterozoic" (abstract via OpenAlex).
  Result: PARTIAL (2.02 Ga matches the Economic Geology paper, not SIR A; the same abstract says MVT was minor until the late Neoproterozoic, while the code gates MVT and sedex on one clock)
  Evidence level: primary abstract and primary full text

- S34 (DEP-087, 094): coded Garven 1985: brines 60-100 C, formation time 0.5-5 Myr, MVT integration 5 Myr. Source says: Garven 1985 abstract "paleoflow rates of 1 to 5 m/yr ... paleotemperatures in the range to 60 to 100 C. Under these possible site conditions, the Pine Point deposits could have formed over a time interval of 0.5 to 5.0 million years" (OpenAlex abstract of doi 10.2113/gsecongeo.80.2.307). For contrast, SIR 2010-5070-A reports ore-event durations of about 200 kyr (biomarkers, Upper Mississippi Valley) and not more than about 50 kyr (Cracow-Silesia).
  Result: MATCH (the 5 Myr integration time is the top of the Garven range; the dating studies in SIR A give much shorter events)
  Evidence level: primary abstract and primary full text

- S35 (DEP-083, 098): coded Garven and Freeze 1984 base case, 1 km water-table drop over 300 km (3.3 m per km). Source says: neither part's abstract (doi 10.2475/ajs.284.10.1085 and .1125) contains a base case; part 2 abstract: "gravity-driven groundwater flow systems are capable of sustaining favorable fluid-flow rates, temperatures, and metal concentrations for ore formation at groundwater-discharge areas near the edge of a basin".
  Result: COULD NOT READ (paywalled; the paper is in two parts and the code does not say which)
  Evidence level: primary abstract (no number)

- S36 (DEP-024): coded convergence ~100 km/Myr, obliquity ~15 degrees, slab age 25-70 Myr. Source says: "a combination of rapid convergence rates (~100 km/Myr), subduction obliquity of ~15 degrees, a subducting plate age between ~25-70 Myr old, and a location far from the subducting trench boundary (>2000 km) represents favorable conditions" (abstract via OpenAlex).
  Result: MATCH (the abstract adds a distance-from-trench criterion the curves do not use)
  Evidence level: primary abstract

#### Survival

- S37 (DEP-166, 177, 191): coded Kesler and Wilkinson 2008: about 125,895 porphyries formed, 47,789 remain (38 percent), copper above 3.3 km within reach, "about two-thirds recycled". Source says: "∼125,895 porphyry copper deposits were formed during Phanerozoic time, that only ∼47,789 of these remain at various crustal depths ... about two-thirds of this has been recycled by uplift and erosion. The amount of copper in deposits above 3.3 km, a likely limit of future mining, could supply current world mine production for 5500 yr" (abstract via OpenAlex). 47,789/125,895 = 0.38.
  Result: MATCH (the "two-thirds" in the abstract refers to the copper concentrated into deposits, not to a count of deposits; 62 percent of the deposits are gone by the paper's own numbers)
  Evidence level: primary abstract

- S38 (DEP-175, 191): coded 158 m/Myr porphyry exhumation (167 epithermal, 63 orogenic gold). Source says: "Exhumation rates calculated from modal ages and emplacement depths for epithermal, porphyry copper, and orogenic gold deposits of Phanerozoic age are 167, 158, and 63 m/m.y., respectively" (abstract via OpenAlex).
  Result: MATCH
  Evidence level: primary abstract

#### Coal

- S39 (DEP-119, 128): coded 1.32 t/m3 bituminous (anthracite 1.47, subbituminous 1.30, lignite 1.29). Source says: Circular 891 Table 2 "Anthracite and semianthracite 1.47, Bituminous coal 1.32, Subbituminous coal 1.30" with "Metric tons per square hectometer-meter 14,700 13,200 13,000 12,900" (the lignite label is garbled in the text layer; 12,900 per hm2-m gives 1.29) (https://pubs.usgs.gov/circ/1983/0891/report.pdf).
  Result: MATCH
  Evidence level: primary full text

- S40 (DEP-125, 129): coded depth limit 1,800 m, resources at least 0.35 m (anthracite and bituminous) / 0.75 m (subbituminous and lignite), reserve base 0.7 m / 1.5 m, ash 33 percent, indicated radius 1.2 km, inferred 4.8 km; "Table 3 p. 24". Source says: Table 3 "Identified and undiscovered resources: Anthracite and bituminous coal <6,000 (1,800) > 14 (35) ... Subbituminous coal and lignite <6,000 (1,800) > 30 (75) ... Reserve base: Anthracite and bituminous coal < 1,000 (300) > 28 (70) ... Subbituminous < 1,000 (300) > 60 (150)"; "All coal beds deeper than 6,000 feet (1,800 m) are excluded"; "beds that contain more than 33 percent ash also are excluded"; "indicated: between 1/4 mile (0.4 km) and 3/4 mile (1.2 km) ... inferred: between 3/4 mile (1.2 km) and 3 miles (4.8 km)". The report's list of tables puts Table 3 on p. 24.
  Result: MATCH (coal_seam.json's note "Table 3 p. 14" is the only wrong page)
  Evidence level: primary full text

- S41 (DEP-117, 118; earth flags 2, 23): coded coal field size 3,800 / 19,800 / 115,000 Mt and density 4 / 14 / 40 per 100,000 km2. Source says: no source gives either set. The type notes say "ESTIMATE, not a USGS figure: no source counts minable seam fields", "every field count is a judgement about what to call a field" and that several inputs are "the research's own arithmetic". Inputs I could check: Fort Union Formation "exists over 22,000 square miles in the Powder River Basin" (PP 1625-A chap. PS); Gillette study area "about 1,026,000 acres (415,208 hectares)" (chap. PG); Wyodak-Anderson "about 5,414,000 acres" and "Grand total (MST) ... 550,000" (chap. PN).
  Result: PARTIAL (the cited inputs MATCH; the tonnage and density distributions are the project's judgement and have no named published source. Wikipedia is listed among the inputs for field areas)
  Evidence level: primary full text (inputs); none for the distributions

#### Petroleum

- S42 (DEP-131, 132): coded Niger Delta Agbada oil field median 12 MMBO, range 1 to 1500, converted at 0.1364 t per barrel. Source says: data form "Size of Undiscovered Fields ... Oil in oil fields (mmbo) min. size 1 median size 12 max. size 1500" and "Number of Undiscovered Fields ... Oil fields min. no. (>0) 200 median no. 580 max no. 1000"; "Assessment Geologist: M.L. Tuttle, M.E. Brownfield and R.R. Charpentier" (au719211.pdf from https://pubs.usgs.gov/dds/dds-060/regions/reg7.zip). 0.1364 t per barrel is 0.858 t/m3 x 0.159 m3 (my arithmetic).
  Result: MATCH (median 12, min 1, max 1500, field counts 200 / 580 / 1000)
  Evidence level: primary full text (data form)

- S43 (DEP-131): coded p90 2.4 and p10 89 MMBO for the same unit. Source says: the form gives only min / median / max; the percentiles were digitised by the project from the "Undiscovered Field-Size Distribution" bar chart, whose bar heights are graphics and not in the text layer.
  Result: COULD NOT READ
  Evidence level: none

- S44 (DEP-136, 144): coded TTI 15 onset, 75 peak, 160 end; deadlines about 500 (40 API), 1000 (50 API), 1500 (wet gas), above 65,000 (dry gas). Source says: "Threshold values of Lopatin's time-temperature index of maturity (TTI) are: 15 onset of oil generation; 75 peak oil generation; 160 end oil generation; approx. 500 40 oil preservation deadline; approx. 1000 50 oil preservation deadline; approx. 1500 wet gas preservation deadline; and > 65,000 dry gas preservation deadline" (OSTI record 6845191 abstract of Waples 1980).
  Result: MATCH
  Evidence level: primary abstract

- S45 (DEP-126, 144): coded TTI 15/75/160/1500 = Ro 0.6/1.1/1.3/3.8. Source says: KGS Table 1 "Onset of oil generation (Waples) 15 ... 0.6; Peak oil generation (Waples) 75 ... 1.1; End oil generation (Waples) 160 ... 1.3; Upper TTI limit for wet gas (Waples) 1,500 ... ~3.8" (https://www.kgs.ku.edu/Current/1997/newell/newell3.html); the table is "TTI to Ro is from Waples (1981)".
  Result: MATCH (Waples 1981, not 1980)
  Evidence level: secondary source (Kansas Geological Survey, Newell)

- S46 (DEP-142, 144): coded source rock generates only above 80 C; oil window 80-120 C; passive-margin gradient 30 C per km. Source says: "Assuming a geothermal gradient typical for a long-standing passive margin (~30 C/km), the depth to the oil window (80 C to 120 C) was greater than the thickness of the entire Ellesmerian sequence" (OFR 2003-253, executive summary, https://pubs.usgs.gov/of/2003/0253/pdf/of03-253.pdf).
  Result: PARTIAL (the numbers are in the report, as a one-sentence assumption in a regional study; they are not a general threshold, and Waples' TTI scheme gives no 80 C)
  Evidence level: primary full text

- S47 (DEP-140): coded deep gas counted below 4.5 km. Source says: "deep (>4.5 km or about 15,000 ft), undiscovered conventional natural gas resources" (Dyman et al. 2002 abstract).
  Result: MATCH (the 10 km assessment depth is the project's own choice; the abstract only defines "deep")
  Evidence level: primary abstract

#### Other

- S48 (DEP-185): coded permissive boundary leaves under 1 in 100,000 to 1,000,000 chance outside. Source says: "Permissive boundaries are drawn for one or more deposit types such that the probability of a deposit lying outside the boundary is negligible, that is, less than 1 in 100,000 to 1,000,000" (Singer 1993 abstract, https://pubs.usgs.gov/publication/70018261).
  Result: MATCH
  Evidence level: primary abstract

- S49 (DEP-162): coded top 10 percent hold 47-79 percent of each metal. Source says: "between 47 and 79 percent of metal contained in the largest 10 percent of deposits" (Singer 1995 abstract, https://pubs.usgs.gov/publication/70197194).
  Result: MATCH
  Evidence level: primary abstract

- S50 (DEP-164, 165): coded Poisson PTRS constants b = 0.931 + 2.53 sqrt(lambda), a = -0.059 + 0.02483 b, invalpha = 1.1239 + 1.1328/(b - 3.4), vr = 0.9277 - 3.6224/(b - 2), switch at lambda 10; Acklam relative error under 1.2e-9, split at 0.02425. Source says: NumPy's implementation of "The transformed rejection method for generating Poisson random variables, W. Hoermann" has the same four constant lines and `if (lam >= 10)` (numpy random distributions.c); Acklam: "relative error has an absolute value less than 1.15 x 10^-9 in the entire region" and "The central region is when 0.02425 <= p <= 1 - 0.02425".
  Result: MATCH (1.2e-9 is a rounding up of 1.15e-9)
  Evidence level: secondary source (NumPy source) for Hörmann; primary web note (archived) for Acklam

- S51 (DEP-068): coded BIF acme 2.65-1.85 Ga (Bekker et al. 2010). Source says: the abstract text I could get stops before any age statement ("Banded iron formation (BIF) is dominant in Archean to earliest Paleoproterozoic successions, whereas granular iron formation (GIF) is much more common in Paleoproterozoic successions").
  Result: COULD NOT READ (abstract truncated in OpenAlex; GeoScienceWorld blocked)
  Evidence level: search snippet only (none for the number)

## Deposit formation, coal, petroleum, placers (ORE)

Results: MATCH 16, PARTIAL 14, MISMATCH 1, COULD NOT READ 11.

Evidence labels: "primary abstract" is an abstract read on the publisher, OSTI, USGS or Crossref/OpenAlex; "primary full text" is a full HTML page of the cited work; "secondary" names the page. Quotes are copied from what the fetch returned; WebFetch answers through a small model, so quotes are as returned.

#### A. Waples 1980 and the TTI correlation

- ORE-159: coded doubling interval 10 degC. Source says: "The rate of the chemical reactions involved in thermal maturation of organic material appears to double with every 10/sup 0/C (18/sup 0/F) rise in temperature." (Waples 1980 abstract, OSTI record 6845191, https://www.osti.gov/biblio/6845191). The KGS page independently says: "the rate of maturation doubles for every 10 degree C rise in temperature" (https://www.kgs.ku.edu/Current/1997/newell/newell3.html).
  Result: MATCH
  Evidence level: primary abstract (OSTI); secondary source (KGS) agrees.

- ORE-160 / 161 / 162: coded oil window TTI 15 onset, 75 peak, 160 end. Source says: "Threshold values of Lopatin's time-temperature index of maturity (TTI) are: 15 onset of oil generation; 75 peak oil generation; 160 end oil generation; approx. 500 40/sup 0/ oil preservation deadline; approx. 1000 50/sup 0/ oil preservation deadline; approx. 1500 wet gas preservation deadline; and > 65,000 dry gas preservation deadline." (same OSTI abstract).
  Result: MATCH (all three)
  Evidence level: primary abstract.

- ORE-197: coded preservation deadlines 500 / 1000 / 1500 / 65,000. Source says: see the quotation above (same abstract). One automated abstract rebuild on OpenAlex garbled this list (it dropped the 1,000 and attached 1,500 to "50 degree"); the OSTI text above is the authoritative one.
  Result: MATCH. The abstract does not say "API gravity"; "40 degree oil" is read by the project as API, which is plausible but not stated in the abstract.
  Evidence level: primary abstract.

- ORE-158: coded TTI increment dt x 2^floor((T - 100)/10), i.e. n = 0 for 100-110 degC. Source says: the abstract gives the doubling per 10 degC only; the KGS page says the method "can be mathematically approximated by a first-order chemical reaction in which the rate of maturation doubles for every 10 degree C rise in temperature" but, as the page was returned to me, it does not state the n = 0 reference interval. Not found in anything I read.
  Result: PARTIAL (doubling confirmed; the 100-110 degC zero point not seen in any source I could read)
  Evidence level: primary abstract; secondary source (KGS).

- ORE-198: coded KGS table: TTI 15 gives Ro 0.6 %, 75 gives 1.1, 160 gives 1.3, 1,500 gives about 3.8. Source says (KGS, Table 1, two separate fetches returned the same rows): "15 : 0.6% : Onset of oil generation (Waples)", "75 : 1.1% : Peak oil generation (Waples)", "160 : 1.3% : End oil generation (Waples)", "1,500 : ~3.8% : Upper TTI limit for wet gas (Waples)" (https://www.kgs.ku.edu/Current/1997/newell/newell3.html). The first fetch described the same three Ro values.
  Result: MATCH to the KGS page as printed. The 3.8 is the KGS value, so it is not a project transcription slip; whether KGS itself is right was not tested (the ledger's own suspicion that 3.8 is high for wet gas stands: AAPG-style ranges put wet gas near 0.8-2.0 % Ro, and Wikipedia gives the gas window as 1.0-1.3 % to about 3.0 %).
  Evidence level: secondary source (KGS web page).

- Open restatements of the 15 / 75 / 160 thresholds. Searched: AAPG Wiki ("Maturation", "Thermal maturation", "Vitrinite reflectance", "Relationships between maturity and hydrocarbon generation", "Petroleum source rocks"), Wikipedia (vitrinite reflectance, kerogen search), SLB Energy Glossary. None restates the TTI thresholds. The AAPG Wiki "Petroleum source rocks" page credits Waples with bringing Lopatin's method to English readers and cites Waples 1980 and 1985, and the "Relationships between maturity and hydrocarbon generation" page says "Values such as 0.6% vitrinite reflectance (Ro) are generally associated with the onset of oil generation or indicate the top of the oil window." The Wikipedia vitrinite reflectance page: "the onset of oil generation is correlated with a reflectance of 0.5-0.6% and the termination of oil generation with a reflectance of 0.85-1.1%. The onset of gas generation ('gas window') is typically associated with values of 1.0-1.3% and terminates around 3.0%." The KGS page is the only open restatement of the three TTI values found.
  Result: PARTIAL for the Ro endpoints (0.6 matches; end of oil 1.3 at TTI 160 in KGS vs 0.85-1.1 in Wikipedia and 1.35 in the project's OFR 94-220 note)
  Evidence level: secondary source (Wikipedia, AAPG Wiki).

#### B. Coal rank, burial, peat

- ORE-152: coded TTI 75 corresponds to Ro about 1.1 %, "the high-volatile bituminous limit". Source says: KGS: "75 : 1.1% : Peak oil generation (Waples)". The KGS row labels 1.1 % as peak oil generation, not as a coal rank limit.
  Result: PARTIAL (the 1.1 % value is confirmed for TTI 75; its use as a rank limit is the project's reading, not the source's)
  Evidence level: secondary source (KGS).

- ORE-155: coded low-volatile bituminous / anthracite boundary at TTI 500 (placeholder). Source says: by log-linear interpolation of the KGS rows (160 gives 1.3 %, 1,500 gives about 3.8 %), TTI 500 is about Ro 2.6 % (my arithmetic: the TTI value of 500 is half-way in log space, which gives 1.3 + 0.51 x 2.5 = 2.57). For the anthracite lower limit, Wikipedia's bituminous coal page (ISO classification) gives "Medium A: Vitrinite reflectance of 1.4 to 2.0" and says that in the international market "bituminous coal is defined as coal with a vitrinite reflectance between 0.5 and 1.9". An open ACS Omega review (https://pmc.ncbi.nlm.nih.gov/articles/PMC12290974/, as returned by the fetch) says "Anthracite: Ro > 2.6%" and lignite "Ro = 0.25-0.38%". An open Frontiers in Microbiology paper (https://pmc.ncbi.nlm.nih.gov/articles/PMC10036379/, Table 2) gives sample values: lignite 0.25, subbituminous C 0.40, high-volatile bituminous 0.81 and 0.89, low-volatile bituminous 1.68.
  Result: PARTIAL. The anthracite onset is stated between about 2.0 % (ISO, upper end of medium rank A) and 2.6 % (review). By the KGS table, Ro 2.0 is TTI about 300 (my interpolation) and Ro 2.6 is TTI about 500. So TTI 500 is at the high end of the range, not wrong outright. The exact ASTM or ISO boundary values in %Ro were not read from a primary source.
  Evidence level: secondary source (Wikipedia; two open papers); arithmetic mine.

- ORE-153 / 154: coded lignite / subbituminous at TTI 3 and subbituminous / high-volatile bituminous at TTI 10 (placeholders). Source says: no Ro-to-rank boundary table was read. Open sample values: lignite Ro 0.25-0.38 % (review), subbituminous C 0.40 % (Table 2, Platt et al. 2023), bituminous reflectance 0.5 and up (ISO via Wikipedia). TTI is not tabulated for these Ro values in any source I read; the KGS table starts at TTI 15 (Ro 0.6 %).
  Result: COULD NOT READ (no source relates TTI 3 or 10 to rank)
  Evidence level: secondary source.

- ORE-116: coded not coded; the research note records about 11:1 peat to coal in a Utah case. Source says: "The peat:coal ratio of the thickness calculated by this method is approximately 11:1." (Ryer and Langer 1980 abstract, rebuilt from OpenAlex, https://api.openalex.org/works/doi:10.1306/212f7b44-2b24-11d7-8648000102c1865d). The abstract is for one bituminous coal of Cretaceous age in central Utah.
  Result: MATCH (11:1; the abstract does not give a 10:1 figure)
  Evidence level: primary abstract.

- Peat accumulation rate (not coded). Source says: "Peat usually accumulates slowly at the rate of about a millimetre per year." (Wikipedia, Peat, citing Keddy 2010 Wetland Ecology; https://en.wikipedia.org/wiki/Peat). No coal-specific accumulation-rate paper was read. The code's mire rule uses total net deposition 5-150 m/Myr (equals 0.005-0.15 mm/yr), which is a different quantity.
  Result: PARTIAL (Earth reference figure recorded; no coded value to compare)
  Evidence level: secondary source (Wikipedia).

- ORE-113 / 114 (coal driver, Bohacs and Suter): coded accommodation roughly matching peat production, drowning when it is too large. Source says: "Terrigenous organic matter preservation as coal requires accommodation increase to approximately match peat production rates." and "Very large accommodation increases stress mires, leading to inundation by clastics or standing water during middle transgressive systems tracts." (Bohacs and Suter 1997 abstract, Crossref).
  Result: MATCH (principle; the curve numbers are the project's design values)
  Evidence level: primary abstract.

- ORE-015 (land plants gate): coded no coal before land plants. Source says: "Coal deposits occur by the Early Devonian" but "the most abundant, geographically extensive, and economically important are of late Paleozoic (Carboniferous-Permian) age." (Nelsen et al. 2016, PMC4780611).
  Result: PARTIAL (the paper puts the first coal in the Early Devonian, about 419-393 Ma, which fits the synthetic land-plant clock at 420 Myr; the paper is about the Carboniferous-Permian peak, not a source for the onset date)
  Evidence level: primary full text (PMC article, introduction).

- ORE-130: coded 1.32 t/m3 for bituminous coal (1,800 short tons per acre-foot). Source says: Circular 891 not read. Arithmetic only: 1,800 short tons x 907.185 kg / 1,233.48 m3 = 1,324 kg/m3.
  Result: COULD NOT READ (PDF unreadable); the conversion is self-consistent
  Evidence level: arithmetic.

#### C. Coal basin areas and field counts

- ORE-123: coded Illinois Basin about 95,000 km2 (36,800 mi2). Source says: Wikipedia, Illinois Basin infobox: "Area: 80,000 sq mi (210,000 km2)" (reference 1 there is a 1987 USGS paper on thrust faults, not an area source). The 36,800 mi2 figure was not found. 36,800 mi2 is 95,300 km2 (arithmetic).
  Result: MISMATCH on the whole-basin area (basin 80,000 mi2 / 210,000 km2 vs coded 36,800 mi2 / 95,000 km2). The coded number may be a coal-bearing area for Illinois alone; that is not demonstrated. The USGS source that probably holds a coal-area figure is PP 1625-D (Hatch and Affolter 2002), whose PDF I could not read. ISGS pages fetched gave no area.
  Evidence level: secondary source (Wikipedia).

- ORE-122: coded Upper Silesia 5,400-5,600 km2. Source says: "Area: 5,400 km2 (in Poland - 4,500 km2, in Czech Republic - 900 km2)" with a note "or 5,600 km2" (Wikipedia, Upper Silesian Coal Basin; refs PWN encyclopedia and an EU "Silesia" PDF).
  Result: MATCH
  Evidence level: secondary source (Wikipedia).

- ORE-126 (Bowen Basin): coded Geoscience Australia about 160,000 km2, Wikipedia more than 60,000. Source says: Geoscience Australia: the Bowen Basin occupies "about 160 000km2" (https://www.ga.gov.au/scientific-topics/energy/province-sedimentary-basin-geology/petroleum/onshore-australia/bowen-basin); Wikipedia: "over 60,000 square kilometres". The two sources differ by a factor of 2.7 because they describe different extents (the larger one includes the Gunnedah and Surat parts of the same system, which I did not verify).
  Result: MATCH for both quoted figures
  Evidence level: secondary source (GA page, Wikipedia).

- ORE-121: coded Sydney Basin onshore 36,000 km2 with five onshore coalfields (Bioregional Assessments context statement). Source says: Bioregional Assessments page: "Covering approximately 24,625 square kilometres, its western boundary is defined by the geological Sydney Basin." and "The Western, Central and Southern coalfields are the main coal fields in the Sydney Basin bioregion." No "36,000" and no count of five was on the page I received (the fetch may have been of the landing page only). Wikipedia, Sydney Basin: "36,295.97 km2".
  Result: PARTIAL (basin area agrees with Wikipedia; the assessment bioregion in the cited program is 24,625 km2 with three main coalfields, which does not support "five coalfields"; this should be re-read in the full context statement)
  Evidence level: primary full text in part (Bioregional Assessments page); secondary source (Wikipedia).

- ORE-118 / 120 (areas by arithmetic): coded Gillette study area about 4,150 km2 (1,026,000 acres) and Fort Union 22,000 mi2 = 57,000 km2. Source says: PP 1625-A chapters not read. Arithmetic: 1,026,000 acres x 4,046.86 m2 = 4,152 km2; 22,000 mi2 x 2.59 = 56,980 km2.
  Result: COULD NOT READ (source PDFs unreadable); the unit conversions are right
  Evidence level: arithmetic.

- ORE-139 / 140 (cyclothem durations 160,000-400,000 yr, one minable seam per 2-4 cycles): Source says: not read. Heckel 1986 and Rasbury et al. 1998 identified and are paywalled; the cyclothem-duration title of Rasbury et al. ("constraints on late Paleozoic cycle durations") fits but no abstract was retrieved.
  Result: COULD NOT READ (paywall)
  Evidence level: none.

#### D. Deposit formation drivers

- ORE-003 / 004 (MVT fluid temperature, three values in the code: 75-200, 60-100, 90-200 degC). Sources read: Garven 1985 abstract: "temperatures ranging 60-100 degC" and "0.5-5.0 million years" (Pine Point model). Basuki 2002 abstract (fluid inclusions, 18 MVT deposits and districts): "The mean Th and salinity values for fifteen MVT deposits are 122 degC +/- 21 degC and 20.7 +/- 2.6 wt% CaCl2 equivalent", "the lower limit of measured MVT deposits (n = 15) ~75 degC". Wikipedia (Carbonate-hosted lead-zinc ore deposits): "typically low temperature (100 degC-150 degC)" and "10-30 wt.% NaCl equivalent". Leach et al. 2005 (90-200 degC) and Leach et al. 2010 USGS (SIR 2010-5070-A) could not be read: the USGS PDF is binary to the fetch tool and its abstract has no temperature.
  Result: PARTIAL. Garven's 60-100 degC is confirmed as his Pine Point model result (a model output, not an ore-fluid measurement). 75 degC is confirmed as the lower end of measured fluid inclusion data (Basuki). The 200 degC upper end and the "90-200 degC from Leach 2005" are unconfirmed. The code's three ranges are three different things (model, data lower limit, data range), not one contradiction.
  Evidence level: primary abstract (Garven, Basuki); secondary source (Wikipedia).

- ORE-013: coded MVT flush integration 5 Myr (Garven: 0.5-5 Myr). Source says: "Under these conditions, Pine Point deposits could have formed over 0.5-5.0 million years during post-Cretaceous time following Rocky Mountain emergence." (Garven 1985 abstract, OpenAlex rebuild).
  Result: MATCH
  Evidence level: primary abstract.

- ORE-001: coded 600 km inboard of Phanerozoic orogenic belts (SIR 2010-5070-A). Source says: the USGS abstract (read) says MVT deposits are "hosted mainly by dolostone and limestone in platform carbonate sequences and usually located at flanks of basins, orogenic forelands, or foreland thrust belts inboard of the clastic rock-dominated passive margin sequences"; "MVT deposits formed mainly during the Phanerozoic with more than 80 percent of the deposits hosted in Phanerozoic rocks". The 600 km figure is not in the abstract.
  Result: COULD NOT READ (the 600 km phrase is in the body; PDF binary)
  Evidence level: primary abstract.

- ORE-014 (2.02 Ga gate on sediment-hosted Zn-Pb). Source says: "The emergence of MVT Pb-Zn deposits in the geologic record at 2.02 Ga, the age of the earliest known deposit of these ores ... corresponds to a time after the Great Oxygenation Event" (Leach et al. 2010, Economic Geology 105:593; abstract as rebuilt by OpenAlex, word order partly scrambled; the intact phrases are "2.02 Ga, age of earliest known deposit of these ores" and "1.85 to 1.58 Ga period of Pb-Zn mineralization in Australia and India"). The abstract also says "MVT reached their maximum abundance during the final assembly and Pangea from Devonian into Carboniferous" and splits sediment-hosted Pb-Zn into clastic-dominated (which includes SEDEX) and MVT.
  Result: PARTIAL. 2.02 Ga is the age of the earliest known MVT deposit. The abstract does not say that clastic-dominated (SEDEX) deposits begin at 2.02 Ga, so gating SEDEX on the same date is the project's extension. It is an earliest-known age, not a physical limit.
  Evidence level: primary abstract (reconstructed).

- ORE-016 / ORE-021 (BIF acme 2.65-1.85 Ga; Superior-type on passive-margin shelves). Source says: Bekker et al. 2010 abstract (rebuilt, truncated): "Larger Superior-type iron formations developed in passive-margin sedimentary rock successions and generally lack direct relationships with volcanic rocks." and "Banded iron formation (BIF) is dominant in Archean to earliest Paleoproterozoic successions, whereas granular iron formation (GIF) is much more common in Paleoproterozoic successions." Wikipedia, Banded iron formation: "most deposits dating to the late Archean (2800-2500 Ma) with a secondary peak of deposition in the Orosirian period of the Paleoproterozoic (1850 Ma)"; Wikipedia, Great Oxidation Event: "Banded iron formation largely disappears from the geological record at 1.85 Ga, after peaking at about 2.5 Ga."
  Result: PARTIAL. ORE-021 (Superior-type on passive margins) MATCH. For the 2.65-1.85 Ga window: the end at 1.85 Ga and a peak at 2.5 Ga are supported by Wikipedia; the start at 2.65 Ga was not found in the cut-off abstract. Note the secondary peak at 1.85 Ga in Wikipedia is a bump, whereas the code treats 1.85 Ga as the end.
  Evidence level: primary abstract (truncated); secondary source (Wikipedia).

- Great Oxidation Event dates (Earth reference). Source says: "the Great Oxidation Event (GOE), began in early Paleoproterozoic ... and continued until the end of Lomagundi carbon isotope excursion ca. 2,060 Ma ... glaciation and onset of the GOE occurred between ca. 2,460 and 2,426 Ma" (Gumsley et al. 2017, PNAS 114:1811, abstract rebuilt from OpenAlex).
  Result: MATCH for reference (GOE onset 2.46-2.426 Ga, end of the Lomagundi excursion about 2.06 Ga). The code's 2.02 Ga and the 1.85 Ga end of BIF are therefore later than the GOE by 0.4 and 0.6 Gyr, as the code's sources intend.
  Evidence level: primary abstract.

- ORE-027: coded 85 % of sediment-hosted base metals within 200 km of a thick-thin lithosphere step. Source says: "The research demonstrates that 85% of sediment-hosted base metals occur within 200 km of lithospheric thickness transitions" (Hoggard et al., preprint abstract via the OSF API record, https://api.osf.io/v2/preprints/2kjvc/). The journal version was not reachable.
  Result: MATCH
  Evidence level: primary abstract (preprint version).

- ORE-058 (porphyry copper footprint 0.6 km2 and 1-1.5 km tall), ORE-059 (emplacement depth 1.5 km). Source says: SIR 2010-5070-B (John et al. 2010) could not be read: the PDF is 26.3 MB, over the 10 MB fetch limit. Wikipedia, Porphyry copper deposit: "emplaced at depths of approximately 1 to 6 kilometres with vertical thicknesses on average of 2 kilometres".
  Result: COULD NOT READ for the 0.6 km2 median ore area and the 1-1.5 km height. The Wikipedia 2 km average vertical thickness differs from the 1-1.5 km "hypogene ore" height (the two may describe different things: whole system versus ore), unresolved.
  Evidence level: secondary source (Wikipedia) for the vertical figure only.

- ORE-057 (porphyry ore density 2.6 vs 2.7 t/m3). Source says: "The average density of granite is between 2.65 and 2.75 g/cm3" (Wikipedia, Granite, citing EduMine "Rock Types and Specific Gravities"). No porphyry-specific tonnage factor was read.
  Result: PARTIAL. Both 2.6 and 2.7 sit within normal granitic rock densities (2.65-2.75; 2.6 is just below the quoted range). The inconsistency between the two code files stays a code issue, not a source issue.
  Evidence level: secondary source (Wikipedia).

#### E. Placers

- ORE-207 / ORE-212 (gold placer transport 5 km; bake moves a placer to a river within 30 km). Source says: nothing on gold transport distance found. USGS Bulletin 1693 model 39a (read in full) gives: "High-energy alluvial where gradients flatten and river velocities lessen", "Cenozoic. Older deposits may have been formed but their preservation is unlikely", "Highest Au values at base of gravel deposits in various gold 'traps' such as natural riffles in floor of river or stream, fractured bedrock". For other minerals: alluvial tin "Economic placers are generally within a few (<8) kilometers of the primary sources", and diamonds "Alluvial diamond deposits may be 1,000 km from source".
  Result: COULD NOT READ (no citable gold-specific distance found). The 5 km range is not contradicted (tin is under 8 km) but is unsupported for gold; 30 km is a gameplay value. Garnett and Bassett 2005 (paywalled) is the place to look.
  Evidence level: primary full text (Bulletin 1693 HTML) for what it does and does not say.

- ORE-208 (stop where gradient falls below 5 m/km). Source says: the only gradient statement read is model 39a's "where gradients flatten and river velocities lessen" and, for tin, a "critical range for deposition of cassiterite" (no number given). No numeric placer gradient found.
  Result: COULD NOT READ (no numeric gradient in any source reached)
  Evidence level: primary full text (Bulletin 1693 HTML).

- ORE-091 and ORE-219 (one-mile rule; gravel density 2.0 t/m3). Source says: "In most cases, the grade and tonnage figures are for districts or for placer operations within one mile (1.6 km) of one another." and "deposit volume and grades have been converted to metric tons and grams per metric ton using 2.0 metric tons per cubic meter--the average density of wet sand and gravel." (Bulletin 1693 grade-and-tonnage model of placer Au-PGE, https://pubs.usgs.gov/bul/b1693/html/bull42lh.htm). The page cites Orris and Bliss (1985).
  Result: MATCH (both). The page number p. 262 was not checked.
  Evidence level: primary full text.

- ORE-218 / ORE-220 (placer environment, gold at base of gravel, Cenozoic). Source says: see the model 39a quotations above.
  Result: MATCH
  Evidence level: primary full text.

- ORE-221 (cassiterite within a few km; diamonds up to 1,000 km). Source says: see the quotations above ("<8" km and "1,000 km").
  Result: MATCH. The code's attribution to Lehmann 2020 for the tin figure is not supported by anything I read; Bulletin 1693 supports it.
  Evidence level: primary full text.

- ORE-068 (sedex ore bodies a few tens of metres thick). Source says: "sheet- or lens-like tabular ore bodies up to a few tens of meters thick" (Bulletin 1693, descriptive model of sedimentary exhalative Zn-Pb, https://pubs.usgs.gov/bul/b1693/html/bull5nmq.htm); age "Middle Proterozoic (1,700-1,400 m.y.); Cambrian to Carboniferous (530-300 m.y.)"; no fluid temperatures on that page.
  Result: MATCH
  Evidence level: primary full text.

#### F. Petroleum physical values

- ORE-187 (0.1364 t per barrel; 0.858 t/m3). Source says: "1 toe = 7.33 barrels of oil equivalent (boe)", "1 barrel of oil equivalent (boe) contains approximately 0.136 toe" (Wikipedia, Tonne of oil equivalent, citing OPEC Annual Statistical Bulletin 2014 conversion factors). 1 / 7.33 = 0.1364. Implied density: 0.1364 t / 0.158987 m3 per barrel = 0.858 t/m3 (arithmetic).
  Result: MATCH for 0.1364 t/bbl (it is the OPEC factor). The 0.858 is its consequence, not a separate claim.
  Evidence level: secondary source (Wikipedia citing OPEC).

- ORE-188 (crude density 0.85 t/m3; "crude oils run about 0.8-0.9"). Source says: "Light crude: Less than 870 kg/m3; Medium oil: 870 to 920 kg/m3; Heavy crude: 920 to 1000 kg/m3" (Wikipedia, API gravity), and "API gravity values of most petroleum liquids fall between 10 and 70 degrees".
  Result: PARTIAL. 0.85 is a light crude; 0.858 and 0.85 differ by 1 % and both are normal. No source for a "global average" crude density was found, and none is needed.
  Evidence level: secondary source (Wikipedia).

- ORE-189 (20 % porosity). Source says: "The porosity of an oil field, or the percentage of the total volume that contains fluids rather than solid rock, is 20-35% or less." (Wikipedia, Petroleum reservoir). No oil-saturation value found for the 75 % used.
  Result: PARTIAL (porosity 20 % within the quoted range; 75 % oil saturation not sourced)
  Evidence level: secondary source (Wikipedia).

- ORE-165 (minimum source TOC 1 wt%). Source says: "approximately 0.5% total organic carbon by weight percent" as the minimum for an effective source rock; "values of 2% are considered the minimum" for shale-gas reservoirs; "values exceeding 10% exist" (SLB Energy Glossary, Total organic carbon, https://glossary.slb.com/terms/t/total_organic_carbon). A fuller poor/fair/good/excellent scale (Peters and Cassa 1994) was not found in an open page.
  Result: PARTIAL (the glossary puts the minimum at about 0.5 wt%; the code's 1 wt% is twice that, a design choice)
  Evidence level: secondary source (SLB glossary).

- ORE-192 (Dyman 2002: deep gas below 4.5 km; USGS assessments have no fixed depth cut-off). Source says: not read (Springer redirect to login).
  Result: COULD NOT READ (paywall)
  Evidence level: none (metadata only).

- ORE-185 / ORE-186 (Niger Delta AU 71920101 field size 12 MMBO; P90 2.4, P10 89 MMBO). Source says: not read (DDS-60 data forms are PDFs inside a zip).
  Result: COULD NOT READ (PDF unreadable)
  Evidence level: none.

- ORE-195 / ORE-196 / ORE-199 / ORE-200 / ORE-201 (USGS TTI and Ro values in PP 1698, OFR 88-450-G, OFR 94-220, DDS-69-B, OFR 2003-253). Source says: none readable (USGS PDFs binary). OFR 2003-253's abstract (read) places generation and migration "between 100-90 Ma" and has no temperature.
  Result: COULD NOT READ
  Evidence level: none (primary abstract of OFR 2003-253 has no relevant number).

## Moisture, Köppen classes and biomes (MOI, BIO)

Results: MATCH 18, PARTIAL 6, MISMATCH 2, COULD NOT READ 2.

Peel et al. 2007, Table 1, as it reads in the PDF text layer (raw, with the lost symbols shown as double spaces): "A Tropical T cold  18 f - Rainforest P dry  60 m - Monsoon Not (Af) & P dry  100-MAP/25 w - Savannah Not (Af) & P dry < 100-MAP/25 B Arid MAP < 10  P threshold W - Desert MAP < 5  P threshold S - Steppe MAP  5  P threshold h - Hot MAT  18 k - Cold MAT < 18 C Temperate T hot > 10 & 0 < T cold < 18 s - Dry Summer P sdry < 40 & P sdry < P wwet /3 w - Dry Winter P wdry < P swet /10 f - Without dry season Not (Cs) or (Cw) a - Hot Summer T hot  22 b - Warm Summer Not (a) & T mon10  4 c - Cold Summer Not (a or b) & 1  T mon10 < 4 D Cold T hot > 10 & T cold  0 s - Dry Summer ... d - Very Cold Winter Not (a or b) & T cold < -38 E Polar T hot < 10 T - Tundra T hot > 0 F - Frost T hot  0". Notes under the table: "T mon10 = number of months where the temperature is above 10 ... P threshold = varies according to the following rules (if 70% of MAP occurs in winter then P threshold = 2 x MAT, if 70% of MAP occurs in summer then P threshold = 2 x MAT + 28, otherwise P threshold = 2 x MAT + 14). Summer (winter) is defined as the warmer (cooler) six month period of ONDJFM and AMJJAS." The symbols lost in the text layer are read here as greater-or-equal in the A, Af, BS, BSh, a, b and Tmon10 rows and less-or-equal in the D and F rows, confirmed against the author's page (https://people.eng.unimelb.edu.au/mpeel/koppen.html: "Tcold >= 18", "Thot >= 22", "Tcold <= 0") and Beck et al. 2018 Table 2 via PMC ("Tcold>=18", "Thot>10 & 0<Tcold<18", "Tcold<=0").

- BIO-002: coded E tested first ("thot < 10 degC checked first, ahead of B", "so cold deserts are polar"). Source says: "Since all locations that satisfy the B climate criteria will also satisfy one of the other (A, C, D or E) climate criteria, the B climates must be identified first." (Peel et al. 2007, section on Table 1.) Beck et al. 2018 Table 2 prints the A, C, D and E rows with "Not (B) &" in front.
  Result: MISMATCH. The code's stated choice contradicts the cited source. In Peel a cell with a warmest month below 10 degC that also passes the B test (MAP < 10 x P_th) is BW or BS; the code calls it ET or EF. By arithmetic from the B rule this can happen only when P_th is positive: P_th = 2 x MAT needs MAT above 0 degC (winter-dominant rain), 2 x MAT + 14 needs MAT above -7 degC (even rain), 2 x MAT + 28 needs MAT above -14 degC (summer-dominant rain). With little rain (e.g. MAT 2 degC and even rain: P_th = 18, so MAP < 180 mm) the cell is B in Peel and polar in the code: cold deserts such as high plateaus.
  Evidence level: primary full text

- BIO-003: coded ET if the warmest month is above 0 degC, else EF. Source says: "E Polar T hot < 10; T - Tundra T hot > 0; F - Frost T hot  0" (Table 1, glyph lost; the author's page gives ET "Thot > 0" and EF "Thot <= 0").
  Result: MATCH.
  Evidence level: primary full text

- BIO-004: coded P_th = 2 MAT + 28 for at least 70 percent of rain in the summer half, + 14 otherwise, + 0 for winter (winter tested first; code uses "pw >= 0.7 MAP"). Source says: "if 70% of MAP occurs in winter then P threshold = 2 x MAT, if 70% of MAP occurs in summer then P threshold = 2 x MAT + 28, otherwise P threshold = 2 x MAT + 14" (Table 1 note).
  Result: MATCH (the paper does not say whether exactly 70 percent counts; the code takes it as at least).
  Evidence level: primary full text

- BIO-005: coded B if MAP < 10 x P_th; BW if MAP < 5 x P_th, else BS. Source says: "B Arid MAP < 10 x P threshold; W - Desert MAP < 5 x P threshold; S - Steppe MAP  5 x P threshold" (Table 1). The Wikipedia/Koppen form: threshold = 20 x MAT + 280/140/0, BW below 50 percent of it, which is the same numbers.
  Result: MATCH.
  Evidence level: primary full text

- BIO-006 and BIO-007: coded h if MAT >= 18 degC else k; A if the coldest month >= 18 degC. Source says: "h - Hot MAT  18; k - Cold MAT < 18" and "A Tropical T cold  18" (glyphs reconstructed from the author's page: "MAT >= 18", "Tcold >= 18").
  Result: MATCH.
  Evidence level: primary full text

- BIO-008, BIO-009, BIO-010: coded Af if the driest month has 60 mm or more; Am if at least 100 - MAP/25; otherwise Aw (code adds As when the driest month is in summer). Source says: "f - Rainforest P dry  60 m - Monsoon Not (Af) & P dry  100-MAP/25 w - Savannah Not (Af) & P dry < 100-MAP/25" (Table 1).
  Result: MATCH for Af, Am and Aw. Peel's table has no As, so BIO-010 is the project's extension, as the ledger says ("uncited").
  Evidence level: primary full text

- BIO-011: coded C if the coldest month is above 0 degC, else D. Source says: "C Temperate T hot > 10 & 0 < T cold < 18 ... D Cold T hot > 10 & T cold  0" (Table 1); the author's page prints "Tcold <= 0" for D. The research note's "or -3 degC" is in Wikipedia, not in this paper.
  Result: MATCH (0 degC, as Peel). The -3 degC option is not part of Peel's scheme; the ledger flag that "-3 is not implemented" is moot for Peel.
  Evidence level: primary full text

- BIO-012: coded s if the driest summer month is under 40 mm and under a third of the wettest winter month. Source says: "s - Dry Summer P sdry < 40 & P sdry < P wwet /3" (Table 1).
  Result: MATCH.
  Evidence level: primary full text

- BIO-013: coded w if the driest winter month is under a tenth of the wettest summer month, tested after s (so a cell that passes both gets s). Source says: "w - Dry Winter P wdry < P swet /10" and, in the text: "When a location satisfied both Cs and Cw criteria, the precipitation for the six months containing summer and the six months containing winter were compared, with Cw assigned if the summer precipitation was greater than the winter precipitation. This additional rule was also applied to locations that satisfied both the Ds and Dw climate criteria as well."
  Result: PARTIAL. The threshold matches; the order does not. Peel assigns w (not s) when both pass and summer is the wetter half; the code always gives s. Cells that pass both are mostly in the wet-summer subtropics (the paper's example is Herberton, north Queensland, over 70 percent of rain in summer, called Cw).
  Evidence level: primary full text

- BIO-014 and BIO-015: coded a if the warmest month >= 22 degC; b if at least four months >= 10 degC. Source says: "a - Hot Summer T hot  22 b - Warm Summer Not (a) & T mon10  4" with "T mon10 = number of months where the temperature is above 10" (Table 1, note).
  Result: MATCH on the 22 degC and 4 months. Boundary note: Peel counts months strictly above 10 degC; the code counts months at or above 10 (also Thot of exactly 10 is neither E nor C/D in Peel, since E is "< 10" and C/D "> 10"; the code puts it in C/D). Only an exact tie differs.
  Evidence level: primary full text

- BIO-016: coded d (D only) if the coldest month is below -38 degC, tested after a and b (so Dxa and Dxb never become d); exactly -38 gives c. Source says: "c - Cold Summer Not (a, b or d)" and "d - Very Cold Winter Not (a or b) & T cold < -38" (Table 1, D section).
  Result: MATCH. This resolves earth flag 2 (the -38 boundary): Peel's d is "< -38", so exactly -38 is c, as coded; the redundant `tcold > 0.0 ||` clause changes nothing.
  Evidence level: primary full text

- BIO-017: coded summer half = whichever of April-September and October-March is warmer. Source says: "Summer (winter) is defined as the warmer (cooler) six month period of ONDJFM and AMJJAS." (Table 1 note).
  Result: MATCH. This refutes earth flag 5 (the suspicion that Peel defines summer by hemisphere): Peel defines it by the warmer half-year, exactly as the code does.
  Evidence level: primary full text

- STATUS P1-17 Earth shares (earth/MOI.md): coded "Peel: B 30.2 %, D 24.6 %, A 19.0 %, C 13.4 %, E 12.8 %". Source says: "Globally the dominant climate class by land area is arid B (30.2%) followed by cold D (24.6%), tropical A (19.0%), temperate C (13.4%) and polar E (12.8%). The most common individual climate type by land area is BWh (14.2%), followed by Aw (11.5%)." (Peel et al. 2007, discussion and conclusion.)
  Result: MATCH.
  Evidence level: primary full text

- BIO-038: coded anchors 0.03, 0.2, 0.5, 1, 2, 5 to bands 0 to 5, "following UNEP's aridity classes". Source says (Zomer et al. 2022, citing "UNEP. World atlas of desertification - Second Edition (United Nations Environment Program, 1997)"): "<0.03 Hyper Arid; 0.03-0.2 Arid; 0.2-0.5 Semi-Arid; 0.5-0.65 Dry sub-humid; >0.65 Humid", with "AI = MA_Prec/MA_ET0". Wikipedia's aridity index article gives the 1992 limits: hyperarid below 0.05, arid 0.05-0.20, semi-arid 0.20-0.50, dry subhumid 0.50-0.65.
  Result: PARTIAL. 0.03, 0.2 and 0.5 are UNEP 1997 boundaries (earth flag 6's suspicion that UNEP uses 0.05 is the 1992 edition; 0.03 is the 1997 value). UNEP has no boundaries at 1, 2 and 5: its humid class is everything above 0.65. The anchors 1, 2, 5 are the project's own extension, and UNEP's index uses a reference (Penman-Monteith) evapotranspiration, where the code uses Thornthwaite PET.
  Evidence level: secondary source (Zomer et al. 2022 via PMC summariser; Wikipedia)

- BIO-095: coded precipitation seasonality = 100 x sqrt(sum((p - mean)^2)/11)/mean ("WorldClim's BIO15"). Source says: "BIO15 = Precipitation Seasonality (Coefficient of Variation)" (worldclim.org/data/bioclim.html); the page gives no formula and no statement on n versus n-1.
  Result: PARTIAL. The variable is named as in the code; the formula (sample standard deviation, n-1 = 11) could not be checked on the pages read.
  Evidence level: secondary source (WorldClim variable list)

- MOI-034: coded P_hat(k,l) = c_w i sigma h_hat / ((1 - i m H_w)(1 + i sigma tau_c)(1 + i sigma tau_f)); sigma = U k + V l; m = sign(sigma) sqrt(abs((N_m^2 - sigma^2)/(sigma^2 - f^2)) (k^2 + l^2)); numerator clipped at 0, denominator kept at least epsilon from 0; x 3600 to mm/h. Source says (fastscape-lem orographic_precipitation.py): "sigma = kx * u0 + ky * v0", "mf_num = param['nm'] ** 2 - sigma ** 2", "mf_den = sigma ** 2 - f_coriolis ** 2", "mf_num[mf_num < 0] = 0.", "mf_den[(mf_den < EPS) & (mf_den >= 0)] = EPS", "m = sign * np.sqrt(np.abs(mf_num / mf_den * (kx ** 2 + ky ** 2)))", "P_karot = ((param['cw'] * 1j * sigma * hhat) / ((1 - (param['hw'] * m * 1j)) * (1 + (sigma * param['conv_time'] * 1j)) * (1 + (sigma * param['fall_time'] * 1j))))", "P *= 3600  # mm hr-1" (https://raw.githubusercontent.com/fastscape-lem/orographic-precipitation/master/orographic_precipitation/orographic_precipitation.py).
  Result: MATCH between the code's formula and the software it ports (this is a check against the Python code, not against the paper).
  Evidence level: primary full text (software source)

- MOI-034 (paper): coded "Smith & Barstad 2004, eq. 49". Source says: not read (the AMS page returned 403; only the abstract via OpenAlex).
  Result: COULD NOT READ (journal page blocked). Stage 2 should read the paper to confirm the transfer function is equation 49 and the units of c_w.
  Evidence level: primary abstract

- MOI-035: coded "the Python module swaps the lengths on non-square rasters". Source says (same file): "x_n_value = np.fft.fftfreq(ny, (1. / ny))", "y_n_value = np.fft.fftfreq(nx, (1. / nx))", "x_len = nx * dx", "y_len = ny * dy", "kx_line = 2 * np.pi * x_n_value / x_len", "ky_line = 2 * np.pi * y_n_value / y_len": the x frequencies run over the ny samples of axis 1 but are divided by nx x dx.
  Result: MATCH (the claim is correct for non-square arrays, where nx != ny).
  Evidence level: primary full text (software source)

- MOI-036 to MOI-040: coded conv time 1000 s, fall time 1000 s, N_m 0.01 1/s, H_w 3400 m, c_w = 0.01295 = 7.4e-3 x 7/4. Source says (fastscape_ext.py): "lapse_rate: environmental lapse rate, default=-4 degrees Celsius/km", "lapse_rate_m: moist adiabatic lapse rate, default=-7", "ref_density: reference saturation water vapor density, default=7.4e-3 kg/m^3", "hw: water vapor scale height, default=3400 m", "nm: moist stability frequency, default=0.01 1/s", "conv_time: default=1000 s", "fall_time: default=1000 s", and "self.cw = self.ref_density * self.lapse_rate_m / self.lapse_rate".
  Result: MATCH on all five. The "7/4" has an origin: lapse_rate_m / lapse_rate = -7 / -4 = 1.75 and 7.4e-3 x 1.75 = 0.01295. This refutes earth flag 13's suspicion (nm 0.005 and a 6.5/5.8 ratio were the author's recollection and are wrong for this package).
  Evidence level: primary full text (software source)

- MOI-043: coded oro_hours_per_day = 0.5, "fastscape's rainfall_frequency". Source says (fastscape_ext.py): "rainfall_frequency: daily rainfall frequency, default=1, units 1/day", described as "number of storms of 1 hour duration per day", converting precipitation from mm m^2/h to m^3/yr with the factor 8.76 x rainfall_frequency.
  Result: PARTIAL. The mechanism (an event rate times hours of rain per day) is the package's rainfall_frequency, but the package default is 1, not 0.5; the project's 0.5 halves it. The ledger notes it as unsourced; it is the project's choice.
  Evidence level: primary full text (software source)

- MOI-051: coded 6.5 degC/km ("the standard atmosphere's"). Source says: Wikipedia, Lapse rate: "temperature lapse rate of 6.50 degC/km (3.56 degF or 1.98 degC/1,000 ft) from sea level to 11 km" (citing ICAO 1993). Observed near-surface values: Minder et al. 2010 abstract: "On the windward side of the range, the various data sources reveal annual mean lapse rates of 3.9-5.2 degC km-1, substantially smaller than the often-assumed 6.5 degC km-1. ... the smallest lapse rates (2.5-3.5 degC km-1) in late-summer minimum temperatures, and largest (6.5-7.5 degC km-1) in spring maximum temperatures." Rolland 2003 abstract: "Yearly lapse rates ranged from -0.54 to -0.58 degC per 100 meters" in the Alps (Italy, Tyrol).
  Result: MATCH for 6.5 as the ICAO standard-atmosphere value; PARTIAL as a description of the real near-surface lapse rate: observed annual means are 3.9 to 5.8 degC/km in the two regions read, and the Worldbuilding Pasta page does say "temperature dropping by 4.46 degC per kilometer" (the value the research note quoted and the code does not use).
  Evidence level: secondary source for the ICAO value (Wikipedia, summariser); primary abstract for Minder and Rolland

- MOI-065: coded Thornthwaite: heat = sum((T/5)^1.514, T > 0); a = 6.75e-7 heat^3 - 7.71e-5 heat^2 + 1.792e-2 heat + 0.49239; PET_m = 16 (10 T/heat)^a; above 26.5 degC -415.85 + 32.24 T - 0.43 T^2. Source says (R package SPEI, thornthwaite.R): "J <- apply(Tt, c(2, 3), function(x) { sum((x / 5)^1.514, na.rm = na.rm) })", "q <- 0.000000675 * J3 - 0.0000771 * J2 + 0.01792 * J + 0.49239", "PET <- K * 16 * (10 * Tave / J)^q", "Tave[Tave < 0] <- 0", "K <- N / 12 * mlen_array / 30" (https://raw.githubusercontent.com/sbegueria/SPEI/master/R/thornthwaite.R).
  Result: PARTIAL. The heat index, exponent polynomial and 16 (10 T/I)^a form match an independent implementation; K = 1 is the code's "uncorrected for day length" (K is the day-length and month-length correction). The branch above 26.5 degC is not in that code, and I did not find it in a source: COULD NOT READ for that part.
  Evidence level: secondary source (SPEI R package source)

- MOI-064: coded precipitation x exp(0.02 (T - 15)), "Held & Soden 2006 (about 2 percent per kelvin)". Source says: "the sensitivity is on the order of 2% K-1 (with a median value of 1.7% K-1), much weaker than CC scaling", "The dashed line in (d) depicts the linear fit of P to T, which increases at a rate of 2.2% K-1", "Solid lines depict the rate of increase in column-integrated water vapor (7.5% K-1)", "The global-mean increase in evaporation is 2% K-1 so the resulting expression for the precipitation response per unit global warming is  P  0.07  T  P  E   0.02 E  (7)" (Held & Soden 2006, text of the PDF; Figure 1 caption and equation 7).
  Result: MATCH for 2 percent per kelvin (fit 2.2, median 1.7 in the models they used). MOI-042 and MOI-063 (0.067 to 0.07 per kelvin, "Clausius-Clapeyron"): the paper's water-vapour rate is 7.5 percent per kelvin and equation 7 uses 0.07; the clamp 0.05 to 3.0 on the orographic factor is the project's.
  Evidence level: primary full text

- MOI-073: coded runoff rises about 3.8 percent per kelvin, "Berner & Kothavala 2001 (GEOCARB)"; params.rs also Berner 1994. Source says: "For the standard case Ws 5 7.4, G 5 4.0, RUN 5 0.045 for colder periods (340-260 Ma and 40-0 Ma) and Ws 5 7.4, G 5 3.3, RUN 5 0.025 for the warmer rest of the Phanerozoic. Values for the same parameters from GEOCARB II are Ws 5 12.9, G 5 6.0, and RUN 5 0.038 for all times." and "the results of Manabe and Bryan (1985) for runoff (RUN 5 0.038 for all time), as used by GEOCARB II" (Berner & Kothavala 2001; the "5" is the extraction's rendering of "=", and the degree symbol was lost).
  Result: MISMATCH. 0.038 per kelvin is GEOCARB II (Berner 1994, after Manabe & Bryan 1985), not Berner & Kothavala 2001: GEOCARB III uses 0.045 per kelvin for the cold periods and 0.025 for the warm rest. The 2001 paper cannot be the source of 0.038; the one-line Berner 1994 citation in params.rs is the correct one for the number.
  Evidence level: primary full text

- MOI-001 (belts at 30 and 60 degrees): coded cell edges at 30 and 60 degrees. Source says: NOAA JetStream: "The high pressure band is located at about 30 N/S latitude and at each pole. Low pressure bands are found at the equator and 50-60 N/S." and the Ferrel cell "between approximately 35 and 60 N/S"; Wikipedia, Hadley cell: "the descending branch of the Hadley cell is located roughly over the 25th parallel north and the 25th parallel south" and the subtropical highs "lie primarily over the ocean between 20 and 40 latitude".
  Result: PARTIAL. 30 degrees for the subtropical high and 50 to 60 for the subpolar low are within the textbook range; the Hadley edge is quoted from 25 to 30. The code's round numbers are a simplification, not a measurement.
  Evidence level: secondary source (NOAA, Wikipedia; summariser)

- MOI-025 and the Worldbuilding Pasta rows (MOI-009, MOI-014, MOI-017 to MOI-021, MOI-046, MOI-048): coded "rain reaches about 2,000 km inland" (note), gyre offsets, subtropical highs at 25 degrees, orographic limits. Source says (blog, summariser): "Rainfall from warm coastal currents can extend approximately 2,000 km downwind before dissipating"; warm currents produce "roughly 15 degC warmer on western coasts, while cold equatorward currents create approximately 10 degC cooler conditions on eastern coasts"; "Subtropical high-pressure zones ... appearing around 25 latitude in winter and 35 latitude in summer"; "relief exceeds 1 km" and "Windward slopes peak in rainfall around 1-1.5 km elevation; areas above 4 km remain generally dry".
  Result: MATCH to the blog for the 2,000 km, the 15 and 10 degC gyre figures and the orographic statements (the code's gyre amplitudes of 3 to 6 degC are smaller, as earth flag 12 says). PARTIAL for MOI-014: the blog places the subtropical highs at 25 (winter) to 35 (summer) degrees; the code centres a fixed bump at 25. All of this is a blog, not measurements.
  Evidence level: secondary source (a blog, via summariser)

- BIO-083 (Ohmura rule): coded ELA fit P = 645 + 296 T + 9 T^2. Source says: "P = a + bT + cT2, whereby a = 645, b = 296 and c = 9, and P and T are in mm w.e. and degC, respectively." (Ohmura, Kasser & Funk 1992, Cambridge Core page, summariser; my prompt contained the equation, so a second look is advised.)
  Result: MATCH (as returned). The standard error is given as 200 mm w.e. (summariser).
  Evidence level: primary (page text via summariser)

- MOI-068: coded Fu's Budyko curve, omega = 2.6 ("Fu 1981; Zhang et al. 2004").
  Result: COULD NOT READ (Fu 1981 not found; Zhang 2004 only the abstract, which does not mention omega). See HYD-034.
  Evidence level: none

---

## Scale, provinces, pole cap and bake (SCL, PRV, POL, BAK)

Results: MATCH 9, PARTIAL 15, COULD NOT READ 4.

Earth shares of province classes (the first priority)
- Earth shares (earth/SCL.md; STATUS P1-10 row): coded as Earth reference cratons 0.71, orogen about 0.17 (12 % orogens + 5 % arcs), extended+rift 0.07. Source says: CRUST 2.0 publishes no province area table; the type key (CNtype2_key.txt) names 360 profiles by setting, e.g. "Archean", "early/mid Proter. shield", "late Proter.", "Platform", "Phanerozoic", "orogen/...", "continental arc", "island arc", "extended crust", "Rift", "continental shelf", "continental slope" (http://igpppublic.ucsd.edu/~gabi/ftp/crust2/CNtype2_key.txt, CNtype2.txt). I downloaded both files and classified type names by keyword (my own mapping: oceanic/melt-affected/inactive ridge = ocean; shelf and slope excluded; island arc and forearc = island arc; orogen and continental arc = orogen; extended, rift, thinned = extended; Archean, Proterozoic, shield = Precambrian; Platform, Phanerozoic, Caspian = platform), weighted each 2-degree cell by cos(latitude), dropped the rows with top edge below 60 S (Antarctica), kept Greenland. Result: continental area (shelf and slope excluded) = 30.5 % of the sphere; of that continental area, Precambrian 50.9 %, platform+Phanerozoic 24.2 % (cratons 75.1 %), orogen incl. continental arcs 14.4 %, island arc/forearc 3.7 % (orogen + arcs 18.1 %), extended + rift 6.6 %, other 0.2 %.
  Result: PARTIAL. Direction and size agree with the sub-agent's figures (cratons 0.71 vs 0.75 here, orogens+arcs 0.17 vs 0.18, extended 0.07 vs 0.066), so the build log's "orogen about 0.17 vs mod 0.40" conclusion stands. But it is not a published number: it depends on a type-name mapping (my own, crude; the generic "Phanerozoic" types might be partly orogenic), and the "derived by a sub-agent" shares in the repo have no method on record to compare against. Run files: scratchpad `crustwork/run.zsh`.
  Evidence level: primary data (CRUST 2.0 files), own classification.
- PRV-001 (classes): coded nine classes after the USGS map. Source says (Commons file description): "Map of world geologic provinces" ... "shields, platforms, Phanerozoic orogenic belts, foreland basins, large igneous provinces, extended continental crust, and oceanic crust ages" (https://commons.wikimedia.org/api/rest_v1/page/html/File%3AWorld_geologic_provinces.jpg).
  Result: PARTIAL. Class names match. The map's orogens are Phanerozoic and its basins are "foreland or intracratonic"; the code's orogen rule also includes active arcs and recent collisions and its basin rule is a sediment-cover test; the USGS rules for assigning cells are not in anything I could read.
  Evidence level: secondary source (Wikimedia Commons file description; Wikipedia "Craton" caption)
- PRV-002 / PRV-003 / PRV-004: coded 20 Ma and 65 Ma. Source says (Wikipedia "Craton", figure legend as returned by the fetch): "Oceanic crust: 0-20 Ma, 20-65 Ma, >65 Ma" (https://en.wikipedia.org/wiki/Craton).
  Result: MATCH
  Evidence level: secondary source (Wikipedia; the legend image itself not read)
- PRV-021: coded 150 km = 1.35 degrees. Source says: "This geologic province map renders only features approximately 150 km across and greater due to the fact that the resolution of the maps should be consistent with the resolution of the seismic refraction data." (Commons file description, same URL as above). 150 km / 111.2 km per degree = 1.35 degrees (my arithmetic).
  Result: MATCH
  Evidence level: secondary source (Wikimedia Commons)
- PRV-010 to PRV-020 (rules, 500 Myr, 3 km cover, etc.): USGS class definitions and age cut-offs for continental classes: COULD NOT READ (the USGS page returned 404; the archive.org copy is blocked for the tool; no definitions in Commons or Wikipedia). Result: COULD NOT READ. Evidence level: none.

Fibonacci and Cortial
- SCL-044 (point set): coded z = 1 - (2i+1)/n, longitude 2 pi frac(i/golden ratio). Source says: nothing read; the only open copy found (arXiv:0912.4540) returned binary; Keinert and Swinbank & Purser were not reachable. arXiv abstract of Gonzalez: "The area of a spherical region can be easily measured by considering which sampling points of a lattice are located inside or outside the region."
  Result: COULD NOT READ (equation); references confirmed to exist with correct titles. 
  Evidence level: primary abstract (Gonzalez only)
- SCL-043 (spacing): coded sqrt(4 pi / n) = 0.0112 rad (n = 100,000), 71 km on 6,370 km. Source says: not a sourced value; my arithmetic gives sqrt(4 pi / 1e5) = 0.01121, x 6370 = 71.4 km; n = 250,000 gives 0.00709, 45.2 km; x 782 km = 8.8 km.
  Result: MATCH (internal arithmetic only)
  Evidence level: arithmetic, not a source
- SCL-029 (R = 6,370 km) and SCL-034 (delta t = 2 Myr): Source says: not read (author PDF 10 MB+ limit, Wiley 403).
  Result: COULD NOT READ
  Evidence level: none (the repo note's own quote "Constants (Appendix A): time step delta t = 2 My; planet radius R = 6,370 km" is the project's, not read here)

Plate size split and ore reach
- SCL-030 (Mallard, split near 10^7.6 km^2, about 40 M km^2): Source says (abstract, Europe PMC): "The theory of plate tectonics describes how the surface of Earth is split into an organized jigsaw of seven large plates of similar sizes and a population of smaller plates whose areas follow a fractal distribution." 10^7.6 km^2 = 39.8 M km^2 (my arithmetic).
  Result: PARTIAL (the paper does describe a large/small split; the 10^7.6 value is not in the abstract and the body was unreachable)
  Evidence level: primary abstract
- SCL-006 (Kesler & Wilkinson, 3.3 km): Source says (abstract): "Model results indicate that approximately 125,895 porphyry copper deposits were formed during Phanerozoic time, with approximately 47,789 remaining at various crustal depths, containing approximately 1.7 x 10^11 tonnes of copper." No depth limit or mining reach in the abstract.
  Result: PARTIAL (paper exists and is about porphyry copper preserved at depth; "3.3 km within mining reach" not found in the abstract and body not read)
  Evidence level: primary abstract (via OpenAlex)
- SCL-041 (Coltice, Pangea-like start): Source says: "The initial continental configuration is designed to resemble Pangea, located above the Atlantic LLSVP" (Coltice et al. 2019, via Europe PMC full-text XML; https://www.ebi.ac.uk/europepmc/webservices/rest/PMC6821462/fullTextXML).
  Result: MATCH (assuming the report meant this paper)
  Evidence level: primary full text (keyword search only)

Solar law
- SCL-039 (L = L0 / (1 + 0.4 (1 - age/present age))): Source says: not read. The ADS scan of the paper is image-only; the Springer page needs a cookie handshake; Wikipedia pages "Faint young Sun paradox", "Sun" and "Solar luminosity" do not state the formula. Reference check only: title, journal, volume, pages match Crossref.
  Result: COULD NOT READ
  Evidence level: none for the equation
- SCL-039 (present_age_gyr 4.57): Source says (Wikipedia "Age of Earth"): "The age of Earth is estimated to be 4.54 +- 0.05 billion years" and "Calcium-aluminium-rich inclusions (CAIs), the oldest known solid matter that formed within the Solar System, are 4.5673 +- 0.00016 billion years old".
  Result: PARTIAL. 4.57 matches the solar-system (CAI) age; Earth itself is 4.54 +- 0.05. The code's text says "the Sun: 4.57", which is consistent; if the ledger or docs call it the planet's age, 4.54 is the figure.
  Evidence level: secondary source (Wikipedia)

Earth constants (NASA Earth Fact Sheet, https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html, updated 15 Nov 2024)
- SCL-029 / radius: coded 6370.0 km. Source says: "Volumetric mean radius (km) 6371.000"; "Equatorial radius (km) 6378.137"; "Polar radius (km) 6356.752".
  Result: PARTIAL (6370 is 1 km, 0.016 %, below the volumetric mean; it is Cortial's number)
  Evidence level: primary full text
- SCL-022 (gravity): coded 9.81 m/s^2. Source says: "Surface gravity (mean) (m/s2) 9.820"; "Surface acceleration (eq) 9.780"; "(pole) 9.832". Wikipedia infobox gives standard gravity 9.80665.
  Result: PARTIAL (9.81 lies between the standard value and the NASA mean; differences 0.01 m/s^2)
  Evidence level: primary full text (NASA), secondary (Wikipedia)
- SCL-023 (day): coded 24.0 h. Source says: "Length of day (hrs) 24.0000".
  Result: MATCH
  Evidence level: primary full text
- SCL-024 (sidereal factor): coded omega = TAU / (day_h * 3600) * (1 + 1/365.25). Source says: "Sidereal rotation period (hrs) 23.9345". My arithmetic: 24 h / (1 + 1/365.25) = 23.9345 h.
  Result: MATCH
  Evidence level: primary full text plus arithmetic
- SCL-025 (axial tilt): coded 23.4 degrees. Source says: "Obliquity to orbit (deg) 23.44" (NASA); Wikipedia infobox "23.4392811".
  Result: PARTIAL (rounded; 0.04 degree below the NASA figure)
  Evidence level: primary full text
- SCL-027 (ocean fraction): coded 0.7 (editor hint "about 0.71"; build log land 0.289). Source says: "About 71 percent of the Earth's surface is water-covered" (USGS Water Science School, https://www.usgs.gov/water-science-school/science/how-much-water-there-earth). Wikipedia infobox: surface area 510,072,000 km^2, water 361,132,000 km^2 (70.8 %), land 148,940,000 km^2 (29.2 %), citing "Pidwirny 2006".
  Result: PARTIAL (0.70 coded vs 0.708 to 0.71; the build log's Earth land fraction 0.289 vs 0.292 here; both are within about 0.003, but they come from different datasets and the repo cites none)
  Evidence level: secondary source (USGS web page, Wikipedia)
- Earth circumference (task brief 40,075 km) and surface area (510 M km^2): Source says (Wikipedia "Earth"): "40075.017 km equatorial", "40007.863 km meridional", "510072000 km2". The NASA sheet has neither. 2 pi x 6371.000 = 40,030 km (my arithmetic), so 40,075 is the equatorial value, not the mean-radius value.
  Result: MATCH (equatorial; note which one is meant)
  Evidence level: secondary source (Wikipedia)
- Earth age 4.54 or 4.57 Gyr: see SCL-039 above (secondary, Wikipedia "Age of Earth").
- SCL-005, SCL-030 (scale arithmetic): coded radius_m = C x 300 / 2 pi; scale factor 0.123 and 0.246. My arithmetic: 16384 x 300 / 2 pi = 782.2 km; 32768 gives 1564.5 km; 782.2 / 6370 = 0.1228; 4 pi x 782.2^2 = 7.69 M km^2; 4 pi x 1564.5^2 = 30.76 M km^2.
  Result: MATCH (internal arithmetic)
  Evidence level: arithmetic
- POL-017 (secant stretch 2.0 at 60 degrees, 2.9 at 70, 3.9 at 75, 5.8 at 80, 11.5 at 85): my arithmetic 1/cos gives 2.00, 2.92, 3.86, 5.76, 11.47.
  Result: MATCH (internal arithmetic)
  Evidence level: arithmetic

Satellite view thresholds
- BAK-020 (Koppen dryness threshold 20 T + 140 mm): Source says (Wikipedia "Koppen climate classification"): the threshold is "multiplying average annual temperature in Celsius by 20, then adding: 280 if 70% or more of the total precipitation is in the spring and summer months; 140 if 30%-70% ...; 0 if less than 30%"; "If the annual precipitation is less than 50% of this threshold, the classification is BW ...; if it is in the range of 50%-100% of the threshold, the classification is BS" (https://en.wikipedia.org/wiki/K%C3%B6ppen_climate_classification).
  Result: PARTIAL. The coded formula is the middle (no strong seasonality) case applied everywhere; the half and all of the threshold as desert and steppe limits match. Peel et al. 2007 Table 1 not read.
  Evidence level: secondary source (Wikipedia)
- BAK-020 (UNEP limits 0.2 and 0.5): Source says (Wikipedia "Aridity index", citing UNEP 1992): "Arid 0.05 < AI < 0.20; Semi-arid 0.20 < AI < 0.50; Dry subhumid 0.50 < AI < 0.65" (https://en.wikipedia.org/wiki/Aridity_index). The coded stand-in AI = 0.42 P / (20 T + 140) gives 0.21 at half the threshold and 0.42 at the whole threshold (my arithmetic).
  Result: PARTIAL (UNEP values confirmed; the stand-in hits 0.2 but not 0.5, 0.42 instead; confirms flag 5 in earth/SCL.md)
  Evidence level: secondary source (Wikipedia)
- BAK-021 (treeline about 10 degC; herbs over about 0 degC): Source says: Koppen polar group E: "average temperatures below 10 degC in every month of the year", ET warmest month "between 0 degC and 10 degC", EF "all 12 months ... below 0 degC" (Wikipedia, Koppen). Tree line per Koerner & Paulsen via Wikipedia "Tree line": "approximately 6 degC" seasonal mean, with seasonal mean "taken over all days whose mean temperature is above 0.9 degC" (https://en.wikipedia.org/wiki/Tree_line).
  Result: PARTIAL (the 10 degC warmest-month rule is the Koppen convention and the code's tree ramp 8.5-12.5 centres on it; the ecological treeline study gives about 6 degC for a growing-season mean, a different quantity; the 0 degC herb limit equals the ET/EF boundary and the code's ramp -1 to 8 is the agent's)
  Evidence level: secondary source (Wikipedia)
- BAK-025 (rainforest over about 18 degC coldest month): Source says: Koppen group A: "all 12 months of the year have average temperatures of 18 degC or higher".
  Result: PARTIAL (18 degC confirmed; the code's smoothstep 14 to 19 degC is a wider ramp; boreal -10 degC and temperate 0 degC have no standard; Koppen D uses coldest month "below 0 degC (or -3 degC)")
  Evidence level: secondary source (Wikipedia)
- BAK-027 (perennial snow where warmest month is below freezing): Source says: Koppen EF is the case where "all 12 months of the year with average temperatures below 0 degC". Snow line (Wikipedia "Snow line"): "The permanent snow line is the level above which snow will lie all year"; equatorial regions "approximately 4,500 metres"; Alps "just below 3,000 metres"; polar regions sea level.
  Result: PARTIAL (the 0 degC criterion is Koppen's; the code's smoothstep(1, -3) is centred at -1 degC; the snow line itself comes from the climate stage's ela_m, no fixed altitude to compare)
  Evidence level: secondary source (Wikipedia)
- BAK-017 (shelf colour at 120 m): Source says (Wikipedia "Continental shelf"): "the shelf break is located at a remarkably uniform depth of roughly 140 m"; shelf "generally limited to water shallower than 100 m".
  Result: PARTIAL (a colour stop, not a physical claim; roughly 20 m shallower than the quoted shelf break)
  Evidence level: secondary source (Wikipedia)

## Plate tectonics (PLT)

Results: MATCH 28, PARTIAL 14, MISMATCH 1, COULD NOT READ 10.

Cortial constants (evidence: primary full text, https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf, downloaded and text-extracted). Appendix A is headed "Constants: The model is completed by a set of global parameters, with the following values. The sea level was set to 0 as the reference elevation." and the table is captioned "Table 3: Overview of the main parameters for our synthetic planet generation algorithm; the values reported in the table were used for generating the examples showed throughout the article." The code defaults are in `crates/deeptime-core/src/params.rs` lines 560-579 and I checked them directly in W. The minus signs are not rendered by my extraction; they show as a missing glyph. z_r, z_a, z_t each carry that glyph and z_c does not, so I read them as negative (-1, -6, -10 km) and +10 km; the exponent signs read the same way (10^-5, 10^-2, 10^-1). Treat the signs as inferred from the glyph pattern.

- PLT-001: coded 2.0 Myr. Source says: "dt Time-step 2My" (Table 3), and section 4: "a discrete time-step dt set to 2My in our implementation". Result: MATCH. Evidence level: primary full text.
- PLT-002: coded 6370 km. Source says: "R Planet radius 6370km" (Table 3). Result: MATCH. Evidence level: primary full text.
- PLT-092: coded ridge_elevation_m -1000 (default); quick preset -2500. Source says: "zr Highest oceanic ridge elevation -1km" (Table 3). Result: MATCH for the default; the preset's -2500 is a stated departure, with the Parsons & Sclater intercept as its justification (see the PS77 entry below). Evidence level: primary full text.
- PLT-093: coded abyssal_elevation_m -6000 (unused by the model). Source says: "za Abyssal plains elevation -6km". Result: MATCH. Evidence level: primary full text.
- PLT-052: coded trench_elevation_m -10000. Source says: "zt Oceanic trench elevation -10km". Result: MATCH. Evidence level: primary full text.
- PLT-040: coded uplift_reference_m 10000. Source says: "zc Highest continental altitude 10km". Result: MATCH. Evidence level: primary full text.
- PLT-034: coded subduction_distance_km 1800 (default), 800 in the pipeline presets. Source says: "rs Subduction distance 1800km". Result: MATCH for the default; the 800 is a calibration departure that the paper does not support or contradict. Evidence level: primary full text.
- PLT-069: coded collision_distance_km 4200 (default), 2000 in presets. Source says: "rc Collision distance 4200km". Result: MATCH for the default; 2000 is a departure. Evidence level: primary full text.
- PLT-070: coded 1.3e-5 per km. Source says: "Δc Collision coefficient 1.3 × 10^-5 km^-1" (Table 3). Result: MATCH. Evidence level: primary full text.
- PLT-022: coded max_plate_speed_mm_yr 100. Source says: "v0 Maximum plate speed 100 mm y^-1". Result: MATCH. Evidence level: primary full text.
- PLT-095 (and earth-file flag 12): coded oceanic_damping_mm_yr 0.04 (default), 0.12 in presets. Source says: "εo Oceanic elevation damping 4 × 10^-2 mm y^-1". Result: MATCH for the default (0.04). The 0.05 in a pools.rs test comment is not Cortial's, as the earth file suspected. Evidence level: primary full text.
- PLT-128: coded continental_erosion_mm_yr 0.03. Source says: "εc Continental erosion 3 × 10^-2 mm y^-1". Result: MATCH (0.0 in presets is a departure). Evidence level: primary full text.
- PLT-130: coded sediment_accretion_mm_yr 0.3. Source says: "εt Sediment accretion 3 × 10^-1 mm y^-1"; text, section 4.5: "oceanic trenches tend to be filled with sediments, mostly imparted by the formation of an accretion wedge ... we approximate the sediment filling process by modifying the elevation of the point samples in the trenches: z(p; t+dt) = z(p; t) + εt dt". Result: MATCH. Evidence level: primary full text.
- PLT-039: coded subduction_uplift_mm_yr 0.6 (default), 0.42 quick and 0.34 full presets. Source says: "u0 Subduction uplift 6 × 10^-1 mm y^-1". Result: MATCH for the default; the presets are calibrations (the paper's uplift is un-calibrated and unbounded: "e^u_j(p) = u0 f(d(p)) g(v(p)) h(z̃_i(p))", z_j(t+dt) = z_j(t) + e^u_j dt). Evidence level: primary full text.
- PLT-029: coded slab_pull_rad_per_1000km_myr 0.002 ("Cortial section 4.1 epsilon, scaled by front length"). Source says: "ω_i(t+dt) = ω_i(t) + ε Σ_{k=0}^{n-1} (c_i × q_k)/||c_i × q_k|| dt ... The modification of the rotation axis are scaled according to the parameter ε; we use ε ≪ 1 so that a single sample point will have a limited influence over the movement of the plate, whereas long subduction fronts with many points samples will have a noticeable influence." (section 4.1; the "≪" is read from a garbled glyph). Result: PARTIAL. The paper's ε is sum-over-samples (so front length already enters), gives no number, and is not in Table 3. The code's 0.002 is the project's own tuning, as earth-file flag 18 suspected. Evidence level: primary full text.
- PLT-068: coded collision_threshold_km 300. Source says: "The collision event is triggered if the interpenetration distance between the two plates is greater than a user-defined threshold, 300km in our system." (section 6). Result: MATCH. Same section: "we also update the distance of its vertices to the convergence front at every time step by overestimating: d(p; t+dt) ≈ d(p; t) + ||s(p)|| dt" - the paper's counter adds the plate's speed magnitude, so the code's counter (relative speed times dt, never decays, called a bug in the collisions design) follows the paper's own overestimate. Evidence level: primary full text.
- PLT-036/PLT-037/PLT-038: coded f = smoothstep pieces with peak 300 km, g = min(v/v0, 1), h = zn². Source says: "f is defined as a piecewise cubic curve with a maximum at a control distance to the boundary, that fades to 0 when the distance reaches the maximum subduction distance. g is defined as a simple linearly increasing function g(v) = v/v0 ... h(z̃_i) = z̃_i^2, z̃_i(p) = (z_i(p) - z_t)/(z_c - z_t)". Result: MATCH for g and h. PARTIAL for f: the ledger says "the cubic shape is the implementation's own", but the paper itself says piecewise cubic; the control distance (300 km in the code) has no value in the paper. Evidence level: primary full text.
- PLT-071/PLT-072/PLT-073: coded Δz = min(δc A, cap) (1 - (d/r)²)², r = rc sqrt(min(v/v0,1) A/A0). Source says: "r = rc sqrt(v(q)/v0 · A/A0)" and "Δz(p) = Δc A f(d(p, R)/r), f(x) = (1 - x²)²" (section 4.2). Result: MATCH for the form; the code's two stated departures (distance from the contact, and the cap of 1,400/1,100/4,000 m) are not in the paper, which leaves the surge uncapped. Evidence level: primary full text.
- PLT-127/PLT-129: coded erosion z -= (z/zc) εc dt and damping z -= (1 - z/z_floor) εo dt. Source says: "z(p;t+dt) = z(p;t) - (z(p;t)/zc) εc dt" and "z(p;t+dt) = z(p;t) - (1 - z(p;t)/zt) εo dt" (section 4.5). Result: MATCH (the code replaces zt by the floor depth, a stated departure). Evidence level: primary full text.
- PLT-082/PLT-083: coded rift_lambda0 0.012 "Cortial section 4.4", prob = λ e^-λ. Source says: "Rifting events are triggered according to a Poisson Law probability whose parameter λ accounts for the relative continent size: P = λ e^-λ with λ = λ0 f(x_P) A/A0 where λ0 defines the average number of rifting events in the considered time window". Result: PARTIAL. The formula matches; the paper gives no value for λ0 (it is not in Table 3), so 0.012 cannot be Cortial's. Evidence level: primary full text.
- PLT-085/PLT-098: coded rift_fragments [2,4], resampling every 10-60 steps. Source says: "Rifting fractures a given plate P into n sub-plates P_i, n ∈ [2;4..." (closing bracket lost in extraction) and "we compute the movement of the plates and perform a global resampling every 10-60 iterations". Result: MATCH (whether the 4 is inclusive cannot be read). Evidence level: primary full text.
- PLT-020 (and earth-file flag 25): coded CORTIAL_TABLE2 rows (60000, 0.19, 0.58, 0.23), (100000, 0.28, 1.22, 0.21), (250000, 1.24, 3.58, 1.50), (500000, 1.90, 13.10, 7.70). Source says: Table 2 "Total 0.19 0.28 1.24 1.90; Oceanic crust 0.58 1.22 3.58 13.10; Plate rifting 0.23 0.21 1.50 7.70" at 60k, 100k, 250k, 500k samples, "for a planet with an initial state featuring 40 plates and a land coverage ratio of 0.3"; section 7.4: "we perform plate rifting and crust generation every 20 to 120My, depending on maximum plate speed". Result: MATCH on all twelve numbers. Evidence level: primary full text.
- PLT-119: coded hotspots as mantle-fixed points. Source says (section 7.5): "we did not generate hot spots in our model, such as the Hawaiian islands chain. This feature could be easily implemented in our model as specific sample points drifting across plates." Result: MATCH on the quote the ledger uses. Evidence level: primary full text.

Age-depth law and sea floor:
- PLT-096/PLT-094 (GDH1): coded depth test 2,600 + 365 sqrt(t) m to 20 Myr and 5,651 - 2,473 exp(-0.0278 t) after; floor -5,600 m. Source says: could not be read; Nature article is closed and no open copy was found. Crosby, McKenzie & Sclater 2006 (GJI, open at academic.oup.com, fetched) describes only the model: Stein & Stein "used a model of a 95-km-thick cooling plate with a basal temperature of 1450 degC". Result: COULD NOT READ (paywall, and no open article reproducing the coefficients was reachable; the search budget was gone). Evidence level: secondary source (Crosby et al. 2006, model description only).
- PLT-092 / PLT-204 (Parsons & Sclater 2.5 km): coded ridge -2,500 m. Source says, as reproduced in King & Adam 2014 (open copy, fetched) Eq. (1)-(2): bathymetry "-2500.0 - 350 sqrt(...)" for t < 70 Myr and, in Eq. (2), "6400.0 ... 3200 exp[... t/62.8]" for t >= 70 Myr, "where p_sm(θ,φ) is the model bathymetry in meters and t(θ,φ) is the age of the ocean floor in millions of years (Parsons and Sclater, 1977)". Wikipedia "Seafloor spreading" gives the old-age law as "d(t) = 6400 - 3200 exp(-t/62.8)". Result: PARTIAL. 2,500 m and 350 appear as the sqrt-law intercept and coefficient (the code's 2.5 km ridge depth matches the intercept). The extracted text of King & Adam shows an extra factor "0.01" inside the root and the exponent that I cannot interpret from the extraction (possibly a unit convention of that paper), so the scale of the coefficient on sqrt(t) is not settled by this reading. Evidence level: secondary source (King & Adam 2014 full text; Wikipedia).
- PLT-096 / earth-file flag 14: coded shipped law z = z_f + (z_r - z_f) exp(-t/tau), tau = 46.7 Myr, giving 3.58, 4.54, 5.24 km at 20, 50, 100 Myr. Source says: nothing new; this is a comparison to GDH1 numbers the earth file computed from the test's own formula. Result: COULD NOT READ the GDH1 coefficients, so the 0.3-0.65 km offset can only be repeated, not confirmed. Evidence level: none beyond the project's own arithmetic.
- PLT-107: coded q = 510/sqrt(t) capped at 300 mW/m2 (GDH1). Result: COULD NOT READ (same paywall). Evidence level: none.
- PLT-201 (mean sea-floor age 64 Myr): earth file "mean 64 Myr". Source says: "We find the mean age of oceanic crust is 64.2 Myr" (Seton et al. 2020 abstract, Crossref). Result: MATCH. Evidence level: primary abstract.
- Earth fast spreading (run review: "East Pacific Rise about 150 mm/yr"): Source says: "The distribution of spreading modes ... slow (20-55 mm/yr) and fast (75-180 mm/yr) spreading systems at 33% and 39%, respectively ... much smaller proportions ... ultraslow (5%) and super-fast (3%)" (Seton et al. 2020 abstract); Wikipedia East Pacific Rise: "Near Easter Island the rate is over 150 mm per year which is the fastest in the world", but "on the northern end ... roughly 60 mm per year". Result: PARTIAL (150 is a regional peak of the fastest ridge, consistent with the 75-180 fast class; it is not a typical or full rate for the ridge as a whole). Evidence level: primary abstract (Seton) and secondary source (Wikipedia).

Forces and speeds:
- PLT-162 / PLT-030 / PLT-023 (the "3 to 4" ratio attributed to Forsyth & Uyeda): coded label "Earth: 3 to 4". Source says (Forsyth & Uyeda 1975 abstract, full): "The results indicate that the forces acting on the downgoing slab control the velocity of the oceanic plates and are an order of magnitude stronger than any other force. Namely, all the oceanic plates attached to substantial amounts of downgoing slab move with a 'terminal velocity' ... The drag on the bottom of the plates which resist motion is stronger under the continents than under the oceans." (https://academic.oup.com/gji/article/43/1/163/586101). No number of 3 to 4. The statement that does exist in an abstract I fetched is Conrad & Lithgow-Bertelloni 2004: "upper and lower mantle slabs that currently move ~4 times faster than nonsubducting plates" (OpenAlex-reconstructed abstract of JGR 109:B10407, partly garbled but this clause is legible). Result: MISMATCH of attribution (the abstract of Forsyth & Uyeda has no ratio; a ratio of about 4 is in Conrad & Lithgow-Bertelloni 2004). The F&U body was not readable (PDF 403), so I cannot say whether F&U has a ratio inside. Evidence level: primary abstract.
- PLT-188 (factor of 8 for continental drag, "often attributed to Forsyth & Uyeda"): Source says: the F&U abstract has only the qualitative sentence above ("stronger under the continents than under the oceans"); no "8". Result: COULD NOT READ the body; the abstract does not support a factor of 8. Evidence level: primary abstract.
- PLT-188 (factor 2-5): coded keel_drag 3 within a 2-5 range. Source says: "deeply penetrating continental roots increase traction magnitudes by a factor of 2-5 compared to 100 km thick lithosphere. A low-viscosity asthenosphere decreases traction magnitudes by a smaller amount" (Conrad & Lithgow-Bertelloni 2006 abstract, OpenAlex). Result: MATCH for the number; PARTIAL for use: it is a computed mantle-flow shear traction ratio for deep roots versus a 100 km lithosphere, not a ratio of drag under continents versus under oceans. Evidence level: primary abstract.
- PLT-182: coded net slab pull as a share of slab weight. Source says: "net slab pull force increases with increasing slab length and dip up to ~8-12% of buoyancy force, making it twice as large as ridge push force. The remainder is primarily used to drive rollback-induced mantle flow (~70%), bend the subducting plate trench (~15-30%) and overcome shear resistance between slab (0-8%)." (Schellart 2004 abstract, OpenAlex.) The result is from laboratory models. Result: MATCH. Evidence level: primary abstract.
- PLT-186 (slab pull share): Source says: "Observed plate motions are best predicted if slabs in the upper mantle generate slab pull forces that account for about half of the total driving force on plates." (Conrad & Lithgow-Bertelloni 2002 abstract) and "Slab suction presently account for about 40-60% of forces on plates, but only ~30% if a low-viscosity asthenosphere decouples plates" (2004 abstract). Result: PARTIAL against the design's slab_suction_share 0.3: the 2004 abstract's "about 30 percent" holds only in the case with a low-viscosity asthenosphere; the baseline is 40-60 percent. Evidence level: primary abstract.
- PLT-187 (ridge push under 10 percent): Source says: "the main plate-driving forces come from subducted slabs (>90%), with forces due to lithospheric effects (e.g., oceanic plate thickening) providing a very minor component (<10%)" (Lithgow-Bertelloni & Richards 1998 abstract, via Crossref). Result: MATCH. Evidence level: primary abstract.
- PLT-187 (Tibet 6-7e12 N/m) and PLT-183 (ridge push 2-3.5e12 N/m): coded Molnar & Lyon-Caen 1988 "Tibet 6-7e12". Source says: the Molnar & Lyon-Caen abstract contains no force number. Copley, Avouac & Royer 2010 abstract: "the resistive force induced by mountain building in the Himalaya-Tibet area ~5-6 × 10^12 N/m, (2) the net force exerted by subduction zones similar in magnitude to ridge-push (~2.5 × 10^12 N/m)". Result: PARTIAL; the open abstract gives 5-6e12 for Himalaya-Tibet (design says 6-7e12, attributed to M&LC, not readable) and 2.5e12 for ridge push (inside the design's 2-3.5e12 range). Evidence level: primary abstract (Copley et al.).
- PLT-183 (gross slab pull 1-3e13 N/m): Source says: a 2024 paper citing Turcotte & Schubert (Geodynamics, 2014) reads "slab pull (10^13 N m^-1)" and "its small magnitude of ~3 × 10^12 N m^-1" for ridge push, both with reference 9 = "Turcotte, D. L. & Schubert, G. Geodynamics (Cambridge University Press, 2014)" (Li et al. 2024, Nat. Commun. 15:6674, Europe PMC full text). Result: PARTIAL: supports an order of 10^13 N/m for slab pull and about 3e12 for ridge push as textbook values; the range "1-3e13" and Turcotte & Schubert's own page were not read. Evidence level: secondary source (Li et al. 2024 citing Turcotte & Schubert).
- PLT-154 (India 150 to 40 mm/yr): coded 150 to 40 mm/yr between 50 and 35 Ma. Source says: "the rate of convergence between India and Asia dropping from ~15 to ~4 cm/yr" between "50 and 35 Ma" (Copley et al. 2010 abstract). Result: MATCH. Evidence level: primary abstract.
- PLT-198 / earth file (Zahirovic: fastest group 85, slowest 26-28 mm/yr): Result: COULD NOT READ (closed article; EarthByte page returned HTTP 500; no abstract in any index). Evidence level: none.
- Run review "RMS plate speed about 40 mm/yr": no source in repo, none found. Result: COULD NOT READ. Evidence level: none.

Wedge and collision:
- PLT-192: coded wedge_taper_deg 1.5; reference "Davis, Suppe & Dahlen 1983, JGR 88:1153-1172". Source says: reference is right (title "Mechanics of fold-and-thrust belts and accretionary wedges", J. Geophys. Res. 88(B2):1153-1172). The abstract: "The critical shape for taper on verge of failure under horizontal compression everywhere, including basal decollement ... in western Taiwan ... effective coefficient of internal friction of wedge, μ = 1.03" and gives no taper angle in degrees. A secondary summary (Wikipedia "Accretionary wedge") quotes "stable taper angles to increase from ~2.5° to 8°-12.5°" depending on sediment permeability and thickness, and "decreasing taper angle (from >15° to <4°) with increased sediment thickness (from <1 to 7 km)". Result: PARTIAL; citation confirmed; the 1.5 degree value is not in the abstract and lies below every taper in the secondary page (which are wedge tapers, and the design applies 1.5 to "surface slope", which is only part of the taper), so it is suspect, unverified. Evidence level: primary abstract (Davis et al.) and secondary source (Wikipedia).
- PLT-180 (uplift rates): coded Himalaya about 5 mm/yr; western Alps 0.5-2 mm/yr (Nocquet 2016); orogen mean 0.1-1 mm/yr (from memory). Source says: "GPS and levelling measurements independently show a regional pattern of uplift reaching ~2.5 mm/yr in the northwestern Alps" (Nocquet et al. 2016 abstract, Crossref). Result: PARTIAL: the paper's peak is ~2.5 mm/yr, above the design's 0.5-2; the abstract is about present-day (post-convergence, glacial-isostatic influenced) uplift. The 0.1-1 mm/yr "orogen-wide mean" remains without a source. Evidence level: primary abstract.
- PLT-154 (collision duration) and PLT-157 (3-4 collisions per 100 Myr): Source says: nothing found. The only open timing found is Nance 2022: "the episodic assembly and breakup of supercontinents at intervals of ca. 500 m.y." and "Wegener's Pangea (c. 325-200 Ma)". Result: COULD NOT READ any source for a collision rate or collision duration; no figure found. Evidence level: primary full text (Nance, for the cycle only).
- PLT-201 (assembly-to-assembly 300-600 Myr, Earth 300-500): Source says: "intervals of ca. 500 m.y." (Nance 2022, PMC9796656). Result: PARTIAL (500 is inside the target but the "300-500" Earth range was not found). Evidence level: primary full text.

Earth figures:
- PLT-145/PLT-203 (plate count): coded Earth "about 14" (editor hint) versus 52 and 7. Source says: "In addition to 14 large plates whose motion was described by NUVEL-1A poles (Africa, Antarctica, Arabia, Australia, Caribbean, Cocos, Eurasia, India, Juan de Fuca, Nazca, North America, Pacific, Philippine Sea, South America), model PB2002 includes 38 small plates ... for a total of 52 plates" (Bird 2003 abstract). Result: PARTIAL: "14" is Bird's count of large plates (his NUVEL-1A set), so the editor hint has a source, but 14 is not "Earth has 14 plates" and it differs from the "seven largest" reading the plate-check uses. Evidence level: primary abstract.
- PLT-143/145 (Pacific 103.3 million km2, 20 percent; seven largest about 92 percent): Source says: Wikipedia "List of tectonic plates" gives Pacific 103,300,000 km2, North American 75,900,000, Eurasian 67,800,000, African 61,300,000, Antarctic 60,900,000, Indo-Australian 58,900,000, South American 43,600,000. The seven sum to 471.7 million km2, which is 92.5 percent of a 510.1 million km2 planet (my arithmetic, the planet area is not in the fetched page), and Pacific is 20.3 percent. Result: MATCH on arithmetic; not read in Bird 2003 itself. Evidence level: secondary source (Wikipedia).
- PLT-010/PLT-158/PLT-159 (continental crust 41 percent): Source says: "Their total area is 210.4 × 10^6 km², or 41% of the earth's surface" (Cogley 1984 abstract, Crossref). Result: MATCH. "29 percent land" is not stated in the abstract; "submerged parts of the continents (30.6% of their total area)" implies land of about 28.5 percent of the planet by my arithmetic. Evidence level: primary abstract.
- PLT-169 / run review ("Earth's mean land height about 800 m"): Source says: "Mean heights are significantly lower than the 875 m commonly quoted from Kossinna (1933), because submerged parts of the continents (30.6% of their total area) are correctly accounted for; the new estimate of mean height is ~120 m" (Cogley 1984 abstract). Result: PARTIAL: the 800 m figure is the old Kossinna-type land mean (875 m), not Cogley's own number; Cogley's ~120 m includes submerged continental crust and so is a different quantity again. Neither is the 427 m median the calibration targets. Evidence level: primary abstract.
- PLT-122/PLT-121/PLT-131 (crust density 2,700-2,800 kg/m3, thickness 35-40 km): Source says: "The weighted average crustal thickness based on these values is 41.1 km ... The mean crustal density calculated from our model is 2830 kg/m3" (Christensen & Mooney 1995 abstract) and Cogley 1984: "The average thickness of continental crust is 36 km". Result: PARTIAL: 2,800 is within 1 percent of 2,830, but the Airy factor uses 2,700; thickness 35.1 km in the code column is the Cogley value, below Christensen & Mooney's 41.1 km. Mantle 3,300 kg/m3 (PREM would be the source): COULD NOT READ (Crossref metadata only). Evidence level: primary abstract.
- PLT-108/109/110 (hotspots; task item Courtillot 2003 and Morgan): Source says: King & Adam 2014 (full text): "Davies (1988) considered 26 locations, Sleep (1990) considered 37 locations, Steinberger (2000) considered 44 locations, Courtillot et al. (2003) considered 49 locations, Anderson (2005) considered more than 60 locations and Ito and van Keken (2007) consider 69 locations". Wikipedia "Hotspot (geology)": "Estimates for the number of hotspots postulated to be fed by mantle plumes have ranged from about 20 to several thousand, with most geologists considering a few dozen to exist", and lists seven primary hotspots (Easter, Iceland, Hawaii, Afar, Louisville, Reunion, Tristan) from Courtillot et al.; track rate "Hawaii ... 92 ± 3 mm/yr" (from Morgan & Morgan 2007, via Wikipedia). Result: PARTIAL: the code's 8 initial hotspots matches the primary-hotspot count of 7 loosely, but the code cites no source and 3 per 100 Myr birth rate and 40-150 Myr lifetime have none. Evidence level: primary full text (King & Adam) and secondary source (Wikipedia).
- PLT-042/PLT-061 (arc-trench distance, slab dip): Source says: "H ranges from 72 to 173 km with a global average of 105 km" for the depth to the top of the slab beneath arc volcanoes (Syracuse & Abers 2006 abstract); Lallemand et al. 2005 gives dip relations (e.g. "Slabs dip more steeply, by about 20° on average, beneath oceanic overriding plates than beneath continental ones") but no global mean dip. Result: COULD NOT READ a trench-to-arc horizontal distance or mean dip; the abstract gives depth only. Evidence level: primary abstract.
- PLT-010 (Stern 2002 arc crust 20-35 km in tens of Myr; Bradley 2008 margin lengths): Result: COULD NOT READ (Stern 2002 abstract has no such number; Bradley abstract unavailable). Source seen: Stern 2002 abstract: "Earth's 55,000 km of convergent plate margins". Evidence level: primary abstract (Stern 2002, partial).
- Orogen/continental erosion and belt widths, Wilson cycle length, number of continental collisions per 100 Myr, "Himalaya about 2,400 km long": Result: COULD NOT READ; no open source found with the search budget gone.

## Climate physics (CLM)

Results: MATCH 15, PARTIAL 17, MISMATCH 3, COULD NOT READ 6.

Evidence labels: "primary full text" = I read the paper's own text; "secondary source" = text in another paper or page I fetched, named; "citing passage" = passage from a citing paper returned by the Semantic Scholar API (weaker still); "primary abstract"; "primary code" = the project's cited source code itself.

#### Energy balance (climlab, NCC81)

- CLM-001: coded S0 = 1365.2 m2. Source says: `S0 = 1365.2       # solar constant (W / m**2)` (https://raw.githubusercontent.com/climlab/climlab/main/climlab/utils/constants.py, and the same at commit fcbde37). Measured TSI for comparison: "which reports a value of 1360.8 m^2 representative of the 2008 solar minimum (Kopp & Lean, 2011)" (https://spot.colorado.edu/~koppg/TSI/, Kopp's own page).
  Result: PARTIAL. Matches climlab's constant exactly. climlab's number is 4.4 m2 (0.32 %) above the Kopp & Lean 2008-minimum value; the climlab file gives no citation for 1365.2.
  Evidence level: primary code (climlab); secondary source (Kopp's page) for the 1360.8.

- CLM-007, CLM-008 (a0 = 0.33, a2 = 0.25): Source says: `class EBM_seasonal(EBM): def __init__(self, a0=0.33, a2=0.25, ai=None, **kwargs):` (climlab/model/ebm.py line 366 at fcbde37; line 401 at v0.9.2). The albedo form is `albedo = self.a0 + self.a2 * P2(np.sin(phi))` (surface/albedo.py line 174, P2 = (3x^2-1)/2).
  Result: MATCH. Same file's base class `EBM` instead has a0 = 0.3, a2 = 0.078.
  Evidence level: primary code.

- CLM-009, CLM-010 (a_i = 0.62, T_f = -10 degC): Source says: `Tf=-10.`, `a0=0.3`, `a2=0.078`, `ai=0.62,` in `EBM.__init__` (ebm.py lines 236-239); and in `EBM_seasonal`'s docstring: "If argument for ice albedo 'ai' is not given, the model will not have an albedo feedback." (line 376), with `ai=None` the default.
  Result: PARTIAL. The numbers are climlab's, but they belong to the base `EBM` class paired with a0 = 0.3, a2 = 0.078. The seasonal class that supplies a0 = 0.33, a2 = 0.25 has NO ice-albedo feedback by default. So the combination in the code (0.33, 0.25, 0.62, -10) is not a climlab default set. climlab's step is hard (ice where T < Tf); the code smooths it over +/-3 degC.
  Evidence level: primary code.

- CLM-020 (90 bands): Source says: `num_lat=90,` (ebm.py line 228). Result: MATCH. Evidence level: primary code.

- climlab A, B, D (210, 2, 0.555) as quoted in the build log: Source says: `A=210.,` `B=2.,` `D=0.555,  # in W / m^2 / degC, same as B` (ebm.py lines 232-234). Origin of those: Rose's lecture notes: "A = 210 m2 and B = 2 m2/degC" from an NCEP reanalysis fit with "best fit A = 214, B = 1.6" and "D = 0.55 m2/degC" (https://www.atmos.albany.edu/facstaff/brose/classes/ATM623_Spring2015/Notes/Lectures/Lecture14%20--%20Diffusive%20energy%20balance%20model.html; WebFetch summary, quotes as returned).
  Result: MATCH (values as the build log says). The swap to NCC81's numbers replaces an NCEP-based fit with a satellite-era fit, so the 3 degC shift is expected from different physics of the fit, not evidence either is wrong.
  Evidence level: primary code; secondary source (lecture notes).

- CLM-003, CLM-004: coded A = 203.3 m2, B = 2.09 m2/degC, attributed to NCC81. Sources say: "The thick red line takes Budyko's (1969) empirical values for present climate conditions: A=203.3 W m-2 and B=2.09 W m-2 degC-1." (Lohmann 2020, ESD 11:1195, https://esd.copernicus.org/articles/11/1195/2020/). And, in Table 1 of Samuelsberg & Jakobsen 2025 (NPG 32:23, PDF text): "A, B, C, C_land and T_s are taken from North et al. (1981)." with the table row "A 203 W m-2, B 2.09 W m-2 degC-1". A web-search summary (search snippet, not fetched) says the pair are the best fit to northern-hemisphere satellite data per North & Coakley (1979).
  Result: PARTIAL. Two independent papers print 203.3/203 and 2.09; one calls them Budyko's, the other NCC81's. NCC81 itself could not be read (scanned PDF with no text layer; Wiley blocked). Not confirmed in the cited paper.
  Evidence level: secondary source (Lohmann 2020; Samuelsberg & Jakobsen 2025).

- CLM-005: coded D = 0.649 m2/K, "North et al. 1981". Source says: no passage found with 0.649. The nearest: "an appropriate value for Earth is ~0.6 W m-2 K-1, North & Kim, 2017" (citing passage in "The Key Factors Controlling the Seasonality of Planetary Climate", 2022). Samuelsberg & Jakobsen 2025 use D = 0.208 "taken from Kaper and Engler (2013)" (different convention: they say D is per radian squared).
  Result: COULD NOT READ (NCC81 unreadable). Order of magnitude (~0.6) is supported; the exact 0.649 is unconfirmed, and one web-search summary that gave "D = 0.649 ... pole-to-equator T = -28 C" is not evidence.
  Evidence level: citing passage.

#### CO2 forcing

- CLM-070 coefficients: coded dF = 5.32 x + 0.39 x^2, x = ln(C/280). Source says: Table 2, "New Radiative Forcing Fits", row CO2: "F_i = 5.32 ln(C/C0) + 0.39 [ln(C/C0)]^2" with "Concentration (ppmv) 200-10,000"; footnote "C0 = 278 x 10^-6 ... All radiative forcings are given from pre-industrial concentrations." (Byrne & Goldblatt 2014 Table 2; the PDF text layer shows `5.32 ln (C/C0) + 0.39`, with the squared bracket lost in extraction, and the article's erratum notes that "several pairs of square brackets were missing" in Table 2 in the original; the squared form is my reading of the corrected article and of the arithmetic below.) Abstract: "CO2 forcing is calculated over the range 100 ppmv to 50,000 ppmv, and the maximum forcing is 38.1 W m-2." Arithmetic check by me: with C0 = 278 and C = 50,000, x = 5.192, 5.32x + 0.39x^2 = 38.1 m2.
  Result: MATCH on coefficients and on 38.1 m2 at 50,000 ppm (x with C0 = 280 gives 38.06). Two small differences: the paper's C0 is 278 ppm, the code's 280; the paper's quantity is the instantaneous forcing at the tropopause F_i, not the stratosphere-adjusted F_a that Myhre's 5.35 gives (the paper says F_i exceeds F_a by "10% for a small increase" and "7.5% larger at 8x CO2").
  Evidence level: primary full text.

- CLM-070 and CLM-071 range of validity: coded: BG fit used above 2,000 ppm with no upper limit; the solver searches 1 to 1e6 ppm (`clock.rs: (1.0f64.ln(), 1.0e6f64.ln())`, `carbon.rs: forcing`). Source says: fit concentration "200-10,000" ppmv (Table 2); calculation range "100 ppmv to 50,000 ppmv" (abstract).
  Result: MISMATCH (range). The fit is stated valid to 10,000 ppmv and the underlying calculation stops at 50,000 ppmv. The code applies the quadratic to 1,000,000 ppm: 100 times the fit's stated range, 20 times the calculated range. By my arithmetic the code gives 69.6 m2 at 1e6 ppm and 62.8 m2 at the coded snowball escape of 5.5e5 ppm, where a pure log (5.35 x) gives 43.8 and 40.6. The paper gives no support above 50,000 ppm.
  Evidence level: primary full text (range); arithmetic mine.

- CLM-069: coded 5.35 x (Myhre 1998). Sources say: Byrne & Goldblatt Table 1 prints "F_a = alpha ln(C/C0), alpha = 5.35" (their row labelled IPCC [1990]); citing passages: "Myhre et al. (1998): ... where 5.35 is an empirical value in W m-2" (2026 vegetation paper); Myhre's abstract (Crossref): "The radiative forcing due to CO2, including shortwave absorption, is 15% lower than the previous IPCC estimate."
  Result: MATCH (secondary). The 5.35 itself is in Myhre's body which I could not read.
  Evidence level: secondary source (Byrne & Goldblatt Table 1); citing passage.

#### Silicate weathering and the carbon cycle

- CLM-063 (temperature factor exp(0.09 dT), about 63 kJ/mol): Source says: "The value used for the standard 15 kcal/mole (ACT = 0.09) is the same as that used in GEOCARB II" and eq. 2 "f(T) = exp{ACT[T(t) - T(0)]} x {1 + RUN[T(t) - T(0)]}^0.65" (Berner & Kothavala 2001, text near eq. 2 and section on activation energy; the PDF extraction renders "=" as "5" and "-" as "2").
  Result: MATCH. 15 kcal/mol is 62.8 kJ/mol by my arithmetic, so "about 63 kJ/mol" holds.
  Evidence level: primary full text.

- CLM-064 (runoff 0.038/K, exponent 0.65): Source says: "Values for the same parameters from GEOCARB II are Ws = 12.93, G = 6.0, and RUN = 0.038 for all times." and the GEOCARB III standard case "RUN = 0.045 for colder periods (340-260 Ma and 40-0 Ma) ... RUN = 0.025 for the warmer rest of the Phanerozoic." The power 0.65 in the Fwsi expression "reflects dilution at high runoff." (Berner & Kothavala 2001).
  Result: PARTIAL. The form and 0.65 match eq. 2. The coded 0.038 is the GEOCARB II value (so the `params.rs` attribution to Berner 1994 is right; the `carbon.rs` header's "Berner & Kothavala 2001 eq. 2" gets the form but not the number, since GEOCARB III uses 0.045 or 0.025). Berner 1994 itself unreadable (scan). The code comment `mod.rs` "GEOCARB's ~3.8 %/K" matches 0.038 and is therefore GEOCARB II's.
  Evidence level: primary full text.

- CLM-065 (CO2 exponent 0.3, Walker et al. 1981): Source says: nothing found for 0.3 in Walker et al.; the same Berner & Kothavala text gives "f(CO2) = (RCO2)^0.5 for pre-vascular plant weathering" (eq. 4) and "f(CO2) = [2 RCO2/(1 + RCO2)]^FERT for weathering affected by vascular plants" (eq. 5), with FERT "previously assumed to be equal to 0.4". Citing passage on Walker 1981: "the e-folding temperature becomes 13.7 K if assuming the exponential relation ... used in J. C. G. Walker et al. (1981)".
  Result: COULD NOT READ for 0.3 (Walker unreadable). Related finding: Walker 1981's temperature law has an e-folding of 13.7 K (0.073 per K, citing passage), not 0.09 per K; the code's 0.09 is Berner's. Berner & Kothavala's CO2 exponent (0.5 for pre-vascular, FERT 0.4 for plant-affected) differs from 0.3.
  Evidence level: primary full text (Berner & Kothavala); citing passage (Walker's 13.7 K).

- CLM-075 (Sun's brightening law): coded L(t) = L0/(1 + 0.4 (1 - t/t0)), t0 = 4.57 Gyr (`carbon.rs: gough`). Sources say: "L(t) = L0 [1 + 2/5 (1 - t/t0)]^-1 (31) when t > t0" with the Sun's age t0 = 4.57 Gyr: "The habitability of a stagnant-lid Earth" (2017, A&A); "Lbol(t) = [1 + 2/5 (1 - t/t0)]^-1 Lbol,0 ... t0 = 4.57 Gyr" ("The Exosphere as a Boundary", 2022); "L(t)/L0 = 1/(1 + 2/5 (1 - t/t0)), where t0 = 4.57 Ga" ("A Closer Earth and the Faint Young Sun Paradox", 2013). Springer abstract summary (search snippet): "about 70% of its current value".
  Result: MATCH (2/5 = 0.4; t0 = 4.57). The coded value at t = 0 is 1/1.4 = 71.4 %, consistent with "about 70 %".
  Evidence level: citing passage (four citing papers). Gough 1981 itself not read.

- CLM-050 (Hoffman 1998: about 350 times today's CO2, in about 3 to 4 Myr at 5.4e12 mol/yr): Source says (abstract): "...ended abruptly when subaerial volcanic outgassing raised atmospheric carbon dioxide to about 350 times the modern level." (Hoffman et al. 1998, Crossref abstract). Hoffman et al. 2017: "Even the present rate of release would yield 1 bar of carbon dioxide in only 20 million years" (quoting Walker et al. 1981 p. 9781).
  Result: PARTIAL. 350 times matches. The 3-4 Myr and 5.4e12 mol/yr are not in the abstract and the body is paywalled. By my arithmetic, 1 bar per 20 Myr is 0.05 bar/Myr, and 350 x 280 ppm is 0.098 bar, about 2 Myr; consistent in order of magnitude with the unit test's "2 to 4 Myr".
  Evidence level: primary abstract; primary full text (Hoffman 2017 sentence).

- CLM-057 (0.5 wt % CO2 in flood basalt, 2,900 kg/m3): Source says (citing passages): "Self et al. [2006] arrived at an estimate of ~0.5 wt.% CO2 based primarily on historical basaltic eruptions." (2023 paper on LIP carbon release); "an estimated Deccan basalt CO2 content of 0.5 weight percent or 14 Tg CO2 per km3 (Self et al., 2006)" (2020 Rajahmundry paper).
  Result: MATCH (0.5 wt %; 14 Tg/km3 agrees with 0.005 x 2,900 kg/m3 = 14.5 Tg/km3 by my arithmetic). CLM-058 (even spread over 1 Myr): COULD NOT READ; no passage found.
  Evidence level: citing passage.

- CLM-053 (3.4 km2/yr of sea floor): Source says (citing passages): "The mean oceanic plate production is estimated (Parsons, 1982; Rowley, 2002) to 3.4 km2/a, also representing a reasonable estimate for how much oceanic lithosphere is subducted per year." (2018 paper on U-cycling); "Ocean crust is currently being produced at a rate of 3.4 km2 per year" (2025 paper citing Rowley).
  Result: MATCH (secondary; also supports setting consumption equal to production). Evidence level: citing passage.

- CLM-054: coded 3.4 km2/yr consumed, with Bird 2003 (51,000 km of subduction zones at 62 mm/yr) in the module doc. Source says: no passage found (the citing passages returned for Bird 2003 were regional and did not give a total subduction length or rate).
  Result: COULD NOT READ (paywalled; see Bird2003 entry). Evidence level: citing passage (irrelevant).

- CLM-048: coded R = 5, "the Revelle factor of about 10 lowered by sea-floor CaCO3 neutralisation (about 7-8 % of a pulse stays airborne)" (Archer 2009). Source says: citing passages give airborne fractions of "10 to 21 %" at 10,000 yr for a 1,000-GtC pulse (a review table) and "40 % after 100 years, 20 % after 1,000, 10 % after 10,000" (a 2022 paper); neither is the 7-8 % the code quotes.
  Result: COULD NOT READ (the code's 7-8 % may refer to the very long-time limit; not checked). Evidence level: citing passage.

- CLM-046: coded reservoir 3.193e18 mol C at 280 ppm. Source says: the COPSE text I could read has no reservoir numbers (they are in tables that are not extractable from the PDF).
  Result: COULD NOT READ (table values not extractable). Evidence level: none.

- CLM-077 (hydrological sensitivity about 2 %/K): Source says (citing passages): "global mean precipitation ... increases by approximately 2% per degree of warming, constrained by energy limitations (... Held & Soden 2006 ...)" (2025 paper); "total precipitation is projected to increase by 1%-3% on average (Allen & Ingram, 2002; Held & Soden, 2006)" (2024 paper).
  Result: MATCH (secondary; ~2 %/K within the 1-3 % range). Evidence level: citing passage.

#### Orbit and insolation

- CLM-032, 033, 035, 037 (Milankovitch constants as coded): obliquity 22.1 to 24.5 degrees, 41 kyr; precession perihelion period 23 kyr; eccentricity cycles 95, 124, 405 kyr; e = 0.028 mean, max about 0.055. Sources say: "The angle of the Earth's axial tilt with respect to the orbital plane (the obliquity of the ecliptic) varies between 22.1 and 24.5, over a cycle of about 41,000 years." ; "Apsidal precession shortens this period to about 21,000 years, at present." ; "The major component of these variations occurs with a period of 405,000 years (eccentricity variation of +/-0.012). Other components have 95,000-year and 124,000-year cycles ... The present eccentricity is 0.0167 and decreasing." ; "highest eccentricity was 0.0679 in the last 250 million years." (https://en.wikipedia.org/wiki/Milankovitch_cycles, fetched). Citing passages for Berger 1978 / Laskar 2004: "precession (~1/19 and 1/23 kyr), eccentricity (~1/100 and ~1/400 kyr)", "95-128 ka and 405 ka orbital eccentricity indexes; Laskar et al., 2004".
  Result: PARTIAL. Obliquity range and period, eccentricity periods (95, 124, 405) and present e match. The code's single 23-kyr precession period is the longer of the two climatic precession periods (19 and 23 kyr; mean about 21 kyr), so one period stands in for two. Code's 405-kyr amplitude 0.011 against 0.012 in Wikipedia; fine. The code's own series has a second obliquity term (53.6 kyr) not in the sources (ledger flag 2).
  Evidence level: secondary source (Wikipedia; not authoritative); citing passages for Berger and Laskar. Neither Berger 1978 nor Laskar 2004 was read.

#### Ice, sea level, snowball

- CLM-103 (snowball ice sheets hold about 1 km of sea level, Hoffman 2017): Source says: "As Snowball ice sheets recede, assuming they hold 1000 m of sea-level change equivalent, gravitational and deformational effects cause sea level to fall by several 10^1 m in the near field (419)." (Hoffman et al. 2017, section on cap-carbonate flooding). Also: "The magnitude of the flooding events associated with both Cryogenian glacial terminations implies the existence of more ice than modeled at 100 mbar of CO2".
  Result: PARTIAL. The 1,000 m is an assumption stated in a sentence about sea-level fingerprints (from the paper's reference 419), not a measured or modelled result of the paper; the paper then says geology implies more ice than its model had at 100 mbar. The code's "thought to hold about 1 km (Hoffman et al. 2017)" is a fair paraphrase of that assumption but overstates it as a finding. The built scenario falls 1.48 km, 48 % above this assumed figure.
  Evidence level: primary full text.

- Snowball escape CO2 (coded about 5.5e5 ppm, 0.55 bar; build log: Hoffman 2017 "10^4-10^5 ppm"): Source says: "Terminations should be accompanied by extraordinarily high atmospheric CO2 levels, equivalent to 10^4 to 10^5 parts per million (ppm) by volume in the present atmosphere" (Hoffman et al. 2017); the abstract: "Geochemical data imply that CO2 was 10^2 PAL (present atmospheric level) at the younger termination"; models "run at CO2 equal to 0.1 and 100 mbar", and "At 100 mbar, the response weakens again because the amount of remaining ice is small".
  Result: MISMATCH. The paper's range is 10^4 to 10^5 ppm, i.e. 0.01 to 0.1 bar; the code's snowball ends at 5.5e5 ppm, 5.5 to 55 times higher than the paper's range and 5 times the paper's own highest modelled CO2 (100 mbar, at which its ice sheets are mostly gone). The same paper says the geochemical constraint at the younger termination is 10^2 PAL, about 0.03 bar.
  Evidence level: primary full text.

- CLM-108 (3.0e7 km3, 65 m for Antarctica plus Greenland): Sources say: "the volume of ice contained in the Antarctic ice sheet (27 million km3) and its potential contribution to sea-level rise (58 m)" (Fretwell et al. 2013, Bedmap2 abstract; "from 57 m to 58 m" in the discussion). "Total ice volume of Greenland is 2.99 +/- 0.02 times 10^6 km3, yielding a potential sea level rise of 7.42 m" (Morlighem et al. 2017, key points).
  Result: MATCH. 27 + 2.99 = 29.99e6 km3 (3.0e7) and 58 + 7.42 = 65.4 m (65 m), my sums.
  Evidence level: primary full text (both).

- CLM-106 (2.47 m per 1e6 km3): coded -(917/1028) x dV / 3.61e8 km2 x 1000 = 2.47 m per 1e6 km3. Sources say: Greenland 7.42 m for 2.99e6 km3 (2.48 m per 1e6 km3, my division); Antarctica 58 m for 27e6 km3 (2.15 m per 1e6 km3, my division). Bedmap2: "the mean elevation of the bed has fallen significantly (by 72.6 m), resulting in a much greater volume of ice below sea-level (from 2.1 x 10^6 km3 to 2.6 x 10^6 km3)", so Antarctic sea-level potential is for ice above flotation.
  Result: PARTIAL. The coded 2.47 equals Greenland's measured ratio but not Antarctica's (2.15, because part of the ice sits below sea level or floats). Applied to 3.0e7 km3 the code's formula gives 74 m, not the 65 m the comment cites for the same ice. (3.61e8 km2 ocean area not checked against a source.)
  Evidence level: primary full text; arithmetic mine.

- CLM-092 (sea water freezes at -1.8 degC): Source says: "in polar regions with an ocean salinity of about 32 PSU, the water begins to freeze at -1.8 degC (28.8 degF)." (NSIDC, https://nsidc.org/learn/parts-cryosphere/sea-ice/science-sea-ice).
  Result: MATCH (for 32 PSU; "about -2 C at typical salinity" in Wikipedia "Seawater"; lower for 35 PSU, TEOS-10 not read).
  Evidence level: secondary source (NSIDC).

- CLM-093/099 (ice 917 kg/m3, sea water 1028): Source says: ice Ih "0.9167-0.9168 g/cm3" (Wikipedia "Ice", citing the CRC handbook); sea water "The average density at the surface is 1.025 kg/L" (Wikipedia "Seawater").
  Result: PARTIAL. Ice 917 matches. The code's sea-water 1028 is 3 kg/m3 (0.3 %) above the 1,025 surface value; 1,028 is a normal mean-ocean figure but I did not find it in a fetched source.
  Evidence level: secondary source (Wikipedia, not authoritative).

- CLM-114 (Ohmura et al. 1992: P = 645 + 296 T + 9 T^2 at the ELAs of 70 glaciers): Source says (Fig. 2 caption): "Annual total precipitation (or winter mass balance plus summer precipitation) and the mean free-atmospheric temperature observed at the ELAs for 70 glaciers. ... The solid line and the broken lines indicate the square regression line and the standard deviation, respectively." Axis label: "mean summer temperature in free atmosphere at ELA". Conclusions: "The climate prevailing at the equilibrium lines is identified as a function of annual total precipitation and summer temperature in the free atmosphere." (Ohmura et al. 1992, J. Glaciol. 38(130)).
  Result: PARTIAL. 70 glaciers, a quadratic (square) regression, summer temperature: confirmed. The three coefficients are NOT readable in the scanned OCR text. Two points of concern: (1) the paper's T is free-atmosphere summer temperature AT the ELA, but the code feeds it a surface three-warmest-month mean taken at sea level and then adds a lapse correction, a different quantity; (2) the data are glaciers with ELAs at a few hundred mm and up, so the code's P = 0 end (-2.35 degC) is an extrapolation (as the build log says).
  Evidence level: primary full text (caption, axis label); coefficients not read.

- CLM-113 (ELA rule, goSPL): Source says: "ELA(x) = (T_ref(x) - T_ELA) / Gamma" and "T_ref(x) = T(x) + Gamma * z(x) ==> ELA(x) = z(x) + (T(x) - T_ELA) / Gamma" with example arguments `--lapse 0.0065 --t-ela -2.0 --band 400` (goSPL gospl/tools/ela_from_temperature.py, docstring lines 8-16, 41-52; raw file https://raw.githubusercontent.com/Geodels/gospl/HEAD/gospl/tools/ela_from_temperature.py).
  Result: MATCH. Evidence level: primary code (the project's cited source).

- CLM-101 (Nye plastic profile): Source says (citing passages): "perfect-plasticity basal shear stress relationship (Nye, 1952 a, b) ... tau_b = rho g h sin(alpha)" and flowline models that "assume perfectly plastic ice rheology (Nye, 1952a, 1952b)".
  Result: PARTIAL (the principle is Nye's; the project's H = sqrt(2 tau0 L / rho g) form and Cuffey & Paterson section 8.7 not read). Evidence level: citing passage.

- CLM-104/105 (Imbrie & Imbrie 1980, b = 0.6, tau_m = 17 kyr): Source says (citing passage): "...smaller than the 17 kyr for the larger late Pleistocene ice sheets [Imbrie and Imbrie, 1980]" (Mudelsee & Raymo 2005, "Slow dynamics of the Northern Hemisphere glaciation"). The derived 42.5 / 10.6 kyr: 17/(1-0.6) = 42.5, 17/(1+0.6) = 10.6 (my arithmetic).
  Result: PARTIAL. 17 kyr supported by one citing passage; b = 0.6 and the asymmetric-time-constant structure not read in the paper (paywalled). Evidence level: citing passage.

#### Earth reference values

- CLM-026, CLM-078, CLM-138 (Earth's global mean surface temperature; code uses 15.6, 15 and 14-15 degC): Source says: "Estimated Jan 1951-Dec 1980 global mean temperature (C): Using air temperature above sea ice: 14.102 +/- 0.019; Using water temperature below sea ice: 14.698 +/- 0.019" (Berkeley Earth Land/Ocean file header, Rohde & Hausfather 2020 ESSD 12:3469, https://berkeley-earth-temperature.s3.us-west-1.amazonaws.com/Global/Land_and_Ocean_complete.txt). Annual anomalies in the same file against that baseline: 2000 +0.445, 2016 +1.059, 2023 +1.229, 2024 +1.311 degC.
  Result: MISMATCH for 15.6. Added to the 1951-1980 mean, the file gives 14.1 degC for that period and 15.41 degC for 2024 (the warmest year in the file), 14.10 + 1.311 by my sum (using the air-above-sea-ice series; the water-temperature series would add 0.6). So 15.6 is above every annual value in the file and is not a long-term mean; 14-15 degC is the right range for the 20th century and 15.0 is about the 2010s. A single citable figure: 14.1 degC for 1951-1980 (Rohde & Hausfather 2020 data), or 14.0 for 1961-1990 (Jones et al. 1999, Rev. Geophys. 37:173-199, which I did not read).
  Evidence level: primary data file.

- Köppen land shares (Peel, Finlayson & McMahon 2007: B 30.2, D 24.6, A 19.0, C 13.4, E 12.8 %): Source says: "Globally the dominant climate class by land area is arid B (30.2%) followed by cold D (24.6%), tropical A (19.0%), temperate C (13.4%) and polar E (12.8%)." (HESS 11:1633, section 4).
  Result: MATCH. Evidence level: primary full text.

#### Items from the P1-26 design doc marked "cited from memory, not re-read" (existence and fit)

- CLM-145 (Bretherton, Peters & Back 2004: P = (tau) exp(a (RH - r_ref)), a = 15.6): Source says (citing passage, Rushley et al. 2018 GRL): "Our ad and Pr derived from the version 5 data do not exactly match the coefficients ad = 15.6, Pr = 8.22 x 10^5 mm d-1 found by Bretherton et al. (2004)." Other citing passages: "Bretherton et al. (2004) used a nonlinear least squares method to fit the relationship between precipitation and column relative humidity (CRH) by an exponential over the tropical oceans"; "an e-folding growth rate of ~21, higher than the value of ~15 documented by observational studies (Bretherton et al., 2004; Rushley et al., 2018)."
  Result: PARTIAL (paper exists, says what it is cited for: an exponential precipitation-column-humidity relation over tropical oceans with exponent about 15.6). Two cautions: the design doc's phrase "15.6-fold per unit RH" is wrong (the exponent a = 15.6 means a factor of e^15.6 per unit RH, or an e-fold per 0.064 RH); and I did not see how Bretherton defines the humidity variable or whether r_ref = 0.7 and the 8-day tau_P are theirs.
  Evidence level: citing passage.

- CLM-143 (Manabe 1969: bucket, S_max = 150 mm): Source says (citing passages): "a simple bucket model widely used in the Earth land community (Manabe, 1969 ...)" with "maximum water capacity of 150 kg m-2"; "runoff only occurs when the total soil water content exceeds the maximum field capacity, set to 150 kg m-2 as in Manabe (1969)."
  Result: MATCH on 150 mm (150 kg/m2 = 150 mm). The 0.75 factor in beta = min(1, S/(0.75 S_max)) not seen. Paper exists, citation right (Part I, 739-774). Evidence level: citing passage.

- CLM-133 (Trenberth & Caron 2001: ocean about 2 PW at 15-20 N): Source says (citing passage, ICON-O 2022): "global northward transport of both configurations has a maximum at ~20N of ~1.8 PW that agrees ... with the reanalysis based estimate from Trenberth and Caron (2001)."
  Result: PARTIAL (1.8 PW at about 20 N is the secondary figure; the design doc says about 2 PW at 15-20 N). Evidence level: citing passage.

- CLM-132 (Ferreira et al. 2018: deep water forms in the narrowest polar-connected basin of 20-70 degrees; upwelling equatorward of 50; T_deep 2 degC): Source says: abstract (Crossref): "While the Atlantic Ocean is ventilated by high-latitude deep water formation and exhibits a pole-to-pole overturning circulation, the Pacific Ocean does not. ... the Atlantic-Pacific asymmetry occurs because the Atlantic is more saline, enabling deep convection." Citing passage: "the relative widths of the Atlantic and Pacific basins have been associated with asymmetries in deep water formation between the basins (Ferreira et al., 2018 ...)"; "the basin selection ... for the mid-depth overturning is controlled by the shape of the continents (Ferreira et al. 2018)."
  Result: PARTIAL. The review exists, the pages are right, and it discusses basin geometry as one control. Its abstract's own answer is salinity. The specific numbers (2 degC, 50 degrees, 20-70 degrees) were not checked. Evidence level: primary abstract; citing passage.

- CLM-141 (Lindzen & Nigam 1987: pressure anomaly p' = -kappa T', kappa = 60 Pa/K): Source says (citing passages): "pressure adjustment mechanism (Lindzen and Nigam 1987), which attributes surface wind variations to pressure gradients arising from air temperature contrasts across the SST front within the MABL" (2026 paper); "SST gradients are known to influence tropical low-level winds and convergence through pressure-gradient, frictional, and momentum-balance constraints".
  Result: PARTIAL (paper exists; mechanism is right; applies to SST gradients over ocean, not land monsoons; kappa = 60 Pa/K not checked). Evidence level: citing passage.

- CLM-134 (Serreze, Kahl & Schnell 1992: winter inversions, Siberian January surface 10-20 K below 850 hPa): Source says (citing passages): "temperature difference across the inversion exceeds 40 K (Serreze et al. 1992)" as a rejection limit; "inversion strengths range from ... to ~2 C (Serreze et al., 1992)" for central Arctic summer; "The temperature difference from the surface to the top of the inversion was 2.6 vs. 11.8 K for the ice island profiles (Serreze et al. 1992)."
  Result: COULD NOT READ (the 10-20 K claim not found; the paper exists, Eurasian Arctic inversions, J. Climate 5: 615-629; author order differs, see entry). Evidence level: citing passage.

- Klein & Hartmann 1993 (low-cloud): Source says (citing passages): "lower-tropospheric stability (LTS), defined as the difference in potential temperature between 700 hPa and the surface (Klein and Hartmann 1993)".
  Result: PARTIAL (exists; cited-for use is low stratiform cloud; no coded value depends on it). Evidence level: citing passage.

---

## What each pass could and could not reach

### Meteor impacts (IMP)

Sources: 44 entries (43 from the sources file plus Herrick et al. 1997, which is the real origin of Collins et al.'s eq. 28). Holsapple 1993 is cross-referenced inside the McKinnonSchenk1985 entry, not given its own heading.

By Access (44 entries): open copy 5 (Collins2005, DavisonCollins2007, Grieve1987, LeFeuvre2011 [not readable here], Pike1985 [image-only scan, unreadable]); book 9; paywalled 22; not found 8 (Gilbert1893, GaultWedekind1978, McKinnonSchenk1985, Pike1980, WoodHead1976, Moore1974, GrieveCintala1992, Donofrio1998). DOIs confirmed through Crossref for 24 sources (plus Holsapple 1993 inside another entry).

By Read in this pass: full text 3 (Collins2005, DavisonCollins2007, Grieve1987 via OCR); abstract 1 (FassettThomson2014, via OpenAlex); fetched but unreadable 1 (Pike1985); access denied 2 (LeFeuvre2011, AlexopoulosMcKinnon1994); Crossref metadata only 21 (every other paywalled paper with a DOI, plus the GrieveStoffler/Ames/etc. deposit papers); not fetched at all 16 (books, proceedings and the Collins-listed references). Collins2005's reference list was also used as secondary bibliographic evidence for Shoemaker1962, GrieveDenceRobertson1977, GrieveCintala1992, Pike1980, McKinnonSchenk1985 and Herrick1997.

Spot checks: 31 rows. MATCH 10, PARTIAL 15, MISMATCH 1, COULD NOT READ 5.

MISMATCH and PARTIAL, one line each:
- MISMATCH IMP-042: Collins et al. 2005 eq. 28 is d_fr = 0.4 D_fr^0.3 km (Herrick et al. 1997, Venus); the coded 0.294 D^0.301 is not in the paper (about 27 percent shallower, 0.72 vs 0.98 km at 20 km).
- PARTIAL IMP-033: the paper switches to the complex law at D_tc > 2.56 km; the code takes the larger of the simple and complex diameters.
- PARTIAL IMP-034: 3.2 km is Collins et al.'s value (after Dence 1965); the cited Grieve 1987 gives about 2 km (sedimentary) and 4 km (crystalline), no 3.2.
- PARTIAL IMP-038, 062, 054: values match but equation numbers are wrong (rim height is eq. 48 not 25; melt is eq. 30 not 12; ejecta is eq. 47 not 32; V_br is eq. 23).
- PARTIAL IMP-020: 20 km/s is the top of the paper's 12-20 km/s asteroid range and its example speed; no default or average is stated.
- PARTIAL IMP-021/022: densities 3000/7800/about 1000 vs the paper's 2000-3000, about 8000, 500-1500 (inside or at the edge).
- PARTIAL IMP-023: 2700 is the paper's value only below water and in the melt formula; its land targets are 2500 and 2750.
- PARTIAL IMP-001: the coded 5.6 +/- 2.8e-15 could not be read in Grieve & Shoemaker 1994; Grieve 1987 quotes 5.4 +/- 2.7e-15 (Grieve 1984).
- PARTIAL IMP-003/004: slope 1.8 is supported above about 20 km (Grieve 1987); the extension below 20 km is the code's own.
- PARTIAL IMP-066: Davison & Collins give a no-crater limit near R = 8 and a linear regression fitted at small R; the 6.3 is the code's own fit to three summary numbers.
- PARTIAL IMP-068: 3.8 km and 71 percent vs NOAA 3.68 km and the paper's 70 percent.
- PARTIAL IMP-014: Davison & Collins support the atmospheric cut-off near 100 m impactors; Bland & Artemieva 2006 itself unread.
- PARTIAL IMP-016: Vredefort is quoted as 170-300 km (secondary), so 300 km is the upper end.
- PARTIAL IMP-083: the only readable diffusion paper, Fassett & Thomson, is lunar (5.5 m2/Myr), about 1800 times below the coded 0.01 m2/yr; the fade times rest on the code's own value.
- Also noted: Davison & Collins 2007 and Collins et al. 2005 date Grieve & Shoemaker's chapter 1995 vs 1994 (book year 1994); the comment "constants were written from the papers as remembered" proved true for the 0.294 law and for the equation numbers, but the eq. 21, 22, 23, 24, 26, 27, 30, 31, 47, 48 values themselves are right.

Paywalled or unreadable sources needed for stage 2 (to settle the open rows): Grieve & Shoemaker 1994 (book chapter; the rate 5.6 +/- 2.8e-15 and slope), Pike 1980 and Pike 1985 (gravity scaling of the transition, central-peak 0.22 D; ADS scans have no text layer, so they need OCR or another copy), Alexopoulos & McKinnon 1994 (Venus peak rings at 40 km), Fernandes & Dietrich 1997 and Roering et al. 1999 (hillslope diffusivity 0.003-0.05 m2/yr), Culling 1960 (diffusion form), Neukum et al. 2001 (rate constant within a factor of 2), Bland & Artemieva 2006 (atmospheric cut-off), Le Feuvre & Wieczorek 2011 (open at HAL but access denied here), Herrick et al. 1997 and Pike 1980 (to trace 0.294 D^0.301), Moore et al. 1974 (blanket ends at 2.3 radii), Masaitis 1998, Grieve & Masaitis 1994 and Naldrett 2004 (deposit facts).

### Volcanoes and magma (VOL, MAG)

Sources cited by the code: 1 (Bremond1995): paywalled (abstract only); read in this pass: abstract fetched. Sources found for uncited figures: 7 papers or reports (Szabo1986, Poland2014, Przeor2022, Scaillet2008, Courtillot2003, Bryan2008, Hildreth2007), plus about 16 Wikipedia pages and 3 USGS pages. Access of the 8 papers: open access 3 (Poland2014, Przeor2022, Hildreth2007), paywalled 5. Read in this pass: abstract fetched 6 (Bremond, Szabo, Poland, Scaillet, Courtillot, Hildreth), search snippet only 1 (Przeor), not fetched beyond metadata 1 (Bryan).

Spot checks: 21 rows. MATCH 4 (VOL-032, VOL-005, hotspot track, MAG-020); PARTIAL 10 (VOL-039, flag 2, flag 5, VOL-019, VOL-021, VOL-022, lifetimes, MAG-002, VOL-052, flood-basalt frequency); MISMATCH 3 (flag 1 against Fuji, flag 3 subsidence, flag 4 caldera widths); COULD NOT READ 4 (VOL-007, VOL-020, VOL-023, VOL-036).

MISMATCH and PARTIAL, one line each:
- MISMATCH flag 1: real Fuji upper flanks are 31-35 degrees; the code comment says 20, the formula gives 15.
- MISMATCH flag 3: Hawaii island subsides 2.7 mm/yr (2,700 m/Myr, Szabo and Moore 1986), the code uses 25 m/Myr, the comment 67 m/Myr; "2 km over 30 Myr" not found.
- MISMATCH flag 4: the caldera comment's numbers are radii; real widths are Crater Lake 8-10 km, Long Valley 16 x 32 km, Yellowstone 55 x 72 km.
- PARTIAL VOL-039: Bremond1995 abstract says spacing is random with no characteristic value; no 50-70 km figure in the abstract.
- PARTIAL flag 2: Mauna Loa's steepest slope about 12 degrees against the comment's 10.
- PARTIAL flag 5: real summit craters about 0.5-1 km across; the code's 2-4 km is a collapse-crater size.
- PARTIAL VOL-019: Cotopaxi is about 2.1 km above its plateau, not 2.9; Rainier relief about 4 km exceeds the code range.
- PARTIAL MAG-002: Kilauea 1.2 and 3.5 km sources; Vesuvius shallow chamber 3-4 km, not 4-5.
- PARTIAL VOL-052: flood basalt area; sources disagree (Siberian Traps 1.5-3.9 versus 7 million km2); the coded 2.5 million km2 is above the Deccan.
- PARTIAL flood-basalt frequency: one per about 22 Myr by my division of Courtillot and Renne; not stated by them.
- PARTIAL lifetimes, VOL-021, VOL-022: only parts were verifiable.

Paywalled sources needed for stage 2: Bremond1995 (full text for the mean spacings and the four named arcs), Scaillet2008 (full text for the Vesuvius chamber depths by eruption), Szabo1986, Bryan2008 (flood-basalt area and frequency), plus the unfetched Self et al. 2007 (Deccan flow thickness; 10.1016/j.jvolgeores.2006.11.012) and a review for Galapagos and Reunion heights.

### Erosion, isostasy and hypsometry (ERO, ISO, HYP)

Part 1 has 36 entries (34 sources cited by the surface-stage code or docs, plus Telford1990 and Schon2015 from the rock-class density strings, which belong to another prefix), and lists 6 further works named in the brief but not cited by the code (Whipple and Tucker 1999, Fernandes and Dietrich 1997, Amante and Eakins 2009, Eakins and Sharman 2012, Athy 1930, Sclater and Christie 1980).

Sources by Access (36): open access 6 (Landlab, Peel2007, Charette2010, Wikipedia-Laterite, Tozer2019, Fastscapelib); open copy 6 (Barnes2014, Yuan2019, Harel2016, Cortial2019, Tzathas2024, Jain2024; of these only Barnes2014, Tzathas2024 and Jain2024 were retrievable by script); paywalled 16 (Braun2013, Bosscher1992, Heimsath1997, Roering1999, Sklar2001, Stock1999, Clark2000, Royden1997, England1982, Bird1991, Rey2001, Beaumont2001, Stein1992, Parsons1977, Cogley1984, Braun2016); not checked 2 (Schott2024, Wessel2019-GMT, Crossref record only); book, not online 5 (Ahnert1976, Hjulstrom1935, Molnar1988, Telford1990, Schon2015); not found 1 (Shepard, unidentified).

Sources by Read in this pass (36): full text fetched 6 (Barnes2014, Landlab source files, Charette2010, Tzathas2024, Wikipedia-Laterite, and the Fastscapelib header, one file only); abstract fetched 5 (Yuan2019 and Stein1992: first sentence only; Heimsath1997 in full; Jain2024; Peel2007 plus part of the PDF text); search result snippet only 0; not fetched 25. A Crossref record was found and compared for 31 of the 36 (all except Ahnert1976, Hjulstrom1935, Shepard, Wikipedia-Laterite and Fastscapelib).

Spot checks (28, S1 to S28), by primary result: MATCH 10 (S6, S7, S8, S9, S10, S12, S14, S17, S21, S27); PARTIAL 7 (S1, S2, S13, S20, S22, S23, S26); MISMATCH 3 (S5, S11, S18); COULD NOT READ 8 (S3, S4, S15, S16, S19, S24, S25, S28). Part 2 lines with two sub-results are counted once, by their main result.

MISMATCH and PARTIAL results, one line each:
- MISMATCH ERO-070 (S5): soil production decay depth H* = 5 m; secondary sources give about 0.25 to 0.5 m (k of 2 to 4 per m; 0.02 per cm), so the suspected factor of ten is confirmed from secondary sources, not from the 1997 text. ERO-069 w0 = 50 m/Myr is inside the secondary range of about 47 to 2,000 m/Myr. Landlab's own weatherer defaults are 1.0 and 1.0, so the code's numbers have no Landlab source.
- MISMATCH ERO-022 and ERO-049 (S11): the code says its face diffusivity is the arithmetic mean "as Landlab's LinearDiffuser"; Landlab master uses the maximum of the two nodes (`map_max_of_link_nodes_to_link`, upwind).
- MISMATCH HYP-010 (S18): "ocean mode -4 to -3 km" against NOAA ETOPO1 ("oceanic abyssal plains, roughly 4,300 meters below sea level") and against the project's own band table, whose peak (0.229) is the -5 to -4 km band.
- PARTIAL ERO-023 (S13): kappa 2 m2/yr is uncited; the only readable primary value (Tzathas et al., 0.1 m2/yr) is 20 times smaller; the literature range 0.001 to 0.01 m2/yr could not be confirmed (Roering 1999, Fernandes and Dietrich 1997 not readable).
- PARTIAL ISO-001, ISO-008, ISO-009 (S22): densities 2,700 / 2,800 against 2,750 (secondary, citing Kearey et al. 2009); mantle 3,300 matches.
- PARTIAL ISO-020, ISO-021 (S23): the strength-height arithmetic is right (5.1 km, 13.4 km at 0.38 g, 2.5 km at 2 g, computed by me), the 140 MPa has no source.
- PARTIAL ERO-003 to ERO-008 (S2): the unit convention m^(1-2m)/yr matches Tzathas; K 2e-9 / 3e-9, m = 1, n = 2 have no external source; Landlab defaults are K 0.001, m 0.5, n 1 and fastscapelib has none.
- PARTIAL ERO-001, ERO-078, ERO-079, HYP-030 (S1): the scheme is Braun and Willett's according to fastscapelib and Landlab headers; the paper itself was not readable.
- PARTIAL ERO-098 (S26): Hjulstrom (via Wikipedia) supports only the ordering of entrainment thresholds, not the 4 to 10 multipliers.
- PARTIAL HYP-028, HYP-029 (S20): fastscapelib is GPL-3.0 as stated, but it has no K, m, n defaults to compare against.
- Smaller notes: ERO-048 tidal range 2 m is the submarine diffuser's default, Landlab's carbonate default is 0.0; ERO-013 threshold slope 0.35 is neither Landlab's 1.0 nor Tzathas's 0.57; Molnar and Lyon-Caen pages are 179-208 in Crossref (coded 179-207); Shepard cannot be identified; the title of Roering et al. 1999 is about nonlinear transport.

Paywalled or unreadable sources needed for stage 2 (value-bearing claims left unchecked): Braun and Willett 2013; Yuan et al. 2019 (G); Stock and Montgomery 1999 and Whipple and Tucker 1999 (K, m, n ranges); Harel et al. 2016 (n > 1; the Edinburgh author manuscript returned 403); Heimsath et al. 1997 and later Heimsath papers (P0 and decay depth); Roering et al. 1999 and Fernandes and Dietrich 1997 (hillslope diffusivity); Sklar and Dietrich 2001; Stein and Stein 1992 (GDH1 coefficients behind ISO-011 to ISO-017) and Parsons and Sclater 1977; Clark and Royden 2000, Royden et al. 1997, Molnar and Lyon-Caen 1988 (ISO-018 to ISO-022); Cogley 1984; Bosscher and Schlager 1992; Cortial et al. 2019 (open at HAL hal-02136820 but blocked for scripts); Peel et al. 2007 (open; the global main-group shares need a manual look at the full text or supplement). The NOAA ETOPO1 hypsographic note's Table 1 is an image and would need a manual reading for the Earth land mean elevation (HYP-005).

### Rivers, lakes and the young landscape (HYD, YNG)

Sources by Access (33 distinct sources in Part 1: 22 main entries, 9 "mentioned for later", 2 candidates):
- open access (confirmed): 12. Leopold1953, Leopold1957, Barnes2021, Zhao2022, Ohmura1992 (page readable without login), Messager2016, Verpoorter2014 (hybrid), Peel2007, Zomer2022, Cordonnier2019, Gailleton2024, Liang2015
- open copy (preprint or agency host): 3. Barnes2014a, Barnes2014b, Lugo1999
- paywalled or closed (confirmed by OpenAlex): 4. Garbrecht1997, Porter2001, Hock2003, Wang2018
- only the abstract or record was reachable, open-access status not checked: 11. Zhang2004, Downing2006, Porter1989, Schumm1963, Schott2024, Genevaux2013, Paris2023, Braun2013, Yuan2019, Nobre2011, Fairfield1991
- book (not online): 2. Holdridge1967 (an unofficial scan exists), MiddletonThomas1997
- not found: 1. Fu1981

Sources by Read in this pass:
- full text fetched: 11. Leopold1953, Leopold1957, Barnes2014a, Barnes2014b, Holdridge1967 (passages), Lugo1999 (partial), Zhao2022, Peel2007 (all decoded from the PDF text layer, "primary"); Messager2016, Zomer2022, Ohmura1992 (through a summariser)
- abstract fetched: 7. Barnes2021, Zhang2004, Verpoorter2014, Downing2006, Porter1989, Schumm1963, Schott2024
- bibliographic record only: 13. Garbrecht1997, Porter2001, Hock2003, Wang2018, and the nine "mentioned for later" papers
- not fetched: 2. Fu1981 (not found), MiddletonThomas1997 (book; values taken from Zomer2022 and Wikipedia)

Spot checks (22 entries in Part 2): MATCH 10 (HYD-001, HYD-002, HYD-029, HYD-015, HYD-049/050, HYD-045, HYD-046, HYD-062/063 Verpoorter, HYD-063 Messager, YNG-010 for the 500 m); PARTIAL 7 (HYD-027, HYD-014, HYD-039, HYD-040, HYD-043, HYD-064, YNG-015); MISMATCH 1 (HYD-030); COULD NOT READ or no source found 4 (HYD-034, HYD-041, HYD-044, HYD-037; the 900 m part of YNG-010 is also unread).

MISMATCH and PARTIAL, one line each:
- HYD-030 MISMATCH: Holdridge's biotemperature also zeroes temperatures above 30 degC; the code has no upper limit, and 58.93 is defined against the two-sided quantity.
- HYD-027 PARTIAL: 1.5 as the meandering threshold is sourced (Leopold & Wolman 1957, arbitrary); "typical 1.1 to 1.5 and over" has no source.
- HYD-014 PARTIAL: the pit queue is in Barnes's Algorithms 2 and 3; Algorithm 4 is the flow-direction variant without one.
- HYD-039 PARTIAL: -2 degC follows from Ohmura's P = 645 + 296 T + 9 T^2 at 100 mm, but T there is a free-atmosphere summer mean with 200 mm scatter.
- HYD-040 PARTIAL: 1,500 +- 150 km3 without the Caspian is in Zhao; "650 mm over 2.3 million km2" is the project's division (641 mm per total lake area; 1,020 mm per open-water area).
- HYD-043 PARTIAL: Zhao supports high-latitude dominance of lake area; the "6-8 degC" has no source.
- HYD-064 PARTIAL: Caspian 371,000 to 389,000 km2 is the range of secondary figures; no primary source.
- YNG-015 PARTIAL: Schott et al. is about erosion-based amplification; the detail noise term is plain value noise.

Not found or not readable: HYD-034 (Fu/Zhang omega 2.6), HYD-041 (open-water evaporation by latitude), HYD-044 (polar lakes 150-350 mm), HYD-037 (3 to 8 mm per degC per day), YNG-010's 900 m at the LGM.

Paywalled or unread and needed for stage 2: Zhang2004 (omega), Garbrecht1997 (flat algorithm origin), Porter2001 (900 m), Schumm1963 (sinuosity classes), Downing2006 (full text, if its lake-share basis matters), Hock2003 and Wang2018 (candidates for HYD-037 and HYD-041/044). Fu1981 was not found at all. Ledger notes the next pass can use: the sources file's Earth-flag suspicion about Holdridge's upper limit is confirmed; the sources file's note that Leopold & Maddock's values "may be right" is confirmed (0.5 and 0.4 are the paper's representative downstream values).

### The amplifier and glacial landforms (AMP, GLA)

Sources: 20 distinct scientific sources in the sources file (Schott2024, Barnes2014, Tzathas2024, Hergarten2021, goSPL, Oerlemans2011, Cuffey2010, Harbor1992, Humphrey1994, Hallet1996, Sugden1976, Alley1998, Alley2003, Porter1989, Porter2001, Ohmura1992, Svensson1959, Sugden1978, Egholm2009, Mitchell2015), plus Argudo2020 and Cordonnier2023 (research note only) and two context papers used for checking (Cook2020, Prasicek2018).
- By access (the 20): open access or open copy 10 (Schott2024 code only, Barnes2014, Tzathas2024 whose open PDF I could not read, Hergarten2021, goSPL, Humphrey1994, Alley1998, Ohmura1992, Svensson1959, Sugden1978); paywalled 7 (Harbor1992, Hallet1996, Alley2003, Porter1989, Porter2001, Egholm2009, Mitchell2015); books 3 (Cuffey2010, Oerlemans2011, Sugden1976). Argudo2020 and Cordonnier2023 were not checked beyond metadata.
- By reading in this pass (the 20): full text fetched 4 (Ohmura1992 as page text with Table 3 missing, Hergarten2021, goSPL ice.rst, Schott2024 shaders only); abstract fetched 9 (Barnes2014, Harbor1992, Humphrey1994, Alley1998, Alley2003, Porter1989, Sugden1978, Egholm2009, Mitchell2015); search snippet only 1 (Tzathas2024); metadata only or not fetched 6 (Hallet1996, Porter2001, Svensson1959, Cuffey2010, Oerlemans2011, Sugden1976).

Spot checks: 26 rows. MATCH 10; PARTIAL 8; MISMATCH 1; COULD NOT READ 7.
- MATCH: GLA-037 coefficients, GLA-001/003 (H&R), GLA-010/011 (via Prasicek), GLA-018 ratio 1.5, GLA-030 500 m, GLA-057, GLA-062 (600 m), GLA-064, AMP-006 (flow_p 1.3), AMP-001 workflow.
- MISMATCH: AMP-038: Schott's 0.9 to 1.4 are absolute tan values in the noisified shader, not multipliers (as written).
- PARTIAL: GLA-037 definition of T and standard error; GLA-002 (1e-5 paired with l = 1; published pairs are 1e-4 with l = 1 and 1e-5 with l = 2); AMP-034 (0.57 is the non-default branch; the default noisified threshold is 0.9-1.4, tan 42 to 54 degrees); AMP-028 (the cap is on S^2); GLA-009 (Hergarten's law is stream power in ice flux, not sliding); GLA-022 (0.3 in Hergarten is an exponent; his ratio is 0.05; Aletsch gives 0.6); AMP-016; sinuosity.
- COULD NOT READ (7 rows): GLA-031 Ohmura's data range (Table 3 unreadable); GLA-002 Hallet rate list; GLA-003 Harbor l = 2; GLA-012 sliding constant B; GLA-054 Svensson b = 2.0-2.2 (and Alley 1998's 1.2-1.7); GLA-030 Porter 2001's 900 m (the paper is about the tropics); AMP-034 Tzathas 0.57. The grouped "Earth reference figures" row is also unread.

Paywalled sources needed for stage 2: Hallet, Hunter and Bogen 1996 (the rates list that sets K_g); Harbor 1992 (exponent l = 2); Svensson 1959 (b = 2.0-2.2, an open Cambridge page that I could only reach as metadata); Porter 2001 (900 m, and whether it is tropical only); Alley et al. 1998 Part II (1.2-1.7); Mitchell and Humphries 2015 (full text for the 600 m relief definition); Egholm 2009 (full text); Tzathas 2024 and Schott 2024 (the Inria and HAL PDFs, which were blocked or binary); Cuffey and Paterson 2010 (Table 3.4 and the sliding law); Oerlemans Minimal Glacier Models (beta); Herman et al. 2015 (the l = 2, 1e-5 pairing). Also worth fetching: the Schott UI source for the defaults the paper actually used (this pass only saw the shader).

### Rock, strata and structure (RCK, STR, DEF)

Counts. Distinct scientific sources entered: 38 (32 cited by the code, plus 6 added as evidence: Lathrop2022, Haederle2002, Mukasa1986, Alavi2007, Pollack1993, NoddyOriginal), of which 3 are software (Noddyverse, NoddyOriginal, tectonics.js).
- By access: open access 6 (Jessell2022, Wellmann2016, Lathrop2022, Noddyverse, NoddyOriginal, tectonics.js); open copy 3 (McQuarrie2004 author version, Jessell1981, Jessell1996; copies listed, not fetched); paywalled 21 (Sklar, Stock, Roering, Heimsath, Cruden, Bateman, Buddington, Hudec, Mouthereau, Biot, Sherwin, Blanc, Gawthorpe, Cowie1992a, Schlische, Kim, Weissel, Haederle, Mukasa, Alavi, Pollack); book (not online) 6 (Telford1990, Schon2015, Winter2010, Yardley1989, Warren2016, Anderson1951); not found 2 (Hjulstrom1935, Doust1990).
- By reading in this pass: full text fetched 5 (Jessell2022 page, Noddyverse files, NoddyOriginal pages, Lathrop2022, tectonics.js front page); abstract fetched 15 (Sklar, Stock, Roering, Cruden, Bateman, Mouthereau, Sherwin, Blanc, Cowie1992a, Schlische, Weissel, Mukasa, Alavi, Pollack, Wellmann); metadata only 8 (Heimsath, Buddington, Hudec, Biot, McQuarrie, Gawthorpe, Kim, Haederle); search snippet or secondary page only 5 (Jessell1981, Jessell1996, Winter2010 chapter titles, Anderson1951, Hjulstrom1935 via Wikipedia); not fetched 5 (Telford, Schon, Yardley, Warren, Doust).

Spot checks: 33 entries. MATCH 5 (RCK-043, RCK-006, DEF-001, DEF-019/020, DEF-073). PARTIAL 17 (including DEF-005 and DEF-063, which are MATCH on the named point and PARTIAL on the wider claim). MISMATCH 3 (RCK-061 thickness against Cruden abstract; DEF-016/017/018 slip ratios against the cited 0.03; DEF-042 coded stock radii against the 100 km2 definition). COULD NOT READ 8 (RCK-041/042 temperatures, RCK-045, RCK-060 area, RCK-063, RCK-022, DEF-004, DEF-013, STR-026/028).

MISMATCH and PARTIAL results, one line each:
- RCK-061 MISMATCH: Cruden 1998 abstract says granites 10-100 km wide are at most 3 km thick and gives no T = 0.6 L^0.6; the code gives 4.1 km at L = 25 km and 8.3 km at 80 km. Full text not read.
- DEF-016/017/018 MISMATCH: coded gamma 0.05 / 0.08 / 0.05 against the cited 0.03 (Schlische, D = 0.03 L^1.06 per Lathrop et al. 2022); all inside the reported typical 0.001-0.1, and no source found for thrust or strike-slip values.
- DEF-042 MISMATCH: the 100 km2 / 5.6 km definition is right (Wikipedia), but coded radii up to 10 km and elongation up to 1.6 give stocks of 100-500 km2.
- RCK-039/040 PARTIAL: slate 250 and greenschist 300 degC sit in the 200-320 and 300-450 degC bands of secondary sources; Winter ch. 21/25 not readable.
- RCK-044 PARTIAL: gneiss 650 degC is at the low end of the granulite lower boundary (700 +/- 50 degC); gneiss is a texture, not a facies.
- RCK-028/030 PARTIAL: mean continental heat flow 65 mW/m2 gives 26 degC/km with k = 2.5, agreeing with 25-30 degC/km; the cap of 120 mW/m2 (48 degC/km) and k = 2.5 are unsourced.
- RCK-056 PARTIAL: 1,600 km length and about a thousand plutons supported; 60 km width, 60 Myr and the 1e-4 figure not found, and the code's own arithmetic gives 1.7e-4.
- RCK-007 PARTIAL: Stock & Montgomery support orders-of-magnitude K contrasts; the code's 0.3-4 range is a designed compression.
- RCK-008 PARTIAL: Hjulstrom supports ranking loose grains by entrainment but not bedrock-style K multipliers; "silt moves at the lowest speeds" is not what the curve shows (cohesion).
- RCK-010 PARTIAL: Roering/Heimsath establish a diffusivity and soil production function, not lithology multipliers.
- RCK-001 PARTIAL: granite 2650 and basalt 2900 agree with Wikipedia; the Telford/Schon tables could not be read and 18 other densities are unchecked.
- RCK-015 PARTIAL: CCD 4500 m sits within 4200-5000 m (Pacific 4200-4500, Indian about 4300, Atlantic about 5000).
- RCK-069/071 PARTIAL: eight lava-lake volcanoes listed; chamber depths and the 1,400 count unconfirmed.
- DEF-005 PARTIAL (range): 16-19 % and 16-30 % shortening support an upper bound near 0.30; 0.08 is a design value; McQuarrie's 20 % not read.
- DEF-002 PARTIAL: Biot's theory was tested on 818 natural fold wavelengths but did not apply directly.
- DEF-015 PARTIAL: 0.03 is a reviewer's quote of Schlische's fit with exponent 1.06; abstracts give no number.
- STR-030/DEF-027 PARTIAL: Zagros deformation zone 200-300 km wide (Alavi); 250 km fits, the 300 km in STR-030 is the upper end; Subandean width not found.
- DEF-059 PARTIAL: Weissel & Karner confirm flexural uplift of rift flanks; 0.2 is a design value.
- DEF-063 PARTIAL (beyond the named repo): the original Noddy has been distributed as a free-distribution notice (tectonique.net), GPLv3 (SourceForge) and GPL-2.0 (pynoddy), then MIT (Loop3D, 2021, and the 2022 ESSD paper); the mod's LGPL port of the MIT noddyverse snapshot is compatible if the MIT and Monash notices are kept.
- DEF-010/038/066 PARTIAL: only the 0.001 tolerance of unEvents.c was seen.

Paywalled or offline sources needed for stage 2 (the ones whose values could not be read):
- Winter 2010 ch. 21 and 25 and Yardley 1989 ch. 3 (all facies and grade temperatures; slate, phyllite, schist, gneiss); Warren 2016 ch. 7 (gypsum to anhydrite 50 degC); Telford et al. 1990 and Schon 2015 (density tables); Anderson 1951 (dips).
- Cruden 1998 full text (is T = 0.6 L^0.6 in it?) and, as a lead, McCaffrey & Petford 1997 (J. Geol. Soc. 154:1-4, from my recall, unverified); Bateman & Chappell 1979 (Tuolumne size); Buddington 1959 (roof depth); Mukasa 1986 and Haederle & Atherton 2002 or Pitcher et al. 1985 (Coastal Batholith width, pluton count, duration).
- Schlische et al. 1996 and Kim & Sanderson 2005 full text (gamma 0.03 and the spread); Cowie & Scholz 1992a (gamma values); Gawthorpe & Leeder 2000; Blanc et al. 2003; McQuarrie 2004 (Caltech author copy, 20 %); Hudec & Jackson 2007; Doust & Omatsola 1990 (AAPG Memoir 48).
- Hjulstrom 1935 (to see the minimum-velocity grain size and the silt statement).
- The tectonics.js source for the 2.2 MPa lithification pressure.
- The Noddy question: confirm the original Monash/Ewen Christopher matrix.c notice and, if the LGPL claim matters, whether any file in the noddyverse commit derives from the GPLv3 SourceForge release.

### Deposit models (DEP)

Sources: 75 entries in Part 1. Two are grouped catch-alls (CoalMisc, Software) holding 12 further items: 4 coal reports and 2 USGS programs identified, 5 coal-note items and the EIS Toolkit UNIDENTIFIED.

By Access:
- open copy (agency report, dataset, web page, preprint or author copy): 35
- open access (journal): 1 (Hoggard2020)
- paywalled (abstract only or not read): 36
- book (not online): 3 (SingerMenzie2010, Titley1995, AbramowitzStegun1964)
- not found: 0

By Read in this pass:
- full text or full data fetched: 22 (Singer2008a, John2010, OrrisBliss1986, Schmidt2007, MosierSinger1986, Cannon1986, Cannon1995, Singer2009, Leach2010a, MosierBriskey1986, Emsbo2016, MenzieMosier1986, duBray1995, Singer2001, USGS2000, Wood1983, Ellis1999, Moore2003, Newell1997, Acklam, NMRA1998data, Tuttle2000)
- abstract fetched: 21 (Singer2005, Singer2008b, SillitoeMcKee1996, Waples1980, Dyman2002, Kesler2006, Kesler2008, Singer1993, Singer1995, Butterworth2016, Richards2003, Cooke2005, Bekker2010 (cut off), Bohacs1997, Hoggard2020, Leach2005, Leach2010b, Garven1984, Garven1985, Sillitoe2005, Sillitoe2010)
- not fetched (metadata only or reference string only): 32

Spot checks (51): MATCH 35; PARTIAL 10; MISMATCH 2; COULD NOT READ 4 (S19, S35, S43, S51).

MISMATCH and PARTIAL, one line each:
- MISMATCH S12: Circular 1178 "p. 15" is printed p. 5 (the 1 km depth rule and the "any part within 1 km counts the whole deposit" rule are correct; the "1-3 km" in the research note mixes it with a remark about development to 3 km).
- MISMATCH S29: the placer type note says no USGS placer density exists; PP 1640-A reports 0.0054 and 0.0061 placer-gold deposits per km2 in two Alaskan areas (unused child type, no engine effect).
- PARTIAL S3: Mo/Au/Ag presence counts 228/256/172 not reproduced (232/249/176 on the downloaded data file, which has 427 tonnage records against the report's 422); the percentiles match.
- PARTIAL S9: Singer et al. 2005 densities (4/15/35) are deposits plus prospects per 100,000 km2 of exposed permissive rock and fall with tract size; the engine applies a flat 15 or 17.8 to the whole tract (the size is right at the paper's 76,000 km2 example, 14.5).
- PARTIAL S10: the 17.8 mean is the project's own split-lognormal result from 4/15/35 (it reproduces), not a published number.
- PARTIAL S11: area dependence confirmed by the Singer 2008b abstract; the equation (0.14 x Area^-0.61) was seen only as a search snippet; Singer and Menzie 2010 not read.
- PARTIAL S13: the Sillitoe 2005 abstract has both "at least 0.5 m.y." and "typically minima of 3 to 9 m.y."; the data file's claim that 3-9 was not found is wrong, and the engine uses only the lower bound.
- PARTIAL S15: the 100 mm per year lower bound is stricter than the sources (SIR B: one Andean stage inferred at 10 cm/yr or more; Sillitoe 2005: semiarid to pluvial, and arid conditions in northern Chile).
- PARTIAL S31: sedex salinity "17-30" mixes the observed 10-30 wt percent TDS with the permissive minimum of more than 17 percent; everything else in that row matches.
- PARTIAL S33: the 2.02 Ga onset is from Leach et al. 2010 (Economic Geology 105:593-625), not SIR 2010-5070-A; the same abstract says only clastic-dominated (sedex) deposits were important before the late Neoproterozoic, so gating MVT on the same clock is not supported.
- PARTIAL S41: the coal field size (3,800 / 19,800 / 115,000 Mt) and density (4 / 14 / 40) have no published source (project judgement); the cited inputs check out.
- PARTIAL S46: the 80 C source-rock threshold is the lower bound of an oil window quoted in one regional study (OFR 2003-253), not a general USGS rule.

Resolved doubts in the earth file (the source supports the code): flag 4 (0.80 is a figure tick, 0.75 is the Cu p10); flag 14 (the Cu-Mo 4,800 Mt is printed in both the table and Fig. 17, not a typo; SHam, CAam and Bulletin 1693 31a all match; SIR N's "1.4 to 120 Mt" is a loose restatement of the printed 1.7 and 130); flag 3 (Circular 891 Table 3 is on p. 24; coal_seam.json also writes p. 14 once).

Other citation notes: Singer2001 authors differ between the PDF title page (Singer, Menzie, Sutphin, Mosier, Bliss) and the USGS catalogue and Crossref (Menzie, Sutphin, Mosier, Bliss, Schulz; editor Singer); John2010 is an edited volume (John as editor); Garven and Freeze 1984 is a two-part paper and the base case cited was not readable; "Leach et al. 2010" in the code names two different papers; the task's "Mosier, Berger & Singer 2009" is Singer, Berger and Moring 2009.

Paywalled sources needed for stage 2 (full text not available here):
- Singer2005 (full text, for the regression coefficients and control-area sizes; only the abstract read), SingerMenzie2010 (book), Singer2008b (the equation).
- Sillitoe2005 (pp. 723-768; only the abstract read), SillitoeMcKee1996, Sillitoe2010.
- Garven1984 (both parts; the 1 km over 300 km base case), Garven1985, Leach2001, Leach2005, Leach2010b, Bradley2003, Bethke1990, Oliver1986 (the MVT head-gradient driver and brine reach).
- Kesler2006 and Kesler2008 (survival model details).
- Bekker2010 (age window), Butterworth2016, Richards2003, Hoggard2020 (open, but only the abstract read).
- Waples1980 (full paper; only the OSTI abstract read), Dyman2002.
- Agterberg1995, Robert2005 (30-50 km camps), Skirrow2019, McCuaig2010, Hronsky2008, Root1992.
- Not opened though open and relevant: Bulletin 2004, OFR 85-213, PP 1698-B, DDS-69-B chapters 4 and 5, and the five unidentified coal-note items.

### Deposit formation, coal, petroleum, placers (ORE)

Counts (counted by grep on this file). Sources in Part 1: 70 entries: 67 cited by the code (4 of them UNIDENTIFIED: NMET-Jharia, SAIMM-coalfields, SilesiaMDPI, RuhrCounts; 2 only probable: Hampson1999, Abramov2004), plus 3 not cited by the code (Lopatin1971, Basuki2002, Gumsley2017).

Access (entries): open access 25; open copy 2 (Hoggard2020 preprint, Cooke2005 figshare); paywalled (abstract only) 34; book (not online) 4; not found 5 (NMET-Jharia, SAIMM-coalfields, SilesiaMDPI, RuhrCounts, Lopatin1971).

Read in this pass (entries): abstract fetched 17; full text fetched 5 (Newell1997, BAP2018, GABowen, Nelsen2016, Cox1986); not fetched 48 (Crossref or OpenAlex or USGS-record metadata only); search result snippet only 0 (no search tool).

Spot checks (42 result lines; some cover two ledger ids): MATCH 16; MISMATCH 1; PARTIAL 14; COULD NOT READ 11. Evidence is primary full text for the Bulletin 1693 placer and sedex pages, primary abstract for Waples 1980 (OSTI), Ryer and Langer, Bohacs and Suter, Garven 1985, Basuki, Hoggard (preprint), Gumsley, and secondary (KGS, Wikipedia, SLB glossary, Geoscience Australia) for the rest.

MISMATCH and PARTIAL results (one line each):
- MISMATCH ORE-123: Illinois Basin coded 36,800 mi2 / 95,000 km2; Wikipedia gives the basin as 80,000 mi2 / 210,000 km2 (the coded figure is unverified and may be a coal-bearing area, not the basin).
- Reference MISMATCH ORE-118/120/128: "Ellis et al. 1999" is not the author of PP 1625-A chapter PS (Flores and Bader); PN and PG are Ellis et al.
- Reference MISMATCH ORE-001/003/014: "Leach et al. 2010" names two different papers (USGS SIR 2010-5070-A by Leach, Taylor, Fey, Diehl, Saltus; Economic Geology 105:593 by Leach, Bradley, Huston, Pisarevsky, Taylor, Gardoll).
- PARTIAL ORE-121: the Bioregional Assessments page gives 24,625 km2 and three main coalfields for the Sydney Basin bioregion, not 36,000 km2 and five (page may be truncated; Wikipedia gives 36,296 km2 for the basin).
- PARTIAL ORE-003/004: 60-100 degC is Garven's model output; 75 degC is the lower end of measured fluid inclusions (Basuki); 90-200 and 75-200 degC unconfirmed.
- PARTIAL ORE-014: 2.02 Ga is the earliest known MVT deposit (Leach 2010 abstract); the extension of the gate to SEDEX is not in the abstract.
- PARTIAL ORE-016: BIF peak 2.5 Ga and disappearance near 1.85 Ga confirmed (Wikipedia); the 2.65 Ga start unconfirmed; 1.85 Ga is also a secondary peak.
- PARTIAL ORE-015: Nelsen 2016 says "Coal deposits occur by the Early Devonian", but the paper is about the coal peak, not its onset.
- PARTIAL ORE-155 (anthracite at TTI 500): by the KGS table TTI 500 is about Ro 2.6 %; ISO medium rank A ends at 2.0 %, an open review says anthracite starts at Ro greater than 2.6 %.
- PARTIAL ORE-152: KGS gives Ro 1.1 for TTI 75 as peak oil, not as a coal rank limit.
- PARTIAL ORE-158: doubling per 10 degC confirmed; the 100-110 degC zero point of n not seen.
- PARTIAL ORE-198: the KGS table matches the code exactly (including TTI 1,500 gives about 3.8); the KGS value itself looks high for a wet-gas limit.
- PARTIAL ORE-165: the glossary gives about 0.5 wt% as the minimum source-rock TOC; the code uses 1.
- PARTIAL ORE-057/058: porphyry density 2.6 and 2.7 both sit in the granite range; the 0.6 km2 footprint and the 1-1.5 km height could not be read; Wikipedia gives 2 km average vertical thickness for a porphyry system.
- PARTIAL ORE-188/189: crude density 0.85 and porosity 20 % are within the quoted ranges; the 75 % saturation has no source.
- COULD NOT READ ORE-207/208/212: no source found for gold transport distance (5 km, 30 km) or placer gradient (5 m/km); the tin (<8 km) and diamond (1,000 km) figures are confirmed in Bulletin 1693.

Paywalled sources needed for stage 2 (highest value first):
- Garnett and Bassett 2005 (Economic Geology 100th anniversary volume): placer gold transport distance and concentration efficiency.
- Leach et al. 2005 (Economic Geology 100th anniversary volume, 561-607) and the body of Leach et al. 2010 (SIR 2010-5070-A, free but unreadable in this pass): MVT fluid temperature (90-200 degC) and the 600 km criterion.
- Bekker et al. 2010: BIF acme dates (2.65-1.85 Ga).
- Dyman et al. 2002: the deep-gas depth definition.
- Heckel 1986 and Rasbury et al. 1998: cyclothem durations.
- Waples 1980 full text: the TTI-Ro correlation and the n = 0 interval (the abstract does not include them).
- The USGS PDFs, which are open access but could not be parsed by this toolset and need a PDF-capable reader: SIR 2010-5070-A and -B, PP 1625-A chapters PS/PG/PN, PP 1625-D, DDS-60 Niger Delta AU 71920101, PP 1698 chap. B, OFR 88-450-G, OFR 94-220, DDS-69-B chaps. 4 and 5, OFR 2003-253, Circular 891 tables 2 and 3.

### Moisture, Köppen classes and biomes (MOI, BIO)

Sources by Access (44 distinct sources in Part 1: 24 main entries, where "Fu1981 and Zhang2004", "MiddletonThomas1997 and Zomer2022", "Fick2017 and ODonnell2012" and "Minder2010 and Rolland2003" count as two each, plus 20 design-only references):
- open access, open copy, or open web/software (confirmed): 14. Peel2007, Beck2018, Beck2023, BernerKothavala2001, Berner1994, Ohmura1992, Zomer2022, ODonnell2012, Tozer2019, Felzenszwalb2012, HeldSoden2006 (GFDL copy), FastscapeLEM (MIT software), WikipediaKoppen (web page), WorldbuildingPasta (blog)
- paywalled or closed (OpenAlex): 4. Thornthwaite1948, Willmott1985, Fick2017, Minder2010
- only the record or abstract was reachable, open-access status not checked: 22. SmithBarstad2004, Zhang2004, Rolland2003, and 19 of the 20 design-only references (all but NorthKim2017)
- book (not online): 2. MiddletonThomas1997, NorthKim2017
- not found: 2. Fu1981, ICAO1993

Sources by Read in this pass:
- full text fetched: 9. Peel2007, FastscapeLEM (source files), HeldSoden2006, BernerKothavala2001 (decoded from the PDF text layer or raw source: primary); Beck2018 (Table 2), Zomer2022, WorldbuildingPasta, WikipediaKoppen, Ohmura1992 (through a summariser)
- abstract fetched: 4. SmithBarstad2004, Zhang2004, Minder2010, Rolland2003
- bibliographic record only: 27. Beck2023, Thornthwaite1948 (its formula was checked in an implementation), Willmott1985, Berner1994, Fick2017 (plus the WorldClim variable page), ODonnell2012, Tozer2019, Felzenszwalb2012, and 19 design-only references
- not fetched: 4. Fu1981, MiddletonThomas1997, ICAO1993, NorthKim2017

Spot checks (28 entries in Part 2): MATCH 16 (BIO-003, BIO-004, BIO-005, BIO-006/007, BIO-008/009/010, BIO-011, BIO-012, BIO-014/015, BIO-016, BIO-017, Peel Earth shares, MOI-034 formula, MOI-035, MOI-036 to MOI-040, MOI-064, BIO-083); PARTIAL 8 (BIO-013, BIO-038, BIO-095, MOI-043, MOI-051, MOI-065, MOI-001, MOI-025/Pasta rows); MISMATCH 2 (BIO-002, MOI-073); COULD NOT READ 2 (MOI-034 equation 49, MOI-068).

MISMATCH and PARTIAL, one line each:
- BIO-002 MISMATCH: Peel says the B climates "must be identified first"; the code tests E first, so cold dry cells are ET/EF where Peel gives BW/BS.
- MOI-073 MISMATCH: 0.038 per kelvin is GEOCARB II (Berner 1994); Berner & Kothavala 2001 (GEOCARB III) use 0.045 (cold) and 0.025 (warm).
- BIO-013 PARTIAL: s/w thresholds match; Peel's tie rule gives w when both pass and summer is wetter, the code always gives s.
- BIO-038 PARTIAL: 0.03, 0.2, 0.5 are UNEP 1997; the anchors 1, 2, 5 are not UNEP (its humid class starts at 0.65); UNEP's index uses Penman-Monteith PET.
- BIO-095 PARTIAL: BIO15 is named as in the code; the formula (n-1) was not on the pages read.
- MOI-043 PARTIAL: fastscape's rainfall_frequency default is 1 (hour per day), the code uses 0.5.
- MOI-051 PARTIAL: 6.5 is the ICAO standard; observed near-surface annual lapse rates in the Cascades and Alps are 3.9 to 5.8 degC/km.
- MOI-065 PARTIAL: Thornthwaite's heat index and polynomial match a secondary implementation; the 26.5 degC branch is unread.
- MOI-001 PARTIAL: Hadley/Ferrel edges are quoted at 25 to 30 and 35 to 60 degrees; 30 and 60 are round numbers.

Refutations of the earth/MOI.md flags: flag 1's worry about the source chain is resolved at the threshold level (the coded thresholds match Peel's Table 1 except the items above); flag 2 (-38 degC) is a match; flag 3 (the -3 degC option) is not in Peel; flag 5 (summer defined by hemisphere) is wrong, Peel defines it by the warmer half-year; flag 4 (E before B) is confirmed as a mismatch; flag 6 (UNEP 0.05) is the 1992 value, 0.03 is the 1997 value; flag 13 (fastscape defaults) is wrong, the defaults and the 7/4 are in fastscape_ext.py.

Paywalled and needed for stage 2: SmithBarstad2004 (equation 49 and the parameter table), Thornthwaite1948 and Willmott1985 (the 26.5 degC branch), Zhang2004 and Fu1981 (omega), Fick2017 (the BIO15 formula), the UNEP atlas (book) for the class limits, and Minder2010 (full text, if the observed lapse rate matters). The two Peel-adjacent items most worth fixing in the code from this pass are BIO-002 (test order) and BIO-013 (the Cs/Cw tie rule).

### Scale, provinces, pole cap and bake (SCL, PRV, POL, BAK)

Sources: 16 scientific entries (the USGS map; Cortial2019; Mallard2016; Swinbank2006; Gonzalez2010; Keinert2015; Kesler2008; Coltice2019; Gough1981; CRUST 2.0 / Bassin2000; Mooney1998; NASA Earth Fact Sheet; Peel2007; UNEP1992; Koerner2004; Connelly2012), plus the project's own documents and game software under "Not bibliographic".
- Access: open access 4 (Coltice2019, Peel2007, NASA sheet, CRUST 2.0 data); open copy 4 (USGS map image, Cortial2019 author PDF, Gonzalez2010 on arXiv, Gough1981 ADS scan); paywalled 7 (Mallard2016, Kesler2008, Keinert2015, Swinbank2006, Mooney1998, Connelly2012, Koerner2004; the last four not tested, so "likely"); book 1 (UNEP1992); not found 0.
- Read in this pass: full text fetched 3 (Coltice2019 by keyword search only, NASA sheet, CRUST 2.0 data); abstract fetched 4 (Mallard2016, Kesler2008, Gonzalez2010, Peel2007); search snippet or secondary only 4 (USGS map, UNEP1992, Koerner2004, Connelly2012); not fetched 5 (Cortial2019, Swinbank2006, Keinert2015, Gough1981, Mooney1998; Crossref metadata only).

Spot checks (29 entries): MATCH 11 (PRV-002/003/004 oceanic ages, PRV-021 150 km, SCL-041 Coltice, SCL-023 day length, SCL-024 sidereal factor, circumference and area, plus arithmetic-only checks SCL-005/030, SCL-043, POL-017); PARTIAL 14 (PRV-001, the Earth province shares, SCL-030, SCL-006, SCL-039 age, radius, gravity, tilt, ocean fraction, BAK-020 twice, BAK-021, BAK-025, BAK-027, BAK-017 counted where listed); COULD NOT READ 4 (PRV-010 to PRV-020 class definitions, SCL-029/SCL-034 Cortial constants, SCL-039 Gough equation, SCL-044 Fibonacci equation); MISMATCH 0 outright (the nearest are the BAK-020 stand-in that misses UNEP 0.5, and the unverified 3.3 km and 10^7.6 km^2 figures).

MISMATCH and PARTIAL results in one line each:
- PRV-001: USGS class names match, but the map's orogens are "Phanerozoic" and its basins "foreland or intracratonic"; assignment rules and continental age cut-offs not found.
- USGS map identity: the code's "Geologic provinces of the world" is the USGS Earthquake Hazards Program crustal-structure page map (earthquake.usgs.gov/data/crust/maps.php, now 404, archived 2020); it is not USGS OFR 97-470 (that is the World Energy Project petroleum-province series); no author or year recoverable.
- Earth province shares: my own CRUST 2.0 name-based classification gives cratons 0.75 (repo 0.71), orogen + arcs 0.18 (repo 0.17), extended + rift 0.066 (repo 0.07); consistent, but no published table exists and the repo's sub-agent method is unrecorded.
- SCL-030: Mallard abstract supports "seven large plates plus fractal smaller plates"; the 10^7.6 km^2 (about 40 M km^2) split is unverified.
- SCL-006: Kesler & Wilkinson abstract has no 3.3 km mining-reach figure; "within mining reach" is the report's inference unless the body says it.
- SCL-039: 4.57 Gyr matches the solar-system age (CAI 4.5673 Ga); Earth is 4.54 +- 0.05; the Gough equation itself unread.
- Earth radius 6370 vs NASA volumetric mean 6371.000 km; gravity 9.81 vs NASA mean 9.820 (standard 9.80665); axial tilt 23.4 vs 23.44; ocean fraction 0.70 vs 0.708 to 0.71 (and build-log land 0.289 vs 0.292): all small rounding gaps.
- BAK-020: UNEP 0.2 and 0.5 confirmed, but the coded stand-in 0.42 P/(20T+140) gives 0.21 and 0.42, so it misses 0.5; Koppen threshold only the mid-seasonality variant.
- BAK-021 / BAK-025 / BAK-027: Koppen 10 degC (tree), 0 degC (polar), 18 degC (tropical) criteria confirmed as conventions; code ramps are wider and the ecological treeline is about 6 degC seasonal mean; boreal -10 degC and the 85 % snow share have no standard.
- BAK-017: shelf colour stop 120 m vs quoted shelf break about 140 m (not a physical claim).

Paywalled or unreachable and needed for stage 2: Cortial2019 (Appendix A, Table 3: R, delta t, trench depth, 500,000-sample default; author PDF is over the tool's 10 MB limit, so needs another route), Mallard2016 (the 10^7.6 km^2 split), Kesler2008 (the 3.3 km figure), Gough1981 (the luminosity equation; ADS scan needs OCR), Keinert2015 and Swinbank2006 (the point-set formula), Peel2007 Table 1 (PDF unreadable to the tool), and the archived USGS crust maps page (web.archive.org blocked for the tool; needs a fetch route that reaches the Internet Archive).

### Plate tectonics (PLT)

Sources in Part 1: 67 entries (66 papers, books and datasets, one software entry). By access: open access or open copy 16; paywalled (abstract only or less) 44; book (not online) 5 (Shepard1963, Shoemake1992, Turcotte & Schubert, Buck2004, Molnar & Lyon-Caen is counted under paywalled); not found 1 (Lallemand, Encycl. Marine Geosciences, year and title unresolved). (Counts are by the Access line; the book count and the paywalled count may overlap by one entry.)

By what was read in this pass: full text fetched 4 (Cortial2019 all sections and tables; King & Adam 2014 introduction and method; Nance 2022 targeted; Li et al. 2024 by grep); abstract fetched 27; metadata only (reference confirmed against Crossref/OpenAlex, no content) 31; not fetched 5-6 (Shepard, Turcotte & Schubert, Lallemand encyclopedia entry, tectonics.js, plus entries read only through a secondary source). No search-snippet-only reads (the search tool was unavailable).

Spot checks: 53 results over the Part 2 bullets (some bullets hold two results): MATCH 28; PARTIAL 14; MISMATCH 1; COULD NOT READ 10 (GDH1 coefficients, GDH1 law comparison, GDH1 heat flow, Zahirovic speeds, RMS 40 mm/yr, F&U factor of 8, collision rate and duration, arc distance and slab dip, Stern/Bradley numbers, orogen belt widths and Wilson cycle length). Evidence levels: primary full text 26 (all Cortial); primary abstract 18; secondary source 4; none 4.

MISMATCH and PARTIAL, one line each.
- MISMATCH: the ratio "3 to 4" (slab-attached over other plates) is attributed to Forsyth & Uyeda 1975 in check.rs, params.rs and two designs; their abstract has no ratio. The "about 4 times faster than nonsubducting plates" is in Conrad & Lithgow-Bertelloni 2004.
- PARTIAL: PLT-029 slab-pull epsilon 0.002: the paper gives only "epsilon much less than 1", no number; the code value is the project's own.
- PARTIAL: PLT-082/083 rift lambda_0 0.012 attributed to "Cortial section 4.4": the formula matches but the paper gives no value.
- PARTIAL: PLT-036 f(d): the ledger calls the cubic "the implementation's own"; the paper itself says "piecewise cubic", but gives no control distance (300 km is the project's).
- PARTIAL: PLT-071/073 collision surge: form matches; the paper's surge is uncapped, so the 1,400/1,100 m caps are a departure. The collision counter that the designs call a bug follows the paper's own "d += ||s(p)|| dt".
- PARTIAL: Cortial constants in the presets: 12 default values match Table 3 exactly, but pipeline presets depart on r_s (800), r_c (2000), z_r (-2500), eps_o (0.12), eps_c (0), u_0 (0.42/0.34), z_t as floor (-5600).
- PARTIAL: PLT-092 Parsons & Sclater: intercept 2,500 m and coefficient 350 appear in the only copy read (King & Adam 2014), but an unexplained 0.01 factor in that extraction leaves the coefficient scale unsettled.
- PARTIAL: PLT-188 keel_drag: the "2-5 times" is real but is a mantle-flow traction ratio (deep roots against a 100 km lithosphere), not continental versus oceanic drag; "factor of 8" is not in the Forsyth & Uyeda abstract.
- PARTIAL: PLT-186 slab_suction_share 0.3: only the low-viscosity-asthenosphere case in Conrad & Lithgow-Bertelloni 2004 gives about 30 percent; the baseline is 40-60 percent.
- PARTIAL: PLT-187/183 Tibet force 6-7e12 N/m (Molnar & Lyon-Caen): that abstract has no number; Copley et al. 2010 give 5-6e12; ridge push 2.5e12 (Copley) and 3e12 (Li et al. citing Turcotte & Schubert) are inside the 2-3.5e12 range; gross slab pull 1-3e13 supported only as an order of 1e13.
- PARTIAL: PLT-192 wedge taper 1.5 degrees: Davis et al. 1983 citation is correct but no taper angle is in the abstract and a secondary summary gives 2.5 degrees and above; value unverified, suspect.
- PARTIAL: PLT-180 uplift: Nocquet 2016 reports up to ~2.5 mm/yr (design says 0.5-2); the 0.1-1 mm/yr orogen mean has no source.
- PARTIAL: PLT-145/PLT-203 plate count: "about 14" is Bird's count of large plates (of 52); not the same as seven plates above 5 percent.
- PARTIAL: Earth mean land height about 800 m: matches the old Kossinna-type 875 m that Cogley 1984 says is too high; Cogley's own mean is about 120 m including submerged crust.
- PARTIAL: crust density and thickness: 2,800 kg/m3 is near Christensen & Mooney 2,830 but the Airy factor uses 2,700; the code column of 35.1 km follows Cogley (36 km), Christensen & Mooney give 41.1 km.
- PARTIAL: hotspot counts: 49 (Courtillot et al., read in King & Adam) and 7 primary (Wikipedia) against the code's 8; no cited source in code.
- PARTIAL: East Pacific Rise 150 mm/yr: a regional maximum near Easter Island (about 60 on its northern end).
- PARTIAL: supercontinent period: Nance 2022 gives ca. 500 Myr; design's Earth range 300-500 not found.
- Reference problems: Mercier de Lépinay et al. 2016 is a paper on transform margins (cited for convergent margin length); Royer & Gordon 1997 is on diffuse boundaries in the Indo-Australian plate (odd fit for the supercontinent-heating context); Tremblay et al. 2015 abstract is about erosion shutting down at about 10 Ma (design uses it for exhumation above 1 mm/yr at 17-12 Ma); Molnar & Lyon-Caen pages are 179-208 not 179-207; the "Lallemand" encyclopedia entry is unidentified; Lenardic et al. 2011 could be two papers (GGG 2011 or EPSL 2005).

Paywalled sources needed for stage 2 (full text, in order of value): Stein & Stein 1992 (GDH1 equations, the age-depth law); Forsyth & Uyeda 1975 (the 3-4 ratio, the factor of 8, the drag contrast; OUP PDF returns 403); Zahirovic et al. 2015 (speed groups 85 and 26-28 mm/yr); Turcotte & Schubert Geodynamics (gross slab pull and ridge push, which edition); Davis, Suppe & Dahlen 1983 (taper values); Conrad & Lithgow-Bertelloni 2004 and 2006 (the 4 times ratio in context, root traction); Molnar & Lyon-Caen 1988 (Tibet force); Bradley 2008 (passive margin lengths and ages); Stern 2002 and 2004 (arc crust build time, initiation); Parsons & Sclater 1977 (to settle the 0.01 factor); Mallard et al. 2016 (large-plate size split); Courtillot et al. 2003 (hotspot types and count).

### Climate physics (CLM)

Counts.

Sources in Part 1: 58 entries (the heading count 62 minus 4 group headings). Not counted: the 5 secondary sources listed together under "Secondary sources used for A and B" and the 4 software entries listed together.

Access (58 entries):
- open access or open copy: 28 (Rose2018, Laskar2004, Bergman2004, Archer2009, Berner1994 [scan], Berner2001, Self2006, Byrne2014, Hoffman2017, Fretwell2013, Morlighem2017, Ohmura1992, gosplDocs, Ziegler2021, RoseMarshall2009, Ferreira2018, Trenberth2001, Stassen2019, Bretherton2004, Bischoff2014, Wagner2015, Serreze1992, Ramirez2024, Beck2018, Beck2023, ETOPO2022, Peel2007, Rohde2020). Several of these are open on paper but the publisher blocks automated download (A&A, AMS).
- paywalled: 15 (North1981 [scan at a course page, unreadable], Berger1978, Hoffman1998, Rowley2002, Bird2003, Walker1981, Myhre1998, Gough1981, Held2006, Nye1952, Imbrie1980, Lindzen1987, Manabe1969, Siler2018, Klein1993).
- book (not online): 2 (Cuffey2010, NorthKim2017).
- not found or unidentified: 1 (Tetens).
- access not checked: 12 (Abbot2010, Zhuang2017, North1983, Stommel1948, Dommenget2011, Fanning1996, Weaver2001, Tozer2019, Rayner2003, Felzenszwalb2012, Adler2018, Kopp2011).

Read in this pass (58 entries):
- full text fetched: 11 (Rose2018 [source files only], Bergman2004 [text, not tables], Berner2001, Byrne2014, Hoffman2017, Fretwell2013, Morlighem2017, Ohmura1992 [text, not the regression equation], gosplDocs, Peel2007, Rohde2020 [data file]).
- abstract or dataset record fetched: 7 (North1981, Hoffman1998, Walker1981, Myhre1998, Ferreira2018, Beck2018, Beck2023).
- search result snippet only (citing passages from the Semantic Scholar API, or Kopp's web page): 17 (Berger1978, Laskar2004, Archer2009, Rowley2002, Bird2003, Self2006, Gough1981, Held2006, Nye1952, Imbrie1980, Trenberth2001, Lindzen1987, Bretherton2004, Manabe1969, Klein1993, Serreze1992, Kopp2011).
- fetched but unreadable (page scan, no text layer): 1 (Berner1994).
- not fetched: 22.

Spot checks: 41 in Part 2 (some bullets cover two ledger rows; counted once, by the bullet's result).
- MATCH: 15 (CLM-007/008; CLM-020; climlab A, B, D; CLM-070 coefficients and 38.1 m2; CLM-069 [secondary]; CLM-063; CLM-075 [secondary]; CLM-057 [secondary]; CLM-053 [secondary]; CLM-077 [secondary]; CLM-108; CLM-092 [secondary]; CLM-113; Peel 2007; CLM-143 for 150 mm [secondary]).
- PARTIAL: 17.
- MISMATCH: 3.
- COULD NOT READ: 6 (CLM-005 D = 0.649; CLM-065 exponent 0.3; CLM-054; CLM-048; CLM-046; CLM-134).
  (15 + 17 + 3 + 6 = 41.)

MISMATCH and PARTIAL results, one line each.
- MISMATCH: the Byrne & Goldblatt fit is stated valid for 200-10,000 ppmv (calculated to 50,000 ppmv); the code applies it to 1e6 ppm, 100 times the fit range.
- MISMATCH: snowball escape at 5.5e5 ppm (0.55 bar) against Hoffman et al. 2017's 10^4-10^5 ppm (0.01-0.1 bar): 5.5 to 55 times higher; the paper's highest modelled CO2 is 100 mbar.
- MISMATCH: Earth's global mean 15.6 degC; Berkeley Earth gives 14.10 degC for 1951-1980 and 15.41 degC for 2024 (warmest year in the file); no annual value reaches 15.6. Citable figure: 14.1 degC (1951-1980, Rohde & Hausfather 2020).
- PARTIAL: A = 203.3 and B = 2.09 are called NCC81's by one citing paper and Budyko's (1969) by another; NCC81 itself unreadable (scan). D = 0.649 unconfirmed (only "about 0.6" found).
- PARTIAL: climlab's a0 = 0.33, a2 = 0.25 are the EBM_seasonal defaults, which have no ice albedo; a_i = 0.62 and T_f = -10 belong to the base EBM class with a0 = 0.3, a2 = 0.078. The code's combination is not a climlab default set.
- PARTIAL: S0 = 1365.2 is climlab's number, 4.4 m2 above Kopp & Lean's 1360.8.
- PARTIAL: runoff 0.038 is GEOCARB II's; Berner & Kothavala 2001 use 0.045 or 0.025. The 0.09 temperature factor matches both. The CO2 exponent 0.3 (Walker et al.) could not be read; Berner & Kothavala use 0.5 for pre-vascular weathering.
- PARTIAL: Hoffman et al. 2017's "1 km of sea level" is an assumption inside the paper, which also says geology needs more ice than its 100 mbar model had; the built snowball falls 1.48 km.
- PARTIAL: 2.47 m per 1e6 km3 equals Greenland's ratio (2.48) not Antarctica's (2.15); the formula gives 74 m for the 3.0e7 km3 whose sea-level equivalent the comment gives as 65 m (those two figures themselves match Bedmap2 + BedMachine v3).
- PARTIAL: Ohmura's T is free-atmosphere summer temperature at the ELA (quadratic fit, 70 glaciers); coefficients unread; the code feeds it a sea-level surface temperature and extrapolates to P = 0.
- PARTIAL: precession uses one 23-kyr period (the mean is about 21 kyr, with 19 and 23 kyr components); obliquity and eccentricity values agree with the secondary sources.
- PARTIAL: Hoffman 1998's 350 times confirmed in the abstract; "3-4 Myr" and 5.4e12 mol/yr not in it.
- PARTIAL: sea-water density 1028 against 1025 surface (ice 917 matches).
- PARTIAL: Imbrie & Imbrie 17 kyr supported by one citing passage, b = 0.6 unseen; Nye principle supported, the project's formula unseen.
- PARTIAL, the "from memory" references: all six exist and the journal, volume and first page are right. Bretherton (exponent 15.6 supported by a citing paper; the design doc's "15.6-fold per unit RH" wording is wrong, it is an exponent), Trenberth & Caron (citing paper gives 1.8 PW at about 20 N, design says about 2 PW at 15-20 N), Ferreira (its abstract credits salinity; basin width is cited as one control), Lindzen & Nigam (SST gradients over ocean, applied here to land), Klein & Hartmann (fine; nothing coded). Serreze, Kahl & Schnell exists but the "10-20 K" claim could not be checked, and Crossref lists the authors as Serreze, Schnell, Kahl. Manabe's 150 mm bucket matches (citing passage). Five of the six could be checked only through citing passages (Ferreira's abstract was read).

Paywalled or unreadable sources needed in stage 2 (to read the actual value):
- North, Cahalan & Coakley 1981 (A, B, D): needs OCR of the scan at https://eisenman.ucsd.edu/courses/2024-sioc224/files/north-1981.pdf or a library copy.
- Walker, Hays & Kasting 1981 (CO2 exponent 0.3, temperature law, "1 bar in 20 Myr").
- Berner 1994 GEOCARB II (open but a scan; OCR needed) for the 0.038 and 6.65e12 mol/yr.
- Berger 1978 and Laskar et al. 2004 (orbital periods and amplitudes; Laskar is open but blocked from automated fetch).
- Gough 1981 (the formula itself; four citing papers agree on it).
- Hoffman et al. 1998 (3-4 Myr, 5.4e12 mol/yr).
- Imbrie & Imbrie 1980 (b, tau_m).
- Myhre et al. 1998 (the 5.35 in the paper itself); Held & Soden 2006 (the 2 %/K statement).
- Rowley 2002, Bird 2003, Self et al. 2006 (1 Myr spread), Archer et al. 2009 (7-8 %): behind CLM-054, 058, 048.
- Ohmura et al. 1992 (open; OCR the regression equation).
- Bergman et al. 2004 tables (3.193e18 mol; open, tables are images).
- AMS papers (Bretherton 2004, Manabe 1969, Lindzen & Nigam 1987, Klein & Hartmann 1993, Serreze et al. 1992, Trenberth & Caron 2001): all blocked from automated download; need a manual fetch.
