// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time workbench: "Rock at depth" and the cross-section tool (decisions log 2026-10-01, "Which
// workbench features?"; crates/deeptime-cli/src/watch/sections/ answers from the run's bake).
//
// - Rock at depth: on a rock layer (any layer coloured with the `rock_class` ramp) a depth control
//   appears under the legend. Moving it below "Surface" swaps the map's picture for the bake's slice,
//   `GET api/runs/<id>/rock/slice.png?depth=<blocks|floor>`: the rock class that many blocks below the
//   surface, over the whole planet, in the rock_class colours. It is the finished world (the bake),
//   so the time slider does not change it, and it exists only for a finished run with a bake.
// - Section tool (S): click two points on the map; `GET api/runs/<id>/rock/section?x0&z0&x1&z1` answers
//   the strata along the line (the shorter way round the planet). The section opens in a dock over the
//   map's lower edge (and large in a dialog): rock classes in the rock_class colours, the surface,
//   water and sea level, the bodies (deposits, seams, plutons) the line crosses, an elevation axis in
//   blocks and the line's length in blocks and km. Click a class in the key to light it up (diamictite,
//   coal). The Section tab of the inspector keeps the line's numbers and the recent sections.
//
// workbench.js exposes what this file needs as `window.DTWB` and calls `window.DTSections` at five
// places: `slice` and `draw` while a pane draws, `click` for the tool, `tool` when it changes.
'use strict';

(function () {
  const W = window.DTWB;
  if (!W) return;
  const { $, esc, ic, clamp, fmtInt, api, toast, modal, niceTicks } = W;
  const enc = encodeURIComponent;

  // ===============================================================================================
  // Page furniture: an icon, styles
  // ===============================================================================================
  const anchor = document.querySelector('symbol#i-box');
  if (anchor && anchor.parentNode) {
    const sym = document.createElementNS('http://www.w3.org/2000/svg', 'symbol');
    sym.id = 'i-section'; sym.setAttribute('viewBox', '0 0 24 24');
    sym.innerHTML = '<path d="M3 6h18" stroke-dasharray="2.6 2.4"/><circle cx="3.5" cy="6" r="1.3"/><circle cx="20.5" cy="6" r="1.3"/>' +
      '<path d="M3 11.5c3-2.2 6 2.2 9 0s6-2.2 9 0M3 16.5c3-2.2 6 2.2 9 0s6-2.2 9 0M3 21c3-2.2 6 2.2 9 0s6-2.2 9 0"/>';
    anchor.parentNode.appendChild(sym);
  }
  const style = document.createElement('style');
  style.textContent = `
.tool-section .mapframe canvas.map { cursor: crosshair; }
.depthctl { margin: 0 14px 14px; padding: 11px 12px 12px; border-radius: var(--r); background: var(--bg-3); border: 1px solid var(--line-2); }
.depthctl .dc-h { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; margin-bottom: 8px; white-space: nowrap; }
.depthctl .dc-t { display: flex; align-items: center; gap: 6px; font-size: 10.5px; font-weight: 600; letter-spacing: .08em; text-transform: uppercase; color: var(--fg-3); }
.depthctl .dc-v { font-size: 13px; font-weight: 600; color: var(--fg-1); font-variant-numeric: tabular-nums; text-align: right; }
.depthctl .dc-v small { font-size: 11px; font-weight: 500; color: var(--fg-3); margin-left: 4px; }
.depthctl .dc-chips { display: flex; flex-wrap: wrap; gap: 4px; margin-bottom: 9px; }
.depthctl .dc-chips button { height: 22px; padding: 0 8px; border-radius: 6px; border: 1px solid var(--line-2); background: var(--bg-2); color: var(--fg-2); font-size: 11.5px; font-variant-numeric: tabular-nums; }
.depthctl .dc-chips button:hover { border-color: var(--line-3); color: var(--fg-1); }
.depthctl .dc-chips button[aria-pressed="true"] { background: var(--accent-soft); border-color: var(--accent-line); color: var(--accent-hi); }
.depthctl input[type=range] { width: 100%; margin: 0; accent-color: var(--accent); }
.depthctl .dc-note { margin-top: 8px; font-size: 11.5px; line-height: 1.45; color: var(--fg-3); }
.depthctl .dc-note.warn { color: #e9c27a; }
.depthctl .dc-note.bad { color: var(--failed); }
.depthctl.off input, .depthctl.off .dc-chips { opacity: .45; pointer-events: none; }
.slicebadge { right: 12px; top: 46px; display: flex; align-items: center; gap: 6px; padding: 4px 9px; border-radius: 8px; background: rgba(10, 13, 19, .78);
  backdrop-filter: blur(6px); border: 1px solid var(--line-2); font-size: 11.5px; color: var(--fg-1); font-weight: 600; max-width: 70%; }
.slicebadge span { color: var(--fg-3); font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

.ab { display: inline-grid; place-items: center; width: 18px; height: 18px; border-radius: 5px; background: var(--accent); color: var(--accent-ink); font-size: 10.5px; font-weight: 700; flex: 0 0 auto; }
.ab.b { background: #f3f5fa; color: #0c1016; }
.sec { padding: 0 14px 14px; overflow-y: auto; flex: 1 1 auto; min-height: 0; display: flex; flex-direction: column; gap: 10px; }
.sec p { margin: 0; color: var(--fg-2); font-size: 12px; line-height: 1.5; }
.sec .sec-t { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 6px; font-size: 12px; color: var(--fg-2); font-variant-numeric: tabular-nums; }
.sec .sec-s { font-size: 11.5px; color: var(--fg-3); margin-top: 3px; font-variant-numeric: tabular-nums; }
.sec .sec-s b { color: var(--fg-1); font-weight: 600; }
.sec .sec-acts { display: flex; flex-wrap: wrap; gap: 6px; }
.sec .recent { display: flex; flex-direction: column; gap: 4px; }
.sec .recent h5, .sv-leg h6 { margin: 6px 0 2px; font-size: 10.5px; font-weight: 600; letter-spacing: .09em; text-transform: uppercase; color: var(--fg-4); }
.sec .recent button { display: flex; align-items: center; gap: 8px; padding: 6px 9px; border-radius: 8px; border: 1px solid var(--line-1); background: var(--bg-3); color: var(--fg-2); font-size: 12px; text-align: left; font-variant-numeric: tabular-nums; }
.sec .recent button:hover { border-color: var(--line-3); color: var(--fg-1); }

.secdock { position: absolute; left: 10px; right: 10px; bottom: 10px; z-index: 5; height: min(50%, 420px); min-height: 270px; display: flex; flex-direction: column;
  border-radius: 12px; background: rgba(10, 13, 19, .965); border: 1px solid var(--line-3); box-shadow: var(--shadow-2); backdrop-filter: blur(8px); animation: fade-in .2s var(--ease); }
.secdock.min { height: auto; min-height: 0; }
.secdock.min .sv-grid { display: none; }
.sd-h { display: flex; align-items: center; gap: 10px; padding: 8px 8px 8px 12px; flex: 0 0 auto; flex-wrap: wrap; }
.sd-t { display: flex; flex-direction: column; gap: 2px; min-width: 0; flex: 1 1 300px; font-size: 12px; color: var(--fg-2); font-variant-numeric: tabular-nums; }
.sd-t > div:first-child { display: flex; align-items: center; flex-wrap: wrap; gap: 3px 6px; }
.sd-t .sd-s { color: var(--fg-3); }
.sd-t .sd-s b { color: var(--fg-1); font-weight: 600; }
.sd-acts { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.sd-acts .btn[aria-pressed="false"] { color: var(--fg-3); }
.sv-grid { display: grid; grid-template-columns: minmax(0, 1fr) 250px; gap: 12px; padding: 0 12px 12px; flex: 1 1 auto; min-height: 0; }
.sv-main { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
.sv-cw { position: relative; flex: 1 1 auto; min-height: 150px; border-radius: 9px; overflow: hidden; background: #070a0f; box-shadow: 0 0 0 1px var(--line-2); }
.sv-cw canvas { position: absolute; inset: 0; width: 100%; height: 100%; display: block; cursor: crosshair; }
.sv-side { display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow-y: auto; padding-right: 2px; }
.sv-read { flex: 0 0 auto; min-height: 58px; padding: 7px 9px; border-radius: 8px; background: var(--bg-2); border: 1px solid var(--line-1); font-size: 11.5px; line-height: 1.5; color: var(--fg-2); font-variant-numeric: tabular-nums; }
.sv-read b { color: var(--fg-1); font-weight: 600; }
.sv-read .sw { display: inline-block; width: 10px; height: 10px; border-radius: 3px; margin-right: 5px; vertical-align: -1px; box-shadow: 0 0 0 1px rgba(255, 255, 255, .25); }
.sv-note { font-size: 11px; color: var(--fg-4); line-height: 1.45; flex: 0 0 auto; }
.sv-leg { display: flex; flex-wrap: wrap; gap: 4px; align-content: flex-start; flex: 1 1 auto; }
.sv-leg h6 { flex: 1 1 100%; }
.sv-leg h6:first-child { margin-top: 0; }
.chip { display: inline-flex; align-items: center; gap: 6px; height: 24px; padding: 0 8px 0 6px; border-radius: 6px; border: 1px solid var(--line-1); background: var(--bg-3); color: var(--fg-2); font-size: 11.5px; white-space: nowrap; }
button.chip:hover { border-color: var(--line-3); color: var(--fg-1); }
button.chip[aria-pressed="true"] { border-color: var(--accent-line); background: var(--accent-soft); color: var(--fg-1); }
.chip i { width: 11px; height: 11px; border-radius: 3px; flex: 0 0 auto; box-shadow: 0 0 0 1px rgba(255, 255, 255, .18); }
.chip i.ring { background: transparent !important; box-shadow: none; border: 2px solid var(--c); }
.chip small { color: var(--fg-4); font-size: 10.5px; }
.sec-big { position: absolute; inset: 0; display: flex; flex-direction: column; padding-top: 12px; }
.sec-big .sv-grid { grid-template-columns: minmax(0, 1fr) 280px; padding: 0 14px 14px; }
@media (max-width: 900px) { .sv-grid { grid-template-columns: 1fr; } .sv-side { display: none; } }
`;
  document.head.appendChild(style);

  // ===============================================================================================
  // Rock at depth
  // ===============================================================================================
  const DEPTHS = [0, 10, 20, 35, 50, 75, 100, 150, 200, 300, 400, 600, 800, 'floor'];
  const CHIPS = [0, 20, 50, 100, 200, 'floor'];
  let depthIdx = 0;
  const depthKey = () => String(DEPTHS[depthIdx]);
  const runs = new Map();   // run id -> { state: idle | loading | ok | none, info, why, images: Map(key -> entry) }
  const X = r => {
    let x = runs.get(r.id);
    if (!x) runs.set(r.id, x = { state: 'idle', info: null, why: '', images: new Map() });
    return x;
  };
  const isRockRamp = name => name === 'rock_class';

  async function ensureInfo(r) {
    const x = X(r);
    if (x.state !== 'idle' || !r.finished()) return;
    x.state = 'loading';
    const res = await api(`api/runs/${enc(r.id)}/rock`);
    if (res.ok && res.body.available) {
      x.state = 'ok'; x.info = res.body;
      nameClasses(res.body);
    } else {
      x.state = 'none';
      x.why = (res.body && (res.body.why || res.body.error)) || 'The workbench did not answer.';
    }
    refresh(true);
  }

  // The rock_class ramp has no labels of its own: name its classes from the bake (the same names
  // everywhere the page reads a colour back: hover, legend).
  function nameClasses(info) {
    const ri = W.rampInfo('rock_class');
    if (!ri || !ri.classes) return;
    for (const c of info.classes) if (c.k !== null && c.k !== undefined && ri.classes[c.k]) ri.classes[c.k].label = c.name.replace(/_/g, ' ');
    ri.labelled = true;
    W.legend();
  }

  function loadSlice(r, x, key) {
    const e = { state: 'loading', img: null, t0: performance.now(), ms: 0, error: '', url: null };
    x.images.set(key, e);
    fetch(`api/runs/${enc(r.id)}/rock/slice.png?depth=${enc(key)}`).then(async res => {
      if (!res.ok) {
        let m = `${res.status} ${res.statusText}`;
        try { const j = await res.json(); m = j.why || j.error || m; } catch (err) { /* not JSON */ }
        throw new Error(m);
      }
      const img = new Image();
      e.url = URL.createObjectURL(await res.blob());
      img.src = e.url;
      await img.decode();
      e.img = img; e.ms = performance.now() - e.t0; e.state = 'ready';
    }).catch(err => {
      e.state = 'error'; e.error = String((err && err.message) || err);
    }).finally(() => { refresh(true); W.redraw(); });
    // Keep the most recent depths only.
    while (x.images.size > 14) {
      const [k0, e0] = x.images.entries().next().value;
      if (e0.url) URL.revokeObjectURL(e0.url);
      x.images.delete(k0);
    }
    return e;
  }

  // workbench.js drawPane: the picture of the bake's slice for run `r` on a rock layer below the
  // surface, 'wait' while it loads, null when the frame's own picture is what to show.
  function slice(r, ramp) {
    if (depthIdx === 0 || !isRockRamp(ramp) || !r.finished()) return null;
    const x = X(r);
    if (x.state === 'idle') { ensureInfo(r); return 'wait'; }
    if (x.state === 'loading') return 'wait';
    if (x.state === 'none') return null;
    const key = depthKey();
    const e = x.images.get(key) || loadSlice(r, x, key);
    if (e.state === 'ready') { x.images.delete(key); x.images.set(key, e); return e.img; }
    return e.state === 'error' ? null : 'wait';
  }
  const sliceShown = (r, ramp) => !!r && depthIdx > 0 && isRockRamp(ramp) && r.finished() && X(r).state === 'ok' && (X(r).images.get(depthKey()) || {}).state === 'ready';

  const depthText = (d, info) => {
    if (d === 'floor') return `World floor${info ? ` <small>y ${info.min_y}</small>` : ''}`;
    if (d === 0) return 'Surface';
    return `${d} blocks${info ? ` <small>≈ ${(d * info.m_per_block_v / 1000).toFixed(1)} km</small>` : ''}`;
  };
  const depthPlain = (d, info) => d === 'floor' ? 'the world floor' : d === 0 ? 'the surface' :
    `${d} blocks (≈ ${(d * ((info && info.m_per_block_v) || 20) / 1000).toFixed(1)} km) below the surface`;

  const ctl = document.createElement('div');
  ctl.className = 'depthctl'; ctl.id = 'depthctl'; ctl.hidden = true;
  ctl.innerHTML = `<div class="dc-h"><span class="dc-t">${ic('layers', 'ic-s')}Rock at depth</span><span class="dc-v" id="dc-v">Surface</span></div>` +
    `<div class="dc-chips" id="dc-chips">${CHIPS.map(c => `<button type="button" data-d="${c}" aria-pressed="false">${c === 0 ? 'Surface' : c === 'floor' ? 'Floor' : `−${c}`}</button>`).join('')}</div>` +
    `<input id="dc-range" type="range" min="0" max="${DEPTHS.length - 1}" step="1" value="0" aria-label="Depth below the surface, in blocks">` +
    '<div class="dc-note" id="dc-note"></div>';
  const legendEl = $('legend');
  if (legendEl) legendEl.after(ctl);
  let commitTimer = null;
  function setDepth(i, now) {
    depthIdx = clamp(i, 0, DEPTHS.length - 1);
    $('dc-range').value = depthIdx;
    refresh(true);
    clearTimeout(commitTimer);
    if (now) W.redraw(); else commitTimer = setTimeout(() => W.redraw(), 220);
  }
  $('dc-range').addEventListener('input', ev => setDepth(Number(ev.target.value), false));
  $('dc-range').addEventListener('change', ev => setDepth(Number(ev.target.value), true));
  $('dc-chips').addEventListener('click', ev => {
    const b = ev.target.closest('button[data-d]');
    if (b) setDepth(DEPTHS.indexOf(b.dataset.d === 'floor' ? 'floor' : Number(b.dataset.d)), true);
  });

  let ctlKey = '';
  function refresh(force) {
    const p = W.focused(), r = p.run;
    const ramp = r ? r.rampOf.get(W.layer()) : null;
    const show = !!r && isRockRamp(ramp);
    ctl.hidden = !show;
    if (!show) return;
    if (r.finished()) ensureInfo(r);
    const x = X(r), info = x.info;
    const d = DEPTHS[depthIdx], e = x.images.get(depthKey());
    let cls = '', text;
    const other = W.pane(1);
    const otherNone = other.run && !other.el.hidden && other.run.id !== r.id && (!other.run.finished() || X(other.run).state === 'none');
    if (!r.finished()) {
      cls = 'warn'; text = 'Rock at depth reads the finished world, so it needs a run that has ended: its bake is written at the end. Until then the map shows the run’s own frames.';
    } else if (x.state === 'idle' || x.state === 'loading') {
      text = 'Checking the run’s bake…';
    } else if (x.state === 'none') {
      cls = 'warn'; text = `Rock at depth needs a run with a bake. ${x.why}`;
    } else if (d === 0) {
      text = 'The surface is the run’s own frame, so it follows the time slider. Move down to read the finished world from its bake.';
    } else if (!e || e.state === 'loading') {
      text = `Reading ${depthPlain(d, info)} from the bake${x.images.size <= 1 ? ' (the first slice of a run opens its strata)' : ''}…`;
    } else if (e.state === 'error') {
      cls = 'bad'; text = e.error;
    } else {
      text = `Rock ${depthPlain(d, info)}, from the finished world (the bake), drawn in ${e.ms < 1000 ? `${Math.round(e.ms)} ms` : `${(e.ms / 1000).toFixed(1)} s`}. The time slider does not change it.${info && info.placeholder ? ' This run is a placeholder: its strata are synthetic.' : ''}`;
    }
    if (otherNone && d !== 0) text += ' Run B has no bake to read: its map keeps its frame.';
    const key = [r.id, depthIdx, x.state, e && e.state, e && Math.round(e.ms), text, cls].join('|');
    if (!force && key === ctlKey) return;
    ctlKey = key;
    ctl.classList.toggle('off', x.state === 'none' || !r.finished());
    $('dc-v').innerHTML = depthText(d, info);
    for (const b of $('dc-chips').children) b.setAttribute('aria-pressed', String((b.dataset.d === 'floor' ? 'floor' : Number(b.dataset.d)) === d));
    const n = $('dc-note');
    n.className = `dc-note ${cls}`; n.textContent = text;
  }

  // A badge on each pane that shows a slice: what it is, since its clock says another time.
  const badges = W.panes.map(p => {
    const el = document.createElement('div');
    el.className = 'overlay slicebadge'; el.hidden = true;
    p.frame.appendChild(el);
    return el;
  });
  function badge(p) {
    const el = badges[p.i], r = p.run;
    const on = sliceShown(r, r && r.rampOf.get(W.layer()));
    el.hidden = !on;
    if (on) {
      const d = DEPTHS[depthIdx], html = `${ic('layers', 'ic-s')}Rock ${d === 0 ? 'at the surface' : d === 'floor' ? 'at the world floor' : `${d} blocks down`} <span>· the finished world (bake)</span>`;
      if (el.dataset.h !== html) { el.dataset.h = html; el.innerHTML = html; }
    }
  }

  // ===============================================================================================
  // Sections: the line on the map
  // ===============================================================================================
  const S = {
    state: 'idle',        // idle | pickB | loading | done | error
    run: null, pane: 0, a: null, b: null, cursor: null,
    data: null, error: '', ms: 0, req: 0,
    range: 'near', bodies: true, hl: null,
    dock: 'open',         // open | min | closed
    hoverCol: null,
    recent: [],
  };
  const views = new Set();
  const shortDLon = d => ((((d + 180) % 360) + 360) % 360) - 180;   // into [-180, 180)

  function hintText() {
    if (S.state === 'pickB') return `${ic('section')}Click the second point <span class="dim">· A is set</span><button class="btn btn-ghost btn-sm" data-tool="select">Done <kbd>Esc</kbd></button>`;
    return `${ic('section')}Click two points to cut a section <span class="dim">· from the run’s bake</span><button class="btn btn-ghost btn-sm" data-tool="select">Done <kbd>Esc</kbd></button>`;
  }
  function tool(t, hint) {
    $('viewport').classList.toggle('tool-section', t === 'section');
    if (t === 'section') {
      hint.innerHTML = hintText();
      // Picking again: the section folds to its bar, so it does not cover the map.
      if (S.state === 'done' && S.dock === 'open') { S.dock = 'min'; dockKey = ''; }
      renderAll();
    } else if (S.state === 'pickB') {
      S.state = S.data ? 'done' : 'idle'; S.a = S.data ? S.a : null; S.cursor = null;
      renderAll(); W.redraw();
    }
  }
  // The pill is out of the dock's way while the dock is open (it would sit on its header).
  let dockOpen = false;
  const setHint = () => {
    if (W.tool() !== 'section') return;
    $('maphint').innerHTML = hintText();
    $('maphint').hidden = dockOpen;
  };

  function click(p, r, lat, lon) {
    const x = X(r);
    if (!r.finished()) { toast('A section reads the finished world: wait for the run to end.', 'warn', 5000); return; }
    if (x.state === 'none') { toast(`This run has no bake to cut: ${x.why}`, 'warn', 6000); return; }
    if (S.state !== 'pickB' || S.run !== r || S.pane !== p.i) {
      S.run = r; S.pane = p.i; S.a = { lat, lon }; S.b = null; S.cursor = null; S.data = null; S.state = 'pickB'; S.hl = null;
      S.req++;
      setHint(); renderAll(); W.redraw();
      W.setITab('section', true);
      if (W.narrow()) W.setMTab('inspect');
      return;
    }
    S.b = { lat, lon }; S.cursor = null;
    go();
  }

  async function go() {
    const r = S.run, token = ++S.req;
    const [x0, z0] = W.blockXZ(r, S.a.lat, S.a.lon), [x1, z1] = W.blockXZ(r, S.b.lat, S.b.lon);
    S.state = 'loading'; S.data = null; S.error = ''; S.dock = 'open';
    setHint(); renderAll(); W.redraw();
    const t0 = performance.now();
    const res = await api(`api/runs/${enc(r.id)}/rock/section?x0=${x0}&z0=${z0}&x1=${x1}&z1=${z1}`);
    if (token !== S.req) return;
    S.ms = performance.now() - t0;
    if (res.ok && res.body.available) {
      S.data = prep(res.body); S.state = 'done';
      const same = o => o.run === r.id && o.a.lat === S.a.lat && o.a.lon === S.a.lon && o.b.lat === S.b.lat && o.b.lon === S.b.lon;
      S.recent = [{ run: r.id, a: S.a, b: S.b, km: S.data.length_km }, ...S.recent.filter(o => !same(o))].slice(0, 6);
    } else {
      S.state = 'error';
      S.error = (res.body && (res.body.why || res.body.error)) || 'The workbench did not answer.';
    }
    setHint(); renderAll(); W.redraw();
  }

  function clear() {
    S.req++; S.state = 'idle'; S.a = S.b = S.cursor = null; S.data = null; S.hoverCol = null; S.hl = null;
    setHint(); renderAll(); W.redraw();
  }

  // The line, its ends and the point under the pointer in a section view, on the map.
  function draw(p, ctx, dpr) {
    badge(p);
    if (!S.a || S.pane !== p.i || p.run !== S.run) return;
    const cw = p.canvas.width, ch = p.canvas.height;
    const [ax, ay] = W.llToCanvas(p, S.a.lat, S.a.lon);
    const end = S.b || (S.state === 'pickB' && S.cursor && S.cursor.pane === p.i ? S.cursor : null);
    ctx.save();
    if (end) {
      // The shorter way round in longitude, as the server cuts it; drawn again a lap to each side
      // so a line across the seam is seen leaving one edge and entering the other.
      const bx = ax + shortDLon(end.lon - S.a.lon) / 360 * cw, by = (90 - end.lat) / 180 * ch;
      ctx.setLineDash(S.state === 'pickB' ? [6 * dpr, 4 * dpr] : []);
      for (const [col, w] of [['rgba(6,9,14,.85)', 4.5 * dpr], ['#fff', 2 * dpr]]) {
        ctx.strokeStyle = col; ctx.lineWidth = w;
        for (const sh of [-cw, 0, cw]) { ctx.beginPath(); ctx.moveTo(ax + sh, ay); ctx.lineTo(bx + sh, by); ctx.stroke(); }
      }
      ctx.setLineDash([]);
      if (S.b) {
        const [ex, ey] = W.llToCanvas(p, S.b.lat, S.b.lon);
        mark(ctx, ex, ey, 'B', '#f3f5fa', '#0c1016', dpr);
        if (S.state === 'done' && S.hoverCol !== null && S.data) {
          const f = S.hoverCol / Math.max(1, S.data.n - 1);
          const hx = ax + (bx - ax) * f, hy = ay + (by - ay) * f;
          for (const sh of [-cw, 0, cw]) { ctx.beginPath(); ctx.arc(hx + sh, hy, 5 * dpr, 0, 2 * Math.PI); ctx.fillStyle = '#ffd23f'; ctx.fill(); ctx.strokeStyle = '#0c1016'; ctx.lineWidth = 1.5 * dpr; ctx.stroke(); }
        }
      }
    }
    mark(ctx, ax, ay, 'A', '#7d8cff', '#0a0e28', dpr);
    ctx.restore();
  }
  function mark(ctx, x, y, letter, fill, ink, dpr) {
    ctx.beginPath(); ctx.arc(x, y, 8.5 * dpr, 0, 2 * Math.PI);
    ctx.fillStyle = fill; ctx.fill(); ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.stroke();
    ctx.fillStyle = ink; ctx.font = `700 ${10.5 * dpr}px Inter, system-ui, sans-serif`; ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(letter, x, y + 0.5 * dpr);
  }
  // The rubber band while the second point is chosen.
  for (const p of W.panes) {
    p.canvas.addEventListener('pointermove', ev => {
      if (S.state !== 'pickB' || W.tool() !== 'section' || S.pane !== p.i) return;
      const rect = p.canvas.getBoundingClientRect(), d = window.devicePixelRatio || 1;
      const [lat, lon] = W.mapLatLon(p, (ev.clientX - rect.left) * d, (ev.clientY - rect.top) * d);
      S.cursor = { pane: p.i, lat, lon };
      W.redraw();
    });
  }

  // ===============================================================================================
  // Sections: the data, and drawing it
  // ===============================================================================================
  const shortName = s => String(s).replace(/^deeptime:/, '').replace(/_/g, ' ');
  function prep(d) {
    const c = d.columns, n = c.ground.length;
    d.n = n;
    d.colour = [];
    d.name = [];
    for (const k of d.classes) { d.colour[k.i] = k.colour; d.name[k.i] = k.name.replace(/_/g, ' '); }
    d.bodyInfo = new Map(d.bodies.map(b => [b.id, b]));
    const area = new Map();
    let minGround = Infinity, maxTop = -Infinity, total = 0;
    for (let j = 0; j < n; j++) {
      minGround = Math.min(minGround, c.ground[j]);
      maxTop = Math.max(maxTop, c.ground[j], c.water[j] === null ? -Infinity : c.water[j]);
      const r = c.runs[j];
      for (let k = 0; k < r.length; k += 2) {
        const blocks = r[k] - (k + 2 < r.length ? r[k + 2] : d.min_y - 1);
        area.set(r[k + 1], (area.get(r[k + 1]) || 0) + blocks);
        total += blocks;
      }
    }
    d.stats = { minGround, maxTop, area, total };
    // What kinds of body the line crosses, for the key.
    d.kinds = new Map();
    for (const b of d.bodies) {
      const key = b.deposit_type ? shortName(b.deposit_type) : b.role;
      d.kinds.set(key, (d.kinds.get(key) || 0) + 1);
    }
    return d;
  }
  const ROLE_COLOURS = { ore: '#ffd23f', pluton: '#ff8fb1', gossan: '#e09a5a', lithocap: '#d9d9d9', reservoir: '#59d0ff', blanket: '#ffb347' };
  const roleColour = b => ROLE_COLOURS[b.role] || '#ffffff';
  const kmText = km => km < 10 ? km.toFixed(1) : fmtInt(km);
  const pct = f => f >= 0.1 ? `${(f * 100).toFixed(0)}%` : f >= 0.001 ? `${(f * 100).toFixed(1)}%` : '<0.1%';
  const SKY = '#0a0d13';
  // `a` with `t` of `b` mixed in: #rrggbb.
  function mix(a, b, t) {
    const pa = parseInt(a.slice(1), 16), pb = parseInt(b.slice(1), 16);
    const ch = s => Math.round(((pa >> s) & 255) * (1 - t) + ((pb >> s) & 255) * t);
    return `#${((1 << 24) | (ch(16) << 16) | (ch(8) << 8) | ch(0)).toString(16).slice(1)}`;
  }

  // y range: the whole column down to the floor, or 200 blocks below the lowest ground.
  function yRange(d) {
    const top = Math.max(d.stats.maxTop, d.sea_level_y);
    const ymax = Math.min(d.max_y, top + Math.max(10, (top - d.min_y) * 0.05));
    const ymin = S.range === 'near' ? Math.max(d.min_y, d.stats.minGround - 200) : d.min_y;
    return [ymin, Math.max(ymax, ymin + 20)];
  }

  function drawSection(v) {
    const d = S.data, cv = v.canvas;
    if (!d || !cv.isConnected) return;
    const dpr = window.devicePixelRatio || 1;
    const cw = Math.max(160, cv.clientWidth), ch = Math.max(100, cv.clientHeight);
    const pw = Math.round(cw * dpr), ph = Math.round(ch * dpr);
    if (cv.width !== pw || cv.height !== ph) { cv.width = pw; cv.height = ph; }
    for (const c of [v.back, v.ov]) if (c.width !== pw || c.height !== ph) { c.width = pw; c.height = ph; }
    const g = v.back.getContext('2d');
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const m = { l: 56, r: 24, t: 10, b: 46 };
    const w = cw - m.l - m.r, h = ch - m.t - m.b;
    const [ymin, ymax] = yRange(d);
    const Y = y => m.t + (ymax - y) / (ymax - ymin) * h;
    const xs = j => m.l + j / d.n * w;
    const snap = a => Math.round(a * dpr) / dpr;           // whole device pixels: no seams between columns or beds
    const X0 = j => snap(xs(j)), X1 = j => Math.max(snap(xs(j + 1)), snap(xs(j)) + 1 / dpr);
    v.geo = { cw, ch, m, w, h, ymin, ymax };
    v.note.textContent = scaleNote(d, w, h, ymin, ymax);
    g.fillStyle = SKY; g.fillRect(0, 0, cw, ch);
    g.save();
    g.beginPath(); g.rect(m.l, m.t, w, h); g.clip();
    const c = d.columns, hl = S.hl;
    // Rock
    for (let j = 0; j < d.n; j++) {
      const r = c.runs[j], x0 = X0(j), x1 = X1(j);
      for (let k = 0; k < r.length; k += 2) {
        const top = r[k], cls = r[k + 1];
        const bottom = k + 2 < r.length ? r[k + 2] + 1 : d.min_y;
        if (top + 1 < ymin || bottom > ymax) continue;
        // Colours are opaque (the others dimmed into the sky by mixing, not by alpha), so a pixel
        // that two columns share does not come out darker or lighter than its neighbours.
        const col = cls < 0 ? '#4a4f5a' : (d.colour[cls] || '#808080');
        g.fillStyle = hl !== null && cls !== hl ? mix(col, SKY, 0.83) : col;
        const y0 = snap(Y(Math.min(top + 1, ymax))), y1 = Math.max(snap(Y(Math.max(bottom, ymin))), y0 + 1 / dpr);
        g.fillRect(x0, y0, x1 - x0, y1 - y0);
      }
    }
    // Water over the ground: the sea, lakes and rivers (blue mixed into the sky, as above).
    g.fillStyle = mix('#4080e0', SKY, 0.42);
    for (let j = 0; j < d.n; j++) {
      const wt = c.water[j];
      if (wt === null) continue;
      const y0 = snap(Y(Math.min(wt + 1, ymax))), y1 = snap(Y(c.ground[j] + 1));
      if (y1 > y0) g.fillRect(X0(j), y0, X1(j) - X0(j), y1 - y0);
    }
    // The surface
    g.beginPath();
    for (let j = 0; j < d.n; j++) { const x = (xs(j) + xs(j + 1)) / 2, y = Y(c.ground[j] + 1); if (j) g.lineTo(x, y); else g.moveTo(x, y); }
    g.lineWidth = 1.4; g.strokeStyle = 'rgba(8, 10, 14, .9)'; g.lineJoin = 'round'; g.stroke();
    // Sea level
    if (d.sea_level_y > ymin && d.sea_level_y < ymax) {
      g.setLineDash([5, 4]); g.lineWidth = 1; g.strokeStyle = 'rgba(160, 215, 255, .9)';
      g.beginPath(); g.moveTo(m.l, Y(d.sea_level_y)); g.lineTo(m.l + w, Y(d.sea_level_y)); g.stroke(); g.setLineDash([]);
    }
    // Bodies the line crosses (deposits, seams, plutons): the fill is drawn opaque on a layer of its
    // own and laid over once, so neighbouring columns do not double up; the edges are 1 px lines.
    if (S.bodies && d.bodies.length) {
      const o = v.ov.getContext('2d');
      o.setTransform(dpr, 0, 0, dpr, 0, 0);
      o.clearRect(0, 0, cw, ch);
      const edge = [];
      const has = (arr, id, lo, hi) => { for (let q = 0; q < arr.length; q += 3) if (arr[q] === id && arr[q + 2] <= hi && arr[q + 1] >= lo) return true; return false; };
      for (let j = 0; j < d.n; j++) {
        const sp = c.bodies[j];
        for (let k = 0; k < sp.length; k += 3) {
          const b = d.bodyInfo.get(sp[k]), col = roleColour(b);
          const x0 = X0(j), x1 = X1(j), y0 = snap(Y(Math.min(sp[k + 1] + 1, ymax))), y1 = snap(Y(Math.max(sp[k + 2], ymin)));
          o.fillStyle = col; o.fillRect(x0, y0, x1 - x0, y1 - y0);
          edge.push([col, x0, y0, x1 - x0, 1], [col, x0, y1 - 1, x1 - x0, 1]);
          const prev = j ? c.bodies[j - 1] : [], next = j + 1 < d.n ? c.bodies[j + 1] : [];
          if (!has(prev, sp[k], sp[k + 2], sp[k + 1])) edge.push([col, x0, y0, 1, y1 - y0]);
          if (!has(next, sp[k], sp[k + 2], sp[k + 1])) edge.push([col, x1 - 1, y0, 1, y1 - y0]);
        }
      }
      g.save(); g.setTransform(1, 0, 0, 1, 0, 0); g.globalAlpha = 0.26; g.drawImage(v.ov, 0, 0); g.restore();
      for (const [col, x, y, ww, hh] of edge) { g.fillStyle = col; g.fillRect(x, y, ww, hh); }
    }
    g.restore();
    // Axes
    g.font = '10.5px Inter, system-ui, sans-serif'; g.fillStyle = '#8f9ab0'; g.strokeStyle = 'rgba(160,175,200,.16)'; g.lineWidth = 1;
    g.textBaseline = 'middle'; g.textAlign = 'right';
    for (const y of niceTicks(ymin, ymax, Math.max(3, Math.floor(h / 38)))) {
      const py = Math.round(Y(y)) + 0.5;
      g.beginPath(); g.moveTo(m.l - 4, py); g.lineTo(m.l, py); g.stroke();
      g.fillText(fmtInt(y), m.l - 7, py);
    }
    if (d.sea_level_y > ymin && d.sea_level_y < ymax) { const sy = Y(d.sea_level_y) - 2, label = `sea level y ${d.sea_level_y}`; g.textAlign = 'left'; g.textBaseline = 'bottom'; g.lineWidth = 3; g.strokeStyle = 'rgba(8, 10, 14, .85)'; g.strokeText(label, m.l + 5, sy); g.fillStyle = 'rgba(170, 220, 255, .98)'; g.fillText(label, m.l + 5, sy); g.lineWidth = 1; }
    g.textBaseline = 'top';
    const L = d.length_blocks;
    for (const t of niceTicks(0, L, Math.max(2, Math.floor(w / 100)))) {
      const px = Math.round(m.l + t / L * w) + 0.5;
      g.beginPath(); g.moveTo(px, m.t + h); g.lineTo(px, m.t + h + 4); g.stroke();
      const lab = fmtInt(t), km = `${kmText(t * d.m_per_block_h / 1000)} km`;
      const half = Math.max(g.measureText(lab).width, g.measureText(km).width) / 2;
      g.textAlign = px - half < 2 ? 'left' : px + half > cw - 2 ? 'right' : 'center';
      const tx = g.textAlign === 'left' ? px - 3 : g.textAlign === 'right' ? px + 3 : px;
      g.fillStyle = '#b0b8c7'; g.fillText(lab, tx, m.t + h + 6);
      g.fillStyle = '#6f7a8e'; g.fillText(km, tx, m.t + h + 18);
    }
    g.fillStyle = '#6f7a8e'; g.textAlign = 'right'; g.textBaseline = 'bottom';
    g.fillText('blocks along the line', m.l + w, ch - 1);
    g.save(); g.translate(11, m.t + h / 2); g.rotate(-Math.PI / 2); g.textAlign = 'center'; g.textBaseline = 'top'; g.fillText('block y (elevation)', 0, 0); g.restore();
    g.strokeStyle = 'rgba(160,175,200,.3)'; g.strokeRect(m.l - 0.5, m.t - 0.5, w + 1, h + 1);
    blit(v);
  }
  function blit(v, col) {
    const cv = v.canvas, g = cv.getContext('2d');
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.drawImage(v.back, 0, 0);
    if (col === undefined || col === null || !v.geo) return;
    const dpr = window.devicePixelRatio || 1, { m, w, h } = v.geo, d = S.data;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const x = m.l + (col + 0.5) / d.n * w;
    g.strokeStyle = 'rgba(255,255,255,.75)'; g.lineWidth = 1;
    g.beginPath(); g.moveTo(Math.round(x) + 0.5, m.t); g.lineTo(Math.round(x) + 0.5, m.t + h); g.stroke();
    if (v.hy !== undefined) { g.beginPath(); g.arc(x, v.hy, 3.2, 0, 2 * Math.PI); g.fillStyle = '#ffd23f'; g.fill(); g.strokeStyle = '#0c1016'; g.stroke(); }
  }
  // The scales: the picture's, and the planet's (a block is wider than it is tall).
  function scaleNote(d, w, h, ymin, ymax) {
    const bx = d.length_blocks / w, by = (ymax - ymin) / h;
    const num = x => x < 10 ? x.toFixed(1) : fmtInt(x);
    return `${num(bx)} blocks a pixel across, ${num(by)} down. A block is ${fmtInt(d.m_per_block_h)} m wide and ${fmtInt(d.m_per_block_v)} m tall, ` +
      `so against the real planet this is stretched ${num(bx / by * d.m_per_block_h / d.m_per_block_v)}× vertically.`;
  }

  // A block x of the section (a line across the seam runs past it) as the world numbers it.
  const wrapX = (d, x) => ((((x + d.circumference / 2) % d.circumference) + d.circumference) % d.circumference) - d.circumference / 2;

  // What is at column j, block y: the rock, its depth under the surface, water, bodies.
  function readout(d, j, y) {
    const c = d.columns, ground = c.ground[j];
    let cls = null;
    const r = c.runs[j];
    for (let k = 0; k < r.length; k += 2) {
      const bottom = k + 2 < r.length ? r[k + 2] + 1 : d.min_y;
      if (y <= r[k] && y >= bottom) { cls = r[k + 1]; break; }
    }
    const f = j / Math.max(1, d.n - 1), along = f * d.length_blocks;
    let h = `<b>${fmtInt(along)}</b> blocks · <b>${kmText(along * d.m_per_block_h / 1000)}</b> km along<br>x ${fmtInt(wrapX(d, c.x[j]))} · z ${fmtInt(c.z[j])}<br>`;
    if (y > ground) {
      const wt = c.water[j];
      h += wt !== null && y <= wt ? `y ${fmtInt(y)}: <b>${esc(c.water_kind[j])} water</b>, ${fmtInt(wt - y + 1)} blocks above the bed` : `y ${fmtInt(y)}: air, ${fmtInt(y - ground)} above the ground (y ${fmtInt(ground)})`;
    } else {
      const nm = cls === null || cls < 0 ? 'unclassified crust' : d.name[cls];
      h += `y ${fmtInt(y)}, ${fmtInt(ground - y)} below the surface:<br><span class="sw" style="background:${cls === null || cls < 0 ? '#4a4f5a' : d.colour[cls]}"></span><b>${esc(nm)}</b>`;
      const sp = c.bodies[j];
      for (let k = 0; k < sp.length; k += 3) {
        if (y <= sp[k + 1] && y >= sp[k + 2]) {
          const b = d.bodyInfo.get(sp[k]);
          h += `<br>inside ${esc(b.role)} body ${b.id}${b.deposit_type ? ` · ${esc(shortName(b.deposit_type))}` : ''}${b.site !== null && b.site !== undefined ? ` (site ${b.site})` : ''}`;
        }
      }
    }
    return h;
  }

  function legendHTML(d) {
    const rows = [...d.stats.area.entries()].sort((a, b) => b[1] - a[1]);
    let h = '<h6>Rock in this section <span style="text-transform:none;letter-spacing:0;font-weight:400">· click one to light it up</span></h6>';
    for (const [cls, a] of rows) {
      const nm = cls < 0 ? 'unclassified crust' : d.name[cls], col = cls < 0 ? '#4a4f5a' : d.colour[cls];
      h += `<button type="button" class="chip" data-cls="${cls}" aria-pressed="${S.hl === cls}"><i style="background:${col}"></i>${esc(nm)} <small>${pct(a / d.stats.total)}</small></button>`;
    }
    if (d.kinds.size) {
      h += '<h6>Bodies the line crosses</h6>';
      for (const [k, n] of [...d.kinds.entries()].sort((a, b) => b[1] - a[1])) {
        const sample = d.bodies.find(b => (b.deposit_type ? shortName(b.deposit_type) : b.role) === k);
        h += `<span class="chip"><i class="ring" style="--c:${roleColour(sample)}"></i>${esc(k)} <small>×${n}</small></span>`;
      }
    }
    return h;
  }

  // One view of the current section in `host`: the picture, and beside it the readout, the key and
  // the scales. The dock has one, the enlarged dialog another.
  function buildView(host) {
    host.innerHTML = '';
    const grid = document.createElement('div');
    grid.className = 'sv-grid';
    grid.innerHTML = '<div class="sv-main"><div class="sv-cw"><canvas aria-label="Cross-section of the strata along the line"></canvas></div></div>' +
      `<div class="sv-side"><div class="sv-read">Move over the section to read it.</div><div class="sv-leg" data-key>${legendHTML(S.data)}</div><div class="sv-note"></div></div>`;
    host.appendChild(grid);
    const v = {
      el: grid, canvas: grid.querySelector('canvas'), back: document.createElement('canvas'), ov: document.createElement('canvas'),
      read: grid.querySelector('.sv-read'), note: grid.querySelector('.sv-note'), key: grid.querySelector('[data-key]'), geo: null, hy: undefined,
    };
    const leave = () => { v.hy = undefined; blit(v); v.read.textContent = 'Move over the section to read it.'; if (S.hoverCol !== null) { S.hoverCol = null; W.redraw(); } };
    v.canvas.addEventListener('pointermove', ev => {
      const rect = v.canvas.getBoundingClientRect(), x = ev.clientX - rect.left, y = ev.clientY - rect.top, G = v.geo;
      if (!G || !S.data) return;
      const { m, w, h, ymin, ymax } = G;
      if (x < m.l || x > m.l + w || y < m.t || y > m.t + h) { leave(); return; }
      const j = clamp(Math.floor((x - m.l) / w * S.data.n), 0, S.data.n - 1);
      v.read.innerHTML = readout(S.data, j, Math.floor(ymax - (y - m.t) / h * (ymax - ymin)));
      v.hy = y;
      blit(v, j);
      if (S.hoverCol !== j) { S.hoverCol = j; W.redraw(); }
    });
    v.canvas.addEventListener('pointerleave', leave);
    views.add(v);
    if (typeof ResizeObserver === 'function') new ResizeObserver(() => drawSection(v)).observe(v.canvas.parentNode);
    requestAnimationFrame(() => drawSection(v));
    return v;
  }
  function redrawViews(keyToo) {
    for (const v of [...views]) {
      if (!v.canvas.isConnected) { views.delete(v); continue; }
      if (keyToo) v.key.innerHTML = legendHTML(S.data);
      drawSection(v);
    }
    for (const b of document.querySelectorAll('[data-sec^="range:"]')) b.setAttribute('aria-pressed', String(b.dataset.sec === `range:${S.range}`));
    for (const b of document.querySelectorAll('[data-sec="bodies"]')) b.setAttribute('aria-pressed', String(S.bodies));
    for (const b of document.querySelectorAll('.sec [data-cls]')) b.setAttribute('aria-pressed', String(S.hl === Number(b.dataset.cls)));
  }

  // ===============================================================================================
  // The dock over the map, the Section tab, the enlarged dialog
  // ===============================================================================================
  const ll = p => W.latLonText(p.lat, p.lon);
  const abChip = (t, p) => `<span class="ab ${t === 'B' ? 'b' : ''}">${t}</span> ${esc(ll(p))}`;
  const secs = ms => ms < 1000 ? `${Math.round(ms)} ms` : `${(ms / 1000).toFixed(1)} s`;
  const stats = d => `<b>${fmtInt(d.length_blocks)}</b> blocks · <b>${kmText(d.length_km)}</b> km · ${fmtInt(d.n)} columns · made in ${secs(d.seconds * 1000)}${S.ms ? ` · ${secs(S.ms)} to load` : ''}`;
  const controls = () => `<div class="seg" role="group" aria-label="Depth range shown">` +
    `<button type="button" data-sec="range:column" aria-pressed="${S.range === 'column'}" data-tip="Down to the world floor">Whole column</button>` +
    `<button type="button" data-sec="range:near" aria-pressed="${S.range === 'near'}" data-tip="200 blocks below the lowest ground">Near the surface</button></div>` +
    `<button type="button" class="btn btn-sm" data-sec="bodies" aria-pressed="${S.bodies}" data-tip="Outline the deposits, seams and plutons the line crosses">${ic('box', 'ic-s')}Bodies</button>`;

  const dock = document.createElement('div');
  dock.className = 'secdock'; dock.hidden = true;
  $('viewport').appendChild(dock);
  let dockKey = '';
  function renderDock() {
    // Over the map's lower edge while a section is on its way, ready or failed; out of the way
    // while the next one is being picked, and when it has been closed.
    const show = S.dock !== 'closed' && !!S.a && !!S.b && ['loading', 'done', 'error'].includes(S.state);
    dock.hidden = !show;
    dockOpen = show && S.dock === 'open';
    setHint();
    if (!show) { dockKey = ''; return; }
    const minimal = S.dock === 'min';
    dock.classList.toggle('min', minimal);
    const key = [S.req, S.state, minimal].join('|');
    if (key === dockKey) return;
    dockKey = key;
    const head = S.a && S.b ? `<span class="ab">A</span> ${esc(ll(S.a))} → <span class="ab b">B</span> ${esc(ll(S.b))}` : '';
    const toggle = `<button type="button" class="btn btn-ghost btn-icon btn-sm" data-sec="min" aria-label="${minimal ? 'Show the section' : 'Fold the section'}" data-tip="${minimal ? 'Show' : 'Fold'}"><svg class="ic ic-s" style="transform:rotate(${minimal ? 180 : 0}deg)"><use href="#i-chev"/></svg></button>` +
      '<button type="button" class="btn btn-ghost btn-icon btn-sm" data-sec="close" aria-label="Close the section" data-tip="Close"><svg class="ic ic-s"><use href="#i-x"/></svg></button>';
    if (S.state === 'loading') {
      dock.innerHTML = `<div class="sd-h"><div class="sd-t"><div>${head}</div><div class="sd-s"><span class="spinner" style="display:inline-block;vertical-align:-3px;margin-right:6px"></span>reading the strata along the line…</div></div><div class="sd-acts">${toggle}</div></div>`;
    } else if (S.state === 'error') {
      dock.innerHTML = `<div class="sd-h"><div class="sd-t"><div>${head}</div><div class="sd-s" style="color:var(--failed)">${esc(S.error)}</div></div><div class="sd-acts"><button type="button" class="btn btn-sm" data-sec="retry">Try again</button>${toggle}</div></div>`;
    } else {
      const d = S.data;
      dock.innerHTML = `<div class="sd-h"><div class="sd-t"><div>${head}</div><div class="sd-s">${stats(d)}</div></div>` +
        `<div class="sd-acts">${controls()}<button type="button" class="btn btn-sm" data-sec="big">${ic('full', 'ic-s')}Enlarge</button><button type="button" class="btn btn-sm" data-sec="swap" data-tip="Cut the same line from the other end">Swap</button>${toggle}</div></div><div class="sd-body" style="display:flex;flex:1 1 auto;min-height:0;flex-direction:column"></div>`;
      if (!minimal) buildView(dock.querySelector('.sd-body'));
    }
    placeDock();
  }
  // The dock takes the map's lower edge, unless the line runs there: then the upper one.
  function placeDock() {
    if (dock.hidden || !S.a || !S.b || !W.pane(S.pane)) return;
    const vp = $('viewport').getBoundingClientRect(), cr = W.pane(S.pane).canvas.getBoundingClientRect();
    const y = lat => cr.top - vp.top + (90 - lat) / 180 * cr.height;
    const lo = Math.min(y(S.a.lat), y(S.b.lat)), hi = Math.max(y(S.a.lat), y(S.b.lat));
    const room = dock.offsetHeight + 24;
    const top = hi > vp.height - room && lo > room;
    dock.style.top = top ? '10px' : 'auto';
    dock.style.bottom = top ? 'auto' : '10px';
  }
  addEventListener('resize', placeDock);

  const panel = $('ip-section');
  function renderPanel() {
    if (!panel) return;
    let h = '<div class="sec">';
    if (S.state === 'idle') {
      h += '<p>Cut the planet open along a line. Pick the Section tool <kbd>S</kbd> and click two points on the map: you get the strata between them, in the map’s rock colours, with the surface, water and sea level, and the deposits and seams the line crosses, read from the run’s bake.</p>' +
        `<div class="sec-acts"><button type="button" class="btn btn-primary btn-sm" data-sec="start">${ic('section', 'ic-s')}Cut a section</button></div>` +
        `<div class="callout">${ic('info')}<div>Sections and the depth slider read the <b>finished world</b>, so they need a run that has ended and has a bake. Each answer is made on request (a fraction of a second for a section) and kept.</div></div>`;
    } else if (S.state === 'pickB') {
      h += `<div class="sec-t">${abChip('A', S.a)}</div><p>Now click the second point on the map. Esc cancels.</p><div class="sec-acts"><button type="button" class="btn btn-sm" data-sec="clear">Cancel</button></div>`;
    } else if (S.state === 'loading') {
      h += `<div class="sec-t">${abChip('A', S.a)} → ${abChip('B', S.b)}</div><div style="display:flex;align-items:center;gap:10px"><div class="spinner"></div><span>Reading the strata along the line…</span></div>`;
    } else if (S.state === 'error') {
      h += `<div class="sec-t">${abChip('A', S.a)} → ${abChip('B', S.b)}</div><div class="callout bad">${ic('alert')}<div><b>No section.</b> ${esc(S.error)}</div></div>` +
        '<div class="sec-acts"><button type="button" class="btn btn-sm" data-sec="retry">Try again</button><button type="button" class="btn btn-sm btn-ghost" data-sec="clear">Clear</button></div>';
    } else {
      const d = S.data;
      h += `<div><div class="sec-t">${abChip('A', S.a)} → ${abChip('B', S.b)}</div><div class="sec-s">${stats(d)}</div></div>` +
        `<div class="sec-acts"><button type="button" class="btn btn-primary btn-sm" data-sec="show">Show over the map</button>` +
        `<button type="button" class="btn btn-sm" data-sec="big">${ic('full', 'ic-s')}Enlarge</button><button type="button" class="btn btn-sm" data-sec="swap">Swap A and B</button>` +
        `<button type="button" class="btn btn-sm" data-sec="again">${ic('section', 'ic-s')}New section</button><button type="button" class="btn btn-sm btn-ghost" data-sec="clear">Clear</button></div>` +
        `<div class="sv-leg" style="gap:4px">${legendHTML(d)}</div><div class="sv-note">${esc(d.stats.total ? `${fmtInt(d.stats.total)} blocks of rock in the ${fmtInt(d.n)} columns.` : '')}</div>`;
    }
    const same = o => S.state === 'done' && S.a && o.a.lat === S.a.lat && o.a.lon === S.a.lon && o.b.lat === S.b.lat && o.b.lon === S.b.lon;
    const others = S.recent.filter(o => !same(o));
    if (others.length) {
      h += `<div class="recent"><h5>Recent sections</h5>${others.map(o => `<button type="button" data-recent="${S.recent.indexOf(o)}">${abChip('A', o.a)} → ${abChip('B', o.b)} <span style="margin-left:auto;color:var(--fg-4)">${kmText(o.km)} km</span></button>`).join('')}</div>`;
    }
    panel.innerHTML = h + '</div>';
  }
  function renderAll() { renderPanel(); renderDock(); }

  // One listener for every control of the section views, the dock and the tab.
  document.addEventListener('click', ev => {
    const cls = ev.target.closest('[data-cls]');
    if (cls && S.data) { const k = Number(cls.dataset.cls); S.hl = S.hl === k ? null : k; redrawViews(true); return; }
    const rec = ev.target.closest('[data-recent]');
    if (rec) {
      const o = S.recent[Number(rec.dataset.recent)], pane = o && W.panes.findIndex(p => p.run && p.run.id === o.run);
      if (!o || pane < 0) { toast('That run is not open.', 'warn'); return; }
      S.run = W.panes[pane].run; S.pane = pane; S.a = o.a; S.b = o.b; go();
      return;
    }
    const b = ev.target.closest('[data-sec]');
    if (!b) return;
    const act = b.dataset.sec;
    if (act.startsWith('range:')) { S.range = act.slice(6); redrawViews(false); }
    else if (act === 'bodies') { S.bodies = !S.bodies; redrawViews(false); }
    else if (act === 'start') W.setTool('section');
    else if (act === 'clear') clear();
    else if (act === 'again') { S.state = 'idle'; S.a = S.b = null; S.data = null; renderAll(); W.setTool('section'); W.redraw(); }
    else if (act === 'retry') go();
    else if (act === 'swap') { const t = S.a; S.a = S.b; S.b = t; go(); }
    else if (act === 'big') openBig();
    else if (act === 'min') { S.dock = S.dock === 'min' ? 'open' : 'min'; dockKey = ''; renderAll(); }
    else if (act === 'show') { S.dock = 'open'; dockKey = ''; renderAll(); }
    else if (act === 'close') { S.dock = 'closed'; renderAll(); }
  });
  function openBig() {
    const d = S.data;
    if (!d) return;
    const title = `Section ${ll(S.a)} → ${ll(S.b)} · ${kmText(d.length_km)} km`;
    modal(title, '<div class="sec-big"><div class="sd-h" style="padding:0 14px 8px"><div class="sd-t"></div><div class="sd-acts"></div></div><div id="sec-big-body" style="display:flex;flex:1 1 auto;min-height:0;flex-direction:column"></div></div>', '');
    const big = $('modalbody').querySelector('.sec-big');
    big.querySelector('.sd-t').innerHTML = `<div class="sd-s">${stats(d)}</div>`;
    big.querySelector('.sd-acts').innerHTML = controls();
    buildView($('sec-big-body'));
  }

  window.DTSections = { slice, draw, click, tool };
  setInterval(() => { try { refresh(false); } catch (e) { console.error(e); } }, 500);
  renderAll();
  refresh(true);
})();
