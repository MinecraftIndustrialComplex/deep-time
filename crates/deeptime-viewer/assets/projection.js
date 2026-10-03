// SPDX-License-Identifier: LGPL-3.0-or-later
// The planet's map projection for the pages (the workbench and the static viewer): the one place
// they convert between block coordinates, map pictures and latitude/longitude. The twin of the Rust
// `deeptime_format::projection` and of Java's `worlddata/Projection`
// (docs/design/equal-area-projection.md §2); loaded before every other script as `window.DTProj`.
//
// Equirectangular: longitude λ = 360°·(x + C/2)/C (the seam x = −C/2 is 0°), latitude
// φ = −360°·z/C (north is −z), the poles at z = ∓C/4, one block 360°/C both ways. A map picture
// (a keyframe) is 2 : 1: x linear in longitude from the seam, y linear in latitude from the north
// pole. The projection is a game choice (ledger: SCL-049); whichever is picked replaces these
// bodies.
(function (root) {
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  root.DTProj = {
    /** |z| of the poles, blocks. */
    poleZ: C => C / 4,
    /** Latitude, degrees, of block z (clamped to ±90° past the poles). */
    latOfZ: (C, z) => clamp(-z / C * 360, -90, 90),
    /** Block z of a latitude. */
    zOfLat: (C, lat) => -lat / 360 * C,
    /** Longitude, degrees in [0, 360), of block x. */
    lonOfX: (C, x) => (((x + C / 2) / C) * 360 % 360 + 360) % 360,
    /** Block x of a longitude (not wrapped). */
    xOfLon: (C, lon) => lon / 360 * C - C / 2,
    /** Degrees of longitude and latitude a step of (dx, dz) blocks spans at latitude `lat`. */
    dLonOfDx: (C, dx, lat) => dx / C * 360,
    dLatOfDz: (C, dz, lat) => -dz / C * 360,
    /** Blocks a step of (dlon, dlat) degrees spans at latitude `lat`. */
    dxOfDLon: (C, dlon, lat) => dlon / 360 * C,
    dzOfDLat: (C, dlat, lat) => -dlat / 360 * C,
    /** Whether block z is on the planet (between the poles; x wraps, so every x is). */
    onPlanet: (C, z) => Math.abs(z) <= C / 4,
    /** Wraps a longitude into [0, 360). */
    wrapLon: lon => ((lon % 360) + 360) % 360,
    /** Column coordinate of a longitude (as given, not wrapped) in a map picture `w` columns wide
     *  (x = 0 the seam). */
    xOfLonPic: (lon, w) => lon / 360 * w,
    /** Longitude (not wrapped) at column coordinate x of a map picture `w` columns wide. */
    lonOfPicX: (x, w) => x / w * 360,
    /** Row coordinate of a latitude in a map picture `h` rows high (y = 0 the north pole). */
    yOfLat: (lat, h) => (90 - lat) / 180 * h,
    /** Latitude at row coordinate y of a map picture `h` rows high. */
    latOfY: (y, h) => 90 - y / h * 180,
    /** The picture's row share (0 north … 1 south) of sin φ, for the globe's texture lookups. */
    vOfSinLat: s => 0.5 - Math.asin(s < -1 ? -1 : s > 1 ? 1 : s) / Math.PI,
    /** The height of a map picture `w` wide (the projection's aspect, 2 : 1). */
    pictureHeight: w => Math.max(1, Math.floor(w / 2)),
  };
})(typeof window !== 'undefined' ? window : globalThis);
