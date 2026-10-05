// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.Formats;
import com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.network.chat.Component;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * The run page, the timelapse (plan §7.12 page 2): progress, a map or globe of the current
 * keyframe with event markers, layer toggles, the event ticker, a scrubber with play/pause, and
 * Cancel while running or Review once done. All state is the {@link TimelapseModel}'s; this page
 * uploads the displayed frame into a {@link KeyframeTexture} when it changes.
 */
final class RunPage implements PlanetEditorScreen.Page {

    /** Map or globe, remembered across rebuilds. */
    private static boolean globe;
    private static boolean markersOn = true;
    /** The page of the shown group's layer list (when it has more layers than fit). */
    private static int layerPage;

    /** The smallest event ticker the side panel keeps under the layer list. */
    private static final int TICKER_MIN = 64;
    private static final int ROW = 18;

    /** The screenshot driver's switch (dev only). */
    static void setGlobe(boolean g) {
        globe = g;
    }

    private PlanetEditorScreen screen;
    private TimelapseModel model;
    private final KeyframeTexture mapTexture = new KeyframeTexture("timelapse");
    private long uploaded = -1;
    private GlobeWidget globeWidget;
    private Scrubber scrubber;
    private Button play, cancel, review, epoch, continueB;
    private EditBox continueBox;
    /** Whether the page was laid out for a finished run. */
    private boolean builtDone;
    /** The continue box's text, kept across rebuilds of the same run's page. */
    private static String continueText;
    private static String continueFor = "";
    /** What the continue box suggests past the current stop, Myr. */
    private static final double CONTINUE_STEP_MYR = 100.0;
    private final List<Button> layerButtons = new ArrayList<>();
    private final List<String> layerNames = new ArrayList<>();
    private List<String> layersShown = List.of();
    private String groupShown = "";
    private Button groupButton;
    private int viewX, viewY, viewW, viewH, sideX, sideW;
    private long lastTick = System.currentTimeMillis();

    @Override
    public void init(PlanetEditorScreen s) {
        screen = s;
        model = s.model().run().orElseThrow();
        int top = 34, bottom = s.height - 52;
        sideW = s.width >= 560 ? 170 : s.width >= 420 ? 150 : 110;
        sideX = s.width - sideW - 6;
        int availW = sideX - 12, availH = bottom - top;
        if (globe) {
            int size = Math.min(availW, availH);
            viewX = 6 + (availW - size) / 2;
            viewY = top;
            viewW = viewH = size;
            globeWidget = s.add(new GlobeWidget(viewX, viewY, size, this::currentPixels, this::currentSize, this::markers));
        } else {
            int[] fit = MapWidget.fit(availW, availH);
            viewX = 6 + (availW - fit[0]) / 2;
            viewY = top + (availH - fit[1]) / 2;
            viewW = fit[0];
            viewH = fit[1];
            s.add(new MapWidget(viewX, viewY, viewW, viewH, mapTexture, this::markers, List::of));
        }
        int tickerY = buildLayerButtons(top, bottom);
        s.add(new EventTicker(sideX, tickerY, sideW, bottom - tickerY, model));

        int sy = s.height - 48;
        play = s.add(Button.builder(Component.literal("▶"), b -> model.togglePlay()).bounds(6, sy, 20, 20).build());
        // Pick the epoch / stop here, and continue (docs/design/playability-steering.md): at the
        // scrubber's right end.
        boolean done = model.state() == TimelapseModel.State.DONE;
        builtDone = done;
        boolean roomForContinue = done && s.width >= 420;
        int epochW = 96, boxW = 48, contW = 72;
        int right = 6 + epochW + (roomForContinue ? 4 + boxW + 4 + contW : 0);
        int rx = s.width - right;
        scrubber = s.add(new Scrubber(30, sy, rx - 34, 20));
        epoch = s.add(Button.builder(Lang.tr(done ? "run.epoch" : "run.stop_here"), b -> {
            model.current().ifPresent(f -> {
                if (screen.model().epochStopMyr(f).isPresent()) {
                    screen.model().pickEpoch(f);
                    screen.refresh();
                }
            });
        }).bounds(rx, sy, epochW, 20).build());
        continueBox = null;
        continueB = null;
        if (roomForContinue) {
            EditBox box = new EditBox(screen.font(), rx + epochW + 4, sy, boxW, 20, Lang.tr("run.continue.to"));
            if (continueText == null || !continueFor.equals(model.stagingId()))
                continueText = screen.model().runStopMyr().stream().mapToObj(v -> Formats.number(v + CONTINUE_STEP_MYR))
                        .findFirst().orElse("");
            continueFor = model.stagingId();
            box.setValue(continueText);
            box.setResponder(t -> continueText = t);
            box.setTooltip(net.minecraft.client.gui.components.Tooltip.create(Lang.tr("run.continue.hint")));
            continueBox = s.add(box);
            continueB = s.add(Button.builder(Lang.tr("run.continue"), b -> {
                if (screen.model().continueProblem(continueText).isEmpty()) {
                    screen.model().continueTo(Formats.parse(continueText));
                    screen.refresh();
                }
            }).bounds(rx + epochW + 4 + boxW + 4, sy, contW, 20).build());
        }

        int by = s.height - 24;
        int bw = Math.min(90, (s.width - 40) / 5);
        int x = s.width / 2 - (5 * bw + 16) / 2;
        s.add(Button.builder(globe ? Lang.tr("run.view.map") : Lang.tr("run.view.globe"), b -> {
            globe = !globe;
            s.refresh();
        }).bounds(x, by, bw, 20).build());
        s.add(Button.builder(markersOn ? Lang.tr("run.markers.on") : Lang.tr("run.markers.off"), b -> {
            markersOn = !markersOn;
            b.setMessage(markersOn ? Lang.tr("run.markers.on") : Lang.tr("run.markers.off"));
        }).bounds(x + bw + 4, by, bw, 20).build());
        cancel = s.add(Button.builder(Lang.tr("run.cancel"), b -> {
            if (model.state() == TimelapseModel.State.RUNNING)
                s.model().cancelRun();
            else
                s.model().discardRun();
            s.refresh();
        }).bounds(x + 2 * (bw + 4), by, bw, 20).build());
        review = s.add(Button.builder(Lang.tr("run.review"), b -> {
            s.model().openReview();
            s.refresh();
        }).bounds(x + 3 * (bw + 4), by, bw, 20).build());
        s.add(Button.builder(Lang.tr("run.back"), b -> {
            s.model().showSetup();
            s.refresh();
        }).bounds(x + 4 * (bw + 4), by, bw, 20).build());
        uploaded = -1;
        tick();
    }

    /**
     * The side panel's layer list: a group button (Planet, Climate, …; a click shows the next group,
     * as the workbench groups its layers) and the group's layers in two columns, paged when they do
     * not fit above the smallest ticker. Returns the y where the ticker starts.
     */
    private int buildLayerButtons(int top, int bottom) {
        layerButtons.clear();
        layerNames.clear();
        layersShown = List.copyOf(model.layers());
        groupShown = model.group();
        if (layersShown.isEmpty())
            return top;
        List<String> groups = model.groups();
        groupButton = screen.add(Button.builder(Component.empty().append(Lang.layerGroup(groupShown))
                        .append(" (" + (groups.indexOf(groupShown) + 1) + "/" + groups.size() + ") ▸"), b -> {
                    model.cycleGroup(1);
                    layerPage = 0;
                    screen.refresh();
                }).bounds(sideX, top, sideW, ROW).build());
        List<String> names = model.groupLayers();
        int rows = Math.max(1, Math.min((names.size() + 1) / 2, (bottom - top - ROW - 2 - TICKER_MIN) / (ROW + 2)));
        int slots = rows * 2;
        boolean paged = names.size() > slots;
        int perPage = paged ? slots - 1 : slots;
        int pages = Math.max(1, (names.size() + perPage - 1) / perPage);
        layerPage = Math.floorMod(layerPage, pages);
        int bw = (sideW - 4) / 2;
        int y0 = top + ROW + 2;
        int slot = 0;
        for (int i = layerPage * perPage; i < names.size() && slot < perPage; i++, slot++) {
            String name = names.get(i);
            Button b = screen.add(Button.builder(Lang.layer(name), btn -> model.setLayer(name))
                    .bounds(sideX + (slot % 2) * (bw + 4), y0 + (slot / 2) * (ROW + 2), bw, ROW).build());
            layerButtons.add(b);
            layerNames.add(name);
        }
        if (paged) {
            int p = layerPage;
            screen.add(Button.builder(Component.literal("▸ " + (p + 1) + "/" + pages), b -> {
                layerPage = p + 1;
                screen.refresh();
            }).bounds(sideX + (slot % 2) * (bw + 4), y0 + (slot / 2) * (ROW + 2), bw, ROW).build());
        }
        return y0 + rows * (ROW + 2) + 2;
    }

    private int[] currentPixels() {
        return model.displayed().map(TimelapseModel.Frame::abgr).orElse(null);
    }

    private int[] currentSize() {
        return model.displayed().map(f -> new int[]{f.info().w(), f.info().h()}).orElse(null);
    }

    private List<MapWidget.Marker> markers() {
        if (!markersOn)
            return List.of();
        return MapWidget.eventMarkers(model.markers(), model::fresh);
    }

    @Override
    public void tick() {
        long now = System.currentTimeMillis();
        model.tick(now - lastTick);
        lastTick = now;
        if (!model.layers().equals(layersShown) || !model.group().equals(groupShown)) {
            screen.refresh(); // new layers arrived (or the shown layer's group changed): lay the side panel out again
            return;
        }
        long v = model.displayVersion();
        if (v != uploaded) {
            model.displayed().ifPresent(f -> mapTexture.upload(f.abgr(), f.info().w(), f.info().h()));
            uploaded = v;
        }
        if (globeWidget != null)
            globeWidget.tick();
        boolean running = model.state() == TimelapseModel.State.RUNNING;
        cancel.setMessage(running ? Lang.tr("run.cancel") : Lang.tr("run.discard"));
        review.active = screen.model().canAccept();
        if (!builtDone && model.state() == TimelapseModel.State.DONE) {
            screen.refresh(); // the run finished: Play this epoch and Continue replace Stop here
            return;
        }
        java.util.OptionalDouble stop = model.current().map(f -> screen.model().epochStopMyr(f))
                .orElse(java.util.OptionalDouble.empty());
        epoch.active = stop.isPresent();
        epoch.setTooltip(net.minecraft.client.gui.components.Tooltip.create(stop.isPresent()
                ? Lang.tr(running ? "run.stop_here.hint" : "run.epoch.hint", Formats.number(stop.getAsDouble()))
                : Lang.tr("run.epoch.none")));
        if (continueB != null)
            continueB.active = screen.model().continueProblem(continueText == null ? "" : continueText).isEmpty();
        play.setMessage(Component.literal(model.playing() ? "❚❚" : "▶"));
        play.active = model.frameCount() > 1;
        String current = model.layer().orElse("");
        for (int i = 0; i < layerButtons.size(); i++) {
            String name = layerNames.get(i);
            Button b = layerButtons.get(i);
            b.active = !name.equals(current);
            // A layer the displayed frame does not carry is greyed; a click picks the nearest frame that has it.
            boolean has = model.currentHas(name);
            b.setMessage(has ? Lang.layer(name) : Lang.layer(name).copy().withStyle(net.minecraft.ChatFormatting.DARK_GRAY));
            b.setTooltip(has ? null : net.minecraft.client.gui.components.Tooltip.create(Lang.tr("run.layer_missing")));
        }
        scrubber.sync();
    }

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        Font font = screen.font();
        int w = screen.width;
        String line = model.progressLine(screen.model().capBytes());
        g.drawString(font, font.plainSubstrByWidth(line, w - 12), 6, 18, 0xffe0e0e0, true);
        int barY = 29;
        g.fill(6, barY, w - 6, barY + 3, 0xff303030);
        int fill = 6 + (int) ((w - 12) * model.fraction());
        int colour = switch (model.state()) {
            case RUNNING -> 0xff3fa0ff;
            case DONE -> 0xff40c040;
            case CANCELLED -> 0xff909090;
            case FAILED -> 0xffe04040;
        };
        g.fill(6, barY, fill, barY + 3, colour);
        if (model.frameCount() == 0) {
            Component wait = model.state() == TimelapseModel.State.RUNNING ? Lang.tr("run.waiting") : Lang.tr("run.no_frames");
            g.drawCenteredString(font, wait, viewX + viewW / 2, viewY + viewH / 2 - 4, 0xffa0a0a0);
        }
        Optional<KeyframeInfo> f = model.current();
        if (f.isPresent()) {
            String label = (model.index() + 1) + "/" + model.frameCount() + " · " + Formats.time(f.get().t()) + " · "
                    + model.layer().map(l -> Lang.layer(l).getString()).orElse("") + (model.following() ? " · live" : "");
            g.drawString(font, font.plainSubstrByWidth(label, viewW - 6), viewX + 3, viewY + 3, 0xffffffff, true);
        }
        model.failure().ifPresent(msg -> g.drawCenteredString(font, font.plainSubstrByWidth(msg, w - 20), w / 2, viewY + viewH / 2 + 8, 0xffff7070));
    }

    @Override
    public void close() {
        mapTexture.close();
        if (globeWidget != null)
            globeWidget.close();
    }

    /** The time scrubber: one step per keyframe. */
    final class Scrubber extends AbstractSliderButton {

        Scrubber(int x, int y, int w, int h) {
            super(x, y, w, h, Component.empty(), 1.0);
            updateMessage();
        }

        private int shownIndex = -2;

        /** Moves the knob to the model's frame (live following moves it along) and relabels it. */
        void sync() {
            int n = model.frameCount();
            double v = n <= 1 ? 1.0 : Math.max(0, model.index()) / (double) (n - 1);
            if (Math.abs(v - value) > 1e-9 || model.index() != shownIndex) {
                value = v;
                shownIndex = model.index();
                updateMessage();
            }
            active = n > 1;
        }

        @Override
        protected void updateMessage() {
            Optional<KeyframeInfo> f = model.current();
            setMessage(f.map(k -> (Component) Component.literal(Formats.time(k.t()))).orElse(Lang.tr("run.scrub")));
        }

        @Override
        protected void applyValue() {
            int n = model.frameCount();
            if (n > 0)
                model.scrubTo((int) Math.round(value * (n - 1)));
            updateMessage();
        }
    }
}
