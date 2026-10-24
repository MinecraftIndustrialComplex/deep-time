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
 * The Deep Time planet editor (plan §7.12; docs/design/create-world-ux.md): the generation page
 * (the timelapse, opened by Create World's Create button through {@link #generate}), the Customize
 * page (opened by the World tab's Customize button for {@code deeptime:planet}) and the review page,
 * or the guidance panel when the natives did not load (plan §6.6). All state lives in an
 * {@link EditorModel}, one per Create World screen, so leaving and coming back keeps the run, the
 * timelapse and the accepted result.
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
    /** The last run whose failure was logged. */
    private Object reportedFailure;

    /** One of the editor's pages. */
    interface Page {
        void init(PlanetEditorScreen s);

        default void tick() {}

        /** Drawn after the widgets (text in free space). */
        default void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {}

        /** The page's title, drawn at the top. */
        default net.minecraft.network.chat.Component title() {
            return Lang.tr("editor.title");
        }

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
        followWorldType(parent.getUiState(), m);
        // The dimensions lost the accepted binding (the world type was switched away and back): restore it.
        m.accepted().ifPresent(a -> {
            boolean bound = PlanetGenerators.overworld(context.selectedDimensions())
                    .flatMap(g -> g.dataRef().stagingId()).filter(a.stagingId()::equals).isPresent();
            if (!bound)
                apply(parent, a);
        });
        return new PlanetEditorScreen(parent, m);
    }

    /**
     * Create World's Create button on a Deep Time world without world data
     * ({@link CreateWorldFlow#interceptCreate}): the generation page, simulating the current settings
     * (the defaults unless Customize changed them) as soon as the estimate is in.
     */
    static Screen generate(CreateWorldScreen parent) {
        EditorModel m = model(parent.getUiState());
        followWorldType(parent.getUiState(), m);
        m.generate();
        return create(parent, parent.getUiState().getSettings());
    }

    /**
     * A cube world type ({@code deeptime:planet_cube[_32k]}, cube plan G13) picks the cube projection
     * in the editor when Customize opens or Create is pressed, so the world type a player chose is the
     * world they get; the editor's own projection field still decides (as for Lambert), and nothing
     * is changed while a run is accepted.
     */
    static void followWorldType(WorldCreationUiState ui, EditorModel m) {
        var type = ui.getWorldType().preset();
        if (type == null || !(type.is(PlanetGenerators.PLANET_CUBE) || type.is(PlanetGenerators.PLANET_CUBE_32K))
                || m.accepted().isPresent())
            return;
        var projection = com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields.byId("projection");
        if (!m.choice(projection).filter("cube"::equals).isPresent())
            m.setChoice(projection, "cube");
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

    /**
     * The Create World flow ended (title screen, world list, server list, or the new world
     * loading): every editor's run that was not accepted stops ({@link EditorModel#abandon}).
     */
    static void abandonAll() {
        java.util.List<EditorModel> models;
        synchronized (MODELS) {
            models = java.util.List.copyOf(MODELS.values());
            MODELS.clear();
        }
        for (EditorModel m : models) {
            if (m.run().map(r -> r.state() == com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel.State.RUNNING).orElse(false)
                    && m.accepted().map(a -> !a.stagingId().equals(m.run().get().stagingId())).orElse(true))
                com.minecraftindustrialcomplex.deeptime.DeepTime.LOGGER.info("Deep Time: the Create World screen closed; cancelling the run that was not accepted");
            m.abandon();
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
                case SETUP -> new CustomizePage();
                case RUN -> new GenerationPage();
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
        model.run().ifPresent(r -> {
            r.poll();
            // A run that failed says why in the log too (the page shows it), once per run.
            if (r.state() == com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel.State.FAILED && r != reportedFailure) {
                reportedFailure = r;
                DeepTime.LOGGER.warn("Deep Time: run {} failed: {}", r.stagingId(), r.failure().orElse("?"));
            }
        });
        if (model.tickAutoStart().isPresent())
            DeepTime.LOGGER.info("Deep Time: generating {} ({}, {} blocks)", model.run().map(r -> r.stagingId()).orElse("?"),
                    model.presetName(), model.circumference());
        if (page != null)
            page.tick();
        if (!wanted().equals(shown))
            rebuildWidgets();
    }

    /**
     * An opaque backdrop (docs/design/create-world-ux.md: the map is the hero, not a picture
     * floating over the dimmed title panorama): the menus' own dark texture over solid black.
     */
    @Override
    public void renderBackground(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        g.fill(0, 0, width, height, 0xff0c0e12);
        renderMenuBackground(g);
    }

    @Override
    public boolean keyPressed(int keyCode, int scanCode, int modifiers) {
        if (page instanceof GenerationPage gp && gp.keyPressed(keyCode))
            return true;
        return super.keyPressed(keyCode, scanCode, modifiers);
    }

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        super.render(g, mouseX, mouseY, partialTick);
        g.drawCenteredString(font, page == null ? title : page.title(), width / 2, 6, 0xffffffff);
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

    /**
     * Play: accept the finished run, bind the world to it and create the world, as Create World's
     * Create button does (the experimental-settings confirmation included).
     */
    void play() {
        accept();
        CreateWorldFlow.create(parent);
    }

    private static void apply(CreateWorldScreen parent, EditorModel.Accepted a) {
        parent.getUiState().updateDimensions((registries, dims) ->
                PlanetGenerators.withStaging(registries, dims, a.stagingId(), a.worldCircumference(), a.projection()));
    }

    /** The page's id, for the dev screenshot driver. */
    Optional<Page> page() {
        return Optional.ofNullable(page);
    }
}
