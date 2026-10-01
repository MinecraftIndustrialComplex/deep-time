// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.mojang.serialization.Codec;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeSet;

/**
 * The ore-side datapack registries (plan §5.4.3, §7.7, §7.8), read on the server only:
 *
 * <ul>
 *   <li>{@code deeptime:deposit_type} ({@code data/<ns>/deeptime/deposit_type/}): the deposit types
 *       the core simulates (P1-14's JSON). Java needs only their products and ore-block policy (the
 *       claimed materials); every other key is the core's and is ignored here.</li>
 *   <li>{@code deeptime:scatter} ({@code data/<ns>/deeptime/scatter/<material>.json}): background
 *       scatter, small host-gated clusters of common materials anywhere.</li>
 *   <li>{@code deeptime:essentials} ({@code data/<ns>/deeptime/essentials/}): the playability
 *       minimums the core's deposit stage guarantees, and the spawn's reach.</li>
 * </ul>
 */
public final class OreData {

    private OreData() {}

    // ---- deposit types ------------------------------------------------------------------------

    /** A product as the core writes it (a {@code c:} material; {@code grade_key} is the core's). */
    public record ProductDef(ResourceLocation material, String role) {
        static final Codec<ProductDef> CODEC = RecordCodecBuilder.create(i -> i.group(
                ResourceLocation.CODEC.fieldOf("material").forGetter(ProductDef::material),
                Codec.STRING.optionalFieldOf("role", "primary").forGetter(ProductDef::role)
        ).apply(i, ProductDef::new));
    }

    /**
     * The {@code minecraft} section: how the type's ore reaches the world. {@code coeVein} and
     * {@code cdgScale} are read at run time by the COE and CDG hooks (P1-16, plan §7.14), so a pack
     * that overrides the type's file in a datapack remaps the vein or rescales the oil of worlds
     * that already exist.
     */
    public record MinecraftDef(String oreBlockPolicy, Optional<String> coeVein, boolean cdgReservoir, double cdgScale) {
        static final MinecraftDef DEFAULT = new MinecraftDef("ev_then_vanilla", Optional.empty(), false, 1.0);
        static final Codec<MinecraftDef> CODEC = RecordCodecBuilder.create(i -> i.group(
                Codec.STRING.optionalFieldOf("ore_block_policy", "ev_then_vanilla").forGetter(MinecraftDef::oreBlockPolicy),
                Codec.STRING.lenientOptionalFieldOf("coe_vein").forGetter(MinecraftDef::coeVein),
                Codec.BOOL.optionalFieldOf("cdg_reservoir", false).forGetter(MinecraftDef::cdgReservoir),
                Codec.doubleRange(0, Double.MAX_VALUE).optionalFieldOf("cdg_scale", 1.0).forGetter(MinecraftDef::cdgScale)
        ).apply(i, MinecraftDef::new));
    }

    /**
     * A USGS-style percentile triple (P90 = the value 90 % of deposits exceed, so P90 ≤ P50 ≤ P10).
     * The core fits a lognormal split at the median to it ({@code deposits::sizing::Lognormal}).
     */
    public record Percentiles(double p90, double p50, double p10) {
        static final Codec<Percentiles> CODEC = RecordCodecBuilder.create(i -> i.group(
                Codec.DOUBLE.fieldOf("p90").forGetter(Percentiles::p90),
                Codec.DOUBLE.fieldOf("p50").forGetter(Percentiles::p50),
                Codec.DOUBLE.fieldOf("p10").forGetter(Percentiles::p10)
        ).apply(i, Percentiles::new));

        /** Φ⁻¹(0.9), the core's {@code Z90}. */
        static final double Z90 = 1.2815515655446005;

        /**
         * The fraction of deposits below {@code x} under the core's split lognormal: σ below the
         * median is (ln P50 − ln P90)/Φ⁻¹(0.9), above it (ln P10 − ln P50)/Φ⁻¹(0.9) (Rust
         * {@code Lognormal::cdf}).
         */
        public double cdf(double x) {
            if (!(x > 0) || !(p50 > 0))
                return 0;
            double d = Math.log(x) - Math.log(p50);
            double s = d < 0 ? (Math.log(p50) - Math.log(Math.max(p90, 1e-300))) / Z90
                    : (Math.log(Math.max(p10, 1e-300)) - Math.log(p50)) / Z90;
            if (!(s > 0))
                return d < 0 ? 0 : 1;
            return normalCdf(d / s);
        }

        /** Φ(x) with the core's erf (Abramowitz–Stegun 7.1.26, |error| < 1.5e-7; Rust {@code random::normal_cdf}). */
        static double normalCdf(double x) {
            double z = x / Math.sqrt(2);
            double sgn = Math.signum(z), a = Math.abs(z);
            double t = 1.0 / (1.0 + 0.3275911 * a);
            double y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
                    + 0.254829592) * t * Math.exp(-a * a);
            return 0.5 * (1.0 + sgn * y);
        }
    }

    /** The {@code sizing} section, as far as Java reads it: the tonnage percentiles in Mt. */
    public record SizingDef(Optional<Percentiles> tonnageMt) {
        static final SizingDef NONE = new SizingDef(Optional.empty());
        static final Codec<SizingDef> CODEC = RecordCodecBuilder.create(i -> i.group(
                Percentiles.CODEC.lenientOptionalFieldOf("tonnage_mt").forGetter(SizingDef::tonnageMt)
        ).apply(i, SizingDef::new));
    }

    /** What Java reads of a deposit type (the core parses the whole schema). */
    public record DepositTypeInfo(List<ProductDef> products, MinecraftDef minecraft, SizingDef sizing) {
        public static final Codec<DepositTypeInfo> CODEC = RecordCodecBuilder.create(i -> i.group(
                ProductDef.CODEC.listOf().optionalFieldOf("products", List.of()).forGetter(DepositTypeInfo::products),
                MinecraftDef.CODEC.optionalFieldOf("minecraft", MinecraftDef.DEFAULT).forGetter(DepositTypeInfo::minecraft),
                SizingDef.CODEC.lenientOptionalFieldOf("sizing", SizingDef.NONE).forGetter(DepositTypeInfo::sizing)
        ).apply(i, DepositTypeInfo::new));
    }

    /** The loaded deposit type {@code id}, if the registry has it. */
    public static Optional<DepositTypeInfo> depositType(RegistryAccess access, ResourceLocation id) {
        return access.registry(DEPOSIT_TYPE).flatMap(r -> r.getOptional(id));
    }

    public static final ResourceKey<Registry<DepositTypeInfo>> DEPOSIT_TYPE =
            ResourceKey.createRegistryKey(ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "deposit_type"));

    // ---- background scatter -------------------------------------------------------------------

    private static final Codec<List<Double>> WINDOW = Codec.DOUBLE.listOf().validate(l -> l.size() == 2
            && l.get(0) >= 0 && l.get(1) <= 1 && l.get(0) <= l.get(1)
            ? DataResult.success(l) : DataResult.error(() -> "y_window must be [f0, f1] with 0 ≤ f0 ≤ f1 ≤ 1, got " + l));

    /**
     * One background-scatter rule (plan §7.8).
     *
     * @param material         a {@code c:} material; its ore blocks come from {@link OreBlockResolver}
     * @param clustersPerChunk mean clusters per chunk (the fraction is a chance of one more)
     * @param size             blocks tried per cluster (a random walk from the cluster's start)
     * @param yWindow          [f0, f1]: fractions of the column's depth from the surface (0) to the
     *                         bottom of the world (1) where clusters start. The plan's "fraction of
     *                         crust thickness": the playable crust is the column below the surface
     * @param hostBias         rock class → acceptance weight (0–1; {@code "*"} for any other class,
     *                         default 1); {@code unclassified_crust} is stone and deepslate that no horizon
     *                         paints as a class (a block of stone or deepslate that is a class's, say shale's or
     *                         gneiss's since P1-28, counts as that class: {@link com.minecraftindustrialcomplex.deeptime.worldgen.BackgroundScatterFeature#hostClasses})
     */
    public record ScatterRule(ResourceLocation material, double clustersPerChunk, int size, List<Double> yWindow,
                              Map<String, Double> hostBias) {
        public static final Codec<ScatterRule> CODEC = RecordCodecBuilder.create(i -> i.group(
                ResourceLocation.CODEC.fieldOf("material").forGetter(ScatterRule::material),
                Codec.doubleRange(0, 64).fieldOf("clusters_per_chunk").forGetter(ScatterRule::clustersPerChunk),
                Codec.intRange(1, 64).fieldOf("size").forGetter(ScatterRule::size),
                WINDOW.optionalFieldOf("y_window", List.of(0.0, 1.0)).forGetter(ScatterRule::yWindow),
                Codec.unboundedMap(Codec.STRING, Codec.doubleRange(0, 1)).optionalFieldOf("host_bias", Map.of())
                        .forGetter(ScatterRule::hostBias)
        ).apply(i, ScatterRule::new));

        /** Whether every host gets the same weight, so no block's class needs finding out. */
        public boolean classBlind() {
            java.util.Set<Double> weights = new java.util.HashSet<>(hostBias.values());
            weights.add(hostBias.getOrDefault("*", 1.0));
            return weights.size() == 1;
        }

        /** The acceptance weight of a host with these classes: the largest listed, else {@code "*"}, else 1. */
        public double bias(List<String> classes) {
            double star = hostBias.getOrDefault("*", 1.0);
            double best = -1;
            for (String c : classes)
                best = Math.max(best, hostBias.getOrDefault(c, star));
            return best < 0 ? star : best;
        }
    }

    public static final ResourceKey<Registry<ScatterRule>> SCATTER =
            ResourceKey.createRegistryKey(ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "scatter"));

    // ---- essentials ---------------------------------------------------------------------------

    /**
     * One essential material (plan §7.8; decisions file "Playability overrides": a pack-defined
     * list). The core's deposit stage keeps at least {@code minSitesPerRegion} sites of the material
     * in every land region of {@code regionBlocks} (exposed ones when {@code exposed}), and the spawn
     * is chosen within {@code maxBlocksFromSpawn} of one when set. The field names and meaning are
     * the core's {@code deposits::playability::Essential}; the list goes to the core in the content
     * bundle ({@link #contentJson}).
     */
    public record Essential(ResourceLocation material, int minSitesPerRegion, int regionBlocks, boolean exposed,
                            Optional<Integer> maxBlocksFromSpawn) {
        public static final Codec<Essential> CODEC = RecordCodecBuilder.create(i -> i.group(
                ResourceLocation.CODEC.fieldOf("material").forGetter(Essential::material),
                Codec.intRange(0, 1024).fieldOf("min_sites_per_region").forGetter(Essential::minSitesPerRegion),
                Codec.intRange(64, 1 << 20).fieldOf("region_blocks").forGetter(Essential::regionBlocks),
                Codec.BOOL.optionalFieldOf("exposed", false).forGetter(Essential::exposed),
                Codec.intRange(1, 1 << 20).optionalFieldOf("max_blocks_from_spawn").forGetter(Essential::maxBlocksFromSpawn)
        ).apply(i, Essential::new));
    }

    public static final ResourceKey<Registry<Essential>> ESSENTIALS =
            ResourceKey.createRegistryKey(ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "essentials"));

    // ---- reads --------------------------------------------------------------------------------

    /** A registry's entries ordered by id (empty when the registry is absent). */
    public static <T> List<Map.Entry<ResourceLocation, T>> entries(RegistryAccess access, ResourceKey<Registry<T>> key) {
        List<Map.Entry<ResourceLocation, T>> out = new ArrayList<>();
        access.registry(key).ifPresent(reg -> reg.entrySet().stream()
                .sorted(Comparator.comparing(e -> e.getKey().location().toString()))
                .forEach(e -> out.add(Map.entry(e.getKey().location(), e.getValue()))));
        return out;
    }

    /**
     * The materials Deep Time claims (plan §7.7): every product of the loaded deposit types plus
     * every background-scatter material, sorted. {@code c:crude_oil} is among them and has no ore
     * blocks; it claims nothing.
     */
    public static List<ResourceLocation> claimedMaterials(RegistryAccess access) {
        TreeSet<ResourceLocation> out = new TreeSet<>();
        for (var e : entries(access, DEPOSIT_TYPE))
            for (ProductDef p : e.getValue().products())
                out.add(p.material());
        for (var e : entries(access, SCATTER))
            out.add(e.getValue().material());
        return List.copyOf(out);
    }

    /**
     * The content bundle the core reads (plan §5.4; {@code Preset::with_content}): today the
     * essentials, in the core's field names.
     */
    public static String contentJson(net.minecraft.server.MinecraftServer server) {
        return contentJson(server.registryAccess());
    }

    public static String contentJson(RegistryAccess access) {
        var root = new com.google.gson.JsonObject();
        var list = new com.google.gson.JsonArray();
        for (var e : entries(access, ESSENTIALS)) {
            Essential es = e.getValue();
            var o = new com.google.gson.JsonObject();
            o.addProperty("material", es.material().toString());
            o.addProperty("min_sites_per_region", es.minSitesPerRegion());
            o.addProperty("region_blocks", es.regionBlocks());
            o.addProperty("exposed", es.exposed());
            es.maxBlocksFromSpawn().ifPresent(m -> o.addProperty("max_blocks_from_spawn", m));
            list.add(o);
        }
        root.add("essentials", list);
        return root.toString();
    }
}
