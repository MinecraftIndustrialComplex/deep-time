// SPDX-License-Identifier: LGPL-3.0-or-later
//! The alluvial fans (HYD-165 to HYD-170): Bull's (1964) laws in square miles, and a canyon that
//! leaves a ridge for flat ground on a window of the D5 planet's Lambert frame (1 block a cell):
//! the fan at the slope break, round on the planet at 0° and at 55° (and three times as wide as
//! long on the grid there), Bull's area unless hemmed in, none on a smooth profile, a larger
//! river's channel as its edge, and no cone over ground that falls away faster than the fan.

use deeptime_format::rivers::Rivers;
use deeptime_format::raster::ZEdges;
use deeptime_format::{GridDesc, Projection};
use serde_json::Value;

use super::fan::{self, Fans, Input};
use super::network::{FlowWalk, Output, kind};

const C: u32 = 16_384;
const M_H: f64 = 300.0;
const M_V: f64 = 35.0;
/// The plain's height, 1/16 block (y 80).
const PLAIN: i32 = 80 * 16;
/// The canyon floor's slope (steeper than the fan's 0.0026), its walls' height over the floor,
/// blocks, and its half-widths of floor and of ridge, blocks of planet.
const CANYON_SLOPE: f64 = 0.02;
const WALL_BLOCKS: f64 = 40.0;
const FLOOR_HALF: f64 = 3.0;
const RIDGE_HALF: f64 = 12.0;
/// How far the canyon runs north of the apex and the river south of it, blocks of planet.
const NORTH: f64 = 160.0;
const SOUTH: f64 = 200.0;
/// The drainage area at the apex, km²: Bull's fan is 1,171 km² sloping 0.0026.
const DRAINAGE: f64 = 2000.0;

fn proj() -> Projection {
    Projection::of(f64::from(C))
}

/// A window of the planet's frame about latitude `lat` (r = 1), `w` × `h` cells, the apex at its
/// column w/2 and `north` rows from its top.
struct Window {
    grid: GridDesc,
    apex: (f64, f64),
}

fn window(lat: f64, w: u32, h: u32, north: u32) -> Window {
    let z0 = proj().z_of_lat(lat).round();
    let grid = GridDesc {
        name: "L3".into(),
        w,
        h,
        r: 1,
        wraps_x: false,
        origin_block_x: -i64::from(w / 2),
        origin_block_z: z0 as i64 - i64::from(north),
        z_edges: ZEdges::Clamp,
    };
    Window {
        grid,
        apex: (0.0, z0),
    }
}

impl Window {
    fn pos(&self, cell: usize) -> (f64, f64) {
        let w = self.grid.w as usize;
        (
            (self.grid.origin_block_x + (cell % w) as i64) as f64,
            (self.grid.origin_block_z + (cell / w) as i64) as f64,
        )
    }

    fn cell(&self, x: f64, z: f64) -> u32 {
        let i = (x.round() as i64 - self.grid.origin_block_x) as u32;
        let j = (z.round() as i64 - self.grid.origin_block_z) as u32;
        j * self.grid.w + i
    }

    /// (east, south) of block (x, z) from the apex, blocks of planet.
    fn planet(&self, x: f64, z: f64) -> (f64, f64) {
        let p = proj();
        let r = f64::from(C) / std::f64::consts::TAU;
        let (lat, lat0) = (p.lat_deg(z), p.lat_deg(self.apex.1));
        (
            (x - self.apex.0) * p.cos_lat(z),
            r * (lat0 - lat).to_radians(),
        )
    }

    /// The block (x, z) `south` blocks of planet south of the apex on its meridian.
    fn at_south(&self, south: f64) -> (f64, f64) {
        let p = proj();
        let r = f64::from(C) / std::f64::consts::TAU;
        let lat = p.lat_deg(self.apex.1) - (south / r).to_degrees();
        (self.apex.0, p.z_of_lat(lat))
    }

    /// The ground: flat plain, and north of the apex a ridge with a canyon down its middle whose
    /// floor falls to the plain at the apex; `ground` overrides it.
    fn elev(&self, ground: &dyn Fn(f64, f64) -> Option<i32>) -> Vec<i16> {
        (0..self.grid.cells())
            .map(|c| {
                let (x, z) = self.pos(c);
                let (e, s) = self.planet(x, z);
                if let Some(g) = ground(e, s) {
                    return g as i16;
                }
                let mut h = f64::from(PLAIN);
                if s < 0.0 && -s <= NORTH + 20.0 && e.abs() <= RIDGE_HALF {
                    h += CANYON_SLOPE * -s * M_H / M_V * 16.0;
                    if e.abs() > FLOOR_HALF {
                        h += WALL_BLOCKS * 16.0;
                    }
                }
                h.round() as i16
            })
            .collect()
    }

    /// The river's walk down the canyon's axis from `NORTH` north of the apex to `SOUTH` south
    /// of it, a cell a block of planet (no repeats).
    fn walk(&self, elev: &[i16], south_end: f64) -> FlowWalk {
        let mut cells: Vec<u32> = Vec::new();
        let mut s = -NORTH;
        while s <= south_end {
            let (x, z) = self.at_south(s);
            let c = self.cell(x, z);
            if cells.last() != Some(&c) {
                cells.push(c);
            }
            s += 0.5;
        }
        FlowWalk {
            area_km2: cells.iter().map(|_| DRAINAGE as f32).collect(),
            fill: cells.iter().map(|&c| elev[c as usize]).collect(),
            cells,
            lead: false,
        }
    }
}

fn output(n: usize, walks: Vec<FlowWalk>) -> Output {
    Output {
        water_level: Vec::new(),
        kind: vec![kind::NONE; n],
        closed_basin: Vec::new(),
        sinks: Vec::new(),
        lakes: Vec::new(),
        rivers: Rivers::empty(C),
        mouths: Vec::new(),
        floodplain: Vec::new(),
        walks,
        stats: Value::Null,
    }
}

fn find(win: &Window, elev: &[i16], net: &Output) -> Fans {
    let taken = |_: u32| false;
    fan::find(
        &Input {
            grid: &win.grid,
            elev,
            sea_raw: 63 * 16,
            m_per_block_h: M_H,
            m_per_block_v: M_V,
            lambert: Some(proj()),
            taken: &taken,
        },
        net,
    )
}

/// The fan's extent on the planet (east-west, north-south) and on the grid (x, z), blocks.
fn extents(win: &Window, f: &fan::Fan) -> ((f64, f64), (f64, f64)) {
    let (mut e, mut s, mut x, mut z) = (
        (f64::MAX, f64::MIN),
        (f64::MAX, f64::MIN),
        (f64::MAX, f64::MIN),
        (f64::MAX, f64::MIN),
    );
    let grow = |r: &mut (f64, f64), v: f64| *r = (r.0.min(v), r.1.max(v));
    for &c in &f.cells {
        let (bx, bz) = win.pos(c as usize);
        let (pe, ps) = win.planet(bx, bz);
        grow(&mut e, pe);
        grow(&mut s, ps);
        grow(&mut x, bx);
        grow(&mut z, bz);
    }
    ((e.1 - e.0, s.1 - s.0), (x.1 - x.0, z.1 - z.0))
}

#[test]
fn bulls_laws_are_in_square_miles() {
    // 100 mi² of drainage: 1.3 × 100^0.88 = 74.8 mi² of fan, sloping 0.022 × 100^−0.32 = 0.00504.
    let a = 100.0 * 2.589_988;
    assert!((fan::fan_area_km2(a) / 2.589_988 - 74.80).abs() < 0.01);
    assert!((fan::fan_slope(a) - 0.005_040).abs() < 1e-6);
    // The free fan: a quarter circle of that area.
    let l = fan::free_length_m(a);
    let quarter = std::f64::consts::FRAC_PI_4 * l * l / 1e6;
    assert!((quarter / fan::fan_area_km2(a) - 1.0).abs() < 1e-9);
    // Bull's range ends at 296 mi²; beyond it the law is extrapolated (the record counts them).
    assert!(fan::fan_area_km2(2000.0) > 1000.0);
}

#[test]
fn a_canyon_leaving_a_ridge_builds_bulls_fan_at_the_slope_break() {
    let win = window(0.0, 260, 400, 180);
    let elev = win.elev(&|_, _| None);
    let net = output(elev.len(), vec![win.walk(&elev, SOUTH)]);
    let fans = find(&win, &elev, &net);
    assert_eq!(fans.fans.len(), 1, "{}", fan::stats(&fans, 1, 0.09, M_V));
    let f = &fans.fans[0];
    // The apex where the canyon meets the plain.
    let (e, s) = win.planet(f.x, f.z);
    assert!(e.abs() < 1.0 && s.abs() <= 1.5, "apex at ({e}, {s})");
    assert!((f.slope - fan::fan_slope(DRAINAGE)).abs() < 1e-12);
    assert!(f.slope_up >= f.slope && f.slope_down < f.slope);
    // Bull's area, to a cell, below the free fan's height (not hemmed).
    let cell_km2 = (M_H / 1000.0).powi(2);
    let area = f.cells.len() as f64 * cell_km2;
    assert!(
        (area / f.bull_km2 - 1.0).abs() < 0.01,
        "{area} km² against {}",
        f.bull_km2
    );
    assert_eq!(fans.hemmed, 0);
    assert!(f.thickness_m < f.free_thickness_m);
    assert!(f.thickness_m >= M_V);
    // A cone: every cell at the cone's height there, over the ground it covers, and the cone a
    // block of planet higher for each 1 / slope blocks nearer the apex.
    for (&c, &h) in f.cells.iter().zip(&f.height) {
        let (x, z) = win.pos(c as usize);
        let cone = fans.cone_raw(f, x, z);
        assert_eq!(i32::from(h), cone.floor() as i32);
        assert!(cone > f64::from(elev[c as usize]));
    }
    let mut raised = elev.clone();
    fans.apply(&mut raised);
    assert!(raised.iter().zip(&elev).all(|(a, b)| a >= b));
    assert!(raised.iter().zip(&elev).filter(|(a, b)| a > b).count() > 10_000);
}

#[test]
fn a_fan_is_round_on_the_planet_at_55_degrees() {
    let mut planet = Vec::new();
    for (lat, w, h, north) in [(0.0, 260, 400, 180), (55.0, 420, 240, 110)] {
        let win = window(lat, w, h, north);
        let elev = win.elev(&|_, _| None);
        let net = output(elev.len(), vec![win.walk(&elev, SOUTH)]);
        let fans = find(&win, &elev, &net);
        assert_eq!(fans.fans.len(), 1, "at {lat}°");
        let f = &fans.fans[0];
        let ((pe, ps), (gx, gz)) = extents(&win, f);
        // A disc about the apex (the ridge's slot is inside it): as long as it is wide.
        let aspect = ps / pe;
        assert!(
            (0.93..=1.07).contains(&aspect),
            "at {lat}°: {ps} × {pe} blocks of planet"
        );
        // Its radius from its area, π ρ² less the slot.
        let rho = (f.bull_km2 * 1e6 / std::f64::consts::PI).sqrt() / M_H;
        assert!((pe / (2.0 * rho) - 1.0).abs() < 0.08, "at {lat}°: {pe} for ρ {rho}");
        let cos2 = proj().cos_lat(win.apex.1).powi(2);
        let grid = gz / gx;
        assert!(
            (grid / (aspect * cos2) - 1.0).abs() < 0.1,
            "at {lat}°: {gz} × {gx} blocks on the grid, cos² φ {cos2}"
        );
        planet.push((aspect, f.cells.len()));
    }
    assert!((planet[0].0 - planet[1].0).abs() < 0.05, "{planet:?}");
    // The frame's cells are of equal area: the same count at both latitudes.
    let (a, b) = (planet[0].1 as f64, planet[1].1 as f64);
    assert!((a / b - 1.0).abs() < 0.01, "{planet:?}");
}

#[test]
fn higher_ground_beside_the_river_hems_the_fan_in() {
    let win = window(0.0, 260, 400, 180);
    // Walls of high ground 14 to 18 blocks of planet either side of the river's axis, the length
    // of the window: the cone fills the valley between them as far as the free fan's height
    // carries it, short of Bull's area.
    let elev = win.elev(&|e, _| (14.0..18.0).contains(&e.abs()).then_some(PLAIN + 640));
    let net = output(elev.len(), vec![win.walk(&elev, SOUTH)]);
    let fans = find(&win, &elev, &net);
    assert_eq!(fans.fans.len(), 1);
    assert_eq!(fans.hemmed, 1);
    let f = &fans.fans[0];
    assert_eq!(f.thickness_m, f.free_thickness_m);
    let area = f.cells.len() as f64 * (M_H / 1000.0).powi(2);
    assert!(area < 0.5 * f.bull_km2, "{area} km² of {}", f.bull_km2);
    for &c in &f.cells {
        let (x, z) = win.pos(c as usize);
        let (e, s) = win.planet(x, z);
        assert!(e.abs() < 14.0, "({e}, {s})");
    }
}

#[test]
fn a_smooth_profile_builds_no_fan() {
    let win = window(0.0, 260, 400, 180);
    // The canyon's slope carries on south of the apex: no break, so nowhere is the ground below
    // gentler than the fan.
    let elev = win.elev(&|_, s| {
        (s >= 0.0).then(|| (f64::from(PLAIN) - CANYON_SLOPE * s * M_H / M_V * 16.0).round() as i32)
    });
    let net = output(elev.len(), vec![win.walk(&elev, SOUTH)]);
    let fans = find(&win, &elev, &net);
    assert!(fans.fans.is_empty(), "{}", fan::stats(&fans, 1, 0.09, M_V));
    assert_eq!(fans.candidates, 0);
}

#[test]
fn a_larger_rivers_channel_is_the_fans_edge() {
    let win = window(0.0, 260, 400, 180);
    let elev = win.elev(&|_, _| None);
    // A larger river across the plain 40 blocks of planet south of the apex, east to west; the
    // canyon's river ends in it.
    let (_, zr) = win.at_south(40.0);
    let mut main = FlowWalk::default();
    let w = win.grid.w as i64;
    for i in (0..w).rev() {
        let c = win.cell((win.grid.origin_block_x + i) as f64, zr);
        main.cells.push(c);
        main.area_km2.push(20_000.0);
        main.fill.push(elev[c as usize]);
    }
    let trib = win.walk(&elev, 40.0);
    assert!(main.cells.contains(trib.cells.last().unwrap()));
    let net = output(elev.len(), vec![main, trib]);
    let fans = find(&win, &elev, &net);
    assert_eq!(fans.fans.len(), 1);
    let f = &fans.fans[0];
    assert_eq!(f.chain, 1);
    for &c in &f.cells {
        let (_, z) = win.pos(c as usize);
        assert!(z < zr.round(), "a fan cell at z {z}, the river at {zr}");
    }
    // Pushed north and sideways, it still reaches Bull's area.
    assert_eq!(fans.hemmed, 0);
    let area = f.cells.len() as f64 * (M_H / 1000.0).powi(2);
    assert!((area / f.bull_km2 - 1.0).abs() < 0.01);
}

#[test]
fn ground_falling_away_faster_than_the_fan_ends_it() {
    let win = window(0.0, 260, 400, 180);
    // East of 20 blocks of planet from the apex's meridian the plain falls 0.01 m/m (the fan's
    // slope is 0.0026), to 10 blocks lower: the cone would stand over all of it, deeper the
    // further east; the fan ends where the ground lies lower than its slope below the apex's.
    let drop = 0.01 * M_H / M_V * 16.0;
    let elev = win.elev(&|e, _| {
        (e > 20.0).then(|| (f64::from(PLAIN) - (drop * (e - 20.0)).min(160.0)).round() as i32)
    });
    let net = output(elev.len(), vec![win.walk(&elev, SOUTH)]);
    let fans = find(&win, &elev, &net);
    assert_eq!(fans.fans.len(), 1, "{}", fan::stats(&fans, 1, 0.09, M_V));
    let f = &fans.fans[0];
    let apex_raise = f.top_raw - f64::from(elev[f.apex as usize]);
    let (mut east, mut west) = (f64::MIN, f64::MAX);
    for (&c, &h) in f.cells.iter().zip(&f.height) {
        let (x, z) = win.pos(c as usize);
        let (e, _) = win.planet(x, z);
        (east, west) = (east.max(e), west.min(e));
        assert!(f64::from(h) - f64::from(elev[c as usize]) <= apex_raise + 1.0);
    }
    // Where the ground falls faster than the cone the fan stops short of the free fan's radius
    // (64 blocks of planet for Bull's 1,171 km²); west, on the flat plain, it reaches it.
    assert!(east < 45.0, "east to {east}");
    assert!(west < -55.0, "west to {west}");
}
