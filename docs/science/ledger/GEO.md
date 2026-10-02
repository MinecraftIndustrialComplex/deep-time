# Ledger GEO: Geothermal features

Part of the claims ledger ([claims.md](../claims.md): how to read the columns and the status marks). Feature page: [geothermal-features.md](../geothermal-features.md). Extraction notes with flags: [extraction-notes/SCL.md](../extraction-notes/SCL.md).

Rows: 0. By status: .

Extracted from the code as it stood on 2026-10-01/02 by a stage-1 agent. Nothing here has been checked against its source. Paths are relative to the repository root; line numbers are left out on purpose.

| id | feature | what the code does | where | value or equation as coded | units | source cited | citation written in | status |
|---|---|---|---|---|---|---|---|---|

Note: hot springs, geysers and fumaroles are not implemented on this branch. A grep of `crates` and `neoforge/src/main` for `geotherm`, `hot spring`, `geyser`, `fumarole` finds no feature code; the only hits are the conductive geothermal gradient used for rock burial temperature and metamorphism (listed in `earth/SCL.md`, subsection "Geothermal (GEO)" under "Simplifications and game choices"), a comment in `crates/deeptime-core/src/climate/ebm.rs` about the geothermal flux, and two data files that mention the words in prose. The feature is only planned: `docs/deeptime-implementation-plan.md` P2-14 "Geothermal features" (Ben, decisions log 2026-10-01, answer "Geothermal features" among four picked ideas from `docs/research/notes/geologist_youtuber_feedback.md`, planned as P2-12 to P2-15, "not started now"). A later log entry (2026-10-01, "Start independent Phase 2 items (impacts, geothermal) now") says geothermal was started on its own branch, to be merged after the v1 work lands; that branch is not this one.
