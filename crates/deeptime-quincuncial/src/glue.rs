// SPDX-License-Identifier: LGPL-3.0-or-later
//! The gluing table: each edge of the world's domain, how it is joined, and the Immersive Portals
//! portal that joins it (PROTOTYPE).
//!
//! An Immersive Portals portal is a rectangle (centre, axisW, axisH, width, height) that shows and
//! teleports to the same rectangle placed at its destination, optionally turned by a quaternion.
//! One-way portals facing into the world (normal = axisW × axisH) make a closed world: a player
//! sees across every seam and is moved when crossing it.

use serde_json::{Value, json};

/// How an edge is joined.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SeamKind {
    /// (x, z) ↦ (2cx − x, 2cz − z): a 180° turn about the vertical through (cx, cz), the edge's
    /// midpoint. Velocities and facing turn by 180° too.
    HalfTurn { cx: f64, cz: f64 },
    /// (x, z) ↦ (x + dx, z + dz): a plain translation.
    Slide { dx: f64, dz: f64 },
}

/// One edge of the domain.
#[derive(Clone, Debug)]
pub struct Seam {
    pub name: &'static str,
    /// The edge lies on the plane `axis` = `at` (`'x'` or `'z'`).
    pub axis: char,
    pub at: f64,
    /// Unit normal pointing into the world.
    pub inward: (f64, f64),
    /// Length of the edge, blocks; the edge is centred on the axis line through the origin.
    pub length: f64,
    pub kind: SeamKind,
}

impl Seam {
    pub fn new(name: &'static str, axis: char, at: f64, inward: (f64, f64), length: f64, kind: SeamKind) -> Seam {
        Seam { name, axis, at, inward, length, kind }
    }

    /// The point of the edge at parameter t ∈ [−length/2, length/2].
    pub fn point(&self, t: f64) -> (f64, f64) {
        if self.axis == 'x' { (self.at, t) } else { (t, self.at) }
    }

    /// The gluing transform applied to a block of the plane.
    pub fn apply(&self, x: f64, z: f64) -> (f64, f64) {
        match self.kind {
            SeamKind::HalfTurn { cx, cz } => (2.0 * cx - x, 2.0 * cz - z),
            SeamKind::Slide { dx, dz } => (x + dx, z + dz),
        }
    }

    /// The centre of the edge.
    pub fn centre(&self) -> (f64, f64) {
        self.point(0.0)
    }

    /// axisW (x, z): axisW × (0, 1, 0) = the inward normal.
    pub fn axis_w(&self) -> (f64, f64) {
        // (wx, 0, wz) × (0, 1, 0) = (−wz, 0, wx) = inward  ⇒  wx = inward.z, wz = −inward.x.
        (self.inward.1, -self.inward.0)
    }

    /// The portal for this edge: a one-way global portal on the edge, `height` blocks tall centred
    /// on `y`, facing into the world. A half-turn portal's destination is its own centre turned
    /// 180° about the vertical; a slide portal's is its centre moved by the slide.
    pub fn portal(&self, y: f64, height: f64) -> Value {
        let (cx, cz) = self.centre();
        let (dx, dz) = self.apply(cx, cz);
        let (wx, wz) = self.axis_w();
        let turn = matches!(self.kind, SeamKind::HalfTurn { .. });
        json!({
            "name": self.name,
            "kind": if turn { "half_turn" } else { "slide" },
            "origin": [cx, y, cz],
            "axis_w": [wx, 0.0, wz],
            "axis_h": [0.0, 1.0, 0.0],
            "width": self.length,
            "height": height,
            "destination": [dx, y, dz],
            // Immersive Portals' rotation NBT (rotationA..D); (0, 0, 1, 0) is the half-turn about +y
            // that the 2026-10-02 prototype checked in game.
            "rotation": if turn { json!([0.0, 0.0, 1.0, 0.0]) } else { Value::Null },
            "transform": match self.kind {
                SeamKind::HalfTurn { cx, cz } => format!("(x, y, z) -> ({}, y, {}) ; yaw + 180°; (vx, vz) -> (-vx, -vz)",
                    lin(2.0 * cx, "x"), lin(2.0 * cz, "z")),
                SeamKind::Slide { dx, dz } => format!("(x, y, z) -> (x {}, y, z {})", signed(dx), signed(dz)),
            },
        })
    }

    /// The `/summon` NBT of this edge's portal, ready for RCON (coordinates written as decimals:
    /// integer summon coordinates are moved to the block centre).
    pub fn summon_command(&self, y: f64, height: f64) -> String {
        let p = self.portal(y, height);
        let f = |v: &Value, i: usize| v[i].as_f64().unwrap();
        let (o, d, w) = (&p["origin"], &p["destination"], &p["axis_w"]);
        let mut nbt = format!(
            "{{CustomName:'\"{}\"',width:{:.1}d,height:{:.1}d,axisWX:{:.1}d,axisWY:0d,axisWZ:{:.1}d,\
             axisHX:0d,axisHY:1d,axisHZ:0d,dimensionTo:\"minecraft:overworld\",\
             destinationX:{:.1}d,destinationY:{:.1}d,destinationZ:{:.1}d",
            self.name, self.length, height, f(w, 0), f(w, 2), f(d, 0), f(d, 1), f(d, 2)
        );
        if matches!(self.kind, SeamKind::HalfTurn { .. }) {
            nbt.push_str(",rotationA:0f,rotationB:0f,rotationC:1f,rotationD:0f");
        }
        nbt.push('}');
        format!("summon immersive_portals:portal {:.1} {:.1} {:.1} {nbt}", f(o, 0), f(o, 1), f(o, 2))
    }
}

fn lin(c: f64, v: &str) -> String {
    if c == 0.0 { format!("-{v}") } else { format!("{c} - {v}") }
}

fn signed(d: f64) -> String {
    if d >= 0.0 { format!("+ {d}") } else { format!("- {}", -d) }
}

#[cfg(test)]
mod tests {
    use crate::world::QWorld;

    #[test]
    fn portals_face_into_the_world_and_their_destination_is_the_glued_point() {
        for w in [QWorld::peirce(8192.0, 0.0), QWorld::guyou(8192.0, 0.0)] {
            let (x0, x1, z0, z1) = w.extent();
            let (mx, mz) = (0.5 * (x0 + x1), 0.5 * (z0 + z1));
            for s in w.seams() {
                let (cx, cz) = s.centre();
                // The inward normal points at the domain's middle.
                assert!(s.inward.0 * (mx - cx) + s.inward.1 * (mz - cz) > 0.0, "{}", s.name);
                let (wx, wz) = s.axis_w();
                assert_eq!((-wz, wx), s.inward);
                // The edge's ends are glued to the edge's (other) ends: the transform maps the edge
                // onto itself (half-turn) or onto the opposite edge (slide).
                let (ex, ez) = s.apply(s.point(0.5 * s.length).0, s.point(0.5 * s.length).1);
                let on_some_edge = w.seams().iter().any(|o| {
                    let v = if o.axis == 'x' { ex } else { ez };
                    (v - o.at).abs() < 1e-9
                });
                assert!(on_some_edge, "{}", s.name);
            }
        }
    }
}
