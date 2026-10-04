#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Runs a command on the test VM detached from the ssh session (tools/remote/detach.sh) and waits for it, so a
dropped ssh connection or a laptop network blip loses a wait, not the job (2026-10-04: one drop killed every
suite step that had been queued for half an hour). The VM's admission tool still admits the job (kind K).

    run(name, cmd, flags=(), pulls=(), kind="auto", log=None) -> (output, exit code, seconds)

`cmd` is any shell command line (quotes, pipes and && are fine: it travels base64-encoded through
tools/review/phase1-run.sh). `flags` are mac.sh flags: --pack syncs the MIC pack; --server turns into a
`server:<name>` kind (its own slot: ports, display, private server folder). Standard library only.
"""

import base64
import os
import re
import subprocess
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MAC = os.path.join(REPO, "tools", "remote", "mac.sh")


def mac(argv, timeout=None):
    r = subprocess.run([MAC, *argv], cwd=REPO, capture_output=True, text=True, errors="replace", timeout=timeout)
    return r.returncode, r.stdout + r.stderr


def run(name, cmd, flags=(), pulls=(), kind="auto", log=None):
    t0 = time.time()
    uniq = f"{name}-{os.getpid()}-{int(t0) % 100000}"
    kind = f"server:{name}" if "--server" in flags and kind == "auto" else kind
    mflags = [f for f in flags if f != "--server"]
    b64 = base64.b64encode(cmd.encode()).decode()
    start = ["--kind", "auto", "--", "tools/remote/detach.sh", "start", uniq, "--kind", kind, "tools/review/phase1-run.sh", b64]
    for attempt in range(8):
        # The first start of a run syncs the checkout; a concurrent sync of the same checkout can collide (git index.lock): retry.
        rc, out = mac(mflags + start)
        if "detach.sh: started" in out:
            break
        time.sleep(30)
    else:
        return f"could not start the detached job: {out[-300:]}", 255, time.time() - t0
    final = ""
    fails = 0
    while True:
        rc, out = mac(["--no-sync", "--", "tools/remote/detach.sh", "wait", uniq, "540", "400"])
        if rc == 3:
            fails = 0
            continue
        if rc in (0, 1) and "EXIT=" in out:
            final = out
            break
        # ssh dropped (255) or the VM was down: the job runs on without us; ask again.
        fails += 1
        if fails > 60:
            return f"lost the VM for good while waiting: {out[-300:]}", 255, time.time() - t0
        time.sleep(30)
    m = re.search(r"^EXIT=(\d+)$", final, re.M)
    code = int(m.group(1)) if m else rc
    for p in pulls:
        for _ in range(5):
            prc, pout = mac(["--no-sync", "--pull", p, "--", "true"])
            if prc == 0:
                break
            time.sleep(20)
    body = "\n".join(l for l in final.splitlines() if not l.startswith("EXIT=") and not l.startswith("detach.sh:"))
    if log:
        os.makedirs(os.path.dirname(log), exist_ok=True)
        open(log, "a").write(f"$ {cmd}\n{body}\n[exit {code}, {time.time() - t0:.0f} s]\n\n")
    return body, code, time.time() - t0
