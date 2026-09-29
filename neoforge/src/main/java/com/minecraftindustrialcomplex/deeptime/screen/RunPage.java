// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.Formats;
import com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.Button;
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
    private Button play, cancel, review;
    private final List<Button> layerButtons = new ArrayList<>();
    private List<String> layersShown = List.of();
    private int viewX, viewY, viewW, viewH, sideX, sideW;
    private long lastTick = System.currentTimeMillis();

    @Override
    public void init(PlanetEditorScreen s) {
        screen = s;
        model = s.model().run().orElseThrow();
        int top = 34, bottom = s.height - 52;
        sideW = s.width >= 420 ? 150 : 110;
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
        buildLayerButtons();
        int tickerY = top + 24 * Math.max(1, (layersShown.size() + 1) / 2) + 2;
        s.add(new EventTicker(sideX, tickerY, sideW, bottom - tickerY, model));

        int sy = s.height - 48;
        play = s.add(Button.builder(Component.literal("▶"), b -> model.togglePlay()).bounds(6, sy, 20, 20).build());
        scrubber = s.add(new Scrubber(30, sy, s.width - 36, 20));

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

    private void buildLayerButtons() {
        layerButtons.forEach(b -> b.visible = false);
        layerButtons.clear();
        layersShown = List.copyOf(model.layers());
        int bw = (sideW - 4) / 2;
        for (int i = 0; i < layersShown.size(); i++) {
            String name = layersShown.get(i);
            Button b = screen.add(Button.builder(Lang.layer(name), btn -> model.setLayer(name))
                    .bounds(sideX + (i % 2) * (bw + 4), 34 + (i / 2) * 22, bw, 20).build());
            layerButtons.add(b);
        }
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
        if (!model.layers().equals(layersShown)) {
            screen.refresh(); // new layers arrived: a new page lays the side panel out again
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
        play.setMessage(Component.literal(model.playing() ? "❚❚" : "▶"));
        play.active = model.frameCount() > 1;
        String current = model.layer().orElse("");
        for (int i = 0; i < layerButtons.size(); i++)
            layerButtons.get(i).active = !layersShown.get(i).equals(current);
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
            String label = (model.index() + 1) + "/" + model.frameCount() + " · " + Formats.time(f.get().t()) + " · " + f.get().stage()
                    + (model.following() ? " · live" : "");
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

        /** Moves the knob to the model's frame (live following moves it along). */
        void sync() {
            int n = model.frameCount();
            double v = n <= 1 ? 1.0 : Math.max(0, model.index()) / (double) (n - 1);
            if (Math.abs(v - value) > 1e-9) {
                value = v;
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
