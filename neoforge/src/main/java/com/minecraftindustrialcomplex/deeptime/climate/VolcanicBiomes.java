// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones.Role;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.core.QuartPos;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Consumer;

/**
 * The pack's volcanic biomes, tied to real volcanoes (Ben, decisions log 2026-10-03, "Tie them to real
 * volcanoes (Recommended)"). A Deep Time level's biome query goes through here (the mixins
 * {@code mixin.VolcanicBiomeSourceMixin} and, with TerraBlender, {@code mixin.compat.terrablender.RegionBiomeSourceMixin}):
 *
 * <ol>
 *   <li><b>Kept out.</b> A biome in {@link #KEEP_OUT} ({@code #deeptime:volcanic}) is never the answer of the
 *       climate search: the search runs on the biome source's own R-tree with a distance metric that puts every
 *       such leaf infinitely far away, so the next-nearest biome of the same tree answers (the pack's own
 *       choice for that climate point once its volcanic biomes are set aside; no hole, no fallback biome).</li>
 *   <li><b>Put in.</b> Where a {@link VolcanicZones} disc of a {@link Role} holds the column, the role's biome
 *       answers: the first biome of the tag {@code #deeptime:volcanic/<role>} (in the tag's order) that the
 *       biome source can place. Surface roles answer at and above the ground (biome depth parameter under
 *       {@link #SURFACE_DEPTH}) on land, never on a river's channel; the climate-gated roles (fresh ash, young
 *       lava) only where the climate point lies inside that biome's own temperature and humidity ranges in the
 *       source's parameter list; the cave role from that biome's own lowest depth down. A role without a
 *       biome in the pack, or not allowed there, gives way to the next role whose disc holds the column, and
 *       in the end to the climate search: without the pack's volcanic biomes (vanilla) nothing changes.</li>
 * </ol>
 *
 * <p>Both lists are datapack tags, so any mod's biomes can join or leave them. The tag order is read from
 * the server's biome registry; the resolved choice is cached per level and parameter list and dropped when
 * tags reload ({@link #tagsUpdated()}).
 */
public final class VolcanicBiomes {

    private VolcanicBiomes() {}

    /** The biomes kept out of every place but their volcanic roles. */
    public static final TagKey<Biome> KEEP_OUT = TagKey.create(Registries.BIOME,
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "volcanic"));

    /** A surface role answers where the biome depth parameter is below this (vanilla's cave biomes begin at 0.2). */
    public static final float SURFACE_DEPTH = 0.2f; // ledger: BIO-112 (vanilla OverworldBiomeBuilder, verified)

    private static final long SURFACE_DEPTH_Q = Climate.quantizeCoord(SURFACE_DEPTH);

    /** The tag of a role's biomes, in order of preference. */
    public static TagKey<Biome> tag(Role role) {
        return TagKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "volcanic/" + role.id));
    }

    private static final AtomicInteger TAGS = new AtomicInteger();

    /** Tags were (re)loaded: every level resolves its role biomes again. */
    public static void tagsUpdated() {
        TAGS.incrementAndGet();
    }

    /**
     * The census asks for the biome as master placed it (no keep-out, no roles) on this thread while
     * this is set; see {@link #plain(java.util.function.Supplier)}.
     */
    private static final ThreadLocal<Boolean> PLAIN = ThreadLocal.withInitial(() -> false);

    /** {@code query} answered as if this feature did not exist (for the census's "before"). */
    public static <T> T plain(java.util.function.Supplier<T> query) {
        boolean was = PLAIN.get();
        PLAIN.set(true);
        try {
            return query.get();
        } finally {
            PLAIN.set(was);
        }
    }

    /** Whether the current thread asked for the plain search. */
    public static boolean plainRequested() {
        return PLAIN.get();
    }

    /** The plain metric: vanilla's. */
    public static final Climate.DistanceMetric<Holder<Biome>> NEAREST = Climate.RTree.Node::distance;

    /** The keep-out metric: a leaf whose biome is in {@link #KEEP_OUT} is infinitely far away. */
    public static final Climate.DistanceMetric<Holder<Biome>> NEAREST_KEPT = (node, values) -> {
        if (node == null)
            return Long.MAX_VALUE;
        if (node instanceof Climate.RTree.Leaf<Holder<Biome>> leaf && leaf.value.is(KEEP_OUT))
            return Long.MAX_VALUE;
        return node.distance(values);
    };

    /** One entry of a biome source's parameter list: the ranges a biome answers in. */
    public record Entry(Climate.Parameter temperature, Climate.Parameter humidity, Climate.Parameter depth, Holder<Biome> biome) {}

    /** What a biome source offers: its search, its biomes and its entries. */
    public interface Source {
        /** Identifies the parameter list (the cache key). */
        Object key();

        /** The nearest biome of the climate point under {@code metric}. */
        Holder<Biome> find(Climate.TargetPoint target, Climate.DistanceMetric<Holder<Biome>> metric);

        /** Every biome the source can place. */
        Set<Holder<Biome>> possibleBiomes();

        /** Every entry the source can answer from. */
        void entries(Consumer<Entry> sink);

        /**
         * The vanilla tree under this one, if the source has one apart from itself (TerraBlender's region 0 under a
         * mod's region): where keeping out a mod's volcanic land biome would leave the sea, the slot takes this
         * tree's biome.
         */
        default Optional<Source> base() {
            return Optional.empty();
        }
    }

    /**
     * What one level resolved against one parameter list. {@code gated}: the geology gate's biomes the source can
     * place and the ground role bits each may stand on ({@link GeologyBiomes}); {@code metrics}: the search's metric
     * per ground role bits, made on first use.
     */
    private record Resolved(Object key, int tags, boolean active, boolean keepOut, Holder<Biome>[] byRole,
                            Map<Holder<Biome>, List<Entry>> entries, Map<Holder<Biome>, Integer> gated,
                            java.util.concurrent.atomic.AtomicReferenceArray<Climate.DistanceMetric<Holder<Biome>>> metrics) {

        /**
         * The keep-out metric of a column whose ground has the role bits {@code ground}: a gated leaf is infinitely far
         * unless its roles meet the ground's (and then counts as any leaf, even if it is volcanic too); any other kept-out
         * (volcanic) leaf is infinitely far.
         */
        Climate.DistanceMetric<Holder<Biome>> metric(int ground) {
            if (gated.isEmpty())
                return NEAREST_KEPT;
            Climate.DistanceMetric<Holder<Biome>> m = metrics.get(ground);
            if (m != null)
                return m;
            boolean out = keepOut;
            m = (node, values) -> {
                if (node == null)
                    return Long.MAX_VALUE;
                if (node instanceof Climate.RTree.Leaf<Holder<Biome>> leaf) {
                    Integer roles = gated.get(leaf.value);
                    if (roles != null)
                        return (roles & ground) != 0 ? node.distance(values) : Long.MAX_VALUE;
                    if (out && leaf.value.is(KEEP_OUT))
                        return Long.MAX_VALUE;
                }
                return node.distance(values);
            };
            metrics.set(ground, m);
            return m;
        }
    }

    /**
     * One Deep Time level's state (held by its {@code RegionSpawnTarget}): its terrain and the volcanic zones,
     * built on first use, and the role biomes resolved for its biome source.
     */
    public static final class Level {
        private final WorldDataTerrain terrain;
        private final int cell, minY, seaLevel;
        private volatile VolcanicZones zones;
        private volatile com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround geology;
        private volatile Resolved resolved;

        public Level(WorldDataTerrain terrain, int cell, int minY, int seaLevel) {
            this.terrain = terrain;
            this.cell = cell;
            this.minY = minY;
            this.seaLevel = seaLevel;
        }

        public WorldDataTerrain terrain() {
            return terrain;
        }

        /** The level's zones (built on first use). */
        public VolcanicZones zones() {
            VolcanicZones z = zones;
            if (z == null) {
                z = terrain.volcanicZones(cell, minY, seaLevel);
                zones = z;
            }
            return z;
        }

        public int seaLevel() {
            return seaLevel;
        }

        /** The level's geology gate ground (built on first use, with the server's rock classes). */
        public com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround geology() {
            var g = geology;
            if (g == null) {
                var server = net.neoforged.neoforge.server.ServerLifecycleHooks.getCurrentServer();
                Map<String, List<String>> groups = new HashMap<>();
                if (server != null)
                    com.minecraftindustrialcomplex.deeptime.rock.RockClassRegistry.byName(server.registryAccess())
                            .forEach((n, rc) -> groups.put(n, rc.groups()));
                g = new com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround(terrain, seaLevel,
                        com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround.groupsOf(groups));
                geology = g;
            }
            return g;
        }

        /**
         * Whether a biome of the geology gate may stand at block (x, z) of this level: true for a biome in no geology
         * role, else whether the column's ground has one of its roles. False before the level's first query.
         */
        public boolean geologyAllows(Holder<Biome> biome, int x, int z) {
            Resolved r = resolved;
            if (r == null)
                return false;
            Integer roles = r.gated.get(biome);
            return roles == null || (roles & geology().rolesAt(x, z)) != 0;
        }

        /** The geology roles a biome may stand on at this level (0: not gated, or before the level's first query). */
        public int gatedRoles(Holder<Biome> biome) {
            Resolved r = resolved;
            return r == null ? 0 : r.gated.getOrDefault(biome, 0);
        }

        /**
         * The biome a role puts at this level, as last resolved by a biome query; empty when the pack has none
         * for it, or before the level's first query.
         */
        public Optional<Holder<Biome>> roleBiome(Role role) {
            Resolved r = resolved;
            return r == null ? Optional.empty() : Optional.ofNullable(r.byRole[role.ordinal()]);
        }

        /** Whether the last resolution found anything to keep out or to place (false: every query is vanilla's). */
        public boolean active() {
            Resolved r = resolved;
            return r != null && r.active;
        }

        private Resolved resolve(Source source) {
            Resolved r = resolved;
            int tags = TAGS.get();
            if (r != null && r.key == source.key() && r.tags == tags)
                return r;
            synchronized (this) {   // once per level and tag load, not once per worldgen thread
                r = resolved;
                tags = TAGS.get();
                if (r != null && r.key == source.key() && r.tags == tags)
                    return r;
                r = build(source, tags);
                resolved = r;
                return r;
            }
        }
    }

    @SuppressWarnings("unchecked")
    private static Resolved build(Source source, int tags) {
        Set<Holder<Biome>> possible = source.possibleBiomes();
        int kept = 0;
        for (Holder<Biome> b : possible)
            if (b.is(KEEP_OUT))
                kept++;
        // Keep-out never empties the search (a tree of nothing but volcanic biomes would answer null).
        boolean keepOut = kept > 0 && kept < possible.size();
        Holder<Biome>[] byRole = new Holder[Role.values().length];
        Optional<Registry<Biome>> registry = biomeRegistry();
        boolean any = false;
        if (registry.isPresent())
            for (Role role : Role.values()) {
                Optional<HolderSet.Named<Biome>> set = registry.get().getTag(tag(role));
                if (set.isEmpty())
                    continue;
                for (Holder<Biome> b : set.get())
                    if (possible.contains(b)) {
                        byRole[role.ordinal()] = b;
                        any = true;
                        break;
                    }
            }
        Map<Holder<Biome>, List<Entry>> entries = new HashMap<>();
        if (any) {
            Set<Holder<Biome>> wanted = new java.util.HashSet<>();
            for (Holder<Biome> b : byRole)
                if (b != null)
                    wanted.add(b);
            source.entries(e -> {
                if (wanted.contains(e.biome()))
                    entries.computeIfAbsent(e.biome(), k -> new ArrayList<>()).add(e);
            });
        }
        StringBuilder roles = new StringBuilder();
        for (Role role : Role.values())
            roles.append(' ').append(role.id).append('=').append(byRole[role.ordinal()] == null ? "-"
                    : byRole[role.ordinal()].unwrapKey().map(k -> k.location().toString()).orElse("?"));
        // The geology gate (decisions log 2026-10-04): the rock-named biomes this source can place, and their roles. Like
        // the keep-out, never every biome of the tree.
        Map<Holder<Biome>, Integer> gated = GeologyBiomes.gated(registry, possible);
        if (gated.size() >= possible.size())
            gated = Map.of();
        if (registry.isPresent()) {
            DeepTime.LOGGER.info("[deeptime] volcanic biomes: {} of {} biomes kept out of the climate search; roles:{}", kept,
                    possible.size(), roles);
            if (!gated.isEmpty()) {
                StringBuilder g = new StringBuilder();
                gated.forEach((b, bits) -> g.append(' ').append(b.unwrapKey().map(k -> k.location().toString()).orElse("?"))
                        .append('=').append(GeologyBiomes.names(bits)));
                DeepTime.LOGGER.info("[deeptime] geology gate: {} rock-named biomes kept to their ground:{}", gated.size(), g);
            }
        }
        // Without a registry the roles are retried on the next query.
        return new Resolved(source.key(), registry.isPresent() ? tags : -1, keepOut || any || !gated.isEmpty(), keepOut, byRole,
                entries, gated, new java.util.concurrent.atomic.AtomicReferenceArray<>(1 << GeologyGround.Role.values().length));
    }

    private static Optional<Registry<Biome>> biomeRegistry() {
        var server = net.neoforged.neoforge.server.ServerLifecycleHooks.getCurrentServer();
        if (server == null)
            return Optional.empty();
        return server.registryAccess().registry(Registries.BIOME);
    }

    /**
     * The biome at quart (qx, qy, qz) of a Deep Time level, or null when this level's source has neither a
     * biome to keep out nor a role biome (the caller then answers as vanilla does, unchanged).
     */
    public static Holder<Biome> find(Level level, Source source, int qx, int qy, int qz, Climate.Sampler sampler) {
        if (PLAIN.get())
            return null;
        Resolved r = level.resolve(source);
        if (!r.active)
            return null;
        Climate.TargetPoint t = sampler.sample(qx, qy, qz);
        Holder<Biome> b = role(level, r, qx, qz, t);
        if (b != null)
            return b;
        // The plain nearest first: where it is not kept out, the keep-out search would find the same leaf.
        Holder<Biome> plain = source.find(t, NEAREST);
        Integer gatedRoles = r.gated.isEmpty() ? null : r.gated.get(plain);
        int ground = 0;
        if (gatedRoles != null) {
            // A rock-named biome: on its ground it answers (even when it is volcanic too), elsewhere it is kept out.
            ground = level.geology().rolesAt(QuartPos.toBlock(qx) + 2, QuartPos.toBlock(qz) + 2);
            if ((gatedRoles & ground) != 0)
                return plain;
        } else if (!r.keepOut || !plain.is(KEEP_OUT)) {
            return plain;
        } else if (!r.gated.isEmpty()) {
            ground = level.geology().rolesAt(QuartPos.toBlock(qx) + 2, QuartPos.toBlock(qz) + 2);
        }
        Holder<Biome> kept;
        try {
            kept = source.find(t, r.metric(ground));
        } catch (NullPointerException emptyTree) {
            // A tree holding nothing but kept-out biomes (a region of volcanic biomes alone) answers null.
            return plain;
        }
        // A mod that put its volcanic land biome in the slot of a land biome surrounded by sea (Biomes O' Plenty's
        // volcanic plains in vanilla's mushroom-fields slot) leaves the sea as the next-nearest biome of its tree:
        // that slot takes the biome the vanilla tree has there instead, so land does not turn into ocean.
        if (water(kept) && !water(plain)) {
            Optional<Source> base = source.base();
            if (base.isPresent()) {
                try {
                    Holder<Biome> under = base.get().find(t, r.metric(ground));
                    if (!water(under))
                        return under;
                } catch (NullPointerException emptyTree) {
                    // keep the region's answer
                }
            }
        }
        return kept;
    }

    private static boolean water(Holder<Biome> b) {
        return b.is(net.minecraft.tags.BiomeTags.IS_OCEAN) || b.is(net.minecraft.tags.BiomeTags.IS_RIVER);
    }

    /** The role biome at the column of quart (qx, qz) for climate point t, or null. */
    private static Holder<Biome> role(Level level, Resolved r, int qx, int qz, Climate.TargetPoint t) {
        boolean any = false;
        for (Holder<Biome> b : r.byRole)
            any |= b != null;
        if (!any)
            return null;
        int x = QuartPos.toBlock(qx) + 2, z = QuartPos.toBlock(qz) + 2;
        int bits = level.zones().rolesAt(x, z);
        if (bits == 0)
            return null;
        boolean surface = t.depth() < SURFACE_DEPTH_Q;
        Boolean land = null;
        for (Role role : Role.values()) {
            if ((bits & (1 << role.ordinal())) == 0)
                continue;
            Holder<Biome> b = r.byRole[role.ordinal()];
            if (b == null)
                continue;
            if (role.cave) {
                if (surface)
                    continue;
                long low = lowestDepth(r.entries.get(b));
                if (low == Long.MAX_VALUE || t.depth() < low)
                    continue;
                return b;
            }
            if (!surface)
                continue;
            // A river channel keeps its river biome (the variant input is exactly 0 there).
            if (t.weirdness() == 0L)
                return null;
            if (land == null)
                land = level.terrain().height(x, z) >= level.seaLevel();
            if (!land)
                return null;
            if (role.climateGated && !fits(r.entries.get(b), t))
                continue;
            return b;
        }
        return null;
    }

    private static long lowestDepth(List<Entry> entries) {
        long low = Long.MAX_VALUE;
        if (entries != null)
            for (Entry e : entries)
                low = Math.min(low, e.depth().min());
        return low;
    }

    /** The climate point lies inside one of the biome's own (temperature, humidity) boxes. */
    private static boolean fits(List<Entry> entries, Climate.TargetPoint t) {
        if (entries == null)
            return false;
        for (Entry e : entries)
            if (t.temperature() >= e.temperature().min() && t.temperature() <= e.temperature().max()
                    && t.humidity() >= e.humidity().min() && t.humidity() <= e.humidity().max())
                return true;
        return false;
    }

    /** The source of a plain {@code MultiNoiseBiomeSource} (no TerraBlender): its parameter list. */
    public record ListSource(net.minecraft.world.level.biome.BiomeSource biomes, Climate.ParameterList<Holder<Biome>> list)
            implements Source {
        @Override
        public Object key() {
            return list;
        }

        @Override
        public Holder<Biome> find(Climate.TargetPoint target, Climate.DistanceMetric<Holder<Biome>> metric) {
            return list.findValueIndex(target, metric);
        }

        @Override
        public Set<Holder<Biome>> possibleBiomes() {
            return biomes.possibleBiomes();
        }

        @Override
        public void entries(Consumer<Entry> sink) {
            for (var p : list.values())
                sink.accept(new Entry(p.getFirst().temperature(), p.getFirst().humidity(), p.getFirst().depth(), p.getSecond()));
        }
    }

    /** Every leaf of an R-tree, as entries (TerraBlender's region trees hold biomes its list does not). */
    public static void leaves(Climate.RTree<Holder<Biome>> tree, Consumer<Entry> sink) {
        walk(tree.root, sink);
    }

    private static void walk(Climate.RTree.Node<Holder<Biome>> node, Consumer<Entry> sink) {
        if (node instanceof Climate.RTree.Leaf<Holder<Biome>> leaf) {
            Climate.Parameter[] p = leaf.parameterSpace;
            sink.accept(new Entry(p[0], p[1], p[4], leaf.value));
        } else if (node instanceof Climate.RTree.SubTree<Holder<Biome>> sub) {
            for (Climate.RTree.Node<Holder<Biome>> child : sub.children)
                walk(child, sink);
        }
    }
}
