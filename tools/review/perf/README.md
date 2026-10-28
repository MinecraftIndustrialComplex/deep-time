# perf readers for the bench gate's profiles

`perf script -F tid,time,ip,sym` output, gzipped, from a frame-pointer build profiled with
`sudo -n perf record -F 199 -g -p <pid>` on the test VM (docs/design/p1-17-benchmark-gate.md,
"The gate on master b71c1e51, and where the cores sit idle"). Standard library only; run them on
the VM with `python3 -I`.

- `conc4.py <script.gz>`: the run's idle cores by stage and by deeptime frame. Each sample's
  concurrency is the sample density within ±10 ms, and rayon's idle frames count as idle only
  when they come before any deeptime frame.
- `under3.py <script.gz> <key>...`: for each frame key, the CPU seconds under it and the innermost
  deeptime frames below it.
- `callers2.py <script.gz> <key>`: the nearest deeptime callers of a frame, by CPU seconds.
