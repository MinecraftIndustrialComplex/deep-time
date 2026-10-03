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
