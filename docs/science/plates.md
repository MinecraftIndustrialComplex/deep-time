# Plate tectonics

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The planet's outer shell is split into rigid plates that move, pull apart at ridges, sink under one another at subduction zones (where one plate dives beneath another), and collide to raise mountain belts. Today's code moves plates by speed rules taken largely from a computer-graphics paper (Cortial and others 2019) and tuned by hand; it also makes ocean floor that deepens with age, hotspots and large igneous provinces, and sets the crust's thickness and height. **This model is about to be replaced** by a balance of forces (slab pull, ridge push, drag, collision resistance; `docs/design/plate-force-balance.md`, chosen 2026-10-02), with collisions that shorten crust instead of lifting it in one step (`docs/design/plate-collisions-and-welding.md`). The ledger rows describe the code as it is today and are marked `pending-rewrite`; the constants of the new design are ledgered as `not-yet-coded`.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [PLT](ledger/PLT.md) | plate setup | PLT-001 to PLT-021, PLT-203, PLT-205, PLT-209 to PLT-211 (26) |
| [PLT](ledger/PLT.md) | plate motion | PLT-022 to PLT-028 (7) |
| [PLT](ledger/PLT.md) | subduction | PLT-029 to PLT-067, PLT-206 to PLT-207 (41) |
| [PLT](ledger/PLT.md) | collisions | PLT-068 to PLT-081, PLT-213 to PLT-218 (20) |
| [PLT](ledger/PLT.md) | rifting | PLT-082 to PLT-091 (10) |
| [PLT](ledger/PLT.md) | ridges and ocean crust | PLT-092 to PLT-107, PLT-204, PLT-212 (18) |
| [PLT](ledger/PLT.md) | hotspots | PLT-108 to PLT-120 (13) |
| [PLT](ledger/PLT.md) | crust and isostasy (plates) | PLT-121 to PLT-142, PLT-208 (23) |
| [PLT](ledger/PLT.md) | plate-check targets | PLT-143 to PLT-173 (31) |
| [PLT](ledger/PLT.md) | designed, not yet coded | PLT-174 to PLT-202 (29) |

Rows by status: `cited-unverified` 51, `from-memory` 6, `abstract-only` 21, `calibrated` 19, `game-choice` 67, `uncited` 54; `pending-rewrite` 183; `not-yet-coded` 29.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/PLT.md](extraction-notes/PLT.md).
- Designs: `docs/design/plate-force-balance.md`, `docs/design/plate-collisions-and-welding.md`, `docs/design/plate-check-baseline.md` (the measured state of today's model).
- Accuracy review of one run: `docs/research/notes/run-review-1a0f969d358.md`.
- Work package in the verification plan: WP-13 (deferred until the rewrite lands) ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- Since 2026-10-02 (plate-motion task 2) a contact keeps a collision state with its real shortening: the convergence across the collision front, integrated (ledger PLT-213 to PLT-217). The built-in presets still transfer terranes and weld plates on the older count of relative speed times time (PLT-068, PLT-218), so the worlds are unchanged; the measurements behind that choice are in the build log entry of that date.

- The accuracy review calls today's model "a convincing cartoon of plate tectonics, not yet an accurate simulation": no Wilson cycle (rifted pieces weld back after a median 18 Myr), one plate over half the planet in about half of all frames, collisions that raise round blobs in one step.
- The force-balance design could not confirm three figures: the gross size of slab pull (from textbook examples, not found in Turcotte & Schubert itself), a factor of 8 for continental drag attributed to Forsyth & Uyeda, and the wedge-taper reference (Davis, Suppe & Dahlen 1983, cited from memory).
