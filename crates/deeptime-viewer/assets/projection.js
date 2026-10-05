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
      let cosGammap;
      if (xpyp > 0.0) { const q = up * sin(gp) / xpyp; cosGammap = sqrt0(1.0 - q * q); } else cosGammap = 1.0;
      const xp = xpyp * xpxpyp;
      const yp = xpyp - xp;
      const rp = sqrt0(xp * xp + cp * cp - 2.0 * xp * cp * cosGammap);
      let alphap;
      if (up * rp <= 0.0) alphap = xip - PI / 6.0;
      else { const q = (yp * yp - up * up - rp * rp) / (-2.0 * up * rp); alphap = q < 1.0 ? acos(q) : 0.0; }
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
