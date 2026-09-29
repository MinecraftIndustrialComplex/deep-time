# Workbench: 3D area preview (P1-20 stage 2)

"Preview this box of this world" without launching Minecraft: the **real generator**, run headless
on the Mac (Deep Time's density, strata pass, deposits, and the pack's surface rules and ores exactly
as a player's world gets them), one box exported as a compact voxel file, and a browser viewer that
shows it in 3D and explains a clicked block (rock horizon, deposit body, the site's "why here" chain).

| Piece | Where |
|---|---|
| Voxel export | `deeptime-inspect export-area` (`crates/deeptime-inspect/src/area.rs` documents the `.dtva` format) |
| Job runner + contract | `tools/workbench/preview.py` (Mac side, standard library only) |
| From the laptop | `tools/workbench/preview-area.sh` (runs `preview.py run` through `tools/remote/mac.sh --server`, pulls the job back) |
| Viewer | `tools/review/pages/area3d/` (`index.html` + `area3d.js`, three.js from jsDelivr) |

## From the laptop

```sh
tools/workbench/preview-area.sh --name porphyry --profile standalone \
    --box -5698,-1158,-5442,-902 \
    --preset crates/deeptime-core/tests/presets/deposits_world_16k.json --length-myr 500
# → smoke/workbench/jobs/porphyry/{status.json, area-0.dtva.gz, probes.jsonl, server.log, …}
```

Options: `--profile standalone|mic|mic-ev|terrablender|ev`, `--size 16384|32768`, `--box x0,z0,x1,z1`
(repeatable; x1, z1 exclusive; at most 256 × 256 unless `--max-side`), `--y lo,hi` (default the whole
build range −352..639), one world source (`--synth <kind> [--strata placeholder|every-class]`,
`--preset <name|file> [--start S] [--length-myr N]`, `--world-data <dir>` (a run's bake),
`--test-terrain <kind>`), `--datapack <dir>` (repeatable; MIC profiles add the MIC rock tags
themselves unless `--no-mic-rock-tags`), `--seed N`, `--keep-world`, `--install` (re-run
`tools/smoke/install.sh <profile>` first; needed after Java changes, since a server set carries the
Deep Time jar it was installed with), `--pack` (sync the MIC pack to the Mac; MIC profiles).

To look at an export, serve the viewer folder and pass `?src=`:
`python3 tools/workbench/preview.py serve` (on the Mac: the contract below plus the viewer at
`/area3d/`), or copy `index.html`, `area3d.js` and the `.dtva.gz` into one folder and open
`index.html?src=area-0.dtva.gz` over HTTP (the Open… button and drag and drop work too).

## What a job does

1. **install** only when `smoke/<profile>/install.json` is missing, or with `--install`;
2. **prepare** the job's own world `<job>/world` (synthetic world data, a copied world-data folder, or
   a `config/deeptime/server.json` preset the server generates from on its first start; datapacks);
   the profile's `server.properties` and `config/deeptime/server.json` are restored afterwards;
3. **boot** the profile's server with `--universe <job> --world world`, so the smoke worlds are never
   touched; takes the Mac's single server lock (`deeptime-cache/server.lock`, the one `mac.sh
   --server` takes; `--lock-held` when the caller already holds it);
4. **generate** each box with Chunky: one rectangle per box, a chunk of margin round it so every box
   chunk is `full`; a box across the seam is split into its two pieces in the canonical span;
5. **probe** Deep Time: `/deeptime probe tw|palette|claims|column`, `sites near` round each box,
   `why <id>` and `probe site <centre>` for every site whose bounds meet a box → `probes.jsonl`;
6. **stop** (`save-all flush`, `stop`) and release the lock;
7. **export** `area-<i>.dtva.gz` per box with `--verify` (decoded again and compared block by block
   with the region files); the world is then deleted unless `--keep-world`.

Measured on the M6 (2026-09-29, standalone, deposit world, 256 × 256 × 992): boot 12.7 s, first-start
generation 3.3 s, Chunky 361 chunks 9.1 s, probes 0.1 s, stop 2.9 s, export 0.5 s, whole job 44 s;
0.94 MB. STATUS.md's P1-20 stage 2 entry has the seam and MIC numbers.

## The contract (for the workbench server)

The workbench server (stage 1, on the Mac, tailnet only) mounts this. Two equivalent ways:

- **In process:** `import preview` (add `tools/workbench` to `sys.path`) and route
  `/api/preview…` to `preview.api(method, path, body_bytes)`, which returns
  `(http_status, content_type, body_bytes)`. Serve `tools/review/pages/area3d/` at `/area3d/`.
- **CLI:** `python3 tools/workbench/preview.py submit <spec options>` prints the job's first status
  (JSON) and starts the runner detached; `preview.py status <job>` and `preview.py list` print JSON.

### HTTP

| Request | Answer |
|---|---|
| `POST /api/preview` with a spec (JSON) | `202` + the job's status (`job`, `state: "queued"`, `status_url`); `400 {"error"}` for a bad spec |
| `GET /api/preview` | `200` + every job's status, newest first |
| `GET /api/preview/<job>` | `200` + `status.json`, `404` if unknown |
| `GET /api/preview/<job>/<file>` | the job's files: `area-0.dtva.gz`, `probes.jsonl`, `server.log`, `runner.log`, `spec.json` |

The panel shows the export with `/area3d/?src=/api/preview/<job>/area-0.dtva.gz`, or in its own page
through `window.Area3D.load(urlOrArrayBuffer, label)` after loading `area3d.js`.

### Spec

```json
{
  "profile": "standalone",
  "size": 16384,
  "boxes": [[-5698, -1158, -5442, -902]],
  "y": [-352, 639],
  "world": {
    "preset": "crates/deeptime-core/tests/presets/deposits_world_16k.json",
    "start": "supercontinent", "length_myr": 500,
    "datapacks": [], "seed": null
  },
  "keep_world": false
}
```

`box` (one) may stand for `boxes`. `world` needs exactly one of `synth` (+ `strata`), `preset`
(+ `start`, `length_myr`, `memory_cap_gib`), `world_data` (a folder with `manifest.json`: a live run's
bake) or `test_terrain`. Paths must lie under the repo, the jobs folder or `…/mic` (or
`DEEPTIME_WORKBENCH_ROOTS`). Boxes: at most 256 × 256, |z| ≤ C/4, x anything (folded).

### Status

`status.json`: `job`, `state` (`queued` → `waiting` (for the server lock) → `installing` →
`preparing` → `booting` → `generating` → `probing` → `stopping` → `exporting` → `done` | `failed`),
`message`, `created`, `updated`, `finished`, `error`, `spec`, `install` (profile, NeoForge, pack,
jar), `timings` (`boot_s`, `first_start_generation_s`, `chunky` per piece, `probes_s`, `stop_s`,
`total_s`), `exports` (`box`, `file`, `url` = `<job>/<file>`, `bytes`, `summary` = export-area's JSON:
stats, bodies, sites, verify).

Jobs live in `smoke/workbench/jobs/` (gitignored; `DEEPTIME_WORKBENCH_JOBS` moves them). One job
boots a server at a time; concurrent jobs queue on the lock.

## Later: the fast column path (after P1-08)

Today every answer comes from a generated world. Once P1-08 lands, `DeepTimeNative.sampleColumn`
(and `deeptime inspect-column`) answer a single column from the bake in milliseconds, before any
chunk exists. It plugs in at two places:
- `preview.py api()`: add `GET /api/column?world=<bake>&x=&z=` that runs `deeptime inspect-column`
  and returns the strata stack, the bodies and sites in the column and the density's surface;
- the viewer's `describe()` (`area3d.js`): when a click lands outside every exported box, or when
  the workbench shows a map without a 3D box, fetch `/api/column` and render the same panel from it.
The export stays the ground truth for what the game actually places (surface rules, carvers, other
mods' features); the column path is the fast preview of what Deep Time decides.
