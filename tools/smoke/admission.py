# SPDX-License-Identifier: LGPL-3.0-or-later
"""Server launchers run themselves through the test VM's admission tool (tools/remote/vm/admit.py,
installed as /usr/local/lib/deeptime/admit.py; Ben, 2026-10-03: load-based admission instead of the
one-server lock).

    import admission
    admission.ensure("smoke:mic")     # first thing in main(): re-executes this script through
                                      # admit.py unless it already is the admitted process

After ensure() the process has its slot in the environment (admission.ports(), server_dir()).
Off the VM (no admit.py installed) ensure() does nothing and the launchers keep their old fixed
ports and folders. Standard library only.
"""

import os
import sys

ADMIT = os.environ.get("DEEPTIME_ADMIT") or "/usr/local/lib/deeptime/admit.py"


def admitted():
    return os.environ.get("DEEPTIME_ADMIT_PID") == str(os.getpid())


def available():
    return os.path.isfile(ADMIT)


def ensure(kind, label=None, status_json=None, argv=None):
    """Re-exec the running script (sys.argv) through `admit.py run --kind <kind> --server`."""
    if admitted() or not available():
        return
    repo = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    cmd = [sys.executable, ADMIT, "run", "--kind", kind, "--server", "--label", label or os.path.basename(repo)]
    if status_json:
        cmd += ["--status-json", status_json]
    argv = argv if argv is not None else [os.path.abspath(sys.argv[0])] + sys.argv[1:]
    sys.stdout.flush()
    sys.stderr.flush()
    os.execv(sys.executable, cmd + ["--", sys.executable] + argv)


def ports(fallback=None):
    """{"game", "rcon", "query", "voice"} from the admission, else fallback."""
    if not os.environ.get("DEEPTIME_PORT_GAME"):
        return fallback
    return {k: int(os.environ[f"DEEPTIME_PORT_{k.upper()}"]) for k in ("game", "rcon", "query", "voice")}


def server_dir():
    """The job's private folder (removed by admit.py when the job ends), or None."""
    return os.environ.get("DEEPTIME_SERVER_DIR") or None
