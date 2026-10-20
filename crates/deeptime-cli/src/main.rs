// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime`: the standalone driver for every simulation stage (plan §9.3).

mod alloc;
mod amplify_view;
mod climate_dem;
mod deposit_audit;
mod dump;
mod erosion_audit;
mod heightmap;
mod hydro;
mod margins_view;
mod parity;
mod plate_check;
mod project;
mod provinces;
mod run;
mod satellite_view;
mod strata_check;
mod structure_sections;
mod structure_view;
mod synth;
mod volcano_view;
mod watch;
mod world_map;

use std::net::IpAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use deeptime_core::compute::ComputeMode;

#[derive(Parser)]
#[command(
    name = "deeptime",
    version,
    about = "Deep Time: planets with a geologic history"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Print the tool and world-data format versions.
    Version,
    /// Run a whole planet from a preset and write its world-data folder (what the game writes).
    Run {
        /// A built-in preset name, a preset JSON file or a share code (DT1.…).
        #[arg(long)]
        preset: String,
        /// Timeline start configuration (supercontinent, scattered_continents, random): sets
        /// timeline.start, as config/deeptime/server.json does on a server.
        #[arg(long)]
        start: Option<String>,
        /// Timeline length in millions of years: sets timeline.length_myr.
        #[arg(long)]
        length_myr: Option<f64>,
        /// Content bundle JSON (rock classes, deposit types; plan §5.4). Optional for now.
        #[arg(long)]
        content: Option<PathBuf>,
        /// A pack placement table (`crates/deeptime-core/data/climate_mapping/packs/<id>.json`)
        /// to bake with, as the game does when that biome mod is installed; repeatable, applied
        /// in order.
        #[arg(long)]
        placement_patch: Vec<PathBuf>,
        /// World-data folder to write (`<world>/deeptime`).
        #[arg(long)]
        out: PathBuf,
        /// Also write an HTML viewer bundle here.
        #[arg(long)]
        viewer: Option<PathBuf>,
        /// Also write a checkpoint after every stage into <out>/checkpoints/.
        #[arg(long)]
        checkpoints: bool,
        /// Where the run stops, Myr after its start (timeline.stop_myr: less than the length
        /// stops it early, more continues it past the present).
        #[arg(long)]
        stop_myr: Option<f64>,
        /// Write epoch checkpoints into <out>/epochs/, so `deeptime resume` can bake another
        /// keyframe's world, stop the run earlier or continue it (docs/design/
        /// playability-steering.md).
        #[arg(long)]
        epochs: bool,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Run a finished (or stopped) run's folder again to another stop, from its epoch
    /// checkpoints (`run --epochs`): an epoch picked, a run stopped early or continued. Only the
    /// steps after the checkpoint that serves the stop run again; the world is the world of a run
    /// planned to that stop.
    Resume {
        /// The run's folder (it holds epochs/).
        #[arg(long)]
        out: PathBuf,
        /// The new stop, Myr after the start (default: the run's own).
        #[arg(long)]
        stop_myr: Option<f64>,
        /// Keep writing epoch checkpoints (on by default; --no-epochs turns them off).
        #[arg(long)]
        no_epochs: bool,
        /// Also write an HTML viewer bundle here.
        #[arg(long)]
        viewer: Option<PathBuf>,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// The browser workbench (P1-20): serve the editor, the runs and a live view of each run (map,
    /// globe, layers and events every step; pause, step, rewind, speed, follow live, compare two
    /// runs). Runs go one at a time; finished ones are kept under --runs. Serves loopback or a
    /// Tailscale address only, never the LAN; tools/viewer/mac-watch.sh runs it on the Mac and
    /// puts it on the tailnet. --mem-cap is the default memory cap of its runs (the page's Memory
    /// field sets another for one run); a run whose estimate is over its cap is refused before
    /// it starts.
    Watch {
        /// A run to start right away (a built-in preset name, a preset JSON file or a share code);
        /// the page starts others.
        #[arg(long)]
        preset: Option<String>,
        /// Timeline start configuration for --preset (supercontinent, scattered_continents,
        /// random).
        #[arg(long)]
        start: Option<String>,
        /// Timeline length for --preset, in millions of years.
        #[arg(long)]
        length_myr: Option<f64>,
        /// Address to serve on: loopback (default) or the machine's Tailscale address.
        #[arg(long, default_value = "127.0.0.1")]
        bind: IpAddr,
        /// Port to serve on.
        #[arg(long, default_value_t = watch::DEFAULT_PORT)]
        port: u16,
        /// Folder the runs are kept in (a folder per run; default: a temporary folder).
        #[arg(long)]
        runs: Option<PathBuf>,
        /// Finished runs kept; the oldest beyond this are deleted.
        #[arg(long, default_value_t = watch::DEFAULT_KEEP)]
        keep: usize,
        /// A frame every this many Myr (default: after every step).
        #[arg(long)]
        every_myr: Option<f64>,
        /// Stop once nothing runs or waits (after --preset's run; default: serve until stopped).
        #[arg(long)]
        exit_when_done: bool,
        /// Stage 2 (P1-20): tools/workbench/preview.py of a checkout with smoke server sets, which
        /// then answers the 3D preview, click-to-inspect and /area3d/ routes (default: 501).
        #[arg(long)]
        preview: Option<PathBuf>,
        /// The Python that runs --preview (stdlib-only; /usr/bin/python3 on the Mac).
        #[arg(long, default_value = "python3")]
        python: PathBuf,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Run one stage: from a checkpoint (--in) or from a preset (--preset; the stages before it
    /// run first), writing the checkpoint after it (--out).
    Stage {
        /// The stage's name in the preset's pipeline.
        name: String,
        /// Checkpoint to start from; its next stage must be <name>.
        #[arg(long = "in")]
        input: Option<PathBuf>,
        /// Preset to start from instead of a checkpoint (name, file or share code).
        #[arg(long)]
        preset: Option<String>,
        /// Timeline start configuration for --preset (supercontinent, scattered_continents,
        /// random).
        #[arg(long)]
        start: Option<String>,
        /// Timeline length for --preset, in millions of years.
        #[arg(long)]
        length_myr: Option<f64>,
        /// Stop after this many steps of the stage and write a mid-stage checkpoint.
        #[arg(long)]
        max_steps: Option<u64>,
        /// Checkpoint to write after the stage.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Also write an HTML viewer bundle (the stage's keyframes, all events and sites) here.
        #[arg(long)]
        viewer: Option<PathBuf>,
        /// Only these keyframe layers go into the viewer bundle (comma-separated field names,
        /// e.g. `elevation,climate/koppen`; default all), to keep a review page small. The
        /// structure stage's map layers and sections come only when a `structure…` name is given.
        #[arg(long, value_delimiter = ',')]
        viewer_layers: Vec<String>,
        /// Folder for the stage's keyframes (default <out>.work, or a temporary folder).
        #[arg(long)]
        work: Option<PathBuf>,
        /// When the stage finishes the run, bake the world data into the work folder (what `run`
        /// writes), so a later stage's parameters can be tried on the game's terrain.
        #[arg(long)]
        bake: bool,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Run a preset once per thread count with per-stage timings and peak native memory.
    Bench {
        /// A built-in preset name, a preset JSON file or a share code.
        #[arg(long)]
        preset: String,
        /// Timeline start configuration (supercontinent, scattered_continents, random).
        #[arg(long)]
        start: Option<String>,
        /// Timeline length in millions of years.
        #[arg(long)]
        length_myr: Option<f64>,
        /// Thread counts, comma-separated.
        #[arg(long, value_delimiter = ',', default_value = "1,2,4,8")]
        threads: Vec<usize>,
        /// Where to write bench.json.
        #[arg(long, default_value = "bench.json")]
        out: PathBuf,
        /// Also write a timing page here.
        #[arg(long)]
        viewer: Option<PathBuf>,
        /// Folder for the runs' world data (default: a temporary folder, removed afterwards).
        #[arg(long)]
        work: Option<PathBuf>,
        /// Native memory cap in bytes (0 = 4 GiB).
        #[arg(long, default_value_t = 0)]
        mem_cap: u64,
        /// Print DEBUG log lines too.
        #[arg(long)]
        verbose: bool,
        /// Where the kernels with a GPU version run (auto, cpu, gpu; default DEEPTIME_COMPUTE,
        /// else auto).
        #[arg(long, value_parser = parse_compute)]
        compute: Option<ComputeMode>,
    },
    /// One climate snapshot (P1-05) on a given relief — e.g. Earth's — to check the climate
    /// against a planet whose Köppen map is known. The raster is raw little-endian f32 metres,
    /// W × W/2 cells, row 0 at the north pole, column i at longitude 360°·i/W.
    ClimateDem {
        /// The relief raster.
        #[arg(long)]
        dem: PathBuf,
        /// Cells around (W); the raster has W/2 rows.
        #[arg(long)]
        width: u32,
        /// A preset whose `climate` and `planet` sections apply (default: the defaults).
        #[arg(long)]
        preset: Option<String>,
        /// The planet's circumference in km (Earth: 40,030).
        #[arg(long, default_value_t = 40030.0)]
        circumference_km: f64,
        /// Also write an HTML viewer bundle (Köppen, temperature, precipitation) here.
        #[arg(long)]
        viewer: Option<PathBuf>,
        /// Also write the snapshot's fields as raw little-endian f32 rasters here (the twelve
        /// monthly sea-level temperatures and precipitations, the annual fields, the sea ice) with
        /// meta.json (the per-cell climate's figures): for tools/validation/climate_vs_obs.py.
        #[arg(long)]
        dump: Option<PathBuf>,
    },
    /// The surface kernels (P1-04) on a plain raster, for the fastscapelib oracle
    /// (tools/validation/fastscape_compare.py): periodic in x, the first and last rows fixed at
    /// 0 m, uniform uplift. Writes initial.f32, final.f32, area.f32 (row-major little-endian) and
    /// meta.json into --out.
    Lem {
        /// Cells per side (W = H), unless --height is given.
        #[arg(long, default_value_t = 256)]
        grid: usize,
        /// Rows (H), default = --grid.
        #[arg(long)]
        height: Option<usize>,
        /// Cell size, metres.
        #[arg(long, default_value_t = 4800.0)]
        dx: f64,
        /// Time steps.
        #[arg(long, default_value_t = 200)]
        steps: usize,
        /// Time step, years.
        #[arg(long, default_value_t = 1.0e5)]
        dt_yr: f64,
        /// Stream-power K, m^(1−2m)/yr.
        #[arg(long, default_value_t = 2.0e-6)]
        k: f64,
        /// Area exponent m.
        #[arg(long, default_value_t = 0.5)]
        m: f64,
        /// Slope exponent n.
        #[arg(long, default_value_t = 1.0)]
        n: f64,
        /// Yuan et al.'s deposition coefficient G.
        #[arg(long, default_value_t = 0.0)]
        g: f64,
        /// Uniform uplift, mm/yr.
        #[arg(long, default_value_t = 1.0)]
        uplift_mm_yr: f64,
        /// Linear diffusivity, m²/yr (0 = none).
        #[arg(long, default_value_t = 0.0)]
        kd: f64,
        /// Seed of the initial noise.
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Output folder.
        #[arg(long)]
        out: PathBuf,
    },
    /// Built-in presets and share codes (plan §5.1).
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },
    /// Write a synthetic world-data folder (Phase 0 and the GameTests).
    Synth {
        /// Which synthetic world.
        #[arg(long, value_enum)]
        kind: SynthKind,
        /// Which horizon stacks the world carries.
        #[arg(long, value_enum, default_value_t = StrataArg::Placeholder)]
        strata: StrataArg,
        /// Circumference C in blocks (a multiple of 64, at least 512).
        #[arg(long)]
        size: u32,
        /// World-data folder to write.
        #[arg(long)]
        out: PathBuf,
        /// Also write an HTML viewer bundle here.
        #[arg(long)]
        viewer: Option<PathBuf>,
        /// Also dump exact sample values as JSON (the Java parity-test fixture).
        #[arg(long)]
        dump_samples: Option<PathBuf>,
        /// Tile encoding.
        #[arg(long, value_enum, default_value_t = CodecArg::Lz4)]
        codec: CodecArg,
    },
    /// The column evaluator (plan §9.3, P1-08): what Deep Time decides for one block column of a
    /// world-data folder — the terrain height, water, the horizon stack, the bodies it passes
    /// through and the rock from the top block down — as JSON, with every other layer's value there
    /// and the sites whose box holds it.
    #[command(allow_negative_numbers = true)]
    InspectColumn {
        /// World-data folder (a run's `--out`, a world's `deeptime/` folder).
        #[arg(long)]
        world_data: PathBuf,
        /// Block x (any lap: it is folded into the canonical span).
        x: i64,
        /// Block z.
        z: i64,
        /// Only what `parity/columns.json` holds per column.
        #[arg(long)]
        parity: bool,
        /// Add each site's "why here" (its forming event and the events that caused it, read from
        /// the world data's event log) to the answer; the workbench's click-to-inspect asks for it.
        #[arg(long, conflicts_with = "parity")]
        why: bool,
    },
    /// How many rock columns a geologist would object to: metamorphic grade inversions (a
    /// higher grade above rock that was never that hot), thin metamorphic slivers, and plutonic
    /// rock lying in the pile like a bed. On a world-data folder: the columns the game builds and
    /// the stored stacks; on a checkpoint: the stacks before and after the bake's smoothing.
    #[command(allow_negative_numbers = true)]
    StrataCheck {
        /// World-data folder (a run's `--out`).
        #[arg(long)]
        world_data: Option<PathBuf>,
        /// A checkpoint whose world has run the structure stage (`run --checkpoints`).
        #[arg(long)]
        checkpoint: Option<PathBuf>,
        /// How many random columns of the world data.
        #[arg(long, default_value_t = 20_000)]
        columns: usize,
        /// Seed of the column choice.
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// A metamorphic horizon thinner than this inside the pile is a sliver, blocks.
        #[arg(long, default_value_t = 8)]
        sliver_blocks: i32,
        /// A plutonic horizon thinner than this between other rocks is a sheet, blocks.
        #[arg(long, default_value_t = 16)]
        sheet_blocks: i32,
        /// Print this column's stack and findings instead of the census, `X,Z` (repeatable).
        #[arg(long = "column", value_name = "X,Z", allow_hyphen_values = true)]
        column: Vec<String>,
    },
    /// Cross-sections of a baked world through its folds and faults (P2-01), each drawn as the
    /// layers were laid down and as the structures left them: across the largest collision belt,
    /// arc and rift of the world's event log, and along any `--line`.
    StructureSections {
        /// World-data folder (a run's `--out`).
        #[arg(long)]
        world_data: PathBuf,
        /// Folder to write the PNGs, `sections.json` and `index.html` into.
        #[arg(long)]
        out: PathBuf,
        /// A line of your own, `TITLE:x0,z0,x1,z1` in blocks (repeatable).
        #[arg(long = "line", allow_hyphen_values = true)]
        line: Vec<String>,
        /// Half the length of a suggested section, blocks.
        #[arg(long, default_value_t = 320.0)]
        half: f64,
        /// Pixels a block.
        #[arg(long, default_value_t = 2)]
        px: u32,
    },
    /// How a baked planet's deposits come out in blocks, per deposit type, beside the background
    /// scatter and vanilla's ore density (no Minecraft server: the bake evaluated as the mod's
    /// deposit and scatter features do). Prints a table; --json writes every site's row.
    #[command(allow_negative_numbers = true)]
    DepositAudit {
        /// World-data folder a planet run baked.
        #[arg(long)]
        world_data: PathBuf,
        /// A folder of scatter rules (`*.json`) laid over the jar's own, by material.
        #[arg(long)]
        scatter_dir: Option<PathBuf>,
        /// Random land chunks the scatter is measured on.
        #[arg(long, default_value_t = 2000)]
        chunks: usize,
        /// Seed of the chunk choice.
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Top solid blocks of a land column counted as soil (no bedrock host).
        #[arg(long, default_value_t = 4)]
        soil: i32,
        /// Also count one block box (X0 Z0 X1 Z1, inclusive), to compare with an export-area.
        #[arg(long = "box", num_args = 4, value_names = ["X0", "Z0", "X1", "Z1"])]
        area: Option<Vec<i64>>,
        /// Write the full report (every site) as JSON.
        #[arg(long)]
        json: Option<PathBuf>,
        /// Only the sites of this deposit type (repeatable; `placer_au` or `deeptime:placer_au`).
        #[arg(long)]
        only: Vec<String>,
        /// Scatter multiplier instead of the world's (`notes.deposits.scatter_mult`).
        #[arg(long)]
        scatter_mult: Option<f64>,
        /// Random land points of the early-game access probe (0: none).
        #[arg(long, default_value_t = 150)]
        access_points: usize,
        /// Blocks under the top solid block the access probe counts ore in.
        #[arg(long, default_value_t = 48)]
        access_depth: i32,
        /// Ore-equivalent (a raw block counts 9) the access probe gathers.
        #[arg(long, default_value_t = 24.0)]
        access_need: f64,
        /// How far the access probe looks, in chunks.
        #[arg(long, default_value_t = 96)]
        access_radius: i64,
    },
    /// Write the Java↔Rust column parity fixture (plan §9.2): columns of a world-data folder as the
    /// column evaluator answers them, for `ColumnParityTest` and the real-planet GameTest.
    Parity {
        /// World-data folder.
        #[arg(long)]
        world_data: PathBuf,
        /// JSON file to write (default: <world-data>/parity/columns.json).
        #[arg(long)]
        out: Option<PathBuf>,
        /// How many columns.
        #[arg(long, default_value_t = parity::DEFAULT_COLUMNS)]
        columns: usize,
        /// Seed of the column choice.
        #[arg(long, default_value_t = parity::DEFAULT_SEED)]
        seed: u64,
    },
    /// Re-run the bake's water part (P1-22: lakes, `water_level`, the river network) on a baked
    /// world-data folder with its own preset and `--set hydro.KEY=VALUE` overrides (tuning).
    Hydro {
        /// World-data folder a planet run baked.
        #[arg(long)]
        world_data: PathBuf,
        /// Sets a preset value first, `--set hydro.width_coef=1.5` (repeatable).
        #[arg(long = "set", value_name = "PATH=VALUE")]
        set: Vec<String>,
    },
    /// Review pictures of a run's volcanoes (P1-21): the finished terrain with them marked, the
    /// settings that made them, and close views before and after the amplifier's shaping.
    VolcanoView {
        /// The run's last checkpoint (`run --checkpoints` writes `<out>/checkpoints/<n>-amplify.dtcp`).
        #[arg(long = "in")]
        input: PathBuf,
        /// The run's world-data folder (its `--out`).
        #[arg(long)]
        world_data: PathBuf,
        /// Folder to write the pictures, `volcanoes.json` and `index.html` into.
        #[arg(long)]
        out: PathBuf,
    },
    /// The terrain of a box of a world-data folder as a float32 NumPy array (H, E, A·N and any
    /// --layers, nearest), for review renders without a Minecraft server.
    #[command(allow_negative_numbers = true)]
    Heightmap {
        /// World-data folder.
        #[arg(long)]
        world_data: PathBuf,
        /// x0,z0,x1,z1 in blocks (ends exclusive).
        #[arg(long = "box", value_delimiter = ',', allow_negative_numbers = true)]
        area: Vec<i64>,
        /// Blocks between samples.
        #[arg(long, default_value_t = 1)]
        step: i64,
        /// More layers to sample (nearest), comma-separated.
        #[arg(long, value_delimiter = ',')]
        layers: Vec<String>,
        /// Also what the column evaluator decides: rock classes at four depths, the horizon
        /// count, the second horizon's top and the water kind.
        #[arg(long)]
        columns: bool,
        /// Also the ground distance one block spans there, metres (the projection's local scale),
        /// and the block's latitude and longitude.
        #[arg(long)]
        scale: bool,
        /// The .npy file to write (and <out>.json).
        #[arg(long)]
        out: PathBuf,
    },
    /// How much rock the surface stage took off and laid on, per region type (orogen, basin,
    /// shield, other land, shelf, ocean) and per crust parcel followed with the crust, the last
    /// frame's land hypsometry (from a checkpoint with the recorded history), and every porphyry's
    /// ledger and depth (from a world-data folder).
    ErosionAudit {
        /// World-data folder a planet run baked (the porphyries).
        #[arg(long)]
        world_data: Option<PathBuf>,
        /// A checkpoint carrying the recorded history (`run --checkpoints`).
        #[arg(long)]
        checkpoint: Option<PathBuf>,
        /// Also write the report (every porphyry) as JSON.
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Run the bake's projection pass (SW-1) on a copy of a baked Lambert world-data folder, with
    /// its own preset and `--set scale.KEY=VALUE` overrides (`scale.projection` is petroff_guyou):
    /// tuning the pass without re-running the history.
    Project {
        /// A Lambert world-data folder a planet run baked (`--set scale.projection=lambert`).
        #[arg(long)]
        world_data: PathBuf,
        /// The projected copy (replaced if it exists).
        #[arg(long)]
        out: PathBuf,
        /// Sets a preset value first, `--set scale.lambda_c_deg=30` (repeatable).
        #[arg(long = "set", value_name = "PATH=VALUE")]
        set: Vec<String>,
    },
    /// Whole-world pictures of a baked world-data folder in the world's own block space
    /// (satellite.png, provinces.png with the layout's cone points and poles, world-map.json).
    WorldMap {
        /// World-data folder.
        #[arg(long)]
        world_data: PathBuf,
        /// The satellite picture's zoom: 2^(z+1) × 2^z tiles of 256 pixels.
        #[arg(long, default_value_t = 2)]
        zoom: u32,
        /// Folder to write into.
        #[arg(long)]
        out: PathBuf,
    },
    /// Dump a world's sites, index lookups, body evaluations and cause chains as JSON (the Java
    /// parity-test fixture, P1-15).
    DumpDeposits {
        /// World-data folder holding sites/, bodies/ and events/.
        #[arg(long)]
        world: PathBuf,
        /// JSON file to write.
        #[arg(long)]
        out: PathBuf,
    },
    /// Write fields of a checkpoint's world as raw little-endian f32 files (`<dir>/<name>.f32`,
    /// `/` in a name becomes `__`) with an `index.json` of their grids, for analysis scripts.
    DumpFields {
        /// Checkpoint to read.
        #[arg(long = "in")]
        input: PathBuf,
        /// Folder to write.
        #[arg(long)]
        out: PathBuf,
        /// Field names (comma-separated); default every field.
        #[arg(long, value_delimiter = ',')]
        fields: Vec<String>,
    },
    /// The plate dynamics of finished runs against the acceptance targets of
    /// docs/design/plate-collisions-and-welding.md §6: largest plate, plate lifetimes and fates,
    /// land gained per step, what each collision lifted, collisions per 100 Myr, continental
    /// crust, active margins, sea-floor age. Prints one table (measure, value, target range, pass
    /// or fail) and exits non-zero when a measure is outside its range. With several runs the
    /// value is their mean.
    PlateCheck {
        /// A run: a checkpoint (`stage --out`), a world-data folder or its manifest.json, or a
        /// JSON file holding the plates note. Repeatable.
        #[arg(long = "run")]
        runs: Vec<PathBuf>,
        /// Also write the table as JSON here.
        #[arg(long)]
        json: Option<PathBuf>,
        /// Also write the first run's whole dynamics record (`notes.plates.dynamics`: steps,
        /// collisions, partings, belts) as JSON here, for analysis beyond the table.
        #[arg(long)]
        record: Option<PathBuf>,
        /// JSON file moving target ranges: {"measure_key": [lo, hi]}, null for an open end or for
        /// no range at all (the measure is then only reported).
        #[arg(long)]
        targets: Option<PathBuf>,
        /// Sets one of the check's definitions, as name=value (repeatable): assembled_share,
        /// dispersed_share (a supercontinent is assembled when the largest piece of continental
        /// crust holds the first share of it, until it holds under the second), mega_plate_share,
        /// min_plate_samples, fast_mm_yr, warmup_myr, start_up_myr, min_piece_share,
        /// rift_window_myr. A "params" object in the targets file does the same.
        #[arg(long = "param")]
        params: Vec<String>,
    },
    /// Draw the continental margins of a checkpoint's plate model by tectonic setting (P1-27:
    /// passive, Andean, subducting, collision, other plate boundary) over the sea floor by age,
    /// with plate boundaries and subduction fronts, as a PNG, and print the margin shares.
    Margins {
        /// A checkpoint written by `stage --out` or `run --checkpoints`.
        #[arg(long = "in")]
        input: PathBuf,
        /// PNG to write.
        #[arg(long)]
        out: PathBuf,
    },
    /// Draw the geologic-provinces view layer (shield, platform, orogen, basin, LIP, extended
    /// crust, oceanic crust by age) of a checkpoint's world as a PNG and print each class's share.
    Provinces {
        /// A checkpoint written by `stage --out` or `run --checkpoints`.
        #[arg(long = "in")]
        input: PathBuf,
        /// PNG to write.
        #[arg(long)]
        out: PathBuf,
        /// The preset's `provinces` section: a collision belt stays an orogen this many Myr
        /// (default 500) …
        #[arg(long)]
        orogen_max_age_myr: Option<f64>,
        /// … an Andean range while lifted within this many Myr (default 50) …
        #[arg(long)]
        andean_max_age_myr: Option<f64>,
        /// … and standing at least this high, m (default 1500). They change the class shares
        /// printed and the PNG; the in-game view layer always uses the defaults.
        #[arg(long)]
        andean_min_elevation_m: Option<f64>,
    },
    /// Draw the satellite view layer (true colour: land cover from the climate, snow and glacier
    /// ice, the sea by depth, sea ice, hill shading, the rivers and lakes of the terrain) of a
    /// checkpoint's world as a PNG, and the rivers & lakes layer beside it (`<out>-rivers.png`).
    Satellite {
        /// A checkpoint written by `stage --out` or `run --checkpoints`.
        #[arg(long = "in")]
        input: PathBuf,
        /// PNG to write.
        #[arg(long)]
        out: PathBuf,
        /// Width in pixels (the height follows the projection's aspect: half of it).
        #[arg(long, default_value_t = 1024)]
        width: u32,
        /// Draw on this elevation field only (`elevation`: the deep-time surface's L1 frame;
        /// `young/elevation`; `amplify/elevation`), not the finest the world has.
        #[arg(long)]
        on: Option<String>,
        /// Sets a value of the checkpoint's preset before drawing, `--set hydro.lake_balance=false`
        /// (repeatable): the view's hydrology settings can be compared without a new run.
        #[arg(long = "set", value_name = "PATH=VALUE")]
        set: Vec<String>,
    },
}

#[derive(Subcommand)]
enum PresetAction {
    /// List the built-in presets.
    List,
    /// Print a preset's canonical JSON (a built-in name, a file or a share code).
    Show { preset: String },
    /// Print a preset's share code (DT1.…).
    Encode { preset: String },
    /// Print the JSON a share code carries.
    Decode { code: String },
}

#[derive(clap::Args)]
struct CommonArgs {
    /// Rayon threads (0 = all processors).
    #[arg(long, default_value_t = 0)]
    threads: usize,
    /// Native memory cap in bytes (0 = 4 GiB).
    #[arg(long, default_value_t = 0)]
    mem_cap: u64,
    /// Print DEBUG log lines too.
    #[arg(long)]
    verbose: bool,
    /// Sets a preset value before the run, `--set amplify.incision_m=8` (repeatable; the value
    /// is JSON, else a string). On `stage --in` it edits the checkpoint's preset, so a later
    /// stage's parameters can be swept without re-running the earlier ones.
    #[arg(long = "set", value_name = "PATH=VALUE")]
    set: Vec<String>,
    /// Holds at most this many bytes of the recorded history's frames in memory, whatever the
    /// cap; the rest goes to a spill file in the run's folder (0 = every frame; default: the
    /// memory cap decides). Results are the same either way; for measuring the spill.
    #[arg(long)]
    history_mem: Option<u64>,
    /// Writes the climate stage's note (summary and curves, JSON) to this file after `stage`.
    #[arg(long)]
    climate_json: Option<PathBuf>,
    /// Where the kernels with a GPU version run: auto (a hardware GPU when there is one), cpu,
    /// or gpu (a software adapter such as lavapipe counts). Default: DEEPTIME_COMPUTE, else auto.
    #[arg(long, value_parser = parse_compute)]
    compute: Option<ComputeMode>,
}

fn parse_compute(s: &str) -> Result<ComputeMode, String> {
    ComputeMode::parse(s).ok_or_else(|| format!("{s}: expected auto, cpu or gpu"))
}

impl CommonArgs {
    fn common(&self) -> run::Common {
        run::Common {
            threads: self.threads,
            mem_cap: self.mem_cap,
            verbose: self.verbose,
            set: self.set.clone(),
            history_mem: self.history_mem,
            climate_json: self.climate_json.clone(),
            compute: self.compute,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum StrataArg {
    /// Three horizons: sandstone, red beds from 6 blocks down, granite from 40 (P0-09).
    Placeholder,
    /// All 62 rock classes: 8 horizons 10 blocks apart; any eleven neighbouring 16-block cell
    /// columns hold every class 20 or more blocks below the surface (P1-12's GameTests).
    EveryClass,
    /// A readable sedimentary basin, the same in every cell: eight beds over granite from 165
    /// blocks down (the synthetic deposit world's strata).
    Basin,
}

#[derive(Clone, Copy, ValueEnum)]
enum SynthKind {
    /// The §7.1 analytic terrain (AnalyticTestTerrain) with latitude climate.
    WrapTest,
    /// A basin (floor y 110, rim y 125, water 120) with a closed channel, on a plain at y 100.
    LakeTest,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CodecArg {
    Raw,
    Lz4,
}

fn finish(what: &str, r: run::CliResult<()>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("deeptime {what}: {e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    alloc::init_from_env();
    let cli = Cli::parse();
    match cli.command {
        None | Some(Command::Version) => {
            println!(
                "deeptime {} (format {})",
                deeptime_core::VERSION,
                deeptime_core::FORMAT_VERSION
            );
            ExitCode::SUCCESS
        }
        Some(Command::Run {
            preset,
            start,
            length_myr,
            content,
            placement_patch,
            out,
            viewer,
            checkpoints,
            stop_myr,
            epochs,
            common,
        }) => finish(
            "run",
            run::run(&run::RunOptions {
                preset,
                start,
                length_myr,
                content,
                placement_patch,
                out,
                viewer,
                checkpoints,
                stop_myr,
                epochs,
                common: common.common(),
            }),
        ),
        Some(Command::Resume {
            out,
            stop_myr,
            no_epochs,
            viewer,
            common,
        }) => finish(
            "resume",
            run::resume(&run::ResumeOptions {
                out,
                stop_myr,
                epochs: !no_epochs,
                viewer,
                common: common.common(),
            }),
        ),
        Some(Command::Watch {
            preset,
            start,
            length_myr,
            bind,
            port,
            runs,
            keep,
            every_myr,
            exit_when_done,
            preview,
            python,
            common,
        }) => finish(
            "watch",
            watch::watch(&watch::WatchOptions {
                preset,
                start,
                length_myr,
                bind,
                port,
                runs: runs.unwrap_or_else(|| std::env::temp_dir().join("deeptime-watch-runs")),
                keep,
                every_myr,
                exit_when_done,
                preview: preview.map(|script| watch::stage2::Preview {
                    python,
                    script,
                    runs: None,
                    deeptime: None,
                }),
                common: common.common(),
            }),
        ),
        Some(Command::Stage {
            name,
            input,
            preset,
            start,
            length_myr,
            max_steps,
            out,
            viewer,
            viewer_layers,
            work,
            bake,
            common,
        }) => finish(
            "stage",
            run::stage(&run::StageOptions {
                name,
                input,
                preset,
                start,
                length_myr,
                max_steps,
                out,
                viewer,
                viewer_layers,
                work,
                bake,
                common: common.common(),
            }),
        ),
        Some(Command::Bench {
            preset,
            start,
            length_myr,
            threads,
            out,
            viewer,
            work,
            mem_cap,
            verbose,
            compute,
        }) => finish(
            "bench",
            run::bench(&run::BenchOptions {
                preset,
                start,
                length_myr,
                threads,
                out,
                viewer,
                work,
                mem_cap,
                verbose,
                compute,
            }),
        ),
        Some(Command::Preset { action }) => finish("preset", preset_command(action)),
        Some(Command::ClimateDem {
            dem,
            width,
            preset,
            circumference_km,
            viewer,
            dump,
        }) => finish(
            "climate-dem",
            climate_dem::run(&climate_dem::Options {
                dem,
                w: width,
                // The DEM is equirectangular (GMT's layout): W/2 rows, whatever the planet's
                // projection (the climate resamples it onto its own grid).
                h: width / 2,
                preset,
                circumference_km,
                viewer,
                dump,
            }),
        ),
        Some(Command::Lem {
            grid,
            height,
            dx,
            steps,
            dt_yr,
            k,
            m,
            n,
            g,
            uplift_mm_yr,
            kd,
            seed,
            out,
        }) => finish(
            "lem",
            lem(
                &deeptime_core::surface::oracle::RasterParams {
                    w: grid,
                    h: height.unwrap_or(grid),
                    dx,
                    steps,
                    dt_yr,
                    k,
                    m,
                    n,
                    g,
                    uplift_m_yr: uplift_mm_yr * 1.0e-3,
                    kd_m2_yr: kd,
                    seed,
                },
                &out,
            ),
        ),
        Some(Command::InspectColumn {
            world_data,
            x,
            z,
            parity,
            why,
        }) => match deeptime_format::ColumnWorld::open(&world_data) {
            Ok(w) => {
                let mut v = if parity {
                    w.sample(x, z).to_json()
                } else {
                    w.inspect(x, z)
                };
                if why && let Err(e) = attach_why(&mut v, &world_data) {
                    eprintln!("deeptime inspect-column: {e}");
                    return ExitCode::FAILURE;
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&v).expect("a column serialises")
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("deeptime inspect-column: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::VolcanoView {
            input,
            world_data,
            out,
        }) => match volcano_view::run(&input, &world_data, &out) {
            Ok(v) => {
                println!("deeptime volcano-view: {v} -> {}", out.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("deeptime volcano-view: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::StructureSections {
            world_data,
            out,
            line,
            half,
            px,
        }) => match structure_sections::run(&world_data, &out, &line, half, px.clamp(1, 8)) {
            Ok(v) => {
                for s in v["sections"].as_array().into_iter().flatten() {
                    println!(
                        "deeptime structure-sections: {} from {} to {} ({:.0} km), {} of {} columns changed -> {}",
                        s["title"].as_str().unwrap_or(""),
                        s["from"],
                        s["to"],
                        s["km"].as_f64().unwrap_or(0.0),
                        s["columns_changed"],
                        s["columns"],
                        out.join(s["structure"].as_str().unwrap_or("")).display()
                    );
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("deeptime structure-sections: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::StrataCheck {
            world_data,
            checkpoint,
            columns,
            seed,
            sliver_blocks,
            sheet_blocks,
            column,
        }) => {
            let at: Result<Vec<(i64, i64)>, String> = column
                .iter()
                .map(|s| {
                    let bad = || format!("--column {s}: expected X,Z");
                    let (x, z) = s.split_once(',').ok_or_else(bad)?;
                    Ok((
                        x.trim().parse().map_err(|_| bad())?,
                        z.trim().parse().map_err(|_| bad())?,
                    ))
                })
                .collect();
            let answer = at.map_err(Into::into).and_then(|at| {
                strata_check::run(&strata_check::Options {
                    world_data: world_data.as_deref(),
                    checkpoint: checkpoint.as_deref(),
                    columns,
                    seed,
                    limits: deeptime_core::structure::realism::Limits {
                        sliver_blocks,
                        sheet_blocks,
                    },
                    at: &at,
                })
            });
            match answer {
                Ok(v) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&v).expect("a census serialises")
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("deeptime strata-check: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Command::DepositAudit {
            world_data,
            scatter_dir,
            chunks,
            seed,
            soil,
            area,
            json,
            only,
            scatter_mult,
            access_points,
            access_depth,
            access_need,
            access_radius,
        }) => {
            let o = deposit_audit::Options {
                scatter_mult,
                access_points,
                access_depth,
                access_need,
                access_radius,
                world_data,
                scatter_dir,
                chunks,
                seed,
                soil,
                area: area.map(|a| [a[0], a[1], a[2], a[3]]),
                json,
                only,
            };
            match deposit_audit::run(&o) {
                Ok(report) => {
                    print!("{}", deposit_audit::table(&report));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("deeptime deposit-audit: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Command::Parity {
            world_data,
            out,
            columns,
            seed,
        }) => {
            let out = out.unwrap_or_else(|| world_data.join("parity/columns.json"));
            let started = std::time::Instant::now();
            match parity::run(&world_data, &out, columns, seed) {
                Ok((n, through)) => {
                    println!(
                        "deeptime parity: {n} columns ({through} through a body) -> {} in {:.2}s",
                        out.display(),
                        started.elapsed().as_secs_f64()
                    );
                    // P1-25: the climate API's evaluator beside it, when the world has the months.
                    let cout = out.with_file_name("climate.json");
                    match parity::run_climate(&world_data, &cout, columns, seed) {
                        Ok(Some(k)) => {
                            println!("deeptime parity: {k} climate samples -> {}", cout.display());
                            ExitCode::SUCCESS
                        }
                        Ok(None) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("deeptime parity (climate): {e}");
                            ExitCode::FAILURE
                        }
                    }
                }
                Err(e) => {
                    eprintln!("deeptime parity: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Command::Hydro { world_data, set }) => {
            let started = std::time::Instant::now();
            match hydro::run(&world_data, &set) {
                Ok(line) => {
                    println!(
                        "deeptime hydro: {line} in {:.2}s",
                        started.elapsed().as_secs_f64()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("deeptime hydro: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Command::Project {
            world_data,
            out,
            set,
        }) => {
            let started = std::time::Instant::now();
            match project::run(&world_data, &out, &set) {
                Ok(note) => {
                    println!(
                        "deeptime project: {} in {:.2}s",
                        deeptime_core::bake::project::summary_line(&note),
                        started.elapsed().as_secs_f64()
                    );
                    println!("{}", serde_json::to_string(&note).unwrap_or_default());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("deeptime project: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Command::WorldMap {
            world_data,
            zoom,
            out,
        }) => match world_map::run(&world_data, zoom, &out) {
            Ok(v) => {
                println!(
                    "deeptime world-map: {} pixels -> {}",
                    v["pixels"],
                    out.display()
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("deeptime world-map: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::Heightmap {
            world_data,
            area,
            step,
            layers,
            columns,
            scale,
            out,
        }) => {
            let b = heightmap::Box2 {
                x0: area[0],
                z0: area[1],
                x1: area[2],
                z1: area[3],
            };
            match heightmap::run(&world_data, &b, step, &layers, columns, scale, &out) {
                Ok((rows, cols)) => {
                    println!("deeptime heightmap: {rows} x {cols} -> {}", out.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("deeptime heightmap: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Command::PlateCheck {
            runs,
            json,
            record,
            targets,
            params,
        }) => match plate_check::run(&plate_check::Options {
            runs: &runs,
            json: json.as_deref(),
            record: record.as_deref(),
            targets: targets.as_deref(),
            params: &params,
        }) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("deeptime plate-check: {e}");
                ExitCode::from(2)
            }
        },
        Some(Command::Margins { input, out }) => match margins_view::run(&input, &out) {
            Ok(m) => {
                println!(
                    "deeptime margins: t_myr={:.0} coast_km={:.0} active={:.3} subduction={:.3} \
                     andean={:.3} boundary={:.3} passive={:.3} regime_active={:.3} \
                     passive_floor_age_p50_myr={:.1} front_km_p50={:.0} landmasses={} \
                     fragment_crust={:.3} fragment_coast_km={:.0} -> {}",
                    m.t_myr,
                    m.coast_km,
                    m.active_share(),
                    m.subduction_share(),
                    f64::from(m.andean) / f64::from(m.coast.max(1)),
                    m.boundary_share(),
                    m.passive_share(),
                    m.regime_active_share(),
                    m.passive_floor_age_p50_myr,
                    m.front_km_p50,
                    m.landmasses,
                    m.fragment_crust_share,
                    m.coast_km * f64::from(m.fragment_coast) / f64::from(m.coast.max(1)),
                    out.display()
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("deeptime margins: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::Provinces {
            input,
            out,
            orogen_max_age_myr,
            andean_max_age_myr,
            andean_min_elevation_m,
        }) => match provinces::run(
            &input,
            &out,
            {
                let mut p = deeptime_core::provinces::ProvinceParams::default();
                p.orogen_max_age_myr = orogen_max_age_myr.unwrap_or(p.orogen_max_age_myr);
                p.andean_max_age_myr = andean_max_age_myr.unwrap_or(p.andean_max_age_myr);
                p.andean_min_elevation_m =
                    andean_min_elevation_m.unwrap_or(p.andean_min_elevation_m);
                p
            }
            .rules(),
        ) {
            Ok(shares) => {
                for (name, pct) in shares {
                    println!("{pct:6.2} %  {name}");
                }
                println!("deeptime provinces: -> {}", out.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("deeptime provinces: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::Satellite {
            input,
            out,
            width,
            on,
            set,
        }) => match satellite_view::run(&input, &out, width, on.as_deref(), &set) {
            Ok(r) => {
                println!(
                    "deeptime satellite: t = {:.1} Myr, drawn on {} -> {}",
                    r.t_years / 1.0e6,
                    r.elevation,
                    out.display()
                );
                if let (Some(p), Some(w)) = (&r.rivers, &r.water) {
                    println!(
                        "deeptime satellite: rivers & lakes -> {} ({} river chains, {} vertices, {:.0} px of lakes, {:.0} of them frozen, {:.0} px of playa); satellite picture {:.0} ms, water derived in {:.0} ms (phases {}) and drawn in {:.0} ms",
                        p.display(),
                        w.chains,
                        w.vertices,
                        w.lake_px,
                        w.frozen_px,
                        w.playa_px,
                        r.satellite_ms,
                        r.derive_ms,
                        w.phases,
                        r.draw_ms
                    );
                    let mut lakes = w.stats.clone();
                    if let Some(o) = lakes.as_object_mut() {
                        for k in [
                            "grid",
                            "millis",
                            "index_entries",
                            "vertices",
                            "open_sea_min_cells",
                        ] {
                            o.remove(k);
                        }
                    }
                    println!("deeptime satellite: lakes {lakes}");
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("deeptime satellite: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::ErosionAudit {
            world_data,
            checkpoint,
            json,
        }) => {
            let mut report = serde_json::Map::new();
            if let Some(ck) = &checkpoint {
                match erosion_audit::history(ck) {
                    Ok(v) => {
                        report.insert("history".into(), v);
                    }
                    Err(e) => {
                        eprintln!("deeptime erosion-audit: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            if let Some(w) = &world_data {
                match erosion_audit::porphyries(w) {
                    Ok(v) => {
                        report.insert("porphyries".into(), v);
                    }
                    Err(e) => {
                        eprintln!("deeptime erosion-audit: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            let v = serde_json::Value::Object(report);
            if let Some(p) = &json
                && let Err(e) = std::fs::write(p, serde_json::to_vec_pretty(&v).expect("JSON"))
            {
                eprintln!("deeptime erosion-audit: {}: {e}", p.display());
                return ExitCode::FAILURE;
            }
            // The summary without the per-porphyry rows.
            let mut short = v.clone();
            if let Some(p) = short.get_mut("porphyries").and_then(|p| p.as_object_mut()) {
                p.remove("rows");
            }
            println!("{}", serde_json::to_string_pretty(&short).expect("JSON"));
            ExitCode::SUCCESS
        }
        Some(Command::DumpDeposits { world, out }) => match dump::run(&world, &out) {
            Ok((sites, bodies, inside)) => {
                println!(
                    "deeptime dump-deposits: {sites} sites, {bodies} bodies ({inside} samples inside) -> {}",
                    out.display()
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("deeptime dump-deposits: {e}");
                ExitCode::FAILURE
            }
        },
        Some(Command::DumpFields { input, out, fields }) => {
            match dump::fields(&input, &out, &fields) {
                Ok(n) => {
                    println!("deeptime dump-fields: {n} fields -> {}", out.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("deeptime dump-fields: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Command::Synth {
            kind,
            strata,
            size,
            out,
            viewer,
            dump_samples,
            codec,
        }) => {
            let opts = synth::Options {
                kind: match kind {
                    SynthKind::WrapTest => synth::Kind::WrapTest,
                    SynthKind::LakeTest => synth::Kind::LakeTest,
                },
                strata: match strata {
                    StrataArg::Placeholder => synth::Strata::Placeholder,
                    StrataArg::EveryClass => synth::Strata::EveryClass,
                    StrataArg::Basin => synth::Strata::Basin,
                },
                size,
                out,
                viewer,
                dump_samples,
                codec: match codec {
                    CodecArg::Raw => deeptime_format::Codec::Raw,
                    CodecArg::Lz4 => deeptime_format::Codec::Lz4Block,
                },
            };
            let started = std::time::Instant::now();
            match synth::run(&opts) {
                Ok(s) => {
                    println!(
                        "deeptime synth: {} C={} -> {} ({} layers, {:.1} MiB) in {:.2}s",
                        opts.kind.name(),
                        size,
                        opts.out.display(),
                        s.layers,
                        s.bytes as f64 / (1024.0 * 1024.0),
                        started.elapsed().as_secs_f64()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("deeptime synth: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

/// `deeptime lem`: the surface kernels on a plain raster (the oracle's input).
fn lem(
    p: &deeptime_core::surface::oracle::RasterParams,
    out: &std::path::Path,
) -> run::CliResult<()> {
    if p.w < 3 || p.h < 3 || p.steps == 0 {
        return Err("--grid and --height ≥ 3, --steps ≥ 1".into());
    }
    std::fs::create_dir_all(out)?;
    let r = deeptime_core::surface::oracle::run(p);
    let write = |name: &str, v: &[f64]| -> std::io::Result<()> {
        let bytes: Vec<u8> = v.iter().flat_map(|&x| (x as f32).to_le_bytes()).collect();
        std::fs::write(out.join(name), bytes)
    };
    write("initial.f32", &r.initial)?;
    write("final.f32", &r.elevation)?;
    write("area.f32", &r.area_m2)?;
    let max = r.elevation.iter().copied().fold(f64::MIN, f64::max);
    let mean = r.elevation.iter().sum::<f64>() / r.elevation.len() as f64;
    let meta = serde_json::json!({
        "w": p.w, "h": p.h, "dx": p.dx, "steps": p.steps, "dt_yr": p.dt_yr,
        "k": p.k, "m": p.m, "n": p.n, "g": p.g, "uplift_m_yr": p.uplift_m_yr, "kd_m2_yr": p.kd_m2_yr,
        "seed": p.seed, "boundary": "periodic x, rows 0 and h−1 fixed at 0 m",
        "dtype": "f32 little-endian, row-major (row j = y index)",
        "seconds": r.seconds, "max_elevation_m": max, "mean_elevation_m": mean,
    });
    std::fs::write(out.join("meta.json"), serde_json::to_vec_pretty(&meta)?)?;
    println!(
        "deeptime lem: {}x{} dx={} steps={} dt_yr={} -> {} (max {:.1} m, mean {:.1} m) in {:.2}s",
        p.w,
        p.h,
        p.dx,
        p.steps,
        p.dt_yr,
        out.display(),
        max,
        mean,
        r.seconds
    );
    Ok(())
}

fn preset_command(action: PresetAction) -> run::CliResult<()> {
    use deeptime_core::{preset, share_code};
    match action {
        PresetAction::List => {
            for (name, json) in preset::BUILTIN {
                let p = preset::Preset::from_json(json)?;
                println!(
                    "{name}\tC={}{}",
                    p.circumference(),
                    if p.header.placeholder {
                        "\tplaceholder (Phase 1 in progress)"
                    } else {
                        ""
                    }
                );
            }
        }
        PresetAction::Show { preset: arg } => {
            let json = preset::resolve(&arg)?;
            let v: serde_json::Value = serde_json::from_str(&share_code::canonical(&json)?)?;
            println!("{}", serde_json::to_string_pretty(&v)?);
        }
        PresetAction::Encode { preset: arg } => {
            println!("{}", share_code::encode(&preset::resolve(&arg)?)?);
        }
        PresetAction::Decode { code } => println!("{}", share_code::decode(&code)?),
    }
    Ok(())
}

/// `inspect-column --why`: puts `why` on each of the answer's sites: the forming event and its
/// cause chain (four links deep, as the 3D export's site cards do) from the world data's event log;
/// and `landforms`: the volcanoes and flood-basalt provinces (P1-21) and the impact craters
/// (P2-08) whose footprint holds the column, nearest first, each with its event and cause chain.
fn attach_why(v: &mut serde_json::Value, world_data: &std::path::Path) -> Result<(), String> {
    use deeptime_format::events::{EventId, EventLog};
    let log = EventLog::load(world_data).map_err(|e| e.to_string())?;
    if let (Some(lat), Some(lon)) = (v["lat"].as_f64(), v["lon"].as_f64()) {
        let m = deeptime_format::Manifest::read(world_data).map_err(|e| e.to_string())?;
        let landforms = landforms_at(&log, lat, lon, &m.scale);
        if !landforms.is_empty() {
            v["landforms"] = serde_json::json!(landforms);
        }
    }
    let Some(sites) = v["sites"].as_array_mut().filter(|s| !s.is_empty()) else {
        return Ok(());
    };
    for site in sites {
        let formed = site["forming_event"]
            .as_u64()
            .and_then(|f| log.get(EventId(f)));
        site["why"] = match formed {
            Some(e) => serde_json::json!({
                "forming_event": e.to_json(),
                "chain": log.cause_chain(e.id, 4).iter().map(|c| c.to_json()).collect::<Vec<_>>(),
            }),
            None => serde_json::json!({ "error": "the forming event is not in the event log" }),
        };
    }
    Ok(())
}

/// The volcano events (`deeptime:volcano_*`) whose footprint holds the point (lat, lon): a notable
/// volcano out to its basal radius (at least 3 km), an arc segment or hotspot chain where one of its
/// listed cones does, a caldera to its rim, a flood-basalt province over its area; and the impact
/// craters (`deeptime:impact_crater`) still in the catalogue, out to their rims; each as
/// `{event, chain, distance_km}`, nearest first (a cone's own event before its group's).
fn landforms_at(
    log: &deeptime_format::events::EventLog,
    lat: f64,
    lon: f64,
    scale: &deeptime_format::Scale,
) -> Vec<serde_json::Value> {
    use deeptime_format::{ParamValue, Place};
    let c = f64::from(scale.circumference_blocks);
    let km_per_block = scale.m_per_block_h / 1000.0;
    let num = |e: &deeptime_format::Event, k: &str| match e.params.get(k) {
        Some(ParamValue::Float(x)) => Some(*x),
        Some(ParamValue::Int(x)) => Some(*x as f64),
        _ => None,
    };
    let mut hits: Vec<(f64, serde_json::Value)> = log
        .events()
        .iter()
        .filter(|e| {
            e.source == deeptime_core::surface::volcanoes::SOURCE
                || e.params.contains_key("volcano_cones")
                || (e.kind == deeptime_core::surface::impacts::kind::CRATER
                    && e.params.get("standing") == Some(&ParamValue::Int(1)))
        })
        .filter_map(|e| {
            // Distance in blocks of the world (the projection's blocks), as km.
            let proj = deeptime_format::Projection::of(c);
            let (x0, z0) = proj.block_of(lat, lon);
            let dist = |la: f64, lo: f64| {
                let (x1, z1) = proj.block_of(la, lo);
                proj.wrap_x(x0 - x1).hypot(z0 - z1) * km_per_block
            };
            // An arc segment or a hotspot chain holds the column when one of its cones does (a
            // small segment's cones are listed on its subduction onset, `volcano_cones`, wherever
            // that event is placed).
            if let Some(ParamValue::Str(cones)) =
                e.params.get("cones").or(e.params.get("volcano_cones"))
            {
                let d = cones
                    .split(';')
                    .filter_map(|cone| {
                        let f: Vec<f64> = cone.split(',').filter_map(|x| x.parse().ok()).collect();
                        (f.len() >= 3)
                            .then(|| (dist(f[0], f[1]), f[2].max(3.0)))
                            .filter(|&(d, r)| d <= r)
                            .map(|(d, _)| d)
                    })
                    .fold(f64::INFINITY, f64::min);
                if !d.is_finite() {
                    return None;
                }
                let chain: Vec<serde_json::Value> = log
                    .cause_chain(e.id, 4)
                    .iter()
                    .map(|x| x.to_json())
                    .collect();
                let mut ev = e.to_json();
                for k in ["cones", "volcano_cones"] {
                    if ev["params"].get(k).is_some() {
                        ev["params"][k] = serde_json::json!("…");
                    }
                }
                return Some((
                    d + 1.0e-3,
                    serde_json::json!({
                        "event": ev,
                        "chain": chain,
                        "distance_km": (d * 10.0).round() / 10.0,
                    }),
                ));
            }
            let Place::Point { lat: la, lon: lo } = e.place else {
                return None;
            };
            let d = dist(f64::from(la), f64::from(lo));
            let reach = match num(e, "area_km2") {
                Some(a) => (a / std::f64::consts::PI).sqrt(),
                None => num(e, "radius_km").unwrap_or(0.0).max(3.0),
            };
            (d <= reach).then(|| {
                let chain: Vec<serde_json::Value> = log
                    .cause_chain(e.id, 4)
                    .iter()
                    .map(|x| x.to_json())
                    .collect();
                (
                    d,
                    serde_json::json!({
                        "event": e.to_json(),
                        "chain": chain,
                        "distance_km": (d * 10.0).round() / 10.0,
                    }),
                )
            })
        })
        .collect();
    hits.sort_by(|a, b| a.0.total_cmp(&b.0));
    hits.into_iter().map(|(_, v)| v).collect()
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn why_here_finds_a_small_arc_segments_cone_on_its_subduction_onset() {
        use deeptime_core::session::GEOLOGY_WRITER_NAME;
        use deeptime_format::{
            Event, EventId, EventLog, EventWriter, GEOLOGY_WRITER, ParamValue, Place,
        };
        let dir = std::env::temp_dir().join(format!("deeptime-landforms-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // The onset sits at the contact's centroid, far from the cone it fed.
        let mut onset = Event::new(
            EventId::new(GEOLOGY_WRITER, 1),
            -40.0e6,
            "deeptime:subduction_onset",
            "deeptime:plates",
        );
        onset.place = Place::Point { lat: 0.0, lon: 0.0 };
        onset.params.insert(
            "volcano_cones".into(),
            ParamValue::Str("10.000,20.000,20.0,2500".into()),
        );
        let mut w = EventWriter::open(&dir, GEOLOGY_WRITER, GEOLOGY_WRITER_NAME).unwrap();
        w.append(&[onset]).unwrap();
        let log = EventLog::load(&dir).unwrap();
        let scale = deeptime_format::Scale::default_for(16_384);
        let hits = super::landforms_at(&log, 10.0, 20.05, &scale);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0]["event"]["kind"], "deeptime:subduction_onset");
        assert_eq!(hits[0]["event"]["params"]["volcano_cones"], "…");
        assert!(super::landforms_at(&log, 0.0, 0.0, &scale).is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn version_nonempty() {
        assert!(!deeptime_core::VERSION.is_empty());
    }

    #[test]
    fn cli_is_well_formed() {
        super::Cli::command().debug_assert();
    }

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("deeptime-cli-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn dummy_preset() -> String {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../deeptime-core/tests/presets/dummy.json"
        )
        .to_owned()
    }

    fn quiet() -> super::run::Common {
        super::run::Common {
            threads: 2,
            mem_cap: 0,
            verbose: false,
            set: Vec::new(),
            history_mem: None,
            climate_json: None,
            compute: None,
        }
    }

    /// The P1-01 acceptance, in process: run the dummy preset with a viewer.
    #[test]
    fn run_writes_the_world_and_the_viewer() {
        let dir = tmp("run");
        super::run::run(&super::run::RunOptions {
            preset: dummy_preset(),
            start: None,
            length_myr: None,
            content: None,
            placement_patch: Vec::new(),
            out: dir.join("dummy"),
            viewer: Some(dir.join("viewer")),
            checkpoints: true,
            stop_myr: None,
            epochs: false,
            common: quiet(),
        })
        .unwrap();
        for f in [
            "dummy/manifest.json",
            "dummy/events/1-0.dtev",
            "dummy/keyframes/index.json",
            "dummy/sites/sites.dtsr",
            "dummy/checkpoints/1-dummy_erode.dtcp",
            "viewer/index.html",
            "viewer/keyframes/kf-0-elevation.png",
            "viewer/events.json",
        ] {
            assert!(dir.join(f).is_file(), "{f}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// P1-14: `deeptime stage deposits --preset <synthetic> --viewer` writes the sites, the
    /// census and the bodies into the viewer.
    #[test]
    fn stage_deposits_on_synthetic_fields() {
        let dir = tmp("deposits");
        let preset = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../deeptime-core/tests/presets/deposits_synthetic_4k.json"
        )
        .to_owned();
        super::run::stage(&super::run::StageOptions {
            name: "deposits".into(),
            input: None,
            preset: Some(preset),
            start: None,
            length_myr: None,
            max_steps: None,
            out: Some(dir.join("deposits.dtcp")),
            viewer: Some(dir.join("viewer")),
            viewer_layers: Vec::new(),
            work: None,
            bake: false,
            common: quiet(),
        })
        .unwrap();
        let sites: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("viewer/sites.json")).unwrap())
                .unwrap();
        assert!(!sites.as_array().unwrap().is_empty());
        let report = std::fs::read_to_string(dir.join("viewer/report.js")).unwrap();
        assert!(report.contains("\"deposits\"") && report.contains("\"bodies\""));
        assert!(dir.join("viewer/keyframes").is_dir());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// P1-15: `run` of the deposit world preset, then `dump-deposits` over it.
    #[test]
    fn deposit_world_dumps() {
        let dir = tmp("dump");
        let preset = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../deeptime-core/tests/presets/deposits_world_4k.json"
        )
        .to_owned();
        super::run::run(&super::run::RunOptions {
            preset,
            start: None,
            length_myr: None,
            content: None,
            placement_patch: Vec::new(),
            out: dir.join("w"),
            viewer: None,
            checkpoints: false,
            stop_myr: None,
            epochs: false,
            common: quiet(),
        })
        .unwrap();
        let (sites, bodies, inside) =
            super::dump::run(&dir.join("w"), &dir.join("dump.json")).unwrap();
        assert!(sites > 0 && bodies >= sites / 2 && inside > 0);
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("dump.json")).unwrap()).unwrap();
        assert_eq!(v["circumference"], 4096);
        assert_eq!(v["sites"].as_array().unwrap().len(), sites);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// `stage` from a preset, then from the checkpoint it wrote, then bench.
    #[test]
    fn stage_chain_and_bench() {
        let dir = tmp("stage");
        let first = super::run::StageOptions {
            name: "dummy_drift".into(),
            input: None,
            preset: Some(dummy_preset()),
            start: None,
            length_myr: None,
            max_steps: None,
            out: Some(dir.join("a.dtcp")),
            viewer: None,
            viewer_layers: Vec::new(),
            work: None,
            bake: false,
            common: quiet(),
        };
        super::run::stage(&first).unwrap();
        let second = super::run::StageOptions {
            name: "dummy_erode".into(),
            input: Some(dir.join("a.dtcp")),
            preset: None,
            start: None,
            length_myr: None,
            max_steps: None,
            out: Some(dir.join("b.dtcp")),
            viewer: Some(dir.join("viewer")),
            viewer_layers: Vec::new(),
            work: None,
            bake: false,
            common: quiet(),
        };
        super::run::stage(&second).unwrap();
        let ck = deeptime_core::checkpoint::Checkpoint::read(&dir.join("b.dtcp")).unwrap();
        assert_eq!(ck.next_stage, 2);
        assert_eq!(ck.world.sites.len(), 3);
        assert!(dir.join("viewer/sites.json").is_file());
        // The same stage again from `b` is refused: it already ran.
        let again = super::run::StageOptions {
            input: Some(dir.join("b.dtcp")),
            out: None,
            viewer: None,
            viewer_layers: Vec::new(),
            ..second
        };
        let err = super::run::stage(&again).unwrap_err().to_string();
        assert!(err.contains("already ran"), "{err}");

        super::run::bench(&super::run::BenchOptions {
            preset: dummy_preset(),
            start: None,
            length_myr: None,
            threads: vec![1, 2],
            out: dir.join("bench.json"),
            viewer: Some(dir.join("bench")),
            work: None,
            mem_cap: 0,
            verbose: false,
            compute: None,
        })
        .unwrap();
        let bench: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("bench.json")).unwrap())
                .unwrap();
        assert_eq!(bench["runs"].as_array().unwrap().len(), 2);
        assert_eq!(bench["runs"][1]["threads"], 2);
        assert!(bench["runs"][0]["peak_bytes"].as_u64().unwrap() > 0);
        // The estimate the editor shows sits beside the measurement (P1-17's memory gate).
        assert!(bench["estimate"]["peak_bytes"].as_u64().unwrap() > 0);
        assert!(!bench["estimate"]["stages"].as_array().unwrap().is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The P1-03 CLI path in miniature: a plate preset by name with the timeline chosen on the
    /// command line, stopped mid-stage, then finished from the checkpoint.
    #[test]
    fn plates_stage_runs_from_a_checkpoint() {
        let dir = tmp("plates");
        std::fs::create_dir_all(&dir).unwrap();
        let json = super::run::load_preset_with(
            "plates_quick_16k",
            Some("scattered_continents"),
            Some(20.0),
        )
        .unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
        v["resolution"]["l0_samples"] = serde_json::json!(3000);
        v["resolution"]["l1_r"] = serde_json::json!(64);
        v["keyframes"]["size"] = serde_json::json!([64, 32]);
        v["keyframes"]["every_myr"] = serde_json::json!(10.0);
        let preset = dir.join("small.json");
        std::fs::write(&preset, v.to_string()).unwrap();
        let stage = |input: Option<std::path::PathBuf>,
                     preset: Option<String>,
                     max_steps: Option<u64>,
                     out: Option<std::path::PathBuf>,
                     viewer: Option<std::path::PathBuf>,
                     work: Option<std::path::PathBuf>| {
            super::run::stage(&super::run::StageOptions {
                name: "plates".into(),
                input,
                preset,
                start: None,
                length_myr: None,
                max_steps,
                out,
                viewer,
                viewer_layers: Vec::new(),
                work,
                bake: false,
                common: quiet(),
            })
        };
        // No start chosen: refused (there is no default story).
        let refused = stage(
            None,
            Some("plates_quick_16k".into()),
            Some(1),
            None,
            None,
            Some(dir.join("refused")),
        );
        assert!(refused.unwrap_err().to_string().contains("timeline.start"));
        stage(
            None,
            Some(preset.to_string_lossy().into_owned()),
            Some(4),
            Some(dir.join("mid.dtcp")),
            None,
            None,
        )
        .unwrap();
        let mid = deeptime_core::checkpoint::Checkpoint::read(&dir.join("mid.dtcp")).unwrap();
        assert_eq!(mid.next_stage_name(), Some("plates"));
        assert!(!mid.stage_state.is_empty());
        assert_eq!(mid.world.t_years, -1.2e7);
        stage(
            Some(dir.join("mid.dtcp")),
            None,
            None,
            Some(dir.join("end.dtcp")),
            Some(dir.join("viewer")),
            None,
        )
        .unwrap();
        let end = deeptime_core::checkpoint::Checkpoint::read(&dir.join("end.dtcp")).unwrap();
        assert_eq!(end.next_stage, 1);
        assert_eq!(end.world.t_years, 0.0);
        assert!(end.world.plates.is_some());
        assert!(dir.join("viewer/keyframes/kf-0-elevation.png").is_file());
        assert!(dir.join("viewer/keyframes/kf-0-crust_age.png").is_file());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// P1-04: `stage surface` runs the plates + surface group; a checkpoint written at the plates
    /// stage (`stage plates --max-steps`) continues with `stage surface --in`.
    #[test]
    fn surface_stage_runs_from_a_plates_checkpoint() {
        let dir = tmp("surface");
        std::fs::create_dir_all(&dir).unwrap();
        let json = super::run::load_preset_with(
            "pipeline_quick_16k",
            Some("scattered_continents"),
            Some(20.0),
        )
        .unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
        v["resolution"]["l0_samples"] = serde_json::json!(3000);
        v["resolution"]["l1_r"] = serde_json::json!(64);
        v["keyframes"]["size"] = serde_json::json!([64, 32]);
        v["keyframes"]["every_myr"] = serde_json::json!(10.0);
        // This test is about the plates + surface group; `stage deposits` has its own test.
        v.as_object_mut().unwrap().remove("deposits");
        v.as_object_mut().unwrap().remove("young");
        v.as_object_mut().unwrap().remove("amplify");
        let preset = dir.join("small.json");
        std::fs::write(&preset, v.to_string()).unwrap();
        let stage = |name: &str,
                     input: Option<std::path::PathBuf>,
                     preset: Option<String>,
                     max_steps: Option<u64>,
                     out: Option<std::path::PathBuf>,
                     viewer: Option<std::path::PathBuf>| {
            super::run::stage(&super::run::StageOptions {
                name: name.into(),
                input,
                preset,
                start: None,
                length_myr: None,
                max_steps,
                out,
                viewer,
                viewer_layers: Vec::new(),
                work: None,
                bake: false,
                common: quiet(),
            })
        };
        stage(
            "plates",
            None,
            Some(preset.to_string_lossy().into_owned()),
            Some(3),
            Some(dir.join("plates.dtcp")),
            None,
        )
        .unwrap();
        let mid = deeptime_core::checkpoint::Checkpoint::read(&dir.join("plates.dtcp")).unwrap();
        assert_eq!(mid.next_stage_name(), Some("plates"));
        assert_eq!(
            mid.stage_names,
            [
                "plates",
                "climate",
                "surface",
                "volcanoes",
                "impacts",
                "structure"
            ]
        );
        assert!(mid.world.surface.is_some(), "the group steps every member");
        assert!(mid.world.fields.contains_key("climate/koppen"));
        stage(
            "surface",
            Some(dir.join("plates.dtcp")),
            None,
            None,
            Some(dir.join("end.dtcp")),
            Some(dir.join("viewer")),
        )
        .unwrap();
        let end = deeptime_core::checkpoint::Checkpoint::read(&dir.join("end.dtcp")).unwrap();
        assert_eq!(end.next_stage, 6);
        assert_eq!(end.world.t_years, 0.0);
        let st = end.world.surface.as_ref().unwrap();
        assert_eq!(st.totals.steps, 10);
        assert!(
            end.world
                .history
                .as_ref()
                .is_some_and(|h| h.frame_count() == 2)
        );
        assert!(
            dir.join("viewer/keyframes/kf-0-surface_drainage.png")
                .is_file()
        );
        assert!(
            dir.join("viewer/keyframes/kf-0-surface_class.png")
                .is_file()
        );
        // P1-07: the structure stage's rock layer, the baked stacks and a cross-section.
        assert!(
            dir.join("viewer/keyframes/kf-0-structure_rock.png")
                .is_file()
        );
        assert!(dir.join("viewer/layers/strata.js").is_file());
        assert!(dir.join("viewer/sections/section-0.png").is_file());
        assert!(
            dir.join("viewer/keyframes/kf-0-climate_koppen.png")
                .is_file()
        );
        // `stage surface --preset` runs the group from the start, and so does `stage climate`.
        stage(
            "climate",
            None,
            Some(preset.to_string_lossy().into_owned()),
            None,
            None,
            None,
        )
        .unwrap();
        stage(
            "surface",
            None,
            Some(preset.to_string_lossy().into_owned()),
            None,
            None,
            None,
        )
        .unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }
    /// P1-14: `stage deposits --preset pipeline_quick_16k` runs plates, surface and structure and
    /// then the deposit stage on the history they recorded; the census names the recorded history,
    /// the viewer bundle carries the deposit layers and the sites, and a checkpoint taken before
    /// the deposit stage continues with `stage deposits --in`.
    #[test]
    fn deposit_stage_reads_the_recorded_history() {
        let dir = tmp("deposits-real");
        std::fs::create_dir_all(&dir).unwrap();
        let json = super::run::load_preset_with(
            "pipeline_quick_16k",
            Some("scattered_continents"),
            Some(20.0),
        )
        .unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
        v["resolution"]["l0_samples"] = serde_json::json!(3000);
        v["resolution"]["l1_r"] = serde_json::json!(64);
        v["keyframes"]["size"] = serde_json::json!([64, 32]);
        v["keyframes"]["every_myr"] = serde_json::json!(10.0);
        v["deposits"]["count_mult"] = serde_json::json!(200.0);
        // The young landscape and amplification (P1-06) have their own CLI test.
        v.as_object_mut().unwrap().remove("young");
        v.as_object_mut().unwrap().remove("amplify");
        let preset = dir.join("small.json");
        std::fs::write(&preset, v.to_string()).unwrap();
        let stage = |name: &str,
                     input: Option<std::path::PathBuf>,
                     preset: Option<String>,
                     out: Option<std::path::PathBuf>,
                     viewer: Option<std::path::PathBuf>| {
            super::run::stage(&super::run::StageOptions {
                name: name.into(),
                input,
                preset,
                start: None,
                length_myr: None,
                max_steps: None,
                out,
                viewer,
                viewer_layers: Vec::new(),
                work: None,
                bake: false,
                common: quiet(),
            })
        };
        // From the preset: the stages before it run first.
        stage(
            "deposits",
            None,
            Some(preset.to_string_lossy().into_owned()),
            Some(dir.join("end.dtcp")),
            Some(dir.join("viewer")),
        )
        .unwrap();
        let end = deeptime_core::checkpoint::Checkpoint::read(&dir.join("end.dtcp")).unwrap();
        assert_eq!(
            end.stage_names,
            [
                "plates",
                "climate",
                "surface",
                "volcanoes",
                "impacts",
                "structure",
                "deposits"
            ]
        );
        assert_eq!(end.next_stage_name(), None, "the run is complete");
        assert!(
            end.world.history.is_some(),
            "the history stays in the world"
        );
        let census: serde_json::Value =
            serde_json::from_str(&end.world.notes[deeptime_core::deposits::STAGE_NAME]).unwrap();
        assert_eq!(census["history"]["source"], "recorded");
        assert_eq!(census["skipped"], serde_json::json!([]));
        assert!(
            std::fs::read_dir(dir.join("viewer/keyframes"))
                .unwrap()
                .any(|e| e
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .contains("deposits_sites"))
        );
        assert!(dir.join("viewer/sites.json").is_file());
        assert!(dir.join("viewer/report.js").is_file());
        // The strata inspector stays with the structure bundle (its layers are ~20 MB).
        assert!(!dir.join("viewer/layers/strata.js").exists());
        // From a checkpoint taken before the stage: the group's end, next stage deposits.
        stage(
            "structure",
            None,
            Some(preset.to_string_lossy().into_owned()),
            Some(dir.join("group.dtcp")),
            None,
        )
        .unwrap();
        let group = deeptime_core::checkpoint::Checkpoint::read(&dir.join("group.dtcp")).unwrap();
        assert_eq!(group.next_stage_name(), Some("deposits"));
        stage("deposits", Some(dir.join("group.dtcp")), None, None, None).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// P1-08: `run` bakes a planet run (every chunk-time layer, format 0.2), `parity` writes the
    /// fixture from it and `inspect-column` answers a column, all through the one evaluator.
    #[test]
    fn a_planet_run_bakes_and_parity_samples_it() {
        let dir = tmp("planet");
        let preset = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../deeptime-core/tests/presets/planet_world_4k.json"
        );
        let mut common = quiet();
        common.set = vec![
            "resolution.l0_samples=2000".into(),
            "resolution.l1_r=32".into(),
            "resolution.l2_r=16".into(),
            "resolution.l3_r=8".into(),
            "keyframes.size=[64, 32]".into(),
            "amplify.erosion_iterations=4".into(),
            "amplify.thermal_iterations=3".into(),
            "amplify.deposition_iterations=2".into(),
        ];
        super::run::run(&super::run::RunOptions {
            preset: preset.into(),
            start: None,
            length_myr: Some(40.0),
            content: None,
            placement_patch: Vec::new(),
            out: dir.join("world"),
            viewer: None,
            checkpoints: false,
            stop_myr: None,
            epochs: false,
            common,
        })
        .unwrap();
        let m = deeptime_format::Manifest::read(&dir.join("world")).unwrap();
        assert_eq!(m.format_version, "0.5");
        assert_eq!(m.notes["bake"]["baked"], true);
        // The bake says how much of its terrain stands above the build limit (whatever the planet).
        let limit = &m.notes["bake"]["terrain"]["build_limit"];
        assert_eq!(limit["max_y"], 639);
        assert!(limit["land_cells"].as_u64().is_some_and(|n| n > 0));
        assert!(limit["cells_above"].as_u64().unwrap() <= limit["land_cells"].as_u64().unwrap());
        assert_eq!(
            limit["cells_above"].as_u64().unwrap() > 0,
            limit["highest_y"].as_f64().unwrap() > 639.0
        );
        let out = dir.join("world/parity/columns.json");
        let (n, _) = super::parity::run(&dir.join("world"), &out, 60, 7).unwrap();
        assert_eq!(n, 60);
        let doc: serde_json::Value = serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        let cols = doc["columns"].as_array().unwrap();
        assert_eq!(cols.len(), 60);
        assert_eq!(
            doc["circumference"], 3072,
            "the 4k planet drawn as a Petroff–Guyou world: 2b, b = 1,536"
        );
        let w = deeptime_format::ColumnWorld::open(&dir.join("world")).unwrap();
        // Compared as text: serde_json's default parser may land a float one ulp off (Java's
        // Double.parseDouble does not), while its writer prints the shortest exact form.
        let text = String::from_utf8(std::fs::read(&out).unwrap()).unwrap();
        for c in cols {
            let (x, z) = (c["x"].as_i64().unwrap(), c["z"].as_i64().unwrap());
            let fresh = w.sample(x, z).to_json().to_string();
            assert!(
                text.contains(&fresh),
                "the fixture holds what the evaluator says: {fresh}"
            );
        }
        // P1-25: the climate fixture, and the evaluator it holds.
        let cout = dir.join("world/parity/climate.json");
        assert_eq!(
            super::parity::run_climate(&dir.join("world"), &cout, 30, 7).unwrap(),
            Some(30)
        );
        let ctext = String::from_utf8(std::fs::read(&cout).unwrap()).unwrap();
        let cdoc: serde_json::Value = serde_json::from_str(&ctext).unwrap();
        let cw = deeptime_format::ClimateWorld::open(&dir.join("world"))
            .unwrap()
            .expect("a planet run bakes the twelve months");
        for s in cdoc["samples"].as_array().unwrap() {
            let (x, y, z) = (
                s["x"].as_i64().unwrap(),
                s["y"].as_i64().unwrap() as i32,
                s["z"].as_i64().unwrap(),
            );
            let fresh = cw.sample(x, y, z);
            assert!(ctext.contains(&fresh.to_json().to_string()));
            assert!(fresh.t_warm >= fresh.t_mean && fresh.t_mean >= fresh.t_cold);
            let p: f64 = fresh.p_month.iter().sum();
            assert!((p - fresh.p_annual).abs() < 1e-6 * fresh.p_annual.max(1.0));
        }
        // (The 4k planet's world is 3,072 blocks round: x in [−1,536, 1,536).)
        let v = w.inspect(-1500, 300);
        assert_eq!(v["wx"], -1500);
        assert!(v["layers"]["climate/temperature"]["bilinear"].is_number());
        assert!(v["strata"].as_array().is_some_and(|a| !a.is_empty()));
        // `--why`: a column inside a site's box gets the site's forming event and chain.
        if let Some((sites, _)) = deeptime_format::sites::read_sites(&dir.join("world")).unwrap()
            && let Some(site) = sites.first()
        {
            let [cx, _, cz] = site.center;
            let mut v = w.inspect(i64::from(cx), i64::from(cz));
            super::attach_why(&mut v, &dir.join("world")).unwrap();
            let held = v["sites"].as_array().unwrap();
            assert!(
                held.iter().any(|s| s["id"] == site.id),
                "the site holds its centre"
            );
            assert!(held.iter().all(|s| s["why"].is_object()), "{v}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
