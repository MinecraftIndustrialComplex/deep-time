// SPDX-License-Identifier: LGPL-3.0-or-later
//! Events of finite extent: Deep Time's addition to the port, not part of Noddy.
//!
//! Noddy models a cube a few kilometres across, so its folds are endless trains and its planar
//! faults endless planes. On a planet a structure has to end. A [`Windowed`] event is one of the
//! ported events whose displacement is scaled by a weight that depends on the horizontal position
//! only and falls smoothly to zero at the edge of a rectangle laid along the structure:
//!
//! `p₀ = p + w(p) · (undo(p) − p)`
//!
//! The weight is a polynomial ([`bump`]), so it needs no table and no library function, and it is
//! exactly zero outside the window, so a structure can be found by its bounding box.
//!
//! Two shapes matter to the world generator (plan §8.5, P2-01):
//! - a **fold train**: an upright sine fold (vertical axial plane, horizontal axis) under a window
//!   a few wavelengths across and several long, an en-echelon bundle of doubly plunging anticlines
//!   and synclines;
//! - a **fault segment**: a translation fault whose slip is greatest at the middle of its trace,
//!   dies out towards its tips ([`Window::half_along`]) and with distance from the trace
//!   ([`Window::half_across`]): a normal fault then drops a tilted block (a half-graben with a
//!   rollover), a thrust lifts a ramp. Only the hanging wall moves, so on the foot-wall side of
//!   the trace the weight matters only for the wedge of hanging wall above the plane.
//!
//! Because the weight does not depend on height, both displace every point of a vertical column
//! by the same vector on each side of the fault plane: a column of the deformed world is a few
//! intervals, each a shifted copy of a column of the undeformed one. The generator's column rule
//! relies on that.

use crate::V3;
use crate::events::{Event, Step};
use crate::taste::Sequence;

/// The window's profile: `(1 − t²)²` for |t| < 1, 0 outside. 1 at the centre, value and slope 0
/// at the edge.
#[inline]
pub fn bump(t: f64) -> f64 {
    if t > -1.0 && t < 1.0 {
        let u = 1.0 - t * t;
        u * u
    } else {
        0.0
    }
}

/// A rectangle on the map, laid along a structure, with a smooth weight inside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    /// The centre, (x, y) = (east, north).
    pub centre: [f64; 2],
    /// Unit vector along the structure (a fold's axis, a fault's strike).
    pub along: [f64; 2],
    /// Unit vector across it. For a fault: pointing down the dip, to the hanging-wall side.
    pub across: [f64; 2],
    /// Half the window's length along the structure.
    pub half_along: f64,
    /// Half its width.
    pub half_across: f64,
}

impl Window {
    /// A window centred on `centre` whose long axis points `azimuth_deg` clockwise from north;
    /// `across` is 90° clockwise from it.
    pub fn new(centre: [f64; 2], azimuth_deg: f64, half_along: f64, half_across: f64) -> Window {
        let a = azimuth_deg.to_radians();
        let along = [a.sin(), a.cos()];
        Window {
            centre,
            along,
            across: [along[1], -along[0]],
            half_along,
            half_across,
        }
    }

    /// The weight at a point (its height does not matter).
    #[inline]
    pub fn weight(&self, p: V3) -> f64 {
        let (dx, dy) = (p[0] - self.centre[0], p[1] - self.centre[1]);
        let a = dx * self.along[0] + dy * self.along[1];
        let wa = bump(a / self.half_along);
        if wa == 0.0 {
            return 0.0;
        }
        let b = dx * self.across[0] + dy * self.across[1];
        wa * bump(b / self.half_across)
    }

    /// Half the side of the square about the centre that holds every point of non-zero weight.
    pub fn reach(&self) -> f64 {
        self.half_along.hypot(self.half_across)
    }
}

/// One of the ported events under a [`Window`].
#[derive(Clone, Debug, PartialEq)]
pub struct Windowed {
    pub inner: Box<Event>,
    pub window: Window,
}

impl Windowed {
    pub fn new(inner: Event, window: Window) -> Windowed {
        Windowed {
            inner: Box::new(inner),
            window,
        }
    }

    pub fn undo(&self, p: V3, place: usize, previous_igneous: bool, seq: &mut Sequence) -> Step {
        let w = self.window.weight(p);
        if w == 0.0 {
            return Step::Moved(p);
        }
        match self.inner.undo(p, place, previous_igneous, seq) {
            Step::Moved(q) => Step::Moved([
                p[0] + w * (q[0] - p[0]),
                p[1] + w * (q[1] - p[1]),
                p[2] + w * (q[2] - p[2]),
            ]),
            Step::Stop => Step::Stop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Fault, Fold, Movement};
    use crate::profile::Profile;

    #[test]
    fn the_bump_is_smooth_and_compact() {
        assert_eq!(bump(0.0), 1.0);
        assert_eq!(bump(1.0), 0.0);
        assert_eq!(bump(-1.0), 0.0);
        assert_eq!(bump(7.0), 0.0);
        assert!(bump(0.999) < 1e-5);
        // Monotone on each side.
        let mut last = 1.0;
        for k in 1..=100 {
            let v = bump(f64::from(k) / 100.0);
            assert!(v <= last);
            assert_eq!(v, bump(-f64::from(k) / 100.0));
            last = v;
        }
    }

    #[test]
    fn a_window_follows_its_azimuth() {
        // Long axis north-east.
        let w = Window::new([100.0, 50.0], 45.0, 400.0, 100.0);
        let s = std::f64::consts::FRAC_1_SQRT_2;
        assert_eq!(w.weight([100.0, 50.0, 9.0]), 1.0);
        assert!(w.weight([100.0 + 300.0 * s, 50.0 + 300.0 * s, 0.0]) > 0.0);
        assert_eq!(w.weight([100.0 + 400.0 * s, 50.0 + 400.0 * s, 0.0]), 0.0);
        assert_eq!(w.weight([100.0 + 150.0 * s, 50.0 - 150.0 * s, 0.0]), 0.0);
        assert!(w.weight([100.0 + 50.0 * s, 50.0 - 50.0 * s, 0.0]) > 0.0);
        // Height never matters.
        assert_eq!(
            w.weight([130.0, 60.0, -500.0]),
            w.weight([130.0, 60.0, 500.0])
        );
        assert!((w.reach() - 400.0f64.hypot(100.0)).abs() < 1e-12);
    }

    #[test]
    fn a_windowed_fold_is_the_fold_at_its_centre_and_nothing_outside() {
        let fold = Fold::new(
            [0.0; 3],
            90.0,
            90.0,
            0.0,
            200.0,
            30.0,
            0.0,
            false,
            Profile::Sine,
        );
        // Axis north-south, window 3 wavelengths wide and 1,000 long, centred on a crest.
        let window = Window::new([50.0, 0.0], 0.0, 500.0, 300.0);
        let e = Windowed::new(Event::Fold(fold.clone()), window);
        let mut seq = Sequence::new();
        let p = [50.0, 0.0, 10.0];
        let (Step::Moved(a), Step::Moved(b)) = (e.undo(p, 1, false, &mut seq), fold.undo(p)) else {
            unreachable!()
        };
        assert_eq!(a, b);
        assert!(
            (a[2] - (10.0 - 30.0)).abs() < 1e-9,
            "a crest: restored 30 lower"
        );
        // Outside: untouched, exactly.
        for q in [
            [400.0, 0.0, 10.0],
            [50.0, 600.0, 10.0],
            [-260.0, -10.0, 3.0],
        ] {
            assert_eq!(e.undo(q, 1, false, &mut seq), Step::Moved(q));
        }
        // Inside: purely vertical, between nothing and the whole fold.
        let q = [90.0, 250.0, 0.0];
        let (Step::Moved(a), Step::Moved(b)) = (e.undo(q, 1, false, &mut seq), fold.undo(q)) else {
            unreachable!()
        };
        assert!(a[0] == q[0] && a[1] == q[1]);
        assert!(a[2].abs() < b[2].abs() && a[2] * b[2] > 0.0);
    }

    #[test]
    fn a_windowed_fault_dies_out_at_its_tips_and_behind_its_trace() {
        // Striking north, dipping 60° east, normal, 20 of slip at the centre.
        let fault = Fault::translation([0.0; 3], 90.0, 60.0, 90.0, 20.0, Movement::HangingWall);
        // Trace 600 long; the slip reaches 250 into the hanging wall.
        let window = Window::new([0.0, 0.0], 0.0, 300.0, 250.0);
        assert_eq!(window.across, [1.0, -0.0]);
        let e = Windowed::new(Event::Fault(fault), window);
        let moved = |p: [f64; 3]| {
            let mut seq = Sequence::new();
            match e.undo(p, 1, false, &mut seq) {
                Step::Moved(q) => ((q[0] - p[0]).hypot(q[2] - p[2]), seq.get(1)),
                Step::Stop => unreachable!(),
            }
        };
        // Hanging wall at the trace's middle: the whole slip.
        let (d, code) = moved([1.0, 0.0, 0.0]);
        assert!(
            (d - 20.0 * bump(1.0 / 250.0)).abs() < 1e-9 && code == 2,
            "{d}"
        );
        // Foot wall: nothing, whatever the weight.
        assert_eq!(moved([-50.0, 0.0, -10.0]), (0.0, 0));
        // Hanging wall above the plane but west of the trace (high up): it moves too.
        let (d, code) = moved([-10.0, 0.0, 100.0]);
        assert!((d - 20.0 * bump(10.0 / 250.0)).abs() < 1e-9 && code == 2);
        // Towards a tip: less. Past it: none.
        let (d, _) = moved([1.0, 200.0, 0.0]);
        assert!(d > 1.0 && d < 10.0, "{d}");
        assert_eq!(moved([1.0, 300.0, 0.0]).0, 0.0);
        // Behind the trace: less, then none (the block is tilted, not dropped whole).
        let (near, _) = moved([50.0, 0.0, 0.0]);
        let (far, _) = moved([200.0, 0.0, 0.0]);
        assert!(near > far && far > 0.0);
        assert_eq!(moved([250.0, 0.0, 0.0]).0, 0.0);
    }
}
