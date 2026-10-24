// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The column-to-LOD conversion against the rasters (plan §10.5 P3-04, the exit test): far terrain's
 * columns ({@link FarColumn}) are the world data's own column rule. On the real test planet
 * ({@code planet4k}, every pipeline stage) and the lake fixture, whatever planet the fixture holds:
 * the geometry is {@link Water}'s decision, the rock is {@link Columns}'s stack, the layers fill the
 * build range without gaps, x wraps, the rasters clamp past the pole edge, and threads agree. The
 * numbers a planet happens to have (how many rivers, how much ice) are printed, never asserted.
 */
class FarColumnTest {

    static WorldData planet, lake;

    @BeforeAll
    static void open() {
        planet = WorldData.open(Fixtures.world("planet4k"));
        lake = WorldData.open(Fixtures.world("lake1024"));
    }

    static Water.Spec spec(WorldData wd) {
        return Water.Spec.of(wd, Columns.NOISE_CELL);
    }

    static FarColumn.Shape[] shapes(int n) {
        FarColumn.Shape[] s = new FarColumn.Shape[(n + 2) * (n + 2)];
        for (int i = 0; i < s.length; i++)
            s[i] = new FarColumn.Shape();
        return s;
    }

    static FarColumn.Soil soil(int depth, boolean bare, boolean frozen) {
        FarColumn.Soil s = new FarColumn.Soil();
        s.depth = depth;
        s.bare = bare;
        s.frozen = frozen;
        return s;
    }

    /** The areas the tests walk: mid-latitudes, the seam on both laps, the pole edge and past it. */
    static List<long[]> areas(WorldData wd, int step) {
        int c = wd.circumference();
        long span = 64L * step;
        long pole = (long) WorldProjection.Lambert.radius(c);   // the pole edge (the planet's projection)
        List<long[]> out = new ArrayList<>();
        for (long z : new long[]{0, c / 8, -c / 8, pole - span / 2, -pole - span / 2, pole + 64}) {
            out.add(new long[]{0, z});
            out.add(new long[]{c / 4, z});
            out.add(new long[]{c / 2 - span / 2, z});       // straddles the seam
            out.add(new long[]{-c / 2 - span / 2, z});      // the same seam, from the other lap
        }
        return out;
    }

    static IceCover.Spec coverSpec(WorldData wd) {
        return wd.iceCover().map(IceCover::spec).orElse(null);
    }

    /**
     * One column's layers must fill [minY, maxY] bottom-up without a gap, with air above the ground
     * (unless the ground stands at the very top of the build range).
     */
    static void assertFills(FarColumn.Layers l, int minY, int maxY, String where) {
        assertTrue(l.count >= 1, where + ": no layers");
        assertEquals(minY, l.bottom[0], where + ": the first layer starts at min_y");
        for (int k = 0; k < l.count; k++) {
            assertTrue(l.top[k] > l.bottom[k], where + ": layer " + k + " is empty or upside down");
            if (k + 1 < l.count)
                assertEquals(l.top[k], l.bottom[k + 1], where + ": a gap or an overlap above layer " + k);
        }
        assertEquals(maxY + 1, l.top[l.count - 1], where + ": the last layer ends at the top of the build range");
        int top = l.topSolid();
        if (top < 0 || l.top[top] <= maxY)
            assertEquals(FarColumn.AIR, l.material[l.count - 1], where + ": the last layer is air");
        for (int k = 0; k < l.count; k++)
            assertTrue(l.material[k] >= 0 && l.material[k] < FarColumn.MATERIALS, where + ": layer " + k + " has no material");
    }

    @Test
    void layersFillTheBuildRangeWithoutGaps() {
        for (WorldData wd : new WorldData[]{planet, lake}) {
            Water.Spec spec = spec(wd);
            RasterSampler strata = wd.has(WorldData.HORIZONS) ? wd.layer(WorldData.HORIZONS) : null;
            Columns.Stack stack = new Columns.Stack(8);
            FarColumn.Layers layers = new FarColumn.Layers();
            long columns = 0, layerCount = 0;
            for (int step : new int[]{1, 2, 8, 64}) {
                if (64L * step > wd.circumference())
                    continue;
                FarColumn.Shape[] grid = shapes(64);
                for (long[] a : areas(wd, step)) {
                    FarColumn.grid(wd, spec, a[0], a[1], step, 64, grid);
                    for (int j = 0; j < 64; j += 3) {
                        for (int i = 0; i < 64; i += 3) {
                            FarColumn.Shape s = grid[(j + 1) * 66 + (i + 1)];
                            long x = a[0] + (long) i * step + step / 2, z = a[1] + (long) j * step + step / 2;
                            if (strata != null)
                                Columns.stack(wd, strata, (int) FarColumn.wrapX(wd, x), (int) z, stack);
                            int depth = FarColumn.exposure(grid, 64, i, j) + 8;
                            for (FarColumn.Soil soil : new FarColumn.Soil[]{soil(4, false, false), soil(1, false, true), soil(4, true, true)}) {
                                FarColumn.layers(spec.minY(), spec.maxY(), s, soil, strata == null ? null : stack, depth, coverSpec(wd), layers);
                                assertFills(layers, spec.minY(), spec.maxY(), "step " + step + " column " + x + ", " + z);
                                columns++;
                                layerCount += layers.count;
                            }
                        }
                    }
                }
            }
            System.out.printf("[far] %s: %d columns checked, %.2f layers a column%n", wd.folder().getFileName(), columns,
                    (double) layerCount / columns);
        }
    }

    @Test
    void fullDetailIsTheWaterColumnRule() {
        for (WorldData wd : new WorldData[]{planet, lake}) {
            Water.Spec spec = spec(wd);
            IceCover ice = wd.iceCover().orElse(null);
            FarColumn.Shape[] grid = shapes(64);
            FarColumn.Layers layers = new FarColumn.Layers();
            int[] kinds = new int[Water.Kind.values().length];
            int iced = 0;
            for (long[] a : areas(wd, 1)) {
                FarColumn.grid(wd, spec, a[0], a[1], 1, 64, grid);
                for (int j = 0; j < 64; j++) {
                    for (int i = 0; i < 64; i++) {
                        long x = a[0] + i, z = a[1] + j;
                        Water.Column col = Water.column(wd, spec, x, z);
                        FarColumn.Shape s = grid[(j + 1) * 66 + (i + 1)];
                        String where = wd.folder().getFileName() + " column " + x + ", " + z + " (" + col.kind().id + ")";
                        assertEquals(col.kind(), s.kind, where);
                        kinds[col.kind().ordinal()]++;
                        int expected;
                        if (col.kind().isWater()) {
                            expected = col.bed();
                            assertEquals(col.top().intValue(), s.waterTop, where + ": the water's top");
                        } else {
                            expected = col.ground();
                            if (col.maxGround() != null && expected > col.maxGround() && col.maxGround() >= spec.minY())
                                expected = col.maxGround();
                            if (col.minGround() != null && expected < col.minGround() && col.minGround() <= spec.maxY())
                                expected = col.minGround();
                        }
                        expected = Math.max(spec.minY() - 1, Math.min(spec.maxY(), expected));
                        assertEquals(expected, s.ground, where + ": the ground");
                        int cover = ice == null || s.water() ? 0 : ice.blocks((int) FarColumn.wrapX(wd, x), (int) z);
                        assertEquals(cover, s.ice, where + ": the ice cover");
                        if (cover > 0)
                            iced++;

                        // The layers show that geometry: the top solid layer ends at the ground, water fills up to its top.
                        FarColumn.layers(spec.minY(), spec.maxY(), s, soil(4, false, false), null, 0, coverSpec(wd), layers);
                        int top = layers.topSolid();
                        assertEquals(s.ground, top < 0 ? spec.minY() - 1 : layers.top[top] - 1, where + ": the top solid layer");
                        if (s.water()) {
                            assertEquals(FarColumn.WATER, layers.material[top + 1], where + ": water over the bed");
                            assertEquals(s.ground + 1, layers.bottom[top + 1], where + ": the water starts on the bed");
                            assertEquals(Math.min(s.waterTop, spec.maxY()) + 1, layers.top[top + 1], where + ": the water's top");
                        } else if (top >= 0 && top + 1 < layers.count) {
                            assertEquals(FarColumn.AIR, layers.material[top + 1], where + ": air over dry ground");
                        }
                    }
                }
            }
            System.out.printf("[far] %s at full detail: none=%d bank=%d river=%d lake=%d sea=%d ice-covered=%d%n",
                    wd.folder().getFileName(), kinds[Water.Kind.NONE.ordinal()], kinds[Water.Kind.BANK.ordinal()],
                    kinds[Water.Kind.RIVER.ordinal()], kinds[Water.Kind.LAKE.ordinal()], kinds[Water.Kind.SEA.ordinal()], iced);
        }
        // The lake fixture exists to have a lake: the conversion must show it.
        Water.Spec spec = spec(lake);
        int lakes = 0;
        FarColumn.Shape[] grid = shapes(64);
        for (int z = -256; z < 256; z += 64)
            for (int x = -512; x < 512; x += 64) {
                FarColumn.grid(lake, spec, x, z, 1, 64, grid);
                for (FarColumn.Shape s : grid)
                    if (s.kind == Water.Kind.LAKE)
                        lakes++;
            }
        assertTrue(lakes > 0, "the lake fixture has no lake column in far terrain");
    }

    @Test
    void rockIsTheStrataColumnRule() {
        Water.Spec spec = spec(planet);
        RasterSampler strata = planet.layer(WorldData.HORIZONS);
        Columns.Stack stack = new Columns.Stack(8), expected = new Columns.Stack(8);
        FarColumn.Shape[] grid = shapes(64);
        FarColumn.Layers layers = new FarColumn.Layers();
        long blocks = 0, runs = 0, columns = 0;
        for (long[] a : areas(planet, 1)) {
            FarColumn.grid(planet, spec, a[0], a[1], 1, 64, grid);
            for (int j = 0; j < 64; j += 5) {
                for (int i = 0; i < 64; i += 5) {
                    FarColumn.Shape s = grid[(j + 1) * 66 + (i + 1)];
                    int x = (int) FarColumn.wrapX(planet, a[0] + i), z = (int) (a[1] + j);
                    Columns.stack(planet, strata, x, z, stack);
                    Columns.stack(planet, strata, x, z, expected);
                    int soilDepth = 3, rockDepth = 40;
                    FarColumn.layers(spec.minY(), spec.maxY(), s, soil(soilDepth, false, false), stack, rockDepth, null, layers);
                    int rockTop = s.ground - (s.ground - spec.minY() + 1 >= soilDepth ? soilDepth : s.ground - spec.minY() + 1);
                    if (rockTop < spec.minY())
                        continue;
                    columns++;
                    int detailFrom = Math.max(spec.minY(), rockTop - rockDepth);
                    for (int y = detailFrom; y <= rockTop; y++) {
                        int k = layers.at(y);
                        String where = "column " + x + ", " + z + " y " + y;
                        assertEquals(FarColumn.ROCK, layers.material[k], where + ": rock under the soil");
                        int h = expected.horizonAt(y);
                        assertEquals(h < 0 ? -1 : expected.classes[h], layers.cls[k], where + ": the rock class");
                        if (h >= 0)
                            assertEquals(expected.units[h], layers.unit[k], where + ": the horizon's unit key");
                        blocks++;
                    }
                    // Below the detail: one run of the rock at that depth, down to the floor.
                    if (detailFrom > spec.minY()) {
                        int k = layers.at(spec.minY());
                        assertEquals(k, layers.at(detailFrom - 1), "column " + x + ", " + z + ": one run below the detail");
                        int h = expected.horizonAt(detailFrom);
                        assertEquals(h < 0 ? -1 : expected.classes[h], layers.cls[k], "column " + x + ", " + z + ": the deep run's class");
                    }
                    // The soil sits on the rock.
                    assertNotEquals(FarColumn.ROCK, layers.material[layers.at(s.ground)], "column " + x + ", " + z + ": soil on top");
                    for (int k = 0; k < layers.count; k++)
                        if (layers.material[k] == FarColumn.ROCK)
                            runs++;
                }
            }
        }
        assertTrue(columns > 100, "only " + columns + " columns with rock were checked");
        System.out.printf("[far] rock: %d columns, %d blocks equal to the strata column rule, %.2f rock runs a column (40 blocks of detail)%n",
                columns, blocks, (double) runs / columns);
    }

    @Test
    void wrapsEastToWestAndClampsPastThePoleEdge() {
        int c = planet.circumference();
        Water.Spec spec = spec(planet);
        for (int step : new int[]{1, 4, 32}) {
            FarColumn.Shape[] a = shapes(64), b = shapes(64), d = shapes(64);
            long pole = (long) WorldProjection.Lambert.radius(c);
            for (long z0 : new long[]{-64, pole - 32, pole + 640}) {
                long x0 = c / 2 - 32L * step;   // the area straddles the seam
                FarColumn.grid(planet, spec, x0, z0, step, 64, a);
                FarColumn.grid(planet, spec, x0 + c, z0, step, 64, b);
                FarColumn.grid(planet, spec, x0 - 3L * c, z0, step, 64, d);
                for (int k = 0; k < a.length; k++) {
                    for (FarColumn.Shape o : new FarColumn.Shape[]{b[k], d[k]}) {
                        assertEquals(a[k].kind, o.kind, "step " + step + ": the kind one lap on");
                        assertEquals(a[k].ground, o.ground, "step " + step + ": the ground one lap on");
                        assertEquals(a[k].water() ? a[k].waterTop : 0, o.water() ? o.waterTop : 0, "step " + step + ": the water one lap on");
                        assertEquals(a[k].ice, o.ice, "step " + step + ": the ice cover one lap on");
                    }
                }
            }
        }
        // Across the seam the columns are the canonical lap's: column C/2 + k is column −C/2 + k.
        FarColumn.Shape[] grid = shapes(64);
        FarColumn.grid(planet, spec, c / 2 - 32, 0, 1, 64, grid);
        for (int i = 0; i < 64; i++) {
            long x = c / 2 - 32 + i;
            Water.Column col = Water.column(planet, spec, FarColumn.wrapX(planet, x), 17);
            FarColumn.Shape s = grid[(17 + 1) * 66 + (i + 1)];
            assertEquals(col.kind(), s.kind, "column " + x + " across the seam");
            if (!col.kind().isWater() && col.minGround() == null && col.maxGround() == null)
                assertEquals(Math.max(spec.minY() - 1, Math.min(spec.maxY(), col.ground())), s.ground, "column " + x + " across the seam");
        }
        // Past the pole edge the rasters clamp: far terrain is what the column evaluator says there.
        long z = (long) WorldProjection.Lambert.radius(c) + 200;
        FarColumn.grid(planet, spec, 96, z, 1, 64, grid);
        for (int i = 0; i < 64; i += 7) {
            Columns.Sample sample = Columns.sample(planet, 96 + i, (int) z + 9);
            FarColumn.Shape s = grid[(9 + 1) * 66 + (i + 1)];
            assertEquals(sample.hydro().kind(), s.kind, "past the pole edge, column " + (96 + i));
        }
    }

    @Test
    void coarseColumnsAreTheirCentres() {
        Water.Spec spec = spec(planet);
        for (int step : new int[]{2, 8, 64}) {
            FarColumn.Shape[] grid = shapes(64);
            for (long[] a : areas(planet, step)) {
                FarColumn.grid(planet, spec, a[0], a[1], step, 64, grid);
                for (int j = 0; j < 64; j += 9) {
                    for (int i = 0; i < 64; i += 9) {
                        long x = a[0] + (long) i * step + step / 2, z = a[1] + (long) j * step + step / 2;
                        long wx = FarColumn.wrapX(planet, x);
                        Rivers r = planet.rivers();
                        Water.Core core = Water.core(planet, spec, wx, z, planet.height(wx, z),
                                r == null ? Rivers.Sample.NONE : r.sample(wx, z));
                        FarColumn.Shape s = grid[(j + 1) * 66 + (i + 1)];
                        String where = "step " + step + " column " + x + ", " + z;
                        assertEquals(core.kind(), s.kind, where);
                        int ground = core.kind().isWater() ? core.bed() : core.ground();
                        assertEquals(Math.max(spec.minY() - 1, Math.min(spec.maxY(), ground)), s.ground, where + ": the ground");
                        if (core.kind().isWater())
                            assertEquals(core.top(), s.waterTop, where + ": the water's top");
                    }
                }
            }
        }
    }

    /**
     * A wide column's rock is its nearest strata cell's stack as stored; a column up to
     * {@link FarColumn#FULL_RULE_MAX_STEP} blocks wide gets the column rule itself. Both are read
     * through the world's folds and faults (P2-01): a narrow column is the strata pass's column, a
     * wide one the cell's stack where no structure touches it and a well-formed stack under the
     * column's own first top where one does.
     */
    @Test
    void wideColumnsTakeTheNearestCellsStack() {
        RasterSampler strata = planet.layer(WorldData.HORIZONS);
        GridDesc g = strata.desc().grid();
        Columns.Stack got = new Columns.Stack(8), rule = new Columns.Stack(8), cell = new Columns.Stack(8);
        Structure.Scratch scratch = new Structure.Scratch();
        int c = planet.circumference(), checked = 0, untouched = 0;
        for (int n = 0; n < 400; n++) {
            long x = -c / 2 + (n * 97L) % c, z = -c / 4 + (n * 61L) % (c / 2);
            for (int step : new int[]{1, 2}) {
                FarColumn.rock(planet, strata, x, z, step, got);
                Columns.deformedStack(planet, strata, (int) x, (int) z, rule, scratch);
                assertEquals(rule.count, got.count, "step " + step + " column " + x + ", " + z);
                for (int h = 0; h < rule.count; h++) {
                    assertEquals(rule.tops[h], got.tops[h]);
                    assertEquals(rule.classes[h], got.classes[h]);
                    assertEquals(rule.units[h], got.units[h]);
                }
            }
            FarColumn.rock(planet, strata, x, z, 8, got);
            long i = Math.round(Math.floor((double) (x - g.originBlockX()) / g.r() + 0.5));
            long j = Math.round(Math.floor((double) (z - g.originBlockZ()) / g.r() + 0.5));
            assertTrue(got.count > 0, "no rock at column " + x + ", " + z);
            // Top-down, strictly descending tops, and every class is one the cell stores, in its order.
            FarColumn.cellStack(strata, x, z, cell);
            boolean same = cell.count == got.count;
            for (int q = 0; same && q < got.count; q++)
                same = cell.tops[q] == got.tops[q] && cell.classes[q] == got.classes[q];
            if (same)
                untouched++;
            int h = 0;
            for (int q = 0; q < got.count; q++) {
                if (q > 0)
                    assertTrue(got.tops[q] < got.tops[q - 1], "tops descend at column " + x + ", " + z);
                if (!same)
                    continue;   // a fold, a fault or an intrusion: rock from elsewhere, or shifted
                while (h < strata.stackDepth() && strata.horizonClass(i, j, h) != got.classes[q])
                    h++;
                assertTrue(h < strata.stackDepth(), "class " + got.classes[q] + " is not in the nearest cell's stack, in order");
            }
            assertEquals((int) Math.floor(strata.horizonTop(i, j, 0)), got.tops[0], "the first horizon's top at column " + x + ", " + z);
            checked++;
        }
        assertEquals(400, checked);
        assertTrue(untouched > 100, "only " + untouched + " of 400 wide columns are the cell's own stack");
    }

    // ---- The layer rules, on hand-made columns ----

    static FarColumn.Shape shape(Water.Kind kind, int ground, int waterTop, int ice) {
        FarColumn.Shape s = new FarColumn.Shape();
        s.kind = kind;
        s.channel = kind == Water.Kind.RIVER;
        s.ground = ground;
        s.waterTop = waterTop;
        s.ice = ice;
        return s;
    }

    static int[] materials(FarColumn.Layers l) {
        return java.util.Arrays.copyOf(l.material, l.count);
    }

    @Test
    void dryGroundIsRockSoilAndAir() {
        FarColumn.Layers l = new FarColumn.Layers();
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 70, Integer.MIN_VALUE, 0), soil(4, false, false), null, 8, null, l);
        assertFills(l, -64, 319, "dry ground");
        assertEquals(List.of(FarColumn.ROCK, FarColumn.SOIL_UNDER, FarColumn.SOIL_TOP, FarColumn.AIR),
                java.util.Arrays.stream(materials(l)).boxed().toList());
        assertEquals(67, l.bottom[1]);   // three blocks under the top block
        assertEquals(70, l.bottom[2]);
        assertEquals(71, l.bottom[3]);
    }

    @Test
    void bareGroundIsRockToTheTop() {
        FarColumn.Layers l = new FarColumn.Layers();
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 70, Integer.MIN_VALUE, 0), soil(4, true, false), null, 8, null, l);
        assertFills(l, -64, 319, "bare ground");
        assertEquals(List.of(FarColumn.ROCK, FarColumn.AIR), java.util.Arrays.stream(materials(l)).boxed().toList());
        assertEquals(71, l.top[0]);
    }

    @Test
    void frozenGroundCarriesASnowLayerAndFrozenWaterAnIceTop() {
        FarColumn.Layers l = new FarColumn.Layers();
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 70, Integer.MIN_VALUE, 0), soil(4, false, true), null, 8, null, l);
        assertFills(l, -64, 319, "frozen ground");
        int snow = l.at(71);
        assertEquals(FarColumn.SNOW_LAYER, l.material[snow]);
        assertEquals(72, l.top[snow]);
        assertEquals(70, l.top[l.topSolid()] - 1, "the snow layer is not the ground");

        FarColumn.layers(-64, 319, shape(Water.Kind.LAKE, 60, 66, 0), soil(4, false, true), null, 8, null, l);
        assertFills(l, -64, 319, "a frozen lake");
        assertEquals(FarColumn.LAKE_BED, l.material[l.at(60)]);
        assertEquals(FarColumn.WATER, l.material[l.at(61)]);
        assertEquals(FarColumn.WATER, l.material[l.at(65)]);
        assertEquals(FarColumn.WATER_ICE, l.material[l.at(66)]);
        assertEquals(FarColumn.AIR, l.material[l.at(67)]);

        // One block of water freezes whole.
        FarColumn.layers(-64, 319, shape(Water.Kind.SEA, 61, 62, 0), soil(4, false, true), null, 8, null, l);
        assertEquals(FarColumn.SEA_BED, l.material[l.at(61)]);
        assertEquals(FarColumn.WATER_ICE, l.material[l.at(62)]);
    }

    @Test
    void riversHaveTheirBedAndTheSeaItsFloor() {
        FarColumn.Layers l = new FarColumn.Layers();
        FarColumn.Shape river = shape(Water.Kind.RIVER, 80, 82, 0);
        FarColumn.layers(-64, 319, river, soil(4, false, false), null, 8, null, l);
        assertFills(l, -64, 319, "a river");
        assertEquals(FarColumn.RIVER_GRAVEL, l.material[l.at(80)]);
        assertEquals(FarColumn.SOIL_UNDER, l.material[l.at(79)]);
        assertEquals(FarColumn.WATER, l.material[l.at(82)]);
        river.wideRiver = true;
        FarColumn.layers(-64, 319, river, soil(4, true, false), null, 8, null, l);
        assertEquals(FarColumn.RIVER_SAND, l.material[l.at(80)], "a wide river's bed is sand, on bare rock too");
        assertEquals(FarColumn.ROCK, l.material[l.at(79)]);

        FarColumn.layers(-64, 319, shape(Water.Kind.SEA, 40, 62, 0), soil(4, false, false), null, 8, null, l);
        assertEquals(FarColumn.SEA_BED, l.material[l.at(40)]);
        assertEquals(FarColumn.WATER, l.material[l.at(41)]);
        assertEquals(63, l.top[l.at(62)]);
        // A channel run on into the sea (HYD-121): the channel's sediment on the bed it cuts.
        FarColumn.Shape mouth = shape(Water.Kind.SEA, 55, 62, 0);
        mouth.channel = true;
        mouth.wideRiver = true;
        FarColumn.layers(-64, 319, mouth, soil(4, false, false), null, 8, null, l);
        assertEquals(FarColumn.RIVER_SAND, l.material[l.at(55)]);
        assertEquals(FarColumn.WATER, l.material[l.at(56)]);
    }

    @Test
    void theIceCoverReplacesTheTopBlocksAndKeepsTheHeight() {
        IceCover.Spec spec = new IceCover.Spec(50.0, 12, 2, 6, 2, "minecraft:snow_block", "minecraft:packed_ice", "minecraft:blue_ice");
        FarColumn.Layers l = new FarColumn.Layers();
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 100, Integer.MIN_VALUE, 8), soil(4, false, false), null, 8, spec, l);
        assertFills(l, -64, 319, "an ice cap");
        assertEquals(100, l.top[l.topSolid()] - 1, "the cover does not raise the ground");
        for (int i = 0; i < 8; i++) {
            int kind = spec.kind(i, 8);
            int want = kind == IceCover.Spec.SNOW ? FarColumn.COVER_SNOW : kind == IceCover.Spec.DEEP_ICE ? FarColumn.COVER_DEEP_ICE : FarColumn.COVER_ICE;
            assertEquals(want, l.material[l.at(100 - i)], i + " blocks below the top of the cover");
        }
        assertEquals(FarColumn.ROCK, l.material[l.at(92)], "the cover is thicker than the soil: rock under it");

        // A thin cover leaves the soil under it, without its top block.
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 100, Integer.MIN_VALUE, 1), soil(4, false, false), null, 8, spec, l);
        assertEquals(FarColumn.COVER_SNOW, l.material[l.at(100)]);
        assertEquals(FarColumn.SOIL_UNDER, l.material[l.at(99)]);
        assertEquals(FarColumn.SOIL_UNDER, l.material[l.at(97)]);
        assertEquals(FarColumn.ROCK, l.material[l.at(96)]);

        // No cover under water.
        FarColumn.Shape lakeShape = new FarColumn.Shape();
        FarColumn.shape(new Water.Core(Water.Kind.LAKE, 60.5, 60, 66.0, 65, 60, Rivers.Sample.NONE), 5, -64, 319, lakeShape);
        assertEquals(0, lakeShape.ice);
    }

    @Test
    void aColumnWithoutGroundIsWaterOrAir() {
        FarColumn.Layers l = new FarColumn.Layers();
        FarColumn.layers(-64, 319, shape(Water.Kind.SEA, -65, 62, 0), soil(4, false, false), null, 8, null, l);
        assertFills(l, -64, 319, "no ground under the sea");
        assertEquals(FarColumn.WATER, l.material[0]);
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, -65, Integer.MIN_VALUE, 3), soil(4, false, true), null, 8, null, l);
        assertFills(l, -64, 319, "no ground");
        assertEquals(1, l.count);
        // Terrain at the build limit: nothing above it.
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 400, Integer.MIN_VALUE, 0), soil(4, false, true), null, 8, null, l);
        assertFalse(l.count == 0);
        assertEquals(320, l.top[l.count - 1]);
        assertEquals(319, l.top[l.topSolid()] - 1, "the ground is cut at the build limit");
    }

    /** The surface rule's water in place of the top block (BAK-067: vanilla's swamp water at y 62). */
    @Test
    void paintedWaterTakesTheTopBlocksPlace() {
        FarColumn.Layers l = new FarColumn.Layers();
        FarColumn.Soil swamp = soil(4, false, false);
        swamp.paintedWater = true;
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 62, Integer.MIN_VALUE, 0), swamp, null, 8, null, l);
        assertFills(l, -64, 319, "painted water on dry ground");
        assertEquals(List.of(FarColumn.ROCK, FarColumn.SOIL_UNDER, FarColumn.WATER, FarColumn.AIR),
                java.util.Arrays.stream(materials(l)).boxed().toList());
        assertEquals(59, l.bottom[1], "the soil keeps its depth from the ground's top");
        assertEquals(62, l.bottom[2]);
        assertEquals(63, l.bottom[3]);
        assertEquals(61, l.top[l.topSolid()] - 1, "the ground under the water is one lower");

        // Frozen: the water's top freezes as a lake's does, and no snow lies on it.
        swamp.frozen = true;
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 62, Integer.MIN_VALUE, 0), swamp, null, 8, null, l);
        assertEquals(FarColumn.WATER_ICE, l.material[l.at(62)]);
        assertEquals(FarColumn.AIR, l.material[l.at(63)]);
        swamp.frozen = false;

        // A bare biome: rock under the water.
        FarColumn.Soil bare = soil(4, true, false);
        bare.paintedWater = true;
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 62, Integer.MIN_VALUE, 0), bare, null, 8, null, l);
        assertEquals(List.of(FarColumn.ROCK, FarColumn.WATER, FarColumn.AIR), java.util.Arrays.stream(materials(l)).boxed().toList());
        assertEquals(62, l.top[0]);

        // No ice cover where the surface is water (the ice pass leaves a fluid surface alone).
        IceCover.Spec spec = new IceCover.Spec(50.0, 12, 2, 6, 2, "minecraft:snow_block", "minecraft:packed_ice", "minecraft:blue_ice");
        FarColumn.layers(-64, 319, shape(Water.Kind.NONE, 62, Integer.MIN_VALUE, 3), swamp, null, 8, spec, l);
        assertFills(l, -64, 319, "painted water under an ice cap's spec");
        assertEquals(FarColumn.WATER, l.material[l.at(62)]);
        assertEquals(FarColumn.SOIL_UNDER, l.material[l.at(61)]);

        // Under standing water the painted block joins it (a mangrove floor at y 60): the bed is one lower.
        FarColumn.layers(-64, 319, shape(Water.Kind.SEA, 60, 62, 0), swamp, null, 8, null, l);
        assertFills(l, -64, 319, "painted water under the sea");
        assertEquals(FarColumn.SOIL_UNDER, l.material[l.at(59)]);
        assertEquals(FarColumn.WATER, l.material[l.at(60)]);
        assertEquals(l.at(60), l.at(62), "one body of water from the painted block up");
        assertEquals(63, l.top[l.at(62)]);
        assertEquals(59, l.top[l.topSolid()] - 1);

        // A bank at the water beside it is sealed (the bank pass fills painted water); one above it, or land, is not.
        FarColumn.Shape bank = new FarColumn.Shape();
        FarColumn.shape(new Water.Column(Water.Kind.BANK, 61.5, 61, null, null, null, 62, null, null, false, null), 0, -64, 319, bank);
        assertEquals(62, bank.ground);
        assertTrue(bank.sealed, "raised to minGround");
        FarColumn.shape(new Water.Column(Water.Kind.BANK, 62.5, 62, null, null, null, 62, null, null, false, null), 0, -64, 319, bank);
        assertTrue(bank.sealed, "standing at minGround");
        FarColumn.shape(new Water.Column(Water.Kind.BANK, 63.5, 63, null, null, null, 62, null, null, false, null), 0, -64, 319, bank);
        assertFalse(bank.sealed, "above minGround");
        FarColumn.shape(new Water.Column(Water.Kind.NONE, 62.5, 62, null, null, null, null, null, null, false, null), 0, -64, 319, bank);
        assertFalse(bank.sealed, "land");
    }

    // ---- Threads and time ----

    /** A hash of every layer of a section, built the way the game builds it (without the biome). */
    static long section(WorldData wd, long x0, long z0, int step, FarColumn.Shape[] grid, Columns.Stack stack, FarColumn.Layers layers) {
        Water.Spec spec = spec(wd);
        RasterSampler strata = wd.has(WorldData.HORIZONS) ? wd.layer(WorldData.HORIZONS) : null;
        IceCover.Spec cover = coverSpec(wd);
        FarColumn.Soil soil = soil(4, false, false);
        FarColumn.grid(wd, spec, x0, z0, step, 64, grid);
        long hash = 1125899906842597L;
        int half = step / 2;
        for (int j = 0; j < 64; j++) {
            for (int i = 0; i < 64; i++) {
                FarColumn.Shape s = grid[(j + 1) * 66 + (i + 1)];
                if (strata != null)
                    FarColumn.rock(wd, strata, FarColumn.wrapX(wd, x0 + (long) i * step + half), z0 + (long) j * step + half, step, stack);
                soil.frozen = s.ice > 0;
                FarColumn.layers(spec.minY(), spec.maxY(), s, soil, strata == null ? null : stack,
                        Math.min(256, FarColumn.exposure(grid, 64, i, j) + 8), cover, layers);
                for (int k = 0; k < layers.count; k++)
                    hash = 31 * (31 * (31 * (31 * hash + layers.bottom[k]) + layers.material[k]) + layers.cls[k]) + layers.unit[k];
            }
        }
        return hash;
    }

    @Test
    void threadsAgree() throws Exception {
        List<long[]> jobs = new ArrayList<>();
        for (int step : new int[]{1, 4})
            for (long[] a : areas(planet, step))
                jobs.add(new long[]{a[0], a[1], step});
        long[] reference = new long[jobs.size()];
        FarColumn.Shape[] grid = shapes(64);
        Columns.Stack stack = new Columns.Stack(8);
        FarColumn.Layers layers = new FarColumn.Layers();
        for (int k = 0; k < jobs.size(); k++)
            reference[k] = section(planet, jobs.get(k)[0], jobs.get(k)[1], (int) jobs.get(k)[2], grid, stack, layers);
        ExecutorService pool = Executors.newFixedThreadPool(6);
        try {
            List<Future<long[]>> runs = new ArrayList<>();
            for (int t = 0; t < 6; t++) {
                int offset = t;
                runs.add(pool.submit(() -> {
                    FarColumn.Shape[] g = shapes(64);
                    Columns.Stack st = new Columns.Stack(8);
                    FarColumn.Layers l = new FarColumn.Layers();
                    long[] out = new long[jobs.size()];
                    for (int n = 0; n < jobs.size(); n++) {
                        int k = (n + offset * 5) % jobs.size();   // every thread in its own order
                        out[k] = section(planet, jobs.get(k)[0], jobs.get(k)[1], (int) jobs.get(k)[2], g, st, l);
                    }
                    return out;
                }));
            }
            for (Future<long[]> run : runs) {
                long[] got = run.get();
                for (int k = 0; k < reference.length; k++)
                    assertEquals(reference[k], got[k], "section " + java.util.Arrays.toString(jobs.get(k)) + " differs on another thread");
            }
        } finally {
            pool.shutdownNow();
        }
    }

    /**
     * The time a section of 64 × 64 columns takes, without the biome lookup and Distant Horizons'
     * own storing ({@code /deeptime probe far bench} and {@code probe dh} measure those in the
     * game). Printed as {@code [bench] far section …}; the test only checks sections were built.
     */
    @Test
    void sectionTime() {
        FarColumn.Shape[] grid = shapes(64);
        Columns.Stack stack = new Columns.Stack(8);
        FarColumn.Layers layers = new FarColumn.Layers();
        int c = planet.circumference();
        for (int step : new int[]{1, 2, 4, 16, 64}) {
            long span = 64L * step;
            if (span > c)
                continue;
            double best = Double.MAX_VALUE;
            int sections = 0;
            long sink = 0;
            for (int pass = 0; pass < 3; pass++) {
                long t0 = System.nanoTime();
                sections = 0;
                for (long z = -c / 4; z < c / 4; z += Math.max(span, c / 16))
                    for (long x = -c / 2; x < c / 2; x += Math.max(span, c / 16)) {
                        sink += section(planet, x, z, step, grid, stack, layers);
                        sections++;
                    }
                best = Math.min(best, (System.nanoTime() - t0) / 1e6 / sections);
            }
            assertTrue(sections > 0 && sink != 0);
            // Where the time goes: the geometry alone, and the strata rule alone, over the same sections.
            Water.Spec spec = spec(planet);
            RasterSampler strata = planet.layer(WorldData.HORIZONS);
            double gridMs = Double.MAX_VALUE, stackMs = Double.MAX_VALUE;
            for (int pass = 0; pass < 2; pass++) {
                long t0 = System.nanoTime();
                for (long z = -c / 4; z < c / 4; z += Math.max(span, c / 16))
                    for (long x = -c / 2; x < c / 2; x += Math.max(span, c / 16))
                        FarColumn.grid(planet, spec, x, z, step, 64, grid);
                gridMs = Math.min(gridMs, (System.nanoTime() - t0) / 1e6 / sections);
                t0 = System.nanoTime();
                for (long z = -c / 4; z < c / 4; z += Math.max(span, c / 16))
                    for (long x = -c / 2; x < c / 2; x += Math.max(span, c / 16))
                        for (int j = 0; j < 64; j++)
                            for (int i = 0; i < 64; i++)
                                FarColumn.rock(planet, strata, FarColumn.wrapX(planet, x + (long) i * step + step / 2),
                                        z + (long) j * step + step / 2, step, stack);
                stackMs = Math.min(stackMs, (System.nanoTime() - t0) / 1e6 / sections);
            }
            System.out.printf("[bench] far section step=%d (%d blocks wide) ms/section=%.3f us/column=%.3f sections=%d "
                            + "(geometry %.3f ms, strata %.3f ms)%n", step, span, best, best * 1000.0 / 4096.0, sections, gridMs, stackMs);
        }
    }
}
