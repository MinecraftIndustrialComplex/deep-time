// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorModel;
import com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel;
import com.minecraftindustrialcomplex.deeptime.worldgen.PlanetGenerators;
import com.mojang.blaze3d.platform.NativeImage;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.components.tabs.TabNavigationBar;
import net.minecraft.client.gui.screens.ConfirmScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.neoforged.neoforge.client.event.ClientTickEvent;
import net.neoforged.neoforge.common.NeoForge;

import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/**
 * Development only: with {@code -Ddeeptime.dev.screenshots=true} the client walks the planet
 * editor on its own and takes screenshots into {@code <gamedir>/screenshots/} — the Create World
 * screen with the Deep Time preset, the setup page before and after choosing a timeline, the
 * timelapse (map and globe) while running and when done, the review page, and the Create World
 * screen after Accept — then, with {@code -Ddeeptime.dev.screenshots.create=true}, creates the
 * world (the staging move at server start) and shoots it in-world, and quits. Every step logs a
 * {@code [deeptime-shot]} line. Used by {@code tools/client/mac-screenshots.sh} (the client on the
 * Mac's desktop session; P1-11's laptop route was {@code tools/client/screenshots.sh}), P1-11's
 * best-effort client check, plan §9.8; never active in a normal game.
 *
 * <p>{@code -Ddeeptime.dev.screenshots.start=<start>} and {@code .length=<Myr>} choose the timeline
 * (default supercontinent, 1000), {@code .preset=<family>} the built-in family (default the first),
 * {@code .layers=a,b,…} the layers shot while the run goes (default plates, Köppen climate, rock,
 * provinces), {@code .cancel=false} skips the cancel-and-restart at the start of the run (by default
 * the first run is cancelled once it has a frame, its staging folder must go, and a second run
 * starts at once, which waits for the first to stop), and {@code .vista=x,y,z,yaw,pitch[;…]} adds, after the in-world
 * shot, one screenshot per viewpoint: the player is put there in spectator mode, the render
 * distance raised to 16 chunks, and the shot taken once the terrain around has had time to
 * generate and draw ({@code 11-vista-<n>}).
 */
final class ScreenshotDriver {

    private static final String PROP = "deeptime.dev.screenshots";
    private static final int STEP_TIMEOUT_TICKS = 20 * 180;

    private enum Step { TITLE, CREATE_SCREEN, EDITOR, ADVANCED, TIMELINE, CANCEL, RUNNING, LAYERS, GLOBE, DONE, REVIEW, ACCEPTED, CREATING, IN_WORLD, VISTA, QUIT }

    /** Layers shot mid-run ({@code .layers=a,b,…}; those the run has). */
    private static final String DEFAULT_LAYERS = "plates,climate/koppen,structure/rock,provinces";
    private static int layerShot;
    private static boolean cancelled;
    private static String cancelledId;

    /** A vista waits until the chunk 160 blocks ahead is loaded, then this long for it to draw … */
    private static final int VISTA_SETTLE_TICKS = 20 * 15;
    /** … and shoots anyway after this long (the step timeout is 180 s). */
    private static final int VISTA_MAX_TICKS = 20 * 170;
    private static int vista;
    private static int vistaLoadedAt = -1;

    private static Step step = Step.TITLE;
    private static int ticks;
    private static int stepTicks;
    private static CreateWorldScreen createScreen;
    private static EditorModel model;
    private static boolean guidance;

    private ScreenshotDriver() {}

    static void init() {
        if (!Boolean.getBoolean(PROP))
            return;
        DeepTime.LOGGER.warn("[deeptime-shot] screenshot driver active (-D{}=true): the client drives itself", PROP);
        NeoForge.EVENT_BUS.addListener(ClientTickEvent.Post.class, e -> tick());
    }

    private static void go(Step next) {
        DeepTime.LOGGER.info("[deeptime-shot] step {} -> {}", step, next);
        step = next;
        stepTicks = 0;
    }

    /**
     * Reads the framebuffer ({@code Screenshot.takeScreenshot}) and writes the PNG with ImageIO:
     * the game's own writer (STB) failed under Xvfb on NixOS ("Could not write image to the PNG file").
     */
    private static void shot(String name) {
        Minecraft mc = Minecraft.getInstance();
        try (NativeImage img = Screenshot.takeScreenshot(mc.getMainRenderTarget())) {
            int w = img.getWidth(), h = img.getHeight();
            BufferedImage out = new BufferedImage(w, h, BufferedImage.TYPE_INT_RGB);
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++) {
                    int abgr = img.getPixelRGBA(x, y);
                    out.setRGB(x, y, (abgr & 0xff) << 16 | (abgr & 0xff00) | (abgr >> 16) & 0xff);
                }
            Path dir = mc.gameDirectory.toPath().resolve("screenshots");
            Files.createDirectories(dir);
            Path file = dir.resolve(name + ".png");
            ImageIO.write(out, "png", file.toFile());
            DeepTime.LOGGER.info("[deeptime-shot] {} -> {} ({}x{})", name, file, w, h);
        } catch (IOException | RuntimeException e) {
            DeepTime.LOGGER.error("[deeptime-shot] screenshot {} failed", name, e);
        }
    }

    private static void tick() {
        Minecraft mc = Minecraft.getInstance();
        Screen screen = mc.screen;
        ticks++;
        stepTicks++;
        if (stepTicks > STEP_TIMEOUT_TICKS && step != Step.QUIT) {
            DeepTime.LOGGER.error("[deeptime-shot] FAILED: step {} timed out (screen {})", step, screen == null ? "none" : screen.getClass().getName());
            shot("zz-timeout-" + step.name().toLowerCase());
            go(Step.QUIT);
            return;
        }
        try {
            switch (step) {
                case TITLE -> {
                    if (screen instanceof TitleScreen && stepTicks > 40) {
                        CreateWorldScreen.openFresh(mc, screen);
                        go(Step.CREATE_SCREEN);
                    }
                }
                case CREATE_SCREEN -> {
                    if (screen instanceof CreateWorldScreen cws) {
                        if (createScreen == null) {
                            createScreen = cws;
                            WorldCreationUiState ui = cws.getUiState();
                            ui.getNormalPresetList().stream()
                                    .filter(e -> e.preset() != null && e.preset().is(PlanetGenerators.PLANET)).findFirst()
                                    .ifPresentOrElse(ui::setWorldType,
                                            () -> DeepTime.LOGGER.error("[deeptime-shot] FAILED: deeptime:planet is not in the world type list"));
                            worldTab(cws);
                            stepTicks = 0;
                        } else if (stepTicks == 20) {
                            DeepTime.LOGGER.info("[deeptime-shot] create button blocked: {}", CreateWorldGuard.block(cws.getUiState()));
                            shot("01-create-world-deeptime");
                            var editor = cws.getUiState().getPresetEditor();
                            if (editor == null) {
                                DeepTime.LOGGER.error("[deeptime-shot] FAILED: no preset editor for deeptime:planet");
                                go(Step.QUIT);
                                return;
                            }
                            mc.setScreen(editor.createEditScreen(cws, cws.getUiState().getSettings()));
                            go(Step.EDITOR);
                        }
                    }
                }
                case EDITOR -> {
                    if (!(screen instanceof PlanetEditorScreen))
                        stepTicks = 0;
                    if (screen instanceof PlanetEditorScreen ed && stepTicks == 20) {
                        model = ed.model();
                        guidance = ed.guidanceOnly();
                        shot(guidance ? "02-guidance-panel" : "02-setup-no-timeline");
                        DeepTime.LOGGER.info("[deeptime-shot] editor opened: guidance={} canRun={} problems={}", guidance, model.canRun(),
                                model.problems());
                        if (guidance) {
                            ed.onClose();
                            go(Step.ACCEPTED);
                        } else if (model.advancedAvailable() > 0) {
                            // The settings audit: the Advanced settings, open and scrolled through.
                            model.setAdvancedOpen(true);
                            SetupPage.showAdvanced();
                            ed.refresh();
                            go(Step.ADVANCED);
                        } else {
                            chooseFamily();
                            model.setStart(System.getProperty(PROP + ".start", "supercontinent"));
                            model.setLengthText(System.getProperty(PROP + ".length", "1000"));
                            ed.refresh();
                            go(Step.TIMELINE);
                        }
                    }
                }
                case ADVANCED -> {
                    if (screen instanceof PlanetEditorScreen ed) {
                        if (stepTicks == 30) {
                            shot("02b-setup-advanced");
                            DeepTime.LOGGER.info("[deeptime-shot] advanced: {} of {} settings in the preset",
                                    model.advancedAvailable(), EditorFields.advancedFields().size());
                            SetupPage.scrollRows(16);
                            ed.refresh();
                        } else if (stepTicks == 60) {
                            shot("02c-setup-advanced-scrolled");
                            // Back to the built-in preset, Advanced closed, for the rest of the walk.
                            model.setAdvancedOpen(false);
                            chooseFamily();
                            model.setStart(System.getProperty(PROP + ".start", "supercontinent"));
                            model.setLengthText(System.getProperty(PROP + ".length", "1000"));
                            ed.refresh();
                            go(Step.TIMELINE);
                        }
                    }
                }
                case TIMELINE -> {
                    if (screen instanceof PlanetEditorScreen ed && model.canRun() && stepTicks > 10) {
                        shot("03-setup-ready");
                        DeepTime.LOGGER.info("[deeptime-shot] preset {} estimate {} cap {} threads {} share code {}", model.presetName(),
                                model.estimate().orElse(null), model.capBytes(), model.deps().machine().clientThreads(), model.exportShareCode());
                        model.startRun();
                        ed.refresh();
                        go(Boolean.parseBoolean(System.getProperty(PROP + ".cancel", "true")) && !cancelled ? Step.CANCEL : Step.RUNNING);
                    } else if (stepTicks == 600) {
                        DeepTime.LOGGER.error("[deeptime-shot] FAILED: Run stays disabled: {}", model.problems());
                        shot("zz-run-disabled");
                        go(Step.QUIT);
                    }
                }
                case CANCEL -> {
                    // Cancel the first run once it has shown a frame: its staging folder must go, and the
                    // next run, started at once, must wait for it to stop (SimSession's launch gate).
                    if (!(screen instanceof PlanetEditorScreen ed))
                        return;
                    TimelapseModel run = model.run().orElseThrow();
                    if (!cancelled) {
                        if ((run.frameCount() >= 1 || stepTicks >= 20 * 120) && stepTicks >= 40) {
                            shot("03b-run-before-cancel");
                            DeepTime.LOGGER.info("[deeptime-shot] cancelling run {} at {}", run.stagingId(), run.progressLine(model.capBytes()));
                            cancelledId = run.stagingId();
                            model.cancelRun();
                            cancelled = true;
                            TimelapseModel next = model.startRun().orElseThrow();
                            ed.refresh();
                            DeepTime.LOGGER.info("[deeptime-shot] restarted at once: waiting for the previous run = {}, launched sessions alive = {}",
                                    next.run().waitingForPrevious(), com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession.launchedLive());
                            stepTicks = 0;
                        }
                    } else if (stepTicks == 10) {
                        shot("03c-run-restarted-after-cancel");
                    } else if (stepTicks > 10 && stepTicks % 20 == 0) {
                        java.nio.file.Path old = com.minecraftindustrialcomplex.deeptime.server.Staging.dir(mc.gameDirectory.toPath(), cancelledId);
                        if (!java.nio.file.Files.exists(old)) {
                            DeepTime.LOGGER.info("[deeptime-shot] the cancelled run's staging folder is gone after {} ticks; the new run waits = {}",
                                    stepTicks, run.run().waitingForPrevious());
                            go(Step.RUNNING);
                        } else if (stepTicks >= 20 * 150) {
                            DeepTime.LOGGER.error("[deeptime-shot] FAILED: the cancelled run's staging folder {} is still there", old);
                            go(Step.RUNNING);
                        }
                    }
                }
                case RUNNING -> {
                    TimelapseModel run = model.run().orElseThrow();
                    if (stepTicks % 400 == 0)
                        DeepTime.LOGGER.info("[deeptime-shot] running: {}", run.progressLine(model.capBytes()));
                    boolean enough = run.frameCount() >= 3 || run.state() != TimelapseModel.State.RUNNING;
                    if (!(screen instanceof PlanetEditorScreen) || !enough || stepTicks < 60)
                        return;
                    shot("04-run-map-live");
                    DeepTime.LOGGER.info("[deeptime-shot] live: {} frames, {} events, layer {}, groups {}, layers {}", run.frameCount(),
                            run.events().size(), run.layer().orElse("-"), run.groups(), run.layers());
                    layerShot = 0;
                    go(Step.LAYERS);
                }
                case LAYERS -> {
                    TimelapseModel run = model.run().orElseThrow();
                    if (!(screen instanceof PlanetEditorScreen ed))
                        return;
                    List<String> wanted = java.util.Arrays.stream(System.getProperty(PROP + ".layers", DEFAULT_LAYERS).split(","))
                            .map(String::trim).filter(run.layers()::contains).toList();
                    if (stepTicks == 1) {
                        if (layerShot >= wanted.size()) {
                            run.setLayer(com.minecraftindustrialcomplex.deeptime.screen.model.LayerCatalog.defaultLayer(run.layers()));
                            run.scrubTo(run.frameCount() - 1); // follow live again
                            RunPage.setGlobe(true);
                            ed.refresh();
                            go(Step.GLOBE);
                            return;
                        }
                        run.setLayer(wanted.get(layerShot));
                        ed.refresh();
                    } else if (stepTicks == 30) {
                        String name = wanted.get(layerShot);
                        shot("04-layer-" + (layerShot + 1) + "-" + name.replaceAll("[^a-z0-9]+", "-"));
                        DeepTime.LOGGER.info("[deeptime-shot] layer {} (group {}, {} layers in it)", name, run.group(), run.groupLayers().size());
                        layerShot++;
                        stepTicks = 0;
                    }
                }
                case GLOBE -> {
                    if (screen instanceof PlanetEditorScreen ed && stepTicks == 20) {
                        shot("05-run-globe-live");
                        RunPage.setGlobe(false);
                        ed.refresh();
                        go(Step.DONE);
                    }
                }
                case DONE -> {
                    TimelapseModel run = model.run().orElseThrow();
                    if (run.state() == TimelapseModel.State.RUNNING || !(screen instanceof PlanetEditorScreen ed)) {
                        stepTicks = 0; // count from the end of the run
                        return;
                    }
                    {
                        if (stepTicks == 40) {
                            var p = run.progress();
                            Runtime rt = Runtime.getRuntime();
                            DeepTime.LOGGER.info("[deeptime-shot] run {}: {} keyframes, {} events, layers {}", run.state(), run.frameCount(),
                                    run.events().size(), run.layers());
                            DeepTime.LOGGER.info("[deeptime-shot] run cost: {} s, native peak {} MiB (cap {} MiB), {} sim threads, "
                                            + "JVM heap {} of {} MiB, {} frames decoded in the ring",
                                    p.elapsedMs() / 1000, p.nativeBytesPeak() >> 20, model.capBytes() >> 20, model.deps().machine().clientThreads(),
                                    (rt.totalMemory() - rt.freeMemory()) >> 20, rt.maxMemory() >> 20, run.ringSize());
                            shot("06-run-done");
                            if (run.layers().contains("deposits/sites"))
                                run.setLayer("deposits/sites");
                            ed.refresh();
                        } else if (stepTicks == 70) {
                            shot("06b-ore-sites");
                            if (run.layers().contains("deposits/favourability/porphyry_cu_mo_au"))
                                run.setLayer("deposits/favourability/porphyry_cu_mo_au");
                            ed.refresh();
                        } else if (stepTicks == 100) {
                            shot("06c-ore-favourability");
                            run.setLayer(com.minecraftindustrialcomplex.deeptime.screen.model.LayerCatalog.defaultLayer(run.layers()));
                            run.scrubTo(run.frameCount() / 2);
                            ed.refresh();
                        } else if (stepTicks == 130) {
                            shot("07-run-scrubbed");
                            if (run.state() != TimelapseModel.State.DONE) {
                                DeepTime.LOGGER.error("[deeptime-shot] FAILED: run ended {} {}", run.state(), run.failure().orElse(""));
                                go(Step.QUIT);
                                return;
                            }
                            model.openReview();
                            ed.refresh();
                            go(Step.REVIEW);
                        }
                    }
                }
                case REVIEW -> {
                    if (screen instanceof PlanetEditorScreen ed && model.review().map(r -> r.ready()).orElse(false) && stepTicks > 20) {
                        shot("08-review");
                        DeepTime.LOGGER.info("[deeptime-shot] review: spawn={} provinces={} error={}",
                                model.review().get().spawn().map(p -> p[0] + "," + p[1]).orElse("none"),
                                model.review().get().provinces().size(), model.review().get().error().orElse("none"));
                        ed.accept();
                        go(Step.ACCEPTED);
                    }
                }
                case ACCEPTED -> {
                    if (!(screen instanceof CreateWorldScreen))
                        stepTicks = 0;
                    if (screen instanceof CreateWorldScreen cws) {
                        if (stepTicks == 2)
                            worldTab(cws);
                        if (stepTicks == 20) {
                            var block = CreateWorldGuard.block(cws.getUiState());
                            DeepTime.LOGGER.info("[deeptime-shot] after {}: create block = {}", guidance ? "guidance" : "accept", block);
                            shot(guidance ? "03-create-world-natives-missing" : "09-create-world-accepted");
                            if (!guidance && Boolean.getBoolean(PROP + ".create") && block == PlanetGenerators.CreateBlock.NONE) {
                                press(cws, "selectWorld.create");
                                go(Step.CREATING);
                            } else {
                                go(Step.QUIT);
                            }
                        }
                    }
                }
                case CREATING -> {
                    if (screen instanceof ConfirmScreen confirm && stepTicks % 20 == 0) {
                        // an experimental-settings confirmation: proceed
                        for (GuiEventListener l : List.copyOf(confirm.children()))
                            if (l instanceof Button b) {
                                b.onPress();
                                break;
                            }
                    }
                    if (mc.level != null && mc.player != null)
                        go(Step.IN_WORLD);
                }
                case IN_WORLD -> {
                    if (mc.screen != null)
                        stepTicks = Math.min(stepTicks, 100); // still loading terrain: wait for the world view
                    if (stepTicks == 200) {
                        shot("10-in-world");
                        DeepTime.LOGGER.info("[deeptime-shot] in world at {}", mc.player == null ? "?" : mc.player.blockPosition());
                        go(vistas().isEmpty() ? Step.QUIT : Step.VISTA);
                    }
                }
                case VISTA -> {
                    List<double[]> vs = vistas();
                    if (vista >= vs.size()) {
                        go(Step.QUIT);
                        return;
                    }
                    double[] v = vs.get(vista);
                    if (stepTicks == 1) {
                        mc.options.renderDistance().set(16);
                        var server = mc.getSingleplayerServer();
                        var uuid = mc.player.getUUID();
                        server.execute(() -> {
                            var sp = server.getPlayerList().getPlayer(uuid);
                            if (sp != null) {
                                sp.setGameMode(net.minecraft.world.level.GameType.SPECTATOR);
                                sp.teleportTo(sp.serverLevel(), v[0], v[1], v[2], (float) v[3], (float) v[4]);
                            }
                        });
                        DeepTime.LOGGER.info("[deeptime-shot] vista {} at {},{},{} yaw {} pitch {}", vista, v[0], v[1], v[2], v[3], v[4]);
                    }
                    if (mc.screen != null)
                        stepTicks = Math.min(stepTicks, 2);
                    if (stepTicks == 2)
                        vistaLoadedAt = -1;
                    // The chunk 160 blocks ahead along the view: loaded means the terrain there exists.
                    double yaw = Math.toRadians(v[3]);
                    int ax = (int) Math.floor(v[0] - Math.sin(yaw) * 160), az = (int) Math.floor(v[2] + Math.cos(yaw) * 160);
                    if (vistaLoadedAt < 0 && stepTicks > 100 && mc.level != null
                            && mc.level.getChunkSource().hasChunk(ax >> 4, az >> 4))
                        vistaLoadedAt = stepTicks;
                    if ((vistaLoadedAt >= 0 && stepTicks == vistaLoadedAt + VISTA_SETTLE_TICKS) || stepTicks == VISTA_MAX_TICKS) {
                        DeepTime.LOGGER.info("[deeptime-shot] vista {}: ahead chunk loaded at tick {}", vista, vistaLoadedAt);
                        shot("11-vista-" + vista);
                        vista++;
                        stepTicks = 0;
                    }
                }
                case QUIT -> {
                    if (stepTicks == 20) {
                        DeepTime.LOGGER.info("[deeptime-shot] done after {} ticks", ticks);
                        mc.stop();
                    }
                }
            }
        } catch (RuntimeException e) {
            DeepTime.LOGGER.error("[deeptime-shot] FAILED at step {}", step, e);
            go(Step.QUIT);
        }
    }

    /** The built-in family of {@code .preset} (default the first). */
    private static void chooseFamily() {
        String want = System.getProperty(PROP + ".preset", "");
        model.selectFamily(model.families().contains(want) ? want : model.families().get(0));
    }

    /** The {@code .vista} viewpoints: x, y, z, yaw, pitch each. */
    private static List<double[]> vistas() {
        String p = System.getProperty(PROP + ".vista", "");
        List<double[]> out = new java.util.ArrayList<>();
        for (String v : p.split(";")) {
            String[] f = v.trim().split(",");
            if (f.length == 5) {
                double[] d = new double[5];
                for (int i = 0; i < 5; i++)
                    d[i] = Double.parseDouble(f[i].trim());
                out.add(d);
            }
        }
        return out;
    }

    private static void worldTab(CreateWorldScreen cws) {
        // A tab switch replaces the screen's widgets: iterate a copy.
        for (GuiEventListener l : List.copyOf(cws.children()))
            if (l instanceof TabNavigationBar bar) {
                bar.selectTab(1, false);
                return;
            }
    }

    private static void press(Screen s, String key) {
        for (GuiEventListener l : List.copyOf(s.children()))
            if (l instanceof Button b && b.getMessage().getContents() instanceof TranslatableContents t && key.equals(t.getKey())) {
                b.onPress();
                return;
            }
        DeepTime.LOGGER.error("[deeptime-shot] no '{}' button", key);
    }
}
