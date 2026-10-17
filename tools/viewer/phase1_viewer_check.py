#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The Phase 1 exit's viewer check (plan §10.3, "Phase 1 exit test" item 6): the viewer bundle `deeptime run
--viewer` writes shows the timelapse, the event ticker, the layers, the ore provinces and three "Why here?"
chains. Drives the bundle in headless Chromium over the DevTools protocol (stdlib only, shot.py's client),
served from a private loopback HTTP server; run it on the VM:

    python3 tools/viewer/phase1_viewer_check.py --bundle review/out/phase1/viewer --out review/out/phase1/viewer-shots
        [--chromium /path/to/chromium] [--size 1500x1000]

Prints `@check PASS|FAIL <title> :: <evidence>` lines (tools/review/phase1.py reads them), writes PNGs to
--out, and exits 1 if a check failed. The checks read the page's DOM the way a reviewer sees it (the ids of
crates/deeptime-viewer/assets/index.html and viewer.js), not the bundle's data files.
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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from shot import WS, free_port  # noqa: E402

failed = []


def check(ok, title, evidence):
    print(f"@check {'PASS' if ok else 'FAIL'} {title} :: {evidence}", flush=True)
    if not ok:
        failed.append(title)
    return ok


def find_chromium(given):
    if given:
        return [given]
    for c in ("chromium", "chromium-browser", "google-chrome"):
        if shutil.which(c):
            return [c]
    return ["nix", "shell", "nixpkgs#chromium", "-c", "chromium"]


class Page:
    def __init__(self, ws):
        self.ws = ws

    def js(self, expr):
        r = self.ws.call("Runtime.evaluate", expression=expr, awaitPromise=True, returnByValue=True)
        if r.get("exceptionDetails"):
            raise RuntimeError(f"{expr[:80]!r}: {r['exceptionDetails'].get('text')} {r['exceptionDetails'].get('exception', {}).get('description', '')[:200]}")
        return r.get("result", {}).get("value")

    def shot(self, path):
        data = self.ws.call("Page.captureScreenshot", format="png", captureBeyondViewport=True)
        with open(path, "wb") as f:
            f.write(base64.b64decode(data["data"]))

    def wait(self, expr, seconds=60):
        end = time.time() + seconds
        while time.time() < end:
            try:
                if self.js(expr):
                    return True
            except RuntimeError:
                pass
            time.sleep(0.5)
        return False


# What the keyframe canvas shows: a cheap fingerprint (non-black pixel count and a sampled hash).
CANVAS = """(() => { const c = document.getElementById('kf'); if (!c || !c.width) return null;
  const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data; let nz = 0, h = 0;
  for (let i = 0; i < d.length; i += 16) { if (d[i] + d[i+1] + d[i+2] > 30) nz++; h = (h * 31 + d[i] + 7 * d[i+1] + 13 * d[i+2]) | 0; }
  return {w: c.width, h: c.height, nonblank: nz, hash: h}; })()"""


def set_scrub(page, v):
    page.js(f"(() => {{ const s = document.getElementById('scrub'); s.value = {v}; s.dispatchEvent(new Event('input', {{bubbles: true}})); }})()")
    time.sleep(1.2)


def set_layer(page, i):
    page.js(f"(() => {{ const s = document.getElementById('kflayer'); s.selectedIndex = {i}; s.dispatchEvent(new Event('change', {{bubbles: true}})); }})()")
    time.sleep(1.2)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bundle", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--chromium")
    ap.add_argument("--size", default="1500x1000")
    a = ap.parse_args()
    w, h = (int(v) for v in a.size.split("x"))
    os.makedirs(a.out, exist_ok=True)
    if not os.path.exists(os.path.join(a.bundle, "index.html")):
        check(False, "the viewer bundle exists", f"no index.html in {a.bundle}")
        sys.exit(1)

    class Quiet(http.server.SimpleHTTPRequestHandler):
        def log_message(self, *args):
            pass

    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=a.bundle))
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    port = free_port()
    profile = tempfile.mkdtemp(prefix="dt-p1view-")
    proc = subprocess.Popen(
        find_chromium(a.chromium) + ["--headless=new", "--disable-gpu", "--no-sandbox", "--hide-scrollbars",
                                     f"--remote-debugging-port={port}", f"--user-data-dir={profile}", f"--window-size={w},{h}", "about:blank"],
        stdout=subprocess.DEVNULL, stderr=open(os.path.join(profile, "chromium.err"), "w"))
    try:
        for _ in range(300):
            try:
                targets = json.load(urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list", timeout=2))
                tgt = next(t for t in targets if t.get("type") == "page")
                break
            except Exception:
                time.sleep(0.5)
        else:
            sys.exit("phase1_viewer_check: Chromium's DevTools endpoint did not come up")
        ws = WS(tgt["webSocketDebuggerUrl"])
        ws.call("Emulation.setDeviceMetricsOverride", width=w, height=h, deviceScaleFactor=1, mobile=False)
        ws.call("Page.enable")
        ws.call("Runtime.enable")
        page = Page(ws)
        ws.call("Page.navigate", url=f"http://127.0.0.1:{srv.server_address[1]}/index.html")
        up = page.wait("document.getElementById('timelapse') && !document.getElementById('timelapse').hidden", 90)

        # 6a. the timelapse: keyframes, a scrubber that moves the picture, a play button.
        if not up:
            check(False, "the viewer shows the timelapse", "the #timelapse section never became visible (no keyframes, events or sites in the bundle?)")
        else:
            time.sleep(2)
            n = int(page.js("Number(document.getElementById('scrub').max) + 1") or 0)
            first = page.js(CANVAS)
            set_scrub(page, 0)
            f0 = page.js(CANVAS)
            l0 = page.js("document.getElementById('tlabel').textContent")
            set_scrub(page, n - 1)
            f1 = page.js(CANVAS)
            l1 = page.js("document.getElementById('tlabel').textContent")
            page.shot(os.path.join(a.out, "timelapse-last.png"))
            play = page.js("!!document.getElementById('play')")
            check(n >= 4 and f0 and f1 and f0["nonblank"] > 1000 and f1["nonblank"] > 1000 and f0["hash"] != f1["hash"] and l0 != l1 and play,
                  "the viewer shows the timelapse",
                  f"{n} keyframes; first frame '{l0}' ({f0 and f0['nonblank']} lit samples), last '{l1}' ({f1 and f1['nonblank']}); the two frames differ: "
                  f"{bool(f0 and f1 and f0['hash'] != f1['hash'])}; play button {play}")
            # 6b. the event ticker grows with time.
            set_scrub(page, 0)
            r0 = page.js("document.querySelectorAll('#ticker .row').length")
            set_scrub(page, n - 1)
            r1 = page.js("document.querySelectorAll('#ticker .row').length")
            kinds = page.js("[...new Set([...document.querySelectorAll('#ticker .row span:last-child')].map(e => e.textContent))].slice(0, 8)")
            check(r1 and r1 > (r0 or 0), "the viewer shows the event ticker",
                  f"{r0} rows at the first frame, {r1} at the last (up to 80 shown); kinds seen {kinds}")
            page.shot(os.path.join(a.out, "ticker.png"))
            # 6c. layers: the layer list, and that choosing others redraws the frame.
            names = page.js("[...document.getElementById('kflayer').options].map(o => o.textContent)") or []
            # A layer is drawn from the frames that carry it (the last keyframe holds only a few layers; a frame without
            # the chosen layer shows the first one), so try the layers on the first, a middle and the last frame and
            # take the frame that shows the most different pictures.
            best = (set(), None)
            trace = {}
            for pos in (0, (n - 1) // 2, n - 1):
                set_scrub(page, pos)
                hashes = set()
                for i in range(min(len(names), 6)):
                    set_layer(page, i)
                    fp = page.js(CANVAS)
                    end = time.time() + 20
                    while time.time() < end and not (fp and fp["nonblank"] > 1000 and fp["hash"] not in hashes):
                        time.sleep(0.5)
                        fp = page.js(CANVAS)
                    if fp and fp["nonblank"] > 1000:
                        hashes.add(fp["hash"])
                trace[pos] = len(hashes)
                if len(hashes) > len(best[0]):
                    best = (hashes, pos)
                if len(hashes) >= 3:
                    break
            hashes = best[0]
            print("layers trace (frame, different pictures over the first layers):", trace, flush=True)
            page.shot(os.path.join(a.out, "layers.png"))
            check(len(names) >= 5 and len(hashes) >= 3, "the viewer shows layers",
                  f"{len(names)} keyframe layers ({', '.join(names[:8])}…); {len(hashes)} different pictures over the first {min(len(names), 6)} layers at frame {best[1]} of {n}")
            # 6d. ore provinces: a deposit-tract / province layer, the census and the site markers.
            prov = [i for i, nme in enumerate(names) if any(k in nme.lower() for k in ("tract", "province", "favour", "prospect", "deposit"))]
            census = page.js("(() => { const c = document.getElementById('census'); return c && !c.hidden ? c.innerText.slice(0, 400) : null; })()")
            sites = page.js("document.querySelectorAll('#sites .row').length")
            drawn = None
            if prov:
                set_layer(page, prov[0])
                set_scrub(page, n - 1)
                drawn = page.js(CANVAS)
                page.shot(os.path.join(a.out, "ore-provinces.png"))
            check(bool(prov) and bool(drawn and drawn["nonblank"] > 1000) and bool(census) and (sites or 0) > 0, "the viewer shows ore provinces",
                  f"province layers {[names[i] for i in prov]}; census {'shown' if census else 'missing'} ({(census or '')[:80]!r}); {sites} sites listed")
            # 6e. three "Why here?" chains: three deposit sites, each with a non-empty chain of linked events.
            ids = page.js("""(() => { const rows = [...document.querySelectorAll('#sites .row[data-site]')];
              const seen = new Set(), out = [];
              for (const r of rows) { const t = r.textContent; const k = (t.match(/deposit · (\\S+)/) || [])[1];
                if (!k || seen.has(k)) continue; seen.add(k); out.push([Number(r.dataset.site), k]); }
              for (const r of rows) { if (out.length >= 3) break; const id = Number(r.dataset.site); if (!out.some(o => o[0] === id)) out.push([id, 'other']); }
              return out.slice(0, 3); })()""") or []
            chains = []
            for sid, kind in ids:
                page.js(f"document.querySelector('#sites .row[data-site=\"{sid}\"]').click()")
                time.sleep(0.8)
                got = page.js("""(() => { const rows = [...document.querySelectorAll('#details table.kv tr')];
                  const r = rows.find(t => t.firstChild && t.firstChild.textContent === 'why here?');
                  return r ? {links: r.querySelectorAll('a.ref').length, text: r.lastChild.textContent.slice(0, 160)} : null; })()""")
                chains.append((sid, kind, got))
                if len(chains) == 1:
                    page.shot(os.path.join(a.out, "why-here-1.png"))
            good = [c for c in chains if c[2] and c[2]["links"] >= 1]
            check(len(chains) == 3 and len(good) == 3, 'the viewer shows three "Why here?" chains',
                  "; ".join(f"site {s} ({k}): {(g or {}).get('links', 'no row')} links" for s, k, g in chains) or "no sites in the list")
            if len(chains) >= 3:
                page.js(f"document.querySelector('#sites .row[data-site=\"{chains[2][0]}\"]').click()")
                time.sleep(0.5)
                page.shot(os.path.join(a.out, "why-here-3.png"))
    except EOFError as e:
        # Chromium died under the page (a crash on a loaded box): say why instead of a bare traceback, so the step can be re-run.
        try:
            tail = open(os.path.join(profile, "chromium.err")).read()[-600:]
        except OSError:
            tail = ""
        print(f"phase1_viewer_check: {e}; chromium exit {proc.poll()}; stderr tail: {tail!r}")
        failed.append("chromium crashed")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        shutil.rmtree(profile, ignore_errors=True)
        srv.shutdown()
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
