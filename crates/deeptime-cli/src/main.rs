// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime`: the standalone driver for every simulation stage (plan §9.3).

mod alloc;
mod dump;
mod run;
mod synth;

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
        Some(Command::Stage {
            name,
            input,
            preset,
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
                out,
                viewer,
                work,
                common: common.common(),
            }),
        ),
        Some(Command::Bench {
            preset,
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
                threads,
                out,
                viewer,
                work,
                mem_cap,
                verbose,
            }),
        ),
        Some(Command::Preset { action }) => finish("preset", preset_command(action)),
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
}
