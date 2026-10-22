#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""CPU seconds used so far by each kind of thread of a JVM, read from /proc (no ps, no jstack).

    dh-thread-cpu.py <marker> [filter]

<marker> is a string of the JVM's command line (tools/client/dh-client-check.sh passes the client's
library folder). Prints one line per thread kind, "<name>: <cpu s> in <n> threads", threads grouped
by their name without trailing digits and dashes (Linux cuts a thread name to 15 characters, so
"DH-World Gen-1" stays whole and "Distant Horizons-..." does not). With [filter], only the kinds
whose name contains it (case-insensitive) are printed. Used to compare Distant Horizons' generator
threads' CPU time across a run (a client whose generator used 6 s of CPU in 7 minutes did no work).
"""
import os
import re
import sys


def pids(marker):
    out = []
    for p in os.listdir("/proc"):
        if not p.isdigit():
            continue
        try:
            with open(f"/proc/{p}/cmdline", "rb") as f:
                if marker.encode() in f.read() and int(p) != os.getpid():
                    out.append(int(p))
        except OSError:
            pass
    return out


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    marker = sys.argv[1]
    flt = sys.argv[2].lower() if len(sys.argv) > 2 else ""
    hz = os.sysconf("SC_CLK_TCK")
    kinds = {}
    for pid in pids(marker):
        # Only the JVM itself: its command line starts with a java binary.
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as f:
                if not f.read().split(b"\0")[0].endswith(b"java"):
                    continue
        except OSError:
            continue
        for tid in os.listdir(f"/proc/{pid}/task"):
            try:
                with open(f"/proc/{pid}/task/{tid}/comm") as f:
                    name = f.read().strip()
                with open(f"/proc/{pid}/task/{tid}/stat") as f:
                    stat = f.read()
            except OSError:
                continue
            fields = stat[stat.rindex(")") + 2:].split()
            cpu = (int(fields[11]) + int(fields[12])) / hz   # utime + stime
            kind = re.sub(r"[-_ ]*\d+$", "", name) or name
            n, c = kinds.get(kind, (0, 0.0))
            kinds[kind] = (n + 1, c + cpu)
    for kind, (n, c) in sorted(kinds.items(), key=lambda kv: -kv[1][1]):
        if flt in kind.lower():
            print(f"{kind}: {c:.1f} s in {n} threads")
    return 0


if __name__ == "__main__":
    sys.exit(main())
