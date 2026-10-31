#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continental column's height from its whole buoyancy, a design aid: law.py [table|ocean|start]

docs/design/continental-buoyancy.md. A column's dry isostatic height over the datum, against the ridge's column
(GDH1's 2,600 m, PLT-352), from the densities of its layers and the thermal contraction of its lithosphere on the
plate model's steady geotherm (ISO-138: T_m 1,350 C at L, q_c 26 mW/m2 decaying over h_r 10 km, q_m = B / L), with a
cratonic keel's depletion offsetting a share of its thermal density. Prints the Earth pairs (cratons and hot
back-arcs at the thicknesses CRUST1.0 measures them), the ocean's age-depth law from the same constants against
GDH1, and the start's normal column. Light: pure Python, run anywhere. The Rust law (plates/buoyancy.rs) must give
the same numbers (its tests carry them)."""
import math
import sys

# Constants (each with its ledger id in the design; the values here are the design's, not tuned).
C = {
    "rho_m0": 3330.0,   # mantle at 0 C, kg/m3
    "alpha": 3.2e-5,    # thermal expansion, 1/K (mantle and crust)
    "t_m": 1350.0,      # ISO-138
    "q_c": 26.0,        # PLT-467
    "b": 3375.0,        # PLT-467, mW/m2 x km
    "h_r": 10.0,        # ISO-138
    "dep_share": 0.40,  # Kaban et al. 2003: composition compensates about 40 % of the roots' thermal density
    "rho_w": 1030.0,    # ISO-044
    "ridge_m": 2600.0,  # PLT-352
    "ridge_crust_km": 7.0,
    "rho_oc0": 2950.0,  # ocean crust at 0 C (placeholder until sourced)
}


def heat(z, l):
    qm = C["b"] / l
    return qm * z + C["q_c"] * C["h_r"] * (1.0 - math.exp(-z / C["h_r"]))


def heat_int(a, b, l):
    """The integral of heat(z) over [a, b], closed form."""
    qm = C["b"] / l
    hr = C["h_r"]
    return qm * (b * b - a * a) / 2.0 + C["q_c"] * hr * ((b - a) + hr * (math.exp(-b / hr) - math.exp(-a / hr)))


def t_int(a, b, l):
    """The integral of T over [a, b] km (C km), the ISO-138 geotherm of a lithosphere l km deep (T_m below l)."""
    if b <= a:
        return 0.0
    top, bot = min(a, l), min(b, l)
    s = 0.0
    if bot > top:
        s += C["t_m"] * heat_int(top, bot, l) / heat(l, l)
    if b > l:
        s += C["t_m"] * (b - max(a, l))
    return s


def rho_a():
    return C["rho_m0"] * (1.0 - C["alpha"] * C["t_m"])


def buoyancy(layers, l_km, keel_km):
    """sum over the column of (rho_a - rho) dz, kg/m3 x km, down to the compensation depth (below l and the crust).
    layers: [(thickness km, rho0 kg/m3)] top first."""
    ra = rho_a()
    z, b = 0.0, 0.0
    for th, r0 in layers:
        # rho(z) = r0 (1 - alpha T(z))
        b += (ra - r0) * th + r0 * C["alpha"] * t_int(z, z + th, l_km)
        z += th
    moho = z
    if l_km > moho:
        # mantle lithosphere: rho_m0 (1 - alpha T) against rho_m0 (1 - alpha T_m): -rho_m0 alpha (T_m - T)
        th = l_km - moho
        deficit = C["rho_m0"] * C["alpha"] * (C["t_m"] * th - t_int(moho, l_km, l_km))
        keel_bot = min(l_km, moho + keel_km)
        keel_def = 0.0
        if keel_bot > moho:
            keel_def = C["rho_m0"] * C["alpha"] * (C["t_m"] * (keel_bot - moho) - t_int(moho, keel_bot, l_km))
        b += -deficit + C["dep_share"] * keel_def
    return b


def free_asthenosphere_m():
    """The dry height of the asthenosphere with nothing on it, m over the datum, from the ridge: GDH1's crest under
    water, dried (settle.rs's water load inverted), less its crust's buoyancy (the crust at T_m)."""
    ra = rho_a()
    dry_ridge = -C["ridge_m"] * (C["rho_m0"] - C["rho_w"]) / C["rho_m0"]  # settle.rs uses rho_m 3,300
    b_ridge = C["ridge_crust_km"] * (ra - C["rho_oc0"] * (1.0 - C["alpha"] * C["t_m"]))
    return dry_ridge - b_ridge / ra * 1000.0


def height_m(layers, l_km, keel_km):
    return free_asthenosphere_m() + buoyancy(layers, l_km, keel_km) / rho_a() * 1000.0


def scaled(layers, t_km):
    tot = sum(t for t, _ in layers)
    return [(t * t_km / tot, r) for t, r in layers]


EARTH_CRUST = [(1.0 / 3, 2720.0), (1.0 / 3, 2800.0), (1.0 / 3, 2920.0)]  # CRUST1.0's cratonic layers (in situ), placeholder


def table():
    print(f"free asthenosphere {free_asthenosphere_m():.0f} m over the datum; rho_a {rho_a():.0f}")
    print(f"{'T km':>6} {'craton L225 K200':>18} {'craton L225 K0':>16} {'back-arc L60':>14} {'L125':>8}")
    for t in range(25, 56, 5):
        col = scaled(EARTH_CRUST, t)
        lb = max(60.0, 1.25 * t)  # PLT-820's floor on the heat-ceiling branch
        print(f"{t:>6} {height_m(col, 225, 200):>18.0f} {height_m(col, 225, 0):>16.0f} {height_m(col, lb, 0):>14.0f} "
              f"{height_m(col, 125, 0):>8.0f}")


def ocean():
    """Half-space subsidence from the same constants against GDH1's 365 sqrt(t) (PLT-352)."""
    kappa = 125.0 ** 2 / (math.pi ** 2 * 62.8)  # km2/Myr, PLT-465/466's
    coef = 2.0 * C["rho_m0"] * C["alpha"] * C["t_m"] / (C["rho_m0"] - C["rho_w"]) * math.sqrt(kappa * 1e6 / math.pi)
    print(f"half-space subsidence {coef:.0f} sqrt(t) m against GDH1's 365 (kappa {kappa:.1f} km2/Myr)")


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else "table"
    {"table": table, "ocean": ocean}[mode]()
