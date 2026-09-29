// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.preset.ShareCode;
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
 */
final class ScreenshotDriver {

    private static final String PROP = "deeptime.dev.screenshots";
    private static final int STEP_TIMEOUT_TICKS = 20 * 180;

    private enum Step { TITLE, CREATE_SCREEN, EDITOR, ADVANCED, TIMELINE, RUNNING, GLOBE, DONE, REVIEW, ACCEPTED, CREATING, IN_WORLD, QUIT }

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
                        } else if (advancedPreset() != null && model.importShareCode(ShareCode.encode(advancedPreset())).isEmpty()) {
                            // settings audit: the Advanced settings, on a preset that carries every knob.
                            model.setAdvancedOpen(true);
                            SetupPage.showAdvanced();
                            ed.refresh();
                            go(Step.ADVANCED);
                        } else {
                            model.setStart("supercontinent");
                            model.setLengthText("1000");
                            ed.refresh();
                            go(Step.TIMELINE);
                        }
                    }
                }
                case ADVANCED -> {
                    if (screen instanceof PlanetEditorScreen ed) {
                        if (stepTicks == 30) {
                            shot("02b-setup-advanced");
                            DeepTime.LOGGER.info("[deeptime-shot] advanced: {} of {} settings in the imported preset",
                                    model.advancedAvailable(), EditorFields.advancedFields().size());
                            SetupPage.scrollRows(16);
                            ed.refresh();
                        } else if (stepTicks == 60) {
                            shot("02c-setup-advanced-scrolled");
                            // Back to the built-in preset, Advanced closed, for the rest of the walk.
                            model.setAdvancedOpen(false);
                            model.selectFamily(model.families().get(0));
                            model.setStart("supercontinent");
                            model.setLengthText("1000");
                            ed.refresh();
                            go(Step.TIMELINE);
                        }
                    }
                }
                case TIMELINE -> {
                    if (screen instanceof PlanetEditorScreen ed && model.canRun() && stepTicks > 10) {
                        shot("03-setup-ready");
                        DeepTime.LOGGER.info("[deeptime-shot] estimate {} share code {}", model.estimate().orElse(null),
                                model.exportShareCode());
                        model.startRun();
                        ed.refresh();
                        go(Step.RUNNING);
                    }
                }
                case RUNNING -> {
                    if (screen instanceof PlanetEditorScreen ed && stepTicks == 70) {
                        shot("04-run-map-live");
                        RunPage.setGlobe(true);
                        ed.refresh();
                        go(Step.GLOBE);
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
                            DeepTime.LOGGER.info("[deeptime-shot] run {}: {} keyframes, {} events, layers {}", run.state(), run.frameCount(),
                                    run.events().size(), run.layers());
                            shot("06-run-done");
                            run.scrubTo(run.frameCount() / 2);
                        } else if (stepTicks == 60) {
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
                        go(Step.QUIT);
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
     * The preset JSON of {@code -Ddeeptime.dev.screenshots.preset=<file>} (tools/client/mac-client.sh passes
     * a pipeline preset), or null: the built-in presets carry none of the Advanced settings until P1-08, so
     * the driver imports one to show them.
     */
    private static String advancedPreset() {
        String path = System.getProperty(PROP + ".preset");
        if (path == null || path.isBlank())
            return null;
        try {
            return Files.readString(Path.of(path));
        } catch (IOException e) {
            DeepTime.LOGGER.error("[deeptime-shot] FAILED: cannot read the advanced-settings preset {}", path, e);
            return null;
        }
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
