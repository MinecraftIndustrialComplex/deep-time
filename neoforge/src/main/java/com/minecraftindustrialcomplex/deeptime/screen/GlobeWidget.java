// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.GlobeProjection;
import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.network.chat.Component;

import java.util.List;
import java.util.function.Supplier;

/**
 * The globe view (plan §7.12 "map/globe toggle"): the current map frame (Lambert equal-area) projected
 * orthographically on the CPU ({@link GlobeProjection}) into a square texture, re-rendered when the
 * frame or the rotation changes. It turns slowly on its own; dragging turns and tilts it.
 */
class GlobeWidget extends AbstractWidget {

    static final int SIZE = 256;
    /** Degrees per second of the idle rotation. */
    static final double SPIN = 12.0;

    private final KeyframeTexture texture = new KeyframeTexture("globe");
    private final Supplier<int[]> source;
    private final Supplier<int[]> sourceSize;
    private final Supplier<List<MapWidget.Marker>> markers;
    private final int[] out = new int[SIZE * SIZE];
    private double lon = 0, lat = 20;
    private boolean spinning = true;
    private int[] lastSource;
    private double lastLon = Double.NaN, lastLat = Double.NaN;

    /**
     * @param source     the frame's ABGR pixels (null before the first frame)
     * @param sourceSize {w, h} of {@code source}
     */
    GlobeWidget(int x, int y, int size, Supplier<int[]> source, Supplier<int[]> sourceSize, Supplier<List<MapWidget.Marker>> markers) {
        super(x, y, size, size, Component.empty());
        this.source = source;
        this.sourceSize = sourceSize;
        this.markers = markers;
    }

    void tick() {
        if (spinning)
            lon = (lon + SPIN / 20.0) % 360.0;
    }

    @Override
    protected void renderWidget(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        int[] src = source.get();
        int[] wh = sourceSize.get();
        if (src != null && wh != null && (src != lastSource || lon != lastLon || lat != lastLat)) {
            new GlobeProjection(lat, lon).render(src, wh[0], wh[1], out, SIZE);
            texture.upload(out, SIZE, SIZE);
            lastSource = src;
            lastLon = lon;
            lastLat = lat;
        }
        int x = getX(), y = getY(), s = getWidth();
        if (texture.ready()) {
            RenderSystem.enableBlend();
            g.blit(texture.id(), x, y, s, s, 0, 0, SIZE, SIZE, SIZE, SIZE);
            RenderSystem.disableBlend();
        }
        GlobeProjection proj = new GlobeProjection(lat, lon);
        double[] uv = new double[2];
        double half = s / 2.0;
        for (MapWidget.Marker m : markers.get()) {
            if (proj.forward(m.lat(), m.lon(), uv) == null)
                continue;
            int cx = x + (int) (half + uv[0] * half), cy = y + (int) (half - uv[1] * half);
            int r = m.fresh() ? 2 : 1;
            g.fill(cx - r - 1, cy - r - 1, cx + r + 2, cy + r + 2, 0xff000000);
            g.fill(cx - r, cy - r, cx + r + 1, cy + r + 1, m.colour());
        }
    }

    @Override
    protected void onDrag(double mouseX, double mouseY, double dragX, double dragY) {
        spinning = false;
        lon = ((lon - dragX * 180.0 / getWidth()) % 360.0 + 360.0) % 360.0;
        lat = Math.max(-89, Math.min(89, lat + dragY * 180.0 / getHeight()));
    }

    @Override
    public void onClick(double mouseX, double mouseY) {
        // a click stops the idle spin; a double action (click again) restarts it
        spinning = !spinning;
    }

    void close() {
        texture.close();
    }

    @Override
    protected void updateWidgetNarration(NarrationElementOutput out) {
        defaultButtonNarrationText(out);
    }

    @Override
    public void playDownSound(net.minecraft.client.sounds.SoundManager handler) {
        // not a button
    }
}
