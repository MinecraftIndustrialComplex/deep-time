// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.commands.arguments.blocks.BlockStateParser;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Property;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.TagsUpdatedEvent;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;
import java.util.function.Predicate;

/**
 * Rock class → blocks (plan §7.5): the rules of {@link PaletteRules} applied to
 * {@code #deeptime:rock/<class>}, {@code #deeptime:rock_fallback/<class>}, the
 * {@code deeptime:rock_states} registry and each {@link RockClass}'s {@code vanilla_fallback}.
 *
 * <p>Rebuilt whenever the server (re)loads its data: NeoForge posts {@link TagsUpdatedEvent}
 * ({@code SERVER_DATA_LOAD}) at server start, after the tags are bound, and on {@code /reload}.
 * Client tag syncs are ignored (in singleplayer the client shares this JVM, and the rock registries
 * are not synced). A snapshot is immutable; chunk generation reads {@link #get()} once per chunk.
 */
public final class RockPalette {

    public static final String TAG_MEMBERS = "rock/";
    public static final String TAG_FALLBACK = "rock_fallback/";

    private static final RockPalette EMPTY = new RockPalette(Map.of(), Map.of(), Map.of());
    private static volatile RockPalette current = EMPTY;

    /** What a block is to the deposit feature and background scatter (P1-15). */
    public enum Host {
        /** Not rock: air, fluids, ore, plants, anything no rock class resolves to. */
        NONE,
        /** Unconsolidated rock (the {@code unconsolidated} family: soil, sand, gravel, clay, …). */
        LOOSE,
        /** Bedrock: the unclassified crust (stone, deepslate) or any other class's block. */
        ROCK
    }

    /** Class name → resolution, ordered by class name. */
    private final Map<String, PaletteRules.Result<BlockState>> byClass;
    /** Problems found while building (unknown classes in rock_states, bad states, missing blocks). */
    private final Map<String, List<String>> problems;
    /** Block → the classes whose resolution contains it (sorted), for host checks and scatter bias. */
    private final Map<Block, List<String>> classesOfBlock;
    /** Block → host kind (ROCK wins where a block serves several classes). */
    private final Map<Block, Host> hostOfBlock;
    /** Class → family ({@link RockClass#family()}). */
    private final Map<String, String> families;

    private RockPalette(Map<String, PaletteRules.Result<BlockState>> byClass, Map<String, List<String>> problems,
                        Map<String, String> families) {
        this.byClass = byClass;
        this.problems = problems;
        this.families = Map.copyOf(families);
        Map<Block, List<String>> classes = new java.util.HashMap<>();
        Map<Block, Host> hosts = new java.util.HashMap<>();
        byClass.forEach((name, r) -> {
            Host kind = "unconsolidated".equals(families.get(name)) ? Host.LOOSE : Host.ROCK;
            for (BlockState b : r.blocks()) {
                classes.computeIfAbsent(b.getBlock(), k -> new ArrayList<>()).add(name);
                hosts.merge(b.getBlock(), kind, (a, c) -> a == Host.ROCK || c == Host.ROCK ? Host.ROCK : Host.LOOSE);
            }
        });
        classes.replaceAll((k, v) -> List.copyOf(v));
        this.classesOfBlock = Map.copyOf(classes);
        this.hostOfBlock = Map.copyOf(hosts);
    }

    /** True for a class of the {@code unconsolidated} family (sand, gravel, soil, …). */
    public boolean isLoose(String rockClass) {
        return "unconsolidated".equals(families.get(rockClass));
    }

    /**
     * What {@code s} is as a host for ore: the unclassified crust (stone, deepslate) and every
     * class's blocks are rock, the unconsolidated classes' blocks loose rock, anything else none.
     */
    public Host host(BlockState s) {
        if (s.is(Blocks.STONE) || s.is(Blocks.DEEPSLATE))
            return Host.ROCK;
        return hostOfBlock.getOrDefault(s.getBlock(), Host.NONE);
    }

    /**
     * The rock classes {@code s} is a block of (several where classes share a stand-in), sorted;
     * {@code unclassified_crust} for stone and deepslate; empty for non-rock.
     */
    public List<String> classesOf(BlockState s) {
        if (s.is(Blocks.STONE) || s.is(Blocks.DEEPSLATE))
            return List.of("unclassified_crust");
        return classesOfBlock.getOrDefault(s.getBlock(), List.of());
    }

    public static void init() {
        NeoForge.EVENT_BUS.addListener(TagsUpdatedEvent.class, e -> {
            if (e.getUpdateCause() != TagsUpdatedEvent.UpdateCause.SERVER_DATA_LOAD)
                return;
            RockPalette p = build(e.getRegistryAccess(), View.LIVE);
            current = p;
            p.log();
        });
    }

    /** The current palette (empty before the first server data load). */
    public static RockPalette get() {
        return current;
    }

    /** Which inputs a palette uses. */
    public record View(Predicate<ResourceLocation> memberFilter, boolean fallbackTags, boolean rockStates) {
        /** Everything: the palette the generator uses. */
        public static final View LIVE = new View(id -> true, true, true);
        /**
         * What a pure-vanilla pack would see: only {@code minecraft:} members of the member tags,
         * no fallback tags and no {@code rock_states}, so only the vanilla member tags and each
         * class's {@code vanilla_fallback} remain (plan §7.5, Q15).
         */
        public static final View VANILLA_ONLY = new View(id -> id.getNamespace().equals("minecraft"), false, false);
    }

    /** Builds a palette from a server's registries (tags must be bound). */
    public static RockPalette build(RegistryAccess access, View view) {
        Map<String, RockClass> classes = RockClassRegistry.byName(access);
        Map<String, List<String>> problems = new TreeMap<>();
        Map<String, List<BlockState>> stateMembers = new LinkedHashMap<>(), stateFallbacks = new LinkedHashMap<>();
        if (view.rockStates()) {
            for (var named : RockStatesRegistry.entries(access)) {
                RockStates rs = named.states();
                if (!classes.containsKey(rs.rockClass()))
                    problems.computeIfAbsent(rs.rockClass(), k -> new ArrayList<>())
                            .add(named.id() + ": rock_states for a class that has no rock_class definition");
                var into = (rs.use() == RockStates.Use.MEMBER ? stateMembers : stateFallbacks)
                        .computeIfAbsent(rs.rockClass(), k -> new ArrayList<>());
                for (RockStates.Entry entry : rs.entries())
                    state(entry, named.id(), problems.computeIfAbsent(rs.rockClass(), k -> new ArrayList<>())).ifPresent(into::add);
            }
        }
        PaletteRules.Kinds<BlockState> kinds = new PaletteRules.Kinds<>() {
            @Override
            public boolean isStone(BlockState b) {
                return b.is(Blocks.STONE);
            }

            @Override
            public boolean isFiller(BlockState b) {
                return b.is(Blocks.STONE) || b.is(Blocks.DEEPSLATE);
            }

            @Override
            public boolean isRejected(BlockState b) {
                return b.isAir();
            }
        };
        Map<String, PaletteRules.Result<BlockState>> out = new TreeMap<>();
        classes.forEach((name, rc) -> {
            List<String> own = problems.computeIfAbsent(name, k -> new ArrayList<>());
            Optional<BlockState> vanilla = rc.vanillaFallback().flatMap(id -> {
                Optional<Block> b = BuiltInRegistries.BLOCK.getOptional(id);
                if (b.isEmpty())
                    own.add("vanilla_fallback " + id + " is not a block");
                return b.map(Block::defaultBlockState);
            });
            var candidates = new PaletteRules.Candidates<>(
                    tag(access, TAG_MEMBERS + name, view.memberFilter()),
                    stateMembers.getOrDefault(name, List.of()),
                    view.fallbackTags() ? tag(access, TAG_FALLBACK + name, id -> true) : List.of(),
                    stateFallbacks.getOrDefault(name, List.of()),
                    vanilla);
            var result = PaletteRules.resolve(name, candidates, kinds, BlockStateParser::serialize);
            own.addAll(result.rejected());
            out.put(name, result);
        });
        problems.values().removeIf(List::isEmpty);
        Map<String, String> families = new TreeMap<>();
        classes.forEach((name, rc) -> families.put(name, rc.family()));
        return new RockPalette(java.util.Collections.unmodifiableMap(out), java.util.Collections.unmodifiableMap(problems),
                families);
    }

    /** A tag's blocks in list order (absent optional entries were dropped when the tag loaded). */
    private static List<BlockState> tag(RegistryAccess access, String path, Predicate<ResourceLocation> filter) {
        TagKey<Block> key = TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, path));
        List<BlockState> out = new ArrayList<>();
        access.registryOrThrow(Registries.BLOCK).getTag(key).ifPresent(set -> {
            for (Holder<Block> h : set)
                if (h.unwrapKey().map(k -> filter.test(k.location())).orElse(false))
                    out.add(h.value().defaultBlockState());
        });
        return out;
    }

    /** A rock_states entry's block state; empty for an absent optional block or a bad entry (reported). */
    static Optional<BlockState> state(RockStates.Entry entry, ResourceLocation file, List<String> problems) {
        Optional<Block> block = BuiltInRegistries.BLOCK.getOptional(entry.block());
        if (block.isEmpty()) {
            if (entry.required())
                problems.add(file + ": required block " + entry.block() + " is not installed");
            return Optional.empty();
        }
        BlockState s = block.get().defaultBlockState();
        for (var p : entry.state().entrySet()) {
            Property<?> property = block.get().getStateDefinition().getProperty(p.getKey());
            if (property == null) {
                problems.add(file + ": " + entry.block() + " has no property '" + p.getKey() + "'");
                return Optional.empty();
            }
            Optional<BlockState> with = with(s, property, p.getValue());
            if (with.isEmpty()) {
                problems.add(file + ": " + entry.block() + " property '" + p.getKey() + "' has no value '" + p.getValue() + "'");
                return Optional.empty();
            }
            s = with.get();
        }
        return Optional.of(s);
    }

    private static <T extends Comparable<T>> Optional<BlockState> with(BlockState s, Property<T> property, String value) {
        return property.getValue(value).map(v -> s.setValue(property, v));
    }

    // -------------------------------------------------------------------------------------------

    /** Every class name the palette knows (the {@code rock_class} registry), sorted. */
    public java.util.Set<String> classes() {
        return byClass.keySet();
    }

    public Optional<PaletteRules.Result<BlockState>> result(String rockClass) {
        return Optional.ofNullable(byClass.get(rockClass));
    }

    /** The primary block of a class; empty if it resolves to nothing or is unknown. */
    public Optional<BlockState> primary(String rockClass) {
        var r = byClass.get(rockClass);
        return r == null ? Optional.empty() : r.primary();
    }

    /**
     * The block for the horizon or body with {@code key} of class {@code rockClass}; null when the
     * class is unknown or resolves to nothing (it stays unclassified crust).
     */
    public BlockState block(String rockClass, long key) {
        var r = byClass.get(rockClass);
        return r == null ? null : r.pick(key);
    }

    /** Resolutions by class index, for a world's {@code strata/classes.json} list (null = unknown). */
    @SuppressWarnings("unchecked")
    public PaletteRules.Result<BlockState>[] forClasses(List<String> classIds) {
        PaletteRules.Result<BlockState>[] out = new PaletteRules.Result[classIds.size()];
        for (int i = 0; i < out.length; i++)
            out[i] = byClass.get(classIds.get(i));
        return out;
    }

    public List<String> unresolved() {
        return byClass.values().stream().filter(r -> r.source() == PaletteRules.Source.NONE).map(PaletteRules.Result::rockClass).toList();
    }

    public Map<String, List<String>> problems() {
        return problems;
    }

    /**
     * The probe's JSON (plan §7.16): {@code classes} (class → its blocks as block-state strings, the
     * primary first; the shape {@code deeptime-inspect rock --palette} reads), {@code source}
     * (member, fallback, vanilla_fallback or none), {@code state} (the primary block state),
     * {@code unresolved} and {@code problems}.
     */
    public JsonObject toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("count", byClass.size());
        o.addProperty("resolved", byClass.size() - unresolved().size());
        JsonObject counts = new JsonObject();
        for (PaletteRules.Source s : PaletteRules.Source.values())
            counts.addProperty(s.id(), byClass.values().stream().filter(r -> r.source() == s).count());
        o.add("sources", counts);
        JsonObject classes = new JsonObject(), source = new JsonObject(), state = new JsonObject();
        byClass.forEach((name, r) -> {
            JsonArray blocks = new JsonArray();
            r.blocks().forEach(b -> blocks.add(BlockStateParser.serialize(b)));
            classes.add(name, blocks);
            source.addProperty(name, r.source().id());
            r.primary().ifPresent(b -> state.addProperty(name, BlockStateParser.serialize(b)));
        });
        o.add("classes", classes);
        o.add("source", source);
        o.add("state", state);
        JsonArray unresolved = new JsonArray();
        unresolved().forEach(unresolved::add);
        o.add("unresolved", unresolved);
        JsonObject probs = new JsonObject();
        problems.forEach((k, v) -> {
            JsonArray a = new JsonArray();
            v.forEach(a::add);
            probs.add(k, a);
        });
        o.add("problems", probs);
        return o;
    }

    private void log() {
        var counts = toJson().getAsJsonObject("sources");
        DeepTime.LOGGER.info("Rock palette: {} classes ({} member, {} fallback, {} vanilla_fallback, {} unresolved{})",
                byClass.size(), counts.get("member"), counts.get("fallback"), counts.get("vanilla_fallback"),
                counts.get("none"), unresolved().isEmpty() ? "" : ": " + unresolved());
        problems.forEach((cls, list) -> list.forEach(p -> DeepTime.LOGGER.warn("Rock palette, class '{}': {}", cls, p)));
    }
}
