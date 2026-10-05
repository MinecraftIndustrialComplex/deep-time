// SPDX-License-Identifier: LGPL-3.0-or-later
//! The rock-at-depth and section routes on a real (tiny) baked planet: what they answer is what
//! the column evaluator ([`ColumnWorld::sample`], the one `inspect-column` and the game use) says.

use super::*;
use crate::run::{Common, RunOptions};
use deeptime_format::ColumnWorld;

fn common() -> Common {
    Common {
        threads: 2,
        mem_cap: 0,
        verbose: false,
        set: Vec::new(),
        history_mem: None,
        climate_json: None,
    }
}

/// A runs folder holding the run `baked` (a tiny planet run, as `deeptime run` makes it) and the
/// run `broken` (failed, with no world data), as the workbench keeps them.
fn workbench(name: &str) -> (PathBuf, Arc<Workbench>) {
    let dir = std::env::temp_dir().join(format!("deeptime-rock-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let runs = dir.join("runs");
    let mut c = common();
    c.set = vec![
        "resolution.l0_samples=2000".into(),
        "resolution.l1_r=32".into(),
        "resolution.l2_r=16".into(),
        "resolution.l3_r=8".into(),
        "keyframes.size=[64, 32]".into(),
        "amplify.erosion_iterations=4".into(),
        "amplify.thermal_iterations=3".into(),
        "amplify.deposition_iterations=2".into(),
    ];
    crate::run::run(&RunOptions {
        preset: concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../deeptime-core/tests/presets/planet_world_4k.json"
        )
        .into(),
        start: None,
        length_myr: Some(40.0),
        content: None,
        out: runs.join("baked/world"),
        viewer: None,
        checkpoints: false,
        common: c,
    })
    .unwrap();
    let run_json = |id: &str, state: &str| {
        json!({ "id": id, "created_utc": "2026-10-01T00:00:00Z", "label": id,
                "preset_json": "{}", "share_code": "", "mem_cap": 0,
                "progress": {}, "status": { "state": state } })
        .to_string()
    };
    std::fs::write(runs.join("baked/run.json"), run_json("baked", "done")).unwrap();
    std::fs::create_dir_all(runs.join("broken")).unwrap();
    std::fs::write(runs.join("broken/run.json"), run_json("broken", "failed")).unwrap();
    let wb = Workbench::open(&runs, 8, common(), None).unwrap();
    (dir, wb)
}

/// Whether the baked run keeps a file starting with `prefix` under `rock/<stamp>/`.
fn kept(dir: &Path, prefix: &str) -> bool {
    std::fs::read_dir(dir.join("runs/baked/rock"))
        .unwrap()
        .flatten()
        .filter(|stamp| stamp.path().is_dir())
        .any(|stamp| {
            std::fs::read_dir(stamp.path())
                .unwrap()
                .flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with(prefix))
        })
}

fn json_of(r: &Reply) -> Value {
    serde_json::from_slice(&r.body).unwrap_or(Value::Null)
}

#[test]
fn rock_at_depth_and_sections_agree_with_the_column_evaluator() {
    let (dir, wb) = workbench("agree");
    let world = ColumnWorld::open(&dir.join("runs/baked/world")).unwrap();

    // What the run offers.
    let info = json_of(&route(&wb, "/api/runs/baked/rock", "").unwrap());
    assert_eq!(info["available"], true, "{info}");
    assert_eq!(
        info["circumference"], 3072,
        "the Petroff–Guyou world's x period 2b"
    );
    assert_eq!(info["depths"][0], 0);
    assert_eq!(info["depths"].as_array().unwrap().last().unwrap(), "floor");
    let classes = info["classes"].as_array().unwrap();
    assert_eq!(classes.len(), world.classes.len());
    assert!(
        classes
            .iter()
            .all(|c| c["colour"].as_str().unwrap().starts_with('#'))
    );
    // A class the ramp knows is found by name: `k` is its index in the ramp's order.
    let granite = classes.iter().find(|c| c["name"] == "granite").unwrap();
    assert_eq!(granite["k"], 0);

    // No bake to read: said plainly, with the reason, never an error page.
    let broken = json_of(&route(&wb, "/api/runs/broken/rock", "").unwrap());
    assert_eq!(broken["available"], false);
    assert!(
        broken["why"].as_str().unwrap().contains("no readable bake"),
        "{broken}"
    );
    let r = route(&wb, "/api/runs/broken/rock/slice.png", "depth=50").unwrap();
    assert_eq!(r.status, 409, "{}", String::from_utf8_lossy(&r.body));
    assert_eq!(json_of(&r)["available"], false);
    assert_eq!(route(&wb, "/api/runs/none/rock", "").unwrap().status, 404);
    assert!(route(&wb, "/api/runs/baked/rocks", "").is_none());
    assert!(route(&wb, "/api/runs/baked/state.json", "").is_none());

    // A slice is a PNG of the asked size, kept, and the same bytes the second time.
    let q = "depth=60&w=128";
    let slice = route(&wb, "/api/runs/baked/rock/slice.png", q).unwrap();
    assert_eq!(slice.status, 200);
    assert_eq!(slice.content_type, "image/png");
    assert_eq!(&slice.body[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(
        u32::from_be_bytes(slice.body[16..20].try_into().unwrap()),
        128
    );
    assert_eq!(
        u32::from_be_bytes(slice.body[20..24].try_into().unwrap()),
        64,
        "the world's aspect, 2 : 1 (the Petroff–Guyou world)"
    );
    assert!(kept(&dir, "slice-d60-w128.png"), "the slice is kept");
    let again = route(&wb, "/api/runs/baked/rock/slice.png", q).unwrap();
    assert_eq!(again.body, slice.body);
    for bad in ["depth=deep", "depth=-3"] {
        let r = route(&wb, "/api/runs/baked/rock/slice.png", bad).unwrap();
        assert_eq!(r.status, 400, "{bad}");
    }

    // The rock a slice holds is the rock the evaluator says is that deep, class for class, at the
    // surface, at depth and at the floor.
    let rw = world::RockWorld::open(&dir.join("runs/baked/world")).unwrap();
    let (c, min_y) = (world.c, world.min_y);
    let mut compared = 0;
    for depth in [
        Depth::Below(0),
        Depth::Below(60),
        Depth::Below(200),
        Depth::Floor,
    ] {
        let s = rw.slice(depth, 128, 64, 2);
        for py in (0..64).step_by(3) {
            for px in (0..128).step_by(5) {
                let (x, z) =
                    slice::pixel_block(&world.manifest.world_projection(), c, 128, 64, px, py);
                let col = world.sample(x, z);
                let y = match depth {
                    Depth::Below(d) => (col.top_y - d as i32).max(min_y),
                    Depth::Floor => min_y,
                };
                let want = col
                    .rock
                    .iter()
                    .find(|(top, bottom, _)| y <= *top && y >= *bottom)
                    .and_then(|(_, _, class)| class.clone());
                let got = s.class[(py * 128 + px) as usize];
                let got = (got != world::NONE).then(|| world.classes[usize::from(got)].clone());
                assert_eq!(
                    got, want,
                    "{depth:?} pixel ({px}, {py}) = block ({x}, {z}), y {y}"
                );
                compared += 1;
            }
        }
    }
    assert!(compared > 1000);

    // A section: JSON along the line, kept, and each column is what the evaluator says.
    // (Inside the 3,072-block world's half period, so the line does not take the short way round.)
    let q = "x0=-700&z0=-400&x1=700&z1=500";
    let sec = route(&wb, "/api/runs/baked/rock/section", q).unwrap();
    assert_eq!(sec.status, 200, "{}", String::from_utf8_lossy(&sec.body));
    let v = json_of(&sec);
    assert_eq!(v["available"], true);
    let n = v["n"].as_u64().unwrap() as usize;
    let len = (1400f64.powi(2) + 900f64.powi(2)).sqrt();
    assert!((v["length_blocks"].as_f64().unwrap() - len).abs() < 1e-6);
    let km_per_block = world.manifest.scale.m_per_block_h / 1000.0;
    assert!((v["length_km"].as_f64().unwrap() - len * km_per_block).abs() < 1e-6);
    assert_eq!(n, len.ceil() as usize + 1);
    let cols = &v["columns"];
    for key in ["x", "z", "ground", "water", "runs", "bodies"] {
        assert_eq!(cols[key].as_array().unwrap().len(), n, "{key}");
    }
    assert_eq!(
        (cols["x"][0].as_i64(), cols["z"][0].as_i64()),
        (Some(-900), Some(-400))
    );
    assert_eq!(
        (cols["x"][n - 1].as_i64(), cols["z"][n - 1].as_i64()),
        (Some(700), Some(500))
    );
    let (mut checked, mut spans) = (0, 0);
    for j in (0..n).step_by(5) {
        let (x, z) = (
            cols["x"][j].as_i64().unwrap(),
            cols["z"][j].as_i64().unwrap(),
        );
        let col = world.sample(x, z);
        let hydro = world.hydro(x, z);
        let ground = hydro.ground;
        assert_eq!(
            cols["ground"][j].as_i64().unwrap(),
            i64::from(ground),
            "column {j}"
        );
        assert_eq!(
            cols["water"][j].as_i64(),
            hydro.top.filter(|_| hydro.kind.is_water()).map(i64::from),
            "column {j}"
        );
        // The runs: top y and class, top-down, each filling to the next top + 1.
        let runs = cols["runs"][j].as_array().unwrap();
        for y in (min_y..=ground.min(col.top_y)).rev() {
            let mut got = None;
            for k in (0..runs.len()).step_by(2) {
                let (top, class) = (runs[k].as_i64().unwrap(), runs[k + 1].as_i64().unwrap());
                let bottom = runs
                    .get(k + 2)
                    .map_or(i64::from(min_y), |t| t.as_i64().unwrap() + 1);
                if i64::from(y) <= top && i64::from(y) >= bottom {
                    got = (class >= 0).then(|| world.classes[class as usize].clone());
                }
            }
            let want = col
                .rock
                .iter()
                .find(|(top, bottom, _)| y <= *top && y >= *bottom)
                .and_then(|(_, _, class)| class.clone());
            assert_eq!(got, want, "column {j} = ({x}, {z}), y {y}");
            checked += 1;
        }
        // The bodies the column passes through, cut at the ground.
        let mut got: Vec<(i64, i64, i64)> = cols["bodies"][j]
            .as_array()
            .unwrap()
            .chunks(3)
            .map(|t| {
                (
                    t[0].as_i64().unwrap(),
                    t[1].as_i64().unwrap(),
                    t[2].as_i64().unwrap(),
                )
            })
            .collect();
        let mut want: Vec<(i64, i64, i64)> = col
            .bodies
            .iter()
            .filter(|b| b.2 <= ground)
            .map(|b| (i64::from(b.0), i64::from(b.1.min(ground)), i64::from(b.2)))
            .collect();
        got.sort_unstable();
        want.sort_unstable();
        assert_eq!(got, want, "bodies of column {j} = ({x}, {z})");
        spans += got.len();
    }
    assert!(checked > 500, "{checked}");
    eprintln!("sections test: {checked} blocks and {spans} body spans compared");
    assert!(
        kept(&dir, "section--900_-400_700_500-n"),
        "the section is kept"
    );
    assert_eq!(
        route(&wb, "/api/runs/baked/rock/section", q).unwrap().body,
        sec.body
    );

    // Through the bodies themselves (the tiny world has a couple of plutons): a line along the
    // east-west axis of each, the class they paint and the spans they leave are the evaluator's.
    assert!(!world.bodies.is_empty(), "the tiny world has bodies");
    let mut through = 0;
    for b in world.bodies.iter().take(4) {
        let (cx, cz) = (b.center[0].round() as i64, b.center[2].round() as i64);
        let reach = f64::from(b.axes[0].max(b.axes[2])).ceil() as i64 + 2;
        // The rock at the body's centre, as a slice reads it (the first painting body there).
        let yc = b.center[1].floor() as i32;
        let col = world.sample(cx, cz);
        if (min_y..=col.top_y).contains(&yc) {
            let mut stack = Vec::new();
            rw.stack(&mut Default::default(), cx, cz, &mut stack);
            let got = rw.class_at(cx, yc, cz, &stack);
            let want = col
                .rock
                .iter()
                .find(|(top, bottom, _)| yc <= *top && yc >= *bottom)
                .and_then(|(_, _, class)| class.clone());
            let got = (got != world::NONE).then(|| world.classes[usize::from(got)].clone());
            assert_eq!(got, want, "centre of body {}", b.id);
        }
        let q = format!("x0={}&z0={cz}&x1={}&z1={cz}", cx - reach, cx + reach);
        let v = json_of(&route(&wb, "/api/runs/baked/rock/section", &q).unwrap());
        let cols = &v["columns"];
        for j in 0..v["n"].as_u64().unwrap() as usize {
            let (x, z) = (
                cols["x"][j].as_i64().unwrap(),
                cols["z"][j].as_i64().unwrap(),
            );
            let col = world.sample(x, z);
            let ground = world.hydro(x, z).ground;
            let runs = cols["runs"][j].as_array().unwrap();
            for y in (min_y..=ground.min(col.top_y)).rev() {
                let mut got = None;
                for k in (0..runs.len()).step_by(2) {
                    let (top, class) = (runs[k].as_i64().unwrap(), runs[k + 1].as_i64().unwrap());
                    let bottom = runs
                        .get(k + 2)
                        .map_or(i64::from(min_y), |t| t.as_i64().unwrap() + 1);
                    if i64::from(y) <= top && i64::from(y) >= bottom {
                        got = (class >= 0).then(|| world.classes[class as usize].clone());
                    }
                }
                let want = col
                    .rock
                    .iter()
                    .find(|(top, bottom, _)| y <= *top && y >= *bottom)
                    .and_then(|(_, _, class)| class.clone());
                assert_eq!(got, want, "body {} column ({x}, {z}), y {y}", b.id);
            }
            let spans = cols["bodies"][j].as_array().unwrap().len() / 3;
            through += spans;
            let want = col.bodies.iter().filter(|t| t.2 <= ground).count();
            assert_eq!(spans, want, "body spans of ({x}, {z})");
        }
    }
    assert!(through > 0, "the lines cross their bodies");
    eprintln!("sections test: {through} body spans through the bodies compared");

    // Two equal points are no section; a line across the seam takes the short way.
    let r = route(&wb, "/api/runs/baked/rock/section", "x0=5&z0=5&x1=5&z1=5").unwrap();
    assert_eq!(r.status, 400);
    let r = route(
        &wb,
        "/api/runs/baked/rock/section",
        "x0=2000&z0=0&x1=-2000&z1=0&n=64",
    )
    .unwrap();
    let v = json_of(&r);
    assert_eq!(
        v["length_blocks"], 96.0,
        "C = 4096: 2000 to -2000 is 96 blocks east: {v}"
    );
    assert_eq!(v["b"]["x"], 2096);
    assert_eq!(v["n"], 64);
    let r = route(&wb, "/api/runs/baked/rock/section", "x0=a&z0=0&x1=1&z1=1").unwrap();
    assert_eq!(r.status, 400);

    wb.close();
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_page_script_is_served() {
    let dir = std::env::temp_dir().join(format!("deeptime-rock-{}-js", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let wb = Workbench::open(&dir, 8, common(), None).unwrap();
    let r = route(&wb, "/sections.js", "").unwrap();
    assert_eq!(r.status, 200);
    assert!(r.content_type.starts_with("text/javascript"));
    assert!(String::from_utf8(r.body).unwrap().contains("DTSections"));
    wb.close();
    std::fs::remove_dir_all(&dir).unwrap();
}
