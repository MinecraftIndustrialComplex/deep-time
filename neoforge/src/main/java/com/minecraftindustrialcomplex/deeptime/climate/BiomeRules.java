// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeRuleSet.Rule;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeRuleSet.Wetland;
import com.minecraftindustrialcomplex.deeptime.worldgen.StrataPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import com.minecraftindustrialcomplex.deeptime.worlddata.ClimateLayers;
import com.minecraftindustrialcomplex.deeptime.worlddata.IceCover;
import com.minecraftindustrialcomplex.deeptime.worlddata.Playa;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;

import java.io.IOException;
import java.io.InputStreamReader;
import java.io.Reader;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.TreeMap;
import java.util.function.Supplier;

/**
 * The game side of Deep Time's biome rules ({@link BiomeRuleSet}, FORMAT.md "Biome rules"; Ben, decisions log
 * 2026-10-04): the rule files of the server's datapacks bound to one biome source (only the files whose
 * {@code requires} biomes the source can place, only the candidates it has), and the per-column facts the
 * rules ask about, read from the world data. {@link VolcanicBiomes#find} consults them in a Deep Time
 * level's biome query: the volcanic roles first (a role's rules before its tag), then the ordinary rules, then
 * the climate search with every exclusive candidate and every {@code #deeptime:volcanic} biome kept out.
 */
public final class BiomeRules {

    private BiomeRules() {}

    /** One rule bound to a source: its candidates (those the source can place) as an identity set. */
    public record Bound(Rule rule, String file, Set<Holder<Biome>> candidates) {}

    /** The rules of one source; {@link #NONE} when no file is active. */
    public record Set_(List<Bound> ordinary, Map<String, List<Bound>> byRole, Set<Holder<Biome>> exclusive, List<String> files) {
        public boolean isEmpty() {
            return ordinary.isEmpty() && byRole.isEmpty();
        }

        public List<Bound> role(String id) {
            return byRole.getOrDefault(id, List.of());
        }
    }

    public static final Set_ NONE = new Set_(List.of(), Map.of(), Set.of(), List.of());

    /** The parsed rule files of the server's datapacks, in the order of their ids (a broken file is logged and skipped). */
    public static List<BiomeRuleSet.File> load(ResourceManager resources) {
        List<BiomeRuleSet.File> out = new ArrayList<>();
        Map<ResourceLocation, Resource> found = new TreeMap<>(resources.listResources(BiomeRuleSet.DIR, p -> p.getPath().endsWith(".json")));
        for (var e : found.entrySet()) {
            String path = e.getKey().getPath();
            String id = e.getKey().getNamespace() + ":" + path.substring(BiomeRuleSet.DIR.length() + 1, path.length() - ".json".length());
            try (Reader r = new InputStreamReader(e.getValue().open(), StandardCharsets.UTF_8)) {
                JsonObject o = JsonParser.parseReader(r).getAsJsonObject();
                out.add(BiomeRuleSet.parse(id, o));
            } catch (IOException | RuntimeException ex) {
                DeepTime.LOGGER.error("[deeptime] biome rules: {} (pack {}) is skipped: {}", id, e.getValue().sourcePackId(), ex.getMessage());
            }
        }
        return out;
    }

    /** {@code files} bound to the biomes {@code possible} of one source. */
    public static Set_ bind(List<BiomeRuleSet.File> files, Set<Holder<Biome>> possible, Registry<Biome> registry) {
        List<Bound> ordinary = new ArrayList<>();
        Map<String, List<Bound>> byRole = new LinkedHashMap<>();
        Set<Holder<Biome>> exclusive = Collections.newSetFromMap(new IdentityHashMap<>());
        List<String> active = new ArrayList<>();
        for (BiomeRuleSet.File f : files) {
            boolean on = f.requires().isEmpty();
            for (String req : f.requires())
                on |= !resolve(List.of(req), possible, registry).isEmpty();
            if (!on)
                continue;
            exclusive.addAll(resolve(f.keepOut(), possible, registry));
            int bound = 0;
            for (Rule r : f.rules()) {
                Set<Holder<Biome>> c = resolve(r.biomes(), possible, registry);
                if (c.isEmpty())
                    continue;
                bound++;
                Bound b = new Bound(r, f.id(), c);
                if (r.role() != null)
                    byRole.computeIfAbsent(r.role(), k -> new ArrayList<>()).add(b);
                else
                    ordinary.add(b);
                if (r.exclusive())
                    exclusive.addAll(c);
            }
            active.add(f.id() + " (" + bound + " of " + f.rules().size() + " rules)");
        }
        if (active.isEmpty())
            return NONE;
        return new Set_(List.copyOf(ordinary), Map.copyOf(byRole), exclusive, List.copyOf(active));
    }

    /** The biomes of {@code ids} (ids and {@code #tags}) that {@code possible} holds, as an identity set. */
    static Set<Holder<Biome>> resolve(List<String> ids, Set<Holder<Biome>> possible, Registry<Biome> registry) {
        Set<Holder<Biome>> out = Collections.newSetFromMap(new IdentityHashMap<>());
        for (String id : ids) {
            if (id.startsWith("#")) {
                ResourceLocation tag = ResourceLocation.tryParse(id.substring(1));
                if (tag == null)
                    continue;
                Optional<HolderSet.Named<Biome>> set = registry.getTag(TagKey.create(Registries.BIOME, tag));
                if (set.isPresent())
                    for (Holder<Biome> b : set.get())
                        if (possible.contains(b))
                            out.add(b);
            } else {
                ResourceLocation key = ResourceLocation.tryParse(id);
                if (key == null)
                    continue;
                registry.getHolder(ResourceKey.create(Registries.BIOME, key)).filter(possible::contains).ifPresent(out::add);
            }
        }
        return out;
    }

    /** The search over {@code candidates} alone: every other leaf is infinitely far away. */
    static Climate.DistanceMetric<Holder<Biome>> only(Set<Holder<Biome>> candidates) {
        return (node, values) -> {
            if (node == null)
                return Long.MAX_VALUE;
            if (node instanceof Climate.RTree.Leaf<Holder<Biome>> leaf && !candidates.contains(leaf.value))
                return Long.MAX_VALUE;
            return node.distance(values);
        };
    }

    /** The nearest of the rule's candidates in {@code source} for {@code t}, or null when the source's tree has none. */
    public static Holder<Biome> search(VolcanicBiomes.Source source, Climate.TargetPoint t, Bound rule) {
        Holder<Biome> b;
        try {
            b = source.find(t, only(rule.candidates()));
        } catch (NullPointerException emptyTree) {
            return null;   // a tree (a TerraBlender region's) without any of the candidates
        }
        return b != null && rule.candidates().contains(b) ? b : null;
    }

    /** Whether {@code b} is in a list of ids and {@code #tags}. */
    static boolean in(Holder<Biome> b, List<String> ids) {
        for (String id : ids) {
            if (id.startsWith("#")) {
                ResourceLocation tag = ResourceLocation.tryParse(id.substring(1));
                if (tag != null && b.is(TagKey.create(Registries.BIOME, tag)))
                    return true;
            } else if (b.unwrapKey().map(k -> k.location().toString().equals(id)).orElse(false)) {
                return true;
            }
        }
        return false;
    }

    // ------------------------------------------------------------------------------------- the facts

    /** The Köppen classes whose wetland is boreal peatland where the world has no wetland flag layer (Dxc, Dxd). */
    static final Set<String> COLD_WETLAND_CLASSES = Set.of("Dfc", "Dfd", "Dsc", "Dsd", "Dwc", "Dwd"); // ledger: BIO-115 (game-choice)

    /** The window below the surface in which the top rock class is read, blocks. */
    static final int ROCK_WINDOW = 6; // ledger: BIO-116 (game-choice)

    /** The layer the bake writes for wetlands where it has one (0 none, 1 swamp band, 2 cold wetland). */
    static final String WETLAND_LAYER = "climate/wetland";

    /** What a level's rules read from its world data, opened on first use. */
    public static final class LevelData {
        final WorldDataTerrain terrain;
        private volatile Opened opened;

        public LevelData(WorldDataTerrain terrain) {
            this.terrain = terrain;
        }

        record Opened(WorldData wd, ClimateLayers climate, RasterSampler koppen, Optional<IceCover> ice, RasterSampler playa,
                      RasterSampler wetland, RasterSampler strata, List<String> classes, double e6, double mPerBlockV) {}

        Opened open() {
            Opened o = opened;
            if (o == null) {
                synchronized (this) {
                    if (opened == null) {
                        WorldData wd = terrain.data();
                        RasterSampler strata = wd.has(WorldData.HORIZONS) ? wd.layer(WorldData.HORIZONS) : null;
                        opened = new Opened(wd, ClimateLayers.of(wd), wd.has("climate/koppen") ? wd.layer("climate/koppen") : null,
                                wd.iceCover(), wd.has(Playa.LAYER) ? wd.layer(Playa.LAYER) : null,
                                wd.has(WETLAND_LAYER) ? wd.layer(WETLAND_LAYER) : null, strata,
                                strata == null ? List.of() : wd.classes(), OverworldBands.defaults().erosion()[6],
                                wd.manifest().mPerBlockV());
                    }
                    o = opened;
                }
            }
            return o;
        }
    }

    /** One column's facts, each computed on first use. */
    static final class Column implements BiomeRuleSet.Facts {
        private final LevelData data;
        private final int x, z, seaLevel;
        private final double height;
        private final java.util.function.IntSupplier roles;
        private final Supplier<Holder<Biome>> plain;
        private LevelData.Opened o;
        private String koppen;
        private ClimateLayers.Sample climate;
        private boolean climateRead;
        private Wetland wetland;
        private String rock;
        private Holder<Biome> plainBiome;

        Column(LevelData data, int x, int z, double height, int seaLevel, java.util.function.IntSupplier roles,
               Supplier<Holder<Biome>> plain) {
            this.data = data;
            this.x = x;
            this.z = z;
            this.height = height;
            this.seaLevel = seaLevel;
            this.roles = roles;
            this.plain = plain;
        }

        private LevelData.Opened o() {
            if (o == null)
                o = data.open();
            return o;
        }

        @Override
        public String koppen() {
            if (koppen == null) {
                LevelData.Opened d = o();
                if (d.koppen() == null) {
                    koppen = "";
                } else {
                    double k = d.koppen().nearest(x, z);
                    int ki = Double.isNaN(k) ? ClimateLayers.SEA : (int) Math.max(0, Math.min(ClimateLayers.SEA, Math.round(k)));
                    koppen = d.climate() != null ? d.climate().code(ki) : BiomeCensus.KOPPEN[ki];
                }
            }
            return koppen;
        }

        private ClimateLayers.Sample climate() {
            if (!climateRead) {
                climateRead = true;
                ClimateLayers c = o().climate();
                climate = c == null ? null : c.sample(x, (int) Math.floor(height), z);
            }
            return climate;
        }

        @Override
        public double tMean() {
            var c = climate();
            return c == null ? Double.NaN : c.tMean();
        }

        @Override
        public double tWarm() {
            var c = climate();
            return c == null ? Double.NaN : c.tWarm();
        }

        @Override
        public double tCold() {
            var c = climate();
            return c == null ? Double.NaN : c.tCold();
        }

        @Override
        public double pMm() {
            var c = climate();
            return c == null ? Double.NaN : c.pAnnual();
        }

        @Override
        public double elevationM() {
            return (height - seaLevel) * o().mPerBlockV();
        }

        @Override
        public Wetland wetland() {
            if (wetland == null) {
                LevelData.Opened d = o();
                if (d.wetland() != null) {
                    double v = d.wetland().nearest(x, z);
                    wetland = v >= 1.5 ? Wetland.COLD : v >= 0.5 ? Wetland.WARM : Wetland.NONE;
                } else if (d.wd().raster(ClimateMapping.EROSION, x, z) >= d.e6()) {
                    // A world without the flag: the bake's wetland rule puts wet, flat, low land in the swamp band (BIO-066).
                    wetland = COLD_WETLAND_CLASSES.contains(koppen()) ? Wetland.COLD : Wetland.WARM;
                } else {
                    wetland = Wetland.NONE;
                }
            }
            return wetland;
        }

        @Override
        public boolean ice() {
            var ice = o().ice();
            return ice.isPresent() && ice.get().blocks(x, z) > 0;
        }

        @Override
        public boolean playa() {
            RasterSampler p = o().playa();
            return p != null && p.nearest(x, z) > 0;
        }

        @Override
        public boolean volcanic() {
            return roles.getAsInt() != 0;
        }

        @Override
        public String topRock() {
            if (rock == null) {
                LevelData.Opened d = o();
                rock = "";
                if (d.strata() != null) {
                    StrataPass.Column col = new StrataPass.Column(Math.max(1, d.strata().stackDepth()));
                    StrataPass.column(d.wd(), d.strata(), x, z, col);
                    int ground = (int) Math.floor(height);
                    for (int h = 0; h < col.count; h++) {
                        if (col.tops[h] < ground - ROCK_WINDOW)
                            break;
                        int cls = col.classes[h];
                        if (cls >= 0 && cls < d.classes().size()) {
                            rock = d.classes().get(cls);
                            break;
                        }
                    }
                }
            }
            return rock;
        }

        @Override
        public boolean plainIn(List<String> ids) {
            if (plainBiome == null)
                plainBiome = plain.get();
            return plainBiome != null && in(plainBiome, ids);
        }
    }
}
