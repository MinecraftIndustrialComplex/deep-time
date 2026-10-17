#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Screenshot a live page after it has run for a while (the workbench, `deeptime watch`).

    nix shell nixpkgs#chromium nixpkgs#python3 -c python3 tools/viewer/shot.py URL OUT.png
        [--wait SECONDS] [--size 1600x900] [--eval JS]...

Chromium's own `--screenshot` fires at the page's load event, before a streamed page has drawn
anything, and `--virtual-time-budget` never runs out while an EventSource is open. This drives
headless Chromium over the DevTools protocol instead (stdlib only: a minimal WebSocket client):
open the URL, wait, optionally run JS (e.g. click a control), wait again, capture.
"""

import argparse
import base64
import json
import os
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time
import urllib.request


def ensure_fonts():
    """The rebuilt test VM has no fonts and no fontconfig file: Chromium aborts when the viewer page lays out text ("Could not find
    any font", exit -6). Point FONTCONFIG_FILE at DejaVu from nixpkgs (nothing is installed globally; NixOS rule)."""
    if os.environ.get("FONTCONFIG_FILE") or shutil.which("fc-list") and subprocess.run(["fc-list"], capture_output=True, text=True).stdout.strip():
        return
    try:
        out = subprocess.run(["nix", "build", "nixpkgs#dejavu_fonts", "--no-link", "--print-out-paths"], capture_output=True, text=True, timeout=600).stdout.split()
        fonts = next(o for o in out if os.path.isdir(os.path.join(o, "share", "fonts")))
    except Exception as e:
        print(f"shot.ensure_fonts: no fonts available ({e}); Chromium may abort")
        return
    d = tempfile.mkdtemp(prefix="dt-fontconfig-")
    conf = os.path.join(d, "fonts.conf")
    with open(conf, "w") as f:
        f.write('<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd"><fontconfig>'
                f'<dir>{fonts}/share/fonts</dir><cachedir>{d}/cache</cachedir></fontconfig>')
    os.environ["FONTCONFIG_FILE"] = conf


class WS:
    """Just enough of RFC 6455 for the DevTools protocol (text frames, client masking)."""

    def __init__(self, url):
        assert url.startswith("ws://")
        hostport, path = url[5:].split("/", 1)
        host, port = hostport.split(":")
        self.sock = socket.create_connection((host, int(port)), timeout=60)
        key = base64.b64encode(os.urandom(16)).decode()
        self.sock.sendall(
            (f"GET /{path} HTTP/1.1\r\nHost: {hostport}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
             f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n").encode())
        head = b""
        while b"\r\n\r\n" not in head:
            head += self.sock.recv(1)
        if b" 101 " not in head.split(b"\r\n", 1)[0]:
            raise RuntimeError(head.decode(errors="replace"))
        self.next_id = 0

    def _recv_exact(self, n):
        buf = b""
        while len(buf) < n:
            chunk = self.sock.recv(n - len(buf))
            if not chunk:
                raise EOFError("DevTools socket closed")
            buf += chunk
        return buf

    def send(self, obj):
        data = json.dumps(obj).encode()
        mask = os.urandom(4)
        n = len(data)
        head = bytes([0x81])
        if n < 126:
            head += bytes([0x80 | n])
        elif n < 65536:
            head += bytes([0x80 | 126]) + struct.pack(">H", n)
        else:
            head += bytes([0x80 | 127]) + struct.pack(">Q", n)
        self.sock.sendall(head + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(data)))

    def recv(self):
        parts = b""
        while True:
            b0, b1 = self._recv_exact(2)
            n = b1 & 0x7F
            if n == 126:
                n = struct.unpack(">H", self._recv_exact(2))[0]
            elif n == 127:
                n = struct.unpack(">Q", self._recv_exact(8))[0]
            payload = self._recv_exact(n)
            parts += payload
            if b0 & 0x80:
                return json.loads(parts)

    def call(self, method, **params):
        self.next_id += 1
        mid = self.next_id
        self.send({"id": mid, "method": method, "params": params})
        while True:
            msg = self.recv()
            if msg.get("id") == mid:
                if "error" in msg:
                    raise RuntimeError(f"{method}: {msg['error']}")
                return msg.get("result", {})


def free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("url")
    ap.add_argument("out")
    ap.add_argument("--wait", type=float, default=8.0, help="seconds after loading, before the first --eval")
    ap.add_argument("--size", default="1600x900")
    ap.add_argument("--eval", action="append", default=[], help="JS to run, then wait --after seconds")
    ap.add_argument("--after", type=float, default=3.0)
    a = ap.parse_args()
    w, h = (int(v) for v in a.size.split("x"))
    chrome = shutil.which("chromium") or shutil.which("chromium-browser") or shutil.which("google-chrome")
    if not chrome:
        sys.exit("shot.py: no chromium on PATH (nix shell nixpkgs#chromium)")
    port = free_port()
    profile = tempfile.mkdtemp(prefix="dt-shot-")
    proc = subprocess.Popen(
        [chrome, "--headless=new", "--disable-gpu", "--no-sandbox", "--hide-scrollbars",
         f"--remote-debugging-port={port}", f"--user-data-dir={profile}", f"--window-size={w},{h}",
         "about:blank"],
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
            sys.exit("shot.py: Chromium's DevTools endpoint did not come up")
        ws = WS(page["webSocketDebuggerUrl"])
        ws.call("Emulation.setDeviceMetricsOverride", width=w, height=h, deviceScaleFactor=1, mobile=False)
        ws.call("Page.enable")
        ws.call("Page.navigate", url=a.url)
        time.sleep(a.wait)
        for js in a.eval:
            r = ws.call("Runtime.evaluate", expression=js, awaitPromise=True, returnByValue=True)
            if r.get("exceptionDetails"):
                print(f"shot.py: {js!r} threw {r['exceptionDetails'].get('text')}", file=sys.stderr)
            time.sleep(a.after)
        shot = ws.call("Page.captureScreenshot", format="png")
        with open(a.out, "wb") as f:
            f.write(base64.b64decode(shot["data"]))
        print(f"{a.out}: {os.path.getsize(a.out)} bytes")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        shutil.rmtree(profile, ignore_errors=True)


if __name__ == "__main__":
    main()
