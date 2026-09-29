// SPDX-License-Identifier: LGPL-3.0-or-later
//! `export-area` on a real region file (the fixture world of tests/fixture.rs: chunks 4..=9 ×
//! 0..=3 of the standalone smoke world, C = 16384): the export decodes to exactly the world's
//! blocks, and a box given past the seam lands on the same columns.

use std::path::{Path, PathBuf};

use deeptime_inspect::area::{self, AreaBox, Context, Probes};
use deeptime_inspect::colors::ColorTable;

const C: i64 = 16384;

fn fixture_world() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world")
}

fn export(bx: AreaBox) -> (area::Decoded, deeptime_inspect::world::World, AreaBox) {
    let bx = bx.normalised(C);
    let w = area::load_world(&fixture_world(), &bx, C).unwrap();
    let (probes, colors) = (Probes::default(), ColorTable::default());
    let ctx = Context {
        circumference: C,
        world_data: None,
        bodies: &[],
        sites: &[],
        events: None,
        probes: &probes,
        colors: &colors,
        meta: serde_json::json!({"test": true}),
    };
    let e = area::build(&w, &bx, &ctx).unwrap();
    let d = area::decode(&area::encode(&e).unwrap()).unwrap();
    (d, w, bx)
}

#[test]
fn real_chunks_round_trip_block_for_block() {
    // Chunks 4..=8 are full: x 64..144, z 0..64, the whole stored height.
    let (d, w, bx) = export(AreaBox::new(64, 0, 144, 64, -64, 319));
    let mut compared = 0u64;
    for zk in 0..d.depth {
        for i in 0..d.width {
            let (x, z) = (bx.world_x(i as i32, C), bx.z0 + zk as i32);
            for y in bx.y0..=bx.y1 {
                assert_eq!(
                    d.block(i, zk, y),
                    w.reg.name(w.block(x, y, z)),
                    "at {x} {y} {z}"
                );
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 80 * 64 * 384);
    let stats = &d.header["stats"];
    assert_eq!(stats["chunks_not_full"].as_array().unwrap().len(), 0);
    assert!(stats["non_air"].as_u64().unwrap() > 80 * 64 * 64, "{stats}");
    // RLE pays: far fewer runs than blocks.
    assert!(stats["runs"].as_u64().unwrap() < stats["voxels"].as_u64().unwrap() / 20);
    assert_eq!(d.header["meta"]["test"], true);
}

#[test]
fn same_columns_one_lap_away_and_proto_chunks_reported() {
    let (a, _, _) = export(AreaBox::new(64, 0, 96, 32, -64, 64));
    let (b, _, bxb) = export(AreaBox::new(64 + C as i32, 0, 96 + C as i32, 32, -64, 64));
    assert_eq!(bxb.x0, 64);
    assert_eq!(a.voxels, b.voxels);
    // Chunk column 9 is the proto-chunk ring, 10 is absent.
    let (p, _, _) = export(AreaBox::new(144, 0, 176, 16, -64, 64));
    let s = &p.header["stats"];
    assert_eq!(s["chunks_not_full"][0][0], 9, "{s}");
    assert_eq!(s["missing_chunks"][0][0], 10, "{s}");
}
