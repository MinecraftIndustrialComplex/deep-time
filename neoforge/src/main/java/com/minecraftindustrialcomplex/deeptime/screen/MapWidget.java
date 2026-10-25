// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.GlobeProjection;
import com.minecraftindustrialcomplex.deeptime.screen.model.NetView;
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
 *
 * <p>The review page of a cube world shows the net instead ({@link NetView}, cube plan G13): the
 * picture is the window, 4 : 3, markers are placed at their point on the faces, the province hints
 * come as a second picture over it, and the border's lines are drawn on top.
 */
class MapWidget extends AbstractWidget {

    /** A marker: a dot in its event kind's colour, larger and ringed in white when {@code fresh}. */
    record Marker(double lat, double lon, int colour, boolean fresh) {}

    /** A tinted latitude–longitude box (province hints). */
    record Box(double lonWest, double lonEast, double latNorth, double latSouth, int argb, String label) {}

    private final KeyframeTexture texture;
    private final Supplier<List<Marker>> markers;
    private final Supplier<List<Box>> boxes;
    /** A cube world's net (null: the Lambert picture), its hints picture and whether it shows. */
    private final NetView netView;
    private final KeyframeTexture overlay;
    private final java.util.function.BooleanSupplier overlayOn;

    MapWidget(int x, int y, int w, int h, KeyframeTexture texture, Supplier<List<Marker>> markers, Supplier<List<Box>> boxes) {
        this(x, y, w, h, texture, markers, boxes, null, null, () -> false);
    }

    /** A cube world's net ({@code net}), with {@code overlay} over the picture while {@code overlayOn}. */
    MapWidget(int x, int y, int w, int h, KeyframeTexture texture, Supplier<List<Marker>> markers, NetView net,
            KeyframeTexture overlay, java.util.function.BooleanSupplier overlayOn) {
        this(x, y, w, h, texture, markers, List::of, net, overlay, overlayOn);
    }

    private MapWidget(int x, int y, int w, int h, KeyframeTexture texture, Supplier<List<Marker>> markers, Supplier<List<Box>> boxes,
            NetView net, KeyframeTexture overlay, java.util.function.BooleanSupplier overlayOn) {
        super(x, y, w, h, Component.empty());
        this.texture = texture;
        this.markers = markers;
        this.boxes = boxes;
        this.netView = net;
        this.overlay = overlay;
        this.overlayOn = overlayOn;
    }

    /** The largest box of {@code aw : ah} inside (w, h) (the net's 4 : 3). */
    static int[] fit(int w, int h, int aw, int ah) {
        int mw = Math.max(1, Math.min(w, h * aw / ah));
        return new int[]{mw, Math.max(1, mw * ah / aw)};
    }

    /** The largest box of the map picture's aspect ({@code MapPicture.height}: π : 1) inside (w, h). */
    static int[] fit(int w, int h) {
        int mw = w;
        while (mw > 1 && com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture.height(mw) > h)
            mw--;
        return new int[]{mw, com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture.height(mw)};
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
        if (netView != null) {
            renderNet(g, mouseX, mouseY, x, y, w, h);
            return;
        }
        var font = net.minecraft.client.Minecraft.getInstance().font;
        for (Box b : boxes.get()) {
            int x0 = x + (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture.xOfLongitude(b.lonWest(), w)),
                    x1 = x + (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture.xOfLongitude(b.lonEast(), w));
            int y0 = y + (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture.yOfLatitude(b.latNorth(), h)),
                    y1 = y + (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture.yOfLatitude(b.latSouth(), h));
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
            // Fresh events (after the previous frame) get a white ring, older ones a dark one (workbench colours by kind).
            g.fill(cx - r - 1, cy - r - 1, cx + r + 2, cy + r + 2, m.fresh() ? 0xffffffff : 0xff000000);
            g.fill(cx - r, cy - r, cx + r + 1, cy + r + 1, m.colour());
        }
        if (isHovered()) {
            double lon = com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture.longitudeDeg(mouseX - x, w);
            double lat = com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture.latitudeDeg(mouseY - y, h);
            String s = String.format(java.util.Locale.ROOT, "%.1f°%s %.1f°E", Math.abs(lat), lat >= 0 ? "N" : "S", lon);
            g.drawString(font, s, x + 3, y + h - 11, 0xffffffff, true);
        }
    }

    /** The net's hints, lines, markers and hover (the picture is drawn). */
    private void renderNet(GuiGraphics g, int mouseX, int mouseY, int x, int y, int w, int h) {
        if (overlay != null && overlay.ready() && overlayOn.getAsBoolean()) {
            RenderSystem.enableBlend();
            g.blit(overlay.id(), x, y, w, h, 0, 0, overlay.width(), overlay.height(), overlay.width(), overlay.height());
            RenderSystem.disableBlend();
        }
        // The joined face sides faint, then the border on top, two pixels wide straddling its line.
        for (int pass = 0; pass < 2; pass++)
            for (NetView.Line l : netView.lines()) {
                boolean border = l.kind() == NetView.LineKind.BORDER;
                if (border != (pass == 1))
                    continue;
                int x0 = x + (int) Math.round(l.fx0() * w), y0 = y + (int) Math.round(l.fy0() * h);
                int x1 = x + (int) Math.round(l.fx1() * w), y1 = y + (int) Math.round(l.fy1() * h);
                int colour = border ? 0xff000000 | GlobeWidget.abgrToArgb(GlobeProjection.BORDER_ABGR) : 0x60ffffff;
                int t0 = border ? -1 : 0, t1 = 1;
                if (x0 == x1)
                    g.fill(x0 + t0, Math.min(y0, y1), x0 + t1, Math.max(y0, y1), colour);
                else
                    g.fill(Math.min(x0, x1), y0 + t0, Math.max(x0, x1), y0 + t1, colour);
            }
        for (Marker m : markers.get()) {
            double[] f = netView.fractionOfLatLon(m.lat(), m.lon());
            if (Double.isNaN(f[0]) || Double.isNaN(f[1]))
                continue;
            int cx = x + (int) (f[0] * w), cy = y + (int) (f[1] * h);
            int r = m.fresh() ? 2 : 1;
            g.fill(cx - r - 1, cy - r - 1, cx + r + 2, cy + r + 2, m.fresh() ? 0xffffffff : 0xff000000);
            g.fill(cx - r, cy - r, cx + r + 1, cy + r + 1, m.colour());
        }
        if (isHovered()) {
            double fx = (mouseX - x + 0.5) / w, fy = (mouseY - y + 0.5) / h;
            double[] ll = netView.latLonAt(fx, fy);
            var face = netView.faceAt(fx, fy);
            if (ll != null && face != null) {
                double lon = ll[1] > 180.0 ? ll[1] - 360.0 : ll[1];
                String s = String.format(java.util.Locale.ROOT, "%.1f°%s %.1f°%s · %s", Math.abs(ll[0]), ll[0] >= 0 ? "N" : "S",
                        Math.abs(lon), lon >= 0 ? "E" : "W", face.name());
                g.drawString(net.minecraft.client.Minecraft.getInstance().font, s, x + 3, y + h - 11, 0xffffffff, true);
            }
        }
    }

    /** Markers for point events. */
    static List<Marker> eventMarkers(List<com.minecraftindustrialcomplex.deeptime.screen.model.GeoEvent> events,
            java.util.function.Predicate<com.minecraftindustrialcomplex.deeptime.screen.model.GeoEvent> fresh) {
        List<Marker> out = new ArrayList<>();
        for (var e : events)
            out.add(new Marker(e.lat(), e.lon(), com.minecraftindustrialcomplex.deeptime.screen.model.EventStyle.colour(e.kind()), fresh.test(e)));
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
