// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime dump-deposits`: what the Rust side reads from a world's sites, bodies and events, as
//! JSON, for the Java parity test (`DepositDataParityTest`, P1-15) — the same role
//! `synth --dump-samples` plays for tile packs. Every site as the viewer's JSON, index lookups at
//! deterministic random blocks, body evaluations ([`Body::locate`] and the zones that apply) at
//! deterministic random blocks in and around every body (across the seam too), and the cause chain
//! of every site's forming event.

use std::path::Path;

use deeptime_core::synth::SplitMix;
use deeptime_format::bodies::read_bodies;
use deeptime_format::sites::read_sites;
use deeptime_format::{EventLog, FormatError, Manifest, fsutil::write_atomic};
use serde_json::{Value, json};

/// Samples per body.
pub const BODY_SAMPLES: usize = 400;

pub fn run(world: &Path, out: &Path) -> Result<(usize, usize, usize), FormatError> {
    let m = Manifest::read(world)?;
    let c = m.scale.circumference_blocks;
    let (sites, index) = read_sites(world)?
        .ok_or_else(|| FormatError::Invalid(format!("{}: no sites/", world.display())))?;
    let bodies = read_bodies(world)?.unwrap_or_default();
    let log = EventLog::load(world)?;
    let mut rng = SplitMix(0xB0D1_E5D0_0D5E_ED00);

    let mut idx = Vec::new();
    // The world's z range (pole to pole, or edge to edge) and 512 blocks past each end.
    let zp = m.world_projection().z_range().1 as i64;
    for _ in 0..500 {
        let x = rng.below(u64::from(c) * 2) as i64 - i64::from(c);
        let z = rng.below(2 * zp as u64 + 1024) as i64 - zp - 512;
        let ids = index.at_block(x as i32, z as i32).to_vec();
        idx.push(json!([x, z, ids]));
    }
    // Every site's centre column too, so the populated cells are covered.
    for s in &sites {
        let ids = index.at_block(s.center[0], s.center[2]).to_vec();
        idx.push(json!([s.center[0], s.center[2], ids]));
    }

    let mut body_out = Vec::new();
    let mut inside_total = 0usize;
    for b in &bodies {
        let [lo, hi] = b.aabb;
        let mut samples = Vec::new();
        let span = |k: usize| (i64::from(hi[k]) - i64::from(lo[k]) + 7).max(1) as u64;
        let pick = |rng: &mut SplitMix, k: usize| i64::from(lo[k]) - 3 + rng.below(span(k)) as i64;
        let mut pts: Vec<[i64; 3]> = vec![[
            b.center[0].floor() as i64,
            b.center[1].floor() as i64,
            b.center[2].floor() as i64,
        ]];
        for n in 0..BODY_SAMPLES {
            let mut p = [pick(&mut rng, 0), pick(&mut rng, 1), pick(&mut rng, 2)];
            // Every fourth sample one planet east or west: the seam wrap.
            if n % 4 == 3 {
                p[0] += if n % 8 == 3 {
                    i64::from(c)
                } else {
                    -i64::from(c)
                };
            }
            pts.push(p);
        }
        for p in pts {
            let (x, y, z) = (p[0] as i32, p[1] as i32, p[2] as i32);
            match b.locate(c, x, y, z) {
                Some(at) => {
                    inside_total += 1;
                    let zones: Vec<usize> = b
                        .zones
                        .iter()
                        .enumerate()
                        .filter(|(_, zn)| b.zones_at(at).any(|q| std::ptr::eq(q, *zn)))
                        .map(|(i, _)| i)
                        .collect();
                    samples.push(json!([x, y, z, at.d, at.v, zones]));
                }
                None => samples.push(json!([x, y, z, null, null, []])),
            }
        }
        body_out.push(json!({ "body": b.to_json(), "samples": samples }));
    }

    let chains: Vec<Value> = sites
        .iter()
        .map(|s| {
            let chain: Vec<u64> = log
                .cause_chain(s.forming_event, 3)
                .iter()
                .map(|e| e.id.0)
                .collect();
            json!({ "site": s.id, "event": s.forming_event.0, "chain": chain })
        })
        .collect();
    let forming: Vec<Value> = sites
        .iter()
        .filter_map(|s| log.get(s.forming_event).map(|e| e.to_json()))
        .collect();

    let v = json!({
        "circumference": c,
        "sites": sites.iter().map(|s| s.to_json()).collect::<Vec<_>>(),
        "index": { "cols": index.cols, "rows": index.rows, "cell_blocks": index.cell_blocks, "samples": idx },
        "bodies": body_out,
        "events": { "count": log.len(), "forming": forming, "chains": chains },
    });
    let bytes = serde_json::to_vec(&v).expect("JSON");
    write_atomic(out, &bytes)?;
    Ok((sites.len(), bodies.len(), inside_total))
}

/// `deeptime dump-fields`: the named fields of a checkpoint's world (every field when `names` is
/// empty) as raw little-endian f32 files with an `index.json` of their grids.
pub fn fields(ckpt: &Path, out: &Path, names: &[String]) -> Result<usize, String> {
    let ck = deeptime_core::checkpoint::Checkpoint::read(ckpt).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut index = serde_json::Map::new();
    for (name, f) in &ck.world.fields {
        if !names.is_empty() && !names.iter().any(|n| n == name) {
            continue;
        }
        let file = format!("{}.f32", name.replace('/', "__"));
        let bytes: Vec<u8> = f.data.iter().flat_map(|x| x.to_le_bytes()).collect();
        write_atomic(&out.join(&file), &bytes).map_err(|e| e.to_string())?;
        index.insert(
            name.clone(),
            json!({ "file": file, "w": f.grid.w, "h": f.grid.h, "r": f.grid.r, "grid": f.grid.name }),
        );
    }
    let n = index.len();
    let text = serde_json::to_vec_pretty(&Value::Object(index)).expect("JSON");
    write_atomic(&out.join("index.json"), &text).map_err(|e| e.to_string())?;
    Ok(n)
}
