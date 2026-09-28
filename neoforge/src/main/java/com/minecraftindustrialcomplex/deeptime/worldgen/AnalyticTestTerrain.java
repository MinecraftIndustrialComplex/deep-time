// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

/**
 * A closed-form test terrain, periodic in x with period C (plan §7.1). It lets the generator,
 * Toroidal World's wrap and the GameTests run before any Rust or world data exists (P0-05, P0-06).
 *
 * <p>Longitude θ = 2πx/C and latitude φ = πz/(C/2) (z ∈ [-C/4, C/4) spans pole to pole):
 * <pre>
 *   H = 63 + 60·cos(θ − π)·cos φ          one continent straddling the seam (θ = ±π)
 *        + 25·sin(3θ + 0.5)·cos 2φ       smaller lands and seas
 *        + 450·g(dx, 420)·g(z − z_r, 260) a mountain range at x_r = C/8, z_r = C/32
 * </pre>
 * where g(d, w) = exp(−d²/2w²) and dx is the periodic distance to x_r. The peak stays below
 * 63 + 60 + 25 + 450 = 598, under the top slide at y 624 (cold-read review 2, gap 8), and the
 * curvature stays under 450/260² ≈ 0.0067 block per block², so vanilla's 4 × 8 × 4 density
 * interpolation reproduces H within a block between cell corners.
 */
public final class AnalyticTestTerrain implements TerrainSource {

    public static final String WRAP_TEST = "wrap-test";

    private final int c;
    private final double twoPiOverC;
    private final double xr, zr;

    public AnalyticTestTerrain(int circumference) {
        this.c = circumference;
        this.twoPiOverC = 2 * Math.PI / circumference;
        this.xr = circumference / 8.0;
        this.zr = circumference / 32.0;
    }

    public static AnalyticTestTerrain of(String kind, int circumference) {
        if (!WRAP_TEST.equals(kind))
            throw new IllegalArgumentException("unknown test terrain '" + kind + "' (known: " + WRAP_TEST + ")");
        return new AnalyticTestTerrain(circumference);
    }

    @Override
    public int circumference() {
        return c;
    }

    private double latitude(int z) {
        // Clamp beyond the poles (|z| >= C/4): the terrain holds its polar value there.
        double phi = Math.PI * z / (c / 2.0);
        return Math.max(-Math.PI / 2, Math.min(Math.PI / 2, phi));
    }

    @Override
    public double height(int x, int z) {
        double theta = x * twoPiOverC;
        double phi = latitude(z);
        double h = 63
                + 60 * Math.cos(theta - Math.PI) * Math.cos(phi)
                + 25 * Math.sin(3 * theta + 0.5) * Math.cos(2 * phi);
        double dx = x - xr;
        dx -= c * Math.rint(dx / c);            // periodic distance to the range
        double dz = z - zr;
        h += 450 * Math.exp(-(dx * dx) / (2 * 420.0 * 420.0) - (dz * dz) / (2 * 260.0 * 260.0));
        return h;
    }

    @Override
    public double raster(String layer, int x, int z) {
        return switch (layer) {
            // Hot at the equator, frozen at the poles.
            case "climate/temperature" -> 0.9 - 1.8 * Math.abs(latitude(z)) / (Math.PI / 2);
            // Land where H is above sea level; the coast falls on vanilla's coast band.
            case "climate/continents" -> Math.max(-1.0, Math.min(1.0, -0.15 + (height(x, z) - 63) / 80.0));
            case "climate/erosion" -> 0.2;
            // Mid weirdness: away from vanilla's river "valley" slice and its peaks.
            case "climate/ridges" -> 0.35;
            default -> 0.0;
        };
    }
}
