// SPDX-License-Identifier: LGPL-3.0-or-later
// The pages' twin of the world's projection (crates/deeptime-viewer/assets/projection.js: DTMath
// and the cube world) against the Rust original, bit for bit: every output in
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
// The world object a manifest without a layout block builds: the Lambert cylinder.
const lam = globalThis.DTProj.world(null, 16384);
if (lam.kind !== 'lambert' || Math.abs(lam.latLon(0, -lam.radius)[0] - 90) > 1e-9) {
  console.error('DTProj.world(null) is not the Lambert cylinder'); process.exit(1);
}

// ---------------------------------------------------------------------------------------------
// The cube world (docs/plans/cube-world-plan.md W1, W2, M4): DTCube, DTCubeTurn, DTCubeMap and
// DTCubeWorld against the "cube" member of the fixture, every value; integers exactly, floats by
// their bits, `null` for Rust's None.
const Cube = globalThis.DTCube;
const CubeTurn = globalThis.DTCubeTurn;
const CubeMap = globalThis.DTCubeMap;
const CubeWorld = globalThis.DTCubeWorld;
const cubeFx = fixture.cube;
if (!cubeFx) { console.error('the fixture has no "cube" member'); process.exit(1); }

const fail = msg => { console.error(`DIFFERENT ${msg}`); process.exit(1); };
/** An integer, string or null: strict equality. */
const eq = (what, got, want) => { checked++; if (got !== want) fail(`${what}: got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`); };
/** An array of integers or strings (or null). */
const eqs = (what, got, want) => {
  if (want === null || got === null) { eq(what, got, want); return; }
  eq(`${what}.length`, got.length, want.length);
  for (let i = 0; i < want.length; i++) eq(`${what}[${i}]`, got[i], want[i]);
};
/** A condition: counts as one value. */
const need = (what, ok) => { checked++; if (!ok) fail(what); };
/** A float triple against hex bits. */
const same3 = (what, got, want) => { for (let i = 0; i < 3; i++) same(`${what}[${i}]`, got[i], want[i]); };
const hexOfBytes = b => Array.from(b, x => x.toString(16).padStart(2, '0')).join('');
const nodeSha = b => require('crypto').createHash('sha256').update(b).digest('hex');

// SHA-256: the pages' synchronous twin against FIPS 180-4's examples and node's own.
for (const [msg, want] of [
  ['', 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'],
  ['abc', 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'],
  ['abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq', '248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1'],
  ['abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu', 'cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1'],
  ['a'.repeat(1000000), 'cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0'],
]) eq(`sha256 of a ${msg.length}-byte message`, CubeMap.sha256(Buffer.from(msg, 'latin1')), want);
for (let n = 0; n < 300; n++) {
  const b = Buffer.alloc(n);
  for (let i = 0; i < n; i++) b[i] = (i * 131 + n * 7 + 5) & 255;
  eq(`sha256 of ${n} patterned bytes`, CubeMap.sha256(b), nodeSha(b));
}

// W1: the face side.
eq('DTCube.FACE_QUANTUM', Cube.FACE_QUANTUM, 256);
eq('faceFor(4096)', Cube.faceFor(4096), 1024);
eq('faceFor(16384)', Cube.faceFor(16384), 3840);
eq('faceFor(32768)', Cube.faceFor(32768), 7424);
for (const [k, r] of cubeFx.face_for.entries()) eq(`cube.face_for[${k}] c ${r.c}`, Cube.faceFor(r.c), r.s);

// W2: the net. Edges (every field), corners, adjacency, blocks (fold), points (foldPoint).
const net = cubeFx.net;
const cube = new Cube(net.s);
eq('net.edges.length', cube.edges.length, net.edges.length);
for (const [k, w] of net.edges.entries()) {
  const e = cube.edges[k], p = `net.edges[${k}] ${w.name}`;
  eq(`${p}.name`, e.name, w.name);
  eq(`${p}.seam`, e.seam, w.seam);
  eq(`${p}.motion`, e.motion, w.motion);
  eq(`${p}.face`, e.face, w.face);
  eq(`${p}.face_side`, e.faceSide, w.face_side);
  eq(`${p}.axis`, e.axis, w.axis);
  eq(`${p}.beyond`, e.beyond, w.beyond);
  eq(`${p}.coord`, e.coord, w.coord);
  eqs(`${p}.span`, e.span, w.span);
  eqs(`${p}.iso`, e.iso.asArray(), w.iso);
  eqs(`${p}.pivot`, e.pivot, w.pivot);
  eq(`${p}.partner`, e.partner, w.partner);
  eq(`${p}.owner`, e.owner, w.owner);
}
const corners = cube.corners();
eq('net.corners.length', corners.length, net.corners.length);
for (const [k, w] of net.corners.entries()) {
  const c = corners[k], p = `net.corners[${k}] ${w.name}`;
  eq(`${p}.name`, c.name, w.name);
  eqs(`${p}.faces`, c.faces, w.faces);
  eqs(`${p}.cube`, c.cube, w.cube);
  eq(`${p}.images.length`, c.images.length, w.images.length);
  for (const [i, img] of w.images.entries()) eqs(`${p}.images[${i}]`, c.images[i], img);
}
for (let face = 0; face < 6; face++) {
  for (let side = 0; side < 4; side++) {
    const a = Cube.ADJACENCY[face][side], w = net.adjacency[face][side];
    eqs(`net.adjacency[${face}][${side}]`, [a.face, a.side, a.quarterTurns, a.shift[0], a.shift[1]], w);
  }
}
for (const [k, w] of net.blocks.entries()) {
  const p = `net.blocks[${k}] (${w.x}, ${w.z})`;
  const r = cube.fold(w.x, w.z);
  eq(`${p}.kind`, r.kind === 'face' ? 'face' : r.kind, w.kind);
  if (w.kind === 'face') {
    eq(`${p}.face`, r.face, w.face);
    eq(`${p}.i`, r.i, w.i);
    eq(`${p}.j`, r.j, w.j);
  } else if (w.kind === 'outside') {
    eq(`${p}.edge`, r.edge, w.edge);
    eq(`${p}.px`, r.x, w.px);
    eq(`${p}.pz`, r.z, w.pz);
    eqs(`${p}.iso`, r.iso.asArray(), w.iso);
  }
}
for (const [k, w] of net.points.entries()) {
  const p = `net.points[${k}]`;
  const r = cube.foldPoint(f(w.x), f(w.z));
  if (w.face === null) { eq(`${p}.face`, r, null); continue; }
  need(`${p}: foldPoint is null, want face ${w.face}`, r !== null);
  eq(`${p}.face`, r.face, w.face);
  same(`${p}.u`, r.u, w.u);
  same(`${p}.v`, r.v, w.v);
  same(`${p}.px`, r.x, w.px);
  same(`${p}.pz`, r.z, w.pz);
  eqs(`${p}.iso`, r.iso.asArray(), w.iso);
  eq(`${p}.edge`, r.edge, w.edge);
}

// M2/M4: the tables (read from the crate's data folder, checked against the .sha256 file and the
// fixture's), forward, jacobian and inverse.
const dataDir = path.join(__dirname, '../../crates/deeptime-format/data');
const maps = {};
for (const mp of cubeFx.maps) {
  const file = fs.readFileSync(path.join(dataDir, `cube-map-${mp.name}.bin`));
  const sumFile = fs.readFileSync(path.join(dataDir, `cube-map-${mp.name}.bin.sha256`), 'utf8').trim().split(/\s+/)[0];
  eq(`maps ${mp.name}: the .sha256 file vs the fixture`, sumFile, mp.sha256);
  eq(`maps ${mp.name}: node's sha256 of the file`, nodeSha(file), mp.sha256);
  const map = CubeMap.loadChecked(file, mp.sha256);
  eq(`maps ${mp.name}.name`, map.name, mp.name);
  eq(`maps ${mp.name}.sha256Hex`, map.sha256Hex(), mp.sha256);
  eq(`maps ${mp.name}.fileName`, CubeMap.fileName(mp.name), `cube-map-${mp.name}.bin`);
  // A wrong hash, a cut file and a wrong magic are refused.
  let threw = false;
  try { CubeMap.loadChecked(file, '0'.repeat(64)); } catch (err) { threw = true; }
  need(`maps ${mp.name}: loadChecked takes a wrong hash`, threw);
  threw = false;
  try { CubeMap.load(file.subarray(0, file.length - 16)); } catch (err) { threw = true; }
  need(`maps ${mp.name}: load takes a cut file`, threw);
  threw = false;
  const wrongMagic = Buffer.from(file); wrongMagic[0] = 0x58;
  try { CubeMap.load(wrongMagic); } catch (err) { threw = true; }
  need(`maps ${mp.name}: load takes a wrong magic`, threw);
  maps[mp.name] = map;
  for (const [k, r] of mp.points.entries()) {
    const p = `maps ${mp.name} points[${k}] face ${r.face}`;
    const u = f(r.u), v = f(r.v);
    same3(`${p}.p`, map.forward(r.face, u, v), r.p);
    const j = map.jacobian(r.face, u, v);
    same3(`${p}.jacobian.p`, j.p, r.p);
    same3(`${p}.du`, j.du, r.du);
    same3(`${p}.dv`, j.dv, r.dv);
  }
  for (const [k, r] of mp.inverse.entries()) {
    const p = `maps ${mp.name} inverse[${k}]`;
    const [face, u, v] = map.inverse(r.p.map(f));
    eq(`${p}.face`, face, r.face);
    same(`${p}.u`, u, r.u);
    same(`${p}.v`, v, r.v);
  }
}

// The worlds: a net, a table and a turn together.
for (const [wi, w] of cubeFx.worlds.entries()) {
  const tag = `worlds[${wi}] (s ${w.s}, ${w.map})`;
  const rows = [0, 1, 2].map(i => [0, 1, 2].map(j => f(w.rotation[3 * i + j])));
  const turn = CubeTurn.fromRows(rows);
  for (let i = 0; i < 9; i++) same(`${tag}.rotation[${i}]`, turn.rows()[Math.floor(i / 3)][i % 3], w.rotation[i]);
  const world = new CubeWorld(new Cube(w.s), maps[w.map], turn, f(w.m_per_block_h));
  same(`${tag}.radius_m`, world.radiusM, w.radius_m);
  for (const [name, got] of [['north_pole', world.northPole()], ['south_pole', world.southPole()]]) {
    const want = w[name];
    const [face, u, v] = got;
    const [x, z] = world.cube.pointOfLocal(face, u, v);
    eq(`${tag}.${name}.face`, face, want.face);
    same(`${tag}.${name}.u`, u, want.u);
    same(`${tag}.${name}.v`, v, want.v);
    same(`${tag}.${name}.x`, x, want.x);
    same(`${tag}.${name}.z`, z, want.z);
  }
  const cs = world.cube.corners();
  eq(`${tag}.corners.length`, cs.length, w.corners.length);
  for (const [k, r] of w.corners.entries()) {
    eq(`${tag}.corners[${k}].name`, cs[k].name, r.name);
    same3(`${tag}.corners[${k}].unit`, world.cornerUnit(cs[k].cube), r.unit);
    const [lat, lon] = world.cornerLatLon(cs[k].cube);
    same(`${tag}.corners[${k}].lat`, lat, r.lat);
    same(`${tag}.corners[${k}].lon`, lon, r.lon);
  }
  for (const [k, r] of w.points.entries()) {
    const p = `${tag}.points[${k}]`;
    const x = f(r.x), z = f(r.z);
    const v = world.unit(x, z), ll = world.latLon(x, z), j = world.jacobian(x, z);
    if (r.v === null) {
      eq(`${p}.unit`, v, null);
      eq(`${p}.latLon`, ll, null);
      eq(`${p}.jacobian`, j, null);
      eq(`${p}.north`, world.north(x, z), null);
      continue;
    }
    need(`${p}: a point of the planet came back null`, v !== null && ll !== null && j !== null);
    same3(`${p}.v`, v, r.v);
    same(`${p}.lat`, ll[0], r.lat);
    same(`${p}.lon`, ll[1], r.lon);
    same(`${p}.m[0]`, j.m[0][0], r.m[0]);
    same(`${p}.m[1]`, j.m[0][1], r.m[1]);
    same(`${p}.m[2]`, j.m[1][0], r.m[2]);
    same(`${p}.m[3]`, j.m[1][1], r.m[3]);
    same(`${p}.sigma`, j.sigma, r.sigma);
    same(`${p}.angle`, j.angle, r.angle);
    // North in block space (not in the fixture): a unit vector that J maps onto due north.
    const [dx, dz] = world.north(x, z);
    const east = dx * j.m[0][0] + dz * j.m[1][0], north = dx * j.m[0][1] + dz * j.m[1][1];
    need(`${p}: north (${dx}, ${dz}) is not a unit vector`, Math.abs(Math.hypot(dx, dz) - 1) < 1e-12);
    need(`${p}: north (${dx}, ${dz}) moves ${east} m east of due north`, Math.abs(east) <= 1e-9 * j.sigma && north > 0);
  }
  for (const [k, r] of w.inverse.entries()) {
    const [x, z] = world.pointOfLatLon(f(r.lat), f(r.lon));
    same(`${tag}.inverse[${k}].x`, x, r.x);
    same(`${tag}.inverse[${k}].z`, z, r.z);
  }
}

// W2: the geometry checks of cube.rs's tests, run in JS (SplitMix64 as the Rust tests', in BigInt).
const MASK64 = (1n << 64n) - 1n;
class SplitMix {
  constructor(seed) { this.state = seed & MASK64; }
  next() {
    this.state = (this.state + 0x9E3779B97F4A7C15n) & MASK64;
    let z = this.state;
    z = ((z ^ (z >> 30n)) * 0xBF58476D1CE4E5B9n) & MASK64;
    z = ((z ^ (z >> 27n)) * 0x94D049BB133111EBn) & MASK64;
    return z ^ (z >> 31n);
  }
  range(lo, hi) { return lo + Number(this.next() % BigInt(hi - lo)); }
}
const SIDES_UNDER_TEST = [1024, 3840, 64];

// Every seam's motion maps the partner's band onto the edge's, and closes up (twice for half
// turns, four times for quarter turns).
for (const s of SIDES_UNDER_TEST) {
  const c = new Cube(s);
  eq(`seams s ${s}: 14 edges`, c.edges.length, 14);
  for (const [k, e] of c.edges.entries()) {
    const p = c.edges[e.partner];
    const tag = `seams s ${s} ${e.name}`;
    eq(`${tag}: the partner's partner`, p.partner, k);
    eq(`${tag}: the partner's seam`, p.seam, e.seam);
    eq(`${tag}: span width`, e.span[1] - e.span[0], s);
    // Every block of the band's first row maps onto the partner edge's inner row, inside its span
    // and on the partner's face: the motion takes beyond-this-edge onto inside-the-partner (and,
    // run from the partner's side, the partner's band onto this edge's inner row).
    let bad = null;
    for (let t = e.span[0]; t < e.span[1] && bad === null; t++) {
      const n = e.beyond === 'max' ? e.coord : e.coord - 1;
      const [x, z] = e.axis === 'x' ? [n, t] : [t, n];
      const [px, pz] = e.iso.cell(x, z);
      const [pn, along] = p.axis === 'x' ? [px, pz] : [pz, px];
      const inner = p.beyond === 'max' ? pn === p.coord - 1 : pn === p.coord;
      const fb = c.faceOfBlock(px, pz);
      if (!inner) bad = `(${x}, ${z}) -> (${px}, ${pz}) not on ${p.name}`;
      else if (!(along >= p.span[0] && along < p.span[1])) bad = `(${x}, ${z}) -> (${px}, ${pz}) outside ${p.name}'s span`;
      else if (fb === null || fb[0] !== p.face) bad = `(${x}, ${z}) -> (${px}, ${pz}) is not on ${p.name}'s face`;
    }
    need(`${tag}: ${bad}`, bad === null);
    need(`${tag}: the partner's motion is this one's inverse`, p.iso.equals(e.iso.inverse()));
    const order = e.motion === 'slide' ? 1 : e.motion === 'half_turn' ? 2 : 4;
    let m = Cube.Iso.IDENTITY;
    for (let i = 0; i < order; i++) m = e.iso.compose(m);
    if (e.motion !== 'slide') {
      need(`${tag}: to the power ${order} is not the identity`, m.equals(Cube.Iso.IDENTITY));
      const [a, b] = e.pivot;
      eqs(`${tag}: the pivot is fixed`, e.iso.point(a, b), [a, b]);
      eq(`${tag}: pivot x on a chunk corner`, ((a % 16) + 16) % 16, 0);
      eq(`${tag}: pivot z on a chunk corner`, ((b % 16) + 16) % 16, 0);
    }
  }
}

// fold(partner) is the identity on 10,000 random outside blocks within s of the planet (Rust
// `fold_round_trips_outside_blocks`): the partner is a planet block, and the motion's inverse
// brings it back.
const withinS = (c, x, z) => {
  const s = c.s;
  x = c.wrapX(x);
  return Cube.FACES.some(fc => {
    const [ox, oz] = c.faceOrigin(fc);
    return [-4 * s, 0, 4 * s].some(o => {
      const dx = Math.max(ox + o - x, x - (ox + o + s - 1), 0);
      const dz = Math.max(oz - z, z - (oz + s - 1), 0);
      return Math.max(dx, dz) <= s;
    });
  });
};
for (const s of SIDES_UNDER_TEST) {
  const c = new Cube(s);
  const rng = new SplitMix(0x0DE57001n ^ BigInt(s));
  let outside = 0, ocean = 0, tries = 0, bad = null;
  while (outside < 10000 && bad === null) {
    tries++;
    if (tries >= 1000000) { bad = 'too many tries'; break; }
    const x = rng.range(-5 * s / 2 - s, 7 * s / 2 + s);
    const z = rng.range(-5 * s / 2, 5 * s / 2);
    if (c.faceOfBlock(x, z) !== null || !withinS(c, x, z)) continue;
    const r = c.fold(x, z);
    if (r.kind === 'ocean') { ocean++; continue; }
    if (r.kind !== 'outside') { bad = `(${x}, ${z}) is off the planet`; break; }
    outside++;
    const back = r.iso.cell(x, z);
    const e = c.edges[r.edge];
    const fb = c.faceOfBlock(r.x, r.z);
    if (back[0] !== r.x || back[1] !== r.z) bad = `(${x}, ${z}): iso.cell is not the partner block`;
    else if (c.fold(r.x, r.z).kind !== 'face') bad = `(${x}, ${z}): the partner block is not on the planet`;
    else if (r.iso.inverse().cell(r.x, r.z).join() !== [x, z].join()) bad = `(${x}, ${z}): the inverse motion does not bring it back`;
    else if (e.depth(c.wrapX(x), z) === null) bad = `(${x}, ${z}): not in edge ${e.name}'s band`;
    else if (fb[0] !== c.edges[e.partner].face) bad = `(${x}, ${z}): the partner block is not on ${c.edges[e.partner].name}'s face`;
  }
  need(`fold round trip s ${s}: ${bad}`, bad === null);
  eq(`fold round trip s ${s}: outside blocks tested`, outside, 10000);
  console.log(`  s ${s}: 10,000 outside blocks round-trip, ${ocean} ocean (past the net's corners)`);
}

// The corner images agree: every image is a corner of the faces that draw it, and each of a
// corner's three faces has an image.
const gnomonic = (fc, a, b) => { const [c0, eu, ev] = Cube.frame(fc); return [0, 1, 2].map(k => c0[k] + a * eu[k] + b * ev[k]); };
for (const s of SIDES_UNDER_TEST) {
  const c = new Cube(s);
  const h = s / 2;
  const cs = c.corners();
  eq(`corners s ${s}: 8`, cs.length, 8);
  for (const cn of cs) {
    const want = cn.cube;
    const viaImage = (img, fc) => {
      const [cx, cz] = c.faceCentre(fc);
      const hits = [];
      for (const off of [0, 4 * s, -4 * s]) {
        const u = (img[0] + off - cx) / h, v = (img[1] - cz) / h;
        if (Math.abs(u) === 1 && Math.abs(v) === 1) hits.push(gnomonic(fc, u, v));
      }
      return hits;
    };
    for (const img of cn.images) {
      let seen = 0;
      for (const fc of Cube.FACES) {
        for (const p of viaImage(img, fc)) {
          need(`corners s ${s} ${cn.name}: image ${img} on face ${fc} which does not meet there`, cn.faces.includes(fc));
          need(`corners s ${s} ${cn.name}: image ${img} via face ${fc} is ${p}, want ${want}`, p.join() === want.join());
          seen++;
        }
      }
      need(`corners s ${s} ${cn.name}: image ${img} touches no face`, seen >= 1);
    }
    for (const fc of cn.faces) {
      need(`corners s ${s} ${cn.name}: face ${fc} has no image`,
        cn.images.some(img => viaImage(img, fc).some(p => p.join() === want.join())));
    }
  }
  // The F0 corners are the quarter-turn pivots.
  const pivots = c.edges.filter(e => e.pivot !== null).map(e => e.pivot.join());
  for (const cn of cs.filter(x => x.faces.includes(0))) {
    eq(`corners s ${s} ${cn.name}: one image`, cn.images.length, 1);
    need(`corners s ${s} ${cn.name}: the image is a pivot`, pivots.includes(cn.images[0].join()));
  }
}

// The adjacency agrees with the cube frames (a neighbour's local point is the same cube point) and
// with the fold (a cell beyond a side is the block the fold gives).
const gnomonicOf = (fc, p) => {
  const [c0, eu, ev] = Cube.frame(fc);
  const d = q => p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
  return [d(eu) / d(c0), d(ev) / d(c0)];
};
for (const fc of Cube.FACES) {
  for (const side of Cube.SIDES) {
    const a = Cube.ADJACENCY[fc][side];
    const tag = `adjacency ${Cube.FACE_NAMES[fc]} ${Cube.SIDE_NAMES[side]}`;
    for (const t of [-0.75, -0.2, 0.0, 0.4, 1.0]) {
      const [u, v] = side === 0 ? [1.0, t] : side === 2 ? [-1.0, t] : side === 1 ? [t, 1.0] : [t, -1.0];
      const p = gnomonic(fc, u, v);
      const [u2, v2] = a.local(u, v);
      const q = gnomonic(a.face, u2, v2);
      need(`${tag} t ${t}: ${p} vs ${q}`, [0, 1, 2].every(k => Math.abs(p[k] - q[k]) < 1e-12));
      const [gu, gv] = gnomonicOf(a.face, p);
      need(`${tag} t ${t}: the gnomonic point of the neighbour`, Math.abs(gu - u2) < 1e-12 && Math.abs(gv - v2) < 1e-12);
    }
    // A step out of the face lands just inside the neighbour, on the side it names, and back.
    const [u, v] = side === 0 ? [1.01, 0.3] : side === 2 ? [-1.01, 0.3] : side === 1 ? [0.3, 1.01] : [0.3, -1.01];
    const [u2, v2] = a.local(u, v);
    need(`${tag}: a step out lands outside the neighbour (${u2}, ${v2})`, Math.abs(u2) < 1.0 && Math.abs(v2) < 1.0);
    const near = a.side === 0 ? u2 > 0.98 : a.side === 2 ? u2 < -0.98 : a.side === 1 ? v2 > 0.98 : v2 < -0.98;
    need(`${tag}: enters ${a.side} at (${u2}, ${v2})`, near);
    const back = Cube.ADJACENCY[a.face][a.side];
    eq(`${tag}: back face`, back.face, fc);
    eq(`${tag}: back side`, back.side, side);
    const [u3, v3] = back.local(u2, v2);
    need(`${tag}: back (${u3}, ${v3})`, Math.abs(u3 - u) < 1e-12 && Math.abs(v3 - v) < 1e-12);
  }
}
for (const s of SIDES_UNDER_TEST) {
  const c = new Cube(s);
  const n = s;
  for (const fc of Cube.FACES) {
    for (const side of Cube.SIDES) {
      for (const depth of [0, 1, Math.floor(n / 3), n - 1]) {
        for (const t of [0, 1, n / 2, n - 2, n - 1]) {
          const [i, j] = side === 0 ? [n + depth, t] : side === 2 ? [-1 - depth, t] : side === 1 ? [t, n + depth] : [t, -1 - depth];
          const on = Cube.cellOn(fc, n, i, j);
          const tag = `adjacency vs fold s ${s} ${Cube.FACE_NAMES[fc]} ${Cube.SIDE_NAMES[side]} depth ${depth} t ${t}`;
          need(`${tag}: cellOn is null`, on !== null);
          const [g, i2, j2, k] = on;
          const [x, z] = c.blockOfCell(fc, i, j);
          const r = c.fold(x, z);
          need(`${tag}: ocean`, r.kind !== 'ocean');
          const [fx, fz, fk] = r.kind === 'face' ? [...c.blockOfCell(r.face, r.i, r.j), 0] : [r.x, r.z, r.iso.k];
          // A block in a void square by F0's corner may belong to the other edge's half (the fold
          // splits the square on its diagonal): the adjacency's answer is then the other half's.
          if (!(depth >= Math.min(t, n - 1 - t))) {
            const [bx, bz] = c.blockOfCell(g, i2, j2);
            need(`${tag}: adjacency (${bx}, ${bz}, ${k}) vs fold (${fx}, ${fz}, ${fk})`, bx === fx && bz === fz && k === fk);
          }
        }
      }
    }
  }
}

// The polar turn puts the planet point (0, lambda0) at F0's centre and the pole at N's; a free turn
// inverts by its transpose; reflections are refused.
{
  const polar = CubeTurn.polar(30.0);
  const c = polar.toCube([Math.cos(Math.PI / 6), Math.sin(Math.PI / 6), 0.0]);
  need('polar turn: (0, 30) at F0\'s centre', Math.abs(c[0] - 1) < 1e-15 && Math.abs(c[1]) < 1e-15 && Math.abs(c[2]) < 1e-15);
  eqs('polar turn: the pole at N\'s centre', polar.toCube([0.0, 0.0, 1.0]), [0, 0, 1]);
  let threw = false;
  try { CubeTurn.fromRows([[1, 0, 0], [0, 1, 0], [0, 0, -1]]); } catch (err) { threw = true; }
  need('a reflection is refused', threw);
  threw = false;
  try { CubeTurn.fromRows([[1, 0, 0], [0, 1, 0], [0, 0.1, 1]]); } catch (err) { threw = true; }
  need('non-orthonormal rows are refused', threw);
  const r = 1 / Math.sqrt(3), s2 = 1 / Math.sqrt(2), t6 = 1 / Math.sqrt(6);
  const free = CubeTurn.fromRows([[r, r, r], [s2, -s2, 0], [t6, t6, -2 * t6]]);
  const rng = new SplitMix(3n);
  for (let i = 0; i < 100; i++) {
    const q = [rng.range(-1000, 1000), rng.range(-1000, 1000), rng.range(-1000, 1000)];
    const back = free.toPlanet(free.toCube(q));
    need(`a free turn inverts by its transpose: ${q} -> ${back}`, [0, 1, 2].every(k => Math.abs(back[k] - q[k]) < 1e-12));
  }
}

console.log(`projection-check: ${checked} values identical to the Rust bits`);
