// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Distant Horizons' API (LGPL-3.0; NOTICE.md), bound by name at run time (plan §10.5 P3-04). Deep
 * Time calls the API and ships none of it.
 *
 * <p><b>Why by name.</b> The API classes have two names in the wild, and one jar cannot be compiled
 * against both:
 * <ul>
 *   <li><b>3.3.0</b> (the MIC pack's pin on 2026-10-01): the Fabric+NeoForge jar was merged with
 *       every class of the NeoForge build renamed {@code <Class>_neoforge} — the API included. On
 *       NeoForge the live API is {@code com.seibel.distanthorizons.api.DhApi_neoforge}; the
 *       unsuffixed {@code DhApi} in the same jar is the Fabric copy, which nothing on NeoForge
 *       reads, so a mod compiled against the published API registers into a dead object.</li>
 *   <li><b>3.3.1 and later</b> (checked: 3.3.1, 3.3.3): only the Minecraft-facing {@code common}
 *       classes keep the suffix; the API has its published names.</li>
 * </ul>
 * {@link #find} takes the suffixed API when the jar has one, else the plain one. The members used
 * are the same in 3.3.0 (API 7.1.0), 3.3.1 (7.1.0) and 3.3.3 (7.2.0), verified with {@code javap}
 * on the three jars; {@code DhApiBindingTest} loads the real 3.3.3 (the MIC pack's pin since pack
 * 62dd14e) and 3.3.1 jars and drives a generator through them.
 *
 * <p>Verified surface (package {@code com.seibel.distanthorizons.api}):
 * <pre>
 * DhApi.worldGenOverrides : IDhApiWorldGeneratorOverrideRegister
 *   registerWorldGeneratorOverride(IDhApiLevelWrapper, IDhApiWorldGenerator) : DhApiResult&lt;Void&gt;
 * DhApi.Delayed.{worldProxy, wrapperFactory, terrainRepo}
 * IDhApiWorldGenerator (interfaces.override.worldGenerator)
 *   getSmallestDataDetailLevel(), getLargestDataDetailLevel() : byte;  runApiValidation() : boolean
 *   getReturnType() : EDhApiWorldGeneratorReturnType  (API_DATA_SOURCES)
 *   generateLod(int chunkPosMinX, int chunkPosMinZ, int lodPosX, int lodPosZ, byte detailLevel,
 *               IDhApiFullDataSource, EDhApiDistantGeneratorMode, ExecutorService,
 *               Consumer&lt;IDhApiFullDataSource&gt;) : CompletableFuture&lt;Void&gt;
 *   preGeneratorTaskStart(), close(), getPriority() (default 10)
 * IDhApiFullDataSource (objects.data)
 *   getWidthInDataColumns() : int
 *   setApiDataPointColumn(int relX, int relZ, EDhApiWorldGenerationStep, List&lt;DhApiTerrainDataPoint&gt;)
 * DhApiTerrainDataPoint.create(byte detailLevel, int blockLight, int skyLight, int bottomY, int topY,
 *                              IDhApiBlockStateWrapper, IDhApiBiomeWrapper)
 * IDhApiWrapperFactory.getBlockStateWrapper(Object[], IDhApiLevelWrapper), getBiomeWrapper(Object[], IDhApiLevelWrapper)
 * IDhApiWorldProxy.worldLoaded(), getAllLoadedLevelWrappers()
 * IDhApiLevelWrapper.getWrappedMcObject(), getMinHeight(), getDhIdentifier()
 * IDhApiTerrainDataRepo.getColumnDataAtBlockPos(IDhApiLevelWrapper, int, int, IDhApiTerrainDataCache)
 * DhApi.Delayed.configs : IDhApiConfig (null until DH starts its API)
 *   graphics() : IDhApiGraphicsConfig;  earthCurvatureRatio() : IDhApiConfigValue&lt;Integer&gt;  (client; {@link DhCurvature})
 *   worldGenerator() : IDhApiWorldGenerationConfig (interfaces.config.both);
 *     enableDistantWorldGeneration() : IDhApiConfigValue&lt;Boolean&gt;  ({@link DhGenerationGate})
 * IDhApiConfigValue: getTrueValue() (the player's file), getApiValue() (the override layer), getDefaultValue(),
 *   getCanBeOverrodeByApi(), setValue(T) and clearValue() (override layer only, never saved)
 * </pre>
 *
 * <p><b>The y of a data source's points</b> is counted from the level's minimum height, not from
 * y 0: {@code FullDataSourceV2.setApiDataPointColumn} packs the points with an offset of 0 (the API
 * chunk path subtracts the chunk's bottom instead), and DH's own generator subtracts
 * {@code getMinHeight()} before it writes. A release build does not validate the range, so a
 * negative y would wrap silently. {@link DhLodGenerator} subtracts the level's minimum height.
 *
 * <p>Plain Java: no Minecraft and no Distant Horizons type appears in a signature here.
 */
public final class DhApiBinding {

    public static final String API = "com.seibel.distanthorizons.api.";
    /** The suffix the 3.3.0 merge gave the NeoForge build's classes. */
    public static final String MERGED_SUFFIX = "_neoforge";

    /** What a registration answered. */
    public record Result(boolean success, String message) {}

    private final ClassLoader loader;
    private final String suffix;
    final Class<?> worldGenerator;
    final Class<?> fullDataSource;
    final Class<?> levelWrapper;
    final Object returnTypeDataSources;
    final Object stepLight;
    /** (byte, int, int, int, int, Object block, Object biome) → Object. */
    final MethodHandle createPoint;
    /** (Object dataSource, int, int, Object step, List) → Object. */
    final MethodHandle setColumn;
    /** (Object dataSource) → int. */
    final MethodHandle widthInColumns;

    private DhApiBinding(ClassLoader loader, String suffix) throws ReflectiveOperationException {
        this.loader = loader;
        this.suffix = suffix;
        this.worldGenerator = api("interfaces.override.worldGenerator.IDhApiWorldGenerator");
        this.fullDataSource = api("objects.data.IDhApiFullDataSource");
        this.levelWrapper = api("interfaces.world.IDhApiLevelWrapper");
        Class<?> returnType = api("enums.worldGeneration.EDhApiWorldGeneratorReturnType");
        this.returnTypeDataSources = constant(returnType, "API_DATA_SOURCES");
        Class<?> step = api("enums.worldGeneration.EDhApiWorldGenerationStep");
        this.stepLight = constant(step, "LIGHT");
        Class<?> point = api("objects.data.DhApiTerrainDataPoint");
        Class<?> block = api("interfaces.block.IDhApiBlockStateWrapper");
        Class<?> biome = api("interfaces.block.IDhApiBiomeWrapper");
        MethodHandles.Lookup lookup = MethodHandles.publicLookup();
        this.createPoint = lookup.findStatic(point, "create",
                        MethodType.methodType(point, byte.class, int.class, int.class, int.class, int.class, block, biome))
                .asType(MethodType.methodType(Object.class, byte.class, int.class, int.class, int.class, int.class, Object.class, Object.class));
        this.setColumn = lookup.findVirtual(fullDataSource, "setApiDataPointColumn",
                        MethodType.methodType(List.class, int.class, int.class, step, List.class))
                .asType(MethodType.methodType(Object.class, Object.class, int.class, int.class, Object.class, List.class));
        this.widthInColumns = lookup.findVirtual(fullDataSource, "getWidthInDataColumns", MethodType.methodType(int.class))
                .asType(MethodType.methodType(int.class, Object.class));
        // The members the generator answers itself must exist under these names, or a renamed API
        // would be answered with UnsupportedOperationException at the first request instead of here.
        for (String name : new String[]{"getSmallestDataDetailLevel", "getLargestDataDetailLevel", "runApiValidation",
                "getReturnType", "preGeneratorTaskStart", "close", "getPriority"})
            method(worldGenerator, name, 0);
        method(worldGenerator, "generateLod", 9);
    }

    /**
     * The binding for the Distant Horizons {@code loader} can see: the merged ({@code _neoforge})
     * API when it is there, else the published names; empty, with the reason in {@code why}, when
     * neither resolves.
     */
    public static Optional<DhApiBinding> find(ClassLoader loader, StringBuilder why) {
        List<String> failures = new ArrayList<>();
        for (String suffix : new String[]{MERGED_SUFFIX, ""}) {
            try {
                Class.forName(API + "DhApi" + suffix, false, loader);
            } catch (ClassNotFoundException | LinkageError e) {
                failures.add("no " + API + "DhApi" + suffix);
                continue;
            }
            try {
                return Optional.of(new DhApiBinding(loader, suffix));
            } catch (ReflectiveOperationException | LinkageError e) {
                failures.add("'" + suffix + "' names: " + e);
            }
        }
        if (why != null)
            why.append(String.join("; ", failures));
        return Optional.empty();
    }

    /** {@code _neoforge} for the 3.3.0 merge, empty for the published names. */
    public String suffix() {
        return suffix;
    }

    /** "published names" or "merged names (_neoforge)". */
    public String naming() {
        return suffix.isEmpty() ? "published names" : "merged names (" + suffix + ")";
    }

    private Class<?> api(String name) throws ClassNotFoundException {
        return Class.forName(API + name + suffix, false, loader);
    }

    private static Object constant(Class<?> enumClass, String name) throws ReflectiveOperationException {
        return enumClass.getField(name).get(null);
    }

    private static Method method(Class<?> owner, String name, int params) throws NoSuchMethodException {
        for (Method m : owner.getMethods())
            if (m.getName().equals(name) && m.getParameterCount() == params)
                return m;
        throw new NoSuchMethodException(owner.getName() + "." + name + "/" + params);
    }

    // ------------------------------------------------------------------------------------------
    // Run-time access (initialises DhApi, so only inside the game).
    // ------------------------------------------------------------------------------------------

    private Object delayed(String field) throws ReflectiveOperationException {
        return Class.forName(API + "DhApi$Delayed" + suffix, true, loader).getField(field).get(null);
    }

    private static final String WORLD_PROXY = "interfaces.world.IDhApiWorldProxy", LEVEL = "interfaces.world.IDhApiLevelWrapper",
            UNSAFE = "interfaces.IDhApiUnsafeWrapper", FACTORY = "interfaces.factories.IDhApiWrapperFactory",
            REGISTER = "interfaces.override.worldGenerator.IDhApiWorldGeneratorOverrideRegister",
            REPO = "interfaces.data.IDhApiTerrainDataRepo";

    /**
     * Calls the public API method {@code name} of interface {@code iface} (an API class name without
     * the package prefix) on {@code target}. The method is looked up on the interface, never on the
     * implementation class, so nothing here needs Distant Horizons' internals to be accessible.
     */
    private Object call(String iface, Object target, String name, Object... args) throws ReflectiveOperationException {
        Class<?> type = api(iface);
        for (Method m : type.getMethods()) {
            if (!m.getName().equals(name) || m.getParameterCount() != args.length)
                continue;
            Class<?>[] p = m.getParameterTypes();
            boolean fits = true;
            for (int i = 0; i < p.length && fits; i++)
                fits = args[i] == null || boxed(p[i]).isInstance(args[i]);
            if (fits)
                return m.invoke(target, args);
        }
        throw new NoSuchMethodException(type.getName() + "." + name + "/" + args.length);
    }

    private static Class<?> boxed(Class<?> c) {
        return c == int.class ? Integer.class : c == boolean.class ? Boolean.class : c == byte.class ? Byte.class : c;
    }

    /** Distant Horizons' version and API version, e.g. {@code 3.3.0 (API 7.1.0)}. */
    public String version() {
        try {
            Class<?> api = Class.forName(API + "DhApi" + suffix, true, loader);
            return api.getMethod("getModVersion").invoke(null) + " (API " + api.getMethod("getApiMajorVersion").invoke(null) + "."
                    + api.getMethod("getApiMinorVersion").invoke(null) + "." + api.getMethod("getApiPatchVersion").invoke(null) + ")";
        } catch (ReflectiveOperationException | LinkageError e) {
            return "unknown (" + e + ")";
        }
    }

    /** The level wrappers Distant Horizons has loaded; empty before its world exists. */
    public List<Object> loadedLevelWrappers() throws ReflectiveOperationException {
        Object proxy = delayed("worldProxy");
        List<Object> out = new ArrayList<>();
        if (proxy == null || !(Boolean) call(WORLD_PROXY, proxy, "worldLoaded"))
            return out;
        for (Object w : (Iterable<?>) call(WORLD_PROXY, proxy, "getAllLoadedLevelWrappers"))
            out.add(w);
        return out;
    }

    /** The Minecraft level a level wrapper wraps. */
    public Object wrappedLevel(Object levelWrapper) throws ReflectiveOperationException {
        return call(UNSAFE, levelWrapper, "getWrappedMcObject");
    }

    public int minHeight(Object levelWrapper) throws ReflectiveOperationException {
        return (Integer) call(LEVEL, levelWrapper, "getMinHeight");
    }

    public String levelId(Object levelWrapper) {
        try {
            return String.valueOf(call(LEVEL, levelWrapper, "getDhIdentifier"));
        } catch (ReflectiveOperationException e) {
            return String.valueOf(levelWrapper);
        }
    }

    /** Distant Horizons' wrapper of a Minecraft {@code BlockState}. */
    public Object blockWrapper(Object blockState, Object levelWrapper) throws ReflectiveOperationException {
        return call(FACTORY, delayed("wrapperFactory"), "getBlockStateWrapper", new Object[]{blockState}, levelWrapper);
    }

    /**
     * Distant Horizons' wrapper of a Minecraft {@code Holder<Biome>}. The factory has two
     * {@code getBiomeWrapper} overloads (an {@code Object[]} and a {@code String}); the argument's
     * type picks the first.
     */
    public Object biomeWrapper(Object biomeHolder, Object levelWrapper) throws ReflectiveOperationException {
        return call(FACTORY, delayed("wrapperFactory"), "getBiomeWrapper", new Object[]{biomeHolder}, levelWrapper);
    }

    /** Registers {@code generator} (a proxy of {@link #worldGenerator}) as the level's world generator. */
    public Result register(Object levelWrapper, Object generator) throws ReflectiveOperationException {
        Field f = Class.forName(API + "DhApi" + suffix, true, loader).getField("worldGenOverrides");
        Object result = call(REGISTER, f.get(null), "registerWorldGeneratorOverride", levelWrapper, generator);
        return new Result((Boolean) result.getClass().getField("success").get(result),
                String.valueOf(result.getClass().getField("message").get(result)));
    }

    private static final String CONFIG = "interfaces.config.IDhApiConfig", GRAPHICS = "interfaces.config.client.IDhApiGraphicsConfig",
            CONFIG_VALUE = "interfaces.config.IDhApiConfigValue";

    /**
     * Distant Horizons' earth-curvature setting ({@code DhApi.Delayed.configs.graphics().
     * earthCurvatureRatio()}, the client's world curvature) as a {@link DhCurvature.Slot}; null
     * before Distant Horizons has started its API ({@code DhApi.Delayed.configs} is null until then).
     * Reads and API-layer writes only: {@code getTrueValue}, {@code getApiValue},
     * {@code getCanBeOverrodeByApi}, {@code setValue} (which, on the API layer, never saves) and
     * {@code clearValue}. The config file's own setter is not reachable from here.
     */
    public DhCurvature.Slot earthCurvature() throws ReflectiveOperationException {
        Object configs = delayed("configs");
        if (configs == null)
            return null;
        Object value = call(GRAPHICS, call(CONFIG, configs, "graphics"), "earthCurvatureRatio");
        return new CurvatureSlot(value);
    }

    /** {@code IDhApiConfigValue<Integer>} behind {@link DhCurvature.Slot}. */
    private final class CurvatureSlot implements DhCurvature.Slot {
        private final Object value;

        CurvatureSlot(Object value) {
            this.value = value;
        }

        private Object get(String method, Object... args) {
            try {
                return call(CONFIG_VALUE, value, method, args);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException("Distant Horizons' " + method + ": " + e, e);
            }
        }

        @Override
        public int playerValue() {
            return (Integer) get("getTrueValue");
        }

        @Override
        public int defaultValue() {
            return (Integer) get("getDefaultValue");
        }

        @Override
        public Integer apiValue() {
            return (Integer) get("getApiValue");
        }

        @Override
        public boolean overridable() {
            return (Boolean) get("getCanBeOverrodeByApi");
        }

        @Override
        public boolean setApi(int ratio) {
            return (Boolean) get("setValue", Integer.valueOf(ratio));
        }

        @Override
        public boolean clearApi() {
            return (Boolean) get("clearValue");
        }
    }

    private static final String WORLD_GEN_CONFIG = "interfaces.config.both.IDhApiWorldGenerationConfig";

    /**
     * Distant Horizons' "distant generation enabled" setting ({@code DhApi.Delayed.configs.
     * worldGenerator().enableDistantWorldGeneration()}, the generator plan as a boolean) as a
     * {@link DhGenerationGate.Slot}; null before Distant Horizons has started its API. The same
     * API-layer reads and writes as {@link #earthCurvature}; the config file is not reachable.
     */
    public DhGenerationGate.Slot generationEnabled() throws ReflectiveOperationException {
        Object configs = delayed("configs");
        if (configs == null)
            return null;
        Object value = call(WORLD_GEN_CONFIG, call(CONFIG, configs, "worldGenerator"), "enableDistantWorldGeneration");
        return new GenerationSlot(value);
    }

    /** {@code IDhApiConfigValue<Boolean>} behind {@link DhGenerationGate.Slot}. */
    private final class GenerationSlot implements DhGenerationGate.Slot {
        private final Object value;

        GenerationSlot(Object value) {
            this.value = value;
        }

        private Object get(String method, Object... args) {
            try {
                return call(CONFIG_VALUE, value, method, args);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException("Distant Horizons' " + method + ": " + e, e);
            }
        }

        @Override
        public boolean playerValue() {
            return Boolean.TRUE.equals(get("getTrueValue"));
        }

        @Override
        public Boolean apiValue() {
            return (Boolean) get("getApiValue");
        }

        @Override
        public boolean overridable() {
            return (Boolean) get("getCanBeOverrodeByApi");
        }

        @Override
        public boolean setApi(boolean enabled) {
            return (Boolean) get("setValue", Boolean.valueOf(enabled));
        }

        @Override
        public boolean clearApi() {
            return (Boolean) get("clearValue");
        }
    }

    /** One stored data point of a column, top-down as Distant Horizons returns them; y in blocks, top exclusive. */
    public record StoredPoint(int bottomY, int topY, Object block, Object biome, int skyLight) {}

    /**
     * The column Distant Horizons holds for block (x, z) of a level (its database or memory), in
     * world y; null when it holds none.
     */
    public List<StoredPoint> storedColumn(Object levelWrapper, int x, int z) throws ReflectiveOperationException {
        Object store = delayed("terrainRepo");
        if (store == null)
            return null;
        Object cache = call(REPO, store, "createSoftCache");
        Object result = call(REPO, store, "getColumnDataAtBlockPos", levelWrapper, x, z, cache);
        if (!(Boolean) result.getClass().getField("success").get(result))
            return null;
        Object[] points = (Object[]) result.getClass().getField("payload").get(result);
        if (points == null)
            return null;
        List<StoredPoint> out = new ArrayList<>();
        for (Object p : points) {
            if (p == null)
                continue;
            Class<?> c = p.getClass();
            Object block = c.getField("blockStateWrapper").get(p), biome = c.getField("biomeWrapper").get(p);
            out.add(new StoredPoint(c.getField("bottomYBlockPos").getInt(p), c.getField("topYBlockPos").getInt(p),
                    block == null ? null : call(UNSAFE, block, "getWrappedMcObject"),
                    biome == null ? null : call(UNSAFE, biome, "getWrappedMcObject"), c.getField("skyLightLevel").getInt(p)));
        }
        return out;
    }
}
