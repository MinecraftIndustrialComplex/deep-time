#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Oracle: Deep Time's stream-power kernels against fastscapelib on the same raster.

Plan §8.2 "Oracle" and §9.2. fastscapelib (https://github.com/fastscape-lem/fastscapelib) is
GPL-3.0. It is used ONLY as an unshipped test oracle: this script pip-installs it into a
throwaway virtualenv and drives it from the outside. It is never vendored, never imported by
shipped code, and nothing of it was ported; the comparison uses its documented Python API only.

What runs
---------
1. Deep Time: ``deeptime lem`` (crates/deeptime-core/src/surface/oracle.rs): a W×H raster of dx
   metres, periodic in x, rows 0 and H−1 held at 0 m (base level), uniform uplift U on every other
   cell before each step (h* = h + U·dt), priority-flood+ε, D8 steepest descent, drainage area =
   upstream cells × dx², then the implicit Braun–Willett stream power (G = 0, no critical slope).
   It writes initial.f32, final.f32, area.f32 (f32 LE, row-major, row = y) and meta.json.
2. fastscapelib 0.3.0 from the same initial.f32 with the parameters read back from meta.json:
   RasterGrid([H, W], [dx, dx], [LOOPED, LOOPED, FIXED_VALUE, FIXED_VALUE]) — the list order is
   left, right, top, bottom (RasterBoundaryStatus docstring), so the x borders loop and rows 0 and
   H−1 are the fixed-value base level; the raster is 8-connected (queen, D8). FlowGraph with
   [PFloodSinkResolver(), SingleFlowRouter()] (or --resolver mst: [SingleFlowRouter(),
   MSTSinkResolver()]); SPLEroder(k_coef=K, area_exp=m, slope_exp=n); per step: uplifted = h + U·dt
   (U = 0 on the fixed rows), update_routes(uplifted), area = accumulate(1.0) (accumulate integrates
   a per-area source over nodes_areas(), so this is m²), h = uplifted − erode(uplifted, area, dt).
3. Comparison on the two final surfaces (interior rows 1..H−2):
   - hypsometry_ks: two-sample Kolmogorov–Smirnov statistic of the elevations.
   - chi_slope_ratio: median channel steepness k_s = S·A^θ (θ = m/n; S to the D8 steepest-descent
     neighbour on each final surface, A each model's own last drainage area; cells with A ≥
     --channel-cells cells), Deep Time / fastscape. At steady state with n = 1 the implicit scheme
     gives k_s = U/K exactly, so both should also match that.
   - extras: mean/max elevation, KS of log10 drainage area, the z–χ regression slope ratio, the
     share of cells with the same D8 receiver, the mean |Δz|, runtimes.
   Exit 0 when hypsometry_ks, chi_slope_ratio and area_log10_ks are within the calibrated
   tolerances (constants below), 1 if not, 2 on a setup error.

How it is run (P1-04, 2026-09-28)
---------------------------------
On the Mac (it has PyPI access; fastscapelib 0.3.0 has macOS arm64 wheels for CPython 3.10–3.13,
and the dev shell's python3 is 3.14, hence ``nix shell nixpkgs#python312``)::

    tools/remote/mac.sh --pull review/out/validation/fastscape -- \\
      'cargo build --release -p deeptime-cli && \\
       nix shell nixpkgs#python312 -c python3 tools/validation/fastscape_compare.py --grid 256'

The script creates (or reuses) a venv at review/out/validation/venv-fastscapelib-0.3.0-py3.12
(gitignored, and excluded from mac.sh's upload rsync so its --delete keeps it), pip-installs
fastscapelib==0.3.0 and numpy there and re-executes itself with the venv's python. Outputs go to
--work (default review/out/validation/fastscape: deeptime/ holds the CLI output, fastscape/ the
oracle's final.f32 and area.f32, summary.json both). On NixOS the manylinux wheels would need
libstdc++ on LD_LIBRARY_PATH (nix shell nixpkgs#stdenv.cc.cc.lib; not tried); --lem-dir reuses a
CLI output pulled from the Mac instead of running the binary, for that route. First run (seed 1):
``hypsometry_ks=0.0209 chi_slope_ratio=1.0000 chi_fit_ratio=0.9997 area_log10_ks=0.0038``;
Deep Time 2.5 s, fastscapelib 3.6 s for 500 steps of 256².

Where the two models differ (measured 2026-09-28)
-------------------------------------------------
- One step from the same mature, lake-free surface (Deep Time's step 49 or 499, restarted in
  fastscapelib from its f32 copy): identical drainage area on 99.98–100 % of cells, mean |Δz|
  ≈ 2 mm, 0.01–0.16 % of cells off by more than 1 cm (D8 near-ties flipped by the f32 rounding).
  The kernels agree.
- The first steps on the 0–1 m noise differ: ~38 % of the cells are in pits. Deep Time fills
  lakes with the sediment from upstream even at G = 0 (spl.rs flux pass: deposition up to the lake
  level) and treats a cell as a lake only when filled > 1 cm above it (LAKE_TOL_M; shallower
  fills still incise); fastscapelib's SPLEroder returns erosion ≥ 0 only (no deposition) and does
  not erode inside the depressions its resolver routes across. The ε-filled flats also break ties
  differently (drainage area equal on 95 % of cells after step 1, 90 % after step 5). The networks
  then diverge (a third of the cells keep the same receiver at 50 Myr), while hypsometry, k_s and
  the area distribution agree — hence statistics, not cell-by-cell differences, are compared.
- fastscapelib limits erosion that would reverse a slope with a neighbour (n_corr, 0–4 cells per
  run here); Deep Time has no such limiter.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FASTSCAPELIB = "fastscapelib==0.3.0"
VENV_MARK = "DEEPTIME_FASTSCAPE_VENV"

# Tolerances, calibrated 2026-09-28 on the Mac (defaults: 256², dx 4800 m, K 2e-6, m 0.5, n 1,
# U 1 mm/yr, dt 1e5 yr, 500 steps = 50 Myr, pflood resolver):
#   seed:            1       2       3       4       5
#   hypsometry_ks    0.0209  0.0127  0.0100  0.0169  0.0395
#   chi_slope_ratio  1.0000  1.0000  1.0000  1.0000  1.0000   (both medians 500.00 = U/K)
#   area_log10_ks    0.0038  0.0022  0.0024  0.0055  0.0029
# Noise floor of the hypsometry (same model, another realisation of the network): fastscapelib
# pflood vs mst, seed 1: 0.036; fastscapelib seed 1 vs 2 / 3: 0.006 / 0.038; Deep Time seed 1 vs
# 2 / 3: 0.012 / 0.012. The two models' networks differ cell by cell (about a third of the cells
# keep the same D8 receiver: the first steps on the 0–1 m noise, with ~38 % of cells in pits, treat
# lakes differently — see below), so hypsometry_ks is bounded by that floor, not by 0: 0.06 is
# 1.5× the worst seed. k_s is exactly U/K at steady state in both implicit schemes (p10–p90 within
# ±0.6 %), so ±2 % catches a wrong area, K or exponent. The drainage-area distribution is set by
# the network statistics only: ≤ 0.02 (4× the worst seed).
HYPSOMETRY_KS_MAX = 0.06
CHI_SLOPE_RATIO_RANGE = (0.98, 1.02)
AREA_LOG10_KS_MAX = 0.02


def default_venv() -> Path:
    v = f"{sys.version_info.major}.{sys.version_info.minor}"
    return REPO / "review" / "out" / "validation" / f"venv-fastscapelib-0.3.0-py{v}"


def venv_ok(py: Path) -> bool:
    if not py.exists():
        return False
    probe = "import fastscapelib, numpy; assert fastscapelib.__version__ == '0.3.0'"
    return subprocess.run([str(py), "-c", probe], capture_output=True).returncode == 0


def enter_venv(venv: Path) -> None:
    """Re-executes this script inside a venv holding fastscapelib (creating it if needed)."""
    try:
        import fastscapelib  # noqa: F401
        import numpy  # noqa: F401

        return
    except ImportError:
        pass
    if os.environ.get(VENV_MARK):
        sys.exit(f"fastscape_compare: fastscapelib is not importable inside {sys.executable}")
    if not (3, 10) <= sys.version_info[:2] <= (3, 13):
        print(
            f"fastscape_compare: warning: Python {sys.version.split()[0]}; fastscapelib 0.3.0 has "
            "wheels for 3.10–3.13 only (use nix shell nixpkgs#python312)",
            file=sys.stderr,
        )
    py = venv / "bin" / "python"
    if not venv_ok(py):
        import shutil

        shutil.rmtree(venv, ignore_errors=True)
        venv.parent.mkdir(parents=True, exist_ok=True)
        print(f"fastscape_compare: creating {venv}", file=sys.stderr)
        subprocess.check_call([sys.executable, "-m", "venv", str(venv)])
        subprocess.check_call(
            [str(py), "-m", "pip", "install", "-q", "--disable-pip-version-check", FASTSCAPELIB, "numpy"]
        )
    env = dict(os.environ, **{VENV_MARK: "1"})
    os.execve(str(py), [str(py), str(Path(__file__).resolve()), *sys.argv[1:]], env)


# ---------------------------------------------------------------------------------------------
# Runs


def run_deeptime(a, out: Path) -> float:
    exe = Path(a.deeptime)
    if not exe.is_absolute():
        exe = REPO / exe
    if not exe.exists():
        sys.exit(f"fastscape_compare: {exe} not found (cargo build --release -p deeptime-cli)")
    cmd = [
        str(exe), "lem", "--grid", str(a.grid), "--height", str(a.height or a.grid),
        "--dx", str(a.dx), "--steps", str(a.steps), "--dt-yr", str(a.dt_yr), "--k", str(a.k),
        "--m", str(a.m), "--n", str(a.n), "--g", "0", "--uplift-mm-yr", str(a.uplift_mm_yr),
        "--kd", "0", "--seed", str(a.seed), "--out", str(out),
    ]
    started = time.perf_counter()
    subprocess.check_call(cmd)
    return time.perf_counter() - started


def load(path: Path, h: int, w: int):
    import numpy as np

    return np.fromfile(path, dtype="<f4").astype(np.float64).reshape(h, w)


def run_fastscape(meta: dict, initial, resolver: str):
    import fastscapelib as fs
    import numpy as np

    h_, w = meta["h"], meta["w"]
    dx, dt, u = meta["dx"], meta["dt_yr"], meta["uplift_m_yr"]
    fixed, looped = fs.NodeStatus.FIXED_VALUE, fs.NodeStatus.LOOPED
    # [left, right, top, bottom]: x loops, rows 0 and h−1 are the fixed-value base level.
    grid = fs.RasterGrid([h_, w], [dx, dx], [looped, looped, fixed, fixed])
    status = np.asarray(grid.nodes_status())
    assert (status[0] == 1).all() and (status[-1] == 1).all() and (status[1:-1, 0] == 3).all()
    if resolver == "pflood":
        ops = [fs.PFloodSinkResolver(), fs.SingleFlowRouter()]
    else:
        ops = [fs.SingleFlowRouter(), fs.MSTSinkResolver()]
    graph = fs.FlowGraph(grid, ops)
    spl = fs.SPLEroder(graph, k_coef=meta["k"], area_exp=meta["m"], slope_exp=meta["n"], tolerance=1e-5)
    uplift = np.full((h_, w), u)
    uplift[0, :] = 0.0
    uplift[-1, :] = 0.0
    z = initial.copy()
    area = None
    corrected = 0
    started = time.perf_counter()
    for _ in range(meta["steps"]):
        uplifted = z + dt * uplift
        graph.update_routes(uplifted)
        area = graph.accumulate(1.0)
        erosion = spl.erode(uplifted, area, dt)
        nc = spl.n_corr
        corrected += nc() if callable(nc) else nc
        z = uplifted - erosion
    return z, np.asarray(area), time.perf_counter() - started, corrected


# ---------------------------------------------------------------------------------------------
# Statistics (numpy only)

# D8 offsets (di, dj, diagonal) in Deep Time's order (grid.rs): E, NE, N, NW, W, SW, S, SE.
OFFSETS = [(1, 0, 0), (1, -1, 1), (0, -1, 0), (-1, -1, 1), (-1, 0, 0), (-1, 1, 1), (0, 1, 0), (1, 1, 1)]


def ks_2samp(a, b) -> float:
    import numpy as np

    a, b = np.sort(np.ravel(a)), np.sort(np.ravel(b))
    v = np.concatenate([a, b])
    fa = np.searchsorted(a, v, side="right") / a.size
    fb = np.searchsorted(b, v, side="right") / b.size
    return float(np.max(np.abs(fa - fb)))


def d8(z, dx):
    """Steepest-descent receiver (flat index), slope and length of every cell; x wraps, rows stop;
    the base rows and cells with no lower neighbour are their own receivers (slope 0)."""
    import numpy as np

    h, w = z.shape
    jj, ii = np.meshgrid(np.arange(h), np.arange(w), indexing="ij")
    rcv = (jj * w + ii).copy()
    slope = np.zeros_like(z)
    length = np.zeros_like(z)
    for di, dj, diag in OFFSETS:
        nj, ni = jj + dj, (ii + di) % w
        ok = (nj >= 0) & (nj < h)
        njc = np.clip(nj, 0, h - 1)
        ln = dx * (np.sqrt(2.0) if diag else 1.0)
        s = np.where(ok, (z - z[njc, ni]) / ln, -np.inf)
        better = s > slope
        slope = np.where(better, s, slope)
        rcv = np.where(better, njc * w + ni, rcv)
        length = np.where(better, ln, length)
    for row in (0, h - 1):
        rcv[row] = jj[row] * w + ii[row]
        slope[row] = 0.0
        length[row] = 0.0
    return rcv, slope, length


def chi(z, area, rcv, length, theta):
    """χ = ∫ (A0/A)^θ dx along the receivers from the base level, A0 = 1 m² (so z = k_s·χ)."""
    import numpy as np

    flat_z, flat_r, flat_l = z.ravel(), rcv.ravel(), length.ravel()
    inv = np.power(np.maximum(area.ravel(), 1.0), -theta)
    out = np.zeros(flat_z.size)
    for c in np.argsort(flat_z, kind="stable"):  # receivers are strictly lower: done first
        r = flat_r[c]
        if r != c:
            out[c] = out[r] + flat_l[c] * inv[c]
    return out.reshape(z.shape)


def stats(z, area, dx, theta, min_area):
    import numpy as np

    rcv, slope, length = d8(z, dx)
    core = np.zeros(z.shape, bool)
    core[1:-1] = True
    chan = core & (area >= min_area) & (slope > 0)
    ks = slope[chan] * np.power(area[chan], theta)
    x = chi(z, area, rcv, length, theta)
    fit = float(np.sum(z[chan] * x[chan]) / np.sum(x[chan] ** 2))
    return {
        "rcv": rcv, "core": core, "chan": chan,
        "ks_median": float(np.median(ks)), "ks_p10": float(np.percentile(ks, 10)),
        "ks_p90": float(np.percentile(ks, 90)), "chi_fit": fit, "channel_cells": int(chan.sum()),
        "mean": float(z[core].mean()), "max": float(z.max()),
    }


# ---------------------------------------------------------------------------------------------


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--grid", type=int, default=256, help="cells per side (W; H too unless --height)")
    p.add_argument("--height", type=int, default=None, help="rows H (default --grid)")
    p.add_argument("--dx", type=float, default=4800.0, help="cell size, m")
    p.add_argument("--steps", type=int, default=500, help="time steps")
    p.add_argument("--dt-yr", type=float, default=1.0e5, help="time step, years")
    p.add_argument("--k", type=float, default=2.0e-6, help="stream-power K, m^(1-2m)/yr")
    p.add_argument("--m", type=float, default=0.5, help="area exponent")
    p.add_argument("--n", type=float, default=1.0, help="slope exponent")
    p.add_argument("--uplift-mm-yr", type=float, default=1.0, help="uniform uplift, mm/yr")
    p.add_argument("--seed", type=int, default=1, help="seed of the initial noise")
    p.add_argument("--channel-cells", type=float, default=10.0,
                   help="drainage area (in cells) above which a cell counts as a channel")
    p.add_argument("--resolver", choices=["pflood", "mst"], default="pflood",
                   help="fastscapelib sink resolver (pflood matches Deep Time's priority flood)")
    p.add_argument("--deeptime", default="target/release/deeptime", help="the deeptime binary")
    p.add_argument("--work", default="review/out/validation/fastscape", help="output folder")
    p.add_argument("--lem-dir", default=None,
                   help="reuse an existing `deeptime lem` output instead of running the binary")
    p.add_argument("--venv", default=os.environ.get("DEEPTIME_VALIDATION_VENV"),
                   help="venv for fastscapelib (default review/out/validation/venv-…)")
    a = p.parse_args()

    enter_venv(Path(a.venv) if a.venv else default_venv())
    import numpy as np

    work = Path(a.work)
    if not work.is_absolute():
        work = REPO / work
    if a.lem_dir:
        lem_dir, dt_seconds = Path(a.lem_dir), None
    else:
        lem_dir = work / "deeptime"
        dt_seconds = run_deeptime(a, lem_dir)
    meta = json.loads((lem_dir / "meta.json").read_text())
    h, w, dx = meta["h"], meta["w"], meta["dx"]
    if meta.get("g", 0) != 0 or meta.get("kd_m2_yr", 0) != 0:
        print("fastscape_compare: the oracle runs G = 0 and no diffusion", file=sys.stderr)
        return 2
    initial = load(lem_dir / "initial.f32", h, w)
    z_dt = load(lem_dir / "final.f32", h, w)
    a_dt = load(lem_dir / "area.f32", h, w)

    z_fs, a_fs, fs_seconds, n_corr = run_fastscape(meta, initial, a.resolver)
    out = work / "fastscape"
    out.mkdir(parents=True, exist_ok=True)
    z_fs.astype("<f4").tofile(out / "final.f32")
    a_fs.astype("<f4").tofile(out / "area.f32")

    theta = meta["m"] / meta["n"]
    min_area = a.channel_cells * dx * dx
    s_dt = stats(z_dt, a_dt, dx, theta, min_area)
    s_fs = stats(z_fs, a_fs, dx, theta, min_area)
    core = s_dt["core"]
    hyps_ks = ks_2samp(z_dt[core], z_fs[core])
    chi_ratio = s_dt["ks_median"] / s_fs["ks_median"]
    area_ks = ks_2samp(np.log10(a_dt[core]), np.log10(a_fs[core]))
    ks_theory = (meta["uplift_m_yr"] / meta["k"]) ** (1.0 / meta["n"])
    same_rcv = float(np.mean(s_dt["rcv"][core] == s_fs["rcv"][core]))
    mad = float(np.mean(np.abs(z_dt[core] - z_fs[core])))

    summary = {
        "hypsometry_ks": hyps_ks, "chi_slope_ratio": chi_ratio,
        "chi_fit_ratio": s_dt["chi_fit"] / s_fs["chi_fit"], "area_log10_ks": area_ks,
        "ks_theory": ks_theory,
        "deeptime": {k: v for k, v in s_dt.items() if k not in ("rcv", "core", "chan")},
        "fastscape": {k: v for k, v in s_fs.items() if k not in ("rcv", "core", "chan")},
        "same_receiver_share": same_rcv, "mean_abs_dz_m": mad, "fastscape_n_corr": n_corr,
        "deeptime_kernel_s": meta.get("seconds"), "deeptime_cli_s": dt_seconds,
        "fastscape_s": fs_seconds, "resolver": a.resolver, "meta": meta,
    }
    (work / "summary.json").write_text(json.dumps(summary, indent=2))

    print(
        f"grid {w}x{h} dx={dx:g} steps={meta['steps']} dt_yr={meta['dt_yr']:g} K={meta['k']:g} "
        f"m={meta['m']:g} n={meta['n']:g} U={meta['uplift_m_yr'] * 1e3:g} mm/yr seed={meta['seed']} "
        f"resolver={a.resolver}"
    )
    for name, s in (("deeptime ", s_dt), ("fastscape", s_fs)):
        print(
            f"  {name}: mean {s['mean']:8.1f} m  max {s['max']:8.1f} m  k_s median {s['ks_median']:7.2f} "
            f"(p10 {s['ks_p10']:.2f}, p90 {s['ks_p90']:.2f})  z-chi slope {s['chi_fit']:7.2f}  "
            f"channel cells {s['channel_cells']}"
        )
    print(
        f"  k_s theory U/K = {ks_theory:.2f}; same D8 receiver {same_rcv:.3f}; mean |dz| {mad:.2f} m; "
        f"fastscape n_corr {n_corr}"
    )
    kernel = meta.get("seconds")
    print(
        f"  runtime: deeptime kernel {kernel:.2f} s"
        + (f" (CLI {dt_seconds:.2f} s)" if dt_seconds is not None else "")
        + f", fastscapelib {fs_seconds:.2f} s"
    )
    print(
        f"hypsometry_ks={hyps_ks:.4f} chi_slope_ratio={chi_ratio:.4f} "
        f"chi_fit_ratio={summary['chi_fit_ratio']:.4f} area_log10_ks={area_ks:.4f}"
    )

    lo, hi = CHI_SLOPE_RATIO_RANGE
    ok = hyps_ks <= HYPSOMETRY_KS_MAX and lo <= chi_ratio <= hi and area_ks <= AREA_LOG10_KS_MAX
    print(
        f"{'PASS' if ok else 'FAIL'}: hypsometry_ks <= {HYPSOMETRY_KS_MAX}, "
        f"{lo} <= chi_slope_ratio <= {hi}, area_log10_ks <= {AREA_LOG10_KS_MAX}"
    )
    if not ok and meta["steps"] * meta["dt_yr"] < 3.0e7:
        print("  (tolerances are calibrated at steady state: 500 steps of 1e5 yr)", file=sys.stderr)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
