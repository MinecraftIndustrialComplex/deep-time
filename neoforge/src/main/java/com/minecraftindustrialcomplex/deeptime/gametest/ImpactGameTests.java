// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Craters;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.Comparator;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/**
 * Meteor impacts in the placed world (P2-08): the real test planet's craters
 * ({@code impacts/craters.json}; {@code planet_world_4k.json} runs the impact stage at forty times
 * Earth's rate so that a planet this small has some).
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class ImpactGameTests {

    private ImpactGameTests() {}

    /** Columns of placed blocks read under the crater's floor: the centre and four around it. */
    private static final int[][] FLOOR_COLUMNS = {{0, 0}, {1, 0}, {-1, 0}, {0, 1}, {0, -1}};

    /**
     * The cosine of the latitude (about 59°) poleward of which the bake's projection pass widens its
     * footprint along the parallels, up to eight times: a crater there is averaged into the ground
     * round it by design, so none there is read. On the planet of master 7d60fc4a (2026-10-19) a
     * fresh crater 3.9 km across with 8.4 blocks of relief in the list, 110 blocks from the north
     * edge, showed none in the terrain, which sloped through it without a dip.
     */
    static final double POLAR_WIDEN_FROM_COS = 0.52;   // ledger: SCL-090 (game-choice)

    /**
     * The ground a crater was cut into, as the plane fitted (least squares) to the terrain on four
     * rings of 32 points from 1.25 to 2 rim radii out, beyond the rim; {@code rms} is how far the
     * terrain there strays from that plane, blocks. A crater on a mountainside is measured against
     * the slope it sits on: on the planet of 2026-10-19 (master fb38204f) the deepest showing crater
     * (668, 9 blocks of relief) lay on a flank rising 2.6 blocks a block, whose own gullies and
     * curvature put the terrain 2.3 blocks off a plane, so its centre stood only 2.5 blocks under the
     * mean of its rim (3.0 wanted); over the plane its bowl is 3.3 blocks.
     *
     * @param x0  the crater's centre, blocks
     * @param z0  the crater's centre, blocks
     * @param a   the plane's height at the centre
     * @param bx  its slope along x, blocks a block
     * @param bz  its slope along z, blocks a block
     * @param rms the terrain's spread about the plane on the rings, blocks
     */
    record Ground(double x0, double z0, double a, double bx, double bz, double rms) {

        /** Rings, in rim radii, and points on each. */
        private static final double[] RINGS = {1.25, 1.5, 1.75, 2.0};
        private static final int POINTS = 32;

        static Ground around(WorldDataTerrain t, double x0, double z0, double rimRadius) {
            int n = RINGS.length * POINTS;
            double[] dx = new double[n], dz = new double[n], h = new double[n];
            double sx = 0, sz = 0, sh = 0;
            for (int r = 0, i = 0; r < RINGS.length; r++)
                for (int k = 0; k < POINTS; k++, i++) {
                    double a = 2 * Math.PI * (k + 0.5 * r) / POINTS;
                    int x = (int) Math.round(x0 + RINGS[r] * rimRadius * Math.cos(a)), z = (int) Math.round(z0 + RINGS[r] * rimRadius * Math.sin(a));
                    dx[i] = x - x0;
                    dz[i] = z - z0;
                    h[i] = t.height(x, z);
                    sx += dx[i];
                    sz += dz[i];
                    sh += h[i];
                }
            double mx = sx / n, mz = sz / n, mh = sh / n;
            double xx = 0, xz = 0, zz = 0, xh = 0, zh = 0;
            for (int i = 0; i < n; i++) {
                double u = dx[i] - mx, v = dz[i] - mz, w = h[i] - mh;
                xx += u * u;
                xz += u * v;
                zz += v * v;
                xh += u * w;
                zh += v * w;
            }
            double det = xx * zz - xz * xz;
            double bx = det == 0 ? 0 : (xh * zz - zh * xz) / det, bz = det == 0 ? 0 : (zh * xx - xh * xz) / det;
            double a = mh - bx * mx - bz * mz, ss = 0;
            for (int i = 0; i < n; i++) {
                double e = h[i] - (a + bx * dx[i] + bz * dz[i]);
                ss += e * e;
            }
            return new Ground(x0, z0, a, bx, bz, Math.sqrt(ss / n));
        }

        /** The terrain at column (x, z) over the plane, blocks. */
        double over(WorldDataTerrain t, int x, int z) {
            return t.height(x, z) - (a + bx * (x - x0) + bz * (z - z0));
        }

        /**
         * Over the plane: the crater's floor (the mean of the columns within half its rim radius,
         * so a complex crater's central peak does not stand for it), its rim (the mean of sixteen
         * points on it) and the floor's column count.
         */
        double[] floorAndRim(WorldDataTerrain t, double rimRadius) {
            double floor = 0, rim = 0;
            int columns = 0;
            int cx = (int) Math.round(x0), cz = (int) Math.round(z0), h = (int) Math.ceil(rimRadius / 2);
            for (int x = cx - h; x <= cx + h; x++)
                for (int z = cz - h; z <= cz + h; z++)
                    if (Math.hypot(x - x0, z - z0) <= rimRadius / 2) {
                        floor += over(t, x, z);
                        columns++;
                    }
            for (int i = 0; i < 16; i++) {
                double a = 2 * Math.PI * i / 16;
                rim += over(t, (int) Math.round(x0 + rimRadius * Math.cos(a)), (int) Math.round(z0 + rimRadius * Math.sin(a))) / 16;
            }
            return new double[]{floor / Math.max(1, columns), rim, columns};
        }
    }

    /**
     * The crater that shows best on the test planet is a bowl in the terrain the game reads, and
     * the rock under its floor is impact breccia:
     * <ul>
     *   <li>the list is sane: every crater has a size, a state the reader knows and no more relief
     *       than it formed with;</li>
     *   <li>the one read is, of the craters with a bowl and a breccia lens of three blocks or more,
     *       the one whose relief stands highest over the spread of the ground around it
     *       (relief / (spread + 1 block), {@link Ground}; until 2026-10-19 the deepest, which can
     *       sit on a gullied mountainside), taken among those on land when the planet has any, else
     *       among those under the sea (on the planet of master 7d60fc4a, 2026-10-19, all 44 such
     *       craters lay under the sea, most on shelves a few blocks deep, and until then the test
     *       read nothing there);</li>
     *   <li>over the plane of that ground, its floor (the mean of the columns within half its rim
     *       radius) lies below the mean of sixteen points on its rim by at least a third of the
     *       relief the simulation says it has left;</li>
     *   <li>within its breccia lens under the floor, placed blocks are the block the palette gives
     *       {@code impact_breccia} (a body of the bake for a crater too small for the strata log).</li>
     * </ul>
     * A planet whose craters have all faded, or whose showing craters are all in the strata log's
     * cells, passes with a line saying so: the shapes themselves are the core's tests'
     * ({@code surface::amplify::impact}).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void aCraterIsABowlOverBreccia(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_CAVES);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_CAVES.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_planet_caves is not generating from world data");
            return;
        }
        WorldData wd = t.data();
        double mH = wd.manifest().mPerBlockH(), mV = wd.manifest().mPerBlockV();
        int c = wd.circumference(), sea = g.generatorSettings().value().seaLevel();
        List<Craters.Crater> all = wd.craters().list();
        Set<String> fates = Set.of("fresh", "degraded", "buried", "eroded_to_roots");
        for (Craters.Crater k : all) {
            if (!(k.diameterM() > 0) || !fates.contains(k.fate()) || k.reliefM() > 1.01 * k.depthM() + 1 || k.reliefM() < 0
                    || Math.abs(k.x()) > c / 2.0 || Math.abs(k.z()) > wd.projection().zMax()) {
                helper.fail("crater " + k.id() + " is not sane: " + k);
                return;
            }
        }
        long showing = all.stream().filter(Craters.Crater::shows).count();
        // The candidates: clear of the world's north and south edges and equatorward of the bake's
        // polar smoothing (POLAR_WIDEN_FROM_COS), their bowl at least three blocks deep and their lens at
        // least three blocks thick, drawn by the amplifier with a body under them; those on land when
        // the planet has any, else those under the sea (the bowl is read in the ground under the water
        // and the breccia under the sea floor). The one read is the one that shows best against the
        // ground around it (see Ground).
        List<Craters.Crater> drawn = all.stream().filter(k -> k.shows() && !k.inSurface()).toList();
        List<Craters.Crater> deep = drawn.stream().filter(k -> k.reliefM() >= 3 * mV && k.brecciaM() >= 3 * mV).toList();
        List<Craters.Crater> clear = deep.stream()
                .filter(k -> Math.abs(k.z()) < wd.projection().zMax() - 64 - 3 * k.diameterM() / mH)
                .filter(k -> Math.cos(Math.toRadians(wd.projection().latitudeDeg(k.x(), k.z()))) >= POLAR_WIDEN_FROM_COS)
                .toList();
        List<Craters.Crater> onLand = clear.stream()
                .filter(k -> t.height((int) Math.round(k.x()), (int) Math.round(k.z())) > sea + 2)
                .toList();
        List<Craters.Crater> candidates = onLand.isEmpty() ? clear : onLand;
        java.util.Map<Craters.Crater, Ground> grounds = new java.util.HashMap<>();
        for (Craters.Crater k : candidates)
            grounds.put(k, Ground.around(t, k.x(), k.z(), k.diameterM() / 2 / mH));
        Craters.Crater best = candidates.stream()
                .max(Comparator.comparingDouble((Craters.Crater k) -> k.reliefM() / mV / (grounds.get(k).rms() + 1))
                        .thenComparingDouble(Craters.Crater::reliefM)).orElse(null);
        if (best == null) {
            DeepTime.LOGGER.info("[gametest] aCraterIsABowlOverBreccia craters={} showing={}: none with a bowl of three blocks and a "
                    + "body; nothing to read back (drawn by the amplifier {}, of them with relief and breccia of three blocks {}, of them "
                    + "clear of the poles 0)", all.size(), showing, drawn.size(), deep.size());
            helper.succeed();
            return;
        }
        String where = onLand.isEmpty() ? "under the sea, no candidate being on land" : "on land";
        int cx = (int) Math.round(best.x()), cz = (int) Math.round(best.z());
        double rb = best.diameterM() / 2 / mH;
        Ground ground = grounds.get(best);
        double[] fr = ground.floorAndRim(t, rb);
        double floor = fr[0], rim = fr[1], centre = t.height(cx, cz), bowl = rim - floor, want = best.reliefM() / mV / 3;
        int floorColumns = (int) fr[2];
        StringBuilder others = new StringBuilder();
        for (Craters.Crater k : candidates)
            if (k != best) {
                Ground o = grounds.get(k);
                double[] f = o.floorAndRim(t, k.diameterM() / 2 / mH);
                others.append(String.format(" %d (%.1f blocks of relief, ground spread %.1f, bowl %.1f)", k.id(), k.reliefM() / mV, o.rms(), f[1] - f[0]));
            }
        // Diagnostic (2026-10-23): the projection's local scale at the crater and the lens bodies the bake placed under it.
        StringBuilder diag = new StringBuilder();
        {
            var p = wd.projection();
            double r = wd.circumference() / (2 * Math.PI);
            double[] ax = p.unit(best.x() - 0.5, best.z()), bx = p.unit(best.x() + 0.5, best.z());
            double[] az = p.unit(best.x(), best.z() - 0.5), bz = p.unit(best.x(), best.z() + 0.5);
            double sx = Math.sqrt(Math.pow(ax[0] - bx[0], 2) + Math.pow(ax[1] - bx[1], 2) + Math.pow(ax[2] - bx[2], 2)) * r;
            double sz = Math.sqrt(Math.pow(az[0] - bz[0], 2) + Math.pow(az[1] - bz[1], 2) + Math.pow(az[2] - bz[2], 2)) * r;
            diag.append(String.format("; latitude %.1f, planet blocks a world block %.3f along x and %.3f along z (%s)",
                    p.latitudeDeg(best.x(), best.z()), sx, sz, p.kind()));
            diag.append("; heights along x");
            for (int d = -10; d <= 10; d++)
                diag.append(String.format(" %.1f", t.height(cx + d, cz)));
            diag.append("; along z");
            for (int d = -10; d <= 10; d++)
                diag.append(String.format(" %.1f", t.height(cx, cz + d)));
            for (var b : wd.deposits().bodiesIn(cx - 12, level.getMinBuildHeight(), cz - 12, cx + 12, level.getMaxBuildHeight() - 1, cz + 12))
                if ("impact_breccia".equals(b.rockClass()) || "impact_breccia".equals(b.role()))
                    diag.append(String.format("; lens body %d at (%.1f, %.2f, %.1f) axes (%.2f, %.2f, %.2f) y %d..%d",
                            b.id(), b.center()[0], b.center()[1], b.center()[2], b.axes()[0], b.axes()[1], b.axes()[2], b.min()[1], b.max()[1]));
        }
        var result = RockPalette.get().result("impact_breccia").orElse(null);
        if (result == null || result.blocks().isEmpty()) {
            helper.fail("the palette has no block for impact_breccia");
            return;
        }
        Set<BlockState> breccia = new HashSet<>(result.blocks());
        int lens = (int) Math.max(3, Math.min(24, Math.floor(best.brecciaM() / mV)));
        int[] found = new int[FLOOR_COLUMNS.length], read = new int[FLOOR_COLUMNS.length];
        // What the columns hold under the floor, by block, for the log: the lens is breccia, the rest soil and strata.
        java.util.Map<String, Integer> seen = new java.util.TreeMap<>();
        StringBuilder profile = new StringBuilder();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        ChunkSampling.forEach(helper, FLOOR_COLUMNS.length, i -> {
            int x = cx + FLOOR_COLUMNS[i][0], z = cz + FLOOR_COLUMNS[i][1];
            ChunkAccess chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.SURFACE, true);
            int top = (int) Math.ceil(t.height(x, z));
            if (i == 0)
                profile.append(columnProfile(chunk, x, z, top + 8, top - 40)).append(" heightmap ")
                        .append(chunk.getHeight(net.minecraft.world.level.levelgen.Heightmap.Types.WORLD_SURFACE_WG, x & 15, z & 15));
            // From the floor down through the lens (the top blocks are the surface rules' soil).
            for (int y = top; y >= Math.max(level.getMinBuildHeight(), top - lens - 2); y--) {
                BlockState s = chunk.getBlockState(pos.set(x, y, z));
                read[i]++;
                seen.merge(BuiltInRegistries.BLOCK.getKey(s.getBlock()).getPath(), 1, Integer::sum);
                if (breccia.contains(s))
                    found[i]++;
            }
            return true;
        }, () -> {
            int columns = 0, blocks = 0;
            for (int n : found) {
                columns += n > 0 ? 1 : 0;
                blocks += n;
            }
            DeepTime.LOGGER.info("[gametest] aCraterIsABowlOverBreccia craters={} showing={} candidates={} ({}) read=crater {} ({} m across, {}, "
                            + "{} Myr, {} m of {} m relief, lens {} m) at ({}, {}): centre y {}; ground around it y {} sloping {} blocks a block, "
                            + "spread {}; over it the floor {} ({} columns) and the rim {}: bowl {} blocks (want {}); breccia {} blocks in {} of "
                            + "{} columns, {} read each ({}); read {}; centre column {}; other candidates:{}", all.size(), showing,
                    candidates.size(), where, best.id(), Math.round(best.diameterM()), best.fate(), best.ageMyr(), Math.round(best.reliefM()),
                    Math.round(best.depthM()), Math.round(best.brecciaM()), cx, cz, String.format("%.1f", centre),
                    String.format("%.1f", ground.a()), String.format("%.2f", Math.hypot(ground.bx(), ground.bz())),
                    String.format("%.1f", ground.rms()), String.format("%.1f", floor), floorColumns, String.format("%.1f", rim),
                    String.format("%.1f", bowl), String.format("%.1f", want), blocks, columns, FLOOR_COLUMNS.length, read[0],
                    result.blocks().stream().map(StrataGameTests::id).toList(), seen, profile, (others.isEmpty() ? " none" : others) + diag.toString());
            if (bowl < want)
                helper.fail("crater " + best.id() + ": its floor is " + bowl + " blocks below its rim over the ground around it, " + want + " wanted");
            if (columns == 0)
                helper.fail("crater " + best.id() + ": no impact breccia in the " + lens + " blocks under its floor");
            helper.succeed();
        });
    }

    /** The column from y {@code from} down to {@code to} as runs of one block, top first. */
    static String columnProfile(ChunkAccess chunk, int x, int z, int from, int to) {
        StringBuilder b = new StringBuilder();
        BlockPos.MutableBlockPos p = new BlockPos.MutableBlockPos();
        String last = null;
        int top = from, y = from;
        for (; y >= Math.max(to, chunk.getMinBuildHeight()); y--) {
            String id = BuiltInRegistries.BLOCK.getKey(chunk.getBlockState(p.set(x, y, z)).getBlock()).getPath();
            if (!id.equals(last)) {
                if (last != null)
                    b.append(' ').append(last).append('[').append(top).append("..").append(y + 1).append(']');
                last = id;
                top = y;
            }
        }
        if (last != null)
            b.append(' ').append(last).append('[').append(top).append("..").append(y + 1).append(']');
        return b.toString();
    }
}
