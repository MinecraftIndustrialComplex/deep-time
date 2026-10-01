// SPDX-License-Identifier: LGPL-3.0-or-later
// Deep Time workbench: the deposits overlay (Ben, 2026-10-01, "Deposits layer"). The pure parts
// live here, in one object, `DTDeposits`, so that workbench.js only wires them to its panes:
//
// - the families: one symbol and one colour each (the colours are the validated dark-mode
//   categorical steps of the data-viz palette, in its slot order; shape is the second cue), coal
//   fields drawn as their footprint areas instead of symbols;
// - `prepare(answer)`: the server's `GET api/runs/<id>/deposits` turned into a model (footprint
//   outlines, a size rank inside each family, counts per family);
// - `draw` / `hit`: the overlay on a map canvas and the site under a point, both through a `view`
//   the caller gives ({ project(lat, lon) -> [x, y], ppdx, ppdy (canvas px per degree), worldW
//   (canvas px for 360 degrees), dpr }), so a map that zooms or pans only has to say so there;
// - text: the legend, the hover card, the site's facts and its cause chain.
//
// The overlay draws data the run already keeps (the sites and bodies the deposit stage placed;
// the events the page already streams give each site's age and "why here"): nothing is computed
// by the simulation for it.
'use strict';

(function (root) {
  const D2R = Math.PI / 180;
  const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));

  // ---------------------------------------------------------------------------------------------
  // The families. Colours: the data-viz palette's dark-mode categorical steps (blue, orange, aqua,
  // yellow, magenta, violet, red; green is left out, it is the land's colour), run through its
  // validator in slot order (all checks pass); coal is a neutral charcoal with a light outline.
  // Shapes are the second cue (and the only one where two hues would touch on the map).
  // ---------------------------------------------------------------------------------------------
  const SHAPES = {   // unit paths, about (0, 0), within about ±1.3
    circle: 'M1 0A1 1 0 1 1 -1 0A1 1 0 1 1 1 0Z',
    up: 'M0 -1.2L1.1 .8L-1.1 .8Z',
    down: 'M0 1.2L1.1 -.8L-1.1 -.8Z',
    diamond: 'M0 -1.25L1 0L0 1.25L-1 0Z',
    square: 'M-.9 -.9H.9V.9H-.9Z',
    hexagon: 'M1.05 0L.52 .91L-.52 .91L-1.05 0L-.52 -.91L.52 -.91Z',
    star: 'M0 -1.35L.34 -.34L1.35 0L.34 .34L0 1.35L-.34 .34L-1.35 0L-.34 -.34Z',
    drop: 'M0 -1.25C.5 -.5 .95 .05 .95 .5A.95 .95 0 0 1 -.95 .5C-.95 .05 -.5 -.5 0 -1.25Z',
    area: 'M1.2 0A1.2 .8 0 1 1 -1.2 0A1.2 .8 0 1 1 1.2 0Z',
  };
  // `key` is the deposit type's id without its namespace. Order is the legend's.
  const FAMILIES = [
    { key: 'coal_seam', label: 'Coal seam field', plural: 'Coal seam fields', colour: '#2a2e37', ui: '#b9c1cf', shape: 'area', area: true },
    { key: 'petroleum_trap', label: 'Petroleum trap', plural: 'Petroleum traps', colour: '#9085e9', shape: 'drop' },
    { key: 'porphyry_cu_mo_au', label: 'Porphyry Cu-Mo-Au', plural: 'Porphyry Cu-Mo-Au', colour: '#d95926', shape: 'up' },
    { key: 'supergene_cu_blanket', label: 'Supergene Cu blanket', plural: 'Supergene Cu blankets', colour: '#199e70', shape: 'down' },
    { key: 'placer_au', label: 'Gold placer', plural: 'Gold placers', colour: '#c98500', shape: 'star' },
    { key: 'mvt_zn_pb', label: 'MVT Zn-Pb', plural: 'MVT Zn-Pb', colour: '#3987e5', shape: 'hexagon' },
    { key: 'sedex_zn_pb', label: 'SEDEX Zn-Pb', plural: 'SEDEX Zn-Pb', colour: '#d55181', shape: 'square' },
    { key: 'bif_fe', label: 'Banded iron formation', plural: 'Banded iron formations', colour: '#e66767', shape: 'diamond' },
  ];
  const BY_KEY = new Map(FAMILIES.map(f => [f.key, f]));
  const extra = new Map();   // a datapack's own types: a colour from the name, a circle
  const shortType = t => String(t || '').replace(/^[^:]*:/, '');
  const cap1 = s => (s ? s.charAt(0).toUpperCase() + s.slice(1) : s);
  function familyOf(type) {
    const key = shortType(type);
    const known = BY_KEY.get(key);
    if (known) return known;
    let f = extra.get(key);
    if (!f) {
      let h = 0;
      for (let i = 0; i < key.length; i++) h = (h * 31 + key.charCodeAt(i)) >>> 0;
      const label = cap1(key.replace(/_/g, ' '));
      f = { key, label, plural: label, colour: `hsl(${h % 360}, 62%, 58%)`, shape: 'circle', other: true };
      extra.set(key, f);
    }
    return f;
  }

  // ---------------------------------------------------------------------------------------------
  // Text
  // ---------------------------------------------------------------------------------------------
  const sig = (v, n) => {
    const a = Math.abs(v);
    if (a >= 100) return String(Math.round(v));
    if (a >= 10) return v.toFixed(Math.max(0, n - 2));
    return v.toFixed(Math.max(0, n - 1));
  };
  function fmtTonnes(t) {
    if (!(t >= 0) || !isFinite(t)) return '—';
    if (t < 1e3) return `${sig(t, 3)} t`;
    if (t < 1e6) return `${sig(t / 1e3, 3)} kt`;
    if (t < 1e9) return `${sig(t / 1e6, 3)} Mt`;
    if (t < 1e12) return `${sig(t / 1e9, 3)} Gt`;
    return `${sig(t / 1e12, 3)} Tt`;
  }
  function fmtArea(km2) {
    if (!(km2 >= 0) || !isFinite(km2)) return '—';
    return km2 >= 100 ? `${Math.round(km2).toLocaleString('en-US')} km²` : `${km2.toFixed(km2 >= 10 ? 0 : 1)} km²`;
  }
  const material = m => String(m || '').replace(/^[^:]*:/, '').replace(/_/g, ' ');
  function fmtGrade(g) {
    if (!(g > 0) || g >= 0.99) return '';
    const pct = g * 100;
    return `${pct >= 10 ? pct.toFixed(0) : pct >= 1 ? pct.toFixed(1) : pct.toPrecision(2)} %`;
  }
  const primary = s => s.products.find(p => p.role === 'primary') || s.products[0] || null;
  function sizeText(s) {
    const p = primary(s);
    return p ? `${fmtTonnes(p.tonnage_t)} ${material(p.material)}` : 'size not recorded';
  }

  // ---------------------------------------------------------------------------------------------
  // The model
  // ---------------------------------------------------------------------------------------------
  const RING = 28;
  // `answer`: { source, circumference, m_per_block_h, sites: [{ id, kind, type, lat, lon, x, y, z,
  // y_range, fp: { x, z, a, c, yaw, km2 }, bodies, products, forming_event, parent }] }.
  function prepare(answer) {
    const C = answer.circumference || 16384;
    const sites = (answer.sites || []).map(raw => {
      const family = familyOf(raw.type);
      const fp = raw.fp || { x: raw.x, z: raw.z, a: 1, c: 1, yaw: 0, km2: 0 };
      // The footprint's outline as degrees about its centre (the seam is x = ±C/2, north is −z).
      const poly = [];
      const cy = Math.cos(fp.yaw), sy = Math.sin(fp.yaw);
      for (let i = 0; i < RING; i++) {
        const th = 2 * Math.PI * i / RING, u = fp.a * Math.cos(th), v = fp.c * Math.sin(th);
        poly.push([(u * cy - v * sy) / C * 360, -(u * sy + v * cy) / C * 360]);
      }
      const p = primary(raw);
      return Object.assign({}, raw, {
        family, poly, rank: 0.5,
        clat: -fp.z / C * 360, clon: (((fp.x + C / 2) / C * 360) % 360 + 360) % 360,
        tonnes: p ? p.tonnage_t : 0,
      });
    });
    // Size rank inside each family (0 smallest … 1 largest by the primary product's tonnage).
    const groups = new Map();
    for (const s of sites) { if (!groups.has(s.family.key)) groups.set(s.family.key, []); groups.get(s.family.key).push(s); }
    for (const g of groups.values()) {
      g.sort((a, b) => a.tonnes - b.tonnes);
      g.forEach((s, i) => { s.rank = g.length > 1 ? i / (g.length - 1) : 0.5; });
    }
    // Sites at one spot (a supergene blanket sits under its porphyry) are drawn side by side.
    const spots = new Map();
    for (const s of sites) {
      const key = `${s.lat},${s.lon}`;
      s.nudge = spots.get(key) || 0;
      spots.set(key, s.nudge + 1);
    }
    const counts = new Map();
    for (const s of sites) counts.set(s.family.key, (counts.get(s.family.key) || 0) + 1);
    return {
      source: answer.source, C, kmPerBlock: (answer.m_per_block_h || 300) / 1000, sites,
      byId: new Map(sites.map(s => [s.id, s])),
      byEvent: new Map(sites.map(s => [s.forming_event, s])),
      counts,
    };
  }

  // The families a legend lists: the known ones (counts of zero stay, it says why a map is bare),
  // then any a datapack added.
  function legendFamilies(model) {
    const out = FAMILIES.slice();
    for (const f of extra.values()) if (model.counts.has(f.key)) out.push(f);
    return out;
  }

  // ---------------------------------------------------------------------------------------------
  // Drawing
  // ---------------------------------------------------------------------------------------------
  const MIN_AREA_PX = 3.5;      // an area is at least this many CSS px per semi-axis
  const SYMBOL_PX = [4.4, 7.2]; // symbol radius, smallest to largest of a family
  const paths = {};
  const shapePath = name => paths[name] || (paths[name] = new Path2D(SHAPES[name]));
  const symbolRadius = (s, dpr) => (SYMBOL_PX[0] + (SYMBOL_PX[1] - SYMBOL_PX[0]) * s.rank) * dpr;

  // The footprint's scale-up to the minimum size and the pixel semi-axes (`pa` along the long one).
  function areaScale(s, view) {
    const pa = s.fp.a / view.C * 360 * view.ppdx, pc = s.fp.c / view.C * 360 * view.ppdy;
    return Math.max(1, MIN_AREA_PX * view.dpr / Math.max(pa, pc, 1e-6));
  }
  // Where a site's symbol (or a coal field's centre) is: its spot, a symbol's width over from any
  // site that shares it.
  function anchor(s, view) {
    if (s.family.area) return view.project(s.clat, s.clon);
    const [x, y] = view.project(s.lat, s.lon);
    return [x + s.nudge * 10 * view.dpr, y];
  }
  const copies = (x, ext, view) => [0, -view.worldW, view.worldW].map(d => x + d).filter(cx => cx > -ext && cx < view.worldW + ext);

  function outline(ctx, s, cx, cy, view, k) {
    ctx.beginPath();
    s.poly.forEach(([dlon, dlat], i) => {
      const x = cx + dlon * view.ppdx * k, y = cy - dlat * view.ppdy * k;
      if (i) ctx.lineTo(x, y); else ctx.moveTo(x, y);
    });
    ctx.closePath();
  }
  function symbol(ctx, s, x, y, r, dpr, ring) {
    ctx.save();
    ctx.translate(x, y); ctx.scale(r, r);
    ctx.lineJoin = 'round';
    const path = shapePath(s.family.shape);
    ctx.lineWidth = 3.4 * dpr / r; ctx.strokeStyle = 'rgba(6,8,12,.85)'; ctx.stroke(path);
    ctx.lineWidth = 1.7 * dpr / r; ctx.strokeStyle = '#fff'; ctx.stroke(path);
    ctx.fillStyle = s.family.colour; ctx.fill(path);
    ctx.restore();
    if (ring) {
      ctx.save();
      ctx.beginPath(); ctx.arc(x, y, r + 5 * dpr, 0, 2 * Math.PI);
      ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.setLineDash([4 * dpr, 3 * dpr]); ctx.stroke();
      ctx.restore();
    }
  }

  // state: { hidden: Set of family keys, hover: site id, selected: site id }.
  function draw(ctx, model, view, state) {
    view = Object.assign({ C: model.C }, view);
    const dpr = view.dpr, hidden = state.hidden || new Set();
    const shown = model.sites.filter(s => !hidden.has(s.family.key));
    ctx.save();
    // Coal fields first, as the area they cover, so the symbols sit on top.
    for (const s of shown) {
      if (!s.family.area) continue;
      const [x0, y0] = view.project(s.clat, s.clon), k = areaScale(s, view);
      const ext = Math.max(s.fp.a, s.fp.c) / view.C * 360 * view.ppdx * k;
      const on = s.id === state.selected || s.id === state.hover;
      for (const cx of copies(x0, ext, view)) {
        outline(ctx, s, cx, y0, view, k);
        ctx.fillStyle = on ? 'rgba(46,52,64,.92)' : 'rgba(30,33,41,.78)'; ctx.fill();
        ctx.lineJoin = 'round';
        ctx.lineWidth = (on ? 2.4 : 1.5) * dpr; ctx.strokeStyle = on ? '#fff' : 'rgba(244,246,250,.92)'; ctx.stroke();
      }
    }
    // The footprint of the site under the pointer or selected, for the families drawn as symbols,
    // and the link to the deposit it came from (a placer's porphyry) or the ones that came of it.
    for (const s of shown) {
      const on = s.id === state.selected || s.id === state.hover;
      if (!on) continue;
      const [x0, y0] = view.project(s.clat, s.clon), k = areaScale(s, view);
      if (!s.family.area) {
        for (const cx of copies(x0, 0, view)) {
          outline(ctx, s, cx, y0, view, k);
          ctx.fillStyle = 'rgba(255,255,255,.1)'; ctx.fill();
          ctx.setLineDash([5 * dpr, 3 * dpr]); ctx.lineWidth = 1.5 * dpr; ctx.strokeStyle = s.family.colour; ctx.stroke(); ctx.setLineDash([]);
        }
      }
      for (const o of model.sites) {
        if (o.parent !== s.id && s.parent !== o.id) continue;
        const [x1, y1] = anchor(o, view), [xs, ys] = anchor(s, view);
        ctx.beginPath(); ctx.moveTo(xs, ys); ctx.lineTo(x1, y1);
        ctx.setLineDash([3 * dpr, 3 * dpr]); ctx.lineWidth = 1.4 * dpr; ctx.strokeStyle = 'rgba(255,255,255,.85)'; ctx.stroke(); ctx.setLineDash([]);
      }
    }
    // Symbols: large ones first, so the small stay visible; the picked ones last.
    const marks = shown.filter(s => !s.family.area).sort((a, b) => a.rank - b.rank).reverse();
    for (const pass of [false, true]) {
      for (const s of marks) {
        const on = s.id === state.selected || s.id === state.hover;
        if (on !== pass) continue;
        const [x0, y0] = anchor(s, view), r = symbolRadius(s, dpr) * (on ? 1.25 : 1);
        for (const cx of copies(x0, r + 6 * dpr, view)) symbol(ctx, s, cx, y0, r, dpr, s.id === state.selected);
      }
    }
    // A selected coal field gets its ring too.
    const sel = state.selected != null ? model.byId.get(state.selected) : null;
    if (sel && sel.family.area && !hidden.has(sel.family.key)) {
      const [x0, y0] = view.project(sel.clat, sel.clon), k = areaScale(sel, view);
      const ext = Math.max(sel.fp.a / view.C * 360 * view.ppdx, sel.fp.c / view.C * 360 * view.ppdy) * k;
      for (const cx of copies(x0, ext, view)) {
        ctx.beginPath(); ctx.arc(cx, y0, ext + 5 * dpr, 0, 2 * Math.PI);
        ctx.lineWidth = 2 * dpr; ctx.strokeStyle = '#fff'; ctx.setLineDash([4 * dpr, 3 * dpr]); ctx.stroke(); ctx.setLineDash([]);
      }
    }
    ctx.restore();
  }

  // The site under canvas point (x, y), or null: a symbol within its radius (plus a margin), a
  // coal field anywhere inside its area, the nearest first (a symbol beats an area it sits on).
  function hit(model, view, x, y, state) {
    view = Object.assign({ C: model.C }, view);
    const dpr = view.dpr, hidden = (state && state.hidden) || new Set();
    let best = null, bd = Infinity;
    const AREA_D = 5 * dpr;
    for (const s of model.sites) {
      if (hidden.has(s.family.key)) continue;
      if (s.family.area) {
        const [x0, y0] = view.project(s.clat, s.clon), k = areaScale(s, view);
        const cy = Math.cos(s.fp.yaw), sy = Math.sin(s.fp.yaw);
        for (const cx of copies(x0, Math.max(s.fp.a, s.fp.c) / view.C * 360 * view.ppdx * k, view)) {
          const bx = (x - cx) / view.ppdx / 360 * view.C, bz = (y - y0) / view.ppdy / 360 * view.C;
          const u = bx * cy + bz * sy, v = -bx * sy + bz * cy;
          const d = (u / (s.fp.a * k)) ** 2 + (v / (s.fp.c * k)) ** 2 <= 1 ? AREA_D : Infinity;
          const near = Math.hypot(x - cx, y - y0);
          const dd = Math.min(d, near <= 7 * dpr ? near : Infinity);
          if (dd < bd) { bd = dd; best = s; }
        }
        continue;
      }
      const [x0, y0] = anchor(s, view), r = symbolRadius(s, dpr) + 4 * dpr;
      for (const cx of copies(x0, r, view)) {
        const d = Math.hypot(x - cx, y - y0);
        if (d <= r && d < bd) { bd = d; best = s; }
      }
    }
    return best;
  }

  // ---------------------------------------------------------------------------------------------
  // Legend, hover card, details
  // ---------------------------------------------------------------------------------------------
  function swatch(f, size) {
    const sz = size || 16, p = SHAPES[f.shape];
    const stroke = f.area ? 'rgba(244,246,250,.92)' : '#fff';
    return `<svg class="dep-sw" width="${sz}" height="${sz}" viewBox="-1.6 -1.6 3.2 3.2" aria-hidden="true"><path d="${p}" fill="${f.colour}" stroke="${stroke}" stroke-width="${f.area ? 0.2 : 0.22}" stroke-linejoin="round"/></svg>`;
  }
  // `info`: { sites: n shown at this frame, note: extra text }. Rows toggle a family.
  function legendHTML(model, hidden, info) {
    const total = model.sites.length;
    const shownN = model.sites.filter(s => !hidden.has(s.family.key)).length;
    let h = `<div class="lh"><span class="lname">Deposits</span><span class="lunit">${shownN === total ? total : `${shownN} of ${total}`} ${total === 1 ? 'site' : 'sites'}</span></div>`;
    h += '<div class="dep-rows">';
    for (const f of legendFamilies(model)) {
      const n = model.counts.get(f.key) || 0, off = hidden.has(f.key);
      h += `<button class="dep-row${n ? '' : ' zero'}" data-fam="${esc(f.key)}" aria-pressed="${!off}" data-tip="${esc(off ? `Show ${f.plural.toLowerCase()}` : `Hide ${f.plural.toLowerCase()}`)}">${swatch(f)}<span class="nm">${esc(f.plural)}</span><span class="n">${n}</span></button>`;
    }
    h += '</div>';
    const where = model.source === 'bake' ? 'The finished run’s baked sites.' : 'Sites as the deposit stage placed them.';
    h += `<div class="note">${esc((info && info.note) || `${where} Coal fields are drawn as their footprint (at least 7 px across), the rest as symbols. Hover for size and age; click for why here.`)}</div>`;
    return h;
  }

  // The hover card. `age`: text such as "312.4 Myr ago" ('' when the forming event is unknown).
  function tipHTML(s, age) {
    const f = s.family, p = primary(s);
    const others = s.products.filter(q => q !== p).map(q => material(q.material));
    const grade = p ? fmtGrade(p.grade) : '';
    let h = `<div class="dt-h">${swatch(f, 16)}<b>${esc(f.label)}</b><span class="id">site ${s.id}</span></div>`;
    h += `<div class="dt-r"><span>Size</span><b>${esc(sizeText(s))}</b></div>`;
    h += `<div class="dt-r"><span>Footprint</span><b>${esc(fmtArea(s.fp.km2))}</b></div>`;
    h += `<div class="dt-r"><span>Age</span><b>${esc(age ? `formed ${age}` : 'not in the event log')}</b></div>`;
    if (grade || others.length) h += `<div class="dt-r dim"><span>${esc(grade ? `grade ${grade}` : '')}</span><b>${others.length ? `with ${esc(others.join(', '))}` : ''}</b></div>`;
    return h;
  }

  // Rows [label, html] of the details card.
  function factRows(s, model, helpers) {
    const h = helpers || {};
    const yr = s.y_range || [s.y, s.y];
    const rows = [
      ['Family', esc(s.family.label)],
      ['Size', esc(sizeText(s))],
      ['Footprint', `${esc(fmtArea(s.fp.km2))} · ${esc(Math.round(s.fp.a * 2))} × ${esc(Math.round(s.fp.c * 2))} blocks`],
      ['Where', esc(h.where || `${s.lat.toFixed(1)}°, ${s.lon.toFixed(1)}°`)],
      ['Block x, z', `${esc(s.x)}, ${esc(s.z)}`],
      ['Depth', `y ${esc(yr[0])} … ${esc(yr[1])}${s.bodies > 1 ? ` · ${esc(s.bodies)} bodies` : ''}`],
    ];
    if (h.age) rows.splice(2, 0, ['Formed', esc(h.age)]);
    const yields = s.products.map(q => {
      const g = fmtGrade(q.grade);
      return `${esc(material(q.material))}${q.role ? ` (${esc(q.role)})` : ''}${g ? ` · ${esc(g)}` : ''} · ${esc(fmtTonnes(q.tonnage_t))}`;
    });
    rows.push(['Yields', yields.join('<br>') || '—']);
    return rows;
  }

  // The causes of an event, breadth first, up to `depth` links away, each once, as { id, depth }
  // (the order `deeptime inspect-column --why` lists them in).
  function causeChain(byId, id, depth) {
    const out = [], seen = new Set([id]);
    let frontier = [id];
    for (let d = 1; d <= (depth || 4); d++) {
      const next = [];
      for (const f of frontier) {
        const e = byId.get(f);
        if (!e) continue;
        for (const c of e.causes || []) {
          if (seen.has(c)) continue;
          seen.add(c);
          if (byId.has(c)) { out.push({ id: c, depth: d }); next.push(c); }
        }
      }
      if (!next.length) break;
      frontier = next;
    }
    return out;
  }

  const CSS = `
.seg > button.dim:not([aria-pressed="false"]) { opacity: .62; }
.legend.dlegend { border-top: 1px solid var(--line-1); padding-top: 12px; margin-top: 2px; }
.legend.dlegend .lh .lname { display: block; }
.dep-rows { display: grid; grid-template-columns: 1fr; gap: 2px; margin-top: 10px; }
.dep-row { display: flex; align-items: center; gap: 9px; height: 24px; padding: 0 8px; border-radius: 7px; border: 1px solid transparent; background: transparent; color: var(--fg-2); font-size: 12px; text-align: left; cursor: pointer; }
.dep-row:hover { background: var(--bg-3); color: var(--fg-1); }
.dep-row .dep-sw { flex: 0 0 auto; }
.dep-row .nm { flex: 1 1 auto; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dep-row .n { color: var(--fg-3); font-variant-numeric: tabular-nums; font-weight: 600; }
.dep-row.zero .n { color: var(--fg-4); font-weight: 500; }
.dep-row[aria-pressed="false"] { opacity: .42; }
.dep-row[aria-pressed="false"] .nm { text-decoration: line-through; }
.dep-tip { position: absolute; z-index: 6; pointer-events: none; min-width: 190px; max-width: 280px; padding: 8px 10px 9px; border-radius: 9px; background: rgba(12,16,22,.94); border: 1px solid var(--line-3); box-shadow: 0 8px 24px rgba(0,0,0,.45); color: var(--fg-2); font-size: 12px; line-height: 1.4; }
.dep-tip .dt-h { display: flex; align-items: center; gap: 7px; margin-bottom: 5px; color: var(--fg-1); }
.dep-tip .dt-h b { font-weight: 600; }
.dep-tip .dt-h .id { margin-left: auto; color: var(--fg-4); font: 11px var(--mono); }
.dep-tip .dt-r { display: flex; justify-content: space-between; gap: 14px; padding: 1px 0; }
.dep-tip .dt-r span { color: var(--fg-3); }
.dep-tip .dt-r b { color: var(--fg-1); font-weight: 500; text-align: right; font-variant-numeric: tabular-nums; }
.dep-tip .dt-r.dim b, .dep-tip .dt-r.dim span { color: var(--fg-3); font-weight: 400; }
.detail .dep-why { margin: 0 0 4px; }
.detail .dep-why .chain .lv { display: flex; }
.detail .dep-why .chain .lv > button { flex: 1 1 auto; min-width: 0; }
.detail .dep-why .lvn { flex: 0 0 auto; width: 18px; color: var(--fg-4); font-size: 10.5px; text-align: center; align-self: center; font-variant-numeric: tabular-nums; }
.detail .dep-kin { display: flex; gap: 6px; flex-wrap: wrap; }
`;
  function injectCSS() {
    if (typeof document === 'undefined' || document.getElementById('dep-style')) return;
    const el = document.createElement('style');
    el.id = 'dep-style';
    el.textContent = CSS;
    document.head.appendChild(el);
  }
  injectCSS();

  root.DTDeposits = {
    FAMILIES, SHAPES, familyOf, prepare, draw, hit, legendHTML, legendFamilies, tipHTML, factRows,
    causeChain, swatch, sizeText, fmtTonnes, fmtArea, fmtGrade, material, symbolRadius,
  };
})(typeof window !== 'undefined' ? window : globalThis);
