// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapspike;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestAssertException;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.MoverType;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.RedStoneWireBlock;
import net.minecraft.world.level.block.RedstoneLampBlock;
import net.minecraft.world.level.block.RepeaterBlock;
import net.minecraft.world.level.block.StairBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.RedstoneSide;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;
import net.minecraft.world.phys.Vec3;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * The wrap-engine spike's proof (docs/design/wrap-engine.md §7): one half-turn seam, and across it a
 * placed block shows on the far side turned and is solid there, water flows, redstone powers, and
 * entities collide with the far side and are carried over turned.
 *
 * <p>Each test builds a site of 16 × 16 blocks far from the GameTest grid in the GameTest server's
 * flat overworld: the stored side is z ≥ z0, the band z ∈ [z0 − 8, z0), x ∈ [cx − 8, cx + 8), glued by
 * the half-turn about (cx, z0) — block (x, z) beyond the seam is block (2cx − x − 1, 2z0 − z − 1), the
 * block form of the Guyou edge (x, z) → (−x, 2b − z). Sites share the level's topology (their edges are
 * unioned), so the tests can run in one batch.
 *
 * <p>Namespace {@code deeptime_wrapspike}, enabled only by {@code ./gametest.sh -PwrapSpike}, which also
 * sets {@code -Ddeeptime.wrapspike=true}; never part of the normal suite.
 */
@GameTestHolder(DeepTime.MODID)
@PrefixGameTestTemplate(false)
public final class WrapSpikeGameTests {

    static final String NS = "deeptime_wrapspike";
    static final String TEMPLATE = "empty_5x5x5";
    static final int HALF = 8;
    static final int Y0 = 100;

    private static final List<WrapEdge> EDGES = new ArrayList<>();

    private WrapSpikeGameTests() {}

    /** One test site: its seam pivot and whether its seam is glued (a control site is not). */
    record Site(ServerLevel level, int cx, int z0, boolean glued) {
        BlockPos at(int dx, int dy, int dz) {
            return new BlockPos(cx + dx, Y0 + dy, z0 + dz);
        }

        void set(int dx, int dy, int dz, BlockState s) {
            level.setBlock(at(dx, dy, dz), s, Block.UPDATE_ALL);
        }

        BlockState stored(int dx, int dy, int dz) {
            return WrapEngine.rawState(level, at(dx, dy, dz));
        }
    }

    /** Forceloads, clears and floors a site, then glues its seam and copies its band. */
    static Site open(GameTestHelper helper, int index, boolean glued) {
        ServerLevel level = helper.getLevel();
        int cx = 4096 + 64 * index, z0 = 4096;
        for (int x = (cx - HALF) >> 4; x <= (cx + HALF - 1) >> 4; x++)
            for (int z = (z0 - HALF) >> 4; z <= (z0 + HALF - 1) >> 4; z++) {
                level.setChunkForced(x, z, true);
                level.getChunk(x, z);
            }
        for (int x = cx - HALF; x < cx + HALF; x++)
            for (int z = z0 - HALF; z < z0 + HALF; z++)
                for (int y = Y0 - 1; y <= Y0 + 4; y++)
                    level.setBlock(new BlockPos(x, y, z), y == Y0 - 1 ? Blocks.STONE.defaultBlockState() : Blocks.AIR.defaultBlockState(),
                            Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE);
        Site site = new Site(level, cx, z0, glued);
        if (glued) {
            synchronized (EDGES) {
                EDGES.addAll(WrapTopology.halfTurnSeam(cx, z0, HALF, HALF).edges());
                WrapEngine.install(level, new WrapTopology(EDGES));
            }
            WrapTopology t = WrapEngine.topology(level);
            int n = WrapEngine.resyncBand(level, t, cx - HALF, cx + HALF - 1, Y0 - 1, Y0 + 4, z0 - HALF, z0 + HALF - 1);
            log("site %d at cx=%d z0=%d glued; %d band blocks copied", index, cx, z0, n);
        } else {
            log("site %d at cx=%d z0=%d NOT glued (control)", index, cx, z0);
        }
        return site;
    }

    static void log(String fmt, Object... args) {
        DeepTime.LOGGER.info("[gametest] wrapspike " + String.format(fmt, args));
    }

    static void check(boolean ok, String fmt, Object... args) {
        String msg = String.format(fmt, args);
        log("%s %s", ok ? "OK  " : "FAIL", msg);
        if (!ok)
            throw new GameTestAssertException(msg);
    }

    // ------------------------------------------------------------------------------------------

    /**
     * A block placed beyond the seam lands on the stored block it is, turned 180°; the band shows it
     * (the chunk storage the client and collisions read holds the turned copy); it is solid on both
     * sides; an armour stand walking at the seam is stopped by it from this side; one walking where
     * nothing stands is carried over and turned; a thrown item crosses and keeps going.
     */
    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void placedAcrossSeamIsTurnedAndSolid(GameTestHelper helper) {
        Site s = open(helper, 0, true);
        ServerLevel level = s.level();
        // "Placing across the seam": a set at the band position, as a player's click there ends up.
        BlockState northStairs = Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, Direction.NORTH);
        level.setBlock(s.at(1, 0, -1), northStairs, Block.UPDATE_ALL);
        BlockState atStored = s.stored(-2, 0, 0); // (2cx − (cx+1) − 1, 2z0 − (z0−1) − 1) = (cx − 2, z0)
        check(atStored.is(Blocks.OAK_STAIRS) && atStored.getValue(StairBlock.FACING) == Direction.SOUTH,
                "placed at band (cx+1, z0-1) facing north -> stored at (cx-2, z0): %s", atStored);
        BlockState viewed = level.getBlockState(s.at(1, 0, -1));
        check(viewed.equals(northStairs), "read back through the seam at (cx+1, z0-1): %s", viewed);
        BlockState copy = s.stored(1, 0, -1);
        check(copy.equals(northStairs), "band copy in chunk storage at (cx+1, z0-1) (what the client is sent): %s", copy);
        check(!atStored.getCollisionShape(level, s.at(-2, 0, 0)).isEmpty() && !copy.getCollisionShape(level, s.at(1, 0, -1)).isEmpty(),
                "solid on both sides of the seam");

        // A block placed inside near the seam shows in the band, turned.
        BlockState eastStairs = Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, Direction.EAST);
        s.set(-5, 0, 0, eastStairs);
        BlockState img = s.stored(4, 0, -1); // image of (cx−5, z0): (cx+4, z0−1)
        check(img.is(Blocks.OAK_STAIRS) && img.getValue(StairBlock.FACING) == Direction.WEST,
                "stored at (cx-5, z0) facing east -> band copy at (cx+4, z0-1): %s", img);

        // Collisions: the far side's block stops an entity at the seam on this side.
        ArmorStand blocked = stand(level, s.cx() + 1.5, s.z0() + 0.6);
        blocked.move(MoverType.SELF, new Vec3(0, 0, -1));
        check(Math.abs(blocked.getZ() - (s.z0() + 0.25)) < 1e-3,
                "armour stand (half-width 0.25) at x=cx+1.5 moving 1 north stops at z=%.4f (expected z0+0.25 = %d.25)",
                blocked.getZ(), s.z0());
        // Where the far side is open, it walks over and is carried across, turned.
        ArmorStand walker = stand(level, s.cx() + 3.5, s.z0() + 0.6);
        float yaw0 = walker.getYRot();
        walker.move(MoverType.SELF, new Vec3(0, 0, -1));
        check(walker.getZ() < s.z0(), "open column: armour stand moved past the seam line to z=%.3f", walker.getZ());
        // A thrown item, no gravity, heading north at 0.2 blocks a tick (it moves only once its freshly
        // forceloaded chunk is entity-ticking, which took more than 20 ticks in the first run).
        ItemEntity item = new ItemEntity(level, s.cx() + 5.5, Y0 + 0.5, s.z0() + 1.5, new ItemStack(Items.DIAMOND));
        item.setNoGravity(true);
        item.setPickUpDelay(32767);
        item.setDeltaMovement(0, 0, -0.2);
        level.addFreshEntity(item);

        // Light: glowstone stored right at the seam, at (cx+6, z0). Its band copy is at (cx-7, z0-1),
        // next to the far-side block (cx-7, z0), which is 13 blocks from the glowstone through ordinary
        // space (block light 2 at most that way) and 1 block through the seam (14).
        s.set(6, 1, 0, Blocks.GLOWSTONE.defaultBlockState());
        helper.runAfterDelay(20, () -> {
            check(Math.abs(walker.getX() - (s.cx() - 3.5)) < 1e-3 && Math.abs(walker.getZ() - (2 * s.z0() - (s.z0() - 0.4))) < 1e-3,
                    "armour stand carried to (%.3f, %.3f), expected (cx-3.5, z0+0.4) = (%.1f, %.1f)",
                    walker.getX(), walker.getZ(), s.cx() - 3.5, s.z0() + 0.4);
            float turn = ((walker.getYRot() - yaw0) % 360f + 360f) % 360f;
            check(Math.abs(turn - 180f) < 1e-3, "armour stand's heading turned by %.1f degrees", turn);
            blocked.discard();
            walker.discard();
        });
        helper.runAfterDelay(120, () -> {
            check(Math.abs(item.getX() - (s.cx() - 5.5)) < 1e-3 && item.getZ() > s.z0() + 1 && item.getDeltaMovement().z > 0,
                    "item crossed: now (%.3f, %.3f) moving %.3f in z (expected x = cx-5.5 = %.1f, z > z0+1, moving south)",
                    item.getX(), item.getZ(), item.getDeltaMovement().z, s.cx() - 5.5);
            item.discard();
            int far = level.getBrightness(net.minecraft.world.level.LightLayer.BLOCK, s.at(-7, 1, 0));
            int copy = level.getBrightness(net.minecraft.world.level.LightLayer.BLOCK, s.at(-7, 1, -1));
            check(far >= 13, "block light on the far side next to the glowstone's band copy: %d (band copy itself %d; at most 2 without the seam)",
                    far, copy);
            helper.succeed();
        });
    }

    private static ArmorStand stand(ServerLevel level, double x, double z) {
        ArmorStand a = EntityType.ARMOR_STAND.create(level);
        a.setNoGravity(true);
        a.moveTo(x, Y0, z, 180f, 0f);
        level.addFreshEntity(a);
        return a;
    }

    // ------------------------------------------------------------------------------------------

    /**
     * Water from a source one block inside the seam flows across it into a channel on the far side
     * and runs on down that channel, level by level; on a control site with no seam the far channel
     * stays dry.
     */
    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void waterFlowsAcrossSeam(GameTestHelper helper) {
        Site glued = open(helper, 1, true);
        Site control = open(helper, 2, false);
        for (Site s : List.of(glued, control)) {
            BlockState wall = Blocks.STONE_BRICKS.defaultBlockState();
            // Channel A at x = cx+2, z0..z0+1; channel B at x = cx−3, z0..z0+6 (B's seam row is A's across).
            for (int dy = 0; dy <= 1; dy++) {
                for (int dz = 0; dz <= 2; dz++) {
                    s.set(1, dy, dz, wall);
                    s.set(3, dy, dz, wall);
                }
                s.set(2, dy, 2, wall);
                for (int dz = 0; dz <= 7; dz++) {
                    s.set(-4, dy, dz, wall);
                    s.set(-2, dy, dz, wall);
                }
                s.set(-3, dy, 7, wall);
            }
            s.set(2, 0, 1, Blocks.WATER.defaultBlockState());
        }
        helper.runAfterDelay(160, () -> {
            int[] amounts = new int[7];
            StringBuilder sb = new StringBuilder();
            for (int k = 0; k < 7; k++) {
                amounts[k] = glued.level().getFluidState(glued.at(-3, 0, k)).getAmount();
                sb.append(amounts[k]).append(k < 6 ? "," : "");
            }
            log("far channel B (cx-3, z0..z0+6) water amounts: [%s]; near channel A z0+1: %d, z0: %d", sb,
                    glued.level().getFluidState(glued.at(2, 0, 1)).getAmount(), glued.level().getFluidState(glued.at(2, 0, 0)).getAmount());
            check(amounts[0] > 0, "water crossed the seam into the far channel's first block (amount %d)", amounts[0]);
            for (int k = 1; k < 6; k++)
                check(amounts[k] > 0 && amounts[k] < amounts[k - 1], "far channel z0+%d holds amount %d < %d", k, amounts[k], amounts[k - 1]);
            check(amounts[6] == 0, "the flow ran out where a 7-block run ends (z0+6 amount %d)", amounts[6]);
            Vec3 flow = glued.level().getFluidState(glued.at(-3, 0, 2)).getFlow(glued.level(), glued.at(-3, 0, 2));
            check(flow.z > 0, "far channel flows away from the seam (flow z = %.3f)", flow.z);
            FluidState copy = WrapEngine.rawState(glued.level(), glued.at(2, 0, -1)).getFluidState();
            check(copy.is(Fluids.FLOWING_WATER) && copy.getAmount() == amounts[0],
                    "band copy beyond channel A (what its side sees) is water of amount %d (far first block %d)", copy.getAmount(), amounts[0]);
            int dry = 0;
            for (int k = 0; k < 7; k++)
                dry += control.level().getFluidState(control.at(-3, 0, k)).isEmpty() ? 1 : 0;
            check(dry == 7, "control site without a seam: far channel dry in %d of 7 blocks", dry);
            helper.succeed();
        });
    }

    // ------------------------------------------------------------------------------------------

    /**
     * A redstone line runs across the seam and lights a lamp on the far side, by wire (power falling one
     * per block across the seam) and through a repeater whose output faces the seam; removing the power
     * turns both off again.
     */
    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void redstonePowersAcrossSeam(GameTestHelper helper) {
        Site s = open(helper, 3, true);
        ServerLevel level = s.level();
        BlockState lamp = Blocks.REDSTONE_LAMP.defaultBlockState();
        // Line 1: wire. Lamp at (cx−3, z0+2); wire (cx−3, z0+1); wire placed beyond the seam at
        // (cx+2, z0−1), which is (cx−3, z0); wires (cx+2, z0..z0+2); power at (cx+2, z0+3).
        s.set(-3, 0, 2, lamp);
        wire(s, -3, 1);
        wire(s, 2, -1);
        wire(s, 2, 0);
        wire(s, 2, 1);
        wire(s, 2, 2);
        // Line 2: repeater. Lamp at (cx−6, z0+1); wire placed beyond the seam at (cx+5, z0−1) = (cx−6, z0);
        // repeater at (cx+5, z0) taking input from the south and driving north, into the seam; wire
        // (cx+5, z0+1); power at (cx+5, z0+2).
        s.set(-6, 0, 1, lamp);
        wire(s, 5, -1);
        s.set(5, 0, 0, Blocks.REPEATER.defaultBlockState().setValue(RepeaterBlock.FACING, Direction.SOUTH));
        wire(s, 5, 1);
        BlockState stored = s.stored(-3, 0, 0);
        check(stored.is(Blocks.REDSTONE_WIRE), "wire placed at band (cx+2, z0-1) stored at (cx-3, z0): %s", stored);
        s.set(2, 0, 3, Blocks.REDSTONE_BLOCK.defaultBlockState());
        s.set(5, 0, 2, Blocks.REDSTONE_BLOCK.defaultBlockState());

        helper.runAfterDelay(20, () -> {
            int[] power = {
                    s.stored(2, 0, 2).getValue(RedStoneWireBlock.POWER), s.stored(2, 0, 1).getValue(RedStoneWireBlock.POWER),
                    s.stored(2, 0, 0).getValue(RedStoneWireBlock.POWER), s.stored(-3, 0, 0).getValue(RedStoneWireBlock.POWER),
                    s.stored(-3, 0, 1).getValue(RedStoneWireBlock.POWER)};
            log("wire power along line 1 (near z0+2, z0+1, z0 | far z0, z0+1): %d %d %d | %d %d", power[0], power[1], power[2], power[3], power[4]);
            for (int i = 1; i < power.length; i++)
                check(power[i] == power[i - 1] - 1, "power falls by one per block across the seam: %d then %d", power[i - 1], power[i]);
            RedstoneSide north = s.stored(2, 0, 0).getValue(RedStoneWireBlock.NORTH);
            check(north.isConnected(), "the wire at the seam connects north, into the far side (%s)", north);
            check(s.stored(-3, 0, 2).getValue(RedstoneLampBlock.LIT), "lamp on the far side lit by wire");
            check(s.stored(5, 0, 0).getValue(RepeaterBlock.POWERED), "repeater powered");
            check(s.stored(-6, 0, 0).getValue(RedStoneWireBlock.POWER) == 15,
                    "wire across the seam from the repeater's output at full power (%d)", s.stored(-6, 0, 0).getValue(RedStoneWireBlock.POWER));
            check(s.stored(-6, 0, 1).getValue(RedstoneLampBlock.LIT), "lamp on the far side lit through the repeater");
            BlockState repCopy = s.stored(-6, 0, -1);
            check(repCopy.is(Blocks.REPEATER) && repCopy.getValue(RepeaterBlock.FACING) == Direction.NORTH && repCopy.getValue(RepeaterBlock.POWERED),
                    "band copy of the repeater beyond the far side, turned and powered: %s", repCopy);
            s.set(2, 0, 3, Blocks.AIR.defaultBlockState());
            s.set(5, 0, 2, Blocks.AIR.defaultBlockState());
            helper.runAfterDelay(20, () -> {
                check(s.stored(-3, 0, 0).getValue(RedStoneWireBlock.POWER) == 0 && s.stored(-3, 0, 1).getValue(RedStoneWireBlock.POWER) == 0,
                        "power removed: far wires at %d and %d", s.stored(-3, 0, 0).getValue(RedStoneWireBlock.POWER),
                        s.stored(-3, 0, 1).getValue(RedStoneWireBlock.POWER));
                check(!s.stored(-3, 0, 2).getValue(RedstoneLampBlock.LIT) && !s.stored(-6, 0, 1).getValue(RedstoneLampBlock.LIT),
                        "both far lamps off again");
                helper.succeed();
            });
        });
    }

    /** A redstone wire set at a site position (stored or beyond the seam), its shape from its neighbours. */
    private static void wire(Site s, int dx, int dz) {
        BlockPos p = s.at(dx, 0, dz);
        BlockState w = Block.updateFromNeighbourShapes(Blocks.REDSTONE_WIRE.defaultBlockState(), s.level(), p);
        s.level().setBlock(p, w, Block.UPDATE_ALL);
    }
}
