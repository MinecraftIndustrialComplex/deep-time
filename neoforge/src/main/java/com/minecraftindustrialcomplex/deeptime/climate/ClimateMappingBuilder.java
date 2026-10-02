// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.terrablender.RegionCatalog;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.world.level.levelgen.DensityFunction;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import net.minecraft.world.level.levelgen.NoiseRouter;
import net.minecraft.world.level.levelgen.NoiseRouterData;
import net.minecraft.world.level.levelgen.RandomState;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.Arrays;
import java.util.Locale;

/**
 * Computes a world's {@link ClimateMapping} once, at its first start (plan §7.2, P1-09), and stores
 * it in the world data folder as {@value ClimateMapping#FILE}; later starts read the file, so a pack
 * update never shifts biomes in chunks nobody has visited yet.
 *
 * <p>The pack's side has two parts. Its <b>band edges</b> come from its copy of
 * {@code data/deeptime/deeptime/climate_mapping/default.json} (the jar ships
 * {@code OverworldBiomeBuilder}'s; a datapack may override the file). Its <b>erosion and
 * peaks-and-valleys distributions</b> come from sampling its own overworld router — the
 * {@code minecraft:overworld} noise settings as the pack's mods left them (Tectonic, Terralith) —
 * at {@value #POINTS} land points (continentalness ≥ the reference's land threshold) of a
 * jittered grid, seed 0: Genesis's trick of evaluating the frozen registries' router directly
 * ({@code minecraft_integration_architecture.md} §3). The reference side is what the bake recorded
 * in {@code notes.bake.climate.reference}.
 */
public final class ClimateMappingBuilder {

    /** Land samples of the pack's router. */
    public static final int POINTS = 65_536;
    /** Quantiles kept (u = i / (Q − 1)). */
    public static final int QUANTILES = 65;
    /** The jittered grid: GRID² candidates spread over ±SPAN blocks. */
    static final int GRID = 640;
    static final int SPAN = 60_000;
    static final String MAPPING = "p1-09";

    public static final ResourceLocation PACK_BANDS =
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "deeptime/climate_mapping/default.json");

    private ClimateMappingBuilder() {}

    /** The pack router's land distributions. */
    public record PackSample(double[] erosion, double[] peaksValleys, int points, int candidates, String settings, double landMin) {
        JsonObject toJson() {
            JsonObject o = new JsonObject();
            o.addProperty("noise_settings", settings);
            o.addProperty("seed", 0);
            o.addProperty("land_min_continents", landMin);
            o.addProperty("points", points);
            o.addProperty("candidates", candidates);
            o.add("erosion", OverworldBands.json(erosion));
            o.add("peaks_valleys", OverworldBands.json(peaksValleys));
            return o;
        }
    }

    /** {@code n} quantiles of {@code v[0..count)} at u = i / (n − 1). */
    static double[] quantiles(double[] v, int count, int n) {
        double[] s = Arrays.copyOf(v, count);
        Arrays.sort(s);
        double[] q = new double[n];
        for (int i = 0; i < n; i++) {
            double x = i / (double) (n - 1) * (count - 1);
            int a = (int) Math.floor(x), b = Math.min(count - 1, a + 1);
            q[i] = s[a] + (x - a) * (s[b] - s[a]);
        }
        return q;
    }

    private static long mix(long z) {
        z = (z ^ (z >>> 30)) * 0xBF58476D1CE4E5B9L;
        z = (z ^ (z >>> 27)) * 0x94D049BB133111EBL;
        return z ^ (z >>> 31);
    }

    /** Samples the pack's overworld router (null when it has none or it cannot be evaluated). */
    public static PackSample sample(RegistryAccess access, double landMin) {
        var holder = access.registryOrThrow(Registries.NOISE_SETTINGS).getHolder(NoiseGeneratorSettings.OVERWORLD);
        if (holder.isEmpty())
            return null;
        RandomState rs = RandomState.create(holder.get().value(), access.lookupOrThrow(Registries.NOISE), 0L);
        NoiseRouter r = rs.router();
        double[] ero = new double[POINTS], pv = new double[POINTS];
        int n = 0, tried = 0;
        double cell = 2.0 * SPAN / GRID;
        outer:
        for (int j = 0; j < GRID; j++) {
            for (int i = 0; i < GRID; i++) {
                long h = mix(((long) j << 32) ^ i ^ 0x5DEECE66DL);
                int x = (int) Math.floor(-SPAN + (i + (h & 0xFFFF) / 65536.0) * cell);
                int z = (int) Math.floor(-SPAN + (j + ((h >>> 16) & 0xFFFF) / 65536.0) * cell);
                var ctx = new DensityFunction.SinglePointContext(x, 0, z);
                tried++;
                if (r.continents().compute(ctx) < landMin)
                    continue;
                ero[n] = r.erosion().compute(ctx);
                pv[n] = NoiseRouterData.peaksAndValleys((float) r.ridges().compute(ctx));
                if (++n == POINTS)
                    break outer;
            }
        }
        if (n < 1000)
            return null;
        return new PackSample(quantiles(ero, n, QUANTILES), quantiles(pv, n, QUANTILES), n, tried,
                NoiseGeneratorSettings.OVERWORLD.location().toString(), landMin);
    }

    /** The pack's band edges: its copy of the resource, else the jar's. */
    public static OverworldBands packBands(ResourceManager resources) {
        if (resources != null) {
            var res = resources.getResource(PACK_BANDS);
            if (res.isPresent()) {
                try (InputStream in = res.get().open()) {
                    return OverworldBands.fromJson(OverworldBands.parse(in).getAsJsonObject("bands"));
                } catch (IOException | RuntimeException e) {
                    DeepTime.LOGGER.warn("Cannot read the pack's {} ({}); using the jar's bands", PACK_BANDS, e.getMessage());
                }
            }
        }
        return OverworldBands.defaults();
    }

    /** {@code notes.bake.climate} when it was written by the P1-09 mapping, else null. */
    static JsonObject bakeClimate(WorldData wd) {
        JsonObject notes = wd.manifest().notes();
        if (notes == null || !notes.has("bake") || !notes.get("bake").isJsonObject())
            return null;
        JsonObject bake = notes.getAsJsonObject("bake");
        if (!bake.has("climate") || !bake.get("climate").isJsonObject())
            return null;
        JsonObject c = bake.getAsJsonObject("climate");
        return c.has("mapping") && MAPPING.equals(c.get("mapping").getAsString()) && c.has("reference") ? c : null;
    }

    /**
     * The world's mapping: {@value ClimateMapping#FILE} when the folder has one; otherwise, for a
     * world the P1-09 bake wrote, built from the pack and written there; otherwise identity (the
     * synthetic worlds carry vanilla values directly).
     */
    public static ClimateMapping forWorld(WorldData wd, RegistryAccess access, ResourceManager resources) {
        return withRegions(tables(wd, access, resources), wd);
    }

    /** The baked count of geologic provinces ({@code notes.bake.meta.provinces.count}), or 0. */
    static int provinceCount(WorldData wd) {
        JsonObject n = wd.manifest().notes();
        for (String k : new String[]{"bake", "meta", "provinces"}) {
            if (n == null || !n.has(k) || !n.get(k).isJsonObject())
                return 0;
            n = n.getAsJsonObject(k);
        }
        return n.has("count") ? n.get("count").getAsInt() : 0;
    }

    /** The assignment's salt: the preset's seed (from the world data's {@code preset.json}), else 0. */
    static long regionSalt(WorldData wd) {
        try {
            JsonObject p = JsonParser.parseString(Files.readString(wd.folder().resolve("preset.json"))).getAsJsonObject();
            return p.has("seed") ? p.get("seed").getAsLong() * 0x9E3779B97F4A7C15L : 0L;
        } catch (IOException | RuntimeException e) {
            return 0L;
        }
    }

    /**
     * Gives the world's geologic provinces their TerraBlender regions (plan §7.3, P1-10), once: when
     * TerraBlender is installed, the world has baked provinces and its mapping has no table yet, the
     * table is dealt out over TerraBlender's registered regions by weight
     * ({@link ProvinceRegions#assign}) and written into {@value ClimateMapping#FILE}. A table that
     * exists is kept as it is, so a biome mod installed or removed later never moves the regions of
     * explored chunks.
     */
    public static ClimateMapping withRegions(ClimateMapping m, WorldData wd) {
        if (m.regions() != null || m.isIdentity())
            return m;
        int provinces = provinceCount(wd);
        if (provinces <= 0)
            return m;
        var catalog = RegionCatalog.load();
        if (catalog.isEmpty())
            return m;
        ClimateMapping out;
        try {
            out = m.withRegions(ProvinceRegions.assign(provinces, catalog.get().overworldRegions(), regionSalt(wd)));
        } catch (IllegalArgumentException e) {
            DeepTime.LOGGER.warn("TerraBlender regions of {}: {}", wd.folder(), e.getMessage());
            return m;
        }
        try {
            out.write(wd.folder().resolve(ClimateMapping.FILE));
        } catch (IOException e) {
            DeepTime.LOGGER.error("Cannot write {} ({}): the regions hold for this start only", ClimateMapping.FILE, e.getMessage());
        }
        DeepTime.LOGGER.info("TerraBlender regions of {}: {} provinces dealt out over {}", wd.folder(), provinces, out.regions().counts());
        return out;
    }

    private static ClimateMapping tables(WorldData wd, RegistryAccess access, ResourceManager resources) {
        Path file = wd.folder().resolve(ClimateMapping.FILE);
        if (Files.isRegularFile(file)) {
            try {
                ClimateMapping m = ClimateMapping.read(file);
                DeepTime.LOGGER.info("Climate mapping of {} read from {} ({})", wd.folder(), ClimateMapping.FILE, m.summary());
                return m;
            } catch (IOException | RuntimeException e) {
                DeepTime.LOGGER.error("Cannot read {} ({}); rebuilding it from the pack", file, e.getMessage());
            }
        }
        JsonObject climate = bakeClimate(wd);
        if (climate == null) {
            DeepTime.LOGGER.debug("World data {} has no P1-09 climate record: biome inputs used as baked", wd.folder());
            return ClimateMapping.IDENTITY;
        }
        long t0 = System.nanoTime();
        JsonObject ref = climate.getAsJsonObject("reference");
        OverworldBands refBands = OverworldBands.fromJson(ref.getAsJsonObject("bands"));
        JsonObject dist = ref.getAsJsonObject("reference");
        double[] refE = OverworldBands.array(dist.getAsJsonArray("erosion"));
        double[] refPv = OverworldBands.array(dist.getAsJsonArray("peaks_valleys"));
        double landMin = dist.has("land_min_continents") ? dist.get("land_min_continents").getAsDouble() : -0.19;
        OverworldBands pack = packBands(resources);
        PackSample s = null;
        String why;
        try {
            s = sample(access, landMin);
            why = s == null ? "the pack has no usable minecraft:overworld router: erosion and weirdness keep the reference's distribution"
                    : "pack bands and the pack router's land distribution";
        } catch (RuntimeException e) {
            why = "sampling the pack's router failed (" + e + "): erosion and weirdness keep the reference's distribution";
            DeepTime.LOGGER.warn("Climate mapping: {}", why);
        }
        JsonObject meta = new JsonObject();
        meta.addProperty("mapping", MAPPING);
        meta.addProperty("computed_utc", Instant.now().toString());
        meta.addProperty("why", why);
        JsonObject packJson = new JsonObject();
        packJson.add("bands", pack.toJson());
        if (s != null)
            packJson.add("router", s.toJson());
        meta.add("pack", packJson);
        meta.add("reference", ref);
        ClimateMapping m = ClimateMapping.build(refBands, refE, refPv, pack, s == null ? null : s.erosion(),
                s == null ? null : s.peaksValleys(), meta);
        try {
            m.write(file);
        } catch (IOException e) {
            DeepTime.LOGGER.error("Cannot write {} ({}): the mapping holds for this start only", file, e.getMessage());
        }
        DeepTime.LOGGER.info(String.format(Locale.ROOT, "Climate mapping of %s built in %.1f s from %s (%d land points of %d): %s",
                wd.folder(), (System.nanoTime() - t0) / 1e9, why, s == null ? 0 : s.points(), s == null ? 0 : s.candidates(),
                m.summary()));
        if (s != null)
            DeepTime.LOGGER.info("Climate mapping pack router land quantiles: erosion={} peaks_valleys={}",
                    Arrays.toString(round(s.erosion())), Arrays.toString(round(s.peaksValleys())));
        return m;
    }

    static double[] round(double[] v) {
        double[] o = new double[v.length];
        for (int i = 0; i < v.length; i++)
            o[i] = Math.round(v[i] * 10_000.0) / 10_000.0;
        return o;
    }
}
