// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time workbench: the "Final world" view (Ben, 2026-10-01, "Zoomable final world"). A deep-zoom
// map of a finished run's bake: the satellite look, the elevation under a hill shade, or the water,
// from the whole planet down to a block a pixel (and past it, magnified), cut on demand by the
// server (`api/runs/<id>/tiles/<layer>/<z>/<x>/<y>.png`, crates/deeptime-cli/src/watch/tiles.rs) as
// an XYZ pyramid of 256-pixel tiles (zoom 0 is two tiles; each zoom doubles both ways; x runs east
// from the seam, y south from the north edge).
//
// This file is the whole view: its own small slippy map on a canvas (no map library, nothing from
// the network), loaded after workbench.js, which hands it `window.DTWB`. Pan and zoom with the
// mouse (drag, wheel, double click) or touch (drag, pinch); the world wraps east-west like the
// planet; the readout gives blocks (x, z), latitude and longitude and, after a pause, what the
// bake says there (`api/runs/<id>/world/sample`); "3D preview here" hands the view's box to the
// Area 3D preview. A view keeps the same block under the cursor while it zooms.
'use strict';

(function () {
  const DT = window.DTWB;
  const stage = document.getElementById('stage');
  const host = document.getElementById('worldview');
  const toggle = document.getElementById('worldbtn');
  if (!DT || !stage || !host || !toggle) return;
  const { esc, fmtInt, latText, lonText } = DT;
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  const proj = window.DTProj;   // the planet's projection (projection.js)
  const T = 256;                       // tile edge, px
  const MAX_FLIGHT = 8;                // tiles loading at once
  const KEEP_TILES = 420;              // decoded tiles kept
  const MAG = 4;                       // magnification past the finest zoom: screen px per block

  const css = document.createElement('style');
  css.textContent = `
.stage.worldmode .viewport { display: none; }
.stage.worldmode .stagebar > :not(.worldseg):not(.grow) { display: none; }
.worldview { position: relative; flex: 1 1 auto; min-height: 0; margin: 8px 14px 14px; border-radius: 10px; overflow: hidden; background: #04060a;
  box-shadow: 0 0 0 1px rgba(160, 175, 200, .22), 0 24px 70px rgba(0, 0, 0, .55); }
.worldview[hidden] { display: none; }
.worldview canvas { position: absolute; inset: 0; width: 100%; height: 100%; display: block; cursor: grab; touch-action: none; outline: none; }
.worldview canvas.drag { cursor: grabbing; }
.wv-bar { position: absolute; z-index: 3; left: 10px; right: 10px; top: 10px; display: flex; gap: 8px; align-items: center; flex-wrap: wrap; pointer-events: none; }
.wv-bar > * { pointer-events: auto; }
.wv-bar .seg, .wv-bar .btn:not(.btn-primary) { background: rgba(10, 13, 19, .78); backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px); }
.wv-bar .seg > button[aria-pressed="true"] { background: var(--bg-4); }
.wv-bar .grow { flex: 1 1 auto; pointer-events: none; }
.wv-readout { position: absolute; z-index: 3; left: 10px; bottom: 10px; max-width: calc(100% - 20px); padding: 6px 10px; border-radius: 8px; font: 500 12px/1.5 var(--mono, ui-monospace, monospace);
  background: rgba(10, 13, 19, .82); border: 1px solid var(--line-2, #2a3140); color: var(--fg-2, #aab); pointer-events: none; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.wv-readout b { color: var(--fg-1, #eef); font-weight: 600; }
.wv-readout .dim { color: var(--fg-3, #889); }
.wv-scale { position: absolute; z-index: 3; right: 12px; bottom: 12px; display: flex; flex-direction: column; align-items: flex-end; gap: 3px; font: 500 11px/1 var(--mono, ui-monospace, monospace); color: #fff; text-shadow: 0 1px 2px #000; pointer-events: none; }
.wv-scale i { display: block; height: 6px; border: 2px solid #fff; border-top: 0; box-shadow: 0 1px 2px #000; }
.wv-msg { position: absolute; inset: 0; z-index: 4; display: grid; place-items: center; text-align: center; padding: 24px; color: var(--fg-2, #aab); background: rgba(4, 6, 10, .86); }
.wv-msg[hidden] { display: none; }
.wv-msg h4 { margin: 8px 0 4px; font-size: 17px; color: var(--fg-1, #eef); }
.wv-msg p { margin: 0; max-width: 46ch; font-size: 13px; }
.wv-info { position: absolute; z-index: 5; left: 10px; top: 56px; width: min(420px, calc(100% - 20px)); max-height: calc(100% - 120px); overflow: auto; padding: 12px 14px; border-radius: 10px; font-size: 12.5px; line-height: 1.5;
  background: rgba(10, 13, 19, .94); border: 1px solid var(--line-2, #2a3140); color: var(--fg-2, #aab); }
.wv-info[hidden] { display: none; }
.wv-info h5 { margin: 0 0 4px; font-size: 13px; color: var(--fg-1, #eef); }
.wv-info p { margin: 0 0 8px; }
.wv-info code { font-family: var(--mono, ui-monospace, monospace); font-size: 11.5px; color: var(--fg-1, #eef); }
.wv-info table { border-collapse: collapse; width: 100%; font-size: 12px; }
.wv-info td { padding: 1px 6px 1px 0; vertical-align: top; }
.wv-info td:last-child { text-align: right; color: var(--fg-1, #eef); font-family: var(--mono, ui-monospace, monospace); }
@media (max-width: 760px) { .worldview { margin: 6px 8px 8px; } .wv-bar .lbl { display: none; } }
`;
  document.head.appendChild(css);

  host.innerHTML = `
<canvas id="wv-canvas" tabindex="0" aria-label="Map of the finished world: drag to pan, wheel or pinch to zoom"></canvas>
<div class="wv-bar">
  <div class="seg" role="group" aria-label="Layer" id="wv-layers"></div>
  <button class="btn btn-sm" id="wv-ovl" aria-pressed="false" data-tip="Draw rivers and lakes over this layer"><span class="lbl">Rivers &amp; lakes</span></button>
  <button class="btn btn-sm" id="wv-grid" aria-pressed="false" data-tip="Latitude and longitude lines, and the seam"><span class="lbl">Grid</span></button>
  <span class="grow"></span>
  <button class="btn btn-sm" id="wv-info-btn" aria-pressed="false" data-tip="What this map is made of, its cache and its speed">About</button>
  <div class="seg" role="group" aria-label="Zoom">
    <button id="wv-out" aria-label="Zoom out" data-tip="Zoom out" data-key="−">−</button>
    <button id="wv-home" data-tip="The whole planet" data-key="0">Whole world</button>
    <button id="wv-in" aria-label="Zoom in" data-tip="Zoom in" data-key="+">+</button>
  </div>
  <button class="btn btn-primary btn-sm" id="wv-3d" data-tip="Open the 3D block preview of the area in the middle of this view (at most 256 × 256 blocks)">3D preview here</button>
</div>
<div class="wv-readout" id="wv-readout" aria-live="off">Move over the map</div>
<div class="wv-scale" id="wv-scale"></div>
<div class="wv-info" id="wv-info" hidden></div>
<div class="wv-msg" id="wv-msg"><div><div class="spinner l"></div><h4>Final world</h4><p>Choose a run.</p></div></div>`;
  const $ = id => document.getElementById(id);
  const canvas = $('wv-canvas'), ctx = canvas.getContext('2d');

  // ----------------------------------------------------------------------------------------------
  // State
  // ----------------------------------------------------------------------------------------------
  const S = {
    open: false, run: null, info: null, layer: 'sat', overlay: false, grid: false,
    cx: 0, cz: 0, bpp: 1,            // view centre (blocks) and blocks per CSS pixel
    w: 0, h: 0, dpr: 1, dirty: true, pointers: new Map(), pinch: null, views: new Map(),
    cursor: null, sampleTimer: 0, sampleSeq: 0, sample: null,
  };
  // A link to a view: `?world=<layer>/<x>/<z>/<blocks per pixel>` opens the map there (the run is
  // `?run=<id>`); the address follows the view, so it can be copied.
  const deep = (() => {
    const v = (new URLSearchParams(location.search).get('world') || '').split('/');
    if (v.length < 4 || !['sat', 'elev', 'water'].includes(v[0])) return null;
    const [cx, cz, bpp] = v.slice(1).map(Number);
    return [cx, cz, bpp].every(isFinite) && bpp > 0 ? { layer: v[0], cx, cz, bpp } : null;
  })();
  let linkTimer = 0;
  function syncLink() {
    clearTimeout(linkTimer);
    linkTimer = setTimeout(() => {
      const q = new URLSearchParams(location.search);
      if (S.open && S.info) q.set('world', `${S.layer}/${Math.round(S.cx)}/${Math.round(S.cz)}/${+S.bpp.toFixed(3)}`); else q.delete('world');
      const qs = q.toString();
      history.replaceState(null, '', `${location.pathname}${qs ? `?${qs}` : ''}${location.hash}`);
    }, 500);
  }
  const C = () => S.info.circumference;
  // The world's projection (the manifest's layout block, world.json `projection`): Petroff–Guyou
  // by default (format 0.5), where the tile frame is exactly the world, or Lambert.
  const Wp = () => {
    if (!S.info) return proj.world(null, 16384);
    if (!S.info._wp) S.info._wp = proj.world(S.info.projection, S.info.circumference);
    return S.info._wp;
  };
  const zLim = () => Wp().zRange[1];
  const s0 = () => C() / (2 * T);                    // blocks per tile pixel at zoom 0
  const minBpp = () => 1 / MAG;
  const maxBpp = () => Math.max(C() / Math.max(S.w, 1), (2 * zLim()) / Math.max(S.h, 1));

  function setView(cx, cz, bpp) {
    S.bpp = clamp(bpp, minBpp(), maxBpp());
    const halfH = S.h * S.bpp / 2;
    // Vertically the view stays on the planet (or centred when it is taller than the planet).
    const lim = zLim();
    S.cz = halfH >= lim ? 0 : clamp(cz, -lim + halfH, lim - halfH);
    const c = C();
    S.cx = ((cx + c / 2) % c + c) % c - c / 2;
    S.dirty = true;
    if (S.run) S.views.set(S.run.id, { cx: S.cx, cz: S.cz, bpp: S.bpp });
    syncLink();
  }
  const zoomAt = (px, py, factor) => {
    // Keep the block under (px, py) where it is.
    const [bx, bz] = toWorld(px, py);
    const bpp = clamp(S.bpp / factor, minBpp(), maxBpp());
    setView(bx - (px - S.w / 2) * bpp, bz - (py - S.h / 2) * bpp, bpp);
  };
  const toWorld = (px, py) => [S.cx + (px - S.w / 2) * S.bpp, S.cz + (py - S.h / 2) * S.bpp];
  const toScreen = (bx, bz) => [S.w / 2 + (bx - S.cx) / S.bpp, S.h / 2 + (bz - S.cz) / S.bpp];
  const home = () => setView(0, 0, maxBpp());

  // ----------------------------------------------------------------------------------------------
  // Tiles
  // ----------------------------------------------------------------------------------------------
  const tiles = new Map();           // key -> { img, state: 'loading' | 'ok' | 'err', t }
  let inflight = 0, clock = 0;
  const tileURL = (layer, z, x, y) => `api/runs/${encodeURIComponent(S.run.id)}/tiles/${layer}/${z}/${x}/${y}.png?v=${S.info.version}`;
  const tileKey = (layer, z, x, y) => `${S.run.id}/${layer}/${z}/${x}/${y}`;
  function tileAt(layer, z, x, y, start) {
    const key = tileKey(layer, z, x, y);
    let t = tiles.get(key);
    if (t) { t.t = ++clock; return t; }
    if (!start) return null;
    t = { img: null, state: 'queued', t: ++clock, layer, z, x, y };
    tiles.set(key, t);
    return t;
  }
  function pump(want) {
    // Start the nearest wanted tiles that are not loading yet; the rest wait for the next draw.
    for (const t of want) {
      if (inflight >= MAX_FLIGHT) break;
      if (t.state !== 'queued') continue;
      t.state = 'loading'; inflight++;
      const img = new Image();
      img.decoding = 'async';
      img.onload = () => { t.img = img; t.state = 'ok'; inflight--; S.dirty = true; };
      img.onerror = () => { t.state = 'err'; inflight--; setTimeout(() => { if (t.state === 'err') tiles.delete(tileKey(t.layer, t.z, t.x, t.y)); }, 4000); S.dirty = true; };
      img.src = tileURL(t.layer, t.z, t.x, t.y);
    }
    if (tiles.size > KEEP_TILES) {
      const old = [...tiles.entries()].filter(([, t]) => t.state === 'ok' || t.state === 'err').sort((a, b) => a[1].t - b[1].t);
      for (const [k] of old.slice(0, tiles.size - KEEP_TILES)) tiles.delete(k);
    }
  }
  // The tile zoom for the current scale: the first whose pixel is no wider than a device pixel.
  function tileZoom() {
    const dev = S.bpp / S.dpr;                       // blocks per device pixel
    const z = Math.ceil(Math.log2(s0() / dev) - 0.3);
    return clamp(z, 0, S.info.max_zoom);
  }

  // ----------------------------------------------------------------------------------------------
  // Drawing
  // ----------------------------------------------------------------------------------------------
  function resize() {
    const r = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const w = Math.max(1, Math.round(r.width)), h = Math.max(1, Math.round(r.height));
    if (w === S.w && h === S.h && dpr === S.dpr) return;
    const first = S.w === 0;
    S.w = w; S.h = h; S.dpr = dpr;
    canvas.width = Math.round(w * dpr); canvas.height = Math.round(h * dpr);
    if (S.info && !first) setView(S.cx, S.cz, S.bpp);
    S.dirty = true;
  }

  function drawLayer(layer, z, want, alpha) {
    const sz = s0() / (1 << z);                      // blocks per tile pixel
    const span = T * sz;                              // blocks per tile
    const px = span / S.bpp;                          // CSS px per tile
    const c = C();
    const left = S.cx - S.w / 2 * S.bpp, top = S.cz - S.h / 2 * S.bpp;
    const k0 = Math.floor((left + c / 2) / span), k1 = Math.floor((left + S.w * S.bpp + c / 2) / span);
    const fz = proj.tileFrameZ(c);   // the tile pyramid's frame (worldmap.rs): 2 : 1, wider than the planet
    const j0 = Math.max(0, Math.floor((top - fz) / span)), j1 = Math.min((1 << z) - 1, Math.floor((top + S.h * S.bpp - fz) / span));
    const nx = 2 << z;
    const d = S.dpr;
    ctx.globalAlpha = alpha;
    ctx.imageSmoothingEnabled = px * d / T < 1.75;   // pixelated once a tile pixel is more than a screen pixel and a half
    for (let j = j0; j <= j1; j++) {
      for (let k = k0; k <= k1; k++) {
        const tx = ((k % nx) + nx) % nx;
        const sx = ((-c / 2 + k * span) - left) / S.bpp, sy = ((fz + j * span) - top) / S.bpp;
        const t = tileAt(layer, z, tx, j, true);
        if (t.state === 'ok') {
          ctx.drawImage(t.img, Math.round(sx * d), Math.round(sy * d), Math.ceil(px * d + 0.5), Math.ceil(px * d + 0.5));
        } else {
          want.push({ t, dist: Math.hypot(sx + px / 2 - S.w / 2, sy + px / 2 - S.h / 2) });
          // Meanwhile: the nearest ancestor that is in, cropped and magnified.
          for (let up = 1; up <= 5 && z - up >= 0; up++) {
            const a = tileAt(layer, z - up, tx >> up, j >> up, false);
            if (a && a.state === 'ok') {
              const n = T >> up, ox = (tx & ((1 << up) - 1)) * n, oy = (j & ((1 << up) - 1)) * n;
              ctx.drawImage(a.img, ox, oy, n, n, Math.round(sx * d), Math.round(sy * d), Math.ceil(px * d + 0.5), Math.ceil(px * d + 0.5));
              break;
            }
          }
        }
      }
    }
    ctx.globalAlpha = 1;
  }

  function niceStep(v) {
    const e = Math.pow(10, Math.floor(Math.log10(v))), m = v / e;
    return (m >= 5 ? 5 : m >= 2 ? 2 : 1) * e;
  }

  function drawGrid() {
    const c = C(), d = S.dpr;
    const left = S.cx - S.w / 2 * S.bpp, top = S.cz - S.h / 2 * S.bpp;
    const degPerBlock = proj.dLonOfDx(c, 1, 0);
    const stepDeg = niceStep(Math.max(S.w, S.h) * S.bpp * degPerBlock / 8);
    const steps = [1, 2, 5, 10, 15, 30, 45, 90];
    const step = steps.find(s => s >= stepDeg) || 90;
    ctx.save();
    ctx.lineWidth = 1; ctx.font = `${11 * d}px ui-monospace, monospace`; ctx.textBaseline = 'top';
    ctx.strokeStyle = 'rgba(255,255,255,.28)'; ctx.fillStyle = 'rgba(255,255,255,.8)';
    for (let lat = -90; lat <= 90; lat += step) {
      const [, y] = toScreen(0, proj.zOfLat(c, lat));
      if (y < 0 || y > S.h) continue;
      ctx.beginPath(); ctx.moveTo(0, Math.round(y * d) + .5); ctx.lineTo(S.w * d, Math.round(y * d) + .5); ctx.stroke();
      ctx.fillText(`${Math.abs(lat)}°${lat < 0 ? 'S' : lat > 0 ? 'N' : ''}`, 56 * d, y * d + 2 * d);
    }
    const lonSpan = proj.dxOfDLon(c, step, 0);   // blocks between meridians
    for (let k = Math.floor((left + c / 2) / lonSpan) - 1; k <= Math.ceil((left + S.w * S.bpp + c / 2) / lonSpan) + 1; k++) {
      const bx = proj.xOfLon(c, k * step);
      const [x] = toScreen(bx, 0);
      if (x < 0 || x > S.w) continue;
      const lon = ((k * step) % 360 + 360) % 360;
      const seam = lon === 0;
      ctx.strokeStyle = seam ? 'rgba(255,200,90,.7)' : 'rgba(255,255,255,.28)';
      ctx.beginPath(); ctx.moveTo(Math.round(x * d) + .5, 0); ctx.lineTo(Math.round(x * d) + .5, S.h * d); ctx.stroke();
      ctx.fillText(seam ? 'seam 0°' : `${lon}°E`, x * d + 3 * d, (S.h - 60) * d);
    }
    ctx.restore();
    void top;
  }

  // The 256 × 256 block square a 3D preview would cover, in the middle of the view.
  function previewBox() {
    const c = C();
    const hw = Math.min(128, S.w * S.bpp / 2), hh = Math.min(128, S.h * S.bpp / 2);
    const x0 = Math.round(S.cx - hw), z0 = Math.round(S.cz - hh);
    const x1 = Math.round(S.cx + hw), z1 = Math.round(S.cz + hh);
    const lim = zLim();
    void c;
    const zz0 = clamp(z0, -lim, lim - 1), zz1 = clamp(z1, zz0 + 1, lim);
    return { x0, z0: zz0, x1: Math.max(x1, x0 + 1), z1: zz1 };
  }
  function drawPreviewBox() {
    if (!S.open) return;
    const b = previewBox();
    const [x0, y0] = toScreen(b.x0, b.z0), [x1, y1] = toScreen(b.x1, b.z1);
    if (x1 - x0 < 24) return;
    const d = S.dpr;
    ctx.save();
    ctx.setLineDash([6 * d, 4 * d]); ctx.lineWidth = 1.5 * d;
    ctx.strokeStyle = 'rgba(0,0,0,.65)'; ctx.strokeRect(x0 * d + 1, y0 * d + 1, (x1 - x0) * d, (y1 - y0) * d);
    ctx.strokeStyle = 'rgba(255,255,255,.95)'; ctx.strokeRect(x0 * d, y0 * d, (x1 - x0) * d, (y1 - y0) * d);
    ctx.restore();
  }

  function scaleBar() {
    const el = $('wv-scale');
    if (!S.info) { el.innerHTML = ''; return; }
    const target = 90 * S.bpp;                       // blocks in about 90 px
    const n = niceStep(target);
    const px = n / S.bpp;
    el.innerHTML = `<span>${fmtInt(n)} blocks</span><i style="width:${px.toFixed(0)}px"></i>`;
  }

  function draw() {
    if (!S.open || !S.info || !S.run) return;
    resize();
    const d = S.dpr;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = '#04060a';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const z = tileZoom();
    const want = [];
    drawLayer(S.layer, z, want, 1);
    if (S.overlay && S.layer !== 'water' && hasLayer('overlay')) drawLayer('overlay', z, want, 1);
    if (S.grid) drawGrid();
    drawPreviewBox();
    want.sort((a, b) => a.dist - b.dist);
    pump(want.map(w => w.t));
    scaleBar();
    readout();
    void d;
  }
  const hasLayer = id => !!S.info && S.info.layers.some(l => l.id === id);

  let raf = 0;
  function frame() {
    raf = 0;
    if (S.dirty) { S.dirty = false; draw(); }
    if (S.open) raf = requestAnimationFrame(frame);
  }

  // ----------------------------------------------------------------------------------------------
  // The readout
  // ----------------------------------------------------------------------------------------------
  function readout() {
    const el = $('wv-readout');
    const z = S.info ? tileZoom() : 0;
    const bpp = S.bpp;
    const scaleText = bpp >= 1 ? `${bpp >= 10 ? bpp.toFixed(0) : bpp.toFixed(1)} blocks a pixel` : `${(1 / bpp).toFixed(1)} pixels a block`;
    if (!S.cursor) { el.innerHTML = `<span class="dim">${esc(scaleText)} · zoom ${z} of ${S.info ? S.info.max_zoom : 0}</span>`; return; }
    const [bx, bz] = toWorld(S.cursor[0], S.cursor[1]);
    const c = C();
    const x = Math.floor(((bx + c / 2) % c + c) % c - c / 2), zz = Math.floor(bz);
    const [lat, lon] = Wp().latLon(bx, bz);
    let h = `x <b>${fmtInt(x)}</b> z <b>${fmtInt(zz)}</b><span class="dim"> · </span><b>${esc(latText(lat))}</b> ${esc(lonText(lon))}`;
    const s = S.sample && S.sample.x === x && S.sample.z === zz ? S.sample : null;
    if (s) {
      h += `<span class="dim"> · </span>y <b>${fmtInt(s.terrain_y)}</b>`;
      if (s.surface && s.surface !== 'land') h += ` <b>${esc(s.surface)}</b>`;
      if (s.koppen) h += `<span class="dim"> · </span>${esc(s.koppen)}`;
      if (s.t_mean_c !== undefined) h += ` ${s.t_mean_c} °C`;
      if (s.precip_mm !== undefined) h += ` ${fmtInt(s.precip_mm)} mm`;
    }
    h += `<span class="dim"> · ${esc(scaleText)}</span>`;
    el.innerHTML = h;
  }

  function askSample() {
    clearTimeout(S.sampleTimer);
    if (!S.cursor || !S.run || !S.info) return;
    S.sampleTimer = setTimeout(async () => {
      const [bx, bz] = toWorld(S.cursor[0], S.cursor[1]);
      const c = C();
      const x = Math.floor(((bx + c / 2) % c + c) % c - c / 2), z = Math.floor(bz);
      const seq = ++S.sampleSeq;
      const res = await DT.api(`api/runs/${encodeURIComponent(S.run.id)}/world/sample?x=${x}&z=${z}`);
      if (seq !== S.sampleSeq || !res.ok) return;
      S.sample = res.body; S.dirty = true;
    }, 140);
  }

  // ----------------------------------------------------------------------------------------------
  // Pointer, wheel, keys
  // ----------------------------------------------------------------------------------------------
  const rel = ev => { const r = canvas.getBoundingClientRect(); return [ev.clientX - r.left, ev.clientY - r.top]; };
  canvas.addEventListener('pointerdown', ev => {
    if (!S.info) return;
    canvas.focus({ preventScroll: true });
    canvas.setPointerCapture(ev.pointerId);
    const [x, y] = rel(ev);
    S.pointers.set(ev.pointerId, { x, y, sx: x, sy: y, t: performance.now() });
    if (S.pointers.size === 2) {
      const [a, b] = [...S.pointers.values()];
      S.pinch = { d: Math.hypot(a.x - b.x, a.y - b.y), bpp: S.bpp };
    }
    canvas.classList.add('drag');
  });
  canvas.addEventListener('pointermove', ev => {
    if (!S.info) return;
    const [x, y] = rel(ev);
    S.cursor = [x, y];
    const p = S.pointers.get(ev.pointerId);
    if (!p) { S.dirty = true; askSample(); return; }
    if (S.pointers.size === 1) {
      setView(S.cx - (x - p.x) * S.bpp, S.cz - (y - p.y) * S.bpp, S.bpp);
      S.sample = null;
    } else if (S.pointers.size === 2 && S.pinch) {
      const other = [...S.pointers.entries()].find(([id]) => id !== ev.pointerId)[1];
      const prev = Math.hypot(p.x - other.x, p.y - other.y), now = Math.hypot(x - other.x, y - other.y);
      const mx = (x + other.x) / 2, my = (y + other.y) / 2;
      const pmx = (p.x + other.x) / 2, pmy = (p.y + other.y) / 2;
      // Zoom about the midpoint, and follow it as it moves.
      const [bx, bz] = toWorld(pmx, pmy);
      const bpp = clamp(S.bpp * (prev / Math.max(now, 1)), minBpp(), maxBpp());
      setView(bx - (mx - S.w / 2) * bpp, bz - (my - S.h / 2) * bpp, bpp);
    }
    p.x = x; p.y = y;
    S.dirty = true;
  });
  const lift = ev => {
    S.pointers.delete(ev.pointerId);
    if (S.pointers.size < 2) S.pinch = null;
    if (!S.pointers.size) canvas.classList.remove('drag');
  };
  canvas.addEventListener('pointerup', lift);
  canvas.addEventListener('pointercancel', lift);
  canvas.addEventListener('pointerleave', () => { if (!S.pointers.size) { S.cursor = null; S.dirty = true; } });
  canvas.addEventListener('wheel', ev => {
    if (!S.info) return;
    ev.preventDefault();
    const [x, y] = rel(ev);
    const dy = ev.deltaMode === 1 ? ev.deltaY * 16 : ev.deltaMode === 2 ? ev.deltaY * 400 : ev.deltaY;
    zoomAt(x, y, Math.exp(-clamp(dy, -240, 240) * (ev.ctrlKey ? 0.01 : 0.0018)));
    S.sample = null; askSample();
  }, { passive: false });
  canvas.addEventListener('dblclick', ev => {
    if (!S.info) return;
    const [x, y] = rel(ev);
    zoomAt(x, y, ev.shiftKey ? 0.5 : 2);
  });
  canvas.addEventListener('keydown', ev => {
    if (!S.info || ev.metaKey || ev.ctrlKey || ev.altKey) return;
    const k = ev.key, step = 0.2 * S.bpp;
    let used = true;
    if (k === 'ArrowLeft') setView(S.cx - S.w * step, S.cz, S.bpp);
    else if (k === 'ArrowRight') setView(S.cx + S.w * step, S.cz, S.bpp);
    else if (k === 'ArrowUp') setView(S.cx, S.cz - S.h * step, S.bpp);
    else if (k === 'ArrowDown') setView(S.cx, S.cz + S.h * step, S.bpp);
    else if (k === '+' || k === '=') zoomAt(S.w / 2, S.h / 2, 1.6);
    else if (k === '-' || k === '_') zoomAt(S.w / 2, S.h / 2, 1 / 1.6);
    else if (k === '0' || k === 'Home') home();
    else if (k === 'Escape') setOpen(false);
    else used = false;
    if (used) { ev.preventDefault(); ev.stopPropagation(); }
  });
  window.addEventListener('resize', () => { S.dirty = true; });
  new ResizeObserver(() => { S.dirty = true; }).observe(host);

  // ----------------------------------------------------------------------------------------------
  // Controls
  // ----------------------------------------------------------------------------------------------
  function buildLayers() {
    const box = $('wv-layers');
    const base = S.info ? S.info.layers.filter(l => !l.overlay) : [];
    box.innerHTML = base.map(l => `<button data-l="${esc(l.id)}" aria-pressed="${l.id === S.layer}">${esc(l.label)}</button>`).join('');
    $('wv-ovl').hidden = !hasLayer('overlay');
    $('wv-ovl').setAttribute('aria-pressed', String(S.overlay));
    $('wv-grid').setAttribute('aria-pressed', String(S.grid));
  }
  $('wv-layers').addEventListener('click', ev => {
    const b = ev.target.closest('[data-l]');
    if (!b) return;
    S.layer = b.dataset.l; DT.store && DT.store.set('wvLayer', S.layer);
    buildLayers(); S.dirty = true; syncLink();
  });
  $('wv-ovl').addEventListener('click', () => { S.overlay = !S.overlay; buildLayers(); S.dirty = true; });
  $('wv-grid').addEventListener('click', () => { S.grid = !S.grid; buildLayers(); S.dirty = true; });
  $('wv-in').addEventListener('click', () => zoomAt(S.w / 2, S.h / 2, 2));
  $('wv-out').addEventListener('click', () => zoomAt(S.w / 2, S.h / 2, 0.5));
  $('wv-home').addEventListener('click', home);
  $('wv-info-btn').addEventListener('click', () => {
    const el = $('wv-info'), on = el.hidden;
    el.hidden = !on; $('wv-info-btn').setAttribute('aria-pressed', String(on));
    if (on) refreshInfo(true);
  });

  $('wv-3d').addEventListener('click', () => {
    if (!S.info || !S.run) return;
    const c = C(), b = previewBox();
    const x0 = ((b.x0 + c / 2) % c + c) % c - c / 2;       // folded; a box across the seam is split by the runner
    const box = { x0, z0: b.z0, x1: x0 + (b.x1 - b.x0), z1: b.z1 };
    let ll;
    if (Wp().isPetroff) {
      // A block box of the Petroff–Guyou world is no lat/lon box: its corners' latitudes and
      // longitudes, for the preview's label.
      const c0 = Wp().latLon(box.x0, box.z0), c1 = Wp().latLon(box.x1, box.z1);
      ll = { north: Math.max(c0[0], c1[0]), south: Math.min(c0[0], c1[0]), west: c0[1], east: c1[1] };
    } else {
      const west = proj.lonOfX(c, box.x0);
      ll = { north: proj.latOfZ(c, box.z0), south: proj.latOfZ(c, box.z1), west, east: west + proj.dLonOfDx(c, box.x1 - box.x0, proj.latOfZ(c, (box.z0 + box.z1) / 2)) };
    }
    const w = box.x1 - box.x0, h = box.z1 - box.z0;
    DT.preview(S.run, box, ll, `${fmtInt(w)} × ${fmtInt(h)} blocks, from the final world map`);
  });

  function infoHTML() {
    const i = S.info;
    if (!i) return '';
    const t = i.timing || {};
    const rows = Object.entries(t).map(([k, v]) => `<tr><td>${esc(k)} tiles drawn</td><td>${fmtInt(v.tiles)} · mean ${v.mean_ms.toFixed(0)} ms · max ${v.max_ms.toFixed(0)} ms</td></tr>`).join('');
    const mib = b => (b / 1048576).toFixed(b >= 1.05e8 ? 0 : 1);
    return `<h5>What the finest zoom is</h5>
<p>${esc(i.finest.statement)}</p>
<p>The last native zoom (${i.max_zoom}) is <code>${i.finest.blocks_per_px}</code> block a pixel; the map magnifies up to ${MAG}× past it. The terrain height is <code>E + A·N</code> at every block (${i.finest.terrain_cell_blocks}-block grid, bilinear, plus sub-cell noise${i.finest.detail ? ` of ${i.finest.detail.wavelength_blocks}-block wavelength` : ''}); the game then draws it between corners four blocks apart.</p>
<h5>Tiles</h5>
<table><tr><td>Pyramid</td><td>${i.max_zoom + 1} zooms · ${fmtInt(i.pyramid.tiles_per_layer)} tiles a layer</td></tr>
<tr><td>Cached for this run</td><td>${fmtInt(i.cache.tiles)} tiles · ${mib(i.cache.bytes)} of ${mib(i.cache.cap_bytes)} MiB</td></tr>${rows}</table>
<p style="margin-top:8px">Drawn when first asked, cached on the Mac under the run's folder, least recently used dropped past the cap. Nothing here changes the run or its bake.</p>`;
  }
  async function refreshInfo(render) {
    if (!S.run) return;
    const res = await DT.api(`api/runs/${encodeURIComponent(S.run.id)}/world.json`);
    if (res.ok && res.body.available && S.info) {
      S.info.timing = res.body.timing; S.info.cache = res.body.cache;
      if (render || !$('wv-info').hidden) $('wv-info').innerHTML = infoHTML();
    }
  }

  // ----------------------------------------------------------------------------------------------
  // Which run, and showing it
  // ----------------------------------------------------------------------------------------------
  function message(html) {
    const m = $('wv-msg');
    m.hidden = !html;
    if (html) m.innerHTML = `<div>${html}</div>`;
  }
  let loading = null, retry = 0, deepPending = deep;
  async function load(run) {
    S.run = run; S.info = null; S.sample = null; clearTimeout(retry);
    if (!run) { message('<h4>Final world</h4><p>Open a run, then come back to see its finished world.</p>'); return; }
    message('<div class="spinner l"></div><h4>Opening the world…</h4><p>Reading the run’s bake.</p>');
    const mine = loading = {};
    const res = await DT.api(`api/runs/${encodeURIComponent(run.id)}/world.json`);
    if (loading !== mine || S.run !== run) return;
    if (!res.ok || !res.body.available) {
      const why = (res.body && (res.body.reason || res.body.error)) || 'It has no world to show.';
      message(`<h4>No final world yet</h4><p>${esc(why)}</p><p style="margin-top:8px;opacity:.7">This view fills in when the run is done; it checks again by itself.</p>`);
      retry = setTimeout(() => { if (S.open && S.run === run) load(run); }, 4000);
      return;
    }
    S.info = res.body;
    if (!S.info.layers.some(l => l.id === S.layer)) S.layer = 'sat';
    message('');
    buildLayers();
    $('wv-info').innerHTML = infoHTML();
    resize();
    const v = S.views.get(run.id);
    if (v) setView(v.cx, v.cz, v.bpp);
    else if (deepPending) { S.layer = deepPending.layer; buildLayers(); setView(deepPending.cx, deepPending.cz, deepPending.bpp); } else home();
    deepPending = null;
    canvas.focus({ preventScroll: true });
    S.dirty = true;
  }

  function setOpen(on) {
    S.open = on;
    stage.classList.toggle('worldmode', on);
    host.hidden = !on;
    toggle.setAttribute('aria-pressed', String(on));
    syncLink();
    if (on) {
      const run = DT.focusedRun();
      if (!S.info || !run || S.run !== run) load(run);
      S.dirty = true;
      if (!raf) raf = requestAnimationFrame(frame);
    }
  }
  toggle.addEventListener('click', () => setOpen(!S.open));
  document.addEventListener('keydown', ev => {
    if (ev.key !== 'w' || ev.metaKey || ev.ctrlKey || ev.altKey) return;
    if (ev.target.closest && ev.target.closest('input, select, textarea, button')) return;
    if (!document.getElementById('modal').hidden) return;
    setOpen(!S.open);
  });
  // Following the focused run while the view is open (the page's run list changes it).
  setInterval(() => {
    if (!S.open) return;
    const run = DT.focusedRun();
    if (run !== S.run) load(run);
  }, 600);
  setInterval(() => { if (S.open && S.info && !$('wv-info').hidden) refreshInfo(false); }, 3000);
  // Opened from a link: wait for the page to have a focused run.
  if (deep) {
    const wait = setInterval(() => { if (DT.focusedRun()) { clearInterval(wait); setOpen(true); } }, 200);
    setTimeout(() => clearInterval(wait), 20000);
  }
  try { S.layer = (DT.store && DT.store.get('wvLayer', 'sat')) || 'sat'; } catch (e) { S.layer = 'sat'; }
})();
