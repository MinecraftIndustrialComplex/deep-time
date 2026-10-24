// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import com.minecraftindustrialcomplex.deeptime.worlddata.SiteRecords;
import com.minecraftindustrialcomplex.deeptime.worlddata.SpawnCandidates;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

import java.nio.file.Path;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.Executor;
import java.util.function.BiFunction;

/**
 * The review page's state (plan §7.12 page 3): the final map, the spawn marker, ore-province hints
 * and a summary, computed once on a background thread from the finished staging folder.
 *
 * <ul>
 *   <li><b>Final map.</b> The world data's {@code elevation} raster (what the world will be)
 *       sampled at {@link #MAP_W} × {@link #MAP_H} and coloured with {@code ramps.json}'s
 *       elevation ramp, the colours of the keyframes and the viewer; a run whose world data has
 *       no elevation layer yet shows its last keyframe instead.</li>
 *   <li><b>Spawn marker.</b> The first spawn candidate above sea level (the server's
 *       {@code SpawnHandler} re-checks it against the generated surface).</li>
 *   <li><b>Province hints.</b> {@link ProvinceHints} over the site registry.</li>
 * </ul>
 *
 * On a cube world (cube plan G13) the final map is the net as it is played ({@link NetView}: the
 * window 4 : 3, the void transparent, the border's lines drawn by the page), and the province hints,
 * latitude and longitude cells, are painted over it as a second picture ({@link #hintsOverlay}).
 *
 * The pack's biome names over the map (plan §7.2, the climate mapping computed here and staged)
 * come with P1-09.
 */
public final class ReviewModel {

    public static final int MAP_W = 512, MAP_H = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureHeight(MAP_W);

    /** The final map. {@code source} is {@code world_data} or {@code keyframe}. */
    public record MapImage(int w, int h, int[] abgr, String source) {}

    /** Counts for the side panel. */
    public record Summary(String source, long events, long sites, long keyframes, int layers, int circumference) {}

    private final Path dir;
    private final int circumference;
    private volatile MapImage map;
    private volatile NetView net;
    private volatile MapImage hintsOverlay;
    private volatile Optional<double[]> spawn = Optional.empty();
    private volatile List<ProvinceHints.Cell> provinces = List.of();
    private volatile Summary summary;
    private volatile String error;
    private volatile boolean ready;

    /**
     * @param lastFrame      the run's last keyframe, the fallback map
     * @param keyframePixels reads a keyframe layer (the run's {@link RunHandle#pixels})
     */
    public ReviewModel(Path dir, int circumference, Optional<KeyframeInfo> lastFrame,
            BiFunction<KeyframeInfo, Integer, int[]> keyframePixels, Ramps ramps, Executor io) {
        this.dir = dir;
        this.circumference = circumference;
        io.execute(() -> compute(lastFrame, keyframePixels, ramps));
    }

    private void compute(Optional<KeyframeInfo> lastFrame, BiFunction<KeyframeInfo, Integer, int[]> keyframePixels, Ramps ramps) {
        try {
            Manifest m = Manifest.read(dir);
            JsonObject run = m.notes().has("run") && m.notes().get("run").isJsonObject() ? m.notes().getAsJsonObject("run") : new JsonObject();
            summary = new Summary(m.source(), count(run, "events"), count(run, "sites"), count(run, "keyframes"), m.layers().size(),
                    m.circumference());
            WorldData wd = null;
            if (m.layer(WorldData.ELEVATION) != null)
                wd = WorldData.open(dir);
            com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection wp = m.projection();
            NetView nv = wp instanceof com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Cube c ? new NetView(c) : null;
            if (wd != null && nv != null) {
                map = netElevationMap(nv, wd, ramps.forLayer(WorldData.ELEVATION));
            } else if (wd != null) {
                map = elevationMap(wd, ramps.forLayer(WorldData.ELEVATION));
            } else if (lastFrame.isPresent()) {
                KeyframeInfo f = lastFrame.get();
                int layer = Math.max(0, f.layerIndex(WorldData.ELEVATION));
                map = new MapImage(f.w(), f.h(), keyframePixels.apply(f, layer), "keyframe");
            }
            spawn = SpawnCandidates.firstDry(dir, m.seaLevelY())
                    .map(c -> wp.latLon(c.x(), c.z()));
            provinces = ProvinceHints.compute(SiteRecords.read(dir));
            if (nv != null && map != null && "world_data".equals(map.source())) {
                hintsOverlay = hintsOverlay(nv, provinces, map.w(), map.h());
                net = nv;
            }
        } catch (RuntimeException e) {
            error = e.getMessage() == null ? e.toString() : e.getMessage();
        } finally {
            ready = true;
        }
    }

    private static long count(JsonObject o, String k) {
        return o.has(k) && o.get(k).isJsonPrimitive() ? o.get(k).getAsLong() : -1;
    }

    /**
     * Samples the elevation raster at map resolution: pixel centres as keyframes place them (a Lambert
     * picture of latitude and longitude), each at its block in the world's projection.
     */
    MapImage elevationMap(WorldData wd, Ramps.Ramp ramp) {
        int[] px = new int[MAP_W * MAP_H];
        com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection wp = wd.projection();
        for (int y = 0; y < MAP_H; y++) {
            double lat = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureLatitudeDeg(y, MAP_H);
            for (int x = 0; x < MAP_W; x++) {
                double lon = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureLongitudeDeg(x, MAP_W);
                double[] b = wp.blockOf(lat, lon);
                px[y * MAP_W + x] = ramp.abgr(wd.height(b[0], b[1]));
            }
        }
        return new MapImage(MAP_W, MAP_H, px, "world_data");
    }

    /** The net picture of a cube world's elevation, {@link NetView#W} × {@link NetView#H}. */
    static MapImage netElevationMap(NetView nv, WorldData wd, Ramps.Ramp ramp) {
        return new MapImage(NetView.W, NetView.H, nv.picture(NetView.W, NetView.H, (x, z) -> ramp.abgr(wd.height(x, z))), "world_data");
    }

    /**
     * The province hints over the net: each pixel on a face in a hinted cell takes the cell's type
     * colour at the alpha the Lambert map's boxes use (0x50), ABGR; the rest transparent.
     */
    static MapImage hintsOverlay(NetView nv, List<ProvinceHints.Cell> cells, int w, int h) {
        java.util.Map<Integer, Integer> byCell = new java.util.HashMap<>();
        for (ProvinceHints.Cell c : cells) {
            int rgb = ProvinceHints.colour(c.type());
            // ARGB 0x50RRGGBB as ABGR.
            int abgr = 0x50 << 24 | (rgb & 0xff) << 16 | (rgb & 0xff00) | (rgb >> 16 & 0xff);
            byCell.put(c.row() * ProvinceHints.COLS + c.col(), abgr);
        }
        int[] px = byCell.isEmpty() ? new int[w * h] : nv.picture(w, h, (x, z) -> {
            double[] ll = nv.projection().latLon(x, z);
            if (Double.isNaN(ll[0]))
                return 0;
            int col = Math.floorMod((int) Math.floor(ll[1] / (360.0 / ProvinceHints.COLS)), ProvinceHints.COLS);
            int row = Math.max(0, Math.min(ProvinceHints.ROWS - 1, (int) Math.floor((90.0 - ll[0]) / (180.0 / ProvinceHints.ROWS))));
            return byCell.getOrDefault(row * ProvinceHints.COLS + col, 0);
        });
        return new MapImage(w, h, px, "hints");
    }

    /** A cube world's net view (when its final map is the net), else empty. */
    public Optional<NetView> net() {
        return Optional.ofNullable(net);
    }

    /** A cube world's province hints as a picture over the net (see {@link #net}), else empty. */
    public Optional<MapImage> hintsOverlay() {
        return Optional.ofNullable(hintsOverlay);
    }

    public Path dir() {
        return dir;
    }

    public boolean ready() {
        return ready;
    }

    public Optional<MapImage> map() {
        return Optional.ofNullable(map);
    }

    /** The spawn as (lat, lon). */
    public Optional<double[]> spawn() {
        return spawn;
    }

    public List<ProvinceHints.Cell> provinces() {
        return provinces;
    }

    public Optional<Summary> summary() {
        return Optional.ofNullable(summary);
    }

    public Optional<String> error() {
        return Optional.ofNullable(error);
    }
}
