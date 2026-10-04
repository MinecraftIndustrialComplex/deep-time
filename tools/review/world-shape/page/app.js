// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time world-shape page: maps, tables and charts. Data: data.js (tools/review/world-shape/compute.py).
'use strict';
(function () {
const D = window.WS_DATA;
if (/(^|[#,])theme-dark/.test(location.hash)) document.documentElement.dataset.theme = 'dark';
const $ = (s, r = document) => r.querySelector(s);
const el = (tag, attrs = {}, html) => {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  if (html != null) e.innerHTML = html;
  return e;
};
const NS = 'http://www.w3.org/2000/svg';
const sv = (tag, attrs = {}, text) => {
  const e = document.createElementNS(NS, tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  if (text != null) e.textContent = text;
  return e;
};
const fmt = (n, d = 0) => Number(n).toLocaleString('en-US', { minimumFractionDigits: d, maximumFractionDigits: d });
const css = (v) => getComputedStyle(document.documentElement).getPropertyValue(v).trim();
const KCOL = { slide: '--slide', quarter: '--quarter', half: '--half' };
const KNAME = { slide: 'slide (translation)', quarter: 'quarter-turn', half: 'half-turn' };
const R16 = D.R16;

// ---------------------------------------------------------------------------------------------
// The candidates. Numbers: docs/research/notes/seamless-projection-survey.md §3 (area as blocks per
// planet area relative to the mean: min / p1 / median / p99 / max; shape ratio a/b median / max).
const C = {
  'lambert': {
    name: 'Lambert cylindrical (today)', short: 'Lambert', fam: 'now', status: 'now', statusText: 'shipped',
    verdict: 'The reference. Equal-area and simple, but not seamless: the poles are walls.',
    area: 'exactly 1', areaNum: [1, 1], shape: [1.33, 27], shapeNote: 'at the 80° stop line; p90 4.8',
    om: '29.0° / 137°',
    corners: 'none; two pole caps (band from 70°, stop at 80°: 1.5 % of the planet unreachable)',
    seams: '1 slide (the x wrap)', seamLen: 5136,
    smooth: 'Smooth across the x wrap; no creases. The pole band is a wall.',
    effort: ['0', '—', '—'],
    eq: 'Straight, 16,384 blocks round, through Toroidal World\'s one slide.',
    pole: 'Storm band from 70°, hard stop at 80° (about 118 blocks of band at 16k). No pole.',
    pros: ['Shipped and tested; nothing to build.', 'Toroidal World handles the only seam with all its compatibility work (Create, Sable, C2ME, Distant Horizons).',
      'True scale on the equator; every reader works on a plain 2:1 strip.', 'No portals and no new dependency.'],
    cons: ['Not seamless: no pole to walk over; 1.5 % of the planet is beyond the stop line.', 'Shapes up to 27× too long near the stop line (4.8× for the worst tenth of the planet).',
      'Its rows resolve the polar caps badly, which also streaks any later resample (visible on every map here).'],
  },
  'guyou-peirce': {
    name: 'Guyou conformal (the prototype)', short: 'Guyou conformal', fam: 'pill', status: 'run', statusText: 'prototype runs',
    verdict: 'Runs in game today, but it is not equal-area: land near its four points is drawn without bound larger.',
    area: '0.46 / 0.46 / 0.79 / 4.6 / ∞', areaNum: [0.46, Infinity], shape: [1, 1], om: '0 (conformal)',
    corners: '4 × 180° missing, at ±45° latitude (the long edges\' midpoints and the rectangle\'s corners)',
    seams: '3 pairs: 1 slide + 2 half-turns', seamLen: 19632,
    smooth: 'Analytic across every seam; no creases.',
    effort: ['built (prototype)', '26–40', '46–75'],
    eq: 'Straight, 13,088 blocks round, one slide.',
    pole: 'The pole is on the long edge: the walk comes out heading south on the far side (half-turn), as over a real pole.',
    pros: ['Already walked across every seam in a real client (prototype §5).', 'Shapes are true everywhere.', 'Keeps a 2:1 grid and the x wrap; straight equator.'],
    cons: ['Not equal-area (Ben, 2026-10-02: "Use an equal area projects FFS"): 1 % of the planet drawn more than 4.6× too large, unbounded at the four points.', 'Strong corners: two walkers passing either side of one end up going opposite ways.'],
  },
  'peirce-square': {
    name: 'Peirce quincuncial square', short: 'Peirce square', fam: 'pill', status: 'out', statusText: 'Guyou chosen',
    verdict: 'The same map as Guyou in its own aspect. Guyou was chosen over it on 2026-10-03.',
    area: 'as Guyou conformal', areaNum: [0.46, Infinity], shape: [1, 1], om: '0',
    corners: '4 × 180° on the equator (the edge midpoints); the south pole is the four corners',
    seams: '4 half-turns (each edge onto itself)', seamLen: 18496,
    smooth: 'Analytic across every seam.',
    effort: ['built (prototype)', '≈ Guyou + 4 turning edges', '—'],
    eq: 'Not a straight walk: four legs with a 90° turn at each singular point.',
    pole: 'The north pole is an ordinary point in the middle; the south pole is where two turning seams meet.',
    pros: ['The north pole is an ordinary walkable point.', 'Singular points fall on the equator, where Earth-like planets have less land.'],
    cons: ['Four turning seams and a square that fits none of Deep Time\'s grids.', 'No straight equator walk.', 'Same area blow-up as Guyou conformal.'],
  },
  'guyou-petroff': {
    name: 'Petroff 2021 in Guyou\'s layout', short: 'Petroff (Guyou)', fam: 'pill', status: 'alt', statusText: 'runner-up',
    verdict: 'The cheapest seamless equal-area world: the prototype with only its map swapped. Shapes are about twice as distorted as the cube\'s.',
    area: 'exactly 1', areaNum: [1, 1], shape: [1.78, 2.67], om: '30.9° / 54°',
    corners: '4 × 180° missing, at ±45° latitude',
    seams: '3 pairs: 1 slide + 2 half-turns', seamLen: 19632,
    smooth: 'Smooth across all seams (ε = 0); small creases inside each hemisphere (≤ 5°, ε ≤ 0.09) on the tile midlines and diagonals.',
    effort: ['1–2', '26–40', '46–75'],
    eq: 'Straight, 13,088 blocks round (the equator drawn 20 % short), one slide.',
    pole: 'As the prototype: over the pole through a half-turn, heading south.',
    pros: ['Only peirce.rs changes: the prototype\'s seams, portal table, resampler and walks all carry over.', 'Exactly equal-area, closed form both ways, CC0 reference code.',
      'Fewest seams of the seamless options (3 pairs, 19,632 blocks) and only 4 corners: fewer portals, fewer three-way corners.', 'Keeps a 2:1 grid with Toroidal World\'s slide, matching "TW slides, engine turns".'],
    cons: ['Shapes up to 2.67× too long; half the planet worse than 1.78× (the cube: 1.26 / 1.42).', 'Four strong corners: the 2× bound is forced near each, and walkers either side of one end up going opposite ways.',
      'The equator is drawn 20 % shorter than today\'s 16,384 blocks.'],
  },
  'guyou-gringorten': {
    name: 'Gringorten 1972 in Guyou\'s layout', short: 'Gringorten', fam: 'pill', status: 'out', statusText: 'ruled out',
    verdict: 'Equal-area, but creased exactly on its seams.',
    area: 'exactly 1', areaNum: [1, 1], shape: [1.79, 2.88], om: '29.7° / 57.9°',
    corners: '4 × 180° at ±45°', seams: '3 pairs: 1 slide + 2 half-turns', seamLen: 19632,
    smooth: 'A 62° crease along the seams (ε 1.9); diagonals 20°.', effort: ['1–2', '26–40', '46–75'],
    eq: 'As Petroff.', pole: 'As Petroff.',
    pros: ['Published, equal-area; ISC-licensed d3 port.'], cons: ['Its crease lies on the seams, where continuity matters most.', 'Worse than Petroff on every number; iterative (2 µs a block).'],
  },
  'guyou-collignon': {
    name: 'Collignon quincuncial in Guyou\'s layout', short: 'Collignon', fam: 'pill', status: 'out', statusText: 'ruled out',
    verdict: 'Trivial closed form, worst shapes and creases everywhere.',
    area: 'exactly 1', areaNum: [1, 1], shape: [2.05, 2.98], om: '38.8° / 59.7°',
    corners: '4 × 180° at ±45°', seams: '3 pairs: 1 slide + 2 half-turns', seamLen: 19632,
    smooth: '63° creases on every seam and diagonal (ε 2.0).', effort: ['1', '26–40', '46–75'],
    eq: 'As Petroff.', pole: 'As Petroff.',
    pros: ['Simplest equal-area square (Lambert azimuthal + a concentric square).'], cons: ['Worst shape distortion of all; folded along every seam and diagonal.'],
  },
  'cube-moser-cross': {
    name: 'Equal-area smooth cube (Moser), cross net', short: 'Moser cube', fam: 'cube', status: 'rec', statusText: 'recommended',
    verdict: 'The least distortion a block-grid world can have, no creases, mild corners. New: needs its own tests.',
    area: '1.000 (table error ≤ 0.6 %)', areaNum: [1, 1], shape: [1.26, 1.42], om: '12.1° / 19.9°',
    corners: '8 × 90° missing, at ±35.3° latitude (λ₀ + 45° + k·90°)',
    seams: '7 pairs: 1 slide + 4 quarter-turns + 2 half-turns', seamLen: 26432,
    smooth: 'No creases: slope jumps ≤ 0.02 (0.6°) at every seam and face edge, the table\'s numerical floor.',
    effort: ['8–11', '31–50', '≈ 49–83'],
    eq: 'Straight, 15,104 blocks round (four faces), one slide (16,384 with 4,096-block faces).',
    pole: 'The pole is the polar face\'s centre, an ordinary point. From face 0 the walk crosses it and leaves through a half-turn onto face 2; from faces 1 and 3 it enters and leaves through quarter-turns.',
    pros: ['Shapes at most 1.42× (bound 1.33×); 2× better than any pillowcase.', 'No creases anywhere: ridges, rivers and coasts never kink.', 'Corners are mild and magnify nothing, so they need not be hidden at sea.',
      'Straight wrapped equator; both poles are ordinary points.', 'The band\'s east–west wrap is a plain slide: Toroidal World can keep it.'],
    cons: ['A new construction (conformal cube + Moser\'s flow), never published: convergence and seam tests are on us.', 'More seams: 7 pairs (14 one-way portals), 26,432 blocks of seam, 8 three-way corners.',
      'Quarter-turn portals are untested in Immersive Portals (the rotation NBT is inferred).', 'Every reader must handle a cross-shaped world: FarTerrain, Distant Horizons, maps, the climate API\'s latitude.', 'Needs a table (≈ 0.4 MB) or its generator in the build.'],
  },
  'cube-moser-3x2': {
    name: 'Equal-area smooth cube, 3 × 2 rectangle', short: 'Moser 3 × 2', fam: 'cube', status: 'alt', statusText: 'net option',
    verdict: 'The same map packed into a rectangle: no wasted raster, one more seam pair, and seams inside the world.',
    area: 'as the Moser cube', areaNum: [1, 1], shape: [1.26, 1.42], om: '12.1° / 19.9°',
    corners: '8 × 90° missing', seams: '8 pairs: 2 slides + 4 quarter-turns + 2 half-turns', seamLen: 30208,
    smooth: 'As the Moser cube.', effort: ['8–11', '32–52', 'engine owns all 8 pairs'],
    eq: 'Straight in game, but crosses two seams per lap (into face 3 by a quarter-turn and back to face 0).',
    pole: 'Two seams on the way over a pole.',
    pros: ['One 3s × 2s rectangle with no unused raster.', 'Same distortion as the cross.'],
    cons: ['8 seam pairs (30,208 blocks); the line between the rows carries six seam edges.', 'No world-wide east–west slide: its two slides are pieces of the inner line and a diagonal jump, so Toroidal World cannot own any edge.', 'Two seams per equator lap.'],
  },
  'cube-qsc-cross': {
    name: 'QSC = Roşca–Plonka equal-area cube, cross net', short: 'QSC cube', fam: 'cube', status: 'alt', statusText: 'fallback',
    verdict: 'Published, closed form, exactly equal-area; folds every ridge 41° on each face diagonal.',
    area: 'exactly 1', areaNum: [1, 1], shape: [1.32, 1.55], om: '16.0° / 24.8°',
    corners: '8 × 90° at ±35.3°', seams: '7 pairs: 1 slide + 4 quarter + 2 half', seamLen: 26432,
    smooth: 'Creased: face diagonals ε 0.87 (41°) everywhere, face edges ≤ 13°. Switch on the crease test to see it.',
    effort: ['5–8', '31–50', '≈ 49–83'],
    eq: 'As the Moser cube.', pole: 'As the Moser cube.',
    pros: ['Published (O\'Neill & Laubscher 1976; Roşca & Plonka 2011), in PROJ (MIT).', 'Exactly equal-area, closed form, 16 ns a block.', 'Same seams as the Moser cube: the natural fallback if the table fails its tests.'],
    cons: ['A straight ridge or coast crossing a face diagonal kinks by 41°; a volcano on a diagonal becomes a teardrop.', 'Slightly more distortion than the Moser cube (1.32 / 1.55).'],
  },
  'cube-healpix-cross': {
    name: 'HEALPix / rHEALPix cube, cross net', short: 'HEALPix', fam: 'cube', status: 'out', statusText: 'ruled out',
    verdict: 'Best median shape (its band is cylindrical), worst creases of the equal-area cubes.',
    area: 'exactly 1', areaNum: [1, 1], shape: [1.18, 2.41], om: '15.7° / 48.9°',
    corners: '8 × 90° at ±41.8°', seams: '7 pairs (or 9 at 45° in its 2:1 form)', seamLen: 26432,
    smooth: 'Polar diagonals ε 2.0 (63°), polar face edges ε ≤ 0.96 (44°).', effort: ['5–8', '31–50', '≈ 49–83'],
    eq: 'As the Moser cube.', pole: 'As the Moser cube.',
    pros: ['Its tropics are Lambert cylindrical, like today.', 'Standard in astronomy; MIT code in PROJ.'], cons: ['63° creases across the polar faces.', 'Polar shapes up to 2.41×.'],
  },
  'cube-eac-cross': {
    name: 'Equi-angular cube (S2 tangent), cross net', short: 'Equi-angular', fam: 'cube', status: 'out', statusText: 'ruled out',
    verdict: 'The graphics cube: cheap and close to uniform, but not equal-area and creased at every edge.',
    area: '0.85 / 0.85 / 1.00 / 1.18 / 1.20', areaNum: [0.85, 1.2], shape: [1.2, 1.73], om: '11.0° / 30.9°',
    corners: '8 × 90°', seams: '7 pairs', seamLen: 26432,
    smooth: 'Every face edge creased 44–47° (ε ≈ 1).', effort: ['4–7', '31–50', '≈ 49–83'],
    eq: 'As the Moser cube.', pole: 'As the Moser cube.',
    pros: ['Two tangents per block; used by Google\'s VR video and S2 (Apache-2.0).'], cons: ['Not equal-area (±20 %).', 'Kinks at every face edge.'],
  },
  'cube-conformal-cross': {
    name: 'Conformal cube, cross net', short: 'Conformal cube', fam: 'cube', status: 'out', statusText: 'not equal-area',
    verdict: 'The Moser cube\'s parent: true shapes and smooth, but area grows without bound at the corners.',
    area: '0.79 / 0.79 / 0.92 / 2.17 / ∞', areaNum: [0.79, Infinity], shape: [1, 1], om: '0',
    corners: '8 × 90°', seams: '7 pairs', seamLen: 26432,
    smooth: 'Analytic across every edge.', effort: ['6–9', '31–50', '≈ 49–83'],
    eq: 'As the Moser cube.', pole: 'As the Moser cube.',
    pros: ['Conformal and smooth; climate models use it.', 'Much milder corners than Peirce (area like r^−⅔, not r⁻²).'], cons: ['Not equal-area: 13 % of the planet off by more than ×1.25.'],
  },
};
const ORDER = { now: ['lambert'], pill: ['guyou-petroff', 'guyou-peirce', 'peirce-square', 'guyou-gringorten', 'guyou-collignon'],
  cube: ['cube-moser-cross', 'cube-moser-3x2', 'cube-qsc-cross', 'cube-healpix-cross', 'cube-eac-cross', 'cube-conformal-cross'] };
const FAMNAME = { now: 'Today', pill: 'Pillowcase', cube: 'Cube' };

// ---------------------------------------------------------------------------------------------
// decision cards
function decisionCards() {
  const box = $('#decision-cards');
  const cards = [
    { h: 'World shape', q: 'Which flat world holds the planet?', opts: [
      ['rec', 'Equal-area smooth cube (Moser)', 'Shapes ≤ 1.42×, no creases, 8 mild corners. QSC is its published fallback.'],
      ['alt', 'Petroff in Guyou\'s 2:1 layout', 'Least work (1–2 days on the prototype), 3 seam pairs, but shapes up to 2.67× and 4 strong corners.'],
      ['now', 'Keep Lambert', 'No seamless world: pole band and stop line stay.']] },
    { h: 'Cube face size', q: 'Blocks along a face edge at 16k (32k).', opts: [
      ['rec', '3,776 (7,552): today\'s scale', 'Same blocks per km² as today; every calibrated scale stays; edges and turn centres on chunk lines.'],
      ['alt', '4,096 (8,192): powers of two', 'Region-aligned, band = today\'s 16,384; but 1.18× more blocks for the same planet (300 → 277 m a block).']] },
    { h: 'Cube net', q: 'How the six faces are unfolded.', opts: [
      ['rec', 'Cross', '7 seam pairs, straight equator with one slide (Toroidal World can keep it), poles at face centres.'],
      ['alt', '3 × 2 rectangle', '8 pairs, seams inside the rectangle, two seams per equator lap, no world-wide slide; packs the raster without waste.']] },
  ];
  for (const c of cards) {
    const d = el('div', { class: 'dcard' });
    d.appendChild(el('h3', {}, c.h));
    d.appendChild(el('div', { class: 'q' }, c.q));
    for (const [k, t, s] of c.opts) {
      const tag = k === 'rec' ? '<span class="tag rec">recommended</span>' : k === 'alt' ? '<span class="tag alt">alternative</span>' : '<span class="tag now">today</span>';
      d.appendChild(el('div', { class: 'opt' + (k === 'rec' ? ' rec' : '') }, `<div>${tag}<br><b>${t}</b><br><span class="note">${s}</span></div>`));
    }
    box.appendChild(d);
  }
}

// ---------------------------------------------------------------------------------------------
// cone-point figure
function coneFigure() {
  const W = 540, H = 200;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': 'Discs of 360, 270 and 180 degrees' });
  const items = [[360, 'ordinary point', '360°'], [270, 'cube corner', '270° (90° missing)'], [180, 'pillowcase corner', '180° (180° missing)']];
  items.forEach(([deg, lab, sub], i) => {
    const cx = 90 + i * 180, cy = 92, r = 62;
    const a0 = -Math.PI / 2, a1 = a0 + deg * Math.PI / 180;
    let d;
    if (deg >= 360) d = `M ${cx - r} ${cy} a ${r} ${r} 0 1 0 ${2 * r} 0 a ${r} ${r} 0 1 0 ${-2 * r} 0`;
    else d = `M ${cx} ${cy} L ${cx + r * Math.cos(a0)} ${cy + r * Math.sin(a0)} A ${r} ${r} 0 ${deg > 180 ? 1 : 0} 1 ${cx + r * Math.cos(a1)} ${cy + r * Math.sin(a1)} Z`;
    s.appendChild(sv('path', { d, fill: i === 0 ? css('--panel-2') : (i === 1 ? css('--accent-bg') : css('--bad-bg')), stroke: css('--ink-2'), 'stroke-width': 1.2 }));
    if (deg < 360) {
      const col = i === 1 ? css('--quarter') : css('--half');
      s.appendChild(sv('line', { x1: cx, y1: cy, x2: cx + r * Math.cos(a0), y2: cy + r * Math.sin(a0), stroke: col, 'stroke-width': 3 }));
      s.appendChild(sv('line', { x1: cx, y1: cy, x2: cx + r * Math.cos(a1), y2: cy + r * Math.sin(a1), stroke: col, 'stroke-width': 3 }));
      s.appendChild(sv('circle', { cx, cy, r: 5, fill: css('--corner'), stroke: '#000', 'stroke-width': 1.2 }));
    }
    for (let k = 1; k <= 3; k++) s.appendChild(sv('circle', { cx, cy, r: r * k / 4, fill: 'none', stroke: css('--ink-3'), 'stroke-width': .6, 'stroke-dasharray': '2 3' }));
    s.appendChild(sv('text', { x: cx, y: 176, 'text-anchor': 'middle', 'font-size': 14, 'font-weight': 600, fill: css('--ink') }, lab));
    s.appendChild(sv('text', { x: cx, y: 194, 'text-anchor': 'middle', 'font-size': 12, fill: css('--ink-3') }, sub));
  });
  $('#cone-fig').appendChild(s);
}

// ---------------------------------------------------------------------------------------------
// map overlay builder (world coordinates are blocks with Y up = north; SVG y = -Y)
function overlaySvg(id, opts) {
  const L = D.layouts[id];
  const [bx0, by0, bx1, by1] = L.bbox;
  const bw = bx1 - bx0, bh = by1 - by0;
  const u = Math.max(bw, bh);
  const s = sv('svg', { viewBox: `${bx0} ${-by1} ${bw} ${bh}`, preserveAspectRatio: 'none' });
  const P = (p) => [p[0], -p[1]];
  const lw = u / 380;
  if (opts.tissot) {
    const g = sv('g', { fill: 'rgba(255,255,255,.18)', stroke: 'rgba(255,255,255,.85)', 'stroke-width': lw * .5 });
    for (const [x, y, ra, rb, ang] of L.tissot) {
      g.appendChild(sv('ellipse', { cx: x, cy: -y, rx: ra, ry: rb, transform: `rotate(${-ang} ${x} ${-y})` }));
    }
    s.appendChild(g);
  }
  if (opts.extra && L.extra && L.extra.band70) {
    const x0 = L.tiles[0][0], w = L.tiles[0][2];
    for (const sg of [1, -1]) {
      const yA = sg * L.extra.band70, yB = sg * L.extra.stop80;
      s.appendChild(sv('rect', { x: x0, y: Math.min(-yA, -yB), width: w, height: Math.abs(yB - yA), fill: 'rgba(240,120,90,.28)' }));
      s.appendChild(sv('line', { x1: x0, x2: x0 + w, y1: -yB, y2: -yB, stroke: css('--bad'), 'stroke-width': lw * 1.4 }));
      s.appendChild(sv('text', { x: x0 + w * 0.01, y: -yB + (sg > 0 ? -1 : 1) * u * 0.012 + (sg > 0 ? 0 : u * 0.02), 'font-size': u * 0.018, fill: '#fff' }, sg > 0 ? 'storm band from 70°, stop at 80° N' : 'stop at 80° S'));
    }
  }
  if (opts.walks && L.walks) {
    const cols = { equator: css('--walk-eq'), pole: css('--walk-pole'), poleB: css('--walk-pole'), cornerA: css('--walk-a'), cornerB: css('--walk-b') };
    const which = opts.walks === 'corner' ? ['cornerA', 'cornerB'] : ['equator', 'pole'];
    for (const k of which) {
      const w = L.walks[k];
      if (!w) continue;
      for (const sg of w.segs) {
        s.appendChild(sv('polyline', { points: sg.map((p) => P(p).join(',')).join(' '), fill: 'none', stroke: '#000', 'stroke-opacity': .45, 'stroke-width': lw * 3.2, 'stroke-linecap': 'round' }));
        s.appendChild(sv('polyline', { points: sg.map((p) => P(p).join(',')).join(' '), fill: 'none', stroke: cols[k], 'stroke-width': lw * 1.9, 'stroke-linecap': 'round' }));
        // heading arrow at the segment's middle
        if (sg.length > 2) {
          const m = Math.floor(sg.length / 2);
          arrowHead(s, P(sg[m - 1]), P(sg[m]), cols[k], u * 0.016);
        }
      }
      w.crossings.forEach((c, i) => {
        for (const p of [c.at, c.to]) {
          s.appendChild(sv('circle', { cx: p[0], cy: -p[1], r: u * 0.008, fill: cols[k], stroke: '#fff', 'stroke-width': lw * .7 }));
        }
      });
      if (w.stopped) s.appendChild(sv('rect', { x: w.stopped[0] - u * 0.008, y: -w.stopped[1] - u * 0.008, width: u * 0.016, height: u * 0.016, fill: css('--bad'), stroke: '#fff', 'stroke-width': lw * .7 }));
    }
  }
  if (opts.seams) {
    const g = sv('g', {});
    const letters = 'ABCDEFGHIJ';
    for (const sm of L.seams) {
      const col = css(KCOL[sm.kind]);
      let a = P(sm.a0), b = P(sm.a1);
      g.appendChild(sv('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1], stroke: '#000', 'stroke-opacity': .55, 'stroke-width': lw * 3.4 }));
      g.appendChild(sv('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1], stroke: col, 'stroke-width': lw * 2.2 }));
      // arrows show which way round the two edges meet: A side a0 -> a1, B side reversed
      const [p, q] = sm.side === 'A' ? [b, a] : [a, b];
      const t = 0.62;
      const m1 = [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t];
      const m0 = [p[0] + (q[0] - p[0]) * (t - 0.02), p[1] + (q[1] - p[1]) * (t - 0.02)];
      arrowHead(g, m0, m1, col, u * 0.02, '#000');
      // pair letter, just inside the edge
      const dx = b[0] - a[0], dy = b[1] - a[1], len = Math.hypot(dx, dy);
      const nx = dy / len, ny = -dx / len; // in SVG coords (y down) the inside of a ccw edge is to this side
      const tp = [a[0] + dx * 0.3 + nx * u * 0.022, a[1] + dy * 0.3 + ny * u * 0.022];
      g.appendChild(sv('circle', { cx: tp[0], cy: tp[1], r: u * 0.014, fill: col, stroke: '#fff', 'stroke-width': lw * .6 }));
      g.appendChild(sv('text', { x: tp[0], y: tp[1] + u * 0.0055, 'text-anchor': 'middle', 'font-size': u * 0.016, 'font-weight': 700, fill: '#fff' }, letters[sm.pair]));
    }
    s.appendChild(g);
  }
  if (opts.corners) {
    const g = sv('g', {});
    for (const [x, y, v, cone] of L.corners) {
      g.appendChild(sv('circle', { cx: x, cy: -y, r: u * 0.011, fill: css('--corner'), stroke: '#000', 'stroke-width': lw * .9 }));
      g.appendChild(sv('text', { x, y: -y + u * 0.004, 'text-anchor': 'middle', 'font-size': u * 0.011, 'font-weight': 700, fill: '#000' }, String(v + 1)));
    }
    if (L.extra && L.extra.southpole) {
      for (const [x, y] of L.extra.southpole) g.appendChild(sv('text', { x: x + (x > 0 ? -1 : 1) * u * 0.035, y: -y + (y > 0 ? 1 : -0.4) * u * 0.03, 'text-anchor': 'middle', 'font-size': u * 0.017, fill: '#fff' }, 'S pole'));
    }
    s.appendChild(g);
  }
  return s;
}
function arrowHead(g, a, b, col, size, stroke) {
  const dx = b[0] - a[0], dy = b[1] - a[1], L = Math.hypot(dx, dy) || 1;
  const ux = dx / L, uy = dy / L, px = -uy, py = ux;
  const tip = b, bl = [b[0] - ux * size + px * size * 0.55, b[1] - uy * size + py * size * 0.55], br = [b[0] - ux * size - px * size * 0.55, b[1] - uy * size - py * size * 0.55];
  g.appendChild(sv('polygon', { points: [tip, bl, br].map((p) => p.join(',')).join(' '), fill: col, stroke: stroke || 'none', 'stroke-width': size * 0.08 }));
}

// ---------------------------------------------------------------------------------------------
// explorer
const state = { id: 'cube-moser-cross', layer: 'map', ov: new Set(['seams', 'corners']) };
const LAYERS = [['map', 'Planet'], ['area', 'Area'], ['shape', 'Shape'], ['crease', 'Crease test']];
const OVS = [['seams', 'Seams'], ['corners', 'Corners'], ['grat', 'Graticule'], ['tissot', 'Tissot'], ['walks', 'Walks'], ['corner', 'Corner walkers']];
function picker(box, ids, onPick, current, idPrefix) {
  for (const fam of ['now', 'pill', 'cube']) {
    const row = el('div', { class: 'row' });
    row.appendChild(el('span', { class: 'fam' }, FAMNAME[fam]));
    for (const id of ORDER[fam]) {
      if (!ids.includes(id)) continue;
      const c = C[id];
      const b = el('button', { class: 'chip', type: 'button', 'aria-pressed': String(id === current), id: idPrefix + id }, `<span class="dot${c.status === 'rec' ? ' rec' : ''}"></span>${c.short}`);
      b.addEventListener('click', () => { box.querySelectorAll('.chip').forEach((x) => x.setAttribute('aria-pressed', 'false')); b.setAttribute('aria-pressed', 'true'); onPick(id); });
      row.appendChild(b);
    }
    box.appendChild(row);
  }
}
function setupExplorer() {
  const hx = location.hash.match(/ex-([a-z0-9-]+?)-(map|area|shape|crease)(?:-([a-z,]+))?(?:,|$)/);
  if (hx && D.layouts[hx[1]]) { state.id = hx[1]; state.layer = hx[2]; if (hx[3]) state.ov = new Set(hx[3].split(',')); }
  picker($('#picker'), Object.keys(D.layouts), (id) => { state.id = id; renderMap(); }, state.id, 'pick-');
  const seg = $('#layer-seg');
  for (const [k, t] of LAYERS) {
    const b = el('button', { type: 'button', 'aria-pressed': String(k === state.layer), id: 'layer-' + k }, t);
    b.addEventListener('click', () => { state.layer = k; seg.querySelectorAll('button').forEach((x) => x.setAttribute('aria-pressed', String(x === b))); renderMap(); });
    seg.appendChild(b);
  }
  const ch = $('#overlay-checks');
  for (const [k, t] of OVS) {
    const lab = el('label', {}, `<input type="checkbox" id="ov-${k}" ${state.ov.has(k) ? 'checked' : ''}> ${t}`);
    lab.querySelector('input').addEventListener('change', (e) => { e.target.checked ? state.ov.add(k) : state.ov.delete(k); renderMap(); });
    ch.appendChild(lab);
  }
  renderMap();
}
function mapStack(box, id, layer, ov) {
  const L = D.layouts[id];
  const [bx0, by0, bx1, by1] = L.bbox;
  box.innerHTML = '';
  box.style.aspectRatio = `${bx1 - bx0} / ${by1 - by0}`;
  box.appendChild(el('img', { src: `${id}-${layer}.webp`, alt: `${C[id].name}: ${layer}`, loading: 'lazy' }));
  if (ov.has('grat')) box.appendChild(el('img', { src: `${id}-grat.webp`, alt: '', class: 'grat', loading: 'lazy' }));
  box.appendChild(overlaySvg(id, { seams: ov.has('seams'), corners: ov.has('corners'), tissot: ov.has('tissot'), walks: ov.has('walks') ? 'main' : (ov.has('corner') ? 'corner' : null), extra: true }));
}
function renderMap() {
  const id = state.id, c = C[id], L = D.layouts[id];
  mapStack($('#mapbox'), id, state.layer, state.ov);
  if (state.ov.has('walks') && state.ov.has('corner')) {
    $('#mapbox').appendChild(overlaySvg(id, { walks: 'corner' }));
  }
  // legend
  const lg = $('#map-legend');
  let h = '';
  if (state.layer === 'area') h += `<span>blocks per planet area: <span class="note" style="color:inherit">¼×</span><span class="bar" style="background:linear-gradient(90deg,#2166ac,#67a9cf,#d1e5f0,#f7f7f7,#fddbc7,#ef8a62,#b2182b)"></span>4× <span style="opacity:.75">(white = equal-area; red = planet drawn larger)</span></span>`;
  if (state.layer === 'shape') h += `<span>shape ratio a/b: 1<span class="bar" style="background:linear-gradient(90deg,#fafaf0,#fee8a0,#fdae61,#e6613c,#be2846,#78145a,#320a46)"></span>4+</span>`;
  if (state.layer === 'crease') h += `<span>hill shade of a smooth made-up planet: straight lines of broken shading are creases</span>`;
  if (state.ov.has('seams')) h += ['slide', 'quarter', 'half'].map((k) => `<span><span class="sw" style="background:var(${KCOL[k]})"></span>${KNAME[k]}</span>`).join('') + '<span>letters pair the glued edges; arrows run the same way along both</span>';
  if (state.ov.has('corners')) h += `<span><span class="cn"></span>corner (copies share a number)</span>`;
  if (state.ov.has('walks')) h += `<span><span class="sw" style="background:var(--walk-eq)"></span>equator walk</span><span><span class="sw" style="background:var(--walk-pole)"></span>over the north pole</span><span>dots: leaving and re-entering through a seam</span>`;
  if (state.ov.has('corner')) h += `<span><span class="sw" style="background:var(--walk-a)"></span><span class="sw" style="background:var(--walk-b)"></span>two walkers either side of a corner</span>`;
  if (state.ov.has('tissot')) h += `<span>ellipses: a 6° disc of planet as drawn</span>`;
  lg.innerHTML = h;
  // panel
  const seamA = L.seams.filter((s) => s.side === 'B');
  const kinds = seamA.reduce((m, s) => (m[s.kind] = (m[s.kind] || 0) + 1, m), {});
  const tagc = { rec: 'rec', alt: 'alt', run: 'run', now: 'now', out: 'out' }[c.status];
  $('#cand-panel').innerHTML = `
    <div class="eyebrow">${FAMNAME[c.fam]} family</div>
    <h3>${c.name}</h3>
    <span class="tag ${tagc}">${c.statusText}</span>
    <p>${c.verdict}</p>
    <dl class="kv">
      <dt>Area</dt><dd class="num">${c.area}</dd>
      <dt>Shape a/b</dt><dd class="num">median ${c.shape[0]} · max ${c.shape[1]}${c.shapeNote ? ` <span class="note">(${c.shapeNote})</span>` : ''}</dd>
      <dt>Corners</dt><dd>${c.corners}</dd>
      <dt>Seams</dt><dd>${c.seams}: ${Object.entries(kinds).map(([k, n]) => `<span class="kind ${k}">${n} ${k}</span>`).join(', ')}; <span class="num">${fmt(c.seamLen)}</span> blocks at 16k</dd>
      <dt>Smoothness</dt><dd>${c.smooth}</dd>
      <dt>Effort</dt><dd class="num">projection ${c.effort[0]} · IP ${c.effort[1]} · engine ${c.effort[2]} <span class="note">agent-days</span></dd>
    </dl>
    <div class="pc"><div class="pro"><h4>For</h4><ul>${c.pros.map((x) => `<li>${x}</li>`).join('')}</ul></div>
    <div class="con"><h4>Against</h4><ul>${c.cons.map((x) => `<li>${x}</li>`).join('')}</ul></div></div>`;
}

// ---------------------------------------------------------------------------------------------
// comparison table
function compareTable() {
  const t = $('#cmp');
  t.innerHTML = `<thead><tr><th>Option</th><th>Area (blocks per planet area, min / p1 / median / p99 / max)</th><th class="n">Shape a/b<br>median / max</th><th>Corners</th><th>Seams</th><th class="n">Seam length (16k)</th><th>Smoothness at seams and creases</th><th class="n">Effort, days<br>projection · IP · engine</th><th>Walk the equator</th><th>Cross a pole</th></tr></thead>`;
  const tb = el('tbody');
  for (const fam of ['cube', 'pill', 'now']) {
    for (const id of ORDER[fam]) {
      const c = C[id];
      const tr = el('tr', { class: c.status === 'rec' ? 'rec' : (c.status === 'now' ? 'ref' : '') });
      tr.innerHTML = `<td><b>${c.name}</b><span class="sub"><span class="tag ${c.status === 'rec' ? 'rec' : c.status === 'alt' ? 'alt' : c.status === 'run' ? 'run' : c.status === 'now' ? 'now' : 'out'}">${c.statusText}</span></span></td>
        <td class="num">${c.area}</td><td class="n">${c.shape[0]} / ${c.shape[1]}</td><td>${c.corners}</td><td>${c.seams}</td><td class="n">${fmt(c.seamLen)}</td>
        <td>${c.smooth}</td><td class="n">${c.effort.join(' · ')}</td><td>${c.eq}</td><td>${c.pole}</td>`;
      tb.appendChild(tr);
    }
  }
  t.appendChild(tb);
}

// ---------------------------------------------------------------------------------------------
// seam technology
function techTable() {
  const cols = ['Immersive Portals (v1)', 'Deep Time wrap engine', 'Toroidal World', 'Distant Horizons', 'Workbench map', 'Chunk and region grid', 'Create and contraptions near seams'];
  const rows = [
    ['Lambert (today)', [
      ['na', 'Not needed.'],
      ['na', 'Not needed.'],
      ['ok', 'Owns the only seam, as shipped.'],
      ['ok', 'Works today (TW guard mixin).'],
      ['ok', 'The 2:1 picture as today.'],
      ['ok', 'C = 16,384: the wrap is region-aligned.'],
      ['ok', 'TW carries Create (126 mixins) and Sable across the slide.']]],
    ['Petroff, Guyou 2:1', [
      ['ok', 'Proven in the prototype: 4 global portals, every seam walked, items, carts and arrows cross [R]. TW must go (it cannot share a world with IP); Destroy and CBC Modern Warfare crash with IP; Sable does not cross; collisions come from the hidden outside.'],
      ['ok', 'Owns 2 half-turn edges; the spike showed blocks, water, redstone, light and entities crossing a half-turn [R].'],
      ['ok', 'Keeps the slide in the engine route ("TW slides, engine turns").'],
      ['mid', 'FarTerrain folds the two long edges (a few dozen lines). Under IP, DH draws the raw world beyond seams, which is the folded planet.'],
      ['ok', 'Still a 2:1 picture.'],
      ['mid', 'b = 6,544 (409 chunks), not region-aligned (8,192 would be 1.57× larger).'],
      ['mid', 'IP: no adjacency across a portal; contraptions do not cross. Engine: kinetic and fluid networks refused at turning seams at first, adapters started in parallel; 13,088 blocks of turning seam.']]],
    ['Moser cube, cross net', [
      ['mid', '14 one-way portals; quarter-turn portals untested (the rotation NBT is inferred); 8 three-way corners, where the earlier prototype measured 30–40 % frame-rate cost; TW must go; the same mod crashes as Guyou.'],
      ['ok', 'Owns 6 turning pairs (4 quarter, 2 half). The engine\'s algebra already takes quarter-turns (a quarter-turn corner is among its unit-test layouts); a whole-world cube is not yet run. All turn centres sit on chunk corners at 3,776 or 4,096.'],
      ['ok', 'Can keep the band\'s east–west slide (period 4s): the band\'s wrap is a plain translation.'],
      ['mid', 'FarTerrain must fold quarter-turns and a cross shape: the same principle, more cases.'],
      ['mid', 'A cross picture (half its 4:3 box empty) or six faces; or keep drawing the planet in any projection, since the map need not be the world.'],
      ['ok', '236 chunks a face at 3,776; region-aligned at 4,096 (see Face size).'],
      ['bad', 'As Guyou, plus quarter-turns swap a shaft\'s x and z axes (an axis remap, not only a sign) [inference]; 22,656 blocks of turning seam, 1.7× Guyou\'s.']]],
    ['Moser cube, 3 × 2', [
      ['mid', '16 one-way portals, six seam edges on the line inside the rectangle (portal planes facing both ways along it).'],
      ['mid', 'Owns all 8 pairs: no world-wide slide for TW to keep, so the engine must also carry what TW carries today.'],
      ['bad', 'No role: its two slides are not a world-wide x wrap.'],
      ['mid', 'As the cross, plus an interior seam line.'],
      ['ok', 'A full 3 × 2 rectangle.'],
      ['ok', 'As the cross.'],
      ['bad', 'As the cross, with 26,416 blocks of turning seam and a seam through the middle of the world.']]],
  ];
  const t = $('#tech-table');
  t.innerHTML = `<thead><tr><th>Shape</th>${cols.map((c) => `<th>${c}</th>`).join('')}</tr></thead>`;
  const tb = el('tbody');
  for (const [name, cells] of rows) {
    const tr = el('tr', name.startsWith('Moser cube, cross') ? { class: 'rec' } : {});
    tr.innerHTML = `<td><b>${name}</b></td>` + cells.map(([s, txt]) => `<td><span class="st ${s}"></span>${txt}</td>`).join('');
    tb.appendChild(tr);
  }
  t.appendChild(tb);
  t.parentElement.insertAdjacentHTML('afterend', '<p class="note"><span class="st ok"></span>works or carries over <span class="st mid" style="margin-left:12px"></span>needs work <span class="st bad" style="margin-left:12px"></span>a real obstacle <span class="st na" style="margin-left:12px"></span>not applicable. [R] = run and seen in the prototype or the spike; the rest is read from the design notes (seamless-wrapping.md §6–8, wrap-engine.md §4–6) or marked as inference.</p>');
}

// ---------------------------------------------------------------------------------------------
// face size
function faceSize() {
  const L = D.layouts['cube-moser-cross'];
  const ub = L.unit_blocks;
  const sizes = [
    { s: 3776, lab: '3,776 (today\'s scale)', world: '16k' }, { s: 4096, lab: '4,096 (power of two)', world: '16k' },
    { s: 7552, lab: '7,552 (today\'s scale)', world: '32k' }, { s: 8192, lab: '8,192 (power of two)', world: '32k' }];
  // fixed points of every seam motion (turn centres) and every face corner, in face units, origin at face 0's centre
  const pts = [], centres = [];
  for (const sm of L.seams) {
    for (const p of [sm.a0, sm.a1]) pts.push([p[0] / ub - 0.5, p[1] / ub - 0.5]);
    if (sm.kind !== 'slide' && sm.side === 'B') {
      // the fixed point of the rotation taking edge A to its partner: solve from the two edges
      const B = L.seams.find((x) => x.pair === sm.pair && x.side === 'A');
      const a0 = sm.a0.map((v) => v / ub), a1 = sm.a1.map((v) => v / ub), b0 = B.a0.map((v) => v / ub), b1 = B.a1.map((v) => v / ub);
      // a0 <-> b1, a1 <-> b0; centre c with |c-a0| = |c-b1|, |c-a1| = |c-b0| (perpendicular bisectors)
      const c = bisect(a0, b1, a1, b0);
      if (c) centres.push([c[0] - 0.5, c[1] - 0.5, sm.kind]);
    }
  }
  const onGrid = (v, g) => Math.abs(v / g - Math.round(v / g)) < 1e-6;
  const rows = sizes.map((z) => {
    const s = z.s;
    const chunkC = centres.filter((c) => onGrid(c[0] * s, 16) && onGrid(c[1] * s, 16)).length;
    const regC = centres.filter((c) => onGrid(c[0] * s, 512) && onGrid(c[1] * s, 512)).length;
    const regE = pts.filter((p) => onGrid(p[0] * s, 512) && onGrid(p[1] * s, 512)).length;
    // region files touched by the six faces (face 0 centred on the origin; north = -z)
    const faces = [[0, 0], [1, 0], [2, 0], [3, 0], [0, -1], [0, 1]];
    const regs = new Set();
    for (const [fx, fz] of faces) {
      const x0 = (fx - 0.5) * s, z0 = (fz - 0.5) * s;
      for (let rx = Math.floor(x0 / 512); rx * 512 < x0 + s; rx++) for (let rz = Math.floor(z0 / 512); rz * 512 < z0 + s; rz++) regs.add(rx + ',' + rz);
    }
    const full = (s / 512) ** 2 * 6;
    const today = z.world === '16k' ? 16384 * 16384 / Math.PI : 32768 * 32768 / Math.PI;
    return { ...z, chunks: s / 16, regionsPer: s / 512, band: 4 * s, blocks: 6 * s * s, vsToday: 6 * s * s / today,
      mpb: 300 * (z.world === '16k' ? 3774.2 : 7548.4) / s, regs: regs.size, regsFull: full, chunkC, regC, nC: centres.length, regE, nE: pts.length, seam: 7 * s };
  });
  const t = $('#size-table');
  t.innerHTML = `<thead><tr><th>Face edge</th><th class="n">World</th><th class="n">Chunks a side</th><th class="n">Regions a side</th><th class="n">Equator lap (band)</th><th class="n">Blocks (6 s²)</th><th class="n">vs today</th><th class="n">Metres a block (same planet)</th><th class="n">Region files</th><th class="n">Turn centres on chunk corners</th><th class="n">… on region corners</th><th class="n">Seam length</th></tr></thead>`;
  const tb = el('tbody');
  for (const r of rows) {
    const tr = el('tr', r.s === 3776 || r.s === 7552 ? { class: 'rec' } : {});
    tr.innerHTML = `<td><b>${r.lab}</b></td><td class="n">${r.world}</td><td class="n">${fmt(r.chunks)}</td><td class="n">${fmt(r.regionsPer, 3)}</td><td class="n">${fmt(r.band)}</td>
      <td class="n">${fmt(r.blocks / 1e6, 1)} M</td><td class="n">${fmt(r.vsToday, 3)}×</td><td class="n">${fmt(r.mpb, 1)}</td><td class="n">${fmt(r.regs)} <span class="sub">${r.regs === r.regsFull ? 'each inside one face' : 'some straddle face edges'}</span></td>
      <td class="n">${r.chunkC} / ${r.nC}</td><td class="n">${r.regC} / ${r.nC}</td><td class="n">${fmt(r.seam)}</td>`;
    tb.appendChild(tr);
  }
  t.appendChild(tb);
  $('#size-text').innerHTML = `
    <p>The survey sized the faces so a block covers the same planet area as today: s = √(2π/3)·R = 3,774 at 16k, rounded to <b>3,776 = 236 chunks</b> (7,552 = 472 chunks at 32k). The alternative is a power of two, <b>4,096</b> (8,192).</p>
    <p><b>For today's scale:</b> every calibrated horizontal scale stays (300 m a block at 16k: erosion, feature widths, deposit spacing were tuned at it); the wrap engine only needs edges and turn centres on chunk lines, and 3,776 gives that (with face 0 centred on the origin every edge and every turn centre is a multiple of 1,888 = 118 chunks); 18 % fewer blocks to generate, store and pregenerate.</p>
    <p><b>For powers of two:</b> every face edge, seam and turn centre lands on a region line, so each region file belongs to one face; the band is exactly <b>16,384</b> blocks round, the same as today's x wrap and Toroidal World's 16k cylinder, so the equator walk is as long as today and the preset names stay literal. The price: the same planet gets 1.18× more blocks (277 m a block instead of 300, slopes 8 % gentler unless retuned), or the planet itself grows 8.5 % in radius.</p>
    <p class="note">Region alignment is a convenience (per-face storage, tools that select whole regions), not a requirement of either seam technology; Distant Horizons sees 64-block-aligned edges either way (3,776 = 59 × 64) [inference].</p>`;
  sizeRuler();
  chunkCorner($('#size-chunks-a'), 3776);
  chunkCorner($('#size-chunks-b'), 4096);
}
function bisect(p, q, r, s2) {
  // intersection of the perpendicular bisectors of pq and rs
  const m1 = [(p[0] + q[0]) / 2, (p[1] + q[1]) / 2], d1 = [q[0] - p[0], q[1] - p[1]];
  const m2 = [(r[0] + s2[0]) / 2, (r[1] + s2[1]) / 2], d2 = [s2[0] - r[0], s2[1] - r[1]];
  // d1·(c - m1) = 0, d2·(c - m2) = 0
  const det = d1[0] * d2[1] - d1[1] * d2[0];
  if (Math.abs(det) < 1e-12) {
    // a half-turn of an edge onto itself... the two bisectors coincide: the centre is the midpoint of p and q
    return m1;
  }
  const b1 = d1[0] * m1[0] + d1[1] * m1[1], b2 = d2[0] * m2[0] + d2[1] * m2[1];
  return [(b1 * d2[1] - b2 * d1[1]) / det, (d1[0] * b2 - d2[0] * b1) / det];
}
function sizeRuler() {
  const W = 620, H = 150, x0 = 20, scale = (W - 40) / 16384;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': 'Band rulers' });
  [[3776, 30, '3,776-block faces: 15,104 round'], [4096, 92, '4,096-block faces: 16,384 round']].forEach(([fs, y, lab]) => {
    const start = -fs / 2;
    for (let k = 0; k < 4; k++) {
      s.appendChild(sv('rect', { x: x0 + (k * fs) * scale, y, width: fs * scale, height: 26, fill: k % 2 ? css('--accent-bg') : css('--panel-2'), stroke: css('--ink-3'), 'stroke-width': .6 }));
      s.appendChild(sv('text', { x: x0 + (k * fs + fs / 2) * scale, y: y + 17, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-2') }, `face ${k}`));
    }
    // region lines (world x = start + offset)
    for (let rx = Math.ceil(start / 512) * 512; rx <= start + 4 * fs + 1; rx += 512) {
      const X = x0 + (rx - start) * scale;
      s.appendChild(sv('line', { x1: X, x2: X, y1: y - 5, y2: y + 31, stroke: css('--ink'), 'stroke-width': .7, 'stroke-opacity': .55 }));
    }
    // wrap seam at both ends
    for (const e of [0, 4 * fs]) s.appendChild(sv('line', { x1: x0 + e * scale, x2: x0 + e * scale, y1: y - 6, y2: y + 32, stroke: css('--slide'), 'stroke-width': 3 }));
    s.appendChild(sv('text', { x: x0, y: y - 9, 'font-size': 11.5, 'font-weight': 600, fill: css('--ink') }, lab));
  });
  s.appendChild(sv('text', { x: x0, y: H - 4, 'font-size': 10.5, fill: css('--ink-3') }, 'thin lines: region boundaries (512 blocks) · teal: the band\'s wrap seam · face 0 centred on x = 0'));
  $('#size-ruler').appendChild(s);
}
function chunkCorner(box, fs) {
  // a 1,100 x 600 block window round the corner where face 0, face 1 and the north face meet
  const cx = fs / 2, cz = -fs / 2; // the corner (north = -z)
  const wx0 = cx - 640, wz0 = cz - 300, ww = 1280, wh = 600;
  const W = 600, H = W * wh / ww, k = W / ww;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H + 16}`, width: '100%', role: 'img', 'aria-label': 'Chunk grid near a face corner' });
  // faces: band face 0 (x < cx, z > cz), face 1 (x > cx, z > cz), north face (x < cx, z < cz), empty (x > cx, z < cz)
  const X = (x) => (x - wx0) * k, Z = (z) => (z - wz0) * k;
  s.appendChild(sv('rect', { x: 0, y: Z(cz), width: X(cx), height: H - Z(cz), fill: css('--panel-2') }));
  s.appendChild(sv('rect', { x: X(cx), y: Z(cz), width: W - X(cx), height: H - Z(cz), fill: css('--accent-bg') }));
  s.appendChild(sv('rect', { x: 0, y: 0, width: X(cx), height: Z(cz), fill: css('--rec-bg') }));
  for (let x = Math.ceil(wx0 / 16) * 16; x < wx0 + ww; x += 16) {
    const r = x % 512 === 0;
    s.appendChild(sv('line', { x1: X(x), x2: X(x), y1: 0, y2: H, stroke: css('--ink'), 'stroke-opacity': r ? .7 : .12, 'stroke-width': r ? 1.4 : .5 }));
  }
  for (let z = Math.ceil(wz0 / 16) * 16; z < wz0 + wh; z += 16) {
    const r = z % 512 === 0;
    s.appendChild(sv('line', { y1: Z(z), y2: Z(z), x1: 0, x2: W, stroke: css('--ink'), 'stroke-opacity': r ? .7 : .12, 'stroke-width': r ? 1.4 : .5 }));
  }
  // seams: N east edge (x = cx, z < cz) quarter-turn to face 1's top (z = cz, x > cx)
  s.appendChild(sv('line', { x1: X(cx), x2: X(cx), y1: 0, y2: Z(cz), stroke: css('--quarter'), 'stroke-width': 4 }));
  s.appendChild(sv('line', { x1: X(cx), x2: W, y1: Z(cz), y2: Z(cz), stroke: css('--quarter'), 'stroke-width': 4 }));
  s.appendChild(sv('line', { x1: X(cx), x2: X(cx), y1: Z(cz), y2: H, stroke: css('--ink-2'), 'stroke-width': 1.5, 'stroke-dasharray': '5 4' }));
  s.appendChild(sv('line', { x1: 0, x2: X(cx), y1: Z(cz), y2: Z(cz), stroke: css('--ink-2'), 'stroke-width': 1.5, 'stroke-dasharray': '5 4' }));
  s.appendChild(sv('circle', { cx: X(cx), cy: Z(cz), r: 7, fill: css('--corner'), stroke: '#000', 'stroke-width': 1.2 }));
  const lab = (x, y, t, a = 'start') => s.appendChild(sv('text', { x, y, 'font-size': 12, fill: css('--ink'), 'text-anchor': a, 'font-weight': 600 }, t));
  lab(8, 16, 'north face');
  lab(8, H - 8, 'band face 0');
  lab(W - 8, H - 8, 'band face 1', 'end');
  lab(W - 8, 16, 'outside the net (folded)', 'end');
  s.appendChild(sv('text', { x: X(cx) + 10, y: Z(cz) - 8, 'font-size': 11.5, fill: css('--ink') }, `corner at x = ${fmt(cx)}, z = ${fmt(cz)}: quarter-turn centre`));
  s.appendChild(sv('text', { x: 0, y: H + 13, 'font-size': 10.5, fill: css('--ink-3') }, `faint: chunk lines (16) · dark: region lines (512) · ${fmt(cx / 16, cx % 16 ? 2 : 0)} chunks, ${fmt(cx / 512, 3)} regions from the origin`));
  box.appendChild(s);
}

// ---------------------------------------------------------------------------------------------
// the net
function netSection() {
  const box = $('#net-maps');
  for (const id of ['cube-moser-cross', 'cube-moser-3x2']) {
    const f = el('figure', { class: 'fig' });
    f.appendChild(el('h4', {}, id.endsWith('cross') ? 'Cross net <span class="tag rec">recommended</span>' : '3 × 2 rectangle'));
    const st = el('div', { class: 'stagebox' });
    const mb = el('div', { class: 'mapbox' });
    st.appendChild(mb);
    f.appendChild(st);
    mapStack(mb, id, 'map', new Set(['seams', 'corners', 'walks']));
    const L = D.layouts[id];
    const kinds = L.seams.filter((s) => s.side === 'B').reduce((m, s) => (m[s.kind] = (m[s.kind] || 0) + 1, m), {});
    f.appendChild(el('figcaption', {}, id.endsWith('cross')
      ? `Four faces round the equator, the poles at the centres of the two polar faces. Seams: ${Object.entries(kinds).map(([k, n]) => `${n} ${KNAME[k]}`).join(', ')}. The equator walk (blue) crosses one slide per lap; over the pole (purple) one half-turn.`
      : `Top row: band faces 0, 1, 2 (north up); bottom row: the north face, face 3 and the south face, turned. Seams: ${Object.entries(kinds).map(([k, n]) => `${n} ${KNAME[k]}`).join(', ')}. The equator walk crosses two seams per lap.`));
    box.appendChild(f);
  }
  const t = $('#net-table');
  const rows = [
    ['Seam pairs (one-way portals)', '7 (14)', '8 (16)'],
    ['Seam length at 16k', '26,432 blocks', '30,208 blocks'],
    ['Seam kinds', '1 slide, 4 quarter-turns, 2 half-turns', '2 slides, 4 quarter-turns, 2 half-turns'],
    ['Seams inside the world', 'none: every seam is on the outline', 'the line between the rows carries six seam edges (parts of five pairs)'],
    ['A world-wide east–west wrap for Toroidal World', 'yes: the band\'s wrap (period 4s)', 'no: one slide is a 2-face jump along the inner line, the other a diagonal jump'],
    ['Equator lap', 'straight in raw coordinates, 1 seam', 'straight in game, 2 seams (a quarter-turn into face 3, then back)'],
    ['Over a pole', '1 seam (from faces 0 and 2) or 2 (from 1 and 3)', '2 seams'],
    ['Raster storage', 'six face rasters, or a 4s × 3s box half empty', 'one 3s × 2s rectangle, no waste'],
    ['Picture for maps and the workbench', 'the familiar band with the poles above and below', 'two strips, the second turned'],
  ];
  t.innerHTML = `<thead><tr><th></th><th>Cross <span class="tag rec">recommended</span></th><th>3 × 2</th></tr></thead><tbody>${rows.map((r) => `<tr><td><b>${r[0]}</b></td><td>${r[1]}</td><td>${r[2]}</td></tr>`).join('')}</tbody>`;
}

// ---------------------------------------------------------------------------------------------
// walking
function walkSection() {
  const box = $('#walk-grid');
  const list = [['lambert', 'Lambert: a strip with walls'], ['guyou-petroff', 'Guyou (Petroff): one slide, two half-turns'], ['cube-moser-cross', 'Cube, cross net'], ['cube-moser-3x2', 'Cube, 3 × 2']];
  for (const [id, title] of list) {
    const L = D.layouts[id];
    const f = el('figure', { class: 'fig' });
    f.appendChild(el('h4', {}, title));
    const st = el('div', { class: 'stagebox' });
    const mb = el('div', { class: 'mapbox' });
    st.appendChild(mb);
    f.appendChild(st);
    mapStack(mb, id, 'map', new Set(['seams', 'walks']));
    const eq = L.walks.equator, po = L.walks.pole;
    const cross = (w) => w.crossings.map((c) => `<span class="kind ${c.kind}">${c.kind === 'slide' ? 'slide' : (c.turn === 180 ? 'half-turn' : 'quarter-turn')}</span> at ${fmt(c.dist)}`).join(', ') || 'none';
    f.appendChild(el('figcaption', {}, `<b>Equator:</b> ${fmt(eq.blocks)} blocks for 360°; seams: ${cross(eq)}.<br><b>North over the pole:</b> ${po.stopped ? `stops at the 80° line after ${fmt(po.blocks)} blocks (${fmt(po.planet_deg)}° of planet)` : `${fmt(po.blocks)} blocks equator to equator; seams: ${cross(po)}`}.`));
    box.appendChild(f);
  }
  eqChart();
  cornerFig();
  sweepChart($('#sweep-cube'), D.sweep_cube, D.lam0_deg, 90, 'λ₀ (degrees)');
  sweepChart($('#sweep-guyou'), D.sweep_guyou, D.lamc_deg, 360, 'λc (degrees)');
}
function eqChart() {
  const series = [['lambert', 'Lambert (today)', css('--ink-3')], ['guyou-petroff', 'Petroff, Guyou', css('--half')], ['cube-moser-cross', 'Moser cube, 3,776', css('--rec')], ['cube-qsc-cross', 'QSC cube', css('--quarter')]];
  const W = 600, H = 260, ml = 46, mr = 14, mt = 14, mb = 40;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': 'Blocks per degree along the equator' });
  const ymin = 30, ymax = 60;
  const xs = (f) => ml + f * (W - ml - mr), ys = (v) => mt + (1 - (v - ymin) / (ymax - ymin)) * (H - mt - mb);
  for (let v = ymin; v <= ymax; v += 5) {
    s.appendChild(sv('line', { x1: ml, x2: W - mr, y1: ys(v), y2: ys(v), stroke: css('--rule'), 'stroke-width': .7 }));
    s.appendChild(sv('text', { x: ml - 6, y: ys(v) + 4, 'text-anchor': 'end', 'font-size': 11, fill: css('--ink-3') }, v));
  }
  for (let f = 0; f <= 1.0001; f += 0.25) {
    s.appendChild(sv('text', { x: xs(f), y: H - mb + 16, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-3') }, `${Math.round(f * 360)}°`));
  }
  s.appendChild(sv('text', { x: (ml + W - mr) / 2, y: H - 6, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-3') }, 'longitude walked along the equator'));
  s.appendChild(sv('text', { x: 12, y: mt + 4, 'font-size': 11, fill: css('--ink-3'), transform: `rotate(-90 12 ${mt + 4})`, 'text-anchor': 'end' }, 'blocks per degree'));
  series.forEach(([id, lab, col], i) => {
    const w = D.layouts[id].walks.equator;
    const tot = w.blocks;
    // convert distance to cumulative planet degrees: integrate blocks/deg
    let deg = 0, prev = 0;
    const pts = [];
    for (const [d, bpd] of w.profile) { deg += (d - prev) / bpd; prev = d; pts.push([deg / 360, bpd]); }
    s.appendChild(sv('polyline', { points: pts.map(([f, v]) => `${xs(Math.min(f, 1))},${ys(Math.max(ymin, Math.min(ymax, v)))}`).join(' '), fill: 'none', stroke: col, 'stroke-width': 2 }));
    const mean = tot / 360;
    s.appendChild(sv('rect', { x: ml + 8 + (i % 2) * 270, y: H - mb - 46 + Math.floor(i / 2) * 18 - 160, width: 12, height: 3, fill: col }));
    s.appendChild(sv('text', { x: ml + 24 + (i % 2) * 270, y: H - mb - 42 + Math.floor(i / 2) * 18 - 160, 'font-size': 11, fill: css('--ink') }, `${lab}: ${fmt(tot)} round (${fmt(mean, 1)}/°)`));
  });
  $('#eq-chart').appendChild(s);
}
function cornerFig() {
  // zoom round the corner each pair passes, in tile units, both worlds side by side
  const W = 600, H = 300;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': 'Walkers either side of a corner' });
  const panels = [['cube-moser-cross', 'cube corner (270°)', [0.35, 0.3, 1.75, 1.95]], ['guyou-petroff', 'pillowcase corner (180°)', [0.0, -0.2, 2.0, 1.25]]];
  panels.forEach(([id, lab, win], pi) => {
    const L = D.layouts[id], ub = L.unit_blocks;
    const ox = pi * 300 + 10, pw = 280, ph = 250;
    const [wx0, wy0, wx1, wy1] = win;
    const k = Math.min(pw / (wx1 - wx0), ph / (wy1 - wy0));
    const X = (x) => ox + (x / ub - wx0) * k, Y = (y) => 20 + (wy1 - y / ub) * k;
    const g = sv('g', {});
    const clip = sv('clipPath', { id: 'cc' + pi });
    clip.appendChild(sv('rect', { x: ox, y: 20, width: pw, height: ph }));
    s.appendChild(clip);
    g.setAttribute('clip-path', `url(#cc${pi})`);
    for (const t of L.tiles) g.appendChild(sv('rect', { x: X(t[0]), y: Y(t[1] + t[3]), width: t[2] / ub * k, height: t[3] / ub * k, fill: css('--panel-2'), stroke: css('--ink-3'), 'stroke-width': .8 }));
    for (const sm of L.seams) g.appendChild(sv('line', { x1: X(sm.a0[0]), y1: Y(sm.a0[1]), x2: X(sm.a1[0]), y2: Y(sm.a1[1]), stroke: css(KCOL[sm.kind]), 'stroke-width': 3 }));
    for (const [cx, cy] of L.corners) g.appendChild(sv('circle', { cx: X(cx), cy: Y(cy), r: 5, fill: css('--corner'), stroke: '#000' }));
    for (const [wk, col] of [['cornerA', css('--walk-a')], ['cornerB', css('--walk-b')]]) {
      const p0 = L.walks[wk].segs[0][0];
      g.appendChild(sv('text', { x: X(p0[0]) + (wk === 'cornerA' ? -14 : 6), y: Y(p0[1]) + 22, 'font-size': 13, 'font-weight': 700, fill: col }, wk === 'cornerA' ? 'A' : 'B'));
      for (const sg of L.walks[wk].segs) {
        g.appendChild(sv('polyline', { points: sg.map((p) => `${X(p[0])},${Y(p[1])}`).join(' '), fill: 'none', stroke: col, 'stroke-width': 2.4 }));
        if (sg.length > 2) arrowHead(g, [X(sg[sg.length - 2][0]), Y(sg[sg.length - 2][1])], [X(sg[sg.length - 1][0]), Y(sg[sg.length - 1][1])], col, 9);
      }
    }
    s.appendChild(g);
    s.appendChild(sv('text', { x: ox, y: 13, 'font-size': 12, 'font-weight': 600, fill: css('--ink') }, lab));
  });
  $('#corner-fig').appendChild(s);
  $('#corner-cap').innerHTML = 'Two walkers, A (red) and B, start side by side heading north, one either side of a corner. On the cube A walks straight on into the polar face while B crosses a quarter-turn seam: they now walk at right angles and their paths cross. In the pillowcase both cross the half-turn seam and come back south, each along the other\'s track: seen from one side of the seam they walk in opposite directions. That is what missing 90° and missing 180° mean for a player.';
}
function sweepChart(box, rows, chosen, xmax, xl) {
  const W = 600, H = 180, ml = 40, mr = 10, mt = 10, mb = 34;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': 'Corner placement sweep' });
  const xs = (v) => ml + v / xmax * (W - ml - mr), ys = (v) => mt + (1 - v) * (H - mt - mb);
  for (const v of [0, 0.5, 1]) {
    s.appendChild(sv('line', { x1: ml, x2: W - mr, y1: ys(v), y2: ys(v), stroke: css('--rule'), 'stroke-width': .7 }));
    s.appendChild(sv('text', { x: ml - 6, y: ys(v) + 4, 'text-anchor': 'end', 'font-size': 11, fill: css('--ink-3') }, `${v * 100}%`));
  }
  const bw = (W - ml - mr) / rows.length;
  for (const [lam, mx] of rows) {
    s.appendChild(sv('rect', { x: xs(lam), y: ys(mx), width: Math.max(1, bw - 0.5), height: ys(0) - ys(mx) || 0.0001, fill: mx === 0 ? css('--rec') : css('--ink-3'), 'fill-opacity': mx === 0 ? 0.9 : 0.6 }));
    if (mx === 0) s.appendChild(sv('rect', { x: xs(lam), y: ys(0) - 3, width: Math.max(1, bw - 0.5), height: 3, fill: css('--rec') }));
  }
  s.appendChild(sv('line', { x1: xs(chosen), x2: xs(chosen), y1: mt, y2: ys(0), stroke: css('--accent'), 'stroke-width': 1.5, 'stroke-dasharray': '4 3' }));
  const ok = rows.filter((r) => r[1] === 0).length / rows.length;
  s.appendChild(sv('text', { x: xs(chosen) + 5, y: mt + 12, 'font-size': 11, fill: css('--accent') }, `used here: ${chosen}°`));
  for (let v = 0; v <= xmax; v += xmax / 6) s.appendChild(sv('text', { x: xs(v), y: H - mb + 15, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-3') }, Math.round(v)));
  s.appendChild(sv('text', { x: (ml + W) / 2, y: H - 4, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-3') }, `${xl} · green: every corner's 3° disc is open sea (${Math.round(ok * 100)} % of turns)`));
  box.appendChild(s);
}

// ---------------------------------------------------------------------------------------------
// simulation grid
function simTable() {
  const rows = [
    ['Lambert (today)', 'A copy: the world is the simulation\'s own grid.', 'A full resample (sphere to strip); the strip still cannot reach the poles.'],
    ['Petroff, Guyou 2:1', 'A full resample; the polar caps come out streaked (Lambert rows resolve them badly) unless sampled from the plates\' sphere.', 'A full resample, but from a grid that resolves the poles evenly: no streaks.'],
    ['Moser cube (cross or 3 × 2)', 'A full resample, with the poles at face centres, where Lambert\'s streaks show worst.', '<b>Nearly a copy:</b> each world face is one grid face, so the bake is a smooth warp within each face (the equal-area table against the grid\'s own face map), with nothing crossing a seam. If the grid used the same equal-area face map it would be an exact copy at matching resolution [inference].'],
  ];
  $('#sim-table').innerHTML = `<thead><tr><th>World shape</th><th>Bake from today\'s Lambert simulation grid</th><th>Bake from a cubed-sphere simulation grid</th></tr></thead><tbody>${rows.map((r, i) => `<tr${i === 2 ? ' class="rec"' : ''}><td><b>${r[0]}</b></td><td>${r[1]}</td><td>${r[2]}</td></tr>`).join('')}</tbody>`;
}

// ---------------------------------------------------------------------------------------------
// findings, method, footer
function findings() {
  const cube = D.sweep_cube, guy = D.sweep_guyou;
  const okC = Math.round(100 * cube.filter((r) => r[1] === 0).length / cube.length);
  const okG = Math.round(100 * guy.filter((r) => r[1] === 0).length / guy.length);
  const items = [
    '<b>The 3 × 2 net has no world-wide slide, so it breaks "TW slides, engine turns".</b> Its 8 pairs are 2 slides, 4 quarter-turns and 2 half-turns, but neither slide is an east–west wrap of the whole world (one is a two-face jump along the line between the rows, the other a diagonal jump). Toroidal World could own no edge, and the engine would have to carry everything TW carries today. The survey described it only as "one more seam inside the world"; in fact six seam edges lie on the line between the rows. This strengthens the cross net.',
    '<b>Power-of-two faces make the band exactly today\'s 16,384 blocks round</b> (32,768 at 32k): the equator walk, Toroidal World\'s cylinder period and the preset names all stay as they are, and every seam lands on a region line. At today\'s scale the equal-area cube draws the equator 8 % short (15,104); Petroff\'s Guyou world draws it 20 % short (13,088). This does not overturn the survey\'s choice of today\'s scale, which keeps every calibrated horizontal scale, but it is a better case for powers of two than "the planet drawn 1.18× larger".',
    `<b>Corner placement on this planet:</b> all eight cube corners have open sea within 3° for ${okC} % of the possible turns λ₀; Guyou's four points for ${okG} % of λc. Eight corners are harder to hide than four, but on an equal-area cube they need not be hidden: they magnify nothing.`,
    '<b>Lambert-bake streaks land on the cube\'s polar face centres.</b> The pole is in the middle of a face, so the radial streaks the prototype found round Guyou\'s poles are more prominent on every cube map. A production bake must sample the caps from the sphere (or from a cubed-sphere simulation grid, see above) before the cube is reviewed for looks.',
    '<b>Nothing here changes the shape recommendation.</b> On the real planet the crease test shows the survey\'s numbers as pictures: QSC\'s diagonals and every edge of the graphics warps fold visibly, the Moser cube and Petroff do not, and the cube\'s shapes are visibly closer to true than Petroff\'s.',
  ];
  $('#findings-list').innerHTML = items.map((x) => `<li>${x}</li>`).join('');
}
function method() {
  $('#method-text').innerHTML = `
  <p><b>Pictures.</b> <span class="mono">tools/review/world-shape/compute.py</span> on the test VM (through <span class="mono">run.sh</span> and the admission tool) rebuilds every world on the survey's own projection code (<span class="mono">tools/research/projection-survey/</span>: Petroff, Gringorten, Collignon, Peirce, QSC by Roşca–Plonka, HEALPix, equi-angular, the conformal cube by Schwarz triangle maps and the Moser flow at 512²). Each world pixel goes to its planet point and samples <span class="mono">deeptime heightmap</span> of workbench run 1a103bb2d2b (earthlike_quick_16k, format 0.4, Lambert) at 4 blocks a sample; colours from height, precipitation and ice; hill shade in world space. Area and shape from the world-to-planet Jacobian (differences within one face only). Seam tables are built from the cube's edge adjacency, checked by continuity: every seam of every world closes to ≤ 4·10⁻⁹ rad. Walks step through the seam motions; their planet tracks give the blocks-per-degree profile.</p>
  <p><b>Numbers.</b> Distortion, crease and cost figures are the survey's (<span class="mono">docs/research/notes/seamless-projection-survey.md</span> §3, §5, §6). Effort: <span class="mono">docs/design/seamless-wrapping.md</span> §8 (26–40 days for Guyou with portals), survey §6.1 (crate 8–11 days, +5–10 outside it for the cube), <span class="mono">docs/design/wrap-engine.md</span> §6 (46–75 days without trains). The engine's extra 3–8 days for the cube (six turning pairs, quarter-turn block-state turns, eight corners) and the 5–8 days for a QSC or HEALPix crate are this page's inference. Seam technology rows: seamless-wrapping.md §5–7, wrap-engine.md §1–4, the decisions log (2026-10-03).</p>
  <p><b>Not done.</b> No game was launched; quarter-turn portals and a whole-world cube in the engine remain untested; the Moser table's convergence is not studied (512² only); the bake is Lambert, so the poles are streaked.</p>`;
  $('#foot').innerHTML = `Generated ${D.generated} from <span class="mono">tools/review/world-shape/</span> · Deep Time (LGPL-3.0-or-later) · three.js r128 (MIT), local copy · distortion data: seamless-projection-survey.md (2026-10-03).`;
  $('#lam0').textContent = D.lam0_deg;
  $('#lamc').textContent = D.lamc_deg;
}

// ---------------------------------------------------------------------------------------------
function init() {
  const only = (location.hash.match(/only-([a-z-]+(?:\+[a-z-]+)*)/) || [])[1];
  if (only) document.querySelectorAll('section').forEach((x) => { if (!only.split('+').includes(x.id)) x.hidden = true; });
  decisionCards();
  coneFigure();
  setupExplorer();
  compareTable();
  techTable();
  faceSize();
  netSection();
  walkSection();
  simTable();
  findings();
  method();
}
init();
window.WS = { C, ORDER, FAMNAME, KCOL, css };
// redraw theme-dependent SVGs when the colour scheme flips
try {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => location.reload());
} catch (e) { /* older browsers */ }
})();
