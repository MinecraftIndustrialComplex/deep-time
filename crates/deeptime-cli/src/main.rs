// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime`: the standalone driver for every simulation stage (plan §9.3).

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
}
