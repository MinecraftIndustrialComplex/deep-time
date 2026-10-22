// SPDX-License-Identifier: LGPL-3.0-or-later
// The planet's map projection for the pages (the workbench and the static viewer): the one place
// they convert between block coordinates, map pictures and latitude/longitude. The twin of the Rust
// `deeptime_format::projection` and of Java's `worlddata/Projection`
// (docs/design/equal-area-projection.md §2); loaded before every other script as `window.DTProj`.
//
// Lambert's cylindrical equal-area projection, true on the equator (ledger: SCL-049): R = C/2π,
// longitude λ = 360°·(x + C/2)/C (the seam x = −C/2 is 0°), latitude φ = asin(−z/R) (north is −z),
// the poles at z = ∓R; east–west a block is cos φ blocks of planet, north–south 1/cos φ. A map
// picture (a keyframe) is π : 1: x linear in longitude from the seam, y linear in sin φ from the
// north pole (z = −R + 2R·y/h).
(function (root) {
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  const D2R = Math.PI / 180;
  const R = C => C / (2 * Math.PI);
  const P = {
    /** R = C/2π, blocks. */
    radius: R,
    /** |z| of the poles, blocks: R. */
    poleZ: C => R(C),
    /** Block z of the north edge of the workbench's tile pyramid (a 2 : 1 frame wider than the planet). */
    tileFrameZ: C => -C / 4,
    /** Whether block z is on the planet (between the poles; x wraps, so every x is). */
    onPlanet: (C, z) => Math.abs(z) <= R(C),
    /** Latitude, degrees, of block z (clamped to ±90° past the poles). */
    latOfZ: (C, z) => Math.asin(clamp(-z / R(C), -1, 1)) / D2R,
    /** Block z of a latitude. */
    zOfLat: (C, lat) => -R(C) * Math.sin(clamp(lat, -90, 90) * D2R),
    /** Longitude, degrees in [0, 360), of block x. */
    lonOfX: (C, x) => (((x + C / 2) / C) * 360 % 360 + 360) % 360,
    /** Block x of a longitude (not wrapped). */
    xOfLon: (C, lon) => lon / 360 * C - C / 2,
    /** Degrees of longitude and latitude a step of (dx, dz) blocks spans at latitude `lat` (local). */
    dLonOfDx: (C, dx, lat) => dx / C * 360,
    dLatOfDz: (C, dz, lat) => -dz / (R(C) * Math.max(Math.cos(lat * D2R), 1e-6)) / D2R,
    /** Blocks a step of (dlon, dlat) degrees spans at latitude `lat` (local). */
    dxOfDLon: (C, dlon, lat) => dlon / 360 * C,
    dzOfDLat: (C, dlat, lat) => -dlat * D2R * R(C) * Math.cos(lat * D2R),
    /** Wraps a longitude into [0, 360). */
    wrapLon: lon => ((lon % 360) + 360) % 360,
    /** Column coordinate of a longitude (as given, not wrapped) in a map picture `w` columns wide
     *  (x = 0 the seam). */
    xOfLonPic: (lon, w) => lon / 360 * w,
    /** Longitude (not wrapped) at column coordinate x of a map picture `w` columns wide. */
    lonOfPicX: (x, w) => x / w * 360,
    /** Row coordinate of a latitude in a map picture `h` rows high (y = 0 the north pole). */
    yOfLat: (lat, h) => (1 - Math.sin(clamp(lat, -90, 90) * D2R)) / 2 * h,
    /** Latitude at row coordinate y of a map picture `h` rows high. */
    latOfY: (y, h) => Math.asin(clamp(1 - 2 * y / h, -1, 1)) / D2R,
    /** Row coordinate of block z in a map picture `h` rows high (it spans pole to pole). */
    yOfZ: (C, z, h) => (z + R(C)) / (2 * R(C)) * h,
    /** The picture's row share (0 north … 1 south) of sin φ, for the globe's texture lookups. */
    vOfSinLat: s => (1 - clamp(s, -1, 1)) / 2,
    /** A map picture's width over its height: π. */
    aspect: Math.PI,
    /** The height of a map picture `w` wide (the projection's aspect, π : 1). */
    pictureHeight: w => Math.max(1, Math.round(w / Math.PI)),
  };
  root.DTProj = P;
})(typeof window !== 'undefined' ? window : globalThis);

// ---------------------------------------------------------------------------------------------
// The seamless world (format 0.5): the portable trigonometry, Petroff's square and the
// Petroff–Guyou world, bit-for-bit twins of `deeptime_format::portable_math`, `petroff` and
// `projection::PetroffGuyou` (and of Java's `PortableMath`, `Petroff`, `WorldProjection`): the
// same IEEE double operations in the same order, `Math.sqrt` (correctly rounded) and comparisons
// only. `tools/viewer/projection-check.js` checks every output bit against
// `crates/deeptime-format/tests/data/projection_parity.json`.
(function (root) {
  // ---- DTMath ---------------------------------------------------------------------------------
  const P1 = 1.57079632673412561417;      // π/2's high part, 33 significant bits (fdlibm pio2_1)
  const P1T = 6.07710050650619224932e-11;  // π/2 − P1
  const HALF_PI = 1.570796326794896558;
  const HALF_PI_LO = 6.123233995736766036e-17;
  const PI = 3.141592653589793116;
  const PI_LO = 1.2246467991473532072e-16;

  const kernelSin = r => {
    const z = r * r;
    const p = -1.0 / 6.0
      + z * (1.0 / 120.0
        + z * (-1.0 / 5040.0
          + z * (1.0 / 362880.0
            + z * (-1.0 / 39916800.0
              + z * (1.0 / 6227020800.0
                + z * (-1.0 / 1307674368000.0
                  + z * (1.0 / 355687428096000.0
                    + z * (-1.0 / 121645100408832000.0
                      + z * (1.0 / 51090942171709440000.0
                        + z * (-1.0 / 25852016738884976640000.0))))))))));
    return r + r * (z * p);
  };
  const kernelCos = r => {
    const z = r * r;
    const q = 1.0 / 24.0
      + z * (-1.0 / 720.0
        + z * (1.0 / 40320.0
          + z * (-1.0 / 3628800.0
            + z * (1.0 / 479001600.0
              + z * (-1.0 / 87178291200.0
                + z * (1.0 / 20922789888000.0
                  + z * (-1.0 / 6402373705728000.0
                    + z * (1.0 / 2432902008176640000.0
                      + z * (-1.0 / 1124000727777607680000.0
                        + z * (1.0 / 620448401733239439360000.0))))))))));
    const hz = 0.5 * z;
    const w = 1.0 - hz;
    return w + (((1.0 - w) - hz) + (z * z) * q);
  };
  const roundHalfAway = x => x >= 0.0 ? Math.floor(x + 0.5) : -Math.floor(-x + 0.5);
  const reduce = x => {
    const k = roundHalfAway(x * (2.0 / PI));
    const r = (x - k * P1) - k * P1T;
    let q = k % 4;
    if (q < 0) q += 4;
    return [q === 0 ? 0 : q, r];   // −0 is quadrant 0
  };
  const isFin = x => Number.isFinite(x);
  const sin = x => {
    if (!isFin(x)) return NaN;
    const [q, r] = reduce(x);
    return q === 0 ? kernelSin(r) : q === 1 ? kernelCos(r) : q === 2 ? -kernelSin(r) : -kernelCos(r);
  };
  const cos = x => {
    if (!isFin(x)) return NaN;
    const [q, r] = reduce(x);
    return q === 0 ? kernelCos(r) : q === 1 ? -kernelSin(r) : q === 2 ? -kernelCos(r) : kernelSin(r);
  };
  const tan = x => sin(x) / cos(x);
  const atanUnit = t => {
    let s = t;
    s = s / (1.0 + Math.sqrt(1.0 + s * s));
    s = s / (1.0 + Math.sqrt(1.0 + s * s));
    s = s / (1.0 + Math.sqrt(1.0 + s * s));
    const z = s * s;
    const p = -1.0 / 3.0
      + z * (1.0 / 5.0
        + z * (-1.0 / 7.0
          + z * (1.0 / 9.0
            + z * (-1.0 / 11.0
              + z * (1.0 / 13.0
                + z * (-1.0 / 15.0
                  + z * (1.0 / 17.0
                    + z * (-1.0 / 19.0
                      + z * (1.0 / 21.0
                        + z * (-1.0 / 23.0
                          + z * (1.0 / 25.0
                            + z * (-1.0 / 27.0
                              + z * (1.0 / 29.0
                                + z * (-1.0 / 31.0))))))))))))));
    return 8.0 * (s + s * (z * p));
  };
  const negZero = x => x === 0 && 1 / x < 0;
  const atan = x => {
    if (Number.isNaN(x)) return NaN;
    const a = Math.abs(x);
    let r;
    if (a <= 1.0) r = atanUnit(a);
    else if (a === Infinity) r = HALF_PI;
    else r = HALF_PI - (atanUnit(1.0 / a) - HALF_PI_LO);
    return x < 0.0 ? -r : r;
  };
  const atan2 = (y, x) => {
    if (Number.isNaN(x) || Number.isNaN(y)) return NaN;
    const ax = Math.abs(x), ay = Math.abs(y);
    if (ax === 0 && ay === 0) {
      const a = (x < 0 || negZero(x)) ? PI : 0.0;
      return (y < 0 || negZero(y)) ? -a : a;
    }
    let a;
    if (ay <= ax) {
      if (ax === Infinity) a = ay === Infinity ? atanUnit(1.0) : 0.0;
      else a = atanUnit(ay / ax);
    } else if (ay === Infinity) {
      a = HALF_PI;
    } else {
      a = HALF_PI - (atanUnit(ax / ay) - HALF_PI_LO);
    }
    if (x < 0.0) a = PI - (a - PI_LO);
    return (y < 0.0 || negZero(y)) ? -a : a;
  };
  const clamp = (v, lo, hi) => v < lo ? lo : (v > hi ? hi : v);
  const asin = x => {
    if (Number.isNaN(x)) return NaN;
    x = clamp(x, -1.0, 1.0);
    return atan2(x, Math.sqrt((1.0 - x) * (1.0 + x)));
  };
  const acos = x => {
    if (Number.isNaN(x)) return NaN;
    x = clamp(x, -1.0, 1.0);
    return atan2(Math.sqrt((1.0 - x) * (1.0 + x)), x);
  };
  const toRadians = d => d * (PI / 180.0);
  const toDegrees = r => r * (180.0 / PI);
  const remEuclid = (a, m) => { const r = a % m; return r < 0 ? r + Math.abs(m) : r; };
  const wrapDeg = a => { const w = remEuclid(a, 360.0); return w >= 360.0 ? 0.0 : w; };
  const M = { sin, cos, tan, atan, atan2, asin, acos, toRadians, toDegrees, PI, HALF_PI };

  // ---- Petroff ---------------------------------------------------------------------------------
  const sign = x => x > 0.0 ? 1.0 : (x < 0.0 ? -1.0 : 0.0);
  const sqrt0 = x => x > 0.0 ? Math.sqrt(x) : 0.0;
  // The forward's switch to the coordinate form of its last step, in local units (the Rust twin's
  // NEAR_SIDE: a numerical switch, not a physical value; ledger: SCL-083).
  const NEAR_SIDE = 1e-6;
  function Petroff() {
    const phi0 = 3.0 * PI / 8.0;
    const sinPhi0 = sin(phi0), cosPhi0 = cos(phi0);
    const a1 = asin(1.0 / Math.sqrt(2.0 - cosPhi0 * cosPhi0));
    const a2 = asin(2.0 * sinPhi0 / Math.sqrt(3.0 - cos(2.0 * phi0)));
    const hp = 12.0 / PI * (a1 + a2 - PI / 2.0);
    const s3 = Math.sqrt(3.0);
    const xip = atan((PI * (hp - 3.0) * (hp - 3.0))
      / (s3 * (PI * (hp * hp - 2.0 * hp + 45.0) - 96.0 * a1 - 48.0 * a2)));
    const psi0 = a1, psi1 = PI - 2.0 * psi0;
    const p0p = atan(s3 / hp);
    const p1p = 7.0 * PI / 6.0 - p0p - xip;
    const p2p = xip - PI / 6.0;
    const cA = acos(cosPhi0 / Math.sqrt(2.0));
    const cB = PI / 2.0 - phi0;
    const a1p = Math.sqrt(hp * hp + 3.0) * sin(PI / 3.0 - atan(hp / s3)) / sin(xip);
    const b1 = atan(Math.sqrt(2.0) * tan(phi0));
    const b2 = PI / 2.0 - b1;
    const c01p = Math.sqrt(hp * hp + 3.0), c2p = 3.0 - hp;
    const f0 = a2, f1 = PI / 2.0 - a2;
    const tanB = [tan(PI / 4.0), tan(b1), tan(b2)];

    this.forward = (lambda, phi) => {
      const twoPi = 2.0 * PI;
      let lam = lambda - twoPi * Math.floor(lambda / twoPi);
      if (lam >= twoPi || lam < 0.0) lam = 0.0;
      const phiC = Math.abs(phi);
      const lambdaC = lam - PI / 4.0;
      let quadrant = Math.floor(2.0 * lam / PI);
      if (quadrant > 3.0) quadrant = 3.0;
      const lambda0 = quadrant * PI / 2.0;
      const dl = lambdaC - lambda0;
      const sinPc = sin(phiC), cosPc = cos(phiC);
      const sinDl = sin(dl), cosDl = cos(dl);
      const theta = Math.abs(atan2(cosPc * sinDl, sinPhi0 * cosPc * cosDl - cosPhi0 * sinPc));
      const r = acos(sinPhi0 * sinPc + cosPhi0 * cosPc * cosDl);
      const s = theta <= psi0 ? 0 : (theta <= psi0 + psi1 ? 1 : 2);
      const beta = s === 0 ? psi0 - theta : (s === 1 ? theta - psi0 : PI - theta);
      const c = s <= 1 ? cA : cB;
      const g = s === 1 ? psi1 : psi0;
      const gp = [p0p, p1p, p2p][s];
      const f = [f0, f1, PI / 4.0][s];
      const ap = s === 0 ? hp : a1p;
      const cp = s <= 1 ? c01p : c2p;
      const sinR = sin(r), cosR = cos(r);
      const sinC = sin(c), cosC = cos(c);
      const x = acos(cosR * cosC + sinR * sinC * cos(beta));
      const gamma = x > 0.0 ? asin(sin(beta) * sinR / sin(x)) : 0.0;
      const sinG = sin(g), cosG = cos(g);
      const epsilon = acos(sinG * sin(gamma) * cosC - cosG * cos(gamma));
      const upupvp = (gamma + g + epsilon - PI) / (f + g - PI / 2.0);
      const sinEps = sin(epsilon);
      let cosXy;
      if (sinEps !== 0.0) { const q = sinG * sinC / sinEps; cosXy = sqrt0(1.0 - q * q); } else cosXy = 0.0;
      const den = 1.0 - cosXy;
      const xpxpyp = den > 0.0 ? sqrt0((1.0 - cos(x)) / den) : 0.0;
      const up = ap * upupvp;
      const xpyp = sqrt0(up * up + cp * cp - 2.0 * up * cp * cos(gp));
      const xp = xpyp * xpxpyp;
      const yp = xpyp - xp;
      let rp, alphap;
      if (up <= NEAR_SIDE || yp <= NEAR_SIDE) {
        // P′ on a side of the planar triangle: place it by coordinates (see the Rust twin).
        const cx = cp * cos(gp), cy = cp * sin(gp);
        const px = cx + xpxpyp * (up - cx);
        const py = cy - xpxpyp * cy;
        rp = Math.sqrt(px * px + py * py);
        alphap = atan2(py, px);
      } else {
        let cosGammap;
        if (xpyp > 0.0) { const q = up * sin(gp) / xpyp; cosGammap = sqrt0(1.0 - q * q); } else cosGammap = 1.0;
        rp = sqrt0(xp * xp + cp * cp - 2.0 * xp * cp * cosGammap);
        if (up * rp <= 0.0) alphap = xip - PI / 6.0;
        else { const q = (yp * yp - up * up - rp * rp) / (-2.0 * up * rp); alphap = q < 1.0 ? acos(q) : 0.0; }
      }
      const thetap = s === 0 ? alphap
        : (s === 1 ? PI - (xip - PI / 6.0) - alphap : PI - (xip - PI / 6.0) + alphap);
      const side = dl < 0.0 ? -1.0 : 1.0;
      const xC = side * rp * sin(thetap);
      const yC = hp - rp * cos(thetap);
      const yH = yC * sign(phi) - 3.0;
      const zeta = PI / 4.0 + quadrant * PI / 2.0;
      const sinZ = sin(zeta), cosZ = cos(zeta);
      const k = s3 / (3.0 * Math.sqrt(2.0));
      return [(xC * cosZ - yH * sinZ / s3) * k, (xC * sinZ + yH * cosZ / s3) * k];
    };

    this.inverse = (xM, yM) => {
      const quadrant = (xM <= 0.0 && yM >= 0.0) ? 2.0 : (yM >= 0.0 ? 1.0 : (xM >= 0.0 ? 0.0 : 3.0));
      const zeta = PI / 4.0 + quadrant * PI / 2.0;
      const sinZ = sin(zeta), cosZ = cos(zeta);
      const xC = Math.sqrt(6.0) * (xM * cosZ + yM * sinZ);
      const yC = 3.0 * Math.sqrt(2.0) * (yM * cosZ - xM * sinZ);
      let sgnphi, yH;
      if (yC < -3.0) { sgnphi = -1.0; yH = -6.0 - yC; } else { sgnphi = 1.0; yH = yC; }
      const dy = hp - 3.0 - yH;
      const rp = Math.sqrt(xC * xC + dy * dy);
      const thetap = Math.abs(atan2(xC, dy));
      const s = thetap <= p0p ? 0 : (thetap <= p0p + p1p ? 1 : 2);
      const alphap = s === 0 ? thetap : (s === 1 ? PI - p2p - thetap : thetap + p2p - PI);
      const c = s <= 1 ? cA : cB;
      const g = s === 1 ? psi1 : psi0;
      const gp = [p0p, p1p, p2p][s];
      const f = [f0, f1, PI / 4.0][s];
      const ap = s === 0 ? hp : a1p;
      const cp = s <= 1 ? c01p : c2p;
      const b = [PI / 4.0, b1, b2][s];
      const tb = tanB[s];
      const betap = gp - alphap;
      const xp = sqrt0(rp * rp + cp * cp - 2.0 * rp * cp * cos(betap));
      const gammap = xp > 0.0 ? asin(rp * sin(betap) / xp) : 0.0;
      const epsp = PI - gp - gammap;
      const yp = epsp > 0.0 ? rp * sin(alphap) / sin(epsp) : 0.0;
      const xy = xp + yp;
      const up = Math.sqrt(Math.abs(cp * cp + xy * xy - 2.0 * cp * xy * cos(gammap)));
      const vp = ap - up;
      const w = vp * (f + g - PI / 2.0) / ap;
      let num = -sin(w);
      let den = cos(b) - cos(w);
      const delta = (den === 0.0 && num === 0.0) ? 0.0 : atan(num / den);
      const gamma = f - delta;
      const tq = tb / cos(delta);
      const cosXy = 1.0 / Math.sqrt(1.0 + tq * tq);
      let x;
      if (xy > 0.0) { const q = xp / xy; x = acos(1.0 - q * q * (1.0 - cosXy)); } else x = 0.0;
      const sinX = sin(x), cosX = cos(x);
      const sinC = sin(c), cosC = cos(c);
      const r = acos(cosX * cosC + sinX * sinC * cos(gamma));
      const sinR = sin(r);
      const beta = sinR !== 0.0 ? asin(sinX * sin(gamma) / sinR) : 0.0;
      const alpha = s === 0 ? psi0 - beta : (s === 1 ? psi0 + beta : PI - beta);
      const cosR = cos(r);
      const phiH = asin(sinPhi0 * cosR - cosPhi0 * sinR * cos(alpha));
      const lambda0 = PI / 4.0 + quadrant * PI / 2.0;
      const phi = sgnphi * phiH;
      num = sin(alpha) * sinR * cosPhi0;
      den = cosR - sinPhi0 * sin(phiH);
      const dl = (den === 0.0 && num === 0.0) ? 0.0 : atan(num / den);
      let lambda = lambda0 + sign(xC) * dl;
      if (Math.abs(phi) === PI / 2.0) lambda = 0.0;
      return [lambda, phi];
    };
  }

  // ---- Petroff–Guyou ---------------------------------------------------------------------------
  const dot3 = (p, q) => p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
  const unitPm = (lambda, phi) => {
    const sp = sin(phi), cp = cos(phi);
    return [cp * cos(lambda), cp * sin(lambda), sp];
  };
  const latLonOfUnit = v => {
    const h = Math.sqrt(v[0] * v[0] + v[1] * v[1]);
    const lat = toDegrees(atan2(v[2], h));
    const lon = h === 0 ? 0.0 : wrapDeg(toDegrees(atan2(v[1], v[0])));
    return [lat, lon];
  };
  /** The Petroff–Guyou world of hemisphere side b and western centre longitude λc (degrees). */
  function PetroffGuyou(b, lambdaCDeg) {
    const pet = new Petroff();
    const le = pet.inverse(0.5, 0.5)[0];
    const e = [cos(le), sin(le), 0.0];
    const a = [0.0, 0.0, 1.0];
    const f = [-e[1], e[0], 0.0];
    const lc = toRadians(lambdaCDeg);
    const a2 = [cos(lc), sin(lc), 0.0];
    const e2 = [0.0, 0.0, 1.0];
    const f2 = [a2[1], -a2[0], 0.0];
    const period = 2 * b;
    const wrapX = x => { const t0 = remEuclid(x + 0.5 * period, period); return (t0 >= period ? 0.0 : t0) - 0.5 * period; };
    const fold = (x, z) => {
      const half = b / 2.0;
      let n = 0;
      for (let k = 0; k < 64; k++) {
        if (z < -half) { x = -x; z = -b - z; n++; }
        else if (z > half) { x = -x; z = b - z; n++; }
        else break;
      }
      return [wrapX(x), clamp(z, -half, half), n];
    };
    const toWorld = v => {
      const pa = dot3(a, v), pe = dot3(e, v), pf = dot3(f, v);
      return [a2[0] * pa + e2[0] * pe + f2[0] * pf, a2[1] * pa + e2[1] * pe + f2[1] * pf, a2[2] * pa + e2[2] * pe + f2[2] * pf];
    };
    const toPolar = v => {
      const pa = dot3(a2, v), pe = dot3(e2, v), pf = dot3(f2, v);
      return [a[0] * pa + e[0] * pe + f[0] * pf, a[1] * pa + e[1] * pe + f[1] * pf, a[2] * pa + e[2] * pe + f[2] * pf];
    };
    const unitInTile = (x, z) => {
      const tx = 2.0 * x / b + 1.0;
      const ty = -2.0 * z / b;
      let u, v, south;
      if (tx <= 1.0) { u = tx; v = ty; south = false; } else { u = 2.0 - tx; v = ty; south = true; }
      const ll = pet.inverse(0.5 * (u + v), 0.5 * (v - u));
      const phi = south ? -ll[1] : ll[1];
      return toWorld(unitPm(ll[0], phi));
    };
    const unit = (x, z) => { const p = fold(x, z); return unitInTile(p[0], p[1]); };
    const pointOfUnit = v => {
      const q = toPolar(v);
      const phi = atan2(q[2], Math.sqrt(q[0] * q[0] + q[1] * q[1]));
      const lam = atan2(q[1], q[0]);
      const south = phi < 0.0;
      const xy = pet.forward(lam, south ? -phi : phi);
      const u = xy[0] - xy[1], vv = xy[0] + xy[1];
      const tx = south ? 2.0 - u : u;
      const x = (tx - 1.0) * b / 2.0;
      const z = -vv * b / 2.0;
      return [wrapX(x), clamp(z, -b / 2.0, b / 2.0)];
    };
    const latLon = (x, z) => latLonOfUnit(unit(x, z));
    const blockOf = (latDeg, lonDeg) => {
      const lat = toRadians(clamp(latDeg, -90.0, 90.0));
      const lon = toRadians(lonDeg);
      return pointOfUnit(unitPm(lon, lat));
    };
    this.kind = 'petroff_guyou';
    this.isPetroff = true;
    this.b = b;
    this.lambdaC = lambdaCDeg;
    this.period = period;
    this.radius = b / Math.sqrt(2.0 * Math.PI);
    this.zRange = [-b / 2.0, b / 2.0];
    this.wrapX = wrapX;
    this.fold = fold;
    this.unit = unit;
    this.pointOfUnit = pointOfUnit;
    this.latLon = latLon;
    this.blockOf = blockOf;
    this.canonical = (x, z) => { const p = fold(x, z); return [p[0], p[1]]; };
  }

  /** A Lambert world in the same shape as PetroffGuyou (the cylinder's formulas of DTProj). */
  function LambertWorld(C) {
    const P = root.DTProj;
    this.kind = 'lambert';
    this.isPetroff = false;
    this.period = C;
    this.radius = C / (2 * Math.PI);
    this.zRange = [-P.poleZ(C), P.poleZ(C)];
    this.wrapX = x => { const t0 = remEuclid(x + 0.5 * C, C); return (t0 >= C ? 0.0 : t0) - 0.5 * C; };
    this.latLon = (x, z) => [P.latOfZ(C, z), P.lonOfX(C, x)];
    this.blockOf = (lat, lon) => [this.wrapX(P.xOfLon(C, lon)), P.zOfLat(C, lat)];
    this.canonical = (x, z) => [this.wrapX(x), clamp(z, this.zRange[0], this.zRange[1])];
  }

  root.DTMath = M;
  root.DTPetroff = Petroff;
  root.DTPetroffGuyou = PetroffGuyou;
  root.DTLatLonOfUnit = latLonOfUnit;
  /** The world projection a manifest's `projection` block describes (absent or Lambert: the
   *  cylinder of circumference C). */
  root.DTProj.world = (block, C) => (block && block.kind === 'petroff_guyou')
    ? new PetroffGuyou(block.hemisphere_blocks, block.lambda_c_deg)
    : new LambertWorld(C);
})(typeof window !== 'undefined' ? window : globalThis);

// ---------------------------------------------------------------------------------------------
// The cube world (docs/plans/cube-world-plan.md, work items W1, W2 and M4; Ben, decisions log
// 2026-10-22): six square faces of side s in a cross-shaped net, and the map from a face's local
// point to the sphere, from a table. Bit-for-bit twins of `deeptime_format::cube` and
// `deeptime_format::cube_map` (and of Java's `worlddata/CubeLayout` and `worlddata/CubeMap`):
//   DTCube       the net's integer and plane geometry (Rust `Cube`, `Iso`, `ADJACENCY`, `face_for`)
//   DTCubeTurn   the rotation that places the cube on the planet (Rust `CubeTurn`)
//   DTCubeMap    the table reader and its evaluator (Rust `CubeMap`), arithmetic of + - * / and
//                Math.sqrt only, in the Rust source's order and grouping, never a fused multiply-add
//                and never Math.floor
//   DTCubeWorld  a net, a table and a turn together (Rust `CubeWorld`): unit vectors, latitude,
//                longitude, the poles and the metres-per-block Jacobian
// `tools/viewer/projection-check.js` checks every output bit against the "cube" member of
// `crates/deeptime-format/tests/data/projection_parity.json`. Integers are JS numbers (exact up to
// 2^53, far beyond any block coordinate); Rust's i64 `rem_euclid` and `div_euclid` are `remI` and
// `divI`. A Rust `Option` is a value or `null`. Not wired into `DTProj.world` yet (plan T1).
(function (root) {
  const M = root.DTMath;
  const latLonOfUnit = root.DTLatLonOfUnit;
  const sqrt = Math.sqrt;

  /** The face side is a multiple of this many blocks: every grid's r (up to 256) divides s, the
   *  east-west period 4s is a multiple of 512 and every turn pivot sits on a chunk corner. */
  const FACE_QUANTUM = 256; // ledger: SCL-100 (game-choice)
  /** The world's map: the conformal cube. */
  const WORLD_MAP = 'conformal-n513'; // ledger: SCL-101
  /** The simulation's map: Moser's equal-area cube on the polar axis. */
  const SIMULATION_MAP = 'moser-t1-n513'; // ledger: SCL-102
  /** Newton steps of the inverse after the inverse table's start. */
  const NEWTON_STEPS = 4; // ledger: SCL-103

  // ---- integers (Rust's i64 and f64::round) -----------------------------------------------------
  /** a mod m in [0, m) for an integer m > 0 (i64::rem_euclid); never -0. */
  const remI = (a, m) => { const r = a % m; return (r < 0 ? r + m : r) + 0; };
  /** i64::div_euclid for an integer m > 0 (exact: a minus its remainder is a multiple of m). */
  const divI = (a, m) => (a - remI(a, m)) / m;
  /** f64::round (half away from zero) `as i64`: 0 for NaN; never -0. */
  const roundI = x => {
    if (x !== x) return 0;
    const t = Math.trunc(x);
    return (Math.abs(x - t) >= 0.5 ? t + (x < 0 ? -1 : 1) : t) + 0;
  };
  const dot3 = (p, q) => (p[0] * q[0] + p[1] * q[1]) + p[2] * q[2];

  // ---- SHA-256 (FIPS 180-4), synchronous: crypto.subtle is async and absent outside secure
  // contexts. A twin of `deeptime_format::sha256`; small inputs only, no streaming. --------------
  const SHA_K = new Uint32Array([
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2]);
  const SHA_H0 = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
  const rotr = (x, n) => (x >>> n) | (x << (32 - n));
  /** The SHA-256 digest (32 bytes) of a Uint8Array. */
  const sha256 = data => {
    const h = Uint32Array.from(SHA_H0);
    const w = new Uint32Array(64);
    const compress = (dv, at) => {
      for (let t = 0; t < 16; t++) w[t] = dv.getUint32(at + 4 * t);
      for (let t = 16; t < 64; t++) {
        const x = w[t - 15], y = w[t - 2];
        const s0 = rotr(x, 7) ^ rotr(x, 18) ^ (x >>> 3);
        const s1 = rotr(y, 17) ^ rotr(y, 19) ^ (y >>> 10);
        w[t] = w[t - 16] + s0 + w[t - 7] + s1;   // wraps mod 2^32 on the store
      }
      let a = h[0], b = h[1], c = h[2], d = h[3], e = h[4], f = h[5], g = h[6], hh = h[7];
      for (let t = 0; t < 64; t++) {
        const S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
        const ch = (e & f) ^ (~e & g);
        const t1 = (hh + S1 + ch + SHA_K[t] + w[t]) | 0;
        const S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
        const maj = (a & b) ^ (a & c) ^ (b & c);
        const t2 = (S0 + maj) | 0;
        hh = g; g = f; f = e; e = (d + t1) | 0; d = c; c = b; b = a; a = (t1 + t2) | 0;
      }
      h[0] += a; h[1] += b; h[2] += c; h[3] += d; h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
    };
    const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
    const n = data.length;
    let at = 0;
    for (; at + 64 <= n; at += 64) compress(dv, at);
    // The padding: 0x80, zeros, then the message length in bits as a big-endian u64.
    const rest = n - at;
    const tail = new Uint8Array(128);
    tail.set(data.subarray(at));
    tail[rest] = 0x80;
    const len = rest < 56 ? 64 : 128;
    const tdv = new DataView(tail.buffer);
    tdv.setUint32(len - 8, Math.floor(n / 0x20000000));   // n·8 / 2^32
    tdv.setUint32(len - 4, (n * 8) >>> 0);
    for (let t = 0; t < len; t += 64) compress(tdv, t);
    const out = new Uint8Array(32);
    const odv = new DataView(out.buffer);
    for (let i = 0; i < 8; i++) odv.setUint32(4 * i, h[i]);
    return out;
  };
  /** A digest as lowercase hex (sha256sum's form). */
  const hexOf = digest => Array.from(digest, b => b.toString(16).padStart(2, '0')).join('');
  /** The bytes of an ArrayBuffer or of any view of one (a Node Buffer included), as a Uint8Array. */
  const bytesOf = x => ArrayBuffer.isView(x) ? new Uint8Array(x.buffer, x.byteOffset, x.byteLength) : new Uint8Array(x);

  // ---- faces ----------------------------------------------------------------------------------
  /** The faces in the format's order (rasters are stored face by face in it). */
  const Face = Object.freeze({ F0: 0, F1: 1, F2: 2, F3: 3, N: 4, S: 5 });
  const FACE_NAMES = Object.freeze(['F0', 'F1', 'F2', 'F3', 'N', 'S']);
  const FACES = Object.freeze([0, 1, 2, 3, 4, 5]);
  /** A side of a face in its own local frame: east is local +u, south +v, clockwise on the map. */
  const FaceSide = Object.freeze({ East: 0, South: 1, West: 2, North: 3 });
  const SIDE_NAMES = Object.freeze(['east', 'south', 'west', 'north']);
  const SIDES = Object.freeze([0, 1, 2, 3]);
  /** Each face's centre in the net, in units of s. */
  const CENTRE_UNITS = [[0, 0], [1, 0], [2, 0], [-1, 0], [0, -1], [0, 1]];
  const AX = [1.0, 0.0, 0.0], AY = [0.0, 1.0, 0.0], AZ = [0.0, 0.0, 1.0];
  const MX = [-1.0, 0.0, 0.0], MY = [0.0, -1.0, 0.0], MZ = [0.0, 0.0, -1.0];
  /** Each face's frame in the cube frame: the unit vector of its centre, then e_u and e_v, the
   *  cube-frame directions of local +u (net +x) and +v (net +z) at the centre. */
  const FRAMES = [
    [AX, AY, MZ], [AY, MX, MZ], [MX, MY, MZ], [MY, AX, MZ], [AZ, AY, AX], [MZ, AY, MX]];
  /** The face whose centre is the largest component of the cube-frame vector p: ties go to x, then
   *  y, then z, and a zero component counts as positive. */
  const faceOfVector = p => {
    const ax = Math.abs(p[0]), ay = Math.abs(p[1]), az = Math.abs(p[2]);
    if (ax >= ay && ax >= az) return p[0] >= 0.0 ? Face.F0 : Face.F2;
    if (ay >= az) return p[1] >= 0.0 ? Face.F1 : Face.F3;
    return p[2] >= 0.0 ? Face.N : Face.S;
  };
  /** The cube world's face side s for a planet `circumference` blocks round at today's scale: the
   *  six faces' 6s² blocks cover the planet's 4πR², rounded to the nearest multiple of
   *  FACE_QUANTUM (4k: 1,024; 16k: 3,840; 32k: 7,424). */
  const faceFor = circumference => {
    const exact = circumference / sqrt(6.0 * Math.PI);
    const q = FACE_QUANTUM;
    return Math.max(roundI(exact / q), 1.0) * q;
  };

  // ---- Iso: a motion of the block grid ----------------------------------------------------------
  /** The point turn of (x, z) by k clockwise quarter turns, on integers. */
  const turnI = (k, x, z) => k === 1 ? [-z, x] : k === 2 ? [-x, -z] : k === 3 ? [z, -x] : [x, z];
  /** A motion of the block grid that keeps blocks on blocks: k clockwise quarter turns (seen from
   *  above with north up) about the origin, then the translation (tx, tz). The wrap engine's
   *  `CellIsometry`, the format's `iso` triple [k, tx, tz]. On block cells the turns are k = 1:
   *  (x, z) -> (-z - 1, x), k = 2: (-x - 1, -z - 1), k = 3: (z, -x - 1); on points k = 1: (-z, x),
   *  k = 2: (-x, -z), k = 3: (z, -x). */
  class Iso {
    /** The motion [k, tx, tz] (k taken mod 4). */
    constructor(k, tx, tz) {
      this.k = remI(k, 4);
      this.tx = tx + 0;   // no -0: Rust's integers have none
      this.tz = tz + 0;
    }
    static translation(dx, dz) { return new Iso(0, dx, dz); }
    /** The half turn about the block corner (a, b). */
    static halfTurnAbout(a, b) { return new Iso(2, 2 * a, 2 * b); }
    /** The quarter turn (k = 1 clockwise, 3 counter-clockwise) about the block corner (a, b). */
    static quarterTurnAbout(k, a, b) {
      const [rx, rz] = turnI(remI(k, 4), a, b);
      return new Iso(k, a - rx, b - rz);
    }
    /** The block cell (x, z) moved. */
    cell(x, z) {
      const [rx, rz] = this.k === 1 ? [-z - 1, x] : this.k === 2 ? [-x - 1, -z - 1]
        : this.k === 3 ? [z, -x - 1] : [x, z];
      return [rx + this.tx, rz + this.tz];
    }
    /** The point (x, z) moved. */
    point(x, z) {
      const [rx, rz] = this.vector(x, z);
      return [rx + this.tx, rz + this.tz];
    }
    /** A direction (dx, dz) turned (no translation). */
    vector(dx, dz) {
      return this.k === 1 ? [-dz, dx] : this.k === 2 ? [-dx, -dz] : this.k === 3 ? [dz, -dx] : [dx, dz];
    }
    /** this ∘ other: first `other`, then this. */
    compose(other) {
      const [rx, rz] = turnI(this.k, other.tx, other.tz);
      return new Iso(this.k + other.k, rx + this.tx, rz + this.tz);
    }
    inverse() {
      const k = (4 - this.k) % 4;
      const [rx, rz] = turnI(k, -this.tx, -this.tz);
      return new Iso(k, rx, rz);
    }
    equals(o) { return this.k === o.k && this.tx === o.tx && this.tz === o.tz; }
    /** [k, tx, tz], the format's `iso`. */
    asArray() { return [this.k, this.tx, this.tz]; }
  }
  Iso.IDENTITY = Object.freeze(new Iso(0, 0, 0));

  // ---- adjacency --------------------------------------------------------------------------------
  /** What lies across one side of a face: the neighbour face, the side of it the step enters
   *  through, and the motion of local cells. For an n x n grid on every face, the cell (i, j) of
   *  this face one step beyond the side (or further out up to a face width) is the neighbour's
   *  cell R_k(i, j) + n·shift, R_k the clockwise cell turn of Iso.cell. */
  class Across {
    constructor(face, side, quarterTurns, shift) {
      this.face = face;
      this.side = side;
      /** Clockwise quarter turns from this face's local frame to the neighbour's. */
      this.quarterTurns = quarterTurns;
      /** The translation in units of the grid's n. */
      this.shift = shift;
    }
    /** The neighbour's cell of this face's cell (i, j) of an n x n face grid. */
    cell(n, i, j) {
      return new Iso(this.quarterTurns, n * this.shift[0], n * this.shift[1]).cell(i, j);
    }
    /** The neighbour's local point of this face's local point (u, v) (continuous, [-1, 1]² on
     *  each face, extended across the side). */
    local(u, v) {
      const r = new Iso(this.quarterTurns, 0, 0);
      const [pu, pv] = r.vector(u + 1.0, v + 1.0);
      return [pu + 2.0 * this.shift[0] - 1.0, pv + 2.0 * this.shift[1] - 1.0];
    }
    /** A direction (du, dv) in this face's local frame, in the neighbour's. */
    vector(du, dv) { return new Iso(this.quarterTurns, 0, 0).vector(du, dv); }
  }
  const across = (face, side, quarterTurns, shift) => Object.freeze(new Across(face, side, quarterTurns, shift));
  /** The face adjacency, ADJACENCY[face][side] in the format's face order and FaceSide order (east,
   *  south, west, north). */
  const ADJACENCY = (() => {
    const { F0, F1, F2, F3, N, S } = Face;
    const { East: E, South: So, West: W, North: No } = FaceSide;
    return Object.freeze([
      [across(F1, W, 0, [-1, 0]), across(S, No, 0, [0, -1]), across(F3, E, 0, [1, 0]), across(N, So, 0, [0, 1])],
      [across(F2, W, 0, [-1, 0]), across(S, E, 1, [2, 0]), across(F0, E, 0, [1, 0]), across(N, E, 3, [1, 1])],
      [across(F3, W, 0, [-1, 0]), across(S, So, 2, [1, 2]), across(F1, E, 0, [1, 0]), across(N, No, 2, [1, 0])],
      [across(F0, W, 0, [-1, 0]), across(S, W, 3, [-1, 1]), across(F2, E, 0, [1, 0]), across(N, W, 1, [0, 0])],
      [across(F1, No, 1, [1, -1]), across(F0, No, 0, [0, -1]), across(F3, No, 3, [0, 0]), across(F2, No, 2, [1, 0])],
      [across(F1, So, 3, [0, 2]), across(F2, So, 2, [1, 2]), across(F3, So, 1, [1, 1]), across(F0, So, 0, [0, 1])],
    ]);
  })();
  /** The cell (i, j) of face `face`'s n x n grid, or the neighbour's cell it is when it lies beyond
   *  one side (at most a face width out): [face, i, j, quarter turns from face's frame]. `null` in
   *  the corner squares beyond two sides at once, where three faces meet at a cube corner. */
  const cellOn = (face, n, i, j) => {
    const outI = i < 0 || i >= n, outJ = j < 0 || j >= n;
    if (!outI && !outJ) return [face, i, j, 0];
    if (outI && outJ) return null;
    if (i < -n || i >= 2 * n || j < -n || j >= 2 * n) return null;
    const side = i >= n ? FaceSide.East : i < 0 ? FaceSide.West : j >= n ? FaceSide.South : FaceSide.North;
    const a = ADJACENCY[face][side];
    const [i2, j2] = a.cell(n, i, j);
    return [a.face, i2, j2, a.quarterTurns];
  };

  // ---- edges, corners and the net ---------------------------------------------------------------
  /** One of the 14 net edges that face the void or the window's ends, and the motion that brings
   *  what lies beyond it back onto the planet (the wrap engine's `WrapEdge`, the manifest's
   *  `edges[]`). `seam` is the Rust `Seam::name` ('east_west', 'ne', 'nw', 'se', 'sw', 'nn', 'ss'),
   *  `motion` the `EdgeMotion::name` ('slide', 'quarter_turn', 'half_turn'), `axis` 'x' or 'z' (the
   *  edge's normal), `beyond` 'min' or 'max'. */
  class NetEdge {
    constructor(f) { Object.assign(this, f); }
    /** How deep the block (x, z) lies beyond the edge, in its band (0 touching the line), or `null`
     *  when it is not beyond the edge within its span. */
    depth(x, z) {
      const [n, t] = this.axis === 'x' ? [x, z] : [z, x];
      if (t < this.span[0] || t >= this.span[1]) return null;
      const d = this.beyond === 'min' ? this.coord - 1 - n : n - this.coord;
      return d >= 0 ? d : null;
    }
    /** How far the point (x, z) lies beyond the edge line (0 on it), or `null` when it is on the
     *  planet's side or outside the span. */
    depthPoint(x, z) {
      const [n, t] = this.axis === 'x' ? [x, z] : [z, x];
      if (t < this.span[0] || t > this.span[1]) return null;
      const d = this.beyond === 'min' ? this.coord - n : n - this.coord;
      return d > 0.0 ? d : null;
    }
  }
  /** The 14 net edges for face side s (Cube.edges). */
  const netEdges = s => {
    const h = s / 2;
    const { F1, F2, F3, N, S } = Face;
    const { East, South, West, North } = FaceSide;
    const e = (face, faceSide, seam, motion, coord, span, iso, pivot, partner) => {
      const axis = (faceSide === East || faceSide === West) ? 'x' : 'z';
      const beyond = (faceSide === East || faceSide === South) ? 'max' : 'min';
      const sideName = faceSide === North ? 'top' : faceSide === South ? 'bottom' : SIDE_NAMES[faceSide];
      return new NetEdge({
        name: `${FACE_NAMES[face]}-${sideName}`, seam, motion, face, faceSide, axis, beyond, coord,
        span, iso, pivot, partner, owner: motion === 'slide' ? 'toroidal_world' : 'wrap_engine',
      });
    };
    const q = 'quarter_turn', ht = 'half_turn';
    const halfN = Iso.halfTurnAbout(s, -s), halfS = Iso.halfTurnAbout(s, s);
    return [
      // 0, 1: the slide.
      e(F3, West, 'east_west', 'slide', -3 * h, [-h, h], Iso.translation(4 * s, 0), null, 1),
      e(F2, East, 'east_west', 'slide', 5 * h, [-h, h], Iso.translation(-4 * s, 0), null, 0),
      // 2, 3: NE, a quarter turn about (s/2, -s/2), clockwise from N to F1.
      e(N, East, 'ne', q, h, [-3 * h, -h], Iso.quarterTurnAbout(1, h, -h), [h, -h], 3),
      e(F1, North, 'ne', q, -h, [h, 3 * h], Iso.quarterTurnAbout(3, h, -h), [h, -h], 2),
      // 4, 5: NW, about (-s/2, -s/2), counter-clockwise from N to F3.
      e(N, West, 'nw', q, -h, [-3 * h, -h], Iso.quarterTurnAbout(3, -h, -h), [-h, -h], 5),
      e(F3, North, 'nw', q, -h, [-3 * h, -h], Iso.quarterTurnAbout(1, -h, -h), [-h, -h], 4),
      // 6, 7: NN, the half turn about (s, -s).
      e(N, North, 'nn', ht, -3 * h, [-h, h], halfN, [s, -s], 7),
      e(F2, North, 'nn', ht, -h, [3 * h, 5 * h], halfN, [s, -s], 6),
      // 8, 9: SE, about (s/2, s/2), counter-clockwise from S to F1.
      e(S, East, 'se', q, h, [h, 3 * h], Iso.quarterTurnAbout(3, h, h), [h, h], 9),
      e(F1, South, 'se', q, h, [h, 3 * h], Iso.quarterTurnAbout(1, h, h), [h, h], 8),
      // 10, 11: SW, about (-s/2, s/2), clockwise from S to F3.
      e(S, West, 'sw', q, -h, [h, 3 * h], Iso.quarterTurnAbout(1, -h, h), [-h, h], 11),
      e(F3, South, 'sw', q, h, [-3 * h, -h], Iso.quarterTurnAbout(3, -h, h), [-h, h], 10),
      // 12, 13: SS, the half turn about (s, s).
      e(S, South, 'ss', ht, 3 * h, [-h, h], halfS, [s, s], 13),
      e(F2, South, 'ss', ht, h, [3 * h, 5 * h], halfS, [s, s], 12),
    ];
  };

  /** The cube world's net for face side s (a multiple of FACE_QUANTUM in real worlds; any even side
   *  in tests). Net geometry only: where the cube sits on the planet is a DTCubeTurn. +x is east
   *  and -z north, as everywhere in Minecraft. The window is x in [-3s/2, 5s/2), z in [-3s/2, 3s/2);
   *  x wraps with period 4s (Toroidal World's slide).
   *
   *  Face   x range          z range           centre   centre in the cube frame
   *  F3     [-3s/2, -s/2)    [-s/2, s/2)       (-s, 0)  -Y
   *  F0     [-s/2, s/2)      [-s/2, s/2)       (0, 0)   +X
   *  F1     [s/2, 3s/2)      [-s/2, s/2)       (s, 0)   +Y
   *  F2     [3s/2, 5s/2)     [-s/2, s/2)       (2s, 0)  -X
   *  N      [-s/2, s/2)      [-3s/2, -s/2)     (0, -s)  +Z
   *  S      [-s/2, s/2)      [s/2, 3s/2)       (0, s)   -Z
   *  (N and S name the net's top and bottom faces, not the poles.)
   *
   *  Local coordinates: block cell (i, j) in [0, s)² is the net block (x0 + i, z0 + j) from the
   *  face's origin (its north-west corner), the continuous local point (u, v) in [-1, 1]² is
   *  ((x - cx)/(s/2), (z - cz)/(s/2)) from its centre. */
  class Cube {
    constructor(s) {
      if (!Number.isInteger(s) || s < 2 || s % 2 !== 0) throw new Error(`cube face side ${s} must be even`);
      /** s, blocks. */
      this.s = s;
      /** The east-west period 4s, blocks (Toroidal World's loop). */
      this.period = 4 * s;
      /** The 14 net edges: the two slide edges, then the turning edges in the wrap engine's order
       *  (N-east, F1-top, N-west, F3-top, N-top, F2-top, S-east, F1-bottom, S-west, F3-bottom,
       *  S-bottom, F2-bottom). */
      this.edges = netEdges(s);
    }
    /** The net for a planet `circumference` blocks round (faceFor). */
    static forCircumference(circumference) { return new Cube(faceFor(circumference)); }
    /** Whether s is a multiple of FACE_QUANTUM (a real world's face). */
    quantised() { return this.s % FACE_QUANTUM === 0; }
    /** The radius of the sphere the six faces stand for at equal area, blocks: 6s² = 4πR². */
    radius() { return this.s * sqrt(1.5 / Math.PI); } // ledger: SCL-078 (6 s^2 = 4 pi R^2)
    /** The window: x in [-3s/2, 5s/2), z in [-3s/2, 3s/2]. */
    window() {
      const h = this.s / 2.0;
      return { xMin: -3.0 * h, xMax: 5.0 * h, zMin: -3.0 * h, zMax: 3.0 * h };
    }
    /** The net block of the face's local cell (0, 0) (its north-west corner). */
    faceOrigin(face) {
      const [cx, cz] = CENTRE_UNITS[face];
      return [cx * this.s - this.s / 2, cz * this.s - this.s / 2];
    }
    /** The face's centre, a block corner. */
    faceCentre(face) {
      const [cx, cz] = CENTRE_UNITS[face];
      return [cx * this.s, cz * this.s];
    }
    /** The block x wrapped into the window [-3s/2, 5s/2). */
    wrapX(x) {
      const lo = -3 * this.s / 2;
      return remI(x - lo, 4 * this.s) + lo;
    }
    /** The point x wrapped into the window [-3s/2, 5s/2). */
    wrapXPoint(x) {
      const p = 4 * this.s;
      const lo = -1.5 * this.s;
      const r = (x - lo) % p;
      const t = r < 0 ? r + p : r;   // f64::rem_euclid
      return (t >= p ? 0.0 : t) + lo;
    }
    /** The face and local cell [face, i, j] of the block (x, z) (x wraps), or `null` off the planet. */
    faceOfBlock(x, z) {
      x = this.wrapX(x);
      const h = this.s / 2;
      // Columns of s from x = -3s/2: 0 F3, 1 F0, 2 F1, 3 F2.
      const col = divI(x + 3 * h, this.s);
      let face;
      if (z >= -h && z < h) face = [Face.F3, Face.F0, Face.F1, Face.F2][col];
      else if (col === 1 && z >= -3 * h && z < -h) face = Face.N;
      else if (col === 1 && z >= h && z < 3 * h) face = Face.S;
      else return null;
      const [ox, oz] = this.faceOrigin(face);
      return [face, x - ox, z - oz];
    }
    /** The net block of a face's local cell (i, j). */
    blockOfCell(face, i, j) {
      const [ox, oz] = this.faceOrigin(face);
      return [ox + i, oz + j];
    }
    /** The face and local point [face, u, v] in [-1, 1]² of the point (x, z) (x wraps), or `null`
     *  off the planet. Faces are closed squares here: a point on a line two faces share is given to
     *  the first in the format's order (both faces map it to the same planet point). */
    localOfPoint(x, z) {
      x = this.wrapXPoint(x);
      const h = this.s / 2.0;
      for (const face of FACES) {
        const [cx, cz] = this.faceCentre(face);
        const u = (x - cx) / h, v = (z - cz) / h;
        if (u >= -1.0 && u <= 1.0 && v >= -1.0 && v <= 1.0) return [face, u, v];
      }
      return null;
    }
    /** The net point [x, z] of a face's local point (u, v). */
    pointOfLocal(face, u, v) {
      const [cx, cz] = this.faceCentre(face);
      const h = this.s / 2.0;
      return [cx + u * h, cz + v * h];
    }
    /** The block (x, z) brought onto the planet: `{kind: 'face', face, i, j}` (on the planet, after
     *  the slide), `{kind: 'outside', edge, x, z, iso}` (beyond a net edge, in its band: the edge's
     *  index in `edges`, the planet block it shows in the window, and the motion from the queried
     *  block to it) or `{kind: 'ocean'}` (in no band, or more than a face width out: past the
     *  corners of the net). Where two bands meet (the void squares by F0's corners) the nearer
     *  edge's motion applies, ties to the earlier edge. */
    fold(x, z) {
      const xw = this.wrapX(x);
      const fb = this.faceOfBlock(xw, z);
      if (fb !== null) return { kind: 'face', face: fb[0], i: fb[1], j: fb[2] };
      let best = -1, bestD = 0;
      for (let k = 2; k < this.edges.length; k++) {
        // Bands a face width deep, the nearer edge first (ledger: SCL-108).
        const d = this.edges[k].depth(xw, z);
        if (d !== null && d < this.s && (best < 0 || d < bestD)) { best = k; bestD = d; }
      }
      if (best < 0) return { kind: 'ocean' };
      const iso = this.edges[best].iso.compose(Iso.translation(xw - x, 0));
      const [px, pz] = iso.cell(x, z);
      if (this.faceOfBlock(px, pz) !== null && px === this.wrapX(px)) {
        return { kind: 'outside', edge: best, x: px, z: pz, iso };
      }
      return { kind: 'ocean' };
    }
    /** The point (x, z) brought onto the planet, as `fold` for blocks: x wraps; a point off the
     *  planet takes the motion of the nearest net edge whose band (a face width deep) holds it.
     *  `{face, u, v, x, z, iso, edge}` (`edge` null on the planet), or `null` past the net's corners. */
    foldPoint(x, z) {
      const xw = this.wrapXPoint(x);
      const slide = roundI(xw - x);
      const on = this.localOfPoint(xw, z);
      if (on !== null) {
        return { face: on[0], u: on[1], v: on[2], x: xw, z, iso: Iso.translation(slide, 0), edge: null };
      }
      let best = -1, bestD = 0;
      for (let k = 2; k < this.edges.length; k++) {
        const d = this.edges[k].depthPoint(xw, z);
        if (d !== null && d <= this.s && (best < 0 || d < bestD)) { best = k; bestD = d; }
      }
      if (best < 0) return null;
      const iso = this.edges[best].iso.compose(Iso.translation(slide, 0));
      const [px, pz] = iso.point(x, z);
      const to = this.localOfPoint(px, pz);
      if (to === null) return null;
      return { face: to[0], u: to[1], v: to[2], x: this.wrapXPoint(px), z: pz, iso, edge: best };
    }
    /** The eight cube corners: n0 .. n3 round face N (counter-clockwise from (+1, +1, +1)), then
     *  s0 .. s3 round face S, each `{name, faces, cube, images}` with its cube-frame direction
     *  (±1, ±1, ±1) and every block corner of the closed window that is this corner. */
    corners() {
      const h = this.s / 2;
      const { F0, F1, F2, F3, N, S } = Face;
      const c = (name, faces, cube, images) => ({ name, faces, cube, images });
      return [
        c('n0', [F0, F1, N], [1, 1, 1], [[h, -h]]),
        c('n1', [F1, F2, N], [-1, 1, 1], [[h, -3 * h], [3 * h, -h]]),
        c('n2', [F2, F3, N], [-1, -1, 1], [[-h, -3 * h], [-3 * h, -h], [5 * h, -h]]),
        c('n3', [F3, F0, N], [1, -1, 1], [[-h, -h]]),
        c('s0', [F0, F1, S], [1, 1, -1], [[h, h]]),
        c('s1', [F1, F2, S], [-1, 1, -1], [[h, 3 * h], [3 * h, h]]),
        c('s2', [F2, F3, S], [-1, -1, -1], [[-h, 3 * h], [-3 * h, h], [5 * h, h]]),
        c('s3', [F3, F0, S], [1, -1, -1], [[-h, h]]),
      ];
    }
  }
  Cube.FACE_QUANTUM = FACE_QUANTUM;
  Cube.faceFor = faceFor;
  Cube.Face = Face;
  Cube.FACE_NAMES = FACE_NAMES;
  Cube.FACES = FACES;
  Cube.FaceSide = FaceSide;
  Cube.SIDE_NAMES = SIDE_NAMES;
  Cube.SIDES = SIDES;
  Cube.Iso = Iso;
  Cube.Across = Across;
  Cube.ADJACENCY = ADJACENCY;
  Cube.cellOn = cellOn;
  Cube.faceOfVector = faceOfVector;
  /** A face's [centre, e_u, e_v] in the cube frame. */
  Cube.frame = face => FRAMES[face];
  /** The face's centre in the net, in units of s. */
  Cube.centreUnits = face => CENTRE_UNITS[face];

  // ---- the turn -------------------------------------------------------------------------------
  /** Where the cube sits on the planet: a rotation, planet frame to cube frame (the manifest's
   *  `projection.rotation`, 3 x 3, row-major). The planet frame is the format's: +x the point
   *  (0°, 0°), +y (0°, 90° E), +z the north pole. The identity is the polar-axis cube at lambda0 = 0
   *  (the simulation's cube). */
  class CubeTurn {
    constructor(m) { this.m = m; }
    /** A turn from its rows (planet to cube), checked to be a rotation: orthonormal to 1e-12 and of
     *  determinant +1. Throws on a reflection or a non-orthonormal matrix. */
    static fromRows(m) {
      for (let i = 0; i < 3; i++) {
        for (let j = 0; j < 3; j++) {
          const a = m[i], b = m[j];
          const d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
          const want = i === j ? 1.0 : 0.0;
          if (Math.abs(d - want) > 1e-12 || !Number.isFinite(d)) {
            throw new Error(`cube rotation rows ${i}, ${j}: dot ${d}, want ${want}`);
          }
        }
      }
      const det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
      if (det < 0.0) throw new Error(`cube rotation has determinant ${det}: a reflection`);
      return new CubeTurn([[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]]);
    }
    /** The polar-axis turn by lambda0 (degrees): the planet point (0°, lambda0) at F0's centre, the
     *  north pole at N's. */
    static polar(lambda0Deg) {
      const l = M.toRadians(lambda0Deg);
      const s = M.sin(l), c = M.cos(l);
      return new CubeTurn([[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]]);
    }
    /** The rows, planet to cube. */
    rows() { return this.m; }
    /** A planet-frame vector in the cube frame (Q·p). */
    toCube(p) {
      const m = this.m;
      return [
        (m[0][0] * p[0] + m[0][1] * p[1]) + m[0][2] * p[2],
        (m[1][0] * p[0] + m[1][1] * p[1]) + m[1][2] * p[2],
        (m[2][0] * p[0] + m[2][1] * p[1]) + m[2][2] * p[2],
      ];
    }
    /** A cube-frame vector in the planet frame (Qᵀ·c). */
    toPlanet(c) {
      const m = this.m;
      return [
        (m[0][0] * c[0] + m[1][0] * c[1]) + m[2][0] * c[2],
        (m[0][1] * c[0] + m[1][1] * c[1]) + m[2][1] * c[2],
        (m[0][2] * c[0] + m[1][2] * c[1]) + m[2][2] * c[2],
      ];
    }
  }
  CubeTurn.IDENTITY = new CubeTurn([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);

  // ---- the map table ----------------------------------------------------------------------------
  const MAGIC = 'DTCM';
  const LAYOUT_VERSION = 1;
  const HEADER_BYTES = 112;

  /** floor(x) clamped to [0, last] (0 for NaN), without Math.floor: truncate, then step down one if
   *  that is above x (a twin of Rust's `floor_clamped`). */
  const floorClamped = (x, last) => {
    let i = Math.trunc(x);
    if (i !== i) i = 0;   // NaN as i64
    if (i > x) i -= 1;
    return (i < 0 ? 0 : (i > last ? last : i)) + 0;
  };
  /** The Catmull-Rom weights of four nodes at t in [0, 1]: exactly (0, 1, 0, 0) at t = 0 and
   *  (0, 0, 1, 0) at t = 1, so the bicubic is exact at its nodes (ledger: SCL-105). */
  const weights = t => {
    const t2 = t * t;
    const t3 = t2 * t;
    return [
      0.5 * ((2.0 * t2 - t) - t3),
      0.5 * ((2.0 - 5.0 * t2) + 3.0 * t3),
      0.5 * ((t + 4.0 * t2) - 3.0 * t3),
      0.5 * (t3 - t2),
    ];
  };
  /** The weights' derivatives along t. */
  const dweights = t => {
    const t2 = t * t;
    return [
      0.5 * ((4.0 * t - 1.0) - 3.0 * t2),
      0.5 * (9.0 * t2 - 10.0 * t),
      0.5 * ((1.0 + 8.0 * t) - 9.0 * t2),
      0.5 * (3.0 * t2 - 2.0 * t),
    ];
  };
  /** Component c of Σ w_k p_k over four (a, b) nodes stored flat from node b, summed from k = 0. */
  const mix = (w, p, b, c) =>
    ((w[0] * p[2 * b + c] + w[1] * p[2 * (b + 1) + c]) + w[2] * p[2 * (b + 2) + c]) + w[3] * p[2 * (b + 3) + c];
  // The bicubic's four row results (and their derivatives): scratch, as value/valueD never nest.
  const ROWS = new Float64Array(8), DROWS = new Float64Array(8);

  /** A padded node grid over [-1, n + 1]², node (i, j) at node index (j + 1)·side + (i + 1) of the
   *  flat (a, b) array, so the four nodes a bicubic row reads along i are adjacent. */
  class Grid {
    constructor(n) {
      this.n = n;
      this.side = n + 3;
      this.nodes = new Float64Array(2 * this.side * this.side).fill(NaN);
    }
    idx(i, j) { return (j + 1) * this.side + (i + 1); }
    get(i, j) { const k = 2 * this.idx(i, j); return [this.nodes[k], this.nodes[k + 1]]; }
    set(i, j, p) { const k = 2 * this.idx(i, j); this.nodes[k] = p[0]; this.nodes[k + 1] = p[1]; }
    /** The bicubic at grid coordinates (x, y), node units: the cubic along x on each of the four
     *  rows, then along y. */
    value(x, y) {
      const i0 = floorClamped(x, this.n - 1), tu = x - i0;
      const j0 = floorClamped(y, this.n - 1), tv = y - j0;
      const wu = weights(tu);
      for (let dj = 0; dj < 4; dj++) {
        const b = (j0 + dj) * this.side + i0;
        ROWS[2 * dj] = mix(wu, this.nodes, b, 0);
        ROWS[2 * dj + 1] = mix(wu, this.nodes, b, 1);
      }
      const wv = weights(tv);
      return [mix(wv, ROWS, 0, 0), mix(wv, ROWS, 0, 1)];
    }
    /** The bicubic at (x, y) and its derivatives along x and y, per node unit. */
    valueD(x, y) {
      const i0 = floorClamped(x, this.n - 1), tu = x - i0;
      const j0 = floorClamped(y, this.n - 1), tv = y - j0;
      const wu = weights(tu), dwu = dweights(tu);
      const wv = weights(tv), dwv = dweights(tv);
      for (let dj = 0; dj < 4; dj++) {
        const b = (j0 + dj) * this.side + i0;
        ROWS[2 * dj] = mix(wu, this.nodes, b, 0);
        ROWS[2 * dj + 1] = mix(wu, this.nodes, b, 1);
        DROWS[2 * dj] = mix(dwu, this.nodes, b, 0);
        DROWS[2 * dj + 1] = mix(dwu, this.nodes, b, 1);
      }
      return {
        val: [mix(wv, ROWS, 0, 0), mix(wv, ROWS, 0, 1)],
        du: [mix(wv, DROWS, 0, 0), mix(wv, DROWS, 0, 1)],
        dv: [mix(dwv, ROWS, 0, 0), mix(dwv, ROWS, 0, 1)],
      };
    }
  }

  /** The ghosts beyond u = 1 and v = 1 and past the corner (cube_map.py `padded`, in its order;
   *  ledger: SCL-105). */
  const padEdgeAndCorner = (g, m) => {
    // Beyond u = 1 (j < m): the mirror of node (m - 1, j) across the edge's plane.
    for (let j = 0; j < m; j++) {
      const p = g.get(m - 1, j);
      g.set(m + 1, j, [1.0 / p[0], p[1] / p[0]]);
    }
    // Beyond v = 1 (i < m): the swap of the above.
    for (let i = 0; i < m; i++) {
      const p = g.get(m + 1, i);
      g.set(i, m + 1, [p[1], p[0]]);
    }
    // Past the corner: linear along the edge, its swap, and linear along the diagonal.
    const pmm = g.get(m, m), pm1 = g.get(m, m - 1), pd = g.get(m - 1, m - 1);
    const e = [2.0 * pmm[0] - pm1[0], 2.0 * pmm[1] - pm1[1]];
    g.set(m, m + 1, e);
    g.set(m + 1, m, [e[1], e[0]]);
    g.set(m + 1, m + 1, [2.0 * pmm[0] - pd[0], 2.0 * pmm[1] - pd[1]]);
  };

  /** A face-frame vector (along the centre, e_u, e_v) in the cube frame. */
  const toCubeFrame = (face, w) => {
    const [c, eu, ev] = FRAMES[face];
    return [
      (c[0] * w[0] + eu[0] * w[1]) + ev[0] * w[2],
      (c[1] * w[0] + eu[1] * w[1]) + ev[1] * w[2],
      (c[2] * w[0] + eu[2] * w[1]) + ev[2] * w[2],
    ];
  };

  /** The printable-ASCII text of a NUL-padded header field. */
  const asciiField = (b, what) => {
    let end = b.indexOf(0);
    if (end < 0) end = b.length;
    let s = '';
    for (let i = 0; i < end; i++) {
      if (b[i] < 0x21 || b[i] > 0x7e) throw new Error(`cube map table: the ${what} is not printable ASCII`);
      s += String.fromCharCode(b[i]);
    }
    return s;
  };

  /** A cube map table, ready to evaluate (the slot of the plan §1.6): a face's local point (u, v)
   *  in [-1, 1]² to the unit sphere and back. The table holds one eighth of a face, the triangle
   *  0 <= v <= u <= 1, as the planet point in gnomonic coordinates of the face frame,
   *  (a, b) = (y/x, z/x) with x along the face centre; the rest of the face is sign flips and a
   *  swap. The forward is a Catmull-Rom bicubic over the nodes, padded by ghost nodes, with a
   *  corner patch for the conformal table (ledger: SCL-104); the inverse starts from a second
   *  table (bilinear) and takes NEWTON_STEPS Newton steps on the forward. Layout: FORMAT.md "The
   *  cube map table" and tools/cube-map/cube_map.py. Build one with `load`, `loadChecked` or
   *  `fetch`. */
  class CubeMap {
    /** A table from its file's bytes (an ArrayBuffer or a view of one). Throws on a bad file. */
    static load(buffer) {
      const bytes = bytesOf(buffer);
      const bad = why => new Error(`cube map table: ${why}`);
      if (bytes.length < HEADER_BYTES || String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]) !== MAGIC) {
        throw bad('not a cube map table (magic DTCM)');
      }
      const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      const u32 = at => dv.getUint32(at, true);
      const f64 = at => dv.getFloat64(at, true);
      const version = u32(4);
      if (version !== LAYOUT_VERSION) throw bad(`layout version ${version}, this reader reads ${LAYOUT_VERSION}`);
      const m = u32(8), q = u32(12);
      const tau = f64(16);
      const flowSteps = u32(24);
      const k = u32(28), f = u32(32);
      if (!(m >= 2 && m <= 65536) || !(q >= 1 && q <= 65536)) throw bad(`m ${m}, q ${q}`);
      if ((k === 0) !== (f === 0) || k >= m || f > 4096) throw bad(`corner patch K ${k}, F ${f} with m ${m}`);
      const name = asciiField(bytes.subarray(40, 72), 'name');
      const commit = asciiField(bytes.subarray(72, 112), 'generator commit');
      const nf = (m + 1) * (m + 2) / 2;
      const ni = (q + 1) * (q + 2) / 2;
      const ps = k > 0 ? k * f + 3 : 0;
      const np = ps * (ps + 1) / 2;
      const want = HEADER_BYTES + 16 * (nf + ni + np);
      if (bytes.length !== want) throw bad(`${bytes.length} bytes, want ${want}`);

      // A triangle stored row by row (for i, for j <= i), (a, b) pairs, as a flat array: the pair
      // (i, j) at 2·(i(i+1)/2 + j).
      const triangle = (at, rows) => {
        const out = new Float64Array(rows * (rows + 1));
        for (let t = 0; t < out.length; t++) out[t] = f64(at + 8 * t);
        return out;
      };
      const pairAt = (tri, i, j) => { const o = i * (i + 1) + 2 * j; return [tri[o], tri[o + 1]]; };
      const finite = nodes => nodes.every(Number.isFinite);

      // The coarse table and its ghosts (cube_map.py `padded`).
      const tri = triangle(HEADER_BYTES, m + 1);
      const g = new Grid(m);
      for (let i = 0; i <= m; i++) {
        for (let j = 0; j <= m; j++) {
          const p = pairAt(tri, Math.max(i, j), Math.min(i, j));
          g.set(i, j, j <= i ? p : [p[1], p[0]]);
        }
      }
      padEdgeAndCorner(g, m);
      // Beyond u = 0: (-a, b) of column 1, rows j in [0, m + 1]; beyond v = 0: (a, -b) of row 1,
      // i in [-1, m + 1].
      for (let j = 0; j <= m + 1; j++) {
        const p = g.get(1, j);
        g.set(-1, j, [-p[0], p[1]]);
      }
      for (let i = -1; i <= m + 1; i++) {
        const p = g.get(i, 1);
        g.set(i, -1, [p[0], -p[1]]);
      }
      if (!finite(g.nodes)) throw bad('forward nodes not finite');

      // The inverse table, as a square: node (kk, ll) at kk·(q + 1) + ll.
      const itri = triangle(HEADER_BYTES + 16 * nf, q + 1);
      const inv = new Float64Array(2 * (q + 1) * (q + 1));
      for (let kk = 0; kk <= q; kk++) {
        for (let ll = 0; ll <= q; ll++) {
          const p = pairAt(itri, Math.max(kk, ll), Math.min(kk, ll));
          const o = 2 * (kk * (q + 1) + ll);
          inv[o] = ll <= kk ? p[0] : p[1];
          inv[o + 1] = ll <= kk ? p[1] : p[0];
        }
      }

      // The corner patch: every node of [-1, n + 1]² stored, the upper triangle by the swap.
      let patch = null;
      if (k > 0) {
        const n = k * f;
        const ptri = triangle(HEADER_BYTES + 16 * (nf + ni), ps);
        const pg = new Grid(n);
        for (let i = -1; i <= n + 1; i++) {
          for (let j = -1; j <= n + 1; j++) {
            const p = pairAt(ptri, Math.max(i, j) + 1, Math.min(i, j) + 1);
            pg.set(i, j, j <= i ? p : [p[1], p[0]]);
          }
        }
        if (!finite(pg.nodes)) throw bad('corner patch nodes not finite');
        patch = { k, f, u0: 1.0 - k / m, mf: m * f, grid: pg };
      }
      return new CubeMap({ name, commit, tau, flowSteps, m, q, bytes, coarse: g, patch, inv });
    }

    /** A table from its bytes, checked against the sha256 a manifest names (lowercase hex). Throws
     *  on a different hash. */
    static loadChecked(buffer, sha256Hex) {
      const map = CubeMap.load(buffer);
      const want = String(sha256Hex).trim().toLowerCase();
      if (map.sha256Hex() !== want) {
        throw new Error(`cube map table ${map.name}: sha256 ${map.sha256Hex()}, the manifest names ${String(sha256Hex).trim()}`);
      }
      return map;
    }

    /** Fetches a table (`url`, a file beside the page) and checks it against the manifest's sha256:
     *  a Promise of the CubeMap. */
    static fetch(url, sha256Hex) {
      if (typeof sha256Hex !== 'string') return Promise.reject(new Error('cube map table: the manifest\'s sha256 is required'));
      return root.fetch(url).then(r => {
        if (!r.ok) throw new Error(`cube map table: ${url}: HTTP ${r.status}`);
        return r.arrayBuffer();
      }).then(b => CubeMap.loadChecked(b, sha256Hex));
    }

    /** The file name a bundle gives a table (`cube-map-conformal-n513.bin`). */
    static fileName(name) { return `cube-map-${name}.bin`; }

    constructor(t) {
      /** The table's name (`conformal-n513`, `moser-t1-n513`). */
      this.name = t.name;
      /** The generator's commit. */
      this.commit = t.commit;
      /** tau: 0 the conformal cube, 1 Moser's equal-area cube. */
      this.tau = t.tau;
      /** The RK4 steps of Moser's flow the generator took (0 for the conformal cube). */
      this.flowSteps = t.flowSteps;
      /** m: forward cells along the triangle's side (the face has 2m + 1 nodes a side). */
      this.m = t.m;
      this.mf = t.m;
      /** q: the inverse table's cells along the gnomonic triangle's side. */
      this.q = t.q;
      this.bytes = t.bytes;
      this.coarse = t.coarse;
      /** The corner patch ({k, f, u0, mf, grid}), or null. */
      this.patch = t.patch;
      this.inv = t.inv;
      this.sha = null;
    }

    /** The file's sha256, lowercase hex (computed on first use). */
    sha256Hex() {
      if (this.sha === null) this.sha = hexOf(sha256(this.bytes));
      return this.sha;
    }

    /** (a, b) on the triangle at (u, v), 0 <= v <= u <= 1, without the edge's exact a = 1 (a Newton
     *  iterate may step a hair outside the triangle). */
    triangleValueRaw(u, v) {
      const p = this.patch;
      if (p === null) return this.coarse.value(u * this.mf, v * this.mf);
      // Coarse cells inward from the patch's edge (v is the smaller coordinate).
      const w = (v - p.u0) * this.mf;
      if (w <= 0.0) return this.coarse.value(u * this.mf, v * this.mf);
      const fv = p.grid.value((u - p.u0) * p.mf, (v - p.u0) * p.mf);
      if (w >= 1.0) return fv;
      const c = this.coarse.value(u * this.mf, v * this.mf);
      const s = w * w * (3.0 - 2.0 * w); // ledger: SCL-106
      return [c[0] + s * (fv[0] - c[0]), c[1] + s * (fv[1] - c[1])];
    }

    /** (a, b) and its derivatives along u and v on the triangle, as `triangleValueRaw`:
     *  `{val, du, dv}`. */
    triangleRaw(u, v) {
      const scaled = (e, k) => ({
        val: e.val,
        du: [e.du[0] * k, e.du[1] * k],
        dv: [e.dv[0] * k, e.dv[1] * k],
      });
      const coarse = () => scaled(this.coarse.valueD(u * this.mf, v * this.mf), this.mf);
      const p = this.patch;
      if (p === null) return coarse();
      const w = (v - p.u0) * this.mf;
      if (w <= 0.0) return coarse();
      const fe = scaled(p.grid.valueD((u - p.u0) * p.mf, (v - p.u0) * p.mf), p.mf);
      if (w >= 1.0) return fe;
      const ce = coarse();
      const s = w * w * (3.0 - 2.0 * w); // ledger: SCL-106
      const ds = 6.0 * w * (1.0 - w) * this.mf;
      const blend = (c, f) => c + s * (f - c);
      return {
        val: [blend(ce.val[0], fe.val[0]), blend(ce.val[1], fe.val[1])],
        du: [blend(ce.du[0], fe.du[0]), blend(ce.du[1], fe.du[1])],
        dv: [
          blend(ce.dv[0], fe.dv[0]) + (fe.val[0] - ce.val[0]) * ds,
          blend(ce.dv[1], fe.dv[1]) + (fe.val[1] - ce.val[1]) * ds,
        ],
      };
    }

    /** (a, b) on the triangle at (u, v), with a = 1 exactly on the edge u = 1. */
    triangle(u, v) {
      const ab = this.triangleValueRaw(u, v);
      if (u === 1.0) ab[0] = 1.0;
      return ab;
    }

    /** The unit vector, cube frame, of face `face`'s local point (u, v) in [-1, 1]². */
    forward(face, u, v) {
      const au = Math.abs(u), av = Math.abs(v);
      const swap = av > au;
      const tu = swap ? av : au, tv = swap ? au : av;
      const ab = this.triangle(tu, tv);
      const a = swap ? ab[1] : ab[0];
      const b = swap ? ab[0] : ab[1];
      const ga = u < 0.0 ? -a : a;
      const gb = v < 0.0 ? -b : b;
      // One division: the reciprocal of r, then products.
      const ir = 1.0 / sqrt((1.0 + ga * ga) + gb * gb);
      return toCubeFrame(face, [ir, ga * ir, gb * ir]);
    }

    /** The unit vector of (u, v) and its derivatives per unit of u and v, cube frame:
     *  `{p, du, dv}`. */
    jacobian(face, u, v) {
      const au = Math.abs(u), av = Math.abs(v);
      const swap = av > au;
      const tu = swap ? av : au, tv = swap ? au : av;
      const e = this.triangleRaw(tu, tv);
      if (tu === 1.0) e.val[0] = 1.0;
      const su = u < 0.0 ? -1.0 : 1.0;
      const sv = v < 0.0 ? -1.0 : 1.0;
      const sw = su * sv;
      // Through the fold: |u|, |v| and the swap.
      let a, b, daDu, daDv, dbDu, dbDv;
      if (swap) {
        a = e.val[1]; b = e.val[0];
        daDu = e.dv[1]; daDv = sw * e.du[1]; dbDu = sw * e.dv[0]; dbDv = e.du[0];
      } else {
        a = e.val[0]; b = e.val[1];
        daDu = e.du[0]; daDv = sw * e.dv[0]; dbDu = sw * e.du[1]; dbDv = e.dv[1];
      }
      const ga = su * a;
      const gb = sv * b;
      const ir = 1.0 / sqrt((1.0 + ga * ga) + gb * gb);
      const n = [ir, ga * ir, gb * ir];
      // d(g/r) = (dg - n (n·dg)) / r, with n·dg = (ga dga + gb dgb) / r.
      const dn = (dga, dgb) => {
        const k = (ga * dga + gb * dgb) * ir;
        return [-(n[0] * k) * ir, (dga - n[1] * k) * ir, (dgb - n[2] * k) * ir];
      };
      return {
        p: toCubeFrame(face, n),
        du: toCubeFrame(face, dn(daDu, dbDu)),
        dv: toCubeFrame(face, dn(daDv, dbDv)),
      };
    }

    /** The inverse table's bilinear start [u, v] at (a, b) on the gnomonic triangle. */
    inverseStart(a, b) {
      const q = this.q;
      const x = a * q, y = b * q;
      const k = floorClamped(x, q - 1), l = floorClamped(y, q - 1);
      const tx = x - k, ty = y - l;
      const row = q + 1, inv = this.inv;
      const o00 = 2 * (k * row + l), o10 = 2 * ((k + 1) * row + l);
      const o01 = 2 * (k * row + l + 1), o11 = 2 * ((k + 1) * row + l + 1);
      const u0 = inv[o00] + tx * (inv[o10] - inv[o00]);
      const u1 = inv[o01] + tx * (inv[o11] - inv[o01]);
      const w0 = inv[o00 + 1] + tx * (inv[o10 + 1] - inv[o00 + 1]);
      const w1 = inv[o01 + 1] + tx * (inv[o11 + 1] - inv[o01 + 1]);
      return [u0 + ty * (u1 - u0), w0 + ty * (w1 - w0)];
    }

    /** (u, v) on the triangle whose forward is (a, b), 0 <= b <= a <= 1: the inverse table's start,
     *  NEWTON_STEPS Newton steps, then clamped into the triangle and put back on its lines (a = 1
     *  gives u = 1, b = 0 gives v = 0, a = b gives v = u), in this order. */
    triangleInverse(a, b) {
      let [u, v] = this.inverseStart(a, b);
      for (let i = 0; i < NEWTON_STEPS; i++) {
        const e = this.triangleRaw(u, v);
        const ra = a - e.val[0], rb = b - e.val[1];
        const det = e.du[0] * e.dv[1] - e.dv[0] * e.du[1];
        const nu = u + (e.dv[1] * ra - e.dv[0] * rb) / det;
        const nv = v + (e.du[0] * rb - e.du[1] * ra) / det;
        u = nu;
        v = nv;
      }
      if (u > 1.0) u = 1.0;
      if (u < 0.0) u = 0.0;
      if (v > u) v = u;
      if (v < 0.0) v = 0.0;
      if (a === 1.0) u = 1.0;
      if (b === 0.0) v = 0.0;
      if (a === b) v = u;
      return [u, v];
    }

    /** The face and local point [face, u, v] in [-1, 1]² of the cube-frame vector p (any length >
     *  0). The face is `faceOfVector`'s (ties to x, then y, then z). */
    inverse(p) {
      const face = faceOfVector(p);
      const [c, eu, ev] = FRAMES[face];
      const x = (c[0] * p[0] + c[1] * p[1]) + c[2] * p[2];
      const y = (eu[0] * p[0] + eu[1] * p[1]) + eu[2] * p[2];
      const z = (ev[0] * p[0] + ev[1] * p[1]) + ev[2] * p[2];
      const a = y / x, b = z / x;
      const aa = Math.abs(a), bb = Math.abs(b);
      const swap = bb > aa;
      const ta = swap ? bb : aa, tb = swap ? aa : bb;
      const [tu, tv] = this.triangleInverse(ta, tb);
      const u = swap ? tv : tu, v = swap ? tu : tv;
      return [face, a < 0.0 ? -u : u, b < 0.0 ? -v : v];
    }
  }
  CubeMap.WORLD_MAP = WORLD_MAP;
  CubeMap.SIMULATION_MAP = SIMULATION_MAP;
  CubeMap.NEWTON_STEPS = NEWTON_STEPS;
  CubeMap.MAGIC = MAGIC;
  CubeMap.LAYOUT_VERSION = LAYOUT_VERSION;
  CubeMap.HEADER_BYTES = HEADER_BYTES;
  CubeMap.sha256 = data => hexOf(sha256(bytesOf(data)));

  // ---- the world --------------------------------------------------------------------------------
  /** The local east and north unit vectors at the unit vector v (east (0, 1, 0) at the poles). */
  const localFrame = v => {
    const h = sqrt(v[0] * v[0] + v[1] * v[1]);
    const east = h > 1e-12 ? [-v[1] / h, v[0] / h, 0.0] : [0.0, 1.0, 0.0];
    const north = [
      v[1] * east[2] - v[2] * east[1],
      v[2] * east[0] - v[0] * east[2],
      v[0] * east[1] - v[1] * east[0],
    ];
    return [east, north];
  };
  /** The planet's metres per block: `{m, sigma, angle}`, m = [[east, north] per block of net +x,
   *  [east, north] per block of net +z] in planet metres, sigma = sqrt|det m| (planet metres per
   *  block), angle = the direction of net +x on the planet, radians from east toward north. */
  const worldJacobian = m => {
    const det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
    return { m, sigma: sqrt(Math.abs(det)), angle: M.atan2(m[0][1], m[0][0]) };
  };
  /** The unit vector of (latitude, longitude) in degrees, on the portable functions. */
  const unitOfLatLonPm = (latDeg, lonDeg) => {
    const phi = M.toRadians(latDeg), lam = M.toRadians(lonDeg);
    const sp = M.sin(phi), cp = M.cos(phi);
    return [cp * M.cos(lam), cp * M.sin(lam), sp];
  };

  /** A cube world: the net, the map table, the turn that places the cube on the planet and the
   *  planet's size. Latitude, the poles and every metres-per-block figure come from here, never from
   *  the net alone. `mPerBlockH` is the nominal metres per block: the planet's radius in metres is
   *  mPerBlockH x cube.radius() (the six faces' area is the sphere's). */
  class CubeWorld {
    constructor(cube, map, turn, mPerBlockH) {
      this.cube = cube;
      this.map = map;
      this.turn = turn;
      /** The planet's radius, metres. */
      this.radiusM = mPerBlockH * cube.radius(); // ledger: SCL-107
    }
    /** The planet unit vector of a face's local point. */
    unitOnFace(face, u, v) { return this.turn.toPlanet(this.map.forward(face, u, v)); }
    /** The planet unit vector the net point (x, z) shows (x wraps, a point beyond a net edge shows
     *  the point it is glued to); `null` past the net's corners. */
    unit(x, z) {
      const p = this.cube.foldPoint(x, z);
      return p === null ? null : this.unitOnFace(p.face, p.u, p.v);
    }
    /** The face and local point [face, u, v] of the planet unit vector p. */
    localOfUnit(p) { return this.map.inverse(this.turn.toCube(p)); }
    /** The net point [x, z] (in the window) of the planet unit vector p. */
    pointOfUnit(p) {
      const [face, u, v] = this.localOfUnit(p);
      return this.cube.pointOfLocal(face, u, v);
    }
    /** [latitude, longitude] in degrees of the net point (x, z), longitude in [0, 360); `null` past
     *  the net's corners. */
    latLon(x, z) {
      const v = this.unit(x, z);
      return v === null ? null : latLonOfUnit(v);
    }
    /** The net point [x, z] of (latitude, longitude), degrees. */
    pointOfLatLon(latDeg, lonDeg) { return this.pointOfUnit(unitOfLatLonPm(latDeg, lonDeg)); }
    /** Metres per block at a face's local point ({m, sigma, angle}). */
    jacobianOnFace(face, u, v) {
      const j = this.map.jacobian(face, u, v);
      const p = this.turn.toPlanet(j.p);
      const du = this.turn.toPlanet(j.du), dv = this.turn.toPlanet(j.dv);
      const [east, north] = localFrame(p);
      // du/dx = dv/dz = 2/s.
      const k = this.radiusM / (this.cube.s / 2.0);
      return worldJacobian([
        [dot3(du, east) * k, dot3(du, north) * k],
        [dot3(dv, east) * k, dot3(dv, north) * k],
      ]);
    }
    /** Metres per block at the net point (x, z), through the fold (beyond a net edge the motion's
     *  turn is applied, so the rows stay per block of the queried point's +x and +z); `null` past
     *  the net's corners. */
    jacobian(x, z) {
      const p = this.cube.foldPoint(x, z);
      if (p === null) return null;
      const j = this.jacobianOnFace(p.face, p.u, p.v);
      if (p.iso.k === 0) return j;
      const row = (dx, dz) => {
        const [vx, vz] = p.iso.vector(dx, dz);
        return [
          j.m[0][0] * vx + j.m[1][0] * vz,
          j.m[0][1] * vx + j.m[1][1] * vz,
        ];
      };
      return worldJacobian([row(1.0, 0.0), row(0.0, 1.0)]);
    }
    /** The unit vector of north in block space at the net point (x, z), as [dx, dz]; `null` past
     *  the net's corners. */
    north(x, z) {
      const jj = this.jacobian(x, z);
      if (jj === null) return null;
      const j = jj.m;
      // Solve J·(dx, dz) = (0, 1): dx·j[0] + dz·j[1] = (east 0, north 1).
      const det = j[0][0] * j[1][1] - j[1][0] * j[0][1];
      if (det === 0.0 || !Number.isFinite(det)) return [0.0, -1.0];
      const dx = -j[1][0] / det, dz = j[0][0] / det;
      const n = sqrt(dx * dx + dz * dz);
      return [dx / n, dz / n];
    }
    /** The north pole's face and local point [face, u, v], found through the turn. */
    northPole() { return this.localOfUnit([0.0, 0.0, 1.0]); }
    /** The south pole's face and local point [face, u, v], found through the turn. */
    southPole() { return this.localOfUnit([0.0, 0.0, -1.0]); }
    /** The planet unit vector of a cube corner (a `corners()` entry's `cube`, (±1, ±1, ±1)). */
    cornerUnit(cube) {
      const r = sqrt(3.0);
      return this.turn.toPlanet([cube[0] / r, cube[1] / r, cube[2] / r]);
    }
    /** [latitude, longitude] in degrees of a cube corner. */
    cornerLatLon(cube) { return latLonOfUnit(this.cornerUnit(cube)); }
  }

  root.DTCube = Cube;
  root.DTCubeTurn = CubeTurn;
  root.DTCubeMap = CubeMap;
  root.DTCubeWorld = CubeWorld;
  root.DTLocalFrame = localFrame;
  root.DTUnitOfLatLonPm = unitOfLatLonPm;
})(typeof window !== 'undefined' ? window : globalThis);
