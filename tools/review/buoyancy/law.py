#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continental column's height from its whole buoyancy, a design aid: law.py [pairs|model|ocean|start|all]

docs/design/continental-buoyancy.md. A column's dry isostatic height over the datum is the free asthenosphere's
height plus the integral of (rho_a - rho) / rho_a down the column to the compensation depth (Lachenbruch & Morgan
1990, eq. 1 as Zoback & Mooney 2003 print it). The free asthenosphere's height comes from the ridge: GDH1's 2,600 m of
water over the ocean's 7 km of crust at the mantle's temperature (PLT-352, PLT-104). rho:

- the crust: each pool's density at its depth (Christensen & Mooney 1995, Table 4: laboratory densities at room
  temperature, compression included), expanded by alpha T on the column's geotherm; CRUST1.0's layers (in situ
  already, from Vp) as they are;
- the mantle lithosphere: rho_m0 (1 - alpha T), the keel's mantle lighter by 40 % of its thermal excess over the
  asthenosphere (Kaban et al. 2003: "partly (about 40%) compensated");
- the asthenosphere: rho_m0 (1 - alpha T_m).

alpha, rho_m0 and T_m are Parsons & Sclater's plate as Stein & Stein 1992 print it (3.28e-5 /K, 3,330 kg/m3, 1,350 C;
the model's 125 km plate, PLT-465, and T_m, ISO-138); the geotherm is the plate model's steady one (ISO-138: T_m at
L, q_c 26 mW/m2 decaying over h_r 10 km, q_m = B / L, PLT-467); L never above the lid's floor 1.25 z_Moho (PLT-820).
The Rust law (plates/buoyancy.rs) gives the same numbers (its tests carry them). Pure Python, light."""
import math
import sys

ALPHA = 3.28e-5       # 1/K, PSM (Stein & Stein 1992, p. 124)                          ledger: PLT-830
RHO_M0 = 3330.0       # kg/m3 at 0 C, PSM (Stein & Stein 1992, p. 124)                  ledger: PLT-830
T_M = 1350.0          # C, ISO-138 (PSM's basal temperature too)
KEEL_SHARE = 0.4      # the keel's compensation of its thermal excess (Kaban et al. 2003)  ledger: PLT-831
RHO_W = 1030.0        # ISO-044
RIDGE_M = 2600.0      # PLT-352
Q_C, B, H_R = 26.0, 3375.0, 10.0   # PLT-467, ISO-138
CRUST_BASE_C = 1080.0              # PLT-822 (the lid's floor, PLT-820)

# Pool densities, kg/m3 at 5 km and per km deeper (Christensen & Mooney 1995, Table 4)      ledger: PLT-832
POOLS = {
    "metamorphic": (2746.0, (2772.0 - 2746.0) / 20.0),      # biotite (tonalite) gneiss, 5 to 25 km
    "felsic_plutonic": (2662.0, (2687.0 - 2662.0) / 20.0),  # granite gneiss, 5 to 25 km
    "felsic_volcanic": (2654.0, (2687.0 - 2662.0) / 20.0),  # granite-granodiorite at 5 km, the gneiss's slope
    "mafic_volcanic": (2878.0, (2998.0 - 2977.0) / 20.0),   # basalt at 5 km, the mafic granulite's slope
    "mafic_plutonic": (2966.0, (2998.0 - 2977.0) / 20.0),   # gabbro-norite-troctolite at 5 km, mafic granulite's
}
# Sediment and sedimentary rock: shale's porosity phi0 e^(-c z) with grain 2,720 and sea water in the pores (Angevine
# et al. 1990, Fig. 3.11, after Sclater & Christie 1980)                                 ledger: PLT-833
SED_PHI0, SED_C_PER_KM, SED_GRAIN = 0.5, 0.5, 2720.0
# The eclogite root (PLT-278 to PLT-280): past 1.5 GPa, 0.3 of the mafic pools at 3,450 kg/m3.
ECLOGITE_PA, ECLOGITE_SHARE, RHO_ECLOGITE, RHO_CRUST_MEAN, G = 1.5e9, 0.3, 3450.0, 2830.0, 9.81
ORDER = ["sediment", "sedimentary", "felsic_volcanic", "metamorphic", "felsic_plutonic", "mafic_volcanic",
         "mafic_plutonic"]  # top to bottom: the cover, the felsic crust, the mafic pools at the bottom (as eclogite_m)


def heat(z, l):
    return B / l * z + Q_C * H_R * (1.0 - math.exp(-z / H_R))


def temp(z, l):
    return T_M if z >= l else T_M * heat(z, l) / heat(l, l)


def rho_a():
    return RHO_M0 * (1.0 - ALPHA * T_M)


def integrate(f, a, b, n=400):
    if b <= a:
        return 0.0
    h = (b - a) / n
    return sum(f(a + (i + 0.5) * h) for i in range(n)) * h


def rho_rock(name, z):
    if name in ("sediment", "sedimentary"):
        phi = SED_PHI0 * math.exp(-SED_C_PER_KM * z)
        return (1.0 - phi) * SED_GRAIN + phi * RHO_W
    r5, slope = POOLS[name]
    return r5 + slope * (z - 5.0)


def lid_floor(moho):
    return moho * T_M / CRUST_BASE_C


def mantle(moho, l, keel):
    """The mantle lithosphere's sum of (rho_a - rho) dz, kg/m3 km: its thermal excess, less the keel's share."""
    le = max(l, lid_floor(moho))
    ra = rho_a()
    ex = integrate(lambda z: ra - RHO_M0 * (1.0 - ALPHA * temp(z, le)), moho, le)
    kb = min(le, moho + keel)
    kex = integrate(lambda z: ra - RHO_M0 * (1.0 - ALPHA * temp(z, le)), moho, kb)
    return ex - KEEL_SHARE * kex


def free_asthenosphere_m():
    ra = rho_a()
    b = integrate(lambda z: ra - rho_rock("mafic_volcanic", z) * (1.0 - ALPHA * T_M), 0.0, 2.0)
    b += integrate(lambda z: ra - rho_rock("mafic_plutonic", z) * (1.0 - ALPHA * T_M), 2.0, 7.0)
    return -RIDGE_M * (ra - RHO_W) / ra - b / ra * 1000.0


def height_layers(layers, l, keel):
    """Layers in situ: [(km, kg/m3)] top first (CRUST1.0's)."""
    ra = rho_a()
    b = sum((ra - r) * th for th, r in layers)
    moho = sum(th for th, _ in layers)
    return free_asthenosphere_m() + (b + mantle(moho, l, keel)) / ra * 1000.0


def height_pools(col, l, keel):
    """col: {pool: km}; each pool's density at its depth, expanded on the geotherm; the eclogite root."""
    ra = rho_a()
    moho = sum(col.values())
    le = max(l, lid_floor(moho))
    z, b = 0.0, 0.0
    for name in ORDER:
        th = col.get(name, 0.0)
        if th > 0.0:
            b += integrate(lambda zz: ra - rho_rock(name, zz) * (1.0 - ALPHA * temp(zz, le)), z, z + th)
            z += th
    # The eclogite root: of the mafic rock deeper than the eclogite depth (the column's bottom), the share
    # ECLOGITE_SHARE weighs RHO_ECLOGITE instead of its own density.
    mafic = col.get("mafic_plutonic", 0.0) + col.get("mafic_volcanic", 0.0)
    depth = ECLOGITE_PA / (RHO_CRUST_MEAN * G) / 1000.0
    deep = min(mafic, max(0.0, moho - depth))
    if deep > 0.0:
        b -= ECLOGITE_SHARE * integrate(lambda zz: (RHO_ECLOGITE - rho_rock("mafic_plutonic", zz))
                                        * (1.0 - ALPHA * temp(zz, le)), moho - deep, moho)
    return free_asthenosphere_m() + (b + mantle(moho, l, keel)) / ra * 1000.0


# Rudnick & Gao's three-layer crust as Hacker et al. 2015 print it (Fig. 1: upper 0-12 km, middle 12-23, lower 23-40;
# the lower crust 80 % mafic), as pools, scaled to t km                                   ledger: PLT-834
def earth_col(t):
    f = t / 40.0
    return {"sediment": 0.1 * f, "sedimentary": 2.0 * f, "felsic_plutonic": 9.9 * f, "metamorphic": (11.0 + 3.4) * f,
            "mafic_plutonic": 13.6 * f}


def plt131():
    return {"sediment": 0.1, "sedimentary": 2.0, "metamorphic": 6.0, "felsic_plutonic": 21.0, "felsic_volcanic": 1.5,
            "mafic_plutonic": 4.5}


# CRUST1.0's own crusts (m4cold.py, p50 per region and band: sediment, upper, middle, lower crust, km and kg/m3) and
# their rock p50 heights over the sea: the Earth pairs.
PAIRS = [
    ("East European platform 35-40", [(2.7, 2315), (11.5, 2740), (12.4, 2820), (11.6, 2920)], 140, "craton"),
    ("East European platform 40-45", [(1.5, 2283), (13.3, 2740), (14.5, 2830), (13.1, 2920)], 150, "craton"),
    ("Siberian platform 40-45", [(2.8, 2400), (12.5, 2740), (13.9, 2830), (12.7, 2920)], 360, "craton"),
    ("Canadian shield 35-40", [(11.4, 2760), (13.3, 2830), (12.5, 2920)], 230, "craton"),
    ("Canadian shield 40-45", [(12.8, 2760), (14.9, 2810), (14.1, 2910)], 310, "craton"),
    ("Australia 40-45", [(0.6, 2350), (13.2, 2740), (14.6, 2830), (13.3, 2920)], 340, "craton"),
    ("Brazilian shield 40-45", [(0.7, 2226), (13.1, 2740), (14.1, 2830), (13.1, 2920)], 320, "craton"),
    ("Basin and Range, Cordillera 30-35", [(0.1, 2110), (11.2, 2740), (11.0, 2780), (10.4, 2860)], 1407, "back-arc"),
    ("Basin and Range, Cordillera 35-40", [(14.7, 2740), (13.0, 2780), (8.1, 2950)], 1606, "back-arc"),
    ("Basin and Range, Cordillera 40-45", [(1.5, 2285), (17.7, 2740), (15.6, 2780), (7.0, 2950)], 1834, "back-arc"),
]
STATES = {"craton": (225.0, 200.0), "back-arc": (60.0, 0.0)}  # PLT-462/463; PLT-461


def pairs():
    print(f"rho_a {rho_a():.0f} kg/m3; free asthenosphere {free_asthenosphere_m():.0f} m (H0 "
          f"{-free_asthenosphere_m() / 1000:.2f} km; Zoback & Mooney's 2.78 +- 0.35)")
    print("Earth's pairs on CRUST1.0's own crusts (in situ densities): the law against the rock p50, m")
    for name, lay, obs, st in PAIRS:
        l, k = STATES[st]
        z = height_layers(lay, l, k)
        print(f"  {name:36s} {st:8s} law {z:6.0f}  Earth {obs:5d}  diff {z - obs:+6.0f}")
    eq = [(11.0, 2740), (12.0, 2800), (12.0, 2920)]
    print(f"  equal crust (35 km of 2,740/2,800/2,920): craton {height_layers(eq, 225, 200):.0f}, back-arc "
          f"{height_layers(eq, 60, 0):.0f}, the difference {height_layers(eq, 60, 0) - height_layers(eq, 225, 200):.0f} "
          f"(Hyndman & Currie 2011: about 1,600)")


def model():
    print("The model's pools (Rudnick & Gao's layers), m over the datum: craton (L 225, K 200) / back-arc (L 60)")
    for t in (30, 35, 38.8, 40, 45, 50):
        c = earth_col(t)
        print(f"  {t:5g} km: {height_pools(c, 225, 200):6.0f} / {height_pools(c, 60, 0):6.0f}")
    c = plt131()
    print(f"  PLT-131's column (35.1 km): {height_pools(c, 225, 200):.0f} / {height_pools(c, 60, 0):.0f}")


def ocean():
    kappa = 125.0 ** 2 / (math.pi ** 2 * 62.8)  # km2/Myr, PLT-465
    coef = 2.0 * RHO_M0 * ALPHA * T_M / (RHO_M0 - RHO_W) * math.sqrt(kappa * 1e6 / math.pi)
    print(f"half-space subsidence from the same constants {coef:.0f} sqrt(t) m; GDH1 365 (kappa {kappa:.1f} km2/Myr)")


def start():
    for target in (-130.0, 0.0, 150.0, 525.0, 900.0):
        lo, hi = 15.0, 80.0
        for _ in range(60):
            mid = 0.5 * (lo + hi)
            if height_pools(earth_col(mid), 225, 200) < target:
                lo = mid
            else:
                hi = mid
        print(f"  the start (craton) at {target:5.0f} m dry: {0.5 * (lo + hi):.1f} km")


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else "all"
    for m in (["pairs", "model", "ocean", "start"] if mode == "all" else [mode]):
        {"pairs": pairs, "model": model, "ocean": ocean, "start": start}[m]()
