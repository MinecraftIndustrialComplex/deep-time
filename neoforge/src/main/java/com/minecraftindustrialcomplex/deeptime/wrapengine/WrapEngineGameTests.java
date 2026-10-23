// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamTurns;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestAssertException;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.MoverType;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.LightLayer;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.RedStoneWireBlock;
import net.minecraft.world.level.block.RedstoneLampBlock;
import net.minecraft.world.level.block.RepeaterBlock;
import net.minecraft.world.level.block.RotatedPillarBlock;
import net.minecraft.world.level.block.SlabBlock;
import net.minecraft.world.level.block.StairBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Half;
import net.minecraft.world.level.block.state.properties.SlabType;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;
import net.minecraft.world.phys.Vec3;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.function.BooleanSupplier;

/**
 * The wrap engine's Phase 1 proof (docs/design/wrap-engine.md §10): across a half-turn seam and across
 * a quarter-turn corner, building, fluids, redstone, collisions, light and entities work; the band
 * stays a faithful turned copy after edits on either side, through the band and straight into chunk
 * storage; and a band chunk that loads (with its source unloaded, then after edits while it was
 * unloaded) is copied from its source.
 *
 * <p>The sites are bounded seams in the GameTest server's flat overworld, far from the test grid, all
 * glued by one topology installed once ({@link #TOPOLOGY}). Each test works in a site's local frame:
 * (u, v) with the edge along u, the stored side at v ≥ 0 and the band at v &lt; 0, so a build that
 * runs straight across the seam is written in plain coordinates and lands, through the fold, on the
 * far side, turned. Expectations come from the topology's own motions, never from pinned coordinates:
 * the same test body runs for both kinds of seam.
 *
 * <p>Namespace {@code deeptime_wrapengine}, enabled only by {@code ./gametest.sh -PwrapEngine}, which
 * also sets {@code -Ddeeptime.wrapengine=true}; never part of the normal suite.
 */
@GameTestHolder(DeepTime.MODID)
@PrefixGameTestTemplate(false)
public final class WrapEngineGameTests {

    static final String NS = "deeptime_wrapengine";
    static final String TEMPLATE = "empty_5x5x5";
    static final int Y0 = 100;
    static final int DEPTH = 16;

    private WrapEngineGameTests() {}

    // --- Sites ----------------------------------------------------------------------------------

    enum Kind { HALF, QUARTER }

    /**
     * A test site: origin (ox, oz) on the edge, u along the edge in [uMin, uMax), its own topology
     * (for expectations; the level has the union), the box of blocks it uses and, on the cube, the
     * boxes {x0, x1, z0, z1} where the seam's far side is (elsewhere on the net).
     */
    record Site(String name, Kind kind, int ox, int oz, int uMin, int uMax, WrapTopology local, boolean glued,
                int boxX0, int boxX1, int boxZ0, int boxZ1, List<int[]> far) {

        Site(String name, Kind kind, int ox, int oz, int uMin, int uMax, WrapTopology local, boolean glued,
             int boxX0, int boxX1, int boxZ0, int boxZ1) {
            this(name, kind, ox, oz, uMin, uMax, local, glued, boxX0, boxX1, boxZ0, boxZ1, List.of());
        }

        /** The site's own box, then its far boxes, each {x0, x1, z0, z1}. */
        List<int[]> boxes() {
            List<int[]> out = new ArrayList<>();
            out.add(new int[]{boxX0, boxX1, boxZ0, boxZ1});
            out.addAll(far);
            return out;
        }

        boolean inBoxes(double x, double z) {
            for (int[] b : boxes())
                if (x >= b[0] && x < b[1] && z >= b[2] && z < b[3])
                    return true;
            return false;
        }

        BlockPos at(int u, int dy, int v) {
            return new BlockPos(ox + u, Y0 + dy, oz + v);
        }

        /** The stored position of {@code p} under this site's gluing (itself when not in a band). */
        BlockPos stored(BlockPos p) {
            WrapTopology.Fold f = local.fold(p.getX(), p.getZ());
            return f == null ? p : new BlockPos(f.x(), p.getY(), f.z());
        }

        /** The motion band → stored at {@code p} (identity when not in a band). */
        CellIsometry iso(BlockPos p) {
            WrapTopology.Fold f = local.fold(p.getX(), p.getZ());
            return f == null ? CellIsometry.IDENTITY : f.iso();
        }

        List<ChunkPos> bandChunks() {
            List<ChunkPos> out = new ArrayList<>();
            local.forEachBandChunk(k -> out.add(new ChunkPos(k)));
            return out;
        }
    }

    static Site half(String name, int cx, int z0, int halfSpan) {
        return new Site(name, Kind.HALF, cx, z0, -halfSpan, halfSpan, WrapLayouts.halfTurnSeam(cx, z0, halfSpan, DEPTH), true,
                cx - halfSpan - 16, cx + halfSpan + 16, z0 - DEPTH, z0 + 16);
    }

    static Site quarter(String name, int x0, int z0, int span) {
        return new Site(name, Kind.QUARTER, x0, z0, 0, span, WrapLayouts.quarterTurnCorner(x0, z0, span, DEPTH), true,
                x0 - DEPTH, x0 + span + 16, z0 - DEPTH, z0 + span + 16);
    }

    static Site site(Kind k, int index) {
        return k == Kind.HALF ? half("half-" + index, 20480 + 512 * index, 20480, 32) : quarter("quarter-" + index, 20480 + 512 * index, 22528, 48);
    }

    static final int BUILD = 0, WATER = 1, REDSTONE = 2, LIGHT = 3, CONSISTENCY = 4, PLAYER = 5, BLOCK_ENTITIES = 6;
    static final Site LONG_HALF = half("long-half", 30720, 20480, 256);
    static final Site LONG_QUARTER = quarter("long-quarter", 32768, 22528, 256);
    /** The water test's control: the same build with no gluing (its {@code local} only names the would-be far side). */
    static final Site CONTROL;

    static {
        Site c = half("control", 20480 + 512 * 7, 24576, 32);
        CONTROL = new Site(c.name(), c.kind(), c.ox(), c.oz(), c.uMin(), c.uMax(), c.local(), false, c.boxX0(), c.boxX1(), c.boxZ0(), c.boxZ1());
    }

    // The cube (design §12): one whole net, far east of the other sites, with the engine's own slides
    // (the GameTest server has no Toroidal World), D = 16. Site coordinates are relative to its centre
    // (CUBE_X0, CUBE_Z0) in h = s/2: F0 [−h, h)², F1 east of it, F2 at [3h, 5h) cut at x = 2s, F3 west,
    // N above F0 and S below. Each site's far boxes hold the blocks its band shows and the band that
    // shows its own blocks; no two sites share a block column.

    static final int CUBE_S = 256, CUBE_H = CUBE_S / 2, CUBE_X0 = 49152, CUBE_Z0 = 20480;
    static final WrapTopology CUBE = WrapLayouts.cubeCross(CUBE_S, DEPTH, true).translated(CUBE_X0, CUBE_Z0);

    /** A site on the cube: origin and far boxes {x0, x1, z0, z1} relative to the cube's centre. */
    static Site cubeSite(String name, Kind kind, int ox, int oz, int uMin, int uMax, int[]... far) {
        List<int[]> f = new ArrayList<>();
        for (int[] b : far)
            f.add(new int[]{CUBE_X0 + b[0], CUBE_X0 + b[1], CUBE_Z0 + b[2], CUBE_Z0 + b[3]});
        int x = CUBE_X0 + ox, z = CUBE_Z0 + oz;
        return new Site(name, kind, x, z, uMin, uMax, CUBE, true, x + uMin - 16, x + uMax + 16, z - DEPTH, z + 16, List.copyOf(f));
    }

    /** F1-top, 32 to 80 blocks from the corner (h, −h): the band shows N's east strip, a quarter turn. */
    static final Site CUBE_QUARTER = cubeSite("cube-quarter", Kind.QUARTER, CUBE_H + 32, -CUBE_H, 0, 48,
            new int[]{CUBE_H - 32, CUBE_H + 16, -CUBE_H - 96, -CUBE_H - 16});
    /** F1-top, 144 to 192 blocks from the corner. */
    static final Site CUBE_QUARTER_EDITS = cubeSite("cube-quarter-edits", Kind.QUARTER, CUBE_H + 144, -CUBE_H, 0, 48,
            new int[]{CUBE_H - 16, CUBE_H + 16, -CUBE_H - 208, -CUBE_H - 128});
    /** F2-top west of Toroidal World's seam: the band shows N's top strip, a half turn about (2h, −2h). */
    static final Site CUBE_HALF = cubeSite("cube-half", Kind.HALF, 3 * CUBE_H + 56, -CUBE_H, -32, 32,
            new int[]{16, CUBE_H, -3 * CUBE_H - 16, -3 * CUBE_H + 32});
    /**
     * N-top west of x = 0, the piece cut at Toroidal World's seam: the band shows F2's east half, which
     * lives at [−2s, −3s/2), a half turn with the period composed in.
     */
    static final Site CUBE_HALF_TW = cubeSite("cube-half-tw", Kind.HALF, -64, -3 * CUBE_H, -32, 32,
            new int[]{-2 * CUBE_S, -2 * CUBE_S + 112, -CUBE_H - 16, -CUBE_H + 16});
    /**
     * The slide at x = 2s next to F2's bottom-east corner (the turn kind names what follows the slide
     * there): F2's east half at [−2s, −3s/2) and S's bottom strip by x = 0.
     */
    static final Site CUBE_SLIDE = cubeSite("cube-slide", Kind.HALF, 2 * CUBE_S, CUBE_H, 0, 0,
            new int[]{-2 * CUBE_S, -2 * CUBE_S + 16, CUBE_H - 16, CUBE_H + 16},
            new int[]{-16, 16, 3 * CUBE_H - 16, 3 * CUBE_H + 16});
    /**
     * The concave cube corner (h, h) where F0, F1 and S meet (its missing quadrant is two overlapping
     * bands), and the convex one at S's outer corner (−h, 3h), which is F3's bottom-west corner
     * (−3h, h) too.
     */
    static final Site CUBE_CORNER = cubeSite("cube-corner", Kind.QUARTER, CUBE_H, CUBE_H, 0, 0,
            new int[]{-CUBE_H - 16, -CUBE_H + 16, 3 * CUBE_H - 16, 3 * CUBE_H + 16},
            new int[]{-3 * CUBE_H - 16, -3 * CUBE_H + 16, CUBE_H - 16, CUBE_H + 16});

    /** Every glued site's edges, installed once on the overworld. */
    static final WrapTopology TOPOLOGY;

    static {
        List<WrapTopology> parts = new ArrayList<>();
        for (Kind k : Kind.values())
            for (int i = BUILD; i <= BLOCK_ENTITIES; i++) // BLOCK_ENTITIES: WrapBlockEntityGameTests (design §12.8)
                parts.add(site(k, i).local());
        parts.add(LONG_HALF.local());
        parts.add(LONG_QUARTER.local());
        parts.addAll(CreateSeamGameTests.topologies()); // the Create adapters' sites (design §11), no Create types
        parts.add(CUBE); // the cube's sites (design §12) share one whole net
        TOPOLOGY = WrapTopology.union("wrap-engine GameTest sites", parts);
    }

    static synchronized void ensureInstalled(ServerLevel level) {
        if (WrapEngine.topology(level) != TOPOLOGY)
            WrapEngine.install(level, TOPOLOGY);
    }

    /** Installs the sites' topology, forceloads the site's boxes, clears them (stone floor at Y0 − 1, air above). */
    static Site open(GameTestHelper helper, Site s) {
        ServerLevel level = helper.getLevel();
        if (s.glued())
            ensureInstalled(level);
        for (int[] b : s.boxes())
            for (int x = b[0] >> 4; x <= (b[1] - 1) >> 4; x++)
                for (int z = b[2] >> 4; z <= (b[3] - 1) >> 4; z++) {
                    level.setChunkForced(x, z, true);
                    level.getChunk(x, z);
                }
        // Clear the stored blocks only; the band follows by replication.
        for (int[] b : s.boxes())
            for (int x = b[0]; x < b[1]; x++)
                for (int z = b[2]; z < b[3]; z++) {
                    if (s.glued() && s.local().fold(x, z) != null)
                        continue;
                    for (int y = Y0 - 1; y <= Y0 + 5; y++)
                        level.setBlock(new BlockPos(x, y, z), y == Y0 - 1 ? Blocks.STONE.defaultBlockState() : Blocks.AIR.defaultBlockState(),
                                Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE);
                }
        log("%s opened at (%d, %d), %s", s.name(), s.ox(), s.oz(), s.glued() ? "glued: " + WrapEngine.stats(level) : "NOT glued (control)");
        return s;
    }

    static void log(String fmt, Object... args) {
        DeepTime.LOGGER.info("[gametest] wrapengine " + String.format(fmt, args));
    }

    static void check(boolean ok, String fmt, Object... args) {
        String msg = String.format(fmt, args);
        log("%s %s", ok ? "OK  " : "FAIL", msg);
        if (!ok)
            throw new GameTestAssertException(msg);
    }

    /** Runs {@code then} on the first tick {@code cond} holds, failing after {@code maxTicks}. */
    static void waitUntil(GameTestHelper helper, int maxTicks, String what, BooleanSupplier cond, Runnable then) {
        if (cond.getAsBoolean()) {
            then.run();
            return;
        }
        if (maxTicks <= 0)
            throw new GameTestAssertException("timed out waiting for " + what);
        helper.runAfterDelay(1, () -> waitUntil(helper, maxTicks - 1, what, cond, then));
    }

    static void whenSynced(GameTestHelper helper, Site s, Runnable then) {
        ServerLevel level = helper.getLevel();
        waitUntil(helper, 200, s.name() + "'s band chunks synced",
                () -> s.bandChunks().stream().allMatch(c -> level.getChunkSource().getChunkNow(c.x, c.z) == null || WrapEngine.isBandSynced(level, c.x, c.z)),
                then);
    }

    /**
     * Every band block of a site at heights Y0 − 1 .. Y0 + 5 holds its stored block turned into the band's
     * frame (what collisions and the client read), and the folded read agrees. Returns the count checked.
     */
    static int audit(ServerLevel level, Site s, String when) {
        int checked = 0, wrong = 0;
        String first = null;
        for (int[] b : s.boxes())
            for (int x = b[0]; x < b[1]; x++)
                for (int z = b[2]; z < b[3]; z++) {
                    WrapTopology.Fold f = s.local().fold(x, z);
                    if (f == null || level.getChunkSource().getChunkNow(x >> 4, z >> 4) == null)
                        continue;
                    for (int y = Y0 - 1; y <= Y0 + 5; y++) {
                        BlockPos q = new BlockPos(x, y, z);
                        BlockState want = level.getBlockState(new BlockPos(f.x(), y, f.z())).rotate(WrapEngine.rotation(f.iso().inverse()));
                        BlockState raw = WrapEngine.rawState(level, q);
                        BlockState read = level.getBlockState(q);
                        checked++;
                        if (raw != want || read != want) {
                            wrong++;
                            if (first == null)
                                first = String.format("(%d, %d, %d): raw %s, read %s, stored turned %s", x, y, z, raw, read, want);
                        }
                    }
                }
        check(wrong == 0, "%s band consistent %s: %d of %d band blocks wrong%s", s.name(), when, wrong, checked, first == null ? "" : "; first " + first);
        return checked;
    }

    // --- Building, collisions, entities ---------------------------------------------------------

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void buildingAcrossHalfTurn(GameTestHelper helper) {
        building(helper, site(Kind.HALF, BUILD));
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void buildingAcrossQuarterTurn(GameTestHelper helper) {
        building(helper, site(Kind.QUARTER, BUILD));
    }

    /**
     * A block placed beyond the seam lands on the stored block it is, turned; the band shows it; it is
     * solid on both sides; an armour stand at the seam is stopped by it from this side; one where the far
     * side is open is carried over, heading turned; a thrown item crosses and keeps going, turned.
     */
    static void building(GameTestHelper helper, Site s0) {
        Site s = open(helper, s0);
        ServerLevel level = helper.getLevel();
        int u1 = 6, u2 = 14, u3 = s.kind() == Kind.HALF ? 22 : 30;
        whenSynced(helper, s, () -> {
            BlockState north = Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, Direction.NORTH);
            BlockPos band = s.at(u1, 0, -1);
            level.setBlock(band, north, Block.UPDATE_ALL);
            BlockPos stored = s.stored(band);
            Direction storedFacing = WrapEngine.turn(s.iso(band), Direction.NORTH);
            BlockState atStored = WrapEngine.rawState(level, stored);
            check(atStored.is(Blocks.OAK_STAIRS) && atStored.getValue(StairBlock.FACING) == storedFacing,
                    "%s: placed at band %s facing north -> stored at %s facing %s: %s", s.name(), band.toShortString(), stored.toShortString(),
                    storedFacing, atStored);
            check(!stored.closerThan(band, 3), "%s: the stored block %s is on the far side, not next to %s", s.name(), stored.toShortString(),
                    band.toShortString());
            BlockState viewed = level.getBlockState(band);
            check(viewed.equals(north), "%s: read back through the seam: %s", s.name(), viewed);
            BlockState copy = WrapEngine.rawState(level, band);
            check(copy.equals(north), "%s: band copy in chunk storage (what the client and collisions read): %s", s.name(), copy);
            check(!atStored.getCollisionShape(level, stored).isEmpty() && !copy.getCollisionShape(level, band).isEmpty(), "%s: solid on both sides",
                    s.name());

            // A block stored on this side near the seam shows in the band beyond the far side, turned.
            BlockState eastLog = Blocks.OAK_LOG.defaultBlockState().setValue(RotatedPillarBlock.AXIS, Direction.Axis.X);
            BlockPos inside = s.at(u2 + 4, 0, 0);
            level.setBlock(inside, eastLog, Block.UPDATE_ALL);
            for (int[] q : s.local().images(inside.getX(), inside.getZ())) {
                BlockPos img = new BlockPos(q[0], inside.getY(), q[1]);
                BlockState is = WrapEngine.rawState(level, img);
                Direction.Axis axis = WrapEngine.turn(s.iso(img).inverse(), Direction.EAST).getAxis();
                check(is.is(Blocks.OAK_LOG) && is.getValue(RotatedPillarBlock.AXIS) == axis, "%s: log stored along x at %s shows at %s along %s: %s",
                        s.name(), inside.toShortString(), img.toShortString(), axis, is);
            }

            // Collisions: the far side's stairs stop an entity at the seam on this side.
            ArmorStand blocked = stand(level, s.ox() + u1 + 0.5, s.oz() + 0.6);
            blocked.move(MoverType.SELF, new Vec3(0, 0, -1));
            check(Math.abs(blocked.getZ() - (s.oz() + 0.25)) < 1e-3, "%s: armour stand (half-width 0.25) moving 1 north stops at z = oz + %.4f",
                    s.name(), blocked.getZ() - s.oz());
            // Where the far side is open, it walks over and is carried across, turned.
            ArmorStand walker = stand(level, s.ox() + u2 + 0.5, s.oz() + 0.6);
            float yaw0 = walker.getYRot();
            walker.move(MoverType.SELF, new Vec3(0, 0, -1));
            check(walker.getZ() < s.oz(), "%s: open column: the stand moved past the seam line (z = oz %.3f)", s.name(), walker.getZ() - s.oz());
            WrapTopology.PointFold expect = s.local().foldPoint(walker.getX(), walker.getZ());
            // A thrown item, no gravity, heading north at 0.2 blocks a tick.
            ItemEntity item = new ItemEntity(level, s.ox() + u3 + 0.5, Y0 + 0.5, s.oz() + 1.5, new ItemStack(Items.DIAMOND));
            item.setNoGravity(true);
            item.setPickUpDelay(32767);
            item.setDeltaMovement(0, 0, -0.2);
            level.addFreshEntity(item);
            WrapTopology.PointFold itemLine = s.local().foldPoint(item.getX(), s.oz() - 0.5);
            double[] itemDir = itemLine.iso().applyVector(0, -1);

            helper.runAfterDelay(20, () -> {
                check(Math.abs(walker.getX() - expect.x()) < 1e-3 && Math.abs(walker.getZ() - expect.z()) < 1e-3,
                        "%s: armour stand carried to (%.3f, %.3f), the topology's fold (%.3f, %.3f)", s.name(), walker.getX(), walker.getZ(),
                        expect.x(), expect.z());
                float turn = ((walker.getYRot() - yaw0) % 360f + 360f) % 360f;
                check(Math.abs(turn - expect.iso().yawDegrees()) < 1e-3, "%s: heading turned by %.1f degrees (motion turns %.1f)", s.name(), turn,
                        expect.iso().yawDegrees());
                check(s.local().inside(Mth.floor(walker.getX()), Mth.floor(walker.getZ())), "%s: the stand is on a stored block now", s.name());
                blocked.discard();
                walker.discard();
            });
            helper.runAfterDelay(120, () -> {
                Vec3 v = item.getDeltaMovement();
                double speed = Math.hypot(v.x, v.z);
                double along = (v.x * itemDir[0] + v.z * itemDir[1]) / Math.max(speed, 1e-9);
                double off = (item.getX() - itemLine.x()) * itemDir[1] - (item.getZ() - itemLine.z()) * itemDir[0];
                check(s.local().inside(Mth.floor(item.getX()), Mth.floor(item.getZ())) && speed > 1e-3 && along > 0.999 && Math.abs(off) < 1e-3,
                        "%s: item crossed: at (%.3f, %.3f) moving (%.3f, %.3f), the turned heading (%.0f, %.0f), %.4f off its line", s.name(),
                        item.getX(), item.getZ(), v.x, v.z, itemDir[0], itemDir[1], off);
                item.discard();
                log("%s: %s", s.name(), WrapEngine.stats(level));
                helper.succeed();
            });
        });
    }

    private static ArmorStand stand(ServerLevel level, double x, double z) {
        ArmorStand a = EntityType.ARMOR_STAND.create(level);
        a.setNoGravity(true);
        a.moveTo(x, Y0, z, 180f, 0f);
        level.addFreshEntity(a);
        return a;
    }

    // --- Fluids ---------------------------------------------------------------------------------

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void waterAcrossHalfTurn(GameTestHelper helper) {
        water(helper, site(Kind.HALF, WATER), CONTROL);
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void waterAcrossQuarterTurn(GameTestHelper helper) {
        water(helper, site(Kind.QUARTER, WATER), null);
    }

    /**
     * A one-wide channel runs straight across the seam (built through the band); water from a source one
     * block inside flows across and on down the far side level by level; the band copy agrees; with no
     * gluing (the control) the far side stays dry.
     */
    static void water(GameTestHelper helper, Site s0, Site control0) {
        Site s = open(helper, s0);
        Site control = control0 == null ? null : open(helper, control0);
        int a = s.kind() == Kind.HALF ? 10 : 20;
        ServerLevel level = helper.getLevel();
        whenSynced(helper, s, () -> {
            for (Site t : control == null ? List.of(s) : List.of(s, control)) {
                BlockState wall = Blocks.STONE_BRICKS.defaultBlockState();
                for (int dy = 0; dy <= 1; dy++) {
                    for (int v = -8; v <= 2; v++) {
                        set(level, t, a - 1, dy, v, wall);
                        set(level, t, a + 1, dy, v, wall);
                    }
                    set(level, t, a, dy, -8, wall);
                    set(level, t, a, dy, 2, wall);
                }
                set(level, t, a, 0, 1, Blocks.WATER.defaultBlockState());
            }
            helper.runAfterDelay(160, () -> {
                int[] amounts = new int[7];
                StringBuilder sb = new StringBuilder();
                for (int k = 0; k < 7; k++) {
                    amounts[k] = level.getFluidState(s.at(a, 0, -1 - k)).getAmount();
                    sb.append(amounts[k]).append(k < 6 ? "," : "");
                }
                log("%s: water amounts across the seam (v = -1 .. -7, read through the fold): [%s]; near v = 1: %d, v = 0: %d", s.name(), sb,
                        level.getFluidState(s.at(a, 0, 1)).getAmount(), level.getFluidState(s.at(a, 0, 0)).getAmount());
                check(amounts[0] > 0, "%s: water crossed the seam (amount %d)", s.name(), amounts[0]);
                for (int k = 1; k < 6; k++)
                    check(amounts[k] > 0 && amounts[k] < amounts[k - 1], "%s: v = %d holds %d < %d", s.name(), -1 - k, amounts[k], amounts[k - 1]);
                check(amounts[6] == 0, "%s: the run ends where a 7-block run ends (v = -7 holds %d)", s.name(), amounts[6]);
                // On the far side, in its own frame, the flow points away from the seam.
                BlockPos far = s.stored(s.at(a, 0, -3));
                Vec3 flow = level.getFluidState(far).getFlow(level, far);
                double[] away = s.iso(s.at(a, 0, -3)).applyVector(0, -1);
                check(flow.x * away[0] + flow.z * away[1] > 0, "%s: far side %s flows (%.2f, %.2f), away from the seam (%.0f, %.0f)", s.name(),
                        far.toShortString(), flow.x, flow.z, away[0], away[1]);
                FluidState copy = WrapEngine.rawState(level, s.at(a, 0, -1)).getFluidState();
                check(copy.is(Fluids.FLOWING_WATER) && copy.getAmount() == amounts[0], "%s: the band copy at v = -1 is water of amount %d (far side %d)",
                        s.name(), copy.getAmount(), amounts[0]);
                if (control != null) {
                    int dry = 0;
                    for (int k = 0; k < 7; k++)
                        dry += level.getFluidState(control.stored(control.at(a, 0, -1 - k))).isEmpty() ? 1 : 0;
                    check(dry == 7, "control site without gluing: the would-be far side is dry in %d of 7 blocks", dry);
                }
                audit(level, s, "after the water ran");
                helper.succeed();
            });
        });
    }

    private static void set(ServerLevel level, Site s, int u, int dy, int v, BlockState state) {
        level.setBlock(s.at(u, dy, v), state, Block.UPDATE_ALL);
    }

    // --- Redstone -------------------------------------------------------------------------------

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void redstoneAcrossHalfTurn(GameTestHelper helper) {
        redstone(helper, site(Kind.HALF, REDSTONE));
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void redstoneAcrossQuarterTurn(GameTestHelper helper) {
        redstone(helper, site(Kind.QUARTER, REDSTONE));
    }

    /**
     * A wire line runs across the seam and lights a lamp on the far side, its power falling one per block
     * across the seam; a repeater whose output faces the seam powers the far side too, and its band copy
     * is turned and powered; removing the power turns both off.
     */
    static void redstone(GameTestHelper helper, Site s0) {
        Site s = open(helper, s0);
        ServerLevel level = helper.getLevel();
        int b = s.kind() == Kind.HALF ? 4 : 12, c = s.kind() == Kind.HALF ? 14 : 28;
        whenSynced(helper, s, () -> {
            BlockState lamp = Blocks.REDSTONE_LAMP.defaultBlockState();
            // Line 1: power at v = 3, wire v = 2 .. -2, lamp v = -3.
            set(level, s, b, 0, -3, lamp);
            for (int v = -2; v <= 2; v++)
                wire(level, s.at(b, 0, v));
            // Line 2: power at v = 2, wire v = 1, repeater at v = 0 driving north, wire v = -1, lamp v = -2.
            set(level, s, c, 0, -2, lamp);
            wire(level, s.at(c, 0, -1));
            set(level, s, c, 0, 0, Blocks.REPEATER.defaultBlockState().setValue(RepeaterBlock.FACING, Direction.SOUTH));
            wire(level, s.at(c, 0, 1));
            set(level, s, b, 0, 3, Blocks.REDSTONE_BLOCK.defaultBlockState());
            set(level, s, c, 0, 2, Blocks.REDSTONE_BLOCK.defaultBlockState());
            helper.runAfterDelay(20, () -> {
                int[] power = new int[5];
                for (int i = 0; i < 5; i++)
                    power[i] = level.getBlockState(s.at(b, 0, 2 - i)).getValue(RedStoneWireBlock.POWER);
                log("%s: wire power v = 2 .. -2 (seam between 0 and -1): %d %d %d | %d %d", s.name(), power[0], power[1], power[2], power[3], power[4]);
                for (int i = 1; i < 5; i++)
                    check(power[i] == power[i - 1] - 1, "%s: power falls by one per block across the seam: %d then %d", s.name(), power[i - 1], power[i]);
                check(WrapEngine.rawState(level, s.stored(s.at(b, 0, -3))).getValue(RedstoneLampBlock.LIT), "%s: the far lamp is lit by wire", s.name());
                check(WrapEngine.rawState(level, s.at(c, 0, 0)).getValue(RepeaterBlock.POWERED), "%s: repeater powered", s.name());
                BlockPos farWire = s.stored(s.at(c, 0, -1));
                check(WrapEngine.rawState(level, farWire).getValue(RedStoneWireBlock.POWER) == 15, "%s: the wire across from the repeater at %d",
                        s.name(), WrapEngine.rawState(level, farWire).getValue(RedStoneWireBlock.POWER));
                check(WrapEngine.rawState(level, s.stored(s.at(c, 0, -2))).getValue(RedstoneLampBlock.LIT), "%s: the far lamp is lit through the repeater",
                        s.name());
                BlockPos rep = s.at(c, 0, 0);
                for (int[] q : s.local().images(rep.getX(), rep.getZ())) {
                    BlockPos img = new BlockPos(q[0], rep.getY(), q[1]);
                    BlockState copy = WrapEngine.rawState(level, img);
                    Direction facing = WrapEngine.turn(s.iso(img).inverse(), Direction.SOUTH);
                    check(copy.is(Blocks.REPEATER) && copy.getValue(RepeaterBlock.FACING) == facing && copy.getValue(RepeaterBlock.POWERED),
                            "%s: the repeater's band copy at %s is turned (facing %s) and powered: %s", s.name(), img.toShortString(), facing, copy);
                }
                audit(level, s, "with the power on");
                set(level, s, b, 0, 3, Blocks.AIR.defaultBlockState());
                set(level, s, c, 0, 2, Blocks.AIR.defaultBlockState());
                helper.runAfterDelay(20, () -> {
                    int far0 = WrapEngine.rawState(level, s.stored(s.at(b, 0, -1))).getValue(RedStoneWireBlock.POWER);
                    int far1 = WrapEngine.rawState(level, s.stored(s.at(b, 0, -2))).getValue(RedStoneWireBlock.POWER);
                    check(far0 == 0 && far1 == 0, "%s: power removed: far wires at %d and %d", s.name(), far0, far1);
                    check(!WrapEngine.rawState(level, s.stored(s.at(b, 0, -3))).getValue(RedstoneLampBlock.LIT)
                            && !WrapEngine.rawState(level, s.stored(s.at(c, 0, -2))).getValue(RedstoneLampBlock.LIT), "%s: both far lamps off again", s.name());
                    audit(level, s, "with the power off");
                    helper.succeed();
                });
            });
        });
    }

    /** A redstone wire at a site position (stored or beyond the seam), its shape from its neighbours. */
    private static void wire(ServerLevel level, BlockPos p) {
        BlockState w = Block.updateFromNeighbourShapes(Blocks.REDSTONE_WIRE.defaultBlockState(), level, p);
        level.setBlock(p, w, Block.UPDATE_ALL);
    }

    // --- Light ----------------------------------------------------------------------------------

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 200)
    public static void lightAcrossHalfTurn(GameTestHelper helper) {
        light(helper, site(Kind.HALF, LIGHT));
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 200)
    public static void lightAcrossQuarterTurn(GameTestHelper helper) {
        light(helper, site(Kind.QUARTER, LIGHT));
    }

    /**
     * Glowstone stored right at the seam lights the far side through its band copy: the far-side block
     * across the seam from it gets block light 14, though in ordinary space it is more than 15 blocks away.
     */
    static void light(GameTestHelper helper, Site s0) {
        Site s = open(helper, s0);
        ServerLevel level = helper.getLevel();
        int g = s.kind() == Kind.HALF ? 10 : 20;
        whenSynced(helper, s, () -> {
            BlockPos glow = s.at(g, 1, 0);
            set(level, s, g, 1, 0, Blocks.GLOWSTONE.defaultBlockState());
            BlockPos far = s.stored(s.at(g, 1, -1));
            int ordinary = Math.abs(far.getX() - glow.getX()) + Math.abs(far.getZ() - glow.getZ());
            helper.runAfterDelay(30, () -> {
                int light = level.getBrightness(LightLayer.BLOCK, far);
                check(ordinary > 15, "%s: the far-side block %s is %d blocks from the glowstone in ordinary space", s.name(), far.toShortString(), ordinary);
                check(light >= 13, "%s: block light on the far side across the seam from the glowstone: %d (0 without the seam)", s.name(), light);
                helper.succeed();
            });
        });
    }

    // --- Band consistency -----------------------------------------------------------------------

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void bandConsistentHalfTurn(GameTestHelper helper) {
        consistency(helper, site(Kind.HALF, CONSISTENCY));
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void bandConsistentQuarterTurn(GameTestHelper helper) {
        consistency(helper, site(Kind.QUARTER, CONSISTENCY));
    }

    /**
     * The band stays a turned copy of the stored world after edits on either side of the seam (stored
     * blocks near the edge), through the band (folded writes), and straight into chunk storage: a direct
     * {@code LevelChunk.setBlockState} into a band chunk is redirected to the stored block (the choke
     * point), and one into a stored chunk is replicated.
     */
    static void consistency(GameTestHelper helper, Site s0) {
        Site s = open(helper, s0);
        ServerLevel level = helper.getLevel();
        whenSynced(helper, s, () -> {
            audit(level, s, "after opening");
            Random r = new Random(s.kind().ordinal() * 31L + 7);
            int flags = Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE;
            // Stored blocks within the band's depth of the edge, on both sides of the seam.
            for (int i = 0; i < 400; i++) {
                BlockPos p = s.at(s.uMin() + r.nextInt(s.uMax() - s.uMin()), r.nextInt(5), r.nextInt(DEPTH));
                level.setBlock(p, randomState(r), flags);
            }
            audit(level, s, "after 400 edits on the stored side");
            // Through the band.
            for (int i = 0; i < 400; i++) {
                BlockPos q = s.at(s.uMin() + r.nextInt(s.uMax() - s.uMin()), r.nextInt(5), -1 - r.nextInt(DEPTH));
                BlockState st = randomState(r);
                level.setBlock(q, st, flags);
                if (i < 40) {
                    BlockState stored = WrapEngine.rawState(level, s.stored(q));
                    check(stored == st.rotate(WrapEngine.rotation(s.iso(q))), "%s: written at band %s, stored %s holds it turned: %s", s.name(),
                            q.toShortString(), s.stored(q).toShortString(), stored);
                }
            }
            audit(level, s, "after 400 edits through the band");
            // Straight into chunk storage: a band chunk (the choke point) and a stored chunk (replication).
            BlockState stairs = Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, Direction.EAST);
            BlockPos q = s.at(s.kind() == Kind.HALF ? 9 : 17, 2, -2);
            LevelChunk bandChunk = level.getChunkAt(q);
            bandChunk.setBlockState(q, stairs, false);
            BlockState storedNow = WrapEngine.rawState(level, s.stored(q));
            check(storedNow == stairs.rotate(WrapEngine.rotation(s.iso(q))), "%s: a direct chunk write into band chunk %s went to the stored block %s: %s",
                    s.name(), bandChunk.getPos(), s.stored(q).toShortString(), storedNow);
            check(WrapEngine.rawState(level, q) == stairs, "%s: and the band copy shows what was written: %s", s.name(), WrapEngine.rawState(level, q));
            BlockPos p = s.at(s.kind() == Kind.HALF ? -9 : 33, 3, 1);
            BlockState log = Blocks.OAK_LOG.defaultBlockState().setValue(RotatedPillarBlock.AXIS, Direction.Axis.Z);
            level.getChunkAt(p).setBlockState(p, log, false);
            int images = s.local().images(p.getX(), p.getZ()).size();
            check(images > 0, "%s: the stored block %s has %d band images", s.name(), p.toShortString(), images);
            audit(level, s, "after direct chunk writes on both sides");
            log("%s: %s", s.name(), WrapEngine.stats(level));
            helper.succeed();
        });
    }

    private static BlockState randomState(Random r) {
        Direction h = Direction.Plane.HORIZONTAL.getRandomDirection(net.minecraft.util.RandomSource.create(r.nextLong()));
        return switch (r.nextInt(6)) {
            case 0 -> Blocks.AIR.defaultBlockState();
            case 1 -> Blocks.GLASS.defaultBlockState();
            case 2 -> Blocks.OAK_LOG.defaultBlockState().setValue(RotatedPillarBlock.AXIS, Direction.Axis.values()[r.nextInt(3)]);
            case 3 -> Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, h).setValue(StairBlock.HALF, r.nextBoolean() ? Half.TOP : Half.BOTTOM);
            case 4 -> Blocks.OAK_SLAB.defaultBlockState().setValue(SlabBlock.TYPE, r.nextBoolean() ? SlabType.TOP : SlabType.BOTTOM);
            default -> Blocks.FURNACE.defaultBlockState().setValue(net.minecraft.world.level.block.FurnaceBlock.FACING, h);
        };
    }

    // --- Chunk unload and reload ----------------------------------------------------------------

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 3000)
    public static void bandReloadHalfTurn(GameTestHelper helper) {
        reload(helper, LONG_HALF);
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 3000)
    public static void bandReloadQuarterTurn(GameTestHelper helper) {
        reload(helper, LONG_QUARTER);
    }

    /**
     * A band chunk 15 chunks from its source: (1) the source is edited while the band chunk is not
     * loaded, and the write does not load it; (2) with the source unloaded too, the band chunk is loaded:
     * its ticket loads the source and it is copied; (3) unloading the band chunk releases the source;
     * (4) the source is edited again while the band chunk is unloaded, and reloading it shows the edit.
     */
    static void reload(GameTestHelper helper, Site s) {
        ServerLevel level = helper.getLevel();
        ensureInstalled(level);
        BlockPos q0 = s.at(s.uMax() - 12, 0, -3);
        ChunkPos band = new ChunkPos(q0);
        WrapTopology.ChunkFold cf = s.local().chunkFold(band.x, band.z);
        ChunkPos src = new ChunkPos(cf.sourceX(), cf.sourceZ());
        int apart = Math.max(Math.abs(band.x - src.x), Math.abs(band.z - src.z));
        check(apart >= 10, "%s: band chunk %s and its source %s are %d chunks apart", s.name(), band, src, apart);
        boolean noSave = level.noSave;
        level.noSave = false; // chunks unload only while the level saves
        BlockPos[] marks = {q0, s.at(s.uMax() - 9, 1, -1), s.at(s.uMax() - 14, 2, -6)};
        BlockState[] first = {Blocks.OAK_LOG.defaultBlockState().setValue(RotatedPillarBlock.AXIS, Direction.Axis.X),
                Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, Direction.WEST), Blocks.GLOWSTONE.defaultBlockState()};
        BlockState[] second = {Blocks.GLASS.defaultBlockState(), Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, Direction.NORTH),
                Blocks.OAK_LOG.defaultBlockState().setValue(RotatedPillarBlock.AXIS, Direction.Axis.Z)};
        waitUntil(helper, 400, "band chunk and source unloaded at the start", () -> !loaded(level, band) && !loaded(level, src), () -> {
            // (1) Edit the source with the band chunk unloaded.
            holdBorder(level, src, true);
            writeMarks(level, s, marks, first);
            check(!loaded(level, band), "%s: editing the source did not load band chunk %s", s.name(), band);
            holdBorder(level, src, false);
            waitUntil(helper, 600, "source " + src + " to unload", () -> !loaded(level, src), () -> {
                // (2) Load the band chunk alone: its ticket brings the source, then the copy.
                long copies0 = copies(level);
                level.setChunkForced(band.x, band.z, true);
                waitUntil(helper, 600, "band chunk " + band + " copied from its source", () -> loaded(level, band) && WrapEngine.isBandSynced(level, band.x, band.z), () -> {
                    check(loaded(level, src), "%s: the band chunk's ticket loaded its source %s", s.name(), src);
                    check(copies(level) > copies0, "%s: a copy ran (%s)", s.name(), WrapEngine.stats(level));
                    checkChunk(level, s, band, "after loading with its source unloaded");
                    checkMarks(level, s, marks, first, "first marks");
                    // (3) Unload the band chunk: the source goes too.
                    level.setChunkForced(band.x, band.z, false);
                    waitUntil(helper, 600, "band chunk and then its source to unload", () -> !loaded(level, band) && !loaded(level, src), () -> {
                        log("%s: band chunk %s unloaded and released its source %s", s.name(), band, src);
                        // (4) Edit the source while the band chunk is unloaded, then reload it.
                        holdBorder(level, src, true);
                        writeMarks(level, s, marks, second);
                        level.setChunkForced(band.x, band.z, true);
                        waitUntil(helper, 600, "band chunk " + band + " copied again", () -> loaded(level, band) && WrapEngine.isBandSynced(level, band.x, band.z), () -> {
                            checkChunk(level, s, band, "after reloading");
                            checkMarks(level, s, marks, second, "second marks");
                            level.setChunkForced(band.x, band.z, false);
                            holdBorder(level, src, false);
                            level.noSave = noSave;
                            log("%s: %s", s.name(), WrapEngine.stats(level));
                            helper.succeed();
                        });
                    });
                });
            });
        });
    }

    /** The test's own hold on a chunk while it edits it. */
    static final net.minecraft.server.level.TicketType<ChunkPos> TEST_TICKET = net.minecraft.server.level.TicketType.create("deeptime_wrap_test",
            java.util.Comparator.comparingLong(ChunkPos::toLong));

    /**
     * Holds a source chunk at the border level (33, FULL but not ticking) while the test edits it. A
     * forced ticket (31) would load its neighbours across the seam at 32, which are band chunks; their
     * tickets would load the stored chunk next to the band chunk under test at 32, and that one's
     * neighbour, the band chunk itself, at 33: correct (the far side of a loaded place is loaded) but
     * it would defeat a test of a band chunk loading after its source was edited.
     */
    private static void holdBorder(ServerLevel level, ChunkPos p, boolean on) {
        if (on) {
            level.getChunkSource().addRegionTicket(TEST_TICKET, p, 0, p);
            level.getChunk(p.x, p.z);
        } else {
            level.getChunkSource().removeRegionTicket(TEST_TICKET, p, 0, p);
        }
    }

    private static boolean loaded(ServerLevel level, ChunkPos p) {
        return level.getChunkSource().getChunkNow(p.x, p.z) != null;
    }

    private static long copies(ServerLevel level) {
        String st = WrapEngine.stats(level);
        int i = st.indexOf("copies=");
        return i < 0 ? 0 : Long.parseLong(st.substring(i + 7, st.indexOf(' ', i)));
    }

    /** Writes the marks at the stored positions the band positions {@code marks} show, turned so they read as given there. */
    private static void writeMarks(ServerLevel level, Site s, BlockPos[] marks, BlockState[] states) {
        for (int i = 0; i < marks.length; i++)
            level.setBlock(s.stored(marks[i]), states[i].rotate(WrapEngine.rotation(s.iso(marks[i]))), Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE);
    }

    private static void checkMarks(ServerLevel level, Site s, BlockPos[] marks, BlockState[] states, String what) {
        for (int i = 0; i < marks.length; i++) {
            BlockState raw = WrapEngine.rawState(level, marks[i]);
            check(raw == states[i], "%s: %s: band copy at %s is %s (want %s)", s.name(), what, marks[i].toShortString(), raw, states[i]);
        }
    }

    /** Every block of a band chunk, at every height, is its source's block turned. */
    private static void checkChunk(ServerLevel level, Site s, ChunkPos band, String when) {
        LevelChunk b = level.getChunkSource().getChunkNow(band.x, band.z);
        int wrong = 0, checked = 0;
        for (int x = band.getMinBlockX(); x <= band.getMaxBlockX(); x++)
            for (int z = band.getMinBlockZ(); z <= band.getMaxBlockZ(); z++) {
                WrapTopology.Fold f = s.local().fold(x, z);
                for (int y = level.getMinBuildHeight(); y < level.getMaxBuildHeight(); y++) {
                    BlockState want = level.getBlockState(new BlockPos(f.x(), y, f.z())).rotate(WrapEngine.rotation(f.iso().inverse()));
                    if (b.getBlockState(new BlockPos(x, y, z)) != want)
                        wrong++;
                    checked++;
                }
            }
        check(wrong == 0, "%s: band chunk %s %s: %d of %d blocks differ from their source turned", s.name(), band, when, wrong, checked);
    }

    // --- A player crossing ----------------------------------------------------------------------

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 200)
    public static void playerCrossesHalfTurn(GameTestHelper helper) {
        player(helper, site(Kind.HALF, PLAYER));
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 200)
    public static void playerCrossesQuarterTurn(GameTestHelper helper) {
        player(helper, site(Kind.QUARTER, PLAYER));
    }

    /** A (mock) player that steps beyond the edge is teleported to where it is, heading turned. */
    @SuppressWarnings("removal")
    static void player(GameTestHelper helper, Site s0) {
        Site s = open(helper, s0);
        ServerLevel level = helper.getLevel();
        int u = s.kind() == Kind.HALF ? 8 : 24;
        whenSynced(helper, s, () -> {
            ServerPlayer p = helper.makeMockServerPlayerInLevel();
            p.teleportTo(level, s.ox() + u + 0.5, Y0, s.oz() + 0.5, 0f, 0f);
            float yaw0 = 0f;
            helper.runAfterDelay(5, () -> {
                check(Math.abs(p.getZ() - (s.oz() + 0.5)) < 1e-3, "%s: the player stands on this side at z = oz + %.3f", s.name(), p.getZ() - s.oz());
                p.teleportTo(level, s.ox() + u + 0.5, Y0, s.oz() - 0.5, yaw0, 0f);
                WrapTopology.PointFold expect = s.local().foldPoint(p.getX(), p.getZ());
                helper.runAfterDelay(3, () -> {
                    check(Math.abs(p.getX() - expect.x()) < 1e-3 && Math.abs(p.getZ() - expect.z()) < 1e-3,
                            "%s: the player was carried to (%.3f, %.3f), the topology's fold (%.3f, %.3f)", s.name(), p.getX(), p.getZ(), expect.x(),
                            expect.z());
                    float turn = ((p.getYRot() - yaw0) % 360f + 360f) % 360f;
                    check(Math.abs(turn - expect.iso().yawDegrees()) < 1e-3, "%s: the player's heading turned by %.1f (motion %.1f)", s.name(), turn,
                            expect.iso().yawDegrees());
                    level.getServer().getPlayerList().remove(p);
                    helper.succeed();
                });
            });
        });
    }

    // --- The cube (design §12) ------------------------------------------------------------------

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void cubeBuildingAcrossQuarterTurn(GameTestHelper helper) {
        building(helper, CUBE_QUARTER);
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void cubeBuildingAcrossHalfTurn(GameTestHelper helper) {
        building(helper, CUBE_HALF);
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void cubeBandConsistentQuarterTurn(GameTestHelper helper) {
        consistency(helper, CUBE_QUARTER_EDITS);
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void cubeBandConsistentAtToroidalSeam(GameTestHelper helper) {
        consistency(helper, CUBE_HALF_TW);
    }

    /**
     * The fold of (x, z) beyond the cube's east slide done in two steps: the slide by −4s, then the
     * fold of the block it reaches (none when that block is stored).
     */
    private static WrapTopology.Fold slideThenFold(int x, int z) {
        int p = 4 * CUBE_S;
        CellIsometry slide = CellIsometry.translation(-p, 0);
        WrapTopology.Fold g = CUBE.fold(x - p, z);
        return g == null ? new WrapTopology.Fold(x - p, z, slide) : new WrapTopology.Fold(g.x(), g.z(), g.iso().compose(slide));
    }

    /**
     * The slide composed with a turn (design §12.3): beyond x = 2s the engine's own slide takes a block
     * to F2's east half at x − 4s; where that lies beyond F2's bottom edge too, the slide's image is
     * beyond the piece of F2-bottom cut at Toroidal World's seam and the block is on S, half turned. The
     * fold is the slide, then the fold there. A block placed in each lands there and the band shows it;
     * an armour stand stepping east across x = 2s is carried by the slide alone, one stepping
     * diagonally off F2's bottom-east corner by the slide and the half turn, heading turned 180°.
     */
    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void cubeSlideComposedWithTurn(GameTestHelper helper) {
        Site s = open(helper, CUBE_SLIDE);
        ServerLevel level = helper.getLevel();
        int east = CUBE_X0 + 2 * CUBE_S, bottom = CUBE_Z0 + CUBE_H;
        CubeLayout net = CubeLayout.of(CUBE_S);
        whenSynced(helper, s, () -> {
            BlockState north = Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, Direction.NORTH);
            BlockPos[] qs = {new BlockPos(east + 5, Y0, bottom - 12), new BlockPos(east + 5, Y0, bottom + 3)};
            int[] turns = {0, 2};
            CubeLayout.Face[] faces = {CubeLayout.Face.F2, CubeLayout.Face.S};
            for (int i = 0; i < qs.length; i++) {
                BlockPos q = qs[i];
                WrapTopology.Fold f = CUBE.fold(q.getX(), q.getZ());
                WrapTopology.Fold g = slideThenFold(q.getX(), q.getZ());
                check(f != null && f.x() == g.x() && f.z() == g.z() && f.iso().equals(g.iso()) && f.iso().quarterTurns() == turns[i],
                        "%s: the fold of %s is the slide then the fold there: %s, two steps %s", s.name(), q.toShortString(), f, g);
                CubeLayout.FaceCell fc = net.faceOfBlock(f.x() - CUBE_X0, f.z() - CUBE_Z0);
                check(fc != null && fc.face() == faces[i], "%s: %s is stored on %s (%s)", s.name(), q.toShortString(), faces[i], fc);
                level.setBlock(q, north, Block.UPDATE_ALL);
                BlockPos stored = new BlockPos(f.x(), q.getY(), f.z());
                Direction facing = WrapEngine.turn(f.iso(), Direction.NORTH);
                BlockState at = WrapEngine.rawState(level, stored);
                check(at.is(Blocks.OAK_STAIRS) && at.getValue(StairBlock.FACING) == facing, "%s: placed at %s facing north -> stored at %s facing %s: %s",
                        s.name(), q.toShortString(), stored.toShortString(), facing, at);
                check(WrapEngine.rawState(level, q).equals(north) && level.getBlockState(q).equals(north), "%s: the band at %s shows it: %s", s.name(),
                        q.toShortString(), WrapEngine.rawState(level, q));
            }
            ArmorStand[] st = {stand(level, east - 0.5, bottom - 11.5), stand(level, east - 0.5, bottom - 0.5)};
            float[] yaw0 = {st[0].getYRot(), st[1].getYRot()};
            st[0].move(MoverType.SELF, new Vec3(1, 0, 0));
            st[1].move(MoverType.SELF, new Vec3(1, 0, 1));
            WrapTopology.PointFold[] want = new WrapTopology.PointFold[st.length];
            for (int i = 0; i < st.length; i++) {
                want[i] = CUBE.foldPoint(st[i].getX(), st[i].getZ());
                check(want[i] != null && want[i].iso().quarterTurns() == turns[i] && s.inBoxes(want[i].x(), want[i].z()),
                        "%s: stand %d stepped to (%.2f, %.2f), beyond the slide: it goes to %s", s.name(), i, st[i].getX() - CUBE_X0, st[i].getZ() - CUBE_Z0, want[i]);
            }
            helper.runAfterDelay(20, () -> {
                for (int i = 0; i < st.length; i++) {
                    ArmorStand e = st[i];
                    check(Math.abs(e.getX() - want[i].x()) < 1e-3 && Math.abs(e.getZ() - want[i].z()) < 1e-3,
                            "%s: stand %d carried to (%.3f, %.3f), the fold (%.3f, %.3f)", s.name(), i, e.getX(), e.getZ(), want[i].x(), want[i].z());
                    float turn = ((e.getYRot() - yaw0[i]) % 360f + 360f) % 360f;
                    check(Math.abs(turn - 90f * turns[i]) < 1e-3, "%s: stand %d's heading turned by %.1f degrees (want %d)", s.name(), i, turn, 90 * turns[i]);
                    check(s.local().inside(Mth.floor(e.getX()), Mth.floor(e.getZ())), "%s: stand %d is on a stored block", s.name(), i);
                    e.discard();
                }
                audit(level, s, "after the slide");
                log("%s: %s", s.name(), WrapEngine.stats(level));
                helper.succeed();
            });
        });
    }

    /**
     * The cube's corners (design §12.4; ledger SCL-182, SCL-183): every image of the eight cone points
     * is kept out (a seam link within 2 blocks of one is refused from both ends; 8 blocks along the
     * same edge it is made); blocks written into the missing quadrant at the concave corner (h, h),
     * where two bands overlap and the chunk is split by its diagonal, land on the stored blocks they
     * are and the band shows them; armour stands left in that quadrant and in the convex quadrant at
     * S's outer corner are carried onto the planet, heading turned by their block's motion.
     */
    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void cubeCornerRefusal(GameTestHelper helper) {
        Site s = open(helper, CUBE_CORNER);
        ServerLevel level = helper.getLevel();
        WrapTopology t = WrapEngine.topology(level);
        int cx = CUBE_X0 + CUBE_H, cz = CUBE_Z0 + CUBE_H;
        whenSynced(helper, s, () -> {
            int images = 0;
            for (WrapCorner c : CUBE.corners())
                for (int[] q : c.images()) {
                    images++;
                    boolean in = WrapEngine.nearCorner(level, new BlockPos(q[0], Y0, q[1])) && WrapEngine.nearCorner(level, new BlockPos(q[0] - 2, Y0, q[1] - 2))
                            && WrapEngine.nearCorner(level, new BlockPos(q[0] + 1, Y0, q[1] + 1));
                    boolean out = !WrapEngine.nearCorner(level, new BlockPos(q[0] + 2, Y0, q[1])) && !WrapEngine.nearCorner(level, new BlockPos(q[0] - 3, Y0, q[1]))
                            && !WrapEngine.nearCorner(level, new BlockPos(q[0], Y0, q[1] + 8));
                    check(in && out, "corner %s's image (%d, %d) is kept out within 2 blocks, and only there", c.name(), q[0] - CUBE_X0, q[1] - CUBE_Z0);
                }
            check(images == 12, "the cube's eight corners have 12 images on the net (n1, n2, s1, s2 twice): %d", images);
            // A seam link across F1-bottom from F1's bottom row: refused one block from the corner, made 8 along.
            for (int along : new int[]{1, 8}) {
                int vx = cx + along, vz = cz - 1;
                WrapTopology.Fold f = t.fold(vx, vz + 1);
                check(f != null, "%s: the block south of F1's bottom row at %d along is beyond F1-bottom", s.name(), along);
                int[] there = SeamTurns.seamLink(t, vx, vz, f.x(), f.z());
                int[] back = SeamTurns.seamLink(t, f.x(), f.z(), vx, vz);
                if (along == 1)
                    check(there == null && back == null, "%s: the link next to the corner is refused from both ends", s.name());
                else
                    check(there != null && there[0] == vx && there[1] == vz + 1 && back != null, "%s: the link %d along is made from both ends", s.name(), along);
            }
            // Writes into the missing quadrant: each lands on its own stored block, turned, and the band shows it.
            BlockState north = Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, Direction.NORTH);
            int[][] cells = {{3, 3}, {5, 2}, {2, 5}, {12, 1}, {1, 12}};
            for (int[] c : cells) {
                BlockPos q = new BlockPos(cx + c[0], Y0 + 1, cz + c[1]);
                WrapTopology.Fold f = CUBE.fold(q.getX(), q.getZ());
                level.setBlock(q, north, Block.UPDATE_ALL);
                BlockPos stored = new BlockPos(f.x(), q.getY(), f.z());
                Direction facing = WrapEngine.turn(f.iso(), Direction.NORTH);
                BlockState at = WrapEngine.rawState(level, stored);
                check(at.is(Blocks.OAK_STAIRS) && at.getValue(StairBlock.FACING) == facing && s.local().inside(f.x(), f.z()),
                        "%s: placed at corner + (%d, %d) -> stored at %s facing %s: %s", s.name(), c[0], c[1], stored.toShortString(), facing, at);
                check(WrapEngine.rawState(level, q).equals(north), "%s: the split band chunk at corner + (%d, %d) shows it: %s", s.name(), c[0], c[1],
                        WrapEngine.rawState(level, q));
            }
            audit(level, s, "after writes into the split quadrant");
            // Stands in the missing quadrant at (h, h) and in the convex quadrant at S's outer corner (−h, 3h).
            int ox = CUBE_X0 - CUBE_H, oz = CUBE_Z0 + 3 * CUBE_H;
            double[][] spots = {{cx + 0.5, cz + 0.5}, {cx + 4.5, cz + 4.5}, {cx + 7.5, cz + 2.5}, {cx + 2.5, cz + 7.5},
                    {ox - 0.5, oz + 0.5}, {ox - 4.5, oz + 4.5}, {ox - 7.5, oz + 2.5}, {ox - 2.5, oz + 7.5}};
            ArmorStand[] st = new ArmorStand[spots.length];
            WrapTopology.PointFold[] want = new WrapTopology.PointFold[spots.length];
            for (int i = 0; i < spots.length; i++) {
                want[i] = CUBE.foldPoint(spots[i][0], spots[i][1]);
                check(want[i] != null && s.inBoxes(want[i].x(), want[i].z()), "%s: stand %d at (%.1f, %.1f) is beyond an edge, going to %s", s.name(), i,
                        spots[i][0] - CUBE_X0, spots[i][1] - CUBE_Z0, want[i]);
                st[i] = stand(level, spots[i][0], spots[i][1]);
            }
            helper.runAfterDelay(10, () -> {
                for (int i = 0; i < st.length; i++) {
                    ArmorStand e = st[i];
                    check(Math.abs(e.getX() - want[i].x()) < 1e-3 && Math.abs(e.getZ() - want[i].z()) < 1e-3
                                    && s.local().inside(Mth.floor(e.getX()), Mth.floor(e.getZ())),
                            "%s: stand %d carried onto the planet at (%.3f, %.3f), the fold (%.3f, %.3f)", s.name(), i, e.getX() - CUBE_X0,
                            e.getZ() - CUBE_Z0, want[i].x() - CUBE_X0, want[i].z() - CUBE_Z0);
                    float turn = ((e.getYRot() - 180f) % 360f + 360f) % 360f;
                    check(Math.abs(turn - want[i].iso().yawDegrees()) < 1e-3, "%s: stand %d's heading turned by %.1f (motion %.1f)", s.name(), i, turn,
                            want[i].iso().yawDegrees());
                    e.discard();
                }
                log("%s: %s", s.name(), WrapEngine.stats(level));
                helper.succeed();
            });
        });
    }
}
