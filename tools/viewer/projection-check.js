// SPDX-License-Identifier: LGPL-3.0-or-later
// The pages' twin of the seamless world's projection (crates/deeptime-viewer/assets/projection.js:
// DTMath, DTPetroff, DTPetroffGuyou) against the Rust original, bit for bit: every output in
// crates/deeptime-format/tests/data/projection_parity.json (written by deeptime-format's
// `parity_fixture` test) must come out with the same bits. Run on the test VM (nothing heavy on the
// laptop):
//
//   tools/remote/mac.sh -- 'nix shell nixpkgs#nodejs -c node tools/viewer/projection-check.js'
//
// Exits non-zero on the first difference.
'use strict';
const fs = require('fs');
const path = require('path');

require(path.join(__dirname, '../../crates/deeptime-viewer/assets/projection.js'));
const M = globalThis.DTMath;
const fixture = JSON.parse(fs.readFileSync(
  path.join(__dirname, '../../crates/deeptime-format/tests/data/projection_parity.json'), 'utf8'));

const buf = new DataView(new ArrayBuffer(8));
const f = hex => { buf.setBigUint64(0, BigInt('0x' + hex)); return buf.getFloat64(0); };
const bits = v => { buf.setFloat64(0, v); return buf.getBigUint64(0).toString(16).padStart(16, '0'); };
let checked = 0;
const same = (what, got, want) => {
  checked++;
  if (bits(got) !== want) {
    console.error(`DIFFERENT ${what}: got ${got} (${bits(got)}), want ${f(want)} (${want})`);
    process.exit(1);
  }
};

for (const [k, r] of fixture.math.entries()) {
  const x = f(r.x), y = f(r.y), u = f(r.u);
  same(`math[${k}].sin`, M.sin(x), r.sin);
  same(`math[${k}].cos`, M.cos(x), r.cos);
  same(`math[${k}].tan`, M.tan(x), r.tan);
  same(`math[${k}].atan`, M.atan(x), r.atan);
  same(`math[${k}].atan2`, M.atan2(y, x), r.atan2);
  same(`math[${k}].asin`, M.asin(u), r.asin);
  same(`math[${k}].acos`, M.acos(u), r.acos);
}
const pet = new globalThis.DTPetroff();
for (const [k, r] of fixture.petroff.entries()) {
  const [l, p] = pet.inverse(f(r.x), f(r.y));
  same(`petroff[${k}].lambda`, l, r.lambda);
  same(`petroff[${k}].phi`, p, r.phi);
  const [x, y] = pet.forward(f(r.lam_in), f(r.phi_in));
  same(`petroff[${k}].x_out`, x, r.x_out);
  same(`petroff[${k}].y_out`, y, r.y_out);
}
for (const w of fixture.worlds) {
  const g = new globalThis.DTPetroffGuyou(w.b, f(w.lambda_c_deg));
  for (const [k, r] of w.points.entries()) {
    const x = f(r.x), z = f(r.z);
    const v = g.unit(x, z);
    for (let i = 0; i < 3; i++) same(`b ${w.b} points[${k}].v[${i}]`, v[i], r.v[i]);
    const [lat, lon] = g.latLon(x, z);
    same(`b ${w.b} points[${k}].lat`, lat, r.lat);
    same(`b ${w.b} points[${k}].lon`, lon, r.lon);
  }
  for (const [k, r] of w.inverse.entries()) {
    const [x, z] = g.blockOf(f(r.lat), f(r.lon));
    same(`b ${w.b} inverse[${k}].x`, x, r.x);
    same(`b ${w.b} inverse[${k}].z`, z, r.z);
  }
}
// The world object a manifest's layout block builds, and the Lambert fallback.
const world = globalThis.DTProj.world({ kind: 'petroff_guyou', hemisphere_blocks: 6656, lambda_c_deg: 31.25 }, 13312);
if (!world.isPetroff || world.period !== 13312 || world.zRange[1] !== 3328) {
  console.error('DTProj.world does not build the Petroff–Guyou world'); process.exit(1);
}
const lam = globalThis.DTProj.world(null, 16384);
if (lam.isPetroff || Math.abs(lam.latLon(0, -lam.radius)[0] - 90) > 1e-9) {
  console.error('DTProj.world(null) is not the Lambert cylinder'); process.exit(1);
}
console.log(`projection-check: ${checked} values identical to the Rust bits`);
