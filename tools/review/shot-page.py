#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Screenshot a review page folder the way a browser shows it: served over plain HTTP from 127.0.0.1,
loaded in headless Chromium (nixpkgs), saved as PNG. For looking at a page before publishing it.

    python3 tools/review/shot-page.py <folder> <out.png> [--width 1280] [--height 4000] [--path index.html#x]
"""
import argparse
import functools
import http.server
import shutil
import subprocess
import threading

ap = argparse.ArgumentParser()
ap.add_argument("folder")
ap.add_argument("out")
ap.add_argument("--width", type=int, default=1280)
ap.add_argument("--height", type=int, default=4000)
ap.add_argument("--path", default="index.html")
ap.add_argument("--budget", type=int, default=15000, help="Chromium virtual time budget, ms")
a = ap.parse_args()


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=a.folder))
threading.Thread(target=srv.serve_forever, daemon=True).start()
browser = next(([c] for c in ("chromium", "chromium-browser", "google-chrome") if shutil.which(c)),
               ["nix", "shell", "nixpkgs#chromium", "-c", "chromium"])
url = f"http://127.0.0.1:{srv.server_address[1]}/{a.path}"
subprocess.run(browser + ["--headless=new", "--no-sandbox", "--hide-scrollbars", "--use-angle=swiftshader",
                          "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", f"--window-size={a.width},{a.height}",
                          f"--virtual-time-budget={a.budget}", f"--screenshot={a.out}", url],
               capture_output=True, timeout=600)
srv.shutdown()
print(a.out)
