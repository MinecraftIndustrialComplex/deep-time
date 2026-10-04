// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.minecraftindustrialcomplex.deeptime.wrapspike.CellIsometry;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapEdge;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapEngine;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapSpikeGameTests;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapSpikeGameTests.Site;
import com.simibubi.create.AllBlocks;
import com.simibubi.create.content.contraptions.AssemblyException;
import com.simibubi.create.content.contraptions.bearing.BearingContraption;
import com.simibubi.create.content.fluids.pipes.FluidPipeBlock;
import com.simibubi.create.content.fluids.tank.CreativeFluidTankBlockEntity;
import com.simibubi.create.content.fluids.tank.FluidTankBlockEntity;
import com.simibubi.create.content.kinetics.base.IRotate;
import com.simibubi.create.content.kinetics.base.KineticBlockEntity;
import com.simibubi.create.content.kinetics.belt.item.BeltConnectorItem;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Direction.Axis;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.material.Fluids;
import net.neoforged.neoforge.fluids.FluidStack;

import java.util.ArrayList;
import java.util.List;

import static com.minecraftindustrialcomplex.deeptime.wrapspike.WrapSpikeGameTests.HALF;
import static com.minecraftindustrialcomplex.deeptime.wrapspike.WrapSpikeGameTests.Y0;
import static com.minecraftindustrialcomplex.deeptime.wrapspike.WrapSpikeGameTests.check;
import static com.minecraftindustrialcomplex.deeptime.wrapspike.WrapSpikeGameTests.log;

/**
 * The Create-across-seams GameTests' bodies (CreateSeamGameTests; docs/design/wrap-engine.md, "Create
 * across turning seams"). Loaded only when Create is installed.
 *
 * <p>Every build is placed twice with the same code at the same site offsets: on a glued site, where the
 * blocks set at band offsets (dz &lt; 0) land on the far side of the seam, turned; and on a control site
 * with no seam, where they stay where they were set. The claim checked is that the glued build behaves
 * as the control one does, block for block, once each far block is seen through the seam: its speed
 * times the turn's axis sign ({@link SeamTurns}) equals the control block's speed, and fluid pumped
 * across arrives.
 */
final class CreateSeamChecks {

    /** Site indices (WrapSpikeGameTests uses 0..3): each site is 64 blocks east of the one before. */
    static final int KIN = 8, KIN_CONTROL = 9, FLUID = 10, FLUID_CONTROL = 11, QUARTER = 12, QUARTER_CONTROL = 13, REFUSE = 14;

    private CreateSeamChecks() {}

    /** One block of a build, at a site offset. */
    record Part(int dx, int dy, int dz, BlockState state) {}

    static BlockState shaft(Axis a) {
        return AllBlocks.SHAFT.getDefaultState().setValue(BlockStateProperties.AXIS, a);
    }

    static BlockState cog(Axis a) {
        return AllBlocks.COGWHEEL.getDefaultState().setValue(BlockStateProperties.AXIS, a);
    }

    static BlockState motor(Direction facing) {
        return AllBlocks.CREATIVE_MOTOR.getDefaultState().setValue(BlockStateProperties.FACING, facing);
    }

    static void build(Site s, List<Part> parts) {
        for (Part p : parts)
            s.set(p.dx(), p.dy(), p.dz(), p.state());
    }

    static KineticBlockEntity kinetic(Site s, int dx, int dy, int dz) {
        // Through the Level, so on a glued site a band offset answers with the far block's entity.
        BlockEntity be = s.level().getBlockEntity(s.at(dx, dy, dz));
        return be instanceof KineticBlockEntity k ? k : null;
    }

    /**
     * Compares a line's blocks at the given offsets: near-side blocks have equal speeds on both sites;
     * a far block's stored speed times the turn's axis sign equals the control speed, and that sign is
     * {@code farSign}. Returns the far blocks' stored speeds (for the log).
     */
    static void compareLine(String name, Site glued, Site control, int[][] offsets, int farSign) {
        StringBuilder sb = new StringBuilder();
        for (int[] o : offsets) {
            KineticBlockEntity g = kinetic(glued, o[0], o[1], o[2]);
            KineticBlockEntity c = kinetic(control, o[0], o[1], o[2]);
            check(g != null && c != null, "%s (%d, %d, %d): kinetic block entity on both sites (glued %s, control %s)",
                    name, o[0], o[1], o[2], g, c);
            check(c.getSpeed() != 0, "%s (%d, %d, %d): the control block turns (%.1f rpm)", name, o[0], o[1], o[2], c.getSpeed());
            CellIsometry iso = SeamFrames.fold(glued.level(), glued.at(o[0], o[1], o[2]));
            if (iso == null) {
                check(g.getSpeed() == c.getSpeed(), "%s near (%d, %d, %d): %.1f rpm, control %.1f", name, o[0], o[1], o[2], g.getSpeed(), c.getSpeed());
                sb.append(String.format(" near(%d,%d)=%.0f", o[0], o[2], g.getSpeed()));
                continue;
            }
            BlockState seen = glued.level().getBlockState(glued.at(o[0], o[1], o[2]));
            Axis seenAxis = ((IRotate) seen.getBlock()).getRotationAxis(seen);
            Axis storedAxis = ((IRotate) g.getBlockState().getBlock()).getRotationAxis(g.getBlockState());
            int sign = SeamTurns.axisSign(iso, seenAxis.ordinal());
            check(storedAxis.ordinal() == SeamTurns.turnedAxis(iso.quarterTurns(), seenAxis.ordinal()),
                    "%s far (%d, %d, %d): stored at %s with axis %s, seen as axis %s (turn %d)", name, o[0], o[1], o[2],
                    g.getBlockPos().toShortString(), storedAxis, seenAxis, iso.quarterTurns() * 90);
            check(sign == farSign, "%s far (%d, %d, %d): axis sign of the turn %d, expected %d", name, o[0], o[1], o[2], sign, farSign);
            check(g.getSpeed() * sign == c.getSpeed(),
                    "%s far (%d, %d, %d): stored speed %.1f rpm x sign %d = %.1f, control %.1f (same rotation seen through the seam)",
                    name, o[0], o[1], o[2], g.getSpeed(), sign, g.getSpeed() * sign, c.getSpeed());
            sb.append(String.format(" far(%d,%d)=%.0f@%s", o[0], o[2], g.getSpeed(), g.getBlockPos().toShortString()));
        }
        log("create %s speeds:%s", name, sb);
    }

    // ------------------------------------------------------------------------------------------

    /**
     * Across a half-turn: a z shaft line (stored speed flips), a vertical cog line (keeps its sign) and an
     * x cog line (flips), each driven by a creative motor on the near side; then the near shaft line's
     * first shaft is broken and the far shafts stop.
     */
    static void kineticsAcrossHalfTurn(GameTestHelper helper) {
        Site glued = WrapSpikeGameTests.open(helper, KIN, true);
        Site control = WrapSpikeGameTests.open(helper, KIN_CONTROL, false);
        List<Part> parts = new ArrayList<>();
        // Line A: z shafts at x = +1 from z = +4 across the seam to z = -3; motor at z = +5 facing north.
        for (int dz = 4; dz >= -3; dz--)
            parts.add(new Part(1, 0, dz, shaft(Axis.Z)));
        parts.add(new Part(1, 0, 5, motor(Direction.NORTH)));
        // Line B: vertical cogs at x = +4 from z = +3 to z = -2; motor above the first, facing down.
        for (int dz = 3; dz >= -2; dz--)
            parts.add(new Part(4, 0, dz, cog(Axis.Y)));
        parts.add(new Part(4, 1, 3, motor(Direction.DOWN)));
        // Line C: x cogs at x = +7 from z = +2 to z = -2; motor west of the first, facing east.
        for (int dz = 2; dz >= -2; dz--)
            parts.add(new Part(7, 0, dz, cog(Axis.X)));
        parts.add(new Part(6, 0, 2, motor(Direction.EAST)));
        build(glued, parts);
        build(control, parts);

        helper.runAfterDelay(40, () -> {
            compareLine("half-turn z shafts", glued, control, offsets(1, 0, 4, -3), -1);
            compareLine("half-turn y cogs", glued, control, offsets(4, 0, 3, -2), 1);
            compareLine("half-turn x cogs", glued, control, offsets(7, 0, 2, -2), -1);
            // Break the shaft next to the motor: the far shafts lose their source across the seam.
            glued.set(1, 0, 4, Blocks.AIR.defaultBlockState());
            helper.runAfterDelay(30, () -> {
                StringBuilder sb = new StringBuilder();
                boolean stopped = true;
                for (int dz = 3; dz >= -3; dz--) {
                    KineticBlockEntity k = kinetic(glued, 1, 0, dz);
                    float v = k == null ? Float.NaN : k.getSpeed();
                    stopped &= v == 0;
                    sb.append(String.format(" %d:%.1f", dz, v));
                }
                check(stopped, "shaft broken next to the motor: every shaft of line A stops, far side included (z:speed%s)", sb);
                helper.succeed();
            });
        });
    }

    /** Offsets {dx, dy, dz} for dz from {@code from} down to {@code to}. */
    static int[][] offsets(int dx, int dy, int from, int to) {
        int[][] o = new int[from - to + 1][];
        for (int i = 0; i < o.length; i++)
            o[i] = new int[]{dx, dy, from - i};
        return o;
    }

    /**
     * Across a quarter-turn (as two cube faces meet at a corner): a z shaft line, which is stored as an x
     * line on the far side with its speed flipped, and a vertical cog line, which keeps its sign.
     */
    static void kineticsAcrossQuarterTurn(GameTestHelper helper) {
        Site glued = WrapSpikeGameTests.open(helper, QUARTER, false);
        Site control = WrapSpikeGameTests.open(helper, QUARTER_CONTROL, false);
        ServerLevel level = glued.level();
        int x0 = glued.cx(), z0 = glued.z0();
        // The band north of the corner (x0, z0), x in [x0, x0 + 8), shows the strip inside the corner's
        // west side turned a quarter clockwise: band block (x0 + i, z0 - 1 - j) is stored at (x0 + j, z0 + i).
        WrapSpikeGameTests.glue(level, List.of(new WrapEdge("create-quarter-north", false, true, z0, x0, x0 + HALF, HALF,
                CellIsometry.quarterTurnAbout(1, x0, z0))));
        int n = WrapEngine.resyncBand(level, WrapEngine.topology(level), x0, x0 + HALF - 1, Y0 - 1, Y0 + 4, z0 - HALF, z0 - 1);
        log("site %d at cx=%d z0=%d glued by a quarter turn about its corner; %d band blocks copied", QUARTER, x0, z0, n);
        List<Part> parts = new ArrayList<>();
        for (int dz = 4; dz >= -3; dz--)
            parts.add(new Part(6, 0, dz, shaft(Axis.Z)));
        parts.add(new Part(6, 0, 5, motor(Direction.NORTH)));
        for (int dz = 3; dz >= -2; dz--)
            parts.add(new Part(3, 0, dz, cog(Axis.Y)));
        parts.add(new Part(3, 1, 3, motor(Direction.DOWN)));
        build(glued, parts);
        build(control, parts);
        helper.runAfterDelay(40, () -> {
            compareLine("quarter-turn z shafts", glued, control, offsets(6, 0, 4, -3), -1);
            compareLine("quarter-turn y cogs", glued, control, offsets(3, 0, 3, -2), 1);
            helper.succeed();
        });
    }

    // ------------------------------------------------------------------------------------------

    /**
     * Across a half-turn: a creative fluid tank of water, a pipe, a mechanical pump pushing north (driven
     * by a cog and a creative motor), pipes across the seam and a fluid tank on the far side. Water must
     * reach the far tank, as it reaches the control site's tank.
     */
    static void fluidsAcrossHalfTurn(GameTestHelper helper) {
        Site glued = WrapSpikeGameTests.open(helper, FLUID, true);
        Site control = WrapSpikeGameTests.open(helper, FLUID_CONTROL, false);
        BlockState pipeZ = ((FluidPipeBlock) AllBlocks.FLUID_PIPE.get()).getAxisState(Axis.Z);
        List<Part> parts = new ArrayList<>();
        parts.add(new Part(2, 0, 6, AllBlocks.CREATIVE_FLUID_TANK.getDefaultState()));
        parts.add(new Part(2, 0, 5, pipeZ));
        parts.add(new Part(2, 0, 4, AllBlocks.MECHANICAL_PUMP.getDefaultState().setValue(BlockStateProperties.FACING, Direction.NORTH)));
        for (int dz = 3; dz >= -2; dz--)
            parts.add(new Part(2, 0, dz, pipeZ));
        parts.add(new Part(2, 0, -3, AllBlocks.FLUID_TANK.getDefaultState()));
        parts.add(new Part(3, 0, 4, cog(Axis.Z)));
        parts.add(new Part(3, 0, 5, motor(Direction.NORTH)));
        build(glued, parts);
        build(control, parts);
        helper.runAfterDelay(2, () -> {
            for (Site s : List.of(glued, control)) {
                BlockEntity be = s.level().getBlockEntity(s.at(2, 0, 6));
                check(be instanceof CreativeFluidTankBlockEntity, "creative tank at (2, 0, 6): %s", be);
                ((CreativeFluidTankBlockEntity.CreativeSmartFluidTank) ((CreativeFluidTankBlockEntity) be).getTankInventory())
                        .setContainedFluid(new FluidStack(Fluids.WATER, 1000));
            }
            KineticBlockEntity pump = kinetic(glued, 2, 0, 4);
            log("create fluids: pump speed glued %.1f, control %.1f", pump == null ? Float.NaN : pump.getSpeed(),
                    kinetic(control, 2, 0, 4) == null ? Float.NaN : kinetic(control, 2, 0, 4).getSpeed());
            poll(helper, glued, control, 0);
        });
    }

    private static int tankAmount(Site s) {
        BlockEntity be = s.level().getBlockEntity(s.at(2, 0, -3));
        if (!(be instanceof FluidTankBlockEntity t) || t.getControllerBE() == null)
            return -1;
        FluidStack f = t.getControllerBE().getTankInventory().getFluid();
        return f.is(Fluids.WATER) ? f.getAmount() : (f.isEmpty() ? 0 : -2);
    }

    private static void poll(GameTestHelper helper, Site glued, Site control, int waited) {
        helper.runAfterDelay(20, () -> {
            int g = tankAmount(glued), c = tankAmount(control);
            int t = waited + 20;
            if ((g > 0 && c > 0) || t >= 800) {
                BlockEntity farTank = glued.level().getBlockEntity(glued.at(2, 0, -3));
                log("create fluids after %d ticks: far tank (stored at %s) holds %d mB of water; control tank %d mB", t,
                        farTank == null ? "?" : farTank.getBlockPos().toShortString(), g, c);
                check(c > 0, "control site (no seam): water reached the tank (%d mB)", c);
                check(farTank != null && farTank.getBlockPos().getZ() >= glued.z0(),
                        "the far tank was set at a band offset and is stored on the far side (%s)", farTank == null ? null : farTank.getBlockPos());
                check(g > 0, "water pumped across the half-turn seam reached the far tank (%d mB)", g);
                helper.succeed();
                return;
            }
            poll(helper, glued, control, t);
        });
    }

    // ------------------------------------------------------------------------------------------

    /**
     * The connections a turning seam refuses, without a crash: a belt between a shaft here and one across
     * the seam (a belt between two shafts on this side still connects); track next to the seam (track
     * further away stays); a bearing whose structure lies across the seam (one on this side assembles).
     */
    static void refusalsAtSeam(GameTestHelper helper) {
        Site s = WrapSpikeGameTests.open(helper, REFUSE, true);
        ServerLevel level = s.level();
        // Belts.
        s.set(-6, 0, 1, shaft(Axis.X));
        s.set(-6, 0, -2, shaft(Axis.X));
        s.set(-6, 0, 3, shaft(Axis.X));
        s.set(-6, 0, 6, shaft(Axis.X));
        int belts0 = SeamRefusals.count(SeamRefusals.Kind.BELT);
        boolean across = BeltConnectorItem.canConnect(level, s.at(-6, 0, 1), s.at(-6, 0, -2));
        boolean here = BeltConnectorItem.canConnect(level, s.at(-6, 0, 3), s.at(-6, 0, 6));
        check(!across && SeamRefusals.count(SeamRefusals.Kind.BELT) > belts0,
                "belt from a shaft here to one across the seam refused (canConnect %s, refusals %d -> %d)", across, belts0,
                SeamRefusals.count(SeamRefusals.Kind.BELT));
        check(here, "belt between two shafts on this side still connects");
        // Contraptions.
        s.set(0, 0, -1, Blocks.STONE.defaultBlockState()); // lands on the far side
        int c0 = SeamRefusals.count(SeamRefusals.Kind.CONTRAPTION);
        String acrossMsg;
        try {
            boolean ok = new BearingContraption(false, Direction.NORTH).assemble(level, s.at(0, 0, 0));
            acrossMsg = "assembled (" + ok + ")";
        } catch (AssemblyException e) {
            acrossMsg = "AssemblyException: " + e.component.getString();
        }
        check(acrossMsg.startsWith("AssemblyException") && SeamRefusals.count(SeamRefusals.Kind.CONTRAPTION) > c0,
                "bearing whose structure lies across the seam refused with Create's assembly message (%s)", acrossMsg);
        s.set(3, 0, 3, Blocks.STONE.defaultBlockState());
        String hereMsg;
        try {
            hereMsg = "assembled " + new BearingContraption(false, Direction.NORTH).assemble(level, s.at(3, 0, 4));
        } catch (AssemblyException e) {
            hereMsg = "AssemblyException: " + e.component.getString();
        }
        check(hereMsg.equals("assembled true"), "bearing with its structure on this side assembles (%s)", hereMsg);
        // Track.
        int t0 = SeamRefusals.count(SeamRefusals.Kind.TRACK);
        s.set(0, 0, 0, AllBlocks.TRACK.getDefaultState());
        s.set(-3, 0, 5, AllBlocks.TRACK.getDefaultState());
        helper.runAfterDelay(10, () -> {
            BlockState atSeam = s.stored(0, 0, 0), away = s.stored(-3, 0, 5);
            check(atSeam.isAir() && SeamRefusals.count(SeamRefusals.Kind.TRACK) > t0,
                    "track next to the seam broken one tick after placing (%s; refusals %d -> %d)", atSeam, t0,
                    SeamRefusals.count(SeamRefusals.Kind.TRACK));
            check(away.is(AllBlocks.TRACK.get()), "track five blocks from the seam stays (%s)", away);
            helper.succeed();
        });
    }
}
