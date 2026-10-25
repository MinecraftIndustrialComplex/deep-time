// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time workbench: the "Final world" view (Ben, 2026-10-01, "Zoomable final world"). A deep-zoom
// map of a finished run's bake: the satellite look, the elevation under a hill shade, or the water,
// from the whole planet down to a block a pixel (and past it, magnified), cut on demand by the
// server (`api/runs/<id>/tiles/<layer>/<z>/<x>/<y>.png`, crates/deeptime-cli/src/watch/tiles.rs) as
// an XYZ pyramid of 256-pixel tiles (zoom 0 is two tiles; each zoom doubles both ways; x runs east
// from the seam, y south from the north edge).
//
// The slippy map itself is mic-viewer's map core (`mic-viewer/map.js`, a pinned git dependency,
// MPL-2.0, NOTICE.md; decisions log 2026-10-21), loaded before this file: pan and zoom with the mouse (drag,
// wheel, double click) or touch (drag, pinch), the world wrapping east-west like the planet, the
// tile cache and the scale bar. This file is Deep Time's plug-in on it, loaded after workbench.js,
// which hands it `window.DTWB`: the projection and the 2 : 1 tile pyramid, the layers, the grid,
// the readout (blocks (x, z), latitude and longitude and, after a pause, what the bake says there,
// `api/runs/<id>/world/sample`) and "3D preview here", which hands the view's box to the Area 3D
// preview. Nothing here comes from the network beyond this server.
'use strict';

(function () {
  const root = typeof window !== 'undefined' ? window : globalThis;
  const T = 256;                       // tile edge, px
  const MAG = 4;                       // magnification past the finest zoom: screen px per block

  // The map core's frame for a run's world.json: the planet wraps east-west every circumference and
  // stays between its poles; the tile pyramid (deeptime-tiles' `TileGeom`) is 2 : 1, 2^(z+1) × 2^z
  // tiles at zoom z, C / (512 · 2^z) blocks a tile pixel, from the seam's west edge and the frame's
  // north edge (`tileFrameZ`, wider than the planet). Pure, so tools/viewer/worldmap-check.js
  // checks it under node against the pyramid this view drew before the move.
  function frameOf(info) {
    const proj = root.DTProj;
    const c = info.circumference;
    const lim = proj.world(info.projection, c).zRange[1];
    const s0 = c / (2 * T);                          // blocks per tile pixel at zoom 0
    return {
      projection: { period: c, zRange: [-lim, lim] },
      pyramid: {
        T, minZoom: 0, maxZoom: info.max_zoom,
        pixel: z => s0 / Math.pow(2, z),
        originX: -c / 2, originZ: proj.tileFrameZ(c),
        rows: z => [0, (1 << z) - 1],
        cols: z => 2 << z,
      },
    };
  }
  root.DTWorldMap = { frameOf, T, MAG };
  if (typeof document === 'undefined') return;

  const DT = root.DTWB;
  const MV = root.MicViewer;
  const stage = document.getElementById('stage');
  const host = document.getElementById('worldview');
  const toggle = document.getElementById('worldbtn');
  if (!DT || !MV || !MV.createMap || !stage || !host || !toggle) return;
  const { esc, fmtInt, latText, lonText } = DT;
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  const proj = root.DTProj;   // the planet's projection (projection.js)

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
.wv-bar .lbl-s { display: none; }
.wv-readout { position: absolute; z-index: 3; left: 10px; bottom: 10px; max-width: calc(100% - 20px); padding: 6px 10px; border-radius: 8px; font: 500 12px/1.5 var(--mono, ui-monospace, monospace);
  background: rgba(10, 13, 19, .82); border: 1px solid var(--line-2, #2a3140); color: var(--fg-2, #aab); pointer-events: none; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.wv-readout b { color: var(--fg-1, #eef); font-weight: 600; }
.wv-readout .dim { color: var(--fg-3, #889); }
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
/* Phones: the stage is as tall as its content there, so the view (whose canvas is absolute) needs a
   height of its own or it collapses to nothing (it did before the map core moved too). */
@media (max-width: 760px) { .worldview { flex: 0 0 auto; height: 72vh; height: 72dvh; min-height: 320px; margin: 6px 8px 8px; } .wv-bar .lbl { display: none; } .wv-bar .lbl-s { display: inline; } }
`;
  document.head.appendChild(css);

  host.innerHTML = `
<canvas id="wv-canvas" tabindex="0" aria-label="Map of the finished world: drag to pan, wheel or pinch to zoom"></canvas>
<div class="wv-bar">
  <div class="seg" role="group" aria-label="Layer" id="wv-layers"></div>
  <button class="btn btn-sm" id="wv-ovl" aria-pressed="false" data-tip="Draw rivers and lakes over this layer"><span class="lbl">Rivers &amp; lakes</span><span class="lbl-s">Rivers</span></button>
  <button class="btn btn-sm" id="wv-grid" aria-pressed="false" data-tip="Latitude and longitude lines, and the seam">Grid</button>
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
<div class="wv-info" id="wv-info" hidden></div>
<div class="wv-msg" id="wv-msg"><div><div class="spinner l"></div><h4>Final world</h4><p>Choose a run.</p></div></div>`;
  const $ = id => document.getElementById(id);
  const canvas = $('wv-canvas');

  // ----------------------------------------------------------------------------------------------
  // State
  // ----------------------------------------------------------------------------------------------
  const S = {
    open: false, run: null, info: null, layer: 'sat', overlay: false, grid: false,
    views: new Map(), sampleTimer: 0, sampleSeq: 0, sample: null,
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
      const v = map.view;
      if (S.open && S.info) q.set('world', `${S.layer}/${Math.round(v.cx)}/${Math.round(v.cz)}/${+v.bpp.toFixed(3)}`); else q.delete('world');
      const qs = q.toString();
      history.replaceState(null, '', `${location.pathname}${qs ? `?${qs}` : ''}${location.hash}`);
    }, 500);
  }
  const C = () => S.info.circumference;
  // The world's projection (the manifest's layout block, world.json `projection`): the Lambert
  // frame until plan T1 wires the cube world in.
  const Wp = () => {
    if (!S.info) return proj.world(null, 16384);
    if (!S.info._wp) S.info._wp = proj.world(S.info.projection, S.info.circumference);
    return S.info._wp;
  };
  const zLim = () => Wp().zRange[1];

  const hasLayer = id => !!S.info && S.info.layers.some(l => l.id === id);
  const tileURL = (run, info, layer) => (z, x, y) =>
    `api/runs/${encodeURIComponent(run.id)}/tiles/${layer}/${z}/${x}/${y}.png?v=${info.version}`;
  function setTiles() {
    if (!S.info || !S.run) { map.setTiles([]); return; }
    const list = [{ id: S.layer, url: tileURL(S.run, S.info, S.layer) }];
    if (S.overlay && S.layer !== 'water' && hasLayer('overlay')) list.push({ id: 'overlay', url: tileURL(S.run, S.info, 'overlay') });
    map.setTiles(list);
  }

  // ----------------------------------------------------------------------------------------------
  // Drawn over the tiles (CSS pixels)
  // ----------------------------------------------------------------------------------------------
  function drawGrid(ctx, m) {
    if (!S.grid || !S.info) return;
    const c = C(), v = m.view;
    const left = v.cx - v.w / 2 * v.bpp;
    const degPerBlock = proj.dLonOfDx(c, 1, 0);
    const stepDeg = MV.niceStep(Math.max(v.w, v.h) * v.bpp * degPerBlock / 8);
    const steps = [1, 2, 5, 10, 15, 30, 45, 90];
    const step = steps.find(s => s >= stepDeg) || 90;
    const px = n => Math.round(n * v.dpr) / v.dpr + 0.5 / v.dpr;   // crisp one-pixel lines
    ctx.lineWidth = 1 / v.dpr; ctx.font = '11px ui-monospace, monospace'; ctx.textBaseline = 'top';
    ctx.strokeStyle = 'rgba(255,255,255,.28)'; ctx.fillStyle = 'rgba(255,255,255,.8)';
    for (let lat = -90; lat <= 90; lat += step) {
      const [, y] = m.toScreen(0, proj.zOfLat(c, lat));
      if (y < 0 || y > v.h) continue;
      ctx.beginPath(); ctx.moveTo(0, px(y)); ctx.lineTo(v.w, px(y)); ctx.stroke();
      ctx.fillText(`${Math.abs(lat)}°${lat < 0 ? 'S' : lat > 0 ? 'N' : ''}`, 56, y + 2);
    }
    const lonSpan = proj.dxOfDLon(c, step, 0);   // blocks between meridians
    for (let k = Math.floor((left + c / 2) / lonSpan) - 1; k <= Math.ceil((left + v.w * v.bpp + c / 2) / lonSpan) + 1; k++) {
      const [x] = m.toScreen(proj.xOfLon(c, k * step), 0);
      if (x < 0 || x > v.w) continue;
      const lon = ((k * step) % 360 + 360) % 360;
      const seam = lon === 0;
      ctx.strokeStyle = seam ? 'rgba(255,200,90,.7)' : 'rgba(255,255,255,.28)';
      ctx.beginPath(); ctx.moveTo(px(x), 0); ctx.lineTo(px(x), v.h); ctx.stroke();
      ctx.fillText(seam ? 'seam 0°' : `${lon}°E`, x + 3, v.h - 60);
    }
  }

  // The 256 × 256 block square a 3D preview would cover, in the middle of the view.
  function previewBox() {
    const v = map.view;
    const hw = Math.min(128, v.w * v.bpp / 2), hh = Math.min(128, v.h * v.bpp / 2);
    const x0 = Math.round(v.cx - hw), z0 = Math.round(v.cz - hh);
    const x1 = Math.round(v.cx + hw), z1 = Math.round(v.cz + hh);
    const lim = zLim();
    const zz0 = clamp(z0, -lim, lim - 1), zz1 = clamp(z1, zz0 + 1, lim);
    return { x0, z0: zz0, x1: Math.max(x1, x0 + 1), z1: zz1 };
  }
  function drawPreviewBox(ctx, m) {
    if (!S.open || !S.info) return;
    const b = previewBox();
    const [x0, y0] = m.toScreen(b.x0, b.z0), [x1, y1] = m.toScreen(b.x1, b.z1);
    if (x1 - x0 < 24) return;
    ctx.setLineDash([6, 4]); ctx.lineWidth = 1.5;
    ctx.strokeStyle = 'rgba(0,0,0,.65)'; ctx.strokeRect(x0 + 1 / m.view.dpr, y0 + 1 / m.view.dpr, x1 - x0, y1 - y0);
    ctx.strokeStyle = 'rgba(255,255,255,.95)'; ctx.strokeRect(x0, y0, x1 - x0, y1 - y0);
  }

  // ----------------------------------------------------------------------------------------------
  // The map
  // ----------------------------------------------------------------------------------------------
  const map = MV.createMap({
    canvas,
    background: '#04060a',
    minBpp: 1 / MAG,
    start: false,
    overlays: [drawGrid, drawPreviewBox],
    onView: v => {
      if (S.run && S.info) S.views.set(S.run.id, { cx: v.cx, cz: v.cz, bpp: v.bpp });
      syncLink();
      readout();
    },
    onHover: () => { readout(); askSample(); },
    onKey: ev => {
      if (ev.key !== 'Escape') return false;
      setOpen(false);
      return true;
    },
  });
  const home = () => map.home();

  // ----------------------------------------------------------------------------------------------
  // The readout
  // ----------------------------------------------------------------------------------------------
  // The block under the cursor, folded onto the planet, or null.
  function cursorBlock() {
    const p = map.cursor;
    if (!p || !S.info) return null;
    const [bx, bz] = map.toWorld(p[0], p[1]);
    const c = C();
    return { bx, bz, x: Math.floor(((bx + c / 2) % c + c) % c - c / 2), z: Math.floor(bz) };
  }
  function readout() {
    const el = $('wv-readout');
    if (!S.info) return;
    const z = map.tileZoom() || 0;
    const bpp = map.view.bpp;
    const scaleText = bpp >= 1 ? `${bpp >= 10 ? bpp.toFixed(0) : bpp.toFixed(1)} blocks a pixel` : `${(1 / bpp).toFixed(1)} pixels a block`;
    const at = cursorBlock();
    if (!at) { el.innerHTML = `<span class="dim">${esc(scaleText)} · zoom ${z} of ${S.info.max_zoom}</span>`; return; }
    const [lat, lon] = Wp().latLon(at.bx, at.bz);
    let h = `x <b>${fmtInt(at.x)}</b> z <b>${fmtInt(at.z)}</b><span class="dim"> · </span><b>${esc(latText(lat))}</b> ${esc(lonText(lon))}`;
    const s = S.sample && S.sample.x === at.x && S.sample.z === at.z ? S.sample : null;
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
    if (!map.cursor || !S.run || !S.info) return;
    S.sampleTimer = setTimeout(async () => {
      const at = cursorBlock();
      if (!at) return;
      const seq = ++S.sampleSeq;
      const res = await DT.api(`api/runs/${encodeURIComponent(S.run.id)}/world/sample?x=${at.x}&z=${at.z}`);
      if (seq !== S.sampleSeq || !res.ok) return;
      S.sample = res.body; readout();
    }, 140);
  }

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
    buildLayers(); setTiles(); syncLink();
  });
  $('wv-ovl').addEventListener('click', () => { S.overlay = !S.overlay; buildLayers(); setTiles(); });
  $('wv-grid').addEventListener('click', () => { S.grid = !S.grid; buildLayers(); map.invalidate(); });
  $('wv-in').addEventListener('click', () => map.zoomAt(map.view.w / 2, map.view.h / 2, 2));
  $('wv-out').addEventListener('click', () => map.zoomAt(map.view.w / 2, map.view.h / 2, 0.5));
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
    if (Wp().kind !== 'lambert') {
      // A block box of a non-Lambert world is no lat/lon box: its corners' latitudes and
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
    map.setTiles([]);
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
    // The view this run was left at, read before the new frame re-clamps the current view (which
    // reports through onView and would overwrite it).
    const v = S.views.get(run.id);
    const f = frameOf(S.info);
    map.setProjection(f.projection, f.pyramid, null);
    if (!v && deepPending) { S.layer = deepPending.layer; buildLayers(); }
    setTiles();
    if (v) map.setView(v.cx, v.cz, v.bpp);
    else if (deepPending) map.setView(deepPending.cx, deepPending.cz, deepPending.bpp);
    else home();
    deepPending = null;
    canvas.focus({ preventScroll: true });
    readout();
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
      map.start();
      map.invalidate();
    } else {
      map.stop();
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
