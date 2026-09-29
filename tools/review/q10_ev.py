#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Plan Q10 review pages ("Excavated Variants in MIC?"), from what tools/review/q10-ev.sh and
tools/client/ev-assets.sh left in review/out/q10/ (pull it from the Mac first).

    python3 tools/review/q10_ev.py merge     # review/out/q10-map-src: one BlueMap app, four maps
    python3 tools/review/q10_ev.py shots     # headless Chromium screenshots of the maps
    python3 tools/review/q10_ev.py page      # review/out/q10-page-src: the review page

then tools/review/publish.sh q10-ev-map review/out/q10-map-src and
tools/review/publish.sh q10-ev review/out/q10-page-src (two private spaces: the four maps alone take
198 of a space's 200 files). Standard library only (Pillow, if present, trims screenshots).
"""
import base64
import collections
import functools
import glob
import html
import http.server
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import threading

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
Q = os.path.join(REPO, "review", "out", "q10")
MAP = os.path.join(REPO, "review", "out", "q10-map-src")
PAGE = os.path.join(REPO, "review", "out", "q10-page-src")
SHOTS = os.path.join(Q, "shots")
SPACES = os.path.join(REPO, "tools", "review", "spaces.json")
PROFILES = {"mic": ("plain", "without EV"), "mic-ev": ("ev", "with EV")}

# The render boxes (name, minX, maxX, minZ, maxZ, cutY), read from q10-ev.sh, and what each shows.
WHAT = {"porphyry": "Porphyry Cu-Mo-Au 2 (y 127–137) with its supergene copper blanket 3 (y 127–131), porphyry 10 and blanket 1",
        "placer": "Gold placer 8 in gravel (y 161–165)",
        "spawn": "Coal seam 29 (y 66–68) and banded iron formation 37 (y 64–68), next to spawn"}
BOXES = [(n, int(a), int(b), int(c), int(d), int(e), WHAT.get(n, n)) for n, a, b, c, d, e in
         (line.split() for line in re.search(r'BOXES="\$\{Q10_BOXES:-(.*?)\}"', open(os.path.join(
             REPO, "tools", "review", "q10-ev.sh")).read(), re.S).group(1).strip().splitlines())]
SECTIONS = [("porphyry", "z = −1002 through porphyry 2 and blanket 3, x −5630…−5535"),
            ("porphyry10", "x = −5554 through porphyry 10, z −1060…−960"),
            ("placer", "z = −1086 through the placer, x −5640…−5540"),
            ("spawn", "x = −16 through the coal seam and the BIF, z −2174…−2111")]
CLAIMED = {"coal", "copper", "iron", "gold", "zinc", "lead", "silver", "molybdenum"}


def load(path, default=None):
    try:
        with open(path) as f:
            return json.load(f)
    except (OSError, ValueError):
        return default


# ---------------------------------------------------------------------------------------------
# merge: one BlueMap web app holding both worlds' maps.
# ---------------------------------------------------------------------------------------------
def merge():
    shutil.rmtree(MAP, ignore_errors=True)
    base = os.path.join(Q, "bluemap-mic", "web")
    shutil.copytree(base, MAP)
    shutil.copytree(os.path.join(Q, "bluemap-mic-ev", "web", "maps"), os.path.join(MAP, "maps"), dirs_exist_ok=True)
    maps = sorted(os.listdir(os.path.join(MAP, "maps")),
                  key=lambda m: load(os.path.join(MAP, "maps", m, "settings.json"), {}).get("sorting", 0))
    s = load(os.path.join(MAP, "settings.json"))
    s["maps"] = maps
    json.dump(s, open(os.path.join(MAP, "settings.json"), "w"))
    # 200 files per space: the web app manifest and its two icons are not needed for a review page.
    for f in glob.glob(os.path.join(MAP, "assets", "logoCircle*.png")) + glob.glob(os.path.join(MAP, "assets", "manifest-*.webmanifest")):
        os.remove(f)
    idx = open(os.path.join(MAP, "index.html")).read()
    idx = re.sub(r'\s*<link rel="manifest"[^>]*>', "", idx)
    idx = re.sub(r"<title>.*?</title>", "<title>Deep Time — Q10 maps: MIC without and with EV</title>", idx)
    desc = ("BlueMap renders of the same Deep Time deposits in the MIC pack, without Excavated Variants and with it: "
            "surface and a cut at the ore level for three sites.")
    idx = idx.replace("<head>", f'<head><meta name="description" content="{html.escape(desc)}">', 1)
    open(os.path.join(MAP, "index.html"), "w").write(idx)
    files = sum(len(fs) for _, _, fs in os.walk(MAP))
    size = sum(os.path.getsize(os.path.join(r, f)) for r, _, fs in os.walk(MAP) for f in fs)
    big = max((os.path.getsize(os.path.join(r, f)), f) for r, _, fs in os.walk(MAP) for f in fs)
    print(f"merge: {MAP}: maps {maps}; {files} files, {size / 1e6:.1f} MB, largest {big[1]} {big[0] / 1e6:.1f} MB")
    if files > 199 or big[0] > 10e6:
        sys.exit("merge: over the page host's limits (200 files per space, 10 MB per file)")


# ---------------------------------------------------------------------------------------------
# shots: the same views of both worlds, from the merged app, in headless Chromium over HTTP.
# ---------------------------------------------------------------------------------------------
def views():
    """(file stem, map id, hash) for every box, map kind and world."""
    out = []
    for name, x0, x1, z0, z1, cut, _ in BOXES:
        cx, cz = (x0 + x1) / 2, (z0 + z1) / 2
        size = max(x1 - x0, z1 - z0)
        for kind in ("planet", "underground"):
            for prof, (prefix, _) in PROFILES.items():
                mid = f"{prefix}_{kind}"
                # Surface: an oblique view of the whole box. Cut: straight down (flat), so the layer at
                # the cut height fills the frame the same way in both worlds.
                cam = (f"{cx:.0f}:{cut + 30}:{cz:.0f}:{size * 2.2:.0f}:0.5:0.85:0:0:perspective" if kind == "planet"
                       else f"{cx:.0f}:{cut}:{cz:.0f}:{size * 0.5:.0f}:0:0:0:1:flat")
                out.append((f"{name}-{kind}-{prefix}", mid, f"#{mid}:{cam}"))
    return out


def chromium():
    for c in ("chromium", "chromium-browser", "google-chrome"):
        if shutil.which(c):
            return [c]
    return ["nix", "shell", "nixpkgs#chromium", "-c", "chromium"]


def trim(png):
    """Cut a screenshot down to the rendered box: drop BlueMap's buttons and the black void around it."""
    try:
        from PIL import Image
    except ImportError:
        return
    im = Image.open(png).convert("RGB")
    w, h = im.size
    body = im.crop((0, 48, w, h - 70))
    mask = body.convert("L").point(lambda v: 255 if v > 24 else 0)
    box = mask.getbbox()
    if box:
        pad = 12
        box = (max(0, box[0] - pad), max(0, box[1] - pad), min(w, box[2] + pad), min(h - 118, box[3] + pad))
        body.crop(box).save(png)


def shots():
    os.makedirs(SHOTS, exist_ok=True)
    class Quiet(http.server.SimpleHTTPRequestHandler):
        def log_message(self, *a):
            pass
    handler = functools.partial(Quiet, directory=MAP)
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    port = srv.server_address[1]
    only = sys.argv[2:] if len(sys.argv) > 2 else None
    for stem, _, h in views():
        if only and not any(o in stem for o in only):
            continue
        png = os.path.join(SHOTS, stem + ".png")
        url = f"http://127.0.0.1:{port}/index.html{h}"
        cmd = chromium() + ["--headless=new", "--no-sandbox", "--hide-scrollbars", "--use-angle=swiftshader",
                            "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "--window-size=1280,860",
                            "--virtual-time-budget=15000", f"--screenshot={png}", url]
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=900)
            err = r.stderr[-300:]
        except subprocess.TimeoutExpired:
            err = "timed out"
        ok = os.path.exists(png)
        if ok:
            trim(png)
        print(f"shots: {stem}: {'ok' if ok else 'FAILED ' + err}")
    srv.shutdown()


# ---------------------------------------------------------------------------------------------
# page
# ---------------------------------------------------------------------------------------------
def report_rows(path):
    """{check name: actual} from a run.py report."""
    rows = {}
    if not os.path.exists(path):
        return rows
    for line in open(path):
        m = re.match(r"\| (.+?) \| (.*?) \| (.*?) \| (PASS|FAIL) \|", line)
        if m:
            rows[m.group(1)] = m.group(3)
    return rows


def num(v):
    try:
        return float(str(v).split()[0])
    except (ValueError, IndexError, TypeError):
        return None


def costs():
    """Boot samples and RSS per profile: the worlds run (first start, restart) and the boots runs."""
    out = {}
    for prof in PROFILES:
        r = report_rows(os.path.join(Q, prof, "report.md"))
        c = {"first_boot_s": num(r.get("boot: JVM start to Done ( (s)")),
             "first_rss": num(r.get("RSS at Done (MB)")),
             "peak_rss": num(r.get("peak RSS (MB)")),
             "gen_s": num(r.get("generation time (s)")),
             "boots": [], "rss": [], "done": []}
        for p in [os.path.join(Q, prof, "report.md")] + sorted(glob.glob(os.path.join(Q, prof, "report-boot-*.md"))):
            rr = report_rows(p)
            key = "restart boot: JVM start to Done ( (s)" if p.endswith("report.md") else "boot: JVM start to Done ( (s)"
            rk = "restart RSS at Done (MB)" if p.endswith("report.md") else "RSS at Done (MB)"
            dk = "restart boot: the server's Done (…s) figure" if p.endswith("report.md") else "boot: the server's Done (…s) figure"
            if num(rr.get(key)) is not None:
                c["boots"].append(num(rr.get(key)))
                c["rss"].append(num(rr.get(rk)))
                c["done"].append(num(rr.get(dk)))
        c["mods"] = int(open(os.path.join(Q, prof, "mod-count.txt")).read().strip() or 0) if os.path.exists(os.path.join(Q, prof, "mod-count.txt")) else None
        out[prof] = c
    return out


def stone_blocks():
    ev = load(os.path.join(Q, "ev-configs.json"), {"stones": {}, "ores": {}})
    return {k.replace("/", "_"): v for k, v in ev["stones"].items()}, ev


def pair_rows():
    """Per box and overall: (host, plain block, ev block) -> count, with the host exact where the EV
    world has a variant (its stone) and estimated otherwise; plus the estimate's agreement."""
    stone_block, _ = stone_blocks()
    rows = collections.Counter()
    agree = total = same_pos = diff_pos = 0
    per_box = {}
    for name, *_ in BOXES:
        d = load(os.path.join(Q, f"pairs-{name}.json"))
        if not d:
            continue
        per_box[name] = d
        diff_pos += d["ore_only_here"] + d["ore_only_in_other"]
        for p in d["pairs"]:
            here, there, host, n, stone = p["ore"], p["other"], p["host"], p["count"], p.get("ev_stone")
            exact = stone_block.get(stone) if there and there.startswith("excavated_variants:") else None
            if exact:
                total += n
                agree += n if exact == host else 0
            rows[(exact or host, here, there, bool(exact))] += n
            same_pos += n
    return rows, agree, total, diff_pos, per_box


def mat(block):
    if not block:
        return None
    path = block.split(":", 1)[1]
    if block.startswith("excavated_variants:"):
        parts = path.split("__")
        if len(parts) == 4:
            ore = parts[1][:-4] if parts[1].endswith("_ore") else parts[1]
            return ore.rsplit("_", 1)[-1]
    if path.startswith("raw_") and path.endswith("_block"):
        return path[4:-6]
    if not path.endswith("_ore"):
        return None
    base = path[:-4]
    for p in ("deepslate_", "nether_", "end_", "blackstone_", "stone_"):
        if base.startswith(p):
            base = base[len(p):]
    return base.rsplit("_", 1)[-1] if base else None


def is_ore(block):
    return mat(block) is not None


def tile(block, size=32):
    """An <img> of the block's texture (data URI), or a placeholder."""
    if not block:
        return ""
    p = os.path.join(Q, "tiles", block.replace(":", "__") + ".png")
    if not os.path.exists(p):
        return f'<span class="notile" title="{html.escape(block)}"></span>'
    b64 = base64.b64encode(open(p, "rb").read()).decode()
    return f'<img class="tile" width="{size}" height="{size}" alt="" title="{html.escape(block)}" src="data:image/png;base64,{b64}">'


def short(block):
    if not block:
        return "—"
    if block.startswith("excavated_variants:"):
        parts = block.split(":", 1)[1].split("__")
        if len(parts) == 4:
            return f"EV: {parts[1].replace('_ore', '')} in {parts[3]}"
    return block.replace("minecraft:", "")


def classes_of():
    """Block -> rock classes it stands for (the mic world's probe palette)."""
    pal = load(os.path.join(Q, "mic", "palette.json"), {})
    out = collections.defaultdict(list)
    for cls, states in (pal.get("classes") or {}).items():
        for s in states:
            out[s.split("[")[0]].append(cls)
    return out


def page():
    shutil.rmtree(PAGE, ignore_errors=True)
    os.makedirs(os.path.join(PAGE, "img"))
    spaces = load(SPACES, {})
    map_url = (spaces.get("q10-ev-map") or {}).get("url", "")
    stone_block, ev = stone_blocks()
    ev_stone_blocks = set(stone_block.values())
    cls = classes_of()
    rows, agree, total, diff_pos, per_box = pair_rows()
    cost = costs()
    summary = load(os.path.join(Q, "ev-assets", "summary.json"), {})
    esc = html.escape

    # --- images: 2D renders and screenshots (copied; the page links them) ----------------------
    def img(src, dest):
        if not os.path.exists(src):
            return None
        shutil.copy2(src, os.path.join(PAGE, "img", dest))
        return "img/" + dest

    # --- the per-host table, Deep Time's own ores only ------------------------------------------
    by_host = collections.defaultdict(lambda: {"plain": collections.Counter(), "ev": collections.Counter(), "exact": 0, "n": 0})
    other_ores = collections.Counter()
    changed = unchanged = 0
    for (host, here, there, exact), n in rows.items():
        m = mat(here) or mat(there)
        if m not in CLAIMED:
            if is_ore(here):
                other_ores[mat(here)] += n
            continue
        h = by_host[host]
        if is_ore(here):
            h["plain"][here] += n
        if is_ore(there):
            h["ev"][there] += n
        h["n"] += n
        h["exact"] += n if exact else 0
        if here != there:
            changed += n
        else:
            unchanged += n

    def counter_cell(c):
        return "".join(f'<div class="ob">{tile(b, 24)}<span>{esc(short(b))}</span><b>{n:,}</b></div>'
                       for b, n in c.most_common())

    host_rows = []
    for host, h in sorted(by_host.items(), key=lambda kv: -kv[1]["n"]):
        if host == "(none)":
            cover = "—"
        elif host in ev_stone_blocks:
            src = next((k for k, v in stone_block.items() if v == host), "")
            cover = "yes" + (" (Deep Time's stand-in)" if src.startswith("deeptime:") else "")
        else:
            cover = "<b>no</b>"
        classes = ", ".join(sorted(set(cls.get(host, []))))
        host_rows.append(
            f"<tr><td>{tile(host, 32)}</td><td><code>{esc(host)}</code><div class='t'>{esc(classes) or 'not a rock class'}</div></td>"
            f"<td class='c'>{cover}</td><td class='r'>{h['n']:,}</td><td>{counter_cell(h['plain'])}</td><td>{counter_cell(h['ev'])}</td></tr>")

    # --- EV coverage of the MIC rock classes -----------------------------------------------------
    cover_rows = []
    for block, cl in sorted(cls.items(), key=lambda kv: kv[0]):
        if block in ("minecraft:air",):
            continue
        yes = block in ev_stone_blocks
        cover_rows.append(f"<tr><td>{tile(block, 24)}</td><td><code>{esc(block)}</code></td><td>{esc(', '.join(sorted(set(cl))))}</td>"
                          f"<td class='c'>{'yes' if yes else '<b>no</b>'}</td></tr>")

    # --- the side-by-side renders -----------------------------------------------------------------
    def pairs_html(items):
        return "".join(
            f"<figure class='pair'><figcaption>{cap}</figcaption><div class='two'>"
            f"<div><div class='lab'>without EV</div>{f'<img src={a!r} alt={cap!r}>' if a else '<i>missing</i>'}</div>"
            f"<div><div class='lab'>with EV</div>{f'<img src={b!r} alt={cap!r}>' if b else '<i>missing</i>'}</div></div></figure>"
            for cap, a, b in items)

    shot_items = []
    for name, x0, x1, z0, z1, cut, what in BOXES:
        for kind, label in (("planet", "surface"), ("underground", f"cut at y {cut}: only blocks at or below it")):
            a = img(os.path.join(SHOTS, f"{name}-{kind}-plain.png"), f"shot-{name}-{kind}-plain.png")
            b = img(os.path.join(SHOTS, f"{name}-{kind}-ev.png"), f"shot-{name}-{kind}-ev.png")
            links = ""
            if map_url:
                for stem, mid, h in views():
                    if stem.startswith(f"{name}-{kind}-"):
                        links += f" · <a href='{esc(map_url)}{esc(h)}'>open {esc(mid)}</a>"
            shot_items.append((f"{esc(what)} — {label} (box x {x0}…{x1}, z {z0}…{z1}){links}", a, b))
    section_items = []
    for name, cap in SECTIONS:
        pa = glob.glob(os.path.join(Q, "mic", "inspect", f"section-{name}", "*.png"))
        pb = glob.glob(os.path.join(Q, "mic-ev", "inspect", f"section-{name}", "*.png"))
        section_items.append((esc("Section " + cap),
                              img(pa[0], f"section-{name}-plain.png") if pa else None,
                              img(pb[0], f"section-{name}-ev.png") if pb else None))
    ore_items = []
    for name, *_rest in BOXES:
        what = _rest[-1]
        ore_items.append((esc("Ore map (top-down, highest ore per column, darker = deeper) — " + what),
                          img(os.path.join(Q, "mic", "inspect", f"ore-{name}", "ore.png"), f"ore-{name}-plain.png"),
                          img(os.path.join(Q, "mic-ev", "inspect", f"ore-{name}", "ore.png"), f"ore-{name}-ev.png")))

    # --- EV texture strip: a few pairs, without EV -> with EV -------------------------------------
    strip = []
    seen = set()
    for (host, here, there, exact), n in sorted(rows.items(), key=lambda kv: -kv[1]):
        if not exact or mat(here) not in CLAIMED or host in seen:
            continue
        seen.add(host)
        strip.append(f"<div class='sw'>{tile(host, 48)}<div class='t'>{esc(host.split(':')[1])}</div>"
                     f"{tile(here, 48)}<div class='t'>{esc(here.split(':')[1])}</div>{tile(there, 48)}<div class='t'>EV variant</div></div>")
        if len(strip) >= 12:
            break

    # --- costs -------------------------------------------------------------------------------------
    cp, ce = cost.get("mic", {}), cost.get("mic-ev", {})

    def med(v):
        v = [x for x in (v or []) if x is not None]
        return statistics.median(v) if v else None

    def fmt(v, unit="", nd=1):
        return "—" if v is None else f"{v:,.{nd}f}{unit}"

    def samples(v):
        return ", ".join(fmt(x, "", 1) for x in (v or []) if x is not None) or "—"

    ev_line = next((l for l in open(os.path.join(Q, "mic-ev", "server-lines.txt"), errors="replace")
                    if "Ore blocks: Excavated Variants" in l), "") if os.path.exists(os.path.join(Q, "mic-ev", "server-lines.txt")) else ""
    m = re.search(r"\((\d+) ores, (\d+) stones, (\d+) generated variants\)", ev_line)
    n_ores, n_stones, n_var = (m.groups() if m else ("?", "?", "?"))
    hides = [l.strip() for l in open(os.path.join(Q, "mic-ev", "server-lines.txt"), errors="replace")
             if "AutoHiding" in l] if os.path.exists(os.path.join(Q, "mic-ev", "server-lines.txt")) else []
    hide_tags = sorted({re.search(r"tag '([^']+)'", l).group(1) for l in hides if re.search(r"tag '([^']+)'", l)})
    gen = summary.get("generated_by_kind", {})
    gen_ms = summary.get("generation_ms_by_kind", {})
    client_secs = summary.get("client_log_seconds") or {}
    boot_delta = (med(ce.get("boots")) or 0) - (med(cp.get("boots")) or 0) if cp.get("boots") and ce.get("boots") else None
    rss_delta = (med(ce.get("rss")) or 0) - (med(cp.get("rss")) or 0) if cp.get("rss") and ce.get("rss") else None
    cost_rows = [
        ("Mod jars in the server set", f"{cp.get('mods')}", f"{ce.get('mods')}",
         "Excavated Variants 4.3.1 (bundles DefaultResources 3.6.0) and Dynamic Asset Generator 6.1.2; both LGPL-3.0-or-later, both "
         "last released 2024-08-13 (still the newest 1.21.1 builds on Modrinth)"),
        ("Blocks EV registers", "—", f"{n_var} variant blocks",
         f"EV knows {n_ores} ores and {n_stones} stones in this set; each variant also gets an item, tags and a loot table"),
        ("Server boot, JVM start to “Done (” (s), median", fmt(med(cp.get("boots"))), fmt(med(ce.get("boots"))),
         f"samples without EV: {samples(cp.get('boots'))}; with EV: {samples(ce.get('boots'))} (existing world, no generation; "
         f"difference of medians {fmt(boot_delta, ' s')}; other agents' builds share the Mac, so single samples are noisy)"),
        ("RSS at “Done (” (MB), median", fmt(med(cp.get("rss")), nd=0), fmt(med(ce.get("rss")), nd=0),
         f"samples without EV: {samples(cp.get('rss'))}; with EV: {samples(ce.get('rss'))} (difference of medians {fmt(rss_delta, ' MB', 0)})"),
        ("First start (world generation, 2 Chunky squares): boot / peak RSS", f"{fmt(cp.get('first_boot_s'), ' s')} / {fmt(cp.get('peak_rss'), ' MB', 0)}",
         f"{fmt(ce.get('first_boot_s'), ' s')} / {fmt(ce.get('peak_rss'), ' MB', 0)}", "one sample each"),
        ("Server data made at start", "—", "2 generated data packs",
         "dynamic_asset_generator:builtin_data and excavated_variants:data (tags, loot tables, recipes for the variants)"),
        ("Client assets made at resource load", "—",
         f"{gen.get('textures', '?')} textures, {gen.get('models', '?')} models, {gen.get('blockstates', '?')} blockstates",
         f"summed generation time {fmt(summary.get('generation_ms_total'), ' ms', 0)} (textures {fmt(gen_ms.get('textures'), ' ms', 0)}), "
         f"from Dynamic Asset Generator's own timing on the Mac mini M6; on every resource reload unless DAG's cache has them"),
        ("Client start to the block atlas (s)", "not measured", fmt(client_secs.get("atlas"), " s", 0),
         "a 5-mod client (Deep Time, Toroidal World, EV, DAG, Create), not the MIC client; "
         f"client RSS at the atlas {fmt(summary.get('client_rss_mb_at_atlas') or summary.get('client_rss_mb'), ' MB', 0)}"),
        ("Recipe viewer", "—", f"Almost Unified hides EV's variant items ({len(hide_tags)} ore tags)",
         "MIC's Almost Unified auto-hides the variants from recipe viewers, e.g. “Hiding 17/22 items for tag '#c:ores/iron'”"),
    ]
    cost_html = "".join(f"<tr><td>{esc(a)}</td><td>{esc(str(b))}</td><td>{esc(str(c))}</td><td class='t'>{esc(d)}</td></tr>"
                        for a, b, c, d in cost_rows)

    # --- headline numbers -------------------------------------------------------------------------
    dt_total = changed + unchanged
    no_cover_hosts = sorted({h for h, d in by_host.items() if h not in ev_stone_blocks and h != "(none)" and d["n"]},
                            key=lambda h: -by_host[h]["n"])
    no_cover_n = sum(by_host[h]["n"] for h in no_cover_hosts)
    stoneish = sum(by_host[h]["n"] for h in ("minecraft:stone", "minecraft:deepslate") if h in by_host)
    total_ores = sum(d["ore_blocks"] for d in per_box.values())
    tiles_html = "".join(strip)
    install = load(os.path.join(Q, "mic", "install.json"), {})

    title = "Deep Time — Q10: Excavated Variants in MIC"
    desc = ("The same Deep Time deposits in the MIC pack without and with Excavated Variants: 3D and 2D renders, "
            "which ore block each host rock gets, and what EV costs.")
    body = f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{esc(title)}</title>
<meta name="description" content="{esc(desc)}">
<style>
:root{{--bg:#fbfaf7;--fg:#1d1d1b;--mut:#6b6a66;--line:#e3e0d8;--card:#fff;--acc:#8a5a14}}
@media (prefers-color-scheme: dark){{:root:not([data-theme=light]){{--bg:#161614;--fg:#ecebe6;--mut:#a3a19a;--line:#34332f;--card:#1f1e1c;--acc:#e0b066}}}}
:root[data-theme=dark]{{--bg:#161614;--fg:#ecebe6;--mut:#a3a19a;--line:#34332f;--card:#1f1e1c;--acc:#e0b066}}
body{{background:var(--bg);color:var(--fg);font:15px/1.55 system-ui,-apple-system,Segoe UI,sans-serif;margin:0;padding:24px 16px}}
main{{max-width:1180px;margin:0 auto}} h1{{margin:0 0 4px;font-size:26px}} h2{{margin:36px 0 8px;font-size:20px}}
.sub,.t{{color:var(--mut)}} .t{{font-size:12.5px}} a{{color:var(--acc)}}
.box{{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px;margin:14px 0}}
.kpis{{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:10px;margin:14px 0}}
.kpi{{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:10px 12px}}
.kpi b{{display:block;font-size:22px;font-variant-numeric:tabular-nums}}
table{{width:100%;border-collapse:collapse;background:var(--card);border:1px solid var(--line);margin:10px 0}}
th,td{{border-top:1px solid var(--line);padding:6px 8px;text-align:left;vertical-align:top}} th{{font-size:13px;color:var(--mut)}}
td.r{{text-align:right;font-variant-numeric:tabular-nums}} td.c{{text-align:center}}
code{{font-size:12.5px;overflow-wrap:anywhere}}
.tile{{image-rendering:pixelated;vertical-align:middle;border:1px solid var(--line)}}
.notile{{display:inline-block;width:24px;height:24px;border:1px dashed var(--line);vertical-align:middle}}
.ob{{display:flex;align-items:center;gap:6px;font-size:13px;margin:2px 0}} .ob b{{margin-left:auto;font-variant-numeric:tabular-nums}}
.pair{{margin:18px 0}} .pair figcaption{{font-weight:600;margin-bottom:6px}}
.two{{display:grid;grid-template-columns:1fr 1fr;gap:10px}} .two img{{width:100%;height:auto;border:1px solid var(--line);border-radius:6px;image-rendering:pixelated}}
.lab{{font-size:12px;color:var(--mut);margin-bottom:2px}}
.strip{{display:flex;flex-wrap:wrap;gap:14px}} .sw{{display:grid;grid-template-columns:auto;justify-items:center;gap:2px;font-size:12px}}
ul{{padding-left:20px}} li{{margin:4px 0}}
@media (max-width:720px){{.two{{grid-template-columns:1fr}} td,th{{padding:5px}}}}
</style></head><body><main>
<h1>{esc(title)}</h1>
<div class="sub">Plan Q10, decisions log 2026-09-28: “decide after renders”. This page is the evidence; it does not decide.
Built {esc(__import__('datetime').date.today().isoformat())} from Deep Time's synthetic deposit world (<code>deposits_world_16k.json</code>, every-class strata),
MIC pack {esc(str(install.get('pack', '?')))} (server set), the MIC rock-tag datapack, level seed {esc(str(install.get('seed', '?')))}, first-start generation on the Mac mini.
The two worlds differ only by two jars: Excavated Variants (EV) 4.3.1 and Dynamic Asset Generator (DAG) 6.1.2.</div>

<div class="kpis">
<div class="kpi"><b>{dt_total:,}</b>Deep Time ore blocks in the three boxes (coal, copper, iron, gold, zinc and the other claimed metals)</div>
<div class="kpi"><b>{changed:,}</b>of them are a different block with EV: an ore-in-this-rock variant</div>
<div class="kpi"><b>{no_cover_n:,}</b>sit in rock EV has no stone for, so they stay stone or deepslate ore either way</div>
<div class="kpi"><b>{n_var}</b>blocks EV adds to MIC; the client paints {gen.get('textures', '?')} textures for them at start</div>
</div>

<div class="box"><b>How to read it.</b> Without EV, Deep Time places the pack's own ore block: <code>#c:ores/&lt;metal&gt;</code> in stone
(above y 0) or deepslate (below). With EV it places EV's variant of that ore for the host rock, when EV has a stone config for the host
(vanilla's and Create's stones, plus Deep Time's four stand-ins). EV's own ore replacer stays off in Deep Time levels (P1-15's compat mixin),
so nothing else changes: the other mods' ores (uranium, nickel, fluorite, platinum, …) and vanilla's diamond, lapis and redstone are
identical in both worlds ({sum(other_ores.values()):,} such blocks in the boxes). {total_ores:,} ore positions were compared; {diff_pos} hold
ore in only one of the two worlds (feature placement differs very slightly between runs).
{"<a href='" + esc(map_url) + "'><b>Open the 3D maps</b></a> (one BlueMap app, four maps: pick “MIC without EV” or “MIC with EV”, surface or cut, from the menu)." if map_url else ""}</div>

<h2>The same blocks, side by side</h2>
<p class="t">Top: a host rock in these worlds. Middle: the ore block Deep Time places in it without EV (the pack's stone or deepslate ore).
Bottom: the block with EV. The most frequent pair per host; textures from the jars and from DAG's output on a client.</p>
<div class="strip">{tiles_html}</div>

<h2>3D: BlueMap</h2>
<p class="t">Screenshots of the four maps from the same camera; the links open the live map at that view.
The cut maps keep only blocks at or below the box's cut height, so the ore bodies show inside the rock.</p>
{pairs_html(shot_items)}

<h2>2D: sections and ore maps</h2>
<p class="t"><code>deeptime-inspect section</code> (4 px per block, colours from each set's textures: with EV the colour table includes DAG's textures)
and <code>deeptime-inspect ore</code> (materials, so it is the same in both worlds by design: it shows where the ore is, not which block).</p>
{pairs_html(section_items)}
{pairs_html(ore_items)}

<h2>Which ore block each host rock gets</h2>
<p class="t">Deep Time's own ores only (claimed materials), all three boxes, every height. Host = EV's stone where the EV world has a variant (exact),
otherwise the most common rock beside the ore block in the same layer (estimate; where both exist the estimate matched EV's stone for
{agree:,} of {total:,} positions, {100 * agree / max(total, 1):.0f}%). “EV stone?” = EV (or Deep Time's stand-in list) has a stone config for this block.</p>
<table><tr><th></th><th>Host rock (rock classes it stands for in MIC)</th><th>EV stone?</th><th>Ore blocks</th><th>Without EV</th><th>With EV</th></tr>
{''.join(host_rows)}</table>

<h2>Which MIC rock blocks EV covers</h2>
<p class="t">Every block the MIC rock tags resolve a class to (the probe palette of the world). “no” means ore in that rock is a stone or deepslate
ore with and without EV.</p>
<table><tr><th></th><th>Block</th><th>Rock classes</th><th>EV stone?</th></tr>{''.join(cover_rows)}</table>

<h2>What EV costs</h2>
<table><tr><th></th><th>Without EV</th><th>With EV</th><th>Notes</th></tr>{cost_html}</table>

<h2>Trade-offs</h2>
<div class="box"><ul>
<li><b>Look.</b> With EV, ore in vanilla's and Create's rocks and in Deep Time's stand-ins takes the rock's own texture (copper in veridium,
iron in crimsite, coal in dripstone, …). Without EV the same blocks are stone ore above y 0 and deepslate ore below, which reads as grey or
dark patches inside coloured rock (see the cut maps and sections).</li>
<li><b>Coverage.</b> EV has no stone for {esc(', '.join(h.split(':')[1] for h in no_cover_hosts[:8]))}{' …' if len(no_cover_hosts) > 8 else ''}
in these boxes ({no_cover_n:,} Deep Time ore blocks), nor for the other MIC rock blocks marked “no” above (Born in Chaos's argillites
carry coal, shale, slate and phyllite in MIC; obsidian; loose sediments). Those stay stone/deepslate ore in both worlds unless stone configs
are added (the way Deep Time adds its four). It has no ore config for Create Nuclear's lead or Ice and Fire's silver either.</li>
<li><b>Mods and upkeep.</b> Two more mods (three with the bundled DefaultResources), {n_var} more blocks and items, generated tags and loot
tables. EV and DAG were last released for 1.21.1 in August 2024. EV's own ore replacer stays on in the pack's other dimensions (only Deep Time
levels switch it off).</li>
<li><b>Start-up.</b> The server-side difference is inside the noise of these samples (table above); each client paints EV's textures when
it loads resources (about {fmt(summary.get('generation_ms_total'), ' ms', 0)} of work on the M6 for {gen.get('textures', '?')} textures).</li>
<li><b>Items.</b> Each variant is its own block and item with its own name; MIC's Almost Unified hides the items in recipe viewers.
EV generates their loot tables and tags at server start (drops were not tested here).</li>
<li><b>Deep Time either way.</b> The deposits, their positions and their sizes are the same; only the block ids differ. Deep Time works with
or without EV; no Deep Time code change is needed to keep or drop it.</li>
</ul></div>

<h2>What is uncertain</h2>
<div class="box"><ul>
<li>EV's textures come from a 5-mod client (Deep Time, Toroidal World, EV, DAG, Create), not the MIC client: the variants are the same
({n_var}; no other MIC mod has an EV ore or stone config), but a MIC resource pack that retextures ores or stones would change the look.</li>
<li>Boot times and RSS are single-digit samples on a shared Mac; small differences are not significant.</li>
<li>BlueMap draws EV's blocks from the pack made of DAG's cache; the in-game look matches the textures, but not lighting or shaders.</li>
<li>The world is Deep Time's synthetic deposit world (real presets wait for P1-03…P1-08); the host rocks are its every-class strata, so every rock
class occurs, more evenly mixed than a real planet's.</li>
<li>Item and loot behaviour of EV's variants was not tested here (no player).</li>
</ul></div>

<h2>Reproduce</h2>
<div class="box t"><code>tools/remote/mac.sh --pack -- 'tools/review/q10-ev.sh install'</code> ·
<code>tools/remote/mac.sh --server -- 'tools/review/q10-ev.sh worlds'</code> · <code>tools/client/ev-assets.sh</code> ·
<code>tools/remote/mac.sh --pull review/out/q10 -- 'tools/review/q10-ev.sh inspect'</code> ·
<code>tools/remote/mac.sh --server -- 'tools/review/q10-ev.sh bluemap'</code> ·
<code>tools/remote/mac.sh --server --pull review/out/q10 -- 'tools/review/q10-ev.sh boots'</code> ·
<code>python3 tools/review/q10_ev.py merge|shots|page</code></div>
</main></body></html>
"""
    open(os.path.join(PAGE, "index.html"), "w").write(body)
    files = sum(len(fs) for _, _, fs in os.walk(PAGE))
    size = sum(os.path.getsize(os.path.join(r, f)) for r, _, fs in os.walk(PAGE) for f in fs)
    print(f"page: {PAGE}: {files} files, {size / 1e6:.1f} MB; Deep Time ores {dt_total} (changed {changed}), "
          f"no EV stone {no_cover_n}, estimate {agree}/{total}")


if __name__ == "__main__":
    step = sys.argv[1] if len(sys.argv) > 1 else ""
    {"merge": merge, "shots": shots, "page": page}.get(step, lambda: sys.exit(__doc__))()
