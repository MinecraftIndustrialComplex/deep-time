// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time viewer v0 (plan §9.3): layer toggles, click-to-inspect, seam marker. Plain JS, no
// build step. bundle.js sets window.DT_BUNDLE; layers/*.js call DT.addValues / DT.addStrata.
'use strict';

(function () {
  const B = window.DT_BUNDLE;
  const C = B.circumference;
  const values = {};   // layer name -> {w, h, stride, view: DataView, type}
  let strata = null;
  const images = {};   // layer name -> HTMLImageElement

  function decode(b64) {
    const bin = atob(b64);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    return new DataView(bytes.buffer);
  }

  window.DT = {
    addValues(name, v) {
      const layer = B.layers.find(l => l.name === name);
      values[name] = { w: v.w, h: v.h, stride: v.stride, view: decode(v.data), type: layer ? layer.values.type : 'i16' };
      if (lastClick) inspect(lastClick.x, lastClick.z);
    },
    addStrata(s) {
      strata = Object.assign({}, s, { topsView: decode(s.tops), classView: decode(s.class_index) });
      if (lastClick) inspect(lastClick.x, lastClick.z);
    },
  };

  // ---- page header and controls -------------------------------------------------------------
  document.getElementById('title').textContent = B.title;
  document.getElementById('meta').textContent =
    `${B.source} · C = ${C} blocks · format ${B.format_version} · sea level y ${B.sea_level_y} · written ${B.created_utc}`;

  const canvas = document.getElementById('map');
  const ctx = canvas.getContext('2d');
  const layerBox = document.getElementById('layers');
  const baseLayers = B.layers.filter(l => l.name !== 'water_level');
  const waterLayer = B.layers.find(l => l.name === 'water_level');
  let current = baseLayers.length ? baseLayers[0].name : null;

  for (const l of baseLayers) {
    const lab = document.createElement('label');
    const r = document.createElement('input');
    r.type = 'radio'; r.name = 'layer'; r.value = l.name; r.checked = l.name === current;
    r.addEventListener('change', () => { current = l.name; legend(); draw(); });
    lab.appendChild(r);
    lab.appendChild(document.createTextNode(' ' + l.name));
    layerBox.appendChild(lab);
  }
  if (!waterLayer) document.getElementById('water').parentElement.style.display = 'none';

  for (const l of B.layers) {
    const img = new Image();
    img.onload = draw;
    img.src = l.image.file;
    images[l.name] = img;
    const s = document.createElement('script');
    s.src = l.values.file;
    document.body.appendChild(s);
  }
  if (B.strata && B.strata.file) {
    const s = document.createElement('script');
    s.src = B.strata.file;
    document.body.appendChild(s);
  }

  // ---- view state: centre block (cx, cz) and k canvas pixels per block ----------------------
  let cx = 0, cz = 0, k = 1;
  let lastClick = null;

  function resize() {
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.max(1, Math.round(canvas.clientWidth * dpr));
    canvas.height = Math.max(1, Math.round(canvas.clientHeight * dpr));
  }
  function fitK() { return canvas.width / C; }
  function fit() { resize(); k = fitK(); cx = 0; cz = 0; draw(); }

  const toScreenX = x => (x - cx) * k + canvas.width / 2;
  const toScreenZ = z => (z - cz) * k + canvas.height / 2;
  const toWorldX = sx => (sx - canvas.width / 2) / k + cx;
  const toWorldZ = sz => (sz - canvas.height / 2) / k + cz;
  const wrapX = x => ((x + C / 2) % C + C) % C - C / 2;

  function drawWrapped(img, l, alpha) {
    if (!img || !img.complete || !img.naturalWidth) return;
    const g = l.grid, s = l.image.stride;
    const half = s * g.r / 2;
    const x0 = g.origin_block_x - half, z0 = g.origin_block_z - half;
    const wBlocks = l.image.w * s * g.r, hBlocks = l.image.h * s * g.r;
    const left = toWorldX(0), right = toWorldX(canvas.width);
    const nFrom = Math.floor((left - x0) / C) - 1, nTo = Math.ceil((right - x0) / C) + 1;
    ctx.globalAlpha = alpha;
    for (let n = nFrom; n <= nTo; n++) {
      const sx = toScreenX(x0 + n * C);
      ctx.drawImage(img, sx, toScreenZ(z0), wBlocks * k, hBlocks * k);
    }
    ctx.globalAlpha = 1;
  }

  function vline(x, colour, dash, label) {
    const left = toWorldX(0), right = toWorldX(canvas.width);
    for (let n = Math.floor((left - x) / C); n <= Math.ceil((right - x) / C); n++) {
      const sx = Math.round(toScreenX(x + n * C)) + 0.5;
      ctx.strokeStyle = colour; ctx.setLineDash(dash); ctx.lineWidth = Math.max(1, (window.devicePixelRatio || 1));
      ctx.beginPath(); ctx.moveTo(sx, toScreenZ(-C / 4)); ctx.lineTo(sx, toScreenZ(C / 4)); ctx.stroke();
      if (label) {
        ctx.setLineDash([]); ctx.fillStyle = colour;
        ctx.font = `${12 * (window.devicePixelRatio || 1)}px system-ui, sans-serif`;
        ctx.fillText(label, sx + 4, Math.max(14, toScreenZ(-C / 4) + 14));
      }
    }
    ctx.setLineDash([]);
  }

  function hline(z, colour) {
    const sz = Math.round(toScreenZ(z)) + 0.5;
    ctx.strokeStyle = colour; ctx.setLineDash([2, 4]); ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(0, sz); ctx.lineTo(canvas.width, sz); ctx.stroke(); ctx.setLineDash([]);
  }

  function draw() {
    if (!current) return;
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const base = B.layers.find(l => l.name === current);
    drawWrapped(images[current], base, 1);
    if (waterLayer && document.getElementById('water').checked && current !== 'water_level') {
      drawWrapped(images.water_level, waterLayer, 0.85);
    }
    if (document.getElementById('grat').checked) {
      for (let lon = 30; lon < 360; lon += 30) vline(lon / 360 * C - C / 2, 'rgba(255,255,255,0.35)', [2, 4], null);   // meridians every 30° (plan §5.2 longitude)
      for (let lat = -60; lat <= 60; lat += 30) hline(-lat / 90 * C / 4, 'rgba(255,255,255,0.35)');
    }
    if (document.getElementById('seam').checked) {
      vline(-C / 2, '#ff3b30', [8, 5], `seam x = ±${C / 2}`);
    }
    if (lastClick) {
      const sx = toScreenX(lastClick.x + C * Math.round((cx - lastClick.x) / C)), sz = toScreenZ(lastClick.z);
      ctx.strokeStyle = '#fff'; ctx.lineWidth = 2;
      ctx.beginPath(); ctx.arc(sx, sz, 6, 0, 2 * Math.PI); ctx.stroke();
      ctx.strokeStyle = '#000'; ctx.lineWidth = 1;
      ctx.beginPath(); ctx.arc(sx, sz, 7.5, 0, 2 * Math.PI); ctx.stroke();
    }
  }

  // ---- legend ------------------------------------------------------------------------------------
  function legend() {
    const l = B.layers.find(x => x.name === current);
    const ramp = B.ramps.ramps[l.ramp];
    const bar = document.getElementById('legendbar');
    const lc = bar.getContext('2d');
    const stops = ramp.stops, lo = stops[0][0], hi = stops[stops.length - 1][0];
    const grad = lc.createLinearGradient(0, 0, bar.width, 0);
    for (const [v, c] of stops) grad.addColorStop((v - lo) / (hi - lo || 1), c);
    lc.fillStyle = grad; lc.fillRect(0, 0, bar.width, bar.height);
    document.getElementById('lmin').textContent = lo;
    document.getElementById('lmax').textContent = hi;
    document.getElementById('lunits').textContent = l.units || '';
  }

  // ---- inspect -----------------------------------------------------------------------------------
  function sample(l, x, z) {
    const v = values[l.name];
    if (!v) return { text: 'loading…' };
    const g = l.grid;
    const u = (x - g.origin_block_x) / g.r / v.stride, w = (z - g.origin_block_z) / g.r / v.stride;
    const vi = ((Math.round(u) % v.w) + v.w) % v.w;
    const vj = Math.min(v.h - 1, Math.max(0, Math.round(w)));
    const idx = vj * v.w + vi;
    const raw = v.type === 'i16' ? v.view.getInt16(idx * 2, true) : v.view.getFloat32(idx * 4, true);
    const i = vi * v.stride, j = vj * v.stride;
    const cell = `${g.name} cell (${i}, ${j}) at block (${g.origin_block_x + i * g.r}, ${g.origin_block_z + j * g.r})`;
    if (l.nodata !== null && raw === l.nodata) return { text: '— (nodata)', cell };
    const phys = raw * l.scale + l.offset;
    const digits = l.scale < 0.01 ? 4 : l.scale < 1 ? 2 : 0;
    return { text: `${phys.toFixed(digits)} ${l.units}  (raw ${raw})`, cell };
  }

  function strataAt(x, z) {
    if (!strata) return null;
    const g = strata.grid, s = strata.stride;
    const vi = ((Math.round((x - g.origin_block_x) / g.r / s) % strata.w) + strata.w) % strata.w;
    const vj = Math.min(strata.h - 1, Math.max(0, Math.round((z - g.origin_block_z) / g.r / s)));
    const n = strata.w * strata.h, idx = vj * strata.w + vi;
    const parts = [];
    for (let h = 0; h < strata.k; h++) {
      const top = strata.topsView.getInt16((h * n + idx) * 2, true);
      const cls = strata.classView.getUint8(h * n + idx);
      if (top === -32768 || cls === 255) continue;
      parts.push(`${strata.classes[cls] || ('#' + cls)} from y ${(top * strata.scale).toFixed(1)}`);
    }
    return parts.join('<br>');
  }

  function esc(s) { return String(s).replace(/[&<>]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c])); }

  function inspect(x, z) {
    lastClick = { x, z };
    const lat = -z / (C / 4) * 90, lon = (x + C / 2) / C * 360;   // plan §5.2: the seam is 0°
    const rows = [
      ['block x, z', `${Math.floor(x)}, ${Math.floor(z)}`],
      ['latitude', `${Math.abs(lat).toFixed(3)}° ${lat >= 0 ? 'N' : 'S'}  (north is −z)`],
      ['longitude', `${lon.toFixed(3)}°  (360°·(x + C/2)/C; the seam x = ±C/2 is 0°)`],
    ];
    for (const l of B.layers) {
      const s = sample(l, x, z);
      rows.push([l.name, esc(s.text) + (s.cell ? `<br><span class="hint">${esc(s.cell)}</span>` : '')]);
    }
    const st = strataAt(x, z);
    if (st !== null) rows.push(['strata', st || '—']);
    document.getElementById('inspectbody').innerHTML =
      '<table>' + rows.map(([a, b]) => `<tr><td>${esc(a)}</td><td>${b}</td></tr>`).join('') + '</table>';
    draw();
  }

  // ---- interaction -------------------------------------------------------------------------------
  let drag = null;
  const dpr = () => window.devicePixelRatio || 1;
  canvas.addEventListener('pointerdown', e => {
    drag = { x: e.clientX, y: e.clientY, cx, cz, moved: false };
    canvas.setPointerCapture(e.pointerId);
  });
  canvas.addEventListener('pointermove', e => {
    if (!drag) return;
    const dx = (e.clientX - drag.x) * dpr(), dy = (e.clientY - drag.y) * dpr();
    if (Math.abs(dx) + Math.abs(dy) > 4) drag.moved = true;
    cx = wrapX(drag.cx - dx / k);
    cz = Math.max(-C / 4, Math.min(C / 4, drag.cz - dy / k));
    draw();
  });
  canvas.addEventListener('pointerup', e => {
    if (drag && !drag.moved) {
      const r = canvas.getBoundingClientRect();
      const sx = (e.clientX - r.left) * dpr(), sz = (e.clientY - r.top) * dpr();
      const x = wrapX(toWorldX(sx)), z = toWorldZ(sz);
      if (z >= -C / 4 && z < C / 4) inspect(x, z);
    }
    drag = null;
  });
  canvas.addEventListener('wheel', e => {
    e.preventDefault();
    const r = canvas.getBoundingClientRect();
    const sx = (e.clientX - r.left) * dpr(), sz = (e.clientY - r.top) * dpr();
    const wx = toWorldX(sx), wz = toWorldZ(sz);
    const nk = Math.max(fitK(), Math.min(16, k * Math.pow(1.0015, -e.deltaY)));
    cx = wrapX(wx - (sx - canvas.width / 2) / nk);
    cz = Math.max(-C / 4, Math.min(C / 4, wz - (sz - canvas.height / 2) / nk));
    k = nk;
    draw();
  }, { passive: false });
  for (const id of ['water', 'seam', 'grat']) document.getElementById(id).addEventListener('change', draw);
  document.getElementById('fit').addEventListener('click', fit);
  document.getElementById('toseam').addEventListener('click', () => { cx = -C / 2; draw(); });
  window.addEventListener('resize', () => { const f = k / fitK(); resize(); k = Math.max(fitK(), f * fitK()); draw(); });

  fit();
  legend();
})();
