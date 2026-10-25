// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.sable.SableTestOps;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.wrap.CubeBorder;
import com.minecraftindustrialcomplex.deeptime.wrap.EdgeBorder;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.UUID;

/**
 * Sable ships at a cube world's temporary border (cube plan W3): the dev planet server with
 * {@code --cube-4k} (planet_world_4k baked as a cube, s = 1,024), whose overworld is the test level.
 * Every Sable call goes through {@link SableTestOps}: no Sable type may appear in this class's method
 * signatures. The slide is {@code jetCrossesSeam} run on the same server at Toroidal World's seam in
 * F2 ({@code --run jetcrossesseam@2041,250,64}).
 */
@GameTestHolder(GameTestSupport.NS_PLANET)
@PrefixGameTestTemplate(false)
public final class SableCubeBorderGameTests {

    /** Launch speed, blocks per second: an Aeronautics jet at full speed or more. */
    static final double SPEED = 60.0;
    /** How long the hulls fly once launched, ticks. */
    static final int RUN_TICKS = 40;
    /** How long the forced chunks may take to become entity-ticking, ticks. */
    static final int LOAD_TICKS = 600;
    /** How far past the stop line a held hull may reach: one tick at 60 blocks/s (3) and a block for Sable's substeps. */
    static final double ALLOWED_DEPTH = 4.0;

    private SableCubeBorderGameTests() {}

    /** A hull's run: where it is assembled, the way it is launched, what it did. */
    private static final class Run {
        final String name;
        final BlockPos anchor;
        final double ux, uz;    // launch direction (unit), or 0, 0 at rest
        UUID id;
        Vec3 start;
        double maxDepth;
        /** The last way out the border gave it (from {@link CubeBorder#outward}), or null. */
        double[] lastOut;
        int pushes;
        Vec3 lastPos;
        long lastSeen = -1;

        Run(String name, BlockPos anchor, double ux, double uz) {
            this.name = name;
            this.anchor = anchor;
            this.ux = ux;
            this.uz = uz;
        }

        boolean moving() {
            return ux != 0 || uz != 0;
        }
    }

    /** Hull blocks: a 3 × 3 deck and a mast, 10 blocks (P0-06's jet). */
    private static List<BlockPos> hull(BlockPos anchor) {
        List<BlockPos> hull = new ArrayList<>();
        for (int dx = -1; dx <= 1; dx++)
            for (int dz = -1; dz <= 1; dz++)
                hull.add(anchor.offset(dx, 0, dz));
        hull.add(anchor.above());
        return hull;
    }

    /** The footprint {@code SableBorder.footprint} reads: the bounds' four corners, or the pose when the bounds look unset. */
    static double[] footprint(AABB b, Vec3 p) {
        if (b == null || !(b.maxX >= b.minX) || !(b.maxZ >= b.minZ) || b.maxX - b.minX > 4096 || b.maxZ - b.minZ > 4096
                || p.x < b.minX - 1 || p.x > b.maxX + 1 || p.z < b.minZ - 1 || p.z > b.maxZ + 1)
            return new double[]{p.x, p.z};
        return new double[]{b.minX, b.minZ, b.maxX, b.minZ, b.minX, b.maxZ, b.maxX, b.maxZ};
    }

    /**
     * Four hulls launched outward at 60 blocks/s from 12 blocks inside the stop line: across N's east
     * segment (+x), F1's top segment (−z), S's bottom segment (+z), and diagonally from F0 into the
     * void square by its north-east corner, where two segments end. Each is held: its footprint is
     * never more than {@value #ALLOWED_DEPTH} blocks outside A(inset) (the planet the inset inside
     * every segment), and after {@value #RUN_TICKS} ticks it is back inside, moving away from the
     * segment that last held it (the corner's hull meets F1's top segment and slides on east along
     * it, as a player would along a wall). A fifth hull at rest in N's warning band, 20 blocks inside
     * the segment, is not moved in x or z.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void sableHullStoppedAtCubeBorder(GameTestHelper helper) {
        if (!ModList.get().isLoaded(SableTestOps.MOD_ID)) {
            helper.fail("Sable is not loaded on this server");
            return;
        }
        ServerLevel level = helper.getLevel();
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || g.cubeLayout() == null) {
            helper.fail("the test level is not a cube world (run on gametest-planet.sh --cube-4k)");
            return;
        }
        EdgeBorder.CubeStop stop = EdgeBorder.cubeStop(level);
        if (stop == null) {
            helper.fail("the cube border is not in force on this level");
            return;
        }
        CubeBorder border = stop.border();
        double inset = stop.inset();
        int s = (int) g.cubeLayout().side(), h = s / 2;
        int y = level.getMaxBuildHeight() - 24;
        double r = Math.sqrt(0.5);
        Map<String, Run> runs = new LinkedHashMap<>();
        runs.put("N east", new Run("N east", new BlockPos(h - 12, y, -s), 1, 0));
        runs.put("F1 top", new Run("F1 top", new BlockPos(s, y, -h + 12), 0, -1));
        runs.put("S bottom", new Run("S bottom", new BlockPos(0, y, 3 * h - 12), 0, 1));
        runs.put("corner", new Run("corner", new BlockPos(h - 14, y, -h + 14), r, -r));
        runs.put("warned", new Run("warned", new BlockPos(h - 20, y, -s + 40), 0, 0));

        // The chunks round each hull's flight: from its anchor to where the launch would carry it
        // unstopped (a hull held at a segment's end slides on along the other side, as the corner's does).
        double reach = SPEED * RUN_TICKS / 20.0;
        List<long[]> forced = new ArrayList<>();
        List<BlockPos> spots = new ArrayList<>();
        for (Run run : runs.values()) {
            BlockPos a = run.anchor;
            BlockPos far = a.offset((int) Math.round(run.ux * reach), 0, (int) Math.round(run.uz * reach));
            for (int cx = (Math.min(a.getX(), far.getX()) - 24) >> 4; cx <= (Math.max(a.getX(), far.getX()) + 24) >> 4; cx++)
                for (int cz = (Math.min(a.getZ(), far.getZ()) - 24) >> 4; cz <= (Math.max(a.getZ(), far.getZ()) + 24) >> 4; cz++) {
                    level.setChunkForced(cx, cz, true);
                    forced.add(new long[]{cx, cz});
                }
            spots.add(a);
            spots.add(a.offset((int) Math.round(run.ux * 20), 0, (int) Math.round(run.uz * 20)));
            spots.add(far);
        }
        long[] startedAt = {-1};
        boolean[] done = {false};
        Runnable cleanUp = () -> {
            for (Run run : runs.values())
                if (run.id != null)
                    SableTestOps.remove(level, run.id);
            for (long[] f : forced)
                level.setChunkForced((int) f[0], (int) f[1], false);
        };
        helper.onEachTick(() -> {
            if (done[0])
                return;
            long t = helper.getTick();
            if (startedAt[0] < 0) {
                boolean ready = true;
                for (BlockPos p : spots)
                    ready &= level.isPositionEntityTicking(p);
                if (!ready) {
                    if (t > LOAD_TICKS) {
                        done[0] = true;
                        cleanUp.run();
                        helper.fail("the border strips never became entity-ticking");
                    }
                    return;
                }
                for (Run run : runs.values()) {
                    List<BlockPos> blocks = hull(run.anchor);
                    for (BlockPos p : blocks)
                        level.setBlock(p, Blocks.OAK_PLANKS.defaultBlockState(), 3);
                    run.id = SableTestOps.assemble(level, run.anchor, blocks);
                    run.start = SableTestOps.position(level, run.id);
                    if (run.moving()) {
                        SableTestOps.addVelocity(level, run.id, new Vec3(SPEED * run.ux, 0, SPEED * run.uz));
                        run.pushes = 1;
                    }
                }
                startedAt[0] = t;
                return;
            }
            long ran = t - startedAt[0];
            for (Run run : runs.values()) {
                Vec3 pos = SableTestOps.position(level, run.id);
                if (pos == null)
                    continue;
                run.lastPos = pos;
                run.lastSeen = ran;
                Vec3 v = SableTestOps.velocity(level, run.id);
                // As in jetCrossesSeam: if the body joined the physics pipeline after the first push, push once more.
                if (run.moving() && ran == 1 && run.pushes < 2 && v != null && v.x * run.ux + v.z * run.uz < SPEED / 2) {
                    double along = v.x * run.ux + v.z * run.uz;
                    SableTestOps.addVelocity(level, run.id, new Vec3((SPEED - along) * run.ux, 0, (SPEED - along) * run.uz));
                    run.pushes++;
                }
                double[] out = border.outward(footprint(SableTestOps.bounds(level, run.id), pos), inset);
                if (out != null) {
                    run.maxDepth = Math.max(run.maxDepth, out[2]);
                    run.lastOut = out;
                }
            }
            if (ran < RUN_TICKS)
                return;
            done[0] = true;
            try {
                List<String> problems = new ArrayList<>();
                StringBuilder d = new StringBuilder();
                for (Run run : runs.values()) {
                    Vec3 pos = SableTestOps.position(level, run.id);
                    Vec3 v = SableTestOps.velocity(level, run.id);
                    if (pos == null || run.start == null) {
                        problems.add(run.name + ": the hull vanished (last seen at tick " + run.lastSeen + " at " + run.lastPos + ")");
                        continue;
                    }
                    double[] end = border.outward(footprint(SableTestOps.bounds(level, run.id), pos), inset);
                    // Outward speed across the segment that last held it (the launch direction before that).
                    double nx = run.lastOut != null ? run.lastOut[0] : run.ux, nz = run.lastOut != null ? run.lastOut[1] : run.uz;
                    double along = v == null ? Double.NaN : v.x * nx + v.z * nz;
                    d.append(String.format(Locale.ROOT, " %s: start=(%.2f,%.2f) end=(%.2f,%.2f) v=(%.3f,%.3f) maxDepth=%.3f endDepth=%s lastOut=(%.3f,%.3f) vOut=%.3f pushes=%d;",
                            run.name, run.start.x, run.start.z, pos.x, pos.z, v == null ? Double.NaN : v.x, v == null ? Double.NaN : v.z,
                            run.maxDepth, end == null ? "0" : String.format(Locale.ROOT, "%.3f", end[2]), nx, nz, along, run.pushes));
                    if (run.moving()) {
                        if (!(run.maxDepth > 0))
                            problems.add(run.name + ": never reached the stop line");
                        if (run.maxDepth > ALLOWED_DEPTH)
                            problems.add(run.name + ": reached " + run.maxDepth + " blocks past the stop line (allowed " + ALLOWED_DEPTH + ")");
                        if (end != null)
                            problems.add(run.name + ": still " + end[2] + " blocks past the stop line after " + RUN_TICKS + " ticks");
                        if (!(along < 0))
                            problems.add(run.name + ": not moving back from the border (outward speed " + along + ")");
                    } else if (Math.hypot(pos.x - run.start.x, pos.z - run.start.z) > 0.5 || run.maxDepth > 0) {
                        problems.add(run.name + ": the hull at rest inside the stop line was moved: " + pos);
                    }
                }
                DeepTime.LOGGER.info("[gametest] sableHullStoppedAtCubeBorder s={} inset={} ticks={} speed={}{}", s, inset, RUN_TICKS, SPEED, d);
                if (!problems.isEmpty())
                    helper.fail("sableHullStoppedAtCubeBorder: " + String.join("; ", problems));
                helper.succeed();
            } finally {
                cleanUp.run();
            }
        });
    }
}
