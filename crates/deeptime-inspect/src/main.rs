// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime-inspect`: renders Minecraft region files for review (plan §9.7). A dev tool; never
//! linked into the mod.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use deeptime_inspect::colors::{self, ColorTable};
use deeptime_inspect::palette::Palette;
use deeptime_inspect::region;
use deeptime_inspect::render::{self, Axis, Out, SeamArgs, SectionArgs};
use deeptime_inspect::surface::ExcludeOpts;
use deeptime_inspect::world::{Area, World};
use deeptime_inspect::worlddata::WorldData;
use deeptime_inspect::{Error, Result};

#[derive(Parser)]
#[command(
    name = "deeptime-inspect",
    version,
    about = "Renders and checks Minecraft 1.21.1 region files (Deep Time plan §9.7)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Args)]
struct Common {
    /// The world or dimension folder (the one holding region/).
    #[arg(long)]
    world: PathBuf,
    /// Output folder for PNGs and JSON.
    #[arg(long, default_value = "review/out/inspect")]
    out: PathBuf,
    /// Pixels per block in the images.
    #[arg(long, default_value_t = 1)]
    scale: usize,
}

#[derive(clap::Args)]
struct Exclude {
    /// Carved-column test: largest half-width of the closing's square, in columns.
    #[arg(long, default_value_t = 32)]
    pit_radius: i32,
    /// Carved-column test: half-width of the opening that first removes narrow bumps.
    #[arg(long, default_value_t = 4)]
    bump_radius: i32,
    /// Carved-column test: blocks below the closing that make a pit.
    #[arg(long, default_value_t = 2)]
    pit_depth: i32,
    /// Carved-column test: columns around a pit also left out (its rim).
    #[arg(long, default_value_t = 2)]
    pit_margin: i32,
    /// Structure test: margin around structure pieces, in blocks.
    #[arg(long, default_value_t = 12)]
    structure_margin: i32,
}

impl Exclude {
    fn opts(&self) -> ExcludeOpts {
        ExcludeOpts {
            pit_radius: self.pit_radius,
            bump_radius: self.bump_radius,
            pit_depth: self.pit_depth,
            pit_margin: self.pit_margin,
            structure_margin: self.structure_margin,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum AxisArg {
    X,
    Z,
}

#[derive(Subcommand)]
enum Cmd {
    /// Builds the block colour table from the jars' own top textures.
    Colors {
        /// A mods folder (every *.jar in it; nested jar-in-jar included).
        #[arg(long)]
        jars: Option<PathBuf>,
        /// Minecraft's client resources jar (neoforge/build/moddev/artifacts/*-client-extra*.jar).
        #[arg(long)]
        vanilla: PathBuf,
        #[arg(long, default_value = "review/out/inspect/colors.json")]
        out: PathBuf,
    },
    /// Top-down view: the highest non-air block per column.
    Topdown {
        #[command(flatten)]
        common: Common,
        /// x0,z0,x1,z1 (x1, z1 exclusive); default: each group of generated chunks.
        #[arg(long, value_parser = Area::parse, allow_hyphen_values = true)]
        area: Option<Area>,
        /// Colour table from `colors` (default: built-in fallback colours).
        #[arg(long)]
        colors: Option<PathBuf>,
    },
    /// Rock class of the first solid block below the surface material.
    Rock {
        #[command(flatten)]
        common: Common,
        #[arg(long, value_parser = Area::parse, allow_hyphen_values = true)]
        area: Option<Area>,
        #[arg(long)]
        colors: Option<PathBuf>,
        /// Block → class dump (`/deeptime probe palette`); overrides --tags.
        #[arg(long)]
        palette: Option<PathBuf>,
        /// Jars, zips or folders with #deeptime:rock/<class> tags (default: the world's
        /// ../mods, its datapacks/ and ./neoforge/src/main/resources).
        #[arg(long)]
        tags: Vec<PathBuf>,
    },
    /// Ore blocks by material, projected from above, shaded by depth.
    Ore {
        #[command(flatten)]
        common: Common,
        #[arg(long, value_parser = Area::parse, allow_hyphen_values = true)]
        area: Option<Area>,
    },
    /// A vertical cross-section; prints water_top_y=<y> (the top face of the highest water
    /// block) for every water surface it crosses.
    Section {
        #[command(flatten)]
        common: Common,
        /// The fixed coordinate: `z` for the plane z = <at> (running along x), `x` for x = <at>.
        #[arg(long, value_enum)]
        axis: AxisArg,
        #[arg(long, allow_hyphen_values = true)]
        at: i32,
        #[arg(long, allow_hyphen_values = true)]
        from: i32,
        #[arg(long, allow_hyphen_values = true)]
        to: i32,
        #[arg(long, allow_hyphen_values = true)]
        y_from: Option<i32>,
        #[arg(long, allow_hyphen_values = true)]
        y_to: Option<i32>,
        /// World data for the overlays (default: <world>/deeptime if present).
        #[arg(long)]
        world_data: Option<PathBuf>,
        #[arg(long)]
        colors: Option<PathBuf>,
    },
    /// The columns either side of x = C/2 as one strip, and their height difference.
    Seam {
        #[command(flatten)]
        common: Common,
        /// Default: <world>/deeptime.
        #[arg(long)]
        world_data: Option<PathBuf>,
        /// The circumference C (default: the world data's).
        #[arg(long)]
        span: Option<i64>,
        #[arg(long, default_value_t = 128)]
        width: i32,
        #[arg(long, allow_hyphen_values = true)]
        z_from: Option<i32>,
        #[arg(long, allow_hyphen_values = true)]
        z_to: Option<i32>,
        #[arg(long)]
        colors: Option<PathBuf>,
        #[command(flatten)]
        exclude: Exclude,
    },
    /// Counts the distinct blocks in an area (all heights), by block id and by namespace. Used to
    /// give BlueMap only the mod jars whose blocks a review render actually contains.
    Blocks {
        /// The world or dimension folder (the one holding region/).
        #[arg(long)]
        world: PathBuf,
        /// x0,z0,x1,z1 (x1, z1 exclusive).
        #[arg(long, value_parser = Area::parse)]
        area: Area,
    },
    /// Lists region files; prints outside=<n> for files outside -C/1024 <= rx < C/1024.
    Regions {
        #[arg(long)]
        world: PathBuf,
        /// The circumference C (default: <world>/deeptime's).
        #[arg(long)]
        span: Option<i64>,
        /// Also write the listing as JSON here.
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Generated surface height vs the world data's elevation: JSON, a histogram and a map.
    Compare {
        #[command(flatten)]
        common: Common,
        #[arg(long)]
        world_data: PathBuf,
        #[arg(long, value_parser = Area::parse, allow_hyphen_values = true)]
        area: Option<Area>,
        #[command(flatten)]
        exclude: Exclude,
    },
    /// Prints one column's blocks from its top block down, with the measured surface.
    Column {
        #[arg(long)]
        world: PathBuf,
        #[arg(long, allow_hyphen_values = true)]
        x: i32,
        #[arg(long, allow_hyphen_values = true)]
        z: i32,
        /// How many blocks below the top block to print.
        #[arg(long, default_value_t = 24)]
        depth: i32,
    },
    /// Copies chunks cx0..=cx1, cz0..=cz1 of one region file into a new one (test fixtures).
    Fixture {
        #[arg(long)]
        region: PathBuf,
        /// cx0,cz0,cx1,cz1 (global chunk coordinates, inclusive).
        #[arg(long, allow_hyphen_values = true)]
        chunks: String,
        #[arg(long)]
        out: PathBuf,
    },
}

fn out_of(c: &Common) -> Out {
    Out {
        dir: c.out.clone(),
        scale: c.scale.max(1),
    }
}

fn default_world_data(world: &Path, given: Option<&PathBuf>) -> Result<Option<WorldData>> {
    match given {
        Some(p) => Ok(Some(WorldData::open(p)?)),
        None => {
            let p = world.join("deeptime");
            if p.join("manifest.json").is_file() {
                Ok(Some(WorldData::open(&p)?))
            } else {
                Ok(None)
            }
        }
    }
}

fn span_of(world: &Path, span: Option<i64>, wd: Option<&WorldData>) -> Result<i64> {
    if let Some(s) = span {
        return Ok(s);
    }
    if let Some(w) = wd {
        return Ok(w.circumference);
    }
    Err(Error(format!(
        "no --span and no world data at {}/deeptime",
        world.display()
    )))
}

fn print_json(v: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::Colors { jars, vanilla, out } => {
            let (ok, missing) = colors::build(jars.as_deref(), &vanilla, &out)?;
            println!("blocks={ok} unresolved={missing} out={}", out.display());
        }
        Cmd::Topdown {
            common,
            area,
            colors,
        } => {
            let table = ColorTable::from_arg(colors.as_deref())?;
            let v = render::topdown(&common.world, area, &table, &out_of(&common))?;
            print_json(&v["images"])?;
        }
        Cmd::Rock {
            common,
            area,
            colors,
            palette,
            tags,
        } => {
            let table = ColorTable::from_arg(colors.as_deref())?;
            let pal = match palette {
                Some(p) => Palette::from_dump(&p)?,
                None => {
                    let sources = if tags.is_empty() {
                        render::default_tag_sources(&common.world)
                    } else {
                        tags
                    };
                    Palette::from_tags(&sources)?
                }
            };
            if pal.blocks.is_empty() {
                eprintln!(
                    "rock: no #deeptime:rock/<class> tags found; every block is unmapped (pass --tags or --palette)"
                );
            }
            let v = render::rock(&common.world, area, &pal, &table, &out_of(&common))?;
            print_json(
                &serde_json::json!({"images": v["images"], "classes": v["classes"], "unmapped_blocks": v["unmapped_blocks"]}),
            )?;
        }
        Cmd::Ore { common, area } => {
            // Site footprints from the world's own world data, when it has sites (P1-15).
            let sites: Vec<render::SiteBox> =
                match deeptime_format::sites::read_sites(&common.world.join("deeptime")) {
                    Ok(Some((sites, _))) => sites
                        .iter()
                        .map(|s| render::SiteBox {
                            id: s.id,
                            deposit_type: s.deposit_type.clone(),
                            x0: s.aabb[0][0],
                            z0: s.aabb[0][2],
                            x1: s.aabb[1][0],
                            z1: s.aabb[1][2],
                        })
                        .collect(),
                    _ => Vec::new(),
                };
            let v = render::ore_with_sites(&common.world, area, &sites, &out_of(&common))?;
            print_json(
                &serde_json::json!({"images": v["images"], "materials": v["materials"],
                "sites": v["sites"], "ore_columns_in_site_footprints": v["ore_columns_in_site_footprints"],
                "ore_columns_outside_sites": v["ore_columns_outside_sites"]}),
            )?;
        }
        Cmd::Section {
            common,
            axis,
            at,
            from,
            to,
            y_from,
            y_to,
            world_data,
            colors,
        } => {
            let table = ColorTable::from_arg(colors.as_deref())?;
            let wd = default_world_data(&common.world, world_data.as_ref())?;
            let args = SectionArgs {
                axis: match axis {
                    AxisArg::X => Axis::X,
                    AxisArg::Z => Axis::Z,
                },
                at,
                from,
                to,
                y_from,
                y_to,
            };
            let mut lines = Vec::new();
            let v = render::section(
                &common.world,
                wd.as_ref(),
                &args,
                &table,
                &out_of(&common),
                &mut lines,
            )?;
            println!("png={}", v["png"].as_str().unwrap_or(""));
            for l in lines {
                println!("{l}");
            }
        }
        Cmd::Seam {
            common,
            world_data,
            span,
            width,
            z_from,
            z_to,
            colors,
            exclude,
        } => {
            let table = ColorTable::from_arg(colors.as_deref())?;
            let wd = default_world_data(&common.world, world_data.as_ref())?;
            let span = span_of(&common.world, span, wd.as_ref())?;
            let z_range = match (z_from, z_to) {
                (Some(a), Some(b)) => Some((a.min(b), a.max(b))),
                (None, None) => None,
                _ => return Err(Error::new("give both --z-from and --z-to, or neither")),
            };
            let args = SeamArgs {
                span,
                width,
                z_range,
                opts: exclude.opts(),
            };
            let v = render::seam(&common.world, wd.as_ref(), &args, &table, &out_of(&common))?;
            println!(
                "seam_rows={} seam_mismatches={} p99_abs_diff_blocks={} png={} diff_png={}",
                v["seam_rows"],
                v["seam_mismatches"],
                v["p99_abs_diff_blocks"],
                v["png"],
                v["diff_png"]
            );
        }
        Cmd::Blocks { world, area } => {
            let w = World::load_area(&world, Some(area))?;
            let mut by_id: std::collections::BTreeMap<String, u64> = Default::default();
            let mut counts: std::collections::HashMap<u32, u64> = std::collections::HashMap::new();
            for (&(cx, cz), chunk) in &w.chunks {
                if !area.contains_chunk(cx, cz) {
                    continue;
                }
                for x in 0..16 {
                    for z in 0..16 {
                        let (bx, bz) = (cx * 16 + x as i32, cz * 16 + z as i32);
                        if bx < area.x0 || bx >= area.x1 || bz < area.z0 || bz >= area.z1 {
                            continue;
                        }
                        for y in chunk.min_y()..=chunk.max_y() {
                            *counts.entry(chunk.get(x, y, z)).or_default() += 1;
                        }
                    }
                }
            }
            let mut by_ns: std::collections::BTreeMap<String, u64> = Default::default();
            for (id, n) in counts {
                let name = w.reg.states[id as usize].name.clone();
                let ns = name.split(':').next().unwrap_or("minecraft").to_string();
                *by_ns.entry(ns).or_default() += n;
                *by_id.entry(name).or_default() += n;
            }
            print_json(
                &serde_json::json!({"area": area.to_json(), "namespaces": by_ns, "blocks": by_id}),
            )?;
        }
        Cmd::Regions { world, span, json } => {
            let wd = default_world_data(&world, None)?;
            let span = span_of(&world, span, wd.as_ref())?;
            let mut lines = Vec::new();
            let v = render::regions(&world, span, &mut lines)?;
            for l in lines {
                println!("{l}");
            }
            if let Some(p) = json {
                std::fs::write(&p, serde_json::to_string_pretty(&v)?)
                    .map_err(|e| Error::io(&p, e))?;
            }
        }
        Cmd::Compare {
            common,
            world_data,
            area,
            exclude,
        } => {
            let wd = WorldData::open(&world_data)?;
            let v = render::compare(&common.world, &wd, area, exclude.opts(), &out_of(&common))?;
            // The summary without the per-area detail, which is in compare.json.
            let mut s = v.clone();
            if let Some(o) = s.as_object_mut() {
                o.remove("maps");
                o.remove("definition");
                o.insert(
                    "json".into(),
                    common.out.join("compare.json").display().to_string().into(),
                );
                o.insert(
                    "maps".into(),
                    v["maps"]
                        .as_array()
                        .map(|a| a.iter().map(|m| m["png"].clone()).collect::<Vec<_>>())
                        .unwrap_or_default()
                        .into(),
                );
            }
            print_json(&s)?;
        }
        Cmd::Column { world, x, z, depth } => {
            let w = deeptime_inspect::world::World::load(&world, |cx, cz| {
                cx == x >> 4 && cz == z >> 4
            })?;
            let Some(c) = w.chunk(x, z) else {
                return Err(Error(format!("no chunk at {x},{z}")));
            };
            println!(
                "chunk {},{} status={} top={:?} terrain_top={:?} water_top={:?}",
                c.cx,
                c.cz,
                c.status,
                w.top(x, z),
                w.terrain_top(x, z),
                w.water_top(x, z)
            );
            if let Some(t) = w.top(x, z) {
                // Around the terrain surface: `depth` blocks above it (up to the top) and below.
                let mid = w.terrain_top(x, z).unwrap_or(t);
                for y in (mid - depth..=(mid + depth).min(t)).rev() {
                    let id = w.block(x, y, z);
                    println!(
                        "{y} {} flags={:#x}",
                        w.reg.states[id as usize].key,
                        w.reg.flags(id)
                    );
                }
            }
        }
        Cmd::Fixture {
            region: src,
            chunks,
            out,
        } => {
            let v: Vec<i32> = chunks
                .split(',')
                .map(|p| p.trim().parse::<i32>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|e| Error(format!("--chunks '{chunks}': {e}")))?;
            if v.len() != 4 {
                return Err(Error::new("--chunks must be cx0,cz0,cx1,cz1"));
            }
            let n = region::extract(&src, &out, v[0], v[1], v[2], v[3])?;
            println!("chunks={n} out={}", out.display());
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("deeptime-inspect: {e}");
            ExitCode::FAILURE
        }
    }
}
