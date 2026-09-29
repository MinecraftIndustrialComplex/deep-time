// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time workbench (`deeptime watch`, P1-20 stage 1; decisions log 2026-09-28). Served on the
// tailnet by the Mac. Three parts:
//
// - The editor (left): the Create World setup page's controls, built from the game's own
//   definitions through the server (`GET api/catalog`: the built-in and pipeline presets, the
//   shared editor_fields.json, the game's lang strings, the start configurations). The server
//   computes the effective preset, its share code, problems and estimate (`POST api/preset`) and
//   decodes codes (`POST api/decode`), with the core's own code; the page only collects choices.
// - The runs (left, below): `GET api/runs` (polled), one running at a time on the Mac, a short
//   queue, cancel, reopen, compare. `POST api/runs` starts one; a running simulation is never
//   changed from here.
// - The viewer (middle and right): one pane, or two in compare mode sharing the layer and the
//   time. Each pane follows its run's stream (`api/runs/<id>/stream`: `hello`, `frame`, `events`,
//   `progress`, `status`, `bye`); a frame's layer image is `api/runs/<id>/frames/<n>/<layer>.png`,
//   the core's keyframe coloured with ramps.json (the static viewer's look). Layers are whatever the
//   frames name, so a new stage's layers appear without a change here.
//
// Stage 2 (a sibling worktree builds the 3D block preview and click-to-inspect): see `Stage2`
// below, the one place that talks to it. URL: `#a=<run>&b=<run>&layer=<name>`; `?tv` = the TV look
// (no editor, dark, larger type, spinning globe, hidden idle cursor, follows the newest run).
'use strict';

(function () {
  const Q = new URLSearchParams(location.search);
  const TV = Q.has('tv');
  const root = document.documentElement;
  if (TV) root.classList.add('tv');
  const $ = id => document.getElementById(id);
  const esc = s => String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const shortKind = k => String(k).replace(/^deeptime:/, '');
  const D2R = Math.PI / 180;

  async function api(path, body) {
    const opt = body === undefined ? {} : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) };
    const r = await fetch(path, opt);
    let j = null;
    try { j = await r.json(); } catch (e) { j = { error: `${r.status} ${r.statusText}` }; }
    return { status: r.status, ok: r.ok, body: j };
  }

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
  const FINAL = ['done', 'failed', 'cancelled', 'interrupted'];

  let ramps = null;
  fetch('ramps.json').then(r => r.json()).then(j => { ramps = j; legend(); }).catch(() => {});

  // =================================================================================================
  // A run's view: its stream, frames, events and images
  // =================================================================================================
  class RunView {
    constructor(id) {
      this.id = id;
      this.meta = null; this.status = null; this.progress = null;
      this.frames = []; this.events = []; this.byId = new Map(); this.effects = new Map();
      this.layers = [];            // every layer a frame named, in order of appearance
      this.images = new Map();     // "n/li" -> Image (LRU)
      this.failed = new Map();
      this.connected = false;
      this.es = new EventSource(`api/runs/${encodeURIComponent(id)}/stream`);
      this.es.addEventListener('hello', ev => this.hello(JSON.parse(ev.data)));
      this.es.addEventListener('frame', ev => this.addFrames([JSON.parse(ev.data)]));
      this.es.addEventListener('events', ev => this.addEvents(JSON.parse(ev.data).events));
      this.es.addEventListener('progress', ev => { this.progress = JSON.parse(ev.data); changed(this); });
      this.es.addEventListener('status', ev => { this.status = JSON.parse(ev.data); changed(this); });
      this.es.addEventListener('bye', () => { this.es.close(); this.connected = false; changed(this); });
      this.es.onerror = () => { this.connected = false; changed(this); };
    }
    close() { this.es.close(); }
    finished() { return !!this.status && FINAL.includes(this.status.state); }
    hello(h) {
      this.connected = true;
      this.meta = h.meta;
      this.frames = []; this.events = []; this.byId.clear(); this.effects.clear();
      this.addFrames(h.frames, true);
      this.addEvents(h.events, true);
      this.progress = h.progress; this.status = h.status;
      changed(this, true);
    }
    addFrames(list, quiet) {
      for (const f of list) {
        this.frames[f.n] = f;
        for (const name of f.layers) if (!this.layers.includes(name)) this.layers.push(name);
      }
      if (!quiet) changed(this, true);
    }
    addEvents(list, quiet) {
      for (const e of list) {
        if (this.byId.has(e.id)) continue;
        this.events.push(e);
        this.byId.set(e.id, e);
        for (const c of e.causes || []) {
          if (!this.effects.has(c)) this.effects.set(c, []);
          this.effects.get(c).push(e.id);
        }
      }
      if (!quiet) { tickerDirty = true; globeDirty = true; mapDirty = true; }
    }
    // The last frame at or before t (the first frame for a t before the run).
    frameAt(t) {
      let lo = 0, hi = this.frames.length - 1;
      if (hi < 0 || t < this.frames[0].t) return 0;
      while (lo < hi) {
        const mid = (lo + hi + 1) >> 1;
        if (this.frames[mid].t <= t) lo = mid; else hi = mid - 1;
      }
      return lo;
    }
    layerIndex(f) { return Math.max(0, f.layers.indexOf(layer)); }
    image(n) {
      const f = this.frames[n];
      if (!f) return null;
      const li = this.layerIndex(f);
      const key = `${n}/${li}`;
      let img = this.images.get(key);
      if (img) { this.images.delete(key); this.images.set(key, img); return img; }
      if (performance.now() - (this.failed.get(key) || -1e9) < 2000) return null;
      img = new Image();
      img.decoding = 'async';
      img.onload = () => { mapDirty = true; };
      img.onerror = () => { this.images.delete(key); this.failed.set(key, performance.now()); };
      img.src = `api/runs/${encodeURIComponent(this.id)}/frames/${n}/${li}.png`;
      this.images.set(key, img);
      while (this.images.size > 120) this.images.delete(this.images.keys().next().value);
      return img;
    }
  }
  const ready = img => !!img && img.complete && img.naturalWidth > 0;

  // =================================================================================================
  // Panes and the shared playhead
  // =================================================================================================
  const panes = [0, 1].map(i => {
    const el = $(i ? 'paneB' : 'paneA');
    return {
      i, el, run: null, cur: -1, drawn: null, box: null,
      canvas: el.querySelector('.map'), ctx: el.querySelector('.map').getContext('2d'),
      clock: el.querySelector('.clock'), badge: el.querySelector('.badge'), waiting: el.querySelector('.waiting'),
      name: el.querySelector('.name'), tag: el.querySelector('.tag'),
    };
  });
  let focus = 0, comparing = false;
  let layer = null;
  let mode = 'live';           // 'live' (the newest frame), 'play' (at `rate`) or 'pause'
  let tp = 0;                  // the playhead while playing, years
  let rate = Number($('rate').value);
  let tickerDirty = true, globeDirty = true, mapDirty = true;
  let boxMode = false;
  const P = () => panes[focus];

  function hashState() {
    const h = new URLSearchParams();
    if (panes[0].run) h.set('a', panes[0].run.id);
    if (comparing && panes[1].run) h.set('b', panes[1].run.id);
    if (layer) h.set('layer', layer);
    history.replaceState(null, '', `${location.pathname}${location.search}#${h}`);
  }

  function open(i, id) {
    const p = panes[i];
    if (p.run && p.run.id === id) return;
    if (p.run) p.run.close();
    p.run = id ? new RunView(id) : null;
    p.cur = -1; p.drawn = null; p.box = null;
    if (i === 1 && id) setCompare(true);
    if (i === focus) { mode = 'live'; buttons(); }
    tickerDirty = globeDirty = mapDirty = true;
    hashState();
    runsDirty = true;
    paneHead(p);
  }

  function setCompare(on) {
    comparing = on;
    $('paneB').hidden = !on;
    $('panes').classList.toggle('compare', on);
    $('compare').classList.toggle('on', on);
    if (!on && focus === 1) setFocus(0);
    if (!on && panes[1].run) { panes[1].run.close(); panes[1].run = null; }
    mapDirty = true;
    hashState();
    runsDirty = true;
  }

  function setFocus(i) {
    if (focus === i) return;
    focus = i;
    for (const p of panes) p.el.classList.toggle('focus', p.i === i);
    tickerDirty = globeDirty = mapDirty = true;
    texImg = null;
    label(); runInfo(); legend();
  }

  // Something arrived for a run: redraw what shows it.
  function changed(run, frames) {
    for (const p of panes) {
      if (p.run !== run) continue;
      if (frames) {
        if (!layer && run.layers.length) layer = run.layers[0];
        layerButtons();
        if (p.i === focus && p.cur < 0 && run.frames.length) { const n = run.frames.length - 1; if (ready(run.image(n))) show(p, n); }
      }
      paneHead(p);
    }
    if (run === P().run) { label(); runInfo(); }
    mapDirty = true;
  }

  function show(p, n) {
    if (!p.run || n < 0 || n >= p.run.frames.length) return;
    if (n !== p.cur && p.i === focus) { tickerDirty = true; globeDirty = true; }
    p.cur = n;
    mapDirty = true;
    if (p.i === focus) label();
  }

  function focusT() {
    const p = P();
    return p.run && p.cur >= 0 ? p.run.frames[p.cur].t : null;
  }

  function setMode(m) {
    mode = m;
    const t = focusT();
    if (t !== null) tp = t;
    buttons();
  }

  let lastTs = null;
  function tick(ts) {
    const dt = lastTs === null ? 0 : Math.min(0.25, (ts - lastTs) / 1000);
    lastTs = ts;
    const p = P(), r = p.run;
    if (r && r.frames.length) {
      const last = r.frames.length - 1;
      let want = p.cur < 0 ? last : p.cur;
      if (mode === 'live') want = last;
      else if (mode === 'play') {
        tp += rate * 1e6 * dt;
        want = Math.max(p.cur, r.frameAt(tp));
        if (want >= last && tp >= r.frames[last].t) { want = last; mode = r.finished() ? 'pause' : 'live'; buttons(); }
        for (let k = 1; k <= 4 && want + k <= last; k++) r.image(want + k);   // prefetch
      }
      if (want !== p.cur) {
        // Switch only once the image is here, so the map never goes blank; hold the playhead.
        if (ready(r.image(want))) show(p, want);
        else if (mode === 'play') tp = Math.min(tp, r.frames[want].t);
      }
    }
    // The other pane shows its frame at the focused pane's time.
    const t = focusT();
    for (const o of panes) {
      if (o === p || !o.run || !o.run.frames.length || (o.i === 1 && !comparing)) continue;
      const want = t === null ? o.run.frames.length - 1 : o.run.frameAt(t);
      if (want !== o.cur && ready(o.run.image(want))) { o.cur = want; mapDirty = true; }
    }
    if (mapDirty) { mapDirty = false; for (const q of panes) if (q.i === 0 || comparing) drawPane(q); }
    if (tickerDirty) { tickerDirty = false; ticker(); }
    spin(dt);
    requestAnimationFrame(tick);
  }

  // =================================================================================================
  // The map
  // =================================================================================================
  const FADE_YEARS = 24e6;
  const placeLatLon = e => (e.place && e.place.type === 'point') ? [e.place.lat, e.place.lon] : null;
  // Keyframe pixel px is centred on lon 360°·px/w (the seam is 0°), py on lat 90° − 180°·py/h.
  function mapXY(lat, lon, f, W) {
    const s = W / f.w;
    return [(((lon % 360) + 360) % 360 / 360 * f.w + 0.5) * s, ((90 - lat) / 180 * f.h + 0.5) * s];
  }
  function mapLatLon(p, x, y) {
    return [Math.max(-90, Math.min(90, 90 - y / p.canvas.height * 180)), ((x / p.canvas.width * 360) % 360 + 360) % 360];
  }
  // Block coordinates of a point (plan §5.2; the static viewer's inspect): x = 360°-longitude
  // scaled to C, from the seam at −C/2; north is −z.
  function blockXZ(run, lat, lon) {
    const C = (run && run.meta && run.meta.circumference) || 0;
    return [Math.round(lon / 360 * C - C / 2), Math.round(-lat / 360 * C)];
  }
  function visibleEvents(run, t) {
    const out = [];
    for (const e of run.events) {
      if (e.t_start > t) continue;
      const ll = placeLatLon(e);
      if (!ll) continue;
      const end = Math.min(Math.max(e.t_end ?? e.t_start, e.t_start), t);
      const age = t - end;
      if (age <= FADE_YEARS) out.push({ e, ll, age });
    }
    return out;
  }

  function drawPane(p) {
    const dpr = window.devicePixelRatio || 1, c = p.canvas, ctx = p.ctx;
    c.width = Math.max(1, Math.round(c.clientWidth * dpr));
    c.height = Math.max(1, Math.round(c.clientHeight * dpr));
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, c.width, c.height);
    const r = p.run;
    p.waiting.hidden = !!(r && r.frames.length);
    if (r && !r.frames.length) {
      const st = r.status ? r.status.state : '';
      p.waiting.textContent = !r.connected && !r.meta ? 'Connecting…' : st === 'queued' ? 'Queued: it starts when the run before it ends.' : st === 'running' ? 'Waiting for the first step…' : `No frames (${st}).`;
    }
    if (!r || p.cur < 0 || !r.frames[p.cur]) { p.clock.textContent = ''; badge(p); return; }
    const f = r.frames[p.cur];
    const img = r.image(p.cur);
    if (ready(img)) p.drawn = img;
    if (p.drawn) ctx.drawImage(p.drawn, 0, 0, c.width, c.height);
    if (p.i === focus && p.drawn && p.drawn !== texImg && ready(p.drawn)) { texImg = p.drawn; G.tex = pixels(p.drawn); globeDirty = true; }
    if ($('markers').checked) {
      const t = f.t, t0 = p.cur > 0 ? r.frames[p.cur - 1].t : -Infinity, rad = 3.5 * dpr;
      const vis = visibleEvents(r, t);
      for (const { e, ll, age } of vis) {
        if (e.t_start > t0) continue;
        const [x, y] = mapXY(ll[0], ll[1], f, c.width);
        ctx.globalAlpha = 1 - 0.75 * age / FADE_YEARS;
        ctx.beginPath(); ctx.arc(x, y, rad, 0, 2 * Math.PI);
        ctx.fillStyle = kindColour(e.kind); ctx.fill();
        ctx.lineWidth = dpr; ctx.strokeStyle = '#000'; ctx.stroke();
      }
      ctx.globalAlpha = 1;
      for (const { e, ll } of vis) {
        if (!(e.t_start > t0)) continue;
        const [x, y] = mapXY(ll[0], ll[1], f, c.width);
        ctx.beginPath(); ctx.arc(x, y, 1.9 * rad, 0, 2 * Math.PI);
        ctx.fillStyle = kindColour(e.kind); ctx.fill();
        ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.stroke();
        ctx.beginPath(); ctx.arc(x, y, 3.2 * rad, 0, 2 * Math.PI);
        ctx.lineWidth = dpr; ctx.strokeStyle = 'rgba(255,255,255,0.6)'; ctx.stroke();
      }
    }
    if (p.box) {
      const [x0, y0, x1, y1] = p.box;
      ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.setLineDash([6 * dpr, 4 * dpr]);
      ctx.strokeRect(Math.min(x0, x1), Math.min(y0, y1), Math.abs(x1 - x0), Math.abs(y1 - y0));
      ctx.setLineDash([]);
    }
    const other = f.layers.includes(layer) ? '' : ` · no ${layer} here: ${f.layers[0]}`;
    p.clock.innerHTML = `${esc(fmtT(f.t))}<small>${esc(f.stage)} · step ${f.n + 1}${esc(other)}</small>`;
    badge(p);
    if (p.i === focus) legend();
  }

  function paneHead(p) {
    const r = p.run;
    p.name.textContent = r ? (r.meta ? r.meta.label : r.id) : (p.i ? 'B' : 'A');
    const bits = [];
    if (comparing) bits.push(p.i ? 'B' : 'A');
    if (r && r.meta) bits.push(r.meta.preset, r.meta.start || '', r.meta.length_myr ? `${r.meta.length_myr} Myr` : '');
    if (r && r.status) bits.push(r.status.state);
    p.tag.textContent = bits.filter(Boolean).join(' · ');
  }

  function badge(p) {
    const b = p.badge, r = p.run;
    b.hidden = !r || p.cur < 0;
    b.className = 'badge';
    if (b.hidden) return;
    if (p.i !== focus) { b.textContent = 'SAME TIME'; return; }
    if (mode === 'live' && !r.finished()) { b.textContent = 'LIVE'; b.classList.add('live'); }
    else if (mode === 'live') b.textContent = r.status.state === 'done' ? 'END OF RUN' : r.status.state.toUpperCase();
    else if (mode === 'play') b.textContent = `REPLAY · ${rate} Myr/s`;
    else b.textContent = 'PAUSED';
  }

  // Map pointer: a click picks an event (or inspects the column and turns the globe there); with
  // ▭ area on, a drag draws a box to preview.
  for (const p of panes) {
    let down = null;
    const at = ev => { const rect = p.canvas.getBoundingClientRect(), d = window.devicePixelRatio || 1; return [(ev.clientX - rect.left) * d, (ev.clientY - rect.top) * d]; };
    p.canvas.addEventListener('pointerdown', ev => {
      setFocus(p.i);
      down = at(ev);
      if (boxMode) { p.canvas.setPointerCapture(ev.pointerId); p.box = [...down, ...down]; }
    });
    p.canvas.addEventListener('pointermove', ev => {
      if (!down || !boxMode) return;
      const [x, y] = at(ev);
      p.box = [down[0], down[1], x, y];
      mapDirty = true;
    });
    p.canvas.addEventListener('pointerup', ev => {
      if (!down) return;
      const [x, y] = at(ev);
      const moved = Math.abs(x - down[0]) + Math.abs(y - down[1]) > 6;
      const start = down;
      down = null;
      const r = p.run;
      if (!r || p.cur < 0) return;
      if (boxMode && moved) {
        const [la0, lo0] = mapLatLon(p, Math.min(start[0], x), Math.min(start[1], y));
        const [la1, lo1] = mapLatLon(p, Math.max(start[0], x), Math.max(start[1], y));
        const [x0, z0] = blockXZ(r, la0, lo0), [x1, z1] = blockXZ(r, la1, lo1);
        Stage2.preview(r, r.frames[p.cur], { x0, z0, x1, z1 }, { north: la0, south: la1, west: lo0, east: lo1 });
        return;
      }
      p.box = null; mapDirty = true;
      const f = r.frames[p.cur], d = window.devicePixelRatio || 1;
      let best = null, bd = (12 * d) ** 2;
      if ($('markers').checked) {
        for (const { e, ll } of visibleEvents(r, f.t)) {
          const [ex, ey] = mapXY(ll[0], ll[1], f, p.canvas.width);
          const dd = (ex - x) ** 2 + (ey - y) ** 2;
          if (dd < bd) { bd = dd; best = e; }
        }
      }
      if (best) { showEvent(r, best.id); return; }
      const [lat, lon] = mapLatLon(p, x, y);
      G.lon0 = lon; G.lat0 = Math.max(-80, Math.min(80, lat)); G.spin = false; globeDirty = true;
      const [bx, bz] = blockXZ(r, lat, lon);
      Stage2.inspect(r, f, bx, bz, lat, lon);
    });
  }

  // =================================================================================================
  // Stage 2 hook (P1-20): the 3D block preview of an area and click-to-inspect columns are built
  // beside this page (a sibling worktree: crates/deeptime-inspect `export-area`, tools/workbench/, a
  // 3D viewer asset folder). This object is the page's only contact with them:
  //   inspect(run, frame, x, z)  → GET  api/inspect?run=&frame=&x=&z=   (a column: rock layers, ore
  //                                bodies, "why here", through deeptime inspect-column's evaluator)
  //   preview(run, frame, box)   → POST api/preview {run, frame, t_years, box: {x0, z0, x1, z1}
  //                                (blocks), lat_lon: {north, south, west, east}}
  // Today both answer 501 {"available": false, "message"}. The planned contract is a job: the
  // answer carries {job, status_url} → poll status → {state, url} (the export to show); render()
  // already follows a `status_url` and embeds a `url`, so wiring stage 2 is a server change plus,
  // at most, this object. The panel #stage2 is the slot it draws into.
  // =================================================================================================
  const Stage2 = {
    body: $('stage2body'),
    async inspect(run, frame, x, z, lat, lon) {
      this.body.innerHTML = `<span class="hint">Inspecting x ${x}, z ${z} (${lat.toFixed(2)}°, ${lon.toFixed(2)}°) of ${esc(run.meta ? run.meta.label : run.id)} at ${esc(fmtT(frame.t))}…</span>`;
      const q = new URLSearchParams({ run: run.id, frame: frame.n, x, z, lat, lon });
      this.render((await api(`api/inspect?${q}`)).body, `column x ${x}, z ${z}`);
    },
    async preview(run, frame, box, latLon) {
      this.body.innerHTML = `<span class="hint">Asking for a 3D preview of x ${box.x0}…${box.x1}, z ${box.z0}…${box.z1}…</span>`;
      const r = await api('api/preview', { run: run.id, frame: frame.n, t_years: frame.t, box, lat_lon: latLon });
      this.render(r.body, `area x ${box.x0}…${box.x1}, z ${box.z0}…${box.z1}`);
    },
    render(j, what) {
      if (!j) return;
      if (j.available === false || j.error) {
        this.body.innerHTML = `<div><b>${esc(what)}</b></div><div class="hint">${esc(j.message || j.error)}</div>`;
        return;
      }
      // Which world the preview is (stage 2 says so: not the run's own terrain until P1-08).
      const note = j.note ? `<div class="hint">${esc(j.note)}</div>` : '';
      if (j.url) {
        this.body.innerHTML = `<div><b>${esc(what)}</b> · <a class="ref" href="${esc(j.url)}" target="_blank" rel="noopener">open</a></div>` +
          note + `<iframe src="${esc(j.url)}" style="width:100%;height:320px;border:1px solid var(--line)"></iframe>`;
        return;
      }
      if (j.status_url) {
        this.body.innerHTML = `<div><b>${esc(what)}</b></div><div class="hint">${esc(j.state || 'queued')}… ${esc(j.message || '')}</div>` + note;
        setTimeout(async () => this.render((await api(j.status_url)).body, what), 1000);
        return;
      }
      this.body.innerHTML = `<div><b>${esc(what)}</b></div><pre class="lines">${esc(JSON.stringify(j, null, 1))}</pre>`;
    },
  };

  // =================================================================================================
  // The globe: an orthographic view of the focused pane's frame, on the CPU
  // =================================================================================================
  const globe = $('globe'), gctx = globe.getContext('2d');
  const G = { lon0: 0, lat0: 20, spin: TV || Q.has('spin'), speed: 6, tex: null };
  let texImg = null, gpre = null, gbuf = null, lastGlobe = 0;
  const off = document.createElement('canvas');
  function pixels(img) {
    off.width = img.naturalWidth; off.height = img.naturalHeight;
    const o = off.getContext('2d', { willReadFrequently: true });
    o.clearRect(0, 0, off.width, off.height);
    o.drawImage(img, 0, 0);
    return { w: off.width, h: off.height, d: o.getImageData(0, 0, off.width, off.height).data };
  }
  function globePre(S) {
    const n = S * S, xs = new Float32Array(n), ys = new Float32Array(n), zs = new Float32Array(n), inside = new Uint8Array(n);
    const r = S / 2 - 2;
    for (let j = 0; j < S; j++) for (let i = 0; i < S; i++) {
      const x = (i + 0.5 - S / 2) / r, y = (S / 2 - j - 0.5) / r, d = x * x + y * y, k = j * S + i;
      if (d <= 1) { inside[k] = 1; xs[k] = x; ys[k] = y; zs[k] = Math.sqrt(1 - d); }
    }
    return { S, xs, ys, zs, inside };
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
    gctx.beginPath(); gctx.arc(S / 2, S / 2, S / 2 - 2, 0, 2 * Math.PI);
    gctx.lineWidth = 2; gctx.strokeStyle = 'rgba(140,180,255,0.55)'; gctx.stroke();
    const p = P(), r = p.run;
    if ($('markers').checked && r && p.cur >= 0) {
      const t = r.frames[p.cur].t, t0 = p.cur > 0 ? r.frames[p.cur - 1].t : -Infinity;
      for (const { e, ll, age } of visibleEvents(r, t)) {
        const q = globeXY(ll[0], ll[1]);
        if (!q) continue;
        const fresh = e.t_start > t0;
        gctx.globalAlpha = fresh ? 1 : 1 - 0.75 * age / FADE_YEARS;
        gctx.beginPath(); gctx.arc(q[0], q[1], fresh ? 6 : 3.5, 0, 2 * Math.PI);
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

  // =================================================================================================
  // Legend, layers, labels, events
  // =================================================================================================
  function legend() {
    const p = P(), r = p.run;
    if (!ramps || !r || p.cur < 0 || !r.frames[p.cur]) return;
    const f = r.frames[p.cur], li = r.layerIndex(f);
    const name = f.ramps && f.ramps[li], ramp = name && ramps.ramps[name];
    $('lname').textContent = f.layers[li] || '';
    if (!ramp) return;
    const bar = $('legendbar'), lc = bar.getContext('2d');
    const stops = ramp.stops, lo = stops[0][0], hi = stops[stops.length - 1][0];
    const grad = lc.createLinearGradient(0, 0, bar.width, 0);
    for (const [v, c] of stops) grad.addColorStop((v - lo) / (hi - lo || 1), c);
    lc.fillStyle = grad; lc.fillRect(0, 0, bar.width, bar.height);
    $('lmin').textContent = lo; $('lmax').textContent = hi; $('lunits').textContent = name;
    $('legendnote').textContent = ramp._comment || '';
  }

  let shownLayers = '';
  function layerButtons() {
    const names = [];
    for (const p of panes) if (p.run && (p.i === 0 || comparing)) for (const n of p.run.layers) if (!names.includes(n)) names.push(n);
    const key = names.join('|') + '#' + layer;
    if (key === shownLayers) return;
    shownLayers = key;
    const box = $('layers');
    box.textContent = '';
    names.forEach((name, i) => {
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
    shownLayers = '';
    layerButtons();
    for (const p of panes) if (p.run && p.cur >= 0) p.run.image(p.cur);
    mapDirty = true; texImg = null;
    hashState();
  }

  function label() {
    const p = P(), r = p.run, s = $('scrub');
    if (!r || !r.frames.length) { s.max = 0; s.value = 0; $('tlabel').textContent = ''; return; }
    s.max = r.frames.length - 1;
    if (p.cur >= 0) s.value = p.cur;
    const f = r.frames[p.cur], live = r.frames[r.frames.length - 1];
    $('tlabel').textContent = f
      ? `step ${p.cur + 1}/${r.frames.length} · ${fmtT(f.t)}${live !== f && !r.finished() ? ` · the run is at ${fmtT(live.t)}` : ''}`
      : '';
    for (const q of panes) badge(q);
  }

  function buttons() {
    $('play').textContent = mode === 'play' ? '❚❚' : '▶︎';
    $('live').classList.toggle('on', mode === 'live');
    $('boxtool').classList.toggle('on', boxMode);
    for (const q of panes) badge(q);
  }

  function ticker() {
    const box = $('ticker'), p = P(), r = p.run;
    if (!r) { box.className = 'hint'; box.textContent = 'No run.'; $('evcount').textContent = ''; return; }
    const t = p.cur >= 0 ? r.frames[p.cur].t : -Infinity, t0 = p.cur > 0 ? r.frames[p.cur - 1].t : -Infinity;
    // Events are logged when they end; the ticker lists them by when they began.
    const upto = r.events.filter(e => e.t_start <= t).sort((a, b) => a.t_start - b.t_start);
    $('evcount').textContent = r.events.length ? `${upto.length} of ${r.events.length}` : '';
    const shown = upto.slice(-150).reverse();
    if (!shown.length) { box.className = 'hint'; box.textContent = 'No events yet.'; return; }
    box.className = '';
    box.innerHTML = shown.map(e =>
      `<div class="row${e.t_start > t0 ? ' new' : ''}" data-ev="${e.id}"><span class="t">${esc(fmtT(e.t_start))}</span>` +
      `<span class="dot" style="background:${kindColour(e.kind)}"></span><span>${esc(shortKind(e.kind))}</span></div>`).join('');
  }

  function evLink(r, id) {
    const e = r.byId.get(id);
    const w = Math.floor(id / 2 ** 48), seq = id - w * 2 ** 48;
    return e ? `<a class="ref" data-ev="${id}">${esc(shortKind(e.kind))} ${w}:${seq}</a>` : `${w}:${seq} (not in this log)`;
  }
  function placeText(p) {
    if (!p) return '—';
    if (p.type === 'point') return `${Math.abs(p.lat).toFixed(2)}° ${p.lat >= 0 ? 'N' : 'S'}, longitude ${p.lon.toFixed(2)}°`;
    if (p.type === 'global') return 'global';
    return esc(`${p.type} ${p[p.type === 'region' ? 'mask' : p.type]}`);
  }
  function showEvent(r, id) {
    const e = r.byId.get(id);
    if (!e) return;
    const param = v => (v && typeof v === 'object' && 'id' in v) ? (r.byId.has(v.id) ? evLink(r, v.id) : esc(`id ${v.id}`)) : esc(v === null ? '—' : v);
    const rows = [
      ['event', evLink(r, e.id)], ['kind', esc(e.kind)], ['source', esc(e.source)],
      ['time', esc(e.t_end > e.t_start ? `${fmtT(e.t_start)} → ${fmtT(e.t_end)}` : fmtT(e.t_start))],
      ['place', placeText(e.place)],
      ['magnitude', esc(e.magnitude === null || e.magnitude === undefined ? '—' : Number(e.magnitude).toPrecision(3))],
    ];
    for (const [k, v] of Object.entries(e.params || {})) rows.push([k, param(v)]);
    rows.push(['caused by', (e.causes || []).length ? e.causes.map(c => evLink(r, c)).join('<br>') : '—']);
    const eff = r.effects.get(e.id) || [];
    rows.push(['led to', eff.length ? eff.map(c => evLink(r, c)).join('<br>') : '—']);
    $('details').innerHTML = kv(rows);
    $('detailpanel').hidden = false;
    const ll = placeLatLon(e);
    if (ll) { G.lon0 = ((ll[1] % 360) + 360) % 360; G.lat0 = Math.max(-80, Math.min(80, ll[0])); G.spin = false; globeDirty = true; }
  }
  const kv = rows => '<table class="kv">' + rows.map(([a, b]) => `<tr><td>${esc(a)}</td><td>${b}</td></tr>`).join('') + '</table>';
  document.body.addEventListener('click', ev => {
    const el = ev.target.closest('[data-ev]');
    if (el && P().run) showEvent(P().run, Number(el.dataset.ev));
  });
  $('closedetails').addEventListener('click', () => { $('detailpanel').hidden = true; });

  function runInfo() {
    const r = P().run, box = $('runinfobody');
    if (!r || !r.meta) { box.innerHTML = '<span class="hint">No run.</span>'; return; }
    const m = r.meta, p = r.progress || {}, st = r.status || {};
    const rows = [
      ['label', esc(m.label)], ['preset', esc(m.preset)], ['start', esc(m.start || '—')],
      ['length', esc(m.length_myr ? `${m.length_myr} Myr` : '—')], ['stages', esc((m.stages || []).join(' → '))],
      ['circumference', esc(`${m.circumference} blocks`)], ['state', esc(st.state + (st.message ? `: ${st.message}` : ''))],
      ['progress', esc(p.total_permille !== undefined ? `${(p.total_permille / 10).toFixed(1)} % · ${fmtDur(p.elapsed_ms)} · peak ${fmtBytes(p.bytes_peak || 0)}` : '—')],
      ['created', esc(m.created_utc)], ['core', esc(m.version)], ['run id', esc(r.id)],
    ];
    if (st.world) rows.push(['world data', esc(st.world)]);
    let h = kv(rows);
    if (st.lines && st.lines.length) h += `<pre class="lines">${esc(st.lines.join('\n'))}</pre>`;
    box.innerHTML = h;
  }

  // =================================================================================================
  // Controls
  // =================================================================================================
  function step(d) {
    const p = P(), r = p.run;
    if (!r || !r.frames.length) return;
    setMode('pause');
    show(p, Math.max(0, Math.min(r.frames.length - 1, (p.cur < 0 ? 0 : p.cur) + d)));
    tp = r.frames[p.cur].t;
  }
  function togglePlay() {
    const p = P(), r = p.run;
    if (!r || !r.frames.length) return;
    if (mode === 'play') { setMode('pause'); return; }
    if (p.cur >= r.frames.length - 1 && r.finished()) show(p, 0);
    setMode('play');
  }
  function rewind() {
    const p = P();
    if (!p.run || !p.run.frames.length) return;
    const was = mode;
    show(p, 0);
    setMode(was === 'pause' ? 'pause' : 'play');
  }
  function setBoxMode(on) { boxMode = on; for (const p of panes) { p.box = null; p.canvas.style.cursor = on ? 'cell' : ''; } mapDirty = true; buttons(); }
  function toggleSidebar() { $('app').classList.toggle('nosidebar'); mapDirty = true; globeDirty = true; }
  $('rewind').addEventListener('click', rewind);
  $('back').addEventListener('click', () => step(-1));
  $('fwd').addEventListener('click', () => step(1));
  $('play').addEventListener('click', togglePlay);
  $('live').addEventListener('click', () => setMode('live'));
  $('rate').addEventListener('change', () => { rate = Number($('rate').value); buttons(); });
  $('scrub').addEventListener('input', () => { const p = P(); if (!p.run) return; setMode('pause'); show(p, Number($('scrub').value)); tp = p.run.frames[p.cur].t; });
  $('markers').addEventListener('change', () => { mapDirty = true; globeDirty = true; });
  $('compare').addEventListener('click', () => {
    if (comparing) { setCompare(false); return; }
    // Compare with the newest other finished run, if any; the runs list's "B" picks another.
    const other = lastRuns.find(r => (!panes[0].run || r.id !== panes[0].run.id) && r.frames > 0);
    setCompare(true);
    if (other) open(1, other.id);
  });
  $('boxtool').addEventListener('click', () => setBoxMode(!boxMode));
  $('togglesidebar').addEventListener('click', toggleSidebar);
  for (const p of panes) p.el.addEventListener('pointerdown', () => setFocus(p.i));
  document.addEventListener('keydown', ev => {
    const k = ev.key;
    const control = ev.target.closest ? ev.target.closest('input, select, textarea, button') : null;
    if (control && !(k === ' ' && control.id === 'scrub')) return;   // a focused control takes its own keys
    if (ev.metaKey || ev.ctrlKey || ev.altKey) return;
    if (k === ' ') { ev.preventDefault(); togglePlay(); }
    else if (k === 'ArrowLeft') step(-1);
    else if (k === 'ArrowRight') step(1);
    else if (k === 'Home') rewind();
    else if (k === 'End' || k === 'l') setMode('live');
    else if (k === 'g') G.spin = !G.spin;
    else if (k === 'm') { $('markers').checked = !$('markers').checked; mapDirty = true; globeDirty = true; }
    else if (k === 'c') $('compare').click();
    else if (k === 'e') toggleSidebar();
    else if (k === 'Escape') setBoxMode(false);
    else if (k === 'f' && root.requestFullscreen) root.requestFullscreen().catch(() => {});
    else if (/^[1-9]$/.test(k)) { const b = $('layers').children[Number(k) - 1]; if (b) setLayer(b.dataset.layer); }
  });
  window.addEventListener('resize', () => { mapDirty = true; globeDirty = true; });
  if (TV) {   // hide the cursor after 3 s without moving it
    let idle = null;
    const wake = () => { root.classList.remove('idle'); clearTimeout(idle); idle = setTimeout(() => root.classList.add('idle'), 3000); };
    document.addEventListener('mousemove', wake);
    wake();
  }

  // =================================================================================================
  // The runs list (polled) and the Mac's status
  // =================================================================================================
  let lastRuns = [], runsDirty = true, runsKey = '', lastRunning = null, firstPoll = true;
  const rows = new Map();   // run id -> row element
  function runRow(r) {
    let el = rows.get(r.id);
    if (!el) {
      el = document.createElement('div');
      el.className = 'run';
      el.dataset.id = r.id;
      el.innerHTML = '<div class="top"><span class="sdot"></span><span class="lbl"></span><span class="st"></span></div><div class="sub"></div><div class="acts"></div>';
      rows.set(r.id, el);
    }
    const inA = panes[0].run && panes[0].run.id === r.id, inB = comparing && panes[1].run && panes[1].run.id === r.id;
    el.classList.toggle('a', !!(inA || inB));
    el.querySelector('.sdot').className = `sdot ${r.state}`;
    el.querySelector('.lbl').textContent = (inA ? 'A · ' : inB ? 'B · ' : '') + r.label;
    const pct = r.total_permille !== null && r.total_permille !== undefined ? ` ${(r.total_permille / 10).toFixed(0)} %` : '';
    el.querySelector('.st').textContent = r.state === 'queued' ? `queued #${r.queue_position}` : r.state === 'running' ? `running${pct}` : r.state;
    const when = r.created_utc ? r.created_utc.replace('T', ' ').replace(/:\d\dZ$/, 'Z') : '';
    el.querySelector('.sub').textContent = [`${r.frames} frames`, `${r.events} events`, r.seconds ? fmtDur(r.seconds * 1000) : '', when, r.message || ''].filter(Boolean).join(' · ');
    const acts = [];
    if (!inA) acts.push(['view', 'View']);
    if (!inB && !inA) acts.push(['compare', 'Compare as B']);
    if (r.state === 'queued' || r.state === 'running') acts.push(['cancel', 'Cancel']);
    acts.push(['settings', 'Load its settings']);
    const key = acts.map(a => a[0]).join(',');
    const box = el.querySelector('.acts');
    if (box.dataset.key !== key) {
      box.dataset.key = key;
      box.innerHTML = acts.map(([a, t]) => `<button data-act="${a}">${t}</button>`).join('');
    }
    return el;
  }
  $('runlist').addEventListener('click', async ev => {
    const b = ev.target.closest('button[data-act]');
    if (!b) return;
    const id = b.closest('.run').dataset.id, r = lastRuns.find(x => x.id === id);
    const act = b.dataset.act;
    if (act === 'view') { open(0, id); setFocus(0); }
    else if (act === 'compare') open(1, id);
    else if (act === 'cancel') {
      if (!confirm(`Cancel ${r ? r.label : id}? A running run stops after its current step and keeps what it made.`)) return;
      const res = await api(`api/runs/${encodeURIComponent(id)}/cancel`, {});
      if (!res.ok) alert(res.body.error);
      poll();
    } else if (act === 'settings' && r) importCode(r.share_code);
  });

  async function poll() {
    let res;
    try { res = await api('api/runs'); } catch (e) { setConn(false, 'the workbench is not answering'); return; }
    if (!res.ok) { setConn(false, `the workbench answered ${res.status}`); return; }
    setConn(true, 'connected');
    const j = res.body;
    lastRuns = j.runs;
    const running = j.runs.find(r => r.state === 'running');
    // Header: the Mac's one run slot.
    $('bar').firstElementChild.style.width = running ? `${(running.total_permille || 0) / 10}%` : '0%';
    $('mactext').textContent = running
      ? `Mac: running ${running.label} · ${fmtT(running.t_years)}${j.queue.length ? ` · ${j.queue.length} waiting` : ''}`
      : `Mac: idle · ${j.runs.length} run${j.runs.length === 1 ? '' : 's'} kept (newest ${j.keep})`;
    $('runscount').textContent = j.runs.length ? `${j.runs.length} · one at a time, ${j.queue_max} may wait` : '';
    const list = $('runlist');
    if (!j.runs.length) { list.className = 'hint'; list.textContent = 'No runs yet.'; }
    else {
      list.className = '';
      const key = j.runs.map(r => r.id).join(',');
      if (key !== runsKey) { list.textContent = ''; runsKey = key; }
      for (const r of j.runs) { const el = runRow(r); if (el.parentNode !== list) list.appendChild(el); }
      for (const id of [...rows.keys()]) if (!j.runs.some(r => r.id === id)) rows.delete(id);
    }
    // Which runs to show: the URL's, else the running one, else the newest; the TV follows new runs.
    if (firstPoll) {
      firstPoll = false;
      const h = new URLSearchParams(location.hash.slice(1));
      if (h.get('layer')) layer = h.get('layer');
      const a = h.get('a') || Q.get('run');
      const pick = (a && j.runs.find(r => r.id === a)) || running || j.runs.find(r => r.frames > 0) || j.runs[0];
      if (pick) open(0, pick.id);
      const b = h.get('b');
      if (b && j.runs.find(r => r.id === b)) open(1, b);
    } else if (running && running.id !== lastRunning && (TV || !panes[0].run)) {
      open(0, running.id);
    }
    lastRunning = running ? running.id : null;
    runsDirty = false;
  }
  function setConn(ok, text) { const c = $('conn'); c.className = ok ? 'ok' : ''; c.textContent = text; }
  setInterval(() => poll(), 1500);

  // =================================================================================================
  // The editor: the Create World setup page's controls
  // =================================================================================================
  let catalog = null, eff = null, effSeq = 0, effTimer = null;
  const form = { preset: null, imported: null, start: '', length: '', fields: {}, cap: 4 };
  const L = (key, ...args) => {
    let s = catalog && catalog.labels[`deeptime.editor.${key}`];
    if (s === undefined) return null;
    for (const a of args) s = s.replace('%s', a);
    return s;
  };
  const familyLabel = f => L(`preset.${f}`) || f.replace(/_/g, ' ');
  const sizeLabel = c => L(`size.${c / 1024}k`) || L('size.other', c.toLocaleString('en')) || `${c} blocks`;

  function buildForm() {
    const fams = [];
    for (const p of catalog.presets) if (!fams.includes(p.family)) fams.push(p.family);
    const sec = t => `<div class="fsec">${esc(t)}</div>`;
    let h = sec(L('preset') || 'Preset');
    h += `<div class="frow"><label for="f_family">${esc(L('preset') || 'Preset')}</label><div class="ctl"><select id="f_family" style="flex:1">` +
      fams.map(f => {
        const ph = catalog.presets.filter(p => p.family === f).every(p => p.placeholder);
        return `<option value="${esc(f)}">${esc(familyLabel(f))}${ph ? ' (placeholder)' : ''}</option>`;
      }).join('') + '<option value="" disabled id="f_importedopt">imported</option></select></div></div>';
    h += `<div class="frow"><label for="f_size">${esc(L('size') || 'Size')}</label><div class="ctl"><select id="f_size" style="flex:1" title="${esc(L('size.tooltip') || '')}"></select></div></div>`;
    h += sec(L('section.timeline') || 'Timeline');
    h += `<div class="frow"><label for="f_start">${esc(L('timeline.start') || 'Start')}</label><div class="ctl"><select id="f_start" style="flex:1"><option value="">${esc(L('timeline.start.none') || 'Choose…')}</option>` +
      catalog.start_configs.map(s => `<option value="${s}">${esc(L(`timeline.start.${s}`) || s)}</option>`).join('') + '</select></div></div>';
    h += `<div class="frow"><label for="f_length">${esc(L('timeline.length') || 'Length (Myr)')}</label><div class="ctl"><input type="number" id="f_length" min="${catalog.length_myr.min}" max="${catalog.length_myr.max}" step="any" placeholder="${catalog.length_myr.min}–${catalog.length_myr.max}"><span class="hint">Myr</span></div></div>`;
    h += `<div class="frow"><label for="f_cap">${esc(L('cap') || 'Memory cap (GiB)')}</label><div class="ctl"><input type="number" id="f_cap" min="0.25" step="0.25" value="4" title="${esc(L('cap.tooltip') || '')}"></div></div>`;
    let section = null;
    for (const f of catalog.fields) {
      if (f.section !== section) { section = f.section; h += sec(L(`section.${section}`) || section); }
      const name = L(`field.${f.id}`) || f.id;
      const range = L('field.range', f.min, f.max, f.unit || '—') || `${f.min}–${f.max}`;
      let ctl;
      if (f.kind === 'number' && f.integer) ctl = `<input type="number" data-f="${f.id}" step="1" min="${f.min}" style="flex:1"><button type="button" data-dice="${f.id}" title="a random seed">🎲</button>`;
      else if (f.kind === 'number') {
        const log = f.min > 0 && f.max / f.min >= 100;
        ctl = `<input type="range" data-r="${f.id}" min="0" max="1000" ${log ? 'data-log="1"' : ''}><input type="number" data-f="${f.id}" step="any" min="${f.min}" max="${f.max}"><span class="hint">${esc(f.unit || '')}</span>`;
      } else if (f.kind === 'toggle') ctl = `<input type="checkbox" data-f="${f.id}">`;
      else ctl = `<select data-f="${f.id}" style="flex:1"></select>`;
      h += `<div class="frow off" id="row_${f.id}" title="${esc(range)}"><label>${esc(name)}</label><div class="ctl">${ctl}</div></div>`;
    }
    h += '<div class="brow"><button type="button" id="f_reset">Reset parameters</button></div>';
    $('form').innerHTML = h;
    $('f_family').addEventListener('change', () => {
      const fam = $('f_family').value, sizes = catalog.presets.filter(p => p.family === fam);
      const cur = form.preset ? catalog.presets.find(p => p.name === form.preset) : null;
      const same = sizes.find(p => cur && p.circumference === cur.circumference) || sizes.find(p => p.circumference === 16384) || sizes[0];
      form.preset = same.name; form.imported = null;
      presetChanged();
    });
    $('f_size').addEventListener('change', () => { form.preset = $('f_size').value; presetChanged(); });
    $('f_start').addEventListener('change', () => { form.start = $('f_start').value; refresh(); });
    $('f_length').addEventListener('input', () => { form.length = $('f_length').value; refresh(); });
    $('f_cap').addEventListener('input', () => { form.cap = Number($('f_cap').value); });
    $('f_reset').addEventListener('click', () => { form.fields = {}; refresh(true); });
    const toSlider = (f, v, log) => log ? 1000 * Math.log(v / f.min) / Math.log(f.max / f.min) : 1000 * (v - f.min) / (f.max - f.min);
    const fromSlider = (f, s, log) => log ? f.min * Math.pow(f.max / f.min, s / 1000) : f.min + (f.max - f.min) * s / 1000;
    const tidy = v => Number(v.toPrecision(v >= 100 ? 4 : 3));
    for (const f of catalog.fields) {
      const input = $('form').querySelector(`[data-f="${f.id}"]`), slider = $('form').querySelector(`[data-r="${f.id}"]`);
      f._set = v => {   // show the effective value without firing
        if (document.activeElement === input) return;
        if (f.kind === 'toggle') input.checked = v === true;
        else if (f.kind === 'choice') input.innerHTML = v === null ? '<option>—</option>' : `<option>${esc(v)}</option>`;
        else input.value = v === null ? '' : v;
        if (slider && typeof v === 'number') slider.value = toSlider(f, v, !!slider.dataset.log);
      };
      input.addEventListener(f.kind === 'number' ? 'input' : 'change', () => {
        form.fields[f.id] = f.kind === 'toggle' ? input.checked : input.value;
        const n = Number(input.value);
        if (slider && isFinite(n)) slider.value = toSlider(f, n, !!slider.dataset.log);
        refresh();
      });
      if (slider) slider.addEventListener('input', () => {
        const v = tidy(fromSlider(f, Number(slider.value), !!slider.dataset.log));
        input.value = v; form.fields[f.id] = v; refresh();
      });
    }
    $('form').addEventListener('click', ev => {
      const b = ev.target.closest('[data-dice]');
      if (!b) return;
      const v = Math.floor(Math.random() * 2 ** 32);
      $('form').querySelector(`[data-f="${b.dataset.dice}"]`).value = v;
      form.fields[b.dataset.dice] = v; refresh();
    });
  }

  function presetChanged() {
    const cur = form.preset ? catalog.presets.find(p => p.name === form.preset) : null;
    const opt = $('f_importedopt');
    if (form.imported) {
      opt.textContent = `imported: ${form.imported.name || 'custom'}`;
      opt.hidden = false;
      $('f_family').value = '';
      const c = form.imported.scale && form.imported.scale.circumference_blocks;
      $('f_size').innerHTML = `<option>${esc(c ? sizeLabel(c) : '?')}</option>`;
      $('f_size').disabled = true;
      $('presetnote').textContent = 'imported';
    } else if (cur) {
      opt.hidden = true;
      $('f_family').value = cur.family;
      $('f_size').disabled = false;
      $('f_size').innerHTML = catalog.presets.filter(p => p.family === cur.family)
        .map(p => `<option value="${p.name}">${esc(sizeLabel(p.circumference))}</option>`).join('');
      $('f_size').value = cur.name;
      $('presetnote').textContent = cur.placeholder ? 'placeholder' : cur.stages.join(' + ');
      $('f_family').title = cur.description;
    }
    refresh(true);
  }

  function refresh(now) {
    clearTimeout(effTimer);
    effTimer = setTimeout(computeEffective, now ? 0 : 250);
  }

  async function computeEffective() {
    const seq = ++effSeq;
    const req = { fields: form.fields };
    if (form.imported) req.imported = form.imported; else req.preset = form.preset;
    if (form.start) req.start = form.start;
    if (form.length !== '' && isFinite(Number(form.length))) req.length_myr = Number(form.length);
    const res = await api('api/preset', req);
    if (seq !== effSeq) return;   // a newer answer is on its way
    eff = res.body;
    if (eff.error) { $('problems').innerHTML = `<li>${esc(eff.error)}</li>`; $('runbtn').disabled = true; return; }
    for (const fs of eff.fields) {
      const f = catalog.fields.find(x => x.id === fs.id), row = $(`row_${fs.id}`);
      if (!f || !row) continue;
      row.classList.toggle('off', !fs.available);
      for (const el of row.querySelectorAll('input, select, button')) el.disabled = !fs.available;
      const name = L(`field.${f.id}`) || f.id;
      row.title = f.phase3 ? (L('field.phase3') || 'Phase 3') : !fs.available ? (L('field.unavailable', name.toLowerCase()) || 'not in this preset') : (L('field.range', f.min, f.max, f.unit || '—') || '');
      const bad = eff.field_problems[fs.id];
      row.classList.toggle('invalid', !!bad);
      if (bad) row.title = `${name}: ${bad}`;
      else f._set(fs.value);
    }
    $('problems').innerHTML = eff.problems.map(p => `<li>${esc(p)}</li>`).join('');
    const e = eff.estimate;
    $('estimate').textContent = e && e.peak_bytes
      ? `Estimate: peak ${fmtBytes(e.peak_bytes)}, about ${fmtDur(1000 * e.stages.reduce((s, x) => s + (x.seconds || 0), 0))} (${e.stages.map(s => s.name).join(', ')})${e.peak_bytes > form.cap * 2 ** 30 ? ' · over the memory cap' : ''}`
      : '';
    $('sharecode').value = eff.share_code || '';
    $('runbtn').disabled = !eff.can_run;
  }

  async function importCode(code) {
    const msg = $('importmsg');
    const res = await api('api/decode', { code: (code || '').trim() });
    if (!res.ok) { msg.className = 'hint bad'; msg.textContent = res.body.error; return; }
    const d = res.body;
    form.preset = d.preset; form.imported = d.preset ? null : d.imported; form.fields = {};
    form.start = d.start || ''; form.length = d.length_myr ?? '';
    $('f_start').value = form.start; $('f_length').value = form.length;
    msg.className = 'hint';
    msg.textContent = d.preset ? `This is the built-in ${d.preset}.` : `Imported ${d.imported.name || 'a custom preset'} (its values are shown; edit and run).`;
    if (!$('newrun').open) $('newrun').open = true;
    presetChanged();
  }

  $('runbtn').addEventListener('click', async () => {
    if (!eff || !eff.can_run) return;
    $('runbtn').disabled = true;
    const res = await api('api/runs', { json: eff.json, label: $('label').value, cap_gib: form.cap });
    if (!res.ok) { $('problems').innerHTML = `<li>${esc(res.body.error)}</li>`; $('runbtn').disabled = false; return; }
    $('label').value = '';
    open(0, res.body.id); setFocus(0);
    await poll();
    $('runbtn').disabled = false;
  });
  $('copycode').addEventListener('click', async () => {
    const t = $('sharecode');
    try { await navigator.clipboard.writeText(t.value); } catch (e) { t.select(); document.execCommand('copy'); }
    $('importmsg').className = 'hint';
    $('importmsg').textContent = L('share.copied') || 'Copied.';
  });
  $('importbtn').addEventListener('click', () => importCode($('importcode').value));

  if (TV) $('app').classList.add('nosidebar');
  api('api/catalog').then(res => {
    catalog = res.body;
    buildForm();
    const def = catalog.presets.find(p => p.name === 'pipeline_quick_16k') || catalog.presets[0];
    form.preset = def.name;
    presetChanged();
  });
  buttons();
  poll();
  requestAnimationFrame(tick);
})();
