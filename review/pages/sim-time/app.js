// SPDX-License-Identifier: LGPL-3.0-or-later
// The sim-time page: draws data.js (tools/review/sim-time-analyse.py) and findings.js.
(function () {
  "use strict";
  const D = window.SIMTIME;
  const F = window.FINDINGS || {};
  const full = D.full;
  const hz = full.icicle_hz || 199;
  const NS = "http://www.w3.org/2000/svg";
  const $ = (id) => document.getElementById(id);
  const fmt = (v, d = 0) => (v == null || !isFinite(v)) ? "n/a"
    : Number(v).toLocaleString("en-US", { minimumFractionDigits: d, maximumFractionDigits: d });
  const sec = (v, d) => fmt(v, d ?? (Math.abs(v) < 10 ? 1 : 0)) + "\u00a0s";
  const pct = (v, d = 1) => fmt(v * 100, d) + "\u00a0%";
  const css = (n) => getComputedStyle(document.documentElement).getPropertyValue(n).trim();
  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  function el(tag, attrs, parent, text) {
    const e = document.createElementNS(NS, tag);
    for (const k in attrs || {}) e.setAttribute(k, attrs[k]);
    if (text != null) e.textContent = text;
    if (parent) parent.appendChild(e);
    return e;
  }
  function h(tag, attrs, parent, html) {
    const e = document.createElement(tag);
    for (const k in attrs || {}) e.setAttribute(k, attrs[k]);
    if (html != null) e.innerHTML = html;
    if (parent) parent.appendChild(e);
    return e;
  }
  const tip = $("tip");
  function showTip(ev, html) {
    tip.innerHTML = html;
    tip.hidden = false;
    const w = tip.offsetWidth, hgt = tip.offsetHeight;
    let x = ev.clientX + 14, y = ev.clientY + 14;
    if (x + w > window.innerWidth - 8) x = ev.clientX - w - 14;
    if (y + hgt > window.innerHeight - 8) y = ev.clientY - hgt - 14;
    tip.style.left = Math.max(4, x) + "px";
    tip.style.top = Math.max(4, y) + "px";
  }
  const hideTip = () => { tip.hidden = true; };
  function lum(hex) {
    const n = parseInt(hex.slice(1), 16);
    const c = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => {
      v /= 255;
      return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    });
    return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
  }
  const textOn = (hex) => (lum(hex) > 0.32 ? "var(--on-light)" : "var(--on-dark)");

  // ------------------------------------------------------------------------------------------
  // systems: the map's nodes, each a set of span paths. Colours are the CGMW map colours of
  // geologic periods (Permian, Jurassic, Cretaceous, Paleogene, Neogene, Devonian, ...).
  const SYS = [
    { k: "create", label: "Session create", c: "#a9b4ae", roots: [["cli:create"]], x: 80, y: 215,
      what: "Reads the preset, builds the plate mesh, the stages and the session's thread pool." },
    { k: "plates", label: "Plates", c: "#f04028", roots: [["cli:drive", "stage:plates"], ["cli:drive", "session:join", "stage:plates"]], x: 180, y: 120,
      what: "The plate step every 2 Myr: motion, boundaries, collisions, rifting, the resample of the mesh and the L1 grid (2,048 × 656) written from it." },
    { k: "climate", label: "Climate", c: "#34b2c9", roots: [["cli:drive", "stage:climate"]], x: 400, y: 95, lp: "r",
      what: "The zonal climate clock every step, the per-cell snapshot every 10 Myr (EBM, ocean, belts, orographic rain), ice and the carbon cycle." },
    { k: "surface", label: "Surface", c: "#7fc64e", roots: [["cli:drive", "stage:surface"]], x: 330, y: 285, lp: "l",
      what: "Erosion and deposition on L1: advection, routing, stream power, diffusion with slope failure, flexure, isostasy, strata and the history frames." },
    { k: "volcanoes", label: "Volcanoes", c: "#fd9a52", roots: [["cli:drive", "stage:volcanoes"]], x: 500, y: 285,
      what: "Arcs, hotspots, shields, calderas and flood basalts built onto the surface." },
    { k: "impacts", label: "Impacts", c: "#ffe619", roots: [["cli:drive", "stage:impacts"]], x: 620, y: 285, lp: "t",
      what: "Meteor craters, ejecta and their degradation." },
    { k: "structwait", label: "Wait for structure", c: "#cb8c37", roots: [["cli:drive", "session:join"]], self: true, x: 90, y: 330,
      what: "The plate step has finished and the session waits for the structure step running beside it: the part of the beside work that lands on the critical path." },
    { k: "beside", label: "Structure beside plates", c: "#cb8c37", roots: [["cli:drive", "session:join", "beside:structure"]], offpath: true, x: 510, y: 395,
      what: "The structure step of step k running inside rayon::join next to the plate step of k + 1 (P1-19). Off the critical path except for the wait." },
    { k: "structure", label: "Structure on the world", c: "#cb8c37", roots: [["cli:drive", "stage:structure"]], x: 700, y: 395,
      what: "The structure steps that cannot run beside: every keyframe step with its layers and the last step with folds, faults and bodies (the horizons bake)." },
    { k: "keyframes", label: "Keyframes and events", c: "#b3e1b6", roots: [["cli:drive", "session:after_step"]], x: 330, y: 545,
      what: "After each step: events published, and every 50 Myr a keyframe image drawn and pushed (LZ4, disk and ring)." },
    { k: "memory", label: "Memory room and spill", c: "#9aa8c9", roots: [["cli:drive", "session:make_room"], ["cli:drive", "session:trim_history"], ["cli:drive", "session:detach"], ["cli:drive", "session:attach"]], x: 160, y: 545,
      what: "Reserving the stage's memory under the cap, spilling history frames to disk, detaching and attaching the beside stage's world copy." },
    { k: "finish", label: "Stage finish", c: "#9aa8c9", roots: [["cli:drive", "finish:*"]], x: 500, y: 545,
      what: "Each stage's finish once its steps are done (the group's last keyframe and outputs included)." },
    { k: "driver", label: "Driver loop", c: "#a9b4ae", roots: [["cli:drive"]], self: true, x: 75, y: 615,
      what: "The session's drive loop between spans: scheduling the next step, locks, log forwarding." },
    { k: "deposits", label: "Deposits", c: "#67a599", roots: [["cli:drive", "stage:deposits"]], x: 800, y: 110, lp: "r",
      what: "Ore deposits from the recorded history: every frame decoded, the cover's horizons baked, rules evaluated, bodies placed." },
    { k: "young", label: "Young landscape", c: "#009270", roots: [["cli:drive", "stage:young"]], x: 800, y: 230, lp: "r",
      what: "The last 2 Myr of landscape on L2 (4,096 × 1,312): routing, stream power, diffusion, glacial." },
    { k: "amplify", label: "Amplify", c: "#f9f97f", roots: [["cli:drive", "stage:amplify"]], x: 800, y: 350, lp: "r",
      what: "L2 to L3 (8,192 × 2,624): seeded detail, erosion, deposition, thermal passes, flood repair, character." },
    { k: "finalize", label: "Finalize", c: "#812b92", roots: [["cli:finalize"]], x: 800, y: 480, lp: "r",
      what: "The bake (terrain, strata, the climate stage's layers, water, surface, meta layers) and the projection of every layer into the world folder." },
    { k: "process", label: "Start-up, report, tear-down", c: "#a9b4ae", roots: [["process:start-up"], ["cli:report"], ["process:tear-down"]], x: 260, y: 615,
      what: "Process start before the session, the bench's own report after it, and freeing everything at exit." },
  ];
  const SYSK = Object.fromEntries(SYS.map((s) => [s.k, s]));
  // [from, to, label, options]: fb = the dashed feedback into the next step; t = where along the
  // edge the label sits; a = its anchor beside the edge (start, end or middle); bend = a curve
  const EDGES = [
    ["create", "plates", "first state", { t: 0.3, a: "end" }], ["plates", "climate", "topography, crust", { a: "middle" }],
    ["plates", "surface", "uplift, motion", { t: 0.8, a: "end" }], ["climate", "surface", "rain, temperature, ice", { t: 0.3, a: "end" }],
    ["surface", "volcanoes", ""], ["volcanoes", "impacts", ""], ["impacts", "beside", "a detached copy", { t: 0.7, a: "end" }],
    ["impacts", "structure", "keyframe steps", { a: "start" }], ["surface", "climate", "next step", { fb: true }],
    ["surface", "keyframes", ""], ["surface", "memory", "history frames", { a: "end" }], ["structure", "deposits", "cover, horizons", { t: 0.62, a: "end" }],
    ["surface", "deposits", "frames", { a: "middle" }], ["deposits", "young", ""], ["young", "amplify", "L2", { a: "start" }],
    ["amplify", "finalize", "L3", { a: "start" }], ["deposits", "finalize", "", { bend: -70 }],
  ];

  const byPath = new Map(full.nodes.map((n) => [n.path.join(">"), n]));
  function matchSpec(list, spec, key) {
    const out = [];
    for (const n of list) {
      const p = n[key || "path"];
      if (p.length !== spec.length) continue;
      let ok = true;
      for (let i = 0; i < spec.length; i++) {
        const s = spec[i];
        if (s.endsWith("*") ? !p[i].startsWith(s.slice(0, -1)) : p[i] !== s) { ok = false; break; }
      }
      if (ok) out.push(n);
    }
    return out;
  }
  function sysStats(s) {
    const r = { wall: 0, work: 0, kern: 0, spin: 0, count: 0, cpuTrace: 0, nodes: [] };
    for (const spec of s.roots) for (const n of matchSpec(full.nodes, spec)) {
      r.nodes.push(n);
      r.wall += s.self ? n.self_s : n.wall_s;
      r.work += s.self ? n.self_work_s : n.work_s;
      r.kern += s.self ? n.self_kernel_s : n.kernel_s;
      r.spin += s.self ? n.self_spin_s : n.spin_s;
      r.count += s.self ? 0 : n.count;
    }
    r.cpu = r.work + r.kern;
    r.busy = r.wall > 0 ? r.cpu / r.wall : 0;
    r.headroom = Math.max(0, r.wall - r.cpu / 8);
    return r;
  }
  for (const s of SYS) s.st = sysStats(s);
  const crit = SYS.filter((s) => !s.offpath).reduce((a, s) => a + s.st.wall, 0);
  const summ = full.summary || {};
  const plain = D.plain || null;
  const plainWall = plain ? (plain.log && plain.log.total_s) || plain.total_s : null;
  const profWall = (summ.log && summ.log.total_s) || summ.total_s || crit;
  const runnerWall = summ.wall_s || crit;
  const gate = F.gate_s || 900;
  const headWall = plainWall || profWall;
  const headCpu = plain && plain.user_s != null ? plain.user_s + plain.system_s : (summ.user_s || 0) + (summ.system_s || 0);

  // the sub-systems of a system: its root nodes' children, merged by name
  function parts(s) {
    if (s.self) return [];
    const m = new Map();
    for (const root of s.st.nodes) {
      const d = root.path.length;
      for (const n of full.nodes) {
        if (n.path.length !== d + 1) continue;
        let ok = true;
        for (let i = 0; i < d; i++) if (n.path[i] !== root.path[i]) { ok = false; break; }
        if (!ok || n.lane !== root.lane) continue;
        const key = n.path[d];
        const a = m.get(key) || { key, wall: 0, self: 0, count: 0, work: 0, kern: 0, spin: 0, kids: [] };
        a.wall += n.wall_s; a.self += n.self_s; a.count += n.count; a.work += n.work_s; a.kern += n.kernel_s; a.spin += n.spin_s;
        a.kids.push(n);
        m.set(key, a);
      }
    }
    return [...m.values()].sort((a, b) => b.wall - a.wall);
  }
  function nice(key) {
    const [cat, what] = key.split(":");
    if (!what) return key;
    if (cat === "stage" || cat === "cli" || cat === "session") return what.replace(/_/g, " ");
    return cat + " · " + what.replace(/_/g, " ");
  }

  // ------------------------------------------------------------------------------------------
  // header
  function header() {
    const gap = headWall - gate;
    $("lede").innerHTML = F.lede || `The gate run takes ${sec(headWall)} on 8 pinned cores against a ${sec(gate)} gate.`;
    const hd = $("headline");
    const tiles = [
      [fmt(headWall, 0), "s", `wall time, quiet, set A${plainWall ? " (plain build)" : ""}`],
      [fmt(gate, 0), "s", `the gate; the run is ${fmt(headWall / gate, 2)}× it, ${sec(gap)} over`],
      [fmt(headCpu, 0), "s", "CPU time, user + system"],
      [fmt(headCpu / headWall, 2), "of 8", "cores busy on average"],
      [fmt(full.reconcile.perf_spin_s / runnerWall, 2), "cores", "spinning in rayon's wait loop on average (perf)"],
    ];
    for (const [v, u, l] of tiles) h("div", {}, hd, `<div class="big">${v}<small>${u}</small></div><div class="lab">${l}</div>`);
    // run bar: the critical path in run order, the gate marked
    const order = ["create", "plates", "climate", "surface", "volcanoes", "impacts", "structwait", "structure", "keyframes", "memory", "finish", "driver", "deposits", "young", "amplify", "finalize", "process"];
    const pb = $("phasebar");
    const total = order.reduce((a, k) => a + SYSK[k].st.wall, 0);
    const seg = h("div", { class: "seg", role: "img", "aria-label": "The run's critical path by system, with the 900 s gate marked" }, pb);
    const leg = $("phaselegend");
    for (const k of order) {
      const s = SYSK[k];
      const w = s.st.wall / total;
      if (w <= 0) continue;
      const d = h("div", {}, seg, w > 0.04 ? `<span>${esc(s.label)}</span>` : "");
      d.style.width = (w * 100) + "%";
      d.style.background = s.c;
      d.style.color = textOn(s.c);
      d.addEventListener("mousemove", (ev) => showTip(ev, `<b>${esc(s.label)}</b><br>${sec(s.st.wall)} on the critical path (${pct(s.st.wall / total)})`));
      d.addEventListener("mouseleave", hideTip);
      if (w > 0.004) h("span", {}, leg, `<i style="background:${s.c}"></i>${esc(s.label)} ${sec(s.st.wall)}`);
    }
    if (window.ResizeObserver) {
      const ro = new ResizeObserver(() => fitSeg());
      ro.observe(seg);
      for (const sp of seg.querySelectorAll("span")) ro.observe(sp);
    }
    const g = h("div", { class: "gate" }, pb, `<span>gate ${sec(gate)}</span>`);
    g.style.left = Math.min(100, gate / total * 100) + "%";
    $("verdict").innerHTML = F.verdict || "";
    for (const f of F.findings || []) h("li", {}, $("findings"), `<span>${f}</span>`);
  }

  // a segment's name only where it fits whole (phone width cuts the short ones)
  function fitSeg() {
    for (const d of document.querySelectorAll(".seg div")) {
      const sp = d.firstElementChild;
      if (sp) sp.style.visibility = sp.offsetWidth > d.clientWidth - 11 ? "hidden" : "";
    }
  }

  // ------------------------------------------------------------------------------------------
  // map
  let selected = "surface";
  function drawMap() {
    const box = $("map");
    box.innerHTML = "";
    const svg = el("svg", { viewBox: "0 0 960 665", role: "group", "aria-label": "System map" }, box);
    const defs = el("defs", {}, svg);
    const mk = el("marker", { id: "arr", viewBox: "0 0 10 10", refX: "9", refY: "5", markerWidth: "7", markerHeight: "7", orient: "auto-start-reverse" }, defs);
    el("path", { d: "M0,0 L10,5 L0,10 z", fill: css("--muted") }, mk);
    const maxT = Math.max(...SYS.map((s) => s.st.wall));
    const R = (s) => Math.max(5, 58 * Math.sqrt(s.st.wall / maxT));
    el("rect", { class: "groupbox", x: 20, y: 30, width: 745, height: 435, rx: 10 }, svg);
    el("text", { class: "glab", x: 34, y: 50 }, svg, "COUPLED GROUP · ONE STEP PER 2 MYR · 500 STEPS");
    el("text", { class: "glab", x: 778, y: 50 }, svg, "AFTER THE GROUP");
    for (const [a, b, lab, o = {}] of EDGES) {
      const A = SYSK[a], B = SYSK[b];
      const dx = B.x - A.x, dy = B.y - A.y, L = Math.hypot(dx, dy) || 1;
      const ra = R(A) + 3, rb = R(B) + 6;
      const x1 = A.x + dx / L * ra, y1 = A.y + dy / L * ra, x2 = B.x - dx / L * rb, y2 = B.y - dy / L * rb;
      let d, lx, ly;
      if (o.fb) {
        const cx = (x1 + x2) / 2 + 70, cy = (y1 + y2) / 2;
        d = `M${x1 + 10},${y1} Q${cx},${cy} ${x2 + 12},${y2}`;
        lx = (x1 + x2) / 2 + 48; ly = cy;
      } else if (o.bend) {
        const cx = (x1 + x2) / 2 + o.bend, cy = (y1 + y2) / 2;
        d = `M${x1},${y1} Q${cx},${cy} ${x2},${y2}`;
        lx = (x1 + x2) / 2 + o.bend / 2; ly = cy;
      } else {
        d = `M${x1},${y1} L${x2},${y2}`;
        const t = o.t ?? 0.5;
        lx = x1 + (x2 - x1) * t; ly = y1 + (y2 - y1) * t;
      }
      el("path", { class: "edge" + (o.fb ? " fb" : ""), d, "marker-end": "url(#arr)" }, svg);
      if (lab) {
        const an = o.fb ? "middle" : o.a || "middle";
        const ox = an === "end" ? -8 : an === "start" ? 8 : 0;
        el("text", { class: "elab", x: lx + ox, y: ly + (an === "middle" ? -4 : 4), "text-anchor": an }, svg, lab);
      }
    }
    for (const s of SYS) {
      const r = R(s);
      const g = el("g", { class: "node" + (s.k === selected ? " sel" : ""), tabindex: "0", role: "button", "aria-label": `${s.label}: ${fmt(s.st.wall)} seconds` }, svg);
      el("circle", { class: "body", cx: s.x, cy: s.y, r, fill: s.c, opacity: s.offpath ? 0.55 : 1, stroke: s.offpath ? css("--fg") : "none", "stroke-dasharray": s.offpath ? "3 3" : "" }, g);
      // ring: cores busy out of 8
      const rr = r + 4, frac = Math.min(1, s.st.busy / 8);
      el("circle", { cx: s.x, cy: s.y, r: rr, fill: "none", stroke: css("--rule"), "stroke-width": 3 }, g);
      if (frac > 0) {
        const C = 2 * Math.PI * rr;
        el("circle", { cx: s.x, cy: s.y, r: rr, fill: "none", stroke: css("--accent"), "stroke-width": 3,
          "stroke-dasharray": `${C * frac} ${C}`, transform: `rotate(-90 ${s.x} ${s.y})` }, g);
      }
      // the label under the node, or beside it (lp: t, l, r) where an edge would cross it
      const lp = s.lp || "b";
      const lx = lp === "l" ? s.x - rr - 8 : lp === "r" ? s.x + rr + 8 : s.x;
      const ly = lp === "t" ? s.y - rr - 22 : lp === "b" ? s.y + rr + 16 : s.y - 3;
      const an = lp === "l" ? "end" : lp === "r" ? "start" : "middle";
      el("text", { class: "nlab", x: lx, y: ly, "text-anchor": an }, g, s.label);
      el("text", { class: "nsec", x: lx, y: ly + 15, "text-anchor": an }, g,
        `${sec(s.st.wall)} · ${fmt(s.st.busy, 1)} cores`);
      const pick = () => { selected = s.k; drawMap(); drawPanel(); };
      g.addEventListener("click", pick);
      g.addEventListener("keydown", (ev) => { if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); pick(); } });
      g.addEventListener("mousemove", (ev) => showTip(ev, `<b>${esc(s.label)}</b><br>${sec(s.st.wall)}${s.offpath ? " beside (off the critical path)" : " on the critical path"}<br>${fmt(s.st.busy, 2)} of 8 cores working`));
      g.addEventListener("mouseleave", hideTip);
    }
  }

  function scalingFor(s) {
    if (!D.scaling) return null;
    const walls = {};
    for (const spec of s.roots) for (const n of matchSpec(D.scaling.nodes, spec)) {
      const src = s.self ? n.selfs : n.walls;
      for (const k in src) walls[k] = (walls[k] || 0) + src[k];
    }
    const ns = Object.keys(walls).map(Number).sort((a, b) => a - b);
    if (!ns.length) return null;
    const pts = ns.filter((n) => n <= 8).map((n) => [1 / n, walls[n]]);
    const f = fit(pts);
    return { walls, ns, S: f ? f[0] : null, P: f ? f[1] : null };
  }
  function fit(pts) {
    const n = pts.length;
    if (n < 2) return null;
    let sx = 0, sy = 0, sxx = 0, sxy = 0;
    for (const [x, y] of pts) { sx += x; sy += y; sxx += x * x; sxy += x * y; }
    const den = n * sxx - sx * sx;
    if (!den) return null;
    let p = (n * sxy - sx * sy) / den, s = (sy - p * sx) / n;
    if (s < 0) { s = 0; p = sxy / sxx; }
    if (p < 0) { s = sy / n; p = 0; }
    return [s, p];
  }

  function miniScaling(parent, sc, color, w = 260, hgt = 150) {
    const svg = el("svg", { viewBox: `0 0 ${w} ${hgt}`, role: "img" }, parent);
    const ns = [1, 2, 4, 8, 16];
    const vals = sc.ns.map((n) => sc.walls[n]).filter((v) => v > 0);
    if (!vals.length) return svg;
    const ideal = sc.walls[1] ? ns.map((n) => sc.walls[1] / n) : [];
    const all = vals.concat(ideal, sc.S != null ? [sc.S + sc.P / 16] : []).filter((v) => v > 0);
    const lo = Math.min(...all) * 0.8, hi = Math.max(...all) * 1.15;
    const L = 40, Rm = 10, T = 10, B = 24;
    const X = (n) => L + (Math.log2(n) / 4) * (w - L - Rm);
    const Y = (v) => T + (1 - (Math.log(v) - Math.log(lo)) / (Math.log(hi) - Math.log(lo))) * (hgt - T - B);
    const ax = el("g", { class: "axis" }, svg);
    for (const n of ns) {
      el("line", { x1: X(n), x2: X(n), y1: T, y2: hgt - B }, ax);
      el("text", { x: X(n), y: hgt - 8, "text-anchor": "middle" }, ax, n);
    }
    const ticks = niceLogTicks(lo, hi);
    for (const t of ticks) {
      el("line", { x1: L, x2: w - Rm, y1: Y(t), y2: Y(t) }, ax);
      el("text", { x: L - 4, y: Y(t) + 4, "text-anchor": "end" }, ax, t >= 1000 ? fmt(t / 1000, 0) + "k" : fmt(t, t < 1 ? 1 : 0));
    }
    el("text", { x: w - Rm, y: hgt - 8, "text-anchor": "end", class: "axis" }, svg).textContent = "";
    if (ideal.length) el("path", { d: ns.map((n, i) => `${i ? "L" : "M"}${X(n)},${Y(ideal[i])}`).join(""), fill: "none", stroke: css("--muted"), "stroke-dasharray": "3 3", "stroke-width": 1 }, svg);
    if (sc.S != null) {
      const pts = [];
      for (let i = 0; i <= 40; i++) { const n = Math.pow(2, i / 10); pts.push(`${i ? "L" : "M"}${X(n)},${Y(sc.S + sc.P / n)}`); }
      el("path", { d: pts.join(""), fill: "none", stroke: css("--accent"), "stroke-width": 1.5 }, svg);
    }
    for (const n of sc.ns) if (sc.walls[n] > 0) el("circle", { cx: X(n), cy: Y(sc.walls[n]), r: 3.5, fill: color, stroke: css("--fg"), "stroke-width": 1 }, svg);
    return svg;
  }
  function niceLogTicks(lo, hi) {
    const out = [];
    for (let e = Math.floor(Math.log10(lo)); e <= Math.ceil(Math.log10(hi)); e++) {
      for (const m of [1, 2, 5]) { const v = m * Math.pow(10, e); if (v >= lo && v <= hi) out.push(v); }
    }
    return out.length > 5 ? out.filter((v) => String(v).startsWith("1")) : out;
  }

  function drawPanel() {
    const s = SYSK[selected];
    const p = $("panel");
    p.innerHTML = "";
    h("h3", {}, p, `<span class="swatch" style="background:${s.c}"></span>${esc(s.label)}`);
    h("p", { class: "note" }, p, esc(s.what));
    const st = s.st;
    const dl = h("dl", { class: "stats" }, p);
    const rows = [
      [s.offpath ? "Wall, beside" : "Wall, critical path", sec(st.wall)],
      ["Share of the run", s.offpath ? "off the path" : pct(st.wall / crit)],
      ["CPU, work + kernel", sec(st.cpu)],
      ["Cores working", fmt(st.busy, 2) + " of 8"],
      ["Cores spinning", fmt(st.wall ? st.spin / st.wall : 0, 2)],
      ["Kernel CPU", sec(st.kern)],
      ["Spans (steps, calls)", s.self ? "n/a" : fmt(st.count)],
      ["Idle core time", sec(st.headroom)],
    ];
    for (const [k, v] of rows) h("div", {}, dl, `<dt>${k}</dt><dd>${v}</dd>`);
    h("p", { class: "note" }, p, "Idle core time is the wall time a perfect spread of the same CPU work over 8 cores would save: wall minus CPU / 8.");
    const sc = scalingFor(s);
    if (sc) {
      const fig = h("div", {}, p);
      const s1 = sc.walls[1], s8 = sc.walls[8];
      h("div", { class: "note" }, fig, `Thread scaling, ${F.scale_myr || 200} Myr runs: ${s1 && s8 ? `1 → 8 threads ${fmt(s1 / s8, 2)}×` : ""}${sc.S != null ? `, serial part ${pct(sc.S / (sc.S + sc.P), 0)} of the 1-thread time` : ""}. Dots measured, line T(n) = S + P / n fitted on 1 to 8, dashed ideal.`);
      miniScaling(fig, sc, s.c, 300, 140);
    }
    const ps = parts(s);
    if (ps.length) {
      const tw = h("div", { class: "tablewrap" }, p);
      const t = h("table", {}, tw);
      t.innerHTML = `<thead><tr><th>Part</th><th class="r">Spans</th><th class="r">Wall</th><th class="r">Cores</th></tr></thead>`;
      const tb = h("tbody", {}, t);
      for (const a of ps.slice(0, 14)) {
        const busy = a.wall ? (a.work + a.kern) / a.wall : 0;
        h("tr", {}, tb, `<td>${esc(nice(a.key))}</td><td class="r">${fmt(a.count)}</td><td class="r">${sec(a.wall)}</td><td class="r">${fmt(busy, 1)}</td>`);
      }
    }
    const fns = new Map();
    for (const n of st.nodes) collectFns(n, fns, s);
    const top = [...fns.entries()].sort((a, b) => b[1] - a[1]).slice(0, 8);
    if (top.length) {
      h("div", { class: "eyebrow" }, p, "Where its CPU goes (self time, perf)");
      const tw = h("div", { class: "tablewrap" }, p);
      const t = h("table", {}, tw);
      const tb = h("tbody", {}, t);
      for (const [k, v] of top) h("tr", {}, tb, `<td class="mono" style="word-break:break-all">${esc(k)}</td><td class="r">${sec(v)}</td>`);
    }
    const o = (F.opportunities || []).filter((x) => (x.systems || []).includes(s.k));
    if (o.length) {
      h("div", { class: "eyebrow" }, p, "Opportunities here");
      const ul = h("ul", { class: "plain" }, p);
      for (const x of o) h("li", {}, ul, `#${x.rank} ${esc(x.title)}: ${gainText(x)}`);
    }
  }
  function collectFns(root, acc, s) {
    if (s.self) {
      for (const [k, v] of root.fns || []) acc.set(k, (acc.get(k) || 0) + v);
      return;
    }
    const d = root.path.length;
    for (const n of full.nodes) {
      if (n.path.length < d || n.lane !== root.lane && n.path.length === d) continue;
      let ok = true;
      for (let i = 0; i < d; i++) if (n.path[i] !== root.path[i]) { ok = false; break; }
      if (!ok) continue;
      for (const [k, v] of n.fns || []) acc.set(k, (acc.get(k) || 0) + v);
    }
  }

  function crossbars() {
    const box = $("crossbars");
    const cats = full.categories || {};
    const total = Object.values(cats).reduce((a, b) => a + b, 0);
    const list = Object.entries(cats).filter(([k]) => k !== "compute").sort((a, b) => b[1] - a[1]);
    const max = Math.max(...list.map((x) => x[1]), 1);
    h("div", { class: "note" }, box, `Of ${sec(total)} of sampled CPU, ${sec(cats.compute || 0)} (${pct((cats.compute || 0) / total)}) is the stages' own computing; the rest:`);
    for (const [k, v] of list) {
      const row = h("div", { class: "crossbar" }, box);
      h("span", {}, row, esc(k));
      const bar = h("div", { class: "bar" }, row);
      const sp = h("span", {}, bar);
      sp.style.width = (v / max * 100) + "%";
      if (k === "rayon wait") sp.style.background = "var(--spin)";
      else if (/fault|munmap|file|mmap|futex|kernel/.test(k)) sp.style.background = "var(--kernel)";
      h("span", { class: "num", style: "text-align:right" }, row, `${sec(v)}`);
    }
  }

  // ------------------------------------------------------------------------------------------
  // timeline
  const PHASEC = { "create": "#a9b4ae", "coupled group": "#7fc64e", "deposits": "#67a599", "young": "#009270", "amplify": "#f9f97f", "finalize": "#812b92" };
  function drawTimeline() {
    const tl = full.timeline;
    const box = $("timeline");
    box.innerHTML = "";
    const W = 1000, H = 290, L = 44, Rm = 12, T = 46, B = 30;
    const n = tl.busy.length, bs = tl.bin_s, tmax = n * bs;
    const X = (t) => L + t / tmax * (W - L - Rm);
    const ymax = 8.5;
    const Y = (v) => T + (1 - v / ymax) * (H - T - B);
    const svg = el("svg", { viewBox: `0 0 ${W} ${H}`, width: "100%", style: "min-width:640px", role: "img", "aria-label": "Cores busy over the run" }, box);
    for (const p of full.phases) {
      if (p.b - p.a <= 0) continue;
      const c = PHASEC[p.label] || "#a9b4ae";
      el("rect", { x: X(p.a), y: 6, width: Math.max(1, X(p.b) - X(p.a)), height: 18, fill: c }, svg);
      if (X(p.b) - X(p.a) > 60) el("text", { x: X(p.a) + 4, y: 19, "font-size": 11, style: `fill:${textOn(c)}` }, svg, p.label);
      el("line", { x1: X(p.a), x2: X(p.a), y1: 24, y2: H - B, stroke: css("--rule"), "stroke-dasharray": "2 3" }, svg);
    }
    const ax = el("g", { class: "axis" }, svg);
    for (let v = 0; v <= 8; v += 2) {
      el("line", { x1: L, x2: W - Rm, y1: Y(v), y2: Y(v) }, ax);
      el("text", { x: L - 6, y: Y(v) + 4, "text-anchor": "end" }, ax, v);
    }
    const step = tmax > 1200 ? 200 : 100;
    for (let t = 0; t <= tmax; t += step) el("text", { x: X(t), y: H - 10, "text-anchor": "middle" }, ax, fmt(t) + " s");
    el("text", { x: 2, y: T - 10, class: "axis" }, svg).textContent = "cores";
    // stacked areas from perf: work, kernel, spin
    const layers = [["work", "--work"], ["kernel", "--kernel"], ["spin", "--spin"]];
    const base = new Array(n).fill(0);
    for (const [k, tok] of layers) {
      const arr = tl[k] || [];
      if (!arr.length) continue;
      let top = "", bot = "";
      for (let i = 0; i < n; i++) {
        const y0 = base[i], y1 = base[i] + (arr[i] || 0);
        top += `${i ? "L" : "M"}${X(i * bs)},${Y(y1)} L${X((i + 1) * bs)},${Y(y1)} `;
        base[i] = y1;
        bot = `L${X((i + 1) * bs)},${Y(y0)} L${X(i * bs)},${Y(y0)} ` + bot;
      }
      el("path", { d: top + bot + "Z", fill: css(tok), stroke: "none" }, svg);
    }
    let line = "";
    for (let i = 0; i < n; i++) line += `${i ? "L" : "M"}${X((i + 0.5) * bs)},${Y(tl.busy[i])} `;
    el("path", { d: line, fill: "none", stroke: css("--busy-line"), "stroke-width": 0.8, opacity: 0.8 }, svg);
    el("line", { x1: L, x2: W - Rm, y1: Y(8), y2: Y(8), stroke: css("--accent"), "stroke-dasharray": "5 4" }, svg);
    el("text", { x: W - Rm, y: Y(8) - 4, "text-anchor": "end", "font-size": 11, style: "fill:var(--accent)" }, svg, "8 cores");
    const hit = el("rect", { x: L, y: T, width: W - L - Rm, height: H - T - B, fill: "transparent" }, svg);
    hit.addEventListener("mousemove", (ev) => {
      const r = svg.getBoundingClientRect();
      const t = ((ev.clientX - r.left) / r.width * W - L) / (W - L - Rm) * tmax;
      const i = Math.max(0, Math.min(n - 1, Math.floor(t / bs)));
      const ph = full.phases.find((p) => t >= p.a && t < p.b);
      showTip(ev, `${fmt(i * bs)} to ${fmt((i + 1) * bs)} s${ph ? " · " + ph.label : ""}<br>work ${fmt(tl.work[i] || 0, 2)} · kernel ${fmt(tl.kernel[i] || 0, 2)} · spin ${fmt(tl.spin[i] || 0, 2)} cores (perf)<br>busy ${fmt(tl.busy[i], 2)} cores (/proc)`);
    });
    hit.addEventListener("mouseleave", hideTip);
    const lg = $("tl-legend");
    lg.innerHTML = `<span><i style="background:var(--work)"></i>work (perf)</span><span><i style="background:var(--kernel)"></i>kernel (perf)</span><span><i style="background:var(--spin)"></i>rayon wait, spinning (perf)</span><span><i style="background:var(--busy-line);height:2px"></i>busy, all threads (/proc)</span>`;
    // threads
    const tb = $("threads");
    tb.innerHTML = "";
    const rows = tl.threads;
    const rowH = 15, lab = 170;
    let wk = 0;
    const rowName = (r) => (/rayon/.test(r.name) ? `worker ${++wk}` : "main thread") + (r.cpu_s ? ` · ${fmt(r.cpu_s)} s` : "");
    const cv = h("canvas", { role: "img", "aria-label": "Per-thread busy heat strips" }, tb);
    const cw = Math.max(640, tb.clientWidth || 900);
    const dpr = window.devicePixelRatio || 1;
    cv.width = cw * dpr; cv.height = (rows.length * rowH + 4) * dpr;
    cv.style.width = cw + "px"; cv.style.height = (rows.length * rowH + 4) + "px";
    const g = cv.getContext("2d");
    g.scale(dpr, dpr);
    g.font = `11px ${css("--f-mono") || "monospace"}`;
    const fg = css("--fg"), sunk = css("--sunk"), work = css("--work");
    rows.forEach((r, j) => {
      const y = j * rowH;
      g.fillStyle = fg;
      g.fillText(rowName(r), 0, y + 11);
      g.fillStyle = sunk;
      g.fillRect(lab, y + 1, cw - lab, rowH - 2);
      const bw = (cw - lab) / n;
      g.fillStyle = work;
      for (let i = 0; i < n; i++) {
        const v = Math.min(1, r.busy[i] || 0);
        if (v <= 0.01) continue;
        g.globalAlpha = 0.15 + 0.85 * v;
        g.fillRect(lab + i * bw, y + 1, Math.max(1, bw), rowH - 2);
      }
      g.globalAlpha = 1;
    });
  }

  // ------------------------------------------------------------------------------------------
  // steps
  const STEPC = { plates: "#f04028", climate: "#34b2c9", surface: "#7fc64e", volcanoes: "#fd9a52", impacts: "#ffe619", structure: "#cb8c37",
    "wait for structure beside": "#e3c08a", "keyframes and events": "#b3e1b6", "session other": "#9aa8c9" };
  function drawSteps() {
    const st = full.steps;
    const box = $("steps");
    box.innerHTML = "";
    if (!st || !st.rows.length) return;
    const cols = st.cols.slice(1).filter((c) => c !== "structure beside");
    const idx = Object.fromEntries(st.cols.map((c, i) => [c, i]));
    const rows = st.rows;
    const tot = rows.map((r) => cols.reduce((a, c) => a + r[idx[c]], 0));
    const sorted = tot.slice().sort((a, b) => a - b);
    const ymax = sorted[Math.floor(sorted.length * 0.995)] * 1.1 || 1;
    const W = Math.max(640, box.clientWidth || 900), H = 260, L = 44, Rm = 8, T = 10, B = 28;
    const dpr = window.devicePixelRatio || 1;
    const cv = h("canvas", { role: "img", "aria-label": "Seconds per plate step, stacked by system" }, box);
    cv.width = W * dpr; cv.height = H * dpr; cv.style.width = W + "px"; cv.style.height = H + "px";
    const g = cv.getContext("2d");
    g.scale(dpr, dpr);
    const Y = (v) => T + (1 - v / ymax) * (H - T - B);
    const bw = (W - L - Rm) / rows.length;
    g.font = `11px ${css("--f-mono") || "monospace"}`;
    g.strokeStyle = css("--rule"); g.fillStyle = css("--muted");
    const yt = ymax > 6 ? 2 : ymax > 3 ? 1 : 0.5;
    for (let v = 0; v <= ymax; v += yt) {
      g.beginPath(); g.moveTo(L, Y(v)); g.lineTo(W - Rm, Y(v)); g.stroke();
      g.fillText(fmt(v, yt < 1 ? 1 : 0) + " s", 2, Y(v) + 4);
    }
    rows.forEach((r, i) => {
      let acc = 0;
      for (const c of cols) {
        const v = r[idx[c]];
        if (v <= 0) continue;
        g.fillStyle = STEPC[c] || "#999";
        const y0 = Y(Math.min(ymax, acc)), y1 = Y(Math.min(ymax, acc + v));
        g.fillRect(L + i * bw, y1, Math.max(1, bw - (bw > 3 ? 0.5 : 0)), y0 - y1);
        acc += v;
      }
    });
    g.fillStyle = css("--muted");
    for (let k = 0; k <= rows.length; k += 100) {
      g.textAlign = k === 0 ? "left" : k === rows.length ? "right" : "center";
      g.fillText(fmt(k * 2) + " Myr", L + k * bw, H - 8);
    }
    g.textAlign = "right";
    g.fillStyle = css("--fg");
    rows.forEach((r, i) => { if (tot[i] > ymax) g.fillText(`step ${i + 1}: ${fmt(tot[i], 1)} s, cut off ▸`, L + i * bw - 4, T + 12); });
    g.textAlign = "left";
    cv.addEventListener("mousemove", (ev) => {
      const rr = cv.getBoundingClientRect();
      const i = Math.floor((ev.clientX - rr.left - L) / bw);
      if (i < 0 || i >= rows.length) { hideTip(); return; }
      const r = rows[i];
      const lines = cols.filter((c) => r[idx[c]] > 0.0005).map((c) => `${c} ${fmt(r[idx[c]], 3)} s`).join("<br>");
      showTip(ev, `step ${i + 1} (${fmt((i + 1) * 2)} Myr), ${fmt(tot[i], 2)} s<br>${lines}<br><span class="muted">structure beside ${fmt(r[idx["structure beside"]] || 0, 3)} s (off the path)</span>`);
    });
    cv.addEventListener("mouseleave", hideTip);
    $("steps-legend").innerHTML = cols.map((c) => `<span><i style="background:${STEPC[c]}"></i>${esc(c)}</span>`).join("");
  }

  // ------------------------------------------------------------------------------------------
  // icicle
  const ICON = full.icicle;
  function colorOfSpan(name) {
    const m = { "cli:create": "create", "cli:finalize": "finalize", "cli:report": "process", "process:start-up": "process", "process:tear-down": "process",
      "session:join": "plates", "beside:structure": "beside", "session:after_step": "keyframes", "session:make_room": "memory", "session:trim_history": "memory",
      "session:detach": "memory", "session:attach": "memory", "outside spans": "process" };
    if (m[name]) return SYSK[m[name]].c;
    if (name.startsWith("stage:")) { const k = name.slice(6); return (SYSK[k] || SYSK.driver).c; }
    if (name.startsWith("finish:")) return SYSK.finish.c;
    return null;
  }
  let icRoot = ICON, icPath = [ICON];
  function drawIcicle() {
    const box = $("icicle");
    box.innerHTML = "";
    const W = box.clientWidth || 900;
    const total = ICON.v;
    const cells = [];
    let maxDepth = 0;
    function walk(n, x, w, depth, color) {
      if (w < 1.5 || depth > 40) return;
      const c = n.s ? (colorOfSpan(n.n) || color) : color;
      cells.push({ n, x, w, depth, color: c });
      maxDepth = Math.max(maxDepth, depth);
      let cx = x;
      for (const k of n.c || []) { const kw = w * k.v / n.v; walk(k, cx, kw, depth + 1, c); cx += kw; }
    }
    walk(icRoot, 0, W, 0, SYSK.driver.c);
    box.style.height = ((maxDepth + 1) * 21 + 2) + "px";
    const frag = document.createDocumentFragment();
    for (const c of cells) {
      const d = document.createElement("div");
      d.className = "cell" + (c.n.s ? " span" : "");
      d.style.left = c.x + "px"; d.style.width = c.w + "px"; d.style.top = (c.depth * 21) + "px";
      if (c.n.s || c.depth === 0) { d.style.background = c.color; d.style.color = textOn(c.color); }
      else { d.style.background = `color-mix(in srgb, ${c.color} 38%, var(--surface))`; d.style.color = "var(--fg)"; }
      d.textContent = c.w > 30 ? (c.n.s ? nice(c.n.n) : c.n.n) : "";
      d.tabIndex = c.w > 30 ? 0 : -1;
      d.addEventListener("mousemove", (ev) => showTip(ev, `<b>${esc(c.n.n)}</b><br>${sec(c.n.v / hz)} CPU · ${pct(c.n.v / total)} of the run`));
      d.addEventListener("mouseleave", hideTip);
      const zoom = () => {
        if (c.n === icRoot && icPath.length > 1) { icPath.pop(); icRoot = icPath[icPath.length - 1]; }
        else if (c.n !== icRoot) { icPath = pathTo(c.n); icRoot = c.n; }
        drawIcicle();
      };
      d.addEventListener("click", zoom);
      d.addEventListener("keydown", (ev) => { if (ev.key === "Enter") zoom(); });
      frag.appendChild(d);
    }
    box.appendChild(frag);
    const cr = $("crumbs");
    cr.innerHTML = "";
    icPath.forEach((n, i) => {
      const b = h("button", { type: "button" }, cr);
      b.textContent = (i === 0 ? "whole run" : (n.s ? nice(n.n) : n.n)) + ` (${sec(n.v / hz)})`;
      b.addEventListener("click", () => { icPath = icPath.slice(0, i + 1); icRoot = n; drawIcicle(); });
    });
  }
  function pathTo(target) {
    const out = [];
    (function f(n) {
      out.push(n);
      if (n === target) return true;
      for (const k of n.c || []) if (f(k)) return true;
      out.pop();
      return false;
    })(ICON);
    return out;
  }

  // ------------------------------------------------------------------------------------------
  // scaling
  function drawScaling() {
    const box = $("scaling");
    box.innerHTML = "";
    if (!D.scaling) { box.textContent = "No scaling runs in this data."; return; }
    const list = SYS.map((s) => ({ s, sc: scalingFor(s) })).filter((x) => x.sc && (x.sc.walls[8] || 0) >= 1)
      .sort((a, b) => (b.sc.walls[8] || 0) - (a.sc.walls[8] || 0));
    const total = { walls: {}, ns: [] };
    for (const [n, r] of Object.entries(D.scaling.runs)) { total.walls[n] = r.total_s; }
    total.ns = Object.keys(total.walls).map(Number).sort((a, b) => a - b);
    const tf = fit(total.ns.filter((n) => n <= 8).map((n) => [1 / n, total.walls[n]]));
    total.S = tf ? tf[0] : null; total.P = tf ? tf[1] : null;
    const items = [{ s: { label: "Whole run", c: "#a9b4ae" }, sc: total }].concat(list);
    for (const { s, sc } of items) {
      const fig = h("figure", {}, box);
      const sp = sc.walls[1] && sc.walls[8] ? sc.walls[1] / sc.walls[8] : null;
      h("figcaption", {}, fig, `<b>${esc(s.label)}</b><span class="muted num">${sp ? fmt(sp, 2) + "× at 8" : ""}</span>`);
      miniScaling(fig, sc, s.c);
      const sh = sc.S != null && sc.S + sc.P > 0 ? sc.S / (sc.S + sc.P) : null;
      h("div", { class: "note" }, fig, `${sec(sc.walls[8] || 0)} at 8 threads${sc.walls[16] ? `, ${sec(sc.walls[16])} at 16` : ""}${sh != null ? ` · serial ${pct(sh, 0)}` : ""}`);
    }
    const tw = $("scaling-table");
    tw.innerHTML = "";
    const t = h("table", {}, tw);
    t.innerHTML = `<thead><tr><th>System</th><th class="r">1 thread</th><th class="r">2</th><th class="r">4</th><th class="r">8</th><th class="r">16</th><th class="r">S (serial)</th><th class="r">P (parallel)</th><th class="r">16, fit</th></tr></thead>`;
    const tb = h("tbody", {}, t);
    for (const { s, sc } of items) {
      const c = (n) => sc.walls[n] != null ? fmt(sc.walls[n], sc.walls[n] < 10 ? 1 : 0) : "n/a";
      h("tr", {}, tb, `<td>${esc(s.label)}</td><td class="r">${c(1)}</td><td class="r">${c(2)}</td><td class="r">${c(4)}</td><td class="r">${c(8)}</td><td class="r">${c(16)}</td><td class="r">${sc.S != null ? fmt(sc.S, 1) : "n/a"}</td><td class="r">${sc.P != null ? fmt(sc.P, 1) : "n/a"}</td><td class="r">${sc.S != null ? fmt(sc.S + sc.P / 16, 1) : "n/a"}</td></tr>`);
    }
    h("caption", { style: "caption-side:bottom;text-align:left;padding-top:0.4rem", class: "note" }, t, "Seconds of wall time in each system at each thread count (critical path; structure beside is its own beside time). S and P from a least-squares fit of T(n) = S + P / n over 1, 2, 4 and 8 threads; the 16-thread run is the check.");
    // the mix check: each stage's share of the 200 Myr run against the full run, both at 8 threads
    const r8 = D.scaling.runs["8"], fullStages = summ.log && summ.log.stages_s;
    if (r8 && r8.stages_s && fullStages) {
      const a = Object.values(fullStages).reduce((x, y) => x + y, 0), b = Object.values(r8.stages_s).reduce((x, y) => x + y, 0);
      const mw = h("div", { class: "tablewrap" }, tw);
      const mt = h("table", { style: "margin-top:1rem" }, mw);
      mt.innerHTML = `<thead><tr><th>Stage</th><th class="r">1,000 Myr, s</th><th class="r">share</th><th class="r">${F.scale_myr || 200} Myr, s</th><th class="r">share</th><th class="r">× 1,000 / ${F.scale_myr || 200}</th></tr></thead>`;
      const mb = h("tbody", {}, mt);
      for (const k of Object.keys(fullStages)) {
        const x = fullStages[k], y = r8.stages_s[k] || 0;
        h("tr", {}, mb, `<td>${esc(k)}</td><td class="r">${fmt(x, 1)}</td><td class="r">${pct(x / a)}</td><td class="r">${fmt(y, 1)}</td><td class="r">${pct(y / b)}</td><td class="r">${y ? fmt(x / y, 2) : "n/a"}</td></tr>`);
      }
      h("caption", { style: "caption-side:bottom;text-align:left;padding-top:0.4rem", class: "note" }, mt, F.mix_note || "The mix check: each stage's share of the shorter scaling run against the full run, both at 8 threads on set A. The coupled group's stages scale with the run's length; deposits, young, amplify and finalize do not, so the shorter run over-weights them.");
    }
  }

  // ------------------------------------------------------------------------------------------
  // opportunities
  function gainText(x) {
    const r = x.gain_lo === x.gain_hi ? `${fmt(x.gain_lo)} s` : `${fmt(x.gain_lo)} to ${fmt(x.gain_hi)} s`;
    return `${r} <span class="est">${x.measured ? "measured" : "est."}</span>`;
  }
  function drawOpps() {
    const box = $("opps");
    box.innerHTML = "";
    const list = F.opportunities || [];
    $("opps-text").innerHTML = F.opps_intro || "";
    if (!list.length) return;
    const t = h("table", { class: "opps" }, box);
    t.innerHTML = `<thead><tr><th class="r">#</th><th>Change</th><th>Evidence</th><th class="r gain">Gain at 8 threads</th><th>Same worlds?</th><th>Effort</th></tr></thead>`;
    const tb = h("tbody", {}, t);
    let lo = 0, hi = 0;
    for (const x of list) {
      const cls = x.identical === "yes" ? "yes" : x.identical === "no" ? "no" : "verify";
      const lab = x.identical === "yes" ? "identical" : x.identical === "no" ? "changes worlds" : "check";
      if (x.identical === "yes" && !x.exclude_sum) { lo += x.gain_lo; hi += x.gain_hi; }
      h("tr", {}, tb, `<td class="r">${x.rank}</td><td><b>${esc(x.title)}</b><div class="note">${x.change || ""}</div></td><td><div class="note" style="color:var(--fg)">${x.evidence || ""}</div></td><td class="r">${gainText(x)}</td><td><span class="pill ${cls}">${lab}</span><div class="note">${x.identical_note || ""}</div></td><td>${esc(x.effort || "")}<div class="note">${x.effort_note || ""}</div></td>`);
    }
    const gap = headWall - gate;
    h("tr", { class: "sum" }, tb, `<td></td><td colspan="2">Identical-output changes together${F.sum_note ? `<div class="note">${F.sum_note}</div>` : ""}</td><td class="r">${fmt(lo)} to ${fmt(hi)} s <span class="est">est.</span></td><td colspan="2">against the ${sec(gap)} between the run and the gate: ${pct(lo / gap, 0)} to ${pct(hi / gap, 0)} of it</td>`);
    // builds
    const bb = $("builds");
    bb.innerHTML = "";
    if (D.builds) {
      h("h3", {}, bb, "Build settings, measured");
      if (F.builds_intro) h("p", {}, bb, F.builds_intro);
      const groups = {};
      for (const [k, v] of Object.entries(D.builds)) {
        const m = k.match(/^(?:e100-)?(\w+)-(\d)$/);
        const g = m ? m[1] : k;
        (groups[g] = groups[g] || []).push(v);
      }
      const plainRuns = groups.plain || [];
      const meanT = (rs) => rs.reduce((a, r) => a + ((r.log && r.log.total_s) || r.total_s || 0), 0) / (rs.length || 1);
      const base = meanT(plainRuns);
      const tw2 = h("div", { class: "tablewrap" }, bb);
      const t2 = h("table", {}, tw2);
      t2.innerHTML = `<thead><tr><th>Build</th><th class="r">Runs (s)</th><th class="r">Mean</th><th class="r">vs plain</th><th>Same worlds?</th></tr></thead>`;
      const tb2 = h("tbody", {}, t2);
      for (const [g, rs] of Object.entries(groups)) {
        const m = meanT(rs);
        const sameNote = (F.build_same || {})[g] || "";
        h("tr", {}, tb2, `<td>${esc((F.build_labels || {})[g] || g)}</td><td class="r">${rs.map((r) => fmt((r.log && r.log.total_s) || r.total_s, 1)).join(", ")}</td><td class="r">${fmt(m, 1)}</td><td class="r">${g === "plain" ? "" : (m < base ? "−" : "+") + pct(Math.abs(m / base - 1))}</td><td>${sameNote}</td></tr>`);
      }
    }
  }

  // ------------------------------------------------------------------------------------------
  // method, reconcile
  function drawMethod() {
    for (const m of F.method || []) h("li", {}, $("method"), m);
    for (const m of F.not_measured || []) h("li", {}, $("notmeasured"), m);
    const r = full.reconcile;
    const box = $("reconcile");
    const t = h("table", {}, box);
    t.innerHTML = `<thead><tr><th>Measure</th><th class="r">Value</th><th>Source</th></tr></thead>`;
    const tb = h("tbody", {}, t);
    const rows = [
      ["Wall, the whole process", sec(r.wall_runner_s, 1), "the runner's clock around the process (perf start included)"],
      ["Wall, the bench's total", sec(r.total_bench_s, 1), "deeptime bench: create to finalize"],
      ["Wall, the plain build alone", plainWall ? sec(plainWall, 1) : "n/a", "the same run without frame pointers, spans or perf"],
      ["CPU, user + system", sec((r.user_s || 0) + (r.system_s || 0), 0), "/usr/bin/time -v"],
      ["CPU, all threads", sec(r.threads_tsv_cpu_s, 0), "/proc/<pid>/task/*/stat at the last sample"],
      ["CPU, the process clock", sec(r.trace_cpu_at_end_s, 0), "CLOCK_PROCESS_CPUTIME_ID at the last span"],
      ["CPU, perf samples", sec(r.perf_cpu_s, 0), `${fmt(full.perf.samples)} samples at ${hz} Hz`],
      ["  of which work", sec(r.perf_work_s, 0), "a frame of ours or a library below the sample"],
      ["  of which kernel", sec(r.perf_kernel_s, 0), "the sample in the kernel on behalf of our code"],
      ["  of which rayon wait", sec(r.perf_spin_s, 0), "a worker in rayon's wait, steal or sleep loop"],
    ];
    for (const [a, b, c] of rows) h("tr", {}, tb, `<td>${esc(a)}</td><td class="r">${b}</td><td class="note">${esc(c)}</td>`);
    if (summ.log && summ.log.stages_s) {
      const t2 = h("table", { style: "margin-top:1rem" }, box);
      t2.innerHTML = `<thead><tr><th>Stage</th><th class="r">Bench timer, profiled</th><th class="r">Span wall</th><th class="r">Bench timer, plain</th><th class="r">perf CPU</th></tr></thead>`;
      const tb2 = h("tbody", {}, t2);
      const map = { plates: ["plates", "beside"], climate: ["climate"], surface: ["surface"], volcanoes: ["volcanoes"], impacts: ["impacts"], structure: ["structure", "structwait"], deposits: ["deposits"], young: ["young"], amplify: ["amplify"], finalize: ["finalize"] };
      for (const [k, v] of Object.entries(summ.log.stages_s)) {
        const ss = (map[k] || []).map((x) => SYSK[x]).filter(Boolean);
        const spanW = ss.filter((s) => !s.offpath).reduce((a, s) => a + s.st.wall, 0);
        const cpu = ss.reduce((a, s) => a + s.st.cpu, 0);
        const pv = plain && plain.log && plain.log.stages_s ? plain.log.stages_s[k] : null;
        h("tr", {}, tb2, `<td>${esc(k)}</td><td class="r">${sec(v, 1)}</td><td class="r">${sec(spanW, 1)}</td><td class="r">${pv != null ? sec(pv, 1) : "n/a"}</td><td class="r">${sec(cpu, 0)}</td>`);
      }
      h("caption", { style: "caption-side:bottom;text-align:left;padding-top:0.4rem", class: "note" }, t2, "The bench's plates timer includes the beside structure steps' join; the span wall puts the wait for structure under structure instead, so plates and structure move between the two columns while their sum agrees.");
    }
  }

  function texts() {
    for (const [id, key] of [["timeline-text", "timeline_text"], ["steps-text", "steps_text"], ["icicle-text", "icicle_text"], ["scaling-text", "scaling_text"]]) {
      if (F[key]) $(id).innerHTML = F[key];
    }
  }

  function all() {
    drawMap(); drawPanel(); drawTimeline(); drawSteps(); drawIcicle(); drawScaling();
  }
  header(); crossbars(); texts(); drawOpps(); drawMethod(); all();
  fitSeg();
  if (document.fonts && document.fonts.ready) document.fonts.ready.then(() => { drawTimeline(); drawSteps(); fitSeg(); });
  let rt = null;
  window.addEventListener("resize", () => { clearTimeout(rt); rt = setTimeout(() => { drawTimeline(); drawSteps(); drawIcicle(); fitSeg(); }, 150); });
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  if (mq.addEventListener) mq.addEventListener("change", all);
  new MutationObserver(all).observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
})();
