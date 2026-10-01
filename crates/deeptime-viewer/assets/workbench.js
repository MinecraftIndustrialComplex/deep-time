// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time workbench (`deeptime watch`, P1-20 stage 1 and 2; decisions log 2026-09-28; the UI
// overhaul of 2026-09-29). Served on the tailnet by the Mac. The page:
//
// - Left sidebar: the runs (`GET api/runs`, polled: one runs at a time on the Mac, a short queue,
//   cancel, open, compare, load settings) and the new-run editor, built from the game's own
//   definitions (`GET api/catalog`: presets, the shared editor_fields.json with its section and
//   field descriptions, the game's lang strings, the start configurations). The server computes the
//   effective preset, share code, problems and estimate (`POST api/preset`) and decodes codes
//   (`POST api/decode`) with the core's own code; the page only collects choices. `POST api/runs`
//   starts a run; a running simulation is never changed from here.
// - The stage: the map of the focused run's frame (one pane, or two in compare mode: a split view
//   with a divider, or side by side), each pane following its run's stream
//   (`api/runs/<id>/stream`: `hello`, `frame`, `events`, `progress`, `status`, `bye`); a frame's
//   layer image is `api/runs/<id>/frames/<n>/<layer>.png`, the core's keyframe coloured with
//   ramps.json. Layers are whatever the frames name, so a new stage's layers appear on their own.
// - Right panel: the globe, the layer picker and legend, and the inspector (events, an event's
//   cause chain, a point's values, stage 2's 3D previews, the run's summary).
// - The dock: transport, speed, follow-live and the timeline (Myr ticks, the run's events).
//
// Stage 2 (`Stage2` below) is the only place that talks to the preview runner: `GET api/inspect`
// and `POST api/preview` answer a job (`status_url` to poll, then the 3D viewer's `url`).
// URL: `#a=<run>&b=<run>&layer=<name>&cmp=split|side`; `?run=<id>`; `?tv` = the big-screen look
// (no chrome, large type, spinning globe, follows the newest run, replays the last one when idle).
'use strict';

(function () {
  const Q = new URLSearchParams(location.search);
  const H0 = new URLSearchParams(location.hash.slice(1));   // read before anything rewrites the hash
  const TV = Q.has('tv');
  const root = document.documentElement;
  if (TV) root.classList.add('tv');
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  const narrow = matchMedia('(max-width: 760px)');
  const medium = matchMedia('(max-width: 1179px)');
  const $ = id => document.getElementById(id);
  const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const ic = (name, cls = '') => `<svg class="ic ${cls}" aria-hidden="true"><use href="#i-${name}"/></svg>`;
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  const fmtInt = n => Math.round(Number(n)).toLocaleString('en-US');
  const shortKind = k => String(k).replace(/^deeptime:/, '');
  const cap1 = s => (s ? s.charAt(0).toUpperCase() + s.slice(1) : s);
  const prettyKind = k => cap1(shortKind(k).replace(/_/g, ' '));
  const D2R = Math.PI / 180;
  const store = {
    get(k, d) { try { const v = localStorage.getItem(`dtwb.${k}`); return v === null ? d : JSON.parse(v); } catch (e) { return d; } },
    set(k, v) { try { localStorage.setItem(`dtwb.${k}`, JSON.stringify(v)); } catch (e) { /* storage blocked */ } },
  };

  async function api(path, body) {
    const opt = body === undefined ? {} : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) };
    let r;
    try { r = await fetch(path, opt); } catch (e) { return { status: 0, ok: false, body: { error: 'The workbench is not answering.' } }; }
    let j = null;
    try { j = await r.json(); } catch (e) { j = { error: `${r.status} ${r.statusText}` }; }
    return { status: r.status, ok: r.ok, body: j || {} };
  }

  // =================================================================================================
  // Formatting
  // =================================================================================================
  const myr = a => (a / 1e6).toLocaleString('en-US', { minimumFractionDigits: 1, maximumFractionDigits: 1 });
  function tParts(t) {
    if (t === null || t === undefined || !isFinite(t)) return ['—', ''];
    if (t === 0) return ['Present', 'day'];
    const a = Math.abs(t);
    const [n, u] = a >= 1e6 ? [myr(a), 'Myr'] : a >= 1e3 ? [(a / 1e3).toFixed(1), 'kyr'] : [a.toFixed(0), 'yr'];
    return t < 0 ? [n, `${u} ago`] : [`+${n}`, u];
  }
  function fmtT(t) {
    if (t === 0) return 'the present';
    const [n, u] = tParts(t);
    return u ? `${n} ${u}` : n;
  }
  const tHTML = t => { const [n, u] = tParts(t); return `${esc(n)}${u ? `<small>${esc(u)}</small>` : ''}`; };
  function fmtDur(ms) {
    if (ms === null || ms === undefined || !isFinite(ms) || ms < 0) return '—';
    const s = Math.round(ms / 1000), m = Math.floor(s / 60);
    return m >= 60 ? `${Math.floor(m / 60)}:${String(m % 60).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}` : `${m}:${String(s % 60).padStart(2, '0')}`;
  }
  function fmtHuman(sec) {
    if (sec === null || sec === undefined || !isFinite(sec)) return '—';
    if (sec < 60) return `${Math.max(1, Math.round(sec))} s`;
    const m = Math.round(sec / 60);
    if (m < 60) return `${m} min`;
    return `${Math.floor(m / 60)} h ${String(m % 60).padStart(2, '0')} min`;
  }
  const fmtBytes = b => b >= 1 << 30 ? `${(b / (1 << 30)).toFixed(2)} GiB` : `${(b / (1 << 20)).toFixed(0)} MiB`;
  function relTime(iso) {
    const t = Date.parse(iso);
    if (!isFinite(t)) return '';
    const s = (Date.now() - t) / 1000;
    if (s < 45) return 'just now';
    if (s < 3600) return `${Math.round(s / 60)} min ago`;
    if (s < 86400) return `${Math.round(s / 3600)} h ago`;
    return new Date(t).toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });
  }
  const latText = lat => `${Math.abs(lat).toFixed(1)}° ${lat >= 0 ? 'N' : 'S'}`;
  const lonText = lon => `${(((lon % 360) + 360) % 360).toFixed(1)}° E`;
  const latLonText = (lat, lon) => `${latText(lat)} · ${lonText(lon)}`;
  function numText(v) {
    if (v === null || v === undefined || !isFinite(v)) return '—';
    const a = Math.abs(v);
    if (a >= 1000) return fmtInt(v);
    if (a >= 100) return v.toFixed(0);
    if (a >= 10) return v.toFixed(1);
    if (a >= 1) return v.toFixed(2);
    return v.toPrecision(2);
  }

  // Event colours by kind (EVENTS.md); a kind not listed gets a colour from its name.
  const KIND_COLOURS = {
    collision: '#e6194b', orogeny_phase: '#b5651d', subduction_onset: '#b04adf', ridge_birth: '#4363d8',
    rift_onset: '#f58231', terrane_accretion: '#bfef45', hotspot_track: '#ff5a1f',
    large_igneous_province: '#ff1f1f', plate_birth: '#3cb44b', plate_death: '#a9a9a9',
    volcano_arc: '#e8552d', volcano_hotspot_chain: '#f0a020', volcano_stratovolcano: '#ff3b1f', volcano_shield: '#ffc02e', volcano_caldera_complex: '#d45cff',
    volcano_caldera: '#ff79c6', volcano_flood_basalt: '#9a8a7a',
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

  // Layers: grouped by their prefix, named for people (the raw name stays in the legend). The
  // first of DEFAULT_LAYERS a run has is shown until someone picks another: the satellite view
  // (true colour, core satellite.rs), else the elevation.
  const DEFAULT_LAYERS = ['satellite', 'elevation'];
  const LAYER_GROUPS = [['', 'Planet'], ['climate', 'Climate'], ['surface', 'Surface'], ['structure', 'Structure'],
    ['strata', 'Strata'], ['volcanoes', 'Volcanoes'], ['deposits', 'Deposits'], ['input', 'Deposit inputs']];
  const layerGroup = name => { const i = name.indexOf('/'); return i < 0 ? '' : name.slice(0, i); };
  const LAYER_WORDS = {
    satellite: 'Satellite', elevation: 'Elevation', plates: 'Plates', crust_age: 'Crust age', regime: 'Tectonic regime', provinces: 'Geologic provinces', water_level: 'Water level',
    t_mean: 'Mean temperature', t_warm: 'Warmest month', t_cold: 'Coldest month', precip: 'Precipitation', koppen: 'Köppen climate',
    aridity: 'Aridity', glacier: 'Glaciers', ela: 'Snow line', temperature: 'Temperature', vegetation: 'Vegetation',
    erosion: 'Erosion', deposition: 'Deposition', sediment: 'Sediment', class: 'Surface rock', drainage: 'Drainage', regolith: 'Regolith',
    rock: 'Rock', log_grade: 'Metamorphic grade', trap_structure: 'Trap structures', outcrop: 'Outcrop', bedrock: 'Bedrock',
    porphyry_cu_mo_au: 'Porphyry Cu-Mo-Au', bif_fe: 'Banded iron', coal_seam: 'Coal seams', mvt_zn_pb: 'MVT Zn-Pb',
    sedex_zn_pb: 'SEDEX Zn-Pb', petroleum_trap: 'Petroleum traps', sites: 'Deposit sites', arc_magma_flux: 'Arc magma flux',
    map: 'Volcanoes', flood_basalt: 'Flood-basalt outcrop',
    carbonate_platform: 'Carbonate platforms', euxinic_basin: 'Euxinic basins', coal_facies: 'Coal facies', erosion_rate: 'Erosion rate',
  };
  const UNIT_SUFFIX = [['_m_myr', 'm/Myr'], ['_mm', 'mm/yr'], ['_km', 'km'], ['_m', 'm'], ['_c', '°C']];
  function layerLabel(name) {
    if (!name) return { label: '', unit: '' };
    let base = name.split('/').pop(), unit = '';
    for (const [s, u] of UNIT_SUFFIX) if (base.endsWith(s) && base.length > s.length) { base = base.slice(0, -s.length); unit = u; break; }
    if (name === 'elevation') unit = 'block y';
    if (name === 'crust_age') unit = 'Myr';
    return { label: LAYER_WORDS[base] || cap1(base.replace(/_/g, ' ')), unit };
  }

  // =================================================================================================
  // Ramps: legends, layer swatches and the value under the pointer (the colour read back)
  // =================================================================================================
  let ramps = null;
  const rampCache = new Map();
  const hexRgb = h => { const v = parseInt(String(h).slice(1, 7), 16); return [(v >> 16) & 255, (v >> 8) & 255, v & 255]; };
  const dist2 = (a, b) => (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2 + (a[2] - b[2]) ** 2;
  function parseLabels(comment, n) {
    const m = comment && comment.match(/(\d+)\s*[-–]\s*(\d+)\s*(?:\(([^)]*)\)|:\s*([^(]*))/);
    if (!m) return null;
    const parts = (m[3] || m[4] || '').split(';')[0].replace(/\.\s*$/, '').split(/,\s*/).map(s => s.trim()).filter(Boolean);
    return parts.length === n ? parts : null;
  }
  function rampInfo(name) {
    if (!ramps || !name || !ramps.ramps[name]) return null;
    if (rampCache.has(name)) return rampCache.get(name);
    const ramp = ramps.ramps[name];
    const stops = ramp.stops.map(([v, c]) => [v, c, hexRgb(c)]);
    // A categorical ramp holds each class k as [k, c], [k + 0.999, c].
    let cat = stops.length >= 3;
    const classes = [];
    for (let i = 0; cat && i < stops.length;) {
      const [v, c, rgb] = stops[i];
      if (!Number.isInteger(v) || (classes.length && v !== classes[0].v + classes.length)) { cat = false; break; }
      classes.push({ v, c, rgb });
      i += (i + 1 < stops.length && stops[i + 1][1] === c && Math.abs(stops[i + 1][0] - v - 0.999) < 1e-6) ? 2 : 1;
    }
    if (cat) {
      const labels = ramp.labels || parseLabels(ramp._comment, classes.length);
      classes.forEach((k, i) => { k.label = labels && labels[i] !== undefined ? labels[i] : null; });
    }
    const info = { name, ramp, stops, cat, classes: cat ? classes : null, labelled: cat && classes.some(k => k.label),
      comment: ramp._comment || '', lo: stops[0][0], hi: stops[stops.length - 1][0] };
    rampCache.set(name, info);
    return info;
  }
  function rampCSS(info) {
    if (!info) return 'var(--bg-5)';
    if (info.cat) {
      const ks = info.classes.slice(0, 6), w = 100 / ks.length;
      return `linear-gradient(90deg, ${ks.map((k, i) => `${k.c} ${(i * w).toFixed(1)}% ${((i + 1) * w).toFixed(1)}%`).join(', ')})`;
    }
    const span = info.hi - info.lo || 1;
    return `linear-gradient(90deg, ${info.stops.map(([v, c]) => `${c} ${(100 * (v - info.lo) / span).toFixed(1)}%`).join(', ')})`;
  }
  function valueAt(info, rgb) {
    if (!info) return null;
    if (info.cat) {
      let best = null, bd = Infinity;
      for (const k of info.classes) { const d = dist2(k.rgb, rgb); if (d < bd) { bd = d; best = k; } }
      return best ? { cls: best, far: bd > 300 } : null;
    }
    let best = null, bd = Infinity;
    for (let i = 0; i + 1 < info.stops.length; i++) {
      const [va, , a] = info.stops[i], [vb, , b] = info.stops[i + 1];
      const d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]], L2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
      const t = L2 ? clamp(((rgb[0] - a[0]) * d[0] + (rgb[1] - a[1]) * d[1] + (rgb[2] - a[2]) * d[2]) / L2, 0, 1) : 0;
      const dd = dist2([a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t], rgb);
      if (dd < bd) { bd = dd; best = va + (vb - va) * t; }
    }
    return { v: best, far: bd > 300 };
  }
  function niceTicks(lo, hi, n) {
    const span = hi - lo;
    if (!(span > 0)) return [lo];
    const raw = span / n, p = 10 ** Math.floor(Math.log10(raw));
    const step = [1, 2, 2.5, 5, 10].map(m => m * p).find(s => span / s <= n) || 10 * p;
    const out = [];
    for (let v = Math.ceil(lo / step) * step; v <= hi + 1e-9; v += step) out.push(+v.toPrecision(10));
    return out;
  }
  function legendHTML(layerName, info, target) {
    const { label, unit } = layerLabel(layerName);
    let h = `<div class="lh"><span class="lname">${esc(label || 'No layer')}</span><span class="lraw">${esc(layerName || '')}${info && info.name !== layerName ? ` · ramp ${esc(info.name)}` : ''}</span><span class="lunit">${esc(unit)}</span></div>`;
    if (!info) return h + '<div class="note">Loading the colours…</div>';
    if (info.cat && info.labelled && info.classes.length <= 40) {
      // A key: swatches with their labels, wider columns for long labels, under the ramp's group
      // headings when it has them (the satellite view's land, snow and ice, sea).
      const wide = info.classes.some(k => String(k.label ?? '').length > 11);
      const cell = k => `<div title="${esc(k.label ?? k.v)}"><i style="background:${k.c}"></i><span>${esc(k.label ?? k.v)}</span></div>`;
      const groups = Array.isArray(info.ramp.groups) ? info.ramp.groups : null;
      if (groups) {
        h += `<div class="cats${wide ? ' wide' : ''} grouped">${groups.map(([g, a, b]) =>
          `<div class="cgroup">${esc(g)}</div>${info.classes.filter(k => k.v >= a && k.v <= b).map(cell).join('')}`).join('')}</div>`;
      } else {
        h += `<div class="cats${wide ? ' wide' : ''}">${info.classes.map(cell).join('')}</div>`;
      }
    } else if (info.cat) {
      h += `<div class="strip">${info.classes.map(k => `<i style="background:${k.c}"></i>`).join('')}</div>`;
      h += `<div class="note" style="-webkit-line-clamp:1">${info.classes.length} classes, unlabelled</div>`;
    } else {
      h += `<canvas class="bar" data-ramp="${esc(info.name)}" data-target="${target}"></canvas><div class="ticks">`;
      const span = info.hi - info.lo || 1;
      const ticks = niceTicks(info.lo, info.hi, 4).filter(v => (v - info.lo) / span > 0.14 && (info.hi - v) / span > 0.14);
      const all = [info.lo, ...ticks, info.hi];
      h += all.map(v => `<span style="left:${(100 * (v - info.lo) / span).toFixed(2)}%">${esc(numText(v))}</span>`).join('');
      h += '</div>';
      if (layerName === 'elevation') h += '<div class="note">Sea level is y 63 (the colour break from blue to green).</div>';
    }
    if (info.comment) h += `<div class="note">${esc(info.comment)}</div>`;
    return h;
  }
  function paintLegendBars(el) {
    for (const c of el.querySelectorAll('canvas.bar')) {
      const info = rampInfo(c.dataset.ramp);
      if (!info) continue;
      const dpr = window.devicePixelRatio || 1;
      c.width = Math.max(1, Math.round(c.clientWidth * dpr)); c.height = Math.max(1, Math.round(c.clientHeight * dpr));
      const g = c.getContext('2d'), grad = g.createLinearGradient(0, 0, c.width, 0), span = info.hi - info.lo || 1;
      for (const [v, col] of info.stops) grad.addColorStop(clamp((v - info.lo) / span, 0, 1), col);
      g.fillStyle = grad; g.fillRect(0, 0, c.width, c.height);
    }
  }

  // =================================================================================================
  // A run's view: its stream, frames, events and images
  // =================================================================================================
  const FINAL = ['done', 'failed', 'cancelled', 'interrupted'];
  class RunView {
    constructor(id) {
      this.id = id;
      this.meta = null; this.status = null; this.progress = null;
      this.frames = []; this.events = []; this.byId = new Map(); this.effects = new Map();
      this.layers = [];            // every layer a frame named, in order of appearance
      this.rampOf = new Map();     // layer -> ramp name
      this.images = new Map();     // "n/li" -> Image (LRU)
      this.failed = new Map();
      this.deposits = null; this.depFrom = -1; this.depState = '';   // the deposits overlay (below)
      this.connected = false;
      this.es = new EventSource(`api/runs/${encodeURIComponent(id)}/stream`);
      this.es.addEventListener('hello', ev => this.hello(JSON.parse(ev.data)));
      this.es.addEventListener('frame', ev => this.addFrames([JSON.parse(ev.data)]));
      this.es.addEventListener('events', ev => this.addEvents(JSON.parse(ev.data).events));
      this.es.addEventListener('progress', ev => { this.progress = JSON.parse(ev.data); changed(this); });
      this.es.addEventListener('status', ev => { this.status = JSON.parse(ev.data); changed(this); depStatus(this); });
      this.es.addEventListener('bye', () => { this.es.close(); this.connected = false; changed(this); });
      this.es.onerror = () => { this.connected = false; changed(this); };
    }
    close() { this.es.close(); }
    finished() { return !!this.status && FINAL.includes(this.status.state); }
    running() { return !!this.status && !this.finished() && this.status.state !== 'queued'; }
    placeholder() { return this.meta ? isPlaceholder(this.meta.preset, this.meta.stages) : false; }
    hello(h) {
      this.connected = true;
      this.meta = h.meta;
      this.frames = []; this.events = []; this.byId.clear(); this.effects.clear();
      this.addFrames(h.frames, true);
      this.addEvents(h.events, true);
      this.progress = h.progress; this.status = h.status;
      changed(this, true);
      eventsDirty();
    }
    addFrames(list, quiet) {
      for (const f of list) {
        this.frames[f.n] = f;
        f.layers.forEach((name, i) => {
          if (!this.layers.includes(name)) this.layers.push(name);
          if (f.ramps && f.ramps[i] && !this.rampOf.has(name)) this.rampOf.set(name, f.ramps[i]);
        });
        depFrame(this, f);
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
      if (!quiet) eventsDirty();
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
    // The whole planned history, [start, end] in years: the timeline's axis.
    range() {
      const m = this.meta || {}, f = this.frames;
      let t0 = isFinite(m.t_start_years) ? m.t_start_years : (f.length ? f[0].t : -1);
      let t1 = m.length_myr ? t0 + m.length_myr * 1e6 : (f.length ? f[f.length - 1].t : 0);
      if (f.length) { t0 = Math.min(t0, f[0].t); t1 = Math.max(t1, f[f.length - 1].t); }
      if (!(t1 > t0)) t1 = t0 + 1;
      return [t0, t1];
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
  // The frame that shows the chosen layer at step n: n itself, else the nearest step that has it
  // (the deposits stage's last frame, for one, holds only deposit layers).
  function displayIndex(r, n) {
    const f = r.frames[n];
    if (!f || !layer || f.layers.includes(layer)) return n;
    for (let k = n - 1; k >= 0 && k >= n - 60; k--) if (r.frames[k] && r.frames[k].layers.includes(layer)) return k;
    for (let k = n + 1; k < r.frames.length && k <= n + 60; k++) if (r.frames[k] && r.frames[k].layers.includes(layer)) return k;
    return n;
  }

  // =================================================================================================
  // Panes, compare and the shared playhead
  // =================================================================================================
  const panes = [0, 1].map(i => {
    const el = $(i ? 'paneB' : 'paneA');
    const q = s => el.querySelector(s);
    return {
      i, el, run: null, cur: -1, drawn: null, boxLL: null, drag: null, pix: null, pixImg: null,
      frame: q('.mapframe'), canvas: q('canvas.map'), ctx: q('canvas.map').getContext('2d'),
      clock: q('.clock'), badge: q('.mapbadge'), waiting: q('.waiting'), axes: q('.axes'),
      pname: q('.pname'), pmeta: q('.pmeta'), plabel: q('.plabel'), readout: q('.readout'), xv: q('.xhair.v'), xh: q('.xhair.h'),
      rect: { x: 0, y: 0, w: 0, h: 0 },
    };
  });
  let focus = 0, comparing = false, cmpMode = store.get('cmpMode', 'split'), split = 0.5;
  let layer = null;
  let mode = 'live';           // 'live' (the newest frame), 'play' (at `rate`) or 'pause'
  let tp = 0;                  // the playhead while playing, years
  const RATES = [0.5, 1, 2, 5, 10, 20, 50, 100];
  let rate = 10;
  let mapDirty = true, tickerDirty = true, globeDirty = true, dockDirty = true, tlDirty = true, layersDirty = true, legendDirty = true;
  let tool = 'select', markersOn = true, gridOn = store.get('grid', true);
  let hiddenKinds = new Set(store.get('hiddenKinds', []));
  const P = () => panes[focus];
  function eventsDirty() { tickerDirty = globeDirty = mapDirty = tlDirty = true; }

  function hashState() {
    const h = new URLSearchParams();
    if (panes[0].run) h.set('a', panes[0].run.id);
    if (comparing && panes[1].run) h.set('b', panes[1].run.id);
    if (layer) h.set('layer', layer);
    if (comparing) h.set('cmp', cmpMode);
    history.replaceState(null, '', `${location.pathname}${location.search}#${h}`);
  }

  function open(i, id) {
    const p = panes[i];
    if (p.run && p.run.id === id) return;
    if (p.run) p.run.close();
    p.run = id ? new RunView(id) : null;
    p.cur = -1; p.drawn = null; p.boxLL = null; p.pix = null; p.pixImg = null;
    if (i === 1 && id && !comparing) setCompare(true);
    if (i === focus) { mode = 'live'; buttons(); texImg = null; G.tex = null; globeDirty = true; }
    tickerDirty = globeDirty = mapDirty = dockDirty = tlDirty = layersDirty = legendDirty = true;
    hashState();
    runsDirty = true;
    paneHead(p);
    stageEmpty();
    if (i === focus) runInfo();
  }

  function setCompare(on, m) {
    if (m) { cmpMode = m; store.set('cmpMode', m); }
    comparing = on;
    $('paneB').hidden = !on;
    const vp = $('viewport');
    vp.classList.toggle('comparing', on);
    vp.classList.toggle('cmp-split', on && cmpMode === 'split');
    vp.classList.toggle('cmp-side', on && cmpMode === 'side');
    for (const b of document.querySelectorAll('[data-cmp]')) b.setAttribute('aria-pressed', String(on ? b.dataset.cmp === cmpMode : b.dataset.cmp === 'off'));
    $('bpick').hidden = !on;
    if (!on && focus === 1) setFocus(0);
    if (!on && panes[1].run) { panes[1].run.close(); panes[1].run = null; }
    if (on && !panes[1].run) {
      const other = lastRuns.find(r => (!panes[0].run || r.id !== panes[0].run.id) && r.frames > 0);
      if (other) open(1, other.id);
    }
    for (const p of panes) paneHead(p);
    bOptions();
    layout();
    mapDirty = layersDirty = true;
    hashState();
    runsDirty = true;
  }

  function setFocus(i) {
    if (focus === i) return;
    focus = i;
    for (const p of panes) p.el.classList.toggle('focus', p.i === i);
    tickerDirty = globeDirty = mapDirty = dockDirty = tlDirty = legendDirty = true;
    texImg = null;
    runInfo();
    runhead();
  }

  // Something arrived for a run: redraw what shows it.
  function changed(run, frames) {
    for (const p of panes) {
      if (p.run !== run) continue;
      if (frames) {
        if (!layer && run.layers.length) layer = DEFAULT_LAYERS.find(n => run.layers.includes(n)) || run.layers[0];
        layersDirty = true;
        if (p.i === focus && p.cur < 0 && run.frames.length) { const n = run.frames.length - 1; if (ready(run.image(displayIndex(run, n)))) show(p, n); }
      }
      paneHead(p);
    }
    if (run === P().run) { dockDirty = tlDirty = true; runInfoSoon(); runhead(); }
    mapDirty = true;
    stageEmpty();
  }

  function show(p, n) {
    if (!p.run || n < 0 || n >= p.run.frames.length) return;
    if (n !== p.cur && p.i === focus) { tickerDirty = globeDirty = tlDirty = legendDirty = true; }
    p.cur = n;
    mapDirty = true;
    if (p.i === focus) dockDirty = true;
  }

  function focusT() {
    const p = P();
    return p.run && p.cur >= 0 && p.run.frames[p.cur] ? p.run.frames[p.cur].t : null;
  }

  function setMode(m) {
    mode = m;
    const t = focusT();
    if (t !== null) tp = t;
    buttons();
  }

  let lastTs = null, tvIdleSince = null;
  function tick(ts) {
    try { frame(ts); } catch (e) { console.error(e); }
    requestAnimationFrame(tick);
  }
  function frame(ts) {
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
        for (let k = 1; k <= 4 && want + k <= last; k++) r.image(displayIndex(r, want + k));   // prefetch
      }
      if (want !== p.cur) {
        // Switch only once the image is here, so the map never goes blank; hold the playhead.
        if (ready(r.image(displayIndex(r, want)))) show(p, want);
        else if (mode === 'play') tp = Math.min(tp, r.frames[want].t);
      }
      if (TV) tvIdle(p, r);
    }
    // The other pane shows its frame at the focused pane's time.
    const t = focusT();
    for (const o of panes) {
      if (o === p || !o.run || !o.run.frames.length || (o.i === 1 && !comparing)) continue;
      const want = t === null ? o.run.frames.length - 1 : o.run.frameAt(t);
      if (want !== o.cur && ready(o.run.image(displayIndex(o.run, want)))) { o.cur = want; mapDirty = true; }
    }
    if (mapDirty) { mapDirty = false; for (const q of panes) if (q.i === 0 || comparing) drawPane(q); }
    if (tickerDirty) { tickerDirty = false; ticker(); tvEvents(); }
    if (layersDirty) { layersDirty = false; layerPanel(); }
    if (legendDirty) { legendDirty = false; legend(); }
    if (dockDirty) { dockDirty = false; label(); }
    if (tlDirty) { tlDirty = false; drawTimeline(); }
    spin(dt);
  }

  // The TV replays the last run while nothing new runs, so the screen is never a still.
  function tvIdle(p, r) {
    if (!r.finished() || mode === 'play') { tvIdleSince = null; return; }
    if (tvIdleSince === null) { tvIdleSince = performance.now(); return; }
    if (performance.now() - tvIdleSince > 25000) {
      tvIdleSince = null;
      rate = 10; buttons();
      show(p, 0); setMode('play');
    }
  }

  // =================================================================================================
  // Layout of the stage: one map, a split view, or two maps side by side (or stacked)
  // =================================================================================================
  function layout() {
    const vp = $('viewport');
    const mob = narrow.matches;
    const side = comparing && cmpMode === 'side';
    if (mob) {
      const w = vp.clientWidth;
      const one = (w - 16) / 2;
      vp.style.setProperty('--vp-h', `${Math.round(side ? 2 * one + 16 + 2 * 28 + 8 : one + 16)}px`);
    }
    const W = vp.clientWidth, H = vp.clientHeight;
    if (!W || !H) return;
    const axes = !TV && !mob;
    const m = TV ? { l: 0, r: 0, t: 0, b: 0 } : mob ? { l: 8, r: 8, t: 8, b: 8 } : { l: 48, r: 22, t: 18, b: 32 };
    const head = side ? 30 : 0;
    const fit = (x, y, w, h) => { const mw = Math.max(40, Math.min(w, 2 * h)); const mh = mw / 2; return [x + (w - mw) / 2, y + (h - mh) / 2, mw, mh]; };
    const place = (p, [x, y, w, h]) => {
      p.rect = { x, y, w, h };
      Object.assign(p.el.style, { left: `${x}px`, top: `${y}px`, width: `${w}px`, height: `${h}px` });
    };
    if (!side) {
      const r = fit(m.l, m.t, W - m.l - m.r, H - m.t - m.b);
      place(panes[0], r);
      if (comparing) place(panes[1], r);
    } else {
      const gap = 20;
      // Two columns or two rows, whichever gives the larger maps.
      const cw = (W - gap) / 2, rh = (H - gap) / 2;
      const colMap = fit(0, 0, cw - m.l - m.r, H - m.t - m.b - head)[2];
      const rowMap = fit(0, 0, W - m.l - m.r, rh - m.t - m.b - head)[2];
      if (colMap >= rowMap) {
        place(panes[0], fit(m.l, m.t + head, cw - m.l - m.r, H - m.t - m.b - head));
        place(panes[1], fit(cw + gap + m.l, m.t + head, cw - m.l - m.r, H - m.t - m.b - head));
      } else {
        place(panes[0], fit(m.l, m.t + head, W - m.l - m.r, rh - m.t - m.b - head));
        place(panes[1], fit(m.l, rh + gap + m.t + head, W - m.l - m.r, rh - m.t - m.b - head));
      }
    }
    for (const p of panes) {
      p.el.classList.toggle('noaxes', !axes || (comparing && cmpMode === 'split' && p.i === 1));
      p.axes.hidden = !axes || (comparing && cmpMode === 'split' && p.i === 1);
      p.el.querySelector('.phead').hidden = !side;
      p.plabel.hidden = !(comparing && cmpMode === 'split');
      p.clock.hidden = comparing && cmpMode === 'split' && p.i === 1;
    }
    const sp = $('splitter');
    sp.hidden = !(comparing && cmpMode === 'split');
    if (!sp.hidden) {
      const r = panes[0].rect;
      Object.assign(sp.style, { left: `${r.x + split * r.w}px`, top: `${r.y}px`, height: `${r.h}px` });
      panes[1].el.style.clipPath = `inset(-40px -40px -40px ${split * r.w}px)`;
    } else panes[1].el.style.clipPath = '';
    fitStagebar();
    mapDirty = true;
  }
  // The stage's toolbar drops its button labels (icons and tooltips stay) before it wraps.
  function fitStagebar() {
    const sb = $('stagebar');
    const was = sb.className;
    sb.classList.remove('compact', 'compact2');
    if (sb.scrollHeight > 56) sb.classList.add('compact');
    if (sb.scrollHeight > 56) sb.classList.add('compact2');
    if (sb.className !== was) requestAnimationFrame(layout);
  }
  function buildAxes() {
    for (const p of panes) {
      let h = '';
      for (const lat of [60, 30, 0, -30, -60]) h += `<span class="lat" style="top:${(90 - lat) / 180 * 100}%">${lat === 0 ? '0°' : `${Math.abs(lat)}°${lat > 0 ? 'N' : 'S'}`}</span>`;
      for (let lon = 0; lon <= 360; lon += 60) h += `<span class="lon${lon % 360 === 0 ? ' seam' : ''}" style="left:${lon / 360 * 100}%">${lon % 360 === 0 ? 'seam' : `${lon}°`}</span>`;
      p.axes.innerHTML = h;
    }
  }
  new ResizeObserver(() => { layout(); tlDirty = true; legendDirty = true; globeSize(); }).observe($('viewport'));
  window.addEventListener('resize', () => { layout(); tlDirty = true; globeDirty = true; });

  // Split view: drag the divider (or use the arrow keys on it).
  {
    const sp = $('splitter');
    let drag = false;
    const at = ev => { const r = panes[0].rect, vr = $('viewport').getBoundingClientRect(); split = clamp((ev.clientX - vr.left - r.x) / r.w, 0.02, 0.98); layout(); };
    sp.addEventListener('pointerdown', ev => { drag = true; sp.setPointerCapture(ev.pointerId); ev.preventDefault(); });
    sp.addEventListener('pointermove', ev => { if (drag) at(ev); });
    sp.addEventListener('pointerup', () => { drag = false; });
    sp.addEventListener('keydown', ev => {
      if (ev.key === 'ArrowLeft' || ev.key === 'ArrowRight') { split = clamp(split + (ev.key === 'ArrowLeft' ? -0.05 : 0.05), 0.02, 0.98); layout(); ev.preventDefault(); ev.stopPropagation(); }
    });
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
  function llToCanvas(p, lat, lon) { return [(((lon % 360) + 360) % 360) / 360 * p.canvas.width, (90 - lat) / 180 * p.canvas.height]; }
  function mapLatLon(p, x, y) {
    return [clamp(90 - y / p.canvas.height * 180, -90, 90), ((x / p.canvas.width * 360) % 360 + 360) % 360];
  }
  // Block coordinates of a point (plan §5.2; the static viewer's inspect): x = 360°-longitude
  // scaled to C, from the seam at −C/2; north is −z.
  function blockXZ(run, lat, lon) {
    const C = (run && run.meta && run.meta.circumference) || 16384;
    return [Math.round(lon / 360 * C - C / 2), Math.round(-lat / 360 * C)];
  }
  const circ = run => (run && run.meta && run.meta.circumference) || 16384;
  function visibleEvents(run, t) {
    const out = [];
    for (const e of run.events) {
      if (e.t_start > t || hiddenKinds.has(shortKind(e.kind))) continue;
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
    const cw = Math.max(1, Math.round(c.clientWidth * dpr)), ch = Math.max(1, Math.round(c.clientHeight * dpr));
    if (c.width !== cw || c.height !== ch) { c.width = cw; c.height = ch; }
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle = '#04060a';
    ctx.fillRect(0, 0, c.width, c.height);
    const r = p.run;
    const has = !!(r && r.frames.length);
    p.waiting.hidden = has || (!r && p.i === 0);
    if (!r && p.i === 1) p.waiting.innerHTML = `${ic('split', 'ic-l')}<div>Choose run B</div><div class="sub">Use the B menu above the map, or Compare on a run card.</div>`;
    if (r && !has) {
      const st = r.status ? r.status.state : '';
      const [title, sub, spin] = !r.connected && !r.meta ? ['Connecting…', 'Opening the run’s stream.', true]
        : st === 'queued' ? ['Queued', 'It starts when the run before it ends. One run at a time on the Mac.', false]
          : st === 'running' || st === 'finalizing' ? ['Starting…', 'Waiting for the first step of the simulation.', true]
            : [`No frames (${st || 'unknown'})`, (r.status && r.status.message) || 'This run made no frames.', false];
      p.waiting.innerHTML = `${spin ? '<div class="spinner l"></div>' : ic(st === 'queued' ? 'clock' : 'alert', 'ic-l')}<div>${esc(title)}</div><div class="sub">${esc(sub)}</div>`;
    }
    if (!has || p.cur < 0 || !r.frames[p.cur]) { p.clock.innerHTML = ''; badge(p); drawGrid(p); p.frame.classList.remove('loading'); return; }
    const f = r.frames[p.cur];
    const dn = displayIndex(r, p.cur), fd = r.frames[dn];
    const img = r.image(dn);
    if (ready(img)) p.drawn = img;
    p.frame.classList.toggle('loading', !ready(img));
    // Continuous layers upscale smoothly; categorical ones (plates, Köppen, rock) keep hard edges.
    const dli = r.layerIndex(fd), dinfo = rampInfo(fd.ramps && fd.ramps[dli]);
    ctx.imageSmoothingEnabled = !!dinfo && !dinfo.cat && c.width > fd.w * 1.2;
    ctx.imageSmoothingQuality = 'high';
    if (p.drawn) ctx.drawImage(p.drawn, 0, 0, c.width, c.height);
    ctx.imageSmoothingEnabled = false;
    if (p.i === focus && p.drawn && p.drawn !== texImg && ready(p.drawn)) { texImg = p.drawn; G.tex = panePixels(p); globeDirty = true; }
    drawGrid(p);
    if (markersOn) {
      const t = f.t, t0 = p.cur > 0 ? r.frames[p.cur - 1].t : -Infinity, rad = (TV ? 4.2 : 3) * dpr;
      const vis = visibleEvents(r, t);
      ctx.lineWidth = dpr;
      for (const { e, ll, age } of vis) {
        if (e.t_start > t0) continue;
        const [x, y] = mapXY(ll[0], ll[1], f, c.width);
        ctx.globalAlpha = 1 - 0.72 * age / FADE_YEARS;
        ctx.beginPath(); ctx.arc(x, y, rad, 0, 2 * Math.PI);
        ctx.fillStyle = kindColour(e.kind); ctx.fill();
        ctx.strokeStyle = 'rgba(0,0,0,.7)'; ctx.stroke();
      }
      ctx.globalAlpha = 1;
      for (const { e, ll } of vis) {
        if (!(e.t_start > t0)) continue;
        const [x, y] = mapXY(ll[0], ll[1], f, c.width), col = kindColour(e.kind);
        ctx.save();
        ctx.shadowColor = col; ctx.shadowBlur = 14 * dpr;
        ctx.beginPath(); ctx.arc(x, y, 2 * rad, 0, 2 * Math.PI);
        ctx.fillStyle = col; ctx.fill();
        ctx.restore();
        ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.stroke();
        ctx.beginPath(); ctx.arc(x, y, 3.6 * rad, 0, 2 * Math.PI);
        ctx.lineWidth = dpr; ctx.strokeStyle = 'rgba(255,255,255,.55)'; ctx.stroke();
      }
      if (selectedEvent && p.i === focus) {
        const e = r.byId.get(selectedEvent), ll = e && placeLatLon(e);
        if (ll) {
          const [x, y] = mapXY(ll[0], ll[1], f, c.width);
          ctx.beginPath(); ctx.arc(x, y, 5 * rad, 0, 2 * Math.PI);
          ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.setLineDash([4 * dpr, 3 * dpr]); ctx.stroke(); ctx.setLineDash([]);
        }
      }
    }
    drawDeposits(p, r, f);
    if (pin && pin.pane === p.i) {
      const [x, y] = llToCanvas(p, pin.lat, pin.lon);
      ctx.beginPath(); ctx.arc(x, y, 7 * dpr, 0, 2 * Math.PI);
      ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.stroke();
      ctx.beginPath(); ctx.arc(x, y, 2.2 * dpr, 0, 2 * Math.PI); ctx.fillStyle = '#fff'; ctx.fill();
    }
    const box = p.drag && p.drag.box ? p.drag.box : p.boxLL;
    if (box) drawBox(p, box, !!(p.drag && p.drag.box));
    // The clock.
    const { label: ln } = layerLabel(layer);
    let sub = `${esc(stageText(f.stage))} · step ${f.n + 1}`;
    if (dn !== p.cur) sub += ` · <span class="warnc">${esc(ln)} from step ${fd.n + 1}</span>`;
    else if (layer && !f.layers.includes(layer)) sub += ` · <span class="warnc">no ${esc(ln)} here</span>`;
    p.clock.innerHTML = `<div class="t">${tHTML(f.t)}</div><div class="s">${sub}</div>`;
    badge(p);
    if (p.i === focus) tvHud(p, f);
  }
  const stageText = s => String(s || '').split('+').join(' + ');

  function drawGrid(p) {
    if (!gridOn || TV) return;
    const c = p.canvas, ctx = p.ctx, dpr = window.devicePixelRatio || 1, W = c.width, H = c.height;
    ctx.save();
    ctx.lineWidth = dpr;
    for (let lat = -60; lat <= 60; lat += 30) {
      const y = Math.round((90 - lat) / 180 * H) + 0.5;
      ctx.strokeStyle = lat === 0 ? 'rgba(255,255,255,.2)' : 'rgba(255,255,255,.09)';
      ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(W, y); ctx.stroke();
    }
    ctx.strokeStyle = 'rgba(255,255,255,.09)';
    for (let lon = 30; lon < 360; lon += 30) {
      const x = Math.round(lon / 360 * W) + 0.5;
      ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, H); ctx.stroke();
    }
    // The seam (world x = ±C/2) is the map's left and right edge.
    ctx.strokeStyle = 'rgba(255,122,122,.55)'; ctx.lineWidth = 2 * dpr; ctx.setLineDash([5 * dpr, 4 * dpr]);
    ctx.beginPath(); ctx.moveTo(dpr, 0); ctx.lineTo(dpr, H); ctx.moveTo(W - dpr, 0); ctx.lineTo(W - dpr, H); ctx.stroke();
    ctx.restore();
  }

  function drawBox(p, b, live) {
    const ctx = p.ctx, dpr = window.devicePixelRatio || 1;
    const [x0, y0] = llToCanvas(p, b.north, b.west), [x1, y1] = llToCanvas(p, b.south, b.east);
    const x = Math.min(x0, x1), y = Math.min(y0, y1), w = Math.max(3 * dpr, Math.abs(x1 - x0)), h = Math.max(3 * dpr, Math.abs(y1 - y0));
    ctx.save();
    ctx.fillStyle = 'rgba(125,140,255,.18)'; ctx.fillRect(x, y, w, h);
    ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.setLineDash([6 * dpr, 4 * dpr]); ctx.strokeRect(x, y, w, h); ctx.setLineDash([]);
    if (b.text) {
      ctx.font = `600 ${11 * dpr}px Inter, system-ui, sans-serif`;
      const tw = ctx.measureText(b.text).width + 12 * dpr, th = 20 * dpr;
      const tx = clamp(x, 0, p.canvas.width - tw), ty = y - th - 4 * dpr >= 0 ? y - th - 4 * dpr : y + h + 4 * dpr;
      ctx.fillStyle = live && b.cut ? 'rgba(244,183,64,.95)' : 'rgba(12,16,22,.9)';
      ctx.beginPath(); ctx.roundRect ? ctx.roundRect(tx, ty, tw, th, 5 * dpr) : ctx.rect(tx, ty, tw, th); ctx.fill();
      ctx.fillStyle = live && b.cut ? '#1a1204' : '#fff'; ctx.textBaseline = 'middle'; ctx.fillText(b.text, tx + 6 * dpr, ty + th / 2);
    }
    ctx.restore();
  }

  function paneHead(p) {
    const r = p.run;
    const name = r ? (r.meta ? r.meta.label : r.id) : (p.i ? 'Choose run B' : 'No run');
    p.pname.textContent = name;
    p.plabel.querySelector('.n').textContent = name;
    const bits = [];
    if (r && r.meta) bits.push(presetName(r.meta.preset), startName(r.meta.start), r.meta.length_myr ? `${fmtInt(r.meta.length_myr)} Myr` : '');
    if (r && r.status) bits.push(r.status.state);
    p.pmeta.textContent = bits.filter(Boolean).join(' · ');
  }

  function badgeState(p) {
    const r = p.run;
    if (!r || p.cur < 0) return null;
    if (p.i !== focus) return ['s-idle', 'Same time'];
    if (r.placeholder()) return ['s-queued', 'Placeholder · not the real simulation'];
    if (mode === 'live' && !r.finished()) return ['s-live', 'Live'];
    if (mode === 'live') return r.status.state === 'done' ? ['s-done', 'End of run'] : [`s-${r.status.state}`, r.status.state];
    if (mode === 'play') return ['s-idle', `Replay · ${rate} Myr/s`];
    return ['s-idle', 'Paused'];
  }
  function badge(p) {
    const b = badgeState(p);
    const html = b ? `<span class="badge ${b[0]}">${b[0] === 's-live' ? '<span class="sdot s-live"></span>' : ''}${esc(b[1])}</span>` : '';
    if (p.badge.dataset.h !== html) { p.badge.dataset.h = html; p.badge.innerHTML = html; }
    p.badge.hidden = comparing && cmpMode === 'split' && p.i === 0;
  }

  // Pixels of the image a pane shows (for the globe's texture and the value under the pointer).
  const off = document.createElement('canvas');
  function pixels(img) {
    off.width = img.naturalWidth; off.height = img.naturalHeight;
    const o = off.getContext('2d', { willReadFrequently: true });
    o.clearRect(0, 0, off.width, off.height);
    o.drawImage(img, 0, 0);
    return { w: off.width, h: off.height, d: o.getImageData(0, 0, off.width, off.height).data };
  }
  function panePixels(p) {
    if (!p.drawn || !ready(p.drawn)) return null;
    if (p.pixImg !== p.drawn) { p.pix = pixels(p.drawn); p.pixImg = p.drawn; }
    return p.pix;
  }
  function sampleValue(p, lat, lon) {
    const r = p.run;
    if (!r || p.cur < 0) return null;
    const pix = panePixels(p);
    if (!pix) return null;
    const dn = displayIndex(r, p.cur), f = r.frames[dn];
    const li = r.layerIndex(f), name = f.layers[li], info = rampInfo(f.ramps && f.ramps[li]);
    const px = Math.min(pix.w - 1, Math.floor((((lon % 360) + 360) % 360) / 360 * pix.w));
    const py = clamp(Math.floor((90 - lat) / 180 * pix.h), 0, pix.h - 1);
    const q = (py * pix.w + px) * 4;
    const rgb = [pix.d[q], pix.d[q + 1], pix.d[q + 2]];
    if (pix.d[q + 3] === 0) return { name, rgb, nodata: true };
    return { name, rgb, info, val: valueAt(info, rgb) };
  }
  function valueText(s) {
    if (!s) return '';
    if (s.nodata) return 'no data';
    if (!s.val) return '';
    // The satellite view is true colour, so the nearest key colour is only what the spot looks like.
    if (s.val.cls && s.info && s.info.name === 'satellite') return `looks like ${(s.val.cls.label || '').toLowerCase()}`;
    if (s.val.cls) return s.val.cls.label ? `${s.val.cls.label}` : `class ${s.val.cls.v}`;
    const { unit } = layerLabel(s.name);
    return `≈ ${numText(s.val.v)}${unit && unit !== 'block y' ? ` ${unit}` : ''}`;
  }

  // Map pointer: hover reads the point; the tool decides what a click or a drag does.
  for (const p of panes) {
    const at = ev => { const rect = p.canvas.getBoundingClientRect(), d = window.devicePixelRatio || 1; return [(ev.clientX - rect.left) * d, (ev.clientY - rect.top) * d, ev.clientX - rect.left, ev.clientY - rect.top]; };
    const hover = ev => {
      if (TV) return;
      const [x, y, cx, cy] = at(ev);
      const r = p.run;
      if (!r || p.cur < 0) { p.readout.hidden = true; return; }
      const [lat, lon] = mapLatLon(p, x, y);
      const [bx, bz] = blockXZ(r, lat, lon);
      p.xv.hidden = p.xh.hidden = false;
      p.xv.style.left = `${cx}px`; p.xh.style.top = `${cy}px`;
      const s = sampleValue(p, lat, lon);
      const { label: ln } = layerLabel(s ? s.name : layer);
      const v = valueText(s);
      p.readout.hidden = false;
      p.readout.innerHTML = `<b>${latText(lat)}</b> ${lonText(lon)}<span class="sep"></span>x <b>${fmtInt(bx)}</b> z <b>${fmtInt(bz)}</b>` +
        (v ? `<span class="sep"></span><span class="sw" style="background:rgb(${s.rgb.join(',')})"></span>${esc(ln)} <b>${esc(v)}</b>` : '');
      if (comparing && cmpMode === 'split') p.readout.style.left = p.i === 1 ? 'auto' : '', p.readout.style.right = p.i === 1 ? '12px' : '';
      if (p.drag && tool === 'area') {
        const [la0, lo0] = mapLatLon(p, Math.min(p.drag.x, x), Math.min(p.drag.y, y));
        const [la1, lo1] = mapLatLon(p, Math.max(p.drag.x, x), Math.max(p.drag.y, y));
        if (Math.abs(x - p.drag.x) + Math.abs(y - p.drag.y) > 6) { p.drag.box = boxInfo(r, la0, la1, lo0, lo1); mapDirty = true; }
      }
    };
    p.canvas.addEventListener('pointerdown', ev => {
      if (TV) return;
      setFocus(p.i);
      const [x, y] = at(ev);
      p.drag = { x, y, box: null };
      if (tool === 'area') p.canvas.setPointerCapture(ev.pointerId);
    });
    p.canvas.addEventListener('pointermove', hover);
    p.canvas.addEventListener('pointerleave', () => { if (!p.drag) { p.readout.hidden = true; p.xv.hidden = p.xh.hidden = true; } });
    p.canvas.addEventListener('pointerup', ev => {
      if (!p.drag) return;
      const [x, y] = at(ev);
      const moved = Math.abs(x - p.drag.x) + Math.abs(y - p.drag.y) > 6;
      const drag = p.drag;
      p.drag = null;
      const r = p.run;
      if (!r || p.cur < 0) return;
      const f = r.frames[p.cur];
      const [lat, lon] = mapLatLon(p, x, y);
      if (tool === 'area') {
        let b = drag.box;
        if (!moved || !b) {   // a click: the largest box a preview makes, 256 × 256 blocks around it
          const d = 128 / circ(r) * 360;
          b = boxInfo(r, clamp(lat + d, -90, 90), clamp(lat - d, -90, 90), lon - d, lon + d);
        }
        p.boxLL = b; mapDirty = true;
        const [x0, z0] = blockXZ(r, b.north, b.west), [x1, z1] = blockXZ(r, b.south, b.east);
        Stage2.preview(r, f, { x0, z0, x1, z1 }, { north: b.north, south: b.south, west: b.west, east: b.east }, b.text);
        return;
      }
      if (tool === 'inspect') {
        G.lon0 = lon; G.lat0 = clamp(lat, -80, 80); G.spin = false; spinButton(); globeDirty = true;
        pin = { pane: p.i, lat, lon }; mapDirty = true;
        const [bx, bz] = blockXZ(r, lat, lon);
        Stage2.inspect(r, f, bx, bz, lat, lon);
        return;
      }
      // Select: the nearest event marker, else the point itself.
      const d = window.devicePixelRatio || 1;
      let best = null, bd = (12 * d) ** 2;
      if (markersOn) {
        for (const { e, ll } of visibleEvents(r, f.t)) {
          const [ex, ey] = mapXY(ll[0], ll[1], f, p.canvas.width);
          const dd = (ex - x) ** 2 + (ey - y) ** 2;
          if (dd < bd) { bd = dd; best = e; }
        }
      }
      if (best) { showEvent(r, best.id); return; }
      showPoint(p, lat, lon);
    });
  }
  // A box's lat/lon bounds with its size in blocks (a preview is at most 256 × 256; larger is cut).
  function boxInfo(r, north, south, west, east) {
    const C = circ(r);
    const w = Math.round(Math.abs(east - west) / 360 * C), h = Math.round(Math.abs(north - south) / 360 * C);
    const cut = w > 256 || h > 256;
    return { north, south, west, east, w, h, cut, text: `${fmtInt(w)} × ${fmtInt(h)} blocks${cut ? ' → cut to 256 × 256' : ''}` };
  }

  // =================================================================================================
  // The deposits overlay (Ben, 2026-10-01): the run's coal fields and ore deposits by family, drawn
  // over whatever layer the map shows. A view of data the run keeps: the sites the deposit stage
  // placed (`GET api/runs/<id>/deposits`, from the world in memory while the run goes on and from
  // the finished run's bake after) and the events the page already holds for each site's age and
  // "why here". Drawing, hit-testing, the legend and the texts are `DTDeposits` (deposits.js); this
  // is the wiring to the panes, the legend card and the inspector. It is an overlay, not a layer:
  // the keyframe layers are images the core draws on the stepping thread, which cannot carry
  // symbols to hover or click, and an overlay works over every base layer, the new ones too.
  // =================================================================================================
  const DEP_LAYER = 'deposits/sites';
  let depositsOn = store.get('deposits', true);
  let depHidden = new Set(store.get('depHidden', []));
  // A frame names the sites layer: the deposit stage has run, and its sites are fetched once (again
  // when the run is done, for the baked ones).
  function depFrame(r, f) {
    if (!f.layers.includes(DEP_LAYER)) return;
    if (r.depFrom < 0 || f.n < r.depFrom) { r.depFrom = f.n; depDirty(); }
    if (!r.depState) loadDeposits(r);
  }
  function depStatus(r) {
    if (r.depFrom >= 0 && r.status && r.status.state === 'done' && r.depSource !== 'bake') loadDeposits(r);
  }
  function depDirty() { mapDirty = legendDirty = true; }
  async function loadDeposits(r) {
    if (r.depState === 'loading') { r.depAgain = true; return; }
    r.depState = 'loading';
    const res = await api(`api/runs/${encodeURIComponent(r.id)}/deposits`);
    if (res.ok && res.body.available) {
      r.deposits = DTDeposits.prepare(res.body); r.depSource = res.body.source; r.depState = 'ready';
    } else r.depState = res.ok ? 'none' : 'failed';
    depDirty(); buttons();
    if (r.depAgain) { r.depAgain = false; loadDeposits(r); }
  }
  // The sites show on the frames where the deposit stage has run.
  const depShown = (r, n) => !!r && !!r.deposits && r.depFrom >= 0 && n >= r.depFrom;
  // The map's pixels for a lat/lon, as the event markers use them.
  function depView(p, f) {
    const c = p.canvas;
    return { project: (lat, lon) => mapXY(lat, lon, f, c.width), ppdx: c.width / 360, ppdy: c.height / 180, worldW: c.width, dpr: window.devicePixelRatio || 1 };
  }
  // The site whose forming event the inspector shows is the selected one.
  const depSelected = (p, r) => { const s = p.i === focus && selectedEvent ? r.deposits.byEvent.get(selectedEvent) : null; return s ? s.id : null; };
  function drawDeposits(p, r, f) {
    if (!depositsOn || !depShown(r, p.cur)) return;
    DTDeposits.draw(p.ctx, r.deposits, depView(p, f), { hidden: depHidden, hover: p.depHover ?? null, selected: depSelected(p, r) });
  }
  const depAge = (r, s) => { const e = r.byId.get(s.forming_event); return e ? fmtT(e.t_start) : ''; };

  // The toggle and the legend.
  function depButton() {
    const b = $('depbtn');
    const r = P().run;
    b.setAttribute('aria-pressed', String(depositsOn));
    const none = !r || !r.deposits;
    b.dataset.tip = none ? 'Deposits: coal fields and ore sites by family (they appear when the deposit stage has placed them)' : 'Deposits: coal fields and ore sites by family, over any layer';
    b.classList.toggle('dim', none);
  }
  $('depbtn').addEventListener('click', () => { depositsOn = !depositsOn; store.set('deposits', depositsOn); depDirty(); buttons(); });
  function depLegend() {
    const box = $('deplegend'), p = P(), r = p.run;
    if (!depositsOn || !r || !r.deposits || r.depFrom < 0) { box.hidden = true; box.dataset.key = ''; return; }
    const pending = p.cur < r.depFrom;
    const key = [r.id, r.depSource, r.deposits.sites.length, [...depHidden].join(), pending].join('|');
    box.hidden = false;
    if (box.dataset.key === key) return;
    box.dataset.key = key;
    const note = pending ? `The deposit stage places the sites at the end of the run; they show from step ${r.depFrom + 1}.` : undefined;
    box.innerHTML = DTDeposits.legendHTML(r.deposits, depHidden, { note });
  }
  $('deplegend').addEventListener('click', ev => {
    const note = ev.target.closest('.note');
    if (note) { note.classList.toggle('open'); return; }
    const b = ev.target.closest('[data-fam]');
    if (!b) return;
    const k = b.dataset.fam;
    if (ev.altKey || ev.metaKey) {   // alone: hide every other family
      const all = [...b.parentElement.children].map(x => x.dataset.fam);
      depHidden = new Set(all.filter(x => x !== k));
    } else if (depHidden.has(k)) depHidden.delete(k); else depHidden.add(k);
    store.set('depHidden', [...depHidden]);
    depDirty();
  });

  // Hover (a card with the family, size and age) and click (the inspector, "why here").
  for (const p of panes) {
    const at = ev => { const rect = p.canvas.getBoundingClientRect(), d = window.devicePixelRatio || 1; return [(ev.clientX - rect.left) * d, (ev.clientY - rect.top) * d, ev.clientX - rect.left, ev.clientY - rect.top]; };
    const siteAt = ev => {
      const r = p.run;
      if (TV || !depositsOn || !r || p.cur < 0 || !depShown(r, p.cur) || !r.frames[p.cur]) return null;
      const [x, y] = at(ev);
      return DTDeposits.hit(r.deposits, depView(p, r.frames[p.cur]), x, y, { hidden: depHidden });
    };
    const tip = () => {
      if (!p.depTip) { p.depTip = document.createElement('div'); p.depTip.className = 'dep-tip'; p.depTip.hidden = true; p.frame.appendChild(p.depTip); }
      return p.depTip;
    };
    const leave = () => { if (p.depHover != null) { p.depHover = null; mapDirty = true; } p.canvas.style.cursor = ''; if (p.depTip) p.depTip.hidden = true; };
    p.canvas.addEventListener('pointermove', ev => {
      if (p.drag && p.drag.box) { leave(); return; }
      const s = siteAt(ev);
      if (!s) { leave(); return; }
      if (p.depHover !== s.id) { p.depHover = s.id; mapDirty = true; }
      p.canvas.style.cursor = tool === 'select' ? 'pointer' : '';
      const t = tip(), [, , cx, cy] = at(ev);
      t.innerHTML = DTDeposits.tipHTML(s, depAge(p.run, s));
      t.hidden = false;
      const w = t.offsetWidth, h = t.offsetHeight, fw = p.frame.clientWidth, fh = p.frame.clientHeight;
      t.style.left = `${clamp(cx + 16 + w > fw ? cx - 16 - w : cx + 16, 4, Math.max(4, fw - w - 4))}px`;
      t.style.top = `${clamp(cy + 16 + h > fh ? cy - 12 - h : cy + 16, 4, Math.max(4, fh - h - 4))}px`;
    });
    p.canvas.addEventListener('pointerleave', leave);
    // A click with the Select tool on a deposit opens it; this runs before the map's own click
    // (a point's values, the nearest event) and takes the click when it is a hit.
    let down = null;
    p.canvas.addEventListener('pointerdown', ev => { down = [ev.clientX, ev.clientY]; }, true);
    p.canvas.addEventListener('pointerup', ev => {
      if (tool !== 'select' || !down || Math.abs(ev.clientX - down[0]) + Math.abs(ev.clientY - down[1]) > 6) return;
      const s = siteAt(ev);
      if (!s) return;
      ev.stopImmediatePropagation();
      p.drag = null;
      setFocus(p.i);
      showSite(p.run, s);
    }, true);
  }

  // A site in the inspector: its card and "why here" (the forming event and its causes, up to four
  // links deep, as `deeptime inspect-column --why` lists them), then the forming event itself, as
  // the Events tab shows any event. Chain links open the events; the kin links open other sites.
  function showSite(r, s) {
    const model = r.deposits, e = r.byId.get(s.forming_event);
    const where = latLonText(s.lat, s.lon), age = depAge(r, s);
    const head = `<div class="dh" style="--c:${s.family.ui || s.family.colour}"><span class="d"></span><div><h4>${esc(s.family.label)}</h4><div class="id">site ${s.id} · ${esc(s.type)}</div></div></div>`;
    const kv = `<div class="kv">${DTDeposits.factRows(s, model, { where, age }).map(([a, b]) => `<div>${esc(a)}</div><div>${b}</div>`).join('')}</div>`;
    const kin = [];
    const parent = s.parent != null ? model.byId.get(s.parent) : null;
    const kids = model.sites.filter(o => o.parent === s.id);
    const siteBtn = o => `<button data-site="${o.id}" style="--c:${o.family.ui || o.family.colour}"><i></i>${esc(o.family.label)}<span class="t">site ${o.id}</span></button>`;
    if (parent) kin.push(`<h5>Came from</h5><div class="chain">${siteBtn(parent)}</div>`);
    if (kids.length) kin.push(`<h5>Led to</h5><div class="chain">${kids.map(siteBtn).join('')}</div>`);
    let why;
    if (e) {
      const chain = DTDeposits.causeChain(r.byId, e.id, 4);
      why = `<h5>Why here</h5><div class="dep-why"><div class="chain"><div class="lv"><span class="lvn">0</span>${evLink(r, e.id)}</div>` +
        chain.map(c => `<div class="lv" style="padding-left:${(c.depth - 1) * 10}px"><span class="lvn">${c.depth}</span>${evLink(r, c.id)}</div>`).join('') +
        `</div></div><div class="desc" style="margin-top:6px">Level 0 formed the deposit; each level below is a cause of the one above it.</div>`;
    } else why = '<h5>Why here</h5><div class="chain"><span class="none">The forming event is not in this run’s event log.</span></div>';
    const acts = `<div class="acts"><button class="btn btn-sm" data-sitegoto="${s.id}">${ic('target', 'ic-s')}Turn the globe here</button></div>`;
    const card = head + kv + acts + why + kin.join('') + (e ? '<h5>Forming event</h5>' : '');
    if (e) {
      showEvent(r, e.id);
      $('details').insertAdjacentHTML('afterbegin', card);
    } else {
      selectedEvent = null; pin = null;
      $('details').innerHTML = card;
      $('details').dataset.curEv = '';
      setITab('details', true);
      if (narrow.matches) setMTab('inspect');
      mapDirty = true; evShownKey = ''; tickerDirty = true;
    }
    G.lon0 = s.lon; G.lat0 = clamp(s.lat, -80, 80); G.spin = false; spinButton(); globeDirty = true;
  }
  $('details').addEventListener('click', ev => {
    const r = P().run;
    const kin = ev.target.closest('[data-site]'), go = ev.target.closest('[data-sitegoto]');
    if (!r || !r.deposits || (!kin && !go)) return;
    const s = r.deposits.byId.get(Number(kin ? kin.dataset.site : go.dataset.sitegoto));
    if (!s) return;
    if (kin) showSite(r, s);
    else { G.lon0 = s.lon; G.lat0 = clamp(s.lat, -80, 80); G.spin = false; spinButton(); globeDirty = true; }
  });

  // =================================================================================================
  // Stage 2: the 3D block preview of an area and click-to-inspect (tools/workbench/preview.py,
  // forwarded by the server). Both answer a job: {job, status_url, note} → poll → {state, url}.
  // Without --preview both answer 501 {"available": false, "message"}.
  // =================================================================================================
  const S2_STEPS = [['queued', 'Queued'], ['waiting', 'Waiting for the server lock'], ['installing', 'Installing the server'],
    ['preparing', 'Preparing'], ['booting', 'Booting Minecraft'], ['generating', 'Generating chunks'], ['probing', 'Probing'],
    ['stopping', 'Saving the world'], ['exporting', 'Exporting blocks'], ['done', 'Ready']];
  const Stage2 = {
    jobs: [], seq: 0,
    async inspect(run, frame, x, z, lat, lon) {
      const job = this.add({ kind: 'column', title: `Column x ${fmtInt(x)}, z ${fmtInt(z)}`, sub: `${latLonText(lat, lon)} · ${runName(run)} at ${fmtT(frame.t)}`,
        run, lat, lon, retry: () => this.inspect(run, frame, x, z, lat, lon) });
      const q = new URLSearchParams({ run: run.id, frame: frame.n, x, z, lat, lon });
      this.answer(job, await api(`api/inspect?${q}`));
    },
    async preview(run, frame, box, latLon, size) {
      const job = this.add({ kind: 'area', title: `Area x ${fmtInt(box.x0)} … ${fmtInt(box.x1)}, z ${fmtInt(box.z0)} … ${fmtInt(box.z1)}`,
        sub: `${size || ''} · ${runName(run)} at ${fmtT(frame.t)}`, retry: () => this.preview(run, frame, box, latLon, size) });
      this.answer(job, await api('api/preview', { run: run.id, frame: frame.n, t_years: frame.t, box, lat_lon: latLon }));
    },
    add(j) {
      j.id = ++this.seq; j.started = Date.now(); j.st = { state: 'sending' }; j.key = '';
      this.jobs.unshift(j);
      for (const old of this.jobs.splice(6)) clearTimeout(old.timer);
      setITab('s2', true);
      if (narrow.matches) setMTab('inspect');
      this.renderAll();
      return j;
    },
    answer(job, res) {
      const b = res.body || {};
      if (b.available === false) job.st = { state: 'unavailable', message: b.message };
      else if (!res.ok && (b.error || res.status === 0)) job.st = { state: 'failed', error: b.error || `HTTP ${res.status}` };
      else job.st = b;
      if (job.st.state === 'done' || job.st.url) this.finished(job);
      this.render(job);
      // A column answered from the run's bake (no job): the full answer goes to the Details tab.
      if (job.st.inspect) showColumn(job);
      this.follow(job);
    },
    follow(job) {
      const st = job.st;
      if (!st.status_url || st.url || ['done', 'failed', 'unavailable'].includes(st.state)) return;
      job.timer = setTimeout(async () => {
        const res = await api(st.status_url);
        if (res.status === 0) { this.follow(job); return; }      // the workbench blinked: keep asking
        const b = res.body || {};
        if (!res.ok && b.error) job.st = { state: 'failed', error: b.error, note: st.note };
        else job.st = Object.assign({ note: st.note }, b);
        if (job.st.state === 'done' || job.st.url) this.finished(job);
        else if (job.st.state === 'failed') toast(`${job.title}: the preview failed`, 'bad');
        this.render(job);
        this.follow(job);
      }, 1500);
    },
    finished(job) {
      if (job.done) return;
      job.done = true; job.ended = Date.now();
      toast(`${job.kind === 'area' ? '3D preview' : 'Column'} ready: ${job.title}`, 'ok', 5000, { label: 'Open', run: () => openViewer(job.st.url, job.title) });
      // One live 3D view at a time: older cards keep their buttons.
      for (const o of this.jobs) if (o !== job && o.embed) { o.embed = false; o.key = ''; this.render(o); }
      job.embed = true;
    },
    renderAll() {
      const box = $('jobs');
      for (const j of this.jobs) if (!j.el) { j.el = document.createElement('article'); j.el.className = 'job'; }
      box.replaceChildren(...this.jobs.map(j => j.el));
      $('s2intro').classList.toggle('compact', this.jobs.length > 0);
      for (const j of this.jobs) this.render(j);
      const active = this.jobs.filter(j => !!j.st.status_url && !j.st.url && !['done', 'failed'].includes(j.st.state) || j.st.state === 'sending').length;
      $('jobcount').hidden = !active; $('jobcount').textContent = active;
    },
    render(job) {
      if (!job.el) { this.renderAll(); return; }
      const st = job.st;
      // An answer that is neither a job nor a viewer (a direct column answer, after P1-08): shown as it is.
      const direct = !!st.inspect;
      const plain = !direct && !st.url && !st.status_url && !st.state && !st.error;
      const state = st.url ? 'done' : direct || plain ? 'answer' : st.state || 'queued';
      const key = [state, st.message, st.error, st.url, st.note, job.embed].join('|');
      const active = !['done', 'failed', 'unavailable', 'answer'].includes(state);
      job.el.classList.toggle('active', active);
      const act = this.jobs.filter(j => !!j.st.status_url && !j.st.url && !['done', 'failed'].includes(j.st.state) || j.st.state === 'sending').length;
      $('jobcount').hidden = !act; $('jobcount').textContent = act;
      if (key === job.key) { this.tickOne(job); return; }
      job.key = key;
      const idx = S2_STEPS.findIndex(s => s[0] === state);
      const stepName = state === 'sending' ? 'Asking the Mac…' : idx >= 0 ? S2_STEPS[idx][1] : cap1(state);
      const badgeCls = state === 'done' || state === 'answer' ? 's-done' : state === 'failed' ? 's-failed' : state === 'unavailable' ? 's-idle' : state === 'queued' || state === 'waiting' ? 's-queued' : 's-finalizing';
      let h = `<div class="jh"><div class="jic">${ic(job.kind === 'area' ? 'cube' : 'crosshair')}</div><div class="jt"><div class="a">${esc(job.title)}</div><div class="b">${esc(job.sub)}</div></div>` +
        `<span class="badge ${badgeCls}">${esc(state === 'done' || state === 'answer' ? 'Ready' : state === 'unavailable' ? 'Off' : state === 'failed' ? 'Failed' : state === 'sending' ? 'Sending' : 'Working')}</span></div>`;
      if (active) {
        h += `<div class="steps">${S2_STEPS.slice(0, -1).map((s, i) => `<i class="${i < idx ? 'done' : i === idx || (idx < 0 && i === 0) ? 'cur' : ''}" title="${esc(s[1])}"></i>`).join('')}</div>`;
        h += `<div class="jrow"><b>${esc(stepName)}</b><span class="elapsed"></span></div>`;
        if (st.message) h += `<div class="jmsg">${esc(st.message)}</div>`;
      } else if (state === 'failed') {
        h += `<div class="callout bad" style="margin-top:10px">${ic('alert')}<div><b>The preview failed.</b> <span class="clamp3">${esc(st.error || st.message || '')}</span></div></div>`;
        h += `<div class="jacts"><button class="btn btn-sm" data-jact="retry">${ic('refresh', 'ic-s')}Try again</button></div>`;
      } else if (state === 'unavailable') {
        h += `<div class="callout warn" style="margin-top:10px">${ic('info')}<div><b>3D preview is off on this workbench.</b> ${esc(st.message || '')}</div></div>`;
      } else if (state === 'answer' && direct) {
        const c = st.inspect, n = (c.body_details || []).length, sn = (c.sites || []).length;
        h += `<div class="jrow" style="margin-top:8px"><span style="display:flex;gap:5px;align-items:center;white-space:nowrap">${ic('check', 'ic-s')}From the run's bake</span><span class="elapsed" style="text-align:right">${fmtInt(st.seconds * 1000)} ms</span></div>` +
          `<div class="jmsg">Terrain ${fmtInt(st.terrain_height)} blocks · ${n ? `${n} ore ${n === 1 ? 'body' : 'bodies'}` : 'no ore body'}${sn ? ` · ${sn} ${sn === 1 ? 'site' : 'sites'}` : ''}</div>` +
          `<div class="jacts"><button class="btn btn-primary btn-sm" data-jact="details">${ic('info', 'ic-s')}Show details</button></div>`;
      } else if (state === 'answer') {
        h += `<pre class="lines" style="margin-top:10px;max-height:260px;overflow:auto">${esc(JSON.stringify(Object.assign({}, st, { note: undefined }), null, 1))}</pre>`;
      } else {
        h += `<div class="jrow" style="margin-top:8px"><span style="display:flex;gap:5px;align-items:center;white-space:nowrap">${ic('check', 'ic-s')}${st.note_kind === 'run' ? "The run's own terrain" : 'Real generator'}</span><span class="elapsed" style="text-align:right"></span></div>`;
        if (job.embed) h += `<div class="embed"><iframe src="${esc(embedURL(st.url))}" title="3D preview" loading="lazy"></iframe></div>`;
        h += `<div class="jacts"><button class="btn btn-primary btn-sm" data-jact="open">${ic('cube', 'ic-s')}Open 3D view</button><a class="btn btn-sm" href="${esc(st.url)}" target="_blank" rel="noopener">${ic('ext', 'ic-s')}New tab</a></div>`;
      }
      if (st.note) {
        const lead = st.note_kind === 'run' ? "This run's own terrain." : st.note_kind === 'test-world' ? 'A test world, not this run.' : 'About this preview.';
        h += `<div class="callout">${ic('info')}<div><b>${esc(lead)}</b> <span class="clamp3 notetext">${esc(st.note)}</span> <button class="linkbtn" data-jact="more">More</button></div></div>`;
      }
      job.el.innerHTML = h;
      this.tickOne(job);
    },
    tickOne(job) {
      const el = job.el && job.el.querySelector('.elapsed');
      if (!el) return;
      const done = !!(job.st.url || ['done', 'failed'].includes(job.st.state));
      if (done) {
        // How long the job took (a finished preview found by inspect may be hours old).
        const tot = job.st.timings && job.st.timings.total_s;
        const took = tot ? tot * 1000 : Date.parse(job.st.finished) - Date.parse(job.st.created);
        el.textContent = isFinite(took) && took > 0 ? `made in ${fmtDur(took)}${job.st.finished ? ` · ${relTime(job.st.finished)}` : ''}` : '';
        return;
      }
      const t0 = Math.min(Date.parse(job.st.created) || Infinity, job.started);
      el.textContent = `${fmtDur(Date.now() - t0)} · usually 1–2 min`;
    },
  };
  setInterval(() => { for (const j of Stage2.jobs) Stage2.tickOne(j); }, 1000);
  $('jobs').addEventListener('click', ev => {
    const b = ev.target.closest('[data-jact]');
    if (!b) return;
    const job = Stage2.jobs.find(j => j.el && j.el.contains(b));
    if (!job) return;
    if (b.dataset.jact === 'open') openViewer(job.st.url, job.title);
    else if (b.dataset.jact === 'details') showColumn(job);
    else if (b.dataset.jact === 'retry') job.retry();
    else if (b.dataset.jact === 'more') { const t = job.el.querySelector('.notetext'); t.classList.toggle('clamp3'); b.textContent = t.classList.contains('clamp3') ? 'More' : 'Less'; }
  });
  const embedURL = url => `${url}${url.includes('?') ? '&' : '?'}embed=1`;
  function openViewer(url, title) {
    modal(title || '3D preview', `<iframe src="${esc(url)}" title="3D preview"></iframe>`,
      `<a class="btn btn-sm" href="${esc(url)}" target="_blank" rel="noopener">${ic('ext', 'ic-s')}New tab</a>`);
  }

  // =================================================================================================
  // The globe: an orthographic view of the focused pane's frame, lit, with a graticule, the seam
  // and the events (on the CPU; the frame is 1024 × 512 at most)
  // =================================================================================================
  const globe = $('globe'), gctx = globe.getContext('2d');
  const G = { lon0: 0, lat0: 20, spin: (TV || Q.has('spin')) && (!reducedMotion.matches || Q.has('spin')), speed: 6, tex: null };
  let texImg = null, gpre = null, gbuf = null, lastGlobe = 0;
  if (TV) $('tv-globe').appendChild(globe);
  function globeSize() {
    const dpr = window.devicePixelRatio || 1;
    const S = clamp(Math.round((globe.clientWidth || 300) * dpr), 120, 560);
    if (globe.width !== S) { globe.width = S; globe.height = S; globeDirty = true; }
  }
  const GR = S => S / 2 * 0.9;   // the sphere's radius, leaving room for the atmosphere
  function globePre(S) {
    const n = S * S, xs = new Float32Array(n), ys = new Float32Array(n), zs = new Float32Array(n), light = new Float32Array(n), inside = new Uint8Array(n);
    const r = GR(S);
    const L = [-0.42, 0.5, 0.76], Ln = Math.hypot(...L);
    for (let j = 0; j < S; j++) for (let i = 0; i < S; i++) {
      const x = (i + 0.5 - S / 2) / r, y = (S / 2 - j - 0.5) / r, d = x * x + y * y, k = j * S + i;
      if (d <= 1) {
        const z = Math.sqrt(1 - d);
        inside[k] = 1; xs[k] = x; ys[k] = y; zs[k] = z;
        const lam = Math.max(0, (x * L[0] + y * L[1] + z * L[2]) / Ln);
        light[k] = 0.3 + 0.78 * Math.pow(lam, 0.85) + 0.08 * Math.pow(1 - z, 3);
      }
    }
    return { S, xs, ys, zs, light, inside };
  }
  function globeXY(lat, lon) {
    const S = globe.width, r = GR(S);
    const la = lat * D2R, dl = (lon - G.lon0) * D2R, cl = Math.cos(G.lat0 * D2R), sl = Math.sin(G.lat0 * D2R);
    const X = Math.cos(la) * Math.sin(dl), Y0 = Math.sin(la), Z0 = Math.cos(la) * Math.cos(dl);
    const y = Y0 * cl - Z0 * sl, z = Y0 * sl + Z0 * cl;
    return z > 0.02 ? [S / 2 + X * r, S / 2 - y * r] : null;
  }
  function globeLatLon(px, py) {
    const S = globe.width, r = GR(S);
    const x = (px - S / 2) / r, y = (S / 2 - py) / r, d = x * x + y * y;
    if (d > 1) return null;
    const z = Math.sqrt(1 - d), cl = Math.cos(G.lat0 * D2R), sl = Math.sin(G.lat0 * D2R);
    const yy = y * cl + z * sl, zz = z * cl - y * sl;
    return [Math.asin(clamp(yy, -1, 1)) / D2R, (((G.lon0 + Math.atan2(x, zz) / D2R) % 360) + 360) % 360];
  }
  function renderGlobe() {
    const S = globe.width;
    if (!gpre || gpre.S !== S) { gpre = globePre(S); gbuf = gctx.createImageData(S, S); }
    const out = gbuf.data, tex = G.tex;
    out.fill(0);
    const cl = Math.cos(G.lat0 * D2R), sl = Math.sin(G.lat0 * D2R), turn = G.lon0 / 360;
    const { xs, ys, zs, light, inside } = gpre;
    for (let k = 0, n = S * S; k < n; k++) {
      if (!inside[k]) continue;
      const x = xs[k], y = ys[k], z = zs[k];
      const yy = y * cl + z * sl, zz = z * cl - y * sl;
      let R = 30, Gn = 36, B = 52;
      if (tex) {
        let u = turn + Math.atan2(x, zz) / (2 * Math.PI);
        u -= Math.floor(u);
        const v = 0.5 - Math.asin(yy < -1 ? -1 : yy > 1 ? 1 : yy) / Math.PI;
        const px = Math.floor(u * tex.w) % tex.w, py = Math.min(tex.h - 1, Math.max(0, Math.floor(v * tex.h)));
        const q = (py * tex.w + px) * 4;
        if (tex.d[q + 3] > 0) { R = tex.d[q]; Gn = tex.d[q + 1]; B = tex.d[q + 2]; }
      }
      const s = light[k], o = k * 4;
      out[o] = Math.min(255, R * s); out[o + 1] = Math.min(255, Gn * s); out[o + 2] = Math.min(255, B * s); out[o + 3] = 255;
    }
    gctx.putImageData(gbuf, 0, 0);
    const r = GR(S), c = S / 2, dpr = S / Math.max(1, globe.clientWidth);
    // Atmosphere and rim.
    const atm = gctx.createRadialGradient(c, c, r * 0.96, c, c, r * 1.1);
    atm.addColorStop(0, 'rgba(120,160,255,.38)'); atm.addColorStop(0.35, 'rgba(110,150,255,.16)'); atm.addColorStop(1, 'rgba(110,150,255,0)');
    gctx.fillStyle = atm; gctx.beginPath(); gctx.arc(c, c, r * 1.1, 0, 2 * Math.PI); gctx.arc(c, c, r, 0, 2 * Math.PI, true); gctx.fill();
    const rim = gctx.createLinearGradient(c - r, c - r, c + r, c + r);
    rim.addColorStop(0, 'rgba(190,210,255,.55)'); rim.addColorStop(1, 'rgba(190,210,255,.06)');
    gctx.lineWidth = 1.2 * dpr; gctx.strokeStyle = rim; gctx.beginPath(); gctx.arc(c, c, r, 0, 2 * Math.PI); gctx.stroke();
    // Graticule and the seam.
    const line = (pts) => { gctx.beginPath(); let pen = false; for (const q of pts) { if (!q) { pen = false; continue; } if (pen) gctx.lineTo(q[0], q[1]); else gctx.moveTo(q[0], q[1]); pen = true; } gctx.stroke(); };
    if (gridOn) {
      gctx.lineWidth = dpr;
      for (let lat = -60; lat <= 60; lat += 30) {
        gctx.strokeStyle = lat === 0 ? 'rgba(255,255,255,.22)' : 'rgba(255,255,255,.1)';
        const pts = []; for (let lon = 0; lon <= 360; lon += 3) pts.push(globeXY(lat, lon)); line(pts);
      }
      gctx.strokeStyle = 'rgba(255,255,255,.1)';
      for (let lon = 30; lon < 360; lon += 30) { const pts = []; for (let lat = -87; lat <= 87; lat += 3) pts.push(globeXY(lat, lon)); line(pts); }
      gctx.strokeStyle = 'rgba(255,122,122,.75)'; gctx.lineWidth = 1.6 * dpr; gctx.setLineDash([4 * dpr, 3 * dpr]);
      { const pts = []; for (let lat = -88; lat <= 88; lat += 2) pts.push(globeXY(lat, 0)); line(pts); }
      gctx.setLineDash([]);
    }
    const p = P(), run = p.run;
    if (markersOn && run && p.cur >= 0 && run.frames[p.cur]) {
      const t = run.frames[p.cur].t, t0 = p.cur > 0 ? run.frames[p.cur - 1].t : -Infinity;
      for (const { e, ll, age } of visibleEvents(run, t)) {
        const q = globeXY(ll[0], ll[1]);
        if (!q) continue;
        const fresh = e.t_start > t0;
        gctx.globalAlpha = fresh ? 1 : 1 - 0.72 * age / FADE_YEARS;
        gctx.beginPath(); gctx.arc(q[0], q[1], (fresh ? 5 : 2.8) * dpr, 0, 2 * Math.PI);
        gctx.fillStyle = kindColour(e.kind); gctx.fill();
        gctx.lineWidth = (fresh ? 1.8 : 1) * dpr; gctx.strokeStyle = fresh ? '#fff' : 'rgba(0,0,0,.6)'; gctx.stroke();
      }
      gctx.globalAlpha = 1;
    }
    if (pin) {
      const q = globeXY(pin.lat, pin.lon);
      if (q) { gctx.beginPath(); gctx.arc(q[0], q[1], 6 * dpr, 0, 2 * Math.PI); gctx.lineWidth = 2 * dpr; gctx.strokeStyle = '#fff'; gctx.stroke(); }
    }
  }
  function spin(dt) {
    if (G.spin && dt > 0) { G.lon0 = (G.lon0 + G.speed * dt) % 360; globeDirty = true; }
    const now = performance.now();
    const hiddenGlobe = !TV && ($('globecard').classList.contains('collapsed') || (narrow.matches && document.body.dataset.mtab !== 'globe'));
    if (globeDirty && !hiddenGlobe && now - lastGlobe > 50) { globeDirty = false; lastGlobe = now; renderGlobe(); }
  }
  function spinButton() { $('spinbtn').setAttribute('aria-pressed', String(G.spin)); }
  let gdrag = null;
  globe.addEventListener('pointerdown', e => { gdrag = { x: e.clientX, y: e.clientY }; G.spin = false; spinButton(); globe.setPointerCapture(e.pointerId); globe.classList.add('dragging'); });
  globe.addEventListener('pointermove', e => {
    const rect = globe.getBoundingClientRect(), s = globe.width / rect.width;
    const ll = globeLatLon((e.clientX - rect.left) * s, (e.clientY - rect.top) * s);
    $('globeread').textContent = ll ? latLonText(ll[0], ll[1]) : 'Drag to turn · double-click to spin';
    if (!gdrag) return;
    const k = 180 / Math.max(1, globe.clientWidth);
    G.lon0 = ((G.lon0 - (e.clientX - gdrag.x) * k) % 360 + 360) % 360;
    G.lat0 = clamp(G.lat0 + (e.clientY - gdrag.y) * k, -89, 89);
    gdrag = { x: e.clientX, y: e.clientY };
    globeDirty = true;
  });
  globe.addEventListener('pointerup', () => { gdrag = null; globe.classList.remove('dragging'); });
  globe.addEventListener('pointerleave', () => { if (!gdrag) $('globeread').textContent = 'Drag to turn · double-click to spin'; });
  globe.addEventListener('dblclick', () => { G.spin = !G.spin; spinButton(); });
  $('spinbtn').addEventListener('click', () => { G.spin = !G.spin; spinButton(); });

  // =================================================================================================
  // Legend and layer picker
  // =================================================================================================
  function legend() {
    depLegend();
    const p = P(), r = p.run;
    const el = $('legend');
    let name = layer, info = null;
    if (r && p.cur >= 0 && r.frames[p.cur]) {
      const f = r.frames[displayIndex(r, p.cur)], li = r.layerIndex(f);
      name = f.layers[li]; info = rampInfo(f.ramps && f.ramps[li]);
    } else if (r && layer) info = rampInfo(r.rampOf.get(layer));
    if (!name) { el.innerHTML = '<div class="lh"><span class="lname muted">No layer yet</span></div>'; return; }
    const key = `${name}|${info ? info.name : ''}|${el.clientWidth}`;
    if (el.dataset.key !== key) { el.dataset.key = key; el.innerHTML = legendHTML(name, info, 'side'); paintLegendBars(el); const n = el.querySelector('.note'); if (n) n.title = 'Click to read all'; }
    if (TV) { const t = $('tv-legend'); if (t.dataset.key !== key) { t.dataset.key = key; t.innerHTML = legendHTML(name, info, 'tv'); paintLegendBars(t); } }
  }

  let shownLayers = '';
  function allLayers() {
    const names = [];
    for (const p of panes) if (p.run && (p.i === 0 || comparing)) for (const n of p.run.layers) if (!names.includes(n)) names.push(n);
    return names;
  }
  const rampOfLayer = n => rampInfo((P().run && P().run.rampOf.get(n)) || (panes[0].run && panes[0].run.rampOf.get(n)) || (panes[1].run && panes[1].run.rampOf.get(n)));
  function layerPanel() {
    const names = allLayers();
    const run = P().run;
    const curFrame = run && P().cur >= 0 ? run.frames[P().cur] : null;
    // The picker button.
    const { label: ln } = layerLabel(layer);
    const g = LAYER_GROUPS.find(x => x[0] === layerGroup(layer || ''));
    $('lpickname').textContent = layer ? ln : 'No layer yet';
    $('lpickgrp').textContent = layer && g ? g[1] : '';
    $('lpicksw').style.background = rampCSS(rampOfLayer(layer));
    $('layercount').textContent = names.length ? `${names.length} layers` : '';
    const key = `${names.join('|')}#${layer}#${ramps ? 1 : 0}#${curFrame ? curFrame.layers.join() : ''}`;
    if (key === shownLayers) return;
    shownLayers = key;
    const box = $('layers');
    if (!names.length) { box.innerHTML = '<div class="empty" style="padding:14px"><p>Layers appear with the first frame.</p></div>'; return; }
    let h = '';
    for (const [grp, gname] of LAYER_GROUPS) {
      const inG = names.filter(n => layerGroup(n) === grp);
      if (!inG.length) continue;
      h += `<div class="lgroup" data-g="${esc(grp)}"><div class="gtitle"><span>${esc(gname)}</span><span>${inG.length}</span></div><div class="items">`;
      for (const n of inG) {
        const i = names.indexOf(n);
        const miss = curFrame && !curFrame.layers.includes(n);
        const { label: lab, unit } = layerLabel(n);
        h += `<button class="lchip${miss ? ' missing' : ''}" data-layer="${esc(n)}" aria-pressed="${n === layer}" data-tip="${esc(n)}${unit ? ` · ${esc(unit)}` : ''}${miss ? ' · shown from the nearest step that has it' : ''}">` +
          `<span class="sw" style="background:${rampCSS(rampOfLayer(n))}"></span>${esc(lab)}${i < 9 ? `<kbd>${i + 1}</kbd>` : ''}</button>`;
      }
      h += '</div></div>';
    }
    box.innerHTML = h;
  }
  function layerPop(show) {
    const pop = $('layerpop'), btn = $('lpickbtn');
    if (narrow.matches) return;
    const on = show === undefined ? pop.hidden : show;
    pop.hidden = !on;
    btn.setAttribute('aria-expanded', String(on));
    if (!on) return;
    const r = btn.getBoundingClientRect(), w = Math.max(r.width + 72, 320);
    pop.style.width = `${w}px`;
    pop.style.left = `${clamp(r.right + 72 - w, 8, innerWidth - w - 8)}px`;
    const below = innerHeight - r.bottom - 12;
    pop.style.top = `${r.bottom + 6}px`;
    pop.style.maxHeight = `${Math.max(200, below)}px`;
    const cur = pop.querySelector('[aria-pressed="true"]');
    if (cur) cur.focus({ preventScroll: false });
  }
  $('lpickbtn').addEventListener('click', ev => { ev.stopPropagation(); layerPop(); });
  document.addEventListener('pointerdown', ev => { if (!$('layerpop').hidden && !ev.target.closest('#layerpop, #lpickbtn')) layerPop(false); });
  function cycleLayer(d) {
    const names = allLayers();
    if (!names.length) return;
    const i = names.indexOf(layer);
    setLayer(names[(i + d + names.length) % names.length]);
  }
  $('legend').addEventListener('click', ev => { const n = ev.target.closest('.note'); if (n) n.classList.toggle('open'); });
  $('lprev').addEventListener('click', () => cycleLayer(-1));
  $('lnext').addEventListener('click', () => cycleLayer(1));
  $('layers').addEventListener('click', ev => {
    const b = ev.target.closest('[data-layer]');
    if (b) { setLayer(b.dataset.layer); layerPop(false); $('lpickbtn').focus(); }
  });
  function setLayer(name) {
    layer = name;
    shownLayers = ''; layersDirty = legendDirty = true;
    for (const p of panes) if (p.run && p.cur >= 0) p.run.image(displayIndex(p.run, p.cur));
    mapDirty = true; texImg = null;
    hashState();
  }
  const layerByNumber = k => allLayers()[k - 1];

  // =================================================================================================
  // The dock: time readout, transport, speed, live, and the timeline
  // =================================================================================================
  function label() {
    const p = P(), r = p.run;
    const tl = $('timeline');
    $('tlempty').hidden = !!(r && r.frames.length);
    if (!r || !r.frames.length) {
      $('tbig').innerHTML = r ? '<span class="muted">Waiting…</span>' : '—';
      $('tsub').textContent = r ? (r.status && r.status.state === 'queued' ? 'Queued on the Mac' : 'No frames yet') : 'No run open';
      tl.setAttribute('aria-valuemax', 0); tl.setAttribute('aria-valuenow', 0);
      buttons();
      return;
    }
    const n = p.cur < 0 ? r.frames.length - 1 : p.cur;
    const f = r.frames[n], live = r.frames[r.frames.length - 1];
    $('tbig').innerHTML = tHTML(f.t);
    let sub = `Step <b>${n + 1}</b> of ${r.frames.length}${r.finished() ? '' : '+'} · ${esc(stageText(f.stage))}`;
    if (live !== f && !r.finished()) sub += ` · the run is at <b>${esc(fmtT(live.t))}</b>`;
    $('tsub').innerHTML = sub;
    tl.setAttribute('aria-valuemax', r.frames.length - 1);
    tl.setAttribute('aria-valuenow', n);
    tl.setAttribute('aria-valuetext', fmtT(f.t));
    for (const q of panes) badge(q);
    buttons();
  }

  function buttons() {
    const r = P().run;
    const playing = mode === 'play';
    $('play').innerHTML = ic(playing ? 'pause' : 'play');
    $('play').setAttribute('aria-label', playing ? 'Pause' : 'Play');
    const lb = $('livebtn');
    const fin = !r || r.finished();
    lb.classList.toggle('on', mode === 'live');
    lb.classList.toggle('end', fin);
    $('livetext').textContent = fin ? (mode === 'live' ? 'At the end' : 'Jump to end') : (mode === 'live' ? 'Live' : 'Go live');
    lb.dataset.tip = fin ? 'Show the last step' : 'Follow the run as it computes';
    $('rate').innerHTML = `${rate} <small>Myr/s</small>`;
    const none = !r || !r.frames.length;
    for (const id of ['rewind', 'back', 'play', 'fwd', 'livebtn', 'slower', 'faster']) $(id).disabled = none;
    if (none) { lb.classList.remove('on'); $('livetext').textContent = 'Live'; }
    for (const t of ['select', 'inspect', 'area']) $(`tool-${t}`).setAttribute('aria-pressed', String(tool === t));
    for (const b of document.querySelectorAll('.s2-tool')) b.setAttribute('aria-pressed', String(b.dataset.tool === tool));
    $('markers').setAttribute('aria-pressed', String(markersOn));
    $('gridbtn').setAttribute('aria-pressed', String(gridOn));
    depButton();
    for (const q of panes) badge(q);
  }

  // Timeline: the run's whole planned span; computed part, playhead, live edge, Myr ticks, and a
  // histogram of the events by kind.
  const TL = { el: $('timeline'), c: $('tlcanvas'), hist: null, histKey: '', drag: false };
  function tlGeom() {
    const W = TL.el.clientWidth, H = TL.el.clientHeight, s = H / 58, pad = 10 * s;
    return { W, H, s, x0: pad, x1: W - pad, laneB: 19 * s, trackY: 24 * s, trackH: 6 * s, tickY: 33 * s, labelY: 47 * s };
  }
  function drawTimeline() {
    const g = tlGeom(), dpr = window.devicePixelRatio || 1, c = TL.c;
    const cw = Math.max(1, Math.round(g.W * dpr)), ch = Math.max(1, Math.round(g.H * dpr));
    if (c.width !== cw || c.height !== ch) { c.width = cw; c.height = ch; }
    const ctx = c.getContext('2d');
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, g.W, g.H);
    const p = P(), r = p.run;
    if (!r || !r.frames.length || g.W < 40 || g.H < 20) return;
    const [t0, t1] = r.range();
    const X = t => g.x0 + (t - t0) / (t1 - t0) * (g.x1 - g.x0);
    const lastT = r.frames[r.frames.length - 1].t;
    const curT = p.cur >= 0 ? r.frames[p.cur].t : lastT;
    const rr = (x, y, w, h, rad) => { ctx.beginPath(); if (ctx.roundRect) ctx.roundRect(x, y, Math.max(0, w), h, rad); else ctx.rect(x, y, Math.max(0, w), h); };
    // Track: the planned span, the computed part, the played part.
    rr(g.x0, g.trackY, g.x1 - g.x0, g.trackH, g.trackH / 2); ctx.fillStyle = '#1a212c'; ctx.fill();
    if (!r.finished() && X(lastT) < g.x1) {
      ctx.save(); rr(X(lastT), g.trackY, g.x1 - X(lastT), g.trackH, g.trackH / 2); ctx.clip();
      ctx.strokeStyle = 'rgba(255,255,255,.07)'; ctx.lineWidth = 1;
      for (let x = X(lastT) - 20; x < g.x1 + 20; x += 5) { ctx.beginPath(); ctx.moveTo(x, g.trackY + g.trackH); ctx.lineTo(x + g.trackH, g.trackY); ctx.stroke(); }
      ctx.restore();
    }
    rr(g.x0, g.trackY, X(lastT) - g.x0, g.trackH, g.trackH / 2); ctx.fillStyle = 'rgba(125,140,255,.3)'; ctx.fill();
    const grad = ctx.createLinearGradient(g.x0, 0, X(curT), 0);
    grad.addColorStop(0, 'rgba(125,140,255,.55)'); grad.addColorStop(1, '#8e9bff');
    rr(g.x0, g.trackY, X(curT) - g.x0, g.trackH, g.trackH / 2); ctx.fillStyle = grad; ctx.fill();
    // Ticks in Myr ago.
    const spanMyr = (t1 - t0) / 1e6, want = Math.max(2, Math.floor((g.x1 - g.x0) / (TV ? 150 : 84)));
    const step = [1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000].find(s => spanMyr / s <= want) || 5000;
    ctx.font = `500 ${10.5 * g.s}px Inter, system-ui, sans-serif`; ctx.textBaseline = 'alphabetic';
    const firstMyr = Math.ceil(-t1 / 1e6 / step) * step;
    for (let a = firstMyr; a <= -t0 / 1e6 + 1e-6; a += step) {
      const x = X(-a * 1e6);
      ctx.fillStyle = 'rgba(160,175,200,.35)'; ctx.fillRect(Math.round(x), g.tickY - 2 * g.s, 1, 4 * g.s);
      const text = a === 0 ? 'now' : `${fmtInt(a)}`;
      const tw = ctx.measureText(text).width;
      ctx.fillStyle = '#7f8a9d';
      ctx.fillText(text, clamp(x - tw / 2, g.x0 - 6, g.x1 + 6 - tw), g.labelY);
    }
    ctx.fillStyle = '#5a6477';
    ctx.font = `600 ${9.5 * g.s}px Inter, system-ui, sans-serif`;
    ctx.fillText('MYR AGO', g.x0, g.labelY + 11 * g.s > g.H ? g.laneB - 8 * g.s : g.labelY + 11 * g.s);
    // Event histogram by kind.
    const hk = `${r.id}|${r.events.length}|${Math.round(g.x1 - g.x0)}|${[...hiddenKinds].join(',')}|${t0}|${t1}|${g.s}`;
    if (TL.histKey !== hk) {
      TL.histKey = hk;
      const bw = 3 * g.s, nb = Math.max(1, Math.ceil((g.x1 - g.x0) / bw)), bins = Array.from({ length: nb }, () => new Map());
      let max = 0;
      for (const e of r.events) {
        const k = shortKind(e.kind);
        if (hiddenKinds.has(k)) continue;
        const b = clamp(Math.floor((X(e.t_start) - g.x0) / bw), 0, nb - 1);
        bins[b].set(k, (bins[b].get(k) || 0) + 1);
      }
      for (const b of bins) { let s = 0; for (const v of b.values()) s += v; b.total = s; max = Math.max(max, s); }
      TL.hist = { bins, max, bw };
    }
    const { bins, max, bw } = TL.hist, laneH = g.laneB - 2 * g.s;
    if (max > 0) {
      bins.forEach((b, i) => {
        if (!b.total) return;
        const hh = Math.max(2 * g.s, Math.sqrt(b.total / max) * laneH);
        let y = g.laneB;
        const x = g.x0 + i * bw;
        const past = x <= X(curT);
        for (const [k, v] of [...b.entries()].sort()) {
          const seg = hh * v / b.total;
          ctx.fillStyle = KIND_COLOURS[k] || kindColour(k);
          ctx.globalAlpha = past ? 0.9 : 0.35;
          ctx.fillRect(x, y - seg, Math.max(1, bw - 1), seg);
          y -= seg;
        }
      });
      ctx.globalAlpha = 1;
    }
    // Live edge.
    if (!r.finished()) {
      const x = X(lastT);
      ctx.save(); ctx.shadowColor = 'rgba(255,77,94,.9)'; ctx.shadowBlur = 10;
      ctx.beginPath(); ctx.arc(x, g.trackY + g.trackH / 2, 4 * g.s, 0, 2 * Math.PI); ctx.fillStyle = '#ff4d5e'; ctx.fill(); ctx.restore();
    }
    // Playhead.
    const x = X(curT);
    ctx.fillStyle = '#fff'; ctx.fillRect(Math.round(x) - 1, 0, 2, g.trackY + g.trackH + 2 * g.s);
    ctx.save(); ctx.shadowColor = 'rgba(0,0,0,.6)'; ctx.shadowBlur = 6;
    ctx.beginPath(); ctx.arc(x, g.trackY + g.trackH / 2, 7 * g.s, 0, 2 * Math.PI); ctx.fillStyle = '#fff'; ctx.fill(); ctx.restore();
    ctx.beginPath(); ctx.arc(x, g.trackY + g.trackH / 2, 3 * g.s, 0, 2 * Math.PI); ctx.fillStyle = '#7d8cff'; ctx.fill();
  }
  function tlTimeAt(clientX) {
    const r = P().run;
    if (!r || !r.frames.length) return null;
    const g = tlGeom(), rect = TL.el.getBoundingClientRect();
    const [t0, t1] = r.range();
    return t0 + clamp((clientX - rect.left - g.x0) / (g.x1 - g.x0), 0, 1) * (t1 - t0);
  }
  function seekTime(t) {
    const p = P(), r = p.run;
    if (!r || !r.frames.length || t === null) return;
    const last = r.frames[r.frames.length - 1];
    if (t >= last.t && !r.finished()) { setMode('live'); return; }
    setMode('pause');
    show(p, r.frameAt(t));
    tp = r.frames[p.cur].t;
  }
  TL.el.addEventListener('pointerdown', ev => { if (TV) return; TL.drag = true; TL.el.setPointerCapture(ev.pointerId); seekTime(tlTimeAt(ev.clientX)); });
  TL.el.addEventListener('pointermove', ev => {
    const t = tlTimeAt(ev.clientX), bub = $('tlbubble'), r = P().run;
    if (t === null || TV) { bub.hidden = true; return; }
    if (TL.drag) seekTime(t);
    const last = r.frames[r.frames.length - 1];
    const n = r.frameAt(t);
    const rect = TL.el.getBoundingClientRect();
    bub.hidden = false;
    bub.style.left = `${clamp(ev.clientX - rect.left, 70, rect.width - 70)}px`;
    bub.innerHTML = t > last.t && !r.finished() ? `<b>${esc(fmtT(t))}</b> · not computed yet` : `<b>${esc(fmtT(r.frames[n].t))}</b> · step ${n + 1}`;
  });
  TL.el.addEventListener('pointerup', () => { TL.drag = false; });
  TL.el.addEventListener('pointerleave', () => { if (!TL.drag) $('tlbubble').hidden = true; });

  // =================================================================================================
  // Inspector: events, details (an event's cause chain, a point's values), the run
  // =================================================================================================
  let selectedEvent = null, pin = null;
  function setITab(name, reveal) {
    for (const b of document.querySelectorAll('[data-itab]')) b.setAttribute('aria-selected', String(b.dataset.itab === name));
    for (const k of ['events', 'details', 's2', 'run']) $(`ip-${k}`).hidden = k !== name;
    store.set('itab', name);
    // Something new to read there: bring the inspector into view on a short screen.
    if (reveal && !narrow.matches && !TV) {
      const rp = $('rpanel'), rr = rp.getBoundingClientRect(), ir = $('inspector').getBoundingClientRect();
      const below = ir.top + Math.min(ir.height, name === 's2' ? 460 : 340) - rr.bottom;
      if (below > 0) rp.scrollBy({ top: below + 12, behavior: reducedMotion.matches ? 'auto' : 'smooth' });
    }
  }
  for (const b of document.querySelectorAll('[data-itab]')) b.addEventListener('click', () => setITab(b.dataset.itab));

  let evShownKey = '';
  function ticker() {
    const box = $('ticker'), p = P(), r = p.run;
    if (!r) { box.innerHTML = '<div class="empty"><p>Open a run to see its events.</p></div>'; $('evcount').textContent = 'No events'; evShownKey = ''; kindChips(null); return; }
    const t = p.cur >= 0 && r.frames[p.cur] ? r.frames[p.cur].t : -Infinity, t0 = p.cur > 0 ? r.frames[p.cur - 1].t : -Infinity;
    // Events are logged when they end; the list orders them by when they began.
    const upto = r.events.filter(e => e.t_start <= t && !hiddenKinds.has(shortKind(e.kind))).sort((a, b) => a.t_start - b.t_start);
    $('evcount').textContent = r.events.length ? `${fmtInt(upto.length)} of ${fmtInt(r.events.length)} up to this step` : 'No events yet';
    kindChips(r);
    const shown = upto.slice(-160).reverse();
    const key = `${r.id}|${t}|${shown.length}|${shown.length ? shown[0].id : ''}|${selectedEvent}|${[...hiddenKinds].join(',')}`;
    if (key === evShownKey) return;
    evShownKey = key;
    if (!shown.length) { box.innerHTML = `<div class="empty">${ic('events', 'art')}<p>${r.events.length ? 'No events up to this step.' : 'Events appear as the simulation makes them.'}</p></div>`; return; }
    box.innerHTML = shown.map(e =>
      `<button class="evrow${e.t_start > t0 ? ' new' : ''}${e.id === selectedEvent ? ' sel' : ''}" data-ev="${e.id}" style="--c:${kindColour(e.kind)}"><span class="d"></span>` +
      `<span class="k">${esc(prettyKind(e.kind))}</span><span class="t">${esc(fmtT(e.t_start))}</span></button>`).join('');
  }
  let kindKey = '';
  function kindChips(r) {
    const box = $('evkinds');
    if (!r) { box.innerHTML = ''; kindKey = ''; return; }
    const counts = new Map();
    for (const e of r.events) { const k = shortKind(e.kind); counts.set(k, (counts.get(k) || 0) + 1); }
    const key = [...counts].map(([k, n]) => `${k}${n}`).join() + [...hiddenKinds].join();
    if (key === kindKey) return;
    kindKey = key;
    box.innerHTML = [...counts].sort((a, b) => b[1] - a[1]).map(([k, n]) =>
      `<button class="kchip" data-kind="${esc(k)}" aria-pressed="${!hiddenKinds.has(k)}" style="--c:${kindColour(k)}"><i></i>${esc(prettyKind(k))}<span class="n">${fmtInt(n)}</span></button>`).join('');
    $('evfilter').innerHTML = `${ic('filter', 'ic-s')}Kinds${hiddenKinds.size ? ` <span class="count">${counts.size - [...hiddenKinds].filter(k => counts.has(k)).length}/${counts.size}</span>` : ''}`;
  }
  $('evfilter').addEventListener('click', () => { const k = $('evkinds'); k.hidden = !k.hidden; $('evfilter').setAttribute('aria-pressed', String(!k.hidden)); });
  $('evkinds').addEventListener('click', ev => {
    const b = ev.target.closest('[data-kind]');
    if (!b) return;
    const k = b.dataset.kind;
    if (ev.altKey || ev.metaKey) { const all = [...b.parentElement.children].map(x => x.dataset.kind); hiddenKinds = new Set(all.filter(x => x !== k)); }
    else if (hiddenKinds.has(k)) hiddenKinds.delete(k); else hiddenKinds.add(k);
    store.set('hiddenKinds', [...hiddenKinds]);
    kindKey = ''; eventsDirty();
  });

  function evLink(r, id) {
    const e = r.byId.get(id);
    const w = Math.floor(id / 2 ** 48), seq = id - w * 2 ** 48;
    return e ? `<button data-ev="${id}" style="--c:${kindColour(e.kind)}"><i></i>${esc(prettyKind(e.kind))}<span class="t">${esc(fmtT(e.t_start))}</span></button>`
      : `<span class="none">${w}:${seq} (not in this log)</span>`;
  }
  function placeText(p) {
    if (!p) return '—';
    if (p.type === 'point') return esc(latLonText(p.lat, p.lon));
    if (p.type === 'global') return 'Global';
    return esc(`${p.type} ${p[p.type === 'region' ? 'mask' : p.type]}`);
  }
  const PARAM_UNITS = { 'mm yr': 'mm/yr', 'm myr': 'm/Myr', myr: 'Myr', km: 'km', km2: 'km²', km3: 'km³', m: 'm', deg: '°', c: '°C' };
  const prettyParam = k => cap1(String(k).replace(/_/g, ' ').replace(/\b(mm yr|m myr|myr|km2|km3|km|deg|m|c)$/, s => `(${PARAM_UNITS[s]})`));
  function showEvent(r, id) {
    const e = r.byId.get(id);
    if (!e) return;
    selectedEvent = id; pin = null;
    const w = Math.floor(e.id / 2 ** 48), seq = e.id - w * 2 ** 48;
    const param = v => (v && typeof v === 'object' && 'id' in v) ? (r.byId.has(v.id) ? `<div class="chain">${evLink(r, v.id)}</div>` : esc(`id ${v.id}`))
      : esc(v === null ? '—' : typeof v === 'number' ? numText(v) : v);
    const rows = [
      ['When', esc(e.t_end > e.t_start ? `${fmtT(e.t_start)} → ${fmtT(e.t_end)}` : fmtT(e.t_start))],
      ['Where', placeText(e.place)],
      ['Magnitude', esc(e.magnitude === null || e.magnitude === undefined ? '—' : Number(e.magnitude).toPrecision(3))],
      ['Source', esc(shortKind(e.source))],
    ];
    for (const [k, v] of Object.entries(e.params || {})) rows.push([prettyParam(k), param(v)]);
    const eff = r.effects.get(e.id) || [];
    const ll = placeLatLon(e);
    $('details').innerHTML =
      `<div class="dh" style="--c:${kindColour(e.kind)}"><span class="d"></span><div><h4>${esc(prettyKind(e.kind))}</h4><div class="id">${esc(e.kind)} · ${w}:${seq}</div></div></div>` +
      `<div class="kv">${rows.map(([a, b]) => `<div>${esc(a)}</div><div>${b}</div>`).join('')}</div>` +
      `<div class="acts">${ll ? `<button class="btn btn-sm" data-dact="locate">${ic('target', 'ic-s')}Turn the globe here</button>` : ''}<button class="btn btn-sm" data-dact="jump">${ic('clock', 'ic-s')}Go to this step</button></div>` +
      `<h5>Caused by</h5><div class="chain">${(e.causes || []).length ? e.causes.map(c => evLink(r, c)).join('') : '<span class="none">Nothing recorded: a first cause.</span>'}</div>` +
      `<h5>Led to</h5><div class="chain">${eff.length ? eff.map(c => evLink(r, c)).join('') : '<span class="none">Nothing yet.</span>'}</div>`;
    $('details').dataset.curEv = id;
    setITab('details', true);
    if (narrow.matches) setMTab('inspect');
    if (ll) { G.lon0 = ((ll[1] % 360) + 360) % 360; G.lat0 = clamp(ll[0], -80, 80); G.spin = false; spinButton(); globeDirty = true; }
    mapDirty = true; evShownKey = ''; tickerDirty = true;
  }
  // A column answered from the run's bake (`deeptime inspect-column --why` through /api/inspect):
  // the terrain, the rock from the top block down, the ore bodies and each site's "why here".
  function showColumn(job) {
    const st = job.st, c = st.inspect, r = job.run;
    if (!c) return;
    selectedEvent = null;
    const i = Math.max(0, panes.findIndex(p => p.run === r));
    const rows = a => a.map(([k, v]) => `<div>${k}</div><div>${v}</div>`).join('');
    const yr = (t, b) => `<span class="tnum">y ${fmtInt(t)} → ${fmtInt(b)}</span>`;
    const ev = e => {
      const id = Number(e.id);
      return r && r.byId.has(id) ? evLink(r, id)
        : `<span class="none">${esc(prettyKind(e.kind))} · ${esc(fmtT(e.t_start))}</span>`;
    };
    const rock = c.rock || [], shown = rock.slice(0, 14);
    const bodies = c.body_details || [], sites = c.sites || [];
    const site = id => sites.find(s => s.id === id);
    let h = `<div class="dh" style="--c:#7cc4ff"><span class="d"></span><div><h4>Column x ${fmtInt(c.x)}, z ${fmtInt(c.z)}</h4>` +
      `<div class="id">${esc(latLonText(c.lat, c.lon))} · from the run's bake · ${fmtInt(st.seconds * 1000)} ms</div></div></div>` +
      `<div class="kv">${rows([
        ['Terrain height', `<b class="tnum">${numText(c.surface)}</b> blocks (top block y ${fmtInt(c.top_y)})`],
        ['Elevation', `${numText(c.elevation)} + ${numText(c.detail)} of detail`],
        ['Water', c.water === null || c.water === undefined ? 'none' : `surface at y ${numText(c.water)}`],
        ['Horizons', `${(c.strata || []).length} in the column's stack`],
      ])}</div>` +
      `<h5>Rock, top block down</h5><div class="kv">${rows(shown.map(([t, b, k]) => [yr(t, b), k ? esc(k.replace(/^deeptime:/, '').replace(/_/g, ' ')) : '<span class="none">unclassified</span>']))}` +
      `${rock.length > shown.length ? `<div></div><div class="none">… and ${rock.length - shown.length} more</div>` : ''}</div>` +
      `<h5>Ore bodies here</h5>` +
      (bodies.length ? `<div class="kv">${rows(bodies.map(b => [yr(b.top, b.bottom),
        `${esc(cap1(String(b.role || 'body').replace(/_/g, ' ')))} of ${esc(String(b.shape || 'body'))}${b.rock_class ? ` in ${esc(String(b.rock_class).replace(/^deeptime:/, '').replace(/_/g, ' '))}` : ''}${b.site != null ? ` · site ${b.site}` : ''}`]))}</div>`
        : '<div class="chain"><span class="none">The column passes through no ore body.</span></div>');
    for (const s of sites) {
      const why = s.why || {};
      const prods = (s.products || []).map(p => `${esc(shortKind(p.material))}${p.role ? ` (${esc(p.role)})` : ''} · grade ${numText(p.grade)} · ${numText(p.tonnage_t)} t`).join('<br>');
      h += `<h5>Why here: ${esc(prettyKind(s.deposit_type || s.kind))} (site ${s.id})</h5>` +
        `<div class="kv">${rows([['Kind', esc(String(s.kind))], ['Yields', prods || '—']])}</div>` +
        `<div class="chain" style="margin-top:8px">${why.forming_event ? `${ev(why.forming_event)}` : `<span class="none">${esc(why.error || 'No forming event recorded.')}</span>`}` +
        `${(why.chain || []).map(ev).join('')}</div>`;
    }
    // Volcanoes and flood basalts whose footprint holds the column (P1-21), nearest first.
    for (const l of (c.landforms || []).slice(0, 3)) {
      const e = l.event || {}, p = e.params || {};
      const facts = [['Distance', `${numText(l.distance_km)} km from its centre`]];
      if (p.fate) facts.push(['Now', esc(String(p.fate).replace(/_/g, ' '))]);
      if (p.peak_height_m != null) facts.push(['Height', `${numText(p.height_m)} m of ${numText(p.peak_height_m)} m at its peak`]);
      if (p.thickness_m != null) facts.push(['Lava pile', `${numText(p.thickness_m)} m in ${numText(p.flows)} flows`]);
      if (p.cones_built != null) facts.push(['Cones', `${numText(p.standing)} standing (${numText(p.live)} live) of ${numText(p.cones_built)} built; the tallest ${numText(p.tallest_m)} m`]);
      if (p.volcanoes_built != null) facts.push(['Volcanoes', `${numText(p.standing)} standing (${numText(p.islands)} islands, ${numText(p.seamounts)} seamounts) of ${numText(p.volcanoes_built)} built`]);
      h += `<h5>Why here: ${esc(prettyKind(e.kind))}</h5><div class="kv">${rows(facts)}</div>` +
        `<div class="chain" style="margin-top:8px">${ev(e)}${(l.chain || []).map(ev).join('')}</div>`;
    }
    for (const b of bodies) if (b.site != null && !site(b.site)) { h += `<div class="desc" style="margin-top:8px">Site ${b.site} owns body ${b.id}; its box is not over this column.</div>`; break; }
    h += `<div class="callout" style="margin-top:12px">${ic('info')}<div><b>This run's own terrain.</b> What Deep Time decides for the column, read from the run's bake: it does not include what other mods' features add. The 3D preview shows the placed blocks.</div></div>` +
      `<div class="acts"><button class="btn btn-sm" data-dact="area">${ic('cube', 'ic-s')}Preview 256 × 256 here</button></div>`;
    $('details').innerHTML = h;
    $('details').dataset.curEv = '';
    $('details').dataset.pt = JSON.stringify({ i, lat: job.lat, lon: job.lon });
    setITab('details', true);
    if (narrow.matches) setMTab('inspect');
    evShownKey = ''; tickerDirty = true;
  }
  function showPoint(p, lat, lon) {
    const r = p.run;
    selectedEvent = null;
    pin = { pane: p.i, lat, lon };
    const [bx, bz] = blockXZ(r, lat, lon);
    const s = sampleValue(p, lat, lon);
    const { label: ln } = layerLabel(s ? s.name : layer);
    const v = valueText(s);
    $('details').innerHTML =
      `<div class="dh" style="--c:#fff"><span class="d" style="background:${s ? `rgb(${s.rgb.join(',')})` : '#fff'}"></span><div><h4>${esc(latLonText(lat, lon))}</h4><div class="id">x ${fmtInt(bx)} · z ${fmtInt(bz)} on C = ${fmtInt(circ(r))}</div></div></div>` +
      `<div class="kv"><div>${esc(ln || 'Layer')}</div><div>${esc(v || '—')}</div><div>Latitude</div><div>${esc(latText(lat))}</div><div>Longitude</div><div>${esc(lonText(lon))} of the seam</div>` +
      `<div>Block x, z</div><div class="tnum">${fmtInt(bx)}, ${fmtInt(bz)}</div><div>Run</div><div>${esc(runName(r))}</div></div>` +
      `<div class="callout" style="margin-top:12px">${ic('info')}<div>Values are read back from the map's colours, so they are approximate.</div></div>` +
      `<h5>Look inside</h5><div class="acts" style="margin-top:0"><button class="btn btn-sm" data-dact="inspect">${ic('crosshair', 'ic-s')}Inspect this column</button><button class="btn btn-sm" data-dact="area">${ic('cube', 'ic-s')}Preview 256 × 256 here</button></div>` +
      '<div class="desc">Each builds a 3D preview on the Mac (about 1–2 minutes).</div>';
    $('details').dataset.curEv = '';
    $('details').dataset.pt = JSON.stringify({ i: p.i, lat, lon });
    setITab('details', true);
    if (narrow.matches) setMTab('inspect');
    G.lon0 = lon; G.lat0 = clamp(lat, -80, 80); G.spin = false; spinButton(); globeDirty = true;
    mapDirty = true; evShownKey = ''; tickerDirty = true;
  }
  $('details').addEventListener('click', ev => {
    const b = ev.target.closest('[data-dact]');
    if (!b) return;
    const r = P().run;
    const act = b.dataset.dact;
    if (act === 'locate' || act === 'jump') {
      const e = r && r.byId.get(Number($('details').dataset.curEv));
      if (!e) return;
      if (act === 'locate') { const ll = placeLatLon(e); if (ll) { G.lon0 = ll[1]; G.lat0 = clamp(ll[0], -80, 80); globeDirty = true; } }
      else { setMode('pause'); show(P(), r.frameAt(e.t_start)); tp = r.frames[P().cur].t; }
      return;
    }
    const pt = JSON.parse($('details').dataset.pt || 'null');
    if (!pt) return;
    const p = panes[pt.i], run = p.run;
    if (!run || p.cur < 0) return;
    const f = run.frames[p.cur];
    if (act === 'inspect') { const [bx, bz] = blockXZ(run, pt.lat, pt.lon); Stage2.inspect(run, f, bx, bz, pt.lat, pt.lon); }
    else {
      const d = 128 / circ(run) * 360, b = boxInfo(run, clamp(pt.lat + d, -90, 90), clamp(pt.lat - d, -90, 90), pt.lon - d, pt.lon + d);
      p.boxLL = b; mapDirty = true;
      const [x0, z0] = blockXZ(run, b.north, b.west), [x1, z1] = blockXZ(run, b.south, b.east);
      Stage2.preview(run, f, { x0, z0, x1, z1 }, { north: b.north, south: b.south, west: b.west, east: b.east }, b.text);
    }
  });
  document.body.addEventListener('click', ev => {
    const el = ev.target.closest('[data-ev]');
    if (el && el.dataset.ev && P().run && !ev.target.closest('[data-dact]')) showEvent(P().run, Number(el.dataset.ev));
  });

  let runInfoTimer = null;
  function runInfoSoon() { if (!runInfoTimer) runInfoTimer = setTimeout(() => { runInfoTimer = null; runInfo(); }, 500); }
  function runInfo() {
    const r = P().run, box = $('runinfo');
    if (!r || !r.meta) { box.innerHTML = '<div class="empty"><p>No run open.</p></div>'; return; }
    const m = r.meta, pr = r.progress || {}, st = r.status || {};
    const state = st.state || 'queued';
    let h = `<div class="dh"><div style="min-width:0"><h4>${esc(m.label)}</h4><div class="id">${esc(r.id)}</div></div><span style="margin-left:auto" class="badge s-${esc(state)}">${esc(state)}</span></div>`;
    h += `<div class="chips" style="margin-bottom:12px">${runChips({ preset: m.preset, start: m.start, length_myr: m.length_myr, circumference: m.circumference, stages: m.stages, seed: runSeed(r.id) })}</div>`;
    if (r.placeholder()) h += `<div class="callout warn" style="margin-bottom:12px">${ic('alert')}<div><b>Placeholder: not the real simulation.</b> This run used the dummy pipeline (${esc((m.stages || []).join(', '))}), which exists only until the real presets replace the placeholders.</div></div>`;
    if (!r.finished() && pr.total_permille !== undefined) {
      const pg = progressOf(r.id, Object.assign({ length_myr: m.length_myr }, pr));
      h += `<div class="runcard" style="cursor:default;animation:none"><div class="prog" style="margin-top:0"><div class="bar"><span style="width:${100 * (pg.p || 0)}%"></span></div>` +
        `<div class="row"><span><b>${(100 * (pg.p || 0)).toFixed(1)}%</b> · ${esc(pr.stage || '')}</span><span>${esc(etaText(pg))}</span></div></div></div>`;
    }
    const runCap = (lastRuns.find(x => x.id === r.id) || {}).mem_cap;
    const rows = [
      ['Preset', `${esc(presetName(m.preset))} <span class="dim mono">${esc(m.preset)}</span>`], ['Start', esc(startName(m.start) || '—')],
      ['Length', esc(m.length_myr ? `${fmtInt(m.length_myr)} Myr` : '—')], ['Stages', esc((m.stages || []).join(' → '))],
      ['Size', esc(`${fmtInt(m.circumference)} blocks around`)],
      ['Elapsed', esc(pr.elapsed_ms !== undefined ? fmtDur(pr.elapsed_ms) : st.seconds ? fmtDur(st.seconds * 1000) : '—')],
      ['Peak memory', esc(pr.bytes_peak ? fmtBytes(pr.bytes_peak) : '—')], ['Memory cap', esc(runCap ? fmtBytes(runCap) : '—')], ['Threads', esc(m.threads || '—')],
      ['Created', esc(m.created_utc ? `${m.created_utc.replace('T', ' ').replace(/:\d\dZ$/, 'Z')} (${relTime(m.created_utc)})` : '—')],
      ['Core', esc(m.version || '—')],
    ];
    if (st.message) rows.push(['Message', esc(st.message)]);
    h += `<div class="kv">${rows.map(([a, b]) => `<div>${esc(a)}</div><div>${b}</div>`).join('')}</div>`;
    const lr = lastRuns.find(x => x.id === r.id);
    if (lr && lr.share_code) h += `<div class="acts"><button class="btn btn-sm" data-ract="copy">${ic('copy', 'ic-s')}Copy share code</button><button class="btn btn-sm" data-ract="settings">${ic('sliders', 'ic-s')}Load its settings</button></div>`;
    if (st.lines && st.lines.length) {
      h += '<h5>Summary</h5>';
      for (const line of st.lines) {
        const toks = line.split(/\s+/).filter(x => /^[a-z0-9_]+=/.test(x));
        if (toks.length >= 3) h += `<div class="stats" style="margin-bottom:8px">${toks.map(tk => { const [k, v] = tk.split('='); return `<div class="stat" title="${esc(k)}"><div class="k">${esc(k.replace(/_/g, ' '))}</div><div class="v">${esc(v)}</div></div>`; }).join('')}</div>`;
        else h += `<pre class="lines" style="margin-bottom:8px">${esc(line)}</pre>`;
      }
    }
    if (st.world) h += `<h5>World data</h5><pre class="lines">${esc(st.world)}</pre>`;
    box.innerHTML = h;
  }
  $('runinfo').addEventListener('click', async ev => {
    const b = ev.target.closest('[data-ract]');
    if (!b) return;
    const r = P().run, lr = r && lastRuns.find(x => x.id === r.id);
    if (!lr) return;
    if (b.dataset.ract === 'copy') copyText(lr.share_code, 'Share code copied');
    else { await importCode(lr.share_code); sideTab('new'); openSidebar(); }
  });

  // =================================================================================================
  // Top bar and the stage's empty states
  // =================================================================================================
  function runName(run) { return run ? (run.meta ? run.meta.label : run.id) : ''; }
  function runhead() {
    const r = P().run;
    $('rh-title').textContent = r ? runName(r) : 'No run open';
    const m = r && r.meta;
    const html = m ? runChips({ preset: m.preset, start: m.start, length_myr: m.length_myr, circumference: m.circumference, stages: m.stages, seed: runSeed(r.id) }, true) : '';
    if ($('rh-chips').dataset.h !== html) { $('rh-chips').dataset.h = html; $('rh-chips').innerHTML = html; }
  }
  function stageEmpty() {
    const el = $('stageempty');
    const none = !panes[0].run;
    el.hidden = !none;
    if (!none) return;
    const art = `<svg class="art"><use href="#i-planet"/></svg>`;
    if (TV) {
      el.innerHTML = connected || firstPoll ? `<div class="empty">${art}<h4>Waiting for a planet</h4><p>The next run started on the workbench appears here live.</p></div>`
        : `<div class="empty">${art}<h4>The workbench is not answering</h4><p>This screen reconnects on its own.</p></div>`;
      return;
    }
    if (!connected && !firstPoll) el.innerHTML = `<div class="empty">${art}<h4>The workbench is not answering</h4><p>Retrying every few seconds. The page reconnects on its own.</p><div class="spinner"></div></div>`;
    else if (firstPoll) el.innerHTML = `<div class="empty"><div class="spinner l"></div><p>Connecting to the Mac…</p></div>`;
    else if (!lastRuns.length) el.innerHTML = `<div class="empty">${art}<h4>No planets yet</h4><p>Pick a preset, choose how the continents start and how long a history to simulate, then press Run. You can watch it live.</p><button class="btn btn-primary" data-go="new">${ic('plus', 'ic-s')}New run</button></div>`;
    else el.innerHTML = `<div class="empty">${art}<h4>Open a run</h4><p>Choose one of the ${lastRuns.length} kept runs, or start a new one.</p><div style="display:flex;gap:8px"><button class="btn" data-go="runs">${ic('list', 'ic-s')}Runs</button><button class="btn btn-primary" data-go="new">${ic('plus', 'ic-s')}New run</button></div></div>`;
  }
  $('stageempty').addEventListener('click', ev => {
    const b = ev.target.closest('[data-go]');
    if (!b) return;
    sideTab(b.dataset.go); openSidebar();
    if (narrow.matches) setMTab(b.dataset.go);
  });

  // TV: the floating title, clock, badge and fresh events.
  function tvHud(p, f) {
    if (!TV) return;
    const r = p.run;
    $('tv-name').textContent = runName(r);
    $('tv-when').innerHTML = tHTML(f.t);
    $('tv-stage').textContent = `${stageText(f.stage)} · step ${f.n + 1}`;
    const b = badgeState(p);
    $('tv-badge').innerHTML = b ? `<span class="badge ${b[0]}">${b[0] === 's-live' ? '<span class="sdot s-live"></span>' : ''}${esc(b[1])}</span>` : '';
    const pr = r.progress || {};
    const pg = progressOf(r.id, Object.assign({ length_myr: r.meta && r.meta.length_myr }, pr));
    $('tv-sub').textContent = !r.finished() && pg.p !== null ? `${(100 * pg.p).toFixed(0)}% simulated · ${etaText(pg)}` : '';
  }
  function tvEvents() {
    if (!TV) return;
    const p = P(), r = p.run;
    if (!r || p.cur < 0) { $('tv-events').innerHTML = ''; return; }
    const t = r.frames[p.cur].t;
    const recent = r.events.filter(e => e.t_start <= t && e.t_start > t - 12e6 && !hiddenKinds.has(shortKind(e.kind))).sort((a, b) => b.t_start - a.t_start).slice(0, 5);
    $('tv-events').innerHTML = recent.map(e => `<div class="tvev" style="--c:${kindColour(e.kind)}"><i></i><span class="k">${esc(prettyKind(e.kind))}</span><span class="t">${esc(fmtT(e.t_start))}</span></div>`).join('');
  }

  // =================================================================================================
  // Controls
  // =================================================================================================
  function step(d) {
    const p = P(), r = p.run;
    if (!r || !r.frames.length) return;
    setMode('pause');
    show(p, clamp((p.cur < 0 ? r.frames.length - 1 : p.cur) + d, 0, r.frames.length - 1));
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
  function setRate(d) {
    const i = clamp(RATES.indexOf(rate) + d, 0, RATES.length - 1);
    rate = RATES[i];
    buttons();
  }
  function setTool(t) {
    tool = t;
    const vp = $('viewport');
    vp.classList.toggle('tool-inspect', t === 'inspect');
    vp.classList.toggle('tool-area', t === 'area');
    const hint = $('maphint');
    hint.hidden = t === 'select' || TV;
    if (t === 'inspect') hint.innerHTML = `${ic('crosshair')}Click a column to inspect it <span class="dim">· from the run's bake</span><button class="btn btn-ghost btn-sm" data-tool="select">Done <kbd>Esc</kbd></button>`;
    if (t === 'area') hint.innerHTML = `${ic('box')}Drag a box, or click for 256 × 256 blocks <span class="dim">· 1–2 min</span><button class="btn btn-ghost btn-sm" data-tool="select">Done <kbd>Esc</kbd></button>`;
    for (const p of panes) p.drag = null;
    mapDirty = true;
    buttons();
  }
  function sideTab(name) {
    for (const b of document.querySelectorAll('[data-stab]')) b.setAttribute('aria-selected', String(b.dataset.stab === name));
    $('pane-runs').hidden = name !== 'runs';
    $('pane-new').hidden = name !== 'new';
    store.set('stab', name);
  }
  function openSidebar() {
    const app = $('app');
    if (medium.matches && !narrow.matches) app.classList.add('drawer');
    else { app.classList.remove('noside'); store.set('noside', false); }
    setTimeout(layout, 280);
  }
  function toggleSidebar() {
    const app = $('app');
    if (narrow.matches) { setMTab(document.body.dataset.mtab === 'runs' ? 'layers' : 'runs'); return; }
    if (medium.matches) app.classList.toggle('drawer');
    else { app.classList.toggle('noside'); store.set('noside', app.classList.contains('noside')); }
    $('sidebar').inert = app.classList.contains('noside') && !medium.matches;
    setTimeout(() => { layout(); globeDirty = true; }, 280);
  }
  function setMTab(name) {
    document.body.dataset.mtab = name;
    for (const b of document.querySelectorAll('[data-mt]')) b.setAttribute('aria-selected', String(b.dataset.mt === name));
    if (name === 'runs' || name === 'new') sideTab(name);
    if (name === 'globe') { globeSize(); globeDirty = true; }
    if (name === 'layers') legendDirty = true;
  }
  $('rewind').addEventListener('click', rewind);
  $('back').addEventListener('click', () => step(-1));
  $('fwd').addEventListener('click', () => step(1));
  $('play').addEventListener('click', togglePlay);
  $('livebtn').addEventListener('click', () => setMode('live'));
  $('slower').addEventListener('click', () => setRate(-1));
  $('faster').addEventListener('click', () => setRate(1));
  $('markers').addEventListener('click', () => { markersOn = !markersOn; mapDirty = globeDirty = true; buttons(); });
  $('gridbtn').addEventListener('click', () => { gridOn = !gridOn; store.set('grid', gridOn); mapDirty = globeDirty = true; buttons(); });
  for (const b of document.querySelectorAll('[data-cmp]')) b.addEventListener('click', () => { const m = b.dataset.cmp; if (m === 'off') setCompare(false); else setCompare(true, m); });
  document.addEventListener('click', ev => { const b = ev.target.closest('[data-tool]'); if (b) setTool(tool === b.dataset.tool && b.classList.contains('s2-tool') ? 'select' : b.dataset.tool); });
  $('togglesidebar').addEventListener('click', toggleSidebar);
  $('app').addEventListener('click', ev => { if (ev.target === $('app') && $('app').classList.contains('drawer')) $('app').classList.remove('drawer'); });
  for (const b of document.querySelectorAll('[data-stab]')) b.addEventListener('click', () => sideTab(b.dataset.stab));
  for (const b of document.querySelectorAll('[data-mt]')) b.addEventListener('click', () => setMTab(b.dataset.mt));
  for (const b of document.querySelectorAll('[data-collapse]')) {
    const card = $(b.dataset.collapse);
    if (store.get(`collapsed.${card.id}`, false)) { card.classList.add('collapsed'); b.setAttribute('aria-expanded', 'false'); }
    b.addEventListener('click', () => {
      const c = card.classList.toggle('collapsed');
      b.setAttribute('aria-expanded', String(!c));
      store.set(`collapsed.${card.id}`, c);
      if (!c) { globeSize(); globeDirty = true; }
    });
  }
  $('bselect').addEventListener('change', () => { const id = $('bselect').value; if (id) open(1, id); });
  $('swapab').addEventListener('click', () => {
    const a = panes[0].run; panes[0].run = panes[1].run; panes[1].run = a;
    for (const p of panes) { p.cur = -1; p.drawn = null; p.pix = null; p.pixImg = null; paneHead(p); }
    texImg = null; mode = 'live';
    tickerDirty = globeDirty = mapDirty = dockDirty = tlDirty = layersDirty = legendDirty = true;
    hashState(); runsDirty = true; bOptions(); runhead(); runInfo(); buttons();
  });
  function bOptions() {
    const sel = $('bselect');
    const a = panes[0].run && panes[0].run.id, b = panes[1].run && panes[1].run.id;
    const html = '<option value="">Choose run B…</option>' + lastRuns.filter(r => r.id !== a).map(r => `<option value="${esc(r.id)}"${r.id === b ? ' selected' : ''}>B · ${esc(r.label)}</option>`).join('');
    if (sel.dataset.h !== html) { sel.dataset.h = html; sel.innerHTML = html; }
  }
  $('fsbtn').addEventListener('click', () => { if (document.fullscreenElement) document.exitFullscreen(); else if (root.requestFullscreen) root.requestFullscreen().catch(() => {}); });
  $('helpbtn').addEventListener('click', help);
  for (const p of panes) p.el.addEventListener('pointerdown', () => setFocus(p.i));

  document.addEventListener('keydown', ev => {
    const k = ev.key;
    if (k === 'Escape') {
      if (!$('modal').hidden) { closeModal(); return; }
      if (!$('layerpop').hidden && !narrow.matches) { layerPop(false); $('lpickbtn').focus(); return; }
      if ($('app').classList.contains('drawer')) { $('app').classList.remove('drawer'); return; }
      if (tool !== 'select') { setTool('select'); return; }
      for (const p of panes) p.boxLL = null;
      pin = null; selectedEvent = null; mapDirty = true;
      return;
    }
    const control = ev.target.closest ? ev.target.closest('input, select, textarea, button, [role=separator]') : null;
    if (control && !(k === ' ' && control.id === 'timeline')) return;   // a focused control takes its own keys
    if (ev.metaKey || ev.ctrlKey || ev.altKey) return;
    if (!$('modal').hidden) return;
    if (k === ' ') { ev.preventDefault(); togglePlay(); }
    else if (k === 'ArrowLeft') { ev.preventDefault(); step(ev.shiftKey ? -10 : -1); }
    else if (k === 'ArrowRight') { ev.preventDefault(); step(ev.shiftKey ? 10 : 1); }
    else if (k === 'ArrowUp') { ev.preventDefault(); cycleLayer(-1); }
    else if (k === 'ArrowDown') { ev.preventDefault(); cycleLayer(1); }
    else if (k === 'Home') { ev.preventDefault(); rewind(); }
    else if (k === 'End' || k === 'l') setMode('live');
    else if (k === 'g') { G.spin = !G.spin; spinButton(); }
    else if (k === 'm') $('markers').click();
    else if (k === 'd') $('depbtn').click();
    else if (k === 'c') { if (comparing) setCompare(false); else setCompare(true); }
    else if (k === 'e') toggleSidebar();
    else if (k === 'v') setTool('select');
    else if (k === 'i') setTool('inspect');
    else if (k === 'b') setTool('area');
    else if (k === '[') setRate(-1);
    else if (k === ']') setRate(1);
    else if (k === '?') help();
    else if (k === 'f' && root.requestFullscreen) root.requestFullscreen().catch(() => {});
    else if (/^[1-9]$/.test(k)) { const n = layerByNumber(Number(k)); if (n) setLayer(n); }
  });
  $('timeline').addEventListener('keydown', ev => { if (ev.key === ' ') { ev.preventDefault(); togglePlay(); } });
  if (TV) {   // hide the cursor after 3 s without moving it
    let idle = null;
    const wake = () => { root.classList.remove('idle'); clearTimeout(idle); idle = setTimeout(() => root.classList.add('idle'), 3000); };
    document.addEventListener('mousemove', wake);
    wake();
  }

  // =================================================================================================
  // Tooltips, toasts, modals
  // =================================================================================================
  {
    const tip = $('tip');
    let timer = null, cur = null, lastHide = 0;
    const place = el => {
      const r = el.getBoundingClientRect(), tr = tip.getBoundingClientRect();
      let x = r.left + r.width / 2 - tr.width / 2, y = r.top - tr.height - 8;
      if (y < 6) y = r.bottom + 8;
      tip.style.left = `${clamp(x, 6, innerWidth - tr.width - 6)}px`; tip.style.top = `${y}px`;
    };
    const showTip = el => {
      const text = el.dataset.tip;
      if (!text || TV) return;
      tip.innerHTML = `${esc(text)}${el.dataset.key ? `<kbd>${esc(el.dataset.key)}</kbd>` : ''}`;
      tip.classList.add('show'); place(el); cur = el;
    };
    const hide = () => { clearTimeout(timer); if (cur) lastHide = performance.now(); cur = null; tip.classList.remove('show'); };
    document.addEventListener('pointerover', ev => {
      if (ev.pointerType === 'touch') return;
      const el = ev.target.closest('[data-tip]');
      if (el === cur) return;
      hide();
      if (!el) return;
      timer = setTimeout(() => showTip(el), performance.now() - lastHide < 400 ? 0 : 450);
    });
    document.addEventListener('focusin', ev => { const el = ev.target.closest && ev.target.closest('[data-tip]'); if (el && el.matches(':focus-visible')) { hide(); showTip(el); } });
    document.addEventListener('focusout', hide);
    document.addEventListener('pointerdown', hide);
  }
  function toast(msg, kind = 'info', ms = 3800, action) {
    const el = document.createElement('div');
    el.className = `toast ${kind}`;
    el.setAttribute('role', kind === 'bad' ? 'alert' : 'status');
    el.innerHTML = `${ic(kind === 'ok' ? 'check' : kind === 'bad' || kind === 'warn' ? 'alert' : 'info')}<span>${esc(msg)}</span>`;
    if (action) {
      const b = document.createElement('button');
      b.className = 'btn btn-sm'; b.textContent = action.label;
      b.addEventListener('click', () => { action.run(); dismiss(); });
      el.appendChild(b);
    }
    $('toasts').appendChild(el);
    while ($('toasts').children.length > 4) $('toasts').firstChild.remove();
    const dismiss = () => { el.classList.add('out'); setTimeout(() => el.remove(), 260); };
    setTimeout(dismiss, ms);
  }
  async function copyText(text, done) {
    try { await navigator.clipboard.writeText(text); }
    catch (e) {
      const t = document.createElement('textarea'); t.value = text; document.body.appendChild(t); t.select();
      try { document.execCommand('copy'); } catch (e2) { /* nothing more to try */ }
      t.remove();
    }
    toast(done || 'Copied', 'ok');
  }
  let modalOpener = null;
  function modal(title, body, acts, small) {
    modalOpener = document.activeElement;
    $('modaltitle').textContent = title;
    $('modalacts').innerHTML = acts || '';
    $('modalbody').innerHTML = body;
    $('modalcard').classList.toggle('small', !!small);
    $('modal').hidden = false;
    $('modalclose').focus();
  }
  function closeModal() {
    $('modal').hidden = true;
    $('modalbody').innerHTML = '';
    if (modalOpener && modalOpener.focus) modalOpener.focus();
  }
  $('modalclose').addEventListener('click', closeModal);
  $('modal').addEventListener('click', ev => { if (ev.target === $('modal')) closeModal(); });
  function help() {
    const K = (keys, what) => `<span>${keys.map(k => `<kbd>${esc(k)}</kbd>`).join('')}</span><span>${esc(what)}</span>`;
    modal('Keyboard shortcuts', `<div class="keys">${[
      K(['Space'], 'Play or pause'), K(['←', '→'], 'One step back or forward (Shift: ten)'), K(['Home'], 'Back to the start'),
      K(['L'], 'Follow the run live (End too)'), K(['[', ']'], 'Slower, faster'), K(['1', '…', '9'], 'Layers in the order they appeared'), K(['↑', '↓'], 'Previous or next layer'),
      K(['V'], 'Select tool: events and point values'), K(['I'], 'Inspect a column in 3D'), K(['B'], 'Preview an area in 3D'),
      K(['C'], 'Compare two runs'), K(['M'], 'Event markers on or off'), K(['D'], 'Deposits on or off'), K(['G'], 'Spin the globe'), K(['E'], 'Runs and editor'),
      K(['F'], 'Full screen'), K(['Esc'], 'Back to Select; clear the box and pin'), K(['?'], 'This list'),
    ].join('')}</div>`, '', true);
  }

  // =================================================================================================
  // The runs list (polled) and the Mac's status
  // =================================================================================================
  let lastRuns = [], runsDirty = true, lastRunning = null, firstPoll = true, connected = false, wasConnected = null;
  const cards = new Map();   // run id -> card element
  const seeds = new Map();   // run id -> seed (from the runs list)
  const runSeed = id => seeds.get(id);
  const groupEls = { active: document.createElement('div'), done: document.createElement('div') };
  groupEls.active.className = groupEls.done.className = 'runs-group';
  const startName = s => (s ? (L(`timeline.start.${s}`) || cap1(String(s).replace(/_/g, ' '))) : '');
  function isPlaceholder(preset, stages) {
    const p = catalog && catalog.presets.find(x => x.name === preset);
    if (p) return !!p.placeholder;
    return (stages || []).some(s => /^dummy/.test(s));
  }
  function runChips(r, compact) {
    const out = [];
    if (isPlaceholder(r.preset, r.stages)) out.push(`<span class="chip" style="color:#ffd07a;border-color:rgba(244,183,64,.35);background:var(--queued-soft)" data-tip="Placeholder: the dummy pipeline, not the real simulation">${ic('alert')}placeholder</span>`);
    out.push(`<span class="chip" data-tip="${esc(r.preset)}">${ic('globe')}${esc(presetName(r.preset))}</span>`);
    if (r.start) out.push(`<span class="chip">${esc(startName(r.start))}</span>`);
    if (r.length_myr) out.push(`<span class="chip"><b>len</b>${esc(fmtInt(r.length_myr))} Myr</span>`);
    if (r.seed !== undefined && r.seed !== null) out.push(`<span class="chip"><b>seed</b>${esc(r.seed)}</span>`);
    return compact ? out.slice(0, 4).join('') : out.join('');
  }
  function runCard(r) {
    let el = cards.get(r.id);
    if (!el) {
      el = document.createElement('article');
      el.className = 'runcard';
      el.dataset.id = r.id;
      el.tabIndex = 0;
      el.innerHTML = '<div class="top"><span class="sdot"></span><span class="pane-tag" hidden></span><span class="lbl"></span><span class="badge"></span></div>' +
        '<div class="chips"></div><div class="prog" hidden><div class="bar"><span></span></div><div class="row"><span class="l"></span><span class="r"></span></div></div>' +
        '<div class="meta"></div><div class="msg" hidden></div><div class="acts"></div><div class="confirm" hidden></div>';
      cards.set(r.id, el);
    }
    const inA = panes[0].run && panes[0].run.id === r.id, inB = comparing && panes[1].run && panes[1].run.id === r.id;
    el.classList.toggle('inA', !!inA); el.classList.toggle('inB', !!inB);
    const tag = el.querySelector('.pane-tag');
    tag.hidden = !(inA || inB); tag.textContent = inA ? 'A' : 'B';
    el.querySelector('.sdot').className = `sdot s-${r.state}`;
    el.querySelector('.lbl').textContent = r.label;
    const pg = progressOf(r.id, r);
    const bd = el.querySelector('.badge');
    bd.className = `badge s-${r.state}`;
    bd.textContent = r.state === 'queued' ? `Queued #${r.queue_position}` : r.state === 'running' ? 'Running' : r.state;
    const chips = runChips(r);
    const ch = el.querySelector('.chips');
    if (ch.dataset.h !== chips) { ch.dataset.h = chips; ch.innerHTML = chips; }
    const prog = el.querySelector('.prog');
    prog.hidden = r.state !== 'running';
    if (r.state === 'running') {
      prog.querySelector('.bar span').style.width = `${100 * (pg.p || 0)}%`;
      prog.querySelector('.l').innerHTML = `<b>${(100 * (pg.p || 0)).toFixed(0)}%</b>${r.t_years !== null && r.t_years !== undefined ? ` · at ${esc(fmtT(-Math.abs(r.t_years)))}` : ''}`;
      prog.querySelector('.r').textContent = etaText(pg);
    }
    const when = relTime(r.created_utc);
    const meta = r.state === 'queued' ? ['waiting for the Mac', when ? `queued ${when}` : '']
      : [`${fmtInt(r.frames)} frames`, `${fmtInt(r.events)} events`, r.seconds ? fmtDur(r.seconds * 1000) : '', when];
    el.querySelector('.meta').textContent = meta.filter(Boolean).join(' · ');
    const msg = el.querySelector('.msg');
    msg.hidden = !(r.message && r.state === 'failed'); msg.textContent = r.message || '';
    const acts = [];
    if (!inA) acts.push(['view', 'Open', 'play', 'btn-sm']);
    if (!inA && !inB) acts.push(['compare', 'Compare', 'split', 'btn-sm btn-ghost']);
    acts.push(['settings', 'Settings', 'sliders', 'btn-sm btn-ghost']);
    if (r.state === 'queued' || r.state === 'running') acts.push(['cancel', 'Cancel', 'stop', 'btn-sm btn-ghost btn-danger']);
    const key = acts.map(a => a[0]).join(',');
    const box = el.querySelector('.acts');
    if (box.dataset.key !== key) {
      box.dataset.key = key;
      box.innerHTML = acts.map(([a, t, i, c]) => `<button class="btn ${c}" data-act="${a}" data-tip="${esc({ view: 'Open in the map (A)', compare: 'Compare with the open run (B)', settings: 'Load its settings into the editor', cancel: 'Stop this run' }[a])}">${ic(i, 'ic-s')}${t}</button>`).join('');
    }
    return el;
  }
  // How far a run is: the share of its history simulated so far (the core's permille weighs the
  // later stages heavily and reads low for most of a run), and the time left at the pace so far.
  const seen = new Map();
  function progressOf(id, x) {
    let p = null;
    if (x.length_myr && x.t_years !== null && x.t_years !== undefined && isFinite(x.t_years)) p = clamp(1 - Math.abs(x.t_years) / (x.length_myr * 1e6), 0, 1);
    else if (x.total_permille !== null && x.total_permille !== undefined) p = x.total_permille / 1000;
    let eta = null;
    if (p !== null && p >= 0.03 && p < 1) {
      if (x.elapsed_ms) eta = x.elapsed_ms * (1 - p) / p;
      else {
        const s = seen.get(id), now = Date.now();
        if (!s) seen.set(id, { t: now, p });
        else if (p > s.p) eta = (now - s.t) * (1 - p) / (p - s.p);
      }
    }
    return { p, eta };
  }
  const etaText = pg => pg.p >= 1 ? 'finishing…' : pg.eta !== null ? `${fmtDur(pg.eta)} left` : 'estimating…';
  $('runlist').addEventListener('click', async ev => {
    const card = ev.target.closest('.runcard');
    if (!card) { const g = ev.target.closest('[data-go]'); if (g) { sideTab(g.dataset.go); if (narrow.matches) setMTab(g.dataset.go); } return; }
    const id = card.dataset.id, r = lastRuns.find(x => x.id === id);
    const b = ev.target.closest('button[data-act], button[data-cact]');
    if (b && b.dataset.cact) {
      const cf = card.querySelector('.confirm');
      if (b.dataset.cact === 'yes') {
        cf.hidden = true;
        const res = await api(`api/runs/${encodeURIComponent(id)}/cancel`, {});
        if (!res.ok) toast(res.body.error || 'Could not cancel', 'bad');
        else toast(r && r.state === 'running' ? 'The run stops after its current step and keeps its frames' : 'Removed from the queue', 'ok');
        poll();
      } else cf.hidden = true;
      return;
    }
    if (!b) { if (!(panes[0].run && panes[0].run.id === id)) { open(0, id); setFocus(0); } if (medium.matches && !narrow.matches) $('app').classList.remove('drawer'); return; }
    const act = b.dataset.act;
    if (act === 'view') { open(0, id); setFocus(0); if (medium.matches && !narrow.matches) $('app').classList.remove('drawer'); }
    else if (act === 'compare') { open(1, id); if (!comparing) setCompare(true); }
    else if (act === 'cancel') {
      const cf = card.querySelector('.confirm');
      cf.innerHTML = `<div><b>Cancel ${esc(r ? r.label : id)}?</b> ${r && r.state === 'running' ? 'It stops after its current step and keeps the frames it made.' : 'It leaves the queue.'}</div>` +
        '<div class="row"><button class="btn btn-sm btn-ghost" data-cact="no">Keep it</button><button class="btn btn-sm btn-danger" data-cact="yes">Cancel run</button></div>';
      cf.hidden = false;
    } else if (act === 'settings' && r) { await importCode(r.share_code); sideTab('new'); if (narrow.matches) setMTab('new'); toast(`Loaded the settings of ${r.label}`, 'ok'); }
  });
  $('runlist').addEventListener('keydown', ev => { if ((ev.key === 'Enter' || ev.key === ' ') && ev.target.classList.contains('runcard')) { ev.preventDefault(); ev.target.click(); } });

  async function poll() {
    const res = await api('api/runs');
    if (!res.ok) {
      setConn(false, res.status ? `answered ${res.status}` : 'not answering');
      if (firstPoll) { firstPoll = false; }
      $('mactext').textContent = 'No answer from the workbench';
      $('macdot').className = 'sdot s-failed';
      if (!lastRuns.length) $('runlist').innerHTML = `<div class="empty">${ic('alert', 'art')}<h4>Can’t reach the workbench</h4><p>${esc(res.body.error || '')} The list comes back when it answers.</p></div>`;
      stageEmpty();
      return;
    }
    setConn(true, 'connected');
    const j = res.body;
    lastRuns = j.runs;
    for (const r of j.runs) if (r.seed !== undefined && r.seed !== null) seeds.set(r.id, r.seed);
    const running = j.runs.find(r => r.state === 'running');
    // Top bar: the Mac's one run slot.
    const rpg = running ? progressOf(running.id, running) : null;
    $('macbar').style.width = running ? `${100 * (rpg.p || 0)}%` : '0%';
    $('macdot').className = `sdot ${running ? 's-running' : 's-idle'}`;
    if (running) {
      $('mactext').innerHTML = `<b>${esc(running.label)}</b> · ${(100 * (rpg.p || 0)).toFixed(0)}% · ${esc(etaText(rpg))}${j.queue.length ? ` · ${j.queue.length} waiting` : ''}`;
    } else $('mactext').innerHTML = `Mac idle · ${j.runs.length} run${j.runs.length === 1 ? '' : 's'} kept`;
    $('runscount').textContent = j.runs.length;
    const list = $('runlist');
    if (!j.runs.length) {
      if (!list.querySelector('.empty.norun')) list.innerHTML = `<div class="empty norun">${ic('list', 'art')}<h4>No runs yet</h4><p>Runs you start appear here, with their progress. The Mac keeps the newest ${j.keep}.</p><button class="btn btn-primary" data-go="new">${ic('plus', 'ic-s')}New run</button></div>`;
    } else {
      const active = j.runs.filter(r => r.state === 'running' || r.state === 'queued').sort((a, b) => (a.state === 'running' ? -1 : b.state === 'running' ? 1 : (a.queue_position || 0) - (b.queue_position || 0)));
      const done = j.runs.filter(r => !active.includes(r));
      const waiting = active.filter(r => r.state === 'queued').length;
      groupEls.active.innerHTML = `<span>On the Mac</span><span>${active.length - waiting} running · ${waiting} of ${j.queue_max} waiting</span>`;
      groupEls.done.innerHTML = `<span>Finished</span><span>newest ${j.keep} kept</span>`;
      const want = [...(active.length ? [groupEls.active, ...active.map(runCard)] : []), ...(done.length ? [groupEls.done, ...done.map(runCard)] : [])];
      const have = [...list.children];
      if (want.length !== have.length || want.some((el, i) => el !== have[i])) list.replaceChildren(...want);
      for (const id of [...cards.keys()]) if (!j.runs.some(r => r.id === id)) cards.delete(id);
    }
    // Which runs to show: the URL's, else the running one, else the newest; the TV follows new runs.
    if (firstPoll) {
      firstPoll = false;
      const h = H0;
      if (h.get('layer')) layer = h.get('layer');
      if (h.get('cmp')) cmpMode = h.get('cmp') === 'side' ? 'side' : 'split';
      const a = h.get('a') || Q.get('run');
      const pick = (a && j.runs.find(r => r.id === a)) || running || j.runs.find(r => r.frames > 0) || j.runs[0];
      if (pick) open(0, pick.id);
      const b = h.get('b');
      if (b && j.runs.find(r => r.id === b)) { open(1, b); setCompare(true, cmpMode); }
      if (!TV && !j.runs.length) sideTab('new');
    } else if (running && running.id !== lastRunning && (TV || !panes[0].run)) {
      open(0, running.id);
    }
    lastRunning = running ? running.id : null;
    runsDirty = false;
    bOptions();
    stageEmpty();
    runButton();
  }
  function setConn(ok, text) {
    connected = ok;
    const c = $('conn');
    c.className = `conn ${ok ? 'ok' : 'bad'}`;
    c.querySelector('.txt').textContent = text;
    if (wasConnected === true && !ok) toast('Lost the workbench: reconnecting…', 'warn', 5000);
    if (wasConnected === false && ok) toast('Reconnected', 'ok');
    wasConnected = ok;
  }
  setInterval(() => poll(), 1500);
  setInterval(() => { for (const r of lastRuns) runCard(r); }, 30000);   // relative times

  // =================================================================================================
  // The editor: the Create World setup page's controls, from the game's own catalogue
  // =================================================================================================
  let catalog = null, eff = null, effSeq = 0, effTimer = null, estimating = false;
  let showPlaceholders = store.get('showPlaceholders', false);
  // cap: the memory cap field in GiB; the server sets it to the workbench's default (deeptime watch
  // --mem-cap) when the catalogue arrives. 0 or empty means "the default", as the server reads it.
  const form = { preset: null, imported: null, start: '', length: '', fields: {}, cap: 0 };
  const defaultCapGiB = () => catalog && catalog.mem_cap_bytes ? catalog.mem_cap_bytes / 2 ** 30 : 4;
  const capGiB = () => form.cap > 0 ? form.cap : defaultCapGiB();
  // What the server is told: nothing (0) while the field holds the default, so the server's own default applies.
  const capToSend = () => form.cap === 0 || Math.abs(form.cap - defaultCapGiB()) < 1e-9 ? 0 : form.cap;
  const fmtCap = g => `${Number(g.toFixed(2))} GiB`;
  const L = (key, ...args) => {
    let s = catalog && catalog.labels[`deeptime.editor.${key}`];
    if (s === undefined || s === null) return null;
    for (const a of args) s = s.replace('%s', a);
    return s;
  };
  // Families as people know them; the raw preset names stay as secondary text.
  const FAMILY = {
    earthlike_quick: ['Earth-like · quick', 'The whole simulation: plates, climate, surface, structure and deposits, at the quick resolution.'],
    earthlike_full: ['Earth-like · full', 'The whole simulation at the full resolution: slower, finer.'],
    pipeline_quick: ['Earth-like · quick', 'The whole simulation: plates, climate, surface, structure and deposits, at the quick resolution.'],
    pipeline_full: ['Earth-like · full', 'The whole simulation at the full resolution: slower, finer.'],
    plates_quick: ['Plates only · quick', 'Plate tectonics alone, at the quick resolution: the fastest look at continents moving.'],
    plates_full: ['Plates only · full', 'Plate tectonics alone, at the full resolution.'],
  };
  const sizeShort = c => `${Math.round(c / 1024)}k`;
  function familyName(fam) {
    if (FAMILY[fam]) return FAMILY[fam][0];
    const l = L(`preset.${fam}`) || fam.replace(/_/g, ' ');
    // Only a family whose presets are all placeholders is called one (the earthlike_* ones are real since P1-08).
    const ps = catalog ? catalog.presets.filter(p => p.family === fam) : [];
    return ps.length && ps.every(p => p.placeholder) ? `Placeholder · ${l}` : l;
  }
  function presetName(name) {
    if (!name) return '';
    const p = catalog && catalog.presets.find(x => x.name === name);
    if (!p) {   // a preset the page no longer lists (the pipeline and plates presets): its family, still
      const m = /^(.*)_(\d+)k$/.exec(String(name));
      return m && FAMILY[m[1]] ? `${FAMILY[m[1]][0]} · ${m[2]}k` : String(name).replace(/_/g, ' ');
    }
    return `${familyName(p.family)} · ${sizeShort(p.circumference)}`;
  }
  const sizeLabel = c => L(`size.${c / 1024}k`) || L('size.other', c.toLocaleString('en')) || `${c} blocks`;
  const START_ART = {
    supercontinent: '<svg viewBox="0 0 48 32"><rect x="1" y="1" width="46" height="30" rx="6" fill="rgba(108,196,255,.14)"/><path d="M14 9c5-4 15-3 20 2s4 12-3 14-15 1-18-4-4-9 1-12z" fill="currentColor" opacity=".85"/></svg>',
    scattered_continents: '<svg viewBox="0 0 48 32"><rect x="1" y="1" width="46" height="30" rx="6" fill="rgba(108,196,255,.14)"/><path d="M7 8c3-2 7-1 8 2s-2 6-5 5-5-4-3-7zM28 6c4-1 8 1 7 4s-6 4-8 2-2-5 1-6zM20 19c3-2 7-1 7 3s-5 4-7 2-2-3 0-5zM36 18c3 0 5 2 4 5s-5 3-6 1 0-6 2-6z" fill="currentColor" opacity=".85"/></svg>',
    random: '<svg viewBox="0 0 48 32"><rect x="1" y="1" width="46" height="30" rx="6" fill="rgba(108,196,255,.14)"/><rect x="16" y="7" width="17" height="17" rx="4" transform="rotate(12 24 16)" fill="none" stroke="currentColor" stroke-width="2"/><circle cx="21" cy="12" r="1.6" fill="currentColor"/><circle cx="24.5" cy="16" r="1.6" fill="currentColor"/><circle cx="28" cy="20" r="1.6" fill="currentColor"/></svg>',
  };
  const QUICK_LENGTHS = [100, 250, 500, 1000];

  function buildForm() {
    const sec = (id, title, desc, body, extra = '') => `<section class="ed-sec" data-sec="${id}"><header><div class="t"><h3>${title}${extra}</h3>${desc ? `<p>${esc(desc)}</p>` : ''}</div>${ic('chev', 'caret')}</header><div class="body">${body}</div></section>`;
    let h = '';
    // Preset: the real simulation first; placeholders only on request.
    h += sec('preset', 'Preset', 'What to simulate, and how big the world is.',
      `<div class="presetlist" id="f_presets" role="radiogroup" aria-label="Preset"></div>
       <div class="field"><div class="fh"><span class="lab">Size</span><span class="val" id="f_sizeval"></span></div><div class="seg full" id="f_sizes" role="group" aria-label="Size"></div><div class="hint">${esc(L('size.tooltip') || '')}</div></div>
       <div id="f_presetwarn"></div>
       <label class="devtoggle"><span class="switch"><input type="checkbox" id="f_showph"${showPlaceholders ? ' checked' : ''}><span></span></span>Show placeholder presets (dev)</label>`);
    h += sec('timeline', esc(L('section.timeline') || 'Timeline'), `${L('timeline.tooltip') || 'How the continents start.'} Then how many million years of history to simulate.`,
      `<div class="field"><div class="fh"><span class="lab">${esc(L('timeline.start') || 'Start')}</span></div><div class="startcards" id="f_start" role="radiogroup" aria-label="Start">` +
      catalog.start_configs.map(s => `<button type="button" class="startcard" role="radio" aria-checked="false" data-start="${esc(s)}">${START_ART[s] || START_ART.random}${esc(L(`timeline.start.${s}`) || s)}</button>`).join('') + '</div></div>' +
      `<div class="field"><div class="fh"><label for="f_length">History length</label><span class="val">${esc(L('timeline.length.hint', catalog.length_myr.min, fmtInt(catalog.length_myr.max)) || '')} Myr</span></div>` +
      `<div class="ctl"><input type="range" id="f_lenr" min="0" max="1000" aria-label="History length"><div class="withunit"><input class="input" type="number" id="f_length" min="${catalog.length_myr.min}" max="${catalog.length_myr.max}" step="any" placeholder="—"><span class="unit">Myr</span></div></div>` +
      `<div class="quick" id="f_quick">${QUICK_LENGTHS.map(v => `<button type="button" data-len="${v}">${fmtInt(v)} Myr</button>`).join('')}</div></div>`);
    // Not the game's tooltip ("Default 4 GiB"): this workbench has its own default, and refuses a run over its cap.
    h += sec('memory', esc(L('section.memory') || 'Memory'), `The simulation's own memory. The default is ${fmtCap(defaultCapGiB())}; a run whose estimate is over its cap is refused before it starts, not partway through. Raise it for one run if the Mac has the RAM.`,
      `<div class="field"><div class="fh"><label for="f_cap">${esc(L('cap') || 'Memory cap (GiB)')}</label><span class="val" id="f_capval"></span></div><div class="ctl"><input type="range" id="f_capr" min="1" max="${Math.max(32, Math.ceil(defaultCapGiB()))}" step="0.25" aria-label="Memory cap"><div class="withunit"><input class="input" type="number" id="f_cap" min="${catalog.min_cap_gib || 0.25}" step="0.25" value="${form.cap}"><span class="unit">GiB</span></div></div></div>`);
    const sections = [];
    for (const f of catalog.fields) if (!sections.includes(f.section)) sections.push(f.section);
    for (const s of sections) {
      let body = '';
      for (const f of catalog.fields.filter(x => x.section === s)) {
        const name = L(`field.${f.id}`) || f.id;
        let ctl;
        const isSeed = f.kind === 'number' && f.integer && f.id === 'seed';   // only the seed gets the dice
        if (isSeed) ctl = `<div class="ctl"><input class="input" type="number" data-f="${f.id}" id="fi_${f.id}" step="1" min="${f.min}"><button type="button" class="btn btn-icon" data-dice="${f.id}" aria-label="A random seed" data-tip="A random seed">${ic('dice')}</button></div>`;
        else if (f.kind === 'number') {
          const log = f.min > 0 && f.max / f.min >= 100;
          ctl = `<div class="ctl"><input type="range" data-r="${f.id}" min="0" max="1000" ${log ? 'data-log="1"' : ''} aria-label="${esc(name)}"><div class="withunit"><input class="input" type="number" data-f="${f.id}" id="fi_${f.id}" step="${f.integer ? 1 : 'any'}" min="${f.min}" max="${f.max}"><span class="unit">${esc(f.unit || '')}</span></div></div>`;
        } else if (f.kind === 'toggle') ctl = `<label class="switch"><input type="checkbox" data-f="${f.id}" id="fi_${f.id}"><span></span></label>`;
        else ctl = `<select class="input" data-f="${f.id}" id="fi_${f.id}">${(f.options || []).map(o => `<option value="${esc(o)}">${esc(L(`option.${f.id}.${o}`) || o)}</option>`).join('')}</select>`;
        const stops = (f.stops || []).length ? `<div class="quick" data-stops="${f.id}">${f.stops.map(s => `<button type="button" data-stopv="${s.value}">${esc(s.label)} · ${s.value}</button>`).join('')}</div>` : '';
        const num = v => f.integer ? fmtInt(v) : numText(v);
        const range = f.kind === 'number' && !isSeed ? `${num(f.min)} – ${num(f.max)}${f.unit ? ` ${f.unit}` : ''}` : '';
        body += `<div class="field off" id="row_${f.id}"><div class="fh"><label for="fi_${f.id}">${esc(name)}</label><span class="val">${esc(range)}</span><span class="lock">${ic('lock', 'ic-s')}not in this preset</span></div>${f.kind === 'toggle' ? `<div class="ctl">${ctl}<span class="hint" style="margin:0">${esc(f.hint || '')}</span></div>` : `${ctl}<div class="hint">${esc(f.hint || '')}</div>${stops}`}<div class="err" hidden></div></div>`;
      }
      h += sec(`s_${s}`, esc(L(`section.${s}`) || cap1(s)), (catalog.sections || {})[s] || '', `<div class="offnote" hidden>${ic('lock', 'ic-s')}<span></span></div>${body}`, `<span class="badge s-idle secbadge" hidden>not in this preset</span>`);
    }
    h += sec('share', esc(L('section.share') || 'Share code'), 'These settings as one line of text: send it, or paste one to load its settings.',
      `<div class="codebox" id="sharecode" tabindex="0" aria-label="Share code">—</div>
       <div class="coderow"><button type="button" class="btn btn-sm" id="copycode">${ic('copy', 'ic-s')}Copy code</button></div>
       <div class="coderow"><input class="input" id="importcode" placeholder="${esc(L('share.hint') || 'Paste a DT1. code')}" spellcheck="false" aria-label="Share code to import"><button type="button" class="btn btn-sm" id="importbtn">${ic('import', 'ic-s')}${esc(L('share.import') || 'Import')}</button></div>
       <div class="importmsg" id="importmsg"></div>`);
    h += '<div style="display:flex;justify-content:flex-end;padding:2px 2px 8px"><button type="button" class="btn btn-ghost btn-sm" id="f_reset">Reset parameters to the preset</button></div>';
    $('form').innerHTML = h;
    wireForm();
  }

  const toSlider = (f, v, log) => log ? 1000 * Math.log(v / f.min) / Math.log(f.max / f.min) : 1000 * (v - f.min) / (f.max - f.min);
  const fromSlider = (f, s, log) => log ? f.min * Math.pow(f.max / f.min, s / 1000) : f.min + (f.max - f.min) * s / 1000;
  const tidy = v => Number(v.toPrecision(v >= 100 ? 4 : 3));
  const fillRange = el => { el.style.setProperty('--p', `${100 * (el.value - el.min) / (el.max - el.min || 1)}%`); };
  const LEN = { min: 1, max: 4600 };
  const lenToSlider = v => 1000 * Math.log(v / LEN.min) / Math.log(LEN.max / LEN.min);
  const lenFromSlider = s => LEN.min * Math.pow(LEN.max / LEN.min, s / 1000);

  function wireForm() {
    const fm = $('form');
    LEN.min = catalog.length_myr.min; LEN.max = catalog.length_myr.max;
    fm.addEventListener('click', ev => {
      const hd = ev.target.closest('.ed-sec > header');
      if (hd) { hd.parentElement.classList.toggle('closed'); return; }
      const pc = ev.target.closest('[data-fam]');
      if (pc) {
        const fam = pc.dataset.fam, sizes = catalog.presets.filter(p => p.family === fam);
        const cur = form.preset ? catalog.presets.find(p => p.name === form.preset) : null;
        const same = sizes.find(p => cur && p.circumference === cur.circumference) || sizes.find(p => p.circumference === 16384) || sizes[0];
        form.preset = same.name; form.imported = null;
        presetChanged();
        return;
      }
      const sz = ev.target.closest('[data-size]');
      if (sz && !sz.disabled) { form.preset = sz.dataset.size; presetChanged(); return; }
      const st = ev.target.closest('[data-start]');
      if (st) { form.start = st.dataset.start; startButtons(); refresh(); return; }
      const ql = ev.target.closest('[data-len]');
      if (ql) { form.length = ql.dataset.len; $('f_length').value = form.length; lengthUI(); refresh(); return; }
      const stop = ev.target.closest('[data-stopv]');   // a named stop of a field (catalogue `stops`)
      if (stop && !stop.disabled) {
        const inp = fm.querySelector(`[data-f="${stop.parentElement.dataset.stops}"]`);
        inp.value = stop.dataset.stopv;
        inp.dispatchEvent(new Event('input', { bubbles: true }));
        return;
      }
      const dice = ev.target.closest('[data-dice]');
      if (dice) {
        const v = Math.floor(Math.random() * 2 ** 32);
        fm.querySelector(`[data-f="${dice.dataset.dice}"]`).value = v;
        form.fields[dice.dataset.dice] = v; refresh();
      }
    });
    $('f_showph').addEventListener('change', () => { showPlaceholders = $('f_showph').checked; store.set('showPlaceholders', showPlaceholders); presetCards(); });
    $('f_length').addEventListener('input', () => { form.length = $('f_length').value; lengthUI(); refresh(); });
    $('f_lenr').addEventListener('input', () => {
      const v = lenFromSlider(Number($('f_lenr').value));
      form.length = String(v >= 100 ? Math.round(v / 10) * 10 : v >= 10 ? Math.round(v) : Number(v.toFixed(1)));
      $('f_length').value = form.length; lengthUI(); refresh();
    });
    // A new cap changes the answer (can this run?), which the server decides: ask it again.
    $('f_cap').addEventListener('input', () => { form.cap = Number($('f_cap').value); $('f_capr').value = capGiB(); fillRange($('f_capr')); capNote(); refresh(); });
    $('f_capr').addEventListener('input', () => { form.cap = Number($('f_capr').value); $('f_cap').value = form.cap; fillRange($('f_capr')); capNote(); refresh(); });
    $('f_capr').value = capGiB(); fillRange($('f_capr')); capNote();
    $('f_reset').addEventListener('click', () => { form.fields = {}; refresh(true); toast('Parameters reset to the preset', 'ok'); });
    for (const f of catalog.fields) {
      const input = fm.querySelector(`[data-f="${f.id}"]`), slider = fm.querySelector(`[data-r="${f.id}"]`);
      f._set = v => {   // show the effective value without firing
        if (document.activeElement === input) return;
        if (f.kind === 'toggle') input.checked = v === true;
        else if (f.kind === 'choice' && f.options && f.options.length) input.value = v === null ? '' : v;   // a fixed list: the options are in the markup
        else if (f.kind === 'choice') input.innerHTML = v === null ? `<option>${esc(L('field.none') || '—')}</option>` : `<option>${esc(Array.isArray(v) ? (v.length ? v.join(', ') : (L('field.none') || 'none')) : v)}</option>`;
        else input.value = v === null ? '' : v;
        if (slider && typeof v === 'number') { slider.value = toSlider(f, v, !!slider.dataset.log); fillRange(slider); }
      };
      input.addEventListener(f.kind === 'number' ? 'input' : 'change', () => {
        form.fields[f.id] = f.kind === 'toggle' ? input.checked : input.value;
        const n = Number(input.value);
        if (slider && isFinite(n)) { slider.value = toSlider(f, n, !!slider.dataset.log); fillRange(slider); }
        refresh();
      });
      if (slider) slider.addEventListener('input', () => {
        const v = f.integer ? Math.round(fromSlider(f, Number(slider.value), !!slider.dataset.log)) : tidy(fromSlider(f, Number(slider.value), !!slider.dataset.log));
        input.value = v; form.fields[f.id] = v; fillRange(slider); refresh();
      });
    }
    $('copycode').addEventListener('click', () => { if (eff && eff.share_code) copyText(eff.share_code, L('share.copied') || 'Share code copied'); });
    $('sharecode').addEventListener('click', () => { const s = getSelection(), rg = document.createRange(); rg.selectNodeContents($('sharecode')); s.removeAllRanges(); s.addRange(rg); });
    $('importbtn').addEventListener('click', () => importCode($('importcode').value));
    $('importcode').addEventListener('keydown', ev => { if (ev.key === 'Enter') importCode($('importcode').value); });
    $('importcode').addEventListener('paste', ev => {
      const t = (ev.clipboardData && ev.clipboardData.getData('text')) || '';
      if (/^\s*DT\d+\./.test(t)) setTimeout(() => importCode($('importcode').value), 0);
    });
  }

  function presetCards() {
    const fams = [];
    for (const p of catalog.presets) if (!fams.includes(p.family)) fams.push(p.family);
    const cur = form.preset ? catalog.presets.find(p => p.name === form.preset) : null;
    const real = fams.filter(f => !catalog.presets.filter(p => p.family === f).every(p => p.placeholder));
    const ph = fams.filter(f => !real.includes(f));
    const shown = [...real, ...(showPlaceholders || (cur && cur.placeholder) ? ph : [])];
    let h = shown.map(fam => {
      const ps = catalog.presets.filter(p => p.family === fam), p0 = ps[0], isPh = ps.every(p => p.placeholder);
      const on = !form.imported && cur && cur.family === fam;
      const desc = FAMILY[fam] ? FAMILY[fam][1] : 'Not a simulated planet: the dummy pipeline, until the real presets replace it.';
      return `<button type="button" class="presetcard${isPh ? ' ph' : ''}" role="radio" aria-checked="${!!on}" data-fam="${esc(fam)}" data-tip="${esc(p0.description)}">` +
        `<span class="pc-top"><span class="pc-name">${esc(familyName(fam))}</span>${isPh ? '<span class="badge s-queued">placeholder</span>' : `<span class="pc-raw">${esc(fam)}</span>`}</span>` +
        `<span class="pc-desc">${esc(desc)}</span>` +
        `<span class="pc-stages">${(p0.stages || []).map(s => `<i>${esc(s.replace(/_/g, ' '))}</i>`).join('')}</span></button>`;
    }).join('');
    if (form.imported) h = `<div class="presetcard imported" aria-checked="true"><span class="pc-top"><span class="pc-name">Imported: ${esc(form.imported.name || 'custom')}</span><span class="pc-raw">share code</span></span><span class="pc-desc">A preset from a share code. Its values are shown below; edit them and run.</span></div>` + h;
    $('f_presets').innerHTML = h;
  }
  function startButtons() {
    for (const b of $('f_start').children) b.setAttribute('aria-checked', String(b.dataset.start === form.start));
    $('f_start').classList.toggle('need', !form.start);
  }
  function lengthUI() {
    const v = Number(form.length);
    const r = $('f_lenr');
    r.value = form.length !== '' && isFinite(v) && v > 0 ? lenToSlider(clamp(v, LEN.min, LEN.max)) : 0;
    fillRange(r);
    for (const b of $('f_quick').children) b.setAttribute('aria-pressed', String(Number(b.dataset.len) === v));
  }

  function presetChanged() {
    const cur = form.preset ? catalog.presets.find(p => p.name === form.preset) : null;
    presetCards();
    const sizes = $('f_sizes');
    if (form.imported) {
      const c = form.imported.scale && form.imported.scale.circumference_blocks;
      sizes.innerHTML = `<button type="button" disabled aria-pressed="true">${esc(c ? sizeLabel(c) : '?')}</button>`;
      $('f_sizeval').textContent = 'from the code';
    } else if (cur) {
      sizes.innerHTML = catalog.presets.filter(p => p.family === cur.family)
        .map(p => `<button type="button" data-size="${esc(p.name)}" aria-pressed="${p.name === cur.name}">${esc(sizeLabel(p.circumference))}</button>`).join('');
      $('f_sizeval').textContent = cur.name;
    }
    $('f_presetwarn').innerHTML = cur && cur.placeholder && !form.imported
      ? `<div class="callout warn">${ic('alert')}<div><b>Placeholder: not the real simulation.</b> ${esc(L('warning.placeholder') || '')} Pick an Earth-like preset to simulate a planet.</div></div>` : '';
    refresh(true);
  }

  // The cap in use, beside the field: the default, or the raised/lowered value for this run.
  function capNote() {
    const el = $('f_capval');
    if (!el) return;
    const g = capGiB(), d = defaultCapGiB();
    el.textContent = Math.abs(g - d) < 1e-9 ? `${fmtCap(g)} · the default` : `${fmtCap(g)} for this run · default ${fmtCap(d)}`;
  }

  function refresh(now) {
    clearTimeout(effTimer);
    estimating = true; renderEstimate();
    effTimer = setTimeout(computeEffective, now ? 0 : 250);
  }

  async function computeEffective() {
    const seq = ++effSeq;
    const req = { fields: form.fields };
    if (capToSend() !== 0) req.cap_gib = capToSend();
    if (form.imported) req.imported = form.imported; else req.preset = form.preset;
    if (form.start) req.start = form.start;
    if (form.length !== '' && isFinite(Number(form.length))) req.length_myr = Number(form.length);
    const res = await api('api/preset', req);
    if (seq !== effSeq) return;   // a newer answer is on its way
    estimating = false;
    eff = res.body;
    if (eff.error) { $('problems').innerHTML = `<li class="err">${ic('alert', 'ic-s')}<span>${esc(cap1(eff.error))}</span></li>`; runButton(); renderEstimate(); return; }
    for (const sec of document.querySelectorAll('.ed-sec[data-sec^="s_"]')) sec.dataset.avail = '0';
    for (const fs of eff.fields) {
      const f = catalog.fields.find(x => x.id === fs.id), row = $(`row_${fs.id}`);
      if (!f || !row) continue;
      row.classList.toggle('off', !fs.available);
      for (const el of row.querySelectorAll('input, select, button')) el.disabled = !fs.available;
      const lock = row.querySelector('.lock');
      lock.innerHTML = `${ic('lock', 'ic-s')}${esc(f.phase3 ? 'Phase 3' : 'not in this preset')}`;
      if (fs.available) row.closest('.ed-sec').dataset.avail = '1';
      const bad = eff.field_problems[fs.id];
      row.classList.toggle('invalid', !!bad);
      const err = row.querySelector('.err');
      err.hidden = !bad; err.innerHTML = bad ? `${ic('alert', 'ic-s')}${esc(cap1(bad))}` : '';
      if (!bad) f._set(fs.value);
    }
    // A section none of whose fields this preset carries folds away, saying why.
    for (const sec of document.querySelectorAll('.ed-sec[data-sec^="s_"]')) {
      const off = sec.dataset.avail !== '1';
      const had = sec.dataset.off;
      sec.querySelector('.secbadge').hidden = !off;
      const note = sec.querySelector('.offnote');
      note.hidden = !off;
      note.querySelector('span').textContent = sec.dataset.sec === 's_fantasy' ? (L('field.phase3') || 'Arrives in Phase 3.') : 'This preset has none of these yet: they arrive with the simulation stages.';
      // Advanced sections (the catalogue's advanced_sections) start closed too; opening one sticks.
      if (had !== String(off)) { sec.dataset.off = String(off); sec.classList.toggle('closed', off || (catalog.advanced_sections || []).includes(sec.dataset.sec.slice(2))); }
    }
    const probs = eff.problems.map(p => `<li${/memory cap/.test(p) ? ' class="err"' : ''} data-prob="${esc(p)}">${ic('alert', 'ic-s')}<span>${esc(cap1(p))}</span></li>`);
    $('problems').innerHTML = probs.join('');
    $('sharecode').textContent = eff.share_code || '—';
    renderEstimate();
    runButton();
  }
  $('problems').addEventListener('click', ev => {
    const li = ev.target.closest('li[data-prob]');
    if (!li) return;
    const p = li.dataset.prob;
    const target = /continents start/.test(p) ? $('f_start') : /length|million years/.test(p) ? $('f_length') : /red fields|not valid/.test(p) ? document.querySelector('.field.invalid .input') : /memory cap|GiB cap/.test(p) ? $('f_cap') : null;
    if (target) {
      const sec = target.closest('.ed-sec');
      if (sec) sec.classList.remove('closed');   // a folded section would hide the field it is about
      target.scrollIntoView({ block: 'center', behavior: reducedMotion.matches ? 'auto' : 'smooth' });
      (target.querySelector('button') || target).focus();
    }
  });

  function estimateSeconds() {
    const e = eff && eff.estimate;
    return e && e.stages ? e.stages.reduce((s, x) => s + (x.seconds || 0), 0) : null;
  }
  // The server decides (eff.can_run holds the reason among eff.problems); this keeps Run off in the
  // moment between a new cap and the server's answer, and draws the bar.
  // What the run needs of the cap: the estimate's peak plus the server's own memory (the server adds it).
  const neededBytes = () => { const e = eff && !eff.error ? eff.estimate : null; return e && e.peak_bytes ? (eff.needed_bytes || e.peak_bytes) : 0; };
  function overCap() { return neededBytes() > capGiB() * 2 ** 30; }
  function renderEstimate() {
    const box = $('estimate');
    const e = eff && !eff.error ? eff.estimate : null;
    const cap = capGiB() * 2 ** 30;
    const capText = Math.abs(capGiB() - defaultCapGiB()) < 1e-9 ? `cap ${fmtCap(capGiB())} · default` : `cap ${fmtCap(capGiB())} · default ${fmtCap(defaultCapGiB())}`;
    if (!e || !e.peak_bytes) {
      const why = estimating ? 'Estimating…' : 'After start and length';
      box.innerHTML = `<div class="est pending"><div class="k">${ic('chip', 'ic-s')}Peak memory</div><div class="v">—</div><div class="sub">${esc(why)} · ${esc(capText)}</div></div>` +
        `<div class="est pending"><div class="k">${ic('clock', 'ic-s')}Time on the Mac</div><div class="v">—</div><div class="sub">${esc(why)}</div></div>`;
      warnings(null);
      return;
    }
    const need = neededBytes();
    const secs = estimateSeconds(), scale = Math.max(cap * 1.15, need * 1.05), over = need > cap;
    // Over the cap by its recorded history only: that goes to the SSD (P1-17), the run just slows.
    const spill = !over && e.spill_bytes > 0 ? ` · ${fmtBytes(e.spill_bytes)} of history to disk` : '';
    const [mv, mu] = fmtBytes(need).split(' ');
    const tt = fmtHuman(secs).split(' ');
    box.innerHTML = `<div class="est${estimating ? ' pending' : ''}" data-tip="${esc(`The estimate, ${fmtBytes(e.peak_bytes)}, plus ${fmtBytes(need - e.peak_bytes)} for the server itself: what the run holds against its cap`)}"><div class="k">${ic('chip', 'ic-s')}Peak memory</div><div class="v">${esc(mv)}<small>${esc(mu)}</small></div>` +
      `<div class="membar"><span class="${over ? 'over' : ''}" style="width:${(100 * need / scale).toFixed(1)}%;${!over && need > 0.8 * cap ? 'background:var(--queued)' : ''}"></span><i style="left:${(100 * cap / scale).toFixed(1)}%"></i></div>` +
      `<div class="sub">${esc(capText + spill)}</div></div>` +
      `<div class="est${estimating ? ' pending' : ''}"><div class="k">${ic('clock', 'ic-s')}Time on the Mac</div><div class="v">${esc(tt[0])}<small>${esc(tt.slice(1).join(' '))}</small></div>` +
      `<div class="sub" data-tip="${esc(e.stages.map(s => `${s.name} ${fmtHuman(s.seconds)}`).join(' · '))}">${esc(e.stages.map(s => s.name).join(' · '))}</div></div>`;
    warnings(e, secs);
    runButton();
  }
  function warnings(e, secs) {
    for (const li of $('problems').querySelectorAll('li.w')) li.remove();
    if (!e) return;
    // Over the cap is a problem the server reports (eff.problems, with Run off); here only "near it".
    const cap = capGiB() * 2 ** 30, out = [];
    const need = neededBytes();
    if (need <= cap && e.spill_bytes > 0) out.push(['', `Over the cap by its recorded history: about ${fmtBytes(e.spill_bytes)} of it goes to the SSD in the run's folder while it runs. Slower, same planet.`]);
    else if (need <= cap && need > 0.8 * cap) out.push(['', `Near the memory cap: about ${fmtBytes(need)} of ${fmtCap(capGiB())}.`]);
    if (secs > 3600) out.push(['', `A long run: about ${fmtHuman(secs)} of the Mac's time, and nothing else runs meanwhile.`]);
    else if (secs > 900) out.push(['', `About ${fmtHuman(secs)} of the Mac's time.`]);
    for (const [cls, text] of out) {
      const li = document.createElement('li');
      li.className = `w ${cls}`; li.dataset.prob = text;
      li.innerHTML = `${ic('alert', 'ic-s')}<span>${esc(text)}</span>`;
      $('problems').appendChild(li);
    }
  }
  function runButton() {
    const b = $('runbtn');
    if (!b) return;
    const ok = !!(eff && !eff.error && eff.can_run && !overCap() && !estimating);
    b.disabled = !ok;
    const running = lastRuns.find(r => r.state === 'running');
    const waiting = lastRuns.filter(r => r.state === 'queued').length;
    b.querySelector('span').textContent = running ? 'Queue run' : 'Run';
    b.dataset.tip = !ok ? 'Fix the items above first' : running ? `Starts after ${running.label}${waiting ? ` and ${waiting} more` : ''}` : 'Start on the Mac now';
  }

  async function importCode(code) {
    const msg = $('importmsg');
    const res = await api('api/decode', { code: (code || '').trim() });
    if (!res.ok) { msg.className = 'importmsg bad'; msg.textContent = L('share.invalid', res.body.error) || res.body.error; toast('Could not import the code', 'bad'); return; }
    const d = res.body;
    form.preset = d.preset; form.imported = d.preset ? null : d.imported; form.fields = {};
    form.start = d.start || ''; form.length = d.length_myr ?? '';
    $('f_length').value = form.length;
    startButtons(); lengthUI();
    msg.className = 'importmsg ok';
    msg.textContent = d.preset ? `This is the built-in ${presetName(d.preset)} (${d.preset}).` : `Imported ${d.imported.name || 'a custom preset'}: its values are shown; edit and run.`;
    presetChanged();
  }

  $('runbtn').addEventListener('click', async () => {
    if (!eff || !eff.can_run || overCap()) return;
    $('runbtn').disabled = true;
    const res = await api('api/runs', { json: eff.json, label: $('label').value, cap_gib: capToSend() });
    if (!res.ok) { toast(res.body.error || 'The run was refused', 'bad', 6000); $('problems').insertAdjacentHTML('afterbegin', `<li class="err" data-prob="${esc(res.body.error)}">${ic('alert', 'ic-s')}<span>${esc(cap1(res.body.error))}</span></li>`); runButton(); return; }
    $('label').value = '';
    const queued = lastRuns.some(r => r.state === 'running');
    toast(queued ? 'Queued: it starts when the Mac is free' : 'Started on the Mac', 'ok');
    open(0, res.body.id); setFocus(0);
    sideTab('runs');
    if (narrow.matches) setMTab('layers');
    await poll();
    runButton();
  });

  // =================================================================================================
  // Start
  // =================================================================================================
  fetch('ramps.json').then(r => r.json()).then(j => { ramps = j; rampCache.clear(); legendDirty = true; shownLayers = ''; layersDirty = true; }).catch(() => {});
  if (!TV) {
    if (store.get('noside', false) && !medium.matches) $('app').classList.add('noside');
    sideTab(store.get('stab', 'runs'));
    setITab(store.get('itab', 'events') === 's2' ? 'events' : store.get('itab', 'events'));
    setMTab('layers');
  }
  buildAxes();
  setCompare(false);
  spinButton();
  api('api/catalog').then(res => {
    if (!res.ok) { $('form').innerHTML = `<div class="empty">${ic('alert', 'art')}<h4>The editor did not load</h4><p>${esc(res.body.error || '')}</p></div>`; return; }
    catalog = res.body;
    form.cap = defaultCapGiB();
    buildForm();
    const def = catalog.presets.find(p => p.name === 'earthlike_quick_16k') || catalog.presets.find(p => !p.placeholder) || catalog.presets[0];
    form.preset = def.name;
    startButtons(); lengthUI();
    presetChanged();
    for (const r of lastRuns) runCard(r);
    runhead(); runInfo();
    for (const p of panes) paneHead(p);
  });
  buttons();
  stageEmpty();
  poll();
  layout();
  globeSize();
  requestAnimationFrame(tick);
})();
