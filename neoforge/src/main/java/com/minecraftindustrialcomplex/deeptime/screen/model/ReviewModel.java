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
            if (wd != null) {
                map = elevationMap(wd, ramps.forLayer(WorldData.ELEVATION));
            } else if (lastFrame.isPresent()) {
                KeyframeInfo f = lastFrame.get();
                int layer = Math.max(0, f.layerIndex(WorldData.ELEVATION));
                map = new MapImage(f.w(), f.h(), keyframePixels.apply(f, layer), "keyframe");
            }
            spawn = SpawnCandidates.firstDry(dir, m.seaLevelY())
                    .map(c -> GlobeProjection.latLon(c.x(), c.z(), circumference, new double[2]));
            provinces = ProvinceHints.compute(SiteRecords.read(dir));
        } catch (RuntimeException e) {
            error = e.getMessage() == null ? e.toString() : e.getMessage();
        } finally {
            ready = true;
        }
    }

    private static long count(JsonObject o, String k) {
        return o.has(k) && o.get(k).isJsonPrimitive() ? o.get(k).getAsLong() : -1;
    }

    /** Samples the elevation raster at map resolution: pixel centres as keyframes place them. */
    MapImage elevationMap(WorldData wd, Ramps.Ramp ramp) {
        int[] px = new int[MAP_W * MAP_H];
        double[] b = new double[2];
        for (int y = 0; y < MAP_H; y++) {
            double lat = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureLatitudeDeg(y, MAP_H);
            for (int x = 0; x < MAP_W; x++) {
                double lon = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureLongitudeDeg(x, MAP_W);
                GlobeProjection.block(lat, lon, circumference, b);
                px[y * MAP_W + x] = ramp.abgr(wd.height(b[0], b[1]));
            }
        }
        return new MapImage(MAP_W, MAP_H, px, "world_data");
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
