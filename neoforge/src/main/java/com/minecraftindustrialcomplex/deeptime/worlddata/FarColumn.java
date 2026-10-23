// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * One block column as far terrain shows it (plan §10.5 P3-04): the column rule of the world data
 * ({@link Water}, {@link Columns}, {@link IceCover}) turned into a short list of vertical layers,
 * without generating a chunk. Distant Horizons' level-of-detail data is exactly such a list per
 * column ({@code compat/dh}), so this is the column-to-LOD conversion.
 *
 * <p>Two steps, so the game can look up the biome between them:
 * <ol>
 *   <li>{@link #shape}: the geometry — the top solid block, the water over it, the ice cover. For
 *       a column one block wide it is {@link Water}'s whole decision (lake, sea, river with its
 *       cut bed, a bank raised or lowered beside water), which is what {@code worldgen/WaterBodies}
 *       makes every generated column match. For a wider column (a coarser level of detail) it is
 *       the same decision at the column's centre, without the bank rule: a bank is one block
 *       wide.</li>
 *   <li>{@link #layers}: the layers, bottom-up and without gaps from {@code min_y} to the top of
 *       the build range: rock by the strata column rule ({@link Columns.Stack}), the soil the biome
 *       gives, the ice cover over the top blocks ({@code worldgen/IcePass}), water, the ice on
 *       frozen water, the snow layer on frozen ground, air.</li>
 * </ol>
 *
 * <p><b>Rock.</b> A far column's sides show wherever a neighbour stands lower, so the strata are
 * kept bed by bed down to {@code rockDepth} blocks below the soil (the caller passes the drop to the
 * lowest neighbour, plus a margin); below that the column is one run of the rock at that depth.
 * Bodies (intrusions, halos) are not drawn: they are rare at the surface and need the body index.
 *
 * <p>Materials are abstract ({@link #ROCK}, {@link #SOIL_TOP}, …); the game resolves them to blocks
 * with the rock palette, the far-terrain surface table and the ice cover's spec. Plain Java, no
 * Minecraft; the scratch objects ({@link Shape}, {@link Layers}) belong to one thread each.
 */
public final class FarColumn {

    private FarColumn() {}

    /** Layer materials. */
    public static final int AIR = 0;
    /** Rock of the class {@link Layers#cls} (−1: the unclassified crust), unit key {@link Layers#unit}. */
    public static final int ROCK = 1;
    /** The biome's top block (grass, sand, …). */
    public static final int SOIL_TOP = 2;
    /** The biome's block under the top (dirt, sandstone, …). */
    public static final int SOIL_UNDER = 3;
    /** The floor of the sea: the biome's underwater block (gravel, sand). */
    public static final int SEA_BED = 4;
    /** The floor of a lake: the biome's top block as it is under water (grass becomes dirt). */
    public static final int LAKE_BED = 5;
    /** A river's bed: gravel in a narrow channel. */
    public static final int RIVER_GRAVEL = 6;
    /** A river's bed: sand in a wide channel ({@link #WIDE_RIVER_HALF_WIDTH}). */
    public static final int RIVER_SAND = 7;
    public static final int WATER = 8;
    /** The frozen top block of water in a freezing biome. */
    public static final int WATER_ICE = 9;
    /** The ice cover's layers ({@link IceCover.Spec#SNOW}, {@code ICE}, {@code DEEP_ICE}). */
    public static final int COVER_SNOW = 10, COVER_ICE = 11, COVER_DEEP_ICE = 12;
    /** The one-block snow layer vanilla lays on frozen ground. */
    public static final int SNOW_LAYER = 13;

    /** Number of materials. */
    public static final int MATERIALS = 14;

    /** A river at least this many blocks half-wide has a sand bed ({@code WaterBodies}: {@code hw >= 3}). */
    public static final double WIDE_RIVER_HALF_WIDTH = 3.0;

    /** A column's geometry. */
    public static final class Shape {
        /** What the column is: land ({@code NONE}), a bank, a river, a lake or the sea. */
        public Water.Kind kind = Water.Kind.NONE;
        /** The y of the top solid block: a river's or lake's bed under water; below min_y for a column without ground. */
        public int ground;
        /** The y of the top water block; meaningful where {@link #kind} is water. */
        public int waterTop;
        /** A river wide enough for a sand bed. */
        public boolean wideRiver;
        /**
         * The bed is a river channel's: a river column, or a sea or lake column whose bed a channel
         * cuts under its ground ({@link Water#cutByChannel}); its top is the channel's sediment.
         */
        public boolean channel;
        /** Blocks of ice cover ({@link IceCover#blocks}); 0 for none, and never laid under water. */
        public int ice;
        /**
         * A bank whose ground is the water's beside it ({@code minGround}): the bank pass fills
         * whatever is open up to there, so water a surface rule paints in place of its top block
         * becomes the block under it (WaterBodies, BAK-067).
         */
        public boolean sealed;

        public boolean water() {
            return kind.isWater() && waterTop > ground;
        }
    }

    /** What the biome gives a column. */
    public static final class Soil {
        /** Blocks of soil, the top block included; ignored when {@link #bare}. */
        public int depth = 3;
        /** No soil: the rock itself is the surface (stony peaks and shores). */
        public boolean bare;
        /** The biome freezes here: water gets an ice top, ground a snow layer. */
        public boolean frozen;
        /**
         * The level's surface rule paints water in place of the ground's top block (vanilla's swamp
         * water at y 62, ledger BAK-067): that block is water, the soil or rock under it shows, and
         * no ice cover is laid on it.
         */
        public boolean paintedWater;
    }

    /** A column's layers, bottom-up: layer i fills the blocks {@code bottom[i] <= y < top[i]}. */
    public static final class Layers {
        public int count;
        public int[] bottom = new int[16], top = new int[16], material = new int[16];
        /** For {@link #ROCK}: the class index (−1 the unclassified crust) and the unit key of its horizon. */
        public int[] cls = new int[16], unit = new int[16];

        void add(int bottomY, int topY, int mat, int c, int u) {
            if (topY <= bottomY)
                return;
            if (count > 0 && material[count - 1] == mat && cls[count - 1] == c && unit[count - 1] == u
                    && top[count - 1] == bottomY) {
                top[count - 1] = topY;
                return;
            }
            if (count == bottom.length) {
                int n = 2 * count;
                bottom = java.util.Arrays.copyOf(bottom, n);
                top = java.util.Arrays.copyOf(top, n);
                material = java.util.Arrays.copyOf(material, n);
                cls = java.util.Arrays.copyOf(cls, n);
                unit = java.util.Arrays.copyOf(unit, n);
            }
            bottom[count] = bottomY;
            top[count] = topY;
            material[count] = mat;
            cls[count] = c;
            unit[count] = u;
            count++;
        }

        /** The index of the layer holding block y, or −1. */
        public int at(int y) {
            for (int i = 0; i < count; i++)
                if (y >= bottom[i] && y < top[i])
                    return i;
            return -1;
        }

        /** The index of the highest layer that is neither air, water, water ice nor a snow layer; −1 for none. */
        public int topSolid() {
            for (int i = count - 1; i >= 0; i--) {
                int m = material[i];
                if (m != AIR && m != WATER && m != WATER_ICE && m != SNOW_LAYER)
                    return i;
            }
            return -1;
        }
    }

    private static int clampGround(int ground, int minY, int maxY) {
        return Math.max(minY - 1, Math.min(maxY, ground));
    }

    /**
     * The geometry of a one-block column from {@link Water}'s decision for it ({@link Water#column}
     * or {@link Water.Region#column}) and its ice cover: what the generated chunk holds there
     * before carvers and features.
     */
    public static void shape(Water.Column col, int iceBlocks, int minY, int maxY, Shape out) {
        out.kind = col.kind();
        out.wideRiver = false;
        out.sealed = false;
        out.channel = col.kind() == Water.Kind.RIVER || Water.cutByChannel(col);
        int ground = col.ground();
        if (col.kind().isWater()) {
            out.waterTop = col.top();
            ground = col.bed();
            if (out.channel && col.river() != null)
                out.wideRiver = col.river().hw() >= WIDE_RIVER_HALF_WIDTH;
        } else {
            out.waterTop = Integer.MIN_VALUE;
            // A bank: lowered to the channel's 1:1 slope, then raised to the water beside it
            // (worldgen/WaterBodies does the two in this order).
            if (col.maxGround() != null && ground > col.maxGround() && col.maxGround() >= minY)
                ground = col.maxGround();
            if (col.minGround() != null && ground < col.minGround() && col.minGround() <= maxY)
                ground = col.minGround();
            // Water painted on the top block leaves the ground one lower when the pass reads it,
            // below minGround wherever the ground stands at minGround: the pass raises it again.
            out.sealed = col.minGround() != null && ground == col.minGround() && ground <= maxY;
        }
        out.ground = clampGround(ground, minY, maxY);
        out.ice = out.water() ? 0 : Math.max(0, iceBlocks);
    }

    /**
     * The geometry of a column wider than a block, from {@link Water#core} at its centre (the
     * terrain height there, the lake, sea or river over it); no bank rule.
     */
    public static void shape(Water.Core core, int iceBlocks, int minY, int maxY, Shape out) {
        out.kind = core.kind();
        out.wideRiver = false;
        out.sealed = false;
        out.channel = core.kind() == Water.Kind.RIVER || Water.cutByChannel(core);
        int ground = core.ground();
        if (core.kind().isWater()) {
            out.waterTop = core.top();
            ground = core.bed();
            Rivers.Hit hit = core.rivers().hit();
            if (out.channel && hit != null)
                out.wideRiver = hit.hw() >= WIDE_RIVER_HALF_WIDTH;
        } else {
            out.waterTop = Integer.MIN_VALUE;
        }
        out.ground = clampGround(ground, minY, maxY);
        out.ice = out.water() ? 0 : Math.max(0, iceBlocks);
    }

    /** The class index and unit key of the horizon holding block y in {@code rock}: {−1, 0} above its first horizon or without one. */
    private static void rockAt(Columns.Stack rock, int y, int[] out) {
        int h = rock == null ? -1 : rock.horizonAt(y);
        out[0] = h < 0 ? -1 : rock.classes[h];
        out[1] = h < 0 ? 0 : rock.units[h];
    }

    /**
     * Fills {@code out} with the column's layers.
     *
     * @param minY      the bottom of the build range (the first layer starts here)
     * @param maxY      the highest block of the build range (the last layer, air, ends at maxY + 1)
     * @param rock      the column's horizon stack ({@link Columns#stack}), or null for a world without strata
     * @param rockDepth how far below the soil the strata are kept bed by bed (≥ 0)
     * @param cover     the ice cover's spec, or null (then {@link Shape#ice} is ignored)
     * @param soil      the biome's soil; with {@link Soil#paintedWater} the ground's top block is water
     *                  (the soil's top material is not drawn, what lies under it is), under the sea or a
     *                  lake too, and no ice cover is laid (the ice pass leaves a fluid surface alone)
     */
    public static void layers(int minY, int maxY, Shape s, Soil soil, Columns.Stack rock, int rockDepth,
                              IceCover.Spec cover, Layers out) {
        out.count = 0;
        int g = clampGround(s.ground, minY, maxY);
        int solid = g - minY + 1;                                   // blocks of ground
        boolean water = s.water();
        // The surface rule's water in place of the top block (BAK-067): the ground under it is one lower,
        // and the ice pass leaves a column whose surface is fluid as it is (IcePass), so no cover.
        boolean painted = soil.paintedWater && solid > 0;
        int coverN = cover == null || water || painted ? 0 : Math.min(Math.max(0, s.ice), solid);
        int soilN = soil.bare ? 0 : Math.min(Math.max(1, soil.depth), solid);
        // A river's bed is one block of gravel or sand on whatever the ground is.
        if (water && s.channel)
            soilN = Math.max(soilN, Math.min(1, solid));
        int rockTop = g - Math.max(Math.max(soilN, coverN), painted ? 1 : 0);   // the highest rock block
        int[] r = new int[2];
        if (rockTop >= minY) {
            int detailFrom = Math.max(minY, rockTop - Math.max(0, rockDepth));
            if (detailFrom > minY) {
                rockAt(rock, detailFrom, r);
                out.add(minY, detailFrom, ROCK, r[0], r[1]);
            }
            // Bed by bed from detailFrom up: a run ends at its horizon's top.
            int y = detailFrom;
            while (y <= rockTop) {
                int h = rock == null ? -1 : rock.horizonAt(y);
                int end = h < 0 ? rockTop : Math.min(rockTop, rock.tops[h]);
                if (end < y)
                    end = rockTop;   // a malformed stack: never loop
                out.add(y, end + 1, ROCK, h < 0 ? -1 : rock.classes[h], h < 0 ? 0 : rock.units[h]);
                y = end + 1;
            }
        }
        // Soil between the rock and the cover.
        int soilTopY = g - coverN;                                  // the highest soil block left under the cover
        if (soilN > coverN) {
            int topMaterial = !water ? SOIL_TOP
                    : s.channel ? (s.wideRiver ? RIVER_SAND : RIVER_GRAVEL)
                    : s.kind == Water.Kind.SEA ? SEA_BED : LAKE_BED;
            boolean topIsTheSurface = coverN == 0;
            int underTop = topIsTheSurface ? soilTopY : soilTopY + 1;   // exclusive
            out.add(g - soilN + 1, underTop, SOIL_UNDER, 0, 0);
            if (topIsTheSurface && !painted)
                out.add(soilTopY, soilTopY + 1, topMaterial, 0, 0);
        }
        // The ice cover replaces the top blocks, snow uppermost (IceCover.Spec.kind counts from the top).
        for (int y = g - coverN + 1; y <= g; y++) {
            int kind = cover.kind(g - y, coverN);
            out.add(y, y + 1, kind == IceCover.Spec.SNOW ? COVER_SNOW : kind == IceCover.Spec.DEEP_ICE ? COVER_DEEP_ICE : COVER_ICE,
                    0, 0);
        }
        int top = painted ? g : g + 1;                              // the first block above the ground
        if (water) {
            int wt = Math.min(s.waterTop, maxY);
            if (soil.frozen) {
                out.add(top, wt, WATER, 0, 0);
                out.add(wt, wt + 1, WATER_ICE, 0, 0);
            } else {
                out.add(top, wt + 1, WATER, 0, 0);
            }
            top = wt + 1;
        } else if (painted) {
            // Dry ground: the one block of water, frozen as a lake's top freezes.
            out.add(top, top + 1, soil.frozen ? WATER_ICE : WATER, 0, 0);
            top++;
        } else if (soil.frozen && solid > 0 && top <= maxY) {
            out.add(top, top + 1, SNOW_LAYER, 0, 0);
            top++;
        }
        out.add(top, maxY + 1, AIR, 0, 0);
    }

    /** The widest column that still gets the full strata column rule ({@link #rock}). */
    public static final int FULL_RULE_MAX_STEP = 2;

    /**
     * The horizon stack of a column {@code step} blocks wide at canonical block (wx, z).
     *
     * <p>Up to {@link #FULL_RULE_MAX_STEP} blocks wide it is the column rule itself
     * ({@link Columns#stack}: the 4 × 4 strata cells around, beds correlated and intertongued), so
     * far rock is the rock the strata pass paints. A wider column takes the stack of its nearest
     * strata cell as stored: the rule's blend between cells moves a bed's edge by less than such a
     * column is wide, and its stencil is built per strata cell, which costs a wide column (one new
     * cell each) ten times what it costs a chunk's columns (one cell for many).
     *
     * <p>Both are read through the world's folds, faults and intrusions where it has any (P2-01,
     * {@link Structure#deformedStack}): a narrow column exactly as the strata pass reads it
     * ({@link Columns#deformedStack}), a wide one with the same events undone over nearest-cell
     * stacks ({@link #cellStack}), so a dyke, a fault's offset or a fold seen from afar is where
     * it is close up.
     */
    public static void rock(WorldData wd, RasterSampler strata, long wx, long z, int step, Columns.Stack out) {
        rock(wd, strata, wx, z, step <= FULL_RULE_MAX_STEP, out);
    }

    /**
     * As {@link #rock(WorldData, RasterSampler, long, long, int, Columns.Stack)}: the column rule
     * when {@code rule}, else the nearest strata cell's stack. The game also takes the cheap stack
     * for a narrow column whose rock nobody sees (soil on top, no side showing below the soil).
     */
    public static void rock(WorldData wd, RasterSampler strata, long wx, long z, boolean rule, Columns.Stack out) {
        Structure st = wd.structure();
        if (st != null)
            st.deformedStack(wd, strata, (int) wx, (int) z, out, DEFORM.get(), !rule);
        else if (rule)
            Columns.stack(wd, strata, (int) wx, (int) z, out);
        else
            cellStack(strata, wx, z, out);
    }

    /** Per thread: the deformed column rule's working memory. */
    private static final ThreadLocal<Structure.Scratch> DEFORM = ThreadLocal.withInitial(Structure.Scratch::new);

    /** The stack of the strata cell nearest block (wx, z), as stored (no blend between cells). */
    static void cellStack(RasterSampler strata, long wx, long z, Columns.Stack out) {
        GridDesc g = strata.desc().grid();
        long i = (long) Math.floor((double) (wx - g.originBlockX()) / g.r() + 0.5);
        long j = (long) Math.floor((double) (z - g.originBlockZ()) / g.r() + 0.5);
        double scale = strata.desc().scale(), offset = strata.desc().offset();
        out.count = 0;
        int k = strata.stackDepth();
        for (int h = 0; h < k; h++) {
            int cls = strata.horizonClass(i, j, h);
            short top = strata.horizonTopRaw(i, j, h);
            if (cls == 255 || top == Short.MIN_VALUE)
                break;
            out.emit(top * scale + offset, cls, strata.horizonKeyTop(i, j, h));
        }
    }

    /** ⌈h⌉ − 1: the top block of terrain height h ({@link Water#topBlock}). */
    public static int topBlock(double h) {
        return Water.topBlock(h);
    }

    /** x folded into [−C/2, C/2). */
    public static long wrapX(WorldData wd, long x) {
        return Water.wrap(wd, x);
    }

    /**
     * The geometry of an n × n grid of columns {@code step} blocks wide whose first column's
     * minimum corner is block (x0, z0), with a ring of one more column around it (so every column
     * of the grid has its four neighbours): {@code out} has (n + 2)² shapes, column (i, j) of the
     * grid at index {@code (j + 1) * (n + 2) + (i + 1)}. x0 may be on any lap.
     *
     * <p>{@code step} 1 evaluates {@link Water.Region} (one gather of the river segments, the
     * terrain height at the noise-cell corners once each) and equals {@link Water#column} at every
     * column, the ring's included. A wider step samples each column at its centre.
     */
    public static void grid(WorldData wd, Water.Spec spec, long x0, long z0, int step, int n, Shape[] out) {
        int m = n + 2;
        IceCover ice = wd.iceCover().orElse(null);
        int minY = spec.minY(), maxY = spec.maxY();
        if (step == 1) {
            // One more ring for the region: a column's bank rule reads its four neighbours' cores.
            Water.Region reg = Water.region(wd, spec, x0 - 2, z0 - 2, n + 4, n + 4);
            for (int j = 0; j < m; j++) {
                for (int i = 0; i < m; i++) {
                    long x = x0 - 1 + i, z = z0 - 1 + j;
                    int cover = ice == null ? 0 : ice.blocks((int) wrapX(wd, x), (int) z);
                    shape(reg.column(i + 1, j + 1), cover, minY, maxY, out[j * m + i]);
                }
            }
            return;
        }
        // Each column asks its own river tile: gathering the segments of the whole area once, as the
        // one-block grid does, would hand every column of a wide area most of the planet's rivers.
        Rivers rivers = wd.rivers();
        int half = step / 2;
        for (int j = 0; j < m; j++) {
            for (int i = 0; i < m; i++) {
                long x = x0 + (long) (i - 1) * step + half, z = z0 + (long) (j - 1) * step + half;
                long wx = wrapX(wd, x);
                double surface = wd.height(wx, z);
                Rivers.Sample rs = rivers == null ? Rivers.Sample.NONE : rivers.sample(wx, z);
                int cover = ice == null ? 0 : ice.blocks((int) wx, (int) z);
                shape(Water.core(wd, spec, wx, z, surface, rs), cover, minY, maxY, out[j * m + i]);
            }
        }
    }

    /**
     * How far below its soil column (i, j) of a {@link #grid} shows its side: the drop from its
     * ground to its lowest neighbour's (water surfaces count as the neighbour's top: what is under
     * water beside a column is not seen from afar), 0 where no neighbour is lower.
     */
    public static int exposure(Shape[] grid, int n, int i, int j) {
        int m = n + 2, at = (j + 1) * m + (i + 1);
        int own = grid[at].ground;
        int lowest = own;
        for (int[] d : Water.NEIGHBOURS) {
            Shape s = grid[at + d[1] * m + d[0]];
            lowest = Math.min(lowest, s.water() ? Math.max(s.ground, s.waterTop) : s.ground);
        }
        return own - lowest;
    }
}
