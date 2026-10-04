// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.preset.ShareCode;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorModel;
import com.minecraftindustrialcomplex.deeptime.screen.model.GlobeProjection;
import com.minecraftindustrialcomplex.deeptime.screen.model.MachineInfo;
import com.minecraftindustrialcomplex.deeptime.screen.model.ReviewModel;
import com.minecraftindustrialcomplex.deeptime.screen.model.SessionRunHandle;
import com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.server.Staging;
import com.minecraftindustrialcomplex.deeptime.server.StagingMover;
import com.minecraftindustrialcomplex.deeptime.server.WorldDataBinder;
import com.minecraftindustrialcomplex.deeptime.worldgen.ActiveWorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.AnalyticTestTerrain;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.PlanetGenerators;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataRef;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import com.minecraftindustrialcomplex.deeptime.wrap.PlanetShape;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;
import com.mojang.serialization.JsonOps;
import com.toroidalworld.api.v1.shape.LoopSpans;
import com.toroidalworld.api.v1.shape.ShapeDimensions;
import net.minecraft.core.Direction;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.resources.RegistryOps;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.WorldDimensions;
import net.minecraft.world.level.storage.LevelResource;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.io.IOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * The Create World screen's server-side half (P1-11, plan §7.12), driven through the client-free
 * {@link EditorModel} on the GameTest server with the real natives and registries:
 *
 * <ul>
 *   <li>{@link #acceptBuildsGenerator}: Accept's dimensions update binds the overworld to a
 *       staging folder at the preset's circumference; the codec keeps it; Toroidal World's shape
 *       follows; Create stays blocked until staged data exists.</li>
 *   <li>{@link #stagingMovedOnStart}: the start-time path of {@link WorldDataBinder} (move, then
 *       bind) on this server's world folder, through the copy-and-verify route.</li>
 *   <li>{@link #editorRunsAcceptsAndBinds}: the whole pipeline — share code in, timeline, estimate,
 *       a run on the deeptime-sim worker into a staging folder, timelapse, review, Accept, the move
 *       and a bound generator whose terrain is the run's baked terrain.</li>
 *   <li>{@link #shareCodeThroughModel}: codes round-trip through the model and the core validates
 *       what the model builds.</li>
 *   <li>{@link #realPresetFieldsLightUp}: a P1-03 planet preset enables the planet and scale
 *       fields, an edit lands where the core reads it, and the core accepts the result.</li>
 * </ul>
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class EditorModelGameTests {

    private EditorModelGameTests() {}

    /**
     * A fast placeholder preset at C = 1,024: the dummy pipeline with keyframes, events and two
     * sites, baking the wrap-test terrain (what the built-in placeholders do at 16k/32k).
     */
    static final String FAST = "{\"schema\":0,\"name\":\"gametest_fast\",\"placeholder\":true,\"scale\":{\"circumference_blocks\":1024},"
            + "\"dummy\":{\"steps\":6,\"sleep_ms\":0,\"stages\":[\"dummy_drift\",\"dummy_erode\"],\"synth\":\"wrap-test\","
            + "\"world\":{\"circumference\":1024,\"grid_r\":16,\"t_start_years\":-1.0e8,\"t_end_years\":0.0,"
            + "\"keyframe_every_years\":2.5e7,\"keyframe_size\":[64,32],\"event_every_steps\":2,\"sites\":2,\"seed\":1}}}";

    static Path gameDir() {
        return FMLPaths.GAMEDIR.get();
    }

    static EditorModel model() {
        return new EditorModel(new EditorModel.Deps(json -> SimSession.estimate(json, ""), SessionRunHandle::launch, gameDir(),
                MachineInfo.current(), Runnable::run, Runnable::run, NativeLoader::isLinked, List::of));
    }

    static WorldDimensions planetDimensions(RegistryAccess registries) {
        return registries.registryOrThrow(Registries.WORLD_PRESET).getOrThrow(PlanetGenerators.PLANET).createWorldDimensions();
    }

    /** Copies the synthetic world data gametest.sh wrote into a new staging folder; returns its id. */
    static String stageTestWorld(GameTestHelper helper) {
        Path world = helper.getLevel().getServer().getWorldPath(LevelResource.ROOT).toAbsolutePath().normalize();
        Path src = world.resolve("deeptime-test");
        if (!Files.isRegularFile(src.resolve("manifest.json")))
            helper.fail("no synthetic world data at " + src + " (gametest.sh writes it)");
        String id = Staging.newId();
        try {
            copyTree(src, Staging.dir(gameDir(), id));
        } catch (IOException e) {
            helper.fail("cannot stage " + src + ": " + e);
        }
        return id;
    }

    static void copyTree(Path from, Path to) throws IOException {
        Files.walkFileTree(from, new SimpleFileVisitor<>() {
            @Override
            public FileVisitResult preVisitDirectory(Path d, BasicFileAttributes a) throws IOException {
                Files.createDirectories(to.resolve(from.relativize(d).toString()));
                return FileVisitResult.CONTINUE;
            }

            @Override
            public FileVisitResult visitFile(Path f, BasicFileAttributes a) throws IOException {
                Files.copy(f, to.resolve(from.relativize(f).toString()));
                return FileVisitResult.CONTINUE;
            }
        });
    }

    /** Heights of a bound generator against the analytic wrap-test terrain it was baked from. */
    static double maxHeightError(GeologicChunkGenerator g, int c) {
        if (!(g.terrain() instanceof WorldDataTerrain t))
            throw new IllegalStateException("not generating from world data: " + g.terrain());
        AnalyticTestTerrain analytic = AnalyticTestTerrain.of(AnalyticTestTerrain.WRAP_TEST, c);
        double worst = 0;
        int zp = (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c);   // the poles (the planet's projection)
        for (int i = 0; i < 64; i++) {
            int x = -c / 2 + (i * 37) % c, z = ((i * 53) % (2 * zp - 16)) - zp + 8;
            worst = Math.max(worst, Math.abs(t.height(x, z) - analytic.height(x, z)));
        }
        return worst;
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 400)
    public static void acceptBuildsGenerator(GameTestHelper helper) {
        RegistryAccess registries = helper.getLevel().getServer().registryAccess();
        WorldDimensions fresh = planetDimensions(registries);
        helper.assertTrue(PlanetGenerators.overworld(fresh).isPresent(), "deeptime:planet has a Deep Time overworld");
        helper.assertTrue(PlanetGenerators.createBlock(fresh, gameDir(), true) == PlanetGenerators.CreateBlock.NOT_SIMULATED,
                "Create is blocked before any run");
        helper.assertTrue(PlanetGenerators.createBlock(fresh, gameDir(), false) == PlanetGenerators.CreateBlock.NATIVES,
                "without natives Create is blocked for Deep Time (plan §6.6)");

        String id = stageTestWorld(helper);
        WorldDimensions accepted = PlanetGenerators.withStaging(registries, fresh, id, 32_768);
        GeologicChunkGenerator g = PlanetGenerators.overworld(accepted).orElseThrow();
        WorldDataRef ref = g.dataRef();
        helper.assertTrue(ref.stagingId().equals(Optional.of(id)), "bound to the staging folder");
        helper.assertTrue(ref.circumferenceBlocks() == 32_768, "the preset's circumference, not the world preset's");
        helper.assertTrue(ref.folder().equals("deeptime"), "world data goes to <world>/deeptime");
        helper.assertTrue(ref.testTerrain().isEmpty(), "no test terrain");
        LevelStem stem = accepted.dimensions().get(LevelStem.OVERWORLD);
        helper.assertTrue(stem.type().is(net.minecraft.resources.ResourceKey.create(Registries.DIMENSION_TYPE,
                net.minecraft.resources.ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "planet_overworld"))),
                "the dimension type stays deeptime:planet_overworld");
        helper.assertTrue(PlanetGenerators.createBlock(accepted, gameDir(), true) == PlanetGenerators.CreateBlock.NONE,
                "Create is enabled once staged data exists");
        helper.assertTrue(PlanetGenerators.createBlock(accepted, gameDir(), false) == PlanetGenerators.CreateBlock.NONE,
                "a finished run needs no natives to create the world");

        // The generator as level.dat stores it.
        RegistryOps<JsonElement> ops = RegistryOps.create(JsonOps.INSTANCE, registries);
        JsonElement json = ChunkGenerator.CODEC.encodeStart(ops, g).getOrThrow();
        JsonObject data = json.getAsJsonObject().getAsJsonObject("world_data");
        helper.assertTrue(data.get("staging_id").getAsString().equals(id) && data.get("circumference_blocks").getAsInt() == 32_768,
                "codec: " + json);
        ChunkGenerator decoded = ChunkGenerator.CODEC.parse(ops, json).getOrThrow();
        helper.assertTrue(decoded instanceof GeologicChunkGenerator dg && dg.dataRef().equals(ref), "codec round trip");

        // Toroidal World's shape follows the new circumference (PlanetShape at bake).
        LoopSpans spans = ShapeDimensions.spansOf(PlanetShape.shape(accepted), LevelStem.OVERWORLD);
        helper.assertTrue(spans != null && spans.loops(Direction.Axis.X) && !spans.loops(Direction.Axis.Z)
                && spans.chunkWidth(Direction.Axis.X) == 2048, "X loops over 2,048 chunks: " + spans);

        // A staging id whose folder is gone blocks Create again.
        try {
            Staging.deleteTree(Staging.dir(gameDir(), id));
        } catch (IOException e) {
            helper.fail("cannot delete the staging folder: " + e);
        }
        helper.assertTrue(PlanetGenerators.createBlock(accepted, gameDir(), true) == PlanetGenerators.CreateBlock.STAGING_MISSING,
                "a vanished staging folder blocks Create");
        DeepTime.LOGGER.info("[gametest] acceptBuildsGenerator staging={} C={} twChunks={} codec={}", id, ref.circumferenceBlocks(),
                spans.chunkWidth(Direction.Axis.X), data);
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 400)
    public static void stagingMovedOnStart(GameTestHelper helper) {
        Path world = helper.getLevel().getServer().getWorldPath(LevelResource.ROOT).toAbsolutePath().normalize();
        String id = stageTestWorld(helper);
        Path staging = Staging.dir(gameDir(), id);
        WorldDataRef ref = new WorldDataRef("deeptime-staged", 1024, Optional.empty(), Optional.of(id),
                PoleCapCurve.DEFAULT_CAP_DEG, PoleCapCurve.DEFAULT_STOP_DEG);
        Path target = world.resolve(ref.folder());
        try {
            Staging.deleteTree(target);
            // Force the copy route (the rename is the JUnit tests' and the one a same-disk game takes).
            var r = WorldDataBinder.moveStaged(gameDir(), world, ref, true).orElseThrow();
            helper.assertTrue(r.kind() == StagingMover.Kind.COPIED, "copied: " + r);
            helper.assertTrue(!Files.exists(staging), "the staging folder is gone");
            helper.assertTrue(Files.isRegularFile(target.resolve("manifest.json")), "the world has its data");
            var again = WorldDataBinder.moveStaged(gameDir(), world, ref, true).orElseThrow();
            helper.assertTrue(again.kind() == StagingMover.Kind.ALREADY_PRESENT, "a restart moves nothing: " + again);
        } catch (StagingMover.StagingException e) {
            helper.fail(e.getMessage());
        } catch (IOException e) {
            helper.fail(e.toString());
        }
        helper.assertTrue(WorldDataBinder.bindLevel(world, ref), "bound");
        helper.assertTrue(ActiveWorldData.get(ref.folder()).circumference() == 1024, "C = 1024");
        GeologicChunkGenerator g = new GeologicChunkGenerator(
                ((GeologicChunkGenerator) helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_DATA)
                        .getChunkSource().getGenerator()).getBiomeSource(),
                ((GeologicChunkGenerator) helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_DATA)
                        .getChunkSource().getGenerator()).sourceSettings(), ref);
        double err = maxHeightError(g, 1024);
        DeepTime.LOGGER.info("[gametest] stagingMovedOnStart staging={} -> {} copied, bound, max |H - analytic| = {}", id, target, err);
        helper.assertTrue(err < 1.0 / 16, "the moved data generates the staged terrain (max error " + err + ")");
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void editorRunsAcceptsAndBinds(GameTestHelper helper) {
        if (!NativeLoader.isLinked())
            helper.fail("natives not linked: " + NativeLoader.status().map(s -> s.describe()).orElse("never loaded"));
        EditorModel m = model();
        helper.assertTrue(m.importShareCode(ShareCode.encode(FAST)).isEmpty(), "the fast preset imports");
        helper.assertTrue(!m.canRun(), "no timeline yet: Run is disabled");
        m.setStart("supercontinent");
        m.setLengthText("100");
        m.refreshEstimate();
        helper.assertTrue(m.estimateError().isEmpty(), "the core accepts the preset: " + m.estimateError().orElse(""));
        helper.assertTrue(m.canRun(), "Run is enabled: " + m.problems());
        TimelapseModel run = m.startRun().orElseThrow(() -> new IllegalStateException("launch failed: " + m.problems()));
        Path staging = run.run().outDir();
        AtomicBoolean finished = new AtomicBoolean();
        helper.onEachTick(() -> {
            if (finished.get())
                return;
            run.poll();
            if (run.state() == TimelapseModel.State.RUNNING)
                return;
            finished.set(true);
            run.poll();
            helper.assertTrue(run.state() == TimelapseModel.State.DONE, "run " + run.state() + " " + run.failure().orElse(""));
            SimSession.Progress p = run.progress();
            helper.assertTrue(run.frameCount() == p.keyframesReady() && run.frameCount() >= 4, "keyframes " + run.frameCount());
            helper.assertTrue(run.events().size() == p.eventsTotal(), "events " + run.events().size() + " of " + p.eventsTotal());
            run.scrubTo(0);
            helper.assertTrue(run.displayed().isPresent(), "frame 0 decodes");
            helper.assertTrue(Staging.complete(staging), "staging folder complete");

            ReviewModel review = m.openReview();
            helper.assertTrue(review.ready() && review.error().isEmpty(), "review: " + review.error().orElse(""));
            helper.assertTrue(review.map().map(ReviewModel.MapImage::source).orElse("").equals("world_data"), "final map from world data");
            helper.assertTrue(review.spawn().isPresent(), "a spawn marker");
            helper.assertTrue(review.summary().map(s -> s.sites() == 2).orElse(false), "two sites");

            EditorModel.Accepted a = m.accept();
            RegistryAccess registries = helper.getLevel().getServer().registryAccess();
            WorldDimensions dims = PlanetGenerators.withStaging(registries, planetDimensions(registries), a.stagingId(), a.circumference());
            GeologicChunkGenerator accepted = PlanetGenerators.overworld(dims).orElseThrow();
            helper.assertTrue(PlanetGenerators.createBlock(dims, gameDir(), true) == PlanetGenerators.CreateBlock.NONE, "Create enabled");

            // The world's start: move the staged data into a world folder and bind it.
            Path world = helper.getLevel().getServer().getWorldPath(LevelResource.ROOT).toAbsolutePath().normalize();
            WorldDataRef ref = new WorldDataRef("deeptime-e2e", accepted.dataRef().circumferenceBlocks(), Optional.empty(),
                    accepted.dataRef().stagingId(), accepted.dataRef().poleCapDeg(), accepted.dataRef().poleStopDeg());
            try {
                Staging.deleteTree(world.resolve(ref.folder()));
                var r = WorldDataBinder.moveStaged(gameDir(), world, ref, false).orElseThrow();
                helper.assertTrue(r.kind() == StagingMover.Kind.RENAMED, "renamed: " + r);
            } catch (StagingMover.StagingException | IOException e) {
                helper.fail(e.toString());
            }
            helper.assertTrue(WorldDataBinder.bindLevel(world, ref), "bound");
            GeologicChunkGenerator g = new GeologicChunkGenerator(accepted.getBiomeSource(), accepted.sourceSettings(), ref);
            double err = maxHeightError(g, 1024);
            helper.assertTrue(err < 1.0 / 16, "the world generates the run's terrain (max error " + err + ")");
            DeepTime.LOGGER.info("[gametest] editorRunsAcceptsAndBinds frames={} events={} layers={} spawn={} provinces={} staging={} "
                            + "moved+bound, max |H - analytic| = {}", run.frameCount(), run.events().size(), run.layers(),
                    review.spawn().map(s -> s[0] + "," + s[1]).orElse("-"), review.provinces().size(), a.stagingId(), err);
            helper.succeed();
        });
    }

    /**
     * P1-11 on the real stages: the editor runs a real planet (every stage and the bake, through the
     * JNI as the Create World screen does; the tests' small planet, C = 4,096, cut to 100 Myr) on 3
     * simulation threads, the timelapse shows its real layers (satellite first, grouped) and events,
     * the review reads the staged world data, Accept binds the generator, and the world's start
     * moves the folder into the world and generates from it: the terrain's land share follows the
     * preset's ocean fraction.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 20 * 60 * 20)
    public static void realPlanetThroughTheEditor(GameTestHelper helper) {
        if (!NativeLoader.isLinked())
            helper.fail("natives not linked: " + NativeLoader.status().map(s -> s.describe()).orElse("never loaded"));
        String json;
        try {
            json = Files.readString(Path.of(System.getProperty("deeptime.repoRoot", ".."), "crates/deeptime-core/tests/presets/planet_world_4k.json"));
        } catch (IOException e) {
            helper.fail("cannot read planet_world_4k.json (deeptime.repoRoot): " + e);
            return;
        }
        MachineInfo small = new MachineInfo(8L << 30, 16L << 30, Runtime.getRuntime().maxMemory(), 4);
        EditorModel m = new EditorModel(new EditorModel.Deps(j -> SimSession.estimate(j, ""), SessionRunHandle::launch, gameDir(), small,
                Runnable::run, Runnable::run, NativeLoader::isLinked, List::of));
        helper.assertTrue(m.importShareCode(ShareCode.encode(json)).isEmpty(), "the real preset imports");
        m.setStart("scattered_continents");
        m.setLengthText("100");
        m.refreshEstimate();
        helper.assertTrue(m.canRun(), "Run is enabled: " + m.problems());
        List<String> stages = m.estimate().orElseThrow().stages().stream().map(s -> s.name()).toList();
        helper.assertTrue(stages.equals(List.of("plates", "climate", "surface", "volcanoes", "impacts", "structure", "deposits", "young",
                "amplify")), "the real stages: " + stages);
        TimelapseModel run = m.startRun().orElseThrow(() -> new IllegalStateException("launch failed: " + m.problems()));
        long started = System.currentTimeMillis();
        AtomicBoolean finished = new AtomicBoolean();
        helper.onEachTick(() -> {
            if (finished.get())
                return;
            run.poll();
            if (run.state() == TimelapseModel.State.RUNNING)
                return;
            finished.set(true);
            run.poll();
            helper.assertTrue(run.state() == TimelapseModel.State.DONE, "run " + run.state() + " " + run.failure().orElse(""));
            SimSession.Progress p = run.progress();
            helper.assertTrue(run.frameCount() == p.keyframesReady() && run.frameCount() >= 3, "keyframes " + run.frameCount());
            helper.assertTrue(run.events().size() == p.eventsTotal() && run.events().size() > 0, "events " + run.events().size());
            helper.assertTrue(run.layers().containsAll(List.of("satellite", "plates", "climate/koppen", "deposits/sites")),
                    "real layers: " + run.layers());
            helper.assertTrue("satellite".equals(run.layer().orElse("")), "the satellite view first: " + run.layer());
            helper.assertTrue(run.markers().size() <= TimelapseModel.MAX_MARKERS, "markers bounded");
            run.scrubTo(0);
            helper.assertTrue(run.displayed().isPresent(), "frame 0 decodes");

            ReviewModel review = m.openReview();
            helper.assertTrue(review.ready() && review.error().isEmpty(), "review: " + review.error().orElse(""));
            helper.assertTrue(review.map().map(ReviewModel.MapImage::source).orElse("").equals("world_data"), "final map from world data");
            helper.assertTrue(review.spawn().isPresent(), "a spawn marker");

            EditorModel.Accepted a = m.accept();
            RegistryAccess registries = helper.getLevel().getServer().registryAccess();
            WorldDimensions dims = PlanetGenerators.withStaging(registries, planetDimensions(registries), a.stagingId(), a.circumference());
            GeologicChunkGenerator accepted = PlanetGenerators.overworld(dims).orElseThrow();
            helper.assertTrue(accepted.dataRef().circumferenceBlocks() == 4096, "bound at the run's circumference");
            helper.assertTrue(PlanetGenerators.createBlock(dims, gameDir(), true) == PlanetGenerators.CreateBlock.NONE, "Create enabled");

            Path world = helper.getLevel().getServer().getWorldPath(LevelResource.ROOT).toAbsolutePath().normalize();
            WorldDataRef ref = new WorldDataRef("deeptime-e2e-real", accepted.dataRef().circumferenceBlocks(), Optional.empty(),
                    accepted.dataRef().stagingId(), accepted.dataRef().poleCapDeg(), accepted.dataRef().poleStopDeg());
            try {
                Staging.deleteTree(world.resolve(ref.folder()));
                var r = WorldDataBinder.moveStaged(gameDir(), world, ref, false).orElseThrow();
                helper.assertTrue(r.kind() == StagingMover.Kind.RENAMED, "renamed: " + r);
            } catch (StagingMover.StagingException | IOException e) {
                helper.fail(e.toString());
            }
            helper.assertTrue(WorldDataBinder.bindLevel(world, ref), "bound");
            GeologicChunkGenerator g = new GeologicChunkGenerator(accepted.getBiomeSource(), accepted.sourceSettings(), ref);
            if (!(g.terrain() instanceof WorldDataTerrain t)) {
                helper.fail("not generating from world data: " + g.terrain());
                return;
            }
            int sea = t.data().manifest().seaLevelY(), land = 0, n = 0;
            int zp = (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(4096);
            for (int i = 0; i < 400; i++) {
                int x = -2048 + (i * 97) % 4096, z = ((i * 61) % (2 * zp - 32)) - zp + 16;
                double h = t.height(x, z);
                helper.assertTrue(Double.isFinite(h), "height at " + x + "," + z);
                n++;
                if (h > sea)
                    land++;
            }
            // Land and sea both (planet_world_4k cut to 100 Myr has about a tenth land: 0.09 on
            // 2026-10-04), and the review's spawn stands on dry land in the moved world's terrain.
            double share = land / (double) n;
            helper.assertTrue(share > 0.02 && share < 0.6, "land share " + share + " (the preset's ocean fraction is 0.7)");
            double[] sp = review.spawn().orElseThrow();
            double[] sb = GlobeProjection.block(sp[0], sp[1], 4096, new double[2]);
            double spawnH = t.height((int) Math.floor(sb[0]), (int) Math.floor(sb[1]));
            helper.assertTrue(spawnH > sea, "the review's spawn " + (int) sb[0] + "," + (int) sb[1] + " is dry land (height " + spawnH + ", sea " + sea + ")");
            DeepTime.LOGGER.info("[gametest] realPlanetThroughTheEditor {} s: frames={} events={} layers={} groups={} native peak {} MiB, "
                            + "spawn={} land share {} staging={} moved+bound", (System.currentTimeMillis() - started) / 1000, run.frameCount(),
                    run.events().size(), run.layers().size(), run.groups(), p.nativeBytesPeak() >> 20,
                    review.spawn().map(s -> s[0] + "," + s[1]).orElse("-"), share, a.stagingId());
            helper.succeed();
        });
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void shareCodeThroughModel(GameTestHelper helper) {
        EditorModel a = model();
        a.selectSize(32_768);
        a.setStart("scattered_continents");
        a.setLengthText("500");   // a real planet since P1-08: a length whose estimate fits the default cap
        a.refreshEstimate();
        helper.assertTrue(a.canRun(), "A can run: " + a.problems());
        String code = a.exportShareCode();

        EditorModel b = model();
        helper.assertTrue(b.importShareCode(code).isEmpty(), "B imports A's code");
        b.refreshEstimate();
        helper.assertTrue(b.effectivePresetJson().equals(a.effectivePresetJson()), "same preset");
        helper.assertTrue(b.presetName().equals("earthlike_quick_32k") && !b.isImported(), "the built-in is selected");
        helper.assertTrue(b.canRun(), "B can run: " + b.problems());
        helper.assertTrue(b.exportShareCode().equals(code), "one preset, one code");

        // The Java and Rust codes agree: the core runs what the model encodes (estimate validates).
        var info = Presets.info(Presets.parseObject(ShareCode.decode(code)));
        helper.assertTrue(info.timeline().orElseThrow().equals(new Presets.Timeline("scattered_continents", 500.0)), "timeline kept");

        // A preset with a section the core does not know is refused by the core (Run stays off), not hidden.
        JsonObject custom = Presets.parseObject(ShareCode.decode(code));
        custom.add("no_such_section", new JsonObject());
        EditorModel c = model();
        helper.assertTrue(c.importShareCode(ShareCode.encode(custom.toString())).isEmpty(), "imports");
        c.refreshEstimate();
        helper.assertTrue(c.estimateError().isPresent() && !c.canRun(),
                "the core refuses an unknown key: " + c.estimateError().orElse("(accepted)"));
        DeepTime.LOGGER.info("[gametest] shareCodeThroughModel code={}… ({} chars) customRefused={}", code.substring(0, 16), code.length(),
                c.estimateError().orElse(""));
        helper.succeed();
    }

    /**
     * A real planet preset (P1-03's pipeline, {@code params.rs}) lights up the editor's planet and
     * scale fields with no screen change, an edit lands at the path the core reads, and the core
     * accepts what the model builds (its estimate validates every section).
     */
    static final String PLANET = "{\"schema\":0,\"name\":\"gametest_planet\",\"seed\":7,"
            + "\"planet\":{\"gravity_m_s2\":9.81,\"day_length_h\":24,\"axial_tilt_deg\":23.4,\"star_flux_rel\":1.0,"
            + "\"ocean_fraction\":0.7,\"regime\":\"mobile_lid\"},"
            + "\"scale\":{\"circumference_blocks\":16384,\"m_per_block_h\":300,\"m_per_block_v\":20},"
            + "\"resolution\":{\"l0_samples\":20000,\"l1_r\":16}}";

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void realPresetFieldsLightUp(GameTestHelper helper) {
        EditorModel m = model();
        helper.assertTrue(m.importShareCode(ShareCode.encode(PLANET)).isEmpty(), "imports");
        helper.assertTrue(m.isImported() && m.circumference() == 16_384, "custom preset at C 16,384");
        List<String> available = new java.util.ArrayList<>();
        for (var f : com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields.ALL)
            if (m.available(f))
                available.add(f.id());
        helper.assertTrue(available.containsAll(List.of("gravity", "day_length", "axial_tilt", "star_flux", "ocean_fraction",
                "m_per_block_h", "m_per_block_v")), "planet and scale fields editable: " + available);
        helper.assertTrue(!available.contains("sky_islands"), "fantasy toggles stay off (Phase 3)");
        var gravity = com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields.byId("gravity");
        m.setFieldText(gravity, "3.71");
        m.setStart("scattered_continents");
        m.setLengthText("500");
        m.refreshEstimate();
        helper.assertTrue(m.estimateError().isEmpty(), "the core accepts the edited preset: " + m.estimateError().orElse(""));
        helper.assertTrue(m.effectivePresetJson().contains("\"gravity_m_s2\":3.71"), "the edit is at planet.gravity_m_s2");
        helper.assertTrue(m.canRun(), "Run enabled: " + m.problems());
        // A value the core's validation refuses shows as the core's message, and Run stays off.
        var ocean = com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields.byId("ocean_fraction");
        m.setFieldText(ocean, "0.99");
        helper.assertTrue(!m.fieldValid(ocean) && !m.canRun(), "0.99 is outside the editor's (and the core's) range");
        DeepTime.LOGGER.info("[gametest] realPresetFieldsLightUp available={} estimate={}", available,
                m.estimate().map(e -> e.peakBytes() + " B, " + e.stages().size() + " stages").orElse("-"));
        helper.succeed();
    }
}
