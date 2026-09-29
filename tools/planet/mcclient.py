#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""A minimal Minecraft 1.21.1 client for one question: does the server let this client in?

Used by run_planet.py's `refuse_client` scenario (P1-18, plan §7.15): Deep Time registers its
payload channels as required, so a NeoForge client without Deep Time, and a vanilla client, must
be refused during the configuration phase, before any world data flows. This speaks just enough of
the protocol (767, offline mode, compression) to get there and pretends to be one of:

  neoforge  a NeoForge client offering the given channel list: it answers the server's
            `neoforge:register` query (NeoForge's ModdedNetworkQueryPayload) with those channels,
            then the server negotiates (NetworkComponentNegotiator) and either sends
            `neoforge:network` (accepted) or `neoforge:modded_network_setup_failed` with the
            refused channel ids, then a disconnect;
  vanilla   a client that ignores the query and just answers the ping, which NeoForge treats as a
            vanilla/other client (NetworkRegistry.initializeOtherConnection).

Wire formats follow Minecraft 1.21.1 and NeoForge 21.1.249 (ConfigurationProtocols, LoginProtocols,
ModdedNetworkQueryPayload, ModdedNetworkSetupFailedPayload), read from their sources. Standard
library only.
"""

import io
import socket
import struct
import time
import uuid
import zlib

PROTOCOL = 767  # 1.21.1
# ConnectionProtocol ordinals (net.minecraft.network.ConnectionProtocol) and PacketFlow ordinals.
PROTOCOL_ORDINAL = {"HANDSHAKING": 0, "PLAY": 1, "STATUS": 2, "LOGIN": 3, "CONFIGURATION": 4}
FLOW_ORDINAL = {"SERVERBOUND": 0, "CLIENTBOUND": 1}


# ---------------------------------------------------------------- primitives

def varint(n):
    n &= 0xFFFFFFFF
    out = bytearray()
    while True:
        b = n & 0x7F
        n >>= 7
        if n:
            out.append(b | 0x80)
        else:
            out.append(b)
            return bytes(out)


def string(s):
    b = s.encode("utf-8")
    return varint(len(b)) + b


def read_varint(f):
    n, shift = 0, 0
    while True:
        b = f.read(1)
        if not b:
            raise EOFError("stream ended inside a VarInt")
        n |= (b[0] & 0x7F) << shift
        if not b[0] & 0x80:
            break
        shift += 7
        if shift > 35:
            raise ValueError("VarInt too long")
    return n - (1 << 32) if n & (1 << 31) else n


def read_string(f):
    n = read_varint(f)
    return f.read(n).decode("utf-8", "replace")


def nbt_strings(f, tag_type, out):
    """Skip one network-NBT payload of `tag_type`, collecting every string in it."""
    if tag_type == 0:
        return
    if tag_type == 1:
        f.read(1)
    elif tag_type == 2:
        f.read(2)
    elif tag_type in (3, 5):
        f.read(4)
    elif tag_type in (4, 6):
        f.read(8)
    elif tag_type == 7:
        (n,) = struct.unpack(">i", f.read(4))
        f.read(n)
    elif tag_type == 8:
        (n,) = struct.unpack(">H", f.read(2))
        out.append(f.read(n).decode("utf-8", "replace"))
    elif tag_type == 9:
        (inner,) = struct.unpack(">b", f.read(1))
        (n,) = struct.unpack(">i", f.read(4))
        for _ in range(max(n, 0)):
            nbt_strings(f, inner, out)
    elif tag_type == 10:
        while True:
            (t,) = struct.unpack(">b", f.read(1))
            if t == 0:
                return
            (n,) = struct.unpack(">H", f.read(2))
            f.read(n)  # the name
            nbt_strings(f, t, out)
    elif tag_type == 11:
        (n,) = struct.unpack(">i", f.read(4))
        f.read(4 * n)
    elif tag_type == 12:
        (n,) = struct.unpack(">i", f.read(4))
        f.read(8 * n)
    else:
        raise ValueError(f"unknown NBT tag {tag_type}")


def read_component(f):
    """A text component as network NBT (1.20.3+): a type byte and an unnamed payload."""
    (t,) = struct.unpack(">b", f.read(1))
    out = []
    nbt_strings(f, t, out)
    return out


# ---------------------------------------------------------------- the connection

class Conn:
    def __init__(self, host, port, timeout):
        self.sock = socket.create_connection((host, port), timeout=timeout)
        self.sock.settimeout(timeout)
        self.threshold = -1
        self.buf = b""

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass

    def send(self, packet_id, body=b""):
        raw = varint(packet_id) + body
        if self.threshold >= 0:
            if len(raw) >= self.threshold:
                comp = zlib.compress(raw)
                raw = varint(len(raw)) + comp
            else:
                raw = varint(0) + raw
        self.sock.sendall(varint(len(raw)) + raw)

    def _read_exact(self, n):
        while len(self.buf) < n:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise EOFError("the server closed the connection")
            self.buf += chunk
        out, self.buf = self.buf[:n], self.buf[n:]
        return out

    def recv(self):
        """One packet: (id, body stream)."""
        # The frame length, a VarInt read byte by byte.
        n, shift = 0, 0
        while True:
            b = self._read_exact(1)[0]
            n |= (b & 0x7F) << shift
            if not b & 0x80:
                break
            shift += 7
        data = self._read_exact(n)
        f = io.BytesIO(data)
        if self.threshold >= 0:
            size = read_varint(f)
            if size:
                f = io.BytesIO(zlib.decompress(f.read()))
        return read_varint(f), f


def query_payload(channels):
    """NeoForge's ModdedNetworkQueryPayload body: {protocol ordinal: {component}}."""
    by_protocol = {}
    for c in channels:
        by_protocol.setdefault(c["protocol"], []).append(c)
    out = varint(len(by_protocol))
    for protocol, comps in by_protocol.items():
        out += varint(PROTOCOL_ORDINAL[protocol]) + varint(len(comps))
        for c in comps:
            out += string(c["id"]) + string(c["version"])
            if c.get("flow"):
                out += b"\x01" + varint(FLOW_ORDINAL[c["flow"]])
            else:
                out += b"\x00"
            out += b"\x01" if c.get("optional") else b"\x00"
    return out


def attempt(host, port, mode, channels=(), name=None, timeout=30.0):
    """Log in as `mode` ('neoforge' with `channels`, or 'vanilla') and report what happened.

    Returns {"outcome": "refused" | "accepted" | "error", "refused_channels": [...],
    "disconnect": [strings], "events": [...]}: `accepted` means the modded negotiation succeeded
    (neoforge:network) or, for a vanilla client, the configuration went on past the ping.
    """
    # Player names are at most 16 characters (StringUtil.isValidPlayerName).
    name = (name or f"dt_{mode[:3]}_{int(time.time() * 1000) % 10000000}")[:16]
    events = []
    result = {"outcome": "error", "refused_channels": [], "disconnect": [], "events": events}
    try:
        conn = Conn(host, port, timeout)
    except OSError as e:
        result["error"] = f"connect: {e}"
        return result
    try:
        conn.send(0x00, varint(PROTOCOL) + string(host) + struct.pack(">H", port) + varint(2))
        conn.send(0x00, string(name) + uuid.uuid4().bytes)
        # Login.
        while True:
            pid, f = conn.recv()
            if pid == 0x03:
                conn.threshold = read_varint(f)
                events.append(f"compression {conn.threshold}")
            elif pid == 0x02:
                events.append("login success")
                conn.send(0x03)  # login acknowledged: now in configuration
                break
            elif pid == 0x00:
                result["outcome"] = "refused"
                result["disconnect"] = [read_string(f)]
                events.append("login disconnect")
                return result
            elif pid == 0x04:
                msg_id = read_varint(f)
                conn.send(0x02, varint(msg_id) + b"\x00")  # custom query: not understood
            elif pid == 0x01:
                result["error"] = "the server asked for encryption (online mode)"
                return result
        # Configuration.
        deadline = time.time() + timeout
        while time.time() < deadline:
            pid, f = conn.recv()
            if pid == 0x01:  # custom payload
                channel = read_string(f)
                events.append(f"payload {channel}")
                if channel == "neoforge:register" and mode == "neoforge":
                    conn.send(0x02, string("neoforge:register") + query_payload(channels))
                elif channel == "neoforge:modded_network_setup_failed":
                    n = read_varint(f)
                    for _ in range(n):
                        cid = read_string(f)
                        result["refused_channels"].append(cid)
                        read_component(f)
                elif channel == "neoforge:network":
                    result["outcome"] = "accepted"
                    return result
            elif pid == 0x02:  # disconnect
                result["outcome"] = "refused"
                result["disconnect"] = read_component(f)
                events.append("disconnect")
                return result
            elif pid == 0x04:  # keep alive
                conn.send(0x04, f.read(8))
            elif pid == 0x05:  # ping
                (ping_id,) = struct.unpack(">i", f.read(4))
                conn.send(0x05, struct.pack(">i", ping_id))
                events.append(f"ping {ping_id}")
            else:
                events.append(f"packet 0x{pid:02x}")
                # A vanilla client that got past the ping is in: the server started configuring it.
                if mode == "vanilla" and pid in (0x0C, 0x0E, 0x07):
                    result["outcome"] = "accepted"
                    return result
        result["error"] = "timeout in configuration"
        return result
    except (EOFError, OSError, ValueError, struct.error, zlib.error) as e:
        result.setdefault("error", f"{type(e).__name__}: {e}")
        if result["refused_channels"] and result["outcome"] == "error":
            result["outcome"] = "refused"
        return result
    finally:
        conn.close()


if __name__ == "__main__":
    import json
    import sys
    host, port, mode = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    chans = json.load(open(sys.argv[4])) if len(sys.argv) > 4 else []
    print(json.dumps(attempt(host, port, mode, chans), indent=1))
