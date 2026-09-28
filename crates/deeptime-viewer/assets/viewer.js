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
        for (let lon = -150; lon <= 180; lon += 30) vline(lon / 360 * C, 'rgba(255,255,255,0.35)', [2, 4], null);
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
      const lat = -z / (C / 4) * 90, lon = x / C * 360;
      const rows = [
        ['block x, z', `${Math.floor(x)}, ${Math.floor(z)}`],
        ['latitude', `${Math.abs(lat).toFixed(3)}° ${lat >= 0 ? 'N' : 'S'}  (north is −z)`],
        ['longitude', `${Math.abs(lon).toFixed(3)}° ${lon >= 0 ? 'E' : 'W'}  (θ = 360°·x/C; seam at ±180°)`],
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

  // ---- timelapse: keyframes, scrubber, event ticker, sites (P1-01) ----------------------------
  function timelapse() {
    const K = B.keyframes || [], E = B.events || [], S = B.sites || [];
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

    // Pixel (px, py) of a keyframe is centred on lon −180° + 360°·px/w, lat 90° − 180°·py/h.
    function toCanvas(lat, lon, f) {
      const s = canvas.width / f.w;
      const px = ((((lon + 180) % 360) + 360) % 360) / 360 * f.w;
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
        ctx.fillStyle = '#ffb000'; ctx.fill(); ctx.lineWidth = dpr; ctx.strokeStyle = '#000'; ctx.stroke();
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
      if (p.type === 'point') return `${Math.abs(p.lat).toFixed(2)}° ${p.lat >= 0 ? 'N' : 'S'}, ${Math.abs(p.lon).toFixed(2)}° ${p.lon >= 0 ? 'E' : 'W'}`;
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
      const products = s.products.map(p =>
        `${esc(p.material)} (${esc(p.role)}): ${esc((p.grade * 100).toPrecision(3))} % · ${esc(Number(p.tonnage_t).toExponential(2))} t`).join('<br>') || '—';
      const chain = whyHere(s.forming_event, 6).map(([d, c]) => `${'&nbsp;'.repeat(2 * d)}↳ ${evLink(c)}`).join('<br>');
      details.className = '';
      details.innerHTML = kv([
        ['site', esc(s.id)],
        ['kind', esc(s.kind)],
        ['type', esc(s.deposit_type)],
        ['centre', esc(`x ${s.center[0]}, y ${s.center[1]}, z ${s.center[2]}`)],
        ['lat, lon', esc(`${s.lat_lon[0].toFixed(2)}, ${s.lat_lon[1].toFixed(2)}`)],
        ['bounds', esc(`${s.aabb[0].join(', ')} … ${s.aabb[1].join(', ')}`)],
        ['products', products],
        ['parent', s.parent === null ? '—' : `<a class="ref" data-site="${s.parent}">site ${s.parent}</a>`],
        ['formed by', evLink(s.forming_event)],
        ['why here?', chain || '—'],
      ]);
    }
    const sitesBox = document.getElementById('sites');
    if (S.length) {
      sitesBox.className = '';
      sitesBox.innerHTML = S.map(s =>
        `<div class="row" data-site="${s.id}"><span class="t">#${s.id}</span><span>${esc(s.kind)} · ${esc(s.deposit_type.replace(/^deeptime:/, ''))}</span></div>`).join('');
    }
    document.getElementById('timelapse').addEventListener('click', ev => {
      const el = ev.target.closest('[data-ev],[data-site]');
      if (!el) return;
      if (el.dataset.ev !== undefined) showEvent(Number(el.dataset.ev));
      else showSite(Number(el.dataset.site));
    });

    show();
  }
})();
