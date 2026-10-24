// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import com.minecraftindustrialcomplex.deeptime.devmap.model.Graticule;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapLayer;
import com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometry;
import com.minecraftindustrialcomplex.deeptime.devmap.net.DevMapState;
import com.minecraftindustrialcomplex.deeptime.devmap.net.MapInfoPayload;
import com.minecraftindustrialcomplex.deeptime.devmap.net.TeleportPayload;
import com.minecraftindustrialcomplex.deeptime.net.ClientPlanetState;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import com.minecraftindustrialcomplex.deeptime.worlddata.PetroffGuyou;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientPacketListener;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * What the map shows now, read once a frame: the level, Deep Time's projection and planet info
 * for it ({@link ClientPlanetState}, the same objects the F3 line uses), the server dev map's info
 * and the picture to draw when the server has the dev map, and the player. The picture is the
 * player's choice ({@link #wanted}, the client config's {@code mapLayer}, satellite by default)
 * when it is here, else the other one ({@link MapLayer#shown}); {@link #image} is null when neither
 * is. {@link #problem} is set when there is nothing to draw.
 */
record MapContext(String dimension, WorldProjection projection, WorldInfoPayload info, MapInfoPayload devInfo,
                  MapLayer wanted, DevMapState.Image image, String problem) {

    static MapContext current() {
        Minecraft mc = Minecraft.getInstance();
        MapLayer wanted = ClientConfig.mapLayer();
        if (mc.level == null || mc.player == null)
            return new MapContext("", null, null, null, wanted, null, "not in a world");
        String dim = mc.level.dimension().location().toString();
        WorldProjection p = ClientPlanetState.projection(mc.level);
        WorldInfoPayload info = ClientPlanetState.worldInfo();
        MapInfoPayload dev = DevMapState.info(dim);
        if (p == null) {
            boolean waiting = info == null || !info.dimension().equals(dim);
            return new MapContext(dim, null, info, dev, wanted, null,
                    waiting ? "waiting for Deep Time's planet info" : "No Deep Time planet in this dimension");
        }
        // The server drew with its projection: a frame for another period is not this world's.
        boolean sameWorld = dev == null || dev.period() == (int) Math.round(p.period());
        MapLayer shown = sameWorld ? MapLayer.shown(wanted, l -> DevMapState.image(dim, l) != null) : null;
        return new MapContext(dim, p, info, dev, wanted, shown == null ? null : DevMapState.image(dim, shown), null);
    }

    /** The picture drawn, or null. */
    MapLayer shown() {
        return image == null ? null : image.layer();
    }

    /**
     * The readout's picture line: which picture is drawn, and when it is not the chosen one, what
     * became of that (still rendering, receiving, or the server's reason it has none).
     */
    String pictureLine() {
        if (devInfo == null)
            return "none (no dev map info from the server)";
        MapInfoPayload.Picture want = devInfo.picture(wanted);
        String why = want == null ? "not sent by the server"
                : DevMapState.receiving(devInfo, wanted) ? "receiving" : want.note();
        if (image != null && image.layer() == wanted)
            return wanted.label().toLowerCase(Locale.ROOT) + ": " + why;
        String shown = image == null ? "none" : image.layer().label().toLowerCase(Locale.ROOT) + " for now";
        return shown + " (" + wanted.label().toLowerCase(Locale.ROOT) + ": " + why + ")";
    }

    /** Whether the server has the dev map (its channel is open on this connection). */
    static boolean serverHasDevMap() {
        ClientPacketListener c = Minecraft.getInstance().getConnection();
        return c != null && c.hasChannel(TeleportPayload.TYPE);
    }

    boolean canTeleport() {
        return devInfo != null && devInfo.canTeleport() && serverHasDevMap();
    }

    /** The projection's name for the readout. */
    String projectionName() {
        if (projection == null)
            return "none";
        if (projection instanceof PetroffGuyou pg)
            return String.format(Locale.ROOT, "Petroff-Guyou (b %d, lambda_c %.2f°, x period %d, z %.0f..%.0f)",
                    pg.hemisphere(), pg.lambdaCDeg(), (long) pg.period(), pg.zMin(), pg.zMax());
        if (projection instanceof WorldProjection.Cube cube) {
            double[] w = cube.cube().window();
            return String.format(Locale.ROOT, "Cube (face %d, x period %d, net x %.0f..%.0f, z %.0f..%.0f)",
                    cube.side(), (long) cube.period(), w[0], w[1], w[2], w[3]);
        }
        return String.format(Locale.ROOT, "Lambert cylindrical equal-area (C %d, poles at z = ±%.1f)",
                (long) projection.period(), projection.zMax());
    }

    /** The readout lines for a point (the player's, or the cursor's with {@code prefix} "cursor"). */
    List<String> readout(double x, double y, double z, boolean withY) {
        List<String> out = new ArrayList<>();
        double[] ll = PlanetGeometry.displayLatLon(projection, x, z);
        double mPerBlock = info != null ? info.mPerBlockH() : 0;
        // x as Toroidal World's F3 XYZ line shows it (wrapped into the period), the raw x beside it
        // when the player is laps away from the origin (TW's "Unwrapped" line).
        double wx = PlanetGeometry.wrapCentred(projection, x);
        String unwrapped = Math.abs(wx - x) > 0.5 ? String.format(Locale.ROOT, " (unwrapped x %.1f)", x) : "";
        out.add(withY ? String.format(Locale.ROOT, "Block %.1f / %.1f / %.1f%s   %s", wx, y, z, unwrapped, PlanetGeometry.formatLatLon(ll[0], ll[1]))
                : String.format(Locale.ROOT, "Block %.0f / %.0f%s   %s", wx, z, unwrapped, PlanetGeometry.formatLatLon(ll[0], ll[1])));
        double seam = PlanetGeometry.seamDistance(projection, x, z);
        double edge = PlanetGeometry.edgeDistance(projection, x, z);
        boolean pg = PlanetGeometry.petroffGuyou(projection);
        if (PlanetGeometry.cube(projection) != null) {
            double w0 = PlanetGeometry.west(projection);
            out.add(String.format(Locale.ROOT, "Face %s   Slide (x = %.0f = %.0f): %.0f blocks   Border: %.0f blocks",
                    PlanetGeometry.faceName(projection, x, z), w0, w0 + projection.period(), seam, edge));
        } else {
            out.add(String.format(Locale.ROOT, "Seam (x = ±%.0f): %.0f blocks   %s: %.0f blocks", projection.period() / 2,
                    seam, pg ? "Edge (z = ±" + Math.round(projection.zMax()) + ")" : "Pole line", edge));
        }
        PlanetGeometry.NearestPole np = PlanetGeometry.nearestPole(projection, x, z);
        String km = mPerBlock > 0 ? String.format(Locale.ROOT, " = %.0f km", np.arcBlocks() * mPerBlock / 1000.0) : "";
        out.add(String.format(Locale.ROOT, "Nearest pole: %s %.0f blocks (block space); %s %.3f° of arc = %.0f blocks%s",
                np.byBlocks(), np.blocks(), np.byArc(), np.arcDeg(), np.arcBlocks(), km));
        return out;
    }

    /** The graticule of this projection (cached per projection object). */
    List<Graticule.Line> graticule() {
        return GraticuleCache.of(projection);
    }

    /** Caches the graticule: sweeping it costs some 30k projections. */
    static final class GraticuleCache {
        private static WorldProjection of;
        private static List<Graticule.Line> lines = List.of();

        static synchronized List<Graticule.Line> of(WorldProjection p) {
            if (p == null)
                return List.of();
            if (p != of) {
                lines = Graticule.lines(p, 15.0, 0.5);
                of = p;
            }
            return lines;
        }
    }
}
