// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import com.minecraftindustrialcomplex.deeptime.devmap.model.Graticule;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapFrame;
import com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometry;
import com.minecraftindustrialcomplex.deeptime.devmap.model.ViewTransform;
import com.minecraftindustrialcomplex.deeptime.devmap.net.PlayersPayload;
import com.minecraftindustrialcomplex.deeptime.devmap.net.TeleportPayload;
import com.minecraftindustrialcomplex.deeptime.net.PlanetDebugLine;
import com.minecraftindustrialcomplex.deeptime.screen.model.GlobeProjection;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.neoforged.neoforge.network.PacketDistributor;
import org.lwjgl.glfw.GLFW;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * The dev map (key: comma by default). Two views of the planet the player is on, both through
 * Deep Time's own projection:
 * <ul>
 *   <li><b>Map</b>: the world as it is laid out in block space (north up), the base image tiled
 *       across the x wrap, the latitude/longitude graticule, the wrap seam (magenta), the edges or
 *       pole lines (orange), the poles, the cone points, the world spawn and the players.</li>
 *   <li><b>Globe</b>: an orthographic sphere ({@code GlobeProjection}, the Create World screen's
 *       globe) of the same picture with the same marks, centred on the player at first.</li>
 * </ul>
 * Left-drag pans or turns, the wheel zooms, right-click teleports there (operators, on a server
 * with the dev map), Tab switches the view, C centres on the player, H toggles the HUD minimap.
 * The readout under the view gives the player's block, latitude and longitude (the F3 line's
 * convention, with Deep Time's own F3 line beside it for comparison), the distances to the seam,
 * the edge and the nearest pole, and the projection.
 */
public final class DevMapScreen extends Screen {

    public enum Mode { MAP, GLOBE }

    static final int GRATICULE = 0x70FFFFFF, GRATICULE_MAIN = 0xD0FFFFFF, SEAM = 0xFFFF3CFF, EDGE = 0xFFFFA020,
            BORDER = 0xA0FFA020, POLE = 0xFF40FFFF, CONE = 0xFFFFFF40, SPAWN = 0xFFFFFFFF, OTHER = 0xFF40C0FF,
            SELF = 0xFFFF3030, OUTLINE = 0xFF000000, PANEL = 0xC0000000, TEXT = 0xFFE0E0E0, DIM = 0xFFA0A0A0;

    private static final int TOP = 22, LINE_H = 10, READOUT_LINES = 7;
    private static Mode lastMode = Mode.MAP;

    private Mode mode = lastMode;
    private ViewTransform view;
    private double globeLat, globeLon, globeRadius = -1;
    private boolean dragging;
    private double hoverX = Double.NaN, hoverZ = Double.NaN;
    private String status = "";
    private int vl, vt, vw, vh;
    private Button hudButton;

    public DevMapScreen() {
        super(Component.literal("Deep Time dev map"));
    }

    @Override
    public boolean isPauseScreen() {
        return false;
    }

    @Override
    protected void init() {
        int x = 4;
        addRenderableWidget(Button.builder(Component.literal("Map"), b -> setMode(Mode.MAP)).bounds(x, 3, 40, 16).build());
        x += 42;
        addRenderableWidget(Button.builder(Component.literal("Globe"), b -> setMode(Mode.GLOBE)).bounds(x, 3, 40, 16).build());
        x += 42;
        addRenderableWidget(Button.builder(Component.literal("Centre (C)"), b -> centreOnPlayer()).bounds(x, 3, 60, 16).build());
        x += 62;
        hudButton = addRenderableWidget(Button.builder(hudLabel(), b -> {
            DevMapClient.toggleHud();
            b.setMessage(hudLabel());
        }).bounds(x, 3, 70, 16).build());
        vl = 4;
        vt = TOP;
        vw = width - 8;
        vh = Math.max(40, height - TOP - (READOUT_LINES * LINE_H + 8) - 4);
        MapContext c = MapContext.current();
        if (view == null && c.projection() != null) {
            view = new ViewTransform(c.projection().period(), 0, 0, 1);
            view.viewport(vl, vt, vw, vh);
            showWholeWorld();
            centreOnPlayer();
        } else if (view != null) {
            view.viewport(vl, vt, vw, vh);
        }
        if (globeRadius < 0)
            globeRadius = Math.min(vw, vh) * 0.46;
    }

    private static Component hudLabel() {
        return Component.literal("HUD map (H): " + (DevMapClient.hudOn() ? "on" : "off"));
    }

    // ---- what the shot driver and the keys use ----

    public Mode mode() {
        return mode;
    }

    public void setMode(Mode m) {
        mode = m;
        lastMode = m;
    }

    /** Fits the whole world's width into the view, centred on the player. */
    public void showWholeWorld() {
        MapContext c = MapContext.current();
        if (view == null || c.projection() == null)
            return;
        double s = Math.min(vw / c.projection().period(), vh / (c.projection().zMax() - c.projection().zMin()));
        view.scale(s);
        view.centre(view.cx(), 0.5 * (c.projection().zMin() + c.projection().zMax()));
        globeRadius = Math.min(vw, vh) * 0.46;
    }

    /** Zooms the map so {@code blocks} blocks span the view's width. */
    public void zoomBlocks(double blocks) {
        if (view != null)
            view.scale(vw / blocks);
    }

    public void centreOnPlayer() {
        Minecraft mc = Minecraft.getInstance();
        MapContext c = MapContext.current();
        if (mc.player == null || c.projection() == null)
            return;
        if (view != null)
            view.centre(mc.player.getX(), mc.player.getZ());
        double[] ll = c.projection().latLon(mc.player.getX(), mc.player.getZ());
        globeLat = Math.max(-89.9, Math.min(89.9, ll[0]));
        globeLon = ll[1];
    }

    /** Centres the map on block (x, z) and the globe on its latitude and longitude. */
    public void centreOn(double x, double z) {
        MapContext c = MapContext.current();
        if (c.projection() == null)
            return;
        if (view != null)
            view.centre(x, z);
        double[] ll = c.projection().latLon(x, z);
        globeLat = Math.max(-89.9, Math.min(89.9, ll[0]));
        globeLon = ll[1];
    }

    /** Turns the globe to centre (lat, lon), simulation longitude. */
    public void globeCentre(double lat, double lon) {
        globeLat = Math.max(-89.9, Math.min(89.9, lat));
        globeLon = lon;
    }

    public ViewTransform view() {
        return view;
    }

    /** The globe's centre {lat, lon} (simulation longitude) and radius, GUI units. */
    public double[] globe() {
        return new double[]{globeLat, globeLon, globeRadius, vl + vw / 2.0, vt + vh / 2.0};
    }

    /** The block (x, z) under screen point (sx, sy), or null (off the globe, or no planet). */
    public double[] blockAt(double sx, double sy) {
        MapContext c = MapContext.current();
        WorldProjection p = c.projection();
        if (p == null || sx < vl || sx > vl + vw || sy < vt || sy > vt + vh)
            return null;
        if (mode == Mode.MAP) {
            double bz = view.blockZ(sy);
            return bz < p.zMin() || bz > p.zMax() ? null : p.canonical(view.blockX(sx), bz);
        }
        double u = (sx - (vl + vw / 2.0)) / globeRadius, v = ((vt + vh / 2.0) - sy) / globeRadius;
        double[] ll = new GlobeProjection(globeLat, globeLon).inverse(u, v, new double[2]);
        return ll == null ? null : PlanetGeometry.blockOf(p, ll[0], ll[1]);
    }

    /** The right-click: asks the server to put the player at the block under (sx, sy). */
    public boolean teleportAt(double sx, double sy) {
        MapContext c = MapContext.current();
        double[] b = blockAt(sx, sy);
        if (b == null)
            return false;
        if (!MapContext.serverHasDevMap()) {
            status = "no teleport: the server has no dev map";
            return false;
        }
        if (!c.canTeleport()) {
            status = "no teleport: needs operator permission";
            return false;
        }
        PacketDistributor.sendToServer(new TeleportPayload(c.dimension(), b[0], b[1]));
        status = String.format(Locale.ROOT, "teleport asked: %.0f %.0f", b[0], b[1]);
        return true;
    }

    /** The screen position of block (x, z) in the map view, nearest copy to the view's centre. */
    public double[] mapScreenPos(double x, double z) {
        MapContext c = MapContext.current();
        if (view == null || c.projection() == null)
            return null;
        double P = c.projection().period();
        double dx = c.projection().wrapX(x - view.cx());
        return new double[]{view.screenX(view.cx() + dx), view.screenY(z)};
    }

    // ---- input ----

    @Override
    public boolean keyPressed(int keyCode, int scanCode, int modifiers) {
        if (DevMapClient.OPEN.matches(keyCode, scanCode)) {
            onClose();
            return true;
        }
        if (keyCode == GLFW.GLFW_KEY_TAB) {
            setMode(mode == Mode.MAP ? Mode.GLOBE : Mode.MAP);
            return true;
        }
        if (keyCode == GLFW.GLFW_KEY_C) {
            centreOnPlayer();
            return true;
        }
        if (keyCode == GLFW.GLFW_KEY_H) {
            DevMapClient.toggleHud();
            if (hudButton != null)
                hudButton.setMessage(hudLabel());
            return true;
        }
        return super.keyPressed(keyCode, scanCode, modifiers);
    }

    private boolean inView(double x, double y) {
        return x >= vl && x <= vl + vw && y >= vt && y <= vt + vh;
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (super.mouseClicked(mouseX, mouseY, button))
            return true;
        if (!inView(mouseX, mouseY))
            return false;
        if (button == GLFW.GLFW_MOUSE_BUTTON_LEFT) {
            dragging = true;
            return true;
        }
        if (button == GLFW.GLFW_MOUSE_BUTTON_RIGHT)
            return teleportAt(mouseX, mouseY);
        return false;
    }

    @Override
    public boolean mouseReleased(double mouseX, double mouseY, int button) {
        if (button == GLFW.GLFW_MOUSE_BUTTON_LEFT)
            dragging = false;
        return super.mouseReleased(mouseX, mouseY, button);
    }

    @Override
    public boolean mouseDragged(double mouseX, double mouseY, int button, double dragX, double dragY) {
        if (dragging && button == GLFW.GLFW_MOUSE_BUTTON_LEFT) {
            if (mode == Mode.MAP && view != null) {
                view.pan(dragX, dragY);
                MapContext c = MapContext.current();
                if (c.projection() != null)
                    view.clampZ(c.projection().zMin(), c.projection().zMax());
            } else {
                double k = Math.toDegrees(1.0 / globeRadius);
                globeLon -= dragX * k;
                globeLat = Math.max(-89.9, Math.min(89.9, globeLat + dragY * k));
            }
            return true;
        }
        return super.mouseDragged(mouseX, mouseY, button, dragX, dragY);
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double scrollX, double scrollY) {
        if (!inView(mouseX, mouseY) || scrollY == 0)
            return super.mouseScrolled(mouseX, mouseY, scrollX, scrollY);
        double f = Math.pow(1.25, scrollY);
        MapContext c = MapContext.current();
        if (mode == Mode.MAP && view != null && c.projection() != null) {
            double min = 0.25 * vw / c.projection().period();
            view.zoomAt(mouseX, mouseY, f, min, 8.0);
        } else {
            globeRadius = Math.max(20, Math.min(Math.min(vw, vh) * 8.0, globeRadius * f));
        }
        return true;
    }

    // ---- drawing ----

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        super.render(g, mouseX, mouseY, partialTick);
        MapContext c = MapContext.current();
        Minecraft mc = Minecraft.getInstance();
        g.drawString(font, "Deep Time dev map: " + (mode == Mode.MAP ? "map (block space)" : "globe")
                + "   drag: pan/turn   wheel: zoom   right-click: teleport   Tab: map/globe", 4 + 42 * 2 + 62 + 72 + 6, 7, DIM, false);
        if (c.problem() != null || view == null) {
            Draw.rect(g, vl, vt, vl + vw, vt + vh, 0xFF101418);
            g.flush();
            String msg = c.problem() != null ? c.problem() : "waiting for Deep Time's planet info";
            g.drawCenteredString(font, msg, vl + vw / 2, vt + vh / 2 - 4, TEXT);
            if (view == null && c.projection() != null)
                init(mc, width, height);
            return;
        }
        double[] hover = inView(mouseX, mouseY) ? blockAt(mouseX, mouseY) : null;
        hoverX = hover == null ? Double.NaN : hover[0];
        hoverZ = hover == null ? Double.NaN : hover[1];
        g.enableScissor(vl, vt, vl + vw, vt + vh);
        if (mode == Mode.MAP)
            renderMap(g, c);
        else
            renderGlobe(g, c);
        g.flush();
        g.disableScissor();
        renderReadout(g, c);
    }

    private void renderMap(GuiGraphics g, MapContext c) {
        WorldProjection p = c.projection();
        Minecraft mc = Minecraft.getInstance();
        double P = p.period(), zMin = p.zMin(), zMax = p.zMax();
        view.viewport(vl, vt, vw, vh);
        float px = Draw.px();
        Draw.rect(g, vl, vt, vl + vw, vt + vh, 0xFF101418);
        int k0 = view.firstCopy(-P / 2, P / 2) - 1, k1 = view.lastCopy(-P / 2, P / 2) + 1;
        if (MapTextures.map(c.image())) {
            MapFrame f = c.image().frame();
            for (int k = k0; k <= k1; k++) {
                double x1 = view.screenX(f.x0() + k * P), x2 = view.screenX(f.x0() + f.w() * f.blocksPerPx() + k * P);
                if (x2 < vl || x1 > vl + vw)
                    continue;
                Draw.texture(g, MapTextures.MAP, x1, view.screenY(f.z0()), x2, view.screenY(f.z0() + f.h() * f.blocksPerPx()), 0, 0, 1, 1);
            }
        } else {
            for (int k = k0; k <= k1; k++)
                Draw.rect(g, view.screenX(-P / 2 + k * P), view.screenY(zMin), view.screenX(P / 2 + k * P), view.screenY(zMax), 0xFF2A4A6A);
            g.flush();
        }
        // The graticule.
        for (Graticule.Line line : c.graticule()) {
            boolean main = Math.abs(line.value()) < 1e-9;
            int col = main ? GRATICULE_MAIN : GRATICULE;
            double w = main ? 2 * px : px;
            for (int k = k0; k <= k1; k++)
                polyline(g, line.xs(), line.zs(), k * P, w, col);
        }
        // The seam, the edges or pole lines, the edge border.
        for (int k = k0; k <= k1; k++) {
            double sx = view.screenX(P / 2 + k * P);
            Draw.dashed(g, sx, view.screenY(zMin), sx, view.screenY(zMax), 2 * px, 4, SEAM);
        }
        double xa = view.blockX(vl) - 1, xb = view.blockX(vl + vw) + 1;
        for (double ze : new double[]{zMin, zMax})
            Draw.line(g, view.screenX(xa), view.screenY(ze), view.screenX(xb), view.screenY(ze), 2 * px, EDGE);
        double inset = c.devInfo() != null ? c.devInfo().borderInset() : Double.NaN;
        if (!Double.isNaN(inset) && inset > 0)
            for (double ze : new double[]{zMin + inset, zMax - inset})
                Draw.dashed(g, view.screenX(xa), view.screenY(ze), view.screenX(xb), view.screenY(ze), px, 3, BORDER);
        // Cone points and poles.
        for (int k = k0; k <= k1; k++) {
            for (double[] cp : PlanetGeometry.conePoints(p))
                Draw.cross(g, view.screenX(cp[0] + k * P), view.screenY(cp[1]), 4, 2 * px, CONE);
            for (PlanetGeometry.Pole pole : PlanetGeometry.poles(p)) {
                if (pole.line())
                    continue;
                for (double[] im : pole.images())
                    Draw.ring(g, view.screenX(im[0] + k * P), view.screenY(im[1]), 5, 2 * px, POLE);
            }
        }
        // Spawn, other players, the player.
        if (mc.level != null) {
            BlockPos sp = mc.level.getSharedSpawnPos();
            double[] s = mapScreenPos(sp.getX() + 0.5, sp.getZ() + 0.5);
            Draw.diamond(g, s[0], s[1], 4, OUTLINE);
            Draw.diamond(g, s[0], s[1], 3, SPAWN);
        }
        String self = mc.player.getGameProfile().getName();
        for (PlayersPayload.Entry e : otherPlayers(c, self)) {
            double[] s = mapScreenPos(e.x(), e.z());
            double yaw = Math.toRadians(e.yaw());
            Draw.arrow(g, s[0], s[1], -Math.sin(yaw), Math.cos(yaw), 5, OTHER, OUTLINE);
        }
        double[] me = mapScreenPos(mc.player.getX(), mc.player.getZ());
        double yaw = Math.toRadians(mc.player.getYRot());
        Draw.arrow(g, me[0], me[1], -Math.sin(yaw), Math.cos(yaw), 7, SELF, 0xFFFFFFFF);
        if (!Double.isNaN(hoverX)) {
            double[] h = mapScreenPos(hoverX, hoverZ);
            Draw.cross(g, h[0], h[1], 3, px, 0xFFFFFFFF);
        }
        g.flush();
        // Labels.
        for (int k = k0; k <= k1; k++) {
            for (PlanetGeometry.Pole pole : PlanetGeometry.poles(p)) {
                if (pole.line()) {
                    if (k == 0)
                        label(g, pole.name() + " pole (z = " + Math.round(pole.images().get(0)[1]) + ")", vl + 4,
                                view.screenY(pole.images().get(0)[1]) + (pole.latDeg() > 0 ? 3 : -11), POLE);
                    continue;
                }
                for (double[] im : pole.images())
                    label(g, pole.name().equals("north") ? "N pole" : "S pole", view.screenX(im[0] + k * P) + 7,
                            view.screenY(im[1]) + (pole.latDeg() > 0 ? 3 : -11), POLE);
            }
            double sx = view.screenX(P / 2 + k * P);
            label(g, "seam x = ±" + Math.round(P / 2), sx + 3, view.screenY(0.5 * (zMin + zMax)), SEAM);
        }
        for (PlayersPayload.Entry e : otherPlayers(c, self)) {
            double[] s = mapScreenPos(e.x(), e.z());
            label(g, e.name(), s[0] + 7, s[1] - 4, OTHER);
        }
        if (mc.level != null) {
            BlockPos sp = mc.level.getSharedSpawnPos();
            double[] s = mapScreenPos(sp.getX() + 0.5, sp.getZ() + 0.5);
            label(g, "spawn", s[0] + 6, s[1] - 4, SPAWN);
        }
        label(g, self, me[0] + 9, me[1] - 4, SELF);
        String scale = String.format(Locale.ROOT, "%.2f px/block, %.0f blocks across", view.scale() * mc.getWindow().getGuiScale(), vw / view.scale());
        label(g, scale, vl + 4, vt + vh - 12, TEXT);
    }

    private static List<PlayersPayload.Entry> otherPlayers(MapContext c, String self) {
        List<PlayersPayload.Entry> out = new ArrayList<>();
        for (PlayersPayload.Entry e : com.minecraftindustrialcomplex.deeptime.devmap.net.DevMapState.players(c.dimension()))
            if (!e.name().equals(self))
                out.add(e);
        return out;
    }

    private void polyline(GuiGraphics g, double[] xs, double[] zs, double dx, double w, int col) {
        double l = vl - 4, r = vl + vw + 4, t = vt - 4, b = vt + vh + 4;
        double px = view.screenX(xs[0] + dx), py = view.screenY(zs[0]);
        for (int i = 1; i < xs.length; i++) {
            double qx = view.screenX(xs[i] + dx), qy = view.screenY(zs[i]);
            if (!((px < l && qx < l) || (px > r && qx > r) || (py < t && qy < t) || (py > b && qy > b)))
                Draw.line(g, px, py, qx, qy, w, col);
            px = qx;
            py = qy;
        }
    }

    private void label(GuiGraphics g, String s, double x, double y, int col) {
        int w = font.width(s);
        g.fill((int) x - 1, (int) y - 1, (int) x + w + 1, (int) y + 9, 0x90000000);
        g.drawString(font, s, (int) x, (int) y, col, false);
    }

    private void renderGlobe(GuiGraphics g, MapContext c) {
        WorldProjection p = c.projection();
        Minecraft mc = Minecraft.getInstance();
        float px = Draw.px();
        double cx = vl + vw / 2.0, cy = vt + vh / 2.0, R = globeRadius;
        Draw.rect(g, vl, vt, vl + vw, vt + vh, 0xFF05070A);
        int size = (int) Math.max(128, Math.min(768, Math.round(2 * R * mc.getWindow().getGuiScale() / 32.0) * 32));
        int[] pic = MapTextures.picture(c.image(), p);
        MapTextures.globe(pic, globeLat, globeLon, size);
        Draw.texture(g, MapTextures.GLOBE, cx - R, cy - R, cx + R, cy + R, 0, 0, 1, 1);
        GlobeProjection gp = new GlobeProjection(globeLat, globeLon);
        // Graticule, in the F3 longitude's frame (meridian 0 bold), drawn straight from latitude/longitude.
        for (int k = 0; k < 24; k++) {
            double d = -180.0 + 15.0 * k;
            double sim = PlanetGeometry.simLon(p, d);
            List<double[]> pts = new ArrayList<>();
            for (int i = 0; i <= 180; i++)
                pts.add(new double[]{-90.0 + i, sim});
            globeLine(g, gp, pts, d == 0 ? 2 * px : px, d == 0 ? GRATICULE_MAIN : GRATICULE);
        }
        for (int lat = -75; lat <= 75; lat += 15) {
            List<double[]> pts = new ArrayList<>();
            for (int i = 0; i <= 360; i++)
                pts.add(new double[]{lat, i});
            globeLine(g, gp, pts, lat == 0 ? 2 * px : px, lat == 0 ? GRATICULE_MAIN : GRATICULE);
        }
        // The seam and the edges, through the projection's latLon.
        double P = p.period(), zMin = p.zMin(), zMax = p.zMax();
        List<double[]> seam = new ArrayList<>();
        for (int i = 0; i <= 400; i++)
            seam.add(p.latLon(P / 2 - 1e-6, zMin + (zMax - zMin) * i / 400.0));
        globeLine(g, gp, seam, 2 * px, SEAM);
        for (double ze : new double[]{zMin, zMax}) {
            List<double[]> edge = new ArrayList<>();
            for (int i = 0; i <= 800; i++)
                edge.add(p.latLon(-P / 2 + P * i / 800.0, ze));
            globeLine(g, gp, edge, 2 * px, EDGE);
        }
        double[] uv = new double[2];
        for (double[] cp : PlanetGeometry.conePoints(p)) {
            double[] ll = p.latLon(cp[0], cp[1]);
            if (gp.forward(ll[0], ll[1], uv) != null)
                Draw.cross(g, cx + uv[0] * R, cy - uv[1] * R, 4, 2 * px, CONE);
        }
        for (PlanetGeometry.Pole pole : PlanetGeometry.poles(p))
            if (gp.forward(pole.latDeg(), 0, uv) != null)
                Draw.ring(g, cx + uv[0] * R, cy - uv[1] * R, 5, 2 * px, POLE);
        Draw.ring(g, cx, cy, R, px, 0x80FFFFFF);
        // Markers.
        String self = mc.player.getGameProfile().getName();
        List<String[]> labels = new ArrayList<>();
        if (mc.level != null) {
            BlockPos sp = mc.level.getSharedSpawnPos();
            double[] ll = p.latLon(sp.getX() + 0.5, sp.getZ() + 0.5);
            if (gp.forward(ll[0], ll[1], uv) != null) {
                Draw.diamond(g, cx + uv[0] * R, cy - uv[1] * R, 4, OUTLINE);
                Draw.diamond(g, cx + uv[0] * R, cy - uv[1] * R, 3, SPAWN);
                labels.add(new String[]{"spawn", String.valueOf(cx + uv[0] * R + 6), String.valueOf(cy - uv[1] * R - 4), String.valueOf(SPAWN)});
            }
        }
        for (PlayersPayload.Entry e : otherPlayers(c, self)) {
            double[] h = globeHeading(p, gp, e.x(), e.z(), e.yaw());
            if (h != null) {
                Draw.arrow(g, cx + h[0] * R, cy - h[1] * R, h[2], -h[3], 5, OTHER, OUTLINE);
                labels.add(new String[]{e.name(), String.valueOf(cx + h[0] * R + 7), String.valueOf(cy - h[1] * R - 4), String.valueOf(OTHER)});
            }
        }
        double[] h = globeHeading(p, gp, mc.player.getX(), mc.player.getZ(), mc.player.getYRot());
        if (h != null) {
            Draw.arrow(g, cx + h[0] * R, cy - h[1] * R, h[2], -h[3], 7, SELF, 0xFFFFFFFF);
            labels.add(new String[]{self, String.valueOf(cx + h[0] * R + 9), String.valueOf(cy - h[1] * R - 4), String.valueOf(SELF)});
        }
        if (!Double.isNaN(hoverX)) {
            double[] ll = p.latLon(hoverX, hoverZ);
            if (gp.forward(ll[0], ll[1], uv) != null)
                Draw.cross(g, cx + uv[0] * R, cy - uv[1] * R, 3, px, 0xFFFFFFFF);
        }
        g.flush();
        for (PlanetGeometry.Pole pole : PlanetGeometry.poles(p))
            if (gp.forward(pole.latDeg(), 0, uv) != null)
                label(g, pole.latDeg() > 0 ? "N pole" : "S pole", cx + uv[0] * R + 7, cy - uv[1] * R - 4, POLE);
        for (String[] l : labels)
            label(g, l[0], Double.parseDouble(l[1]), Double.parseDouble(l[2]), Integer.parseInt(l[3]));
        label(g, String.format(Locale.ROOT, "centre %s", PlanetGeometry.formatLatLon(globeLat, PlanetGeometry.displayLon(p, globeLon))),
                vl + 4, vt + vh - 12, TEXT);
    }

    /**
     * Where a player stands on the globe and which way it faces: {u, v, du, dv} (screen u right, v
     * up), from the point and a point 16 blocks ahead in block space, both through the projection;
     * null on the far side.
     */
    static double[] globeHeading(WorldProjection p, GlobeProjection gp, double x, double z, float yawDeg) {
        double yaw = Math.toRadians(yawDeg);
        double[] a = p.latLon(x, z);
        double[] b = p.latLon(x - Math.sin(yaw) * 16, z + Math.cos(yaw) * 16);
        double[] ua = gp.forward(a[0], a[1], new double[2]);
        if (ua == null)
            return null;
        double[] ub = gp.forward(b[0], b[1], new double[2]);
        double du = ub == null ? 0 : ub[0] - ua[0], dv = ub == null ? 1 : ub[1] - ua[1];
        return new double[]{ua[0], ua[1], du, dv};
    }

    private void globeLine(GuiGraphics g, GlobeProjection gp, List<double[]> latLons, double w, int col) {
        double cx = vl + vw / 2.0, cy = vt + vh / 2.0, R = globeRadius;
        double[] uv = new double[2];
        double lx = Double.NaN, ly = Double.NaN;
        for (double[] ll : latLons) {
            if (gp.forward(ll[0], ll[1], uv) == null) {
                lx = Double.NaN;
                continue;
            }
            double sx = cx + uv[0] * R, sy = cy - uv[1] * R;
            // A jump of more than a fifth of the disk is a cut (a fold of the projection), not a line.
            if (!Double.isNaN(lx) && Math.hypot(sx - lx, sy - ly) < 0.4 * R)
                Draw.line(g, lx, ly, sx, sy, w, col);
            lx = sx;
            ly = sy;
        }
    }

    private void renderReadout(GuiGraphics g, MapContext c) {
        Minecraft mc = Minecraft.getInstance();
        int top = vt + vh + 4;
        g.fill(vl, top, vl + vw, height - 2, PANEL);
        List<String> lines = new ArrayList<>();
        String planet = c.info() == null ? "?" : String.format(Locale.ROOT, "%s, %.1f m/block", c.info().planetName(), c.info().mPerBlockH());
        lines.add("Projection: " + c.projectionName() + "   Planet: " + planet);
        double x = mc.player.getX(), y = mc.player.getY(), z = mc.player.getZ();
        lines.addAll(c.readout(x, y, z, true));
        String f3 = PlanetDebugLine.format(c.info(), x, z, 0f);
        lines.add("Deep Time's F3 line: " + (f3 == null ? "(none)" : f3));
        if (!Double.isNaN(hoverX)) {
            double[] ll = PlanetGeometry.displayLatLon(c.projection(), hoverX, hoverZ);
            lines.add(String.format(Locale.ROOT, "Cursor: block %.0f / %.0f   %s   seam %.0f   %s", hoverX, hoverZ,
                    PlanetGeometry.formatLatLon(ll[0], ll[1]), PlanetGeometry.seamDistance(c.projection(), hoverX),
                    c.canTeleport() ? "(right-click: teleport)" : ""));
        } else {
            lines.add("Cursor: off the map");
        }
        String server = !MapContext.serverHasDevMap() ? "server has no dev map: Deep Time's projection only, no base image, players or teleport"
                : c.devInfo() == null ? "waiting for the server's dev map info" : c.devInfo().note()
                + (c.image() == null && c.devInfo().imageId() != 0 ? " (receiving)" : "")
                + (c.devInfo().canTeleport() ? "   teleport: yes (operator)" : "   teleport: no (not an operator)");
        lines.add("Server: " + server + (status.isEmpty() ? "" : "   | " + status));
        int yy = top + 3;
        for (String s : lines) {
            g.drawString(font, s, vl + 4, yy, TEXT, false);
            yy += LINE_H;
        }
    }
}
