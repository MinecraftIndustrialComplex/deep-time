#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Headless Chromium screenshots of the P1-15 BlueMap render (run on the test VM):

    python3 tools/review/p1-15-bluemap-shots.py <profile>

Reads review/out/p1-15-real/<profile>/bluemap/web, writes shots/*.png beside it. The views are the
boxes of tools/review/p1-15-bluemap.sh: the range's flank (surface), sites 19 and 20 (surface and the
cut at y 25), site 6 (surface and the cut at y -440)."""
import functools
import http.server
import os
import shutil
import subprocess
import sys
import threading

profile = sys.argv[1]
web = f"review/out/p1-15-real/{profile}/bluemap/web"
shots = f"review/out/p1-15-real/{profile}/bluemap/shots"
os.makedirs(shots, exist_ok=True)
VIEWS = [
    ("arc-range-surface", "planet", "2000:70:-1725:700:0.5:0.85:0:0:perspective"),
    ("site19-20-surface", "planet", "2050:60:-1370:380:0.5:0.85:0:0:perspective"),
    ("site19-20-cut-y25", "underground", "2050:25:-1370:160:0:0:0:1:flat"),
    ("site6-placer4-surface", "planet", "2250:95:-1600:420:0.5:0.85:0:0:perspective"),
    ("site6-cut-y-440", "underground", "2250:-440:-1600:110:0:0:0:1:flat"),
]


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=web))
threading.Thread(target=srv.serve_forever, daemon=True).start()
port = srv.server_address[1]
chromium = ["chromium"] if shutil.which("chromium") else ["nix", "shell", "nixpkgs#chromium", "-c", "chromium"]
for stem, mid, cam in VIEWS:
    png = f"{shots}/{stem}.png"
    cmd = chromium + ["--headless=new", "--no-sandbox", "--hide-scrollbars", "--use-angle=swiftshader",
                      "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "--window-size=1280,860",
                      "--virtual-time-budget=20000", f"--screenshot={png}",
                      f"http://127.0.0.1:{port}/index.html#{mid}:{cam}"]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=900)
        err = r.stderr[-200:]
    except subprocess.TimeoutExpired:
        err = "timed out"
    print(f"{stem}: {'ok' if os.path.exists(png) else 'FAILED ' + err}", flush=True)
