// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.GlobeProjection;
import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.network.chat.Component;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;

/**
 * The map view (Lambert equal-area, π : 1) (plan §7.12): a keyframe (or the review page's final map) stretched
 * into a 2:1 box, with overlays placed by latitude and longitude the way the viewer places them
 * (pixel (px, py) centred on 360°·px/w, 90° − 180°·py/h). Hovering shows the latitude and longitude.
 */
class MapWidget extends AbstractWidget {

    /** A marker: a dot, or (with {@code fresh}) a larger white dot. */
    record Marker(double lat, double lon, int colour, boolean fresh) {}

    /** A tinted latitude–longitude box (province hints). */
    record Box(double lonWest, double lonEast, double latNorth, double latSouth, int argb, String label) {}

    private final KeyframeTexture texture;
    private final Supplier<List<Marker>> markers;
    private final Supplier<List<Box>> boxes;

    MapWidget(int x, int y, int w, int h, KeyframeTexture texture, Supplier<List<Marker>> markers, Supplier<List<Box>> boxes) {
        super(x, y, w, h, Component.empty());
        this.texture = texture;
        this.markers = markers;
        this.boxes = boxes;
    }

    /** The largest box of the map picture's aspect ({@code Projection.pictureHeight}: π : 1) inside (w, h). */
    static int[] fit(int w, int h) {
        int mw = w;
        while (mw > 1 && com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureHeight(mw) > h)
            mw--;
        return new int[]{mw, com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureHeight(mw)};
    }

    @Override
    protected void renderWidget(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        int x = getX(), y = getY(), w = getWidth(), h = getHeight();
        g.fill(x - 1, y - 1, x + w + 1, y + h + 1, 0xff202020);
        g.fill(x, y, x + w, y + h, 0xff000000);
        if (texture.ready()) {
            RenderSystem.enableBlend();
            g.blit(texture.id(), x, y, w, h, 0, 0, texture.width(), texture.height(), texture.width(), texture.height());
            RenderSystem.disableBlend();
        }
        var font = net.minecraft.client.Minecraft.getInstance().font;
        for (Box b : boxes.get()) {
            int x0 = x + (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureXOfLongitude(b.lonWest(), w)),
                    x1 = x + (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureXOfLongitude(b.lonEast(), w));
            int y0 = y + (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureYOfLatitude(b.latNorth(), h)),
                    y1 = y + (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureYOfLatitude(b.latSouth(), h));
            g.fill(x0, y0, x1, y1, b.argb());
            g.renderOutline(x0, y0, x1 - x0, y1 - y0, (b.argb() | 0xff000000));
            if (!b.label().isEmpty() && x1 - x0 > font.width(b.label()) + 2)
                g.drawString(font, b.label(), x0 + 2, y0 + 2, 0xffffffff, true);
        }
        double[] p = new double[2];
        for (Marker m : markers.get()) {
            GlobeProjection.mapPixel(m.lat(), m.lon(), w, h, p);
            int cx = x + (int) p[0], cy = y + (int) p[1];
            int r = m.fresh() ? 2 : 1;
            g.fill(cx - r - 1, cy - r - 1, cx + r + 2, cy + r + 2, 0xff000000);
            g.fill(cx - r, cy - r, cx + r + 1, cy + r + 1, m.colour());
        }
        if (isHovered()) {
            double lon = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureLongitudeDeg(mouseX - x, w);
            double lat = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureLatitudeDeg(mouseY - y, h);
            String s = String.format(java.util.Locale.ROOT, "%.1f°%s %.1f°E", Math.abs(lat), lat >= 0 ? "N" : "S", lon);
            g.drawString(font, s, x + 3, y + h - 11, 0xffffffff, true);
        }
    }

    /** Markers for point events. */
    static List<Marker> eventMarkers(List<com.minecraftindustrialcomplex.deeptime.screen.model.GeoEvent> events,
            java.util.function.Predicate<com.minecraftindustrialcomplex.deeptime.screen.model.GeoEvent> fresh) {
        List<Marker> out = new ArrayList<>();
        for (var e : events)
            out.add(new Marker(e.lat(), e.lon(), fresh.test(e) ? 0xffffffff : 0x99ffffff, fresh.test(e)));
        return out;
    }

    @Override
    protected void updateWidgetNarration(NarrationElementOutput out) {
        defaultButtonNarrationText(out);
    }

    @Override
    public void playDownSound(net.minecraft.client.sounds.SoundManager handler) {
        // a map is not a button
    }
}
