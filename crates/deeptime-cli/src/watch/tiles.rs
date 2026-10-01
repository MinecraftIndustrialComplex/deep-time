// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench's "Final world" tiles: the server side of the deep-zoom map of a finished run's
//! bake ([`super::worldmap`] draws them). Routes under `/api/runs/<id>/`, answered by [`route`]:
//!
//! - `world.json`: whether the run has a world to show (`available`), and if so the pyramid
//!   (`tile_px`, `min_zoom`, `max_zoom`, `circumference`), the layers, the world's scale, what the
//!   finest zoom is made of, the tile cache and the render times so far;
//! - `tiles/<layer>/<z>/<x>/<y>.png`: one 256-pixel tile (`sat`, `elev`, `water`, or `overlay`, a
//!   transparent rivers-and-lakes layer), XYZ-style: `x` east from the seam, `y` south from the
//!   north edge, zoom 0 two tiles wide ([`super::worldmap`] has the geometry);
//! - `world/sample?x=&z=`: what the bake says at a block (terrain height, water, climate).
//!
//! **View only.** Nothing here writes into a run's world data or touches a simulation; a tile is
//! drawn when it is asked for and kept in a cache under the run's own folder.
//!
//! **The cache.** `<run>/tiles/v<RENDER_VERSION>/<layer>/<z>/<x>/<y>.png`, bounded per run (a
//! least-recently-asked eviction when the folder passes the cap, 384 MiB, `DEEPTIME_TILE_CACHE_MIB`
//! sets another). A folder of another render version is deleted when the cache is first opened. The
//! folder goes with the run when the workbench prunes it. At most two runs' maps (the mips and the
//! smoothed fields, about 60 MiB each at the 16k size) are held in memory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::worldmap::{
    BakeMap, Layer, RENDER_VERSION, TILE, TileGeom, blocks_per_px, pyramid_tiles,
};

/// A reply for the HTTP layer to wrap.
pub struct Answer {
    pub status: u16,
    pub content_type: &'static str,
    pub cache: &'static str,
    pub body: Vec<u8>,
}

/// Tiles never change for a finished run, and the URL names the run.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const DEFAULT_CACHE_MIB: u64 = 384;
const MAX_OPEN_MAPS: usize = 2;
/// A map that failed to open is not tried again for this long.
const RETRY: Duration = Duration::from_secs(30);

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn json_answer(status: u16, v: &Value) -> Answer {
    Answer {
        status,
        content_type: "application/json",
        cache: "no-store",
        body: v.to_string().into_bytes(),
    }
}

fn error(status: u16, message: impl Into<String>) -> Answer {
    json_answer(status, &json!({ "error": message.into() }))
}

// ---------------------------------------------------------------------------------------------
// The cache
// ---------------------------------------------------------------------------------------------

/// A run's tile folder with a size bound.
pub struct TileCache {
    /// `<run>/tiles/v<N>`.
    root: PathBuf,
    cap: u64,
    state: Mutex<CacheState>,
}

#[derive(Default)]
struct CacheState {
    scanned: bool,
    /// `layer/z/x/y.png` → (bytes, last asked).
    files: HashMap<String, (u64, u64)>,
    bytes: u64,
    clock: u64,
}

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

impl TileCache {
    pub fn new(run_dir: &Path, cap: u64) -> TileCache {
        TileCache {
            root: run_dir.join("tiles").join(format!("v{RENDER_VERSION}")),
            cap,
            state: Mutex::new(CacheState::default()),
        }
    }

    /// Reads the folder once: what is on disk counts against the cap, the oldest files (by
    /// modification time) are the first to go, and folders of other render versions are removed.
    fn scan(&self, s: &mut CacheState) {
        if s.scanned {
            return;
        }
        s.scanned = true;
        if let Some(parent) = self.root.parent()
            && let Ok(rd) = std::fs::read_dir(parent)
        {
            for e in rd.flatten() {
                if e.path() != self.root {
                    let _ = std::fs::remove_dir_all(e.path());
                }
            }
        }
        let mut found: Vec<(std::time::SystemTime, String, u64)> = Vec::new();
        let walk = |dir: &Path, rel: &str, out: &mut Vec<(std::time::SystemTime, String, u64)>| {
            fn go(
                dir: &Path,
                rel: &str,
                out: &mut Vec<(std::time::SystemTime, String, u64)>,
                depth: u32,
            ) {
                let Ok(rd) = std::fs::read_dir(dir) else {
                    return;
                };
                for e in rd.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    let path = e.path();
                    let rel_name = if rel.is_empty() {
                        name
                    } else {
                        format!("{rel}/{name}")
                    };
                    if path.is_dir() && depth < 4 {
                        go(&path, &rel_name, out, depth + 1);
                    } else if let Ok(md) = e.metadata()
                        && md.is_file()
                        && rel_name.ends_with(".png")
                    {
                        out.push((
                            md.modified().unwrap_or(std::time::UNIX_EPOCH),
                            rel_name,
                            md.len(),
                        ));
                    } else if rel_name.ends_with(".tmp") {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
            go(dir, rel, out, 0);
        };
        walk(&self.root, "", &mut found);
        found.sort();
        for (_, key, size) in found {
            s.clock += 1;
            s.bytes += size;
            s.files.insert(key, (size, s.clock));
        }
    }

    /// The tile `key` (`layer/z/x/y.png`) if it is cached.
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        {
            let mut s = lock(&self.state);
            self.scan(&mut s);
            s.clock += 1;
            let clock = s.clock;
            s.files.get_mut(key)?.1 = clock;
        }
        match std::fs::read(self.root.join(key)) {
            Ok(b) => Some(b),
            Err(_) => {
                // Gone behind our back: forget it.
                let mut s = lock(&self.state);
                if let Some((size, _)) = s.files.remove(key) {
                    s.bytes -= size;
                }
                None
            }
        }
    }

    /// Stores a tile (written aside and renamed in) and evicts the least recently asked files
    /// while the folder is over the cap.
    pub fn put(&self, key: &str, data: &[u8]) {
        let path = self.root.join(key);
        if let Some(dir) = path.parent()
            && std::fs::create_dir_all(dir).is_err()
        {
            return;
        }
        let tmp = path.with_extension(format!(
            "png.{}.tmp",
            TMP_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        if std::fs::write(&tmp, data).is_err() || std::fs::rename(&tmp, &path).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return;
        }
        let mut s = lock(&self.state);
        self.scan(&mut s);
        s.clock += 1;
        let clock = s.clock;
        if let Some((old, _)) = s.files.insert(key.to_owned(), (data.len() as u64, clock)) {
            s.bytes -= old;
        }
        s.bytes += data.len() as u64;
        if s.bytes > self.cap {
            // Down to nine tenths, so the next tile does not evict again.
            let target = self.cap / 10 * 9;
            let mut order: Vec<(u64, String)> = s
                .files
                .iter()
                .filter(|(k, _)| k.as_str() != key)
                .map(|(k, v)| (v.1, k.clone()))
                .collect();
            order.sort_unstable();
            for (_, k) in order {
                if s.bytes <= target {
                    break;
                }
                if let Some((size, _)) = s.files.remove(&k) {
                    s.bytes -= size;
                    let _ = std::fs::remove_file(self.root.join(&k));
                }
            }
        }
    }

    /// (bytes, files, cap).
    pub fn stats(&self) -> (u64, usize, u64) {
        let mut s = lock(&self.state);
        self.scan(&mut s);
        (s.bytes, s.files.len(), self.cap)
    }
}

// ---------------------------------------------------------------------------------------------
// The service
// ---------------------------------------------------------------------------------------------

enum Slot {
    Empty,
    Open(Arc<BakeMap>),
    Failed(String, Instant),
}

/// One run's maps and tile cache.
struct RunTiles {
    world: PathBuf,
    cache: TileCache,
    map: Mutex<Slot>,
    used: AtomicU64,
}

struct Service {
    cap: u64,
    runs: Mutex<HashMap<PathBuf, Arc<RunTiles>>>,
    clock: AtomicU64,
}

fn service() -> &'static Service {
    static S: OnceLock<Service> = OnceLock::new();
    S.get_or_init(|| {
        let mib = std::env::var("DEEPTIME_TILE_CACHE_MIB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&m| m > 0)
            .unwrap_or(DEFAULT_CACHE_MIB);
        Service {
            cap: mib << 20,
            runs: Mutex::new(HashMap::new()),
            clock: AtomicU64::new(0),
        }
    })
}

impl Service {
    fn run(&self, run_dir: &Path) -> Arc<RunTiles> {
        let mut runs = lock(&self.runs);
        if runs.len() > 32 {
            runs.retain(|_, r| r.world.exists());
        }
        Arc::clone(runs.entry(run_dir.to_owned()).or_insert_with(|| {
            Arc::new(RunTiles {
                world: run_dir.join("world"),
                cache: TileCache::new(run_dir, self.cap),
                map: Mutex::new(Slot::Empty),
                used: AtomicU64::new(0),
            })
        }))
    }

    /// The run's opened map, opening it (a second or so at the 16k size) the first time.
    fn map(&self, rt: &Arc<RunTiles>) -> Result<Arc<BakeMap>, (u16, String)> {
        if !rt.world.join("manifest.json").is_file() {
            return Err((
                409,
                "this run has no baked world yet (it is still running, or it did not finish)"
                    .into(),
            ));
        }
        rt.used.store(
            self.clock.fetch_add(1, Ordering::Relaxed),
            Ordering::Relaxed,
        );
        let opened = {
            let mut slot = lock(&rt.map);
            match &*slot {
                Slot::Open(m) => return Ok(Arc::clone(m)),
                Slot::Failed(msg, at) if at.elapsed() < RETRY => return Err((409, msg.clone())),
                _ => {}
            }
            match BakeMap::open(&rt.world) {
                Ok(m) => {
                    let m = Arc::new(m);
                    *slot = Slot::Open(Arc::clone(&m));
                    m
                }
                Err(e) => {
                    *slot = Slot::Failed(e.clone(), Instant::now());
                    return Err((409, e));
                }
            }
        };
        // Keep at most MAX_OPEN_MAPS in memory: the least recently used others are let go.
        let others: Vec<Arc<RunTiles>> = lock(&self.runs)
            .values()
            .filter(|r| !Arc::ptr_eq(r, rt))
            .cloned()
            .collect();
        let mut open: Vec<(u64, Arc<RunTiles>)> = others
            .into_iter()
            .filter(|r| matches!(r.map.try_lock().as_deref(), Ok(Slot::Open(_))))
            .map(|r| (r.used.load(Ordering::Relaxed), r))
            .collect();
        open.sort_by_key(|(u, _)| *u);
        while open.len() + 1 > MAX_OPEN_MAPS {
            let (_, r) = open.remove(0);
            if let Ok(mut g) = r.map.try_lock() {
                *g = Slot::Empty;
            }
        }
        Ok(opened)
    }
}

// ---------------------------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------------------------

/// Answers `tail` (what follows `/api/runs/<id>/`) of the run in `run_dir` if it is one of the
/// map's routes; `None` for any other.
pub fn route(run_dir: &Path, tail: &str, query: &str) -> Option<Answer> {
    match tail {
        "world.json" => Some(world_json(run_dir)),
        "world/sample" => Some(sample(run_dir, query)),
        _ => tail.strip_prefix("tiles/").map(|rest| tile(run_dir, rest)),
    }
}

fn world_json(run_dir: &Path) -> Answer {
    let svc = service();
    let rt = svc.run(run_dir);
    let map = match svc.map(&rt) {
        Ok(m) => m,
        Err((_, reason)) => {
            return json_answer(200, &json!({ "available": false, "reason": reason }));
        }
    };
    let m = &map.meta;
    let (bytes, files, cap) = rt.cache.stats();
    let per_layer = pyramid_tiles(m.max_zoom);
    let mut layers = vec![
        json!({"id": "sat", "label": "Satellite"}),
        json!({"id": "elev", "label": "Elevation"}),
        json!({"id": "water", "label": "Water"}),
    ];
    if m.rivers > 0 || m.lakes {
        layers.push(json!({"id": "overlay", "label": "Rivers and lakes", "overlay": true}));
    }
    let detail = m.detail.as_ref().map(|d| {
        json!({
            "wavelength_blocks": d.wavelength_blocks,
            "octaves": d.octaves,
            "applies_to_blocks_per_px_at_most": 4,
        })
    });
    json_answer(
        200,
        &json!({
            "available": true,
            "tile_px": TILE,
            "min_zoom": 0,
            "max_zoom": m.max_zoom,
            "circumference": m.c,
            "version": RENDER_VERSION,
            "layers": layers,
            "sea_level_y": m.sea_level_y,
            "min_y": m.min_y,
            "max_y": m.max_y,
            "m_per_block_h": m.m_per_block_h,
            "m_per_block_v": m.m_per_block_v,
            "blocks_per_px": (0..=m.max_zoom).map(|z| blocks_per_px(m.c, z)).collect::<Vec<_>>(),
            "finest": {
                "blocks_per_px": blocks_per_px(m.c, m.max_zoom),
                "terrain_cell_blocks": m.l3_r,
                "detail": detail,
                "statement": format!(
                    "The terrain is evaluated at every block: the {0}-block L3 grid, bilinear, plus \
                     the game's sub-cell detail noise. Colour comes from the climate and rock \
                     grids (16 and 8 blocks a cell), lapsed to each block's height.",
                    m.l3_r
                ),
            },
            "has": {
                "climate": m.climate,
                "monthly_climate": m.monthly,
                "detail": m.detail.is_some(),
                "river_chains": m.rivers,
                "lakes": m.lakes,
            },
            "pyramid": { "tiles_per_layer": per_layer, "zooms": m.max_zoom + 1 },
            "cache": { "bytes": bytes, "tiles": files, "cap_bytes": cap },
            "timing": map.timing.json(),
            "source": m.source,
            "created_utc": m.created_utc,
            "preview_max_blocks": 256,
        }),
    )
}

fn query_f64(query: &str, key: &str) -> Option<f64> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| v.parse::<f64>().ok())
        .filter(|v| v.is_finite())
}

fn sample(run_dir: &Path, query: &str) -> Answer {
    let (Some(x), Some(z)) = (query_f64(query, "x"), query_f64(query, "z")) else {
        return error(400, "give x and z (blocks)");
    };
    let svc = service();
    let rt = svc.run(run_dir);
    let map = match svc.map(&rt) {
        Ok(m) => m,
        Err((st, e)) => return error(st, e),
    };
    match map.sample(x.floor() as i64, z.floor() as i64) {
        Ok(v) => json_answer(200, &v),
        Err(e) => error(500, e),
    }
}

/// `<layer>/<z>/<x>/<y>.png`.
fn parse_tile(rest: &str) -> Option<(Layer, u32, u32, u32)> {
    let mut it = rest.split('/');
    let layer = Layer::parse(it.next()?)?;
    let z = it.next()?.parse().ok()?;
    let x = it.next()?.parse().ok()?;
    let y = it.next()?.strip_suffix(".png")?.parse().ok()?;
    it.next().is_none().then_some((layer, z, x, y))
}

fn tile(run_dir: &Path, rest: &str) -> Answer {
    let Some((layer, z, x, y)) = parse_tile(rest) else {
        return error(
            404,
            "tiles are <layer>/<z>/<x>/<y>.png, the layer sat, elev, water or overlay",
        );
    };
    let svc = service();
    let rt = svc.run(run_dir);
    let map = match svc.map(&rt) {
        Ok(m) => m,
        Err((st, e)) => return error(st, e),
    };
    let Some(geom) = TileGeom::new(map.meta.c, z, x, y) else {
        return error(
            404,
            format!(
                "no tile {z}/{x}/{y}: zoom 0 to {}, x below 2^(z+1), y below 2^z",
                map.meta.max_zoom
            ),
        );
    };
    let key = format!("{}/{z}/{x}/{y}.png", layer.id());
    let png = match rt.cache.get(&key) {
        Some(bytes) => bytes,
        None => {
            let rgba = match map.render(layer, &geom) {
                Ok(r) => r,
                Err(e) => return error(500, format!("tile {key}: {e}")),
            };
            let encoded = if layer.opaque() {
                let rgb: Vec<u8> = rgba
                    .chunks_exact(4)
                    .flat_map(|p| [p[0], p[1], p[2]])
                    .collect();
                deeptime_viewer::encode_png_rgb(TILE as u32, TILE as u32, &rgb)
            } else {
                deeptime_viewer::encode_png(TILE as u32, TILE as u32, &rgba)
            };
            match encoded {
                Ok(b) => {
                    rt.cache.put(&key, &b);
                    b
                }
                Err(e) => return error(500, format!("tile {key}: {e}")),
            }
        }
    };
    Answer {
        status: 200,
        content_type: "image/png",
        cache: IMMUTABLE,
        body: png,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("dt-tiles-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn tile_paths_parse_and_nothing_else_does() {
        assert_eq!(
            parse_tile("sat/3/5/2.png"),
            Some((Layer::Satellite, 3, 5, 2))
        );
        assert_eq!(
            parse_tile("overlay/0/1/0.png"),
            Some((Layer::Overlay, 0, 1, 0))
        );
        for bad in [
            "sat/3/5/2",
            "sat/3/5/2.jpg",
            "rock/3/5/2.png",
            "sat/-1/0/0.png",
            "sat/3/5.png",
            "sat/3/5/2/9.png",
            "sat/3/../2.png",
            "sat/a/5/2.png",
        ] {
            assert_eq!(parse_tile(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_cache_stays_under_its_cap_and_drops_the_least_recently_asked() {
        let run = tmp("cap");
        let cache = TileCache::new(&run, 1000);
        let blob = vec![7u8; 100];
        for k in 0..14 {
            cache.put(&format!("sat/0/{k}/0.png"), &blob);
            // Ask for tile 0 all along, so it is never the oldest.
            assert!(cache.get("sat/0/0/0.png").is_some());
        }
        let (bytes, files, cap) = cache.stats();
        assert_eq!(cap, 1000);
        assert!(bytes <= 1000, "{bytes}");
        assert!((8..=10).contains(&files), "{files}");
        assert!(cache.get("sat/0/0/0.png").is_some(), "the favourite stays");
        assert!(cache.get("sat/0/1/0.png").is_none(), "the oldest went");
        // The counts match the disk.
        let on_disk: u64 = walk_bytes(&run.join("tiles"));
        assert_eq!(on_disk, bytes);
        // A new cache over the same folder counts what is there, and drops other versions.
        std::fs::create_dir_all(run.join("tiles/v0/sat")).unwrap();
        std::fs::write(run.join("tiles/v0/sat/old.png"), b"x").unwrap();
        let again = TileCache::new(&run, 1000);
        assert_eq!(again.stats().0, bytes);
        assert!(!run.join("tiles/v0").exists());
        std::fs::remove_dir_all(&run).unwrap();
    }

    fn walk_bytes(dir: &Path) -> u64 {
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let md = e.metadata().unwrap();
            n += if md.is_dir() {
                walk_bytes(&e.path())
            } else {
                md.len()
            };
        }
        n
    }

    #[test]
    fn a_run_without_a_bake_says_so() {
        let run = tmp("nobake");
        let a = route(&run, "world.json", "").unwrap();
        assert_eq!(a.status, 200);
        let v: Value = serde_json::from_slice(&a.body).unwrap();
        assert_eq!(v["available"], false);
        assert!(v["reason"].as_str().unwrap().contains("no baked world"));
        let t = route(&run, "tiles/sat/0/0/0.png", "").unwrap();
        assert_eq!(t.status, 409);
        assert!(route(&run, "frames/1/0.png", "").is_none());
        std::fs::remove_dir_all(&run).unwrap();
    }
}
