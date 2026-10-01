#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Drives the workbench's deposits overlay in headless Chromium and takes its screenshots (the
checks of the "Deposits layer" task, 2026-10-01). Stdlib only; run on the Mac against a private
`deeptime watch` there (never Ben's on port 8642):

    python3 tools/viewer/deposits_shots.py --url http://127.0.0.1:PORT/ --run RUN_ID --out DIR
        [--chromium /path/to/Chromium] [--size 1600x900]

It opens the run, waits for the overlay's legend, then (real mouse input through DevTools)
hovers a coal field and a symbol, clicks each, switches the base layer, toggles the overlay off and
on, and checks what the page says at every step. Prints PASS/FAIL lines, writes PNGs to DIR, and
exits 1 when a check failed.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from shot import WS, free_port  # noqa: E402

failed = []


def check(ok, what):
    print(("PASS " if ok else "FAIL ") + what, flush=True)
    if not ok:
        failed.append(what)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", required=True)
    ap.add_argument("--run", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--chromium", default=shutil.which("chromium") or shutil.which("google-chrome"))
    ap.add_argument("--size", default="1600x900")
    a = ap.parse_args()
    if not a.chromium:
        sys.exit("deposits_shots.py: no Chromium (pass --chromium)")
    w, h = (int(v) for v in a.size.split("x"))
    os.makedirs(a.out, exist_ok=True)
    port = free_port()
    profile = tempfile.mkdtemp(prefix="dt-dep-")
    proc = subprocess.Popen(
        [a.chromium, "--headless=new", "--disable-gpu", "--no-sandbox", "--hide-scrollbars",
         f"--remote-debugging-port={port}", f"--user-data-dir={profile}", f"--window-size={w},{h}", "about:blank"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        for _ in range(150):
            try:
                targets = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list", timeout=2))
                page = next(t for t in targets if t.get("type") == "page")
                break
            except Exception:
                time.sleep(0.2)
        else:
            sys.exit("deposits_shots.py: Chromium's DevTools endpoint did not come up")
        ws = WS(page["webSocketDebuggerUrl"])
        ws.call("Emulation.setDeviceMetricsOverride", width=w, height=h, deviceScaleFactor=1, mobile=False)
        ws.call("Page.enable")
        errors = []

        def ev(js):
            r = ws.call("Runtime.evaluate", expression=js, awaitPromise=True, returnByValue=True)
            if r.get("exceptionDetails"):
                errors.append(js[:80])
                print("JS threw:", json.dumps(r["exceptionDetails"])[:300], flush=True)
                return None
            return r.get("result", {}).get("value")

        def shot(name):
            import base64
            d = ws.call("Page.captureScreenshot", format="png")
            p = os.path.join(a.out, name)
            with open(p, "wb") as f:
                f.write(base64.b64decode(d["data"]))
            print(f"shot {p}", flush=True)

        def mouse(kind, x, y, button="none", clicks=0):
            ws.call("Input.dispatchMouseEvent", type=kind, x=x, y=y, button=button, clickCount=clicks)

        def wait(js, secs=60, what="condition"):
            until = time.time() + secs
            while time.time() < until:
                if ev(js):
                    return True
                time.sleep(0.4)
            print(f"timed out waiting for {what}", flush=True)
            return False

        def run_state():
            return ev("fetch('api/runs').then(r => r.json()).then(j => (j.runs.find(r => r.id === %s) || {}).state)" % json.dumps(a.run))

        ws.call("Page.navigate", url=f"{a.url}?run={a.run}")
        # The page is open while the run goes on: the sites appear when the deposit stage has placed
        # them (the live path), and the legend switches to "baked" when the run is done.
        ok = wait("(() => { const b = document.querySelector('#deplegend'); return !!b && !b.hidden && b.querySelectorAll('.dep-row').length > 0; })()", 7200, "the deposits legend")
        check(ok, "the deposits legend appears once the run's frames name the sites layer")
        state = run_state()
        note = ev("document.querySelector('#deplegend .note').textContent") or ""
        if state != "done":
            check("as the deposit stage placed" in note, f"while the run is {state} the sites are the live ones ({note[:50]!r})")
            shot("00-live-overlay.png")
            check(wait("(() => { const n = document.querySelector('#deplegend .note'); return !!n && n.textContent.includes('baked'); })()", 1800, "the baked sites"),
                  "when the run is done the legend switches to the baked sites, without a reload")
        else:
            print("the run was already done: the live path was not observed here", flush=True)
        # Let the base map image load.
        time.sleep(3)
        shot("01-satellite-overlay.png")

        legend = ev("""(() => [...document.querySelectorAll('#deplegend .dep-row')].map(r => [r.dataset.fam, r.querySelector('.nm').textContent, Number(r.querySelector('.n').textContent)]))()""")
        print("legend:", legend, flush=True)
        check(legend is not None and len(legend) >= 8, "the legend lists every family (with counts, zeros included)")
        counts = {k: n for k, _, n in (legend or [])}
        site_total = sum(counts.values())
        data = ev(f"fetch('api/runs/{a.run}/deposits').then(r => r.json())")
        check(bool(data) and data.get("available") is True and len(data["sites"]) == site_total,
              f"the legend counts add up to the {site_total} sites the server sent")
        check(ev("document.querySelector('#depbtn').getAttribute('aria-pressed')") == "true", "the Deposits toggle is on")

        # Where the first coal field and the first site of another family are, in viewport pixels.
        loc = ev("""(() => {
          const c = document.querySelector('#paneA canvas.map'), r = c.getBoundingClientRect();
          const seen = new Map();   // sites on one spot are drawn 10 px apart, in id order (deposits.js)
          return fetch('api/runs/__RUN__/deposits').then(x => x.json()).then(d => d.sites.map(s => ({
            id: s.id, type: s.type, x: r.left + r.width * ((s.lon % 360) / 360) + 10 * (() => { const k = `${s.lat},${s.lon}`, n = seen.get(k) || 0; seen.set(k, n + 1); return n; })(), y: r.top + r.height * (90 - s.lat) / 180,
            ev: s.forming_event, parent: s.parent })));
        })()""".replace("__RUN__", a.run))
        coal = [s for s in loc if s["type"].endswith("coal_seam")]
        other = [s for s in loc if not s["type"].endswith("coal_seam")]
        print(f"{len(coal)} coal fields, {len(other)} other sites", flush=True)

        def hover(s):
            mouse("mouseMoved", s["x"] - 30, s["y"] - 30)
            mouse("mouseMoved", s["x"], s["y"])
            time.sleep(0.5)
            return ev("""(() => { const t = [...document.querySelectorAll('.dep-tip')].find(e => !e.hidden); return t ? t.innerText : null; })()""")

        if coal:
            tip = hover(coal[0])
            print("coal tip:", repr(tip), flush=True)
            check(bool(tip) and "Coal seam field" in tip and "Size" in tip and "Age" in tip and "Footprint" in tip and "coal" in tip,
                  "hovering a coal field shows its family, size, footprint and age")
            shot("02-hover-coal.png")
            mouse("mousePressed", coal[0]["x"], coal[0]["y"], "left", 1)
            mouse("mouseReleased", coal[0]["x"], coal[0]["y"], "left", 1)
            time.sleep(0.6)
            det = ev("document.querySelector('#details').innerText")
            print("coal details:", repr((det or "")[:300]), flush=True)
            check(ev("document.querySelector('#ip-details').hidden === false") is True, "clicking a coal field opens the Details tab")
            low = (det or "").lower()
            check("why here" in low and "coal seam field" in low and "forming event" in low and "deposit formation" in low,
                  "the details carry the site, 'Why here' and the forming event")
            check(ev("document.querySelectorAll('#details .dep-why .lv').length") >= 1, "the cause chain lists the forming event first")
            shot("03-click-coal-why-here.png")
        for s in other:
            if s["parent"] is not None:
                continue
            tip = hover(s)
            print(s["type"], "tip:", repr(tip), flush=True)
            check(bool(tip) and "Size" in tip and "Age" in tip, f"hovering {s['type'].split(':')[1]} (site {s['id']}) shows the card")
            shot(f"04-hover-{s['type'].split(':')[1]}.png")
            mouse("mousePressed", s["x"], s["y"], "left", 1)
            mouse("mouseReleased", s["x"], s["y"], "left", 1)
            time.sleep(0.6)
            det = ev("document.querySelector('#details').innerText")
            check("why here" in (det or "").lower(), f"clicking site {s['id']} opens its why-here")
            shot(f"05-click-{s['type'].split(':')[1]}.png")
            break
        # A placer's parent link (the kin buttons).
        kids = [s for s in loc if s["parent"] is not None]
        if kids:
            k = kids[0]
            hover(k)
            mouse("mousePressed", k["x"], k["y"], "left", 1)
            mouse("mouseReleased", k["x"], k["y"], "left", 1)
            time.sleep(0.6)
            check(ev("!!document.querySelector('#details [data-site]')") is True, "a placer or blanket links to the deposit it came from")
            shot("06-click-child-kin.png")

        # Over another base layer: elevation.
        ev("document.querySelector('[data-layer=\"elevation\"]') && document.querySelector('[data-layer=\"elevation\"]').click()")
        time.sleep(2.5)
        check(ev("document.querySelector('#deplegend').hidden") is False, "the overlay and its legend stay over the elevation layer")
        shot("07-elevation-overlay.png")

        # Hide a family from the legend, then the whole overlay.
        if legend:
            fam = next((k for k, _, n in legend if n > 0), None)
            if fam:
                ev(f"document.querySelector('#deplegend [data-fam=\"{fam}\"]').click()")
                time.sleep(0.5)
                check(ev(f"document.querySelector('#deplegend [data-fam=\"{fam}\"]').getAttribute('aria-pressed')") == "false", f"a legend row hides its family ({fam})")
                shot("08-family-hidden.png")
                ev(f"document.querySelector('#deplegend [data-fam=\"{fam}\"]').click()")
        ev("document.querySelector('#depbtn').click()")
        time.sleep(0.6)
        check(ev("document.querySelector('#deplegend').hidden") is True, "the toggle hides the overlay and its legend")
        shot("09-overlay-off.png")
        ev("document.querySelector('#depbtn').click()")
        time.sleep(0.6)
        check(ev("document.querySelector('#deplegend').hidden") is False, "and shows them again")

        # What the overlay costs a frame: the run's sites, and ten times as many (every redraw does it).
        cost = ev("""(async () => {
          const d = await fetch('api/runs/__RUN__/deposits').then(r => r.json());
          const c = document.createElement('canvas'); c.width = 2048; c.height = 1024;
          const ctx = c.getContext('2d');
          const view = { project: (lat, lon) => [lon / 360 * c.width, (90 - lat) / 180 * c.height], ppdx: c.width / 360, ppdy: c.height / 180, worldW: c.width, dpr: 1 };
          const run = answer => {
            const t = performance.now(), m = DTDeposits.prepare(answer), tp = performance.now() - t;
            const N = 100, st = { hidden: new Set() };
            const t0 = performance.now(); for (let i = 0; i < N; i++) DTDeposits.draw(ctx, m, view, st);
            const t1 = performance.now(); for (let i = 0; i < N; i++) DTDeposits.hit(m, view, 700 + i, 300, st);
            const t2 = performance.now();
            return { sites: m.sites.length, prepare_ms: +tp.toFixed(2), draw_ms: +((t1 - t0) / N).toFixed(3), hit_ms: +((t2 - t1) / N).toFixed(4) };
          };
          const big = Object.assign({}, d, { sites: Array.from({ length: 10 }, (_, k) => d.sites.map(s => Object.assign({}, s, { id: s.id + 1000 * k, forming_event: s.forming_event + 1e6 * k }))).flat() });
          return { run: run(d), x10: run(big) };
        })()""".replace("__RUN__", a.run))
        print("render cost (2048 x 1024 canvas):", json.dumps(cost), flush=True)
        check(bool(cost) and cost["run"]["draw_ms"] < 8, "drawing the overlay costs a few ms at most per frame")

        # Back to the first step: no sites yet (the deposit stage has not run there).
        ev("document.querySelector('#rewind').click()")
        time.sleep(1.5)
        leg = ev("document.querySelector('#deplegend').innerText")
        check(bool(leg) and "step" in leg.lower(), "before the deposit stage the legend says when the sites appear")
        shot("10-first-step.png")
        check(not errors, "no script errors")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        shutil.rmtree(profile, ignore_errors=True)
    print(f"{len(failed)} failed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
