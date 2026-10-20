// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.screen.model.CustomizeTopics;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorModel;
import com.minecraftindustrialcomplex.deeptime.screen.model.GenerationStatus;
import com.minecraftindustrialcomplex.deeptime.screen.model.LayerCatalog;
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
 * Development only: with {@code -Ddeeptime.dev.screenshots=true} the client walks Create World on
 * its own, as a player would (docs/design/create-world-ux.md), and takes screenshots into
 * {@code <gamedir>/screenshots/}: Create World as it opens (it must already be on Deep Time when
 * the natives loaded; the walk logs FAILED otherwise), the Customize page, the generation page that
 * Create opens (preparing, live with its History, Map and More tabs, a few layers, the globe, done,
 * scrubbed), the final map and spawn, then Play; with {@code -Ddeeptime.dev.screenshots.create=true}
 * the world is created (the staging move at server start) and shot in-world, else the walk stops
 * at Create World after Accept. Every step logs a {@code [deeptime-shot]} line. Used by
 * {@code tools/client/vm-client.sh} (the test VM, Xvfb) and {@code tools/client/mac-screenshots.sh};
 * never active in a normal game.
 *
 * <p>Properties ({@code -Ddeeptime.dev.screenshots.<name>}): {@code length=<Myr>} (the only setting
 * changed by default, so a walk takes minutes), {@code start=<start>} and {@code preset=<family>}
 * (left at the defaults unless given), {@code customize=false} skips the Customize shots,
 * {@code layers=a,b,…} the layers shot while the run goes (default plates, Köppen climate, rock,
 * provinces), {@code cancel=false} skips the cancel-and-create-again at the start (by default the
 * first run is cancelled once it has a frame, which returns to Create World; its staging folder must
 * go once it stopped, then Create is pressed again), {@code epochs=true} also picks the scrubbed
 * frame's epoch once the run is done ({@code 07b-epoch-picked}) and keeps going by
 * {@code continue=<Myr>} (default 50; {@code 07c-continued}), and {@code vista=x,y,z,yaw,pitch[;…]}
 * adds, after the in-world shot, one screenshot per viewpoint: the player is put there in spectator
 * mode, the render distance raised to 16 chunks, and the shot taken once the terrain around has had
 * time to generate and draw ({@code 11-vista-<n>}).
 */
final class ScreenshotDriver {

    private static final String PROP = "deeptime.dev.screenshots";
    private static final int STEP_TIMEOUT_TICKS = 20 * 180;

    private enum Step {
        TITLE, CREATE_SCREEN, CUSTOMIZE, ADVANCED, PRESS_CREATE, GENERATING, CANCEL, RUNNING, TABS, LAYERS, GLOBE, DONE, EPOCH,
        CONTINUE, REVIEW, ACCEPTED, CREATING, IN_WORLD, VISTA, QUIT
    }

    /** Layers shot mid-run ({@code .layers=a,b,…}; those the run has). */
    private static final String DEFAULT_LAYERS = "plates,climate/koppen,structure/rock,provinces";
    private static int layerShot;
    private static boolean cancelled;
    private static String cancelledId;
    private static TimelapseModel cancelledRun;

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

    /**
     * Puts the game's idea of the pointer in the top-left corner. Opening a screen centres it, and in
     * the centre it hovers a setting or the map, so a tooltip covered part of the shots. GLFW drops
     * the motion event of its own warp, so the handler's fields are set directly (a dev-only walk).
     */
    private static void parkPointer(Minecraft mc) {
        try {
            for (String name : new String[]{"xpos", "ypos"}) {
                java.lang.reflect.Field f = net.minecraft.client.MouseHandler.class.getDeclaredField(name);
                f.setAccessible(true);
                f.setDouble(mc.mouseHandler, 1.0);
            }
        } catch (ReflectiveOperationException | RuntimeException e) {
            DeepTime.LOGGER.warn("[deeptime-shot] could not park the pointer ({}); tooltips may show in the shots", e.toString());
        }
    }

    private static void status(String when) {
        DeepTime.LOGGER.info("[deeptime-shot] {}: status {} ({}), {}%", when, GenerationStatus.state(model),
                GenerationStatus.sentence(model), Math.round(GenerationStatus.fraction(model) * 100));
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
                        parkPointer(mc);
                        CreateWorldScreen.openFresh(mc, screen);
                        go(Step.CREATE_SCREEN);
                    }
                }
                case CREATE_SCREEN -> {
                    if (!(screen instanceof CreateWorldScreen cws))
                        return;
                    if (createScreen == null) {
                        createScreen = cws;
                        stepTicks = 0;
                    } else if (stepTicks == 15) {
                        // The screen exactly as Create World opens it, before the driver touches it.
                        WorldCreationUiState ui = cws.getUiState();
                        boolean deepTime = ui.getWorldType().preset() != null && ui.getWorldType().preset().is(PlanetGenerators.PLANET);
                        DeepTime.LOGGER.info("[deeptime-shot] fresh world type: {} (natives {})",
                                ui.getWorldType().preset() == null ? "custom"
                                        : ui.getWorldType().preset().unwrapKey().map(Object::toString).orElse("?"),
                                NativeLoader.isLinked() ? "loaded" : "missing");
                        if (NativeLoader.isLinked() && !deepTime)
                            DeepTime.LOGGER.error("[deeptime-shot] FAILED: Create World did not open on deeptime:planet");
                        shot("00-create-world-fresh");
                        worldTab(cws);
                        if (!deepTime)
                            ui.getNormalPresetList().stream()
                                    .filter(e -> e.preset() != null && e.preset().is(PlanetGenerators.PLANET)).findFirst()
                                    .ifPresentOrElse(ui::setWorldType,
                                            () -> DeepTime.LOGGER.error("[deeptime-shot] FAILED: deeptime:planet is not in the world type list"));
                    } else if (stepTicks == 35) {
                        DeepTime.LOGGER.info("[deeptime-shot] create button blocked: {}", CreateWorldGuard.block(cws.getUiState()));
                        shot("01-create-world-deeptime");
                        model = PlanetEditorScreen.model(cws.getUiState());
                        if (!NativeLoader.isLinked() || Boolean.parseBoolean(System.getProperty(PROP + ".customize", "true"))) {
                            var editor = cws.getUiState().getPresetEditor();
                            if (editor == null) {
                                DeepTime.LOGGER.error("[deeptime-shot] FAILED: no preset editor (Customize) for deeptime:planet");
                                go(Step.QUIT);
                                return;
                            }
                            mc.setScreen(editor.createEditScreen(cws, cws.getUiState().getSettings()));
                            go(Step.CUSTOMIZE);
                        } else {
                            applyProps();
                            go(Step.PRESS_CREATE);
                        }
                    }
                }
                case CUSTOMIZE -> {
                    if (!(screen instanceof PlanetEditorScreen))
                        stepTicks = 0;
                    if (screen instanceof PlanetEditorScreen ed && stepTicks == 20) {
                        guidance = ed.guidanceOnly();
                        DeepTime.LOGGER.info("[deeptime-shot] Customize opened: guidance={} canRun={} problems={}", guidance, model.canRun(),
                                model.problems());
                        if (guidance) {
                            shot("02-guidance-panel");
                            ed.onClose();
                            go(Step.ACCEPTED);
                        } else {
                            // The walk's settings, typed as a player would (the timeline shows as changed),
                            // and one setting edited to show its mark and reset; undone before Create.
                            applyProps();
                            EditorFields.Field seed = EditorFields.byId("seed");
                            if (model.available(seed))
                                model.setFieldText(seed, "20261019");
                            CustomizePage.setTopic(CustomizeTopics.Topic.PLANET);
                            ed.refresh();
                            go(Step.ADVANCED);
                        }
                    }
                }
                case ADVANCED -> {
                    // The Customize page's topics, then a search under Advanced.
                    if (screen instanceof PlanetEditorScreen ed) {
                        if (stepTicks == 20) {
                            shot("02-customize-planet");
                            DeepTime.LOGGER.info("[deeptime-shot] customize: {} changes, {} of {} settings in the preset",
                                    model.editCount(), EditorFields.ALL.stream().filter(model::available).count(), EditorFields.ALL.size());
                            CustomizePage.setTopic(CustomizeTopics.Topic.CLIMATE);
                            ed.refresh();
                        } else if (stepTicks == 40) {
                            shot("02b-customize-climate");
                            CustomizePage.setTopic(CustomizeTopics.Topic.ROCKS);
                            ed.refresh();
                        } else if (stepTicks == 60) {
                            shot("02c-customize-rocks");
                            CustomizePage.setTopic(CustomizeTopics.Topic.ADVANCED);
                            CustomizePage.setQuery("river");
                            ed.refresh();
                        } else if (stepTicks == 80) {
                            shot("02d-customize-search");
                            CustomizePage.setQuery("");
                            CustomizePage.setTopic(CustomizeTopics.Topic.PLANET);
                            model.resetField(EditorFields.byId("seed"));
                            ed.onClose(); // back to Create World, as the player would
                            go(Step.PRESS_CREATE);
                        }
                    }
                }
                case PRESS_CREATE -> {
                    if (screen instanceof CreateWorldScreen cws && stepTicks == 10) {
                        DeepTime.LOGGER.info("[deeptime-shot] pressing Create: preset {} ({} blocks), share code {}", model.presetName(),
                                model.circumference(), model.exportShareCode());
                        press(cws, "selectWorld.create");
                        go(Step.GENERATING);
                    }
                }
                case GENERATING -> {
                    // Create opened the generation page; it starts the run once the estimate is in.
                    if (!(screen instanceof PlanetEditorScreen ed)) {
                        if (stepTicks == 100) {
                            DeepTime.LOGGER.error("[deeptime-shot] FAILED: Create did not open the generation page (screen {})",
                                    screen == null ? "none" : screen.getClass().getName());
                            shot("zz-create-no-generation");
                            go(Step.QUIT);
                        }
                        return;
                    }
                    if (stepTicks == 3) {
                        shot(cancelled ? "03d-generation-again" : "03-generation-preparing");
                        status("generation page opened");
                    }
                    var run = model.run().filter(r -> !r.stagingId().equals(cancelledId));
                    if (run.isPresent() && stepTicks > 3) {
                        DeepTime.LOGGER.info("[deeptime-shot] run {} started after {} ticks: estimate {}, cap {} MiB, {} threads",
                                run.get().stagingId(), stepTicks, model.estimate().orElse(null), model.capBytes() >> 20,
                                model.deps().machine().clientThreads());
                        go(Boolean.parseBoolean(System.getProperty(PROP + ".cancel", "true")) && !cancelled ? Step.CANCEL : Step.RUNNING);
                    } else if (stepTicks == 600) {
                        DeepTime.LOGGER.error("[deeptime-shot] FAILED: no run started: {}", model.problems());
                        shot("zz-generation-did-not-start");
                        go(Step.QUIT);
                    } else if (GenerationStatus.state(model) == GenerationStatus.State.BLOCKED && stepTicks > 3) {
                        shot("zz-generation-blocked");
                        DeepTime.LOGGER.error("[deeptime-shot] FAILED: the generation page is blocked: {}", model.problems());
                        go(Step.QUIT);
                    }
                }
                case CANCEL -> {
                    // Cancel once the run has shown a frame: back to Create World, the folder goes, Create again.
                    if (!cancelled) {
                        if (!(screen instanceof PlanetEditorScreen ed))
                            return;
                        TimelapseModel run = model.run().orElseThrow();
                        if ((run.frameCount() >= 1 || stepTicks >= 20 * 120) && stepTicks >= 40) {
                            shot("03b-generation-before-cancel");
                            DeepTime.LOGGER.info("[deeptime-shot] cancelling run {} at {}", run.stagingId(), run.progressLine(model.capBytes()));
                            cancelledId = run.stagingId();
                            cancelledRun = run;
                            // The Cancel button.
                            model.cancelRun();
                            ed.onClose();
                            cancelled = true;
                            stepTicks = 0;
                        }
                    } else if (stepTicks == 5) {
                        shot("03c-create-world-after-cancel");
                        DeepTime.LOGGER.info("[deeptime-shot] after Cancel: screen {}, run state {}", screen == null ? "none"
                                : screen.getClass().getSimpleName(), cancelledRun.state());
                    } else if (stepTicks > 5 && cancelledRun.state() != TimelapseModel.State.RUNNING) {
                        Path old = com.minecraftindustrialcomplex.deeptime.server.Staging.dir(mc.gameDirectory.toPath(), cancelledId);
                        if (!Files.exists(old) && screen instanceof CreateWorldScreen cws) {
                            DeepTime.LOGGER.info("[deeptime-shot] the cancelled run stopped ({}) and its staging folder is gone after {} ticks; Create again",
                                    cancelledRun.state(), stepTicks);
                            press(cws, "selectWorld.create");
                            go(Step.GENERATING);
                        } else if (stepTicks >= 20 * 150) {
                            DeepTime.LOGGER.error("[deeptime-shot] FAILED: the cancelled run's staging folder {} is still there", old);
                            go(Step.QUIT);
                        }
                    } else if (stepTicks > 5) {
                        cancelledRun.poll(); // nobody polls it once its page is gone
                        if (stepTicks >= 20 * 170) {
                            DeepTime.LOGGER.error("[deeptime-shot] FAILED: the cancelled run did not stop ({})", cancelledRun.progressLine(model.capBytes()));
                            go(Step.QUIT);
                        }
                    }
                }
                case RUNNING -> {
                    TimelapseModel run = model.run().orElseThrow();
                    if (stepTicks % 400 == 0)
                        DeepTime.LOGGER.info("[deeptime-shot] running: {}", run.progressLine(model.capBytes()));
                    boolean enough = run.frameCount() >= 3 || run.state() != TimelapseModel.State.RUNNING;
                    if (!(screen instanceof PlanetEditorScreen ed) || !enough || stepTicks < 60)
                        return;
                    GenerationPage.setTab(GenerationPage.Tab.HISTORY);
                    ed.refresh();
                    shot("04-generation-live");
                    status("live");
                    DeepTime.LOGGER.info("[deeptime-shot] live: {} frames, {} events, layer {}, groups {}, layers {}", run.frameCount(),
                            run.events().size(), run.layer().orElse("-"), run.groups(), run.layers());
                    go(Step.TABS);
                }
                case TABS -> {
                    if (!(screen instanceof PlanetEditorScreen ed))
                        return;
                    if (stepTicks == 1) {
                        GenerationPage.setTab(GenerationPage.Tab.MORE);
                        ed.refresh();
                    } else if (stepTicks == 20) {
                        shot("04a-generation-more");
                        GenerationPage.setDetails(true);
                        ed.refresh();
                    } else if (stepTicks == 40) {
                        shot("04b-generation-details");
                        GenerationPage.setDetails(false);
                        GenerationPage.setTab(GenerationPage.Tab.MAP);
                        ed.refresh();
                        layerShot = 0;
                        go(Step.LAYERS);
                    }
                }
                case LAYERS -> {
                    TimelapseModel run = model.run().orElseThrow();
                    if (!(screen instanceof PlanetEditorScreen ed))
                        return;
                    List<String> wanted = java.util.Arrays.stream(System.getProperty(PROP + ".layers", DEFAULT_LAYERS).split(","))
                            .map(String::trim).filter(run.layers()::contains).toList();
                    if (stepTicks == 1) {
                        if (layerShot >= wanted.size()) {
                            run.setLayer(LayerCatalog.defaultLayer(run.layers()));
                            run.scrubTo(run.frameCount() - 1); // follow live again
                            GenerationPage.setGlobe(true);
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
                        shot("05-generation-globe");
                        GenerationPage.setGlobe(false);
                        GenerationPage.setTab(GenerationPage.Tab.HISTORY);
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
                    if (stepTicks == 40) {
                        var p = run.progress();
                        Runtime rt = Runtime.getRuntime();
                        DeepTime.LOGGER.info("[deeptime-shot] run {}: {} keyframes, {} events, layers {}", run.state(), run.frameCount(),
                                run.events().size(), run.layers());
                        DeepTime.LOGGER.info("[deeptime-shot] run cost: {} s, native peak {} MiB (cap {} MiB), {} sim threads, "
                                        + "JVM heap {} of {} MiB, {} frames decoded in the ring",
                                p.elapsedMs() / 1000, p.nativeBytesPeak() >> 20, model.capBytes() >> 20, model.deps().machine().clientThreads(),
                                (rt.totalMemory() - rt.freeMemory()) >> 20, rt.maxMemory() >> 20, run.ringSize());
                        status("done");
                        shot("06-generation-done");
                        GenerationPage.setTab(GenerationPage.Tab.MAP);
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
                        run.setLayer(LayerCatalog.defaultLayer(run.layers()));
                        run.scrubTo(run.frameCount() / 2);
                        GenerationPage.setTab(GenerationPage.Tab.HISTORY);
                        ed.refresh();
                    } else if (stepTicks == 130) {
                        shot("07-generation-scrubbed");
                        if (run.state() != TimelapseModel.State.DONE) {
                            DeepTime.LOGGER.error("[deeptime-shot] FAILED: run ended {} {}", run.state(), run.failure().orElse(""));
                            go(Step.QUIT);
                            return;
                        }
                        if (Boolean.getBoolean(PROP + ".epochs")) {
                            // The latest frame before the run's end (the end's own frames have
                            // nothing to pick): shown now, shot and picked next.
                            int pick = -1;
                            for (int i = 0; i < run.frameCount(); i++)
                                if (model.epochStopMyr(run.frames().get(i)).isPresent())
                                    pick = i;
                            if (pick >= 0) {
                                run.scrubTo(pick);
                                ed.refresh();
                                return;
                            }
                        }
                        model.openReview();
                        ed.refresh();
                        go(Step.REVIEW);
                    } else if (stepTicks == 160) {
                        shot("07a-epoch-to-pick");
                        var f = run.current().orElseThrow();
                        var stop = model.epochStopMyr(f);
                        DeepTime.LOGGER.info("[deeptime-shot] epoch: frame {} at t={} -> stop {}", f.n(), f.t(), stop);
                        if (stop.isPresent())
                            model.pickEpoch(f);
                        else
                            model.openReview();
                        ed.refresh();
                        go(stop.isPresent() ? Step.EPOCH : Step.REVIEW);
                    }
                }
                case EPOCH, CONTINUE -> {
                    TimelapseModel run = model.run().orElseThrow();
                    if (run.state() == TimelapseModel.State.RUNNING || !(screen instanceof PlanetEditorScreen ed)) {
                        stepTicks = 0;
                        return;
                    }
                    if (stepTicks == 40) {
                        var p = run.progress();
                        DeepTime.LOGGER.info("[deeptime-shot] {} run {}: stop {} Myr, {} keyframes, {} s", step, run.state(),
                                model.runStopMyr(), run.frameCount(), p.elapsedMs() / 1000);
                        shot(step == Step.EPOCH ? "07b-epoch-picked" : "07c-continued");
                        if (run.state() != TimelapseModel.State.DONE) {
                            DeepTime.LOGGER.error("[deeptime-shot] FAILED: {} run ended {} {}", step, run.state(), run.failure().orElse(""));
                            go(Step.QUIT);
                            return;
                        }
                        if (step == Step.EPOCH) {
                            double more = Double.parseDouble(System.getProperty(PROP + ".continue", "50"));
                            model.continueTo(model.runStopMyr().orElseThrow() + more);
                            ed.refresh();
                            go(Step.CONTINUE);
                        } else {
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
                        if (Boolean.getBoolean(PROP + ".create")) {
                            DeepTime.LOGGER.info("[deeptime-shot] Play");
                            ed.play();
                            go(Step.CREATING);
                        } else {
                            ed.accept();
                            go(Step.ACCEPTED);
                        }
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
                            go(Step.QUIT);
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
                        DeepTime.LOGGER.info("[deeptime-shot] in world at {}, generator {}", mc.player == null ? "?" : mc.player.blockPosition(),
                                mc.getSingleplayerServer() == null ? "?" : mc.getSingleplayerServer().overworld().getChunkSource().getGenerator().getClass().getSimpleName());
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

    /**
     * The walk's settings: the length ({@code .length}, default 300 Myr so a walk takes minutes)
     * and, only when given, the start and the built-in family. Everything else stays at the defaults
     * a player gets.
     */
    private static void applyProps() {
        String family = System.getProperty(PROP + ".preset", "");
        if (!family.isEmpty() && model.families().contains(family))
            model.selectFamily(family);
        String start = System.getProperty(PROP + ".start", "");
        if (!start.isEmpty())
            model.setStart(start);
        model.setLengthText(System.getProperty(PROP + ".length", "300"));
        DeepTime.LOGGER.info("[deeptime-shot] settings: preset {}, length {} Myr", model.presetName(), System.getProperty(PROP + ".length", "300"));
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
        DeepTime.LOGGER.error("[deeptime-shot] FAILED: no '{}' button", key);
    }
}
