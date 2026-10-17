#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Screenshots of a BlueMap web app (tools/review/render-bluemap.sh's <out>/web), for the Phase 1 exit's scale
comparison and its geology check. Headless Chromium over the DevTools protocol (tools/viewer/shot.py's client),
the app served from a private loopback HTTP server; run it on the VM (tools/remote/mac.sh):

    python3 tools/review/phase1_shots.py --web <out>/web --map planet --out <dir>/<prefix>
        --view NAME:X:Y:Z:DISTANCE:ANGLE[:flat]  [--view …]  [--size 1400x900] [--wait 60]

A view is a BlueMap camera hash: perspective at DISTANCE blocks with the camera ANGLE (radians from straight
down, 0 = flat view straight down, ~1.2 = nearly horizontal) over the point X,Y,Z, or `flat` for BlueMap's
top-down orthographic view. The picture is cut to BlueMap's map area (its buttons and the status bar dropped).
Prints one `shot: <file>` line per view; exits 1 if a picture is missing or blank.
"""

import argparse
import base64
import functools
import http.server
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "viewer"))
from shot import WS, ensure_fonts, free_port  # noqa: E402


def chromium():
    for c in ("chromium", "chromium-browser", "google-chrome"):
        if shutil.which(c):
            return [c]
    return ["nix", "shell", "nixpkgs#chromium", "-c", "chromium"]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--web", required=True)
    ap.add_argument("--map", default="planet")
    ap.add_argument("--out", required=True, help="path prefix: <out>-<view name>.png")
    ap.add_argument("--view", action="append", required=True)
    ap.add_argument("--size", default="1400x900")
    ap.add_argument("--wait", type=float, default=60.0, help="seconds for the tiles to load before the picture")
    a = ap.parse_args()
    w, h = (int(v) for v in a.size.split("x"))
    os.makedirs(os.path.dirname(os.path.abspath(a.out)), exist_ok=True)

    class Quiet(http.server.SimpleHTTPRequestHandler):
        def log_message(self, *args):
            pass

    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=a.web))
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{srv.server_address[1]}/index.html"
    ensure_fonts()   # a rebuilt VM has no fonts: Chromium aborts without them
    port = free_port()
    profile = tempfile.mkdtemp(prefix="dt-p1shot-")
    proc = subprocess.Popen(
        chromium() + ["--headless=new", "--no-sandbox", "--hide-scrollbars", "--use-angle=swiftshader", "--enable-unsafe-swiftshader",
                      "--ignore-gpu-blocklist", f"--remote-debugging-port={port}", f"--user-data-dir={profile}", f"--window-size={w},{h}", "about:blank"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    bad = 0
    try:
        for _ in range(300):
            try:
                tgt = next(t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list", timeout=2)) if t.get("type") == "page")
                break
            except Exception:
                time.sleep(0.5)
        else:
            sys.exit("phase1_shots: Chromium's DevTools endpoint did not come up")
        ws = WS(tgt["webSocketDebuggerUrl"])
        ws.call("Emulation.setDeviceMetricsOverride", width=w, height=h, deviceScaleFactor=1, mobile=False)
        ws.call("Page.enable")
        for spec in a.view:
            parts = spec.split(":")
            name, x, y, z, dist, ang = parts[:6]
            flat = len(parts) > 6 and parts[6] == "flat"
            cam = (f"{x}:{y}:{z}:{dist}:0:0:0:1:flat" if flat else f"{x}:{y}:{z}:{dist}:0.6:{ang}:0:0:perspective")
            # A fresh load per view: BlueMap restores its last camera from the hash on load only.
            ws.call("Page.navigate", url="about:blank")
            time.sleep(1)
            ws.call("Page.navigate", url=f"{base}#{a.map}:{cam}")
            time.sleep(a.wait)
            data = ws.call("Page.captureScreenshot", format="png", clip={"x": 0, "y": 48, "width": w, "height": h - 48 - 70, "scale": 1})
            path = f"{a.out}-{name}.png"
            with open(path, "wb") as f:
                f.write(base64.b64decode(data["data"]))
            size = os.path.getsize(path)
            ok = size > 20000   # a blank (all-sky or all-black) frame compresses far below this
            bad += not ok
            print(f"shot: {path} ({size} bytes){'' if ok else ' BLANK?'}", flush=True)
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        shutil.rmtree(profile, ignore_errors=True)
        srv.shutdown()
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
