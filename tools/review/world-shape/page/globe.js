// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time world-shape page: the 3D globe. Each finalist's net (the Guyou squares or the cube's cross) is
// folded about its internal edges into a pillowcase or a cube, then inflated onto the planet along the
// projection itself (each mesh vertex carries its planet point from compute.py). Seams, corners, walks and
// the corner close-ups are drawn on the planet once it is round. three.js r128 (global THREE), local copy.
'use strict';
(function () {
const D = window.WS_DATA, W = window.WS;
const canvas = document.getElementById('globe');
if (!window.THREE || !canvas || !W) return;
const T = THREE;
let renderer;
try { renderer = new T.WebGLRenderer({ canvas, antialias: true, alpha: true }); } catch (e) {
  canvas.insertAdjacentHTML('afterend', '<p class="note">WebGL is not available in this browser, so the 3D view is off.</p>');
  return;
}
renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
const scene = new T.Scene();
const camera = new T.PerspectiveCamera(35, 4 / 3, 0.05, 100);
scene.add(new T.AmbientLight(0xffffff, 0.62));
const sun = new T.DirectionalLight(0xffffff, 0.55);
sun.position.set(-2, 3, 4);
scene.add(sun);

// display frame: planet x (lon 0) -> +z (towards the camera), y (lon 90 E) -> +x, z (north) -> +y
const Dm = new T.Matrix4().set(0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1);
const world = new T.Group();
scene.add(world);
const foldGroup = new T.Group();
const planetGroup = new T.Group();
planetGroup.matrixAutoUpdate = false;
planetGroup.matrix.copy(Dm);
world.add(foldGroup, planetGroup);

const ids = [W.G, W.K];
let cur = null;
let t = 2.0;
const opts = { seams: true, corners: true, walks: true, close: true };
const loader = new T.TextureLoader();
const texCache = {};
function tex(url) {
  if (!texCache[url]) {
    const tx = loader.load(url, () => render());
    tx.anisotropy = 4;
    texCache[url] = tx;
  }
  return texCache[url];
}
function decode(b64) {
  const bin = atob(b64), n = bin.length >> 1, a = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    let v = bin.charCodeAt(2 * i) | (bin.charCodeAt(2 * i + 1) << 8);
    if (v > 32767) v -= 65536;
    a[i] = v / 32767;
  }
  return a;
}
function ll2v(ll, r) {
  const la = ll[0] * Math.PI / 180, lo = ll[1] * Math.PI / 180;
  return new T.Vector3(r * Math.cos(la) * Math.cos(lo), r * Math.cos(la) * Math.sin(lo), r * Math.sin(la));
}
const ease = (x) => x < 0 ? 0 : x > 1 ? 1 : x * x * (3 - 2 * x);
function tube(pts, r, color) {
  if (pts.length < 2) return null;
  const geo = new T.TubeGeometry(new T.CatmullRomCurve3(pts), Math.min(400, pts.length * 2), r, 5, false);
  return new T.Mesh(geo, new T.MeshBasicMaterial({ color }));
}
function splitTrack(track) {
  // break a lat/lon track where it jumps (a seam crossing is one point on the planet, so jumps are rare)
  const out = [[]];
  for (let i = 0; i < track.length; i++) {
    if (i && Math.abs(track[i][0] - track[i - 1][0]) + Math.abs(((track[i][1] - track[i - 1][1] + 540) % 360) - 180) > 8) out.push([]);
    out[out.length - 1].push(track[i]);
  }
  return out;
}

function build(id) {
  const L = D.layouts[id], M = L.mesh;
  const st = { id, L, M, tiles: [], radius: M.radius };
  foldGroup.clear();
  planetGroup.clear();
  const [bx0, by0, bx1, by1] = L.bbox.map((v) => v / L.unit_blocks);
  const map = tex(`${id}-map.webp`);
  for (const tl of M.tiles) {
    const nx = tl.nx, ny = tl.ny, nv = (nx + 1) * (ny + 1);
    const sph = decode(tl.xyz);
    const flat = new Float32Array(nv * 2), uv = new Float32Array(nv * 2);
    for (let j = 0; j <= ny; j++) for (let i = 0; i <= nx; i++) {
      const k = j * (nx + 1) + i, X = tl.x0 + tl.w * i / nx, Y = tl.y0 + tl.h * j / ny;
      flat[2 * k] = X; flat[2 * k + 1] = Y;
      uv[2 * k] = (X - bx0) / (bx1 - bx0); uv[2 * k + 1] = (Y - by0) / (by1 - by0);
    }
    const idx = [];
    for (let j = 0; j < ny; j++) for (let i = 0; i < nx; i++) {
      const a = j * (nx + 1) + i, b = a + 1, c = a + nx + 1, d = c + 1;
      idx.push(a, b, d, a, d, c);
    }
    const g = new T.BufferGeometry();
    g.setAttribute('position', new T.BufferAttribute(new Float32Array(nv * 3), 3));
    g.setAttribute('uv', new T.BufferAttribute(uv, 2));
    g.setIndex(idx);
    const mesh = new T.Mesh(g, new T.MeshLambertMaterial({ map, side: T.DoubleSide }));
    foldGroup.add(mesh);
    st.tiles.push({ tl, nx, ny, nv, sph, flat, g, mesh, centre: [tl.x0 + tl.w / 2, tl.y0 + tl.h / 2] });
  }
  st.parent = new Array(st.tiles.length).fill(-1);
  st.hinge = new Array(st.tiles.length).fill(null);
  const seen = new Set([0]), q = [0], order = [0];
  while (q.length) {
    const a = q.shift();
    for (const h of M.hinges) {
      let b = -1;
      if (h.parent === a && !seen.has(h.child)) b = h.child; else if (h.child === a && !seen.has(h.parent)) b = h.parent;
      if (b < 0) continue;
      seen.add(b); q.push(b); order.push(b);
      st.parent[b] = a; st.hinge[b] = h;
    }
  }
  st.order = order;
  st.foldAngle = M.fold === 'cube' ? Math.PI / 2 : Math.PI;
  st.Rk = alignRotation(st, foldedPositions(st, 1));
  const r = st.radius, o = L.overlay;
  const KC = { slide: W.css('--slide'), quarter: W.css('--quarter'), half: W.css('--half') };
  st.seamObjs = [];
  const add = (m) => { if (m) { planetGroup.add(m); st.seamObjs.push(m); } };
  if (opts.seams) {
    for (const s of o.seams) add(tube(s.ll.map((ll) => ll2v(ll, r * 1.004)), r * 0.008, KC[s.kind]));
    for (const e of o.edges) {
      const ln = new T.Line(new T.BufferGeometry().setFromPoints(e.map((ll) => ll2v(ll, r * 1.003))), new T.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.55 }));
      add(ln);
    }
  }
  if (opts.corners) {
    for (const c of o.corners) {
      const m = new T.Mesh(new T.SphereGeometry(r * 0.02, 16, 12), new T.MeshBasicMaterial({ color: W.css('--corner') }));
      m.position.copy(ll2v(c.ll, r * 1.006));
      add(m);
    }
  }
  if (opts.walks) {
    const cols = { equator: W.css('--walk-eq'), pole: W.css('--walk-pole'), diag: W.css('--walk-diag') };
    for (const k of ['equator', 'pole', 'diag']) {
      const w = L.walks && L.walks[k];
      if (!w) continue;
      for (const part of splitTrack(w.track)) add(tube(part.map((ll) => ll2v(ll, r * 1.007)), r * 0.005, cols[k]));
    }
  }
  if (opts.close) {
    const C = D.corners[id];
    const cols = { globe_ring: W.css('--walk-ring'), globe_A: W.css('--walk-a'), globe_B: W.css('--walk-b') };
    for (const k of ['globe_ring', 'globe_A', 'globe_B']) {
      for (const part of splitTrack(C[k])) add(tube(part.map((ll) => ll2v(ll, r * 1.008)), r * 0.004, cols[k]));
    }
  }
  cur = st;
  update();
}

function tileMatrices(st, a) {
  const mats = new Array(st.tiles.length);
  for (const k of st.order) {
    if (st.parent[k] < 0) { mats[k] = new T.Matrix4(); continue; }
    const h = st.hinge[k], p0 = new T.Vector3(h.p0[0], h.p0[1], 0), p1 = new T.Vector3(h.p1[0], h.p1[1], 0);
    const ax = p1.clone().sub(p0).normalize();
    const c = st.tiles[k].centre, w = new T.Vector3(c[0], c[1], 0).sub(p0);
    w.sub(ax.clone().multiplyScalar(w.dot(ax)));
    const sgn = ax.clone().cross(w).z > 0 ? -1 : 1;
    const R = new T.Matrix4().makeTranslation(p0.x, p0.y, 0)
      .multiply(new T.Matrix4().makeRotationAxis(ax, sgn * a))
      .multiply(new T.Matrix4().makeTranslation(-p0.x, -p0.y, 0));
    mats[k] = mats[st.parent[k]].clone().multiply(R);
  }
  return mats;
}
function foldedPositions(st, f) {
  const out = [];
  const mats = tileMatrices(st, st.foldAngle * ease(f));
  const cen = new T.Vector3();
  st.tiles.forEach((tt, i) => cen.add(new T.Vector3(tt.centre[0], tt.centre[1], 0).applyMatrix4(mats[i])));
  cen.multiplyScalar(1 / st.tiles.length);
  const v = new T.Vector3();
  st.tiles.forEach((tt, i) => {
    const a = new Float32Array(tt.nv * 3), m = mats[i];
    const z0 = st.M.fold === 'pillow' && i > 0 ? 0.012 : 0;
    for (let k = 0; k < tt.nv; k++) {
      v.set(tt.flat[2 * k], tt.flat[2 * k + 1], z0).applyMatrix4(m).sub(cen);
      a[3 * k] = v.x; a[3 * k + 1] = v.y; a[3 * k + 2] = v.z;
    }
    out.push(a);
  });
  return out;
}
function alignRotation(st, fin) {
  const t0 = st.tiles[0], nx = t0.nx, ny = t0.ny;
  const kc = Math.round(ny / 2) * (nx + 1) + Math.round(nx / 2), kr = Math.round(ny / 2) * (nx + 1) + Math.round(3 * nx / 4);
  const get = (arr, k) => new T.Vector3(arr[3 * k], arr[3 * k + 1], arr[3 * k + 2]);
  const frame = (a, b) => {
    const e1 = a.clone().normalize(), e2 = b.clone().sub(e1.clone().multiplyScalar(b.dot(e1))).normalize(), e3 = e1.clone().cross(e2);
    return new T.Matrix4().makeBasis(e1, e2, e3);
  };
  const Ff = frame(get(fin[0], kc), get(fin[0], kr));
  const Fs = frame(get(t0.sph, kc), get(t0.sph, kr));
  return Fs.multiply(Ff.clone().transpose());
}
const qI = new T.Quaternion();
function update() {
  const st = cur;
  if (!st) return;
  const f = Math.min(t, 1), inflate = ease(Math.max(0, t - 1));
  const fp = foldedPositions(st, f);
  const qT = new T.Quaternion().setFromRotationMatrix(Dm.clone().multiply(st.Rk));
  const q = qI.clone().slerp(qT, ease(f));
  const v = new T.Vector3(), s = new T.Vector3();
  st.tiles.forEach((tt, i) => {
    const pos = tt.g.attributes.position.array, a = fp[i];
    for (let k = 0; k < tt.nv; k++) {
      v.set(a[3 * k], a[3 * k + 1], a[3 * k + 2]).applyQuaternion(q);
      if (inflate > 0) {
        s.set(tt.sph[3 * k], tt.sph[3 * k + 1], tt.sph[3 * k + 2]).multiplyScalar(st.radius).applyMatrix4(Dm);
        v.lerp(s, inflate);
      }
      pos[3 * k] = v.x; pos[3 * k + 1] = v.y; pos[3 * k + 2] = v.z;
    }
    tt.g.attributes.position.needsUpdate = true;
    tt.g.computeVertexNormals();
    tt.g.computeBoundingSphere();
  });
  for (const o of st.seamObjs) o.visible = t >= 1.97;
  render();
}

let az = 0.35, el = 0.25, dist = 4.2;
function placeCamera() {
  camera.position.set(dist * Math.sin(az) * Math.cos(el), dist * Math.sin(el), dist * Math.cos(az) * Math.cos(el));
  camera.lookAt(0, 0, 0);
}
function resize() {
  const w = canvas.clientWidth, h = canvas.clientHeight;
  if (!w || !h) return;
  renderer.setSize(w, h, false);
  camera.aspect = w / h;
  camera.updateProjectionMatrix();
  render();
}
let raf = 0;
function render() {
  if (raf) return;
  raf = requestAnimationFrame(() => { raf = 0; placeCamera(); renderer.render(scene, camera); });
}
const ptrs = new Map();
let pinch0 = 0;
canvas.addEventListener('pointerdown', (e) => { canvas.setPointerCapture(e.pointerId); ptrs.set(e.pointerId, [e.clientX, e.clientY]); canvas.style.cursor = 'grabbing'; });
canvas.addEventListener('pointermove', (e) => {
  if (!ptrs.has(e.pointerId)) return;
  const [x0, y0] = ptrs.get(e.pointerId);
  if (ptrs.size === 1) {
    az -= (e.clientX - x0) * 0.008;
    el = Math.max(-1.45, Math.min(1.45, el + (e.clientY - y0) * 0.008));
  }
  ptrs.set(e.pointerId, [e.clientX, e.clientY]);
  if (ptrs.size === 2) {
    const [a, b] = [...ptrs.values()];
    const d = Math.hypot(a[0] - b[0], a[1] - b[1]);
    if (pinch0) dist = Math.max(1.4, Math.min(9, dist * pinch0 / d));
    pinch0 = d;
  }
  render();
});
const up = (e) => { ptrs.delete(e.pointerId); pinch0 = 0; canvas.style.cursor = 'grab'; };
canvas.addEventListener('pointerup', up);
canvas.addEventListener('pointercancel', up);
canvas.addEventListener('wheel', (e) => { e.preventDefault(); dist = Math.max(1.4, Math.min(9, dist * Math.exp(e.deltaY * 0.001))); render(); }, { passive: false });
window.addEventListener('resize', resize);

const slider = document.getElementById('g-t'), play = document.getElementById('g-play');
slider.addEventListener('input', () => { t = slider.value / 100; update(); });
let anim = null;
play.addEventListener('click', () => {
  if (anim) { cancelAnimationFrame(anim); anim = null; play.textContent = 'Play'; return; }
  const reduce = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const from = t >= 1.99 ? 0 : t, start = performance.now(), dur = reduce ? 1 : 7000 * (2 - from) / 2;
  if (from === 0) { t = 0; update(); }
  play.textContent = 'Pause';
  const step = (now) => {
    t = Math.min(2, from + (2 - from) * (now - start) / dur);
    slider.value = Math.round(t * 100);
    update();
    if (t < 2) anim = requestAnimationFrame(step); else { anim = null; play.textContent = 'Play'; }
  };
  anim = requestAnimationFrame(step);
});
const caps = {
  [W.G]: 'Petroff–Guyou: the two hemisphere squares fold together into a pillowcase, glued round all four edges, then inflate. The pillowcase\'s four corners are the four corners of the world.',
  [W.K]: 'Moser cube: the four band faces wrap into a ring, the polar faces close the lid and the floor, then the cube inflates along the Moser map. Seams (coloured) and the eight corners appear at the end.',
};
const pick = document.getElementById('g-picker');
const chips = {};
function choose(id) {
  for (const [k, b] of Object.entries(chips)) b.setAttribute('aria-pressed', String(k === id));
  build(id);
  document.getElementById('g-caption').textContent = caps[id];
}
for (const id of ids) {
  const b = document.createElement('button');
  b.type = 'button'; b.className = 'chip'; b.textContent = W.NAME[id];
  b.addEventListener('click', () => choose(id));
  chips[id] = b;
  pick.appendChild(b);
}
const close = document.getElementById('g-close');
for (const id of ids) {
  const b = document.createElement('button');
  b.type = 'button'; b.className = 'act'; b.textContent = `${W.NAME[id]}: corner close-up`;
  b.addEventListener('click', () => {
    opts.close = true;
    document.getElementById('g-ov-close').checked = true;
    t = 2; slider.value = 200;
    choose(id);
    const ll = D.corners[id].globe_ll;
    az = ll[1] * Math.PI / 180; el = ll[0] * Math.PI / 180; dist = 1.9;
    render();
  });
  close.appendChild(b);
}
const ch = document.getElementById('g-checks');
for (const [k, lab] of [['seams', 'Seams'], ['corners', 'Corners'], ['walks', 'Walks'], ['close', 'Corner walkers and ring']]) {
  const l = document.createElement('label');
  l.innerHTML = `<input type="checkbox" id="g-ov-${k}" ${opts[k] ? 'checked' : ''}> ${lab}`;
  l.querySelector('input').addEventListener('change', (e) => { opts[k] = e.target.checked; build(cur.id); });
  ch.appendChild(l);
}
const hg = location.hash.match(/gt-(\d+)(?:-([a-z-]+?))?(?:,|$)/);
const startId = hg && hg[2] && ids.includes(hg[2]) ? hg[2] : W.K;
if (hg) { t = Math.min(2, +hg[1] / 100); slider.value = Math.round(t * 100); }
choose(startId);
resize();
})();
