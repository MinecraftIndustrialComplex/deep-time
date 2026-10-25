// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;

import java.util.Comparator;
import java.util.HashMap;
import java.util.Map;

/**
 * Where a Deep Time level's rock-named biomes lie (the geology gate, decisions log 2026-10-04; {@link GeologyBiomes}),
 * as the level places them now and as the plain climate search would (no gate, no volcanic layer), on
 * {@link BiomeCensus}'s grid of land columns. Used by {@code /deeptime probe biomes} (its {@code geology} section) and
 * {@code GeologyBiomeGameTests}.
 *
 * <ul>
 *   <li>{@code columns}: land columns sampled; {@code rock_named}: of them, in a geology role (the level's gated
 *       biomes); {@code off_ground}: of those, on ground that has none of the biome's roles.</li>
 *   <li>{@code by_biome}: rock-named columns by biome, each with how many are off their ground.</li>
 *   <li>{@code replaced}: columns whose plain biome is rock-named and off its ground, by the biome now;
 *       {@code holes}: of them, an ocean, river or cave biome (land turned to water by the gate).</li>
 *   <li>{@code ground}: land columns with each role's ground.</li>
 *   <li>{@code columns_changed}: columns whose biome differs from the plain search's (every Deep Time biome layer
 *       off); {@code columns_changed_by_rules}: of them, decided by an ordinary biome rule (the snow line, peatland,
 *       the gate's own rules).</li>
 * </ul>
 */
public final class GeologyCensus {

    private GeologyCensus() {}

    /** One placement's counts. */
    public static final class Side {
        public int columns, rockNamed, offGround;
        public final Map<String, int[]> byBiome = new HashMap<>();
        public final StringBuilder examples = new StringBuilder();

        JsonObject json() {
            JsonObject o = new JsonObject();
            o.addProperty("columns", columns);
            o.addProperty("rock_named", rockNamed);
            o.addProperty("off_ground", offGround);
            JsonObject b = new JsonObject();
            byBiome.entrySet().stream().sorted(Comparator.comparingInt((Map.Entry<String, int[]> e) -> -e.getValue()[0]))
                    .forEach(e -> {
                        JsonObject x = new JsonObject();
                        x.addProperty("columns", e.getValue()[0]);
                        x.addProperty("off_ground", e.getValue()[1]);
                        b.add(e.getKey(), x);
                    });
            o.add("by_biome", b);
            if (!examples.isEmpty())
                o.addProperty("off_ground_examples", examples.toString().trim());
            return o;
        }
    }

    /** The census of one level. */
    public record Result(Side now, Side plain, Map<String, Integer> replaced, int holes, int differ, int differByRules, int gatedBiomes,
                         JsonObject json) {}

    static TagKey<Biome> c(String path) {
        return TagKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath("c", path));
    }

    private static String name(Holder<Biome> b) {
        return b.unwrapKey().map(k -> k.location().toString()).orElse("?");
    }

    /**
     * Samples every {@code step} blocks inside the world's z edge ({@code zHalf}) less {@code poleMargin}; on a cube
     * world the faces only ({@link WorldDataTerrain#onFaces}: a void column shows its partner's, so each planet column
     * counts once).
     */
    public static Result run(ServerLevel level, int step, int poleMargin) {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !(g.terrain() instanceof WorldDataTerrain t))
            throw new IllegalStateException(level.dimension().location() + " is not a Deep Time level on world data");
        Climate.Sampler sampler = level.getChunkSource().randomState().sampler();
        if (!(sampler.spawnTarget() instanceof RegionSpawnTarget marker))
            throw new IllegalStateException(level.dimension().location() + "'s sampler does not carry the level's spawn target");
        var biomes = g.getBiomeSource();
        int sea = g.getSeaLevel();
        GeologyGround ground = marker.volcanic().geology();
        // One query binds the level's biome rules; the gate is every exclusive rule with a ground condition.
        biomes.getNoiseBiome(0, QuartPos.fromBlock(sea), 0, sampler);
        Map<Holder<Biome>, Integer> gated = GeologyBiomes.gated(marker.volcanic().rules());
        int c = t.circumference(), zp = (int) t.zHalf();
        Side now = new Side(), plain = new Side();
        Map<String, Integer> replaced = new HashMap<>();
        int holes = 0, differ = 0, differByRules = 0;
        int[] groundCount = new int[GeologyGround.Role.values().length];
        TagKey<Biome> ocean = c("is_ocean"), river = c("is_river"), cave = c("is_cave"), underground = c("is_underground");
        for (int z = -zp + step / 2; z < zp; z += step) {
            if (Math.abs(z) > zp - poleMargin)
                continue;
            for (int x = -c / 2 + step / 2; x < c / 2; x += step) {
                if (!t.onFaces(x, z))
                    continue;
                double y = t.height(x, z);
                if (y <= sea)
                    continue;
                int qx = QuartPos.fromBlock(x), qy = QuartPos.fromBlock((int) Math.floor(y)), qz = QuartPos.fromBlock(z);
                Holder<Biome> b = biomes.getNoiseBiome(qx, qy, qz, sampler);
                // A volcanic role's biome (Terralith's Basalt Cliffs as a young lava field) is placed by its zone, not
                // by the climate search: the gate does not apply to it.
                String why = VolcanicBiomes.lastDecision();
                boolean byRole = why != null && why.startsWith("role:");
                Holder<Biome> p = VolcanicBiomes.plain(() -> biomes.getNoiseBiome(qx, qy, qz, sampler));
                int bx = QuartPos.toBlock(qx) + 2, bz = QuartPos.toBlock(qz) + 2;
                int roles = gated.isEmpty() ? 0 : ground.rolesAt(bx, bz);
                for (GeologyGround.Role r : GeologyGround.Role.values())
                    if ((roles & r.bit()) != 0)
                        groundCount[r.ordinal()]++;
                boolean pOff = tally(plain, p, gated, roles, x, z);
                if (byRole)
                    now.columns++;
                else
                    tally(now, b, gated, roles, x, z);
                if (b != p) {
                    differ++;
                    if (VolcanicBiomes.byOrdinaryRule(why, marker.volcanic().rules()))
                        differByRules++;
                }
                if (pOff && b != p) {
                    replaced.merge(name(b), 1, Integer::sum);
                    if (b.is(ocean) || b.is(river) || b.is(cave) || b.is(underground))
                        holes++;
                }
            }
        }
        JsonObject o = new JsonObject();
        o.addProperty("level", level.dimension().location().toString());
        o.addProperty("step", step);
        JsonObject gj = new JsonObject();
        gated.forEach((b, bits) -> gj.addProperty(name(b), GeologyBiomes.names(bits)));
        o.add("gated_biomes", gj);
        JsonObject grd = new JsonObject();
        for (GeologyGround.Role r : GeologyGround.Role.values())
            grd.addProperty(r.id, groundCount[r.ordinal()]);
        o.add("ground", grd);
        o.add("now", now.json());
        o.add("plain", plain.json());
        JsonObject rep = new JsonObject();
        replaced.entrySet().stream().sorted(Map.Entry.<String, Integer>comparingByValue(Comparator.reverseOrder())).limit(12)
                .forEach(e -> rep.addProperty(e.getKey(), e.getValue()));
        o.add("replaced_by", rep);
        o.addProperty("replaced_holes", holes);
        o.addProperty("columns_changed", differ);
        o.addProperty("columns_changed_by_rules", differByRules);
        return new Result(now, plain, replaced, holes, differ, differByRules, gated.size(), o);
    }

    /** Counts one column; true when its biome is rock-named and off its ground. */
    private static boolean tally(Side side, Holder<Biome> b, Map<Holder<Biome>, Integer> gated, int ground, int x, int z) {
        side.columns++;
        Integer roles = gated.get(b);
        if (roles == null)
            return false;
        side.rockNamed++;
        int[] n = side.byBiome.computeIfAbsent(name(b), k -> new int[2]);
        n[0]++;
        if ((roles & ground) != 0)
            return false;
        side.offGround++;
        n[1]++;
        if (side.examples.length() < 400)
            side.examples.append(' ').append(name(b)).append('@').append(x).append(',').append(z)
                    .append('[').append(GeologyBiomes.names(ground)).append(']');
        return true;
    }
}
