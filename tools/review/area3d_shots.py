#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Screenshots of area exports (`.dtva.gz`) in the area3d viewer, headless, on the Mac.

    python3 tools/review/area3d_shots.py OUT_PREFIX FILE.dtva.gz[:name[:az,el,zoom]]...

Serves tools/review/pages/area3d/ with the exports on a loopback port and has Chromium's headless
shell (playwright's, in ~/Library/Caches/ms-playwright on the Mac; DEEPTIME_CHROME overrides)
screenshot `index.html?src=…&cam=az,el,zoom&theme=light` into OUT_PREFIX-<name>.png. Run it on the
Mac through tools/remote/mac.sh (never on the laptop: headless Chromium is heavy). Standard library
only.
"""

import functools
import glob
import http.server
import os
import shutil
import subprocess
import sys
import tempfile
import threading

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def chrome() -> str:
    if os.environ.get("DEEPTIME_CHROME"):
        return os.environ["DEEPTIME_CHROME"]
    found = sorted(glob.glob(os.path.expanduser(
        "~/Library/Caches/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-*/chrome-headless-shell")))
    if not found:
        sys.exit("no chrome-headless-shell found; set DEEPTIME_CHROME")
    return found[-1]


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    prefix = sys.argv[1]
    d = tempfile.mkdtemp(prefix="a3d-")
    for f in ("index.html", "area3d.js"):
        shutil.copy(os.path.join(REPO, "tools/review/pages/area3d", f), d)
    jobs = []
    for i, arg in enumerate(sys.argv[2:]):
        parts = arg.split(":")
        name = parts[1] if len(parts) > 1 else str(i)
        cam = parts[2] if len(parts) > 2 else "35,28,1"
        shutil.copy(parts[0], os.path.join(d, f"a{i}.dtva.gz"))
        jobs.append((f"a{i}.dtva.gz", name, cam))
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=d))
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    port = srv.server_address[1]
    exe = chrome()
    status = 0
    for src, name, cam in jobs:
        png = os.path.abspath(f"{prefix}-{name}.png")
        url = f"http://127.0.0.1:{port}/index.html?src={src}&cam={cam}&theme=light"
        cmd = [exe, "--headless", "--no-sandbox", "--hide-scrollbars", "--use-angle=swiftshader",
               "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "--window-size=1600,1100",
               "--virtual-time-budget=60000", f"--screenshot={png}", url]
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=900)
        if os.path.exists(png):
            print(png)
        else:
            print(f"failed {png}: {r.stderr[-400:]}")
            status = 1
    srv.shutdown()
    shutil.rmtree(d, ignore_errors=True)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
