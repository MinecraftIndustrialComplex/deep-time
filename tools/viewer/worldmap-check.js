// SPDX-License-Identifier: LGPL-3.0-or-later
// The "Final world" view on mic-viewer's map core (decisions log 2026-10-21): Deep Time's plug-in
// (crates/deeptime-viewer/assets/worldmap.js, `DTWorldMap.frameOf`) hands the core its 2 : 1 tile
// pyramid, and the core must pick the same tile zoom and draw the same tiles in the same places as
// the view's own loop did before the move (worldmap.js at master b1b7336a, `tileZoom` and
// `drawLayer`, restated below). Random views on three planet sizes and both projections. Run on the
// test VM (nothing heavy on the laptop):
//
//   tools/remote/mac.sh -- 'nix shell nixpkgs#nodejs -c node tools/viewer/worldmap-check.js'
//
// Exits non-zero at the first failed assertion.
'use strict';
const assert = require('assert');
const path = require('path');
const fs = require('fs'), vm = require('vm');
const assets = path.join(__dirname, '../../crates/deeptime-viewer/assets');
const core = path.join(__dirname, '../../vendor/mic-viewer/web/map.js');
// The page's scripts at least compile.
for (const f of [path.join(assets, 'worldmap.js'), core]) new vm.Script(fs.readFileSync(f, 'utf8'), { filename: f });
require(path.join(assets, 'projection.js'));
const MV = require(core);
require(path.join(assets, 'worldmap.js'));
const { frameOf, T } = globalThis.DTWorldMap;
const proj = globalThis.DTProj;

// The view before the move: its zoom rule and its tile loop (worldmap.js at b1b7336a).
function oldZoom(C, maxZoom, bpp, dpr) {
  const s0 = C / (2 * T);
  const z = Math.ceil(Math.log2(s0 / (bpp / dpr)) - 0.3);
  return Math.max(0, Math.min(maxZoom, z));
}
function oldTiles(C, z, cx, cz, bpp, w, h) {
  const span = T * (C / (2 * T)) / (1 << z);
  const left = cx - w / 2 * bpp, top = cz - h / 2 * bpp;
  const k0 = Math.floor((left + C / 2) / span), k1 = Math.floor((left + w * bpp + C / 2) / span);
  const fz = proj.tileFrameZ(C);
  const j0 = Math.max(0, Math.floor((top - fz) / span)), j1 = Math.min((1 << z) - 1, Math.floor((top + h * bpp - fz) / span));
  const nx = 2 << z, out = [];
  for (let j = j0; j <= j1; j++) {
    for (let k = k0; k <= k1; k++) {
      out.push({ x: ((k % nx) + nx) % nx, y: j, sx: ((-C / 2 + k * span) - left) / bpp, sy: ((fz + j * span) - top) / bpp });
    }
  }
  return out;
}
const key = t => `${t.x}/${t.y}@${t.sx.toFixed(6)},${t.sy.toFixed(6)}`;

let seed = 12345;
const rnd = () => ((seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648);
let views = 0, tiles = 0;
for (const C of [4096, 13312, 32768]) {
  for (const projection of [null, { kind: 'petroff_guyou', layout_version: 1, hemisphere_blocks: C / 2, lambda_c_deg: 71.5 }]) {
    const info = { circumference: C, max_zoom: Math.round(Math.log2(C / 512)) + 1, projection };
    const f = frameOf(info);
    const pyr = f.pyramid;
    assert.strictEqual(f.projection.period, C);
    assert.strictEqual(pyr.cols(0), 2);
    assert.deepStrictEqual(pyr.rows(3), [0, 7]);
    for (let i = 0; i < 400; i++) {
      const w = 200 + Math.floor(rnd() * 1800), h = 200 + Math.floor(rnd() * 1200), dpr = [1, 1.5, 2][i % 3];
      const bpp = Math.pow(2, -2 + rnd() * (Math.log2(C / w) + 2));
      const cx = (rnd() - 0.5) * C * 1.3, cz = (rnd() - 0.5) * C * 0.45;
      const z = MV.zoomFor(pyr, bpp / dpr);
      assert.strictEqual(z, oldZoom(C, info.max_zoom, bpp, dpr), `zoom C=${C} bpp=${bpp} dpr=${dpr}`);
      const left = cx - w / 2 * bpp, top = cz - h / 2 * bpp;
      const now = MV.tileCover(pyr, z, left, top, w * bpp, h * bpp)
        .map(c => ({ x: c.x, y: c.y, sx: (c.bx - left) / bpp, sy: (c.bz - top) / bpp }));
      const was = oldTiles(C, z, cx, cz, bpp, w, h);
      assert.deepStrictEqual(now.map(key).sort(), was.map(key).sort(), `tiles C=${C} view ${cx},${cz},${bpp} ${w}x${h}`);
      views++; tiles += now.length;
    }
  }
}
console.log(`worldmap-check: ok (${views} views, ${tiles} tiles: the core draws the tiles the view drew before the move)`);
