// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime`: the standalone driver for every simulation stage (plan §9.3).

mod alloc;
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
        /// Preset JSON file (tests/presets/dummy.json until the real stages exist).
        #[arg(long)]
        preset: PathBuf,
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
        /// Preset to start from instead of a checkpoint.
        #[arg(long)]
        preset: Option<PathBuf>,
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
        #[arg(long)]
        preset: PathBuf,
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
    /// Write a synthetic world-data folder (Phase 0 and the GameTests).
    Synth {
        /// Which synthetic world.
        #[arg(long, value_enum)]
        kind: SynthKind,
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
            content,
            out,
            viewer,
            checkpoints,
            common,
        }) => finish(
            "run",
            run::run(&run::RunOptions {
                preset,
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
        Some(Command::Synth {
            kind,
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

    fn dummy_preset() -> std::path::PathBuf {
        std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../deeptime-core/tests/presets/dummy.json"
        ))
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
