// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.nativebridge.CancelledException;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeStatus;
import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.preset.PresetException;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataRef;
import net.minecraft.core.registries.Registries;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.storage.LevelResource;
import net.neoforged.fml.loading.FMLPaths;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.StandardCopyOption;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

/**
 * Generates a dedicated server's world data on its first start (plan §7.13, P1-13; decisions
 * file, "Dedicated servers: seamless"). {@link WorldDataBinder}'s {@code ServerAboutToStartEvent}
 * listener (priority HIGHEST) calls {@link #prepare} before it binds, so the binder binds what this
 * wrote. In {@code DedicatedServer.initServer} the event comes before {@code loadLevel()} and before
 * RCON, so progress goes to the console only.
 *
 * <ol>
 *   <li>Only a dedicated server whose overworld generator is Deep Time's, without a test terrain
 *       or a staging folder, is considered.</li>
 *   <li>{@code <world>/deeptime/manifest.json} present (a previous start, or {@code deeptime run}
 *       output copied in): nothing to do.</li>
 *   <li>Otherwise read {@code config/deeptime/server.json} ({@link ServerConfig}); if it is
 *       missing, write the template and stop. Resolve the preset (built-in, file or share code),
 *       apply the timeline, and stop with both numbers if the preset's circumference is not the
 *       level-type's. Natives that did not load stop the server with their guidance and the
 *       equivalent {@code deeptime run} command (plan §6.6).</li>
 *   <li>Run a {@link SimSession} to completion into {@code <world>/deeptime.generating/}, logging
 *       a progress line every 5 s, then rename that folder to {@code <world>/deeptime/}: the
 *       session writes {@code manifest.json} last, and the rename makes the folder appear whole.
 *       A Ctrl-C while it runs cancels the session.</li>
 * </ol>
 *
 * Every stop goes through {@link ServerStop}: before {@code loadLevel()}, so no half-made world
 * is saved. Nothing here depends on which stages a preset runs: the real presets of P1-03…P1-08
 * replace the built-in placeholders without a change here.
 */
public final class FirstStartGenerator {

    /** Progress line interval (plan §7.13). */
    static final long PROGRESS_EVERY_MS = 5_000;

    /** The generation folder, next to the world-data folder, renamed into place when complete. */
    static final String GENERATING_SUFFIX = ".generating";

    private static final double GIB = 1L << 30;

    private FirstStartGenerator() {}

    /** A reason to stop the server, with what to do about it. */
    private static final class Stop extends Exception {
        private static final long serialVersionUID = 1L;
        final String details;
        final boolean failure;

        Stop(String headline, String details) {
            this(headline, details, true);
        }

        Stop(String headline, String details, boolean failure) {
            super(headline);
            this.details = details;
            this.failure = failure;
        }
    }

    /**
     * Generates the world data if this start needs it. Returns {@code true} to go on (bind and
     * load the level), {@code false} when the server is stopping.
     */
    public static boolean prepare(MinecraftServer server) {
        if (!server.isDedicatedServer())
            return true;
        LevelStem stem = server.registryAccess().registryOrThrow(Registries.LEVEL_STEM).get(LevelStem.OVERWORLD);
        if (stem == null || !(stem.generator() instanceof GeologicChunkGenerator g))
            return true;
        WorldDataRef ref = g.dataRef();
        if (ref.effectiveTestTerrain().isPresent() || ref.stagingId().isPresent())
            return true;
        Path world = server.getWorldPath(LevelResource.ROOT).toAbsolutePath().normalize();
        Path dir = world.resolve(ref.folder());
        if (Files.isRegularFile(dir.resolve(Manifest.FILE))) {
            DeepTime.LOGGER.info("[deeptime] world data present at {}: not regenerating", dir);
            return true;
        }
        try {
            generate(server, ref, world, dir);
            return true;
        } catch (Stop s) {
            ServerStop.request(server, s.getMessage(), s.details, s.failure);
            return false;
        } catch (RuntimeException e) {
            DeepTime.LOGGER.error("[deeptime] first-start generation failed unexpectedly", e);
            ServerStop.request(server, "first-start generation failed: " + e,
                    "See the stack trace above. Nothing was written to " + dir + ".", true);
            return false;
        }
    }

    private static void generate(MinecraftServer server, WorldDataRef ref, Path world, Path dir) throws Stop {
        int c = ref.circumferenceBlocks();
        if (Files.exists(dir))
            throw new Stop(dir + " exists but has no " + Manifest.FILE,
                    "An incomplete copy of world data? Finish copying it (the manifest is written last), or remove\n"
                            + "the folder to let Deep Time generate the world from config/deeptime/server.json.");

        // 1. The config, or its template.
        Path configDir = FMLPaths.CONFIGDIR.get().resolve(DeepTime.MODID);
        Path configFile = configDir.resolve(ServerConfig.FILE);
        if (!Files.isRegularFile(configFile)) {
            try {
                Files.createDirectories(configDir);
                Path tmp = configDir.resolve(ServerConfig.FILE + ".tmp");
                Files.writeString(tmp, ServerConfig.template(), StandardCharsets.UTF_8);
                Files.move(tmp, configFile, StandardCopyOption.REPLACE_EXISTING);
            } catch (IOException e) {
                throw new Stop("no " + configFile + ", and the template could not be written: " + e,
                        "Create it by hand; see the plan's §7.13 for its fields.");
            }
            throw new Stop("this Deep Time world has no world data and no " + configFile,
                    "Wrote a template to " + configFile + ".\n"
                            + "Fill in \"preset\" and \"timeline\" there (the _ keys explain them), then start the server again.\n"
                            + "The level-type in server.properties sets the size: " + levelTypeFor(c) + " gives C = " + c + ".");
        }
        ServerConfig config;
        try {
            config = ServerConfig.parse(Files.readString(configFile, StandardCharsets.UTF_8));
        } catch (ServerConfig.Invalid e) {
            throw new Stop(configFile + " cannot be used",
                    "Fix these fields (the _ keys of the template explain them), then start the server again:\n  - "
                            + String.join("\n  - ", e.problems()));
        } catch (IOException e) {
            throw new Stop("cannot read " + configFile + ": " + e, "Check the file's permissions.");
        }

        // 2. The preset, its timeline and its circumference.
        Presets.Resolved resolved;
        JsonObject preset;
        try {
            resolved = Presets.resolve(config.preset(), configDir, FMLPaths.GAMEDIR.get());
            preset = Presets.parseObject(resolved.json());
        } catch (PresetException e) {
            throw new Stop("the preset in " + configFile + " cannot be used: " + e.getMessage(),
                    "Set \"preset\" to a built-in preset (" + String.join(", ", Presets.BUILTIN)
                            + "), a preset file or a share code (DT1.…).");
        }
        Presets.Info info = Presets.info(preset);
        Presets.Timeline timeline;
        if (config.timeline().isPresent()) {
            timeline = config.timeline().get();
            if (info.timeline().isPresent() && !info.timeline().get().equals(timeline))
                DeepTime.LOGGER.info("[deeptime] server.json's timeline ({}) replaces the preset's ({})", timeline,
                        info.timeline().get());
        } else if (info.timeline().isPresent() && info.timeline().get().problems("timeline").isEmpty()) {
            timeline = info.timeline().get();
        } else {
            throw new Stop("no timeline: " + configFile + " names none, and neither does the preset " + resolved.label(),
                    "There is no default story. Add to " + configFile + ":\n"
                            + "  \"timeline\": {\"start\": \"supercontinent\" | \"scattered_continents\" | \"random\", \"length_myr\": <1 to 4600>}");
        }
        String presetJson = Presets.withTimeline(preset, timeline);
        String name = info.name().orElse(resolved.label());
        if (info.circumference().isEmpty())
            throw new Stop("the preset " + resolved.label() + " names no circumference (scale.circumference_blocks)",
                    "Use a built-in preset, or a preset or share code from this version of Deep Time.");
        int presetC = info.circumference().getAsInt();
        if (presetC != c)
            throw new Stop("circumference mismatch: the preset " + name + " is for C = " + presetC
                            + " blocks, but this world's level-type gives C = " + c + " blocks",
                    "The level-type in server.properties (" + levelTypeFor(c) + ") fixes the world's size before the first start.\n"
                            + "Either choose a preset for C = " + c + " (" + presetsFor(c) + "),\n"
                            + "or set level-type=" + levelTypeFor(presetC).replace(":", "\\:") + " in server.properties and delete the world folder "
                            + world.getFileName() + "\n(it holds no world yet, but its level.dat already records the old level-type).");

        // 3. The natives (plan §6.6).
        String cli = "deeptime run --preset " + quote(config.preset()) + " --start " + timeline.start()
                + " --length-myr " + timeline.lengthMyr() + " --out " + quote(dir.toString());
        NativeStatus natives = NativeLoader.status().orElse(null);
        if (natives == null || !natives.isOk() || !NativeLoader.isLinked()) {
            String block = natives == null ? "Deep Time natives were not loaded." : natives.describe();
            throw new Stop("the Deep Time native library is not available here"
                            + (natives == null || natives.isOk() ? "" : " (" + natives.failureClass() + ")"),
                    block + "\n"
                            + "Without it this server cannot simulate the world's history. On any supported machine run\n"
                            + "  " + cli + "\n"
                            + "then copy that deeptime/ folder into " + world + " and start the server again.");
        }

        // 4. Estimate, then run.
        long cap = config.memoryCapBytes();
        List<String> stageNames = new ArrayList<>();
        long peak;
        try {
            JsonObject est = Presets.parseObject(SimSession.estimate(presetJson, ""));
            peak = est.get("peak_bytes").getAsLong();
            for (JsonElement st : est.getAsJsonArray("stages"))
                stageNames.add(st.getAsJsonObject().get("name").getAsString());
        } catch (RuntimeException e) {
            throw new Stop("the preset " + name + " is not valid: " + e.getMessage(),
                    "Fix the preset (or choose a built-in one) in " + configFile + ".");
        }
        if (peak > cap)
            throw new Stop(String.format(Locale.ROOT, "the preset %s needs about %.1f GiB of native memory, above memory_cap_gib %.1f",
                            name, peak / GIB, cap / GIB),
                    "Raise \"memory_cap_gib\" in " + configFile + " (the machine needs that much RAM besides the server's -Xmx),\n"
                            + "or choose a smaller preset.");
        if (info.placeholder())
            DeepTime.LOGGER.warn("[deeptime] {} is a Phase 1 placeholder, not a simulated planet: {}", name,
                    info.description().orElse("the dummy pipeline plus a synthetic terrain"));

        Path work = world.resolve(ref.folder() + GENERATING_SUFFIX);
        try {
            if (Files.exists(work)) {
                DeepTime.LOGGER.info("[deeptime] removing an unfinished generation at {}", work);
                deleteTree(work);
            }
            Files.createDirectories(work);
        } catch (IOException e) {
            throw new Stop("cannot prepare " + work + ": " + e, "Check that the world folder is writable.");
        }
        int threads = config.threads() > 0 ? config.threads() : Runtime.getRuntime().availableProcessors();
        DeepTime.LOGGER.info("[deeptime] generating the world's history on first start: preset {}, C = {}, timeline {}, "
                        + "{} threads, native cap {} GiB, estimate {} GiB, {} stages -> {}",
                name.equals(resolved.label()) ? name : name + " (" + resolved.label() + ")", c, timeline, threads,
                gib(cap), gib(peak), stageNames.size(), dir);
        long started = System.nanoTime();
        String manifestJson = run(server, presetJson, work, threads, cap, stageNames, started);

        // 5. Into place, whole.
        try {
            if (!Files.isRegularFile(work.resolve(Manifest.FILE)))
                throw new IOException("the session finished without writing " + Manifest.FILE);
            try {
                Files.move(work, dir, StandardCopyOption.ATOMIC_MOVE);
            } catch (AtomicMoveNotSupportedException e) {
                Files.move(work, dir);
            }
        } catch (IOException e) {
            throw new Stop("cannot move the generated world data into " + dir + ": " + e,
                    "The complete data is in " + work + "; rename it to " + dir.getFileName() + " and start again.");
        }
        long ms = TimeUnit.NANOSECONDS.toMillis(System.nanoTime() - started);
        DeepTime.LOGGER.info("[deeptime] world data written to {} in {} ({})", dir, duration(ms), summary(manifestJson));
    }

    /** Runs the session to completion on this (the server) thread's watch; returns the manifest JSON. */
    private static String run(MinecraftServer server, String presetJson, Path work, int threads, long cap,
            List<String> stageNames, long started) throws Stop {
        SimSession session;
        try {
            session = SimSession.start(presetJson, "", work, threads, cap);
        } catch (RuntimeException e) {
            deleteQuietly(work);
            throw new Stop("the simulation could not start: " + e.getMessage(), "Nothing was written.");
        }
        long nextLine = started + TimeUnit.MILLISECONDS.toNanos(PROGRESS_EVERY_MS);
        try {
            while (true) {
                try {
                    return session.completion().get(250, TimeUnit.MILLISECONDS);
                } catch (TimeoutException e) {
                    // still running
                }
                if (!server.isRunning()) {
                    // Ctrl-C (the shutdown hook halts the server and waits for this thread).
                    session.cancel();
                    session.close();
                    deleteQuietly(work);
                    throw new Stop("generation cancelled: the server is shutting down",
                            "Nothing was kept; the next start generates again.", false);
                }
                long now = System.nanoTime();
                if (now >= nextLine) {
                    DeepTime.LOGGER.info("[deeptime] {}", progressLine(session.progress(), stageNames, cap));
                    nextLine += TimeUnit.MILLISECONDS.toNanos(PROGRESS_EVERY_MS);
                    if (nextLine < now)
                        nextLine = now + TimeUnit.MILLISECONDS.toNanos(PROGRESS_EVERY_MS);
                }
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            session.close();
            deleteQuietly(work);
            throw new Stop("generation interrupted", "Nothing was kept; the next start generates again.", false);
        } catch (ExecutionException e) {
            Throwable cause = e.getCause() == null ? e : e.getCause();
            deleteQuietly(work);
            if (cause instanceof CancelledException)
                throw new Stop("generation cancelled", "Nothing was kept; the next start generates again.", false);
            DeepTime.LOGGER.error("[deeptime] the simulation failed", cause);
            throw new Stop("the simulation failed: " + cause.getClass().getSimpleName() + ": " + cause.getMessage(),
                    "Nothing was kept. The native log above has the details.");
        }
    }

    /**
     * {@code <stage> 37% (−312 Ma) · total 18% · 1m12s elapsed · ~2m40s left · native 1.3/4.0 GiB}
     * (plan §7.13, with the run's total added).
     */
    static String progressLine(SimSession.Progress p, List<String> stageNames, long cap) {
        StringBuilder b = new StringBuilder();
        if (p.state() == SimSession.Progress.RUNNING) {
            String stage = p.stageIndex() < stageNames.size() ? stageNames.get(p.stageIndex())
                    : "stage " + (p.stageIndex() + 1) + "/" + p.stageCount();
            b.append(stage).append(' ').append(p.stagePermille() / 10).append('%');
            b.append(" (").append(ma(p.simulatedTime())).append(')');
            b.append(" · total ").append(p.totalPermille() / 10).append('%');
        } else {
            b.append("writing world data · total 100%");
        }
        b.append(" · ").append(duration(p.elapsedMs())).append(" elapsed");
        if (p.state() == SimSession.Progress.RUNNING)
            b.append(" · ").append(p.etaMs() < 0 ? "time left unknown" : "~" + duration(p.etaMs()) + " left");
        b.append(" · native ").append(gib(p.nativeBytesInUse())).append('/').append(gib(cap)).append(" GiB");
        return b.toString();
    }

    /** Simulated time (years × −1) as {@code −312 Ma}, or {@code 0 Ma} at the present. */
    static String ma(long yearsAgo) {
        long ma = Math.round(yearsAgo / 1e6);
        return ma == 0 ? "0 Ma" : (ma > 0 ? "−" + ma : "+" + (-ma)) + " Ma";
    }

    static String duration(long ms) {
        long s = Math.max(0, ms) / 1000;
        if (s < 60)
            return s + "s";
        if (s < 3600)
            return (s / 60) + "m" + String.format(Locale.ROOT, "%02d", s % 60) + "s";
        return (s / 3600) + "h" + String.format(Locale.ROOT, "%02d", (s / 60) % 60) + "m";
    }

    private static String gib(long bytes) {
        return String.format(Locale.ROOT, "%.1f", bytes / GIB);
    }

    static String levelTypeFor(int c) {
        return switch (c) {
            case 16_384 -> "deeptime:planet";
            case 32_768 -> "deeptime:planet_32k";
            default -> "a Deep Time world preset with circumference_blocks " + c;
        };
    }

    private static String presetsFor(int c) {
        List<String> names = new ArrayList<>();
        for (String n : Presets.BUILTIN) {
            Presets.builtin(n).map(Presets::parseObject).map(Presets::info)
                    .filter(i -> i.circumference().isPresent() && i.circumference().getAsInt() == c)
                    .ifPresent(i -> names.add(n));
        }
        return names.isEmpty() ? "no built-in preset has that size" : String.join(", ", names);
    }

    private static String summary(String manifestJson) {
        try {
            JsonObject m = Presets.parseObject(manifestJson);
            JsonObject run = m.getAsJsonObject("notes").getAsJsonObject("run");
            JsonArray layers = m.getAsJsonArray("layers");
            return String.format(Locale.ROOT, "%d events, %d sites, %d keyframes, %d layers, source %s",
                    run.get("events").getAsLong(), run.get("sites").getAsLong(), run.get("keyframes").getAsLong(),
                    layers == null ? 0 : layers.size(), m.get("source").getAsString());
        } catch (RuntimeException e) {
            return "manifest written";
        }
    }

    private static String quote(String s) {
        return s.matches("[A-Za-z0-9_./:=+-]+") ? s : "'" + s.replace("'", "'\\''") + "'";
    }

    private static void deleteQuietly(Path p) {
        try {
            deleteTree(p);
        } catch (IOException e) {
            DeepTime.LOGGER.warn("[deeptime] could not remove {}: {}", p, e.toString());
        }
    }

    static void deleteTree(Path root) throws IOException {
        if (!Files.exists(root))
            return;
        Files.walkFileTree(root, new SimpleFileVisitor<>() {
            @Override
            public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) throws IOException {
                Files.delete(file);
                return FileVisitResult.CONTINUE;
            }

            @Override
            public FileVisitResult postVisitDirectory(Path d, IOException exc) throws IOException {
                if (exc != null)
                    throw exc;
                Files.delete(d);
                return FileVisitResult.CONTINUE;
            }
        });
    }
}
