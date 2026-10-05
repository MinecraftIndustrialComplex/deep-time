#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-24: one player-side planet generation (simulation + bake) on an approximate player PC, on the
# test VM. The PC is a cgroup scope: AllowedCPUs (whole cores, both SMT threads of each, as a
# desktop's cores are) and MemoryMax (what a 16 GB PC has left beside the OS and the Minecraft
# client), and the Rayon thread count the game itself uses on a client (processors - 1,
# MachineInfo.clientThreads), with the client's default native cap of 4 GiB.
#
#   tools/review/p1-24-profile.sh <profile> <preset> <start> <length_myr> <out dir> [cpu list] [-- extra deeptime run args]
#
#   profile   mid6 = 6 cores / 12 threads, 6 GiB for the simulation (a mid-range gaming PC)
#             low4 = 4 cores / 8 threads, 6 GiB (an older or laptop PC)
#             bg2  = 2 cores / 4 threads, 2 Rayon threads at nice 19, 6 GiB (a background run
#                    beside the game on a mid-range PC: option c of docs/design/faster-create-world.md)
#   cpu list  the VM's CPUs to pin to (default: the first 12 or 8); siblings are n, n+1 on the VM
#
# Writes <out>/run.log (the run's own output, each line stamped with the time it was printed, and
# /usr/bin/time -v), <out>/mem.log (the run's resident memory every 2 s), <out>/load.log (the VM's
# load every 10 s: the box is shared, so wall time is only as good as the load beside it) and
# <out>/result.json (wall, CPU, peak RSS, per-stage seconds, each phase's wall time and peak RSS,
# the bake, the folder's size). P124_MEM=12G overrides the profile's memory (to measure a preset
# that does not fit), P124_BIN another `deeptime` binary, P124_ENV environment words for the run
# (`MALLOC_ARENA_MAX=2`, `LD_PRELOAD=...`: sudo drops the caller's environment); the run traces its
# memory per phase into <out>/memtrace.jsonl (DEEPTIME_MEM_TRACE). The world data is
# deleted afterwards unless P124_KEEP=1. Runs through the admission tool as
# sim:<preset>:<threads>t when it is not already inside an admitted job.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
[ $# -ge 5 ] || { sed -n 4,20p "$0"; exit 2; }
profile="$1"; preset="$2"; start="$3"; length="$4"; out="$5"; shift 5
cpus=""
if [ $# -gt 0 ] && [ "$1" != "--" ]; then cpus="$1"; shift; fi
[ "${1:-}" = "--" ] && shift
case "$profile" in
    mid6) ncpu=12; mem=${P124_MEM:-6G} ;;
    low4) ncpu=8; mem=${P124_MEM:-6G} ;;
    bg2) ncpu=4; mem=${P124_MEM:-6G} ;;
    *) echo "unknown profile $profile" >&2; exit 2 ;;
esac
threads=$((ncpu - 1)); nice=0
[ "$profile" = bg2 ] && { threads=2; nice=19; }
[ -n "$cpus" ] || cpus="0-$((ncpu - 1))"
bin="${P124_BIN:-$REPO/target/release/deeptime}"
[ -x "$bin" ] || { echo "build first: cargo build --release -p deeptime-cli" >&2; exit 2; }

if [ -z "${DEEPTIME_ADMIT_JOB:-}" ] && command -v deeptime-admit >/dev/null; then
    exec deeptime-admit run --kind "sim:${preset}:${threads}t" --label "p1-24-$profile-$preset-$start-$length" -- "$0" "$profile" "$preset" "$start" "$length" "$out" "$cpus" -- "$@"
fi

rm -rf "$out"; mkdir -p "$out"
( while true; do echo "$(date +%s) $(cut -d' ' -f1-3 /proc/loadavg)"; sleep 10; done ) > "$out/load.log" &
sampler=$!
# The run's memory every 2 s, from outside the scope: the scope's anonymous memory (memory.stat
# anon: what the run holds resident, without the page cache of the files it writes) in kB.
unit="p124-$$-$RANDOM"
( f="/sys/fs/cgroup/system.slice/$unit.scope/memory.stat"; while true; do
    [ -r "$f" ] && echo "$EPOCHREALTIME $(awk '$1=="anon"{print int($2/1024)}' "$f")"
    sleep 2; done ) > "$out/mem.log" &
memsampler=$!
t0=$(date +%s.%N)
# A system-level scope (sudo, run as this user): AllowedCPUs and MemoryMax are the PC; its
# cpu.pressure says how long the run's threads waited for a CPU (other jobs on the same CPUs), and
# memory.peak is the scope's peak (resident memory plus the page cache of what it wrote).
sudo systemd-run --scope -q --unit="$unit" --uid="$(id -u)" --gid="$(id -g)" -p "AllowedCPUs=$cpus" \
    -p "MemoryMax=$mem" -p MemorySwapMax=0 bash -c '
    cg=/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup); out="$1"; shift
    set -o pipefail
    /usr/bin/time -v "$@" 2>&1 | while IFS= read -r l; do printf "%s %s\\n" "$EPOCHREALTIME" "$l"; done > "$out/run.log"; s=$?
    { echo "cpu $(grep ^some "$cg/cpu.pressure")"; echo "memory_peak $(cat "$cg/memory.peak")";
      echo "usage_usec $(grep ^usage_usec "$cg/cpu.stat")"; } > "$out/cgroup.txt"
    exit $s' p124 "$out" nice -n "$nice" env ${P124_ENV:-} DEEPTIME_MEM_TRACE="$out/memtrace.jsonl" "$bin" run --preset "$preset" --start "$start" --length-myr "$length" \
    --out "$out/world" --threads "$threads" --mem-cap 4294967296 "$@"
status=$?
t1=$(date +%s.%N)
kill "$sampler" "$memsampler" 2>/dev/null
size=$(du -sb "$out/world" 2>/dev/null | cut -f1)
python3 - "$out" "$status" "$t0" "$t1" "${size:-0}" "$profile" "$preset" "$start" "$length" "$cpus" "$threads" "$mem" <<'PY'
import json, re, sys
out, status, t0, t1, size, profile, preset, start, length, cpus, threads, mem = sys.argv[1:]
log = open(f"{out}/run.log", errors="replace").read()
def grab(pat, conv=float):
    m = re.search(pat, log)
    return conv(m.group(1)) if m else None
stages = {}
m = re.search(r"deeptime run: timings (.*)", log)
if m:
    for name, sec, steps in re.findall(r"(\w+)=([\d.]+)s/(\d+)steps", m.group(1)):
        stages[name] = float(sec)
total = grab(r"\) in ([\d.]+)s")
# Phases from the stamped lines: "stage X started" .. "stage X finished" (the coupled stages start
# together and finish as one), and the bake after the last finished stage.
lines = []
for raw in log.splitlines():
    t, _, rest = raw.partition(" ")
    try:
        lines.append((float(t), rest))
    except ValueError:
        pass
rss = []
try:
    for l in open(f"{out}/mem.log"):
        p = l.split()
        if len(p) == 2:
            rss.append((float(p[0]), int(p[1]) / 1048576))
except OSError:
    pass
phases, open_at = [], None
for t, rest in lines:
    m2 = re.search(r"stage (\S+) (started|finished)", rest)
    if not m2:
        continue
    if m2.group(2) == "started" and open_at is None:
        open_at = t
    elif m2.group(2) == "finished" and open_at is not None:
        phases.append((m2.group(1), open_at, t))
        open_at = None
end_t = lines[-1][0] if lines else None
if open_at is not None and end_t:
    phases.append(("unfinished", open_at, end_t))
elif phases and end_t:
    phases.append(("bake+write", phases[-1][2], end_t))
def peak(a, b):
    v = [g for t, g in rss if a - 1 <= t <= b + 1]
    return round(max(v), 2) if v else None
phase_rows = [{"phase": n, "wall_s": round(b - a, 1), "peak_rss_gib": peak(a, b)} for n, a, b in phases]
loads = [float(l.split()[1]) for l in open(f"{out}/load.log") if len(l.split()) > 1]
user = grab(r"User time \(seconds\): ([\d.]+)")
cg = {}
try:
    for line in open(f"{out}/cgroup.txt"):
        if line.startswith("cpu "):
            cg["cpu_wait_s"] = round(int(re.search(r"total=(\d+)", line).group(1)) / 1e6, 1)
        elif line.startswith("memory_peak "):
            cg["memory_peak_gib"] = round(int(line.split()[1]) / 2**30, 2)
except (OSError, AttributeError, ValueError):
    pass
sys_t = grab(r"System time \(seconds\): ([\d.]+)")
r = {
    "profile": profile, "preset": preset, "start": start, "length_myr": float(length),
    "cpus": cpus, "threads": int(threads), "memory_max": mem, "exit": int(status),
    "wall_s": round(float(t1) - float(t0), 1), "deeptime_total_s": total,
    "user_s": user, "sys_s": sys_t,
    "max_rss_gib": round((grab(r"Maximum resident set size \(kbytes\): (\d+)", int) or 0) / 1048576, 2),
    "native_peak_mib": grab(r"peak ([\d.]+) MiB\)"),
    "stages_s": stages,
    "phases": phase_rows,
    "sampled_peak_rss_gib": round(max(g for _, g in rss), 2) if rss else None,
    "bake_s": round(total - sum(stages.values()), 1) if total and stages else None,
    "world_bytes": int(size),
    "cpu_wait_s": cg.get("cpu_wait_s"), "scope_memory_peak_gib": cg.get("memory_peak_gib"),
    "load_mean": round(sum(loads) / len(loads), 1) if loads else None,
    "load_max": max(loads) if loads else None,
}
json.dump(r, open(f"{out}/result.json", "w"), indent=1)
print(json.dumps(r))
PY
[ "${P124_KEEP:-0}" = 1 ] || rm -rf "$out/world"
exit "$status"
