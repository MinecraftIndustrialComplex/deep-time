// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time world-shape page: the 3D globe. A world's net (cube or pillowcase tiles, or Lambert's strip) is
// folded about its internal edges into a cube, a pillowcase or a cylinder, then inflated onto the planet
// along the projection itself (each mesh vertex carries its planet point from compute.py). Seams, corners
// and walks are drawn on the planet once it is round. three.js r128 (global THREE), local copy.
'use strict';
(function () {
const D = window.WS_DATA, W = window.WS;
const canvas = document.getElementById('globe');
if (!window.THREE || !canvas) return;
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

const ids = Object.keys(D.layouts).filter((id) => D.layouts[id].mesh && D.layouts[id].mesh.fold);
const extra = Object.keys(D.layouts).filter((id) => !ids.includes(id));
let cur = null;
let t = 2.0;
const opts = { seams: true, corners: true, walks: true, corner: false, planetTex: false };
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

// ------------------------------------------------------------------------------------------
function build(id) {
  const L = D.layouts[id], M = L.mesh && L.mesh.fold ? L.mesh : null;
  const st = { id, L, M, tiles: [], radius: M ? M.radius : 0.8 };
  foldGroup.clear();
  planetGroup.clear();
  if (M) {
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
    // hinge tree from tile 0
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
    // the rotation that takes the final folded shape onto the planet
    const fin = foldedPositions(st, 1);
    st.Rk = alignRotation(st, fin);
  }
  // planet overlays
  const r = st.radius;
  const o = L.overlay;
  if (opts.planetTex || !M) {
    const sg = new T.SphereGeometry(r * 0.999, 128, 64);
    // three's sphere: re-map to the planet frame (z north, x at lon 0)
    const pos = sg.attributes.position, uvs = sg.attributes.uv;
    for (let i = 0; i < pos.count; i++) {
      const u = uvs.getX(i), v = uvs.getY(i);
      const lon = (u - 0.5) * 2 * Math.PI, lat = (v - 0.5) * Math.PI;
      pos.setXYZ(i, r * 0.999 * Math.cos(lat) * Math.cos(lon), r * 0.999 * Math.cos(lat) * Math.sin(lon), r * 0.999 * Math.sin(lat));
    }
    sg.computeVertexNormals();
    st.globe = new T.Mesh(sg, new T.MeshLambertMaterial({ map: tex('globe.webp'), side: T.FrontSide }));
    planetGroup.add(st.globe);
  }
  const KC = { slide: W.css('--slide'), quarter: W.css('--quarter'), half: W.css('--half') };
  st.seamObjs = [];
  if (opts.seams) {
    for (const s of o.seams) {
      const pts = s.ll.map((ll) => ll2v(ll, r * 1.004));
      const geo = new T.TubeGeometry(new T.CatmullRomCurve3(pts), 64, r * 0.008, 6, false);
      const m = new T.Mesh(geo, new T.MeshBasicMaterial({ color: KC[s.kind] }));
      planetGroup.add(m); st.seamObjs.push(m);
    }
    for (const e of o.edges) {
      const pts = e.map((ll) => ll2v(ll, r * 1.003));
      const ln = new T.Line(new T.BufferGeometry().setFromPoints(pts), new T.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.55 }));
      planetGroup.add(ln); st.seamObjs.push(ln);
    }
  }
  if (opts.corners) {
    for (const c of o.corners) {
      const m = new T.Mesh(new T.SphereGeometry(r * 0.022, 16, 12), new T.MeshBasicMaterial({ color: W.css('--corner') }));
      m.position.copy(ll2v(c.ll, r * 1.006));
      planetGroup.add(m); st.seamObjs.push(m);
    }
  }
  const walkCols = { equator: W.css('--walk-eq'), pole: W.css('--walk-pole'), poleB: W.css('--walk-pole'), cornerA: W.css('--walk-a'), cornerB: W.css('--walk-b') };
  const which = [].concat(opts.walks ? ['equator', 'pole'] : [], opts.corner ? ['cornerA', 'cornerB'] : []);
  for (const k of which) {
    const w = L.walks && L.walks[k];
    if (!w || w.track.length < 2) continue;
    const pts = w.track.map((ll) => ll2v(ll, r * 1.007));
    const geo = new T.TubeGeometry(new T.CatmullRomCurve3(pts), Math.min(400, pts.length * 2), r * 0.006, 5, false);
    const m = new T.Mesh(geo, new T.MeshBasicMaterial({ color: walkCols[k] }));
    planetGroup.add(m); st.seamObjs.push(m);
  }
  cur = st;
  update();
}

// rigid fold: each child turns about its hinge (in flat coordinates) by +-a, towards -z
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
function cylinder(X, Y, u) {
  if (u < 1e-4) return new T.Vector3(X, Y, 0);
  const r = 1 / u, ph = X * u;
  return new T.Vector3(r * Math.sin(ph), Y, r * Math.cos(ph) - r + u);
}
// positions of every vertex at fold fraction f (0 flat .. 1 folded), centred on the folded shape
function foldedPositions(st, f) {
  const out = [];
  if (st.M.fold === 'cylinder') {
    const u = ease(f);
    for (const tt of st.tiles) {
      const a = new Float32Array(tt.nv * 3);
      for (let k = 0; k < tt.nv; k++) { const v = cylinder(tt.flat[2 * k], tt.flat[2 * k + 1], u); a[3 * k] = v.x; a[3 * k + 1] = v.y; a[3 * k + 2] = v.z; }
      out.push(a);
    }
    return out;
  }
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
  // Rk maps folded directions to planet directions: Fs * Ff^T
  return Fs.multiply(Ff.clone().transpose());
}

const qI = new T.Quaternion();
function update() {
  const st = cur;
  if (!st) return;
  if (st.M) {
    const f = Math.min(t, 1), inflate = ease(Math.max(0, t - 1));
    const fp = foldedPositions(st, f);
    const toPlanet = Dm.clone().multiply(st.Rk);
    const qT = new T.Quaternion().setFromRotationMatrix(toPlanet);
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
      tt.mesh.visible = !(opts.planetTex && t >= 1.999);
    });
  }
  const show = !st.M || t >= 1.97;
  for (const o of st.seamObjs) o.visible = show;
  if (st.globe) st.globe.visible = !st.M || (opts.planetTex && t >= 1.999);
  render();
}

// ------------------------------------------------------------------------------------------
// camera orbit
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
    if (pinch0) dist = Math.max(1.6, Math.min(9, dist * pinch0 / d));
    pinch0 = d;
  }
  render();
});
const up = (e) => { ptrs.delete(e.pointerId); pinch0 = 0; canvas.style.cursor = 'grab'; };
canvas.addEventListener('pointerup', up);
canvas.addEventListener('pointercancel', up);
canvas.addEventListener('wheel', (e) => { e.preventDefault(); dist = Math.max(1.6, Math.min(9, dist * Math.exp(e.deltaY * 0.001))); render(); }, { passive: false });
window.addEventListener('resize', resize);

// ------------------------------------------------------------------------------------------
// controls
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
  'lambert': 'Lambert: the strip rolls into a cylinder and each point moves straight out to the sphere (Archimedes\' equal-area map). The caps above 80° are never reached.',
  'guyou-petroff': 'Guyou layout (Petroff\'s map): the two hemisphere squares fold together into a pillowcase, glued round all four edges, then inflate. The corners of the pillowcase are the four singular points.',
  'guyou-peirce': 'The prototype\'s conformal map in the same pillowcase.',
  'guyou-gringorten': 'Gringorten in the pillowcase.', 'guyou-collignon': 'Collignon in the pillowcase.',
  'cube-moser-cross': 'Cross net: the four band faces wrap into a ring, the polar faces close the lid and floor, then the cube inflates along the Moser map. Seams (coloured) and the eight corners appear on the planet at the end.',
  'cube-moser-3x2': '3 × 2 net: the same cube, unfolded as two strips of three. It folds into the same cube; only which edges are seams differs.',
  'cube-qsc-cross': 'QSC folds the same way; switch the map explorer to its crease test to see where it kinks.',
  'cube-healpix-cross': 'HEALPix on the cross net: its corners sit at ±41.8° rather than ±35.3°.',
  'cube-eac-cross': 'The equi-angular cube.', 'cube-conformal-cross': 'The conformal cube, the Moser map\'s starting point.',
  'peirce-square': 'Peirce\'s square has no fold here: seams and corners shown on the planet.',
};
function setCaption(id) { document.getElementById('g-caption').textContent = caps[id] || ''; }
W.FAMNAME && (function () {
  const box = document.getElementById('g-picker');
  const all = ids.concat(extra);
  for (const fam of ['now', 'pill', 'cube']) {
    const row = document.createElement('div');
    row.className = 'row';
    row.innerHTML = `<span class="fam">${W.FAMNAME[fam]}</span>`;
    for (const id of W.ORDER[fam]) {
      if (!all.includes(id)) continue;
      const b = document.createElement('button');
      b.type = 'button'; b.className = 'chip'; b.id = 'g-' + id;
      b.setAttribute('aria-pressed', String(id === 'cube-moser-cross'));
      b.textContent = W.C[id].short;
      b.addEventListener('click', () => { box.querySelectorAll('.chip').forEach((x) => x.setAttribute('aria-pressed', 'false')); b.setAttribute('aria-pressed', 'true'); build(id); setCaption(id); });
      row.appendChild(b);
    }
    box.appendChild(row);
  }
  const ch = document.getElementById('g-checks');
  for (const [k, lab] of [['seams', 'Seams'], ['corners', 'Corners'], ['walks', 'Walks'], ['corner', 'Corner walkers'], ['planetTex', 'Bake texture (not the world map)']]) {
    const l = document.createElement('label');
    l.innerHTML = `<input type="checkbox" id="g-ov-${k}" ${opts[k] ? 'checked' : ''}> ${lab}`;
    l.querySelector('input').addEventListener('change', (e) => { opts[k] = e.target.checked; build(cur.id); });
    ch.appendChild(l);
  }
})();
const hg = location.hash.match(/gt-(\d+)(?:-([a-z0-9-]+?))?(?:,|$)/);
const startId = hg && hg[2] && D.layouts[hg[2]] ? hg[2] : 'cube-moser-cross';
if (hg) { t = Math.min(2, +hg[1] / 100); slider.value = Math.round(t * 100); }
build(startId);
setCaption(startId);
document.querySelectorAll('#g-picker .chip').forEach((x) => x.setAttribute('aria-pressed', String(x.id === 'g-' + startId)));
resize();
})();
