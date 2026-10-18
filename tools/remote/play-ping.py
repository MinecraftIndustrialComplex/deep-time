#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""A Minecraft server-list ping to Ben's play server through the laptop's tunnel (stdlib only).

    nix shell nixpkgs#python3 -c python3 -I tools/remote/play-ping.py [--host 127.0.0.1] [--port 25765]

Prints the version, the players and the motd; exit status 1 when nothing answers. The tunnel is the
user unit deeptime-play-tunnel (tools/remote/tunnels.sh recreates it); it retries every 2 minutes
while the VM is down.
"""
import argparse
import json
import socket
import struct
import sys


def varint(n):
    out = b""
    while True:
        b = n & 0x7F
        n >>= 7
        out += bytes([b | (0x80 if n else 0)])
        if not n:
            return out


def read_varint(s):
    n = shift = 0
    while True:
        b = s.recv(1)
        if not b:
            raise EOFError("connection closed")
        n |= (b[0] & 0x7F) << shift
        if not b[0] & 0x80:
            return n
        shift += 7


def packet(pid, payload=b""):
    body = varint(pid) + payload
    return varint(len(body)) + body


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--port", type=int, default=25765)
    a = ap.parse_args()
    try:
        s = socket.create_connection((a.host, a.port), timeout=15)
        # Handshake (protocol 767 = 1.21.1, next state 1 = status), then the status request.
        s.sendall(packet(0, varint(767) + varint(len(a.host)) + a.host.encode() + struct.pack(">H", a.port) + varint(1)) + packet(0))
        read_varint(s)  # packet length
        read_varint(s)  # packet id
        n = read_varint(s)
        data = b""
        while len(data) < n:
            chunk = s.recv(n - len(data))
            if not chunk:
                raise EOFError("short status")
            data += chunk
        j = json.loads(data)
    except (OSError, EOFError, ValueError) as e:
        print(f"play-ping: no answer from {a.host}:{a.port}: {e}", file=sys.stderr)
        return 1
    desc = j.get("description")
    if isinstance(desc, dict):
        desc = desc.get("text", desc)
    print("version:", j.get("version"))
    print("players:", j.get("players"))
    print("motd:", desc)
    return 0


if __name__ == "__main__":
    sys.exit(main())
