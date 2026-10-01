#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Checks and screenshots of the workbench's expandable Inspector (task "Inspector fullscreen",
2026-10-01). Headless Chromium over the DevTools protocol (tools/viewer/shot.py's WebSocket
client), against a private `deeptime watch` with a finished, baked run (never Ben's on 8642):

    python3 tools/workbench/inspector_shots.py --url http://127.0.0.1:PORT/ --run RUN_ID --out DIR
        [--chromium /path/to/chromium] [--size 1600x1000]

Takes the side panel and the expanded inspector on every tab, checks that tab, scroll and the
selected event survive an expand and a collapse, the open and close paths (button, P, Esc, scrim),
the focus (returns to the button, Tab stays inside), a phone-width page (no button, the mobile tab
layout intact) and ?tv (no button), and that the page threw nothing. Prints PASS/FAIL lines and
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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", required=True)
    ap.add_argument("--run", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--chromium", default=os.environ.get("DEEPTIME_CHROME") or shutil.which("chromium"))
    ap.add_argument("--size", default="1600x1000")
    a = ap.parse_args()
    if not a.chromium:
        sys.exit("inspector_shots.py: no Chromium (pass --chromium or set DEEPTIME_CHROME)")
    w, h = (int(v) for v in a.size.split("x"))
    os.makedirs(a.out, exist_ok=True)
    port = shot.free_port()
    profile = tempfile.mkdtemp(prefix="dt-insp-")
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
            sys.exit("inspector_shots.py: Chromium's DevTools endpoint did not come up")
        ws = shot.WS(page["webSocketDebuggerUrl"])
        ws.call("Page.enable")
        # Everything the page throws or logs as an error, kept in window.__errs across navigations.
        ws.call("Page.addScriptToEvaluateOnNewDocument", source="""
          window.__errs = [];
          addEventListener('error', e => window.__errs.push('error: ' + e.message));
          addEventListener('unhandledrejection', e => window.__errs.push('rejection: ' + (e.reason && e.reason.message || e.reason)));
          const ce = console.error; console.error = (...a) => { window.__errs.push('console.error: ' + a.join(' ')); ce.apply(console, a); };
        """)
        js_errors = []

        def size(sw, sh):
            ws.call("Emulation.setDeviceMetricsOverride", width=sw, height=sh, deviceScaleFactor=1, mobile=False)

        def ev(js):
            r = ws.call("Runtime.evaluate", expression=js, awaitPromise=True, returnByValue=True)
            if r.get("exceptionDetails"):
                js_errors.append(js[:80])
                print("JS threw:", json.dumps(r["exceptionDetails"])[:300], flush=True)
                return None
            return r.get("result", {}).get("value")

        def snap(name):
            d = ws.call("Page.captureScreenshot", format="png")
            with open(os.path.join(a.out, name), "wb") as f:
                f.write(base64.b64decode(d["data"]))
            print(f"shot {name}", flush=True)

        def key(k, code=None, vk=0, text=None, shift=False):
            mod = 8 if shift else 0
            ws.call("Input.dispatchKeyEvent", type="keyDown", key=k, code=code or k, windowsVirtualKeyCode=vk, text=text or "", modifiers=mod)
            ws.call("Input.dispatchKeyEvent", type="keyUp", key=k, code=code or k, windowsVirtualKeyCode=vk, modifiers=mod)

        def esc():
            key("Escape", "Escape", 27)
            time.sleep(0.3)

        def mouse(kind, x, y, button="none", clicks=0):
            ws.call("Input.dispatchMouseEvent", type=kind, x=x, y=y, button=button, clickCount=clicks)

        def click(x, y):
            mouse("mouseMoved", x, y)
            mouse("mousePressed", x, y, "left", 1)
            mouse("mouseReleased", x, y, "left", 1)

        def wait(js, secs=60, what="condition"):
            until = time.time() + secs
            while time.time() < until:
                if ev(js):
                    return True
                time.sleep(0.4)
            print(f"timed out waiting for {what}", flush=True)
            return False

        def state():
            return ev("""(() => { const c = document.getElementById('inspector'), r = c.getBoundingClientRect();
              const vis = id => !document.getElementById(id).hidden;
              return { big: c.classList.contains('big'), x: r.left, y: r.top, w: r.width, h: r.height, vw: innerWidth, vh: innerHeight,
                tab: (document.querySelector('[data-itab][aria-selected=true]') || {}).dataset.itab,
                panes: ['events','details','s2','section','run'].filter(k => vis('ip-' + k)),
                exp: document.getElementById('inspbig').getAttribute('aria-expanded'), label: document.getElementById('inspbig').getAttribute('aria-label'),
                focus: document.activeElement && document.activeElement.id, scrim: !document.getElementById('inspscrim').hidden,
                role: c.getAttribute('role') }; })()""")

        def tab(name):
            if not state()["big"]:
                ev("document.getElementById('inspector').scrollIntoView({block: 'end'})")
            ev(f"document.querySelector('[data-itab=\"{name}\"]').click()")
            time.sleep(0.5)

        def big():
            ev("document.getElementById('inspbig').click()")
            time.sleep(0.6)

        size(w, h)
        ws.call("Page.navigate", url=f"{a.url}?run={a.run}")
        check(wait(f"fetch('api/runs').then(r => r.json()).then(j => (j.runs.find(r => r.id === {json.dumps(a.run)}) || {{}}).state === 'done')", 1800, "the run to finish"), "the run is done")
        check(wait("document.querySelectorAll('#ticker .evrow').length > 3", 120, "events in the list"), "the Events tab lists the run's events")
        time.sleep(2.5)

        ev("document.getElementById('inspector').scrollIntoView({block: 'end'})")
        s = state()
        check(s["exp"] == "false" and s["label"] == "Expand the inspector" and not s["big"], f"the Inspector header has an Expand button (aria-label {s['label']!r}, collapsed)")
        side_w = s["w"]
        snap("01-events-collapsed.png")

        # State to carry across an expand: a scrolled list and a picked event.
        ev("document.getElementById('ticker').scrollTop = 120")
        top0 = ev("document.getElementById('ticker').scrollTop")
        rp0 = ev("document.getElementById('rpanel').scrollTop")
        # Real keyboard use: focus the button and press Enter.
        ev("document.getElementById('inspbig').focus()")
        key("Enter", "Enter", 13, "\r")
        time.sleep(0.6)
        s = state()
        check(s["big"] and s["exp"] == "true" and s["scrim"] and s["role"] == "dialog", "Enter on the focused button expands the inspector (dialog, scrim)")
        check(s["w"] >= s["vw"] - 60 and s["h"] >= s["vh"] - 60 and s["w"] > side_w * 3, f"it covers most of the window ({s['w']:.0f} x {s['h']:.0f} of {s['vw']} x {s['vh']}; side panel was {side_w:.0f} wide)")
        rp1 = ev("document.getElementById('rpanel').scrollTop"); check(rp1 == rp0, f"the side panel behind keeps its scroll position ({rp0} -> {rp1})")
        check(s["label"] == "Close the expanded inspector" and s["focus"] == "inspbig", "the button turns into Close and keeps focus")
        check(s["tab"] == "events" and s["panes"] == ["events", "details"], f"Events tab shows the list and the details side by side (panes {s['panes']})")
        check(abs((ev("document.getElementById('ticker').scrollTop") or 0) - (top0 or 0)) < 2, f"list scroll is kept (was {top0})")
        snap("02-events-expanded-nothing-picked.png")

        # Pick an event in the expanded list: it stays on the Events tab with the details beside it.
        ev("document.querySelectorAll('#ticker .evrow')[2].click()")
        time.sleep(0.6)
        s = state()
        sel = ev("document.querySelector('#ticker .evrow.sel') && document.querySelector('#ticker .evrow.sel').dataset.ev")
        check(s["tab"] == "events" and s["panes"] == ["events", "details"] and bool(sel) and "caused by" in (ev("document.getElementById('details').innerText") or "").lower(),
              "picking an event keeps the list and shows its causes beside it")
        snap("03-events-expanded-picked.png")
        esc()
        s = state()
        check(not s["big"] and s["focus"] == "inspbig" and not s["scrim"], "Esc closes it and focus returns to the button")
        check(ev("document.querySelector('#ticker .evrow.sel') && document.querySelector('#ticker .evrow.sel').dataset.ev") == sel, "the selected event is kept")
        check(abs((ev("document.getElementById('ticker').scrollTop") or 0) - (top0 or 0)) < 2, "list scroll is kept after collapsing")
        rp2 = ev("document.getElementById('rpanel').scrollTop")
        check(rp2 == rp0 and not ev("!!document.getElementById('inspghost')"), f"the side panel is as it was (scroll {rp0} -> {rp2}, no stand-in left)")
        check(s["tab"] == "events" and "caused by" in (ev("document.getElementById('details').innerText") or "").lower(),
              "collapsed again it is on the Events tab with the pick selected, and the Details tab has its causes")
        tab("details")
        snap("04-details-collapsed.png")

        # Every tab, collapsed then expanded.
        # The 3D tab needs a job card: inspect a column on the map (answered from the bake).
        ev("document.getElementById('tool-inspect').click()")
        r = ev("(() => { const r = document.querySelector('#paneA canvas.map').getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; })()")
        click(r[0] + r[2] * 0.42, r[1] + r[3] * 0.40)
        check(wait("document.querySelectorAll('#jobs .job').length > 0", 60, "a job card"), "an inspect click makes a job card on the 3D tab")
        wait("!!(document.querySelector('#jobs [data-jact=details]') || document.querySelector('#jobs .callout'))", 60, "the job to answer")
        ev("document.getElementById('tool-select').click()")
        # A section for the Section tab.
        ev("document.getElementById('tool-section').click()")
        click(r[0] + r[2] * 0.30, r[1] + r[3] * 0.42)
        time.sleep(0.4)
        click(r[0] + r[2] * 0.52, r[1] + r[3] * 0.50)
        wait("!!(document.querySelector('#ip-section .sv-grid, #ip-section [data-sec=big]') || document.querySelector('.secdock canvas'))", 60, "the section")
        time.sleep(1.5)
        ev("document.getElementById('tool-select').click()")
        # A stand-in preview card (the real ones need the Minecraft server): an iframe in a real .embed.
        ev("""(() => { const j = document.createElement('div'); j.className = 'job'; j.id = 'fakejob';
          j.innerHTML = '<div class="jh"><div class="jt"><div class="a">Stand-in 3D preview</div><div class="b">for the layout check</div></div></div><div class="embed"><iframe title="3D preview" srcdoc="<body style=\\'margin:0;background:linear-gradient(135deg,#1a2740,#0b0f15)\\'><p style=\\'color:#9fb0ff;font:14px sans-serif;padding:12px\\'>iframe fills the embed area</p>"></iframe></div>';
          document.getElementById('jobs').appendChild(j); })()""")

        names = {"events": "events", "details": "details", "s2": "3d", "section": "section", "run": "run"}
        for t, n in names.items():
            tab(t)
            s = state()
            check(s["tab"] == t and t in s["panes"], f"{t}: the tab opens collapsed")
            snap(f"10-{n}-collapsed.png")
            big()
            s = state()
            check(s["big"] and s["tab"] == t, f"{t}: expanding keeps the tab")
            if t == "s2":
                emb = ev("(() => { const e = document.querySelector('#fakejob .embed'), i = e.querySelector('iframe'), c = document.getElementById('inspector').getBoundingClientRect(), er = e.getBoundingClientRect(), ir = i.getBoundingClientRect(); return { ew: er.width, eh: er.height, iw: ir.width, ih: ir.height, cw: c.width }; })()")
                check(emb["eh"] >= 380 and emb["ew"] > 560 and abs(emb["iw"] - emb["ew"]) < 3 and abs(emb["ih"] - emb["eh"]) < 3,
                      f"3D: the preview iframe fills a large embed ({emb['iw']:.0f} x {emb['ih']:.0f}; was 210 high)")
                check(len(ev("[...document.querySelectorAll('#jobs .job')].map(j => j.id)")) >= 2, "3D: the job cards are kept")
            if t == "section":
                sc = ev("(() => { const c = document.querySelector('#ip-section .sec-inline canvas'); if (!c) return null; const r = c.getBoundingClientRect(); return { w: r.width, h: r.height, bw: c.width, enlarge: [...document.querySelectorAll('#ip-section [data-sec=big]')].some(b => b.offsetParent) }; })()")
                check(bool(sc) and sc["w"] > 700 and sc["h"] > 300 and not sc["enlarge"], f"Section: the drawing is large in the tab, no second Enlarge ({sc})")
            snap(f"11-{n}-expanded.png")
            big()
            s = state()
            check(not s["big"] and s["tab"] == t and s["focus"] == "inspbig", f"{t}: collapsing keeps the tab and focus returns to the button")
        # Section drawn again small after collapsing.
        tab("section")
        check(ev("!!document.querySelector('#ip-section .sec-inline')") is False and ev("!!document.querySelector('#ip-section [data-sec=big]')") is True, "Section: collapsed, the panel is as before (with its Enlarge button)")
        ev("document.querySelector('#ip-section [data-sec=big]').click()")
        time.sleep(0.8)
        check(ev("!document.getElementById('modal').hidden && !!document.querySelector('#sec-big-body canvas')") is True, "the existing Enlarge dialog still opens")
        esc()
        check(ev("document.getElementById('modal').hidden") is True, "and Esc closes it")

        # The P shortcut, Esc, the scrim, the dialog on top of the expanded card.
        ev("document.activeElement && document.activeElement.blur()")
        key("p", "KeyP", 80, "p")
        time.sleep(0.5)
        check(state()["big"], "P expands the inspector")
        key("p", "KeyP", 80, "p")
        time.sleep(0.5)
        check(not state()["big"], "P again puts it back")
        key("p", "KeyP", 80, "p")
        time.sleep(0.4)
        # Tab stays inside the card.
        inside = True
        for _ in range(45):
            key("Tab", "Tab", 9)
            inside &= bool(ev("document.getElementById('inspector').contains(document.activeElement)"))
        check(inside, "45 Tab presses never leave the expanded inspector")
        inside = True
        for _ in range(12):
            key("Tab", "Tab", 9, shift=True)
            inside &= bool(ev("document.getElementById('inspector').contains(document.activeElement)"))
        check(inside, "Shift+Tab wraps inside it too")
        # A dialog from inside (Enlarge) stacks above it and closes first.
        tab("section")
        ev("document.getElementById('modal').hidden = true")
        ev("window.DTWB.modal('Test dialog', '<p>x</p>', '', true)")
        time.sleep(0.3)
        z = ev("(() => { const m = document.getElementById('modal'); const r = m.getBoundingClientRect(); const e = document.elementFromPoint(r.width / 2, r.height / 2); return !!e && m.contains(e); })()")
        check(z, "a dialog opened over the expanded inspector is on top")
        esc()
        check(state()["big"] and ev("document.getElementById('modal').hidden") is True, "Esc closes the dialog first, the inspector stays")
        mouse("mousePressed", 8, 8, "left", 1); mouse("mouseReleased", 8, 8, "left", 1)
        time.sleep(0.4)
        check(not state()["big"], "a click on the dimmed window closes it")
        ev("document.activeElement && document.activeElement.blur()")
        key("?", "Slash", 191, "?", shift=True)
        time.sleep(0.4)
        keys = ev("document.getElementById('modalbody').innerText") or ""
        check("Pop the inspector out" in keys, "the keyboard help lists the P shortcut")
        snap("20-help.png")
        esc()

        errs = ev("window.__errs") or []
        check(not errs and not js_errors, f"no page errors on the desktop page ({errs[:3]})")

        # Phone width: no button, the mobile tab layout as before.
        size(390, 844)
        ws.call("Page.navigate", url=f"{a.url}?run={a.run}")
        wait("document.querySelectorAll('#ticker .evrow').length > 0", 60, "events")
        time.sleep(2)
        ev("document.querySelector('#mtabs [data-mtab=inspect], #mtabs button[data-tab=inspect]') && document.querySelector('#mtabs [data-mtab=inspect], #mtabs button[data-tab=inspect]').click()")
        ev("window.DTWB.setMTab('inspect')")
        time.sleep(0.6)
        m = ev("""(() => { const b = document.getElementById('inspbig'), c = document.getElementById('inspector'), r = c.getBoundingClientRect();
          return { btn: getComputedStyle(b).display, shown: getComputedStyle(c).display, w: r.width, vw: innerWidth, sw: document.documentElement.scrollWidth, pos: getComputedStyle(c).position }; })()""")
        check(m["btn"] == "none" and m["shown"] != "none" and m["pos"] != "fixed" and m["sw"] <= m["vw"], f"phone: no expand button, the inspector tab shows in the page, no sideways scroll ({m})")
        key("p", "KeyP", 80, "p")
        time.sleep(0.4)
        check(ev("document.getElementById('inspector').classList.contains('big')") is False, "phone: P does nothing")
        snap("30-phone-inspector.png")
        ev("document.querySelector('[data-itab=\"run\"]').click()")
        time.sleep(0.3)
        snap("31-phone-run-tab.png")
        # Narrowing while expanded closes it.
        size(w, h)
        time.sleep(0.8)
        big()
        check(state()["big"], "back at desktop width, expanding works")
        size(390, 844)
        time.sleep(0.8)
        check(ev("document.getElementById('inspector').classList.contains('big')") is False, "narrowing the window while expanded puts the inspector back")
        check(not (ev("window.__errs") or []), "no page errors on the phone page")

        # ?tv is left alone.
        size(w, h)
        ws.call("Page.navigate", url=f"{a.url}?run={a.run}&tv")
        time.sleep(3)
        t = ev("""(() => ({ btn: getComputedStyle(document.getElementById('inspbig')).display, panel: getComputedStyle(document.getElementById('rpanel')).display }))()""")
        key("p", "KeyP", 80, "p")
        time.sleep(0.4)
        check(t["btn"] == "none" and ev("document.getElementById('inspector').classList.contains('big')") is False, f"?tv: no button, P does nothing ({t})")
        check(not (ev("window.__errs") or []), "no page errors in ?tv")
        snap("40-tv.png")
    finally:
        proc.terminate()
        shutil.rmtree(profile, ignore_errors=True)
    print(f"\n{'FAILED: ' + str(len(failed)) if failed else 'all checks passed'}")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
