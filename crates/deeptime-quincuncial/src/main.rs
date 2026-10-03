// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime-quincuncial`: the seamless-wrapping prototype's tool (docs/design/seamless-wrapping.md).
//!
//!   deeptime-quincuncial stats    --layout guyou --size 8192 --lon -20        measurements (JSON)
//!   deeptime-quincuncial portals  --layout guyou --size 8192 --lon -20        gluing table + /summon lines
//!   deeptime-quincuncial place    --layout guyou --size 8192 --src <world>    singular points vs land
//!   deeptime-quincuncial resample --layout guyou --size 8192 --lon -20 --src <world> --out <dir>

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use deeptime_quincuncial::resample::{resample, strip_circumference};
use deeptime_quincuncial::stats::{measure, place};
use deeptime_quincuncial::world::{Layout, QWorld};
use serde_json::json;

#[derive(Parser)]
#[command(
    name = "deeptime-quincuncial",
    about = "PROTOTYPE: quincuncial worlds glued by portals"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Args, Clone)]
struct WorldArgs {
    /// peirce (pole-centred square) or guyou (equator straight, 2:1).
    #[arg(long, default_value = "guyou")]
    layout: String,
    /// Peirce: the square's side a; Guyou: the hemisphere square's side b (the world is 2b × b).
    #[arg(long)]
    size: f64,
    /// Peirce: λ₀, the west edge midpoint's longitude; Guyou: the western hemisphere's centre.
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    lon: f64,
}

impl WorldArgs {
    fn world(&self, lon: f64) -> Result<QWorld, String> {
        match Layout::parse(&self.layout) {
            Some(Layout::Peirce) => Ok(QWorld::peirce(self.size, lon)),
            Some(Layout::Guyou) => Ok(QWorld::guyou(self.size, lon)),
            None => Err(format!("--layout {}: peirce or guyou", self.layout)),
        }
    }
}

#[derive(Subcommand)]
enum Cmd {
    /// The measurements for the design doc.
    Stats {
        #[command(flatten)]
        w: WorldArgs,
        #[arg(long, default_value_t = 1200)]
        grid: usize,
    },
    /// The gluing table and the Immersive Portals /summon command of every seam.
    Portals {
        #[command(flatten)]
        w: WorldArgs,
        #[arg(long, default_value_t = 64.0, allow_hyphen_values = true)]
        y: f64,
        #[arg(long, default_value_t = 1200.0)]
        height: f64,
    },
    /// Scans the longitude for where the singular points fall on a baked planet.
    Place {
        #[command(flatten)]
        w: WorldArgs,
        #[arg(long)]
        src: PathBuf,
        #[arg(long, default_value_t = 1.0)]
        step: f64,
        #[arg(long, default_value_t = 3.0)]
        radius: f64,
    },
    /// Resamples a baked world-data folder onto the quincuncial world.
    Resample {
        #[command(flatten)]
        w: WorldArgs,
        #[arg(long)]
        src: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// The strip's circumference C′ (default 4 × the domain's height).
        #[arg(long)]
        circumference: Option<u32>,
    },
}

fn run() -> Result<(), String> {
    match Cli::parse().command {
        Cmd::Stats { w, grid } => {
            let world = w.world(w.lon)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&measure(&world, grid)).unwrap()
            );
        }
        Cmd::Portals { w, y, height } => {
            let world = w.world(w.lon)?;
            let seams = world.seams();
            let out = json!({
                "layout": world.layout().name(),
                "domain": world.extent(),
                "strip_circumference": strip_circumference(&world),
                "portals": seams.iter().map(|s| s.portal(y, height)).collect::<Vec<_>>(),
                "summon": seams.iter().map(|s| s.summon_command(y, height)).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        }
        Cmd::Place {
            w,
            src,
            step,
            radius,
        } => {
            let ww = w.clone();
            let v = place(&src, move |lon| ww.world(lon).unwrap(), step, radius)?;
            println!("{}", serde_json::to_string_pretty(&v).unwrap());
        }
        Cmd::Resample {
            w,
            src,
            out,
            circumference,
        } => {
            let world = w.world(w.lon)?;
            let c = circumference.unwrap_or(strip_circumference(&world).round() as u32);
            let t0 = std::time::Instant::now();
            let r = resample(&src, &out, &world, c)?;
            for (name, method, bytes) in &r.layers {
                println!("{name:32} {method:10} {bytes:>12} bytes");
            }
            println!("dropped: {}", r.dropped.join(", "));
            println!("spawn candidates carried: {}", r.spawn_candidates);
            println!("C′ = {c}; {:.1} s", t0.elapsed().as_secs_f64());
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("deeptime-quincuncial: {e}");
            ExitCode::FAILURE
        }
    }
}
