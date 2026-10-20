// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.EditorModel;
import com.minecraftindustrialcomplex.deeptime.screen.model.Formats;
import com.minecraftindustrialcomplex.deeptime.screen.model.GenerationStatus;
import com.minecraftindustrialcomplex.deeptime.screen.model.Legend;
import com.minecraftindustrialcomplex.deeptime.screen.model.Milestones;
import com.minecraftindustrialcomplex.deeptime.screen.model.Phrase;
import com.minecraftindustrialcomplex.deeptime.screen.model.Problem;
import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;
import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.ChatFormatting;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.OptionalDouble;

/**
 * The generation page (docs/design/create-world-ux.md; plan §7.12 page 2): what Create World's
 * Create button opens for a Deep Time world. One status sentence, a progress bar and the time left
 * at the top; the planet's map (or globe) as the hero, with the time scrubber, play/pause and the
 * optional "stop here" or "pick this epoch" under it; a panel with the planet's History (the event
 * log grouped and counted in plain words, coloured like the map's markers), the Map tab (layer
 * groups, layers, the globe and marker switches and the shown layer's legend) and More (keep going
 * past today, the final map and spawn, the developer's details); and one bar of actions at the
 * bottom with Play, the primary one, on the right. On a tall screen the map spans the width and the
 * panel sits below it, History and Map side by side.
 *
 * <p>Esc returns to Create World and the run keeps going; Create opens this page again.
 * F3 shows the developer's progress line (stage percentages, native memory) under the bar.
 */
final class GenerationPage implements PlanetEditorScreen.Page {

    enum Tab { HISTORY, MAP, MORE }

    /** Remembered across rebuilds and runs. */
    private static Tab tab = Tab.HISTORY;
    private static boolean globe;
    private static boolean markersOn = true;
    private static boolean details;
    /** The Keep going box: million years more. */
    private static String moreText = "100";

    /** The dev screenshot driver's switches. */
    static void setGlobe(boolean g) {
        globe = g;
    }

    static void setTab(Tab t) {
        tab = t;
    }

    static void setDetails(boolean d) {
        details = d;
    }

    private static final int GAP = 8, TOP = 44, ROW_H = 20, TAB_H = 16;
    private static final int TEXT = 0xffe8e8e8, DIM = 0xff9aa3ad, WARN = 0xffffc060, ERROR = 0xffff7070;

    private PlanetEditorScreen screen;
    private EditorModel em;
    /** The run shown (null while preparing or blocked). */
    private TimelapseModel model;
    private final KeyframeTexture mapTexture = new KeyframeTexture("timelapse");
    private long uploaded = -1;
    private GlobeWidget globeWidget;
    private Scrubber scrubber;
    private Button playPause, epoch, primary, keepGoing;
    private EditBox moreBox;
    private PanelList history, main;
    private final List<Button> tabButtons = new ArrayList<>();
    private final List<Tab> tabsShown = new ArrayList<>();
    /** The map view's box, the area it sits in, and the panel's. */
    private int viewX, viewY, viewW, viewH, panelX, panelY, panelW, panelH, bottom;
    private boolean wide;
    private GenerationStatus.State builtState;
    private Object historyKey, mainKey;
    private long lastTick = System.currentTimeMillis();

    @Override
    public Component title() {
        return switch (GenerationStatus.state(em)) {
            case DONE -> Lang.tr("gen.title.done");
            case CANCELLED -> Lang.tr("gen.title.cancelled");
            case FAILED, BLOCKED -> Lang.tr("gen.title.problem");
            default -> Lang.tr("gen.title");
        };
    }

    @Override
    public void init(PlanetEditorScreen s) {
        screen = s;
        em = s.model();
        model = em.run().orElse(null);
        builtState = GenerationStatus.state(em);
        int w = s.width, h = s.height;
        bottom = h - 32;
        layout(w);

        // The view.
        if (globe) {
            int size = Math.min(viewW, viewH);
            int gx = viewX + (viewW - size) / 2, gy = viewY + (viewH - size) / 2;
            globeWidget = s.add(new GlobeWidget(gx, gy, size, this::currentPixels, this::currentSize, this::markers));
        } else {
            s.add(new MapWidget(viewX, viewY, viewW, viewH, mapTexture, this::markers, List::of));
        }

        // The time row under the view: play/pause, the scrubber, stop here / pick this epoch.
        int sy = viewY + viewH + 4;
        int epochW = Math.min(110, Math.max(80, viewW / 4));
        playPause = s.add(Button.builder(Component.literal("▶"), b -> {
            if (model != null)
                model.togglePlay();
        }).bounds(viewX, sy, 20, ROW_H).build());
        scrubber = s.add(new Scrubber(viewX + 24, sy, viewW - 24 - 4 - epochW, ROW_H));
        epoch = s.add(Button.builder(Lang.tr(done() ? "gen.epoch" : "gen.stop_here"), b -> pickEpoch())
                .bounds(viewX + viewW - epochW, sy, epochW, ROW_H).build());

        // The panel: tabs, then the list (or More's widgets).
        buildPanel();

        // The action bar.
        int by = h - 26;
        boolean going = builtState == GenerationStatus.State.PREPARING || builtState == GenerationStatus.State.STARTING
                || builtState == GenerationStatus.State.RUNNING || builtState == GenerationStatus.State.BAKING;
        int bw = Math.min(96, (w - 4 * GAP) / 3);
        Button left = s.add(Button.builder(Lang.tr(going ? "gen.cancel" : "gen.back"), b -> {
            if (going)
                em.cancelRun();
            s.onClose();
        }).bounds(GAP, by, bw, ROW_H).build());
        left.setTooltip(Tooltip.create(Lang.tr(going ? "gen.cancel.hint" : "gen.back.hint")));
        Button customize = s.add(Button.builder(Lang.tr("gen.customize"), b -> {
            em.openCustomize(true);
            s.refresh();
        }).bounds(GAP + bw + 4, by, bw, ROW_H).build());
        customize.setTooltip(Tooltip.create(Lang.tr("gen.customize.hint")));
        boolean retry = builtState == GenerationStatus.State.BLOCKED || builtState == GenerationStatus.State.CANCELLED
                || builtState == GenerationStatus.State.FAILED;
        int pw = Math.min(120, bw + 24);
        primary = s.add(Button.builder(retry ? Lang.tr("gen.retry") : Lang.tr("gen.play").withStyle(ChatFormatting.BOLD), b -> {
            if (retry) {
                em.generate();
                s.refresh();
            } else if (em.canAccept()) {
                s.play();
            }
        }).bounds(w - GAP - pw, by, pw, ROW_H).build());
        uploaded = -1;
        tick();
    }

    /**
     * Places the view and the panel. Side by side: the panel on the right and the map left of it.
     * Stacked, when the map gets much larger that way (a tall screen such as 2000×1500 at GUI
     * scale 3): the map across the width, the panel below it.
     */
    private void layout(int w) {
        int sideW = Math.max(140, Math.min(210, w * 32 / 100));
        int sideAreaW = w - 2 * GAP - sideW - GAP;
        int sideAreaH = bottom - TOP - ROW_H - 4 - 4;
        int[] side = MapWidget.fit(sideAreaW, sideAreaH);
        int fullW = w - 2 * GAP;
        int minPanel = 8 * PanelList.ROW + TAB_H + 8;
        int stackAreaH = bottom - TOP - ROW_H - 4 - 6 - minPanel;
        int[] stack = MapWidget.fit(fullW, Math.max(20, stackAreaH));
        boolean stacked = stack[0] > side[0] * 5 / 4;
        if (stacked) {
            viewW = stack[0];
            viewH = stack[1];
            if (globe)
                viewW = fullW;
            viewX = GAP + (fullW - viewW) / 2;
            viewY = TOP;
            panelX = GAP;
            panelY = viewY + viewH + 4 + ROW_H + 6;
            panelW = fullW;
            panelH = bottom - 4 - panelY;
        } else {
            viewW = side[0];
            viewH = side[1];
            if (globe) {
                viewW = sideAreaW;
                viewH = sideAreaH;
            }
            viewX = GAP + (sideAreaW - viewW) / 2;
            // The view and its time row as one block, centred in the height.
            viewY = TOP + Math.max(0, (sideAreaH - viewH) / 2);
            panelX = w - GAP - sideW;
            panelY = TOP;
            panelW = sideW;
            panelH = bottom - 4 - TOP;
        }
        wide = panelW >= 400;
    }

    // ------------------------------------------------------------------------------------------
    // The panel
    // ------------------------------------------------------------------------------------------

    private void buildPanel() {
        tabButtons.clear();
        tabsShown.clear();
        int listX = panelX, listW = panelW;
        if (wide) {
            // History has the left half to itself; the tabs switch the right half.
            int hw = (panelW - GAP) / 2;
            history = screen.add(new PanelList(Minecraft.getInstance(), panelX, panelY + TAB_H + 2, hw, panelH - TAB_H - 2));
            listX = panelX + hw + GAP;
            listW = panelW - hw - GAP;
            tabsShown.addAll(List.of(Tab.MAP, Tab.MORE));
            if (tab == Tab.HISTORY)
                tab = Tab.MAP;
        } else {
            history = null;
            tabsShown.addAll(List.of(Tab.HISTORY, Tab.MAP, Tab.MORE));
        }
        int tw = (listW - 2 * (tabsShown.size() - 1)) / tabsShown.size();
        for (int i = 0; i < tabsShown.size(); i++) {
            Tab t = tabsShown.get(i);
            Button b = screen.add(Button.builder(Lang.tr("gen.tab." + t.name().toLowerCase(java.util.Locale.ROOT)), x -> {
                tab = t;
                screen.refresh();
            }).bounds(listX + i * (tw + 2), panelY, tw, TAB_H).build());
            b.active = t != tab;
            tabButtons.add(b);
        }
        historyKey = null;
        mainKey = null;
        main = null;
        moreBox = null;
        keepGoing = null;
        int y = panelY + TAB_H + 2;
        int h = panelH - TAB_H - 2;
        if (tab == Tab.MORE) {
            buildMore(listX, y, listW, h);
        } else {
            main = screen.add(new PanelList(Minecraft.getInstance(), listX, y, listW, h));
        }
        refreshLists();
    }

    /** More: keep going past today, the final map and spawn, the developer's details. */
    private void buildMore(int x, int y, int w, int h) {
        int yy = y + 4;
        Font font = screen.font();
        yy += 10 + font.split(Lang.tr("gen.more.keep_going.hint"), w - 4).size() * 10 + 2;
        int boxW = Math.max(40, Math.min(60, w / 3));
        moreBox = new EditBox(font, x + 2, yy, boxW, ROW_H, Lang.tr("gen.more.keep_going.amount"));
        moreBox.setValue(moreText);
        moreBox.setResponder(t -> moreText = t);
        moreBox.setTooltip(Tooltip.create(Lang.tr("gen.more.keep_going.amount")));
        screen.add(moreBox);
        keepGoing = screen.add(Button.builder(Lang.tr("gen.more.keep_going"), b -> keepGoing())
                .bounds(x + 2 + boxW + 4, yy, w - boxW - 8, ROW_H).build());
        yy += ROW_H + 8;
        Button review = screen.add(Button.builder(Lang.tr("gen.more.review"), b -> {
            em.openReview();
            screen.refresh();
        }).bounds(x + 2, yy, w - 4, ROW_H).build());
        review.active = em.canAccept();
        review.setTooltip(Tooltip.create(Lang.tr("gen.more.review.hint")));
        yy += ROW_H + 4;
        screen.add(Button.builder(Lang.tr(details ? "gen.more.details.hide" : "gen.more.details.show"), b -> {
            details = !details;
            screen.refresh();
        }).bounds(x + 2, yy, w - 4, ROW_H).build()).setTooltip(Tooltip.create(Lang.tr("gen.more.details.hint")));
    }

    private void keepGoing() {
        OptionalDouble end = em.runStopMyr();
        double more = Formats.parse(moreText);
        if (end.isEmpty() || !(more > 0))
            return;
        String to = Formats.number(end.getAsDouble() + more);
        if (em.canContinue() && em.continueProblem(to).isEmpty()) {
            em.continueTo(Formats.parse(to));
            screen.refresh();
        }
    }

    /** Rebuilds the lists' rows when what they show changed. */
    private void refreshLists() {
        PanelList hl = wide ? history : tab == Tab.HISTORY ? main : null;
        if (hl != null) {
            Object key = model == null ? "none" : List.of(model.events().size(), model.cursorTime(), model.index(), model.frameCount());
            if (!key.equals(historyKey)) {
                historyKey = key;
                hl.set(historyRows(hl));
            }
        }
        if (tab == Tab.MAP && main != null) {
            Object key = model == null ? "none" : List.of(model.layers().size(), model.group(), model.layer().orElse(""));
            if (!key.equals(mainKey)) {
                mainKey = key;
                main.set(mapRows(main));
            }
        }
    }

    private List<PanelList.Row> historyRows(PanelList list) {
        List<PanelList.Row> rows = new ArrayList<>();
        if (wide)
            rows.add(PanelList.heading(Lang.tr("gen.tab.history")));
        if (model == null || model.events().isEmpty()) {
            rows.addAll(PanelList.wrapped(Lang.tr("gen.history.empty"), list.getRowWidth(), DIM));
            return rows;
        }
        Font font = screen.font();
        int timeW = font.width("0,000");
        rows.add(PanelList.text(Lang.tr("gen.history.column"), 0xff7d8590));
        for (Milestones.Row r : Milestones.of(model.events(), model.cursorTime(), Milestones.bucketYears(model.lengthYears())))
            rows.add(PanelList.milestone(Phrase.myrAgo(r.tYears()), timeW, r.colour(), Lang.milestone(r), model.freshAt(r.tYears())));
        return rows;
    }

    private List<PanelList.Row> mapRows(PanelList list) {
        List<PanelList.Row> rows = new ArrayList<>();
        if (model == null || model.layers().isEmpty()) {
            rows.addAll(PanelList.wrapped(Lang.tr("gen.map.empty"), list.getRowWidth(), DIM));
            return rows;
        }
        List<String> groups = model.groups();
        rows.add(PanelList.stepper(() -> Component.empty().append(Lang.layerGroup(model.group()))
                        .append(groups.size() > 1 ? " (" + (groups.indexOf(model.group()) + 1) + "/" + groups.size() + ")" : ""),
                () -> model.cycleGroup(-1), () -> model.cycleGroup(1)));
        for (String name : model.groupLayers())
            rows.add(PanelList.choice(Lang.layer(name), () -> model.layer().map(name::equals).orElse(false),
                    () -> !model.currentHas(name), () -> model.currentHas(name) ? null : Lang.tr("run.layer_missing"),
                    () -> model.setLayer(name)));
        rows.add(PanelList.gap());
        rows.add(PanelList.toggle(Lang.tr("gen.map.globe"), () -> globe, () -> {
            globe = !globe;
            screen.refresh();
        }));
        rows.add(PanelList.toggle(Lang.tr("gen.map.markers"), () -> markersOn, () -> markersOn = !markersOn));
        model.layer().ifPresent(layer -> {
            rows.add(PanelList.gap());
            rows.add(PanelList.heading(Lang.tr("gen.map.legend", Lang.layer(layer))));
            for (Legend.Part p : Legend.of(Ramps.builtin(), layer)) {
                switch (p) {
                    case Legend.Heading hd -> rows.add(PanelList.text(Lang.phrase(hd.text()).withStyle(ChatFormatting.BOLD), DIM));
                    case Legend.Swatch sw -> rows.add(PanelList.swatch(sw.rgb(), Lang.phrase(sw.text())));
                    case Legend.Bar bar -> rows.add(PanelList.bar(bar.rgb(), Lang.phrase(bar.low()), Lang.phrase(bar.high())));
                    case Legend.Note n -> rows.addAll(PanelList.wrapped(Lang.phrase(n.text()), list.getRowWidth(), DIM));
                }
            }
            rows.add(PanelList.gap());
            rows.addAll(PanelList.wrapped(Lang.tr("gen.map.markers.legend"), list.getRowWidth(), DIM));
        });
        return rows;
    }

    // ------------------------------------------------------------------------------------------
    // Ticking
    // ------------------------------------------------------------------------------------------

    private boolean done() {
        return model != null && model.state() == TimelapseModel.State.DONE;
    }

    private void pickEpoch() {
        if (model == null)
            return;
        model.current().ifPresent(f -> {
            if (em.epochStopMyr(f).isPresent()) {
                em.pickEpoch(f);
                screen.refresh();
            }
        });
    }

    @Override
    public void tick() {
        if (em.run().orElse(null) != model || GenerationStatus.state(em) != builtState) {
            screen.refresh(); // a run started, was picked up again, or changed state: new buttons
            return;
        }
        long now = System.currentTimeMillis();
        if (model != null)
            model.tick(now - lastTick);
        lastTick = now;
        if (model != null) {
            long v = model.displayVersion();
            if (v != uploaded) {
                model.displayed().ifPresent(f -> mapTexture.upload(f.abgr(), f.info().w(), f.info().h()));
                uploaded = v;
            }
        }
        if (globeWidget != null)
            globeWidget.tick();
        refreshLists();
        boolean running = model != null && model.state() == TimelapseModel.State.RUNNING;
        playPause.setMessage(Component.literal(model != null && model.playing() ? "❚❚" : "▶"));
        playPause.active = model != null && model.frameCount() > 1;
        playPause.setTooltip(Tooltip.create(Lang.tr("gen.play_back.hint")));
        Optional<KeyframeInfo> f = model == null ? Optional.empty() : model.current();
        OptionalDouble stop = f.map(em::epochStopMyr).orElse(OptionalDouble.empty());
        epoch.active = stop.isPresent();
        epoch.setTooltip(Tooltip.create(stop.isPresent()
                ? Lang.tr(running ? "gen.stop_here.hint" : "gen.epoch.hint", Lang.phrase(Phrase.yearsAgo(f.get().t())))
                : Lang.tr(running ? "gen.stop_here.none" : "gen.epoch.none")));
        if (keepGoing != null) {
            OptionalDouble end = em.runStopMyr();
            double more = Formats.parse(moreText);
            keepGoing.active = em.canContinue() && end.isPresent() && more > 0
                    && em.continueProblem(Formats.number(end.getAsDouble() + more)).isEmpty();
            keepGoing.setTooltip(Tooltip.create(Lang.tr("gen.more.keep_going.button_hint")));
        }
        boolean retry = builtState == GenerationStatus.State.BLOCKED || builtState == GenerationStatus.State.CANCELLED
                || builtState == GenerationStatus.State.FAILED;
        if (!retry) {
            primary.active = em.canAccept();
            primary.setTooltip(Tooltip.create(Lang.tr(primary.active ? "gen.play.hint" : "gen.play.waiting")));
        }
        scrubber.sync();
    }

    // ------------------------------------------------------------------------------------------
    // Drawing
    // ------------------------------------------------------------------------------------------

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        Font font = screen.font();
        int w = screen.width;
        GenerationStatus.State state = GenerationStatus.state(em);

        // Status: the sentence on the left, the percentage and time left on the right, the bar under them.
        Component right = rightOfStatus(state);
        int rw = right == null ? 0 : font.width(right);
        Component sentence = Lang.phrase(GenerationStatus.sentence(em));
        int sentenceColour = state == GenerationStatus.State.FAILED || state == GenerationStatus.State.BLOCKED ? ERROR : TEXT;
        drawFitted(g, font, sentence, GAP, 20, w - 2 * GAP - rw - 8, sentenceColour);
        if (right != null)
            g.drawString(font, right, w - GAP - rw, 20, DIM, false);
        int barY = 31;
        g.fill(GAP, barY, w - GAP, barY + 5, 0xff2a2f37);
        int fill = GAP + (int) Math.round((w - 2 * GAP) * GenerationStatus.fraction(em));
        int colour = switch (state) {
            case DONE -> 0xff4cc552;
            case CANCELLED -> 0xff808890;
            case FAILED, BLOCKED -> 0xffe04848;
            case BAKING -> 0xff7fc8ff;
            default -> 0xff3f9fff;
        };
        if (fill > GAP)
            g.fill(GAP, barY, fill, barY + 5, colour);
        if (details && model != null)
            g.drawString(font, font.plainSubstrByWidth(model.progressLine(em.capBytes()), w - 2 * GAP), GAP, barY + 7, 0xff707880, false);
        separator(g, Screen.HEADER_SEPARATOR, details ? barY + 17 : TOP - 4, w);
        separator(g, Screen.FOOTER_SEPARATOR, bottom, w);

        // Over the view: the layer's name, "live" while following the newest frame, or what the view waits for.
        if (model != null && model.frameCount() > 0) {
            Component caption = model.layer().map(Lang::layer).orElse(Component.empty());
            label(g, font, caption, viewX + 3, viewY + 3);
            if (model.following()) {
                Component live = Lang.tr("gen.live");
                label(g, font, live, viewX + viewW - 3 - font.width(live), viewY + 3);
            }
        } else {
            List<Component> lines = new ArrayList<>();
            switch (state) {
                case BLOCKED -> {
                    lines.add(Lang.tr("gen.blocked.why"));
                    blocker().ifPresent(p -> lines.add(Lang.problem(p)));
                    lines.add(Lang.tr("gen.blocked.what_now"));
                }
                case FAILED -> model.failure().ifPresent(msg -> lines.add(Component.literal(msg)));
                case CANCELLED -> lines.add(Lang.tr("gen.cancelled.what_now"));
                default -> {
                    lines.add(Lang.tr("gen.waiting"));
                    lines.add(Lang.tr("gen.waiting.hint"));
                }
            }
            int y = viewY + viewH / 2 - lines.size() * 6;
            for (Component c : lines) {
                for (var seq : font.split(c, viewW - 16)) {
                    g.drawCenteredString(font, seq, viewX + viewW / 2, y, state == GenerationStatus.State.BLOCKED ? WARN : DIM);
                    y += 11;
                }
            }
        }

        // More's words.
        if (tab == Tab.MORE) {
            int x = wide ? panelX + (panelW - GAP) / 2 + GAP : panelX;
            int lw = wide ? panelW - (panelW - GAP) / 2 - GAP : panelW;
            int y = panelY + TAB_H + 6;
            g.drawString(font, Lang.tr("gen.more.keep_going.title").withStyle(ChatFormatting.BOLD), x + 2, y, 0xffffd060, false);
            y += 10;
            for (var seq : font.split(Lang.tr("gen.more.keep_going.hint"), lw - 4)) {
                g.drawString(font, seq, x + 2, y, DIM, false);
                y += 10;
            }
        }
    }

    /** "42% · about 3 min left" while going, "Took 4 min" when done. */
    private Component rightOfStatus(GenerationStatus.State state) {
        if (state == GenerationStatus.State.DONE && model != null)
            return Lang.tr("gen.status.took", Lang.phrase(Phrase.duration(model.progress().elapsedMs() / 1000.0)));
        Optional<Phrase> left = GenerationStatus.timeLeft(em);
        if (state == GenerationStatus.State.RUNNING || state == GenerationStatus.State.BAKING) {
            String pct = Formats.percent(model.progress().totalPermille());
            return left.<Component>map(p -> Lang.tr("gen.status.progress", pct, Lang.phrase(p))).orElse(Component.literal(pct));
        }
        return left.<Component>map(Lang::phrase).orElse(null);
    }

    /** The first problem that keeps the run from starting, else the failed launch. */
    private Optional<Problem> blocker() {
        List<Problem> ps = em.problems();
        for (Problem p : ps)
            if (p.blocking())
                return Optional.of(p);
        for (Problem p : ps)
            if (p.key().equals("editor.warning.launch_failed"))
                return Optional.of(p);
        return Optional.empty();
    }

    private static void drawFitted(GuiGraphics g, Font font, Component text, int x, int y, int w, int colour) {
        if (font.width(text) <= w) {
            g.drawString(font, text, x, y, colour, true);
            return;
        }
        var cut = net.minecraft.locale.Language.getInstance().getVisualOrder(font.substrByWidth(text, Math.max(0, w - font.width("…"))));
        int end = g.drawString(font, cut, x, y, colour, true);
        g.drawString(font, "…", end, y, colour, true);
    }

    /** Text on a dark backing over the map. */
    private static void label(GuiGraphics g, Font font, Component text, int x, int y) {
        g.fill(x - 2, y - 2, x + font.width(text) + 2, y + 9, 0xa0000000);
        g.drawString(font, text, x, y, 0xffffffff, false);
    }

    private static void separator(GuiGraphics g, net.minecraft.resources.ResourceLocation tex, int y, int w) {
        RenderSystem.enableBlend();
        g.blit(tex, 0, y, 0.0F, 0.0F, w, 2, 32, 2);
        RenderSystem.disableBlend();
    }

    private int[] currentPixels() {
        return model == null ? null : model.displayed().map(TimelapseModel.Frame::abgr).orElse(null);
    }

    private int[] currentSize() {
        return model == null ? null : model.displayed().map(f -> new int[]{f.info().w(), f.info().h()}).orElse(null);
    }

    private List<MapWidget.Marker> markers() {
        if (!markersOn || model == null)
            return List.of();
        return MapWidget.eventMarkers(model.markers(), model::fresh);
    }

    /** F3 shows or hides the developer's line. */
    boolean keyPressed(int key) {
        if (key == org.lwjgl.glfw.GLFW.GLFW_KEY_F3) {
            details = !details;
            return true;
        }
        return false;
    }

    @Override
    public void close() {
        mapTexture.close();
        if (globeWidget != null)
            globeWidget.close();
    }

    /** The time scrubber: one step per keyframe, labelled with the frame's time in years ago. */
    final class Scrubber extends AbstractSliderButton {

        private int shownIndex = -2;

        Scrubber(int x, int y, int w, int h) {
            super(x, y, w, h, Component.empty(), 1.0);
            updateMessage();
        }

        void sync() {
            int n = model == null ? 0 : model.frameCount();
            double v = n <= 1 ? 1.0 : Math.max(0, model.index()) / (double) (n - 1);
            int idx = model == null ? -1 : model.index();
            if (Math.abs(v - value) > 1e-9 || idx != shownIndex) {
                value = v;
                shownIndex = idx;
                updateMessage();
            }
            active = n > 1;
        }

        @Override
        protected void updateMessage() {
            Optional<KeyframeInfo> f = model == null ? Optional.empty() : model.current();
            setMessage(f.<Component>map(k -> Lang.phrase(Phrase.yearsAgo(k.t()))).orElse(Lang.tr("gen.scrub")));
            setTooltip(Tooltip.create(Lang.tr("gen.scrub.hint")));
        }

        @Override
        protected void applyValue() {
            int n = model == null ? 0 : model.frameCount();
            if (n > 0)
                model.scrubTo((int) Math.round(value * (n - 1)));
            updateMessage();
        }
    }
}
