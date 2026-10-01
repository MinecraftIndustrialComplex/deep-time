// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

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
 *   <li>a <b>magma chamber</b>: an ellipsoid of lava whose top lies {@code magma_chamber_depth_km}
 *       under the crater floor. Its radius is a tenth of the volcano's basal radius (1.2 to 4.5 km)
 *       times {@code magma_chamber_size}, its half-height a fifth of that radius. The world is 15
 *       times taller than wide, so in blocks a chamber is a tall lens (7 across and 40 high for a
 *       2-km chamber). A chamber that would reach the lava floor is raised, and squeezed where the
 *       ground is too thin for it;</li>
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
    public static final int COVER = 8;
    /** The plug: the conduit's lava stops this far under the crater floor. */
    public static final int PLUG = 6;
    /** Under water the plug stops this far under the bed. */
    public static final int BURIED = 3;
    /** A crater lake's radius, blocks: half the crater's, within these bounds. */
    public static final int LAKE_MIN_RADIUS = 2, LAKE_MAX_RADIUS = 10;
    /** The most ground a crater lake may cut away to sit level, blocks; a steeper summit keeps its plug. */
    public static final int LAKE_MAX_CUT = 12;
    /** A crater lake is at most this deep, blocks. */
    public static final int LAKE_MAX_DEPTH = 10;
    /** Ground above a lake's surface is opened up to this far above it. */
    public static final int LAKE_CLEAR = 24;
    /** Blocks kept between a chamber's bottom and the lava floor. */
    public static final int FLOOR_GAP = 8;

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
        NONE, CHAMBER, CHAMBER_SHELL, CONDUIT, CONDUIT_SHELL, PLUG, LAKE, LAKE_SHELL, LAKE_AIR;

        /** Lava. */
        public boolean molten() {
            return this == CHAMBER || this == CONDUIT || this == LAKE;
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
     */
    public record Body(int id, String kind, double activity, int x, int z, int floorY, int chamberY, int chamberRh,
                       int chamberRv, int conduitTop, int plugTop, boolean submerged, boolean lake, int lakeRadius,
                       int lakeTop, int lakeBottom, int reach) {

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
            int cheb = Math.max(Math.abs(dx), Math.abs(dz));
            if (cheb <= 2 && y >= chamberY) {
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
            return Part.NONE;
        }

        /** The lowest and highest y any part of the body may occupy. */
        public int minY() {
            return chamberY - chamberRv - 2;
        }

        public int maxY() {
            return Math.max(plugTop, lake ? lakeTop + LAKE_CLEAR : plugTop);
        }

        /** Lava blocks of the chamber (the ellipsoid's volume), for reports. */
        public long chamberBlocks() {
            return Math.round(4.0 / 3.0 * Math.PI * chamberRh * chamberRh * chamberRv);
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

    private record Draft(Volcanoes.Volcano v, int x, int z, int floorY, int cy, int rh, int rv, boolean submerged,
                         int lakeRadius, int lakeTop, boolean lakeFits) {}

    /**
     * Builds the world's bodies.
     *
     * @param lavaFloorY open space below this is the lava floor ({@link LavaRules#floodY})
     * @param mH         metres per block across, {@code mV} up
     * @param lakeMaxAbsZ crater lakes only at volcanoes with |z| up to this (the pole cap's line: nobody
     *                   reaches a volcano beyond it)
     */
    public static Magma build(List<Volcanoes.Volcano> volcanoes, MagmaRules rules, Heights heights, Wet wet, int circumference,
                              int seaLevel, int lavaFloorY, double mH, double mV, int lakeMaxAbsZ) {
        if (rules.chamberSize() <= 0.0 || volcanoes.isEmpty())
            return new Magma(List.of(), circumference);
        List<Draft> drafts = new ArrayList<>();
        for (Volcanoes.Volcano v : volcanoes) {
            int x = (int) Math.round(v.x()), z = (int) Math.round(v.z());
            if (circumference > 0)
                x = Math.floorMod(x + circumference / 2, circumference) - circumference / 2;
            int floorY = top(heights.at(x, z));
            double rhM = Math.clamp(0.1 * v.radiusM(), 1200.0, 4500.0) * rules.chamberSize();
            int rh = Math.max(2, (int) Math.round(rhM / mH));
            int rv = Math.max(4, (int) Math.round(0.2 * rhM / mV));
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
            drafts.add(new Draft(v, x, z, floorY, cy, rh, rv, submerged, r, lakeTop, fits));
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
            out.add(new Body(d.v.id(), d.v.kind(), d.v.activity(), d.x, d.z, d.floorY, d.cy, d.rh, d.rv, conduitTop, plugTop,
                    d.submerged, lake, d.lakeRadius, lake ? d.lakeTop : 0, lakeBottom, reach));
        }
        return new Magma(out, circumference);
    }
}
