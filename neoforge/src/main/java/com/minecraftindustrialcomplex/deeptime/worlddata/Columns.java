// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.util.ArrayList;
import java.util.List;

/**
 * The column evaluator (plan §6.1, §8.8, §9.2; FORMAT.md, "Columns"): what Deep Time decides for
 * one block column — the terrain height, the water surface, the horizon stack, the bodies the
 * column passes through and the rock from the terrain's top block down to the floor. Rust's
 * {@code deeptime_format::column} is its twin, operation for operation: {@code deeptime parity}
 * writes Rust's answers for 500 columns as {@code parity/columns.json}, {@code ColumnParityTest}
 * asserts this class gives the same, and the strata pass ({@code worldgen/StrataPass}) builds its
 * columns with {@link #stack}. Plain Java, no Minecraft: thread-safe over an open {@link WorldData}.
 */
public final class Columns {

    private Columns() {}

    /** One column's horizons, top-down: integer tops and class indexes. */
    public static class Stack {
        public final int[] tops;
        public final int[] classes;
        public int count;

        public Stack(int k) {
            tops = new int[k];
            classes = new int[k];
        }

        /** The index of the horizon holding block y, or −1 above the first horizon's top. */
        public int horizonAt(int y) {
            if (count == 0 || y > tops[0])
                return -1;
            int h = 0;
            while (h + 1 < count && y <= tops[h + 1])
                h++;
            return h;
        }
    }

    /**
     * Fills {@code out} with the stack at block (x, z): each horizon's class from the nearest cell,
     * its top bilinear between the four cells around (the nearest cell's top where a corner lacks
     * the horizon), floored to a block and never above the horizon over it; the walk stops at the
     * first unused horizon of the nearest cell. Rust {@code deeptime_format::raster::stack_at}.
     */
    public static void stack(WorldData wd, RasterSampler strata, int x, int z, Stack out) {
        var grid = strata.desc().grid();
        int c = wd.circumference();
        int k = Math.min(strata.stackDepth(), out.tops.length);
        double wx = Math.floorMod(x + c / 2, c) - c / 2;
        double u = (wx - grid.originBlockX()) / grid.r();
        double v = (z - grid.originBlockZ()) / (double) grid.r();
        long in = (long) Math.floor(u + 0.5), jn = (long) Math.floor(v + 0.5);
        long i0 = (long) Math.floor(u), j0 = (long) Math.floor(v);
        double fu = u - i0, fv = v - j0;
        int used = 0;
        int prev = Integer.MAX_VALUE;
        for (int h = 0; h < k; h++) {
            int cls = strata.horizonClass(in, jn, h);
            double nearest = strata.horizonTop(in, jn, h);
            if (cls == 255 || Double.isNaN(nearest))
                break;
            double t00 = strata.horizonTop(i0, j0, h), t10 = strata.horizonTop(i0 + 1, j0, h);
            double t01 = strata.horizonTop(i0, j0 + 1, h), t11 = strata.horizonTop(i0 + 1, j0 + 1, h);
            double top = Double.isNaN(t00) || Double.isNaN(t10) || Double.isNaN(t01) || Double.isNaN(t11) ? nearest
                    : (t00 + (t10 - t00) * fu) * (1 - fv) + (t01 + (t11 - t01) * fu) * fv;
            int t = Math.min((int) Math.floor(top), prev);   // tops never rise with depth
            out.tops[used] = t;
            out.classes[used] = cls;
            prev = t;
            used++;
        }
        out.count = used;
    }

    /** A run of blocks, top y down to bottom y (inclusive), of one rock class (null = unclassified crust). */
    public record Run(int top, int bottom, String rockClass) {}

    /** A maximal run of a column's blocks inside one body. */
    public record BodyRun(int id, int top, int bottom) {}

    /**
     * One column as the evaluator decides it.
     *
     * @param x         as asked
     * @param wx        x folded into [−C/2, C/2)
     * @param elevation E: the {@code elevation} layer, bilinear at (wx, z)
     * @param detail    A·N, the detail term (0 without one)
     * @param surface   H = E + A·N
     * @param topY      ⌈H⌉ − 1 clamped to [min_y − 1, max_y]: the terrain's top block at a noise-cell corner
     * @param water     the {@code water_level} layer, nearest at (wx, z); NaN where it has none
     * @param horizons  [top y, class index] per horizon, top-down
     * @param bodies    runs inside bodies, ascending body id, each body's runs top-down
     * @param rock      the rock from {@code topY} down to min_y: a body's class (a zone's winning) over the horizon's
     * @param hydro     the column's lake, river, sea or bank ({@link Water}, P1-22)
     */
    public record Sample(int x, int z, int wx, double elevation, double detail, double surface, int topY, double water,
                         int[][] horizons, List<BodyRun> bodies, List<Run> rock, Water.Column hydro) {}

    /** The noise cell width of the {@code deeptime:planet} generator the parity fixture assumes. */
    public static final int NOISE_CELL = 4;

    /** Evaluates column (x, z) of {@code wd}. */
    public static Sample sample(WorldData wd, int x, int z) {
        Manifest m = wd.manifest();
        int c = wd.circumference();
        int minY = m.minY(), maxY = m.maxY();
        int wx = Math.floorMod(x + c / 2, c) - c / 2;
        double bx = wx, bz = z;
        double elevation = wd.elevation(bx, bz);
        double detail = wd.detail(bx, bz);
        double surface = wd.detailNoise() == null ? elevation : elevation + detail;
        int topY;
        if (Double.isFinite(surface)) {
            long t = (long) Math.ceil(surface) - 1;
            topY = (int) Math.max(minY - 1L, Math.min(maxY, t));
        } else {
            topY = minY - 1;
        }
        double water = wd.has(WorldData.WATER_LEVEL) ? wd.layer(WorldData.WATER_LEVEL).nearest(bx, bz) : Double.NaN;

        int[][] horizons;
        Stack st;
        if (wd.has(WorldData.HORIZONS)) {
            RasterSampler strata = wd.layer(WorldData.HORIZONS);
            st = new Stack(strata.stackDepth());
            stack(wd, strata, x, z, st);
        } else {
            st = new Stack(0);
        }
        horizons = new int[st.count][];
        for (int h = 0; h < st.count; h++)
            horizons[h] = new int[]{st.tops[h], st.classes[h]};
        List<String> classes = wd.has(WorldData.HORIZONS) ? wd.classes() : List.of();

        // Bodies whose box footprint holds the column, ascending id.
        List<Body> near = new ArrayList<>();
        for (Body b : wd.deposits().bodies().bodies()) {
            if (z < b.min()[2] || z > b.max()[2])
                continue;
            long span = (long) b.max()[0] - b.min()[0];
            if (span >= c - 1L || Math.floorMod((long) wx - b.min()[0], (long) c) <= span)
                near.add(b);
        }
        near.sort((a, b) -> Integer.compareUnsigned(a.id(), b.id()));

        List<BodyRun> bodies = new ArrayList<>();
        for (Body b : near) {
            int lo = Math.max(b.min()[1], minY), hi = Math.min(b.max()[1], maxY);
            int runTop = Integer.MIN_VALUE, runBottom = 0;
            for (int y = hi; y >= lo; y--) {
                boolean inside = b.locate(c, wx, y, z) != null;
                if (inside) {
                    if (runTop == Integer.MIN_VALUE)
                        runTop = y;
                    runBottom = y;
                } else if (runTop != Integer.MIN_VALUE) {
                    bodies.add(new BodyRun(b.id(), runTop, runBottom));
                    runTop = Integer.MIN_VALUE;
                }
            }
            if (runTop != Integer.MIN_VALUE)
                bodies.add(new BodyRun(b.id(), runTop, runBottom));
        }

        List<Run> rock = new ArrayList<>();
        for (int y = topY; y >= minY; y--) {
            int h = st.horizonAt(y);
            String cls = null;
            if (h >= 0) {
                int idx = st.classes[h];
                cls = idx < classes.size() ? classes.get(idx) : null;
            }
            for (Body b : near) {
                Body.Location at = b.locate(c, wx, y, z);
                if (at == null)
                    continue;
                String bc = b.rockClass();
                for (Body.Zone zone : b.zones())
                    if (zone.rockClass() != null && zone.contains(at.d(), at.v()))
                        bc = zone.rockClass();
                if (bc != null)
                    cls = bc;
            }
            Run last = rock.isEmpty() ? null : rock.getLast();
            if (last != null && java.util.Objects.equals(last.rockClass(), cls))
                rock.set(rock.size() - 1, new Run(last.top(), y, cls));
            else
                rock.add(new Run(y, y, cls));
        }
        return new Sample(x, z, wx, elevation, detail, surface, topY, water, horizons, List.copyOf(bodies),
                List.copyOf(rock), Water.column(wd, Water.Spec.of(wd, NOISE_CELL), x, z));
    }
}
