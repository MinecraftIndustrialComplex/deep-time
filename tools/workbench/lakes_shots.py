#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Checks and screenshots of the workbench's lakes (task "climate-sized and frozen lakes",
2026-10-01). Headless Chromium over the DevTools protocol (tools/viewer/shot.py's WebSocket
client), against a private `deeptime watch` with a finished run (never Ben's on 8642):

    python3 tools/workbench/lakes_shots.py --url http://127.0.0.1:PORT/ --run RUN_ID --out DIR
        [--chromium /path/to/chromium] [--size 1600x1000]

On the Rivers & lakes layer it checks the legend (Lake, Frozen lake, Salt flat), finds a lake and,
when the planet has one, a frozen lake and a salt flat by their key colours on the map, and reads
the pointer's text over each. Then, on the Satellite layer, it puts the pointer on the same spots:
a lake must read "lake" ("frozen lake"), never "looks like shallow sea". Prints PASS/FAIL lines and
exits 1 when a check failed. Run it on the test VM, never the laptop.
"""

import argparse
import base64
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

failed = []


def check(ok, what):
    print(("PASS " if ok else "FAIL ") + what, flush=True)
    if not ok:
        failed.append(what)


# The centre of the largest patch of each lake colour on the map canvas (client coordinates).
FIND = """(() => {
  const c = document.querySelector('#paneA canvas.map'), r = c.getBoundingClientRect();
  const g = c.getContext('2d'), d = g.getImageData(0, 0, c.width, c.height).data, w = c.width, h = c.height;
  const keys = %s, out = {};
  for (const [name, rgb] of Object.entries(keys)) {
    const is = (x, y) => { const q = (y * w + x) * 4; return Math.abs(d[q] - rgb[0]) + Math.abs(d[q + 1] - rgb[1]) + Math.abs(d[q + 2] - rgb[2]) < 9; };
    let best = null;
    for (let y = 6; y < h - 6; y += 2) for (let x = 6; x < w - 6; x += 2) {
      if (!is(x, y)) continue;
      let k = 1;
      while (k < 40 && x + k < w && y + k < h && x - k >= 0 && y - k >= 0 && is(x + k, y) && is(x - k, y) && is(x, y + k) && is(x, y - k)) k++;
      if (k >= 3 && (!best || k > best.k)) best = { k, x, y };
    }
    if (best) out[name] = { x: r.left + best.x * r.width / w, y: r.top + best.y * r.height / h, k: best.k };
  }
  return out;
})()"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", required=True)
    ap.add_argument("--run", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--chromium", default=os.environ.get("DEEPTIME_CHROME") or shutil.which("chromium"))
    ap.add_argument("--size", default="1600x1000")
    a = ap.parse_args()
    if not a.chromium:
        sys.exit("lakes_shots.py: no Chromium (pass --chromium or set DEEPTIME_CHROME)")
    w, h = (int(v) for v in a.size.split("x"))
    os.makedirs(a.out, exist_ok=True)
    ramps = json.load(open(os.path.join(REPO, "crates", "deeptime-viewer", "assets", "ramps.json")))["ramps"]
    rgb = lambda s: [int(s[i:i + 2], 16) for i in (1, 3, 5)]
    keys = {k: rgb(ramps["rivers"]["keys"][k]) for k in ("lake", "frozen_lake", "salt_flat")}
    port = shot.free_port()
    profile = tempfile.mkdtemp(prefix="dt-lakes-")
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
            sys.exit("lakes_shots.py: Chromium's DevTools endpoint did not come up")
        ws = shot.WS(page["webSocketDebuggerUrl"])
        ws.call("Page.enable")
        ws.call("Page.addScriptToEvaluateOnNewDocument", source="""
          window.__errs = [];
          addEventListener('error', e => window.__errs.push('error: ' + e.message));
          addEventListener('unhandledrejection', e => window.__errs.push('rejection: ' + (e.reason && e.reason.message || e.reason)));
        """)

        def ev(js):
            r = ws.call("Runtime.evaluate", expression=js, awaitPromise=True, returnByValue=True)
            if r.get("exceptionDetails"):
                print("JS threw:", json.dumps(r["exceptionDetails"])[:300], flush=True)
                failed.append("a script threw: " + js[:60])
                return None
            return r.get("result", {}).get("value")

        def snap(name):
            d = ws.call("Page.captureScreenshot", format="png")
            with open(os.path.join(a.out, name), "wb") as f:
                f.write(base64.b64decode(d["data"]))
            print(f"shot {name}", flush=True)

        def wait(js, secs=60, what="condition"):
            until = time.time() + secs
            while time.time() < until:
                if ev(js):
                    return True
                time.sleep(0.4)
            print(f"timed out waiting for {what}", flush=True)
            return False

        def hover(x, y):
            # Twice: the satellite view's pointer loads the frame's Rivers & lakes layer on the first.
            text = ""
            for _ in range(8):
                ws.call("Input.dispatchMouseEvent", type="mouseMoved", x=x + 1, y=y, button="none", clickCount=0)
                ws.call("Input.dispatchMouseEvent", type="mouseMoved", x=x, y=y, button="none", clickCount=0)
                time.sleep(0.4)
                text = ev("(document.querySelector('#paneA .readout') || {}).innerText || ''") or ""
                if "lake" in text or "salt flat" in text:
                    break
            return " ".join(text.split())

        ws.call("Emulation.setDeviceMetricsOverride", width=w, height=h, deviceScaleFactor=1, mobile=False)
        ws.call("Page.navigate", url=f"{a.url}?run={a.run}")
        check(wait(f"fetch('api/runs').then(r => r.json()).then(j => (j.runs.find(r => r.id === {json.dumps(a.run)}) || {{}}).state === 'done')", 3600, "the run to finish"), "the run is done")
        check(wait("!!document.querySelector('[data-layer=\"rivers\"]')", 120, "the layer list"), "the run has a Rivers & lakes layer")
        time.sleep(3)

        ev("document.querySelector('[data-layer=\"rivers\"]').click()")
        time.sleep(3)
        legend = ev("[...document.querySelectorAll('.legend .cats div')].map(d => d.textContent.trim())") or []
        check(all(k in legend for k in ("Lake", "Frozen lake", "Salt flat")), f"the Rivers & lakes legend lists Lake, Frozen lake and Salt flat ({legend})")
        snap("workbench-rivers-layer.png")
        spots = ev(FIND % json.dumps(keys)) or {}
        print("found on the map:", json.dumps(spots), flush=True)
        check("lake" in spots, "the planet has a lake to point at")
        names = {"lake": "lake", "frozen_lake": "frozen lake", "salt_flat": "salt flat"}
        for k, s in spots.items():
            text = hover(s["x"], s["y"])
            check(text.endswith(names[k]), f"Rivers & lakes layer: the pointer on a {names[k]} reads \"{text}\"")

        ev("document.querySelector('[data-layer=\"satellite\"]').click()")
        time.sleep(3)
        snap("workbench-satellite.png")
        for k, s in spots.items():
            if k == "salt_flat":
                continue  # drawn in the satellite view's own Salt flat class colour, which reads as it did
            text = hover(s["x"], s["y"])
            check(text.endswith(names[k]) and "looks like" not in text, f"Satellite view: the pointer on a {names[k]} reads \"{text}\"")
        if "frozen_lake" not in spots:
            print("note: this planet has no frozen lake large enough to point at", flush=True)
        # Dry land still reads as what it looks like.
        r = ev("(() => { const r = document.querySelector('#paneA canvas.map').getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; })()")
        seen = set()
        for fx in (0.1, 0.3, 0.5, 0.7, 0.9):
            for fy in (0.3, 0.5, 0.7):
                ws.call("Input.dispatchMouseEvent", type="mouseMoved", x=r[0] + r[2] * fx, y=r[1] + r[3] * fy, button="none", clickCount=0)
                time.sleep(0.15)
                t = " ".join((ev("(document.querySelector('#paneA .readout') || {}).innerText || ''") or "").split())
                seen.add(t.split("Satellite")[-1].strip())
        check(any(t.startswith("looks like") for t in seen), f"elsewhere the satellite view still reads what the spot looks like ({sorted(seen)[:6]})")
        errs = ev("window.__errs") or []
        check(not errs, f"the page threw nothing ({errs[:3]})")
    finally:
        proc.terminate()
        shutil.rmtree(profile, ignore_errors=True)
    print("lakes_shots.py:", "FAILED: " + "; ".join(failed) if failed else "all checks passed", flush=True)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
