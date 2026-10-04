// SPDX-License-Identifier: LGPL-3.0-or-later
//! Per-block cost of each candidate world projection's inverse (block -> unit vector on the planet).
//!
//! Research tool for docs/research/notes/seamless-projection-survey.md (2026-10-03). Not shipped.
//! Single file, no dependencies:
//!   rustc -C opt-level=3 -C target-cpu=native -o /tmp/projbench tools/research/projection-survey/bench.rs
//!   /tmp/projbench
//! Each inverse is evaluated on a 2,048 x 2,048 grid of one tile (4.2 million blocks), single thread;
//! the sum of the outputs is printed so nothing is optimised away. Ports: Petroff's inverse from his
//! CC0 code (Zenodo 4661351), Gringorten's from d3-geo-projection (ISC), QSC's from PROJ (MIT); the
//! rest written from the formulas in the note.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, SQRT_2};
use std::hint::black_box;
use std::time::Instant;

type V3 = [f64; 3];

fn ll(lat: f64, lon: f64) -> V3 {
    let (sl, cl) = lat.sin_cos();
    let (so, co) = lon.sin_cos();
    [cl * co, cl * so, sl]
}

fn norm(v: V3) -> V3 {
    let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / r, v[1] / r, v[2] / r]
}

// --- Lambert cylindrical equal-area (today's world): x = lon, y = sin(lat)
fn lambert(u: f64, v: f64) -> V3 {
    ll((v * 0.98).asin(), u * PI)
}

// --- HEALPix (H = 4, K = 3), tile coordinates mapped to x in [-pi, pi), y in [-pi/2, pi/2]
fn healpix(u: f64, v: f64) -> V3 {
    let x = u * PI;
    let y = v * FRAC_PI_2;
    let ay = y.abs();
    if ay <= FRAC_PI_4 {
        ll((8.0 / (3.0 * PI) * y).asin(), x)
    } else {
        let sigma = 2.0 - 4.0 * ay / PI;
        let xc = -PI + (2.0 * (2.0 * (x + PI) / PI).floor() + 1.0) * FRAC_PI_4;
        let lam = if sigma > 0.0 { xc + (x - xc) / sigma } else { xc };
        let s = y.signum() * (1.0 - sigma * sigma / 3.0);
        ll(s.asin(), lam)
    }
}

// --- gnomonic / equi-angular cube faces
fn gnomonic(u: f64, v: f64) -> V3 {
    norm([u, v, 1.0])
}
fn eac(u: f64, v: f64) -> V3 {
    norm([(FRAC_PI_4 * u).tan(), (FRAC_PI_4 * v).tan(), 1.0])
}

// --- QSC (PROJ qsc.cpp inverse, face-local)
fn qsc(u: f64, v: f64) -> V3 {
    let (area, x, y) = if u >= v.abs() {
        (0.0, u, v)
    } else if v >= u.abs() {
        (1.0, v, -u)
    } else if -u >= v.abs() {
        (2.0, -u, -v)
    } else {
        (3.0, -v, u)
    };
    let nu = (x * x + y * y).sqrt().atan();
    let mu = y.atan2(x);
    let t = PI / 12.0 * mu.tan();
    let th = (t.sin() / (t.cos() - 1.0 / SQRT_2)).atan();
    let cm = mu.cos();
    let tn = nu.tan();
    let cosphi = (1.0 - cm * cm * tn * tn * (1.0 - (1.0 / th.cos()).atan().cos())).clamp(-1.0, 1.0);
    let sinphi = (1.0 - cosphi * cosphi).sqrt();
    let (s, c) = (th + area * FRAC_PI_2).sin_cos();
    [sinphi * c, sinphi * s, cosphi]
}

// --- Rosca & Plonka 2011
fn rosca(u: f64, v: f64) -> V3 {
    let beta = (PI / 6.0).sqrt();
    let (x, y) = (beta * u, beta * v);
    let c4 = 2f64.powf(0.25);
    let (xx, yy) = if y.abs() <= x.abs() {
        if x == 0.0 {
            (0.0, 0.0)
        } else {
            let a = y * PI / (12.0 * x);
            let (s, c) = a.sin_cos();
            let d = (SQRT_2 - c).sqrt();
            (c4 * (x / beta) * (SQRT_2 * c - 1.0) / d, c4 * (x / beta) * SQRT_2 * s / d)
        }
    } else {
        let a = x * PI / (12.0 * y);
        let (s, c) = a.sin_cos();
        let d = (SQRT_2 - c).sqrt();
        (c4 * (y / beta) * SQRT_2 * s / d, c4 * (y / beta) * (SQRT_2 * c - 1.0) / d)
    };
    let q = xx * xx + yy * yy;
    let k = (1.0 - q / 4.0).max(0.0).sqrt();
    [k * xx, k * yy, 1.0 - q / 2.0]
}

// --- Peirce / Guyou: zeta = cn(w | 1/2) by the AGM (A&S 16.4) and the addition formula (16.21.3)
fn sncndn(u: f64, m: f64) -> (f64, f64, f64) {
    let mut a = [0.0f64; 10];
    let mut c = [0.0f64; 10];
    a[0] = 1.0;
    let mut b = (1.0 - m).sqrt();
    c[0] = m.sqrt();
    let mut n = 0;
    while n < 9 && c[n].abs() > 1e-15 {
        let an = a[n];
        a[n + 1] = 0.5 * (an + b);
        c[n + 1] = 0.5 * (an - b);
        b = (an * b).sqrt();
        n += 1;
    }
    let mut phi = (1u64 << n) as f64 * a[n] * u;
    for k in (1..=n).rev() {
        phi = 0.5 * (phi + (c[k] / a[k] * phi.sin()).asin());
    }
    let (s, cc) = phi.sin_cos();
    (s, cc, (1.0 - m * s * s).sqrt())
}

const K_HALF: f64 = 1.854_074_677_301_372; // K(1/2)

fn guyou(u: f64, v: f64) -> V3 {
    let (x, z) = ((u + v) / 2.0, (v - u) / 2.0);
    let wr = K_HALF + K_HALF * x;
    let wi = -K_HALF * z;
    let (sn, cn, dn) = sncndn(wr, 0.5);
    let (sn1, cn1, dn1) = sncndn(wi, 0.5);
    let den = cn1 * cn1 + 0.5 * sn * sn * sn1 * sn1;
    let zr = cn * cn1 / den;
    let zi = -sn * dn * sn1 * dn1 / den;
    let r2 = zr * zr + zi * zi;
    [2.0 * zr / (1.0 + r2), 2.0 * zi / (1.0 + r2), (1.0 - r2) / (1.0 + r2)]
}

// --- Petroff 2021 inverse (port of his CC0 new_projection_inverse)
struct Petroff {
    phi0: f64,
    a1: f64,
    a2: f64,
    hp: f64,
    xip: f64,
    p0p: f64,
    p1p: f64,
    p2p: f64,
}

impl Petroff {
    fn new() -> Petroff {
        let phi0 = 3.0 * PI / 8.0;
        let a1 = (1.0 / (2.0 - phi0.cos().powi(2)).sqrt()).asin();
        let a2 = (2.0 * phi0.sin() / (3.0 - (2.0 * phi0).cos()).sqrt()).asin();
        let hp = 12.0 / PI * (a1 + a2 - FRAC_PI_2);
        let xip = ((PI * (hp - 3.0).powi(2))
            / (3f64.sqrt() * (PI * (hp * hp - 2.0 * hp + 45.0) - 96.0 * a1 - 48.0 * a2)))
            .atan();
        let p0p = (3f64.sqrt() / hp).atan();
        let p1p = 7.0 * PI / 6.0 - p0p - xip;
        let p2p = xip - PI / 6.0;
        Petroff { phi0, a1, a2, hp, xip, p0p, p1p, p2p }
    }

    fn inv(&self, u: f64, v: f64) -> V3 {
        let (xm, ym) = ((u + v) / 2.0, (v - u) / 2.0);
        let p = self;
        let quadrant = (ym >= 0.0) as i32 + (xm <= 0.0) as i32 + 2 * ((ym <= 0.0 && xm < 0.0) as i32);
        let zeta = FRAC_PI_4 + quadrant as f64 * FRAC_PI_2;
        let (sz, cz) = zeta.sin_cos();
        let xc = 6f64.sqrt() * (xm * cz + ym * sz);
        let yc = 3.0 * SQRT_2 * (ym * cz - xm * sz);
        let (sg, yh) = if yc < -3.0 { (-1.0, -6.0 - yc) } else { (1.0, yc) };
        let rp = (xc * xc + (p.hp - 3.0 - yh).powi(2)).sqrt();
        let thp = xc.atan2(p.hp - 3.0 - yh).abs();
        let s0 = thp <= p.p0p;
        let s1 = !s0 && thp <= p.p0p + p.p1p;
        let alp = if s0 { thp } else if s1 { PI - p.p2p - thp } else { thp + p.p2p - PI };
        let c = if s0 || s1 { (p.phi0.cos() / SQRT_2).acos() } else { FRAC_PI_2 - p.phi0 };
        let psi0 = p.a1;
        let psi1 = PI - 2.0 * psi0;
        let g = if s1 { psi1 } else { psi0 };
        let gp = if s0 { p.p0p } else if s1 { p.p1p } else { p.p2p };
        let f = if s0 { p.a2 } else if s1 { FRAC_PI_2 - p.a2 } else { FRAC_PI_4 };
        let ap = if s0 {
            p.hp
        } else {
            (p.hp * p.hp + 3.0).sqrt() * (PI / 3.0 - (p.hp / 3f64.sqrt()).atan()).sin() / p.xip.sin()
        };
        let cp = if s0 || s1 { (p.hp * p.hp + 3.0).sqrt() } else { 3.0 - p.hp };
        let b = if s0 {
            FRAC_PI_4
        } else if s1 {
            (SQRT_2 * p.phi0.tan()).atan()
        } else {
            FRAC_PI_2 - (SQRT_2 * p.phi0.tan()).atan()
        };
        let bp = gp - alp;
        let xp = (rp * rp + cp * cp - 2.0 * rp * cp * bp.cos()).sqrt();
        let gpp = if xp > 0.0 { (rp * bp.sin() / xp).clamp(-1.0, 1.0).asin() } else { 0.0 };
        let ep = PI - gp - gpp;
        let yp = if ep > 0.0 { rp * alp.sin() / ep.sin() } else { 0.0 };
        let up = (cp * cp + (xp + yp).powi(2) - 2.0 * cp * (xp + yp) * gpp.cos()).abs().sqrt();
        let vp = ap - up;
        let k = vp * (f + g - FRAC_PI_2) / ap;
        let delta = (-k.sin() / (b.cos() - k.cos())).atan();
        let gamma = f - delta;
        let cos_xy = 1.0 / (1.0 + (b.tan() / delta.cos()).powi(2)).sqrt();
        let x = (1.0 - (xp / (xp + yp)).powi(2) * (1.0 - cos_xy)).clamp(-1.0, 1.0).acos();
        let r = (x.cos() * c.cos() + x.sin() * c.sin() * gamma.cos()).clamp(-1.0, 1.0).acos();
        let beta = (x.sin() * gamma.sin() / r.sin()).clamp(-1.0, 1.0).asin();
        let alpha = if s0 { psi0 - beta } else if s1 { psi0 + beta } else { PI - beta };
        let phih = (p.phi0.sin() * r.cos() - p.phi0.cos() * r.sin() * alpha.cos()).clamp(-1.0, 1.0).asin();
        let lam0 = FRAC_PI_4 + quadrant as f64 * FRAC_PI_2;
        let lam = lam0
            + xc.signum()
                * (alpha.sin() * r.sin() * p.phi0.cos() / (r.cos() - p.phi0.sin() * phih.sin())).atan();
        ll(sg * phih, lam)
    }
}

// --- Gringorten (d3-geo-projection's hexadecant inverse: bisection, 50 steps)
fn gringorten(u: f64, v: f64) -> V3 {
    let sx = if u >= 0.0 { 1.0 } else { -1.0 };
    let sy = if v >= 0.0 { 1.0 } else { -1.0 };
    let x0 = -sx * u;
    let y0 = -sy * v;
    let t = y0 / x0 < 1.0;
    let (x, y) = if t { (y0, x0) } else { (x0, y0) };
    let (mut lo, mut hi, mut r) = (0.0f64, 1.0f64, 0.5f64);
    let (mut sin_phi, mut z, mut vv, mut p2, mut a2, mut g) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for _ in 0..50 {
        let r2 = r * r;
        sin_phi = r.sqrt();
        z = (1.0 / (1.0 + r2).sqrt()).asin();
        vv = (1.0 - r2) + r * (1.0 + r2) * z;
        p2 = (1.0 - sin_phi) / vv;
        let p = p2.sqrt();
        a2 = p2 * (1.0 + r2);
        let h = p * (1.0 - r2);
        g = (a2 - x * x).max(0.0).sqrt();
        let yy = y + h + r * g;
        if (hi - lo).abs() < 1e-12 || yy == 0.0 {
            break;
        }
        if yy > 0.0 {
            lo = r
        } else {
            hi = r
        }
        r = 0.5 * (lo + hi);
    }
    let r2 = r * r;
    let phi = sin_phi.asin();
    let cphi = phi.cos();
    let sec = 1.0 / cphi;
    let p = p2.sqrt();
    let drd = 2.0 * sin_phi * cphi;
    let dvd = (-3.0 * r + z * (1.0 + 3.0 * r2)) * drd;
    let dp2 = (-vv * cphi - (1.0 - sin_phi) * dvd) / (vv * vv);
    let dpd = 0.5 * dp2 / p;
    let dhd = (1.0 - r2) * dpd - 2.0 * r * p * drd;
    let zeta = -2.0 * sec * dhd;
    let mu = -sec * drd;
    let nu = -sec * (r * (1.0 + r2) * dp2 + p2 * (1.0 + 3.0 * r2) * drd);
    let mut lam = FRAC_PI_4 * (x * (zeta + mu * g) + nu * (x / a2.sqrt()).clamp(-1.0, 1.0).asin());
    if t {
        lam = -FRAC_PI_2 - lam;
    }
    let l = sx * ((lam.sin() * cphi).atan2(-phi.sin()) + PI);
    let pp = sy * (lam.cos() * cphi).clamp(-1.0, 1.0).asin();
    let q = ll(pp, l);
    [q[1], q[2], q[0]]
}

// --- a precomputed table (e.g. the Moser-flow cube face), 513 x 513 vectors, bilinear
struct Table {
    n: usize,
    d: Vec<V3>,
}

impl Table {
    fn new(n: usize) -> Table {
        let mut d = Vec::with_capacity(n * n);
        for i in 0..n {
            for j in 0..n {
                let u = -1.0 + 2.0 * i as f64 / (n - 1) as f64;
                let v = -1.0 + 2.0 * j as f64 / (n - 1) as f64;
                d.push(rosca(u, v));
            }
        }
        Table { n, d }
    }
    fn get(&self, u: f64, v: f64) -> V3 {
        let s = (self.n - 1) as f64;
        let fx = ((u + 1.0) * 0.5 * s).clamp(0.0, s - 1e-9);
        let fy = ((v + 1.0) * 0.5 * s).clamp(0.0, s - 1e-9);
        let (i, j) = (fx as usize, fy as usize);
        let (tx, ty) = (fx - i as f64, fy - j as f64);
        let a = self.d[i * self.n + j];
        let b = self.d[(i + 1) * self.n + j];
        let c = self.d[i * self.n + j + 1];
        let e = self.d[(i + 1) * self.n + j + 1];
        let mut o = [0.0; 3];
        for k in 0..3 {
            o[k] = (a[k] * (1.0 - tx) + b[k] * tx) * (1.0 - ty) + (c[k] * (1.0 - tx) + e[k] * tx) * ty;
        }
        norm(o)
    }
}

fn bench(name: &str, f: &dyn Fn(f64, f64) -> V3) {
    let n = 2048usize;
    let mut acc = 0.0;
    // warm up
    for i in 0..n {
        let u = -0.999 + 1.998 * i as f64 / n as f64;
        acc += f(u, 0.3)[0];
    }
    let t0 = Instant::now();
    for i in 0..n {
        let u = -0.9995 + 1.999 * (i as f64 + 0.5) / n as f64;
        for j in 0..n {
            let v = -0.9995 + 1.999 * (j as f64 + 0.5) / n as f64;
            let p = f(black_box(u), black_box(v));
            acc += p[0] + p[1] + p[2];
        }
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("{name:<28} {:8.1} ns/block   (checksum {acc:.3})", dt * 1e9 / (n * n) as f64);
}

fn main() {
    let pet = Petroff::new();
    let tab = Table::new(513);
    bench("lambert-cyl (shipped)", &lambert);
    bench("healpix", &healpix);
    bench("cube-gnomonic", &gnomonic);
    bench("cube-eac", &eac);
    bench("cube-qsc", &qsc);
    bench("cube-rosca-plonka", &rosca);
    bench("table-513-bilinear", &|u, v| tab.get(u, v));
    bench("guyou-peirce (cn)", &guyou);
    bench("petroff", &|u, v| pet.inv(u, v));
    bench("gringorten (bisection)", &gringorten);
}
