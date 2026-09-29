// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time viewer (plan §9.3). v0: layer toggles, click-to-inspect, seam marker. P1-01: the
// timelapse (keyframe strip, time scrubber, event ticker, site list with the "Why here?" cause
// chain). Plain JS, no build step. bundle.js sets window.DT_BUNDLE; layers/*.js call
// DT.addValues / DT.addStrata.
'use strict';

(function () {
  const B = window.DT_BUNDLE;
  const C = B.circumference;
  const values = {};   // layer name -> {w, h, stride, view: DataView, type}
  let strata = null;
  const images = {};   // layer name -> HTMLImageElement
  // Site markers by deposit type: the same colours as ramps.json's `sites` ramp (P1-14).
  const TYPE_COLOURS = {
    'deeptime:porphyry_cu_mo_au': '#d7301f', 'deeptime:supergene_cu_blanket': '#fdae61',
    'deeptime:placer_au': '#ffd700', 'deeptime:bif_fe': '#a6611a', 'deeptime:coal_seam': '#9e9e9e',
    'deeptime:mvt_zn_pb': '#1a9850', 'deeptime:sedex_zn_pb': '#66c2a5', 'deeptime:petroleum_trap': '#8073ac',
  };

  function decode(b64) {
    const bin = atob(b64);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    return new DataView(bytes.buffer);
  }

  // ---- page header -----------------------------------------------------------------------------
  document.getElementById('title').textContent = B.title;
  document.getElementById('meta').textContent =
    `${B.source} · C = ${C} blocks · format ${B.format_version} · sea level y ${B.sea_level_y} · written ${B.created_utc}`;

  // ---- world-data layers (hidden when the bundle has none) ------------------------------
  if (B.layers.length) layerView(); else document.getElementById('layerview').hidden = true;
  timelapse();

  function layerView() {
    // lastClick and inspect are declared below; the layer scripts call these after loading.
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
  }

  // ---- column strata (P1-07): meridian cross-sections of the horizon stacks, and a legend ------
  function structure(st) {
    const box = document.getElementById('structurebody');
    document.getElementById('structure').hidden = false;
    const esc = s => String(s).replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
    const deg = (v, pos, neg) => `${Math.abs(v).toFixed(1)}°${v >= 0 ? pos : neg}`;
    let h = '';
    if (st.summary) h += `<p class="hint">${esc(st.summary)}</p>`;
    for (const s of st.sections || []) {
      h += `<figure class="xsec"><img src="${esc(s.file)}" width="${s.w}" height="${s.h}" alt="${esc(s.title)}">` +
        `<figcaption>${esc(s.title)} — lon ${s.lon.toFixed(1)}°, ${deg(s.lat0, 'N', 'S')} (left) to ${deg(s.lat1, 'N', 'S')} (right), ` +
        `${s.km.toFixed(0)} km; y ${s.y0}…${s.y1}; vertical exaggeration of the picture ×${s.vertical_exaggeration.toFixed(0)} in real metres (the Minecraft world itself is ×15)` +
        `${s.per_cell ? '; each strata cell drawn as its own stack' : '; columns as the game reads them'}</figcaption></figure>`;
    }
    if (st.legend && st.legend.length) {
      h += '<div class="legend">' + st.legend.map(l =>
        `<span><i style="background:${esc(l.colour)}"></i>${esc(l.class)}</span>`).join('') + '</div>';
    }
    box.innerHTML = h;
  }

  // ---- the deposit census (P1-14): counts per type next to the §8.6 expectation --------------
  function census(c) {
    const box = document.getElementById('censusbody');
    document.getElementById('census').hidden = false;
    const esc = s => String(s).replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
    const f = (v, d) => (typeof v === 'number') ? v.toFixed(d) : esc(v ?? '—');
    const cols = [['id', 'type', v => esc(String(v).replace(/^deeptime:/, ''))], ['sites', 'sites', v => f(v, 0)],
      ['expected_formula', 'ρ·A·F (§8.6)', v => f(v, 2)], ['expected', 'with provinces', v => f(v, 2)], ['drawn', 'drawn', v => f(v, 0)],
      ['tracts', 'tracts', v => f(v, 0)], ['permissive_km2', 'permissive km²', v => f(v, 0)], ['mean_f', 'mean F', v => f(v, 2)],
      ['buried', 'buried', v => f(v, 0)], ['truncated', 'truncated', v => f(v, 0)], ['eroded', 'eroded', v => f(v, 0)],
      ['too_deep', 'too deep', v => f(v, 0)], ['children', 'children', v => f(v, 0)], ['guaranteed', 'guaranteed', v => f(v, 0)],
      ['dropped_separation', 'dropped', v => f(v, 0)]];
    let h = `<p class="hint">Regional specialisation ${esc(c.province)} (${esc(c.provinces)} provinces of ≈${esc(c.province_km)} km) · count_mult ${esc(c.count_mult)} · ${esc(c.sites)} sites, ${esc(c.bodies)} bodies. ${esc(c.formula || '')}</p>`;
    h += '<table class="census"><tr>' + cols.map(x => `<th>${esc(x[1])}</th>`).join('') + '</tr>';
    for (const t of c.types || []) h += '<tr>' + cols.map(x => `<td>${x[2](t[x[0]])}</td>`).join('') + '</tr>';
    box.innerHTML = h + '</table>';
  }

  // ---- timelapse: keyframes, scrubber, event ticker, sites (P1-01) ----------------------------
  function timelapse() {
    const K = B.keyframes || [], E = B.events || [], S = B.sites || [];
    const R = window.DT_REPORT || null;   // P1-14: {deposits: census, bodies: [...]}
    const bodiesOf = new Map();
    for (const b of (R && R.bodies) || []) {
      if (b.site === null) continue;
      if (!bodiesOf.has(b.site)) bodiesOf.set(b.site, []);
      bodiesOf.get(b.site).push(b);
    }
    if (R && R.deposits) census(R.deposits);
    if (R && R.structure) structure(R.structure);
    if (!K.length && !E.length && !S.length) return;
    document.getElementById('timelapse').hidden = false;
    const byId = new Map(E.map(e => [e.id, e]));
    const effects = new Map();
    for (const e of E) for (const c of e.causes) {
      if (!effects.has(c)) effects.set(c, []);
      effects.get(c).push(e.id);
    }
    const canvas = document.getElementById('kf');
    const ctx = canvas.getContext('2d');
    const scrub = document.getElementById('scrub');
    const label = document.getElementById('tlabel');
    const layerSel = document.getElementById('kflayer');
    const strip = document.getElementById('strip');
    const frames = new Map();   // "<n>/<layer>" -> Image
    let idx = 0, playing = null;
    const layerNames = K.length ? K[0].layers.map(l => l.name) : [];
    let layer = layerNames[0];
    for (const n of layerNames) {
      const o = document.createElement('option');
      o.value = o.textContent = n;
      layerSel.appendChild(o);
    }
    layerSel.hidden = layerNames.length < 2;
    layerSel.addEventListener('change', () => { layer = layerSel.value; show(); });

    const tNow = () => K.length ? K[idx].t : Infinity;
    const tPrev = () => idx > 0 ? K[idx - 1].t : -Infinity;
    function fmtT(t) {
      if (!isFinite(t)) return '';
      if (t === 0) return 'the present (t = 0)';
      const a = Math.abs(t);
      const s = a >= 1e6 ? `${(a / 1e6).toFixed(1)} Myr` : a >= 1e3 ? `${(a / 1e3).toFixed(1)} kyr` : `${a.toFixed(0)} yr`;
      return t < 0 ? `${s} ago` : `+${s}`;
    }
    const shortKind = k => k.replace(/^deeptime:/, '');
    function esc(s) { return String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c])); }
    function image(n, name) {
      const key = `${n}/${name}`;
      if (!frames.has(key)) {
        const f = K[n].layers.find(l => l.name === name) || K[n].layers[0];
        const img = new Image();
        img.onload = () => { if (n === idx) draw(); };
        img.src = f.file;
        frames.set(key, img);
      }
      return frames.get(key);
    }

    // Pixel (px, py) of a keyframe is centred on lon 360°·px/w (plan §5.2: the seam is 0°) and
    // lat 90° − 180°·py/h.
    function toCanvas(lat, lon, f) {
      const s = canvas.width / f.w;
      const px = ((lon % 360) + 360) % 360 / 360 * f.w;
      const py = (90 - lat) / 180 * f.h;
      return [(px + 0.5) * s, (py + 0.5) * s];
    }
    function placeLatLon(e) {
      const p = e.place;
      if (p.type === 'point') return [p.lat, p.lon];
      if (p.type === 'site') { const s = S.find(x => x.id === p.site); return s ? s.lat_lon : null; }
      return null;
    }

    function draw() {
      const dpr = window.devicePixelRatio || 1;
      canvas.width = Math.max(1, Math.round(canvas.clientWidth * dpr));
      canvas.height = Math.max(1, Math.round(canvas.clientHeight * dpr));
      ctx.imageSmoothingEnabled = false;
      ctx.fillStyle = '#000';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      if (!K.length) return;
      const f = K[idx];
      const img = image(idx, layer);
      if (img.complete && img.naturalWidth) ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
      if (!document.getElementById('markers').checked) return;
      const t = tNow(), t0 = tPrev(), r = 4 * dpr;
      for (const e of E) {
        if (e.t_start > t || e.place.type === 'site') continue;
        const ll = placeLatLon(e);
        if (!ll) continue;
        const [x, y] = toCanvas(ll[0], ll[1], f);
        const fresh = e.t_start > t0;
        ctx.beginPath(); ctx.arc(x, y, fresh ? 1.6 * r : r, 0, 2 * Math.PI);
        ctx.fillStyle = fresh ? '#ffffff' : 'rgba(255,255,255,0.45)'; ctx.fill();
        ctx.lineWidth = dpr; ctx.strokeStyle = '#000'; ctx.stroke();
      }
      for (const s of S) {
        const fe = byId.get(s.forming_event);
        if (fe && fe.t_start > t) continue;
        const [x, y] = toCanvas(s.lat_lon[0], s.lat_lon[1], f);
        ctx.beginPath();
        ctx.moveTo(x, y - 1.8 * r); ctx.lineTo(x + 1.8 * r, y); ctx.lineTo(x, y + 1.8 * r); ctx.lineTo(x - 1.8 * r, y); ctx.closePath();
        ctx.fillStyle = TYPE_COLOURS[s.deposit_type] || '#ffb000'; ctx.fill(); ctx.lineWidth = dpr; ctx.strokeStyle = '#000'; ctx.stroke();
      }
    }

    function ticker() {
      const box = document.getElementById('ticker');
      const t = tNow(), t0 = tPrev();
      const shown = E.filter(e => e.t_start <= t).slice(-80).reverse();
      if (!shown.length) { box.className = 'hint'; box.textContent = 'No events yet.'; return; }
      box.className = '';
      box.innerHTML = shown.map(e =>
        `<div class="row${e.t_start > t0 ? ' new' : ''}" data-ev="${e.id}"><span class="t">${esc(fmtT(e.t_start))}</span>` +
        `<span>${esc(shortKind(e.kind))}</span></div>`).join('');
    }

    function show() {
      if (K.length) {
        const f = K[idx];
        scrub.value = idx;
        label.textContent = `${idx + 1}/${K.length} · ${fmtT(f.t)} · ${f.stage}`;
        for (const fig of strip.children) fig.classList.toggle('on', Number(fig.dataset.n) === idx);
      }
      draw();
      ticker();
    }

    // ---- strip and scrubber --------------------------------------------------------------------
    if (K.length) {
      scrub.max = K.length - 1;
      scrub.addEventListener('input', () => { idx = Number(scrub.value); show(); });
      K.forEach((f, n) => {
        const fig = document.createElement('figure');
        fig.dataset.n = n;
        const img = document.createElement('img');
        img.loading = 'lazy';
        img.src = f.layers[0].file;
        img.alt = `keyframe ${n}`;
        const cap = document.createElement('figcaption');
        cap.textContent = fmtT(f.t);
        fig.append(img, cap);
        fig.addEventListener('click', () => { idx = n; show(); });
        strip.appendChild(fig);
      });
      const btn = document.getElementById('play');
      btn.addEventListener('click', () => {
        if (playing) { clearInterval(playing); playing = null; btn.textContent = '▶'; return; }
        if (idx === K.length - 1) idx = 0;
        btn.textContent = '❚❚';
        playing = setInterval(() => {
          if (idx >= K.length - 1) { clearInterval(playing); playing = null; btn.textContent = '▶'; return; }
          idx++; show();
        }, 400);
      });
      idx = K.length - 1;
    } else {
      document.getElementById('tlcontrols').hidden = true;
      canvas.hidden = true;
    }
    document.getElementById('markers').addEventListener('change', draw);
    window.addEventListener('resize', draw);

    // ---- details: events, sites and the "Why here?" chain ----------------------------------
    const details = document.getElementById('details');
    function evLink(id) {
      const e = byId.get(id);
      const w = Math.floor(id / 2 ** 48), seq = id - w * 2 ** 48;
      return e ? `<a class="ref" data-ev="${id}">${esc(shortKind(e.kind))} ${w}:${seq}</a>` : `${w}:${seq} (not in this log)`;
    }
    function kv(rows) {
      return '<table class="kv">' + rows.map(([a, b]) => `<tr><td>${esc(a)}</td><td>${b}</td></tr>`).join('') + '</table>';
    }
    function placeText(p) {
      if (p.type === 'point') return `${Math.abs(p.lat).toFixed(2)}° ${p.lat >= 0 ? 'N' : 'S'}, longitude ${p.lon.toFixed(2)}°`;
      if (p.type === 'site') return `<a class="ref" data-site="${p.site}">site ${p.site}</a>`;
      if (p.type === 'global') return 'global';
      return esc(`${p.type} ${p[p.type === 'region' ? 'mask' : p.type]}`);
    }
    function paramText(v) {
      if (v && typeof v === 'object' && 'id' in v) return byId.has(v.id) ? evLink(v.id) : esc(`id ${v.id}`);
      return esc(v === null ? '—' : v);
    }
    function showEvent(id) {
      const e = byId.get(id);
      if (!e) return;
      const rows = [
        ['event', evLink(e.id)],
        ['kind', esc(e.kind)],
        ['source', esc(e.source)],
        ['time', esc(e.t_end > e.t_start ? `${fmtT(e.t_start)} → ${fmtT(e.t_end)}` : fmtT(e.t_start))],
        ['place', placeText(e.place)],
        ['magnitude', esc(e.magnitude === null ? '—' : Number(e.magnitude).toPrecision(3))],
      ];
      for (const [k, v] of Object.entries(e.params)) rows.push([k, paramText(v)]);
      rows.push(['caused by', e.causes.length ? e.causes.map(evLink).join('<br>') : '—']);
      const eff = effects.get(e.id) || [];
      rows.push(['led to', eff.length ? eff.map(evLink).join('<br>') : '—']);
      details.className = '';
      details.innerHTML = kv(rows);
    }
    function whyHere(id, depth) {
      const out = [], seen = new Set([id]);
      let frontier = [id];
      for (let d = 0; d < depth && frontier.length; d++) {
        const next = [];
        for (const f of frontier) {
          const e = byId.get(f);
          if (!e) continue;
          for (const c of e.causes) if (!seen.has(c)) { seen.add(c); out.push([d + 1, c]); next.push(c); }
        }
        frontier = next;
      }
      return out;
    }
    function showSite(id) {
      const s = S.find(x => x.id === id);
      if (!s) return;
      // Grades are mass fractions: precious metals read better in g/t.
      const grade = g => g < 1e-4 ? `${(g * 1e6).toPrecision(3)} g/t` : `${(g * 100).toPrecision(3)} %`;
      const products = s.products.map(p =>
        `${esc(p.material)} (${esc(p.role)}): ${esc(grade(p.grade))} · ${esc(Number(p.tonnage_t).toExponential(2))} t`).join('<br>') ||
        (s.kind === 'deposit' ? 'none left (eroded, or leached into its supergene blanket)' : '—');
      const chain = whyHere(s.forming_event, 6).map(([d, c]) => `${'&nbsp;'.repeat(2 * d)}↳ ${evLink(c)}`).join('<br>');
      const rows = [
        ['site', esc(s.id)],
        ['kind', esc(s.kind)],
        ['type', esc(s.deposit_type)],
        ['centre', esc(`x ${s.center[0]}, y ${s.center[1]}, z ${s.center[2]}`)],
        ['lat, lon', esc(`${s.lat_lon[0].toFixed(2)}°, ${s.lat_lon[1].toFixed(2)}° (the seam is 0°)`)],
        ['bounds', esc(`${s.aabb[0].join(', ')} … ${s.aabb[1].join(', ')}`)],
        ['products', products],
        ['parent', s.parent === null ? '—' : `<a class="ref" data-site="${s.parent}">site ${s.parent}</a>`],
        ['formed by', evLink(s.forming_event)],
      ];
      // P1-14: the mineral-system scores behind the forming event ("Why here?").
      const fe = byId.get(s.forming_event);
      if (fe && fe.params && 'favourability' in fe.params) {
        const p = fe.params, pct = v => (v === null || v === undefined) ? '—' : `${Math.round(100 * v)} %`;
        rows.push(['favourability', esc(pct(p.favourability))]);
        if ('score_source' in p) {
          rows.push(['components', esc(`source ${pct(p.score_source)} · driver ${pct(p.score_driver)} · pathway ${pct(p.score_pathway)} · trap ${pct(p.score_trap)} · host ${pct(p.score_host)} · clock ${pct(p.score_clock)}`)]);
        }
        if (p.limiting) rows.push(['weakest here', esc(p.limiting)]);
        if (p.neighbours_limited_by) rows.push(['barren nearby for lack of', esc(p.neighbours_limited_by)]);
        if (p.host_class) rows.push(['host rock', esc(p.host_class)]);
        if (p.fate) rows.push(['preservation', esc(`${p.fate}${p.fraction_left !== undefined && p.fraction_left !== null ? ` (${pct(p.fraction_left)} left)` : ''}`)]);
        if (p.rank) rows.push(['coal rank', esc(p.rank)]);
      }
      rows.push(['why here?', chain || '—']);
      const bodies = bodiesOf.get(s.id) || [];
      if (bodies.length) rows.push(['bodies', bodies.map(b => esc(`${b.role}: ${b.shape} ${b.axes.map(v => v.toFixed(1)).join(' × ')} blocks`)).join('<br>') + section(bodies)]);
      details.className = '';
      details.innerHTML = kv(rows);
    }

    // A plan view and a vertical section of a site's bodies (blocks; zones as nested outlines).
    function section(bodies) {
      const W = 300, H = 150, pad = 14;
      const ore = bodies[0];
      let ext = 1, yLo = Infinity, yHi = -Infinity;
      for (const b of bodies) {
        ext = Math.max(ext, b.axes[0], b.axes[2]);
        yLo = Math.min(yLo, b.center[1] - b.axes[1]);
        yHi = Math.max(yHi, b.center[1] + b.axes[1]);
      }
      const clip = bodies.map(b => b.clip_top_y).find(v => v !== null && v !== undefined);
      if (clip !== undefined) yHi = Math.max(yHi, clip + 2);
      const half = W / 2 - pad;
      const sp = Math.min(half - pad, H / 2 - pad) / ext;     // plan scale (left half; any yaw fits)
      const ss = Math.min((half - pad) / ext, (H - 2 * pad) / Math.max(1, yHi - yLo)); // section scale
      const colours = ['#c0392b', '#e67e22', '#f1c40f', '#27ae60', '#2980b9', '#8e44ad'];
      let svg = `<svg class="section" viewBox="0 0 ${W} ${H}" role="img" aria-label="plan and section of the site's bodies">`;
      svg += `<text x="4" y="11">plan</text><text x="${W / 2 + 4}" y="11">section (x–y)</text>`;
      const pcx = W / 4, pcy = H / 2, scx = 3 * W / 4;
      const sy = y => H - pad - (y - yLo) * ss;
      bodies.forEach((b, bi) => {
        const dx = (b.center[0] - ore.center[0]), dz = (b.center[2] - ore.center[2]);
        const rot = b.yaw * 180 / Math.PI;
        const shells = [1, ...b.zones.map(z => z.d[1]).filter(d => d < 1)].sort((a, c) => c - a);
        shells.forEach((d, k) => {
          const col = colours[(bi * 2 + k) % colours.length];
          svg += `<ellipse cx="${pcx + dx * sp}" cy="${pcy + dz * sp}" rx="${Math.max(0.5, b.axes[0] * d * sp)}" ry="${Math.max(0.5, b.axes[2] * d * sp)}" transform="rotate(${rot} ${pcx + dx * sp} ${pcy + dz * sp})" fill="none" stroke="${col}" stroke-width="1"/>`;
          const x0 = scx + dx * ss, y0 = sy(b.center[1]);
          if (b.metric === 'ellipsoid') {
            svg += `<ellipse cx="${x0}" cy="${y0}" rx="${Math.max(0.5, b.axes[0] * d * ss)}" ry="${Math.max(0.5, b.axes[1] * d * ss)}" fill="none" stroke="${col}" stroke-width="1"/>`;
          } else {
            svg += `<rect x="${x0 - b.axes[0] * d * ss}" y="${y0 - b.axes[1] * d * ss}" width="${Math.max(0.5, 2 * b.axes[0] * d * ss)}" height="${Math.max(0.5, 2 * b.axes[1] * d * ss)}" fill="none" stroke="${col}" stroke-width="1"/>`;
          }
        });
      });
      if (clip !== undefined) svg += `<line x1="${W / 2 + 2}" x2="${W - 2}" y1="${sy(clip)}" y2="${sy(clip)}" stroke="#7f8c8d" stroke-dasharray="4 3"/><text x="${W - 60}" y="${sy(clip) - 3}">eroded to</text>`;
      svg += `<text x="4" y="${H - 3}">${esc(`${(2 * ext).toFixed(0)} blocks across`)}</text></svg>`;
      return svg;
    }
    const sitesBox = document.getElementById('sites');
    if (S.length) {
      sitesBox.className = '';
      const mt = s => s.products.length ? `${(s.products[0].tonnage_t / 1e6).toPrecision(2)} Mt` : '';
      sitesBox.innerHTML = S.map(s =>
        `<div class="row" data-site="${s.id}"><span class="t">#${s.id}</span><span>${esc(s.kind)} · ${esc(s.deposit_type.replace(/^deeptime:/, ''))} · ${esc(mt(s))}</span></div>`).join('');
    }
    document.getElementById('timelapse').addEventListener('click', ev => {
      const el = ev.target.closest('[data-ev],[data-site]');
      if (!el) return;
      if (el.dataset.ev !== undefined) showEvent(Number(el.dataset.ev));
      else showSite(Number(el.dataset.site));
    });

    show();
    // Review links: index.html?site=<id> or ?event=<id> opens that item's details.
    const q = new URLSearchParams(location.search);
    if (q.has('site')) showSite(Number(q.get('site')));
    else if (q.has('event')) showEvent(Number(q.get('event')));
  }
})();
