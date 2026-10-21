#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Drives the workbench's Final world view (worldmap.js over mic-viewer's map core; decisions log
2026-10-21) in headless Chromium and takes its screenshots. Stdlib only; run on the test VM against
a private `deeptime watch` there (never Ben's on port 8642), over a finished run:

    python3 tools/viewer/worldmap_shots.py --url http://127.0.0.1:PORT/ --run RUN_ID --out DIR
        [--chromium /path/to/chromium]

Checks, reading the view back from the address the page keeps (`?world=layer/x/z/bpp`) and from
the canvas's pixels:
  - opening the view shows the whole planet (the circumference fits the canvas's width);
  - "+" halves the blocks a pixel, "Whole world" goes back, the zoom stops at the magnification
    limit (4 screen pixels a block);
  - a drag across the seam wraps x onto the planet and both sides of the seam are drawn;
  - a link opens at its layer and view, and a reload keeps it;
  - Escape closes the view; at phone width the view fits with no sideways scroll, has room (300 px
    or more) and draws the planet;
  - no script errors.
Prints PASS/FAIL lines, writes PNGs to DIR, and exits 1 when a check failed.
"""

import argparse
import base64
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


# Records script errors from the first byte of every page.
RECORDER = "window.__errs = []; addEventListener('error', e => __errs.push(String(e.message)));" \
           "addEventListener('unhandledrejection', e => __errs.push('rejection: ' + String(e.reason)));"

# The view as the page states it: the address's `world` (written 0.5 s after the view settles),
# the canvas box, and whether the left and right thirds of the canvas carry ink.
STATE = """(() => {
  const c = document.getElementById('wv-canvas'), host = document.getElementById('worldview');
  const q = new URLSearchParams(location.search).get('world');
  const r = c ? c.getBoundingClientRect() : { width: 0, height: 0 };
  let ink = [0, 0];
  if (c && c.width > 8 && c.height > 8) {
    const g = c.getContext('2d'), w = c.width, h = c.height;
    const lit = (x0) => { const d = g.getImageData(x0, Math.floor(h * 0.3), Math.floor(w / 3), Math.floor(h * 0.4)).data;
      let n = 0; for (let i = 0; i < d.length; i += 16) if (d[i] + d[i + 1] + d[i + 2] > 60) n++; return n; };
    ink = [lit(0), lit(w - Math.floor(w / 3))];
  }
  return { world: q, w: r.width, h: r.height, open: !!host && !host.hidden, ink,
           readout: (document.getElementById('wv-readout') || {}).textContent || '',
           scrollW: document.documentElement.scrollWidth, innerW: innerWidth, errs: window.__errs || [] };
})()"""


def parse_world(q):
    if not q:
        return None
    p = q.split("/")
    return {"layer": p[0], "cx": float(p[1]), "cz": float(p[2]), "bpp": float(p[3])}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", required=True)
    ap.add_argument("--run", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--chromium", default=shutil.which("chromium") or shutil.which("google-chrome"))
    a = ap.parse_args()
    if not a.chromium:
        sys.exit("worldmap_shots.py: no Chromium (pass --chromium)")
    os.makedirs(a.out, exist_ok=True)
    info = json.load(urllib.request.urlopen(f"{a.url}api/runs/{a.run}/world.json", timeout=60))
    if not info.get("available"):
        sys.exit(f"worldmap_shots.py: run {a.run} has no world: {info}")
    C = info["circumference"]
    port = free_port()
    profile = tempfile.mkdtemp(prefix="dt-wm-")
    proc = subprocess.Popen(
        [a.chromium, "--headless=new", "--disable-gpu", "--no-sandbox", "--hide-scrollbars",
         f"--remote-debugging-port={port}", f"--user-data-dir={profile}", "--window-size=1600,1000", "about:blank"],
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
            sys.exit("worldmap_shots.py: Chromium's DevTools endpoint did not come up")
        ws = WS(page["webSocketDebuggerUrl"])
        ws.call("Page.enable")
        ws.call("Page.addScriptToEvaluateOnNewDocument", source=RECORDER)

        def size(w, h, mobile=False):
            ws.call("Emulation.setDeviceMetricsOverride", width=w, height=h, deviceScaleFactor=1, mobile=mobile)

        def ev(js):
            r = ws.call("Runtime.evaluate", expression=js, awaitPromise=True, returnByValue=True)
            if r.get("exceptionDetails"):
                print("JS threw:", json.dumps(r["exceptionDetails"])[:300], flush=True)
                return None
            return r.get("result", {}).get("value")

        def shot(name):
            d = ws.call("Page.captureScreenshot", format="png")
            p = os.path.join(a.out, name)
            with open(p, "wb") as f:
                f.write(base64.b64decode(d["data"]))
            print(f"shot {p}", flush=True)

        def wait(js, secs=60, what="condition"):
            until = time.time() + secs
            while time.time() < until:
                if ev(js):
                    return True
                time.sleep(0.3)
            print(f"timed out waiting for {what}", flush=True)
            return False

        def settled():
            # The address follows the view 0.5 s after it settles; tiles need a moment more.
            time.sleep(1.6)
            return ev(STATE)

        def click(sel):
            return ev(f"(() => {{ const b = document.querySelector({json.dumps(sel)}); if (!b) return false; b.click(); return true; }})()")

        def drag(x0, y0, x1, y1, steps=12):
            ws.call("Input.dispatchMouseEvent", type="mouseMoved", x=x0, y=y0, button="none")
            ws.call("Input.dispatchMouseEvent", type="mousePressed", x=x0, y=y0, button="left", clickCount=1)
            for i in range(1, steps + 1):
                ws.call("Input.dispatchMouseEvent", type="mouseMoved", x=x0 + (x1 - x0) * i / steps,
                        y=y0 + (y1 - y0) * i / steps, button="left", buttons=1)
                time.sleep(0.02)
            ws.call("Input.dispatchMouseEvent", type="mouseReleased", x=x1, y=y1, button="left", clickCount=1)

        ready = "(() => { const r = document.getElementById('wv-readout'); return !!r && /a pixel|pixels a block/.test(r.textContent) && !!new URLSearchParams(location.search).get('world'); })()"

        # 1. Opening the view: the whole planet.
        size(1600, 1000)
        ws.call("Page.navigate", url=f"{a.url}?run={a.run}")
        check(wait("!!document.getElementById('worldbtn') && !!window.DTWorldMap", 60, "the page"), "the page loads the map core and the plug-in")
        time.sleep(2)
        click("#worldbtn")
        check(wait(ready, 60, "the world view"), "the Final world view opens and states its scale")
        s = settled()
        v = parse_world(s and s["world"])
        shot("01-whole-world.png")
        home_bpp = v["bpp"] if v else 0
        check(bool(v) and v["bpp"] * s["w"] >= C * 0.995,
              f"it opens on the whole planet: {v and v['bpp']} blocks a pixel x {s and s['w']:.0f} px >= the circumference {C}")
        check(bool(v) and s["ink"][0] > 50 and s["ink"][1] > 50, f"the map is drawn across the canvas (ink {s and s['ink']})")

        # 2. Zoom controls and the magnification limit.
        click("#wv-in")
        s = settled()
        v = parse_world(s["world"])
        check(bool(v) and abs(v["bpp"] - home_bpp / 2) <= 0.002 * home_bpp + 0.001, f"+ halves the blocks a pixel ({home_bpp} to {v and v['bpp']})")
        click("#wv-home")
        s = settled()
        v = parse_world(s["world"])
        check(bool(v) and abs(v["bpp"] - home_bpp) <= 0.002 * home_bpp + 0.001, f"Whole world goes back ({v and v['bpp']})")
        for _ in range(16):
            click("#wv-in")
            time.sleep(0.05)
        s = settled()
        v = parse_world(s["world"])
        check(bool(v) and abs(v["bpp"] - 0.25) < 1e-9, f"the zoom stops at 4 screen pixels a block ({v and v['bpp']} blocks a pixel)")
        shot("02-magnified.png")

        # 3. Across the seam: a view just west of it, dragged west, wraps x onto the planet.
        x0 = C / 2 - 300
        ws.call("Page.navigate", url=f"{a.url}?run={a.run}&world=sat/{int(x0)}/0/4")
        check(wait(ready, 60, "the linked view"), "a link opens the view by itself")
        s = settled()
        v = parse_world(s["world"])
        check(bool(v) and v["layer"] == "sat" and abs(v["cx"] - int(x0)) <= 1 and abs(v["bpp"] - 4) < 1e-9 and abs(v["cz"]) <= 1,
              f"the link's view is the view (sat/{int(x0)}/0/4 -> {s['world']})")
        cx0, cy0 = 800, 560
        drag(cx0, cy0, cx0 - 300, cy0)
        s = settled()
        v = parse_world(s["world"])
        want = x0 + 300 * 4 - C
        check(bool(v) and abs(v["cx"] - want) <= 8, f"dragging 300 px west across the seam wraps x onto the planet ({s['world']}, expected x near {want:.0f})")
        check(s["ink"][0] > 50 and s["ink"][1] > 50, f"both sides of the seam are drawn (ink {s['ink']})")
        shot("03-across-the-seam.png")

        # 4. A link to a layer; a reload keeps the view.
        ws.call("Page.navigate", url=f"{a.url}?run={a.run}&world=elev/2400/0/6")
        check(wait(ready, 60, "the elevation link"), "a link to the elevation layer opens")
        s = settled()
        check(s["world"] == "elev/2400/0/6", f"the elevation link keeps its layer and view ({s['world']})")
        pressed = ev("(() => { const b = document.querySelector('#wv-layers [aria-pressed=\"true\"]'); return b ? b.dataset.l : null; })()")
        check(pressed == "elev", f"the elevation button is pressed ({pressed})")
        shot("04-elevation-link.png")
        ws.call("Page.reload")
        check(wait(ready, 60, "the reloaded view"), "a reload opens the view again")
        s = settled()
        check(s["world"] == "elev/2400/0/6", f"a reload keeps the view ({s['world']})")

        # 5. Escape closes it.
        ev("document.getElementById('wv-canvas').focus()")
        ws.call("Input.dispatchKeyEvent", type="keyDown", key="Escape", code="Escape", windowsVirtualKeyCode=27)
        ws.call("Input.dispatchKeyEvent", type="keyUp", key="Escape", code="Escape", windowsVirtualKeyCode=27)
        s = settled()
        check(not s["open"] and not s["world"], f"Escape closes the view and drops it from the address (open {s['open']}, world {s['world']})")
        errs = list(s["errs"])

        # 6. Phone width.
        size(390, 844, mobile=True)
        ws.call("Page.navigate", url=f"{a.url}?run={a.run}")
        wait("!!document.getElementById('worldbtn')", 60, "the page at phone width")
        time.sleep(2)
        click("#worldbtn")
        check(wait(ready, 60, "the world view at phone width"), "the view opens at phone width")
        s = settled()
        check(0 < s["w"] <= 390 and s["scrollW"] <= s["innerW"], f"at 390 px it fits with no sideways scroll (canvas {s['w']:.0f} px, page {s['scrollW']} of {s['innerW']})")
        check(s["h"] >= 300 and sum(s["ink"]) > 0, f"at 390 px the map has room and draws the planet (canvas {s['w']:.0f} x {s['h']:.0f} px, ink {s['ink']})")
        shot("05-phone.png")
        errs += s["errs"]
        check(not errs, f"no script errors ({errs[:3]})")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        shutil.rmtree(profile, ignore_errors=True)
    print(f"{len(failed)} failed" if failed else "all passed", flush=True)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
