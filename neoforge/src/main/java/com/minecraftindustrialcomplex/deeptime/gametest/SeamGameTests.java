// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.toroidalworld.api.v1.ToroidalShape;
import com.toroidalworld.api.v1.ToroidalWorldApi;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.NbtUtils;
import net.minecraft.nbt.Tag;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.LeavesBlock;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.levelgen.feature.ConfiguredFeature;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.levelgen.structure.BuiltinStructures;
import net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece;
import net.minecraft.world.level.levelgen.structure.Structure;
import net.minecraft.world.level.levelgen.structure.StructureStart;
import net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer;
import net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement;
import net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool;
import net.minecraft.world.level.levelgen.structure.templatesystem.LiquidSettings;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * The seam in a real Toroidal World-shaped Deep Time overworld (P0-06). Planet namespace: these
 * run only on the dev planet server (./gametest-planet.sh), whose overworld is the planet; the
 * GameTest server's flat overworld has no seam. The harness positions them next to x = C/2.
 */
@GameTestHolder(GameTestSupport.NS_PLANET)
@PrefixGameTestTemplate(false)
public final class SeamGameTests {

    private SeamGameTests() {}

    static int circumference(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)) {
            helper.fail("the test level is not a Deep Time planet: " + level.getChunkSource().getGenerator().getClass().getName());
            throw new IllegalStateException();
        }
        return g.dataRef().circumferenceBlocks();
    }

    static ToroidalShape shape(GameTestHelper helper) {
        ToroidalShape shape = ToroidalWorldApi.shapeOf(helper.getLevel()).orElse(null);
        if (shape == null) {
            helper.fail("the level has no Toroidal World shape");
            throw new IllegalStateException();
        }
        return shape;
    }

    /**
     * The chunk holding {@code pos} after folding, only if it is already loaded (the harness
     * forceloads the seam strip). Never forces a load: a synchronous load of a seam chunk can wait
     * in TW's seam-generation queue (P0-06 finding).
     */
    static LevelChunk loadedChunk(ServerLevel level, ToroidalShape shape, BlockPos pos) {
        BlockPos f = shape.fold(pos);
        return level.getChunkSource().getChunkNow(f.getX() >> 4, f.getZ() >> 4);
    }

    /**
     * Across the seam the world continues. Positions past the edge are folded through TW's API
     * first (TW's rule: fold a position before using it; a raw read at chunk C/32 can stall in TW's
     * seam-generation queue): x = C/2 + k folds to −C/2 + k, and the step from the last canonical
     * column (C/2 − 1) to the first (−C/2) is no larger than the terrain's own steps beside it.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void columnsContinuous(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        var shape = com.toroidalworld.api.v1.ToroidalWorldApi.shapeOf(level).orElse(null);
        if (shape == null) {
            helper.fail("the level has no Toroidal World shape");
            return;
        }
        int badFolds = 0, jumps = 0, rows = 0, unloadedRows = 0;
        StringBuilder detail = new StringBuilder();
        // Rows inside the strip the harness forceloads around the seam (z ± 32).
        for (int i = 0; i < 16; i++) {
            int z = (i - 8) * 4 + 1;
            for (int k = 0; k < 4; k++) {
                var folded = shape.fold(new net.minecraft.core.BlockPos(c / 2 + k, 64, z));
                if (folded.getX() != -c / 2 + k || folded.getZ() != z) {
                    badFolds++;
                    detail.append(" fold(").append(c / 2 + k).append(")=").append(folded.getX());
                }
            }
            // Only chunks the harness's forceload already holds: a synchronous load of a seam chunk
            // can wait in TW's seam-generation queue (P0-06 finding), so the test never forces one.
            var east = level.getChunkSource().getChunkNow((c / 2 - 1) >> 4, z >> 4);
            var west = level.getChunkSource().getChunkNow((-c / 2) >> 4, z >> 4);
            if (east == null || west == null) {
                unloadedRows++;
                continue;
            }
            rows++;
            int a = east.getHeight(Heightmap.Types.OCEAN_FLOOR, c / 2 - 2, z);
            int b = east.getHeight(Heightmap.Types.OCEAN_FLOOR, c / 2 - 1, z);
            int across = west.getHeight(Heightmap.Types.OCEAN_FLOOR, -c / 2, z);
            int after = west.getHeight(Heightmap.Types.OCEAN_FLOOR, -c / 2 + 1, z);
            if (Math.abs(across - b) > Math.max(Math.max(Math.abs(b - a), Math.abs(after - across)), 2)) {
                jumps++;
                detail.append(" jump(z=").append(z).append("):").append(b).append("->").append(across);
            }
        }
        DeepTime.LOGGER.info("[gametest] columnsContinuous C={} rows={} unloadedRows={} badFolds={} seamJumps={}{}", c, rows,
                unloadedRows, badFolds, jumps, detail);
        helper.assertTrue(rows >= 8, "only " + rows + " seam rows loaded");
        helper.assertTrue(badFolds == 0 && jumps == 0, "seam discontinuities:" + detail);
        helper.succeed();
    }

    /**
     * An entity that moves itself across the seam is still one entity, now inside the canonical
     * span. TW folds a non-player entity at the end of its own server tick, so the entity must move
     * under its own motion (a gravity-free snowball flying east), not be pushed from outside.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void entityCrossesSeam(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        var ball = net.minecraft.world.entity.EntityType.SNOWBALL.create(level);
        if (ball == null) {
            helper.fail("could not create a snowball");
            return;
        }
        double y = 300;   // clear air, far above any terrain near the seam
        ball.setNoGravity(true);
        ball.moveTo(c / 2.0 - 3.5, y, 0.5, 0, 0);
        ball.setDeltaMovement(0.25, 0, 0);   // 5 blocks a second eastward, across x = C/2
        level.addFreshEntity(ball);
        var id = ball.getUUID();
        helper.runAfterDelay(40, () -> {
            var found = level.getEntity(id);
            double x = found == null ? Double.NaN : found.getX();
            boolean inSpan = found != null && x >= -c / 2.0 && x < c / 2.0;
            boolean crossed = found != null && x < 0;   // it now flies on from the west edge
            DeepTime.LOGGER.info("[gametest] entityCrossesSeam C={} found={} x={} inCanonicalSpan={} crossed={}", c,
                    found != null, x, inSpan, crossed);
            if (found != null)
                found.discard();
            helper.assertTrue(found != null, "the entity vanished at the seam");
            helper.assertTrue(inSpan && crossed, "entity at x=" + x + ": expected just east of " + (-c / 2) + " after crossing");
            helper.succeed();
        });
    }

    /** Templates tried in order for {@link #structureStraddlesSeam}: vanilla village houses. */
    private static final String[] HOUSES = {
            "village/plains/houses/plains_small_house_1",
            "village/plains/houses/plains_small_house_2",
            "village/plains/houses/plains_medium_house_1",
    };

    /**
     * A structure that straddles the seam is whole on both sides. The test builds a one-piece
     * structure start (a vanilla village house, a template, so every block is known) whose piece
     * runs from a few blocks west of x = C/2 to a few blocks past it, and places it chunk by chunk
     * as world generation does: {@code placeInChunk} for every canonical chunk the piece touches,
     * each with its own writable box. The chunks at x = −C/2 do not intersect the piece's raw
     * bounding box, so only Toroidal World's {@code StructureStartMixin} (which moves the start
     * into the chunk's frame) can put the west half there. Then every template block, folded, is
     * compared with the level.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void structureStraddlesSeam(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        ToroidalShape shape = shape(helper);
        var templates = level.getStructureManager();
        String id = null;
        StructureTemplate template = null;
        for (String h : HOUSES) {
            Optional<StructureTemplate> t = templates.get(ResourceLocation.withDefaultNamespace(h));
            if (t.isPresent() && t.get().getSize().getX() >= 5) {
                id = h;
                template = t.get();
                break;
            }
        }
        if (template == null) {
            helper.fail("no vanilla village house template found");
            return;
        }
        BlockPos spot = helper.absolutePos(GameTestSupport.CENTRE);
        int sizeX = template.getSize().getX();
        // (sizeX + 1) / 2 columns end at C/2 - 1; the rest lie past the edge.
        BlockPos origin = new BlockPos(c / 2 - (sizeX + 1) / 2, spot.getY() + 40, spot.getZ());
        var element = StructurePoolElement.single("minecraft:" + id).apply(StructureTemplatePool.Projection.RIGID);
        BoundingBox box = element.getBoundingBox(templates, origin, Rotation.NONE);
        var piece = new PoolElementStructurePiece(templates, element, origin, 0, Rotation.NONE, box,
                LiquidSettings.IGNORE_WATERLOGGING);
        Structure structure = level.registryAccess().registryOrThrow(Registries.STRUCTURE).getOrThrow(BuiltinStructures.VILLAGE_PLAINS);
        var start = new StructureStart(structure, new ChunkPos(origin), 0, new PiecesContainer(List.of(piece)));

        // The canonical chunks the piece touches.
        List<ChunkPos> chunks = new ArrayList<>();
        for (int cx = box.minX() >> 4; cx <= box.maxX() >> 4; cx++)
            for (int cz = box.minZ() >> 4; cz <= box.maxZ() >> 4; cz++)
                chunks.add(new ChunkPos(shape.foldChunk(Direction.Axis.X, cx), cz));
        int westChunks = 0;
        for (ChunkPos cp : chunks) {
            if (level.getChunkSource().getChunkNow(cp.x, cp.z) == null) {
                helper.fail("chunk " + cp + " is not loaded (the harness forceloads the seam strip)");
                return;
            }
            if (cp.x < 0)
                westChunks++;
            BoundingBox writable = new BoundingBox(cp.getMinBlockX(), level.getMinBuildHeight() + 1, cp.getMinBlockZ(),
                    cp.getMaxBlockX(), level.getMaxBuildHeight() - 1, cp.getMaxBlockZ());
            start.placeInChunk(level, level.structureManager(), level.getChunkSource().getGenerator(),
                    RandomSource.create(cp.toLong()), writable, cp);
        }

        // Every template block, folded, against the level (block identity: shape states may update).
        CompoundTag nbt = template.save(new CompoundTag());
        ListTag paletteTag = nbt.contains("palettes", Tag.TAG_LIST)
                ? nbt.getList("palettes", Tag.TAG_LIST).getList(0) : nbt.getList("palette", Tag.TAG_COMPOUND);
        var blockLookup = level.holderLookup(Registries.BLOCK);
        List<BlockState> palette = new ArrayList<>();
        for (Tag t : paletteTag)
            palette.add(NbtUtils.readBlockState(blockLookup, (CompoundTag) t));
        int eastOk = 0, eastBad = 0, westOk = 0, westBad = 0, eastSolid = 0, westSolid = 0;
        StringBuilder detail = new StringBuilder();
        for (Tag t : nbt.getList("blocks", Tag.TAG_COMPOUND)) {
            CompoundTag b = (CompoundTag) t;
            ListTag p = b.getList("pos", Tag.TAG_INT);
            BlockPos raw = origin.offset(p.getInt(0), p.getInt(1), p.getInt(2));
            BlockState expected = palette.get(b.getInt("state"));
            if (expected.is(Blocks.STRUCTURE_BLOCK) || expected.is(Blocks.JIGSAW) || expected.is(Blocks.STRUCTURE_VOID))
                continue;
            LevelChunk chunk = loadedChunk(level, shape, raw);
            BlockPos folded = shape.fold(raw);
            boolean west = folded.getX() != raw.getX();
            boolean ok = chunk != null && chunk.getBlockState(folded).getBlock() == expected.getBlock();
            if (ok) {
                if (west) westOk++; else eastOk++;
                if (!expected.isAir()) {
                    if (west) westSolid++; else eastSolid++;
                }
            } else {
                if (west) westBad++; else eastBad++;
                if (detail.length() < 400)
                    detail.append(' ').append(folded.toShortString()).append(':')
                            .append(BuiltInRegistries.BLOCK.getKey(expected.getBlock())).append("!=")
                            .append(chunk == null ? "unloaded" : BuiltInRegistries.BLOCK.getKey(chunk.getBlockState(folded).getBlock()));
            }
        }
        DeepTime.LOGGER.info("[gametest] structureStraddlesSeam C={} template=minecraft:{} box={} chunks={} westChunks={} "
                        + "east ok={} bad={} solid={} west ok={} bad={} solid={}{}", c, id, box, chunks, westChunks,
                eastOk, eastBad, eastSolid, westOk, westBad, westSolid, detail);
        helper.assertTrue(westChunks > 0 && eastSolid > 0 && westSolid > 0, "the piece does not straddle the seam");
        helper.assertTrue(eastBad == 0 && westBad == 0, "blocks missing or wrong across the seam:" + detail);
        helper.succeed();
    }

    /**
     * A tree that straddles the seam is whole: vanilla's oak feature with its trunk in the last
     * canonical column (x = C/2 − 1) puts leaves one and two blocks past the edge, which land at
     * x = −C/2 and −C/2 + 1. Each west leaf is then ticked, which recomputes its distance from
     * its neighbours (across the seam, back to the trunk); no leaf may be left decaying.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void treeStraddlesSeam(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        ToroidalShape shape = shape(helper);
        BlockPos spot = helper.absolutePos(GameTestSupport.CENTRE);
        BlockPos trunk = new BlockPos(c / 2 - 1, spot.getY() + 60, spot.getZ());
        var oak = level.registryAccess().registryOrThrow(Registries.CONFIGURED_FEATURE)
                .getHolder(ResourceKey.create(Registries.CONFIGURED_FEATURE, ResourceLocation.withDefaultNamespace("oak")));
        if (oak.isEmpty()) {
            helper.fail("no minecraft:oak configured feature");
            return;
        }
        ConfiguredFeature<?, ?> feature = oak.get().value();
        boolean placed = feature.place(level, level.getChunkSource().getGenerator(), RandomSource.create(42L), trunk);
        int logs = 0, eastLeaves = 0, westLeaves = 0, decaying = 0, unloaded = 0;
        for (int dx = -3; dx <= 3; dx++)
            for (int dy = 0; dy <= 9; dy++)
                for (int dz = -3; dz <= 3; dz++) {
                    BlockPos raw = trunk.offset(dx, dy, dz);
                    LevelChunk chunk = loadedChunk(level, shape, raw);
                    if (chunk == null) {
                        unloaded++;
                        continue;
                    }
                    BlockPos folded = shape.fold(raw);
                    BlockState state = chunk.getBlockState(folded);
                    if (dx == 0 && dz == 0 && state.is(Blocks.OAK_LOG))
                        logs++;
                    if (!state.is(Blocks.OAK_LEAVES))
                        continue;
                    boolean west = folded.getX() != raw.getX();
                    if (west) {
                        westLeaves++;
                        // Recompute the distance from the neighbours, which are across the seam.
                        state.tick(level, folded, level.random);
                        state = chunk.getBlockState(folded);
                    } else {
                        eastLeaves++;
                    }
                    if (!state.getValue(LeavesBlock.PERSISTENT) && state.getValue(LeavesBlock.DISTANCE) >= LeavesBlock.DECAY_DISTANCE)
                        decaying++;
                }
        DeepTime.LOGGER.info("[gametest] treeStraddlesSeam C={} placed={} trunk={} logs={} eastLeaves={} westLeaves={} decaying={} unloaded={}",
                c, placed, trunk.toShortString(), logs, eastLeaves, westLeaves, decaying, unloaded);
        helper.assertTrue(placed && logs >= 4, "the oak was not placed (placed=" + placed + ", logs=" + logs + ")");
        helper.assertTrue(unloaded == 0, unloaded + " tree blocks in unloaded chunks");
        helper.assertTrue(eastLeaves > 0 && westLeaves > 0, "leaves on one side only: east " + eastLeaves + ", west " + westLeaves);
        helper.assertTrue(decaying == 0, decaying + " leaves would decay (distance 7) after crossing the seam");
        helper.succeed();
    }

    /**
     * Informational (plan P0-06): does a Create kinetic network work across the seam? A creative
     * motor at x = C/2 − 4 facing east drives a shaft line through x = C/2; after two seconds each
     * shaft's speed is read from its block entity. Logs the speeds and whether the rotation reached
     * the west side, and succeeds whatever it finds (it is also not required). Create is not a
     * compile dependency: blocks come from the registry, speeds from the saved data ({@code Speed}).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200, required = false)
    public static void shaftLineAcrossSeam(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        ToroidalShape shape = shape(helper);
        if (!ModList.get().isLoaded("create")) {
            DeepTime.LOGGER.info("[gametest] shaftLineAcrossSeam skipped: Create is not loaded");
            helper.succeed();
            return;
        }
        Block motor = BuiltInRegistries.BLOCK.get(ResourceLocation.fromNamespaceAndPath("create", "creative_motor"));
        Block shaft = BuiltInRegistries.BLOCK.get(ResourceLocation.fromNamespaceAndPath("create", "shaft"));
        if (motor == Blocks.AIR || shaft == Blocks.AIR) {
            DeepTime.LOGGER.info("[gametest] shaftLineAcrossSeam skipped: create:creative_motor or create:shaft missing");
            helper.succeed();
            return;
        }
        BlockPos spot = helper.absolutePos(GameTestSupport.CENTRE);
        int y = spot.getY() + 20, z = spot.getZ();
        BlockState motorState = motor.defaultBlockState();
        if (motorState.hasProperty(BlockStateProperties.FACING))
            motorState = motorState.setValue(BlockStateProperties.FACING, Direction.EAST);
        BlockState shaftState = shaft.defaultBlockState();
        if (shaftState.hasProperty(BlockStateProperties.AXIS))
            shaftState = shaftState.setValue(BlockStateProperties.AXIS, Direction.Axis.X);
        // Raw positions: the level folds those past the edge (TW's LevelMixin).
        level.setBlock(new BlockPos(c / 2 - 4, y, z), motorState, 3);
        List<BlockPos> line = new ArrayList<>();
        for (int x = c / 2 - 3; x <= c / 2 + 3; x++) {
            BlockPos raw = new BlockPos(x, y, z);
            level.setBlock(raw, shaftState, 3);
            line.add(raw);
        }
        helper.runAfterDelay(40, () -> {
            StringBuilder speeds = new StringBuilder();
            int eastTurning = 0, westTurning = 0, westShafts = 0;
            for (BlockPos raw : line) {
                BlockPos folded = shape.fold(raw);
                boolean west = folded.getX() != raw.getX();
                LevelChunk chunk = loadedChunk(level, shape, raw);
                var be = chunk == null ? null : chunk.getBlockEntity(folded);
                float speed = be == null ? Float.NaN : be.saveWithoutMetadata(level.registryAccess()).getFloat("Speed");
                boolean isShaft = chunk != null && chunk.getBlockState(folded).is(shaft);
                speeds.append(' ').append(folded.getX()).append('=').append(isShaft ? String.valueOf(speed) : "missing");
                if (west)
                    westShafts++;
                if (speed != 0 && !Float.isNaN(speed)) {
                    if (west) westTurning++; else eastTurning++;
                }
            }
            DeepTime.LOGGER.info("[gametest] shaftLineAcrossSeam C={} rotationPropagates={} eastTurning={} westTurning={}/{} speeds:{}",
                    c, westShafts > 0 && westTurning == westShafts, eastTurning, westTurning, westShafts, speeds);
            helper.succeed();
        });
    }
}
