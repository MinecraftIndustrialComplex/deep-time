// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhFarTerrain;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrain;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrainCheck;
import com.minecraftindustrialcomplex.deeptime.worlddata.FarColumn;
import net.minecraft.core.Holder;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * Far terrain against generated chunks (plan §10.5 P3-04), on the real baked planet
 * ({@code deeptime:test_planet_real}: every pipeline stage, the pack's biomes, surface rules and
 * features). Far terrain is built without a chunk; these tests then generate the chunk and compare
 * the same columns. Distant Horizons is not needed here and no GameTest-server run loads it (it cannot run on the
 * GameTest server); what Distant Horizons stores of these columns is the smoke check's
 * ({@code tools/smoke/run.py --dh}).
 *
 * <p>The thresholds are properties any planet must meet, not one planet's numbers: the ground and
 * the water come from the same column rule the chunk is built by, so they agree but for what
 * features and the pack's surface rules do on top; the top block comes from a table where the part
 * of the pack's surface rule that reads no chunk does not decide it, so its agreement is logged and
 * only a floor is asserted. The ground is read under trees (mangrove roots
 * included), plants and coral reefs, and a column a carver cut below far terrain's ground is counted
 * apart (at most one in eight: on the MIC set Terralith's canyon, which reaches from y −20 to 70, cut
 * a sea bed two blocks under the water down to y −2 in two of the 64 columns east of the seam,
 * 2026-10-03). The planet-wide test reads chunks with their features, the seam test before them (see
 * there).
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class FarTerrainGameTests {

    private FarTerrainGameTests() {}

    static FarTerrain far(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        return FarTerrain.of(level).orElseGet(() -> {
            helper.fail("test_planet_real has no far terrain (it is not generating from world data)");
            throw new IllegalStateException();
        });
    }

    /** A spread of columns over the planet, clear of the pole edge and of the build floor. */
    static int[] column(int i, int c) {
        int x = -c / 2 + 11 + i * (c / 131);
        int z = ((i * 53) % 113 - 56) * ((int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c) / 64) + 7;
        return new int[]{x, z};
    }

    /**
     * Heights, water, the biome and the ground block of far terrain's columns against the generated
     * chunk (features included: the comparison looks under trees and plants).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 4800)
    public static void farTerrainMatchesGeneratedChunks(GameTestHelper helper) {
        FarTerrain far = far(helper);
        ServerLevel level = far.level();
        int c = far.data().circumference();
        FarTerrainCheck.Tally tally = new FarTerrainCheck.Tally();
        ChunkSampling.forEach(helper, 128, i -> {
            int[] p = column(i, c);
            FarTerrain.Column column = far.column(p[0], p[1]);
            if (column.ground() < level.getMinBuildHeight() + 32)
                return false;   // the floor band: the router's bottom slide, not the bake
            ChunkAccess chunk = level.getChunkSource().getChunk(p[0] >> 4, p[1] >> 4, ChunkStatus.FEATURES, true);
            tally.add(column, FarTerrainCheck.real(chunk, p[0], p[1]), chunk);
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] farTerrainMatchesGeneratedChunks {}", tally);
            assertAgrees(helper, tally, 64, "");
            helper.succeed();
        });
    }

    /** One column where far terrain and the chunk differ, for the log: the blocks between the two grounds ({@code *}: in the air carving mask). */
    static String diag(ChunkAccess chunk, FarTerrain.Column far, FarTerrainCheck.RealColumn real, int x, int z) {
        StringBuilder sb = new StringBuilder();
        sb.append("(").append(x).append(", ").append(z).append(") far ").append(far.ground()).append(" real ").append(real.ground())
                .append(" status ").append(chunk.getPersistedStatus()).append(" biome ").append(real.biome().unwrapKey().map(k -> k.location().toString()).orElse("?"));
        net.minecraft.world.level.chunk.CarvingMask air = null, liquid = null;
        if (chunk instanceof net.minecraft.world.level.chunk.ProtoChunk p) {
            air = p.getCarvingMask(net.minecraft.world.level.levelgen.GenerationStep.Carving.AIR);
            liquid = p.getCarvingMask(net.minecraft.world.level.levelgen.GenerationStep.Carving.LIQUID);
        }
        sb.append(" masks ").append(air != null).append('/').append(liquid != null).append(" column:");
        net.minecraft.core.BlockPos.MutableBlockPos pos = new net.minecraft.core.BlockPos.MutableBlockPos();
        String last = null;
        int runTop = 0;
        int lo = Math.min(far.ground(), real.ground()) - 2, hi = Math.max(far.ground(), real.ground()) + 3;
        for (int y = hi; y >= lo; y--) {
            String id = net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(chunk.getBlockState(pos.set(x, y, z)).getBlock()).getPath();
            if (air != null && air.get(x, y, z))
                id += "*";
            if (!id.equals(last)) {
                if (last != null)
                    sb.append(' ').append(last).append('@').append(runTop).append("..").append(y + 1);
                last = id;
                runTop = y;
            }
        }
        sb.append(' ').append(last).append('@').append(runTop).append("..").append(lo);
        return sb.toString();
    }

    /** The properties far terrain must meet against generated chunks (see the class comment). */
    static void assertAgrees(GameTestHelper helper, FarTerrainCheck.Tally tally, int minColumns, String where) {
        helper.assertTrue(tally.columns >= minColumns, where + "only " + tally.columns + " columns compared");
        // Carvers are not far terrain's to draw (FarTerrainCheck.Tally#carved), but they must stay the
        // exception: a generator that let them cut most columns would hide here otherwise.
        helper.assertTrue(8 * tally.carved <= tally.columns + tally.carved,
                where + tally.carved + " of " + (tally.columns + tally.carved) + " columns were cut by a carver: " + tally.carvedAt);
        helper.assertTrue(tally.share(tally.heightWithin1) >= 0.90,
                where + "the ground is within a block in only " + tally.heightWithin1 + " of " + tally.columns + " columns; worst " + tally.worstHeight);
        helper.assertTrue(tally.share(tally.heightWithin2) >= 0.95,
                where + "the ground is within two blocks in only " + tally.heightWithin2 + " of " + tally.columns + " columns; worst " + tally.worstHeight);
        helper.assertTrue(tally.share(tally.waterAgree) >= 0.95,
                where + "water or dry agrees in only " + tally.waterAgree + " of " + tally.columns + " columns");
        helper.assertTrue(tally.waterBoth == 0 || tally.waterTopExact >= 0.9 * tally.waterBoth,
                where + "the water's top agrees in only " + tally.waterTopExact + " of " + tally.waterBoth + " water columns");
        helper.assertTrue(tally.share(tally.biomeSame) >= 0.95,
                where + "the biome agrees in only " + tally.biomeSame + " of " + tally.columns + " columns: " + tally.toJson().get("biome_differences"));
        helper.assertTrue(tally.share(tally.groundBlockSame) >= 0.4,
                where + "the ground block agrees in only " + tally.groundBlockSame + " of " + tally.columns + " columns: "
                        + tally.toJson().get("ground_block_differences"));
    }

    /**
     * Far terrain against generated chunks on both sides of the seam (Ben, 2026-10-02, with the patch
     * that lets this generator through Toroidal World's guard): the 64 blocks west of x = C/2 and the
     * 64 blocks east of it (x = −C/2 on, the planet's westernmost columns), on four rows from the
     * equator to mid-latitudes. Each side must meet the planet-wide test's properties on its own.
     *
     * <p>What it checks is the fold, so it reads each chunk as the column rule and the carvers left it,
     * before any feature (Deep Time's terrain is complete by then; the planet-wide test bounds what
     * features add). A chunk's features write into its neighbours, so every chunk is read at the carvers
     * before any is taken on to its features, and a chunk something else had already taken further is
     * not counted. Why not after features, as until 2026-10-17: a side's 16 columns are 4 blocks apart,
     * so one patch of a feature covers a quarter of a side or more, which the planet-wide test's 128
     * scattered columns never meet. On the planet of P1-26's rain the equator west of the seam turned
     * from sea to land, and there mangrove trees stood on 4 of the 16 columns (standalone) and
     * Terralith's basalt cliffs stood basalt columns 5 to 10 blocks over the ground on 9 (MIC). The same
     * columns after their features are logged, not asserted.
     *
     * <p>Four rows are fixed (the equator, two at ±C/16, one at C/8); up to {@link #LAND_ROWS} more are
     * where far terrain shows the dry ground each side still lacks, searched up to 7/8 of the way to
     * the world's z edge, so the fold is checked on land whatever the planet has at the fixed rows
     * (see {@link #landRows}). On the planet of 2026-10-18 the seam lies in warm ocean, and the
     * fixed rows held 3 dry columns west of it and 15 east; on that of 2026-10-19 the land east of the
     * seam lay outside the fixed rows' span, and four rows within it brought the east to only 14.
     * Each side must compare at least {@link #LAND_COLUMNS} dry columns, or every dry column far
     * terrain shows on the rows read when they hold fewer (a planet whose seam is all sea passes that
     * side with a line saying so), and hold their ground within a block in nine of ten.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 4800)
    public static void farTerrainMatchesGeneratedChunksAcrossTheSeam(GameTestHelper helper) {   // ledger: BAK-042
        FarTerrain far = far(helper);
        ServerLevel level = far.level();
        int c = far.data().circumference();
        int[] fixed = {7, c / 16 + 7, -c / 16 + 7, c / 8 + 7};
        int[] landRows = landRows(far, c, fixed);
        int[] rows = java.util.stream.IntStream.concat(java.util.Arrays.stream(fixed), java.util.Arrays.stream(landRows)).toArray();
        int n = 2 * 16 * rows.length;
        FarTerrainCheck.Tally west = new FarTerrainCheck.Tally(), east = new FarTerrainCheck.Tally();
        FarTerrainCheck.Tally westAfter = new FarTerrainCheck.Tally(), eastAfter = new FarTerrainCheck.Tally();
        int[] taken = {0};   // chunks something else had already taken past the carvers: not counted before features
        int[] farDry = new int[2];   // dry columns far terrain shows among those compared (no carver's cut), west and east
        ChunkSampling.forEach(helper, 2 * n, j -> {
            int i = j % n;
            boolean features = j >= n;   // every chunk is read at the carvers first, then all of them after their features
            boolean eastSide = (i & 1) == 1;
            int k = (i >> 1) % 16, row = rows[(i >> 1) / 16];
            int x = seamX(c, eastSide, k);
            FarTerrain.Column column = far.column(x, row);
            if (column.ground() < level.getMinBuildHeight() + 32)
                return false;
            ChunkAccess chunk = level.getChunkSource().getChunk(x >> 4, row >> 4, features ? ChunkStatus.FEATURES : ChunkStatus.CARVERS, true);
            if (!features && chunk.getPersistedStatus() != ChunkStatus.CARVERS) {
                taken[0]++;
                return true;
            }
            FarTerrainCheck.RealColumn real = FarTerrainCheck.real(chunk, x, row);
            FarTerrainCheck.Tally into = features ? (eastSide ? eastAfter : westAfter) : (eastSide ? east : west);
            int carved = into.carved;
            into.add(column, real, chunk);
            if (!features && !column.water() && into.carved == carved)
                farDry[eastSide ? 1 : 0]++;   // a carver's cut is set aside, so it is not a dry column the side must compare
            if (Math.abs(column.ground() - real.ground()) > 2 || column.water() != real.water()
                    || !features && column.groundBlock().getBlock() != real.groundBlock().getBlock())
                DeepTime.LOGGER.info("[gametest] seamDiag {} {}", features ? "after features" : "before features", diag(chunk, column, real, x, row));
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] farTerrainMatchesGeneratedChunksAcrossTheSeam C={} rows={} takenFurtherAlready={} farDry west {} "
                            + "east {}{} west {} east {} (after features, logged only: west {} east {})", c, java.util.Arrays.toString(rows), taken[0],
                    farDry[0], farDry[1], farDry[0] < LAND_COLUMNS || farDry[1] < LAND_COLUMNS
                            ? " (fewer than " + LAND_COLUMNS + " dry columns on a side on the rows read: every one there is checked)" : "",
                    west, east, westAfter, eastAfter);
            assertAgrees(helper, west, 32, "west of the seam: ");
            assertAgrees(helper, east, 32, "east of the seam: ");
            assertLand(helper, west, farDry[0], "west of the seam: ");
            assertLand(helper, east, farDry[1], "east of the seam: ");
            helper.succeed();
        });
    }

    /** Sea-bed columns the sea-bed test takes of each kind (warm oceans, all other seas), at most one per chunk. */
    static final int SEA_BED_COLUMNS = 48;

    /** Whether a biome is a warm ocean (vanilla's name; Terralith paints both sandstone below its shallow band). */
    static boolean warmOcean(Holder<Biome> b) {
        String path = b.unwrapKey().map(k -> k.location().getPath()).orElse("");
        return path.equals("warm_ocean") || path.equals("deep_warm_ocean");
    }

    /**
     * Far terrain's sea beds against the chunk's, wherever the planet's seas are (BAK-065): the
     * level's whole surface rule paints a sea bed's top block, so the two agree, warm oceans
     * included (Terralith paints their floors sandstone below the shallow band, where the surface
     * table has sand: on the planet of 2026-10-21 the seam test, whose columns lie where the seam
     * falls, met 39 such columns). Up to {@link #SEA_BED_COLUMNS} sea columns in warm oceans and as
     * many in other seas, one per chunk, found by far terrain alone on rows over the planet; each
     * chunk is read at the carvers, before any feature (coral, disks), as the seam test reads them,
     * and one something else had taken further is not counted. Both sets must agree on the ground
     * block in nine of ten columns; the warm set only when the planet has at least 8 such columns.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 4800)
    public static void farTerrainSeaBedsFollowTheSurfaceRule(GameTestHelper helper) {   // ledger: BAK-065
        FarTerrain far = far(helper);
        ServerLevel level = far.level();
        int c = far.data().circumference();
        int minY = level.getMinBuildHeight() + 32;
        int zMax = (int) (0.75 * far.data().projection().zMax());
        List<FarTerrain.Column> warm = new ArrayList<>(), other = new ArrayList<>();
        java.util.Set<Long> chunks = new java.util.HashSet<>();
        // Rows 64 blocks apart from the equator out, alternating north and south; columns 97 apart
        // (prime, so the rows do not line up), offset per row.
        for (int r = 0; 64 * (r / 2) <= zMax && (warm.size() < SEA_BED_COLUMNS || other.size() < SEA_BED_COLUMNS); r++) {
            int z = (r % 2 == 0 ? 1 : -1) * 64 * ((r + 1) / 2) + 7;
            for (int x = -c / 2 + 5 + 13 * r; x < c / 2; x += 97) {
                FarTerrain.Column col = far.column(x, z);
                if (!col.water() || col.ground() < minY || col.waterTop() <= col.ground())
                    continue;
                List<FarTerrain.Column> into = warmOcean(col.biome()) ? warm : other;
                if (into.size() >= SEA_BED_COLUMNS || !chunks.add(net.minecraft.world.level.ChunkPos.asLong(x >> 4, z >> 4)))
                    continue;
                into.add(col);
            }
        }
        List<FarTerrain.Column> all = new ArrayList<>(warm);
        all.addAll(other);
        FarTerrainCheck.Tally warmTally = new FarTerrainCheck.Tally(), otherTally = new FarTerrainCheck.Tally();
        int[] taken = {0};
        ChunkSampling.forEach(helper, all.size(), i -> {
            FarTerrain.Column column = all.get(i);
            ChunkAccess chunk = level.getChunkSource().getChunk(column.x() >> 4, column.z() >> 4, ChunkStatus.CARVERS, true);
            if (chunk.getPersistedStatus() != ChunkStatus.CARVERS) {
                taken[0]++;
                return true;
            }
            FarTerrainCheck.RealColumn real = FarTerrainCheck.real(chunk, column.x(), column.z());
            (i < warm.size() ? warmTally : otherTally).add(column, real, chunk);
            if (column.groundBlock().getBlock() != real.groundBlock().getBlock() || column.water() != real.water())
                DeepTime.LOGGER.info("[gametest] seaBedDiag {} far layers {}", diag(chunk, column, real, column.x(), column.z()),
                        column.layers().subList(Math.max(0, column.layers().size() - 3), column.layers().size()));
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] farTerrainSeaBedsFollowTheSurfaceRule found warm {} other {} takenFurtherAlready={} warm {} other {}",
                    warm.size(), other.size(), taken[0], warmTally.toJson(), otherTally.toJson());
            helper.assertTrue(otherTally.columns >= 16, "only " + otherTally.columns + " sea-bed columns outside warm oceans compared");
            helper.assertTrue(otherTally.share(otherTally.groundBlockSame) >= 0.9, "the sea bed's block agrees in only "
                    + otherTally.groundBlockSame + " of " + otherTally.columns + " columns outside warm oceans: " + otherTally.toJson().get("ground_block_differences"));
            if (warmTally.columns >= 8)
                helper.assertTrue(warmTally.share(warmTally.groundBlockSame) >= 0.9, "the sea bed's block agrees in only "
                        + warmTally.groundBlockSame + " of " + warmTally.columns + " warm-ocean columns: " + warmTally.toJson().get("ground_block_differences"));
            helper.succeed();
        });
    }

    /** Column k (0 to 15) of a side of the seam: 4 blocks apart, west of x = C/2 or east of x = −C/2. */
    static int seamX(int c, boolean eastSide, int k) {
        return eastSide ? -c / 2 + 4 * k + 1 : c / 2 - 64 + 4 * k + 2;
    }

    /** Dry columns each side of the seam must compare, where the planet has them (a game choice, BAK-042). */
    static final int LAND_COLUMNS = 16;   // ledger: BAK-042

    /** How many rows the seam test may add where the land is (a game choice, BAK-042). */
    static final int LAND_ROWS = 8;   // ledger: BAK-042

    /** Dry columns far terrain shows on row z west and east of the seam (the build floor's band left out, as the test does). */
    static int[] dryColumns(FarTerrain far, int c, int z) {
        int minY = far.level().getMinBuildHeight() + 32;
        int[] dry = new int[2];
        for (int side = 0; side < 2; side++)
            for (int k = 0; k < 16; k++) {
                FarTerrain.Column col = far.column(seamX(c, side == 1, k), z);
                if (!col.water() && col.ground() >= minY)
                    dry[side]++;
            }
        return dry;
    }

    /**
     * Up to {@link #LAND_ROWS} rows (z ≡ 7 mod 16, |z| at most 7/8 of the world's z edge: the poles
     * on a Lambert world, the turning edges ±b/2 on a Petroff–Guyou one) chosen greedily for the dry
     * columns each side still lacks:
     * every side aims at one and a half times {@link #LAND_COLUMNS} (a margin for chunks another test
     * took further first) counting the fixed rows, each row's worth is the dry columns it brings to
     * a side short of its aim, and ties go to the row nearer the equator. The search stops when both
     * sides reach their aim or no row brings any. Far terrain only: no chunk is generated to choose
     * them. Until 2026-10-19 four rows were taken within the fixed rows' span (|z| at most C/8 + 7)
     * where the poorer side was driest, which on a planet whose land meets the seam on one side only
     * further out could not find it.
     */
    static int[] landRows(FarTerrain far, int c, int[] fixed) {
        int zMax = (int) (0.875 * far.data().projection().zMax());
        int aim = LAND_COLUMNS * 3 / 2;
        int[] short_ = {aim, aim}, band = new int[2];
        List<int[]> candidates = new ArrayList<>();   // {z, dry west, dry east}
        for (int z = 7 - 16 * Math.floorDiv(zMax + 7, 16); z <= zMax; z += 16) {
            int zz = z;
            int[] dry = dryColumns(far, c, z);
            band[0] += dry[0];
            band[1] += dry[1];
            if (java.util.Arrays.stream(fixed).anyMatch(f -> f == zz)) {
                short_[0] -= dry[0];
                short_[1] -= dry[1];
            } else {
                candidates.add(new int[]{z, dry[0], dry[1]});
            }
        }
        List<Integer> out = new ArrayList<>();
        List<String> picked = new ArrayList<>();
        while (out.size() < LAND_ROWS && (short_[0] > 0 || short_[1] > 0)) {
            int[] best = null;
            int bestGain = 0;
            for (int[] r : candidates) {
                int gain = Math.min(r[1], Math.max(0, short_[0])) + Math.min(r[2], Math.max(0, short_[1]));
                if (gain > bestGain || best != null && gain == bestGain && Math.abs(r[0]) < Math.abs(best[0])) {
                    best = r;
                    bestGain = gain;
                }
            }
            if (best == null)
                break;
            candidates.remove(best);
            out.add(best[0]);
            picked.add(best[0] + ":" + best[1] + "/" + best[2]);
            short_[0] -= best[1];
            short_[1] -= best[2];
        }
        DeepTime.LOGGER.info("[gametest] seam land rows {} (row:dry west/east) in |z| <= {}; dry columns in the whole band west {} east {}; "
                + "still short of {} after them: west {} east {}", picked, zMax, band[0], band[1], aim, Math.max(0, short_[0]), Math.max(0, short_[1]));
        return out.stream().mapToInt(Integer::intValue).toArray();
    }

    /**
     * A side of the seam compared enough dry columns ({@link #LAND_COLUMNS}, or all {@code farDry}
     * that far terrain shows on the rows read when fewer), and their ground agrees as the
     * planet-wide floor asks.
     */
    static void assertLand(GameTestHelper helper, FarTerrainCheck.Tally tally, int farDry, String where) {   // ledger: BAK-042
        int wanted = Math.min(LAND_COLUMNS, farDry);
        helper.assertTrue(tally.land >= wanted, where + "only " + tally.land + " dry columns compared (far terrain shows " + farDry
                + " on these rows): the seam is not checked on land");
        helper.assertTrue(tally.landHeightWithin1 >= 0.90 * tally.land,
                where + "the dry ground is within a block in only " + tally.landHeightWithin1 + " of " + tally.land + " dry columns");
    }

    /**
     * Distant Horizons is loaded when the run's mods include it ({@code -Ddeeptime.dh.expected},
     * set by neoforge/build.gradle from the server set and its GameTest exclusions), and then far
     * terrain is bound to it: the API resolves and the real test planet's override registered. A
     * run that expects it and lacks it fails here, rather than every far-terrain test passing
     * without Distant Horizons. Today no GameTest-server run has it: Distant Horizons 3.3.3 (as
     * 3.3.0 and 3.3.1) casts the server to {@code DedicatedServer} at start, which NeoForge's GameTest server is
     * not (tools/smoke/profiles/mic.gametest-excluded), so its path is the smoke check's
     * ({@code tools/smoke/run.py --dh}) on a dedicated server.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void farTerrainHasDistantHorizonsWhenExpected(GameTestHelper helper) {
        boolean expected = Boolean.getBoolean("deeptime.dh.expected");
        boolean installed = DhFarTerrain.installed();
        DeepTime.LOGGER.info("[gametest] farTerrainHasDistantHorizonsWhenExpected expected={} installed={} twGuardPatch={}",
                expected, installed, System.getProperty("deeptime.twDhGuard", "not configured"));
        helper.assertTrue(installed || !expected,
                "this run's mods include Distant Horizons (-Ddeeptime.dh.expected=true) but it is not loaded");
        if (installed && DhFarTerrain.enabled()) {
            helper.assertTrue(DhFarTerrain.api().isPresent(), "Distant Horizons is loaded but its API did not bind: " + DhFarTerrain.unavailable());
            ServerLevel level = RealPlanetGameTests.level(helper);
            DhFarTerrain.Bound bound = DhFarTerrain.bound(level.dimension());
            helper.assertTrue(bound != null && bound.registered(), "far terrain is not registered with Distant Horizons for "
                    + level.dimension().location() + (bound == null ? "" : ": " + bound.message()));
        }
        helper.succeed();
    }

    private record Layer(int bottom, int top, BlockState block, int light) {}

    private static List<Layer> layers(FarTerrain far, int x, int z, int step, List<Holder<Biome>> biome) {
        List<Layer> out = new ArrayList<>();
        far.build(x, z, step, 1, (relX, relZ, count, bottom, top, block, material, light, b) -> {
            biome.add(b);
            for (int k = 0; k < count; k++)
                out.add(new Layer(bottom[k], top[k], block[k], light[k]));
        });
        return out;
    }

    /**
     * The seam and the build range: a column one lap east or west is the same column, biome and
     * blocks included, at full and at coarse detail; every column's layers run from the level's
     * bottom to its top without a gap; and a whole section builds across the seam.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 600)
    public static void farTerrainWrapsAndFillsTheColumn(GameTestHelper helper) {
        FarTerrain far = far(helper);
        ServerLevel level = far.level();
        int c = far.data().circumference();
        int minY = level.getMinBuildHeight(), maxY = level.getMaxBuildHeight();
        int checked = 0;
        for (int step : new int[]{1, 8}) {
            for (int i = 0; i < 48; i++) {
                int[] p = column(i * 2, c);
                List<Holder<Biome>> b0 = new ArrayList<>(), b1 = new ArrayList<>(), b2 = new ArrayList<>();
                List<Layer> here = layers(far, p[0], p[1], step, b0);
                helper.assertTrue(here.equals(layers(far, p[0] + c, p[1], step, b1)) && here.equals(layers(far, p[0] - 2 * c, p[1], step, b2)),
                        "column " + p[0] + ", " + p[1] + " at step " + step + " differs one lap on");
                helper.assertTrue(b0.equals(b1) && b0.equals(b2), "the biome of column " + p[0] + ", " + p[1] + " differs one lap on");
                helper.assertTrue(here.get(0).bottom() == minY && here.get(here.size() - 1).top() == maxY,
                        "column " + p[0] + ", " + p[1] + " does not span the build range");
                for (int k = 0; k + 1 < here.size(); k++)
                    helper.assertTrue(here.get(k).top() == here.get(k + 1).bottom(), "a gap in column " + p[0] + ", " + p[1]);
                checked++;
            }
        }
        // A section across the seam: its columns east of the seam are the planet's westernmost.
        int[] columns = {0};
        int[] ground = new int[64];
        far.build(c / 2 - 32, 0, 1, 64, (relX, relZ, count, bottom, top, block, material, light, biome) -> {
            columns[0]++;
            if (relZ == 20) {
                int g = minY - 1;
                for (int k = 0; k < count; k++)
                    if (material[k] != FarColumn.AIR && material[k] != FarColumn.WATER && material[k] != FarColumn.WATER_ICE
                            && material[k] != FarColumn.SNOW_LAYER)
                        g = top[k] - 1;
                ground[relX] = g;
            }
        });
        helper.assertTrue(columns[0] == 4096, "a section has " + columns[0] + " columns");
        for (int i = 0; i < 64; i++) {
            int x = c / 2 - 32 + i, canonical = i < 32 ? x : x - c;
            int expected = far.column(canonical, 20).ground();
            helper.assertTrue(ground[i] == expected, "across the seam, column " + x + " has ground " + ground[i]
                    + ", the planet's column " + canonical + " has " + expected);
        }
        DeepTime.LOGGER.info("[gametest] farTerrainWrapsAndFillsTheColumn columns={} seamSectionColumns={}", checked, columns[0]);
        helper.succeed();
    }

    /**
     * A section is cheap and threads agree: sections built on four threads at once, each in its own
     * order, equal the ones built alone column for column, and the time a section takes is logged
     * ({@code ms/section}) for full and coarse detail. The only time asserted is a generous ceiling a
     * real chunk generator could not meet (a section is 16 chunks). The agreement caught vanilla's
     * warm-started climate search settling a tie by the thread's history (see
     * {@code ColdBiomeSearch}); a differing column is logged with its ground and biome on both sides.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void farTerrainSectionsAreCheapAndThreadSafe(GameTestHelper helper) {
        FarTerrain far = far(helper);
        try {
            sectionsAreCheapAndThreadSafe(helper, far);
        } catch (java.util.concurrent.ExecutionException | java.util.concurrent.TimeoutException | InterruptedException e) {
            DeepTime.LOGGER.error("[gametest] farTerrainSectionsAreCheapAndThreadSafe threw", e);
            helper.fail("building sections on other threads failed: " + e);
        }
    }

    private static void sectionsAreCheapAndThreadSafe(GameTestHelper helper, FarTerrain far)
            throws java.util.concurrent.ExecutionException, java.util.concurrent.TimeoutException, InterruptedException {
        int c = far.data().circumference();
        int[][] jobs = new int[24][];
        for (int k = 0; k < jobs.length; k++)
            jobs[k] = new int[]{-c / 2 + (k * 7 % 24) * (c / 24) / 64 * 64, ((k * 5) % 12 - 6) * ((int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c) / 8) / 64 * 64, k % 3 == 0 ? 8 : 1};
        SectionColumn[][] alone = new SectionColumn[jobs.length][];
        for (int k = 0; k < jobs.length; k++)
            alone[k] = sectionColumns(far, jobs[k]);
        int mismatches = threadMismatches(far, jobs, alone);
        helper.assertTrue(mismatches == 0, mismatches + " sections differ on another thread (columns in the log, threadsafeDiag)");
        double[] ms = new double[2];
        for (int d = 0; d < 2; d++) {
            int step = d == 0 ? 1 : 8;
            double best = Double.MAX_VALUE;
            for (int pass = 0; pass < 3; pass++) {
                long t0 = System.nanoTime();
                for (int[] job : jobs)
                    hash(far, new int[]{job[0], job[1], step});
                best = Math.min(best, (System.nanoTime() - t0) / 1e6 / jobs.length);
            }
            ms[d] = best;
        }
        DeepTime.LOGGER.info("[gametest] farTerrainSectionsAreCheapAndThreadSafe sections={} threads=4 ms/section: detail 0 {} detail 3 {}",
                jobs.length, String.format(java.util.Locale.ROOT, "%.3f", ms[0]), String.format(java.util.Locale.ROOT, "%.3f", ms[1]));
        helper.assertTrue(ms[0] < 250.0 && ms[1] < 250.0, "a section takes " + ms[0] + " ms at full detail, " + ms[1] + " ms at detail 3");
        helper.succeed();
    }

    /** One column of a section, for comparing threads: its own hash, its ground and that block, its biome. */
    private record SectionColumn(long hash, int ground, BlockState block, Holder<Biome> biome) {}

    private static SectionColumn[] sectionColumns(FarTerrain far, int[] job) {
        SectionColumn[] out = new SectionColumn[64 * 64];
        far.build(job[0], job[1], job[2], 64, (relX, relZ, count, bottom, top, block, material, light, biome) -> {
            long v = 1125899906842597L;
            int ground = Integer.MIN_VALUE;
            BlockState g = null;
            for (int k = 0; k < count; k++) {
                v = 31 * (31 * (31 * (31 * v + bottom[k]) + top[k]) + block[k].hashCode()) + light[k];
                if (material[k] != FarColumn.AIR && material[k] != FarColumn.WATER && material[k] != FarColumn.WATER_ICE
                        && material[k] != FarColumn.SNOW_LAYER) {
                    ground = top[k] - 1;
                    g = block[k];
                }
            }
            out[relZ * 64 + relX] = new SectionColumn(31 * v + biome.hashCode(), ground, g, biome);
        });
        return out;
    }

    /**
     * Builds every section on four fresh threads at once, each in its own order; the number of
     * (thread, section) pairs that differ from {@code alone}. The first differing columns are logged
     * ({@code threadsafeDiag}): where, and the ground, its block and the biome on both sides.
     */
    private static int threadMismatches(FarTerrain far, int[][] jobs, SectionColumn[][] alone)
            throws java.util.concurrent.ExecutionException, java.util.concurrent.TimeoutException, InterruptedException {
        java.util.concurrent.ExecutorService pool = java.util.concurrent.Executors.newFixedThreadPool(4);
        int mismatches = 0, logged = 0;
        try {
            List<java.util.concurrent.Future<SectionColumn[][]>> runs = new ArrayList<>();
            for (int t = 0; t < 4; t++) {
                int offset = t;
                runs.add(pool.submit(() -> {
                    SectionColumn[][] out = new SectionColumn[jobs.length][];
                    for (int n = 0; n < jobs.length; n++) {
                        int k = (n + offset * 7) % jobs.length;
                        out[k] = sectionColumns(far, jobs[k]);
                    }
                    return out;
                }));
            }
            for (int t = 0; t < runs.size(); t++) {
                SectionColumn[][] got = runs.get(t).get(120, java.util.concurrent.TimeUnit.SECONDS);
                for (int k = 0; k < jobs.length; k++) {
                    int differ = 0;
                    for (int q = 0; q < got[k].length; q++) {
                        SectionColumn a = alone[k][q], b = got[k][q];
                        if (a.hash() == b.hash())
                            continue;
                        differ++;
                        if (logged++ < 16) {
                            int step = jobs[k][2], x = jobs[k][0] + (q % 64) * step + step / 2, z = jobs[k][1] + (q / 64) * step + step / 2;
                            DeepTime.LOGGER.info("[gametest] threadsafeDiag thread {} section {} column ({}, {}): alone ground {} {} biome {}; thread ground {} {} biome {}",
                                    t, java.util.Arrays.toString(jobs[k]), x, z,
                                    a.ground(), a.block(), a.biome().getRegisteredName(), b.ground(), b.block(), b.biome().getRegisteredName());
                        }
                    }
                    if (differ > 0) {
                        mismatches++;
                        DeepTime.LOGGER.info("[gametest] threadsafeDiag thread {} section {} differs in {} columns",
                                t, java.util.Arrays.toString(jobs[k]), differ);
                    }
                }
            }
        } finally {
            pool.shutdownNow();
        }
        return mismatches;
    }

    private static long hash(FarTerrain far, int[] job) {
        long[] h = {1125899906842597L};
        far.build(job[0], job[1], job[2], 64, (relX, relZ, count, bottom, top, block, material, light, biome) -> {
            long v = h[0];
            for (int k = 0; k < count; k++)
                v = 31 * (31 * (31 * (31 * v + bottom[k]) + top[k]) + block[k].hashCode()) + light[k];
            h[0] = 31 * v + biome.hashCode();
        });
        return h[0];
    }
}
