// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules.Source;
import com.minecraftindustrialcomplex.deeptime.rock.RockClassRegistry;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.server.DeepTimeCommands;
import com.minecraftindustrialcomplex.deeptime.worldgen.StrataPass;
import net.minecraft.commands.arguments.blocks.BlockStateParser;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.TreeSet;

/**
 * Rock tags, {@code rock_states} and the strata pass (P1-12, plan §7.5). Every test runs in every
 * profile; what it expects depends on what is installed:
 *
 * <ul>
 *   <li><b>No rock datapack</b> ({@code ./gametest.sh}): Deep Time's own vanilla member tags and each
 *       class's {@code vanilla_fallback} (Deep Time ships no fallback tags, Q15).</li>
 *   <li><b>The MIC rock tags</b> ({@code -PwithMic}, which adds the MIC rock-tag datapack through
 *       {@code -Ddeeptime.extraDatapack}): Create's stones and the other modded members in their
 *       classes, and the first <em>installed</em> entry of each fallback list.</li>
 * </ul>
 *
 * The gametest pack adds three test classes ({@code deeptime_test:test_fallback},
 * {@code test_state_fallback}, {@code test_states}) whose outcome is the same everywhere, and the
 * rock-tag README's {@code rock_states} example (Project Atmosphere's dust at {@code layers=8} as
 * loess, {@code required: false}).
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class StrataGameTests {

    private StrataGameTests() {}

    private static final String BIC = "born_in_chaos_v1", CREATE = "create", ICE_AND_FIRE = "iceandfire",
            ATMOSPHERE = "projectatmosphere";

    static boolean rockTags(GameTestHelper helper) {
        return GameTestSupport.extraDatapackActive(helper.getLevel().getServer());
    }

    static boolean loaded(String mod) {
        return ModList.get().isLoaded(mod);
    }

    static String id(BlockState s) {
        return BlockStateParser.serialize(s);
    }

    /** Collects mismatches, so one run reports every wrong class. */
    static final class Expect {
        final List<String> wrong = new ArrayList<>();
        int checked;

        void that(RockPalette p, String cls, Source source, String... blocks) {
            checked++;
            var r = p.result(cls).orElse(null);
            if (r == null) {
                wrong.add(cls + ": no such class");
                return;
            }
            List<String> got = r.blocks().stream().map(StrataGameTests::id).toList();
            if (r.source() != source || !got.equals(List.of(blocks)))
                wrong.add(cls + ": expected " + source.id() + " " + List.of(blocks) + ", got " + r.source().id() + " " + got);
        }

        /** Only the primary (first) block is checked. */
        void primary(RockPalette p, String cls, Source source, String block) {
            checked++;
            var r = p.result(cls).orElse(null);
            String got = r == null ? "no such class" : r.source().id() + " " + r.primary().map(StrataGameTests::id).orElse("nothing");
            if (r == null || r.source() != source || r.primary().map(StrataGameTests::id).filter(block::equals).isEmpty())
                wrong.add(cls + ": expected " + source.id() + " " + block + ", got " + got);
        }

        void done(GameTestHelper helper, String test) {
            DeepTime.LOGGER.info("[gametest] {} checked={} wrong={}{}", test, checked, wrong.size(), wrong.isEmpty() ? "" : " " + wrong);
            helper.assertTrue(wrong.isEmpty(), wrong.size() + " wrong: " + wrong);
            helper.succeed();
        }
    }

    /** Deep Time's own classes (the {@code deeptime} namespace of the rock_class registry). */
    static List<String> deepTimeClasses(GameTestHelper helper) {
        List<String> out = new ArrayList<>();
        RockClassRegistry.entries(helper.getLevel().registryAccess()).forEach((id, rc) -> {
            if (id.getNamespace().equals(DeepTime.MODID))
                out.add(rc.name());
        });
        return out;
    }

    /**
     * The classes whose {@code vanilla_fallback} is plain stone or deepslate (P1-28; Ben, decisions log
     * 2026-10-01, "Use stone for common rocks"): the common fine-grained grey sedimentary rocks are stone,
     * the common altered and deep ones deepslate; coal's stone is older (2026-09-27). The twin of
     * {@code CRUST_STAND_INS} in tools/dev/gen_rock_classes.py and of the JUnit {@code RockDataTest}.
     */
    static final Map<String, String> CRUST_STAND_INS = Map.of(
            "coal", "minecraft:stone", "shale", "minecraft:stone", "mudstone", "minecraft:stone",
            "slate", "minecraft:deepslate", "phyllite", "minecraft:deepslate", "schist", "minecraft:deepslate",
            "gneiss", "minecraft:deepslate");

    /** A block's default state as the palette prints it (deepslate is {@code deepslate[axis=y]}). */
    static String stateOf(String block) {
        return BlockStateParser.serialize(net.minecraft.core.registries.BuiltInRegistries.BLOCK
                .get(net.minecraft.resources.ResourceLocation.parse(block)).defaultBlockState());
    }

    /**
     * A pure-vanilla pack resolves all 60 classes: the palette built from only the {@code minecraft:}
     * members of the member tags plus each class's {@code vanilla_fallback} gives every class a
     * vanilla block, stone or deepslate for exactly the {@link #CRUST_STAND_INS} and for no other
     * class, and rejects nothing. Without a rock datapack the live palette is exactly that view.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void vanillaOnlyResolvesEveryClass(GameTestHelper helper) {
        var access = helper.getLevel().registryAccess();
        RockPalette vanilla = RockPalette.build(access, RockPalette.View.VANILLA_ONLY);
        RockPalette live = RockPalette.get();
        List<String> ours = deepTimeClasses(helper);
        Expect e = new Expect();
        if (ours.size() != 60)
            e.wrong.add("Deep Time ships " + ours.size() + " rock classes, not 60");
        Map<String, Integer> bySource = new TreeMap<>();
        for (String cls : ours) {
            e.checked++;
            var r = vanilla.result(cls).orElseThrow();
            bySource.merge(r.source().id(), 1, Integer::sum);
            if (r.source() == Source.NONE)
                e.wrong.add(cls + " resolves to nothing");
            for (BlockState b : r.blocks()) {
                if (!id(b).startsWith("minecraft:"))
                    e.wrong.add(cls + ": " + id(b) + " is not vanilla");
                if (StrataPass.isCrust(b) != CRUST_STAND_INS.containsKey(cls))
                    e.wrong.add(cls + ": " + id(b) + (StrataPass.isCrust(b) ? " is stone or deepslate, which P1-28 reserves for "
                            + CRUST_STAND_INS.keySet() : ", but P1-28 gives it " + CRUST_STAND_INS.get(cls)));
            }
            if (!r.rejected().isEmpty())
                e.wrong.add(cls + " rejected " + r.rejected());
            var l = live.result(cls).orElse(null);
            if (!rockTags(helper) && (l == null || l.source() != r.source() || !l.blocks().equals(r.blocks())))
                e.wrong.add(cls + ": live palette " + (l == null ? "lacks it" : l.source().id() + " " + l.blocks().stream().map(StrataGameTests::id).toList())
                        + " differs from the vanilla view");
        }
        // The README's vanilla table, spot-checked: members, stand-ins and the coal exception.
        e.that(vanilla, "soil", Source.MEMBER, "minecraft:dirt", "minecraft:rooted_dirt");
        e.that(vanilla, "sand", Source.MEMBER, "minecraft:sand", "minecraft:red_sand");
        e.that(vanilla, "marble", Source.MEMBER, "minecraft:calcite");
        e.that(vanilla, "basalt", Source.MEMBER, "minecraft:smooth_basalt");
        e.that(vanilla, "limestone", Source.VANILLA_FALLBACK, "minecraft:calcite");
        CRUST_STAND_INS.forEach((cls, block) -> e.that(vanilla, cls, Source.VANILLA_FALLBACK, stateOf(block)));
        e.that(vanilla, "loess", Source.VANILLA_FALLBACK, "minecraft:smooth_sandstone");
        e.that(vanilla, "peridotite", Source.VANILLA_FALLBACK, "minecraft:tuff");
        e.that(vanilla, "diamictite", Source.VANILLA_FALLBACK, "minecraft:cobbled_deepslate");
        e.that(vanilla, "black_shale", Source.VANILLA_FALLBACK, "minecraft:smooth_basalt");
        e.that(vanilla, "rhyolite", Source.VANILLA_FALLBACK, "minecraft:granite");
        DeepTime.LOGGER.info("[gametest] vanillaOnlyResolvesEveryClass classes={} sources={} rockTags={}", ours.size(), bySource, rockTags(helper));
        e.done(helper, "vanillaOnlyResolvesEveryClass");
    }

    /**
     * Stone and deepslate stand in for the common classes (P1-28, {@link #CRUST_STAND_INS}) as their
     * {@code vanilla_fallback}, the last resort: without a rock datapack the live palette gives exactly
     * those classes stone or deepslate and no other class either. A pack's fallback tags are read
     * first, so with the MIC rock tags every one of those classes resolves from its fallback tag
     * instead (the pack decides what it looks like; coal's list ends in stone, after Born in Chaos'
     * black argillite when that is installed).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void crustStandInsYieldToPackTags(GameTestHelper helper) {
        RockPalette live = RockPalette.get();
        Expect e = new Expect();
        boolean tags = rockTags(helper);
        CRUST_STAND_INS.forEach((cls, block) -> {
            if (!tags)
                e.that(live, cls, Source.VANILLA_FALLBACK, stateOf(block));
            else {
                e.checked++;
                var r = live.result(cls).orElse(null);
                if (r == null || r.source() != Source.FALLBACK)
                    e.wrong.add(cls + ": with the rock tags the pack's fallback list should decide, got "
                            + (r == null ? "no class" : r.source().id() + " " + r.blocks().stream().map(StrataGameTests::id).toList()));
            }
        });
        if (tags && !loaded(BIC))
            e.that(live, "coal", Source.FALLBACK, "minecraft:stone");
        if (!tags) {
            for (String cls : live.classes()) {
                if (CRUST_STAND_INS.containsKey(cls))
                    continue;
                e.checked++;
                live.result(cls).orElseThrow().blocks().stream().filter(StrataPass::isCrust)
                        .forEach(b -> e.wrong.add(cls + " resolves to " + id(b) + ", which is for " + CRUST_STAND_INS.keySet()));
            }
        }
        // The palette tells a class from the filler where a block cannot: stone is the unclassified crust
        // and every class that resolves to it, deepslate likewise.
        if (!tags) {
            e.checked++;
            var stone = live.classesOf(Blocks.STONE.defaultBlockState());
            var deepslate = live.classesOf(Blocks.DEEPSLATE.defaultBlockState());
            for (var en : CRUST_STAND_INS.entrySet()) {
                var of = en.getValue().equals("minecraft:stone") ? stone : deepslate;
                if (!of.contains(en.getKey()) || !of.contains(RockPalette.UNCLASSIFIED_CRUST))
                    e.wrong.add("classesOf(" + en.getValue() + ") = " + of + " lacks " + en.getKey() + " or the unclassified crust");
            }
            if (stone.contains("slate") || deepslate.contains("shale"))
                e.wrong.add("stone and deepslate classes mixed up: " + stone + " / " + deepslate);
        }
        DeepTime.LOGGER.info("[gametest] crustStandInsYieldToPackTags rockTags={} bornInChaos={} shale={} gneiss={} coal={}", tags, loaded(BIC),
                live.primary("shale").map(StrataGameTests::id).orElse("nothing"), live.primary("gneiss").map(StrataGameTests::id).orElse("nothing"),
                live.primary("coal").map(StrataGameTests::id).orElse("nothing"));
        e.done(helper, "crustStandInsYieldToPackTags");
    }

    /**
     * Mining stone and deepslate drops vanilla's cobblestone and cobbled deepslate (so stone tools,
     * furnaces and every cobblestone recipe work on a planet made mostly of the two), and every other
     * block a class can place drops what vanilla gives, unchanged (Ben, 2026-10-01: no recipe, item
     * tag or drop is widened). The loot tables and recipes in play are vanilla's; none comes from
     * Deep Time's own pack.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void miningTheCommonRocksDropsVanillaCobbles(GameTestHelper helper) {
        var level = helper.getLevel();
        Expect e = new Expect();
        var tool = new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.DIAMOND_PICKAXE);
        Map<String, String> expected = new TreeMap<>(Map.ofEntries(
                Map.entry("minecraft:stone", "minecraft:cobblestone"),
                Map.entry("minecraft:deepslate", "minecraft:cobbled_deepslate"),
                Map.entry("minecraft:granite", "minecraft:granite"), Map.entry("minecraft:diorite", "minecraft:diorite"),
                Map.entry("minecraft:andesite", "minecraft:andesite"), Map.entry("minecraft:smooth_basalt", "minecraft:smooth_basalt"),
                Map.entry("minecraft:tuff", "minecraft:tuff"), Map.entry("minecraft:sandstone", "minecraft:sandstone"),
                Map.entry("minecraft:red_sandstone", "minecraft:red_sandstone"), Map.entry("minecraft:calcite", "minecraft:calcite"),
                Map.entry("minecraft:dripstone_block", "minecraft:dripstone_block"), Map.entry("minecraft:obsidian", "minecraft:obsidian"),
                Map.entry("minecraft:packed_mud", "minecraft:packed_mud"), Map.entry("minecraft:polished_tuff", "minecraft:polished_tuff")));
        BlockPos pos = helper.absolutePos(new BlockPos(1, 1, 1));
        expected.forEach((block, drop) -> {
            e.checked++;
            var state = net.minecraft.core.registries.BuiltInRegistries.BLOCK
                    .get(net.minecraft.resources.ResourceLocation.parse(block)).defaultBlockState();
            var got = net.minecraft.world.level.block.Block.getDrops(state, level, pos, null, null, tool).stream()
                    .map(s -> net.minecraft.core.registries.BuiltInRegistries.ITEM.getKey(s.getItem()).toString()).toList();
            if (!got.equals(List.of(drop)))
                e.wrong.add(block + " drops " + got + ", vanilla gives " + drop);
        });
        // Nothing of Deep Time's overrides a loot table, a recipe or an item tag of vanilla's or the common namespace.
        var resources = level.getServer().getResourceManager();
        for (String dir : List.of("loot_table", "recipe", "tags/item"))
            for (String ns : List.of("minecraft", "c", "neoforge")) {
                e.checked++;
                resources.listResources(dir, rl -> rl.getNamespace().equals(ns)).forEach((rl, res) -> {
                    if (res.sourcePackId().equals("mod/" + DeepTime.MODID))
                        e.wrong.add("Deep Time's jar ships " + rl);
                });
            }
        DeepTime.LOGGER.info("[gametest] miningTheCommonRocksDropsVanillaCobbles checked={} wrong={}", e.checked, e.wrong.size());
        e.done(helper, "miningTheCommonRocksDropsVanillaCobbles");
    }

    /**
     * An empty member tag falls back to the first <em>installed</em> fallback entry, in list order:
     * {@code test_fallback}'s list is [an absent optional block, air (rejected),
     * polished_andesite, andesite] then a {@code rock_states} fallback entry, so polished_andesite;
     * {@code test_state_fallback} has only {@code rock_states} fallback entries (an absent optional
     * block, then polished_granite). With the MIC rock tags, classes whose first stand-in is modded
     * take it when its mod is installed and the vanilla entry after it otherwise.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void firstInstalledFallback(GameTestHelper helper) {
        RockPalette live = RockPalette.get();
        Expect e = new Expect();
        e.that(live, "test_fallback", Source.FALLBACK, "minecraft:polished_andesite");
        e.that(live, "test_state_fallback", Source.FALLBACK, "minecraft:polished_granite");
        e.checked++;
        var rejected = live.result("test_fallback").map(PaletteRules.Result::rejected).orElse(List.of());
        if (rejected.stream().noneMatch(s -> s.startsWith("minecraft:air")))
            e.wrong.add("test_fallback did not reject air: " + rejected);
        if (rockTags(helper)) {
            // The MIC rows since 2026-10-01 (decisions log, "Use stone for common rocks").
            e.that(live, "shale", Source.FALLBACK, "minecraft:stone");
            e.that(live, "hornfels", Source.FALLBACK, loaded(BIC) ? "born_in_chaos_v1:fired_black_argillite" : "minecraft:smooth_basalt");
            e.that(live, "peridotite", Source.FALLBACK, loaded(CREATE) ? "create:veridium" : "minecraft:tuff");
            e.that(live, "anhydrite", Source.FALLBACK, loaded(CREATE) ? "create:limestone" : "minecraft:diorite");
            e.that(live, "bauxite", Source.FALLBACK, loaded(CREATE) ? "create:ochrum" : "minecraft:red_sand");
            e.that(live, "gneiss", Source.FALLBACK, stateOf("minecraft:deepslate"));
        } else {
            e.that(live, "shale", Source.VANILLA_FALLBACK, "minecraft:stone");
            e.that(live, "gneiss", Source.VANILLA_FALLBACK, stateOf("minecraft:deepslate"));
        }
        DeepTime.LOGGER.info("[gametest] firstInstalledFallback test_fallback={} shale={} peridotite={} rockTags={}",
                live.primary("test_fallback").map(StrataGameTests::id).orElse("nothing"),
                live.primary("shale").map(StrataGameTests::id).orElse("nothing"),
                live.primary("peridotite").map(StrataGameTests::id).orElse("nothing"), rockTags(helper));
        e.done(helper, "firstInstalledFallback");
    }

    /**
     * {@code rock_states} member entries set a full block state and skip absent optional blocks:
     * {@code test_states} is snow at {@code layers=8}; with Project Atmosphere installed, loess is its
     * dust at {@code layers=8} (the rock-tag README's example), otherwise loess keeps its stand-in.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void rockStatesMember(GameTestHelper helper) {
        RockPalette live = RockPalette.get();
        Expect e = new Expect();
        e.that(live, "test_states", Source.MEMBER, "minecraft:snow[layers=8]");
        if (loaded(ATMOSPHERE))
            e.primary(live, "loess", Source.MEMBER, "projectatmosphere:dust[layers=8]");
        else
            e.that(live, "loess", rockTags(helper) ? Source.FALLBACK : Source.VANILLA_FALLBACK, "minecraft:smooth_sandstone");
        DeepTime.LOGGER.info("[gametest] rockStatesMember test_states={} loess={} projectAtmosphere={}",
                live.primary("test_states").map(StrataGameTests::id).orElse("nothing"),
                live.primary("loess").map(StrataGameTests::id).orElse("nothing"), loaded(ATMOSPHERE));
        e.done(helper, "rockStatesMember");
    }

    /**
     * Create's ore stones keep their geological classes (the decisions file) and the other modded
     * members resolve with the MIC rock tags; without them those classes use their vanilla
     * stand-ins and no modded block appears in the palette.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void createStonesKeepTheirClasses(GameTestHelper helper) {
        RockPalette live = RockPalette.get();
        Expect e = new Expect();
        boolean tags = rockTags(helper), create = tags && loaded(CREATE);
        Source modded = Source.MEMBER, standIn = tags ? Source.FALLBACK : Source.VANILLA_FALLBACK;
        e.that(live, "limestone", create ? modded : standIn, create ? "create:limestone" : "minecraft:calcite");
        e.that(live, "dolostone", create ? modded : standIn, create ? "create:asurine" : "minecraft:dripstone_block");
        e.that(live, "ironstone", create ? modded : standIn, create ? "create:ochrum" : "minecraft:red_sandstone");
        e.that(live, "banded_iron_formation", create ? modded : standIn, create ? "create:crimsite" : "minecraft:granite");
        e.that(live, "serpentinite", create ? modded : standIn, create ? "create:veridium" : "minecraft:tuff");
        e.that(live, "scoria", create ? modded : standIn, create ? "create:scoria" : "minecraft:smooth_basalt");
        boolean bic = tags && loaded(BIC), iaf = tags && loaded(ICE_AND_FIRE);
        e.that(live, "black_shale", bic ? modded : standIn, bic ? "born_in_chaos_v1:black_argillite" : "minecraft:smooth_basalt");
        e.that(live, "volcanic_ash", iaf ? modded : standIn, iaf ? "iceandfire:ash" : "minecraft:tuff");
        // Diamictite (lithified till, 2026-09-30) is newer than the MIC rock tags: until a pack names
        // it, its own stand-in holds; a pack fallback tag of the same block is just as right.
        e.checked++;
        var diamictite = live.result("diamictite").orElse(null);
        List<String> diamictiteBlocks = diamictite == null ? List.of() : diamictite.blocks().stream().map(StrataGameTests::id).toList();
        boolean standInHolds = diamictite != null && diamictite.source() == Source.VANILLA_FALLBACK
                && diamictiteBlocks.equals(List.of("minecraft:cobbled_deepslate"));
        boolean packNamesIt = tags && diamictite != null && diamictite.source() != Source.VANILLA_FALLBACK
                && diamictite.source() != Source.NONE;
        if (!standInHolds && !packNamesIt)
            e.wrong.add("diamictite: " + (diamictite == null ? "no such class" : diamictite.source().id() + " " + diamictiteBlocks));
        if (!tags) {
            for (String cls : live.classes()) {
                e.checked++;
                for (BlockState b : live.result(cls).orElseThrow().blocks())
                    if (!id(b).startsWith("minecraft:"))
                        e.wrong.add(cls + ": modded " + id(b) + " without the rock tags");
            }
        }
        DeepTime.LOGGER.info("[gametest] createStonesKeepTheirClasses rockTags={} create={} limestone={} dolostone={} ironstone={} bif={} serpentinite={}",
                tags, loaded(CREATE), live.primary("limestone").map(StrataGameTests::id).orElse("nothing"),
                live.primary("dolostone").map(StrataGameTests::id).orElse("nothing"),
                live.primary("ironstone").map(StrataGameTests::id).orElse("nothing"),
                live.primary("banded_iron_formation").map(StrataGameTests::id).orElse("nothing"),
                live.primary("serpentinite").map(StrataGameTests::id).orElse("nothing"));
        e.done(helper, "createStonesKeepTheirClasses");
    }

    /**
     * The strata pass paints every class into the world: {@code test_planet_data} carries the
     * every-class strata ({@code deeptime synth --strata every-class}), where any ten neighbouring
     * 16-block cell columns hold all 60 classes 20 or more blocks below the surface. In ten cell
     * columns on two rows, at SURFACE status, every horizon from the third down holds its class's
     * block (the member its key picks) at two depths, and a class that resolves to nothing leaves the crust (a
     * class that resolves to stone or deepslate (P1-28) is painted as that block, even where the crust was
     * the other one).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void everyClassInTheWorld(GameTestHelper helper) {
        var level = WorldDataGameTests.level(helper);
        var wd = WorldDataGameTests.terrain(helper, level).data();
        List<String> classes = StrataPass.classes(wd);
        var strata = wd.layer(StrataPass.LAYER);
        var grid = strata.desc().grid();
        RockPalette palette = RockPalette.get();
        Expect e = new Expect();
        TreeSet<String> seen = new TreeSet<>();
        long i0 = Math.floorDiv(-grid.originBlockX(), grid.r()), j0 = Math.floorDiv(-grid.originBlockZ(), grid.r());
        StrataPass.Column col = new StrataPass.Column(strata.stackDepth());
        for (long j : new long[]{j0, j0 + 3}) {
            for (long i = i0; i < i0 + 10; i++) {
                int x = (int) grid.blockX(i), z = (int) grid.blockZ(j);
                var chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.SURFACE, true);
                StrataPass.column(wd, strata, x, z, col);
                for (int h = 2; h < col.count; h++) {
                    int cls = col.classes[h];
                    String name = classes.get(cls);
                    BlockState expected = palette.block(name, StrataPass.horizonKey(col.units[h], cls));
                    int bottom = h + 1 < col.count ? col.tops[h + 1] + 1 : col.tops[h] - 9;
                    // The column rule mixes in the neighbouring cell columns' classes as thin
                    // tongues (format 0.2), so a horizon's index no longer says how deep it lies:
                    // sample only horizons with blocks to spare, 20 blocks or more under the
                    // surface rules and clear of the bedrock floor.
                    if (col.tops[h] - bottom < 2 || col.tops[h] > col.tops[0] - 20
                            || bottom < level.getMinBuildHeight() + 8)
                        continue;
                    for (int y : new int[]{col.tops[h] - 1, bottom + 1}) {
                        e.checked++;
                        BlockState actual = chunk.getBlockState(new BlockPos(x, y, z));
                        boolean ok = expected == null ? StrataPass.isCrust(actual) : actual == expected;
                        if (!ok)
                            e.wrong.add("(" + x + "," + y + "," + z + ") h" + h + " " + name + ": expected "
                                    + (expected == null ? "crust" : id(expected)) + ", got " + id(actual));
                    }
                    seen.add(name);
                }
            }
        }
        if (classes.size() != 60)
            e.wrong.add("test_planet_data has " + classes.size() + " classes; gametest.sh must write it with --strata every-class");
        TreeSet<String> missing = new TreeSet<>(classes);
        missing.removeAll(seen);
        if (!missing.isEmpty())
            e.wrong.add("classes never sampled: " + missing);
        DeepTime.LOGGER.info("[gametest] everyClassInTheWorld classes={} seen={} samples={}", classes.size(), seen.size(), e.checked);
        e.done(helper, "everyClassInTheWorld");
    }

    /** {@code /deeptime probe palette} runs and lists every class, all resolved. */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void probePaletteListsEveryClass(GameTestHelper helper) {
        var server = helper.getLevel().getServer();
        server.getCommands().performPrefixedCommand(server.createCommandSourceStack(), "deeptime probe palette");
        var json = RockPalette.get().toJson();
        int count = json.get("count").getAsInt(), resolved = json.get("resolved").getAsInt();
        DeepTime.LOGGER.info("[gametest] probePalette count={} resolved={} sources={} unresolved={} problems={} rockTags={}",
                count, resolved, json.get("sources"), json.get("unresolved"), json.getAsJsonObject("problems").keySet(), rockTags(helper));
        helper.assertTrue(count >= 60, "the palette lists " + count + " classes");
        helper.assertTrue(resolved == count, (count - resolved) + " classes unresolved: " + json.get("unresolved"));
        helper.assertTrue(DeepTimeCommands.PROBE_PREFIX.startsWith("[deeptime-probe]"), "probe prefix");
        helper.succeed();
    }
}
