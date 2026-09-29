// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time live page (`deeptime watch`; decisions log 2026-09-28): follows a run while it runs on
// the Mac. The watch server streams over SSE (`stream`): `hello` (everything so far), then
// `frame` (one per step: n, t, stage, w, h, layers, ramps), `events`, `progress`, `status` and
// `bye`. A frame's layer image is `frames/<run>/<n>/<layer index>.png`, the core's keyframe
// coloured with ramps.json, exactly what the static viewer's timelapse shows. Layers are whatever
// the frames name, so a new stage's layers appear here without a change. Plain JS, no build step.
// `?tv` = the TV look (dark, larger type, spinning globe, hidden idle cursor); `?layer=<name>`.
'use strict';

(function () {
  const Q = new URLSearchParams(location.search);
  const TV = Q.has('tv');
  const root = document.documentElement;
  if (TV) root.classList.add('tv');
  const $ = id => document.getElementById(id);
  const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const shortKind = k => String(k).replace(/^deeptime:/, '');
  const D2R = Math.PI / 180;

  // Event colours by kind (EVENTS.md); a kind not listed gets a colour from its name.
  const KIND_COLOURS = {
    collision: '#e6194b', orogeny_phase: '#b5651d', subduction_onset: '#b04adf', ridge_birth: '#4363d8',
    rift_onset: '#f58231', terrane_accretion: '#bfef45', hotspot_track: '#ff5a1f',
    large_igneous_province: '#ff1f1f', plate_birth: '#3cb44b', plate_death: '#a9a9a9',
    erosion_pulse: '#ffe119', basin_subsidence: '#42d4f4', marine_transgression: '#1e90ff',
    marine_regression: '#9ecae1', glaciation: '#e8f4ff', deposit_formation: '#ffd700',
  };
  function kindColour(kind) {
    const k = shortKind(kind);
    if (KIND_COLOURS[k]) return KIND_COLOURS[k];
    let h = 0;
    for (let i = 0; i < k.length; i++) h = (h * 31 + k.charCodeAt(i)) >>> 0;
    return `hsl(${h % 360}, 70%, 60%)`;
  }

  // ---- state -------------------------------------------------------------------------------------
  let meta = null, run = null, ramps = null, progress = null, status = null;
  const frames = [];            // by n: {n, t, stage, w, h, layers, ramps}
  const events = [];            // in emission order
  const byId = new Map(), effects = new Map();
  const layerOrder = [];        // every layer a frame named, in order of appearance
  let layer = Q.get('layer');
  let cur = -1;                 // the frame on screen
  let mode = 'live';            // 'live' (the newest frame), 'play' (at `rate`) or 'pause'
  let tp = 0;                   // the playhead while playing, years
  let rate = Number($('rate').value);
  let tickerDirty = true, globeDirty = true, mapDirty = false;

  function fmtT(t) {
    if (t === null || t === undefined || !isFinite(t)) return '';
    if (t === 0) return 'the present';
    const a = Math.abs(t);
    const s = a >= 1e6 ? `${(a / 1e6).toFixed(1)} Myr` : a >= 1e3 ? `${(a / 1e3).toFixed(1)} kyr` : `${a.toFixed(0)} yr`;
    return t < 0 ? `${s} ago` : `+${s}`;
  }
  function fmtDur(ms) {
    if (ms === null || ms === undefined || ms < 0) return '—';
    const s = Math.round(ms / 1000), m = Math.floor(s / 60);
    return m >= 60 ? `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}` : `${m}:${String(s % 60).padStart(2, '0')}`;
  }
  const fmtBytes = b => b >= 1 << 30 ? `${(b / (1 << 30)).toFixed(2)} GiB` : `${(b / (1 << 20)).toFixed(0)} MiB`;
  const finished = () => !!status && (status.state === 'done' || status.state === 'failed');
  const tNow = () => cur >= 0 ? frames[cur].t : -Infinity;
  const tPrev = () => cur > 0 ? frames[cur - 1].t : -Infinity;

  // ---- the stream --------------------------------------------------------------------------------
  function setConn(ok, text) {
    const c = $('conn');
    c.className = ok ? 'ok' : '';
    c.textContent = text;
  }

  function reset() {
    frames.length = 0; events.length = 0; byId.clear(); effects.clear(); layerOrder.length = 0;
    images.clear(); failed.clear(); cur = -1; drawnImg = null; texImg = null; G.tex = null;
    $('layers').textContent = '';
    mode = 'live';
  }

  function hello(h) {
    if (run !== null && h.run !== run) reset();   // a new run on the same port
    run = h.run;
    meta = h.meta;
    document.title = meta.title;
    $('title').textContent = meta.title;
    const bits = [meta.preset];
    if (meta.start) bits.push(`start ${meta.start}`);
    if (meta.length_myr) bits.push(`${meta.length_myr} Myr`);
    bits.push(`C = ${meta.circumference} blocks`, `stages ${meta.stages.join(' → ')}`,
      meta.frame_every_myr ? `a frame every ${meta.frame_every_myr} Myr` : 'a frame every step');
    $('meta').textContent = bits.join(' · ');
    addFrames(h.frames);
    events.length = 0; byId.clear(); effects.clear();
    addEvents(h.events);
    setProgress(h.progress);
    setStatus(h.status);
    runInfo();
  }

  function addFrames(list) {
    let newLayer = false;
    for (const f of list) {
      frames[f.n] = f;
      for (const name of f.layers) if (!layerOrder.includes(name)) { layerOrder.push(name); newLayer = true; }
    }
    if (!layer && layerOrder.length) layer = layerOrder[0];   // a frame without `layer` shows its first
    if (newLayer) layerButtons();
    const s = $('scrub');
    s.max = Math.max(0, frames.length - 1);
    $('waiting').hidden = frames.length > 0;
    if (mode === 'live' && frames.length) imageFor(frames.length - 1);   // start loading the newest now
    if (cur < 0 && frames.length) show(frames.length - 1);
    label();
  }

  function addEvents(list) {
    for (const e of list) {
      if (byId.has(e.id)) continue;
      events.push(e);
      byId.set(e.id, e);
      for (const c of e.causes || []) {
        if (!effects.has(c)) effects.set(c, []);
        effects.get(c).push(e.id);
      }
    }
    tickerDirty = true;
    globeDirty = true;
    mapDirty = true;
  }

  function setProgress(p) {
    progress = p;
    header();
  }

  function setStatus(s) {
    status = s;
    header();
    runInfo();
    buttons();
  }

  function connect() {
    const es = new EventSource('stream');
    es.addEventListener('hello', ev => { hello(JSON.parse(ev.data)); setConn(true, finished() ? 'connected · run finished' : 'connected · live'); });
    es.addEventListener('frame', ev => addFrames([JSON.parse(ev.data)]));
    es.addEventListener('events', ev => addEvents(JSON.parse(ev.data).events));
    es.addEventListener('progress', ev => setProgress(JSON.parse(ev.data)));
    es.addEventListener('status', ev => {
      setStatus(JSON.parse(ev.data));
      if (finished()) setConn(true, 'connected · run finished');
    });
    es.addEventListener('bye', () => { es.close(); setConn(false, 'the watch server stopped'); });
    es.onerror = () => { if (es.readyState !== EventSource.CLOSED) setConn(false, 'reconnecting…'); };
  }

  // ---- frame images: an LRU of <img>, loaded on demand ---------------------------------------
  const images = new Map();     // "n/layer index" -> Image (Map order = least recently used first)
  const failed = new Map();     // key -> time of the failure (retried after 2 s)
  const MAX_IMAGES = 160;
  const layerIndex = f => Math.max(0, f.layers.indexOf(layer));
  const ready = img => !!img && img.complete && img.naturalWidth > 0;
  function imageFor(n) {
    const f = frames[n];
    if (!f || run === null) return null;
    const li = layerIndex(f);
    const key = `${n}/${li}`;
    let img = images.get(key);
    if (img) {
      images.delete(key);
      images.set(key, img);
      return img;
    }
    if (performance.now() - (failed.get(key) || -1e9) < 2000) return null;
    img = new Image();
    img.decoding = 'async';
    img.onload = () => { if (n === cur) draw(); };
    img.onerror = () => { images.delete(key); failed.set(key, performance.now()); };
    img.src = `frames/${run}/${n}/${li}.png`;
    images.set(key, img);
    while (images.size > MAX_IMAGES) images.delete(images.keys().next().value);
    return img;
  }

  // ---- playback ------------------------------------------------------------------------------------
  function frameAt(t) {   // the last frame at or before t
    let lo = 0, hi = frames.length - 1;
    if (hi < 0 || t < frames[0].t) return 0;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (frames[mid].t <= t) lo = mid; else hi = mid - 1;
    }
    return lo;
  }

  function show(n) {
    if (n < 0 || n >= frames.length) return;
    if (n !== cur) { tickerDirty = true; globeDirty = true; }
    cur = n;
    draw();
    label();
  }

  function setMode(m) {
    mode = m;
    if (cur >= 0) tp = frames[cur].t;
    buttons();
  }

  let lastTs = null;
  function tick(ts) {
    const dt = lastTs === null ? 0 : Math.min(0.25, (ts - lastTs) / 1000);
    lastTs = ts;
    const last = frames.length - 1;
    if (last >= 0) {
      let want = cur;
      if (mode === 'live') want = last;
      else if (mode === 'play') {
        tp += rate * 1e6 * dt;
        want = Math.max(cur, frameAt(tp));
        if (want >= last && tp >= frames[last].t) {
          want = last;
          mode = finished() ? 'pause' : 'live';
          buttons();
        }
        for (let k = 1; k <= 4 && want + k <= last; k++) imageFor(want + k);   // prefetch
      }
      if (want !== cur) {
        // Switch only once the image is here, so the map never goes blank; hold the playhead.
        if (ready(imageFor(want))) show(want);
        else if (mode === 'play') tp = Math.min(tp, frames[want].t);
      }
    }
    if (mapDirty) { mapDirty = false; draw(); }
    if (tickerDirty) { tickerDirty = false; ticker(); }
    spin(dt);
    requestAnimationFrame(tick);
  }

  // ---- the map -------------------------------------------------------------------------------------
  const map = $('map'), mctx = map.getContext('2d');
  let drawnImg = null;
  const FADE_YEARS = 40e6;

  function placeLatLon(e) {
    const p = e.place || {};
    return p.type === 'point' ? [p.lat, p.lon] : null;
  }
  function mapXY(lat, lon, f, W) {   // keyframe pixel px is centred on lon 360°·px/w, py on lat 90° − 180°·py/h
    const s = W / f.w;
    const px = ((lon % 360) + 360) % 360 / 360 * f.w, py = (90 - lat) / 180 * f.h;
    return [(px + 0.5) * s, (py + 0.5) * s];
  }
  // Events on screen at time t: begun by t and ended (or still going) within the fade window.
  function visibleEvents(t) {
    const out = [];
    for (const e of events) {
      if (e.t_start > t) continue;
      const ll = placeLatLon(e);
      if (!ll) continue;
      const end = Math.min(Math.max(e.t_end ?? e.t_start, e.t_start), t);
      const age = t - end;
      if (age <= FADE_YEARS) out.push({ e, ll, age });
    }
    return out;
  }

  function draw() {
    const dpr = window.devicePixelRatio || 1;
    map.width = Math.max(1, Math.round(map.clientWidth * dpr));
    map.height = Math.max(1, Math.round(map.clientHeight * dpr));
    mctx.imageSmoothingEnabled = false;
    mctx.fillStyle = '#000';
    mctx.fillRect(0, 0, map.width, map.height);
    if (cur < 0) return;
    const f = frames[cur];
    const img = imageFor(cur);
    if (ready(img)) drawnImg = img;
    if (drawnImg) mctx.drawImage(drawnImg, 0, 0, map.width, map.height);
    if (drawnImg !== texImg && ready(drawnImg)) { texImg = drawnImg; G.tex = pixels(drawnImg); globeDirty = true; }
    if ($('markers').checked) {
      const t = tNow(), t0 = tPrev(), r = 3.5 * dpr;
      const vis = visibleEvents(t);
      for (const { e, ll, age } of vis) {
        const fresh = e.t_start > t0;
        if (fresh) continue;
        const [x, y] = mapXY(ll[0], ll[1], f, map.width);
        mctx.globalAlpha = 1 - 0.75 * age / FADE_YEARS;
        mctx.beginPath(); mctx.arc(x, y, r, 0, 2 * Math.PI);
        mctx.fillStyle = kindColour(e.kind); mctx.fill();
        mctx.lineWidth = dpr; mctx.strokeStyle = '#000'; mctx.stroke();
      }
      mctx.globalAlpha = 1;
      for (const { e, ll } of vis) {
        if (!(e.t_start > t0)) continue;
        const [x, y] = mapXY(ll[0], ll[1], f, map.width);
        mctx.beginPath(); mctx.arc(x, y, 1.9 * r, 0, 2 * Math.PI);
        mctx.fillStyle = kindColour(e.kind); mctx.fill();
        mctx.lineWidth = 2 * dpr; mctx.strokeStyle = '#fff'; mctx.stroke();
        mctx.beginPath(); mctx.arc(x, y, 3.2 * r, 0, 2 * Math.PI);
        mctx.lineWidth = dpr; mctx.strokeStyle = 'rgba(255,255,255,0.6)'; mctx.stroke();
      }
    }
    $('clock').innerHTML = `${esc(fmtT(f.t))}<small>${esc(f.stage)} · step ${f.n + 1}</small>`;
    legend();
  }

  const off = document.createElement('canvas');
  function pixels(img) {
    off.width = img.naturalWidth;
    off.height = img.naturalHeight;
    const o = off.getContext('2d', { willReadFrequently: true });
    o.clearRect(0, 0, off.width, off.height);
    o.drawImage(img, 0, 0);
    return { w: off.width, h: off.height, d: o.getImageData(0, 0, off.width, off.height).data };
  }

  map.addEventListener('click', ev => {
    if (cur < 0) return;
    const f = frames[cur], rect = map.getBoundingClientRect(), dpr = window.devicePixelRatio || 1;
    const x = (ev.clientX - rect.left) * dpr, y = (ev.clientY - rect.top) * dpr;
    let best = null, bd = (12 * dpr) ** 2;
    if ($('markers').checked) {
      for (const { e, ll } of visibleEvents(tNow())) {
        const [ex, ey] = mapXY(ll[0], ll[1], f, map.width);
        const d = (ex - x) ** 2 + (ey - y) ** 2;
        if (d < bd) { bd = d; best = e; }
      }
    }
    if (best) showEvent(best.id);
    else {   // centre the globe on the clicked point
      G.lon0 = x / map.width * 360;
      G.lat0 = Math.max(-80, Math.min(80, 90 - y / map.height * 180));
      G.spin = false;
      globeDirty = true;
    }
  });

  // ---- the globe: an orthographic view of the same frame, on the CPU ---------------------------
  const globe = $('globe'), gctx = globe.getContext('2d');
  const G = { lon0: 0, lat0: 20, spin: TV || Q.has('spin'), speed: 6, tex: null };
  let texImg = null, gpre = null, gbuf = null, lastGlobe = 0;

  function globePre(S) {
    const n = S * S, xs = new Float32Array(n), ys = new Float32Array(n), zs = new Float32Array(n), inside = new Uint8Array(n);
    const r = S / 2 - 2;
    for (let j = 0; j < S; j++) {
      for (let i = 0; i < S; i++) {
        const x = (i + 0.5 - S / 2) / r, y = (S / 2 - j - 0.5) / r, d = x * x + y * y, k = j * S + i;
        if (d <= 1) { inside[k] = 1; xs[k] = x; ys[k] = y; zs[k] = Math.sqrt(1 - d); }
      }
    }
    return { S, r, xs, ys, zs, inside };
  }

  function globeXY(lat, lon) {
    const S = globe.width, r = S / 2 - 2;
    const la = lat * D2R, dl = (lon - G.lon0) * D2R, cl = Math.cos(G.lat0 * D2R), sl = Math.sin(G.lat0 * D2R);
    const X = Math.cos(la) * Math.sin(dl), Y0 = Math.sin(la), Z0 = Math.cos(la) * Math.cos(dl);
    const y = Y0 * cl - Z0 * sl, z = Y0 * sl + Z0 * cl;
    return z > 0.02 ? [S / 2 + X * r, S / 2 - y * r] : null;
  }

  function renderGlobe() {
    const S = globe.width;
    if (!gpre || gpre.S !== S) { gpre = globePre(S); gbuf = gctx.createImageData(S, S); }
    const out = gbuf.data, tex = G.tex;
    out.fill(0);
    const cl = Math.cos(G.lat0 * D2R), sl = Math.sin(G.lat0 * D2R), turn = G.lon0 / 360;
    const { xs, ys, zs, inside } = gpre;
    for (let k = 0, n = S * S; k < n; k++) {
      if (!inside[k]) continue;
      const x = xs[k], y = ys[k], z = zs[k];
      const yy = y * cl + z * sl, zz = z * cl - y * sl;
      const shade = 0.42 + 0.58 * z;
      let R = 34, Gn = 36, B = 44;
      if (tex) {
        let u = turn + Math.atan2(x, zz) / (2 * Math.PI);
        u -= Math.floor(u);
        const lat = Math.asin(yy < -1 ? -1 : yy > 1 ? 1 : yy) / D2R;
        const px = Math.round(u * tex.w) % tex.w;
        const py = Math.min(tex.h - 1, Math.max(0, Math.round((90 - lat) / 180 * tex.h)));
        const q = (py * tex.w + px) * 4;
        if (tex.d[q + 3] > 0) { R = tex.d[q]; Gn = tex.d[q + 1]; B = tex.d[q + 2]; }
      }
      const o = k * 4;
      out[o] = R * shade; out[o + 1] = Gn * shade; out[o + 2] = B * shade; out[o + 3] = 255;
    }
    gctx.putImageData(gbuf, 0, 0);
    const r = S / 2 - 2;
    gctx.beginPath(); gctx.arc(S / 2, S / 2, r, 0, 2 * Math.PI);
    gctx.lineWidth = 2; gctx.strokeStyle = 'rgba(140,180,255,0.55)'; gctx.stroke();
    if ($('markers').checked && cur >= 0) {
      const t = tNow(), t0 = tPrev();
      for (const { e, ll, age } of visibleEvents(t)) {
        const p = globeXY(ll[0], ll[1]);
        if (!p) continue;
        const fresh = e.t_start > t0;
        gctx.globalAlpha = fresh ? 1 : 1 - 0.75 * age / FADE_YEARS;
        gctx.beginPath(); gctx.arc(p[0], p[1], fresh ? 6 : 3.5, 0, 2 * Math.PI);
        gctx.fillStyle = kindColour(e.kind); gctx.fill();
        gctx.lineWidth = fresh ? 2 : 1; gctx.strokeStyle = fresh ? '#fff' : '#000'; gctx.stroke();
      }
      gctx.globalAlpha = 1;
    }
  }

  function spin(dt) {
    if (G.spin && dt > 0) { G.lon0 = (G.lon0 + G.speed * dt) % 360; globeDirty = true; }
    const now = performance.now();
    if (globeDirty && now - lastGlobe > 60) { globeDirty = false; lastGlobe = now; renderGlobe(); }
  }

  let gdrag = null;
  globe.addEventListener('pointerdown', e => { gdrag = { x: e.clientX, y: e.clientY }; G.spin = false; globe.setPointerCapture(e.pointerId); });
  globe.addEventListener('pointermove', e => {
    if (!gdrag) return;
    const k = 180 / Math.max(1, globe.clientWidth);
    G.lon0 = ((G.lon0 - (e.clientX - gdrag.x) * k) % 360 + 360) % 360;
    G.lat0 = Math.max(-89, Math.min(89, G.lat0 + (e.clientY - gdrag.y) * k));
    gdrag = { x: e.clientX, y: e.clientY };
    globeDirty = true;
  });
  globe.addEventListener('pointerup', () => { gdrag = null; });
  globe.addEventListener('dblclick', () => { G.spin = !G.spin; });

  // ---- legend, layers, labels ---------------------------------------------------------------------
  function legend() {
    if (!ramps || cur < 0) return;
    const f = frames[cur], li = layerIndex(f);
    const name = f.ramps && f.ramps[li], ramp = name && ramps.ramps[name];
    $('lname').textContent = f.layers[li] || '';
    if (!ramp) return;
    const bar = $('legendbar'), lc = bar.getContext('2d');
    const stops = ramp.stops, lo = stops[0][0], hi = stops[stops.length - 1][0];
    const grad = lc.createLinearGradient(0, 0, bar.width, 0);
    for (const [v, c] of stops) grad.addColorStop((v - lo) / (hi - lo || 1), c);
    lc.fillStyle = grad; lc.fillRect(0, 0, bar.width, bar.height);
    $('lmin').textContent = lo;
    $('lmax').textContent = hi;
    $('lunits').textContent = name;
    $('legendnote').textContent = ramp._comment || '';
  }

  function layerButtons() {
    const box = $('layers');
    box.textContent = '';
    layerOrder.forEach((name, i) => {
      const b = document.createElement('button');
      b.innerHTML = (i < 9 ? `<kbd>${i + 1}</kbd>` : '') + esc(name);
      b.dataset.layer = name;
      b.className = name === layer ? 'on' : '';
      b.addEventListener('click', () => setLayer(name));
      box.appendChild(b);
    });
  }
  function setLayer(name) {
    layer = name;
    for (const b of $('layers').children) b.classList.toggle('on', b.dataset.layer === name);
    const img = imageFor(cur);
    if (ready(img)) draw(); else if (img) img.addEventListener('load', draw, { once: true });
  }

  function label() {
    const s = $('scrub');
    if (cur >= 0) s.value = cur;
    const f = frames[cur];
    const live = frames.length ? frames[frames.length - 1] : null;
    $('tlabel').textContent = f
      ? `step ${cur + 1}/${frames.length} · ${fmtT(f.t)}${live && live !== f && !finished() ? ` · the run is at ${fmtT(live.t)}` : ''}`
      : '';
    badge();
  }

  function badge() {
    const b = $('badge');
    b.hidden = cur < 0;
    b.className = '';
    if (mode === 'live' && !finished()) { b.textContent = 'LIVE'; b.className = 'live'; }
    else if (mode === 'live') b.textContent = 'END OF RUN';
    else if (mode === 'play') b.textContent = `REPLAY · ${rate} Myr/s`;
    else b.textContent = 'PAUSED';
  }

  function buttons() {
    $('play').textContent = mode === 'play' ? '❚❚' : '▶︎';
    $('live').classList.toggle('on', mode === 'live');
    badge();
  }

  function header() {
    const bar = $('bar'), text = $('progtext');
    const st = status ? status.state : 'running';
    bar.className = st === 'done' ? 'done' : st === 'failed' ? 'failed' : '';
    const p = progress;
    if (!p) return;
    bar.firstElementChild.style.width = `${(st === 'done' ? 1000 : p.total_permille) / 10}%`;
    const parts = [];
    if (st === 'done') parts.push(`finished in ${fmtDur(p.elapsed_ms)}`);
    else if (st === 'failed') parts.push(`FAILED: ${status.message || ''}`);
    else if (st === 'finalizing') parts.push('writing the world data…');
    else {
      parts.push(`${p.stage || '—'} ${(p.total_permille / 10).toFixed(1)} %`, `sim ${fmtT(p.t_years)}`,
        `${fmtDur(p.elapsed_ms)} elapsed`, `ETA ${fmtDur(p.eta_ms)}`);
    }
    parts.push(`${p.keyframes} frames`, `${p.events} events`, `peak ${fmtBytes(p.bytes_peak)}`);
    text.textContent = parts.join(' · ');
  }

  function runInfo() {
    if (!meta) return;
    const rows = [
      ['preset', esc(meta.preset) + (meta.preset_arg && meta.preset_arg !== meta.preset ? ` <span class="hint">(${esc(meta.preset_arg)})</span>` : '')],
      ['start', esc(meta.start || '—')], ['length', esc(meta.length_myr ? `${meta.length_myr} Myr` : '—')],
      ['stages', esc(meta.stages.join(' → '))], ['circumference', esc(`${meta.circumference} blocks`)],
      ['threads', esc(meta.threads)], ['started', esc(meta.started_utc)], ['core', esc(meta.version)],
      ['run id', esc(run)],
    ];
    if (status && status.world) rows.push(['world data', esc(status.world)]);
    let h = '<table class="kv">' + rows.map(([a, b]) => `<tr><td>${esc(a)}</td><td>${b}</td></tr>`).join('') + '</table>';
    if (status && status.lines && status.lines.length) h += `<pre>${esc(status.lines.join('\n'))}</pre>`;
    $('runinfo').innerHTML = h;
  }

  // ---- events: the ticker and the details panel --------------------------------------------------
  function ticker() {
    const box = $('ticker');
    const t = tNow(), t0 = tPrev();
    const upto = events.filter(e => e.t_start <= t);
    $('evcount').textContent = events.length ? `${upto.length} of ${events.length}` : '';
    const shown = upto.slice(-150).reverse();
    if (!shown.length) { box.className = 'hint'; box.textContent = 'No events yet.'; return; }
    box.className = '';
    box.innerHTML = shown.map(e =>
      `<div class="row${e.t_start > t0 ? ' new' : ''}" data-ev="${e.id}"><span class="t">${esc(fmtT(e.t_start))}</span>` +
      `<span class="dot" style="background:${kindColour(e.kind)}"></span><span>${esc(shortKind(e.kind))}</span></div>`).join('');
  }

  function evLink(id) {
    const e = byId.get(id);
    const w = Math.floor(id / 2 ** 48), seq = id - w * 2 ** 48;
    return e ? `<a class="ref" data-ev="${id}">${esc(shortKind(e.kind))} ${w}:${seq}</a>` : `${w}:${seq} (not in this log)`;
  }
  function placeText(p) {
    if (!p) return '—';
    if (p.type === 'point') return `${Math.abs(p.lat).toFixed(2)}° ${p.lat >= 0 ? 'N' : 'S'}, longitude ${p.lon.toFixed(2)}°`;
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
      ['event', evLink(e.id)], ['kind', esc(e.kind)], ['source', esc(e.source)],
      ['time', esc(e.t_end > e.t_start ? `${fmtT(e.t_start)} → ${fmtT(e.t_end)}` : fmtT(e.t_start))],
      ['place', placeText(e.place)],
      ['magnitude', esc(e.magnitude === null || e.magnitude === undefined ? '—' : Number(e.magnitude).toPrecision(3))],
    ];
    for (const [k, v] of Object.entries(e.params || {})) rows.push([k, paramText(v)]);
    rows.push(['caused by', (e.causes || []).length ? e.causes.map(evLink).join('<br>') : '—']);
    const eff = effects.get(e.id) || [];
    rows.push(['led to', eff.length ? eff.map(evLink).join('<br>') : '—']);
    $('details').innerHTML = '<table class="kv">' + rows.map(([a, b]) => `<tr><td>${esc(a)}</td><td>${b}</td></tr>`).join('') + '</table>';
    $('detailpanel').hidden = false;
    const ll = placeLatLon(e);
    if (ll) { G.lon0 = ((ll[1] % 360) + 360) % 360; G.lat0 = Math.max(-80, Math.min(80, ll[0])); G.spin = false; globeDirty = true; }
  }
  document.body.addEventListener('click', ev => {
    const el = ev.target.closest('[data-ev]');
    if (el) showEvent(Number(el.dataset.ev));
  });
  $('closedetails').addEventListener('click', () => { $('detailpanel').hidden = true; });

  // ---- controls ------------------------------------------------------------------------------------
  function step(d) {
    if (!frames.length) return;
    setMode('pause');
    show(Math.max(0, Math.min(frames.length - 1, cur + d)));
    tp = frames[cur].t;
  }
  function togglePlay() {
    if (!frames.length) return;
    if (mode === 'play') { setMode('pause'); return; }
    if (cur >= frames.length - 1 && finished()) show(0);
    setMode('play');
  }
  function rewind() {
    if (!frames.length) return;
    const was = mode;
    show(0);
    setMode(was === 'pause' ? 'pause' : 'play');
  }
  $('rewind').addEventListener('click', rewind);
  $('back').addEventListener('click', () => step(-1));
  $('fwd').addEventListener('click', () => step(1));
  $('play').addEventListener('click', togglePlay);
  $('live').addEventListener('click', () => setMode('live'));
  $('rate').addEventListener('change', () => { rate = Number($('rate').value); badge(); });
  $('scrub').addEventListener('input', () => { setMode('pause'); show(Number($('scrub').value)); tp = frames[cur].t; });
  $('markers').addEventListener('change', () => { draw(); globeDirty = true; });
  document.addEventListener('keydown', ev => {
    const k = ev.key;
    const control = ev.target.closest ? ev.target.closest('input, select, textarea, button') : null;
    if (control && !(k === ' ' && control.id === 'scrub')) return;   // a focused button takes space itself
    if (ev.metaKey || ev.ctrlKey || ev.altKey) return;
    if (k === ' ') { ev.preventDefault(); togglePlay(); }
    else if (k === 'ArrowLeft') step(-1);
    else if (k === 'ArrowRight') step(1);
    else if (k === 'Home') rewind();
    else if (k === 'End' || k === 'l') setMode('live');
    else if (k === 'g') G.spin = !G.spin;
    else if (k === 'e') { $('markers').checked = !$('markers').checked; draw(); globeDirty = true; }
    else if (k === 'f' && document.documentElement.requestFullscreen) document.documentElement.requestFullscreen().catch(() => {});
    else if (/^[1-9]$/.test(k) && layerOrder[Number(k) - 1]) setLayer(layerOrder[Number(k) - 1]);
  });
  window.addEventListener('resize', () => { draw(); globeDirty = true; });

  if (TV) {   // hide the cursor after 3 s without moving it
    let idle = null;
    const wake = () => { root.classList.remove('idle'); clearTimeout(idle); idle = setTimeout(() => root.classList.add('idle'), 3000); };
    document.addEventListener('mousemove', wake);
    wake();
  }

  fetch('ramps.json').then(r => r.json()).then(j => { ramps = j; legend(); }).catch(() => {});
  buttons();
  connect();
  requestAnimationFrame(tick);
})();
