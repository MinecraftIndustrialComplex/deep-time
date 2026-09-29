// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.RegistryOps;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.levelgen.feature.ConfiguredFeature;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.TreeMap;

/**
 * What {@code deeptime:claim_ores} removes (plan §7.7), decided per placed feature:
 *
 * <ul>
 *   <li><b>Claimed ore.</b> The feature (followed into nested features: random selectors, weighted
 *       lists, anything its configuration holds) places a block of {@code #c:ores/<m>} for a
 *       claimed material m ({@link OreData#claimedMaterials}: the deposit types' products and the
 *       scatter list, plus the modifier's extra materials). Removed.</li>
 *   <li><b>Rock blob</b> (only with {@code claim_rock_features}, plan Q4). Every block the feature
 *       places is bedrock of some rock class or the unclassified crust
 *       ({@link RockPalette.Host#ROCK}): vanilla's granite, diorite, andesite and tuff blobs,
 *       Terralith's re-stoned blobs, Create's striated layers, Born in Chaos's argillite. Loose rock
 *       (dirt, gravel, clay and sand pockets) is not a blob. The modifier's {@code rock_features}
 *       list names more. Removed.</li>
 *   <li>Everything else stays: unknown ores keep scattering, gated by host rock (plan §7.5).</li>
 * </ul>
 *
 * Only ore-shaped features are judged: {@code minecraft:ore}, {@code scattered_ore},
 * {@code replace_single_block}, {@code create:layered_ore} and {@code lithostitched:ore}, plus a
 * modifier's {@code ore_feature_types}; selectors are followed into the features they hold, and a
 * feature holding anything else (Terralith's boulders, pillars, floating islands, ponds) is never a
 * rock blob and claims ore only through its ore-shaped parts. The blocks an ore-shaped feature
 * places are read generically: its configured feature is encoded with its own codec and every block
 * state in the result is collected, except under a rule test's keys ({@code target},
 * {@code predicate}, …), which name what is replaced. That covers {@code minecraft:ore} and
 * {@code create:layered_ore} alike without compiling against any mod. A feature that places a
 * claimed ore and something unknown is removed; one that places rock and an unknown ore (Create
 * Nuclear's striated layers with uranium) is kept.
 */
public final class ClaimRules {

    /** Why a feature is removed. */
    public enum Verdict { KEEP, ORE, ROCK }

    /** Keys whose values name what a feature replaces or tests, not what it places. */
    private static final Set<String> TEST_KEYS = Set.of("target", "predicate", "location_predicate", "replaceable",
            "can_place_on", "can_replace", "blocks_to_replace", "replace", "target_blocks", "input_predicate");

    /** Feature types that only hold other features (followed, never judged themselves). */
    public static final Set<ResourceLocation> CONTAINER_TYPES = Set.of(
            ResourceLocation.withDefaultNamespace("random_selector"),
            ResourceLocation.withDefaultNamespace("simple_random_selector"),
            ResourceLocation.withDefaultNamespace("random_boolean_selector"));
    /** The ore-shaped feature types judged by default (a modifier's {@code ore_feature_types} adds more). */
    public static final List<ResourceLocation> DEFAULT_ORE_TYPES = List.of(
            ResourceLocation.withDefaultNamespace("ore"),
            ResourceLocation.withDefaultNamespace("scattered_ore"),
            ResourceLocation.withDefaultNamespace("replace_single_block"),
            ResourceLocation.fromNamespaceAndPath("create", "layered_ore"),
            ResourceLocation.fromNamespaceAndPath("lithostitched", "ore"));

    private final List<ResourceLocation> materials;
    private final List<TagKey<Block>> tags;
    private final boolean rockFeatures;
    private final Set<ResourceLocation> extraRock, keep, oreTypes;
    private final RockPalette palette;
    private final RegistryOps<JsonElement> ops;
    private final Map<ResourceLocation, Verdict> verdicts = new java.util.concurrent.ConcurrentHashMap<>();
    private final Map<ResourceLocation, Set<String>> placed = new java.util.concurrent.ConcurrentHashMap<>();

    public ClaimRules(RegistryAccess access, List<ResourceLocation> extraMaterials, boolean rockFeatures,
                      List<ResourceLocation> extraRock, List<ResourceLocation> keep, List<ResourceLocation> oreTypes) {
        java.util.TreeSet<ResourceLocation> m = new java.util.TreeSet<>(OreData.claimedMaterials(access));
        m.addAll(extraMaterials);
        this.materials = List.copyOf(m);
        this.tags = materials.stream().map(OreBlockResolver::oreTag).toList();
        this.rockFeatures = rockFeatures;
        this.extraRock = Set.copyOf(extraRock);
        this.keep = Set.copyOf(keep);
        Set<ResourceLocation> types = new HashSet<>(DEFAULT_ORE_TYPES);
        types.addAll(oreTypes);
        this.oreTypes = Set.copyOf(types);
        this.palette = RockPalette.build(access, RockPalette.View.LIVE);
        this.ops = RegistryOps.create(JsonOps.INSTANCE, access);
    }

    public List<ResourceLocation> materials() {
        return materials;
    }

    public boolean rockFeatures() {
        return rockFeatures;
    }

    /** The verdict for a placed feature (cached per id; direct holders are never cached). */
    public Verdict verdict(Holder<PlacedFeature> feature) {
        Optional<ResourceLocation> id = feature.unwrapKey().map(ResourceKey::location);
        if (id.isPresent() && keep.contains(id.get()))
            return Verdict.KEEP;
        if (id.isPresent()) {
            Verdict v = verdicts.get(id.get());
            if (v != null)
                return v;
        }
        Leaves leaves = leaves(feature.value());
        id.ifPresent(i -> placed.put(i, leaves.oreBlocks()));
        Verdict v = decide(leaves);
        if (v == Verdict.KEEP && id.isPresent() && rockFeatures && extraRock.contains(id.get()))
            v = Verdict.ROCK;
        if (v == Verdict.KEEP && rockFeatures && configuredId(feature).map(extraRock::contains).orElse(false))
            v = Verdict.ROCK;
        Verdict fv = v;
        id.ifPresent(i -> verdicts.put(i, fv));
        return v;
    }

    private static Optional<ResourceLocation> configuredId(Holder<PlacedFeature> feature) {
        return feature.value().feature().unwrapKey().map(ResourceKey::location);
    }

    /** The block ids a placed feature places (after {@link #verdict} computed them). */
    public Set<String> placedBlocksOf(ResourceLocation placedFeature) {
        return placed.getOrDefault(placedFeature, Set.of());
    }

    /**
     * The judged parts of a feature: the blocks its ore-shaped features place, and how many of its
     * features are neither ore-shaped nor containers (terrain features: boulders, pillars,
     * floating islands, ponds).
     */
    record Leaves(Set<String> oreBlocks, int others) {}

    Verdict decide(Leaves leaves) {
        Set<String> blocks = leaves.oreBlocks();
        if (blocks.isEmpty())
            return Verdict.KEEP;
        boolean allRock = leaves.others() == 0;
        for (String id : blocks) {
            Block b = BuiltInRegistries.BLOCK.getOptional(ResourceLocation.tryParse(id)).orElse(null);
            if (b == null) {
                allRock = false;
                continue;
            }
            for (TagKey<Block> t : tags)
                if (b.defaultBlockState().is(t))
                    return Verdict.ORE;
            if (palette.host(b.defaultBlockState()) != RockPalette.Host.ROCK)
                allRock = false;
        }
        return rockFeatures && allRock ? Verdict.ROCK : Verdict.KEEP;
    }

    /**
     * Follows the feature through its containers: the blocks every ore-shaped feature in it places,
     * and the count of the others.
     */
    Leaves leaves(PlacedFeature feature) {
        Set<String> out = new LinkedHashSet<>();
        int[] others = {0};
        Set<ConfiguredFeature<?, ?>> seen = java.util.Collections.newSetFromMap(new java.util.IdentityHashMap<>());
        collect(feature.feature().value(), out, others, seen);
        return new Leaves(out, others[0]);
    }

    private void collect(ConfiguredFeature<?, ?> cf, Set<String> out, int[] others, Set<ConfiguredFeature<?, ?>> seen) {
        if (!seen.add(cf))
            return;
        ResourceLocation type = BuiltInRegistries.FEATURE.getKey(cf.feature());
        if (type != null && oreTypes.contains(type))
            encode(cf).ifPresent(json -> walk(json, out, false));
        else if (type == null || !CONTAINER_TYPES.contains(type))
            others[0]++;
        cf.getFeatures().forEach(nested -> {
            if (nested != cf)
                collect(nested, out, others, seen);
        });
    }

    private Optional<JsonElement> encode(ConfiguredFeature<?, ?> cf) {
        try {
            return ConfiguredFeature.DIRECT_CODEC.encodeStart(ops, cf).result();
        } catch (RuntimeException e) {
            DeepTime.LOGGER.debug("claim_ores: cannot encode {}: {}", cf, e.toString());
            return Optional.empty();
        }
    }

    /** Collects block-state objects ({@code {"Name": …}}) outside rule-test keys. */
    private static void walk(JsonElement e, Set<String> out, boolean inTest) {
        if (e == null || e.isJsonNull() || inTest)
            return;
        if (e.isJsonArray()) {
            for (JsonElement c : e.getAsJsonArray())
                walk(c, out, false);
            return;
        }
        if (!e.isJsonObject())
            return;
        JsonObject o = e.getAsJsonObject();
        if (o.has("Name") && o.get("Name").isJsonPrimitive()) {
            out.add(o.get("Name").getAsString());
            return;
        }
        for (Map.Entry<String, JsonElement> kv : o.entrySet())
            walk(kv.getValue(), out, TEST_KEYS.contains(kv.getKey()));
    }

    /**
     * The verdict of every registered placed feature, by id (a dry run of the modifier: the GameTests
     * and {@code /deeptime probe claims} read it).
     */
    public Map<ResourceLocation, Verdict> planAll(RegistryAccess access) {
        Map<ResourceLocation, Verdict> out = new TreeMap<>();
        access.registryOrThrow(Registries.PLACED_FEATURE).holders()
                .forEach(h -> out.put(h.key().location(), verdict(h)));
        return out;
    }

    // ---- the current server's rules -------------------------------------------------------------

    private static final Map<ClaimOresBiomeModifier, ClaimRules> CACHE = new java.util.IdentityHashMap<>();
    private static java.lang.ref.WeakReference<net.minecraft.server.MinecraftServer> cacheServer =
            new java.lang.ref.WeakReference<>(null);

    /** The rules for a server and a modifier's settings (built once per server and modifier). */
    static synchronized ClaimRules forServer(net.minecraft.server.MinecraftServer server, ClaimOresBiomeModifier m) {
        if (cacheServer.get() != server) {
            CACHE.clear();
            REMOVED.clear();
            cacheServer = new java.lang.ref.WeakReference<>(server);
        }
        return CACHE.computeIfAbsent(m, k -> {
            ClaimRules r = new ClaimRules(server.registryAccess(), m.materials(), m.claimRockFeatures(), m.rockFeatures(),
                    m.keep(), m.oreFeatureTypes());
            DeepTime.LOGGER.info("claim_ores: claimed materials {} (rock features {})", r.materials(),
                    r.rockFeatures() ? "on" : "off");
            return r;
        });
    }

    /**
     * The registered {@code deeptime:claim_ores} modifiers of a server (Deep Time's own and any a pack
     * adds), by id.
     */
    public static Map<ResourceLocation, ClaimOresBiomeModifier> modifiers(net.minecraft.server.MinecraftServer server) {
        Map<ResourceLocation, ClaimOresBiomeModifier> out = new TreeMap<>();
        server.registryAccess().registry(net.neoforged.neoforge.registries.NeoForgeRegistries.Keys.BIOME_MODIFIERS)
                .ifPresent(reg -> reg.entrySet().forEach(e -> {
                    if (e.getValue() instanceof ClaimOresBiomeModifier m)
                        out.put(e.getKey().location(), m);
                }));
        return out;
    }

    /**
     * Rules with the settings of the server's {@code deeptime:claim_ores} modifier (a dry run works
     * in any world; the defaults when the modifier is absent).
     */
    public static ClaimRules dryRun(net.minecraft.server.MinecraftServer server) {
        ClaimOresBiomeModifier m = modifiers(server).get(ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "claim_ores"));
        if (m == null)
            return new ClaimRules(server.registryAccess(), List.of(), true, List.of(), List.of(), List.of());
        return new ClaimRules(server.registryAccess(), m.materials(), m.claimRockFeatures(), m.rockFeatures(), m.keep(),
                m.oreFeatureTypes());
    }

    /** What the modifier removed on this server, by feature id → verdict (for the probe and tests). */
    public static final Map<ResourceLocation, Verdict> REMOVED = java.util.Collections.synchronizedMap(new LinkedHashMap<>());

    static void recordRemoved(ResourceLocation id, Verdict v) {
        REMOVED.put(id, v);
    }

    /** Placed-feature ids grouped by verdict (dry-run summaries). */
    public static Map<Verdict, List<ResourceLocation>> byVerdict(Map<ResourceLocation, Verdict> plan) {
        Map<Verdict, List<ResourceLocation>> out = new TreeMap<>();
        plan.forEach((id, v) -> out.computeIfAbsent(v, k -> new ArrayList<>()).add(id));
        return out;
    }

    static Set<ResourceLocation> ids(List<ResourceLocation> l) {
        return new HashSet<>(l);
    }
}
