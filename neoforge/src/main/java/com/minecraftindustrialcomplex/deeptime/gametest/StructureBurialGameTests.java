// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.StructureBurial;
import com.minecraftindustrialcomplex.deeptime.worldgen.StructureBurialPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.FluidTags;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.levelgen.structure.BuiltinStructures;
import net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece;
import net.minecraft.world.level.levelgen.structure.Structure;
import net.minecraft.world.level.levelgen.structure.StructurePiece;
import net.minecraft.world.level.levelgen.structure.StructureStart;
import net.minecraft.world.level.levelgen.structure.pools.JigsawJunction;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.TreeMap;

/**
 * Structures placed at an absolute height are buried under Deep Time's ground (SCL-200; Ben's single-player
 * world, 2026-10-22: a trial chamber's deepslate blob hanging in open sea water over a trench). They run on
 * {@code deeptime:test_planet_real}, the real baked test planet, and call {@code Structure.generate} as world
 * generation and {@code /place structure} do, so the mixin at its return is what is under test.
 *
 * <p>Each test makes a start twice from the same seed, once with the rule off for this thread
 * ({@link StructureBurialPass#withoutBurial}) and once as the game does, and compares them: with the same
 * random numbers the two are the same structure, so the difference between them is exactly what the rule did.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class StructureBurialGameTests {

    private StructureBurialGameTests() {}

    /** A 16-block grid can miss a dip between its samples; a piece may stand this much above {@code ground - cover} at a column. */
    private static final int SLACK = 3;

    /** A place on the planet: the least and the greatest ground height over the 257 x 257 columns around it. */
    private record Site(int x, int z, int least, int most) {}

    private static ServerLevel planet(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_REAL);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_REAL.location() + " does not exist");
            throw new IllegalStateException();
        }
        return level;
    }

    private static WorldDataTerrain terrain(GameTestHelper helper, ServerLevel level) {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_planet_real is not generating from world data");
            throw new IllegalStateException();
        }
        return t;
    }

    /**
     * The deepest place (least of the greatest heights around it: a whole footprint under the sea) or the highest
     * (greatest of the least heights: a whole footprint under high ground).
     */
    private static Site find(WorldDataTerrain t, boolean deep) {
        int c = t.circumference(), zp = (int) t.zHalf();
        Site best = null;
        for (int z = -zp + 192; z < zp - 192; z += 32)
            for (int x = -c / 2; x < c / 2; x += 32) {
                double least = Double.POSITIVE_INFINITY, most = Double.NEGATIVE_INFINITY;
                for (int dx = -128; dx <= 128; dx += 32)
                    for (int dz = -128; dz <= 128; dz += 32) {
                        double h = t.height(x + dx, z + dz);
                        least = Math.min(least, h);
                        most = Math.max(most, h);
                    }
                Site s = new Site(x, z, (int) Math.floor(least), (int) Math.floor(most));
                if (best == null || (deep ? s.most < best.most : s.least > best.least))
                    best = s;
            }
        return best;
    }

    private static StructureStart start(ServerLevel level, Structure structure, ChunkPos at) {
        var g = level.getChunkSource().getGenerator();
        return structure.generate(level.registryAccess(), g, g.getBiomeSource(), level.getChunkSource().randomState(),
                level.getStructureManager(), level.getSeed(), at, 0, level, biome -> true);
    }

    private static BoundingBox union(StructureStart s) {
        return BoundingBox.encapsulatingBoxes(s.getPieces().stream().map(StructurePiece::getBoundingBox).toList()).orElseThrow();
    }

    /**
     * What a move by {@code dy} must have done to the start {@code before}, as the pieces' boxes, the jigsaw
     * positions, the junctions' ground levels and the start's own box (the cached one, which a stale cache would
     * leave where it was). Returns the first difference or null.
     */
    private static String movedBy(StructureStart before, StructureStart after, int dy) {
        List<StructurePiece> a = before.getPieces(), b = after.getPieces();
        if (a.size() != b.size())
            return "piece count " + a.size() + " -> " + b.size();
        for (int i = 0; i < a.size(); i++) {
            BoundingBox x = a.get(i).getBoundingBox(), y = b.get(i).getBoundingBox();
            if (a.get(i).getClass() != b.get(i).getClass())
                return "piece " + i + " kind changed";
            if (y.minX() != x.minX() || y.maxX() != x.maxX() || y.minZ() != x.minZ() || y.maxZ() != x.maxZ()
                    || y.minY() != x.minY() + dy || y.maxY() != x.maxY() + dy)
                return "piece " + i + " box " + x + " -> " + y + " (expected dy " + dy + ")";
            if (a.get(i) instanceof PoolElementStructurePiece pa && b.get(i) instanceof PoolElementStructurePiece pb) {
                if (!pb.getPosition().equals(pa.getPosition().offset(0, dy, 0)))
                    return "piece " + i + " position " + pa.getPosition() + " -> " + pb.getPosition() + " (expected dy " + dy + ")";
                List<JigsawJunction> ja = pa.getJunctions(), jb = pb.getJunctions();
                if (ja.size() != jb.size())
                    return "piece " + i + " junction count " + ja.size() + " -> " + jb.size();
                for (int k = 0; k < ja.size(); k++)
                    if (jb.get(k).getSourceX() != ja.get(k).getSourceX() || jb.get(k).getSourceZ() != ja.get(k).getSourceZ()
                            || jb.get(k).getDeltaY() != ja.get(k).getDeltaY()
                            || jb.get(k).getSourceGroundY() != ja.get(k).getSourceGroundY() + dy)
                        return "piece " + i + " junction " + k + " ground y " + ja.get(k).getSourceGroundY() + " -> "
                                + jb.get(k).getSourceGroundY() + " (expected dy " + dy + ")";
            }
        }
        BoundingBox sa = before.getBoundingBox(), sb = after.getBoundingBox();
        if (sb.minY() != sa.minY() + dy || sb.maxY() != sa.maxY() + dy || sb.minX() != sa.minX() || sb.maxX() != sa.maxX()
                || sb.minZ() != sa.minZ() || sb.maxZ() != sa.maxZ())
            return "start box " + sa + " -> " + sb + " (expected dy " + dy + ")";
        return null;
    }

    /** The highest a piece may stand over each of its own columns, tested on an 8-block grid: the first piece that stands higher, or null. */
    private static String standsClear(WorldDataTerrain t, StructureStart s, int cover, int[] worst) {
        String first = null;
        for (StructurePiece p : s.getPieces()) {
            BoundingBox b = p.getBoundingBox();
            for (int x : grid(b.minX(), b.maxX()))
                for (int z : grid(b.minZ(), b.maxZ())) {
                    int over = b.maxY() - ((int) Math.floor(t.height(x, z)) - cover);
                    if (over > worst[0]) {
                        worst[0] = over;
                        if (over > SLACK && first == null)
                            first = p.getClass().getSimpleName() + " " + b + " stands " + over + " blocks above ground - " + cover
                                    + " at (" + x + ", " + z + ") where H = " + t.height(x, z);
                    }
                }
        }
        return first;
    }

    private static int[] grid(int lo, int hi) {
        int n = Math.max(1, Math.ceilDiv(hi - lo, 8));
        int[] out = new int[n + 1];
        for (int i = 0; i <= n; i++)
            out[i] = lo + (int) Math.round((double) (hi - lo) * i / n);
        return out;
    }

    private static Optional<Structure> structure(ServerLevel level, ResourceKey<Structure> key) {
        return level.registryAccess().registryOrThrow(Registries.STRUCTURE).getHolder(key).map(h -> h.value());
    }

    /**
     * The reported case. Over the deepest sea of the real test planet, each structure that places by an absolute
     * height is made as the game makes it and as vanilla would, and the rule is read back:
     * <ul>
     *   <li>the premise: the vanilla start's top reaches within {@code cover} of the least ground under it (it would
     *       float or sit in the sea);</li>
     *   <li>the move: the buried start is the vanilla one shifted by exactly the rule's {@code dy} (boxes,
     *       jigsaw positions, junction ground levels, the start's box), its top {@code cover} blocks under the
     *       least ground over its footprint, and every piece stands clear of its own columns' ground;</li>
     *   <li>a start the rule cannot bury inside the world is dropped;</li>
     *   <li>the structures that follow the ground (buried treasure, the mesa mineshaft) are not moved.</li>
     * </ul>
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 600)
    public static void absoluteStructuresAreBuriedUnderTheSea(GameTestHelper helper) {
        ServerLevel level = planet(helper);
        WorldDataTerrain t = terrain(helper, level);
        Site site = find(t, true);
        helper.assertTrue(site.most < 0, "the deepest sea of the planet (greatest H around it " + site.most
                + ") lies more than 63 blocks under the sea: the premise of the test");
        ChunkPos at = new ChunkPos(site.x >> 4, site.z >> 4);
        int minY = level.getMinBuildHeight();
        StringBuilder log = new StringBuilder();
        int checked = 0;
        long movedBefore = StructureBurialPass.moved();

        record Case(ResourceKey<Structure> key, boolean covered) {}
        for (Case c : List.of(new Case(BuiltinStructures.TRIAL_CHAMBERS, true), new Case(BuiltinStructures.ANCIENT_CITY, true),
                new Case(BuiltinStructures.STRONGHOLD, true), new Case(BuiltinStructures.MINESHAFT, true),
                new Case(BuiltinStructures.MINESHAFT_MESA, false), new Case(BuiltinStructures.BURIED_TREASURE, false))) {
            Optional<Structure> opt = structure(level, c.key());
            if (opt.isEmpty()) {
                log.append(' ').append(c.key().location()).append(": not registered;");
                continue;
            }
            Structure s = opt.get();
            String id = c.key().location().toString();
            helper.assertTrue(StructureBurialPass.looksAt(s, level.registryAccess()) == c.covered(),
                    id + (c.covered() ? " must be covered by the rule" : " must be left alone by the rule"));
            StructureStart vanilla = StructureBurialPass.withoutBurial(() -> start(level, s, at));
            StructureStart game = start(level, s, at);
            if (!vanilla.isValid()) {
                log.append(' ').append(id).append(": no start here;");
                continue;
            }
            BoundingBox vb = union(vanilla);
            var gen = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
            int ground = StructureBurial.leastGround(StructureBurial.onFaces(t::height, gen.cubeLayout()), vb.minX(), vb.maxX(),
                    vb.minZ(), vb.maxZ());
            int dy = c.covered() ? StructureBurial.plan(vb.minY(), vb.maxY(), ground, minY) : 0;
            log.append(' ').append(id).append(": vanilla y ").append(vb.minY()).append("..").append(vb.maxY())
                    .append(" least ground ").append(ground).append(" dy ").append(dy == StructureBurial.DROP ? "DROP" : dy).append(';');
            if (dy == StructureBurial.DROP) {
                helper.assertTrue(!game.isValid(), id + ": a start that cannot be buried inside the world is dropped");
                checked++;
                continue;
            }
            helper.assertTrue(game.isValid(), id + ": the game's start is missing (vanilla's is there)");
            String bad = movedBy(vanilla, game, dy);
            helper.assertTrue(bad == null, id + " is not the vanilla start moved by " + dy + ": " + bad);
            if (!c.covered()) {
                helper.assertTrue(dy == 0, id + " follows the ground and must not move");
            } else if (dy != 0) {
                BoundingBox gb = union(game);
                helper.assertTrue(gb.maxY() == ground - StructureBurial.COVER, id + ": the top of the buried start is at y " + gb.maxY()
                        + ", expected " + (ground - StructureBurial.COVER) + " (least ground " + ground + " - cover)");
                int[] worst = {Integer.MIN_VALUE};
                String high = standsClear(t, game, StructureBurial.COVER, worst);
                helper.assertTrue(high == null, id + ": " + high);
                log.append(" top ").append(gb.maxY()).append(" worst piece over ground-cover ").append(worst[0]).append(';');
            }
            if (c.key() == BuiltinStructures.TRIAL_CHAMBERS)
                helper.assertTrue(dy < 0 && vb.maxY() > ground - StructureBurial.COVER,
                        "the premise: a vanilla trial chamber (top y " + vb.maxY() + ") reaches the sea floor or the water over it (ground " + ground + ")");
            checked++;
        }
        helper.assertTrue(StructureBurialPass.moved() > movedBefore, "the mixin moved no start");
        helper.assertTrue(StructureBurialPass.mismatched() == 0, "a piece did not move with its structure (see the log)");
        helper.assertTrue(checked >= 4, "only " + checked + " structures were checked:" + log);

        // The switch: with -Ddeeptime.buryStructures=false the game's start is vanilla's (this thread only reads it within the call).
        Structure chambers = structure(level, BuiltinStructures.TRIAL_CHAMBERS).orElseThrow();
        String old = System.getProperty(StructureBurial.PROPERTY);
        StructureStart off;
        try {
            System.setProperty(StructureBurial.PROPERTY, "false");
            off = start(level, chambers, at);
        } finally {
            if (old == null)
                System.clearProperty(StructureBurial.PROPERTY);
            else
                System.setProperty(StructureBurial.PROPERTY, old);
        }
        StructureStart vanilla = StructureBurialPass.withoutBurial(() -> start(level, chambers, at));
        helper.assertTrue(movedBy(vanilla, off, 0) == null, "-Ddeeptime.buryStructures=false must leave the vanilla start");
        DeepTime.LOGGER.info("[gametest] absoluteStructuresAreBuriedUnderTheSea site=({}, {}) ground {}..{}:{} {}",
                site.x, site.z, site.least, site.most, log, StructureBurialPass.stats());
        helper.succeed();
    }

    /**
     * Ground that already covers a start leaves it where it is. Under the highest ground of the planet each
     * structure is made as the game makes it and as vanilla would, and the two are the same start (no move, none
     * dropped). Nothing was moved by the rule here.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 600)
    public static void buriedStructuresStayWhereTheyAre(GameTestHelper helper) {
        ServerLevel level = planet(helper);
        WorldDataTerrain t = terrain(helper, level);
        Site site = find(t, false);
        helper.assertTrue(site.least > 17 + StructureBurial.COVER + 10,
                "the highest ground of the planet (least H around it " + site.least + ") covers a trial chamber (top y 17): the premise of the test");
        ChunkPos at = new ChunkPos(site.x >> 4, site.z >> 4);
        long movedBefore = StructureBurialPass.moved(), droppedBefore = StructureBurialPass.dropped();
        StringBuilder log = new StringBuilder();
        int checked = 0;
        for (var key : List.of(BuiltinStructures.TRIAL_CHAMBERS, BuiltinStructures.ANCIENT_CITY, BuiltinStructures.MINESHAFT)) {
            Structure s = structure(level, key).orElse(null);
            if (s == null)
                continue;
            StructureStart vanilla = StructureBurialPass.withoutBurial(() -> start(level, s, at));
            StructureStart game = start(level, s, at);
            helper.assertTrue(vanilla.isValid() && game.isValid(), key.location() + ": no start on high ground");
            BoundingBox vb = union(vanilla);
            helper.assertTrue(vb.maxY() <= site.least - StructureBurial.COVER,
                    key.location() + ": the premise: the vanilla start (top y " + vb.maxY() + ") is under the least ground " + site.least);
            String bad = movedBy(vanilla, game, 0);
            helper.assertTrue(bad == null, key.location() + " was changed under high ground: " + bad);
            log.append(' ').append(key.location()).append(" y ").append(vb.minY()).append("..").append(vb.maxY()).append(';');
            checked++;
        }
        helper.assertTrue(checked >= 2, "only " + checked + " structures were checked");
        helper.assertTrue(StructureBurialPass.moved() == movedBefore && StructureBurialPass.dropped() == droppedBefore,
                "the rule moved or dropped a start that was already under the ground");
        DeepTime.LOGGER.info("[gametest] buriedStructuresStayWhereTheyAre site=({}, {}) ground {}..{}:{}", site.x, site.z, site.least, site.most, log);
        helper.succeed();
    }

    private static boolean structureBlock(BlockState s) {
        String p = BuiltInRegistries.BLOCK.getKey(s.getBlock()).getPath();
        return p.contains("tuff_brick") || p.contains("polished_tuff") || p.contains("chiseled_tuff") || p.contains("copper")
                || p.equals("trial_spawner") || p.equals("vault") || p.equals("iron_bars") || p.equals("crafter");
    }

    /** A piece's template ({@code minecraft:trial_chambers/corridor/end_2}), or its class for a piece that is not a pool element. */
    private static String pieceName(StructurePiece p) {
        if (!(p instanceof PoolElementStructurePiece pe))
            return p.getClass().getSimpleName();
        String e = pe.getElement().toString();   // Single[Left[minecraft:trial_chambers/corridor/end_2]]
        return e.substring(e.lastIndexOf('[') + 1).replace("]", "");
    }

    /**
     * What one placed start holds, read back from the generated chunks inside its pieces' boxes. Every cell is read
     * before the start is placed and again after. Before, a wet cell is the sea's (it lies over its column's real
     * floor: the open sea, or a lake) or a cave's (under that floor: a carver's tunnel, which a level without
     * aquifers fills with water below sea level). After, a wet cell is the template's own (the placing changed it)
     * or the world's left in place (the cell is as it was: the placing did not write it, or wrote the same water over
     * it, as {@code corridor/end_2}'s basin over a flooded tunnel does under MIC).
     */
    private static final class Tally {
        long read, structure, structureWet, seaBefore, caveBefore, water, left, columns;
        int caveLow = Integer.MAX_VALUE, caveHigh = Integer.MIN_VALUE, caveUnder = Integer.MAX_VALUE, leftLow = Integer.MAX_VALUE, leftHigh = Integer.MIN_VALUE;
        /** Least over the sampled columns of (the real floor's top solid y) - (the start's top y): the cover that stands over it. */
        int cover = Integer.MAX_VALUE;
        String firstSea, firstLeft;
        /** The waterlogged structure blocks by the piece that holds them: template, {count, least y, greatest y}. */
        final Map<String, int[]> wetPieces = new TreeMap<>();

        String line() {
            StringBuilder w = new StringBuilder();
            wetPieces.forEach((k, v) -> w.append(w.isEmpty() ? "" : ", ").append(k).append(": ").append(v[0])
                    .append(" at y ").append(v[1]).append("..").append(v[2]));
            return "read " + read + " structure " + structure + " waterlogged " + structureWet + (structureWet == 0 ? "" : " (" + w + ")")
                    + "; water before placing: the sea's " + seaBefore + (firstSea == null ? "" : " (first " + firstSea + ")")
                    + ", a cave's " + caveBefore + (caveBefore == 0 ? "" : " (y " + caveLow + ".." + caveHigh + ", " + caveUnder + " or more under the floor)")
                    + "; water after: " + water + " (the template's " + (water - left) + ", the world's left in place " + left
                    + (left == 0 ? "" : " at y " + leftLow + ".." + leftHigh + ", first " + firstLeft) + ")"
                    + "; cover " + (columns == 0 ? "n/a" : cover) + " over " + columns + " columns";
        }
    }

    /**
     * The cells of {@code start}'s pieces' boxes in {@code chunk} (and {@code writable}) as they stand now, before it
     * is placed; counts their water into {@code t}, the sea's (at or over the column's real floor, its top solid
     * block in the {@code OCEAN_FLOOR_WG} heightmap, which water does not stop) apart from a cave's (under it).
     */
    private static Map<Long, BlockState> before(LevelChunk chunk, StructureStart start, BoundingBox writable, Tally t) {
        Map<Long, BlockState> cells = new HashMap<>();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        for (StructurePiece p : start.getPieces()) {
            BoundingBox b = p.getBoundingBox();
            if (!b.intersects(writable))
                continue;
            for (int x = Math.max(b.minX(), writable.minX()); x <= Math.min(b.maxX(), writable.maxX()); x++)
                for (int z = Math.max(b.minZ(), writable.minZ()); z <= Math.min(b.maxZ(), writable.maxZ()); z++) {
                    int floor = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z);
                    for (int y = Math.max(b.minY(), writable.minY()); y <= Math.min(b.maxY(), writable.maxY()); y++) {
                        BlockState s = chunk.getBlockState(pos.set(x, y, z));
                        if (cells.putIfAbsent(pos.asLong(), s) != null || !s.getFluidState().is(FluidTags.WATER))
                            continue;
                        if (y >= floor) {
                            t.seaBefore++;
                            if (t.firstSea == null)
                                t.firstSea = pos.toShortString() + " over the floor at " + floor + " in " + pieceName(p);
                        } else {
                            t.caveBefore++;
                            t.caveLow = Math.min(t.caveLow, y);
                            t.caveHigh = Math.max(t.caveHigh, y);
                            t.caveUnder = Math.min(t.caveUnder, floor - y);
                        }
                    }
                }
        }
        return cells;
    }

    /** Reads the cells {@link #before} took, after {@code start} was placed, into {@code t} (each cell once, under the first piece whose box holds it). */
    private static void blocks(LevelChunk chunk, StructureStart start, BoundingBox writable, Map<Long, BlockState> before, Tally t) {
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        for (StructurePiece p : start.getPieces()) {
            BoundingBox b = p.getBoundingBox();
            if (!b.intersects(writable))
                continue;
            for (int x = Math.max(b.minX(), writable.minX()); x <= Math.min(b.maxX(), writable.maxX()); x++)
                for (int z = Math.max(b.minZ(), writable.minZ()); z <= Math.min(b.maxZ(), writable.maxZ()); z++)
                    for (int y = Math.max(b.minY(), writable.minY()); y <= Math.min(b.maxY(), writable.maxY()); y++) {
                        BlockState was = before.remove(pos.set(x, y, z).asLong());
                        if (was == null)
                            continue;
                        BlockState s = chunk.getBlockState(pos);
                        t.read++;
                        boolean wet = s.getFluidState().is(FluidTags.WATER);
                        if (structureBlock(s)) {
                            t.structure++;
                            if (wet) {
                                t.structureWet++;
                                int[] w = t.wetPieces.computeIfAbsent(pieceName(p), k -> new int[]{0, Integer.MAX_VALUE, Integer.MIN_VALUE});
                                w[0]++;
                                w[1] = Math.min(w[1], y);
                                w[2] = Math.max(w[2], y);
                            }
                        }
                        if (!wet)
                            continue;
                        t.water++;
                        if (s == was) {
                            t.left++;
                            t.leftLow = Math.min(t.leftLow, y);
                            t.leftHigh = Math.max(t.leftHigh, y);
                            if (t.firstLeft == null)
                                t.firstLeft = pos.toShortString() + " " + BuiltInRegistries.BLOCK.getKey(s.getBlock()) + " in " + pieceName(p);
                        }
                    }
        }
    }

    /** The real floor over the footprint, on an 8-block grid of this chunk's columns that lie under a piece (read before anything is placed). */
    private static void floor(LevelChunk chunk, StructureStart start, BoundingBox writable, Tally t) {
        int top = union(start).maxY();
        for (int x = writable.minX(); x <= writable.maxX(); x += 8)
            for (int z = writable.minZ(); z <= writable.maxZ(); z += 8) {
                boolean under = false;
                for (StructurePiece p : start.getPieces()) {
                    BoundingBox b = p.getBoundingBox();
                    if (x >= b.minX() && x <= b.maxX() && z >= b.minZ() && z <= b.maxZ()) {
                        under = true;
                        break;
                    }
                }
                if (!under)
                    continue;
                t.columns++;
                t.cover = Math.min(t.cover, chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z) - top);
            }
    }

    /**
     * Placed as world generation places it, a buried trial chamber stands under the floor and not in the sea. The
     * start is made over the deepest sea twice (buried as the game makes it; at vanilla's own height as a control),
     * both are placed chunk by chunk into the generated level ({@code placeInChunk}, as
     * {@code SeamGameTests.structureStraddlesSeam} does), and every cell inside their pieces' boxes is read before
     * and after:
     * <ul>
     *   <li>the structure's own blocks (tuff bricks, copper, spawners) are there;</li>
     *   <li>no cell the buried chamber takes held the sea's water before it was placed (water at or over its column's
     *       real floor), and that floor (the generated chunks', not the model's H) stands at least {@code cover} above
     *       its top over every sampled column of its footprint;</li>
     *   <li>the control, at vanilla's height, sticks out of that floor and its cells held the sea: the picture of the
     *       report, and the proof that the read sees the sea.</li>
     * </ul>
     * The other water is logged, not asserted, because none of it is the sea's or the rule's:
     * <ul>
     *   <li>a cave's, before the placing: {@code deeptime:planet_nocaves} turns aquifers off, so the biomes' carvers
     *       fill every tunnel they cut under sea level with water, also under the sea floor (52 cells at y -169..-147
     *       in the buried chamber's boxes at the deepest sea of master 049c4278's planet, 34 or more blocks
     *       under the floor; the template wrote over all of them);</li>
     *   <li>the template's own, after it: vanilla's trial chamber templates carry basins ({@code corridor/end_2}: 60
     *       water and 23 waterlogged waxed oxidized cut copper stairs; {@code intersection/intersection_1}: 966
     *       water; {@code corridor/atrium_1}: 17; one each in {@code corridor/addon/display_1}, {@code display_2}
     *       and {@code corridor/entrance_1}; read from the 1.21.1 client jar's templates).</li>
     * </ul>
     * Waterlogged blocks do not tell a chamber in the sea from one in rock: trial chambers place with
     * {@code LiquidSettings.IGNORE_WATERLOGGING}, so the world's water never waterlogs their blocks, and the count
     * only says whether {@code corridor/end_2} fell in the sampled chunks. Until 2026-10-23 this test asserted that
     * count was 0, and it failed once master bf8477b2's planet moved the deepest sea to (-128, -512): 23 in the
     * buried chamber and the same 23 in the control, all {@code corridor/end_2}'s stairs. The chamber is placed over
     * finished chunks here, without the terrain adaptation ({@code encapsulate}) that generation applies, which only
     * adds rock round its pieces.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void buriedChamberStandsUnderTheFloor(GameTestHelper helper) {
        ServerLevel level = planet(helper);
        WorldDataTerrain t = terrain(helper, level);
        Site site = find(t, true);
        Structure chambers = structure(level, BuiltinStructures.TRIAL_CHAMBERS).orElse(null);
        helper.assertTrue(chambers != null, "no minecraft:trial_chambers");
        ChunkPos at = new ChunkPos(site.x >> 4, site.z >> 4);
        StructureStart start = start(level, chambers, at);
        StructureStart control = StructureBurialPass.withoutBurial(() -> start(level, chambers, at));
        helper.assertTrue(start.isValid() && control.isValid(), "no trial chamber start at the deepest sea");
        BoundingBox box = union(start), controlBox = union(control);
        helper.assertTrue(box.maxY() <= site.most - StructureBurial.COVER + SLACK,
                "the start (top y " + box.maxY() + ") is under the ground around it (H up to " + site.most + ")");

        // Every chunk a piece touches, at most 12 of them spread over the start.
        Set<ChunkPos> touched = new LinkedHashSet<>();
        for (StructurePiece p : start.getPieces()) {
            BoundingBox b = p.getBoundingBox();
            for (int cx = b.minX() >> 4; cx <= b.maxX() >> 4; cx++)
                for (int cz = b.minZ() >> 4; cz <= b.maxZ() >> 4; cz++)
                    touched.add(new ChunkPos(cx, cz));
        }
        List<ChunkPos> all = new ArrayList<>(touched), chunks = new ArrayList<>();
        int stride = Math.max(1, all.size() / 12);
        for (int i = 0; i < all.size() && chunks.size() < 12; i += stride)
            chunks.add(all.get(i));

        Tally buried = new Tally(), vanilla = new Tally();
        ChunkSampling.forEach(helper, chunks.size(), i -> {
            ChunkPos cp = chunks.get(i);
            LevelChunk chunk = (LevelChunk) level.getChunkSource().getChunk(cp.x, cp.z, ChunkStatus.FULL, true);
            BoundingBox writable = new BoundingBox(cp.getMinBlockX(), level.getMinBuildHeight() + 1, cp.getMinBlockZ(),
                    cp.getMaxBlockX(), level.getMaxBuildHeight() - 1, cp.getMaxBlockZ());
            floor(chunk, start, writable, buried);
            floor(chunk, control, writable, vanilla);
            for (StructureStart st : List.of(start, control)) {
                Tally into = st == start ? buried : vanilla;
                Map<Long, BlockState> cells = before(chunk, st, writable, into);
                st.placeInChunk(level, level.structureManager(), level.getChunkSource().getGenerator(),
                        RandomSource.create(cp.toLong()), writable, cp);
                blocks(chunk, st, writable, cells, into);
            }
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] buriedChamberStandsUnderTheFloor site=({}, {}) ground {}..{} chunks {}/{}\n  buried (box y {}..{}): {}\n  vanilla control (box y {}..{}): {}",
                    site.x, site.z, site.least, site.most, chunks.size(), all.size(), box.minY(), box.maxY(), buried.line(),
                    controlBox.minY(), controlBox.maxY(), vanilla.line());
            helper.assertTrue(buried.structure > 0, "none of the structure's own blocks (tuff bricks, copper, spawners) was found in the "
                    + buried.read + " blocks read: the chamber was not placed");
            helper.assertTrue(buried.seaBefore == 0, buried.seaBefore + " of the cells the buried chamber takes held the sea's water (at or over "
                    + "their column's real floor) before it was placed: " + buried.line());
            helper.assertTrue(buried.columns > 0 && buried.cover >= StructureBurial.COVER - SLACK,
                    "the real floor stands only " + buried.cover + " blocks over the buried chamber's top (cover " + StructureBurial.COVER + ")");
            helper.assertTrue(vanilla.columns > 0 && vanilla.cover < 0 && vanilla.seaBefore > 0,
                    "the control (vanilla height) does not stick out of the floor into the sea: " + vanilla.line());
            helper.succeed();
        });
    }
}
