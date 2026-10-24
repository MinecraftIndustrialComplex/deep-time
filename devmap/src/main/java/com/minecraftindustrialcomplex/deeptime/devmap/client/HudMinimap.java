// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import com.minecraftindustrialcomplex.deeptime.devmap.model.Graticule;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapFrame;
import com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometry;
import com.minecraftindustrialcomplex.deeptime.devmap.model.ViewTransform;
import com.minecraftindustrialcomplex.deeptime.devmap.net.DevMapState;
import com.minecraftindustrialcomplex.deeptime.devmap.net.PlayersPayload;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;

/**
 * The corner minimap (the HUD key, period by default): a north-up square of the block-space map
 * around the player in the dev map's picture (satellite by default, the choice made with S on the
 * map screen), with the graticule, the seam, the edges, the poles and the players, and the
 * player's latitude and longitude under it. Top right, or bottom right while F3 is open so it does
 * not cover the debug text. Hidden while the dev map screen is open. On a cube the seam is the slide
 * and the edges are the border's segments, as on the map screen.
 */
final class HudMinimap {

    private HudMinimap() {}

    static void render(GuiGraphics g, DeltaTracker delta) {
        Minecraft mc = Minecraft.getInstance();
        if (!DevMapClient.hudOn() || mc.player == null || mc.level == null || mc.options.hideGui
                || mc.screen instanceof DevMapScreen)
            return;
        MapContext c = MapContext.current();
        Font font = mc.font;
        int size = ClientConfig.hudSize();
        int sw = g.guiWidth(), sh = g.guiHeight();
        int left = sw - size - 4;
        boolean f3 = mc.getDebugOverlay().showDebugScreen();
        int top = f3 ? sh - size - 4 - 22 : 4;
        int bottom = top + size;
        if (c.projection() == null) {
            g.fill(left, top, left + size, top + 24, 0xA0000000);
            g.drawString(font, "dev map:", left + 3, top + 3, 0xFFE0E0E0, false);
            g.drawString(font, c.problem() == null ? "" : shorten(font, c.problem(), size - 6), left + 3, top + 13, 0xFFE0E0E0, false);
            return;
        }
        WorldProjection p = c.projection();
        double P = p.period(), zMin = p.zMin(), zMax = p.zMax();
        double scale = size / (double) ClientConfig.hudBlocks();
        ViewTransform v = new ViewTransform(P, mc.player.getX(), mc.player.getZ(), scale);
        v.viewport(left, top, size, size);
        float px = Draw.px();
        g.fill(left - 1, top - 1, left + size + 1, bottom + 1, 0xFF000000);
        g.enableScissor(left, top, left + size, bottom);
        Draw.rect(g, left, top, left + size, bottom, 0xFF101418);
        double w0 = PlanetGeometry.west(p);
        boolean cube = PlanetGeometry.cube(p) != null;
        int k0 = v.firstCopy(w0, w0 + P) - 1, k1 = v.lastCopy(w0, w0 + P) + 1;
        if (MapTextures.map(c.image())) {
            MapFrame f = c.image().frame();
            for (int k = k0; k <= k1; k++) {
                double x1 = v.screenX(f.x0() + k * P), x2 = v.screenX(f.x0() + f.w() * f.blocksPerPx() + k * P);
                if (x2 < left || x1 > left + size)
                    continue;
                Draw.texture(g, MapTextures.MAP, x1, v.screenY(f.z0()), x2, v.screenY(f.z0() + f.h() * f.blocksPerPx()), 0, 0, 1, 1);
            }
        } else {
            for (int k = k0; k <= k1; k++)
                for (double[] q : PlanetGeometry.pieces(p))
                    Draw.rect(g, v.screenX(q[0] + k * P), v.screenY(q[1]), v.screenX(q[2] + k * P), v.screenY(q[3]), 0xFF2A4A6A);
        }
        double l = left - 4, r = left + size + 4, t = top - 4, b = bottom + 4;
        for (Graticule.Line line : c.graticule()) {
            int col = Math.abs(line.value()) < 1e-9 ? DevMapScreen.GRATICULE_MAIN : DevMapScreen.GRATICULE;
            for (int k = k0; k <= k1; k++) {
                double[] xs = line.xs(), zs = line.zs();
                double ax = v.screenX(xs[0] + k * P), ay = v.screenY(zs[0]);
                for (int i = 1; i < xs.length; i++) {
                    double bx = v.screenX(xs[i] + k * P), by = v.screenY(zs[i]);
                    if (!((ax < l && bx < l) || (ax > r && bx > r) || (ay < t && by < t) || (ay > b && by > b)))
                        Draw.line(g, ax, ay, bx, by, px, col);
                    ax = bx;
                    ay = by;
                }
            }
        }
        for (int k = k0; k <= k1; k++) {
            if (cube) {
                PlanetGeometry.Polyline slide = PlanetGeometry.slide(p);
                double sx = v.screenX(slide.xs()[0] + k * P);
                if (sx >= l && sx <= r)
                    Draw.dashed(g, sx, Math.max(t, v.screenY(slide.zs()[0])), sx, Math.min(b, v.screenY(slide.zs()[1])), 2 * px, 3, DevMapScreen.SEAM);
                for (PlanetGeometry.Polyline e : PlanetGeometry.borderOutline(p, 0.0)) {
                    double[] xs = e.xs(), zs = e.zs();
                    for (int i = 1; i < xs.length; i++)
                        Draw.line(g, v.screenX(xs[i - 1] + k * P), v.screenY(zs[i - 1]), v.screenX(xs[i] + k * P), v.screenY(zs[i]), 2 * px,
                                DevMapScreen.EDGE);
                }
            } else {
                double sx = v.screenX(P / 2 + k * P);
                if (sx >= l && sx <= r)
                    Draw.dashed(g, sx, Math.max(t, v.screenY(zMin)), sx, Math.min(b, v.screenY(zMax)), 2 * px, 3, DevMapScreen.SEAM);
            }
            for (PlanetGeometry.Pole pole : PlanetGeometry.poles(p))
                if (!pole.line())
                    for (double[] im : pole.images())
                        Draw.ring(g, v.screenX(im[0] + k * P), v.screenY(im[1]), 4, 2 * px, DevMapScreen.POLE);
        }
        for (double ze : cube ? new double[0] : new double[]{zMin, zMax}) {
            double sy = v.screenY(ze);
            if (sy >= t && sy <= b)
                Draw.line(g, left, sy, left + size, sy, 2 * px, DevMapScreen.EDGE);
        }
        String self = mc.player.getGameProfile().getName();
        for (PlayersPayload.Entry e : DevMapState.players(c.dimension())) {
            if (e.name().equals(self))
                continue;
            double dx = PlanetGeometry.wrapCentred(p, e.x() - mc.player.getX());
            double yaw = Math.toRadians(e.yaw());
            Draw.arrow(g, v.screenX(mc.player.getX() + dx), v.screenY(e.z()), -Math.sin(yaw), Math.cos(yaw), 4, DevMapScreen.OTHER, DevMapScreen.OUTLINE);
        }
        double yaw = Math.toRadians(mc.player.getYRot());
        Draw.arrow(g, left + size / 2.0, top + size / 2.0, -Math.sin(yaw), Math.cos(yaw), 5, DevMapScreen.SELF, 0xFFFFFFFF);
        g.flush();
        g.disableScissor();
        double[] ll = PlanetGeometry.displayLatLon(p, mc.player.getX(), mc.player.getZ());
        String s1 = PlanetGeometry.formatLatLon(ll[0], ll[1]);
        String s2 = cube
                ? String.format(java.util.Locale.ROOT, "%s  slide %.0f  border %.0f", PlanetGeometry.faceName(p, mc.player.getX(), mc.player.getZ()),
                        PlanetGeometry.seamDistance(p, mc.player.getX(), mc.player.getZ()), PlanetGeometry.edgeDistance(p, mc.player.getX(), mc.player.getZ()))
                : String.format(java.util.Locale.ROOT, "seam %.0f  pole %.0f", PlanetGeometry.seamDistance(p, mc.player.getX(), mc.player.getZ()),
                        PlanetGeometry.nearestPole(p, mc.player.getX(), mc.player.getZ()).blocks());
        g.fill(left - 1, bottom + 1, left + size + 1, bottom + 21, 0xA0000000);
        g.drawString(font, shorten(font, s1, size), left + 1, bottom + 2, 0xFFE0E0E0, false);
        g.drawString(font, shorten(font, s2, size), left + 1, bottom + 11, 0xFFA0A0A0, false);
    }

    private static String shorten(Font font, String s, int width) {
        return font.width(s) <= width ? s : font.plainSubstrByWidth(s, width);
    }
}
