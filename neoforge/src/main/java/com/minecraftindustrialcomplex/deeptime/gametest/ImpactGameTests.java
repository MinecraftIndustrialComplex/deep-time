// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Craters;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
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

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
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
     * How far the bake's projection pass may average a crater along its parallel for the crater to be
     * read, as a share of its rim radius: the footprint's half-width beyond the bilinear read
     * ({@link #footprintAlongParallelM}) at most half the radius, so the floor the test reads (the
     * columns within half the rim radius) is not averaged with the rim. Equatorward of
     * {@link #POLAR_WIDEN_FROM_COS} too the footprint is round on the planet at the source grid's
     * north–south spacing or wider, and a crater the simulation drew round in its Lambert blocks is
     * narrow east–west there, so a small crater at a high latitude is averaged into its walls: on the
     * planet of master 4ab29084 (2026-10-23) crater 572 (2.6 km across at 57°, 7.8 blocks of relief)
     * was averaged over 0.56 of its radius and its floor stood 2.4 blocks under its rim.
     */
    static final double FOOTPRINT_SHARE = 0.5;   // game-choice (agent, 2026-10-23): the test's tolerance; the footprint itself is ledger SCL-090

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

        static Ground around(WorldDataTerrain t, InLambert s, double rimRadius) {
            double x0 = s.x0(), z0 = s.z0();
            int n = RINGS.length * POINTS;
            double[] dx = new double[n], dz = new double[n], h = new double[n];
            double sx = 0, sz = 0, sh = 0;
            for (int r = 0, i = 0; r < RINGS.length; r++)
                for (int k = 0; k < POINTS; k++, i++) {
                    double[] q = s.point(RINGS[r] * rimRadius, 2 * Math.PI * (k + 0.5 * r) / POINTS);
                    int x = (int) Math.round(q[0]), z = (int) Math.round(q[1]);
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
        double[] floorAndRim(WorldDataTerrain t, InLambert s, double rimRadius) {
            double floor = 0, rim = 0;
            int columns = 0;
            for (int[] q : s.within(rimRadius / 2)) {
                floor += over(t, q[0], q[1]);
                columns++;
            }
            for (int i = 0; i < 16; i++) {
                double[] q = s.point(rimRadius, 2 * Math.PI * i / 16);
                rim += over(t, (int) Math.round(q[0]), (int) Math.round(q[1])) / 16;
            }
            return new double[]{floor / Math.max(1, columns), rim, columns};
        }
    }

    /**
     * A crater's surroundings measured in the frame the simulation draws it in, its Lambert cylinder
     * (the amplifier's crater shape, the impact bodies and their footprints all work in Lambert
     * blocks), where a crater is round. Lengths are in the world's blocks of the equal-area sphere
     * ({@code WorldProjection.radius()}), so a crater {@code D} metres across has a rim radius of
     * {@code D / 2 / mPerBlockH} of them: a distance is {@code radius · hypot(Δλ, Δ sin φ)}, and a
     * bearing is an angle from east toward north. On the planet such a crater is long north–south and
     * narrow east–west, by 1 / cos² of its latitude (seamless-world.md, "Simplifications and limits"),
     * and on the Petroff–Guyou world it is a further ellipse in world blocks. Read as a circle in world
     * blocks (until 2026-10-23) or on the sphere, a crater far from the equator has its rim taken on
     * its walls one way and beyond it the other: on the planet of master 4ab29084 crater 537 (8.9 km
     * across at 55° south) showed a bowl of 1.5 blocks measured round on the sphere and 3.3 in this
     * frame.
     *
     * @param x0 the centre, world blocks
     * @param z0 the centre, world blocks
     */
    record InLambert(WorldProjection p, double x0, double z0, double lat0, double lon0) {

        static InLambert at(WorldProjection p, double x0, double z0) {
            double[] ll = p.latLon(x0, z0);
            return new InLambert(p, x0, z0, ll[0], ll[1]);
        }

        /** The distance from the centre to the point (x, z) on the cylinder, blocks. */
        double distance(double x, double z) {
            double[] ll = p.latLon(x, z);
            double dl = Math.toRadians(ll[1] - lon0);
            dl -= 2 * Math.PI * Math.floor(dl / (2 * Math.PI) + 0.5);
            return p.radius() * Math.hypot(dl, Math.sin(Math.toRadians(ll[0])) - Math.sin(Math.toRadians(lat0)));
        }

        /** The world point {x, z} at {@code s} blocks from the centre on bearing {@code a} (radians from east toward north); x unwrapped about the centre. */
        double[] point(double s, double a) {
            double lon = lon0 + Math.toDegrees(s / p.radius() * Math.cos(a));
            double sl = Math.max(-1, Math.min(1, Math.sin(Math.toRadians(lat0)) + s / p.radius() * Math.sin(a)));
            double[] q = p.blockOf(Math.toDegrees(Math.asin(sl)), ((lon % 360) + 360) % 360);
            return new double[]{x0 + p.wrapX(q[0] - x0), q[1]};
        }

        /** The columns {x, z} within {@code s} blocks of the centre (x unwrapped about it). */
        List<int[]> within(double s) {
            double xLo = x0, xHi = x0, zLo = z0, zHi = z0;
            for (int i = 0; i < 32; i++) {
                double[] q = point(s, 2 * Math.PI * i / 32);
                xLo = Math.min(xLo, q[0]);
                xHi = Math.max(xHi, q[0]);
                zLo = Math.min(zLo, q[1]);
                zHi = Math.max(zHi, q[1]);
            }
            List<int[]> out = new ArrayList<>();
            for (int x = (int) Math.floor(xLo) - 1; x <= (int) Math.ceil(xHi) + 1; x++)
                for (int z = (int) Math.floor(zLo) - 1; z <= (int) Math.ceil(zHi) + 1; z++)
                    if (distance(x, z) <= s)
                        out.add(new int[]{x, z});
            return out;
        }
    }

    /**
     * The half-width of the bake's projection footprint at world point (x, z) along the parallel,
     * beyond the bilinear read, in metres of the Lambert cylinder (as a crater's radius is):
     * (d / cos φ − cell) / 2 with d = max(stretch · cell, cell / cos φ), the footprint's diameter on
     * the planet (SCL-090, {@code bake::project::Mapping::footprint}; stretch the largest metres on the
     * planet per metre of world block there). The source grid's cell is taken as the world elevation
     * grid's: the simulation's block is about 6% shorter than the Petroff–Guyou world's, so this is a
     * little wide.
     */
    static double footprintAlongParallelM(WorldData wd, double x, double z) {
        WorldProjection p = wd.projection();
        double cell = wd.manifest().grids().get(wd.manifest().layer(WorldData.ELEVATION).grid()).r() * wd.manifest().mPerBlockH();
        double cos = Math.max(1e-6, Math.cos(Math.toRadians(p.latitudeDeg(x, z))));
        double[] x1 = p.unit(x + 0.25, z), x0 = p.unit(x - 0.25, z), z1 = p.unit(x, z + 0.25), z0 = p.unit(x, z - 0.25);
        double a = 0, b = 0, ab = 0;
        for (int i = 0; i < 3; i++) {
            double ex = (x1[i] - x0[i]) / 0.5, ez = (z1[i] - z0[i]) / 0.5;
            a += ex * ex;
            b += ez * ez;
            ab += ex * ez;
        }
        double stretch = Math.sqrt(0.5 * (a + b) + Math.sqrt(0.25 * (a - b) * (a - b) + ab * ab)) * p.radius();
        double d = Math.max(stretch * cell, cell / cos);
        return Math.max(0, d / cos - cell) / 2;
    }

    /**
     * The crater that shows best on the test planet is a bowl in the terrain the game reads, and
     * the rock under its floor is impact breccia:
     * <ul>
     *   <li>the list is sane: every crater has a size, a state the reader knows and no more relief
     *       than it formed with;</li>
     *   <li>the one read is, of the craters with a bowl and a breccia lens of three blocks or more
     *       that the world can show (clear of its north and south edges, equatorward of the polar
     *       widening, no more averaged by the projection than {@link #FOOTPRINT_SHARE} allows, and
     *       with a third of their relief above the spread of the ground around them), the one whose
     *       relief stands highest over that spread (relief / (spread + 1 block), {@link Ground};
     *       until 2026-10-19 the deepest, which can sit on a gullied mountainside), taken among those
     *       on land when the planet has any, else among those under the sea (on the planet of master
     *       7d60fc4a, 2026-10-19, all 44 such craters lay under the sea, most on shelves a few blocks
     *       deep, and until then the test read nothing there; on that of master 4ab29084, 2026-10-23,
     *       the two on land were 572, too small for its footprint at 57°, and 568, whose 2 blocks
     *       wanted lay in ground spread over 14);</li>
     *   <li>measured round in the simulation's frame ({@link InLambert}), over the plane of that
     *       ground, its floor (the mean of the columns within half its rim radius) lies below the mean
     *       of sixteen points on its rim by at least a third of the relief the simulation says it has
     *       left;</li>
     *   <li>within its breccia lens under the floor, placed blocks are the block the palette gives
     *       {@code impact_breccia} (a body of the bake for a crater too small for the strata log).
     *       The surface rules' soil stays over the lens (a body never replaces it), so a lens
     *       thinner than its soil shows no breccia: 572's lens of 3.7 blocks lay wholly in its 5
     *       blocks of dirt.</li>
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
        // The candidates: their bowl at least three blocks deep and their lens at least three blocks
        // thick, drawn by the amplifier with a body under them; clear of the world's north and south
        // edges and equatorward of the bake's polar smoothing (POLAR_WIDEN_FROM_COS); resolved by the
        // projection's footprint (FOOTPRINT_SHARE); a third of their relief above the ground's spread
        // (a smaller bowl is lost in it); those on land when the planet has any, else those under the
        // sea (the bowl is read in the ground under the water and the breccia under the sea floor).
        // The one read is the one that shows best against the ground around it (see Ground).
        List<Craters.Crater> drawn = all.stream().filter(k -> k.shows() && !k.inSurface()).toList();
        List<Craters.Crater> deep = drawn.stream().filter(k -> k.reliefM() >= 3 * mV && k.brecciaM() >= 3 * mV).toList();
        List<Craters.Crater> clear = deep.stream()
                .filter(k -> Math.abs(k.z()) < wd.projection().zMax() - 64 - 3 * k.diameterM() / mH)
                .filter(k -> Math.cos(Math.toRadians(wd.projection().latitudeDeg(k.x(), k.z()))) >= POLAR_WIDEN_FROM_COS)
                .toList();
        List<Craters.Crater> resolved = clear.stream()
                .filter(k -> footprintAlongParallelM(wd, k.x(), k.z()) <= FOOTPRINT_SHARE * k.diameterM() / 2)
                .toList();
        Map<Craters.Crater, InLambert> frames = new HashMap<>();
        Map<Craters.Crater, Ground> grounds = new HashMap<>();
        for (Craters.Crater k : resolved) {
            frames.put(k, InLambert.at(wd.projection(), k.x(), k.z()));
            grounds.put(k, Ground.around(t, frames.get(k), k.diameterM() / 2 / mH));
        }
        List<Craters.Crater> readable = resolved.stream().filter(k -> k.reliefM() / mV / 3 >= grounds.get(k).rms()).toList();
        List<Craters.Crater> onLand = readable.stream()
                .filter(k -> t.height((int) Math.round(k.x()), (int) Math.round(k.z())) > sea + 2)
                .toList();
        List<Craters.Crater> candidates = onLand.isEmpty() ? readable : onLand;
        String sieve = String.format("drawn by the amplifier %d, of them with relief and breccia of three blocks %d, of them clear of the "
                + "edges and poles %d, of them resolved by the footprint %d, of them standing over their ground %d", drawn.size(), deep.size(),
                clear.size(), resolved.size(), readable.size());
        Craters.Crater best = candidates.stream()
                .max(Comparator.comparingDouble((Craters.Crater k) -> k.reliefM() / mV / (grounds.get(k).rms() + 1))
                        .thenComparingDouble(Craters.Crater::reliefM)).orElse(null);
        if (best == null) {
            DeepTime.LOGGER.info("[gametest] aCraterIsABowlOverBreccia craters={} showing={}: none with a bowl of three blocks and a "
                    + "body that the world can show; nothing to read back ({})", all.size(), showing, sieve);
            helper.succeed();
            return;
        }
        String where = onLand.isEmpty() ? "under the sea, no candidate being on land" : "on land";
        int cx = (int) Math.round(best.x()), cz = (int) Math.round(best.z());
        double rb = best.diameterM() / 2 / mH;
        Ground ground = grounds.get(best);
        double[] fr = ground.floorAndRim(t, frames.get(best), rb);
        double floor = fr[0], rim = fr[1], centre = t.height(cx, cz), bowl = rim - floor, want = best.reliefM() / mV / 3;
        int floorColumns = (int) fr[2];
        String placed = String.format("latitude %.1f°, footprint %.2f of its radius", wd.projection().latitudeDeg(best.x(), best.z()),
                footprintAlongParallelM(wd, best.x(), best.z()) / (best.diameterM() / 2));
        StringBuilder others = new StringBuilder();
        for (Craters.Crater k : candidates)
            if (k != best) {
                Ground o = grounds.get(k);
                double[] f = o.floorAndRim(t, frames.get(k), k.diameterM() / 2 / mH);
                others.append(String.format(" %d (%.1f blocks of relief, ground spread %.1f, bowl %.1f)", k.id(), k.reliefM() / mV, o.rms(), f[1] - f[0]));
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
        Map<String, Integer> seen = new java.util.TreeMap<>();
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
            DeepTime.LOGGER.info("[gametest] aCraterIsABowlOverBreccia craters={} showing={} candidates={} ({}; {}) read=crater {} ({} m "
                            + "across, {}, {} Myr, {} m of {} m relief, lens {} m) at ({}, {}), {}: centre y {}; ground around it y {} "
                            + "sloping {} blocks a block, spread {}; over it the floor {} ({} columns) and the rim {}: bowl {} blocks (want "
                            + "{}); breccia {} blocks in {} of {} columns, {} read each ({}); read {}; centre column {}; other candidates:{}",
                    all.size(), showing, candidates.size(), where, sieve, best.id(), Math.round(best.diameterM()), best.fate(), best.ageMyr(),
                    Math.round(best.reliefM()), Math.round(best.depthM()), Math.round(best.brecciaM()), cx, cz, placed,
                    String.format("%.1f", centre), String.format("%.1f", ground.a()), String.format("%.2f", Math.hypot(ground.bx(), ground.bz())),
                    String.format("%.1f", ground.rms()), String.format("%.1f", floor), floorColumns, String.format("%.1f", rim),
                    String.format("%.1f", bowl), String.format("%.1f", want), blocks, columns, FLOOR_COLUMNS.length, read[0],
                    result.blocks().stream().map(StrataGameTests::id).toList(), seen, profile, others.isEmpty() ? " none" : others);
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
