// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.util.List;

/**
 * One body of {@code bodies/bodies.dtb} (plan §5.7.4): a parametric primitive with a transform, an
 * AABB and zones. {@link #locate} is Rust's {@code Body::locate} (FORMAT.md, "Bodies": the
 * evaluation Java must do exactly), operation for operation; {@code DepositDataParityTest} compares
 * the two on every body of a synthetic deposit world.
 *
 * @param id        body id
 * @param site      the site it belongs to, or {@link SiteData#NONE}
 * @param shape     shape code (0 ellipsoid … 10 dome)
 * @param role      {@code ore}, {@code lithocap}, {@code gossan}, …
 * @param center    block space; x in the canonical span
 * @param axes      semi-axes a (local x), b (local y, vertical half-height), c (local z), blocks
 * @param clipTopY  blocks above it are outside, or {@link #NO_CLIP}
 * @param min       inclusive AABB min (x may leave the canonical span)
 * @param max       inclusive AABB max
 * @param rockClass the rock class the whole body's host becomes, or null
 * @param ageMa     when the body formed, Myr before the end of the world's history, or NaN (FORMAT.md)
 */
public record Body(int id, int site, int shape, String role, double[] center, float[] axes, float yaw, float dip,
                   int clipTopY, int[] min, int[] max, String rockClass, List<Zone> zones, float ageMa) {

    /** A body with no age (deposit bodies, tests). */
    public Body(int id, int site, int shape, String role, double[] center, float[] axes, float yaw, float dip,
                int clipTopY, int[] min, int[] max, String rockClass, List<Zone> zones) {
        this(id, site, shape, role, center, axes, yaw, dip, clipTopY, min, max, rockClass, zones, Float.NaN);
    }

    public static final int NO_CLIP = Integer.MAX_VALUE;

    /** Shape names by code (FORMAT.md). */
    public static final List<String> SHAPES = List.of("ellipsoid", "stockwork_shell", "lens", "pod", "sheet", "dyke",
            "pipe", "blanket", "seam", "channel", "dome");

    /**
     * One zone. {@code material} null: an alteration or cap zone (rock only). {@code density} is the
     * probability that a host block in the zone becomes ore; {@code raw} the share of those ore
     * blocks that are the material's raw-ore block instead (vanilla's ore veins; FORMAT.md).
     */
    public record Zone(String name, String material, String rockClass, float d0, float d1, float v0, float v1,
                       float grade, float density, float raw) {

        /** Rust's {@code Zone::contains}: [lo, hi), an upper bound ≥ 1 inclusive. */
        public boolean contains(double d, double v) {
            return within(d, d0, d1) && within(v, v0, v1);
        }

        private static boolean within(double x, float lo, float hi) {
            double l = lo, h = hi;
            return x >= l && (x < h || (h >= 1.0 && x <= h));
        }
    }

    /** Where a block centre lies in a body: normalised distance (≤ 1) and vertical position (0 top, 1 bottom). */
    public record Location(double d, double v) {}

    /** The ellipsoid metric's shapes (0 ellipsoid, 1 stockwork_shell, 2 lens, 3 pod, 10 dome). */
    public boolean ellipsoidMetric() {
        return shape <= 3 || shape == 10;
    }

    public String shapeName() {
        return shape >= 0 && shape < SHAPES.size() ? SHAPES.get(shape) : "shape" + shape;
    }

    /**
     * Where the centre of block (x, y, z) lies in the body on a planet of circumference {@code c}
     * (x wraps), or null outside it. The steps and their order are FORMAT.md's.
     */
    public Location locate(int c, int x, int y, int z) {
        if (clipTopY != NO_CLIP && y > clipTopY)
            return null;
        double cf = c;
        double dx = (double) x + 0.5 - center[0];
        dx -= Math.floor(dx / cf + 0.5) * cf;
        double dy = (double) y + 0.5 - center[1];
        double dz = (double) z + 0.5 - center[2];
        double yawD = yaw, dipD = dip;
        double sy = Math.sin(yawD), cy = Math.cos(yawD);
        double sd = Math.sin(dipD), cd = Math.cos(dipD);
        // Rows of the world → local rotation (Rust `to_local`), each dotted with (dx, dy, dz).
        double lx = cy * dx + 0.0 * dy + sy * dz;
        double ly = sd * sy * dx + cd * dy + -sd * cy * dz;
        double lz = -cd * sy * dx + sd * dy + cd * cy * dz;
        double a = Math.max(axes[0], 1e-6), b = Math.max(axes[1], 1e-6), cc = Math.max(axes[2], 1e-6);
        double d;
        if (ellipsoidMetric()) {
            double qa = lx / a, qb = ly / b, qc = lz / cc;
            d = Math.sqrt(qa * qa + qb * qb + qc * qc);
        } else {
            double qa = lx / a, qc = lz / cc;
            d = Math.max(Math.sqrt(qa * qa + qc * qc), Math.abs(ly / b));
        }
        if (d > 1.0)
            return null;
        double v = Math.max(0.0, Math.min(1.0, (b - ly) / (2.0 * b)));
        return new Location(d, v);
    }

    /** True when block (x, y, z) lies in the AABB (x wraps). */
    public boolean aabbContains(int c, int x, int y, int z) {
        if (y < min[1] || y > max[1] || z < min[2] || z > max[2])
            return false;
        long span = (long) max[0] - min[0];
        return span >= c - 1L || Math.floorMod((long) x - min[0], (long) c) <= span;
    }

    /** True when the AABB overlaps the block box [x0, x1] × [y0, y1] × [z0, z1] (x wraps). */
    public boolean aabbOverlaps(int c, int x0, int y0, int z0, int x1, int y1, int z1) {
        if (max[1] < y0 || min[1] > y1 || max[2] < z0 || min[2] > z1)
            return false;
        long a0 = min[0], a1 = max[0];
        if (a1 - a0 >= c - 1L || (long) x1 - x0 >= c - 1L)
            return true;
        long shift = Math.floorDiv((long) x0 - a0, (long) c) * c;
        for (long k = -1; k <= 1; k++) {
            long b0 = a0 + shift + k * c, b1 = a1 + shift + k * c;
            if (b1 >= x0 && b0 <= x1)
                return true;
        }
        return false;
    }
}
