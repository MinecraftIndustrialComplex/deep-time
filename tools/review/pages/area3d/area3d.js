// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time area preview (P1-20 stage 2): renders a `deeptime-inspect export-area` file (.dtva,
// gzipped or not; the format is documented in crates/deeptime-inspect/src/area.rs) as block
// voxels with three.js. Greedy-meshed in 32³ sections; a section is re-meshed only when a cut, a
// toggle or the colour mode changes what it shows. Click a block for its id, position, rock
// horizon, deposit body and the site's "why here" chain.
//
// Load: ?src=<url>, the Open… button, drag and drop, or exports.json next to the page
// ([{"src", "label"}]). Embedding: window.Area3D.load(arrayBuffer | url, label).
// URL state for review shots: &preset=all|nostone|ores &colour=block|strata|body &cuty=<y>
// &sec=x|z &secpos=<unwrapped x or z> &flip=1 &site=<id> (cut through that site) &bodies=1
// &pick=x,y,z (select a block) &cam=<azimuth deg>,<elevation deg>,<distance factor>.

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

const S = 32;                                   // section edge in blocks
const CATS = ['rock', 'crust', 'soil', 'plant', 'water', 'lava', 'ore'];
const CAT_LABEL = {
  rock: 'Rock (painted strata, other stone)', crust: 'Crust (stone, deepslate)', soil: 'Soil and sand',
  plant: 'Plants, trees, snow', water: 'Water', lava: 'Lava', ore: 'Ore',
};
const PRESETS = {
  all: { rock: 1, crust: 1, soil: 1, plant: 1, water: 1, lava: 1, ore: 1, bodies: 0 },
  nostone: { rock: 0, crust: 0, soil: 0, plant: 0, water: 0, lava: 1, ore: 1, bodies: 0 },
  ores: { rock: 0, crust: 0, soil: 0, plant: 0, water: 0, lava: 0, ore: 1, bodies: 1 },
};
// Face shading: +x −x +y −y +z −z.
const SHADE = [0.80, 0.80, 1.00, 0.55, 0.68, 0.68];

const $ = (id) => document.getElementById(id);
const params = new URLSearchParams(location.search);
const esc = (s) => String(s ?? '').replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const fmt = (n) => Number(n).toLocaleString('en-US');
const short = (id) => String(id ?? '').replace(/^[a-z0-9_.-]+:/, '');

// ---------------------------------------------------------------------------------------------
// Theme.
// ---------------------------------------------------------------------------------------------
const THEMES = ['auto', 'light', 'dark'];
let theme = 'auto';
try { theme = localStorage.getItem('area3d-theme') || 'auto'; } catch { /* storage blocked */ }
if (params.get('theme')) theme = params.get('theme');
function applyTheme() {
  if (theme === 'auto') delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
  $('theme').textContent = { auto: '◐', light: '☀', dark: '☾' }[theme];
  if (renderer) scene.background = new THREE.Color(getComputedStyle(document.documentElement).getPropertyValue('--scene').trim());
}
$('theme').onclick = () => {
  theme = THEMES[(THEMES.indexOf(theme) + 1) % 3];
  try { localStorage.setItem('area3d-theme', theme); } catch { /* ignore */ }
  applyTheme();
};
matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyTheme);

// ---------------------------------------------------------------------------------------------
// Decoding.
// ---------------------------------------------------------------------------------------------
async function inflate(buf) {
  const u = new Uint8Array(buf);
  if (u[0] === 0x1f && u[1] === 0x8b) {
    const stream = new Blob([u]).stream().pipeThrough(new DecompressionStream('gzip'));
    return await new Response(stream).arrayBuffer();
  }
  return buf;
}

function decode(buf) {
  const dv = new DataView(buf);
  const magic = String.fromCharCode(...new Uint8Array(buf, 0, 4));
  if (magic !== 'DTVA') throw new Error('not a Deep Time area export (DTVA)');
  const version = dv.getUint32(4, true);
  if (version !== 1) throw new Error(`DTVA version ${version}; this viewer reads 1`);
  const hl = dv.getUint32(8, true);
  const h = JSON.parse(new TextDecoder().decode(new Uint8Array(buf, 12, hl)));
  const blob = Math.ceil((12 + hl) / 4) * 4;
  const sec = (name, T) => {
    const s = h.sections[name];
    return s ? new T(buf, blob + s.offset, s.count) : null;
  };
  const counts = sec('run_counts', Uint16Array), vals = sec('run_values', Uint16Array), lens = sec('run_lengths', Uint16Array);
  const W = h.width, D = h.depth, H = h.height;
  const entBlock = new Uint16Array(h.entries.length), entTag = new Uint16Array(h.entries.length);
  h.entries.forEach(([b, t], e) => { entBlock[e] = b; entTag[e] = t; });
  const nonAir = new Uint8Array(h.entries.length);
  for (let e = 0; e < h.entries.length; e++) nonAir[e] = h.blocks[entBlock[e]].cat !== 'air' ? 1 : 0;
  // The rows holding anything but air: the dense grid covers only those.
  let lo = H, hi = -1, r = 0;
  for (let c = 0; c < W * D; c++) {
    let y = 0;
    for (let k = 0; k < counts[c]; k++, r++) {
      if (nonAir[vals[r]]) { if (y < lo) lo = y; if (y + lens[r] - 1 > hi) hi = y + lens[r] - 1; }
      y += lens[r];
    }
  }
  if (hi < lo) { lo = 0; hi = 0; }
  const Hc = hi - lo + 1;
  const dense = new Uint16Array(W * D * Hc);
  r = 0;
  for (let c = 0; c < W * D; c++) {
    let y = 0;
    const base = c * Hc - lo;
    for (let k = 0; k < counts[c]; k++, r++) {
      const v = vals[r], a = Math.max(y, lo), b = Math.min(y + lens[r], hi + 1);
      if (v !== 0 && b > a) dense.fill(v, base + a, base + b);
      y += lens[r];
    }
  }
  const tops = sec('strata_tops', Int16Array), classes = sec('strata_classes', Uint8Array);
  const K = h.strata ? h.strata.k : 0;
  const bodyById = new Map((h.bodies || []).map((b, i) => [b.id, Object.assign(b, { index: i })]));
  const siteById = new Map((h.sites || []).map((s) => [s.id, s]));
  const members = {};
  for (const [cls, states] of Object.entries(h.palette?.classes || {})) members[cls] = states.map((s) => s.split('[')[0]);
  return { h, W, D, H, Hc, yLo: lo, y0: h.y0, dense, entBlock, entTag, tops, classes, K, bodyById, siteById, members };
}

// ---------------------------------------------------------------------------------------------
// Scene.
// ---------------------------------------------------------------------------------------------
let renderer, scene, camera, controls, group, highlight, seamLine;
let A = null;                                    // the loaded area
let sections = [];                               // {sx, sy, sz, opaque, trans, key}
let queue = [];
let version = 0;
const view = { cats: { ...PRESETS.all }, bodies: false, colour: 'block', vividOres: true, cutY: 0, secAxis: 'none', secPos: 0, flip: false };
const opaqueMat = new THREE.MeshBasicMaterial({ vertexColors: true });
const transMat = new THREE.MeshBasicMaterial({ vertexColors: true, transparent: true, opacity: 0.42, depthWrite: false, side: THREE.DoubleSide });

function initScene() {
  const host = $('scene');
  renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  host.appendChild(renderer.domElement);
  scene = new THREE.Scene();
  camera = new THREE.PerspectiveCamera(45, 1, 0.5, 20000);
  controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.12;
  group = new THREE.Group();
  scene.add(group);
  const hg = new THREE.EdgesGeometry(new THREE.BoxGeometry(1.06, 1.06, 1.06));
  highlight = new THREE.LineSegments(hg, new THREE.LineBasicMaterial({ color: 0xff2d55, depthTest: false }));
  highlight.renderOrder = 10;
  highlight.visible = false;
  group.add(highlight);
  applyTheme();
  const resize = () => {
    const w = host.clientWidth, hgt = host.clientHeight;
    renderer.setSize(w, hgt);
    camera.aspect = w / Math.max(1, hgt);
    camera.updateProjectionMatrix();
  };
  new ResizeObserver(resize).observe(host);
  resize();
  let down = null;
  renderer.domElement.addEventListener('pointerdown', (e) => { down = [e.clientX, e.clientY]; });
  renderer.domElement.addEventListener('pointerup', (e) => {
    if (down && Math.hypot(e.clientX - down[0], e.clientY - down[1]) < 5) pick(e);
    down = null;
  });
  renderer.setAnimationLoop(frame);
}

function frame() {
  const t0 = performance.now();
  while (queue.length && performance.now() - t0 < 14) buildSection(queue.shift());
  const left = queue.length;
  if (A) status(left ? `Meshing… ${left} sections left` : A.statusLine);
  controls.update();
  renderer.render(scene, camera);
}

function status(text) { const s = $('status'); if (s.textContent !== text) s.textContent = text; }

// ---------------------------------------------------------------------------------------------
// What each voxel shows.
// ---------------------------------------------------------------------------------------------
let entCls, entCol;                             // per entry: 0 hidden, 1 opaque, 2 see-through; colour id
const colourCache = new Map();

function hexToRgb(hex) {
  const v = parseInt(hex.slice(1, 7), 16);
  return [(v >> 16) / 255, ((v >> 8) & 255) / 255, (v & 255) / 255];
}
function hueRgb(i, s = 0.55, l = 0.55) {
  const c = new THREE.Color().setHSL(((i * 0.61803398875) % 1 + 1) % 1, s, l);
  return [c.r, c.g, c.b];
}
// Ore blocks' textures average to the stone they sit in, so "vivid ores" paints them by material.
const ORE_RGB = {
  copper: '#e8703a', iron: '#d9a27a', gold: '#ffd21f', coal: '#1c1c1c', diamond: '#3fe0e8', lapis: '#2446c8',
  redstone: '#e0201b', emerald: '#1fd66b', zinc: '#9fd4c8', lead: '#6f6ad0', silver: '#e9eef2', tin: '#c9d3dd',
  uranium: '#9bff3a', nickel: '#c7c28a', quartz: '#f1ebe0', netherite: '#5a3a33',
};
// Colour ids: block b → b; horizon class k → 10000 + k; body index i → 20000 + i; grey block → 30000 + b.
function rgbOf(id) {
  let c = colourCache.get(id);
  if (c) return c;
  if (id < 10000 && view.vividOres && A.h.blocks[id].cat === 'ore') {
    const m = A.h.blocks[id].ore;
    c = ORE_RGB[m] ? hexToRgb(ORE_RGB[m]) : hueRgb([...String(m)].reduce((a, ch) => a * 31 + ch.charCodeAt(0), 7), 0.9, 0.55);
  } else if (id < 10000) c = hexToRgb(A.h.blocks[id].color);
  else if (id < 20000) c = hueRgb(id - 10000, 0.45, 0.58);
  else if (id < 30000) c = hueRgb(id - 20000 + 3, 0.85, 0.52);
  else { const [r, g, b] = hexToRgb(A.h.blocks[id - 30000].color); const y = 0.3 * r + 0.59 * g + 0.11 * b; c = [y * 0.85 + 0.1, y * 0.85 + 0.1, y * 0.85 + 0.1]; }
  colourCache.set(id, c);
  return c;
}

function swatch(b) {
  const [r, g, bl] = rgbOf(b);
  return `rgb(${Math.round(r * 255)},${Math.round(g * 255)},${Math.round(bl * 255)})`;
}

function classifyEntries() {
  const n = A.h.entries.length;
  entCls = new Uint8Array(n);
  entCol = new Int32Array(n);
  for (let e = 0; e < n; e++) {
    const b = A.entBlock[e], blk = A.h.blocks[b], cat = blk.cat, tag = A.entTag[e];
    const firstBody = tag ? A.bodyById.get(A.h.tags[tag][0].body) : null;
    let cls = 0;
    if (cat === 'air') cls = 0;
    else if (view.cats[cat]) cls = cat === 'water' ? 2 : 1;
    else if (view.bodies && tag) cls = 2;
    entCls[e] = cls;
    if (cls === 2 && tag && !view.cats[cat]) entCol[e] = 20000 + firstBody.index;
    else if (view.colour === 'body') entCol[e] = firstBody && cat !== 'ore' ? 20000 + firstBody.index : (cat === 'ore' ? b : 30000 + b);
    else entCol[e] = b;
  }
}

// The horizon index holding y in box column col, or −1 above the first top.
function horizonAt(col, y) {
  if (!A.K) return -1;
  const base = col * A.K;
  if (A.tops[base] === -32768 || y > A.tops[base]) return -1;
  let hz = 0;
  while (hz + 1 < A.K && A.classes[base + hz + 1] !== 255 && y <= A.tops[base + hz + 1]) hz++;
  return hz;
}

// ---------------------------------------------------------------------------------------------
// Meshing.
// ---------------------------------------------------------------------------------------------
function cutAway(i, yy, zk) {
  if (yy + A.yLo + A.y0 > view.cutY) return true;
  if (view.secAxis === 'x') return view.flip ? i < view.secPos : i > view.secPos;
  if (view.secAxis === 'z') return view.flip ? zk < view.secPos : zk > view.secPos;
  return false;
}
function clsAt(i, yy, zk) {
  if (i < 0 || zk < 0 || yy < 0 || i >= A.W || zk >= A.D || yy >= A.Hc) return 0;
  if (cutAway(i, yy, zk)) return 0;
  return entCls[A.dense[(zk * A.W + i) * A.Hc + yy]];
}
function colAt(i, yy, zk) {
  const e = A.dense[(zk * A.W + i) * A.Hc + yy];
  if (view.colour === 'strata' && entCls[e] === 1) {
    const cat = A.h.blocks[A.entBlock[e]].cat;
    if (cat === 'rock' || cat === 'crust') {
      const hz = horizonAt(zk * A.W + i, yy + A.yLo + A.y0);
      if (hz >= 0) return 10000 + A.classes[(zk * A.W + i) * A.K + hz];
    }
  }
  return entCol[e];
}

function sectionKey(s) {
  const y0 = s.sy * S, y1 = Math.min(A.Hc, y0 + S) - 1;
  const cutRel = view.cutY - A.y0 - A.yLo;
  if (cutRel < y0) return 'empty';
  let k = `${version}|${Math.min(cutRel, y1 + 1)}`;
  if (view.secAxis !== 'none') {
    const a0 = (view.secAxis === 'x' ? s.sx : s.sz) * S;
    const a1 = a0 + S - 1;
    const p = view.secPos;
    if (!view.flip && p < a0) return 'empty';
    if (view.flip && p > a1) return 'empty';
    k += `|${view.secAxis}${view.flip ? 'f' : ''}${Math.max(a0 - 1, Math.min(p, a1 + 1))}`;
  }
  return k;
}

function schedule() {
  if (!A) return;
  const pending = [];
  for (const s of sections) {
    const key = sectionKey(s);
    if (key === s.key) continue;
    if (key === 'empty') { setMeshes(s, null, null); s.key = key; continue; }
    s.want = key;
    pending.push(s);
  }
  // Nearest to the camera first.
  const cam = group.worldToLocal(camera.position.clone());
  pending.sort((a, b) => dist2(a, cam) - dist2(b, cam));
  queue = pending;
}
function dist2(s, p) {
  const dx = s.sx * S + S / 2 - p.x, dy = s.sy * S + S / 2 - p.y, dz = s.sz * S + S / 2 - p.z;
  return dx * dx + dy * dy + dz * dz;
}

function setMeshes(s, o, t) {
  for (const m of [s.opaque, s.trans]) if (m) { group.remove(m); m.geometry.dispose(); }
  s.opaque = o; s.trans = t;
  if (o) group.add(o);
  if (t) group.add(t);
}

function buildSection(s) {
  if (sectionKey(s) !== s.want) return;         // superseded while queued
  const lo = [s.sx * S, s.sy * S, s.sz * S];
  const hi = [Math.min(A.W, lo[0] + S), Math.min(A.Hc, lo[1] + S), Math.min(A.D, lo[2] + S)];
  const out = [{ pos: [], col: [] }, { pos: [], col: [] }];   // opaque, see-through
  const mask = new Int32Array(S * S);
  const p = [0, 0, 0], q = [0, 0, 0];
  for (let d = 0; d < 3; d++) {
    const u = (d + 1) % 3, v = (d + 2) % 3;
    const nu = hi[u] - lo[u], nv = hi[v] - lo[v];
    for (let sign = 1; sign >= -1; sign -= 2) {
      const shade = SHADE[d * 2 + (sign > 0 ? 0 : 1)];
      for (let pass = 0; pass < 2; pass++) {
        for (let x = lo[d]; x < hi[d]; x++) {
          let any = false;
          for (let a = 0; a < nu; a++) {
            for (let b = 0; b < nv; b++) {
              p[d] = x; p[u] = lo[u] + a; p[v] = lo[v] + b;
              const c = clsAt(p[0], p[1], p[2]);
              let m = 0;
              if (c === pass + 1) {
                q[0] = p[0]; q[1] = p[1]; q[2] = p[2]; q[d] += sign;
                const cq = clsAt(q[0], q[1], q[2]);
                if (pass === 0 ? cq !== 1 : cq === 0) { m = colAt(p[0], p[1], p[2]) + 1; any = true; }
              }
              mask[a * S + b] = m;
            }
          }
          if (!any) continue;
          // Greedy: grow each face along v, then along u.
          for (let a = 0; a < nu; a++) {
            for (let b = 0; b < nv;) {
              const m = mask[a * S + b];
              if (!m) { b++; continue; }
              let w = 1;
              while (b + w < nv && mask[a * S + b + w] === m) w++;
              let hgt = 1;
              grow: while (a + hgt < nu) {
                for (let k = 0; k < w; k++) if (mask[(a + hgt) * S + b + k] !== m) break grow;
                hgt++;
              }
              for (let da = 0; da < hgt; da++) for (let k = 0; k < w; k++) mask[(a + da) * S + b + k] = 0;
              const o = [0, 0, 0];
              o[d] = x + (sign > 0 ? 1 : 0); o[u] = lo[u] + a; o[v] = lo[v] + b;
              const du = [0, 0, 0], dv = [0, 0, 0];
              du[u] = hgt; dv[v] = w;
              const v0 = o, v1 = [o[0] + du[0], o[1] + du[1], o[2] + du[2]];
              const v2 = [v1[0] + dv[0], v1[1] + dv[1], v1[2] + dv[2]], v3 = [o[0] + dv[0], o[1] + dv[1], o[2] + dv[2]];
              const tri = sign > 0 ? [v0, v1, v2, v0, v2, v3] : [v0, v2, v1, v0, v3, v2];
              const rgb = rgbOf(m - 1), f = shade;
              const T = out[pass];
              for (const vv of tri) { T.pos.push(vv[0], vv[1], vv[2]); T.col.push(rgb[0] * f, rgb[1] * f, rgb[2] * f); }
              b += w;
            }
          }
        }
      }
    }
  }
  const mk = (T, mat) => {
    if (!T.pos.length) return null;
    const g = new THREE.BufferGeometry();
    g.setAttribute('position', new THREE.Float32BufferAttribute(T.pos, 3));
    g.setAttribute('color', new THREE.Float32BufferAttribute(T.col, 3));
    g.computeBoundingSphere();
    const mesh = new THREE.Mesh(g, mat);
    if (mat === transMat) mesh.renderOrder = 2;
    return mesh;
  };
  setMeshes(s, mk(out[0], opaqueMat), mk(out[1], transMat));
  s.key = s.want;
}

// ---------------------------------------------------------------------------------------------
// Coordinates.
// ---------------------------------------------------------------------------------------------
function fold(x) {
  const c = A.h.circumference;
  return c > 0 ? (((x + c / 2) % c) + c) % c - c / 2 : x;
}
function worldOf(i, yy, zk) {
  const ux = A.h.box.x0 + i;
  return { ux, x: fold(ux), y: yy + A.yLo + A.y0, z: A.h.box.z0 + zk };
}
function colOf(x) {                                   // box column of a folded or unwrapped x
  const c = A.h.circumference;
  let d = x - A.h.box.x0;
  if (c > 0) d = ((d % c) + c) % c;
  return d >= 0 && d < A.W ? d : null;
}

// ---------------------------------------------------------------------------------------------
// Picking and the inspect panel.
// ---------------------------------------------------------------------------------------------
const ray = new THREE.Raycaster();
function pick(ev) {
  if (!A) return;
  const r = renderer.domElement.getBoundingClientRect();
  const ndc = new THREE.Vector2(((ev.clientX - r.left) / r.width) * 2 - 1, -((ev.clientY - r.top) / r.height) * 2 + 1);
  ray.setFromCamera(ndc, camera);
  const meshes = [];
  for (const s of sections) { if (s.opaque) meshes.push(s.opaque); if (s.trans) meshes.push(s.trans); }
  const hit = ray.intersectObjects(meshes, false)[0];
  if (!hit) { select(null); return; }
  const p = group.worldToLocal(hit.point.clone());
  const n = hit.face.normal;
  select([Math.floor(p.x - n.x * 0.5), Math.floor(p.y - n.y * 0.5), Math.floor(p.z - n.z * 0.5)]);
}

function select(v) {
  const panel = $('inspect');
  if (!v) { highlight.visible = false; panel.classList.remove('show'); return; }
  const [i, yy, zk] = v;
  if (i < 0 || zk < 0 || i >= A.W || zk >= A.D || yy < 0 || yy >= A.Hc) return;
  highlight.position.set(i + 0.5, yy + 0.5, zk + 0.5);
  highlight.visible = true;
  panel.innerHTML = describe(i, yy, zk);
  panel.classList.add('show');
  panel.querySelector('[data-close]').onclick = () => select(null);
  panel.querySelectorAll('[data-site]').forEach((el) => { el.onclick = () => cutThroughSite(+el.dataset.site); });
}

function eventLine(e) {
  if (!e) return '';
  const kind = short(e.kind).replace(/_/g, ' ');
  const ma = (t) => `${(-t / 1e6).toFixed(1)} Ma`;
  const when = e.t_start === e.t_end ? ma(e.t_start) : `${ma(e.t_start)} to ${ma(e.t_end)}`;
  const p = e.params || {};
  const extra = [p.limiting ? `limiting ${p.limiting}` : '', p.feature || ''].filter(Boolean).join(', ');
  return `<b>${esc(kind)}</b> ${esc(when)}${e.text ? ` — ${esc(e.text)}` : ''}${extra ? ` <span class="muted">(${esc(extra)})</span>` : ''}`;
}

function siteHtml(site) {
  if (!site) return '';
  const prod = (site.products || []).map((p) =>
    `${esc(short(p.material))} ${(p.grade * 100).toFixed(p.grade < 0.01 ? 3 : 2)} % · ${fmt(Math.round(p.tonnage_t))} t <span class="muted">${esc(p.role)}</span>`).join('<br>');
  const why = site.why || {};
  let chain = '';
  if (why.lines && why.lines.length) {
    chain = `<ol class="chain">${why.lines.map((l) => `<li>${esc(l.trim().replace(/^<-\s*/, '← '))}</li>`).join('')}</ol>`;
  } else if (why.forming_event) {
    chain = `<ol class="chain"><li>formed: ${eventLine(why.forming_event)}</li>${(why.chain || []).map((e) => `<li>← ${eventLine(e)}</li>`).join('')}</ol>`;
  } else if (why.error) chain = `<div class="warn small">${esc(why.error)}</div>`;
  const c = site.center || [];
  return `<div class="site"><div><b>Site ${site.id}</b>: ${esc(short(site.type))} <span class="muted">(${esc(site.kind)})</span></div>
    <div class="kv"><b>centre</b><span>${c.join(', ')} <a href="#" data-site="${site.id}" class="small">cut here</a></span>
    ${site.parent != null ? `<b>parent</b><span>site ${site.parent}</span>` : ''}
    ${prod ? `<b>products</b><span>${prod}</span>` : ''}
    ${site.coe_vein ? `<b>COE vein</b><span>${esc(short(site.coe_vein))}</span>` : ''}
    ${site.cdg_charge_mb ? `<b>CDG oil</b><span>${fmt(site.cdg_charge_mb)} mB</span>` : ''}</div>
    <div class="small muted" style="margin-top:4px">Why here${why.source ? ` (${why.source === 'probe' ? '/deeptime why' : 'world data'})` : ''}</div>${chain}</div>`;
}

function describe(i, yy, zk) {
  const e = A.dense[(zk * A.W + i) * A.Hc + yy];
  const b = A.h.blocks[A.entBlock[e]];
  const tag = A.entTag[e];
  const w = worldOf(i, yy, zk);
  const col = zk * A.W + i;
  const hz = horizonAt(col, w.y);
  const clsNames = A.h.strata?.classes || [];
  let rock = '';
  if (A.K) {
    if (hz < 0) rock = '<span class="muted">above the strata stack (nothing is painted here)</span>';
    else {
      const cls = clsNames[A.classes[col * A.K + hz]] || `class ${A.classes[col * A.K + hz]}`;
      const top = A.tops[col * A.K + hz];
      const next = hz + 1 < A.K && A.classes[col * A.K + hz + 1] !== 255 ? A.tops[col * A.K + hz + 1] + 1 : null;
      const isMember = (A.members[cls] || []).includes(b.id);
      let verdict;
      if (b.cat === 'rock' || b.cat === 'crust') {
        if (isMember) verdict = '<span class="ok">the strata pass painted it: a member of this class</span>';
        else if (b.cat === 'crust') verdict = '<span class="warn">unclassified crust: the class resolved to nothing here, or a later feature put stone back</span>';
        else verdict = 'not this horizon\'s rock (a deposit body\'s class, a surface rule or a feature)';
      } else verdict = `<span class="muted">${esc(b.cat)} over the horizon's rock</span>`;
      rock = `horizon ${hz + 1} of the column: <b>${esc(cls)}</b> <span class="muted">(y ${next ?? '…'} to ${top})</span><br><span class="small">${verdict}</span>`;
    }
  }
  const classes = (b.classes || []).length ? `<b>block is in</b><span>${b.classes.map(esc).join(', ')}</span>` : '';
  let bodies = '';
  if (tag) {
    for (const t of A.h.tags[tag]) {
      const body = A.bodyById.get(t.body);
      const zones = t.zones.map((zi) => body.zones[zi]).map((z) =>
        `${esc(z.name)}${z.material ? ` (${esc(short(z.material))}, p ${(+z.density).toFixed(2)})` : ''}${z.rock_class ? ` → ${esc(z.rock_class)}` : ''}`);
      bodies += `<div class="site"><div><b>Body ${body.id}</b>: ${esc(body.role)}, ${esc(body.shape.replace(/_/g, ' '))}${body.rock_class ? ` → ${esc(body.rock_class)}` : ''}</div>
        <div class="kv"><b>zones here</b><span>${zones.join('<br>') || '<span class="muted">none (outside every zone)</span>'}</span>
        <b>in the box</b><span>${fmt(body.voxels_in_box)} blocks, ${fmt(body.ore_blocks_in_box)} ore</span></div></div>`;
      if (body.site != null) bodies += siteHtml(A.siteById.get(body.site));
    }
  } else if (b.cat === 'ore') {
    const claimed = (A.h.claims?.materials || []).map(short);
    bodies = `<div class="site small">${claimed.includes(b.ore) ? 'Outside every deposit body: <b>background scatter</b> (Deep Time claims this material; its own scatter places these).'
      : 'Outside every deposit body; the material is <b>not claimed</b> by Deep Time, so the pack\'s own ore features place it.'}</div>`;
  }
  const seam = w.ux !== w.x ? ` <span class="muted">(unwrapped ${w.ux}, across the seam)</span>` : '';
  let stack = '';
  if (A.K && A.tops[col * A.K] !== -32768) {
    const rows = [];
    for (let k = 0; k < A.K && A.classes[col * A.K + k] !== 255; k++) {
      rows.push(`<tr class="${k === hz ? 'here' : ''}"><td>${k + 1}</td><td>${esc(clsNames[A.classes[col * A.K + k]] || '?')}</td><td>top ${A.tops[col * A.K + k]}</td></tr>`);
    }
    stack = `<h2>Strata stack of this column</h2><table class="col">${rows.join('')}</table>`;
  }
  return `<div class="row" style="justify-content:space-between"><h2 style="margin:0">Block</h2><button data-close aria-label="Close">✕</button></div>
    <div style="margin:6px 0"><span class="sw" style="background:${swatch(A.entBlock[e])}"></span> <b>${esc(b.id)}</b> <span class="muted">${esc(b.cat)}${b.ore ? `, ${esc(b.ore)} ore` : ''}</span></div>
    <div class="kv"><b>x</b><span>${w.x}${seam}</span><b>y</b><span>${w.y}</span><b>z</b><span>${w.z}</span>
    <b>chunk</b><span>${w.x >> 4}, ${w.z >> 4}</span>${classes}
    ${rock ? `<b>rock</b><span>${rock}</span>` : ''}</div>
    ${bodies ? `<h2>Deposit</h2>${bodies}` : ''}${stack}`;
}

// ---------------------------------------------------------------------------------------------
// Controls.
// ---------------------------------------------------------------------------------------------
function buildControls() {
  const counts = A.h.stats.categories || {};
  $('cats').innerHTML = CATS.map((c) =>
    `<label><span><input type="checkbox" data-cat="${c}" ${view.cats[c] ? 'checked' : ''}> ${CAT_LABEL[c]}</span><span class="n">${fmt(counts[c] || 0)}</span></label>`).join('');
  $('cats').querySelectorAll('input').forEach((el) => {
    el.onchange = () => { view.cats[el.dataset.cat] = el.checked ? 1 : 0; markPreset(null); changed(); };
  });
  const yMin = A.y0 + A.yLo, yMax = A.y0 + A.yLo + A.Hc - 1;
  const cy = $('cuty');
  cy.min = yMin; cy.max = yMax; cy.value = view.cutY;
  $('cuty-v').textContent = view.cutY;
  cy.oninput = () => { view.cutY = +cy.value; $('cuty-v').textContent = cy.value; schedule(); };
  const sp = $('secpos');
  const secUpdate = () => {
    const ax = view.secAxis;
    sp.disabled = ax === 'none';
    if (ax === 'none') { $('sec-v').textContent = ''; schedule(); return; }
    sp.min = 0; sp.max = (ax === 'x' ? A.W : A.D) - 1;
    const w = ax === 'x' ? worldOf(view.secPos, 0, 0) : null;
    $('sec-v').innerHTML = ax === 'x' ? `x = ${w.x}${w.ux !== w.x ? ` (unwrapped ${w.ux})` : ''}` : `z = ${A.h.box.z0 + view.secPos}`;
    schedule();
  };
  $('secaxis').value = view.secAxis;
  $('secflip').checked = view.flip;
  sp.value = view.secPos;
  $('secaxis').onchange = () => {
    view.secAxis = $('secaxis').value;
    view.secPos = Math.floor((view.secAxis === 'x' ? A.W : A.D) / 2);
    sp.value = view.secPos; secUpdate();
  };
  $('secflip').onchange = () => { view.flip = $('secflip').checked; secUpdate(); };
  sp.oninput = () => { view.secPos = +sp.value; secUpdate(); };
  secUpdate.call();
  $('bodies').checked = view.bodies;
  $('bodies').onchange = () => { view.bodies = $('bodies').checked; changed(); };
  $('colour').value = view.colour;
  $('colour').onchange = () => { view.colour = $('colour').value; changed(); };
  $('vivid').checked = view.vividOres;
  $('vivid').onchange = () => { view.vividOres = $('vivid').checked; colourCache.clear(); changed(); };
  document.querySelectorAll('[data-preset]').forEach((el) => { el.onclick = () => applyPreset(el.dataset.preset); });
  const sites = A.h.sites || [];
  $('sites').innerHTML = sites.length ? sites.map((s) =>
    `<li data-site="${s.id}"><b>${s.id}</b> ${esc(short(s.type))} <span class="muted">${esc(s.kind)} · ${(s.products || []).map((p) => short(p.material)).join(', ') || 'no products'}</span></li>`).join('')
    : '<li class="muted">none</li>';
  $('sites').querySelectorAll('[data-site]').forEach((el) => { el.onclick = () => cutThroughSite(+el.dataset.site); });
  const st = A.h.stats, meta = A.h.meta || {};
  const world = meta.world ? Object.entries(meta.world).filter(([k, v]) => v && !['datapacks', 'datapacks_used'].includes(k))
    .map(([k, v]) => `${k} ${esc(String(v).split('/').pop())}`).join(', ') : '';
  const strata = st.strata ? `${(100 * st.strata.painted / st.strata.checked).toFixed(2)} % of rock below the first horizon is the horizon's (or its body's) class; ${fmt(st.strata.crust)} crust, ${fmt(st.strata.other)} other` : 'no strata layer';
  const ores = Object.entries(st.ore || {}).map(([m, o]) => `${m} ${fmt(o.in_bodies)} in bodies / ${fmt(o.outside_claimed)} scatter / ${fmt(o.outside_unclaimed)} unclaimed`).join('<br>');
  const checks = (st.probe_site_checks || []);
  const agree = checks.length ? `${checks.filter((c) => c.bodies_agree).length} of ${checks.length} <code>probe site</code> answers match the body tags` +
    (checks.some((c) => c.block_agrees != null) ? `; ${checks.filter((c) => c.block_agrees).length} of ${checks.filter((c) => c.block_agrees != null).length} blocks match` : '') : '';
  $('about').innerHTML = `Box x ${A.h.box.x0}..${A.h.box.x1 - 1}, z ${A.h.box.z0}..${A.h.box.z1 - 1}, y ${A.h.y0}..${A.h.y1} (C = ${fmt(A.h.circumference)})` +
    `${A.h.seam?.straddles ? `, <b>across the seam</b> at column ${A.h.seam.column}` : ''}.<br>` +
    `${meta.profile ? `Profile <b>${esc(meta.profile)}</b>${world ? `, ${world}` : ''}. ` : ''}${meta.install?.pack_ref ? `MIC pack ${esc(meta.install.pack_ref)}. ` : ''}` +
    `${fmt(st.non_air)} blocks, ${A.h.blocks.length} block types, ${(A.h.bodies || []).length} deposit bodies.<br>Strata: ${strata}.<br>` +
    `${ores ? `Ore: ${ores}<br>` : ''}${agree}${st.missing_chunks?.length ? `<br><span class="warn">${st.missing_chunks.length} chunks missing</span>` : ''}`;
}

function markPreset(name) {
  document.querySelectorAll('[data-preset]').forEach((el) => el.classList.toggle('on', el.dataset.preset === name));
}
function applyPreset(name) {
  const p = PRESETS[name];
  if (!p) return;
  for (const c of CATS) view.cats[c] = p[c];
  view.bodies = !!p.bodies;
  document.querySelectorAll('#cats input').forEach((el) => { el.checked = !!view.cats[el.dataset.cat]; });
  $('bodies').checked = view.bodies;
  markPreset(name);
  changed();
}
function changed() { version++; classifyEntries(); schedule(); }

function cutThroughSite(id) {
  const s = A.siteById.get(id);
  if (!s) return;
  // The body to show: the site's ore body with the most ore in the box, else any of its bodies.
  const bodies = (A.h.bodies || []).filter((x) => x.site === id);
  const b = bodies.filter((x) => x.role === 'ore').sort((p, q) => q.ore_blocks_in_box - p.ore_blocks_in_box)[0] || bodies[0];
  const i = colOf(b ? Math.floor(b.center[0]) : s.center[0]);
  const zk = (b ? Math.floor(b.center[2]) : s.center[2]) - A.h.box.z0;
  const cy = b ? Math.round((b.aabb[0][1] + b.aabb[1][1]) / 2) : s.center[1];
  // A notch: the top cut at the body's middle and a section along x through its centre (west half
  // kept), so the ore shows in its host rock on both faces, with the strata around it.
  view.cutY = Math.max(+$('cuty').min, Math.min(+$('cuty').max, cy));
  $('cuty').value = view.cutY; $('cuty-v').textContent = view.cutY;
  if (i != null) {
    view.secAxis = 'x'; view.secPos = i; view.flip = false;
    const sp = $('secpos');
    sp.disabled = false; sp.min = 0; sp.max = A.W - 1; sp.value = i;
    $('secaxis').value = 'x'; $('secflip').checked = false;
    const w = worldOf(i, 0, 0);
    $('sec-v').innerHTML = `x = ${w.x}${w.ux !== w.x ? ` (unwrapped ${w.ux})` : ''}`;
  }
  schedule();
  if (i != null && zk >= 0 && zk < A.D) {
    const yy = Math.max(0, Math.min(A.Hc - 1, view.cutY - A.y0 - A.yLo));
    const zoom = Number((params.get('cam') || '').split(',')[2]) || 1;
    const d = Math.max(A.W, A.D) * 0.55 * zoom;
    controls.target.set(i, yy, zk);
    camera.position.set(i + d * 0.62, yy + d * 0.7, zk + d * 0.45);
    group.localToWorld(controls.target);
    group.localToWorld(camera.position);
  }
}

// ---------------------------------------------------------------------------------------------
// Loading.
// ---------------------------------------------------------------------------------------------
async function load(src, label) {
  status('Loading…');
  $('drop').classList.remove('show');
  let buf = src;
  let bytes = 0;
  try {
    if (typeof src === 'string') {
      const r = await fetch(src);
      if (!r.ok) throw new Error(`${src}: HTTP ${r.status}`);
      buf = await r.arrayBuffer();
    }
    bytes = buf.byteLength;
    const t0 = performance.now();
    A = decode(await inflate(buf));
    A.decodeMs = performance.now() - t0;
  } catch (err) {
    status(`Could not load: ${err.message}`);
    $('drop').classList.add('show');
    return;
  }
  for (const s of sections) setMeshes(s, null, null);
  sections = [];
  colourCache.clear();
  for (let sz = 0; sz < Math.ceil(A.D / S); sz++)
    for (let sy = 0; sy < Math.ceil(A.Hc / S); sy++)
      for (let sx = 0; sx < Math.ceil(A.W / S); sx++) sections.push({ sx, sy, sz, key: null });
  group.position.set(-A.W / 2, -A.Hc / 2, -A.D / 2);
  // State from the URL (first load only) or defaults.
  const yTop = A.y0 + A.yLo + A.Hc - 1;
  view.cutY = yTop; view.secAxis = 'none'; view.secPos = 0; view.flip = false;
  const pr = params.get('preset');
  if (pr && PRESETS[pr]) { Object.assign(view.cats, PRESETS[pr]); view.bodies = !!PRESETS[pr].bodies; markPreset(pr); }
  if (params.get('bodies')) view.bodies = params.get('bodies') === '1';
  if (params.get('colour')) view.colour = params.get('colour');
  if (params.get('cuty')) view.cutY = +params.get('cuty');
  if (params.get('sec')) {
    view.secAxis = params.get('sec');
    const p = +params.get('secpos');
    view.secPos = view.secAxis === 'x' ? (colOf(p) ?? Math.floor(A.W / 2)) : Math.max(0, Math.min(A.D - 1, p - A.h.box.z0));
    view.flip = params.get('flip') === '1';
  }
  classifyEntries();
  buildControls();
  // Camera: from the south-east, above.
  // The distance frames the whole box (its diagonal) at the 45° field of view.
  const [az, el, df] = (params.get('cam') || '35,28,1').split(',').map(Number);
  const R = Math.hypot(A.W, A.D, A.Hc) * 1.15 * df;
  const surf = surfaceGuess();
  controls.target.set(0, (surf - A.Hc / 2) * 0.35, 0);
  camera.position.set(R * Math.cos(el * Math.PI / 180) * Math.sin(az * Math.PI / 180) + 0,
    controls.target.y + R * Math.sin(el * Math.PI / 180), R * Math.cos(el * Math.PI / 180) * Math.cos(az * Math.PI / 180));
  controls.update();
  version++;
  schedule();
  if (params.get('site')) cutThroughSite(+params.get('site'));
  if (params.get('pick')) {
    const [x, y, z] = params.get('pick').split(',').map(Number);
    const i = colOf(x);
    if (i != null) setTimeout(() => select([i, y - A.y0 - A.yLo, z - A.h.box.z0]), 0);
  }
  const st = A.h.stats;
  A.statusLine = `${label || ''}${label ? ' · ' : ''}${(bytes / 1e6).toFixed(2)} MB · ${fmt(st.non_air)} blocks · ${A.W}×${A.D}×${A.H} · click a block`;
  $('subtitle').textContent = `x ${A.h.box.x0}..${A.h.box.x1 - 1}, z ${A.h.box.z0}..${A.h.box.z1 - 1}` +
    `${A.h.meta?.profile ? ` · ${A.h.meta.profile}` : ''}${A.h.seam?.straddles ? ' · across the seam' : ''}`;
  drawSeam();
  window.__area3dReady = () => queue.length === 0;
}

function surfaceGuess() {
  // The mean top non-air row of a sparse grid of columns (dense-relative).
  let sum = 0, n = 0;
  for (let zk = 0; zk < A.D; zk += 16) for (let i = 0; i < A.W; i += 16) {
    const base = (zk * A.W + i) * A.Hc;
    for (let y = A.Hc - 1; y >= 0; y--) if (A.dense[base + y] && A.h.blocks[A.entBlock[A.dense[base + y]]].cat !== 'air') { sum += y; n++; break; }
  }
  return n ? sum / n : A.Hc / 2;
}

function drawSeam() {
  if (seamLine) { group.remove(seamLine); seamLine = null; }
  if (!A.h.seam?.straddles) return;
  const x = A.h.seam.column;
  const g = new THREE.BufferGeometry().setFromPoints([
    new THREE.Vector3(x, A.Hc + 4, 0), new THREE.Vector3(x, A.Hc + 4, A.D),
    new THREE.Vector3(x, A.Hc + 4, A.D), new THREE.Vector3(x, 0, A.D),
    new THREE.Vector3(x, A.Hc + 4, 0), new THREE.Vector3(x, 0, 0)]);
  seamLine = new THREE.LineSegments(g, new THREE.LineBasicMaterial({ color: 0xff3b30 }));
  group.add(seamLine);
}

async function start() {
  initScene();
  const f = $('file');
  f.onchange = async () => { if (f.files[0]) load(await f.files[0].arrayBuffer(), f.files[0].name); };
  addEventListener('dragover', (e) => e.preventDefault());
  addEventListener('drop', async (e) => {
    e.preventDefault();
    const file = e.dataTransfer.files[0];
    if (file) load(await file.arrayBuffer(), file.name);
  });
  $('toggle-controls').onclick = () => $('controls').classList.toggle('open');
  window.Area3D = { load, select, view, cutThroughSite };
  const src = params.get('src');
  if (src) return load(src, src.split('/').pop());
  try {
    const r = await fetch('exports.json');
    if (r.ok) {
      const list = await r.json();
      const sel = $('exports');
      sel.innerHTML = list.map((x, k) => `<option value="${k}">${esc(x.label || x.src)}</option>`).join('');
      sel.hidden = list.length < 2;
      sel.onchange = () => load(list[+sel.value].src, list[+sel.value].label);
      const k = Math.max(0, list.findIndex((x) => x.label === params.get('export')));
      sel.value = k;
      if (list.length) return load(list[k].src, list[k].label);
    }
  } catch { /* no list */ }
  status('Open an export to start');
  $('drop').classList.add('show');
}
start();
