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

394 works (the margins and spires task added Pratson1996 and the cold wetland task Xu2018, 2026-10-03; P1-26 T6 added HEXTOR, Kopparapu2013, Abbot2012, Hu2011 and Pierrehumbert2011 the same day: the tables below were not recounted for them). Stage 1 listed 475 entries, the flexure feature added 13 and the force balance (task 5, 2026-10-02) 2 (Cloos1993, Coblentz1994, both from memory); 103 entries were merged into 385 works (the merged keys are listed below).

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
- Read in stage 2 (WP-1, 2026-10-02): full text read again by the stage 2 agent (text extracted from the open copy): every equation the code uses was read: eq. 1, 21, 22, 23, 24, 25, 26, 27, 28 (d_fr = 0.4 D_fr^0.3 km, from Herrick et al. 1997), 29, 30, 31, 43-48, Table 6 (final craters 1.2, 23.7 and 186 km; energies 1.3e16, 1.5e21, 1.65e24 J) and the target densities and speeds of pp. 818-819. Confirmed: the stage 1 findings (eq. 28 is 0.4 D^0.3, not 0.294 D^0.301; eq. numbers 48, 47, 30 for rim, ejecta, melt). The code now reproduces Table 6 in a test.
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
- Read in stage 2 (WP-1, 2026-10-02): still not fetched (a book). Its value 5.6 +/- 2.8e-15 per km2 per year for D >= 20 km was read as quoted in Hughes 2000, p. 434 (see Hughes2000). Not available online in any copy tried (archive.org lending copy, no text).
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
- Read in stage 2 (WP-1, 2026-10-02): not fetched (Springer 403, ADS refused). The constancy of the rate was read as reported by Hughes 2000, p. 429, from Neukum & Ivanov 1994: 'very nearly constant for about the last 2000 Myr'.
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
- Read in stage 2 (WP-1, 2026-10-02): read through OCR of the open ADS scan (25 pages): abstract (iron meteorites; 1 km craters on land every 50,000 years); p. 608 Fig. 1 caption ('departure from the expected size-frequency distribution in the terrestrial crater data is evident in craters <10-20 km in diameter'); p. 611 (18 km/s and 45 degrees, citing Gilbert 1893 and Shoemaker 1962); p. 617 and p. 624 (16 of 17 craters under 1.5 km with a known impactor are iron; the proportion of stones returns above 10 km); p. 625 (Meteor Crater-sized craters every 20,000 years on the whole Earth). Confirms IMP-014 and supports IMP-004.
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
- Read in stage 2 (WP-1, 2026-10-02): not fetched (HAL access denied, 2026-10-02; Unpaywall lists the HAL copy as the only open one).
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
- Read in stage 2 (WP-1, 2026-10-02): not fetched. Davison & Collins 2007 (full text) use its pi-group scaling to derive their eq. 2 and give its constants as 'beta = 0.22 for solid rock in the gravity regime'.
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
- Read in stage 2 (WP-1, 2026-10-02): full text read again through the OCR layer (stage 2): pp. 247-248 (morphology; simple d_t = 0.29 D^0.98 and d_a = 0.14 D^1.02 km, complex d_a = 0.27 D^0.16 km, transition about 2 km in sedimentary and 4 km in crystalline targets), p. 253 (melt sheets 'up to several hundred meters thick'), the section on the known record (N proportional to D^-1.8 for large diameters; largest ~140 km) and 'Cratering Rate Estimates' (5.4 +/- 2.7e-15 per km2 per year for D >= 20 km, n = 7, Grieve 1984; 2.2 +/- 1.1e-14 for D >= 10 km, Shoemaker 1977). No 3.2 km here.
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
- Read in stage 2 (WP-1, 2026-10-02): fetched (open ADS scan, 31 pages) and read by OCR: p. 2165 (mean transition diameters 3.1 km for Earth and 18.7 km for the Moon, 'exactly the ratio of their values of g'; log D_t = log 3300 - 1.00 log g; Mars 6 km and Mercury 16 km deviate; complex craters six times deeper on the Moon, depth 'could well scale inversely with g'), Table 2 (Earth simple d = 0.137 D^1.02, complex d = 0.23 D^0.24 km), Fig. 4 (transition 2.25 km sedimentary, 4.75 km crystalline).
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
- Read in stage 2 (WP-1, 2026-10-02): not fetched (GeoScienceWorld 403). Not supported by what was read about peak-ring onsets elsewhere (see WoodHead1976).
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
- Read in stage 2 (WP-1, 2026-10-02): fetched (open ADS scan) and read by OCR: Table 5, onset diameters of central-peak and peak-ring basins (peak ring: Mercury 120, Mars 145, Moon 175 km) and the discussion that gravity is not the dominant control. Does not give the ring-to-rim diameter ratio.
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
- Read in stage 2 (WP-1, 2026-10-02): not fetched. The ratio of a ring's diameter to the rim's was read instead in Pike 1985.
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
- Read in stage 2 (WP-1, 2026-10-02): fetched (open ADS scan, image only) and read by OCR (20 pages): abstract and Table 1. Central-peak base is 20-25 % of the rim diameter in small complex craters (half that in ringed ones); the inner ring of a two-ring crater without a peak is half the outer's diameter (0.5, not the planets' 0.4, when there is a peak); Vredefort 150 +/- 10 km.
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
- Read in stage 2 (WP-1, 2026-10-02): not fetched (ADS refused). Read as cited by Collins et al. 2005, p. 828: 'the power of -3 is a good approximation of data from explosion experiments (McGetchin et al. 1973)'.
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
- Read in stage 2 (WP-1, 2026-10-02): fetched (open ADS scan) and read by OCR: p. 83, eq. 1: the radius of continuous ejecta R_ce = 2.348 R_c for lunar craters of 0.65-218 km across (the 2.3 of the code).
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
- Read in stage 2 (WP-1, 2026-10-02): fetched (open ADS scan) and its abstract read by OCR; the body of the scan was not read for numbers. It confirms that melt volume rises faster than the transient cavity's with event size.
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
- Read in stage 2 (WP-1, 2026-10-02): full text read again (stage 2): Table 1 (iSALE model parameters), eq. 1-3 (eq. 3: D_f,wet / D_f,dry = 1 - 0.14 R), Table 2 (the runs), p. 1922 (3.2 km and R = 3.5 resurge rule), p. 1925 (no crater above R = 8), eq. 4-8 and Table 3. The code's ocean rule is now their eq. 3 and regime limits.
- Reference check: the citation is right. The code's note that it fits "three numbers" is accurate for the abstract. The paper's own cutoff is not 6.3 (Part 2).

### Hughes2000
- Reference: Hughes, D. W. (2000). A new approach to the calculation of the cratering rate of the Earth over the last 125 +/- 20 Myr. Monthly Notices of the Royal Astronomical Society 317(2):429-437.
- DOI or URL: DOI not checked. Open ADS scan with a text layer (bibcode 2000MNRAS.317..429H, https://articles.adsabs.harvard.edu/pdf/2000MNRAS.317..429H).
- Cited as: not cited in the code before stage 2; now cited in scaling.rs for the shape of the production function below 20 km and for the quotation of the Grieve & Shoemaker rate.
- Ledger ids: IMP-001, IMP-002, IMP-003, IMP-004, IMP-005
- Access: open copy (ADS scan, 9 pages)
- Read so far (by the project): not used before stage 2.
- Read in stage 2 (WP-1, 2026-10-02): full text read. Abstract: rates of craters larger than D over the last 125 +/- 20 Myr on the stable continents: 12.0 +/- 0.7 (D > 2.4 km), 9.5 +/- 0.6 (5 km), 6.5 +/- 0.5 (12 km), 3.0 +/- 0.3 (22 km) x 1e-15 per km2 per year, and 3.46 +/- 0.30 for D > 20 km; 'previous researchers have considerably overestimated the rate at which small 2.4 < D < 20 km craters are being produced'; 'the relationship between crater production rate and crater diameter is not a simple power law in the 2.4 < D < 40 km diameter range'. p. 434: earlier D > 20 km rates 'Hughes (1981) 2.6 +/- 0.9, Grieve & Dence (1979) 3.5 +/- 1.3, Grieve (1984) 5.4 +/- 2.7 and Grieve & Shoemaker (1994) 5.6 +/- 2.8 x 1e-15 km-2 yr-1'. Table 2: the full rate table. p. 429: the lunar rate (Neukum & Ivanov 1994) 'very nearly constant for about the last 2000 Myr'.
- Reference check: matches. His figures are for craters surviving on stable cratons; he argues the completeness is near 1 so they are production rates, but they could be lower limits.

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
- Read in stage 2 (WP-1, 2026-10-02): not fetched (paywalled).
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
- Read in stage 2 (WP-1, 2026-10-02): not fetched (Wiley 403).
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
- Read in stage 2 (WP-1, 2026-10-02): abstract only (OpenAlex), unchanged; the paper is paywalled (Unpaywall: not open).
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
- Read in stage 2 (WP-1, 2026-10-02): not fetched. Unpaywall lists an open copy at the publisher (Wiley), but a script is refused (403, 2026-10-02) and so is the web fetch tool.
- Reference check: matches. Whether it gives the 0.003-0.05 m^2/yr range is unchecked.

### Roering1999
- Reference: Roering, J. J., Kirchner, J. W., and Dietrich, W. E. (1999). Evidence for nonlinear, diffusive sediment transport on hillslopes and implications for landscape morphology. Water Resources Research 35(3):853-870.
- DOI or URL: (IMP) 10.1029/1998WR900090 (Crossref-verified) (ERO, ISO, HYP) 10.1029/1998WR900090.
- Cited as: (IMP) "Roering, Kirchner & Dietrich 1999" (ERO, ISO, HYP) "Roering, Kirchner & Dietrich 1999, WRR 35(3):853-870" (rock_class JSON), "hillslope transport coefficient D". (RCK, STR, DEF) "Roering, Kirchner & Dietrich 1999 (WRR 35(3):853-870)"
- Ledger ids: ERO-101 to ERO-150, ERO-153 to ERO-163, ERO-177, IMP-074, IMP-085, IMP-086, IMP-089, IMP-090, RCK-010
- Read in the plateau erosion task (2026-10-16): full text, from the second author's reprint at https://seismo.berkeley.edu/~kirchner/reprints/1999_29_Roering_nonlinear.pdf (text extracted with pdftotext). The paper's numeric D values are now read: the nonlinear law fitted to Oregon Coast Range topography gives "K = 0.0032 ± 0.0009 m2/yr and Sc = 1.25 ± 0.10"; per hillslope (Table 1) "Values for K range from 0.0031 to 0.0045 m2/yr with an average of 0.0036 m2/yr"; with other fitting choices "K varies between 0.0015 and 0.0045 m2/yr, and Sc varies between 1.0 and 1.4". ERO-177.
- Access: paywalled (metadata only) [other sections: paywalled (Wiley); not fetched. / paywalled (abstract only)]; the author's reprint above is open.
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): (IMP) from memory (params.rs); judgment framework (rock-class JSON) (ERO, ISO, HYP) no record. (RCK, STR, DEF) not stated
- Read in this pass: (IMP) not fetched (ERO, ISO, HYP) not fetched (403 / no open copy). (RCK, STR, DEF) abstract fetched (OpenAlex)
- Read in stage 2 (WP-1, 2026-10-02): not fetched. Unpaywall lists an open copy at the publisher (Wiley), but a script is refused (403, 2026-10-02) and so is the web fetch tool.
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
- Read in WP-7 (2026-10-02): still a landing page (WP-3 owns the soil production rows).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Osinski2013/Osinski2013.txt, the paragraph on the duration of impact hydrothermal systems.

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
- Read in the carbonate and ore-deposit task (2026-10-19, ledger IMP-142, IMP-143): the abstract through OpenAlex (https://api.openalex.org/works/doi:10.1080/00206819409465452), by the task's research sub-agent: "Approximately 25% of the known terrestrial impact craters are associated with some form of economic resources and ~12% currently are exploited"; "iron and uranium ore exposed at Ternovka ... and Carswell (Saskatchewan)", both progenetic, exploitable "because of uplift in the center of complex impact structures" (the abstract places Ternovka in Russia; it is in Ukraine). The paper itself was not read.

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
- Cited again in the carbonate and ore-deposit task (2026-10-19, ORE-042, IMP-142) for the Sudbury hydrothermal system as the model's hydrothermal driver; not read.

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

### Edmonds2019
- Reference: Edmonds, M., Cashman, K. V., Holness, M. and Jackson, M. (2019). Architecture and dynamics of magma reservoirs. Philosophical Transactions of the Royal Society A 377(2139): 20180298.
- DOI or URL: 10.1098/rsta.2018.0298; https://pmc.ncbi.nlm.nih.gov/articles/PMC6335485/
- Access: open copy on PubMed Central; the publisher PDF returned 403 to scripted access.
- Library (the project's source library, not distributed): not fetched as a file (the PDF endpoints returned HTTP 403 or an HTML page; the PMC XML endpoint returned a server error).
- Read in this pass (2026-10-02): the PMC page through a page-fetch tool that returns a model's answer to a question, not the text; quotes recorded in the ledger are from that answer and were not checked against the PDF: "vertical extent of the crust ... 'trans-crustal mush system'"; "water-rich arc magmatic systems are typically vertically extensive and dominated by mushy super-solidus conditions"; sill thicknesses "must be less than 10 m thick"; "multiple, 'stacked' storage areas for magma...separated by subsolidus material"; "5-15% melt" in upper crustal reservoirs.
- Ledger ids: MAG-043, MAG-044, MAG-047, MAG-054
- Used for: the architecture of the crust-deep column.

### Cashman2017
- Reference: Cashman, K. V., Sparks, R. S. J. and Blundy, J. D. (2017). Vertically extensive and unstable magmatic systems: a unified view of igneous processes. Science 355(6331): eaag3055.
- DOI or URL: 10.1126/science.aag3055
- Access: paywalled; no open copy reached.
- Library (the project's source library, not distributed): not fetched.
- Read in this pass: nothing; named in Ben's decision (2026-10-02) and in the ledger as the picture followed. Not cited for any number.
- Ledger ids: MAG-043
- Used for: the idea of a vertically extensive system.

### Sparks2019
- Reference: Sparks, R. S. J., Annen, C., Blundy, J. D., Cashman, K. V., Rust, A. C. and Jackson, M. D. (2019). Formation and dynamics of magma reservoirs. Philosophical Transactions of the Royal Society A 377(2139): 20180019.
- DOI or URL: 10.1098/rsta.2018.0019
- Access: not open on PubMed Central (Europe PMC flags it not open access).
- Library (the project's source library, not distributed): not fetched.
- Read in this pass: nothing; named in Ben's decision.
- Ledger ids: MAG-043
- Used for: the idea of a vertically extensive system.

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
- Ledger ids: PRV-006 (REV-2, 2026-10-19).
- Read in REV-2 (2026-10-19): the abstract on ADS, for the definition: over 0.1 Mkm2 in area and 0.1 Mkm3 in volume, mainly mafic, emplaced within about 50 Myr with most of the volume (over 75 %) in pulses of 1 to 5 Myr; seamounts, hotspot tracks and normal oceanic crust are excluded. The library's full text was not reopened.

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
- Ledger ids: VOL-016, VOL-017, VOL-019, VOL-020, VOL-026, VOL-029, VOL-030, VOL-091; HNT-016, HNT-085, HNT-086
- Also used 2026-10-22 (crater depth, D7): p. 8 read again: crater depths under 100 to 860 m, median 240 m, no correlation between crater depth and edifice height, depth to width 0.01 to 0.29, median 0.11; crater widths 0.6 to 11 km, median 2.2 km, correlated with the basal and summit widths. A copy of the submitted version is in the VM's library again (`Grosse2014/`).
- Also used 2026-10-02 (volcano decisions): table 1 re-read for the target of VOL-019 (stratovolcano height median 1.02 km, mean 1.13, s.d. 0.55, range 0.17-4.57) and VOL-020 (height/basal width median 0.13, range 0.02-0.30; basal width median 7.9 km).
- Access: free to read at the publisher (bronze); the publisher returned a page, not the PDF; the authors' submitted version is in the CONICET repository
- Library (the project's source library, not distributed): fetched WP-2 2026-10-02 (PDF, 23 pages, text layer). From: https://ri.conicet.gov.ar/handle/11336/7280 (submitted version; the pages carry the journal's running heads). Text file: Grosse2014.txt.
- Read in WP-2 (2026-10-02), full text: 759 composite volcanoes from the 90-m SRTM model (those of the Global Volcanism Program list it covers, with a basal width over 2 km). Table 1 (p. 7): stratovolcanoes (n = 512): basal width median 7.9 km (range 2.0-65.9), height median 1.02 km, mean 1.13, s.d. 0.55 (range 0.17-4.57), height/basal width median 0.13 (0.02-0.30), mean flank slope 17.7 degrees, main flank 21.5, steepest average slope 27.7 degrees at 0.79 of the height; shields (n = 89): basal width median 15.3 km, height 0.81 km, height/width 0.06, flank slope 8.1 degrees. p. 8: crater and caldera data only for 104 large ones: 0.6-11 km across, median 2.2 km; depth under 100 to 860 m, median 240 m. Subaerial parts only.

### ClagueSherrod2014
- Reference: Clague, D. A. and Sherrod, D. R. (2014). Growth and degradation of Hawaiian volcanoes. Chapter 3 in Poland, M. P., Takahashi, T. J. and Landowski, C. M. (eds), Characteristics of Hawaiian Volcanoes, U.S. Geological Survey Professional Paper 1801, pp. 97-146.
- DOI or URL: 10.3133/pp18013; https://pubs.usgs.gov/pp/1801/downloads/pp1801_Chap3_Clague.pdf
- Cited as: "Clague & Sherrod 2014" (added by WP-2 in comments)
- Ledger ids: VOL-007, VOL-009, VOL-017, VOL-021, VOL-041, VOL-048, VOL-058, MAG-002; HNT-087
- Access: open access (agency report)
- Library: fetched WP-2 2026-10-02 (PDF, text layer, 343,082 characters). Text file: ClagueSherrod2014.txt.
- Read in WP-2 (2026-10-02), full text: abstract (p. 97): "Principal growth occurs in the first 1-2 million years"; "Shallow magma reservoirs (1-7-km depth)" of shield-stage volcanoes, "smaller reservoirs at 20-30-km depth persist". p. 99: "The sizes and spacing of the volcanoes are nonuniform"; only eight volcanoes northwest of the main islands grew to 1,500 m or more above sea level. p. 114: lava flows on a shield dip 2-10 degrees, submarine slopes are steeper. pp. 130-131: subsidence under the growing volcano's load "lasts perhaps 1 m.y." and sinks shorelines "by more than 1 km" (2.6 m/kyr off the Island of Hawaii); thermal subsidence of the plate along much of the chain "~0.01 m/k.y."; reefs drown about 33 Myr after their volcano formed. Mauna Loa's summit 4,169 m.

### Christiansen2001
- Reference: Christiansen, R. L. (2001). The Quaternary and Pliocene Yellowstone Plateau volcanic field of Wyoming, Idaho, and Montana. U.S. Geological Survey Professional Paper 729-G.
- DOI or URL: 10.3133/pp729G; https://pubs.usgs.gov/pp/pp729g/pp729g.pdf
- Cited as: "Christiansen 2001" (added by WP-2 in comments and hints)
- Ledger ids: VOL-006, VOL-023, VOL-027, VOL-029, VOL-045, VOL-046, VOL-074; HNT-060, HNT-063, HNT-088
- Also read 2026-10-02 (volcano decisions), the summary and the first sections: the field has three volcanic cycles, each climaxed by a caldera-forming ash-flow eruption: the Huckleberry Ridge Tuff at 2.1 Ma (more than 2,450 km3 over 15,500 km2, a caldera more than 75 km long), the Mesa Falls Tuff at 1.3 Ma (more than 280 km3, a caldera about 16 km across) and the Lava Creek Tuff at 640 ka (more than 1,000 km3, the Yellowstone caldera): three in about 1.5 Myr (VOL-074).
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
- Ledger ids: AMP-016, AMP-017, ERO-080, ERO-184, HYD-014
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
- Read so far (by the project): not read (403). Ruetenik et al. 2023 (read 2026-10-18) report its n as 2.6 ("Our optimised n∼1.3 ... is much lower than the 2.6 reported by Harel et al. (2016)"), a secondary reading only.
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
- Ledger ids: ISO-018, ISO-020, ISO-022; ISO-070 (from memory: the law is derived with local isostasy); PLT-300, PLT-301 (2026-10-20: the channel flow out of Tibet and its 15 km channel, from memory); PLT-414 (2026-10-22: the channel viscosities, abstract)
- Access: paywalled (GeoScienceWorld 403); not fetched. [other sections: paywalled (GSA; not tested).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Merged keys: Clark2000
- Read so far (by the project): (ERO, ISO, HYP) bibliographic check only. (PLT) not stated; not in code.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) metadata only.
- Reference check: (ERO, ISO, HYP) Crossref matches title, authors, volume 28, issue 8, first page 703 (the end page 706 is not in the Crossref record, taken as written). Attribution of "D0 range 10^2 to 10^5" to this paper cannot be checked. (PLT) correct.
- Read in the high-ground task (2026-10-22, PLT-414): the abstract from OpenAlex's record. "For an assumed 15-km-thick channel", viscosities of "10^18 Pa · s beneath the low-gradient margins, 10^21 Pa · s beneath the steep margins" and "an upper bound of 10^16 Pa · s beneath the plateau" (the record drops the exponents' superscripts). The 15 km channel of PLT-301 is now read in the abstract. The abstract gives no diffusivity: the 10^2 to 10^5 km²/Myr range follows from the channel law K = ρ g h³ / (12 μ) with these viscosities (2.4 × 10^5 km²/Myr at 10^18 Pa s, 240 at 10^21), the project's arithmetic (plates.md, "As a thin viscous sheet").

### Royden1997
- Reference: Royden, L. H., Burchfiel, B. C., King, R. W., Wang, E., Chen, Z., Shen, F. and Liu, Y. (1997). Surface deformation and lower crustal flow in eastern Tibet. Science 276(5313):788-790.
- DOI or URL: 10.1126/science.276.5313.788.
- Cited as: (ERO, ISO, HYP) "Royden et al. 1997, Science 276:788-790". (PLT) "Royden et al. 1997".
- Ledger ids: ISO-018, ISO-020; PLT-300 (2026-10-20, as the channel flow's background; not fetched)
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
- Read in the high-ground task (2026-10-22, PLT-414): the abstract, from OpenAlex's record (the full text stays paywalled): the lithosphere as "a thin sheet of power law material overlying an inviscid substrate"; "The material flows in response to forces applied to its boundaries (for example, the indenting of one continent by another) and to forces in its interior arising from gradients in crustal thickness"; "In the limit of a very weak medium (Ar > 10) the lithosphere is unable to sustain appreciable crustal elevation contrasts"; "the forces arising from crustal thickness contrasts are great enough to produce net extension in the region of thick crust in front of the influx boundary". Ledger ids added: PLT-414.

### MolnarLyonCaen1988
- Reference: Molnar, P. and Lyon-Caen, H. (1988). Some simple physical aspects of the support, structure, and evolution of mountain belts. In: Clark, S. P., Burchfiel, B. C. and Suppe, J. (eds), Processes in Continental Lithospheric Deformation, Geological Society of America Special Paper 218, pp. 179-208.
- DOI or URL: 10.1130/SPE218-p179.
- Cited as: (ERO, ISO, HYP) "Molnar & Lyon-Caen 1988, GSA Special Paper 218:179-207". (PLT) "Molnar & Lyon-Caen 1988 (GSA Spec. Pap. 218)", "pages 179-207" in the force-balance design (Tibet 6-7e12 N/m).
- Ledger ids: ISO-019, ISO-020, PLT-187, FRC-014
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
- Ledger ids: ISO-019, ISO-020, PLT-255 (orogenic collapse, 2026-10-03: the abstract's "cannot maintain short-wavelength (100 km) Moho relief of more than 1 km for times greater than 10–20 m.y. at most", Basin and Range); PLT-300 (2026-10-20: the same abstract quote beside the lower-crustal flow); PLT-414 (2026-10-22: leveling and the pancake, abstract)
- Access: paywalled; not fetched. [other sections: paywalled (abstract only).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (ERO, ISO, HYP) bibliographic check only. (PLT) not stated; not in code.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) abstract fetched (Crossref). The abstract states that flux is proportional to the third power of the topographic gradient (cubic power law creep), which matches the project's "channel law cube".
- Reference check: (ERO, ISO, HYP) Crossref matches volume 96, issue B6, pages 10275-10286 (2026-10-01 Crossref query on the title). (PLT) correct.
- Read in the high-ground task (2026-10-22, PLT-414): the abstract from OpenAlex's record. Lower-crustal extrusion "removes crust from under mountains and smooths and levels the topography"; an isolated high becomes "pancake shaped, with a flat central plateau and steeper flanks spreading outward", and "When adjacent 'pancakes' merge, most of the relief is eliminated"; with published flow laws extrusion is negligible under oceans, marginal under shields and platforms, important under elevated plains and "dominant beneath high plateaus and/or hot, delaminated regions"; a delaminated Tibet may reduce "100-km-wavelength topographic features to 200 m relief in no more than 0.03-0.13 m.y.", "leaving only features with wavelengths over 400 km at the present".

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
- Read in the high-ground task (2026-10-22, PLT-412): the abstract, verbatim, from the University of Minnesota's research portal (Experts@Minnesota, https://experts.umn.edu/en/publications/gravitational-collapse-of-the-continental-crust-definition-regime/; the full text stays paywalled). "Gravitational collapse corresponds to the decay of lateral contrast in gravitational potential energy that builds up during lithospheric deformation"; "When the surrounding lithosphere is fixed (fixed-boundary collapse), collapse occurs through a transfer of gravitational potential energy from the elevated regions towards the low lands. This transfer is accommodated by a combination of gravitational sliding of the brittle crust and horizontal spread of the lower crust. In orogenic domains, fixed-boundary divergent collapse implies the lateral growth of the orogenic domain at the expense of the surrounding lithosphere"; and of convergent collapse, "a deficit in gravitational potential energy drives crustal material towards the deformed lithosphere. This regime can be expected to occur following thinning of the continental crust." Ledger ids added: PLT-412.

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
- Read 2026-10-21 (sea level from the ocean's volume; ledger PLT-351, PLT-352): the paper still not read (paywalled). A research sub-agent read Stein 1995 (below) for the ridge depth 2,600 m and the heat-flow law's decay 0.0278; the depth law's 365, 5,651, 2,473 and its 20-Myr knee were found only on a secondary page that cites the paper (github.com/Elkhan-Isayev/pangaea-drift).

### ParsonsSclater1977
- Reference: Parsons, B. and Sclater, J. G. (1977). An analysis of the variation of ocean floor bathymetry and heat flow with age. Journal of Geophysical Research 82(5):803-827.
- DOI or URL: 10.1029/JB082i005p00803.
- Cited as: (ERO, ISO, HYP) "Parsons & Sclater 1977 (mean ridge crest depth 2.5 km)" (build log). (PLT) "Parsons & Sclater 1977" (ridge crest depth 2.5 km); no title or journal given in the code.
- Ledger ids: ISO-014, PLT-204, ISO-099
- Access: paywalled; not fetched. [other sections: paywalled (no abstract fetched).]
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Merged keys: Parsons1977
- Read so far (by the project): (ERO, ISO, HYP) no record. (PLT) not stated.
- Read in this pass: (ERO, ISO, HYP) not fetched. (PLT) not fetched itself; its formulas were read as reproduced in King & Adam 2014 (a secondary open copy) and Wikipedia "Seafloor spreading".
- Read in the release-pace task (2026-10-23, the abstract through OpenAlex, by a Sonnet sub-agent of the release agent; ISO-099): "Inverting the data gives a plate thickness of 125±10 km, a bottom boundary temperature of 1350°±275°C"; "for regions older than 20 m.y. B.P. by the relation d(t) = 6400 - 3200 exp (−t/62.8) m"; "the heat flow only responds to the bottom boundary at approximately twice the age at which the depth does".
- Reference check: (ERO, ISO, HYP) Crossref matches the title, volume 82, issue 5, pages 803-827. The sources file gives no year-title pair; the 1977 paper is the correct identification. (PLT) now identifiable. Identification only; the "2.5 km" is the intercept of the square-root law (see Part 2).
- Read 2026-10-21 (ledger PLT-352, as the cross-check): the scanned first page at https://topex.ucsd.edu/geodynamics/parsons_sclater77.pdf, by OCR: "Between 0 and 70 m.y. B.P. the depth can be represented by the relation d(t) = 2500 + 350 t^1/2 m ... and for regions older than 20 m.y. B.P. by the relation d(t) = 6400 - 3200 exp(-t/62.8) m" (OCR gave 6400 as "6L00"; Wikipedia's secondary copy resolves it).

### Cogley1984
- Reference: Cogley, J. G. (1984). Continental margins and the extent and number of the continents. Reviews of Geophysics 22(2):101-122.
- DOI or URL: 10.1029/RG022i002p00101.
- Cited as: (ERO, ISO, HYP) "Cogley 1984 (continental crust 41 % of the surface)". (PLT) "Cogley 1984, Rev. Geophys. 22:101-122" (29 percent land on 41 percent continental crust).
- Ledger ids: PLT-516 (2026-10-23: the 41 % against the transference arm's continental share); RLC-030 (2026-10-23: the 41 % crust share, as the realism-checks page uses it); HYP-012, HYP-013, PLT-158, PLT-159, PLT-205; HYP-035 (2026-10-20: the 41 % share as the stand-in contour for the continental edge); HYP-042 (2026-10-22: the 30.6 % submerged against the drowned-crust target)
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
- Ledger ids: BAK-020, BIO-001 to BIO-017, BIO-158 (EF under the ice cover, through BIO-003), HYD-046, HYP-025; none (comparison only); notes: BIO-017 is a game-choice about the summer half; and BIO-001 to BIO-016; comparison only; supporting source found in this pass
- Access: open access. [other sections: open access (gold)]
- Library (the project's source library, not distributed): fetched (PDF, 12 pages, text layer, 46,987 characters). From: https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf. Text file: Peel2007.txt.
- Merged keys: Peel2007 (Koppen-Geiger)
- Read so far (by the project): (ERO, ISO, HYP) no record. (HYD, YNG) n/a here (MOI, BIO) nothing stated; the research note it points to cites Wikipedia, not this paper (SCL, PRV, POL, BAK) not stated (CLM) the shares were "quoted in the 2026-09-29 P1-17 entry"
- Read in this pass: (ERO, ISO, HYP) abstract fetched; PDF text partly extracted (title page, abstract, methods, Africa section). The page with global main-group shares was not located. (HYD, YNG) full text fetched (Table 1 with notes, the paragraphs on test order and on the Cs/Cw tie rule, the discussion with land shares) (MOI, BIO) full text fetched (decoded PDF text layer: Table 1 with its notes, the paragraphs on test order and on the Cs/Cw tie rule, the discussion with the land shares) (SCL, PRV, POL, BAK) abstract fetched (landing page); PDF returned as binary so Table 1 not read; criteria were taken from Wikipedia's Koppen article instead (secondary) (CLM) full text fetched (section 4 conclusions)
- Reference check: (ERO, ISO, HYP) Crossref matches title, volume 11, issue 5, pages 1633-1644. The abstract in the HTML page says "BWh (14.2%, Hot desert) followed by Aw (11.5%, Tropical savannah)"; the build log's shares B 30.2 / D 24.6 / A 19.0 / C 13.4 / E 12.8 % were not found in the text read (S19). (HYD, YNG) matches. See the MOI file for the full Table 1 check. (MOI, BIO) authors, year, journal, volume, pages match. Table 1 is the table the code cites. The research note's source (Wikipedia) differs from the paper in the B threshold's form only (20 x MAT + 280/140/0 against 10 x (2 x MAT + 28/14/0), the same numbers) and in listing a "-3 degC" option the paper does not have. Details in Part 2. (SCL, PRV, POL, BAK) the code names no source. Peel et al. 2007 and Kottek et al. 2006 are the standard references per Wikipedia. (CLM) matches.
- Cited as (HYD): "Koppen's E (polar climate: warmest month below 10 degC)", no paper named
- Stage 2 read (WP-11, 2026-10-02, Peel2007.txt): full text. Table 1 and its notes, the text on test order (p. 1637), the Cs/Cw tie rule (p. 1639), Table 2 (Herberton Post Office) and the world shares (p. 1642). All BIO-001 to BIO-017 rows checked; BIO-002 and BIO-013 contradicted the code and were fixed.

### CharetteSmith2010
- Reference: Charette, M. A. and Smith, W. H. F. (2010). The volume of Earth's ocean. Oceanography 23(2):112-114.
- DOI or URL: (ERO, ISO, HYP) 10.5670/oceanog.2010.51; open PDF https://tos.org/oceanography/assets/docs/23-2_charette.pdf. (PLT) 10.5670/oceanog.2010.51.
- Cited as: (ERO, ISO, HYP) "Charette & Smith 2010, Oceanography 23(2):112-114 (ocean volume and mean depth on SRTM30_PLUS)". (PLT) "Charette & Smith 2010, Oceanography 23(2):112-114" (sea mean 3,682 m; 361.84 million km2 = 70.9 percent).
- Ledger ids: HYP-007, PLT-097, PLT-169, PLT-170, PLT-362 (2026-10-21: Earth's water for the volume-balanced sea level's test numbers; the ocean's 1.332e9 km3 cited, not re-read)
- Access: open access. [other sections: open access (Oceanography Society).]
- Library (the project's source library, not distributed): fetched (PDF, 3 pages, text layer, 10,430 characters). From: https://tos.org/oceanography/assets/docs/23-2_charette.pdf. Text file: CharetteSmith2010.txt.
- Merged keys: Charette2010
- Read so far (by the project): (ERO, ISO, HYP) no record. (PLT) not stated.
- Read in this pass: (ERO, ISO, HYP) full text fetched (text of the 3-page article including Tables 1 and 2). (PLT) metadata only (OpenAlex abstract is only a copyright line).
- Reference check: (ERO, ISO, HYP) Crossref matches title, volume 23, issue 2, pages 112-114. The article says: "For this effort, we relied on the SRTM30PLUS V5 bathymetry database (Becker et al., 2009)", so the project's "on SRTM30_PLUS" is right. (PLT) correct. Numbers not read.
- Read 2026-10-21 (ledger ISO-085, CLM-106): the PDF, Table 2: "Mean or Total 3682.22 m, 361.841 (10^6 km2), 1.33238 (10^9 km3)", 1-sigma 1.2 % on depth and volume, 0.14 % on area.

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
- Ledger ids: RLC-003 (2026-10-23: the present-day control of the Earth-history pipeline, tools/review/earth-paleodem.py); PLT-513 (2026-10-23, the docking lane: Earth's land across a coast, tools/review/earth-across.py on the 15 arc-minute grid); HYP-001 to HYP-011, HYP-032, PLT-168, PLT-170; none (relief used in the Earth comparisons); notes: as the dataset behind GMT earth_relief; relief used in the Earth comparisons; HYP-035 (2026-10-20: the median distance from land to the coast on the model's sampling, computed on earth_relief_15m_p); PLT-360 (2026-10-21: the coastal high ground, the hypsometry and the highest 1,000 km box, computed on earth_relief_15m_p by tools/review/continents_census.py); HYP-050, HYP-051 (2026-10-22, the trench budget: the floor deeper than 6 km and the deepest cell on earth_relief_15m_p, tools/review/earth-deep-floor.py)
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
- Read in WP-7 (2026-10-02): the library copy is still a landing page; the density table was not read. Densities were checked against Daly, Manger and Clark 1966 and Manger 1963 instead (RCK-001, RCK-086, RCK-087).

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
- Read in WP-7 (2026-10-02): not read (book).

### Not cited by the code but named in the task brief (identified for stage 2)
- Whipple, K. X. and Tucker, G. E. (1999). Dynamics of the stream-power river incision model: Implications for height limits of mountain ranges, landscape response timescales, and research needs. Journal of Geophysical Research 104(B8):17661-17674. DOI 10.1029/1999JB900120 (Crossref confirmed). Paywalled; not fetched.
- Fernandes, N. F. and Dietrich, W. E. (1997). Hillslope evolution by diffusive processes: The timescale for equilibrium adjustments. Water Resources Research 33(6):1307-1318. DOI 10.1029/97WR00534 (Crossref confirmed). Paywalled; not fetched.
- Amante, C. and Eakins, B. W. (2009). ETOPO1 1 arc-minute global relief model: procedures, data sources and analysis. NOAA Technical Memorandum NESDIS NGDC-24. Dataset DOI 10.7289/V5C8276M (DataCite: "ETOPO1 1 Arc-Minute Global Relief Model", NOAA NCEI, 2009). Open.
- Eakins, B. W. and Sharman, G. F. (2012). Hypsographic curve of Earth's surface from ETOPO1. NOAA National Geophysical Data Center, Boulder, CO. PDF https://www.ncei.noaa.gov/sites/default/files/2023-01/Hypsographic%20Curve%20of%20Earth%E2%80%99s%20Surface%20from%20ETOPO1.pdf (full text read; Table 1 is an image and was not readable). Companion: "World Ocean Volumes" https://www.ncei.noaa.gov/sites/default/files/2023-01/World%20Ocean%20Volumes.pdf (table read).
- Athy, L. F. (1930). Density, porosity, and compaction of sedimentary rocks. AAPG Bulletin 14(1):1-24. DOI 10.1306/3D93289E-16B1-11D7-8645000102C1865D. Not coded: the surface stage has no porosity or compaction (ERO-028). Its exponential law was tried in the plate model's column on 2026-10-20 and reverted (isostasy-and-height-limit.md, "Known before the page is written"). Not read.
- Sclater, J. G. and Christie, P. A. F. (1980). Continental stretching: an explanation of the post-mid-Cretaceous subsidence of the central North Sea basin. Journal of Geophysical Research 85(B7):3711-3739. DOI 10.1029/JB085iB07p03711. Not coded, as above; its shaly sandstone was the 2026-10-20 trial's cover (paywalled, not read; the values read at second hand in pyBacktrack's lithology table, below, pyBacktrack_lithologies).

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
- Ledger ids: HYD-029, HYD-030, HYD-035, HYD-041, BIO-126 (the dry forest precipitation band, from memory: the band is read off the life-zone chart, which the scan's text layer does not carry; the text says "Most of these savannas are to be found in the Tropical Dry Forest Life Zone", read 2026-10-04)
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
- Ledger ids: BIO-083 (merged into BIO-082 2026-10-18, the fallback it supported removed), GLA-031, GLA-037, GLA-039, HYD-039; notes: and HYD-039 in the HYD file; and glaciers.rs
- Access: open access (Cambridge Core) [other sections: open access (Cambridge archive; the page shows the full text) / open access (page readable without login) / open access (bronze)]
- Library (the project's source library, not distributed): fetched (PDF, 15 pages, text layer, 55,097 characters). From: https://www.cambridge.org/core/services/aop-cambridge-core/content/view/8DCF1C24E32A6781CC365853535CD16F/S0022143000002276a.pdf/div-class-title-climate-at-the-equilibrium-line-of-glaciers-div.pdf. Text file: Ohmura1992.txt.
- Read so far (by the project): (HYD, YNG) "cited from memory, not re-read" (AMP, GLA) from memory, not re-read (STATUS 2026-10-01) (MOI, BIO) "cited from memory, not re-read" for the -2 degC figure (CLM) the build log says the equilibrium-line summer temperature was "cited from memory, not re-read" (hydrology agent's check of a related number)
- Read in this pass: (HYD, YNG) page fetched via summariser (equation and one sentence). My prompt contained the equation, so treat the quote as needing a second look. (AMP, GLA) full text partly fetched (page text via WebFetch: abstract, the regression sentence, the temperature definition, the standard error; Table 3 was not readable) (MOI, BIO) page fetched (summariser); see the HYD file for the check (CLM) full text fetched (OCR text layer; the regression equation itself is not machine-readable)
- Reference check: (HYD, YNG) authors, year, journal, volume and pages match. (AMP, GLA) authors, year, journal, volume, issue and pages match. (MOI, BIO) matches. (CLM) matches. Its T is the mean summer temperature in the FREE ATMOSPHERE at the ELA, not a surface or sea-level temperature (see Part 2).

### Messager2016
- Reference: Messager, Mathis L.; Lehner, Bernhard; Grill, Gunther; Nedeva, Irena; Schmitt, Oliver. 2016. Estimating the volume and age of water stored in global lakes using a geo-statistical approach. Nature Communications 7:13603.
- DOI or URL: https://doi.org/10.1038/ncomms13603 ; PMC https://pmc.ncbi.nlm.nih.gov/articles/PMC5171767/
- Cited as: "Messager et al. 2016, Nat. Commun. 7:13603 (HydroLAKES: 1.42 million lakes of 10 ha and more, 2.67 million km2, 1.8 % of land)"
- Ledger ids: HYD-063, HYD-090
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 11 pages, text layer, 59,360 characters). From: https://www.nature.com/articles/ncomms13603.pdf. Text file: Messager2016.txt.
- Read so far (by the project): 2026-10-20, the lakes task: the full text (PMC) for the lake count, area and land share, quoted in HYD-063; also "21.2 million natural lakes with a surface area of at least 0.01 km2 (1 ha)", "3.23 × 10^6 km2 or 2.2% of global land area". "Global land area" is not defined further in the text read.
- Read in this pass: full text fetched (PMC, summariser: lake count, area, land share, comparison with Verpoorter)
- Reference check: matches. Note HydroLAKES has 1.43 million polygons, of which 1.42 million are natural lakes (the 1.42 million is natural lakes only).

### Verpoorter2014
- Reference: Verpoorter, Charles; Kutser, Tiit; Seekell, David A.; Tranvik, Lars J. 2014. A global inventory of lakes based on high-resolution satellite imagery. Geophysical Research Letters 41(18):6396-6402.
- DOI or URL: https://doi.org/10.1002/2014GL060641 (hybrid open access per OpenAlex); HAL record https://insu.hal.science/hal-01066344
- Cited as: "Verpoorter et al. 2014, GRL 41:6396 (117 million lakes over 0.002 km2, 5.0 million km2, 3.7 % of non-glaciated land)"
- Ledger ids: HYD-062, HYD-063, HYD-090
- Access: open access (hybrid)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): the abstract, 2026-10-20 (the lakes task, through Semantic Scholar; the publisher answered 403): about 117 million lakes over 0.002 km2, about 5 x 10^6 km2, about 3.7 % of the nonglaciated land. The paper itself is not read.
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: matches (authors, year, journal, volume, pages, DOI).

### AllenPavelsky2018
- Reference: Allen, George H. and Pavelsky, Tamlin M. 2018. Global extent of rivers and streams. Science 361(6402):585-588.
- DOI or URL: https://doi.org/10.1126/science.aat0636 ; author copy https://cdr.lib.unc.edu/downloads/w6634d23t (UNC Carolina Digital Repository)
- Cited as: "Allen & Pavelsky 2018, Science 361:585 (rivers and streams 773,000 km2, 0.58 % of nonglaciated land; GRWL)"
- Ledger ids: HYD-092
- Access: paywalled (publisher); open copy (the authors' accepted manuscript in the UNC repository)
- Library (the project's source library, not distributed): fetched 2026-10-20 (PDF, UNC repository copy, text layer). Text file: AllenPavelsky2018.txt.
- Read in the lakes task (2026-10-20): full text, the abstract and the results: "the global surface area of rivers and streams at mean annual discharge is 773,000 ± 79,000 km2, or 0.58 ± 0.06% of Earth's nonglaciated land surface"; "the total observed area of rivers measured by the GRWL Database is 468,000 km2, or 0.35% of Earth's nonglaciated land surface"; rivers wider than 90 m 404,000 km2.

### pyBacktrack_lithologies
- Reference: EarthByte group, pyBacktrack (software), bundled lithology table `pybacktrack/bundle_data/lithologies/primary.txt`. https://github.com/EarthByte/pyBacktrack
- Cited as: the secondary source of Sclater & Christie 1980's porosity-depth parameters, for the porous cover tried in the plate model's column on 2026-10-20 and reverted (isostasy-and-height-limit.md)
- Ledger ids: none (the trial was reverted)
- Access: open (software, GitHub)
- Library: no saved copy (read through raw.githubusercontent.com on 2026-10-20).
- Read in the lakes task (2026-10-20): the header "# name density porosity decay / # kg/m3 (%/100) m" and the four rows "# Sclater and Christie [1980]": "Shaley_sand 2680 0.56 2564", "Shale 2700 0.63 1960", "Sand 2650 0.49 3704", "Chalk 2710 0.7 1408".

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
- Stage 2 read (WP-11, 2026-10-02, Zomer2022.txt): full text. The class table (<0.03, 0.03-0.2, 0.2-0.5, 0.5-0.65, >0.65) given 'as a general reference' from UNEP; BIO-038 split into BIO-038 (verified) and BIO-100 (the mapping's own anchors).

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

### SyvitskiMilliman2007
- Reference: Syvitski, J. P. M. and Milliman, J. D. (2007). Geology, geography, and humans battle for dominance over the delivery of fluvial sediment to the coastal ocean. Journal of Geology 115(1):1-19.
- DOI or URL: 10.1086/509246; an open copy at https://website.whoi.edu/gfd/wp-content/uploads/sites/14/2018/10/syvitski__Milliman_2007_75324.pdf
- Cited as: "BQART (Syvitski & Milliman 2007)" (hydro/delta.rs).
- Ledger ids: HYD-130, HYD-131
- Access: paywalled; an open copy (WHOI).
- Library: not fetched (the copy read is in the deltas agent's scratch folder, not the library).
- Read so far (by the project): 2026-10-23, the deltas agent, the WHOI copy in full text: Qs = ωBQ^0.31A^0.5RT for T ≥ 2 °C, Qs = 2ωBQ^0.31A^0.5R for T < 2 °C, ω = 0.02 for kg/s (0.0006 for Mt/yr), Q km³/yr, A km², R km, T °C; B = I L (1 − TE) Eh with I = 1 + 0.09 Ag, L from 0.5 to 3 (mean 1), TE the trapping of lakes and reservoirs; B = 1 explains 68 % of the variance.

### Caldwell2019
- Reference: Caldwell, R. L., Edmonds, D. A., Baumgardner, S., Paola, C., Roy, S. and Nienhuis, J. H. (2019). A global delta dataset and the environmental variables that predict delta formation on marine coastlines. Earth Surface Dynamics 7(3):773-787.
- DOI or URL: 10.5194/esurf-7-773-2019
- Cited as: "Caldwell et al. 2019" (hydro/delta.rs stats; rivers-and-lakes.md).
- Ledger ids: HYD-143
- Access: open access.
- Library: not fetched.
- Read so far (by the project): 2026-10-23, the deltas agent, the PDF as text: 5,399 coastal rivers at least 50 m wide, 40 % (2,174) with a delta; Eq. 1, ln(π/(1 − π)) = 1.45 + 0.000589 Qw + 2.56 Qs − 0.974 Hw − 0.187 Ht; Eq. 2 without waves and tides; the largest rivers have a delta with likelihood near 1; 85 % of the global flux enters deltas.

### Edmonds2020
- Reference: Edmonds, D. A., Caldwell, R. L., Brondizio, E. S. and Siani, S. M. O. (2020). Coastal flooding will disproportionately impact people on river deltas. Nature Communications 11:4741.
- DOI or URL: 10.1038/s41467-020-18531-4 (PMC7525510)
- Cited as: "Edmonds et al. 2020" (hydro/delta.rs stats; rivers-and-lakes.md).
- Ledger ids: HYD-143
- Access: open access.
- Library: not fetched.
- Read so far (by the project): 2026-10-23, the deltas agent, an open PDF copy as text: deltas hold 847,936 km² of geomorphic area (0.57 % of the land), 710,187 km² habitable; the Amazon 84,429 km² and the Ganges-Brahmaputra 80,174 km²; the ten largest 48 % of the area; the polygon runs from the delta node (the upstream-most bifurcation) to the shoreline.

### Nienhuis2026
- Reference: Nienhuis, J. H., Paniagua-Arroyave, J. F., Dunn, F. E., Cohen, S. and Törnqvist, T. E. (2026). Global river sediment retention on delta plains. AGU Advances. Code and data: jhnienhuis/GlobalSedimentRetention v3, Zenodo 10.5281/zenodo.23034704 (MIT).
- DOI or URL: preprint 10.22541/essoar.15002227/v1; code and data https://github.com/jhnienhuis/GlobalSedimentRetention/tree/v3
- Cited as: "Nienhuis et al., `get_retention.m`" and "the 1,741 deltas of their data" (hydro/delta.rs).
- Ledger ids: HYD-132, HYD-133, HYD-134
- Access: the article not reached (ESSOAr 403); the code and data open.
- Library: not fetched.
- Read so far (by the project): 2026-10-23, the deltas agent: `get_retention.m` (depth = 0.08 Q^0.65, at most 69 m; plain volume area × depth over t_delta), `Text_Numbers.m` (t_delta 7,000 yr, bulk density 1,600 kg/m³) and GlobalDeltaRetention.xlsx (1,741 deltas: flux-weighted retention 0.285, mean 0.041, total area 874,136 km²; the agent's fit log10 A[m²] = 5.463 + 0.932 log10 Qs[kg/s]); the abstract through search results (about 830,000 km² of deltaic land, 28 ± 14 % of the global flux retained on delta plains).

### Edmonds2021
- Reference: Edmonds, D. A., Chadwick, A. J., Lamb, M. P., Lorenzo-Trueba, J., Murray, A. B., Nardin, W., Salter, G. and Shaw, J. B. (2021). Morphodynamic modeling of river-dominated deltas: a review and future perspectives. Treatise on Geomorphology, 2nd edition (the author list from memory).
- DOI or URL: an open copy at https://lamb.caltech.edu/documents/19809/Edmonds_etal_2021.pdf
- Cited as: "Edmonds et al. 2021" (hydro/delta.rs; rivers-and-lakes.md).
- Ledger ids: HYD-137, HYD-138, HYD-139, HYD-140
- Access: an open copy (Caltech).
- Library: not fetched.
- Read so far (by the project): 2026-10-23, the deltas agent, the Caltech copy: the avulsion length scales with the backwater length L_b = h/S (after Jerolmack & Swenson 2007); lobes fan from an apex (the avulsion node); mouth bars form within about two channel widths of a stable jet (after Canestrelli et al. 2014); channel widths, depths and lengths fall with each bifurcation (after Edmonds & Slingerland 2007).

### JerolmackSwenson2007
- Reference: Jerolmack, D. J. and Swenson, J. B. (2007). Scaling relationships and evolution of distributary networks on wave-influenced deltas. Geophysical Research Letters 34:L23402.
- DOI or URL: 10.1029/2007GL031823
- Cited as: "Jerolmack & Swenson 2007" (rivers-and-lakes.md, through Edmonds et al. 2021).
- Ledger ids: HYD-137
- Access: paywalled; not fetched.
- Library: not fetched.
- Read so far (by the project): at second hand only (Edmonds et al. 2021; search results), 2026-10-23.

### EdmondsSlingerland2007
- Reference: Edmonds, D. A. and Slingerland, R. L. (2007). Mechanics of river mouth bar formation: implications for the morphodynamics of delta distributary networks. Journal of Geophysical Research 112:F02034.
- DOI or URL: 10.1029/2006JF000574
- Cited as: "Edmonds & Slingerland 2007" (rivers-and-lakes.md).
- Ledger ids: HYD-139
- Access: paywalled (Wiley 403); not fetched.
- Library: not fetched.
- Read so far (by the project): the abstract as quoted in search results, 2026-10-23: widths, depths and lengths of distributaries fall with each bifurcation; the bar forms within 0 < x/W < 2 of the mouth.

### Ganti2014
- Reference: Ganti, V., Chu, Z., Lamb, M. P., Nittrouer, J. A. and Parker, G. (2014). Testing morphodynamic controls on the location and frequency of river avulsions on fans versus deltas: Huanghe (Yellow River), China. Geophysical Research Letters 41(22):7882-7890.
- DOI or URL: 10.1002/2014GL061918
- Cited as: "Ganti et al. 2014" (rivers-and-lakes.md; physics-gap-audit.md).
- Ledger ids: HYD-137
- Access: paywalled (Wiley 403 on 2026-10-23); not fetched.
- Library: not fetched.
- Read so far (by the project): the abstract as quoted in search results, 2026-10-23: where the river debouches from the Loess Plateau, avulsions occur at an abrupt decrease in bed slope and recur about every 607 years; the key points say fans and deltas have different drivers for avulsion and delta lobe size scales with the backwater length. The abstract's delta sentences were cut off.

### Canestrelli2014
- Reference: Canestrelli, A., Nardin, W., Edmonds, D., Fagherazzi, S. and Slingerland, R. (2014). Importance of frictional effects and jet instability on the morphodynamics of river mouth bars and levees. Journal of Geophysical Research: Oceans 119 (the reference from memory).
- DOI or URL: not checked.
- Cited as: "Canestrelli et al. 2014" (rivers-and-lakes.md, through Edmonds et al. 2021).
- Ledger ids: HYD-138
- Access: not checked.
- Library: not fetched.
- Read so far (by the project): at second hand only (Edmonds et al. 2021), 2026-10-23.

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
- Ledger ids: CLM-101, GLA-010 to GLA-012, BIO-157 (section 4.6, the steady-state balance of a glacier; from memory, 2026-10-18)
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
- Read in WP-7 (2026-10-02): not read (book); the Notre Dame lecture PDFs refuse the test machine. The formation temperatures RCK-039 to RCK-044 were compared with Baxter et al. 2002, Stipp et al. 2002, Ferrill et al. 2004 and the Tulane lecture notes (secondary).

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
- Read in WP-7 (2026-10-02): not read (book).

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
- Read in WP-7 (2026-10-02): not read (book). RCK-045 was checked against Orti et al. 2012 and Hardie 1967.

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
- Read in WP-7 (2026-10-02): still abstract only (the full text is behind a bot check). The equation T = 0.6 L^0.6 is not in this paper's abstract; it is the fit of Cruden and McCaffrey 2001 (Phys. Chem. Earth A 26), which the same author restates in Cruden 2008 (SKB R-08-138, library `Cruden2008_SKB`), read in full: b = 0.6 +/- 0.15 and a = 0.6 +/- 0.1 for 66 plutons, with L the circle-equivalent diameter. RCK-061 now cites that source.

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
- Read in WP-7 (2026-10-02): still not read. The Tuolumne Intrusive Complex area of about 1,100 km2 was read in Ardill et al. 2020 (Front. Earth Sci. 8:190, library `Tuolumne_Frontiers2020`); the 45 x 25 km outline was not found.

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
- Read in WP-7 (2026-10-02): not read. Secondary lecture notes put the epizone at 0 to about 5-6 km and the mesozone to about 11-12 km.

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
- Read in WP-7 (2026-10-02): not read (paywalled). Quoted at second hand: roofs of more than several hundred metres resist diapirs without extension or shortening. STR-026, STR-027 are now game choices.

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
- Read in WP-7 (2026-10-02): not read. The Niger Delta wedge is 12 km thick in Sanuade and Kaka 2017 (secondary). STR-028, STR-029 are now game choices.

### Mouthereau2007
- Reference: Mouthereau, F., Tensi, J., Bellahsen, N., Lacombe, O., De Boisgrollier, T. & Kargar, S. (2007). Tertiary sequence of deformation in a thin-skinned/thick-skinned collision belt: The Zagros Folded Belt (Fars, Iran). Tectonics 26(5):TC5006.
- DOI or URL: 10.1029/2007TC002098
- Cited as: "Mouthereau, Tensi, Bellahsen, Lacombe, De Boisgrollier & Kargar 2007, Tectonics 26:TC5006"
- Ledger ids: DEF-001, DEF-005, PLT-241, PLT-242 (plate tasks 3 and 4: "the 200-km-width ZFB"; underplating of Arabian crust "represents 75% of the total shortening", abstract)
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
- Ledger ids: DEF-002, PLT-245 (plate tasks 3 and 4: the Himalaya-Tibet force of about 7 x 10^12 N/m, Molnar & Lyon-Caen 1988 as cited, text lines 1370-1373)
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
- Read in WP-7 (2026-10-02): not read (paywalled); only secondary mentions of 1,600 km and 'up to 1000' plutons were found.

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
- Read in WP-7 (2026-10-02): not read (paywalled).

### Alavi2007
- Reference: Alavi, M. (2007). Structures of the Zagros fold-thrust belt in Iran. American Journal of Science 307(9):1064-1095.
- DOI or URL: 10.2475/09.2007.02
- Cited as: not cited by the code; added as evidence for the belt width (STR-030, DEF-027).
- Ledger ids: DEF-027, STR-030; notes: evidence only, PLT-241 (plate tasks 3 and 4: the Zagros fold-thrust belt "an average width of ~300 km", p. 1068, as an Earth comparison for the belt width)
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): fetched (PDF, 39 pages, text layer, 97,543 characters). From: https://ajsonline.org/article/65872.pdf. Text file: Alavi2007.txt.
- Substitute: repository or open copy found by the lookups and saved as above (version not stated: the first pages were matched against the title, not compared with the printed version).
- Read so far (by the project): not applicable
- Read in this pass: abstract fetched (OpenAlex)
- Reference check: not applicable.
- Read in WP-7 (2026-10-02): full text of the abstract and the passages on belt width: 'an average width of ~300 km' for the Zagros fold-thrust belt and 'a ~200 to 300 km-wide zone of distributed, partly synchronous, deformation'. STR-030's 300 km agrees.
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
- Read in WP-7 (2026-10-02): not read; its 65 mW/m2 continental mean is restated in Davies and Davies 2010 (library `Davies2010`, read in full).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Singer2008a/of1155.txt: rules (PDF pp. 3-6), Table 1 (PDF p. 9), correlations (p. 8), the text layer of Figs. 1-26 (every intercept label), and the companion database PorCuTX2008.txt parsed to recount presence counts, subtypes, correlations, areas and the sulfide-area regression (all reproduced). Rows verified: DEP-001 to DEP-009, DEP-013, DEP-017, DEP-029 to DEP-032 and others.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the USGS record abstract only (Singer2005/Singer2005.txt): 4 / 15 / 35 per 100,000 km2 of exposed permissive rock, range 2 to 128, 286 deposits and prospects, 19 control areas, density inversely related to permissive area. Full text paywalled.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the USGS record abstract only (Singer2008b/Singer2008b.txt): tract size predicts the number of deposits; the equation is not in the abstract.

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
- Ledger ids: DEP-016 to DEP-018, DEP-032, DEP-035 to DEP-049, DEP-209, DEP-212, ORE-104
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/sir/2010/5070/b/pdf/SIR10-5070B.pdf]
- Library (the project's source library, not distributed): fetched (PDF, 186 pages, text layer). From: https://pubs.usgs.gov/sir/2010/5070/b/. Text file: sir5070b.txt.
- Read so far (by the project): (DEP) printed pages cited (pp. 4, 25, 28, 65-68, 120); text searched for density with no numeric value found. (ORE) page numbers cited (PDF opened).
- Read in this pass: (DEP) full text fetched (text layer; searched for every number in the ledger rows; Table E1 itself not extracted, its summary sentences were). (ORE) abstract fetched (USGS page; two different fetches listed the authors in different orders; the series page lists David A. John first as editor).
- Reference check: (DEP) the USGS catalogue lists David A. John as editor and Ayuso et al. as chapter authors; "John et al. 2010" is the usual short form but strictly John is the series editor. (ORE) matches "John et al. 2010". The report is a descriptive model; the median ore area 0.6 km2 and the 1-1.5 km height could not be read.
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): John2010/sir5070b.txt printed pp. 4, 25, 28 (Table E1), 65-68, 120: ore and alteration areas, hypogene ore height, supergene thicknesses, grades and enrichment factors, the 10 cm/yr passage and its context.
- Read again 2026-10-17 (gossan task, P1-19 exit): the PDF fetched from https://pubs.usgs.gov/sir/2010/5070/b/pdf/SIR10-5070B.pdf, text by pdftotext on the test VM (scratch, deleted). Printed p. 4 and pp. 64-65: 'The thickness of leached capping is highly variable and ranges from 0 to several hundred meters in most PCDs.' P. 67: supergene enrichment puts 'an upper oxidized leached zone (gossan), a supergene zone beneath the gossan, and hypogene (primary) ore below' (fig. L1), with the enrichment 'below the water table'; supergene-enriched zones 'can range from a few meters to more than 200 meters in thickness and generally have a hematitic leached cap as much as tens of meters in thickness'. P. 120: leaching 'is only possible when the rate of leaching and drawdown of the water table exceed the rate of erosion'; the chalcocite blankets 'are considered to have formed at the top of the water table'; pyrite gives limonitic, chalcopyrite goethitic and chalcocite hematitic leached capping.

### CookPorter2005
- Reference: Cook, S.S., and Porter, T.M., 2005, The geologic history of oxidation and supergene enrichment in the porphyry copper deposits of southwestern North America, in Porter, T.M., ed., Super Porphyry Copper and Gold Deposits: A Global Perspective, v. 1: Adelaide, PGC Publishing, p. 207-242.
- DOI or URL: https://portergeo.com.au/full_text/Cook_Porter_SW_NAmer_Supergene-PGC_Publishing.pdf (the publisher's open author copy; 13.8 MB, 37 PDF pages, text layer)
- Cited as: "Cook and Porter 2005".
- Ledger ids: DEP-209, DEP-211, DEP-212
- Access: open (author copy)
- Library (the project's source library, not distributed): fetched 2026-10-17 on the test VM as scratch for the gossan task (not added to the library folder). Text by pdftotext.
- Read in this pass (2026-10-17, gossan task): searched for every statement of a leached cap's thickness and exposure. Inspiration, Arizona (PDF p. 19): 'Prior to mining, the remnant leached cap varied from 0 to 300 m in thickness', over an oxide zone averaging 60 m. Silver Bell, El Tiro pit (PDF p. 14): a '60 m thick ... supergene enriched sulphide blanket overlain by a 30 m thick remnant leached capping which averaged 300 ppm Cu', its contact with the blanket 'sharp and generally parallel[ing] the current surface'; older 'hematite capping exposed on ridges and slopes'. Tyrone, New Mexico (PDF p. 26): a blanket 'from a few metres to more than 100 m in thickness' 'overlain by a comparable remnant thickness of leached capping (60 to 150 m thick)'; the profile came from several cycles of enrichment and leaching. PDF p. 24: a deposit 'represented at surface by heavily iron stained leached capping outcrops'. Morenci (PDF p. 23): the pyritic zone began 60 to 180 m below the surface and the chalcocite zone was 30 to 120 m thick (not used for the cap).
- Reference check: the deposit-by-deposit figures were read as text; the page numbers are PDF pages (the chapter's printed pages are 207 to 242; the PDF's first page carries the chapter's title block).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): USGS Publications Warehouse record 70018070 abstract, new library folder SillitoeMcKee1996/ (abstract only).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the same file as Cox1986 (same md5); see Cox1986.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): OrrisBliss1985.txt printed p. 2 (caveat 1).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): new library folder Bliss1992/ (USGS Bulletin 2004, b2004.pdf and b2004.txt); printed p. 2 read.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Schmidt2007/sir5039.txt printed p. 28 and Table 1 (PLACER1 tract, 55,350 km2, 1 / 2 / 7 deposits).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the same file as Cox1986 (byte-identical); see Cox1986.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the same file as Cox1986 (byte-identical); see Cox1986.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): ofr95831.txt (identical to duBray1995): Superior Fe printed pp. 252-254, Algoma Fe pp. 209-210.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Singer2009/of1252.txt: spatial rules (p. 2), deposit types (pp. 3-5), Tables 2 and 3 (pp. 9-11); the report's database SedZn-PbTX.txt and its metadata were added to the folder and reproduce the presence counts and correlations.

### Leach2010a
- Reference: Leach, D.L., Taylor, R.D., Fey, D.L., Diehl, S.F., and Saltus, R.W., 2010, A deposit model for Mississippi Valley-type lead-zinc ores, chap. A of Mineral deposit models for resource assessment: U.S. Geological Survey Scientific Investigations Report 2010-5070-A, viii + 43 p.
- DOI or URL: (DEP) 10.3133/sir20105070A ; https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf (ORE) https://pubs.usgs.gov/sir/2010/5070/a/ ; PDF https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf (6.3 MB; binary to WebFetch). Note the PDF name is SIR10-5070A.pdf.
- Cited as: (DEP) "Leach et al. 2010", "SIR 2010-5070-A (printed pp. 3-5, 8, 19, 27)". (ORE) "USGS SIR 2010-5070-A (the MVT deposit model)"; also called "Leach et al. 2010" in places.
- Ledger ids: DEP-081, DEP-090, DEP-091, DEP-100, ORE-001, ORE-003, ORE-008, ORE-009, ORE-271, ORE-272 (2026-10-20); notes: also cited in DEP-084, 085, 086, 095; and the MVT notes in ORE-003
- Access: open access (agency report). [other sections: open copy (agency report) at https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf]
- Library (the project's source library, not distributed): fetched (PDF, 64 pages, text layer). From: https://pubs.usgs.gov/sir/2010/5070/a/pdf/SIR10-5070A.pdf. Text file: sir5070a.txt.
- Read so far (by the project): (DEP) printed pp. 3-5, 8, 19, 27. (ORE) not stated; quoted as a phrase.
- Read in this pass: (DEP) full text fetched (Table 1, vertical extent, fluid temperature and salinity, the 600 km criterion, district areas). (ORE) abstract fetched (USGS page). The PDF is COULD NOT READ.
- Reference check: (DEP) matches. "Leach et al. 2010" in the code is used for two different papers (this one and Leach2010b); the 2.02 Ga claim belongs to Leach2010b. (ORE) matches. Important: the code uses "Leach et al. 2010" for two different papers (this report, and Leach2010b below, Economic Geology 105:593). The authors differ.
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Leach2010a/sir5070a.txt: Table 1 (printed p. 4) and text pp. 3, 5-8, 10-11, 19, 25, 27 (600 km tract guide, fluid temperatures and salinities, Pine Point, ore-event durations). It has no 2.02 Ga statement: that is Leach2010b.
- Read in the carbonate and ore-deposit task (2026-10-19, DEP-088, ORE-271), by the task's research sub-agent from the PDF: "hosted mainly by dolostone and limestone in platform carbonate sequences and usually located at flanks of basins, orogenic forelands, or foreland thrust belts inboard of the clastic rock-dominated passive margin sequences"; "More MVT deposits are hosted in dolostone rather than limestone, and dolostone-hosted deposits are generally larger and contain higher Pb, Zn, and Ag grades" (no numeric split; Navan is the named limestone exception).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the same file as Cox1986 (byte-identical); see Cox1986.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Emsbo2016/sir5070n.txt printed pp. 2, 8, 27, 28, 30, 32, 37, 39 (brine temperatures and salinities, 'absence of deposits older than 1.8 Ga', basin fill, sandstone density).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the same file as Cox1986 (byte-identical); see Cox1986.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): identical file to Cannon1995; see there.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Singer2001/pp1640a.txt: abstract (printed p. A1), Table 1 and the porphyry copper passage (printed p. A8): Nevada 0.00015 and Arizona 0.00071 and 0.0012 deposits per km2; no Zn-Pb, sedex or iron-formation density.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): USGS2000a/c1178.txt printed p. 5 (PDF p. 11): the 1 km assessment depth and the whole-deposit rule (the code cited p. 15; corrected).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the three sedex shapefile attribute tables in nmra.zip parsed on the test machine: 12 tracts with estimates, 207,508 km2, sums 2 / 10 / 26 (DEP-107 reproduced).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Wood1983/circ891.txt: glossary (printed p. 8: coal field, region, province), Table 1 (p. 17), Table 2 (p. 22, re-extracted with layout), Table 3 (p. 24), text pp. 24-27 (reliability radii).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): chapter PS (by-line Flores and Bader: Fort Union 22,000 mi2, PS-3; named coal fields), chapter PN (Wyodak-Anderson resource area 21,911 km2, Tables PN-1 and PN-6), chapter PG (Gillette study area 1,026,000 acres, PG-16; Table PG-2).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Nuccio1994/Nuccio1994.txt, Methods (PDF p. 2).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): new library folder Higley2005/ (USGS PP 1698, P1698.pdf and text); printed pp. 22 and 26 read (TTI 10 onset, Ro 1.35 after Waples 1980).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Spencer1988/Spencer1988.txt, 'Thermal maturation and burial history' (printed p. 10).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the folder Nuccio2003 holds a different item (a well-log plate); chapter 4 of DDS-69-B was fetched into a new folder Nuccio2003_DDS69B_ch4/ and printed p. 6 read (Ro thresholds).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the folder Dubiel2003 holds chapter 14 (Nelson), not chapter 5; chapter 5 was fetched into a new folder Dubiel2003_DDS69B_ch5/ and its Appendix B data form (printed p. 36) and p. 27 read.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Moore2003/ofr03253.txt, executive summary (printed p. 8: oil window 80-120 C at about 30 C/km) and the TOC passages.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Tuttle2000/au719211.txt, the whole data form (median 12 MMBO, min 1, max 1500; field counts) and the field-size bar chart re-measured from the PDF's vector geometry: p90 2.36, p50 12.04, p10 89.6 MMBO by log-linear interpolation (a figure reading, no printed percentile).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Ofr9950H/Ofr9950H.txt: 'Mean API used = 35 (0.85 g/cm3)' for Niger Delta oil (corroborates the 0.85 t/m3 crude density; not cited by the code).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the OSTI abstract only (Waples1980/Waples1980.txt): TTI 15 / 75 / 160 and the preservation deadlines.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Newell1997/Newell1997.txt, the procedure paragraph and Table 1 (TTI to Ro after Waples 1981).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only (167 / 158 / 63 m per Myr).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only (125,895 formed, 47,789 remain, 3.3 km).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the USGS record abstract (Singer1993/Singer1993.txt): three-part method; permissive boundaries leave 'less than 1 in 100,000 to 1,000,000' outside.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): not readable: no abstract in OpenAlex, paywalled.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only (lognormal body, hyperbolic tail).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only (47 to 79 % of each metal in the largest 10 % of deposits).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Crossref abstract, new library folder Robert2005/ (abstract only).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): only a web-search summary of the abstract (the publisher page refused the fetch); rows ORE-032 to ORE-034 say so.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): OpenAlex abstract, new library folder Richards2003/ (abstract only).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the author's copy deposited on figshare, added to Cooke2005/ as Cooke2005.pdf and Cooke2005_fulltext.txt; abstract and the compression and crustal-thickening passages read (PDF pp. 1, 8-10, 15).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract only (University of Johannesburg repository page), new library folder Bekker2010/: Superior-type BIF first appeared 'ca. 2.6 Ga', giant BIF 2.6-2.4 Ga, iron formations 'generally disappeared at ca. 1.85 Ga'. No open full text found. The code's acme was changed from 2.65 to 2.6 Ga on this reading.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Crossref abstract only, new library folder BohacsSuter1997/.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): full text (PMC4780611), new library folder Nelsen2016/ (html and text): Introduction 'Coal deposits occur by the Early Devonian'; Discussion lignin origin '~420 Ma'.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Hoggard2020/Hoggard2020.txt, the abstract only (85 % within 200 km of the thick-thin transition).

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
- Ledger ids: DEP-112, DEP-114, DEP-188, ORE-009, ORE-010, ORE-014, ORE-024, ORE-272 (2026-10-20, the abstract's clastic-dominated hosts); notes: claim
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): (DEP) not stated; probably via the research note. (ORE) not stated.
- Read in this pass: (DEP) abstract fetched (OpenAlex). (ORE) abstract fetched (OpenAlex inverted index; the rebuilt text is word-order scrambled in places, so quote only the intact phrases).
- Reference check: (DEP) this is the paper that carries the 2.02 Ga statement (see S33). The project has conflated it with Leach2010a. (ORE) matches (authors, year, volume, pages). The abstract divides the deposits into clastic-dominated (which includes what were called SEDEX) and MVT; the code applies the 2.02 Ga gate to both.
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract only (OpenAlex), new library folder Leach2010b/: clastic-dominated and MVT deposits first appear at 2.02 Ga; only clastic-dominated deposits were important until the late Neoproterozoic.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract of part 1 only (OpenAlex), new library folder GarvenFreeze1984/; the base case (1 km over 300 km) is not in it; part 2 not obtained.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract only (OpenAlex), new library folder Garven1985/: Pine Point model 60-100 C, 0.5-5.0 Myr.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Crossref abstract (full), new library folder Sillitoe2005/Sillitoe2005.txt (abstract only; chapter paywalled): 'at least 0.5 m.y. but typically minima of 3 to 9 m.y.'; 'two or three times the hypogene tenor'.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only.
- Read 2026-10-17 (gossan task): a full open copy exists (Society of Economic Geologists field-trip readings, https://pyrite.utah.edu/fieldtrips/SEGFperu-2012/Readings/Sillitoe%202010.pdf; 4.4 MB, text layer), fetched on the test VM and searched for leached cap and gossan thickness. It gives none: it says 'The economically important results of supergene oxidation and enrichment in porphyry Cu systems have been addressed elsewhere (Sillitoe, 2005)' and mentions only paleowater-table positions at the tops of lithocaps (printed pp. 23-24). The 'Access: paywalled' line above is out of date for a reader with that URL; the lithocap rows (DEP-021, DEP-190) were not re-checked against it in this pass.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): abstract via OpenAlex only (first 2,500 characters).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Blackman2021/Blackman2021.txt (arXiv v3): Fig. 4 (xoshiro256 code), Tables 2 and 3, the SplitMix seeding advice.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the library file is Hormann's April 1992 preprint of the same method; Algorithm PTRS and its constants read on rendered pages (printed pp. 6-7); they match the code. The 1993 journal article remains unread.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): new library folder AbramowitzStegun1964/ (scanned, no text layer); formulas 7.1.25 to 7.1.28, printed p. 299, read on a rendered page.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Acklam/acklam3.txt, the coefficient listing, break-points and error statement (all match the code).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the folder held only the landing page; chapters AM, AR, MC and GL of DDS-60 were fetched into a new folder USGS2000b_DDS60chapters/ and the minimum-field-size and fractile passages read.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Averitt1975/Averitt1975.txt: Table 1 (printed p. 7, coal-bearing area by state), the number-of-beds table (p. 8), Tables 2 and 6, the Mammoth zone (pp. 63-64), the No. 5 and Herrin beds (pp. 68-69).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): the full report added to Trippi2011/ (OFR2011-1296.pdf and text); printed pp. 1-14 read: 34.7 thousand km2 of coalfield in 3.29 million km2 of land; Table 1 lists about 150 coalfield and lignite names and no per-field areas.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): nine context-statement and resource-assessment pages added to BAP2018/ (Commonwealth open content) and read: 36,000 km2 onshore basin, five coalfields named by the Coalfield Geology Council of NSW, Southern Coalfield seam thicknesses.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Hatch2002/Hatch2002.txt, Executive Summary pp. A1-A5, Introduction p. B1, p. D9: the Illinois Basin's coal-bearing area is 36,800 mi2 in Illinois, 6,500 in Indiana and 6,400 in western Kentucky (the 36,800 used by the coal type before stage 2 is Illinois alone).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Ruppert2014/Ruppert2014.txt, 'Coal Regions and Fields' (p. 1): 23 named Appalachian coal fields; no thickness or area tables.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): SAIMM-coalfields/SAIMM-coalfields.txt: p. 95 ('19 coalfields'), Tables I and III. No coalfield areas.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): GABowen/GABowen.txt: the Bowen Basin occupies about 160,000 km2, its southern half under the Surat Basin; no coalfield list.

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Crossref abstract only, new library folder RyerLanger1980/ (peat:coal about 11:1 for one Cretaceous bed).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Cox1986/b1693.txt (MosierSinger1986, MenzieMosier1986, MosierBriskey1986, Cannon1986 and OrrisBliss1986 are byte-identical copies of this file): models 39a (placer Au), 34a (Superior Fe), 28b (Algoma Fe), 32a/32b (MVT), 31a (sedex) text; the grade-tonnage arrow labels are not in the OCR layer and were read on rendered page images (PDF page = printed + 16).

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
- Read in stage 2 (WP-9/10, 2026-10-02, by sub-agents in the source library): Crossref abstract, new library folder Garnett2005/ (abstract only): transport 'from less than a kilometer to hundreds of kilometers'.

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


### Smolinski2014
- Reference: Smolinski, A., Rompalski, P., Cybulski, K., Checko, J., and Howaniec, N., 2014, Chemometric study of trace elements in hard coals of the Upper Silesian Coal Basin, Poland: The Scientific World Journal 2014, article 234204.
- DOI or URL: 10.1155/2014/234204 ; https://pmc.ncbi.nlm.nih.gov/articles/PMC4055605/
- Cited as: (stage 2) the Upper Silesian Coal Basin's area and documented deposits in the coal type's notes.
- Ledger ids: ORE-122
- Access: open access (CC BY).
- Library (the project's source library, not distributed): fetched in stage 2 (Europe PMC full-text XML), folder Smolinski2014/, text file Smolinski2014.txt.
- Read in stage 2 (WP-9/10, 2026-10-02): the geological setting: basin 5,600 km2, documented area 3,295 km2, 120 documented deposits; the coal series and their seam counts.

### Hancox2014
- Reference: Hancox, P.J., and Götz, A.E., 2014, South Africa's coalfields - a 2014 perspective: International Journal of Coal Geology 132:170-254.
- DOI or URL: 10.1016/j.coal.2014.06.019 ; author-accepted manuscript at https://repository.up.ac.za/bitstream/handle/2263/58453/Hancox_South_2014.pdf
- Cited as: (stage 2) the South African coalfield areas in the coal type's notes.
- Ledger ids: ORE-126
- Access: paywalled; author-accepted manuscript in the University of Pretoria repository.
- Library (the project's source library, not distributed): fetched in stage 2, folder Hancox2014/, text file Hancox2014.txt (some digits garbled in the text layer; those statements are not relied on).
- Read in stage 2 (WP-9/10, 2026-10-02): section 4, the coalfield location paragraphs: 19 coalfields over some 9.7 million ha; Witbank over 568,000 ha, Highveld about 700,000 ha, Waterberg about 360,000 ha.

### ICS2023
- Reference: International Commission on Stratigraphy, 2023, International Chronostratigraphic Chart, v2023/09.
- DOI or URL: https://stratigraphy.org/ICSchart/ChronostratChart2023-09.pdf
- Cited as: (stage 2) the base of the Devonian, 419.2 +/- 3.2 Ma, for the land-plants clock (deposits/clocks.rs LAND_PLANTS_GA).
- Ledger ids: ORE-015
- Access: open.
- Library (the project's source library, not distributed): fetched in stage 2, folder ICS2023/, text file ChronostratChart2023-09.txt.
- Read in stage 2 (WP-9/10, 2026-10-02): the Devonian base, '419.2 +/-3.2' Ma, read by the WP-9/10 agent.

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

- Read in WP-14a (2026-10-02): Read WP-14a: saturation vapour pressure about 7 %/K (p. 5686) and column vapour 7.5 %/K in models (Fig. 2, p. 5689), as a consistency check for CLM-144.
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
- Stage 2 read (WP-11, 2026-10-02, ODonnell2012.txt): full text. Bio 15 is the coefficient of variation of the twelve monthly totals; eq. 16 divides by 1 + Bio12/12. n or n-1 is not stated.

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
- Ledger ids: PLT-145, SCL-030, PLT-372 (2026-10-21)
- Read in the plate-geometry task (2026-10-21): full text (the Caltech course copy, now in the library as `Mallard2016/nature17992-Mallard.pdf`, `mallard.txt`). Abstract: "seven large plates of similar sizes and a population of smaller plates whose areas follow a fractal distribution"; "subduction geometry drives the tectonic fragmentation that generates plates. The spacing between the slabs controls the layout of large plates, and the stresses caused by the bending of trenches break plates into smaller fragments". Main text: "an interlocking mosaic of 52 tectonic plates", "a group of seven large plates of similar area covering up to 94% of the planet". PLT-372.
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
- Ledger ids: PLT-188, SCL-041 (the force-balance page, plate-forces.md, quotes its abstract)
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

### Laske2013 (CRUST1.0)
- Reference: Laske, G., Masters, G., Ma, Z., Pasyanos, M., 2013. Update on CRUST1.0: A 1-degree Global Model of Earth's Crust. Geophysical Research Abstracts 15, EGU2013-2658 (EGU General Assembly abstract). Model: CRUST1.0, 1 x 1 degree, eight layers (water, ice, three sediment layers, upper, middle and lower crystalline crust) over the mantle.
- DOI or URL: https://igppweb.ucsd.edu/~gabi/crust1.html ; data http://igppweb.ucsd.edu/~gabi/crust1/crust1.0.tar.gz (crust1.bnds: the top of each layer, km)
- Cited as: "CRUST1.0 (Laske, Masters, Ma & Pasyanos 2013)" (tools/review/earth-crust.py, tools/review/continents_census.py)
- Ledger ids: HYP-040, HYP-041, PLT-361 (2026-10-21: the continental crust's drowned share, shelf and thickness by cut); HYP-042, HYP-043 (2026-10-22: the drowned share by the rock's height, the hypsometry of crust at least 20 km thick, the crossing thickness, the area by thickness class)
- Access: open (data files)
- Library (the project's source library, not distributed): the model's data, fetched 2026-10-20 to the land-shape task's scratch folder on the test VM; the abstract was not fetched.
- Read so far (by the project): the data read and measured (tools/review/earth-crust.py); the abstract is cited from the model's own page, not read (from-memory).
- Reference check: the citation the model's page asks for is the conference abstract, as for CRUST 2.0 (Bassin2000).

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
- Ledger ids: PLT-023, PLT-030, PLT-162, PLT-182, PLT-185, PLT-189, PLT-199, FRC-010, FRC-012, FRC-016, FRC-017
- Access: paywalled (abstract only).
- Library (the project's source library, not distributed): fetched in WP-13a (publisher's free PDF, 38 pages, text layer). From: https://academic.oup.com/gji/article-pdf/43/1/163/1576393/43-1-163.pdf. Text file: ForsythUyeda1975.txt.
- Substitute: none found.
- Read so far (by the project): abstract only / secondary pages. Force-balance task 5 (2026-10-02): the full text in the library, by a Sonnet sub-agent with line numbers: slab forces "an order of magnitude stronger than any other force" and the "terminal velocity" (Summary, lines 27-35; 716-727; 7.0 cm/yr at 1237-1239); continental drag "approximately eight times" the oceanic (1238-1241, a result of their inverse fit); slab-attached plates 6-9 cm/yr, others 0-4 cm/yr (699-704); ridge push of McKenzie 1972 "of the order of 300 bars" (1441-1450); their lithosphere 80 km.
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
- Ledger ids: PLT-062, PLT-067, PLT-207, PLT-056 and PLT-371 (2026-10-21: polarity reversal against transference), PLT-503 (2026-10-23: initiation off the passive margin), PLT-516 (2026-10-23, the docking lane: transference, read again in full: "a buoyant crustal block enters the subduction zone and causes it to fail, and in the process is sutured to the original hanging wall ... the oceanic lithosphere outboard of the collision zone ultimately ruptures to become a new subduction zone"; India: "It takes less work to deform large parts of continental Asia than it does to start a new subduction zone in the Indian Plate")
- Access: paywalled (no abstract available in OpenAlex); an author copy is open (2026-10-21).
- Library (the project's source library, not distributed): fetched 2026-10-21 (author copy and the author's encyclopedia chapter "Subduction initiation"). Files: `Stern2004/SternEPSL04.pdf`, `SternEPSL04.txt`, `SubInitEoG221.pdf`.
- Substitute: none needed.
- Read so far (by the project): full text, 2026-10-21 (the plate-geometry task). Abstract: induced nucleation (INSZ) has "Two subclasses ... transference and polarity reversal"; "Transference INSZ moves the new subduction zone outboard of the failed one"; polarity reversal "also follows collision, but continued convergence in this case results in a new subduction zone forming behind the magmatic arc; the response of the Solomon convergent margin following collision with the Ontong Java Plateau is the best example". p. 279: "Like transference, polarity reversal is triggered by buoyant crust entering the trench, but differs from transference, in that the new subduction zone nucleates in the originally overriding plate"; the back-arc lithosphere "should be relatively thin and weak. Consequently, polarity reversal INSZ should require less compression than transference INSZ", "supported by the fact that subduction on the south side of the Solomons began almost immediately upon arrival of" the plateau; of India: "It takes less work to deform large parts of continental Asia than it does to start a new subduction zone in the Indian Plate". The OJP crust "averaging 36 km thick", against "the ~20 km expected to result in subduction zone failure". PLT-056, PLT-371.
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
- Cited as: "Stern 2002" (intra-oceanic arcs take tens of Myr to build 20-35 km of crust); no title or journal in the code. Since 2026-10-02 the code cites Cloos 1993 and Stern 2010 instead (PLT-047).
- Ledger ids: none since 2026-10-02 (was PLT-047)
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
- Ledger ids: PLT-160, PLT-167, PLT-207, PLT-261
- Access: paywalled (abstract not obtainable here).
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): abstract or secondary pages.
- Read in this pass: metadata only (OpenAlex abstract field is null; ScienceDirect 403).
- Reference check: correct (author affiliation USGS Anchorage). The figures 105,000 km passive / 53,000 km convergent and 104/181 Myr ages could not be checked.
- Read 2026-10-04 (the ocean–continent transition task, a research sub-agent, full text from a repository copy, https://repository.geologyscience.ru/server/api/core/bitstreams/2230260d-7a9a-4757-b200-f847df8cec5b/content): "The present-day passive margins have an aggregate length of 105,000 km, even longer than the spreading ridges (65,000 km) or the convergent plate boundaries (53,000 km)"; "a mean age of 104 m.y., and a maximum age of 180 m.y."; lengths "taken as the great-circle distance between endpoints". Ledger PLT-261 (comparison only).

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
- Ledger ids: CLM-054, PLT-143, PLT-145, PLT-203, PLT-311, PLT-330 (2026-10-20: the transform test the arcs' fronts take), PLT-372 (2026-10-21: Table 1, the plate count by area)
- Access: open access (G3 is open), not fetched as full text here. [other sections: paywalled (Unpaywall closed)]
- Library (the project's source library, not distributed): fetched in WP-13a (full text PDF, author's site). From: http://peterbird.name/publications/2003_PB2002/2001GC000252.pdf. Text file: Bird2003.txt.
- Read so far (by the project): (PLT) "confirmed from abstract or secondary pages"; areas from Wikipedia. (CLM) not stated; no title or journal given
- Read in this pass: (PLT) abstract fetched (OpenAlex reconstruction). (CLM) search result snippet only (the citing passages returned were about regional plate boundaries and did not mention 51,000 km or 62 mm/yr)
- Reference check: (PLT) correct. The abstract gives "14 large plates ... plus 38 small plates ... for a total of 52 plates". The area numbers and "seven largest 92 percent" are not in the abstract. (CLM) IDENTIFICATION UNCERTAIN. Bird 2003 is the PB2002 plate-boundary model; a total subduction length of 51,000 km at 62 mm/yr is plausible from it but I could not confirm it is in that paper.
- Read in WP-13a (2026-10-02): Table 1 (areas in steradians): Pacific 2.57685 sr, 20.5 % of the sphere; seven plates above 5 %, together 81 % (not 92 %); Table 3 re-read: subduction 51,310 of 260,487 km (19.7 %). PLT-143, PLT-145, PLT-213, CHK-001, CHK-003.
- Read in the land-shape task (2026-10-20): the PDF fetched again from the author's site into the source library (`Bird2003/`, the VM's library had lost it in the 2026-10-16 rebuild), text extracted; paragraphs [126] and [127]: a subduction zone is a step "with a convergent (component of) velocity and a Benioff zone ... and/or a parallel volcanic arc with Quaternary activity"; other steps are "strike-slip if the computed relative velocity ... has an azimuth within ±20 of the azimuth of the plate boundary step". PLT-311.
- Read in the plate-geometry task (2026-10-21): Table 1's 52 areas in steradians copied into `tools/review/plate-geometry.py` (BIRD_SR); the counts above 1, 0.3, 0.1, 0.0377 and 0.01 sr are 7, 10, 14, 19 and 31. PLT-372.

### Morra2013
- Reference: Morra, G., Seton, M., Quevedo, L., Müller, R. D. (2013). Organization of the tectonic plates in the last 200 Myr. Earth and Planetary Science Letters 373:93-101.
- DOI or URL: 10.1016/j.epsl.2013.04.020.
- Cited as: "Morra et al. 2013, EPSL 373:93-101".
- Ledger ids: PLT-145, PLT-372 (2026-10-21)
- Access: paywalled; an author copy is open (2026-10-21).
- Library (the project's source library, not distributed): fetched 2026-10-21 (author copy). Files: `Morra2013/Morra_etal_Plates200Ma_EPSL_2013.pdf`, `morra.txt`.
- Substitute: none needed.
- Read so far (by the project): full text, 2026-10-21 (the plate-geometry task). Abstract: plate sizes follow "either one (Sornette and Pisarenko, 2003) or two (Bird, 2003) statistically distinct groups"; "two distributions indicate distinct evolutionary laws for large and small plates, the first tied to mantle flow, the second determined by a hierarchical fragmentation process"; over the last 200 Myr "(i) large and small plates display distinct statistical distributions, (ii) the small plates display little organisational change since 60 Ma and (iii) the large plates oscillate between heterogeneous (200–170 Myr and 65–50 Ma) and homogeneous (120–100 Ma) plate tessellations on a timescale of about 100 Myr". Fig. 2 caption: below log(Size) about 7.5 (km²) "the small plates follow a power-law with an exponent α∼3–5", "the largest 6–7 plates follow a steeper slope". PLT-372.
- Read in this pass: metadata only.
- Reference check: correct.

### Lonsdale2005
- Reference: Lonsdale, P. (2005). Creation of the Cocos and Nazca plates by fission of the Farallon plate. Tectonophysics 404(3-4):237-264.
- DOI or URL: 10.1016/j.tecto.2005.05.011.
- Cited as: "Lonsdale 2005, Tectonophysics 404:237-264".
- Ledger ids: PLT-146, PLT-532 (2026-10-24)
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: no open copy located by the lookups tried (Crossref licence links, Europe PMC, Semantic Scholar, and OpenAlex for part of the list).
- Substitute: none found.
- Read so far (by the project): not stated.
- Read in this pass: metadata only.
- Reference check: correct; the design cites it for Farallon and not for lifetime statistics (the paper is about a plate fission event).
- Read 2026-10-24 (the front lane, PLT-532; abstract only, the Scripps Scholars page through the lane's research agent): the Farallon plate's fission into the Cocos and Nazca plates at about 23 Ma, by the "divergence of slab-pull stresses".

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
- Ledger ids: PLT-154, PLT-177, PLT-200, PLT-245 (plate tasks 3 and 4: India-Asia convergence from about 15 to about 4 cm/yr, thickening of Tibet the dominant cause; abstract)
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
- Ledger ids: PLT-180, PLT-242 (plate tasks 3 and 4: "as much as 280 km of shortening was accommodated by underthrusting of the European margin")
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
- Ledger ids: PLT-188, FRC-010, FRC-027
- Access: open copy at Wiley (bronze), abstract read here.
- Library (the project's source library, not distributed): fetched in WP-13a (4 pages, text layer). From: https://discovery.ucl.ac.uk/id/eprint/146995/1/2005GL025621.pdf. Text file: ConradLB2006.txt.
- Read so far (by the project): from abstract. Force-balance task 5 (2026-10-02): the full text in the library, by a Sonnet sub-agent: roots "increase traction magnitudes by a factor of 2-5 compared to 100 km thick lithosphere" (abstract, lines 19-21), "about three times larger on average" (165-169); reference lithosphere 100 km (97-98).
- Read in this pass: abstract fetched.
- Reference check: correct.
- Read in WP-13a (2026-10-02): full text: continental roots "increase traction magnitudes by a factor of 2-5 compared to 100 km thick lithosphere" (abstract and section 3). PLT-188 verified.

### LithgowBertelloniRichards1998
- Reference: Lithgow-Bertelloni, C., Richards, M. A. (1998). The dynamics of Cenozoic and Mesozoic plate motions. Reviews of Geophysics 36(1):27-78.
- DOI or URL: 10.1029/97RG02282.
- Cited as: "Lithgow-Bertelloni & Richards 1998, Rev. Geophys. 36:27-78".
- Ledger ids: PLT-187, FRC-009, FRC-016
- Access: paywalled (abstract).
- Library (the project's source library, not distributed): fetched in WP-13a (52 pages, text layer). From: https://discovery.ucl.ac.uk/id/eprint/125253/1/97RG02282.pdf. Text file: LithgowBertelloniRichards1998.txt.
- Substitute: none found.
- Read so far (by the project): from abstract. Force-balance task 5 (2026-10-02): the full text in the library, by a Sonnet sub-agent: ">90%" slabs, "<10%" lithospheric effects (abstract, lines 15-17; 2386-2388; 3019); upper mantle 4-5 x 10^20 Pa s (2198-2200), a weak asthenosphere "of the order of 10^19 Pa s" (2467-2475); they give no ridge push or slab pull in N/m.
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
- Ledger ids: PLT-187, FRC-012 (not read: the library has no copy)
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
- Ledger ids: PLT-184, FRC-026
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): from abstract.
- Read in this pass: metadata only.
- Reference check: correct.
- Read in WP-13a (2026-10-02): text not obtainable (Durham's repository refuses scripts). The break-off times are read in the same group's EGU 2011 abstract (vanHunen2011EGU, below). PLT-184.
- Read 2026-10-20 (the crust-budget task; a research sub-agent fetched the full text from a course copy, https://www.eps.mcgill.ca/~courses/c350/ProbSets/vanHunen_2011_EPSL.pdf, and quoted it): "collision after the subduction of old, strong subducting oceanic slab leads to slab break-off at 20–25 Myr after the onset of continental collision, and subsequently a slab tear migrates more or less horizontally through the slab with a propagation speed of 100–150 mm/yr. In contrast, young, weak oceanic slabs show the first break-off already 10 Myr after continental collision, and can experience tear migration rates up to 800 mm/yr"; in the asymmetric model "the tear propagates over 1000–1200 km, which suggests an average tear propagation rate of about 300 mm/yr"; observed rates "20–940 mm/yr ... (Yoshioka and Wortel, 1995)" and Carpathian depocentre migration 70–450 mm/yr. PLT-184's 10 Myr is the paper's young-slab break-off time. A tear along the trench at 300 mm/yr was tried as the collisions' end (plates.md "The crust budget against Stern & Scholl") and not kept.

### Zahirovic2015
- Reference: Zahirovic, S., Müller, R. D., Seton, M., Flament, N. (2015). Tectonic speed limits from plate kinematic reconstructions. Earth and Planetary Science Letters 418:40-52.
- DOI or URL: 10.1016/j.epsl.2015.02.037; the EarthByte summary page https://www.earthbyte.org/tectonic-speed-limits-from-plate-kinematic-reconstructions/ returned HTTP 500 / timeouts in this pass.
- Cited as: "Zahirovic et al. 2015, EPSL 418:40-52".
- Ledger ids: PLT-025, PLT-198, PLT-181, FRC-024
- Access: paywalled (closed; OpenAlex has no open URL and no abstract).
- Library (the project's source library, not distributed): fetched in WP-13a (accepted manuscript, 39 pages, text layer; the landing page is kept as Zahirovic2015_landing.txt). From: https://ro.uow.edu.au/ndownloader/files/50501994. Text file: Zahirovic2015.txt.
- Read so far (by the project): link given. Force-balance task 5 (2026-10-02): the full text in the library, by a Sonnet sub-agent: "a median RMS plate speed of ~4 cm/yr over 200 Myr" (lines 123-125); fastest plates ~8.5 cm/yr RMS, slowest ~2.6-2.8; oceanic plates 2-3 times faster than continental; plates with more than 25 % of their perimeter subducting median 7.9 cm/yr, without subduction 2.6 (652-662).
- Read in this pass: metadata only (Crossref). Speed numbers could not be read.
- Reference check: correct.
- Read in WP-13a (2026-10-02): full text (accepted manuscript): median RMS plate speed ~4 cm/yr over 200 Myr, 4.2 over 3,952 samples; global RMS ~5-10 cm/yr (section 3.2); category medians: no subduction 2.6, over 25 % subducting 7.9, over 50 % continental 2.8, under 25 % continental 8.1 cm/yr (section 3.5); oceanic plates "2-3 times faster" (abstract); present RMS Antarctica 1.1, Africa 2.0, Pacific 7.9 cm/yr. PLT-181, PLT-190, PLT-197 to PLT-199, CHK-020 to CHK-025.

### DeCellesGiles1996
- Reference: DeCelles, P. G., Giles, K. A. (1996). Foreland basin systems. Basin Research 8(2):105-123.
- DOI or URL: (PLT) 10.1046/j.1365-2117.1996.01491.x. (ISO) 10.1046/j.1365-2117.1996.01491.x; full text https://pages.uoregon.edu/rdorsey/BasinAnalysis/BasinPapers/DeCelles%20&%20Giles%201996.pdf
- Cited as: (PLT) "DeCelles & Giles 1996, Basin Research 8:105-123" ("via a secondary source"). (ISO) docs/design/flexure.md section 11
- Ledger ids: PLT-194; none (the acceptance figures of docs/design/plate-force-balance.md); notes: the acceptance figures of docs/design/plate-force-balance.md, PLT-249 (plate tasks 3 and 4: foredeeps 100-300 km wide and 2-8 km thick, forebulges 60-470 km, for the foreland measure)
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
- Ledger ids: ISO-040 to ISO-042, ISO-067, ISO-096, PLT-183, PLT-193, FRC-002, FRC-004, FRC-008, FRC-012 (ridge push from memory; not read)
- Access: book (not online).
- Library (the project's source library, not distributed): not fetched. Reason: book, no open copy named.
- Read so far (by the project): not read; gross slab pull "not confirmed in Turcotte & Schubert itself". For the flexure feature (2026-10-02) a sub-agent read chapter 3 as a PDF (https://sseh.uchicago.edu/doc/Geodynamics_Turcotte_Schubert_ch_3.pdf: E = 70 GPa and ν = 0.25 in the periodic-loading example, 100 GPa elsewhere) and the flexure formulas in lecture notes on the chapter (https://topex.ucsd.edu/geodynamics/12flexure.pdf); the edition of that PDF was not recorded.
- Read in this pass: not fetched; a 2024 paper that cites it was read (see Li2024).
- Reference check: edition not stated anywhere in the code or designs; the numbers depend on the edition and chapter.
- Read in WP-13a (2026-10-02): not read. Its slab-pull law is read as Faccenna et al. 2012's eq. 1, which follows Turcotte & Schubert 1982 (PLT-182, PLT-183). A course directory's copy of a chapter was found and not used: it is not an author's or publisher's copy.
- Read 2026-10-22 (the flexure guard, ISO-093): chapter 3 (the same PDF) by a Sonnet sub-agent of the flexure-guard agent: the fibre stress of a bent plate, sigma_xx = E y / (1 - nu^2) x |d2w/dx2|, largest at its faces, sigma_max = 6M/h^2 (eqs. 3.64 to 3.70 and 3.86).
- Read 2026-10-23 (the trench rows, ISO-096): §3.13 of the same PDF, by a Sonnet sub-agent of the trench-row agent: under a water-covered plate, water fills "the space vacated" by the deflection and the restoring force is (rho_m - rho_w) g w (eqs. 3.101 to 3.103). The chapter copy ends at §3.14 and holds no line-load fit of the Mariana; the book's Mariana figures (x_b, w_b, h) were not found and are not used.

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
- Read so far (by the project): from abstract. Force-balance task 5 (2026-10-02): the abstract again (the library holds the landing page only): "extensional stress of 20-50 MPa in its interior" for a homogeneous supercontinent; the thickness it is averaged over is not in the abstract.
- Read in this pass: abstract fetched (Crossref).
- Reference check: correct. The abstract does give "extensional stress of 20-50 MPa in its interior" for a homogeneous supercontinent, matching the design.
- Read in WP-13a (2026-10-02): no full text (the Curtin record has no file; Wiley refuses scripts). PLT-196 stays abstract-only.

### Cloos1993
- Reference: Cloos, M. (1993). Lithospheric buoyancy and collisional orogenesis: subduction of oceanic plateaus, continental margins, island arcs, spreading ridges, and seamounts. Geological Society of America Bulletin 105(6):715-737.
- DOI or URL: 10.1130/0016-7606(1993)105<0715:LBACOS>2.3.CO;2.
- Cited as: in ledger FRC-005 as the usual source for oceanic lithosphere becoming denser than the mantle after about 10 Myr; since 2026-10-02 "Cloos 1993" for the island-arc maturity rule (PLT-047, HNT-090). Since 2026-10-05 for the collision's buoyancy (FRC-028 to FRC-030, PLT-275): 80-Myr oceanic lithosphere about 0.04 g/cm³ denser than the asthenosphere, 100 km of lithosphere with a 30 km continental crust about 0.09 g/cm³ lighter (abstract).
- Ledger ids: FRC-005, PLT-047, HNT-092, PLT-289 (2026-10-18: the context for ocean floor as thick as a continent turning continental at a contact), PLT-331 (2026-10-20: island arcs "built upon ocean crust")
- Access: paywalled.
- Library (the project's source library, not distributed): abstract only, 2026-10-02 (OpenAlex record of the GSW page): `Cloos1993/openalex.json`, text `Cloos1993/abstract.txt`.
- Read so far (by the project): FRC-005 from memory (force-balance task 5, 2026-10-02). Abstract read 2026-10-02 (continental crust fix): "Oceanic lithosphere that is ~10 m.y. old is the youngest that is more dense than the asthenosphere"; "Lithosphere that is 100 km thick with a 30-km-thick granitic continental crust resists Subduction because it is ~0.09 gm/cm3 less dense than the asthenosphere"; "only bodies of continental and oceanic island arc crust that are > ~15 km thick make the lithosphere buoyant enough to jam a subduction zone. Oceanic island arc complexes built upon ocean crust typically must be active for more than ~20 m.y. to attain crustal thicknesses so that their attempted subduction causes collisional orogenesis." The reference agrees with the OpenAlex record.

### Coblentz1994
- Reference: Coblentz, D. D., Richardson, R. M., Sandiford, M. (1994). On the gravitational potential of the Earth's lithosphere. Tectonics 13(4):929-945.
- DOI or URL: 10.1029/94TC00033.
- Cited as: in ledger FRC-013 as the usual source for a mid-ocean ridge and continental lithosphere near sea level having about the same gravitational potential energy.
- Ledger ids: FRC-013
- Access: paywalled.
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): from memory (force-balance task 5, 2026-10-02); reference written from memory, not checked against Crossref.

### DavisSuppeDahlen1983
- Reference: Davis, D., Suppe, J., Dahlen, F. A. (1983). Mechanics of fold-and-thrust belts and accretionary wedges. Journal of Geophysical Research 88(B2):1153-1172.
- DOI or URL: 10.1029/JB088iB02p01153.
- Cited as: "Davis, Suppe & Dahlen 1983, JGR 88:1153-1172" (from memory).
- Ledger ids: PLT-192, PLT-244 (plate tasks 3 and 4: a critical wedge grows at constant taper; abstract)
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
- Ledger ids: (none; supports 121, 122, 131); notes: none; supports 121, 122, 131, PLT-247 (plate tasks 3 and 4: mean continental crust 41.1 km, orogens 46.3 km; abstract)
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
- Read 2026-10-21: the abstract on EarthByte's page: "We find the mean age of oceanic crust is 64.2 Myrs, slightly older than previous estimates".

### SyracuseAbers2006
- Reference: Syracuse, E. M., Abers, G. A. (2006). Global compilation of variations in slab depth beneath arc volcanoes and implications. Geochemistry, Geophysics, Geosystems 7(5):Q05017.
- DOI or URL: 10.1029/2005GC001045.
- Cited as: not cited; candidate source for arc-slab depth (flag 10; PLT-042).
- Ledger ids: (supports 042, 061, 164); notes: supports 042, 061, 164; PLT-042 (2026-10-21 note)
- Read in the plate-geometry task (2026-10-21): abstract (OpenAlex; in the library as `Syracuse2006/abstract_openalex.txt`): "33,000 km of the global arc system at 839 volcanic centers"; the depth to the slab's top under the volcanoes, "averaged over 500-km-long arc segments, H ranges from 72 to 173 km with a global average of 105 km". A depth, not the arc's distance from the trench (PLT-042); the distance needs the slab's dip, which the abstract does not give per arc.
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
- Ledger ids: (supports 183, 187); notes: supports 183, 187, PLT-245 (plate tasks 3 and 4: "the late-Cenozoic resistance force due to the gravitational potential energy (GPE) of the Himalayas and Tibetan Plateau ranges from 3 x 10^12 N m-1 to 8 x 10^12 N m-1", text line 111; read for that sentence)
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

- Read in WP-14a (2026-10-02): WP-14a saved climlab's MIT source files (library climlab_src/: ebm.py, albedo.py, LICENSE) and read the defaults: EBM A 210, B 2, D 0.555, water_depth 10 m, Tf -10, a0 0.3, a2 0.078, ai 0.62, num_lat 90; EBM_seasonal and P2Albedo a0 0.33, a2 0.25 (CLM-001, CLM-007 to CLM-010, CLM-020).
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

- Read in WP-14a (2026-10-02): Read WP-14a for the carried-over constants: A 203.3 and B 2.09 (p. 93, eq. 7), co-albedo 0.681 - 0.202 P2 (p. 95, eq. 18), D 0.649 (p. 97), Budyko's ice albedo 0.62 and -10 degC edge (p. 104), C_L 0.16 B years and C_W 4.7 B years (p. 118). CLM-003 to CLM-005, CLM-009, CLM-010, CLM-013, CLM-014, CLM-122, CLM-138.
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
- Ledger ids: CLM-069, CLM-073, CLM-168 (the doubling anchor of the CO₂ table's scale)
- Access: paywalled (abstract only)
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Substitute: none found.
- Read so far (by the project): not stated
- Read in this pass: abstract fetched (Crossref) plus citing passages
- Reference check: matches. Byrne & Goldblatt's Table 1 prints the same 5.35 form (their row attributes it to "IPCC [1990]" and the adjusted forcing F_a).

- Read in WP-14a (2026-10-02): Not read in WP-14a; its 5.35 was read in IPCC 2001 Table 6.2 (CLM-069).
### Byrne2014
- Reference: Byrne, B., & Goldblatt, C. (2014). Radiative forcing at high concentrations of well-mixed greenhouse gases. Geophysical Research Letters 41(1): 152-160.
- DOI or URL: 10.1002/2013GL058456; open copy https://dspace.library.uvic.ca/server/api/core/bitstreams/65d2a0aa-a7eb-44ba-b8e7-ac34d0d894d7/content (the article as corrected 17 Apr 2015)
- Cited as: "Byrne & Goldblatt 2014, GRL 41: 152-160, Table 2"
- Ledger ids: CLM-070, CLM-071 (both retired in P1-26 T6: the fit is no longer the forcing law), CLM-166, CLM-167 (cross-check of the table at 10^4 and 5·10^4 ppm)
- Access: open copy at the UVic URL above (publisher page returned 403)
- Library (the project's source library, not distributed): fetched (PDF, 10 pages, text layer, 29,061 characters). From: https://dspace.library.uvic.ca/server/api/core/bitstreams/65d2a0aa-a7eb-44ba-b8e7-ac34d0d894d7/content. Text file: Byrne2014.txt.
- Read so far (by the project): "checked in their Table 2 by a Sonnet lookup" (a sub-agent)
- Read in this pass: full text fetched (all sections, Tables 1 and 2)
- Reference check: matches (authors, year, volume, pages, Table 2).

- Read in WP-14a (2026-10-02): Read WP-14a: Table 2 CO2 fit and its range 200-10,000 ppmv; abstract: computed over 100-50,000 ppmv, 38.1 W/m2 at the top (CLM-070, CLM-166).
### Hoffman2017
- Reference: Hoffman, P. F., Abbot, D. S., Ashkenazy, Y., Benn, D. I., Brocks, J. J., Cohen, P. A., Cox, G. M., Creveling, J. R., Donnadieu, Y., Erwin, D. H., Fairchild, I. J., Ferreira, D., Goodman, J. C., Halverson, G. P., Jansen, M. F., Le Hir, G., Love, G. D., Macdonald, F. A., Maloof, A. C., Partin, C. A., Ramstein, G., Rose, B. E. J., Rose, C. V., Sadler, P. M., Tziperman, E., Voigt, A., & Warren, S. G. (2017). Snowball Earth climate dynamics and Cryogenian geology-geobiology. Science Advances 3(11): e1600983.
- DOI or URL: 10.1126/sciadv.1600983; open copy https://dspace.library.uvic.ca/bitstream/1828/11278/1/hoffman_paul_scienceadvances_2017.pdf
- Cited as: "Hoffman et al. 2017, Sci. Adv. 3: e1600983"
- Ledger ids: CLM-103, CLM-166, CLM-172
- Access: open access
- Library (the project's source library, not distributed): fetched (PDF, 45 pages, text layer, 258,807 characters). From: https://dspace.library.uvic.ca/bitstream/1828/11278/1/hoffman_paul_scienceadvances_2017.pdf. Text file: Hoffman2017.txt.
- Read so far (by the project): not stated
- Read in this pass: full text fetched (text layer searched for sea level, ppm, bar, CO2 termination, ice thickness; not every section read)
- Reference check: matches. The paper's CO2 range for termination is 10^4 to 10^5 ppm, and the ~1 km of sea level is stated as an assumption (see Part 2).

- Read in WP-14a (2026-10-02): Read WP-14a: terminations at 10^4 to 10^5 ppm (p. 3 of 43); cloud radiative forcing >= 10 W/m2 and cloud-free FOAM never deglaciating (p. 10 of 43) (CLM-166).
- 2026-10-21 (plates.md, "White patches, drowned continents and coastal high ground"): the Cryogenian's two glaciations' dates (Sturtian about 717 to 660 Ma, Marinoan about 650 to 635 Ma) are cited beside it from memory; not read in it (the library copy was not found on the rebuilt VM).
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
- Read 2026-10-21 (ledger ISO-085): Table 7, "Volume excluding ice shelves (10^6 km3) 25.34 [Bedmap1], 26.54 [Bedmap2]", 26.92 with the shelves; Table 8, "Total potential contribution 58.3" m.

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
- Ledger ids since 2026-10-21: ISO-085 (Greenland's 2.99e6 km3, as read for CLM-108).

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

- Read in WP-14a (2026-10-02): Read in full (eq. 2, eqs. 5-6, Tables 1-2, pp. 2845-2847) for CLM-119, CLM-120, CLM-122; the design's surface-dependent D was corrected to its eq. 2 (theta in eq. 2 is the colatitude).
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

- Read in WP-14a (2026-10-02): The paper could not be fetched (Elsevier refuses scripted access). The release on GitHub (ElsevierSoftwareX/SOFTX-D-16-00023) was read by a sub-agent: its manual is saved (Zhuang2017_manual.txt); the GPL-3 Fortran was read for the constants and not kept or copied (grid 128 x 65, B 2.15, A 210.3 at 315 ppm, Keq 0.65, Kocean 0.40, Kland 0.65, KlandNP 0.28, KlandSP 0.20, the factor sin^5 of the colatitude, soil 2 m, mixed layer 70 m). Ledger ids now also CLM-122.
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

- Read in WP-14a (2026-10-02): Not obtained (AGU/Wiley and ADS refuse scripted access; NASA NTRS holds metadata only). The design's 1.87e6 J/m2/K attributed to this lineage was traced to Zhuang et al.'s land surface (CLM-122).
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

- Read in WP-14a (2026-10-02): Not obtained. An open PDF exists only on a course page that re-hosts the publisher's file, which the library rules exclude. Ferreira et al. 2018 (p. 337) is the secondary statement used (CLM-128).
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

- Read in WP-14a (2026-10-02): Fetched and read in full (NSF public access repository, https://par.nsf.gov/servlets/purl/10101075; Ferreira2018.txt): basin width as one cause of Atlantic sinking (section 4.2, pp. 337-338; p. 344). CLM-132 verified; it gives no basin widths, AMOC number or deep-water temperature (CLM-163).
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

- Read in WP-14a (2026-10-02): Fetched (a copy deposited on Zenodo, https://zenodo.org/records/1234671, depositor unknown; Trenberth2001.txt) and read: atmosphere 5.0 +/- 0.14 PW at 43 N (abstract); the ocean peak only in Fig. 7, about 2 PW near 15-20 N, read by eye; 1.5-2.0 PW observed at 24 N (p. 3439). CLM-133 verified.
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

- Read in WP-14a (2026-10-02): Not obtained (AMS refuses scripted access; NASA NTRS record only; no author copy found). Read through Back & Bretherton 2009 instead (CLM-141 stays from-memory).
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

- Read in WP-14a (2026-10-02): Fetched (the author's copy, https://users.monash.edu.au/~dietmard/papers/dommenget.and.floeter.greb.paper.cdym2011.pdf; Dommenget2011.txt; correct DOI 10.1007/s00382-011-1026-0) and read: eq. 7 bulk evaporation with a turbulent wind of 3 m/s (ocean) and 2 m/s (land) added to the 850 hPa wind (section 3.3); kappa = 8e5 m2/s in section 3.5 but 2e5 in Table 2 (the paper disagrees with itself); C_w and rho_air not numbered; 3.75 x 3.75 degree grid (CLM-142, CLM-139).
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

- Read in WP-14a (2026-10-02): Read WP-14a: 3.75 degree grid (p. 426), eq. 2 (p. 427), eqs. 12-16 (c_eva 0.58/0.25, c_turb 5.4/11.5 m/s, p. 431), eq. 18 (f = 2.5) (CLM-119, CLM-142).
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

- Read in WP-14a (2026-10-02): Fetched (a co-author's copy, https://www.aos.wisc.edu/~lback/wvpprecip.pdf; Bretherton2004.txt) and read: daily fit eq. 1 (a_d 15.6, r_d 0.603, p. 1519), monthly fit eq. 2 (a_m 11.4, r_m 0.522, p. 1521), W*ref 72 mm (p. 1521). The design's rain law was changed to the monthly fit (CLM-145).
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

- Read in WP-14a (2026-10-02): Fetched from GFDL (https://www.gfdl.noaa.gov/bibliography/related_files/sm6901.pdf; Manabe1969.txt, poor text layer, equations read on the page image) and read: eqs. 18 and 20, W_K = 0.75 W_FC, field capacity 15 cm (p. 743). CLM-143 verified.
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

- Read in WP-14a (2026-10-02): Fetched (the author's site, https://eisenman.ucsd.edu/papers/Wagner-Eisenman-2015.pdf; library key WagnerEisenman2015) and read: Table 1 (p. 4001), k = 2 W/m/K and L_f of pure ice, a 75 m mixed layer (p. 4002) (notes on CLM-013, CLM-093, CLM-095).
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

- Read in WP-14a (2026-10-02): Not obtained (AMS refuses scripted access; no author copy found). The inversion size was checked against the NCEP/NCAR reanalysis instead (CLM-165); CLM-134 stays from-memory.
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

- Read in WP-14a (2026-10-02): Read WP-14a: eq. 1 and the diffusion values, p. 3 (Do,1 = 0.5; Do,2 = 0.02 below 45 degrees, 0.0002 above). CLM-121. The design had the year as 2023.
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

- Read in WP-14a (2026-10-02): Murray 1967 (J. Appl. Meteor. 6: 203) could not be obtained; its coefficients were read only on a CIRES page quoting it (6.1078 hPa, 17.269388 and 35.86 K over water). Not verified (CLM-142).
### ETOPO2022
- Reference: NOAA National Centers for Environmental Information (2022). ETOPO 2022 15 Arc-Second Global Relief Model. NOAA NCEI.
- DOI or URL: 10.25921/fd45-gt74 (title and year confirmed through the DataCite API)
- Cited as: "ETOPO 2022"
- Ledger ids: ERO-168, HNT-093 (margins and spires task, 2026-10-03), PLT-265 (the ocean–continent transition, 2026-10-04: the same transects as the comparison), YNG-022 and AMP-074 (sea-floor spikes task, 2026-10-20: deep sea-floor slope distributions from the 60 and 15 arc-second grids)
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

- Read in WP-14a (2026-10-02): WP-14a fetched the Berkeley Earth Land/Ocean data file (library BerkeleyEarth_LandOcean/Land_and_Ocean_complete.txt, https://berkeley-earth-temperature.s3.us-west-1.amazonaws.com/Global/Land_and_Ocean_complete.txt) and averaged its monthly anomalies over 1850-1900: -0.306 K (air above sea ice) and -0.296 K (water below), so 13.80 and 14.40 degC (CLM-138).
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

## Per-cell climate design: sources added in WP-14a (2026-10-02)

### DonohoeBattisti2011
- Reference: Donohoe, A., & Battisti, D. S. (2011). Atmospheric and surface contributions to planetary albedo. Journal of Climate 24(16): 4402-4418.
- DOI or URL: 10.1175/2011JCLI3946.1
- Ledger ids: CLM-123
- Library: fetched in WP-14a, the accepted manuscript from the co-author's page (https://atmos.uw.edu/~david/Donohoe_Battisti_Albedo.pdf). Text file: DonohoeBattisti2011.txt. Manuscript pagination, not the journal's.
- Read in WP-14a: in full by a sub-agent: 88 % of planetary albedo is atmospheric; the atmosphere attenuates the surface contribution by about three (68 % globally, about 60 % in the dry subtropics, eq. 10).

### McCarthy2015
- Reference: McCarthy, G. D., Smeed, D. A., Johns, W. E., et al. (2015). Measuring the Atlantic Meridional Overturning Circulation at 26°N. Progress in Oceanography 130: 91-111.
- DOI or URL: 10.1016/j.pocean.2014.10.006
- Ledger ids: CLM-131
- Library: fetched in WP-14a, the NERC accepted manuscript (http://nora.nerc.ac.uk/id/eprint/508654/). Text file: McCarthy2015.txt.
- Read in WP-14a: abstract and highlights: 17.2 Sv and 1.22 PW, April 2004 to October 2012.

### FrajkaWilliams2019
- Reference: Frajka-Williams, E., Ansorge, I. J., Baehr, J., et al. (2019). Atlantic Meridional Overturning Circulation: observed transport and variability. Frontiers in Marine Science 6: 260.
- DOI or URL: 10.3389/fmars.2019.00260
- Ledger ids: CLM-131
- Library: fetched in WP-14a (open access). Text file: FrajkaWilliams2019.txt.
- Read in WP-14a: section 2.2.4: 17.0 +/- 4.4 Sv, April 2004 to February 2017.

### BackBretherton2009
- Reference: Back, L. E., & Bretherton, C. S. (2009). On the relationship between SST gradients, boundary layer winds, and convergence over the tropical oceans. Journal of Climate 22(15): 4182-4196.
- DOI or URL: 10.1175/2009JCLI2392.1
- Ledger ids: CLM-141
- Library: fetched in WP-14a, the author's copy (https://www.aos.wisc.edu/~lback/SSTgradients.pdf). Text file: BackBretherton2009.txt.
- Read in WP-14a: its account of Lindzen & Nigam 1987 (a layer to 700 hPa and a back-pressure term, p. 4183; 850 hPa more realistic, p. 4195).

### LargeYeager2004
- Reference: Large, W. G., & Yeager, S. G. (2004). Diurnal to decadal global forcing for ocean and sea-ice models: the data sets and flux climatologies. NCAR Technical Note NCAR/TN-460+STR.
- DOI or URL: 10.5065/D6KK98Q6
- Ledger ids: CLM-128
- Library: fetched in WP-14a (https://www.aoml.noaa.gov/ftp/phod/sklee/articles/core2/large_yeager_2004_core.pdf). Text file: LargeYeager2004.txt.
- Read in WP-14a: eq. 6a, the neutral 10-m drag coefficient (1.06e-3 at 5 m/s, 1.18e-3 at 10 m/s, computed).

### IPCC2001_ch6
- Reference: Ramaswamy, V., et al. (2001). Radiative forcing of climate change. In Climate Change 2001: The Scientific Basis (IPCC TAR, WG1), chapter 6, pp. 349-416. Cambridge University Press.
- DOI or URL: https://www.ipcc.ch/site/assets/uploads/2018/03/TAR-06.pdf
- Ledger ids: CLM-069
- Library: fetched in WP-14a. Text file: IPCC2001_ch6.txt.
- Read in WP-14a: Table 6.2 (p. 358), the three CO2 expressions and the footnote crediting Myhre et al. 1998b.

### Kalnay1996
- Reference: Kalnay, E., et al. (1996). The NCEP/NCAR 40-year reanalysis project. Bulletin of the American Meteorological Society 77: 437-471. Data: NOAA PSL, NCEP/NCAR Reanalysis 1 monthly long-term means 1991-2020.
- DOI or URL: 10.1175/1520-0477(1996)077<0437:TNYRP>2.0.CO;2; data https://downloads.psl.noaa.gov/Datasets/ncep.reanalysis/Monthlies/
- Ledger ids: CLM-164, CLM-165, CLM-229, CLM-231
- Library: the data files (uwnd, vwnd, air at pressure levels and sigma 0.995) and the two scripts that read them, folder NCEP_R1_ltm/. The paper itself was not fetched. Public-domain US government data, used for validation only.
- Read in WP-14a: computed zonal-mean winds and January inversions (see the rows).
- Read 2026-10-21 (the plateau-warmth task): a sub-agent fetched the 1991-2020 long-term means of air temperature and geopotential height at pressure levels (https://psl.noaa.gov/thredds/dodsC/Datasets/ncep.reanalysis.derived/pressure/air.mon.ltm.nc.ascii and hgt.mon.ltm.nc.ascii; the files' climo_period "1991/01/01 - 2020/12/31") and interpolated the free air's temperature linearly in height to plateau stations' heights over lowland columns near Tibet, the Altiplano, the Rockies and Ethiopia (CLM-229, CLM-231); NCEP-DOE R2 monthly means agreed within 0.3 K. The files were not kept.

### Stull2017
- Reference: Stull, R. (2017). Practical Meteorology: An Algebra-based Survey of Atmospheric Science, version 1.02b. University of British Columbia. Chapter 2.
- DOI or URL: https://www.eoas.ubc.ca/books/Practical_Meteorology/
- Ledger ids: CLM-123
- Library: fetched in WP-14a (CC BY-NC-SA; chapters 1, 2, 4). Text file: Stull2017_Ch02-radiation.txt.
- Read in WP-14a: Table 2-5, typical albedos (p. 43).

### Norton1979
- Reference: Norton, C. C., Mosher, F. R., & Hinton, B. (1979). An investigation of surface albedo variations during the recent Sahel drought. NASA CR-157198 (grant final report).
- DOI or URL: https://ntrs.nasa.gov/citations/19790021556
- Ledger ids: CLM-123
- Library: fetched in WP-14a. Text file: Norton1979.txt.
- Read in WP-14a: p. 2, Charney et al. 1977's Sahel albedo change from 0.14 to 0.35 (a secondary citation of Charney).

### Guilloteau2000
- Reference: Guilloteau, S. (2000). Atmospheric absorption: water (section 8.1.4). IRAM Interferometry School lecture notes.
- DOI or URL: https://web-archives.iram.fr/IS/IS2002/html_1/node86.html
- Ledger ids: CLM-144
- Library: fetched in WP-14a. Text file: Guilloteau2000.txt.
- Read in WP-14a: "The scale height of water ... is small (typically 2 km)". Institutional lecture notes, not peer reviewed.

## Lithospheric flexure (ISO-040 to ISO-077, ISO-091 to ISO-093)

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
- Read 2026-10-22 (the flexure guard): full text (https://oceaniron.org/wp-content/uploads/sites/14/2018/10/Watts_1978_114185.pdf) by a Sonnet sub-agent of the flexure-guard agent: Te 20-30 km, and "the large deformations and associated flexural stresses (> 1 kbar) at seamount loads do not appear to change appreciably through time". ISO-093.

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
- Read 2026-10-22 (the flexure guard): full text (https://oceaniron.org/wp-content/uploads/sites/14/2018/10/Karner_Watts_1983_114245.pdf) by a Sonnet sub-agent of the flexure-guard agent: the Alps' Bouguer minimum "about 100-200 mGal in amplitude", the Himalaya's "at least 100 mGal and may on some profiles exceed 400 mGal" (Bouguer anomalies, mostly the roots, not departures from isostasy). ISO-093.
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
- Ledger ids: ISO-064, ISO-078 (2026-10-20, the continent budget: the abstract's 3-5 kyr as the settling's time, 4 kyr, `plates/settle.rs`)
- Access: open (publisher page).
- Library (the project's source library, not distributed): not fetched. Reason: the publisher or host refuses scripted access (HTTP 403/405/429); open access status as recorded under Access.
- Read so far (by the project): abstract by a Sonnet sub-agent of the flexure agent on 2026-10-02: relaxation times "close to 3-5 kyr".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### MinecraftSrc1211
- Reference: Mojang Studios. Minecraft 1.21.1, `net.minecraft.world.level.biome.OverworldBiomeBuilder`, as decompiled and published in the NeoForge 21.1.249 sources jar.
- DOI or URL: none (the jar is built by the project's Gradle setup, `neoforge-21.1.249-sources.jar`)
- Cited as: "OverworldBiomeBuilder, Minecraft 1.21.1" (default.json, OverworldBands.java, VolcanicBiomes.java)
- Ledger ids: BIO-021 to BIO-024, BIO-112, BIO-161
- Access: game source, not redistributed
- Library (the project's source library, not distributed): the one class, text file MinecraftSrc1211/OverworldBiomeBuilder.java.
- Stage 2 read (WP-11, 2026-10-02): the temperature, humidity, continentalness, erosion and weirdness constants; all equal the bake's reference bands.
- Read 2026-10-03 (volcanic biomes): the depth parameters of `addSurfaceBiome` (points 0.0 and 1.0), `addUndergroundBiome` (span 0.2 to 0.9) and `addBottomBiome` (point 1.1); BIO-112.
- Read 2026-10-20 (the ice spikes report, in the workbench's NeoForge 21.1.249 sources jar on the test machine): `addHighSlice`, `addMidSlice` and `addLowSlice`, which pick each continentalness and erosion cell takes (in the frozen, driest cell near-inland and mid-inland differ at erosion E0 and E1, and at E5 in the high slice: in the low slice near-inland E0 and E1 take the middle biome, mid-inland the slope-if-cold pick; in the high slice near-inland E0 takes the slope biome, mid-inland the peak biome), and the tables `MIDDLE_BIOMES`, `MIDDLE_BIOMES_VARIANT`, `PLATEAU_BIOMES` and `PLATEAU_BIOMES_VARIANT` (ice spikes the positive-weirdness variant of the frozen, driest middle and plateau cells) and `mushroomFieldsContinentalness` (-1.2 to -1.05); BIO-161.

### DeCellesGiles1996_flexure
- Reference: DeCelles, P. G., Giles, K. A. (1996). Foreland basin systems. Basin Research 8(2):105-123. (Also `DeCellesGiles1996` in the Plate tectonics section, where stage 1 saw the abstract only.)
- DOI or URL: 10.1046/j.1365-2117.1996.01491.x; full text https://pages.uoregon.edu/rdorsey/BasinAnalysis/BasinPapers/DeCelles%20&%20Giles%201996.pdf
- Cited as: docs/design/flexure.md section 11
- Ledger ids: none (the acceptance figures of docs/design/plate-force-balance.md).
- Access: open copy.
- Read so far (by the project): full text by a Sonnet sub-agent of the flexure agent on 2026-10-02: "Foredeep depozones are typically 100-300 km wide and 2-8 km thick"; forebulges 60-470 km wide and a few tens to a few hundred metres high, as a flexural prediction.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct; this settles what stage 1 could not see.


### StewartJamieson2019
- Reference: Stewart, H. A., Jamieson, A. J. (2019). The five deeps: The location and depth of the deepest place in each of the world's oceans. Earth-Science Reviews 197:102896.
- DOI or URL: 10.1016/j.earscirev.2019.102896; open copy (CC BY 4.0) https://nora.nerc.ac.uk/id/eprint/524544/
- Cited as: "Stewart & Jamieson 2019" (surface/guard.rs)
- Ledger ids: ISO-071
- Access: open copy.
- Library (the project's source library, not distributed): not filed; the NORA PDF was fetched and its first two pages extracted on the laptop, 2026-10-18.
- Read so far (by the project): full text, pages 1 and 2, by the surface-divergence agent on 2026-10-18. Abstract: "Challenger Deep in the Mariana Trench (Pacific Ocean; 10,925 m, 11.332° N/142.202° E)"; p. 2: "Challenger Deep in the Mariana Trench is not > 11,000 m deep (10,925 m; van Haren et al., 2017)".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (journal, volume and article number on the PDF's first page).

### Dangol2020
- Reference: Dangol, S., Joshi, P., KC, S., Thapa, M., Banjara, B., KC, S., Bhandari, S. (2020). Measurement of Height of Mt. Sagarmatha. Journal on Geoinformatics, Nepal 20:59-66 (Survey Department, Nepal).
- DOI or URL: https://www.nepjol.info/index.php/NJG/article/download/39479/30219/114898
- Cited as: "Nepal and China, 8 December 2020" (surface/guard.rs)
- Ledger ids: ISO-071
- Access: open copy.
- Library (the project's source library, not distributed): not filed; the NepJOL PDF was fetched and extracted on the laptop, 2026-10-18.
- Read so far (by the project): full text by the surface-divergence agent on 2026-10-18. Abstract: "orthometric height was determined as 8848.86 m from the ellipsoid height observed at the top of Sagarmatha and precise geoid determined. The height was determined on the base of International Height Reference System (IHRS) and final height was announced jointly from Nepal and China on 8th of December 2020"; p. 64: "The final height was determined as 8848.86 m (IHRS) jointly". The Chinese side's account (Geo-spatial Information Science, doi 10.1080/10095020.2022.2128901) refused scripted access (403).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (journal and pages on the PDF's page footers).

### EshaghTenzer2021
- Reference: Eshagh, M., Tenzer, R. (2021). The temporal viscoelastic model of flexural isostasy for estimating the elastic thickness of the lithosphere. Geophysical Journal International 227(3):1700-1714.
- DOI or URL: 10.1093/gji/ggab292
- Cited as: "Eshagh & Tenzer 2021" (the limit a weakened plate tends to)
- Ledger ids: ISO-056
- Access: open (publisher).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): the Summary and Section 2.2 in full text (the first 100,000 of 163,909 characters) by a Sonnet sub-agent of the polar-holes agent, 2026-10-18: "The response of a viscoelastic plate on a load yields a flexure with values between those limited based on the Airy-Heiskanen (cf. Heiskanen 1931) and elastic plate theories"; at t = 0 "A viscoelastic plate thus acts as an elastic shell"; as t goes to infinity "the corresponding flexure will be the same as that defined by the Airy-Heiskanen isostatic model". D is fixed in their model and time enters through exp(-t/tau), tau = eta/E; they cite Watts 2001, p. 225, for the viscoelastic equation.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: journal, volume and pages as the sub-agent read them; not checked against the printed version.

### Buoninfante2023
- Reference: Buoninfante, S., et al. (2023). Gravity evidence for a heterogeneous crust of Mercury. Scientific Reports 13:19854.
- DOI or URL: 10.1038/s41598-023-46081-4
- Cited as: "Buoninfante et al. 2023" (the zero-rigidity limit)
- Ledger ids: ISO-056
- Access: open (PMC).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): full text (PMC) by a Sonnet sub-agent of the polar-holes agent, 2026-10-18: "the flexural rigidity D→0, and we may assume the local compensation Airy model".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: title, journal and article number as read; the author list after the first not looked up.

### Beaumont1978
- Reference: Beaumont, C. (1978). The evolution of sedimentary basins on a viscoelastic lithosphere: theory and examples. Geophysical Journal of the Royal Astronomical Society 55(2):471-497.
- DOI or URL: 10.1111/j.1365-246X.1978.tb04283.x
- Cited as: "Beaumont 1978"
- Ledger ids: ISO-056, ISO-099
- Access: paywalled (abstract visible on OUP).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract by a Sonnet sub-agent of the polar-holes agent, 2026-10-18: "long wavelengths being compensated most rapidly", "flexural rigidity of ~ 5 x 10^25 Nm and relaxation time constant ~ 10^6 yr". The abstract states no Airy limit.
- Read in the release-pace task (2026-10-23, the abstract through OpenAlex, by a Sonnet sub-agent of the release agent; ISO-099): "A model, in which a 50-km wide graben subsides exponentially with a time constant of 5 × 10^7 yr during the interval 180-100 Myr bp, is shown to be consistent with the largest scale structure of the North Sea Basin if the underlying lithosphere is viscoelastic with a flexural rigidity of ~ 5 × 10^25 Nm and relaxation time constant ~ 10^6 yr." The 10^6 yr is a fit to one basin.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (the year is 1978, not 1976 as it is sometimes given).

### WattsZhong2000
- Reference: Watts, A. B., Zhong, S. (2000). Observations of flexure and the rheology of oceanic lithosphere. Geophysical Journal International 142(3):855-875.
- DOI or URL: 10.1046/j.1365-246x.2000.00189.x
- Cited as: "Watts & Zhong 2000" (the counterpoint: an oceanic plate stays elastic on long timescales)
- Ledger ids: ISO-056, ISO-099
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract by a Sonnet sub-agent of the polar-holes agent, 2026-10-18: rigidity "depends on load age, being large at short times and small at long times"; the plate "behaves as a thin elastic plate on long timescales".
- Read in the release-pace task (2026-10-23, full text by a Sonnet sub-agent of the release agent: the text layer garbled, OCR, the key sentences checked on the page images, pp. 856 and 861; ISO-099): "The timescale over which this relaxation occurs is apparently in the range 0 to 1-2 Myr"; "no single relaxation time can explain all the data. Furthermore, no relaxation time can explain the data in the long term"; "~ 50 per cent of the subsequent flexure is reached within 10 ka" and "it takes a further 100 Myr to reach the same amount of flexure as occurred during the first 10 kyr of loading"; conclusion (2): "The decrease is most rapid during the first 1-2 Myr after loading, but appears to continue on longer timescales, although changes during 1-2 to 100 Myr are small"; conclusion (1): Te is "a function of the thermal age of the lithosphere at the time of loading, being well described by the depth to the 300-600 °C oceanic isotherms".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: as read in the abstract.

### Guevara2022
- Reference: Guevara, V. E., Smye, A. J., Caddick, M. J., Searle, M. P., Olsen, T., Whalen, L., Kylander-Clark, A. R. C., Jercinovic, M. J., Waters, D. J. (2022). A modern pulse of ultrafast exhumation and diachronous crustal melting in the Nanga Parbat Massif. Science Advances 8(31):eabm2689.
- DOI or URL: 10.1126/sciadv.abm2689; full text https://www.ebi.ac.uk/europepmc/webservices/rest/PMC9355356/fullTextXML
- Cited as: "Guevara et al. 2022" (Earth's fastest exhumation)
- Ledger ids: ISO-076
- Access: open (PMC9355356).
- Library (the project's source library, not distributed): not filed; fetched to the laptop's scratch by a Sonnet sub-agent of the polar-holes agent, 2026-10-18, and deleted with the task's scratch.
- Read so far (by the project): full text (Europe PMC XML), fetched by a Sonnet sub-agent and the values read by the polar-holes agent, 2026-10-18: "a pulse of ultrafast exhumation (9 to 13 mm/year) that began ~1 Ma and was preceded by several million years of slower, but still rapid, exhumation (2 to 5 mm/year)"; "the fastest known intracontinental exhumation rates on the modern Earth".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (title and article number in the XML).

### Govin2020
- Reference: Govin, G., van der Beek, P., Najman, Y., Millar, I., Gemignani, L., Huyghe, P., Dupont-Nivet, G., Bernet, M., Mark, C., Wijbrans, J. (2020). Early onset and late acceleration of rapid exhumation in the Namche Barwa syntaxis, eastern Himalaya. Geology 48(12):1139-1143.
- DOI or URL: 10.1130/G47720.1; full text https://nora.nerc.ac.uk/id/eprint/529465/1/1139.pdf
- Cited as: "Govin et al. 2020"
- Ledger ids: ISO-076
- Access: open (gold open access; NORA copy).
- Library (the project's source library, not distributed): not filed; fetched to the laptop's scratch by a Sonnet sub-agent of the polar-holes agent, 2026-10-18, and deleted with the task's scratch.
- Read so far (by the project): full text, fetched by a Sonnet sub-agent and the values read by the polar-holes agent, 2026-10-18: "Initial, intermediate, and final exhumation rates are 0.9 ± 0.4, 4.0 ± 2.0, and 8.6 ± 1.0 km/m.y., respectively" (the late acceleration at 1.3 ± 0.8 Ma).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (on the PDF).

### Deng2021
- Reference: Deng, K., Wittmann, H., Yang, S., von Blanckenburg, F. (2021). The upper limit of denudation rate measurement from cosmogenic 10Be(meteoric)/9Be ratios in Taiwan. Journal of Geophysical Research: Earth Surface 126(10):e2021JF006221.
- DOI or URL: 10.1029/2021JF006221; abstract https://api.crossref.org/works/10.1029/2021JF006221
- Cited as: "Deng et al. 2021"
- Ledger ids: ISO-077
- Access: paywalled (abstract through Crossref).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract (Crossref), fetched by a Sonnet sub-agent and read by the polar-holes agent, 2026-10-18: "8.1 to >30 mm/yr in the Liwu mainstem". A GFZ project page ("fastest denudation rates on Earth") corroborates it. The full text was not read.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: as read in the abstract.

### Lidberg2010
- Reference: Lidberg, M., Johansson, J. M., Scherneck, H.-G., Milne, G. A. (2010). Recent results based on continuous GPS observations of the GIA process in Fennoscandia from BIFROST. Journal of Geodynamics 50(1):8-18.
- DOI or URL: 10.1016/j.jog.2009.11.010; author preprint https://www.nordicgeodeticcommission.com/wp-content/uploads/2014/10/7-local_120398.pdf
- Cited as: "Lidberg et al. 2010" (the fastest glacial rebound measured)
- Ledger ids: ISO-076
- Access: paywalled; author preprint open.
- Library (the project's source library, not distributed): not filed; fetched to the laptop's scratch by a Sonnet sub-agent of the polar-holes agent, 2026-10-18, and deleted with the task's scratch.
- Read so far (by the project): the author preprint's velocity table, fetched by a Sonnet sub-agent and the values read by the polar-holes agent, 2026-10-18: station SKE0 (64.879 N, 21.048 E), vertical 10.95 mm/yr; OVE0 9.79, OULU 9.59.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (title and authors on the preprint); the preprint, not the printed version.

### Sella2007
- Reference: Sella, G. F., Stein, S., Dixon, T. H., Craymer, M., James, T. S., Mazzotti, S., Dokka, R. K. (2007). Observation of glacial isostatic adjustment in "stable" North America with GPS. Geophysical Research Letters 34(2):L02306.
- DOI or URL: 10.1029/2006GL027081; abstract https://digitalcommons.usf.edu/geo_facpub/447/
- Cited as: "Sella et al. 2007"
- Ledger ids: ISO-077
- Access: paywalled (abstract on the USF repository).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract on the USF repository by a Sonnet sub-agent of the polar-holes agent, 2026-10-18: uplift of about 10 mm/yr near Hudson Bay. The full text was not read (Wiley 403).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: as read in the abstract.

### Brothers2009
- Reference: Brothers, D. S., Driscoll, N. W., Kent, G. M., Harding, A. J., Babcock, J. M., Baskin, R. L. (2009). Tectonic evolution of the Salton Sea inferred from seismic reflection data. Nature Geoscience 2:581-584.
- DOI or URL: 10.1038/NGEO590; full text https://igppweb.ucsd.edu/~dkilb/NG/SaltonSea/Publications_files/brothers_NG_2009.pdf
- Cited as: "Brothers et al. 2009" (fast subsidence)
- Ledger ids: ISO-076
- Access: paywalled; a copy on an author's institution's site.
- Library (the project's source library, not distributed): not filed; fetched to the laptop's scratch by a Sonnet sub-agent of the polar-holes agent, 2026-10-18, and deleted with the task's scratch.
- Read so far (by the project): full text, fetched by a Sonnet sub-agent and the values read by the polar-holes agent, 2026-10-18: "Differential subsidence (>10 mm yr−1) in the southern sub-basin"; "Average subsidence measured over the ad 830 time-horizon just south of the hinge zone is between 9 and 12 mm yr−1"; sedimentation "nearly 19 mm yr−1 since ∼ad 1360".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (on the PDF).

### Garzione2008
- Reference: Garzione, C. N., et al. (2008). Rise of the Andes. Science 320(5881):1304-1307.
- DOI or URL: 10.1126/science.1148615; copy https://www.sas.rochester.edu/ees/SIREAL/CAUGHT/css/publication/2008/Garzione%20et%20al_2008.pdf
- Cited as: "Garzione et al. 2008" (a fast plateau rise sustained over millions of years)
- Ledger ids: ISO-076
- Access: paywalled; a copy on the first author's institution's site.
- Library (the project's source library, not distributed): not filed; fetched to the laptop's scratch by a Sonnet sub-agent of the polar-holes agent, 2026-10-18, and deleted with the task's scratch.
- Read so far (by the project): full text, fetched by a Sonnet sub-agent and the values read by the polar-holes agent, 2026-10-18: "a pulse of rapid surface uplift of ~1.5 to 3.5 km (2.5 ± 1 km) between ~10 and 6 Ma". A correction to the article exists (Science 321:1295) and was not read.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (title, volume and first page on the copy's cover page). Not GarzioneEtAl2006, a different paper.

### XieHeller2009
- Reference: Xie, X., Heller, P. L. (2009). Plate tectonics and basin subsidence history. Geological Society of America Bulletin 121(1-2):55-64.
- DOI or URL: 10.1130/B26398.1; copy https://serc.carleton.edu/download/files/49990/xie_heller_2009.pdf
- Cited as: "Xie & Heller 2009" (how much a basin subsides, and for how long)
- Ledger ids: ISO-076
- Access: paywalled; a copy on SERC.
- Library (the project's source library, not distributed): not filed; fetched to the laptop's scratch by a Sonnet sub-agent of the polar-holes agent, 2026-10-18, and deleted with the task's scratch.
- Read so far (by the project): full text, fetched by a Sonnet sub-agent and the values read by the polar-holes agent, 2026-10-18: foreland basins, "The maximum magnitude of tectonic subsidence seen in compiled curves is ~3 km"; strike-slip basins, "rapid and short-lived (typically <10 m.y.) subsidence", "Tectonic subsidence typically exceeds 2 km and reaches 4 km in exceptional cases". Magnitudes and durations, not rates.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct (on the PDF).

### WattsBurov2003
- Reference: Watts, A. B., Burov, E. B. (2003). Lithospheric strength and its relationship to the elastic and seismogenic layer thickness. Earth and Planetary Science Letters 213(1-2):113-131.
- DOI or URL: 10.1016/S0012-821X(03)00289-9; copy https://www.whoi.edu/fileserver.do?id=114324&pt=10&p=75973
- Cited as: "Watts & Burov 2003" (a plate yields where it is bent hard)
- Ledger ids: ISO-093, ISO-120
- Access: paywalled; a copy on WHOI's file server.
- Library (the project's source library, not distributed): not filed; fetched to /tmp by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22 (pdftotext; its minus signs come out as "3", decoded with the paper's own 5e-6 and 1.3e-6).
- Read so far (by the project): full text, by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22: "Te slowly decreases with increasing curvature and, hence, bending stress"; for 80 Ma lithosphere at a strain rate of 1e-14 s^-1 the ratio Te(YSE)/Te(elastic) is 1 for K <= 1e-8 m^-1, and "For K = 1e-6 m^-1 the ratio is ~0.5, indicating 50% yielding"; at the northern Chile trench (K = 1.3e-6 m^-1) "Te is 22 +- 2 km" against 34 km from the plate's age; "Ideal elastic materials support any stress level. In the case of real materials, however, stress levels are limited to the rock strength at the corresponding depth"; Byerlee's brittle strength at the Moho (~35 km) 0.6 GPa (tension) to 2.0 GPa (compression).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: journal, volume, pages and DOI as the sub-agent read them; the title is from memory, not checked.

### HunterWatts2016
- Reference: Hunter, J., Watts, A. B. (2016). Gravity anomalies, flexure and mantle rheology seaward of circum-Pacific trenches. Geophysical Journal International 207(1):288-316.
- DOI or URL: 10.1093/gji/ggw275; https://academic.oup.com/gji/article/207/1/288/2583844
- Cited as: "Hunter & Watts 2016" (an elastic plate's stresses are too high; Te falls landward of the outer rise)
- Ledger ids: ISO-093, ISO-120
- Access: open (OUP).
- Library (the project's source library, not distributed): not filed; read on the publisher's page.
- Read so far (by the project): about 200,000 of 279,000 characters, all of section 8 and Appendix C, by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22: "the elastic model, unlike viscous models, predicts unreasonably high bending stresses (up to 900 MPa) in the outermost portions of the flexed plate"; weakening switched on at "kappa = 1x10^-7 m^-1"; "Te landward of the outer rise is generally 40-65 per cent less than the Te seaward" (mean 51 +- 11 %, range 20-72 %); the outer rise's gravity high "<16 mGal at the Middle America and Peru-Chile trenches to up to 40 mGal at old (>80 Ma) trenches".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: journal, volume, pages and DOI as the sub-agent read them; the title is from memory, not checked.

### Watts2025_Hawaii
- Reference: Watts, A. B., Xu, C., Wessel, P., Shillington, D. J., Boston, B., Dunn, R. A. (2025). Seismic and gravity constraints on plate flexure and mantle rheology along the whole Hawaiian-Emperor seamount chain. Nature Communications 16:11013.
- DOI or URL: 10.1038/s41467-025-65442-3
- Cited as: "Watts et al. 2025" (the largest flexurally held load on Earth, in gravity)
- Ledger ids: ISO-093, ISO-120
- Access: open.
- Library (the project's source library, not distributed): not filed; read on the publisher's PDF.
- Read so far (by the project): full text, by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22: "observed free-air gravity anomalies that are in the range -100 to +300 mGal"; Vening Meinesz and Gunn showed "the southeast end of the chain was regionally rather than locally isostatically compensated"; "It is apparently only in high curvature regions of the seaward wall of a trench and the region immediately beneath a large volcanic edifice where bending stresses are sufficiently high that the flexed oceanic plate weakens".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.

### tenBrink2004
- Reference: ten Brink, U. S., Danforth, W., Polloni, C., et al. (2004). New seafloor map of the Puerto Rico Trench helps assess earthquake and tsunami hazards. Eos, Transactions American Geophysical Union 85(37):349-354.
- DOI or URL: copy https://earthjay.com/earthquakes/20180928_indonesia/ten_brink_etal_2004_seafloor_map_puerto_rico_trench.pdf
- Cited as: "ten Brink et al. 2004" (the most negative free-air anomaly on Earth)
- Ledger ids: ISO-093, ISO-120
- Access: paywalled; a copy on a teaching site.
- Library (the project's source library, not distributed): not filed.
- Read so far (by the project): full text of the Eos note, by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22: "the Puerto Rico Trench is also characterized by the most negative free-air gravity anomaly on Earth, -380 mGal, located 50 km south of the trench, where water depth is 7950 m".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: the first three authors, volume and pages as the sub-agent read them; the title is from memory, not checked.

### Stern_OceanTrenches
- Reference: Stern, R. J. Ocean Trenches. In: Encyclopedia of Geology (Elsevier), Earth Processes.
- DOI or URL: proof copy https://personal.utdallas.edu/~rjstern/pdfs/TrenchProof.pdf
- Cited as: "Stern, Ocean Trenches" (trenches against isostasy, in gravity)
- Ledger ids: ISO-093, ISO-097
- Access: the author's proof copy.
- Library (the project's source library, not distributed): not filed.
- Read so far (by the project): full text (proof), by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22: "Typically, the gravity over the outer swell is about 50 mGals higher than expected from isostasy, while gravity over the trench is about 200 mGals less than that expected from isostatic considerations".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: the edition and year of the encyclopedia were not recorded.
- Read 2026-10-23 (the trench rows, ISO-097): the 2021 Encyclopedia of Geology chapter, https://personal.utdallas.edu/~rjstern/pdfs/OceanicTrenchesEoG221.pdf, by a Sonnet sub-agent of the trench-row agent: the elastic thickness of the bent plate is 20 to 30 km for most trench profiles.

### Cattin2001
- Reference: Cattin, R., et al. (2001). Gravity anomalies, crustal structure and thermo-mechanical support of the Himalaya of Central Nepal. Geophysical Journal International 147(2):381-392.
- DOI or URL: https://academic.oup.com/gji/article/147/2/381/719559
- Cited as: "Cattin et al. 2001" (the Himalaya held up and its foreland pulled down)
- Ledger ids: ISO-093
- Access: open (OUP).
- Library (the project's source library, not distributed): not filed.
- Read so far (by the project): about 100,000 of 130,000 characters, by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22: the Bouguer anomaly goes "from about -40 to -500 mgal"; "The foreland appears overcompensated while the high range is undercompensated"; Ganga basin Te 40-50 km. No isostatic-anomaly number.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: first author, journal, volume and pages as the sub-agent read them; the title is from memory, not checked.

### McNuttMenard1982
- Reference: McNutt, M. K., Menard, H. W. (1982). Constraints on yield strength in the oceanic lithosphere derived from observations of flexure. Geophysical Journal of the Royal Astronomical Society 71(2):363-394.
- DOI or URL: https://academic.oup.com/gji/article/71/2/363/579182
- Cited as: "McNutt & Menard 1982" (bending moment saturates at high curvature)
- Ledger ids: ISO-093
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract, by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22: the bending moment saturates at high curvature, read as a depth-dependent yield strength; old oceanic lithosphere has "no long-term strength below about 40 km depth"; the method "is most appropriate for trench profiles with curvatures greater than 10^-7 m^-1".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: journal, volume and pages as the sub-agent read them; the title is from memory, not checked.

### GoetzeEvans1979
- Reference: Goetze, C., Evans, B. (1979). Stress and temperature in the bending lithosphere as constrained by experimental rock mechanics. Geophysical Journal of the Royal Astronomical Society 59(3):463-478.
- DOI or URL: https://academic.oup.com/gji/article/59/3/463/651917
- Cited as: "Goetze & Evans 1979" (the yield-strength envelope)
- Ledger ids: ISO-093
- Access: paywalled (abstract visible).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): the fetch tool's paraphrase of the abstract, by a Sonnet sub-agent of the flexure-guard agent, 2026-10-22: a limiting yield-strength curve from brittle and ductile laws; stresses must reach "values greater than 100 MPa" at 25-45 km depth. A paraphrase, not a quotation.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: journal, volume and pages as the sub-agent read them; the title is from memory, not checked.

### CODATA2022_G
- Reference: Mohr, P. J., Newell, D. B., Taylor, B. N., Tiesinga, E. CODATA recommended values of the fundamental physical constants: 2022. NIST Reference on Constants, Units, and Uncertainty.
- DOI or URL: https://physics.nist.gov/cgi-bin/cuu/Value?bg
- Cited as: "CODATA 2022" (the Newtonian constant of gravitation)
- Ledger ids: ISO-120
- Access: open (NIST).
- Library (the project's source library, not distributed): not filed; read on NIST's page.
- Read so far (by the project): NIST's value page, by the polar-peaks agent, 2026-10-23: "6.674 30 x 10-11 m3 kg-1 s-2", standard uncertainty "0.000 15 x 10-11", "Source: 2022 CODATA". That the 2018 adjustment gave the same value is from memory.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: the page and its value as read; the authors of the 2022 adjustment are from memory, not checked.

### Snyder1987
- Reference: Snyder, J. P. (1987). Map Projections: A Working Manual. U.S. Geological Survey Professional Paper 1395. Washington, D.C.: U.S. Government Printing Office. ix + 385 pp.
- DOI or URL: 10.3133/pp1395; https://pubs.usgs.gov/pp/1395/report.pdf
- Cited as: "Snyder 1987" (the cylindrical equal-area projection's scale factors)
- Ledger ids: ISO-127
- Access: open (USGS).
- Library (the project's source library, not distributed): not filed; read in the USGS PDF.
- Read so far (by the project): chapter 10, "Cylindrical Equal-Area Projection" (pp. 76 to 85), chapter 4 (p. 21) and the table footnote on p. 27, by a Sonnet sub-agent of the flexure-faults agent, 2026-10-23, the equations checked on the rendered page image. P. 77, "FORMULAS FOR THE SPHERE": x = R (λ − λ0) cos φs (10-1), y = R sin φ / cos φs (10-2), h = cos φ / cos φs (10-2a), k = 1/h (10-2b), "where φs is the standard parallel (N. or S.), or the Equator in Lambert's original form"; "There is no distortion of area anywhere on the projections"; "the parallels are spaced from the Equator in proportion to the sine of the latitude". P. 21: the scale factor "along the meridian is called h and along the parallel, k".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: title, series and pages from the USGS record.

### Hetenyi1946
- Reference: Hetényi, M. (1946). Beams on Elastic Foundation: Theory with Applications in the Fields of Civil and Mechanical Engineering. University of Michigan Studies, Scientific Series, Vol. XVI. Ann Arbor: The University of Michigan Press.
- DOI or URL: https://archive.org/details/in.ernet.dli.2015.73919 (a scan of the 1955 fourth printing)
- Cited as: "Hetényi 1946" (a uniform load over a finite length of a beam on an elastic foundation)
- Ledger ids: ISO-127
- Access: open scan (archive.org, Digital Library of India).
- Library (the project's source library, not distributed): not filed.
- Read so far (by the project): chapter 1, pp. 3 to 17, in the scan's text and on page images of pp. 3, 4, 15 and 16, by a Sonnet sub-agent of the flexure-faults agent, 2026-10-23. Eq. (1) EI d⁴y/dx⁴ = −ky + q; λ = (k/4EI)^¼ (eq. f, p. 4); D_λx = e^(−λx) cos λx (eq. h, p. 12). Section 5, "Uniformly Distributed Loading": under the load, "y_C = (q/2k)(2 − D_λa − D_λb)" (7a, p. 15), a and b the distances to the load's two ends; to the left of it "y_C = (q/2k)(D_λa − D_λb)" (8a, p. 16), and to the right the same with a and b exchanged and the sign changed (9a).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: title, series and publisher from the scan's title page.

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
- Ledger ids: PLT-213 (and PLT-214, PLT-221 by reference), PLT-401 (2026-10-21: the CCB's length, speed and area), PLT-500 and PLT-501 (2026-10-23: the boundary measure's shares and the trenches' facing), PLT-502 (2026-10-23: PB2002_steps.dat's segments), PLT-531 (2026-10-24: the boundary's geometry, its pieces, junctions, tortuosity and the plates' perimeters), PLT-532 (2026-10-24: a plate is one region, [112] and [123]). The same work is under `Bird2003` in the plate tectonics and climate sections for other claims.
- Access: open (the author's site).
- Read so far (by the project): full text, 2026-10-02: Table 3 "Collective Properties of Plate Boundaries by Class": CCB 23,003 km (8.8 %), CTF 26,132 (10.0 %), CRB 27,472 (10.5 %), OSR 67,338 (25.8 %), OTF 47,783 (18.3 %), OCB 17,449 (6.7 %), SUB 51,310 (19.7 %), total 260,487 km (100 %); paragraph 126 defines SUB as convergent steps "with a Benioff zone of intermediate to deep earthquakes and/or a parallel volcanic arc with Quaternary activity", "not divided into continental and oceanic types"; 5,819 steps of mean length 44.7 km. 2026-10-23 (V1 gap 2): SUB's area flux 0.086516 m2/s over 51,310 km is 53.2 mm/yr across the trench, against a mean relative speed of 62.3 (the agent's arithmetic on Table 3); and PB2002_steps.dat (the steps file beside the paper, `Bird2003/PB2002_steps.dat`) read with a lane script: OSR runs median 75 km and 165 km length-weighted, transform runs 0.71 of the ridge's length, 82 % of the ridge and 49 % of the subduction zones within 20 degrees of orthogonal to the relative motion.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: correct.
- Read again 2026-10-21 (the crust flux into collision fronts, PLT-401; the library copy `Bird2003/Bird2003.txt`): Table 3's other columns for the continental convergent boundary, "23,003 (8.8%)" km, mean velocity "26.2" mm/a, area production "0.013616 (12.6%)" m2/s (consumed: 0.43 km2 a year; over the length, 18.7 mm/yr of normal convergence, against the full vector's 26.2); paragraph 127: "Continental plate boundary steps are any that are not oceanic. (Thus, transitional steps are considered continental.)", oceanic steps being those "within seafloor whose age is known from linear magnetic anomalies to be less than 180 Ma" and/or "at water depths exceeding 2000 m", and SUB steps classed first, so a CCB step is a convergent step on continental or transitional lithosphere with no Benioff zone or arc.
- Read again 2026-10-23 (the docking lane's diagnosis, plates.md "What transference exposes"; the library copy `Bird2003/Bird2003.txt`): Table 3's area production by class, m2/s: CRB +0.011502 (continental rift boundaries: +0.36 km2 a year made, against the CCB's 0.013616, 0.43 km2 a year consumed), SUB 0.086516, OSR +0.095348, OCB 0.007141, CTF 0.000599, OTF +0.001022.
- Read again 2026-10-24 (the front lane, plates.md "The front's length: why the plates are ragged"; the paper's text and its auxiliary files `PB2002_boundaries.dig`, `PB2002_plates.dig` and `PB2002_steps.dat`, kept in the library's `Bird2003/` with the lane's output `PB2002_geometry.json`): [110] "plates + junctions = boundaries + 2", and with triple junctions only and no island plates "junctions = 2 x (plates - 2), and boundaries = 3 x (plates - 2)"; [111] "52 plates ... Together they exactly cover the Earth, with no gaps or overlaps. There are 100 triple-junction points"; [112] "A plate boundary is a continuous simply-connected curve on the surface of the Earth which everywhere separates the same two different plates, and either (1) begins at a junction and ends at a junction, or (2) closes on itself in an isolated loop. There are 150 plate boundaries in model PB2002, and all are of type 1", the Pacific with 23 neighbours, 6 small plates with 2; [118] the soft limit of plate size near Africa's, "The plate width at which this soft limit occurs is about one Earth radius", mantle convection exerting tractions on larger plates "that tend to break them up"; [122] the boundaries file, 6,048 points in 229 segments; [123] the plates file (52 closed outlines) and "5,819 steps in model PB2002, with mean length of 44.7 km and length range from 1 km to 109 km". Measured from the files with the lane's scripts (`tools/review/pool/bird/bird.py`, Bird's divider walk of a fixed chord, and `bird_raster.py`, PB2002 on the model's 100,000-point lattice): the boundary 246,541 km at a 77 km chord and 231,009 at 380 km, the plates' perimeters over their spherical caps' 1.418 and 1.274, tortuosity (a piece's length over the great circle between its junctions) sum over sum 1.220 and 1.143, the mostly-SUB pieces' 1.145 and 1.110 (ledger PLT-531 has the rest). Also for PLT-532 the same day: [123] PB2002_plates.dig "contains 52 segments, each titled with the two-letter identifier of a plate. Each segment is a closed curve outlining that plate"; [112] "The Shetland plate comes closest to being an 'island' plate (type 2) because it is almost surrounded by the Antarctica plate, but it also has a short boundary with the Scotia plate"; [79] "The term 'microplate' has not been precisely defined, but in practice is used most often for a plate of 100–500 km dimensions which appears spontaneously in the center of an ocean spreading system, and rotates rapidly with respect to its neighboring plates".

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

### Davies2010
- Reference: Davies, J.H. and Davies, D.R. 2010. Earth's surface heat flux. Solid Earth 1:5-24, doi 10.5194/se-1-5-2010 (Creative Commons Attribution 3.0).
- Ledger ids: GEO-052, GEO-057, GEO-059, GEO-060
- Access: open.
- Library: `Davies2010/Davies2010.pdf` and `Davies2010.txt`.
- Read in this pass (GEO heat flow, 2026-10-02): full text of Tables 3 and 7 and Eq. 2. Continents (2.073e14 m2): 14.7 TW, mean 70.9 mW/m2; oceans 105.4; global 91.6. Mean heat flow by geology: Cenozoic extensional 102.4 (16 observations), Quaternary 82.0, Tertiary 77.3, continental and island arc margins 73.8, Precambrian 59.9. The empirical continental fit q = 102 - 2.25 sqrt(t) (t in Myr, Precambrian excluded; R squared 0.75 for the category means).

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

## Inherited iron-formation basins and richer deposits (ORE-261 to ORE-266)

Added 2026-10-02 for the owner's answers "Earth-like patch field" (inherited banded iron formation)
and "Rich Deposits, Thinner Scatter, Also make deposits use some of the vanilla ore vein features".
A research sub-agent (Sonnet) fetched the texts into the source library on the test machine; the
quotes below are what was read. The counts table at the top of this file was not recounted for
these entries.

### WAEPA1974_HamersleyIronProvince
- Reference: Conservation Through Reserves Committee, 1974, Report to the Environmental Protection Authority (Western Australia), Section Two, Systems Eight-Twelve, p. 8-1/8-2.
- DOI or URL: https://library.dbca.wa.gov.au/static/Journals/080500/080500-03.02.pdf
- Ledger ids: ORE-261, ORE-263
- Access: open (government report).
- Library: `WAEPA1974_HamersleyIronProvince/` (excerpt of the OCR text). Read: "The Hamersley Iron Province is broadly an ovoid depositional basin some 100,000 km2 in area".

### MarshSchlinger1980
- Reference: Marsh, B.D. and Schlinger, C.M., 1980, Petrologic and geophysical study of the source of long wavelength crustal magnetic anomalies. Final report, NASA CR-175168, Johns Hopkins University.
- DOI or URL: https://ntrs.nasa.gov/api/citations/19840012878/downloads/19840012878.pdf
- Ledger ids: ORE-261, ORE-263
- Access: open.
- Library: `MarshSchlinger1980/` (full text, a secondary compilation). Read: "The Hammersley group may have once extended over 150,000 km2 with average thickness of 1000 m (Trendall, 1973)"; "the Caue itabirite is thought to have once covered 100,000 km2 ... (Dorr, 1973)".

### Beukes1987
- Reference: Beukes, N.J., 1987, Facies relations, depositional environments and diagenesis in a major early Proterozoic stromatolitic carbonate platform to basinal sequence, Campbellrand Subgroup, Transvaal Supergroup, Southern Africa. Sedimentary Geology 54:1-46.
- Ledger ids: ORE-261, ORE-263
- Access: paywalled (abstract read).
- Library: `Beukes1987/` (abstract, University of Johannesburg repository). Read: the platform "covers an area of approximately 190,000 km2 on the Kaapvaal craton. The original depository probably extended across the entire 600,000 km2 surface area of the craton and beyond." (the carbonate platform under the Kuruman and Penge iron formations, not the iron formation alone).

### Pufahl1996
- Reference: Pufahl, P.K., 1996, MSc thesis, Lakehead University (Gunflint and Biwabik iron formations, Animikie Basin), citing Morey 1983.
- DOI or URL: https://knowledgecommons.lakeheadu.ca/bitstream/handle/2453/2470/PufahlP1996m-1b.pdf
- Ledger ids: ORE-261, ORE-263
- Access: open.
- Library: `Pufahl1996/` (full text, OCR). Read: the Lake Superior iron formations crop out "as an oval shaped region encompassing 220 000 km2 (Morey, 1983)" (the region enclosing the iron ranges).

### Bunting1986
- Reference: Bunting, J.A., 1986, Geology of the eastern part of the Nabberu Basin, Western Australia. Geological Survey of Western Australia Bulletin 131.
- DOI or URL: https://library.dbca.wa.gov.au/static/Journals/080326/080326-131.pdf
- Ledger ids: ORE-261, ORE-263
- Access: open (government report).
- Library: `Bunting1986/` (excerpts of the text). Read: "The Nabberu Basin is exposed over an area of approximately 60 000 km2 and is 600 km long by 120 km wide"; "None of the present boundaries represents the original depositional extent of the basin."

### WikipediaKurskMagneticAnomaly, WikipediaIronQuadrangle, LabradorTrough_dimensions (secondary)
- References: Wikipedia, "Kursk Magnetic Anomaly" and "Iron Quadrangle" (fetched 2026-10-02); Quebec geological survey lexicon, Kaniapiskau Supergroup (https://gq.mines.gouv.qc.ca/lexique-stratigraphique/province-de-churchill/supergroupe-de-kaniapiskau_en/).
- Ledger ids: ORE-261, ORE-263
- Library: the three folders of these keys. Read: the Kursk ores "are spread over an area estimated at 120,000 km2"; the Quadrilátero Ferrífero covers "about 7,000 square kilometres"; the Labrador Trough "forms a narrow strip up to 160 km wide that extends for 850 km" (no published area: the project's estimate is about 50,000 km², 850 km by about 60 km).

### Hasterok2022
- Reference: Hasterok, D., Halpin, J.A., Hand, M., Collins, A.S., Kreemer, C., Gard, M.G. and Glorie, S., 2022, New maps of global geological provinces and tectonic plates. Earth-Science Reviews 231:104069.
- DOI or URL: 10.1016/j.earscirev.2022.104069; data https://github.com/dhasterok/global_tectonics
- Ledger ids: ORE-261
- Access: open data (the paper not read).
- Library: `Hasterok2022/` (the research sub-agent's areas computed from the open shapefiles in an equal-area projection; a derived analysis, not source text). Read: continental crust 172.7 million km²; provinces typed shield (34.4) and craton (19.1), overlap removed, 53.5 million km²; the cratons layer (Archean basement) 40.8 million km².

### Abbott2000
- Reference: Abbott, D., Sparks, D., Herzberg, C., Mooney, W., Nikishin, A. and Zhang, Y.S., 2000, Quantifying Precambrian crustal extraction: the root is the answer. Tectonophysics 322:163-190.
- DOI or URL: https://escweb.wr.usgs.gov/share/mooney/2000_Tecton_PrecambExtraction.pdf
- Ledger ids: ORE-261
- Access: open copy (USGS author page).
- Library: `Abbott2000/` (full text). Read, Table 1: surface area of Archean crust 36.8 / 40.9 / 45.0 million km² (min / mean / max), Early Proterozoic 29.3 / 32.5 / 35.7 (the mean printed as 132.5, corrected from a later table).

### Klein2005
- Reference: Klein, C., 2005, Some Precambrian banded iron-formations (BIFs) from around the world: their age, geologic setting, mineralogy, metamorphism, geochemistry, and origin. American Mineralogist 90:1473-1499.
- DOI or URL: 10.2138/am.2005.1871
- Ledger ids: none (asked for; holds no area figure)
- Access: paywalled (abstract read).
- Library: `Klein2005/` (abstract). No basin area is stated in the abstract.

### MinecraftSources: OreVeinifier (ORE-266)
- See MinecraftSources above. Read 2026-10-02 for ORE-266: `net.minecraft.world.level.levelgen.OreVeinifier`: inside a vein a block is ore with probability 0.1 to 0.3 (`MIN_RICHNESS`, `MAX_RICHNESS`) of the 0.7 that are solid (`VEIN_SOLIDNESS`), and an ore block is the raw-ore block with `CHANCE_OF_RAW_ORE_BLOCK` = 0.02; the rest is filler (granite in copper veins, tuff in iron veins).

### MinecraftSources: falling blocks and brushable block entities (ORE-269)
- See MinecraftSources above. Read 2026-10-16 for ORE-269 (NeoForge 21.1.249 / Minecraft 1.21.1 decompiled sources): `FallingBlock.isFree` (a block under a gravity block is free when it is air, fire, a liquid or replaceable, so suspicious gravel and sand fall like their plain blocks); `BrushableBlockEntity` (the loot table and its seed are saved as `LootTable` and `LootTableSeed`; `unpackLootTable` rolls the table with the CHEST parameter set and takes only the first stack); `LootTable.getRandomItems(params, seed)` (a seed of 0 is not applied, the level's random sequence is used); `WorldGenRegion.setBlock` and `getBlockEntity` (a block with a block entity set during generation leaves a placeholder that `getBlockEntity` turns into the real block entity). Nothing here is a source for the physics of panning (ORE-267 is `from-memory`).
- Read again 2026-10-17 for ORE-269 (the same sources): `BrushableBlockEntity.loadAdditional` reads `item` only when there is no `LootTable`, and `saveAdditional` writes `item` only when there is no table; `brush` needs 10 strokes at least 10 ticks apart, then `dropContent` drops `item.split(10 to 30)` once and the block becomes its `turnsInto` (gravel, sand); `LootTable.getRandomItems(params, seed)` is `getRandomItemsRaw` through a stack splitter and then `CommonHooks.modifyLoot` (NeoForge's global loot modifiers), with `LootContext.Builder.withOptionalRandomSeed(seed)` = `RandomSource.create(seed)` for a seed that is not 0; `ProtoChunk.getBlockEntity` returns only block entity objects, while `getBlockEntityNbtForSaving` falls back to the pending tag (a chunk read back from disk below full status holds its block entities only as tags); `GameTestServer.create` (NeoForge 21.1.249 sources) selects every available pack (`new DataPackConfig(new ArrayList<>(packRepository.getAvailableIds()), List.of())`), feature packs too.
- Read again 2026-10-17 for ORE-269, after the second MIC gate failure (NeoForge 21.1.249 sources, read on the test VM): `ChunkSerializer.read` returns a chunk saved in full as `new ImposterProtoChunk((LevelChunk) chunk, false)` and leaves its `block_entities` list inside the `LevelChunk.PostLoadProcessor` of `postLoadChunk` (`loadStatic` and `setBlockEntity`, or `setBlockEntityNbt` for `keepPacked`), which `ChunkStatusTasks.full` runs (`runPostLoad`) only when the chunk is made full again; until then the chunk has no block entity object and no pending tag. `ImposterProtoChunk.isUnsaved` is always false and `ChunkMap.save` returns at once for a chunk that is not unsaved, so such a chunk dropped below full is not written again and its saved block entities stay as they were.

### Lootr 1.21.1-1.11.38.125: what it converts (ORE-269)
- Lootr (noobanidus), `lootr-neoforge-1.21.1-1.11.38.125.jar` from MIC's server set, decompiled with CFR on the test VM 2026-10-17 (read, not redistributed). `MixinLevelChunk` hooks `LevelChunk.updateBlockEntityTicker` and hands every block entity of a level chunk to `BlockEntityTicker.addEntity` (also `MixinBrushableBlockEntity` on `tryLoadLootTable`), which keeps it unless Lootr is disabled, the dimension is blocked, its type is in `#lootr:convert/blacklist` or it is Lootr's own. On each server tick (`HandleTick`) an entry whose chunk and the 5 by 5 chunks around it are loaded at full status is processed: it is skipped when its loot table is null (silently by default, `skip_logging_no_loot_table_at_generation`), when a structure rule of the config excludes it, when the table is in the config's `loot_table_blacklist` or `loot_modid_blacklist`, or when the block has no Lootr replacement; otherwise the vanilla table is cleared and the block is set to Lootr's block (`#lootr:convert/gravels` holds `minecraft:suspicious_gravel`, `/sands` `minecraft:suspicious_sand`) with the table and seed, per player. The jar has no inter-mod message handler and no API to exempt a block entity; its built-in feature datapack `lootr_no_suspicious_blocks` (off by default) empties both convert tags for every block. `BlockEntityTicker.isValidEntity` admits an entry without looking at its loot table; the table is read when the entry is processed (`replaceEntitiesInChunk`), so a table set after the block was placed still counts. `LoadedChunks` holds the chunks whose `ChunkEvent.Load` came at full status. `HandleResourcePacks` registers `lootr_no_suspicious_blocks` with `PackSource.FEATURE`, not required, at `Pack.Position.TOP`; MIC's pack (branch `deep-time`, read 2026-10-17) has no Lootr config and enables the pack nowhere (no `lootr_no_suspicious_blocks`, no `initial-enabled-packs`). Used for ORE-269: a brushable block entity with no loot table is never converted.

## Rock classes, metamorphism, plutons and strata: sources read in WP-7 (RCK, STR)

Added in stage 2, package WP-7 (2026-10-02). Each entry is held in the project's source library (the test machine's `deeptime-sources` folder, one folder per key; not distributed, never committed) unless it says "no saved copy". "Read" says what was read and at what level; secondary means lecture notes, encyclopedias or a software manual that tabulates a paper.

### DalyMangerClark1966
- Reference: Daly, R. A., Manger, G. E. and Clark, S. P. (1966). Density of rocks. In: Clark, S. P. (ed.), Handbook of Physical Constants, Geological Society of America Memoir 97, section 4, pp. 19-26.
- Ledger ids: RCK-001, RCK-087
- Library: `DalyMangerClark1966/DalyMangerClark1966.pdf` and `.txt` (a copy of the GSA chapter). Tables 4-1 (holocrystalline igneous), 4-2 (natural glasses) and 4-5 (metamorphic) legible; table 4-4 (sedimentary) is a rotated page and did not extract.
- Read: primary full text. Granite 155 samples mean 2.667 (2.516-2.809); granodiorite 2.716; diorite 2.839; gabbro 2.976; peridotite (fresh) 3.234 (3.152-3.276); dunite 3.277; anorthosite 2.734; rhyolite obsidian 2.370; slate 2.81; schist 2.70-3.03; gneiss 2.59-3.06; amphibolite 2.99; eclogite 3.392. Fresh rock, so upper bounds for field rock.

### Manger1963
- Reference: Manger, G. E. (1963). Porosity and bulk density of sedimentary rocks. U.S. Geological Survey Bulletin 1144-E.
- Ledger ids: RCK-001, RCK-087, STR-049
- Library: `Manger1963/Manger1963.pdf` and `.txt` (OCR text, columns scrambled; rows read where clean). https://pubs.usgs.gov/bul/1144e/report.pdf
- Read: primary full text. Sandstone 2.13-2.69 dry; limestone, dolomite, marble, chert, conglomerate, chalk (Great Britain, 31 samples, porosity 17.6-42.8 %, dry 1.94), shale 1.73-2.71, slate 2.74-2.84, loose sand, gravel, silt, clay, loess, soils; Phosphoria Formation 2.91; iron formation and ironstone ore 3.46-4.53, Biwabik 2.75.

### HandbookOfMineralogy
- Reference: Anthony, J. W., Bideaux, R. A., Bladh, K. W. and Nichols, M. C. (eds), Handbook of Mineralogy. Mineralogical Society of America. https://www.handbookofmineralogy.org
- Ledger ids: RCK-085, RCK-088
- Library: `HandbookOfMineralogy/` (quartz, calcite, dolomite, halite, gypsum, anhydrite, hematite, magnetite, fluorapatite, albite, anorthite; PDF and text).
- Read: primary reference text. Quartz H 7 D 2.65; calcite H 3 D 2.7102; dolomite D 2.86; halite H 2-2.5 D 2.168; gypsum H 1.5-2 D 2.317; anhydrite H 3-3.5 D 2.98; hematite D 5.26; magnetite D 5.175; fluorapatite D 3.1-3.25; albite 2.60-2.65, anorthite 2.74-2.76.

### French1998
- Reference: French, B. M. (1998). Traces of Catastrophe: A Handbook of Shock-Metamorphic Effects in Terrestrial Meteorite Impact Structures. Lunar and Planetary Institute Contribution 954. https://www.lpi.usra.edu/publications/books/CB-954/CB-954.pdf
- Ledger ids: RCK-090
- Library: `French1998/French1998.pdf` and `.txt`.
- Read: primary full text of the sections on impact breccias, suevite and melt rocks, and the introduction.

### StoefflerGrieve2007
- Reference: Stoeffler, D. and Grieve, R. A. F. (2007). Impactites. In: Fettes, D. and Desmons, J. (eds), Metamorphic Rocks: A Classification and Glossary of Terms. Recommendations of the IUGS Subcommission on the Systematics of Metamorphic Rocks. Cambridge University Press, chapter 2.11.
- Ledger ids: RCK-090
- Library: `StoefflerGrieve2007/` (author-circulated copy, PDF and text).
- Read: primary full text of the classification and glossary entries (suevite, lithic breccia, impact melt rock).

### Hardie1967
- Reference: Hardie, L. A. (1967). The gypsum-anhydrite equilibrium at one atmosphere pressure. American Mineralogist 52:171-200. https://minsocam.org/ammin/AM52/AM52_171.pdf
- Ledger ids: RCK-045
- Library: `Hardie1967/Hardie1967.pdf` and `.txt` (noisy OCR text layer; numbers re-checked against the abstract).
- Read: abstract and discussion. 58 +/- 2 degC for pure water; 38-42 degC from solubility measurements; the transition drops to 18 degC in seawater saturated with halite and gypsum.

### Orti2012
- Reference: Orti, F., Rosell, L., Bet Playa, E. and Salvany, J. M. (2012). Meganodular anhydritization: a new mechanism of gypsum to anhydrite conversion (Palaeogene-Neogene, Ebro Basin, North-east Spain). Sedimentology 59:1257-1277. doi 10.1111/j.1365-3091.2011.01305.x
- Ledger ids: RCK-045
- Library: `Orti2012/Orti2012.pdf` and `.txt` (repository copy).
- Read: primary full text of the discussion of the temperature and depth of the conversion (about 50 degC at 450-500 m with clastic or carbonate accompanying rock; about 20 degC with halite).

### Davies2010
- Reference: Davies, J. H. and Davies, D. R. (2010). Earth's surface heat flux. Solid Earth 1:5-24. CC BY 3.0.
- Ledger ids: RCK-030
- Library: `Davies2010/Davies2010.pdf` and `.txt`.
- Read: primary full text. Table 7: continent 70.9 mW/m2, ocean 105.4, global 91.6; continental and island-arc margins 73.8 mW/m2 (standard deviation 91.1); 65 mW/m2 from Pollack et al. 1993 as restated.

### Meinert1992
- Reference: Meinert, L. D. (1992). Skarns and skarn deposits. Geoscience Canada 19(4):145-162.
- Ledger ids: RCK-054
- Library: `Meinert1992/Meinert1992.pdf` and `.txt` (a scan, read by OCR).
- Read: primary full text (definitions of skarn, calc-silicate hornfels, reaction skarn).

### NelsonTulane
- Reference: Nelson, S. A., Tulane University, lecture notes for EENS 2120 Petrology: "Types of metamorphism", "Metamorphic rock classification and facies", "Contact metamorphism", "Regional metamorphism", "Metamorphic reactions". https://www2.tulane.edu/~sanelson/eens212/
- Ledger ids: RCK-039 to RCK-049, RCK-054, STR-044
- Library: `Nelson_Tulane_EENS212/` (five pages, PDF and text).
- Read: secondary (lecture notes): diagenesis below 200 degC and 300 MPa; facies series at 30 and at 10 degC/km; protolith-rock pairs; Franciscan, Dalradian and southern Appalachian case-study conditions.

### HackerUCSB
- Reference: Hacker, B. R., University of California Santa Barbara, GEO 102C Metamorphic Petrology, lecture notes parts 1, 2, 3, 5, 6, 13 and 14.
- Ledger ids: RCK-028, RCK-030, RCK-032, STR-006
- Library: `Hacker_UCSB_geo102C/` (HTML and text).
- Read: secondary (lecture notes): Fourier's law worked example (3 W/m/K x 20 K/km = 60 mW/m2); gradients of 5-10 K/km in cold subduction zones, 20 K/km in cratons, 50 K/km in rifts and arcs; crust never thicker than about 75 km; the 200-m ophiolite sole.

### Robertson1988
- Reference: Robertson, E. C. (1988). Thermal properties of rocks. U.S. Geological Survey Open-File Report 88-441.
- Ledger ids: RCK-029
- Library: `Robertson1988/Robertson1988.pdf` and `.txt`.
- Read: primary full text, table 2: gneiss 2.1-3.0 (across foliation) and 2.9-4.5 (along), marble 2.8-2.9, amphibolite 2.9 W/m/K at 300 K and 5 MPa.

### Baxter2002
- Reference: Baxter, E. F., Ague, J. J. and DePaolo, D. J. (2002). Prograde temperature-time evolution in the Barrovian type-locality constrained by Sm/Nd garnet ages from Glen Clova, Scotland. Journal of the Geological Society 159:71-82.
- Ledger ids: RCK-042, RCK-044
- Library: `Baxter2002/Baxter2002.pdf` and `.txt` (author-institution copy).
- Read: primary full text. Garnet zone c. 500-550 degC, kyanite c. 550-600 degC, sillimanite zone c. 650-700 degC at c. 6 kbar.

### Stipp2002
- Reference: Stipp, M., Stunitz, H., Heilbronner, R. and Schmid, S. M. (2002). Dynamic recrystallization of quartz: correlation between natural and experimental conditions. Geological Society, London, Special Publications 200:171-190.
- Ledger ids: RCK-040
- Library: `Stipp2002/Stipp2002.pdf` and `.txt` (OCR; noisy).
- Read: primary full text of the abstract and summary: brittle-to-plastic transition near 280 degC; bulging recrystallisation 280-400 degC, subgrain rotation 400-500 degC, grain-boundary migration 500-700 degC (strain-rate dependent).

### Ferrill2004
- Reference: Ferrill, D. A., Morris, A. P., Evans, M. A., Burkhard, M., Groshong, R. H. and Onasch, C. M. (2004). Calcite twin morphology: a low-temperature deformation geothermometer. Journal of Structural Geology 26:1521-1529.
- Ledger ids: RCK-040
- Library: `Ferrill2004/Ferrill2004.pdf` and `.txt` (repository copy).
- Read: primary full text. Calcite twinning dominates below about 400 degC; dynamic recrystallisation important above 250 degC.

### Annen2017
- Reference: Annen, C. (2017). Factors affecting the thickness of thermal aureoles. Frontiers in Earth Science 5:82. doi 10.3389/feart.2017.00082
- Ledger ids: RCK-053, RCK-091
- Library: `Annen2017/Annen2017.pdf` and `.txt`.
- Read: primary full text of the abstract and introduction (aureole thickness proportional to intrusion thickness in the idealised case; Manaslu about 100 m round a several-km sheet; Western Adamello 1.7 km round a 5.5-km intrusion).
- Read in REV-2 (2026-10-19, a Sonnet lookup, full text online): aureole thickness 'is a linear function of the intrusion thickness' for a single pulse; its table of aureoles: Alta stock 2-3 km, tremolite isograd about 1 km out and (citing Cook and Bowman 1994) talc about 1.5 km, periclase about 200 m; Western Adamello 5.5 km, andalusite 1.7 km; Cupsuctic 6-8 km, biotite a few hundred m to 1.5 km; Skaergaard 4 km radius, floor aureole 390 m; Mooselookmeguntic 2 km, garnet beyond 1 km; Manaslu 5-12 km, under 100 m. Incremental emplacement narrows the aureole up to about fourfold; diffusivity contrasts halve or double it. No comparison of carbonate with pelite hosts.

### Mottram2014
- Reference: Mottram, C. M. et al. (2014). Developing an inverted Barrovian sequence; insights from monazite petrochronology. Earth and Planetary Science Letters 403:418-431. CC BY.
- Ledger ids: STR-006, STR-044
- Library: `Mottram2014/Mottram2014.pdf` and `.txt`.
- Read: primary full text of the introduction (inverted metamorphic sequences in ophiolite soles, the Caledonides, Appalachians, Variscan belt, Canadian Cordillera and Himalaya).

### Sass1994
- Reference: Sass, J. H., Lachenbruch, A. H., Galanis, S. P., Morgan, P., Priest, S. S., Moses, T. H. and Munroe, R. J. (1994). Thermal regime of the southern Basin and Range Province: 1. Heat flow data from Arizona and the Mojave Desert of California and Nevada. Journal of Geophysical Research 99(B11):22093-22119.
- Ledger ids: RCK-030
- Library: `Sass1994_SouthBasinRange/Sass1994.pdf` and `.txt` (state geological survey copy).
- Read: primary full text of the abstract (heat flow from 5 mW/m2 on the Colorado Plateau to more than 150 mW/m2 in the Salton Trough rocks; 67 +/- 4 mW/m2 for the metamorphic core complexes).

### Bateman1992
- Reference: Bateman, P. C. (1992). Plutonism in the central part of the Sierra Nevada batholith, California. U.S. Geological Survey Professional Paper 1483. https://pubs.usgs.gov/pp/1483/report.pdf
- Ledger ids: RCK-019, RCK-060, RCK-062, RCK-063, RCK-064
- Library: `Bateman1992/Bateman1992_PP1483.pdf` and `.txt`.
- Read: primary, abstract and introduction in full and the passages on pluton size, shape and depth: plutons from under 1 to over 1,000 km2; batholith at least 100 km2; Mount Givens 80 x 17 km (about 1,400 km2); Lamarck over 60 x 10 km; tonalite most common in the west, hornblende-biotite granodiorite along the axis; 3.5-7 km of rock eroded since emplacement.

### Ducea2001
- Reference: Ducea, M. (2001). The California arc: thick granitic batholiths, eclogitic residues, lithospheric-scale thrusting, and magmatic flare-ups. GSA Today 11(11):4-10. https://www.geosociety.org/gsatoday/archive/11/11/pdf/i1052-5173-11-11-4.pdf
- Ledger ids: RCK-056, RCK-064
- Library: `Ducea2001/Ducea2001.pdf` and `.txt`.
- Read: primary full text. The arc was active 220-80 Ma; about 600 plutons for almost 65 % of the exposed arc; baseline addition about 10 km3 per km per Myr, flare-up as high as 85; exposed plutons are tonalite and granodiorite with under 3 % mafic rock.

### Glazner2004
- Reference: Glazner, A. F., Bartley, J. M., Coleman, D. S., Gray, W. and Taylor, R. Z. (2004). Are plutons assembled over millions of years by amalgamation from small magma chambers? GSA Today 14(4):4-11.
- Ledger ids: RCK-068
- Library: `Glazner2004/Glazner2004.pdf` and `.txt`.
- Read: primary full text (incremental growth of large plutons; Half Dome Granodiorite emplaced over more than 3 Myr, 92.8-88.8 Ma; Tuolumne 95-85 Ma).

### Greene2006
- Reference: Greene, A. R., DeBari, S. M., Kelemen, P. B., Blusztajn, J. and Clift, P. D. (2006). A detailed geochemical study of island arc crust: the Talkeetna Arc section, south-central Alaska. Journal of Petrology 47(6):1051-1093.
- Ledger ids: RCK-065
- Library: `Greene2006/Greene2006.pdf` and `.txt`.
- Read: primary abstract (mid-crustal plutonic rocks are gabbroic, dioritic to tonalitic over 30-40 % of the area, with mafic dikes).

### DietlKoyi2008
- Reference: Dietl, C. and Koyi, H. (2008). Formation of tabular plutons: results and implications of centrifuge modelling. Journal of Geosciences 53:253-261. doi 10.3190/jgeosci.032
- Ledger ids: RCK-061
- Library: `DietlKoyi2008/DietlKoyi2008.pdf` and `.txt`.
- Read: primary full text of the introduction, which restates McCaffrey and Petford 1997 (laccoliths T = 0.12 L^0.88, 135 intrusions; plutons T = 0.29 L^0.80, 21 intrusions) and Petford and Clemens 2000 (exponent 0.6 +/- 0.1 for more than 100 plutons).

### VigneresseBurg2003
- Reference: Vigneresse, J.-L. and Burg, J.-P. (2003). The paradoxical aspect of the Himalayan granites. Journal of the Virtual Explorer 11, paper 3.
- Ledger ids: RCK-056, RCK-058
- Library: `VigneresseBurg2003/VigneresseBurg2003.html` and `.txt`.
- Read: primary full text. Magmatic production rates: island arcs 20-40 km3 per km per Myr; Peruvian and Canadian Coastal about 200; High Himalayan leucogranites under 10; sheets 1.5-2 km thick.

### Tuolumne_Frontiers2020
- Reference: Ardill, K. E., Paterson, S. R., Stanback, J., Alasino, P. H. et al. (2020). Schlieren-bound magmatic structures record crystal flow-sorting in dynamic upper-crustal magma-mush chambers. Frontiers in Earth Science 8:190. doi 10.3389/feart.2020.00190
- Ledger ids: RCK-060, RCK-063
- Library: `Tuolumne_Frontiers2020/f.html` and `f.txt`.
- Read: primary full text of the geological setting: the Tuolumne Intrusive Complex is about 1,100 km2, emplaced over about 10 Myr (95-85 Ma) at 6-10 km depth.

### Cruden2008_SKB
- Reference: Cruden, A. R. (2008). Emplacement mechanisms and structural influences of a younger granite intrusion into older wall rocks: a principal study with application to the Gotemar and Uthammar granites. Swedish Nuclear Fuel and Waste Management Co., report R-08-138. https://www.osti.gov/etdeweb/servlets/purl/951485
- Ledger ids: RCK-057, RCK-061, STR-007
- Library: `Cruden2008_SKB/Cruden2008_SKB_R-08-138.pdf` and `.txt`.
- Read: primary full text of section 2.1.7, "Empirical power law": the author restating McCaffrey and Petford 1997 and Cruden and McCaffrey 2001. T = b L^a; 66 plutons; L the equivalent diameter of a circle; b = 0.6 (+/-0.15), a = 0.6 (+/-0.1); data points Gasborn (L 4.5 km, T 1.6 km), Lucerne (56 km, 4.5 km), Sierra Nevada (600 km, 15 km).

### ChristensenMooney1995
- Reference: Christensen, N. I. and Mooney, W. D. (1995). Seismic velocity structure and composition of the continental crust: a global view. Journal of Geophysical Research 100(B6):9761-9788.
- Ledger ids: RCK-033, PLT-256 (orogenic collapse, 2026-10-03: orogens 46.3 km, global 41.1 km, as comparison), PLT-281 (2026-10-06: the mean crustal density, 2830 kg/m3)
- Library: `ChristensenMooney1995/abstract.json` (the USGS publications record).
- Read: abstract only. Average crustal thickness 41.1 km (extended 30.5 km, orogens 46.3 km); "The mean crustal density calculated from our model is 2830 kg/m3".
- Read again in the high-ground task (2026-10-22, PLT-412 and PLT-413): the abstract, from OpenAlex's record (doi 10.1029/95JB00259): "extended crust showing an average thickness of 30.5 km and orogens an average of 46.3 km. Shields and platforms have an average crustal thickness nearly equal to the global average"; "The weighted average crustal thickness based on these values is 41.1 km." Ledger ids added: PLT-412, PLT-413.

### BrownJohnson2018_dataset
- Reference: Brown, M. and Johnson, T. (2018). Global age, temperature and pressure data for secular change in metamorphism. EarthChem Library, doi 10.1594/IEDA/111316; slides of M. Brown's presidential address to the Mineralogical Society of America.
- Ledger ids: RCK-048
- Library: `Brown_UMD_MMSA/` (dataset abstract and slides).
- Read: dataset abstract. Three apparent thermal-gradient classes: high T/P above 775 degC per GPa, intermediate 375-775, low below 375 (blueschists and low-temperature eclogites).

### tectonicsjs_Crust
- Reference: Davidson, tectonics.js, noncompiled/models/lithosphere/Crust.js, `model_lithification`. https://github.com/davidson16807/tectonics.js
- Ledger ids: RCK-022
- Library: no saved copy (read through the GitHub API on 2026-10-02).
- Read: the source comment "2.2e6 Pascals is the pressure equivalent of 500ft of sediment @ 1500kg/m^3 density; 500ft from http://wiki.aapg.org/Sandstone_diagenetic_processes", and "convert sedimentary ... where it exceeds pressure/temperature equivalent of 300MPa/11km" for metamorphosis. The AAPG wiki page, fetched through a tool, does not state a 500-ft figure.

### Read at second hand in WP-7 (no saved copy; secondary)
- Sclater, J. G. and Christie, P. A. F. (1980), JGR 85:3711, porosity-depth parameters as tabulated in the pyBacktrack documentation (https://pybacktrack.readthedocs.io); Bjorlykke et al. (2009), AAPG Search and Discovery 50192; Sanuade and Kaka (2017), Geological Quarterly 61(1):106-123, on the Niger Delta; Hudec and Jackson (2007) as quoted by the EGU TS blog; Schultz-Ela, Jackson and Vendeville (1994) abstract; Ponce, Denton and Miller (2013), USGS, abstract (Mountain Pass carbonatite density); Christeson et al. (2018), EPSL, abstract (Chicxulub suevite and melt rock density); Devriese, Davis and Oldenburg (2017), Interpretation (kimberlite density, qualitative); RocFall documentation table of the New Zealand PETLAB rock-property database (Tenzer et al. 2011); Wikipedia pages (Carbonate compensation depth, Salt tectonics, Cathedral Peak Granodiorite, Migmatite, Gneiss, Blueschist, Granulite, Petroleum trap, Diamictite, Greisen, Skarn, Travertine); SEPM Strata and LibreTexts pages on the CCD; Greenwood and Berman, "Metamorphic rock", The Canadian Encyclopedia.
- Closed and not read: Winter 2010, Yardley 1989, Spear 1993, Bucher and Grapes 2011, Warren 2016, Telford et al. 1990, Schon 2015, Cruden 1998, McCaffrey and Petford 1997, Cruden and McCaffrey 2001, Buddington 1959, Haederle and Atherton 2002, Mukasa 1986, Hudec and Jackson 2007, Doust and Omatsola 1990, Pollack et al. 1993, Heimsath et al. 1997, Roering et al. 1999, Kelsey and Hand 2015, Brown 2014.

### Ducea2001
- Reference: Ducea, M. (2001). The California arc: thick granitic batholiths, eclogitic residues, lithospheric-scale thrusting, and magmatic flare-ups. GSA Today 11(11):4-10.
- Cited as: "Ducea 2001, GSA Today" (plate tasks 3 and 4, ledger PLT-242)
- Ledger ids: PLT-242
- Access: open (GSA Today)
- Library (the project's source library, not distributed): Ducea2001.pdf, Ducea2001.txt.
- Read in plate tasks 3 and 4 (2026-10-02): the passage on crustal thickness, p. 5: "The present crust beneath the Sierra Nevada is thin (33 km thick ...) and is probably an effect of Late Cenozoic extension and delamination"; "the Mesozoic crust was at least ~70 km thick". Used only as evidence that thickened crust can lose its root, not for a number in the code.

## Arc crust and continental crust (PLT-045, PLT-047), added 2026-10-02 with the continental crust fix

### Stern2010
- Reference: Stern, R. J. (2010). The anatomy and ontogeny of modern intra-oceanic arc systems. In Kusky, T. M., Zhai, M.-G. and Xiao, W. (eds), The Evolving Continents: Understanding Processes of Continental Growth. Geological Society, London, Special Publications 338:7-34.
- DOI or URL: 10.1144/SP338.2; author-hosted copy https://earthjay.com/earthquakes/20160729_mariana/stern_2010_anatomy_ontogeny_intraoceanic_arcs.pdf
- Cited as: "Stern 2010".
- Ledger ids: PLT-045, PLT-047
- Access: paywalled at the publisher; a copy is hosted openly.
- Library (the project's source library, not distributed): fetched 2026-10-02 (the hosted copy). Text file: `Stern2010/Stern2010.txt` (pdftotext).
- Read so far (by the project): full text, 2026-10-02. p. 9: "Because crustal thickness typically is reflected in elevation, intra-oceanic arcs are generally mostly below sea level"; Moho "at depths between 25 and 35 km. This is where IOASs are thickest, but is nevertheless significantly thinner than continental crust, which is typically c. 40 km thick. IOAS crust also differs from continental crust in being significantly faster (by 0.5-1.0 km s-1)", i.e. more mafic; the crust "thickens at a rate of several hundred metres per million years"; p. 26: "a crustal thickening rate of c. 500 m Ma-1. When IOAS crust thickens sufficiently to stabilize garnet in the lower crust (greater than c. 30 km thick)...".

### SternScholl2010
- Reference: Stern, R. J. and Scholl, D. W. (2010). Yin and yang of continental crust creation and destruction by plate tectonic processes. International Geology Review 52(1):1-31.
- DOI or URL: 10.1080/00206810903332322; author copy https://personal.utdallas.edu/~rjstern/pdfs/YinyangIGR10.pdf
- Cited as: "Stern & Scholl 2010".
- Ledger ids: PLT-242 and PLT-401 (2026-10-21: the suture lengths and the losses per km of suture), PLT-045 (since 2026-10-17 the value), PLT-046, PLT-285, PLT-286, HNT-125, HNT-126, PLT-331 (2026-10-20: what the intra-oceanic arc rate counts), PLT-258 (2026-10-03: "the bulk of the approximately 40 km-thick crust that must have originally existed is missing" at UHP terranes; continental crust lost at all collision zones "approximately 0.7 km3/year", Scholl & von Huene 2009 as cited), PLT-370 (2026-10-21: the arcs split by the crust they stand on; 14,500 and 27,500 km); PLT-440, PLT-441, PLT-442 and the crust budget's loss rows (2026-10-22, the trench budget: Table 1's ocean-margin and suture losses)
- Access: paywalled at the publisher; author copy open.
- Library (the project's source library, not distributed): fetched 2026-10-02 (author copy). Text file: `SternScholl2010/SternScholl2010.txt`.
- Read so far (by the project): full text, 2026-10-02. Abstract: creation and destruction of continental crust "is either in balance (~3.2 km3/year, or 3.2 AU) or more crust is being destroyed than created"; p. 9: the Aleutian and Izu-Bonin-Mariana arcs, "which have evolved since 45-50 Ma", give "crustal growth estimates of 90-180 km3/km-million years"; growth is front-loaded (several hundred km3/km/Myr in the first few Myr, "a few tens or less" for mature arcs); about 2.7 AU of growth at convergent margins (Table 1). Read again 2026-10-17 (plate task 8; the copy at `SternScholl2010/SternScholl2010.txt`): p. 11, intra-oceanic arcs "approximately 100 km3/km-million years (500 km3/million years for first 10 million years plus 30 km3/million years for 60 million years thereafter). At this rate, crustal growth at the present 14,500 km of intra-oceanic arcs" gives 1.5 AU; p. 12, continental arcs after Haschke & Günther 2003: "The 35 km3/km-million years rate is essentially the rate of Reymer and Schubert (1984). It implies a reasonable intrusive : extrusive ratio of approximately 7:1 and is adopted as a useful long-term rate estimate for continental margin arc productivity. The combined length of continental margin arcs is approximately 27,500 km (Scholl and von Huene 2007). The corresponding global addition rate is thus approximately 1.0 AU"; Table 1 (p. 12): oceanic arcs 1.5, continental arcs 1.0, collision zones 0.2, rifts and hotspots 0.5 AU of additions; losses at sutures about 0.7 AU; p. 24, new arcs 300 to 500 km3/km/Myr, mature ones 30 to 35. Read again 2026-10-20 (the arc-basement task, PLT-331; the copy at `SternScholl2010/SternScholl2010.txt`): p. 7, of the geophysical arc growth rates, "The largest sources of error are as follows: (1) uncertainty about whether to include an original 6–7 km thickness of ‘trapped’ oceanic crust"; "early concepts assumed that new arcs grew on a substrate of pre-existing oceanic crust that was trapped when the new subduction zone formed (Dickinson and Sealy 1979). More recent considerations indicate that the substrate for intra-oceanic arcs is produced when subduction begins (Stern and Bloomer 1992; Hall et al. 2003)"; Figure 6 caption, after Tatsumi et al. (2008): "Incipient arc magmatism replaces the pre-existing oceanic crust to create the initial mafic arc crust"; Introduction: "thinner (∼7 km), denser (∼2.90 g/cc) oceanic crust".
- Read in the plate-geometry task (2026-10-21), the library text again: p. 11-12, intra-oceanic arc growth at "the present 14,500 km of intra-oceanic arcs alone (principally the Aleutian, South Sandwich, Kermadec, Tonga, Vanuatu, Solomon, New Britain, Izu–Bonin–Mariana, Indonesia east of Java or Banda, and the Lesser Antilles arcs)" gives 1.5 AU; "The combined length of continental margin arcs is approximately 27,500 km (Scholl and von Huene 2007)", 1.0 AU; so 14,500 of 42,000 km (35 %) of the arcs stand on ocean floor. PLT-370.
- Read in the collision-flux task (2026-10-21, the library text, p. 18-19; PLT-242, PLT-401): "Scholl and von Huene (2009) infer that crust has been removed at an estimated average global length of collisional margins of ~6000 km. This length is based on the estimate of Hildebrand and Bowring (1999) that over geologic time the length of global active sutures has ranged from 1000 to 20,000 km. Presently, the global length of major ongoing suturing is ~12,000 km (today mostly associated with closing of the Tethys and Australia against Indonesia)"; "Building the new 'Pangea' will thus entail 25,000 km of crustal suturing. In contrast, there is likely negligible suturing immediately after breakup of a supercontinent"; "We thus accept the long-term average of 6000 km-long for continental suturing"; "Crustal recycling at suture zones is estimated at ~115 km3/km-million years. This includes ~15 km3 due to sediment subduction, 50 km3 of tectonically eroded upper plate, and 50 km3 for the loss of deeply subducted lower plate crust, resulting in an additional ~0.7 AU of crustal losses at suture zones"; India, "As a minimum estimate, let us further assume that 10% of this has been lost by deep subduction since collision began approximately 50 million years ago", 0.24 km3/year; "Such estimates do not inspire confidence that we really understand this flux". Hildebrand and Bowring 1999 is cited through this paper, not read.
- Read in the trench-budget task (2026-10-22, the author copy, by a research sub-agent; PLT-440, PLT-441, PLT-442): Table 1, "A balance sheet for modern continental crust" (AU = km3/yr), every cell: additions, intra-oceanic arcs 1.5, continental arc 1.0, total at ocean margin subduction zones 2.5 (Clift et al. 2009: 2.98), at crust-suturing subduction zones 0.2, at rifts, hotspots and volcanic rifted margins 0.5, total 3.2; losses at ocean margins, sediment subduction -1.1 (Clift et al. 2009: 1.65, all convergent margins), subduction erosion -1.4 (1.33), total -2.5; at sutures, sediment subduction -0.1, subduction erosion -0.3, continental crust -0.3 (0.4, subduction of passive margins); foundering unknown; total >3.2 (3.38). p. 15: "Sediment subduction averages approximately 15 km3/km-million years of trench at accreting (i.e. widening seaward) sediment-dominated margins. The average sediment subduction rate is estimated as twice this (approximately 30 km3/million years-km) at sediment-starved or landward-narrowing margins"; "large accretionary bodies at sediment-dominated margins sequester approximately 70% of all sediments entering subduction zones"; "accretionary margins are generally located where the orthogonal underthrusting rate is 35-45 km/million years or lower. At non-accreting margins, 100% of incoming sediments are consumed and the typical underthrusting rate is approximately 80 km/million years". p. 16: "~60 km3/km-million years of continental crust is recycled into the mantle by subduction, ~25 km3 subducted sediment and ~35 km3 tectonically eroded from the forearc. The corresponding global loss for the present ~43,000 km-length of convergent margins is thus ~2.5 AU". Fig. 3 prints the two ocean-margin losses as ~1.0 and ~-1.5 (the 25 and 35 per km times 43,000 km give 1.08 and 1.51); p. 19's "2.9 km3/y due to sediment subduction and subduction erosion combined" is Table 1's 1.1 + 1.4 + 0.1 + 0.3.

### TetreaultBuiter2014
- Reference: Tetreault, J. L. and Buiter, S. J. H. (2014). Future accreted terranes: a compilation of island arcs, oceanic plateaus, submarine ridges, seamounts, and continental fragments. Solid Earth 5:1243-1275.
- DOI or URL: 10.5194/se-5-1243-2014 (open access, CC BY 3.0).
- Cited as: "Tetreault & Buiter 2014".
- Ledger ids: PLT-047 (secondary statement of Cloos 1993's 15 km)
- Access: open.
- Library (the project's source library, not distributed): fetched 2026-10-02 (publisher PDF). Text file: `TetreaultBuiter2014/TetreaultBuiter2014.txt`.
- Read so far (by the project): full text, 2026-10-02. Abstract: "Island arcs have an average crustal thickness of 26 km, average bulk crustal density of 2.79 g cm-3"; section 3.1: "Cloos (1993) estimated that island arcs greater than 15 km in thickness are buoyant enough to collide with continental crust; however, the paucity of whole crustal sections of island arcs in the geologic record does not agree with this hypothesis (Condie and Kroner, 2013)."

### JichaJagoutz2015
- Reference: Jicha, B. R. and Jagoutz, O. (2015). Magma production rates for intraoceanic arcs. Elements 11(2):105-111.
- DOI or URL: 10.2113/gselements.11.2.105
- Cited as: not cited in the code; named in the task for the continental crust fix as a source for arc crust growth.
- Ledger ids: PLT-045 (comparison, no value taken)
- Access: paywalled (OpenAlex lists no open copy, 2026-10-02).
- Library (the project's source library, not distributed): abstract only, 2026-10-02 (OpenAlex record of the GSW page): `JichaJagoutz2015/openalex.json`, text `JichaJagoutz2015/abstract.txt`.
- Read so far (by the project): abstract, 2026-10-02. It gives no numbers: "Our revised magma production rates for active intraoceanic arcs are consistent with those calculated for mid-ocean ridge segments that have slow to moderate spreading rates. This is surprising because magma production at intraoceanic arcs has traditionally been assumed to be significantly less than that at mid-ocean ridges." No value in the code or the ledger rests on it; the rates PLT-045 is compared with are Stern 2010's and Stern & Scholl 2010's.

## Continental margins (ERO-168 to ERO-170, HNT-093), added 2026-10-03

### Pratson1996
- Reference: Pratson, L. F. and Haxby, W. F. (1996). What is the slope of the U.S. continental slope? Geology 24(1):3-6.
- DOI or URL: 10.1130/0091-7613(1996)024<0003:WITSOT>2.3.CO;2
- Cited as: "Pratson & Haxby 1996" (the regional slope of the steepest clastic margin, 2.5°).
- Ledger ids: ERO-168, HNT-093, YNG-022 (sea-floor spikes task, 2026-10-20: the steepest mean local slope of a margin, 12.0°, against the sea floor's critical slope)
- Access: paywalled (GSA); the abstract is on Duke Scholars (https://fds.duke.edu/db/pratt/gendell/faculty/lincoln.pratson/publications/279477).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract only, 2026-10-03 (margins and spires task): regional and mean local slopes, West Florida (passive carbonate) 4.4° and 12.0°, New Jersey–Maryland (passive clastic) 2.5° and 7.6°, Louisiana (salt) 0.5° and 2.9°, California (strike-slip) 1.8° and 5.2°, Oregon (convergent) 2.0° and 5.2°, from multibeam bathymetry of five U.S. margins.
- Reference check: matches (title, journal, volume, pages on the Duke page).

ETOPO 2022 (entry above, under the climate section) was used as data in the same task: the 60 arc-second surface GeoTIFF was downloaded to the test VM and the continental margins measured with the planet's transect code (build log 2026-10-03).

## Per-cell climate as built: sources added with P1-26 T6 (2026-10-03)

The sources of the CO₂ forcing and the snowball cloud term (research note `docs/research/notes/high-co2-forcing-and-snowball-escape.md`, section 7, which says how each was read). The library folders are named as in the note.

### HEXTOR
- Reference: Haqq-Misra, J., & Hayworth, B. P. C. (2022). An Energy Balance Model for Rapidly and Synchronously Rotating Terrestrial Planets. The Planetary Science Journal (volume and article number not recorded in the note; arXiv 2201.02685). The software and its tables: HEXTOR, Blue Marble Space, https://github.com/BlueMarbleSpace/hextor, MIT licence, Copyright (c) 2020 Blue Marble Space, commit f800e25f.
- DOI or URL: arXiv 2201.02685 (the paper); https://github.com/BlueMarbleSpace/hextor (the table `model/radiation/radiation_N2_CO2_Sun.h5`)
- Cited as: "Haqq-Misra & Hayworth 2022, PSJ (HEXTOR, MIT; table radiation_N2_CO2_Sun.h5, commit f800e25f)"
- Ledger ids: CLM-167 (the table), CLM-168 (its scale)
- Access: open (arXiv; GitHub, MIT).
- Library (the project's source library, not distributed): fetched; folders `HaqqMisraHayworth2022` (arXiv text, `HaqqMisraHayworth2022.txt`) and `HEXTOR_software` (a sparse clone of `model/`; the table exported to `olr_sun_1bar_N2.csv` with the analysis scripts under `analysis/`).
- Read in the research note (2026-10-03): the paper's radiative-transfer section 2, Appendix A and the software statement; `radiation.f90`'s header and v1 reader; `driver.f`'s OLR parameterizations; the Sun table read with h5py (1,748 rows: 19 temperatures, 190 to 370 K, by 92 CO₂ rows (91 distinct), 1.0e-4 to 10 bar; "1,748 values of FIR … radiative-convective climate model of Kopparapu et al. (2013)", "background N2 pressure of 1 bar"). The table ships in the repository as `crates/deeptime-core/src/climate/co2_table.rs` (1,729 values, the duplicated column dropped) under the MIT licence, credited in NOTICE.md and the file header.
- Not read: the Haqq-Misra et al. 2016 fit's online supplement (its transcription in HEXTOR's `driver.f` was tested and rejected: 347 W/m² at 288 K and 280 ppm where the table gives 260).

### Kopparapu2013
- Reference: Kopparapu, R. K., Ramirez, R., Kasting, J. F., Eymet, V., Robinson, T. D., Mahadevan, S., Terrien, R. C., Domagal-Goldman, S., Meadows, V., & Deshpande, R. (2013). Habitable Zones Around Main-Sequence Stars: New Estimates. The Astrophysical Journal 765(2): 131.
- DOI or URL: 10.1088/0004-637X/765/2/131; arXiv 1301.6674
- Cited as: "Kopparapu et al. 2013, ApJ 765: 131"
- Ledger ids: CLM-167
- Access: open (arXiv).
- Library (the project's source library, not distributed): fetched (arXiv copy). Text file: Kopparapu2013.txt.
- Read in the research note (2026-10-03): section 2 (the model: cloud-free radiative-convective, moist adiabat, updated CO₂ collision-induced absorption) and its caveats on dense CO₂ (pp. 5 to 8), in particular that the method "tends to overestimate the greenhouse effect of dense CO2 atmospheres (Goldblatt & Zahnle 2011)" while neglecting CO₂ clouds "may cause us to underestimate" it. Goldblatt & Zahnle 2011 was not obtained.

### Abbot2012
- Reference: Abbot, D. S., et al. (2012). Clouds and Snowball Earth deglaciation. Geophysical Research Letters 39: L20711. (The note names only the first author, the title and the reference; the full author list was not recorded.)
- DOI or URL: not recorded in the note
- Cited as: "Abbot et al. 2012, GRL 39: L20711"
- Ledger ids: CLM-172
- Library (the project's source library, not distributed): fetched, the published version from the Max Planck institutional repository (MPG.PuRe). Text file: Abbot2012.txt.
- Read in the research note (2026-10-03): in full (4 pages). Six GCMs at surface albedo 0.6 and S = 1285 W/m²: the cloud radiative forcing "varies widely among the models"; FOAM's condensate "an order of magnitude lower"; in the cloudy models it is "near zero in the winter hemisphere … but rises to ≈40 W m⁻² in the summer hemisphere"; clouds "could warm a Snowball enough to reduce the CO₂ required for deglaciation by a factor of 10–100" relative to FOAM.

### Hu2011
- Reference: Hu, Y., Yang, J., Ding, F., & Peltier, W. R. (2011). Model-dependence of the CO₂ threshold for melting the hard Snowball Earth. Climate of the Past 7: 17-25.
- DOI or URL: not recorded in the note (open access, Copernicus)
- Cited as: "Hu et al. 2011, Clim. Past 7: 17"
- Ledger ids: CLM-172, CLM-166 (the literature's escape thresholds)
- Library (the project's source library, not distributed): fetched (publisher, CC BY). Text file: Hu2011.txt.
- Read in the research note (2026-10-03): in full. Escape CO₂ about 0.21 bar in a radiative-convective model (0.94 S₀, surface albedo 0.663), 0.38 bar in CAM3 with pressure broadening and collision-induced absorption, 1.0 bar without; January zonal-mean longwave cloud forcing maxima FOAM 6 to 10.5 W/m², CAM3 26 to 32, LMDz 50 (pp. 19 to 20). Le Hir et al. 2007 and Abbot & Pierrehumbert 2010 are known only as cited here.

### Pierrehumbert2011
- Reference: Pierrehumbert, R. T., Abbot, D. S., Voigt, A., & Koll, D. (2011). Climate of the Neoproterozoic. Annual Review of Earth and Planetary Sciences 39: 417-460.
- DOI or URL: not recorded in the note (author institution's copy, University of Chicago)
- Cited as: "Pierrehumbert et al. 2011, Annu. Rev. Earth Planet. Sci. 39: 417" (the note writes "Pierrehumbert et al. 2011")
- Ledger ids: CLM-167 (the ppm to pCO₂ mapping, Table 1), CLM-172 (c_ld 10 to 30 W/m², Table 2), CLM-166
- Library (the project's source library, not distributed): fetched (author institution's copy). Text file: Pierrehumbert2011.txt.
- Read in the research note (2026-10-03): sections 3.1 to 3.3, Table 1 (inventory, partial pressure and molar concentration; the row "10⁵ Pa gives 39.7 % molar, pCO₂ = 7.95·10⁴ Pa and p_tot = 2.00·10⁵ Pa" checked), Table 2 (the 1-D EBM's escape inventories for cloud terms c_ld 10, 20 and 30 W/m²) and section 6. Its OLR fit (a Python supplement on the Annual Reviews site) was not fetched.


## Orogenic collapse (PLT-254 to PLT-259), added 2026-10-03

Read by a research sub-agent of the collapse task (2026-10-03) through publisher pages, Crossref and
USGS publication records; no copy was saved to the library. Bird 1991 and Christensen & Mooney 1995
(entries above) were read again for the same rows.

### HillenbrandWilliams2022
- Reference: Hillenbrand, I. W. and Williams, M. L. (2022). Geochemical evidence for diachronous uplift and synchronous collapse of the high elevation Variscan hinterland. Geophysical Research Letters 49(21):e2022GL100435.
- DOI or URL: 10.1029/2022GL100435 (USGS record https://pubs.usgs.gov/publication/70238122).
- Ledger ids: PLT-255, PLT-256, PLT-390 (2026-10-21: the hot window)
- Access: open record of the abstract; full text not read.
- Library: not fetched.
- Read so far (by the project): abstract, 2026-10-03: "thick crust (55–70 km) capable of supporting 3–5 km elevations developed diachronously from east to west between ca. 350 and 315 Ma. Crustal thinning occurred from ca. 315 to 290 Ma across the orogen."

### Dewey1988
- Reference: Dewey, J. F. (1988). Extensional collapse of orogens. Tectonics 7(6):1123-1139.
- DOI or URL: 10.1029/TC007i006p01123
- Ledger ids: PLT-254
- Access: paywalled; abstract read.
- Library: not fetched.
- Read so far (by the project): abstract, 2026-10-03: collapse explains "the return of orogenic large crustal thicknesses to normal without very much erosional denudation"; "Even where overall convergence is continuing, orogenic collapse may be enhanced by subduction rollback".

### Carbonell2000
- Reference: Carbonell, R. and others (2000). [Wide-angle seismic study of the Urals crustal root; title not recorded by the sub-agent.] Journal of Geophysical Research 105(B6):13755-13777. Co-authors and title to be taken from the DOI before quoting.
- DOI or URL: 10.1029/2000JB900048
- Ledger ids: PLT-256, PLT-282 (2026-10-06: the Urals' crust, against the eclogite root), PLT-390 (2026-10-21: the cold root that keeps T_c)
- Access: paywalled; abstract read.
- Library: not fetched.
- Read so far (by the project): abstract, 2026-10-03: "a 50–56 km crustal thickness beneath the central part of the orogen in contrast to Moho depths of ≈ 45 km documented at the edges of the transect". The IGCP 559 transect page (earthscrust.org.au, Urals) adds that the root "is consistent with the preservation of the Urals as an intact, unextended orogen".

### ReyTeyssierWhitney2010
- Reference: Rey, P. F., Teyssier, C. and Whitney, D. L. (2010). Limit of channel flow in orogenic plateaux. Lithosphere 2(5):328-332.
- DOI or URL: 10.1130/L114.1 (author copy https://www.earthbyte.org/Resources/Pdf/rey_L114_v1.pdf)
- Ledger ids: PLT-257, PLT-300 (2026-10-20: K₀ = 150² / 15 km²/Myr, the lower-crustal flow `plates/flow.rs`), PLT-391 (2026-10-21: the reason for the order)
- Access: open author copy; full text read.
- Library: not fetched.
- Read so far (by the project): full text, 2026-10-03: "Over 15 m.y., the length scale of channel-flow extrusion is 150 km rather than 1500 km"; a channel "travels at most 150 km into the foreland after 12 m.y. of gravitational collapse"; flow "1 cm yr–1 or less".

### WernickeEtAl1987
- Reference: Wernicke, B. P., Christiansen, R. L., England, P. C. and Sonder, L. J. (1987). Tectonomagmatic evolution of Cenozoic extension in the North American Cordillera. Geological Society, London, Special Publications 28:203-221.
- DOI or URL: 10.1144/GSL.SP.1987.028.01.15
- Ledger ids: PLT-254, PLT-257, PLT-390 (2026-10-21: the timing set by the heat)
- Access: read in full by the sub-agent.
- Library: not fetched.
- Read so far (by the project): full text, 2026-10-03: "We conclude that the locus of extension is controlled principally by crustal thickness, while its timing is governed by the thermal state of the lithosphere at the time of thickening"; the extended area "widened to several hundred kilometres".

### PeronPinvidicOsmundsen2020
- Reference: Peron-Pinvidic, G. and Osmundsen, P. T. (2020). From orogeny to rifting: insights from the Norwegian 'reactivation phase'. Scientific Reports 10:14860.
- DOI or URL: 10.1038/s41598-020-71893-z (open access)
- Ledger ids: PLT-255, PLT-257, PLT-390 (2026-10-21: collapse to rifting in 30-40 Myr)
- Access: open; full text read.
- Library: not fetched.
- Read so far (by the project): full text, 2026-10-03: "Fossen constrained the onset of the orogenic collapse to about 405 Ma"; "the Høybakken detachment zone was mostly active between 400 and 365 Ma"; "cumulative displacements ... on the order of 40–100 km".

### JolivetBrun2010
- Reference: Jolivet, L. and Brun, J.-P. (2010). Cenozoic geodynamic evolution of the Aegean. International Journal of Earth Sciences 99:109-138.
- DOI or URL: 10.1007/s00531-008-0366-4
- Ledger ids: PLT-254, PLT-390 (2026-10-21: where a hot orogen thins to)
- Access: author PDF; full text read.
- Library: not fetched.
- Read so far (by the project): full text, 2026-10-03: "The Cyclades archipelago is underlain by a thinned crust with a very constant thickness around 26 km"; "The extended domain is superposed to a collapsed Early Cenozoic mountain belts".

### MooneyProdehlPavlenkova2002
- Reference: Mooney, W. D., Prodehl, C. and Pavlenkova, N. I. (2002). Seismic velocity structure of the continental lithosphere from controlled source data. In: Lee, W. H. K. and others (eds), International Handbook of Earthquake and Engineering Seismology, Part A. Academic Press. (Pages and DOI not checked.)
- DOI or URL: USGS author copy https://escweb.wr.usgs.gov/share/mooney/54seismicvelocitystruct.pdf
- Ledger ids: PLT-256, PLT-258
- Access: author copy, full text read.
- Library: not fetched.
- Read so far (by the project): full text, 2026-10-03 (research sub-agent): "The thickest crust (70+ km) occurs beneath the Tibetan Plateau and the South American Andes"; "Orogenic crust has a wide range of thickness of 20-80km, and an average thickness of 43 km"; "Isostatic forces uplift crust thicker than about 50 km, leading to erosion and crustal thinning".

### WangEtAl2021Tibet
- Reference: Wang, H. and others (2021). No mafic layer in 80 km thick Tibetan crust. Nature Communications 12:1069.
- DOI or URL: 10.1038/s41467-021-21420-z (open, PMC7886915)
- Ledger ids: PLT-258
- Access: open; full text read.
- Library: not fetched.
- Read so far (by the project): full text, 2026-10-03 (research sub-agent): "Moho at 80 ± 2 km" beneath the Lhasa terrane; felsic thick crust "precludes future delamination due to lack of mafic rocks that may transform into high density eclogitic rocks".

### ZhaoEtAl2020Moho
- Reference: Zhao, G. and others (2020). Moho beneath Tibet based on a joint analysis of gravity and seismic data. Geochemistry, Geophysics, Geosystems 21(2):e2019GC008849.
- DOI or URL: 10.1029/2019GC008849 (GFZ open copy)
- Ledger ids: PLT-258
- Access: open copy; full text read.
- Library: not fetched.
- Read so far (by the project): full text, 2026-10-03 (research sub-agent): Moho "60–80 km within the plateau"; maximum "~79 km is found in Karakorum and in the southern Lhasa block".

### LundsternMiller2021
- Reference: Lundstern, J.-E. and Miller, E. L. (2021). [Chapter on Basin and Range extension.] Geological Society of America Special Paper 555, chapter 13. Title not recorded by the sub-agent; take it from the DOI before quoting.
- DOI or URL: 10.1130/2021.2555(13) (USGS record https://pubs.usgs.gov/publication/70226446)
- Ledger ids: none (cited on plates.md as a limit: thinning can lag thickening by tens of Myr)
- Access: abstract read (USGS record).
- Library: not fetched.
- Read so far (by the project): abstract, 2026-10-03 (research sub-agent): the collapse model is "difficult to reconcile with ... moderate regional extension (≤50%) and the discovery that most high-angle, Basin and Range faults slipped rapidly ca. 17 Ma, tens of millions of years after crustal thickening occurred".

### SundellEtAl2021
- Reference: Sundell, K. E., Laskowski, A. K., Kapp, P. A., Ducea, M. N. and Chapman, J. B. (2021). Jurassic to Neogene quantitative crustal thickness estimates in southern Tibet. GSA Today 31(6).
- DOI or URL: 10.1130/GSATG461A.1 (open: https://www.geosociety.org/gsa-today/june-2021/jurassic-to-neogene-quantitative-crustal-thickness-estimates-in-southern-tibet)
- Ledger ids: PLT-259, PLT-287
- Access: open; full text read.
- Library: not fetched.
- Read so far (by the project): full text, 2026-10-03 (research sub-agent): "crustal thickness increased by ~40 km at ~1.3 mm/a between 60 and 30 Ma to near modern crustal thickness, before the onset of Miocene east-west extension"; "Our results are inconsistent with models that invoke net crustal thinning via orogenic collapse (Dewey, 1988) beginning in the Miocene".
- Read again (plate spikes, 2026-10-18): the full text through the web fetch tool, for the thickening rate (PLT-287): "thickening at ~1.3 mm/a between 60 and 30 Ma (Fig. 3D)"; the Gaussian kernel model "increases from ~40 to ~70 km between 60 and 40 Ma" and "remains steady-state from 40 Ma to present"; "Paleogene crustal thickness estimates indicate monotonic crustal thickening at rates of ~1.3 mm/a to >60 km". Licence CC-BY-NC; no page numbers on the web page.

### BaslerEtAl2026
- Reference: Basler, L. and others (2026). [Paleogene crustal thickness of Nevada.] Tectonics 45. Title and authors to be taken from the DOI before quoting.
- DOI or URL: 10.1029/2025TC009110
- Ledger ids: PLT-259
- Access: abstract read (Crossref).
- Library: not fetched.
- Read so far (by the project): abstract, 2026-10-03 (research sub-agent): "crust at least 50 km thick was present until the Early Miocene, which suggests that thickened crust was maintained for a protracted interval following Cretaceous shortening. These results indicate that most crustal thinning was contemporaneous with a change in plate boundary conditions".
## Quincuncial projections (SCL-070 to SCL-075), added 2026-10-03 with the seamless-wrapping prototype

The prototype crate `crates/deeptime-quincuncial` (not shipped) rests on these. None was read in full this session.

### Peirce1879
- Reference: Peirce, C. S. (1879). A quincuncial projection of the sphere. American Journal of Mathematics 2(4):394-396 (volume and pages from memory).
- Cited as: "Peirce 1879".
- Ledger ids: SCL-070, SCL-074
- Access: JSTOR (paywalled); not fetched.
- Read so far (by the project): through Wikipedia "Peirce quincuncial projection" (docs/research/notes/world-projection-options.md, 2026-10-02).

### Guyou1887
- Reference: Guyou, É. (1887). Nouveau système de projection de la sphère: généralisation de la projection de Mercator. Annales Hydrographiques, série 2, 9:16-35 (from memory, not checked).
- Cited as: "Guyou 1887".
- Ledger ids: SCL-071, SCL-074
- Access: not fetched.
- Read so far (by the project): through Wikipedia "Guyou hemisphere-in-a-square projection", fetched 2026-10-03 by the prototype agent: "an oblique aspect of the Peirce quincuncial projection"; "conformal except for the four corners of each hemisphere's square"; "The Equator is represented by a horizontal line"; "the sphere as a rectangle of aspect ratio 2:1"; developed by Émile Guyou in 1887.

### Lee1976
- Reference: Lee, L. P. (1976). Conformal Projections Based on Elliptic Functions. Cartographica Monograph 16 (supplement to Canadian Cartographer 13).
- Cited as: "Lee 1976".
- Ledger ids: SCL-070, SCL-080 (the conformal cube, seamless-projection survey), SCL-101 (the cube world's conformal table, 2026-10-22)
- Access: not fetched. Read so far: catalogue entry only (via world-projection-options.md). 2026-10-22 (cube world phase 0): still not fetched; the table's map is the survey's Schwarz-triangle construction written from memory (SCL-080), and nothing in it is taken from Lee.

### AbramowitzStegun1964
- Reference: Abramowitz, M. and Stegun, I. A. (eds.) (1964). Handbook of Mathematical Functions. National Bureau of Standards Applied Mathematics Series 55. Chapter 16 (Jacobian elliptic functions, L. M. Milne-Thomson) and 17 (elliptic integrals).
- Cited as: "A&S 16.4", "A&S 16.21.2-4", "A&S 17.6".
- Ledger ids: SCL-073
- Access: public domain (US government work); not fetched this session. Read so far: from memory.

### DLMF
- Reference: NIST Digital Library of Mathematical Functions, §19.36 (Carlson's duplication algorithm), §22.15 (inverse Jacobian functions), §19.25. https://dlmf.nist.gov/
- Cited as: "DLMF 19.36.1", "DLMF 22.15.13 with 19.25.5".
- Ledger ids: SCL-073
- Access: open; not fetched this session. Read so far: from memory.

### Carlson1995
- Reference: Carlson, B. C. (1995). Numerical computation of real or complex elliptic integrals. Numerical Algorithms 10:13-26. doi:10.1007/BF02198293.
- Cited as: "Carlson 1995".
- Ledger ids: SCL-073
- Access: arXiv math/9409227 (open); not fetched this session. Read so far: from memory.

## Seamless world projections (SCL-078 to SCL-085), added 2026-10-03 with the seamless-projection survey

The survey `docs/research/notes/seamless-projection-survey.md` and its tools (`tools/research/projection-survey/`) rest on these; nothing shipped does yet. "Sub-agent" means a Sonnet sub-agent of the survey session, whose transcriptions were kept in the session scratchpad (not committed, not in the source library).

### Petroff2021
- Reference: Petroff, M. A. (2021). A square equal-area map projection with low angular distortion, minimal cusps, and closed-form solutions. ACM Transactions on Spatial Algorithms and Systems 7(4), article 18. doi:10.1145/3460521. arXiv:2008.13670 (v3). Code: Zenodo, doi:10.5281/zenodo.4661351 (`new_projection.py`, CC0 1.0).
- Cited as: "Petroff 2021".
- Ledger ids: SCL-083, SCL-086
- Access: open (arXiv; Zenodo).
- Read so far (by the project): full text and code read by a sub-agent, 2026-10-03: forward Eqs 1-41, inverse Eqs 47-74, Table 1 (Collignon quincuncial mean omega 0.68 rad, max 1.05; Gringorten 0.52, 1.05; this work 0.54, 0.95), "phi0 was set to this exact value" (3 pi / 8); the inverse ported to numpy and Rust in the survey's tools and Table 1's mean and maximum reproduced. The 2026-10-02 note world-projection-options.md also read §§1-3 and Table 1. The supplement archive (square-map-projection-supplement.zip) downloaded from Zenodo 2026-10-04 and `new_projection.py` (both directions, CC0 header) ported line by line into `crates/deeptime-format/src/petroff.rs` (the seamless world, format 0.5). 2026-10-20: `new_projection.py` re-read at the forward's `alphaprime` step (its "Avoid divide by zero" fallback `alphaprime = (xiprime - np.pi / 6) * idx` where `uprime * rprime <= 0`, the cause of the forward's misses on exact inputs on the diagonals and at the cone points; SCL-083); the paper not re-read.

### RoscaPlonka2011
- Reference: Roşca, D. and Plonka, G. (2011). Uniform spherical grids via equal area projection from the cube to the sphere. Journal of Computational and Applied Mathematics 236(6):1033-1041. doi:10.1016/j.cam.2011.07.009.
- Cited as: "Rosca & Plonka 2011".
- Ledger ids: SCL-082
- Access: paywalled; author preprint (Göttingen, dated 2011-04-14) open.
- Read so far (by the project): the preprint read in full by a sub-agent, 2026-10-03; Eqs 12-13 (the area-preserving map from the square to the curved square), the inverse (§4) and Eqs 18-20 (inverse Lambert azimuthal) transcribed from page images; beta = sqrt(pi/6).

### CalabrettaGreisen2002
- Reference: Calabretta, M. R. and Greisen, E. W. (2002). Representations of celestial coordinates in FITS. Astronomy & Astrophysics 395:1077-1122. doi:10.1051/0004-6361:20021327. arXiv:astro-ph/0207413.
- Cited as: "Calabretta & Greisen 2002".
- Ledger ids: SCL-082 (QSC, the same map as Rosca & Plonka's)
- Access: open (arXiv).
- Read so far (by the project): §5.6 (TSC, CSC, QSC; Eqs 163-185 and the CSC coefficient tables) read by a sub-agent, 2026-10-03, equations confirmed on page images; quoted: QSC's derivation "applies only in the quadrant ... non-differentiable along the diagonals"; CSC's equal-area deviation "+13.7% and -4.1% at the edges of the face".

### ONeillLaubscher1976
- Reference: O'Neill, E. M. and Laubscher, R. E. (1976). Extended studies of a quadrilateralized spherical cube Earth data base. Naval Environmental Prediction Research Facility, Technical Paper 3-76 (title and report number from memory).
- Cited as: "O'Neill & Laubscher 1976".
- Ledger ids: SCL-082
- Access: not online; not fetched. Read so far: through Calabretta & Greisen 2002 and PROJ `qsc.cpp` (sub-agent).

### Rancic1996
- Reference: Rančić, M., Purser, R. J. and Mesinger, F. (1996). A global shallow-water model using an expanded spherical cube: gnomonic versus conformal coordinates. Quarterly Journal of the Royal Meteorological Society 122(532):959-982. doi:10.1002/qj.49712253209.
- Cited as: "Rancic, Purser & Mesinger 1996".
- Ledger ids: SCL-080, SCL-101 (the cube world's conformal table, 2026-10-22)
- Access: paywalled; not read (a sub-agent tried, 2026-10-03). The only open implementation found (CSIRO CCAM `jimcc_m.f90`) is GPL-3 and was not used. 2026-10-22 (cube world phase 0): not read; cited only as the conformal cubic grid the world's map is an instance of.

### Moser1965
- Reference: Moser, J. (1965). On the volume elements on a manifold. Transactions of the American Mathematical Society 120(2):286-294.
- Cited as: "Moser 1965".
- Ledger ids: SCL-081, SCL-102 (the cube world's simulation table, 2026-10-22)
- Access: not fetched. Read so far: from memory (the flow dx/dt = -grad u / rho_t with Lap u = rho - 1 is the standard textbook form of Moser's argument). 2026-10-22: still not fetched; the simulation's table was generated with that form and measured (SCL-081), not checked against the paper.

### DacorognaMoser1990
- Reference: Dacorogna, B. and Moser, J. (1990). On a partial differential equation involving the Jacobian determinant. Annales de l'Institut Henri Poincaré C, Analyse non linéaire 7(1):1-26 (journal, volume and pages from memory, not checked).
- Cited as: "Dacorogna & Moser 1990".
- Ledger ids: none (cited with Moser 1965 for the flow behind SCL-081 and SCL-102: a prescribed Jacobian determinant on a domain with boundary, which is the face-by-face problem the generator solves with zero normal flux)
- Access: not fetched. Read so far: from memory (added 2026-10-22, cube world phase 0, at the plan's D6).

### Shoemake1992
- Reference: Shoemake, K. (1992). Uniform random rotations. In: Kirk, D. (ed.), Graphics Gems III, Academic Press, San Diego, pp. 124-132 (editor, publisher and pages from memory, not checked).
- Cited as: "Shoemake 1992".
- Ledger ids: SCL-120 (the cube world's turn search draws its random turns as Shoemake's uniform unit quaternions: three uniform numbers u1, u2, u3 give (sqrt(1 - u1) sin 2 pi u2, sqrt(1 - u1) cos 2 pi u2, sqrt(u1) sin 2 pi u3, sqrt(u1) cos 2 pi u3))
- Access: not fetched (a book chapter). Read so far: from memory (added 2026-10-22, cube world phase 1). The construction is checked only by its use: a uniform draw is all the search needs, and the refinement does the rest.

### Thurston1998
- Reference: Thurston, W. P. (1998). Shapes of polyhedra and triangulations of the sphere. Geometry & Topology Monographs 1:511-549. arXiv:math/9801088.
- Cited as: "Thurston 1998".
- Ledger ids: none (the survey's §1: the cone-angle sum and the developing map near a cone point)
- Access: open (arXiv). Read so far: introduction and §8 read by a sub-agent, 2026-10-03: "the sum of the apex curvatures is 2π times the Euler characteristic".

### DimitrijevicStrobl2025
- Reference: Dimitrijević, A. and Strobl, P. (2025). Hexahedral projections: a comprehensive review and ranking. ISPRS International Journal of Geo-Information 14(3):122. doi:10.3390/ijgi14030122 (DOI from the journal's pattern, not checked).
- Cited as: "Dimitrijević & Strobl 2025".
- Ledger ids: none (the survey's cross-checks: QSC max a/b 1.555, HEALPix 2.430; continuity classes)
- Access: open. Read so far: the ranking, continuity and distortion sections read by a sub-agent from a Semantic Scholar copy, 2026-10-03.

### Xu2018
- Reference: Xu, J., Morris, P. J., Liu, J. and Holden, J. (2018). PEATMAP: Refining estimates of global peatland distribution based on a meta-analysis. CATENA 160:134-140. doi:10.1016/j.catena.2017.09.010 (DOI from the publisher URL S0341816217303004; not checked in Crossref).
- Cited as: "PEATMAP (Xu et al. 2018)".
- Ledger ids: BIO-137
- Access: paywalled at the publisher (an open copy exists at CORE, record 227455185); the paper's text was not fetched: ScienceDirect, CORE and Semantic Scholar all answered 403 (2026-10-03). Read so far: a search-result summary of the abstract (global peatland 4.23 million km2, 2.84 % of the land; northern peatlands 3.21 million km2), and two secondary pages: PMC11718193 (a 2025 study of Holocene peatland area, which writes "we used an area of 3.21 Mkm2 for the modern northern peatlands from PEATMAP by Xu et al.") and the International Peatland Society page "Where can peatlands be found?" (continent shares: Asia 38.4 %, North America 31.6 %, Europe 12.5 %; boreal and temperate northern peatlands "formed under high precipitation-low temperature climatic regimes").
## Biome rules for Expanded Ecosphere (BIO-115 to BIO-130), added 2026-10-04

### Walker2005
- Reference: Walker, D. A.; Raynolds, M. K.; Daniëls, F. J. A.; Einarsson, E.; Elvebakk, A.; Gould, W. A.; Katenin, A. E.; Kholod, S. S.; Markon, C. J.; Melnikov, E. S.; Moskalenko, N. G.; Talbot, S. S.; Yurtsev, B. A. and the CAVM Team (2005). The Circumpolar Arctic vegetation map. Journal of Vegetation Science 16:267-282.
- Cited as: "Walker et al. 2005 (CAVM), bioclimate subzones by summer warmth".
- Ledger ids: BIO-124
- Access: not fetched. Read so far: from memory (the five bioclimate subzones A to E, with July means rising from about 1-3 degC in A to about 9-12 degC in E; the 5 degC split used is near the B/C edge); to be checked against the paper's subzone table.

### Alaback1991
- Reference: Alaback, P. B. (1991). Comparative ecology of temperate rainforests of the Americas along analogous climatic gradients. Revista Chilena de Historia Natural 64:399-412.
- Cited as: "Alaback 1991, the climatic limits of temperate rainforest".
- Ledger ids: BIO-127
- Access: not fetched. Read so far: from memory (temperate rainforest with annual precipitation over about 1,400 mm, a mean annual temperature of about 4 to 12 degC and cool summers); to be checked.

## Terralith placement table and geology gate (BIO-140 to BIO-154), added 2026-10-04

### Scatena2010
- Reference: Scatena, F. N., Bruijnzeel, L. A., Bubb, P. and Das, S. (2010). Setting the stage. In: Bruijnzeel, L. A., Scatena, F. N. and Hamilton, L. S. (eds.), Tropical Montane Cloud Forests: Science for Conservation and Management, Cambridge University Press, pp. 3-13. ISBN 978-0-521-76035-5.
- DOI or URL: Cambridge's excerpt of Part I, https://www.hydrology.nl/images/docs/ihp/nl/Tropical_Montane_Cloud_Forests.pdf
- Cited as: "Scatena et al. 2010 (Tropical Montane Cloud Forests, ch. 1)".
- Ledger ids: BIO-145; BIO-144 (its latitude is not in the excerpt)
- Access: book; the publisher's excerpt is open.
- Read in this pass (2026-10-04, excerpt fetched and converted with pdftotext): the change from lowland to lower montane forest "is normally observed at the elevation where the average minimum temperature drops below 18 °C ... On large equatorial inland mountains this transition usually occurs at an altitude of 1200–1500 m.a.s.l. and coincides with the general range of incipient and intermittent cloud formation"; the regional altitude table gives lower limits of cloud forest from 1000 to 2000 m (most 1000–1500 m), and 400–700 m on small islands; the potential-distribution method it describes (Kapos et al. 2000) takes mountain land from 1000 m (300 m where the local relief exceeds 300 m). The latitude limits (about 23° N in Mexico, 25° S in Argentina) come from a search-result snippet of another work, not read.

## Sphere-native simulation grids, added 2026-10-04 with the sphere-native grids note

The research note `docs/research/notes/sphere-native-grids.md` and its benchmark (`tools/research/sphere-grid-bench/`) rest on these; nothing shipped does yet, so no ledger row cites them. "Sub-agent" means a Sonnet sub-agent of the note's session, whose transcriptions were kept in the session scratchpad (not committed, not in the source library). The note also cites, for their sphere use, works already listed above: [Braun2013] (not read), [Barnes2014], [Cortial2019], [SmithBarstad2004], [Hergarten2021], [Fastscapelib] (its `HealpixGrid`, release 0.3.0, 2025) and [Landlab] (its `IcosphereGlobalGrid`). The note's §13 lists every source with how it was read; the entries below are the ones its conclusions lean on.

### PutmanLin2007
- Reference: Putman, W. M. and Lin, S.-J. (2007). Finite-volume transport on various cubed-sphere grids. Journal of Computational Physics 227(1):55-78. doi:10.1016/j.jcp.2007.07.022.
- Ledger ids: SCL-142 (2026-10-22: the cube grid's corner ghost fill, FV3's x-direction rule as the sphere-native grids note describes it; not re-read). Earlier: the note's §2.2, gnomonic against conformal and elliptic cubed spheres, conformal min/mean cell 0.16 at c360; cited 2026-10-22 on seamless-world.md's cube section as background for the conformal cube's small corner cells.
- Access: author copy open. Read so far: full text read by a sub-agent as extracted text, 2026-10-04 (a web summariser's version was discarded after it invented content); not re-read for the cube world.

### Harris2021FV3
- Reference: Harris, L., Chen, X., Putman, W., Zhou, L. and Chen, J.-H. (2021). A scientific description of the GFDL finite-volume cubed-sphere dynamical core. NOAA Technical Memorandum OAR GFDL 2021-001 (authors from memory of the memo; the sub-agent recorded the memo number).
- Ledger ids: SCL-142 (2026-10-22: the cube grid's corner ghost fill follows FV3's `fill_corners` as the 2026-10-04 reading recorded it; no code was ported and the memo was not re-read). Earlier: §2.2, edge imprinting, the Duo-Grid.
- Access: open. Read so far: relevant sections by a sub-agent, 2026-10-04; FV3's `fv_arrays.F90`, `fv_mp_mod.F90` and `sw_core.F90` read as code (ghost width 3, corner fills).

### Ronchi1996
- Reference: Ronchi, C., Iacono, R. and Paolucci, P. S. (1996). The "cubed sphere": a new method for the solution of partial differential equations in spherical geometry. Journal of Computational Physics 124(1):93-114.
- Ledger ids: none (background for the cube world's gnomonic face coordinates, in which the cube map tables store their nodes; seamless-world.md, 2026-10-22)
- Access: paywalled. Read so far: abstract only (reference checked on Crossref by a sub-agent).

### Sadourny1972
- Reference: Sadourny, R. (1972). Conservative finite-difference approximations of the primitive equations on quasi-uniform spherical grids. Monthly Weather Review 100(2):136-144 (issue from memory; volume and pages as the sphere-native grids note lists them).
- Cited as: "Sadourny 1972".
- Ledger ids: none (background: the first cubed-sphere grid, cited on seamless-world.md's cube section, 2026-10-22)
- Access: not fetched. Read so far: the reference only (sphere-native-grids.md §13; added here 2026-10-22 at the plan's D6).

### StaniforthThuburn2012
- Reference: Staniforth, A. and Thuburn, J. (2012). Horizontal grids for global weather and climate prediction models: a review. Quarterly Journal of the Royal Meteorological Society 138(662):1-26. doi:10.1002/qj.958.
- Ledger ids: none (§2.1)
- Access: paywalled. Read so far: abstract (Crossref) and the ten desiderata as quoted in the authors' later GungHo paper (Staniforth, Melvin and Wood 2013), by a sub-agent.

### Gorski2005
- Reference: Górski, K. M., Hivon, E., Banday, A. J., Wandelt, B. D., Hansen, F. K., Reinecke, M. and Bartelmann, M. (2005) (full author list, pages and issue from memory). HEALPix: a framework for high-resolution discretization and fast analysis of data distributed on the sphere. The Astrophysical Journal 622(2):759-771. arXiv:astro-ph/0409513.
- Ledger ids: none (§2.4, §3.1; the benchmark's HEALPix centres follow its nested-scheme face formulas)
- Access: open (arXiv). Read so far: full text by two sub-agents, 2026-10-04.

### TomitaSatoh2004 and Satoh2014
- Reference: Tomita, H. and Satoh, M. (2004). A new dynamical framework of nonhydrostatic global model using the icosahedral grid. Fluid Dynamics Research 34:357-400 (the sub-agent read the authors' 2004 ECMWF seminar paper on NICAM); Satoh, M. et al. (2014). The Non-hydrostatic Icosahedral Atmospheric Model: description and development. Progress in Earth and Planetary Science 1:18. doi:10.1186/s40645-014-0018-1.
- Ledger ids: none (§2.3: structured 2-D storage on 10 diamonds; N² against N³ cost; memory throughput)
- Access: open (the seminar paper and PEPS). Read so far: relevant sections by a sub-agent; Yashiro's 2014 ECMWF slides (NICAM data structure and offsets) in full by another.

### Adamidis2025
- Reference: Adamidis, P. et al. (2025). Geoscientific Model Development 18:905 (an exascale case study of ICON's performance).
- Ledger ids: none (§3.2: "horizontal neighbour relations cannot be exploited in the memory layout"; ICON behaves like HPCG)
- Access: open. Read so far: relevant sections by a sub-agent.

### KageyamaSato2004
- Reference: Kageyama, A. and Sato, T. (2004). "Yin-Yang grid": an overset grid in spherical geometry. Geochemistry, Geophysics, Geosystems 5(9):Q09005. arXiv:physics/0403123.
- Ledger ids: none (§2.1, §2.4)
- Access: open (arXiv). Read so far: full text by a sub-agent.

### Malardel2016
- Reference: Malardel, S., Wedi, N., Deconinck, W., Diamantakis, M., Kühnlein, C., Mozdzynski, G., Hamrud, M. and Smolarkiewicz, P. (2016) (full author list, pages and issue from memory). A new grid for the IFS. ECMWF Newsletter 146:23-28.
- Ledger ids: none (§2.1, §2.4)
- Access: open. Read so far: full text by a sub-agent.

### Schaeffer2013
- Reference: Schaeffer, N. (2013). Efficient spherical harmonic transforms aimed at pseudospectral numerical simulations. Geochemistry, Geophysics, Geosystems 14(3):751-758. arXiv:1202.6522.
- Ledger ids: none (§5: SHTns timings, Table 1)
- Access: open (arXiv). Read so far: full text by a sub-agent.

### Stevens2019
- Reference: Stevens, B. et al. (2019). DYAMOND: the DYnamics of the Atmospheric general circulation Modeled On Non-hydrostatic Domains. Progress in Earth and Planetary Science 6:61.
- Ledger ids: none (§2.2: simulated days per day by model)
- Access: open. Read so far: relevant sections by a sub-agent.

### Araujo2021
- Reference: Araújo, R. C., Gross, L. and Xavier-de-Souza, S. (2021). Boosting memory access locality of the spectral element method with Hilbert space-filling curves. arXiv:2104.08416.
- Ledger ids: none (§3.2)
- Access: open. Read so far: full text by a sub-agent.

### Aupy2016
- Reference: Aupy, G., Park, J. and Raghavan, P. (2016). Locality-aware Laplacian mesh smoothing. arXiv:1606.00803.
- Ledger ids: none (§3.2)
- Access: open. Read so far: full text by a sub-agent.

### Bercea2016
- Reference: Bercea, G.-T. et al. (2016). A structure-exploiting numbering algorithm for finite elements on extruded meshes, and its performance evaluation in Firedrake. arXiv:1604.05937 (published in Geoscientific Model Development 9).
- Ledger ids: none (§3.2)
- Access: open. Read so far: full text by a sub-agent.

### Kreutzer2014
- Reference: Kreutzer, M., Hager, G., Wellein, G., Fehske, H. and Bishop, A. R. (2014) (full author list, pages and issue from memory). A unified sparse matrix data format for efficient general sparse matrix-vector multiplication on modern processors with wide SIMD units. SIAM Journal on Scientific Computing 36(5):C401-C423. arXiv:1307.6209.
- Ledger ids: none (§3.2: SIMD and memory-bound kernels)
- Access: open (arXiv). Read so far: full text by a sub-agent.

### Salles2020 and Salles2023
- Reference: Salles, T., Mallard, C. and Zahirovic, S. (2020) (full author list, pages and issue from memory). gospl: Global Scalable Paleo Landscape Evolution. Journal of Open Source Software 5(56):2804. doi:10.21105/joss.02804; Salles, T. et al. (2023). Nature 624:115-121, doi:10.1038/s41586-023-06777-z (Methods describe goSPL's global runs); Salles, T. et al. (2023). Hundred million years of landscape dynamics from catchment to global scale. Science 379:918-923, doi:10.1126/science.add2541.
- Ledger ids: none (§4)
- Access: JOSS and the Nature Methods (PMC) open; Science paywalled. Read so far: JOSS and Nature Methods in full, goSPL documentation (technical guide, HPC scaling) by a sub-agent; Science abstract only.

### CordonnierBovyBraun2019
- Reference: Cordonnier, G., Bovy, B. and Braun, J. (2019). A versatile, linear complexity algorithm for flow routing in topographies with depressions. Earth Surface Dynamics 7:549-562. doi:10.5194/esurf-7-549-2019.
- Ledger ids: none (§4: routing on any planar graph)
- Access: open. Read so far: full text by a sub-agent.

### Cordonnier2016
- Reference: Cordonnier, G., Braun, J., Cani, M.-P., Benes, B., Galin, É., Peytavie, A. and Guérin, É. (2016) (full author list, pages and issue from memory). Large scale terrain generation from tectonic uplift and fluvial erosion. Computer Graphics Forum 35(2):165-175. doi:10.1111/cgf.12820.
- Ledger ids: none (§4, §6)
- Access: author copy open. Read so far: full text by two sub-agents.

### HergartenRobl2022
- Reference: Hergarten, S. and Robl, J. (2022). The linear feedback precipitation model (LFPM 1.0): a simple and efficient model for orographic precipitation in the context of landform evolution modeling. Geoscientific Model Development 15:2063-2084. doi:10.5194/gmd-15-2063-2022.
- Ledger ids: CLM-193, CLM-194, CLM-195, CLM-198, CLM-200 (the upwind rain, P1-26 T9; also §5 of the sphere-grids note: a non-spectral orographic rain model; §11 question 3)
- Access: open. Read so far: full text by a sub-agent; the article page (WebFetch) by the P1-26 T9 agent on 2026-10-04: equations 10-11 (vapour and cloud water, condensation, fallout, re-evaporation beta), 27 (L_l L_s = L_c L_f), 42-43 (evapotranspiration returns epsilon of the rain) and section 7's lengths (L_c = L_f = 25 km, L_l = 500 km, L_d = 25 km). Written from the paper, not from OpenLEM's code (GPL).

## Rifted margins: the ocean–continent transition (PLT-261 to PLT-264), added 2026-10-04

Read by a research sub-agent of the transition task (2026-10-04); quotes are verbatim from the text
it fetched. Many publisher pages refused it (Wiley, ScienceDirect, GeoScienceWorld); what it could
not open is listed under "Not read". Bradley 2008 (above) was read in full in the same pass.

### Franke2013
- Reference: Franke, D. (2013). Rifting, lithosphere breakup and volcanism: comparison of magma-poor and volcanic rifted margins. Marine and Petroleum Geology 43:63-87.
- DOI or URL: 10.1016/j.marpetgeo.2012.11.003; author's accepted manuscript at http://mantleplumes.webspace.durham.ac.uk/wp-content/uploads/sites/129/2021/05/Franke_RiftingMagmatism_MPG.pdf
- Cited as: "Franke 2013" (the width of crustal thinning at volcanic margins).
- Ledger ids: PLT-262
- Access: accepted manuscript, open.
- Read so far (by the project): full text (accepted manuscript), 2026-10-04: at volcanic margins "Crustal thinning typically occurs over a comparably short distance in the order of only 50 to 100 km"; "Magma-poor rifted margins in contrast show wide domains of extended crust"; volcanic rifted margins are "the majority of passive continental margins worldwide".

### Brune2014
- Reference: Brune, S., Heine, C., Pérez-Gussinyé, M. and Sobolev, S. V. (2014). Rift migration explains continental margin asymmetry and crustal hyper-extension. Nature Communications 5:4014.
- DOI or URL: 10.1038/ncomms5014
- Cited as: "Brune et al. 2014" (hyper-extended crust up to 200 km wide).
- Ledger ids: PLT-261, PLT-262
- Access: open access.
- Read so far (by the project): full text, 2026-10-04: "up to 200 km wide areas of hyper-extended continental crust"; "hyper-extended (with a thickness <10 km) crustal slivers that vary in widths between 70 km offshore Iberia and 200 km off Angola".

### Sapin2021
- Reference: Sapin, F., Ringenbach, J.-C. and Clerc, C. (2021). Rifted margins classification and forcing parameters. Scientific Reports 11:8199.
- DOI or URL: 10.1038/s41598-021-87648-3
- Cited as: "Sapin et al. 2021" (the necking domain's width; crust at the transition).
- Ledger ids: PLT-261, PLT-262
- Access: open access.
- Read so far (by the project): full text, 2026-10-04: "The width of the Necking Domain: this domain is the most variable in terms of width and general shape"; narrow, "~ 50 km", to "extending over more than 110 km across the shelf"; "the Exhumation Domain can reach a width of more than 150 km"; in the Coral Sea "a thinning of the crust from 22 km down to a 7–8 km thick ... in direct contact with the steady oceanic crust of similar thickness".

### Harris2014
- Reference: Harris, P. T., Macmillan-Lawler, M., Rupp, J. and Baker, E. K. (2014). Geomorphology of the oceans. Marine Geology 352:4-24.
- DOI or URL: 10.1016/j.margeo.2014.01.011; author copy https://moodle2.units.it/pluginfile.php/486751/mod_folder/content/0/HarrisetalMARGO2014.pdf
- Cited as: "Harris et al. 2014" (shelf and slope widths of passive margins; shelf, slope and rise areas).
- Ledger ids: PLT-262, PLT-265, HYP-050 (2026-10-22: the hadal floor)
- Access: author copy, open.
- Read so far (by the project): full text, 2026-10-04: "continental shelf width of passive margins (88 km) is nearly three times that of active margins (31 km)" (abstract; Table 2: 84 ± 0.66 km, all shelves 57 km); "the average width of active slopes (36 km) is less than the average width of passive margin slopes (46 km)"; slope "a narrow band 41 km wide"; Table 5: shelf 8.91 % of the ocean, slope 5.42 %, rise 8.24 % (superimposed on the abyss); "the Earth's margins are approximately 35% active and 65% passive". No shelf-break depth.
- Read in the trench-budget task (2026-10-22, the full text through the Wayback Machine's copy of the cbics.org PDF, by a research sub-agent; HYP-050): Table 5, "Hadal | Seafloor occurring at depths >6000 m. | 3,437,930 km2 | 0.95 %" of the "361,883,510 km2" mapped on the "SRTM bathymetry grid" (North Pacific 2,296,130 km2); "Trench | A long narrow, characteristically very deep and asymmetrical depression of the sea floor, with relatively steep sides. | 1,967,350 km2 | 0.544 %", "A total of 56 trenches were mapped".

### McKenzie1978
- Reference: McKenzie, D. (1978). Some remarks on the development of sedimentary basins. Earth and Planetary Science Letters 40:25-32.
- DOI or URL: 10.1016/0012-821X(78)90071-7 (the DOI from the standard citation, not seen on the scan); scan at http://www.zetaware.com/public/McKenzie_1978.pdf
- Cited as: "McKenzie 1978" (uniform stretching by β; initial and thermal subsidence).
- Ledger ids: PLT-261, PLT-263, PLT-264, ISO-078 (2026-10-20, the continent budget: τ = 62.8 My of Table 1, tried as the settling's time and dropped, `plates/settle.rs`), PLT-415 (2026-10-22, the high-ground task: τ = 62.8 My as the e-fold of the thin sheet's viscosity with the age of the orogeny; the form is the agent's), ISO-099 (2026-10-23, the release's pace: τ as the time a plate the `Te` law weakens takes to warm; the form is the agent's), PLT-460 and PLT-465 and PLT-466 (2026-10-23, the lithosphere's thermal state: the plate's a and τ, τ scaled with a²), PLT-476 (2026-10-23, stage 2: the lithosphere stretched by the crust's β)
- Read in the stage 2-3 task (2026-10-23, the scan by a sub-agent, eq. 1 on the page image; PLT-476): eq. 1, S_i ∝ (1 − 1/β), "where a is the thickness of the lithosphere and t_c the initial thickness of the continental crust"; "S_i is positive if t_c >~ 18 km. Hence land areas will subside but regions with thin crust can be elevated by the stretching"; Table 2, total thermal subsidence e(0) 781 m at β = 1.25, 1,300 at 1.5, 1,950 at 2, 2,930 at 4, 3,520 at 10, 3,890 at infinity; real basins "an approximate value of β estimated from Fig. 4 of 1.5" (the North Sea) and "β ~ 2" (the Aegean).
- Access: scanned copy, open.
- Read so far (by the project): full text (OCR; the equations partly garbled), 2026-10-04: "a unit length of continental lithosphere is suddenly extended to a length β, causing upwelling of hot asthenosphere"; "The slow subsidence and the heat flow depend only on the amount of stretching"; Table 1: τ = 62.8 My, E0 = 3.2 km, ρ_c 2.8, ρ_m 3.33, ρ_w 1.0 g/cm³; thermal subsidence S_t ≈ E0 r (1 − e^(−t/τ)), r = (β/π) sin(π/β).
- Read in the release-pace task (2026-10-23, the scan's full text by a Sonnet sub-agent of the release agent, eq. 6 checked on the page image; ISO-099): eq. 6, τ = a²/(π²κ), κ "the thermal diffusivity" (Table 1 gives no κ; 0.0080 cm²/s back from τ and a, the sub-agent's arithmetic); Table 1 "(mostly taken from Parsons and Sclater [9])": "a = 125 km", "Tl = 1333°C", "τ = 62.8 My"; "After a time which depends on the thermal time constant of the stretched lithosphere, τ/β², the heat flux starts to decrease."

Not read: Péron-Pinvidic, Manatschal & Osmundsen 2013 (Marine and Petroleum Geology 43:21-47), Sutra et al. 2013 (G-cubed 14:2575), Mohn et al. 2012 (Tectonics 31:TC1012), Chenin et al. 2017 (Geosphere 13:559; a search summary gives magma-poor margins "approximately 165 km on average", unverified).

### Portenga2011
- Reference: Portenga, E. W. and Bierman, P. R. (2011). Understanding Earth's eroding surface with 10Be. GSA Today 21(8):4-10.
- DOI or URL: 10.1130/G111A.1 (the DOI from memory, not checked); https://www.geosociety.org/gsatoday/archive/21/8/article/i1052-5173-21-8-4.htm
- Cited as: "Portenga & Bierman 2011" (global compilation of cosmogenic erosion rates; the erosion audit's reference line).
- Ledger ids: ERO-171, ISO-076
- Access: open (GSA Today).
- Read so far (by the project): full text, 2026-10-05, and again by the polar-holes agent, 2026-10-18 ("Drainage basins erode more quickly (mean = 218 m Myr−1; median = 54 m Myr−1)"): outcrops 12 ± 1.3 m/Myr mean, 5.4 median (n = 450); drainage basins 218 ± 35 mean, 54 median (n = 1,149); seismically active basins 367 ± 55 (n = 221), inactive 182 ± 30 (n = 928); active and inactive outcrops alike (14 ± 1.6 and 13 ± 1.4).

### Montgomery2002
- Reference: Montgomery, D. R. and Brandon, M. T. (2002). Topographic controls on erosion rates in tectonically active mountain ranges. Earth and Planetary Science Letters 201(3-4):481-489.
- DOI or URL: 10.1016/S0012-821X(02)00725-2 (the DOI from memory, not checked); copy https://repository.geologyscience.ru/bitstream/handle/123456789/28097/Mont_02.pdf
- Cited as: "Montgomery & Brandon 2002" (erosion rate against mean local relief; the erosion audit's reference line).
- Ledger ids: ERO-172, ERO-179, ERO-180, ERO-181
- Access: copy on a repository, open.
- Read so far (by the project): full text, 2026-10-05: tectonically inactive areas E = 0.2 Rz (mm/yr, Rz in km); active ranges above that line with mean local relief 1,000-2,000 m; E0 = 0.01 mm/yr from the chemical denudation of the 35 largest rivers; Olympic Mountains E0 0.016-0.059 mm/yr; rates above 10 mm/yr rare. Read again 2026-10-18 (the same copy, text extracted with pdftotext) for the erosion recalibration: eq. 2, E = E0 + K Rz / (1 - (Rz/Rc)^2), fitted with E0 = 0.01 mm/yr, Rc = 1,500 m and K = 2.5e-4 mm/yr (Fig. 4); the power law E = 1.4e-6 Rz^1.8 (E mm/yr, Rz m, Rz above 100 m; R² 0.66; one standard error 1.6 to 2.0 for the exponent), which "underestimates erosion rates at high mean local relief" (Fig. 5); Rz is the range within a 10-km-diameter circle of each 30-arc-second GTOPO30 cell; at most 2 % of the land of Asia, Europe and the Americas has Rz above 1,500 m and at most 5 % above 1,000 m (Fig. 7).

### Tremblay2015
- Reference: Tremblay, M. M. et al. (2015). Erosion in southern Tibet shut down at ∼10 Ma due to enhanced rock uplift within the Himalaya. Proceedings of the National Academy of Sciences 112(39):12030-12035.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC4593086
- Cited as: "Tremblay et al. 2015" (the plateau's slow exhumation; the erosion audit's reference line for high ground).
- Ledger ids: ERO-174
- Access: open (PubMed Central).
- Read so far (by the project): full text, 2026-10-05: exhumation over 1.0 km/Ma from 17-16 Ma for about 5 Myr, then below 0.01 km/Ma (under 750 m in total) from 12-11 Ma to the present; cosmogenic rates in the Indus and Yarlung drainages typically below 100 m/Ma.

### Ruetenik2023
- Reference: Ruetenik, G. A., Jansen, J. D., Val, P. and Ylä-Mella, L. (2023). Optimising global landscape evolution models with 10Be. Earth Surface Dynamics 11:865-880.
- DOI or URL: 10.5194/esurf-11-865-2023; https://esurf.copernicus.org/articles/11/865/2023/
- Cited as: "Ruetenik et al. 2023" (the stream power law's slope exponent n fitted to the world's cosmogenic denudation rates; the source of the shipped n).
- Ledger ids: ERO-182, ERO-006
- Access: open (Copernicus, CC BY).
- Read so far (by the project): full text through the web fetch tool, 2026-10-18: 3,640 catchments on 3-arcsec HydroSHEDS DEMs, the model run for one step and its outlet flux compared with OCTOPUS v.2 rates; stream power alone optimised at n about 1.28 (Q0.01 1.23 to 1.43); with linear hillslope diffusion n about 2.3 (Q0.01 1.59 to 2.70); m/n held at 0.45; no absolute K reported, since K and D covary; their stream-power-only n of about 1.3 is "much lower than the 2.6 reported by Harel et al. (2016)".
## The eclogite root and whole-column isostasy (PLT-278 to PLT-282), added 2026-10-06

Read by a research sub-agent on 2026-10-05 for plate task 7 (Ben, 2026-10-06: "Fix isostasy first, then land"). "Full text" means the agent read the paper; "snippet" means only a search tool's excerpt, not checked against the paper.

### Wittlinger2009
- Reference: Wittlinger, G., Farra, V., Hetényi, G., Vergne, J. and Nábělek, J. (2009). Seismic velocities in Southern Tibet lower crust: a receiver function approach for eclogite detection. Geophysical Journal International 177(3):1037-1049.
- DOI or URL: http://real.mtak.hu/13079/1/1280309.pdf (open copy)
- Cited as: "Wittlinger et al. 2009".
- Ledger ids: PLT-278, PLT-280
- Access: open repository copy.
- Read so far (by the project): full text, 2026-10-05: "If the required temperatures (~600° to ~700°) and pressures (~15 to ~20 kbars) are plausible in the lower crust at ~60 km depth beneath southern Tibet (Mechie et al. 2004), the origin of the hydrous fluids remains debatable"; Moho "~75 to ~78 km"; the abstract reads the lower-crust layer as "the about 19 km thick eclogitic layer". Co-authors written from memory; check before quoting.

### ShiF2018
- Reference: Shi, F., Wang, Y., Yu, T., Zhu, L., Zhang, J., Wen, J., Gasc, J., Incel, S., Schubnel, A., Li, Z., Chen, T., Liu, W., Prakapenka, V. and Jin, Z. (2018). Lower-crustal earthquakes in southern Tibet are linked to eclogitization of dry metastable granulite. Nature Communications 9:3483.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC6113232/
- Cited as: "Shi F. et al. 2018".
- Ledger ids: PLT-278, PLT-280
- Access: open (PubMed Central).
- Read so far (by the project): full text, 2026-10-05: "below ~60 km the lower crust may have reached roughly 30% eclogitization, assuming a homogeneous mixture of granulite- and eclogite-facies minerals"; "Actual degrees of eclogitization depend on temperature, age, and volatiles and therefore vary"; Moho "~80 km". Co-authors written from memory; check before quoting.

### ShiD2020
- Reference: Shi, D., Klemperer, S. L., Shi, J., Wu, Z. and Zhao, W. (2020). Localized foundering of Indian lower crust in the India-Tibet collision zone. Proceedings of the National Academy of Sciences 117(40):24742-24747.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC7547243/
- Cited as: "Shi D. et al. 2020".
- Ledger ids: PLT-279, PLT-280, ISO-043, PLT-477 (2026-10-23: why stage 3's root removal leaves the eclogite pool)
- Access: open (PubMed Central).
- Read so far (by the project): full text, 2026-10-05: densities "granulitic rocks (2,850 kg/m3...)", "the mantle (3,300 kg/m3...)", "mafic eclogite (3,450 kg/m3) (values taken from ref. 16)" (Hetényi et al. 2007); "the lower crust being only ~30% eclogitized so gravitationally stable"; "the doublet layer has a density ~3,000-3,100 kg/m3"; Moho locally "~90 km". Co-authors written from memory; check before quoting.

### KrystopowiczCurrie2013
- Reference: Krystopowicz, N. J. and Currie, C. A. (2013). Crustal eclogitization and lithosphere delamination in orogens. Earth and Planetary Science Letters 361:195-207.
- DOI or URL: https://sites.ualberta.ca/~ccurrie1/papers/KrystopowiczCurrie_EPSL_2013.pdf
- Cited as: "Krystopowicz & Currie 2013".
- Ledger ids: PLT-278, PLT-279, PLT-280
- Access: author's copy.
- Read so far (by the project): full text, 2026-10-05: the eclogite field from the basalt diagram of Hacker et al. 2003; "Kinetic hinderance will delay the density change"; water appears required; "Assuming densities of 3000 kg/m3 for mafic granulite and 3500-3600 kg/m3 for mafic eclogite"; Table 1 eclogite 3450, crust 2800, mantle 3250; "7-8% density increase corresponds to ~35-50% eclogitization"; exhumed lower crust of the Urals, Caledonides and Dabie Shan shows partial eclogitization; the South Urals' lithosphere persists through a felsic composition or "kinetic hindrance due to the absence of fluids".

### HyndmanCurrie2011
- Reference: Hyndman, R. D. and Currie, C. A. (2011). Why is the North America Cordillera high? Hot backarcs, thermal isostasy, and mountain belts. Geology 39(8):783-786.
- DOI or URL: http://web.uvic.ca/~rhyndman/PublicationPDFs/Hyndman&Currie2011WithFigs&Appendix.pdf
- Cited as: "Hyndman & Currie 2011".
- Ledger ids: PLT-281, PLT-282, ISO-110 and ISO-111 (2026-10-23, the lithosphere's thermal state: E = 1,600 m and the frame)
- Access: author's copy.
- Read so far (by the project): full text, 2026-10-05: "The slopes between crustal thickness and elevation provide the crust-mantle density contrast, 131±12 m/km for the Cordillera and 141±27 m/km for stable areas"; the reference crust 2850 and mantle 3260 kg/m3 give 126 m/km; sea-level crustal thickness ~27 km (Cordillera) and ~38 km (craton); thermal state moves elevation by about 1,600 m at equal thickness; "the Himalaya-Tibet region of India-Asia collision where the crust reaches ~70 km and elevations ~5 km".

### LuffiDucea2022
- Reference: Luffi, P. and Ducea, M. N. (2022). Chemical mohometry: assessing crustal thickness of ancient orogens using geochemical and isotopic data. Reviews of Geophysics 60(2):e2021RG000753.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC9788079/
- Cited as: "Luffi & Ducea 2022".
- Ledger ids: PLT-281, ISO-043
- Access: open (PubMed Central).
- Read so far (by the project): full text, 2026-10-05, the arc relation: "H = 6.79 (±0.95) × h + 26.40 (±1.45), R2 = 0.93"; Airy "requires ρm/ρc = 1.15, which is satisfied by a 2.83-2.87 g/cm3 arc crust underlain by a 3.25-3.30 g/cm3 lithospheric mantle".

### Buntin2021
- Reference: Buntin, S. and others (2021). [Seismic velocities and densities of mantle rocks and eclogites; title not recorded by the sub-agent.] Nature Communications 12.
- DOI or URL: https://pmc.ncbi.nlm.nih.gov/articles/PMC8589968/
- Cited as: "Buntin et al. 2021".
- Ledger ids: PLT-279
- Access: open (PubMed Central).
- Read so far (by the project): full text, 2026-10-05: "3.23-3.39 g/cm3 for peridotites with mean values of 3.31-3.35 g/cm3 and up to 3.75 g/cm3 for eclogites, with common values of 3.45-3.50 g/cm3 reported worldwide". Title and co-authors to be taken from the record before quoting.

### Carbonell1996
- Reference: Carbonell, R. and others (1996). [Deep seismic profile across the southern Urals; title not recorded by the sub-agent.] Science 274:222.
- Cited as: "Carbonell et al. 1996".
- Ledger ids: PLT-282
- Access: paywalled; abstract read (via Crossref).
- Read so far (by the project): abstract, 2026-10-05: "12- to 15-kilometer-thick crustal root, yielding a total crustal thickness of 55 to 58 kilometers", with a "sharp" crust-mantle transition.

### Stadtlander1999
- Reference: Stadtlander, R., Mechie, J. and Schulze, A. (1999). Deep structure of the southern Ural mountains as derived from wide-angle seismic data. Geophysical Journal International 137:501.
- Cited as: "Stadtlander, Mechie & Schulze 1999".
- Ledger ids: PLT-282
- Access: paywalled; abstract read (via OpenAlex).
- Read so far (by the project): abstract, 2026-10-05: a "15-18 km thick crustal root" centred 50-80 km east of the highest topography; "If the deeper parts of the thickened crust also have high densities (small density contrast of about -0.1 g cm-3 with respect to the uppermost mantle) then this helps to explain the absence of a pronounced gravity minimum"; the high velocities read as mafic or ultramafic rock, not eclogite. Title as recorded by the sub-agent; check before quoting.

### Artyushkov2000
- Reference: Artyushkov, E. V. and others (2000). [On the Southern Urals' crust and its low altitude.] Tectonophysics 320:271.
- Cited as: "Artyushkov et al. 2000".
- Ledger ids: PLT-282
- Access: paywalled; search snippet only.
- Read so far (by the project): snippet only, 2026-10-05, not checked against the paper: "Since the mean altitude of the Southern Urals is only 600-800 m, their crust should have a high mean density. A layer of dense mafic rocks, ~20 km thick, still exists at the base of the Uralian crust"; crust "up to ~60 km thick, at a mean altitude ~0.5 km".

### Leech2001
- Reference: Leech, M. L. (2001). Arrested orogenic development: eclogitization, delamination, and tectonic collapse. Earth and Planetary Science Letters 185:149-159.
- Cited as: "Leech 2001".
- Ledger ids: PLT-278, PLT-282
- Access: paywalled; search snippet only.
- Read so far (by the project): snippet only, 2026-10-05, not checked against the paper: "it is likely that the base of the Uralian crust has not undergone major eclogitization and therefore is under fluid-absent conditions"; "fluid is required for complete eclogitization".

## Plateau erosion and the hillslope diffusivity (ERO-175 to ERO-178), added 2026-10-16

Roering1999 (the measured diffusivity, ERO-177) is under "Erosion, isostasy and hypsometry" above; its entry records what was read on 2026-10-16.

### Salles2019
- Reference: Salles, T. (2019). eSCAPE: Regional to Global Scale Landscape Evolution Model v2.0. Geoscientific Model Development 12:4165-4184.
- DOI or URL: https://gmd.copernicus.org/articles/12/4165/2019/ (the DOI 10.5194/gmd-12-4165-2019 from the journal's pattern, not checked; the end page from memory)
- Cited as: "Salles 2019 (eSCAPE)" (the hillslope diffusion coefficient a global landscape evolution model takes at coarse resolution).
- Ledger ids: ERO-178
- Access: open (Copernicus).
- Read so far (by the project): full text through the web fetch tool, 2026-10-16: §3.2.2, "hillslope processes are also accounted for using a diffusion coefficient D of 10−2 m2 yr−1" on a grid "built with a resolution of approximately 200 m"; §3.2.3 (the global run), "the diffusion coefficient D equals 10−1" (no units in that sentence) on meshes "at 16 km resolution". The fetched text does not say that D is adjusted for the resolution.

## The crust budget on sourced rates (plate task 8: PLT-045 to PLT-048, PLT-285, HNT-125 to HNT-127), added 2026-10-17

Stern & Scholl 2010 (the rates, PLT-045 and PLT-285) and Cloos 1993 (the arcs' maturity, PLT-047) are above; their entries record what was read on 2026-10-17. Fetched and read by a research sub-agent and checked by the plate task 8 agent against the text files, 2026-10-17; texts in the project's source library on the test VM (`/data/ban/deeptime-sources/<key>/`), not distributed.

### White2006
- Reference: White, S. M., Crisp, J. A. and Spera, F. J. (2006). Long-term volumetric eruption rates and magma budgets. Geochemistry, Geophysics, Geosystems 7, doi:10.1029/2005GC001002.
- DOI or URL: 10.1029/2005GC001002
- Cited as: "White et al. 2006" (the intrusive to extrusive ratio of the island arcs' crust).
- Ledger ids: PLT-046
- Access: open copy (the copy read is the journal's proof, with line numbers; issue and article number not recorded).
- Library (the project's source library, not distributed): `White2006/White2006.pdf`, `White2006.txt` (pdftotext), 2026-10-17.
- Read so far (by the project): full text, 2026-10-17: "A ratio of 5:1 could be viewed as common to most magmatic systems when the considerable uncertainty is considered. Ratios higher than 10:1 are uncommon in our data set"; "an I:E ratio of 5:1 may be regarded as a better average value"; Table 3: the Aleutians 1:1 to 3:1, Hawaii 6:1; basaltic shields 1:1 to 2:1, oceanic ridges at least 5:1.

### Eichelberger2015
- Reference: Eichelberger, N., McQuarrie, N., Ryan, J., Karimi, B., Beck, S. and Zandt, G. (2015). Evolution of crustal thickening in the central Andes, Bolivia. Earth and Planetary Science Letters 426:191-203.
- DOI or URL: 10.1016/j.epsl.2015.06.035 (from the PDF's first page)
- Cited as: "Eichelberger et al. 2015" (the cordillera's shortening rate).
- Ledger ids: PLT-048, HYP-018, HNT-127
- Access: open copy.
- Library (the project's source library, not distributed): `Eichelberger2015/Eichelberger2015.pdf`, `Eichelberger2015.txt`, 2026-10-17.
- Read so far (by the project): full text, 2026-10-17: Bolivia has "the largest shortening magnitudes (~300 km, 40%) and crustal thicknesses (up to 75 km) of the entire chain" (p. 191); "As shortening accumulates from the restored state (50 Ma) to the modern state (0 Ma), the area of the mesh elements contracts. Assuming pure shear deformation, the decrease in area leads to a proportional increase in predicted thickness" (p. 193), "calculated in 5 Myr intervals by assuming constant volume"; magmatic addition about 15 % (p. 197) to about 30 % (p. 200) of the crust in the models.

### Clift2009
- Reference: Clift, P. D., Vannucchi, P. and Phipps Morgan, J. (2009). Crustal redistribution, crust-mantle recycling and Phanerozoic evolution of the continental crust. Earth-Science Reviews 97:80-104.
- DOI or URL: 10.1016/j.earscirev.2009.10.003; repository copy https://hdl.handle.net/2158/375652 (FLORE, Università degli Studi di Firenze)
- Cited as: "Clift et al. 2009" (comparison for both arc rates).
- Ledger ids: PLT-045, PLT-285, PLT-401 (2026-10-21: the rifted margins lost in Cenozoic collisions)
- Access: open (accepted manuscript in the repository).
- Library (the project's source library, not distributed): `Clift2009/Clift2009.pdf`, `Clift2009.txt`, 2026-10-17.
- Read so far (by the project): full text, 2026-10-17: "oceanic arcs average rates of 125 km3/Ma/km compared to 81 km3/Ma/km for continental arcs" (p. 84); to keep constant crustal volumes "arc magmatism would have to maintain production of around 3.8 km3/a (or 94 km3/Ma/km of trench)"; continental flood basalts "only around 0.12 km3/a"; accreted oceanic plateaus 1.1 km3/a.
- Read in the collision-flux task (2026-10-21, the library text, section 5.1; PLT-401): the rifted margins lost in Cenozoic collisions, "In total these sum to almost 16,000 km of rifted margin (Table 2)", taken at "an average crustal thickness of 18 km (half normal continental (Christensen and Mooney, 1995))", "around 50% of the subducted margins were volcanic (50-km-wide continent-ocean transition (COT)) and the remainder non-volcanic (150-km-wide COT)"; "We calculate that around 28 × 10^6 km3 would have been subducted since 65 Ma, resulting in a long-term average loss of 0.43 km3/a", "as high as 0.65 km3/a" if all were non-volcanic. Figure 7's caption: Taiwan's collision is "transferring oceanic arc crust to the Asian margin (88%)".

### Parcutela2023
- Reference: Parcutela, N. E., Austria, R. S., Dimalanta, C. B., Valera, G. T. V., Gabo-Ratio, J. A. S., Payot, B. D., Armada, L. T., Sangalang, K. J. F. and Yumul, G. P. Jr. (2023). Enhanced arc magmatic productivity of the Western Pacific island arcs deduced from gravity-derived arc crustal growth rates. Frontiers in Earth Science 11:1107833.
- DOI or URL: 10.3389/feart.2023.1107833
- Cited as: "Parcutela et al. 2023" (a lower island-arc rate).
- Ledger ids: PLT-045
- Access: open access.
- Library (the project's source library, not distributed): `Frontiers2023_WPacificArcs/Frontiers2023_WPacificArcs.pdf`, `.txt`, 2026-10-17. (The volume number 11 is from memory.)
- Read so far (by the project): full text, 2026-10-17: "the Pacific island arcs (i.e., Bonin, Izu, Kermadec, Mariana, and Vanuatu) recorded higher growth rates (28-60 km3/km/m.y.)"; the arcs over younger, smaller plates grow slower.

### HaschkeGunther2003
- Reference: Haschke, M. and Günther, A. (2003). Balancing crustal thickening in arcs by tectonic vs. magmatic means. Geology 31(11):933-936.
- DOI or URL: 10.1130/G19945.1
- Cited as: "Haschke & Günther 2003" (the continental arcs' 35 km3/km/Myr, through Stern & Scholl 2010).
- Ledger ids: PLT-285, PLT-048, HNT-126
- Access: paywalled; abstract only (OpenAlex record, `abstracts_closed_sources_openalex.txt` in the library).
- Read so far (by the project): abstract, 2026-10-17: the Late Cretaceous to late Eocene arc of North Chile; "a minimum of 9 km of arc-normal shortening" producing "approximately 5.4 km of tec[tonic thickening]" (the record is cut there); the 35 km3/km/Myr and the tectonic to magmatic ratio are read through Stern & Scholl 2010, p. 12.

### KleyMonaldi1998
- Reference: Kley, J. and Monaldi, C. R. (1998). Tectonic shortening and crustal thickness in the Central Andes: How good is the correlation? Geology 26(8):723-726.
- DOI or URL: 10.1130/0091-7613(1998)026<0723:TSACTI>2.3.CO;2
- Cited as: "Kley & Monaldi 1998" (shortening falls short of the Andes' crust).
- Ledger ids: PLT-048
- Access: paywalled; the OpenAlex record holds only the landing page's author block (no abstract text).
- Read so far (by the project): title and record only, 2026-10-17; the shortening magnitudes "<250 km (e.g. Kley and Monaldi, 1998)" are as Eichelberger et al. 2015 cite them; the 70-80 % is Allmendinger et al. 1997's.

### Allmendinger1997
- Reference: Allmendinger, R. W., Jordan, T. E., Kay, S. M. and Isacks, B. L. (1997). The evolution of the Altiplano-Puna plateau of the Central Andes. Annual Review of Earth and Planetary Sciences 25:139-174. (Co-authors from memory.)
- DOI or URL: 10.1146/annurev.earth.25.1.139
- Cited as: "Allmendinger et al. 1997".
- Ledger ids: PLT-048
- Access: paywalled; abstract only (OpenAlex record).
- Read so far (by the project): abstract, 2026-10-17: the plateau "was uplifted primarily because of crustal thickening produced by horizontal shortening of a thermally softened lithosphere. Nonetheless, known shortening at the surface accounts for only 70-80% of the observed crustal thickening, suggesting that magmatic addition and other processes ... may contribute significantly".

### FrancisHawkesworth1994
- Reference: Francis, P. W. and Hawkesworth, C. J. (1994). Late Cenozoic rates of magmatic activity in the Central Andes and their relationships to continental crust formation and thickening. Journal of the Geological Society 151(5):845-854.
- DOI or URL: 10.1144/gsjgs.151.5.0845
- Cited as: "Francis & Hawkesworth 1994" (a lower continental-arc rate).
- Ledger ids: PLT-285
- Access: paywalled; abstract only (OpenAlex record).
- Read so far (by the project): abstract, 2026-10-17: "a consistent rate of effusive volcanism of 3 X 10-6 km3 a-1 km-1" in the Central Volcanic Zone; the overall magmatic addition (about 13 km3/km/Myr) as the research sub-agent read the rest of the abstract (the record is cut in the copy kept).

## Playability steering (STE)

### CharetteSmith2010
- Reference: Charette, M. A. and Smith, W. H. F. (2010). The volume of Earth's ocean. Oceanography 23(2):112-114.
- DOI or URL: 10.5670/oceanog.2010.51; https://tos.org/oceanography/article/the-volume-of-earths-ocean ; the institution's release: https://www.whoi.edu/press-room/news-release/whoi-study-calculates-volume-and-depth-of-the-worlds-oceans/
- Cited as: "Charette & Smith 2010" (Earth's ocean volume, against which a steered sea-level offset's water is reported).
- Ledger ids: STE-009
- Access: open (the journal page shows the abstract; the numbers are in the PDF).
- Read so far (by the project): 2026-10-05, the WHOI release only: "1.332 billion cubic kilometers", mean depth "3,682.2 meters". The paper itself was not opened.
- Read 2026-10-21: the paper itself (Table 2, as in the entry above), so STE-009 is `verified`.

### HaqSchutter2008
- Reference: Haq, B. U. and Schutter, S. R. (2008). A chronology of Paleozoic sea-level changes. Science 322:64-68.
- Cited as: "Haq & Schutter 2008" (the size of long-term sea-level change, a reference for the steering's sea rate).
- Ledger ids: STE-013
- Access: paywalled. Read so far (by the project): from memory.
- Read 2026-10-21: the abstract: "172 eustatic events ... varying in magnitude from a few tens of meters to approximately 125 meters"; no level against today.

### Miller2005
- Reference: Miller, K. G. et al. (2005). The Phanerozoic record of global sea-level change. Science 310:1293-1298.
- Cited as: "Miller et al. 2005" (as above).
- Ledger ids: STE-013
- Access: paywalled. Read so far (by the project): from memory.
- Read 2026-10-21 (ledger ISO-087): the abstract (Europe PMC, PMID 16311326): "Long-term sea level peaked at 100 +/- 50 meters during the Cretaceous".

### Berner2004
- Reference: Berner, R. A. (2004). The Phanerozoic Carbon Cycle: CO2 and O2. Oxford University Press.
- Cited as: "Berner 2004, GEOCARB f_G" (how far volcanic and metamorphic degassing varied, a reference for the steering's outgassing bound).
- Ledger ids: STE-014
- Access: book. Read so far (by the project): from memory.

## Ice sheets and mountain ice (CLM-150, CLM-203), added 2026-10-18

What an ice sheet is, for the temperate present's land-ice target (decisions log 2026-10-18: it counts ice sheets only). Fetched and read by the erosion-recalibration agent on 2026-10-18.

### Cogley2011
- Reference: Cogley, J. G., Hock, R., Rasmussen, L. A., Arendt, A. A., Bauder, A., Braithwaite, R. J., Jansson, P., Kaser, G., Möller, M., Nicholson, L. and Zemp, M. (2011). Glossary of Glacier Mass Balance and Related Terms. IHP-VII Technical Documents in Hydrology No. 86, IACS Contribution No. 2, UNESCO-IHP, Paris.
- DOI or URL: https://wgms.ch/downloads/Cogley_etal_2011.pdf
- Cited as: "Cogley et al. 2011, glossary, Ice sheet" (the 50,000 km² that makes an ice body an ice sheet).
- Ledger ids: CLM-203
- Access: open (UNESCO technical document, the WGMS copy).
- Library (the project's source library, not distributed): `Cogley2011/Cogley2011.pdf`, `Cogley2011.txt` (pdftotext), 2026-10-18.
- Read so far (by the project): full text, 2026-10-18. p. 53, "Ice sheet": "An ice body that covers an area of continental size, generally defined as covering 50 000 km2 or more. Currently there are only two ice sheets, the Greenland Ice Sheet and the Antarctic Ice Sheet." p. 52, "Ice cap": "A dome-shaped ice body with radial flow, largely obscuring the subsurface topography and generally defined as covering less than 50 000 km2 (see ice sheet)"; "Ice body": "Any continuous mass of ice". "Icefield": "A large ice body that covers mountainous terrain but is not thick enough to obscure all of the subsurface topography". "Mountain glacier": "A glacier that is confined by surrounding mountain terrain, also called an alpine glacier."

### NSIDC-IceSheetFacts
- Reference: National Snow and Ice Data Center. Ice Sheet Quick Facts (web page).
- DOI or URL: https://nsidc.org/learn/parts-cryosphere/ice-sheets/ice-sheet-quick-facts
- Cited as: "NSIDC, Ice Sheet Quick Facts" (the 50,000 km² definition; the two sheets' areas).
- Ledger ids: CLM-150, CLM-203
- Access: open (web page).
- Read so far (by the project): through the web fetch tool, 2026-10-18: "An ice sheet is a mass of glacial land ice extending more than 50,000 square kilometers"; "An ice cap covers less than 50,000 square kilometers ... and comprises several merged glaciers"; the Antarctic Ice Sheet "covers almost 14 million square kilometers", Greenland's stretches "across 1.7 million square kilometers"; "Now, Earth has just two ice sheets".

### NSIDC-GlacierFacts
- Reference: National Snow and Ice Data Center. Glacier Quick Facts (web page).
- DOI or URL: https://nsidc.org/learn/parts-cryosphere/glaciers/glacier-quick-facts
- Cited as: "NSIDC, Glacier Quick Facts" (Earth's land under ice today and at the last glacial maximum).
- Ledger ids: CLM-150; RLC-060, RLC-061 (2026-10-23: land under ice today and at the last glacial maximum, the realism checks)
- Access: open (web page).
- Read so far (by the project): through the web fetch tool, 2026-10-18: "Presently, 10 percent of land area on Earth is covered with glacial ice", glaciers, ice caps and the ice sheets of Greenland and Antarctica; "Glacierized areas cover over 15 million square kilometers"; at the maximum of the last ice age "glaciers covered about 32 percent of the total land area"; "There are over 200,000 glaciers distinct from the ice sheets".

### RGI
- Reference: RGI 7.0 Consortium. Randolph Glacier Inventory, a dataset of global glacier outlines, version 7.0 (rgidata.org; NSIDC data set NSIDC-0770, doi:10.5067/f6jmovy5navz).
- DOI or URL: https://rgidata.org/ ; https://rgidata.org/user_guide/06_dataset_summary.html
- Cited as: "the Randolph Glacier Inventory" (the glaciers outside the two ice sheets and their area).
- Ledger ids: CLM-203
- Access: open (data and user guide).
- Read so far (by the project): through the web fetch tool, 2026-10-18: "The RGI excludes the ice sheets of Greenland and Antarctica"; "It includes all glaciers larger than 0.01 km²"; NSIDC-0770, "a snapshot of the world's glaciers outside of ice sheets". The user guide's dataset summary: 215,547 glaciers in RGI 6.0 and 274,531 in 7.0; its terminus-type table's rows sum to about 705,740 km² (6.0) and 706,744 km² (7.0) (the sums are the agent's, from the table as the fetch tool returned it).

## What a conveyor's sinking region can carry (CLM-204, CLM-205), added 2026-10-18

The conveyor's sinking capacity and the tests' physical ceilings on the per-cell climate, added with the fix for the narrow-inlet conveyor (build log 2026-10-18).

### Walin1982
- Reference: Walin, G. (1982). On the relation between sea-surface heat flow and thermal circulation in the ocean. Tellus 34(2):187-195.
- DOI or URL: not looked up (from memory).
- Cited as: "Walin 1982; from memory" (a region's water-mass transformation is its surface heat loss over rho c_p times the cooling).
- Ledger ids: CLM-204
- Access: not checked.
- Library: not fetched.
- Read so far (by the project): from memory; the paper was not fetched or read. The form of the bound (water a region can sink = surface heat loss / (rho c_p dT)) is the agent's recollection of the water-mass transformation framework; the rate 8e-6 m/s is calibrated, not taken from the paper.

### WMO-Extremes2025
- Reference: World Meteorological Organization. Records of Weather and Climate Extremes (table of the WMO World Weather and Climate Extremes Archive, as of 31 July 2025).
- DOI or URL: https://wmo.int/sites/default/files/2025-07/Table_Records_25Jul2025.pdf (from https://wmo.int/site/world-weather-and-climate-extremes-archive)
- Cited as: "WMO's Records of Weather and Climate Extremes, as of 31 July 2025" (the wettest place on Earth in the long run).
- Ledger ids: CLM-205
- Access: open (agency PDF).
- Read so far (by the project): fetched and read 2026-10-18 (pdftotext of the table), the greatest average annual precipitation of each WMO region: Region II, Asia, 11.872 m (467.4 in.), 38 years, Mawsynram, India, 25 deg 18' N, 91 deg 35' E, 1,431 m; Region V, South-West Pacific, 11.64 m (460.0 in.), 30 years, Mt. Waialeale, Kauai, Hawaii, 1,569 m; Region I, Africa, 10.287 m (405.0 in.), 32 years, Debundscha, Cameroon, 9 m. The other regions' records are lower: Region III, South America, 8.99 m; Region IV, North America, 7 m (Henderson Lake, British Columbia); Region VI, Europe, 4.593 m (Crkvice).

## The fastest a column thickens and rises (PLT-287, PLT-288), added 2026-10-18

The rate limits of the plate step, added with the fix for the plate spikes (build log 2026-10-18 "Plate spikes"). Sundell et al. 2021 (above, under the orogenic collapse) gives the second thickening rate.

### GarzioneEtAl2006
- Reference: Garzione, C. N., Molnar, P., Libarkin, J. C. and MacFadden, B. J. (2006). Rapid late Miocene rise of the Bolivian Altiplano: evidence for removal of mantle lithosphere. Earth and Planetary Science Letters 241(3-4):543-556.
- DOI or URL: 10.1016/j.epsl.2005.11.026
- Cited as: "Garzione et al. 2006" (the Altiplano's crustal thickening rate, eq. 2, and its surface uplift rate).
- Ledger ids: PLT-287, PLT-288
- Access: paywalled at the publisher; a PDF copy was read.
- Library: not kept (the PDF and its pdftotext extraction were read in the task's scratch, which was deleted).
- Read so far (by the project): full text, 2026-10-18 (plate spikes agent). Abstract, p. 543: surface uplift "of ~2.5 to 3.5 km occurred between ~10.3 Ma and 6.8 ± 0.4 Ma"; "Only the removal of dense eclogitic lower crust and mantle lithosphere can generate a change of this magnitude and rapidity". Section 7.2, p. 551: "u H = W dH / dt" (eq. 2 gives dh/dt from it under Airy isostasy, with crust 2.8 and mantle 3.3 × 10³ kg/m³); "Using a maximum long term shortening rate over the past 10 Myr of u = 10 mm/yr [21] over a plateau width of W = 300 km and an initial crustal thickness of H = 50 km, the average rate of elevation change, dh / dt, should be ~3% of the crustal shortening rate, u, or 0.3 mm/yr"; "crustal thickening of 8 km over ~4 Myr could raise the surface ~1.2 km"; "A 2.5 to 3.5 km rise of the Altiplano in a period as short 1 to 4 Myr implies an average vertical speed between 3.5 and 0.6 mm/yr".

## The ice cover's balance on L2 (BIO-157), added 2026-10-18

Cited for the principle that a glacier's net balance summed over its area is zero in a steady state, which fixes its terminus and is what the accumulation-area ratio (AAR) and area-altitude balance ratio (AABR) methods of reconstructing an equilibrium line rest on. All from the agent's memory of the literature on 2026-10-18; none was fetched or read, and the references were not checked against Crossref.

### Meier1962
- Reference: Meier, M. F. and Post, A. S. (1962). Recent variations in mass net budgets of glaciers in western North America. International Association of Scientific Hydrology Publication 58 (Symposium of Obergurgl), 63-77.
- DOI or URL: not looked up.
- Cited as: "Meier & Post 1962" (bake/ice.rs module doc).
- Ledger ids: BIO-157
- Access: not looked up.
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): from memory (the accumulation-area ratio of glaciers in a steady state, about 0.6 to 0.7).

### Porter1975
- Reference: Porter, S. C. (1975). Equilibrium-line altitudes of late Quaternary glaciers in the Southern Alps, New Zealand. Quaternary Research 5(1):27-47.
- DOI or URL: not looked up.
- Cited as: "Porter 1975" (bake/ice.rs module doc).
- Ledger ids: BIO-157
- Access: not looked up.
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): from memory (the AAR method, a steady-state ratio of about 0.6 applied to reconstructed glaciers).

### Osmaston2005
- Reference: Osmaston, H. (2005). Estimates of glacier equilibrium line altitudes by the Area x Altitude, the Area x Altitude Balance Ratio and the Area x Altitude Balance Index methods and their validation. Quaternary International 138-139:22-31.
- DOI or URL: not looked up.
- Cited as: "Osmaston 2005" (bake/ice.rs module doc).
- Ledger ids: BIO-157
- Access: not looked up.
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): from memory (the AABR method: the ablation area's balance gradient times its area-weighted height below the line equals the accumulation area's above it, the balance ratio the ratio of the two gradients).

## Resampling lumps: the crust lost to the mantle (PLT-258, PLT-289 to PLT-292), added 2026-10-18

The figure the plate model's loss to the mantle is compared with, read again with the fix for the resampling's lumps (build log 2026-10-18 "Resampling lumps"). Cloos 1993 (above, under the force balance) is the context for PLT-289. PLT-290 to PLT-292 are bookkeeping and cite nothing.

### ScholvonHuene2009
- Reference: Scholl, D. W. and von Huene, R. (2009). Implications of estimated magmatic additions and recycling losses at the subduction zones of accretionary (non-collisional) and collisional (suturing) orogens. In: Cawood, P. A. and Kröner, A. (eds), Earth Accretionary Systems in Space and Time. Geological Society, London, Special Publications 318:105-125.
- DOI or URL: 10.1144/SP318.4
- Cited as: "Scholl & von Huene 2009" (through Stern & Scholl 2010: continental crust lost at collision zones, about 0.7 km3/yr).
- Ledger ids: PLT-258
- Access: paywalled.
- Library: not kept (the abstract was read from the Crossref record, 2026-10-18).
- Read so far (by the project): cited through Stern & Scholl 2010 since 2026-10-03. Abstract, 2026-10-18 (resampling lumps agent, Crossref record): at ocean-margin subduction zones (OMSZs) "Crustal recycling at Mesozoic and younger OMSZs is assessed at c. 60 km3 Ma−1 km−1 (c. 60% by subduction erosion). The corresponding global recycling rate is c. 2.5 AU"; at crust-suturing subduction zones (CSSZs) "the combined upper and lower plate losses of CIA crust via subduction erosion, sediment subduction, and lower plate crustal detachment and sinking are assessed far less securely at c. 115 km3 Ma−1 km−1. At a global length of 6000 km, recycling at CSSZs is accordingly c. 0.7 AU"; the 6000 km "presumes that in the past the global length of crustal collision zones averaged c. 6000 km, which is one-half that under way since the early Tertiary"; the collective loss is c. 3.2 AU against SZ additions of +2.7 AU, so net growth is about nil; 1 AU is 1 km3 of solid material a year. So the 0.7 km3/yr in PLT-258 is the paper's figure for collision zones alone and was cited correctly; it rests on the assumed past mean length (about 1.4 km3/yr at today's, the agent's arithmetic), and the loss at all subduction zones is 3.2 km3/yr, most of it by subduction erosion at ocean margins, which the plate model does not have.

## The surface stage's numerics: the diffusion solve, the flood (ERO-183, ERO-184), added 2026-10-18

The methods behind the P1-19 changes to the surface stage's solvers (build log 2026-10-18 "Surface numerics"). They set how the same equations are solved, not what the equations are. Barnes, Lehman & Mulla 2014 (above, [Barnes2014]) is the serial flood both floods return to on a tie-free surface.

### Barnes2016
- Reference: Barnes, R. (2016). Parallel Priority-Flood depression filling for trillion cell digital elevation models on desktops or clusters. Computers & Geosciences 96:56-68.
- DOI or URL: 10.1016/j.cageo.2016.07.001; open preprint https://arxiv.org/abs/1606.06204 (v2).
- Cited as: "Barnes 2016, Parallel Priority-Flood depression filling for trillion cell digital elevation models on desktops or clusters (Computers & Geosciences 96:56-68; arXiv 1606.06204)" (surface/pflood/parallel.rs module doc; pflood.rs "Parallelism").
- Ledger ids: ERO-184
- Access: open copy (arXiv preprint).
- Library: not kept (read through the web fetch tool, 2026-10-18).
- Read so far (by the project): in full, arXiv v2, 2026-10-18 (P1-19 surface agent), and implemented from the paper; the author's code was not opened. Read: the tiles are flooded on their own from their edges with watershed labels, recording for each pair of labels that meet the lowest level of the meeting (Algorithm 1, the per-tile pass, and Algorithm 2, the joining of tile edges); a Priority-Flood over the graph of labels gives each label its spill level; the tiles are then raised to it. The paper's own limit: "This paper considers only the most common case wherein ε = 0. Setting ε > 0 requires more complex methods than those described here", which is why the code adds its second phase (an exact Priority-Flood+ε per group of undecided cells, the project's own).

### Briggs2000
- Reference: Briggs, W. L., Henson, V. E. and McCormick, S. F. (2000). A Multigrid Tutorial, 2nd edition. SIAM, Philadelphia.
- DOI or URL: not looked up.
- Cited as: "Briggs, Henson & McCormick 2000, A Multigrid Tutorial, 2nd ed., SIAM: ch. 2-3" (surface/diffusion/multigrid.rs module doc and `OMEGA`).
- Ledger ids: ERO-183
- Access: book (paywalled).
- Library: not fetched.
- Read so far (by the project): from memory (the smoothing of high-frequency error by damped Jacobi, the coarse-grid correction of the smooth error, the V-cycle; the damping 2/3 for one dimension, 4/5 for the five-point Laplacian in two).

### Tatebe1993
- Reference: Tatebe, O. (1993). The multigrid preconditioned conjugate gradient method. In: Proceedings of the Sixth Copper Mountain Conference on Multigrid Methods, NASA Conference Publication 3224, pp. 621-634.
- DOI or URL: not found (NASA Technical Reports Server, not located).
- Cited as: "Tatebe 1993, 'The multigrid preconditioned conjugate gradient method', Proc. 6th Copper Mountain Conference on Multigrid Methods, NASA CP-3224, 621-634" (surface/diffusion/multigrid.rs module doc).
- Ledger ids: ERO-183
- Access: open (a NASA conference publication), copy not located.
- Library: not fetched.
- Read so far (by the project): from memory (a multigrid cycle as the conjugate gradients' preconditioner, which must then be symmetric and positive definite: equal pre- and post-smoothing). The reference itself (volume, pages 621-634, April 1993) was checked against the reference lists of later papers found by web search, 2026-10-18, not against the paper.

### Braess1995
- Reference: Braess, D. (1995). Towards algebraic multigrid for elliptic problems of second order. Computing 55(4):379-393.
- DOI or URL: not looked up.
- Cited as: "Braess 1995, 'Towards algebraic multigrid for elliptic problems of second order', Computing 55:379-393" (surface/diffusion/multigrid.rs module doc).
- Ledger ids: ERO-183, SCL-146 (2026-10-22: the cube's multigrid keeps ERO-183's constants on 2 x 2 face blocks)
- Access: paywalled (Springer).
- Library: not fetched.
- Read so far (by the project): from memory (piecewise-constant aggregation makes the Galerkin coarse operator too stiff for smooth error, about twice for the Laplacian, remedied by over-weighting the coarse correction). The project uses the equivalent halving of the coarse couplings (`SCALE` 0.5) and measured the effect itself (ERO-183).

## Ocean crust as drilled: the Penrose sequence (PLT-104, PLT-293, PLT-294, RCK-095, RCK-096), added 2026-10-19

The sources behind the ocean crust's own column (build log 2026-10-19 "Crust templates and the relief lift"): the thickness of new crust and of its layer 2, the share of lavas and sheeted dikes in two drilled holes, the ophiolite sequence the classes follow, and the sediment the census compares with.

### White1992
- Reference: White, R. S., McKenzie, D. and O'Nions, R. K. (1992). Oceanic crustal thickness from seismic measurements and rare earth element inversions. Journal of Geophysical Research: Solid Earth 97(B13):19683-19715.
- DOI or URL: 10.1029/92JB01749
- Cited as: "White et al. 1992" (plates/pools.rs oceanic_column).
- Ledger ids: PLT-104
- Access: paywalled (AGU).
- Library: not kept (the abstract was read from the publisher page and Crossref, 2026-10-19).
- Read so far (by the project): abstract, 2026-10-19 (crust templates agent): "the igneous section of oceanic crust averages 7.1±0.8 km thick away from anomalous regions such as fracture zones and hot-spots, with extremal bounds of 5.0-8.5 km".

### Christeson2019
- Reference: Christeson, G. L., Goff, J. A. and Reece, R. S. (2019). Synthesis of oceanic crustal structure from two-dimensional seismic profiles. Reviews of Geophysics 57:504-529.
- DOI or URL: 10.1029/2019RG000641
- Cited as: "Christeson et al. 2019" (plates/pools.rs oceanic_column; structure/columns.rs OCEAN_LAVA_SHARE).
- Ledger ids: PLT-104, RCK-095
- Access: open (AGU review).
- Library: not kept.
- Read so far (by the project): abstract, 2026-10-19 (crust templates agent): "Layer 2 has an average thickness of 1.84 km", "Total crustal thickness averages 6.15 km and is similar across all spreading rates". The layer 2 and layer 3 thicknesses with their deviations (1.84 ± 0.61 km, 4.31 ± 0.99 km) come from a secondary reading of section 4.1 and were not checked.

### Alt1996
- Reference: Alt, J. C., Laverne, C. and others (1996). The synthesis of the alteration of Hole 504B. Proceedings of the Ocean Drilling Program, Scientific Results 148, chapter 34 (the full title and author list not checked).
- DOI or URL: https://www-odp.tamu.edu/publications/148_SR/VOLUME/CHAPTERS/sr148_34.pdf
- Cited as: "Hole 504B" (structure/columns.rs OCEAN_LAVA_SHARE).
- Ledger ids: RCK-095
- Access: open (ODP publications).
- Library: not kept (fetched 2026-10-19).
- Read so far (by the project): the chapter, 2026-10-19 (crust templates agent): Hole 504B penetrated "274.5 m sediment, 571.5 m of volcanic rocks, a 209 m transition zone, and 1050 m into a sheeted dike complex" (1056 m elsewhere in the chapter), the dikes altered to greenschist assemblages at 300 to 400 degC.

### IODP309-312
- Reference: Expedition 309/312 Scientists (2006). Expedition 309/312 summary. Proceedings of the Integrated Ocean Drilling Program 309/312 (the editors and the page not checked).
- DOI or URL: https://iodp.tamu.edu/publications/exp309_312/103/103_7.htm
- Cited as: "Hole 1256D" (structure/columns.rs OCEAN_LAVA_SHARE).
- Ledger ids: RCK-095
- Access: open (IODP publications).
- Library: not kept (fetched 2026-10-19).
- Read so far (by the project): the summary's section on the drilled sequence, 2026-10-19 (crust templates agent): Hole 1256D, in crust formed at a superfast spreading rate, has 754 m of extrusive rocks, a 57 m transition zone and 346 m of sheeted dikes over gabbro reached at 1406.6 m below the sea floor.

### Penrose1972
- Reference: Anonymous (1972). Penrose Field Conference on ophiolites. Geotimes 17(12):24-25.
- DOI or URL: none (print).
- Cited as: "Penrose Field Conference 1972" (structure/columns.rs pool_class).
- Ledger ids: RCK-096
- Access: print only; copy not located.
- Library: not fetched.
- Read so far (by the project): through a secondary page that reproduces the definition (travelinggeologist.com, "Pilgrimage to the Semail ophiolite", 2015), 2026-10-19 (crust templates agent): from the bottom, an ultramafic complex, a gabbroic complex, a mafic sheeted dike complex and a mafic volcanic complex, with sediments and minor sodic felsic intrusive rocks (plagiogranite) as associated rocks. Not read in Geotimes, so the row is `from-memory`.

### Straume2019
- Reference: Straume, E. O., Gaina, C. and others (2019). GlobSed: updated total sediment thickness in the world's oceans. Geochemistry, Geophysics, Geosystems 20(4):1756-1772.
- DOI or URL: 10.1029/2018GC008115; author copy https://epic.awi.de/49326/
- Cited as: "GlobSed (Straume et al. 2019)" (strata.md, the census of the ocean crust's sediment by age).
- Ledger ids: HYP-044 (2026-10-22: the ocean's sediment against the model's conserved pools; strata.md's comparison rests on it too). PLT-440 (2026-10-22, the trench budget: the ocean's sediment the share acts on).
- Access: open author copy.
- Library: not kept (read 2026-10-19 from the author-accepted PDF, text-extracted; again 2026-10-22 by a research sub-agent, Table 3: "This study" about 3.37 x 10^8 km3 over about 3.63 x 10^8 km2, mean 927 m; margins 1.43 x 10^8 km3 over 4.69 x 10^7 km2; beyond 200 km of the continent-ocean boundary 1.13 x 10^8 km3 over 2.79 x 10^8 km2).
- Read so far (by the project): 2026-10-19 (crust templates agent): abstract, "sediment thickness increases with age of the oceanic crust, towards high southern and northern latitudes, and towards the equator"; Table 3, the mean over the whole grid 927 m, continental margins 3044 m, the deep ocean more than 200 km from the continent-ocean boundary 404 m; section 3, the ocean younger than 82 Myr within 72 degS to 72 degN "is characterized by an average sediment thickness of 267 m".

### WhiteMcKenzie1989
- Reference: White, R. and McKenzie, D. (1989). Magmatism at rift zones: the generation of volcanic continental margins and flood basalts. Journal of Geophysical Research 94(B6):7685-7729.
- DOI or URL: 10.1029/JB094iB06p07685
- Cited as: "White & McKenzie 1989" (the thick igneous crust of a volcanic margin's transition).
- Ledger ids: PLT-294
- Access: Wiley, not fetched.
- Library: not kept.
- Read so far (by the project): nothing. Cited from memory (crust templates agent, 2026-10-19): the ocean–continent transition of a volcanic margin is thick igneous crust (seaward-dipping lavas over a high-velocity lower crust of underplated gabbro), thicker than normal ocean crust and thinning to it seaward. No thickness is quoted because none was read. Franke 2013 (above, read in full on 2026-10-04) gives the share of volcanic margins; its accepted manuscript did not answer (HTTP 522) when it was checked again on 2026-10-19.

## The present-day nudge's cap and the Köppen floors' noise (CLM-151, CLM-206, BIO-072), added 2026-10-19

The temperate present's CO2 cap rests on the late Quaternary's glacial-to-interglacial range; the GameTest floors are compared with a share's confidence bound (build log 2026-10-19, "The present-day nudge's cap").

### Luthi2008
- Reference: Lüthi, D., Le Floch, M., Bereiter, B., Blunier, T., Barnola, J.-M., Siegenthaler, U., Raynaud, D., Jouzel, J., Fischer, H., Kawamura, K. and Stocker, T. F. (2008). High-resolution carbon dioxide concentration record 650,000-800,000 years before present. Nature 453(7193):379-382.
- DOI or URL: 10.1038/nature06949; abstract on OSTI, https://www.osti.gov/etdeweb/biblio/21286198; data PANGAEA 10.1594/PANGAEA.710901.
- Cited as: "Lüthi et al. 2008" (climate/params.rs, `present_co2_factor_max`: the late Quaternary's 172 to 300 ppm).
- Ledger ids: CLM-206, CLM-151
- Access: paywalled (Nature); the abstract is open.
- Library: not kept (read through the web fetch tool, 2026-10-19).
- Read so far (by the project): the abstract, 2026-10-19: "We report the lowest carbon dioxide concentration measured in an ice core, which extends the pre-industrial range of carbon dioxide concentrations during the late Quaternary by about 10 p.p.m.v. to 172-300 p.p.m.v."; the record reaches 800,000 yr before present over eight glacial cycles, and CO2 stayed below 180 p.p.m.v. for about 3,000 yr in Marine Isotope Stage 16. The paper itself was not read.

### Wilson1927
- Reference: Wilson, E. B. (1927). Probable inference, the law of succession, and statistical inference. Journal of the American Statistical Association 22(158):209-212.
- DOI or URL: 10.1080/01621459.1927.10502953 (from memory, not looked up).
- Cited as: "Wilson's score interval, Wilson 1927, JASA 22: 209" (neoforge gametest/ShareBound.java).
- Ledger ids: BIO-072
- Access: paywalled (Taylor & Francis).
- Library: not fetched.
- Read so far (by the project): from memory. The interval's form was checked by deriving it, not against the paper: the bounds are the roots in p of (p̂ - p)² = z² p(1 - p) / n, (p̂ + z²/2n ± z sqrt(p̂(1 - p̂)/n + z²/4n²)) / (1 + z²/n). The one-sided 99 % quantile of the standard normal, 2.326, is from memory (the tables give 2.3263).

## Carbonate on the sea floor, MVT hosts, impact deposits and placers (ERO-191 to ERO-195, ORE-270, ORE-271, IMP-142, IMP-143, DEP-214), added 2026-10-19

The carbonate and ore-deposit task (build log 2026-10-19): pelagic chalk above the CCD once the planktic calcifiers arrive, MVT in the carbonate horizons the columns hold, source, pathway and trap for the impact metals with densities from Earth's inventory, and placers kept off oceanic crust and the deep shelf. "Read by the research sub-agent" means a Sonnet sub-agent of the task fetched the source and quoted it on 2026-10-19; the quotes are in the ledger rows. Existing entries this task also read: Leach2010a (MVT hosts, ORE-271), Grieve1994 (abstract via OpenAlex, IMP-142, IMP-143) and Ames1998 (cited, not read, ORE-042, IMP-142).

### RidgwellZeebe2005
- Reference: Ridgwell, A. and Zeebe, R. E. (2005). The role of the global carbonate cycle in the regulation and evolution of the Earth system. Earth and Planetary Science Letters 234(3-4):299-315.
- DOI or URL: 10.1016/j.epsl.2005.03.006; open copy https://sseh.uchicago.edu/doc/Ridgwell_and_Zeebe_2005.pdf
- Cited as: "Ridgwell and Zeebe 2005, EPSL 234: 307" (deposits/clocks.rs; surface/carbonate.rs).
- Ledger ids: ORE-270, ERO-194
- Access: open copy.
- Library: not kept (fetched through the web fetch tool and the PDF's text layer extracted with pdftotext, 2026-10-19).
- Read so far (by the project): full text, 2026-10-19. P. 307: "it is not until the Mesozoic that a marked proliferation in coccolithophore and planktic foraminiferal diversity and abundance is observed ... Only then would a substantive deep-sea sedimentary carbonate sink have been possible"; "the mean area of platform carbonates during the Mesozoic and Cenozoic is much reduced compared to the Paleozoic"; the abstract's "Mid-Mesozoic Revolution"; the "Neritan" and "Cretan" ocean modes.

### Ridgwell2005
- Reference: Ridgwell, A. (2005). A Mid Mesozoic Revolution in the regulation of ocean chemistry. Marine Geology 217(3-4):339-357.
- DOI or URL: 10.1016/j.margeo.2004.10.036
- Cited as: "Ridgwell 2005" (surface/carbonate.rs, deposits/clocks.rs).
- Ledger ids: ORE-270
- Access: paywalled.
- Library: not fetched.
- Read so far (by the project): Crossref metadata and abstract only, 2026-10-19.

### MillimanDroxler1995
- Reference: Milliman, J. D. and Droxler, A. W. (1995). Calcium carbonate sedimentation in the global ocean: linkages between the neritic and deep-sea. Oceanography 8(3).
- DOI or URL: https://tos.org/oceanography/assets/docs/8-3_milliman.pdf
- Cited as: "Milliman and Droxler 1995, after Milliman 1993" (surface/carbonate.rs).
- Ledger ids: ERO-191
- Access: open access.
- Library: not kept.
- Read so far (by the project): the text layer, by the research sub-agent, 2026-10-19: the budget table "modified from Milliman (1993)", in 10^12 mol/yr: deep sea 290 x 10^6 km2, production 24.0, accumulation 11.0; slopes 32 x 10^6 km2, 5.0 and 4.0; reefs, banks, shelves and Halimeda mounds about 14.5 of accumulation; total accumulation 32; neritic and pelagic accumulation "roughly equal".

### Milliman1993
- Reference: Milliman, J. D. (1993). Production and accumulation of calcium carbonate in the ocean: budget of a nonsteady state. Global Biogeochemical Cycles 7(4):927-957.
- DOI or URL: 10.1029/93GB02524
- Cited as: "Milliman 1993".
- Ledger ids: ERO-191
- Access: open access (AGU).
- Library: not fetched.
- Read so far (by the project): the abstract through Crossref, by the research sub-agent, 2026-10-19: "about 5 billion tons (bt) per year, of which about 3 bt accumulate in sediments; the other 40% is dissolved. Nearly half of the carbonate sediment accumulates on reefs, banks, and tropical shelves."

### Dutkiewicz2015
- Reference: Dutkiewicz, A., Müller, R. D., O'Callaghan, S. and Jónasson, H. (2015). Census of seafloor sediments in the world's ocean. Geology 43(9):795-798.
- DOI or URL: 10.1130/G36883.1; supplement ftp://ftp.earthbyte.org/papers/Dutkiewicz_etal_seafloor_lithology/Dutkiewicz_etal_census_of_seafloor_sediments_Geology_suppmat_2015.pdf
- Cited as: "Dutkiewicz et al. 2015, supplement table DR1".
- Ledger ids: ERO-191
- Access: paywalled paper; open supplement.
- Library: not kept.
- Read so far (by the project): supplement table DR1, by the research sub-agent, 2026-10-19: per ocean, clay / calcareous ooze / fine-grained calcareous %: N Atlantic 27/36/27, S Atlantic 34/39/21, N Pacific 62/16/5, S Pacific 43/29/15, N Indian 40/33/13, S Indian 33/38/14, Southern 29/21/6 (the unweighted mean, about 30 % ooze and 14 % fine calcareous, is the sub-agent's arithmetic).

### Diesing2020
- Reference: Diesing, M. (2020). Deep-sea sediments of the global ocean. Earth System Science Data 12(4):3367-3381.
- DOI or URL: 10.5194/essd-12-3367-2020; https://essd.copernicus.org/articles/12/3367/2020/
- Cited as: "Diesing 2020: 37 %" (surface/carbonate.rs).
- Ledger ids: ERO-191
- Access: open access.
- Library: not kept.
- Read so far (by the project): the article, by the research sub-agent, 2026-10-19: of the seabed below 500 m, calcareous 36.8 %, clay 31.0 %, radiolarian ooze 14.2 %, lithogenous 9.5 %, diatom ooze 8.5 %. (Its preprint's 50.3 % is a share of samples, not of area.)

### HartmannMoosdorf2012
- Reference: Hartmann, J. and Moosdorf, N. (2012). The new global lithological map database GLiM: A representation of rock properties at the Earth surface. Geochemistry, Geophysics, Geosystems 13(12):Q12004.
- DOI or URL: 10.1029/2012GC004370
- Cited as: "Hartmann and Moosdorf 2012" (erosion-and-sediment.md, the land's carbonate share).
- Ledger ids: none (a comparison figure on the science page)
- Access: open access.
- Library: not fetched.
- Read so far (by the project): the abstract through Crossref, by the research sub-agent, 2026-10-19: "64% sediments (a third of which are carbonates) ... 10% are covered by water or ice". The pure carbonate class's 7.8 % is from memory.

### Goldscheider2020
- Reference: Goldscheider, N., Chen, Z., Auler, A. S., Bakalowicz, M., Broda, S., Drew, D., Hartmann, J., Jiang, G., Moosdorf, N., Stevanovic, Z. and Veni, G. (2020). Global distribution of carbonate rocks and karst water resources. Hydrogeology Journal 28:1661-1677.
- DOI or URL: 10.1007/s10040-020-02139-5
- Cited as: "Goldscheider et al. 2020" (erosion-and-sediment.md).
- Ledger ids: none (a comparison figure)
- Access: open access.
- Library: not kept.
- Read so far (by the project): the PDF, by the research sub-agent, 2026-10-19: carbonate rocks crop out on 15.2 % (20.3 million km2) of the ice-free land, 9.4 % continuous and 5.8 % discontinuous.

### James2022
- Reference: James, S., Chandran, S. R., Santosh, M., Pradeepkumar, A. P., Praseetha, B. S. and Sajinkumar, K. S. (2022). Meteorite impact craters as hotspots for mineral resources and energy fuels: A global review. Energy Geoscience 3(2):136-146. With the LPSC 2023 abstract 2841 on the same work.
- DOI or URL: 10.1016/j.engeos.2021.12.006; https://digitalcommons.mtu.edu/michigantech-p/16147/; https://www.hou.usra.edu/meetings/lpsc2023/pdf/2841.pdf
- Cited as: "James et al. 2022 / LPSC 2023" (impact deposit type notes).
- Ledger ids: IMP-143
- Access: open access.
- Library: not kept.
- Read so far (by the project): the abstract and the LPSC abstract, by the research sub-agent, 2026-10-19: "60 of the 208 terrestrial craters have been identified to host diverse resources"; "hydrocarbons (19), major minerals (10)". The author list is from memory beyond the first author.

### WikipediaImpactStructures2026
- Reference: Wikipedia contributors, "List of impact structures on Earth" (compiled after the Earth Impact Database, Planetary and Space Science Centre, University of New Brunswick), revision of 2026-10-19.
- DOI or URL: https://en.wikipedia.org/wiki/List_of_impact_structures_on_Earth (the wikitext, action=raw)
- Cited as: "the Wikipedia list of impact structures, from the Earth Impact Database" (impact deposit type notes).
- Ledger ids: IMP-143
- Access: open (web page; secondary to the database).
- Library: not kept (downloaded to a scratch folder, 2026-10-19).
- Read so far (by the project): the whole list's wikitext, 2026-10-19; the implementing agent summed pi/4 D^2 over the rows: 199 structures with a diameter, 135,890 km2; the 105 at least 8 km across, 134,649 km2; the 51 at least 20 km across, 127,810 km2.

### Clark2009
- Reference: Clark, P. U., Dyke, A. S., Shakun, J. D., Carlson, A. E., Clark, J., Wohlfarth, B., Mitrovica, J. X., Hostetler, S. W. and McCabe, A. M. (2009). The Last Glacial Maximum. Science 325(5941):710-714.
- DOI or URL: 10.1126/science.1172873
- Cited as: "Clark et al. 2009" (deposits.md, the drowned placer's depth).
- Ledger ids: DEP-214
- Access: paywalled; an author copy was read.
- Library: not kept.
- Read so far (by the project): the PDF, by the research sub-agent, 2026-10-19: "peak eustatic sea-level fall of ~130 m over the LGM", elsewhere "127.5 ± 7.5 m".

### ARDF-NM253
- Reference: U.S. Geological Survey, Alaska Resource Data File, record NM253 (the Nome offshore placer).
- DOI or URL: https://mrdata.usgs.gov/ardf/show-ardf.php?ardf_num=NM253
- Cited as: "the gold of the Bering shelf off Nome" (deposits/mod.rs, spawn_placer).
- Ledger ids: DEP-214
- Access: open (agency database).
- Library: not kept.
- Read so far (by the project): the record, by the research sub-agent, 2026-10-19: the placer "extends from Present Beach offshore for 2 miles to water depths of about 60 feet" (about 18 m).

### Warren2000
- Reference: Warren, J. (2000). Dolomite: occurrence, evolution and economically important associations. Earth-Science Reviews 52(1-3):1-81.
- DOI or URL: 10.1016/S0012-8252(00)00022-2
- Cited as: "Warren 2000" (erosion-and-sediment.md, dolostone).
- Ledger ids: none (a comparison on the science page)
- Access: paywalled; a copy was read.
- Library: not kept.
- Read so far (by the project): the PDF, by the research sub-agent, 2026-10-19: the settings of section 4 (4.1 syndepositional, sabkha-style, Abu Dhabi; 4.6 burial; 4.7 hydrothermal); "up to 50% of the world's carbonate reservoirs are dolomites".

### RudnickGao2003
- Reference: Rudnick, R. L. and Gao, S. (2003). Composition of the continental crust. In: Rudnick, R. L. (ed.), Treatise on Geochemistry vol. 3, The Crust, Elsevier, pp. 1-64.
- DOI or URL: 10.1016/B0-08-043751-6/03016-4 (from memory, not looked up)
- Cited as: "Rudnick & Gao 2003, from-memory" (impact_basin_u.json source note).
- Ledger ids: ORE-039; HYP-042 (2026-10-22)
- Access: paywalled (book chapter); an open copy at web.archive.org/web/2020/http://www.i2massociates.com/downloads/4.1RudnickGaoCrustcomposition.pdf.
- Library: not fetched.
- Read so far (by the project): from memory (upper continental crust about 2.7 ppm U); not read in this work. Read 2026-10-22 (the drowned continents task; a research sub-agent, the open copy, full text): "The lateral extent of the continents is marked by the break in slope on the continental shelf. Using this definition, 31% of continental area is submerged beneath the oceans (Figure 1; Cogley, 1984)".

### Hofmann1988
- Reference: Hofmann, A. W. (1988). Chemical differentiation of the Earth: the relationship between mantle, continental crust, and oceanic crust. Earth and Planetary Science Letters 90(3):297-314.
- DOI or URL: 10.1016/0012-821X(88)90132-X (from memory, not looked up)
- Cited as: "Hofmann 1988, from-memory" (impact_basin_u.json source note).
- Ledger ids: ORE-039
- Access: paywalled.
- Library: not fetched.
- Read so far (by the project): from memory (N-MORB about 0.07 ppm U); not read in this work.

### Wilber1990
- Reference: Wilber, R. J., Milliman, J. D. and Halley, R. B. (1990). Accumulation of bank-top sediment on the western slope of Great Bahama Bank: rapid progradation of a carbonate megabank. Geology 18(10):970-974.
- DOI or URL: 10.1130/0091-7613(1990)018<0970:AOBTSO>2.3.CO;2; abstract at https://pubs.usgs.gov/publication/70137851
- Cited as: "Wilber, Milliman & Halley 1990" (surface/carbonate.rs, `exported`).
- Ledger ids: ERO-195
- Access: paywalled; the abstract is open (USGS Publications Warehouse).
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-20: "large-scale export of bank-top sediment and rapid progradation of the slope during the Holocene"; a wedge "up to 90 m thick" of "predominantly aragonite mud derived from the bank since flooding of the platform 6-8 ka"; its volume "40%-80% that of Holocene sediment currently retained on the bank"; vertical accumulation up to 11 to 15 m/ka. Also cited for ERO-197 (the exported mud lies on the slope, off the bank top), 2026-10-20.

### DemaisonMoore1980
- Reference: Demaison, G. J. and Moore, G. T. (1980). Anoxic environments and oil source bed genesis. AAPG Bulletin 64(8):1179-1209 (from memory, not looked up).
- DOI or URL: not looked up.
- Cited as: "Demaison & Moore 1980, from-memory" (surface/facies.rs `classify`, via ERO-196).
- Ledger ids: ERO-196
- Access: paywalled (AAPG Datapages), from memory.
- Library: not fetched.
- Read so far (by the project): from memory, 2026-10-20 (the review fix that lets euxinic water veto chalk and periplatform limestone): anoxic bottom water, more than the sediment's make-up or the productivity alone, decides whether organic matter is kept, so organic-rich carbonate muds are source rocks as black shales are. Not read in this work.
## The sea floor's critical slope (YNG-022, YNG-023, AMP-074), added 2026-10-20

The sea-floor spikes task (build log 2026-10-20) measured the planet's deep sea floor against ETOPO 2022 (entry above, under the climate section: the 60 arc-second global grid and five 15 arc-second tiles were downloaded to the test VM and their slopes measured with `tools/review/seafloor/earth_stats.py`) and set the sea floor's critical slope from it. The papers below give what the grids cannot: trench walls and scarps from multibeam, abyssal hill sizes, seamount flank slopes, and how much of the ocean the grids only predict.

### StewartJamieson2018
- Reference: Stewart, H. A. and Jamieson, A. J. (2018). Habitat heterogeneity of hadal trenches: considerations and implications for future studies. Progress in Oceanography 161:47-65.
- DOI or URL: 10.1016/j.pocean.2018.01.007 (printed on the PDF's first page).
- Cited as: "Stewart & Jamieson 2018" (trench walls, scarps and ridges in multibeam bathymetry).
- Ledger ids: YNG-022, AMP-074
- Access: open copy (the PDF was fetched by a sub-agent, 2026-10-20).
- Library (the project's source library, not distributed): not filed; read as a text extraction (pdftotext) on the laptop, 2026-10-20.
- Read so far (by the project): full text, 2026-10-20 (sea-floor spikes task). Kermadec Trench: "The mean slope on the overidding west side is 7.7° ± 1.2 S.D. and on the underridding east side is 5.9° ± 1.8 S.D." (sic); the bend faults' "seafloor offsets frequently between 400 and 1200 m ... within 30 km of the trench axis, reducing to 150–400 m offset with increasing distance", and "Slope angles associated with the fault scarps are typically between 10° and 40° with slopes frequently in excess of 25°"; accretionary ridges "form slopes generally up to 15°, although locally can exceed 20°"; a pinnacle "approximately 750 m in height" in a sediment pond; "slope angles of up to 40° locally". Sediment ponds are nearly flat (under 4°). The hadal zone as a whole: its areas "combined total over 800,000 km−2 [sic] and represents the deepest 45% of the global ocean (Jamieson et al., 2010)"; hadal means "deeper than 6000 m (or 6500 m; Watling et al., 2013)".
- Reference check: correct (journal, volume, pages and DOI on the PDF's first page). Not the same work as StewartJamieson2019 (the five deeps).

### GoffSmithMarks2004
- Reference: Goff, J. A., Smith, W. H. F. and Marks, K. M. (2004). The contributions of abyssal hill morphology and noise to altimetric gravity fabric. Oceanography 17(1):24-37.
- DOI or URL: 10.5670/oceanog.2004.64 (OpenAlex, 2026-10-20); open copy on tos.org.
- Cited as: "Goff, Smith & Marks 2004" (abyssal hill heights and widths).
- Ledger ids: YNG-022
- Access: open (The Oceanography Society).
- Library (the project's source library, not distributed): not filed; read as a text extraction on the laptop, 2026-10-20.
- Read so far (by the project): full text, 2026-10-20. Table 2 (mean abyssal hill properties, after Goff et al. 1997): rms height Hb 56.1 m (Pacific-Cocos, 110 mm/yr) to 235.8 m (northern MAR, 28 mm/yr), characteristic width λbn 1.76 to 8.21 km, aspect ratio 2.6 to 6.6. Text: along the slow-spreading Mid-Atlantic Ridge "abyssal hills have characteristic heights of 150-400 m and widths of 4-14 km", along the fast East Pacific Rise "50-200 m height, 1-5 km width"; the altimetric gravity filter is "~18-20 km wide (full-wavelength)".
- Reference check: authors, journal, volume and issue on the PDF; pages 24-37 and the DOI from OpenAlex.

### Sandwell2022
- Reference: Sandwell, D. T., Goff, J. A., Gevorgian, J., Harper, H., Kim, S.-S., Yu, Y., Tozer, B., Wessel, P. and Smith, W. H. F. (2022). Improved bathymetric prediction using geological information: SYNBATH. Earth and Space Science 9(2):e2021EA002069.
- DOI or URL: 10.1029/2021EA002069 (printed on the PDF).
- Cited as: "Sandwell et al. 2022 (SYNBATH)".
- Ledger ids: YNG-022, AMP-074
- Access: gold open access (AGU).
- Library (the project's source library, not distributed): not filed; read as a text extraction on the laptop, 2026-10-20.
- Read so far (by the project): full text, 2026-10-20. "∼20% of the ocean floor has been surveyed by ships at a spatial resolution of 400 m or better. The remaining 80% has depth predicted from satellite altimeter-derived gravity measurements"; satellite gravity "can only resolve ½ wavelength of 6 km when the regional depth is 4 km"; seamounts are fitted with a Gaussian whose width is 2.4 times the height, "a maximum seamount slope of 0.25 independent of seamount height" (14°). This is why the ETOPO deep-sea slope statistics are a lower bound for the true sea floor.
- Reference check: correct (title, authors and DOI on the PDF; volume 9, issue 2 from OpenAlex).

### WesselSandwellKim2010
- Reference: Wessel, P., Sandwell, D. T. and Kim, S.-S. (2010). The global seamount census. Oceanography 23(1):24-33.
- DOI or URL: 10.5670/oceanog.2010.60 (OpenAlex, 2026-10-20).
- Cited as: "Wessel, Sandwell & Kim 2010" (abyssal hill and fracture zone amplitudes on slow-ridge flanks; mean sea-floor slope).
- Ledger ids: YNG-022
- Access: open (The Oceanography Society).
- Library (the project's source library, not distributed): not filed; read as a text extraction on the laptop, 2026-10-20.
- Read so far (by the project): full text, 2026-10-20. "Abyssal hills and fracture zones on the flanks of slow spreading ridges commonly have topographic amplitude greater than 1 km"; Figure 3: the mean slope of the sea floor from ship soundings "commonly exceeds 0.05 on the flanks of the seafloor spreading ridges, especially the ridges spreading at a rate of < 70 mm yr-1 (Becker and Sandwell, 2008)". Wessel 2001 (below) was read in its abstract only; this later census by the same author gives the slope figure.
- Reference check: authors, title, journal, volume, issue and first page on the PDF.

### BeckerSandwell2008
- Reference: Becker, J. J. and Sandwell, D. T. (2008). Global estimates of seafloor slope from single-beam ship soundings. Journal of Geophysical Research 113:C05028.
- DOI or URL: 10.1029/2006JC003879 (OpenAlex).
- Cited as: "Becker & Sandwell 2008" (satellite bathymetry is not a proxy for multibeam slopes).
- Ledger ids: YNG-022
- Access: paywalled (AGU); the abstract is open. The PDF fetch returned an empty file.
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract only, 2026-10-20 (OpenAlex): "the small-scale but ubiquitous abyssal hills and fracture zones dominate the global map of rough topography"; "satellite bathymetry is not a valid proxy for multibeam measurements, but edited single-beam sonar data are adequate to provide a global perspective on features with horizontal wavelengths as small as 2 km". Its global slope numbers were not read.
- Reference check: title, year, DOI, volume 113 and issue C5 from OpenAlex; the article number C05028 from memory.

### GoffJordan1988
- Reference: Goff, J. A. and Jordan, T. H. (1988). Stochastic modeling of seafloor morphology: inversion of Sea Beam data for second-order statistics. Journal of Geophysical Research 93(B11):13589-13608.
- DOI or URL: 10.1029/JB093iB11p13589 (OpenAlex, 2026-10-20).
- Cited as: "Goff & Jordan 1988" (the abyssal hill statistical model).
- Ledger ids: YNG-022 (context only: no value rests on it)
- Access: paywalled (AGU).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract only, 2026-10-20 (OpenAlex): "The seafloor is modeled as a stationary, zero-mean, Gaussian random field completely specified by its two-point covariance function", an anisotropic covariance with "five free parameters describing the amplitude, orientation, characteristic wave numbers, and Hausdorff (fractal) dimension", inverted from Sea Beam data "at scale lengths less than 100 km or so". The abstract gives no number the project uses; the hill sizes come from GoffSmithMarks2004, which tabulates the later inversions.
- Reference check: authors, year, journal, volume, issue and pages from OpenAlex.

### Goff1991
- Reference: Goff, J. A. (1991). A global and regional stochastic analysis of near-ridge abyssal hill morphology. Journal of Geophysical Research 96(B13):21713-21737.
- DOI or URL: 10.1029/91JB02275 (OpenAlex, 2026-10-20).
- Cited as: "Goff 1991" (abyssal hill rms height and width against spreading rate).
- Ledger ids: YNG-022 (context only)
- Access: paywalled (AGU).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract only, 2026-10-20 (OpenAlex): 64 Sea Beam swaths from the East Pacific Rise, the Mid-Atlantic Ridge and the Central Indian Ridge; "The slowest spreading rate subset (Mid-Atlantic data) exhibit the largest scales (rms height and characteristic width and length) of abyssal hills. These parameters generally decrease as spreading rate increases up to the fast spreading rate data (Pacific-Cocos) but increase going from fast to very fast (Pacific-Nazca)"; "steeper inward facing slopes in the medium and fast spreading rate data". Its own numbers were not read; GoffSmithMarks2004 Table 2 gives its Pacific-Cocos values (Hb 56.1 m, λbn 2.00 km) with the citation "Goff (1991)".
- Reference check: authors, year, journal, volume, issue and pages from OpenAlex.

### SmithJordan1988
- Reference: Smith, D. K. and Jordan, T. H. (1988). Seamount statistics in the Pacific Ocean. Journal of Geophysical Research 93(B4):2899-2918.
- DOI or URL: 10.1029/JB093iB04p02899 (OpenAlex, 2026-10-20).
- Cited as: "Smith & Jordan 1988" (seamount size distribution).
- Ledger ids: YNG-022 (context only)
- Access: paywalled (AGU).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract only, 2026-10-20 (OpenAlex): about 157,000 km of wide-beam profiles in eight regions of the eastern and southern Pacific; "the average number of seamounts with summit heights h ≥ H is well-approximated by the exponential frequency-size distribution: ν(H)=νo e^(−βH)", better than a power law. It says nothing of flank slopes. No value in the code rests on it.
- Reference check: authors, year, journal, volume, issue and pages from OpenAlex.

### Smith1988
- Reference: Smith, D. K. (1988). Shape analysis of Pacific seamounts. Earth and Planetary Science Letters 90(4):457-466.
- DOI or URL: 10.1016/0012-821X(88)90143-4 (OpenAlex, 2026-10-20; OpenAlex has no abstract).
- Cited as: "Smith 1988" (seamount height to basal radius).
- Ledger ids: YNG-022 (context only)
- Access: paywalled (Elsevier).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): not read. From memory only: a height to basal radius ratio of about 0.21 ± 0.08 for Pacific seamounts, which would put a cone's flank at about 12°; the project uses Sandwell2022's measured 0.25 maximum slope instead.
- Reference check: title, journal, volume, issue and pages from OpenAlex; the ratio from memory.

### Wessel2001
- Reference: Wessel, P. (2001). Global distribution of seamounts inferred from gridded Geosat/ERS-1 altimetry. Journal of Geophysical Research 106(B9):19431-19441.
- DOI or URL: 10.1029/2000JB000083 (OpenAlex, 2026-10-20).
- Cited as: "Wessel 2001" (seamounts as axisymmetric Gaussians; heights limited by plate age).
- Ledger ids: YNG-022 (context only)
- Access: paywalled (AGU).
- Library (the project's source library, not distributed): not fetched.
- Read so far (by the project): abstract only, 2026-10-20 (OpenAlex): "a forward modeling approach based on an axisymmetric Gaussian seamount"; "The global seamount distribution is long-tailed and resembles a power law distribution for seamounts in the height range 2-7 km"; the limiting height is "a simple function of plate age at the time of seamount emplacement". No flank slope in the abstract; the Gaussian's measured width is in Sandwell2022.
- Reference check: authors, year, journal, volume, issue and pages from OpenAlex.
## Contact aureoles, stocks and the LIP rule (RCK-053, RCK-091, DEF-078, DEF-079, PRV-006), added 2026-10-19

Annen2017 (RCK-053, RCK-091) and Bryan2008 (PRV-006) have their entries above; their REV-2 reading is recorded there.

### Johansson2018
- Reference: Johansson, L., Zahirovic, S. and Mueller, R. D. (2018). The interplay between the eruption and weathering of Large Igneous Provinces and the deep-time carbon cycle. Geophysical Research Letters 45(11): 5380-5389.
- DOI or URL: 10.1029/2017GL076691; the LIP polygons are EarthByte's published dataset (CC-BY 4.0).
- Ledger ids: none (an Earth comparison in provinces.md, not a constant in the code).
- Access: the polygons are open; the paper was not opened.
- Read in REV-2 (2026-10-19, a Sonnet lookup on the test machine): the polygons were downloaded and their area measured on the sphere: union 29.84 Mkm2, 5.85 % of Earth's surface; on land 9.80 Mkm2, 1.92 % of the surface and 6.68 % of the land (Natural Earth 50 m land, 146.7 Mkm2); polygons at least half on land 9.76 Mkm2 (1,025), the rest 20.09 Mkm2 (1,501); dropping seamount and ridge polygons leaves 23.6 Mkm2, 4.6 %. Single provinces: Ontong Java 1.88 Mkm2, Caribbean 0.90, Manihiki 0.78, Parana 0.67, Siberian 0.64, Ethiopian 0.57, Deccan 0.51. The polygons are mapped outcrop and inferred extent, so the area is a present-day footprint, not an erupted area. The scripts are on the test machine under /Volumes/Graphide/ban/deeptime-runs/rev2-lip/.

### Brenner2021
- Reference: from-memory: a 2021 study of fluid-borne heat transport in a carbonate-hosted contact aureole (Brenner et al.); the full reference was not pinned down.
- Ledger ids: RCK-091 (from-memory).
- Access: not fetched; search excerpts only.
- Read: nothing beyond search excerpts saying fluid flow carries heat further into a carbonate host. Do not cite it as read.

## The cordillera at Earth's typical continental arc (PLT-048, PLT-320), added 2026-10-20

The source of the cordillera's shortening rate since the crust-budget task (build log 2026-10-20 "Crust budget against Stern & Scholl"). Stern & Scholl 2010 (above) gives the budget the model's crust was booked against; van Hunen & Allen 2011 (above) was read in full for the collisions' end.

### HeuretLallemand2005
- Reference: Heuret, A. and Lallemand, S. (2005). Plate motions, slab dynamics and back-arc deformation. Physics of the Earth and Planetary Interiors 149(1-2):31-51.
- DOI or URL: 10.1016/j.pepi.2004.08.022
- Cited as: "Heuret & Lallemand 2005".
- Ledger ids: PLT-048 (the value since 2026-10-20), PLT-320, HNT-127, HYP-018
- Access: paywalled at the publisher; a full-text PDF was fetched 2026-10-20 (kept in the agent's scratch, not in the library).
- Read so far (by the project): full text, 2026-10-20. Section 3.2: "upper plate absolute motion appears to exert a dominant control on back-arc deformation, relative to the influence of trench absolute motion"; "back-arc deformation rates cannot exceed the Vup(n) value"; section 1: "among a set of 26 parameters tested by Jarrard (1986), Vup was found to be the best single predictor of strain regime". Table 1, read in its text layer and tabulated by the agent: 88 two-degree segments of continental upper plates (Andaman, Sumatra, Java, Ryukyu, Nankai, Japan, Kamchatka, Alaska, Cascadia, Mexico, Costa Rica, Colombia, Peru, North Chile, Juan Fernandez, South Chile, the Chile triple junction and Patagonia); 21 of them are class C3, "lithospheric scale back-arc thrusting", with a mean measured back-arc deformation rate Vd(n) of 7.1 mm/yr over the 14 that give one; the others shorten little or extend. The length-weighted mean shortening is about 1.7 mm/yr (21 x 7.1 / 88, the agent's arithmetic), and the mean of max(Vup(n), 0) over the 88 is 18.1 mm/yr (the shortening about 0.09 of it). Segments over oceanic upper plates (Izu-Bonin, Mariana, Tonga, Kermadec, the Aleutians, the Antilles, the Sandwich arc and others) were left out.

## The arc's basement and the arcs' fronts (PLT-330, PLT-331), added 2026-10-20

The sources of the ocean crust kept under crust made continental and of the arcs' fronts (build log 2026-10-20 "The arc's basement and the arcs' fronts"). Stern & Scholl 2010 (above, its entry records the passage read this time) says what Table 1's intra-oceanic arc rate counts; Bird 2003 (above) gives the transform test the fronts now take (PLT-311); Cloos 1993 (above, abstract) the buoyant arc crust that jams a trench.

### BoutelierChemendaBurg2003
- Reference: Boutelier, D., Chemenda, A. and Burg, J.-P. (2003). Subduction versus accretion of intra-oceanic volcanic arcs: insight from thermo-mechanical analogue experiments. Earth and Planetary Science Letters 212(1-2):31-45.
- DOI or URL: 10.1016/S0012-821X(03)00239-5
- Cited as: "Boutelier, Chemenda & Burg 2003 (from memory, not read)".
- Ledger ids: PLT-331 (the ocean crust under an arc going down with the slab at a collision while the arc's crust accretes)
- Access: paywalled; OpenAlex (2026-10-20) has the record but no abstract.
- Library (the project's source library, not distributed): none.
- Read so far (by the project): nothing. The claim taken from it, that an intra-oceanic arc reaching a trench loses its lower part to the slab while its upper crust is scraped off and accretes, is from memory and unchecked.

## Basins that outlast their sediment (HYD-110, PLT-129), added 2026-10-21

The sources named for a lake's sediment trap (HYD-110) and for the sediment-loaded ocean floor measured and taken back out (PLT-129's note), build log 2026-10-21 "Basins that outlast their sediment". None was fetched; every claim taken from them is from memory.

### Brune1953
- Reference: Brune, G. M. (1953). Trap efficiency of reservoirs. Transactions, American Geophysical Union 34(3):407-418 (from memory; DOI not checked).
- Cited as: "Brune 1953 (from memory, not read)".
- Ledger ids: HYD-110
- Access: not fetched.
- Read so far (by the project): nothing. The claim taken from it, that a reservoir whose capacity is large against its yearly inflow traps nearly all of the sediment that reaches it, is from memory and unchecked.

### Lehman1975
- Reference: Lehman, J. T. (1975). Reconstructing the rate of accumulation of lake sediment: the effect of sediment focusing. Quaternary Research 5(4):541-550 (from memory; not checked).
- Cited as: "Lehman 1975 (from memory, not read)".
- Ledger ids: HYD-110
- Access: not fetched.
- Read so far (by the project): nothing. The claim taken from it, that a lake's sediment accumulates fastest in its deep basin (sediment focusing), is from memory and unchecked.

### StecklerWatts1978
- Reference: Steckler, M. S. and Watts, A. B. (1978). Subsidence of the Atlantic-type continental margin off New York. Earth and Planetary Science Letters 41(1):1-13 (the title from memory; the volume and pages seen on the abstract's record).
- DOI or URL: 10.1016/0012-821X(78)90036-5; abstract at https://ora.ox.ac.uk/objects/uuid:038601f7-8449-4b15-a4fe-6b1db4050f08
- Cited as: "Steckler & Watts 1978" (abstract only).
- Ledger ids: PLT-129 (the 2026-10-21 note); lithosphere-thermal-state.md (stage 2, the margin's cooling time; no row)
- Access: paywalled (Elsevier and ADS refused, Unpaywall lists it closed); the abstract from Oxford's repository.
- Read so far (by the project): the abstract, 2026-10-23 (a sub-agent of the gap-4 stages 2-3 task): "The remaining subsidence has been interpreted in terms of a simple thermal model for the cooling lithosphere. Based on this model a thickness of the thermal lithosphere of 113-139 km is estimated". No time constant or β in the abstract; with McKenzie's τ = a²/(π²κ) scaled from 62.8 My at 125 km, 113 to 139 km is about 51 to 78 Myr (the sub-agent's arithmetic). The backstripping relation taken for PLT-129 is still from memory and unchecked.

## Sea level from the ocean's volume (ISO-085 to ISO-087, PLT-350 to PLT-353, CLM-210), added 2026-10-21

The sources of the planet's water, the floor's age-depth law and the check on the sea-level curve, build log 2026-10-21 "Sea level from the ocean's volume". Read by a research sub-agent on 2026-10-21 as each entry says; numbers seen only in search-engine summaries were not used.

### EakinsSharman2010
- Reference: Eakins, B. W. and Sharman, G. F. (2010). Volumes of the World's Oceans from ETOPO1. NOAA National Geophysical Data Center, Boulder, CO.
- DOI or URL: a mirror at https://climatechange.usf.edu/documents/Thermal%20Expansion/ocean_volumes.pdf (the NOAA page itself not read).
- Cited as: "Eakins & Sharman 2010" (the ocean volume beside Charette & Smith 2010).
- Ledger ids: ISO-085
- Access: open.
- Read so far (by the project): the mirror, 2026-10-21: "Total: 361,900,000 ... 1,335,000,000 ... 3688 ... 10,803" (km2, km3, mean and greatest depth m); 0.1 % on the area, 1 % on the volume.

### VaughanEtAl2013_AR5Ch4
- Reference: Vaughan, D. G., Comiso, J. C., et al. (2013). Observations: Cryosphere. In: Climate Change 2013: The Physical Science Basis. Contribution of Working Group I to the Fifth Assessment Report of the IPCC, chapter 4. Cambridge University Press.
- DOI or URL: https://www.ipcc.ch/site/assets/uploads/2018/02/WG1AR5_Chapter04_FINAL.pdf
- Cited as: "IPCC AR5 WG1 Ch. 4, Table 4.1".
- Ledger ids: ISO-085, CLM-099, CLM-210
- Access: open.
- Read so far (by the project): Table 4.1, 2026-10-21: sea-level equivalents Antarctic ice sheet 58.3 m, Greenland 7.36 m, glaciers 0.41 m (total about 66.1 m); note b: "ice density of 917 kg m-3, a seawater density of 1028 kg m-3, with seawater replacing ice currently below sea level".

### Stein1995
- Reference: Stein, C. A. (1995). Heat flow of the Earth. In: Ahrens, T. J. (ed.), Global Earth Physics: A Handbook of Physical Constants, AGU Reference Shelf 1, pp. 144-158 (pages from memory).
- DOI or URL: an open copy at https://unicamp.br/fea/ortega/extensao/Stein.pdf
- Cited as: "Stein 1995" (GDH1's ridge depth and decay constant, read in place of the paywalled Stein & Stein 1992).
- Ledger ids: PLT-352
- Access: open copy (OCR text, garbled but legible).
- Read so far (by the project): 2026-10-21: "q(t) = 510 t^-1/2 for ages less than or equal to 55 Ma and 48 + 96 exp(-0.0278 t) for ages greater than 55 Ma"; Table 2: GDH1 ridge depth 2600 m, plate thickness 95 km, basal temperature 1450 C, expansion coefficient 3.1e-5. The depth law's coefficients are not in what was read.

### Muller2008
- Reference: Müller, R. D., Sdrolias, M., Gaina, C., Steinberger, B. and Heine, C. (2008). Long-term sea-level fluctuations driven by ocean basin dynamics. Science 319(5868):1357-1362 (authors after the first and pages from memory).
- DOI or URL: 10.1126/science.1151540 (from memory); abstract via Europe PMC, PMID 18323446.
- Cited as: "Müller et al. 2008".
- Ledger ids: ISO-087
- Access: paywalled; abstract read.
- Read so far (by the project): the abstract, 2026-10-21: "we model a Late Cretaceous sea level that is 170 (85 to 270) meters higher than it is today"; published estimates "differ ... from approximately 40 to approximately 250 meters". The figure and the supplement were not opened.

### Haq1987
- Reference: Haq, B. U., Hardenbol, J. and Vail, P. R. (1987). Chronology of fluctuating sea levels since the Triassic. Science 235(4793):1156-1167 (from memory).
- Cited as: "Haq et al. 1987" (the 250-m Cretaceous high stand).
- Ledger ids: ISO-087
- Access: paywalled; abstract read (no numbers).
- Read so far (by the project): the abstract, 2026-10-21, gives no levels; the 250 m is read at second hand in Miller et al. 2011 (below).

### Miller2011
- Reference: Miller, K. G., Mountain, G. S., Wright, J. D. and Browning, J. V. (2011). A 180-million-year record of sea level and ice volume variations from continental margin and deep-sea isotopic records. Oceanography 24(2):40-53 (authors after the first and pages from memory).
- DOI or URL: https://tos.org/oceanography/assets/docs/24-2_miller.pdf
- Cited as: "Miller et al. 2011".
- Ledger ids: ISO-087
- Access: open.
- Read so far (by the project): 2026-10-21, the Fig. 5 caption: "in the range of 100-200 m for the peak at ca. 80 million years (versus the 250 m of Haq et al., 1987)"; and the hydro-isostatic share: a 120 m water addition gives "about two-thirds of this figure due to isostatic adjustment (i.e., 80 m ...)".

### Kominz1998
- Reference: Kominz, M. A., Miller, K. G. and Browning, J. V. (1998). Long-term and short-term global Cenozoic sea-level estimates. Geology 26(4):311-314 (authors after the first and pages from memory).
- DOI or URL: https://eps.rutgers.edu/images/stories/faculty/miller_kenneth_g/kgmpdf/98-Kominz.Geology.pdf
- Cited as: "Kominz et al. 1998" (the backstripping relation, the water-loaded share).
- Ledger ids: PLT-353
- Access: open copy.
- Read so far (by the project): 2026-10-21: the method "assumes an Airy isostatic response to loads", with "R2 = [R1 - TS] [(ρa - ρw)/ρa]" (a the asthenosphere, w water; the minus signs lost in the text extraction and restored).

### Parsons1982
- Reference: Parsons, B. (1982). Causes and consequences of the relation between area and age of the ocean floor. Journal of Geophysical Research 87(B1):289-302.
- DOI or URL: https://www.osti.gov/biblio/5545743 (the abstract)
- Cited as: "Parsons 1982" (the starting ocean's steady-state age–area distribution).
- Ledger ids: PLT-356
- Access: paywalled; abstract read.
- Read so far (by the project): the abstract on OSTI, 2026-10-21: the area of ocean floor with age "is approximately described by dA/dt = C0(1-(t/tm)), where C0 is the rate of crustal generation and tm the maximum age", explained by "a balance between crustal generation and consumption where consumption is uniformly distributed with age"; the consumption per unit age scatters about "a mean value equal to C0/tm (approx. 1.7 x 10^-8 km2/yr2)". The abstract gives neither C0 nor t_m.

## White patches, drowned continents and coastal high ground, added 2026-10-21

The sources named for the plateaus' summer warmth (CLM-220) and the Cryogenian's dates in plates.md's section of 2026-10-21. None was fetched in full.

### YaoZhang2015
- Reference: Yao, Y. and Zhang, B. (2015). The mass elevation effect of the Tibetan Plateau and its implications for Alpine treelines. International Journal of Climatology 35(8):1833-1846.
- DOI or URL: 10.1002/joc.4123
- Cited as: "Yao & Zhang 2015 (abstract)".
- Ledger ids: CLM-220, CLM-230
- Access: paywalled; the publisher refused the page to scripted access (HTTP 403, 2026-10-21). The abstract is on NASA's MODIS science-team publications page, https://modis.gsfc.nasa.gov/sci_team/pubs/abstract.php?id=19238 (read 2026-10-21).
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the abstract, verbatim (2026-10-21, the plateau-warmth task): "The monthly mean air temperature in the interior main plateau is approximately 2-7 degrees C higher than in the surrounding mountains and adjacent lowland areas"; "At an elevation of 4500m ... the monthly mean temperature differences between the plateau and the Sichuan Basin range from 3.58 degrees C (April) to 6.63 degrees C (June); the monthly temperature differences between the plateau and the Qilian Mts. range from 1.6 degrees C (July) to 7.7 degrees C (March)"; the plateau's air from MODIS land-surface temperature, the Sichuan Basin's free air from the lapse rate measured on Mt. Emei; the warmest month's 10 degC isotherm pushed up to 4,600-4,700 m, the interior's treeline 500-1,000 m higher than the eastern edge's. The monthly table was not read.

### Rooney2015
- Reference: Rooney, A. D., Strauss, J. V., Brandon, A. D. and Macdonald, F. A. (2015). A Cryogenian chronology: two long-lasting synchronous Neoproterozoic glaciations. Geology 43(5):459-462 (from memory; not checked).
- Cited as: "Rooney et al. 2015 (from memory, not read)".
- Ledger ids: none (plates.md, 2026-10-21: the Cryogenian's dates beside Ben's run)
- Access: not fetched.
- Read so far (by the project): nothing. The dates taken from it (Sturtian about 717 to 660 Ma, Marinoan ending about 635 Ma) are from memory and unchecked.

## The plate geometry: arcs, reversals and where subduction starts (PLT-370 to PLT-372; notes on PLT-042, PLT-056, PLT-062, PLT-064), added 2026-10-21

The sources of the plate-geometry task (build log 2026-10-21 "The plate geometry"; plates.md, the section of that name). Above and read again or for the first time in full this time: Stern 2004 (polarity reversal against transference), Stern & Scholl 2010 (the arcs' lengths by the crust they stand on), Bird 2003 (Table 1), Morra et al. 2013 and Mallard et al. 2016 (the plate sizes), Syracuse & Abers 2006 (abstract). Texts in the project's source library on the test VM (`/data/ban/deeptime-sources/<key>/`), not distributed.

### Crameri2020
- Reference: Crameri, F., Magni, V., Domeier, M., Shephard, G. E., Chotalia, K., Cooper, G., Eakin, C. M., Grima, A. G., Gürer, D., Király, Á., Mulyukova, E., Peters, K., Robert, B. and Thielmann, M. (2020). A transdisciplinary and community-driven database to unravel subduction zone initiation. Nature Communications 11:3750.
- DOI or URL: 10.1038/s41467-020-17522-9 (open access).
- Cited as: "Crameri et al. 2020".
- Ledger ids: PLT-062, PLT-064 (2026-10-21 notes), PLT-371 (the polarity reversals' rate), PLT-503 and PLT-505 (2026-10-23: initiation off the passive margin, and under compression), PLT-514 and PLT-516 (2026-10-23, the docking lane: Table 1's four polarity reversals against the model's confirmed reversals; "in the forearc region (a case of episodic subduction, wherein an arc is accreted to the overriding plate and subduction jumps to the front)" and "the many occurrences of collisional events along a pre-existing subduction trench that temporally and spatially coincide with SZI events", read again in the full text)
- Access: open.
- Library (the project's source library, not distributed): fetched 2026-10-21 with Supplementary Information and Supplementary Data 1. Files: `Crameri2020/crameri2020.pdf`, `crameri2020.txt`, `41467_2020_17522_MOESM1_ESM.pdf` to `MOESM3_ESM.xlsx`, `Supplementary_Data1_Analysis_sheet.txt` (the xlsx's analysis sheet as text).
- Read so far (by the project): full text and Supplementary Data 1's analysis sheet, 2026-10-21. Table 1: 13 subduction-zone initiation (SZI) events from 6 ± 1 to 104 ± 1 Ma, four of them polarity reversals (Aleutian 53 ± 2, Tonga-Kermadec 50 ± 5, New Hebrides-New Britain 10 ± 2, Philippine 9 ± 3 Ma); some candidates were "deemed too unclear to be included", so 13 is a floor. "All SZI events start less than 1500 km away from other active subduction zones and, in most cases, less than 500 km"; "At least nine of all thirteen events ... occur at or near a pre-existing convergent plate boundary"; "collision events along pre-existing subduction trenches are often precursors of SZI events"; "subduction breeds subduction"; "horizontally forced subduction zone initiation is dominant over the last 100 Ma". Supplementary Data 1: the subducting plates' mean ages at initiation, Aleutian 31.7, Izu-Bonin 40.7, Sunda-Java 44.8, Lesser Antilles 49, Tonga-Kermadec 54.4, South Sandwich 61.2, New Hebrides-New Britain 62.5, Philippine 66.2, Halmahera 117.4 Myr. Read again 2026-10-23 (V1 gap 2): "heterogeneous oceanic plate interiors are more favourable for SZI than ocean-continent boundaries, or more specifically, passive margins"; "we find no event that is purely plate-buoyancy driven or, in other words, spontaneous"; "only two (i.e., Anatolia and Oman) of the SZI database events formed" near a passive margin.

### GurnisHallLavier2004
- Reference: Gurnis, M., Hall, C. and Lavier, L. (2004). Evolving force balance during incipient subduction. Geochemistry, Geophysics, Geosystems 5(7):Q07001.
- DOI or URL: 10.1029/2003GC000681 (open, the author's site).
- Cited as: "Gurnis et al. 2004".
- Ledger ids: PLT-062, PLT-064 (2026-10-21 notes), PLT-270 (2026-10-23: `from-memory` to `verified`), PLT-505 (2026-10-23), PLT-516 to PLT-518 (2026-10-23, the docking lane: transference's force and critical convergence, read again in full: "reaching a maximum value dependent on plate thickness, but typically ranging from 2 to 3 × 10^12 N/m for cases that become self-sustaining", "Cumulative convergence, not the rate of convergence, is the dominant control", the critical convergence "only varies between 100 and 130 km (Table 3)", "initial elastic bending force ... between 10^12 and 10^13 N/m")
- Access: open.
- Library (the project's source library, not distributed): fetched 2026-10-21. Files: `Gurnis2004/GURggg04.pdf`, `GURggg04.txt`.
- Read so far (by the project): full text, 2026-10-21. Abstract: "Nearly half of all active subduction zones initiated during the Cenozoic"; "Subduction initiation is known to have occurred in a variety of tectonic settings: old fracture zones, transform faults, and extinct spreading centers and through polarity reversal behind active subduction zones" ("Izu-Bonin-Mariana (IBM) along a fracture zone, Tonga-Kermadec along an extinct subduction boundary, New Hebrides within a back arc, and Puysegur-Fiordland along a spreading center"); "The large initial bending resistance can be entirely eliminated in ridge models, explaining the propensity for new subduction zones to form through polarity reversals"; "A FZ must be underthrust by the older plate for 100–150 km before a transition from forced to self-sustaining" subduction. Table 1 "Cenozoic Subduction Initiation": eleven zones, their heritage fracture zone, transform, spreading centre, extinct subduction zone or polarity reversal (New Britain-San Cristobal, New Hebrides); none from old floor far from a plate boundary.

### HallGurnis2003
- Reference: Hall, C. E., Gurnis, M., Sdrolias, M., Lavier, L. L. and Müller, R. D. (2003). Catastrophic initiation of subduction following forced convergence across fracture zones. Earth and Planetary Science Letters 212(1-2):15-30.
- DOI or URL: 10.1016/S0012-821X(03)00242-5 (the record in CaltechAUTHORS).
- Cited as: "Hall et al. 2003".
- Ledger ids: PLT-062 (2026-10-21 note), PLT-518 (2026-10-23, the docking lane: the abstract's "approximately 100 km of convergence", read again)
- Access: paywalled; the abstract is open.
- Library (the project's source library, not distributed): the CaltechAUTHORS record with the abstract, 2026-10-21. File: `Hall2003/abstract.txt`.
- Read so far (by the project): abstract, 2026-10-21: "a fracture zone could be converted into a self-sustaining subduction zone after approximately 100 km of convergence"; "Using global plate reconstructions, we suggest that IBM nucleation was associated with a change in plate motion between 55 and 45 Ma".

### SornettePisarenko2003
- Reference: Sornette, D. and Pisarenko, V. (2003). Fractal plate tectonics. Geophysical Research Letters 30(3):1105 (venue and article number as Mallard et al. 2016 cite it; the issue and DOI 10.1029/2002GL015043 from memory).
- DOI or URL: arXiv cond-mat/0202320 (the preprint read).
- Cited as: "Sornette & Pisarenko 2003".
- Ledger ids: PLT-372
- Access: the preprint is open.
- Library (the project's source library, not distributed): fetched 2026-10-21 (arXiv preprint). Files: `SornettePisarenko2003/arxiv.pdf`, `arxiv.txt`.
- Read so far (by the project): the preprint's abstract and text, 2026-10-21: Bird's plate areas follow "the power law with exponent 0.25±0.05", "the most robust and parsimonious model for all plates, including the very largest plates, when taking into account the constraint that the plates areas must sum up to 4π steradians".

### EnglandEngdahlThatcher2004
- Reference: England, P., Engdahl, R. and Thatcher, W. (2004). Systematic variation in the depths of slabs beneath arc volcanoes. Geophysical Journal International 156(2):377-408 (from memory; the abstract was fetched without its record's metadata).
- DOI or URL: 10.1111/j.1365-246X.2003.02132.x (from memory).
- Cited as: "England et al. 2004 (abstract)".
- Ledger ids: PLT-042 (2026-10-21 note)
- Access: open at the publisher (from memory); only the abstract was fetched.
- Library (the project's source library, not distributed): the abstract from OpenAlex, 2026-10-21. File: `England2004/abstract_openalex.txt`.
- Read so far (by the project): abstract, 2026-10-21: the depths to the tops of the intermediate-depth seismicity under arc volcanoes are "constant, to within a few kilometres, within individual arc segments, but differ by tens of kilometres from one arc segment to another. The range in depths is from 65 km to 130 km"; they correlate inversely "with the descent speed of the subducting plate". A depth, not a distance from the trench.

## The plateau's warmth (CLM-225 to CLM-231, MOI-085), added 2026-10-21

The sources of the mass-elevation effect (docs/science/mass-elevation-effect.md). A sub-agent of the plateau-warmth task fetched and read them on 2026-10-21; the quotes below are from that reading. Its downloads were not kept. Yao & Zhang 2015 (the abstract) and Kalnay et al. 1996 (the R1 long-term means) are in the sections above.

### Han2018MEE
- Reference: Han, F. et al. (2018). Characterizing the Mass Elevation Effect across the Tibetan Plateau. Journal of Mountain Science 15(12):2651-2665 (the co-authors not recorded).
- DOI or URL: 10.1007/s11629-018-4978-2; read at https://jms.imde.ac.cn/en/article/pdf/preview/10.1007/s11629-018-4978-2.pdf
- Cited as: "Han et al. 2018".
- Ledger ids: CLM-230
- Access: open (the journal's own PDF).
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21). Fig. 4: "y = 0.0019x + 0.584 ... R² = 0.6775", x the mountain base elevation (MBE, m), y the annual mean MEE (degC), over 57 MBE plots; Table 3: Core I MBE 4,708 to 4,934 m, MEE 11.44 to 11.59 degC; Qaidam Basin 2,889 m, 5.65 degC. "MEE is obviously higher in the interior than in the exterior of the plateau even with similar mountain base elevation (MBE)"; "MEE may be low in areas with high mountain peaks and relatively low MBE, especially in some high isolated peaks" (the Pamir and Karakoram 4.13 to 6.37 degC). Its baseline: 17 lowland stations around the plateau extrapolated with NCEP-derived lapse rates; the plateau's air from a MODIS land-surface-temperature regression. The magnitudes are not used (CLM-230).

### Han2018bMEE
- Reference: Han, F. et al. (2018). Journal of Mountain Science 15(7):1510-1519 (the title and co-authors not recorded; a study of the Tibetan Plateau's monthly MEE).
- DOI or URL: 10.1007/s11629-018-4865-x; read at https://jms.imde.ac.cn/en/article/pdf/preview/10.1007/s11629-018-4865-x.pdf
- Cited as: "Han et al. 2018b".
- Ledger ids: CLM-230
- Access: open (the journal's own PDF).
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21). The interior's "monthly MEEs fall between 6.5647°C-13.8246°C, with the highest MEE mostly in August and the minimum mostly in January and February"; the periphery's "monthly MEEs are high in cold season and low in warm season ... from 1.5020°C (July) to 9.7750°C (December)". The monthly MEE-against-MBE R² 0.43 (May) to 0.77 (October); the monthly slopes are in a figure only and were not read.

### Han2020MEE
- Reference: Han, F. et al. (2020). What factors determine the mass elevation effect of the Tibetan Plateau? Journal of Mountain Science 17(11):2742-2749 (the co-authors not recorded).
- DOI or URL: 10.1007/s11629-020-6011-9; read at https://jms.imde.ac.cn/en/article/pdf/preview/10.1007/s11629-020-6011-9.pdf
- Cited as: "Han et al. 2020".
- Ledger ids: CLM-230
- Access: open (the journal's own PDF).
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21). Table 2's seven-factor model (R² 0.877); the explained variance: MBE 39.77 %, the distance to the massif's edges 23.02 %, continentality 14.48 %, the maximum elevation 11.41 %, the plot's area 2.92 %; "When only MBE is left, R2 can even amount to 0.677"; "AREA and NDVI can simply be neglected" (the area of each of the 57 plots, not of the whole massif).

### Han2012MBE
- Reference: Han, F. et al. (2012). Journal of Geographical Sciences 22(4):609-616 (the title and co-authors not recorded; mountain base elevation as the measure of the MEE, against the timberline).
- DOI or URL: 10.1007/s11442-012-0950-1; https://www.geogsci.com/EN/10.1007/s11442-012-0950-1
- Cited as: "Han et al. 2012 (abstract)".
- Ledger ids: none (background for CLM-230 on mass-elevation-effect.md)
- Access: the abstract on the journal's page (in its embedded JSON); the journal's PDF came back empty.
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the abstract (2026-10-21): "the more extensive the land mass and especially the higher the mountain base in the interior of land mass, the greater the mass elevation effect, this paper takes mountain base elevation (MBE) as the magnitude of MEE"; latitude, longitude and MBE explain 25.11, 29.43 and 45.46 % of the timberline's height. No regression coefficients.

### Zhao2014MEE
- Reference: Zhao, F. et al. (2014). Journal of Geographical Sciences 24(2):226-236 (the title and co-authors not recorded; the MEE's share in the Northern Hemisphere's treelines).
- DOI or URL: 10.1007/s11442-014-1084-4; https://www.geogsci.com/EN/10.1007/s11442-014-1084-4
- Cited as: "Zhao et al. 2014 (abstract)".
- Ledger ids: none (background for CLM-230 on mass-elevation-effect.md)
- Access: the abstract on the journal's page.
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the abstract (2026-10-21): 516 Northern Hemisphere treeline sites, R² 0.904; latitude 45.02 %, continentality 6.04 %, the MEE 48.94 % of the explained variance.

### Zhao2015Treeline
- Reference: Zhao, F. et al. (2015). Contribution of mass elevation effect to the altitudinal distribution of global treelines. Journal of Mountain Science 12(2):289-297 (the co-authors not recorded).
- DOI or URL: 10.1007/s11629-014-3223-x; read at https://jms.imde.ac.cn/en/article/pdf/preview/10.1007/s11629-014-3223-x.pdf
- Cited as: "Zhao et al. 2015".
- Ledger ids: none (background for CLM-230 on mass-elevation-effect.md)
- Access: open (the journal's own PDF).
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21). TL = a Lat + b K + c IMBE + d over 594 sites, R² 0.92; Table 1's global coefficients: constant 3,223, latitude -47.1, continentality 5.63, IMBE 0.80: "it is elevated by 80m for 100m increase in IMBE" (Northern Hemisphere 74 m, Southern 61 m). The sub-agent's own reading, not the paper's: at 0.6 to 0.65 K per 100 m that is about 0.5 K per 100 m of base height, about 2.5 times Han et al. 2018's slope.

### YaoZhang2013
- Reference: Yao, Y. and Zhang, B. (2013). PLoS ONE 8(7):e68750 (the title not recorded; station-based monthly temperature against altitude over the Tibetan Plateau, 109 stations, 2001-2007).
- DOI or URL: 10.1371/journal.pone.0068750; read at https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0068750
- Cited as: "Yao & Zhang 2013".
- Ledger ids: CLM-230
- Access: open access.
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21; its tables are images and were not read). At 4,500 m against the Sichuan Basin: "The difference is large (4-6°C) in warm months from April to October, and small (1-2°C) in other cold months"; against the Qilian Mountains "small (5.9-7.0°C) in warm months, and large (6.1-10.7°C) in other cold months"; "temperature lapse rates are steeper on isolated mountains near the sea than on extensive mountain ranges that provide their own heating". Its Sichuan free air is built from station regressions and MODIS points, not soundings.

### He2016Andes
- Reference: He, Y., Zhang, B. et al. (2016). The Mass Elevation Effect of the Central Andes and Its Implications for the Southern Hemisphere's Highest Treeline. Mountain Research and Development 36(2):213-221 (the title's end and the co-authors' initials from the publisher's URL and from memory; not checked).
- DOI or URL: 10.1659/MRD-JOURNAL-D-15-00027
- Cited as: "He et al. 2016 (abstract)".
- Ledger ids: CLM-230
- Access: BioOne refused scripted access (an Incapsula block); the abstract read through the OpenAlex API, the full text only as a WebFetch summary.
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the abstract (2026-10-21): "The mean air temperature in the Altiplano in summer is approximately 5 K higher than it is above the adjacent lowlands at the same mean elevation, averaging about 3700 m above sea level"; the effect "increased from the outer Andean east-facing slope to the interior of the Altiplano in summer". Its baseline is NCEP/NCAR R1's free air, as here (CLM-229). The summary of the full text (January -1.8 to 7.1 K, above 5 K at stations over 3,700 m) is not counted as read.

### PepinSeidel2005
- Reference: Pepin, N. and Seidel, D. (2005). A global comparison of surface and free-air temperatures at high elevations. Journal of Geophysical Research 110:D03104.
- DOI or URL: 10.1029/2004JD005047; read at https://arl.noaa.gov/wp_arl/wp-content/uploads/documents/JournalPDFs/PepinAndSeidel.JGR2005.pdf
- Cited as: "Pepin & Seidel 2005".
- Ledger ids: CLM-230
- Access: Wiley refused scripted access (403); NOAA ARL's copy read.
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21). Surface minus R1 free air at 1,084 sites above 500 m: "surface sites being relatively warm in comparison with the reanalysis in summer and cool in winter, especially in continental areas"; "Mountaintop sites nearly always show regime type 2 (a small summer/winter contrast)"; a weak rise of the difference with elevation (Asia r 0.42). Only 17 sites above 4,000 m, most of them Tibetan valley stations. No mean by terrain type and no plateau-against-summit contrast.

### Yanai1992
- Reference: Yanai, M., Li, C. and Song, Z. (1992). Seasonal heating of the Tibetan Plateau and its effects on the evolution of the Asian summer monsoon. Journal of the Meteorological Society of Japan 70(1B):319-351 (the title from memory).
- DOI or URL: https://www.jstage.jst.go.jp/article/jmsj1965/70/1B/70_1B_319/_pdf
- Cited as: "Yanai, Li & Song 1992".
- Ledger ids: CLM-230
- Access: open (J-STAGE).
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21): "when compared at the same level, the air immediately above the Plateau surface is warmer than the air of surrounding areas even in winter. An 'explosive' growth of this warm boundary layer air occurs in May, and warm anomalies of 8° to 9℃ occupy the whole tropospheric air column above the Plateau by August" (anomalies from the domain's nine-month mean along 32.5 N, not a surface-against-free-air difference); "During winter the Plateau is a heat sink, but it is surrounded by regions of more intense cooling".

### Flohn1968
- Reference: Flohn, H. (1968). Contributions to a meteorology of the Tibetan Highlands. Atmospheric Science Paper 130, Colorado State University (the title from memory).
- DOI or URL: https://mountainscholar.org/bitstream/handle/10217/225/0130_Bluebook.pdf
- Cited as: "Flohn 1968".
- Ledger ids: CLM-230
- Access: open (the university's repository).
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21): "At least during the warm season, the surface temperatures are definitely higher than the latitudinal average in the free atmosphere at similar heights". No number taken.

### Pan2021
- Reference: Pan et al. (2021). PLoS ONE 16(10):e0258549 (the authors and the title not recorded; the Tibetan Plateau's air against the Loess Plateau's and Jinji Mountain's).
- DOI or URL: 10.1371/journal.pone.0258549; read at https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8530289/
- Cited as: "Pan et al. 2021".
- Ledger ids: none (a cross-check on mass-elevation-effect.md)
- Access: open access (PubMed Central).
- Library (the project's source library, not distributed): none.
- Read so far (by the project): the full text (2026-10-21): at 5,000 m "the temperature of the Tibetan Plateau was approximately 0-3°C higher than that of the Loess Plateau and approximately 3-7°C higher than that of Jinji Mountain"; Tibet minus the Loess Plateau in July 2.62 degC.

### StationNormals2026
- Reference: the 1991-2020 monthly normals of the plateau stations (Nagqu, Tingri, Shiquanhe, Madoi and Lhasa: the China Meteorological Administration; El Alto and Oruro: Bolivia's SENAMHI; Puno and Juliaca: Peru's SENAMHI; Leadville, Climax, Alamosa and Pikes Peak: NOAA NCEI; Addis Ababa: the service its box names not recorded) and of the lowland stations used for the lapse (Chengdu, Wuhan, Shanghai, Delhi, Santa Cruz, Cuiaba), as the climate boxes of the towns' English Wikipedia articles give them.
- DOI or URL: the towns' articles on en.wikipedia.org, read as raw wikitext.
- Cited as: "the stations' 1991-2020 normals (Wikipedia's climate boxes)".
- Ledger ids: CLM-220, CLM-229, CLM-231
- Access: open; copies of the national services' tables, not the services' own.
- Library (the project's source library, not distributed): none.
- Read so far (by the project): a sub-agent read the boxes on 2026-10-21 (CLM-229's values). El Alto's box is labelled 1991-2020 but cites DWD's 1961-1990 source; Pikes Peak's normals are of low confidence and it was left out of the fit, as was Lhasa (a valley station). The latitudes of Shiquanhe, Tingri, Madoi, Puno, Juliaca and Oruro are from memory.

## The high ground's extent: post-orogenic collapse (PLT-390, PLT-391; leads for PLT-050 and PLT-242), added 2026-10-21

The sources of the high-ground task (build log 2026-10-21 "The high ground"; plates.md, "The high ground's extent: post-orogenic collapse (2026-10-21)"). Read by a research sub-agent of the task through publisher pages, OpenAlex and Crossref; no copy was saved to the library. Above and read again this time: Hillenbrand & Williams 2022 (abstract), Jolivet & Brun 2010 (full text), Peron-Pinvidic & Osmundsen 2020 (full text), Carbonell et al. 2000 (abstract). Titles marked "as the agent recalls it" were not read in the source and must be checked against the DOI before quoting.

### Puziewicz2025
- Reference: Puziewicz, J., Aulbach, S., Vanderhaeghe, O., Grégoire, M. and Ziobro-Mikrut, M. (2025). [Title not recorded by the sub-agent.] EGU General Assembly 2025, abstract EGU25-2372.
- DOI or URL: 10.5194/egusphere-egu25-2372
- Cited as: "Puziewicz et al. 2025".
- Ledger ids: PLT-390
- Access: open; the full abstract read (a conference abstract, not peer reviewed).
- Library: not fetched.
- Read so far (by the project): the full abstract, 2026-10-21: "the thickened orogenic root (Moho depth ca 55-60 km) flattened by lateral crustal flow and gravitational collapse"; "In the Bohemian Massif, the crust is still fairly thick (ca. 35 km)"; "Where flattening occurred, it produced a relatively flat Moho at ca 30-32 km depth [3]" (its [3] is Artemieva & Meissner 2012, Tectonophysics 530-531:18-49, not read); "the lithospheric and asthenospheric mantle underlying the orogen must have been exhumed by 20-30 km".

### Long2019
- Reference: Long, S. P. (2019). Geometry and magnitude of extension in the Basin and Range Province (39°N), Utah, Nevada, and California, USA: constraints from a province-scale cross section (title as the agent recalls it). Geological Society of America Bulletin 131(1-2):99-119.
- DOI or URL: 10.1130/B31974.1
- Cited as: "Long 2019".
- Ledger ids: PLT-390
- Access: paywalled; the abstract read.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-21: "230 ± 42 km of cumulative extension (46% ± 8%), and an average pre-extensional thickness of 54 ± 6 km"; the eastern domain "restores to a 60 ± 11 km thickness" and the western "66 ± 5 km"; "The eastern domain underwent a protracted, Late Cretaceous-Miocene transition to an extensional regime, while widespread extension in the western domain did not start until the Miocene". The modern 37 ± 1 km came only through a search tool's summary (not read); 54 / 1.46 = 37 km is the agent's arithmetic.

### Profeta2015
- Reference: Profeta, L. and others (2015). Quantifying crustal thickness over time in magmatic arcs (title as the agent recalls it). Scientific Reports 5:17786.
- DOI or URL: 10.1038/srep17786 (open access)
- Cited as: "Profeta et al. 2015".
- Ledger ids: none (a lead for PLT-050, the cordillera's height; plates.md, "The high ground's extent")
- Access: open; the full text and the supplementary tables read.
- Library: not fetched.
- Read so far (by the project): the full text and the supplement, 2026-10-21. The present crustal thickness (depth to Moho) of the arcs in the global fit: Central Andes 65 ± 0.7 km, Northern Andes 54.4 ± 4.7, Southern Andes 40 ± 5, Guatemala 44 ± 3, NE Aleutian 40 ± 3, Cascades 38 ± 1.9, Mexico 37.2 ± 9, Kamchatka 34.6 ± 4.8 (the continental arcs); Luzon 30.8, Honshu 30.2, Central America 28 ± 7, Sunda 27.8, Izu-Bonin 20.5, Tonga 20, Aleutian 18.9, Mariana 14.5, South Sandwich 11.8. Other geochemical correlations "break down where crustal thicknesses exceed 45 km"; delamination follows thickening "cyclically in long-lived arcs". The eight continental arcs' mean is 44.15 km and their median 40 km (the agent's arithmetic).

### Ingalls2016
- Reference: Ingalls, M., Rowley, D. B., Currie, B. S. and Colman, A. S. (2016). Large-scale subduction of continental crust implied by India-Asia mass-balance calculation (title as the agent recalls it). Nature Geoscience 9(11):848-853. Addendum: Nature Geoscience 10:393 (2017). Comment: van Hinsbergen, D. J. J., Lippert, P. C. and Huang, W. (2017). Unfeasible subduction? Nature Geoscience 10:878-879 (10.1038/s41561-017-0017-0). Reply: Rowley, D. B. and Ingalls, M. (2017). Nature Geoscience 10:879-880 (10.1038/s41561-017-0016-1).
- DOI or URL: 10.1038/ngeo2806
- Cited as: "Ingalls et al. 2016".
- Ledger ids: PLT-242 (2026-10-21: weighed against Stern & Scholl 2010 and measured as an arm, plates.md "The crust flux into collision fronts"; first a lead in "The high ground's extent")
- Access: paywalled; the abstract, the addendum's full text and the first pages of the comment and the reply read.
- Library: not fetched.
- Read so far (by the project): 2026-10-21. Abstract: "about 50% of the pre-collisional continental crustal mass cannot be accounted for in the crustal reservoir preserved at Earth's surface today", "a mass equivalent to about 15% of the total oceanic crustal subduction flux since 56 million years ago"; no uncertainty on the 50 % was read. The addendum changes no number and adds Replumaz et al. 2010 ("about 40 to 50% of Indian crust is missing", not read). The comment: the Himalaya holds accreted upper crust of "less than about 1,000 km" restored, so "at least 1,400 km of Greater Indian continental crust ... subducted ... at reconstructed subduction rates of up to 18 cm per year". The reply: tomography shows "a thick (about 19 km, approximately half the thickness of the incoming Indian crust) layer of eclogitized lower crust" subducted "during only the last 10 million years or less"; "The fraction of exhumed relative to subducted upper crustal eclogite is unknown".

### YakovlevClark2014
- Reference: Yakovlev, P. V. and Clark, M. K. (2014). Conservation and redistribution of crust during the Indo-Asian collision (title as the agent recalls it). Tectonics 33(6):1016-1027.
- DOI or URL: 10.1002/2013TC003469
- Cited as: "Yakovlev & Clark 2014".
- Ledger ids: PLT-242 (2026-10-21, with Ingalls et al. 2016)
- Access: paywalled; the abstract read (OpenAlex).
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-21: "a ~30% deficit between original and modern orogen volumes if the average global crustal thickness of 41 km is assumed prior to collision, even once eastward extrusion and crustal flow are considered"; "recycling of a magnitude that is greater than one half of the modern orogenic mass"; "this discrepancy can be eliminated with a 23-29 km thick crust within the orogen prior to collision along with a thick southern Tibet margin".

## The crust flux into collision fronts (PLT-242, PLT-400, PLT-401), added 2026-10-21

Read again for this task, entries above: Bird2003_Table3, SternScholl2010, Clift2009, Ingalls2016,
YakovlevClark2014.

### vanHinsbergen2022
- Reference: van Hinsbergen, D. J. J. (2022). Indian plate paleogeography, subduction and horizontal underthrusting below Tibet: paradoxes, controversies and opportunities. National Science Review 9(8):nwac074.
- DOI or URL: 10.1093/nsr/nwac074; full text https://pmc.ncbi.nlm.nih.gov/articles/PMC9385461/
- Cited as: "van Hinsbergen 2022".
- Ledger ids: PLT-401 (2026-10-21: the India-Asia collision's convergence and shortening, against the model's per-collision shortening)
- Access: open (PubMed Central).
- Library: not fetched.
- Read so far (by the project): the first 100,000 characters of the PMC text, 2026-10-21, through a fetch tool's excerpts: "after which >4000 km of India–Asia plate convergence occurred"; "3500 and 4500 km of post-60-Ma convergence at the longitude of the western and eastern Himalayan syntaxis"; "∼1000–1200 km of Cenozoic convergence was accommodated by shortening and extrusion in the overriding plate of Tibet"; the "Himalaya fold-thrust belt give somewhat smaller numbers, between 600 and 900 km"; in the model with an intra-oceanic arc, "large amounts (≤1000 km at the longitude of Bhutan) of continental subduction prior to horizontal underthrusting"; a fully continental Greater India needs "thousands of kilometers of continental subduction".

## The drowned continents (HYP-042 to HYP-046), added 2026-10-22

Read again for this task, entries above: Cogley1984 (abstract), RudnickGao2003, Laske2013,
Straume2019, Harris2014, SternScholl2010.

### Harrison1983
- Reference: Harrison, C. G. A., Miskell, K. J., Brass, G. W., Saltzman, E. S. and Sloan, J. L. (1983). Continental hypsography. Tectonics 2(4):357-377 (authors after the first, volume and pages from memory; not checked against Crossref here).
- DOI or URL: 10.1029/TC002i004p00357 (from memory); read in a Wayback copy of the eScholarship PDF.
- Cited as: "Harrison et al. 1983".
- Ledger ids: HYP-042
- Access: open copy (eScholarship).
- Library: not fetched.
- Read so far (by the project): 2026-10-22, a research sub-agent, the Wayback copy through pdftotext (Table 2 lost in the OCR, so the continents' mean heights are not read): "We have included within each continent the area of shelf out to a depth of 0.2 km. This is roughly the position of the shelf edge break and thus serves as a convenient boundary for the continental areas"; Table 5: continental area 170.895 x 10^6 km^2 (33.51 %), oceanic 339.113 x 10^6 km^2 (66.49 %).

### Conrad2013
- Reference: Conrad, C. P. (2013). The solid Earth's influence on sea level. Geological Society of America Bulletin 125(7-8):1027-1052 (the issue and pages from memory).
- DOI or URL: https://www.clintconrad.no/papers/Conrad_GSAB2013.pdf (the author's copy).
- Cited as: "Conrad 2013".
- Ledger ids: HYP-044
- Access: open (author's copy).
- Library: not fetched.
- Read so far (by the project): 2026-10-22, a research sub-agent, the "Marine Sedimentation" section: today's ocean sediment raises the sea by 166 m, 107 m at 65 Ma; the thicknesses isostatically compensated "following Sykes (1996)"; Pacific sediment about four times thinner than the Atlantic's or Indian's.

### Dutkiewicz2017
- Reference: Dutkiewicz, A., Müller, R. D., Wang, X., O'Callaghan, S., Cannon, J. and Wright, N. M. (2017). Predicting sediment thickness on vanished ocean crust since 200 Ma. Geochemistry, Geophysics, Geosystems 18 (authors after the first two and the title from memory).
- DOI or URL: https://www.earthbyte.org/webdav/ftp/Data_Collections/Dutkiewicz_etal_2017_G3/Dutkiewicz_Muller_et_al_sediment_thickness_G3_2017.pdf
- Cited as: "Dutkiewicz et al. 2017".
- Ledger ids: HYP-044
- Access: open (EarthByte copy).
- Library: not fetched.
- Read so far (by the project): 2026-10-22, a research sub-agent, the abstract and sections 4 and 5: the sea would be 210 m lower without the ocean's sediment (the Sykes method; about 150 m by their variant of Conrad's); the contribution rose from 80 m to 210 m since 120 Ma; modelled global mean thickness 365 m now.

### Syvitski2005
- Reference: Syvitski, J. P. M., Vörösmarty, C. J., Kettner, A. J. and Green, P. (2005). Impact of humans on the flux of terrestrial sediment to the global coastal ocean. Science 308(5720):376-380.
- DOI or URL: 10.1126/science.1109454
- Cited as: "Syvitski et al. 2005 (via Walling 2006)".
- Ledger ids: HYP-045
- Access: paywalled; not fetched.
- Library: not fetched.
- Read so far (by the project): at second hand only, 2026-10-22: Walling 2006 quotes the pre-human flux to the coast as 14 Gt a year and the modern as 12.6 Gt a year.

### Walling2006
- Reference: Walling, D. E. (2006). Human impact on land-ocean sediment transfer by the world's rivers. Geomorphology 79(3-4):192-216 (the title from memory).
- DOI or URL: a PDF at https://www.rmpcecologia.com/disciplinas/impactos/literatura/art57_hu2018.pdf
- Cited as: "Walling 2006".
- Ledger ids: HYP-045
- Access: an open copy.
- Library: not fetched.
- Read so far (by the project): 2026-10-22, a research sub-agent, for Syvitski et al. 2005's 14 and 12.6 Gt a year.

### vonHueneScholl1991
- Reference: von Huene, R. and Scholl, D. W. (1991). Observations at convergent margins concerning sediment subduction, subduction erosion, and the growth of continental crust. Reviews of Geophysics 29(3):279-316.
- DOI or URL: 10.1029/91RG00969 (from memory); abstract at https://www.usgs.gov/publications/observations-convergent-margins-concerning-sediment-subduction-subduction-erosion-and
- Cited as: "von Huene & Scholl 1991".
- Ledger ids: HYP-045, PLT-440, PLT-441 (2026-10-22, the trench budget)
- Access: paywalled; the abstract open (USGS).
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22 (a research sub-agent): sediment subduction at 43,500 km of margin about 1.5 km^3 a year of solid, 1.0 over the long term; about 20 % of the incoming sediment accreted at margins with small to medium prisms, about 70 % subducted at margins with large prisms; subduction erosion about 1.1 km^3 a year.
- Read in the trench-budget task (2026-10-22, the abstract on the USGS page through the Wayback Machine, by a research sub-agent; the page cuts the abstract off at "subcrustally subducted materia"): "type 2 margins (~19,000 km in global length)", where "effectively all incoming sediment is subducted"; at "small-to-medium-sized prisms ... (~16,300 km)" "approximately 20% of the incoming sediment is skimmed off ... and frontally accreted. The remaining 80% subducts"; at "margins bordered by large prisms (~8,200 km)" "roughly 70% of the incoming trench floor section is subducted"; "the contemporary rate of solid-volume sediment subduction at convergent ocean margins (~43,500 km) is calculated to be 1.5 km3/yr", long term 1.0; subduction erosion "long-term rates ... range from 25 to 50 km3/m.y. per kilometer of margin. A conservative long-term rate ... is 30 km3/km/m.y.", about 0.6 km3/yr on type 2 margins and 1.1 with type 1 sectors.

### SimpsonSimpson2018
- Reference: Simpson and Simpson (2018), arXiv:1810.08658 (the authors' initials and the title not recorded by the sub-agent).
- DOI or URL: https://arxiv.org/pdf/1810.08658
- Cited as: "Simpson & Simpson 2018".
- Ledger ids: HYP-046
- Access: open (arXiv).
- Library: not fetched.
- Read so far (by the project): 2026-10-22, a research sub-agent, full text: "Reported mean land elevations by date are 700m (1895), 840m (1921), 871m (1931), and 797m (2012)"; the land fraction "is 0.292 and 0.2905 for the 1931 and 2012 estimates".

## Volcano craters, cone gullies and steep ground (VOL-026, VOL-090 onwards, AMP-080 to AMP-084), added 2026-10-22

Read again for this task, entries above: Grosse2014 (p. 8, the craters: VOL-026, VOL-091).
Texts in the project's source library on the test VM (`/data/ban/deeptime-sources/<key>/`), not
distributed.

### PikeClow1981
- Reference: Pike, R. J. and Clow, G. D. (1981). Revised classification of terrestrial volcanoes and catalog of topographic dimensions, with new results on edifice volume. U.S. Geological Survey Open-File Report 81-1038.
- DOI or URL: 10.3133/ofr811038 (the USGS series pattern; not resolved here).
- Cited as: "Pike & Clow 1981"
- Ledger ids: VOL-026, VOL-090, VOL-091
- Access: open (USGS).
- Library: fetched (PDF with a text layer, a typed report; the text has OCR-like slips such as "BOO." for 800). Text file: PikeClow1981.txt.
- Read so far (by the project): 2026-10-22, the volcano-realism research and this task: table 2, "Topographic dimensions of 697 terrestrial volcanoes, by class": DIAM the rim crest's average diameter, DEPTH the crater floor's average depth below the rim crest, HEIGHT the rim crest's average height above the pre-volcano ground, WIDTH the flank's width from rim crest to the edifice's edge. A script over classes SC and SA (stratocones with a summit crater only; 103 rows the text kept whole): crater depth median 150 m (10th to 90th percentile 50 to 345 m), diameter median 500 m (250 to 1,375 m), height median 1,630 m; depth over height median 0.09, 90th percentile 0.26, largest 0.37; depth over diameter median 0.26.

### OHara2024
- Reference: O'Hara, D., Goren, L., van Wees, R. M. J., Campforts, B., Grosse, P., Lahitte, P., Kereszturi, G. and Kervyn, M. (2024). Time-varying drainage basin development and erosion on volcanic edifices. Earth Surface Dynamics 12:709-726.
- DOI or URL: 10.5194/esurf-12-709-2024
- Cited as: "O'Hara et al. 2024"
- Ledger ids: VOL-068, VOL-100 to VOL-105 (the cone gullies)
- Access: open access (CC BY 4.0).
- Library: fetched (PDF, text layer). Text file: OHara2024.txt.
- Read so far (by the project): 2026-10-22, full text. Stratovolcanoes in Indonesia, Papua New Guinea, New Zealand and Guatemala on the 30 m SRTM model, ages from the present to the early Pleistocene, lifespans of activity 100 to 1,000 kyr. §4.1: at about 10 % eroded volume "narrow (∼20° normalized width angle) and uniform ... drainages form that extend from the summit region to the lower flanks (i.e., 'parasol ribbing'; Ollier, 1988), giving a high drainage density (∼1 km−1)"; at 30 to 40 % eroded volume, on 10 to 100 kyr, summit basins widen to 40 to 60°, the network becomes dendritic, drainage density about 0.5 km−1; over more than 1 Myr the edifice loses its single-peaked cone (citing Karátson et al. 2012, Dibacto et al. 2020, Ollier 1988). §4.3: "the number of basins that reach the summit on most edifices is greater than 10" (summit basins: those in the upper 30 % of the height), with a weak trend in time; the ratio of the edifice's effective radius to the spacing of summit-basin outlets along its perimeter is about 1 to 3; younger edifices have more basins at every distance from the peak. Channels counted from 1.0 km² of drainage area. Again 2026-10-22 for the cone gullies (VOL-101, VOL-103), Fig. 4 on the page image (p. 715): panel h, "Normalized Eroded Volume" (%) against the age of activity (YPB, log axis), the fit "5.36log(x)+-1.56 (R² = 0.50) (p-val = 0.006)", data about 10 to 37 % over about 3 kyr to 300 kyr; panel a, edifice relief, "-0.56log(x)+4.21" (R² 0.77). §2: an edifice's age is "the log-mean between the most recent eruption and the oldest date of activity" (linear means give similar results, Fig. S5).

### SwansonWright1981
- Reference: Swanson, D. A. and Wright, T. L. (1981). Guide to geologic field trip between Lewiston, Idaho, and Kimberly, Oregon, emphasizing the Columbia River Basalt Group. In Johnston, D. A. and Donnelly-Nolan, J. (eds.), Guides to some volcanic terranes in Washington, Idaho, Oregon, and northern California. U.S. Geological Survey Circular 838.
- DOI or URL: https://npshistory.com/publications/geology/circ/838/sec1b.htm (a transcription; the contents page names the authors and the circular's editors and year)
- Cited as: "USGS Circular 838", "Swanson & Wright 1981"
- Ledger ids: AMP-084 (and the cliff bands of AMP-082)
- Access: open (USGS; the transcription is free).
- Library: not fetched into the library; read on the web page.
- Read so far (by the project): 2026-10-22, the D3 agent, section "Physical characteristics of flows", through a page fetch whose summary quotes at most about 125 characters: flows of the Grande Ronde, Wanapum and Saddle Mountains Basalts "range from a few tens of centimeters to more than 100 m thick, averaging 30-40 m"; "Even the thinner flows generally show evidence of having ponded"; the entablature "about 70 percent of the thickness of a flow". The page does not describe cliffs, benches or terraces (the words do not appear).

### SchmidtMontgomery1995
- Reference: Schmidt, K. M. and Montgomery, D. R. (1995). Limits to relief. Science 270(5236):617-620.
- DOI or URL: 10.1126/science.270.5236.617
- Cited as: "Schmidt & Montgomery 1995"
- Ledger ids: AMP-080 (the direction only)
- Access: paywalled (the publisher's page refused the fetch, HTTP 403).
- Library: not fetched.
- Read so far (by the project): 2026-10-22, the D3 agent, the abstract through OpenAlex: slope profiles where bedrock landsliding is widespread, against a model of the largest stable hillslopes, show that "mountain-scale material strength can limit topographic relief"; strengths back-calculated in the northern Cascades and the Santa Cruz Mountains are far below laboratory values for intact rock and match the weakest members of each formation; against "the conventional assumption that relief is incision-limited".

### DiBiase2018
- Reference: DiBiase, R. A., Rossi, M. W. and Neely, A. B. (2018). Fracture density and grain size controls on the relief structure of bedrock landscapes. Geology 46(5):399-402. From memory: not checked.
- DOI or URL: 10.1130/G40006.1 (from memory, not resolved)
- Cited as: "DiBiase et al. 2018"
- Ledger ids: AMP-080 (a simplification's note, not a value)
- Access: not checked.
- Library: not fetched.
- Read so far (by the project): not read; cited from memory for the idea that the fracture density of the rock, not only its strength, sets how rocky and steep bedrock landscapes stand.

## Lava flows (VOL-140 to VOL-163), added 2026-10-22

Read for review fix 8 (the lava flows' paint) by a research sub-agent on 2026-10-22, through a page
fetch whose summary quotes at most about 125 characters. "Search summary" means the fact was seen
only in a search result, not on the page. Publisher pages of the Royal Society, GSA, Wiley, Springer,
ResearchGate and the Smithsonian GVP refused scripted access, and USGS PDFs came back unreadable.
Nothing was saved to the library.

### Riker2009
- Reference: Riker, J. M., Cashman, K. V., Kauahikaua, J. P. and Montierth, C. M. (2009). The length of channelized lava flows: insight from the 1859 eruption of Mauna Loa Volcano, Hawai'i. Journal of Volcanology and Geothermal Research 183(3-4):139-156 (the pages from a search summary).
- DOI or URL: 10.1016/j.jvolgeores.2009.03.002; abstract at https://pubs.usgs.gov/publication/70198250
- Cited as: "Riker et al. 2009".
- Ledger ids: VOL-140
- Access: paywalled; the abstract open (USGS).
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22: "paired 'a'ā and pāhoehoe flows of exceptional length (51 km)". The abstract does not call it the longest flow; that wording was only in a search summary.

### OregonState-VolcanoWorld
- Reference: Oregon State University, Volcano World pages "How far can lava flow?" (https://volcano.oregonstate.edu/faq/how-far-can-lava-flow) and on aa (https://volcano.oregonstate.edu/a039a).
- DOI or URL: as above.
- Cited as: "Oregon State Volcano World".
- Ledger ids: VOL-140, VOL-158
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: the 1859 Mauna Loa flow "about 51 kilometers long from the vent to the ocean"; aa "range in thickness from 1-10 meters" (proximal 1 to 3 m, distal up to 10 m, clinker usually 1 to 2 m); pahoehoe and aa separate at about 6 m³/s, the 1859 aa erupted at 208 m³/s; "Each can be found at any distance from the vent although the names imply otherwise."

### Walker1973
- Reference: Walker, G. P. L. (1973). Lengths of lava flows. Philosophical Transactions of the Royal Society of London A 274(1238):107-118 (Crossref: "Mount Etna and the 1971 eruption - Lengths of lava flows").
- DOI or URL: 10.1098/rsta.1973.0030
- Cited as: "Walker 1973".
- Ledger ids: VOL-143, VOL-144
- Access: paywalled (refused); the abstract through Crossref and OpenAlex.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22: the rate of effusion is "the principal factor influencing the length of a lava flow"; basaltic extrusions have a median length "3.2 times that of high-viscosity andesite, trachyte and rhyolite ones". A search summary adds that Walker's effusion rate is a mean output rate (volume over duration).

### Malin1980
- Reference: Malin, M. C. (1980). Lengths of Hawaiian lava flows. Geology 8(7):306-308.
- DOI or URL: 10.1130/0091-7613(1980)8<306:LOHLF>2.0.CO;2
- Cited as: "Malin 1980".
- Ledger ids: VOL-143
- Access: paywalled; the abstract through OpenAlex.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22: 87 historic Hawaiian flows give "little support for a direct relationship between flow length and effusion rate"; length goes better with erupted volume; "no single factor appears most important"; tube-fed flows travel farther. Pinkerton and Wilson 1994 (Bull. Volcanol.), seen only in a search summary, find the data agree with Walker once short and tube-fed flows are set aside.

### USGS-MaunaLoaFAQ
- Reference: U.S. Geological Survey, Hawaiian Volcano Observatory, "Frequently asked questions about Mauna Loa volcano".
- DOI or URL: https://www.usgs.gov/volcanoes/mauna-loa/frequently-asked-questions-about-mauna-loa-volcano
- Cited as: "USGS HVO, Mauna Loa FAQ".
- Ledger ids: VOL-145, VOL-152
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: "Since 1843, Mauna Loa has erupted 34 times" (2022 the latest; 1984 to 2022 its longest quiet spell); "As much as 35 percent to 40 percent (695 to 795 square miles)" of the volcano covered in the past 1,000 years, 14 percent in the past 150 years. The USGS geology page gives about 90 percent within the past 4,000 years; a share of 98 percent under 10,000 years (Lockwood and Lipman 1987) was seen only in a search summary.

### USGS-VolcanoesUS
- Reference: U.S. Geological Survey, "Volcanoes of the United States" (general interest publication).
- DOI or URL: https://pubs.usgs.gov/gip/volcus/ustext.html
- Cited as: "USGS, Volcanoes of the United States".
- Ledger ids: VOL-152
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: "about 90 percent of Kilauea's surface is covered with lava flows that are less than 1,100 years old".

### USGS-MedicineLake
- Reference: U.S. Geological Survey, "Lava flows and domes of Medicine Lake Volcano".
- DOI or URL: https://www.usgs.gov/volcanoes/medicine-lake/science/lava-flows-and-domes-medicine-lake-volcano
- Cited as: "USGS, Medicine Lake".
- Ledger ids: VOL-141
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: andesite and basaltic andesite flows "typically limited in length to less than 12 km", under 25 km²; basalt flows can cover more than 300 km²; the Giant Crater basalt flow 45 km long, about 200 km².

### USGS-VHP-LavaFlows
- Reference: U.S. Geological Survey, Volcano Hazards Program, "Lava flows destroy everything in their path".
- DOI or URL: https://www.usgs.gov/programs/VHP/lava-flows-destroy-everything-their-path
- Cited as: "USGS VHP".
- Ledger ids: VOL-141
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: "Fluid basalt flows can extend tens of kilometers from an erupting vent"; andesite flows "rarely extend more than 8 km (5 mi) from their vents"; dacite and rhyolite domes from "many individual flows >30 m (100 ft) thick".

### NPS-LavaFlows
- Reference: U.S. National Park Service, "Lava flows" (volcanoes subject page).
- DOI or URL: https://www.nps.gov/subjects/volcanoes/lava-flows.htm
- Cited as: "NPS, Lava flows".
- Ledger ids: VOL-142, VOL-157
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: Trident Volcano's andesite block flows "each about 1.5-2.5 mi (2.5-4 km) long and 80-200 ft (25-60 m) thick"; intermediate flows "thicker than basaltic ones and have very steep flow fronts"; lava tubes "usually only found in pāhoehoe flows". Its rhyolite thickness conversion is inconsistent and was not used.

### USGS-FS144-02
- Reference: U.S. Geological Survey Fact Sheet 144-02 (2002), on Kilauea's Pu'u 'O'o eruption.
- DOI or URL: https://pubs.usgs.gov/fs/2002/fs144-02/
- Cited as: "USGS FS 144-02".
- Ledger ids: VOL-157
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: Pu'u 'O'o aa flows "typically 10 to 15 feet (3-5 m) thick"; tube-fed pahoehoe built a coastal plain 10 to 35 m thick; the flow field 15 km wide along the coast.

### PetersonTilling1980
- Reference: Peterson, D. W. and Tilling, R. I. (1980). Transition of basaltic lava from pahoehoe to aa, Kilauea Volcano, Hawaii: field observations and key factors. Journal of Volcanology and Geothermal Research 7(3-4):271-293 (the title from memory).
- DOI or URL: 10.1016/0377-0273(80)90033-5; abstract at https://pubs.usgs.gov/publication/70012218
- Cited as: "Peterson & Tilling 1980".
- Ledger ids: VOL-158
- Access: paywalled; the abstract open (USGS).
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22: "Nearly all Hawaiian basaltic lava erupts as pahoehoe, and some changes to aa during flowage and cooling"; "if viscosity is low, a high rate of shear is required to begin the transition to aa"; slope and flow rate act through the flow's mechanics.

### RowlandWalker1990
- Reference: Rowland, S. K. and Walker, G. P. L. (1990). Pahoehoe and aa in Hawaii: volumetric flow rate controls the lava structure. Bulletin of Volcanology 52:615-628 (the title from memory).
- DOI or URL: 10.1007/BF00301212
- Cited as: "Rowland & Walker 1990".
- Ledger ids: VOL-158
- Access: paywalled (refused).
- Library: not fetched.
- Read so far (by the project): a search summary of the abstract only, 2026-10-22: aa forms above a volumetric flow rate of about 5 to 10 m³/s, pahoehoe below, aa by "rapid and concentrated flow in open channels"; pahoehoe units usually under 1 m thick.

### HVO-VW1996
- Reference: USGS Hawaiian Volcano Observatory, Volcano Watch, 5 April 1996, "Age and appearance of lava flows".
- DOI or URL: https://www.usgs.gov/news/volcano-watch-age-and-appearance-lava-flows
- Cited as: "HVO Volcano Watch 1996".
- Ledger ids: VOL-148, VOL-149
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: the dry Kona coast has 10 to 20 in/yr of rain, and its flows of 137 years, about 200 years and over 1,500 years are "bare, black, glistening" and "appear to have erupted yesterday"; wet Hilo has 100 to 200 in/yr, and a flow only 115 years old has "large 'ohi'a trees ... throughout"; the 1984 flows already carry lichen, ferns and small ohia.

### HVO-VW1999
- Reference: USGS Hawaiian Volcano Observatory, Volcano Watch, 21 January 1999, on lava-flow forests and primary succession.
- DOI or URL: https://www.usgs.gov/news/volcano-watch-lava-flow-forest-primary-succession
- Cited as: "HVO Volcano Watch 1999".
- Ledger ids: VOL-149
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: "A forest can develop in wet regions in less than 150 years"; no figure for dry areas. Its "20 and 40 percent a century" for Hualalai and Mauna Loa is probably loose wording for a thousand years and was not used.

### HVO-VW2006
- Reference: USGS Hawaiian Volcano Observatory, Volcano Watch, 7 December 2006, on the colour of weathering lava flows; and 15 October 2015, on dating flows by their look.
- DOI or URL: the USGS Volcano Watch archive (https://www.usgs.gov/observatories/hvo/news), the two articles by date.
- Cited as: "HVO Volcano Watch 2006".
- Ledger ids: VOL-151, VOL-162
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: a flow weathers "from black to gray, to brown, and tan, then orange, and finally red"; the 2015 article: how useful a flow's look is for its age "is influenced by elevation, rainfall, and shielding by forest and tephra deposits". Neither gives years per colour.

### Kinney2015
- Reference: Kinney, K. M. and co-authors (2015). PLoS ONE 10(6):e0123995 (dryland vegetation on Hawaiian lava flows of known age; the authors after the first and the title not recorded).
- DOI or URL: https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0123995
- Cited as: "Kinney et al. 2015".
- Ledger ids: VOL-148
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: dryland flows under 300 years "96.1% barren on pāhoehoe and 94.1% on aʻa"; tree height peaks at 0.75 to 1.5 ky on pahoehoe and 1.5 to 3 ky on aa.

### Wells1985
- Reference: Wells, S. G., Dohrenwend, J. C., McFadden, L. D., Turrin, B. D. and Mahrer, K. D. (1985). Late Cenozoic landscape evolution on lava flow surfaces of the Cima volcanic field, Mojave Desert, California. GSA Bulletin 96(12):1518-1529 (the title from memory).
- DOI or URL: 10.1130/0016-7606(1985)96<1518:LCLEOL>2.0.CO;2
- Cited as: "Wells et al. 1985".
- Ledger ids: VOL-147
- Access: paywalled (refused).
- Library: not fetched.
- Read so far (by the project): not read. Through Wikipedia's Cima page (fetched, secondary): "Younger flows often show crisp lava flow features", older ones "mantled with younger material" with desert varnish; a search summary: the eolian mantle is most extensive on flows 0.25 to 0.75 Myr old and erodes above about 0.7 Myr. The youngest Cima flow, Black Tank, is 15,000 ± 5,000 years old (Wikipedia, fetched) or about 12 ka (MillerBuesch2022).

### MillerBuesch2022
- Reference: Miller, D. M. and Buesch, D. C. (2022), an abstract on the age of the Black Tank flow, Cima volcanic field (the full reference not recorded by the sub-agent).
- DOI or URL: not recorded.
- Cited as: "Miller & Buesch 2022".
- Ledger ids: VOL-147
- Access: the abstract open.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22: Black Tank about 12 ka.

### Favalli2005
- Reference: Favalli, M., Pareschi, M. T., Neri, A. and Isola, I. (2005). Forecasting lava flow paths by a stochastic approach. Geophysical Research Letters 32(3), L03305 (the title from memory; the article number from a search summary).
- DOI or URL: 10.1029/2004GL021718
- Cited as: "Favalli et al. 2005".
- Ledger ids: VOL-156
- Access: paywalled (refused); Crossref metadata and an abstract summary only.
- Library: not fetched.
- Read so far (by the project): metadata and an abstract summary, 2026-10-22: DOWNFLOW traces many steepest-descent paths over randomly perturbed topography. Its own Δh and path count were not read (Etna h = 3 m, n = 10,000 only in search summaries).

### Chevrel2021
- Reference: Chevrel, M. O. and co-authors (2021). Natural Hazards and Earth System Sciences 21:2355 (a lava-flow hazard study using DOWNFLOW; the authors after the first and the title not recorded).
- DOI or URL: https://nhess.copernicus.org/articles/21/2355/2021/
- Cited as: "Chevrel et al. 2021".
- Ledger ids: VOL-156
- Access: open.
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: each pixel perturbed "in the range of ±Δh", Δh "the characteristic vertical height of an obstacle"; Δh 0 to 5 m and N 100 to 10,000 tested, N = 10,000 chosen.

### Papale2018
- Reference: Papale, P. (2018). Global time-size distribution of volcanic eruptions on Earth. Scientific Reports 8:6838 (the title from memory).
- DOI or URL: 10.1038/s41598-018-25286-y
- Cited as: "Papale 2018".
- Ledger ids: VOL-146
- Access: open (PMC).
- Library: not fetched.
- Read so far (by the project): fetched 2026-10-22: about 50 % of eruptions are VEI 1 or lower, and "very low VEI eruptions are mostly associated with the formation of lava flows".

### Heliker2003, Siebert2010, Wikipedia-Etna
- Reference: Heliker, C. and co-authors, USGS Professional Paper 1676 (Pu'u 'O'o); Siebert, L., Simkin, T. and Kimberly, P. (2010), Volcanoes of the World, 3rd edition; Wikipedia, "Mount Etna" (secondary).
- DOI or URL: https://pubs.usgs.gov/pp/1676/ (from memory); none; https://en.wikipedia.org/wiki/Mount_Etna
- Cited as: "Heliker et al.", "Siebert et al. 2010", "Wikipedia, Mount Etna".
- Ledger ids: VOL-146, VOL-159
- Access: open; a book; open.
- Library: not fetched.
- Read so far (by the project): Heliker et al. only in a search summary (Pu'u 'O'o's open channels 2 to 5 m deep and 5 to 25 m wide, tubes 3 to 5 m wide); Siebert et al. not read (it may tabulate how often arc eruptions make lava flows); Etna's page fetched 2026-10-22: "Since the year AD 1600, at least 60 flank eruptions", nearly half since 1900.

### WhiteheadClift2009
- Reference: Whitehead, J. A. and Clift, P. D. (2009). Continent elevation, mountains, and erosion: Freeboard implications. Journal of Geophysical Research: Solid Earth 114(B5).
- DOI or URL: 10.1029/2008JB006176
- Cited as: "Whitehead & Clift 2009".
- Ledger ids: PLT-412
- Access: paywalled; the abstract from OpenAlex's record.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22 (the high-ground task): "If continent elevation (freeboard) remains at the present value of a few hundred meters, the past continent‐ocean area ratio might have been quite different, depending on earlier volumes of continental crust and water. We conclude that, along with the ages of ocean basins, continental crustal thickening exerts a first‐order control on the global sea level over hundreds of million years"; since 65 Ma "a 156‐m increase of continent elevation over sea level", "affected most strongly by the roughly 1.7% continent surface area decrease caused by Cenozoic mountain building".

### HousemanEngland1986
- Reference: Houseman, G. A. and England, P. C. (1986). Finite strain calculations of continental deformation: 1. Method and general results for convergent zones. Journal of Geophysical Research 91(B3):3651-3663.
- DOI or URL: 10.1029/JB091iB03p03651
- Cited as: "Houseman & England 1986" (the thin viscous sheet, the Argand number).
- Ledger ids: PLT-414
- Access: paywalled; the abstract from OpenAlex's record.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22 (the high-ground task): "the strength of the sheet relative to the gravitational forces (which may be expressed as a dimensionless number: the Argand number Ar)"; "For n > 3 and Ar between 1 and 10 a plateau of thickened crust is formed in front of the indenter"; "the length scale of deformation increases with Ar and time since the start of the collision and decreases with n".

### EnglandHouseman1989
- Reference: England, P. and Houseman, G. (1989). Extension during continental convergence, with application to the Tibetan Plateau. Journal of Geophysical Research 94(B12):17561-17579.
- DOI or URL: 10.1029/JB094iB12p17561
- Cited as: "England & Houseman 1989".
- Ledger ids: PLT-414, PLT-415, PLT-478 (2026-10-23: the instability "would only occur after there had already been substantial thickening of the lithosphere", the same abstract read again)
- Access: paywalled; the abstract from OpenAlex's record.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22 (the high-ground task): "For the last 5 m.y. or so the plateau has been thinning by the mechanism of extension on north‐south trending normal faults"; the collision time scale "(∼50 m.y. in the case of India and Asia)"; "the increase in surface height (as much as 2 km) and of potential energy (5 to 10 × 10 12 N m −1 ) resulting from convective instability of the lower lithosphere are sufficient for east‐west extension to replace north‐south compression as the dominant feature of the stress field of the Tibetan plateau".

### EnglandMolnar1997
- Reference: England, P. and Molnar, P. (1997). Active deformation of Asia: from kinematics to dynamics. Science 278(5338):647-650.
- DOI or URL: 10.1126/science.278.5338.647
- Cited as: "England & Molnar 1997" (the Tibetan lithosphere's viscosity).
- Ledger ids: PLT-414
- Access: paywalled; the abstract from OpenAlex's record.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22 (the high-ground task): "gradients of stress point uphill toward the center of Tibet, where gravitational potential energy per unit surface area reaches a maximum"; "This balance, in the region of the Tibetan plateau, yields an estimate of 10 22 pascal second for the average viscosity of the Tibetan lithosphere, which is only about 10 to 100 times greater than the viscosity of the convecting upper mantle".

### FleschEtAl2000
- Reference: Flesch, L. M., Holt, W. E., Haines, A. J. and Shen-Tu, B. (2000). Dynamics of the Pacific-North American plate boundary in the western United States. Science 287(5454):834-836.
- DOI or URL: 10.1126/science.287.5454.834
- Cited as: "Flesch et al. 2000" (the Basin and Range's viscosity).
- Ledger ids: PLT-414
- Access: paywalled; the abstract from OpenAlex's record.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22 (the high-ground task): "Deformation in the Basin and Range is driven, in part, by gravitational potential energy differences, but extension directions there are modified by plate interaction stresses. The California shear zone has relatively low vertically averaged viscosity of about 10(21) pascal.seconds, whereas the Basin and Range has a higher vertically averaged viscosity of 10(22) pascal.seconds."

### SonderJones1999
- Reference: Sonder, L. J. and Jones, C. H. (1999). Western United States extension: how the west was widened. Annual Review of Earth and Planetary Sciences 27:417-462.
- DOI or URL: 10.1146/annurev.earth.27.1.417
- Cited as: "Sonder & Jones 1999".
- Ledger ids: PLT-414
- Access: paywalled; the abstract from OpenAlex's record.
- Library: not fetched.
- Read so far (by the project): the abstract, 2026-10-22 (the high-ground task): "Forces driving extension can originate at plate boundaries, through basal traction, basal normal forces, or from buoyancy forces internal to the crust and lithospheric mantle. The latter two are most responsible for driving extension where it is observed in the Basin and Range."

## The cube simulation grid and its surface solvers (SCL-140 to SCL-152), added 2026-10-22

Written for the cube world's phase 4 (S1, S2) by the agent that coded them. None of these was
fetched: the formulas were written from memory and checked numerically (docs/science/cube-simulation-grid.md,
"Earth against the mod"). Putman & Lin 2007, Harris et al. 2021 and Braess 1995 above also carry
these rows.

### VanOosterom1983
- Reference: Van Oosterom, A. and Strackee, J. (1983). The solid angle of a plane triangle. IEEE Transactions on Biomedical Engineering BME-30(2):125-126 (from memory).
- DOI or URL: not looked up.
- Cited as: "Van Oosterom & Strackee 1983" (sphere/cube.rs: solid_angle).
- Ledger ids: SCL-140
- Access: paywalled (IEEE), not fetched.
- Library: not fetched.
- Read so far (by the project): from memory: tan(Omega/2) = |a . (b x c)| / (|a||b||c| + (a . b)|c| + (a . c)|b| + (b . c)|a|), used with unit vectors. Checked by the cube's cell areas summing to 4 pi R^2 within 2.5e-16.

### Meyer2003
- Reference: Meyer, M., Desbrun, M., Schröder, P. and Barr, A. H. (2003). Discrete differential-geometry operators for triangulated 2-manifolds. In Visualization and Mathematics III, Springer, 35-57 (from memory).
- DOI or URL: not looked up.
- Cited as: "Meyer, Desbrun, Schröder & Barr 2003" (sphere/cube.rs module notes, the cotangent weights).
- Ledger ids: SCL-141
- Access: not fetched (an author copy is said to be open).
- Library: not fetched.
- Read so far (by the project): from memory: the linear finite-element (P1) stiffness of a triangle mesh gives each edge half the sum of the cotangents of the two angles opposite it. Checked by the Laplacian of degree-1 and degree-2 harmonics on the cube (SCL-141).

### Hilbert1891
- Reference: Hilbert, D. (1891). Über die stetige Abbildung einer Linie auf ein Flächenstück. Mathematische Annalen 38(3):459-460 (from memory).
- DOI or URL: not looked up.
- Cited as: "Hilbert curve" (sphere/cube.rs: hilbert_key).
- Ledger ids: SCL-143
- Access: not fetched.
- Library: not fetched.
- Read so far (by the project): nothing; the code is the standard iterative index-to-curve map, checked by its tests (each face's keys are a permutation, consecutive cells are side neighbours).

## The trench budget (PLT-440 to PLT-443, HYP-050, HYP-051), added 2026-10-22

The sources of the trench budget (build log 2026-10-22 "The trench budget"; plates.md, the section of that name). Stern & Scholl 2010, von Huene & Scholl 1991, Straume et al. 2019, Harris et al. 2014 and Tozer et al. 2019 are above, with what was read this time. Read by a research sub-agent on 2026-10-22; the quotes are as read, and the sub-agent's own arithmetic on a table is marked as such. Not read: Grellet & Dubois 1982 (metadata only), Scholl & von Huene 2007 (GSA Memoir 200, Table 2; every route blocked), Jamieson et al. 2010, Zheng & Wang 2023.

### CliftVannucchi2004
- Reference: Clift, P. and Vannucchi, P. (2004). Controls on tectonic accretion versus erosion in subduction zones: implications for the origin and recycling of the continental crust. Reviews of Geophysics 42(2):RG2001.
- DOI or URL: 10.1029/2003RG000127; repository copy https://repository.lsu.edu/cgi/viewcontent.cgi?article=1495&context=geo_pubs (read through the Wayback Machine's copy).
- Cited as: "Clift & Vannucchi 2004".
- Ledger ids: PLT-440, PLT-441, PLT-442; PLT-522 and PLT-523 (2026-10-23, the trench's retreat: Table 2's retreat range and the global 90 km3/km/Myr as the check on the volume it takes).
- Access: paywalled at the publisher; the repository copy open.
- Library: not kept.
- Read so far (by the project): full text with Tables 1 and 2, 2026-10-22 (research sub-agent). Abstract: "Tectonic erosion is favored in regions where convergence rates exceed 6 ± 0.1 cm yr-1 and where the sedimentary cover is <1 km. Accretion preferentially occurs in regions of slow convergence (<7.6 cm yr-1) and/or trench sediment thicknesses >1 km." Body: "erosive plate margins do not appear to form in regions where the orthogonal convergence rate is <6.3 cm yr-1, while accretionary margins do not form in regions where the orthogonal convergence rate exceeds 7.6 cm yr-1. The Philippine Trench is the exception"; [34] "accretion appears to be an effective process when convergent rates are <76 mm yr-1 and where the sedimentary cover is >1 km thick"; [40] "a global average rate of crustal loss of 90 km3 m.y.-1 km-1 calculated in this study"; [38] at erosive margins "only 12-48% (median 22%) of the total material subducted below the forearc is derived from the sediments in the subducting plate"; [39] "Only 7-37% of the sediment reaching the trenches appears to be added to the accretionary complexes, with the bulk subducted to depth", "Globally, the median proportion of the rock delivered as sediment to the trench that is accreted from the subducting plate is only 17%"; [53] "57% of the global active margins seem to be of the erosive type", "Even at the accretionary margins, typically 83% of the incoming sedimentary pile is subducted beneath the forearc wedge", "eroded forearc basement constitutes approximately 55% of the crustal material subducted to depth". No global flux in km3/yr is printed. The sub-agent's arithmetic on Tables 1 and 2 (not the paper's): basement erosion on the 19 erosive margins 20.0 (Mariana) to 139.4 (Peru) km3 per km and Myr, length-weighted 86.8, about 1.99 km3 a year in all; erosive margins 22,925 km of the 40,225 km tabulated (57.0 %); sediment delivered 1.90 km3 a year, 1.62 subducted and 0.28 accreted (14.6 %); trench retreat 0.9 to 4.7 km/Myr, median 3.0.

### SchollvonHuene2010
- Reference: Scholl, D. W. and von Huene, R. (2010). Subduction zone recycling processes and the rock record of crustal suture zones. Canadian Journal of Earth Sciences 47(5) (volume and issue from-memory).
- DOI or URL: read in Gale OneFile (document GALE|A230063766).
- Cited as: "Scholl & von Huene 2010".
- Ledger ids: PLT-441; PLT-522 (2026-10-23: the ~2.5 km/Ma of the trench's retreat as the rate) and PLT-523.
- Access: paywalled; read through a library database.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-22 (research sub-agent): "for each kilometre of OMSZ sediment subduction and subduction erosion combined, ~60 km3/Ma of solid volume crust is recycled toward the mantle"; "Subduction erosion truncates the margin (migrates the trench inboard) at ~2.5 km/Ma" (range 1 to 5); "OMSZs with wide middle prisms of accreted material constitute an important 25% of the global length of modern convergent margins"; the forearc's "long-term subsidence of the submerged forearc across its width and to depths of 3-5 km".

### Jarrard1986
- Reference: Jarrard, R. D. (1986). Relations among subduction parameters. Reviews of Geophysics 24(2):217-284.
- DOI or URL: 10.1029/RG024i002p00217 (from-memory); read through the Wayback Machine's copy of a ResearchGate upload (OCR text).
- Cited as: "Jarrard 1986".
- Ledger ids: PLT-443 (the trench's depth against the regional floor).
- Access: paywalled; a copy hosted.
- Library: not kept.
- Read so far (by the project): full text with OCR noise, 2026-10-22 (research sub-agent): Table 11, "Ad = -0.81 + 0.0185 x As + 0.0816 x DipI" (Table 2's units: Ad, km, the maximum trench depth less the regional crustal depth; As, slab age, m.y.; DipI, the intermediate dip, degrees), R = 0.77, "account for 60% of the variance in Ad"; Ad's correlation with slab age R = 0.57, with the subduction rate 0.45, with the dip 0.52; "Maximum trench depth is strongly correlated with regional seafloor depth away from the trench"; the Marianas, "a maximum depth of 11 km, the deepest of all trenches"; "Trench depths can be affected by three variables that are not specifically treated in this study: thickness of sedimentary trench fill, thickness of sediments on the oceanic crust before entering the trench, and presence of aseismic ridges". Not read: the per-trench rows of Table 1.

### Stern2021
- Reference: Stern, R. J. (2021). Ocean trenches. In Encyclopedia of Geology, 2nd edition, vol. 3, 845-854. Elsevier.
- DOI or URL: 10.1016/B978-0-08-102908-4.00099-0; author copy https://personal.utdallas.edu/~rjstern/pdfs/OceanicTrenchesEoG221.pdf
- Cited as: "Stern 2021".
- Ledger ids: HYP-051, PLT-443.
- Access: author copy open.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-22 (research sub-agent): Table 1, "Maximum trench depth" (m): Challenger (Mariana) 10,920, Tonga 10,800, Philippine 10,057, Kermadec 10,047, Izu-Bonin 9,780, Kuril 9,550, North New Hebrides 9,175, New Britain 8,940, Yap 8,650, Puerto Rico 8,605, South Sandwich 8,325, South Solomons 8,322, Peru-Chile 8,170, Japan 8,130, Palau 8,055, Aleutians 7,680, Ryukyu 7,460, Sunda 7,125, Middle America 6,660, Hellenic 5,092, Nankai 4,900, Calabrian 4,200, Makran 3,200, Cascadia 3,136; "Table 1 shows that all trenches deeper than 8000 m are nonaccretionary. In contrast, all trenches with growing accretionary prisms are shallower than 8000 m"; "The relative depth is affected by the age of the lithosphere at the trench, the convergence rate, and the dip of the subducted slab at intermediate depths"; "There are about 50,000 km of convergent plate margins in the world". Fig. 7 (after Grellet & Dubois 1982), read by eye, not numbers: the trench from about 5 km near age 0 to about 11 km at about 140 Ma, the normal floor from about 3 to about 6 km, so 2 to 5 km of relief with age. The sub-agent's counts on Table 1: 4 trenches deeper than 10 km, 15 than 8 km, 19 than 6 km; the maxima's median 8,246 m.

## The trench rows (ISO-096, ISO-097), added 2026-10-23

Also read for these rows: Turcotte & Schubert (TurcotteSchubert_Geodynamics) and Stern's chapter (Stern_OceanTrenches, the same chapter as Stern2021 above, from another copy), with what was read on 2026-10-23 under each.

### LevittSandwell1995
- Reference: Levitt, D. A., Sandwell, D. T. (1995). Lithospheric bending at subduction zones based on depth soundings and satellite gravity. Journal of Geophysical Research 100(B1):379-400.
- DOI or URL: 10.1029/94JB02468; author copy https://topex.ucsd.edu/sandwell/publications/60.pdf
- Cited as: "Levitt & Sandwell 1995" (the restoring force under a water-filled trench; trench profiles fitted to the bathymetry)
- Ledger ids: ISO-096
- Access: author copy (open).
- Library (the project's source library, not distributed): not filed.
- Read so far (by the project): full text (author copy), by a Sonnet sub-agent of the trench-row agent, 2026-10-23: "the thin plate is acted upon by a restoring force g(rho_m - rho_w)w, where w is the plate deflection ... rho_m and rho_w are mantle and water density"; the trench profiles are fitted to the depth soundings directly.
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: journal, volume and first page as the sub-agent read them; the title, the issue and the last page are from memory, not checked.

### ZhangLinZhan2014
- Reference: Zhang, F., Lin, J., Zhan, W. (2014). Variations in oceanic plate bending along the Mariana trench. Earth and Planetary Science Letters 401:206-214.
- DOI or URL: 10.1016/j.epsl.2014.05.032; open copy https://www.whoi.edu/fileserver.do?id=188564&pt=2&p=68128
- Cited as: "Zhang, Lin & Zhan 2014" (the Mariana's trench relief; the Challenger Deep's depth)
- Ledger ids: ISO-096, ISO-097
- Access: open copy (WHOI).
- Library (the project's source library, not distributed): not filed.
- Read so far (by the project): full text, by a Sonnet sub-agent of the trench-row agent, 2026-10-23: "the trench relief varies from 0.9 to 5.7 km" along the Mariana; "the Challenger Deep, with trench depth of about 10.9 km and trench relief of 5.7 km" (the far-field floor there, about 5.2 km, is the agent's subtraction); relief is "calculated from subtracting the trench-axis depth from a far-field reference depth"; eq. 1, "(rho_m - rho_w) g w represents hydrostatic restoring force".
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: journal, volume and first page as the sub-agent read them; the title, the DOI and the last page are from memory, not checked.

### ZhangTrench2018
- Reference: Zhang, F., Lin, J., Zhou, Z., Yang, H., Zhan, W. (2018). Intra- and intertrench variations in flexural bending of the Manila, Mariana and global trenches: implications on plate weakening in controlling trench dynamics. Geophysical Journal International 212(2):1429-1449 (authors, title and last page from memory; the volume and first page from the URL). Not the same work as Zhang2018 (supercontinent breakup).
- DOI or URL: https://academic.oup.com/gji/article/212/2/1429/4604782
- Cited as: "Zhang et al. 2018 (GJI 212:1429)" (the world's section-averaged trench relief)
- Ledger ids: ISO-097
- Access: open (OUP).
- Library (the project's source library, not distributed): not filed.
- Read so far (by the project): full text, by a Sonnet sub-agent of the trench-row agent, 2026-10-23: trench relief W0 is "trench axis depth relative to a far-field reference seafloor depth"; the global "section-averaged W0 is in the range of 0.9-4.7 km" (Manila 0.4 to 2.1 km, southern Mariana 1.7 to 5.7 km).
- Read in this pass: not part of stage 1; see "Read so far".
- Reference check: the authors and the title are from memory, not checked; journal, volume and first page from the URL.
## The release's pace (ISO-099), added 2026-10-23

The sources of the release's pace (build log 2026-10-23; flexure.md, the section of that name). McKenzie 1978, Parsons & Sclater 1977, Beaumont 1978 and Watts & Zhong 2000 are above, with what was read this time. Read by a Sonnet sub-agent of the release agent on 2026-10-23; the quotes are as read, en dashes turned to hyphens, and the sub-agent's own arithmetic is marked as such. Not read: Turcotte & Schubert on the thermal time (only the front matter was reachable), Watts 1978 and Lambeck & Nakiboglu 1981 (not opened), England & Thompson 1984 (abstract only, on heating thickened crust, nothing on weakening), Royden et al. 1983 (abstract only: Pannonian thermal subsidence and heat flow "about twice those predicted by assuming a uniform extension model"), Sclater et al. 1980 (a search tool's paraphrase only).

### HyndmanCurrieMazzotti2005
- Reference: Hyndman, R. D., Currie, C. A. and Mazzotti, S. P. (2005). Subduction zone backarcs, mobile belts, and orogenic heat. GSA Today 15(2):4-10.
- DOI or URL: 10.1130/1052-5173(2005)015<4:SZBMBA>2.0.CO;2
- Cited as: "Hyndman, Currie & Mazzotti 2005".
- Ledger ids: ISO-099, PLT-461, PLT-465, PLT-470, ISO-112 (2026-10-23, the lithosphere's thermal state)
- Access: open (GSA Today).
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: "The thermal effect of extension decays with a time constant of ~50 m.y."; "Moho temperatures are 800-900 °C and lithosphere thicknesses are 50-60 km, compared to 400-500 °C and 150-300 km for cratons and other regions with a thermotectonic age greater than ca. 300 Ma. The temperature differences result in backarc lithospheres that are weaker than cratons by more than a factor of 10"; "Most of the rapid decrease in heat flow appears to be in the several hundred m.y. following the last thermotectonic event."

### CurrieHyndman2006
- Reference: Currie, C. A. and Hyndman, R. D. (2006). The thermal structure of subduction zone back arcs. Journal of Geophysical Research 111:B08404.
- DOI or URL: 10.1029/2005JB004024
- Cited as: "Currie & Hyndman 2006".
- Ledger ids: ISO-099, PLT-461, PLT-465, ISO-112 (2026-10-23, the lithosphere's thermal state)
- Access: author copy.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: "uniformly high temperatures in the shallow mantle and a thin lithosphere (1200°C at 60 km depth) over back-arc widths of 250 to >900 km"; "Following subduction termination, we find that the high temperatures decay over a timescale of about 300 Myr"; Te "generally less than 20 km for the Cordillera back arc", the Archean craton "generally greater than 80 km"; "The most rapid cooling occurs in the first 50-100 Myr."

### CurrieHuismansBeaumont2008
- Reference: Currie, C. A., Huismans, R. S. and Beaumont, C. (2008). Thinning of continental backarc lithosphere by flow-induced gravitational instability. Earth and Planetary Science Letters 269:435-446.
- DOI or URL: 10.1016/j.epsl.2008.02.037 (from memory, not checked)
- Cited as: "Currie, Huismans & Beaumont 2008".
- Ledger ids: ISO-099, PLT-464 (2026-10-23, the lithosphere's thermal state)
- Access: author copy.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23 (model results, not observations): "rapidly-growing downwellings which result in removal of lower backarc lithosphere on timescales of 5-10 Ma"; "In the models, the majority of thinning occurs within 30 Ma of subduction initiation"; "by 60 Ma, the lithosphere is likely close to its stable steady-state thickness" (from a 120 km lithosphere). No observed onset time for a back-arc's weakening was found.

### HorvathCloetingh1996
- Reference: Horváth, F. and Cloetingh, S. (1996). Stress-induced late-stage subsidence anomalies in the Pannonian basin. Tectonophysics 266:287-300.
- DOI or URL: 10.1016/S0040-1951(96)00194-1
- Cited as: "Horváth & Cloetingh 1996" (a basin that opened and filled as it subsided).
- Ledger ids: none (flexure.md, "The release's pace": the fill)
- Access: open (VU research portal).
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: "the Pannonian depression has been filled up by a fluvial dominated delta-system which prograded from the basin periphery towards the interior"; "Sediment accumulation rate was significantly higher than the subsidence rate during Late Miocene time"; "Late Pliocene and Quaternary strata (0-2.4 Ma) can only be found in the Great Hungarian Plain with a thickness up to 700 m. The surface of this area is indeed a plain and with an elevation between 90-120 m above sea level."

### Horvath2015
- Reference: Horváth, F., Musitz, B., Balázs, A., Végh, A., Uhrin, A., Nádor, A., Koroknai, B., Pap, N., Tóth, T. and Wórum, G. (2015). Evolution of the Pannonian basin and its geothermal resources. Geothermics 53:328-352 (the author list after the first from memory).
- DOI or URL: 10.1016/j.geothermics.2014.07.009
- Cited as: "Horváth et al. 2015".
- Ledger ids: none (flexure.md, "The release's pace": the fill)
- Access: open (Utrecht repository).
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: "deep troughs exceeding a thickness of 5 km, shallow basin areas typically with a depth of 1 to 2 km"; "Rapid subsidence began at places which led to locally 800-1000 m water depths"; "Lake Pannon reached its largest extent about 9.8 Ma. Then, rapid infill of clastic material started by inflow of large rivers"; the basin is "'over-elevated', in other words it should be well below the sea level in case of equilibrium."

### Vey2021
- Reference: Vey, S., Al-Halbouni, D. et al. (2021) (the initials from memory). Delayed subsidence of the Dead Sea shore due to hydro-meteorological changes. Scientific Reports (volume and article number not checked).
- DOI or URL: 10.1038/s41598-021-91949-y
- Cited as: "Vey et al. 2021" (the Dead Sea's level).
- Ledger ids: none (flexure.md, "The release's pace": Earth's lowest dry land)
- Access: open.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: "The lake level has steadily decreased with now more than - 110 cm/year, leaving the level in October 2018 at 433 m below mean sea-level (msl)."

## The lithosphere's thermal state (PLT-460 to PLT-470, ISO-110 to ISO-112), added 2026-10-23

The sources of the lithosphere's thermal state (gap 4; `lithosphere-thermal-state.md`). McKenzie 1978, Hyndman & Currie 2011, Hyndman, Currie & Mazzotti 2005, Currie & Hyndman 2006 and Currie, Huismans & Beaumont 2008 are above; their ledger ids now include the rows of this section. Read by a Sonnet sub-agent of the gap-4 agent on 2026-10-23; the quotes are as read, en dashes turned to hyphens. Not read: Lachenbruch & Morgan 1990 (Tectonophysics 174:39, abstract only, no thermal rule in it), Petitjean et al. 2006 (G-cubed 7, abstract: "it takes at least 200 Myr for a hot mantle plume to erode the thick lithospheric root"), Angevine, Heller & Paola 1990 (a course scan of chapter 4 read: "For the parameters that we have given, τ = 62.8 My", κ = 8 × 10^-7 m²/s, a 125 km lithosphere; not given an entry).

### Jordan1978
- Reference: Jordan, T. H. (1978). Composition and development of the continental tectosphere. Nature 274:544-548.
- DOI or URL: 10.1038/274544a0
- Cited as: "Jordan 1978" (cratons as depleted, buoyant, stiff keels).
- Ledger ids: PLT-460
- Access: paywalled.
- Library: not kept.
- Read so far (by the project): abstract only, 2026-10-23: "Beneath the old continental nuclei are thick root zones which translate coherently during plate motions. These zones are apparently stabilised against convective disruption by the depletion of the continental upper mantle in a basalt-like component."

### Jordan1988
- Reference: Jordan, T. H. (1988). Structure and formation of the continental tectosphere. Journal of Petrology Special Volume 1:11-37.
- DOI or URL: 10.1093/petrology/Special_Volume.1.11
- Cited as: "Jordan 1988".
- Ledger ids: none (lithosphere-thermal-state.md, the keel)
- Access: paywalled.
- Library: not kept.
- Read so far (by the project): abstract only (OpenAlex), 2026-10-23: "a thick (> 300 km) thermal boundary layer (TBL) beneath the ancient cratonic nuclei", "stabilized against convective disruption by a buoyant, viscous, chemical boundary layer (CBL) depleted in basaltic constituents".

### Lee2011
- Reference: Lee, C.-T. A., Luffi, P. and Chin, E. J. (2011). Building and destroying continental mantle. Annual Review of Earth and Planetary Sciences 39:59-90 (the co-authors and pages from memory).
- DOI or URL: https://static1.squarespace.com/static/54b9bb6fe4b07b4a7d145b55/t/54bc8490e4b054d2824068be/1421640848178/63-2011LEE-Craton-AnnualReviews.pdf
- Cited as: "Lee 2011" (Jordan's isopycnic hypothesis as stated there).
- Ledger ids: PLT-460
- Access: author copy.
- Library: not kept.
- Read so far (by the project): full text, section 1, 2026-10-23: "Jordan proposed the isopycnic (iso = equal, pycnic = density) hypothesis: Continental mantle is made up of low-density, melt-depleted peridotite residua, whose low densities exactly offset the effect of thermal contraction associated with a growing thermal boundary layer (Jordan 1978, 1988)"; the longevity of continental mantle "demands an intrinsically stronger (more viscous) mantle".

### NybladePollack1993
- Reference: Nyblade, A. A. and Pollack, H. N. (1993). A global analysis of heat flow from Precambrian terrains: implications for the thermal structure of Archean and Proterozoic lithosphere. Journal of Geophysical Research 98(B7):12207-12218.
- DOI or URL: 10.1029/93JB00521
- Cited as: "Nyblade & Pollack 1993" (the Archean mean heat flow).
- Ledger ids: PLT-467
- Access: paywalled (Wiley returned 403).
- Library: not kept.
- Read so far (by the project): abstract only (Crossref), 2026-10-23: heat flow is "low in Archean cratons and Proterozoic terrains adjacent to cratonic margins and higher in Proterozoic terrains that are more than a few hundred kilometers from a craton". The means are read only as cited: "The Archean parts of the cratons have low average surface heat flow values of about 41 mW m−2" and "Proterozoic regions display average heat flow around 48 mW m−2" (an open GJI paper, 180(3):999, full text); 41 ± 11 as Jaupart & Mareschal 1999 cite it (a search summary, not read). Nyblade 1996's abstract (JGR 101:17737) gives 55 ± 17 mW m-2 for undisturbed late Proterozoic terrains; 48 against 55 is unresolved.

### HasterokChapman2007
- Reference: Hasterok, D. and Chapman, D. S. (2007). Continental thermal isostasy: 1. Methods and sensitivity, and 2. Application to North America. Journal of Geophysical Research 112:B06414 and B06415.
- DOI or URL: 10.1029/2006JB004663 and 10.1029/2006JB004664 (from memory, not checked)
- Cited as: "Hasterok & Chapman 2007".
- Ledger ids: ISO-110
- Access: paywalled (Wiley and ResearchGate returned 403).
- Library: not kept.
- Read so far (by the project): abstracts (Crossref), 2026-10-23: part 1, "thermal isostasy can produce nearly 3 km of relief between cold shield platforms and hot rift zones"; part 2, "The best fitting continental heat flow–elevation model has a reference heat flow of 46.6 mW m−2 at 0 km", "compositionally adjusted elevations show a clear trend with ∼3 km difference between hot and cold provinces". Hasterok & Chapman 2011 (EPSL, full text, section 6.1): "a rough correlation between elevation and HF that accounts for approximately 2.5 km of elevation", zero-elevation heat flow 47.5 mW/m².

### HousemanMolnar1997
- Reference: Houseman, G. A. and Molnar, P. (1997). Gravitational (Rayleigh-Taylor) instability of a layer with non-linear viscosity and convective thinning of continental lithosphere. Geophysical Journal International 128:125-150.
- DOI or URL: 10.1111/j.1365-246X.1997.tb04075.x
- Cited as: "Houseman & Molnar 1997" (the heating time; root removal for stage 3).
- Ledger ids: PLT-464, PLT-477
- Access: open (OUP, a scan with an OCR layer; the equations checked on the page images).
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: p. 131, "The timescale for thinning of the boundary layer depends strongly on the mantle viscosity, but may require less than 10 Myr if the viscosity is less than about 10^21 Pa s"; conclusions, "between 50 (for rheological parameters representative of dry olivine) and 75 per cent (for wet olivine) of the lithospheric mantle would be involved in an instability whose duration is of the order of 10 Myr"; p. 144, the thermal diffusion time "about 20 Myr for wavelengths of 150 km"; eq. 30, the drop-off time tb.

### ConradMolnar1997
- Reference: Conrad, C. P. and Molnar, P. (1997). The growth of Rayleigh-Taylor-type instabilities in the lithosphere for various rheological and density structures. Geophysical Journal International 129:95-112.
- DOI or URL: https://www.clintconrad.no/papers/Conrad_GJI1997.pdf
- Cited as: "Conrad & Molnar 1997".
- Ledger ids: none (lithosphere-thermal-state.md, stage 3)
- Access: author copy.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: "e-folding times of three million years, for asthenospheric viscosities of about 10^19 Pa s"; viscosities greater than about 10^20 Pa s (the exponent garbled in the PDF) "allow thermal diffusion to slow growth rates to the point of stopping Rayleigh-Taylor growth completely"; p. 96, thermal smoothing τ = λ²/4π²κ, "τ = 70 to 300 million years" for wavelengths of 300 to 600 km.

### Rolandone2002
- Reference: Rolandone, F., Jaupart, C., Mareschal, J.-C., Gariépy, C., Bienfait, G., Carbonne, C. and Lapointe, R. (2002). Surface heat flow, crustal temperatures and mantle heat flow in the Proterozoic Trans-Hudson Orogen, Canadian Shield. Journal of Geophysical Research 107(B12):2341 (the author list after the first from memory).
- DOI or URL: https://seismo.berkeley.edu/~frede/rolandone-et-al-JGR02.pdf
- Cited as: "Rolandone et al. 2002".
- Ledger ids: PLT-462, PLT-463, PLT-467
- Access: author copy.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: abstract, mean Trans-Hudson heat flow "42 ± 9 mW m-2", mantle heat flow upper bound 16 and lower bound 11-12 mW m-2; section 5.1, the Superior, Trans-Hudson and Grenville provinces' "mean values and standard deviations of 41- 42 and 9 - 11 mW m-2"; para 63, "Jaupart and Mareschal [1999] concluded that lithosphere thickness beneath old cratons is unlikely to be smaller than 200 km and larger than 350 km", Jaupart et al. [1998] "a more restricted range of 200 - 250 km".

### Sleep2003
- Reference: Sleep, N. H. (2003). Survival of Archean cratonal lithosphere. Journal of Geophysical Research 108(B6):2302.
- DOI or URL: https://geosci.uchicago.edu/~archer/deep_earth_readings/sleep.2003.craton_lith.pdf
- Cited as: "Sleep 2003".
- Ledger ids: PLT-462, PLT-467
- Access: open copy.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: abstract, "Modern cratonal lithospheric thicknesses are relatively uniform, 225 km"; section 2.2.1, "Jaupart et al. [1998]... compile mantle heat flows of 7-15 mW m-2 for the Canadian shield. Jaupart and Mareschal [1999] obtain 10-15 mW m-2 for eastern Canada and 17 mW m-2 for the Kaapvaal craton."

### JaupartMareschal1999
- Reference: Jaupart, C. and Mareschal, J.-C. (1999). The thermal structure and thickness of continental roots. Lithos 48:93-114.
- DOI or URL: 10.1016/S0024-4937(99)00023-7
- Cited as: "Jaupart & Mareschal 1999" (as Rolandone et al. 2002 and Sleep 2003 cite it).
- Ledger ids: PLT-463, PLT-467
- Access: paywalled (ScienceDirect returned 403; Crossref has no abstract).
- Library: not kept.
- Read so far (by the project): not read; only as cited (Rolandone2002, Sleep2003). A search summary of its abstract gives 200 to 330 km; Rolandone et al. cite 350 km as the upper bound.

### Michaut2007
- Reference: Michaut, C., Jaupart, C. and Bell, D. R. (2007). Transient geotherms in Archean continental lithosphere: new constraints on thickness and heat production of the subcontinental lithospheric mantle. Journal of Geophysical Research 112:B04408 (the title and co-authors from memory).
- DOI or URL: 10.1029/2006JB004464 (from memory, not checked)
- Cited as: "Michaut et al. 2007".
- Ledger ids: none (lithosphere-thermal-state.md, the cooling's length)
- Access: paywalled.
- Library: not kept.
- Read so far (by the project): abstract (Crossref), 2026-10-23: Kaapvaal lithosphere "between 200 and 270 km, with most values between 210 and 250 km", basal heat flow "between 12-16 mW m−2"; "Because of its large thickness and thermal relaxation time, Archean lithosphere cannot be in thermal equilibrium".

### Marcilly2022
- Reference: Marcilly, C. M., Torsvik, T. H. and Conrad, C. P. (2022). Global Phanerozoic sea levels from paleogeographic flooding maps. Gondwana Research 110:128-142.
- DOI or URL: 10.1016/j.gr.2022.05.011 ; https://www.clintconrad.no/papers/Marcilly_etal_GR2022.pdf
- Cited as: "Marcilly, Torsvik and Conrad 2022", "Marcilly et al. 2022, Table 1" (exposed land and continental crust at 10 Myr steps; the hypsometric slope).
- Ledger ids: RLC-010, RLC-011, RLC-013, RLC-014, RLC-030, RLC-031 (2026-10-23: Earth's land share and flooded continental crust over the Phanerozoic, the realism checks)
- Access: open (the author's copy on the Conrad group's page).
- Library: not kept (the PDF and its two text extractions were scratch files of the 2026-10-23 realism-checks task, outside the repository).
- Read so far (by the project): the PDF text, 2026-10-23: Table 1 in full (age; continental crust area; modern-day crust area; exposed land area, in 10^6 km2, 0 to 520 Ma at 10 Myr steps) and the paragraph giving the hypsometric slope C = 0.0057 km per 10^6 km2. Values read: exposed land 149.00 at 0 Ma, 94.19 at 80 Ma, 66.95 at 510 Ma; continental crust 210.49 at 0 Ma. The shares in the ledger divide by 510.07 x 10^6 km2 (the project's arithmetic).
- Reference check: the title, venue and DOI are as printed on the fetched copy. The table's minus signs are a special character that breaks a naive number parse (five rows, 0, 10, 30, 180 and 210 Ma, were skipped by the first parse and read by the second).

### ScoteseWright2018
- Reference: Scotese, C. R. and Wright, N. (2018). PALEOMAP Paleodigital Elevation Models (PaleoDEMS) for the Phanerozoic. PALEOMAP Project (EarthByte data collection; report and 1 x 1 degree netCDF maps 1 to 88, 0 to 540 Ma).
- DOI or URL: https://www.earthbyte.org/paleodem-resource-scotese-and-wright-2018/ ; data https://www.earthbyte.org/webdav/ftp/Data_Collections/Scotese_Wright_2018_PaleoDEM/Scotese_Wright_2018_Maps_1-88_1degX1deg_PaleoDEMS_nc.zip
- Cited as: "Scotese and Wright 2018 PaleoDEMs" (the report's own citation line: "Scotese, C.R., and Wright, N., 2018. PALEOMAP Paleodigital Elevation Models (PaleoDEMS) for the Phanerozoic PALEOMAP Project").
- Ledger ids: RLC-003, RLC-012, RLC-014, RLC-020, RLC-040, RLC-041, RLC-051 (2026-10-23: the census's land measures run on Earth's own paleo-elevation maps)
- Access: open; Creative Commons Attribution 4.0 (the License.txt that comes in the data folder).
- Library: not kept (scratch copies in /tmp of the 2026-10-23 task; the data reproduce from the URL and tools/review/earth-paleodem.py).
- Read so far (by the project): the report's text, 2026-10-23, for its caveats: a "first draft", the highest elevations "likely overestimated", maps built in two phases that are not homogeneous. The data were used, not only read: 109 maps, 0 to 540 Ma every 5 Myr, run through the census's helpers by tools/review/earth-paleodem.py with a present-day SRTM15+ control (Tozer2019). The 0 Ma map and the control agree (land 0.276 against 0.291; largest landmass 0.156).
- Reference check: the report names itself the source of the 120 paleogeographic reconstructions (117 Phanerozoic, 3 Precambrian) and says the DEMs were produced in two phases; it gives no uncertainty per map. A land bridge of one 1 degree cell joins two landmasses, so the largest-landmass measure jumps between neighbouring maps (80 Ma 0.070, 240 Ma 0.363).

### Condie2015
- Reference: Condie, K., Pisarevsky, S. A., Korenaga, J. and Gardoll, S. (2015). Is the rate of supercontinent assembly changing with time? Precambrian Research 259:278-289. The title is `from-memory`; the authors, journal, volume and pages are as RLC-071 and PLT-157 record them.
- DOI or URL: not checked.
- Cited as: "Condie 2015, Precambrian Research 259" (collision frequency through time; a rate of about six collisions per 100 Myr within a range of one to twenty).
- Ledger ids: RLC-002, RLC-071, RLC-072, RLC-075 (2026-10-23: the collision rate as a realism check)
- Access: the abstract was reachable from repository pages; the full-text PDF host (people.earth.yale.edu) did not resolve from the laptop, curl and the fetch tool alike.
- Library: not kept.
- Read so far (by the project): abstract only, 2026-10-23: craton collisions peak at 1850 and 600 Ma, less at 1100 and 350 Ma, and are "very low" over 150 to 0 Ma; not related to supercontinent assembly. The figures "about one to 20 per 100 Myr, with an average of six" come from a search-engine extract recorded in PLT-157 and were NOT found in the pages read: RLC-072 carries them `cited-unverified`. The proposed M7 ceiling of 6 leans on them, so reading the full text is the first open item of the realism-checks page.
- Reference check: not done; no copy of the paper was read.

### Macdonald2019
- Reference: Macdonald, F. A., Swanson-Hysell, N. L., Park, Y., Lisiecki, L. and Jagoutz, O. (2019). Arc-continent collisions in the tropics set Earth's climate state. Science 364:181-184. The full author list is `from-memory`; the title, journal, volume and pages are as PLT-157 records them.
- DOI or URL: 10.1126/science.aav5300 (from memory, not checked); the analysis code and data are on GitHub as Swanson-Hysell-Group/Arc_Continent_Analysis (from memory, the repository was not opened).
- Cited as: "Macdonald et al. 2019" (arc-continent sutures per 100 Myr and the Phanerozoic peaks of total suture length).
- Ledger ids: PLT-157; RLC-073 (2026-10-23: the literature's collision counts are for a different object than the model's); PLT-508 (2026-10-23, V1 gap 2: the reference for collcoast's docking episodes); PLT-512 (2026-10-23, the docking lane: their published suture table counted by exhumation start, 104 sutures in 540 Myr, 39 in the last 100 Myr, the lengths per 100 Myr)
- Access: paywalled at Science; the copy that was read is the one PLT-157 records (the full text was read by the 2026-10-22 research agent and again in the first pass of the 2026-10-23 realism-checks task).
- Library: not kept.
- Read so far (by the project): in full: six peaks of total suture length in 540 Myr for the major orogenies (about 1.1 per 100 Myr) and 15 to 46 ophiolite-bearing arc-continent sutures per 100 Myr (Paleozoic 44 in 290 Myr, Mesozoic 32 in 185, Cenozoic 30 in 65). RLC-073 uses these only to say they count sutures, not the model's terranes of at least 3.64 million km2 that end in a weld, so they bound nothing.
- Reference check: not done by the realism-checks task.

### WikipediaRealismChecks2026
- Reference: Wikipedia (read 2026-10-23, tertiary sources and said to be): Last Glacial Maximum; Ice sheet; Greenhouse and icehouse Earth; Late Paleozoic icehouse; Caledonian orogeny; Variscan orogeny; Uralian orogeny; Cimmerian Plate; Alpine orogeny; Indian Plate; Arabian Plate. (The Indosinian orogeny page returned 404 and was not used.)
- DOI or URL: https://en.wikipedia.org/wiki/<page title> for each of the above.
- Cited as: "Wikipedia" in the realism-checks rows, never as the only source of a number that sets a band.
- Ledger ids: RLC-013, RLC-060, RLC-061, RLC-062, RLC-070, RLC-074 (2026-10-23)
- Access: open.
- Library: not kept.
- Read so far (by the project): the pages above, in full or in the sections named in the rows: the land under ice today and at the Last Glacial Maximum (25 to 32 percent of the land, by figure), the greenhouse and icehouse intervals of the Phanerozoic, and the dates of the main collisions the writer's from-memory count of M7 leans on. Zachos et al. 2001 was cited by these pages and not read.
- Reference check: the greenhouse and icehouse interval list is inconsistent inside its own page (RLC-062); no band edge rests on it.

## The lithosphere's stages 2 and 3 (PLT-475 to PLT-478), added 2026-10-23

The sources of stage 2 (rifted margins drawn by a stretching factor) and stage 3 (root removal) of the lithosphere's thermal state (gap 4; `lithosphere-thermal-state.md`). McKenzie 1978, Steckler & Watts 1978, Shi D. et al. 2020, Houseman & Molnar 1997 and England & Houseman 1989 are above; their ledger ids now include this section's rows. Read by a sub-agent of the gap-4 stages 2-3 agent on 2026-10-23, as each entry says; quotes as read, en dashes turned to hyphens. Titles marked "from memory" were not seen on the record and are unchecked. Numbers seen only in search-engine summaries were not used.

### ManatschalChenin2023
- Reference: Manatschal, G. and Chenin, P. (2023). A field guide to the fossil rifted margins in the Alps (the title from memory and the copy's file name, unchecked). Geological Field Trips and Maps 15(1.2).
- DOI or URL: 10.3301/GFT.2023.02; https://www.geologicalfieldtripsandmaps.com/fulltext/70/fossil-rifted-margins-in-the-alps.pdf
- Cited as: "Manatschal & Chenin 2023".
- Ledger ids: PLT-475
- Access: open.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: "the proximal domain where the top basement and Moho are sub-horizontal and parallel and where the crust is only a little thinned (30+-5 km); the necking domain where the crust is thinned from 30+-5 km to ca. 10 km; the hyperextended domain where the crust is thinned from ca. 10 km to 0 km"; "Based on a worldwide compilation of seismic sections, Chenin et al. (2017) proposed ... necking domains range between ca. 10 and 100 km with an average of 55 km; hyperextended domains between ca. 20 and 70 km with an average of 50 km; and exhumed mantle domains between 20 and 110 km with an average of 60 km"; a figure caption separates a "stretching phase (low-beta phase)" from the "early necking phase (onset of the high-beta phase)". Chenin et al. 2017 itself not read.

### Montadert1979
- Reference: Montadert, L., Roberts, D. G., De Charpal, O. and Guennoc, P. (1979). Rifting and subsidence of the northern continental margin of the Bay of Biscay (the title from memory). Initial Reports of the Deep Sea Drilling Project 48:1025-1060.
- DOI or URL: https://deepseadrilling.org/48/volume/dsdp48_54.pdf
- Cited as: "Montadert et al. 1979".
- Ledger ids: PLT-475
- Access: open.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23: "The thinning of the continental crust cannot be explained by the 10 to 15 per cent of extension estimated for the upper brittle part"; "The Moho discontinuity, 25 km deep in the vicinity of the shelf break, is 12 km deep in the lower part of the margin. In this area the ductile part of the crust (6.3 km/s) is only 3 km thick"; post-rift subsidence "decreases continuously from the oceanic/continental crust boundary (4000 m) to the shelf break. For each point of the margin, the subsidence versus time curve is an exponential, the time constant of which increases with depth"; "Only in the lower part of the margin is the constant similar to the one of the oceanic crust. There is, therefore, not a single law that characterizes the subsidence of a whole margin, but one law for each point". Figs. 31 and 32 seen on the page image (about 4,000 m at the boundary falling to about 1 km or less at the shelf edge over 200 km, read off the plot).

### Lang2026
- Reference: Lang, ten Brink and Makovsky (2026; initials and title not recorded). Communications Earth & Environment 7:548.
- DOI or URL: 10.1038/s43247-026-03742-2
- Cited as: "Lang, ten Brink & Makovsky 2026".
- Ledger ids: PLT-476
- Access: open.
- Library: not kept.
- Read so far (by the project): full text, 2026-10-23, the Baltimore Canyon Trough: "~500 km wide"; uniform stretching predicts sediment wherever the crust is under 34 km, but sediment "accumulated only where the crust is thinner than 25.5-31 km"; crust fully magmatic at 17.4 +- 0.36 km; a best-fit mantle to crust "beta ratio = 2.8"; thermal diffusivity rising "from 38% to 100% over 55 km" at the hinge.

### LePichonSibuet1981
- Reference: Le Pichon, X. and Sibuet, J.-C. (1981). Passive margins: a model of formation (the title from memory). Journal of Geophysical Research 86(B5):3708-3720.
- Cited as: "Le Pichon & Sibuet 1981".
- Ledger ids: none (lithosphere-thermal-state.md, stage 2)
- Access: paywalled; the abstract from Crossref.
- Library: not kept.
- Read so far (by the project): abstract, 2026-10-23: "The amount of brittle stretching observed in the upper 8 km of the prestretched continental crust reaches a maximum value of about 3. This large amount of thinning is comparable to the thinning of the whole continental crust observed by seismic refraction measurements".

### HousemanMcKenzieMolnar1981
- Reference: Houseman, G. A., McKenzie, D. P. and Molnar, P. (1981). Convective instability of a thickened boundary layer and its relevance for the thermal evolution of continental convergent belts (the title from memory). Journal of Geophysical Research 86(B7):6115-6132.
- DOI or URL: 10.1029/JB086iB07p06115
- Cited as: "Houseman, McKenzie & Molnar 1981".
- Ledger ids: PLT-477
- Access: paywalled; the abstract from Crossref.
- Library: not kept.
- Read so far (by the project): abstract, 2026-10-23: "When crust thickens during crustal shortening, the underlying mantle lithosphere must shorten and thicken also, causing the submersion of cold, dense material into the surrounding asthenosphere"; "the boundary layer is removed in times less than the duration of deformation in some collision zones (30-50 m.y.)"; "in some cases the entire mantle lithosphere may detach from the lower crust during crustal shortening".

### Conrad2000
- Reference: Conrad, C. P. (2000). Convective instability of thickening mantle lithosphere (the title from memory). Geophysical Journal International 143:52-70.
- Cited as: "Conrad 2000".
- Ledger ids: PLT-477
- Access: the abstract from OpenAlex.
- Library: not kept.
- Read so far (by the project): abstract, 2026-10-23: "at most only the lower 30 per cent of the lithosphere participates in the downwelling associated with this initial super-exponential growth event"; "The hottest 60 per cent of the mantle lithosphere could be removed in such an event, which would be followed by an influx of hot, buoyant asthenosphere that causes rapid surface uplift"; Tibet "shortened by about 50 per cent in ~30Myr and afterwards, at ~8Ma, experienced a period of rapid uplift".

### MolnarEnglandMartinod1993
- Reference: Molnar, P., England, P. and Martinod, J. (1993). Mantle dynamics, uplift of the Tibetan Plateau, and the Indian monsoon (the title from memory). Reviews of Geophysics 31(4):357-396.
- DOI or URL: 10.1029/93RG02030
- Cited as: "Molnar, England & Martinod 1993".
- Ledger ids: none (lithosphere-thermal-state.md, stage 3)
- Access: paywalled; the abstract from Crossref.
- Library: not kept.
- Read so far (by the project): abstract, 2026-10-23: "Convective removal of lower lithosphere beneath the Tibetan Plateau can account for a rapid increase in the mean elevation of the Tibetan Plateau of 1000 m or more in a few million years"; "uplift was rapid: approximately 1000 m to 2500 m in a few million years".

### KayKay1993
- Reference: Kay, R. W. and Kay, S. M. (1993). Delamination and delamination magmatism (the title from memory). Tectonophysics 219:177-189.
- DOI or URL: 10.1016/0040-1951(93)90295-U
- Cited as: "Kay & Kay 1993".
- Ledger ids: PLT-478
- Access: paywalled; the abstract from Semantic Scholar.
- Library: not kept.
- Read so far (by the project): abstract, 2026-10-23: "Crustal thickening that results from shortening may result in a crustal root that, due to phase changes, becomes denser than the underlying mantle lithosphere and should delaminate with it"; "a critical amount of shortening is required for the densifying effect of cooler temperature to counterbalance the effect of composition". No thickness threshold in the abstract.
## Plate boundary organisation (PLT-500 to PLT-507), added 2026-10-23

The sources of V1 gap 2 (build log 2026-10-23 "Gap 2"; plates.md, the section "Plate boundary organisation"). Read again for it: Bird 2003 (Table 3 and PB2002_steps.dat), Stern 2004, Crameri et al. 2020 and Gurnis et al. 2004 (above). Texts in the project's source library on the test VM (`/data/ban/deeptime-sources/<key>/`), not distributed.

### Gerya2012
- Reference: Gerya, T. (2012). Origin and models of oceanic transform faults. Tectonophysics 522-523:34-54.
- DOI or URL: 10.1016/j.tecto.2011.07.006
- Cited as: "Gerya 2012".
- Ledger ids: PLT-502, PLT-507
- Access: open copy (the author's ETH page).
- Library (the project's source library, not distributed): fetched 2026-10-23. Files: `Gerya2012/Gerya2012.pdf`, `Gerya2012.txt`.
- Read so far (by the project): full text, 2026-10-23. §1: "The perpendicularity of the ridges and the transform faults is so common that it is considered to be an 'intrinsic property' of the oceanic spreading process (Oldenburg and Brune, 1972). Tens to hundreds of kilometer long active transform fault sections connecting two spreading centers"; §4: transform faults' development "should react nearly instantaneously to changes in spreading direction (Menard and Atwater, 1968; ...)".

### Aghaei2014
- Reference: Aghaei, O., Nedimović, M. R., Carton, H., Carbotte, S. M., Canales, J. P. and Mutter, J. C. (2014). Crustal thickness and Moho character of the fast-spreading East Pacific Rise from 9°42'N to 9°57'N from poststack-migrated 3-D MCS data. Geochemistry, Geophysics, Geosystems 15:634-657.
- DOI or URL: 10.1002/2013GC005069
- Cited as: "Aghaei et al. 2014" (after Macdonald 1998).
- Ledger ids: PLT-502
- Access: open (AGU).
- Library (the project's source library, not distributed): fetched 2026-10-23. Files: `Aghaei2014/Aghaei2014.pdf`, `Aghaei2014.txt`.
- Read so far (by the project): the introduction, 2026-10-23: "First-order and second-order (large scale) ridge segments, bounded by transform faults and overlapping spreading centers (OSCs), respectively"; "First-order segments extend for 600 ± 300 km and persist for >5 × 10^6 years [e.g., Macdonald, 1998]. Second-order segments extend for 140 ± 90 km".

### MacdonaldFox1990
- Reference: Macdonald, K. C. and Fox, P. J. (1990). The mid-ocean ridge. Scientific American 262(6):72-79 (the volume, issue and pages from memory).
- DOI or URL: the text as kept on a course page (`Macdonald_SciAm.html`).
- Cited as: "Macdonald & Fox 1990".
- Ledger ids: PLT-502
- Access: the article's text, open on a course page.
- Library (the project's source library, not distributed): fetched 2026-10-23. File: `Macdonald_SciAm.html`.
- Read so far (by the project): full text, 2026-10-23: "First-order discontinuities offset the ridge segments by at least 20 kilometers and usually more than 50 kilometers"; "These discontinuities typically define segments from 200 to 800 kilometers long"; the transforms' traces run in the ridge's flanks "for hundreds to thousands of kilometers".

### AtwaterMacdonald1977
- Reference: Atwater, T. and Macdonald, K. C. (1977). Are spreading centers perpendicular to their transform faults? Nature 270:715-719.
- DOI or URL: 10.1038/270715a0
- Cited as: "Atwater & Macdonald 1977".
- Ledger ids: PLT-502
- Access: paywalled.
- Library (the project's source library, not distributed): the abstract page, fetched 2026-10-23. File: `AtwaterMacdonald1977.html`.
- Read so far (by the project): abstract, 2026-10-23: "Of eight segmented, slow-spreading centres (half-rate less than 3 cm yr−1) we did not find a case which was, in fact, perpendicular. All were 6–38° oblique"; "Fast- and some intermediate-rate spreading centres, on the other hand, seem to be perpendicular within the errors of measurements".

### Heidbach2018
- Reference: Heidbach, O., Rajabi, M., Cui, X., Fuchs, K., Müller, B., Reinecker, J., Reiter, K., Tingay, M., Wenzel, F., Xie, F., Ziegler, M. O., Zoback, M.-L. and Zoback, M. (2018). The World Stress Map database release 2016: Crustal stress pattern across scales. Tectonophysics 744:484-498.
- DOI or URL: 10.1016/j.tecto.2018.07.007 (post-print at GFZpublic).
- Cited as: "Heidbach et al. 2018".
- Ledger ids: PLT-504
- Access: open post-print.
- Library (the project's source library, not distributed): fetched 2026-10-23. Files: `Heidbach2018/Heidbach2018.pdf`, `Heidbach2018.txt`.
- Read so far (by the project): full text, 2026-10-23. §3: the maximum horizontal stress "is often sub-parallel to the direction of absolute plate motion indicating that the forces that drive plate motion also control the first-order intra-plate crustal stress pattern"; §1: the wavelength of the stress pattern "can be several thousand kilometres"; §2: the 2008 release "revealed that plate boundary forces are not enough to fully explain the crustal stress pattern".

### AudetBurgmann2011
- Reference: Audet, P. and Bürgmann, R. (2011). Dominant role of tectonic inheritance in supercontinent cycles. Nature Geoscience 4:184-187 (the title from memory).
- DOI or URL: 10.1038/ngeo1080
- Cited as: "Audet & Bürgmann 2011".
- Ledger ids: PLT-506
- Access: paywalled.
- Library (the project's source library, not distributed): the abstract page, fetched 2026-10-23. File: `Audet2011/ngeo1080.html`.
- Read so far (by the project): abstract, 2026-10-23: "effective elastic thickness is high in Archean cratons, but low in the surrounding Phanerozoic belts"; "directions of mechanical weakness align with large gradients in effective elastic thickness and with tectonic boundaries"; "during the supercontinent cycle, strain is concentrated at pre-existing zones of weakness".

### BuiterTorsvik2014
- Reference: Buiter, S. J. H. and Torsvik, T. H. (2014). A review of Wilson Cycle plate margins: a role for mantle plumes in continental break-up along sutures? Gondwana Research 26:627-653 (from memory).
- DOI or URL: not checked.
- Cited as: not cited (looked for as a source for PLT-506).
- Ledger ids: none
- Access: paywalled; no abstract or open copy found 2026-10-23.
- Library: not kept (search pages only, `BuiterTorsvik2014/`).
- Read so far (by the project): not read.

### Byerlee1978
- Reference: Byerlee, J. (1978). Friction of rocks. Pure and Applied Geophysics 116:615-626 (from memory).
- DOI or URL: not checked (10.1007/BF00876528 from memory).
- Cited as: "Byerlee's friction" (from memory).
- Ledger ids: PLT-506 (the exponent 2 of `rift_weakness`: a brittle strength growing linearly with depth integrates to a plate strength proportional to the square of its thickness)
- Access: not looked for.
- Library: not kept.
- Read so far (by the project): not read; the linear growth of frictional strength with normal stress, so with depth, is from memory.

### Matthews2016
- Reference: Matthews, K. J., Maloney, K. T., Zahirovic, S., Williams, S. E., Seton, M. and Müller, R. D. (2016). Global plate boundary evolution and kinematics since the late Paleozoic. Global and Planetary Change 146:226-250.
- DOI or URL: 10.1016/j.gloplacha.2016.10.002 (accepted manuscript open).
- Cited as: "Matthews et al. 2016".
- Ledger ids: PLT-507
- Access: open accepted manuscript.
- Library (the project's source library, not distributed): fetched 2026-10-23. Files: `Matthews2016/Matthews2016.pdf`, `Matthews2016.txt`.
- Read so far (by the project): full text, 2026-10-23. Abstract: "Except for during the period ~260–160 Ma, the number of plates (16–46) and ratio of 'large' plates (≥10^7.5 km2) to smaller plates (~2.7–6.6) are fairly similar to present-day values (46 and 6.6, respectively)"; the model's reorganisations at 270, 230 and 120 Ma, and "the well-recognized plate reorganisation at ~50 Ma" and "the 100 Ma reorganisation"; "Terrane collision is a key trigger of subduction zone reorganisations such as polarity" reversals.

### PhilipponCorti2016
- Reference: Philippon, M. and Corti, G. (2016). Obliquity along plate boundaries. Tectonophysics 693:171-182.
- DOI or URL: 10.1016/j.tecto.2016.05.033
- Cited as: "Philippon & Corti 2016" (plates.md only).
- Ledger ids: none (plates.md, the obliquity of Earth's boundaries)
- Access: paywalled (HAL holds a notice only).
- Library: not kept (the abstract from the HAL API).
- Read so far (by the project): abstract, 2026-10-23: "roughly only 8% of the plate boundaries total length shows a very low obliquity (ranging from 0 to 10 deg, sub-orthogonal to the plate displacement)".

## The front's length: one plate, one body (PLT-530 to PLT-533), added 2026-10-24

The sources of the front lane's diagnosis (plates.md "The front's length: why the plates are ragged", "What Earth does"). Bird 2003 is under `Bird2003_Table3` (read again the same day), Lonsdale 2005 under `Lonsdale2005` and Mallard et al. 2016 under `Mallard2016`.

### StockLee1994
- Reference: Stock, J. M. and Lee, J. (1994). Do microplates in subduction zones leave a geological record? Tectonics 13(6):1472-1487 (the title from memory; the running head reads "Microplates in subduction zones").
- DOI or URL: 10.1029/94TC01808; CaltechAUTHORS https://authors.library.caltech.edu/records/mdth9-4yp93/files/tect936.pdf
- Cited as: "Stock & Lee 1994".
- Ledger ids: PLT-532, PLT-534 (2026-10-24: "The fragmentation of a subducting plate creates a new triple junction, two limbs of which are along the trench"; the Cocos-Rivera-North America triple junction "has not moved much with respect to the North America plate since its inception because subduction of both plates has been nearly orthogonal to the trench", both checked in `tect936.txt`)
- Access: open copy (the authors' institutional repository).
- Library (the project's source library, not distributed): fetched 2026-10-24. Files: `StockLee1994/tect936.pdf` (a scan with a text layer), `tect936.txt` (pdftotext -layout).
- Read so far (by the project): full text, 2026-10-24 (the lane's research agent, then the quotes checked in the library's text). Introduction: "As a spreading center approaches a trench in a subduction setting, the intervening subducted plate may fragment into smaller pieces down to linear dimensions of ~100 km (microplates)"; "However, if a small fragment breaks off a large plate, then it may lose its slab-pull force and become driven entirely by different conditions." "Styles of Microplate Death": the Monterey, Arguello, Guadalupe and Magdalena plates "became attached to the Pacific plate when their ridge boundaries with the Pacific plate stopped spreading"; "None of these microplates became attached to the North America plate." Conclusions: "a very small microplate with insufficient slab-pull force may stagnate or attach itself mainly to the overriding plate".

### Gordon1998
- Reference: Gordon, R. G. (1998). The plate tectonic approximation: plate nonrigidity, diffuse plate boundaries, and global plate reconstructions. Annual Review of Earth and Planetary Sciences 26:615-642.
- DOI or URL: 10.1146/annurev.earth.26.1.615
- Cited as: "Gordon 1998".
- Ledger ids: PLT-532
- Access: paywalled; the full text was read from a copy on a third-party site (2026-10-24).
- Library: not kept.
- Read so far (by the project): full text, 2026-10-24 (the lane's research agent): "The central assumption of plate tectonics is that the plates are rigid."

### Wang2013
- Reference: Wang, Y., Forsyth, D. W., Rau, C. J., Carriero, N., Schmandt, B., Gaherty, J. B. and Savage, B. (2013). Fossil slabs attached to unsubducted fragments of the Farallon plate. Proceedings of the National Academy of Sciences 110(14):5342-5346 (authors, title and pages from memory).
- DOI or URL: 10.1073/pnas.1214880110 (open access).
- Cited as: "Wang et al. 2013".
- Ledger ids: PLT-532, PLT-534 (2026-10-24)
- Access: open.
- Library (the project's source library, not distributed): fetched 2026-10-24 from an authors' institutional copy (the publisher's page had returned a page shell). Files: `Wang2013/wang.pdf`, `wang.txt`.
- Read so far (by the project): full text, 2026-10-24 (the lane's research agent): "Some fragments were incorporated into the Pacific plate as subduction and spreading ceased." For PLT-534 (checked in `wang.txt`): "Some fragments of the Farallon plate still exist as separate plates subducting beneath North America, including the Cocos and Juan de Fuca plates, which subsequently further fragmented generating the actively subducting Rivera, Gorda, and Explorer microplates" (p. 5342); high-velocity anomalies "which we interpret as the fossil slabs still attached to the remnant oceanic microplates" (p. 5342); "there is no high-velocity anomaly beneath northern Baja California where the spreading center intersected the trench and a slab gap formed" (p. 5343).

### Nicholson1994
- Reference: Nicholson, C., Sorlien, C. C., Atwater, T., Crowell, J. C. and Luyendyk, B. P. (1994). Microplate capture, rotation of the western Transverse Ranges, and initiation of the San Andreas transform as a low-angle fault system. Geology 22:491-495 (authors and title from memory).
- DOI or URL: not checked.
- Cited as: "Nicholson et al. 1994".
- Ledger ids: PLT-532
- Access: paywalled.
- Library: not kept.
- Read so far (by the project): abstract only, 2026-10-24 (the lane's research agent): the Monterey microplate captured by the Pacific plate at about 20 Ma.

## A boundary belongs to the lithosphere on its two sides (PLT-534), added 2026-10-24

The sources of the front lane's inheritance law (plates.md "Step 3b"). Stock & Lee 1994 and Wang et al. 2013 are above.

### Ishizuka2010
- Reference: Ishizuka, O., Yuasa, M., Tamura, Y., Shukuno, H., Stern, R. J., Naka, J., Joshima, M. and Taylor, R. N. (2010). Migrating shoshonitic magmatism tracks Izu-Bonin-Mariana intra-oceanic arc rift propagation. Earth and Planetary Science Letters 294:111-122 (title, authors and pages read on the copy's first page).
- DOI or URL: not checked.
- Cited as: "Ishizuka et al. 2010".
- Ledger ids: PLT-534
- Access: paywalled; an open copy on a co-author's institutional site (utdallas.edu).
- Library (the project's source library, not distributed): fetched 2026-10-24. Files: `Ishizuka2010/ishizuka2010.pdf`, `ishizuka2010.txt`.
- Read so far (by the project): full text, 2026-10-24 (the lane's research agent; the quote checked in the library's text): section 2, "The West Mariana Ridge (WMR) is a remnant arc (Karig, 1972) that separated from the magmatic arc by opening of the Mariana Trough."

### SternFouchKlemperer2003
- Reference: Stern, R. J., Fouch, M. J. and Klemperer, S. L. (2003). An overview of the Izu-Bonin-Mariana subduction factory. In: Eiler, J. (ed.), Inside the Subduction Factory, AGU Geophysical Monograph 138:175-222 (the volume, editor and pages from memory; the copy read is the authors' preprint, titled "An Overview of the Izu-Bonin-Mariana Subduction Factory").
- DOI or URL: not checked.
- Cited as: "Stern, Fouch & Klemperer 2003".
- Ledger ids: PLT-534
- Access: an open preprint.
- Library (the project's source library, not distributed): fetched 2026-10-24. Files: `Stern2003IBM/stern2003.pdf`, `stern2003.txt`.
- Read so far (by the project): full text, 2026-10-24 (the lane's research agent; the quote checked in the library's text): the Mariana Trough "is bounded to the east by the active Mariana arc, to the west by the remnant arc of the West Mariana Ridge [Karig, 1972], and to the south by the Challenger Deep part of the Mariana Trench"; the Palau-Kyushu Ridge is "arc crust rifted from the IBM by formation of the Shikoku and Parece-Vela basins".

### Karig1971
- Reference: Karig, D. E. (1971). Origin and development of marginal basins in the western Pacific. Journal of Geophysical Research 76(11):2542-2561.
- DOI or URL: 10.1029/JB076i011p02542
- Cited as: "Karig 1971".
- Ledger ids: PLT-534
- Access: paywalled (abstract only, through OpenAlex).
- Library: not kept.
- Read so far (by the project): abstract only, 2026-10-24 (the lane's research agent): "the portion of the Benioff zone shallower than 150 km, and lying between the volcanic chain and the trench, remains relatively undeformed along the trend of arc systems"; "as the inter-arc basin opens, the trench is forced to migrate and the Benioff zone is flattened, except in areas where crust can be consumed in a second trench behind the extensional zone."

### TaylorKarner1983
- Reference: Taylor, B. and Karner, G. D. (1983). On the evolution of marginal basins. Reviews of Geophysics 21(8):1727-1741 (the last page from memory).
- DOI or URL: 10.1029/RG021i008p01727
- Cited as: "Taylor & Karner 1983".
- Ledger ids: PLT-534
- Access: paywalled (abstract only).
- Library: not kept.
- Read so far (by the project): abstract only, 2026-10-24 (the lane's research agent): "(5) rifting of the volcanic arc may occur on either side of the line of active volcanoes (±50 km) and may vary from one side to the other along strike; (6) a remnant arc is not always developed".

### Faccenna2017
- Reference: Faccenna, C., Oncken, O., Holt, A. F. and Becker, T. W. (2017). Initiation of the Andean orogeny by lower mantle subduction. Earth and Planetary Science Letters 463:189-201 (authors from memory).
- DOI or URL: not checked.
- Cited as: "Faccenna et al. 2017".
- Ledger ids: PLT-534
- Access: an open preprint (the University of Texas Institute for Geophysics).
- Library (the project's source library, not distributed): fetched 2026-10-24. Files: `Faccenna2017/f.pdf`, `f.txt`.
- Read so far (by the project): full text, 2026-10-24 (the lane's research agent): section 3, "At ~130 Ma, the South Atlantic ocean basin started opening, with the separation of South America from Africa"; the introduction, a model "unable to explain why the Nazca subduction system rolled back during the opening of the Southern Atlantic for almost ~100 Myr". That the Andean trench stays with South America because only it borders the Nazca plate is PLT-534's reading, not the paper's (from-memory).

## The outline between the samples (PLT-536), added 2026-10-24

The sources of the front lane's level-set membership (plates.md "Step 3c").

### OsherSethian1988
- Reference: Osher, S. and Sethian, J. A. (1988). Fronts propagating with curvature-dependent speed: algorithms based on Hamilton-Jacobi formulations. Journal of Computational Physics 79(1):12-49 (the volume and pages from memory; the title and authors read on the copy's first page).
- DOI or URL: 10.1016/0021-9991(88)90002-2; the authors' copy at https://math.berkeley.edu/~sethian/Papers/sethian.osher.88.pdf
- Cited as: "Osher & Sethian 1988".
- Ledger ids: PLT-536
- Access: an open copy (the author's page).
- Library (the project's source library, not distributed): fetched 2026-10-24. Files: `OsherSethian1988/os88.pdf`, `os88.txt` (pdftotext -layout).
- Read so far (by the project): full text, 2026-10-24 (the front lane). Abstract: "We devise new numerical algorithms, called PSC algorithms, for following fronts propagating with curvature-dependent speed ... the front can also be passively advected by an underlying flow ... The algorithms handle topological merging and breaking naturally"; "By viewing the surface as a level set, topological complexities and changes in the moving front are handled naturally"; section III.D: "we initialize ψ by taking ψ(x , 0) = 1 ± d 2, where d is the distance from the point x to the initial surface, and the plus (minus) sign is chosen if x is inside (outside) the initial surface".

### SussmanSmerekaOsher1994
- Reference: Sussman, M., Smereka, P. and Osher, S. (1994). A level set approach for computing solutions to incompressible two-phase flow. Journal of Computational Physics 114(1):146-159 (authors, title, volume and pages from memory).
- DOI or URL: 10.1006/jcph.1994.1155
- Cited as: "Sussman, Smereka & Osher 1994".
- Ledger ids: PLT-536
- Access: paywalled; the UCLA CAM report copy did not answer (2026-10-24).
- Library: not kept.
- Read so far (by the project): abstract only, 2026-10-24 (the OSTI record): "The interface between the two fluids is considered to be sharp and is described as the zero level set of a smooth function." That the level-set function is kept a signed distance by reinitialising it from its zero level set, which PLT-536 does after every resampling, is the paper's method as the lane remembers it (from-memory).
