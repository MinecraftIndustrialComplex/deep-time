// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Where a Deep Time world has magma: the bodies under and in its active volcanoes, in plain Java
 * (no Minecraft types, so JUnit can test it). Built once per level from the baked list of active
 * volcanoes ({@link Volcanoes}), the world's settings ({@link MagmaRules}) and the terrain height,
 * so every chunk and every thread sees the same bodies; {@link MagmaPass} places the blocks.
 *
 * <p>Each active volcano gets (FORMAT.md, "Lava"):
 * <ul>
 *   <li>a <b>crust-deep column</b> below the chamber (Ben, decisions log 2026-10-02, "Crust-deep magma
 *       system"; the transcrustal picture of Cashman, Sparks and Blundy 2017): lava <b>lenses</b>, wider
 *       than tall and a few blocks thick, stacked tens of blocks apart and growing with depth, joined by
 *       dike-like conduit segments, down to a basal lens that straddles the lava floor; set in a body of
 *       altered hot rock (the <b>mush</b>, {@link Body#mush}). Sizes follow the volcano's kind, basal
 *       radius and activity (ledger MAG-043 to MAG-062);</li>
 *   <li>a <b>magma chamber</b>: an ellipsoid of lava whose top lies {@code magma_chamber_depth_km}
 *       under the crater floor. Its radius is a tenth of the volcano's basal radius (1.2 to 4.5 km)
 *       times {@code magma_chamber_size}, its half-height a fifth of that radius. Heights are drawn
 *       15 times larger than widths (300 m across, 20 m up a block), so in blocks a chamber is a
 *       tall lens (a 2-km chamber is 7 blocks in radius, 13 to 15 across, and 40 high). A chamber
 *       that would reach the lava floor is raised, and squeezed where the ground is too thin for
 *       it (ledger MAG-002 to MAG-009);</li>
 *   <li>a <b>conduit</b>: a plus-shaped column of lava from the chamber's centre up to six blocks
 *       under the crater floor, under a plug of magma blocks. On land the plug reaches the
 *       surface (hot ground in the crater); under water it stops three blocks under the bed;</li>
 *   <li>at the most active volcanoes standing on dry land ({@code crater_lake_share} of them), a
 *       <b>crater lava lake</b> instead of the plug: lava up to one block under the lowest ground
 *       of the ring around it, the ground above its surface opened to the sky (baked summits are
 *       seldom bowls at this scale, so the lake usually sits in a pit a few blocks deep), the
 *       conduit opening into its floor. Volcanoes beyond the pole cap's line get none.</li>
 * </ul>
 * Every body is sealed: open space (air, cave water, the sea) within a block or two of magma
 * becomes basalt, so lava never touches water and never hangs over a cave. A volcano whose summit
 * is under the sea, or in a lake or river, gets a chamber and a buried conduit and no lava lake.
 * A dead volcano is not in the list and gets nothing.
 */
public final class Magma {

    /** No chamber lava within this many blocks of the ground above it (the pass checks the column). */
    public static final int COVER = 8; // ledger: MAG-010
    /** The plug: the conduit's lava stops this far under the crater floor. */
    public static final int PLUG = 6; // ledger: MAG-016
    /** Under water the plug stops this far under the bed. */
    public static final int BURIED = 3; // ledger: MAG-017
    /** A crater lake's radius, blocks: half the crater's, within these bounds. */
    public static final int LAKE_MIN_RADIUS = 2, LAKE_MAX_RADIUS = 10; // ledger: MAG-021
    /** The most ground a crater lake may cut away to sit level, blocks; a steeper summit keeps its plug. */
    public static final int LAKE_MAX_CUT = 12; // ledger: MAG-023
    /** A crater lake is at most this deep, blocks. */
    public static final int LAKE_MAX_DEPTH = 10; // ledger: MAG-022
    /** Ground above a lake's surface is opened up to this far above it. */
    public static final int LAKE_CLEAR = 24; // ledger: MAG-022
    /** Blocks kept between a chamber's bottom and the lava floor. */
    public static final int FLOOR_GAP = 8; // ledger: MAG-007

    /** Silicic centres hold larger shallow chambers: the chamber radius times this. */
    public static final double SILICIC_CHAMBER = 1.6; // ledger: MAG-050
    /** Lens radius at the top of the column, as a share of the chamber's radius, by kind. */
    public static final double LENS_SHARE_ARC = 1.0, LENS_SHARE_SILICIC = 0.9, LENS_SHARE_SHIELD = 1.4; // ledger: MAG-045
    /** An inactive volcano's lenses are this share of an active one's; the factor runs from here to 1 with activity. */
    public static final double LENS_MIN_ACTIVITY = 0.7; // ledger: MAG-046
    /** The lowest lens is this many times wider than the highest. */
    public static final double LENS_GROWTH = 2.2; // ledger: MAG-047
    /** Smallest and largest lens radius, blocks. */
    public static final int LENS_MIN_RADIUS = 5, LENS_MAX_RADIUS = 90; // ledger: MAG-048
    /** A lens's half-height is this share of its radius, within 2 to 5 blocks: wider than tall, a few blocks thick. */
    public static final double LENS_FLATNESS = 0.2; // ledger: MAG-044
    public static final int LENS_MIN_HALF = 2, LENS_MAX_HALF = 5;
    /** The first lens lies this share of the spacing under the chamber's bottom. */
    public static final double FIRST_GAP = 0.8; // ledger: MAG-049
    /** A lens's centre may lie this share of its radius off the column's axis (the conduits follow). */
    public static final double LENS_OFFSET = 0.15; // ledger: MAG-049
    /** Each lens's radius is drawn between these shares of the column's radius at its depth. */
    public static final double LENS_JITTER_LO = 0.9, LENS_JITTER_HI = 1.05; // ledger: MAG-049
    /** Each middle lens may lie this share of the spacing off its even position. */
    public static final double LENS_SPREAD = 0.2; // ledger: MAG-049
    /** The basal lens's centre lies this far above the lava floor (blocks), never closer than {@link #BEDROCK_GAP} to the world's floor. */
    public static final int BASAL_RISE = 2, BEDROCK_GAP = 6; // ledger: MAG-053
    /** The mush's radius: this times the column's radius, plus 3 blocks, rising to the second at a lens. */
    public static final double MUSH_BASE = 1.25, MUSH_BULGE = 0.25; // ledger: MAG-054
    public static final int MUSH_RIM = 3;
    /** Where the shield, and where the silicic share of the column differ: the mush is this much broader under a shield. */
    public static final double MUSH_SHIELD = 1.1; // ledger: MAG-054

    /** Terrain height H(x, z), blocks. */
    @FunctionalInterface
    public interface Heights {
        double at(int x, int z);
    }

    /** Whether column (x, z) holds or borders water (the sea, a lake, a river, a bank). */
    @FunctionalInterface
    public interface Wet {
        boolean at(int x, int z);
    }

    /** What a block of a body is. */
    public enum Part {
        NONE, CHAMBER, CHAMBER_SHELL, CONDUIT, CONDUIT_SHELL, PLUG, LAKE, LAKE_SHELL, LAKE_AIR, LENS, LENS_SHELL;

        /** Lava. */
        public boolean molten() {
            return this == CHAMBER || this == CONDUIT || this == LAKE || this == LENS;
        }
    }

    /**
     * One volcano's magma.
     *
     * @param x          the crater's centre column (x canonical)
     * @param floorY     the top block of the crater floor there
     * @param chamberY   the chamber's centre
     * @param chamberRh  its horizontal radius, blocks; 0: the volcano has no magma (too little rock)
     * @param chamberRv  its vertical half-height, blocks
     * @param conduitTop the conduit's highest lava block
     * @param plugTop    the highest block the plug may reach ({@code conduitTop} with a lake)
     * @param submerged  the summit is under or beside water
     * @param lake       it holds a crater lava lake …
     * @param lakeRadius … of this radius, lava from {@code lakeBottom} to {@code lakeTop}
     * @param reach      horizontal radius that holds everything, blocks
     * @param deep       the crust-deep column under the chamber (stacked lenses, conduits and the body of
     *                   altered hot rock around them); null: the volcano has none
     */
    public record Body(int id, String kind, double activity, int x, int z, int floorY, int chamberY, int chamberRh,
                       int chamberRv, int conduitTop, int plugTop, boolean submerged, boolean lake, int lakeRadius,
                       int lakeTop, int lakeBottom, int reach, Deep deep) {

        /** What the block at offset (dx, dz) from the centre column and height y is. */
        public Part at(int dx, int dz, int y) {
            if (chamberRh <= 0)
                return Part.NONE;
            long d2 = (long) dx * dx + (long) dz * dz;
            if (lake) {
                int r = lakeRadius;
                if (d2 <= (long) r * r) {
                    if (y >= lakeBottom && y <= lakeTop)
                        return Part.LAKE;
                    if (y > lakeTop && y <= lakeTop + LAKE_CLEAR)
                        return Part.LAKE_AIR;
                    if (y == lakeBottom - 1 && Math.abs(dx) + Math.abs(dz) > 1)
                        return Part.LAKE_SHELL;
                } else if (d2 <= (long) (r + 2) * (r + 2) && y >= lakeBottom - 1 && y <= lakeTop) {
                    return Part.LAKE_SHELL;
                }
            }
            double qy = (double) (y - chamberY) / chamberRv;
            if ((double) d2 / ((double) chamberRh * chamberRh) + qy * qy <= 1.0)
                return Part.CHAMBER;
            Part deepPart = deep == null ? Part.NONE : deep.at(dx, dz, y);
            if (deepPart.molten())
                return deepPart;
            int cheb = Math.max(Math.abs(dx), Math.abs(dz));
            if (cheb <= 2 && y >= (deep == null ? chamberY : deep.axisBottom)) {
                if (Math.abs(dx) + Math.abs(dz) <= 1) {
                    if (y <= conduitTop)
                        return Part.CONDUIT;
                    if (y <= plugTop)
                        return Part.PLUG;
                } else if (y <= conduitTop + 1) {
                    return Part.CONDUIT_SHELL;
                }
            }
            double sh = chamberRh + 1.5, sv = chamberRv + 1.5, sy = (y - chamberY) / sv;
            if (d2 / (sh * sh) + sy * sy <= 1.0)
                return Part.CHAMBER_SHELL;
            return deepPart;
        }

        /** Whether the block at offset (dx, dz) and height y lies in the body of altered hot rock around the column. */
        public boolean mush(int dx, int dz, int y) {
            return chamberRh > 0 && deep != null && deep.mush(dx, dz, y);
        }

        /** The lowest and highest y any part of the body may occupy. */
        public int minY() {
            return deep != null ? Math.min(chamberY - chamberRv - 2, deep.bottom) : chamberY - chamberRv - 2;
        }

        public int maxY() {
            return Math.max(plugTop, lake ? lakeTop + LAKE_CLEAR : plugTop);
        }

        /** Lava blocks of the chamber (the ellipsoid's volume), for reports. */
        public long chamberBlocks() {
            return Math.round(4.0 / 3.0 * Math.PI * chamberRh * chamberRh * chamberRv);
        }

        /** Lava blocks of the column's lenses and conduit segments (volumes, before the cover rule), for reports. */
        public long columnBlocks() {
            return deep == null ? 0 : deep.moltenBlocks();
        }

        /** Number of lenses in the column. */
        public int lensCount() {
            return deep == null ? 0 : deep.n;
        }
    }

    /**
     * The crust-deep column under a chamber: {@code n} lava lenses from the highest to the basal one,
     * joined by dike-like conduit segments, and the body of altered hot rock around them. Immutable.
     * Arrays are indexed from the highest lens (index 0) to the lowest.
     */
    public static final class Deep {
        final int n;
        final int[] ly, lox, loz, lrh, lrv;
        /** Lowest y of any part; the highest y of a lens or segment part. */
        final int bottom, top;
        /** The main axis (the chamber's conduit) runs down to this y: the highest lens's centre. */
        final int axisBottom;
        /** The mush: highest and lowest y, and the squared radius at each (negative: none). */
        final int mushTop, mushBottom;
        final long[] mushR2;
        final int reach;

        Deep(int[] ly, int[] lox, int[] loz, int[] lrh, int[] lrv, int mushTop, int mushBottom, long[] mushR2, int mushReach) {
            this.n = ly.length;
            this.ly = ly;
            this.lox = lox;
            this.loz = loz;
            this.lrh = lrh;
            this.lrv = lrv;
            this.axisBottom = ly[0];
            int lo = Integer.MAX_VALUE, hi = Integer.MIN_VALUE, rc = 0;
            for (int k = 0; k < n; k++) {
                lo = Math.min(lo, ly[k] - lrv[k] - 2);
                hi = Math.max(hi, ly[k] + lrv[k] + 2);
                rc = Math.max(rc, Math.max(Math.abs(lox[k]), Math.abs(loz[k])) + lrh[k] + 2);
            }
            this.mushTop = mushTop;
            this.mushBottom = mushBottom;
            this.mushR2 = mushR2;
            this.bottom = mushR2 == null ? lo : Math.min(lo, mushBottom);
            this.top = hi;
            this.reach = Math.max(rc, mushR2 == null ? 0 : mushReach);
        }

        /** Index of the first lens whose centre is at or below y (n when none). */
        private int first(int y) {
            int lo = 0, hi = n;
            while (lo < hi) {
                int mid = (lo + hi) >>> 1;
                if (ly[mid] <= y)
                    hi = mid;
                else
                    lo = mid + 1;
            }
            return lo;
        }

        /** The lens or conduit part at an offset from the body's centre column, or NONE (the mush is {@link #mush}). */
        Part at(int dx, int dz, int y) {
            if (y > top || y < bottom)
                return Part.NONE;
            int k0 = first(y);
            Part best = Part.NONE;
            for (int k = Math.max(0, k0 - 4), e = Math.min(n - 1, k0 + 3); k <= e; k++) {
                int ddy = y - ly[k];
                if (Math.abs(ddy) > lrv[k] + 2)
                    continue;
                long ex = dx - lox[k], ez = dz - loz[k], d2 = ex * ex + ez * ez;
                double q = (double) ddy / lrv[k];
                if ((double) d2 / ((double) lrh[k] * lrh[k]) + q * q <= 1.0)
                    return Part.LENS;
                double sh = lrh[k] + 1.5, sv = lrv[k] + 1.5, sy = ddy / sv;
                if (d2 / (sh * sh) + sy * sy <= 1.0)
                    best = Part.LENS_SHELL;
            }
            for (int k = Math.max(0, k0 - 5), e = Math.min(n - 2, k0 + 3); k <= e; k++) {
                if (y > ly[k] + 1 || y < ly[k + 1] - 1)
                    continue;
                int ex = dx - lox[k], ez = dz - loz[k];
                if (Math.abs(ex) > 2 || Math.abs(ez) > 2)
                    continue;
                if (Math.abs(ex) + Math.abs(ez) <= 1 && y <= ly[k] && y >= ly[k + 1])
                    return Part.CONDUIT;
                if (best == Part.NONE)
                    best = Part.CONDUIT_SHELL;
            }
            return best;
        }

        boolean mush(int dx, int dz, int y) {
            if (mushR2 == null || y > mushTop || y < mushBottom)
                return false;
            return (long) dx * dx + (long) dz * dz <= mushR2[y - mushBottom];
        }

        /** Volume of the lenses plus the conduit segments between them, blocks (before the cover rule). */
        long moltenBlocks() {
            double v = 0;
            for (int k = 0; k < n; k++) {
                v += 4.0 / 3.0 * Math.PI * lrh[k] * lrh[k] * lrv[k];
                if (k + 1 < n)
                    v += 5.0 * Math.max(0, ly[k] - ly[k + 1]);
            }
            return Math.round(v);
        }

        /** The lenses' (y, radius, half-height), for reports. */
        public int[][] lenses() {
            int[][] out = new int[n][];
            for (int k = 0; k < n; k++)
                out[k] = new int[]{ly[k], lrh[k], lrv[k], lox[k], loz[k]};
            return out;
        }
    }

    public static final Magma NONE = new Magma(List.of(), 0);

    private final List<Body> bodies;
    private final int circumference;

    private Magma(List<Body> bodies, int circumference) {
        this.bodies = List.copyOf(bodies);
        this.circumference = circumference;
    }

    public List<Body> bodies() {
        return bodies;
    }

    /** x − bx folded into [−C/2, C/2): the offset east of a body's centre across the seam. */
    public int dx(int x, Body b) {
        int d = x - b.x;
        if (circumference <= 0)
            return d;
        return Math.floorMod(d + circumference / 2, circumference) - circumference / 2;
    }

    /** The bodies whose reach touches block box [x0, x1] × [z0, z1]. */
    public List<Body> near(int x0, int z0, int x1, int z1) {
        List<Body> out = null;
        for (Body b : bodies) {
            if (b.chamberRh <= 0 || b.z + b.reach < z0 || b.z - b.reach > z1)
                continue;
            int lo = dx(x0, b), hi = lo + (x1 - x0);
            if (hi < -b.reach || lo > b.reach)
                continue;
            if (out == null)
                out = new ArrayList<>(2);
            out.add(b);
        }
        return out == null ? List.of() : out;
    }

    /** What block (x, y, z) is, over all bodies (the first that claims it). */
    public Part at(int x, int y, int z) {
        for (Body b : bodies) {
            if (b.chamberRh <= 0 || Math.abs(z - b.z) > b.reach)
                continue;
            int dx = dx(x, b);
            if (Math.abs(dx) > b.reach)
                continue;
            Part p = b.at(dx, z - b.z, y);
            if (p != Part.NONE)
                return p;
        }
        return Part.NONE;
    }

    /** The top block of ground of height h: ⌈h⌉ − 1. */
    static int top(double h) {
        return (int) Math.ceil(h) - 1;
    }

    private static double unit(long seed, int k, int salt) {
        return (PaletteRules.mix(seed ^ ((long) k << 8) ^ salt) >>> 11) / (double) (1L << 53);
    }

    private static int lensHalf(int r) {
        return (int) Math.clamp(Math.round(LENS_FLATNESS * r), LENS_MIN_HALF, LENS_MAX_HALF);
    }

    /**
     * The crust-deep column of one volcano: lenses from {@code FIRST_GAP} spacings under the chamber's
     * bottom to a basal lens just above the lava floor, evenly spread (each a little off its place and
     * its axis), wider with depth, with the conduit segments between them and the mush around; null where
     * the chamber already sits at the floor.
     */
    private static Deep buildDeep(Volcanoes.Volcano v, MagmaRules rules, int cy, int rh, int rv, int lavaFloorY, int minY) {
        double share = "shield".equals(v.kind()) ? LENS_SHARE_SHIELD : v.silicic() ? LENS_SHARE_SILICIC : LENS_SHARE_ARC;
        double act = LENS_MIN_ACTIVITY + (1.0 - LENS_MIN_ACTIVITY) * Math.clamp(v.activity(), 0.0, 1.0);
        double f = rules.lensWidth() * share * act;
        int rTop = (int) Math.clamp(Math.round(rh * f), LENS_MIN_RADIUS, LENS_MAX_RADIUS);
        int rBot = (int) Math.clamp(Math.round(rTop * LENS_GROWTH), rTop, LENS_MAX_RADIUS);
        int basalY = Math.max(lavaFloorY + BASAL_RISE, minY + BEDROCK_GAP + lensHalf(rBot));
        double spacing = rules.lensSpacingBlocks();
        int yFirst = cy - rv - (int) Math.round(FIRST_GAP * spacing);
        int span = yFirst - basalY;
        int n;
        if (cy - rv - 3 <= basalY + lensHalf(rBot) + 2)
            return null;                      // no room under the chamber
        if (span < 6 || rules.lensCount() == 1) {
            n = 1;
            yFirst = basalY;
            span = 0;
        } else {
            n = (int) Math.clamp(Math.round(span / spacing) + 1, 2, rules.lensCount());
        }
        double sa = n > 1 ? (double) span / (n - 1) : 0.0;
        long seed = PaletteRules.mix(0x6D61676D614C6E73L ^ ((long) v.id() << 20));
        int[] ly = new int[n], lox = new int[n], loz = new int[n], lrh = new int[n], lrv = new int[n];
        for (int k = 0; k < n; k++) {
            double t = n == 1 ? 1.0 : (double) k / (n - 1);
            int y = k == 0 || k == n - 1 ? (k == 0 && n > 1 ? yFirst : basalY)
                    : (int) Math.round(yFirst - t * span + (unit(seed, k, 1) - 0.5) * 2.0 * LENS_SPREAD * sa);
            if (k > 0)
                y = Math.min(y, ly[k - 1] - 2);
            ly[k] = y;
            double env = rTop + (rBot - rTop) * t;
            int r = k == n - 1 ? rBot : k == 0 ? rTop
                    : Math.max(LENS_MIN_RADIUS, (int) Math.round(env * (LENS_JITTER_LO + (LENS_JITTER_HI - LENS_JITTER_LO) * unit(seed, k, 2))));
            lrh[k] = r;
            lrv[k] = lensHalf(r);
            if (k > 0) {
                double a = 2.0 * Math.PI * unit(seed, k, 3), m = LENS_OFFSET * r * unit(seed, k, 4);
                lox[k] = (int) Math.round(Math.cos(a) * m);
                loz[k] = (int) Math.round(Math.sin(a) * m);
            }
        }
        // The body of altered hot rock: from the chamber's equator down to just under the basal lens.
        long[] mush = null;
        int mushTop = cy, mushBottom = ly[n - 1] - lrv[n - 1] - 2, mushReach = 0;
        double mw = rules.mushWidth();
        if (mw > 0.0 && mushTop > mushBottom) {
            mush = new long[mushTop - mushBottom + 1];
            double kind = "shield".equals(v.kind()) ? MUSH_SHIELD : 1.0;
            double yTop = ly[0];
            int step = Math.max(1, n > 1 ? (int) Math.round(sa) : 8);
            for (int y = mushBottom; y <= mushTop; y++) {
                double env;
                if (y >= yTop)
                    env = rTop + (rh - rTop) * Math.min(1.0, (y - yTop) / Math.max(1.0, cy - yTop));
                else
                    env = rTop + (rBot - rTop) * Math.min(1.0, (yTop - y) / Math.max(1.0, (double) span));
                double near = Double.MAX_VALUE;
                for (int k = 0; k < n; k++)
                    near = Math.min(near, Math.abs(y - ly[k]));
                double bump = Math.max(0.0, 1.0 - near / (0.5 * step));
                double dome = Math.sqrt(Math.clamp((cy - y) / (double) Math.max(2, rv), 0.0, 1.0));
                double rad = mw * kind * ((MUSH_BASE + MUSH_BULGE * bump) * env + MUSH_RIM) * dome;
                mush[y - mushBottom] = (long) (rad * rad);
                mushReach = Math.max(mushReach, (int) Math.ceil(rad) + 1);
            }
        }
        return new Deep(ly, lox, loz, lrh, lrv, mushTop, mushBottom, mush, mushReach);
    }

    private record Draft(Volcanoes.Volcano v, int x, int z, int floorY, int cy, int rh, int rv, boolean submerged,
                         int lakeRadius, int lakeTop, boolean lakeFits, Deep deep) {}

    /**
     * Builds the world's bodies.
     *
     * @param lavaFloorY open space below this is the lava floor ({@link LavaRules#floodY})
     * @param minY       the world's floor (the level's {@code min_y}): the column stops short of its bedrock
     * @param mH         metres per block across, {@code mV} up
     * @param lakeMaxAbsZ crater lakes only at volcanoes with |z| up to this (the pole cap's line: nobody
     *                   reaches a volcano beyond it)
     */
    public static Magma build(List<Volcanoes.Volcano> volcanoes, MagmaRules rules, Heights heights, Wet wet, int circumference,
                              int seaLevel, int lavaFloorY, double mH, double mV, int lakeMaxAbsZ) {
        return build(volcanoes, rules, heights, wet, circumference, seaLevel, lavaFloorY, lavaFloorY - 10, mH, mV, lakeMaxAbsZ);
    }

    /** As {@link #build(List, MagmaRules, Heights, Wet, int, int, int, double, double, int)} with the world's floor given. */
    public static Magma build(List<Volcanoes.Volcano> volcanoes, MagmaRules rules, Heights heights, Wet wet, int circumference,
                              int seaLevel, int lavaFloorY, int minY, double mH, double mV, int lakeMaxAbsZ) {
        if (rules.chamberSize() <= 0.0 || volcanoes.isEmpty())
            return new Magma(List.of(), circumference);
        List<Draft> drafts = new ArrayList<>();
        for (Volcanoes.Volcano v : volcanoes) {
            int x = (int) Math.round(v.x()), z = (int) Math.round(v.z());
            if (circumference > 0)
                x = Math.floorMod(x + circumference / 2, circumference) - circumference / 2;
            int floorY = top(heights.at(x, z));
            double rhM = Math.clamp(0.1 * v.radiusM(), 1200.0, 4500.0) * rules.chamberSize()
                    * (v.silicic() ? SILICIC_CHAMBER : 1.0); // ledger: MAG-003, MAG-050
            int rh = Math.max(2, (int) Math.round(rhM / mH));
            int rv = Math.max(4, (int) Math.round(0.2 * rhM / mV)); // ledger: MAG-004
            int cy = floorY - (int) Math.round(rules.chamberDepthKm() * 1000.0 / mV) - rv;
            int low = lavaFloorY + FLOOR_GAP, high = floorY - 3 * COVER;
            if (cy - rv < low)
                cy = low + rv;
            if (cy + rv > high) {
                rv = (high - low) / 2;
                cy = low + rv;
            }
            if (rv < 3)
                rh = 0;   // no room between the ground and the lava floor
            boolean submerged = floorY <= seaLevel + 2;
            for (int dx = -2; dx <= 2 && !submerged; dx++)
                for (int dz = -2; dz <= 2 && !submerged; dz++)
                    submerged = wet.at(x + dx, z + dz);
            // A lake's level: one block under the lowest ground of the ring around it.
            int r = v.craterRadiusM() > 0.0
                    ? Math.clamp(Math.round(0.5 * v.craterRadiusM() / mH), LAKE_MIN_RADIUS, LAKE_MAX_RADIUS) : LAKE_MIN_RADIUS;
            int lakeTop = Integer.MAX_VALUE;
            boolean fits = rh > 0 && !submerged && Math.abs(z) <= lakeMaxAbsZ;
            for (int dx = -r - 2; dx <= r + 2 && fits; dx++)
                for (int dz = -r - 2; dz <= r + 2 && fits; dz++) {
                    int d2 = dx * dx + dz * dz;
                    if (d2 > (r + 2) * (r + 2))
                        continue;
                    if (wet.at(x + dx, z + dz))
                        fits = false;
                    else if (d2 > r * r)
                        lakeTop = Math.min(lakeTop, top(heights.at(x + dx, z + dz)) - 1);
                }
            fits = fits && lakeTop != Integer.MAX_VALUE && lakeTop > seaLevel + 2 && floorY - lakeTop <= LAKE_MAX_CUT;
            Deep deep = rh > 0 && rules.hasColumn() ? buildDeep(v, rules, cy, rh, rv, lavaFloorY, minY) : null;
            drafts.add(new Draft(v, x, z, floorY, cy, rh, rv, submerged, r, lakeTop, fits, deep));
        }
        // The most active volcanoes that can hold a lake get one.
        List<Draft> able = new ArrayList<>(drafts.stream().filter(Draft::lakeFits).toList());
        able.sort(Comparator.comparingDouble((Draft d) -> -d.v.activity()).thenComparingInt(d -> d.v.id()));
        int lakes = (int) Math.ceil(rules.craterLakeShare() * able.size() - 1e-9);
        List<Draft> withLake = able.subList(0, Math.min(lakes, able.size()));
        List<Body> out = new ArrayList<>();
        for (Draft d : drafts) {
            boolean lake = withLake.contains(d);
            int lakeBottom = 0, conduitTop, plugTop;
            if (lake) {
                lakeBottom = Math.max(Math.min(d.floorY, d.lakeTop) - 2, d.lakeTop - LAKE_MAX_DEPTH + 1);
                conduitTop = lakeBottom - 1;
                plugTop = conduitTop;
            } else {
                conduitTop = d.floorY - PLUG;
                plugTop = d.submerged ? d.floorY - BURIED : d.floorY + 4;
            }
            int reach = Math.max(d.rh + 2, lake ? d.lakeRadius + 2 : 2);
            if (d.deep != null)
                reach = Math.max(reach, d.deep.reach);
            out.add(new Body(d.v.id(), d.v.kind(), d.v.activity(), d.x, d.z, d.floorY, d.cy, d.rh, d.rv, conduitTop, plugTop,
                    d.submerged, lake, d.lakeRadius, lake ? d.lakeTop : 0, lakeBottom, reach, d.deep));
        }
        return new Magma(out, circumference);
    }
}
