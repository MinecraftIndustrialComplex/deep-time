// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

import org.junit.jupiter.api.Test;

import java.io.File;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import java.net.URL;
import java.net.URLClassLoader;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import java.util.function.Consumer;
import java.util.jar.JarFile;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The run-time binding against the real Distant Horizons jars (plan §10.5 P3-04: "the exact DH
 * version matters"). Gradle resolves the jars from Modrinth ({@code dh_version}, the MIC pack's
 * pin, and {@code dh_next_version}) and names them in {@code deeptime.test.dhJars}; each is opened
 * in its own class loader, with no Minecraft on it, which the API classes do not need.
 *
 * <p>For every jar: the binding resolves under the naming the jar really has; a
 * {@link DhLodGenerator} proxy implements that jar's {@code IDhApiWorldGenerator}; and a
 * {@code generateLod} call made through the jar's own interface method fills a data source with
 * columns Distant Horizons would accept (its own rules, restated here: block-sized points, no gap,
 * no overlap, heights in range, y counted from the level's bottom).
 */
class DhApiBindingTest {

    static Map<String, File> jars() {
        String prop = System.getProperty("deeptime.test.dhJars", "");
        Map<String, File> out = new HashMap<>();
        for (String entry : prop.split(File.pathSeparator)) {
            if (entry.isBlank())
                continue;
            int eq = entry.indexOf('=');
            out.put(entry.substring(0, eq), new File(entry.substring(eq + 1)));
        }
        return out;
    }

    static boolean has(File jar, String entry) throws Exception {
        try (JarFile jf = new JarFile(jar)) {
            return jf.getEntry(entry) != null;
        }
    }

    @Test
    void theJarsAreThere() {
        Map<String, File> jars = jars();
        assertTrue(jars.size() >= 2, "deeptime.test.dhJars names " + jars.keySet() + ": run through Gradle's test task");
        jars.forEach((name, file) -> assertTrue(file.isFile(), name + ": " + file + " is missing"));
    }

    @Test
    void bindsUnderTheNamesEachJarHas() throws Exception {
        for (Map.Entry<String, File> e : jars().entrySet()) {
            File jar = e.getValue();
            boolean merged = has(jar, "com/seibel/distanthorizons/api/DhApi_neoforge.class");
            try (URLClassLoader loader = new URLClassLoader(new URL[]{jar.toURI().toURL()}, ClassLoader.getPlatformClassLoader())) {
                StringBuilder why = new StringBuilder();
                DhApiBinding api = DhApiBinding.find(loader, why).orElse(null);
                assertNotNull(api, e.getKey() + ": no binding (" + why + ")");
                assertEquals(merged ? DhApiBinding.MERGED_SUFFIX : "", api.suffix(), e.getKey() + ": the naming");
                assertEquals("com.seibel.distanthorizons.api.interfaces.override.worldGenerator.IDhApiWorldGenerator" + api.suffix(),
                        api.worldGenerator.getName());
                assertTrue(api.worldGenerator.isInterface());
                System.out.printf("[dh] %s (%s): %s%n", e.getKey(), jar.getName(), api.naming());
            }
        }
    }

    /** A loader without Distant Horizons has no binding, and says why. */
    @Test
    void noDistantHorizonsNoBinding() {
        StringBuilder why = new StringBuilder();
        assertTrue(DhApiBinding.find(ClassLoader.getPlatformClassLoader(), why).isEmpty());
        assertTrue(why.toString().contains("DhApi"), why.toString());
    }

    /** What a fake data source recorded. */
    static final class Recorded {
        final Map<Long, List<Object>> columns = new HashMap<>();
        final List<Object> steps = new ArrayList<>();
    }

    @Test
    void aGeneratorFillsADataSourceThroughTheRealInterface() throws Exception {
        for (Map.Entry<String, File> e : jars().entrySet()) {
            try (URLClassLoader loader = new URLClassLoader(new URL[]{e.getValue().toURI().toURL()}, ClassLoader.getPlatformClassLoader())) {
                DhApiBinding api = DhApiBinding.find(loader, null).orElseThrow();
                String s = api.suffix(), base = DhApiBinding.API;
                Class<?> blockType = Class.forName(base + "interfaces.block.IDhApiBlockStateWrapper" + s, false, loader);
                Class<?> biomeType = Class.forName(base + "interfaces.block.IDhApiBiomeWrapper" + s, false, loader);
                Class<?> pointType = Class.forName(base + "objects.data.DhApiTerrainDataPoint" + s, false, loader);
                Class<?> modeType = Class.forName(base + "enums.worldGeneration.EDhApiDistantGeneratorMode" + s, false, loader);
                Object mode = modeType.getField("FEATURES").get(null);

                // Stand-ins for the game's blocks and biome, and for Distant Horizons' wrappers of them.
                Map<Object, Object> wrapped = new HashMap<>();
                DhLodGenerator.Wrappers wrappers = new DhLodGenerator.Wrappers() {
                    @Override
                    public synchronized Object block(Object blockState) {
                        return wrapped.computeIfAbsent(blockState, k -> Proxy.newProxyInstance(loader, new Class<?>[]{blockType},
                                (p, m, a) -> m.getName().equals("toString") ? "block " + k : m.getName().equals("hashCode") ? k.hashCode()
                                        : m.getName().equals("equals") ? p == a[0] : null));
                    }

                    @Override
                    public synchronized Object biome(Object biomeHolder) {
                        return wrapped.computeIfAbsent(biomeHolder, k -> Proxy.newProxyInstance(loader, new Class<?>[]{biomeType},
                                (p, m, a) -> m.getName().equals("toString") ? "biome " + k : m.getName().equals("hashCode") ? k.hashCode()
                                        : m.getName().equals("equals") ? p == a[0] : null));
                    }

                    @Override
                    public int minY() {
                        return -512;
                    }

                    @Override
                    public int height() {
                        return 1152;
                    }
                };
                // Columns whose ground rises with x and z: rock, soil, air, in world y.
                DhLodGenerator.Source source = (x0, z0, step, n, sink) -> {
                    int[] bottom = new int[3], top = new int[3], light = {0, 15, 15};
                    Object[] block = {"rock", "soil", "air"};
                    for (int j = 0; j < n; j++)
                        for (int i = 0; i < n; i++) {
                            int ground = -300 + ((x0 + i * step) & 255) + ((z0 + j * step) & 127);
                            bottom[0] = -512;
                            top[0] = ground - 3;
                            bottom[1] = ground - 3;
                            top[1] = ground + 1;
                            bottom[2] = ground + 1;
                            top[2] = 640;
                            sink.column(i, j, 3, bottom, top, block, light, "plains");
                        }
                };
                DhLodGenerator generator = new DhLodGenerator(api, source, wrappers, 12, true, "test");
                Object proxy = generator.proxy();
                assertTrue(api.worldGenerator.isInstance(proxy));
                // Toroidal World's guard looks at the class name: this must not look like Distant Horizons' own.
                assertFalse(proxy.getClass().getName().startsWith("com.seibel.distanthorizons."), proxy.getClass().getName());
                // ...so Deep Time's patch to that guard (TwDhGuard) recognises it by its handler, and nothing else.
                assertSame(generator, DhLodGenerator.of(proxy));
                assertEquals("test", generator.levelName());
                assertNull(DhLodGenerator.of(new Object()));
                assertNull(DhLodGenerator.of(null));
                assertNull(DhLodGenerator.of(java.lang.reflect.Proxy.newProxyInstance(api.worldGenerator.getClassLoader(),
                        new Class<?>[]{api.worldGenerator}, (self, m, a) -> null)), "another mod's proxy generator");

                assertSame(api.returnTypeDataSources, api.worldGenerator.getMethod("getReturnType").invoke(proxy));
                assertEquals("API_DATA_SOURCES", String.valueOf(api.returnTypeDataSources));
                assertEquals((byte) 0, api.worldGenerator.getMethod("getSmallestDataDetailLevel").invoke(proxy));
                assertEquals((byte) 12, api.worldGenerator.getMethod("getLargestDataDetailLevel").invoke(proxy));
                assertEquals(true, api.worldGenerator.getMethod("runApiValidation").invoke(proxy));
                assertEquals(10, api.worldGenerator.getMethod("getPriority").invoke(proxy));
                api.worldGenerator.getMethod("preGeneratorTaskStart").invoke(proxy);
                api.worldGenerator.getMethod("close").invoke(proxy);
                assertTrue(proxy.equals(proxy));
                assertTrue(proxy.toString().contains("Deep Time"));
                assertEquals(proxy.hashCode(), proxy.hashCode());

                Method generateLod = null;
                for (Method m : api.worldGenerator.getMethods())
                    if (m.getName().equals("generateLod"))
                        generateLod = m;
                assertNotNull(generateLod);
                ExecutorService pool = Executors.newFixedThreadPool(3);
                try {
                    for (byte detail : new byte[]{0, 3}) {
                        Recorded rec = new Recorded();
                        Object dataSource = Proxy.newProxyInstance(loader, new Class<?>[]{api.fullDataSource}, (p, m, a) -> {
                            switch (m.getName()) {
                                case "getWidthInDataColumns":
                                    return 64;
                                case "setApiDataPointColumn":
                                    synchronized (rec) {
                                        List<?> list = (List<?>) a[a.length - 1];
                                        rec.columns.put(((long) (Integer) a[0] << 32) | (Integer) a[1], new ArrayList<>(list));
                                        rec.steps.add(a.length == 4 ? a[2] : null);
                                    }
                                    return a[a.length - 1];
                                default:
                                    return null;
                            }
                        });
                        boolean[] told = {false};
                        Consumer<Object> done = d -> told[0] = d == dataSource;
                        @SuppressWarnings("unchecked")
                        CompletableFuture<Void> future = (CompletableFuture<Void>) generateLod.invoke(proxy, 4, -8, 1, -2, detail,
                                dataSource, mode, pool, done);
                        future.get(60, TimeUnit.SECONDS);
                        assertTrue(told[0], "the result consumer got the data source");
                        assertEquals(4096, rec.columns.size(), e.getKey() + " detail " + detail + ": every column was set");
                        assertTrue(rec.steps.stream().allMatch(st -> st == api.stepLight), "columns are written as finished (LIGHT)");
                        int step = 1 << detail;
                        for (Map.Entry<Long, List<Object>> col : rec.columns.entrySet()) {
                            int i = (int) (col.getKey() >> 32), j = (int) (long) col.getKey();
                            int ground = -300 + ((64 + i * step) & 255) + ((-128 + j * step) & 127);
                            List<Object> points = col.getValue();
                            assertEquals(3, points.size());
                            int last = 0;   // y counts from the level's bottom
                            for (Object p : points) {
                                assertTrue(pointType.isInstance(p));
                                assertEquals((byte) 0, pointType.getField("detailLevel").getByte(p), "block-sized points");
                                int b = pointType.getField("bottomYBlockPos").getInt(p), t = pointType.getField("topYBlockPos").getInt(p);
                                assertEquals(last, b, "no gap and no overlap");
                                assertTrue(t > b && t - b < 6095, "a height Distant Horizons accepts");
                                last = t;
                            }
                            assertEquals(1152, last, "the column reaches the level's top");
                            Object soil = points.get(1);
                            assertEquals(ground + 1 + 512, pointType.getField("topYBlockPos").getInt(soil), "the ground, counted from the bottom");
                            assertEquals(15, pointType.getField("skyLightLevel").getInt(soil));
                            assertEquals("block soil", String.valueOf(pointType.getField("blockStateWrapper").get(soil)));
                            assertEquals("biome plains", String.valueOf(pointType.getField("biomeWrapper").get(soil)));
                        }
                        assertEquals(0, generator.failures());
                    }
                    assertEquals(2, generator.sections());
                    assertTrue(generator.meanMillis() >= 0.0);
                    assertEquals(3.0, generator.meanPointsPerColumn(), 1e-9);
                    System.out.printf("[dh] %s: generateLod through %s filled 2 × 4096 columns (%s)%n", e.getKey(),
                            api.worldGenerator.getSimpleName(), generator.byDetail());
                } finally {
                    pool.shutdownNow();
                }
            }
        }
    }
}
