// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.atomic.AtomicLongArray;
import java.util.concurrent.atomic.LongAdder;
import java.util.function.Consumer;

/**
 * Deep Time's world generator as Distant Horizons sees it (plan §10.5 P3-04): an implementation of
 * DH's {@code IDhApiWorldGenerator} that answers {@code generateLod} from far-terrain columns
 * instead of generating chunks. It is a {@link Proxy} of the interface {@link DhApiBinding} found,
 * because that interface has two names ({@link DhApiBinding}).
 *
 * <p><b>What DH asks.</b> {@code generateLod(chunkPosMinX, chunkPosMinZ, lodPosX, lodPosZ,
 * detailLevel, dataSource, mode, executor, consumer)}: fill a data source of 64 × 64 columns, each
 * {@code 2^detailLevel} blocks wide, whose minimum corner is block (16·chunkPosMinX,
 * 16·chunkPosMinZ). The generator says which detail levels it takes
 * ({@code getSmallest/LargestDataDetailLevel}); DH asks coarse levels far away and level 0 near, so
 * a request always costs 4,096 columns, whatever area it covers. The work runs on the executor DH
 * passes (its world-generator pool), so DH's thread settings govern it.
 *
 * <p><b>What it answers.</b> Per column a list of data points bottom-up, without gaps, from the
 * level's bottom to its top (air above the ground: DH asks for it, "so light propagates"), each
 * with a block, the column's biome and a sky light. y counts from the level's minimum height
 * ({@link DhApiBinding}). Columns are written with generation step {@code LIGHT}, the step of a
 * finished chunk, so DH never queues them again for its own chunk generation.
 *
 * <p>Thread-safe: DH calls {@code generateLod} from several threads; every request has its own
 * list, the wrapper caches are concurrent, and {@link Source} implementations are thread-safe.
 * Plain Java: blocks and biomes pass through as objects.
 */
public final class DhLodGenerator implements InvocationHandler {

    /** Where the columns come from ({@code farterrain/FarTerrain} in the game). */
    public interface Source {
        /** Sends the n × n columns, each {@code step} blocks wide, of the area whose minimum corner is block (x0, z0). */
        void build(int x0, int z0, int step, int n, ColumnSink sink);
    }

    /** Receives one column: {@code count} layers bottom-up, layer i filling {@code bottom[i] <= y < top[i]} in world y. */
    public interface ColumnSink {
        void column(int relX, int relZ, int count, int[] bottom, int[] top, Object[] block, int[] skyLight, Object biome);
    }

    /** Distant Horizons' wrappers of the game's blocks and biomes, and the level's y range. */
    public interface Wrappers {
        Object block(Object blockState) throws ReflectiveOperationException;

        Object biome(Object biomeHolder) throws ReflectiveOperationException;

        /** The level's minimum y: data points count from it. */
        int minY();

        /** The level's height in blocks. */
        int height();
    }

    /** DH's own generator takes data detail levels 0 to 12 (columns of 1 to 4,096 blocks). */
    public static final int DEFAULT_MAX_DETAIL = 12;

    private final DhApiBinding api;
    private final Source source;
    private final Wrappers wrappers;
    private final byte maxDetail;
    private final boolean validate;
    private final String name;
    private final Object proxy;
    private final Map<Object, Object> blockWrappers = new ConcurrentHashMap<>(), biomeWrappers = new ConcurrentHashMap<>();

    private final LongAdder sections = new LongAdder(), nanos = new LongAdder(), points = new LongAdder(), failures = new LongAdder();
    private final AtomicLongArray sectionsByDetail = new AtomicLongArray(32), nanosByDetail = new AtomicLongArray(32);
    private volatile String firstFailure;
    /** The x extent of the sections asked so far (block x, end exclusive): shows requests from both sides of the seam. */
    private final java.util.concurrent.atomic.AtomicInteger minBlockX = new java.util.concurrent.atomic.AtomicInteger(Integer.MAX_VALUE),
            maxBlockX = new java.util.concurrent.atomic.AtomicInteger(Integer.MIN_VALUE);

    public DhLodGenerator(DhApiBinding api, Source source, Wrappers wrappers, int maxDetail, boolean validate, String name) {
        this.api = api;
        this.source = source;
        this.wrappers = wrappers;
        this.maxDetail = (byte) Math.max(0, Math.min(24, maxDetail));
        this.validate = validate;
        this.name = name;
        this.proxy = Proxy.newProxyInstance(api.worldGenerator.getClassLoader(), new Class<?>[]{api.worldGenerator}, this);
    }

    /** The object to register with Distant Horizons. */
    public Object proxy() {
        return proxy;
    }

    /**
     * The Deep Time generator behind {@code generator} when it is one of {@link #proxy()}'s
     * objects, else null. Toroidal World's guard sees only the proxy ({@code TwDhGuard}).
     */
    public static DhLodGenerator of(Object generator) {
        if (generator != null && Proxy.isProxyClass(generator.getClass())
                && Proxy.getInvocationHandler(generator) instanceof DhLodGenerator g)
            return g;
        return null;
    }

    /** The level this generator was made for ({@code minecraft:overworld}). */
    public String levelName() {
        return name;
    }

    @Override
    @SuppressWarnings("unchecked")
    public Object invoke(Object self, Method method, Object[] args) throws Throwable {
        switch (method.getName()) {
            case "generateLod":
                if (args != null && args.length == 9)
                    return generateLod((Integer) args[0], (Integer) args[1], (Byte) args[4], args[5], (ExecutorService) args[7],
                            (Consumer<Object>) args[8]);
                break;
            case "getReturnType":
                return api.returnTypeDataSources;
            case "getSmallestDataDetailLevel":
                return (byte) 0;
            case "getLargestDataDetailLevel":
                return maxDetail;
            case "runApiValidation":
                return validate;
            case "getPriority":
                return 10;   // the API's default for an override that is not DH's own (which is −1)
            case "preGeneratorTaskStart":
            case "close":
            case "finishDelayedSetup":
                return null;
            case "getDelayedSetupComplete":
                return true;
            case "hashCode":
                if (args == null)
                    return System.identityHashCode(self);
                break;
            case "equals":
                if (args != null && args.length == 1)
                    return self == args[0];
                break;
            case "toString":
                if (args == null)
                    return "Deep Time far terrain (" + name + ")";
                break;
            default:
                break;
        }
        if (method.isDefault())
            return InvocationHandler.invokeDefault(self, method, args);
        throw new UnsupportedOperationException("Deep Time's far terrain does not implement " + method);
    }

    private CompletableFuture<Void> generateLod(int chunkMinX, int chunkMinZ, byte detail, Object dataSource,
                                                ExecutorService executor, Consumer<Object> done) {
        Runnable work = () -> {
            long t0 = System.nanoTime();
            try {
                fill(chunkMinX * 16, chunkMinZ * 16, detail, dataSource);
            } catch (Throwable t) {
                failures.increment();
                if (firstFailure == null)
                    firstFailure = "section at block " + chunkMinX * 16 + ", " + chunkMinZ * 16 + " detail " + detail + ": " + t;
                throw t instanceof RuntimeException r ? r : new IllegalStateException(t);
            }
            long dt = System.nanoTime() - t0;
            sections.increment();
            nanos.add(dt);
            int d = Math.max(0, Math.min(31, detail));
            sectionsByDetail.incrementAndGet(d);
            minBlockX.accumulateAndGet(chunkMinX * 16, Math::min);
            maxBlockX.accumulateAndGet(chunkMinX * 16 + (64 << d), Math::max);
            nanosByDetail.addAndGet(d, dt);
            if (done != null)
                done.accept(dataSource);
        };
        return executor == null ? CompletableFuture.runAsync(work) : CompletableFuture.runAsync(work, executor);
    }

    /** Fills {@code dataSource} (a DH {@code IDhApiFullDataSource}) for the area at block (x0, z0), columns {@code 2^detail} wide. */
    void fill(int x0, int z0, int detail, Object dataSource) throws Throwable {
        if (detail < 0 || detail > 24)
            throw new IllegalArgumentException("detail level " + detail);
        int n = (int) api.widthInColumns.invokeExact(dataSource);
        int minY = wrappers.minY(), height = wrappers.height();
        List<Object> column = new ArrayList<>(16);
        Throwable[] failure = {null};
        long[] made = {0};
        source.build(x0, z0, 1 << detail, n, (relX, relZ, count, bottom, top, block, skyLight, biome) -> {
            if (failure[0] != null)
                return;
            try {
                column.clear();
                Object biomeWrapper = biomeWrappers.get(biome);
                if (biomeWrapper == null) {
                    biomeWrapper = wrappers.biome(biome);
                    biomeWrappers.put(biome, biomeWrapper);
                }
                for (int k = 0; k < count; k++) {
                    int b = Math.max(0, bottom[k] - minY), t = Math.min(height, top[k] - minY);
                    if (t <= b)
                        continue;
                    Object blockWrapper = blockWrappers.get(block[k]);
                    if (blockWrapper == null) {
                        blockWrapper = wrappers.block(block[k]);
                        blockWrappers.put(block[k], blockWrapper);
                    }
                    column.add((Object) api.createPoint.invokeExact((byte) 0, 0, skyLight[k], b, t, blockWrapper, biomeWrapper));
                }
                made[0] += column.size();
                Object ignored = (Object) api.setColumn.invokeExact(dataSource, relX, relZ, api.stepLight, column);
            } catch (Throwable t) {
                failure[0] = t;
            }
        });
        points.add(made[0]);
        if (failure[0] != null)
            throw failure[0];
    }

    // ------------------------------------------------------------------------------------------
    // Measurements
    // ------------------------------------------------------------------------------------------

    public long sections() {
        return sections.sum();
    }

    public long failures() {
        return failures.sum();
    }

    /** The first failed request, or null. */
    public String firstFailure() {
        return firstFailure;
    }

    /** Mean milliseconds a section took, over every section so far; NaN for none. */
    public double meanMillis() {
        long n = sections.sum();
        return n == 0 ? Double.NaN : nanos.sum() / 1e6 / n;
    }

    /** Mean data points per column, over every section so far. */
    public double meanPointsPerColumn() {
        long n = sections.sum();
        return n == 0 ? Double.NaN : points.sum() / (n * 4096.0);
    }

    /** "detail 0: 1234 sections, 5.67 ms; detail 3: …" for the levels that were asked. */
    /** The x extent of the sections asked so far, "min..max" in blocks, or "none". */
    public String xExtent() {
        int lo = minBlockX.get(), hi = maxBlockX.get();
        return lo > hi ? "none" : lo + ".." + hi;
    }

    public String byDetail() {
        StringBuilder sb = new StringBuilder();
        for (int d = 0; d < 32; d++) {
            long n = sectionsByDetail.get(d);
            if (n == 0)
                continue;
            if (sb.length() > 0)
                sb.append("; ");
            sb.append(String.format(java.util.Locale.ROOT, "detail %d: %d sections, %.2f ms", d, n, nanosByDetail.get(d) / 1e6 / n));
        }
        return sb.toString();
    }

    public int maxDetail() {
        return maxDetail;
    }
}
