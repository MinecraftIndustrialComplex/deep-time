// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometry;
import com.minecraftindustrialcomplex.deeptime.net.PlanetDebugLine;
import com.minecraftindustrialcomplex.deeptime.screen.model.GlobeProjection;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.mojang.blaze3d.platform.NativeImage;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;
import net.neoforged.neoforge.client.event.ClientTickEvent;
import net.neoforged.neoforge.common.NeoForge;

import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * Development only: with {@code -Ddeeptime_devmap.shots=true} the client, once in a world, walks
 * the dev map on its own and quits ({@code tools/client/devmap-shots.sh}). For each place (the
 * player's own spot, beside the wrap seam, beside the north pole; {@code
 * -Ddeeptime_devmap.shots.places=name:x:z;...} overrides) it right-clicks the place on the map
 * (the operator teleport), waits for the terrain, checks the readout against Deep Time's own F3
 * line and the markers' screen positions against the projection's inverse, and takes the HUD
 * (with F3), the map and the globe. Every step logs a {@code [devmap-shot]} line; a failed check
 * logs {@code FAIL}. Screenshots go to {@code <gamedir>/screenshots/devmap-*.png}.
 */
final class ShotDriver {

    private static final String PROP = "deeptime_devmap.shots";
    private static final int TIMEOUT = 20 * 240;

    private enum Step { WAIT_WORLD, WAIT_IMAGE, GO, ARRIVE, SETTLE, HUD, MAP, GLOBE, NEXT, QUIT }

    private record Place(String name, double x, double z) {}

    private static Step step = Step.WAIT_WORLD;
    private static int stepTicks, ticks, place, shotNo, fails;
    private static List<Place> places;
    private static DevMapScreen screen;

    private ShotDriver() {}

    static void init() {
        if (!Boolean.getBoolean(PROP))
            return;
        DevMap.LOGGER.warn("[devmap-shot] dev map shot driver active (-D{}=true): the client drives itself", PROP);
        NeoForge.EVENT_BUS.addListener(ClientTickEvent.Post.class, e -> tick());
    }

    private static void go(Step next) {
        DevMap.LOGGER.info("[devmap-shot] step {} -> {} (tick {})", step, next, ticks);
        step = next;
        stepTicks = 0;
    }

    private static void log(String fmt, Object... args) {
        DevMap.LOGGER.info("[devmap-shot] " + fmt, args);
    }

    private static void fail(String fmt, Object... args) {
        fails++;
        DevMap.LOGGER.error("[devmap-shot] FAIL " + fmt, args);
    }

    private static void tick() {
        Minecraft mc = Minecraft.getInstance();
        ticks++;
        stepTicks++;
        if (stepTicks > TIMEOUT && step != Step.QUIT) {
            fail("step {} timed out", step);
            shot("timeout-" + step.name().toLowerCase(Locale.ROOT));
            go(Step.QUIT);
            return;
        }
        try {
            run(mc);
        } catch (RuntimeException e) {
            DevMap.LOGGER.error("[devmap-shot] FAIL step {} threw", step, e);
            fails++;
            go(Step.QUIT);
        }
    }

    private static void run(Minecraft mc) {
        MapContext c = MapContext.current();
        switch (step) {
            case WAIT_WORLD -> {
                if (mc.player != null && mc.level != null && mc.screen == null && c.projection() != null && stepTicks > 100) {
                    places = places(c.projection());
                    log("in world {} at {} {} {}; projection {}; places {}", c.dimension(), mc.player.getX(), mc.player.getY(),
                            mc.player.getZ(), c.projectionName(), places);
                    go(Step.WAIT_IMAGE);
                }
            }
            case WAIT_IMAGE -> {
                boolean has = MapContext.serverHasDevMap();
                boolean noImage = c.devInfo() != null && c.devInfo().imageId() == 0 && !c.devInfo().note().contains("rendering");
                boolean teleport = c.canTeleport();
                if ((c.image() != null || noImage || !has) && (teleport || stepTicks > 20 * 60)) {
                    log("server dev map {}, info {}, image {}, teleport {}", has, c.devInfo() == null ? "none" : c.devInfo().note(),
                            c.image() == null ? "none" : c.image().frame(), teleport);
                    if (!teleport)
                        fail("no teleport permission: the places cannot be visited, only the spawn is shot");
                    go(Step.GO);
                }
            }
            case GO -> {
                Place pl = places.get(place);
                if (stepTicks == 1) {
                    mc.options.hideGui = false;
                    if (mc.getDebugOverlay().showDebugScreen())
                        mc.getDebugOverlay().toggleOverlay();
                    DevMapClient.hud(false);
                    screen = new DevMapScreen();
                    mc.setScreen(screen);
                    screen.setMode(DevMapScreen.Mode.MAP);
                    screen.zoomBlocks(1024);
                    screen.centreOn(pl.x, pl.z);
                }
                if (stepTicks == 5) {
                    double[] s = screen.mapScreenPos(pl.x, pl.z);
                    double[] b = screen.blockAt(s[0], s[1]);
                    log("place {}: right-click on the map at screen {} {} = block {} (asked {} {})", pl.name, s[0], s[1],
                            b == null ? "none" : String.format(Locale.ROOT, "%.2f %.2f", b[0], b[1]), pl.x, pl.z);
                    if (!c.canTeleport()) {
                        go(Step.SETTLE);
                        return;
                    }
                    screen.mouseClicked(s[0], s[1], 1);
                    go(Step.ARRIVE);
                }
            }
            case ARRIVE -> {
                Place pl = places.get(place);
                double dx = c.projection().wrapX(mc.player.getX() - pl.x), dz = mc.player.getZ() - pl.z;
                if (Math.hypot(dx, dz) < 8) {
                    log("place {}: arrived at {} {} {} ({} blocks from the asked point; the server keeps a margin from the border)",
                            pl.name, mc.player.getX(), mc.player.getY(), mc.player.getZ(), Math.hypot(dx, dz));
                    mc.setScreen(null);
                    go(Step.SETTLE);
                } else if (stepTicks > 20 * 60) {
                    fail("place {}: the player did not arrive (at {} {}, asked {} {})", pl.name, mc.player.getX(), mc.player.getZ(), pl.x, pl.z);
                    mc.setScreen(null);
                    go(Step.SETTLE);
                }
            }
            case SETTLE -> {
                if (mc.screen != null)
                    mc.setScreen(null);
                // Time for the server to generate and the software renderer to draw the terrain around.
                if (stepTicks == 20 * 25) {
                    checkReadout(c, mc);
                    go(Step.HUD);
                }
            }
            case HUD -> {
                if (stepTicks == 1) {
                    if (!mc.getDebugOverlay().showDebugScreen())
                        mc.getDebugOverlay().toggleOverlay();
                    DevMapClient.hud(true);
                }
                if (stepTicks == 10) {
                    shot(placeName() + "-hud");
                    mc.getDebugOverlay().toggleOverlay();
                    go(Step.MAP);
                }
            }
            case MAP -> {
                if (stepTicks == 1) {
                    screen = new DevMapScreen();
                    mc.setScreen(screen);
                    screen.setMode(DevMapScreen.Mode.MAP);
                    if (place == 0) {
                        screen.showWholeWorld();
                        screen.centreOn(mc.player.getX(), 0.5 * (c.projection().zMin() + c.projection().zMax()));
                    }
                }
                if (stepTicks == 10 && place == 0)
                    shot(placeName() + "-map-whole");
                if (stepTicks == 12) {
                    screen.zoomBlocks(1600);
                    screen.centreOnPlayer();
                    checkMapMarker(c, mc);
                }
                if (stepTicks == 20) {
                    shot(placeName() + "-map");
                    go(Step.GLOBE);
                }
            }
            case GLOBE -> {
                if (stepTicks == 1) {
                    screen.setMode(DevMapScreen.Mode.GLOBE);
                    screen.centreOnPlayer();
                }
                boolean ready = c.image() == null || MapTextures.picture(c.image(), c.projection()) != null;
                if (stepTicks > 2 && ready && stepTicks < 1000) {
                    checkGlobeMarker(c, mc);
                    stepTicks = 1000;
                }
                if (stepTicks == 1010) {
                    shot(placeName() + "-globe");
                    go(Step.NEXT);
                }
            }
            case NEXT -> {
                mc.setScreen(null);
                DevMapClient.hud(false);
                place++;
                go(place < places.size() && (place == 0 || c.canTeleport()) ? Step.GO : Step.QUIT);
            }
            case QUIT -> {
                if (stepTicks == 20) {
                    log("done after {} ticks: {} shots, {} failed checks", ticks, shotNo, fails);
                    mc.stop();
                }
            }
        }
    }

    private static String placeName() {
        return String.format(Locale.ROOT, "%d-%s", place + 1, places.get(place).name);
    }

    /** The places: the player's own spot, beside the seam, beside the north pole (or the property's list). */
    private static List<Place> places(WorldProjection p) {
        List<Place> out = new ArrayList<>();
        String prop = System.getProperty(PROP + ".places", "");
        if (!prop.isBlank()) {
            for (String s : prop.split(";")) {
                String[] f = s.trim().split(":");
                if (f.length == 3)
                    out.add(new Place(f[0], Double.parseDouble(f[1]), Double.parseDouble(f[2])));
            }
            return out;
        }
        Minecraft mc = Minecraft.getInstance();
        out.add(new Place("spawn", Math.floor(mc.player.getX()) + 0.5, Math.floor(mc.player.getZ()) + 0.5));
        out.add(new Place("seam", p.period() / 2 - 25.5, 0.5 * (p.zMin() + p.zMax()) + 0.5));
        PlanetGeometry.Pole north = PlanetGeometry.poles(p).stream().filter(q -> q.latDeg() > 0).findFirst().orElseThrow();
        double[] n = north.images().get(0);
        double inward = n[1] <= 0.5 * (p.zMin() + p.zMax()) ? 60 : -60;
        out.add(new Place("north-pole", north.line() ? 0.5 : n[0] + 0.5, n[1] + inward + 0.5));
        return out;
    }

    /** The readout against Deep Time's own F3 line, and the distances, logged. */
    private static void checkReadout(MapContext c, Minecraft mc) {
        double x = mc.player.getX(), z = mc.player.getZ();
        String f3 = PlanetDebugLine.format(c.info(), x, z, 0f);
        double[] ll = PlanetGeometry.displayLatLon(c.projection(), x, z);
        String mine = PlanetGeometry.formatLatLon(ll[0], ll[1]);
        log("{}: F3 line '{}'", placeName(), f3);
        for (String line : c.readout(x, mc.player.getY(), z, true))
            log("{}: readout {}", placeName(), line);
        if (f3 == null || !f3.contains(mine))
            fail("{}: the readout's latitude/longitude '{}' is not in Deep Time's F3 line '{}'", placeName(), mine, f3);
        else
            log("{}: readout agrees with F3 ({})", placeName(), mine);
    }

    /** The player's marker on the map, read back through the screen's own inverse, must be the player's block. */
    private static void checkMapMarker(MapContext c, Minecraft mc) {
        double x = mc.player.getX(), z = mc.player.getZ();
        double[] s = screen.mapScreenPos(x, z);
        double[] b = screen.blockAt(s[0], s[1]);
        double[] want = c.projection().canonical(x, z);
        double d = b == null ? Double.NaN : Math.hypot(c.projection().wrapX(b[0] - want[0]), b[1] - want[1]);
        log("{}: map marker at screen {} {} reads back block {} (player {} {}): {} blocks off", placeName(), s[0], s[1],
                b == null ? "none" : String.format(Locale.ROOT, "%.3f %.3f", b[0], b[1]), want[0], want[1], d);
        if (!(d < 0.01))
            fail("{}: the map marker reads back {} blocks from the player", placeName(), d);
    }

    /**
     * The player's marker on the globe, centred and turned 35° away, read back through the
     * globe's inverse and the projection's {@code blockOf}: must be the player's block.
     */
    private static void checkGlobeMarker(MapContext c, Minecraft mc) {
        WorldProjection p = c.projection();
        double x = mc.player.getX(), z = mc.player.getZ();
        double[] ll = p.latLon(x, z);
        double[] want = p.canonical(x, z);
        for (double turn : new double[]{0, 35}) {
            double lat = ll[0] - Math.copySign(turn * 0.5, ll[0]), lon = ll[1] + turn;
            screen.globeCentre(lat, lon);
            double[] g = screen.globe();
            double[] h = DevMapScreen.globeHeading(p, new GlobeProjection(g[0], g[1]), x, z, mc.player.getYRot());
            if (h == null) {
                fail("{}: the player is on the globe's far side (turn {})", placeName(), turn);
                continue;
            }
            double sx = g[3] + h[0] * g[2], sy = g[4] - h[1] * g[2];
            double[] b = screen.blockAt(sx, sy);
            double d = b == null ? Double.NaN : Math.hypot(p.wrapX(b[0] - want[0]), b[1] - want[1]);
            // Mirror images on the Petroff-Guyou edge are the same point.
            if (b != null && d > 0.5 && PlanetGeometry.petroffGuyou(p))
                d = Math.min(d, Math.hypot(p.wrapX(b[0] + want[0]), b[1] - want[1]));
            log("{}: globe marker (turned {}°) at screen {} {} reads back block {} (player {} {}): {} blocks off",
                    placeName(), turn, sx, sy, b == null ? "none" : String.format(Locale.ROOT, "%.3f %.3f", b[0], b[1]),
                    want[0], want[1], d);
            if (!(d < 0.5))
                fail("{}: the globe marker (turned {}°) reads back {} blocks from the player", placeName(), turn, d);
        }
        screen.centreOnPlayer();
    }

    /**
     * Reads the framebuffer and writes the PNG with ImageIO (Deep Time's ScreenshotDriver does the
     * same: the game's own writer failed under Xvfb).
     */
    private static void shot(String name) {
        Minecraft mc = Minecraft.getInstance();
        shotNo++;
        try (NativeImage img = Screenshot.takeScreenshot(mc.getMainRenderTarget())) {
            int w = img.getWidth(), h = img.getHeight();
            BufferedImage out = new BufferedImage(w, h, BufferedImage.TYPE_INT_RGB);
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++) {
                    int abgr = img.getPixelRGBA(x, y);
                    out.setRGB(x, y, (abgr & 0xff) << 16 | (abgr & 0xff00) | (abgr >> 16) & 0xff);
                }
            Path dir = mc.gameDirectory.toPath().resolve("screenshots");
            Files.createDirectories(dir);
            Path file = dir.resolve("devmap-" + name + ".png");
            ImageIO.write(out, "png", file.toFile());
            log("shot {} -> {} ({}x{})", name, file, w, h);
        } catch (IOException | RuntimeException e) {
            fails++;
            DevMap.LOGGER.error("[devmap-shot] FAIL screenshot {}", name, e);
        }
    }
}
