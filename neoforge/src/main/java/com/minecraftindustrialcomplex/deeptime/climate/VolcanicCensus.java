// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.Geothermal;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Where a Deep Time level's volcanic biomes lie (decisions log 2026-10-03; {@link VolcanicBiomes}), both as
 * the level places them now and as the plain climate search would (master's behaviour before the change:
 * wherever their parameters fall), on the same grid of surface columns as {@link BiomeCensus}. Used by
 * {@code /deeptime probe biomes} (its {@code volcanic} section) and the GameTests in {@code VolcanicBiomeGameTests}.
 *
 * <ul>
 *   <li>{@code columns}: land columns sampled; {@code volcanic}: of them, in {@code #deeptime:volcanic};
 *       {@code far}: of those, more than {@link #FAR} blocks from every active volcano's summit and every
 *       hot-spring pool; {@code outside}: more than {@link #OUTSIDE} blocks outside every surface zone.</li>
 *   <li>{@code pools}: hot-spring pools on land; {@code pools_by_playa}: of them, within
 *       {@link VolcanicBiomes#PLAYA_MARGIN} of a playa, where no surface role holds; {@code pools_in_role}: of the
 *       others, in the role's biome.</li>
 *   <li>{@code replaced}: columns whose plain biome is volcanic and whose biome now is not, by the biome now
 *       (what fills the place of a kept-out biome); {@code holes}: of them, an ocean, river or cave biome.</li>
 *   <li>{@code columns_changed}: columns whose biome differs from the plain search's (0 without volcanic biomes).</li>
 *   <li>{@code specks}: surface biome cells (4 × 4 blocks) in a volcanic biome none of whose four neighbours
 *       has the same biome, over every surface zone's box.</li>
 * </ul>
 */
public final class VolcanicCensus {

    private VolcanicCensus() {}

    /** "Far from a volcano": more than this many blocks from every summit and hot-spring pool. */
    public static final int FAR = 256; // ledger: BIO-114 (game-choice)
    /** "Outside the zones": more than this many blocks outside every surface zone (biome blending's slack and more). */
    public static final int OUTSIDE = 8; // ledger: BIO-114 (game-choice)

    /** One placement's counts. */
    public static final class Side {
        public int columns, volcanic, far, outside, poolsInRole, volcanicCells, specks, onGeologicGround;
        public final Map<String, Integer> byBiome = new HashMap<>();
        public final StringBuilder examples = new StringBuilder();

        JsonObject json() {
            JsonObject o = new JsonObject();
            o.addProperty("columns", columns);
            o.addProperty("volcanic", volcanic);
            o.addProperty("volcanic_share", columns == 0 ? null : round(volcanic / (double) columns, 5));
            o.addProperty("far", far);
            o.addProperty("far_share_of_volcanic", volcanic == 0 ? null : round(far / (double) volcanic, 4));
            o.addProperty("outside", outside);
            o.addProperty("outside_share_of_volcanic", volcanic == 0 ? null : round(outside / (double) volcanic, 4));
            o.addProperty("pools_in_role", poolsInRole);
            o.addProperty("volcanic_cells", volcanicCells);
            o.addProperty("specks", specks);
            o.addProperty("volcanic_on_their_geologic_ground", onGeologicGround);
            JsonObject b = new JsonObject();
            byBiome.entrySet().stream().sorted(Map.Entry.<String, Integer>comparingByValue(Comparator.reverseOrder()))
                    .forEach(e -> b.addProperty(e.getKey(), e.getValue()));
            o.add("by_biome", b);
            if (!examples.isEmpty())
                o.addProperty("far_examples", examples.toString().trim());
            return o;
        }
    }

    /** The census of one level. */
    public record Result(Side now, Side plain, int pools, int poolsByPlaya, int differ, Map<String, Integer> replaced, int holes,
                         Map<String, int[]> zones, Map<String, String> roles, JsonObject json) {}

    private static double round(double v, int places) {
        double f = Math.pow(10, places);
        return Math.round(v * f) / f;
    }

    static TagKey<Biome> c(String path) {
        return TagKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath("c", path));
    }

    private static String name(Holder<Biome> b) {
        return b.unwrapKey().map(k -> k.location().toString()).orElse("?");
    }

    /** Samples every {@code step} blocks inside the pole edge less {@code poleMargin}. */
    public static Result run(ServerLevel level, int step, int poleMargin) {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !(g.terrain() instanceof WorldDataTerrain t))
            throw new IllegalStateException(level.dimension().location() + " is not a Deep Time level on world data");
        Climate.Sampler sampler = level.getChunkSource().randomState().sampler();
        if (!(sampler.spawnTarget() instanceof RegionSpawnTarget marker))
            throw new IllegalStateException(level.dimension().location() + "'s sampler does not carry the level's spawn target");
        var biomes = g.getBiomeSource();
        var settings = g.generatorSettings().value();
        int minY = settings.noiseSettings().minY(), sea = settings.seaLevel();
        VolcanicZones zones = marker.volcanic().zones();
        Geothermal geo = t.geothermal(g.waterReach(), minY, sea);
        List<Volcanoes.Volcano> volcanoes = t.data().volcanoes().list();
        int c = t.circumference();
        List<int[]> sources = new ArrayList<>();
        for (Volcanoes.Volcano v : volcanoes)
            sources.add(new int[]{(int) Math.round(v.x()), (int) Math.round(v.z())});
        List<Geothermal.Site> pools = geo.sites().stream().filter(s -> s.kind() == Geothermal.Kind.POOL).toList();
        for (Geothermal.Site s : pools)
            sources.add(new int[]{s.x(), s.z()});

        Side now = new Side(), plain = new Side();
        Map<String, Integer> replaced = new HashMap<>();
        int holes = 0, differ = 0;
        int zp = (int) t.zHalf();   // the poles (Lambert) or the turning edges (Petroff–Guyou)
        TagKey<Biome> ocean = c("is_ocean"), river = c("is_river"), cave = c("is_cave"), underground = c("is_underground");
        for (int z = -zp + step / 2; z < zp; z += step) {
            if (Math.abs(z) > zp - poleMargin)
                continue;
            for (int x = -c / 2 + step / 2; x < c / 2; x += step) {
                double y = t.height(x, z);
                if (y <= sea)
                    continue;
                int qx = QuartPos.fromBlock(x), qy = QuartPos.fromBlock((int) Math.floor(y)), qz = QuartPos.fromBlock(z);
                Holder<Biome> b = biomes.getNoiseBiome(qx, qy, qz, sampler);
                String why = VolcanicBiomes.lastDecision();
                Holder<Biome> p = VolcanicBiomes.plain(() -> biomes.getNoiseBiome(qx, qy, qz, sampler));
                double near = nearest(sources, x, z, c), out = zones.outsideSurfaceZones(x, z);
                // A volcanic biome that is also rock-named (Terralith's Basalt Cliffs) may stand on its own ground away
                // from volcanoes (the geology gate's rule put it there, decisions log 2026-10-04): not counted as volcanic there.
                if (b.is(VolcanicBiomes.KEEP_OUT) && GeologyBiomes.byGroundRule(why, marker.volcanic().rules())) {
                    now.columns++;
                    now.onGeologicGround++;
                } else {
                    tally(now, b, near, out, x, z);
                }
                tally(plain, p, near, out, x, z);
                if (b != p)
                    differ++;
                if (p.is(VolcanicBiomes.KEEP_OUT) && !b.is(VolcanicBiomes.KEEP_OUT)) {
                    replaced.merge(name(b), 1, Integer::sum);
                    if (b.is(ocean) || b.is(river) || b.is(cave) || b.is(underground))
                        holes++;
                }
            }
        }
        // The pools' own biome; a pool by a playa takes the flat's (no surface role within PLAYA_MARGIN of one: BIO-165).
        int byPlaya = 0;
        for (Geothermal.Site s : pools) {
            int qx = QuartPos.fromBlock(s.x()), qy = QuartPos.fromBlock(s.groundY()), qz = QuartPos.fromBlock(s.z());
            if (marker.volcanic().byPlaya(QuartPos.toBlock(qx) + 2, QuartPos.toBlock(qz) + 2)) {
                byPlaya++;
                continue;
            }
            Holder<Biome> b = biomes.getNoiseBiome(qx, qy, qz, sampler);
            Holder<Biome> p = VolcanicBiomes.plain(() -> biomes.getNoiseBiome(qx, qy, qz, sampler));
            if (b.is(VolcanicBiomes.tag(VolcanicZones.Role.HOT_SPRING_FIELD)))
                now.poolsInRole++;
            if (p.is(VolcanicBiomes.tag(VolcanicZones.Role.HOT_SPRING_FIELD)))
                plain.poolsInRole++;
        }
        specks(zones, t, biomes, sampler, now, plain);

        Map<String, int[]> byRole = new java.util.LinkedHashMap<>();
        for (VolcanicZones.Zone zn : zones.zones()) {
            int[] n = byRole.computeIfAbsent(zn.role().id, k -> new int[2]);
            n[0]++;
            n[1] += (int) Math.round(zn.area());
        }
        Map<String, String> roles = new java.util.LinkedHashMap<>();
        double mH = t.data().manifest().mPerBlockH();
        JsonObject o = new JsonObject();
        o.addProperty("level", level.dimension().location().toString());
        o.addProperty("step", step);
        o.addProperty("far_blocks", FAR);
        o.addProperty("outside_blocks", OUTSIDE);
        o.addProperty("active_volcanoes", volcanoes.size());
        o.addProperty("pools", pools.size());
        o.addProperty("pools_by_playa", byPlaya);
        o.addProperty("reach", t.volcanicBiomeRules().reach());
        JsonObject zj = new JsonObject();
        for (var e : byRole.entrySet()) {
            JsonObject z = new JsonObject();
            z.addProperty("zones", e.getValue()[0]);
            z.addProperty("disc_area_blocks", e.getValue()[1]);
            z.addProperty("mean_disc_km2", round(e.getValue()[1] / (double) e.getValue()[0] * mH * mH / 1e6, 1));
            zj.add(e.getKey(), z);
        }
        o.add("zones", zj);
        RegionSpawnTarget rt = marker;
        JsonObject rj = new JsonObject();
        for (VolcanicZones.Role role : VolcanicZones.Role.values()) {
            String id = rt.volcanic().roleBiome(role).map(VolcanicCensus::name).orElse("-");
            roles.put(role.id, id);
            rj.addProperty(role.id, id);
        }
        o.add("role_biomes", rj);
        o.add("now", now.json());
        o.add("plain", plain.json());
        JsonObject rep = new JsonObject();
        replaced.entrySet().stream().sorted(Map.Entry.<String, Integer>comparingByValue(Comparator.reverseOrder())).limit(12)
                .forEach(e -> rep.addProperty(e.getKey(), e.getValue()));
        o.add("replaced_by", rep);
        o.addProperty("replaced_holes", holes);
        o.addProperty("columns_changed", differ);
        o.addProperty("active", marker.volcanic().active());
        return new Result(now, plain, pools.size(), byPlaya, differ, replaced, holes, byRole, roles, o);
    }

    private static void tally(Side side, Holder<Biome> b, double near, double out, int x, int z) {
        side.columns++;
        if (!b.is(VolcanicBiomes.KEEP_OUT))
            return;
        side.volcanic++;
        side.byBiome.merge(name(b), 1, Integer::sum);
        if (near > FAR) {
            side.far++;
            if (side.examples.length() < 400)
                side.examples.append(' ').append(name(b)).append('@').append(x).append(',').append(z);
        }
        if (out > OUTSIDE)
            side.outside++;
    }

    private static double nearest(List<int[]> sources, int x, int z, int c) {
        double best = Double.POSITIVE_INFINITY;
        for (int[] s : sources) {
            int dx = c <= 0 ? x - s[0] : Math.floorMod(x - s[0] + c / 2, c) - c / 2;
            best = Math.min(best, Math.hypot(dx, z - s[1]));
        }
        return best;
    }

    /** Isolated volcanic biome cells at the surface over every surface zone's box, both placements. */
    private static void specks(VolcanicZones zones, WorldDataTerrain t, net.minecraft.world.level.biome.BiomeSource biomes,
                               Climate.Sampler sampler, Side now, Side plain) {
        Map<Long, Holder<Biome>[]> cells = new HashMap<>();
        java.util.function.LongFunction<Holder<Biome>[]> at = key -> cells.computeIfAbsent(key, k -> {
            long kk = k;
            int qx = (int) (kk >> 32), qz = (int) kk;
            int x = QuartPos.toBlock(qx) + 2, z = QuartPos.toBlock(qz) + 2;
            int qy = QuartPos.fromBlock((int) Math.floor(t.height(x, z)));
            @SuppressWarnings("unchecked")
            Holder<Biome>[] pair = new Holder[]{biomes.getNoiseBiome(qx, qy, qz, sampler),
                    VolcanicBiomes.plain(() -> biomes.getNoiseBiome(qx, qy, qz, sampler))};
            return pair;
        });
        java.util.Set<Long> seen = new java.util.HashSet<>();
        for (VolcanicZones.Zone zn : zones.zones()) {
            if (zn.role().cave)
                continue;
            int r = (int) Math.ceil(zn.radius()) + 8;
            for (int qx = QuartPos.fromBlock(zn.x() - r); qx <= QuartPos.fromBlock(zn.x() + r); qx++)
                for (int qz = QuartPos.fromBlock(zn.z() - r); qz <= QuartPos.fromBlock(zn.z() + r); qz++) {
                    long k = key(qx, qz);
                    if (!seen.add(k))
                        continue;
                    Holder<Biome>[] here = at.apply(k);
                    for (int side = 0; side < 2; side++) {
                        Holder<Biome> b = here[side];
                        if (!b.is(VolcanicBiomes.KEEP_OUT))
                            continue;
                        Side s = side == 0 ? now : plain;
                        s.volcanicCells++;
                        boolean alone = true;
                        for (int[] d : new int[][]{{1, 0}, {-1, 0}, {0, 1}, {0, -1}})
                            if (at.apply(key(qx + d[0], qz + d[1]))[side] == b) {
                                alone = false;
                                break;
                            }
                        if (alone)
                            s.specks++;
                    }
                }
        }
    }

    private static long key(int qx, int qz) {
        return ((long) qx << 32) | (qz & 0xFFFFFFFFL);
    }
}
