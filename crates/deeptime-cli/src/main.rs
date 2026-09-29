// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime`: the standalone driver for every simulation stage (plan §9.3).

mod alloc;
mod dump;
mod run;
mod synth;
mod watch;

use std::net::IpAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

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
        /// World-data folder to write (`<world>/deeptime`).
        #[arg(long)]
        out: PathBuf,
        /// Also write an HTML viewer bundle here.
        #[arg(long)]
        viewer: Option<PathBuf>,
        /// Also write a checkpoint after every stage into <out>/checkpoints/.
        #[arg(long)]
        checkpoints: bool,
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Run a planet like `run` and serve a live page of it while it runs (map, globe, layers and
    /// events every step; pause, step, rewind, speed, follow live). Serves loopback or a Tailscale
    /// address only, never the LAN; tools/viewer/mac-watch.sh runs it on the Mac and puts it on
    /// the tailnet.
    Watch {
        /// A built-in preset name, a preset JSON file or a share code (DT1.…).
        #[arg(long)]
        preset: String,
        /// Timeline start configuration (supercontinent, scattered_continents, random).
        #[arg(long)]
        start: Option<String>,
        /// Timeline length in millions of years.
        #[arg(long)]
        length_myr: Option<f64>,
        /// Content bundle JSON (rock classes, deposit types; plan §5.4).
        #[arg(long)]
        content: Option<PathBuf>,
        /// Address to serve on: loopback (default) or the machine's Tailscale address.
        #[arg(long, default_value = "127.0.0.1")]
        bind: IpAddr,
        /// Port to serve on.
        #[arg(long, default_value_t = watch::DEFAULT_PORT)]
        port: u16,
        /// Folder for the world data (<work>/world) and the frame images (<work>/frames); default
        /// a temporary folder.
        #[arg(long)]
        work: Option<PathBuf>,
        /// A frame every this many Myr (default: after every step).
        #[arg(long)]
        every_myr: Option<f64>,
        /// Also write the static viewer bundle here when the run is done.
        #[arg(long)]
        viewer: Option<PathBuf>,
        /// Stop serving when the run is done (default: serve until stopped).
        #[arg(long)]
        exit_when_done: bool,
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
        /// Folder for the stage's keyframes (default <out>.work, or a temporary folder).
        #[arg(long)]
        work: Option<PathBuf>,
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
}

impl CommonArgs {
    fn common(&self) -> run::Common {
        run::Common {
            threads: self.threads,
            mem_cap: self.mem_cap,
            verbose: self.verbose,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum StrataArg {
    /// Three horizons: sandstone, red beds from 6 blocks down, granite from 40 (P0-09).
    Placeholder,
    /// All 59 rock classes: 8 horizons 10 blocks apart; any ten neighbouring 16-block cell
    /// columns hold every class 20 or more blocks below the surface (P1-12's GameTests).
    EveryClass,
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
            out,
            viewer,
            checkpoints,
            common,
        }) => finish(
            "run",
            run::run(&run::RunOptions {
                preset,
                start,
                length_myr,
                content,
                out,
                viewer,
                checkpoints,
                common: common.common(),
            }),
        ),
        Some(Command::Watch {
            preset,
            start,
            length_myr,
            content,
            bind,
            port,
            work,
            every_myr,
            viewer,
            exit_when_done,
            common,
        }) => finish(
            "watch",
            watch::watch(&watch::WatchOptions {
                preset,
                start,
                length_myr,
                content,
                bind,
                port,
                work,
                every_myr,
                viewer,
                exit_when_done,
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
            work,
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
                work,
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
            }),
        ),
        Some(Command::Preset { action }) => finish("preset", preset_command(action)),
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

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

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
            out: dir.join("dummy"),
            viewer: Some(dir.join("viewer")),
            checkpoints: true,
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
            work: None,
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
            out: dir.join("w"),
            viewer: None,
            checkpoints: false,
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
            work: None,
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
            work: None,
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
        })
        .unwrap();
        let bench: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("bench.json")).unwrap())
                .unwrap();
        assert_eq!(bench["runs"].as_array().unwrap().len(), 2);
        assert_eq!(bench["runs"][1]["threads"], 2);
        assert!(bench["runs"][0]["peak_bytes"].as_u64().unwrap() > 0);
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
                work,
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
                work: None,
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
        assert_eq!(mid.stage_names, ["plates", "surface"]);
        assert!(mid.world.surface.is_some(), "the group steps both stages");
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
        assert_eq!(end.next_stage, 2);
        assert_eq!(end.world.t_years, 0.0);
        let st = end.world.surface.as_ref().unwrap();
        assert_eq!(st.totals.steps, 10);
        assert!(
            end.world
                .history
                .as_ref()
                .is_some_and(|h| h.frames.len() == 2)
        );
        assert!(
            dir.join("viewer/keyframes/kf-0-surface_drainage.png")
                .is_file()
        );
        assert!(
            dir.join("viewer/keyframes/kf-0-surface_class.png")
                .is_file()
        );
        // `stage surface --preset` runs the group from the start.
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
}
