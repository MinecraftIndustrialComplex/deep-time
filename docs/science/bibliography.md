# Bibliography

Every scientific source the code, its comments, its data files and the build log rely on, with the
full reference as far as stage 1 (2026-10-02) could establish it, the ledger ids that rest on it,
how it had been read, and, since stage 2 package WP-0 (2026-10-02), whether the project's source
library holds a readable copy.

The sources that only the research notes and design documents cite (tool surveys, mod
documentation, prior art) are listed separately in
[bibliography-research-notes.md](bibliography-research-notes.md), as those documents give them;
where a mention there is the same work as an entry here it carries `[= Key]`.

## How to read an entry

- **Reference**: authors, year, title, venue. Established by a stage 1 sub-agent from Crossref,
  OpenAlex, publisher pages and agency sites. A reference marked `UNIDENTIFIED` could not be pinned
  down from the citation in the code.
- **Cited as**: the string the code or documents use.
- **Ledger ids**: the rows of [claims.md](claims.md) that rely on it. Rows that WP-0 retired as
  duplicates are replaced by the row that now owns the claim.
- **Access**: whether the source can be read freely, as stage 1 recorded it. "Paywalled" sources are
  the ones stage 2 needs a library, an author copy or a preprint for.
- **Library**: new in WP-0. Whether a copy is in the project's source library (a folder on the test
  machine, one folder per key, with the PDF or page and a plain-text extraction beside it; it is
  **not distributed** and never committed, for copyright), where the copy came from, and the name of
  the text file. A text file whose source is a scan has no usable text layer. "Landing page only"
  means the publisher or repository page was saved but not the paper.
- **Substitute**: for a paywalled work, the lawful open copy that stands in for the printed version
  (a preprint, author or institutional repository copy), or "none found".
- **Merged keys**: other keys that stage 1 gave this work. See the table of merged keys below.
- **Identification (stage 2, WP-0)**: how a stage 1 `UNIDENTIFIED` or `not found` citation was
  resolved, or why it was not.
- **Read so far (by the project)**: what the code, the build log or the research notes say about how
  the agent that used the source read it. "Not stated" is the common case, and for this project it
  usually means the value was written from the agent's memory of the literature.
- **Read in this pass**: what the stage 1 bibliography agent itself fetched. "Full text fetched"
  means it had the text and quoted from it; "abstract fetched" means the abstract only; "snippet or
  secondary" means a search summary, a citing paper or an encyclopedia page. Fetching in WP-0 does
  not change these lines and does not make any ledger row `verified`: only reading the value in the
  source does.
- **Reference check**: whether the citation as written matches the real work (authors, year,
  venue, equation and table numbers), and whether the work appears to contain what is attributed to
  it.

Where stage 1 had entered one work under two keys in two sections, the entry below carries both
sections' lines, marked with the section in parentheses where they differ.

The values themselves are checked in [spot-checks.md](spot-checks.md).

Limits of this pass. Stage 1's web search budget ran out partway, so many sources were identified
from bibliographic databases without being opened. WP-0 fetched by direct request only (DOIs,
publisher and agency pages, repositories, Crossref, Europe PMC, Semantic Scholar and, until its
daily allowance ran out, OpenAlex); publishers that refuse scripted access (Wiley, Elsevier, Science,
PNAS, AMS and a few repositories) could not be fetched even where the work is open access, and those
are marked as such. Scanned papers have no text layer; where OCR text exists it is named. No shadow
library was used. Nothing in the library is checked against the printed version.

## Counts

385 works. Stage 1 listed 475 entries and the flexure feature added 13; 103 entries were merged into 385 works (the merged keys are listed below).

By access, as recorded for the work (the most open access class among its merged entries):

| | Works |
|---|---|
| open access | 93 |
| open copy | 49 |
| paywalled | 163 |
| book (not online) | 34 |
| other, software, data or web page | 28 |
| not found | 18 |
| **all** | **385** |

By what the project's source library now holds (WP-0; the library is on the test machine and is not distributed):

| | Works |
|---|---|
| full text fetched, with a text layer or as HTML/XML text | 104 |
| fetched, scan without a text layer | 6 |
| landing page only (abstract or page shell) | 35 |
| not fetched | 240 |
| **all** | **385** |

Library status by access class:

| access | text | scan | landing page only | not fetched |
|---|---|---|---|---|
| open access | 63 | 0 | 8 | 22 |
| open copy | 30 | 1 | 9 | 9 |
| paywalled | 7 | 0 | 10 | 146 |
| book | 1 | 0 | 2 | 31 |
| other | 1 | 0 | 5 | 22 |
| not found | 2 | 5 | 1 | 10 |

Of the paywalled works, 7 now have a repository or open copy in the library (a substitute for the printed version) and 146 have none.

## Merged keys

One entry per work. These are the keys stage 1 used that no longer exist as entries; ledger ids on
them were combined into the entry named.

| Old key (section) | Now | Note |
|---|---|---|
| GrieveMasaitis1994 (IMP) | Grieve1994 | same work |
| Courtillot2003 (VOL, MAG) | Courtillot2003a | split: two different works that shared one key |
| Barnes2014a (HYD, YNG) | Barnes2014 | same work |
| Clark2000 (ERO, ISO, HYP) | ClarkRoyden2000 | same work |
| England1982 (ERO, ISO, HYP) | EnglandMcKenzie1982 | same work |
| Molnar1988 (ERO, ISO, HYP) | MolnarLyonCaen1988 | same work |
| Stein1992 (ERO, ISO, HYP) | SteinStein1992 | same work |
| Parsons1977 (ERO, ISO, HYP) | ParsonsSclater1977 | same work |
| Peel2007 (Koppen-Geiger) (SCL, PRV, POL, BAK) | Peel2007 | same work |
| Charette2010 (ERO, ISO, HYP) | CharetteSmith2010 | same work |
| Singer2008 (ORE) | Singer2008a | same work |
| Sillitoe1996 (ORE) | SillitoeMcKee1996 | same work |
| USGS2000 (DEP) | USGS2000a | split: two different reports that shared one key |
| USGS2000 (ORE) | USGS2000b | split: two different reports that shared one key |
| Held2006 (CLM) | HeldSoden2006 | same work |
| Berner2001 (CLM) | BernerKothavala2001 | same work |
| DeCellesGiles1996_flexure (ISO) | DeCellesGiles1996 | same work |
| Courtillot2003 (PLT) | Courtillot2003b | split: two different works that shared one key |
| Fu1981 and Zhang2004 | Fu1981 + Zhang2004 | joint entry, divided between the works named |
| MiddletonThomas1997 and Zomer2022 | MiddletonThomas1997 + Zomer2022 | joint entry, divided between the works named |
| Fick2017 and ODonnell2012 (BIO15) | Fick2017 + ODonnell2012 | joint entry, divided between the works named |
| Minder2010 and Rolland2003 (observed lapse rates, not cited by the project) | Minder2010 + Rolland2003 | joint entry, divided between the works named |

Keys that appeared in more than one section and were simply combined (same key, same work): 74 works. Their entries below list the ledger ids from every section.

## Unidentified and not found

- `Gilbert1893`: Identified and read: the paper is the Presidential address of 1892, printed in Bulletin of the Philosophical Society of Washington 12 (1892-1894), p. 241 onwards (the volume's own contents page lists 'The Moon's Face', G. K. Gilbert, p. 241). The volume is on the Internet Archive (public domain; the Biodiversity Heritage Library records no copyright restriction) and is in the library with its text.
- `GaultWedekind1978`: Retrieved from the NASA ADS page scans (conference proceedings, scan only; OCR text in the library). Citation details unchanged.
- `McKinnonSchenk1985`: Retrieved from the NASA ADS page scan (two-page abstract; OCR text in the library).
- `Pike1980`: Retrieved from the NASA ADS page scans (proceedings paper; OCR text in the library).
- `WoodHead1976`: Retrieved from the NASA ADS page scans (proceedings paper; OCR text in the library).
- `Moore1974`: Retrieved from the NASA ADS page scans (proceedings paper; OCR text in the library).
- `GrieveCintala1992`: Identified: Meteoritics 27(5):526-538, DOI 10.1111/j.1945-5100.1992.tb01074.x (Crossref, 2026-10-02). No copy was obtained: the ADS download returned an anti-robot page and the Wiley page refuses scripted access.
- `Donofrio1998`: Not resolved. Oil and Gas Journal 96(19), 1998; not in Crossref and not online. A trade-press article, so no lawful open copy is expected.
- `Shepard`: Probably the same work as Shepard1963 (Submarine Geology, 2nd edition, 1963), the only Shepard in the bibliography; the code names no year or title, so this is a guess and not a match.
- `Fu1981`: Not resolved beyond author and year: Crossref has no record of a 1981 paper by B. P. Fu in Scientia Atmospherica Sinica. Zhang et al. 2004 (key Zhang2004) cite it as the source of the one-parameter Budyko form and are the route to the equation.
- `Software`: Identified from its own reference field: three separate works (Ellefsen 2017, USGS Techniques and Methods 7-C14, the MapMark4 user's guide; Duval 2012, USGS Open-File Report 2004-1344, EMINERS; and the EIS Toolkit software). Stage 1 kept them under one key; they were not split because no ledger row distinguishes them.
- `Lopatin1971 (related; not cited by the code)`: Russian journal, not online; the method is documented in Waples1980 (paywalled) and in the USGS reports in the library.
- `NMET-Jharia`: Not resolved. NMET is India's National Mineral Exploration Trust; the proposal is not a published work and Crossref has no record of it.
- `SAIMM-coalfields`: Identified and read: Jeffrey, L. S. (2005), Characterization of the coal resources of South Africa, Journal of the Southern African Institute of Mining and Metallurgy 105(2):95-102. Its text says 'Coal is found in South Africa in 19 coalfields' and has a Table I of seam information per coalfield; open PDF at https://www.saimm.co.za/Journal/v105n02p095.pdf. The year follows from the volume number (105 is 2005); the extracted text of the first page carries no year.
- `SilesiaMDPI`: Not resolved. A Crossref search for Upper Silesian seam or cyclothem counts in an MDPI journal found no clear match.
- `RuhrCounts`: Not resolved. Hampson, Stollhofen and Flint 1999 (Sedimentology 46, DOI 10.1046/j.1365-3091.1999.00273.x; key Hampson1999) is the best candidate for a sequence-stratigraphic count of Ruhr coal seams; nothing ties 'the Ruhr counts' to it.
- `ICAO1993`: Probably the ICAO Manual of the ICAO Standard Atmosphere (extended to 80 kilometres), Doc 7488, third edition, 1993, the document the 6.5 K/km standard lapse rate is attributed to; the project never named it and it was not seen.
- `Lallemand_EncMarineGeosciences`: Probably the entry 'Active Continental Margin' by S. Lallemand in the Encyclopedia of Marine Geosciences (Springer, 2014), DOI 10.1007/978-94-007-6644-0_102-2 (Crossref); the entry 'Subduction' (DOI 10.1007/978-94-007-6644-0_103-1) is the other candidate. Not read; paywalled.
- `Tetens`: Not resolved: Tetens, O. (1930), Zeitschrift fuer Geophysik 6:297-309 is the likely work (an old German journal). The saturation-vapour formula in common use is quoted in many open textbooks.

---

## Meteor impacts (IMP)

### Collins2005
- Reference: Collins, G. S., Melosh, H. J., and Marcus, R. A. (2005). Earth Impact Effects Program: A Web-based computer program for calculating the regional environmental consequences of a meteoroid impact on Earth. Meteoritics & Planetary Science 40(6):817-840.
- DOI or URL: 10.1111/j.1945-5100.2005.tb00157.x. Open copy (Imperial College Spiral, hdl 10044/1/11554, file CollinsEtAl2005.pdf, 1.6 MB): https://spiral.imperial.ac.uk/bitstreams/8065368c-096c-4787-aab7-cbaf1a7d7277/download . The program's own page links the paper as `static/effects.pdf` on https://impact.ese.ic.ac.uk/ImpactEarth/ImpactEffects/ (the older path /ImpactEffects/effects.pdf returned 404; Wiley returned 403).
- Cited as: "Collins, Melosh & Marcus 2005", "Collins et al. 2005, eq. N" (eqs. 12, 21, 22, 24, 25, 26, 27, 28, 31, 32 in code comments).
- Ledger ids: IMP-020, IMP-023, IMP-029 to IMP-032, IMP-037 to IMP-042, IMP-054, IMP-062, IMP-063
- Access: open copy at the Spiral URL above.
- Library (the project's source library, not distributed): fetched (PDF, 24 pages, text layer). From: https://spiral.imperial.ac.uk/bitstreams/8065368c-096c-4787-aab7-cbaf1a7d7277/download. Text file: collins2005.txt.
- Read so far (by the project): from memory ("the paper could not be fetched").
- Read in this pass: full text fetched (all 24 pages; read in detail: Introduction, "Crater dimensions and melt production" pp. 822-825, "Ejecta deposit" pp. 828-829, the worked example and Table 6, the reference list).
- Reference check: authors, year, venue, volume(issue) and pages are right. Equation numbering in the paper marks the equations the program implements with an asterisk. Of the numbers the code cites: eqs. 21, 22, 24, 26, 27, 28, 31 are right (but see the value MISMATCH for eq. 28); the melt law is eq. 30 (code says 12; paper eq. 12 is in the atmospheric-entry section); the rim height is eq. 48 (code says 25; paper eq. 25 is transient depth d_tc = D_tc/(2 sqrt 2)); the ejecta thickness is eq. 47 (code says 32; paper eq. 32 is the fireball radius); V_br = 0.032 D^3 is eq. 23 (code groups it under "eqs. 24 and 26"). The complex-crater depth law in eq. 28 is 0.4 D^0.3, not 0.294 D^0.301 (Part 2, IMP-042). The paper credits its eq. 21 to Holsapple & Schmidt 1982, Schmidt & Housen 1987 and Gault 1974, not to Gault & Wedekind 1978 or Chapman & McKinnon 1986, and it has no default impact speed (Part 2, IMP-020).

### GrieveShoemaker1994
- Reference: Grieve, R. A. F. and Shoemaker, E. M. (1994). The record of past impacts on Earth. In: Gehrels, T. (ed.), Hazards Due to Comets and Asteroids, University of Arizona Press, Tucson, pp. 417-462.
- DOI or URL: no DOI found. Internet Archive lending copy of the book (not text-searchable by me): https://archive.org/details/hazardsduetocome0000unse (year 1994 in its metadata).
- Cited as: "Grieve & Shoemaker 1994".
- Ledger ids: IMP-001 to IMP-003, IMP-008, IMP-019
- Access: book (not online); lending copy at archive.org was not readable (403 on the search-inside service, djvu text withheld).
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://archive.org/details/hazardsduetocome0000unse. File: GrieveShoemaker1994.txt.
- Read so far (by the project): secondary ("checked against the literature by a search"); the scaling.rs header says "as remembered".
- Read in this pass: not fetched (tried archive.org; no text obtained).
- Reference check: pages 417-462 and title match the reference list of Davison & Collins 2007, which dates the chapter 1995; Collins et al. 2005 date the same volume 1994 (Bottke et al. 1994 in it). The book is 1994 (archive.org). The value 5.6 +/- 2.8 was NOT found in any text I could read; the older figure 5.4 +/- 2.7 x 10^-15 km^-2 yr^-1 (D >= 20 km, N = 7, Grieve 1984) is quoted in Grieve 1987 (see Grieve1987).

### Neukum2001
- Reference: Neukum, G., Ivanov, B. A., and Hartmann, W. K. (2001). Cratering Records in the Inner Solar System in Relation to the Lunar Reference System. Space Science Reviews 96(1-4):55-86.
- DOI or URL: 10.1023/A:1011989004263 (Crossref-verified)
- Cited as: "Neukum, Ivanov & Hartmann 2001"
- Ledger ids: IMP-005
- Access: paywalled (Springer; only Crossref metadata seen)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched (metadata only)
- Reference check: authors, year, venue, volume and pages match the real paper. Whether it says the rate was "constant within a factor of two for the last ~3 Gyr" is not checked.

### Bland2006
- Reference: Bland, P. A. and Artemieva, N. A. (2006). The rate of small impacts on Earth. Meteoritics & Planetary Science 41(4):607-631.
- DOI or URL: 10.1111/j.1945-5100.2006.tb00485.x (Crossref-verified)
- Cited as: "Bland & Artemieva 2006"
- Ledger ids: IMP-014
- Access: paywalled (Wiley; metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched (metadata only)
- Reference check: authors, year, venue, volume and pages match. The topic (small impactors broken up by the atmosphere) fits the claim; Davison & Collins 2007 cite it and say impactors "much smaller than 100 m ... are significantly affected by atmospheric entry" (full text read). The actual statement about stony bodies is not checked.

### LeFeuvre2011
- Reference: Le Feuvre, M. and Wieczorek, M. A. (2011). Nonuniform cratering of the Moon and a revised crater chronology of the inner Solar System. Icarus 214(1):1-20.
- DOI or URL: 10.1016/j.icarus.2011.03.010 (Crossref-verified). Open copies listed by OpenAlex: https://hal.science/hal-00768797 (green OA; the document URL returned "Access Denied" to me), http://hdl.handle.net/2262/65826 (Trinity College Dublin TARA; not tried).
- Cited as: "Le Feuvre & Wieczorek 2011"
- Ledger ids: IMP-020
- Access: open copy (HAL, green) at https://hal.science/hal-00768797
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): from memory
- Read in this pass: not fetched (HAL access denied)
- Reference check: authors, year, venue, volume and pages match. Whether it gives 20 km/s as the Earth mean for asteroids is not checked.

### Gilbert1893
- Reference: Gilbert, G. K. (1893). The Moon's face: a study of the origin of its features. Bulletin of the Philosophical Society of Washington 12:241-292. [Details from my recollection, not verified in this pass; the sources file gives only author and year.]
- DOI or URL: none found
- Cited as: "Gilbert 1893" (with Shoemaker 1962, for the sin 2 theta angle distribution)
- Ledger ids: IMP-024
- Access: not found (not searched)
- Library (the project's source library, not distributed): fetched (PDF, 664 pages, text layer, 1,200,000 characters). From: https://archive.org/details/bulletinofp1218921894phil (Internet Archive scan of Bulletin of the Philosophical Society of Washington vol. 12, 1892-1894; the paper starts on p. 241 of the volume; the text is the archive's own OCR). Text file: Gilbert1893.vol12.txt.
- Substitute: repository or open copy found by the lookups and saved as above (version not stated: the first pages were matched against the title, not compared with the printed version).
- Identification (stage 2, WP-0): Identified and read: the paper is the Presidential address of 1892, printed in Bulletin of the Philosophical Society of Washington 12 (1892-1894), p. 241 onwards (the volume's own contents page lists 'The Moon's Face', G. K. Gilbert, p. 241). The volume is on the Internet Archive (public domain; the Biodiversity Heritage Library records no copyright restriction) and is in the library with its text.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: Collins et al. 2005 cite only Shoemaker 1962 for the 45-degree most likely angle; they do not cite Gilbert.

### Shoemaker1962
- Reference: Shoemaker, E. M. (1962). Interpretation of lunar craters. In: Kopal, Z. (ed.), Physics and Astronomy of the Moon, Academic Press, New York, pp. 283-359. (As in the reference list of Collins et al. 2005.)
- DOI or URL: none
- Cited as: "Shoemaker 1962"
- Ledger ids: IMP-024
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. 2005 (full text) say "The most likely impact angle is 45 degrees (Shoemaker 1962)".
- Reference check: consistent with Collins et al.'s citation. The dP = sin 2 theta d theta form is not stated in Collins et al.

### SchmidtHousen1987
- Reference: Schmidt, R. M. and Housen, K. R. (1987). Some recent advances in the scaling of impact and explosion cratering. International Journal of Impact Engineering 5(1-4):543-560.
- DOI or URL: 10.1016/0734-743X(87)90069-8 (Crossref-verified)
- Cited as: "Schmidt & Housen 1987"
- Ledger ids: IMP-030
- Access: paywalled (metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: matches. Collins et al. 2005 name it (with Holsapple & Schmidt 1982, JGR 87:1849-1870, and Gault 1974) as the basis of eq. 21 and as the source of the water constant 1.365.

### GaultWedekind1978
- Reference: Gault, D. E. and Wedekind, J. A. (1978). Experimental studies of oblique impact. Proceedings of the 9th Lunar and Planetary Science Conference, pp. 3843-3875. [Recollection, not verified.]
- DOI or URL: none found
- Cited as: "Gault & Wedekind 1978" (for the sin^(1/3) theta factor)
- Ledger ids: IMP-030
- Access: not found (not searched)
- Library (the project's source library, not distributed): fetched (PDF, 33 pages, no text layer: scan). From: https://articles.adsabs.harvard.edu/pdf/1978LPSC....9.3843G. Text file: GaultWedekind1978.txt (empty or nearly so). OCR text (machine-read, errors likely): GaultWedekind1978.ocr.txt.
- Identification (stage 2, WP-0): Retrieved from the NASA ADS page scans (conference proceedings, scan only; OCR text in the library). Citation details unchanged.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: Collins et al. 2005 do not cite this paper; they cite Gault 1974 (Impact cratering, in A Primer in Lunar Geology, NASA Ames, pp. 137-175). They do not attribute the sin^(1/3) theta factor to anyone.

### ChapmanMcKinnon1986
- Reference: Chapman, C. R. and McKinnon, W. B. (1986). Cratering of planetary satellites. In: Burns, J. A. and Matthews, M. S. (eds), Satellites, University of Arizona Press, pp. 492-580. [Recollection, not verified.]
- DOI or URL: none found
- Cited as: "Chapman & McKinnon 1986"
- Ledger ids: IMP-030
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: not cited by Collins et al. 2005 (see GaultWedekind1978).

### McKinnonSchenk1985
- Reference: McKinnon, W. B. and Schenk, P. M. (1985). Ejecta blanket scaling on the Moon and Mercury: inferences for projectile populations (abstract). Proceedings of the 16th Lunar and Planetary Science Conference, pp. 544-545. (As in the reference list of Collins et al. 2005.)
- DOI or URL: none
- Cited as: "McKinnon & Schenk 1985 (as in Holsapple 1993)"
- Ledger ids: IMP-032
- Access: not found (conference abstract; not looked up)
- Library (the project's source library, not distributed): fetched (PDF, 2 pages, no text layer: scan). From: https://articles.adsabs.harvard.edu/pdf/1985LPI....16..544M. Text file: McKinnonSchenk1985.txt (empty or nearly so). OCR text (machine-read, errors likely): McKinnonSchenk1985.ocr.txt.
- Identification (stage 2, WP-0): Retrieved from the NASA ADS page scan (two-page abstract; OCR text in the library).
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. 2005 (full text) say eq. 27 is the "functional form established by McKinnon and Schenk (1985), which lies intermediate between the estimates of Croft (1985) and Holsapple (1993)".
- Reference check: Collins et al. cite it directly, so "as in Holsapple 1993" is the code's own gloss. Holsapple 1993 is Annu. Rev. Earth Planet. Sci. 21:333-373, DOI 10.1146/annurev.ea.21.050193.002001 (Crossref-verified).

### GrieveDenceRobertson1977
- Reference: Grieve, R. A. F., Dence, M. R., and Robertson, P. B. (1977). Cratering processes: as interpreted from the occurrence of impact melts. In: Roddy, D. J., Pepin, R. O., and Merrill, R. B. (eds), Impact and Explosion Cratering, Pergamon Press, New York, pp. 791-814. (Reference list of Collins et al. 2005.) A companion: Dence, M. R., Grieve, R. A. F., and Robertson, P. B. (1977), Terrestrial impact structures: principal characteristics and energy considerations, same volume, pp. 247-275.
- DOI or URL: none
- Cited as: "Grieve, Dence & Robertson 1977"
- Ledger ids: IMP-039, IMP-052
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. use Grieve et al. 1977 for the melt-and-breccia lens (p. 823) and the uniform melt sheet (p. 825), and Dence et al. 1977 for d_tc = D_tc/(2 sqrt 2).
- Reference check: Fine for the lens; the claim "complex-crater fill thickness equals the crater's depth" (IMP-052) is not in Collins et al. and is not checked.

### Melosh1989
- Reference: Melosh, H. J. (1989). Impact Cratering: A Geologic Process. Oxford University Press, New York, 245 pp. (Reference list of Collins et al. 2005.)
- DOI or URL: none
- Cited as: "Melosh 1989" (collapse and lens, transition, excavation depth, section 6.3, fig. 8.1)
- Ledger ids: IMP-036, IMP-039, IMP-053, IMP-058, IMP-128
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: real book. Section and figure numbers not checked.

### Grieve1987
- Reference: Grieve, R. A. F. (1987). Terrestrial impact structures. Annual Review of Earth and Planetary Sciences 15:245-270.
- DOI or URL: 10.1146/annurev.ea.15.050187.001333 (Crossref-verified). Open scan with OCR text: https://www.wipp.energy.gov/library/cra/2009_cra/references/Others/Grieve_1987_Terrestrial_Impact_Structures_ERMS241228.pdf
- Cited as: "Grieve 1987"
- Ledger ids: IMP-034, IMP-035
- Access: open copy (scan) at the WIPP URL above
- Library (the project's source library, not distributed): fetched (PDF, 26 pages, text layer). From: https://www.wipp.energy.gov/library/cra/2009_cra/references/Others/Grieve_1987_Terrestrial_Impact_Structures_ERMS241228.pdf. Text file: grieve1987.txt.
- Read so far (by the project): from memory
- Read in this pass: full text fetched (OCR layer; read the morphology section, the cratering-rate section and the size-frequency section)
- Reference check: venue, volume and pages match. It supports the 2 km / 4 km split (IMP-035) but I found no 3.2 km in its OCR text, so it is not the source of the 3.2 km figure (IMP-034); Collins et al. give 3.2 km, classified by Dence 1965. It also contains the older rate figure used as a secondary check on IMP-001.

### Pike1980
- Reference: Pike, R. J. (1980). Control of crater morphology by gravity and target type: Mars, Earth, Moon. Proceedings of the 11th Lunar and Planetary Science Conference (Geochimica et Cosmochimica Acta Supplement 14), pp. 2159-2189. (Collins et al. print the pages as 2159-2190 and the series as "Geochimica et Cosmochimica Acta 3".)
- DOI or URL: none found
- Cited as: "Pike 1980"
- Ledger ids: IMP-036
- Access: not found (an ADS scan probably exists; not fetched. The ADS scan of Pike 1985 had no text layer, so this one likely has none either)
- Library (the project's source library, not distributed): fetched (PDF, 31 pages, no text layer: scan). From: https://articles.adsabs.harvard.edu/pdf/1980LPSC...11.2159P. Text file: Pike1980.txt (empty or nearly so). OCR text (machine-read, errors likely): Pike1980.ocr.txt.
- Identification (stage 2, WP-0): Retrieved from the NASA ADS page scans (proceedings paper; OCR text in the library).
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: title and series fit the claim (gravity control of the transition). Collins et al. cite it only for terrestrial complex-crater depth data. Page end differs between the code's source list (2189) and Collins et al. (2190).

### AlexopoulosMcKinnon1994
- Reference: Alexopoulos, J. S. and McKinnon, W. B. (1994). Large impact craters and basins on Venus, with implications for ring mechanics on the terrestrial planets. In: Dressler, B. O., Grieve, R. A. F., and Sharpton, V. L. (eds), Large Meteorite Impacts and Planetary Evolution, Geological Society of America Special Paper 293, pp. 29-50.
- DOI or URL: 10.1130/SPE293-p29 (Crossref-verified; Crossref dates the chapter 1992, the volume is 1994). GeoScienceWorld page returned 403: https://pubs.geoscienceworld.org/books/book/440/chapter/3798885/
- Cited as: "Alexopoulos & McKinnon 1994"
- Ledger ids: IMP-045
- Access: paywalled (not even an abstract seen)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched (403)
- Reference check: title, authors, volume and pages match. Whether it gives about 40 km for the onset of peak rings on Venus is unchecked. The editors and the year 1994 for SP 293 are from my recollection plus the code's own year; Crossref's 1992 is a discrepancy to settle.

### WoodHead1976
- Reference: Wood, C. A. and Head, J. W. (1976). Comparison of impact basins on Mercury, Mars and the Moon. Proceedings of the 7th Lunar Science Conference, pp. 3629-3651. [Recollection, not verified.]
- DOI or URL: none found
- Cited as: "Wood & Head 1976"
- Ledger ids: IMP-046
- Access: not found (not searched)
- Library (the project's source library, not distributed): fetched (PDF, 23 pages, no text layer: scan). From: https://articles.adsabs.harvard.edu/pdf/1976LPSC....7.3629W. Text file: WoodHead1976.txt (empty or nearly so). OCR text (machine-read, errors likely): WoodHead1976.ocr.txt.
- Identification (stage 2, WP-0): Retrieved from the NASA ADS page scans (proceedings paper; OCR text in the library).
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: unverified.

### PikeSpudis1987
- Reference: Pike, R. J. and Spudis, P. D. (1987). Basin-ring spacing on the Moon, Mercury, and Mars. Earth, Moon, and Planets 39(2):129-194.
- DOI or URL: 10.1007/BF00054060 (Crossref-verified)
- Cited as: "Pike & Spudis 1987"
- Ledger ids: IMP-046
- Access: paywalled (metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 190 chars).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: real paper. It concerns multi-ring basins; the claim "ring diameter about half the rim's" for peak-ring craters is unchecked.

### Pike1985
- Reference: Pike, R. J. (1985). Some morphologic systematics of complex impact structures. Meteoritics 20(1):49-68.
- DOI or URL: 10.1111/j.1945-5100.1985.tb00846.x (Crossref-verified). ADS scan (JBIG2 images, 20 pages, no text layer; unreadable by my tools): https://articles.adsabs.harvard.edu/pdf/1985Metic..20...49P
- Cited as: "Pike 1985, Meteoritics 20:49-68"
- Ledger ids: IMP-048
- Access: open copy (scan only) at the ADS URL
- Library (the project's source library, not distributed): fetched (PDF, 20 pages, no text layer: scan). From: https://articles.adsabs.harvard.edu/pdf/1985Metic..20...49P. Text file: pike1985.txt. OCR text (machine-read, errors likely): pike1985.ocr.txt.
- Read so far (by the project): from memory
- Read in this pass: fetched but unreadable (image-only scan)
- Reference check: venue, volume and pages match; title found by Crossref.

### Pike1977
- Reference: Pike, R. J. (1977). Size-dependence in the shape of fresh impact craters on the Moon. In: Roddy, D. J., Pepin, R. O., and Merrill, R. B. (eds), Impact and Explosion Cratering, Pergamon Press, New York, pp. 489-509. [Recollection, not verified.]
- DOI or URL: none
- Cited as: "Pike 1977" (comment only)
- Ledger ids: IMP-050
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: unverified; the code uses it only in comments.

### McGetchin1973
- Reference: McGetchin, T. R., Settle, M., and Head, J. W. (1973). Radial thickness variation in impact crater ejecta: implications for lunar basin deposits. Earth and Planetary Science Letters 20(2):226-236.
- DOI or URL: 10.1016/0012-821X(73)90162-3 (Crossref-verified)
- Cited as: "McGetchin, Settle & Head 1973"
- Ledger ids: IMP-054, IMP-055
- Access: paywalled (metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. (full text) cite it: "The power of 3 is a good approximation of data from explosion experiments (McGetchin et al. 1973)... the power can vary between 2.5 and 3.5."
- Reference check: matches. Collins et al. eq. 47 is not "McGetchin" itself but their own volume-conservation model using the r^-3 falloff.

### Moore1974
- Reference: Moore, H. J., Hodges, C. A., and Scott, D. H. (1974). Multiringed basins: illustrated by Orientale and associated features. Proceedings of the 5th Lunar Science Conference, pp. 71-100. [Recollection, not verified; the title suggests the paper is about large basins, which may not be the source of a 2.3-radii blanket limit for ordinary craters.]
- DOI or URL: none found
- Cited as: "Moore, Hodges & Scott 1974"
- Ledger ids: IMP-057
- Access: not found (not searched)
- Library (the project's source library, not distributed): fetched (PDF, 30 pages, no text layer: scan). From: https://articles.adsabs.harvard.edu/pdf/1974LPSC....5...71M. Text file: Moore1974.txt (empty or nearly so). OCR text (machine-read, errors likely): Moore1974.ocr.txt.
- Identification (stage 2, WP-0): Retrieved from the NASA ADS page scans (proceedings paper; OCR text in the library).
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: unverified; flag for stage 2.

### GrieveCintala1992
- Reference: Grieve, R. A. F. and Cintala, M. J. (1992). An analysis of differential impact melt-crater scaling and implications for the terrestrial impact record. Meteoritics 27:526-538. (Collins et al. print the title as "An analysis of differential melt-crater scaling and implications for the terrestrial impact record".)
- DOI or URL: none looked up
- Cited as: "Grieve & Cintala 1992"
- Ledger ids: IMP-062
- Access: not found (not looked up)
- Library (the project's source library, not distributed): not fetched. Reason: the download was a web page or an anti-robot interstitial, not the paper.
- Identification (stage 2, WP-0): Identified: Meteoritics 27(5):526-538, DOI 10.1111/j.1945-5100.1992.tb01074.x (Crossref, 2026-10-02). No copy was obtained: the ADS download returned an anti-robot page and the Wiley page refuses scripted access.
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. cite it (with Pierazzo et al. 1997 and Pierazzo & Melosh 2000) as support for the melt volume.
- Reference check: venue, volume and pages as in Collins et al.'s list.

### PierazzoVickery1997
- Reference: Pierazzo, E., Vickery, A. M., and Melosh, H. J. (1997). A reevaluation of impact melt production. Icarus 127(2):408-423.
- DOI or URL: 10.1006/icar.1997.5713 (Crossref-verified)
- Cited as: "Pierazzo, Vickery & Melosh 1997"
- Ledger ids: IMP-062
- Access: paywalled (metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched; Collins et al. take epsilon_m = 5.2 MJ/kg for granite "(see Pierazzo et al. 1997)".
- Reference check: matches. Collins et al. also cite Pierazzo & Melosh 2000 (Icarus 145:252-261) for the angle dependence.

### DavisonCollins2007
- Reference: Davison, T. and Collins, G. S. (2007). The effect of the oceans on the terrestrial crater size-frequency distribution: Insight from numerical modeling. Meteoritics & Planetary Science 42(11):1915-1927.
- DOI or URL: 10.1111/j.1945-5100.2007.tb00550.x. Open copy (Imperial Spiral, hdl 10044/1/11559, DavisonCollins2007.pdf): https://spiral.imperial.ac.uk/server/api/core/bitstreams/ba3d1b52-99e7-4439-926d-7a38cdfb3e7c/content . Another repository copy: University of Arizona, http://hdl.handle.net/10150/656356 .
- Cited as: "Davison & Collins 2007"
- Ledger ids: IMP-065 to IMP-067
- Access: open copy at the Spiral URL
- Library (the project's source library, not distributed): fetched (PDF, 13 pages, text layer). From: https://spiral.imperial.ac.uk/server/api/core/bitstreams/ba3d1b52-99e7-4439-926d-7a38cdfb3e7c/content. Text file: dc2007.txt.
- Read so far (by the project): abstract only
- Read in this pass: full text fetched (abstract, introduction, method, results, Table 2, discussion)
- Reference check: the citation is right. The code's note that it fits "three numbers" is accurate for the abstract. The paper's own cutoff is not 6.3 (Part 2).

### Culling1960
- Reference: Culling, W. E. H. (1960). Analytical theory of erosion. The Journal of Geology 68(3):336-344.
- DOI or URL: 10.1086/626663 (Crossref-verified)
- Cited as: "Culling 1960"
- Ledger ids: IMP-072, IMP-083
- Access: paywalled (metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: matches (title found by Crossref). The paper is general erosion theory; it gives no crater-specific diffusivity.

### Soderblom1970
- Reference: Soderblom, L. A. (1970). A model for small-impact erosion applied to the lunar surface. Journal of Geophysical Research 75(14):2655-2661.
- DOI or URL: 10.1029/JB075i014p02655 (Crossref-verified)
- Cited as: "Soderblom 1970"
- Ledger ids: IMP-072
- Access: paywalled (metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: matches. Its title says the diffusion comes from small impacts on the Moon, not soil creep.

### FassettThomson2014
- Reference: Fassett, C. I. and Thomson, B. J. (2014). Crater degradation on the lunar maria: Topographic diffusion and the rate of erosion on the Moon. Journal of Geophysical Research: Planets 119(10):2255-2271.
- DOI or URL: 10.1002/2014JE004698 (Crossref-verified). OpenAlex lists no open copy.
- Cited as: "Fassett & Thomson 2014"
- Ledger ids: IMP-072, IMP-083
- Access: paywalled (abstract only, via OpenAlex)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: abstract fetched (OpenAlex reconstruction of the abstract; wording has gaps)
- Reference check: matches. It is a LUNAR study (diffusivity about 5.5 m^2/Myr), not an Earth hillslope value (Part 2, IMP-083).

### Fernandes1997
- Reference: Fernandes, N. F. and Dietrich, W. E. (1997). Hillslope evolution by diffusive processes: The timescale for equilibrium adjustments. Water Resources Research 33(6):1307-1318.
- DOI or URL: 10.1029/97WR00534 (Crossref-verified)
- Cited as: "Fernandes & Dietrich 1997"
- Ledger ids: IMP-074
- Access: paywalled (metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: not fetched
- Reference check: matches. Whether it gives the 0.003-0.05 m^2/yr range is unchecked.

### Roering1999
- Reference: Roering, J. J., Kirchner, J. W., and Dietrich, W. E. (1999). Evidence for nonlinear, diffusive sediment transport on hillslopes and implications for landscape morphology. Water Resources Research 35(3):853-870.
- DOI or URL: (IMP) 10.1029/1998WR900090 (Crossref-verified) (ERO, ISO, HYP) 10.1029/1998WR900090.
- Cited as: (IMP) "Roering, Kirchner & Dietrich 1999" (ERO, ISO, HYP) "Roering, Kirchner & Dietrich 1999, WRR 35(3):853-870" (rock_class JSON), "hillslope transport coefficient D". (RCK, STR, DEF) "Roering, Kirchner & Dietrich 1999 (WRR 35(3):853-870)"
- Ledger ids: ERO-101 to ERO-150, ERO-153 to ERO-163, IMP-074, IMP-085, IMP-086, IMP-089, IMP-090, RCK-010
- Access: paywalled (metadata only) [other sections: paywalled (Wiley); not fetched. / paywalled (abstract only)]
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): (IMP) from memory (params.rs); judgment framework (rock-class JSON) (ERO, ISO, HYP) no record. (RCK, STR, DEF) not stated
- Read in this pass: (IMP) not fetched (ERO, ISO, HYP) not fetched (403 / no open copy). (RCK, STR, DEF) abstract fetched (OpenAlex)
- Reference check: (IMP) matches the rock-class JSON (WRR 35(3):853-870). Note this paper's point is NONLINEAR diffusion, while the code uses linear diffusion. (ERO, ISO, HYP) Crossref matches. The title says "nonlinear" transport. The JSON text uses it as "a linear creep multiplier", which is a use of its linear coefficient D, whereas the paper's contribution is a nonlinear law with a critical slope; the code's linear diffusion is the small-slope limit. The paper's numeric D values were not read. (RCK, STR, DEF) authors, year, volume, issue, pages match. The paper's finding is a nonlinear (gradient-dependent) transport law, not a lithology-dependent linear coefficient; it is cited only for the idea of a creep coefficient D.

### Sklar2001
- Reference: Sklar, L. S. and Dietrich, W. E. (2001). Sediment and rock strength controls on river incision into bedrock. Geology 29(12):1087-1090.
- DOI or URL: (IMP) 10.1130/0091-7613(2001)029<1087:SARSCO>2.0.CO;2 (Crossref-verified) (ERO, ISO, HYP) 10.1130/0091-7613(2001)029<1087:SARSCO>2.0.CO;2.
- Cited as: (IMP) "Sklar & Dietrich 2001" (ERO, ISO, HYP) "Sklar & Dietrich 2001, Geology 29(12):1087-1090 (river incision scales roughly with tensile strength^-2)". (RCK, STR, DEF) "Sklar & Dietrich 2001, Geology 29(12):1087-1090"
- Ledger ids: ERO-097, ERO-102 to ERO-150, ERO-153 to ERO-163, IMP-085, IMP-086, IMP-089, IMP-090
- Access: paywalled (metadata only) [other sections: paywalled (GeoScienceWorld 403); not fetched. / paywalled (abstract only)]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (IMP) no statement; numbers labelled "judgment" (ERO, ISO, HYP) no record. (RCK, STR, DEF) not stated
- Read in this pass: (IMP) not fetched (RCK, STR, DEF) abstract fetched (OpenAlex, https://api.openalex.org/works/doi:10.1130/0091-7613(2001)029<1087:SARSCO>2.0.CO;2)
- Reference check: (IMP) matches. (ERO, ISO, HYP) Crossref matches title, authors, volume 29, issue 12, start page 1087; the end page 1090 is in the GeoScienceWorld URL (`/29/12/1087-1090/`). The attributed content (inverse square of tensile strength) is not checked. (RCK, STR, DEF) authors, year, journal, volume, issue and pages match exactly. The abstract supports the attributed claim (see spot check RCK-006).

### Stock1999
- Reference: Stock, J. D. and Montgomery, D. R. (1999). Geologic constraints on bedrock river incision using the stream power law. Journal of Geophysical Research: Solid Earth 104(B3):4983-4993.
- DOI or URL: (IMP) 10.1029/98JB02139 (Crossref-verified) (ERO, ISO, HYP) 10.1029/98JB02139.
- Cited as: (IMP) "Stock & Montgomery 1999" (ERO, ISO, HYP) "Stock & Montgomery 1999, JGR 104(B3):4983-4993 (lithologic K contrasts span orders of magnitude)". (RCK, STR, DEF) "Stock & Montgomery 1999, JGR 104(B3):4983-4993"
- Ledger ids: ERO-097, ERO-101 to ERO-150, IMP-085, IMP-086, IMP-089, IMP-090
- Access: paywalled (metadata only) [other sections: paywalled (abstract elided at Semantic Scholar; Wiley 403). / paywalled (abstract only)]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (IMP) no statement (ERO, ISO, HYP) no record. (RCK, STR, DEF) not stated
- Read in this pass: (IMP) not fetched (RCK, STR, DEF) abstract fetched (OpenAlex)
- Reference check: (IMP) matches. (ERO, ISO, HYP) Crossref matches title, volume 104, issue B3, pages 4983-4993. The content attributed is not checked. Note a trap: the DOI 10.1029/98JB02840 is a different paper (La Reunion seismic transect). (RCK, STR, DEF) authors, year, volume, issue, pages match. Abstract content supports "lithologic K contrasts span orders of magnitude".

### Heimsath1997
- Reference: Heimsath, A. M., Dietrich, W. E., Nishiizumi, K., and Finkel, R. C. (1997). The soil production function and landscape equilibrium. Nature 388(6640):358-361.
- DOI or URL: (IMP) 10.1038/41056 (Crossref-verified) (ERO, ISO, HYP) 10.1038/41056. Page https://www.nature.com/articles/41056 (abstract reachable only through the cookie redirect). (RCK, STR, DEF) 10.1038/41056
- Cited as: (IMP) "Heimsath et al. 1997" (ERO, ISO, HYP) "Heimsath et al. 1997" (weathering.rs, rock_class JSON sources.diffusivity). (RCK, STR, DEF) "Heimsath et al. 1997, Nature 388:358-361"
- Ledger ids: ERO-068 to ERO-070, IMP-086, IMP-090, RCK-010
- Access: paywalled (metadata only) [other sections: paywalled (abstract fetched). / paywalled (abstract only; OpenAlex had no abstract)]
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://www.nature.com/articles/41056.pdf. File: Heimsath1997.txt.
- Read so far (by the project): (IMP) no statement (ERO, ISO, HYP) abstract unavailable ("The Heimsath abstract was unavailable"); rate constants not retrieved. (RCK, STR, DEF) not stated
- Read in this pass: (IMP) not fetched (ERO, ISO, HYP) abstract fetched (full abstract, verbatim in S5). The paper's numerical parameters were not available; they are quoted in S5 from secondary sources. (RCK, STR, DEF) metadata only
- Reference check: (IMP) matches. (ERO, ISO, HYP) Crossref matches authors, title, volume 388, issue 6640, pages 358-361, July 1997. The paper is a northern California (Tennessee Valley) study using 10Be and 26Al in bedrock under soil; the code uses its form but not its numbers (the 5 m depth scale is about ten times too deep, see S5). (RCK, STR, DEF) authors, year, volume, pages match the OpenAlex record.

### Naldrett2004
- Reference: Naldrett, A. J. (2004). Magmatic Sulfide Deposits: Geology, Geochemistry and Exploration. Springer, Berlin. [Subtitle and publisher from my recollection, not verified.]
- DOI or URL: (IMP) none looked up (ORE) 10.1007/978-3-662-08444-1
- Cited as: (IMP) "Naldrett 2004" (ORE) "Naldrett 2004 (Magmatic Sulfide Deposits)".
- Ledger ids: IMP-095, IMP-097, IMP-105, IMP-106, IMP-111
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): (IMP) no statement (ORE) not stated.
- Read in this pass: (IMP) not fetched (ORE) not fetched (Crossref metadata only).
- Reference check: (IMP) unverified. (ORE) matches.

### GrieveStoffler1991
- Reference: Grieve, R. A. F., Stoffler, D., and Deutsch, A. (1991). The Sudbury structure: Controversial or misunderstood? Journal of Geophysical Research: Planets 96(E5):22753-22764.
- DOI or URL: 10.1029/91JE02513 (Crossref-verified)
- Cited as: "Grieve, Stoffler & Deutsch 1991"
- Ledger ids: IMP-095, IMP-097
- Access: paywalled (metadata only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: matches (JGR 96, pages 22753-22764). Whether it gives the 2.5-3 km melt-sheet thickness is unchecked.

### Lightfoot2016
- Reference: Lightfoot, P. C. (2016). Nickel Sulfide Ores and Impact Melts: Origin of the Sudbury Igneous Complex. Elsevier, Amsterdam. [Publisher from my recollection, not verified.]
- DOI or URL: (IMP) none looked up (ORE) no DOI found (a 2017 Economic Geology book review exists: 10.5382/econgeo.112.6.br01).
- Cited as: (IMP) "Lightfoot 2016" (ORE) "Lightfoot 2016, Nickel Sulfide Ores and Impact Melts".
- Ledger ids: IMP-111, ORE-044
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): (IMP) no statement (ORE) not stated.
- Read in this pass: (IMP) not fetched (ORE) not fetched (existence confirmed only through the Crossref book-review record).
- Reference check: (IMP) unverified. (ORE) publisher and page count not confirmed in this pass.

### Abramov2004
- Reference: Abramov, O. and Kring, D. A. (2004). Numerical modeling of an impact-induced hydrothermal system at the Sudbury crater. Journal of Geophysical Research: Planets 109(E10), article E10007.
- DOI or URL: (IMP) 10.1029/2003JE002213 (Crossref-verified) (ORE) 10.1029/2003JE002213
- Cited as: "Abramov & Kring 2004"
- Ledger ids: IMP-098, IMP-099, IMP-113, ORE-042
- Access: paywalled (metadata only) [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (IMP) no statement (ORE) not stated.
- Read in this pass: (IMP) not fetched (ORE) not fetched (Crossref metadata only).
- Reference check: (IMP) matches. Its subject is Sudbury specifically; the generalisation to "10 thousand to 1 million years" (IMP-099) is unchecked. (ORE) the paper models one crater (Sudbury); the code uses it for the generic 8-25 km crater range and the 1e4-1e6 yr circulation time. Article number E10007 is from memory of the usual form and was not confirmed.

### Osinski2013
- Reference: Osinski, G. R., Tornabene, L. L., Banerjee, N. R., Cockell, C. S., Flemming, R., Izawa, M. R. M., McCutcheon, J., Parnell, J., Preston, L. J., Pickersgill, A. E., Pontefract, A., Sapers, H. M., and Southam, G. (2013). Impact-generated hydrothermal systems on Earth and Mars. Icarus 224(2):347-363.
- DOI or URL: (IMP) 10.1016/j.icarus.2012.08.030 (Crossref-verified) (ORE) 10.1016/j.icarus.2012.08.030
- Cited as: (IMP) "Osinski et al. 2013" (ORE) "Osinski et al. 2013 (Icarus 224:347-363)".
- Ledger ids: IMP-098, IMP-099, IMP-113, IMP-116, ORE-042
- Access: paywalled (metadata only) [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): fetched (PDF, 73 pages, text layer, 83,576 characters). From: https://eprints.gla.ac.uk/142388/1/142388.pdf. Text file: Osinski2013.txt.
- Substitute: repository or open copy found by the lookups and saved as above (version not stated: the first pages were matched against the title, not compared with the printed version).
- Read so far (by the project): (IMP) no statement (ORE) not stated.
- Read in this pass: (IMP) not fetched (ORE) not fetched (Crossref metadata only).
- Reference check: matches.

### Masaitis1998
- Reference: Masaitis, V. L. (1998). Popigai crater: Origin and distribution of diamond-bearing impactites. Meteoritics & Planetary Science 33(2):349-359.
- DOI or URL: (IMP) 10.1111/j.1945-5100.1998.tb01639.x (Crossref-verified) (ORE) 10.1111/j.1945-5100.1998.tb01639.x
- Cited as: (IMP) "Masaitis 1998" (ORE) "Masaitis 1998, MAPS 33:349-359".
- Ledger ids: IMP-100, IMP-101, IMP-120, ORE-041
- Access: paywalled (metadata only) [other sections: paywalled (abstract only) (open-access status not checked).]
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): (IMP) no statement (ORE) not stated.
- Read in this pass: (IMP) not fetched (ORE) not fetched (Crossref metadata only).
- Reference check: (IMP) matches (MAPS 33, pages 349-359; first-name initial V. L. from my recollection). (ORE) matches (volume, issue, pages).

### Donofrio1998
- Reference: Donofrio, R. R. (1998). North American impact structures hold giant field potential. Oil & Gas Journal 96(19):69-83. [Author initials and title from my recollection; the sources file gives only author, journal and pages. Not verified.]
- DOI or URL: none found
- Cited as: "Donofrio 1998"
- Ledger ids: IMP-103, IMP-104, IMP-124, IMP-127
- Access: not found
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Not resolved. Oil and Gas Journal 96(19), 1998; not in Crossref and not online. A trade-press article, so no lawful open copy is expected.
- Read so far (by the project): no statement
- Read in this pass: not fetched
- Reference check: unverified.

### Grieve2005
- Reference: Grieve, R. A. F. (2005). Economic natural resource deposits at terrestrial impact structures. In: McDonald, I., Boyce, A. J., Butler, I. B., Herrington, R. J., and Polya, D. A. (eds), Mineral Deposits and Earth Evolution, Geological Society, London, Special Publications 248:1-29. [Editors from my recollection; Crossref confirms title, volume, pages, year.]
- DOI or URL: (IMP) 10.1144/GSL.SP.2005.248.01.01 (Crossref-verified) (ORE) 10.1144/GSL.SP.2005.248.01.01
- Cited as: (IMP) "Grieve 2005" (ORE) "Grieve 2005 (Geol. Soc. London Spec. Publ. 248:1-29)".
- Ledger ids: IMP-103, IMP-104, IMP-124, IMP-127, ORE-039
- Access: paywalled (metadata only) [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (IMP) no statement (ORE) not stated.
- Read in this pass: (IMP) not fetched (ORE) not fetched (Crossref metadata only).
- Reference check: matches.

### Grieve1994
- Reference: Grieve, R. A. F. and Masaitis, V. L. (1994). The economic potential of terrestrial impact craters. International Geology Review 36(2):105-151.
- DOI or URL: (IMP) 10.1080/00206819409465452 (Crossref-verified) (ORE) 10.1080/00206819409465452
- Cited as: (IMP) "Grieve & Masaitis 1994" (ORE) "Grieve & Masaitis 1994, Int. Geol. Rev. 36(2):105-151".
- Ledger ids: IMP-105, IMP-106, IMP-111, IMP-116, IMP-120, IMP-124, IMP-125, ORE-039, ORE-041
- Access: paywalled (metadata only) [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Merged keys: GrieveMasaitis1994
- Read so far (by the project): (IMP) no statement (ORE) not stated.
- Read in this pass: (IMP) not fetched (ORE) not fetched (Crossref metadata only).
- Reference check: (IMP) matches. (ORE) matches (volume, issue, pages).

### Ames1998
- Reference: Ames, D. E., Watkinson, D. H., and Parrish, R. R. (1998). Dating of a regional hydrothermal system induced by the 1850 Ma Sudbury impact event. Geology 26(5):447-450.
- DOI or URL: (IMP) 10.1130/0091-7613(1998)026<0447:DOARHS>2.3.CO;2 (Crossref-verified) (ORE) 10.1130/0091-7613(1998)026<0447:DOARHS>2.3.CO;2
- Cited as: (IMP) "Ames, Watkinson & Parrish 1998" (ORE) "Ames, Watkinson & Parrish 1998 (Geology 26:447-450)".
- Ledger ids: IMP-116, ORE-042
- Access: paywalled (metadata only) [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (IMP) no statement (ORE) not stated.
- Read in this pass: (IMP) not fetched (ORE) not fetched (Crossref metadata only; Crossref prints first page 447).
- Reference check: (IMP) matches. (ORE) matches (volume, issue, first page).

### Herrick1997 (not in the sources file; it is the real source of Collins et al.'s eq. 28)
- Reference: Herrick, R. R., Sharpton, V. L., Malin, M. C., Lyons, S. N., and Feely, K. (1997). Morphology and morphometry of impact craters. In: Bougher, S. W., Hunten, D. M., and Phillips, R. J. (eds), Venus II, University of Arizona Press, Tucson, pp. 1015-1046. (Collins et al. print the last author as "Freely".)
- DOI or URL: none
- Cited as: not cited in the code; named by Collins et al. 2005 for d_fr = 0.4 D_fr^0.3
- Ledger ids: IMP-042; notes: indirect
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: see IMP-042.

### Not bibliographic
Ben's decisions log entries (2026-09-29, 09-30, 10-01), docs/deeptime-design-decisions.md scale table, MIC rock-tag README, impact_structures.json ("not a USGS source"), the deposit-type and rock-class JSON notes, docs/research/notes/geologist_youtuber_feedback.md, STATUS.md build log, tools/review/p2-08-craters.py and p2-08-impacts.sh.

---

## Volcanoes and magma (VOL, MAG)

### Bremond1995
- Reference: de Bremond d'Ars, J., Jaupart, C. and Sparks, R. S. J. (1995). Distribution of volcanoes in active margins. Journal of Geophysical Research 100(B10): 20421-20432.
- DOI or URL: 10.1029/95JB02153 (https://agupubs.onlinelibrary.wiley.com/doi/10.1029/95JB02153). No open copy found.
- Cited as: "de Bremond d'Ars, Jaupart & Sparks 1995, JGR 100(B10):20421"
- Ledger ids: VOL-039 (the code no longer cites it for a value: the comment now quotes its abstract); HNT-058
- Access: free to read at the publisher (OpenAlex: bronze open access), but the publisher refuses scripted fetches (HTTP 403, retried WP-2 2026-10-02)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (Crossref and OpenAlex). WP-2 (2026-10-02): abstract re-read in full from OpenAlex: "There is no characteristic spacing between volcanoes. In each arc, the distribution of volcano spacing is best represented by a Gamma distribution which corresponds to randomly generated points"; 16 margins, 479 volcanic systems. The per-arc mean spacings are in Hildreth 2007 instead.
- Reference check: authors, year, journal, volume, issue and first page are right (the pages are 20421-20432). The content does NOT obviously match the attribution. The abstract says the study covers 16 active margins and 479 volcanic systems, arcs are ribbons about 10 times longer than wide, and "Volcano spacing follows a random distribution" (Gamma distribution, "no characteristic spacing" in the search summary), explained by gravitational instability in layered fluids. The code comment attributes to it "Real volcanic fronts space their centres 50-70 km apart (the Andes, Cascades, Aleutians, Japan)". The abstract gives no mean spacing and names none of those four arcs. The full text (which may tabulate mean spacings per margin) could not be read. #### Part 1b: sources found for the uncited Earth figures (not cited by the code; added by this pass)

### Szabo1986
- Reference: Szabo, B. J. and Moore, J. G. (1986). Age of the -360-m reef terrace, Hawaii, and the rate of late Pleistocene subsidence of the island. Geology 14(11): 967-968 (volume and pages from the DOI string 10.1130/0091-7613(1986)14<967:AOMRTH>2.0.CO;2; issue not checked).
- DOI or URL: 10.1130/0091-7613(1986)14<967:AOMRTH>2.0.CO;2
- Access: paywalled (abstract only, via OpenAlex)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read in this pass: abstract fetched
- Used for: island subsidence rate (VOL-048).

### Poland2014
- Reference: Poland, M. P., Miklius, A. and Montgomery-Brown, E. K. (2014). Magma supply, storage, and transport at shield-stage Hawaiian volcanoes. Chapter 5 in Characteristics of Hawaiian Volcanoes, U.S. Geological Survey Professional Paper 1801.
- DOI or URL: 10.3133/pp18015; PDF https://pubs.usgs.gov/pp/1801/downloads/pp1801_Chap5_Poland.pdf (open, but came back as unreadable binary).
- Access: open access (agency report)
- Library (the project's source library, not distributed): fetched (PDF, 56 pages, text layer, 214,729 characters). From: https://pubs.usgs.gov/pp/1801/downloads/pp1801_Chap5_Poland.pdf. Text file: Poland2014.txt.
- Read in this pass: abstract fetched (USGS publication page). The chapter text with reservoir depths was not readable.
- Read in WP-2 (2026-10-02), full text from the library copy: a reservoir 1-2 km beneath the centre of Kilauea's caldera and another, "a few cubic kilometers at 2-4 km beneath the summit", under its south part (p. 194); InSAR fits "a sill-like source at 3-km depth" (fig. 14 caption, p. 199).
- Ledger ids: MAG-002, MAG-003, MAG-004; HNT-081
- Used for: Kilauea reservoir depths (MAG-002), size (MAG-003) and shape (MAG-004).

### Przeor2022
- Reference: Przeor, M., D'Auria, L., Pepe, S. et al. (2022). Elastic interaction between Mauna Loa and Kilauea evidenced by independent component analysis. Scientific Reports.
- DOI or URL: not recorded (found by Europe PMC search).
- Access: open access
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read in this pass: search snippet only (abstract as relayed: Kilauea deformation from "a volumetric source at 1.2 km depth" and "a sill-shaped source located at a depth of 3.5 km")
- Used for: Kilauea shallow reservoir depth (MAG-002).

### Scaillet2008
- Reference: Scaillet, B., Pichavant, M. and Cioni, R. (2008). Upward migration of Vesuvius magma chamber over the past 20,000 years. Nature 455(7210): 216-219.
- DOI or URL: 10.1038/nature07232
- Access: paywalled (abstract only, via Europe PMC)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 1010 chars).
- Substitute: none found.
- Read in this pass: abstract fetched
- Used for: Vesuvius chamber depth (MAG-002).

### Courtillot2003a
- Reference: Courtillot, V. E. and Renne, P. R. (2003). On the ages of flood basalt events. Comptes Rendus Geoscience 335(1): 113-140.
- DOI or URL: 10.1016/S1631-0713(03)00006-3
- Access: paywalled (abstract only via OpenAlex; the journal may be open, not tried)
- Library (the project's source library, not distributed): fetched (HTML page, 115,494 characters of text). From: https://doi.org/10.1016/s1631-0713(03)00006-3. Text file: Courtillot2003a.txt.
- Substitute: repository or open copy found by the lookups and saved as above (version not stated: the first pages were matched against the title, not compared with the printed version).
- Merged keys: Courtillot2003
- Read in this pass: abstract fetched
- Used for: flood-basalt frequency (VOL-052 and run-review "one per 20-30 Myr").
- Read in WP-2 (2026-10-02), library text: the Deccan section is 2,500 m thick, at least 1,800 m of it erupted in well under 1 Myr; "a typical duration of 1 million years for the bulk of trap eruption"; no individual flow thicknesses (VOL-036, VOL-052).

### Bryan2008
- Reference: Bryan, S. E. and Ernst, R. E. (2008). Revised definition of Large Igneous Provinces (LIPs). Earth-Science Reviews 86(1-4): 175-202 (OpenAlex dates it 2007, online first).
- DOI or URL: 10.1016/j.earscirev.2007.08.008
- Access: paywalled; abstract not available through OpenAlex
- Library (the project's source library, not distributed): fetched (PDF, 43 pages, text layer, 89,416 characters). From: https://eprints.qut.edu.au/40262/1/COVERSHEET_C40262.pdf. Text file: Bryan2008.txt.
- Substitute: repository or open copy found by the lookups and saved as above (version not stated: the first pages were matched against the title, not compared with the printed version).
- Read in this pass: not fetched (metadata only)
- Used for: suggested stage-2 reference for LIP area and frequency; the fields were not read.

### Hildreth2007
- Reference: Hildreth, W. (2007). Quaternary magmatism in the Cascades: geologic perspectives. U.S. Geological Survey Professional Paper 1744.
- DOI or URL: 10.3133/pp1744
- Access: open access (agency report)
- Library (the project's source library, not distributed): fetched (PDF, 136 pages, text layer, 526,357 characters). From: https://pubs.usgs.gov/pp/pp1744/pp1744.pdf. Text file: Hildreth2007.txt.
- Read in this pass: abstract fetched; it has no lifetime or spacing figures. Stage-2 candidate for stratovolcano lifetimes and vent spacing.
- Read in WP-2 (2026-10-02), full text from the library copy: p. 56, "The oft-repeated old myth of evenly spaced arc volcanoes 40 to 70 km apart ... doesn't bear scrutiny in any arc"; table 4 (p. 58), spacings of the 17 largest Cascade centres 35-170 km, mean 87 km, s.d. 45 km (54 and 56 km with lesser or mafic centres added); table 10 (p. 76), mean spacing of large centres 42 km in the Cascades (30 pairs, s.d. 37), 27 km on the Alaska Peninsula (44 pairs, s.d. 23), 42 km in the Andean Southern Volcanic Zone (35 pairs, s.d. 22), and large centres on the volcanic front 12 of 26, 45 of 45 and 36 of 65; p. 64, "most of the large evolved centers have lifespans in the range 0.1 to 1 Myr".
- Ledger ids: VOL-004, VOL-038, VOL-039; HNT-058

### Grosse2014
- Reference: Grosse, P., Euillades, P. A., Euillades, L. D. and van Wyk de Vries, B. (2014). A global database of composite volcano morphometry. Bulletin of Volcanology 76: 784 (16 pages; online 2013).
- DOI or URL: 10.1007/s00445-013-0784-4
- Cited as: "Grosse et al. 2014" (added by WP-2 in comments of `volcanoes/params.rs`, `volcanoes/shape.rs` and `volcanoes/mod.rs`)
- Ledger ids: VOL-016, VOL-017, VOL-019, VOL-020, VOL-026, VOL-029, VOL-030; HNT-016
- Access: free to read at the publisher (bronze); the publisher returned a page, not the PDF; the authors' submitted version is in the CONICET repository
- Library (the project's source library, not distributed): fetched WP-2 2026-10-02 (PDF, 23 pages, text layer). From: https://ri.conicet.gov.ar/handle/11336/7280 (submitted version; the pages carry the journal's running heads). Text file: Grosse2014.txt.
- Read in WP-2 (2026-10-02), full text: 759 composite volcanoes from the 90-m SRTM model (those of the Global Volcanism Program list it covers, with a basal width over 2 km). Table 1 (p. 7): stratovolcanoes (n = 512): basal width median 7.9 km (range 2.0-65.9), height median 1.02 km, mean 1.13, s.d. 0.55 (range 0.17-4.57), height/basal width median 0.13 (0.02-0.30), mean flank slope 17.7 degrees, main flank 21.5, steepest average slope 27.7 degrees at 0.79 of the height; shields (n = 89): basal width median 15.3 km, height 0.81 km, height/width 0.06, flank slope 8.1 degrees. p. 8: crater and caldera data only for 104 large ones: 0.6-11 km across, median 2.2 km; depth under 100 to 860 m, median 240 m. Subaerial parts only.

### ClagueSherrod2014
- Reference: Clague, D. A. and Sherrod, D. R. (2014). Growth and degradation of Hawaiian volcanoes. Chapter 3 in Poland, M. P., Takahashi, T. J. and Landowski, C. M. (eds), Characteristics of Hawaiian Volcanoes, U.S. Geological Survey Professional Paper 1801, pp. 97-146.
- DOI or URL: 10.3133/pp18013; https://pubs.usgs.gov/pp/1801/downloads/pp1801_Chap3_Clague.pdf
- Cited as: "Clague & Sherrod 2014" (added by WP-2 in comments)
- Ledger ids: VOL-007, VOL-009, VOL-017, VOL-021, VOL-041, VOL-048, VOL-058, MAG-002
- Access: open access (agency report)
- Library: fetched WP-2 2026-10-02 (PDF, text layer, 343,082 characters). Text file: ClagueSherrod2014.txt.
- Read in WP-2 (2026-10-02), full text: abstract (p. 97): "Principal growth occurs in the first 1-2 million years"; "Shallow magma reservoirs (1-7-km depth)" of shield-stage volcanoes, "smaller reservoirs at 20-30-km depth persist". p. 99: "The sizes and spacing of the volcanoes are nonuniform"; only eight volcanoes northwest of the main islands grew to 1,500 m or more above sea level. p. 114: lava flows on a shield dip 2-10 degrees, submarine slopes are steeper. pp. 130-131: subsidence under the growing volcano's load "lasts perhaps 1 m.y." and sinks shorelines "by more than 1 km" (2.6 m/kyr off the Island of Hawaii); thermal subsidence of the plate along much of the chain "~0.01 m/k.y."; reefs drown about 33 Myr after their volcano formed. Mauna Loa's summit 4,169 m.

### Christiansen2001
- Reference: Christiansen, R. L. (2001). The Quaternary and Pliocene Yellowstone Plateau volcanic field of Wyoming, Idaho, and Montana. U.S. Geological Survey Professional Paper 729-G.
- DOI or URL: 10.3133/pp729G; https://pubs.usgs.gov/pp/pp729g/pp729g.pdf
- Cited as: "Christiansen 2001" (added by WP-2 in comments and hints)
- Ledger ids: VOL-006, VOL-023, VOL-027, VOL-029, VOL-045, VOL-046; HNT-060, HNT-063
- Access: open access (agency report)
- Library: fetched WP-2 2026-10-02 (PDF, 37.7 MB, text layer, 1.13 million characters). Text file: Christiansen2001.txt.
- Read in WP-2 (2026-10-02), the summary (pp. G1-G2), physiography (p. G6) and the cycle chronology: caldera-forming eruptions at 2.1 Ma (Huckleberry Ridge Tuff, more than 2,450 km3 over 15,500 km2, caldera more than 75 km long), 1.3 Ma (Mesa Falls Tuff, more than 280 km3) and 0.64 Ma (Lava Creek Tuff, more than 1,000 km3 over more than 7,500 km2); the Yellowstone caldera is a basin 85 by 45 km. The plateau's average elevation is about 2,400 m; Island Park and the eastern Snake River Plain lie at about 1,600-1,900 m.

### Rasmussen2022
- Reference: Rasmussen, D. J., Plank, T. A., Roman, D. C. and Zimmer, M. M. (2022). Magmatic water content controls the pre-eruptive depth of arc magmas. Science 375(6585): 1169-1172.
- DOI or URL: 10.1126/science.abm5174
- Ledger ids: MAG-002; HNT-081
- Access: paywalled; no open copy found (OpenAlex)
- Library: not fetched
- Read in WP-2 (2026-10-02): abstract only (OpenAlex): magma storage depths under active arc volcanoes "vary widely (~0 to 20 kilometers)"; "geophysically observed magma depths (6 +/- 3 kilometers)".

### ChaussardAmelung2014
- Reference: Chaussard, E. and Amelung, F. (2014). Regional controls on magma ascent and storage in volcanic arcs. Geochemistry, Geophysics, Geosystems 15(4): 1407-1418.
- DOI or URL: 10.1002/2013GC005216
- Ledger ids: MAG-002, VOL-005
- Access: free to read at the publisher (bronze), but the publisher refuses scripted fetches
- Library: not fetched
- Read in WP-2 (2026-10-02): abstract only (OpenAlex): 70 volcanoes in 15 regions; shallow reservoirs (under 5 km) are 52 % more common in young than in Precambrian crust, 42 % more common in thin (under 25 km) than in thick (over 45 km) crust, and 33-69 % more common under extension or strike-slip than under compression.

### TenBrink1991
- Reference: ten Brink, U. (1991). Volcano spacing and plate rigidity. Geology 19(4): 397-400.
- DOI or URL: 10.1130/0091-7613(1991)019<0397:VSAPR>2.3.CO;2
- Ledger ids: VOL-007
- Access: paywalled
- Library: not fetched
- Read in WP-2 (2026-10-02): abstract only (publisher page via OpenAlex): spacing of hotspot volcanoes (seven oceanic groups, seven East African) correlates with the plate's elastic thickness, as the distance to the flexural inflection point, proportional to Te^(3/4); spacing in island arcs "is fairly uniform and is much larger" than that distance. No values in the abstract.

### Lev2019
- Reference: Lev, E., Ruprecht, P., Oppenheimer, C., Peters, N., Patrick, M. R., Hernandez, P. A. et al. (2019). A global synthesis of lava lake dynamics. Journal of Volcanology and Geothermal Research 381: 16-31.
- DOI or URL: 10.1016/j.jvolgeores.2019.04.010
- Ledger ids: MAG-020 (the source the count of persistent lava lakes should rest on)
- Access: free to read at the publisher (bronze, accepted version per OpenAlex), but the publisher refused a scripted fetch (HTTP 403, WP-2 2026-10-02); OpenAlex holds no abstract
- Library: not fetched
- Read: not read

### USGS-MaunaLoa, USGS-ActiveVolcanoesFAQ
- Reference: U.S. Geological Survey, Volcano Hazards Program pages "Mauna Loa" (https://www.usgs.gov/volcanoes/mauna-loa) and "How many active volcanoes are there on Earth?" (https://www.usgs.gov/faqs/how-many-active-volcanoes-are-there-earth).
- Ledger ids: VOL-021, VOL-022; MAG-020; HNT-016, HNT-059
- Library: fetched WP-2 2026-10-02 (HTML; text files USGS-MaunaLoa.txt, USGS-ActiveVolcanoesFAQ.txt).
- Read in WP-2 (2026-10-02): Mauna Loa rises "to more than 4 km" above sea level, its submarine flanks descend "an additional 5 km" to the sea floor, which its mass depresses "another 8 km"; its subaerial part extends about 120 km from the island's southern tip to the summit caldera. "There are about 1,350 potentially active volcanoes worldwide, aside from the continuous belts of volcanoes on the ocean floor at spreading centers"; about 500 have erupted in historical time.

### MinecraftSources
- Reference: Minecraft 1.21.1 (Mojang) as decompiled in the NeoForge 21.1.249 sources jar (`neoforge/build/moddev/artifacts/neoforge-21.1.249-sources.jar`, made by any Gradle build). Not a scientific source: it is where the vanilla lava rules the mod replaces are defined.
- Ledger ids: MAG-028, MAG-029, MAG-030, MAG-031
- Read in WP-2 (2026-10-02): `NoiseBasedChunkGenerator.createFluidPicker` (lava below min(-54, sea level)); `Aquifer.NoiseBasedAquifer.computeSubstance` (asks the global picker first), `computeFluidType` (lava where the surface level is at or below -10 and the lava noise exceeds 0.3 in magnitude), `computeRandomizedFluidSurfaceLevel` (40-block cells centred at 40 cellY + 20, spread noise x 10 quantised to 3).

### Web references used (secondary): Wikipedia pages Mount Fuji, Mauna Loa, Mauna Kea, Hawaiian-Emperor seamount chain, Ojos del Salado, Crater Lake, Yellowstone hotspot, Mount St. Helens, Mount Rainier, Mount Shasta, Cotopaxi, Lava lake, Large igneous province, Deccan Traps, Siberian Traps, Altiplano-Puna volcanic complex; USGS pages "How many active volcanoes are there on Earth?", Mauna Loa, Long Valley caldera. Their own cited references are named in the spot checks.


---

## Erosion, isostasy and hypsometry (ERO, ISO, HYP)

### Braun2013
- Reference: Braun, J. and Willett, S. D. (2013). A very efficient O(n), implicit and parallel method to solve the stream power equation governing fluvial incision and landscape evolution. Geomorphology 180-181:170-179.
- DOI or URL: 10.1016/j.geomorph.2012.10.008. Green copy listed at GFZ Potsdam pubman (https://gfzpublic.gfz.de/pubman/item/item_1945954); the page is a JavaScript application, no file link retrievable by script.
- Cited as: "Braun & Willett 2013 (Geomorphology 180-181:170-179)", "Braun & Willett" (spl.rs, routing.rs, params.rs k_spl).
- Ledger ids: ERO-001, ERO-003, ERO-078, ERO-079, HYP-030; notes: also the units of ERO-003/004
- Access: paywalled (abstract elided in Semantic Scholar and OpenAlex); GFZ green copy exists but not retrievable here.
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://gfzpublic.gfz.de/pubman/item/item_1945954. File: Braun2013.txt.
- Read so far (by the project): abstract only (via ResearchGate); code says "implemented from the paper".
- Read in this pass: not fetched (ScienceDirect 403; aggregators return no abstract). Secondary confirmation only: fastscapelib's `spl.hpp` header and Landlab's `FastscapeEroder` both name this paper as the implicit scheme they implement (see Landlab and Fastscapelib entries).
- Reference check: Crossref matches title, authors, volume 180-181, pages 170-179, year 2013 exactly. No equation or appendix number is cited by the code, so nothing to check there. Landlab's docstring points to "Braun & Willett 2013, Appendix B" for the flow-direction independence of the scheme.

### Yuan2019
- Reference: Yuan, X. P., Braun, J., Guerit, L., Rouby, D. and Cordonnier, G. (2019). A new efficient method to solve the stream power law model taking into account sediment deposition. Journal of Geophysical Research: Earth Surface 124(6):1346-1365.
- DOI or URL: 10.1029/2018JF004867. Open copies listed: https://hal.science/hal-02136641v1/document (blocked by Anubis), https://publishup.uni-potsdam.de/frontdoor/index/index/docId/49084 (metadata page with the first abstract sentence only), GFZ pubman item_4140893.
- Cited as: "Yuan, Braun, Guerit, Rouby & Cordonnier 2019", "Yuan et al. 2019 deposition coefficient G".
- Ledger ids: ERO-002, ERO-009, ERO-078
- Access: open copy at https://publishup.uni-potsdam.de/frontdoor/index/index/docId/49084 and HAL hal-02136641 (neither readable by script here); paywalled at Wiley.
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://gfzpublic.gfz-potsdam.de/pubman/item/item_4140893. File: Yuan2019.txt.
- Read so far (by the project): not stated (two-sentence note summary with the DOI).
- Read in this pass: abstract fetched in part (first sentence only, from the Potsdam landing page).
- Reference check: Crossref matches authors, title, volume 124, issue 6, pages 1346-1365. Nothing in the code cites an equation number; the value G = 1 is "not attributed" in the ledger and could not be checked against the paper (see spot check S3).

### Barnes2014
- Reference: Barnes, R., Lehman, C. and Mulla, D. (2014). Priority-flood: An optimal depression-filling and watershed-labeling algorithm for digital elevation models. Computers & Geosciences 62:117-127.
- DOI or URL: (ERO, ISO, HYP) 10.1016/j.cageo.2013.04.024; open preprint https://arxiv.org/abs/1511.04463 (PDF https://arxiv.org/pdf/1511.04463). (HYD, YNG) https://doi.org/10.1016/j.cageo.2013.04.024 ; open preprint https://arxiv.org/abs/1511.04463 and https://richard.science/sci/2014_depressions.pdf (AMP, GLA) 10.1016/j.cageo.2013.04.024; arXiv https://arxiv.org/abs/1511.04463
- Cited as: (ERO, ISO, HYP) "Barnes, Lehman & Mulla 2014 ... Algorithms 3 and 4" (pflood.rs; plan section 8.2). (HYD, YNG) "Barnes, Lehman & Mulla 2014, Algorithm 3 with the pit queue of Algorithm 4" (hydro/mod.rs module doc); research note gives arXiv 1511.04463 (AMP, GLA) "Barnes, Lehman & Mulla 2014 (Priority-Flood+epsilon)"
- Ledger ids: AMP-016, AMP-017, ERO-080, HYD-014
- Access: open copy (arXiv preprint) at https://arxiv.org/pdf/1511.04463. [other sections: open copy (author preprint at the two URLs above); the journal version is paywalled / open copy (arXiv) at https://arxiv.org/abs/1511.04463]
- Library (the project's source library, not distributed): fetched (PDF, 17 pages, text layer, 49,580 characters). From: https://arxiv.org/pdf/1511.04463. Text file: Barnes2014.txt.
- Merged keys: Barnes2014a
- Read so far (by the project): (ERO, ISO, HYP) in full ("implemented from the paper"). (HYD, YNG) "from the paper" (RichDEM, GPL, not opened); nothing says whether full text or abstract was read (AMP, GLA) not stated
- Read in this pass: (ERO, ISO, HYP) full text fetched (arXiv v1: abstract and the Algorithm 1 to 5 captions). (HYD, YNG) full text fetched (author preprint: abstract, captions and pseudocode of Algorithms 1 to 5) (AMP, GLA) abstract fetched (arXiv abs page). The arXiv PDF came back as unreadable binary, so the epsilon variant's algorithm number was not read.
- Reference check: (ERO, ISO, HYP) Crossref matches authors, title, volume 62, pages 117-127. The arXiv preprint carries the same citation line. Algorithm numbering in the preprint: 1 Priority-Flood, 2 Improved Priority-Flood (plain queue for pits), 3 Priority-Flood+epsilon, 4 Priority-Flood+FlowDirs, 5 Improved Priority-Flood+Watershed Labels. So "Algorithms 3 and 4" is the epsilon variant plus the flow-direction variant; consistent with the code's use (see S14). The preprint numbering may differ from the journal's; not checked. (HYD, YNG) title, authors, year, volume, pages all match (the project gave no title). The citation "Algorithm 3 with the pit queue of Algorithm 4" does not match the paper's numbering: see Part 2, HYD-014. (AMP, GLA) authors, year, journal, volume and pages match. The abstract says the algorithm "can also be adapted to label watersheds and determine flow directions through either incremental elevation changes or depression carving. In the case of incremental elevation changes, the algorithm includes safety checks not present in prior works". That supports the existence of an epsilon (incremental elevation) variant; the name "Priority-Flood+epsilon" as written in the code could not be checked in the text.

### Landlab
- Reference: Landlab (MIT licence), component library. Components used for equations: ThresholdEroder, SimpleSubmarineDiffuser, CarbonateProducer, ExponentialWeatherer, ExponentialWeathererIntegrated, LinearDiffuser, Lithology/MaterialLayers. Recommended software citation (not cited by the code): Hobley, D. E. J., Adams, J. M., Nudurupati, S. S., et al. (2017). Creative computing with Landlab: an open-source toolkit for building, coupling, and exploring two-dimensional numerical models of Earth-surface dynamics. Earth Surface Dynamics 5(1):21-46.
- DOI or URL: https://github.com/landlab/landlab (files read at branch master, 2026-10-01: `src/landlab/components/{carbonate/carbonate_producer.py, marine_sediment_transport/simple_submarine_diffuser.py, threshold_eroder/threshold_eroder.py, weathering/exponential_weathering.py, weathering/exponential_weathering_integrated.py, diffusion/diffusion.py, stream_power/fastscape_stream_power.py}`); paper DOI 10.5194/esurf-5-21-2017 (Crossref confirms title, volume 5, issue 1, pages 21-46).
- Cited as: "Landlab `ThresholdEroder`", "Landlab `SimpleSubmarineDiffuser`", "Landlab default", "Landlab's `LinearDiffuser`", etc.
- Ledger ids: ERO-012, ERO-022, ERO-027, ERO-031, ERO-045 to ERO-048, ERO-057 to ERO-062, ERO-068
- Access: open access (source code on GitHub).
- Library (the project's source library, not distributed): fetched (PDF, 26 pages, text layer, 102,198 characters). From: https://www.earth-surf-dynam.net/5/21/2017/esurf-5-21-2017.pdf. Text file: Landlab.txt.
- Read so far (by the project): component names only, "from source"; default values not shown as retrieved.
- Read in this pass: full text fetched (seven source files above, downloaded with curl and grepped; lithology/MaterialLayers not read).
- Reference check: all component names and paths exist. Two statements about Landlab in the code do not match the source: LinearDiffuser maps node diffusivities to links with the maximum, not the mean (S11); and the weatherer defaults are placeholders 1.0 and 1.0 (S6). The carbonate and submarine defaults match (S7 to S10). Caveat: master as of 2026-10-01; the version the code was read against is not recorded.

### Bosscher1992
- Reference: Bosscher, H. and Schlager, W. (1992). Computer simulation of reef growth. Sedimentology 39(3):503-512.
- DOI or URL: 10.1111/j.1365-3091.1992.tb02130.x.
- Cited as: "Bosscher & Schlager 1992" (carbonate.rs, plan section 8.2).
- Ledger ids: ERO-057
- Access: paywalled (abstract not fetched).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): no record; credited via Landlab.
- Read in this pass: not fetched. The growth law and the parameter ranges are known here only through Landlab's docstring (secondary source).
- Reference check: Crossref matches title, authors, volume 39, issue 3, pages 503-512, June 1992. Landlab's docstring gives the same citation and the law G = G_m tanh(I_0 e^(-kz)/I_k).

### Ahnert1976
- Reference: Ahnert, F. (1976). Brief description of a comprehensive three-dimensional process-response model of landform development. Zeitschrift fuer Geomorphologie, Supplementband 25:29-49.
- DOI or URL: no DOI found. Citation text taken from the Landlab docstring (`exponential_weathering.py`, lines 52-55: "Ahnert, F. (1976). Brief description of a comprehensive three-dimensional process-response model of landform development Z. Geomorphol. Suppl. 25, 29 - 49.").
- Cited as: "Ahnert 1976" (weathering.rs).
- Ledger ids: ERO-068
- Access: book/journal supplement (not online).
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated.
- Read in this pass: not fetched; reference text copied from Landlab's source.
- Reference check: the code writes only "Ahnert 1976 (soil production in the style of)"; Landlab uses the same phrase "in the style of Ahnert (1976)". Matches.

### Hjulstrom1935
- Reference: Hjulstroem, F. (1935). Studies of the morphological activity of rivers as illustrated by the River Fyris. Bulletin of the Geological Institution of the University of Upsala 25:221-527.
- DOI or URL: (ERO, ISO, HYP) none (printed bulletin). Citation form from the Wikipedia article "Hjulstroem curve" (secondary). (RCK, STR, DEF) no DOI. No open scan found.
- Cited as: (ERO, ISO, HYP) "Hjulstrom 1935, Bull. Geol. Inst. Univ. Uppsala 25". (RCK, STR, DEF) "Hjulström 1935, Bull. Geol. Inst. Univ. Uppsala 25"
- Ledger ids: ERO-098, ERO-153 to ERO-163
- Access: book (not online). [other sections: not found (journal volume from 1935; not located online)]
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): (ERO, ISO, HYP) no record. (RCK, STR, DEF) not stated
- Read in this pass: (ERO, ISO, HYP) not fetched (secondary only). (RCK, STR, DEF) secondary source only (Wikipedia, "Hjulström curve": https://en.wikipedia.org/wiki/Hjulstr%C3%B6m_curve)
- Reference check: (ERO, ISO, HYP) the citation matches the standard one; the code gives no pages (221-527). The paper is about entrainment, transport and deposition velocities by grain size, not about an erodibility multiplier on K; the multipliers 4 to 10 are the project's own (S26). (RCK, STR, DEF) Wikipedia gives the citation as "Hjulstrom, F. (1935). Studies of the morphological activity of rivers as illustrated by the River Fyris, Bulletin. Geological Institute Upsalsa, 25, 221-527." The code's volume matches; pages are missing in the code (221-527). The work is about loose grain entrainment, deposition and transport velocities in a river; it is not about bedrock incision, so it can support at most a ranking of loose classes (see spot check RCK-008).

### Harel2016
- Reference: Harel, M.-A., Mudd, S. M. and Attal, M. (2016). Global analysis of the stream power law parameters based on worldwide 10Be denudation rates. Geomorphology 268:184-196.
- DOI or URL: 10.1016/j.geomorph.2016.05.035; author accepted manuscript https://www.research.ed.ac.uk/files/25670599/25669129._Harel_et_al_2016_author_accepted_manuscript_002_.pdf (403 from both WebFetch and curl).
- Cited as: "Harel, Mudd & Attal 2016", "worldwide 10Be denudation rates favour n > 1" (params.rs).
- Ledger ids: ERO-006
- Access: open copy (author manuscript) at the Edinburgh URL above (not retrievable here); paywalled at Elsevier.
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: Crossref gives title, authors, volume 268, pages 184-196. The code cites it by authors and year only (no journal), and the sources file likewise; the title confirms the paper is about stream power parameters from 10Be rates. The claim "favour n > 1" is not checked.

### Schott2024
- Reference: Schott, H., Galin, E., Guerin, E., Paris, A. and Peytavie, A. (2024). Terrain amplification using multi scale erosion. ACM Transactions on Graphics 43(4).
- DOI or URL: (ERO, ISO, HYP) 10.1145/3658200; code https://github.com/H-Schott/MultiScaleErosion (MIT). (HYD, YNG) https://doi.org/10.1145/3658200 ; software https://github.com/H-Schott/MultiScaleErosion (MIT, "Copyright (c) 2023 Schott"; commit 964a3d51 exists, dated 25 September 2026, message "warp control") (AMP, GLA) 10.1145/3658200; code https://github.com/H-Schott/MultiScaleErosion (MIT licence; shaders in data/shaders/).
- Cited as: (ERO, ISO, HYP) "Schott et al. 2024, Terrain Amplification using Multi-scale Erosion, ACM TOG 43(4)"; commit 964a3d5, `data/shaders/thermal.glsl`. (HYD, YNG) "in the style of Schott et al. 2024" (design decisions table; NOTICE.md) (AMP, GLA) "Schott et al. 2024, Terrain Amplification using Multi-scale Erosion (ACM TOG 43(4)); code H-Schott/MultiScaleErosion (MIT, commit 964a3d5)"
- Ledger ids: AMP-001, AMP-006 to AMP-010, AMP-023, AMP-028, AMP-034, AMP-038, ERO-164, ERO-165, YNG-015; notes: and the method behind 003, 005
- Access: open copy at the authors' repository (code only); the paper PDF was not reachable (ACM 403, HAL behind an anti-bot wall) [other sections: not found as open text in this pass (not tried beyond Crossref). / not checked (ACM; the code is open)]
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://github.com/H-Schott/MultiScaleErosion. File: Schott2024.txt.
- Read so far (by the project): (ERO, ISO, HYP) code ported with its pass changed. (HYD, YNG) NOTICE.md says shaders and main.cpp read on 2026-09-29 (AMP, GLA) the shaders (ported "equations, not lines"); the paper text probably not read in full
- Read in this pass: (ERO, ISO, HYP) not fetched. (HYD, YNG) abstract fetched (OpenAlex); commit and licence checked on GitHub (AMP, GLA) full text of two shader files fetched (thermal.glsl, erosion.glsl at branch main; the project's commit 964a3d5 was not checked); the paper itself not fetched
- Reference check: (ERO, ISO, HYP) Crossref matches venue, volume 43, issue 4, and the title (with "Multi Scale", no hyphen). Author list from Crossref: Hugo Schott, Eric Galin, Eric Guerin, Axel Paris, Adrien Peytavie. (HYD, YNG) matches (the project gave no title). The commit hash named in the sources file exists. (AMP, GLA) authors, year, venue, volume and DOI are right (search results and the author's site give July 2024, ACM Trans. Graph., presented at SIGGRAPH 2024). The shader values the code attributes to Schott are partly misread: see spot checks AMP-034 and AMP-038.

### ClarkRoyden2000
- Reference: Clark, M. K. and Royden, L. H. (2000). Topographic ooze: Building the eastern margin of Tibet by lower crustal flow. Geology 28(8):703-706.
- DOI or URL: 10.1130/0091-7613(2000)28<703:TOBTEM>2.0.CO;2.
- Cited as: (ERO, ISO, HYP) "Clark & Royden 2000, Geology 28(8):703-706" (spread.rs). (PLT) "Clark & Royden 2000, Geology 28:703".
- Ledger ids: ISO-018, ISO-020, ISO-022
- Access: paywalled (GeoScienceWorld 403); not fetched. [other sections: paywalled (GSA; not tested).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Merged keys: Clark2000
- Read so far (by the project): (ERO, ISO, HYP) bibliographic check only. (PLT) not stated; not in code.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) metadata only.
- Reference check: (ERO, ISO, HYP) Crossref matches title, authors, volume 28, issue 8, first page 703 (the end page 706 is not in the Crossref record, taken as written). Attribution of "D0 range 10^2 to 10^5" to this paper cannot be checked. (PLT) correct.

### Royden1997
- Reference: Royden, L. H., Burchfiel, B. C., King, R. W., Wang, E., Chen, Z., Shen, F. and Liu, Y. (1997). Surface deformation and lower crustal flow in eastern Tibet. Science 276(5313):788-790.
- DOI or URL: 10.1126/science.276.5313.788.
- Cited as: (ERO, ISO, HYP) "Royden et al. 1997, Science 276:788-790". (PLT) "Royden et al. 1997".
- Ledger ids: ISO-018, ISO-020
- Access: paywalled; not fetched. [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (ERO, ISO, HYP) bibliographic check only. (PLT) not stated; not in code.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) abstract fetched.
- Reference check: (ERO, ISO, HYP) Crossref matches title, volume 276, issue 5313, pages 788-790. (PLT) identifiable; correct.

### EnglandMcKenzie1982
- Reference: England, P. and McKenzie, D. (1982). A thin viscous sheet model for continental deformation. Geophysical Journal of the Royal Astronomical Society 70(2):295-321 (now Geophysical Journal International). Correction: England, P. and McKenzie, D. (1983), 73(2):523-532.
- DOI or URL: (ERO, ISO, HYP) 10.1111/j.1365-246X.1982.tb04969.x; correction 10.1111/j.1365-246x.1983.tb03328.x. (PLT) 10.1111/j.1365-246X.1982.tb04969.x.
- Cited as: (ERO, ISO, HYP) "England & McKenzie 1982 (GJRAS 70(2):295-321; correction 73:523, 1983)". (PLT) "England & McKenzie 1982 (GJRAS 70:295)".
- Ledger ids: ISO-019, ISO-020
- Access: paywalled; not fetched. [other sections: paywalled.]
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Merged keys: England1982
- Read so far (by the project): (ERO, ISO, HYP) bibliographic check only. (PLT) not stated; not in code.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) metadata only.
- Reference check: (ERO, ISO, HYP) Crossref matches title, volume 70, issue 2, pages 295-321, and the correction record (volume 73, issue 2, pages 523-532). (PLT) correct. A correction exists (GJRAS 73:523-532, 1983).

### MolnarLyonCaen1988
- Reference: Molnar, P. and Lyon-Caen, H. (1988). Some simple physical aspects of the support, structure, and evolution of mountain belts. In: Clark, S. P., Burchfiel, B. C. and Suppe, J. (eds), Processes in Continental Lithospheric Deformation, Geological Society of America Special Paper 218, pp. 179-208.
- DOI or URL: 10.1130/SPE218-p179.
- Cited as: (ERO, ISO, HYP) "Molnar & Lyon-Caen 1988, GSA Special Paper 218:179-207". (PLT) "Molnar & Lyon-Caen 1988 (GSA Spec. Pap. 218)", "pages 179-207" in the force-balance design (Tibet 6-7e12 N/m).
- Ledger ids: ISO-019, ISO-020, PLT-187
- Access: book (not online); paywalled chapter. [other sections: book chapter (abstract visible via OpenAlex; full text not online).]
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Merged keys: Molnar1988
- Read so far (by the project): (ERO, ISO, HYP) bibliographic check only. (PLT) not stated.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) abstract fetched.
- Reference check: (ERO, ISO, HYP) title is not written in the code; Crossref gives the title above and pages 179-208. The code has 179-207, a one-page difference (immaterial, but the Crossref end page is 208). The editors are not verified in this pass. (PLT) title and page start correct; the editors and the "6-7e12 N/m" figure are not in the abstract. The abstract does state that more gravitational potential energy is stored in a column with a thick crustal root and that ranges reach a maximum mean elevation; no number.

### Bird1991
- Reference: Bird, P. (1991). Lateral extrusion of lower crust from under high topography in the isostatic limit. Journal of Geophysical Research 96(B6):10275-10286.
- DOI or URL: 10.1029/91JB00370.
- Cited as: (ERO, ISO, HYP) "Bird 1991, JGR 96(B6):10275-10286". (PLT) "Bird 1991 (JGR 96:10275)".
- Ledger ids: ISO-019, ISO-020
- Access: paywalled; not fetched. [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (ERO, ISO, HYP) bibliographic check only. (PLT) not stated; not in code.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) abstract fetched (Crossref). The abstract states that flux is proportional to the third power of the topographic gradient (cubic power law creep), which matches the project's "channel law cube".
- Reference check: (ERO, ISO, HYP) Crossref matches volume 96, issue B6, pages 10275-10286 (2026-10-01 Crossref query on the title). (PLT) correct.

### Rey2001
- Reference: Rey, P., Vanderhaeghe, O. and Teyssier, C. (2001). Gravitational collapse of the continental crust: definition, regimes and modes. Tectonophysics 342(3-4):435-449.
- DOI or URL: 10.1016/S0040-1951(01)00174-3.
- Cited as: (ERO, ISO, HYP) "Rey, Vanderhaeghe & Teyssier 2001, Tectonophysics 342(3-4):435-449". (PLT) "Rey, Vanderhaeghe & Teyssier 2001 (Tectonophysics 342:435)".
- Ledger ids: ISO-019, ISO-020
- Access: paywalled; not fetched. [other sections: paywalled.]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (ERO, ISO, HYP) bibliographic check only. (PLT) not stated; not in code.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) metadata only.
- Reference check: (ERO, ISO, HYP) Crossref matches title, volume 342, issue 3-4, pages 435-449. (PLT) correct.

### Beaumont2001
- Reference: Beaumont, C., Jamieson, R. A., Nguyen, M. H. and Lee, B. (2001). Himalayan tectonics explained by extrusion of a low-viscosity crustal channel coupled to focused surface denudation. Nature 414(6865):738-742.
- DOI or URL: 10.1038/414738a.
- Cited as: (ERO, ISO, HYP) "Beaumont et al. 2001 (channel flow coupled to erosion), Nature 414:738" (build log only). (PLT) "Beaumont et al. 2001 (Nature 414:738)".
- Ledger ids: ISO-018, ISO-020; notes: design background for ISO-018
- Access: paywalled; not fetched. [other sections: paywalled.]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (ERO, ISO, HYP) no record. (PLT) not stated; not in code.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) metadata only.
- Reference check: (ERO, ISO, HYP) Crossref matches volume 414, issue 6865, pages 738-742. (PLT) correct.

### Cortial2019
- Reference: Cortial, Y., Peytavie, A., Galin, E. and Guerin, E. (2019). Procedural tectonic planets. Computer Graphics Forum 38(2):1-11 (Eurographics 2019).
- DOI or URL: (ERO, ISO, HYP) 10.1111/cgf.13614; open copy https://hal.science/hal-02136820 (Anubis blocks scripted access). (SCL, PRV, POL, BAK) 10.1111/cgf.13614 ; author copy https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf (PLT) 10.1111/cgf.13614; open copies https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf and https://hal.science/hal-02136820v1/file/2019-Procedural-Tectonic-Planets.pdf (HAL blocked WebFetch with an Anubis page; the LIRIS copy downloaded fine with curl).
- Cited as: (ERO, ISO, HYP) "Cortial et al. 2019 (procedural tectonic planets; oceanic dampening z -= (1 - z / z_t) eps_o dt, z_r, z_a, z_t, z_c, section 4.5)". (SCL, PRV, POL, BAK) "Cortial, Peytavie, Galin & Guerin 2019, Procedural Tectonic Planets, Appendix A, Table 3"; "Cortial: 2 My"; "Cortial: 6,370 km" (PLT) "Cortial, Peytavie, Galin & Guerin 2019", "Cortial Appendix A, Table 3", "Table 2", sections 3, 4.1-4.5, 6, 7.5.
- Ledger ids: HYP-015, ISO-008, ISO-011, ISO-013 to ISO-015, ISO-025 to ISO-027, PLT-019 to PLT-022, PLT-028, PLT-029, PLT-036, PLT-041, PLT-052, PLT-055, PLT-056, PLT-068, PLT-070, PLT-073, PLT-081, PLT-085, PLT-093, PLT-098, PLT-100, PLT-101, PLT-119, PLT-130, SCL-029, SCL-030, SCL-034, SCL-042 to SCL-044; notes: context SCL-030; from the sources file
- Access: open copy at https://hal.science/hal-02136820 (not retrievable by script here). [other sections: open copy (author PDF) at the URL above; Wiley page returned 403 / open copy at the LIRIS URL above.]
- Library (the project's source library, not distributed): fetched (PDF, 11 pages, text layer, 46,531 characters). From: https://perso.liris.cnrs.fr/eric.galin/Articles/2019-planets.pdf. Text file: Cortial2019.txt.
- Read so far (by the project): (ERO, ISO, HYP) the plate model is ported "from the paper" per other documents; not assessed here. (SCL, PRV, POL, BAK) research note says read from the PDF in a 2026-09-27 session; Appendix A coverage not stated (PLT) in full per P1-03 log ("implemented from Cortial"), no statement of constant-by-constant verification.
- Read in this pass: (ERO, ISO, HYP) not fetched. (SCL, PRV, POL, BAK) not fetched (author PDF exceeds the tool's 10 MB limit; Wiley and Eurographics digital library 403) (PLT) full text fetched (every section, Table 1, Table 2, Table 3 in Appendix A, section 6, 7.4, 7.5).
- Reference check: (ERO, ISO, HYP) Crossref matches title "Procedural Tectonic Planets", volume 38, issue 2, pages 1-11. The equation and symbol names (z_r, z_a, z_t, z_c, section 4.5) could not be checked. (SCL, PRV, POL, BAK) authors, title, journal, year, volume match Crossref. Table and appendix numbering not verifiable. (PLT) authors, year, title, journal, volume(issue) correct. Pages 1-11 (Crossref). Table numbering in the paper: Table 1 = parametrization of the crust, Table 2 = timings, Table 3 = Appendix A constants. So params.rs "Appendix A, Table 3", the research note's "Table 1 for per-sample parameters" and "Table 2 for timings" are all right (this settles earth-file flag 24). Departures from the paper are listed in Part 2.

### SteinStein1992
- Reference: Stein, C. A. and Stein, S. (1992). A model for the global variation in oceanic depth and heat flow with lithospheric age. Nature 359(6391):123-129.
- DOI or URL: (ERO, ISO, HYP) 10.1038/359123a0. (PLT) 10.1038/359123a0 (Nature page redirects to a login wall).
- Cited as: "Stein & Stein 1992, Nature 359:123-129 (GDH1)".
- Ledger ids: ISO-011 to ISO-013, ISO-016, ISO-017, PLT-204
- Access: paywalled (abstract truncated at Nature). [other sections: paywalled (closed; no abstract in OpenAlex or Semantic Scholar).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Merged keys: Stein1992
- Read so far (by the project): (ERO, ISO, HYP) not stated; the 4.2 / 5.0 / 5.5 km depths are quoted without a source. (PLT) not stated; formulas appear in a test comment.
- Read in this pass: (ERO, ISO, HYP) abstract fetched in part (first sentence only). (PLT) metadata only (Crossref, OpenAlex, Semantic Scholar). Equations not read.
- Reference check: (ERO, ISO, HYP) Crossref matches title, authors, volume 359, issue 6391, pages 123-129. The model name GDH1 is not in the part of the abstract that was reachable. (PLT) citation correct (authors Carol A. Stein and Seth Stein). The depth equations in the test comment could not be confirmed against the paper.

### ParsonsSclater1977
- Reference: Parsons, B. and Sclater, J. G. (1977). An analysis of the variation of ocean floor bathymetry and heat flow with age. Journal of Geophysical Research 82(5):803-827.
- DOI or URL: 10.1029/JB082i005p00803.
- Cited as: (ERO, ISO, HYP) "Parsons & Sclater 1977 (mean ridge crest depth 2.5 km)" (build log). (PLT) "Parsons & Sclater 1977" (ridge crest depth 2.5 km); no title or journal given in the code.
- Ledger ids: ISO-014, PLT-204
- Access: paywalled; not fetched. [other sections: paywalled (no abstract fetched).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Merged keys: Parsons1977
- Read so far (by the project): (ERO, ISO, HYP) no record. (PLT) not stated.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) not fetched itself; its formulas were read as reproduced in King & Adam 2014 (a secondary open copy) and Wikipedia "Seafloor spreading".
- Reference check: (ERO, ISO, HYP) Crossref matches the title, volume 82, issue 5, pages 803-827. The sources file gives no year-title pair; the 1977 paper is the correct identification. (PLT) now identifiable. Identification only; the "2.5 km" is the intercept of the square-root law (see Part 2).

### Cogley1984
- Reference: Cogley, J. G. (1984). Continental margins and the extent and number of the continents. Reviews of Geophysics 22(2):101-122.
- DOI or URL: 10.1029/RG022i002p00101.
- Cited as: (ERO, ISO, HYP) "Cogley 1984 (continental crust 41 % of the surface)". (PLT) "Cogley 1984, Rev. Geophys. 22:101-122" (29 percent land on 41 percent continental crust).
- Ledger ids: HYP-012, HYP-013, PLT-158, PLT-159, PLT-205
- Access: paywalled; not fetched. [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (ERO, ISO, HYP) no record. (PLT) not stated.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) abstract fetched (Crossref JSON, full abstract).
- Reference check: (ERO, ISO, HYP) identified by Crossref; the title, volume, pages are as above. The 41 % figure is not checked. (PLT) citation correct (issue 2). The abstract gives 41 percent for continental crust but not "29 percent land"; it gives 30.6 percent of continental area submerged, from which land is about 28.5 percent of the planet (arithmetic on the abstract's numbers, not a stated value).

### Shepard
- Reference: UNIDENTIFIED. Cited as "Shepard (mean shelf break 0 to -130 m)" in the build log with no year or title. Most likely a Francis P. Shepard book on submarine geology; I could not confirm which edition because the web search budget was spent and a Crossref query returned nothing.
- DOI or URL: none.
- Cited as: "Shepard's mean shelf break" (build log 2026-09-29 P1-17 partial).
- Ledger ids: HYP-013
- Access: not found.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Probably the same work as Shepard1963 (Submarine Geology, 2nd edition, 1963), the only Shepard in the bibliography; the code names no year or title, so this is a guess and not a match.
- Read so far (by the project): no record.
- Read in this pass: not fetched.
- Reference check: cannot be checked. Suggest asking the author which Shepard work was meant.

### Peel2007
- Reference: Peel, M. C., Finlayson, B. L. and McMahon, T. A. (2007). Updated world map of the Koeppen-Geiger climate classification. Hydrology and Earth System Sciences 11(5):1633-1644.
- DOI or URL: (ERO, ISO, HYP) 10.5194/hess-11-1633-2007; open access https://hess.copernicus.org/articles/11/1633/2007/. (HYD, YNG) https://doi.org/10.5194/hess-11-1633-2007 (open access) ; https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf (MOI, BIO) https://doi.org/10.5194/hess-11-1633-2007 ; PDF https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf ; the author's page with the same table https://people.eng.unimelb.edu.au/mpeel/koppen.html (SCL, PRV, POL, BAK) 10.5194/hess-11-1633-2007 ; https://hess.copernicus.org/articles/11/1633/2007/ (CLM) 10.5194/hess-11-1633-2007
- Cited as: (ERO, ISO, HYP) "Peel, Finlayson & McMahon 2007 (HESS 11:1633)". (MOI, BIO) "Peel, Finlayson & McMahon 2007, Table 1" (koppen.rs header "the rules the research note cites"; plan §8.3; STATUS P1-05, P1-17) (SCL, PRV, POL, BAK) not cited ("Koppen's dryness threshold (20 T + 140 mm)", no year)
- Ledger ids: BAK-020, BIO-001 to BIO-017, HYD-046, HYP-025; none (comparison only); notes: BIO-017 is a game-choice about the summer half; and BIO-001 to BIO-016; comparison only; supporting source found in this pass
- Access: open access. [other sections: open access (gold)]
- Library (the project's source library, not distributed): fetched (PDF, 12 pages, text layer, 46,987 characters). From: https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf. Text file: Peel2007.txt.
- Merged keys: Peel2007 (Koppen-Geiger)
- Read so far (by the project): (ERO, ISO, HYP) no record. (HYD, YNG) n/a here (MOI, BIO) nothing stated; the research note it points to cites Wikipedia, not this paper (SCL, PRV, POL, BAK) not stated (CLM) the shares were "quoted in the 2026-09-29 P1-17 entry"
- Read in this pass: (ERO, ISO, HYP) abstract fetched; PDF text partly extracted (title page, abstract, methods, Africa section). The page with global main-group shares was not located. (HYD, YNG) full text fetched (Table 1 with notes, the paragraphs on test order and on the Cs/Cw tie rule, the discussion with land shares) (MOI, BIO) full text fetched (decoded PDF text layer: Table 1 with its notes, the paragraphs on test order and on the Cs/Cw tie rule, the discussion with the land shares) (SCL, PRV, POL, BAK) abstract fetched (landing page); PDF returned as binary so Table 1 not read; criteria were taken from Wikipedia's Koppen article instead (secondary) (CLM) full text fetched (section 4 conclusions)
- Reference check: (ERO, ISO, HYP) Crossref matches title, volume 11, issue 5, pages 1633-1644. The abstract in the HTML page says "BWh (14.2%, Hot desert) followed by Aw (11.5%, Tropical savannah)"; the build log's shares B 30.2 / D 24.6 / A 19.0 / C 13.4 / E 12.8 % were not found in the text read (S19). (HYD, YNG) matches. See the MOI file for the full Table 1 check. (MOI, BIO) authors, year, journal, volume, pages match. Table 1 is the table the code cites. The research note's source (Wikipedia) differs from the paper in the B threshold's form only (20 x MAT + 280/140/0 against 10 x (2 x MAT + 28/14/0), the same numbers) and in listing a "-3 degC" option the paper does not have. Details in Part 2. (SCL, PRV, POL, BAK) the code names no source. Peel et al. 2007 and Kottek et al. 2006 are the standard references per Wikipedia. (CLM) matches.
- Cited as (HYD): "Koppen's E (polar climate: warmest month below 10 degC)", no paper named

### CharetteSmith2010
- Reference: Charette, M. A. and Smith, W. H. F. (2010). The volume of Earth's ocean. Oceanography 23(2):112-114.
- DOI or URL: (ERO, ISO, HYP) 10.5670/oceanog.2010.51; open PDF https://tos.org/oceanography/assets/docs/23-2_charette.pdf. (PLT) 10.5670/oceanog.2010.51.
- Cited as: (ERO, ISO, HYP) "Charette & Smith 2010, Oceanography 23(2):112-114 (ocean volume and mean depth on SRTM30_PLUS)". (PLT) "Charette & Smith 2010, Oceanography 23(2):112-114" (sea mean 3,682 m; 361.84 million km2 = 70.9 percent).
- Ledger ids: HYP-007, PLT-097, PLT-169, PLT-170
- Access: open access. [other sections: open access (Oceanography Society).]
- Library (the project's source library, not distributed): fetched (PDF, 3 pages, text layer, 10,430 characters). From: https://tos.org/oceanography/assets/docs/23-2_charette.pdf. Text file: CharetteSmith2010.txt.
- Merged keys: Charette2010
- Read so far (by the project): (ERO, ISO, HYP) no record. (PLT) not stated.
- Read in this pass: (ERO, ISO, HYP) full text fetched (text of the 3-page article including Tables 1 and 2). (PLT) metadata only (OpenAlex abstract is only a copyright line).
- Reference check: (ERO, ISO, HYP) Crossref matches title, volume 23, issue 2, pages 112-114. The article says: "For this effort, we relied on the SRTM30PLUS V5 bathymetry database (Becker et al., 2009)", so the project's "on SRTM30_PLUS" is right. (PLT) correct. Numbers not read.

### Tzathas2024
- Reference: Tzathas, P., Gailleton, B., Steer, P. and Cordonnier, G. (2024). Physically-based analytical erosion for fast terrain generation. Computer Graphics Forum 43(2) (Eurographics 2024).
- DOI or URL: (ERO, ISO, HYP) 10.1111/cgf.15033; author PDF https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf. (AMP, GLA) 10.1111/cgf.15033; preprint https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf (also HAL hal-04525371)
- Cited as: (ERO, ISO, HYP) "Tzathas et al. 2024 (author PDF)". (AMP, GLA) "Tzathas 2024 (Analytical Terrains, Eurographics; Inria research-only code)"
- Ledger ids: AMP-034, HYP-031; notes: research note only; not cited by code
- Access: open copy at the Inria URL above. [other sections: open copy (Inria preprint) at the URL above, but the PDF came back as binary and the Wiley and EG library pages returned 403]
- Library (the project's source library, not distributed): fetched (PDF, 14 pages, text layer, 64,524 characters). From: https://www-sop.inria.fr/reves/Basilic/2024/TGSC24/Analytical_Terrains_EG.pdf. Text file: Tzathas2024.txt.
- Read so far (by the project): (ERO, ISO, HYP) the note quotes timings from it. (AMP, GLA) the PDF was linked; nothing says it was read
- Read in this pass: (ERO, ISO, HYP) full text fetched (curl, text extracted; Table 1, Table 2, thermal erosion section). (AMP, GLA) search snippet only (title, authors, venue; says the method also covers "hillslope erosion and landslides (thermal erosion)")
- Reference check: (ERO, ISO, HYP) Crossref matches title, authors, volume 43, issue 2. The sources file gives only "Tzathas et al. 2024" with the PDF link; the title above is the PDF's. (AMP, GLA) authors, year and venue match. The volume (43(2)) and article number come from the DOI and search results. Whether the paper uses atan 30 degrees (0.57) could not be read.

### Jain2024
- Reference: Jain, A., Kerbl, B., Gain, J., Finley, B. and Cordonnier, G. (2024). FastFlow: GPU acceleration of flow and depression routing for landscape simulation. Computer Graphics Forum 43(7) (Pacific Graphics 2024).
- DOI or URL: 10.1111/cgf.15243; author PDF https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf.
- Cited as: "Jain et al. 2024 FastFlow".
- Ledger ids: HYP-031
- Access: open copy at the Inria URL above.
- Library (the project's source library, not distributed): fetched (PDF, 13 pages, text layer, 57,736 characters). From: https://www-sop.inria.fr/reves/Basilic/2024/JKGFC24/FastFlowPG2024_Author_Version.pdf. Text file: Jain2024.txt.
- Read so far (by the project): the note quotes timings from it.
- Read in this pass: abstract fetched (title page and abstract only).
- Reference check: Crossref matches title, authors, volume 43, issue 7.

### Wikipedia-Laterite
- Reference: Wikipedia contributors, "Laterite", https://en.wikipedia.org/wiki/Laterite (web page, version as of 2026-10-01).
- DOI or URL: https://en.wikipedia.org/wiki/Laterite.
- Cited as: "Wikipedia: Laterite" (research note section 3).
- Ledger ids: ERO-073
- Access: open access.
- Library (the project's source library, not distributed): fetched (HTML page, 29,180 characters of text). From: https://en.wikipedia.org/wiki/Laterite. Text file: Wikipedia-Laterite.txt.
- Read so far (by the project): cited as a web source.
- Read in this pass: full page fetched (formation conditions).
- Reference check: not a scientific source; the statement attributed to it is present (S21).

### Braun2016
- Reference: Braun, J., Mercier, J., Guillocheau, F. and Robin, C. (2016). A simple model for regolith formation by chemical weathering. Journal of Geophysical Research: Earth Surface 121(11):2140-2171.
- DOI or URL: 10.1002/2016JF003914.
- Cited as: "Braun et al. 2016, 'regolith by chemical weathering'" (research note only).
- Ledger ids: none (not used in code); notes: not used in code
- Access: paywalled; not fetched.
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://gfzpublic.gfz-potsdam.de/pubman/item/item_2130890. File: Braun2016.txt.
- Read so far (by the project): abstract via OpenAlex; coefficients not accessible.
- Read in this pass: not fetched.
- Reference check: Crossref gives the full title above (the note's phrase "regolith by chemical weathering" is a fragment). Authors: Braun, Mercier, Guillocheau, Robin.

### Tozer2019
- Reference: Tozer, B., Sandwell, D. T., Smith, W. H. F., Olson, C., Beale, J. R. and Wessel, P. (2019). Global bathymetry and topography at 15 arc sec: SRTM15+. Earth and Space Science 6(10):1847-1864.
- DOI or URL: (ERO, ISO, HYP) 10.1029/2019EA000658 (gold open access; DOAJ record https://doaj.org/article/95db1757cac2402aaa9c72408bdc82ed). (MOI, BIO) https://doi.org/10.1029/2019EA000658 (gold open access) (PLT) 10.1029/2019EA000658 (open access in Wiley). (CLM) 10.1029/2019EA000658
- Cited as: (ERO, ISO, HYP) "Tozer et al. 2019" (SRTM15+) in build log 2026-10-01. (MOI, BIO) "GMT earth_relief (SRTM15+, Tozer et al. 2019)" (PLT) "Tozer et al. 2019 (SRTM15+, GMT earth_relief 30 arc-minute)".
- Ledger ids: HYP-001 to HYP-011, HYP-032, PLT-168, PLT-170; none (relief used in the Earth comparisons); notes: as the dataset behind GMT earth_relief; relief used in the Earth comparisons
- Access: open access. [other sections: open access (Wiley/AGU). / not checked (journal is open access)]
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): (ERO, ISO, HYP) no record. (MOI, BIO) used as data (PLT) numbers computed by the project from the GMT grid. (CLM) data used
- Read in this pass: (ERO, ISO, HYP) not fetched. (MOI, BIO) bibliographic record only (PLT) abstract fetched. The abstract describes the dataset; it contains no land-fraction or hypsometry numbers.
- Reference check: (ERO, ISO, HYP) Crossref matches. GMT's `earth_relief_01d` and the 30 arc-minute grid are resampled products of SRTM15+; the paper itself publishes no land fraction or median elevation, so the project's Earth numbers are its own computations on the grid (not independently checkable from the paper). (MOI, BIO) matches (authors, year, volume, pages). (PLT) correct. The project's Earth numbers (land 0.289, median 427 m) are its own computation and cannot be checked against this paper; they depend on the 30 arc-minute resampling. (CLM) matches.

### Wessel2019-GMT
- Reference: Wessel, P., Luis, J. F., Uieda, L., Scharroo, R., Wobbe, F., Smith, W. H. F. and Tian, D. (2019). The Generic Mapping Tools version 6. Geochemistry, Geophysics, Geosystems 20(11):5556-5564.
- DOI or URL: 10.1029/2019GC008515.
- Cited as: "GMT earth_relief" (no paper cited by the project; this is GMT's standard citation).
- Ledger ids: HYP-001 to HYP-011, HYP-032
- Access: not checked (Crossref only).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): the files were pulled as `earth_720.f32`; the 1 degree vs 30 arc-minute naming is inconsistent (flag 13).
- Read in this pass: not fetched.
- Reference check: added for the dataset; Crossref matches the title and pages above.

### Fastscapelib
- Reference: fastscapelib 0.3.0, fastscape-lem (GPL-3.0). Its SPLEroder header says "The method is detailed in Braun and Willet's (2013) and has been slightly adapted."
- DOI or URL: https://github.com/fastscape-lem/fastscapelib (header read: `include/fastscapelib/eroders/spl.hpp`, main branch; `LICENSE`, GPL version 3).
- Cited as: "fastscapelib 0.3.0 (GPL-3.0)" (fastscape_compare.py, spl.rs).
- Ledger ids: HYP-028, HYP-029
- Access: open access (source).
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 3833 chars).
- Read so far (by the project): documented Python API only; the C++ was not read (the project's rule).
- Read in this pass: source header and constructor signature fetched; no default K, m or n exists (only `tolerance = 1e-3`).
- Reference check: licence GPL-3.0 confirmed. See S20.

### Telford1990
- Reference: Telford, W. M., Geldart, L. P. and Sheriff, R. E. (1990). Applied Geophysics, 2nd edition. Cambridge University Press.
- DOI or URL: (ERO, ISO, HYP) 10.1017/CBO9781139167932 (Crossref record, published 1990-10-26). (RCK, STR, DEF) no DOI. Library copy (lending only, not readable by fetch): https://archive.org/details/appliedgeophysic0000telf (djvu text returned HTTP 401).
- Cited as: (ERO, ISO, HYP) "Telford, Geldart & Sheriff 1990 (Applied Geophysics, 2nd ed.)" (rock_class JSON sources.density_kg_m3). (RCK, STR, DEF) "Telford, Geldart & Sheriff 1990 (Applied Geophysics, 2nd ed.)"; "the rock-density tables of ..."
- Ledger ids: RCK-001; none in ERO/ISO/HYP (density rows belong to another prefix); notes: density rows belong to another prefix
- Access: book (not online). [other sections: book (not online; archive.org lending copy only)]
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://archive.org/details/appliedgeophysic0000telf. File: Telford1990.txt.
- Read so far (by the project): (ERO, ISO, HYP) "cf." only. (RCK, STR, DEF) not stated
- Read in this pass: (ERO, ISO, HYP) not fetched. (RCK, STR, DEF) not fetched (bibliographic record only)
- Reference check: (ERO, ISO, HYP) matches title and edition. (RCK, STR, DEF) authors, year, edition match the Open Library record. No table or page number given in the code, so the density table cannot be pinned (the density table is in the physical-properties chapter; table number not verified).

### Schon2015
- Reference: Schoen, J. H. (2015). Physical Properties of Rocks: Fundamentals and Principles of Petrophysics, 2nd edition. Developments in Petroleum Science 65, Elsevier.
- DOI or URL: (ERO, ISO, HYP) 10.1016/C2014-0-03002-X (Crossref); chapter DOIs use 10.1016/B978-0-08-100404-3. (RCK, STR, DEF) no DOI found. Open Library: https://openlibrary.org/isbn/9780081004043
- Cited as: (ERO, ISO, HYP) "Schoen 2015 (Physical Properties of Rocks, 2nd ed.)" (rock_class JSON). (RCK, STR, DEF) "Schön 2015 (Physical Properties of Rocks, 2nd ed.)"
- Ledger ids: RCK-001; none in ERO/ISO/HYP
- Access: book (not online).
- Library (the project's source library, not distributed): not fetched. Reason: book; no open copy named.
- Read so far (by the project): (ERO, ISO, HYP) "cf." only. (RCK, STR, DEF) not stated
- Read in this pass: (ERO, ISO, HYP) not fetched. (RCK, STR, DEF) not fetched (bibliographic record only)
- Reference check: (ERO, ISO, HYP) title and year match the Crossref record; the series number 65 is from memory and was not verified. (RCK, STR, DEF) title and edition exist; year 2015 vs 2016 record unresolved. No table or page number in the code.

### Not cited by the code but named in the task brief (identified for stage 2)
- Whipple, K. X. and Tucker, G. E. (1999). Dynamics of the stream-power river incision model: Implications for height limits of mountain ranges, landscape response timescales, and research needs. Journal of Geophysical Research 104(B8):17661-17674. DOI 10.1029/1999JB900120 (Crossref confirmed). Paywalled; not fetched.
- Fernandes, N. F. and Dietrich, W. E. (1997). Hillslope evolution by diffusive processes: The timescale for equilibrium adjustments. Water Resources Research 33(6):1307-1318. DOI 10.1029/97WR00534 (Crossref confirmed). Paywalled; not fetched.
- Amante, C. and Eakins, B. W. (2009). ETOPO1 1 arc-minute global relief model: procedures, data sources and analysis. NOAA Technical Memorandum NESDIS NGDC-24. Dataset DOI 10.7289/V5C8276M (DataCite: "ETOPO1 1 Arc-Minute Global Relief Model", NOAA NCEI, 2009). Open.
- Eakins, B. W. and Sharman, G. F. (2012). Hypsographic curve of Earth's surface from ETOPO1. NOAA National Geophysical Data Center, Boulder, CO. PDF https://www.ncei.noaa.gov/sites/default/files/2023-01/Hypsographic%20Curve%20of%20Earth%E2%80%99s%20Surface%20from%20ETOPO1.pdf (full text read; Table 1 is an image and was not readable). Companion: "World Ocean Volumes" https://www.ncei.noaa.gov/sites/default/files/2023-01/World%20Ocean%20Volumes.pdf (table read).
- Athy, L. F. (1930). Density, porosity, and compaction of sedimentary rocks. AAPG Bulletin 14(1):1-24. DOI 10.1306/3D93289E-16B1-11D7-8645000102C1865D. Not coded: the surface stage has no porosity or compaction (ERO-028).
- Sclater, J. G. and Christie, P. A. F. (1980). Continental stretching: an explanation of the post-mid-Cretaceous subsidence of the central North Sea basin. Journal of Geophysical Research 85(B7):3711-3739. DOI 10.1029/JB085iB07p03711. Not coded, as above.

### Not bibliographic
RichDEM and Barnes2013-Depressions (software, "were not opened", not relied on); docs/research/notes/surface_processes_climate.md; docs/deeptime-implementation-plan.md; docs/deeptime-design-decisions.md; STATUS.md build-log entries; rock_class JSON files; tools/review/p1-17-hyps.py and companions; the decisions of Ben (Tectonics scaling, Build range, Vertical scale, Sub-grid detail, squeeze, general flexure).

---

## Rivers, lakes and the young landscape (HYD, YNG)

### Leopold1953
- Reference: Leopold, Luna B. and Maddock, Thomas, Jr. 1953. The hydraulic geometry of stream channels and some physiographic implications. U.S. Geological Survey Professional Paper 252, 57 p.
- DOI or URL: https://doi.org/10.3133/pp252 ; open PDF https://pubs.usgs.gov/pp/0252/report.pdf
- Cited as: "Leopold & Maddock 1953: width ∝ Q^0.5, depth ∝ Q^0.4" (hydro/mod.rs comment; plan §8.4)
- Ledger ids: HYD-001 to HYD-004; notes: HYD-003, HYD-004 are the project's own coefficients
- Access: open access (USGS Publications Warehouse, scanned PDF)
- Library (the project's source library, not distributed): fetched (PDF, 64 pages, text layer, 190,737 characters). From: https://pubs.usgs.gov/pp/0252/report.pdf. Text file: Leopold1953.txt.
- Read so far (by the project): nothing says the paper was read
- Read in this pass: full text fetched (OCR text layer of the PDF; the passage describing Figure 19, the paragraph introducing equations (1) to (3), and the legend of the figure on the next page)
- Reference check: matches (authors, year, series number). The paper is a Professional Paper, not a journal article. Exponent values: see Part 2, HYD-001 and HYD-002.

### Leopold1957
- Reference: Leopold, Luna B. and Wolman, M. Gordon. 1957. River channel patterns: braided, meandering, and straight. U.S. Geological Survey Professional Paper 282-B, pp. 39-85.
- DOI or URL: https://doi.org/10.3133/pp282B ; open PDF https://pubs.usgs.gov/pp/0282b/report.pdf
- Cited as: not cited by the project. Proposed as the citable source for the meandering threshold (HYD-027).
- Ledger ids: HYD-027; notes: proposed
- Access: open access (USGS)
- Library (the project's source library, not distributed): fetched (PDF, 50 pages, text layer, 147,959 characters). From: https://pubs.usgs.gov/pp/0282b/report.pdf. Text file: Leopold1957.txt.
- Read so far (by the project): not read (project wrote the Earth figure "from memory, not sourced")
- Read in this pass: full text fetched (OCR text layer; the sections on sinuosity and Figure 46)
- Reference check: n/a (new source). Values: see Part 2, HYD-027.

### Schumm1963
- Reference: Schumm, Stanley A. 1963. Sinuosity of alluvial rivers on the Great Plains. Geological Society of America Bulletin 74(9):1089-1100.
- DOI or URL: https://doi.org/10.1130/0016-7606(1963)74[1089:SOAROT]2.0.CO;2 ; record https://pubs.usgs.gov/publication/70221081
- Cited as: not cited by the project (proposed for HYD-027)
- Ledger ids: HYD-027; notes: proposed
- Access: only the abstract was reachable; open-access status not checked
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://pubs.usgs.gov/publication/70221081. File: Schumm1963.txt.
- Read so far (by the project): not read
- Read in this pass: abstract fetched (USGS record). The abstract classifies 50 stable Great Plains rivers into five pattern types (tortuous, irregular, regular, transitional, straight) by sinuosity but gives no sinuosity numbers.
- Reference check: n/a.

### Barnes2014b
- Reference: Barnes, Richard; Lehman, Clarence; Mulla, David. 2014. An efficient assignment of drainage direction over flat surfaces in raster digital elevation models. Computers & Geosciences 62:128-135.
- DOI or URL: https://doi.org/10.1016/j.cageo.2013.01.009 ; open preprint https://richard.science/sci/2014_flats.pdf
- Cited as: "Garbrecht & Martz 1997, as improved by Barnes, Lehman & Mulla 2014 ... Computers & Geosciences 62:128-135" (network.rs doc of flat_masks)
- Ledger ids: HYD-015
- Access: open copy (author preprint); journal version paywalled
- Library (the project's source library, not distributed): fetched (PDF, 13 pages, text layer, 40,532 characters). From: https://richard.science/sci/2014_flats.pdf. Text file: Barnes2014b.txt.
- Read so far (by the project): "from the paper"
- Read in this pass: full text fetched (author preprint: abstract, Steps 1 to 3 text)
- Reference check: matches exactly (title, authors, volume, pages). Algorithm check: see Part 2, HYD-015.

### Garbrecht1997
- Reference: Garbrecht, Jurgen and Martz, Lawrence W. 1997. The assignment of drainage direction over flat surfaces in raster digital elevation models. Journal of Hydrology 193(1-4):204-213.
- DOI or URL: https://doi.org/10.1016/S0022-1694(96)03138-1
- Cited as: "Garbrecht & Martz 1997" (named only through the Barnes paper)
- Ledger ids: HYD-015
- Access: paywalled (not opened; closed per OpenAlex)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 11 chars).
- Substitute: none found.
- Read so far (by the project): named only via Barnes
- Read in this pass: not fetched (bibliographic record only, via OpenAlex)
- Reference check: record matches the name and year; the project gave no title or pages.

### Barnes2021
- Reference: Barnes, Richard; Callaghan, Kerry L.; Wickert, Andrew D. 2021. Computing water flow through complex landscapes, Part 3: Fill-Spill-Merge: flow routing in depression hierarchies. Earth Surface Dynamics 9(1):105-121.
- DOI or URL: https://doi.org/10.5194/esurf-9-105-2021 (open access)
- Cited as: "Fill-Spill-Merge, Barnes, Callaghan & Wickert 2021 (ESurf 9:105)"; code r-barnes/Barnes2020-FillSpillMerge (MIT)
- Ledger ids: HYD-049, HYD-050
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 17 pages, text layer, 61,225 characters). From: https://esurf.copernicus.org/articles/9/105/2021/esurf-9-105-2021.pdf. Text file: Barnes2021.txt.
- Read so far (by the project): research note quotes a sentence; nothing says full text read
- Read in this pass: abstract fetched (Crossref record)
- Reference check: matches (authors, year, journal, volume, first page). Title has "Part 3" (project omitted it).

### Holdridge1967
- Reference: Holdridge, Leslie R. 1967. Life Zone Ecology (revised edition). Tropical Science Center, San Jose, Costa Rica.
- DOI or URL: none (book). A library scan is at https://app.ingemmet.gob.pe/biblioteca/pdf/Amb-56.pdf (title per its listing: "Life Zone Ecology by L.R. Holdridge, with photographic supplement prepared by Joseph A. Tosi Jr."). The scan's text layer starts at the table of contents, so I could not confirm the edition year from the scan itself; Lugo et al. 1999 cite it as "Holdridge, L.R. (1967) Life zone ecology. Tropical Science Center. San Jose, Costa Rica".
- Cited as: "L. R. Holdridge 1967, Life Zone Ecology, Tropical Science Center: PET = 58.93 x the mean annual biotemperature"
- Ledger ids: HYD-029, HYD-030, HYD-035, HYD-041
- Access: book (not online), plus an unofficial library scan
- Library (the project's source library, not distributed): fetched (PDF, 149 pages, text layer, 330,833 characters). From: https://app.ingemmet.gob.pe/biblioteca/pdf/Amb-56.pdf. Text file: Holdridge1967.txt.
- Read so far (by the project): nothing says it was consulted
- Read in this pass: full text fetched (scan OCR; the passages on biotemperature and on the factor 58.93)
- Reference check: the title, publisher and the 58.93 factor match. The definition of biotemperature does not match the code: see Part 2, HYD-030.

### Lugo1999
- Reference: Lugo, A. E.; Brown, S. L.; Dodson, R.; Smith, T. S.; Shugart, H. H. 1999. The Holdridge life zones of the conterminous United States in relation to ecosystem mapping. Journal of Biogeography 26:1025-1038.
- DOI or URL: copy at https://epa-prgs.ornl.gov/radionuclides/documents/Holdridge_LifeZones.pdf
- Cited as: not cited by the project (found in this pass; secondary confirmation of the Holdridge formulae)
- Ledger ids: HYD-029, HYD-030; notes: confirmation
- Access: open copy (ORNL/EPA host)
- Library (the project's source library, not distributed): fetched (PDF, 15 pages, text layer, 49,293 characters). From: https://epa-prgs.ornl.gov/radionuclides/documents/Holdridge_LifeZones.pdf. Text file: Lugo1999.txt.
- Read so far (by the project): not read
- Read in this pass: full text fetched (partial: methods and the figure caption)
- Reference check: n/a.

### Zhao2022
- Reference: Zhao, Gang; Li, Yao; Zhou, Liangzhi; Gao, Huilin. 2022. Evaporative water loss of 1.42 million global lakes. Nature Communications 13:3686.
- DOI or URL: https://doi.org/10.1038/s41467-022-31125-6 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC9240014/ ; PDF copy https://www.atmos.albany.edu/facstaff/zhou/pdf/pdf_papers/Zhao_et_al_2022.pdf
- Cited as: "Zhao et al. 2022 ... 1,500 +- 150 km3 a year without the Caspian, about 650 mm over 2.3 million km2"
- Ledger ids: HYD-040, HYD-043, HYD-065
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 10 pages, text layer, 48,974 characters). From: https://www.atmos.albany.edu/facstaff/zhou/pdf/pdf_papers/Zhao_et_al_2022.pdf. Text file: Zhao2022.txt.
- Read so far (by the project): nothing says how it was read
- Read in this pass: full text fetched (Results and Methods text; read from the PDF text layer and from PMC)
- Reference check: authors, year, journal, article number match. The project's "650 mm" and "2.3 million km2" are not in the text: see Part 2, HYD-040.

### Fu1981
- Reference: Fu, B. P. 1981. On the calculation of the evaporation from land surface (in Chinese). Scientia Atmospherica Sinica 5(1):23-31. (This line is from my recollection; I could not look it up, no source found with the tools left.)
- DOI or URL: none found
- Cited as: "Fu 1981; Zhang et al. 2004" (surface/climate.rs: Fu's form of the Budyko curve, omega 2.6) (from the joint entry Fu1981 and Zhang2004: "Fu 1981; Zhang et al. 2004" (surface/climate.rs header and fu_et))
- Ledger ids: HYD-034, HYD-035, YNG-011; notes: also MOI-068
- Access: not found
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Not resolved beyond author and year: Crossref has no record of a 1981 paper by B. P. Fu in Scientia Atmospherica Sinica. Zhang et al. 2004 (key Zhang2004) cite it as the source of the one-parameter Budyko form and are the route to the equation.
- Read so far (by the project): "From papers"; nothing more
- Read in this pass: not fetched
- Reference check: UNIDENTIFIED beyond author and year in what I could verify. I tried OpenAlex by DOI (the paper has none) and Wikipedia (404). The equation as coded cannot be checked against this paper. (from the joint entry Fu1981 and Zhang2004: see the HYD file. The Zhang abstract does not mention Fu's equation or omega = 2.6.)

### Zhang2004
- Reference: Zhang, Lu; Hickel, Klaus; Dawes, Warrick R.; Chiew, Francis H. S.; Western, Andrew W.; Briggs, Peter R. 2004. A rational function approach for estimating mean annual evapotranspiration. Water Resources Research 40(2), W02502.
- DOI or URL: https://doi.org/10.1029/2003WR002710
- Cited as: "Fu 1981; Zhang et al. 2004" (from the joint entry Fu1981 and Zhang2004: "Fu 1981; Zhang et al. 2004" (surface/climate.rs header and fu_et))
- Ledger ids: HYD-034, HYD-035, YNG-011; notes: also MOI-068
- Access: only the abstract was reachable; open-access status not checked
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): nothing stated
- Read in this pass: abstract fetched (OpenAlex). The abstract is about a rational-function model of ET/PET; it does not mention Fu's equation or omega.
- Reference check: authors, year, journal, volume match. Whether the paper contains "omega = 2.6" is not shown by the abstract. Needed for stage 2 (paywalled). (from the joint entry Fu1981 and Zhang2004: see the HYD file. The Zhang abstract does not mention Fu's equation or omega = 2.6.)

### Ohmura1992
- Reference: Ohmura, Atsumu; Kasser, Peter; Funk, Martin. 1992. Climate at the equilibrium line of glaciers. Journal of Glaciology 38(130):397-411.
- DOI or URL: (HYD, YNG) https://doi.org/10.3189/S0022143000002276 ; Cambridge Core page https://www.cambridge.org/core/product/identifier/S0022143000002276/type/journal_article (AMP, GLA) 10.3189/S0022143000002276 (Cambridge page https://www.cambridge.org/core/journals/journal-of-glaciology/article/climate-at-the-equilibrium-line-of-glaciers/8DCF1C24E32A6781CC365853535CD16F) (MOI, BIO) https://doi.org/10.3189/S0022143000002276 (CLM) 10.3189/S0022143000002276; open PDF on Cambridge Core (see Unpaywall: bronze)
- Cited as: (HYD, YNG) "Ohmura et al. 1992 ... about -2 degC" (HYD-039); "Ohmura, Kasser & Funk 1992 (ELA fit P = 645 + 296 T + 9 T^2)" (MOI/BIO) (AMP, GLA) "Ohmura, Kasser & Funk 1992" (MOI, BIO) "Ohmura, Kasser & Funk 1992 (ELA fit P = 645 + 296 T + 9 T^2)" (glaciers.rs header, params.rs) (CLM) "Ohmura, Kasser & Funk 1992" (P = 645 + 296 T + 9 T^2, 70 glaciers)
- Ledger ids: BIO-083, GLA-031, GLA-037, GLA-039, HYD-039; notes: and HYD-039 in the HYD file; and glaciers.rs
- Access: open access (Cambridge Core) [other sections: open access (Cambridge archive; the page shows the full text) / open access (page readable without login) / open access (bronze)]
- Library (the project's source library, not distributed): fetched (PDF, 15 pages, text layer, 55,097 characters). From: https://www.cambridge.org/core/services/aop-cambridge-core/content/view/8DCF1C24E32A6781CC365853535CD16F/S0022143000002276a.pdf/div-class-title-climate-at-the-equilibrium-line-of-glaciers-div.pdf. Text file: Ohmura1992.txt.
- Read so far (by the project): (HYD, YNG) "cited from memory, not re-read" (AMP, GLA) from memory, not re-read (STATUS 2026-10-01) (MOI, BIO) "cited from memory, not re-read" for the -2 degC figure (CLM) the build log says the equilibrium-line summer temperature was "cited from memory, not re-read" (hydrology agent's check of a related number)
- Read in this pass: (HYD, YNG) page fetched via summariser (equation and one sentence). My prompt contained the equation, so treat the quote as needing a second look. (AMP, GLA) full text partly fetched (page text via WebFetch: abstract, the regression sentence, the temperature definition, the standard error; Table 3 was not readable) (MOI, BIO) page fetched (summariser); see the HYD file for the check (CLM) full text fetched (OCR text layer; the regression equation itself is not machine-readable)
- Reference check: (HYD, YNG) authors, year, journal, volume and pages match. (AMP, GLA) authors, year, journal, volume, issue and pages match. (MOI, BIO) matches. (CLM) matches. Its T is the mean summer temperature in the FREE ATMOSPHERE at the ELA, not a surface or sea-level temperature (see Part 2).

### Messager2016
- Reference: Messager, Mathis L.; Lehner, Bernhard; Grill, Gunther; Nedeva, Irena; Schmitt, Oliver. 2016. Estimating the volume and age of water stored in global lakes using a geo-statistical approach. Nature Communications 7:13603.
- DOI or URL: https://doi.org/10.1038/ncomms13603 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC5171767/
- Cited as: "Messager et al. 2016, Nat. Commun. 7:13603 (HydroLAKES: 1.42 million lakes of 10 ha and more, 2.67 million km2, 1.8 % of land)"
- Ledger ids: HYD-063
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 11 pages, text layer, 59,360 characters). From: https://www.nature.com/articles/ncomms13603.pdf. Text file: Messager2016.txt.
- Read so far (by the project): nothing stated
- Read in this pass: full text fetched (PMC, summariser: lake count, area, land share, comparison with Verpoorter)
- Reference check: matches. Note HydroLAKES has 1.43 million polygons, of which 1.42 million are natural lakes (the 1.42 million is natural lakes only).

### Verpoorter2014
- Reference: Verpoorter, Charles; Kutser, Tiit; Seekell, David A.; Tranvik, Lars J. 2014. A global inventory of lakes based on high-resolution satellite imagery. Geophysical Research Letters 41(18):6396-6402.
- DOI or URL: https://doi.org/10.1002/2014GL060641 (hybrid open access per OpenAlex); HAL record https://insu.hal.science/hal-01066344
- Cited as: "Verpoorter et al. 2014, GRL 41:6396 (117 million lakes over 0.002 km2, 5.0 million km2, 3.7 % of non-glaciated land)"
- Ledger ids: HYD-062, HYD-063
- Access: open access (hybrid)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): the run review lists it among linked sources; nothing says the paper was read
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: matches (authors, year, journal, volume, pages, DOI).

### Downing2006
- Reference: Downing, John A.; Prairie, Yves T.; Cole, Jonathan J.; Duarte, Carlos M.; Tranvik, Lars J.; Striegl, Robert G.; McDowell, William H.; Kortelainen, Pirkko; Caraco, Nina F.; Melack, John M.; Middelburg, Jack J. 2006. The global abundance and size distribution of lakes, ponds, and impoundments. Limnology and Oceanography 51(5):2388-2397.
- DOI or URL: https://doi.org/10.4319/lo.2006.51.5.2388
- Cited as: not cited by the project (named in the task as a third lake-share estimate)
- Ledger ids: HYD-062, HYD-063; notes: comparison
- Access: only the abstract was reachable (USGS record https://pubs.usgs.gov/publication/70028401); open-access status not checked
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://pubs.usgs.gov/publication/70028401. File: Downing2006.txt.
- Read so far (by the project): n/a
- Read in this pass: abstract fetched
- Reference check: n/a.

### MiddletonThomas1997
- Reference: Middleton, Nick and Thomas, David (eds.). 1997. World Atlas of Desertification, 2nd edition. UNEP / Arnold, London. (First edition: UNEP 1992, compiled by Middleton and Thomas.) The Wikipedia article on the aridity index cites "UNEP (1992) World Atlas of Desertification"; Zomer et al. 2022 cite "UNEP. World atlas of desertification - Second Edition (United Nations Environment Program, 1997)". I did not see the atlas.
- DOI or URL: none
- Cited as: "UNEP's arid and hyper-arid classes" (network.rs ARID_INDEX); "UNEP's hyper-arid, arid, semi-arid, dry sub-humid, humid" (biomes.rs) (from the joint entry MiddletonThomas1997 and Zomer2022: "UNEP's hyper-arid, arid, semi-arid, dry sub-humid, humid" (biomes.rs header, placement.json _comment); no document named)
- Ledger ids: BIO-038, HYD-045; notes: and BIO-038
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): nothing; no document named
- Read in this pass: not fetched; values read from Zomer2022 (below) and Wikipedia
- Reference check: UNIDENTIFIED in the code (no document named); identified here as UNEP 1992 (first edition) or 1997 (second edition). The two editions differ in the hyper-arid limit: see Part 2, HYD-045. (from the joint entry MiddletonThomas1997 and Zomer2022: UNIDENTIFIED in the code; identified here as UNEP 1992 or 1997. Class limits differ between the two editions: see Part 2, BIO-038.)

### Zomer2022
- Reference: Zomer, Robert J.; Xu, Jianchu; Trabucco, Antonio. 2022. Version 3 of the Global Aridity Index and Potential Evapotranspiration Database. Scientific Data 9:409.
- DOI or URL: https://doi.org/10.1038/s41597-022-01493-1 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC9287331/
- Cited as: not cited by the project (open access source for the UNEP class table) (from the joint entry MiddletonThomas1997 and Zomer2022: "UNEP's hyper-arid, arid, semi-arid, dry sub-humid, humid" (biomes.rs header, placement.json _comment); no document named)
- Ledger ids: BIO-038, HYD-045; notes: confirmation
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 15 pages, text layer, 46,810 characters). From: https://www.nature.com/articles/s41597-022-01493-1.pdf. Text file: Zomer2022.txt.
- Read so far (by the project): n/a
- Read in this pass: full text fetched (PMC, summariser: the class table and the definition AI = MA_Prec/MA_ET0)
- Reference check: n/a (article number 409 is from my memory of the record, not seen; Crossref/EuropePMC gave volume 9 and year 2022). (from the joint entry MiddletonThomas1997 and Zomer2022: UNIDENTIFIED in the code; identified here as UNEP 1992 or 1997. Class limits differ between the two editions: see Part 2, BIO-038.)

### Porter1989
- Reference: Porter, Stephen C. 1989. Some geological implications of average Quaternary glacial conditions. Quaternary Research 32(3):245-261.
- DOI or URL: (HYD, YNG) https://doi.org/10.1016/0033-5894(89)90092-6 (AMP, GLA) 10.1016/0033-5894(89)90092-6
- Cited as: (HYD, YNG) "Porter 1989 (Quat. Res. 32:245-261): snowlines some 500 m lower than today" (glacial.rs; plan §8.2) (AMP, GLA) "Porter 1989, Quat. Res. 32:245-261"
- Ledger ids: GLA-030
- Access: only the abstract was reachable; open-access status not checked [other sections: paywalled (abstract only, via OpenAlex)]
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://doi.org/10.1016/0033-5894(89)90092-6. File: Porter1989.txt.
- Read so far (by the project): (HYD, YNG) nothing stated (AMP, GLA) not stated
- Read in this pass: (HYD, YNG) abstract fetched (OpenAlex, reconstructed from the inverted index) (AMP, GLA) abstract fetched
- Reference check: (HYD, YNG) matches (the project's title is absent; authors, year, volume, pages match). (AMP, GLA) matches.

### Porter2001
- Reference: Porter, Stephen C. 2001. Snowline depression in the tropics during the Last Glaciation. Quaternary Science Reviews 20(10):1067-1091. (OpenAlex lists the year as 2000, the online date; volume 20 issue 10 is the 2001 print.)
- DOI or URL: (HYD, YNG) https://doi.org/10.1016/S0277-3791(00)00178-5 (AMP, GLA) 10.1016/S0277-3791(00)00178-5
- Cited as: (HYD, YNG) "Porter 2001" for "~900 m at the last glacial maximum" (glacial.rs comment) (AMP, GLA) "Porter 2001 (about 900 m at the last glacial maximum)"
- Ledger ids: GLA-030
- Access: paywalled [other sections: paywalled; abstract not available through OpenAlex]
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 11 chars).
- Substitute: none found.
- Read so far (by the project): (HYD, YNG) nothing stated (AMP, GLA) not stated; no bibliographic detail was given
- Read in this pass: (HYD, YNG) not fetched (OpenAlex has no abstract). The paper is about the tropics; whether it gives a global ~900 m figure is not established. (AMP, GLA) not fetched (metadata only)
- Reference check: (HYD, YNG) title and journal match what the code's "Porter 2001" would be; the 900 m claim is unchecked. (AMP, GLA) this is my identification of an otherwise vague citation, found by author and year. The paper is about the TROPICS, while the code uses "about 900 m" as a general last-glacial-maximum snowline lowering. The 900 m figure could not be checked. Identification is likely but not confirmed.

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

---

## The amplifier and glacial landforms (AMP, GLA)

### Hergarten2021
- Reference: Hergarten, S. (2021). Modeling glacial and fluvial landform evolution at large scales using a stream-power approach. Earth Surface Dynamics 9: 937-952.
- DOI or URL: https://esurf.copernicus.org/articles/9/937/2021/ (open access)
- Cited as: "Hergarten 2021, Earth Surf. Dynam. 9:937-952"
- Ledger ids: GLA-009, GLA-022, GLA-025
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 16 pages, text layer, 63,223 characters). From: https://esurf.copernicus.org/articles/9/937/2021/esurf-9-937-2021.pdf. Text file: Hergarten2021.txt.
- Read so far (by the project): not stated
- Read in this pass: full text fetched (article page, two queries; relayed quotes of Eqs 10-13, 19, 23)
- Reference check: authors, year, journal, volume and pages match. Content check: see spot checks GLA-009 and GLA-022; the paper's glacial law is stream power in ice flux, not the sliding-speed law the code uses.

### goSPL
- Reference: goSPL documentation, technical guide "ice" (Geodels/gospl on GitHub; GPL). Software; no paper cited.
- DOI or URL: https://github.com/Geodels/gospl (raw: docs/tech_guide/ice.rst)
- Cited as: "goSPL documentation (ice.rst, ela_from_temperature.py)"
- Ledger ids: GLA-009, GLA-036
- Access: open (documentation)
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://github.com/Geodels/gospl. File: goSPL.txt.
- Read so far (by the project): documentation only
- Read in this pass: full text of ice.rst fetched. ela_from_temperature.py not fetched.
- Reference check: the quote about the removed SIA solve is exact (see spot check). The temperature-based ELA rule the code credits to goSPL did not appear in ice.rst (which gives a precipitation-ramp mass balance), so it presumably lives in ela_from_temperature.py, not fetched.

### Oerlemans2011
- Reference: Oerlemans, J. Minimal Glacier Models. Utrecht Publishing & Archiving Services (Igitur), Utrecht. OpenAlex lists the book as 2008 with a handle https://dspace.library.uu.nl/handle/1874/224622; the project says 2011 (probably the second edition).
- DOI or URL: handle 1874/224622 (returned 403)
- Cited as: "Oerlemans 2011, Minimal Glacier Models"
- Ledger ids: GLA-005, GLA-006
- Access: book (open PDF at Utrecht, not reachable by my tool)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: edition and year unverified (2008 versus 2011).

### Cuffey2010
- Reference: Cuffey, K. M. and Paterson, W. S. B. (2010). The Physics of Glaciers, 4th edition. Butterworth-Heinemann (Elsevier/Academic Press), 693 pp. (ISBN 978-0-12-369461-4).
- DOI or URL: (AMP, GLA) none (CLM) none found (Crossref returned only chapters of the earlier editions). ISBN not looked up.
- Cited as: (AMP, GLA) "Cuffey & Paterson 2010, The Physics of Glaciers, 4th ed., section 8" (CLM) "Cuffey & Paterson 2010, The Physics of Glaciers, section 8.7"
- Ledger ids: CLM-101, GLA-010 to GLA-012
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: (AMP, GLA) not fetched; the rate-factor value was read in a paper that cites it (Prasicek et al. 2018, see spot checks) (CLM) not fetched
- Reference check: (AMP, GLA) the book exists as cited. The location "section 8" was not checked (I did not see the book's contents). (CLM) exists (4th edition 2010). Section number not checked.

### Harbor1992
- Reference: Harbor, J. M. (1992). Numerical modeling of the development of U-shaped valleys by glacial erosion. Geological Society of America Bulletin 104(10): 1364-1375.
- DOI or URL: 10.1130/0016-7606(1992)104<1364:NMOTDO>2.3.CO;2
- Cited as: "Harbor 1992, GSA Bull. 104:1364-1375"
- Ledger ids: GLA-001, GLA-003, GLA-054, GLA-055
- Access: paywalled (abstract only, via OpenAlex)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, journal, volume, issue and pages match. The abstract does not state the exponent l = 2 (see spot check).

### Humphrey1994
- Reference: Humphrey, N. F. and Raymond, C. F. (1994). Hydrology, erosion and sediment production in a surging glacier: Variegated Glacier, Alaska, 1982-83. Journal of Glaciology 40(136): 539-552.
- DOI or URL: 10.1017/S0022143000012429 (OpenAlex; the Cambridge page for this journal also resolves under 10.3189). Journal of Glaciology back issues are open.
- Cited as: "Humphrey & Raymond 1994, J. Glaciol. 40:539-552"
- Ledger ids: GLA-001, GLA-003
- Access: open access (Cambridge archive), abstract fetched
- Library (the project's source library, not distributed): fetched (HTML page, 68,292 characters of text). From: https://doi.org/10.1017/S0022143000012429. Text file: Humphrey1994.txt.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (via OpenAlex). The title was not in the project's list; I took it from OpenAlex.
- Reference check: authors, volume, issue and pages match.

### Hallet1996
- Reference: Hallet, B., Hunter, L. and Bogen, J. (1996). Rates of erosion and sediment evacuation by glaciers: a review of field data and their implications. Global and Planetary Change 12(1-4): 213-235.
- DOI or URL: 10.1016/0921-8181(95)00021-6
- Cited as: "Hallet, Hunter & Bogen 1996, Global Planet. Change 12:213-235"
- Ledger ids: GLA-002, GLA-041, GLA-043
- Access: paywalled; abstract not available through OpenAlex
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): the project quotes its rates without page references; reading not recorded
- Read in this pass: not fetched (metadata only). A later paper (Cook et al. 2020) cites the title and was read for context.
- Reference check: authors, year, journal, volume and pages match.

### Sugden1976
- Reference: Sugden, D. E. and John, B. S. (1976). Glaciers and Landscape: A Geomorphological Approach. Edward Arnold, London.
- DOI or URL: none (book)
- Cited as: "Sugden & John 1976 (selective linear erosion)"
- Ledger ids: GLA-014, GLA-016
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: not checked (the full reference is mine from the citation form; no search done).

### Alley1998
- Reference: Alley, R. B., Lawson, D. E., Evenson, E. B., Strasser, J. C. and Larson, G. J. (1998). Glaciohydraulic supercooling: a freeze-on mechanism to create stratified, debris-rich basal ice: II. Theory. Journal of Glaciology 44(148): 563-569.
- DOI or URL: 10.1017/S0022143000002070 (OpenAlex)
- Cited as: "Alley et al. 1998, J. Glaciol. 44 (glaciohydraulic supercooling, theory 1.2 to 1.7)"
- Ledger ids: GLA-018
- Access: open access (Cambridge archive); abstract fetched
- Library (the project's source library, not distributed): fetched (HTML page, 45,765 characters of text). From: https://doi.org/10.1017/S0022143000002070. Text file: Alley1998.txt.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (via OpenAlex)
- Reference check: the citation is to Part II (Theory); Part I (field evidence) is Lawson et al. 1998, 44(148): 547-562. The abstract does not contain the 1.2-1.7 ratios.

### Alley2003
- Reference: Alley, R. B., Lawson, D. E., Larson, G. J., Evenson, E. B. and Baker, G. S. (2003). Stabilizing feedbacks in glacier-bed erosion. Nature 424(6950): 758-760.
- DOI or URL: 10.1038/nature01839
- Cited as: "Alley et al. 2003, Nature 424:758-760"
- Ledger ids: GLA-018
- Access: paywalled (abstract only, via Europe PMC)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, journal, volume and pages match.

### Svensson1959
- Reference: Svensson, H. (1959). Is the cross-section of a glacial valley a parabola? Journal of Glaciology 3(25): 362-363.
- DOI or URL: 10.3189/S0022143000017032 (OpenAlex)
- Cited as: "Svensson 1959, J. Glaciol. 3:362-363"
- Ledger ids: GLA-054, GLA-055
- Access: open access (Cambridge archive); abstract null in OpenAlex
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://doi.org/10.3189/S0022143000017032. File: Svensson1959.txt.
- Read so far (by the project): not stated
- Read in this pass: not fetched (metadata only)
- Reference check: authors, year, volume, issue and pages match. The title is from my own OpenAlex title query (cross-section, glacial valley, parabola, year 1959), which returned this item; the abstract was null.

### Sugden1978
- Reference: Sugden, D. E. (1978). Glacial erosion by the Laurentide ice sheet. Journal of Glaciology 20(83): 367-391.
- DOI or URL: 10.3189/S0022143000013915 (Crossref/OpenAlex)
- Cited as: "Sugden 1978, J. Glaciol. 20:367-391"
- Ledger ids: GLA-057
- Access: open access (Cambridge archive); abstract fetched
- Library (the project's source library, not distributed): fetched (PDF, 25 pages, text layer, 67,921 characters). From: https://www.cambridge.org/core/services/aop-cambridge-core/content/view/75DABAF1DF923B4925867321A176BE0A/S0022143000013915a.pdf/div-class-title-glacial-erosion-by-the-laurentide-ice-sheet-div.pdf. Text file: Sugden1978.txt.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (via OpenAlex)
- Reference check: matches.

### Egholm2009
- Reference: Egholm, D. L., Nielsen, S. B., Pedersen, V. K. and Lesemann, J.-E. (2009). Glacial effects limiting mountain height. Nature 460(7257): 884-887.
- DOI or URL: 10.1038/nature08263
- Cited as: "Egholm et al. 2009, Nature 460:884-887 (glacial buzzsaw)"
- Ledger ids: GLA-064
- Access: paywalled (abstract only, via Europe PMC)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, journal, volume and pages match.

### Mitchell2015
- Reference: Mitchell, S. G. and Humphries, E. E. (2015). Glacial cirques and the relationship between equilibrium line altitudes and mountain range height. Geology 43(1): 35-38.
- DOI or URL: 10.1130/G36180.1
- Cited as: "Mitchell & Humphries 2015, Geology 43:35-38"
- Ledger ids: GLA-062, GLA-064
- Access: paywalled (abstract only, reconstructed from OpenAlex; GeoScienceWorld returned 403)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, journal, volume and pages match. OpenAlex dates it 2014 (online first).

### Cook2020 (context, not cited by the code)
- Reference: Cook, S. J., Swift, D. A., Kirkbride, M. P., Knight, P. G. and Waller, R. I. (2020). The empirical basis for modelling glacial erosion rates. Nature Communications 11: 759.
- DOI or URL: 10.1038/s41467-020-14583-8 (https://pmc.ncbi.nlm.nih.gov/articles/PMC7005307/)
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 7 pages, text layer, 32,053 characters). From: https://www.nature.com/articles/s41467-020-14583-8.pdf. Text file: Cook2020_context_not_cited_by_the_code.txt.
- Read in this pass: full text fetched (several queries)
- Used for: independent statements of the erosion-rule constants.

### Prasicek2018 (context, not cited by the code)
- Reference: Prasicek, G., Herman, F., Robl, J. and Braun, J. (2018). Glacial steady state topography controlled by the coupled influence of tectonics and climate. Journal of Geophysical Research: Earth Surface 123.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC6055901/
- Access: open access
- Library (the project's source library, not distributed): fetched (HTML page, 71,355 characters of text). From: https://pmc.ncbi.nlm.nih.gov/articles/PMC6055901/. Text file: Prasicek2018_context_not_cited_by_the_code.txt.
- Read in this pass: full text fetched (Table 1 values as relayed)
- Used for: the rate factor A quoted from Cuffey and Paterson, and the pairings of K_g and l.

### Argudo2020 and Cordonnier2023 (research note only)
- Cordonnier, G., Jouvet, G., Peytavie, A., Braun, J., Cani, M.-P., Benes, B., Galin, E., Guérin, E. and Gain, J. (2023). Forming Terrains by Glacial Erosion. ACM Transactions on Graphics 42(4): 1-14. DOI 10.1145/3592422. Access: not checked (metadata only via OpenAlex). Authors in the metadata are as listed here.
- Argudo, O. et al. (2020). Simulation, modeling and authoring of glaciers. ACM Transactions on Graphics 39(6). Not verified in this pass (OpenAlex search returned nothing); the title and venue are from the project's note, unconfirmed.

#### Not bibliographic
Catmull-Rom interpolation (named only), the design-decisions log entries (Ben), `docs/research/notes/grid-artifact-audit.md` and `surface_processes_climate.md`, the "standard atmosphere" lapse rate, the Aletsch Glacier example (checked below), Neoproterozoic snowball diamictites and the Dwyka Group (naming only), `tools/review/*` and `fastscape_compare.py` (fastscapelib, GPL), `NOTICE.md`.

---

## Rock, strata and structure (RCK, STR, DEF)

### Winter2010
- Reference: Winter, J. D. (2010). Principles of Igneous and Metamorphic Petrology, 2nd ed. Prentice Hall / Pearson. (Open Library lists Pearson editions dated 2009-2014; the printed year of the 2nd edition is 2010 per the code and was not separately verified.)
- DOI or URL: no DOI. Author's lecture slides for the chapters (open): https://www.whitman.edu/geology/winter/JDW_PetClass.htm
- Cited as: "Winter 2010, Principles of Igneous and Metamorphic Petrology, 2nd ed., ch. 21 and 25"
- Ledger ids: RCK-039 to RCK-044, RCK-052
- Access: book (not online); author's chapter slides are open but the .ppt came back as binary and was not read
- Library (the project's source library, not distributed): not fetched. Reason: book; no open copy named.
- Read so far (by the project): not stated (chapter level only)
- Read in this pass: search result snippet only / chapter titles fetched from the Whitman page
- Reference check: the Whitman page for the second edition lists Chapter 21 "Introduction to Metamorphism" and Chapter 25 "Metamorphic Facies and Mafic Rocks" (the book title of ch. 25 is "Metamorphic Facies and Metamorphosed Mafic Rocks"), so the chapter numbers in the code are right. Chapter 21 covers metamorphic field gradients and ch. 25 the facies; neither chapter number tells which figure holds each boundary.

### Yardley1989
- Reference: Yardley, B. W. D. (1989). An Introduction to Metamorphic Petrology. Longman Earth Science Series. Longman Scientific & Technical, Harlow. ISBN 0-582-30096-7 (Open Library).
- DOI or URL: no DOI.
- Cited as: "Yardley 1989, An Introduction to Metamorphic Petrology, ch. 3"
- Ledger ids: RCK-039 to RCK-044, RCK-052
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: not fetched (bibliographic record only)
- Reference check: title, author, year, publisher exist. Chapter 3 content not checked (a 2020 second edition by Yardley & Warren also exists, so a "ch. 3" citation depends on the edition).

### Warren2016
- Reference: Warren, J. K. (2016). Evaporites: A Geological Compendium, 2nd ed. Springer, Cham. ISBN 978-3-319-13511-3 (Open Library lists the Springer 2nd-edition ISBN 9783319135113; subtitle and year from my recall of the edition, not confirmed by the fetch).
- DOI or URL: no DOI fetched.
- Cited as: "Warren 2016, Evaporites, 2nd ed., ch. 7"
- Ledger ids: RCK-045
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: not fetched (bibliographic record only)
- Reference check: author, title and edition exist; chapter 7 content not checked.

### Cruden1998
- Reference: Cruden, A. R. (1998). On the emplacement of tabular granites. Journal of the Geological Society 155(5):853-862.
- DOI or URL: 10.1144/gsjgs.155.5.0853
- Cited as: "Cruden 1998, J. Geol. Soc. 155:853-862" (tabular-pluton scaling T = 0.6 L^0.6)
- Ledger ids: RCK-057, RCK-061
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: authors, year, volume and pages are right. The abstract contains no thickness-length power law and no 0.6 coefficient or exponent; it gives widths of 10-100 km and thicknesses of 3 km or less (see spot check RCK-061). Whether the equation appears in the full text is not known. A thickness-length power law is the topic of McCaffrey & Petford (1997), "Are granitic intrusions scale invariant?", J. Geol. Soc. 154:1-4 (title and venue from my recall; an OpenAlex search for it returned nothing relevant); that is a lead for stage 2, not a verified fact.

### Bateman1979
- Reference: Bateman, P. C. & Chappell, B. W. (1979). Crystallization, fractionation, and solidification of the Tuolumne Intrusive Series, Yosemite National Park, California. Geological Society of America Bulletin 90(5):465-482.
- DOI or URL: 10.1130/0016-7606(1979)90<465:CFASOT>2.0.CO;2
- Cited as: "Bateman & Chappell 1979" (Tuolumne, about 1,100 km2, about 45 x 25 km)
- Ledger ids: RCK-060
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: authors, year, journal, volume and pages match. The abstract does not state an area or dimensions; the code's 1,100 km2 and 45 x 25 km are unconfirmed.

### Buddington1959
- Reference: Buddington, A. F. (1959). Granite emplacement with special reference to North America. Geological Society of America Bulletin 70(6):671-747 (OpenAlex shows only the first page, 671; the end page 747 is from my recall).
- DOI or URL: 10.1130/0016-7606(1959)70[671:GEWSRT]2.0.CO;2
- Cited as: "Buddington 1959" (epizonal to mesozonal emplacement)
- Ledger ids: RCK-063
- Access: paywalled (no abstract)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: the paper exists and is the classic granite-emplacement paper; code cites no page. Whether it gives a 2-6 km roof depth is not checked.

### Hudec2007
- Reference: Hudec, M. R. & Jackson, M. P. A. (2007). Terra infirma: Understanding salt tectonics. Earth-Science Reviews 82(1-2):1-28.
- DOI or URL: (RCK, STR, DEF) 10.1016/j.earscirev.2007.01.001 (ORE) 10.1016/j.earscirev.2007.01.001 (OpenAlex).
- Cited as: "Hudec & Jackson 2007, Earth-Sci. Rev. 82:1-28"
- Ledger ids: STR-026, STR-027
- Access: paywalled (no abstract in OpenAlex) [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: (RCK, STR, DEF) metadata only (ORE) not fetched (OpenAlex metadata only).
- Reference check: (RCK, STR, DEF) authors, year, volume, pages match. It is a review of salt tectonics; the specific 100 m / 1,000 m thresholds are not in anything fetched. (ORE) matches (authors, year, journal, volume, pages). Title is "Terra infirma: understanding salt tectonics".

### Doust1990
- Reference: Doust, H. & Omatsola, E. (1990). Niger Delta. In: Edwards, J. D. & Santogrossi, P. A. (eds), Divergent/Passive Margin Basins, AAPG Memoir 48, pp. 201-238 (editors and pages from my recall; not confirmed).
- DOI or URL: (RCK, STR, DEF) none found (an OpenAlex filter for 1990 "Niger Delta" returned only a single-author Doust paper, Geol. Soc. London Spec. Publ. 50:365, DOI 10.1144/GSL.SP.1990.050.01.21, which is a different work). (ORE) 10.1306/M48508C4 (Crossref; Crossref dates the chapter 1989 online, the memoir is conventionally 1990).
- Cited as: "Doust & Omatsola 1990, AAPG Memoir 48"
- Ledger ids: STR-028, STR-029
- Access: book (not online). [other sections: not found online]
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: (RCK, STR, DEF) not fetched (ORE) not fetched (Crossref metadata only).
- Reference check: (RCK, STR, DEF) unverified. Doust & Omatsola 1990 as a book chapter was not located in the databases reached; the 1,500 m sediment threshold is unsourced. (ORE) matches (authors, memoir number). Chapter title "Niger Delta".

### Mouthereau2007
- Reference: Mouthereau, F., Tensi, J., Bellahsen, N., Lacombe, O., De Boisgrollier, T. & Kargar, S. (2007). Tertiary sequence of deformation in a thin-skinned/thick-skinned collision belt: The Zagros Folded Belt (Fars, Iran). Tectonics 26(5):TC5006.
- DOI or URL: 10.1029/2007TC002098
- Cited as: "Mouthereau, Tensi, Bellahsen, Lacombe, De Boisgrollier & Kargar 2007, Tectonics 26:TC5006"
- Ledger ids: DEF-001, DEF-005
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 1010 chars).
- Substitute: none found.
- Read so far (by the project): not stated (the quoted phrase is in quotation marks in the comment)
- Read in this pass: abstract fetched (OpenAlex, full abstract)
- Reference check: author list, year, journal, volume and issue match; "TC5006" article number is not shown in the OpenAlex record (not verified). The abstract contains both numbers the code attributes to it (see spot checks DEF-001, DEF-005).
- Read in WP-8 (2026-10-02): full text, fetched from the HAL open copy (hal-00634887) through the web archive; library Mouthereau2007.pdf, Mouthereau2007_raw.txt (article page = PDF page - 1). Fig. 5 caption: 149 anticlines, "Zagros folds have a dominant wavelength of 15.8 ± 5.3 km"; §[55]: 11 km near the Surmeh fault to 30 km in the High Zagros; art. p. 5: maximum fold amplitudes 500-1,000 m; §[7]: "a 200-kmwide arcuate folded belt", sedimentary pile up to 12 km, axial lengths often over 100 km; abstract and §[18]: 65-78 km (16-19 %) across the whole Zagros, only about 15 km of it across the Folded Belt (§[70]); §[53]-[54]: buckling, 2πH(ηl/6ηm)^(1/3) with a 2 km layer (DEF-001, DEF-002, DEF-004, DEF-005, DEF-007, DEF-027).

### Biot1961
- Reference: Biot, M. A. (1961). Theory of folding of stratified viscoelastic media and its implications in tectonics and orogenesis. Geological Society of America Bulletin 72(11):1595-1620.
- DOI or URL: 10.1130/0016-7606(1961)72[1595:TOFOSV]2.0.CO;2
- Cited as: "Biot 1961, Geol. Soc. Am. Bull. 72:1595-1620"
- Ledger ids: DEF-002
- Access: paywalled (no abstract)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: authors, year, volume, issue, pages match.
- WP-8 (2026-10-02): not obtained. The dominant-wavelength relation is now cited to Schmalholz & Mancktelow 2016 (eq. 13), which credits Biot 1957 and Ramberg 1962 (DEF-002).

### Sherwin1968
- Reference: Sherwin, J.-A. & Chapple, W. M. (1968). Wavelengths of single layer folds: a comparison between theory and observation. American Journal of Science 266(3):167-179.
- DOI or URL: 10.2475/ajs.266.3.167
- Cited as: "Sherwin & Chapple 1968, Am. J. Sci. 266:167-179"
- Ledger ids: DEF-002
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: authors, year, volume, pages match. The paper compares Biot's theory with 818 fold wavelengths from six sites (quartz veins in slate and phyllite), so "tested on natural folds" is correct; it found the viscosity ratios needed were low and Biot's theory could not be applied directly.
- WP-8 (2026-10-02): not obtained; no longer cited by the code as read (DEF-002).

### Blanc2003
- Reference: Blanc, E. J.-P., Allen, M. B., Inger, S. & Hassani, H. (2003). Structural styles in the Zagros Simple Folded Zone, Iran. Journal of the Geological Society 160(3):401-412.
- DOI or URL: 10.1144/0016-764902-110
- Cited as: "Blanc, Allen, Inger & Hassani 2003, J. Geol. Soc. 160:401-412"
- Ledger ids: DEF-004
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: authors, year, volume, issue, pages match. The abstract is about a balanced cross-section with about 49 km of shortening over 5 Myr and several decollement horizons; it gives no fold length-to-wavelength ratio.
- WP-8 (2026-10-02): not obtained; the code now cites Alavi 2007 (read in full) for the fold length ratio (DEF-004).

### McQuarrie2004
- Reference: McQuarrie, N. (2004). Crustal scale geometry of the Zagros fold-thrust belt, Iran. Journal of Structural Geology 26(3):519-535.
- DOI or URL: 10.1016/j.jsg.2003.08.009. OpenAlex reports a submitted version at the Caltech Authors repository (not fetched).
- Cited as: "McQuarrie 2004, J. Struct. Geol. 26:519-535" (Zagros shortened about 20 %)
- Ledger ids: DEF-005
- Access: open copy (author's submitted version at Caltech Authors, per OpenAlex; URL not obtained)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 11 chars).
- Read so far (by the project): not stated
- Read in this pass: metadata only (OpenAlex dates it 15 Oct 2003 online; volume 26 is 2004)
- Reference check: title, journal, volume, pages match. The title the code gives is only "McQuarrie 2004" (no title). The "about 20 %" figure is not confirmed.
- WP-8 (2026-10-02): the Caltech Authors record (1k507-wsf19) holds metadata only, files restricted; abstract read (McQuarrie2004_abstract_only.txt): "Total shortening in the Zagros fold–thrust belt is 70±20 km, which corresponds to ~20% shortening of the Arabian block" (DEF-005, abstract only).

### Gawthorpe2000
- Reference: Gawthorpe, R. L. & Leeder, M. R. (2000). Tectono-sedimentary evolution of active extensional basins. Basin Research 12(3-4):195-218.
- DOI or URL: 10.1046/j.1365-2117.2000.00121.x
- Cited as: "Gawthorpe & Leeder 2000, Basin Res. 12:195-218" (faults grow by linkage of segments)
- Ledger ids: DEF-013
- Access: open access? Not determined (OpenAlex shows it as a cited journal article; the licence was not read). Treat as paywalled (metadata only).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: authors, year, volume, issue, pages match. "Segments of some tens of km" is unconfirmed.
- WP-8 (2026-10-02): not obtained (the publisher refuses scripted access; the UEA record 32628 has no full text; no archived copy). DEF-013 is labelled a design value and the comment says the source was not read.

### Cowie1992a
- Reference: Cowie, P. A. & Scholz, C. H. (1992). Displacement-length scaling relationship for faults: data synthesis and discussion. Journal of Structural Geology 14(10):1149-1156.
- DOI or URL: 10.1016/0191-8141(92)90066-6
- Cited as: "Cowie & Scholz 1992, J. Struct. Geol. 14:1149-1156" (D = gamma L)
- Ledger ids: DEF-015 to DEF-018
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 11 chars).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: the pages 1149-1156 are the "data synthesis and discussion" paper, and it is the right one for a data-compilation citation. A companion paper by the same authors (Cowie & Scholz 1992, "Physical explanation for the displacement-length relationship of faults using a post-yield fracture mechanics model", JSG 14(10):1133-1148, DOI 10.1016/0191-8141(92)90065-5) is the theoretical one; the code does not cite it. The abstract of the cited paper gives no value of gamma.
- WP-8 (2026-10-02): not obtained; no longer cited by the code (DEF-015 cites Schlische et al. 1996 through Lathrop et al. 2022).

### Schlische1996
- Reference: Schlische, R. W., Young, S. S., Ackermann, R. V. & Gupta, A. (1996). Geometry and scaling relations of a population of very small rift-related normal faults. Geology 24(8):683-686.
- DOI or URL: 10.1130/0091-7613(1996)024<0683:GASROA>2.3.CO;2
- Cited as: "Schlische, Young, Ackermann & Gupta 1996, Geology 24:683-686" (gamma about 0.03 for normal faults)
- Ledger ids: DEF-015
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary); the 0.03 comes from a secondary review (Lathrop2022)
- Reference check: authors, year, volume, pages match. The abstract says displacement scales linearly with length across eight orders of magnitude and gives no number; the review below quotes the paper's best fit as D = 0.03 L^1.06.
- WP-8 (2026-10-02): abstract read from an archived journal page (Schlische1996_abstract_page.txt): "the maximum observed displacement scales linearly with fault length ... no significant change in displacement geometry and the linear length-displacement scaling relation between small and large faults". The 0.03 is read in Lathrop et al. 2022 (DEF-015).

### Lathrop2022
- Reference: Lathrop, B. A., Jackson, C. A.-L., Bell, R. E. & Rotevatn, A. (2022). Displacement/length scaling relationships for normal faults; a review, critique, and revised compilation. Frontiers in Earth Science 10:907543.
- DOI or URL: 10.3389/feart.2022.907543; open access: https://www.frontiersin.org/journals/earth-science/articles/10.3389/feart.2022.907543/full
- Cited as: not cited by the code (added by this pass as the secondary source used for DEF-015)
- Ledger ids: DEF-015 to DEF-018; notes: evidence only
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 23 pages, text layer, 106,853 characters). From: https://eprints.gla.ac.uk/308438/1/308438.pdf. Text file: Lathrop2022.txt.
- Read so far (by the project): not applicable
- Read in this pass: full text fetched (via WebFetch summary of the full page; passages on Schlische 1996, Cowie & Scholz 1992a and the typical gamma range)
- Reference check: not applicable.
- Read in WP-8 (2026-10-02): full text (new reading-order extraction Lathrop2022_raw.txt). p. 2: c "typically fall[s] between 0.001 and 0.1"; p. 4: strike-slip D/L "as high as 1:1", normal faults at most 1:2 (citing Kim & Sanderson 2005); p. 7: Schlische et al. 1996 "D = 0.03L1.06" from 201 Dan River faults and 346 others; p. 6 and table 1: the review's own fit Dmax = 0.03 L^0.92. No separate value for reverse or strike-slip faults (DEF-015 to DEF-018).

### Kim2005
- Reference: Kim, Y.-S. & Sanderson, D. J. (2005). The relationship between displacement and length of faults: a review. Earth-Science Reviews 68(3-4):317-334.
- DOI or URL: 10.1016/j.earscirev.2004.06.003
- Cited as: "Kim & Sanderson 2005, Earth-Sci. Rev. 68:317-334" (review, order-of-magnitude spread)
- Ledger ids: DEF-015
- Access: paywalled (no abstract in OpenAlex; publisher pages returned 403)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: metadata only
- Reference check: authors, volume, issue, pages match. OpenAlex dates it 2004 (online); the issue is January 2005 per my recall, so "2005" is acceptable but may be cited as 2004 elsewhere. The "order-of-magnitude spread" is not confirmed from this paper.
- WP-8 (2026-10-02): not obtained (Southampton eprints 52991 has no PDF; no archived copy). Its findings are cited only as Lathrop et al. 2022 report them.

### Anderson1951
- Reference: Anderson, E. M. (1951). The Dynamics of Faulting and Dyke Formation with Applications to Britain, 2nd ed., revised. Oliver and Boyd, Edinburgh. 206 pp.
- DOI or URL: no DOI. Open Library: https://openlibrary.org/works/OL7161171W
- Cited as: "Anderson 1951, The Dynamics of Faulting, 2nd ed."
- Ledger ids: DEF-019, DEF-020
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book; no open copy named.
- Read so far (by the project): not stated
- Read in this pass: not fetched (secondary sources only)
- Reference check: the code's title is shortened; the full title includes "and Dyke Formation with Applications to Britain". The edition (2nd, 1951) matches. The first edition is 1942 (Wikipedia gives 1905 for the theory's first publication).
- WP-8 (2026-10-02): the book was not obtained; the code now cites Anderson 1905 (read in full), the paper that first set out the theory (DEF-019 to DEF-021).

### Weissel1989
- Reference: Weissel, J. K. & Karner, G. D. (1989). Flexural uplift of rift flanks due to mechanical unloading of the lithosphere during extension. Journal of Geophysical Research 94(B10):13919-13950.
- DOI or URL: 10.1029/JB094iB10p13919
- Cited as: "Weissel & Karner 1989, J. Geophys. Res. 94:13919-13950"
- Ledger ids: DEF-059
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: authors, year, volume, pages match.
- WP-8 (2026-10-02): not obtained; DEF-059 stays `abstract-only` on stage 1's abstract.


### Schmalholz2016
- Reference: Schmalholz, S. M. & Mancktelow, N. S. (2016). Folding and necking across the scales: a review of theoretical and experimental results and their applications. Solid Earth 7:1417-1465.
- DOI or URL: 10.5194/se-7-1417-2016 (open access)
- Cited as: "Schmalholz & Mancktelow 2016, Solid Earth 7:1417-1465, eq. 13" (added to the code in WP-8)
- Ledger ids: DEF-002
- Access: open access
- Library: fetched WP-8 2026-10-02 (PDF, 49 pages, text layer). Text files: Schmalholz2016.txt, Schmalholz2016_raw.txt (printed page = PDF page + 1416).
- Read in WP-8 (2026-10-02): full text for the passages used. Eq. (13), p. 1422: "Ld = 2π H (R/6)^(1/3) ... the dominant wavelength is directly proportional to the layer thickness ... first derived by Biot (1957)", Ramberg 1962 identical (p. 1423); p. 1425: lower limit of wavelength to thickness about 4; p. 1442: "common wavelength to thickness ratios are between 2 and 16 with a mean value at ∼ 6.5 (e.g. Hudleston and Treagus, 2010)"; p. 1434: a multilayer of m equal layers raises the wavelength by m^(1/3) (Biot 1961); eq. (32), p. 1436: Ld = 1.9 √(H Htot); pp. 1430-1433: linear theory good to about 20 % shortening and 25° limbs.

### Elliott1976
- Reference: Elliott, D. (1976). The energy balance and deformation mechanisms of thrust sheets. Philosophical Transactions of the Royal Society of London A 283:289-312.
- DOI or URL: 10.1098/rsta.1976.0086
- Cited as: "Elliott 1976, Phil. Trans. R. Soc. A 283:289-312, p. 297" (added to the code in WP-8)
- Ledger ids: DEF-013, DEF-015, DEF-017
- Access: free to read at the publisher (fetched through the web archive; the live site refuses scripted access)
- Library: fetched WP-8 2026-10-02 (PDF, 26 pages, text layer). Text files: Elliott1976.txt, Elliott1976_raw.txt (printed page = PDF page + 288).
- Read in WP-8 (2026-10-02): p. 297 and fig. 3: for 20 thrusts of the Canadian Rockies foothills and front ranges "a roughly linear relation between strike length l on the map and displacement u = Kl (9) where K ~ 7 %" (the "=" read through OCR); fig. 4, p. 298: the McConnell thrust "has a strike length of 410 km".

### Anderson1905
- Reference: Anderson, E. M. (1905). The dynamics of faulting. Transactions of the Edinburgh Geological Society 8:387-402.
- DOI or URL: open scan at Zenodo record 1650397 (CC0), https://zenodo.org/records/1650397
- Cited as: "Anderson 1905, Trans. Edinb. Geol. Soc. 8:387-402, pp. 389 and 402" (added to the code in WP-8)
- Ledger ids: DEF-019, DEF-020, DEF-021, DEF-024
- Access: open (public domain by age; CC0 scan)
- Library: fetched WP-8 2026-10-02 (PDF scan with OCR, 16 pages). Text files: Anderson1905.txt, Anderson1905_raw.txt (printed page = PDF page + 386; letter-spaced OCR).
- Read in WP-8 (2026-10-02): p. 389: tan 2θ = 1/μ, "θ = 45° for μ = 0; θ = 30° for μ = 1/√3 or ·577; θ = 22½° for μ = 1", θ the angle between the fault plane and the greatest pressure; p. 402 (summary): thrusts form when the greatest pressure is horizontal and the least vertical and "dip ... at angles of less than 45°"; normal faults when the greatest pressure is vertical, "at angles of more than 45°"; wrench planes when the greatest and least are horizontal, "their hade is theoretically vertical". Anderson does not state 60° and 30° as such.

### WaldronSnyder2020
- Reference: Waldron, J. W. F. & Snyder, M. (2020). Geological Structures: a Practical Introduction. University of Alberta, open textbook (CC BY-NC).
- DOI or URL: https://pressbooks.openeducationalberta.ca/introductorystructuralgeology/
- Cited as: in the ledger and the page only (WP-8)
- Ledger ids: DEF-019, DEF-021, DEF-024, DEF-031
- Access: open access
- Library: fetched WP-8 2026-10-02 (PDF, 225 pages, text layer). Text files: WaldronSnyder2020.txt, WaldronSnyder2020_raw.txt (printed page = PDF page - 12).
- Read in WP-8 (2026-10-02): p. 151: Anderson's gravity, thrust and wrench regimes; p. 152: normal faults "dipping towards and away from each other with dips around 60°, at least near the surface"; p. 158: "Most major strike-slip faults are found to dip steeply, confirming Anderson's prediction".

### CollettiniSibson2001
- Reference: Collettini, C. & Sibson, R. H. (2001). Normal faults, normal friction? Geology 29(10):927-930.
- DOI or URL: 10.1130/0091-7613(2001)029<0927:NFNF>2.0.CO;2
- Cited as: "Collettini & Sibson 2001, Geology 29:927-930, abstract" (added to the code in WP-8)
- Ledger ids: DEF-019, DEF-020
- Access: paywalled (abstract only)
- Library: archived abstract page, CollettiniSibson2001_abstract_page.txt (WP-8 2026-10-02).
- Read in WP-8 (2026-10-02): abstract only: normal-slip earthquake ruptures (n = 25) dip from 65° to 30°; reverse ruptures (n = 33) show "a dominant 30° ± 5° peak"; both consistent with a friction coefficient of about 0.6.

### Herail1990
- Reference: Hérail, G., Baby, P., López, M., Oller, J., López, O., Salinas, R., Sempéré, T., Beccar, G. & Toledo, H. Structure and kinematic evolution of Subandean thrust system of Bolivia. ORSTOM symposium paper (IRD Horizon document 31021). The year is the library key, not read on the scan.
- DOI or URL: https://horizon.documentation.ird.fr/exl-doc/pleins_textes/pleins_textes_4/colloques/31021.pdf
- Cited as: "Hérail et al., ORSTOM symposium paper, pp. 179-180, a poor scan" (added to the code in WP-8)
- Ledger ids: DEF-027
- Access: open (institutional repository)
- Library: fetched WP-8 2026-10-02 (PDF scan with poor OCR). Text file: Herail1990.txt.
- Read in WP-8 (2026-10-02): pp. 179-180: the northern Subandean zone is "60 to GO. km-wide" (second figure unreadable), shortening 136 km (51 %); the southern zone "100 to 140 km-wide", shortening 70 km (33 %) to 139 km (53 %).

### DelaneyPollard1981
- Reference: Delaney, P. T. & Pollard, D. D. (1981). Deformation of host rocks and flow of magma during growth of minette dikes and breccia-bearing intrusions near Ship Rock, New Mexico. U.S. Geological Survey Professional Paper 1202.
- DOI or URL: https://pubs.usgs.gov/pp/1202/report.pdf
- Cited as: "Delaney & Pollard 1981, USGS PP 1202" (added to the code in WP-8)
- Ledger ids: DEF-047, DEF-048, DEF-049
- Access: open (government report)
- Library: fetched WP-8 2026-10-02 (PDF, 69 pages, text layer). Text files: DelaneyPollard1981.txt, DelaneyPollard1981_raw.txt.
- Read in WP-8 (2026-10-02): abstract (PDF p. 6): seven dikes radiate from Ship Rock; the northeastern dike is about 2,900 m long, "an average thickness of 2.3 m, and a maximum thickness of 7.2 m", in 35 segments; three plugs "smaller than 30 m in diameter"; PDF p. 12: "The largest of these dikes are 9, 4, and 3 km long"; PDF p. 15: segments 0.6 to 4.6 m in average thickness.

### Krumbholz2014
- Reference: Krumbholz, M., Hieronymus, C. F., Burchardt, S., Troll, V. R., Tanner, D. C. & Friese, N. (2014). Weibull-distributed dyke thickness reflects probabilistic character of host-rock strength. Nature Communications 5:3272.
- DOI or URL: 10.1038/ncomms4272 (open access)
- Cited as: in the ledger and the page only (WP-8)
- Ledger ids: DEF-047, DEF-048
- Access: open access
- Library: fetched WP-8 2026-10-02 (PDF, 7 pages, text layer). Text files: Krumbholz2014.txt, Krumbholz2014_raw.txt.
- Read in WP-8 (2026-10-02): p. 2: dykes are penny-shaped "with a thickness-to-length ratio of 0.01–0.0001", their "thickness in nature ranges from centimetres to hundreds of metres"; fig. 1: 487 to 1,918 thickness measurements per locality in Iceland and the Canary Islands. The summary statistics are in the supplement, not read.

### Bunger2013
- Reference: Bunger, A. P., Menand, T., Cruden, A. R., Zhang, X. & Halls, H. (2013). Analytical predictions for a natural spacing within dyke swarms. Earth and Planetary Science Letters 375:270-279.
- DOI or URL: 10.1016/j.epsl.2013.05.044; author manuscript HAL hal-00855272
- Cited as: "Bunger, Menand, Cruden, Zhang & Halls 2013, Earth Planet. Sci. Lett. 375:270-279" (added to the code in WP-8)
- Ledger ids: DEF-046, DEF-047, DEF-048
- Access: open copy (author manuscript, fetched through the web archive)
- Library: fetched WP-8 2026-10-02 (manuscript PDF, 50 pages, text layer). Text files: Bunger2013.txt, Bunger2013_raw.txt.
- Read in WP-8 (2026-10-02): fig. 1 caption: the Mackenzie dykes "extend over more than 2,000 km with an average thickness of 30 m"; manuscript lines 523-542: thickness 1 to 150 m, mean 30 m; mean spacing about 6.7 km 500 km from the centre to about 25 km 2,100 km out, the most continuous distal dykes 35 to 65 km apart; lines 555-573: Matachewan dykes 10 to 20 m on average.

### NMT_Shiprock
- Reference: New Mexico Bureau of Geology and Mineral Resources. Ship Rock (Virtual Geologic Tour of New Mexico), web page.
- DOI or URL: https://geoinfo.nmt.edu/tour/landmarks/shiprock/home.html
- Cited as: "Ship Rock 500 m, New Mexico Bureau of Geology" (added to the code in WP-8)
- Ledger ids: DEF-049
- Access: open (agency page)
- Library: saved page, NMT_Shiprock (WP-8 2026-10-02).
- Read in WP-8 (2026-10-02): "The main part of the landform is 600 meters high, and 500 meters in diameter ... Ship Rock ... is a volcanic neck, or the central feeder pipe".

### Latutrie2020
- Reference: Latutrie, B. (2020). Volcanology of two maar-diatremes: Round Butte and Twin Peaks, Hopi Buttes volcanic field, Navajo Nation, Arizona. PhD thesis, Institut national de la recherche scientifique (INRS), Centre Eau Terre Environnement, Québec.
- DOI or URL: https://espace.inrs.ca/id/eprint/10410/1/T956.pdf
- Cited as: in the ledger only (WP-8)
- Ledger ids: DEF-049
- Access: open (institutional repository)
- Library: fetched WP-8 2026-10-02 (PDF, 231 pages, text layer). Text files: Latutrie2020.txt, Latutrie2020_raw.txt.
- Read in WP-8 (2026-10-02): PDF p. 49: Round Butte, "a diatreme 170-190 m in diameter".

### Earle2019
- Reference: Earle, S. (2019). Physical Geology, 2nd edition. BCcampus Open Education, Victoria, B.C. (CC BY).
- DOI or URL: https://opentextbc.ca/physicalgeology2ed/ (chapter pages read through the web archive)
- Cited as: "Earle 2019, Physical Geology 2nd ed., §3.5" (added to the code in WP-8)
- Ledger ids: DEF-042; the page's craton comparison
- Access: open access
- Library: saved chapter pages §3.5, §21.1, §21.2 (Earle2019*.txt), WP-8 2026-10-02.
- Read in WP-8 (2026-10-02): §3.5: "if the body has an exposed surface area greater than 100 square kilometres (km2), then it's a batholith; smaller than 100 km2 and it's a stock. Batholiths are typically formed only when a number of stocks coalesce beneath the surface"; §21.1-21.2: the Canadian Shield is where the ancient rocks of the craton are exposed, "typically strongly metamorphosed", elsewhere "covered with younger rocks".

### WikipediaCraton
- Reference: Wikipedia, "Craton" (secondary source).
- DOI or URL: https://en.wikipedia.org/wiki/Craton
- Cited as: in the page only (WP-8)
- Ledger ids: none (the page's craton comparison)
- Access: open (encyclopedia)
- Library: saved page, WikipediaCraton (WP-8 2026-10-02).
- Read in WP-8 (2026-10-02): cratons have "the cratonic basement of metamorphosed crystalline and metamorphic rocks and the platform, which is a younger, weakly deformed sedimentary cover".
### Jessell1981
- Reference: Jessell, M. W. (1981). "Noddy": an interactive map creation package. Unpublished MSc thesis, Imperial College of Science and Technology, University of London (Royal School of Mines, Department of Geology), September 1981.
- DOI or URL: https://tectonique.net/noddy/jessell_msc_1981.pdf (open copy; listed in a search result, not fetched or read)
- Cited as: not cited by the code as a paper ("Imperial College 1981-82" in NOTICE); the Noddy author's own page names this as the foundational citation.
- Ledger ids: DEF-063; notes: origin of the ported code
- Access: open copy at https://tectonique.net/noddy/jessell_msc_1981.pdf
- Library (the project's source library, not distributed): fetched (PDF, 52 pages, text layer, 52,755 characters). From: https://tectonique.net/noddy/jessell_msc_1981.pdf. Text file: Jessell1981.txt.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only
- Reference check: tectonique.net states "For publications using Noddy, the foundational reference is: Jessell, M., 1981. NODDY - An interactive map creation package".
- Read in WP-8 (2026-10-02): full text (Jessell1981.txt). §3 preamble: the pitch convention ("zero results in a plunge direction of the dip direction of the plane minus 90 degrees, with positive amplitudes resulting in displacements in the direction of the pitch"); §3.2: the fold W' = W + A Fr(U) Fe(V), the profile normalised to 2π, Fe equal to 0.5 at V = n (so 0.693147 is ln 2), the amplitude "the largest positive digitised amplitude"; §3.6: the translation fault X' = X + αS (DEF-010, DEF-038, DEF-066).

### Jessell1996
- Reference: Jessell, M. W. & Valenta, R. K. (1996). Structural geophysics: integrated structural and geophysical modelling. In: De Paor, D. G. (ed.), Structural Geology and Personal Computers, Computer Methods in the Geosciences 15. Pergamon, Oxford, pp. 303-324.
- DOI or URL: no DOI found. Records: https://research-repository.uwa.edu.au/en/publications/structural-geophysics-integrated-structural-and-geophysical-model/ ; https://www.academia.edu/20046786/Structural_Geophysics (copy listed in a search, not fetched)
- Cited as: "Jessell & Valenta 1996, Structural geophysics ..., in De Paor (ed.), Structural Geology and Personal Computers, Pergamon, 303-324"
- Ledger ids: DEF-010, DEF-038, DEF-063
- Access: open copy (academia.edu / ResearchGate copy listed in search results; not fetched)
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://research-repository.uwa.edu.au/en/publications/structural-geophysics-integrated-structural-and-geophysical-model/. File: Jessell1996.txt.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only
- Reference check: a search snippet gives "Computer Methods in the Geosciences, volume 15, pages 303-324, 1996", consistent with the code. Whether the fold sine table (629 values) or fault geometry is described in this chapter is not checked.
- WP-8 (2026-10-02): not obtained (no lawful full text found); the conventions it would document are read in Jessell 1981 and the source code.

### Jessell2022
- Reference: Jessell, M., Guo, J., Li, Y., Lindsay, M., Scalzo, R., Giraud, J., Pirot, G., Cripps, E. & Ogarko, V. (2022). Into the Noddyverse: a massive data store of 3D geological models for machine learning and inversion applications. Earth System Science Data 14(1):381-392.
- DOI or URL: 10.5194/essd-14-381-2022; open access: https://essd.copernicus.org/articles/14/381/2022/
- Cited as: "Jessell et al. 2022, Earth Syst. Sci. Data 14:381-392"
- Ledger ids: DEF-063, DEF-073
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 12 pages, text layer, 49,933 characters). From: https://essd.copernicus.org/articles/14/381/2022/essd-14-381-2022.pdf. Text file: Jessell2022.txt.
- Read so far (by the project): not stated
- Read in this pass: full text fetched (via WebFetch of the article page; passages on timing and code licence)
- Reference check: year, journal, volume, pages match. Authors and title now supplied in full. The code's "Jessell et al. 2022" is correct.
- Read in WP-8 (2026-10-02): full text (Jessell2022.txt). p. 383: Noddy unravels the kinematics point by point, "taking under 15 s to generate 200 × 200 × 200 voxel models with both geological and geophysical representations combined using an Intel® Xeon® Gold 6254 CPU"; p. 390: "All codes and data are released under the MIT License" and the 4 km model dimensions (DEF-063, DEF-070, DEF-073).

### Wellmann2016
- Reference: Wellmann, J. F., Thiele, S. T., Lindsay, M. D. & Jessell, M. W. (2016). pynoddy 1.0: an experimental platform for automated 3-D kinematic and potential field modelling. Geoscientific Model Development 9(3):1019-1035.
- DOI or URL: 10.5194/gmd-9-1019-2016; open access: https://gmd.copernicus.org/articles/9/1019/2016/
- Cited as: "Wellmann et al. 2016, Geosci. Model Dev. 9:1019-1035"
- Ledger ids: DEF-063, DEF-073
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 17 pages, text layer, 62,327 characters). From: https://gmd.copernicus.org/articles/9/1019/2016/gmd-9-1019-2016.pdf. Text file: Wellmann2016.txt.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched
- Reference check: authors, year, volume, pages match.

### Noddyverse (software)
- Reference: Loop3D / Jessell, M. et al. noddyverse: C code to generate random Noddy models (the modified Noddy source). GitHub repository, commit 6a21b2374516f8506c19cabcbe3ead00be9cc45d (the commit hash was not looked up; only the repository head was fetched). Archived release with DOI 10.5281/zenodo.4589883 (DOI taken from the ESSD article).
- DOI or URL: https://github.com/Loop3D/noddyverse ; https://doi.org/10.5281/zenodo.4589883
- Cited as: "Loop3D/noddyverse ... MIT License, Copyright (c) 2021 Loop3D"
- Ledger ids: DEF-010, DEF-038, DEF-063 to DEF-073
- Access: open access
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://github.com/Loop3D/noddyverse. File: Noddyverse_software.txt.
- Read so far (by the project): licence "checked in its LICENSE" (STATUS.md)
- Read in this pass: full text fetched (LICENSE file, repo front page, src/ listing, unEvents.c first 40 lines)
- Reference check: licence matches what the code says, see spot check DEF-063. The LICENSE title line is "MIT License", copyright line "Copyright (c) 2021 Loop3D". The fetched unEvents.c header has no copyright or licence comment. The repository carries a src/Licence.h that is a registration-ID stub (not a licence text). Other published releases of the same program carry different licences, see Noddy licence history under spot check DEF-063.
- Read in WP-8 (2026-10-02): the source tree at commit 6a21b2374516f8506c19cabcbe3ead00be9cc45d, downloaded as that commit's tarball onto the test machine (scratch, deleted with the package; not in the library). Read: unEvents.c (TOLERANCE 0.001, the constants 0.693147, 6.2831853 and 3.1415927, unbiot's commented-out call), utils.c (TOLERANCE 0.01, rotset and its clamp, 3.1415926535), getEvent.c (dip 90 stored as 89.999), nodStruc.h (X_PROFILE_RESOLUTION 628, Y_PROFILE_RESOLUTION 100, ARRAY_LENGTH_OF_STRAT_CODE 64), proflib.c (calcProfile, getArrayForProfile), nodLib3.c (initGraphPoints: the default sine profile of 21 points), matrix.c (rotateX), taste.c and which.c (the functions DEF-064 names) (DEF-010, DEF-063 to DEF-069).

### NoddyOriginal (software, other releases)
- Reference: Noddy, Jessell, M. W. (Imperial College 1981-82; Monash University 1994; University of Western Australia). Distributions: tectonique.net Windows release and GitHub; SourceForge project "Noddy"; pynoddy (Wellmann et al.).
- DOI or URL: https://tectonique.net/noddy/ ; https://sourceforge.net/projects/noddy/ ; https://github.com/flohorovicic/pynoddy ; https://github.com/Loop3D/noddy_qt
- Cited as: not cited separately; relevant to the licence the port inherits.
- Ledger ids: DEF-063
- Access: open access
- Library (the project's source library, not distributed): fetched (HTML page, 12,411 characters of text). From: https://github.com/Loop3D/noddy_qt. Text file: NoddyOriginal_software_other_releases.txt.
- Read so far (by the project): MIT licence checked for noddyverse only
- Read in this pass: full text fetched (the pages named)
- Reference check: see spot check DEF-063.

### tectonics.js (software)
- Reference: davidson16807. tectonics.js: "a 3d planet simulator in your web browser". GitHub.
- DOI or URL: https://github.com/davidson16807/tectonics.js
- Cited as: "tectonics.js lithifies sediment under 2.2 MPa of overburden, about 150 m (`plates::pools`)"
- Ledger ids: RCK-016, RCK-022
- Access: open access (licence shown as CC-BY-4.0)
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://github.com/davidson16807/tectonics.js. File: tectonics.js_software.txt.
- Read so far (by the project): not stated
- Read in this pass: full text fetched (repository front page only)
- Reference check: the repository exists with that name and owner; the front page says nothing about lithification or overburden, so the 2.2 MPa figure was not located (it would be in the source code, not read here).

### Haederle2002
- Reference: Haederle, M. & Atherton, M. P. (2002). Shape and intrusion style of the Coastal Batholith, Peru. Tectonophysics 345(1-4):17-28.
- DOI or URL: 10.1016/S0040-1951(01)00204-9
- Cited as: not cited by the code (the code gives no reference for the Coastal Batholith); added by this pass as the reference the Wikipedia article on the batholith uses.
- Ledger ids: RCK-056; notes: evidence only
- Access: paywalled (no abstract in OpenAlex)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not applicable
- Read in this pass: metadata only
- Reference check: not applicable.

### Mukasa1986
- Reference: Mukasa, S. B. (1986). Zircon U-Pb ages of super-units in the Coastal batholith, Peru: implications for magmatic and tectonic processes. Geological Society of America Bulletin 97(2):241-254.
- DOI or URL: 10.1130/0016-7606(1986)97<241:ZUAOSI>2.0.CO;2
- Cited as: not cited by the code; added as evidence for RCK-056.
- Ledger ids: RCK-056; notes: evidence only
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not applicable
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: not applicable.

### Alavi2007
- Reference: Alavi, M. (2007). Structures of the Zagros fold-thrust belt in Iran. American Journal of Science 307(9):1064-1095.
- DOI or URL: 10.2475/09.2007.02
- Cited as: not cited by the code; added as evidence for the belt width (STR-030, DEF-027).
- Ledger ids: DEF-027, STR-030; notes: evidence only
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): fetched (PDF, 39 pages, text layer, 97,543 characters). From: https://ajsonline.org/article/65872.pdf. Text file: Alavi2007.txt.
- Substitute: repository or open copy found by the lookups and saved as above (version not stated: the first pages were matched against the title, not compared with the printed version).
- Read so far (by the project): not applicable
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: not applicable.
- Read in WP-8 (2026-10-02): full text (Alavi2007.txt). Abstract (p. 1064): cover 7-12 km thick, "a ⬃200 to 300 km-wide zone", minimum shortening 16-30 %; p. 1068: average width about 300 km, anticlines "long (up to ⬃150 –200 km)"; p. 1077: SW-verging (foreland-verging) asymmetric folds, interlimb angles 0-70° in the NE, 110-140° further SW, about 170° at the front, aspect ratios (axial length to half wavelength) typically over 10; table 1 (p. 1084): shortening 16.1 to 30.2 % on six sections (DEF-004, DEF-005, DEF-027, DEF-030).

### Pollack1993
- Reference: Pollack, H. N., Hurter, S. J. & Johnson, J. R. (1993). Heat flow from the Earth's interior: analysis of the global data set. Reviews of Geophysics 31(3):267-280.
- DOI or URL: 10.1029/93RG01249
- Cited as: not cited by the code; added as an Earth reference for RCK-028 to RCK-030.
- Ledger ids: RCK-028 to RCK-030; notes: evidence only
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not applicable
- Read in this pass: abstract fetched (OpenAlex summary)
- Reference check: not applicable.

### Not bibliographic
Minecraft mod and game material: MIC rock-tag README (outside the repo); FORMAT.md; plan docs (docs/deeptime-implementation-plan.md, STATUS.md, decisions log, build log, research notes); Ben's picks and decisions (STR-002/003 design rows, "Ben's pick 2026-09-27", diamictite, stone and deepslate stand-ins). Unsourced Earth figures (no reference in the code): Zagros "about 250 km", Subandean "100-200 km", Coastal Batholith pluton counts, magma chamber depths (Kilauea, St Helens, Vesuvius), lava lakes "eight of some 1,400", stock "100 km2", CCD "4.5-5 km". Cited-but-unlisted "Buchan"-type sources: none. Athy 1930 and Sclater & Christie 1980 are not cited anywhere in this subsystem (no porosity-depth law is coded; sediment lithification is the single 150 m / 2.2 MPa rule in RCK-022).

---

## Deposit models (DEP)

### Singer2008a
- Reference: Singer, D.A., Berger, V.I., and Moring, B.C., 2008, Porphyry copper deposits of the world: database and grade and tonnage models, 2008 (version 1.0): U.S. Geological Survey Open-File Report 2008-1155, 45 p. plus data folder.
- DOI or URL: (DEP) 10.3133/ofr20081155; https://pubs.usgs.gov/of/2008/1155/ ; PDF https://pubs.usgs.gov/of/2008/1155/of2008-1155.pdf ; data file https://pubs.usgs.gov/of/2008/1155/data/PorCuTX2008.txt (ORE) https://pubs.usgs.gov/publication/ofr20081155
- Cited as: (DEP) "Singer, Berger & Moring 2008, USGS OFR 2008-1155", "OFR 2008-1155 Table 1 General Cu", "PorCuTX2008.txt (690 records)". (ORE) "USGS OFR 2008-1155 (Singer, Berger & Moring 2008)".
- Ledger ids: DEP-001 to DEP-009, DEP-013, DEP-017, DEP-018, DEP-029 to DEP-034, DEP-158; notes: companion data file: DEP-003 to 005, 009, 029
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/of/2008/1155/of2008-1155.pdf]
- Library (the project's source library, not distributed): fetched (PDFs, 45 pages, text layer). From: https://pubs.usgs.gov/of/2008/1155/. Text file: of1155.txt, PorCuTX2008.txt.
- Merged keys: Singer2008
- Read so far (by the project): (DEP) in full (table, figure and PDF page of each number recorded in data file); figure intercepts read from rendered pages; data file counted. (ORE) "Fig. 24, digitised".
- Read in this pass: (DEP) full text fetched (whole 45 p. text layer, including Table 1, the text labels of Figs. 1-26, rules, correlations); companion data file fetched and recounted (see spot check S3). (ORE) abstract fetched (the 2 km same-deposit rule is not in the abstract).
- Reference check: (DEP) matches (authors, year, series, title). The PDF page numbers in the ledger run one lower than the order of text streams I extracted (Table 1 is on the page after the correlation paragraph); that is a page-count convention, not a content problem. Note the data file I downloaded has 427 records with a tonnage, the report says 422. (ORE) matches.

### Singer2005
- Reference: Singer, D.A., Berger, V.I., Menzie, W.D., and Berger, B.R., 2005, Porphyry copper deposit density: Economic Geology 100(3):491-514.
- DOI or URL: (DEP) 10.2113/gsecongeo.100.3.491 ; USGS record https://pubs.usgs.gov/publication/70197200 (carries the full abstract) (ORE) 10.2113/gsecongeo.100.3.491
- Cited as: (DEP) "Singer, Berger, Menzie & Berger 2005", "Singer et al. 2005", "Economic Geology 100(3):491-514". (ORE) "Singer, Berger & Menzie 2005 (USGS record 70197200)".
- Ledger ids: DEP-011, DEP-013, DEP-034, DEP-147, DEP-153, DEP-166
- Access: paywalled (abstract only); full abstract open at the USGS record. [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://pubs.usgs.gov/publication/70197200. File: Singer2005.txt.
- Read so far (by the project): (DEP) abstract only. (ORE) via the USGS record, not stated whether full text.
- Read in this pass: (DEP) abstract fetched (full abstract from the USGS Publications Warehouse API). Full text not obtainable (GeoScienceWorld 403). (ORE) not fetched (Crossref metadata only).
- Reference check: (DEP) matches. Crossref lists the issue date as 2005-05-01 (issue 3 is May 2005). Important content point: the abstract defines the densities as deposits AND prospects (286) per 100,000 km2 of EXPOSED permissive rock in 19 control areas, and says density is inversely related to exposed permissive area (regression given for estimating the number in a tract). The code's "17.8 mean" is not in the paper (see S10). (ORE) the code drops the fourth author (B. R. Berger).

### Singer2008b
- Reference: Singer, D.A., 2008, Mineral deposit densities for estimating mineral resources: Mathematical Geosciences 40(1):33-46.
- DOI or URL: 10.1007/s11004-007-9127-3 ; https://pubs.usgs.gov/publication/70197201
- Cited as: not cited by the project (found in this pass; it is the follow-up to Singer2005 and contains the area-density regression).
- Ledger ids: DEP-011; notes: relevant to DEP-011, 147, 153
- Access: paywalled (abstract only); abstract open at the USGS record.
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://pubs.usgs.gov/publication/70197201. File: Singer2008b.txt.
- Read so far (by the project): not stated (not cited).
- Read in this pass: abstract fetched; a search-result snippet gave "deposit densities of 0.14 Area^-0.61 for porphyry copper" (equation not seen in the fetched abstract; units not seen).
- Reference check: n/a.

### SingerMenzie2010
- Reference: Singer, D.A., and Menzie, W.D., 2010, Quantitative mineral resource assessments: an integrated approach: New York, Oxford University Press (chapter "Estimating the Number of Undiscovered Deposits", doi 10.1093/oso/9780195399592.003.0011).
- DOI or URL: 10.1093/oso/9780195399592.003.0011 (chapter); book ISBN 9780195399592
- Cited as: named in the stage-1 task, not in the project's sources file.
- Ledger ids: none
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (search snippet only: the book "provides newly developed deposit density models").
- Reference check: n/a. The project's density numbers all come from Singer2005 abstract, not the book.

### John2010
- Reference: John, D.A., ed., Ayuso, R.A., Barton, M.D., Blakely, R.J., Bodnar, R.J., Dilles, J.H., Gray, F., Graybeal, F.T., Mars, J.L., McPhee, D., Seal, R.R., Taylor, R.D., and Vikre, P., 2010, Porphyry copper deposit model, chap. B of Mineral deposit models for resource assessment: U.S. Geological Survey Scientific Investigations Report 2010-5070-B, xii + 169 p.
- DOI or URL: (DEP) 10.3133/sir20105070B ; https://pubs.usgs.gov/sir/2010/5070/b/ ; PDF https://pubs.usgs.gov/sir/2010/5070/b/pdf/SIR10-5070B.pdf (ORE) https://pubs.usgs.gov/sir/2010/5070/b/ ; PDF .../pdf/SIR10-5070B.pdf (26.3 MB; exceeded the 10 MB fetch limit).
- Cited as: (DEP) "John, D.A., et al., 2010", "SIR 2010-5070-B". (ORE) "USGS SIR 2010-5070-B (John et al. 2010)" with page numbers (Table E1 p. 28; p. 25; p. 66; pp. 4, 65; p. 68; p. 120).
- Ledger ids: DEP-016 to DEP-018, DEP-032, DEP-035 to DEP-049, ORE-104
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/sir/2010/5070/b/pdf/SIR10-5070B.pdf]
- Library (the project's source library, not distributed): fetched (PDF, 186 pages, text layer). From: https://pubs.usgs.gov/sir/2010/5070/b/. Text file: sir5070b.txt.
- Read so far (by the project): (DEP) printed pages cited (pp. 4, 25, 28, 65-68, 120); text searched for density with no numeric value found. (ORE) page numbers cited (PDF opened).
- Read in this pass: (DEP) full text fetched (text layer; searched for every number in the ledger rows; Table E1 itself not extracted, its summary sentences were). (ORE) abstract fetched (USGS page; two different fetches listed the authors in different orders; the series page lists David A. John first as editor).
- Reference check: (DEP) the USGS catalogue lists David A. John as editor and Ayuso et al. as chapter authors; "John et al. 2010" is the usual short form but strictly John is the series editor. (ORE) matches "John et al. 2010". The report is a descriptive model; the median ore area 0.6 km2 and the 1-1.5 km height could not be read.

### SillitoeMcKee1996
- Reference: Sillitoe, R.H., and McKee, E.H., 1996, Age of supergene oxidation and enrichment in the Chilean porphyry copper province: Economic Geology 91(1):164-179.
- DOI or URL: (DEP) 10.2113/gsecongeo.91.1.164 ; USGS record 70018070 (ORE) 10.2113/gsecongeo.91.1.164
- Cited as: (DEP) "Sillitoe & McKee 1996". (ORE) "Sillitoe & McKee 1996 (USGS record 70018070)".
- Ledger ids: DEP-041, DEP-048
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Merged keys: Sillitoe1996
- Read so far (by the project): (DEP) abstract only. (ORE) via the USGS record, not stated whether full text.
- Read in this pass: (DEP) abstract fetched (OpenAlex). (ORE) not fetched (Crossref metadata only).
- Reference check: (DEP) matches. (ORE) matches. This paper concerns the Chilean province only.

### OrrisBliss1986
- Reference: Orris, G.J., and Bliss, J.D., 1986, Grade and tonnage model of placer Au-PGE (model 39a), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693, p. 261-264 (with descriptive model by Yeend, W.E., p. 261).
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf (Bulletin DOI 10.3133/b1693)
- Cited as: "Orris & Bliss 1986, Bulletin 1693 model 39a, Fig. 196 (p. 263)".
- Ledger ids: DEP-050 to DEP-055, DEP-059
- Access: open copy (agency report) at https://pubs.usgs.gov/bul/1693/report.pdf (scanned pages)
- Library (the project's source library, not distributed): fetched (PDF, 400 pages, text layer, 569,768 characters). From: https://pubs.usgs.gov/bul/1693/report.pdf. Text file: OrrisBliss1986.txt.
- Read so far (by the project): figure intercepts read at 400 dpi, text pp. 261-262.
- Read in this pass: full text fetched (text layer for pp. 261-262; Figs. 196 and 197 decoded from the scan and read).
- Reference check: matches (Bulletin text says "By Greta J. Orris and James D. Bliss", reference to Orris and Bliss 1985). Figure numbers and pages match (Fig. 196 p. 263, Fig. 197 p. 264).

### OrrisBliss1985
- Reference: Orris, G.J., and Bliss, J.D., 1985, Geologic and grade-volume data on 330 gold placer deposits: U.S. Geological Survey Open-File Report 85-213, 172 p.
- DOI or URL: 10.3133/ofr85213 ; https://pubs.usgs.gov/of/1985/0213/report.pdf
- Cited as: "Orris & Bliss 1985, OFR 85-213".
- Ledger ids: DEP-055, DEP-061
- Access: open copy (agency report) at https://pubs.usgs.gov/of/1985/0213/report.pdf
- Library (the project's source library, not distributed): fetched (PDF, 173 pages, text layer, 208,403 characters). From: https://pubs.usgs.gov/of/1985/0213/report.pdf. Text file: OrrisBliss1985.txt.
- Read so far (by the project): caveat 1 p. 2 cited.
- Read in this pass: not fetched (catalogue record only; the USGS catalogue says 173 p.).
- Reference check: matches Bulletin 1693's own reference list entry.

### Bliss1992
- Reference: Bliss, J.D., ed., 1992, Developments in mineral deposit modeling: U.S. Geological Survey Bulletin 2004, 168 p.
- DOI or URL: not found (series record not located through the API)
- Cited as: "Bliss 1992, Bulletin 2004".
- Ledger ids: DEP-061
- Access: open copy (agency report), not located by me
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): p. 2 overview paragraph.
- Read in this pass: not fetched. Title and 168 p. confirmed only from the reference list of OFR 2008-1155 ("Developments in deposit modeling").
- Reference check: OFR 2008-1155 lists the title as "Developments in deposit modeling"; the project writes "Developments in mineral deposit modeling". Minor title difference; check against the Bulletin cover.

### Schmidt2007
- Reference: Schmidt, J.M., Light, T.D., Drew, L.J., Wilson, F.H., Miller, M.L., and Saltus, R.W., 2007, Undiscovered locatable mineral resources in the Bay Resource Management Plan Area, southwestern Alaska: a probabilistic assessment: U.S. Geological Survey Scientific Investigations Report 2007-5039, vi + 50 p.
- DOI or URL: 10.3133/sir20075039 ; https://pubs.usgs.gov/sir/2007/5039/pdf/sir20075039.pdf
- Cited as: "Schmidt et al. 2007, SIR 2007-5039 p. 28 and Table 1".
- Ledger ids: DEP-051, DEP-056, DEP-061; notes: also restated in DEP-051
- Access: open copy (agency report) at https://pubs.usgs.gov/sir/2007/5039/pdf/sir20075039.pdf
- Library (the project's source library, not distributed): fetched (PDF, 60 pages, text layer). From: https://pubs.usgs.gov/sir/2007/5039/pdf/sir20075039.pdf. Text file: sir5039.txt.
- Read so far (by the project): p. 28 and Table 1.
- Read in this pass: full text fetched (tract table row for PLACER1 read, see S29).
- Reference check: matches. The USGS catalogue gives authors as Schmidt, Light, Drew, Wilson, Miller, Saltus.

### MosierSinger1986
- Reference: Mosier, D.L., and Singer, D.A., 1986, Grade and tonnage model of Superior Fe and Algoma Fe deposits (model 34a), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693, p. 228-230.
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf
- Cited as: "Mosier & Singer 1986, Bulletin 1693 p. 228-230 (model 34a)", Figs. 172-173.
- Ledger ids: DEP-062, DEP-063
- Access: open copy (agency report), scanned.
- Library (the project's source library, not distributed): fetched (PDF, 400 pages, text layer, 569,768 characters). From: https://pubs.usgs.gov/bul/1693/report.pdf. Text file: MosierSinger1986.txt.
- Read so far (by the project): figure intercepts read from the printed arrows at 300 dpi.
- Read in this pass: full text fetched (Figs. 172-174 decoded from the scan and read).
- Reference check: matches. The Bulletin says Algoma and Superior are combined "because they are not significantly different in tonnage or grades" (Algoma has its own descriptive model 28b, Superior 34a).

### Cannon1986
- Reference: Cannon, W.F., 1986, Descriptive model of Superior Fe (model 34a, p. 228) and of Algoma Fe (model 28b, p. 198), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693.
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf
- Cited as: "Cannon 1986".
- Ledger ids: DEP-062, DEP-073
- Access: open copy (agency report), scanned.
- Library (the project's source library, not distributed): fetched (PDF, 400 pages, text layer, 569,768 characters). From: https://pubs.usgs.gov/bul/1693/report.pdf. Text file: Cannon1986.txt.
- Read so far (by the project): not stated.
- Read in this pass: full text fetched (model headings only: "Model 28b DESCRIPTIVE MODEL OF ALGOMA Fe By William F. Cannon", "Model 34a DESCRIPTIVE MODEL OF SUPERIOR Fe By William F. Cannon").
- Reference check: matches (model numbers 28b and 34a confirmed).

### Cannon1995
- Reference: Cannon, W.F., Hadley, D.G., and Horton, R.J., 1995, Superior Fe deposits (model 34a) and Algoma Fe deposits (model 28b), in du Bray, E.A., ed., Preliminary compilation of descriptive geoenvironmental mineral deposit models: U.S. Geological Survey Open-File Report 95-831, 272 p.
- DOI or URL: 10.3133/ofr95831 ; https://pubs.usgs.gov/of/1995/0831/report.pdf
- Cited as: "Cannon, Hadley & Horton 1995, OFR 95-831 pp. 209-212, 252-255".
- Ledger ids: DEP-062, DEP-063, DEP-073
- Access: open copy (agency report) at https://pubs.usgs.gov/of/1995/0831/report.pdf
- Library (the project's source library, not distributed): fetched (PDF, 275 pages, text layer). From: https://pubs.usgs.gov/of/1995/0831/report.pdf. Text file: ofr95831.txt.
- Read so far (by the project): pp. 209-212, 252-255.
- Read in this pass: full text fetched (searched for deposit size statements; see S20, S25).
- Reference check: not checked author list of the two chapters (the PDF text I searched did not expose the chapter by-lines).

### Singer2009
- Reference: Singer, D.A., Berger, V.I., and Moring, B.C., 2009, Sediment-hosted zinc-lead deposits of the world: database and grade and tonnage models: U.S. Geological Survey Open-File Report 2009-1252, v + 62 p.
- DOI or URL: (DEP) 10.3133/ofr20091252 ; https://pubs.usgs.gov/of/2009/1252/ ; PDF https://pubs.usgs.gov/of/2009/1252/of2009-1252_text.pdf (ORE) https://pubs.usgs.gov/publication/ofr20091252
- Cited as: (DEP) "Singer, Berger & Moring 2009, OFR 2009-1252 Table 2 (printed pp. 9-10)". The stage-1 task calls it "Mosier, Berger & Singer 2009"; the report's authors are Singer, Berger and Moring. (ORE) "USGS OFR 2009-1252 (Singer, Berger & Moring 2009) p. 2".
- Ledger ids: DEP-074 to DEP-081, DEP-102 to DEP-105, DEP-108, DEP-114
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/of/2009/1252/of2009-1252_text.pdf]
- Library (the project's source library, not distributed): fetched (PDF, 67 pages, text layer). From: https://pubs.usgs.gov/of/2009/1252/. Text file: of1252.txt.
- Read so far (by the project): (DEP) Table 2 (printed pp. 9-10), text p. 8, p. 2; n_reported counted from the database. (ORE) page cited.
- Read in this pass: (DEP) full text fetched (Table 2, correlation paragraph, two-kilometer rule). (ORE) abstract fetched (the 2 km rule is not in the abstract).
- Reference check: (DEP) matches the project's citation; the task prompt's "Mosier, Berger & Singer" is a misattribution. (ORE) matches. The report mixes MVT and sedex under "sediment-hosted zinc-lead".

### Leach2010a
- Reference: Leach, D.L., Taylor, R.D., Fey, D.L., Diehl, S.F., and Saltus, R.W., 2010, A deposit model for Mississippi Valley-type lead-zinc ores, chap. A of Mineral deposit models for resource assessment: U.S. Geological Survey Scientific Investigations Report 2010-5070-A, viii + 43 p.
- DOI or URL: (DEP) 10.3133/sir20105070A ; https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf (ORE) https://pubs.usgs.gov/sir/2010/5070/a/ ; PDF https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf (6.3 MB; binary to WebFetch). Note the PDF name is SIR10-5070A.pdf.
- Cited as: (DEP) "Leach et al. 2010", "SIR 2010-5070-A (printed pp. 3-5, 8, 19, 27)". (ORE) "USGS SIR 2010-5070-A (the MVT deposit model)"; also called "Leach et al. 2010" in places.
- Ledger ids: DEP-081, DEP-090, DEP-091, DEP-100, ORE-001, ORE-003, ORE-008, ORE-009; notes: also cited in DEP-084, 085, 086, 095; and the MVT notes in ORE-003
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf]
- Library (the project's source library, not distributed): fetched (PDF, 64 pages, text layer). From: https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf. Text file: sir5070a.txt.
- Read so far (by the project): (DEP) printed pp. 3-5, 8, 19, 27. (ORE) not stated; quoted as a phrase.
- Read in this pass: (DEP) full text fetched (Table 1, vertical extent, fluid temperature and salinity, the 600 km criterion, district areas). (ORE) abstract fetched (USGS page). The PDF is COULD NOT READ.
- Reference check: (DEP) matches. "Leach et al. 2010" in the code is used for two different papers (this one and Leach2010b); the 2.02 Ga claim belongs to Leach2010b. (ORE) matches. Important: the code uses "Leach et al. 2010" for two different papers (this report, and Leach2010b below, Economic Geology 105:593). The authors differ.

### MosierBriskey1986
- Reference: Mosier, D.L., and Briskey, J.A., 1986, Grade and tonnage model of southeast Missouri Pb-Zn and Appalachian Zn deposits (models 32a and 32b), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693, p. 224-226.
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf
- Cited as: "Mosier & Briskey 1986, Bulletin 1693 models 32a/32b, Figs. 168-171 (printed pp. 225-226)".
- Ledger ids: DEP-101
- Access: open copy (agency report), scanned.
- Library (the project's source library, not distributed): fetched (PDF, 400 pages, text layer, 569,768 characters). From: https://pubs.usgs.gov/bul/1693/report.pdf. Text file: MosierBriskey1986.txt.
- Read so far (by the project): figure intercepts.
- Read in this pass: full text fetched (Figs. 168-171 decoded from the scan and read).
- Reference check: matches (Bulletin contents: "Grade and tonnage model of southeast Missouri Pb-Zn and Appalachian Zn deposits, by Dan L. Mosier and Joseph A. Briskey").

### Emsbo2016
- Reference: Emsbo, P., Seal, R.R., Breit, G.N., Diehl, S.F., and Shah, A.K., 2016, Sedimentary exhalative (sedex) zinc-lead-silver deposit model: U.S. Geological Survey Scientific Investigations Report 2010-5070-N, 57 p.
- DOI or URL: (DEP) 10.3133/sir20105070N ; https://pubs.usgs.gov/sir/2010/5070/n/sir20105070n.pdf (ORE) https://pubs.usgs.gov/publication/sir20105070N
- Cited as: (DEP) "Emsbo et al. 2016", "SIR 2010-5070-N pp. 2, 27, 28, 36, 39". (ORE) "SIR 2010-5070-N p. 2"; key sir2010-5070n.
- Ledger ids: DEP-106, DEP-107, DEP-116, ORE-025, ORE-026, ORE-197 to ORE-204
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/sir/2010/5070/n/sir20105070n.pdf]
- Library (the project's source library, not distributed): fetched (PDF, 72 pages, text layer). From: https://pubs.usgs.gov/sir/2010/5070/n/sir20105070n.pdf. Text file: sir5070n.txt.
- Read so far (by the project): (DEP) printed pp. 2, 27, 28, 36, 39. (ORE) in full (printed pp. 2, 28 cited).
- Read in this pass: (DEP) full text fetched (searched for each number in the ledger rows). (ORE) abstract fetched (USGS page; no temperatures or gradients in it).
- Reference check: (DEP) matches (chapter N published 2016 in a 2010 series). (ORE) matches exactly (authors, year, title, report number, 57 p.).

### MenzieMosier1986
- Reference: Menzie, W.D., and Mosier, D.L., 1986, Grade and tonnage model of sedimentary exhalative Zn-Pb (model 31a), in Cox, D.P., and Singer, D.A., eds., Mineral deposit models: U.S. Geological Survey Bulletin 1693, p. 212-215; descriptive model by Briskey, J.A., 1986, p. 211-212.
- DOI or URL: https://pubs.usgs.gov/bul/1693/report.pdf
- Cited as: "Menzie & Mosier 1986, Bulletin 1693 p. 212-215 (model 31a), Figs. 159-163".
- Ledger ids: DEP-116
- Access: open copy (agency report), scanned.
- Library (the project's source library, not distributed): fetched (PDF, 400 pages, text layer, 569,768 characters). From: https://pubs.usgs.gov/bul/1693/report.pdf. Text file: MenzieMosier1986.txt.
- Read so far (by the project): figure intercepts Figs. 159-163.
- Read in this pass: full text fetched (Figs. 159-163 decoded from the scan and read).
- Reference check: matches (Fig. 159 p. 213, Figs. 160-161 p. 214, Figs. 162-163 p. 215; "by W. David Menzie and Dan L. Mosier").

### duBray1995
- Reference: du Bray, E.A., ed., 1995, Preliminary compilation of descriptive geoenvironmental mineral deposit models: U.S. Geological Survey Open-File Report 95-831, ii + 272 p.
- DOI or URL: 10.3133/ofr95831
- Cited as: appears only as the container of Cannon1995.
- Ledger ids: DEP-199
- Access: open copy (agency report)
- Library (the project's source library, not distributed): fetched (PDF, 275 pages, text layer). From: https://doi.org/10.3133/ofr95831. Text file: ofr95831.txt.
- Read so far (by the project): see Cannon1995.
- Read in this pass: full text fetched; it also restates Menzie and Mosier 1986 (sedex "1.7 (90th percentile) to 130 (10th percentile) million tons; median ... 15") and Mosier and Singer 1986 (median 170 Mt).
- Reference check: matches.

### Singer2001
- Reference: Singer, D.A., Menzie, W.D., Sutphin, D.M., Mosier, D.L., and Bliss, J.D., 2001, Mineral deposit density: an update, in Singer, D.A., ed., Contributions to global mineral resource assessment research: U.S. Geological Survey Professional Paper 1640, chap. A, p. A1-A13.
- DOI or URL: 10.3133/pp1640A ; https://pubs.usgs.gov/pp/p1640a/p1640A.pdf
- Cited as: "Singer, Menzie, Sutphin, Mosier & Bliss 2001, PP 1640-A Table 1".
- Ledger ids: DEP-065, DEP-080, DEP-107
- Access: open copy (agency report) at https://pubs.usgs.gov/pp/p1640a/p1640A.pdf
- Library (the project's source library, not distributed): fetched (PDF, 21 pages, text layer). From: https://pubs.usgs.gov/pp/p1640a/p1640A.pdf. Text file: pp1640a.txt.
- Read so far (by the project): Table 1 (13 deposit types, no Zn-Pb, no iron formation) searched.
- Read in this pass: full text fetched (abstract and contents; Table 1 body not extracted, abstract lists its densities).
- Reference check: the PDF title page reads "By Donald A. Singer, W. David Menzie, David M. Sutphin, Dan L. Mosier, and James D. Bliss", which matches the project. The USGS catalogue and Crossref list different authors for this DOI (Menzie, Sutphin, Mosier, Bliss, Schulz; editor Singer). Cataloguing discrepancy; follow the PDF.

### USGS2000a
- Reference: U.S. Geological Survey National Mineral Resource Assessment Team, 2000, 1998 assessment of undiscovered deposits of gold, silver, copper, lead, and zinc in the United States: U.S. Geological Survey Circular 1178, 21 p. plus CD-ROM.
- DOI or URL: 10.3133/cir1178 ; https://pubs.usgs.gov/circ/c1178/c1178.pdf
- Cited as: "Circular 1178 p. 15: USGS assessments count deposits to 1 km"; "USGS Circular 1178 (1998 National Mineral Resource Assessment)".
- Ledger ids: DEP-107, DEP-154
- Access: open copy (agency report) at https://pubs.usgs.gov/circ/c1178/c1178.pdf
- Library (the project's source library, not distributed): fetched (PDF, 29 pages, text layer). From: https://pubs.usgs.gov/circ/c1178/c1178.pdf. Text file: c1178.txt.
- Merged keys: USGS2000
- Read so far (by the project): not read (page cited from memory of an agent; no data file records it).
- Read in this pass: full text fetched (the 1 km rule located; see S12).
- Reference check: the content is right but the page is wrong: the rule is on printed p. 5 (and the 1 km definition in the introduction), not p. 15.

### NMRA1998data
- Reference: U.S. Geological Survey, National Mineral Resource Assessment 1998, permissive tract shapefiles and attributes (tracts with estimated numbers of undiscovered deposits at 90, 50 and 10 percent): USGS Mineral Resources Online Spatial Data.
- DOI or URL: https://mrdata.usgs.gov/nmra/ (download https://mrdata.usgs.gov/nmra/nmra.zip)
- Cited as: "mrdata.usgs.gov/nmra", "12 NMRA tracts totalling 207,508 km2".
- Ledger ids: DEP-107
- Access: open copy (dataset)
- Library (the project's source library, not distributed): fetched (PDFs, 0 pages, text layer). From: https://mrdata.usgs.gov/nmra/. Text file: nmra.zip, nmra_tracts.txt, nmra_types.txt.
- Read so far (by the project): research by a Sonnet sub-agent, no read statement.
- Read in this pass: dataset fetched (three sedex shapefile attribute tables parsed; see S28).
- Reference check: n/a (dataset).

### Wood1983
- Reference: Wood, G.H., Jr., Kehn, T.M., Carter, M.D., and Culbertson, W.C., 1983, Coal resource classification system of the U.S. Geological Survey: U.S. Geological Survey Circular 891, vii + 65 p.
- DOI or URL: (DEP) 10.3133/cir891 ; https://pubs.usgs.gov/circ/1983/0891/report.pdf (ORE) https://pubs.usgs.gov/publication/cir891
- Cited as: (DEP) "Wood et al. 1983, Circular 891, Table 2 p. 22, Table 3 p. 24". (ORE) "USGS Circular 891 (Wood et al. 1983)", "Table 2 p. 22", "Table 3 p. 24".
- Ledger ids: DEP-128 to DEP-130, ORE-117, ORE-130, ORE-131, ORE-133, ORE-139, ORE-141 to ORE-145, ORE-151
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/circ/1983/0891/report.pdf]
- Library (the project's source library, not distributed): fetched (PDF, 72 pages, text layer). From: https://pubs.usgs.gov/circ/1983/0891/report.pdf. Text file: circ891.txt.
- Read so far (by the project): (DEP) Table 2 p. 22, Table 3 p. 24, text pp. 25-27 (lignite row read from the rendered page). (ORE) in full (page-level quotations; lignite row read from the rendered page).
- Read in this pass: (DEP) full text fetched (Tables 2 and 3, reliability definitions). (ORE) not fetched (citation page only; the USGS record prints the third and fourth authors as "Carter, M.D., and Culbertson, W.").
- Reference check: (DEP) matches. coal_seam.json says "Table 3 p. 14" in one note; the report's own list of tables puts Table 3 on p. 24. (ORE) matches. Arithmetic only: 6,000 ft is 1,828.8 m (the circular's "(1,800 m)" is a rounding); 1,800 short tons per acre-foot is 1.324 t/m3.

### Ellis1999
- Reference: Ellis, M.S., Gunther, G.L., Ochs, A.M., Roberts, S.B., Wilde, E.M., Schuenemeyer, J.H., Power, H.C., Stricker, G.D., and Blake, D., 1999, Coal resources, Powder River Basin, chap. PN; Ellis, M.S., Flores, R.M., Ochs, A.M., Stricker, G.D., Gunther, G.L., Rossi, G.S., Bader, L.R., Schuenemeyer, J.H., and Power, H.C., 1999, Gillette coalfield, Powder River Basin: geology, coal quality, and coal resources, chap. PG; both in Fort Union Coal Assessment Team, 1999 resource assessment of selected Tertiary coal beds and zones in the northern Rocky Mountains and Great Plains region: U.S. Geological Survey Professional Paper 1625-A. Chapter PS (Fort Union, Powder River) also used.
- DOI or URL: (DEP) 10.3133/pp1625A ; https://pubs.usgs.gov/pp/p1625a/Chapters/PN.pdf ; PG.pdf ; PS.pdf (ORE) https://pubs.usgs.gov/pp/p1625a/ (chapter list); chapter PDFs Chapters/PN.pdf, PG.pdf, PS.pdf under that URL.
- Cited as: (DEP) "Ellis et al. 1999, PP 1625-A chap. PN / PG / PS". (ORE) "Ellis et al. 1999, PP 1625-A chapters PN, PG, PS".
- Ledger ids: DEP-130, ORE-118 to ORE-124, ORE-128
- Access: open access (agency report). [other sections: open copy (agency report)]
- Library (the project's source library, not distributed): fetched (PDFs, 191 pages, text layer). From: https://pubs.usgs.gov/pp/p1625a/Chapters/PN.pdf. Text file: pp1625PN.txt, pp1625PG.txt, pp1625PS.txt.
- Read so far (by the project): (DEP) tables PN-1, PG-2 cited. (ORE) in full (chapter PDFs, PDF pages quoted).
- Read in this pass: (DEP) full text fetched for PN, PG, PS (three numbers checked, S41). (ORE) not fetched (chapter list read; PDFs unreadable).
- Reference check: (DEP) the by-lines come from the project's data file; the catalogue lists the volume's author as "Fort Union Coal Assessment Team and USGS Energy Resources Team". I did not see the by-lines of the PDFs. (ORE) MISMATCH in attribution. "Ellis et al. 1999" is right for PN and PG only. Chapter PS (the Fort Union 22,000 mi2 figure) is Flores and Bader, not Ellis et al. The volume is credited to the Fort Union Coal Assessment Team.

### Nuccio1994
- Reference: Nuccio, V.F., 1994, Vitrinite reflectance data for the Paleocene Fort Union and Eocene Wind River Formations, and burial history of a drill hole located in central Wind River Basin, Wyoming: U.S. Geological Survey Open-File Report 94-220, 43 p.
- DOI or URL: (DEP) 10.3133/ofr94220 (ORE) https://pubs.usgs.gov/publication/ofr94220 ; PDF https://pubs.usgs.gov/of/1994/0220/report.pdf (binary to WebFetch).
- Cited as: (DEP) "Nuccio 1994, OFR 94-220". (ORE) "OFR 94-220".
- Ledger ids: ORE-152 to ORE-155, ORE-197 to ORE-204
- Access: open access (agency report). [other sections: open copy (agency report)]
- Library (the project's source library, not distributed): fetched (PDF, 44 pages, text layer, 17,860 characters). From: https://pubs.usgs.gov/of/1994/0220/report.pdf. Text file: Nuccio1994.txt.
- Read so far (by the project): (DEP) Methods PDF p. 2 (Waples 1985 thresholds as cited there). (ORE) in full (PDF p. 2 and Fig. 2 cited).
- Read in this pass: (DEP) not fetched (catalogue record only). (ORE) not fetched (PDF unreadable by WebFetch; citation page only).
- Reference check: (DEP) title matches. (ORE) matches (the USGS page prints the author as "Nuccio, V.").

### Higley2005
- Reference: Higley, D.K., and Cox, D.O., 2005, Oil and gas exploration and development along the Front Range in the Denver Basin of Colorado, Nebraska, and Wyoming, chap. B of Energy resource studies, northern Front Range, Colorado: U.S. Geological Survey Professional Paper 1698 (162 p.).
- DOI or URL: (DEP) 10.3133/pp1698 (ORE) https://doi.org/10.3133/pp1698 (volume; USGS page https://pubs.usgs.gov/publication/pp1698 lists Fishman 2005, 162 p.). Chapter B PDF not located.
- Cited as: (DEP) "Higley & Cox 2005, USGS PP 1698 chap. B". (ORE) "USGS PP 1698 p. 22"; key pp1698-b.
- Ledger ids: ORE-160 to ORE-162, ORE-195, ORE-197 to ORE-204
- Access: open access (agency report). [other sections: open copy (agency report)]
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.43, 1950 chars).
- Read so far (by the project): (DEP) pages cited in the data file (pp. 22, 26). (ORE) in full (PDF cited by page).
- Read in this pass: (DEP) not fetched (only the series record resolved; chapter B record not found by search). (ORE) not fetched (the USGS page gave the volume citation only; chapter title and authors taken from the project's sources file, not confirmed).
- Reference check: (DEP) full title taken from the project's data file; not independently confirmed. (ORE) volume and editor confirmed; chapter B authorship and title not confirmed.

### Spencer1988
- Reference: Spencer, C.W., and Wilson, R.J., 1988, Petroleum geology and principal exploration plays in the Uinta-Piceance-Eagle Basins Province, Utah and Colorado: U.S. Geological Survey Open-File Report 88-450-G, 35 p.
- DOI or URL: (DEP) 10.3133/ofr88450G (ORE) https://pubs.usgs.gov/publication/ofr88450G
- Cited as: (DEP) "Spencer & Wilson 1988, OFR 88-450-G". (ORE) "OFR 88-450-G".
- Ledger ids: ORE-160 to ORE-162, ORE-196 to ORE-204
- Access: open access (agency report). [other sections: open copy (agency report)]
- Library (the project's source library, not distributed): fetched (PDF, 38 pages, text layer, 54,370 characters). From: https://pubs.usgs.gov/of/1988/0450g/report.pdf. Text file: Spencer1988.txt.
- Read so far (by the project): (DEP) PDF p. 10. (ORE) in full (PDF pages cited).
- Read in this pass: (DEP) not fetched (catalogue record confirms title and authors). (ORE) not fetched (citation page only; no abstract on it).
- Reference check: matches.

### Nuccio2003
- Reference: Nuccio, V.F., and Roberts, L.N.R., 2003, Thermal maturity and oil and gas generation history of petroleum systems in the Uinta-Piceance Province, Utah and Colorado, chap. 4 of USGS Uinta-Piceance Assessment Team, Petroleum systems and geologic assessment of oil and gas in the Uinta-Piceance Province, Utah and Colorado: U.S. Geological Survey Digital Data Series DDS-69-B.
- DOI or URL: (DEP) not found by search (DDS-69-B) (ORE) https://pubs.usgs.gov/publication/ds69B (volume; lists Nuccio and Roberts among the contributing authors, 28 chapters).
- Cited as: (DEP) "Nuccio & Roberts 2003, DDS-69-B ch. 4". (ORE) "DDS-69-B chap. 4"; key dds69b-ch4.
- Ledger ids: ORE-160 to ORE-162, ORE-197 to ORE-204
- Access: open access (agency report). [other sections: open copy (agency report), not located by me]
- Library (the project's source library, not distributed): fetched (PDF, 1 pages, text layer, 2,349 characters). From: https://pubs.usgs.gov/dds/0069-B/ch15_plate-1.pdf. Text file: Nuccio2003.txt.
- Read so far (by the project): (DEP) p. 6. (ORE) in full (printed p. 6 cited).
- Read in this pass: (DEP) not fetched. (ORE) not fetched (volume page only; chapter title and chapter-author pairing not confirmed).
- Reference check: (DEP) not checked (title from the project's data file). (ORE) volume title and series confirmed; chapter pairing unconfirmed.

### Dubiel2003
- Reference: Dubiel, R.F., 2003, Geology, depositional models, and oil and gas assessment of the Green River Total Petroleum System, Uinta-Piceance Province, eastern Utah and western Colorado, chap. 5 of U.S. Geological Survey Digital Data Series DDS-69-B.
- DOI or URL: (DEP) not found by search (ORE) https://pubs.usgs.gov/publication/ds69B (volume).
- Cited as: (DEP) "Dubiel 2003, DDS-69-B ch. 5". (ORE) key dds69b-ch5.
- Ledger ids: ORE-160 to ORE-162, ORE-197 to ORE-204, ORE-206
- Access: open access (agency report). [other sections: open copy (agency report), not located by me]
- Library (the project's source library, not distributed): fetched (PDF, 35 pages, text layer, 48,524 characters). From: https://pubs.usgs.gov/dds/dds-069/dds-069-b/REPORTS/Chapter_14.pdf. Text file: Dubiel2003.txt.
- Read so far (by the project): (DEP) p. 36. (ORE) in full (Appendix B printed p. 36 cited).
- Read in this pass: (DEP) not fetched. (ORE) not fetched (volume page only).
- Reference check: (DEP) not checked. (ORE) volume confirmed; chapter pairing unconfirmed. #### Coal

### Moore2003
- Reference: Moore, T.E., and Potter, C.J., 2003, Structural plays in Ellesmerian sequence and correlative strata of the National Petroleum Reserve, Alaska: U.S. Geological Survey Open-File Report 2003-253, 40 p.
- DOI or URL: (DEP) 10.3133/ofr03253 ; https://pubs.usgs.gov/of/2003/0253/pdf/of03-253.pdf (ORE) https://pubs.usgs.gov/publication/ofr03253
- Cited as: (DEP) "Moore & Potter 2003, OFR 2003-253 p. 8". (ORE) "OFR 2003-253".
- Ledger ids: ORE-160 to ORE-163, ORE-197 to ORE-204
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/of/2003/0253/pdf/of03-253.pdf]
- Library (the project's source library, not distributed): fetched (PDF, 58 pages, text layer). From: https://pubs.usgs.gov/of/2003/0253/pdf/of03-253.pdf. Text file: ofr03253.txt.
- Read so far (by the project): (DEP) p. 8. (ORE) in full (printed p. 8 cited).
- Read in this pass: (DEP) full text fetched (oil window sentence located, S46). (ORE) abstract fetched (USGS page). The abstract says hydrocarbon generation and migration "occurred primarily by sedimentary burial principally between 100-90 Ma"; it does not state an 80-120 degC oil window.
- Reference check: matches.

### Tuttle2000
- Reference: Tuttle, M.L., Brownfield, M.E., and Charpentier, R.R. (assessment geologists), 2000, Agbada Reservoirs, assessment unit 71920101 (Niger Delta Province 7192), assessment-unit summary and data form, in U.S. Geological Survey World Energy Assessment Team, U.S. Geological Survey World Petroleum Assessment 2000: description and results: U.S. Geological Survey Digital Data Series DDS-60 (file reg7/P7/tps/AU/au719211.pdf in regions/reg7.zip). Related chapters: Ahlbrandt et al. (AR), Charpentier and Klett (MC), Schmoker and Klett (AM, also Bulletin 2165), Klett et al. (GL), all 2000.
- DOI or URL: (DEP) https://pubs.usgs.gov/dds/dds-060/ ; https://pubs.usgs.gov/dds/dds-060/regions/reg7.zip ; DDS-60 DOI not resolved by Crossref (ORE) none; DDS-60 index https://pubs.usgs.gov/dds/dds-060/ (data form inside regions/reg7.zip, as the project describes).
- Cited as: (DEP) "USGS World Petroleum Assessment 2000 (DDS-60), AU 71920101 data form". (ORE) "Tuttle, Brownfield & Charpentier 2000, DDS-60 AU 71920101"; key dds60-au71920101.
- Ledger ids: DEP-132, ORE-185, ORE-186, ORE-190, ORE-197 to ORE-205
- Access: open access (agency report; zip of PDFs). [other sections: open copy (agency report) at https://pubs.usgs.gov/dds/dds-060/regions/reg7.zip]
- Library (the project's source library, not distributed): fetched (PDFs, 8 pages, text layer). From: https://pubs.usgs.gov/dds/dds-060/. Text file: au719211.txt, reg7.zip.
- Read so far (by the project): (DEP) data form p50 printed; p90 and p10 measured off the histogram at 250 dpi. (ORE) in full (data form read, histogram digitised by the project).
- Read in this pass: (DEP) data form fetched and text read (size and number tables, assessment geologists); histogram bar heights not readable from the text layer. (ORE) not fetched. The authors on the AU form could not be confirmed (the DDS-60 index lists only the chapter authors).
- Reference check: (DEP) matches (assessment geologists "M.L. Tuttle, M.E. Brownfield and R.R. Charpentier", date 5/28/98). (ORE) authorship of the AU form unconfirmed; the three-name author list may belong to the Niger Delta Province geology report rather than the AU form.

### Ofr9950H
- Reference: Tuttle, M.L., Charpentier, R.R., and Brownfield, M.E., 1999 (catalogue form of the names), The Niger Delta petroleum system: Niger Delta Province, Nigeria, Cameroon, and Equatorial Guinea, Africa: U.S. Geological Survey Open-File Report 99-50-H, 65 p.
- DOI or URL: 10.3133/ofr9950H
- Cited as: not cited by the project (found as the descriptive report for the Niger Delta unit).
- Ledger ids: none
- Access: open copy (agency report)
- Library (the project's source library, not distributed): fetched (PDF, 70 pages, text layer, 95,840 characters). From: https://pubs.usgs.gov/of/1999/0050h/report.pdf. Text file: Ofr9950H.txt.
- Read so far (by the project): not stated.
- Read in this pass: catalogue record only.
- Reference check: n/a.

### Waples1980
- Reference: Waples, D.W., 1980, Time and temperature in petroleum formation: application of Lopatin's method to petroleum exploration: AAPG Bulletin 64(6):916-926.
- DOI or URL: (DEP) https://www.osti.gov/biblio/6845191 (abstract) (ORE) 10.1306/2F9193D2-16CE-11D7-8645000102C1865D (Crossref). Abstract: https://www.osti.gov/biblio/6845191 (OSTI ID 6845191, "Am. Assoc. Pet. Geol. Bull.; (United States)", 64:6, June 1980). OpenAlex lists it as closed access.
- Cited as: (DEP) "Waples 1980, AAPG Bulletin 64:916". (ORE) "Waples 1980, AAPG Bulletin 64:916-926"; "Waples 1980, AAPG Bulletin 64:916".
- Ledger ids: ORE-158 to ORE-162, ORE-197 to ORE-204
- Access: paywalled (abstract only); abstract open at OSTI.
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://www.osti.gov/biblio/6845191. File: Waples1980.txt.
- Read so far (by the project): (DEP) abstract via OSTI 6845191. (ORE) abstract only (OSTI record).
- Read in this pass: (DEP) abstract fetched (OSTI API record, full abstract). (ORE) abstract fetched (OSTI, quoted verbatim in spot checks below). Full text not fetched (AAPG/Datapages returned 403).
- Reference check: (DEP) matches. (ORE) matches (author, year, volume, issue, pages). The abstract contains the three oil-window thresholds, the doubling rule and the four preservation deadlines. It contains no TTI-to-Ro table and no statement of the temperature interval at which n = 0. The abstract is written "approx. 500 40 degree oil preservation deadline" with the degree sign garbled as "40/sup 0/", which the project read as API gravity; the abstract itself does not say "API".

### Newell1997
- Reference: Newell, K.D., 1997 (web version 1998), Comparison of maturation data and fluid-inclusion homogenization temperatures to simple thermal models: implications for thermal history and fluid flow in the midcontinent, part 3 "TTI modeling for localities in central Kansas", Table 1: Kansas Geological Survey, Current Research in Earth Sciences, Bulletin 240, part 2.
- DOI or URL: (DEP) https://www.kgs.ku.edu/Current/1997/newell/newell3.html (ORE) https://www.kgs.ku.edu/Current/1997/newell/newell3.html (article front page: .../newell/newell1.html). No DOI.
- Cited as: (DEP) "Newell (initials not shown) ca. 1997, KGS Bulletin 240 Table 1". (ORE) "Waples 1981 as tabulated by Newell, KGS Bulletin 240 part 2, Table 1"; "kgs-b240".
- Ledger ids: ORE-152 to ORE-155, ORE-197 to ORE-204
- Access: open access (state survey web page). [other sections: open copy (state survey web page)]
- Library (the project's source library, not distributed): fetched (PDF, 0 pages, text layer). From: https://www.kgs.ku.edu/Current/1997/newell/newell3.html. Text file: kgs.txt.
- Read so far (by the project): (DEP) web page; Waples' own table not opened. (ORE) secondary source (web page, flagged non-USGS).
- Read in this pass: (DEP) full text fetched (Table 1 read, S45). Author is K. David Newell (part 1 of the same article gives the by-line); the project wrote "initials not shown". (ORE) full text fetched (the section page: Table 1 rows and the doubling sentence; article front page for title and author). The page does not print a reference list, so what "Waples 1981" is cannot be read from it. The text says "Waples (1980, 1981)".
- Reference check: (DEP) Table 1 is attributed to Waples (1981) for TTI to Ro, so it is not Waples 1980. (ORE) author is K. David Newell (the sources file lacked the initials). Title and the "1997" year: the URL path says 1997, web version dated 1998-03-16. The "Waples 1981" it credits is probably Waples's 1981 book (Organic Geochemistry for Exploration Geologists), but I could not confirm that from the page; treat as unidentified.

### Dyman2002
- Reference: Dyman, T.S., Crovelli, R.A., Bartberger, C.E., and Takahashi, K.I., 2002, Worldwide estimates of deep natural gas resources based on the U.S. Geological Survey World Petroleum Assessment 2000: Natural Resources Research 11(3):207-218.
- DOI or URL: (DEP) 10.1023/A:1019860722244 ; USGS record 70023990 (ORE) 10.1023/A:1019860722244 (OpenAlex).
- Cited as: "Dyman et al. 2002, Nat. Resour. Res. 11:207".
- Ledger ids: ORE-190 to ORE-192
- Access: paywalled (abstract only); abstract open at the USGS record. [other sections: paywalled (abstract only); Springer page redirected to a login, abstract not retrieved.]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) no read statement. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (USGS record; deep means more than 4.5 km). (ORE) not fetched (OpenAlex metadata only).
- Reference check: (DEP) matches (the project writes "Nat. Resour. Res." for Natural Resources Research). (ORE) matches (year, journal, volume, first page). I did not confirm that the paper defines "deep" as below 4.5 km or that USGS assessments have no depth cut-off.

### Kesler2006
- Reference: Kesler, S.E., and Wilkinson, B.H., 2006, The role of exhumation in the temporal distribution of ore deposits: Economic Geology 101(5):919-922.
- DOI or URL: 10.2113/gsecongeo.101.5.919
- Cited as: (DEP) "Kesler & Wilkinson 2006, Economic Geology 101(5):919". (ORE) "Kesler & Wilkinson 2006 and 2008 (exhumation)".
- Ledger ids: DEP-166, DEP-168, DEP-172, DEP-175, DEP-191, ORE-220, ORE-221
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) OpenAlex/Semantic Scholar abstract. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (OpenAlex). (ORE) not fetched (Crossref metadata only).
- Reference check: (DEP) matches. It is a 4-page paper (919-922). (ORE) matches.

### Kesler2008
- Reference: Kesler, S.E., and Wilkinson, B.H., 2008, Earth's copper resources estimated from tectonic diffusion of porphyry copper deposits: Geology 36(3):255-258.
- DOI or URL: (DEP) 10.1130/G24317A.1 (SCL, PRV, POL, BAK) 10.1130/G24317A.1 ; https://pubs.geoscienceworld.org/geology/article/36/3/255-258/29682 (403)
- Cited as: (DEP) "Kesler & Wilkinson 2008, Geology". (ORE) "Kesler & Wilkinson ... 2008 (exhumation)". (SCL, PRV, POL, BAK) "Kesler & Wilkinson 2008, doi 10.1130/g24317a.1"
- Ledger ids: DEP-166, DEP-168, DEP-172, DEP-177, DEP-191, ORE-220, ORE-221, SCL-006
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): (DEP) abstract-level. (ORE) not stated. (SCL, PRV, POL, BAK) not stated; DOI linked
- Read in this pass: (DEP) abstract fetched (OpenAlex). (ORE) not fetched (Crossref metadata only; last page 258 from the usual form, not confirmed). (SCL, PRV, POL, BAK) abstract fetched (OpenAlex record; the abstract text is the tool's reconstruction from the index)
- Reference check: (DEP) matches. (ORE) probable match for "2008"; the code's phrasing "(exhumation)" fits the 2006 paper better than this one. #### Found in this pass, not cited by the code (SCL, PRV, POL, BAK) citation matches. What it is cited for ("deposits above 3.3 km are within mining reach") is NOT in the abstract; the abstract is about estimating the number of porphyry copper deposits formed (about 125,895 in the Phanerozoic), preserved (about 47,789), and total copper (about 1.7 x 10^11 t in preserved porphyry deposits, 3 x 10^11 t all resources). The 3.3 km figure may be in the body; unverified.

### Singer1993
- Reference: Singer, D.A., 1993, Basic concepts in three-part quantitative assessments of undiscovered mineral resources: Nonrenewable Resources 2(2):69-81.
- DOI or URL: 10.1007/BF02272804 ; https://pubs.usgs.gov/publication/70018261
- Cited as: "Singer 1993".
- Ledger ids: DEP-145, DEP-184, DEP-185
- Access: paywalled (abstract only); abstract open at the USGS record.
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://pubs.usgs.gov/publication/70018261. File: Singer1993.txt.
- Read so far (by the project): abstract in the research note.
- Read in this pass: abstract fetched (USGS record, full abstract).
- Reference check: matches.

### Root1992
- Reference: Root, D.H., Menzie, W.D., and Scott, W.A., 1992, Computer Monte Carlo simulation in quantitative resource estimation: Nonrenewable Resources 1(2):125-138.
- DOI or URL: 10.1007/BF01782266
- Cited as: "Root, Menzie & Scott 1992 (Monte Carlo combination of three-part assessments)".
- Ledger ids: DEP-145
- Access: paywalled (abstract only), not read
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: title and venue found by search; the project gave only a parenthetical description.

### Skirrow2019
- Reference: Skirrow, R.G., Murr, J., Schofield, A., Huston, D.L., van der Wielen, S., Czarnota, K., and others, 2019, Mapping iron oxide Cu-Au (IOCG) mineral potential in Australia using a knowledge-driven mineral systems-based approach: Ore Geology Reviews 113:103011.
- DOI or URL: 10.1016/j.oregeorev.2019.103011
- Cited as: "Skirrow et al. 2019".
- Ledger ids: DEP-178
- Access: paywalled (abstract only), not read
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 11 chars).
- Substitute: none found.
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: probable identification (the project gives no title). It is an IOCG case study, so it is a weak authority for a general "components are multiplied" rule.

### McCuaig2010
- Reference: McCuaig, T.C., Beresford, S., and Hronsky, J., 2010, Translating the mineral systems approach into an effective exploration targeting system: Ore Geology Reviews 38(3):128-138.
- DOI or URL: 10.1016/j.oregeorev.2010.05.008
- Cited as: "McCuaig, Beresford & Hronsky 2010".
- Ledger ids: DEP-178
- Access: paywalled (abstract only), not read
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 11 chars).
- Substitute: none found.
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Hronsky2008
- Reference: Hronsky, J.M.A., and Groves, D.I., 2008, Science of targeting: definition, strategies, targeting and performance measurement: Australian Journal of Earth Sciences 55(1):3-12.
- DOI or URL: 10.1080/08120090701581356
- Cited as: "Hronsky & Groves 2008".
- Ledger ids: DEP-178
- Access: paywalled (abstract only), not read
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Agterberg1995
- Reference: Agterberg, F.P., 1995, Multifractal modeling of the sizes and grades of giant and supergiant deposits: International Geology Review 37(1):1-8.
- DOI or URL: 10.1080/00206819509465388
- Cited as: "Agterberg 1995 (lognormal with Pareto tail)".
- Ledger ids: DEP-015, DEP-155, DEP-162
- Access: paywalled (abstract only), not read
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: probable identification (project gives only author and year); title concerns multifractal, not "lognormal with Pareto tail" per se.

### Singer1995
- Reference: Singer, D.A., 1995, World class base and precious metal deposits: a quantitative analysis: Economic Geology 90(1):88-104.
- DOI or URL: 10.2113/gsecongeo.90.1.88 ; USGS record 70197194
- Cited as: "Singer 1995 (world-class share)".
- Ledger ids: DEP-015, DEP-155, DEP-162
- Access: paywalled (abstract only); abstract open at the USGS record.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (USGS record).
- Reference check: matches ("between 47 and 79 percent of metal contained in the largest 10 percent of deposits").

### Robert2005
- Reference: Robert, F., Poulsen, K.H., Cassidy, K.F., and Hodgson, C.J., 2005, Gold metallogeny of the Superior and Yilgarn cratons, in One Hundredth Anniversary Volume (Economic Geology): Society of Economic Geologists, p. 1001-1033.
- DOI or URL: 10.5382/av100.30
- Cited as: (DEP) "Robert et al. 2005 (orogenic-gold camps 30-50 km)". (ORE) "Robert et al. 2005 (orogenic-gold camps every 30-50 km)".
- Ledger ids: DEP-015, ORE-078
- Access: paywalled (abstract only), not read [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) abstract. (ORE) not stated.
- Read in this pass: (DEP) not fetched (Crossref metadata only; the 30-50 km figure could not be checked). (ORE) not fetched (Crossref metadata only).
- Reference check: (DEP) probable identification (project gives only author and year). (ORE) identified; the 30-50 km spacing is unverified. The paper is about the Superior and Yilgarn cratons.

### Butterworth2016
- Reference: Butterworth, N.P., Steinberg, D., Müller, R.D., Williams, S.E., Merdith, A.S., and Hardy, S., 2016, Tectonic environments of South American porphyry copper magmatism through time revealed by spatiotemporal data mining: Tectonics 35(12):2847-2862.
- DOI or URL: 10.1002/2016TC004289
- Cited as: (DEP) "Butterworth et al. 2016 (machine learning on South American subduction)". (ORE) "Butterworth et al. 2016".
- Ledger ids: ORE-032 to ORE-034
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) abstract. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (OpenAlex). (ORE) not fetched (OpenAlex metadata only).
- Reference check: (DEP) matches. (ORE) matches. The paper covers South American porphyry copper only; the code applies its convergence-rate, obliquity and slab-age peaks to all porphyries.

### Richards2003
- Reference: Richards, J.P., 2003, Tectono-magmatic precursors for porphyry Cu-(Mo-Au) deposit formation: Economic Geology 98(8):1515-1533.
- DOI or URL: 10.2113/gsecongeo.98.8.1515
- Cited as: "Richards 2003".
- Ledger ids: DEP-025, DEP-026, ORE-030, ORE-031
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) abstract. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (OpenAlex). (ORE) not fetched (OpenAlex metadata only).
- Reference check: (DEP) matches (abstract covers oxidised, hydrous magmas and fault-controlled ascent). (ORE) matches.

### Cooke2005
- Reference: Cooke, D.R., Hollings, P., and Walshe, J.L., 2005, Giant porphyry deposits: characteristics, distribution, and tectonic controls: Economic Geology 100(5):801-818.
- DOI or URL: (DEP) 10.2113/gsecongeo.100.5.801 (author copy: https://figshare.com/articles/journal_contribution/Giant_porphyry_deposits_Characteristics_distribution_and_tectonic_controls/22856222) (ORE) 10.2113/gsecongeo.100.5.801 ; open copy https://figshare.com/articles/journal_contribution/Giant_porphyry_deposits_Characteristics_distribution_and_tectonic_controls/22856222
- Cited as: "Cooke et al. 2005".
- Ledger ids: DEP-025, DEP-026, ORE-030, ORE-031
- Access: open copy (author copy on figshare, per OpenAlex) [other sections: open copy (figshare) at the URL above.]
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://figshare.com/articles/journal_contribution/Giant_porphyry_deposits_Characteristics_distribution_and_tectonic_controls/22856222. File: Cooke2005.txt.
- Read so far (by the project): (DEP) abstract. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (OpenAlex); the figshare copy was not opened. (ORE) not fetched (OpenAlex metadata only).
- Reference check: (DEP) matches ("compressive tectonic environments, thickened continental crust"). (ORE) matches.

### Bekker2010
- Reference: Bekker, A., Slack, J.F., Planavsky, N., Krapež, B., Hofmann, A., Konhauser, K.O., and Rouxel, O.J., 2010, Iron formation: the sedimentary product of a complex interplay among mantle, tectonic, oceanic, and biospheric processes: Economic Geology 105(3):467-508.
- DOI or URL: 10.2113/gsecongeo.105.3.467
- Cited as: (DEP) "Bekker et al. 2010 (BIF acme 2.65-1.85 Ga)". (ORE) "Bekker et al. 2010 (BIF acme 2.65-1.85 Ga; Superior-type on passive-margin shelves)".
- Ledger ids: ORE-016, ORE-019, ORE-021
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 404 text/html.
- Substitute: none found.
- Read so far (by the project): (DEP) abstract. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched but cut off before the age statement (OpenAlex); the 2.65-1.85 Ga window not seen. (ORE) abstract fetched in part (OpenAlex; the stored index ends at "Secular changes in deposition, identified more than 20 years ago,", so the dates are cut off).
- Reference check: (DEP) title and journal match; age window unverified. (ORE) matches (authors, year, volume, pages).

### Bohacs1997
- Reference: Bohacs, K., and Suter, J., 1997, Sequence stratigraphic distribution of coaly rocks: fundamental controls and paralic examples: AAPG Bulletin 81(10):1612-1639.
- DOI or URL: (DEP) 10.1306/3B05C3FC-172A-11D7-8645000102C1865D (ORE) 10.1306/3b05c3fc-172a-11d7-8645000102c1865d (Crossref).
- Cited as: "Bohacs & Suter 1997".
- Ledger ids: DEP-122, ORE-112 to ORE-114
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) abstract. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (OpenAlex). (ORE) abstract fetched (Crossref abstract, verbatim).
- Reference check: (DEP) matches ("significant volumes of terrigenous organic matter can be preserved to form coals only when and where the overall increase in accommodation approximately equals the production rate of peat"). (ORE) matches (authors, year, volume, issue, pages, DOI).

### Nelsen2016
- Reference: Nelsen, M.P., DiMichele, W.A., Peters, S.E., and Boyce, C.K., 2016, Delayed fungal evolution did not cause the Paleozoic peak in coal production: Proceedings of the National Academy of Sciences 113(9):2442-2447.
- DOI or URL: (DEP) 10.1073/pnas.1517943113 (ORE) 10.1073/pnas.1517943113 ; open copy https://pmc.ncbi.nlm.nih.gov/articles/PMC4780611/
- Cited as: "Nelsen et al. 2016".
- Ledger ids: DEP-122, ORE-015, ORE-113, ORE-114
- Access: open access. [other sections: open copy at https://www.ncbi.nlm.nih.gov/pmc/articles/4780611 (per Semantic Scholar's open-access field; link not opened)]
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): (DEP) abstract. (ORE) not stated.
- Read in this pass: (DEP) not fetched (Crossref metadata only). (ORE) full text fetched (PMC; the introduction's statement on the first appearance of coal and the Carboniferous-Permian peak).
- Reference check: (DEP) probable identification only: the project gives author and year, and this is the 2016 Nelsen et al. paper on coal production. (ORE) the DOI and authors are right, but the paper is about the Carboniferous-Permian coal peak and fungi, not about when coal formation began. It does contain the sentence "Coal deposits occur by the Early Devonian", so it supports the code's land-plant gate only indirectly. #### Ore genesis and drivers

### Hoggard2020
- Reference: Hoggard, M.J., Czarnota, K., Richards, F.D., Huston, D.L., Jaques, A.L., and Ghelichkhan, S., 2020, Global distribution of sediment-hosted metals controlled by craton edge stability: Nature Geoscience 13(7):504-510.
- DOI or URL: (DEP) 10.1038/s41561-020-0593-2 (ORE) 10.1038/s41561-020-0593-2 ; preprint 10.31223/osf.io/2kjvc (EarthArXiv).
- Cited as: (DEP) "Hoggard et al. 2020 (85 percent within 200 km)". (ORE) "Hoggard et al. 2020 (85 % within 200 km)".
- Ledger ids: ORE-027, ORE-028
- Access: open access [other sections: open copy (preprint) at https://doi.org/10.31223/osf.io/2kjvc (API record https://api.osf.io/v2/preprints/2kjvc/); the journal version is paywalled.]
- Library (the project's source library, not distributed): fetched (HTML page, 31,072 characters of text). From: https://doi.org/10.1038/s41561-020-0593-2. Text file: Hoggard2020.txt.
- Read so far (by the project): (DEP) abstract. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (Semantic Scholar). (ORE) abstract fetched (OSF API record of the preprint).
- Reference check: matches.

### Leach2001
- Reference: Leach, D.L., Bradley, D.C., Lewchuk, M.T., Symons, D.T.A., de Marsily, G., and Brannon, J., 2001, Mississippi Valley-type lead-zinc deposits through geological time: implications from recent age-dating research: Mineralium Deposita 36(8):711-740.
- DOI or URL: 10.1007/s001260100208
- Cited as: (DEP) "Leach et al. 2001". (ORE) "Leach et al. 2001 (MVT mainly in contractional events)".
- Ledger ids: DEP-082, DEP-094 to DEP-098, ORE-006, ORE-008 to ORE-010, ORE-013, ORE-014; notes: as a group
- Access: paywalled, not read [other sections: paywalled (abstract only); abstract not retrieved.]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) no read statement. (ORE) not stated.
- Read in this pass: (DEP) not fetched (metadata only). (ORE) not fetched (OpenAlex metadata only).
- Reference check: (DEP) probable identification (project gives author and year). (ORE) matches. A published discussion exists (Kesler and Carrigan 2002, Mineralium Deposita 37:800-802, open copy https://hdl.handle.net/2027.42/41891).

### Bradley2003
- Reference: Bradley, D.C., and Leach, D.L., 2003, Tectonic controls of Mississippi Valley-type lead-zinc mineralization in orogenic forelands: Mineralium Deposita 38(6):652-667.
- DOI or URL: 10.1007/s00126-003-0355-2
- Cited as: (DEP) "Bradley & Leach 2003". (ORE) "Bradley & Leach 2003 (flexure-induced faults)".
- Ledger ids: DEP-095, ORE-008, ORE-009
- Access: paywalled, not read [other sections: paywalled (abstract only); abstract not retrieved.]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) no read statement. (ORE) not stated.
- Read in this pass: (DEP) not fetched (Crossref metadata only). (ORE) not fetched (OpenAlex metadata only).
- Reference check: (DEP) matches. (ORE) matches (authors, year, volume, pages).

### Leach2005
- Reference: Leach, D.L., Sangster, D.F., Kelley, K.D., Large, R.R., Garven, G., Allen, C.R., Gutzmer, J., and Walters, S., 2005, Sediment-hosted lead-zinc deposits: a global perspective, in One Hundredth Anniversary Volume (Economic Geology): Society of Economic Geologists, p. 561-607.
- DOI or URL: (DEP) 10.5382/av100.18 (ORE) 10.5382/av100.18 (Crossref; Crossref prints the title "Sediment-Hosted Lead-Zinc Deposits"). Editors listed from the usual volume record, not from Crossref.
- Cited as: (DEP) "Leach et al. 2005 (90-200 C)". (ORE) "Leach et al. 2005 (90-200 degC MVT fluids)".
- Ledger ids: DEP-082, DEP-094, DEP-095, ORE-003
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) no read statement. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (OpenAlex); the 90-200 C figure is not in the abstract. (ORE) not fetched (Crossref metadata only).
- Reference check: (DEP) matches; the temperature figure remains unchecked (SIR 2010-5070-A gives 50-250, mostly 75-200 C). (ORE) identified; the 90-200 degC statement is unverified.

### Leach2010b
- Reference: Leach, D.L., Bradley, D.C., Huston, D.L., Pisarevsky, S.A., Taylor, R.D., and Gardoll, S.J., 2010, Sediment-hosted lead-zinc deposits in Earth history: Economic Geology 105(3):593-625.
- DOI or URL: (DEP) 10.2113/gsecongeo.105.3.593 (ORE) 10.2113/gsecongeo.105.3.593 (Crossref).
- Cited as: (DEP) the code and plan write "Leach et al. 2010" for the 2.02 Ga onset; the project attaches it to SIR 2010-5070-A. (ORE) "Leach et al. 2010 (Economic Geology 105:593; sediment-hosted Pb-Zn ... first appearance at 2.02 Ga)".
- Ledger ids: DEP-112, DEP-114, DEP-188, ORE-009, ORE-010, ORE-014, ORE-024; notes: claim
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) not stated; probably via the research note. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (OpenAlex). (ORE) abstract fetched (OpenAlex inverted index; the rebuilt text is word-order scrambled in places, so quote only the intact phrases).
- Reference check: (DEP) this is the paper that carries the 2.02 Ga statement (see S33). The project has conflated it with Leach2010a. (ORE) matches (authors, year, volume, pages). The abstract divides the deposits into clastic-dominated (which includes what were called SEDEX) and MVT; the code applies the 2.02 Ga gate to both.

### Garven1984
- Reference: Garven, G., and Freeze, R.A., 1984, Theoretical analysis of the role of groundwater flow in the genesis of stratabound ore deposits: 1. Mathematical and numerical model, American Journal of Science 284(10):1085-1124; 2. Quantitative results, American Journal of Science 284(10):1125-1174.
- DOI or URL: 10.2475/ajs.284.10.1085 ; 10.2475/ajs.284.10.1125
- Cited as: "Garven & Freeze 1984 (base case: 1 km water-table drop over 300 km)".
- Ledger ids: DEP-095, DEP-098, ORE-006
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) no read statement. (ORE) not stated.
- Read in this pass: (DEP) abstracts of both parts fetched (OpenAlex); the base case numbers are not in either abstract. (ORE) abstract fetched (OpenAlex, both parts). Neither abstract mentions the 1 km over 300 km base case.
- Reference check: (DEP) the paper is two parts; the code does not say which. The base case is probably in part 2. (ORE) the code does not say which part; the base case is most likely in part 2 (quantitative results). Unconfirmed.

### Garven1985
- Reference: Garven, G., 1985, The role of regional fluid flow in the genesis of the Pine Point deposit, Western Canada sedimentary basin: Economic Geology 80(2):307-324.
- DOI or URL: 10.2113/gsecongeo.80.2.307
- Cited as: (DEP) "Garven 1985". (ORE) "Garven 1985 (Pine Point, 60-100 degC, flushes of 0.5-5 Myr)".
- Ledger ids: DEP-082, DEP-094, DEP-095, ORE-003, ORE-006, ORE-013
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) abstract (research note). (ORE) not stated.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: (DEP) matches. (ORE) matches. The abstract contains both the 60-100 degC and the 0.5-5.0 Myr values.

### Bethke1990
- Reference: Bethke, C.M., and Marshak, S., 1990, Brine migrations across North America: the plate tectonics of groundwater: Annual Review of Earth and Planetary Sciences 18:287-315.
- DOI or URL: 10.1146/annurev.ea.18.050190.001443
- Cited as: (DEP) "Bethke & Marshak 1990 quote". (ORE) "Bethke & Marshak 1990 (topographic relief the dominant driving force)".
- Ledger ids: DEP-095, ORE-006
- Access: paywalled, not read [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) no read statement. (ORE) not stated (the quoted phrase suggests the text was seen).
- Read in this pass: (DEP) not fetched (Crossref metadata only). (ORE) not fetched (OpenAlex metadata only).
- Reference check: matches.

### Oliver1986
- Reference: Oliver, J., 1986, Fluids expelled tectonically from orogenic belts: their role in hydrocarbon migration and other geologic phenomena: Geology 14(2):99 (end page not seen).
- DOI or URL: 10.1130/0091-7613(1986)14<99:FETFOB>2.0.CO;2
- Cited as: "Oliver 1986".
- Ledger ids: DEP-095
- Access: paywalled, not read
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Sillitoe2005
- Reference: Sillitoe, R.H., 2005, Supergene oxidized and enriched porphyry copper and related deposits, in One Hundredth Anniversary Volume (Economic Geology): Society of Economic Geologists, p. 723-768.
- DOI or URL: 10.5382/av100.22
- Cited as: (DEP) "Sillitoe 2005 (supergene windows, 0.5 Myr, 2-3x)". (ORE) "Sillitoe 2005".
- Ledger ids: DEP-041, DEP-043
- Access: paywalled (abstract only); abstract open via Crossref. [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) abstract in the research note; the data file says "typically 3-9 Myr" was not found in any USGS document. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (Crossref, full abstract). (ORE) not fetched (Crossref metadata only).
- Reference check: (DEP) matches. The abstract itself contains both "at least 0.5 m.y." and "typically minima of 3 to 9 m.y." (S13). (ORE) matches.

### Titley1995
- Reference: Titley, S.R., and Marozas, D.C., 1995, Processes and products of supergene copper enrichment, in Pierce, F.W., and Bolm, J.G., eds., Porphyry copper deposits of the American Cordillera: Arizona Geological Society Digest 20, p. 156-168.
- DOI or URL: none (reference taken from the SIR 2010-5070-B reference list)
- Cited as: "Titley & Marozas 1995" (as cited inside SIR 2010-5070-B).
- Ledger ids: DEP-040, DEP-046
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): second-hand through the SIR.
- Read in this pass: not fetched; reference string read in SIR 2010-5070-B.
- Reference check: matches.

### Clark1990
- Reference: Clark, A.H., Tosdal, R.M., Farrar, E., and Plazolles V., A., 1990, Geomorphologic environment and age of supergene enrichment of the Cuajone, Quellaveco, and Toquepala porphyry copper deposits, southeastern Peru: Economic Geology 85(7):1604-1628.
- DOI or URL: 10.2113/gsecongeo.85.7.1604
- Cited as: "Clark et al. 1990" (as cited in SIR 2010-5070-B).
- Ledger ids: DEP-040
- Access: paywalled, not read
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): second-hand through the SIR.
- Read in this pass: not fetched; reference string read in SIR 2010-5070-B.
- Reference check: matches. It concerns southern Peru; the SIR sentence quoted for the "10 cm/year" climate is attributed to it.

### Mote2001
- Reference: Mote, T.I., Becker, T.A., Renne, P., and Brimhall, G.H., 2001, Chronology of exotic mineralization at El Salvador, Chile, by 40Ar/39Ar dating of copper wad and supergene alunite: Economic Geology 96(2):351-366.
- DOI or URL: 10.2113/gsecongeo.96.2.351
- Cited as: "Mote et al. 2001" (as cited in SIR 2010-5070-B).
- Ledger ids: DEP-048
- Access: paywalled, not read
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): second-hand through the SIR.
- Read in this pass: not fetched; reference string read in SIR 2010-5070-B (SIR initials "T.L. Mote", Crossref "T.I. Mote").
- Reference check: matches.

### Sillitoe2010
- Reference: Sillitoe, R.H., 2010, Porphyry copper systems: Economic Geology 105(1):3-41.
- DOI or URL: 10.2113/gsecongeo.105.1.3
- Cited as: "Sillitoe 2010".
- Ledger ids: DEP-021, DEP-190; notes: also DEP-021, 188
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches (abstract says stocks "vertically elongate (>3 km)", pluton paleodepths 5 to 15 km, life spans ~100,000 to several million years).

### Seedorff2005
- Reference: Seedorff, E., Dilles, J.H., Proffett, J.M., Einaudi, M.T., Zurcher, L., Stavast, W.J.A., and others, 2005, Porphyry deposits: characteristics and origin of hypogene features, in One Hundredth Anniversary Volume (Economic Geology): Society of Economic Geologists, p. 251-298.
- DOI or URL: 10.5382/av100.10
- Cited as: "Seedorff et al. 2005".
- Ledger ids: DEP-190
- Access: paywalled, not read
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract.
- Read in this pass: not fetched (Crossref metadata only; the Crossref listing shows the first six authors; the full author list was not retrieved).
- Reference check: probable identification.

### Heckel1986
- Reference: Heckel, P.H., 1986, Sea-level curve for Pennsylvanian eustatic marine transgressive-regressive depositional cycles along midcontinent outcrop belt, North America: Geology 14(4):330 (end page not seen).
- DOI or URL: 10.1130/0091-7613(1986)14<330:SCFPEM>2.0.CO;2
- Cited as: (DEP) "Heckel 1986 (cyclothem durations)". (ORE) "Heckel 1986".
- Ledger ids: ORE-139 to ORE-145
- Access: paywalled, not read [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) via the coal research sub-agent, no read statement. (ORE) search snippet only.
- Read in this pass: (DEP) not fetched (Crossref metadata only). (ORE) not fetched.
- Reference check: (DEP) probable identification; the project gives only author and year. (ORE) identified; the code's claim is about cyclothem durations (160,000-400,000 yr), which I did not verify.

### Rasbury1998
- Reference: Rasbury, E.T., Hanson, G.N., Meyers, W.J., Holt, W.E., Goldstein, R.H., and Saller, A.H., 1998, U-Pb dates of paleosols: constraints on late Paleozoic cycle durations and boundary ages: Geology 26(5):403 (end page not seen).
- DOI or URL: 10.1130/0091-7613(1998)026<0403:UPDOPC>2.3.CO;2
- Cited as: (DEP) "Rasbury et al. 1998 (cyclothem durations)". (ORE) "Rasbury et al. 1998".
- Ledger ids: ORE-139 to ORE-145
- Access: paywalled, not read [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) via the coal research sub-agent, no read statement. (ORE) search snippet only.
- Read in this pass: (DEP) not fetched (Crossref metadata only; end page not seen). (ORE) not fetched.
- Reference check: (DEP) probable identification. (ORE) identified by title; the title fits the cyclothem-duration claim, which I did not read.

### Blackman2021
- Reference: Blackman, D., and Vigna, S., 2021, Scrambled linear pseudorandom number generators: ACM Transactions on Mathematical Software 47(4):1-32.
- DOI or URL: 10.1145/3460772
- Cited as: "Blackman & Vigna 2021 (xoshiro256++)".
- Ledger ids: DEP-164
- Access: open copy (arXiv preprint https://arxiv.org/pdf/1805.01407 per Semantic Scholar's open-access field; not opened)
- Library (the project's source library, not distributed): fetched (PDF, 32 pages, text layer, 79,855 characters). From: https://arxiv.org/pdf/1805.01407. Text file: Blackman2021.txt.
- Read so far (by the project): "the constants of the paper".
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches.

### Marsaglia1964
- Reference: Marsaglia, G., and Bray, T.A., 1964, A convenient method for generating normal variables: SIAM Review 6(3):260-264.
- DOI or URL: 10.1137/1006063
- Cited as: "Marsaglia polar method" (the project gives no citation; this is the usual source of the polar method).
- Ledger ids: DEP-164
- Access: paywalled, not read
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): no read statement.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: probable identification.

### Hormann1993
- Reference: Hörmann, W., 1993, The transformed rejection method for generating Poisson random variables: Insurance: Mathematics and Economics 12(1):39-45.
- DOI or URL: 10.1016/0167-6687(93)90997-4
- Cited as: "Hormann 1993, Insurance: Mathematics and Economics 12:39".
- Ledger ids: DEP-164
- Access: paywalled, not read
- Library (the project's source library, not distributed): fetched (PDF, 14 pages, text layer, 29,451 characters). From: https://research.wu.ac.at/ws/files/18953249/document.pdf. Text file: Hormann1993.txt.
- Substitute: repository or open copy found by the lookups and saved as above (version not stated: the first pages were matched against the title, not compared with the printed version).
- Read so far (by the project): "the constants of the paper".
- Read in this pass: not fetched; the constants were checked against NumPy's implementation of the algorithm (S50).
- Reference check: matches.

### AbramowitzStegun1964
- Reference: Abramowitz, M., and Stegun, I.A., eds., 1964, Handbook of mathematical functions with formulas, graphs, and mathematical tables: National Bureau of Standards Applied Mathematics Series 55, formula 7.1.26.
- DOI or URL: none
- Cited as: "Abramowitz & Stegun 7.1.26 (error under 1.5e-7)".
- Ledger ids: DEP-165
- Access: book (not online here)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): no read statement.
- Read in this pass: not fetched.
- Reference check: the edition year (1964) is the standard first edition; not checked here.

### Acklam
- Reference: Acklam, P.J., An algorithm for computing the inverse normal cumulative distribution function (web note, undated, around 2000-2003).
- DOI or URL: http://home.online.no/~pjacklam/notes/invnorm/ (read through the Internet Archive capture of 2012 at https://web.archive.org/web/2012/http://home.online.no/~pjacklam/notes/invnorm/index.html)
- Cited as: "Acklam's rational approximation of the inverse normal CDF".
- Ledger ids: DEP-165
- Access: open copy (web page, archived)
- Library (the project's source library, not distributed): fetched (PDF, 0 pages, text layer). From: http://home.online.no/~pjacklam/notes/invnorm/. Text file: acklam3.txt.
- Read so far (by the project): no read statement.
- Read in this pass: full text fetched (relative error and the 0.02425 split read, S50).
- Reference check: matches the algorithm; undated web note, not a publication.

### Software
- Reference: Ellefsen, K.J., 2017, User's guide for MapMark4: an R package for the probability calculations in three-part mineral resource assessments: U.S. Geological Survey Techniques and Methods, book 7, chap. C14 (doi 10.3133/tm7c14); Duval, J.S., 2012, Version 3.0 of EMINERS, Economic Mineral Resource Simulator: U.S. Geological Survey Open-File Report 2004-1344 (doi 10.3133/ofr20041344); EIS Toolkit (EUPL-1.2, offline calibration only): Reference: UNIDENTIFIED (no citation given; not searched).
- DOI or URL: as given.
- Cited as: "MapMark4 (USGS R package, CC0, method reference)", "EMINERS", "EIS Toolkit".
- Ledger ids: none (no id relies on a number from them); notes: no id relies on a number from them
- Access: open copy (agency reports)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Identified from its own reference field: three separate works (Ellefsen 2017, USGS Techniques and Methods 7-C14, the MapMark4 user's guide; Duval 2012, USGS Open-File Report 2004-1344, EMINERS; and the EIS Toolkit software). Stage 1 kept them under one key; they were not split because no ledger row distinguishes them.
- Read so far (by the project): compared against, not ported.
- Read in this pass: not fetched (catalogue records only).
- Reference check: the two USGS items were found by search and are the documented references for those programs; the match to what the project means is probable.

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

### Not bibliographic
docs/deeptime-design-decisions.md (Ben's decisions and its log entries); docs/deeptime-implementation-plan.md §8.6, §7.8; docs/research/notes/ore_genesis_modeling.md (a project note; its abstract-level reading of journals is reflected above); MIC rock-tag datapack README; STATUS.md build-log entries; Create ores and Create: Diesel Generators, Create Ore Excavation 1.6.8 (Minecraft mods); Wikipedia pages used for some coal field areas (weak, not a source of record); the Sonnet sub-agent research for coal and for SEDEX tract figures (a process, not a source); named in the earth file only: Hartley & Rice 2005, Yanites & Kesler 2015, Warren 2010, Griffin 2013, Diaz-Rodriguez 2021, Hagemann 2016 (cited "at title level only" in the research note; they do not appear in the sources file or in a ledger row, so they were not resolved).

---

## Deposit formation, coal, petroleum, placers (ORE)

### Lopatin1971 (related; not cited by the code)
- Reference: Lopatin, N. V., 1971, Temperature and geologic time factors in coalification (in Russian): Akademiya Nauk SSSR, Izvestiya, Seriya Geologicheskaya, v. 3, p. 95-106.
- DOI or URL: none found. Cited by the AAPG Wiki article "Petroleum source rocks" (https://wiki.aapg.org/Petroleum_source_rocks).
- Cited as: not cited by the code; the code credits "Lopatin's method" via Waples.
- Ledger ids: ORE-158; notes: none; background for ORE-158
- Access: not found (Russian journal, not online).
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://wiki.aapg.org/Petroleum_source_rocks. File: Lopatin1971_related_not_cited_by_the_code.txt.
- Identification (stage 2, WP-0): Russian journal, not online; the method is documented in Waples1980 (paywalled) and in the USGS reports in the library.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (citation read from the AAPG Wiki page).
- Reference check: n/a. The AAPG Wiki says Lopatin "first described maturation modeling by calculating time-temperature indices for coals, basing his calculations on conversion kinetics of vitrinite" and that Waples "brought the Lopatin method to English-reading geologists". The wiki also lists Waples 1985, Geochemistry in petroleum exploration (IHRDC, Boston, 232 p.).

### Magoon1994
- Reference: Magoon, Leslie B., and Dow, Wallace G., eds., 1994, The petroleum system: from source to trap: AAPG Memoir 60. The "critical moment / charge in order" idea is in the memoir's opening chapter (Magoon and Dow, "The petroleum system", chapter 1); chapter pages not verified in this pass.
- DOI or URL: 10.1306/M60585 (Crossref confirms title, editors, 1994, AAPG).
- Cited as: "Magoon & Dow 1994", DOI 10.1306/m60585.
- Ledger ids: ORE-167
- Access: book (not online).
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: the DOI is the whole memoir, not a chapter; cite the chapter (DOI 10.1306/M60585C1 would be its form, unverified) if the claim is about the critical moment.

### USGS2000b
- Reference: U.S. Geological Survey World Petroleum Assessment 2000 Team, 2000, U.S. Geological Survey World Petroleum Assessment 2000: description and results: U.S. Geological Survey Digital Data Series DDS-60. Chapters used: AR, Analysis of assessment results (Ahlbrandt, Charpentier, Klett, Schmoker, Schenk); MC, Monte Carlo simulation method (Charpentier and Klett); AM, The Seventh Approximation (Schmoker and Klett; also USGS Bulletin 2165); GL, Glossary (Klett, Schmoker, Charpentier, Ahlbrandt, Ulmishek).
- DOI or URL: https://pubs.usgs.gov/dds/dds-060/ (chapter authors as listed on that page).
- Cited as: "DDS-60 chaps. AR, MC, AM, GL"; keys dds60-ar, dds60-mc, dds60-am, dds60-gl.
- Ledger ids: ORE-204, ORE-205
- Access: open access (agency report).
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://pubs.usgs.gov/dds/dds-060/. File: USGS2000b.txt.
- Merged keys: USGS2000
- Read so far (by the project): in full (pages cited).
- Read in this pass: not fetched (index page only; PDFs unreadable). The Bulletin 2165 page https://pubs.usgs.gov/bul/2165/ returned 404.
- Reference check: chapter titles and authors match the project's description; the 1,000-1,500 MMBO and 6,000 cf/bbl values were not checked.

### Averitt1975
- Reference: Averitt, Paul, 1975, Coal resources of the United States, January 1, 1974: U.S. Geological Survey Bulletin 1412, vi, 131 p.
- DOI or URL: https://pubs.usgs.gov/publication/b1412
- Cited as: "Averitt 1975, USGS Bulletin 1412".
- Ledger ids: ORE-118
- Access: open access (agency report).
- Library (the project's source library, not distributed): fetched (PDF, 139 pages, text layer, 347,073 characters). From: https://pubs.usgs.gov/bul/1412/report.pdf. Text file: Averitt1975.txt.
- Read so far (by the project): not stated beyond the research sub-agent.
- Read in this pass: not fetched (citation page only).
- Reference check: matches. The "458,600 mi2 = 13 %" and Mammoth bed figures were not checked.

### Trippi2011
- Reference: Trippi, Michael H., and Tewalt, Susan J., 2011, Geographic information system (GIS) representation of coal-bearing areas in India and Bangladesh: U.S. Geological Survey Open-File Report 2011-1296, iii, 23 p.
- DOI or URL: https://pubs.usgs.gov/publication/ofr20111296
- Cited as: "USGS OFR 2011-1296 Table 1 (CFRI 1993)".
- Ledger ids: ORE-118, ORE-124
- Access: open access (agency report).
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 500 text/html.
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched (shapefiles of coalfields; no counts or areas in it).
- Reference check: authors were not given in the project's citation; now resolved. The "113 coalfields" and "34.7 thousand km2" were not checked.

### BAP2018
- Reference: Bioregional Assessments Programme, 2018 (finalised), Context statement for the Sydney Basin bioregion (Product 1.1), Australian Government.
- DOI or URL: https://www.bioregionalassessments.gov.au/assessments/11-context-statement-sydney-basin-bioregion
- Cited as: "Bioregional Assessments Programme, context statement for the Sydney Basin".
- Ledger ids: ORE-121, ORE-128
- Access: open access (web).
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://www.bioregionalassessments.gov.au/assessments/11-context-statement-sydney-basin-bioregion. File: BAP2018.txt.
- Read so far (by the project): not stated.
- Read in this pass: full text fetched in part (landing page; two targeted questions). See ORE-121 spot check: the page gives different numbers from the ledger.
- Reference check: no authors printed; date from page footer.

### Hatch2002
- Reference: Hatch, Joseph R., and Affolter, Ronald H., 2002, Resource assessment of the Springfield, Herrin, Danville, and Baker coals in the Illinois Basin: U.S. Geological Survey Professional Paper 1625-D.
- DOI or URL: https://doi.org/10.3133/pp1625D
- Cited as: "USGS PP 1625-D" (cumulative coal per field).
- Ledger ids: ORE-128, ORE-129
- Access: open access (agency report).
- Library (the project's source library, not distributed): fetched (PDF, 360 pages, text layer, 531,564 characters). From: https://pubs.usgs.gov/pp/1625d/report.pdf. Text file: Hatch2002.txt.
- Read so far (by the project): "several are the research's own arithmetic".
- Read in this pass: not fetched (USGS page only).
- Reference check: identified from the report number. This is the Illinois Basin volume and is the likely home of any Illinois Basin coal-area figure.

### Ruppert2014
- Reference: Ruppert, Leslie F., and Ryder, Robert T., eds., 2014, Coal and petroleum resources in the Appalachian basin: distribution, geologic framework, and geochemical character: U.S. Geological Survey Professional Paper 1708. Chapter D.1: Coal and coalbed-methane resources in the Appalachian and Black Warrior basins: maps showing the distribution of coal fields, coal beds, and coalbed-methane fields.
- DOI or URL: https://doi.org/10.3133/pp1708 ; chapter https://pubs.usgs.gov/publication/pp1708D.1
- Cited as: "PP 1708-D1".
- Ledger ids: ORE-128, ORE-129
- Access: open access (agency report).
- Library (the project's source library, not distributed): fetched (PDF, 25 pages, text layer, 39,716 characters). From: https://pubs.usgs.gov/pp/1708/d1/pdf/pp1708_d1.pdf. Text file: Ruppert2014.txt.
- Read so far (by the project): research's own arithmetic in part.
- Read in this pass: not fetched (USGS page only; chapter authors not shown).
- Reference check: identified from the report number.

### NMET-Jharia
- Reference: UNIDENTIFIED. "NMET Jharia proposal" (National Mineral Exploration Trust, India). I did not search for it (no search tool); no bibliographic form was available.
- DOI or URL: none.
- Cited as: "NMET Jharia proposal".
- Ledger ids: ORE-128, ORE-129
- Access: not found.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Not resolved. NMET is India's National Mineral Exploration Trust; the proposal is not a published work and Crossref has no record of it.
- Read so far (by the project): not stated.
- Read in this pass: not fetched.
- Reference check: cannot identify.

### SAIMM-coalfields
- Reference: UNIDENTIFIED. "SAIMM J. 105(2)" (Journal of the Southern African Institute of Mining and Metallurgy, vol. 105 no. 2, with "Table I" and "p. 95: 19 coalfields"). A Crossref query on the likely title returned only 2017-2018 articles from the same journal, so I could not tie the 105(2) paper to an author or title.
- DOI or URL: none.
- Cited as: "SAIMM J. 105(2) p. 95", "Table I".
- Ledger ids: ORE-126, ORE-128
- Access: not found.
- Library (the project's source library, not distributed): fetched (PDF, 8 pages, text layer, 38,608 characters). From: https://www.saimm.co.za/Journal/v105n02p095.pdf. Text file: SAIMM-coalfields.txt.
- Identification (stage 2, WP-0): Identified and read: Jeffrey, L. S. (2005), Characterization of the coal resources of South Africa, Journal of the Southern African Institute of Mining and Metallurgy 105(2):95-102. Its text says 'Coal is found in South Africa in 19 coalfields' and has a Table I of seam information per coalfield; open PDF at https://www.saimm.co.za/Journal/v105n02p095.pdf. The year follows from the volume number (105 is 2005); the extracted text of the first page carries no year.
- Read so far (by the project): listed as NOT verified.
- Read in this pass: not fetched.
- Reference check: cannot identify.

### Hampson1999
- Reference: probable identification: Hampson, Gary, Stollhofen, Harald, and Flint, Steve, 1999, A sequence stratigraphic model for the Lower Coal Measures (Upper Carboniferous) of the Ruhr district, north-west Germany: Sedimentology, v. 46, no. 6, p. 1199-1231. The code gives only "Hampson 1999"; this is the only 1999 Hampson paper on coal cycles I found by Crossref.
- DOI or URL: 10.1046/j.1365-3091.1999.00273.x
- Cited as: "Hampson 1999".
- Ledger ids: ORE-139, ORE-140
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): search snippet only.
- Read in this pass: not fetched.
- Reference check: probable, unconfirmed.

### SilesiaMDPI
- Reference: UNIDENTIFIED. "the Silesia MDPI paper" (cyclothem or seam counts for Upper Silesia). I could not search for it.
- DOI or URL: none.
- Cited as: "the Silesia MDPI paper".
- Ledger ids: ORE-139, ORE-140
- Access: not found.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Not resolved. A Crossref search for Upper Silesian seam or cyclothem counts in an MDPI journal found no clear match.
- Read so far (by the project): search snippet only.
- Read in this pass: not fetched.
- Reference check: cannot identify.

### RuhrCounts
- Reference: UNIDENTIFIED. "the Ruhr counts" (measures per seam, 12 m for all coals and about 60 m for economic ones). Possibly the Hampson1999 paper above; not confirmed.
- DOI or URL: none.
- Cited as: "the Ruhr counts".
- Ledger ids: ORE-139, ORE-140
- Access: not found.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Not resolved. Hampson, Stollhofen and Flint 1999 (Sedimentology 46, DOI 10.1046/j.1365-3091.1999.00273.x; key Hampson1999) is the best candidate for a sequence-stratigraphic count of Ruhr coal seams; nothing ties 'the Ruhr counts' to it.
- Read so far (by the project): search snippet only.
- Read in this pass: not fetched.
- Reference check: cannot identify.

### GABowen
- Reference: Geoscience Australia, Bowen Basin (basin summary page, onshore Australia petroleum provinces).
- DOI or URL: https://www.ga.gov.au/scientific-topics/energy/province-sedimentary-basin-geology/petroleum/onshore-australia/bowen-basin
- Cited as: "Geoscience Australia (Bowen Basin about 160,000 km2)".
- Ledger ids: ORE-126
- Access: open access (agency web page).
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://www.ga.gov.au/scientific-topics/energy/province-sedimentary-basin-geology/petroleum/onshore-australia/bowen-basin. File: GABowen.txt.
- Read so far (by the project): listed as NOT verified.
- Read in this pass: full text fetched (the area sentence).
- Reference check: page found and the figure matches (see ORE-126 spot check).

### Ryer1980
- Reference: Ryer, Thomas A., and Langer, Anna W., 1980, Thickness change involved in the peat-to-coal transformation for a bituminous coal of Cretaceous age in central Utah: Journal of Sedimentary Petrology, v. 50, no. 3, p. 987-992 (OpenAlex names the journal Journal of Sedimentary Research, the same SEPM journal).
- DOI or URL: 10.1306/212f7b44-2b24-11d7-8648000102c1865d
- Cited as: "Ryer & Langer 1980".
- Ledger ids: ORE-116
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated; not used by the code.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: matches. The paper is a single Utah case; the abstract says "approximately 11:1".

### Cox1986
- Reference: Cox, Dennis P., and Singer, Donald A., eds., 1986, Mineral deposit models: U.S. Geological Survey Bulletin 1693, xiii, 379 p. (DEP covers the grade-tonnage models.) Descriptive models used: 39a placer Au-PGE (Warren E. Yeend); sedimentary exhalative Zn-Pb; alluvial placer Sn; diamond placers.
- DOI or URL: https://doi.org/10.3133/b1693 ; HTML version https://pubs.usgs.gov/bul/b1693/html/bullfrms.htm ; placer Au-PGE descriptive https://pubs.usgs.gov/bul/b1693/html/bull6945.htm ; placer Au-PGE grade-tonnage .../bull42lh.htm ; sedex descriptive .../bull5nmq.htm ; alluvial placer Sn .../bull3pke.htm ; diamond placers .../bull2xv7.htm.
- Cited as: "USGS Bulletin 1693 (Cox & Singer eds. 1986), models 31a, 39a, 39d, 39e".
- Ledger ids: DEP-054, DEP-055, DEP-059, DEP-114, ORE-220, ORE-221
- Access: open access (HTML pages fetched).
- Library (the project's source library, not distributed): fetched (PDF, 400 pages, text layer). From: https://doi.org/10.3133/b1693. Text file: b1693.txt.
- Read so far (by the project): "figure intercepts and page numbers cited".
- Read in this pass: full text fetched (the five HTML pages above).
- Reference check: matches. Notes: the USGS series page prints "Originally published 1996; Version 1.0 posted 1998" (the print volume is 1986). The HTML pages do not print the model numbers 31a, 39d, 39e in the text I received, so the mapping of those numbers to sedex, alluvial tin and diamond placers rests on the ledger. The p. 262 page number for the gravel density could not be checked.

### Thomas1949
- Reference: Thomas, Marjorie, 1949, A generalization of Poisson's binomial limit for use in ecology: Biometrika, v. 36, no. 1-2, p. 18-25.
- DOI or URL: 10.1093/biomet/36.1-2.18
- Cited as: "Thomas 1949 (Thomas cluster process)".
- Ledger ids: ORE-077
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: matches (author is Marjorie Thomas).

### Garnett2005
- Reference: Garnett, Richard H. T., and Bassett, Neville C., 2005, Placer deposits, in Economic Geology One Hundredth Anniversary Volume 1905-2005: Society of Economic Geologists, p. 813-843 (OpenAlex lists p. 813-843).
- DOI or URL: 10.5382/av100.25
- Cited as: "Garnett & Bassett 2005 (placers)".
- Ledger ids: ORE-220, ORE-221
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not read; the note says full text is needed.
- Read in this pass: not fetched (OpenAlex metadata only).
- Reference check: identified. This is the review that would carry any placer-efficiency or transport-distance figures; it is the first paper to get through a library.

### Lehmann2020
- Reference: Lehmann, Bernd, 2021 (online 2020), Formation of tin ore deposits: a reassessment: Lithos, v. 402-403, article 105756.
- DOI or URL: 10.1016/j.lithos.2020.105756
- Cited as: "Lehmann 2020 (cassiterite within a few km)".
- Ledger ids: ORE-221
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (Crossref metadata only).
- Reference check: the paper is an ore-formation review, not obviously about placer transport; the "within a few km" statement is supported by Cox1986 (alluvial placer Sn, "<8 km"), which I read.

### Basuki2002
- Reference: Basuki, N. I. (Crossref lists this author only; a co-author may exist), 2002, A review of fluid inclusion temperatures and salinities in Mississippi Valley-type Zn-Pb deposits: identifying thresholds for metal transport: Exploration and Mining Geology, v. 11, no. 1-4, p. 1-17.
- DOI or URL: 10.2113/11.1-4.1
- Cited as: not cited.
- Ledger ids: ORE-003; notes: supports ORE-003, 004
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: n/a.

### Gumsley2017
- Reference: Gumsley, Ashley P., Chamberlain, Kevin R., Bleeker, Wouter, Soderlund, Ulf, de Kock, Michiel O., Larsson, Emilie R., and Bekker, Andrey, 2017, Timing and tempo of the Great Oxidation Event: Proceedings of the National Academy of Sciences, v. 114, no. 8, p. 1811-1816.
- DOI or URL: 10.1073/pnas.1608824114 ; open copy https://www.pnas.org/content/pnas/114/8/1811.full.pdf
- Cited as: not cited (it is the reference behind the Wikipedia GOE dates).
- Ledger ids: ORE-014; notes: Earth reference for ORE-014, 016, 017
- Access: open access.
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: n/a. Not bibliographic: Excavated Variants; Create Ore Excavation (COE); Create: Diesel Generators (CDG) and its default thresholds; Create ore stones; vanilla 1.21.1 ore-density measurement (tools/review/deposit-audit.sh); deposit audit tool; project documents (docs/deeptime-design-decisions.md, docs/research/notes/ore_genesis_modeling.md, plan sections, surface_processes_climate.md section 4, build log STATUS.md); Wikipedia pages (Upper Silesia, Indian field areas, Bowen Basin), listed in the project as weak secondary sources.

---

## Moisture, Köppen classes and biomes (MOI, BIO)

### WikipediaKoppen
- Reference: Wikipedia, "Koppen climate classification" (accessed 2026-10-01 through WebFetch; cites Beck et al. 2018 and Kottek et al. 2006 for the criteria).
- DOI or URL: https://en.wikipedia.org/wiki/K%C3%B6ppen_climate_classification
- Cited as: "Source: Wikipedia" (research note surface_processes_climate.md section 5, "Koppen-Geiger rules")
- Ledger ids: BIO-001 to BIO-016
- Access: open access (web page, changes over time)
- Library (the project's source library, not distributed): fetched (HTML page, 44,104 characters of text). From: https://en.wikipedia.org/wiki/K%C3%B6ppen_climate_classification. Text file: WikipediaKoppen.txt.
- Read so far (by the project): a URL in the note
- Read in this pass: full page fetched (summariser): B threshold "Multiply the average annual temperature in degC by 20, then add: 280 if 70% or more of the total precipitation is in the high-sun half of the year; 140 if 30%-70% ...; 0 if less than 30%"; BW below "50%" of the threshold; "(or -3 degC)" for C/D; E "warmest temperature of any month being below 10 degC"; s "driest month of summer receives less than 40 mm" with "at least three times as much precipitation in the wettest month of winter"; w "at least ten times as much rain in the wettest month of summer as in the driest month of winter"; a "above 22 degC"; d "coldest month averaging below -38 degC"; Am "at least 100 - (total annual precipitation(mm)/25)".
- Reference check: the note's text matches this page. Not a scientific source; the 20 x MAT + 280/140/0 form is Koppen's original and is arithmetically the same as Peel's 10 x (2 x MAT + 28/14/0).

### Beck2018
- Reference: Beck, Hylke E.; Zimmermann, Niklaus E.; McVicar, Tim R.; Vergopolan, Noemi; Berg, Alexis; Wood, Eric F. 2018. Present and future Koppen-Geiger climate classification maps at 1-km resolution. Scientific Data 5:180214.
- DOI or URL: (MOI, BIO) https://doi.org/10.1038/sdata.2018.214 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC6207062/ ; data figshare 6396959 (CLM) 10.1038/sdata.2018.214
- Cited as: (MOI, BIO) "Beck et al. 2018, Sci. Data 5, 180214" (koppen.rs COLOURS; design doc; earth/MOI.md validation data) (CLM) "Beck et al. 2018, Sci. Data 5, 180214 (figshare 6396959)"
- Ledger ids: none (a legend palette; validation data); none (validation only); notes: a legend palette; validation data; validation only
- Access: open access (gold) [other sections: open access]
- Library (the project's source library, not distributed): fetched (PDF, 12 pages, text layer, 31,082 characters). From: https://www.nature.com/articles/sdata2018214.pdf. Text file: Beck2018.txt.
- Read so far (by the project): (MOI, BIO) data downloaded and used for Earth comparisons (CLM) used for validation only
- Read in this pass: (MOI, BIO) Table 2 read through PMC (summariser); this table is Peel's Table 1 with "Not (B)" added to the A, C, D and E rows (CLM) dataset record fetched (figshare API): title and file list confirmed
- Reference check: (MOI, BIO) matches (authors, year, volume, article number). The paper's title says 1 km; whether it also provides the 0.5 degree maps the project used was not checked. (CLM) matches.

### Beck2023
- Reference: Beck, Hylke E.; McVicar, Tim R.; Vergopolan, Noemi; Berg, Alexis; Lutsko, Nicholas J.; Dufour, Ambroise; Zeng, Zhenzhong; Jiang, Xin Sheng; van Dijk, Albert I. J. M.; Miralles, Diego G. 2023. High-resolution (1 km) Koppen-Geiger maps for 1901-2099 based on constrained CMIP6 projections. Scientific Data 10:724.
- DOI or URL: (MOI, BIO) https://doi.org/10.1038/s41597-023-02549-6 ; data figshare 21789074 (CLM) 10.1038/s41597-023-02549-6
- Cited as: (MOI, BIO) "Beck et al. 2023, Sci. Data 10, 724 (monthly temperature and precipitation, 1961 to 1990)" (CLM) "Beck et al. 2023, Sci. Data 10, 724 (monthly climatologies, figshare 21789074)"
- Ledger ids: none (validation data for earth/MOI.md); none (validation only); notes: validation data for earth/MOI.md; validation only
- Access: open access (gold) [other sections: open access]
- Library (the project's source library, not distributed): fetched (PDF, 16 pages, text layer, 64,858 characters). From: https://www.nature.com/articles/s41597-023-02549-6.pdf. Text file: Beck2023.txt.
- Read so far (by the project): (MOI, BIO) used as observed data (CLM) used for validation only
- Read in this pass: (MOI, BIO) bibliographic record only (OpenAlex) (CLM) dataset record fetched (figshare API). The record lists `climate_data_*.zip` files, which fits "monthly climatologies"; the 1961-1990 period was not checked.
- Reference check: (MOI, BIO) authors, year, volume, article number match. The paper's title is about Koppen-Geiger maps; whether the figshare 21789074 data are monthly temperature and precipitation was not checked. (CLM) matches.

### SmithBarstad2004
- Reference: Smith, Ronald B. and Barstad, Idar. 2004. A linear theory of orographic precipitation. Journal of the Atmospheric Sciences 61(12):1377-1391.
- DOI or URL: https://doi.org/10.1175/1520-0469(2004)061<1377:ALTOOP>2.0.CO;2
- Cited as: "Smith & Barstad 2004, eq. 49 (J. Atmos. Sci. 61, 1377)"
- Ledger ids: MOI-034 to MOI-036, MOI-040; notes: and MOI-036 to MOI-040 through the software
- Access: only the record was reachable (the AMS page returned 403); open-access status not checked
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 404 text/html;charset=utf-8.
- Read so far (by the project): nothing stated; formula taken from the Python port
- Read in this pass: abstract fetched (OpenAlex): "a linear theory of orographic precipitation that incorporates airflow dynamics, condensed water advection, and downslope evaporation ... vertically integrated steady-state equations solved via Fourier transform"
- Reference check: matches (authors, year, journal, volume, issue, pages). Whether the transfer function is equation 49 could not be read.

### FastscapeLEM
- Reference: Lange, Raphael (copyright 2020) and contributors, fastscape-lem/orographic-precipitation, "a Python framework that implements the Linear Theory of Orographic Precipitation following Smith & Barstad (2004)". MIT licence.
- DOI or URL: https://github.com/fastscape-lem/orographic-precipitation ; commit baf8a0c13855fdef88b4ad995fd59c6e69834e00 (12 August 2024, "Merge pull request #9 from EstebanAce/master - update build system")
- Cited as: "fastscape-lem/orographic-precipitation (MIT, copyright 2020 Raphael Lange; commit baf8a0c) ... the defaults of fastscape_ext.py" (orographic.rs header, params.rs, NOTICE.md)
- Ledger ids: MOI-034 to MOI-040, MOI-043
- Access: open access (software, MIT)
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://github.com/fastscape-lem/orographic-precipitation. File: FastscapeLEM.txt.
- Read so far (by the project): ported (equations and defaults)
- Read in this pass: full text fetched (the source of orographic_precipitation.py and fastscape_ext.py on the master branch; the licence file)
- Reference check: commit exists and the licence reads "MIT License, Copyright (c) 2020 Raphael Lange". The commit is a merge commit on master; the package may have changed since (not diffed).

### HeldSoden2006
- Reference: Held, Isaac M. and Soden, Brian J. 2006. Robust responses of the hydrological cycle to global warming. Journal of Climate 19(21):5686-5699.
- DOI or URL: (MOI, BIO) https://doi.org/10.1175/JCLI3990.1 ; open copy https://www.gfdl.noaa.gov/bibliography/related_files/ih0601.pdf (CLM) 10.1175/JCLI3990.1
- Cited as: (MOI, BIO) "Held & Soden 2006 (about 2 percent per kelvin)" (CLM) "Held & Soden 2006" (journal not named in the code)
- Ledger ids: CLM-079, MOI-042, MOI-063, MOI-064; notes: and the 0.07/K relatives MOI-042, MOI-063
- Access: open copy (GFDL host) [other sections: paywalled (Unpaywall closed)]
- Library (the project's source library, not distributed): fetched (PDF, 14 pages, text layer, 41,683 characters). From: https://www.gfdl.noaa.gov/bibliography/related_files/ih0601.pdf. Text file: HeldSoden2006.txt.
- Merged keys: Held2006
- Read so far (by the project): (MOI, BIO) nothing stated (CLM) not stated
- Read in this pass: (MOI, BIO) full text fetched (decoded PDF text layer: Figure 1 caption, the text on mass flux and Equation 7) (CLM) search result snippet only (citing passages)
- Reference check: (MOI, BIO) matches. The abstract itself has no percent-per-kelvin figure; the numbers are in the text (Part 2). (CLM) matches.

### Thornthwaite1948
- Reference: Thornthwaite, C. W. 1948. An approach toward a rational classification of climate. Geographical Review 38(1):55-94. (The end page 94 is from my recollection; OpenAlex lists only the first page.)
- DOI or URL: https://doi.org/10.2307/210739
- Cited as: "Thornthwaite 1948" (snapshot.rs thornthwaite doc)
- Ledger ids: BIO-038, MOI-065, MOI-066; notes: and BIO-038 through the aridity index
- Access: paywalled (closed per OpenAlex)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 190 chars).
- Substitute: none found.
- Read so far (by the project): nothing stated
- Read in this pass: bibliographic record only; the formula was checked in a secondary implementation (the R package SPEI source)
- Reference check: title, author, year, volume, issue match the record; pages see above.

### Willmott1985
- Reference: Willmott, Cort J.; Rowe, Clinton M.; Mintz, Yale. 1985. Climatology of the terrestrial seasonal water cycle. Journal of Climatology 5(6):589-606.
- DOI or URL: https://doi.org/10.1002/joc.3370050602
- Cited as: "Willmott et al. 1985 (form above 26.5 degC)"
- Ledger ids: MOI-065
- Access: paywalled (closed per OpenAlex)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): nothing stated
- Read in this pass: bibliographic record only
- Reference check: authors, year, volume, issue, pages match the record. That this paper holds the -415.85 + 32.24 T - 0.43 T^2 branch could not be checked.

### BernerKothavala2001
- Reference: Berner, Robert A. and Kothavala, Zavareh. 2001. GEOCARB III: a revised model of atmospheric CO2 over Phanerozoic time. American Journal of Science 301(2):182-204.
- DOI or URL: (MOI, BIO) https://doi.org/10.2475/ajs.301.2.182 ; open PDF https://ajsonline.org/article/61570.pdf (CLM) 10.2475/ajs.301.2.182; PDF https://ajs.scholasticahq.com/article/61570.pdf
- Cited as: (MOI, BIO) "Berner & Kothavala 2001 (runoff rises about 3.8 percent per kelvin, GEOCARB)"; params.rs also cites Berner 1994 for 0.038 (CLM) "Berner & Kothavala 2001, GEOCARB III, Am. J. Sci. 301: 182-204, eq. 2", "GEOCARB's ~3.8 %/K", "Berner's GEOCARB / GEOCARB III (degassing 6.65e12 mol C/yr)"
- Ledger ids: CLM-051, CLM-064, CLM-079
- Access: open access (bronze) [other sections: open access (bronze, publisher PDF)]
- Library (the project's source library, not distributed): fetched (PDF, 23 pages, text layer, 55,189 characters). From: https://ajsonline.org/article/61570.pdf. Text file: BernerKothavala2001.txt.
- Merged keys: Berner2001
- Read so far (by the project): (MOI, BIO) nothing stated (CLM) not stated
- Read in this pass: (MOI, BIO) full text fetched (decoded PDF text layer; the passages on the runoff coefficient RUN) (CLM) full text fetched (equations 1-5 and 10-12, the passages on ACT, RUN, FERT, appendix 1 definitions). Did not find the number 6.65e12 in the readable text.
- Reference check: (MOI, BIO) authors (Berner and Kothavala), year, journal, volume, pages match. The value attributed to this paper does not: see Part 2, MOI-073. (CLM) eq. 2 is the right equation for the temperature and runoff factors. The code's 0.038 is GEOCARB II's RUN, not GEOCARB III's (see Part 2).

### Berner1994
- Reference: Berner, Robert A. 1994. GEOCARB II: a revised model of atmospheric CO2 over Phanerozoic time. American Journal of Science 294(1):56-91.
- DOI or URL: (MOI, BIO) https://doi.org/10.2475/ajs.294.1.56 ; open PDF https://ajs.scholasticahq.com/article/60629.pdf (CLM) 10.2475/ajs.294.1.56; PDF https://ajs.scholasticahq.com/article/60629.pdf (page scan, no text layer)
- Cited as: (MOI, BIO) "Berner 1994" (params.rs runoff_temp_per_k) (CLM) "Berner 1994, GEOCARB II, Am. J. Sci. 294: 56-91"
- Ledger ids: CLM-052, CLM-063, CLM-064, CLM-079
- Access: open access (bronze per OpenAlex) [other sections: open access (bronze) but the PDF is a page scan I could not read]
- Library (the project's source library, not distributed): not fetched. Reason: the download was a web page or an anti-robot interstitial, not the paper.
- Read so far (by the project): (MOI, BIO) nothing stated (CLM) not stated
- Read in this pass: (MOI, BIO) bibliographic record only. Berner & Kothavala 2001 state that GEOCARB II used RUN = 0.038 for all times (results of Manabe and Bryan 1985 for runoff). (CLM) not fetched as readable text (scan). Its constants are quoted in Berner2001 (see below).
- Reference check: (MOI, BIO) authors, year, journal, volume, pages match the record. (CLM) matches.

### WorldbuildingPasta
- Reference: "Worldbuilding Pasta" (blog author; no personal name given on the page I read). An Apple Pie from Scratch, Part VIb: Climate: Biomes and Climate Zones. Blog post, May 2020.
- DOI or URL: https://worldbuildingpasta.blogspot.com/2020/05/an-apple-pie-from-scratch-part-vib.html
- Cited as: "Worldbuilding Pasta, An Apple Pie From Scratch, Part VIb" (belts.rs header, params.rs sst_*, lapse.rs via the research note)
- Ledger ids: GLA-038, MOI-009, MOI-014, MOI-017 to MOI-021, MOI-025, MOI-046, MOI-048; notes: the 4.46 degC/km figure, unused
- Access: open access (blog)
- Library (the project's source library, not distributed): fetched (HTML page, 120,983 characters of text). From: https://worldbuildingpasta.blogspot.com/2020/05/an-apple-pie-from-scratch-part-vib.html. Text file: WorldbuildingPasta.txt.
- Read so far (by the project): a URL in the research note; "no statement whether the page was fetched or recalled"
- Read in this pass: full page fetched (summariser: lapse rate, rain reach, gyre effects, ITCZ and pressure latitudes, relief effects)
- Reference check: a non-peer-reviewed source (a worldbuilding blog). Title on the page: "An Apple Pie from Scratch, Part VIb: Climate: Biomes and Climate Zones". It states its own lapse rate (4.46 degC per km, as used by Clima-Sim) and does not cite measurements for the others.

### ICAO1993
- Reference: International Civil Aviation Organization, 1993, as cited in Wikipedia's "Lapse rate" article (reference 7 there). Full title of the ICAO document not seen by me.
- DOI or URL: none seen
- Cited as: "the standard atmosphere's" 6.5 degC/km (lapse.rs header, params.rs); no document named
- Ledger ids: GLA-038, MOI-053
- Access: not found (the ICAO document itself)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Probably the ICAO Manual of the ICAO Standard Atmosphere (extended to 80 kilometres), Doc 7488, third edition, 1993, the document the 6.5 K/km standard lapse rate is attributed to; the project never named it and it was not seen.
- Read so far (by the project): nothing; no document named
- Read in this pass: not fetched; the Wikipedia page quotes it
- Reference check: UNIDENTIFIED in the code; the ICAO standard atmosphere is the usual source for 6.5 K/km below 11 km.

### Felzenszwalb2012
- Reference: Felzenszwalb, Pedro F. and Huttenlocher, Daniel P. 2012. Distance transforms of sampled functions (the title is from my recollection; the record I got had no title). Theory of Computing 8:415-428.
- DOI or URL: (MOI, BIO) https://doi.org/10.4086/toc.2012.v008a019 ; open PDF https://theoryofcomputing.org/articles/v008a019/v008a019.pdf (CLM) 10.4086/toc.2012.v008a019
- Cited as: (MOI, BIO) "Felzenszwalb and Huttenlocher's distance transform (written from the paper's description)" (bake/biomes.rs distance_cells) (CLM) "Felzenszwalb & Huttenlocher 2012"
- Ledger ids: BIO-063, CLM-110; notes: the algorithm is not a ledger row
- Access: open access (diamond) [other sections: open access (Theory of Computing is a free journal; not checked by me)]
- Library (the project's source library, not distributed): fetched (PDF, 14 pages, text layer, 29,445 characters). From: https://theoryofcomputing.org/articles/v008a019/v008a019.pdf. Text file: Felzenszwalb2012.txt.
- Read so far (by the project): (MOI, BIO) "written from the paper's description" (CLM) "implemented from the paper"
- Read in this pass: (MOI, BIO) bibliographic record only (CLM) not fetched
- Reference check: (MOI, BIO) the project gave no year or venue; the algorithm is not a scientific claim. (CLM) matches.

### Fick2017
- Reference: Fick, Stephen E. and Hijmans, Robert J. 2017. WorldClim 2: new 1-km spatial resolution climate surfaces for global land areas. International Journal of Climatology 37(12):4302-4315.
- DOI or URL: https://doi.org/10.1002/joc.5086
- Cited as: "WorldClim's BIO15" (ClimateSample.java precipitationSeasonality)
- Ledger ids: BIO-095
- Access: closed (OpenAlex); the variable list is at https://www.worldclim.org/data/bioclim.html (open)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): nothing stated
- Read in this pass: WorldClim page fetched: "BIO15 = Precipitation Seasonality (Coefficient of Variation)"; the USGS landing page did not define BIO15 on the page I got
- Reference check: the variable name matches; the exact formula (n or n-1 in the standard deviation) is not on the pages I read.

### ODonnell2012
- Reference: O'Donnell, Michael S. and Ignizio, Drew A. 2012. Bioclimatic predictors for supporting ecological applications in the conterminous United States. U.S. Geological Survey Data Series 691.
- DOI or URL: https://pubs.usgs.gov/publication/ds691
- Cited as: "WorldClim's BIO15" (ClimateSample.java precipitationSeasonality)
- Ledger ids: BIO-095
- Access: open (USGS)
- Library (the project's source library, not distributed): fetched (PDF, 17 pages, text layer, 42,211 characters). From: https://pubs.usgs.gov/ds/691/ds691.pdf. Text file: ODonnell2012.txt.
- Read so far (by the project): nothing stated
- Read in this pass: WorldClim page fetched: "BIO15 = Precipitation Seasonality (Coefficient of Variation)"; the USGS landing page did not define BIO15 on the page I got
- Reference check: the variable name matches; the exact formula (n or n-1 in the standard deviation) is not on the pages I read.

### Minder2010
- Reference: Minder, Justin R.; Mote, Philip W.; Lundquist, Jessica D. 2010. Surface temperature lapse rates over complex terrain: lessons from the Cascade Mountains. Journal of Geophysical Research: Atmospheres 115(D14), D14122.
- DOI or URL: https://doi.org/10.1029/2009JD013493
- Cited as: not cited by the project
- Ledger ids: GLA-038; notes: comparison
- Access: closed (OpenAlex)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): n/a
- Read in this pass: abstracts fetched (OpenAlex)
- Reference check: n/a. (Article number D14122 for Minder is from my recollection, not seen.)

### Rolland2003
- Reference: Rolland, Christian. 2003. Spatial and seasonal variations of air temperature lapse rates in Alpine regions. Journal of Climate 16(7):1032-1046.
- DOI or URL: https://doi.org/10.1175/1520-0442(2003)016<1032:SASVOA>2.0.CO;2
- Cited as: not cited by the project
- Ledger ids: GLA-038; notes: comparison
- Access: not checked
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 404 text/html;charset=utf-8.
- Read so far (by the project): n/a
- Read in this pass: abstracts fetched (OpenAlex)
- Reference check: n/a. (Article number D14122 for Minder is from my recollection, not seen.)

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

---

## Scale, provinces, pole cap and bake (SCL, PRV, POL, BAK)

### USGS-geologic-provinces-map (no author, no year; key: USGSprov)
- Reference: U.S. Geological Survey, Earthquake Hazards Program, crustal-structure maps page, "map of world geologic provinces" (legend: Shield; Platform; Phanerozoic orogen; Foreland or intracratonic basin; Large igneous province; Extended continental crust; Oceanic crust 0-20 Ma, 20-65 Ma, >65 Ma). Author named only as "USGS". It is the map reproduced as `World_geologic_provinces.jpg` (Wikipedia/Commons, uploaded 22 April 2005, the figure in the Wikipedia "Craton" article). The map is a companion to the USGS global seismic-refraction crustal database (the Mooney crust programme); I did not find a named compiler, a dated report, or an OFR number for the map itself. NOT USGS Open-File Report 97-470: that series (World Energy Project, e.g. OFR 97-470F, "Maps showing geology, oil and gas fields, and geologic provinces of the Asia Pacific Region", Steinshouer, Qiang, McCabe & Ryder, 1999, fetched at https://pubs.usgs.gov/publication/ofr97470F) maps petroleum geologic provinces by region and has a different scheme.
- DOI or URL: original page https://earthquake.usgs.gov/data/crust/maps.php (now HTTP 404; Commons records it and an archive.org copy, snapshot 2020-06-03, `web.archive.org/web/20200603051545/https://earthquake.usgs.gov/data/crust/maps.php`, which the tool cannot fetch). Image: https://upload.wikimedia.org/wikipedia/commons/a/a9/World_geologic_provinces.jpg ; file page https://commons.wikimedia.org/wiki/File:World_geologic_provinces.jpg
- Cited as: "USGS map 'Geologic provinces of the world'", "USGS: 20 Ma", "USGS: 65 Ma", "the USGS map's 150 km"
- Ledger ids: BAK-010, PRV-001 to PRV-004, PRV-010, PRV-020 to PRV-022; notes: context for PRV-010 to PRV-020, PRV-022, BAK-010
- Access: open copy (image, public domain) at the URLs above; the explanatory page is gone from usgs.gov and only on the Internet Archive
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://commons.wikimedia.org/wiki/File:World_geologic_provinces.jpg. File: USGS-geologic-provinces-map_no_author_no_year_key_USGSprov.txt.
- Read so far (by the project): not stated ("by eye against the USGS map")
- Read 2026-10-02 (P1-10 agent): the Wikimedia Commons file page (https://commons.wikimedia.org/wiki/File:World_geologic_provinces.jpg), fetched through the web-fetch tool, which summarises the page. It gives: description "Map of world geologic provinces", author USGS, source the dead earthquake.usgs.gov/data/crust/maps.php, uploaded to Wikipedia 22 April 2005, public domain; legend shield, platform, Phanerozoic orogen, foreland/intracratonic basin, large igneous province, extended continental crust, oceanic crust 0-20, 20-65 and >65 Ma; "renders only features approximately 150 km across and greater" (consistent with the seismic refraction data). So the class names, the three oceanic age bands and the 150 km figure are read in a secondary source; the map image and the USGS page were not read, no rule for assigning a cell to a class was found, and the mod's rules (PRV-010 to PRV-020) are its own. A second fetch of the USGS pages (certmapper, earthquake.usgs.gov/data/crust) failed (504, 404) and the Internet Archive is blocked for the tool. Ledger status `abstract-only` for PRV-001 to PRV-004 and PRV-021.
- Read in this pass: search result snippet only / secondary (Commons file description and Wikipedia caption fetched; the image itself and the USGS page not read)
- Reference check: the title in the code ("Geologic provinces of the world") is the Wikipedia caption wording, not a USGS title; the map has no recoverable author or year. Class names and the three oceanic age bands (0-20, 20-65, >65 Ma) match the code. Legend wording differs from the code's classes in two places: the map says "Phanerozoic orogen" and "Foreland or intracratonic basin". The stated 150 km figure is a feature-size limit chosen to match the seismic-refraction data resolution, so the code's "smallest feature the map shows" use is faithful. Class definitions and the age cut-off for continental classes were not found (no legend text beyond names).

### Mallard2016
- Reference: Mallard, C., Coltice, N., Seton, M., Mueller, R. D., Tackley, P. J., 2016. Subduction controls the distribution and fragmentation of Earth's tectonic plates. Nature 535(7610): 140-143. Published 14 June 2016.
- DOI or URL: (SCL, PRV, POL, BAK) 10.1038/nature17992 ; https://www.nature.com/articles/nature17992 ; PubMed 27309815 ; submitted version on HAL hal-01355818 (page blocked for the tool by an Anubis challenge) (PLT) 10.1038/nature17992.
- Cited as: (SCL, PRV, POL, BAK) "Mallard et al. 2016" (plan §8.1), "Mallard et al. 2016, Nature 535:140-143" (plate-collisions doc) (PLT) "Mallard et al. 2016, Nature 535:140-143".
- Ledger ids: PLT-145, SCL-030
- Access: paywalled (abstract only); HAL submitted version exists but was not reachable; the research note links a Caltech course copy (https://web.gps.caltech.edu/~jstock/161/PapersToRead/nature17992-Mallard.pdf, certificate error from this tool) [other sections: paywalled in Nature (a Caltech-hosted copy was used by the research note; not located here).]
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://www.nature.com/articles/nature17992. File: Mallard2016.txt.
- Read so far (by the project): (SCL, PRV, POL, BAK) not stated (PLT) research note Q2 says the PDF was read from web.gps.caltech.edu.
- Read in this pass: (SCL, PRV, POL, BAK) abstract fetched (Europe PMC record) (PLT) metadata only.
- Reference check: (SCL, PRV, POL, BAK) citation matches (authors, venue, volume, pages, year). The abstract supports "seven large plates ... and smaller plates whose areas follow a fractal distribution". The specific split near 10^7.6 km^2 was not in what I could read; see spot check SCL-030. (PLT) correct.

### Swinbank2006
- Reference: Swinbank, R., Purser, R. J., 2006. Fibonacci grids: A novel approach to global modelling. Quarterly Journal of the Royal Meteorological Society 132(619): 1769-1793.
- DOI or URL: 10.1256/qj.05.227
- Cited as: "Swinbank & Purser 2006 ('Fibonacci grids')"
- Ledger ids: SCL-043, SCL-044
- Access: not tested (Wiley/RMetS; likely paywalled)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): "Implemented from the formula; no code was ported." Whether read is not stated
- Read in this pass: not fetched (Crossref metadata only)
- Reference check: exists; authors, title, journal, volume, issue, pages, year match. The code's short title "Fibonacci grids" omits the subtitle.

### Gonzalez2010
- Reference: Gonzalez, A., 2010. Measurement of areas on a sphere using Fibonacci and latitude-longitude lattices. Mathematical Geosciences 42(1): 49-64. (Online first 2009; Crossref lists the year as 2009, the print issue is January 2010.)
- DOI or URL: 10.1007/s11004-009-9257-x ; open copy https://arxiv.org/abs/0912.4540
- Cited as: "Gonzalez 2010 ('Measurement of areas on a sphere using Fibonacci and latitude-longitude lattices')"
- Ledger ids: SCL-043, SCL-044
- Access: open copy at arxiv.org/abs/0912.4540
- Library (the project's source library, not distributed): fetched (PDF, 19 pages, text layer, 32,233 characters). From: https://arxiv.org/pdf/0912.4540. Text file: Gonzalez2010.txt.
- Read so far (by the project): formula implemented; read status not stated
- Read in this pass: abstract fetched (arXiv); arXiv PDF came back binary and unreadable
- Reference check: title and journal match. Author's name is Alvaro Gonzalez (accent on A). Year 2010 is right for the issue; 2009 for the online DOI.

### Keinert2015
- Reference: Keinert, B., Innmann, M., Saenger, M., Stamminger, M., 2015. Spherical Fibonacci mapping. ACM Transactions on Graphics 34(6): article 193 (Crossref pages "1-7"; proceedings of SIGGRAPH Asia 2015).
- DOI or URL: 10.1145/2816795.2818131
- Cited as: "Keinert et al. 2015 ('Spherical Fibonacci mapping')"
- Ledger ids: SCL-043, SCL-044
- Access: paywalled (ACM); an author copy on lgdv.cs.fau.de was tried and returned 404
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): formula implemented; read status not stated
- Read in this pass: not fetched (Crossref metadata only)
- Reference check: title, journal, volume, issue, year match. Article number and page count of the printed version not confirmed.

### Coltice2019
- Reference: Coltice, N., Husson, L., Faccenna, C., Arnould, M., 2019. What drives tectonic plates? Science Advances 5(10): eaax4295.
- DOI or URL: (SCL, PRV, POL, BAK) 10.1126/sciadv.aax4295 ; open access, Europe PMC PMC6821462 (PLT) 10.1126/sciadv.aax4295; https://pmc.ncbi.nlm.nih.gov/articles/PMC6821462/ (open access).
- Cited as: (SCL, PRV, POL, BAK) "Coltice (run used a Pangea-like start)" (no year or title in the report; the research note tectonics_engines.md names "Coltice et al. 2019, What drives tectonic plates?") (PLT) "Coltice, Husson, Faccenna & Arnould 2019, Sci. Adv. 5:eaax4295".
- Ledger ids: PLT-188, SCL-041
- Access: open access
- Library (the project's source library, not distributed): fetched (full-text XML from Europe PMC, 41,153 characters of text). From: epmcxml:PMC6821462. Text file: Coltice2019.txt.
- Read so far (by the project): (SCL, PRV, POL, BAK) not stated for this claim (PLT) link given.
- Read in this pass: (SCL, PRV, POL, BAK) full text fetched (Europe PMC XML, searched for "Pangea/supercontinent" only; title and authors confirmed) (PLT) metadata and one-line abstract only (not fetched in full).
- Reference check: (SCL, PRV, POL, BAK) identification inferred (the report gives only "Coltice"); the other repo Coltice citation (Coltice et al. 2007, Geology 35:391-394) is a different paper on mantle heating. Volume/issue/article number taken from the repo note, not re-confirmed by me. (PLT) correct.

### Gough1981
- Reference: Gough, D. O., 1981. Solar interior structure and luminosity variations. Solar Physics 74(1): 21-34. (Also appears as a chapter of the proceedings "Physics of Solar Variations", Reidel 1981, pp. 21-34, DOI 10.1007/978-94-010-9633-1_4.)
- DOI or URL: (SCL, PRV, POL, BAK) 10.1007/BF00151270 ; scanned PDF https://articles.adsabs.harvard.edu/pdf/1981SoPh...74...21G (CLM) 10.1007/BF00151270
- Cited as: (SCL, PRV, POL, BAK) "Gough 1981", "Solar interior structure and luminosity variations", "Solar Phys. 74: 21-34" (CLM) "Gough 1981"
- Ledger ids: CLM-075, SCL-039
- Access: open copy (ADS page scan, image-only) at the ADS URL; publisher page paywalled [other sections: paywalled (Springer; Unpaywall closed)]
- Library (the project's source library, not distributed): not fetched. Reason: the download was a web page or an anti-robot interstitial, not the paper.
- Read so far (by the project): (SCL, PRV, POL, BAK) not stated; the comment gives the correct title and pages (CLM) not stated
- Read in this pass: (SCL, PRV, POL, BAK) not readable (ADS PDF is JBIG2 page images; the tool cannot read it; Springer redirects to a cookie handshake) (CLM) search result snippet only (citing passages)
- Reference check: (SCL, PRV, POL, BAK) title, journal, volume, pages, year, single author all match Crossref. The luminosity law (equation and number) not confirmed. (CLM) matches. Crossref also lists the same text as a book chapter in "Physics of Solar Variations" (Springer, 1981, doi 10.1007/978-94-010-9633-1_4); same pages.

### Bassin2000 (CRUST 2.0)
- Reference: Bassin, C., Laske, G., Masters, G., 2000. The current limits of resolution for surface wave tomography in North America. EOS Transactions AGU 81, F897 (Fall Meeting abstract). Model: CRUST 2.0, 2 x 2 degree global crustal model, 360 key one-dimensional profiles.
- DOI or URL: https://igppweb.ucsd.edu/~gabi/crust2.html ; data http://igpppublic.ucsd.edu/~gabi/ftp/crust2/ (CNtype2.txt, CNtype2_key.txt, CN-README)
- Cited as: "CRUST 2.0 crust types ... derived by a sub-agent" (the user's brief gives "Bassin, Laske & Masters 2000"; the repo cites only "CRUST 2.0")
- Ledger ids: none (only earth/SCL.md, province shares); notes: only earth/SCL.md, province shares
- Access: open (data files)
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://igppweb.ucsd.edu/~gabi/crust2.html. File: Bassin2000_CRUST_2.0.txt.
- Read so far (by the project): derived by a sub-agent; no method, file or URL
- Read in this pass: data fetched and processed (see Part 2, Earth shares)
- Reference check: the citation the model's own page asks for is a conference abstract, not a model paper. The paper-level reference for the predecessor CRUST 5.1 is Mooney2008 below. The 360 type names describe tectonic settings (Archean, early/mid Proterozoic, late Proterozoic, Platform, Phanerozoic, orogen, continental arc, extended crust, rift, shelf, slope, oceanic ...), so a province share needs a mapping from type name to class; none is published with the model.

### Mooney1998 (CRUST 5.1)
- Reference: Mooney, W. D., Laske, G., Masters, T. G., 1998. CRUST 5.1: A global crustal model at 5 x 5 degrees. Journal of Geophysical Research 103(B1): 727-747.
- DOI or URL: 10.1029/97JB02122
- Cited as: not cited by the code (predecessor of CRUST 2.0)
- Ledger ids: none
- Access: not tested (AGU; likely paywalled)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): not cited
- Read in this pass: not fetched (Crossref metadata only)
- Reference check: listed because CRUST 2.0's own page points to it and no paper describes CRUST 2.0.

### NASA-EarthFactSheet
- Reference: Williams, D. R. (NASA Goddard Space Flight Center, NSSDC), Earth Fact Sheet. Page footer: "Last Updated: 15 November 2024, DRW". (Author initials from the page; full name not on the fetched text.)
- DOI or URL: https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html
- Cited as: none in the repo (supplied here as the citable source for Earth constants)
- Ledger ids: SCL-005, SCL-022 to SCL-027, SCL-029; notes: comparison
- Access: open access
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.29, 2845 chars).
- Read so far (by the project): not cited
- Read in this pass: full text fetched
- Reference check: n/a. The sheet gives no surface area, land/ocean fraction or circumference.

### UNEP1992 (aridity classes)
- Reference: UNEP, 1992. World Atlas of Desertification (Middleton, N., Thomas, D., eds.), United Nations Environment Programme / Arnold, London. Edition and publisher details not verified in this pass (Wikipedia names "UNEP (1992)").
- DOI or URL: none found
- Cited as: "UNEP aridity limits 0.2 and 0.5" (no reference)
- Ledger ids: BAK-020
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (Wikipedia "Aridity index" table)
- Reference check: values 0.05, 0.20, 0.50, 0.65 as P/PET limits confirmed in the Wikipedia table; the atlas itself not read.

### Koerner2004 (treeline)
- Reference: Koerner, C., Paulsen, J., 2004. A world-wide study of high altitude treeline temperatures. Journal of Biogeography 31(5): 713-732.
- DOI or URL: not looked up
- Cited as: not cited (supporting source found via Wikipedia "Tree line")
- Ledger ids: BAK-021
- Access: not tested
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): not cited
- Read in this pass: search result snippet only (Wikipedia: "seasonal mean temperature is approximately 6 degC")
- Reference check: citation as given on Wikipedia.

### Connelly2012 (age of the solar system)
- Reference: Connelly, J. N., Bizzarro, M., Krot, A. N., Nordlund, A., Wielandt, D., Ivanova, M. A., 2012. The absolute chronology and thermal processing of solids in the solar protoplanetary disk. Science 338(6107): 651-655.
- DOI or URL: 10.1126/science.1226919
- Cited as: not cited (supporting source for 4.57 Gyr)
- Ledger ids: SCL-039
- Access: not tested
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): not cited
- Read in this pass: not fetched (Crossref metadata only; the 4.5673 Ga CAI age was seen on Wikipedia "Age of Earth", which does not name this paper in what I fetched, so the link to Connelly is my inference)
- Reference check: n/a

### Not bibliographic
Toroidal World (MIT) docs, Immersive Portals source, TFC Eratosthenes, Civilization V/VI polar caps, RPG-Maker and GameDev forum threads, EU4/Civ IV/Starbound/KSP/Space Engineers/Seed of Andromeda, Sable 2.0.5 API, Distant Horizons API, vanilla Minecraft physics constants and `Biome.coldEnoughToSnow`, the design-decisions file and decisions log, the implementation plan, the research report and notes, build-log entries, Tectonic mod's y640 ceiling.

---

## Plate tectonics (PLT)

### ForsythUyeda1975
- Reference: Forsyth, D., Uyeda, S. (1975). On the relative importance of the driving forces of plate motion. Geophysical Journal of the Royal Astronomical Society 43(1):163-200.
- DOI or URL: 10.1111/j.1365-246X.1975.tb00631.x; https://academic.oup.com/gji/article/43/1/163/586101 (abstract page; PDF returns 403).
- Cited as: "Forsyth & Uyeda 1975, Geophys. J. R. Astron. Soc. 43:163-200".
- Ledger ids: PLT-023, PLT-030, PLT-162, PLT-182, PLT-185, PLT-189, PLT-199
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): fetched in WP-13a (publisher's free PDF, 38 pages, text layer). From: https://academic.oup.com/gji/article-pdf/43/1/163/1576393/43-1-163.pdf. Text file: ForsythUyeda1975.txt.
- Substitute: none found.
- Read so far (by the project): abstract only / secondary pages.
- Read in this pass: abstract fetched (publisher page).
- Reference check: citation correct. The journal title "Geophysical Journal of the Royal Astronomical Society" is the 1975 name (now Geophysical Journal International). The abstract does NOT contain the "3 to 4" speed ratio or any factor of 8 (see Part 2); those must come from the body or from another paper.
- Read in WP-13a (2026-10-02): full text. Slab-attached plates 6-9 cm/yr, others 0-4 cm/yr, terminal velocity (p. 178); "no obvious relation between velocity and area" (p. 177, Fig. 5); slab pull and resistance an order of magnitude above the rest, drag under continents "approximately eight times" that under oceans (p. 189, Tables 5 and 6); continental viscosity "about eight times larger" (p. 194); Table 1 (areas, speeds, trench lengths) used to compute Earth statistics. Ledger: PLT-162, PLT-181 to PLT-189, PLT-197 to PLT-199, CHK-020 to CHK-027, CHK-056. The "3 to 4" is not a sentence of the paper (fix 18).

### Shepard1963
- Reference: Shepard, F. P. (1963). Submarine Geology, 2nd ed. Harper & Row, New York.
- DOI or URL: none found.
- Cited as: "Shepard 1963, Submarine Geology" (shelf break about 130 m).
- Ledger ids: PLT-011
- Access: book (not online).
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated.
- Read in this pass: not fetched.
- Reference check: edition and publisher are from memory of the book, not verified in this pass; the code cites no edition.

### Shoemake1992
- Reference: Shoemake, K. (1992). Uniform random rotations. In: D. Kirk (ed.), Graphics Gems III, Academic Press, pp. 124-132.
- DOI or URL: 10.1016/B978-0-08-050755-2.50036-1.
- Cited as: "Shoemake 1992, Uniform random rotations".
- Ledger ids: PLT-209
- Access: book (not online) chapter; Crossref confirms pages 124-132.
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated.
- Read in this pass: not fetched (metadata only).
- Reference check: correct. Editor name not checked.

### Stern2004
- Reference: Stern, R. J. (2004). Subduction initiation: spontaneous and induced. Earth and Planetary Science Letters 226(3-4):275-292.
- DOI or URL: 10.1016/S0012-821X(04)00498-4 (OpenAlex lists 10.1016/j.epsl.2004.08.007).
- Cited as: "Stern 2004, EPSL 226:275".
- Ledger ids: PLT-062, PLT-067, PLT-207
- Access: paywalled (no abstract available in OpenAlex).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated.
- Read in this pass: metadata only.
- Reference check: correct.

### SternGerya2018
- Reference: Stern, R. J., Gerya, T. (2018). Subduction initiation in nature and models: a review. Tectonophysics 746:173-198.
- DOI or URL: 10.1016/j.tecto.2017.10.014.
- Cited as: "Stern & Gerya 2018, Tectonophysics 746:173".
- Ledger ids: PLT-067
- Access: paywalled (not tested for an open copy).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated.
- Read in this pass: metadata only.
- Reference check: correct.

### Stern2002
- Reference: Stern, R. J. (2002). Subduction zones. Reviews of Geophysics 40(4):1012.
- DOI or URL: 10.1029/2001RG000108.
- Cited as: "Stern 2002" (intra-oceanic arcs take tens of Myr to build 20-35 km of crust); no title or journal in the code.
- Ledger ids: PLT-047
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): not stated.
- Read in this pass: abstract fetched.
- Reference check: identified by the claim (arc building). The abstract says "Earth's 55,000 km of convergent plate margins" but does not give "tens of Myr" or "20-35 km"; those would be in the body (not read).

### Bradley2008
- Reference: Bradley, D. C. (2008). Passive margins through earth history. Earth-Science Reviews 91(1-4):1-26.
- DOI or URL: 10.1016/j.earscirev.2008.08.001.
- Cited as: "Bradley 2008, Earth-Sci. Rev. 91:1-26".
- Ledger ids: PLT-160, PLT-167, PLT-207
- Access: paywalled (abstract not obtainable here).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract or secondary pages.
- Read in this pass: metadata only (OpenAlex abstract field is null; ScienceDirect 403).
- Reference check: correct (author affiliation USGS Anchorage). The figures 105,000 km passive / 53,000 km convergent and 104/181 Myr ages could not be checked.

### Lallemand_EncMarineGeosciences
- Reference: Lallemand, S. entry in Encyclopedia of Marine Geosciences (Springer). Year and entry title not given in the code and not resolved.
- DOI or URL: not found.
- Cited as: "Lallemand, Encycl. Marine Geosciences" (about 45,000 km active continental margins).
- Ledger ids: PLT-160
- Access: not found.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Probably the entry 'Active Continental Margin' by S. Lallemand in the Encyclopedia of Marine Geosciences (Springer, 2014), DOI 10.1007/978-94-007-6644-0_102-2 (Crossref); the entry 'Subduction' (DOI 10.1007/978-94-007-6644-0_103-1) is the other candidate. Not read; paywalled.
- Read so far (by the project): not stated.
- Read in this pass: not fetched.
- Reference check: UNIDENTIFIED as to year and entry title (search budget was gone). Needs one search in stage 2.

### Schellart2008
- Reference: Schellart, W. P., Stegman, D. R., Freeman, J. (2008). Global trench migration velocities and slab migration induced upper mantle volume fluxes: constraints to find an Earth reference frame based on minimizing viscous dissipation. Earth-Science Reviews 88(1-2):118-144.
- DOI or URL: 10.1016/j.earscirev.2008.01.005.
- Cited as: "Schellart et al. 2008, Earth-Sci. Rev. 88:118, Table 1" (28,000-32,000 km "tallied from").
- Ledger ids: PLT-160
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): tallied by an agent from Table 1.
- Read in this pass: metadata only.
- Reference check: correct journal and page. The paper is about trench migration; that it has a table of margin lengths is plausible but not checked.

### MercierdeLepinay2016
- Reference: Mercier de Lépinay, M., Loncke, L., Basile, C., Roest, W. R., Patriat, M., Maillard, A., De Clarens, P. (2016). Transform continental margins - Part 2: a worldwide review. Tectonophysics 693:96-115.
- DOI or URL: 10.1016/j.tecto.2016.05.038.
- Cited as: "Mercier de Lepinay et al. 2016, Tectonophysics 693" (convergent about 48 percent of margin length).
- Ledger ids: PLT-160
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 1010 chars).
- Substitute: none found.
- Read so far (by the project): abstract or secondary.
- Read in this pass: metadata only.
- Reference check: identified by title search. It is a paper on TRANSFORM margins; a global margin-type length inventory may be inside it but the paper's focus is not convergent margins. Flag for stage 2.

### Bird2003
- Reference: Bird, P. (2003). An updated digital model of plate boundaries. Geochemistry, Geophysics, Geosystems 4(3):1027.
- DOI or URL: (PLT) 10.1029/2001GC000252 (open access at AGU/Wiley; blocked to WebFetch with 403). (CLM) 10.1029/2001GC000252
- Cited as: (PLT) "Bird 2003, G3 4(3):1027" (52 plates; Pacific 103.3 million km2; seven largest about 92 percent); editor hint "Earth: about 14". (CLM) "Bird 2003" (no title or journal in the code; I assigned this paper because the claim concerns subduction-zone length and rate)
- Ledger ids: CLM-054, PLT-143, PLT-145, PLT-203
- Access: open access (G3 is open), not fetched as full text here. [other sections: paywalled (Unpaywall closed)]
- Library (the project's source library, not distributed): fetched in WP-13a (full text PDF, author's site). From: http://peterbird.name/publications/2003_PB2002/2001GC000252.pdf. Text file: Bird2003.txt.
- Read so far (by the project): (PLT) "confirmed from abstract or secondary pages"; areas from Wikipedia. (CLM) not stated; no title or journal given
- Read in this pass: (PLT) abstract fetched (OpenAlex reconstruction). (CLM) search result snippet only (the citing passages returned were about regional plate boundaries and did not mention 51,000 km or 62 mm/yr)
- Reference check: (PLT) correct. The abstract gives "14 large plates ... plus 38 small plates ... for a total of 52 plates". The area numbers and "seven largest 92 percent" are not in the abstract. (CLM) IDENTIFICATION UNCERTAIN. Bird 2003 is the PB2002 plate-boundary model; a total subduction length of 51,000 km at 62 mm/yr is plausible from it but I could not confirm it is in that paper.
- Read in WP-13a (2026-10-02): Table 1 (areas in steradians): Pacific 2.57685 sr, 20.5 % of the sphere; seven plates above 5 %, together 81 % (not 92 %); Table 3 re-read: subduction 51,310 of 260,487 km (19.7 %). PLT-143, PLT-145, PLT-213, CHK-001, CHK-003.

### Morra2013
- Reference: Morra, G., Seton, M., Quevedo, L., Müller, R. D. (2013). Organization of the tectonic plates in the last 200 Myr. Earth and Planetary Science Letters 373:93-101.
- DOI or URL: 10.1016/j.epsl.2013.04.020.
- Cited as: "Morra et al. 2013, EPSL 373:93-101".
- Ledger ids: PLT-145
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract or secondary.
- Read in this pass: metadata only.
- Reference check: correct.

### Lonsdale2005
- Reference: Lonsdale, P. (2005). Creation of the Cocos and Nazca plates by fission of the Farallon plate. Tectonophysics 404(3-4):237-264.
- DOI or URL: 10.1016/j.tecto.2005.05.011.
- Cited as: "Lonsdale 2005, Tectonophysics 404:237-264".
- Ledger ids: PLT-146
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated.
- Read in this pass: metadata only.
- Reference check: correct; the design cites it for Farallon and not for lifetime statistics (the paper is about a plate fission event).

### Wilson1966
- Reference: Wilson, J. T. (1966). Did the Atlantic close and then re-open? Nature 211(5050):676-681.
- DOI or URL: 10.1038/211676a0.
- Cited as: "J. T. Wilson 1966, Nature 211:676-681".
- Ledger ids: PLT-148
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract or secondary.
- Read in this pass: metadata only.
- Reference check: correct.

### Nance2022
- Reference: Nance, R. D. (2022). The supercontinent cycle and Earth's long-term climate. Annals of the New York Academy of Sciences 1515(1):33-49.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC9796656/ (open access).
- Cited as: "Nance 2022, PMC9796656".
- Ledger ids: PLT-148, PLT-201
- Access: open access.
- Library (the project's source library, not distributed): fetched (HTML page, 77,792 characters of text). From: https://pmc.ncbi.nlm.nih.gov/articles/PMC9796656/. Text file: Nance2022.txt.
- Read so far (by the project): abstract-only status in the ledger.
- Read in this pass: full text fetched (targeted at cycle length and supercontinent dates).
- Reference check: title/author/journal not named in the code; now identified.
- Read in WP-13a (2026-10-02): full text again: "intervals of ca. 500 m.y."; Pangea c. 325-200 Ma, Rodinia c. 950-800 Ma, Pannotia c. 620-580 Ma. PLT-201, CHK-036.

### Mitchell2021
- Reference: Mitchell, R. N., Zhang, N., Salminen, J., Liu, Y., Spencer, C. J., Steinberger, B., Murphy, J. B., Li, Z.-X. (2021). The supercontinent cycle. Nature Reviews Earth & Environment 2(5):358-374.
- DOI or URL: 10.1038/s43017-021-00160-0.
- Cited as: "Mitchell et al. 2021, Nat. Rev. Earth Environ. 2:358-374".
- Ledger ids: PLT-148, PLT-201
- Access: paywalled.
- Library (the project's source library, not distributed): fetched in WP-13a (accepted manuscript, Curtin University; the landing page is kept as Mitchell2021_landing.txt). From: https://curate.curtin.edu.au/ndownloader/files/62792113. Text file: Mitchell2021.txt.
- Read so far (by the project): abstract-only status.
- Read in this pass: metadata only.
- Reference check: correct.
- Read in WP-13a (2026-10-02): full text (accepted manuscript): a cycle "often considered to last 400-800 million years", ~600 Myr dominant; tenures "between 100 and 300 Myr". PLT-201, CHK-035, CHK-036.

### Copley2010
- Reference: Copley, A., Avouac, J.-P., Royer, J.-Y. (2010). India-Asia collision and the Cenozoic slowdown of the Indian plate: implications for the forces driving plate motions. Journal of Geophysical Research 115(B3):B03410.
- DOI or URL: 10.1029/2009JB006634.
- Cited as: "Copley, Avouac & Royer 2010, JGR 115:B03410".
- Ledger ids: PLT-154, PLT-177, PLT-200
- Access: paywalled (abstract only; Wiley).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): not stated beyond section 3 remark.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: correct, and the abstract also contains the force magnitudes used elsewhere in the designs (Part 2).
- Read in WP-13a (2026-10-02): abstract again: convergence "dropping from ~15 to ~4 cm/yr" between 50 and 35 Ma; Himalaya-Tibet resistance ~5-6 x 10^12 N/m; ridge push ~2.5 x 10^12 N/m; crustal thickening "the dominant cause". The Cambridge and HAL copies were not reachable or hold no file. PLT-154, PLT-177, PLT-187, PLT-200, CHK-037.

### DaviesvonBlanckenburg1995
- Reference: Davies, J. H., von Blanckenburg, F. (1995). Slab breakoff: a model of lithosphere detachment and its test in the magmatism and deformation of collisional orogens. Earth and Planetary Science Letters 129(1-4):85-102.
- DOI or URL: 10.1016/0012-821X(94)00237-S.
- Cited as: "Davies & von Blanckenburg 1995 (EPSL 129:85-102)".
- Ledger ids: PLT-179, PLT-196
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): contextual.
- Read in this pass: metadata only.
- Reference check: correct.

### RoyerGordon1997
- Reference: Royer, J.-Y., Gordon, R. G. (1997). The motion and boundary between the Capricorn and Australian plates. Science 277(5330):1268-1274.
- DOI or URL: 10.1126/science.277.5330.1268.
- Cited as: "Royer & Gordon 1997".
- Ledger ids: PLT-179, PLT-196
- Access: paywalled (abstract).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): contextual.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct. Its topic (diffuse plate boundaries in the Indo-Australian plate) is an odd match for "supercontinent heating" context; the design may mean it as the example of plate splitting. Flag for the stage 2 reader.

### Gurnis1988
- Reference: Gurnis, M. (1988). Large-scale mantle convection and the aggregation and dispersal of supercontinents. Nature 332(6166):695-699.
- DOI or URL: 10.1038/332695a0.
- Cited as: "Gurnis 1988 (Nature 332:695-699)".
- Ledger ids: PLT-179, PLT-196
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): contextual.
- Read in this pass: metadata only.
- Reference check: correct.

### Coltice2007
- Reference: Coltice, N., Phillips, B. R., Bertrand, H., Ricard, Y., Rey, P. (2007). Global warming of the mantle at the origin of flood basalts over supercontinents. Geology 35(5):391-394.
- DOI or URL: 10.1130/G23240A.1.
- Cited as: "Coltice et al. 2007 (Geology 35:391-394)".
- Ledger ids: PLT-179
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract-only status.
- Read in this pass: metadata only.
- Reference check: correct.

### HeronLowman2011
- Reference: Heron, P. J., Lowman, J. P. (2011). The effects of supercontinent size and thermal insulation on the formation of mantle plumes. Tectonophysics 510(1-2):28-38.
- DOI or URL: 10.1016/j.tecto.2011.07.002.
- Cited as: "Heron & Lowman 2011".
- Ledger ids: PLT-179
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract-only status.
- Read in this pass: metadata only.
- Reference check: identified by title search; correct.

### Lenardic2011
- Reference: Lenardic, A., Moresi, L., Jellinek, A. M., O'Neill, C. J., Cooper, C. M., Lee, C. T. (2011). Continents, supercontinents, mantle thermal mixing, and mantle thermal isolation: theory, numerical simulations, and laboratory experiments. Geochemistry, Geophysics, Geosystems 12(10):Q10016 (article number from memory of the journal; the Crossref record gave only the issue).
- DOI or URL: 10.1029/2011GC003663.
- Cited as: "Lenardic et al. 2011" (no venue in the code).
- Ledger ids: PLT-179
- Access: paywalled (not tested).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): abstract-only status.
- Read in this pass: metadata only.
- Reference check: identified by author list and year; there is also Lenardic et al. 2005 (EPSL 234:317) on continental insulation, which is the other candidate.

### BercoviciLong2014
- Reference: Bercovici, D., Long, M. D. (2014). Slab rollback instability and supercontinent dispersal. Geophysical Research Letters 41(19):6659-6666.
- DOI or URL: 10.1002/2014GL061251.
- Cited as: "Bercovici & Long 2014 (GRL 41:6659-6666)".
- Ledger ids: PLT-179, PLT-196
- Access: paywalled (abstract; AGU).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): abstract-only status.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct; title given here for the first time (the code gives none).

### Nocquet2016
- Reference: Nocquet, J.-M., Sue, C., Walpersdorf, A., Tran, T., Lenôtre, N., Vernant, P., Cushing, M., Jouanne, F., Masson, F., Baize, S., Chéry, J., van der Beek, P. A. (2016). Present-day uplift of the western Alps. Scientific Reports 6:28404.
- DOI or URL: 10.1038/srep28404 (open access).
- Cited as: "Nocquet et al. 2016, Sci. Rep. 6:28404".
- Ledger ids: PLT-180
- Access: open access.
- Library (the project's source library, not distributed): fetched (HTML page, 27,074 characters of text). From: https://doi.org/10.1038/srep28404. Text file: Nocquet2016.txt.
- Read so far (by the project): abstract-only status.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct.
- Read in WP-13a (2026-10-02): full text: uplift "reaching ~2.5 mm/yr in the northwestern Alps" (abstract); nothing on the Himalaya. PLT-180.

### Tremblay2015
- Reference: Tremblay, M. M., Fox, M., Schmidt, J. L., Tripathy-Lang, A., Wielicki, M. M., Harrison, T. M., Zeitler, P. K., Shuster, D. L. (2015). Erosion in southern Tibet shut down at ~10 Ma due to enhanced rock uplift within the Himalaya. Proceedings of the National Academy of Sciences 112(39):12030-12035.
- DOI or URL: 10.1073/pnas.1515652112 (PNAS, open after 6 months).
- Cited as: "Tremblay et al. 2015, PNAS 112:12030".
- Ledger ids: PLT-180
- Access: open access (PNAS).
- Library (the project's source library, not distributed): fetched (HTML page, 43,364 characters of text). From: https://www.ncbi.nlm.nih.gov/pmc/articles/4593086. Text file: Tremblay2015.txt.
- Read so far (by the project): abstract-only status.
- Read in this pass: abstract fetched.
- Reference check: identified by volume and first page; the paper is about erosion rates falling, while the design cites it for "exhumation above 1 mm/yr at 17-12 Ma" (not verifiable from the abstract).
- Read in WP-13a (2026-10-02): full text: "rapid exhumation exceeding 1 km/Ma from 17-16 to 12-11 Ma" in southern Tibet. PLT-180.

### Schellart2004
- Reference: Schellart, W. P. (2004). Quantifying the net slab pull force as a driving mechanism for plate tectonics. Geophysical Research Letters 31(7):L07611.
- DOI or URL: 10.1029/2004GL019528.
- Cited as: "Schellart 2004, GRL 31:L07611" ("abstract read in full").
- Ledger ids: PLT-182
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): abstract in full.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: correct.
- Read in WP-13a (2026-10-02): abstract again (OpenAlex): net slab pull up to ~8-12 % of the slab's buoyancy, "up to twice as large as the ridge push force". Full text not obtainable (Wiley refuses scripts; the ANU record holds only a deposit record). PLT-182.

### ConradLB2002
- Reference: Conrad, C. P., Lithgow-Bertelloni, C. (2002). How mantle slabs drive plate tectonics. Science 298(5591):207-209.
- DOI or URL: 10.1126/science.1074161.
- Cited as: "Conrad & Lithgow-Bertelloni 2002, Science 298:207-209".
- Ledger ids: PLT-186, PLT-199
- Access: paywalled (abstract).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched.
- Reference check: correct.
- Read in WP-13a (2026-10-02): abstract again: upper-mantle slab pull "about half of the total driving force". No open copy found (UCL Discovery records 404, the lab page has no file). PLT-186.

### ConradLB2004
- Reference: Conrad, C. P., Lithgow-Bertelloni, C. (2004). The temporal evolution of plate driving forces: importance of "slab suction" versus "slab pull" during the Cenozoic. Journal of Geophysical Research 109(B10):B10407.
- DOI or URL: 10.1029/2004JB002991.
- Cited as: "Conrad & Lithgow-Bertelloni 2004, JGR 109:B10407".
- Ledger ids: PLT-186, PLT-199
- Access: paywalled (abstract).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched (OpenAlex; reconstruction partly garbled, key sentence legible).
- Reference check: correct. This abstract, not Forsyth & Uyeda, contains the "about 4 times faster" statement (Part 2).
- Read in WP-13a (2026-10-02): abstract again: subducting plates "currently move ~4 times faster"; suction and pull about 40 and 60 % of the forces, suction ~30 % with a low-viscosity asthenosphere. Full text not obtainable (Wiley; Deep Blue refuses scripts). PLT-162, PLT-186, CHK-025, CHK-056.

### ConradLB2006
- Reference: Conrad, C. P., Lithgow-Bertelloni, C. (2006). Influence of continental roots and asthenosphere on plate-mantle coupling. Geophysical Research Letters 33(5):L05312.
- DOI or URL: 10.1029/2005GL025621 (bronze open access at Wiley; PDF returns 403 to scripts).
- Cited as: "Conrad & Lithgow-Bertelloni 2006, GRL 33:L05312".
- Ledger ids: PLT-188
- Access: open copy at Wiley (bronze), abstract read here.
- Library (the project's source library, not distributed): fetched in WP-13a (4 pages, text layer). From: https://discovery.ucl.ac.uk/id/eprint/146995/1/2005GL025621.pdf. Text file: ConradLB2006.txt.
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched.
- Reference check: correct.
- Read in WP-13a (2026-10-02): full text: continental roots "increase traction magnitudes by a factor of 2-5 compared to 100 km thick lithosphere" (abstract and section 3). PLT-188 verified.

### LithgowBertelloniRichards1998
- Reference: Lithgow-Bertelloni, C., Richards, M. A. (1998). The dynamics of Cenozoic and Mesozoic plate motions. Reviews of Geophysics 36(1):27-78.
- DOI or URL: 10.1029/97RG02282.
- Cited as: "Lithgow-Bertelloni & Richards 1998, Rev. Geophys. 36:27-78".
- Ledger ids: PLT-187
- Access: paywalled (abstract).
- Library (the project's source library, not distributed): fetched in WP-13a (52 pages, text layer). From: https://discovery.ucl.ac.uk/id/eprint/125253/1/97RG02282.pdf. Text file: LithgowBertelloniRichards1998.txt.
- Substitute: none found.
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched (Crossref JSON).
- Reference check: correct. (The 1995 GRL paper 22:1317-1320, same authors, has the same ">90 % slabs, <10 % lithospheric thickening" statement.)
- Read in WP-13a (2026-10-02): full text: slabs "in excess of 90 %" of the driving forces, lithospheric thickening "of the order of 10 %" (abstract and conclusions). PLT-187.

### Carlson1983
- Reference: Carlson, R. L., Hilde, T. W. C., Uyeda, S. (1983). The driving mechanism of plate tectonics: relation to age of the lithosphere at trenches. Geophysical Research Letters 10(4):297-300.
- DOI or URL: 10.1029/GL010i004p00297.
- Cited as: "Carlson, Hilde & Uyeda 1983, GRL 10:297-300".
- Ledger ids: PLT-182
- Access: paywalled (abstract).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct.
- Read in WP-13a (2026-10-02): abstract again (OpenAlex): speed against age at the trench, r^2 0.98 over 15 trenches; ridge push "just sufficient to overcome drag". Full text not obtainable. PLT-182.

### ParsonsRichter1980
- Reference: Parsons, B., Richter, F. M. (1980). A relation between the driving force and geoid anomaly associated with mid-ocean ridges. Earth and Planetary Science Letters 51(2):445-450.
- DOI or URL: 10.1016/0012-821X(80)90223-X.
- Cited as: "Parsons & Richter 1980, EPSL 51:445-450".
- Ledger ids: PLT-187
- Access: paywalled (no abstract in OpenAlex).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from abstract.
- Read in this pass: metadata only.
- Reference check: title now identified; correct.
- Read in WP-13a (2026-10-02): not obtainable (no abstract, no open copy). Its 3 x 10^12 N/m stays unread; Copley et al. 2010's abstract gives ridge push ~2.5 x 10^12 N/m (PLT-187).

### vanHunenAllen2011
- Reference: van Hunen, J., Allen, M. B. (2011). Continental collision and slab break-off: a comparison of 3-D numerical models with observations. Earth and Planetary Science Letters 302(1-2):27-37.
- DOI or URL: 10.1016/j.epsl.2010.11.035.
- Cited as: "van Hunen & Allen 2011, EPSL 302:27-37" (10-25 Myr).
- Ledger ids: PLT-184
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): from abstract.
- Read in this pass: metadata only.
- Reference check: correct.
- Read in WP-13a (2026-10-02): text not obtainable (Durham's repository refuses scripts). The break-off times are read in the same group's EGU 2011 abstract (vanHunen2011EGU, below). PLT-184.

### Zahirovic2015
- Reference: Zahirovic, S., Müller, R. D., Seton, M., Flament, N. (2015). Tectonic speed limits from plate kinematic reconstructions. Earth and Planetary Science Letters 418:40-52.
- DOI or URL: 10.1016/j.epsl.2015.02.037; the EarthByte summary page https://www.earthbyte.org/tectonic-speed-limits-from-plate-kinematic-reconstructions/ returned HTTP 500 / timeouts in this pass.
- Cited as: "Zahirovic et al. 2015, EPSL 418:40-52".
- Ledger ids: PLT-025, PLT-198
- Access: paywalled (closed; OpenAlex has no open URL and no abstract).
- Library (the project's source library, not distributed): fetched in WP-13a (accepted manuscript, 39 pages, text layer; the landing page is kept as Zahirovic2015_landing.txt). From: https://ro.uow.edu.au/ndownloader/files/50501994. Text file: Zahirovic2015.txt.
- Read so far (by the project): link given.
- Read in this pass: metadata only (Crossref). Speed numbers could not be read.
- Reference check: correct.
- Read in WP-13a (2026-10-02): full text (accepted manuscript): median RMS plate speed ~4 cm/yr over 200 Myr, 4.2 over 3,952 samples; global RMS ~5-10 cm/yr (section 3.2); category medians: no subduction 2.6, over 25 % subducting 7.9, over 50 % continental 2.8, under 25 % continental 8.1 cm/yr (section 3.5); oceanic plates "2-3 times faster" (abstract); present RMS Antarctica 1.1, Africa 2.0, Pacific 7.9 cm/yr. PLT-181, PLT-190, PLT-197 to PLT-199, CHK-020 to CHK-025.

### DeCellesGiles1996
- Reference: DeCelles, P. G., Giles, K. A. (1996). Foreland basin systems. Basin Research 8(2):105-123.
- DOI or URL: (PLT) 10.1046/j.1365-2117.1996.01491.x. (ISO) 10.1046/j.1365-2117.1996.01491.x; full text https://pages.uoregon.edu/rdorsey/BasinAnalysis/BasinPapers/DeCelles%20&%20Giles%201996.pdf
- Cited as: (PLT) "DeCelles & Giles 1996, Basin Research 8:105-123" ("via a secondary source"). (ISO) docs/design/flexure.md section 11
- Ledger ids: PLT-194; none (the acceptance figures of docs/design/plate-force-balance.md); notes: the acceptance figures of docs/design/plate-force-balance.md
- Access: open copy. [other sections: paywalled (abstract visible).]
- Library (the project's source library, not distributed): fetched (PDF, 19 pages, text layer, 79,621 characters). From: https://pages.uoregon.edu/rdorsey/BasinAnalysis/BasinPapers/DeCelles%20&%20Giles%201996.pdf. Text file: DeCellesGiles1996.txt.
- Merged keys: DeCellesGiles1996_flexure
- Read so far (by the project): (PLT) via a secondary source. (ISO) full text by a Sonnet sub-agent of the flexure agent on 2026-10-02: "Foredeep depozones are typically 100-300 km wide and 2-8 km thick"; forebulges 60-470 km wide and a few tens to a few hundred metres high, as a flexural prediction.
- Read in this pass: (PLT) abstract fetched (Crossref, first part). (ISO) not part of stage 1; see "Read so far".
- Reference check: (PLT) correct. The visible part of the abstract defines depozones; the 100-300 km and 2-8 km figures were not seen. (ISO) correct; this settles what stage 1 could not see.
- Read in WP-13a (2026-10-02): full text again: foredeeps "typically 100-300 km wide and 2-8 km thick"; forebulges "of the order of 60-470 km wide". PLT-194, CHK-039, CHK-040.

### TurcotteSchubert_Geodynamics
- Reference: Turcotte, D. L., Schubert, G. Geodynamics. Cambridge University Press (editions 1982, 2002, 2014).
- DOI or URL: none; Internet Archive copy (geodynamics0000turc) is lending-only (HTTP 401).
- Cited as: "Turcotte & Schubert, Geodynamics" (flexure; gross slab pull "not confirmed").
- Ledger ids: ISO-040 to ISO-042, ISO-067, PLT-183, PLT-193
- Access: book (not online).
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not read; gross slab pull "not confirmed in Turcotte & Schubert itself". For the flexure feature (2026-10-02) a sub-agent read chapter 3 as a PDF (https://sseh.uchicago.edu/doc/Geodynamics_Turcotte_Schubert_ch_3.pdf: E = 70 GPa and ν = 0.25 in the periodic-loading example, 100 GPa elsewhere) and the flexure formulas in lecture notes on the chapter (https://topex.ucsd.edu/geodynamics/12flexure.pdf); the edition of that PDF was not recorded.
- Read in this pass: not fetched; a 2024 paper that cites it was read (see Li2024).
- Reference check: edition not stated anywhere in the code or designs; the numbers depend on the edition and chapter.
- Read in WP-13a (2026-10-02): not read. Its slab-pull law is read as Faccenna et al. 2012's eq. 1, which follows Turcotte & Schubert 1982 (PLT-182, PLT-183). A course directory's copy of a chapter was found and not used: it is not an author's or publisher's copy.

### Buck2004
- Reference: Buck, W. R. (2004). Consequences of asthenospheric variability on continental rifting. In: Karner, G. D., Taylor, B., Driscoll, N. W., Kohlstedt, D. L. (eds.), Rheology and Deformation of the Lithosphere at Continental Margins, Columbia University Press, pp. 1-30.
- DOI or URL: 10.7312/karn12738-002.
- Cited as: "Buck 2004 (in Karner et al., eds., ..., 1-30)".
- Ledger ids: PLT-196
- Access: book chapter (not online).
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): from abstracts.
- Read in this pass: metadata only.
- Reference check: correct; title identified here.

### Bialas2010
- Reference: Bialas, R. W., Buck, W. R., Qin, R. (2010). How much magma is required to rift a continent? Earth and Planetary Science Letters 292(1-2):68-78.
- DOI or URL: 10.1016/j.epsl.2010.01.021.
- Cited as: "Bialas, Buck & Qin 2010 (EPSL, volume not confirmed)".
- Ledger ids: PLT-196
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from abstracts.
- Read in this pass: metadata only.
- Reference check: volume now resolved (292:68-78).

### DalZilio2018
- Reference: Dal Zilio, L., Faccenda, M., Capitanio, F. (2018). The role of deep subduction in supercontinent breakup. Tectonophysics 746:312-324.
- DOI or URL: 10.1016/j.tecto.2017.03.006.
- Cited as: "Dal Zilio, Faccenda & Capitanio 2018 (Tectonophysics 746:312-324)".
- Ledger ids: PLT-196
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from abstracts.
- Read in this pass: metadata only.
- Reference check: correct.

### Zhang2018
- Reference: Zhang, N., Dang, Z., Huang, C., Li, Z.-X. (2018). The dominant driving force for supercontinent breakup: plume push or subduction retreat? Geoscience Frontiers 9(4):997-1007.
- DOI or URL: 10.1016/j.gsf.2018.01.010.
- Cited as: "Zhang et al. 2018 (Geosci. Front. 9:997-1007)".
- Ledger ids: PLT-196
- Access: open access (Geoscience Frontiers is open; not fetched).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): from abstracts.
- Read in this pass: metadata only.
- Reference check: correct.

### Huang2019
- Reference: Huang, C., Zhang, N., Li, Z.-X., Ding, M., Dang, Z., Pourteau, A., Zhong, S. (2019). Modeling the inception of supercontinent breakup: stress state and the importance of orogens. Geochemistry, Geophysics, Geosystems 20(11):4830-4848.
- DOI or URL: 10.1029/2019GC008538; https://par.nsf.gov/biblio/10127684 (open).
- Cited as: "Huang et al. 2019 (G3 20:4830-4848)".
- Ledger ids: PLT-196
- Access: open copy at par.nsf.gov.
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://par.nsf.gov/biblio/10127684. File: Huang2019.txt.
- Read so far (by the project): from abstract.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct. The abstract does give "extensional stress of 20-50 MPa in its interior" for a homogeneous supercontinent, matching the design.
- Read in WP-13a (2026-10-02): no full text (the Curtin record has no file; Wiley refuses scripts). PLT-196 stays abstract-only.

### DavisSuppeDahlen1983
- Reference: Davis, D., Suppe, J., Dahlen, F. A. (1983). Mechanics of fold-and-thrust belts and accretionary wedges. Journal of Geophysical Research 88(B2):1153-1172.
- DOI or URL: 10.1029/JB088iB02p01153.
- Cited as: "Davis, Suppe & Dahlen 1983, JGR 88:1153-1172" (from memory).
- Ledger ids: PLT-192
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory, not checked.
- Read in this pass: abstract fetched (OpenAlex).
- Reference check: the citation is correct (authors, year, volume, pages; issue B2). The paper's title was missing in the design. The abstract concerns critical taper and gives no taper angle in degrees.
- Read in WP-13a (2026-10-02): abstract (OpenAlex): a critical wedge "slides stably, continuing to grow at constant taper"; Taiwan's surface slope and basal dip are in the body, not read (Wiley). Dahlen 1990 (Annu. Rev. Earth Planet. Sci. 18:55-99) is closed too. The Himalayan figures come from DeCelles & Carrapa 2021 instead. PLT-192.

### Christensen_Mooney1995
- Reference: Christensen, N. I., Mooney, W. D. (1995). Seismic velocity structure and composition of the continental crust: a global view. Journal of Geophysical Research 100(B6):9761-9788.
- DOI or URL: 10.1029/95JB00259.
- Cited as: not cited by the project; used here as a source for continental crust thickness and density.
- Ledger ids: (none; supports 121, 122, 131); notes: none; supports 121, 122, 131
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched.
- Reference check: n/a.

### Seton2020
- Reference: Seton, M., Müller, R. D., Zahirovic, S., Williams, S., Wright, N. M., Cannon, J., Whittaker, J. M., Matthews, K. J., McGirr, R. (2020). A global data set of present-day oceanic crustal age and seafloor spreading parameters. Geochemistry, Geophysics, Geosystems 21(10):e2020GC009214.
- DOI or URL: 10.1029/2020GC009214 (open access).
- Cited as: not cited by the project; the earth file's "mean 64 Myr" for sea-floor age needs a source.
- Ledger ids: (supports 201 and the run review); notes: supports 201 and the run review
- Access: open access.
- Library (the project's source library, not distributed): fetched in WP-13a as data: the GMT remote server's grid earth_age_06m_p.grd (6 arc-minute, "EarthByte Earth Seafloor Age [Seton et al., 2020]") with the server's listing earth_age_server.txt. From: https://oceania.generic-mapping-tools.org/server/earth/earth_age/. Folder: Seton2020_earth_age.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (Crossref, full).
- Reference check: n/a.
- Read in WP-13a (2026-10-02): the grid, area-weighted by cos(latitude) over the 58 % of the sphere it covers (computed): median 55.4 Myr, mean 64.1, 90th percentile 133, 99th 173, oldest 339 Myr; 0.61 % older than 180 Myr and 0.06 % older than 250. Ledger: PLT-161, PLT-201, CHK-019, CHK-032 to CHK-034. The paper itself was not read.

### SyracuseAbers2006
- Reference: Syracuse, E. M., Abers, G. A. (2006). Global compilation of variations in slab depth beneath arc volcanoes and implications. Geochemistry, Geophysics, Geosystems 7(5):Q05017.
- DOI or URL: 10.1029/2005GC001045.
- Cited as: not cited; candidate source for arc-slab depth (flag 10; PLT-042).
- Ledger ids: (supports 042, 061, 164); notes: supports 042, 061, 164
- Access: open (G3).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (Crossref, full).
- Reference check: n/a.

### Lallemand2005
- Reference: Lallemand, S., Heuret, A., Boutelier, D. (2005). On the relationships between slab dip, back-arc stress, upper plate absolute motion, and crustal nature in subduction zones. Geochemistry, Geophysics, Geosystems 6(9):Q09006.
- DOI or URL: 10.1029/2005GC000917.
- Cited as: not cited; candidate for slab dip (the code has no dip; PLT-061).
- Ledger ids: (supports 061); notes: supports 061
- Access: open (G3).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched (Crossref, full).
- Reference check: n/a. The abstract gives dip relations (dips differ by about 20 degrees between oceanic and continental upper plates) but no global mean dip.

### Courtillot2003b
- Reference: Courtillot, V., Davaille, A., Besse, J., Stock, J. (2003). Three distinct types of hotspots in the Earth's mantle. Earth and Planetary Science Letters 205(3-4):295-308.
- DOI or URL: 10.1016/S0012-821X(02)01048-8.
- Cited as: named in the task (hotspot count); not cited in the code.
- Ledger ids: (supports 108-110); notes: supports 108-110
- Access: paywalled (no abstract in Crossref/OpenAlex).
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 301 text/html.
- Substitute: none found.
- Merged keys: Courtillot2003
- Read so far (by the project): not read.
- Read in this pass: metadata only. The number of hotspots they considered (49) was read in King & Adam 2014; "seven primary hotspots" was read on Wikipedia "Hotspot (geology)" (secondary).
- Reference check: correct.

### KingAdam2014
- Reference: King, S. D., Adam, C. (2014). Hotspot swells revisited. Physics of the Earth and Planetary Interiors 235:66-83 (volume from memory; the PDF header was not captured).
- DOI or URL: 10.1016/j.pepi.2014.07.006; open copy http://hdl.handle.net/10919/103026 (Virginia Tech).
- Cited as: not cited by the project; used here as an open, readable reproduction of the Parsons & Sclater (1977) law and of hotspot compilations.
- Ledger ids: (supports 092, 108-110); notes: supports 092, 108-110
- Access: open copy.
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: http://hdl.handle.net/10919/103026. File: KingAdam2014.txt.
- Read so far (by the project): not read.
- Read in this pass: full text fetched (introduction, Method).
- Reference check: n/a.

### Li2024
- Reference: Li, Y., Liu, L., Li, S., Peng, D., Cao, Z., Li, X. (2024). Cenozoic India-Asia collision driven by mantle dragging the cratonic root. Nature Communications 15:6674.
- DOI or URL: 10.1038/s41467-024-51107-0; https://pmc.ncbi.nlm.nih.gov/articles/PMC11303558/ (open).
- Cited as: not cited by the project; used as an open secondary source that cites Turcotte & Schubert for force orders of magnitude.
- Ledger ids: (supports 183, 187); notes: supports 183, 187
- Access: open access.
- Library (the project's source library, not distributed): fetched (HTML page, 60,549 characters of text). From: https://doi.org/10.1038/s41467-024-51107-0. Text file: Li2024.txt.
- Read so far (by the project): not read.
- Read in this pass: full text fetched (Europe PMC XML, grep for force magnitudes and reference 9).
- Reference check: n/a.

### DeMets1994
- Reference: DeMets, C., Gordon, R. G., Argus, D. F., Stein, S. (1994). Effect of recent revisions to the geomagnetic reversal time scale on estimates of current plate motions. Geophysical Research Letters 21(20):2191-2194.
- DOI or URL: 10.1029/94GL02118.
- Cited as: not cited (NUVEL-1A is the model behind Bird 2003's 14 large plates).
- Ledger ids: notes: none
- Access: paywalled (abstract).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not read.
- Read in this pass: abstract fetched; no spreading-rate numbers in it.
- Reference check: n/a.

### Davidson_tectonicsjs
- Reference: Davidson, C. tectonics.js, https://github.com/davidson16807/tectonics.js, files `RockColumn.js`, `Crust.js`, commit ce371fa (CC BY 4.0). Software.
- Ledger ids: PLT-131, PLT-136
- Library (the project's source library, not distributed): not fetched. Reason: no DOI or URL.

### Not bibliographic
GPlates/pyGPlates (software, GPL-2.0), the community ports of Cortial (robertoost/RNGalaxy, AurelienBesnier/ProceduralPlanets, gubace/Procedural-Tectonic-Planets), PlaTec/PyPlatec, WorldEngine, tectonics.cpp, Undiscovered Worlds, StagYY, ASPECT, Underworld, LaMEM, CitcomS, Rayleigh, pTatin3D, Wikipedia pages (Pacific plate, Izanagi plate, Kula plate, Iapetus Ocean, Supercontinent cycle, Alleghanian orogeny, Uralian orogeny, Himalayan foreland basin, List of tectonic plates), the geosociety.org "Broken sheets" page, the EarthByte page, `docs/deeptime-design-decisions.md`, `docs/deeptime-implementation-plan.md`, `docs/design/plate-collisions-and-welding.md`, `docs/design/plate-force-balance.md`, `docs/research/notes/tectonics_engines.md`, `docs/research/notes/run-review-1a0f969d358.md`, STATUS.md build-log entries, and the editor hint file `editor_fields.json`.

---

## Climate physics (CLM)

### Rose2018
- Reference: Rose, B. E. J. (2018). CLIMLAB: a Python toolkit for interactive, process-oriented climate modeling. Journal of Open Source Software 3(24): 659. Code: https://github.com/climlab/climlab (MIT; v0.9.2 tag 91eae4b; project cites commit fcbde37, a real commit dated 2026-07-23, "Merge pull request #284 from brian-rose/micromamba").
- DOI or URL: 10.21105/joss.00659
- Cited as: "climlab (Rose 2018, JOSS 3, 659)", "climlab const.S0", "climlab EBM_seasonal / StepFunctionAlbedo / AplusBT / MeridionalHeatDiffusion / DailyInsolation"
- Ledger ids: CLM-001, CLM-006 to CLM-010, CLM-020, CLM-027, CLM-029
- Access: open access (paper and code)
- Library (the project's source library, not distributed): fetched (PDF, 2 pages, text layer, 5,009 characters). From: https://joss.theoj.org/papers/10.21105/joss.00659.pdf. Text file: Rose2018.txt.
- Read so far (by the project): "read on 2026-09-28 (commit fcbde37)", equations ported not code; per-default reading not stated
- Read in this pass: full text fetched (source only): climlab/model/ebm.py, surface/albedo.py, utils/constants.py at main, v0.9.2 and fcbde37 (ebm.py identical in the defaults below at fcbde37 and v0.9.2; main differs from v0.9.2 at line 101, a docstring-area change, defaults unchanged). The JOSS paper itself was not fetched.
- Reference check: matches. Citation volume is 3, issue 24, article 659. Note the code's defaults are NOT NCC81: Rose's lecture notes (Albany ATM623, Lecture 14, fetched) say A = 210 m2 and B = 2 m2/degC "we are going to use these values", derived from an NCEP fit (best fit A = 214, B = 1.6), and D = 0.55.

### North1981
- Reference: North, G. R., Cahalan, R. F., & Coakley, J. A., Jr. (1981). Energy balance climate models. Reviews of Geophysics and Space Physics 19(1): 91-121.
- DOI or URL: 10.1029/RG019I001P00091. Image-only scan at https://eisenman.ucsd.edu/courses/2024-sioc224/files/north-1981.pdf (no text layer; JBIG2 images; I could not OCR it). NASA NTRS record 19820032405 (abstract only, no download).
- Cited as: "North, Cahalan & Coakley 1981", "North et al. 1981", "Rev. Geophys. 19, 91"
- Ledger ids: CLM-003 to CLM-006
- Access: paywalled (abstract only); scanned copy at the URL above, unreadable without OCR
- Library (the project's source library, not distributed): fetched (PDF, 31 pages, text layer, 138,049 characters). From: https://eisenman.ucsd.edu/courses/2024-sioc224/files/north-1981.pdf. Text file: North1981.txt.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (NTRS). Full text attempted, unreadable (scan).
- Reference check: authors, year, venue, pages match. The code's field comment writes "North et al. 1981" for D and ebm.rs writes the full name; same paper. Whether A = 203.3, B = 2.09, D = 0.649 are NCC81's own numbers is NOT confirmed (see Part 2): citing papers disagree on whether A and B originate with Budyko (1969) or North & Coakley (1979).

### Berger1978
- Reference: Berger, A. L. (1978). Long-term variations of daily insolation and Quaternary climatic changes. Journal of the Atmospheric Sciences 35(12): 2362-2367.
- DOI or URL: 10.1175/1520-0469(1978)035<2362:LTVODI>2.0.CO;2
- Cited as: "Berger 1978"
- Ledger ids: CLM-027, CLM-029, CLM-033, CLM-035, CLM-037
- Access: paywalled (Unpaywall: closed; AMS site blocks automated download)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches. Note a different Berger 1978 paper exists (Quaternary Research 9: 139-167, "Long-term variations of caloric insolation resulting from the Earth's orbital elements"); the code's J. Atmos. Sci. citation is the right one for the daily-insolation formula.

### Laskar2004
- Reference: Laskar, J., Robutel, P., Joutel, F., Gastineau, M., Correia, A. C. M., & Levrard, B. (2004). A long-term numerical solution for the insolation quantities of the Earth. Astronomy & Astrophysics 428(1): 261-285.
- DOI or URL: 10.1051/0004-6361:20041335; https://www.aanda.org/articles/aa/pdf/2004/46/aa1335.pdf (open, but automated fetch returned a bot challenge / 403)
- Cited as: "Laskar et al. 2004, A&A 428: 261-285"
- Ledger ids: CLM-033, CLM-035, CLM-037
- Access: open access (Unpaywall: bronze)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Bergman2004
- Reference: Bergman, N. M., Lenton, T. M., & Watson, A. J. (2004). COPSE: A new model of biogeochemical cycling over Phanerozoic time. American Journal of Science 304(5): 397-437.
- DOI or URL: 10.2475/ajs.304.5.397; open PDF https://ajs.scholasticahq.com/article/61725.pdf
- Cited as: "Bergman, Lenton & Watson 2004 COPSE"
- Ledger ids: CLM-046, CLM-051
- Access: open access (bronze, publisher PDF)
- Library (the project's source library, not distributed): fetched (PDF, 41 pages, text layer, 88,971 characters). From: https://ajs.scholasticahq.com/article/61725.pdf. Text file: Bergman2004.txt.
- Read so far (by the project): not stated
- Read in this pass: full text fetched (text only; the reservoir and flux tables are not machine-readable, so the numerical constants were not seen)
- Reference check: matches. The paper writes "Berner's GEOCARB" degassing constants k12/k13 as GEOCARB values times present reservoir sizes; I did not find the number 6.65e12 in the readable text.

### Archer2009
- Reference: Archer, D., Eby, M., Brovkin, V., Ridgwell, A., Cao, L., Mikolajewicz, U., Caldeira, K., Matsumoto, K., Munhoven, G., Montenegro, A., & Tokos, K. (2009). Atmospheric lifetime of fossil fuel carbon dioxide. Annual Review of Earth and Planetary Sciences 37: 117-134.
- DOI or URL: 10.1146/annurev.earth.031208.100206; green copy https://orbi.uliege.be/handle/2268/12933
- Cited as: "Archer et al. 2009"
- Ledger ids: CLM-048
- Access: open copy at https://orbi.uliege.be/handle/2268/12933 (landing page only seen)
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://orbi.uliege.be/handle/2268/12933. File: Archer2009.txt.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches (year 2009, vol 37, pp 117-134).

### Hoffman1998
- Reference: Hoffman, P. F., Kaufman, A. J., Halverson, G. P., & Schrag, D. P. (1998). A Neoproterozoic snowball Earth. Science 281(5381): 1342-1346.
- DOI or URL: 10.1126/science.281.5381.1342
- Cited as: "Hoffman et al. 1998"
- Ledger ids: CLM-050
- Access: paywalled (abstract only; Unpaywall closed)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (via Crossref)
- Reference check: matches. The abstract has the 350-times figure but not "about 3 to 4 Myr" or "5.4e12 mol/yr".

### Rowley2002
- Reference: Rowley, D. B. (2002). Rate of plate creation and destruction: 180 Ma to present. Geological Society of America Bulletin 114(8): 927-933.
- DOI or URL: 10.1130/0016-7606(2002)114<0927:ROPCAD>2.0.CO;2
- Cited as: "Rowley 2002"
- Ledger ids: CLM-053, CLM-054
- Access: paywalled (Unpaywall closed)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Self2006
- Reference: Self, S., Widdowson, M., Thordarson, T., & Jay, A. E. (2006). Volatile fluxes during flood basalt eruptions and potential effects on the global environment: A Deccan perspective. Earth and Planetary Science Letters 248(1-2): 518-532.
- DOI or URL: 10.1016/j.epsl.2006.05.041; green copy https://hull-repository.worktribe.com/output/3566573
- Cited as: "Self et al. 2006"
- Ledger ids: CLM-057, CLM-058
- Access: open copy (Hull repository, landing page only)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Walker1981
- Reference: Walker, J. C. G., Hays, P. B., & Kasting, J. F. (1981). A negative feedback mechanism for the long-term stabilization of Earth's surface temperature. Journal of Geophysical Research 86(C10): 9776-9782.
- DOI or URL: 10.1029/JC086IC10P09776
- Cited as: "Walker, Hays & Kasting 1981", "Walker, Hays & Kasting 1981 ("0.3")"
- Ledger ids: CLM-060, CLM-063, CLM-065
- Access: paywalled (abstract only; Unpaywall closed)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (Crossref) plus citing passages
- Reference check: matches.

### Myhre1998
- Reference: Myhre, G., Highwood, E. J., Shine, K. P., & Stordal, F. (1998). New estimates of radiative forcing due to well mixed greenhouse gases. Geophysical Research Letters 25(14): 2715-2718.
- DOI or URL: 10.1029/98GL01908
- Cited as: "Myhre et al. 1998"
- Ledger ids: CLM-069, CLM-073
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (Crossref) plus citing passages
- Reference check: matches. Byrne & Goldblatt's Table 1 prints the same 5.35 form (their row attributes it to "IPCC [1990]" and the adjusted forcing F_a).

### Byrne2014
- Reference: Byrne, B., & Goldblatt, C. (2014). Radiative forcing at high concentrations of well-mixed greenhouse gases. Geophysical Research Letters 41(1): 152-160.
- DOI or URL: 10.1002/2013GL058456; open copy https://dspace.library.uvic.ca/server/api/core/bitstreams/65d2a0aa-a7eb-44ba-b8e7-ac34d0d894d7/content (the article as corrected 17 Apr 2015)
- Cited as: "Byrne & Goldblatt 2014, GRL 41: 152-160, Table 2"
- Ledger ids: CLM-070, CLM-071
- Access: open copy at the UVic URL above (publisher page returned 403)
- Library (the project's source library, not distributed): fetched (PDF, 10 pages, text layer, 29,061 characters). From: https://dspace.library.uvic.ca/server/api/core/bitstreams/65d2a0aa-a7eb-44ba-b8e7-ac34d0d894d7/content. Text file: Byrne2014.txt.
- Read so far (by the project): "checked in their Table 2 by a Sonnet lookup" (a sub-agent)
- Read in this pass: full text fetched (all sections, Tables 1 and 2)
- Reference check: matches (authors, year, volume, pages, Table 2).

### Hoffman2017
- Reference: Hoffman, P. F., Abbot, D. S., Ashkenazy, Y., Benn, D. I., Brocks, J. J., Cohen, P. A., Cox, G. M., Creveling, J. R., Donnadieu, Y., Erwin, D. H., Fairchild, I. J., Ferreira, D., Goodman, J. C., Halverson, G. P., Jansen, M. F., Le Hir, G., Love, G. D., Macdonald, F. A., Maloof, A. C., Partin, C. A., Ramstein, G., Rose, B. E. J., Rose, C. V., Sadler, P. M., Tziperman, E., Voigt, A., & Warren, S. G. (2017). Snowball Earth climate dynamics and Cryogenian geology-geobiology. Science Advances 3(11): e1600983.
- DOI or URL: 10.1126/sciadv.1600983; open copy https://dspace.library.uvic.ca/bitstream/1828/11278/1/hoffman_paul_scienceadvances_2017.pdf
- Cited as: "Hoffman et al. 2017, Sci. Adv. 3: e1600983"
- Ledger ids: CLM-103
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 45 pages, text layer, 258,807 characters). From: https://dspace.library.uvic.ca/bitstream/1828/11278/1/hoffman_paul_scienceadvances_2017.pdf. Text file: Hoffman2017.txt.
- Read so far (by the project): not stated
- Read in this pass: full text fetched (text layer searched for sea level, ppm, bar, CO2 termination, ice thickness; not every section read)
- Reference check: matches. The paper's CO2 range for termination is 10^4 to 10^5 ppm, and the ~1 km of sea level is stated as an assumption (see Part 2).

### Nye1952
- Reference: Nye, J. F. (1952). The mechanics of glacier flow. Journal of Glaciology 2(12): 82-93.
- DOI or URL: 10.3189/S0022143000033967
- Cited as: "Nye 1952, J. Glaciol. 2: 82-93"
- Ledger ids: CLM-101
- Access: paywalled (Unpaywall closed)
- Library (the project's source library, not distributed): fetched (PDF, 12 pages, text layer, 31,196 characters). From: https://www.cambridge.org/core/services/aop-cambridge-core/content/view/5FB3C31120796459A837491ACB085F32/S0022143000033967a.pdf/div-class-title-the-mechanics-of-glacier-flow-div.pdf. Text file: Nye1952.txt.
- Substitute: repository or open copy found by the lookups and saved as above (version not stated: the first pages were matched against the title, not compared with the printed version).
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches (issue 12).

### Imbrie1980
- Reference: Imbrie, J., & Imbrie, J. Z. (1980). Modeling the climatic response to orbital variations. Science 207(4434): 943-953.
- DOI or URL: 10.1126/science.207.4434.943
- Cited as: "Imbrie & Imbrie 1980"
- Ledger ids: CLM-104, CLM-105
- Access: paywalled (Unpaywall closed)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Fretwell2013
- Reference: Fretwell, P., Pritchard, H. D., Vaughan, D. G., Bamber, J. L., Barrand, N. E., Bell, R., et al. (2013). Bedmap2: improved ice bed, surface and thickness datasets for Antarctica. The Cryosphere 7(1): 375-393.
- DOI or URL: 10.5194/tc-7-375-2013; https://tc.copernicus.org/articles/7/375/2013/tc-7-375-2013.pdf
- Cited as: "Fretwell et al. 2013" (Antarctica and Greenland about 3.0e7 km3, 65 m)
- Ledger ids: CLM-108
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 19 pages, text layer, 74,822 characters). From: https://tc.copernicus.org/articles/7/375/2013/tc-7-375-2013.pdf. Text file: Fretwell2013.txt.
- Read so far (by the project): not stated
- Read in this pass: full text fetched (abstract and discussion passages on ice volume and sea-level potential)
- Reference check: matches. It covers Antarctica only; Greenland comes from Morlighem2017.

### Morlighem2017
- Reference: Morlighem, M., Williams, C. N., Rignot, E., An, L., Arndt, J. E., Bamber, J. L., et al. (2017). BedMachine v3: Complete bed topography and ocean bathymetry mapping of Greenland from multibeam echo sounding combined with mass conservation. Geophysical Research Letters 44(21): 11051-11061.
- DOI or URL: 10.1002/2017GL074954; open copy https://pure.au.dk/ws/files/120033749/Morlighem_et_al_2017_Geophysical_Research_Letters.pdf
- Cited as: "Morlighem et al. 2017"
- Ledger ids: CLM-108
- Access: open access (hybrid; many repository copies)
- Library (the project's source library, not distributed): fetched (PDF, 11 pages, text layer, 44,690 characters). From: https://pure.au.dk/ws/files/120033749/Morlighem_et_al_2017_Geophysical_Research_Letters.pdf. Text file: Morlighem2017.txt.
- Read so far (by the project): not stated
- Read in this pass: full text fetched (key points, abstract, results text)
- Reference check: matches. Page range 11051-11061 is from my memory of the article number format, not from Crossref (Crossref returned no page); treat as unverified.

### Abbot2010
- Reference: Abbot, D. S., & Pierrehumbert, R. T. (2010). Mudball: Surface dust and Snowball Earth deglaciation. Journal of Geophysical Research: Atmospheres 115(D3): D03104.
- DOI or URL: 10.1029/2009JD012007
- Cited as: "Abbot & Pierrehumbert 2010" (snowball dust darkening)
- Ledger ids: none (feature not shipped); notes: feature not shipped
- Access: not checked
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: exists, title matches the topic. Article number D03104 is from my memory, not Crossref (which returned "2009JD012007" as the article id); treat as unverified.

### gosplDocs
- Reference: goSPL (Global Scalable Paleo Landscape Evolution), Geodels/gospl, `gospl/tools/ela_from_temperature.py`. Software, GPL.
- DOI or URL: https://github.com/Geodels/gospl/blob/HEAD/gospl/tools/ela_from_temperature.py
- Cited as: "goSPL documentation, ELA rule"
- Ledger ids: GLA-036
- Access: open access (source file)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 503 text/html; charset=utf-8.
- Read so far (by the project): the rule quoted from the documentation; code not used (GPL)
- Read in this pass: full text fetched (the file)
- Reference check: matches the rule; no journal paper involved.

### Ziegler2021
- Reference: Ziegler, E., & Rehfeld, K. (2021). TransEBM v. 1.0: description, tuning, and validation of a transient model of the Earth's energy balance in two dimensions. Geoscientific Model Development 14(5): 2843-2866.
- DOI or URL: 10.5194/gmd-14-2843-2021
- Cited as: "Ziegler & Rehfeld 2021, GMD 14, 2843" (Table 1 constants)
- Ledger ids: CLM-119, CLM-120
- Access: open access (GMD)
- Library (the project's source library, not distributed): fetched (PDF, 24 pages, text layer, 83,679 characters). From: https://gmd.copernicus.org/articles/14/2843/2021/gmd-14-2843-2021.pdf. Text file: Ziegler2021.txt.
- Read so far (by the project): not stated; code is GPL-3 "oracle only"
- Read in this pass: not fetched
- Reference check: matches.

### Zhuang2017
- Reference: Zhuang, K., North, G. R., & Stevens, M. J. (2017). A NetCDF version of the two-dimensional energy balance model based on the full multigrid algorithm. SoftwareX 6: 198-202.
- DOI or URL: 10.1016/j.softx.2017.07.003
- Cited as: "Zhuang, North & Stevens 2017, SoftwareX 6, 198"
- Ledger ids: CLM-119, CLM-120
- Access: not checked (SoftwareX is open access)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 11 chars).
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### North1983
- Reference: North, G. R., Mengel, J. G., & Short, D. A. (1983). Simple energy balance model resolving the seasons and the continents: Application to the astronomical theory of the ice ages. Journal of Geophysical Research 88(C11): 6576-6586.
- DOI or URL: 10.1029/JC088IC11P06576
- Cited as: "North, Mengel & Short 1983, JGR 88(C11) 6576"
- Ledger ids: CLM-122
- Access: not checked
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): "paper; equations only"
- Read in this pass: not fetched
- Reference check: matches (DOI as the design doc gives it).

### NorthKim2017
- Reference: North, G. R., & Kim, K.-Y. (2017). Energy Balance Climate Models. Wiley-VCH.
- DOI or URL: 10.1002/9783527698844
- Cited as: "North & Kim 2017, Energy Balance Climate Models, Wiley (ch. 8)"
- Ledger ids: none directly
- Access: book (not online)
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches. Chapter 8 not checked.

### Stommel1948
- Reference: Stommel, H. (1948). The westward intensification of wind-driven ocean currents. Transactions, American Geophysical Union 29(2): 202-206.
- DOI or URL: 10.1029/TR029I002P00202
- Cited as: "Stommel 1948, Trans. AGU 29, 202"
- Ledger ids: CLM-128
- Access: not checked
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): "implemented from the paper"
- Read in this pass: not fetched
- Reference check: matches.

### RoseMarshall2009
- Reference: Rose, B. E. J., & Marshall, J. (2009). Ocean heat transport, sea ice, and multiple climate states: Insights from energy balance models. Journal of the Atmospheric Sciences 66(9): 2828-2843.
- DOI or URL: 10.1175/2009JAS3039.1; MIT copy http://hdl.handle.net/1721.1/53519
- Cited as: "Rose & Marshall 2009, JAS 66, 2828"
- Ledger ids: none
- Access: open copy (Unpaywall: bronze; MIT repository)
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: http://hdl.handle.net/1721.1/53519. File: RoseMarshall2009.txt.
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### Ferreira2018
- Reference: Ferreira, D., Cessi, P., Coxall, H. K., de Boer, A., Dijkstra, H. A., Drijfhout, S. S., Eldevik, T., Harnik, N., McManus, J. F., Marshall, D. P., Nilsson, J., Roquet, F., Schneider, T., & Wills, R. C. (2018). Atlantic-Pacific asymmetry in deep water formation. Annual Review of Earth and Planetary Sciences 46: 327-352.
- DOI or URL: 10.1146/annurev-earth-082517-010045; green copy https://dspace.library.uu.nl/handle/1874/428290
- Cited as: "Ferreira et al. 2018, Annu. Rev. Earth Planet. Sci. 46, 327" (marked "cited from memory, not re-read")
- Ledger ids: CLM-132
- Access: open copy (Utrecht repository, landing page)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): from memory
- Read in this pass: abstract fetched (Crossref/Semantic Scholar)
- Reference check: EXISTS and the citation is right (46: 327-352). What it is cited for (deep water forms in the narrowest polar-connected basin; basin width selects the Atlantic) is NOT what the abstract leads with: the abstract says the asymmetry occurs "because the Atlantic is more saline". Citing papers also say basin width/continent shape is one proposed control (see Part 2).

### Trenberth2001
- Reference: Trenberth, K. E., & Caron, J. M. (2001). Estimates of meridional atmosphere and ocean heat transports. Journal of Climate 14(16): 3433-3443.
- DOI or URL: 10.1175/1520-0442(2001)014<3433:EOMAAO>2.0.CO;2
- Cited as: "Trenberth & Caron 2001, J. Climate 14, 3433"
- Ledger ids: CLM-133
- Access: open access (bronze; AMS site blocks automated download)
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 404 text/html;charset=utf-8.
- Read so far (by the project): not stated, not marked from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: matches.

### Lindzen1987
- Reference: Lindzen, R. S., & Nigam, S. (1987). On the role of sea surface temperature gradients in forcing low-level winds and convergence in the tropics. Journal of the Atmospheric Sciences 44(17): 2418-2436.
- DOI or URL: 10.1175/1520-0469(1987)044<2418:OTROSS>2.0.CO;2
- Cited as: "Lindzen & Nigam 1987, JAS 44, 2418" (marked from memory)
- Ledger ids: CLM-141
- Access: paywalled (Unpaywall closed)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, citation right. The paper is about SST gradients forcing marine boundary-layer pressure gradients; the design uses the idea for land-sea (monsoon) temperature contrasts. That is an extension, not the paper's subject.

### Dommenget2011
- Reference: Dommenget, D., & Flöter, J. (2011). Conceptual understanding of climate change with a globally resolved energy balance model. Climate Dynamics 37(11-12): 2143-2165.
- DOI or URL: 10.1007/s00382-011-1026-0
- Cited as: "Dommenget & Floter 2011, Clim. Dyn. 37, 2143"
- Ledger ids: CLM-142
- Access: not checked
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): "equations only"; code has no licence
- Read in this pass: not fetched
- Reference check: matches.

### Stassen2019
- Reference: Stassen, C., Dommenget, D., & Loveday, N. (2019). A hydrological cycle model for the Globally Resolved Energy Balance (GREB) model v1.0. Geoscientific Model Development 12(1): 425-440.
- DOI or URL: 10.5194/gmd-12-425-2019
- Cited as: "Stassen, Dommenget & Loveday 2019, GMD 12, 425"
- Ledger ids: CLM-142
- Access: open access (GMD)
- Library (the project's source library, not distributed): fetched (PDF, 16 pages, text layer, 50,007 characters). From: https://gmd.copernicus.org/articles/12/425/2019/gmd-12-425-2019.pdf. Text file: Stassen2019.txt.
- Read so far (by the project): "equations only"
- Read in this pass: not fetched
- Reference check: matches.

### Fanning1996
- Reference: Fanning, A. F., & Weaver, A. J. (1996). An atmospheric energy-moisture balance model: Climatology, interpentadal climate change, and coupling to an ocean general circulation model. Journal of Geophysical Research 101(D10): 15111-15128.
- DOI or URL: 10.1029/96JD01017
- Cited as: "Fanning & Weaver 1996, JGR 101(D10), 15111"
- Ledger ids: none
- Access: not checked
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): not stated; code licence unverifiable
- Read in this pass: not fetched
- Reference check: matches.

### Weaver2001
- Reference: Weaver, A. J., Eby, M., Wiebe, E. C., Bitz, C. M., Duffy, P. B., Ewen, T. L., Fanning, A. F., Holland, M. M., MacFadyen, A., Matthews, H. D., Meissner, K. J., Saenko, O., Schmittner, A., Wang, H., & Yoshimori, M. (2001). The UVic Earth System Climate Model: Model description, climatology, and applications to past, present and future climates. Atmosphere-Ocean 39(4): 361-428.
- DOI or URL: 10.1080/07055900.2001.9649686
- Cited as: "Weaver et al. 2001"
- Ledger ids: none
- Access: not checked
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### Bretherton2004
- Reference: Bretherton, C. S., Peters, M. E., & Back, L. E. (2004). Relationships between water vapor path and precipitation over the tropical oceans. Journal of Climate 17(7): 1517-1528.
- DOI or URL: 10.1175/1520-0442(2004)017<1517:RBWVPA>2.0.CO;2
- Cited as: "Bretherton, Peters & Back 2004, J. Climate 17, 1517" (marked from memory)
- Ledger ids: CLM-145
- Access: open access (bronze). AMS site blocks automated download; the Wayback copy is a page scan (no text layer).
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 404 text/html;charset=utf-8.
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, citation right. See Part 2 for the 15.6.

### Manabe1969
- Reference: Manabe, S. (1969). Climate and the ocean circulation: I. The atmospheric circulation and the hydrology of the earth's surface. Monthly Weather Review 97(11): 739-774.
- DOI or URL: 10.1175/1520-0493(1969)097<0739:CATOC>2.3.CO;2
- Cited as: "Manabe 1969, Mon. Weather Rev. 97, 739" (marked from memory)
- Ledger ids: CLM-143
- Access: paywalled (Unpaywall closed)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, citation right (Part I, pp. 739-774; Part II by Manabe & Bryan follows at 775-805).

### Siler2018
- Reference: Siler, N., Roe, G. H., & Armour, K. C. (2018). Insights into the zonal-mean response of the hydrologic cycle to global warming from a diffusive energy balance model. Journal of Climate 31(18): 7481-7493.
- DOI or URL: 10.1175/JCLI-D-18-0081.1
- Cited as: "Siler, Roe & Armour 2018"
- Ledger ids: none
- Access: paywalled (Unpaywall closed)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### Bischoff2014
- Reference: Bischoff, T., & Schneider, T. (2014). Energetic constraints on the position of the intertropical convergence zone. Journal of Climate 27(13): 4937-4951.
- DOI or URL: 10.1175/JCLI-D-13-00650.1
- Cited as: "Bischoff & Schneider 2014"
- Ledger ids: none
- Access: open access (bronze)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): not stated
- Read in this pass: not fetched
- Reference check: matches.

### Wagner2015
- Reference: Wagner, T. J. W., & Eisenman, I. (2015). How climate model complexity influences sea ice stability. Journal of Climate 28(10): 3998-4014.
- DOI or URL: 10.1175/JCLI-D-14-00654.1
- Cited as: "Wagner & Eisenman 2015, J. Climate 28, 3998"
- Ledger ids: none
- Access: open access (bronze)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): "equations only"
- Read in this pass: not fetched
- Reference check: matches.

### Klein1993
- Reference: Klein, S. A., & Hartmann, D. L. (1993). The seasonal cycle of low stratiform clouds. Journal of Climate 6(8): 1587-1606.
- DOI or URL: 10.1175/1520-0442(1993)006<1587:TSCOLS>2.0.CO;2
- Cited as: "Klein & Hartmann 1993, J. Climate 6, 1587" (marked from memory)
- Ledger ids: none (follow-up not coded); notes: follow-up not coded
- Access: paywalled (Unpaywall closed)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, citation right. Citing papers define lower-tropospheric stability (LTS) from this paper, which fits "low cloud" as the stated use.

### Serreze1992
- Reference: Serreze, M. C., Kahl, J. D., & Schnell, R. C. (1992). Low-level temperature inversions of the Eurasian Arctic and comparisons with Soviet drifting station data. Journal of Climate 5(6): 615-629. (Crossref lists the author order Serreze, Schnell, Kahl.)
- DOI or URL: 10.1175/1520-0442(1992)005<0615:LLTIOT>2.0.CO;2
- Cited as: "Serreze, Kahl & Schnell 1992, J. Climate 5: 615" (marked from memory)
- Ledger ids: CLM-134
- Access: open access (bronze); AMS blocks automated download
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: HTTP 404 text/html;charset=utf-8.
- Read so far (by the project): from memory
- Read in this pass: search result snippet only (citing passages)
- Reference check: EXISTS, pages right. Author order differs: Crossref gives Serreze, Schnell, Kahl; the code writes Serreze, Kahl & Schnell. The paper is about inversions over the Eurasian Arctic (soundings and drifting stations), which fits "winter surface inversion"; the specific "Siberia's January surface 10-20 K below 850 hPa" could not be checked.

### Ramirez2024
- Reference: Ramirez, R. M. (2024). A new 2D energy balance model for simulating the climates of rapidly and slowly rotating terrestrial planets. The Planetary Science Journal 5(1): 2.
- DOI or URL: 10.3847/PSJ/AD0729; preprint arXiv 2310.15992
- Cited as: "PlaHab (Ramirez 2023, PSJ)"
- Ledger ids: CLM-121
- Access: open access (PSJ; arXiv preprint)
- Library (the project's source library, not distributed): fetched (PDF, 17 pages, text layer, 53,950 characters). From: https://iopscience.iop.org/article/10.3847/PSJ/ad0729/pdf. Text file: Ramirez2024.txt.
- Read so far (by the project): "not inspected, nothing taken"
- Read in this pass: not fetched (arxiv.org/html/2310.15992 returned 404; PDF not text-readable)
- Reference check: IDENTIFIED by title search (the name "PlaHab" returns nothing in Crossref). Year is 2024 in the journal (2023 as a preprint); the design doc's "Ramirez 2023" is the preprint year. I did not confirm that the paper uses 1/25 for the zonal-to-meridional ratio.

### Tetens
- Reference: UNIDENTIFIED in this pass. The code names only "Tetens" for a saturation-vapour formula. Likely Tetens, O. (1930), "Über einige meteorologische Begriffe", Zeitschrift für Geophysik 6: 297-309; this is my recollection, not verified (no DOI, no lookup possible).
- DOI or URL: none
- Cited as: "Tetens"
- Ledger ids: CLM-142
- Access: not found
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Identification (stage 2, WP-0): Not resolved: Tetens, O. (1930), Zeitschrift fuer Geophysik 6:297-309 is the likely work (an old German journal). The saturation-vapour formula in common use is quoted in many open textbooks.
- Read so far (by the project): no citation given
- Read in this pass: not fetched
- Reference check: cannot confirm.

### ETOPO2022
- Reference: NOAA National Centers for Environmental Information (2022). ETOPO 2022 15 Arc-Second Global Relief Model. NOAA NCEI.
- DOI or URL: 10.25921/fd45-gt74 (title and year confirmed through the DataCite API)
- Cited as: "ETOPO 2022"
- Ledger ids: none
- Access: open access (dataset)
- Library (the project's source library, not distributed): only the landing page was saved (abstract or a page shell; no full text). From: https://doi.org/10.25921/fd45-gt74. File: ETOPO2022.txt.
- Read so far (by the project): data named
- Read in this pass: not fetched
- Reference check: matches.

### Rayner2003
- Reference: Rayner, N. A., Parker, D. E., Horton, E. B., Folland, C. K., Alexander, L. V., Rowell, D. P., Kent, E. C., & Kaplan, A. (2003). Global analyses of sea surface temperature, sea ice, and night marine air temperature since the late nineteenth century. Journal of Geophysical Research 108(D14): 4407 (Crossref article id 2002JD002670).
- DOI or URL: 10.1029/2002JD002670
- Cited as: "HadISST" (named only)
- Ledger ids: none
- Access: not checked
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Read so far (by the project): data named
- Read in this pass: not fetched
- Reference check: HadISST1 reference (my assignment; the code names no paper). Article number 4407 is from my memory.

### Adler2018
- Reference: Adler, R. F., Sapiano, M. R. P., Huffman, G. J., Wang, J.-J., Gu, G., Bolvin, D., Chiu, L., Schneider, U., Becker, A., Nelkin, E., Xie, P., Ferraro, R., & Shin, D.-B. (2018). The Global Precipitation Climatology Project (GPCP) Monthly Analysis (New Version 2.3) and a Review of 2017 Global Precipitation. Atmosphere 9(4): 138.
- DOI or URL: 10.3390/atmos9040138
- Cited as: "GPCP v2.3" (named only)
- Ledger ids: none
- Access: open access (MDPI; not checked)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): data named
- Read in this pass: not fetched
- Reference check: matches the version named.

### Rohde2020
- Reference: Rohde, R. A., & Hausfather, Z. (2020). The Berkeley Earth Land/Ocean Temperature Record. Earth System Science Data 12(4): 3469-3479. Data file: https://berkeley-earth-temperature.s3.us-west-1.amazonaws.com/Global/Land_and_Ocean_complete.txt (analysis run 2025-01).
- DOI or URL: 10.5194/essd-12-3469-2020
- Cited as: none (supplied here for Earth's global mean temperature)
- Ledger ids: CLM-026, CLM-078, CLM-138
- Access: open access (paper and data)
- Library (the project's source library, not distributed): fetched (PDF, 11 pages, text layer, 38,089 characters). From: https://essd.copernicus.org/articles/12/3469/2020/essd-12-3469-2020.pdf. Text file: Rohde2020.txt.
- Read so far (by the project): n/a
- Read in this pass: full text fetched (the data file header and the annual series, not the paper)
- Reference check: n/a

### Kopp2011
- Reference: Kopp, G., & Lean, J. L. (2011). A new, lower value of total solar irradiance: Evidence and climate significance. Geophysical Research Letters 38(1): L01706.
- DOI or URL: 10.1029/2010GL045777
- Cited as: none (supplied here for the solar constant)
- Ledger ids: CLM-001
- Access: paywalled? (Wiley page blocked; not checked)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): n/a
- Read in this pass: search result snippet only (Kopp's own web page, https://spot.colorado.edu/~koppg/TSI/, quotes the value)
- Reference check: n/a. Article number L01706 is from my memory.

### TransEBM and related EBM literature (designed-not-coded rows)


### Added Earth-reference sources (not cited in the code)


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

---

## Lithospheric flexure (ISO-040 to ISO-067)

### Calmant1990
- Reference: Calmant, S., Francheteau, J., Cazenave, A. (1990). Elastic layer thickening with age of the oceanic lithosphere: a tool for prediction of the age of volcanoes or oceanic crust. Geophysical Journal International 100(1):59-67.
- DOI or URL: 10.1111/j.1365-246X.1990.tb04567.x; full text https://horizon.documentation.ird.fr/exl-doc/pleins_textes/pleins_textes_5/b_fdi_31-32/34824.pdf
- Cited as: "Calmant, Francheteau & Cazenave 1990"
- Ledger ids: ISO-048
- Access: open copy.
- Library (the project's source library, not distributed): fetched (PDF, 9 pages, text layer, 33,978 characters). From: https://horizon.documentation.ird.fr/exl-doc/pleins_textes/pleins_textes_5/b_fdi_31-32/34824.pdf. Text file: Calmant1990.txt.
- Read so far (by the project): full text by a Sonnet sub-agent of the flexure agent on 2026-10-02: "Te (km) = (2.7 +- 0.15) sqrt(t (Ma))", a weighted regression of 38 seamount estimates; rheology prediction 3.3 (wet) to 4.3 (dry).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### Watts1978
- Reference: Watts, A. B. (1978). An analysis of isostasy in the world's oceans 1. Hawaiian-Emperor Seamount Chain. Journal of Geophysical Research 83(B12):5989-6004.
- DOI or URL: 10.1029/JB083iB12p05989
- Cited as: "Watts 1978"
- Ledger ids: ISO-048, ISO-056; notes: and the comment of ISO-048
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: Te 20-30 km along the chain, "corresponds closely to the 450 +- 150 C isotherm".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct. That a load keeps the elastic thickness of its time of loading is how the abstract is commonly summarised; the paper body was not read.

### WattsTenBrink1989
- Reference: Watts, A. B., ten Brink, U. S. (1989). Crustal structure, flexure, and subsidence history of the Hawaiian Islands. Journal of Geophysical Research 94(B8):10473-10500.
- DOI or URL: 10.1029/JB094iB08p10473
- Cited as: "Watts & ten Brink 1989"
- Ledger ids: ISO-049
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: a good fit with Te = 40 km, "higher than expected for 80 m.y. oceanic lithosphere".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct. The arch's distance and height, often attributed to it, were not confirmed.

### Wessel1993
- Reference: Wessel, P. (1993). A reexamination of the flexural deformation beneath the Hawaiian Islands. Journal of Geophysical Research 98(B7):12177-12190.
- DOI or URL: 10.1029/93JB00523
- Cited as: docs/design/flexure.md section 11
- Ledger ids: ISO-049; notes: context for ISO-049
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: Te 35 km away from the load, 25 km under it.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### BurovDiament1995
- Reference: Burov, E. B., Diament, M. (1995). The effective elastic thickness (Te) of continental lithosphere: What does it really mean? Journal of Geophysical Research 100(B3):3905-3927.
- DOI or URL: 10.1029/94JB02770
- Cited as: "Burov & Diament 1995"
- Ledger ids: ISO-050
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.09, 1010 chars).
- Substitute: none found.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: 70-90 km for cratons, 10-20 km at the low end; thermotectonic age is one of at least three controls (with crust-mantle decoupling and plate curvature).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct. The model uses the age control only.

### PerezGussinyeWatts2005
- Reference: Pérez-Gussinyé, M., Watts, A. B. (2005). The long-term strength of Europe and its implications for plate-forming processes. Nature 436:381-384.
- DOI or URL: 10.1038/nature03854
- Cited as: docs/design/flexure.md section 11
- Ledger ids: ISO-050; notes: context for ISO-050
- Access: paywalled (abstract on Europe PMC).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: mean Te above 60 km for lithosphere of 1.5 Gyr and older, under 30 km for younger.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### Lu2024
- Reference: Lu, Z., Audet, P., Li, C.-F., Zhang, J., Li, S. (2024). Strength of continental lithosphere governed by the time since the last orogeny. Earth and Planetary Science Letters 643:118894.
- DOI or URL: 10.1016/j.epsl.2024.118894
- Cited as: "Lu et al. 2024"
- Ledger ids: ISO-051, ISO-052
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no copy could be fetched: html not the work (title match 0.00, 11 chars).
- Substitute: none found.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02 (on the authors' group page; the publisher's page was blocked): continental Te from 5 to about 140 km, mean 50 km, governed by the time since the last orogeny.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: record confirmed in Crossref. It gives no recovery law in the abstract: the exponential and its 300 Myr are not from it.

### KarnerWatts1983
- Reference: Karner, G. D., Watts, A. B. (1983). Gravity anomalies and flexure of the lithosphere at mountain ranges. Journal of Geophysical Research 88(B12):10449-10477.
- DOI or URL: 10.1029/JB088iB12p10449
- Cited as: docs/design/flexure.md section 11
- Ledger ids: ISO-050, ISO-055; notes: context for ISO-050, ISO-055
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: Te 80-100 km for the Himalaya's foreland, 25-50 km for the Alps, 80-130 km for the Appalachians.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### LyonCaenMolnar1983
- Reference: Lyon-Caen, H., Molnar, P. (1983). Constraints on the structure of the Himalaya from an analysis of gravity anomalies and a flexural model of the lithosphere. Journal of Geophysical Research 88(B10):8171-8191.
- DOI or URL: 10.1029/JB088iB10p08171
- Cited as: "Lyon-Caen & Molnar 1983"
- Ledger ids: ISO-055
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: flexural rigidity about 0.7e25 N m under the Ganga basin and Lesser Himalaya, 0.1 to 1.0e23 N m beneath the Greater Himalaya.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct. It is evidence for a stiff foreland beside a weak belt, not for a break along every plate boundary.

### BrotchieSilvester1969
- Reference: Brotchie, J. F., Silvester, R. (1969). On crustal flexure. Journal of Geophysical Research 74(22):5240-5252.
- DOI or URL: 10.1029/JB074i022p05240
- Cited as: "Brotchie & Silvester 1969"
- Ledger ids: ISO-067
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not read; the point-load formula was read as quoted in Wickert 2016 (eq. 5-6, full text) by a Sonnet sub-agent of the flexure agent on 2026-10-02.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: record correct.

### Wickert2016
- Reference: Wickert, A. D. (2016). Open-source modular solutions for flexural isostasy: gFlex v1.0. Geoscientific Model Development 9(3):997-1017.
- DOI or URL: 10.5194/gmd-9-997-2016
- Cited as: docs/design/flexure.md section 11
- Ledger ids: ISO-067
- Access: open access (the paper is CC-BY; the code is GPL-3).
- Library (the project's source library, not distributed): fetched (PDF, 21 pages, text layer, 74,175 characters). From: https://gmd.copernicus.org/articles/9/997/2016/gmd-9-997-2016.pdf. Text file: Wickert2016.txt.
- Read so far (by the project): the paper's full text by a Sonnet sub-agent of the flexure agent on 2026-10-02, for the point-load formula and the licence. The code was not opened and nothing is ported.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### WieczerkowskiMitrovicaWolf1999
- Reference: Wieczerkowski, K., Mitrovica, J. X., Wolf, D. (1999). A revised relaxation-time spectrum for Fennoscandia. Geophysical Journal International 139(1):69-86.
- DOI or URL: 10.1046/j.1365-246X.1999.00924.x
- Cited as: docs/design/flexure.md sections 7 and 11
- Ledger ids: ISO-064
- Access: open (publisher page).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
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


## Force-balance plate motion: sources added in WP-13a (2026-10-02)

### FaccennaBecker2012
- Reference: Faccenna, C., Becker, T. W., Lallemand, S., Steinberger, B. (2012). On the role of slab pull in the Cenozoic motion of the Pacific plate. Geophysical Research Letters 39(3):L03305.
- DOI or URL: 10.1029/2011GL050155; author's preprint https://twb.ig.utexas.edu/preprints/fbls11.pdf.
- Ledger ids: PLT-182, PLT-183, PLT-184.
- Access: open copy (author's preprint, the published layout).
- Library (the project's source library, not distributed): fetched (PDF, text layer). Text file: FaccennaBecker2012.txt.
- Read in WP-13a (2026-10-02): eq. 1, F_sp = 0.25 drho g l 2.32 (kappa A)^(1/2) "following Turcotte and Schubert [1982]", slab length l "assumed constant at 700 km", drho 70 kg/m3, g about 10 m/s2, kappa 1e-6 m2/s, the 0.25 for the slab's heating after Carlson et al. 1983.

### DeCellesCarrapa2021
- Reference: DeCelles, P. G., Carrapa, B. (2021). Coupled rapid erosion and foreland sedimentation control orogenic wedge kinematics in the Himalayan thrust belt of central Nepal. Journal of Geophysical Research: Solid Earth 126(5):e2020JB021256.
- DOI or URL: 10.1029/2020JB021256; NSF public access copy https://par.nsf.gov/servlets/purl/10229963.
- Ledger ids: PLT-192.
- Access: open copy (NSF Public Access Repository).
- Library (the project's source library, not distributed): fetched (PDF, text layer). Text file: DeCellesCarrapa2021.txt.
- Read in WP-13a (2026-10-02): section 2.4.1 and the conclusions: Lesser Himalayan surface slopes about 0.4-1.6 deg, total taper 6-7.5 deg; foredeep total taper 2.6-3.1 deg; Greater Himalaya surface slopes 5.3-15.8 deg.

### vanHunen2011EGU
- Reference: van Hunen, J., Faccenna, C., Allen, M. (2011). Slab break-off and applications to Tethys subduction. Geophysical Research Abstracts 13, EGU2011-6298 (EGU General Assembly 2011).
- DOI or URL: https://meetingorganizer.copernicus.org/EGU2011/EGU2011-6298.pdf.
- Ledger ids: PLT-184.
- Access: open (conference abstract).
- Library (the project's source library, not distributed): fetched (PDF, text layer). Text file: vanHunen2011EGU.txt.
- Read in WP-13a (2026-10-02): the whole abstract: an old, strong slab breaks off "at 20-25 Myr after the first onset of continental collision", young or weak slabs "already 10 Myr after"; slabs rest on the mantle's "viscosity increase across 660 km". It stands in for van Hunen & Allen 2011 (EPSL), whose text could not be obtained.

### SchmalholzMancktelow2016
- Reference: Schmalholz, S. M., Mancktelow, N. S. (2016). Folding and necking across the scales: a review of theoretical and experimental results and their applications. Solid Earth 7:1417-1465.
- DOI or URL: 10.5194/se-7-1417-2016 (open access, CC BY 3.0).
- Ledger ids: PLT-187 (secondary for Molnar & Lyon-Caen 1988).
- Access: open access.
- Library (the project's source library, not distributed): already present under the key Schmalholz2016 (added by another package). Text file: Schmalholz2016.txt.
- Read in WP-13a (2026-10-02): the passage citing Molnar & Lyon-Caen 1988 for a horizontal driving force "of ~7 x 10^12 N m-1" in Tibet.

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
## Geothermal features and editor hints (GEO, HNT), added in WP-15 and WP-16

Works read for the geothermal page and the hint figure table (2026-10-02). Not counted in the totals above. "Fetch tool" means the page was read through a page-fetch tool that returns quoted sentences; the agency sites named return only a script stub to a plain download, so no copy is in the library.

### NPS_YellowstoneHydrothermal
- Reference: National Park Service, "Hydrothermal Features", Yellowstone National Park. https://www.nps.gov/yell/learn/nature/hydrothermal-features.htm
- Ledger ids: GEO-007, GEO-019, GEO-022, GEO-026, GEO-033, GEO-044
- Access: open (agency page).
- Library: saved page, `NPS_YellowstoneHydrothermal/nps.html` and `nps.txt`.
- Read in this pass: full text read for: more than 10,000 hydrothermal features of which more than 500 are geysers; hot-spring water begins as rain; a fumarole has "so little water in its system that the water boils away before reaching the surface", steam vents as hot as 138 degC; travertine terraces form where thermal water rises through limestone; thermophiles live at specific temperatures and show gradations of colour.

### NPS_YellowstoneVolcano
- Reference: National Park Service, "Volcano", Yellowstone National Park. https://www.nps.gov/yell/learn/nature/volcano.htm
- Ledger ids: HNT-063
- Library: saved page, `NPS_YellowstoneVolcano/nps.txt`. Read: "the present 30- by 45-mile-wide Yellowstone Caldera" (631,000 years ago).

### PaulsonIngebritsen1991
- Reference: Paulson, K.M. and Ingebritsen, S.E. 1991. Sodium and chloride data from selected streams in the Lassen area, north-central California, and their relation to thermal-fluid discharge from the Lassen hydrothermal system. U.S. Geological Survey Water-Resources Investigations Report 90-4201. https://pubs.usgs.gov/wri/1990/4201/report.pdf
- Ledger ids: GEO-018
- Access: open (government report).
- Library: `PaulsonIngebritsen1991/report.pdf` and `report.txt` (text layer).
- Read in this pass: abstract and introduction: steam rises to discharge at 1,800 to 2,500 m, fumaroles and steam-heated acid-sulfate springs there, high-chloride springs at about 1,500 m.

### WEC2013_Geothermal
- Reference: World Energy Council, 2013. World Energy Resources: Geothermal (chapter 9). https://www.worldenergy.org/assets/images/imported/2013/10/WER_2013_9_Geothermal.pdf
- Ledger ids: GEO-012, GEO-013
- Access: open.
- Library: `WEC2013_Geothermal/wec.pdf` and `wec.txt`.
- Read in this pass: the passage saying high-temperature fields (above 180 degC) lie along plate boundaries (the Pacific ring of fire, Iceland on the Mid-Atlantic Ridge, the East African Rift Valley) and at hotspots such as Hawaii and Yellowstone.

### NASAEarthFactSheet
- Reference: NASA Space Science Data Coordinated Archive, Earth Fact Sheet. https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html
- Ledger ids: HNT-001, HNT-002, HNT-003, HNT-004
- Library: `NASAEarthFactSheet/earthfact.html` and `earthfact.txt`. Read: mean surface gravity 9.820 m/s2, standard gravity 9.80665 m/s2 (CODATA 2018), day 24.0000 h, sidereal rotation 23.9345 h, obliquity 23.44 deg, solar irradiance 1361.0 W/m2.

### NOAA_OceanFraction
- Reference: NOAA National Ocean Service, "How many oceans are there?". https://oceanservice.noaa.gov/facts/howmanyoceans.html
- Ledger ids: HNT-005
- Library: `NOAA_OceanFraction/noaa.html` and `noaa.txt`. Read: the ocean "covers 71 percent of the Earth".

### USGS pages read through a fetch tool (no saved copy)
- USGS Yellowstone Volcano Observatory, "Diverse chemistry of Yellowstone's hydrothermal features" (GEO-022): silica sinter in the geyser basins, travertine at Mammoth.
- USGS, "Alterations to go! Hydrothermal alteration in Yellowstone" (GEO-039): opal, kaolinite, alunite, pyrite; yellow, orange and red altered rhyolite.
- USGS Hawaiian Volcano Observatory, "Volcano Watch: Many forms of sulfur are found on Kilauea Volcano", 2018 (GEO-037): SO2 + 2H2S = 3S + 2H2O; stable as a solid below 115 degC.
- USGS Hawaiian Volcano Observatory, "Volcano Watch: How high is Mauna Loa?" (HNT-016, HNT-059): 4,170 m above sea level, flanks continue 5,000 m below.
- USGS, "Geology of Crater Lake National Park" (HNT-062): a basin "measuring 8-by-10-km (5-by-6-mi) across and more than 1 km deep".

### Seen in search summaries only (not read)
- Hurwitz, S. and Lowenstern, J.B. 2014. Dynamics of the Yellowstone hydrothermal system. Reviews of Geophysics 52:375-411, doi 10.1002/2014RG000452 (abstract only; the publisher blocks scripted downloads). GEO-007.
- White, D.E. 1967, mercury deposits and hot springs; the Steamboat Springs and Sulphur Bank deposits (GEO-041).
- Etheridge et al. 1996 (HNT-050); U.S. Standard Atmosphere 1976, NASA TM-X-74335 (HNT-043); Bouvier and Wadhwa 2010, Nature Geoscience 3:637 (HNT-054).

