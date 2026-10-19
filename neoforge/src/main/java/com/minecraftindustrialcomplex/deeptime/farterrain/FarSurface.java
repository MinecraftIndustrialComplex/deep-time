// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.farterrain;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.WaterBodies;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Far terrain's surface table (plan §10.5 P3-04): the soil a biome's far columns show. A real
 * chunk's top blocks come from the pack's surface rules, which run over a generated chunk; far
 * terrain has none, so it reads the datapack file {@code data/deeptime/deeptime/far_terrain/surface.json}
 * (the jar ships a default, a pack overrides it): rules of biome ids and {@code #tags}, the first
 * that holds the biome wins, each giving the top block, the block under it, the soil depth, the
 * sea-bed block, or {@code bare} (the rock is the surface). The leading rules of the pack's surface
 * rule that read no chunk run first ({@link PackSurfaceRule}); this table stands for the rest.
 * Everything else a far column shows (the rock, water, the ice cover, snow) comes from the world
 * data, as in real chunks.
 *
 * <p>Immutable after {@link #read}; {@link #of} caches per biome and is thread-safe.
 */
public final class FarSurface {

    public static final ResourceLocation RESOURCE =
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "deeptime/far_terrain/surface.json");
    private static final String JAR_RESOURCE = "data/deeptime/deeptime/far_terrain/surface.json";

    /** What one biome's columns show. {@code lakeBed} is {@code top} as it is under a lake (grass becomes dirt). */
    public record Choice(BlockState top, BlockState under, int depth, BlockState seaBed, BlockState lakeBed, boolean bare) {}

    private record Rule(List<ResourceLocation> ids, List<TagKey<Biome>> tags, BlockState top, BlockState under, Integer depth,
                        BlockState seaBed, Boolean bare) {
        boolean holds(Holder<Biome> biome) {
            for (ResourceLocation id : ids)
                if (biome.is(id))
                    return true;
            for (TagKey<Biome> tag : tags)
                if (biome.is(tag))
                    return true;
            return false;
        }
    }

    private final Choice fallback;
    private final List<Rule> rules;
    private final String source;
    private final Map<Holder<Biome>, Choice> cache = new ConcurrentHashMap<>();

    private FarSurface(Choice fallback, List<Rule> rules, String source) {
        this.fallback = fallback;
        this.rules = rules;
        this.source = source;
    }

    /** Where the table came from, and its size, for the log. */
    public String describe() {
        return source + ", " + rules.size() + " rules";
    }

    /** The table of a server's datapacks; a missing or malformed file falls back to the jar's default (logged). */
    public static FarSurface read(ResourceManager resources) {
        if (resources != null) {
            try {
                var res = resources.getResource(RESOURCE);
                if (res.isPresent()) {
                    try (Reader r = res.get().openAsReader()) {
                        return parse(JsonParser.parseReader(r).getAsJsonObject(), "datapack " + res.get().sourcePackId());
                    }
                }
            } catch (IOException | RuntimeException e) {
                DeepTime.LOGGER.error("Far terrain: cannot read {} ({}); using the built-in table", RESOURCE, e.toString());
            }
        }
        return builtin();
    }

    /** The table the jar ships. */
    public static FarSurface builtin() {
        try (InputStream in = FarSurface.class.getClassLoader().getResourceAsStream(JAR_RESOURCE)) {
            if (in != null)
                return parse(JsonParser.parseReader(new InputStreamReader(in, StandardCharsets.UTF_8)).getAsJsonObject(), "built-in");
        } catch (IOException | RuntimeException e) {
            DeepTime.LOGGER.error("Far terrain: the built-in surface table is unreadable ({})", e.toString());
        }
        return new FarSurface(hardDefault(), List.of(), "hard-coded default");
    }

    private static Choice hardDefault() {
        return choice(Blocks.GRASS_BLOCK.defaultBlockState(), Blocks.DIRT.defaultBlockState(), 4, Blocks.GRAVEL.defaultBlockState(), false);
    }

    private static Choice choice(BlockState top, BlockState under, int depth, BlockState seaBed, boolean bare) {
        BlockState lakeBed = WaterBodies.underWater(top);
        return new Choice(top, under, Math.max(1, Math.min(64, depth)), seaBed, lakeBed, bare);
    }

    static FarSurface parse(JsonObject doc, String source) {
        Choice hard = hardDefault();
        JsonObject d = doc.has("default") ? doc.getAsJsonObject("default") : new JsonObject();
        Choice fallback = choice(block(d, "top").orElse(hard.top()), block(d, "under").orElse(hard.under()),
                d.has("depth") ? d.get("depth").getAsInt() : hard.depth(), block(d, "sea_bed").orElse(hard.seaBed()),
                d.has("bare") && d.get("bare").getAsBoolean());
        List<Rule> rules = new ArrayList<>();
        JsonArray arr = doc.has("rules") ? doc.getAsJsonArray("rules") : new JsonArray();
        for (JsonElement e : arr) {
            JsonObject o = e.getAsJsonObject();
            List<ResourceLocation> ids = new ArrayList<>();
            List<TagKey<Biome>> tags = new ArrayList<>();
            for (JsonElement b : o.getAsJsonArray("biomes")) {
                String s = b.getAsString();
                ResourceLocation rl = ResourceLocation.tryParse(s.startsWith("#") ? s.substring(1) : s);
                if (rl == null) {
                    DeepTime.LOGGER.warn("Far terrain surface table ({}): '{}' is not an id; skipped", source, s);
                } else if (s.startsWith("#")) {
                    tags.add(TagKey.create(Registries.BIOME, rl));
                } else {
                    ids.add(rl);
                }
            }
            rules.add(new Rule(List.copyOf(ids), List.copyOf(tags), block(o, "top").orElse(null), block(o, "under").orElse(null),
                    o.has("depth") ? o.get("depth").getAsInt() : null, block(o, "sea_bed").orElse(null),
                    o.has("bare") ? o.get("bare").getAsBoolean() : null));
        }
        return new FarSurface(fallback, List.copyOf(rules), source);
    }

    private static Optional<BlockState> block(JsonObject o, String key) {
        if (!o.has(key))
            return Optional.empty();
        ResourceLocation rl = ResourceLocation.tryParse(o.get(key).getAsString());
        Optional<Block> b = rl == null ? Optional.empty() : BuiltInRegistries.BLOCK.getOptional(rl);
        if (b.isEmpty())
            DeepTime.LOGGER.warn("Far terrain surface table: no block '{}' ({}); the default is used", o.get(key).getAsString(), key);
        return b.map(Block::defaultBlockState);
    }

    /** What {@code biome}'s far columns show. */
    public Choice of(Holder<Biome> biome) {
        Choice c = cache.get(biome);
        if (c == null) {
            c = resolve(biome);
            cache.put(biome, c);
        }
        return c;
    }

    private Choice resolve(Holder<Biome> biome) {
        for (Rule r : rules) {
            if (!r.holds(biome))
                continue;
            return choice(r.top() != null ? r.top() : fallback.top(), r.under() != null ? r.under() : fallback.under(),
                    r.depth() != null ? r.depth() : fallback.depth(), r.seaBed() != null ? r.seaBed() : fallback.seaBed(),
                    r.bare() != null ? r.bare() : fallback.bare());
        }
        return fallback;
    }
}
