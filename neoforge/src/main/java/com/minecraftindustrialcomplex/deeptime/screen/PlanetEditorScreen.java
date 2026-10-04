// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorModel;
import com.minecraftindustrialcomplex.deeptime.worldgen.PlanetGenerators;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.narration.NarratableEntry;
import net.minecraft.client.gui.components.Renderable;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.client.gui.screens.worldselection.WorldCreationContext;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;
import net.neoforged.fml.loading.FMLPaths;

import java.util.Collections;
import java.util.Map;
import java.util.Optional;
import java.util.WeakHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * The Deep Time planet editor (plan §7.12), opened by the Create World screen's Customize button
 * for {@code deeptime:planet}: the setup page, the run page (the timelapse) and the review page,
 * or the guidance panel when the natives did not load (plan §6.6). All state lives in an
 * {@link EditorModel}, one per Create World screen, so closing and reopening Customize keeps the
 * run, the timelapse and the accepted result.
 */
public final class PlanetEditorScreen extends Screen {

    /** One model per Create World screen (its UI state), dropped with it. */
    private static final Map<WorldCreationUiState, EditorModel> MODELS = Collections.synchronizedMap(new WeakHashMap<>());

    /** Decodes keyframes (plan §6.2: never on the render thread). */
    static final ExecutorService LOADER = daemon("deeptime-keyframes");
    /** Estimates, folder deletion, the review page's map. */
    static final ExecutorService IO = daemon("deeptime-editor-io");

    private final CreateWorldScreen parent;
    private final EditorModel model;
    private Page page;
    private Object shown;

    /** One of the editor's pages. */
    interface Page {
        void init(PlanetEditorScreen s);

        default void tick() {}

        /** Drawn after the widgets (text in free space). */
        default void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {}

        default void close() {}
    }

    private PlanetEditorScreen(CreateWorldScreen parent, EditorModel model) {
        super(Lang.tr("editor.title"));
        this.parent = parent;
        this.model = model;
    }

    /** The {@code PresetEditor} of {@code deeptime:planet}. */
    public static Screen create(CreateWorldScreen parent, WorldCreationContext context) {
        EditorModel m = model(parent.getUiState());
        // The dimensions lost the accepted binding (the world type was switched away and back): restore it.
        m.accepted().ifPresent(a -> {
            boolean bound = PlanetGenerators.overworld(context.selectedDimensions())
                    .flatMap(g -> g.dataRef().stagingId()).filter(a.stagingId()::equals).isPresent();
            if (!bound)
                apply(parent, a);
        });
        return new PlanetEditorScreen(parent, m);
    }

    /** The model of a Create World screen (created on first use). */
    static EditorModel model(WorldCreationUiState ui) {
        return MODELS.computeIfAbsent(ui, k -> new EditorModel(EditorModel.Deps.game(FMLPaths.GAMEDIR.get(), LOADER, IO,
                () -> packContent(ui))));
    }

    /**
     * The content bundle of a run started here: the pack placement tables in Deep Time's jar whose biomes the Create
     * World screen's data packs register (decisions log 2026-10-04; {@code climate.PlacementPacks}). Empty when none
     * apply or they cannot be read (the run then uses the default table; the log says why).
     */
    private static String packContent(WorldCreationUiState ui) {
        try {
            var chosen = com.minecraftindustrialcomplex.deeptime.climate.ClimatePlacement.chosen(
                    com.minecraftindustrialcomplex.deeptime.climate.ClimatePlacement.jarPacks(),
                    ui.getSettings().worldgenLoadContext());
            // The bake records the ones it used (notes.bake.climate.placement.patches).
            return com.minecraftindustrialcomplex.deeptime.climate.PlacementPacks.addTo("", chosen);
        } catch (java.io.IOException | RuntimeException e) {
            DeepTime.LOGGER.warn("[deeptime] pack placement tables not used: {}", e.toString());
            return "";
        }
    }

    /** Every live editor model's staging ids (kept by the abandoned-folder cleanup). */
    static java.util.Set<String> heldStagingIds() {
        java.util.Set<String> out = new java.util.HashSet<>();
        synchronized (MODELS) {
            for (EditorModel m : MODELS.values())
                out.addAll(m.heldStagingIds());
        }
        return out;
    }

    private static ExecutorService daemon(String name) {
        return Executors.newSingleThreadExecutor(r -> {
            Thread t = new Thread(r, name);
            t.setDaemon(true);
            return t;
        });
    }

    EditorModel model() {
        return model;
    }

    CreateWorldScreen parent() {
        return parent;
    }

    net.minecraft.client.gui.Font font() {
        return font;
    }

    /** Adds a widget of the current page. */
    <T extends GuiEventListener & Renderable & NarratableEntry> T add(T widget) {
        return addRenderableWidget(widget);
    }

    /** True when this editor can only show the guidance panel: no natives and nothing finished to review. */
    boolean guidanceOnly() {
        return !NativeLoader.isLinked() && model.accepted().isEmpty()
                && model.run().map(r -> r.state() != com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel.State.DONE).orElse(true);
    }

    private Object wanted() {
        return guidanceOnly() ? "guidance" : model.page();
    }

    @Override
    protected void init() {
        if (page != null)
            page.close();
        shown = wanted();
        if (guidanceOnly()) {
            page = new GuidancePanel();
        } else {
            page = switch (model.page()) {
                case SETUP -> new SetupPage();
                case RUN -> new RunPage();
                case REVIEW -> new ReviewPage();
            };
        }
        page.init(this);
    }

    @Override
    public void tick() {
        model.estimateRequest().ifPresent(json -> IO.execute(() -> {
            try {
                String r = model.deps().estimator().estimate(json);
                model.acceptEstimate(json, r, null);
            } catch (RuntimeException e) {
                model.acceptEstimate(json, null, e.getMessage() == null ? e.toString() : e.getMessage());
            }
        }));
        model.run().ifPresent(r -> r.poll());
        if (page != null)
            page.tick();
        if (!wanted().equals(shown))
            rebuildWidgets();
    }

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        super.render(g, mouseX, mouseY, partialTick);
        g.drawCenteredString(font, title, width / 2, 6, 0xffffffff);
        if (page != null)
            page.render(g, mouseX, mouseY, partialTick);
    }

    /** Back to the Create World screen; a running simulation keeps going (reopen Customize to see it). */
    @Override
    public void onClose() {
        minecraft.setScreen(parent);
    }

    @Override
    public void removed() {
        if (page != null)
            page.close();
        page = null;
        super.removed();
    }

    /** Page switches go through the model; the next tick rebuilds. */
    void refresh() {
        rebuildWidgets();
    }

    /** Accept: bind the world's overworld to the run's staging folder and return to Create World. */
    void accept() {
        EditorModel.Accepted a = model.accept();
        apply(parent, a);
        DeepTime.LOGGER.info("Deep Time: accepted run {} ({}); the world will be created from {}", a.stagingId(), a.label(),
                com.minecraftindustrialcomplex.deeptime.server.Staging.dir(FMLPaths.GAMEDIR.get(), a.stagingId()));
        minecraft.setScreen(parent);
    }

    private static void apply(CreateWorldScreen parent, EditorModel.Accepted a) {
        parent.getUiState().updateDimensions((registries, dims) ->
                PlanetGenerators.withStaging(registries, dims, a.stagingId(), a.circumference()));
    }

    /** The page's id, for the dev screenshot driver. */
    Optional<Page> page() {
        return Optional.ofNullable(page);
    }
}
