// SPDX-License-Identifier: LGPL-3.0-or-later
// Checks of the workbench's deposits overlay module (crates/deeptime-viewer/assets/deposits.js),
// which is pure apart from the canvas it draws on. Run on the Mac (nothing heavy on the laptop):
//
//   tools/remote/mac.sh -- 'nix shell nixpkgs#nodejs -c node tools/viewer/deposits-check.js'
//
// Exits non-zero at the first failed assertion.
'use strict';
const assert = require('assert');
const path = require('path');

// The page's scripts at least compile (a syntax error would blank the workbench).
const fs = require('fs'), vm = require('vm');
for (const f of ['projection.js', 'workbench.js', 'deposits.js']) {
  const file = path.join(__dirname, '../../crates/deeptime-viewer/assets', f);
  new vm.Script(fs.readFileSync(file, 'utf8'), { filename: file });
}

globalThis.Path2D = class { constructor(d) { this.d = d; } };
require(path.join(__dirname, '../../crates/deeptime-viewer/assets/projection.js'));
require(path.join(__dirname, '../../crates/deeptime-viewer/assets/deposits.js'));
const P = globalThis.DTProj;   // the planet's projection (Lambert equal-area)
const D = globalThis.DTDeposits;

const C = 16384;
const raw = (id, type, x, z, extra) => Object.assign({
  id, kind: 'deposit', type, x, y: 20, z, y_range: [10, 30], lat: P.latOfZ(C, z), lon: P.lonOfX(C, x),
  fp: { x, z, a: 40, c: 60, yaw: 0.5, km2: 678.6 }, bodies: 3,
  products: [{ material: 'c:coal', grade: 1, tonnage_t: 1.98e10, role: 'primary' }], forming_event: 1000 + id, parent: null,
}, extra || {});
const answer = {
  source: 'bake', circumference: C, m_per_block_h: 300,
  sites: [
    raw(1, 'deeptime:coal_seam', -3000, -1200),
    raw(2, 'deeptime:coal_seam', -3100, -1250, { products: [{ material: 'c:coal', grade: 1, tonnage_t: 2e9, role: 'primary' }] }),
    raw(3, 'deeptime:porphyry_cu_mo_au', 4000, 900, { products: [{ material: 'c:copper', grade: 0.0044, tonnage_t: 2.4e8, role: 'primary' }, { material: 'c:gold', grade: 1e-6, tonnage_t: 40, role: 'byproduct' }] }),
    raw(4, 'deeptime:placer_au', 4100, 950, { parent: 3 }),
    raw(5, 'datapack:glow_stone', 0, 0),
    raw(6, 'deeptime:bif_fe', -8190, 100),   // at the seam: lon just east of 0
    raw(7, 'deeptime:supergene_cu_blanket', 4000, 900, { parent: 3 }),   // under its porphyry: the same spot
  ],
};

// Families: one symbol and colour each; an unknown type still gets both.
assert.strictEqual(D.familyOf('deeptime:coal_seam').area, true);
assert.strictEqual(D.familyOf('deeptime:placer_au').shape, 'star');
const shapes = new Set(D.FAMILIES.map(f => f.shape)), colours = new Set(D.FAMILIES.map(f => f.colour));
assert.strictEqual(shapes.size, D.FAMILIES.length, 'a shape per family');
assert.strictEqual(colours.size, D.FAMILIES.length, 'a colour per family');
assert.strictEqual(D.familyOf('datapack:glow_stone').label, 'Glow stone');
assert.ok(D.FAMILIES.every(f => D.SHAPES[f.shape]));

// The model: counts, ranks inside a family, outlines, the event index.
const m = D.prepare(answer);
assert.strictEqual(m.counts.get('coal_seam'), 2);
assert.strictEqual(m.byId.get(1).rank, 1, 'the larger coal field ranks 1');
assert.strictEqual(m.byId.get(2).rank, 0);
assert.strictEqual(m.byId.get(3).rank, 0.5, 'alone in its family');
assert.strictEqual(m.byEvent.get(1003).id, 3);
assert.strictEqual(m.byId.get(1).poly.length, 28);
// The outline is the ellipse's, in blocks: its extreme is the long semi-axis (c = 60 blocks).
const reach = Math.max(...m.byId.get(1).poly.map(([dx, dz]) => Math.hypot(dx, dz)));
assert.ok(Math.abs(reach - 60) < 1e-6, `reach ${reach}`);

// Pointing: a canvas of the whole planet at the map's aspect (π : 1), as the workbench's map is.
const W = 2048, H = P.pictureHeight(W);
const view = {
  project: (lat, lon) => [P.xOfLonPic(P.wrapLon(lon), W), P.yOfLat(lat, H)],
  ppbx: W / C, ppbz: P.yOfZ(C, 1, H) - P.yOfZ(C, 0, H), worldW: W, dpr: 1,
};
const at = s => view.project(s.lat, s.lon);
const state = { hidden: new Set() };
let [x, y] = at(m.byId.get(3));
assert.strictEqual(D.hit(m, view, x + 3, y + 2, state).id, 3, 'a symbol is hit near its centre');
assert.strictEqual(D.hit(m, view, x + 40, y, state), null, 'and not far from it');
// A blanket under its porphyry is drawn a symbol's width over, and is hit there.
assert.strictEqual(m.byId.get(3).nudge, 0);
assert.strictEqual(m.byId.get(7).nudge, 1);
assert.strictEqual(D.hit(m, view, x - 1, y, state).id, 3);
assert.strictEqual(D.hit(m, view, x + 10, y, state).id, 7, 'the blanket beside its porphyry');
// A coal field is hit anywhere inside its area (the centre, and the long axis' end), not outside.
[x, y] = at(m.byId.get(1));
assert.strictEqual(D.hit(m, view, x, y, state).id, 1);
const k = Math.max(1, 3.5 / Math.max(40 * view.ppbx, 60 * view.ppbz));   // the area is drawn at least 3.5 px per semi-axis
const along = 0.8 * 60 * k * view.ppbx;
assert.strictEqual(D.hit(m, view, x + along * Math.cos(0.5 + Math.PI / 2), y + along * Math.sin(0.5 + Math.PI / 2), state).id, 1, 'along the long axis (c is the long one)');
assert.strictEqual(D.hit(m, view, x + 60, y, state), null);
// A symbol on the other side of the seam is hit from either edge.
const seam = m.byId.get(6);
const [sx, sy] = at(seam);
assert.ok(sx < 10, `the seam site is at x ${sx}`);
assert.strictEqual(D.hit(m, view, sx + 1, sy, state).id, 6);
assert.strictEqual(D.hit(m, view, sx + W - 1, sy, state).id, 6, 'seen across the seam (the right edge)');
// A hidden family is not hit, and not drawn.
assert.strictEqual(D.hit(m, view, ...at(m.byId.get(3)), { hidden: new Set(['porphyry_cu_mo_au']) }), null);

// Drawing: every shown coal field is one filled outline, every symbol one fill; nothing for hidden ones.
function recorder() {
  const log = [], props = {};
  const ctx = new Proxy({}, {
    get: (_, p) => (p in props ? props[p] : (...a) => { log.push([p, ...a]); }),
    set: (_, p, v) => { props[p] = v; return true; },
  });
  return { ctx, log };
}
let r = recorder();
D.draw(r.ctx, m, view, state);
const fills = r.log.filter(e => e[0] === 'fill').length;
assert.strictEqual(fills, 2 + 4 + 2, 'two coal fields, four symbols, and the seam site drawn at both edges (its symbol straddles the seam)');
r = recorder();
D.draw(r.ctx, m, view, { hidden: new Set(D.FAMILIES.map(f => f.key).concat(['glow_stone'])) });
assert.strictEqual(r.log.filter(e => e[0] === 'fill').length, 0);
// The picked site gets its footprint outline and the kin line (a placer and its porphyry).
r = recorder();
D.draw(r.ctx, m, view, { hidden: new Set(), selected: 4 });
assert.ok(r.log.some(e => e[0] === 'lineTo') && r.log.filter(e => e[0] === 'moveTo').length >= 2, 'footprint and the line to its porphyry');

// Text.
assert.strictEqual(D.fmtTonnes(1.98e10), '19.8 Gt');
assert.strictEqual(D.fmtTonnes(2.4e8), '240 Mt');
assert.strictEqual(D.fmtTonnes(40), '40.0 t');
assert.strictEqual(D.fmtArea(1298.4), '1,298 km²');
assert.strictEqual(D.fmtArea(7.25), '7.3 km²');
assert.strictEqual(D.sizeText(m.byId.get(1)), '19.8 Gt coal');
assert.strictEqual(D.fmtGrade(0.0044), '0.44 %');
assert.strictEqual(D.fmtGrade(1), '');
const tip = D.tipHTML(m.byId.get(3), '312.4 Myr ago');
assert.ok(tip.includes('Porphyry Cu-Mo-Au') && tip.includes('240 Mt copper') && tip.includes('formed 312.4 Myr ago') && tip.includes('with gold'), tip);
assert.ok(D.tipHTML(m.byId.get(1), '').includes('not in the event log'));
const leg = D.legendHTML(m, new Set(['coal_seam']), {});
assert.ok(leg.includes('5 of 7 sites'), leg);
assert.ok(/data-fam="coal_seam" aria-pressed="false"/.test(leg) && /data-fam="mvt_zn_pb"[^>]*>[\s\S]*?<span class="n">0<\/span>/.test(leg), 'a hidden family is off; an empty one lists 0');
assert.ok(leg.includes('data-fam="glow_stone"'), 'a datapack family is listed');
const rows = D.factRows(m.byId.get(3), m, { where: '12.0° N · 30° E', age: '312.4 Myr ago' }).map(r => r[0]);
assert.deepStrictEqual(rows.slice(0, 3), ['Family', 'Size', 'Formed']);
// The card's block is the world's (`block`, a cube bake's centre), not the simulation's
// x and z the overlay is drawn at; without one, x and z.
const blockRow = s => D.factRows(s, m, {}).find(r => r[0] === 'Block x, z')[1];
assert.strictEqual(blockRow(m.byId.get(3)), '4000, 900');
const pg = D.prepare(Object.assign({}, answer, { sites: [raw(15, 'deeptime:coal_seam', 7056, -1686, { block: [2391, -1829] })] }));
assert.strictEqual(blockRow(pg.byId.get(15)), '2391, -1829');

// The cause chain: breadth first, each once, to the depth asked, unknown ids skipped.
const ev = id => ({ id, causes: [] });
const byId = new Map([[1, Object.assign(ev(1), { causes: [2, 3, 99] })], [2, Object.assign(ev(2), { causes: [4] })],
  [3, Object.assign(ev(3), { causes: [4, 1] })], [4, Object.assign(ev(4), { causes: [5] })], [5, ev(5)]]);
assert.deepStrictEqual(D.causeChain(byId, 1, 4), [{ id: 2, depth: 1 }, { id: 3, depth: 1 }, { id: 4, depth: 2 }, { id: 5, depth: 3 }]);
assert.deepStrictEqual(D.causeChain(byId, 1, 2).map(c => c.id), [2, 3, 4]);

console.log('deposits.js: all checks passed');
