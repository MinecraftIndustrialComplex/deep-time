// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.GridDesc;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

/**
 * Keeps the mouths of the router's noise caves away from Deep Time's water (Ben, 2026-10-20:
 * "bodies of water just have these dirt walls around them, extremely unprofessional"). The
 * router's near-surface band, min(terrain, k/scale · 5 · entrances), opens vanilla's cave
 * entrances as pits from the surface down to about 12 blocks below it. Beside a river, a lake or
 * a shallow sea such a pit is a hole below the water's level that nothing fills: the aquifer
 * floods only where the ground is well under sea level, and {@link WaterBodies} then builds the
 * river's bed, its banks and the sea's floor up from the pit's bottom. That fill was the sheer
 * dirt wall, 10 to 27 blocks tall, around every body of water near a cave mouth (the play world
 * of 2026-10-19, region copies around x −393, z 225: water at y 62 on one block of dirt, air from
 * y 60 down to 51, grass on the pit's floor).
 *
 * <p>The rule, a function of (x, z) and the world data only: {@code shield(x, z)} is 1 within
 * {@value #FULL_BLOCKS} blocks of the water's edge and falls to 0 at {@value #FADE_BLOCKS}
 * (smoothstep), and the router adds {@value #LIFT} · shield to the entrance term, which closes
 * every cave mouth where the shield is 1 (5 · entrances is never below −{@value #LIFT} + 1.5625)
 * and lets them back in gradually. The ground there stays at H, so the water lies in its channel
 * and on its bed as the column rule says. Caves deeper than the band still run under the water
 * (Ben, decisions log 2026-09-30, "Allow caves under water"), and the carvers still cut under it.
 * The water counted is
 * <ul>
 *   <li>a river channel: its edge is the centreline's distance less the half-width
 *       ({@link Rivers#edgeDistance});</li>
 *   <li>a lake: a wet {@code water_level} cell, less the shore rule's reach (a lake column lies
 *       within that reach of one);</li>
 *   <li>the sea: an {@code elevation} cell whose E − A (the lowest the detail term can take the
 *       ground) is at or under the sea's top block, less {@value #SEA_REACH} blocks (a sea column,
 *       H bilinear between its noise cell's corners at or under the top, lies that close to one:
 *       a corner within 4√2 blocks, each corner's E − A bilinear between cells within 2√2). It is
 *       counted only for ground no more than {@value #SHALLOW_FULL_BLOCKS} blocks under the sea's
 *       top and fades out by {@value #SHALLOW_NONE_BLOCKS}: under deeper water vanilla's aquifer
 *       floods the cave mouths (its sea wherever a sampled preliminary surface, the highest
 *       8-block cell under H, lies 9 or more blocks under sea level), as on vanilla's ocean
 *       floors. A cell in a closed basin below sea level ({@code closed_basin}, HYD-100; ledger: HYD-103) is not
 *       sea: no sea stands there, and the basin's own lake, if any, counts as a lake.</li>
 * </ul>
 * Ledger HYD-078. Plain Java, no Minecraft, thread-safe.
 */
public final class WaterCaveShield {

    /** Full shield within this distance of the water's edge, blocks (game-choice, HYD-078). */
    public static final double FULL_BLOCKS = 8.0;
    /** No shield beyond this distance, blocks (game-choice, HYD-078). */
    public static final double FADE_BLOCKS = 16.0;
    /** The router's lift of the entrance term at shield 1 (planet.json; HYD-078). */
    public static final double LIFT = 16.0;
    /**
     * Ground at most this far under the sea's top block counts the sea fully, blocks (game-choice,
     * HYD-078): the aquifer floods a column's pits from its own sample once H ≤ 56 (6 under the
     * top), but it samples at its grid points, up to a cell away, so 2 blocks of margin.
     */
    public static final double SHALLOW_FULL_BLOCKS = 8.0;
    /** Ground this far or more under the sea's top block does not count the sea, blocks (game-choice, HYD-078). */
    public static final double SHALLOW_NONE_BLOCKS = 14.0;
    /** How near a sea column lies to an {@code elevation} cell with E − A at or under the sea's top, at most, blocks (HYD-078). */
    public static final double SEA_REACH = 8.5;
    /** The largest detail amplitude A, blocks (a u8 in sixteenths, FORMAT.md): cells with E − this above the top are not sea. */
    static final double MAX_AMPLITUDE = 255.0 / 16.0;

    private final WorldData wd;
    private final int seaTop;
    private final int reach;
    private final Rivers rivers;
    private final RasterSampler levels, elevation, amplitude, closed;
    /** Whether the amplitude layer shares the elevation layer's grid (both L3), so a cell's A is read by index. */
    private final boolean sameGrid;

    public WaterCaveShield(WorldData wd, Water.Spec spec) {
        this.wd = wd;
        this.seaTop = spec.seaLevel() - 1;
        this.reach = spec.reach();
        this.rivers = wd.rivers();
        this.levels = wd.has(WorldData.WATER_LEVEL) ? wd.layer(WorldData.WATER_LEVEL) : null;
        this.elevation = wd.has(WorldData.ELEVATION) ? wd.layer(WorldData.ELEVATION) : null;
        this.amplitude = wd.amplitudeLayer();
        this.closed = wd.has(WorldData.CLOSED_BASIN) ? wd.layer(WorldData.CLOSED_BASIN) : null;
        this.sameGrid = elevation != null && amplitude != null && sameGrid(elevation.desc().grid(), amplitude.desc().grid());
    }

    private static boolean sameGrid(GridDesc a, GridDesc b) {
        return a.r() == b.r() && a.originBlockX() == b.originBlockX() && a.originBlockZ() == b.originBlockZ();
    }

    /** 1 at or within {@link #FULL_BLOCKS} of the water's edge, 0 from {@link #FADE_BLOCKS}, smoothstep between. */
    public static double weight(double distance) {
        if (!(distance > FULL_BLOCKS))
            return 1.0;
        if (distance >= FADE_BLOCKS)
            return 0.0;
        double t = (distance - FULL_BLOCKS) / (FADE_BLOCKS - FULL_BLOCKS);
        return 1.0 - t * t * (3.0 - 2.0 * t);
    }

    /** How much the sea counts at ground height {@code ground}: 1 on land and over a shallow sea floor, 0 over deep water. */
    public static double seaWeight(double ground, int seaTop) {
        double depth = seaTop - ground;
        if (!(depth > SHALLOW_FULL_BLOCKS))
            return 1.0;
        if (depth >= SHALLOW_NONE_BLOCKS)
            return 0.0;
        double t = (depth - SHALLOW_FULL_BLOCKS) / (SHALLOW_NONE_BLOCKS - SHALLOW_FULL_BLOCKS);
        return 1.0 - t * t * (3.0 - 2.0 * t);
    }

    /** The shield from the distances to inland water (rivers, lakes) and to the sea, at ground height {@code ground}. */
    public static double combine(double inland, double sea, double ground, int seaTop) {
        return Math.max(weight(inland), weight(sea) * seaWeight(ground, seaTop));
    }

    /** The shield at column (x, z), any lap: 0 (cave mouths as vanilla's) to 1 (none). */
    public double at(long x, long z) {
        long c = wd.circumference();
        long wx = Math.floorMod(x + c / 2, c) - c / 2;
        double river = rivers == null ? Double.POSITIVE_INFINITY : rivers.edgeDistance(wx, z, FADE_BLOCKS);
        if (weight(river) >= 1.0)
            return 1.0;
        double inland = Math.min(river, lakeDistance(wx, z));
        double w = weight(inland);
        if (w >= 1.0 || elevation == null)
            return w;
        double sw = seaWeight(wd.height(wx, z), seaTop);
        if (sw <= w)
            return w;
        return Math.max(w, sw * weight(seaDistance(wx, z)));
    }

    /** Distance from block (x, z) to the nearest wet {@code water_level} cell, less the shore rule's reach; +∞ past the fade. */
    double lakeDistance(long x, long z) {
        if (levels == null)
            return Double.POSITIVE_INFINITY;
        double d = nearestCell(levels, x, z, FADE_BLOCKS + reach, false);
        return Math.max(0.0, d - reach);
    }

    /** Distance from block (x, z) to the nearest {@code elevation} cell that can be sea, less {@link #SEA_REACH}; +∞ past the fade. */
    double seaDistance(long x, long z) {
        double d = nearestCell(elevation, x, z, FADE_BLOCKS + SEA_REACH, true);
        return Math.max(0.0, d - SEA_REACH);
    }

    /**
     * The distance from block (x, z) to the nearest cell of {@code layer} within {@code radius}
     * blocks that is wet (a lake level, not NaN) or, with {@code sea}, whose E − A is under the sea's
     * top block; +∞ when none is.
     */
    private double nearestCell(RasterSampler layer, long x, long z, double radius, boolean sea) {
        GridDesc g = layer.desc().grid();
        int r = g.r();
        long rad = (long) Math.ceil(radius);
        long i0 = Math.floorDiv(x - rad - g.originBlockX(), r), i1 = Math.floorDiv(x + rad - g.originBlockX(), r) + 1;
        long j0 = Math.floorDiv(z - rad - g.originBlockZ(), r), j1 = Math.floorDiv(z + rad - g.originBlockZ(), r) + 1;
        double best2 = radius * radius;
        boolean found = false;
        for (long j = j0; j <= j1; j++) {
            double dz = g.originBlockZ() + j * (double) r - z;
            if (dz * dz > best2)
                continue;
            for (long i = i0; i <= i1; i++) {
                double dx = g.originBlockX() + i * (double) r - x;
                double d2 = dx * dx + dz * dz;
                if (d2 > best2 || (found && d2 >= best2))
                    continue;
                double v = layer.get(i, j);
                boolean hit;
                if (sea) {
                    // The amplitude is read only where the cell can be sea at all.
                    if (Double.isNaN(v) || v - MAX_AMPLITUDE > seaTop) {
                        hit = false;
                    } else {
                        double a = amplitude == null ? 0.0 : sameGrid ? amplitude.get(i, j)
                                : amplitude.nearest(g.originBlockX() + i * (double) r, g.originBlockZ() + j * (double) r);
                        hit = v - (Double.isNaN(a) ? 0.0 : a) <= seaTop
                                && (closed == null || closed.nearest(g.originBlockX() + i * (double) r, g.originBlockZ() + j * (double) r) != 1.0);
                    }
                } else {
                    hit = !Double.isNaN(v);
                }
                if (hit) {
                    best2 = d2;
                    found = true;
                }
            }
        }
        return found ? Math.sqrt(best2) : Double.POSITIVE_INFINITY;
    }
}
