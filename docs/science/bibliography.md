# Bibliography

Every scientific source the code, its comments, its data files and the build log rely on, with the
full reference as far as stage 1 (2026-10-02) could establish it, the ledger ids that rest on it,
and how it has been read.

The sources that only the research notes and design documents cite (tool surveys, mod
documentation, prior art) are listed separately in
[bibliography-research-notes.md](bibliography-research-notes.md), as those documents give them.

## How to read an entry

- **Reference**: authors, year, title, venue. Established by a stage 1 sub-agent from Crossref,
  OpenAlex, publisher pages and agency sites. A reference marked `UNIDENTIFIED` could not be pinned
  down from the citation in the code.
- **Cited as**: the string the code or documents use.
- **Ledger ids**: the rows of [claims.md](claims.md) that rely on it.
- **Access**: whether the source can be read freely. "Paywalled" sources are the ones stage 2
  needs a library, an author copy or a preprint for.
- **Read so far (by the project)**: what the code, the build log or the research notes say about how
  the agent that used the source read it. "Not stated" is the common case, and for this project it
  usually means the value was written from the agent's memory of the literature.
- **Read in this pass**: what the stage 1 bibliography agent itself fetched. "Full text fetched"
  means it had the text and quoted from it; "abstract fetched" means the abstract only; "snippet or
  secondary" means a search summary, a citing paper or an encyclopedia page.
- **Reference check**: whether the citation as written matches the real work (authors, year,
  venue, equation and table numbers), and whether the work appears to contain what is attributed to
  it.

The values themselves are checked in [spot-checks.md](spot-checks.md).

Limits of this pass. The web search budget ran out partway, so many sources were identified from
bibliographic databases without being opened. The fetch tool cannot read PDF files; agents that read
a PDF did so by extracting its text themselves, and scanned papers without a text layer could not be
read at all. Keys were chosen per subsystem, so one work can appear under two keys in two sections;
known duplicates are listed below. De-duplicating across sections is a stage 2 task.

## Counts

475 entries (a work cited in two subsystems is counted twice).

Thirteen more entries were added with the flexure feature on 2026-10-02, in their own section at
the end ("Lithospheric flexure"). The three tables below count stage 1's 475 only.

By access:

| | Sources |
|---|---|
| paywalled | 206 |
| open access | 108 |
| open copy | 70 |
| book (not online) | 40 |
| other or not checked | 29 |
| not found | 15 |
| unidentified | 7 |
| **all** | **475** |

By what the stage 1 bibliography pass itself read:

| | Sources |
|---|---|
| not fetched (metadata at most) | 228 |
| abstract fetched | 119 |
| full text fetched | 74 |
| snippet or secondary only | 29 |
| other | 24 |
| fetched but unreadable | 1 |
| **all** | **475** |

By what the project had recorded about its own reading before this pass:

| | Sources |
|---|---|
| not stated | 211 |
| other statement (see the entry) | 80 |
| abstract only | 56 |
| from memory, or marked unverified | 45 |
| read in full or in part (as the project states) | 26 |
| not cited by the project (added in stage 1) | 21 |
| not read | 20 |
| secondary source or search result | 16 |
| **all** | **475** |

## Unidentified

Citations that could not be resolved to a real reference:

- `Shepard` (ERO): cited as "Shepard's mean shelf break" (build log 2026-09-29 P1-17 partial).
- `Software` (DEP): cited as "MapMark4 (USGS R package, CC0, method reference)", "EMINERS", "EIS Toolkit".
- `NMET-Jharia` (ORE): cited as "NMET Jharia proposal".
- `SAIMM-coalfields` (ORE): cited as "SAIMM J. 105(2) p. 95", "Table I".
- `SilesiaMDPI` (ORE): cited as "the Silesia MDPI paper".
- `RuhrCounts` (ORE): cited as "the Ruhr counts".
- `Tetens` (CLM): cited as "Tetens"

## Keys that appear in more than one section

`Abramov2004` (IMP, ORE), `Ames1998` (IMP, ORE), `Barnes2014` (ERO, AMP), `Beaumont2001` (ERO, PLT), `Beck2018` (MOI, CLM), `Beck2023` (MOI, CLM), `Bekker2010` (DEP, ORE), `Berner1994` (MOI, CLM), `Bethke1990` (DEP, ORE), `Bird1991` (ERO, PLT), `Bird2003` (PLT, CLM), `Bohacs1997` (DEP, ORE), `Bradley2003` (DEP, ORE), `Butterworth2016` (DEP, ORE), `Cogley1984` (ERO, PLT), `Coltice2019` (SCL, PLT), `Cooke2005` (DEP, ORE), `Cortial2019` (ERO, PLT), `Cortial2019` (ERO, SCL), `Courtillot2003` (VOL, PLT), `Cuffey2010` (AMP, CLM), `Doust1990` (RCK, ORE), `Dubiel2003` (DEP, ORE), `Dyman2002` (DEP, ORE), `Ellis1999` (DEP, ORE), `Emsbo2016` (DEP, ORE), `Felzenszwalb2012` (MOI, CLM), `Fu1981` (HYD, MOI), `Garven1984` (DEP, ORE), `Garven1985` (DEP, ORE), `Gough1981` (SCL, CLM), `Grieve2005` (IMP, ORE), `Heckel1986` (DEP, ORE), `Heimsath1997` (IMP, ERO), `Heimsath1997` (IMP, RCK), `Higley2005` (DEP, ORE), `Hjulstrom1935` (ERO, RCK), `Hoggard2020` (DEP, ORE), `Hudec2007` (RCK, ORE), `John2010` (DEP, ORE), `Kesler2006` (DEP, ORE), `Kesler2008` (DEP, ORE), `Kesler2008` (DEP, SCL), `Leach2001` (DEP, ORE), `Leach2005` (DEP, ORE), `Leach2010a` (DEP, ORE), `Leach2010b` (DEP, ORE), `Lightfoot2016` (IMP, ORE), `Mallard2016` (SCL, PLT), `Masaitis1998` (IMP, ORE), `MiddletonThomas1997` (HYD, MOI), `Moore2003` (DEP, ORE), `Naldrett2004` (IMP, ORE), `Nelsen2016` (DEP, ORE), `Newell1997` (DEP, ORE), `Nuccio1994` (DEP, ORE), `Nuccio2003` (DEP, ORE), `Ohmura1992` (HYD, AMP), `Ohmura1992` (HYD, CLM), `Ohmura1992` (HYD, MOI), `Osinski2013` (IMP, ORE), `Peel2007` (ERO, CLM), `Peel2007` (ERO, HYD), `Peel2007` (ERO, MOI), `Peel2007` (ERO, SCL), `Porter1989` (HYD, AMP), `Porter2001` (HYD, AMP), `Rasbury1998` (DEP, ORE), `Rey2001` (ERO, PLT), `Richards2003` (DEP, ORE), `Robert2005` (DEP, ORE), `Roering1999` (IMP, ERO), `Roering1999` (IMP, RCK), `Royden1997` (ERO, PLT), `Schon2015` (ERO, RCK), `Schott2024` (ERO, AMP), `Schott2024` (ERO, HYD), `Sillitoe2005` (DEP, ORE), `Singer2005` (DEP, ORE), `Singer2009` (DEP, ORE), `Sklar2001` (IMP, ERO), `Sklar2001` (IMP, RCK), `Spencer1988` (DEP, ORE), `Stock1999` (IMP, ERO), `Stock1999` (IMP, RCK), `Telford1990` (ERO, RCK), `Tozer2019` (ERO, CLM), `Tozer2019` (ERO, MOI), `Tozer2019` (ERO, PLT), `Tuttle2000` (DEP, ORE), `Tzathas2024` (ERO, AMP), `USGS2000` (DEP, ORE), `Waples1980` (DEP, ORE), `Wood1983` (DEP, ORE)

---

## Meteor impacts (IMP)

### Collins2005
- Reference: Collins, G. S., Melosh, H. J., and Marcus, R. A. (2005). Earth Impact Effects Program: A Web-based computer program for calculating the regional environmental consequences of a meteoroid impact on Earth. Meteoritics & Planetary Science 40(6):817-840.
- DOI or URL: 10.1111/j.1945-5100.2005.tb00157.x. Open copy (Imperial College Spiral, hdl 10044/1/11554, file CollinsEtAl2005.pdf, 1.6 MB): https://spiral.imperial.ac.uk/bitstreams/8065368c-096c-4787-aab7-cbaf1a7d7277/download . The program's own page links the paper as `static/effects.pdf` on https://impact.ese.ic.ac.uk/ImpactEarth/ImpactEffects/ (the older path /ImpactEffects/effects.pdf returned 404; Wiley returned 403).
- Cited as: "Collins, Melosh & Marcus 2005", "Collins et al. 2005, eq. N" (eqs. 12, 21, 22, 24, 25, 26, 27, 28, 31, 32 in code comments).
- Ledger ids: IMP-020, 023, 029, 030, 031, 032, 037, 038, 039, 040, 041, 042, 054, 062, 063
- Access: open copy at the Spiral URL above.
- Read so far (by the project): from memory ("the paper could not be fetched").
- Read in this pass: full text fetched (all 24 pages; read in detail: Introduction, "Crater dimensions and melt production" pp. 822-825, "Ejecta deposit" pp. 828-829, the worked example and Table 6, the reference list).
- Reference check: authors, year, venue, volume(issue) and pages are right. Equation numbering in the paper marks the equations the program implements with an asterisk. Of the numbers the code cites: eqs. 21, 22, 24, 26, 27, 28, 31 are right (but see the value MISMATCH for eq. 28); the melt law is eq. 30 (code says 12; paper eq. 12 is in the atmospheric-entry section); the rim height is eq. 48 (code says 25; paper eq. 25 is transient depth d_tc = D_tc/(2 sqrt 2)); the ejecta thickness is eq. 47 (code says 32; paper eq. 32 is the fireball radius); V_br = 0.032 D^3 is eq. 23 (code groups it under "eqs. 24 and 26"). The complex-crater depth law in eq. 28 is 0.4 D^0.3, not 0.294 D^0.301 (Part 2, IMP-042). The paper credits its eq. 21 to Holsapple & Schmidt 1982, Schmidt & Housen 1987 and Gault 1974, not to Gault & Wedekind 1978 or Chapman & McKinnon 1986, and it has no default impact speed (Part 2, IMP-020).

### GrieveShoemaker1994
- Reference: Grieve, R. A. F. and Shoemaker, E. M. (1994). The record of past impacts on Earth. In: Gehrels, T. (ed.), Hazards Due to Comets and Asteroids, University of Arizona Press, Tucson, pp. 417-462.
- DOI or URL: no DOI found. Internet Archive lending copy of the book (not text-searchable by me): https://archive.org/details/hazardsduetocome0000unse (year 1994 in its metadata).
- Cited as: "Grieve & Shoemaker 1994".
- Ledger ids: IMP-001, 002, 003, 008, 019
- Access: book (not online); lending copy at archive.org was not readable (403 on the search-inside service, djvu text withheld).
- Read so far (by the project): secondary ("checked against the literature by a search"); the scaling.rs header says "as remembered".
- Read in this pass: not fetched (tried archive.org; no text obtained).
- Reference check: pages 417-462 and title match the reference list of Davison & Collins 2007, which dates the chapter 1995; Collins et al. 2005 date the same volume 1994 (Bottke et al. 1994 in it). The book is 1994 (archive.org). The value 5.6 +/- 2.8 was NOT found in any text I could read; the older figure 5.4 +/- 2.7 x 10^-15 km^-2 yr^-1 (D >= 20 km, N = 7, Grieve 1984) is quoted in Grieve 1987 (see Grieve1987).

### Neukum2001
- Reference: Neukum, G., Ivanov, B. A., and Hartmann, W. K. (2001). Cratering Records in the Inner Solar System in Relation to the Lunar Reference System. Space Science Reviews 96(1-4):55-86.
- DOI or URL: 10.1023/A:1011989004263 (Crossref-verified)
- Cited as: "Neukum, Ivanov & Hartmann 2001"
- Ledger ids: IMP-005
- Access: paywalled (Springer; only Crossref metadata seen)
- Read so far (by the project): from memory
- Read in this pass: not fetched (metadata only)
- Reference check: authors, year, venue, volume and pages match the real paper. Whether it says the rate was "constant within a factor of two for the last ~3 Gyr" is not checked.

### Bland2006
- Reference: Bland, P. A. and Artemieva, N. A. (2006). The rate of small impacts on Earth. Meteoritics & Planetary Science 41(4):607-631.
- DOI or URL: 10.1111/j.1945-5100.2006.tb00485.x (Crossref-verified)
- Cited as: "Bland & Artemieva 2006"
- Ledger ids: IMP-014
- Access: paywalled (Wiley; metadata only)
- Read so far (by the project): from memory
- Read in this pass: not fetched (metadata only)
- Reference check: authors, year, venue, volume and pages match. The topic (small impactors broken up by the atmosphere) fits the claim; Davison & Collins 2007 cite it and say impactors "much smaller than 100 m ... are significantly affected by atmospheric entry" (full text read). The actual statement about stony bodies is not checked.

### LeFeuvre2011
- Reference: Le Feuvre, M. and Wieczorek, M. A. (2011). Nonuniform cratering of the Moon and a revised crater chronology of the inner Solar System. Icarus 214(1):1-20.
- DOI or URL: 10.1016/j.icarus.2011.03.010 (Crossref-verified). Open copies listed by OpenAlex: https://hal.science/hal-00768797 (green OA; the document URL returned "Access Denied" to me), http://hdl.handle.net/2262/65826 (Trinity College Dublin TARA; not tried).
- Cited as: "Le Feuvre & Wieczorek 2011"
- Ledger ids: IMP-020
- Access: open copy (HAL, green) at https://hal.science/hal-00768797
- Read so far (by the project): from memory
- Read in this pass: not fetched (HAL access denied)
- Reference check: authors, year, venue, volume and pages match. Whether it gives 20 km/s as the Earth mean for asteroids is not checked.

### Gilbert1893
- Reference: Gilbert, G. K. (1893). The Moon's face: a study of the origin of its features. Bulletin of the Philosophical Society of Washington 12:241-292. [Details from my recollection, not verified in this pass; the sources file gives only author and year.]
- DOI or URL: none found
- Cited as: "Gilbert 1893" (with Shoemaker 1962, for the sin 2 theta angle distribution)
- Ledger ids: IMP-024
- Access: not found (not searched)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: Collins et al. 2005 cite only Shoemaker 1962 for the 45-degree most likely angle; they do not cite Gilbert.

### Shoemaker1962
- Reference: Shoemaker, E. M. (1962). Interpretation of lunar craters. In: Kopal, Z. (ed.), Physics and Astronomy of the Moon, Academic Press, New York, pp. 283-359. (As in the reference list of Collins et al. 2005.)
- DOI or URL: none
- Cited as: "Shoemaker 1962"
- Ledger ids: IMP-024
- Access: book (not online)
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. 2005 (full text) say "The most likely impact angle is 45 degrees (Shoemaker 1962)".
- Reference check: consistent with Collins et al.'s citation. The dP = sin 2 theta d theta form is not stated in Collins et al.

### SchmidtHousen1987
- Reference: Schmidt, R. M. and Housen, K. R. (1987). Some recent advances in the scaling of impact and explosion cratering. International Journal of Impact Engineering 5(1-4):543-560.
- DOI or URL: 10.1016/0734-743X(87)90069-8 (Crossref-verified)
- Cited as: "Schmidt & Housen 1987"
- Ledger ids: IMP-030
- Access: paywalled (metadata only)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: matches. Collins et al. 2005 name it (with Holsapple & Schmidt 1982, JGR 87:1849-1870, and Gault 1974) as the basis of eq. 21 and as the source of the water constant 1.365.

### GaultWedekind1978
- Reference: Gault, D. E. and Wedekind, J. A. (1978). Experimental studies of oblique impact. Proceedings of the 9th Lunar and Planetary Science Conference, pp. 3843-3875. [Recollection, not verified.]
- DOI or URL: none found
- Cited as: "Gault & Wedekind 1978" (for the sin^(1/3) theta factor)
- Ledger ids: IMP-030
- Access: not found (not searched)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: Collins et al. 2005 do not cite this paper; they cite Gault 1974 (Impact cratering, in A Primer in Lunar Geology, NASA Ames, pp. 137-175). They do not attribute the sin^(1/3) theta factor to anyone.

### ChapmanMcKinnon1986
- Reference: Chapman, C. R. and McKinnon, W. B. (1986). Cratering of planetary satellites. In: Burns, J. A. and Matthews, M. S. (eds), Satellites, University of Arizona Press, pp. 492-580. [Recollection, not verified.]
- DOI or URL: none found
- Cited as: "Chapman & McKinnon 1986"
- Ledger ids: IMP-030
- Access: book (not online)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: not cited by Collins et al. 2005 (see GaultWedekind1978).

### McKinnonSchenk1985
- Reference: McKinnon, W. B. and Schenk, P. M. (1985). Ejecta blanket scaling on the Moon and Mercury: inferences for projectile populations (abstract). Proceedings of the 16th Lunar and Planetary Science Conference, pp. 544-545. (As in the reference list of Collins et al. 2005.)
- DOI or URL: none
- Cited as: "McKinnon & Schenk 1985 (as in Holsapple 1993)"
- Ledger ids: IMP-032
- Access: not found (conference abstract; not looked up)
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. 2005 (full text) say eq. 27 is the "functional form established by McKinnon and Schenk (1985), which lies intermediate between the estimates of Croft (1985) and Holsapple (1993)".
- Reference check: Collins et al. cite it directly, so "as in Holsapple 1993" is the code's own gloss. Holsapple 1993 is Annu. Rev. Earth Planet. Sci. 21:333-373, DOI 10.1146/annurev.ea.21.050193.002001 (Crossref-verified).

### GrieveDenceRobertson1977
- Reference: Grieve, R. A. F., Dence, M. R., and Robertson, P. B. (1977). Cratering processes: as interpreted from the occurrence of impact melts. In: Roddy, D. J., Pepin, R. O., and Merrill, R. B. (eds), Impact and Explosion Cratering, Pergamon Press, New York, pp. 791-814. (Reference list of Collins et al. 2005.) A companion: Dence, M. R., Grieve, R. A. F., and Robertson, P. B. (1977), Terrestrial impact structures: principal characteristics and energy considerations, same volume, pp. 247-275.
- DOI or URL: none
- Cited as: "Grieve, Dence & Robertson 1977"
- Ledger ids: IMP-039, 052
- Access: book (not online)
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. use Grieve et al. 1977 for the melt-and-breccia lens (p. 823) and the uniform melt sheet (p. 825), and Dence et al. 1977 for d_tc = D_tc/(2 sqrt 2).
- Reference check: Fine for the lens; the claim "complex-crater fill thickness equals the crater's depth" (IMP-052) is not in Collins et al. and is not checked.

### Melosh1989
- Reference: Melosh, H. J. (1989). Impact Cratering: A Geologic Process. Oxford University Press, New York, 245 pp. (Reference list of Collins et al. 2005.)
- DOI or URL: none
- Cited as: "Melosh 1989" (collapse and lens, transition, excavation depth, section 6.3, fig. 8.1)
- Ledger ids: IMP-036, 039, 053, 058, 128
- Access: book (not online)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: real book. Section and figure numbers not checked.

### Grieve1987
- Reference: Grieve, R. A. F. (1987). Terrestrial impact structures. Annual Review of Earth and Planetary Sciences 15:245-270.
- DOI or URL: 10.1146/annurev.ea.15.050187.001333 (Crossref-verified). Open scan with OCR text: https://www.wipp.energy.gov/library/cra/2009_cra/references/Others/Grieve_1987_Terrestrial_Impact_Structures_ERMS241228.pdf
- Cited as: "Grieve 1987"
- Ledger ids: IMP-034, 035
- Access: open copy (scan) at the WIPP URL above
- Read so far (by the project): from memory
- Read in this pass: full text fetched (OCR layer; read the morphology section, the cratering-rate section and the size-frequency section)
- Reference check: venue, volume and pages match. It supports the 2 km / 4 km split (IMP-035) but I found no 3.2 km in its OCR text, so it is not the source of the 3.2 km figure (IMP-034); Collins et al. give 3.2 km, classified by Dence 1965. It also contains the older rate figure used as a secondary check on IMP-001.

### Pike1980
- Reference: Pike, R. J. (1980). Control of crater morphology by gravity and target type: Mars, Earth, Moon. Proceedings of the 11th Lunar and Planetary Science Conference (Geochimica et Cosmochimica Acta Supplement 14), pp. 2159-2189. (Collins et al. print the pages as 2159-2190 and the series as "Geochimica et Cosmochimica Acta 3".)
- DOI or URL: none found
- Cited as: "Pike 1980"
- Ledger ids: IMP-036
- Access: not found (an ADS scan probably exists; not fetched. The ADS scan of Pike 1985 had no text layer, so this one likely has none either)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: title and series fit the claim (gravity control of the transition). Collins et al. cite it only for terrestrial complex-crater depth data. Page end differs between the code's source list (2189) and Collins et al. (2190).

### AlexopoulosMcKinnon1994
- Reference: Alexopoulos, J. S. and McKinnon, W. B. (1994). Large impact craters and basins on Venus, with implications for ring mechanics on the terrestrial planets. In: Dressler, B. O., Grieve, R. A. F., and Sharpton, V. L. (eds), Large Meteorite Impacts and Planetary Evolution, Geological Society of America Special Paper 293, pp. 29-50.
- DOI or URL: 10.1130/SPE293-p29 (Crossref-verified; Crossref dates the chapter 1992, the volume is 1994). GeoScienceWorld page returned 403: https://pubs.geoscienceworld.org/books/book/440/chapter/3798885/
- Cited as: "Alexopoulos & McKinnon 1994"
- Ledger ids: IMP-045
- Access: paywalled (not even an abstract seen)
- Read so far (by the project): from memory
- Read in this pass: not fetched (403)
- Reference check: title, authors, volume and pages match. Whether it gives about 40 km for the onset of peak rings on Venus is unchecked. The editors and the year 1994 for SP 293 are from my recollection plus the code's own year; Crossref's 1992 is a discrepancy to settle.

### WoodHead1976
- Reference: Wood, C. A. and Head, J. W. (1976). Comparison of impact basins on Mercury, Mars and the Moon. Proceedings of the 7th Lunar Science Conference, pp. 3629-3651. [Recollection, not verified.]
- DOI or URL: none found
- Cited as: "Wood & Head 1976"
- Ledger ids: IMP-046
- Access: not found (not searched)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: unverified.

### PikeSpudis1987
- Reference: Pike, R. J. and Spudis, P. D. (1987). Basin-ring spacing on the Moon, Mercury, and Mars. Earth, Moon, and Planets 39(2):129-194.
- DOI or URL: 10.1007/BF00054060 (Crossref-verified)
- Cited as: "Pike & Spudis 1987"
- Ledger ids: IMP-046
- Access: paywalled (metadata only)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: real paper. It concerns multi-ring basins; the claim "ring diameter about half the rim's" for peak-ring craters is unchecked.

### Pike1985
- Reference: Pike, R. J. (1985). Some morphologic systematics of complex impact structures. Meteoritics 20(1):49-68.
- DOI or URL: 10.1111/j.1945-5100.1985.tb00846.x (Crossref-verified). ADS scan (JBIG2 images, 20 pages, no text layer; unreadable by my tools): https://articles.adsabs.harvard.edu/pdf/1985Metic..20...49P
- Cited as: "Pike 1985, Meteoritics 20:49-68"
- Ledger ids: IMP-048
- Access: open copy (scan only) at the ADS URL
- Read so far (by the project): from memory
- Read in this pass: fetched but unreadable (image-only scan)
- Reference check: venue, volume and pages match; title found by Crossref.

### Pike1977
- Reference: Pike, R. J. (1977). Size-dependence in the shape of fresh impact craters on the Moon. In: Roddy, D. J., Pepin, R. O., and Merrill, R. B. (eds), Impact and Explosion Cratering, Pergamon Press, New York, pp. 489-509. [Recollection, not verified.]
- DOI or URL: none
- Cited as: "Pike 1977" (comment only)
- Ledger ids: IMP-050
- Access: book (not online)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: unverified; the code uses it only in comments.

### McGetchin1973
- Reference: McGetchin, T. R., Settle, M., and Head, J. W. (1973). Radial thickness variation in impact crater ejecta: implications for lunar basin deposits. Earth and Planetary Science Letters 20(2):226-236.
- DOI or URL: 10.1016/0012-821X(73)90162-3 (Crossref-verified)
- Cited as: "McGetchin, Settle & Head 1973"
- Ledger ids: IMP-054, 055
- Access: paywalled (metadata only)
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. (full text) cite it: "The power of 3 is a good approximation of data from explosion experiments (McGetchin et al. 1973)... the power can vary between 2.5 and 3.5."
- Reference check: matches. Collins et al. eq. 47 is not "McGetchin" itself but their own volume-conservation model using the r^-3 falloff.

### Moore1974
- Reference: Moore, H. J., Hodges, C. A., and Scott, D. H. (1974). Multiringed basins: illustrated by Orientale and associated features. Proceedings of the 5th Lunar Science Conference, pp. 71-100. [Recollection, not verified; the title suggests the paper is about large basins, which may not be the source of a 2.3-radii blanket limit for ordinary craters.]
- DOI or URL: none found
- Cited as: "Moore, Hodges & Scott 1974"
- Ledger ids: IMP-057
- Access: not found (not searched)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: unverified; flag for stage 2.

### GrieveCintala1992
- Reference: Grieve, R. A. F. and Cintala, M. J. (1992). An analysis of differential impact melt-crater scaling and implications for the terrestrial impact record. Meteoritics 27:526-538. (Collins et al. print the title as "An analysis of differential melt-crater scaling and implications for the terrestrial impact record".)
- DOI or URL: none looked up
- Cited as: "Grieve & Cintala 1992"
- Ledger ids: IMP-062
- Access: not found (not looked up)
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. cite it (with Pierazzo et al. 1997 and Pierazzo & Melosh 2000) as support for the melt volume.
- Reference check: venue, volume and pages as in Collins et al.'s list.

### PierazzoVickery1997
- Reference: Pierazzo, E., Vickery, A. M., and Melosh, H. J. (1997). A reevaluation of impact melt production. Icarus 127(2):408-423.
- DOI or URL: 10.1006/icar.1997.5713 (Crossref-verified)
- Cited as: "Pierazzo, Vickery & Melosh 1997"
- Ledger ids: IMP-062
- Access: paywalled (metadata only)
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. take epsilon_m = 5.2 MJ/kg for granite "(see Pierazzo et al. 1997)".
- Reference check: matches. Collins et al. also cite Pierazzo & Melosh 2000 (Icarus 145:252-261) for the angle dependence.

### DavisonCollins2007
- Reference: Davison, T. and Collins, G. S. (2007). The effect of the oceans on the terrestrial crater size-frequency distribution: Insight from numerical modeling. Meteoritics & Planetary Science 42(11):1915-1927.
- DOI or URL: 10.1111/j.1945-5100.2007.tb00550.x. Open copy (Imperial Spiral, hdl 10044/1/11559, DavisonCollins2007.pdf): https://spiral.imperial.ac.uk/server/api/core/bitstreams/ba3d1b52-99e7-4439-926d-7a38cdfb3e7c/content . Another repository copy: University of Arizona, http://hdl.handle.net/10150/656356 .
- Cited as: "Davison & Collins 2007"
- Ledger ids: IMP-065, 066, 067
- Access: open copy at the Spiral URL
- Read so far (by the project): abstract only
- Read in this pass: full text fetched (abstract, introduction, method, results, Table 2, discussion)
- Reference check: the citation is right. The code's note that it fits "three numbers" is accurate for the abstract. The paper's own cutoff is not 6.3 (Part 2).

### Culling1960
- Reference: Culling, W. E. H. (1960). Analytical theory of erosion. The Journal of Geology 68(3):336-344.
- DOI or URL: 10.1086/626663 (Crossref-verified)
- Cited as: "Culling 1960"
- Ledger ids: IMP-072, 083
- Access: paywalled (metadata only)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: matches (title found by Crossref). The paper is general erosion theory; it gives no crater-specific diffusivity.

### Soderblom1970
- Reference: Soderblom, L. A. (1970). A model for small-impact erosion applied to the lunar surface. Journal of Geophysical Research 75(14):2655-2661.
- DOI or URL: 10.1029/JB075i014p02655 (Crossref-verified)
- Cited as: "Soderblom 1970"
- Ledger ids: IMP-072
- Access: paywalled (metadata only)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: matches. Its title says the diffusion comes from small impacts on the Moon, not soil creep.

### FassettThomson2014
- Reference: Fassett, C. I. and Thomson, B. J. (2014). Crater degradation on the lunar maria: Topographic diffusion and the rate of erosion on the Moon. Journal of Geophysical Research: Planets 119(10):2255-2271.
- DOI or URL: 10.1002/2014JE004698 (Crossref-verified). OpenAlex lists no open copy.
- Cited as: "Fassett & Thomson 2014"
- Ledger ids: IMP-072, 083
- Access: paywalled (abstract only, via OpenAlex)
- Read so far (by the project): from memory
- Read in this pass: abstract fetched (OpenAlex reconstruction of the abstract; wording has gaps)
- Reference check: matches. It is a LUNAR study (diffusivity about 5.5 m^2/Myr), not an Earth hillslope value (Part 2, IMP-083).

### Fernandes1997
- Reference: Fernandes, N. F. and Dietrich, W. E. (1997). Hillslope evolution by diffusive processes: The timescale for equilibrium adjustments. Water Resources Research 33(6):1307-1318.
- DOI or URL: 10.1029/97WR00534 (Crossref-verified)
- Cited as: "Fernandes & Dietrich 1997"
- Ledger ids: IMP-074
- Access: paywalled (metadata only)
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: matches. Whether it gives the 0.003-0.05 m^2/yr range is unchecked.

### Roering1999
- Reference: Roering, J. J., Kirchner, J. W., and Dietrich, W. E. (1999). Evidence for nonlinear, diffusive sediment transport on hillslopes and implications for landscape morphology. Water Resources Research 35(3):853-870.
- DOI or URL: 10.1029/1998WR900090 (Crossref-verified)
- Cited as: "Roering, Kirchner & Dietrich 1999"
- Ledger ids: IMP-074, 086, 090
- Access: paywalled (metadata only)
- Read so far (by the project): from memory (params.rs); judgment framework (rock-class JSON)
- Read in this pass: not fetched
- Reference check: matches the rock-class JSON (WRR 35(3):853-870). Note this paper's point is NONLINEAR diffusion, while the code uses linear diffusion.

### Sklar2001
- Reference: Sklar, L. S. and Dietrich, W. E. (2001). Sediment and rock strength controls on river incision into bedrock. Geology 29(12):1087-1090.
- DOI or URL: 10.1130/0091-7613(2001)029<1087:SARSCO>2.0.CO;2 (Crossref-verified)
- Cited as: "Sklar & Dietrich 2001"
- Ledger ids: IMP-085, 089
- Access: paywalled (metadata only)
- Read so far (by the project): no statement; numbers labelled "judgment"
- Read in this pass: not fetched
- Reference check: matches.

### Stock1999
- Reference: Stock, J. D. and Montgomery, D. R. (1999). Geologic constraints on bedrock river incision using the stream power law. Journal of Geophysical Research: Solid Earth 104(B3):4983-4993.
- DOI or URL: 10.1029/98JB02139 (Crossref-verified)
- Cited as: "Stock & Montgomery 1999"
- Ledger ids: IMP-085, 089
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches.

### Heimsath1997
- Reference: Heimsath, A. M., Dietrich, W. E., Nishiizumi, K., and Finkel, R. C. (1997). The soil production function and landscape equilibrium. Nature 388(6640):358-361.
- DOI or URL: 10.1038/41056 (Crossref-verified)
- Cited as: "Heimsath et al. 1997"
- Ledger ids: IMP-086, 090
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches.

### Naldrett2004
- Reference: Naldrett, A. J. (2004). Magmatic Sulfide Deposits: Geology, Geochemistry and Exploration. Springer, Berlin. [Subtitle and publisher from my recollection, not verified.]
- DOI or URL: none looked up
- Cited as: "Naldrett 2004"
- Ledger ids: IMP-095, 097, 105, 106, 111
- Access: book (not online)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: unverified.

### GrieveStoffler1991
- Reference: Grieve, R. A. F., Stoffler, D., and Deutsch, A. (1991). The Sudbury structure: Controversial or misunderstood? Journal of Geophysical Research: Planets 96(E5):22753-22764.
- DOI or URL: 10.1029/91JE02513 (Crossref-verified)
- Cited as: "Grieve, Stoffler & Deutsch 1991"
- Ledger ids: IMP-095, 097
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches (JGR 96, pages 22753-22764). Whether it gives the 2.5-3 km melt-sheet thickness is unchecked.

### Lightfoot2016
- Reference: Lightfoot, P. C. (2016). Nickel Sulfide Ores and Impact Melts: Origin of the Sudbury Igneous Complex. Elsevier, Amsterdam. [Publisher from my recollection, not verified.]
- DOI or URL: none looked up
- Cited as: "Lightfoot 2016"
- Ledger ids: IMP-107, 111
- Access: book (not online)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: unverified.

### Abramov2004
- Reference: Abramov, O. and Kring, D. A. (2004). Numerical modeling of an impact-induced hydrothermal system at the Sudbury crater. Journal of Geophysical Research: Planets 109(E10), article E10007.
- DOI or URL: 10.1029/2003JE002213 (Crossref-verified)
- Cited as: "Abramov & Kring 2004"
- Ledger ids: IMP-098, 099, 113
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches. Its subject is Sudbury specifically; the generalisation to "10 thousand to 1 million years" (IMP-099) is unchecked.

### Osinski2013
- Reference: Osinski, G. R., Tornabene, L. L., Banerjee, N. R., Cockell, C. S., Flemming, R., Izawa, M. R. M., McCutcheon, J., Parnell, J., Preston, L. J., Pickersgill, A. E., Pontefract, A., Sapers, H. M., and Southam, G. (2013). Impact-generated hydrothermal systems on Earth and Mars. Icarus 224(2):347-363.
- DOI or URL: 10.1016/j.icarus.2012.08.030 (Crossref-verified)
- Cited as: "Osinski et al. 2013"
- Ledger ids: IMP-098, 099, 113, 116
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches.

### Masaitis1998
- Reference: Masaitis, V. L. (1998). Popigai crater: Origin and distribution of diamond-bearing impactites. Meteoritics & Planetary Science 33(2):349-359.
- DOI or URL: 10.1111/j.1945-5100.1998.tb01639.x (Crossref-verified)
- Cited as: "Masaitis 1998"
- Ledger ids: IMP-100, 101, 117, 120
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches (MAPS 33, pages 349-359; first-name initial V. L. from my recollection).

### Donofrio1998
- Reference: Donofrio, R. R. (1998). North American impact structures hold giant field potential. Oil & Gas Journal 96(19):69-83. [Author initials and title from my recollection; the sources file gives only author, journal and pages. Not verified.]
- DOI or URL: none found
- Cited as: "Donofrio 1998"
- Ledger ids: IMP-103, 104, 124, 127
- Access: not found
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: unverified.

### Grieve2005
- Reference: Grieve, R. A. F. (2005). Economic natural resource deposits at terrestrial impact structures. In: McDonald, I., Boyce, A. J., Butler, I. B., Herrington, R. J., and Polya, D. A. (eds), Mineral Deposits and Earth Evolution, Geological Society, London, Special Publications 248:1-29. [Editors from my recollection; Crossref confirms title, volume, pages, year.]
- DOI or URL: 10.1144/GSL.SP.2005.248.01.01 (Crossref-verified)
- Cited as: "Grieve 2005"
- Ledger ids: IMP-103, 104, 124, 127
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches.

### GrieveMasaitis1994
- Reference: Grieve, R. A. F. and Masaitis, V. L. (1994). The economic potential of terrestrial impact craters. International Geology Review 36(2):105-151.
- DOI or URL: 10.1080/00206819409465452 (Crossref-verified)
- Cited as: "Grieve & Masaitis 1994"
- Ledger ids: IMP-105, 106, 111, 116, 120, 124, 125
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches.

### Ames1998
- Reference: Ames, D. E., Watkinson, D. H., and Parrish, R. R. (1998). Dating of a regional hydrothermal system induced by the 1850 Ma Sudbury impact event. Geology 26(5):447-450.
- DOI or URL: 10.1130/0091-7613(1998)026<0447:DOARHS>2.3.CO;2 (Crossref-verified)
- Cited as: "Ames, Watkinson & Parrish 1998"
- Ledger ids: IMP-116
- Access: paywalled (metadata only)
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches.

### Herrick1997 (not in the sources file; it is the real source of Collins et al.'s eq. 28)
- Reference: Herrick, R. R., Sharpton, V. L., Malin, M. C., Lyons, S. N., and Feely, K. (1997). Morphology and morphometry of impact craters. In: Bougher, S. W., Hunten, D. M., and Phillips, R. J. (eds), Venus II, University of Arizona Press, Tucson, pp. 1015-1046. (Collins et al. print the last author as "Freely".)
- DOI or URL: none
- Cited as: not cited in the code; named by Collins et al. 2005 for d_fr = 0.4 D_fr^0.3
- Ledger ids: IMP-042 (indirect)
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: see IMP-042.

### Not bibliographic
Ben's decisions log entries (2026-09-29, 09-30, 10-01), docs/deeptime-design-decisions.md scale table, MIC rock-tag README, impact_structures.json ("not a USGS source"), the deposit-type and rock-class JSON notes, docs/research/notes/geologist_youtuber_feedback.md, STATUS.md build log, tools/review/p2-08-craters.py and p2-08-impacts.sh.

## Volcanoes and magma (VOL, MAG)

### Bremond1995
- Reference: de Bremond d'Ars, J., Jaupart, C. and Sparks, R. S. J. (1995). Distribution of volcanoes in active margins. Journal of Geophysical Research 100(B10): 20421-20432.
- DOI or URL: 10.1029/95JB02153 (https://agupubs.onlinelibrary.wiley.com/doi/10.1029/95JB02153). No open copy found.
- Cited as: "de Bremond d'Ars, Jaupart & Sparks 1995, JGR 100(B10):20421"
- Ledger ids: VOL-039 (and indirectly VOL-009, VOL-040)
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (Crossref and OpenAlex)
- Reference check: authors, year, journal, volume, issue and first page are right (the pages are 20421-20432). The content does NOT obviously match the attribution. The abstract says the study covers 16 active margins and 479 volcanic systems, arcs are ribbons about 10 times longer than wide, and "Volcano spacing follows a random distribution" (Gamma distribution, "no characteristic spacing" in the search summary), explained by gravitational instability in layered fluids. The code comment attributes to it "Real volcanic fronts space their centres 50-70 km apart (the Andes, Cascades, Aleutians, Japan)". The abstract gives no mean spacing and names none of those four arcs. The full text (which may tabulate mean spacings per margin) could not be read.

#### Part 1b: sources found for the uncited Earth figures (not cited by the code; added by this pass)

### Szabo1986
- Reference: Szabo, B. J. and Moore, J. G. (1986). Age of the -360-m reef terrace, Hawaii, and the rate of late Pleistocene subsidence of the island. Geology 14(11): 967-968 (volume and pages from the DOI string 10.1130/0091-7613(1986)14<967:AOMRTH>2.0.CO;2; issue not checked).
- DOI or URL: 10.1130/0091-7613(1986)14<967:AOMRTH>2.0.CO;2
- Access: paywalled (abstract only, via OpenAlex)
- Read in this pass: abstract fetched
- Used for: island subsidence rate (VOL-048).

### Poland2014
- Reference: Poland, M. P., Miklius, A. and Montgomery-Brown, E. K. (2014). Magma supply, storage, and transport at shield-stage Hawaiian volcanoes. Chapter 5 in Characteristics of Hawaiian Volcanoes, U.S. Geological Survey Professional Paper 1801.
- DOI or URL: 10.3133/pp18015; PDF https://pubs.usgs.gov/pp/1801/downloads/pp1801_Chap5_Poland.pdf (open, but came back as unreadable binary).
- Access: open access (agency report)
- Read in this pass: abstract fetched (USGS publication page). The chapter text with reservoir depths was not readable.
- Used for: Kilauea reservoir depths (MAG-002). The abstract gives the deep rift zone at about 3 km to 9 km depth and supply of about 0.1 km3/yr, not the summit reservoir depth.

### Przeor2022
- Reference: Przeor, M., D'Auria, L., Pepe, S. et al. (2022). Elastic interaction between Mauna Loa and Kilauea evidenced by independent component analysis. Scientific Reports.
- DOI or URL: not recorded (found by Europe PMC search).
- Access: open access
- Read in this pass: search snippet only (abstract as relayed: Kilauea deformation from "a volumetric source at 1.2 km depth" and "a sill-shaped source located at a depth of 3.5 km")
- Used for: Kilauea shallow reservoir depth (MAG-002).

### Scaillet2008
- Reference: Scaillet, B., Pichavant, M. and Cioni, R. (2008). Upward migration of Vesuvius magma chamber over the past 20,000 years. Nature 455(7210): 216-219.
- DOI or URL: 10.1038/nature07232
- Access: paywalled (abstract only, via Europe PMC)
- Read in this pass: abstract fetched
- Used for: Vesuvius chamber depth (MAG-002).

### Courtillot2003
- Reference: Courtillot, V. E. and Renne, P. R. (2003). On the ages of flood basalt events. Comptes Rendus Geoscience 335(1): 113-140.
- DOI or URL: 10.1016/S1631-0713(03)00006-3
- Access: paywalled (abstract only via OpenAlex; the journal may be open, not tried)
- Read in this pass: abstract fetched
- Used for: flood-basalt frequency (VOL-052 and run-review "one per 20-30 Myr").

### Bryan2008
- Reference: Bryan, S. E. and Ernst, R. E. (2008). Revised definition of Large Igneous Provinces (LIPs). Earth-Science Reviews 86(1-4): 175-202 (OpenAlex dates it 2007, online first).
- DOI or URL: 10.1016/j.earscirev.2007.08.008
- Access: paywalled; abstract not available through OpenAlex
- Read in this pass: not fetched (metadata only)
- Used for: suggested stage-2 reference for LIP area and frequency; the fields were not read.

### Hildreth2007
- Reference: Hildreth, W. (2007). Quaternary magmatism in the Cascades: geologic perspectives. U.S. Geological Survey Professional Paper 1744.
- DOI or URL: 10.3133/pp1744
- Access: open access (agency report)
- Read in this pass: abstract fetched; it has no lifetime or spacing figures. Stage-2 candidate for stratovolcano lifetimes and vent spacing.

### Web references used (secondary): Wikipedia pages Mount Fuji, Mauna Loa, Mauna Kea, Hawaiian-Emperor seamount chain, Ojos del Salado, Crater Lake, Yellowstone hotspot, Mount St. Helens, Mount Rainier, Mount Shasta, Cotopaxi, Lava lake, Large igneous province, Deccan Traps, Siberian Traps, Altiplano-Puna volcanic complex; USGS pages "How many active volcanoes are there on Earth?", Mauna Loa, Long Valley caldera. Their own cited references are named in the spot checks.


## Erosion, isostasy and hypsometry (ERO, ISO, HYP)

### Braun2013
- Reference: Braun, J. and Willett, S. D. (2013). A very efficient O(n), implicit and parallel method to solve the stream power equation governing fluvial incision and landscape evolution. Geomorphology 180-181:170-179.
- DOI or URL: 10.1016/j.geomorph.2012.10.008. Green copy listed at GFZ Potsdam pubman (https://gfzpublic.gfz.de/pubman/item/item_1945954); the page is a JavaScript application, no file link retrievable by script.
- Cited as: "Braun & Willett 2013 (Geomorphology 180-181:170-179)", "Braun & Willett" (spl.rs, routing.rs, params.rs k_spl).
- Ledger ids: ERO-001, ERO-078, ERO-079, HYP-030 (also the units of ERO-003/004).
- Access: paywalled (abstract elided in Semantic Scholar and OpenAlex); GFZ green copy exists but not retrievable here.
- Read so far (by the project): abstract only (via ResearchGate); code says "implemented from the paper".
- Read in this pass: not fetched (ScienceDirect 403; aggregators return no abstract). Secondary confirmation only: fastscapelib's `spl.hpp` header and Landlab's `FastscapeEroder` both name this paper as the implicit scheme they implement (see Landlab and Fastscapelib entries).
- Reference check: Crossref matches title, authors, volume 180-181, pages 170-179, year 2013 exactly. No equation or appendix number is cited by the code, so nothing to check there. Landlab's docstring points to "Braun & Willett 2013, Appendix B" for the flow-direction independence of the scheme.

### Yuan2019
- Reference: Yuan, X. P., Braun, J., Guerit, L., Rouby, D. and Cordonnier, G. (2019). A new efficient method to solve the stream power law model taking into account sediment deposition. Journal of Geophysical Research: Earth Surface 124(6):1346-1365.
- DOI or URL: 10.1029/2018JF004867. Open copies listed: https://hal.science/hal-02136641v1/document (blocked by Anubis), https://publishup.uni-potsdam.de/frontdoor/index/index/docId/49084 (metadata page with the first abstract sentence only), GFZ pubman item_4140893.
- Cited as: "Yuan, Braun, Guerit, Rouby & Cordonnier 2019", "Yuan et al. 2019 deposition coefficient G".
- Ledger ids: ERO-002, ERO-009, ERO-078.
- Access: open copy at https://publishup.uni-potsdam.de/frontdoor/index/index/docId/49084 and HAL hal-02136641 (neither readable by script here); paywalled at Wiley.
- Read so far (by the project): not stated (two-sentence note summary with the DOI).
- Read in this pass: abstract fetched in part (first sentence only, from the Potsdam landing page).
- Reference check: Crossref matches authors, title, volume 124, issue 6, pages 1346-1365. Nothing in the code cites an equation number; the value G = 1 is "not attributed" in the ledger and could not be checked against the paper (see spot check S3).

### Barnes2014
- Reference: Barnes, R., Lehman, C. and Mulla, D. (2014). Priority-flood: An optimal depression-filling and watershed-labeling algorithm for digital elevation models. Computers & Geosciences 62:117-127.
- DOI or URL: 10.1016/j.cageo.2013.04.024; open preprint https://arxiv.org/abs/1511.04463 (PDF https://arxiv.org/pdf/1511.04463).
- Cited as: "Barnes, Lehman & Mulla 2014 ... Algorithms 3 and 4" (pflood.rs; plan section 8.2).
- Ledger ids: ERO-080.
- Access: open copy (arXiv preprint) at https://arxiv.org/pdf/1511.04463.
- Read so far (by the project): in full ("implemented from the paper").
- Read in this pass: full text fetched (arXiv v1: abstract and the Algorithm 1 to 5 captions).
- Reference check: Crossref matches authors, title, volume 62, pages 117-127. The arXiv preprint carries the same citation line. Algorithm numbering in the preprint: 1 Priority-Flood, 2 Improved Priority-Flood (plain queue for pits), 3 Priority-Flood+epsilon, 4 Priority-Flood+FlowDirs, 5 Improved Priority-Flood+Watershed Labels. So "Algorithms 3 and 4" is the epsilon variant plus the flow-direction variant; consistent with the code's use (see S14). The preprint numbering may differ from the journal's; not checked.

### Landlab
- Reference: Landlab (MIT licence), component library. Components used for equations: ThresholdEroder, SimpleSubmarineDiffuser, CarbonateProducer, ExponentialWeatherer, ExponentialWeathererIntegrated, LinearDiffuser, Lithology/MaterialLayers. Recommended software citation (not cited by the code): Hobley, D. E. J., Adams, J. M., Nudurupati, S. S., et al. (2017). Creative computing with Landlab: an open-source toolkit for building, coupling, and exploring two-dimensional numerical models of Earth-surface dynamics. Earth Surface Dynamics 5(1):21-46.
- DOI or URL: https://github.com/landlab/landlab (files read at branch master, 2026-10-01: `src/landlab/components/{carbonate/carbonate_producer.py, marine_sediment_transport/simple_submarine_diffuser.py, threshold_eroder/threshold_eroder.py, weathering/exponential_weathering.py, weathering/exponential_weathering_integrated.py, diffusion/diffusion.py, stream_power/fastscape_stream_power.py}`); paper DOI 10.5194/esurf-5-21-2017 (Crossref confirms title, volume 5, issue 1, pages 21-46).
- Cited as: "Landlab `ThresholdEroder`", "Landlab `SimpleSubmarineDiffuser`", "Landlab default", "Landlab's `LinearDiffuser`", etc.
- Ledger ids: ERO-012, ERO-022, ERO-027, ERO-031, ERO-045, ERO-046, ERO-047, ERO-048, ERO-057, ERO-058, ERO-059, ERO-060, ERO-061, ERO-062, ERO-068.
- Access: open access (source code on GitHub).
- Read so far (by the project): component names only, "from source"; default values not shown as retrieved.
- Read in this pass: full text fetched (seven source files above, downloaded with curl and grepped; lithology/MaterialLayers not read).
- Reference check: all component names and paths exist. Two statements about Landlab in the code do not match the source: LinearDiffuser maps node diffusivities to links with the maximum, not the mean (S11); and the weatherer defaults are placeholders 1.0 and 1.0 (S6). The carbonate and submarine defaults match (S7 to S10). Caveat: master as of 2026-10-01; the version the code was read against is not recorded.

### Bosscher1992
- Reference: Bosscher, H. and Schlager, W. (1992). Computer simulation of reef growth. Sedimentology 39(3):503-512.
- DOI or URL: 10.1111/j.1365-3091.1992.tb02130.x.
- Cited as: "Bosscher & Schlager 1992" (carbonate.rs, plan section 8.2).
- Ledger ids: ERO-057.
- Access: paywalled (abstract not fetched).
- Read so far (by the project): no record; credited via Landlab.
- Read in this pass: not fetched. The growth law and the parameter ranges are known here only through Landlab's docstring (secondary source).
- Reference check: Crossref matches title, authors, volume 39, issue 3, pages 503-512, June 1992. Landlab's docstring gives the same citation and the law G = G_m tanh(I_0 e^(-kz)/I_k).

### Ahnert1976
- Reference: Ahnert, F. (1976). Brief description of a comprehensive three-dimensional process-response model of landform development. Zeitschrift fuer Geomorphologie, Supplementband 25:29-49.
- DOI or URL: no DOI found. Citation text taken from the Landlab docstring (`exponential_weathering.py`, lines 52-55: "Ahnert, F. (1976). Brief description of a comprehensive three-dimensional process-response model of landform development Z. Geomorphol. Suppl. 25, 29 - 49.").
- Cited as: "Ahnert 1976" (weathering.rs).
- Ledger ids: ERO-068.
- Access: book/journal supplement (not online).
- Read so far (by the project): not stated.
- Read in this pass: not fetched; reference text copied from Landlab's source.
- Reference check: the code writes only "Ahnert 1976 (soil production in the style of)"; Landlab uses the same phrase "in the style of Ahnert (1976)". Matches.

### Heimsath1997
- Reference: Heimsath, A. M., Dietrich, W. E., Nishiizumi, K. and Finkel, R. C. (1997). The soil production function and landscape equilibrium. Nature 388(6640):358-361.
- DOI or URL: 10.1038/41056. Page https://www.nature.com/articles/41056 (abstract reachable only through the cookie redirect).
- Cited as: "Heimsath et al. 1997" (weathering.rs, rock_class JSON sources.diffusivity).
- Ledger ids: ERO-068, ERO-069, ERO-070, ERO-099 and the 62 rock-class diffusivity rows.
- Access: paywalled (abstract fetched).
- Read so far (by the project): abstract unavailable ("The Heimsath abstract was unavailable"); rate constants not retrieved.
- Read in this pass: abstract fetched (full abstract, verbatim in S5). The paper's numerical parameters were not available; they are quoted in S5 from secondary sources.
- Reference check: Crossref matches authors, title, volume 388, issue 6640, pages 358-361, July 1997. The paper is a northern California (Tennessee Valley) study using 10Be and 26Al in bedrock under soil; the code uses its form but not its numbers (the 5 m depth scale is about ten times too deep, see S5).

### Roering1999
- Reference: Roering, J. J., Kirchner, J. W. and Dietrich, W. E. (1999). Evidence for nonlinear, diffusive sediment transport on hillslopes and implications for landscape morphology. Water Resources Research 35(3):853-870.
- DOI or URL: 10.1029/1998WR900090.
- Cited as: "Roering, Kirchner & Dietrich 1999, WRR 35(3):853-870" (rock_class JSON), "hillslope transport coefficient D".
- Ledger ids: ERO-099, ERO-102 to ERO-163.
- Access: paywalled (Wiley); not fetched.
- Read so far (by the project): no record.
- Read in this pass: not fetched (403 / no open copy).
- Reference check: Crossref matches. The title says "nonlinear" transport. The JSON text uses it as "a linear creep multiplier", which is a use of its linear coefficient D, whereas the paper's contribution is a nonlinear law with a critical slope; the code's linear diffusion is the small-slope limit. The paper's numeric D values were not read.

### Sklar2001
- Reference: Sklar, L. S. and Dietrich, W. E. (2001). Sediment and rock strength controls on river incision into bedrock. Geology 29(12):1087-1090.
- DOI or URL: 10.1130/0091-7613(2001)029<1087:SARSCO>2.0.CO;2.
- Cited as: "Sklar & Dietrich 2001, Geology 29(12):1087-1090 (river incision scales roughly with tensile strength^-2)".
- Ledger ids: ERO-097 and the bedrock rock-class rows ERO-102 to ERO-163.
- Access: paywalled (GeoScienceWorld 403); not fetched.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: Crossref matches title, authors, volume 29, issue 12, start page 1087; the end page 1090 is in the GeoScienceWorld URL (`/29/12/1087-1090/`). The attributed content (inverse square of tensile strength) is not checked.

### Stock1999
- Reference: Stock, J. D. and Montgomery, D. R. (1999). Geologic constraints on bedrock river incision using the stream power law. Journal of Geophysical Research 104(B3):4983-4993.
- DOI or URL: 10.1029/98JB02139.
- Cited as: "Stock & Montgomery 1999, JGR 104(B3):4983-4993 (lithologic K contrasts span orders of magnitude)".
- Ledger ids: ERO-097 and the bedrock rock-class rows.
- Access: paywalled (abstract elided at Semantic Scholar; Wiley 403).
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: Crossref matches title, volume 104, issue B3, pages 4983-4993. The content attributed is not checked. Note a trap: the DOI 10.1029/98JB02840 is a different paper (La Reunion seismic transect).

### Hjulstrom1935
- Reference: Hjulstroem, F. (1935). Studies of the morphological activity of rivers as illustrated by the River Fyris. Bulletin of the Geological Institution of the University of Upsala 25:221-527.
- DOI or URL: none (printed bulletin). Citation form from the Wikipedia article "Hjulstroem curve" (secondary).
- Cited as: "Hjulstrom 1935, Bull. Geol. Inst. Univ. Uppsala 25".
- Ledger ids: ERO-098 and the 11 loose-class rock rows.
- Access: book (not online).
- Read so far (by the project): no record.
- Read in this pass: not fetched (secondary only).
- Reference check: the citation matches the standard one; the code gives no pages (221-527). The paper is about entrainment, transport and deposition velocities by grain size, not about an erodibility multiplier on K; the multipliers 4 to 10 are the project's own (S26).

### Harel2016
- Reference: Harel, M.-A., Mudd, S. M. and Attal, M. (2016). Global analysis of the stream power law parameters based on worldwide 10Be denudation rates. Geomorphology 268:184-196.
- DOI or URL: 10.1016/j.geomorph.2016.05.035; author accepted manuscript https://www.research.ed.ac.uk/files/25670599/25669129._Harel_et_al_2016_author_accepted_manuscript_002_.pdf (403 from both WebFetch and curl).
- Cited as: "Harel, Mudd & Attal 2016", "worldwide 10Be denudation rates favour n > 1" (params.rs).
- Ledger ids: ERO-006.
- Access: open copy (author manuscript) at the Edinburgh URL above (not retrievable here); paywalled at Elsevier.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: Crossref gives title, authors, volume 268, pages 184-196. The code cites it by authors and year only (no journal), and the sources file likewise; the title confirms the paper is about stream power parameters from 10Be rates. The claim "favour n > 1" is not checked.

### Schott2024
- Reference: Schott, H., Galin, E., Guerin, E., Paris, A. and Peytavie, A. (2024). Terrain amplification using multi scale erosion. ACM Transactions on Graphics 43(4).
- DOI or URL: 10.1145/3658200; code https://github.com/H-Schott/MultiScaleErosion (MIT).
- Cited as: "Schott et al. 2024, Terrain Amplification using Multi-scale Erosion, ACM TOG 43(4)"; commit 964a3d5, `data/shaders/thermal.glsl`.
- Ledger ids: ERO-164, ERO-165.
- Access: not found as open text in this pass (not tried beyond Crossref).
- Read so far (by the project): code ported with its pass changed.
- Read in this pass: not fetched.
- Reference check: Crossref matches venue, volume 43, issue 4, and the title (with "Multi Scale", no hyphen). Author list from Crossref: Hugo Schott, Eric Galin, Eric Guerin, Axel Paris, Adrien Peytavie.

### Clark2000
- Reference: Clark, M. K. and Royden, L. H. (2000). Topographic ooze: Building the eastern margin of Tibet by lower crustal flow. Geology 28(8):703-706.
- DOI or URL: 10.1130/0091-7613(2000)28<703:TOBTEM>2.0.CO;2.
- Cited as: "Clark & Royden 2000, Geology 28(8):703-706" (spread.rs).
- Ledger ids: ISO-018, ISO-022.
- Access: paywalled (GeoScienceWorld 403); not fetched.
- Read so far (by the project): bibliographic check only.
- Read in this pass: not fetched.
- Reference check: Crossref matches title, authors, volume 28, issue 8, first page 703 (the end page 706 is not in the Crossref record, taken as written). Attribution of "D0 range 10^2 to 10^5" to this paper cannot be checked.

### Royden1997
- Reference: Royden, L. H., Burchfiel, B. C., King, R. W., Wang, E., Chen, Z., Shen, F. and Liu, Y. (1997). Surface deformation and lower crustal flow in eastern Tibet. Science 276(5313):788-790.
- DOI or URL: 10.1126/science.276.5313.788.
- Cited as: "Royden et al. 1997, Science 276:788-790".
- Ledger ids: ISO-018.
- Access: paywalled; not fetched.
- Read so far (by the project): bibliographic check only.
- Read in this pass: not fetched.
- Reference check: Crossref matches title, volume 276, issue 5313, pages 788-790.

### England1982
- Reference: England, P. and McKenzie, D. (1982). A thin viscous sheet model for continental deformation. Geophysical Journal of the Royal Astronomical Society 70(2):295-321 (now Geophysical Journal International). Correction: England, P. and McKenzie, D. (1983), 73(2):523-532.
- DOI or URL: 10.1111/j.1365-246X.1982.tb04969.x; correction 10.1111/j.1365-246x.1983.tb03328.x.
- Cited as: "England & McKenzie 1982 (GJRAS 70(2):295-321; correction 73:523, 1983)".
- Ledger ids: ISO-019.
- Access: paywalled; not fetched.
- Read so far (by the project): bibliographic check only.
- Read in this pass: not fetched.
- Reference check: Crossref matches title, volume 70, issue 2, pages 295-321, and the correction record (volume 73, issue 2, pages 523-532).

### Molnar1988
- Reference: Molnar, P. and Lyon-Caen, H. (1988). Some simple physical aspects of the support, structure, and evolution of mountain belts. In: Clark, S. P., Burchfiel, B. C. and Suppe, J. (eds), Processes in Continental Lithospheric Deformation, Geological Society of America Special Paper 218, pp. 179-208.
- DOI or URL: 10.1130/SPE218-p179.
- Cited as: "Molnar & Lyon-Caen 1988, GSA Special Paper 218:179-207".
- Ledger ids: ISO-019.
- Access: book (not online); paywalled chapter.
- Read so far (by the project): bibliographic check only.
- Read in this pass: not fetched.
- Reference check: title is not written in the code; Crossref gives the title above and pages 179-208. The code has 179-207, a one-page difference (immaterial, but the Crossref end page is 208). The editors are not verified in this pass.

### Bird1991
- Reference: Bird, P. (1991). Lateral extrusion of lower crust from under high topography in the isostatic limit. Journal of Geophysical Research 96(B6):10275-10286.
- DOI or URL: 10.1029/91JB00370.
- Cited as: "Bird 1991, JGR 96(B6):10275-10286".
- Ledger ids: ISO-019.
- Access: paywalled; not fetched.
- Read so far (by the project): bibliographic check only.
- Read in this pass: not fetched.
- Reference check: Crossref matches volume 96, issue B6, pages 10275-10286 (2026-10-01 Crossref query on the title).

### Rey2001
- Reference: Rey, P., Vanderhaeghe, O. and Teyssier, C. (2001). Gravitational collapse of the continental crust: definition, regimes and modes. Tectonophysics 342(3-4):435-449.
- DOI or URL: 10.1016/S0040-1951(01)00174-3.
- Cited as: "Rey, Vanderhaeghe & Teyssier 2001, Tectonophysics 342(3-4):435-449".
- Ledger ids: ISO-019.
- Access: paywalled; not fetched.
- Read so far (by the project): bibliographic check only.
- Read in this pass: not fetched.
- Reference check: Crossref matches title, volume 342, issue 3-4, pages 435-449.

### Beaumont2001
- Reference: Beaumont, C., Jamieson, R. A., Nguyen, M. H. and Lee, B. (2001). Himalayan tectonics explained by extrusion of a low-viscosity crustal channel coupled to focused surface denudation. Nature 414(6865):738-742.
- DOI or URL: 10.1038/414738a.
- Cited as: "Beaumont et al. 2001 (channel flow coupled to erosion), Nature 414:738" (build log only).
- Ledger ids: none directly (design background for ISO-018).
- Access: paywalled; not fetched.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: Crossref matches volume 414, issue 6865, pages 738-742.

### Cortial2019
- Reference: Cortial, Y., Peytavie, A., Galin, E. and Guerin, E. (2019). Procedural tectonic planets. Computer Graphics Forum 38(2):1-11 (Eurographics 2019).
- DOI or URL: 10.1111/cgf.13614; open copy https://hal.science/hal-02136820 (Anubis blocks scripted access).
- Cited as: "Cortial et al. 2019 (procedural tectonic planets; oceanic dampening z -= (1 - z / z_t) eps_o dt, z_r, z_a, z_t, z_c, section 4.5)".
- Ledger ids: ISO-011, ISO-015, ISO-025, ISO-026, ISO-027, ISO-008.
- Access: open copy at https://hal.science/hal-02136820 (not retrievable by script here).
- Read so far (by the project): the plate model is ported "from the paper" per other documents; not assessed here.
- Read in this pass: not fetched.
- Reference check: Crossref matches title "Procedural Tectonic Planets", volume 38, issue 2, pages 1-11. The equation and symbol names (z_r, z_a, z_t, z_c, section 4.5) could not be checked.

### Stein1992
- Reference: Stein, C. A. and Stein, S. (1992). A model for the global variation in oceanic depth and heat flow with lithospheric age. Nature 359(6391):123-129.
- DOI or URL: 10.1038/359123a0.
- Cited as: "Stein & Stein 1992, Nature 359:123-129 (GDH1)".
- Ledger ids: ISO-011, ISO-012, ISO-013, ISO-016, ISO-017.
- Access: paywalled (abstract truncated at Nature).
- Read so far (by the project): not stated; the 4.2 / 5.0 / 5.5 km depths are quoted without a source.
- Read in this pass: abstract fetched in part (first sentence only).
- Reference check: Crossref matches title, authors, volume 359, issue 6391, pages 123-129. The model name GDH1 is not in the part of the abstract that was reachable.

### Parsons1977
- Reference: Parsons, B. and Sclater, J. G. (1977). An analysis of the variation of ocean floor bathymetry and heat flow with age. Journal of Geophysical Research 82(5):803-827.
- DOI or URL: 10.1029/JB082i005p00803.
- Cited as: "Parsons & Sclater 1977 (mean ridge crest depth 2.5 km)" (build log).
- Ledger ids: ISO-014.
- Access: paywalled; not fetched.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: Crossref matches the title, volume 82, issue 5, pages 803-827. The sources file gives no year-title pair; the 1977 paper is the correct identification.

### Cogley1984
- Reference: Cogley, J. G. (1984). Continental margins and the extent and number of the continents. Reviews of Geophysics 22(2):101-122.
- DOI or URL: 10.1029/RG022i002p00101.
- Cited as: "Cogley 1984 (continental crust 41 % of the surface)".
- Ledger ids: HYP-012, HYP-013.
- Access: paywalled; not fetched.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: identified by Crossref; the title, volume, pages are as above. The 41 % figure is not checked.

### Shepard
- Reference: UNIDENTIFIED. Cited as "Shepard (mean shelf break 0 to -130 m)" in the build log with no year or title. Most likely a Francis P. Shepard book on submarine geology; I could not confirm which edition because the web search budget was spent and a Crossref query returned nothing.
- DOI or URL: none.
- Cited as: "Shepard's mean shelf break" (build log 2026-09-29 P1-17 partial).
- Ledger ids: HYP-013.
- Access: not found.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: cannot be checked. Suggest asking the author which Shepard work was meant.

### Peel2007
- Reference: Peel, M. C., Finlayson, B. L. and McMahon, T. A. (2007). Updated world map of the Koeppen-Geiger climate classification. Hydrology and Earth System Sciences 11(5):1633-1644.
- DOI or URL: 10.5194/hess-11-1633-2007; open access https://hess.copernicus.org/articles/11/1633/2007/.
- Cited as: "Peel, Finlayson & McMahon 2007 (HESS 11:1633)".
- Ledger ids: HYP-025.
- Access: open access.
- Read so far (by the project): no record.
- Read in this pass: abstract fetched; PDF text partly extracted (title page, abstract, methods, Africa section). The page with global main-group shares was not located.
- Reference check: Crossref matches title, volume 11, issue 5, pages 1633-1644. The abstract in the HTML page says "BWh (14.2%, Hot desert) followed by Aw (11.5%, Tropical savannah)"; the build log's shares B 30.2 / D 24.6 / A 19.0 / C 13.4 / E 12.8 % were not found in the text read (S19).

### Charette2010
- Reference: Charette, M. A. and Smith, W. H. F. (2010). The volume of Earth's ocean. Oceanography 23(2):112-114.
- DOI or URL: 10.5670/oceanog.2010.51; open PDF https://tos.org/oceanography/assets/docs/23-2_charette.pdf.
- Cited as: "Charette & Smith 2010, Oceanography 23(2):112-114 (ocean volume and mean depth on SRTM30_PLUS)".
- Ledger ids: HYP-007.
- Access: open access.
- Read so far (by the project): no record.
- Read in this pass: full text fetched (text of the 3-page article including Tables 1 and 2).
- Reference check: Crossref matches title, volume 23, issue 2, pages 112-114. The article says: "For this effort, we relied on the SRTM30PLUS V5 bathymetry database (Becker et al., 2009)", so the project's "on SRTM30_PLUS" is right.

### Tzathas2024
- Reference: Tzathas, P., Gailleton, B., Steer, P. and Cordonnier, G. (2024). Physically-based analytical erosion for fast terrain generation. Computer Graphics Forum 43(2) (Eurographics 2024).
- DOI or URL: 10.1111/cgf.15033; author PDF https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf.
- Cited as: "Tzathas et al. 2024 (author PDF)".
- Ledger ids: HYP-031.
- Access: open copy at the Inria URL above.
- Read so far (by the project): the note quotes timings from it.
- Read in this pass: full text fetched (curl, text extracted; Table 1, Table 2, thermal erosion section).
- Reference check: Crossref matches title, authors, volume 43, issue 2. The sources file gives only "Tzathas et al. 2024" with the PDF link; the title above is the PDF's.

### Jain2024
- Reference: Jain, A., Kerbl, B., Gain, J., Finley, B. and Cordonnier, G. (2024). FastFlow: GPU acceleration of flow and depression routing for landscape simulation. Computer Graphics Forum 43(7) (Pacific Graphics 2024).
- DOI or URL: 10.1111/cgf.15243; author PDF https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf.
- Cited as: "Jain et al. 2024 FastFlow".
- Ledger ids: HYP-031.
- Access: open copy at the Inria URL above.
- Read so far (by the project): the note quotes timings from it.
- Read in this pass: abstract fetched (title page and abstract only).
- Reference check: Crossref matches title, authors, volume 43, issue 7.

### Wikipedia-Laterite
- Reference: Wikipedia contributors, "Laterite", https://en.wikipedia.org/wiki/Laterite (web page, version as of 2026-10-01).
- DOI or URL: https://en.wikipedia.org/wiki/Laterite.
- Cited as: "Wikipedia: Laterite" (research note section 3).
- Ledger ids: ERO-073.
- Access: open access.
- Read so far (by the project): cited as a web source.
- Read in this pass: full page fetched (formation conditions).
- Reference check: not a scientific source; the statement attributed to it is present (S21).

### Braun2016
- Reference: Braun, J., Mercier, J., Guillocheau, F. and Robin, C. (2016). A simple model for regolith formation by chemical weathering. Journal of Geophysical Research: Earth Surface 121(11):2140-2171.
- DOI or URL: 10.1002/2016JF003914.
- Cited as: "Braun et al. 2016, 'regolith by chemical weathering'" (research note only).
- Ledger ids: none (not used in code).
- Access: paywalled; not fetched.
- Read so far (by the project): abstract via OpenAlex; coefficients not accessible.
- Read in this pass: not fetched.
- Reference check: Crossref gives the full title above (the note's phrase "regolith by chemical weathering" is a fragment). Authors: Braun, Mercier, Guillocheau, Robin.

### Tozer2019
- Reference: Tozer, B., Sandwell, D. T., Smith, W. H. F., Olson, C., Beale, J. R. and Wessel, P. (2019). Global bathymetry and topography at 15 arc sec: SRTM15+. Earth and Space Science 6(10):1847-1864.
- DOI or URL: 10.1029/2019EA000658 (gold open access; DOAJ record https://doaj.org/article/95db1757cac2402aaa9c72408bdc82ed).
- Cited as: "Tozer et al. 2019" (SRTM15+) in build log 2026-10-01.
- Ledger ids: HYP-001 to HYP-011, HYP-032 (as the dataset behind GMT earth_relief).
- Access: open access.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: Crossref matches. GMT's `earth_relief_01d` and the 30 arc-minute grid are resampled products of SRTM15+; the paper itself publishes no land fraction or median elevation, so the project's Earth numbers are its own computations on the grid (not independently checkable from the paper).

### Wessel2019-GMT
- Reference: Wessel, P., Luis, J. F., Uieda, L., Scharroo, R., Wobbe, F., Smith, W. H. F. and Tian, D. (2019). The Generic Mapping Tools version 6. Geochemistry, Geophysics, Geosystems 20(11):5556-5564.
- DOI or URL: 10.1029/2019GC008515.
- Cited as: "GMT earth_relief" (no paper cited by the project; this is GMT's standard citation).
- Ledger ids: HYP-001 to HYP-011, HYP-032.
- Access: not checked (Crossref only).
- Read so far (by the project): the files were pulled as `earth_720.f32`; the 1 degree vs 30 arc-minute naming is inconsistent (flag 13).
- Read in this pass: not fetched.
- Reference check: added for the dataset; Crossref matches the title and pages above.

### Fastscapelib
- Reference: fastscapelib 0.3.0, fastscape-lem (GPL-3.0). Its SPLEroder header says "The method is detailed in Braun and Willet's (2013) and has been slightly adapted."
- DOI or URL: https://github.com/fastscape-lem/fastscapelib (header read: `include/fastscapelib/eroders/spl.hpp`, main branch; `LICENSE`, GPL version 3).
- Cited as: "fastscapelib 0.3.0 (GPL-3.0)" (fastscape_compare.py, spl.rs).
- Ledger ids: HYP-028, HYP-029.
- Access: open access (source).
- Read so far (by the project): documented Python API only; the C++ was not read (the project's rule).
- Read in this pass: source header and constructor signature fetched; no default K, m or n exists (only `tolerance = 1e-3`).
- Reference check: licence GPL-3.0 confirmed. See S20.

### Telford1990
- Reference: Telford, W. M., Geldart, L. P. and Sheriff, R. E. (1990). Applied Geophysics, 2nd edition. Cambridge University Press.
- DOI or URL: 10.1017/CBO9781139167932 (Crossref record, published 1990-10-26).
- Cited as: "Telford, Geldart & Sheriff 1990 (Applied Geophysics, 2nd ed.)" (rock_class JSON sources.density_kg_m3).
- Ledger ids: none in ERO/ISO/HYP (density rows belong to another prefix).
- Access: book (not online).
- Read so far (by the project): "cf." only.
- Read in this pass: not fetched.
- Reference check: matches title and edition.

### Schon2015
- Reference: Schoen, J. H. (2015). Physical Properties of Rocks: Fundamentals and Principles of Petrophysics, 2nd edition. Developments in Petroleum Science 65, Elsevier.
- DOI or URL: 10.1016/C2014-0-03002-X (Crossref); chapter DOIs use 10.1016/B978-0-08-100404-3.
- Cited as: "Schoen 2015 (Physical Properties of Rocks, 2nd ed.)" (rock_class JSON).
- Ledger ids: none in ERO/ISO/HYP.
- Access: book (not online).
- Read so far (by the project): "cf." only.
- Read in this pass: not fetched.
- Reference check: title and year match the Crossref record; the series number 65 is from memory and was not verified.

### Not cited by the code but named in the task brief (identified for stage 2)
- Whipple, K. X. and Tucker, G. E. (1999). Dynamics of the stream-power river incision model: Implications for height limits of mountain ranges, landscape response timescales, and research needs. Journal of Geophysical Research 104(B8):17661-17674. DOI 10.1029/1999JB900120 (Crossref confirmed). Paywalled; not fetched.
- Fernandes, N. F. and Dietrich, W. E. (1997). Hillslope evolution by diffusive processes: The timescale for equilibrium adjustments. Water Resources Research 33(6):1307-1318. DOI 10.1029/97WR00534 (Crossref confirmed). Paywalled; not fetched.
- Amante, C. and Eakins, B. W. (2009). ETOPO1 1 arc-minute global relief model: procedures, data sources and analysis. NOAA Technical Memorandum NESDIS NGDC-24. Dataset DOI 10.7289/V5C8276M (DataCite: "ETOPO1 1 Arc-Minute Global Relief Model", NOAA NCEI, 2009). Open.
- Eakins, B. W. and Sharman, G. F. (2012). Hypsographic curve of Earth's surface from ETOPO1. NOAA National Geophysical Data Center, Boulder, CO. PDF https://www.ncei.noaa.gov/sites/default/files/2023-01/Hypsographic%20Curve%20of%20Earth%E2%80%99s%20Surface%20from%20ETOPO1.pdf (full text read; Table 1 is an image and was not readable). Companion: "World Ocean Volumes" https://www.ncei.noaa.gov/sites/default/files/2023-01/World%20Ocean%20Volumes.pdf (table read).
- Athy, L. F. (1930). Density, porosity, and compaction of sedimentary rocks. AAPG Bulletin 14(1):1-24. DOI 10.1306/3D93289E-16B1-11D7-8645000102C1865D. Not coded: the surface stage has no porosity or compaction (ERO-028).
- Sclater, J. G. and Christie, P. A. F. (1980). Continental stretching: an explanation of the post-mid-Cretaceous subsidence of the central North Sea basin. Journal of Geophysical Research 85(B7):3711-3739. DOI 10.1029/JB085iB07p03711. Not coded, as above.

### Not bibliographic
RichDEM and Barnes2013-Depressions (software, "were not opened", not relied on); docs/research/notes/surface_processes_climate.md; docs/deeptime-implementation-plan.md; docs/deeptime-design-decisions.md; STATUS.md build-log entries; rock_class JSON files; tools/review/p1-17-hyps.py and companions; the decisions of Ben (Tectonics scaling, Build range, Vertical scale, Sub-grid detail, squeeze, general flexure).

## Rivers, lakes and the young landscape (HYD, YNG)

### Leopold1953
- Reference: Leopold, Luna B. and Maddock, Thomas, Jr. 1953. The hydraulic geometry of stream channels and some physiographic implications. U.S. Geological Survey Professional Paper 252, 57 p.
- DOI or URL: https://doi.org/10.3133/pp252 ; open PDF https://pubs.usgs.gov/pp/0252/report.pdf
- Cited as: "Leopold & Maddock 1953: width ∝ Q^0.5, depth ∝ Q^0.4" (hydro/mod.rs comment; plan §8.4)
- Ledger ids: HYD-001, HYD-002 (HYD-003, HYD-004 are the project's own coefficients)
- Access: open access (USGS Publications Warehouse, scanned PDF)
- Read so far (by the project): nothing says the paper was read
- Read in this pass: full text fetched (OCR text layer of the PDF; the passage describing Figure 19, the paragraph introducing equations (1) to (3), and the legend of the figure on the next page)
- Reference check: matches (authors, year, series number). The paper is a Professional Paper, not a journal article. Exponent values: see Part 2, HYD-001 and HYD-002.

### Leopold1957
- Reference: Leopold, Luna B. and Wolman, M. Gordon. 1957. River channel patterns: braided, meandering, and straight. U.S. Geological Survey Professional Paper 282-B, pp. 39-85.
- DOI or URL: https://doi.org/10.3133/pp282B ; open PDF https://pubs.usgs.gov/pp/0282b/report.pdf
- Cited as: not cited by the project. Proposed as the citable source for the meandering threshold (HYD-027).
- Ledger ids: HYD-027 (proposed)
- Access: open access (USGS)
- Read so far (by the project): not read (project wrote the Earth figure "from memory, not sourced")
- Read in this pass: full text fetched (OCR text layer; the sections on sinuosity and Figure 46)
- Reference check: n/a (new source). Values: see Part 2, HYD-027.

### Schumm1963
- Reference: Schumm, Stanley A. 1963. Sinuosity of alluvial rivers on the Great Plains. Geological Society of America Bulletin 74(9):1089-1100.
- DOI or URL: https://doi.org/10.1130/0016-7606(1963)74[1089:SOAROT]2.0.CO;2 ; record https://pubs.usgs.gov/publication/70221081
- Cited as: not cited by the project (proposed for HYD-027)
- Ledger ids: HYD-027 (proposed)
- Access: only the abstract was reachable; open-access status not checked
- Read so far (by the project): not read
- Read in this pass: abstract fetched (USGS record). The abstract classifies 50 stable Great Plains rivers into five pattern types (tortuous, irregular, regular, transitional, straight) by sinuosity but gives no sinuosity numbers.
- Reference check: n/a.

### Barnes2014a
- Reference: Barnes, Richard; Lehman, Clarence; Mulla, David. 2014. Priority-Flood: an optimal depression-filling and watershed-labeling algorithm for digital elevation models. Computers & Geosciences 62:117-127.
- DOI or URL: https://doi.org/10.1016/j.cageo.2013.04.024 ; open preprint https://arxiv.org/abs/1511.04463 and https://richard.science/sci/2014_depressions.pdf
- Cited as: "Barnes, Lehman & Mulla 2014, Algorithm 3 with the pit queue of Algorithm 4" (hydro/mod.rs module doc); research note gives arXiv 1511.04463
- Ledger ids: HYD-014
- Access: open copy (author preprint at the two URLs above); the journal version is paywalled
- Read so far (by the project): "from the paper" (RichDEM, GPL, not opened); nothing says whether full text or abstract was read
- Read in this pass: full text fetched (author preprint: abstract, captions and pseudocode of Algorithms 1 to 5)
- Reference check: title, authors, year, volume, pages all match (the project gave no title). The citation "Algorithm 3 with the pit queue of Algorithm 4" does not match the paper's numbering: see Part 2, HYD-014.

### Barnes2014b
- Reference: Barnes, Richard; Lehman, Clarence; Mulla, David. 2014. An efficient assignment of drainage direction over flat surfaces in raster digital elevation models. Computers & Geosciences 62:128-135.
- DOI or URL: https://doi.org/10.1016/j.cageo.2013.01.009 ; open preprint https://richard.science/sci/2014_flats.pdf
- Cited as: "Garbrecht & Martz 1997, as improved by Barnes, Lehman & Mulla 2014 ... Computers & Geosciences 62:128-135" (network.rs doc of flat_masks)
- Ledger ids: HYD-015
- Access: open copy (author preprint); journal version paywalled
- Read so far (by the project): "from the paper"
- Read in this pass: full text fetched (author preprint: abstract, Steps 1 to 3 text)
- Reference check: matches exactly (title, authors, volume, pages). Algorithm check: see Part 2, HYD-015.

### Garbrecht1997
- Reference: Garbrecht, Jurgen and Martz, Lawrence W. 1997. The assignment of drainage direction over flat surfaces in raster digital elevation models. Journal of Hydrology 193(1-4):204-213.
- DOI or URL: https://doi.org/10.1016/S0022-1694(96)03138-1
- Cited as: "Garbrecht & Martz 1997" (named only through the Barnes paper)
- Ledger ids: HYD-015
- Access: paywalled (not opened; closed per OpenAlex)
- Read so far (by the project): named only via Barnes
- Read in this pass: not fetched (bibliographic record only, via OpenAlex)
- Reference check: record matches the name and year; the project gave no title or pages.

### Barnes2021
- Reference: Barnes, Richard; Callaghan, Kerry L.; Wickert, Andrew D. 2021. Computing water flow through complex landscapes, Part 3: Fill-Spill-Merge: flow routing in depression hierarchies. Earth Surface Dynamics 9(1):105-121.
- DOI or URL: https://doi.org/10.5194/esurf-9-105-2021 (open access)
- Cited as: "Fill-Spill-Merge, Barnes, Callaghan & Wickert 2021 (ESurf 9:105)"; code r-barnes/Barnes2020-FillSpillMerge (MIT)
- Ledger ids: HYD-049, HYD-050
- Access: open access
- Read so far (by the project): research note quotes a sentence; nothing says full text read
- Read in this pass: abstract fetched (Crossref record)
- Reference check: matches (authors, year, journal, volume, first page). Title has "Part 3" (project omitted it).

### Holdridge1967
- Reference: Holdridge, Leslie R. 1967. Life Zone Ecology (revised edition). Tropical Science Center, San Jose, Costa Rica.
- DOI or URL: none (book). A library scan is at https://app.ingemmet.gob.pe/biblioteca/pdf/Amb-56.pdf (title per its listing: "Life Zone Ecology by L.R. Holdridge, with photographic supplement prepared by Joseph A. Tosi Jr."). The scan's text layer starts at the table of contents, so I could not confirm the edition year from the scan itself; Lugo et al. 1999 cite it as "Holdridge, L.R. (1967) Life zone ecology. Tropical Science Center. San Jose, Costa Rica".
- Cited as: "L. R. Holdridge 1967, Life Zone Ecology, Tropical Science Center: PET = 58.93 x the mean annual biotemperature"
- Ledger ids: HYD-029, HYD-030, HYD-035, HYD-041
- Access: book (not online), plus an unofficial library scan
- Read so far (by the project): nothing says it was consulted
- Read in this pass: full text fetched (scan OCR; the passages on biotemperature and on the factor 58.93)
- Reference check: the title, publisher and the 58.93 factor match. The definition of biotemperature does not match the code: see Part 2, HYD-030.

### Lugo1999
- Reference: Lugo, A. E.; Brown, S. L.; Dodson, R.; Smith, T. S.; Shugart, H. H. 1999. The Holdridge life zones of the conterminous United States in relation to ecosystem mapping. Journal of Biogeography 26:1025-1038.
- DOI or URL: copy at https://epa-prgs.ornl.gov/radionuclides/documents/Holdridge_LifeZones.pdf
- Cited as: not cited by the project (found in this pass; secondary confirmation of the Holdridge formulae)
- Ledger ids: HYD-029, HYD-030 (confirmation)
- Access: open copy (ORNL/EPA host)
- Read so far (by the project): not read
- Read in this pass: full text fetched (partial: methods and the figure caption)
- Reference check: n/a.

### Zhao2022
- Reference: Zhao, Gang; Li, Yao; Zhou, Liangzhi; Gao, Huilin. 2022. Evaporative water loss of 1.42 million global lakes. Nature Communications 13:3686.
- DOI or URL: https://doi.org/10.1038/s41467-022-31125-6 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC9240014/ ; PDF copy https://www.atmos.albany.edu/facstaff/zhou/pdf/pdf_papers/Zhao_et_al_2022.pdf
- Cited as: "Zhao et al. 2022 ... 1,500 +- 150 km3 a year without the Caspian, about 650 mm over 2.3 million km2"
- Ledger ids: HYD-040, HYD-043, HYD-065
- Access: open access
- Read so far (by the project): nothing says how it was read
- Read in this pass: full text fetched (Results and Methods text; read from the PDF text layer and from PMC)
- Reference check: authors, year, journal, article number match. The project's "650 mm" and "2.3 million km2" are not in the text: see Part 2, HYD-040.

### Fu1981
- Reference: Fu, B. P. 1981. On the calculation of the evaporation from land surface (in Chinese). Scientia Atmospherica Sinica 5(1):23-31. (This line is from my recollection; I could not look it up, no source found with the tools left.)
- DOI or URL: none found
- Cited as: "Fu 1981; Zhang et al. 2004" (surface/climate.rs: Fu's form of the Budyko curve, omega 2.6)
- Ledger ids: HYD-034, HYD-035, YNG-011 (also MOI-068)
- Access: not found
- Read so far (by the project): "From papers"; nothing more
- Read in this pass: not fetched
- Reference check: UNIDENTIFIED beyond author and year in what I could verify. I tried OpenAlex by DOI (the paper has none) and Wikipedia (404). The equation as coded cannot be checked against this paper.

### Zhang2004
- Reference: Zhang, Lu; Hickel, Klaus; Dawes, Warrick R.; Chiew, Francis H. S.; Western, Andrew W.; Briggs, Peter R. 2004. A rational function approach for estimating mean annual evapotranspiration. Water Resources Research 40(2), W02502.
- DOI or URL: https://doi.org/10.1029/2003WR002710
- Cited as: "Fu 1981; Zhang et al. 2004"
- Ledger ids: HYD-034, HYD-035, YNG-011 (also MOI-068)
- Access: only the abstract was reachable; open-access status not checked
- Read so far (by the project): nothing stated
- Read in this pass: abstract fetched (OpenAlex). The abstract is about a rational-function model of ET/PET; it does not mention Fu's equation or omega.
- Reference check: authors, year, journal, volume match. Whether the paper contains "omega = 2.6" is not shown by the abstract. Needed for stage 2 (paywalled).

### Ohmura1992
- Reference: Ohmura, Atsumu; Kasser, Peter; Funk, Martin. 1992. Climate at the equilibrium line of glaciers. Journal of Glaciology 38(130):397-411.
- DOI or URL: https://doi.org/10.3189/S0022143000002276 ; Cambridge Core page https://www.cambridge.org/core/product/identifier/S0022143000002276/type/journal_article
- Cited as: "Ohmura et al. 1992 ... about -2 degC" (HYD-039); "Ohmura, Kasser & Funk 1992 (ELA fit P = 645 + 296 T + 9 T^2)" (MOI/BIO)
- Ledger ids: HYD-039; BIO-083 (and glaciers.rs)
- Access: open access (Cambridge Core)
- Read so far (by the project): "cited from memory, not re-read"
- Read in this pass: page fetched via summariser (equation and one sentence). My prompt contained the equation, so treat the quote as needing a second look.
- Reference check: authors, year, journal, volume and pages match.

### Messager2016
- Reference: Messager, Mathis L.; Lehner, Bernhard; Grill, Gunther; Nedeva, Irena; Schmitt, Oliver. 2016. Estimating the volume and age of water stored in global lakes using a geo-statistical approach. Nature Communications 7:13603.
- DOI or URL: https://doi.org/10.1038/ncomms13603 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC5171767/
- Cited as: "Messager et al. 2016, Nat. Commun. 7:13603 (HydroLAKES: 1.42 million lakes of 10 ha and more, 2.67 million km2, 1.8 % of land)"
- Ledger ids: HYD-063
- Access: open access
- Read so far (by the project): nothing stated
- Read in this pass: full text fetched (PMC, summariser: lake count, area, land share, comparison with Verpoorter)
- Reference check: matches. Note HydroLAKES has 1.43 million polygons, of which 1.42 million are natural lakes (the 1.42 million is natural lakes only).

### Verpoorter2014
- Reference: Verpoorter, Charles; Kutser, Tiit; Seekell, David A.; Tranvik, Lars J. 2014. A global inventory of lakes based on high-resolution satellite imagery. Geophysical Research Letters 41(18):6396-6402.
- DOI or URL: https://doi.org/10.1002/2014GL060641 (hybrid open access per OpenAlex); HAL record https://insu.hal.science/hal-01066344
- Cited as: "Verpoorter et al. 2014, GRL 41:6396 (117 million lakes over 0.002 km2, 5.0 million km2, 3.7 % of non-glaciated land)"
- Ledger ids: HYD-062, HYD-063
- Access: open access (hybrid)
- Read so far (by the project): the run review lists it among linked sources; nothing says the paper was read
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: matches (authors, year, journal, volume, pages, DOI).

### Downing2006
- Reference: Downing, John A.; Prairie, Yves T.; Cole, Jonathan J.; Duarte, Carlos M.; Tranvik, Lars J.; Striegl, Robert G.; McDowell, William H.; Kortelainen, Pirkko; Caraco, Nina F.; Melack, John M.; Middelburg, Jack J. 2006. The global abundance and size distribution of lakes, ponds, and impoundments. Limnology and Oceanography 51(5):2388-2397.
- DOI or URL: https://doi.org/10.4319/lo.2006.51.5.2388
- Cited as: not cited by the project (named in the task as a third lake-share estimate)
- Ledger ids: HYD-062/063 (comparison)
- Access: only the abstract was reachable (USGS record https://pubs.usgs.gov/publication/70028401); open-access status not checked
- Read so far (by the project): n/a
- Read in this pass: abstract fetched
- Reference check: n/a.

### MiddletonThomas1997
- Reference: Middleton, Nick and Thomas, David (eds.). 1997. World Atlas of Desertification, 2nd edition. UNEP / Arnold, London. (First edition: UNEP 1992, compiled by Middleton and Thomas.) The Wikipedia article on the aridity index cites "UNEP (1992) World Atlas of Desertification"; Zomer et al. 2022 cite "UNEP. World atlas of desertification - Second Edition (United Nations Environment Program, 1997)". I did not see the atlas.
- DOI or URL: none
- Cited as: "UNEP's arid and hyper-arid classes" (network.rs ARID_INDEX); "UNEP's hyper-arid, arid, semi-arid, dry sub-humid, humid" (biomes.rs)
- Ledger ids: HYD-045 (and BIO-038)
- Access: book (not online)
- Read so far (by the project): nothing; no document named
- Read in this pass: not fetched; values read from Zomer2022 (below) and Wikipedia
- Reference check: UNIDENTIFIED in the code (no document named); identified here as UNEP 1992 (first edition) or 1997 (second edition). The two editions differ in the hyper-arid limit: see Part 2, HYD-045.

### Zomer2022
- Reference: Zomer, Robert J.; Xu, Jianchu; Trabucco, Antonio. 2022. Version 3 of the Global Aridity Index and Potential Evapotranspiration Database. Scientific Data 9:409.
- DOI or URL: https://doi.org/10.1038/s41597-022-01493-1 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC9287331/
- Cited as: not cited by the project (open access source for the UNEP class table)
- Ledger ids: HYD-045 (confirmation), BIO-038
- Access: open access
- Read so far (by the project): n/a
- Read in this pass: full text fetched (PMC, summariser: the class table and the definition AI = MA_Prec/MA_ET0)
- Reference check: n/a (article number 409 is from my memory of the record, not seen; Crossref/EuropePMC gave volume 9 and year 2022).

### Peel2007
- Reference: Peel, Murray C.; Finlayson, Brian L.; McMahon, Thomas A. 2007. Updated world map of the Koppen-Geiger climate classification. Hydrology and Earth System Sciences 11(5):1633-1644.
- DOI or URL: https://doi.org/10.5194/hess-11-1633-2007 (open access) ; https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf
- Cited as (HYD): "Koppen's E (polar climate: warmest month below 10 degC)", no paper named
- Ledger ids: HYD-046 (and BIO-001 to BIO-016)
- Access: open access
- Read so far (by the project): n/a here
- Read in this pass: full text fetched (Table 1 with notes, the paragraphs on test order and on the Cs/Cw tie rule, the discussion with land shares)
- Reference check: matches. See the MOI file for the full Table 1 check.

### Porter1989
- Reference: Porter, Stephen C. 1989. Some geological implications of average Quaternary glacial conditions. Quaternary Research 32(3):245-261.
- DOI or URL: https://doi.org/10.1016/0033-5894(89)90092-6
- Cited as: "Porter 1989 (Quat. Res. 32:245-261): snowlines some 500 m lower than today" (glacial.rs; plan §8.2)
- Ledger ids: YNG-010
- Access: only the abstract was reachable; open-access status not checked
- Read so far (by the project): nothing stated
- Read in this pass: abstract fetched (OpenAlex, reconstructed from the inverted index)
- Reference check: matches (the project's title is absent; authors, year, volume, pages match).

### Porter2001
- Reference: Porter, Stephen C. 2001. Snowline depression in the tropics during the Last Glaciation. Quaternary Science Reviews 20(10):1067-1091. (OpenAlex lists the year as 2000, the online date; volume 20 issue 10 is the 2001 print.)
- DOI or URL: https://doi.org/10.1016/S0277-3791(00)00178-5
- Cited as: "Porter 2001" for "~900 m at the last glacial maximum" (glacial.rs comment)
- Ledger ids: YNG-010
- Access: paywalled
- Read so far (by the project): nothing stated
- Read in this pass: not fetched (OpenAlex has no abstract). The paper is about the tropics; whether it gives a global ~900 m figure is not established.
- Reference check: title and journal match what the code's "Porter 2001" would be; the 900 m claim is unchecked.

### Schott2024
- Reference: Schott, Hugo; Galin, Eric; Guerin, Eric; Paris, Axel; Peytavie, Adrien. 2024. Terrain amplification using multi-scale erosion. ACM Transactions on Graphics 43(4), article 1-12 (SIGGRAPH 2024).
- DOI or URL: https://doi.org/10.1145/3658200 ; software https://github.com/H-Schott/MultiScaleErosion (MIT, "Copyright (c) 2023 Schott"; commit 964a3d51 exists, dated 25 September 2026, message "warp control")
- Cited as: "in the style of Schott et al. 2024" (design decisions table; NOTICE.md)
- Ledger ids: YNG-015
- Access: not checked (ACM; the code is open)
- Read so far (by the project): NOTICE.md says shaders and main.cpp read on 2026-09-29
- Read in this pass: abstract fetched (OpenAlex); commit and licence checked on GitHub
- Reference check: matches (the project gave no title). The commit hash named in the sources file exists.

### Mentioned for later, not implemented (no ledger ids)
Each verified only as a bibliographic record (OpenAlex), not read.
- Cordonnier2019: Cordonnier, Guillaume; Bovy, Benoit; Braun, Jean. 2019. A versatile, linear complexity algorithm for flow routing in topographies with depressions. Earth Surface Dynamics 7(2):549-562. https://doi.org/10.5194/esurf-7-549-2019 (open access journal). The project's "Cordonnier et al. 2019 basin graph and cost anchor" fits this paper; the "100-400 s per call at 16,384^2" figure is not checked.
- Genevaux2013: Genevaux, Jean-David; Galin, Eric; Guerin, Eric; Peytavie, Adrien; Benes, Bedrich. 2013. Terrain generation using procedural models based on hydrology. ACM Transactions on Graphics 32(4):1-13. https://doi.org/10.1145/2461912.2461996
- Paris2023: Paris, Axel; Guerin, Eric; Collon, Pauline; Galin, Eric. 2023. Authoring and simulating meandering rivers. ACM Transactions on Graphics 42(6):1-14. https://doi.org/10.1145/3618350
- Gailleton2024: Gailleton, Boris; Steer, Philippe; Davy, Philippe; Schwanghart, Wolfgang; Bernard, Thomas. 2024. GraphFlood 1.0: an efficient algorithm to approximate 2D hydrodynamics for landscape evolution models. Earth Surface Dynamics 12(6):1295-1313. https://doi.org/10.5194/esurf-12-1295-2024
- Liang2015 (the model behind pyDeltaRCM): Liang, Man; Voller, Vaughan R.; Paola, Chris. 2015. A reduced-complexity model for river delta formation, Part 1: Modeling deltas with channel dynamics. Earth Surface Dynamics 3(1):67-86. https://doi.org/10.5194/esurf-3-67-2015. The project names "pyDeltaRCM" (software); I did not check that package.
- Fairfield1991: Fairfield, John and Leymarie, Pierre. 1991. Drainage networks from grid digital elevation models. Water Resources Research 27(5):709-717. https://doi.org/10.1029/90WR02658. The project names "Fairfield & Leymarie rho8" with "no citation"; this is the paper that introduces rho8 (the name rho8 itself I did not see in the abstract, which I could not fetch).
- Braun2013: Braun, Jean and Willett, Sean D. 2013 (online 2012). A very efficient O(n), implicit and parallel method to solve the stream power equation governing fluvial incision and landscape evolution. Geomorphology 180-181:170-179. https://doi.org/10.1016/j.geomorph.2012.10.008
- Yuan2019: Yuan, Xiaoping; Braun, Jean; Guerit, Laure; Rouby, Delphine; Cordonnier, Guillaume. 2019. A new efficient method to solve the stream power law model taking into account sediment deposition. Journal of Geophysical Research: Earth Surface 124(6):1346-1365. https://doi.org/10.1029/2018JF004867
- Nobre2011: Nobre, A. D.; Cuartas, L. A.; Hodnett, M.; Renno, C. D.; Rodrigues, G.; Silveira, A.; Waterloo, M.; Saleska, S. 2011. Height Above the Nearest Drainage, a hydrologically relevant new terrain model. Journal of Hydrology 404(1-2):13-29. https://doi.org/10.1016/j.jhydrol.2011.03.051

### Candidate sources for claims that have none (not cited by the project; suggestions, not read)
- Hock2003: Hock, Regine. 2003. Temperature index melt modelling in mountain areas. Journal of Hydrology 282(1-4):104-115. https://doi.org/10.1016/S0022-1694(03)00257-9 (closed access). Candidate for the degree-day factor range (HYD-037); abstract not available to me, so I do not know what range it gives.
- Wang2018: Wang, Wei; Lee, Xuhui; et al. 2018. Global lake evaporation accelerated by changes in surface energy allocation in a warmer climate. Nature Geoscience 11(6):410-414. https://doi.org/10.1038/s41561-018-0114-8 (closed). Candidate for lake evaporation by latitude (HYD-041/044); not read.

### Not bibliographic
Minecraft's OverworldBiomeBuilder and the 1.21.1 carvers (game code), RichDEM (GPL software, deliberately not opened), Landlab LakeMapperBarnes and HeightAboveDrainageCalculator (software options, no ids), Undiscovered Worlds (software), the plan, STATUS.md, run review and decision log (repo documents).

---

## The amplifier and glacial landforms (AMP, GLA)

### Schott2024
- Reference: Schott, H., Galin, E., Guérin, E., Paris, A. and Peytavie, A. (2024). Terrain Amplification using Multi-scale Erosion. ACM Transactions on Graphics 43(4), article 145, pp. 1-12 (SIGGRAPH 2024).
- DOI or URL: 10.1145/3658200; code https://github.com/H-Schott/MultiScaleErosion (MIT licence; shaders in data/shaders/).
- Cited as: "Schott et al. 2024, Terrain Amplification using Multi-scale Erosion (ACM TOG 43(4)); code H-Schott/MultiScaleErosion (MIT, commit 964a3d5)"
- Ledger ids: AMP-001, 006, 007, 008, 009, 010, 023, 028, 034, 038 (and the method behind 003, 005)
- Access: open copy at the authors' repository (code only); the paper PDF was not reachable (ACM 403, HAL behind an anti-bot wall)
- Read so far (by the project): the shaders (ported "equations, not lines"); the paper text probably not read in full
- Read in this pass: full text of two shader files fetched (thermal.glsl, erosion.glsl at branch main; the project's commit 964a3d5 was not checked); the paper itself not fetched
- Reference check: authors, year, venue, volume and DOI are right (search results and the author's site give July 2024, ACM Trans. Graph., presented at SIGGRAPH 2024). The shader values the code attributes to Schott are partly misread: see spot checks AMP-034 and AMP-038.

### Barnes2014
- Reference: Barnes, R., Lehman, C. and Mulla, D. (2014). Priority-Flood: An optimal depression-filling and watershed-labeling algorithm for digital elevation models. Computers & Geosciences 62: 117-127.
- DOI or URL: 10.1016/j.cageo.2013.04.024; arXiv https://arxiv.org/abs/1511.04463
- Cited as: "Barnes, Lehman & Mulla 2014 (Priority-Flood+epsilon)"
- Ledger ids: AMP-016, AMP-017
- Access: open copy (arXiv) at https://arxiv.org/abs/1511.04463
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (arXiv abs page). The arXiv PDF came back as unreadable binary, so the epsilon variant's algorithm number was not read.
- Reference check: authors, year, journal, volume and pages match. The abstract says the algorithm "can also be adapted to label watersheds and determine flow directions through either incremental elevation changes or depression carving. In the case of incremental elevation changes, the algorithm includes safety checks not present in prior works". That supports the existence of an epsilon (incremental elevation) variant; the name "Priority-Flood+epsilon" as written in the code could not be checked in the text.

### Tzathas2024
- Reference: Tzathas, P., Gailleton, B., Steer, P. and Cordonnier, G. (2024). Physically-based analytical erosion for fast terrain generation. Computer Graphics Forum 43(2) (Eurographics 2024), e15033.
- DOI or URL: 10.1111/cgf.15033; preprint https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf (also HAL hal-04525371)
- Cited as: "Tzathas 2024 (Analytical Terrains, Eurographics; Inria research-only code)"
- Ledger ids: relevant to AMP-034 (research note only; not cited by code)
- Access: open copy (Inria preprint) at the URL above, but the PDF came back as binary and the Wiley and EG library pages returned 403
- Read so far (by the project): the PDF was linked; nothing says it was read
- Read in this pass: search snippet only (title, authors, venue; says the method also covers "hillslope erosion and landslides (thermal erosion)")
- Reference check: authors, year and venue match. The volume (43(2)) and article number come from the DOI and search results. Whether the paper uses atan 30 degrees (0.57) could not be read.

### Hergarten2021
- Reference: Hergarten, S. (2021). Modeling glacial and fluvial landform evolution at large scales using a stream-power approach. Earth Surface Dynamics 9: 937-952.
- DOI or URL: https://esurf.copernicus.org/articles/9/937/2021/ (open access)
- Cited as: "Hergarten 2021, Earth Surf. Dynam. 9:937-952"
- Ledger ids: GLA-009, GLA-022, GLA-025
- Access: open access
- Read so far (by the project): not stated
- Read in this pass: full text fetched (article page, two queries; relayed quotes of Eqs 10-13, 19, 23)
- Reference check: authors, year, journal, volume and pages match. Content check: see spot checks GLA-009 and GLA-022; the paper's glacial law is stream power in ice flux, not the sliding-speed law the code uses.

### goSPL
- Reference: goSPL documentation, technical guide "ice" (Geodels/gospl on GitHub; GPL). Software; no paper cited.
- DOI or URL: https://github.com/Geodels/gospl (raw: docs/tech_guide/ice.rst)
- Cited as: "goSPL documentation (ice.rst, ela_from_temperature.py)"
- Ledger ids: GLA-009, GLA-036
- Access: open (documentation)
- Read so far (by the project): documentation only
- Read in this pass: full text of ice.rst fetched. ela_from_temperature.py not fetched.
- Reference check: the quote about the removed SIA solve is exact (see spot check). The temperature-based ELA rule the code credits to goSPL did not appear in ice.rst (which gives a precipitation-ramp mass balance), so it presumably lives in ela_from_temperature.py, not fetched.

### Oerlemans2011
- Reference: Oerlemans, J. Minimal Glacier Models. Utrecht Publishing & Archiving Services (Igitur), Utrecht. OpenAlex lists the book as 2008 with a handle https://dspace.library.uu.nl/handle/1874/224622; the project says 2011 (probably the second edition).
- DOI or URL: handle 1874/224622 (returned 403)
- Cited as: "Oerlemans 2011, Minimal Glacier Models"
- Ledger ids: GLA-005, GLA-006
- Access: book (open PDF at Utrecht, not reachable by my tool)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: edition and year unverified (2008 versus 2011).

### Cuffey2010
- Reference: Cuffey, K. M. and Paterson, W. S. B. (2010). The Physics of Glaciers, 4th edition. Butterworth-Heinemann (Elsevier/Academic Press), 693 pp. (ISBN 978-0-12-369461-4).
- DOI or URL: none
- Cited as: "Cuffey & Paterson 2010, The Physics of Glaciers, 4th ed., section 8"
- Ledger ids: GLA-010, GLA-011, GLA-012
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched; the rate-factor value was read in a paper that cites it (Prasicek et al. 2018, see spot checks)
- Reference check: the book exists as cited. The location "section 8" was not checked (I did not see the book's contents).

### Harbor1992
- Reference: Harbor, J. M. (1992). Numerical modeling of the development of U-shaped valleys by glacial erosion. Geological Society of America Bulletin 104(10): 1364-1375.
- DOI or URL: 10.1130/0016-7606(1992)104<1364:NMOTDO>2.3.CO;2
- Cited as: "Harbor 1992, GSA Bull. 104:1364-1375"
- Ledger ids: GLA-001, GLA-003, GLA-054, GLA-055
- Access: paywalled (abstract only, via OpenAlex)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, journal, volume, issue and pages match. The abstract does not state the exponent l = 2 (see spot check).

### Humphrey1994
- Reference: Humphrey, N. F. and Raymond, C. F. (1994). Hydrology, erosion and sediment production in a surging glacier: Variegated Glacier, Alaska, 1982-83. Journal of Glaciology 40(136): 539-552.
- DOI or URL: 10.1017/S0022143000012429 (OpenAlex; the Cambridge page for this journal also resolves under 10.3189). Journal of Glaciology back issues are open.
- Cited as: "Humphrey & Raymond 1994, J. Glaciol. 40:539-552"
- Ledger ids: GLA-001, GLA-003
- Access: open access (Cambridge archive), abstract fetched
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (via OpenAlex). The title was not in the project's list; I took it from OpenAlex.
- Reference check: authors, volume, issue and pages match.

### Hallet1996
- Reference: Hallet, B., Hunter, L. and Bogen, J. (1996). Rates of erosion and sediment evacuation by glaciers: a review of field data and their implications. Global and Planetary Change 12(1-4): 213-235.
- DOI or URL: 10.1016/0921-8181(95)00021-6
- Cited as: "Hallet, Hunter & Bogen 1996, Global Planet. Change 12:213-235"
- Ledger ids: GLA-002, GLA-041, GLA-043
- Access: paywalled; abstract not available through OpenAlex
- Read so far (by the project): the project quotes its rates without page references; reading not recorded
- Read in this pass: not fetched (metadata only). A later paper (Cook et al. 2020) cites the title and was read for context.
- Reference check: authors, year, journal, volume and pages match.

### Sugden1976
- Reference: Sugden, D. E. and John, B. S. (1976). Glaciers and Landscape: A Geomorphological Approach. Edward Arnold, London.
- DOI or URL: none (book)
- Cited as: "Sugden & John 1976 (selective linear erosion)"
- Ledger ids: GLA-014, GLA-016
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: not checked (the full reference is mine from the citation form; no search done).

### Alley1998
- Reference: Alley, R. B., Lawson, D. E., Evenson, E. B., Strasser, J. C. and Larson, G. J. (1998). Glaciohydraulic supercooling: a freeze-on mechanism to create stratified, debris-rich basal ice: II. Theory. Journal of Glaciology 44(148): 563-569.
- DOI or URL: 10.1017/S0022143000002070 (OpenAlex)
- Cited as: "Alley et al. 1998, J. Glaciol. 44 (glaciohydraulic supercooling, theory 1.2 to 1.7)"
- Ledger ids: GLA-018
- Access: open access (Cambridge archive); abstract fetched
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (via OpenAlex)
- Reference check: the citation is to Part II (Theory); Part I (field evidence) is Lawson et al. 1998, 44(148): 547-562. The abstract does not contain the 1.2-1.7 ratios.

### Alley2003
- Reference: Alley, R. B., Lawson, D. E., Larson, G. J., Evenson, E. B. and Baker, G. S. (2003). Stabilizing feedbacks in glacier-bed erosion. Nature 424(6950): 758-760.
- DOI or URL: 10.1038/nature01839
- Cited as: "Alley et al. 2003, Nature 424:758-760"
- Ledger ids: GLA-018
- Access: paywalled (abstract only, via Europe PMC)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, journal, volume and pages match.

### Porter1989
- Reference: Porter, S. C. (1989). Some geological implications of average Quaternary glacial conditions. Quaternary Research 32(3): 245-261.
- DOI or URL: 10.1016/0033-5894(89)90092-6
- Cited as: "Porter 1989, Quat. Res. 32:245-261"
- Ledger ids: GLA-030
- Access: paywalled (abstract only, via OpenAlex)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: matches.

### Porter2001
- Reference: Porter, S. C. (2001). Snowline depression in the tropics during the Last Glaciation. Quaternary Science Reviews 20(10): 1067-1091.
- DOI or URL: 10.1016/S0277-3791(00)00178-5
- Cited as: "Porter 2001 (about 900 m at the last glacial maximum)"
- Ledger ids: GLA-030
- Access: paywalled; abstract not available through OpenAlex
- Read so far (by the project): not stated; no bibliographic detail was given
- Read in this pass: not fetched (metadata only)
- Reference check: this is my identification of an otherwise vague citation, found by author and year. The paper is about the TROPICS, while the code uses "about 900 m" as a general last-glacial-maximum snowline lowering. The 900 m figure could not be checked. Identification is likely but not confirmed.

### Ohmura1992
- Reference: Ohmura, A., Kasser, P. and Funk, M. (1992). Climate at the equilibrium line of glaciers. Journal of Glaciology 38(130): 397-411.
- DOI or URL: 10.3189/S0022143000002276 (Cambridge page https://www.cambridge.org/core/journals/journal-of-glaciology/article/climate-at-the-equilibrium-line-of-glaciers/8DCF1C24E32A6781CC365853535CD16F)
- Cited as: "Ohmura, Kasser & Funk 1992"
- Ledger ids: GLA-031, GLA-037, GLA-039
- Access: open access (Cambridge archive; the page shows the full text)
- Read so far (by the project): from memory, not re-read (STATUS 2026-10-01)
- Read in this pass: full text partly fetched (page text via WebFetch: abstract, the regression sentence, the temperature definition, the standard error; Table 3 was not readable)
- Reference check: authors, year, journal, volume, issue and pages match.

### Svensson1959
- Reference: Svensson, H. (1959). Is the cross-section of a glacial valley a parabola? Journal of Glaciology 3(25): 362-363.
- DOI or URL: 10.3189/S0022143000017032 (OpenAlex)
- Cited as: "Svensson 1959, J. Glaciol. 3:362-363"
- Ledger ids: GLA-054, GLA-055
- Access: open access (Cambridge archive); abstract null in OpenAlex
- Read so far (by the project): not stated
- Read in this pass: not fetched (metadata only)
- Reference check: authors, year, volume, issue and pages match. The title is from my own OpenAlex title query (cross-section, glacial valley, parabola, year 1959), which returned this item; the abstract was null.

### Sugden1978
- Reference: Sugden, D. E. (1978). Glacial erosion by the Laurentide ice sheet. Journal of Glaciology 20(83): 367-391.
- DOI or URL: 10.3189/S0022143000013915 (Crossref/OpenAlex)
- Cited as: "Sugden 1978, J. Glaciol. 20:367-391"
- Ledger ids: GLA-057
- Access: open access (Cambridge archive); abstract fetched
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (via OpenAlex)
- Reference check: matches.

### Egholm2009
- Reference: Egholm, D. L., Nielsen, S. B., Pedersen, V. K. and Lesemann, J.-E. (2009). Glacial effects limiting mountain height. Nature 460(7257): 884-887.
- DOI or URL: 10.1038/nature08263
- Cited as: "Egholm et al. 2009, Nature 460:884-887 (glacial buzzsaw)"
- Ledger ids: GLA-064
- Access: paywalled (abstract only, via Europe PMC)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, journal, volume and pages match.

### Mitchell2015
- Reference: Mitchell, S. G. and Humphries, E. E. (2015). Glacial cirques and the relationship between equilibrium line altitudes and mountain range height. Geology 43(1): 35-38.
- DOI or URL: 10.1130/G36180.1
- Cited as: "Mitchell & Humphries 2015, Geology 43:35-38"
- Ledger ids: GLA-062, GLA-064
- Access: paywalled (abstract only, reconstructed from OpenAlex; GeoScienceWorld returned 403)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, journal, volume and pages match. OpenAlex dates it 2014 (online first).

### Cook2020 (context, not cited by the code)
- Reference: Cook, S. J., Swift, D. A., Kirkbride, M. P., Knight, P. G. and Waller, R. I. (2020). The empirical basis for modelling glacial erosion rates. Nature Communications 11: 759.
- DOI or URL: 10.1038/s41467-020-14583-8 (https://pmc.ncbi.nlm.nih.gov/articles/PMC7005307/)
- Access: open access
- Read in this pass: full text fetched (several queries)
- Used for: independent statements of the erosion-rule constants.

### Prasicek2018 (context, not cited by the code)
- Reference: Prasicek, G., Herman, F., Robl, J. and Braun, J. (2018). Glacial steady state topography controlled by the coupled influence of tectonics and climate. Journal of Geophysical Research: Earth Surface 123.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC6055901/
- Access: open access
- Read in this pass: full text fetched (Table 1 values as relayed)
- Used for: the rate factor A quoted from Cuffey and Paterson, and the pairings of K_g and l.

### Argudo2020 and Cordonnier2023 (research note only)
- Cordonnier, G., Jouvet, G., Peytavie, A., Braun, J., Cani, M.-P., Benes, B., Galin, E., Guérin, E. and Gain, J. (2023). Forming Terrains by Glacial Erosion. ACM Transactions on Graphics 42(4): 1-14. DOI 10.1145/3592422. Access: not checked (metadata only via OpenAlex). Authors in the metadata are as listed here.
- Argudo, O. et al. (2020). Simulation, modeling and authoring of glaciers. ACM Transactions on Graphics 39(6). Not verified in this pass (OpenAlex search returned nothing); the title and venue are from the project's note, unconfirmed.

#### Not bibliographic
Catmull-Rom interpolation (named only), the design-decisions log entries (Ben), `docs/research/notes/grid-artifact-audit.md` and `surface_processes_climate.md`, the "standard atmosphere" lapse rate, the Aletsch Glacier example (checked below), Neoproterozoic snowball diamictites and the Dwyka Group (naming only), `tools/review/*` and `fastscape_compare.py` (fastscapelib, GPL), `NOTICE.md`.

## Rock, strata and structure (RCK, STR, DEF)

### Telford1990
- Reference: Telford, W. M., Geldart, L. P. & Sheriff, R. E. (1990). Applied Geophysics, 2nd ed. Cambridge University Press. ISBN 0-521-33938-3 (Open Library record, 1990 edition).
- DOI or URL: no DOI. Library copy (lending only, not readable by fetch): https://archive.org/details/appliedgeophysic0000telf (djvu text returned HTTP 401).
- Cited as: "Telford, Geldart & Sheriff 1990 (Applied Geophysics, 2nd ed.)"; "the rock-density tables of ..."
- Ledger ids: RCK-001
- Access: book (not online; archive.org lending copy only)
- Read so far (by the project): not stated
- Read in this pass: not fetched (bibliographic record only)
- Reference check: authors, year, edition match the Open Library record. No table or page number given in the code, so the density table cannot be pinned (the density table is in the physical-properties chapter; table number not verified).

### Schon2015
- Reference: Schön, J. H. (2015). Physical Properties of Rocks: Fundamentals and Principles of Petrophysics, 2nd ed. Developments in Petroleum Science 65. Elsevier. ISBN 978-0-08-100404-3 (ISBN recalled from memory and matched to the title by an Open Library lookup, which dates the record 2016; the printed year is 2015 per the code).
- DOI or URL: no DOI found. Open Library: https://openlibrary.org/isbn/9780081004043
- Cited as: "Schön 2015 (Physical Properties of Rocks, 2nd ed.)"
- Ledger ids: RCK-001
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched (bibliographic record only)
- Reference check: title and edition exist; year 2015 vs 2016 record unresolved. No table or page number in the code.

### Sklar2001
- Reference: Sklar, L. S. & Dietrich, W. E. (2001). Sediment and rock strength controls on river incision into bedrock. Geology 29(12):1087-1090.
- DOI or URL: 10.1130/0091-7613(2001)029<1087:SARSCO>2.0.CO;2
- Cited as: "Sklar & Dietrich 2001, Geology 29(12):1087-1090"
- Ledger ids: RCK-006
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex, https://api.openalex.org/works/doi:10.1130/0091-7613(2001)029<1087:SARSCO>2.0.CO;2)
- Reference check: authors, year, journal, volume, issue and pages match exactly. The abstract supports the attributed claim (see spot check RCK-006).

### Stock1999
- Reference: Stock, J. D. & Montgomery, D. R. (1999). Geologic constraints on bedrock river incision using the stream power law. Journal of Geophysical Research 104(B3):4983-4993.
- DOI or URL: 10.1029/98JB02139
- Cited as: "Stock & Montgomery 1999, JGR 104(B3):4983-4993"
- Ledger ids: RCK-006, RCK-007
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: authors, year, volume, issue, pages match. Abstract content supports "lithologic K contrasts span orders of magnitude".

### Hjulstrom1935
- Reference: Hjulström, F. (1935). Studies of the morphological activity of rivers as illustrated by the River Fyris. Bulletin of the Geological Institution of the University of Upsala 25:221-527.
- DOI or URL: no DOI. No open scan found.
- Cited as: "Hjulström 1935, Bull. Geol. Inst. Univ. Uppsala 25"
- Ledger ids: RCK-008
- Access: not found (journal volume from 1935; not located online)
- Read so far (by the project): not stated
- Read in this pass: secondary source only (Wikipedia, "Hjulström curve": https://en.wikipedia.org/wiki/Hjulstr%C3%B6m_curve)
- Reference check: Wikipedia gives the citation as "Hjulstrom, F. (1935). Studies of the morphological activity of rivers as illustrated by the River Fyris, Bulletin. Geological Institute Upsalsa, 25, 221-527." The code's volume matches; pages are missing in the code (221-527). The work is about loose grain entrainment, deposition and transport velocities in a river; it is not about bedrock incision, so it can support at most a ranking of loose classes (see spot check RCK-008).

### Roering1999
- Reference: Roering, J. J., Kirchner, J. W. & Dietrich, W. E. (1999). Evidence for nonlinear, diffusive sediment transport on hillslopes and implications for landscape morphology. Water Resources Research 35(3):853-870.
- DOI or URL: 10.1029/1998WR900090
- Cited as: "Roering, Kirchner & Dietrich 1999 (WRR 35(3):853-870)"
- Ledger ids: RCK-009, RCK-010, RCK-011, RCK-012
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: authors, year, volume, issue, pages match. The paper's finding is a nonlinear (gradient-dependent) transport law, not a lithology-dependent linear coefficient; it is cited only for the idea of a creep coefficient D.

### Heimsath1997
- Reference: Heimsath, A. M., Dietrich, W. E., Nishiizumi, K. & Finkel, R. C. (1997). The soil production function and landscape equilibrium. Nature 388(6640):358-361.
- DOI or URL: 10.1038/41056
- Cited as: "Heimsath et al. 1997, Nature 388:358-361"
- Ledger ids: RCK-010
- Access: paywalled (abstract only; OpenAlex had no abstract)
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: authors, year, volume, pages match the OpenAlex record.

### Winter2010
- Reference: Winter, J. D. (2010). Principles of Igneous and Metamorphic Petrology, 2nd ed. Prentice Hall / Pearson. (Open Library lists Pearson editions dated 2009-2014; the printed year of the 2nd edition is 2010 per the code and was not separately verified.)
- DOI or URL: no DOI. Author's lecture slides for the chapters (open): https://www.whitman.edu/geology/winter/JDW_PetClass.htm
- Cited as: "Winter 2010, Principles of Igneous and Metamorphic Petrology, 2nd ed., ch. 21 and 25"
- Ledger ids: RCK-039, RCK-040, RCK-041, RCK-042, RCK-043, RCK-044, RCK-052
- Access: book (not online); author's chapter slides are open but the .ppt came back as binary and was not read
- Read so far (by the project): not stated (chapter level only)
- Read in this pass: search result snippet only / chapter titles fetched from the Whitman page
- Reference check: the Whitman page for the second edition lists Chapter 21 "Introduction to Metamorphism" and Chapter 25 "Metamorphic Facies and Mafic Rocks" (the book title of ch. 25 is "Metamorphic Facies and Metamorphosed Mafic Rocks"), so the chapter numbers in the code are right. Chapter 21 covers metamorphic field gradients and ch. 25 the facies; neither chapter number tells which figure holds each boundary.

### Yardley1989
- Reference: Yardley, B. W. D. (1989). An Introduction to Metamorphic Petrology. Longman Earth Science Series. Longman Scientific & Technical, Harlow. ISBN 0-582-30096-7 (Open Library).
- DOI or URL: no DOI.
- Cited as: "Yardley 1989, An Introduction to Metamorphic Petrology, ch. 3"
- Ledger ids: RCK-039 to RCK-044, RCK-052
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched (bibliographic record only)
- Reference check: title, author, year, publisher exist. Chapter 3 content not checked (a 2020 second edition by Yardley & Warren also exists, so a "ch. 3" citation depends on the edition).

### Warren2016
- Reference: Warren, J. K. (2016). Evaporites: A Geological Compendium, 2nd ed. Springer, Cham. ISBN 978-3-319-13511-3 (Open Library lists the Springer 2nd-edition ISBN 9783319135113; subtitle and year from my recall of the edition, not confirmed by the fetch).
- DOI or URL: no DOI fetched.
- Cited as: "Warren 2016, Evaporites, 2nd ed., ch. 7"
- Ledger ids: RCK-045
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched (bibliographic record only)
- Reference check: author, title and edition exist; chapter 7 content not checked.

### Cruden1998
- Reference: Cruden, A. R. (1998). On the emplacement of tabular granites. Journal of the Geological Society 155(5):853-862.
- DOI or URL: 10.1144/gsjgs.155.5.0853
- Cited as: "Cruden 1998, J. Geol. Soc. 155:853-862" (tabular-pluton scaling T = 0.6 L^0.6)
- Ledger ids: RCK-057, RCK-061
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: authors, year, volume and pages are right. The abstract contains no thickness-length power law and no 0.6 coefficient or exponent; it gives widths of 10-100 km and thicknesses of 3 km or less (see spot check RCK-061). Whether the equation appears in the full text is not known. A thickness-length power law is the topic of McCaffrey & Petford (1997), "Are granitic intrusions scale invariant?", J. Geol. Soc. 154:1-4 (title and venue from my recall; an OpenAlex search for it returned nothing relevant); that is a lead for stage 2, not a verified fact.

### Bateman1979
- Reference: Bateman, P. C. & Chappell, B. W. (1979). Crystallization, fractionation, and solidification of the Tuolumne Intrusive Series, Yosemite National Park, California. Geological Society of America Bulletin 90(5):465-482.
- DOI or URL: 10.1130/0016-7606(1979)90<465:CFASOT>2.0.CO;2
- Cited as: "Bateman & Chappell 1979" (Tuolumne, about 1,100 km2, about 45 x 25 km)
- Ledger ids: RCK-060
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: authors, year, journal, volume and pages match. The abstract does not state an area or dimensions; the code's 1,100 km2 and 45 x 25 km are unconfirmed.

### Buddington1959
- Reference: Buddington, A. F. (1959). Granite emplacement with special reference to North America. Geological Society of America Bulletin 70(6):671-747 (OpenAlex shows only the first page, 671; the end page 747 is from my recall).
- DOI or URL: 10.1130/0016-7606(1959)70[671:GEWSRT]2.0.CO;2
- Cited as: "Buddington 1959" (epizonal to mesozonal emplacement)
- Ledger ids: RCK-063
- Access: paywalled (no abstract)
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: the paper exists and is the classic granite-emplacement paper; code cites no page. Whether it gives a 2-6 km roof depth is not checked.

### Hudec2007
- Reference: Hudec, M. R. & Jackson, M. P. A. (2007). Terra infirma: Understanding salt tectonics. Earth-Science Reviews 82(1-2):1-28.
- DOI or URL: 10.1016/j.earscirev.2007.01.001
- Cited as: "Hudec & Jackson 2007, Earth-Sci. Rev. 82:1-28"
- Ledger ids: STR-026, STR-027
- Access: paywalled (no abstract in OpenAlex)
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: authors, year, volume, pages match. It is a review of salt tectonics; the specific 100 m / 1,000 m thresholds are not in anything fetched.

### Doust1990
- Reference: Doust, H. & Omatsola, E. (1990). Niger Delta. In: Edwards, J. D. & Santogrossi, P. A. (eds), Divergent/Passive Margin Basins, AAPG Memoir 48, pp. 201-238 (editors and pages from my recall; not confirmed).
- DOI or URL: none found (an OpenAlex filter for 1990 "Niger Delta" returned only a single-author Doust paper, Geol. Soc. London Spec. Publ. 50:365, DOI 10.1144/GSL.SP.1990.050.01.21, which is a different work).
- Cited as: "Doust & Omatsola 1990, AAPG Memoir 48"
- Ledger ids: STR-028, STR-029
- Access: not found online
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: unverified. Doust & Omatsola 1990 as a book chapter was not located in the databases reached; the 1,500 m sediment threshold is unsourced.

### Mouthereau2007
- Reference: Mouthereau, F., Tensi, J., Bellahsen, N., Lacombe, O., De Boisgrollier, T. & Kargar, S. (2007). Tertiary sequence of deformation in a thin-skinned/thick-skinned collision belt: The Zagros Folded Belt (Fars, Iran). Tectonics 26(5):TC5006.
- DOI or URL: 10.1029/2007TC002098
- Cited as: "Mouthereau, Tensi, Bellahsen, Lacombe, De Boisgrollier & Kargar 2007, Tectonics 26:TC5006"
- Ledger ids: DEF-001, DEF-005
- Access: paywalled (abstract only)
- Read so far (by the project): not stated (the quoted phrase is in quotation marks in the comment)
- Read in this pass: abstract fetched (OpenAlex, full abstract)
- Reference check: author list, year, journal, volume and issue match; "TC5006" article number is not shown in the OpenAlex record (not verified). The abstract contains both numbers the code attributes to it (see spot checks DEF-001, DEF-005).

### Biot1961
- Reference: Biot, M. A. (1961). Theory of folding of stratified viscoelastic media and its implications in tectonics and orogenesis. Geological Society of America Bulletin 72(11):1595-1620.
- DOI or URL: 10.1130/0016-7606(1961)72[1595:TOFOSV]2.0.CO;2
- Cited as: "Biot 1961, Geol. Soc. Am. Bull. 72:1595-1620"
- Ledger ids: DEF-002
- Access: paywalled (no abstract)
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: authors, year, volume, issue, pages match.

### Sherwin1968
- Reference: Sherwin, J.-A. & Chapple, W. M. (1968). Wavelengths of single layer folds: a comparison between theory and observation. American Journal of Science 266(3):167-179.
- DOI or URL: 10.2475/ajs.266.3.167
- Cited as: "Sherwin & Chapple 1968, Am. J. Sci. 266:167-179"
- Ledger ids: DEF-002
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: authors, year, volume, pages match. The paper compares Biot's theory with 818 fold wavelengths from six sites (quartz veins in slate and phyllite), so "tested on natural folds" is correct; it found the viscosity ratios needed were low and Biot's theory could not be applied directly.

### Blanc2003
- Reference: Blanc, E. J.-P., Allen, M. B., Inger, S. & Hassani, H. (2003). Structural styles in the Zagros Simple Folded Zone, Iran. Journal of the Geological Society 160(3):401-412.
- DOI or URL: 10.1144/0016-764902-110
- Cited as: "Blanc, Allen, Inger & Hassani 2003, J. Geol. Soc. 160:401-412"
- Ledger ids: DEF-004
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: authors, year, volume, issue, pages match. The abstract is about a balanced cross-section with about 49 km of shortening over 5 Myr and several decollement horizons; it gives no fold length-to-wavelength ratio.

### McQuarrie2004
- Reference: McQuarrie, N. (2004). Crustal scale geometry of the Zagros fold-thrust belt, Iran. Journal of Structural Geology 26(3):519-535.
- DOI or URL: 10.1016/j.jsg.2003.08.009. OpenAlex reports a submitted version at the Caltech Authors repository (not fetched).
- Cited as: "McQuarrie 2004, J. Struct. Geol. 26:519-535" (Zagros shortened about 20 %)
- Ledger ids: DEF-005
- Access: open copy (author's submitted version at Caltech Authors, per OpenAlex; URL not obtained)
- Read so far (by the project): not stated
- Read in this pass: metadata only (OpenAlex dates it 15 Oct 2003 online; volume 26 is 2004)
- Reference check: title, journal, volume, pages match. The title the code gives is only "McQuarrie 2004" (no title). The "about 20 %" figure is not confirmed.

### Gawthorpe2000
- Reference: Gawthorpe, R. L. & Leeder, M. R. (2000). Tectono-sedimentary evolution of active extensional basins. Basin Research 12(3-4):195-218.
- DOI or URL: 10.1046/j.1365-2117.2000.00121.x
- Cited as: "Gawthorpe & Leeder 2000, Basin Res. 12:195-218" (faults grow by linkage of segments)
- Ledger ids: DEF-013
- Access: open access? Not determined (OpenAlex shows it as a cited journal article; the licence was not read). Treat as paywalled (metadata only).
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: authors, year, volume, issue, pages match. "Segments of some tens of km" is unconfirmed.

### Cowie1992a
- Reference: Cowie, P. A. & Scholz, C. H. (1992). Displacement-length scaling relationship for faults: data synthesis and discussion. Journal of Structural Geology 14(10):1149-1156.
- DOI or URL: 10.1016/0191-8141(92)90066-6
- Cited as: "Cowie & Scholz 1992, J. Struct. Geol. 14:1149-1156" (D = gamma L)
- Ledger ids: DEF-015, DEF-016, DEF-017, DEF-018
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: the pages 1149-1156 are the "data synthesis and discussion" paper, and it is the right one for a data-compilation citation. A companion paper by the same authors (Cowie & Scholz 1992, "Physical explanation for the displacement-length relationship of faults using a post-yield fracture mechanics model", JSG 14(10):1133-1148, DOI 10.1016/0191-8141(92)90065-5) is the theoretical one; the code does not cite it. The abstract of the cited paper gives no value of gamma.

### Schlische1996
- Reference: Schlische, R. W., Young, S. S., Ackermann, R. V. & Gupta, A. (1996). Geometry and scaling relations of a population of very small rift-related normal faults. Geology 24(8):683-686.
- DOI or URL: 10.1130/0091-7613(1996)024<0683:GASROA>2.3.CO;2
- Cited as: "Schlische, Young, Ackermann & Gupta 1996, Geology 24:683-686" (gamma about 0.03 for normal faults)
- Ledger ids: DEF-015
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary); the 0.03 comes from a secondary review (Lathrop2022)
- Reference check: authors, year, volume, pages match. The abstract says displacement scales linearly with length across eight orders of magnitude and gives no number; the review below quotes the paper's best fit as D = 0.03 L^1.06.

### Lathrop2022
- Reference: Lathrop, B. A., Jackson, C. A.-L., Bell, R. E. & Rotevatn, A. (2022). Displacement/length scaling relationships for normal faults; a review, critique, and revised compilation. Frontiers in Earth Science 10:907543.
- DOI or URL: 10.3389/feart.2022.907543; open access: https://www.frontiersin.org/journals/earth-science/articles/10.3389/feart.2022.907543/full
- Cited as: not cited by the code (added by this pass as the secondary source used for DEF-015)
- Ledger ids: DEF-015 to DEF-018 (evidence only)
- Access: open access
- Read so far (by the project): not applicable
- Read in this pass: full text fetched (via WebFetch summary of the full page; passages on Schlische 1996, Cowie & Scholz 1992a and the typical gamma range)
- Reference check: not applicable.

### Kim2005
- Reference: Kim, Y.-S. & Sanderson, D. J. (2005). The relationship between displacement and length of faults: a review. Earth-Science Reviews 68(3-4):317-334.
- DOI or URL: 10.1016/j.earscirev.2004.06.003
- Cited as: "Kim & Sanderson 2005, Earth-Sci. Rev. 68:317-334" (review, order-of-magnitude spread)
- Ledger ids: DEF-015
- Access: paywalled (no abstract in OpenAlex; publisher pages returned 403)
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: authors, volume, issue, pages match. OpenAlex dates it 2004 (online); the issue is January 2005 per my recall, so "2005" is acceptable but may be cited as 2004 elsewhere. The "order-of-magnitude spread" is not confirmed from this paper.

### Anderson1951
- Reference: Anderson, E. M. (1951). The Dynamics of Faulting and Dyke Formation with Applications to Britain, 2nd ed., revised. Oliver and Boyd, Edinburgh. 206 pp.
- DOI or URL: no DOI. Open Library: https://openlibrary.org/works/OL7161171W
- Cited as: "Anderson 1951, The Dynamics of Faulting, 2nd ed."
- Ledger ids: DEF-019, DEF-020
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched (secondary sources only)
- Reference check: the code's title is shortened; the full title includes "and Dyke Formation with Applications to Britain". The edition (2nd, 1951) matches. The first edition is 1942 (Wikipedia gives 1905 for the theory's first publication).

### Weissel1989
- Reference: Weissel, J. K. & Karner, G. D. (1989). Flexural uplift of rift flanks due to mechanical unloading of the lithosphere during extension. Journal of Geophysical Research 94(B10):13919-13950.
- DOI or URL: 10.1029/JB094iB10p13919
- Cited as: "Weissel & Karner 1989, J. Geophys. Res. 94:13919-13950"
- Ledger ids: DEF-059
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: authors, year, volume, pages match.

### Jessell1981
- Reference: Jessell, M. W. (1981). "Noddy": an interactive map creation package. Unpublished MSc thesis, Imperial College of Science and Technology, University of London (Royal School of Mines, Department of Geology), September 1981.
- DOI or URL: https://tectonique.net/noddy/jessell_msc_1981.pdf (open copy; listed in a search result, not fetched or read)
- Cited as: not cited by the code as a paper ("Imperial College 1981-82" in NOTICE); the Noddy author's own page names this as the foundational citation.
- Ledger ids: DEF-063 (origin of the ported code)
- Access: open copy at https://tectonique.net/noddy/jessell_msc_1981.pdf
- Read so far (by the project): not stated
- Read in this pass: search result snippet only
- Reference check: tectonique.net states "For publications using Noddy, the foundational reference is: Jessell, M., 1981. NODDY - An interactive map creation package".

### Jessell1996
- Reference: Jessell, M. W. & Valenta, R. K. (1996). Structural geophysics: integrated structural and geophysical modelling. In: De Paor, D. G. (ed.), Structural Geology and Personal Computers, Computer Methods in the Geosciences 15. Pergamon, Oxford, pp. 303-324.
- DOI or URL: no DOI found. Records: https://research-repository.uwa.edu.au/en/publications/structural-geophysics-integrated-structural-and-geophysical-model/ ; https://www.academia.edu/20046786/Structural_Geophysics (copy listed in a search, not fetched)
- Cited as: "Jessell & Valenta 1996, Structural geophysics ..., in De Paor (ed.), Structural Geology and Personal Computers, Pergamon, 303-324"
- Ledger ids: DEF-010, DEF-038, DEF-063
- Access: open copy (academia.edu / ResearchGate copy listed in search results; not fetched)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only
- Reference check: a search snippet gives "Computer Methods in the Geosciences, volume 15, pages 303-324, 1996", consistent with the code. Whether the fold sine table (629 values) or fault geometry is described in this chapter is not checked.

### Jessell2022
- Reference: Jessell, M., Guo, J., Li, Y., Lindsay, M., Scalzo, R., Giraud, J., Pirot, G., Cripps, E. & Ogarko, V. (2022). Into the Noddyverse: a massive data store of 3D geological models for machine learning and inversion applications. Earth System Science Data 14(1):381-392.
- DOI or URL: 10.5194/essd-14-381-2022; open access: https://essd.copernicus.org/articles/14/381/2022/
- Cited as: "Jessell et al. 2022, Earth Syst. Sci. Data 14:381-392"
- Ledger ids: DEF-063, DEF-073
- Access: open access
- Read so far (by the project): not stated
- Read in this pass: full text fetched (via WebFetch of the article page; passages on timing and code licence)
- Reference check: year, journal, volume, pages match. Authors and title now supplied in full. The code's "Jessell et al. 2022" is correct.

### Wellmann2016
- Reference: Wellmann, J. F., Thiele, S. T., Lindsay, M. D. & Jessell, M. W. (2016). pynoddy 1.0: an experimental platform for automated 3-D kinematic and potential field modelling. Geoscientific Model Development 9(3):1019-1035.
- DOI or URL: 10.5194/gmd-9-1019-2016; open access: https://gmd.copernicus.org/articles/9/1019/2016/
- Cited as: "Wellmann et al. 2016, Geosci. Model Dev. 9:1019-1035"
- Ledger ids: DEF-063, DEF-073
- Access: open access
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, volume, pages match.

### Noddyverse (software)
- Reference: Loop3D / Jessell, M. et al. noddyverse: C code to generate random Noddy models (the modified Noddy source). GitHub repository, commit 6a21b2374516f8506c19cabcbe3ead00be9cc45d (the commit hash was not looked up; only the repository head was fetched). Archived release with DOI 10.5281/zenodo.4589883 (DOI taken from the ESSD article).
- DOI or URL: https://github.com/Loop3D/noddyverse ; https://doi.org/10.5281/zenodo.4589883
- Cited as: "Loop3D/noddyverse ... MIT License, Copyright (c) 2021 Loop3D"
- Ledger ids: DEF-010, DEF-038, DEF-063 to DEF-073
- Access: open access
- Read so far (by the project): licence "checked in its LICENSE" (STATUS.md)
- Read in this pass: full text fetched (LICENSE file, repo front page, src/ listing, unEvents.c first 40 lines)
- Reference check: licence matches what the code says, see spot check DEF-063. The LICENSE title line is "MIT License", copyright line "Copyright (c) 2021 Loop3D". The fetched unEvents.c header has no copyright or licence comment. The repository carries a src/Licence.h that is a registration-ID stub (not a licence text). Other published releases of the same program carry different licences, see Noddy licence history under spot check DEF-063.

### NoddyOriginal (software, other releases)
- Reference: Noddy, Jessell, M. W. (Imperial College 1981-82; Monash University 1994; University of Western Australia). Distributions: tectonique.net Windows release and GitHub; SourceForge project "Noddy"; pynoddy (Wellmann et al.).
- DOI or URL: https://tectonique.net/noddy/ ; https://sourceforge.net/projects/noddy/ ; https://github.com/flohorovicic/pynoddy ; https://github.com/Loop3D/noddy_qt
- Cited as: not cited separately; relevant to the licence the port inherits.
- Ledger ids: DEF-063
- Access: open access
- Read so far (by the project): MIT licence checked for noddyverse only
- Read in this pass: full text fetched (the pages named)
- Reference check: see spot check DEF-063.

### tectonics.js (software)
- Reference: davidson16807. tectonics.js: "a 3d planet simulator in your web browser". GitHub.
- DOI or URL: https://github.com/davidson16807/tectonics.js
- Cited as: "tectonics.js lithifies sediment under 2.2 MPa of overburden, about 150 m (`plates::pools`)"
- Ledger ids: RCK-022, RCK-016
- Access: open access (licence shown as CC-BY-4.0)
- Read so far (by the project): not stated
- Read in this pass: full text fetched (repository front page only)
- Reference check: the repository exists with that name and owner; the front page says nothing about lithification or overburden, so the 2.2 MPa figure was not located (it would be in the source code, not read here).

### Haederle2002
- Reference: Haederle, M. & Atherton, M. P. (2002). Shape and intrusion style of the Coastal Batholith, Peru. Tectonophysics 345(1-4):17-28.
- DOI or URL: 10.1016/S0040-1951(01)00204-9
- Cited as: not cited by the code (the code gives no reference for the Coastal Batholith); added by this pass as the reference the Wikipedia article on the batholith uses.
- Ledger ids: RCK-056 (evidence only)
- Access: paywalled (no abstract in OpenAlex)
- Read so far (by the project): not applicable
- Read in this pass: metadata only
- Reference check: not applicable.

### Mukasa1986
- Reference: Mukasa, S. B. (1986). Zircon U-Pb ages of super-units in the Coastal batholith, Peru: implications for magmatic and tectonic processes. Geological Society of America Bulletin 97(2):241-254.
- DOI or URL: 10.1130/0016-7606(1986)97<241:ZUAOSI>2.0.CO;2
- Cited as: not cited by the code; added as evidence for RCK-056.
- Ledger ids: RCK-056 (evidence only)
- Access: paywalled (abstract only)
- Read so far (by the project): not applicable
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: not applicable.

### Alavi2007
- Reference: Alavi, M. (2007). Structures of the Zagros fold-thrust belt in Iran. American Journal of Science 307(9):1064-1095.
- DOI or URL: 10.2475/09.2007.02
- Cited as: not cited by the code; added as evidence for the belt width (STR-030, DEF-027).
- Ledger ids: STR-030, DEF-027 (evidence only)
- Access: paywalled (abstract only)
- Read so far (by the project): not applicable
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: not applicable.

### Pollack1993
- Reference: Pollack, H. N., Hurter, S. J. & Johnson, J. R. (1993). Heat flow from the Earth's interior: analysis of the global data set. Reviews of Geophysics 31(3):267-280.
- DOI or URL: 10.1029/93RG01249
- Cited as: not cited by the code; added as an Earth reference for RCK-028 to RCK-030.
- Ledger ids: RCK-028, RCK-029, RCK-030 (evidence only)
- Access: paywalled (abstract only)
- Read so far (by the project): not applicable
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: not applicable.

### Not bibliographic
Minecraft mod and game material: MIC rock-tag README (outside the repo); FORMAT.md; plan docs (docs/deeptime-implementation-plan.md, STATUS.md, decisions log, build log, research notes); Ben's picks and decisions (STR-002/003 design rows, "Ben's pick 2026-09-27", diamictite, stone and deepslate stand-ins). Unsourced Earth figures (no reference in the code): Zagros "about 250 km", Subandean "100-200 km", Coastal Batholith pluton counts, magma chamber depths (Kilauea, St Helens, Vesuvius), lava lakes "eight of some 1,400", stock "100 km2", CCD "4.5-5 km". Cited-but-unlisted "Buchan"-type sources: none. Athy 1930 and Sclater & Christie 1980 are not cited anywhere in this subsystem (no porosity-depth law is coded; sediment lithification is the single 150 m / 2.2 MPa rule in RCK-022).

## Deposit models (DEP)

### Singer2008a
- Reference: Singer, D.A., Berger, V.I., and Moring, B.C., 2008, Porphyry copper deposits of the world: database and grade and tonnage models, 2008 (version 1.0): U.S. Geological Survey Open-File Report 2008-1155, 45 p. plus data folder.
- DOI or URL: 10.3133/ofr20081155; https://pubs.usgs.gov/of/2008/1155/ ; PDF https://pubs.usgs.gov/of/2008/1155/of2008-1155.pdf ; data file https://pubs.usgs.gov/of/2008/1155/data/PorCuTX2008.txt
- Cited as: "Singer, Berger & Moring 2008, USGS OFR 2008-1155", "OFR 2008-1155 Table 1 General Cu", "PorCuTX2008.txt (690 records)".
- Ledger ids: DEP-001 to 009, 013, 017, 029 to 034, 158 (companion data file: DEP-003 to 005, 009, 029).
- Access: open copy (agency report) at https://pubs.usgs.gov/of/2008/1155/of2008-1155.pdf
- Read so far (by the project): in full (table, figure and PDF page of each number recorded in data file); figure intercepts read from rendered pages; data file counted.
- Read in this pass: full text fetched (whole 45 p. text layer, including Table 1, the text labels of Figs. 1-26, rules, correlations); companion data file fetched and recounted (see spot check S3).
- Reference check: matches (authors, year, series, title). The PDF page numbers in the ledger run one lower than the order of text streams I extracted (Table 1 is on the page after the correlation paragraph); that is a page-count convention, not a content problem. Note the data file I downloaded has 427 records with a tonnage, the report says 422.

### Singer2005
- Reference: Singer, D.A., Berger, V.I., Menzie, W.D., and Berger, B.R., 2005, Porphyry copper deposit density: Economic Geology 100(3):491-514.
- DOI or URL: 10.2113/gsecongeo.100.3.491 ; USGS record https://pubs.usgs.gov/publication/70197200 (carries the full abstract)
- Cited as: "Singer, Berger, Menzie & Berger 2005", "Singer et al. 2005", "Economic Geology 100(3):491-514".
- Ledger ids: DEP-011, 013, 034, 147, 153, 166.
- Access: paywalled (abstract only); full abstract open at the USGS record.
- Read so far (by the project): abstract only.
- Read in this pass: abstract fetched (full abstract from the USGS Publications Warehouse API). Full text not obtainable (GeoScienceWorld 403).
- Reference check: matches. Crossref lists the issue date as 2005-05-01 (issue 3 is May 2005). Important content point: the abstract defines the densities as deposits AND prospects (286) per 100,000 km2 of EXPOSED permissive rock in 19 control areas, and says density is inversely related to exposed permissive area (regression given for estimating the number in a tract). The code's "17.8 mean" is not in the paper (see S10).

### Singer2008b
- Reference: Singer, D.A., 2008, Mineral deposit densities for estimating mineral resources: Mathematical Geosciences 40(1):33-46.
- DOI or URL: 10.1007/s11004-007-9127-3 ; https://pubs.usgs.gov/publication/70197201
- Cited as: not cited by the project (found in this pass; it is the follow-up to Singer2005 and contains the area-density regression).
- Ledger ids: none (relevant to DEP-011, 147, 153).
- Access: paywalled (abstract only); abstract open at the USGS record.
- Read so far (by the project): not stated (not cited).
- Read in this pass: abstract fetched; a search-result snippet gave "deposit densities of 0.14 Area^-0.61 for porphyry copper" (equation not seen in the fetched abstract; units not seen).
- Reference check: n/a.

### SingerMenzie2010
- Reference: Singer, D.A., and Menzie, W.D., 2010, Quantitative mineral resource assessments: an integrated approach: New York, Oxford University Press (chapter "Estimating the Number of Undiscovered Deposits", doi 10.1093/oso/9780195399592.003.0011).
- DOI or URL: 10.1093/oso/9780195399592.003.0011 (chapter); book ISBN 9780195399592
- Cited as: named in the stage-1 task, not in the project's sources file.
- Ledger ids: none.
- Access: book (not online)
- Read so far (by the project): not stated.
- Read in this pass: not fetched (search snippet only: the book "provides newly developed deposit density models").
- Reference check: n/a. The project's density numbers all come from Singer2005 abstract, not the book.

### John2010
- Reference: John, D.A., ed., Ayuso, R.A., Barton, M.D., Blakely, R.J., Bodnar, R.J., Dilles, J.H., Gray, F., Graybeal, F.T., Mars, J.L., McPhee, D., Seal, R.R., Taylor, R.D., and Vikre, P., 2010, Porphyry copper deposit model, chap. B of Mineral deposit models for resource assessment: U.S. Geological Survey Scientific Investigations Report 2010-5070-B, xii + 169 p.
- DOI or URL: 10.3133/sir20105070B ; https://pubs.usgs.gov/sir/2010/5070/b/ ; PDF https://pubs.usgs.gov/sir/2010/5070/b/pdf/SIR10-5070B.pdf
- Cited as: "John, D.A., et al., 2010", "SIR 2010-5070-B".
- Ledger ids: DEP-016, 018, 032, 035 to 049.
- Access: open copy (agency report) at https://pubs.usgs.gov/sir/2010/5070/b/pdf/SIR10-5070B.pdf
- Read so far (by the project): printed pages cited (pp. 4, 25, 28, 65-68, 120); text searched for density with no numeric value found.
- Read in this pass: full text fetched (text layer; searched for every number in the ledger rows; Table E1 itself not extracted, its summary sentences were).
- Reference check: the USGS catalogue lists David A. John as editor and Ayuso et al. as chapter authors; "John et al. 2010" is the usual short form but strictly John is the series editor.

### SillitoeMcKee1996
- Reference: Sillitoe, R.H., and McKee, E.H., 1996, Age of supergene oxidation and enrichment in the Chilean porphyry copper province: Economic Geology 91(1):164-179.
- DOI or URL: 10.2113/gsecongeo.91.1.164 ; USGS record 70018070
- Cited as: "Sillitoe & McKee 1996".
- Ledger ids: DEP-041, 048.
- Access: paywalled (abstract only)
- Read so far (by the project): abstract only.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches.

### OrrisBliss1986
- Reference: Orris, G.J., and Bliss, J.D., 1986, Grade and tonnage model of placer Au-PGE (model 39a), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693, p. 261-264 (with descriptive model by Yeend, W.E., p. 261).
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf (Bulletin DOI 10.3133/b1693)
- Cited as: "Orris & Bliss 1986, Bulletin 1693 model 39a, Fig. 196 (p. 263)".
- Ledger ids: DEP-050 to 055, 059.
- Access: open copy (agency report) at https://pubs.usgs.gov/bul/1693/report.pdf (scanned pages)
- Read so far (by the project): figure intercepts read at 400 dpi, text pp. 261-262.
- Read in this pass: full text fetched (text layer for pp. 261-262; Figs. 196 and 197 decoded from the scan and read).
- Reference check: matches (Bulletin text says "By Greta J. Orris and James D. Bliss", reference to Orris and Bliss 1985). Figure numbers and pages match (Fig. 196 p. 263, Fig. 197 p. 264).

### OrrisBliss1985
- Reference: Orris, G.J., and Bliss, J.D., 1985, Geologic and grade-volume data on 330 gold placer deposits: U.S. Geological Survey Open-File Report 85-213, 172 p.
- DOI or URL: 10.3133/ofr85213 ; https://pubs.usgs.gov/of/1985/0213/report.pdf
- Cited as: "Orris & Bliss 1985, OFR 85-213".
- Ledger ids: DEP-055, 061.
- Access: open copy (agency report) at https://pubs.usgs.gov/of/1985/0213/report.pdf
- Read so far (by the project): caveat 1 p. 2 cited.
- Read in this pass: not fetched (catalogue record only; the USGS catalogue says 173 p.).
- Reference check: matches Bulletin 1693's own reference list entry.

### Bliss1992
- Reference: Bliss, J.D., ed., 1992, Developments in mineral deposit modeling: U.S. Geological Survey Bulletin 2004, 168 p.
- DOI or URL: not found (series record not located through the API)
- Cited as: "Bliss 1992, Bulletin 2004".
- Ledger ids: DEP-061.
- Access: open copy (agency report), not located by me
- Read so far (by the project): p. 2 overview paragraph.
- Read in this pass: not fetched. Title and 168 p. confirmed only from the reference list of OFR 2008-1155 ("Developments in deposit modeling").
- Reference check: OFR 2008-1155 lists the title as "Developments in deposit modeling"; the project writes "Developments in mineral deposit modeling". Minor title difference; check against the Bulletin cover.

### Schmidt2007
- Reference: Schmidt, J.M., Light, T.D., Drew, L.J., Wilson, F.H., Miller, M.L., and Saltus, R.W., 2007, Undiscovered locatable mineral resources in the Bay Resource Management Plan Area, southwestern Alaska: a probabilistic assessment: U.S. Geological Survey Scientific Investigations Report 2007-5039, vi + 50 p.
- DOI or URL: 10.3133/sir20075039 ; https://pubs.usgs.gov/sir/2007/5039/pdf/sir20075039.pdf
- Cited as: "Schmidt et al. 2007, SIR 2007-5039 p. 28 and Table 1".
- Ledger ids: DEP-056, 061 (also restated in DEP-051).
- Access: open copy (agency report) at https://pubs.usgs.gov/sir/2007/5039/pdf/sir20075039.pdf
- Read so far (by the project): p. 28 and Table 1.
- Read in this pass: full text fetched (tract table row for PLACER1 read, see S29).
- Reference check: matches. The USGS catalogue gives authors as Schmidt, Light, Drew, Wilson, Miller, Saltus.

### MosierSinger1986
- Reference: Mosier, D.L., and Singer, D.A., 1986, Grade and tonnage model of Superior Fe and Algoma Fe deposits (model 34a), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693, p. 228-230.
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf
- Cited as: "Mosier & Singer 1986, Bulletin 1693 p. 228-230 (model 34a)", Figs. 172-173.
- Ledger ids: DEP-062, 063.
- Access: open copy (agency report), scanned.
- Read so far (by the project): figure intercepts read from the printed arrows at 300 dpi.
- Read in this pass: full text fetched (Figs. 172-174 decoded from the scan and read).
- Reference check: matches. The Bulletin says Algoma and Superior are combined "because they are not significantly different in tonnage or grades" (Algoma has its own descriptive model 28b, Superior 34a).

### Cannon1986
- Reference: Cannon, W.F., 1986, Descriptive model of Superior Fe (model 34a, p. 228) and of Algoma Fe (model 28b, p. 198), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693.
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf
- Cited as: "Cannon 1986".
- Ledger ids: DEP-062, 073.
- Access: open copy (agency report), scanned.
- Read so far (by the project): not stated.
- Read in this pass: full text fetched (model headings only: "Model 28b DESCRIPTIVE MODEL OF ALGOMA Fe By William F. Cannon", "Model 34a DESCRIPTIVE MODEL OF SUPERIOR Fe By William F. Cannon").
- Reference check: matches (model numbers 28b and 34a confirmed).

### Cannon1995
- Reference: Cannon, W.F., Hadley, D.G., and Horton, R.J., 1995, Superior Fe deposits (model 34a) and Algoma Fe deposits (model 28b), in du Bray, E.A., ed., Preliminary compilation of descriptive geoenvironmental mineral deposit models: U.S. Geological Survey Open-File Report 95-831, 272 p.
- DOI or URL: 10.3133/ofr95831 ; https://pubs.usgs.gov/of/1995/0831/report.pdf
- Cited as: "Cannon, Hadley & Horton 1995, OFR 95-831 pp. 209-212, 252-255".
- Ledger ids: DEP-062, 063, 073.
- Access: open copy (agency report) at https://pubs.usgs.gov/of/1995/0831/report.pdf
- Read so far (by the project): pp. 209-212, 252-255.
- Read in this pass: full text fetched (searched for deposit size statements; see S20, S25).
- Reference check: not checked author list of the two chapters (the PDF text I searched did not expose the chapter by-lines).

### Singer2009
- Reference: Singer, D.A., Berger, V.I., and Moring, B.C., 2009, Sediment-hosted zinc-lead deposits of the world: database and grade and tonnage models: U.S. Geological Survey Open-File Report 2009-1252, v + 62 p.
- DOI or URL: 10.3133/ofr20091252 ; https://pubs.usgs.gov/of/2009/1252/ ; PDF https://pubs.usgs.gov/of/2009/1252/of2009-1252_text.pdf
- Cited as: "Singer, Berger & Moring 2009, OFR 2009-1252 Table 2 (printed pp. 9-10)". The stage-1 task calls it "Mosier, Berger & Singer 2009"; the report's authors are Singer, Berger and Moring.
- Ledger ids: DEP-074 to 081, 102 to 105, 108, 114.
- Access: open copy (agency report) at https://pubs.usgs.gov/of/2009/1252/of2009-1252_text.pdf
- Read so far (by the project): Table 2 (printed pp. 9-10), text p. 8, p. 2; n_reported counted from the database.
- Read in this pass: full text fetched (Table 2, correlation paragraph, two-kilometer rule).
- Reference check: matches the project's citation; the task prompt's "Mosier, Berger & Singer" is a misattribution.

### Leach2010a
- Reference: Leach, D.L., Taylor, R.D., Fey, D.L., Diehl, S.F., and Saltus, R.W., 2010, A deposit model for Mississippi Valley-type lead-zinc ores, chap. A of Mineral deposit models for resource assessment: U.S. Geological Survey Scientific Investigations Report 2010-5070-A, viii + 43 p.
- DOI or URL: 10.3133/sir20105070A ; https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf
- Cited as: "Leach et al. 2010", "SIR 2010-5070-A (printed pp. 3-5, 8, 19, 27)".
- Ledger ids: DEP-081, 090, 091, 100 (also cited in DEP-084, 085, 086, 095).
- Access: open copy (agency report) at https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf
- Read so far (by the project): printed pp. 3-5, 8, 19, 27.
- Read in this pass: full text fetched (Table 1, vertical extent, fluid temperature and salinity, the 600 km criterion, district areas).
- Reference check: matches. "Leach et al. 2010" in the code is used for two different papers (this one and Leach2010b); the 2.02 Ga claim belongs to Leach2010b.

### MosierBriskey1986
- Reference: Mosier, D.L., and Briskey, J.A., 1986, Grade and tonnage model of southeast Missouri Pb-Zn and Appalachian Zn deposits (models 32a and 32b), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693, p. 224-226.
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf
- Cited as: "Mosier & Briskey 1986, Bulletin 1693 models 32a/32b, Figs. 168-171 (printed pp. 225-226)".
- Ledger ids: DEP-101.
- Access: open copy (agency report), scanned.
- Read so far (by the project): figure intercepts.
- Read in this pass: full text fetched (Figs. 168-171 decoded from the scan and read).
- Reference check: matches (Bulletin contents: "Grade and tonnage model of southeast Missouri Pb-Zn and Appalachian Zn deposits, by Dan L. Mosier and Joseph A. Briskey").

### Emsbo2016
- Reference: Emsbo, P., Seal, R.R., Breit, G.N., Diehl, S.F., and Shah, A.K., 2016, Sedimentary exhalative (sedex) zinc-lead-silver deposit model: U.S. Geological Survey Scientific Investigations Report 2010-5070-N, 57 p.
- DOI or URL: 10.3133/sir20105070N ; https://pubs.usgs.gov/sir/2010/5070/n/sir20105070n.pdf
- Cited as: "Emsbo et al. 2016", "SIR 2010-5070-N pp. 2, 27, 28, 36, 39".
- Ledger ids: DEP-106, 107, 110, 116, 144.
- Access: open copy (agency report) at https://pubs.usgs.gov/sir/2010/5070/n/sir20105070n.pdf
- Read so far (by the project): printed pp. 2, 27, 28, 36, 39.
- Read in this pass: full text fetched (searched for each number in the ledger rows).
- Reference check: matches (chapter N published 2016 in a 2010 series).

### MenzieMosier1986
- Reference: Menzie, W.D., and Mosier, D.L., 1986, Grade and tonnage model of sedimentary exhalative Zn-Pb (model 31a), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693, p. 212-215; descriptive model by Briskey, J.A., 1986, p. 211-212.
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf
- Cited as: "Menzie & Mosier 1986, Bulletin 1693 p. 212-215 (model 31a), Figs. 159-163".
- Ledger ids: DEP-116.
- Access: open copy (agency report), scanned.
- Read so far (by the project): figure intercepts Figs. 159-163.
- Read in this pass: full text fetched (Figs. 159-163 decoded from the scan and read).
- Reference check: matches (Fig. 159 p. 213, Figs. 160-161 p. 214, Figs. 162-163 p. 215; "by W. David Menzie and Dan L. Mosier").

### duBray1995
- Reference: du Bray, E.A., ed., 1995, Preliminary compilation of descriptive geoenvironmental mineral deposit models: U.S. Geological Survey Open-File Report 95-831, ii + 272 p.
- DOI or URL: 10.3133/ofr95831
- Cited as: appears only as the container of Cannon1995.
- Ledger ids: see Cannon1995.
- Access: open copy (agency report)
- Read so far (by the project): see Cannon1995.
- Read in this pass: full text fetched; it also restates Menzie and Mosier 1986 (sedex "1.7 (90th percentile) to 130 (10th percentile) million tons; median ... 15") and Mosier and Singer 1986 (median 170 Mt).
- Reference check: matches.

### Singer2001
- Reference: Singer, D.A., Menzie, W.D., Sutphin, D.M., Mosier, D.L., and Bliss, J.D., 2001, Mineral deposit density: an update, in Singer, D.A., ed., Contributions to global mineral resource assessment research: U.S. Geological Survey Professional Paper 1640, chap. A, p. A1-A13.
- DOI or URL: 10.3133/pp1640A ; https://pubs.usgs.gov/pp/p1640a/p1640A.pdf
- Cited as: "Singer, Menzie, Sutphin, Mosier & Bliss 2001, PP 1640-A Table 1".
- Ledger ids: DEP-065, 080, 107.
- Access: open copy (agency report) at https://pubs.usgs.gov/pp/p1640a/p1640A.pdf
- Read so far (by the project): Table 1 (13 deposit types, no Zn-Pb, no iron formation) searched.
- Read in this pass: full text fetched (abstract and contents; Table 1 body not extracted, abstract lists its densities).
- Reference check: the PDF title page reads "By Donald A. Singer, W. David Menzie, David M. Sutphin, Dan L. Mosier, and James D. Bliss", which matches the project. The USGS catalogue and Crossref list different authors for this DOI (Menzie, Sutphin, Mosier, Bliss, Schulz; editor Singer). Cataloguing discrepancy; follow the PDF.

### USGS2000
- Reference: U.S. Geological Survey National Mineral Resource Assessment Team, 2000, 1998 assessment of undiscovered deposits of gold, silver, copper, lead, and zinc in the United States: U.S. Geological Survey Circular 1178, 21 p. plus CD-ROM.
- DOI or URL: 10.3133/cir1178 ; https://pubs.usgs.gov/circ/c1178/c1178.pdf
- Cited as: "Circular 1178 p. 15: USGS assessments count deposits to 1 km"; "USGS Circular 1178 (1998 National Mineral Resource Assessment)".
- Ledger ids: DEP-107, 154, 171.
- Access: open copy (agency report) at https://pubs.usgs.gov/circ/c1178/c1178.pdf
- Read so far (by the project): not read (page cited from memory of an agent; no data file records it).
- Read in this pass: full text fetched (the 1 km rule located; see S12).
- Reference check: the content is right but the page is wrong: the rule is on printed p. 5 (and the 1 km definition in the introduction), not p. 15.

### NMRA1998data
- Reference: U.S. Geological Survey, National Mineral Resource Assessment 1998, permissive tract shapefiles and attributes (tracts with estimated numbers of undiscovered deposits at 90, 50 and 10 percent): USGS Mineral Resources Online Spatial Data.
- DOI or URL: https://mrdata.usgs.gov/nmra/ (download https://mrdata.usgs.gov/nmra/nmra.zip)
- Cited as: "mrdata.usgs.gov/nmra", "12 NMRA tracts totalling 207,508 km2".
- Ledger ids: DEP-107.
- Access: open copy (dataset)
- Read so far (by the project): research by a Sonnet sub-agent, no read statement.
- Read in this pass: dataset fetched (three sedex shapefile attribute tables parsed; see S28).
- Reference check: n/a (dataset).

### Wood1983
- Reference: Wood, G.H., Jr., Kehn, T.M., Carter, M.D., and Culbertson, W.C., 1983, Coal resource classification system of the U.S. Geological Survey: U.S. Geological Survey Circular 891, vii + 65 p.
- DOI or URL: 10.3133/cir891 ; https://pubs.usgs.gov/circ/1983/0891/report.pdf
- Cited as: "Wood et al. 1983, Circular 891, Table 2 p. 22, Table 3 p. 24".
- Ledger ids: DEP-119, 123, 125, 128, 129, 130.
- Access: open copy (agency report) at https://pubs.usgs.gov/circ/1983/0891/report.pdf
- Read so far (by the project): Table 2 p. 22, Table 3 p. 24, text pp. 25-27 (lignite row read from the rendered page).
- Read in this pass: full text fetched (Tables 2 and 3, reliability definitions).
- Reference check: matches. coal_seam.json says "Table 3 p. 14" in one note; the report's own list of tables puts Table 3 on p. 24.

### Ellis1999
- Reference: Ellis, M.S., Gunther, G.L., Ochs, A.M., Roberts, S.B., Wilde, E.M., Schuenemeyer, J.H., Power, H.C., Stricker, G.D., and Blake, D., 1999, Coal resources, Powder River Basin, chap. PN; Ellis, M.S., Flores, R.M., Ochs, A.M., Stricker, G.D., Gunther, G.L., Rossi, G.S., Bader, L.R., Schuenemeyer, J.H., and Power, H.C., 1999, Gillette coalfield, Powder River Basin: geology, coal quality, and coal resources, chap. PG; both in Fort Union Coal Assessment Team, 1999 resource assessment of selected Tertiary coal beds and zones in the northern Rocky Mountains and Great Plains region: U.S. Geological Survey Professional Paper 1625-A. Chapter PS (Fort Union, Powder River) also used.
- DOI or URL: 10.3133/pp1625A ; https://pubs.usgs.gov/pp/p1625a/Chapters/PN.pdf ; PG.pdf ; PS.pdf
- Cited as: "Ellis et al. 1999, PP 1625-A chap. PN / PG / PS".
- Ledger ids: DEP-118, 130.
- Access: open copy (agency report)
- Read so far (by the project): tables PN-1, PG-2 cited.
- Read in this pass: full text fetched for PN, PG, PS (three numbers checked, S41).
- Reference check: the by-lines come from the project's data file; the catalogue lists the volume's author as "Fort Union Coal Assessment Team and USGS Energy Resources Team". I did not see the by-lines of the PDFs.

### Nuccio1994
- Reference: Nuccio, V.F., 1994, Vitrinite reflectance data for the Paleocene Fort Union and Eocene Wind River Formations, and burial history of a drill hole located in central Wind River Basin, Wyoming: U.S. Geological Survey Open-File Report 94-220, 43 p.
- DOI or URL: 10.3133/ofr94220
- Cited as: "Nuccio 1994, OFR 94-220".
- Ledger ids: DEP-126, 144.
- Access: open copy (agency report)
- Read so far (by the project): Methods PDF p. 2 (Waples 1985 thresholds as cited there).
- Read in this pass: not fetched (catalogue record only).
- Reference check: title matches.

### Higley2005
- Reference: Higley, D.K., and Cox, D.O., 2005, Oil and gas exploration and development along the Front Range in the Denver Basin of Colorado, Nebraska, and Wyoming, chap. B of Energy resource studies, northern Front Range, Colorado: U.S. Geological Survey Professional Paper 1698 (162 p.).
- DOI or URL: 10.3133/pp1698
- Cited as: "Higley & Cox 2005, USGS PP 1698 chap. B".
- Ledger ids: DEP-136, 144.
- Access: open copy (agency report)
- Read so far (by the project): pages cited in the data file (pp. 22, 26).
- Read in this pass: not fetched (only the series record resolved; chapter B record not found by search).
- Reference check: full title taken from the project's data file; not independently confirmed.

### Spencer1988
- Reference: Spencer, C.W., and Wilson, R.J., 1988, Petroleum geology and principal exploration plays in the Uinta-Piceance-Eagle Basins Province, Utah and Colorado: U.S. Geological Survey Open-File Report 88-450-G, 35 p.
- DOI or URL: 10.3133/ofr88450G
- Cited as: "Spencer & Wilson 1988, OFR 88-450-G".
- Ledger ids: DEP-136, 144.
- Access: open copy (agency report)
- Read so far (by the project): PDF p. 10.
- Read in this pass: not fetched (catalogue record confirms title and authors).
- Reference check: matches.

### Nuccio2003
- Reference: Nuccio, V.F., and Roberts, L.N.R., 2003, Thermal maturity and oil and gas generation history of petroleum systems in the Uinta-Piceance Province, Utah and Colorado, chap. 4 of USGS Uinta-Piceance Assessment Team, Petroleum systems and geologic assessment of oil and gas in the Uinta-Piceance Province, Utah and Colorado: U.S. Geological Survey Digital Data Series DDS-69-B.
- DOI or URL: not found by search (DDS-69-B)
- Cited as: "Nuccio & Roberts 2003, DDS-69-B ch. 4".
- Ledger ids: DEP-136, 144.
- Access: open copy (agency report), not located by me
- Read so far (by the project): p. 6.
- Read in this pass: not fetched.
- Reference check: not checked (title from the project's data file).

### Dubiel2003
- Reference: Dubiel, R.F., 2003, Geology, depositional models, and oil and gas assessment of the Green River Total Petroleum System, Uinta-Piceance Province, eastern Utah and western Colorado, chap. 5 of U.S. Geological Survey Digital Data Series DDS-69-B.
- DOI or URL: not found by search
- Cited as: "Dubiel 2003, DDS-69-B ch. 5".
- Ledger ids: DEP-136, 144.
- Access: open copy (agency report), not located by me
- Read so far (by the project): p. 36.
- Read in this pass: not fetched.
- Reference check: not checked.

### Moore2003
- Reference: Moore, T.E., and Potter, C.J., 2003, Structural plays in Ellesmerian sequence and correlative strata of the National Petroleum Reserve, Alaska: U.S. Geological Survey Open-File Report 2003-253, 40 p.
- DOI or URL: 10.3133/ofr03253 ; https://pubs.usgs.gov/of/2003/0253/pdf/of03-253.pdf
- Cited as: "Moore & Potter 2003, OFR 2003-253 p. 8".
- Ledger ids: DEP-136, 142, 144.
- Access: open copy (agency report) at https://pubs.usgs.gov/of/2003/0253/pdf/of03-253.pdf
- Read so far (by the project): p. 8.
- Read in this pass: full text fetched (oil window sentence located, S46).
- Reference check: matches.

### Tuttle2000
- Reference: Tuttle, M.L., Brownfield, M.E., and Charpentier, R.R. (assessment geologists), 2000, Agbada Reservoirs, assessment unit 71920101 (Niger Delta Province 7192), assessment-unit summary and data form, in U.S. Geological Survey World Energy Assessment Team, U.S. Geological Survey World Petroleum Assessment 2000: description and results: U.S. Geological Survey Digital Data Series DDS-60 (file reg7/P7/tps/AU/au719211.pdf in regions/reg7.zip). Related chapters: Ahlbrandt et al. (AR), Charpentier and Klett (MC), Schmoker and Klett (AM, also Bulletin 2165), Klett et al. (GL), all 2000.
- DOI or URL: https://pubs.usgs.gov/dds/dds-060/ ; https://pubs.usgs.gov/dds/dds-060/regions/reg7.zip ; DDS-60 DOI not resolved by Crossref
- Cited as: "USGS World Petroleum Assessment 2000 (DDS-60), AU 71920101 data form".
- Ledger ids: DEP-131, 134, 144.
- Access: open copy (agency report) at https://pubs.usgs.gov/dds/dds-060/regions/reg7.zip
- Read so far (by the project): data form p50 printed; p90 and p10 measured off the histogram at 250 dpi.
- Read in this pass: data form fetched and text read (size and number tables, assessment geologists); histogram bar heights not readable from the text layer.
- Reference check: matches (assessment geologists "M.L. Tuttle, M.E. Brownfield and R.R. Charpentier", date 5/28/98).

### Ofr9950H
- Reference: Tuttle, M.L., Charpentier, R.R., and Brownfield, M.E., 1999 (catalogue form of the names), The Niger Delta petroleum system: Niger Delta Province, Nigeria, Cameroon, and Equatorial Guinea, Africa: U.S. Geological Survey Open-File Report 99-50-H, 65 p.
- DOI or URL: 10.3133/ofr9950H
- Cited as: not cited by the project (found as the descriptive report for the Niger Delta unit).
- Ledger ids: none.
- Access: open copy (agency report)
- Read so far (by the project): not stated.
- Read in this pass: catalogue record only.
- Reference check: n/a.

### CoalMisc
- Reference (grouped; "research by a Sonnet sub-agent" in the project): Averitt, P., 1975, Coal resources of the United States, January 1, 1974: U.S. Geological Survey Bulletin 1412, 131 p. (doi 10.3133/b1412); Trippi, M.H., and Tewalt, S.J., 2011, Geographic information system (GIS) representation of coal-bearing areas in India and Bangladesh: U.S. Geological Survey Open-File Report 2011-1296, 23 p. (doi 10.3133/ofr20111296); Hatch, J.R., and Affolter, R.H., 2002, Resource assessment of the Springfield, Herrin, Danville, and Baker coals in the Illinois Basin: U.S. Geological Survey Professional Paper 1625-D (doi 10.3133/pp1625d); Luppens, J.A., Scott, D.C., Haacke, J., Osmonson, L.M., and Pierce, P.E., 2015, Coal geology and assessment of coal resources and reserves in the Powder River Basin, Wyoming and Montana: U.S. Geological Survey Professional Paper 1809 (doi 10.3133/pp1809).
- DOI or URL: as given.
- Cited as: "Averitt 1975, USGS Bulletin 1412", "USGS OFR 2011-1296 Table 1 (Indian coalfields, CFRI 1993)", "USGS PP 1625-D", "PP 1809".
- Ledger ids: DEP-117, 118.
- Access: open copy (agency reports)
- Read so far (by the project): Sonnet sub-agent research; "several are the research's own arithmetic".
- Read in this pass: not fetched (catalogue records only). Not identified: "USGS PP 1708-D1" (PP 1708 is the Appalachian basin volume; the D1 chapter was not resolved), IEA CCC/32, Bioregional Assessments Programme Sydney Basin context statement, SAIMM J. 105(2) p. 95 and Table I (South African coalfields), NMET Jharia proposal: Reference: UNIDENTIFIED for these five (the WebSearch budget was spent).
- Reference check: the four identified reports exist with the titles above. Whether they say what the notes attribute to them was not checked.

#### Journals and books

### Waples1980
- Reference: Waples, D.W., 1980, Time and temperature in petroleum formation: application of Lopatin's method to petroleum exploration: AAPG Bulletin 64(6):916-926.
- DOI or URL: https://www.osti.gov/biblio/6845191 (abstract)
- Cited as: "Waples 1980, AAPG Bulletin 64:916".
- Ledger ids: DEP-136, 144.
- Access: paywalled (abstract only); abstract open at OSTI.
- Read so far (by the project): abstract via OSTI 6845191.
- Read in this pass: abstract fetched (OSTI API record, full abstract).
- Reference check: matches.

### Newell1997
- Reference: Newell, K.D., 1997 (web version 1998), Comparison of maturation data and fluid-inclusion homogenization temperatures to simple thermal models: implications for thermal history and fluid flow in the midcontinent, part 3 "TTI modeling for localities in central Kansas", Table 1: Kansas Geological Survey, Current Research in Earth Sciences, Bulletin 240, part 2.
- DOI or URL: https://www.kgs.ku.edu/Current/1997/newell/newell3.html
- Cited as: "Newell (initials not shown) ca. 1997, KGS Bulletin 240 Table 1".
- Ledger ids: DEP-126, 144.
- Access: open copy (state survey web page)
- Read so far (by the project): web page; Waples' own table not opened.
- Read in this pass: full text fetched (Table 1 read, S45). Author is K. David Newell (part 1 of the same article gives the by-line); the project wrote "initials not shown".
- Reference check: Table 1 is attributed to Waples (1981) for TTI to Ro, so it is not Waples 1980.

### Dyman2002
- Reference: Dyman, T.S., Crovelli, R.A., Bartberger, C.E., and Takahashi, K.I., 2002, Worldwide estimates of deep natural gas resources based on the U.S. Geological Survey World Petroleum Assessment 2000: Natural Resources Research 11(3):207-218.
- DOI or URL: 10.1023/A:1019860722244 ; USGS record 70023990
- Cited as: "Dyman et al. 2002, Nat. Resour. Res. 11:207".
- Ledger ids: DEP-140.
- Access: paywalled (abstract only); abstract open at the USGS record.
- Read so far (by the project): no read statement.
- Read in this pass: abstract fetched (USGS record; deep means more than 4.5 km).
- Reference check: matches (the project writes "Nat. Resour. Res." for Natural Resources Research).

### Kesler2006
- Reference: Kesler, S.E., and Wilkinson, B.H., 2006, The role of exhumation in the temporal distribution of ore deposits: Economic Geology 101(5):919-922.
- DOI or URL: 10.2113/gsecongeo.101.5.919
- Cited as: "Kesler & Wilkinson 2006, Economic Geology 101(5):919".
- Ledger ids: DEP-166, 168, 172, 175, 191.
- Access: paywalled (abstract only)
- Read so far (by the project): OpenAlex/Semantic Scholar abstract.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches. It is a 4-page paper (919-922).

### Kesler2008
- Reference: Kesler, S.E., and Wilkinson, B.H., 2008, Earth's copper resources estimated from tectonic diffusion of porphyry copper deposits: Geology 36(3):255-258.
- DOI or URL: 10.1130/G24317A.1
- Cited as: "Kesler & Wilkinson 2008, Geology".
- Ledger ids: DEP-166, 168, 172, 177, 191.
- Access: paywalled (abstract only)
- Read so far (by the project): abstract-level.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches.

### Singer1993
- Reference: Singer, D.A., 1993, Basic concepts in three-part quantitative assessments of undiscovered mineral resources: Nonrenewable Resources 2(2):69-81.
- DOI or URL: 10.1007/BF02272804 ; https://pubs.usgs.gov/publication/70018261
- Cited as: "Singer 1993".
- Ledger ids: DEP-145, 184, 185.
- Access: paywalled (abstract only); abstract open at the USGS record.
- Read so far (by the project): abstract in the research note.
- Read in this pass: abstract fetched (USGS record, full abstract).
- Reference check: matches.

### Root1992
- Reference: Root, D.H., Menzie, W.D., and Scott, W.A., 1992, Computer Monte Carlo simulation in quantitative resource estimation: Nonrenewable Resources 1(2):125-138.
- DOI or URL: 10.1007/BF01782266
- Cited as: "Root, Menzie & Scott 1992 (Monte Carlo combination of three-part assessments)".
- Ledger ids: DEP-145.
- Access: paywalled (abstract only), not read
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: title and venue found by search; the project gave only a parenthetical description.

### Skirrow2019
- Reference: Skirrow, R.G., Murr, J., Schofield, A., Huston, D.L., van der Wielen, S., Czarnota, K., and others, 2019, Mapping iron oxide Cu-Au (IOCG) mineral potential in Australia using a knowledge-driven mineral systems-based approach: Ore Geology Reviews 113:103011.
- DOI or URL: 10.1016/j.oregeorev.2019.103011
- Cited as: "Skirrow et al. 2019".
- Ledger ids: DEP-178.
- Access: paywalled (abstract only), not read
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: probable identification (the project gives no title). It is an IOCG case study, so it is a weak authority for a general "components are multiplied" rule.

### McCuaig2010
- Reference: McCuaig, T.C., Beresford, S., and Hronsky, J., 2010, Translating the mineral systems approach into an effective exploration targeting system: Ore Geology Reviews 38(3):128-138.
- DOI or URL: 10.1016/j.oregeorev.2010.05.008
- Cited as: "McCuaig, Beresford & Hronsky 2010".
- Ledger ids: DEP-178.
- Access: paywalled (abstract only), not read
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Hronsky2008
- Reference: Hronsky, J.M.A., and Groves, D.I., 2008, Science of targeting: definition, strategies, targeting and performance measurement: Australian Journal of Earth Sciences 55(1):3-12.
- DOI or URL: 10.1080/08120090701581356
- Cited as: "Hronsky & Groves 2008".
- Ledger ids: DEP-178.
- Access: paywalled (abstract only), not read
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Agterberg1995
- Reference: Agterberg, F.P., 1995, Multifractal modeling of the sizes and grades of giant and supergiant deposits: International Geology Review 37(1):1-8.
- DOI or URL: 10.1080/00206819509465388
- Cited as: "Agterberg 1995 (lognormal with Pareto tail)".
- Ledger ids: DEP-015, 155, 162.
- Access: paywalled (abstract only), not read
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: probable identification (project gives only author and year); title concerns multifractal, not "lognormal with Pareto tail" per se.

### Singer1995
- Reference: Singer, D.A., 1995, World class base and precious metal deposits: a quantitative analysis: Economic Geology 90(1):88-104.
- DOI or URL: 10.2113/gsecongeo.90.1.88 ; USGS record 70197194
- Cited as: "Singer 1995 (world-class share)".
- Ledger ids: DEP-015, 155, 162.
- Access: paywalled (abstract only); abstract open at the USGS record.
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (USGS record).
- Reference check: matches ("between 47 and 79 percent of metal contained in the largest 10 percent of deposits").

### Robert2005
- Reference: Robert, F., Poulsen, K.H., Cassidy, K.F., and Hodgson, C.J., 2005, Gold metallogeny of the Superior and Yilgarn cratons, in One Hundredth Anniversary Volume (Economic Geology): Society of Economic Geologists, p. 1001-1033.
- DOI or URL: 10.5382/av100.30
- Cited as: "Robert et al. 2005 (orogenic-gold camps 30-50 km)".
- Ledger ids: DEP-015.
- Access: paywalled (abstract only), not read
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only; the 30-50 km figure could not be checked).
- Reference check: probable identification (project gives only author and year).

### Butterworth2016
- Reference: Butterworth, N.P., Steinberg, D., Müller, R.D., Williams, S.E., Merdith, A.S., and Hardy, S., 2016, Tectonic environments of South American porphyry copper magmatism through time revealed by spatiotemporal data mining: Tectonics 35(12):2847-2862.
- DOI or URL: 10.1002/2016TC004289
- Cited as: "Butterworth et al. 2016 (machine learning on South American subduction)".
- Ledger ids: DEP-024.
- Access: paywalled (abstract only)
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches.

### Richards2003
- Reference: Richards, J.P., 2003, Tectono-magmatic precursors for porphyry Cu-(Mo-Au) deposit formation: Economic Geology 98(8):1515-1533.
- DOI or URL: 10.2113/gsecongeo.98.8.1515
- Cited as: "Richards 2003".
- Ledger ids: DEP-023, 025, 026.
- Access: paywalled (abstract only)
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches (abstract covers oxidised, hydrous magmas and fault-controlled ascent).

### Cooke2005
- Reference: Cooke, D.R., Hollings, P., and Walshe, J.L., 2005, Giant porphyry deposits: characteristics, distribution, and tectonic controls: Economic Geology 100(5):801-818.
- DOI or URL: 10.2113/gsecongeo.100.5.801 (author copy: https://figshare.com/articles/journal_contribution/Giant_porphyry_deposits_Characteristics_distribution_and_tectonic_controls/22856222)
- Cited as: "Cooke et al. 2005".
- Ledger ids: DEP-023, 025, 026.
- Access: open copy (author copy on figshare, per OpenAlex)
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (OpenAlex); the figshare copy was not opened.
- Reference check: matches ("compressive tectonic environments, thickened continental crust").

### Bekker2010
- Reference: Bekker, A., Slack, J.F., Planavsky, N., Krapež, B., Hofmann, A., Konhauser, K.O., and Rouxel, O.J., 2010, Iron formation: the sedimentary product of a complex interplay among mantle, tectonic, oceanic, and biospheric processes: Economic Geology 105(3):467-508.
- DOI or URL: 10.2113/gsecongeo.105.3.467
- Cited as: "Bekker et al. 2010 (BIF acme 2.65-1.85 Ga)".
- Ledger ids: DEP-068.
- Access: paywalled (abstract only)
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched but cut off before the age statement (OpenAlex); the 2.65-1.85 Ga window not seen.
- Reference check: title and journal match; age window unverified.

### Bohacs1997
- Reference: Bohacs, K., and Suter, J., 1997, Sequence stratigraphic distribution of coaly rocks: fundamental controls and paralic examples: AAPG Bulletin 81(10):1612-1639.
- DOI or URL: 10.1306/3B05C3FC-172A-11D7-8645000102C1865D
- Cited as: "Bohacs & Suter 1997".
- Ledger ids: DEP-121, 122.
- Access: paywalled (abstract only)
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches ("significant volumes of terrigenous organic matter can be preserved to form coals only when and where the overall increase in accommodation approximately equals the production rate of peat").

### Nelsen2016
- Reference: Nelsen, M.P., DiMichele, W.A., Peters, S.E., and Boyce, C.K., 2016, Delayed fungal evolution did not cause the Paleozoic peak in coal production: Proceedings of the National Academy of Sciences 113(9):2442-2447.
- DOI or URL: 10.1073/pnas.1517943113
- Cited as: "Nelsen et al. 2016".
- Ledger ids: DEP-121, 122.
- Access: open copy at https://www.ncbi.nlm.nih.gov/pmc/articles/4780611 (per Semantic Scholar's open-access field; link not opened)
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: probable identification only: the project gives author and year, and this is the 2016 Nelsen et al. paper on coal production.

### Hoggard2020
- Reference: Hoggard, M.J., Czarnota, K., Richards, F.D., Huston, D.L., Jaques, A.L., and Ghelichkhan, S., 2020, Global distribution of sediment-hosted metals controlled by craton edge stability: Nature Geoscience 13(7):504-510.
- DOI or URL: 10.1038/s41561-020-0593-2
- Cited as: "Hoggard et al. 2020 (85 percent within 200 km)".
- Ledger ids: DEP-111.
- Access: open access
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (Semantic Scholar).
- Reference check: matches.

### Leach2001
- Reference: Leach, D.L., Bradley, D.C., Lewchuk, M.T., Symons, D.T.A., de Marsily, G., and Brannon, J., 2001, Mississippi Valley-type lead-zinc deposits through geological time: implications from recent age-dating research: Mineralium Deposita 36(8):711-740.
- DOI or URL: 10.1007/s001260100208
- Cited as: "Leach et al. 2001".
- Ledger ids: DEP-082 to 087, 094 to 098 (as a group).
- Access: paywalled, not read
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (metadata only).
- Reference check: probable identification (project gives author and year).

### Bradley2003
- Reference: Bradley, D.C., and Leach, D.L., 2003, Tectonic controls of Mississippi Valley-type lead-zinc mineralization in orogenic forelands: Mineralium Deposita 38(6):652-667.
- DOI or URL: 10.1007/s00126-003-0355-2
- Cited as: "Bradley & Leach 2003".
- Ledger ids: DEP-084, 095.
- Access: paywalled, not read
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Leach2005
- Reference: Leach, D.L., Sangster, D.F., Kelley, K.D., Large, R.R., Garven, G., Allen, C.R., Gutzmer, J., and Walters, S., 2005, Sediment-hosted lead-zinc deposits: a global perspective, in One Hundredth Anniversary Volume (Economic Geology): Society of Economic Geologists, p. 561-607.
- DOI or URL: 10.5382/av100.18
- Cited as: "Leach et al. 2005 (90-200 C)".
- Ledger ids: DEP-082, 094, 095.
- Access: paywalled (abstract only)
- Read so far (by the project): no read statement.
- Read in this pass: abstract fetched (OpenAlex); the 90-200 C figure is not in the abstract.
- Reference check: matches; the temperature figure remains unchecked (SIR 2010-5070-A gives 50-250, mostly 75-200 C).

### Leach2010b
- Reference: Leach, D.L., Bradley, D.C., Huston, D.L., Pisarevsky, S.A., Taylor, R.D., and Gardoll, S.J., 2010, Sediment-hosted lead-zinc deposits in Earth history: Economic Geology 105(3):593-625.
- DOI or URL: 10.2113/gsecongeo.105.3.593
- Cited as: the code and plan write "Leach et al. 2010" for the 2.02 Ga onset; the project attaches it to SIR 2010-5070-A.
- Ledger ids: DEP-086, 112, 188 (claim), not listed in the sources file as a separate source.
- Access: paywalled (abstract only)
- Read so far (by the project): not stated; probably via the research note.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: this is the paper that carries the 2.02 Ga statement (see S33). The project has conflated it with Leach2010a.

### Garven1984
- Reference: Garven, G., and Freeze, R.A., 1984, Theoretical analysis of the role of groundwater flow in the genesis of stratabound ore deposits: 1. Mathematical and numerical model, American Journal of Science 284(10):1085-1124; 2. Quantitative results, American Journal of Science 284(10):1125-1174.
- DOI or URL: 10.2475/ajs.284.10.1085 ; 10.2475/ajs.284.10.1125
- Cited as: "Garven & Freeze 1984 (base case: 1 km water-table drop over 300 km)".
- Ledger ids: DEP-083, 095, 098.
- Access: paywalled (abstract only)
- Read so far (by the project): no read statement.
- Read in this pass: abstracts of both parts fetched (OpenAlex); the base case numbers are not in either abstract.
- Reference check: the paper is two parts; the code does not say which. The base case is probably in part 2.

### Garven1985
- Reference: Garven, G., 1985, The role of regional fluid flow in the genesis of the Pine Point deposit, Western Canada sedimentary basin: Economic Geology 80(2):307-324.
- DOI or URL: 10.2113/gsecongeo.80.2.307
- Cited as: "Garven 1985".
- Ledger ids: DEP-082, 083, 087, 094, 095.
- Access: paywalled (abstract only)
- Read so far (by the project): abstract (research note).
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches.

### Bethke1990
- Reference: Bethke, C.M., and Marshak, S., 1990, Brine migrations across North America: the plate tectonics of groundwater: Annual Review of Earth and Planetary Sciences 18:287-315.
- DOI or URL: 10.1146/annurev.ea.18.050190.001443
- Cited as: "Bethke & Marshak 1990 quote".
- Ledger ids: DEP-083, 095.
- Access: paywalled, not read
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Oliver1986
- Reference: Oliver, J., 1986, Fluids expelled tectonically from orogenic belts: their role in hydrocarbon migration and other geologic phenomena: Geology 14(2):99 (end page not seen).
- DOI or URL: 10.1130/0091-7613(1986)14<99:FETFOB>2.0.CO;2
- Cited as: "Oliver 1986".
- Ledger ids: DEP-095.
- Access: paywalled, not read
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Sillitoe2005
- Reference: Sillitoe, R.H., 2005, Supergene oxidized and enriched porphyry copper and related deposits, in One Hundredth Anniversary Volume (Economic Geology): Society of Economic Geologists, p. 723-768.
- DOI or URL: 10.5382/av100.22
- Cited as: "Sillitoe 2005 (supergene windows, 0.5 Myr, 2-3x)".
- Ledger ids: DEP-041, 043.
- Access: paywalled (abstract only); abstract open via Crossref.
- Read so far (by the project): abstract in the research note; the data file says "typically 3-9 Myr" was not found in any USGS document.
- Read in this pass: abstract fetched (Crossref, full abstract).
- Reference check: matches. The abstract itself contains both "at least 0.5 m.y." and "typically minima of 3 to 9 m.y." (S13).

### Titley1995
- Reference: Titley, S.R., and Marozas, D.C., 1995, Processes and products of supergene copper enrichment, in Pierce, F.W., and Bolm, J.G., eds., Porphyry copper deposits of the American Cordillera: Arizona Geological Society Digest 20, p. 156-168.
- DOI or URL: none (reference taken from the SIR 2010-5070-B reference list)
- Cited as: "Titley & Marozas 1995" (as cited inside SIR 2010-5070-B).
- Ledger ids: DEP-040, 046.
- Access: book (not online)
- Read so far (by the project): second-hand through the SIR.
- Read in this pass: not fetched; reference string read in SIR 2010-5070-B.
- Reference check: matches.

### Clark1990
- Reference: Clark, A.H., Tosdal, R.M., Farrar, E., and Plazolles V., A., 1990, Geomorphologic environment and age of supergene enrichment of the Cuajone, Quellaveco, and Toquepala porphyry copper deposits, southeastern Peru: Economic Geology 85(7):1604-1628.
- DOI or URL: 10.2113/gsecongeo.85.7.1604
- Cited as: "Clark et al. 1990" (as cited in SIR 2010-5070-B).
- Ledger ids: DEP-040.
- Access: paywalled, not read
- Read so far (by the project): second-hand through the SIR.
- Read in this pass: not fetched; reference string read in SIR 2010-5070-B.
- Reference check: matches. It concerns southern Peru; the SIR sentence quoted for the "10 cm/year" climate is attributed to it.

### Mote2001
- Reference: Mote, T.I., Becker, T.A., Renne, P., and Brimhall, G.H., 2001, Chronology of exotic mineralization at El Salvador, Chile, by 40Ar/39Ar dating of copper wad and supergene alunite: Economic Geology 96(2):351-366.
- DOI or URL: 10.2113/gsecongeo.96.2.351
- Cited as: "Mote et al. 2001" (as cited in SIR 2010-5070-B).
- Ledger ids: DEP-048.
- Access: paywalled, not read
- Read so far (by the project): second-hand through the SIR.
- Read in this pass: not fetched; reference string read in SIR 2010-5070-B (SIR initials "T.L. Mote", Crossref "T.I. Mote").
- Reference check: matches.

### Sillitoe2010
- Reference: Sillitoe, R.H., 2010, Porphyry copper systems: Economic Geology 105(1):3-41.
- DOI or URL: 10.2113/gsecongeo.105.1.3
- Cited as: "Sillitoe 2010".
- Ledger ids: DEP-190 (also DEP-021, 188).
- Access: paywalled (abstract only)
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches (abstract says stocks "vertically elongate (>3 km)", pluton paleodepths 5 to 15 km, life spans ~100,000 to several million years).

### Seedorff2005
- Reference: Seedorff, E., Dilles, J.H., Proffett, J.M., Einaudi, M.T., Zurcher, L., Stavast, W.J.A., and others, 2005, Porphyry deposits: characteristics and origin of hypogene features, in One Hundredth Anniversary Volume (Economic Geology): Society of Economic Geologists, p. 251-298.
- DOI or URL: 10.5382/av100.10
- Cited as: "Seedorff et al. 2005".
- Ledger ids: DEP-190.
- Access: paywalled, not read
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only; the Crossref listing shows the first six authors; the full author list was not retrieved).
- Reference check: probable identification.

### Heckel1986
- Reference: Heckel, P.H., 1986, Sea-level curve for Pennsylvanian eustatic marine transgressive-regressive depositional cycles along midcontinent outcrop belt, North America: Geology 14(4):330 (end page not seen).
- DOI or URL: 10.1130/0091-7613(1986)14<330:SCFPEM>2.0.CO;2
- Cited as: "Heckel 1986 (cyclothem durations)".
- Ledger ids: DEP-123.
- Access: paywalled, not read
- Read so far (by the project): via the coal research sub-agent, no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: probable identification; the project gives only author and year.

### Rasbury1998
- Reference: Rasbury, E.T., Hanson, G.N., Meyers, W.J., Holt, W.E., Goldstein, R.H., and Saller, A.H., 1998, U-Pb dates of paleosols: constraints on late Paleozoic cycle durations and boundary ages: Geology 26(5):403 (end page not seen).
- DOI or URL: 10.1130/0091-7613(1998)026<0403:UPDOPC>2.3.CO;2
- Cited as: "Rasbury et al. 1998 (cyclothem durations)".
- Ledger ids: DEP-123.
- Access: paywalled, not read
- Read so far (by the project): via the coal research sub-agent, no read statement.
- Read in this pass: not fetched (Crossref metadata only; end page not seen).
- Reference check: probable identification.

### Blackman2021
- Reference: Blackman, D., and Vigna, S., 2021, Scrambled linear pseudorandom number generators: ACM Transactions on Mathematical Software 47(4):1-32.
- DOI or URL: 10.1145/3460772
- Cited as: "Blackman & Vigna 2021 (xoshiro256++)".
- Ledger ids: DEP-164.
- Access: open copy (arXiv preprint https://arxiv.org/pdf/1805.01407 per Semantic Scholar's open-access field; not opened)
- Read so far (by the project): "the constants of the paper".
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Marsaglia1964
- Reference: Marsaglia, G., and Bray, T.A., 1964, A convenient method for generating normal variables: SIAM Review 6(3):260-264.
- DOI or URL: 10.1137/1006063
- Cited as: "Marsaglia polar method" (the project gives no citation; this is the usual source of the polar method).
- Ledger ids: DEP-164.
- Access: paywalled, not read
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: probable identification.

### Hormann1993
- Reference: Hörmann, W., 1993, The transformed rejection method for generating Poisson random variables: Insurance: Mathematics and Economics 12(1):39-45.
- DOI or URL: 10.1016/0167-6687(93)90997-4
- Cited as: "Hormann 1993, Insurance: Mathematics and Economics 12:39".
- Ledger ids: DEP-164.
- Access: paywalled, not read
- Read so far (by the project): "the constants of the paper".
- Read in this pass: not fetched; the constants were checked against NumPy's implementation of the algorithm (S50).
- Reference check: matches.

### AbramowitzStegun1964
- Reference: Abramowitz, M., and Stegun, I.A., eds., 1964, Handbook of mathematical functions with formulas, graphs, and mathematical tables: National Bureau of Standards Applied Mathematics Series 55, formula 7.1.26.
- DOI or URL: none
- Cited as: "Abramowitz & Stegun 7.1.26 (error under 1.5e-7)".
- Ledger ids: DEP-165.
- Access: book (not online here)
- Read so far (by the project): no read statement.
- Read in this pass: not fetched.
- Reference check: the edition year (1964) is the standard first edition; not checked here.

### Acklam
- Reference: Acklam, P.J., An algorithm for computing the inverse normal cumulative distribution function (web note, undated, around 2000-2003).
- DOI or URL: http://home.online.no/~pjacklam/notes/invnorm/ (read through the Internet Archive capture of 2012 at https://web.archive.org/web/2012/http://home.online.no/~pjacklam/notes/invnorm/index.html)
- Cited as: "Acklam's rational approximation of the inverse normal CDF".
- Ledger ids: DEP-165.
- Access: open copy (web page, archived)
- Read so far (by the project): no read statement.
- Read in this pass: full text fetched (relative error and the 0.02425 split read, S50).
- Reference check: matches the algorithm; undated web note, not a publication.

### Software
- Reference: Ellefsen, K.J., 2017, User's guide for MapMark4: an R package for the probability calculations in three-part mineral resource assessments: U.S. Geological Survey Techniques and Methods, book 7, chap. C14 (doi 10.3133/tm7c14); Duval, J.S., 2012, Version 3.0 of EMINERS, Economic Mineral Resource Simulator: U.S. Geological Survey Open-File Report 2004-1344 (doi 10.3133/ofr20041344); EIS Toolkit (EUPL-1.2, offline calibration only): Reference: UNIDENTIFIED (no citation given; not searched).
- DOI or URL: as given.
- Cited as: "MapMark4 (USGS R package, CC0, method reference)", "EMINERS", "EIS Toolkit".
- Ledger ids: none (no id relies on a number from them).
- Access: open copy (agency reports)
- Read so far (by the project): compared against, not ported.
- Read in this pass: not fetched (catalogue records only).
- Reference check: the two USGS items were found by search and are the documented references for those programs; the match to what the project means is probable.

### Not bibliographic
docs/deeptime-design-decisions.md (Ben's decisions and its log entries); docs/deeptime-implementation-plan.md §8.6, §7.8; docs/research/notes/ore_genesis_modeling.md (a project note; its abstract-level reading of journals is reflected above); MIC rock-tag datapack README; STATUS.md build-log entries; Create ores and Create: Diesel Generators, Create Ore Excavation 1.6.8 (Minecraft mods); Wikipedia pages used for some coal field areas (weak, not a source of record); the Sonnet sub-agent research for coal and for SEDEX tract figures (a process, not a source); named in the earth file only: Hartley & Rice 2005, Yanites & Kesler 2015, Warren 2010, Griffin 2013, Diaz-Rodriguez 2021, Hagemann 2016 (cited "at title level only" in the research note; they do not appear in the sources file or in a ledger row, so they were not resolved).

## Deposit formation, coal, petroleum, placers (ORE)

### Waples1980
- Reference: Waples, Douglas W., 1980, Time and temperature in petroleum formation: application of Lopatin's method to petroleum exploration: AAPG Bulletin, v. 64, no. 6, p. 916-926.
- DOI or URL: 10.1306/2F9193D2-16CE-11D7-8645000102C1865D (Crossref). Abstract: https://www.osti.gov/biblio/6845191 (OSTI ID 6845191, "Am. Assoc. Pet. Geol. Bull.; (United States)", 64:6, June 1980). OpenAlex lists it as closed access.
- Cited as: "Waples 1980, AAPG Bulletin 64:916-926"; "Waples 1980, AAPG Bulletin 64:916".
- Ledger ids: ORE-158, 159, 160, 161, 162, 197.
- Access: paywalled (abstract only); abstract open at OSTI.
- Read so far (by the project): abstract only (OSTI record).
- Read in this pass: abstract fetched (OSTI, quoted verbatim in spot checks below). Full text not fetched (AAPG/Datapages returned 403).
- Reference check: matches (author, year, volume, issue, pages). The abstract contains the three oil-window thresholds, the doubling rule and the four preservation deadlines. It contains no TTI-to-Ro table and no statement of the temperature interval at which n = 0. The abstract is written "approx. 500 40 degree oil preservation deadline" with the degree sign garbled as "40/sup 0/", which the project read as API gravity; the abstract itself does not say "API".

### Newell1997
- Reference: Newell, K. David, 1997 (web version 16 March 1998), Comparison of maturation data and fluid-inclusion homogenization temperatures to simple thermal models: implications for thermal history and fluid flow in the Midcontinent: Kansas Geological Survey, Current Research in Earth Sciences, Bulletin 240, part 2. The cited page is the section "TTI Modeling for Localities in Central Kansas", with Table 1 (TTI to Ro, credited to Waples).
- DOI or URL: https://www.kgs.ku.edu/Current/1997/newell/newell3.html (article front page: .../newell/newell1.html). No DOI.
- Cited as: "Waples 1981 as tabulated by Newell, KGS Bulletin 240 part 2, Table 1"; "kgs-b240".
- Ledger ids: ORE-152, 198.
- Access: open access (state survey web page).
- Read so far (by the project): secondary source (web page, flagged non-USGS).
- Read in this pass: full text fetched (the section page: Table 1 rows and the doubling sentence; article front page for title and author). The page does not print a reference list, so what "Waples 1981" is cannot be read from it. The text says "Waples (1980, 1981)".
- Reference check: author is K. David Newell (the sources file lacked the initials). Title and the "1997" year: the URL path says 1997, web version dated 1998-03-16. The "Waples 1981" it credits is probably Waples's 1981 book (Organic Geochemistry for Exploration Geologists), but I could not confirm that from the page; treat as unidentified.

### Lopatin1971 (related; not cited by the code)
- Reference: Lopatin, N. V., 1971, Temperature and geologic time factors in coalification (in Russian): Akademiya Nauk SSSR, Izvestiya, Seriya Geologicheskaya, v. 3, p. 95-106.
- DOI or URL: none found. Cited by the AAPG Wiki article "Petroleum source rocks" (https://wiki.aapg.org/Petroleum_source_rocks).
- Cited as: not cited by the code; the code credits "Lopatin's method" via Waples.
- Ledger ids: (none; background for ORE-158)
- Access: not found (Russian journal, not online).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (citation read from the AAPG Wiki page).
- Reference check: n/a. The AAPG Wiki says Lopatin "first described maturation modeling by calculating time-temperature indices for coals, basing his calculations on conversion kinetics of vitrinite" and that Waples "brought the Lopatin method to English-reading geologists". The wiki also lists Waples 1985, Geochemistry in petroleum exploration (IHRDC, Boston, 232 p.).

### Magoon1994
- Reference: Magoon, Leslie B., and Dow, Wallace G., eds., 1994, The petroleum system: from source to trap: AAPG Memoir 60. The "critical moment / charge in order" idea is in the memoir's opening chapter (Magoon and Dow, "The petroleum system", chapter 1); chapter pages not verified in this pass.
- DOI or URL: 10.1306/M60585 (Crossref confirms title, editors, 1994, AAPG).
- Cited as: "Magoon & Dow 1994", DOI 10.1306/m60585.
- Ledger ids: ORE-167.
- Access: book (not online).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: the DOI is the whole memoir, not a chapter; cite the chapter (DOI 10.1306/M60585C1 would be its form, unverified) if the claim is about the critical moment.

### Hudec2007
- Reference: Hudec, Michael R., and Jackson, Martin P. A., 2007, Terra infirma: understanding salt tectonics: Earth-Science Reviews, v. 82, no. 1-2, p. 1-28.
- DOI or URL: 10.1016/j.earscirev.2007.01.001 (OpenAlex).
- Cited as: "Hudec & Jackson 2007, Earth-Sci. Rev. 82:1-28".
- Ledger ids: ORE-181.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: matches (authors, year, journal, volume, pages). Title is "Terra infirma: understanding salt tectonics".

### Doust1990
- Reference: Doust, Harry, and Omatsola, Ebi, 1990, Niger Delta, in Edwards, J. D., and Santogrossi, P. A., eds., Divergent/passive margin basins: AAPG Memoir 48. Page range not confirmed in this pass.
- DOI or URL: 10.1306/M48508C4 (Crossref; Crossref dates the chapter 1989 online, the memoir is conventionally 1990).
- Cited as: "Doust & Omatsola 1990, AAPG Memoir 48".
- Ledger ids: ORE-183.
- Access: book (not online).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches (authors, memoir number). Chapter title "Niger Delta".

### Dyman2002
- Reference: Dyman, Thaddeus S., Crovelli, Robert A., Bartberger, Charles E., and Takahashi, Kenneth I., 2002, Worldwide estimates of deep natural gas resources based on the U.S. Geological Survey World Petroleum Assessment 2000: Natural Resources Research, v. 11, no. 3, p. 207-218.
- DOI or URL: 10.1023/A:1019860722244 (OpenAlex).
- Cited as: "Dyman et al. 2002, Nat. Resour. Res. 11:207".
- Ledger ids: ORE-192.
- Access: paywalled (abstract only); Springer page redirected to a login, abstract not retrieved.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: matches (year, journal, volume, first page). I did not confirm that the paper defines "deep" as below 4.5 km or that USGS assessments have no depth cut-off.

### Higley2005
- Reference: Higley, Debra K., and Cox, Dave O., 2005, Oil and gas exploration and development along the Front Range in the Denver Basin of Colorado, Nebraska, and Wyoming, chap. B in Fishman, Neil S., ed., Energy resource studies, northern Front Range, Colorado: U.S. Geological Survey Professional Paper 1698.
- DOI or URL: https://doi.org/10.3133/pp1698 (volume; USGS page https://pubs.usgs.gov/publication/pp1698 lists Fishman 2005, 162 p.). Chapter B PDF not located.
- Cited as: "USGS PP 1698 p. 22"; key pp1698-b.
- Ledger ids: ORE-195, 203.
- Access: open access (agency report).
- Read so far (by the project): in full (PDF cited by page).
- Read in this pass: not fetched (the USGS page gave the volume citation only; chapter title and authors taken from the project's sources file, not confirmed).
- Reference check: volume and editor confirmed; chapter B authorship and title not confirmed.

### Spencer1988
- Reference: Spencer, C. W., and Wilson, R. J., 1988, Petroleum geology and principal exploration plays in the Uinta-Piceance-Eagle Basins Province, Utah and Colorado: U.S. Geological Survey Open-File Report 88-450-G, ii, 35 p.
- DOI or URL: https://pubs.usgs.gov/publication/ofr88450G
- Cited as: "OFR 88-450-G".
- Ledger ids: ORE-196, 203.
- Access: open access (agency report).
- Read so far (by the project): in full (PDF pages cited).
- Read in this pass: not fetched (citation page only; no abstract on it).
- Reference check: matches.

### Nuccio1994
- Reference: Nuccio, V. F., 1994, Vitrinite reflectance data for the Paleocene Fort Union and Eocene Wind River formations, and burial history of a drill hole located in central Wind River basin, Wyoming: U.S. Geological Survey Open-File Report 94-220, 43 p.
- DOI or URL: https://pubs.usgs.gov/publication/ofr94220 ; PDF https://pubs.usgs.gov/of/1994/0220/report.pdf (binary to WebFetch).
- Cited as: "OFR 94-220".
- Ledger ids: ORE-199.
- Access: open access (agency report).
- Read so far (by the project): in full (PDF p. 2 and Fig. 2 cited).
- Read in this pass: not fetched (PDF unreadable by WebFetch; citation page only).
- Reference check: matches (the USGS page prints the author as "Nuccio, V.").

### Nuccio2003
- Reference: Nuccio, V. F., and Roberts, L. N. R., 2003, Thermal maturity and oil and gas generation history of petroleum systems in the Uinta-Piceance Province, chap. 4 in USGS Uinta-Piceance Assessment Team, Petroleum systems and geologic assessment of oil and gas in the Uinta-Piceance Province, Utah and Colorado: U.S. Geological Survey Data Series DDS-69-B.
- DOI or URL: https://pubs.usgs.gov/publication/ds69B (volume; lists Nuccio and Roberts among the contributing authors, 28 chapters).
- Cited as: "DDS-69-B chap. 4"; key dds69b-ch4.
- Ledger ids: ORE-200.
- Access: open access (agency report).
- Read so far (by the project): in full (printed p. 6 cited).
- Read in this pass: not fetched (volume page only; chapter title and chapter-author pairing not confirmed).
- Reference check: volume title and series confirmed; chapter pairing unconfirmed.

### Moore2003
- Reference: Moore, T. E., and Potter, C. J., 2003, Structural plays in Ellesmerian sequence and correlative strata of the National Petroleum Reserve, Alaska: U.S. Geological Survey Open-File Report 2003-253, 40 p.
- DOI or URL: https://pubs.usgs.gov/publication/ofr03253
- Cited as: "OFR 2003-253".
- Ledger ids: ORE-201.
- Access: open access (agency report).
- Read so far (by the project): in full (printed p. 8 cited).
- Read in this pass: abstract fetched (USGS page). The abstract says hydrocarbon generation and migration "occurred primarily by sedimentary burial principally between 100-90 Ma"; it does not state an 80-120 degC oil window.
- Reference check: matches.

### Emsbo2016
- Reference: Emsbo, Poul, Seal, R. R., Breit, G. N., Diehl, S. F., and Shah, A. K., 2016, Sedimentary exhalative (sedex) zinc-lead-silver deposit model: U.S. Geological Survey Scientific Investigations Report 2010-5070-N, 57 p.
- DOI or URL: https://pubs.usgs.gov/publication/sir20105070N
- Cited as: "SIR 2010-5070-N p. 2"; key sir2010-5070n.
- Ledger ids: ORE-026, 202.
- Access: open access (agency report).
- Read so far (by the project): in full (printed pp. 2, 28 cited).
- Read in this pass: abstract fetched (USGS page; no temperatures or gradients in it).
- Reference check: matches exactly (authors, year, title, report number, 57 p.).

### Tuttle2000
- Reference: Tuttle, M. L., Brownfield, M. E., and Charpentier, R. R., 2000, Agbada Reservoirs, Assessment Unit 71920101 (Niger Delta Province 7192): data form in U.S. Geological Survey World Petroleum Assessment 2000, DDS-60 (region 7 files).
- DOI or URL: none; DDS-60 index https://pubs.usgs.gov/dds/dds-060/ (data form inside regions/reg7.zip, as the project describes).
- Cited as: "Tuttle, Brownfield & Charpentier 2000, DDS-60 AU 71920101"; key dds60-au71920101.
- Ledger ids: ORE-185, 186, 205.
- Access: open access (agency report; zip of PDFs).
- Read so far (by the project): in full (data form read, histogram digitised by the project).
- Read in this pass: not fetched. The authors on the AU form could not be confirmed (the DDS-60 index lists only the chapter authors).
- Reference check: authorship of the AU form unconfirmed; the three-name author list may belong to the Niger Delta Province geology report rather than the AU form.

### USGS2000
- Reference: U.S. Geological Survey World Petroleum Assessment 2000 Team, 2000, U.S. Geological Survey World Petroleum Assessment 2000: description and results: U.S. Geological Survey Digital Data Series DDS-60. Chapters used: AR, Analysis of assessment results (Ahlbrandt, Charpentier, Klett, Schmoker, Schenk); MC, Monte Carlo simulation method (Charpentier and Klett); AM, The Seventh Approximation (Schmoker and Klett; also USGS Bulletin 2165); GL, Glossary (Klett, Schmoker, Charpentier, Ahlbrandt, Ulmishek).
- DOI or URL: https://pubs.usgs.gov/dds/dds-060/ (chapter authors as listed on that page).
- Cited as: "DDS-60 chaps. AR, MC, AM, GL"; keys dds60-ar, dds60-mc, dds60-am, dds60-gl.
- Ledger ids: ORE-204, 205.
- Access: open access (agency report).
- Read so far (by the project): in full (pages cited).
- Read in this pass: not fetched (index page only; PDFs unreadable). The Bulletin 2165 page https://pubs.usgs.gov/bul/2165/ returned 404.
- Reference check: chapter titles and authors match the project's description; the 1,000-1,500 MMBO and 6,000 cf/bbl values were not checked.

### Dubiel2003
- Reference: Dubiel, R. F., 2003, Geology, depositional models, and oil and gas assessment of the Green River Total Petroleum System, chap. 5 in USGS Uinta-Piceance Assessment Team, DDS-69-B.
- DOI or URL: https://pubs.usgs.gov/publication/ds69B (volume).
- Cited as: key dds69b-ch5.
- Ledger ids: ORE-206.
- Access: open access (agency report).
- Read so far (by the project): in full (Appendix B printed p. 36 cited).
- Read in this pass: not fetched (volume page only).
- Reference check: volume confirmed; chapter pairing unconfirmed.

#### Coal

### Wood1983
- Reference: Wood, Gordon H., Jr., Kehn, Thomas M., Carter, M. Devereux, and Culbertson, William C., 1983, Coal resource classification system of the U.S. Geological Survey: U.S. Geological Survey Circular 891, vii, 65 p.
- DOI or URL: https://pubs.usgs.gov/publication/cir891
- Cited as: "USGS Circular 891 (Wood et al. 1983)", "Table 2 p. 22", "Table 3 p. 24".
- Ledger ids: ORE-117, 130, 131, 132, 133, 134, 135, 142.
- Access: open access (agency report).
- Read so far (by the project): in full (page-level quotations; lignite row read from the rendered page).
- Read in this pass: not fetched (citation page only; the USGS record prints the third and fourth authors as "Carter, M.D., and Culbertson, W.").
- Reference check: matches. Arithmetic only: 6,000 ft is 1,828.8 m (the circular's "(1,800 m)" is a rounding); 1,800 short tons per acre-foot is 1.324 t/m3.

### Ellis1999
- Reference: Chapters of U.S. Geological Survey Professional Paper 1625-A (Fort Union Coal Assessment Team and USGS Energy Resources Team, 1999, 1999 resource assessment of selected Tertiary coal beds and zones in the northern Rocky Mountains and Great Plains region; HTML plus 2 CD-ROMs). Chapter PN: Ellis, M. S., Gunther, G. L., Ochs, A. M., Roberts, S. B., Wilde, E. M., Schuenemeyer, J. H., Power, H. C., Stricker, G. D., and Blake, D., Coal resources, Powder River Basin. Chapter PG: Ellis, M. S., Flores, R. M., Ochs, A. M., Stricker, G. D., Gunther, G. L., Rossi, G. S., Bader, L. R., Schuenemeyer, J. H., and Power, H. C., Gillette coalfield, Powder River Basin: geology, coal quality, and coal resources. Chapter PS: Flores, R. M., and Bader, L. R., Fort Union coal in the Powder River Basin, Wyoming and Montana: a synthesis.
- DOI or URL: https://pubs.usgs.gov/pp/p1625a/ (chapter list); chapter PDFs Chapters/PN.pdf, PG.pdf, PS.pdf under that URL.
- Cited as: "Ellis et al. 1999, PP 1625-A chapters PN, PG, PS".
- Ledger ids: ORE-118, 120, 128.
- Access: open access (agency report).
- Read so far (by the project): in full (chapter PDFs, PDF pages quoted).
- Read in this pass: not fetched (chapter list read; PDFs unreadable).
- Reference check: MISMATCH in attribution. "Ellis et al. 1999" is right for PN and PG only. Chapter PS (the Fort Union 22,000 mi2 figure) is Flores and Bader, not Ellis et al. The volume is credited to the Fort Union Coal Assessment Team.

### Averitt1975
- Reference: Averitt, Paul, 1975, Coal resources of the United States, January 1, 1974: U.S. Geological Survey Bulletin 1412, vi, 131 p.
- DOI or URL: https://pubs.usgs.gov/publication/b1412
- Cited as: "Averitt 1975, USGS Bulletin 1412".
- Ledger ids: ORE-118.
- Access: open access (agency report).
- Read so far (by the project): not stated beyond the research sub-agent.
- Read in this pass: not fetched (citation page only).
- Reference check: matches. The "458,600 mi2 = 13 %" and Mammoth bed figures were not checked.

### Trippi2011
- Reference: Trippi, Michael H., and Tewalt, Susan J., 2011, Geographic information system (GIS) representation of coal-bearing areas in India and Bangladesh: U.S. Geological Survey Open-File Report 2011-1296, iii, 23 p.
- DOI or URL: https://pubs.usgs.gov/publication/ofr20111296
- Cited as: "USGS OFR 2011-1296 Table 1 (CFRI 1993)".
- Ledger ids: ORE-118, 124.
- Access: open access (agency report).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched (shapefiles of coalfields; no counts or areas in it).
- Reference check: authors were not given in the project's citation; now resolved. The "113 coalfields" and "34.7 thousand km2" were not checked.

### BAP2018
- Reference: Bioregional Assessments Programme, 2018 (finalised), Context statement for the Sydney Basin bioregion (Product 1.1), Australian Government.
- DOI or URL: https://www.bioregionalassessments.gov.au/assessments/11-context-statement-sydney-basin-bioregion
- Cited as: "Bioregional Assessments Programme, context statement for the Sydney Basin".
- Ledger ids: ORE-121, 128.
- Access: open access (web).
- Read so far (by the project): not stated.
- Read in this pass: full text fetched in part (landing page; two targeted questions). See ORE-121 spot check: the page gives different numbers from the ledger.
- Reference check: no authors printed; date from page footer.

### Hatch2002
- Reference: Hatch, Joseph R., and Affolter, Ronald H., 2002, Resource assessment of the Springfield, Herrin, Danville, and Baker coals in the Illinois Basin: U.S. Geological Survey Professional Paper 1625-D.
- DOI or URL: https://doi.org/10.3133/pp1625D
- Cited as: "USGS PP 1625-D" (cumulative coal per field).
- Ledger ids: ORE-128, 129.
- Access: open access (agency report).
- Read so far (by the project): "several are the research's own arithmetic".
- Read in this pass: not fetched (USGS page only).
- Reference check: identified from the report number. This is the Illinois Basin volume and is the likely home of any Illinois Basin coal-area figure.

### Ruppert2014
- Reference: Ruppert, Leslie F., and Ryder, Robert T., eds., 2014, Coal and petroleum resources in the Appalachian basin: distribution, geologic framework, and geochemical character: U.S. Geological Survey Professional Paper 1708. Chapter D.1: Coal and coalbed-methane resources in the Appalachian and Black Warrior basins: maps showing the distribution of coal fields, coal beds, and coalbed-methane fields.
- DOI or URL: https://doi.org/10.3133/pp1708 ; chapter https://pubs.usgs.gov/publication/pp1708D.1
- Cited as: "PP 1708-D1".
- Ledger ids: ORE-128, 129.
- Access: open access (agency report).
- Read so far (by the project): research's own arithmetic in part.
- Read in this pass: not fetched (USGS page only; chapter authors not shown).
- Reference check: identified from the report number.

### NMET-Jharia
- Reference: UNIDENTIFIED. "NMET Jharia proposal" (National Mineral Exploration Trust, India). I did not search for it (no search tool); no bibliographic form was available.
- DOI or URL: none.
- Cited as: "NMET Jharia proposal".
- Ledger ids: ORE-128, 129.
- Access: not found.
- Read so far (by the project): not stated.
- Read in this pass: not fetched.
- Reference check: cannot identify.

### SAIMM-coalfields
- Reference: UNIDENTIFIED. "SAIMM J. 105(2)" (Journal of the Southern African Institute of Mining and Metallurgy, vol. 105 no. 2, with "Table I" and "p. 95: 19 coalfields"). A Crossref query on the likely title returned only 2017-2018 articles from the same journal, so I could not tie the 105(2) paper to an author or title.
- DOI or URL: none.
- Cited as: "SAIMM J. 105(2) p. 95", "Table I".
- Ledger ids: ORE-126, 128.
- Access: not found.
- Read so far (by the project): listed as NOT verified.
- Read in this pass: not fetched.
- Reference check: cannot identify.

### Heckel1986
- Reference: Heckel, Philip H., 1986, Sea-level curve for Pennsylvanian eustatic marine transgressive-regressive depositional cycles along midcontinent outcrop belt, North America: Geology, v. 14, no. 4, p. 330-334 (Crossref gives first page 330; last page from the project's context, not confirmed).
- DOI or URL: 10.1130/0091-7613(1986)14<330:SCFPEM>2.0.CO;2
- Cited as: "Heckel 1986".
- Ledger ids: ORE-139, 140.
- Access: paywalled (abstract only).
- Read so far (by the project): search snippet only.
- Read in this pass: not fetched.
- Reference check: identified; the code's claim is about cyclothem durations (160,000-400,000 yr), which I did not verify.

### Rasbury1998
- Reference: Rasbury, E. T., Hanson, G. N., Meyers, W. J., Holt, W. E., Goldstein, R. H., and Saller, A. H., 1998, U-Pb dates of paleosols: constraints on late Paleozoic cycle durations and boundary ages: Geology, v. 26, no. 5, p. 403-406 (Crossref: first page 403).
- DOI or URL: 10.1130/0091-7613(1998)026<0403:UPDOPC>2.3.CO;2
- Cited as: "Rasbury et al. 1998".
- Ledger ids: ORE-139, 140.
- Access: paywalled (abstract only).
- Read so far (by the project): search snippet only.
- Read in this pass: not fetched.
- Reference check: identified by title; the title fits the cyclothem-duration claim, which I did not read.

### Hampson1999
- Reference: probable identification: Hampson, Gary, Stollhofen, Harald, and Flint, Steve, 1999, A sequence stratigraphic model for the Lower Coal Measures (Upper Carboniferous) of the Ruhr district, north-west Germany: Sedimentology, v. 46, no. 6, p. 1199-1231. The code gives only "Hampson 1999"; this is the only 1999 Hampson paper on coal cycles I found by Crossref.
- DOI or URL: 10.1046/j.1365-3091.1999.00273.x
- Cited as: "Hampson 1999".
- Ledger ids: ORE-139, 140.
- Access: paywalled (abstract only).
- Read so far (by the project): search snippet only.
- Read in this pass: not fetched.
- Reference check: probable, unconfirmed.

### SilesiaMDPI
- Reference: UNIDENTIFIED. "the Silesia MDPI paper" (cyclothem or seam counts for Upper Silesia). I could not search for it.
- DOI or URL: none.
- Cited as: "the Silesia MDPI paper".
- Ledger ids: ORE-139, 140.
- Access: not found.
- Read so far (by the project): search snippet only.
- Read in this pass: not fetched.
- Reference check: cannot identify.

### RuhrCounts
- Reference: UNIDENTIFIED. "the Ruhr counts" (measures per seam, 12 m for all coals and about 60 m for economic ones). Possibly the Hampson1999 paper above; not confirmed.
- DOI or URL: none.
- Cited as: "the Ruhr counts".
- Ledger ids: ORE-139, 140.
- Access: not found.
- Read so far (by the project): search snippet only.
- Read in this pass: not fetched.
- Reference check: cannot identify.

### GABowen
- Reference: Geoscience Australia, Bowen Basin (basin summary page, onshore Australia petroleum provinces).
- DOI or URL: https://www.ga.gov.au/scientific-topics/energy/province-sedimentary-basin-geology/petroleum/onshore-australia/bowen-basin
- Cited as: "Geoscience Australia (Bowen Basin about 160,000 km2)".
- Ledger ids: ORE-126.
- Access: open access (agency web page).
- Read so far (by the project): listed as NOT verified.
- Read in this pass: full text fetched (the area sentence).
- Reference check: page found and the figure matches (see ORE-126 spot check).

### Bohacs1997
- Reference: Bohacs, Kevin, and Suter, John, 1997, Sequence stratigraphic distribution of coaly rocks: fundamental controls and paralic examples: AAPG Bulletin, v. 81, no. 10, p. 1612-1639.
- DOI or URL: 10.1306/3b05c3fc-172a-11d7-8645000102c1865d (Crossref).
- Cited as: "Bohacs & Suter 1997".
- Ledger ids: ORE-112, 113, 114.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched (Crossref abstract, verbatim).
- Reference check: matches (authors, year, volume, issue, pages, DOI).

### Ryer1980
- Reference: Ryer, Thomas A., and Langer, Anna W., 1980, Thickness change involved in the peat-to-coal transformation for a bituminous coal of Cretaceous age in central Utah: Journal of Sedimentary Petrology, v. 50, no. 3, p. 987-992 (OpenAlex names the journal Journal of Sedimentary Research, the same SEPM journal).
- DOI or URL: 10.1306/212f7b44-2b24-11d7-8648000102c1865d
- Cited as: "Ryer & Langer 1980".
- Ledger ids: ORE-116.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated; not used by the code.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches. The paper is a single Utah case; the abstract says "approximately 11:1".

### Nelsen2016
- Reference: Nelsen, Matthew P., DiMichele, William A., Peters, Shanan E., and Boyce, C. Kevin, 2016, Delayed fungal evolution did not cause the Paleozoic peak in coal production: Proceedings of the National Academy of Sciences, v. 113, no. 9, p. 2442-2447.
- DOI or URL: 10.1073/pnas.1517943113 ; open copy https://pmc.ncbi.nlm.nih.gov/articles/PMC4780611/
- Cited as: "Nelsen et al. 2016".
- Ledger ids: ORE-015.
- Access: open access.
- Read so far (by the project): not stated.
- Read in this pass: full text fetched (PMC; the introduction's statement on the first appearance of coal and the Carboniferous-Permian peak).
- Reference check: the DOI and authors are right, but the paper is about the Carboniferous-Permian coal peak and fungi, not about when coal formation began. It does contain the sentence "Coal deposits occur by the Early Devonian", so it supports the code's land-plant gate only indirectly.

#### Ore genesis and drivers

### Leach2010a
- Reference: Leach, David L., Taylor, Ryan D., Fey, David L., Diehl, Sharon F., and Saltus, Richard W., 2010, A deposit model for Mississippi Valley-type lead-zinc ores, chap. A of Mineral deposit models for resource assessment: U.S. Geological Survey Scientific Investigations Report 2010-5070-A, 52 p.
- DOI or URL: https://pubs.usgs.gov/sir/2010/5070/a/ ; PDF https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf (6.3 MB; binary to WebFetch). Note the PDF name is SIR10-5070A.pdf.
- Cited as: "USGS SIR 2010-5070-A (the MVT deposit model)"; also called "Leach et al. 2010" in places.
- Ledger ids: ORE-001 (and the MVT notes in ORE-003).
- Access: open access (agency report).
- Read so far (by the project): not stated; quoted as a phrase.
- Read in this pass: abstract fetched (USGS page). The PDF is COULD NOT READ.
- Reference check: matches. Important: the code uses "Leach et al. 2010" for two different papers (this report, and Leach2010b below, Economic Geology 105:593). The authors differ.

### Leach2010b
- Reference: Leach, D. L., Bradley, D. C., Huston, D., Pisarevsky, S. A., Taylor, R. D., and Gardoll, S. J., 2010, Sediment-hosted lead-zinc deposits in Earth history: Economic Geology, v. 105, no. 3, p. 593-625.
- DOI or URL: 10.2113/gsecongeo.105.3.593 (Crossref).
- Cited as: "Leach et al. 2010 (Economic Geology 105:593; sediment-hosted Pb-Zn ... first appearance at 2.02 Ga)".
- Ledger ids: ORE-009, 010, 014, 024, 068.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched (OpenAlex inverted index; the rebuilt text is word-order scrambled in places, so quote only the intact phrases).
- Reference check: matches (authors, year, volume, pages). The abstract divides the deposits into clastic-dominated (which includes what were called SEDEX) and MVT; the code applies the 2.02 Ga gate to both.

### Leach2005
- Reference: Leach, David L., Sangster, Donald F., Kelley, Karen D., Large, Ross R., Garven, Grant, Allen, Cameron R., Gutzmer, Jens, and Walters, Steve, 2005, Sediment-hosted lead-zinc deposits: a global perspective, in Hedenquist, J. W., Thompson, J. F. H., Goldfarb, R. J., and Richards, J. P., eds., Economic Geology One Hundredth Anniversary Volume 1905-2005: Society of Economic Geologists, p. 561-607.
- DOI or URL: 10.5382/av100.18 (Crossref; Crossref prints the title "Sediment-Hosted Lead-Zinc Deposits"). Editors listed from the usual volume record, not from Crossref.
- Cited as: "Leach et al. 2005 (90-200 degC MVT fluids)".
- Ledger ids: ORE-003, 004.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: identified; the 90-200 degC statement is unverified.

### Leach2001
- Reference: Leach, D. L., Bradley, D., Lewchuk, M. T., Symons, D. T. A., de Marsily, G., and Brannon, J., 2001, Mississippi Valley-type lead-zinc deposits through geological time: implications from recent age-dating research: Mineralium Deposita, v. 36, no. 8, p. 711-740.
- DOI or URL: 10.1007/s001260100208
- Cited as: "Leach et al. 2001 (MVT mainly in contractional events)".
- Ledger ids: ORE-006, 007.
- Access: paywalled (abstract only); abstract not retrieved.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: matches. A published discussion exists (Kesler and Carrigan 2002, Mineralium Deposita 37:800-802, open copy https://hdl.handle.net/2027.42/41891).

### Bradley2003
- Reference: Bradley, Dwight C., and Leach, David L., 2003, Tectonic controls of Mississippi Valley-type lead-zinc mineralization in orogenic forelands: Mineralium Deposita, v. 38, no. 6, p. 652-667.
- DOI or URL: 10.1007/s00126-003-0355-2
- Cited as: "Bradley & Leach 2003 (flexure-induced faults)".
- Ledger ids: ORE-008.
- Access: paywalled (abstract only); abstract not retrieved.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: matches (authors, year, volume, pages).

### Garven1985
- Reference: Garven, Grant, 1985, The role of regional fluid flow in the genesis of the Pine Point deposit, Western Canada sedimentary basin: Economic Geology, v. 80, no. 2, p. 307-324.
- DOI or URL: 10.2113/gsecongeo.80.2.307
- Cited as: "Garven 1985 (Pine Point, 60-100 degC, flushes of 0.5-5 Myr)".
- Ledger ids: ORE-003, 004, 006, 013.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches. The abstract contains both the 60-100 degC and the 0.5-5.0 Myr values.

### Garven1984
- Reference: Garven, Grant, and Freeze, R. Allan, 1984, Theoretical analysis of the role of groundwater flow in the genesis of stratabound ore deposits: 1, Mathematical and numerical model (American Journal of Science, v. 284, no. 10, p. 1085-1124); 2, Quantitative results (same issue, p. 1125-1174).
- DOI or URL: 10.2475/ajs.284.10.1085 ; 10.2475/ajs.284.10.1125
- Cited as: "Garven & Freeze 1984 (base case: 1 km water-table drop over 300 km)".
- Ledger ids: ORE-006.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched (OpenAlex, both parts). Neither abstract mentions the 1 km over 300 km base case.
- Reference check: the code does not say which part; the base case is most likely in part 2 (quantitative results). Unconfirmed.

### Bethke1990
- Reference: Bethke, Craig M., and Marshak, Stephen, 1990, Brine migrations across North America: the plate tectonics of groundwater: Annual Review of Earth and Planetary Sciences, v. 18, p. 287-315.
- DOI or URL: 10.1146/annurev.ea.18.050190.001443
- Cited as: "Bethke & Marshak 1990 (topographic relief the dominant driving force)".
- Ledger ids: ORE-006.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated (the quoted phrase suggests the text was seen).
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: matches.

### Hoggard2020
- Reference: Hoggard, Mark J., Czarnota, Karol, Richards, Fred D., Huston, David L., Jaques, A. Lynton, and Ghelichkhan, Siavash, 2020, Global distribution of sediment-hosted metals controlled by craton edge stability: Nature Geoscience, v. 13, no. 7, p. 504-510.
- DOI or URL: 10.1038/s41561-020-0593-2 ; preprint 10.31223/osf.io/2kjvc (EarthArXiv).
- Cited as: "Hoggard et al. 2020 (85 % within 200 km)".
- Ledger ids: ORE-027.
- Access: open copy (preprint) at https://doi.org/10.31223/osf.io/2kjvc (API record https://api.osf.io/v2/preprints/2kjvc/); the journal version is paywalled.
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched (OSF API record of the preprint).
- Reference check: matches.

### Bekker2010
- Reference: Bekker, Andrey, Slack, John F., Planavsky, Noah, Krapez, Bryan, Hofmann, Axel, Konhauser, Kurt O., and Rouxel, Olivier J., 2010, Iron formation: the sedimentary product of a complex interplay among mantle, tectonic, oceanic, and biospheric processes: Economic Geology, v. 105, no. 3, p. 467-508.
- DOI or URL: 10.2113/gsecongeo.105.3.467
- Cited as: "Bekker et al. 2010 (BIF acme 2.65-1.85 Ga; Superior-type on passive-margin shelves)".
- Ledger ids: ORE-016, 021.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched in part (OpenAlex; the stored index ends at "Secular changes in deposition, identified more than 20 years ago,", so the dates are cut off).
- Reference check: matches (authors, year, volume, pages).

### Richards2003
- Reference: Richards, Jeremy P., 2003, Tectono-magmatic precursors for porphyry Cu-(Mo-Au) deposit formation: Economic Geology, v. 98, no. 8, p. 1515-1533.
- DOI or URL: 10.2113/gsecongeo.98.8.1515
- Cited as: "Richards 2003".
- Ledger ids: ORE-031, 035.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: matches.

### Butterworth2016
- Reference: Butterworth, Nathaniel P., Steinberg, D., Muller, R. Dietmar, Williams, Simon E., Merdith, Andrew, and Hardy, Stephen, 2016, Tectonic environments of South American porphyry copper magmatism through time revealed by spatiotemporal data mining: Tectonics, v. 35, no. 12, p. 2847-2862.
- DOI or URL: 10.1002/2016TC004289
- Cited as: "Butterworth et al. 2016".
- Ledger ids: ORE-032, 033, 034.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: matches. The paper covers South American porphyry copper only; the code applies its convergence-rate, obliquity and slab-age peaks to all porphyries.

### Cooke2005
- Reference: Cooke, David R., Hollings, Pete, and Walshe, John L., 2005, Giant porphyry deposits: characteristics, distribution, and tectonic controls: Economic Geology, v. 100, no. 5, p. 801-818.
- DOI or URL: 10.2113/gsecongeo.100.5.801 ; open copy https://figshare.com/articles/journal_contribution/Giant_porphyry_deposits_Characteristics_distribution_and_tectonic_controls/22856222
- Cited as: "Cooke et al. 2005".
- Ledger ids: ORE-036.
- Access: open copy (figshare) at the URL above.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: matches.

### Singer2005
- Reference: Singer, D. A., Berger, V. I., Menzie, W. D., and Berger, B. R., 2005, Porphyry copper deposit density: Economic Geology, v. 100, no. 3, p. 491-514. (DEP covers this.)
- DOI or URL: 10.2113/gsecongeo.100.3.491
- Cited as: "Singer, Berger & Menzie 2005 (USGS record 70197200)".
- Ledger ids: ORE-086.
- Access: paywalled (abstract only).
- Read so far (by the project): via the USGS record, not stated whether full text.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: the code drops the fourth author (B. R. Berger).

### Singer2008
- Reference: Singer, Donald A., Berger, Vladimir I., and Moring, Barry C., 2008, Porphyry copper deposits of the world: database and grade and tonnage models, 2008 (Version 1.0): U.S. Geological Survey Open-File Report 2008-1155, 45 p. (DEP covers this.)
- DOI or URL: https://pubs.usgs.gov/publication/ofr20081155
- Cited as: "USGS OFR 2008-1155 (Singer, Berger & Moring 2008)".
- Ledger ids: ORE-059, 086.
- Access: open access (agency report).
- Read so far (by the project): "Fig. 24, digitised".
- Read in this pass: abstract fetched (the 2 km same-deposit rule is not in the abstract).
- Reference check: matches.

### Singer2009
- Reference: Singer, Donald A., Berger, Vladimir I., and Moring, Barry C., 2009, Sediment-hosted zinc-lead deposits of the world: database and grade and tonnage models: U.S. Geological Survey Open-File Report 2009-1252, v, 62 p. (DEP covers this.)
- DOI or URL: https://pubs.usgs.gov/publication/ofr20091252
- Cited as: "USGS OFR 2009-1252 (Singer, Berger & Moring 2009) p. 2".
- Ledger ids: ORE-087, 088.
- Access: open access (agency report).
- Read so far (by the project): page cited.
- Read in this pass: abstract fetched (the 2 km rule is not in the abstract).
- Reference check: matches. The report mixes MVT and sedex under "sediment-hosted zinc-lead".

### John2010
- Reference: John, David A., ed., Ayuso, Robert A., Barton, Mark D., Blakely, Richard J., Bodnar, Robert J., Dilles, John H., Gray, Floyd, Graybeal, Fred T., Mars, John C., McPhee, Darcy K., Seal, Robert R., Taylor, Ryan D., and Vikre, Peter G., 2010, Porphyry copper deposit model, chap. B of Mineral deposit models for resource assessment: U.S. Geological Survey Scientific Investigations Report 2010-5070-B, xii, 169 p.
- DOI or URL: https://pubs.usgs.gov/sir/2010/5070/b/ ; PDF .../pdf/SIR10-5070B.pdf (26.3 MB; exceeded the 10 MB fetch limit).
- Cited as: "USGS SIR 2010-5070-B (John et al. 2010)" with page numbers (Table E1 p. 28; p. 25; p. 66; pp. 4, 65; p. 68; p. 120).
- Ledger ids: ORE-058, 059, 071, 098, 100, 104, 105.
- Access: open access (agency report).
- Read so far (by the project): page numbers cited (PDF opened).
- Read in this pass: abstract fetched (USGS page; two different fetches listed the authors in different orders; the series page lists David A. John first as editor).
- Reference check: matches "John et al. 2010". The report is a descriptive model; the median ore area 0.6 km2 and the 1-1.5 km height could not be read.

### Sillitoe2005
- Reference: Sillitoe, Richard H., 2005, Supergene oxidized and enriched porphyry copper and related deposits, in Economic Geology One Hundredth Anniversary Volume 1905-2005: Society of Economic Geologists, p. 723-768.
- DOI or URL: 10.5382/av100.22
- Cited as: "Sillitoe 2005".
- Ledger ids: ORE-098, 099.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Sillitoe1996
- Reference: Sillitoe, Richard H., and McKee, Edwin H., 1996, Age of supergene oxidation and enrichment in the Chilean porphyry copper province: Economic Geology, v. 91, no. 1, p. 164-179.
- DOI or URL: 10.2113/gsecongeo.91.1.164
- Cited as: "Sillitoe & McKee 1996 (USGS record 70018070)".
- Ledger ids: ORE-099.
- Access: paywalled (abstract only).
- Read so far (by the project): via the USGS record, not stated whether full text.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches. This paper concerns the Chilean province only.

### Cox1986
- Reference: Cox, Dennis P., and Singer, Donald A., eds., 1986, Mineral deposit models: U.S. Geological Survey Bulletin 1693, xiii, 379 p. (DEP covers the grade-tonnage models.) Descriptive models used: 39a placer Au-PGE (Warren E. Yeend); sedimentary exhalative Zn-Pb; alluvial placer Sn; diamond placers.
- DOI or URL: https://doi.org/10.3133/b1693 ; HTML version https://pubs.usgs.gov/bul/b1693/html/bullfrms.htm ; placer Au-PGE descriptive https://pubs.usgs.gov/bul/b1693/html/bull6945.htm ; placer Au-PGE grade-tonnage .../bull42lh.htm ; sedex descriptive .../bull5nmq.htm ; alluvial placer Sn .../bull3pke.htm ; diamond placers .../bull2xv7.htm.
- Cited as: "USGS Bulletin 1693 (Cox & Singer eds. 1986), models 31a, 39a, 39d, 39e".
- Ledger ids: ORE-068, 091, 218, 219, 220, 221.
- Access: open access (HTML pages fetched).
- Read so far (by the project): "figure intercepts and page numbers cited".
- Read in this pass: full text fetched (the five HTML pages above).
- Reference check: matches. Notes: the USGS series page prints "Originally published 1996; Version 1.0 posted 1998" (the print volume is 1986). The HTML pages do not print the model numbers 31a, 39d, 39e in the text I received, so the mapping of those numbers to sedex, alluvial tin and diamond placers rests on the ledger. The p. 262 page number for the gravel density could not be checked.

### Robert2005
- Reference: Robert, Francois, Poulsen, K. Howard, Cassidy, Kevin F., and Hodgson, C. Jay, 2005, Gold metallogeny of the Superior and Yilgarn cratons, in Economic Geology One Hundredth Anniversary Volume 1905-2005: Society of Economic Geologists, p. 1001-1033.
- DOI or URL: 10.5382/av100.30
- Cited as: "Robert et al. 2005 (orogenic-gold camps every 30-50 km)".
- Ledger ids: ORE-078.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: identified; the 30-50 km spacing is unverified. The paper is about the Superior and Yilgarn cratons.

### Thomas1949
- Reference: Thomas, Marjorie, 1949, A generalization of Poisson's binomial limit for use in ecology: Biometrika, v. 36, no. 1-2, p. 18-25.
- DOI or URL: 10.1093/biomet/36.1-2.18
- Cited as: "Thomas 1949 (Thomas cluster process)".
- Ledger ids: ORE-077.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches (author is Marjorie Thomas).

### Grieve1994
- Reference: Grieve, R. A. F., and Masaitis, V. L., 1994, The economic potential of terrestrial impact craters: International Geology Review, v. 36, no. 2, p. 105-151.
- DOI or URL: 10.1080/00206819409465452
- Cited as: "Grieve & Masaitis 1994, Int. Geol. Rev. 36(2):105-151".
- Ledger ids: ORE-039, 041, 043.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches (volume, issue, pages).

### Grieve2005
- Reference: Grieve, Richard A. F., 2005, Economic natural resource deposits at terrestrial impact structures, in Geological Society, London, Special Publications, v. 248, p. 1-29.
- DOI or URL: 10.1144/GSL.SP.2005.248.01.01
- Cited as: "Grieve 2005 (Geol. Soc. London Spec. Publ. 248:1-29)".
- Ledger ids: ORE-039.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Masaitis1998
- Reference: Masaitis, V. L., 1998, Popigai crater: origin and distribution of diamond-bearing impactites: Meteoritics & Planetary Science, v. 33, no. 2, p. 349-359.
- DOI or URL: 10.1111/j.1945-5100.1998.tb01639.x
- Cited as: "Masaitis 1998, MAPS 33:349-359".
- Ledger ids: ORE-041.
- Access: paywalled (abstract only) (open-access status not checked).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches (volume, issue, pages).

### Abramov2004
- Reference: probable identification: Abramov, Oleg, and Kring, David A., 2004, Numerical modeling of an impact-induced hydrothermal system at the Sudbury crater: Journal of Geophysical Research, v. 109, no. E10, E10007. The code gives only "Abramov & Kring 2004"; this is the 2004 Abramov and Kring paper Crossref returned, and it is specific to Sudbury.
- DOI or URL: 10.1029/2003JE002213
- Cited as: "Abramov & Kring 2004".
- Ledger ids: ORE-042.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: the paper models one crater (Sudbury); the code uses it for the generic 8-25 km crater range and the 1e4-1e6 yr circulation time. Article number E10007 is from memory of the usual form and was not confirmed.

### Osinski2013
- Reference: Osinski, Gordon R., Tornabene, Livio L., Banerjee, Neil R., Cockell, Charles S., Flemming, Roberta, Izawa, Matthew R. M., McCutcheon, Jenine, Parnell, John, Preston, Louisa J., Pickersgill, Annemarie E., Pontefract, Alexandra, Sapers, Haley M., and Southam, Gordon, 2013, Impact-generated hydrothermal systems on Earth and Mars: Icarus, v. 224, no. 2, p. 347-363.
- DOI or URL: 10.1016/j.icarus.2012.08.030
- Cited as: "Osinski et al. 2013 (Icarus 224:347-363)".
- Ledger ids: ORE-042.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Ames1998
- Reference: Ames, D. E., Watkinson, D. H., and Parrish, R. R., 1998, Dating of a regional hydrothermal system induced by the 1850 Ma Sudbury impact event: Geology, v. 26, no. 5, p. 447-450.
- DOI or URL: 10.1130/0091-7613(1998)026<0447:DOARHS>2.3.CO;2
- Cited as: "Ames, Watkinson & Parrish 1998 (Geology 26:447-450)".
- Ledger ids: ORE-042.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only; Crossref prints first page 447).
- Reference check: matches (volume, issue, first page).

### Naldrett2004
- Reference: Naldrett, Anthony J., 2004, Magmatic sulfide deposits: geology, geochemistry and exploration: Springer, Berlin.
- DOI or URL: 10.1007/978-3-662-08444-1
- Cited as: "Naldrett 2004 (Magmatic Sulfide Deposits)".
- Ledger ids: ORE-043.
- Access: book (not online).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Lightfoot2016
- Reference: Lightfoot, Peter C., 2016, Nickel sulfide ores and impact melts: origin of the Sudbury Igneous Complex: Elsevier.
- DOI or URL: no DOI found (a 2017 Economic Geology book review exists: 10.5382/econgeo.112.6.br01).
- Cited as: "Lightfoot 2016, Nickel Sulfide Ores and Impact Melts".
- Ledger ids: ORE-044.
- Access: book (not online).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (existence confirmed only through the Crossref book-review record).
- Reference check: publisher and page count not confirmed in this pass.

### Garnett2005
- Reference: Garnett, Richard H. T., and Bassett, Neville C., 2005, Placer deposits, in Economic Geology One Hundredth Anniversary Volume 1905-2005: Society of Economic Geologists, p. 813-843 (OpenAlex lists p. 813-843).
- DOI or URL: 10.5382/av100.25
- Cited as: "Garnett & Bassett 2005 (placers)".
- Ledger ids: ORE-220, 221.
- Access: paywalled (abstract only).
- Read so far (by the project): not read; the note says full text is needed.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: identified. This is the review that would carry any placer-efficiency or transport-distance figures; it is the first paper to get through a library.

### Lehmann2020
- Reference: Lehmann, Bernd, 2021 (online 2020), Formation of tin ore deposits: a reassessment: Lithos, v. 402-403, article 105756.
- DOI or URL: 10.1016/j.lithos.2020.105756
- Cited as: "Lehmann 2020 (cassiterite within a few km)".
- Ledger ids: ORE-221.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: the paper is an ore-formation review, not obviously about placer transport; the "within a few km" statement is supported by Cox1986 (alluvial placer Sn, "<8 km"), which I read.

### Kesler2006
- Reference: Kesler, S. E., and Wilkinson, B. H., 2006, The role of exhumation in the temporal distribution of ore deposits: Economic Geology, v. 101, no. 5, p. 919-922.
- DOI or URL: 10.2113/gsecongeo.101.5.919
- Cited as: "Kesler & Wilkinson 2006 and 2008 (exhumation)".
- Ledger ids: ORE-220, 221.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Kesler2008
- Reference: Kesler, Stephen E., and Wilkinson, Bruce H., 2008, Earth's copper resources estimated from tectonic diffusion of porphyry copper deposits: Geology, v. 36, no. 3, p. 255-258.
- DOI or URL: 10.1130/G24317A.1
- Cited as: "Kesler & Wilkinson ... 2008 (exhumation)".
- Ledger ids: ORE-220, 221.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only; last page 258 from the usual form, not confirmed).
- Reference check: probable match for "2008"; the code's phrasing "(exhumation)" fits the 2006 paper better than this one.

#### Found in this pass, not cited by the code

### Basuki2002
- Reference: Basuki, N. I. (Crossref lists this author only; a co-author may exist), 2002, A review of fluid inclusion temperatures and salinities in Mississippi Valley-type Zn-Pb deposits: identifying thresholds for metal transport: Exploration and Mining Geology, v. 11, no. 1-4, p. 1-17.
- DOI or URL: 10.2113/11.1-4.1
- Cited as: not cited.
- Ledger ids: (supports ORE-003, 004)
- Access: paywalled (abstract only).
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: n/a.

### Gumsley2017
- Reference: Gumsley, Ashley P., Chamberlain, Kevin R., Bleeker, Wouter, Soderlund, Ulf, de Kock, Michiel O., Larsson, Emilie R., and Bekker, Andrey, 2017, Timing and tempo of the Great Oxidation Event: Proceedings of the National Academy of Sciences, v. 114, no. 8, p. 1811-1816.
- DOI or URL: 10.1073/pnas.1608824114 ; open copy https://www.pnas.org/content/pnas/114/8/1811.full.pdf
- Cited as: not cited (it is the reference behind the Wikipedia GOE dates).
- Ledger ids: (Earth reference for ORE-014, 016, 017)
- Access: open access.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: n/a.

Not bibliographic: Excavated Variants; Create Ore Excavation (COE); Create: Diesel Generators (CDG) and its default thresholds; Create ore stones; vanilla 1.21.1 ore-density measurement (tools/review/deposit-audit.sh); deposit audit tool; project documents (docs/deeptime-design-decisions.md, docs/research/notes/ore_genesis_modeling.md, plan sections, surface_processes_climate.md section 4, build log STATUS.md); Wikipedia pages (Upper Silesia, Indian field areas, Bowen Basin), listed in the project as weak secondary sources.

## Moisture, Köppen classes and biomes (MOI, BIO)

### Peel2007
- Reference: Peel, Murray C.; Finlayson, Brian L.; McMahon, Thomas A. 2007. Updated world map of the Koppen-Geiger climate classification. Hydrology and Earth System Sciences 11(5):1633-1644.
- DOI or URL: https://doi.org/10.5194/hess-11-1633-2007 ; PDF https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf ; the author's page with the same table https://people.eng.unimelb.edu.au/mpeel/koppen.html
- Cited as: "Peel, Finlayson & McMahon 2007, Table 1" (koppen.rs header "the rules the research note cites"; plan §8.3; STATUS P1-05, P1-17)
- Ledger ids: BIO-001 to BIO-016 (BIO-017 is a game-choice about the summer half)
- Access: open access (gold)
- Read so far (by the project): nothing stated; the research note it points to cites Wikipedia, not this paper
- Read in this pass: full text fetched (decoded PDF text layer: Table 1 with its notes, the paragraphs on test order and on the Cs/Cw tie rule, the discussion with the land shares)
- Reference check: authors, year, journal, volume, pages match. Table 1 is the table the code cites. The research note's source (Wikipedia) differs from the paper in the B threshold's form only (20 x MAT + 280/140/0 against 10 x (2 x MAT + 28/14/0), the same numbers) and in listing a "-3 degC" option the paper does not have. Details in Part 2.

### WikipediaKoppen
- Reference: Wikipedia, "Koppen climate classification" (accessed 2026-10-01 through WebFetch; cites Beck et al. 2018 and Kottek et al. 2006 for the criteria).
- DOI or URL: https://en.wikipedia.org/wiki/K%C3%B6ppen_climate_classification
- Cited as: "Source: Wikipedia" (research note surface_processes_climate.md section 5, "Koppen-Geiger rules")
- Ledger ids: indirectly BIO-001 to BIO-016
- Access: open access (web page, changes over time)
- Read so far (by the project): a URL in the note
- Read in this pass: full page fetched (summariser): B threshold "Multiply the average annual temperature in degC by 20, then add: 280 if 70% or more of the total precipitation is in the high-sun half of the year; 140 if 30%-70% ...; 0 if less than 30%"; BW below "50%" of the threshold; "(or -3 degC)" for C/D; E "warmest temperature of any month being below 10 degC"; s "driest month of summer receives less than 40 mm" with "at least three times as much precipitation in the wettest month of winter"; w "at least ten times as much rain in the wettest month of summer as in the driest month of winter"; a "above 22 degC"; d "coldest month averaging below -38 degC"; Am "at least 100 - (total annual precipitation(mm)/25)".
- Reference check: the note's text matches this page. Not a scientific source; the 20 x MAT + 280/140/0 form is Koppen's original and is arithmetically the same as Peel's 10 x (2 x MAT + 28/14/0).

### Beck2018
- Reference: Beck, Hylke E.; Zimmermann, Niklaus E.; McVicar, Tim R.; Vergopolan, Noemi; Berg, Alexis; Wood, Eric F. 2018. Present and future Koppen-Geiger climate classification maps at 1-km resolution. Scientific Data 5:180214.
- DOI or URL: https://doi.org/10.1038/sdata.2018.214 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC6207062/ ; data figshare 6396959
- Cited as: "Beck et al. 2018, Sci. Data 5, 180214" (koppen.rs COLOURS; design doc; earth/MOI.md validation data)
- Ledger ids: none (a legend palette; validation data)
- Access: open access (gold)
- Read so far (by the project): data downloaded and used for Earth comparisons
- Read in this pass: Table 2 read through PMC (summariser); this table is Peel's Table 1 with "Not (B)" added to the A, C, D and E rows
- Reference check: matches (authors, year, volume, article number). The paper's title says 1 km; whether it also provides the 0.5 degree maps the project used was not checked.

### Beck2023
- Reference: Beck, Hylke E.; McVicar, Tim R.; Vergopolan, Noemi; Berg, Alexis; Lutsko, Nicholas J.; Dufour, Ambroise; Zeng, Zhenzhong; Jiang, Xin Sheng; van Dijk, Albert I. J. M.; Miralles, Diego G. 2023. High-resolution (1 km) Koppen-Geiger maps for 1901-2099 based on constrained CMIP6 projections. Scientific Data 10:724.
- DOI or URL: https://doi.org/10.1038/s41597-023-02549-6 ; data figshare 21789074
- Cited as: "Beck et al. 2023, Sci. Data 10, 724 (monthly temperature and precipitation, 1961 to 1990)"
- Ledger ids: none (validation data for earth/MOI.md)
- Access: open access (gold)
- Read so far (by the project): used as observed data
- Read in this pass: bibliographic record only (OpenAlex)
- Reference check: authors, year, volume, article number match. The paper's title is about Koppen-Geiger maps; whether the figshare 21789074 data are monthly temperature and precipitation was not checked.

### SmithBarstad2004
- Reference: Smith, Ronald B. and Barstad, Idar. 2004. A linear theory of orographic precipitation. Journal of the Atmospheric Sciences 61(12):1377-1391.
- DOI or URL: https://doi.org/10.1175/1520-0469(2004)061<1377:ALTOOP>2.0.CO;2
- Cited as: "Smith & Barstad 2004, eq. 49 (J. Atmos. Sci. 61, 1377)"
- Ledger ids: MOI-034, MOI-035 (and MOI-036 to MOI-040 through the software)
- Access: only the record was reachable (the AMS page returned 403); open-access status not checked
- Read so far (by the project): nothing stated; formula taken from the Python port
- Read in this pass: abstract fetched (OpenAlex): "a linear theory of orographic precipitation that incorporates airflow dynamics, condensed water advection, and downslope evaporation ... vertically integrated steady-state equations solved via Fourier transform"
- Reference check: matches (authors, year, journal, volume, issue, pages). Whether the transfer function is equation 49 could not be read.

### FastscapeLEM
- Reference: Lange, Raphael (copyright 2020) and contributors, fastscape-lem/orographic-precipitation, "a Python framework that implements the Linear Theory of Orographic Precipitation following Smith & Barstad (2004)". MIT licence.
- DOI or URL: https://github.com/fastscape-lem/orographic-precipitation ; commit baf8a0c13855fdef88b4ad995fd59c6e69834e00 (12 August 2024, "Merge pull request #9 from EstebanAce/master - update build system")
- Cited as: "fastscape-lem/orographic-precipitation (MIT, copyright 2020 Raphael Lange; commit baf8a0c) ... the defaults of fastscape_ext.py" (orographic.rs header, params.rs, NOTICE.md)
- Ledger ids: MOI-034 to MOI-040, MOI-043
- Access: open access (software, MIT)
- Read so far (by the project): ported (equations and defaults)
- Read in this pass: full text fetched (the source of orographic_precipitation.py and fastscape_ext.py on the master branch; the licence file)
- Reference check: commit exists and the licence reads "MIT License, Copyright (c) 2020 Raphael Lange". The commit is a merge commit on master; the package may have changed since (not diffed).

### HeldSoden2006
- Reference: Held, Isaac M. and Soden, Brian J. 2006. Robust responses of the hydrological cycle to global warming. Journal of Climate 19(21):5686-5699.
- DOI or URL: https://doi.org/10.1175/JCLI3990.1 ; open copy https://www.gfdl.noaa.gov/bibliography/related_files/ih0601.pdf
- Cited as: "Held & Soden 2006 (about 2 percent per kelvin)"
- Ledger ids: MOI-064 (and the 0.07/K relatives MOI-042, MOI-063)
- Access: open copy (GFDL host)
- Read so far (by the project): nothing stated
- Read in this pass: full text fetched (decoded PDF text layer: Figure 1 caption, the text on mass flux and Equation 7)
- Reference check: matches. The abstract itself has no percent-per-kelvin figure; the numbers are in the text (Part 2).

### Thornthwaite1948
- Reference: Thornthwaite, C. W. 1948. An approach toward a rational classification of climate. Geographical Review 38(1):55-94. (The end page 94 is from my recollection; OpenAlex lists only the first page.)
- DOI or URL: https://doi.org/10.2307/210739
- Cited as: "Thornthwaite 1948" (snapshot.rs thornthwaite doc)
- Ledger ids: MOI-065, MOI-066 (and BIO-038 through the aridity index)
- Access: paywalled (closed per OpenAlex)
- Read so far (by the project): nothing stated
- Read in this pass: bibliographic record only; the formula was checked in a secondary implementation (the R package SPEI source)
- Reference check: title, author, year, volume, issue match the record; pages see above.

### Willmott1985
- Reference: Willmott, Cort J.; Rowe, Clinton M.; Mintz, Yale. 1985. Climatology of the terrestrial seasonal water cycle. Journal of Climatology 5(6):589-606.
- DOI or URL: https://doi.org/10.1002/joc.3370050602
- Cited as: "Willmott et al. 1985 (form above 26.5 degC)"
- Ledger ids: MOI-065
- Access: paywalled (closed per OpenAlex)
- Read so far (by the project): nothing stated
- Read in this pass: bibliographic record only
- Reference check: authors, year, volume, issue, pages match the record. That this paper holds the -415.85 + 32.24 T - 0.43 T^2 branch could not be checked.

### Fu1981 and Zhang2004
- Reference: see Fu1981 and Zhang2004 in bib/HYD.md (same citation, "Fu 1981; Zhang et al. 2004"). Fu, B. P. 1981 could not be identified beyond author and year (recollected: Scientia Atmospherica Sinica 5(1):23-31, in Chinese; not verified). Zhang, L.; Hickel, K.; Dawes, W. R.; Chiew, F. H. S.; Western, A. W.; Briggs, P. R. 2004. A rational function approach for estimating mean annual evapotranspiration. Water Resources Research 40(2), W02502, https://doi.org/10.1029/2003WR002710.
- Cited as: "Fu 1981; Zhang et al. 2004" (surface/climate.rs header and fu_et)
- Ledger ids: MOI-068
- Access: Fu1981 not found; Zhang2004 only the abstract was reachable
- Read so far (by the project): nothing stated
- Read in this pass: Zhang2004 abstract fetched (OpenAlex); Fu1981 not fetched
- Reference check: see the HYD file. The Zhang abstract does not mention Fu's equation or omega = 2.6.

### BernerKothavala2001
- Reference: Berner, Robert A. and Kothavala, Zavareh. 2001. GEOCARB III: a revised model of atmospheric CO2 over Phanerozoic time. American Journal of Science 301(2):182-204.
- DOI or URL: https://doi.org/10.2475/ajs.301.2.182 ; open PDF https://ajsonline.org/article/61570.pdf
- Cited as: "Berner & Kothavala 2001 (runoff rises about 3.8 percent per kelvin, GEOCARB)"; params.rs also cites Berner 1994 for 0.038
- Ledger ids: MOI-073
- Access: open access (bronze)
- Read so far (by the project): nothing stated
- Read in this pass: full text fetched (decoded PDF text layer; the passages on the runoff coefficient RUN)
- Reference check: authors (Berner and Kothavala), year, journal, volume, pages match. The value attributed to this paper does not: see Part 2, MOI-073.

### Berner1994
- Reference: Berner, Robert A. 1994. GEOCARB II: a revised model of atmospheric CO2 over Phanerozoic time. American Journal of Science 294(1):56-91.
- DOI or URL: https://doi.org/10.2475/ajs.294.1.56 ; open PDF https://ajs.scholasticahq.com/article/60629.pdf
- Cited as: "Berner 1994" (params.rs runoff_temp_per_k)
- Ledger ids: MOI-073
- Access: open access (bronze per OpenAlex)
- Read so far (by the project): nothing stated
- Read in this pass: bibliographic record only. Berner & Kothavala 2001 state that GEOCARB II used RUN = 0.038 for all times (results of Manabe and Bryan 1985 for runoff).
- Reference check: authors, year, journal, volume, pages match the record.

### Ohmura1992
- Reference: Ohmura, Atsumu; Kasser, Peter; Funk, Martin. 1992. Climate at the equilibrium line of glaciers. Journal of Glaciology 38(130):397-411.
- DOI or URL: https://doi.org/10.3189/S0022143000002276
- Cited as: "Ohmura, Kasser & Funk 1992 (ELA fit P = 645 + 296 T + 9 T^2)" (glaciers.rs header, params.rs)
- Ledger ids: BIO-083 (and HYD-039 in the HYD file)
- Access: open access (page readable without login)
- Read so far (by the project): "cited from memory, not re-read" for the -2 degC figure
- Read in this pass: page fetched (summariser); see the HYD file for the check
- Reference check: matches.

### MiddletonThomas1997 and Zomer2022
- Reference: Middleton, Nick and Thomas, David (eds.). 1997. World Atlas of Desertification, 2nd edition. UNEP / Arnold, London (first edition UNEP 1992). Zomer, Robert J.; Xu, Jianchu; Trabucco, Antonio. 2022. Version 3 of the Global Aridity Index and Potential Evapotranspiration Database. Scientific Data 9. https://doi.org/10.1038/s41597-022-01493-1 (PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC9287331/)
- Cited as: "UNEP's hyper-arid, arid, semi-arid, dry sub-humid, humid" (biomes.rs header, placement.json _comment); no document named
- Ledger ids: BIO-038
- Access: atlas is a book (not online); Zomer2022 open access
- Read so far (by the project): nothing stated; no document, year or threshold values given
- Read in this pass: Zomer2022 full text fetched (summariser: class table); the atlas not seen
- Reference check: UNIDENTIFIED in the code; identified here as UNEP 1992 or 1997. Class limits differ between the two editions: see Part 2, BIO-038.

### Fick2017 and ODonnell2012 (BIO15)
- Reference: Fick, Stephen E. and Hijmans, Robert J. 2017. WorldClim 2: new 1-km spatial resolution climate surfaces for global land areas. International Journal of Climatology 37(12):4302-4315. https://doi.org/10.1002/joc.5086. O'Donnell, Michael S. and Ignizio, Drew A. 2012. Bioclimatic predictors for supporting ecological applications in the conterminous United States. U.S. Geological Survey Data Series 691. https://pubs.usgs.gov/publication/ds691
- DOI or URL: as above; the variable list is at https://www.worldclim.org/data/bioclim.html
- Cited as: "WorldClim's BIO15" (ClimateSample.java precipitationSeasonality)
- Ledger ids: BIO-095
- Access: Fick2017 closed (OpenAlex); ODonnell2012 open (USGS); the WorldClim page is open
- Read so far (by the project): nothing stated
- Read in this pass: WorldClim page fetched: "BIO15 = Precipitation Seasonality (Coefficient of Variation)"; the USGS landing page did not define BIO15 on the page I got
- Reference check: the variable name matches; the exact formula (n or n-1 in the standard deviation) is not on the pages I read.

### WorldbuildingPasta
- Reference: "Worldbuilding Pasta" (blog author; no personal name given on the page I read). An Apple Pie from Scratch, Part VIb: Climate: Biomes and Climate Zones. Blog post, May 2020.
- DOI or URL: https://worldbuildingpasta.blogspot.com/2020/05/an-apple-pie-from-scratch-part-vib.html
- Cited as: "Worldbuilding Pasta, An Apple Pie From Scratch, Part VIb" (belts.rs header, params.rs sst_*, lapse.rs via the research note)
- Ledger ids: MOI-009, MOI-014, MOI-017 to MOI-021, MOI-025, MOI-046, MOI-048, MOI-051 (the 4.46 degC/km figure, unused)
- Access: open access (blog)
- Read so far (by the project): a URL in the research note; "no statement whether the page was fetched or recalled"
- Read in this pass: full page fetched (summariser: lapse rate, rain reach, gyre effects, ITCZ and pressure latitudes, relief effects)
- Reference check: a non-peer-reviewed source (a worldbuilding blog). Title on the page: "An Apple Pie from Scratch, Part VIb: Climate: Biomes and Climate Zones". It states its own lapse rate (4.46 degC per km, as used by Clima-Sim) and does not cite measurements for the others.

### ICAO1993
- Reference: International Civil Aviation Organization, 1993, as cited in Wikipedia's "Lapse rate" article (reference 7 there). Full title of the ICAO document not seen by me.
- DOI or URL: none seen
- Cited as: "the standard atmosphere's" 6.5 degC/km (lapse.rs header, params.rs); no document named
- Ledger ids: MOI-051, MOI-053
- Access: not found (the ICAO document itself)
- Read so far (by the project): nothing; no document named
- Read in this pass: not fetched; the Wikipedia page quotes it
- Reference check: UNIDENTIFIED in the code; the ICAO standard atmosphere is the usual source for 6.5 K/km below 11 km.

### Minder2010 and Rolland2003 (observed lapse rates, not cited by the project)
- Reference: Minder, Justin R.; Mote, Philip W.; Lundquist, Jessica D. 2010. Surface temperature lapse rates over complex terrain: lessons from the Cascade Mountains. Journal of Geophysical Research: Atmospheres 115(D14), D14122. https://doi.org/10.1029/2009JD013493. Rolland, Christian. 2003. Spatial and seasonal variations of air temperature lapse rates in Alpine regions. Journal of Climate 16(7):1032-1046. https://doi.org/10.1175/1520-0442(2003)016<1032:SASVOA>2.0.CO;2
- Cited as: not cited by the project
- Ledger ids: MOI-051 (comparison)
- Access: Minder2010 closed (OpenAlex); Rolland2003 not checked
- Read so far (by the project): n/a
- Read in this pass: abstracts fetched (OpenAlex)
- Reference check: n/a. (Article number D14122 for Minder is from my recollection, not seen.)

### Tozer2019
- Reference: Tozer, Brook; Sandwell, David T.; Smith, Walter H. F.; Olson, Christopher; Beale, J.; Wessel, Paul. 2019. Global bathymetry and topography at 15 arc sec: SRTM15+. Earth and Space Science 6(10):1847-1864.
- DOI or URL: https://doi.org/10.1029/2019EA000658 (gold open access)
- Cited as: "GMT earth_relief (SRTM15+, Tozer et al. 2019)"
- Ledger ids: none (relief used in the Earth comparisons)
- Access: open access
- Read so far (by the project): used as data
- Read in this pass: bibliographic record only
- Reference check: matches (authors, year, volume, pages).

### Felzenszwalb2012
- Reference: Felzenszwalb, Pedro F. and Huttenlocher, Daniel P. 2012. Distance transforms of sampled functions (the title is from my recollection; the record I got had no title). Theory of Computing 8:415-428.
- DOI or URL: https://doi.org/10.4086/toc.2012.v008a019 ; open PDF https://theoryofcomputing.org/articles/v008a019/v008a019.pdf
- Cited as: "Felzenszwalb and Huttenlocher's distance transform (written from the paper's description)" (bake/biomes.rs distance_cells)
- Ledger ids: BIO-063 (the algorithm is not a ledger row)
- Access: open access (diamond)
- Read so far (by the project): "written from the paper's description"
- Read in this pass: bibliographic record only
- Reference check: the project gave no year or venue; the algorithm is not a scientific claim.

### Design-only references of docs/design/p1-26-per-cell-climate.md (no MOI or BIO row relies on them yet)
All verified as bibliographic records (OpenAlex), none read; open-access status not checked. A dagger marks those the design doc itself says were "cited from memory, not re-read".
- North1983: North, G. R.; Mengel, J. G.; Short, D. A. 1983. Simple energy balance model resolving the seasons and the continents: application to the astronomical theory of the ice ages. Journal of Geophysical Research 88(C11):6576-6586. https://doi.org/10.1029/JC088iC11p06576. Matches the design doc.
- Zhuang2017: Zhuang, Kelin; North, Gerald R.; Stevens, Mark J. 2017. A NetCDF version of the two-dimensional energy balance model based on the full multigrid algorithm. SoftwareX 6:198-202. https://doi.org/10.1016/j.softx.2017.07.003. Matches.
- Ziegler2021: Ziegler, Elisa and Rehfeld, Kira. 2021. TransEBM v. 1.0: description, tuning, and validation of a transient model of the Earth's energy balance in two dimensions. Geoscientific Model Development 14(5):2843-2866. https://doi.org/10.5194/gmd-14-2843-2021. Matches (design doc: GMD 14, 2843).
- NorthKim2017: North, Gerald R. and Kim, Kwang-Yul. 2017. Energy Balance Climate Models. Wiley-VCH. Book; not looked up.
- North1981: North, Gerald R.; Cahalan, Robert F.; Coakley, James A. 1981. Energy balance climate models. Reviews of Geophysics and Space Physics 19(1):91-121. https://doi.org/10.1029/RG019i001p00091. Matches.
- Stommel1948: Stommel, Henry. 1948. The westward intensification of wind-driven ocean currents. Transactions, American Geophysical Union 29(2):202-206. https://doi.org/10.1029/TR029i002p00202. Matches.
- Rose2009: Rose, Brian E. J. and Marshall, John. 2009. Ocean heat transport, sea ice, and multiple climate states: insights from energy balance models. Journal of the Atmospheric Sciences 66(9):2828-2843. https://doi.org/10.1175/2009JAS3039.1. Matches.
- Trenberth2001: Trenberth, Kevin E. and Caron, Julie M. 2001. Estimates of meridional atmosphere and ocean heat transports. Journal of Climate 14(16):3433-3443. https://doi.org/10.1175/1520-0442(2001)014<3433:EOMAAO>2.0.CO;2. Matches.
- Dommenget2011: Dommenget, Dietmar and Floeter, Janine. 2011. Conceptual understanding of climate change with a globally resolved energy balance model. Climate Dynamics 37(11-12):2143-2165. https://doi.org/10.1007/s00382-011-1026-0. Matches.
- Stassen2019: Stassen, Christian; Dommenget, Dietmar; Loveday, Nicholas. 2019. A hydrological cycle model for the Globally Resolved Energy Balance (GREB) model v1.0. Geoscientific Model Development 12(1):425-440. https://doi.org/10.5194/gmd-12-425-2019. Matches.
- Fanning1996: Fanning, Augustus F. and Weaver, Andrew J. 1996. An atmospheric energy-moisture balance model: climatology, interpentadal climate change, and coupling to an ocean general circulation model. Journal of Geophysical Research 101(D10):15111-15128. https://doi.org/10.1029/96JD01017. Matches. (The design doc also names "Weaver et al. 2001 (UVic EMBM)"; not looked up.)
- Siler2018: Siler, Nicholas; Roe, Gerard H.; Armour, Kyle C. 2018. Insights into the zonal-mean response of the hydrologic cycle to global warming from a diffusive energy balance model. Journal of Climate 31(18):7481-7493. https://doi.org/10.1175/JCLI-D-18-0081.1. Matches.
- Bischoff2014: Bischoff, Tobias and Schneider, Tapio. 2014. Energetic constraints on the position of the intertropical convergence zone. Journal of Climate 27(13):4937-4951. https://doi.org/10.1175/JCLI-D-13-00650.1. Matches.
- Wagner2015: Wagner, Till J. W. and Eisenman, Ian. 2015. How climate model complexity influences sea ice stability. Journal of Climate 28(10):3998-4014. https://doi.org/10.1175/JCLI-D-14-00654.1. Matches.
- Ferreira2018 (dagger): Ferreira, David; Cessi, Paola; Coxall, Helen K.; de Boer, Agatha M.; Dijkstra, Henk A.; Drijfhout, Sybren S.; Eldevik, Tor; Harnik, Nili; McManus, Jerry F.; Marshall, David P.; Nilsson, Johan; Roquet, Fabien; Schneider, Tapio; Wills, Robert C. 2018. Atlantic-Pacific asymmetry in deep water formation. Annual Review of Earth and Planetary Sciences 46(1):327-352. https://doi.org/10.1146/annurev-earth-082517-010045. Matches the design doc.
- Lindzen1987 (dagger): Lindzen, Richard S. and Nigam, Sumant. 1987. On the role of sea surface temperature gradients in forcing low-level winds and convergence in the tropics. Journal of the Atmospheric Sciences 44(17):2418-2436. https://doi.org/10.1175/1520-0469(1987)044<2418:OTROSS>2.0.CO;2. Matches.
- Bretherton2004 (dagger): Bretherton, Christopher S.; Peters, Matthew E.; Back, Larissa E. 2004. Relationships between water vapor path and precipitation over the tropical oceans. Journal of Climate 17(7):1517-1528. https://doi.org/10.1175/1520-0442(2004)017<1517:RBWVPA>2.0.CO;2. Matches.
- Manabe1969 (dagger): Manabe, Syukuro. 1969. Climate and the ocean circulation: I. The atmospheric circulation and the hydrology of the earth's surface. Monthly Weather Review 97(11):739-774. https://doi.org/10.1175/1520-0493(1969)097<0739:CATOC>2.3.CO;2. Matches (the record's title is truncated to "Climate and the ocean circulation"; the Part I subtitle is from my recollection).
- Klein1993 (dagger): Klein, Stephen A. and Hartmann, Dennis L. 1993. The seasonal cycle of low stratiform clouds. Journal of Climate 6(8):1587-1606. https://doi.org/10.1175/1520-0442(1993)006<1587:TSCOLS>2.0.CO;2. Matches.
- Serreze1992 (dagger): Serreze, Mark C.; Schnell, Russell C.; Kahl, Jonathan D. W. 1992. Low-level temperature inversions of the Eurasian Arctic and comparisons with Soviet drifting station data. Journal of Climate 5(6):615-629. https://doi.org/10.1175/1520-0442(1992)005<0615:LLTIOT>2.0.CO;2. MISMATCH in author order: the design doc writes "Serreze, Kahl & Schnell"; the record lists Serreze, Schnell, Kahl.

### Not scientific sources, or not checked
Wikipedia "Hadley cell" and NOAA JetStream "Global atmospheric circulations" (used only to check the 30 and 60 degree belts, Part 2). goSPL's documentation for the ELA rule and climlab (Rose 2018, JOSS, named in the design doc) were not opened; no MOI or BIO row relies on them beyond BIO-083's use of the Ohmura rule. Clausius-Clapeyron (MOI-042, MOI-063; no author): identified in this pass with the 0.07 per kelvin relation inside Held & Soden 2006, eq. 7 (Part 2). "Earth's zonal means in round numbers" (belt_precip, MOI-008, MOI-062): no source behind the numbers; not looked for. The Amazon recycling figure (MOI-026): not looked for.

### Not bibliographic
Vanilla's OverworldBiomeBuilder, Terralith, Tectonic (game and mod data), rustfft 6.4.1 (library), climlab as a code dependency (another subsystem's), the plan, design documents and decision log (repo documents).

---

## Scale, provinces, pole cap and bake (SCL, PRV, POL, BAK)

### USGS-geologic-provinces-map (no author, no year; key: USGSprov)
- Reference: U.S. Geological Survey, Earthquake Hazards Program, crustal-structure maps page, "map of world geologic provinces" (legend: Shield; Platform; Phanerozoic orogen; Foreland or intracratonic basin; Large igneous province; Extended continental crust; Oceanic crust 0-20 Ma, 20-65 Ma, >65 Ma). Author named only as "USGS". It is the map reproduced as `World_geologic_provinces.jpg` (Wikipedia/Commons, uploaded 22 April 2005, the figure in the Wikipedia "Craton" article). The map is a companion to the USGS global seismic-refraction crustal database (the Mooney crust programme); I did not find a named compiler, a dated report, or an OFR number for the map itself. NOT USGS Open-File Report 97-470: that series (World Energy Project, e.g. OFR 97-470F, "Maps showing geology, oil and gas fields, and geologic provinces of the Asia Pacific Region", Steinshouer, Qiang, McCabe & Ryder, 1999, fetched at https://pubs.usgs.gov/publication/ofr97470F) maps petroleum geologic provinces by region and has a different scheme.
- DOI or URL: original page https://earthquake.usgs.gov/data/crust/maps.php (now HTTP 404; Commons records it and an archive.org copy, snapshot 2020-06-03, `web.archive.org/web/20200603051545/https://earthquake.usgs.gov/data/crust/maps.php`, which the tool cannot fetch). Image: https://upload.wikimedia.org/wikipedia/commons/a/a9/World_geologic_provinces.jpg ; file page https://commons.wikimedia.org/wiki/File:World_geologic_provinces.jpg
- Cited as: "USGS map 'Geologic provinces of the world'", "USGS: 20 Ma", "USGS: 65 Ma", "the USGS map's 150 km"
- Ledger ids: PRV-001, PRV-002, PRV-003, PRV-004, PRV-021 (context for PRV-010 to PRV-020, PRV-022, BAK-010)
- Access: open copy (image, public domain) at the URLs above; the explanatory page is gone from usgs.gov and only on the Internet Archive
- Read so far (by the project): not stated ("by eye against the USGS map")
- Read in this pass: search result snippet only / secondary (Commons file description and Wikipedia caption fetched; the image itself and the USGS page not read)
- Reference check: the title in the code ("Geologic provinces of the world") is the Wikipedia caption wording, not a USGS title; the map has no recoverable author or year. Class names and the three oceanic age bands (0-20, 20-65, >65 Ma) match the code. Legend wording differs from the code's classes in two places: the map says "Phanerozoic orogen" and "Foreland or intracratonic basin". The stated 150 km figure is a feature-size limit chosen to match the seismic-refraction data resolution, so the code's "smallest feature the map shows" use is faithful. Class definitions and the age cut-off for continental classes were not found (no legend text beyond names).

### Cortial2019
- Reference: Cortial, Y., Peytavie, A., Galin, E., Guerin, E., 2019. Procedural Tectonic Planets. Computer Graphics Forum 38(2): 1-11 (Eurographics 2019). (Crossref: first-page 1, last 11 not shown; volume/issue confirmed.)
- DOI or URL: 10.1111/cgf.13614 ; author copy https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf
- Cited as: "Cortial, Peytavie, Galin & Guerin 2019, Procedural Tectonic Planets, Appendix A, Table 3"; "Cortial: 2 My"; "Cortial: 6,370 km"
- Ledger ids: SCL-029, SCL-034, SCL-042, SCL-043, SCL-044 (context SCL-030)
- Access: open copy (author PDF) at the URL above; Wiley page returned 403
- Read so far (by the project): research note says read from the PDF in a 2026-09-27 session; Appendix A coverage not stated
- Read in this pass: not fetched (author PDF exceeds the tool's 10 MB limit; Wiley and Eurographics digital library 403)
- Reference check: authors, title, journal, year, volume match Crossref. Table and appendix numbering not verifiable.

### Mallard2016
- Reference: Mallard, C., Coltice, N., Seton, M., Mueller, R. D., Tackley, P. J., 2016. Subduction controls the distribution and fragmentation of Earth's tectonic plates. Nature 535(7610): 140-143. Published 14 June 2016.
- DOI or URL: 10.1038/nature17992 ; https://www.nature.com/articles/nature17992 ; PubMed 27309815 ; submitted version on HAL hal-01355818 (page blocked for the tool by an Anubis challenge)
- Cited as: "Mallard et al. 2016" (plan §8.1), "Mallard et al. 2016, Nature 535:140-143" (plate-collisions doc)
- Ledger ids: SCL-030
- Access: paywalled (abstract only); HAL submitted version exists but was not reachable; the research note links a Caltech course copy (https://web.gps.caltech.edu/~jstock/161/PapersToRead/nature17992-Mallard.pdf, certificate error from this tool)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (Europe PMC record)
- Reference check: citation matches (authors, venue, volume, pages, year). The abstract supports "seven large plates ... and smaller plates whose areas follow a fractal distribution". The specific split near 10^7.6 km^2 was not in what I could read; see spot check SCL-030.

### Swinbank2006
- Reference: Swinbank, R., Purser, R. J., 2006. Fibonacci grids: A novel approach to global modelling. Quarterly Journal of the Royal Meteorological Society 132(619): 1769-1793.
- DOI or URL: 10.1256/qj.05.227
- Cited as: "Swinbank & Purser 2006 ('Fibonacci grids')"
- Ledger ids: SCL-043, SCL-044
- Access: not tested (Wiley/RMetS; likely paywalled)
- Read so far (by the project): "Implemented from the formula; no code was ported." Whether read is not stated
- Read in this pass: not fetched (Crossref metadata only)
- Reference check: exists; authors, title, journal, volume, issue, pages, year match. The code's short title "Fibonacci grids" omits the subtitle.

### Gonzalez2010
- Reference: Gonzalez, A., 2010. Measurement of areas on a sphere using Fibonacci and latitude-longitude lattices. Mathematical Geosciences 42(1): 49-64. (Online first 2009; Crossref lists the year as 2009, the print issue is January 2010.)
- DOI or URL: 10.1007/s11004-009-9257-x ; open copy https://arxiv.org/abs/0912.4540
- Cited as: "Gonzalez 2010 ('Measurement of areas on a sphere using Fibonacci and latitude-longitude lattices')"
- Ledger ids: SCL-043, SCL-044
- Access: open copy at arxiv.org/abs/0912.4540
- Read so far (by the project): formula implemented; read status not stated
- Read in this pass: abstract fetched (arXiv); arXiv PDF came back binary and unreadable
- Reference check: title and journal match. Author's name is Alvaro Gonzalez (accent on A). Year 2010 is right for the issue; 2009 for the online DOI.

### Keinert2015
- Reference: Keinert, B., Innmann, M., Saenger, M., Stamminger, M., 2015. Spherical Fibonacci mapping. ACM Transactions on Graphics 34(6): article 193 (Crossref pages "1-7"; proceedings of SIGGRAPH Asia 2015).
- DOI or URL: 10.1145/2816795.2818131
- Cited as: "Keinert et al. 2015 ('Spherical Fibonacci mapping')"
- Ledger ids: SCL-043, SCL-044
- Access: paywalled (ACM); an author copy on lgdv.cs.fau.de was tried and returned 404
- Read so far (by the project): formula implemented; read status not stated
- Read in this pass: not fetched (Crossref metadata only)
- Reference check: title, journal, volume, issue, year match. Article number and page count of the printed version not confirmed.

### Kesler2008
- Reference: Kesler, S. E., Wilkinson, B. H., 2008. Earth's copper resources estimated from tectonic diffusion of porphyry copper deposits. Geology 36(3): 255-258.
- DOI or URL: 10.1130/G24317A.1 ; https://pubs.geoscienceworld.org/geology/article/36/3/255-258/29682 (403)
- Cited as: "Kesler & Wilkinson 2008, doi 10.1130/g24317a.1"
- Ledger ids: SCL-006
- Access: paywalled (abstract only)
- Read so far (by the project): not stated; DOI linked
- Read in this pass: abstract fetched (OpenAlex record; the abstract text is the tool's reconstruction from the index)
- Reference check: citation matches. What it is cited for ("deposits above 3.3 km are within mining reach") is NOT in the abstract; the abstract is about estimating the number of porphyry copper deposits formed (about 125,895 in the Phanerozoic), preserved (about 47,789), and total copper (about 1.7 x 10^11 t in preserved porphyry deposits, 3 x 10^11 t all resources). The 3.3 km figure may be in the body; unverified.

### Coltice2019
- Reference: Coltice, N., Husson, L., Faccenna, C., Arnould, M., 2019. What drives tectonic plates? Science Advances 5(10): eaax4295.
- DOI or URL: 10.1126/sciadv.aax4295 ; open access, Europe PMC PMC6821462
- Cited as: "Coltice (run used a Pangea-like start)" (no year or title in the report; the research note tectonics_engines.md names "Coltice et al. 2019, What drives tectonic plates?")
- Ledger ids: SCL-041
- Access: open access
- Read so far (by the project): not stated for this claim
- Read in this pass: full text fetched (Europe PMC XML, searched for "Pangea/supercontinent" only; title and authors confirmed)
- Reference check: identification inferred (the report gives only "Coltice"); the other repo Coltice citation (Coltice et al. 2007, Geology 35:391-394) is a different paper on mantle heating. Volume/issue/article number taken from the repo note, not re-confirmed by me.

### Gough1981
- Reference: Gough, D. O., 1981. Solar interior structure and luminosity variations. Solar Physics 74(1): 21-34. (Also appears as a chapter of the proceedings "Physics of Solar Variations", Reidel 1981, pp. 21-34, DOI 10.1007/978-94-010-9633-1_4.)
- DOI or URL: 10.1007/BF00151270 ; scanned PDF https://articles.adsabs.harvard.edu/pdf/1981SoPh...74...21G
- Cited as: "Gough 1981", "Solar interior structure and luminosity variations", "Solar Phys. 74: 21-34"
- Ledger ids: SCL-039
- Access: open copy (ADS page scan, image-only) at the ADS URL; publisher page paywalled
- Read so far (by the project): not stated; the comment gives the correct title and pages
- Read in this pass: not readable (ADS PDF is JBIG2 page images; the tool cannot read it; Springer redirects to a cookie handshake)
- Reference check: title, journal, volume, pages, year, single author all match Crossref. The luminosity law (equation and number) not confirmed.

### Bassin2000 (CRUST 2.0)
- Reference: Bassin, C., Laske, G., Masters, G., 2000. The current limits of resolution for surface wave tomography in North America. EOS Transactions AGU 81, F897 (Fall Meeting abstract). Model: CRUST 2.0, 2 x 2 degree global crustal model, 360 key one-dimensional profiles.
- DOI or URL: https://igppweb.ucsd.edu/~gabi/crust2.html ; data http://igpppublic.ucsd.edu/~gabi/ftp/crust2/ (CNtype2.txt, CNtype2_key.txt, CN-README)
- Cited as: "CRUST 2.0 crust types ... derived by a sub-agent" (the user's brief gives "Bassin, Laske & Masters 2000"; the repo cites only "CRUST 2.0")
- Ledger ids: none (only earth/SCL.md, province shares)
- Access: open (data files)
- Read so far (by the project): derived by a sub-agent; no method, file or URL
- Read in this pass: data fetched and processed (see Part 2, Earth shares)
- Reference check: the citation the model's own page asks for is a conference abstract, not a model paper. The paper-level reference for the predecessor CRUST 5.1 is Mooney2008 below. The 360 type names describe tectonic settings (Archean, early/mid Proterozoic, late Proterozoic, Platform, Phanerozoic, orogen, continental arc, extended crust, rift, shelf, slope, oceanic ...), so a province share needs a mapping from type name to class; none is published with the model.

### Mooney1998 (CRUST 5.1)
- Reference: Mooney, W. D., Laske, G., Masters, T. G., 1998. CRUST 5.1: A global crustal model at 5 x 5 degrees. Journal of Geophysical Research 103(B1): 727-747.
- DOI or URL: 10.1029/97JB02122
- Cited as: not cited by the code (predecessor of CRUST 2.0)
- Ledger ids: none
- Access: not tested (AGU; likely paywalled)
- Read so far (by the project): not cited
- Read in this pass: not fetched (Crossref metadata only)
- Reference check: listed because CRUST 2.0's own page points to it and no paper describes CRUST 2.0.

### NASA-EarthFactSheet
- Reference: Williams, D. R. (NASA Goddard Space Flight Center, NSSDC), Earth Fact Sheet. Page footer: "Last Updated: 15 November 2024, DRW". (Author initials from the page; full name not on the fetched text.)
- DOI or URL: https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html
- Cited as: none in the repo (supplied here as the citable source for Earth constants)
- Ledger ids: SCL-005, SCL-022 to SCL-027, SCL-029 (comparison)
- Access: open access
- Read so far (by the project): not cited
- Read in this pass: full text fetched
- Reference check: n/a. The sheet gives no surface area, land/ocean fraction or circumference.

### Peel2007 (Koppen-Geiger)
- Reference: Peel, M. C., Finlayson, B. L., McMahon, T. A., 2007. Updated world map of the Koppen-Geiger climate classification. Hydrology and Earth System Sciences 11: 1633-1644.
- DOI or URL: 10.5194/hess-11-1633-2007 ; https://hess.copernicus.org/articles/11/1633/2007/
- Cited as: not cited ("Koppen's dryness threshold (20 T + 140 mm)", no year)
- Ledger ids: BAK-020 (supporting source found in this pass)
- Access: open access
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (landing page); PDF returned as binary so Table 1 not read; criteria were taken from Wikipedia's Koppen article instead (secondary)
- Reference check: the code names no source. Peel et al. 2007 and Kottek et al. 2006 are the standard references per Wikipedia.

### UNEP1992 (aridity classes)
- Reference: UNEP, 1992. World Atlas of Desertification (Middleton, N., Thomas, D., eds.), United Nations Environment Programme / Arnold, London. Edition and publisher details not verified in this pass (Wikipedia names "UNEP (1992)").
- DOI or URL: none found
- Cited as: "UNEP aridity limits 0.2 and 0.5" (no reference)
- Ledger ids: BAK-020
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (Wikipedia "Aridity index" table)
- Reference check: values 0.05, 0.20, 0.50, 0.65 as P/PET limits confirmed in the Wikipedia table; the atlas itself not read.

### Koerner2004 (treeline)
- Reference: Koerner, C., Paulsen, J., 2004. A world-wide study of high altitude treeline temperatures. Journal of Biogeography 31(5): 713-732.
- DOI or URL: not looked up
- Cited as: not cited (supporting source found via Wikipedia "Tree line")
- Ledger ids: BAK-021
- Access: not tested
- Read so far (by the project): not cited
- Read in this pass: search result snippet only (Wikipedia: "seasonal mean temperature is approximately 6 degC")
- Reference check: citation as given on Wikipedia.

### Connelly2012 (age of the solar system)
- Reference: Connelly, J. N., Bizzarro, M., Krot, A. N., Nordlund, A., Wielandt, D., Ivanova, M. A., 2012. The absolute chronology and thermal processing of solids in the solar protoplanetary disk. Science 338(6107): 651-655.
- DOI or URL: 10.1126/science.1226919
- Cited as: not cited (supporting source for 4.57 Gyr)
- Ledger ids: SCL-039
- Access: not tested
- Read so far (by the project): not cited
- Read in this pass: not fetched (Crossref metadata only; the 4.5673 Ga CAI age was seen on Wikipedia "Age of Earth", which does not name this paper in what I fetched, so the link to Connelly is my inference)
- Reference check: n/a

### Not bibliographic
Toroidal World (MIT) docs, Immersive Portals source, TFC Eratosthenes, Civilization V/VI polar caps, RPG-Maker and GameDev forum threads, EU4/Civ IV/Starbound/KSP/Space Engineers/Seed of Andromeda, Sable 2.0.5 API, Distant Horizons API, vanilla Minecraft physics constants and `Biome.coldEnoughToSnow`, the design-decisions file and decisions log, the implementation plan, the research report and notes, build-log entries, Tectonic mod's y640 ceiling.

## Plate tectonics (PLT)

### Cortial2019
- Reference: Cortial, Y., Peytavie, A., Galin, E., Guérin, E. (2019). Procedural Tectonic Planets. Computer Graphics Forum (Eurographics 2019 proceedings) 38(2):1-11.
- DOI or URL: 10.1111/cgf.13614; open copies https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf and https://hal.science/hal-02136820v1/file/2019-Procedural-Tectonic-Planets.pdf (HAL blocked WebFetch with an Anubis page; the LIRIS copy downloaded fine with curl).
- Cited as: "Cortial, Peytavie, Galin & Guerin 2019", "Cortial Appendix A, Table 3", "Table 2", sections 3, 4.1-4.5, 6, 7.5.
- Ledger ids: 001, 002, 019, 020, 021, 022, 028, 029, 034, 036-041, 052, 055, 056, 068, 070-073, 081-085, 092-093, 095, 098, 100-101, 119, 126-130 (from the sources file).
- Access: open copy at the LIRIS URL above.
- Read so far (by the project): in full per P1-03 log ("implemented from Cortial"), no statement of constant-by-constant verification.
- Read in this pass: full text fetched (every section, Table 1, Table 2, Table 3 in Appendix A, section 6, 7.4, 7.5).
- Reference check: authors, year, title, journal, volume(issue) correct. Pages 1-11 (Crossref). Table numbering in the paper: Table 1 = parametrization of the crust, Table 2 = timings, Table 3 = Appendix A constants. So params.rs "Appendix A, Table 3", the research note's "Table 1 for per-sample parameters" and "Table 2 for timings" are all right (this settles earth-file flag 24). Departures from the paper are listed in Part 2.

### ForsythUyeda1975
- Reference: Forsyth, D., Uyeda, S. (1975). On the relative importance of the driving forces of plate motion. Geophysical Journal of the Royal Astronomical Society 43(1):163-200.
- DOI or URL: 10.1111/j.1365-246X.1975.tb00631.x; https://academic.oup.com/gji/article/43/1/163/586101 (abstract page; PDF returns 403).
- Cited as: "Forsyth & Uyeda 1975, Geophys. J. R. Astron. Soc. 43:163-200".
- Ledger ids: 023, 030, 162, 182, 185, 189, 199.
- Access: paywalled (abstract only).
- Read so far (by the project): abstract only / secondary pages.
- Read in this pass: abstract fetched (publisher page).
- Reference check: citation correct. The journal title "Geophysical Journal of the Royal Astronomical Society" is the 1975 name (now Geophysical Journal International). The abstract does NOT contain the "3 to 4" speed ratio or any factor of 8 (see Part 2); those must come from the body or from another paper.

### SteinStein1992
- Reference: Stein, C. A., Stein, S. (1992). A model for the global variation in oceanic depth and heat flow with lithospheric age. Nature 359(6391):123-129.
- DOI or URL: 10.1038/359123a0 (Nature page redirects to a login wall).
- Cited as: "Stein & Stein 1992, Nature 359:123-129 (GDH1)".
- Ledger ids: 094, 095, 096, 107, 204.
- Access: paywalled (closed; no abstract in OpenAlex or Semantic Scholar).
- Read so far (by the project): not stated; formulas appear in a test comment.
- Read in this pass: metadata only (Crossref, OpenAlex, Semantic Scholar). Equations not read.
- Reference check: citation correct (authors Carol A. Stein and Seth Stein). The depth equations in the test comment could not be confirmed against the paper.

### ParsonsSclater1977
- Reference: Parsons, B., Sclater, J. G. (1977). An analysis of the variation of ocean floor bathymetry and heat flow with age. Journal of Geophysical Research 82(5):803-827.
- DOI or URL: 10.1029/JB082i005p00803.
- Cited as: "Parsons & Sclater 1977" (ridge crest depth 2.5 km); no title or journal given in the code.
- Ledger ids: 092, 204.
- Access: paywalled (no abstract fetched).
- Read so far (by the project): not stated.
- Read in this pass: not fetched itself; its formulas were read as reproduced in King & Adam 2014 (a secondary open copy) and Wikipedia "Seafloor spreading".
- Reference check: now identifiable. Identification only; the "2.5 km" is the intercept of the square-root law (see Part 2).

### Cogley1984
- Reference: Cogley, J. G. (1984). Continental margins and the extent and number of the continents. Reviews of Geophysics 22(2):101-122.
- DOI or URL: 10.1029/RG022I002P00101.
- Cited as: "Cogley 1984, Rev. Geophys. 22:101-122" (29 percent land on 41 percent continental crust).
- Ledger ids: 010, 158, 159, 205.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched (Crossref JSON, full abstract).
- Reference check: citation correct (issue 2). The abstract gives 41 percent for continental crust but not "29 percent land"; it gives 30.6 percent of continental area submerged, from which land is about 28.5 percent of the planet (arithmetic on the abstract's numbers, not a stated value).

### Shepard1963
- Reference: Shepard, F. P. (1963). Submarine Geology, 2nd ed. Harper & Row, New York.
- DOI or URL: none found.
- Cited as: "Shepard 1963, Submarine Geology" (shelf break about 130 m).
- Ledger ids: 011.
- Access: book (not online).
- Read so far (by the project): not stated.
- Read in this pass: not fetched.
- Reference check: edition and publisher are from memory of the book, not verified in this pass; the code cites no edition.

### Shoemake1992
- Reference: Shoemake, K. (1992). Uniform random rotations. In: D. Kirk (ed.), Graphics Gems III, Academic Press, pp. 124-132.
- DOI or URL: 10.1016/B978-0-08-050755-2.50036-1.
- Cited as: "Shoemake 1992, Uniform random rotations".
- Ledger ids: 209.
- Access: book (not online) chapter; Crossref confirms pages 124-132.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (metadata only).
- Reference check: correct. Editor name not checked.

### Stern2004
- Reference: Stern, R. J. (2004). Subduction initiation: spontaneous and induced. Earth and Planetary Science Letters 226(3-4):275-292.
- DOI or URL: 10.1016/S0012-821X(04)00498-4 (OpenAlex lists 10.1016/j.epsl.2004.08.007).
- Cited as: "Stern 2004, EPSL 226:275".
- Ledger ids: 062, 067, 207.
- Access: paywalled (no abstract available in OpenAlex).
- Read so far (by the project): not stated.
- Read in this pass: metadata only.
- Reference check: correct.

### SternGerya2018
- Reference: Stern, R. J., Gerya, T. (2018). Subduction initiation in nature and models: a review. Tectonophysics 746:173-198.
- DOI or URL: 10.1016/j.tecto.2017.10.014.
- Cited as: "Stern & Gerya 2018, Tectonophysics 746:173".
- Ledger ids: 067.
- Access: paywalled (not tested for an open copy).
- Read so far (by the project): not stated.
- Read in this pass: metadata only.
- Reference check: correct.

### Stern2002
- Reference: Stern, R. J. (2002). Subduction zones. Reviews of Geophysics 40(4):1012.
- DOI or URL: 10.1029/2001RG000108.
- Cited as: "Stern 2002" (intra-oceanic arcs take tens of Myr to build 20-35 km of crust); no title or journal in the code.
- Ledger ids: 047.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched.
- Reference check: identified by the claim (arc building). The abstract says "Earth's 55,000 km of convergent plate margins" but does not give "tens of Myr" or "20-35 km"; those would be in the body (not read).

### Bradley2008
- Reference: Bradley, D. C. (2008). Passive margins through earth history. Earth-Science Reviews 91(1-4):1-26.
- DOI or URL: 10.1016/j.earscirev.2008.08.001.
- Cited as: "Bradley 2008, Earth-Sci. Rev. 91:1-26".
- Ledger ids: 160, 167, 207.
- Access: paywalled (abstract not obtainable here).
- Read so far (by the project): abstract or secondary pages.
- Read in this pass: metadata only (OpenAlex abstract field is null; ScienceDirect 403).
- Reference check: correct (author affiliation USGS Anchorage). The figures 105,000 km passive / 53,000 km convergent and 104/181 Myr ages could not be checked.

### Lallemand_EncMarineGeosciences
- Reference: Lallemand, S. entry in Encyclopedia of Marine Geosciences (Springer). Year and entry title not given in the code and not resolved.
- DOI or URL: not found.
- Cited as: "Lallemand, Encycl. Marine Geosciences" (about 45,000 km active continental margins).
- Ledger ids: 160.
- Access: not found.
- Read so far (by the project): not stated.
- Read in this pass: not fetched.
- Reference check: UNIDENTIFIED as to year and entry title (search budget was gone). Needs one search in stage 2.

### Schellart2008
- Reference: Schellart, W. P., Stegman, D. R., Freeman, J. (2008). Global trench migration velocities and slab migration induced upper mantle volume fluxes: constraints to find an Earth reference frame based on minimizing viscous dissipation. Earth-Science Reviews 88(1-2):118-144.
- DOI or URL: 10.1016/j.earscirev.2008.01.005.
- Cited as: "Schellart et al. 2008, Earth-Sci. Rev. 88:118, Table 1" (28,000-32,000 km "tallied from").
- Ledger ids: 160.
- Access: paywalled.
- Read so far (by the project): tallied by an agent from Table 1.
- Read in this pass: metadata only.
- Reference check: correct journal and page. The paper is about trench migration; that it has a table of margin lengths is plausible but not checked.

### MercierdeLepinay2016
- Reference: Mercier de Lépinay, M., Loncke, L., Basile, C., Roest, W. R., Patriat, M., Maillard, A., De Clarens, P. (2016). Transform continental margins - Part 2: a worldwide review. Tectonophysics 693:96-115.
- DOI or URL: 10.1016/j.tecto.2016.05.038.
- Cited as: "Mercier de Lepinay et al. 2016, Tectonophysics 693" (convergent about 48 percent of margin length).
- Ledger ids: 160.
- Access: paywalled.
- Read so far (by the project): abstract or secondary.
- Read in this pass: metadata only.
- Reference check: identified by title search. It is a paper on TRANSFORM margins; a global margin-type length inventory may be inside it but the paper's focus is not convergent margins. Flag for stage 2.

### Tozer2019
- Reference: Tozer, B., Sandwell, D. T., Smith, W. H. F., Olson, C., Beale, J. R., Wessel, P. (2019). Global bathymetry and topography at 15 arc sec: SRTM15+. Earth and Space Science 6(10):1847-1864.
- DOI or URL: 10.1029/2019EA000658 (open access in Wiley).
- Cited as: "Tozer et al. 2019 (SRTM15+, GMT earth_relief 30 arc-minute)".
- Ledger ids: 168-170.
- Access: open access (Wiley/AGU).
- Read so far (by the project): numbers computed by the project from the GMT grid.
- Read in this pass: abstract fetched. The abstract describes the dataset; it contains no land-fraction or hypsometry numbers.
- Reference check: correct. The project's Earth numbers (land 0.289, median 427 m) are its own computation and cannot be checked against this paper; they depend on the 30 arc-minute resampling.

### CharetteSmith2010
- Reference: Charette, M. A., Smith, W. H. F. (2010). The volume of Earth's ocean. Oceanography 23(2):112-114.
- DOI or URL: 10.5670/oceanog.2010.51.
- Cited as: "Charette & Smith 2010, Oceanography 23(2):112-114" (sea mean 3,682 m; 361.84 million km2 = 70.9 percent).
- Ledger ids: 097, 169-170.
- Access: open access (Oceanography Society).
- Read so far (by the project): not stated.
- Read in this pass: metadata only (OpenAlex abstract is only a copyright line).
- Reference check: correct. Numbers not read.

### ClarkRoyden2000
- Reference: Clark, M. K., Royden, L. H. (2000). Topographic ooze: building the eastern margin of Tibet by lower crustal flow. Geology 28(8):703-706.
- DOI or URL: 10.1130/0091-7613(2000)28<703:TOBTEM>2.0.CO;2.
- Cited as: "Clark & Royden 2000, Geology 28:703".
- Ledger ids: 123, 125.
- Access: paywalled (GSA; not tested).
- Read so far (by the project): not stated; not in code.
- Read in this pass: metadata only.
- Reference check: correct.

### EnglandMcKenzie1982
- Reference: England, P., McKenzie, D. (1982). A thin viscous sheet model for continental deformation. Geophysical Journal of the Royal Astronomical Society 70(2):295-321.
- DOI or URL: 10.1111/j.1365-246X.1982.tb04969.x.
- Cited as: "England & McKenzie 1982 (GJRAS 70:295)".
- Ledger ids: 123.
- Access: paywalled.
- Read so far (by the project): not stated; not in code.
- Read in this pass: metadata only.
- Reference check: correct. A correction exists (GJRAS 73:523-532, 1983).

### MolnarLyonCaen1988
- Reference: Molnar, P., Lyon-Caen, H. (1988). Some simple physical aspects of the support, structure, and evolution of mountain belts. In: Clark, S. P., Burchfiel, B. C., Suppe, J. (eds.), Processes in Continental Lithospheric Deformation, Geological Society of America Special Paper 218, pp. 179-207 (OpenAlex gives 179-208).
- DOI or URL: 10.1130/SPE218-p179.
- Cited as: "Molnar & Lyon-Caen 1988 (GSA Spec. Pap. 218)", "pages 179-207" in the force-balance design (Tibet 6-7e12 N/m).
- Ledger ids: 123, 187.
- Access: book chapter (abstract visible via OpenAlex; full text not online).
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched.
- Reference check: title and page start correct; the editors and the "6-7e12 N/m" figure are not in the abstract. The abstract does state that more gravitational potential energy is stored in a column with a thick crustal root and that ranges reach a maximum mean elevation; no number.

### Bird1991
- Reference: Bird, P. (1991). Lateral extrusion of lower crust from under high topography in the isostatic limit. Journal of Geophysical Research 96(B6):10275-10286.
- DOI or URL: 10.1029/91JB00370.
- Cited as: "Bird 1991 (JGR 96:10275)".
- Ledger ids: 123.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated; not in code.
- Read in this pass: abstract fetched (Crossref). The abstract states that flux is proportional to the third power of the topographic gradient (cubic power law creep), which matches the project's "channel law cube".
- Reference check: correct.

### Royden1997
- Reference: Royden, L. H., Burchfiel, B. C., King, R. W., Wang, E., Chen, Z., Shen, F., Liu, Y. (1997). Surface deformation and lower crustal flow in eastern Tibet. Science 276(5313):788-790.
- DOI or URL: 10.1126/science.276.5313.788.
- Cited as: "Royden et al. 1997".
- Ledger ids: 123.
- Access: paywalled (abstract only).
- Read so far (by the project): not stated; not in code.
- Read in this pass: abstract fetched.
- Reference check: identifiable; correct.

### Rey2001
- Reference: Rey, P., Vanderhaeghe, O., Teyssier, C. (2001). Gravitational collapse of the continental crust: definition, regimes and modes. Tectonophysics 342(3-4):435-449.
- DOI or URL: 10.1016/S0040-1951(01)00174-3.
- Cited as: "Rey, Vanderhaeghe & Teyssier 2001 (Tectonophysics 342:435)".
- Ledger ids: 123.
- Access: paywalled.
- Read so far (by the project): not stated; not in code.
- Read in this pass: metadata only.
- Reference check: correct.

### Beaumont2001
- Reference: Beaumont, C., Jamieson, R. A., Nguyen, M. H., Lee, B. (2001). Himalayan tectonics explained by extrusion of a low-viscosity crustal channel coupled to focused surface denudation. Nature 414(6865):738-742.
- DOI or URL: 10.1038/414738a.
- Cited as: "Beaumont et al. 2001 (Nature 414:738)".
- Ledger ids: 123.
- Access: paywalled.
- Read so far (by the project): not stated; not in code.
- Read in this pass: metadata only.
- Reference check: correct.

### Bird2003
- Reference: Bird, P. (2003). An updated digital model of plate boundaries. Geochemistry, Geophysics, Geosystems 4(3):1027.
- DOI or URL: 10.1029/2001GC000252 (open access at AGU/Wiley; blocked to WebFetch with 403).
- Cited as: "Bird 2003, G3 4(3):1027" (52 plates; Pacific 103.3 million km2; seven largest about 92 percent); editor hint "Earth: about 14".
- Ledger ids: 143, 145, 203.
- Access: open access (G3 is open), not fetched as full text here.
- Read so far (by the project): "confirmed from abstract or secondary pages"; areas from Wikipedia.
- Read in this pass: abstract fetched (OpenAlex reconstruction). 
- Reference check: correct. The abstract gives "14 large plates ... plus 38 small plates ... for a total of 52 plates". The area numbers and "seven largest 92 percent" are not in the abstract.

### Morra2013
- Reference: Morra, G., Seton, M., Quevedo, L., Müller, R. D. (2013). Organization of the tectonic plates in the last 200 Myr. Earth and Planetary Science Letters 373:93-101.
- DOI or URL: 10.1016/j.epsl.2013.04.020.
- Cited as: "Morra et al. 2013, EPSL 373:93-101".
- Ledger ids: 145.
- Access: paywalled.
- Read so far (by the project): abstract or secondary.
- Read in this pass: metadata only.
- Reference check: correct.

### Mallard2016
- Reference: Mallard, C., Coltice, N., Seton, M., Müller, R. D., Tackley, P. J. (2016). Subduction controls the distribution and fragmentation of Earth's tectonic plates. Nature 535(7610):140-143.
- DOI or URL: 10.1038/nature17992.
- Cited as: "Mallard et al. 2016, Nature 535:140-143".
- Ledger ids: 145.
- Access: paywalled in Nature (a Caltech-hosted copy was used by the research note; not located here).
- Read so far (by the project): research note Q2 says the PDF was read from web.gps.caltech.edu.
- Read in this pass: metadata only.
- Reference check: correct.

### Lonsdale2005
- Reference: Lonsdale, P. (2005). Creation of the Cocos and Nazca plates by fission of the Farallon plate. Tectonophysics 404(3-4):237-264.
- DOI or URL: 10.1016/j.tecto.2005.05.011.
- Cited as: "Lonsdale 2005, Tectonophysics 404:237-264".
- Ledger ids: 146.
- Access: paywalled.
- Read so far (by the project): not stated.
- Read in this pass: metadata only.
- Reference check: correct; the design cites it for Farallon and not for lifetime statistics (the paper is about a plate fission event).

### Wilson1966
- Reference: Wilson, J. T. (1966). Did the Atlantic close and then re-open? Nature 211(5050):676-681.
- DOI or URL: 10.1038/211676a0.
- Cited as: "J. T. Wilson 1966, Nature 211:676-681".
- Ledger ids: 148.
- Access: paywalled.
- Read so far (by the project): abstract or secondary.
- Read in this pass: metadata only.
- Reference check: correct.

### Nance2022
- Reference: Nance, R. D. (2022). The supercontinent cycle and Earth's long-term climate. Annals of the New York Academy of Sciences 1515(1):33-49.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC9796656/ (open access).
- Cited as: "Nance 2022, PMC9796656".
- Ledger ids: 148, 201.
- Access: open access.
- Read so far (by the project): abstract-only status in the ledger.
- Read in this pass: full text fetched (targeted at cycle length and supercontinent dates).
- Reference check: title/author/journal not named in the code; now identified.

### Mitchell2021
- Reference: Mitchell, R. N., Zhang, N., Salminen, J., Liu, Y., Spencer, C. J., Steinberger, B., Murphy, J. B., Li, Z.-X. (2021). The supercontinent cycle. Nature Reviews Earth & Environment 2(5):358-374.
- DOI or URL: 10.1038/s43017-021-00160-0.
- Cited as: "Mitchell et al. 2021, Nat. Rev. Earth Environ. 2:358-374".
- Ledger ids: 148, 201.
- Access: paywalled.
- Read so far (by the project): abstract-only status.
- Read in this pass: metadata only.
- Reference check: correct.

### Copley2010
- Reference: Copley, A., Avouac, J.-P., Royer, J.-Y. (2010). India-Asia collision and the Cenozoic slowdown of the Indian plate: implications for the forces driving plate motions. Journal of Geophysical Research 115(B3):B03410.
- DOI or URL: 10.1029/2009JB006634.
- Cited as: "Copley, Avouac & Royer 2010, JGR 115:B03410".
- Ledger ids: 154, 177, 200.
- Access: paywalled (abstract only; Wiley).
- Read so far (by the project): not stated beyond section 3 remark.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: correct, and the abstract also contains the force magnitudes used elsewhere in the designs (Part 2).

### DaviesvonBlanckenburg1995
- Reference: Davies, J. H., von Blanckenburg, F. (1995). Slab breakoff: a model of lithosphere detachment and its test in the magmatism and deformation of collisional orogens. Earth and Planetary Science Letters 129(1-4):85-102.
- DOI or URL: 10.1016/0012-821X(94)00237-S.
- Cited as: "Davies & von Blanckenburg 1995 (EPSL 129:85-102)".
- Ledger ids: 179, 196.
- Access: paywalled.
- Read so far (by the project): contextual.
- Read in this pass: metadata only.
- Reference check: correct.

### RoyerGordon1997
- Reference: Royer, J.-Y., Gordon, R. G. (1997). The motion and boundary between the Capricorn and Australian plates. Science 277(5330):1268-1274.
- DOI or URL: 10.1126/science.277.5330.1268.
- Cited as: "Royer & Gordon 1997".
- Ledger ids: 179, 196.
- Access: paywalled (abstract).
- Read so far (by the project): contextual.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct. Its topic (diffuse plate boundaries in the Indo-Australian plate) is an odd match for "supercontinent heating" context; the design may mean it as the example of plate splitting. Flag for the stage 2 reader.

### Gurnis1988
- Reference: Gurnis, M. (1988). Large-scale mantle convection and the aggregation and dispersal of supercontinents. Nature 332(6166):695-699.
- DOI or URL: 10.1038/332695a0.
- Cited as: "Gurnis 1988 (Nature 332:695-699)".
- Ledger ids: 179, 196.
- Access: paywalled.
- Read so far (by the project): contextual.
- Read in this pass: metadata only.
- Reference check: correct.

### Coltice2007
- Reference: Coltice, N., Phillips, B. R., Bertrand, H., Ricard, Y., Rey, P. (2007). Global warming of the mantle at the origin of flood basalts over supercontinents. Geology 35(5):391-394.
- DOI or URL: 10.1130/G23240A.1.
- Cited as: "Coltice et al. 2007 (Geology 35:391-394)".
- Ledger ids: 179.
- Access: paywalled.
- Read so far (by the project): abstract-only status.
- Read in this pass: metadata only.
- Reference check: correct.

### HeronLowman2011
- Reference: Heron, P. J., Lowman, J. P. (2011). The effects of supercontinent size and thermal insulation on the formation of mantle plumes. Tectonophysics 510(1-2):28-38.
- DOI or URL: 10.1016/j.tecto.2011.07.002.
- Cited as: "Heron & Lowman 2011".
- Ledger ids: 179.
- Access: paywalled.
- Read so far (by the project): abstract-only status.
- Read in this pass: metadata only.
- Reference check: identified by title search; correct.

### Lenardic2011
- Reference: Lenardic, A., Moresi, L., Jellinek, A. M., O'Neill, C. J., Cooper, C. M., Lee, C. T. (2011). Continents, supercontinents, mantle thermal mixing, and mantle thermal isolation: theory, numerical simulations, and laboratory experiments. Geochemistry, Geophysics, Geosystems 12(10):Q10016 (article number from memory of the journal; the Crossref record gave only the issue).
- DOI or URL: 10.1029/2011GC003663.
- Cited as: "Lenardic et al. 2011" (no venue in the code).
- Ledger ids: 179.
- Access: paywalled (not tested).
- Read so far (by the project): abstract-only status.
- Read in this pass: metadata only.
- Reference check: identified by author list and year; there is also Lenardic et al. 2005 (EPSL 234:317) on continental insulation, which is the other candidate.

### BercoviciLong2014
- Reference: Bercovici, D., Long, M. D. (2014). Slab rollback instability and supercontinent dispersal. Geophysical Research Letters 41(19):6659-6666.
- DOI or URL: 10.1002/2014GL061251.
- Cited as: "Bercovici & Long 2014 (GRL 41:6659-6666)".
- Ledger ids: 179, 196.
- Access: paywalled (abstract; AGU).
- Read so far (by the project): abstract-only status.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct; title given here for the first time (the code gives none).

### Nocquet2016
- Reference: Nocquet, J.-M., Sue, C., Walpersdorf, A., Tran, T., Lenôtre, N., Vernant, P., Cushing, M., Jouanne, F., Masson, F., Baize, S., Chéry, J., van der Beek, P. A. (2016). Present-day uplift of the western Alps. Scientific Reports 6:28404.
- DOI or URL: 10.1038/srep28404 (open access).
- Cited as: "Nocquet et al. 2016, Sci. Rep. 6:28404".
- Ledger ids: 180.
- Access: open access.
- Read so far (by the project): abstract-only status.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct.

### Tremblay2015
- Reference: Tremblay, M. M., Fox, M., Schmidt, J. L., Tripathy-Lang, A., Wielicki, M. M., Harrison, T. M., Zeitler, P. K., Shuster, D. L. (2015). Erosion in southern Tibet shut down at ~10 Ma due to enhanced rock uplift within the Himalaya. Proceedings of the National Academy of Sciences 112(39):12030-12035.
- DOI or URL: 10.1073/pnas.1515652112 (PNAS, open after 6 months).
- Cited as: "Tremblay et al. 2015, PNAS 112:12030".
- Ledger ids: 180.
- Access: open access (PNAS).
- Read so far (by the project): abstract-only status.
- Read in this pass: abstract fetched.
- Reference check: identified by volume and first page; the paper is about erosion rates falling, while the design cites it for "exhumation above 1 mm/yr at 17-12 Ma" (not verifiable from the abstract).

### Schellart2004
- Reference: Schellart, W. P. (2004). Quantifying the net slab pull force as a driving mechanism for plate tectonics. Geophysical Research Letters 31(7):L07611.
- DOI or URL: 10.1029/2004GL019528.
- Cited as: "Schellart 2004, GRL 31:L07611" ("abstract read in full").
- Ledger ids: 182.
- Access: paywalled (abstract only).
- Read so far (by the project): abstract in full.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: correct.

### ConradLB2002
- Reference: Conrad, C. P., Lithgow-Bertelloni, C. (2002). How mantle slabs drive plate tectonics. Science 298(5591):207-209.
- DOI or URL: 10.1126/science.1074161.
- Cited as: "Conrad & Lithgow-Bertelloni 2002, Science 298:207-209".
- Ledger ids: 186, 199.
- Access: paywalled (abstract).
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched.
- Reference check: correct.

### ConradLB2004
- Reference: Conrad, C. P., Lithgow-Bertelloni, C. (2004). The temporal evolution of plate driving forces: importance of "slab suction" versus "slab pull" during the Cenozoic. Journal of Geophysical Research 109(B10):B10407.
- DOI or URL: 10.1029/2004JB002991.
- Cited as: "Conrad & Lithgow-Bertelloni 2004, JGR 109:B10407".
- Ledger ids: 186, 199.
- Access: paywalled (abstract).
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched (OpenAlex; reconstruction partly garbled, key sentence legible).
- Reference check: correct. This abstract, not Forsyth & Uyeda, contains the "about 4 times faster" statement (Part 2).

### ConradLB2006
- Reference: Conrad, C. P., Lithgow-Bertelloni, C. (2006). Influence of continental roots and asthenosphere on plate-mantle coupling. Geophysical Research Letters 33(5):L05312.
- DOI or URL: 10.1029/2005GL025621 (bronze open access at Wiley; PDF returns 403 to scripts).
- Cited as: "Conrad & Lithgow-Bertelloni 2006, GRL 33:L05312".
- Ledger ids: 188.
- Access: open copy at Wiley (bronze), abstract read here.
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched.
- Reference check: correct.

### LithgowBertelloniRichards1998
- Reference: Lithgow-Bertelloni, C., Richards, M. A. (1998). The dynamics of Cenozoic and Mesozoic plate motions. Reviews of Geophysics 36(1):27-78.
- DOI or URL: 10.1029/97RG02282.
- Cited as: "Lithgow-Bertelloni & Richards 1998, Rev. Geophys. 36:27-78".
- Ledger ids: 187.
- Access: paywalled (abstract).
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched (Crossref JSON).
- Reference check: correct. (The 1995 GRL paper 22:1317-1320, same authors, has the same ">90 % slabs, <10 % lithospheric thickening" statement.)

### Carlson1983
- Reference: Carlson, R. L., Hilde, T. W. C., Uyeda, S. (1983). The driving mechanism of plate tectonics: relation to age of the lithosphere at trenches. Geophysical Research Letters 10(4):297-300.
- DOI or URL: 10.1029/GL010i004p00297.
- Cited as: "Carlson, Hilde & Uyeda 1983, GRL 10:297-300".
- Ledger ids: 182.
- Access: paywalled (abstract).
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct.

### ParsonsRichter1980
- Reference: Parsons, B., Richter, F. M. (1980). A relation between the driving force and geoid anomaly associated with mid-ocean ridges. Earth and Planetary Science Letters 51(2):445-450.
- DOI or URL: 10.1016/0012-821X(80)90223-X.
- Cited as: "Parsons & Richter 1980, EPSL 51:445-450".
- Ledger ids: 187.
- Access: paywalled (no abstract in OpenAlex).
- Read so far (by the project): from abstract.
- Read in this pass: metadata only.
- Reference check: title now identified; correct.

### vanHunenAllen2011
- Reference: van Hunen, J., Allen, M. B. (2011). Continental collision and slab break-off: a comparison of 3-D numerical models with observations. Earth and Planetary Science Letters 302(1-2):27-37.
- DOI or URL: 10.1016/j.epsl.2010.11.035.
- Cited as: "van Hunen & Allen 2011, EPSL 302:27-37" (10-25 Myr).
- Ledger ids: 184.
- Access: paywalled.
- Read so far (by the project): from abstract.
- Read in this pass: metadata only.
- Reference check: correct.

### Coltice2019
- Reference: Coltice, N., Husson, L., Faccenna, C., Arnould, M. (2019). What drives tectonic plates? Science Advances 5(10):eaax4295.
- DOI or URL: 10.1126/sciadv.aax4295; https://pmc.ncbi.nlm.nih.gov/articles/PMC6821462/ (open access).
- Cited as: "Coltice, Husson, Faccenna & Arnould 2019, Sci. Adv. 5:eaax4295".
- Ledger ids: 188.
- Access: open access.
- Read so far (by the project): link given.
- Read in this pass: metadata and one-line abstract only (not fetched in full).
- Reference check: correct.

### Zahirovic2015
- Reference: Zahirovic, S., Müller, R. D., Seton, M., Flament, N. (2015). Tectonic speed limits from plate kinematic reconstructions. Earth and Planetary Science Letters 418:40-52.
- DOI or URL: 10.1016/j.epsl.2015.02.037; the EarthByte summary page https://www.earthbyte.org/tectonic-speed-limits-from-plate-kinematic-reconstructions/ returned HTTP 500 / timeouts in this pass.
- Cited as: "Zahirovic et al. 2015, EPSL 418:40-52".
- Ledger ids: 025, 198.
- Access: paywalled (closed; OpenAlex has no open URL and no abstract).
- Read so far (by the project): link given.
- Read in this pass: metadata only (Crossref). Speed numbers could not be read.
- Reference check: correct.

### DeCellesGiles1996
- Reference: DeCelles, P. G., Giles, K. A. (1996). Foreland basin systems. Basin Research 8(2):105-123.
- DOI or URL: 10.1046/j.1365-2117.1996.01491.x.
- Cited as: "DeCelles & Giles 1996, Basin Research 8:105-123" ("via a secondary source").
- Ledger ids: 194.
- Access: paywalled (abstract visible).
- Read so far (by the project): via a secondary source.
- Read in this pass: abstract fetched (Crossref, first part).
- Reference check: correct. The visible part of the abstract defines depozones; the 100-300 km and 2-8 km figures were not seen.

### TurcotteSchubert_Geodynamics
- Reference: Turcotte, D. L., Schubert, G. Geodynamics. Cambridge University Press (editions 1982, 2002, 2014).
- DOI or URL: none; Internet Archive copy (geodynamics0000turc) is lending-only (HTTP 401).
- Cited as: "Turcotte & Schubert, Geodynamics" (flexure; gross slab pull "not confirmed").
- Ledger ids: 183, 193; ISO-040, ISO-041, ISO-042, ISO-067.
- Access: book (not online).
- Read so far (by the project): not read; gross slab pull "not confirmed in Turcotte & Schubert itself". For the flexure feature (2026-10-02) a sub-agent read chapter 3 as a PDF (https://sseh.uchicago.edu/doc/Geodynamics_Turcotte_Schubert_ch_3.pdf: E = 70 GPa and ν = 0.25 in the periodic-loading example, 100 GPa elsewhere) and the flexure formulas in lecture notes on the chapter (https://topex.ucsd.edu/geodynamics/12flexure.pdf); the edition of that PDF was not recorded.
- Read in this pass: not fetched; a 2024 paper that cites it was read (see Li2024).
- Reference check: edition not stated anywhere in the code or designs; the numbers depend on the edition and chapter.

### Buck2004
- Reference: Buck, W. R. (2004). Consequences of asthenospheric variability on continental rifting. In: Karner, G. D., Taylor, B., Driscoll, N. W., Kohlstedt, D. L. (eds.), Rheology and Deformation of the Lithosphere at Continental Margins, Columbia University Press, pp. 1-30.
- DOI or URL: 10.7312/karn12738-002.
- Cited as: "Buck 2004 (in Karner et al., eds., ..., 1-30)".
- Ledger ids: 196.
- Access: book chapter (not online).
- Read so far (by the project): from abstracts.
- Read in this pass: metadata only.
- Reference check: correct; title identified here.

### Bialas2010
- Reference: Bialas, R. W., Buck, W. R., Qin, R. (2010). How much magma is required to rift a continent? Earth and Planetary Science Letters 292(1-2):68-78.
- DOI or URL: 10.1016/j.epsl.2010.01.021.
- Cited as: "Bialas, Buck & Qin 2010 (EPSL, volume not confirmed)".
- Ledger ids: 196.
- Access: paywalled.
- Read so far (by the project): from abstracts.
- Read in this pass: metadata only.
- Reference check: volume now resolved (292:68-78).

### DalZilio2018
- Reference: Dal Zilio, L., Faccenda, M., Capitanio, F. (2018). The role of deep subduction in supercontinent breakup. Tectonophysics 746:312-324.
- DOI or URL: 10.1016/j.tecto.2017.03.006.
- Cited as: "Dal Zilio, Faccenda & Capitanio 2018 (Tectonophysics 746:312-324)".
- Ledger ids: 196.
- Access: paywalled.
- Read so far (by the project): from abstracts.
- Read in this pass: metadata only.
- Reference check: correct.

### Zhang2018
- Reference: Zhang, N., Dang, Z., Huang, C., Li, Z.-X. (2018). The dominant driving force for supercontinent breakup: plume push or subduction retreat? Geoscience Frontiers 9(4):997-1007.
- DOI or URL: 10.1016/j.gsf.2018.01.010.
- Cited as: "Zhang et al. 2018 (Geosci. Front. 9:997-1007)".
- Ledger ids: 196.
- Access: open access (Geoscience Frontiers is open; not fetched).
- Read so far (by the project): from abstracts.
- Read in this pass: metadata only.
- Reference check: correct.

### Huang2019
- Reference: Huang, C., Zhang, N., Li, Z.-X., Ding, M., Dang, Z., Pourteau, A., Zhong, S. (2019). Modeling the inception of supercontinent breakup: stress state and the importance of orogens. Geochemistry, Geophysics, Geosystems 20(11):4830-4848.
- DOI or URL: 10.1029/2019GC008538; https://par.nsf.gov/biblio/10127684 (open).
- Cited as: "Huang et al. 2019 (G3 20:4830-4848)".
- Ledger ids: 196.
- Access: open copy at par.nsf.gov.
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct. The abstract does give "extensional stress of 20-50 MPa in its interior" for a homogeneous supercontinent, matching the design.

### DavisSuppeDahlen1983
- Reference: Davis, D., Suppe, J., Dahlen, F. A. (1983). Mechanics of fold-and-thrust belts and accretionary wedges. Journal of Geophysical Research 88(B2):1153-1172.
- DOI or URL: 10.1029/JB088iB02p01153.
- Cited as: "Davis, Suppe & Dahlen 1983, JGR 88:1153-1172" (from memory).
- Ledger ids: 192.
- Access: paywalled (abstract only).
- Read so far (by the project): from memory, not checked.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: the citation is correct (authors, year, volume, pages; issue B2). The paper's title was missing in the design. The abstract concerns critical taper and gives no taper angle in degrees.

### Christensen_Mooney1995
- Reference: Christensen, N. I., Mooney, W. D. (1995). Seismic velocity structure and composition of the continental crust: a global view. Journal of Geophysical Research 100(B6):9761-9788.
- DOI or URL: 10.1029/95JB00259.
- Cited as: not cited by the project; used here as a source for continental crust thickness and density.
- Ledger ids: (none; supports 121, 122, 131).
- Access: paywalled (abstract only).
- Read so far (by the project): not read.
- Read in this pass: abstract fetched.
- Reference check: n/a.

### Seton2020
- Reference: Seton, M., Müller, R. D., Zahirovic, S., Williams, S., Wright, N. M., Cannon, J., Whittaker, J. M., Matthews, K. J., McGirr, R. (2020). A global data set of present-day oceanic crustal age and seafloor spreading parameters. Geochemistry, Geophysics, Geosystems 21(10):e2020GC009214.
- DOI or URL: 10.1029/2020GC009214 (open access).
- Cited as: not cited by the project; the earth file's "mean 64 Myr" for sea-floor age needs a source.
- Ledger ids: (supports 201 and the run review).
- Access: open access.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (Crossref, full).
- Reference check: n/a.

### SyracuseAbers2006
- Reference: Syracuse, E. M., Abers, G. A. (2006). Global compilation of variations in slab depth beneath arc volcanoes and implications. Geochemistry, Geophysics, Geosystems 7(5):Q05017.
- DOI or URL: 10.1029/2005GC001045.
- Cited as: not cited; candidate source for arc-slab depth (flag 10; PLT-042).
- Ledger ids: (supports 042, 061, 164).
- Access: open (G3).
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (Crossref, full).
- Reference check: n/a.

### Lallemand2005
- Reference: Lallemand, S., Heuret, A., Boutelier, D. (2005). On the relationships between slab dip, back-arc stress, upper plate absolute motion, and crustal nature in subduction zones. Geochemistry, Geophysics, Geosystems 6(9):Q09006.
- DOI or URL: 10.1029/2005GC000917.
- Cited as: not cited; candidate for slab dip (the code has no dip; PLT-061).
- Ledger ids: (supports 061).
- Access: open (G3).
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (Crossref, full).
- Reference check: n/a. The abstract gives dip relations (dips differ by about 20 degrees between oceanic and continental upper plates) but no global mean dip.

### Courtillot2003
- Reference: Courtillot, V., Davaille, A., Besse, J., Stock, J. (2003). Three distinct types of hotspots in the Earth's mantle. Earth and Planetary Science Letters 205(3-4):295-308.
- DOI or URL: 10.1016/S0012-821X(02)01048-8.
- Cited as: named in the task (hotspot count); not cited in the code.
- Ledger ids: (supports 108-110).
- Access: paywalled (no abstract in Crossref/OpenAlex).
- Read so far (by the project): not read.
- Read in this pass: metadata only. The number of hotspots they considered (49) was read in King & Adam 2014; "seven primary hotspots" was read on Wikipedia "Hotspot (geology)" (secondary).
- Reference check: correct.

### KingAdam2014
- Reference: King, S. D., Adam, C. (2014). Hotspot swells revisited. Physics of the Earth and Planetary Interiors 235:66-83 (volume from memory; the PDF header was not captured).
- DOI or URL: 10.1016/j.pepi.2014.07.006; open copy http://hdl.handle.net/10919/103026 (Virginia Tech).
- Cited as: not cited by the project; used here as an open, readable reproduction of the Parsons & Sclater (1977) law and of hotspot compilations.
- Ledger ids: (supports 092, 108-110).
- Access: open copy.
- Read so far (by the project): not read.
- Read in this pass: full text fetched (introduction, Method).
- Reference check: n/a.

### Li2024
- Reference: Li, Y., Liu, L., Li, S., Peng, D., Cao, Z., Li, X. (2024). Cenozoic India-Asia collision driven by mantle dragging the cratonic root. Nature Communications 15:6674.
- DOI or URL: 10.1038/s41467-024-51107-0; https://pmc.ncbi.nlm.nih.gov/articles/PMC11303558/ (open).
- Cited as: not cited by the project; used as an open secondary source that cites Turcotte & Schubert for force orders of magnitude.
- Ledger ids: (supports 183, 187).
- Access: open access.
- Read so far (by the project): not read.
- Read in this pass: full text fetched (Europe PMC XML, grep for force magnitudes and reference 9).
- Reference check: n/a.

### DeMets1994
- Reference: DeMets, C., Gordon, R. G., Argus, D. F., Stein, S. (1994). Effect of recent revisions to the geomagnetic reversal time scale on estimates of current plate motions. Geophysical Research Letters 21(20):2191-2194.
- DOI or URL: 10.1029/94GL02118.
- Cited as: not cited (NUVEL-1A is the model behind Bird 2003's 14 large plates).
- Ledger ids: (none).
- Access: paywalled (abstract).
- Read so far (by the project): not read.
- Read in this pass: abstract fetched; no spreading-rate numbers in it.
- Reference check: n/a.

### Davidson_tectonicsjs
- Reference: Davidson, C. tectonics.js, https://github.com/davidson16807/tectonics.js, files `RockColumn.js`, `Crust.js`, commit ce371fa (CC BY 4.0). Software.
- Ledger ids: 131-136. Access: open source. Read in this pass: not fetched. Reference check: the repo owner is a GitHub user; no paper.

### Not bibliographic
GPlates/pyGPlates (software, GPL-2.0), the community ports of Cortial (robertoost/RNGalaxy, AurelienBesnier/ProceduralPlanets, gubace/Procedural-Tectonic-Planets), PlaTec/PyPlatec, WorldEngine, tectonics.cpp, Undiscovered Worlds, StagYY, ASPECT, Underworld, LaMEM, CitcomS, Rayleigh, pTatin3D, Wikipedia pages (Pacific plate, Izanagi plate, Kula plate, Iapetus Ocean, Supercontinent cycle, Alleghanian orogeny, Uralian orogeny, Himalayan foreland basin, List of tectonic plates), the geosociety.org "Broken sheets" page, the EarthByte page, `docs/deeptime-design-decisions.md`, `docs/deeptime-implementation-plan.md`, `docs/design/plate-collisions-and-welding.md`, `docs/design/plate-force-balance.md`, `docs/research/notes/tectonics_engines.md`, `docs/research/notes/run-review-1a0f969d358.md`, STATUS.md build-log entries, and the editor hint file `editor_fields.json`.

## Climate physics (CLM)

### Rose2018
- Reference: Rose, B. E. J. (2018). CLIMLAB: a Python toolkit for interactive, process-oriented climate modeling. Journal of Open Source Software 3(24): 659. Code: https://github.com/climlab/climlab (MIT; v0.9.2 tag 91eae4b; project cites commit fcbde37, a real commit dated 2026-07-23, "Merge pull request #284 from brian-rose/micromamba").
- DOI or URL: 10.21105/joss.00659
- Cited as: "climlab (Rose 2018, JOSS 3, 659)", "climlab const.S0", "climlab EBM_seasonal / StepFunctionAlbedo / AplusBT / MeridionalHeatDiffusion / DailyInsolation"
- Ledger ids: CLM-001, 006, 007, 008, 009, 010, 020, 027, 029
- Access: open access (paper and code)
- Read so far (by the project): "read on 2026-09-28 (commit fcbde37)", equations ported not code; per-default reading not stated
- Read in this pass: full text fetched (source only): climlab/model/ebm.py, surface/albedo.py, utils/constants.py at main, v0.9.2 and fcbde37 (ebm.py identical in the defaults below at fcbde37 and v0.9.2; main differs from v0.9.2 at line 101, a docstring-area change, defaults unchanged). The JOSS paper itself was not fetched.
- Reference check: matches. Citation volume is 3, issue 24, article 659. Note the code's defaults are NOT NCC81: Rose's lecture notes (Albany ATM623, Lecture 14, fetched) say A = 210 m2 and B = 2 m2/degC "we are going to use these values", derived from an NCEP fit (best fit A = 214, B = 1.6), and D = 0.55.

### North1981
- Reference: North, G. R., Cahalan, R. F., & Coakley, J. A., Jr. (1981). Energy balance climate models. Reviews of Geophysics and Space Physics 19(1): 91-121.
- DOI or URL: 10.1029/RG019I001P00091. Image-only scan at https://eisenman.ucsd.edu/courses/2024-sioc224/files/north-1981.pdf (no text layer; JBIG2 images; I could not OCR it). NASA NTRS record 19820032405 (abstract only, no download).
- Cited as: "North, Cahalan & Coakley 1981", "North et al. 1981", "Rev. Geophys. 19, 91"
- Ledger ids: CLM-003, 004, 005, 006
- Access: paywalled (abstract only); scanned copy at the URL above, unreadable without OCR
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (NTRS). Full text attempted, unreadable (scan).
- Reference check: authors, year, venue, pages match. The code's field comment writes "North et al. 1981" for D and ebm.rs writes the full name; same paper. Whether A = 203.3, B = 2.09, D = 0.649 are NCC81's own numbers is NOT confirmed (see Part 2): citing papers disagree on whether A and B originate with Budyko (1969) or North & Coakley (1979).

### Berger1978
- Reference: Berger, A. L. (1978). Long-term variations of daily insolation and Quaternary climatic changes. Journal of the Atmospheric Sciences 35(12): 2362-2367.
- DOI or URL: 10.1175/1520-0469(1978)035<2362:LTVODI>2.0.CO;2
- Cited as: "Berger 1978"
- Ledger ids: CLM-027, 029, 033, 035, 037
- Access: paywalled (Unpaywall: closed; AMS site blocks automated download)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches. Note a different Berger 1978 paper exists (Quaternary Research 9: 139-167, "Long-term variations of caloric insolation resulting from the Earth's orbital elements"); the code's J. Atmos. Sci. citation is the right one for the daily-insolation formula.

### Laskar2004
- Reference: Laskar, J., Robutel, P., Joutel, F., Gastineau, M., Correia, A. C. M., & Levrard, B. (2004). A long-term numerical solution for the insolation quantities of the Earth. Astronomy & Astrophysics 428(1): 261-285.
- DOI or URL: 10.1051/0004-6361:20041335; https://www.aanda.org/articles/aa/pdf/2004/46/aa1335.pdf (open, but automated fetch returned a bot challenge / 403)
- Cited as: "Laskar et al. 2004, A&A 428: 261-285"
- Ledger ids: CLM-033, 035, 037
- Access: open access (Unpaywall: bronze)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Bergman2004
- Reference: Bergman, N. M., Lenton, T. M., & Watson, A. J. (2004). COPSE: A new model of biogeochemical cycling over Phanerozoic time. American Journal of Science 304(5): 397-437.
- DOI or URL: 10.2475/ajs.304.5.397; open PDF https://ajs.scholasticahq.com/article/61725.pdf
- Cited as: "Bergman, Lenton & Watson 2004 COPSE"
- Ledger ids: CLM-046, 051
- Access: open access (bronze, publisher PDF)
- Read so far (by the project): not stated
- Read in this pass: full text fetched (text only; the reservoir and flux tables are not machine-readable, so the numerical constants were not seen)
- Reference check: matches. The paper writes "Berner's GEOCARB" degassing constants k12/k13 as GEOCARB values times present reservoir sizes; I did not find the number 6.65e12 in the readable text.

### Archer2009
- Reference: Archer, D., Eby, M., Brovkin, V., Ridgwell, A., Cao, L., Mikolajewicz, U., Caldeira, K., Matsumoto, K., Munhoven, G., Montenegro, A., & Tokos, K. (2009). Atmospheric lifetime of fossil fuel carbon dioxide. Annual Review of Earth and Planetary Sciences 37: 117-134.
- DOI or URL: 10.1146/annurev.earth.031208.100206; green copy https://orbi.uliege.be/handle/2268/12933
- Cited as: "Archer et al. 2009"
- Ledger ids: CLM-048
- Access: open copy at https://orbi.uliege.be/handle/2268/12933 (landing page only seen)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches (year 2009, vol 37, pp 117-134).

### Hoffman1998
- Reference: Hoffman, P. F., Kaufman, A. J., Halverson, G. P., & Schrag, D. P. (1998). A Neoproterozoic snowball Earth. Science 281(5381): 1342-1346.
- DOI or URL: 10.1126/science.281.5381.1342
- Cited as: "Hoffman et al. 1998"
- Ledger ids: CLM-050
- Access: paywalled (abstract only; Unpaywall closed)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (via Crossref)
- Reference check: matches. The abstract has the 350-times figure but not "about 3 to 4 Myr" or "5.4e12 mol/yr".

### Berner1994
- Reference: Berner, R. A. (1994). GEOCARB II: a revised model of atmospheric CO2 over Phanerozoic time. American Journal of Science 294(1): 56-91.
- DOI or URL: 10.2475/ajs.294.1.56; PDF https://ajs.scholasticahq.com/article/60629.pdf (page scan, no text layer)
- Cited as: "Berner 1994, GEOCARB II, Am. J. Sci. 294: 56-91"
- Ledger ids: CLM-052, 063, 064
- Access: open access (bronze) but the PDF is a page scan I could not read
- Read so far (by the project): not stated
- Read in this pass: not fetched as readable text (scan). Its constants are quoted in Berner2001 (see below).
- Reference check: matches.

### Berner2001
- Reference: Berner, R. A., & Kothavala, Z. (2001). GEOCARB III: A revised model of atmospheric CO2 over Phanerozoic time. American Journal of Science 301(2): 182-204.
- DOI or URL: 10.2475/ajs.301.2.182; PDF https://ajs.scholasticahq.com/article/61570.pdf
- Cited as: "Berner & Kothavala 2001, GEOCARB III, Am. J. Sci. 301: 182-204, eq. 2", "GEOCARB's ~3.8 %/K", "Berner's GEOCARB / GEOCARB III (degassing 6.65e12 mol C/yr)"
- Ledger ids: CLM-051, 064, 079
- Access: open access (bronze, publisher PDF)
- Read so far (by the project): not stated
- Read in this pass: full text fetched (equations 1-5 and 10-12, the passages on ACT, RUN, FERT, appendix 1 definitions). Did not find the number 6.65e12 in the readable text.
- Reference check: eq. 2 is the right equation for the temperature and runoff factors. The code's 0.038 is GEOCARB II's RUN, not GEOCARB III's (see Part 2).

### Rowley2002
- Reference: Rowley, D. B. (2002). Rate of plate creation and destruction: 180 Ma to present. Geological Society of America Bulletin 114(8): 927-933.
- DOI or URL: 10.1130/0016-7606(2002)114<0927:ROPCAD>2.0.CO;2
- Cited as: "Rowley 2002"
- Ledger ids: CLM-053, 054
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Bird2003
- Reference: Bird, P. (2003). An updated digital model of plate boundaries. Geochemistry, Geophysics, Geosystems 4(3): 1027 (article 2001GC000252).
- DOI or URL: 10.1029/2001GC000252
- Cited as: "Bird 2003" (no title or journal in the code; I assigned this paper because the claim concerns subduction-zone length and rate)
- Ledger ids: CLM-054
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): not stated; no title or journal given
- Read in this pass: search result snippet only (the citing passages returned were about regional plate boundaries and did not mention 51,000 km or 62 mm/yr)
- Reference check: IDENTIFICATION UNCERTAIN. Bird 2003 is the PB2002 plate-boundary model; a total subduction length of 51,000 km at 62 mm/yr is plausible from it but I could not confirm it is in that paper.

### Self2006
- Reference: Self, S., Widdowson, M., Thordarson, T., & Jay, A. E. (2006). Volatile fluxes during flood basalt eruptions and potential effects on the global environment: A Deccan perspective. Earth and Planetary Science Letters 248(1-2): 518-532.
- DOI or URL: 10.1016/j.epsl.2006.05.041; green copy https://hull-repository.worktribe.com/output/3566573
- Cited as: "Self et al. 2006"
- Ledger ids: CLM-057, 058
- Access: open copy (Hull repository, landing page only)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Walker1981
- Reference: Walker, J. C. G., Hays, P. B., & Kasting, J. F. (1981). A negative feedback mechanism for the long-term stabilization of Earth's surface temperature. Journal of Geophysical Research 86(C10): 9776-9782.
- DOI or URL: 10.1029/JC086IC10P09776
- Cited as: "Walker, Hays & Kasting 1981", "Walker, Hays & Kasting 1981 ("0.3")"
- Ledger ids: CLM-060, 063, 065
- Access: paywalled (abstract only; Unpaywall closed)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (Crossref) plus citing passages
- Reference check: matches.

### Myhre1998
- Reference: Myhre, G., Highwood, E. J., Shine, K. P., & Stordal, F. (1998). New estimates of radiative forcing due to well mixed greenhouse gases. Geophysical Research Letters 25(14): 2715-2718.
- DOI or URL: 10.1029/98GL01908
- Cited as: "Myhre et al. 1998"
- Ledger ids: CLM-069, 073
- Access: paywalled (abstract only)
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (Crossref) plus citing passages
- Reference check: matches. Byrne & Goldblatt's Table 1 prints the same 5.35 form (their row attributes it to "IPCC [1990]" and the adjusted forcing F_a).

### Byrne2014
- Reference: Byrne, B., & Goldblatt, C. (2014). Radiative forcing at high concentrations of well-mixed greenhouse gases. Geophysical Research Letters 41(1): 152-160.
- DOI or URL: 10.1002/2013GL058456; open copy https://dspace.library.uvic.ca/server/api/core/bitstreams/65d2a0aa-a7eb-44ba-b8e7-ac34d0d894d7/content (the article as corrected 17 Apr 2015)
- Cited as: "Byrne & Goldblatt 2014, GRL 41: 152-160, Table 2"
- Ledger ids: CLM-070, 071
- Access: open copy at the UVic URL above (publisher page returned 403)
- Read so far (by the project): "checked in their Table 2 by a Sonnet lookup" (a sub-agent)
- Read in this pass: full text fetched (all sections, Tables 1 and 2)
- Reference check: matches (authors, year, volume, pages, Table 2).

### Gough1981
- Reference: Gough, D. O. (1981). Solar interior structure and luminosity variations. Solar Physics 74(1): 21-34.
- DOI or URL: 10.1007/BF00151270
- Cited as: "Gough 1981"
- Ledger ids: CLM-075
- Access: paywalled (Springer; Unpaywall closed)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches. Crossref also lists the same text as a book chapter in "Physics of Solar Variations" (Springer, 1981, doi 10.1007/978-94-010-9633-1_4); same pages.

### Held2006
- Reference: Held, I. M., & Soden, B. J. (2006). Robust responses of the hydrological cycle to global warming. Journal of Climate 19(21): 5686-5699.
- DOI or URL: 10.1175/JCLI3990.1
- Cited as: "Held & Soden 2006" (journal not named in the code)
- Ledger ids: CLM-077, 079
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Hoffman2017
- Reference: Hoffman, P. F., Abbot, D. S., Ashkenazy, Y., Benn, D. I., Brocks, J. J., Cohen, P. A., Cox, G. M., Creveling, J. R., Donnadieu, Y., Erwin, D. H., Fairchild, I. J., Ferreira, D., Goodman, J. C., Halverson, G. P., Jansen, M. F., Le Hir, G., Love, G. D., Macdonald, F. A., Maloof, A. C., Partin, C. A., Ramstein, G., Rose, B. E. J., Rose, C. V., Sadler, P. M., Tziperman, E., Voigt, A., & Warren, S. G. (2017). Snowball Earth climate dynamics and Cryogenian geology-geobiology. Science Advances 3(11): e1600983.
- DOI or URL: 10.1126/sciadv.1600983; open copy https://dspace.library.uvic.ca/bitstream/1828/11278/1/hoffman_paul_scienceadvances_2017.pdf
- Cited as: "Hoffman et al. 2017, Sci. Adv. 3: e1600983"
- Ledger ids: CLM-103
- Access: open access
- Read so far (by the project): not stated
- Read in this pass: full text fetched (text layer searched for sea level, ppm, bar, CO2 termination, ice thickness; not every section read)
- Reference check: matches. The paper's CO2 range for termination is 10^4 to 10^5 ppm, and the ~1 km of sea level is stated as an assumption (see Part 2).

### Nye1952
- Reference: Nye, J. F. (1952). The mechanics of glacier flow. Journal of Glaciology 2(12): 82-93.
- DOI or URL: 10.3189/S0022143000033967
- Cited as: "Nye 1952, J. Glaciol. 2: 82-93"
- Ledger ids: CLM-101
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches (issue 12).

### Cuffey2010
- Reference: Cuffey, K. M., & Paterson, W. S. B. (2010). The Physics of Glaciers, 4th edition. Butterworth-Heinemann / Elsevier. (Section 8.7 cited by the code.)
- DOI or URL: none found (Crossref returned only chapters of the earlier editions). ISBN not looked up.
- Cited as: "Cuffey & Paterson 2010, The Physics of Glaciers, section 8.7"
- Ledger ids: CLM-101
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: exists (4th edition 2010). Section number not checked.

### Felzenszwalb2012
- Reference: Felzenszwalb, P. F., & Huttenlocher, D. P. (2012). Distance transforms of sampled functions. Theory of Computing 8(1): 415-428.
- DOI or URL: 10.4086/toc.2012.v008a019
- Cited as: "Felzenszwalb & Huttenlocher 2012"
- Ledger ids: CLM-110
- Access: open access (Theory of Computing is a free journal; not checked by me)
- Read so far (by the project): "implemented from the paper"
- Read in this pass: not fetched
- Reference check: matches.

### Imbrie1980
- Reference: Imbrie, J., & Imbrie, J. Z. (1980). Modeling the climatic response to orbital variations. Science 207(4434): 943-953.
- DOI or URL: 10.1126/science.207.4434.943
- Cited as: "Imbrie & Imbrie 1980"
- Ledger ids: CLM-104, 105
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Fretwell2013
- Reference: Fretwell, P., Pritchard, H. D., Vaughan, D. G., Bamber, J. L., Barrand, N. E., Bell, R., et al. (2013). Bedmap2: improved ice bed, surface and thickness datasets for Antarctica. The Cryosphere 7(1): 375-393.
- DOI or URL: 10.5194/tc-7-375-2013; https://tc.copernicus.org/articles/7/375/2013/tc-7-375-2013.pdf
- Cited as: "Fretwell et al. 2013" (Antarctica and Greenland about 3.0e7 km3, 65 m)
- Ledger ids: CLM-108
- Access: open access
- Read so far (by the project): not stated
- Read in this pass: full text fetched (abstract and discussion passages on ice volume and sea-level potential)
- Reference check: matches. It covers Antarctica only; Greenland comes from Morlighem2017.

### Morlighem2017
- Reference: Morlighem, M., Williams, C. N., Rignot, E., An, L., Arndt, J. E., Bamber, J. L., et al. (2017). BedMachine v3: Complete bed topography and ocean bathymetry mapping of Greenland from multibeam echo sounding combined with mass conservation. Geophysical Research Letters 44(21): 11051-11061.
- DOI or URL: 10.1002/2017GL074954; open copy https://pure.au.dk/ws/files/120033749/Morlighem_et_al_2017_Geophysical_Research_Letters.pdf
- Cited as: "Morlighem et al. 2017"
- Ledger ids: CLM-108
- Access: open access (hybrid; many repository copies)
- Read so far (by the project): not stated
- Read in this pass: full text fetched (key points, abstract, results text)
- Reference check: matches. Page range 11051-11061 is from my memory of the article number format, not from Crossref (Crossref returned no page); treat as unverified.

### Ohmura1992
- Reference: Ohmura, A., Kasser, P., & Funk, M. (1992). Climate at the equilibrium line of glaciers. Journal of Glaciology 38(130): 397-411.
- DOI or URL: 10.3189/S0022143000002276; open PDF on Cambridge Core (see Unpaywall: bronze)
- Cited as: "Ohmura, Kasser & Funk 1992" (P = 645 + 296 T + 9 T^2, 70 glaciers)
- Ledger ids: CLM-114, 116
- Access: open access (bronze)
- Read so far (by the project): the build log says the equilibrium-line summer temperature was "cited from memory, not re-read" (hydrology agent's check of a related number)
- Read in this pass: full text fetched (OCR text layer; the regression equation itself is not machine-readable)
- Reference check: matches. Its T is the mean summer temperature in the FREE ATMOSPHERE at the ELA, not a surface or sea-level temperature (see Part 2).

### Abbot2010
- Reference: Abbot, D. S., & Pierrehumbert, R. T. (2010). Mudball: Surface dust and Snowball Earth deglaciation. Journal of Geophysical Research: Atmospheres 115(D3): D03104.
- DOI or URL: 10.1029/2009JD012007
- Cited as: "Abbot & Pierrehumbert 2010" (snowball dust darkening)
- Ledger ids: none (feature not shipped)
- Access: not checked
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: exists, title matches the topic. Article number D03104 is from my memory, not Crossref (which returned "2009JD012007" as the article id); treat as unverified.

### gosplDocs
- Reference: goSPL (Global Scalable Paleo Landscape Evolution), Geodels/gospl, `gospl/tools/ela_from_temperature.py`. Software, GPL.
- DOI or URL: https://github.com/Geodels/gospl/blob/HEAD/gospl/tools/ela_from_temperature.py
- Cited as: "goSPL documentation, ELA rule"
- Ledger ids: CLM-113
- Access: open access (source file)
- Read so far (by the project): the rule quoted from the documentation; code not used (GPL)
- Read in this pass: full text fetched (the file)
- Reference check: matches the rule; no journal paper involved.

### TransEBM and related EBM literature (designed-not-coded rows)


### Ziegler2021
- Reference: Ziegler, E., & Rehfeld, K. (2021). TransEBM v. 1.0: description, tuning, and validation of a transient model of the Earth's energy balance in two dimensions. Geoscientific Model Development 14(5): 2843-2866.
- DOI or URL: 10.5194/gmd-14-2843-2021
- Cited as: "Ziegler & Rehfeld 2021, GMD 14, 2843" (Table 1 constants)
- Ledger ids: CLM-119, 120
- Access: open access (GMD)
- Read so far (by the project): not stated; code is GPL-3 "oracle only"
- Read in this pass: not fetched
- Reference check: matches.

### Zhuang2017
- Reference: Zhuang, K., North, G. R., & Stevens, M. J. (2017). A NetCDF version of the two-dimensional energy balance model based on the full multigrid algorithm. SoftwareX 6: 198-202.
- DOI or URL: 10.1016/j.softx.2017.07.003
- Cited as: "Zhuang, North & Stevens 2017, SoftwareX 6, 198"
- Ledger ids: CLM-119, 120
- Access: not checked (SoftwareX is open access)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### North1983
- Reference: North, G. R., Mengel, J. G., & Short, D. A. (1983). Simple energy balance model resolving the seasons and the continents: Application to the astronomical theory of the ice ages. Journal of Geophysical Research 88(C11): 6576-6586.
- DOI or URL: 10.1029/JC088IC11P06576
- Cited as: "North, Mengel & Short 1983, JGR 88(C11) 6576"
- Ledger ids: CLM-122
- Access: not checked
- Read so far (by the project): "paper; equations only"
- Read in this pass: not fetched
- Reference check: matches (DOI as the design doc gives it).

### NorthKim2017
- Reference: North, G. R., & Kim, K.-Y. (2017). Energy Balance Climate Models. Wiley-VCH.
- DOI or URL: 10.1002/9783527698844
- Cited as: "North & Kim 2017, Energy Balance Climate Models, Wiley (ch. 8)"
- Ledger ids: none directly
- Access: book (not online)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches. Chapter 8 not checked.

### Stommel1948
- Reference: Stommel, H. (1948). The westward intensification of wind-driven ocean currents. Transactions, American Geophysical Union 29(2): 202-206.
- DOI or URL: 10.1029/TR029I002P00202
- Cited as: "Stommel 1948, Trans. AGU 29, 202"
- Ledger ids: CLM-128
- Access: not checked
- Read so far (by the project): "implemented from the paper"
- Read in this pass: not fetched
- Reference check: matches.

### RoseMarshall2009
- Reference: Rose, B. E. J., & Marshall, J. (2009). Ocean heat transport, sea ice, and multiple climate states: Insights from energy balance models. Journal of the Atmospheric Sciences 66(9): 2828-2843.
- DOI or URL: 10.1175/2009JAS3039.1; MIT copy http://hdl.handle.net/1721.1/53519
- Cited as: "Rose & Marshall 2009, JAS 66, 2828"
- Ledger ids: none
- Access: open copy (Unpaywall: bronze; MIT repository)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### Ferreira2018
- Reference: Ferreira, D., Cessi, P., Coxall, H. K., de Boer, A., Dijkstra, H. A., Drijfhout, S. S., Eldevik, T., Harnik, N., McManus, J. F., Marshall, D. P., Nilsson, J., Roquet, F., Schneider, T., & Wills, R. C. (2018). Atlantic-Pacific asymmetry in deep water formation. Annual Review of Earth and Planetary Sciences 46: 327-352.
- DOI or URL: 10.1146/annurev-earth-082517-010045; green copy https://dspace.library.uu.nl/handle/1874/428290
- Cited as: "Ferreira et al. 2018, Annu. Rev. Earth Planet. Sci. 46, 327" (marked "cited from memory, not re-read")
- Ledger ids: CLM-132
- Access: open copy (Utrecht repository, landing page)
- Read so far (by the project): from memory
- Read in this pass: abstract fetched (Crossref/Semantic Scholar)
- Reference check: EXISTS and the citation is right (46: 327-352). What it is cited for (deep water forms in the narrowest polar-connected basin; basin width selects the Atlantic) is NOT what the abstract leads with: the abstract says the asymmetry occurs "because the Atlantic is more saline". Citing papers also say basin width/continent shape is one proposed control (see Part 2).

### Trenberth2001
- Reference: Trenberth, K. E., & Caron, J. M. (2001). Estimates of meridional atmosphere and ocean heat transports. Journal of Climate 14(16): 3433-3443.
- DOI or URL: 10.1175/1520-0442(2001)014<3433:EOMAAO>2.0.CO;2
- Cited as: "Trenberth & Caron 2001, J. Climate 14, 3433"
- Ledger ids: CLM-133
- Access: open access (bronze; AMS site blocks automated download)
- Read so far (by the project): not stated, not marked from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Lindzen1987
- Reference: Lindzen, R. S., & Nigam, S. (1987). On the role of sea surface temperature gradients in forcing low-level winds and convergence in the tropics. Journal of the Atmospheric Sciences 44(17): 2418-2436.
- DOI or URL: 10.1175/1520-0469(1987)044<2418:OTROSS>2.0.CO;2
- Cited as: "Lindzen & Nigam 1987, JAS 44, 2418" (marked from memory)
- Ledger ids: CLM-141
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, citation right. The paper is about SST gradients forcing marine boundary-layer pressure gradients; the design uses the idea for land-sea (monsoon) temperature contrasts. That is an extension, not the paper's subject.

### Dommenget2011
- Reference: Dommenget, D., & Flöter, J. (2011). Conceptual understanding of climate change with a globally resolved energy balance model. Climate Dynamics 37(11-12): 2143-2165.
- DOI or URL: 10.1007/s00382-011-1026-0
- Cited as: "Dommenget & Floter 2011, Clim. Dyn. 37, 2143"
- Ledger ids: CLM-142
- Access: not checked
- Read so far (by the project): "equations only"; code has no licence
- Read in this pass: not fetched
- Reference check: matches.

### Stassen2019
- Reference: Stassen, C., Dommenget, D., & Loveday, N. (2019). A hydrological cycle model for the Globally Resolved Energy Balance (GREB) model v1.0. Geoscientific Model Development 12(1): 425-440.
- DOI or URL: 10.5194/gmd-12-425-2019
- Cited as: "Stassen, Dommenget & Loveday 2019, GMD 12, 425"
- Ledger ids: CLM-142
- Access: open access (GMD)
- Read so far (by the project): "equations only"
- Read in this pass: not fetched
- Reference check: matches.

### Fanning1996
- Reference: Fanning, A. F., & Weaver, A. J. (1996). An atmospheric energy-moisture balance model: Climatology, interpentadal climate change, and coupling to an ocean general circulation model. Journal of Geophysical Research 101(D10): 15111-15128.
- DOI or URL: 10.1029/96JD01017
- Cited as: "Fanning & Weaver 1996, JGR 101(D10), 15111"
- Ledger ids: none
- Access: not checked
- Read so far (by the project): not stated; code licence unverifiable
- Read in this pass: not fetched
- Reference check: matches.

### Weaver2001
- Reference: Weaver, A. J., Eby, M., Wiebe, E. C., Bitz, C. M., Duffy, P. B., Ewen, T. L., Fanning, A. F., Holland, M. M., MacFadyen, A., Matthews, H. D., Meissner, K. J., Saenko, O., Schmittner, A., Wang, H., & Yoshimori, M. (2001). The UVic Earth System Climate Model: Model description, climatology, and applications to past, present and future climates. Atmosphere-Ocean 39(4): 361-428.
- DOI or URL: 10.1080/07055900.2001.9649686
- Cited as: "Weaver et al. 2001"
- Ledger ids: none
- Access: not checked
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### Bretherton2004
- Reference: Bretherton, C. S., Peters, M. E., & Back, L. E. (2004). Relationships between water vapor path and precipitation over the tropical oceans. Journal of Climate 17(7): 1517-1528.
- DOI or URL: 10.1175/1520-0442(2004)017<1517:RBWVPA>2.0.CO;2
- Cited as: "Bretherton, Peters & Back 2004, J. Climate 17, 1517" (marked from memory)
- Ledger ids: CLM-145
- Access: open access (bronze). AMS site blocks automated download; the Wayback copy is a page scan (no text layer).
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, citation right. See Part 2 for the 15.6.

### Manabe1969
- Reference: Manabe, S. (1969). Climate and the ocean circulation: I. The atmospheric circulation and the hydrology of the earth's surface. Monthly Weather Review 97(11): 739-774.
- DOI or URL: 10.1175/1520-0493(1969)097<0739:CATOC>2.3.CO;2
- Cited as: "Manabe 1969, Mon. Weather Rev. 97, 739" (marked from memory)
- Ledger ids: CLM-143
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, citation right (Part I, pp. 739-774; Part II by Manabe & Bryan follows at 775-805).

### Siler2018
- Reference: Siler, N., Roe, G. H., & Armour, K. C. (2018). Insights into the zonal-mean response of the hydrologic cycle to global warming from a diffusive energy balance model. Journal of Climate 31(18): 7481-7493.
- DOI or URL: 10.1175/JCLI-D-18-0081.1
- Cited as: "Siler, Roe & Armour 2018"
- Ledger ids: none
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### Bischoff2014
- Reference: Bischoff, T., & Schneider, T. (2014). Energetic constraints on the position of the intertropical convergence zone. Journal of Climate 27(13): 4937-4951.
- DOI or URL: 10.1175/JCLI-D-13-00650.1
- Cited as: "Bischoff & Schneider 2014"
- Ledger ids: none
- Access: open access (bronze)
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### Wagner2015
- Reference: Wagner, T. J. W., & Eisenman, I. (2015). How climate model complexity influences sea ice stability. Journal of Climate 28(10): 3998-4014.
- DOI or URL: 10.1175/JCLI-D-14-00654.1
- Cited as: "Wagner & Eisenman 2015, J. Climate 28, 3998"
- Ledger ids: none
- Access: open access (bronze)
- Read so far (by the project): "equations only"
- Read in this pass: not fetched
- Reference check: matches.

### Klein1993
- Reference: Klein, S. A., & Hartmann, D. L. (1993). The seasonal cycle of low stratiform clouds. Journal of Climate 6(8): 1587-1606.
- DOI or URL: 10.1175/1520-0442(1993)006<1587:TSCOLS>2.0.CO;2
- Cited as: "Klein & Hartmann 1993, J. Climate 6, 1587" (marked from memory)
- Ledger ids: none (follow-up not coded)
- Access: paywalled (Unpaywall closed)
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, citation right. Citing papers define lower-tropospheric stability (LTS) from this paper, which fits "low cloud" as the stated use.

### Serreze1992
- Reference: Serreze, M. C., Kahl, J. D., & Schnell, R. C. (1992). Low-level temperature inversions of the Eurasian Arctic and comparisons with Soviet drifting station data. Journal of Climate 5(6): 615-629. (Crossref lists the author order Serreze, Schnell, Kahl.)
- DOI or URL: 10.1175/1520-0442(1992)005<0615:LLTIOT>2.0.CO;2
- Cited as: "Serreze, Kahl & Schnell 1992, J. Climate 5: 615" (marked from memory)
- Ledger ids: CLM-134
- Access: open access (bronze); AMS blocks automated download
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, pages right. Author order differs: Crossref gives Serreze, Schnell, Kahl; the code writes Serreze, Kahl & Schnell. The paper is about inversions over the Eurasian Arctic (soundings and drifting stations), which fits "winter surface inversion"; the specific "Siberia's January surface 10-20 K below 850 hPa" could not be checked.

### Ramirez2024
- Reference: Ramirez, R. M. (2024). A new 2D energy balance model for simulating the climates of rapidly and slowly rotating terrestrial planets. The Planetary Science Journal 5(1): 2.
- DOI or URL: 10.3847/PSJ/AD0729; preprint arXiv 2310.15992
- Cited as: "PlaHab (Ramirez 2023, PSJ)"
- Ledger ids: CLM-121
- Access: open access (PSJ; arXiv preprint)
- Read so far (by the project): "not inspected, nothing taken"
- Read in this pass: not fetched (arxiv.org/html/2310.15992 returned 404; PDF not text-readable)
- Reference check: IDENTIFIED by title search (the name "PlaHab" returns nothing in Crossref). Year is 2024 in the journal (2023 as a preprint); the design doc's "Ramirez 2023" is the preprint year. I did not confirm that the paper uses 1/25 for the zonal-to-meridional ratio.

### Tetens
- Reference: UNIDENTIFIED in this pass. The code names only "Tetens" for a saturation-vapour formula. Likely Tetens, O. (1930), "Über einige meteorologische Begriffe", Zeitschrift für Geophysik 6: 297-309; this is my recollection, not verified (no DOI, no lookup possible).
- DOI or URL: none
- Cited as: "Tetens"
- Ledger ids: CLM-142
- Access: not found
- Read so far (by the project): no citation given
- Read in this pass: not fetched
- Reference check: cannot confirm.

### Beck2018
- Reference: Beck, H. E., Zimmermann, N. E., McVicar, T. R., Vergopolan, N., Berg, A., & Wood, E. F. (2018). Present and future Köppen-Geiger climate classification maps at 1-km resolution. Scientific Data 5: 180214. Dataset: figshare 6396959 (doi 10.6084/m9.figshare.6396959).
- DOI or URL: 10.1038/sdata.2018.214
- Cited as: "Beck et al. 2018, Sci. Data 5, 180214 (figshare 6396959)"
- Ledger ids: none (validation only)
- Access: open access
- Read so far (by the project): used for validation only
- Read in this pass: dataset record fetched (figshare API): title and file list confirmed
- Reference check: matches.

### Beck2023
- Reference: Beck, H. E., McVicar, T. R., Vergopolan, N., Berg, A., Lutsko, N. J., Dufour, A., Zeng, Z., Jiang, X., van Dijk, A. I. J. M., & Miralles, D. G. (2023). High-resolution (1 km) Köppen-Geiger maps for 1901-2099 based on constrained CMIP6 projections. Scientific Data 10: 724. Dataset: figshare 21789074 (doi 10.6084/m9.figshare.21789074.v2).
- DOI or URL: 10.1038/s41597-023-02549-6
- Cited as: "Beck et al. 2023, Sci. Data 10, 724 (monthly climatologies, figshare 21789074)"
- Ledger ids: none (validation only)
- Access: open access
- Read so far (by the project): used for validation only
- Read in this pass: dataset record fetched (figshare API). The record lists `climate_data_*.zip` files, which fits "monthly climatologies"; the 1961-1990 period was not checked.
- Reference check: matches.

### Tozer2019
- Reference: Tozer, B., Sandwell, D. T., Smith, W. H. F., Olson, C., Beale, J. R., & Wessel, P. (2019). Global bathymetry and topography at 15 arc sec: SRTM15+. Earth and Space Science 6(10): 1847-1864.
- DOI or URL: 10.1029/2019EA000658
- Cited as: "GMT earth_relief (SRTM15+, Tozer et al. 2019)"
- Ledger ids: none
- Access: not checked (journal is open access)
- Read so far (by the project): data used
- Read in this pass: not fetched
- Reference check: matches.

### ETOPO2022
- Reference: NOAA National Centers for Environmental Information (2022). ETOPO 2022 15 Arc-Second Global Relief Model. NOAA NCEI.
- DOI or URL: 10.25921/fd45-gt74 (title and year confirmed through the DataCite API)
- Cited as: "ETOPO 2022"
- Ledger ids: none
- Access: open access (dataset)
- Read so far (by the project): data named
- Read in this pass: not fetched
- Reference check: matches.

### Rayner2003
- Reference: Rayner, N. A., Parker, D. E., Horton, E. B., Folland, C. K., Alexander, L. V., Rowell, D. P., Kent, E. C., & Kaplan, A. (2003). Global analyses of sea surface temperature, sea ice, and night marine air temperature since the late nineteenth century. Journal of Geophysical Research 108(D14): 4407 (Crossref article id 2002JD002670).
- DOI or URL: 10.1029/2002JD002670
- Cited as: "HadISST" (named only)
- Ledger ids: none
- Access: not checked
- Read so far (by the project): data named
- Read in this pass: not fetched
- Reference check: HadISST1 reference (my assignment; the code names no paper). Article number 4407 is from my memory.

### Adler2018
- Reference: Adler, R. F., Sapiano, M. R. P., Huffman, G. J., Wang, J.-J., Gu, G., Bolvin, D., Chiu, L., Schneider, U., Becker, A., Nelkin, E., Xie, P., Ferraro, R., & Shin, D.-B. (2018). The Global Precipitation Climatology Project (GPCP) Monthly Analysis (New Version 2.3) and a Review of 2017 Global Precipitation. Atmosphere 9(4): 138.
- DOI or URL: 10.3390/atmos9040138
- Cited as: "GPCP v2.3" (named only)
- Ledger ids: none
- Access: open access (MDPI; not checked)
- Read so far (by the project): data named
- Read in this pass: not fetched
- Reference check: matches the version named.

### Peel2007
- Reference: Peel, M. C., Finlayson, B. L., & McMahon, T. A. (2007). Updated world map of the Köppen-Geiger climate classification. Hydrology and Earth System Sciences 11(5): 1633-1644.
- DOI or URL: 10.5194/hess-11-1633-2007
- Cited as: "Peel, Finlayson & McMahon 2007 (HESS 11:1633)"
- Ledger ids: none (comparison only)
- Access: open access
- Read so far (by the project): the shares were "quoted in the 2026-09-29 P1-17 entry"
- Read in this pass: full text fetched (section 4 conclusions)
- Reference check: matches.

### Added Earth-reference sources (not cited in the code)


### Rohde2020
- Reference: Rohde, R. A., & Hausfather, Z. (2020). The Berkeley Earth Land/Ocean Temperature Record. Earth System Science Data 12(4): 3469-3479. Data file: https://berkeley-earth-temperature.s3.us-west-1.amazonaws.com/Global/Land_and_Ocean_complete.txt (analysis run 2025-01).
- DOI or URL: 10.5194/essd-12-3469-2020
- Cited as: none (supplied here for Earth's global mean temperature)
- Ledger ids: CLM-026, 078, 138
- Access: open access (paper and data)
- Read so far (by the project): n/a
- Read in this pass: full text fetched (the data file header and the annual series, not the paper)
- Reference check: n/a

### Kopp2011
- Reference: Kopp, G., & Lean, J. L. (2011). A new, lower value of total solar irradiance: Evidence and climate significance. Geophysical Research Letters 38(1): L01706.
- DOI or URL: 10.1029/2010GL045777
- Cited as: none (supplied here for the solar constant)
- Ledger ids: CLM-001
- Access: paywalled? (Wiley page blocked; not checked)
- Read so far (by the project): n/a
- Read in this pass: search result snippet only (Kopp's own web page, https://spot.colorado.edu/~koppg/TSI/, quotes the value)
- Reference check: n/a. Article number L01706 is from my memory.

### Secondary sources used for A and B
- Lohmann, G. (2020). Temperatures from energy balance models: the effective heat capacity matters. Earth System Dynamics 11(4): 1195-1208, doi 10.5194/esd-11-1195-2020 (fetched; quoted below).
- Samuelsberg, A., & Jakobsen, P. K. (2025). Solving a North-type energy balance model using boundary integral methods. Nonlinear Processes in Geophysics 32(1): 23-33, doi 10.5194/npg-32-23-2025 (PDF text read; Table 1).
- Rushley, S. S., Kim, D., Bretherton, C. S., & Ahn, M.-S. (2018). Reexamining the nonlinear moisture-precipitation relationship over the tropical oceans. Geophysical Research Letters 45(2): 1133-1140, doi 10.1002/2017GL076296 (citing passage only).
- NSIDC, "Science of sea ice" page, https://nsidc.org/learn/parts-cryosphere/sea-ice/science-sea-ice (fetched).
- Brian Rose, ATM 623 lecture 14 notes, University at Albany (fetched).

### Software named in the research note (no coefficient in the ledger rests on them)
- PlaSim: Fraedrich, K., Jansen, H., Kirk, E., Luksch, U., & Lunkeit, F. (2005). The Planet Simulator: Towards a user friendly model. Meteorologische Zeitschrift 14(3): 299-304, doi 10.1127/0941-2948/2005/0043.
- ExoPlaSim: Paradise, A., Macdonald, E., Menou, K., Lee, C., & Fan, B. L. (2022). ExoPlaSim: Extending the Planet Simulator for exoplanets. MNRAS 511(3): 3272-3303, doi 10.1093/mnras/stac172.
- SpeedyWeather.jl: Klöwer, M., et al. (2024). SpeedyWeather.jl: Reinventing atmospheric general circulation models towards interactivity and extensibility. JOSS 9(98): 6323, doi 10.21105/joss.06323.
- Isca: Vallis, G. K., et al. (2018). Isca, v1.0: a framework for the global modelling of the atmospheres of Earth and other planets at varying levels of complexity. GMD 11(3): 843-859, doi 10.5194/gmd-11-843-2018.
- These four: found by Crossref title match, not fetched. The note gives no citation strings, so which paper the note means is my assignment. LFPM/OpenLEM (landscape-evolution models) and the "Worldbuilding Pasta ExoPlaSim guide" (a web guide) I did not identify as literature.

#### Not bibliographic
Plan section 8.3, the design decisions log, STATUS.md build log, NOTICE.md, `docs/research/notes/surface_processes_climate.md`, `docs/design/p1-26-per-cell-climate.md`, the GMT `earth_relief_*` files (SRTM15+ is Tozer2019), HadISST and GPCP (data products, see Rayner2003 and Adler2018), no Minecraft mods in this subsystem.

---

## Lithospheric flexure (ISO-040 to ISO-067)

Added with the feature (build log 2026-10-02, "General lithospheric flexure"). Every record below
was checked against Crossref by a Sonnet sub-agent of the flexure agent on 2026-10-02; "Read so far" says what was
read of the work itself. `TurcotteSchubert_Geodynamics` (Plate tectonics section) is the other
source of this feature: its chapter 3 was read as a PDF for ISO-041 and ISO-042, and through
lecture notes for ISO-040 and ISO-067.

### Calmant1990
- Reference: Calmant, S., Francheteau, J., Cazenave, A. (1990). Elastic layer thickening with age of the oceanic lithosphere: a tool for prediction of the age of volcanoes or oceanic crust. Geophysical Journal International 100(1):59-67.
- DOI or URL: 10.1111/j.1365-246X.1990.tb04567.x; full text https://horizon.documentation.ird.fr/exl-doc/pleins_textes/pleins_textes_5/b_fdi_31-32/34824.pdf
- Cited as: "Calmant, Francheteau & Cazenave 1990"
- Ledger ids: ISO-048.
- Access: open copy.
- Read so far (by the project): full text by a Sonnet sub-agent of the flexure agent on 2026-10-02: "Te (km) = (2.7 +- 0.15) sqrt(t (Ma))", a weighted regression of 38 seamount estimates; rheology prediction 3.3 (wet) to 4.3 (dry).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### Watts1978
- Reference: Watts, A. B. (1978). An analysis of isostasy in the world's oceans 1. Hawaiian-Emperor Seamount Chain. Journal of Geophysical Research 83(B12):5989-6004.
- DOI or URL: 10.1029/JB083iB12p05989
- Cited as: "Watts 1978"
- Ledger ids: ISO-056 (and the comment of ISO-048).
- Access: paywalled (abstract visible).
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: Te 20-30 km along the chain, "corresponds closely to the 450 +- 150 C isotherm".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct. That a load keeps the elastic thickness of its time of loading is how the abstract is commonly summarised; the paper body was not read.

### WattsTenBrink1989
- Reference: Watts, A. B., ten Brink, U. S. (1989). Crustal structure, flexure, and subsidence history of the Hawaiian Islands. Journal of Geophysical Research 94(B8):10473-10500.
- DOI or URL: 10.1029/JB094iB08p10473
- Cited as: "Watts & ten Brink 1989"
- Ledger ids: ISO-049.
- Access: paywalled (abstract visible).
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: a good fit with Te = 40 km, "higher than expected for 80 m.y. oceanic lithosphere".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct. The arch's distance and height, often attributed to it, were not confirmed.

### Wessel1993
- Reference: Wessel, P. (1993). A reexamination of the flexural deformation beneath the Hawaiian Islands. Journal of Geophysical Research 98(B7):12177-12190.
- DOI or URL: 10.1029/93JB00523
- Cited as: docs/design/flexure.md section 11
- Ledger ids: none (context for ISO-049).
- Access: paywalled (abstract visible).
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: Te 35 km away from the load, 25 km under it.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### BurovDiament1995
- Reference: Burov, E. B., Diament, M. (1995). The effective elastic thickness (Te) of continental lithosphere: What does it really mean? Journal of Geophysical Research 100(B3):3905-3927.
- DOI or URL: 10.1029/94JB02770
- Cited as: "Burov & Diament 1995"
- Ledger ids: ISO-050.
- Access: paywalled (abstract visible).
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: 70-90 km for cratons, 10-20 km at the low end; thermotectonic age is one of at least three controls (with crust-mantle decoupling and plate curvature).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct. The model uses the age control only.

### PerezGussinyeWatts2005
- Reference: Pérez-Gussinyé, M., Watts, A. B. (2005). The long-term strength of Europe and its implications for plate-forming processes. Nature 436:381-384.
- DOI or URL: 10.1038/nature03854
- Cited as: docs/design/flexure.md section 11
- Ledger ids: none (context for ISO-050).
- Access: paywalled (abstract on Europe PMC).
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: mean Te above 60 km for lithosphere of 1.5 Gyr and older, under 30 km for younger.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### Lu2024
- Reference: Lu, Z., Audet, P., Li, C.-F., Zhang, J., Li, S. (2024). Strength of continental lithosphere governed by the time since the last orogeny. Earth and Planetary Science Letters 643:118894.
- DOI or URL: 10.1016/j.epsl.2024.118894
- Cited as: "Lu et al. 2024"
- Ledger ids: ISO-051, ISO-052.
- Access: paywalled.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02 (on the authors' group page; the publisher's page was blocked): continental Te from 5 to about 140 km, mean 50 km, governed by the time since the last orogeny.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: record confirmed in Crossref. It gives no recovery law in the abstract: the exponential and its 300 Myr are not from it.

### KarnerWatts1983
- Reference: Karner, G. D., Watts, A. B. (1983). Gravity anomalies and flexure of the lithosphere at mountain ranges. Journal of Geophysical Research 88(B12):10449-10477.
- DOI or URL: 10.1029/JB088iB12p10449
- Cited as: docs/design/flexure.md section 11
- Ledger ids: none (context for ISO-050, ISO-055).
- Access: paywalled (abstract visible).
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: Te 80-100 km for the Himalaya's foreland, 25-50 km for the Alps, 80-130 km for the Appalachians.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### LyonCaenMolnar1983
- Reference: Lyon-Caen, H., Molnar, P. (1983). Constraints on the structure of the Himalaya from an analysis of gravity anomalies and a flexural model of the lithosphere. Journal of Geophysical Research 88(B10):8171-8191.
- DOI or URL: 10.1029/JB088iB10p08171
- Cited as: "Lyon-Caen & Molnar 1983"
- Ledger ids: ISO-055.
- Access: paywalled (abstract visible).
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: flexural rigidity about 0.7e25 N m under the Ganga basin and Lesser Himalaya, 0.1 to 1.0e23 N m beneath the Greater Himalaya.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct. It is evidence for a stiff foreland beside a weak belt, not for a break along every plate boundary.

### BrotchieSilvester1969
- Reference: Brotchie, J. F., Silvester, R. (1969). On crustal flexure. Journal of Geophysical Research 74(22):5240-5252.
- DOI or URL: 10.1029/JB074i022p05240
- Cited as: "Brotchie & Silvester 1969"
- Ledger ids: ISO-067.
- Access: paywalled.
- Read so far (by the project): not read; the point-load formula was read as quoted in Wickert 2016 (eq. 5-6, full text) by a Sonnet sub-agent of the flexure agent on 2026-10-02.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: record correct.

### Wickert2016
- Reference: Wickert, A. D. (2016). Open-source modular solutions for flexural isostasy: gFlex v1.0. Geoscientific Model Development 9(3):997-1017.
- DOI or URL: 10.5194/gmd-9-997-2016
- Cited as: docs/design/flexure.md section 11
- Ledger ids: ISO-067.
- Access: open access (the paper is CC-BY; the code is GPL-3).
- Read so far (by the project): the paper's full text by a Sonnet sub-agent of the flexure agent on 2026-10-02, for the point-load formula and the licence. The code was not opened and nothing is ported.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### WieczerkowskiMitrovicaWolf1999
- Reference: Wieczerkowski, K., Mitrovica, J. X., Wolf, D. (1999). A revised relaxation-time spectrum for Fennoscandia. Geophysical Journal International 139(1):69-86.
- DOI or URL: 10.1046/j.1365-246X.1999.00924.x
- Cited as: docs/design/flexure.md sections 7 and 11
- Ledger ids: ISO-064.
- Access: open (publisher page).
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: relaxation times "close to 3-5 kyr".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### DeCellesGiles1996_flexure
- Reference: DeCelles, P. G., Giles, K. A. (1996). Foreland basin systems. Basin Research 8(2):105-123. (Also `DeCellesGiles1996` in the Plate tectonics section, where stage 1 saw the abstract only.)
- DOI or URL: 10.1046/j.1365-2117.1996.01491.x; full text https://pages.uoregon.edu/rdorsey/BasinAnalysis/BasinPapers/DeCelles%20&%20Giles%201996.pdf
- Cited as: docs/design/flexure.md section 11
- Ledger ids: none (the acceptance figures of docs/design/plate-force-balance.md).
- Access: open copy.
- Read so far (by the project): full text by a Sonnet sub-agent of the flexure agent on 2026-10-02: "Foredeep depozones are typically 100-300 km wide and 2-8 km thick"; forebulges 60-470 km wide and a few tens to a few hundred metres high, as a flexural prediction.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct; this settles what stage 1 could not see.

## Plate-check boundary mix and arc heights (PLT-213 to PLT-221)

Added with the feature (build log 2026-10-02, "plate-check boundary mix and arc heights"). Each source
below was opened by the agent that built the measures on 2026-10-02; "Read so far" says how much.
Bird 2003, Hildreth 2007 and Gregory-Wodzicki 2000 were downloaded as PDFs and their text extracted
with pdftotext; the others are abstracts on the publisher's or the laboratory's page.

### Bird2003_Table3
- Reference: Bird, P. (2003). An updated digital model of plate boundaries. Geochemistry, Geophysics, Geosystems 4(3):1027.
- DOI or URL: 10.1029/2001GC000252; full text http://peterbird.name/publications/2003_PB2002/2001GC000252.pdf
- Cited as: "Bird 2003, Table 3"
- Ledger ids: PLT-213 (and PLT-214, PLT-221 by reference). The same work is under `Bird2003` in the plate tectonics and climate sections for other claims.
- Access: open (the author's site).
- Read so far (by the project): full text, 2026-10-02: Table 3 "Collective Properties of Plate Boundaries by Class": CCB 23,003 km (8.8 %), CTF 26,132 (10.0 %), CRB 27,472 (10.5 %), OSR 67,338 (25.8 %), OTF 47,783 (18.3 %), OCB 17,449 (6.7 %), SUB 51,310 (19.7 %), total 260,487 km (100 %); paragraph 126 defines SUB as convergent steps "with a Benioff zone of intermediate to deep earthquakes and/or a parallel volcanic arc with Quaternary activity", "not divided into continental and oceanic types"; 5,819 steps of mean length 44.7 km.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### Hildreth2007_pp1744
- Reference: Hildreth, W. (2007). Quaternary magmatism in the Cascades: geologic perspectives. U.S. Geological Survey Professional Paper 1744.
- DOI or URL: 10.3133/pp1744; full text https://pubs.usgs.gov/pp/pp1744/pp1744.pdf
- Cited as: "Hildreth 2007, p. 3 and Table 2"
- Ledger ids: PLT-218, PLT-219. The same work is under `Hildreth2007` in the volcanoes section.
- Access: open (agency report).
- Read so far (by the project): full text, 2026-10-02: the arc "extends 1,250 km north-south"; "Elevation of the pre-Quaternary basement beneath the volcanic chain ... ranges from 1,200 to 1,500 m in much of Oregon to more than 2,000 m in northern Washington. Among the major cones, a dozen exceed 3,000 m in elevation but only Rainier and Shasta exceed 4,000 m" (p. 3); Table 2 (p. 7): Rainier 4,392 m, Shasta 4,316 m, Adams 3,742 m, Hood 3,425 m, Baker 3,286 m.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### GregoryWodzicki2000
- Reference: Gregory-Wodzicki, K. M. (2000). Uplift history of the Central and Northern Andes: A review. Geological Society of America Bulletin 112(7):1091-1105.
- DOI or URL: 10.1130/0016-7606(2000)112<1091:UHOTCA>2.0.CO;2; open copy http://geomorphology.sese.asu.edu/Papers/gregory-wodzicki_andes_uplift_gsab00.pdf
- Cited as: "Gregory-Wodzicki 2000"
- Ledger ids: PLT-218.
- Access: open copy.
- Read so far (by the project): full text, 2026-10-02: section "Morphotectonic provinces: across-strike variation", Altiplano subdomain: "The magmatic arc consists of widely spaced volcanic peaks superimposed on a 4500-m-high plateau (Western Cordillera)"; the Altiplano is a "3700-m-high plateau". The paper reviews paleoelevation; only this description of present relief was used.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### Baker2008
- Reference: Baker, E. T., Embley, R. W., Walker, S. L., Resing, J. A., Lupton, J. E., Nakamura, K.-I., de Ronde, C. E. J., Massoth, G. J. (2008). Hydrothermal activity and volcano distribution along the Mariana arc. Journal of Geophysical Research 113:B08S09.
- DOI or URL: 10.1029/2007JB005423 (full text blocked to the fetch tool, 403); abstract at https://www.pmel.noaa.gov/public/pmel/publications-search/search_abstract.php?fmContributionNum=3118
- Cited as: "Baker et al. 2008"
- Ledger ids: PLT-215.
- Access: open (abstract read).
- Read so far (by the project): abstract only, 2026-10-02: "76 volcanic edifices along 1370 km of arc, grouped into 60 'volcanic centers,' of which at least 26 (20 submarine) are hydrothermally or volcanically active"; all intraoceanic arcs combined may emit about 10 % of the global mid-ocean ridge's hydrothermal output. The full text, which would give the emergent islands, was not read.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### deRonde2003
- Reference: de Ronde, C. E. J., Massoth, G. J., Baker, E. T., Lupton, J. E. (2003). Submarine hydrothermal venting related to volcanic arcs. In: Simmons, S. F., Graham, I. J. (eds), Volcanic, Geothermal and Ore-Forming Fluids: Rulers and Witnesses of Processes within the Earth, Society of Economic Geologists Special Publication 10:91-110 (Giggenbach Memorial Volume).
- DOI or URL: abstract at https://www.pmel.noaa.gov/public/pmel/publications-search/search_abstract.php?fmContributionNum=2468
- Cited as: "de Ronde et al. 2003"
- Ledger ids: PLT-215, PLT-216.
- Access: abstract read.
- Read so far (by the project): abstract only, 2026-10-02: 21 volcanic arcs with a submarine component, almost 22,000 km (about 93 % in the Pacific); "696 volcanoes occur along these arcs, with at least 209 (30%) being submarine"; 31 active vent sites on submarine arc volcanoes, two thirds shallower than 1,000 m; less than 3 % of arc length surveyed.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (the book title was taken from the series, not the abstract page).

### Baker2003_Kermadec
- Reference: Baker, E. T., Feely, R. A., de Ronde, C. E. J., Massoth, G. J., Wright, I. C. (2003). Submarine hydrothermal venting on the southern Kermadec volcanic arc front (offshore New Zealand): location and extent of particle plume signatures. In: Larter, R. D., Leat, P. T. (eds), Intra-Oceanic Subduction Systems: Tectonic and Magmatic Processes, Geological Society of London Special Publication 219:141-161.
- DOI or URL: abstract at https://www.pmel.noaa.gov/public/pmel/publications-search/search_abstract.php?fmContributionNum=2437
- Cited as: "Baker et al. 2003"
- Ledger ids: PLT-216.
- Access: abstract read.
- Read so far (by the project): abstract only, 2026-10-02: "13 volcanoes along 260 km of the southern Kermadec arc", hydrothermal activity at seven. The vent depth range of 250-1,660 m below sea level was read in the abstract of de Ronde et al. 2003, not in this one.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.
