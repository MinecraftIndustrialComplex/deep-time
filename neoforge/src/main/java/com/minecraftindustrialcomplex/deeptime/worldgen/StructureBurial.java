// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;

import java.util.Set;

/**
 * The rule that buries a structure placed at an absolute height (Ben's single-player world,
 * 2026-10-22: a trial chamber hanging in open sea water above a trench). No Minecraft types here, so a
 * unit test reads the whole rule; {@link StructureBurialPass} applies it to a structure start.
 *
 * <p><b>Why.</b> Vanilla gives several structures an absolute start height that assumes vanilla's ground,
 * which is always above y 0: trial chambers start at y -40 to -20 (and are about 57 blocks tall), ancient
 * cities at y -27, and mineshafts and strongholds are put below sea level, y 63 minus 10. Deep Time's ground
 * is the world's own: the sea floor's median is about y -56 (the median ocean depth is 4.2 km at 35 m a
 * block), and a trench reaches y -303. So over most of the ocean those structures stand in the water, and
 * the terrain a structure wraps itself in (trial chambers' {@code encapsulate}, the ancient city's
 * {@code beard_box}) hangs around it as a blob of deepslate in open water.
 *
 * <p><b>The rule.</b> Find the least ground height H over the start's footprint (the world data's own H, the one
 * the terrain is built from) and, if the start's top reaches {@code H - cover}, move every piece down so that
 * the top sits {@code cover} blocks under that least ground. A start that is already buried deeper stays
 * where it is. A start that would then reach below {@code min_y + 8} is dropped (the structure sets take
 * the next candidate or none). The structure is moved, not dropped, so its frequency does not fall: trial
 * chambers are progression content (trial keys, vaults, breeze rods).
 *
 * <p><b>Which structures.</b> Only those that place by an absolute height: see {@link #covers}.
 * Structures that follow the ground (buried treasure, trail ruins, ruins and wrecks) are never touched.
 *
 * <p>Ledger SCL-200 ({@link #COVER}, a game choice) and SCL-201 ({@link #covers}). {@code -Ddeeptime.buryStructures=false} turns the rule
 * off, for A/B checks.
 */
public final class StructureBurial {

    private StructureBurial() {}

    /** The ground height H(x, z) in blocks (the terrain's own, {@code TerrainSource.height}). */
    @FunctionalInterface
    public interface Height {
        double at(int x, int z);
    }

    /** The rock kept over a buried structure's top, blocks (ledger: SCL-200, game-choice: the coordinator, 2026-10-22). */
    public static final int COVER = 8;   // ledger: SCL-200

    /** A structure that would end lower than min_y plus this many blocks is dropped (SCL-200). */
    public static final int FLOOR_MARGIN = 8;   // ledger: SCL-200

    /** The ground is sampled on a grid of at most this many blocks over the footprint, plus its edges and corners (SCL-200). */
    public static final int GRID = 16;   // ledger: SCL-200

    /** {@link #plan} says: this start cannot be buried inside the world; drop it. */
    public static final int DROP = Integer.MIN_VALUE;

    /** The system property that switches the rule off ({@code -Ddeeptime.buryStructures=false}). */
    public static final String PROPERTY = "deeptime.buryStructures";

    /** On unless {@code -Ddeeptime.buryStructures=false}; read at each use, so a probe can flip it. */
    public static boolean enabled() {
        return !"false".equalsIgnoreCase(System.getProperty(PROPERTY, "true"));
    }

    private static final Set<String> UNDERGROUND_STEPS = Set.of("underground_structures", "underground_decoration", "strongholds");   // ledger: SCL-201

    /**
     * Whether the rule looks at a structure (ledger: SCL-201, game-choice): it places itself by an absolute height, below ground, whatever the
     * ground is. Decided on the structure's own JSON (the encoded {@code Structure}, so any mod's jigsaw kind
     * answers the same way, with no list to keep):
     * <ul>
     *   <li>{@code minecraft:stronghold}: always (vanilla sets its top anywhere from the world floor up to y 52 with
     *       {@code moveBelowSeaLevel}, in step {@code surface_structures});</li>
     *   <li>in an underground step ({@code underground_structures}, {@code underground_decoration},
     *       {@code strongholds}): a structure with a {@code start_height} (a jigsaw: trial chambers, ancient
     *       cities, Cataclysm's and Bosses of Mass Destruction's underground sites) and no
     *       {@code project_start_to_heightmap}, and the ordinary {@code minecraft:mineshaft};</li>
     *   <li>never: structures that follow the ground (buried treasure, which sits at y 90 in its box and is placed
     *       by the heightmap; trail ruins, projected to the surface; the mesa mineshaft, which is set between sea
     *       level and the ground), structures of a type this rule does not know (Ice and Fire's dragon caves
     *       find their own spot), and every structure above ground or in other steps.</li>
     * </ul>
     */
    public static boolean covers(JsonElement structure) {
        if (!structure.isJsonObject())
            return false;
        JsonObject s = structure.getAsJsonObject();
        String type = text(s, "type");
        if (type.equals("minecraft:stronghold"))
            return true;
        if (!UNDERGROUND_STEPS.contains(text(s, "step")) || s.has("project_start_to_heightmap"))
            return false;
        if (type.equals("minecraft:buried_treasure"))
            return false;
        if (type.equals("minecraft:mineshaft"))
            return text(s, "mineshaft_type").equals("normal");
        return s.has("start_height");
    }

    private static String text(JsonObject o, String key) {
        JsonElement e = o.get(key);
        return e != null && e.isJsonPrimitive() ? e.getAsString() : "";
    }

    /**
     * The shift the rule gives a start, in blocks.
     *
     * @param bottomY      the start's lowest block (its bounding box's minY)
     * @param topY         the start's highest block (maxY)
     * @param leastGround  the least ground height over its footprint, as a block level ({@link #leastGround})
     * @param worldMinY    the level's lowest block
     * @param cover        the rock to keep over the top
     * @return 0 when the start stays (its top is already at or under {@code leastGround - cover}); a negative
     *         number of blocks to move every piece by; {@link #DROP} when moving would put the bottom under
     *         {@code worldMinY + FLOOR_MARGIN}
     */
    public static int plan(int bottomY, int topY, int leastGround, int worldMinY, int cover) {
        long target = (long) leastGround - cover;   // the highest block the structure may reach
        if (topY <= target)
            return 0;
        long dy = target - topY;
        if (bottomY + dy < (long) worldMinY + FLOOR_MARGIN)
            return DROP;
        return (int) dy;
    }

    /** {@link #plan} with {@link #COVER}. */
    public static int plan(int bottomY, int topY, int leastGround, int worldMinY) {
        return plan(bottomY, topY, leastGround, worldMinY, COVER);
    }

    /**
     * The least ground height over the box {@code [minX, maxX] x [minZ, maxZ]}, as a block level (floor of the
     * least H): H sampled on a grid of at most {@code step} blocks, the box's edges and corners included. NaN
     * samples are skipped; {@code Integer.MAX_VALUE} when there is none (nothing to bury against).
     */
    public static int leastGround(Height height, int minX, int maxX, int minZ, int maxZ, int step) {
        int[] xs = axis(minX, maxX, step), zs = axis(minZ, maxZ, step);
        double least = Double.POSITIVE_INFINITY;
        for (int x : xs)
            for (int z : zs) {
                double h = height.at(x, z);
                if (!Double.isNaN(h) && h < least)
                    least = h;
            }
        return Double.isInfinite(least) ? Integer.MAX_VALUE : (int) Math.floor(least);
    }

    /**
     * The ground a start is buried against on a cube world: {@code height} on the faces, NaN off them, which
     * {@link #leastGround} skips. A chunk wholly off the faces places no structure piece ({@code CubeFill}, ledger
     * SCL-210), so the ground the void shows past a turning edge (the partner face's, folded) carries nothing of
     * the start; before 2026-10-25 a start by a seam was buried against it, deeper than its own ground asked or
     * dropped. {@code height} itself with no cube.
     */
    public static Height onFaces(Height height, CubeLayout cube) {
        return cube == null ? height : (x, z) -> cube.faceOfBlock(x, z) != null ? height.at(x, z) : Double.NaN;
    }

    /** {@link #leastGround} on {@link #GRID}. */
    public static int leastGround(Height height, int minX, int maxX, int minZ, int maxZ) {
        return leastGround(height, minX, maxX, minZ, maxZ, GRID);
    }

    /**
     * The sample coordinates along one axis: {@code lo}, then every {@code step} blocks (at most), and
     * {@code hi}. A span of {@code n} blocks is cut into {@code ceil(n / step)} equal parts, so both ends are
     * grid lines and no part is longer than {@code step}.
     */
    static int[] axis(int lo, int hi, int step) {
        if (hi <= lo)
            return new int[]{lo};
        int span = hi - lo;
        int parts = Math.max(1, Math.ceilDiv(span, Math.max(1, step)));
        int[] out = new int[parts + 1];
        for (int i = 0; i <= parts; i++)
            out[i] = lo + (int) Math.round((double) span * i / parts);
        return out;
    }
}
