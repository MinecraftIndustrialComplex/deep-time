// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time world-shape page (head-to-head): maps, corner figures, tables and charts.
// Data: data.js (tools/review/world-shape/compute.py, run on the test VM). No network.
'use strict';
(function () {
const D = window.WS_DATA;
if (/debug/.test(location.hash)) {
  window.addEventListener('error', (e) => document.body.insertAdjacentHTML('afterbegin',
    `<pre style="background:#600;color:#fff;padding:8px;white-space:pre-wrap">${e.message} ${e.filename}:${e.lineno}:${e.colno}</pre>`));
}
if (/(^|[#,])theme-dark/.test(location.hash)) document.documentElement.dataset.theme = 'dark';
if (/(^|[#,])theme-light/.test(location.hash)) document.documentElement.dataset.theme = 'light';
// review screenshots only (#from-<id>): hide everything before element <id>, since a headless
// screenshot cannot scroll to an anchor
const fromM = /(^|[#,])from-([\w-]+)/.exec(location.hash);
if (fromM) {
  window.addEventListener('load', () => {
    let el = document.getElementById(fromM[2]);
    while (el && el !== document.body) {
      for (let p = el.previousElementSibling; p; p = p.previousElementSibling) p.style.display = 'none';
      el = el.parentElement;
    }
  });
}
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
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
const pct = (x, d = 0) => `${fmt(100 * x, d)} %`;
const css = (v) => getComputedStyle(document.documentElement).getPropertyValue(v).trim();
const KCOL = { slide: '--slide', quarter: '--quarter', half: '--half' };
const KNAME = { slide: 'slide', quarter: 'quarter-turn', half: 'half-turn' };
const G = 'guyou-petroff', K = 'cube-moser-cross';
const NAME = { [G]: 'Petroff–Guyou', [K]: 'Moser cube' };
const LONG = { [G]: 'Petroff 2021 in Guyou\'s 2:1 layout', [K]: 'Moser equal-area cube, cross net' };
const WCOL = { [G]: '--half', [K]: '--quarter' };

// ---------------------------------------------------------------------------------------------
// numbers in the prose: <span data-v="stats|guyou-petroff|land_shape_gt|1.5" data-f="pct">
function getPath(path) {
  let o = D;
  for (const k of path.split('|')) { if (o == null) return undefined; o = o[k]; }
  return o;
}
function bind() {
  for (const e of $$('[data-v]')) {
    const v = getPath(e.dataset.v);
    if (v === undefined || v === null) { e.textContent = '?'; e.title = 'missing: ' + e.dataset.v; continue; }
    const f = e.dataset.f || 'int';
    e.textContent = f === 'pct' ? pct(v) : f === 'pct1' ? pct(v, 1) : f === 'f1' ? fmt(v, 1) : f === 'f2' ? fmt(v, 2)
      : f === 'f3' ? fmt(v, 3) : f === 'deg' ? `${fmt(v, 0)}°` : f === 'sci' ? Number(v).toExponential(0) : f === 'raw' ? String(v) : fmt(v);
  }
}

// ---------------------------------------------------------------------------------------------
// the cone-point figure
function coneFigure() {
  const W = 540, H = 200;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': 'Discs of 360, 270 and 180 degrees' });
  const items = [[360, 'ordinary point', '360°'], [270, 'cube corner', '270° (90° missing)'], [180, 'Guyou corner', '180° (180° missing)']];
  items.forEach(([deg, lab, sub], i) => {
    const cx = 90 + i * 180, cy = 92, r = 62;
    const a0 = -Math.PI / 2, a1 = a0 + deg * Math.PI / 180;
    const d = deg >= 360 ? `M ${cx - r} ${cy} a ${r} ${r} 0 1 0 ${2 * r} 0 a ${r} ${r} 0 1 0 ${-2 * r} 0`
      : `M ${cx} ${cy} L ${cx + r * Math.cos(a0)} ${cy + r * Math.sin(a0)} A ${r} ${r} 0 ${deg > 180 ? 1 : 0} 1 ${cx + r * Math.cos(a1)} ${cy + r * Math.sin(a1)} Z`;
    s.appendChild(sv('path', { d, fill: i === 0 ? css('--panel-2') : (i === 1 ? css('--quarter-bg') : css('--half-bg')), stroke: css('--ink-2'), 'stroke-width': 1.2 }));
    if (deg < 360) {
      const col = i === 1 ? css('--quarter') : css('--half');
      for (const a of [a0, a1]) s.appendChild(sv('line', { x1: cx, y1: cy, x2: cx + r * Math.cos(a), y2: cy + r * Math.sin(a), stroke: col, 'stroke-width': 3 }));
      s.appendChild(sv('circle', { cx, cy, r: 5, fill: css('--corner'), stroke: '#000', 'stroke-width': 1.2 }));
    }
    for (let k = 1; k <= 3; k++) s.appendChild(sv('circle', { cx, cy, r: r * k / 4, fill: 'none', stroke: css('--ink-3'), 'stroke-width': .6, 'stroke-dasharray': '2 3' }));
    s.appendChild(sv('text', { x: cx, y: 176, 'text-anchor': 'middle', 'font-size': 14, 'font-weight': 600, fill: css('--ink') }, lab));
    s.appendChild(sv('text', { x: cx, y: 194, 'text-anchor': 'middle', 'font-size': 12, fill: css('--ink-3') }, sub));
  });
  $('#cone-fig').appendChild(s);
}

function arrowHead(g, a, b, col, size, stroke) {
  const dx = b[0] - a[0], dy = b[1] - a[1], L = Math.hypot(dx, dy) || 1;
  const ux = dx / L, uy = dy / L, px = -uy, py = ux;
  const bl = [b[0] - ux * size + px * size * 0.55, b[1] - uy * size + py * size * 0.55], br = [b[0] - ux * size - px * size * 0.55, b[1] - uy * size - py * size * 0.55];
  g.appendChild(sv('polygon', { points: [b, bl, br].map((p) => p.join(',')).join(' '), fill: col, stroke: stroke || 'none', 'stroke-width': size * 0.08 }));
}
function polyline(g, pts, col, w, extra = {}) {
  const p = pts.map((q) => q.join(',')).join(' ');
  g.appendChild(sv('polyline', { points: p, fill: 'none', stroke: '#000', 'stroke-opacity': .5, 'stroke-width': w * 1.7, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }));
  g.appendChild(sv('polyline', Object.assign({ points: p, fill: 'none', stroke: col, 'stroke-width': w, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }, extra)));
}

// ---------------------------------------------------------------------------------------------
// the two worlds side by side
const WALKCOL = { equator: '--walk-eq', pole: '--walk-pole', poleB: '--walk-pole', diag: '--walk-diag', cornerA: '--walk-a', cornerB: '--walk-b' };
function seamOverlay(s, L, u, opts) {
  const lw = u / 380;
  const P = (p) => [p[0], -p[1]];
  if (opts.walks && L.walks) {
    for (const k of ['equator', 'pole', 'poleB', 'diag']) {
      const w = L.walks[k];
      if (!w) continue;
      const col = css(WALKCOL[k]);
      for (const sg of w.segs) {
        polyline(s, sg.map(P), col, lw * 1.8);
        if (sg.length > 2) { const m = Math.floor(sg.length / 2); arrowHead(s, P(sg[m - 1]), P(sg[m]), col, u * 0.016); }
      }
      for (const c of w.crossings) for (const p of [c.at, c.to]) s.appendChild(sv('circle', { cx: p[0], cy: -p[1], r: u * 0.007, fill: col, stroke: '#fff', 'stroke-width': lw * .7 }));
    }
  }
  if (opts.seams) {
    const letters = 'ABCDEFGHIJ';
    for (const sm of L.seams) {
      const col = css(KCOL[sm.kind]);
      const a = P(sm.a0), b = P(sm.a1);
      s.appendChild(sv('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1], stroke: '#000', 'stroke-opacity': .55, 'stroke-width': lw * 3.4 }));
      s.appendChild(sv('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1], stroke: col, 'stroke-width': lw * 2.2 }));
      const [p, q] = sm.side === 'A' ? [b, a] : [a, b];
      const m1 = [p[0] + (q[0] - p[0]) * 0.62, p[1] + (q[1] - p[1]) * 0.62], m0 = [p[0] + (q[0] - p[0]) * 0.6, p[1] + (q[1] - p[1]) * 0.6];
      arrowHead(s, m0, m1, col, u * 0.02, '#000');
      const dx = b[0] - a[0], dy = b[1] - a[1], len = Math.hypot(dx, dy);
      const nx = dy / len, ny = -dx / len;
      const tp = [a[0] + dx * 0.3 + nx * u * 0.024, a[1] + dy * 0.3 + ny * u * 0.024];
      s.appendChild(sv('circle', { cx: tp[0], cy: tp[1], r: u * 0.016, fill: col, stroke: '#fff', 'stroke-width': lw * .6 }));
      s.appendChild(sv('text', { x: tp[0], y: tp[1] + u * 0.006, 'text-anchor': 'middle', 'font-size': u * 0.018, 'font-weight': 700, fill: '#fff' }, letters[sm.pair]));
    }
  }
  if (opts.corners) {
    for (const [x, y, v] of L.corners) {
      s.appendChild(sv('circle', { cx: x, cy: -y, r: u * 0.012, fill: css('--corner'), stroke: '#000', 'stroke-width': lw * .9 }));
      s.appendChild(sv('text', { x, y: -y + u * 0.0045, 'text-anchor': 'middle', 'font-size': u * 0.012, 'font-weight': 700, fill: '#000' }, String(v + 1)));
    }
  }
}
function overlaySvg(id, opts) {
  const L = D.layouts[id];
  const [bx0, by0, bx1, by1] = L.bbox;
  const bw = bx1 - bx0, bh = by1 - by0, u = Math.max(bw, bh);
  const s = sv('svg', { viewBox: `${bx0} ${-by1} ${bw} ${bh}`, preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  if (opts.tissot) {
    const g = sv('g', { fill: 'rgba(255,255,255,.18)', stroke: 'rgba(255,255,255,.85)', 'stroke-width': u / 760 });
    for (const [x, y, ra, rb, ang] of L.tissot) g.appendChild(sv('ellipse', { cx: x, cy: -y, rx: ra, ry: rb, transform: `rotate(${-ang} ${x} ${-y})` }));
    s.appendChild(g);
  }
  seamOverlay(s, L, u, opts);
  if (opts.spawn) {
    const sp = D.stats[id].dist.spawn_blocks;
    s.appendChild(sv('circle', { cx: sp[0], cy: -sp[1], r: u * 0.014, fill: '#fff', stroke: '#000', 'stroke-width': u / 400 }));
    s.appendChild(sv('circle', { cx: sp[0], cy: -sp[1], r: u * 0.005, fill: '#000' }));
  }
  return s;
}
const LAYERS = [['map', 'Planet'], ['shape', 'Shape'], ['area', 'Area'], ['north', 'North error'], ['bands', 'Latitude bands'], ['dist', 'Distance from spawn'], ['compass', 'Compass error'], ['crease', 'Crease test']];
const OVS = [['seams', 'Seams'], ['corners', 'Corners'], ['coast', 'Coast'], ['grat', 'Graticule'], ['walks', 'Walks'], ['tissot', 'Tissot']];
const ex = { layer: 'map', ov: new Set(['seams', 'corners']) };
function mapStack(box, id, layer, ov) {
  const L = D.layouts[id];
  const [bx0, by0, bx1, by1] = L.bbox;
  box.innerHTML = '';
  box.style.aspectRatio = `${bx1 - bx0} / ${by1 - by0}`;
  box.appendChild(el('img', { src: `${id}-${layer}.webp`, alt: `${NAME[id]}: ${layer}`, loading: 'lazy' }));
  if (layer === 'dist') box.appendChild(el('img', { src: `${id}-pdist.webp`, alt: '', loading: 'lazy' }));
  if (ov.has('coast') || layer === 'shape' || layer === 'north') box.appendChild(el('img', { src: `${id}-coast.webp`, alt: '', loading: 'lazy', class: layer === 'map' ? '' : 'faint' }));
  if (ov.has('grat')) box.appendChild(el('img', { src: `${id}-grat.webp`, alt: '', loading: 'lazy' }));
  box.appendChild(overlaySvg(id, { seams: ov.has('seams'), corners: ov.has('corners'), tissot: ov.has('tissot'), walks: ov.has('walks'), spawn: layer === 'dist' || layer === 'compass' }));
}
const LEG = {
  map: () => 'Hypsometric colours from the bake; hill shade drawn in world space.',
  shape: () => `shape ratio a/b: 1<span class="bar" style="background:linear-gradient(90deg,#fafaf0,#fee8a0,#fdae61,#e6613c,#be2846,#78145a,#320a46)"></span>4+ <span>(how many times too long one direction is drawn; white lines: coasts)</span>`,
  area: () => `blocks per planet area: ¼×<span class="bar" style="background:linear-gradient(90deg,#2166ac,#67a9cf,#d1e5f0,#f7f7f7,#fddbc7,#ef8a62,#b2182b)"></span>4× <span>(both are equal-area: all white)</span>`,
  north: () => `angle between F3's north (−z) and true north: 0°<span class="bar" style="background:linear-gradient(90deg,#fafaf0,#fee8a0,#fdae61,#e6613c,#be2846,#78145a,#320a46)"></span>180°`,
  bands: () => ['equatorial (0–10°)', 'tropics (10–23.4°)', 'subtropics (23.4–35°)', 'temperate (35–55°)', 'boreal (55–66.6°)', 'polar (66.6–90°)'].map((t, i) => `<span><span class="sw" style="background:${['#d63c2e', '#f08c32', '#ebcd50', '#5aaa5a', '#3c82aa', '#afa0dc'][i]}"></span>${t}</span>`).join(''),
  dist: () => `walking distance from spawn (dot), bands every 1,000 blocks: 0<span class="bar" style="background:linear-gradient(90deg,#fff7ec,#fdd49e,#fc8d59,#d7301f,#7f0000,#280028)"></span>9,000 <span><span class="sw" style="background:#1a59f2"></span>the planet's own distance, every 1,000 blocks</span>`,
  compass: () => `how far a vanilla compass (raw coordinates) points from the true way back to spawn: 0°<span class="bar" style="background:linear-gradient(90deg,#fafaf0,#fee8a0,#fdae61,#e6613c,#be2846,#78145a,#320a46)"></span>180°`,
  crease: () => 'hill shade of a smooth made-up planet: straight lines of broken shading are creases',
};
function renderWorlds() {
  for (const id of [G, K]) mapStack($(`#mb-${id}`), id, ex.layer, ex.ov);
  let h = `<span>${LEG[ex.layer]()}</span>`;
  if (ex.ov.has('seams')) h += ['slide', 'quarter', 'half'].map((k) => `<span><span class="sw" style="background:var(${KCOL[k]})"></span>${KNAME[k]}</span>`).join('') + '<span>letters pair glued edges; arrows run the same way on both</span>';
  if (ex.ov.has('corners')) h += '<span><span class="cn"></span>corner (copies share a number)</span>';
  if (ex.ov.has('walks')) h += `<span><span class="sw" style="background:var(--walk-eq)"></span>equator</span><span><span class="sw" style="background:var(--walk-pole)"></span>north over a pole</span><span><span class="sw" style="background:var(--walk-diag)"></span>45° from spawn</span><span>dots: out of and into a seam</span>`;
  if (ex.ov.has('tissot')) h += '<span>ellipses: a 6° disc of planet as drawn</span>';
  $('#worlds-legend').innerHTML = h;
}
function setupWorlds() {
  const seg = $('#layer-seg');
  for (const [k, t] of LAYERS) {
    const b = el('button', { type: 'button', 'aria-pressed': String(k === ex.layer), 'data-layer': k }, t);
    b.addEventListener('click', () => { ex.layer = k; $$('button', seg).forEach((x) => x.setAttribute('aria-pressed', String(x === b))); renderWorlds(); });
    seg.appendChild(b);
  }
  const ch = $('#overlay-checks');
  for (const [k, t] of OVS) {
    const lab = el('label', {}, `<input type="checkbox" ${ex.ov.has(k) ? 'checked' : ''}> ${t}`);
    lab.querySelector('input').addEventListener('change', (e) => { e.target.checked ? ex.ov.add(k) : ex.ov.delete(k); renderWorlds(); });
    ch.appendChild(lab);
  }
  const hx = location.hash.match(/layer-([a-z]+)/);
  if (hx && LAYERS.some((l) => l[0] === hx[1])) { ex.layer = hx[1]; $$('button', seg).forEach((x) => x.setAttribute('aria-pressed', String(x.dataset.layer === ex.layer))); }
  if (/ov-walks/.test(location.hash)) ex.ov.add('walks');
  renderWorlds();
}

// ---------------------------------------------------------------------------------------------
// corners up close
function cornerFrame(id, kind) {
  const C = D.corners[id];
  const box = el('div', { class: 'mapbox square' });
  const [X0, Y0, X1, Y1] = C.window_t;
  const w = X1 - X0;
  const s = sv('svg', { viewBox: `${X0} ${-Y1} ${w} ${Y1 - Y0}`, preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  const ub = D.layouts[id].unit_blocks;
  const T = (p) => [p[0] / ub, -p[1] / ub];   // overlay blocks -> svg (tile units, y down)
  const lw = w / 300;
  const defs = sv('defs');
  const pat = sv('pattern', { id: `hatch-${id}-${kind}`, width: w / 40, height: w / 40, patternUnits: 'userSpaceOnUse', patternTransform: 'rotate(45)' });
  pat.appendChild(sv('rect', { width: w / 40, height: w / 40, fill: css('--gap-bg') }));
  pat.appendChild(sv('line', { x1: 0, y1: 0, x2: 0, y2: w / 40, stroke: css('--gap-ink'), 'stroke-width': w / 140 }));
  defs.appendChild(pat);
  s.appendChild(defs);
  return { C, box, s, T, lw, w, X0, Y0, X1, Y1, ub, pat: `hatch-${id}-${kind}` };
}
function missingWedge(f, id) {
  const { C, s, X0, Y0, X1, Y1 } = f;
  const [vx, vy] = C.V;
  const pts = id === G ? [[X0, -Y1], [X1, -Y1], [X1, -vy], [X0, -vy]] : [[vx, -Y1], [X1, -Y1], [X1, -vy], [vx, -vy]];
  s.appendChild(sv('polygon', { points: pts.map((p) => p.join(',')).join(' '), fill: `url(#${f.pat})` }));
}
function cornerSeams(f, id, withLetters) {
  const { C, s, T, lw, w } = f;
  const L = D.layouts[id];
  for (const sm of L.seams) {
    const a = T(sm.a0), b = T(sm.a1);
    const col = css(KCOL[sm.kind]);
    s.appendChild(sv('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1], stroke: '#000', 'stroke-opacity': .5, 'stroke-width': lw * 3.4 }));
    s.appendChild(sv('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1], stroke: col, 'stroke-width': lw * 2.2 }));
  }
  for (const [a0, a1] of L.internal) {
    const a = T(a0), b = T(a1);
    s.appendChild(sv('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1], stroke: '#fff', 'stroke-opacity': .8, 'stroke-width': lw * 1.2, 'stroke-dasharray': `${w / 60} ${w / 80}` }));
  }
  const [vx, vy] = C.V;
  s.appendChild(sv('circle', { cx: vx, cy: -vy, r: w / 55, fill: css('--corner'), stroke: '#000', 'stroke-width': lw }));
}
function label(s, x, y, t, size, anchor = 'middle', col = '#fff') {
  s.appendChild(sv('text', { x, y, 'text-anchor': anchor, 'font-size': size, 'font-weight': 700, fill: col, stroke: 'rgba(0,0,0,.65)', 'stroke-width': size * 0.18, 'paint-order': 'stroke' }, t));
}
function faceLabels(f, id) {
  const { C, s, w } = f;
  const [vx, vy] = C.V;
  const o = w * 0.3, z = w * 0.045;
  if (id === G) {
    label(s, vx - o, -vy + o, 'west square', z); label(s, vx + o, -vy + o, 'east square', z);
    label(s, vx, -vy - o * 0.9, 'missing: 180°', z, 'middle', css('--gap-label'));
  } else {
    label(s, vx - o, -vy + o, 'F0 (band)', z); label(s, vx + o, -vy + o, 'F1 (band)', z); label(s, vx - o, -vy - o, 'N (polar)', z);
    label(s, vx + o, -vy - o, 'missing: 90°', z, 'middle', css('--gap-label'));
  }
}
function arcLabel(f, id) {
  const { C, s, w, lw } = f;
  const [vx, vy] = C.V;
  const r = w * 0.12;
  const [a0, a1] = id === G ? [180, 360] : [90, 360];
  const P = (a) => [vx + r * Math.cos(a * Math.PI / 180), -(vy + r * Math.sin(a * Math.PI / 180))];
  const p0 = P(a0), p1 = P(a1);
  s.appendChild(sv('path', { d: `M ${p0[0]} ${p0[1]} A ${r} ${r} 0 ${a1 - a0 > 180 ? 1 : 0} 0 ${p1[0]} ${p1[1]}`, fill: 'none', stroke: css('--corner'), 'stroke-width': lw * 1.6 }));
}
function cornerNbhd(id) {
  const f = cornerFrame(id, 'nbhd');
  f.box.appendChild(el('img', { src: `corner-${id}-nbhd.webp`, alt: `${NAME[id]}: the faces round a corner`, loading: 'lazy' }));
  missingWedge(f, id);
  cornerSeams(f, id);
  arcLabel(f, id);
  faceLabels(f, id);
  scaleBar(f);
  f.box.appendChild(f.s);
  return f.box;
}
function scaleBar(f) {
  const { s, X0, Y0, w, ub, lw } = f;
  const L = 500 / ub;
  const x = X0 + w * 0.05, y = -Y0 - w * 0.05;
  s.appendChild(sv('line', { x1: x, y1: y, x2: x + L, y2: y, stroke: '#fff', 'stroke-width': lw * 2 }));
  label(s, x + L / 2, y - w * 0.02, '500 blocks', w * 0.032);
}
function cornerPaths(id) {
  const f = cornerFrame(id, 'paths');
  const C = f.C;
  f.box.appendChild(el('img', { src: `corner-${id}-nbhd.webp`, alt: `${NAME[id]}: walkers and a ring round a corner`, loading: 'lazy', class: 'dim' }));
  missingWedge(f, id);
  cornerSeams(f, id);
  const P = (p) => [p[0], -p[1]];
  const cols = { A: css('--walk-a'), B: css('--walk-b') };
  for (const k of ['A', 'B']) {
    const w = C.walkers[k];
    w.segs_t.forEach((sg, i) => {
      polyline(f.s, sg.map(P), cols[k], f.lw * 2.2);
      if (sg.length > 2) arrowHead(f.s, P(sg[sg.length - 2]), P(sg[sg.length - 1]), cols[k], f.w * 0.035, '#000');
    });
    const p0 = w.segs_t[0][0];
    label(f.s, p0[0], -p0[1] + f.w * 0.05, k, f.w * 0.045, 'middle', cols[k]);
  }
  const ring = C.ring;
  for (const sg of ring.segs_t) {
    polyline(f.s, sg.map(P), css('--walk-ring'), f.lw * 2.2);
    if (sg.length > 2) { const m = Math.floor(sg.length / 2); arrowHead(f.s, P(sg[m - 1]), P(sg[m]), css('--walk-ring'), f.w * 0.032, '#000'); }
  }
  for (const c of ring.crossings) for (const p of [c.at, c.to]) f.s.appendChild(sv('circle', { cx: p[0] / f.ub, cy: -p[1] / f.ub, r: f.w / 90, fill: css('--walk-ring'), stroke: '#fff', 'stroke-width': f.lw * .7 }));
  f.box.appendChild(f.s);
  return f.box;
}
function cornerView(id) {
  const C = D.corners[id], V = C.view;
  const [a0, b0, a1, b1] = V.window_ab;
  const box = el('div', { class: 'mapbox square' });
  box.style.aspectRatio = `${a1 - a0} / ${b1 - b0}`;
  box.appendChild(el('img', { src: `corner-${id}-view.webp`, alt: `${NAME[id]}: what a player near a corner sees`, loading: 'lazy' }));
  box.appendChild(el('img', { src: `corner-${id}-viewtint.webp`, alt: '', loading: 'lazy' }));
  const w = a1 - a0;
  const s = sv('svg', { viewBox: `${a0} ${-b1} ${w} ${b1 - b0}`, preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  const lw = w / 300, d = V.d, th = C.cone * Math.PI / 180;
  const ray = (psi, col, dash) => {
    const dx = Math.sin(psi), dy = -Math.cos(psi);
    const L = 4000;
    s.appendChild(sv('line', { x1: 0, y1: -d, x2: dx * L, y2: -(d + dy * L), stroke: col, 'stroke-width': lw * 1.6, 'stroke-dasharray': dash || 'none' }));
  };
  ray(th / 2, '#fff', `${w / 70} ${w / 90}`);
  ray(-th / 2, '#fff', `${w / 70} ${w / 90}`);
  if (Math.abs(C.cone - 270) < 1) {
    s.appendChild(sv('line', { x1: 0, y1: -d, x2: 0, y2: -b1, stroke: css('--bad-hi'), 'stroke-width': lw * 2.4 }));
    label(s, w * 0.02, -(d + (b1 - d) * 0.8), 'tear', w * 0.032, 'start', css('--bad-hi'));
  }
  s.appendChild(sv('circle', { cx: 0, cy: -d, r: w / 70, fill: css('--corner'), stroke: '#000', 'stroke-width': lw }));
  label(s, w * 0.025, -d + w * 0.012, 'corner', w * 0.03, 'start');
  // the player and the direction of view
  s.appendChild(sv('circle', { cx: 0, cy: 0, r: w / 60, fill: '#fff', stroke: '#000', 'stroke-width': lw }));
  arrowHead(s, [0, 0], [0, -d * 0.55], '#fff', w * 0.03, '#000');
  label(s, w * 0.03, w * 0.012, 'you', w * 0.032, 'start');
  if (Math.abs(C.cone - 180) < 1) {
    s.appendChild(sv('circle', { cx: 0, cy: -2 * d, r: w / 60, fill: 'none', stroke: '#fff', 'stroke-width': lw * 1.6, 'stroke-dasharray': `${w / 120} ${w / 160}` }));
    label(s, w * 0.03, -2 * d + w * 0.012, 'you again (turned 180°)', w * 0.03, 'start');
  }
  const sb = 500, x = a0 + w * 0.05, y = -b0 - w * 0.04;
  s.appendChild(sv('line', { x1: x, y1: y, x2: x + sb, y2: y, stroke: '#fff', 'stroke-width': lw * 2 }));
  label(s, x + sb / 2, y - w * 0.018, '500 blocks', w * 0.03);
  box.appendChild(s);
  return box;
}
function cornerRaw(id) {
  const f = cornerFrame(id, 'raw');
  f.box.appendChild(el('img', { src: `corner-${id}-raw.webp`, alt: `${NAME[id]}: one raw copy round a corner`, loading: 'lazy' }));
  f.box.appendChild(el('img', { src: `corner-${id}-rawtear.webp`, alt: '', loading: 'lazy' }));
  cornerSeams(f, id);
  const [vx, vy] = f.C.V;
  const o = f.w * 0.3, z = f.w * 0.04;
  if (id === G) {
    label(f.s, vx - o, -vy - o * 1.1, 'east square,', z * 0.9);
    label(f.s, vx - o, -vy - o * 0.8, 'turned 180°', z * 0.9);
    label(f.s, vx + o, -vy - o * 1.1, 'west square,', z * 0.9);
    label(f.s, vx + o, -vy - o * 0.8, 'turned 180°', z * 0.9);
  } else {
    label(f.s, vx + o, -vy - o * 1.1, 'F1, turned 90°', z);
    label(f.s, vx + o, -vy - o * 0.75, '(N-east/F1 is first)', z * 0.8);
  }
  f.box.appendChild(f.s);
  return f.box;
}
function blockGrid(id) {
  const B = D.corners[id].blocks;
  const [Vx, Vz] = B.V;
  const n = 8, cs = 46, pad = 18;
  const W = n * cs + 2 * pad, H = n * cs + 2 * pad;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': `${NAME[id]}: block cells round a corner` });
  const FC = { F0: '--face-a', F1: '--face-b', N: '--face-c', W: '--face-a', E: '--face-b' };
  const ids = {};
  let li = 0;
  const letters = 'abcdefghijklmnopqrstuvwxyz';
  for (const c of B.cells) if (c.face && Math.abs(c.x + 0.5 - Vx) < 2 && Math.abs(c.z + 0.5 - Vz) < 2) ids[`${c.x},${c.z}`] = letters[li++];
  const X = (x) => pad + (x - (Vx - 4)) * cs, Z = (z) => pad + (z - (Vz - 4)) * cs;
  const defs = sv('defs');
  const pat = sv('pattern', { id: `bh-${id}`, width: 8, height: 8, patternUnits: 'userSpaceOnUse', patternTransform: 'rotate(45)' });
  pat.appendChild(sv('rect', { width: 8, height: 8, fill: css('--gap-bg') }));
  pat.appendChild(sv('line', { x1: 0, y1: 0, x2: 0, y2: 8, stroke: css('--gap-ink'), 'stroke-width': 2.5 }));
  defs.appendChild(pat);
  s.appendChild(defs);
  for (const c of B.cells) {
    const x = X(c.x), y = Z(c.z);
    if (c.face) {
      s.appendChild(sv('rect', { x, y, width: cs, height: cs, fill: css(FC[c.face] || '--panel-2'), stroke: css('--ink-3'), 'stroke-width': .6 }));
      const t = ids[`${c.x},${c.z}`];
      s.appendChild(sv('text', { x: x + cs / 2, y: y + cs / 2 + 4, 'text-anchor': 'middle', 'font-size': 12, 'font-weight': t ? 700 : 400, fill: css('--ink') }, t || c.face));
    } else {
      const multi = c.claims.length > 1 && new Set(c.claims.map((q) => q.to.join(','))).size > 1;
      s.appendChild(sv('rect', { x, y, width: cs, height: cs, fill: `url(#bh-${id})`, stroke: multi ? css('--bad-hi') : css('--ink-3'), 'stroke-width': multi ? 2 : .6 }));
      c.claims.slice(0, 2).forEach((q, i) => {
        const t = ids[q.to.join(',')] || q.face || '?';
        const turn = ['', '↻90', '↻180', '↺90'][q.k];
        s.appendChild(sv('text', { x: x + cs / 2, y: y + (c.claims.length > 1 ? 17 + i * 16 : cs / 2 + 4), 'text-anchor': 'middle', 'font-size': 11, 'font-weight': 600, fill: i === 0 ? css('--ink') : css('--bad-hi') }, `${t}${turn}`));
      });
    }
  }
  // face edges (thick) and the corner
  s.appendChild(sv('line', { x1: X(Vx), y1: pad, x2: X(Vx), y2: H - pad, stroke: css('--ink'), 'stroke-width': 1.4, 'stroke-dasharray': '4 3' }));
  s.appendChild(sv('line', { x1: pad, y1: Z(Vz), x2: W - pad, y2: Z(Vz), stroke: css('--ink'), 'stroke-width': 1.4, 'stroke-dasharray': '4 3' }));
  s.appendChild(sv('circle', { cx: X(Vx), cy: Z(Vz), r: 6, fill: css('--corner'), stroke: '#000', 'stroke-width': 1.2 }));
  s.appendChild(sv('text', { x: pad, y: 12, 'font-size': 11, fill: css('--ink-3') }, `x ${fmt(Vx - 4)}…${fmt(Vx + 3)} →, z ${fmt(Vz - 4)}…${fmt(Vz + 3)} ↓ (north up)`));
  return s;
}
function cornerNotes(id) {
  const B = D.corners[id].blocks;
  const items = [];
  for (const t of B.touching) {
    const nm = `(${fmt(t.x)}, ${fmt(t.z)}) in ${t.face}`;
    items.push(`<li><b>${nm}</b>: ${t.notes.length ? t.notes.join('; ') : 'all eight neighbours are ordinary and distinct'}.</li>`);
  }
  return `<ul class="tight">${items.join('')}</ul>`;
}
function cornersSection() {
  for (const id of [G, K]) {
    $(`#cn-nbhd-${id}`).appendChild(cornerNbhd(id));
    $(`#cn-paths-${id}`).appendChild(cornerPaths(id));
    $(`#cn-view-${id}`).appendChild(cornerView(id));
    $(`#cn-raw-${id}`).appendChild(cornerRaw(id));
    $(`#cn-blocks-${id}`).appendChild(blockGrid(id));
    $(`#cn-notes-${id}`).innerHTML = cornerNotes(id);
  }
}

// ---------------------------------------------------------------------------------------------
// plane tiling
function tilingFigure() {
  const T = D.tiling.guyou;
  const [X0, Y0, X1, Y1] = T.window_t;
  const box = $('#tile-guyou');
  box.style.aspectRatio = `${X1 - X0} / ${Y1 - Y0}`;
  box.appendChild(el('img', { src: 'tiling-guyou.webp', alt: 'The plane tiled by copies of the Petroff–Guyou world', loading: 'lazy' }));
  const s = sv('svg', { viewBox: `${X0} ${-Y1} ${X1 - X0} ${Y1 - Y0}`, preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  const lw = (X1 - X0) / 600;
  for (let i = -2; i <= 2; i++) for (let j = -2; j <= 2; j++) {
    for (const turned of [false, true]) {
      const x = -1 + 4 * i, y = (turned ? 1 : -1) + 4 * j;
      if (x > X1 || x + 4 < X0 || y > Y1 || y + 2 < Y0) continue;
      const home = !turned && i === 0 && j === 0;
      s.appendChild(sv('rect', { x, y: -(y + 2), width: 4, height: 2, fill: 'none', stroke: home ? css('--corner') : '#fff', 'stroke-opacity': home ? 1 : .7, 'stroke-width': lw * (home ? 3 : 1.2) }));
      if (turned) label(s, x + 2, -(y + 1) + 0.06, 'turned 180°', 0.17);
      else if (home) label(s, x + 2, -(y + 1) + 0.06, 'the world', 0.2, 'middle', css('--corner'));
    }
  }
  for (let x = -5; x <= 7; x += 2) for (let y = -3; y <= 3; y += 2) s.appendChild(sv('circle', { cx: x, cy: -y, r: 0.07, fill: css('--corner'), stroke: '#000', 'stroke-width': lw }));
  box.appendChild(s);
  // Guyou corner, full turn
  const gc = $('#tile-guyou-corner');
  gc.appendChild(el('img', { src: 'tiling-guyou-corner.webp', alt: 'A Guyou corner and its full turn', loading: 'lazy' }));
  const s2 = sv('svg', { viewBox: '0 -2 2 2', preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  s2.appendChild(sv('line', { x1: 0, y1: -1, x2: 2, y2: -1, stroke: css('--half'), 'stroke-width': 0.012 }));
  s2.appendChild(sv('line', { x1: 1, y1: 0, x2: 1, y2: -2, stroke: '#fff', 'stroke-width': 0.006, 'stroke-dasharray': '0.03 0.02' }));
  s2.appendChild(sv('circle', { cx: 1, cy: -1, r: 0.035, fill: css('--corner'), stroke: '#000', 'stroke-width': 0.006 }));
  label(s2, 0.5, -0.45, 'west', 0.07); label(s2, 1.5, -0.45, 'east', 0.07);
  label(s2, 0.5, -1.55, 'east, turned', 0.07); label(s2, 1.5, -1.55, 'west, turned', 0.07);
  gc.appendChild(s2);
  // cube corner: three faces and two candidates
  const cc = $('#tile-cube-corner');
  cc.appendChild(el('img', { src: 'tiling-cube-mix.webp', alt: 'A cube corner: the missing quarter has two candidate pictures', loading: 'lazy' }));
  const s3 = sv('svg', { viewBox: '0 -2 2 2', preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  s3.appendChild(sv('line', { x1: 1, y1: -1, x2: 1, y2: -2, stroke: css('--quarter'), 'stroke-width': 0.012 }));
  s3.appendChild(sv('line', { x1: 1, y1: -1, x2: 2, y2: -1, stroke: css('--quarter'), 'stroke-width': 0.012 }));
  s3.appendChild(sv('circle', { cx: 1, cy: -1, r: 0.035, fill: css('--corner'), stroke: '#000', 'stroke-width': 0.006 }));
  label(s3, 0.5, -0.45, 'F0', 0.08); label(s3, 1.5, -0.45, 'F1', 0.08); label(s3, 0.5, -1.55, 'N', 0.08);
  label(s3, 1.5, -1.6, 'F1 turned', 0.065); label(s3, 1.5, -1.45, 'or N turned?', 0.065, 'middle', css('--gap-label'));
  cc.appendChild(s3);
  for (const k of ['a', 'b']) {
    const b = $(`#tile-cube-${k}`);
    b.appendChild(el('img', { src: `tiling-cube-${k}.webp`, alt: '', loading: 'lazy' }));
  }
  // the cube net's raw surroundings
  const R = D.tiling.cube_rawext;
  const [rx0, ry0, rx1, ry1] = R.window_t;
  const rb = $('#rawext-cube');
  rb.style.aspectRatio = `${rx1 - rx0} / ${ry1 - ry0}`;
  rb.appendChild(el('img', { src: 'rawext-cube.webp', alt: 'The cube net and the raw world round it under the engine\'s edge order', loading: 'lazy' }));
  rb.appendChild(el('img', { src: 'rawext-cube-tear.webp', alt: '', loading: 'lazy' }));
  const s4 = sv('svg', { viewBox: `${rx0} ${-ry1} ${rx1 - rx0} ${ry1 - ry0}`, preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  const tiles = [[0, 0, 'F0'], [1, 0, 'F1'], [2, 0, 'F2'], [3, 0, 'F3'], [0, 1, 'N'], [0, -1, 'S']];
  for (const [x, y, t] of tiles) {
    s4.appendChild(sv('rect', { x, y: -(y + 1), width: 1, height: 1, fill: 'none', stroke: css('--corner'), 'stroke-width': 0.012 }));
    label(s4, x + 0.5, -(y + 0.5) + 0.04, t, 0.12, 'middle', css('--corner'));
  }
  rb.appendChild(s4);
}

// ---------------------------------------------------------------------------------------------
// tables and charts
function seamTable(id, box) {
  const L = D.layouts[id];
  const letters = 'ABCDEFGHIJ';
  const term = (a, v) => a === 0 ? '' : (a === 1 ? `+${v}` : a === -1 ? `−${v}` : `${a > 0 ? '+' : '−'}${Math.abs(a)}${v}`);
  const f = (row, t) => {
    let s = `${term(row[0], 'x')}${term(row[1], 'z')}`.replace(/^\+/, '');
    if (t) s += ` ${t > 0 ? '+' : '−'} ${fmt(Math.abs(t))}`;
    return s;
  };
  const rows = L.seam_mc.map((m) => `<tr><td><span class="pill" style="background:var(${KCOL[m.kind]})">${letters[m.pair]}</span> ${m.name}</td><td><span class="kind ${m.kind}">${KNAME[m.kind]}</span></td>
    <td class="mono">x′ = ${f(m.M[0], m.T[0])}<br>z′ = ${f(m.M[1], m.T[1])}</td><td class="n">${m.dyaw > 0 ? '+' : ''}${m.dyaw}°</td>
    <td class="mono n">(${fmt(m.mid[0])}, ${fmt(m.mid[1])}) → (${fmt(m.mid_to[0])}, ${fmt(m.mid_to[1])})</td></tr>`).join('');
  box.innerHTML = `<table><thead><tr><th>Seam (edges joined)</th><th>Kind</th><th>F3 coordinates jump to</th><th class="n">Yaw</th><th class="n">Example: edge midpoint</th></tr></thead><tbody>${rows}</tbody></table>`;
}
function tripTable() {
  const heads = D.layouts[G].trips.map((t) => t.heading);
  const cell = (t) => {
    const p = t.per10k;
    const parts = [];
    if (p.slide) parts.push(`<span class="kind slide">${fmt(p.slide, 1)} slide</span>`);
    if (p.half) parts.push(`<span class="kind half">${fmt(p.half, 1)} half</span>`);
    if (p.quarter) parts.push(`<span class="kind quarter">${fmt(p.quarter, 1)} quarter</span>`);
    return parts.join('<br>') || 'none';
  };
  const rows = heads.map((h, i) => `<tr><td class="n">${h === 0 ? 'due east (0°)' : h === 90 ? 'due north (90°)' : `${h}° north of east`}</td><td>${cell(D.layouts[G].trips[i])}</td><td>${cell(D.layouts[K].trips[i])}</td></tr>`).join('');
  $('#trip-table').innerHTML = `<table><thead><tr><th class="n">Straight course from spawn (raw heading)</th><th>${NAME[G]}: seams per 10,000 blocks</th><th>${NAME[K]}: seams per 10,000 blocks</th></tr></thead><tbody>${rows}</tbody></table>`;
}
function walkSummary(id, k) {
  const w = D.layouts[id].walks[k];
  const cr = w.crossings.map((c) => `<span class="kind ${c.kind}">${KNAME[c.kind]}</span> at ${fmt(c.dist)} (yaw ${c.yaw[0]}° → ${c.yaw[1]}°, F3 (${fmt(c.mc_at[0])}, ${fmt(c.mc_at[1])}) → (${fmt(c.mc_to[0])}, ${fmt(c.mc_to[1])}))`);
  return { blocks: w.blocks, deg: w.planet_deg, text: cr.length ? cr.join('; ') : 'no seam' };
}
function walksTable() {
  const rows = [['equator', 'Due east along the equator, once round'], ['pole', 'North from the equator, over the pole, to the equator beyond'], ['poleB', 'North from another point of the equator'], ['diag', 'Straight at 45° from spawn, 24,000 blocks']];
  const cell = (id, k) => { const s = walkSummary(id, k); return `<b>${fmt(s.blocks)} blocks</b> for ${fmt(s.deg, 0)}° of planet<span class="sub">${s.text}</span>`; };
  $('#walks-table').innerHTML = `<table><thead><tr><th>Walk</th><th>${NAME[G]}</th><th>${NAME[K]}</th></tr></thead><tbody>${rows.map(([k, t]) => `<tr><td><b>${t}</b></td><td>${cell(G, k)}</td><td>${cell(K, k)}</td></tr>`).join('')}</tbody></table>`;
}
function eqChart() {
  const series = [[G, NAME[G], css('--half')], [K, NAME[K], css('--quarter')]];
  const W = 600, H = 250, ml = 46, mr = 14, mt = 34, mb = 40;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': 'Blocks per degree along the equator' });
  const ymin = 25, ymax = 55;
  const xs = (f) => ml + f * (W - ml - mr), ys = (v) => mt + (1 - (v - ymin) / (ymax - ymin)) * (H - mt - mb);
  for (let v = ymin; v <= ymax; v += 5) {
    s.appendChild(sv('line', { x1: ml, x2: W - mr, y1: ys(v), y2: ys(v), stroke: css('--rule'), 'stroke-width': .7 }));
    s.appendChild(sv('text', { x: ml - 6, y: ys(v) + 4, 'text-anchor': 'end', 'font-size': 11, fill: css('--ink-3') }, v));
  }
  s.appendChild(sv('line', { x1: ml, x2: W - mr, y1: ys(45.51), y2: ys(45.51), stroke: css('--ink-3'), 'stroke-width': 1.2, 'stroke-dasharray': '5 4' }));
  s.appendChild(sv('text', { x: W - mr, y: ys(45.51) - 5, 'text-anchor': 'end', 'font-size': 11, fill: css('--ink-3') }, 'true scale (today\'s Lambert equator): 45.5'));
  for (let f = 0; f <= 1.0001; f += 0.25) s.appendChild(sv('text', { x: xs(f), y: H - mb + 16, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-3') }, `${Math.round(f * 360)}°`));
  s.appendChild(sv('text', { x: (ml + W - mr) / 2, y: H - 6, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-3') }, 'longitude walked along the equator'));
  series.forEach(([id, lab, col], i) => {
    const w = D.layouts[id].walks.equator;
    let deg = 0, prev = 0;
    const pts = [];
    for (const [d, bpd] of w.profile) { deg += (d - prev) / bpd; prev = d; pts.push([deg / 360, bpd]); }
    s.appendChild(sv('polyline', { points: pts.map(([f, v]) => `${xs(Math.min(f, 1))},${ys(Math.max(ymin, Math.min(ymax, v)))}`).join(' '), fill: 'none', stroke: col, 'stroke-width': 2.2 }));
    s.appendChild(sv('rect', { x: ml + 8 + i * 270, y: 10, width: 14, height: 3, fill: col }));
    s.appendChild(sv('text', { x: ml + 28 + i * 270, y: 15, 'font-size': 11.5, fill: css('--ink') }, `${lab}: ${fmt(w.blocks)} blocks round`));
  });
  $('#eq-chart').appendChild(s);
}
function sweepChart(box, rows, chosen, xmax, xl) {
  const W = 600, H = 170, ml = 40, mr = 10, mt = 10, mb = 34;
  const s = sv('svg', { viewBox: `0 0 ${W} ${H}`, width: '100%', role: 'img', 'aria-label': 'Corner placement sweep' });
  const xs = (v) => ml + v / xmax * (W - ml - mr), ys = (v) => mt + (1 - v) * (H - mt - mb);
  for (const v of [0, 0.5, 1]) {
    s.appendChild(sv('line', { x1: ml, x2: W - mr, y1: ys(v), y2: ys(v), stroke: css('--rule'), 'stroke-width': .7 }));
    s.appendChild(sv('text', { x: ml - 6, y: ys(v) + 4, 'text-anchor': 'end', 'font-size': 11, fill: css('--ink-3') }, `${v * 100}%`));
  }
  const bw = (W - ml - mr) / rows.length;
  for (const [lam, mx] of rows) {
    s.appendChild(sv('rect', { x: xs(lam), y: ys(mx), width: Math.max(1, bw - 0.5), height: Math.max(0.0001, ys(0) - ys(mx)), fill: css('--ink-3'), 'fill-opacity': 0.6 }));
    if (mx === 0) s.appendChild(sv('rect', { x: xs(lam), y: ys(0) - 4, width: Math.max(1, bw - 0.5), height: 4, fill: css('--rec') }));
  }
  s.appendChild(sv('line', { x1: xs(chosen), x2: xs(chosen), y1: mt, y2: ys(0), stroke: css('--accent'), 'stroke-width': 1.5, 'stroke-dasharray': '4 3' }));
  const ok = rows.filter((r) => r[1] === 0).length / rows.length;
  s.appendChild(sv('text', { x: xs(chosen) + 5, y: mt + 12, 'font-size': 11, fill: css('--accent') }, `used here: ${chosen}°`));
  for (let v = 0; v <= xmax; v += xmax / 6) s.appendChild(sv('text', { x: xs(v), y: H - mb + 15, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-3') }, Math.round(v)));
  s.appendChild(sv('text', { x: (ml + W) / 2, y: H - 4, 'text-anchor': 'middle', 'font-size': 11, fill: css('--ink-3') }, `${xl} · green: every corner's 3° disc is open sea (${Math.round(ok * 100)} % of turns)`));
  box.appendChild(s);
}
function mapGridNote() {
  // vanilla map grid: a map of scale k covers 128·2^k blocks; its edges sit at multiples of that size minus 64
  const rows = [];
  const pos = { [G]: [[6544, 'east/west edges x = ±6,544'], [3272, 'north/south edges z = ±3,272']], [K]: [[3776, 'face edges x, z = multiples of 3,776']] };
  for (const id of [G, K]) for (const [v, lab] of pos[id]) {
    const offs = [0, 1, 2, 3, 4].map((k) => { const sz = 128 << k; const o = ((v + 64) % sz + sz) % sz; return Math.min(o, sz - o); });
    rows.push(`<tr><td>${NAME[id]}: ${lab}</td>${offs.map((o) => `<td class="n">${o === 0 ? '<b>on a map edge</b>' : `${fmt(o)} blocks in`}</td>`).join('')}</tr>`);
  }
  $('#mapgrid-table').innerHTML = `<table><thead><tr><th>Seam line</th>${[0, 1, 2, 3, 4].map((k) => `<th class="n">scale ${k} (${fmt(128 << k)})</th>`).join('')}</tr></thead><tbody>${rows.join('')}</tbody></table>`;
}
function statsTables() {
  const S = D.stats;
  const r = (lab, f) => `<tr><td>${lab}</td><td class="n">${f(S[G])}</td><td class="n">${f(S[K])}</td></tr>`;
  $('#felt-table').innerHTML = `<table><thead><tr><th>On this planet (weighted by planet area)</th><th class="n">${NAME[G]}</th><th class="n">${NAME[K]}</th></tr></thead><tbody>
    ${r('Land drawn with shapes more than 1.25× too long', (s) => pct(s.land_shape_gt['1.25']))}
    ${r('… more than 1.5×', (s) => pct(s.land_shape_gt['1.5']))}
    ${r('… more than 2×', (s) => pct(s.land_shape_gt['2.0']))}
    ${r('Land shape ratio: median / 90th / 99th percentile', (s) => s.land_shape_q50_90_99.map((v) => fmt(v, 2)).join(' / '))}
    ${r('Whole planet: median / 90th / 99th percentile', (s) => s.all_shape_q50_90_99.map((v) => fmt(v, 2)).join(' / '))}
  </tbody></table>`;
  $('#north-table').innerHTML = `<table><thead><tr><th>F3's north (−z) against true north</th><th class="n">${NAME[G]}</th><th class="n">${NAME[K]}</th></tr></thead><tbody>
    ${r('Planet within 5° of true north', (s) => pct(s.north_within['5']))}
    ${r('… within 15°', (s) => pct(s.north_within['15']))}
    ${r('… within 45°', (s) => pct(s.north_within['45']))}
    ${r('Land within 15°', (s) => pct(s.land_north_within['15']))}
    ${r('Median / 90th percentile error', (s) => s.north_q50_90.map((v) => `${fmt(v, 0)}°`).join(' / '))}
  </tbody></table>`;
  $('#dist-table').innerHTML = `<table><thead><tr><th>Walking from the shared spawn</th><th class="n">${NAME[G]}</th><th class="n">${NAME[K]}</th></tr></thead><tbody>
    ${r('Spawn in F3 coordinates', (s) => `(${fmt(s.dist.spawn_mc[0])}, ${fmt(s.dist.spawn_mc[1])})`)}
    ${r('Farthest point (blocks; the planet\'s own: ' + fmt(S[G].dist.planet_max) + ')', (s) => fmt(s.dist.max_walk))}
    ${r('Median walking distance to a random place', (s) => fmt(s.dist.q50_walk))}
    ${r('Walk ÷ planet distance: 10th / 50th / 90th percentile', (s) => s.dist.ratio_q10_50_90.map((v) => fmt(v, 2)).join(' / '))}
    ${r('Places more than 20 % farther to walk than on the planet', (s) => pct(s.dist.ratio_gt_1_2))}
    ${r('Places more than 15 % nearer to walk', (s) => pct(s.dist.ratio_lt_0_85))}
    ${r('Compass more than 45° off the way back', (s) => pct(s.dist.compass_gt_45))}
    ${r('Compass more than 90° off (points away)', (s) => pct(s.dist.compass_gt_90))}
  </tbody></table>`;
  const b = (s, i) => `${fmt(s.bow[i].bow_blocks)} blocks (${fmt(s.bow[i].min_blocks_from_eq)} to ${fmt(s.bow[i].max_blocks_from_eq)} from the equator)`;
  $('#bands-table').innerHTML = `<table><thead><tr><th>How far a parallel strays across the world (z)</th><th class="n">${NAME[G]}</th><th class="n">${NAME[K]}</th></tr></thead><tbody>
    ${r('15° parallel', (s) => b(s, 0))}${r('30° parallel', (s) => b(s, 1))}</tbody></table>`;
}

// ---------------------------------------------------------------------------------------------
// appendix
const APPX = [
  ['lambert', 'Lambert cylindrical (today)', 'Equal-area, shipped, not seamless: poles are walls, shapes up to 27× near the 80° stop.', '1 slide', '1.33 / 27'],
  ['guyou-peirce', 'Guyou conformal (the prototype)', 'Runs in game today, but not equal-area: land near its 4 points drawn without bound larger.', '1 slide, 2 half', 'area 0.46× to ∞'],
  ['peirce-square', 'Peirce quincuncial square', 'Same map, square aspect; four turning edges, no straight equator. Guyou chosen 2026-10-03.', '4 half', 'area 0.46× to ∞'],
  ['guyou-gringorten', 'Gringorten in Guyou\'s layout', 'Equal-area but creased 62° exactly on its seams; worse than Petroff on every number.', '1 slide, 2 half', '1.79 / 2.88'],
  ['guyou-collignon', 'Collignon quincuncial', 'Simplest equal-area square; worst shapes, 63° creases on every seam and diagonal.', '1 slide, 2 half', '2.05 / 2.98'],
  ['cube-moser-3x2', 'Moser cube, 3 × 2 net', 'Same map packed into a rectangle; 8 seam pairs, seams inside the world, no world-wide slide for Toroidal World.', '2 slides, 4 quarter, 2 half', '1.26 / 1.42'],
  ['cube-qsc-cross', 'QSC (= Roşca–Plonka) cube', 'Published closed form, exactly equal-area; folds every ridge 41° on each face diagonal. The cube\'s fallback map.', 'as the cross', '1.32 / 1.55'],
  ['cube-healpix-cross', 'HEALPix cube', 'Best median shape, worst creases of the equal-area cubes (63° across the polar faces).', 'as the cross', '1.18 / 2.41'],
  ['cube-eac-cross', 'Equi-angular cube (S2)', 'The graphics cube: cheap, not equal-area (±20 %), creased 44–47° at every face edge.', 'as the cross', '1.20 / 1.73'],
  ['cube-conformal-cross', 'Conformal cube', 'The Moser cube\'s parent: true shapes, but area grows without bound at the corners.', 'as the cross', 'area 0.79× to ∞'],
];
function appendix() {
  const rows = APPX.map(([id, name, why, seams, shape]) => {
    const A = D.appendix[id];
    const ar = A ? `${A.bbox[2] - A.bbox[0]} / ${A.bbox[3] - A.bbox[1]}` : '2 / 1';
    return `<tr><td class="thumb"><div class="thumbbox" style="aspect-ratio:${ar}">${A ? `<img src="${id}-map.webp" alt="" loading="lazy">` : ''}</div></td>
      <td><b>${name}</b><span class="sub">${why}</span></td><td>${seams}</td><td class="n">${shape}</td>
      <td class="thumb"><div class="thumbbox" style="aspect-ratio:${ar}">${A ? `<img src="${id}-crease.webp" alt="" loading="lazy">` : ''}</div></td></tr>`;
  }).join('');
  $('#appx-table').innerHTML = `<table><thead><tr><th>Planet</th><th>Candidate and why not</th><th>Seams</th><th class="n">Shape a/b median / max</th><th>Crease test</th></tr></thead><tbody>${rows}</tbody></table>`;
}

// ---------------------------------------------------------------------------------------------
function init() {
  const atSea = (rows) => rows.filter((r) => r[1] === 0).length / rows.length;
  D.corner_sea = { guyou: atSea(D.sweep_guyou), cube: atSea(D.sweep_cube) };
  bind();
  coneFigure();
  setupWorlds();
  cornersSection();
  tilingFigure();
  seamTable(G, $('#seams-guyou'));
  seamTable(K, $('#seams-cube'));
  tripTable();
  walksTable();
  eqChart();
  sweepChart($('#sweep-cube'), D.sweep_cube, D.lam0_deg, 90, 'λ₀ (degrees)');
  sweepChart($('#sweep-guyou'), D.sweep_guyou, D.lamc_deg, 360, 'λc (degrees)');
  mapGridNote();
  statsTables();
  appendix();
  $('#foot').innerHTML = `Generated ${D.generated} on the test VM from <span class="mono">tools/review/world-shape/</span> · Deep Time (LGPL-3.0-or-later) · three.js r128 (MIT), local copy · distortion figures: seamless-projection-survey.md.`;
}
init();
window.WS = { css, NAME, G, K, D };
try { window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => location.reload()); } catch (e) { /* older browsers */ }
})();
