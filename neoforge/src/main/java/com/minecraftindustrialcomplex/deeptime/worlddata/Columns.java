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

    /** Cells in the column rule's stencil (4 × 4). */
    public static final int CELLS = 16;
    /** The stencil index of cell (i0, j0), the reference of the weighted mean. */
    private static final int REF = 5;
    /** Keys per band of an interfingering facies change where the layer does not say (Rust {@code strata::BAND_KEYS}). */
    public static final int BAND_KEYS = 64;
    /** A class's share below which it takes no part of a band, where the layer does not say (Rust {@code strata::MINOR_SHARE}). */
    public static final double MINOR_SHARE = 0.15;

    /**
     * One column's horizons, top-down: integer tops, class indexes and unit keys (the key top of
     * the stored horizon a horizon came from; {@code StrataPass} picks a class's member by it). The
     * arrays grow as needed; {@link #count} are used. Also caches the stencil of the last L1 cell
     * square it was asked about, so the 256 columns of a chunk read the stacks once. One per thread.
     */
    public static class Stack {
        public int[] tops;
        public int[] classes;
        public int[] units;
        public int count;

        // The cached stencil: the layer and square it was built for.
        private RasterSampler cachedLayer;
        private long cachedI0 = Long.MIN_VALUE, cachedJ0 = Long.MIN_VALUE;
        private int cachedMinY;
        private int m;                         // keys
        private int[] keys = new int[64];
        private double[] y = new double[CELLS * 64];            // y[cn * keysCap + q]
        private int keysCap = 64;
        private final int[][] cTop = new int[CELLS][], cClass = new int[CELLS][], cKt = new int[CELLS][],
                cKb = new int[CELLS][];
        private final int[] cN = new int[CELLS];
        private int[] over = new int[CELLS * 64];                // over[q * CELLS + cn], −1 = none
        // Scratch for one interval.
        private final int[] eOrder = new int[CELLS], eClass = new int[CELLS], eUnit = new int[CELLS];
        private final double[] eThick = new double[CELLS], share = new double[CELLS];
        private final double[] w = new double[CELLS];

        public Stack(int k) {
            int cap = Math.max(8, 4 * k);
            tops = new int[cap];
            classes = new int[cap];
            units = new int[cap];
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

        /** Rust {@code strata::emit}. */
        void emit(double top, int cls, int unit) {
            int t = (int) Math.floor(top);
            if (count > 0) {
                int last = tops[count - 1];
                t = Math.min(t, last);
                if (last == t)
                    count--;
            }
            if (count > 0 && classes[count - 1] == cls)
                return;
            if (count == tops.length) {
                tops = java.util.Arrays.copyOf(tops, 2 * count);
                classes = java.util.Arrays.copyOf(classes, 2 * count);
                units = java.util.Arrays.copyOf(units, 2 * count);
            }
            tops[count] = t;
            classes[count] = cls;
            units[count] = unit;
            count++;
        }

        /** Rust {@code strata::height_at}: cell cn's height at key s. */
        private double heightAt(int cn, int s, double floor, double scale, double offset) {
            int n = cN[cn];
            if (n == 0)
                return floor;
            int[] kt = cKt[cn], kb = cKb[cn], tp = cTop[cn];
            if (s <= kt[0])
                return tp[0] * scale + offset;
            int h = 0;
            while (h + 1 < n && kt[h + 1] <= s)
                h++;
            double t = tp[h] * scale + offset;
            double b = h + 1 < n ? tp[h + 1] * scale + offset : floor;
            if (s >= kb[h])
                return b;
            return t - (t - b) * ((double) (s - kt[h]) / (double) (kb[h] - kt[h]));
        }

        /** Rust {@code strata::horizon_over}: cell cn's horizon over [s0, s1], or −1. */
        private int horizonOver(int cn, int s0, int s1) {
            int n = cN[cn];
            int[] kt = cKt[cn];
            if (n == 0 || s0 < kt[0])
                return -1;
            int h = 0;
            while (h + 1 < n && kt[h + 1] <= s0)
                h++;
            return s1 <= cKb[cn][h] ? h : -1;
        }

        /** Reads the 16 stacks of the square (i0, j0) and everything the columns in it share. */
        private void build(RasterSampler strata, int minY, long i0, long j0) {
            int k = strata.stackDepth();
            int total = 0;
            for (int b = 0; b < 4; b++) {
                for (int a = 0; a < 4; a++) {
                    int cn = b * 4 + a;
                    long i = i0 - 1 + a, j = j0 - 1 + b;
                    if (cTop[cn] == null || cTop[cn].length < k) {
                        cTop[cn] = new int[k];
                        cClass[cn] = new int[k];
                        cKt[cn] = new int[k];
                        cKb[cn] = new int[k];
                    }
                    int n = 0;
                    for (int h = 0; h < k; h++) {
                        int cls = strata.horizonClass(i, j, h);
                        short top = strata.horizonTopRaw(i, j, h);
                        if (cls == 255 || top == Short.MIN_VALUE)
                            break;
                        cTop[cn][n] = top;
                        cClass[cn][n] = cls;
                        cKt[cn][n] = strata.horizonKeyTop(i, j, h);
                        cKb[cn][n] = strata.horizonKeyBase(i, j, h);
                        n++;
                    }
                    cN[cn] = n;
                    total += 2 * n;
                }
            }
            if (keys.length < total)
                keys = new int[Math.max(total, 2 * keys.length)];
            int cnt = 0;
            for (int cn = 0; cn < CELLS; cn++)
                for (int h = 0; h < cN[cn]; h++) {
                    keys[cnt++] = cKt[cn][h];
                    keys[cnt++] = cKb[cn][h];
                }
            java.util.Arrays.sort(keys, 0, cnt);
            int u = 0;
            for (int q = 0; q < cnt; q++)
                if (u == 0 || keys[q] != keys[u - 1])
                    keys[u++] = keys[q];
            m = u;
            if (keysCap < m) {
                keysCap = Math.max(m, 2 * keysCap);
                y = new double[CELLS * keysCap];
                over = new int[CELLS * keysCap];
            }
            var d = strata.desc();
            double floor = minY;
            for (int cn = 0; cn < CELLS; cn++)
                for (int q = 0; q < m; q++)
                    y[cn * keysCap + q] = heightAt(cn, keys[q], floor, d.scale(), d.offset());
            for (int q = 0; q + 1 < m; q++)
                for (int cn = 0; cn < CELLS; cn++)
                    over[q * CELLS + cn] = horizonOver(cn, keys[q], keys[q + 1]);
            cachedLayer = strata;
            cachedMinY = minY;
            cachedI0 = i0;
            cachedJ0 = j0;
        }

        private double height(int q) {
            double base = y[REF * keysCap + q];
            double sum = 0.0;
            for (int cn = 0; cn < CELLS; cn++)
                sum += w[cn] * (y[cn * keysCap + q] - base);
            return base + sum;
        }

        /** Fills this stack for block (x, z): Rust {@code strata::stack_at}. */
        void fill(WorldData wd, RasterSampler strata, int x, int z) {
            count = 0;
            var grid = strata.desc().grid();
            long c = (long) grid.w() * grid.r();
            long wxl = Math.floorMod(x + c / 2, c) - c / 2;
            double u = (double) (wxl - grid.originBlockX()) / grid.r();
            double v = (double) ((long) z - grid.originBlockZ()) / grid.r();
            long i0 = (long) Math.floor(u), j0 = (long) Math.floor(v);
            double[] bu = bspline(u - i0), bv = bspline(v - j0);
            for (int b = 0; b < 4; b++)
                for (int a = 0; a < 4; a++)
                    w[b * 4 + a] = bu[a] * bv[b];
            int minY = wd.manifest().minY();
            if (cachedLayer != strata || cachedI0 != i0 || cachedJ0 != j0 || cachedMinY != minY)
                build(strata, minY, i0, j0);
            if (m == 0)
                return;
            if (m == 1) {
                for (int cn = 0; cn < CELLS; cn++)
                    if (cN[cn] > 0) {
                        emit(height(0), cClass[cn][0], cKt[cn][0]);
                        break;
                    }
                return;
            }
            double top = height(0);
            for (int q = 0; q + 1 < m; q++) {
                int s0 = keys[q], s1 = keys[q + 1];
                double bottom = height(q + 1);
                int ne = 0;
                for (int cn = 0; cn < CELLS; cn++) {
                    if (w[cn] <= 0.0)
                        continue;
                    int h = over[q * CELLS + cn];
                    if (h < 0)
                        continue;
                    double t = w[cn] * (y[cn * keysCap + q] - y[cn * keysCap + q + 1]);
                    if (t <= 0.0)
                        continue;
                    int cls = cClass[cn][h], kt = cKt[cn][h];
                    int order = kt + cKb[cn][h];
                    int e = 0;
                    while (e < ne && eClass[e] != cls)
                        e++;
                    if (e < ne) {
                        eOrder[e] = Math.min(eOrder[e], order);
                        eUnit[e] = Math.min(eUnit[e], kt);
                        eThick[e] += t;
                    } else {
                        eOrder[ne] = order;
                        eClass[ne] = cls;
                        eUnit[ne] = kt;
                        eThick[ne] = t;
                        ne++;
                    }
                }
                if (ne == 1) {
                    emit(top, eClass[0], eUnit[0]);
                } else if (ne > 1) {
                    // Insertion sort by (order, class); classes are distinct.
                    for (int p = 1; p < ne; p++) {
                        int o = eOrder[p], cl = eClass[p], un = eUnit[p];
                        double th = eThick[p];
                        int r = p - 1;
                        while (r >= 0 && (eOrder[r] > o || (eOrder[r] == o && eClass[r] > cl))) {
                            eOrder[r + 1] = eOrder[r];
                            eClass[r + 1] = eClass[r];
                            eUnit[r + 1] = eUnit[r];
                            eThick[r + 1] = eThick[r];
                            r--;
                        }
                        eOrder[r + 1] = o;
                        eClass[r + 1] = cl;
                        eUnit[r + 1] = un;
                        eThick[r + 1] = th;
                    }
                    interfinger(ne, s0, s1, top, bottom, strata.desc().tongueBandKeys(),
                            strata.desc().tongueMinorShare());
                }
                top = bottom;
            }
            int[] minBlocks = strata.desc().minBlocks();
            if (minBlocks != null)
                dropSlivers(minBlocks);
        }

        /**
         * Rust {@code strata::drop_slivers}, the column rule's step 5: from the second horizon down, a
         * horizon that is not the last and is thinner than its class's least thickness is removed (the
         * one above reaches down in its place), and so is one of the class of the last one kept.
         */
        private void dropSlivers(int[] minBlocks) {
            int n = count;
            int kept = Math.min(1, n);
            for (int h = 1; h < n; h++) {
                int cls = classes[h];
                int need = cls < minBlocks.length ? minBlocks[cls] : 1;
                if (h + 1 < n && need > 1 && tops[h] - tops[h + 1] < need)
                    continue;
                if (classes[kept - 1] == cls)
                    continue;
                tops[kept] = tops[h];
                classes[kept] = cls;
                units[kept] = units[h];
                kept++;
            }
            count = kept;
        }

        /** Rust {@code strata::interfinger}. */
        private void interfinger(int ne, int s0, int s1, double top, double bottom, int bandKeys, double minor) {
            double total = 0.0;
            for (int e = 0; e < ne; e++)
                total += eThick[e];
            double kept = 0.0;
            for (int e = 0; e < ne; e++) {
                share[e] = Math.max(eThick[e] / total - minor, 0.0);
                kept += share[e];
            }
            if (kept <= 0.0) {
                kept = 0.0;
                for (int e = 0; e < ne; e++) {
                    share[e] = eThick[e] / total;
                    kept += share[e];
                }
            }
            double k0 = s0, k1 = s1, band = bandKeys;
            for (int n = s0 / bandKeys; n <= (s1 - 1) / bandKeys; n++) {
                double base = n * bandKeys;
                double sum = 0.0;
                for (int e = 0; e < ne; e++) {
                    double lo = base + band * (sum / kept);
                    sum += share[e];
                    double hi = base + band * (sum / kept);
                    lo = Math.max(lo, k0);
                    hi = Math.min(hi, k1);
                    if (lo < hi)
                        emit(top - (lo - k0) / (k1 - k0) * (top - bottom), eClass[e], eUnit[e]);
                }
            }
        }
    }

    /**
     * The uniform cubic B-spline weights of the cells i0 − 1 … i0 + 2 at fraction {@code f} of the
     * way from i0 to i0 + 1: Rust {@code strata::bspline}, the same operations.
     */
    public static double[] bspline(double f) {
        double g = 1.0 - f;
        double f2 = f * f;
        double f3 = f2 * f;
        return new double[]{g * g * g / 6.0, (3.0 * f3 - 6.0 * f2 + 4.0) / 6.0,
                (-3.0 * f3 + 3.0 * f2 + 3.0 * f + 1.0) / 6.0, f3 / 6.0};
    }

    /**
     * Fills {@code out} with the stack at block (x, z) by the column rule of format 0.2 (FORMAT.md,
     * "{@code horizon_stack} cells"): the 4 × 4 strata cells around, B-spline weights, their
     * stacks correlated by stratigraphic key, facies changes intertongued. Rust
     * {@code deeptime_format::strata::stack_at}.
     */
    public static void stack(WorldData wd, RasterSampler strata, int x, int z, Stack out) {
        out.fill(wd, strata, x, z);
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
     * @param horizons  [top y, class index, unit key] per horizon, top-down
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
            horizons[h] = new int[]{st.tops[h], st.classes[h], st.units[h]};
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
