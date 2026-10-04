// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/**
 * A microbenchmark of {@code Level.getBlockState}, the busiest call in the game, with and without the
 * wrap engine (docs/design/wrap-engine.md §5). Runs inside one GameTest call on the server thread:
 * 16,384 fixed positions in a 2 × 2-chunk patch of varied blocks. The modes are interleaved: four
 * passes, each running every mode for 2 warm-up rounds and 5 measured rounds of 4,194,304 reads, so
 * JIT order effects and the shared VM's load fall on every mode alike; the median over the 20 measured
 * rounds of each mode and its interquartile range in nanoseconds per read are logged on
 * {@code [gametest] wrapbench} lines.
 *
 * <p>Modes: with the engine off ({@code ./gametest.sh -PwrapBench}) only {@code baseline}; with it on
 * ({@code -PwrapEngine}) {@code woven-no-topology} (the mixins woven, no topology on the level),
 * {@code topology-away} (a topology whose bands are far away: the grid test rejects at its bounding
 * box), {@code topology-interior} (a whole-world Guyou layout with the patch deep inside: the grid's
 * cell says no band), {@code band} (every read is in a band and folds: the slow path, a fold, a read
 * at the stored position and a turn), then {@code woven-no-topology} again. The topology is swapped in
 * for the read path only and put back within the call, so the functional tests' sites are not
 * disturbed.
 *
 * <p>Namespace {@code deeptime_wrapbench}, enabled by {@code -PwrapBench} or {@code -PwrapEngine}.
 */
@GameTestHolder(DeepTime.MODID)
@PrefixGameTestTemplate(false)
public final class WrapBenchGameTests {

    static final String NS = "deeptime_wrapbench";
    static final int X0 = 40960, Z0 = 8192, Y_LO = 90, Y_HI = 110;
    static final int N = 1 << 14, CALLS = 1 << 22, WARMUP = 2, ROUNDS = 5, PASSES = 4;
    /** Defeats dead-code elimination. */
    static volatile long sink;

    private WrapBenchGameTests() {}

    @GameTest(templateNamespace = NS, template = "empty_5x5x5", timeoutTicks = 2000, batch = "wrapbench")
    public static void getBlockStateBenchmark(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        // The patch (2 × 2 chunks at X0, Z0) and, north of it, its mirror: the stored side of a half-turn
        // seam whose band is the patch.
        for (int cx = X0 >> 4; cx < (X0 >> 4) + 2; cx++)
            for (int cz = (Z0 >> 4) - 2; cz < (Z0 >> 4) + 2; cz++) {
                level.setChunkForced(cx, cz, true);
                level.getChunk(cx, cz);
            }
        Random r = new Random(1234);
        BlockState[] palette = {Blocks.STONE.defaultBlockState(), Blocks.DIRT.defaultBlockState(), Blocks.AIR.defaultBlockState(),
                Blocks.OAK_STAIRS.defaultBlockState(), Blocks.GLASS.defaultBlockState(), Blocks.OAK_LOG.defaultBlockState()};
        for (int x = X0; x < X0 + 32; x++)
            for (int z = Z0 - 32; z < Z0 + 32; z++)
                for (int y = Y_LO; y < Y_HI; y++)
                    level.setBlock(new BlockPos(x, y, z), palette[r.nextInt(palette.length)], Block.UPDATE_CLIENTS | Block.UPDATE_KNOWN_SHAPE);
        BlockPos[] positions = new BlockPos[N];
        for (int i = 0; i < N; i++)
            positions[i] = new BlockPos(X0 + r.nextInt(32), Y_LO + r.nextInt(Y_HI - Y_LO), Z0 + r.nextInt(32));

        List<String> lines = new ArrayList<>();
        if (!WrapEngine.enabled()) {
            List<Double> base = new ArrayList<>();
            for (int pass = 0; pass < PASSES; pass++)
                measure(level, positions, base);
            lines.add(report("baseline (engine off)", base));
        } else {
            WrapTopology away = WrapEngineGameTests.TOPOLOGY;
            // Guyou with h = 65,536: the patch lies 24,576 blocks inside its rectangle.
            WrapTopology interior = WrapLayouts.guyou(65536, 32, true);
            // A half-turn seam about (X0 + 16, Z0): its band (z in [Z0, Z0 + 32)) is the patch, its stored side north of it.
            WrapTopology band = new WrapTopology("bench seam", List.of(new WrapEdge("bench", false, false, Z0, X0 - 32, X0 + 64, 32,
                    CellIsometry.halfTurnAbout(X0 + 16, Z0))));
            check(!interior.mayFold(X0 + 5, Z0 + 5) && band.fold(X0 + 5, Z0 + 5) != null && !away.mayFold(X0 + 5, Z0 + 5), "bench topologies set up");
            String[] names = {"woven-no-topology", "topology-away", "topology-interior", "band (every read folds)"};
            WrapTopology[] tops = {null, away, interior, band};
            List<List<Double>> ns = new ArrayList<>();
            for (int m = 0; m < names.length; m++)
                ns.add(new ArrayList<>());
            WrapTopology saved = WrapEngine.swapReadTopologyForBenchmark(level, null);
            try {
                for (int pass = 0; pass < PASSES; pass++)
                    for (int m = 0; m < names.length; m++) {
                        WrapEngine.swapReadTopologyForBenchmark(level, tops[m]);
                        measure(level, positions, ns.get(m));
                    }
            } finally {
                WrapEngine.swapReadTopologyForBenchmark(level, saved);
            }
            for (int m = 0; m < names.length; m++)
                lines.add(report(names[m], ns.get(m)));
        }
        for (String l : lines)
            DeepTime.LOGGER.info("[gametest] wrapbench {}", l);
        for (int cx = X0 >> 4; cx < (X0 >> 4) + 2; cx++)
            for (int cz = (Z0 >> 4) - 2; cz < (Z0 >> 4) + 2; cz++)
                level.setChunkForced(cx, cz, false);
        helper.succeed();
    }

    private static void check(boolean ok, String what) {
        if (!ok)
            throw new net.minecraft.gametest.framework.GameTestAssertException("wrapbench: " + what);
    }

    private static void measure(ServerLevel level, BlockPos[] positions, List<Double> out) {
        for (int i = 0; i < WARMUP; i++)
            round(level, positions);
        for (int i = 0; i < ROUNDS; i++)
            out.add(round(level, positions));
    }

    private static String report(String mode, List<Double> ns) {
        double[] sorted = ns.stream().mapToDouble(Double::doubleValue).sorted().toArray();
        int n = sorted.length;
        return String.format("mode=%s ns/call median=%.2f p25=%.2f p75=%.2f min=%.2f (%d rounds of %d reads in %d interleaved passes, %d positions)", mode,
                sorted[n / 2], sorted[n / 4], sorted[(3 * n) / 4], sorted[0], n, CALLS, PASSES, N);
    }

    private static double round(ServerLevel level, BlockPos[] positions) {
        long acc = 0;
        long t0 = System.nanoTime();
        for (int i = 0; i < CALLS; i++)
            if (level.getBlockState(positions[i & (N - 1)]).isAir())
                acc++;
        long dt = System.nanoTime() - t0;
        sink += acc;
        return dt / (double) CALLS;
    }
}
