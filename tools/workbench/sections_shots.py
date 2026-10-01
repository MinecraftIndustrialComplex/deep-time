#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Screenshots of the workbench's rock at depth and section tool, headless, on the Mac.

    python3 tools/workbench/sections_shots.py http://127.0.0.1:8661 <run id> <out dir>
        [--layer structure/rock] [--depths 20,100,floor]
        [--section name=x0,z0,x1,z1]... [--size 1600x1000]

Drives the page the way a person does, over the DevTools protocol (tools/viewer/shot.py's
WebSocket client; Chromium's headless shell from playwright's cache, DEEPTIME_CHROME overrides):
opens the run on a rock layer, moves the depth control to each depth in turn, then for each
--section (block coordinates, as the server takes them) picks the section tool, clicks the two
points on the map with real mouse events, waits for the section and shoots the dock, the Section
tab and the enlarged view, each also with --light's class lit up in the key. Writes
<out dir>/<name>.png and prints each step's timing and anything the page threw. Run it on the Mac (tools/remote/mac.sh), never the laptop.
"""

import argparse
import base64
import glob
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.request

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
spec = importlib.util.spec_from_file_location("shot", os.path.join(REPO, "tools", "viewer", "shot.py"))
shot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(shot)


def chrome():
    if os.environ.get("DEEPTIME_CHROME"):
        return os.environ["DEEPTIME_CHROME"]
    found = sorted(glob.glob(os.path.expanduser(
        "~/Library/Caches/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-*/chrome-headless-shell")))
    if not found:
        sys.exit("no chrome-headless-shell found; set DEEPTIME_CHROME")
    return found[-1]


class Page:
    def __init__(self, ws, size):
        self.ws, self.size = ws, size

    def js(self, expr, wait=True):
        r = self.ws.call("Runtime.evaluate", expression=expr, awaitPromise=wait, returnByValue=True)
        if r.get("exceptionDetails"):
            raise RuntimeError(f"{expr[:120]!r}: {r['exceptionDetails'].get('exception', {}).get('description') or r['exceptionDetails'].get('text')}")
        return r.get("result", {}).get("value")

    def until(self, cond, what, timeout=60):
        t0 = time.time()
        while time.time() - t0 < timeout:
            if self.js(f"Promise.resolve({cond})"):
                return time.time() - t0
            time.sleep(0.15)
        raise RuntimeError(f"timed out waiting for {what}")

    def shoot(self, path):
        data = self.ws.call("Page.captureScreenshot", format="png")["data"]
        with open(path, "wb") as f:
            f.write(base64.b64decode(data))
        print(f"  {path}: {os.path.getsize(path):,} bytes")

    def click(self, x, y):
        for kind in ("mousePressed", "mouseReleased"):
            self.ws.call("Input.dispatchMouseEvent", type=kind, x=x, y=y, button="left", clickCount=1)
            time.sleep(0.05)

    def move(self, x, y):
        self.ws.call("Input.dispatchMouseEvent", type="mouseMoved", x=x, y=y)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("base")
    ap.add_argument("run")
    ap.add_argument("out")
    ap.add_argument("--layer", default="structure/rock")
    ap.add_argument("--depths", default="20,100,floor")
    ap.add_argument("--section", action="append", default=[])
    ap.add_argument("--light", default="diamictite", help="a rock class to light up in the section's key")
    ap.add_argument("--size", default="1600x1000")
    ap.add_argument("--no-bake", action="store_true", help="the run has no bake: shoot what the page says instead of waiting for a slice")
    a = ap.parse_args()
    w, h = (int(v) for v in a.size.split("x"))
    os.makedirs(a.out, exist_ok=True)
    info = json.load(urllib.request.urlopen(f"{a.base}/api/runs/{a.run}/rock"))
    C = info.get("circumference", 16384)
    port = shot.free_port()
    profile = tempfile.mkdtemp(prefix="dt-sections-")
    proc = subprocess.Popen([chrome(), "--disable-gpu", "--no-sandbox", "--hide-scrollbars", f"--remote-debugging-port={port}",
                             f"--user-data-dir={profile}", f"--window-size={w},{h}", "about:blank"],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        for _ in range(100):
            try:
                targets = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list", timeout=2))
                page = next(t for t in targets if t.get("type") == "page")
                break
            except Exception:
                time.sleep(0.2)
        else:
            sys.exit("the DevTools endpoint did not come up")
        ws = shot.WS(page["webSocketDebuggerUrl"])
        ws.call("Emulation.setDeviceMetricsOverride", width=w, height=h, deviceScaleFactor=1, mobile=False)
        ws.call("Page.enable")
        ws.call("Page.addScriptToEvaluateOnNewDocument",
                source="window.__errs = []; addEventListener('error', e => __errs.push(String(e.message)));"
                       "addEventListener('unhandledrejection', e => __errs.push('rejected: ' + e.reason));")
        p = Page(ws, (w, h))
        ws.call("Page.navigate", url=f"{a.base}/?run={a.run}#a={a.run}&layer={a.layer}")
        t = p.until("!!document.querySelector('#depthctl') && !document.querySelector('#depthctl').hidden", "the depth control")
        print(f"page up, depth control after {t:.1f} s")
        p.until("!!window.DTWB.pane(0).drawn", "the map's first picture")
        time.sleep(1.0)
        p.shoot(os.path.join(a.out, "rock-surface.png"))
        for d in a.depths.split(","):
            t0 = time.time()
            p.js(f"document.querySelector('#dc-chips button[data-d=\"{d}\"]').click()", wait=False)
            if a.no_bake:   # the run has no bake to read: the page says so
                time.sleep(1.5)
            else:
                p.until("document.querySelector('.slicebadge:not([hidden])') !== null", f"the slice at {d}", 90)
            time.sleep(0.6)
            note = p.js("document.querySelector('#dc-note').textContent")
            print(f"depth {d}: on screen after {time.time() - t0:.1f} s: {note}")
            p.shoot(os.path.join(a.out, f"rock-{d}.png"))
        # Rock 50 blocks down under the sections' map: buried beds show where a line cuts them.
        p.js("document.querySelector('#dc-chips button[data-d=\"50\"]').click()", wait=False)
        time.sleep(1.5)
        for spec_ in a.section:
            name, coords = spec_.split("=", 1)
            x0, z0, x1, z1 = (float(v) for v in coords.split(","))
            ll = lambda x, z: (-z / C * 360.0, (x + C / 2) / C * 360.0)
            (la0, lo0), (la1, lo1) = ll(x0, z0), ll(x1, z1)
            p.js("window.DTWB.setTool('section')", wait=False)
            time.sleep(0.3)
            pos = lambda la, lo: p.js(
                "(() => { const r = document.querySelector('#paneA canvas.map').getBoundingClientRect();"
                f"return [r.left + (({lo} % 360) / 360) * r.width, r.top + ((90 - {la}) / 180) * r.height]; }})()")
            xa, ya = pos(la0, lo0)
            xb, yb = pos(la1, lo1)
            p.move(xa, ya)
            p.click(xa, ya)
            p.move(xa + (xb - xa) * 0.6, ya + (yb - ya) * 0.6)
            time.sleep(0.4)
            p.shoot(os.path.join(a.out, f"{name}-pick-b.png"))
            t0 = time.time()
            p.click(xb, yb)
            p.until("(() => { const d = document.querySelector('.secdock'); return !!d && !d.hidden && !!d.querySelector('canvas'); })()", f"section {name}")
            print(f"section {name}: on screen after {time.time() - t0:.2f} s")
            time.sleep(0.8)
            box = p.js("(() => { const r = document.querySelector('.secdock canvas').getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; })()")
            p.move(box[0] + box[2] * 0.55, box[1] + box[3] * 0.4)
            time.sleep(0.4)
            p.shoot(os.path.join(a.out, f"{name}-dock.png"))
            if a.light:
                lit = p.js("(() => { const c = [...document.querySelectorAll('.secdock [data-cls]')].find(e => e.textContent.trim().startsWith(%r)); if (!c) return null; c.click(); return c.textContent; })()" % a.light)
                if lit:
                    time.sleep(0.5)
                    print(f"lit up in the dock: {lit}")
                    p.shoot(os.path.join(a.out, f"{name}-dock-lit.png"))
                    p.js("(() => { const e = document.querySelector('.secdock [data-cls][aria-pressed=true]'); if (e) e.click(); })()", wait=False)
            p.js("window.DTWB.setITab('section')", wait=False)
            time.sleep(0.3)
            p.shoot(os.path.join(a.out, f"{name}-tab.png"))
            p.js("document.querySelector('.secdock [data-sec=big]').click()", wait=False)
            p.until("!!document.querySelector('.sec-big canvas')", "the enlarged view")
            time.sleep(0.8)
            box = p.js("(() => { const r = document.querySelector('.sec-big canvas').getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; })()")
            p.move(box[0] + box[2] * 0.5, box[1] + box[3] * 0.35)
            time.sleep(0.4)
            p.shoot(os.path.join(a.out, f"{name}-big.png"))
            if a.light:
                lit = p.js("(() => { const c = [...document.querySelectorAll('.sec-big [data-cls]')].find(e => e.textContent.trim().startsWith(%r)); if (!c) return null; c.click(); return c.textContent; })()" % a.light)
                if lit:
                    time.sleep(0.5)
                    p.shoot(os.path.join(a.out, f"{name}-big-lit.png"))
            p.js("document.querySelector('#modalclose').click()", wait=False)
            time.sleep(0.3)
        if a.section:
            # The dock, the tab and the depth control answer clicks as they should.
            state = "(() => { const d = document.querySelector('.secdock'); return [d.hidden, d.classList.contains('min')]; })()"
            checks = []
            checks.append(("dock open after a section", p.js(state) == [False, False]))
            p.js("document.querySelector('.secdock [data-sec=min]').click()", wait=False)
            checks.append(("fold", p.js(state) == [False, True]))
            p.js("document.querySelector('.secdock [data-sec=min]').click()", wait=False)
            checks.append(("unfold", p.js(state) == [False, False]))
            p.js("document.querySelector('.secdock [data-sec=close]').click()", wait=False)
            checks.append(("close", p.js(state)[0] is True))
            p.js("window.DTWB.setITab('section'); document.querySelector('#ip-section [data-sec=show]').click()", wait=False)
            checks.append(("show again from the tab", p.js(state) == [False, False]))
            p.js("document.querySelector('#dc-chips button[data-d=\"0\"]').click()", wait=False)
            time.sleep(0.6)
            checks.append(("surface hides the slice badge", p.js("document.querySelector('.slicebadge').hidden") is True))
            p.js("document.dispatchEvent(new KeyboardEvent('keydown', {key: 'Escape', bubbles: true}))", wait=False)
            checks.append(("Esc leaves the section tool", p.js("window.DTWB.tool()") == "select"))
            for what, ok in checks:
                print(f"check {what}: {'ok' if ok else 'FAILED'}")
        errs = p.js("window.__errs")
        print("page errors:", errs if errs else "none")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        shutil.rmtree(profile, ignore_errors=True)


if __name__ == "__main__":
    main()
