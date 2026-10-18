// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.api.DeepTimeSites;
import com.minecraftindustrialcomplex.deeptime.api.Site;
import com.minecraftindustrialcomplex.deeptime.ore.OreBlockResolver;
import com.minecraftindustrialcomplex.deeptime.ore.OreData;
import com.minecraftindustrialcomplex.deeptime.ore.PlacerPanning;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.server.SiteCommands;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.BackgroundScatterFeature;
import com.minecraftindustrialcomplex.deeptime.worldgen.DepositFeature;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.GossanSkin;
import com.minecraftindustrialcomplex.deeptime.worldgen.OreDraws;
import com.minecraftindustrialcomplex.deeptime.worldgen.StrataPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.Tag;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.TagKey;
import net.minecraft.world.entity.item.ItemEntity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BrushableBlockEntity;
import net.minecraft.world.level.block.entity.ChestBlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.storage.loot.BuiltInLootTables;
import net.minecraft.world.level.storage.loot.LootParams;
import net.minecraft.world.level.storage.loot.LootTable;
import net.minecraft.world.level.storage.loot.parameters.LootContextParamSets;
import net.minecraft.world.level.storage.loot.parameters.LootContextParams;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.common.Tags;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

/**
 * Deposits in the world (P1-15, plan §7.6): the {@code deeptime:test_deposits} level reads the
 * deposit engine's synthetic world ({@code deposits_world_4k.json}: sites, bodies and events, and a
 * terrain baked from the synthetic history's present elevation) through its fixed biome
 * {@code deeptime:test_ores}, whose only features are the deposits and the background scatter. The
 * tests pick their site from the site registry (the first small deposit with an ore block in this
 * pack, porphyry copper preferred), generate its chunks to FEATURES and compare every block of its
 * ore body with {@link DepositFeature#materialAt} and {@link OreBlockResolver}. Runs in every
 * profile: vanilla ores by default, the MIC rock tags with {@code -PwithMic}, Excavated Variants'
 * ore-in-rock blocks with {@code -PwithEv}.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class DepositGameTests {

    private DepositGameTests() {}

    /** Largest horizontal extent (blocks) of a body the tests generate. */
    static final int MAX_EXTENT = 96;

    static ServerLevel level(GameTestHelper helper, ResourceKey<Level> key) {
        ServerLevel level = helper.getLevel().getServer().getLevel(key);
        if (level == null)
            helper.fail("level " + key.location() + " does not exist");
        return level;
    }

    static WorldData data(GameTestHelper helper, ServerLevel level) {
        if (level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain w)
            return w.data();
        helper.fail(level.dimension().location() + " is not generating from world data");
        throw new IllegalStateException();
    }

    static int canonical(int c, int x) {
        return Math.floorMod(x + c / 2, c) - c / 2;
    }

    /** The site and ore body the tests use. */
    record Target(Site site, Body body, List<String> materials) {}

    /** Blocks of ore a body must be due (before the host check) to be a test target. */
    static final int MIN_DUE = 200;

    /**
     * The first site (by id) with products whose ore body spans at most {@link #MAX_EXTENT} blocks
     * and is due at least {@link #MIN_DUE} blocks of materials that have an ore block here (a pure
     * count of {@link DepositFeature#materialAt}; bodies eroded to a sliver are skipped), porphyry
     * copper first; fails when there is none.
     */
    static Target target(GameTestHelper helper, ServerLevel level, WorldData wd) {
        DeepTimeSites sites = DeepTimeSites.of(level);
        OreBlockResolver ores = OreBlockResolver.get();
        int c = wd.circumference();
        Target fallback = null;
        for (Site s : sites.all()) {
            if (s.products().isEmpty())
                continue;
            for (Body b : wd.deposits().bodies().ofSite(s.id())) {
                if (!b.role().equals("ore") || !DepositFeature.placesOre(b))
                    continue;
                if (b.max()[0] - b.min()[0] > MAX_EXTENT || b.max()[2] - b.min()[2] > MAX_EXTENT)
                    continue;
                if (b.max()[1] < level.getMinBuildHeight() + 8)
                    continue;
                List<String> mats = new ArrayList<>();
                for (Body.Zone z : b.zones())
                    if (z.material() != null && z.density() > 0 && !ores.blocks(ResourceLocation.parse(z.material())).isEmpty()
                            && !mats.contains(z.material()))
                        mats.add(z.material());
                if (mats.isEmpty())
                    continue;
                int due = 0;
                for (int x = b.min()[0]; x <= b.max()[0] && due < MIN_DUE; x++)
                    for (int y = Math.max(level.getMinBuildHeight(), b.min()[1]); y <= b.max()[1] && due < MIN_DUE; y++)
                        for (int z = b.min()[2]; z <= b.max()[2] && due < MIN_DUE; z++) {
                            String m = DepositFeature.materialAt(b, c, x, y, z);
                            if (m != null && mats.contains(m))
                                due++;
                        }
                if (due < MIN_DUE)
                    continue;
                Target t = new Target(s, b, mats);
                if (s.depositType().getPath().startsWith("porphyry"))
                    return t;
                if (fallback == null)
                    fallback = t;
            }
        }
        if (fallback == null)
            helper.fail("no site with a small ore body due ≥ " + MIN_DUE + " ore blocks among " + sites.all().size() + " sites");
        return fallback;
    }

    /** What one scan of a body's blocks found. */
    record Scan(int due, int placed, int exact, int missed, int notHost, int extra, int unexplained, int scatter,
                int chunks, Map<String, Integer> blocks, List<String> problems, int leached) {}

    private static final Map<ResourceKey<Level>, Scan> SCANS = new HashMap<>();

    /**
     * Generates the body's chunks to FEATURES and compares every block of its AABB: where the
     * deposit's draw says ore, a host block must have become that material's ore (the resolver's
     * block for the host the strata pass painted there); elsewhere, every ore block must be
     * explained by a body or the background scatter. The blocks of an exposed gossan's leached cap
     * ({@link GossanSkin.Caps}, DEP-212) are not due: the strata pass painted the cap onto the ground
     * (the host there is the gossan's rock, which {@code predictedRock} does not know) and the cap
     * holds no copper, so ore in one is another gossan zone's or the scatter's, never this body's.
     */
    static synchronized Scan scan(GameTestHelper helper, ServerLevel level, WorldData wd, Target t) {
        Scan cached = SCANS.get(level.dimension());
        if (cached != null)
            return cached;
        int c = wd.circumference();
        Body b = t.body();
        RockPalette palette = RockPalette.get();
        OreBlockResolver ores = OreBlockResolver.get();
        boolean loose = b.rockClass() != null && palette.isLoose(b.rockClass());
        List<OreData.ScatterRule> rules = BackgroundScatterFeature.rules(level.registryAccess());
        Map<Long, ChunkAccess> chunks = new HashMap<>();
        Map<Long, Set<Long>> scatterPlan = new HashMap<>();
        Map<Long, GossanSkin.Caps> capsOf = new HashMap<>();
        boolean ownCap = GossanSkin.ROLE.equals(b.role()) || loose;   // a gossan's own gold and a placer's are not held back
        int minY = Math.max(level.getMinBuildHeight(), b.min()[1]), maxY = Math.min(level.getMaxBuildHeight() - 1, b.max()[1]);
        int due = 0, placed = 0, exact = 0, missed = 0, notHost = 0, extra = 0, unexplained = 0, scatter = 0, leached = 0;
        Map<String, Integer> blocks = new TreeMap<>(), notHostBlocks = new TreeMap<>();
        List<String> problems = new ArrayList<>();
        for (int x = b.min()[0]; x <= b.max()[0]; x++) {
            int xc = canonical(c, x);
            for (int z = b.min()[2]; z <= b.max()[2]; z++) {
                long key = ChunkPos.asLong(xc >> 4, z >> 4);
                ChunkAccess chunk = chunks.computeIfAbsent(key,
                        k -> level.getChunkSource().getChunk(xc >> 4, (int) (k >> 32), ChunkStatus.FEATURES, true));
                GossanSkin.Caps caps = capsOf.computeIfAbsent(key, k -> GossanSkin.Caps.of(wd,
                        wd.deposits().bodiesInChunk(xc >> 4, (int) (k >> 32), level.getMinBuildHeight(), level.getMaxBuildHeight() - 1)));
                GossanSkin.Span cap = ownCap ? null : caps.window(xc, z);
                for (int y = minY; y <= maxY; y++) {
                    BlockState state = chunk.getBlockState(new BlockPos(xc, y, z));
                    Body.Zone zone = DepositFeature.zoneAt(b, c, x, y, z);
                    String m = zone == null ? null : zone.material();
                    boolean hasBlock = m != null && t.materials().contains(m);
                    boolean inCap = cap != null && caps.holdsNoOre(cap, y);
                    if (hasBlock && inCap) {
                        leached++;   // the leached cap holds none of this body's ore (DEP-212): not due
                        hasBlock = false;
                    }
                    if (hasBlock) {
                        due++;
                        ResourceLocation mat = ResourceLocation.parse(m);
                        BlockState raw = DepositFeature.rawAt(b, zone, c, x, y, z) ? ores.rawBlock(mat) : null;
                        if (raw != null && state == raw) {
                            // A raw-ore block of a rich core (vanilla's ore veins): exactly the one due.
                            placed++;
                            exact++;
                            blocks.merge(net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString(), 1, Integer::sum);
                        } else if (raw == null && isPanned(state, mat)) {
                            // A placer's gold as suspicious gravel or sand (no gold ore for the host in this pack).
                            placed++;
                            exact++;
                            blocks.merge(net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString(), 1, Integer::sum);
                        } else if (raw == null && OreBlockResolver.isOreOf(state, mat)) {
                            placed++;
                            blocks.merge(net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString(), 1, Integer::sum);
                            BlockState host = StrataPass.predictedRock(wd, xc, y, z);
                            if (host == null)
                                host = y < 0 ? Blocks.DEEPSLATE.defaultBlockState() : Blocks.STONE.defaultBlockState();
                            if (state == ores.resolve(mat, host, y))
                                exact++;
                            else if (problems.size() < 8)
                                problems.add("(" + xc + "," + y + "," + z + ") " + state.getBlock() + " for host " + host.getBlock()
                                        + " window " + cap + " above " + chunk.getBlockState(new BlockPos(xc, y + 1, z)).getBlock()
                                        + " below " + chunk.getBlockState(new BlockPos(xc, y - 1, z)).getBlock());
                        } else {
                            RockPalette.Host kind = palette.host(state);
                            if (loose && ores.panned(mat, state, state) != null
                                    && !PlacerPanning.supported(chunk.getBlockState(new BlockPos(xc, y - 1, z)))) {
                                // A placer's gold cell over air, water or a plant: left plain gravel or sand (it would fall).
                                notHost++;
                                notHostBlocks.merge("unsupported "
                                        + net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(state.getBlock()), 1, Integer::sum);
                            } else if (kind == RockPalette.Host.ROCK || (kind == RockPalette.Host.LOOSE && loose)) {
                                missed++;
                                if (problems.size() < 8)
                                    problems.add("(" + xc + "," + y + "," + z + ") still " + state.getBlock() + ", " + m + " due");
                            } else {
                                notHost++;
                                notHostBlocks.merge(net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString(), 1, Integer::sum);
                            }
                        }
                        continue;
                    }
                    // Not due from this body: any ore of a claimed material must be another body's or scatter.
                    String oreOf = null;
                    for (String mm : t.materials())
                        if (OreBlockResolver.isOreOf(state, ResourceLocation.parse(mm)))
                            oreOf = mm;
                    for (OreData.ScatterRule r : rules)
                        if (oreOf == null && OreBlockResolver.isOreOf(state, r.material()))
                            oreOf = r.material().toString();
                    if (oreOf == null)
                        continue;
                    extra++;
                    if (explainedByScatter(wd, rules, scatterPlan, level.getMinBuildHeight(), xc, y, z, oreOf)) {
                        scatter++;
                        continue;
                    }
                    boolean otherBody = false;
                    for (Body o : wd.deposits().bodiesIn(xc, y, z, xc, y, z))
                        // In a cap only a gossan's own zones and a placer's gold explain ore: no other body's ore is placed there.
                        if ((!inCap || explainsInCap(o, palette)) && oreOf.equals(DepositFeature.materialAt(o, c, xc, y, z)))
                            otherBody = true;
                    if (!otherBody) {
                        unexplained++;
                        if (problems.size() < 8)
                            problems.add("(" + xc + "," + y + "," + z + ") unexplained " + state.getBlock());
                    }
                }
            }
        }
        Scan s = new Scan(due, placed, exact, missed, notHost, extra, unexplained, scatter, chunks.size(), blocks, problems, leached);
        DeepTime.LOGGER.info("[gametest] deposit scan {} site={} ({}) body={} {} aabb={}..{} materials={}: due={} placed={} exact={} "
                        + "missed={} notHost={} {} extraOre={} (scatter {}) unexplained={} leachedCap={} chunks={} blocks={} problems={}",
                level.dimension().location(), t.site().id(), t.site().depositType(), b.id(), b.shapeName(),
                java.util.Arrays.toString(b.min()), java.util.Arrays.toString(b.max()), t.materials(), due, placed, exact,
                missed, notHost, notHostBlocks, extra, scatter, unexplained, leached, chunks.size(), blocks, problems);
        SCANS.put(level.dimension(), s);
        return s;
    }

    /** True when ore a body draws can sit in a leached cap's window: a gossan's own zones and a placer's (loose rock's) gold are not held back. */
    static boolean explainsInCap(Body o, RockPalette palette) {
        return GossanSkin.ROLE.equals(o.role()) || (o.rockClass() != null && palette.isLoose(o.rockClass()));
    }

    /** True when {@code state} is the suspicious gravel or sand a placer puts for {@code material} (only gold has a panning table). */
    static boolean isPanned(BlockState state, ResourceLocation material) {
        return PlacerPanning.tableOf(material) != null && (state.is(Blocks.SUSPICIOUS_GRAVEL) || state.is(Blocks.SUSPICIOUS_SAND));
    }

    static boolean explainedByScatter(WorldData wd, List<OreData.ScatterRule> rules, Map<Long, Set<Long>> cache, int minY,
                                      int x, int y, int z, String material) {
        for (OreData.ScatterRule r : rules) {
            if (!r.material().toString().equals(material))
                continue;
            long key = ChunkPos.asLong(x >> 4, z >> 4) ^ ((long) r.material().hashCode() << 17);
            Set<Long> set = cache.computeIfAbsent(key, k -> {
                Set<Long> out = new HashSet<>();
                for (var p : BackgroundScatterFeature.plan(wd, r, x >> 4, z >> 4, minY))
                    out.add(BlockPos.asLong(p.x(), p.y(), p.z()));
                return out;
            });
            if (set.contains(BlockPos.asLong(x, y, z)))
                return true;
        }
        return false;
    }

    /**
     * A known site's ore body holds its ore: every host block its zones' draws select became the
     * material's ore block for the host the strata pass painted, none was skipped, and every other
     * ore block in the body's box is another body's or the background scatter's.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 6000)
    public static void bodyHasOre(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = data(helper, level);
        Target t = target(helper, level, wd);
        Scan s = scan(helper, level, wd, t);
        helper.assertTrue(s.placed() >= 20, "only " + s.placed() + " ore blocks in site " + t.site().id() + "'s body (" + s.due() + " due)");
        helper.assertTrue(s.missed() == 0, s.missed() + " host blocks were due ore and are not: " + s.problems());
        helper.assertTrue(s.exact() == s.placed(), (s.placed() - s.exact()) + " ore blocks are not the resolver's block for their host: "
                + s.problems());
        helper.assertTrue(s.unexplained() == 0, s.unexplained() + " ore blocks are neither a body's nor scatter: " + s.problems());
        helper.succeed();
    }

    /**
     * An exposed gossan's leached cap holds none of the deposit's ore (DEP-212): the first exposed
     * gossan of the test world with at least {@link #MIN_CAP_DUE} blocks of its site's ore draw inside
     * its cap window ({@link GossanSkin.Caps}, a pure count) is generated to FEATURES, and not one
     * block of the window holds an ore the site's ore bodies draw there, except what the gossan's own
     * zones or the background scatter explain. The skin repaints the cap onto the ground, so the ore
     * features once took the painted rock as the host and placed Excavated Variants' variant for
     * it; the window is the rule that keeps them out, and this test fails when ore comes back.
     * It also reads the skin's own counters: the cap hangs from the terrain model's ground within the
     * margin ({@link GossanSkin#GROUND_MARGIN}), and the share of columns the margin had to hold is
     * small.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 6000)
    public static void gossanCapHoldsNoOre(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = data(helper, level);
        RockPalette palette = RockPalette.get();
        int c = wd.circumference(), minY = level.getMinBuildHeight(), maxY = level.getMaxBuildHeight() - 1;
        Body gossan = null;
        List<Body> oreBodies = List.of();
        int gossans = 0, exposed = 0, bestDue = 0;
        for (Body g : wd.deposits().bodies().bodies()) {
            if (!GossanSkin.ROLE.equals(g.role()) || g.rockClass() == null
                    || g.max()[0] - g.min()[0] > MAX_EXTENT || g.max()[2] - g.min()[2] > MAX_EXTENT)
                continue;
            gossans++;
            GossanSkin.Caps caps = GossanSkin.Caps.of(wd, List.of(g));
            if (caps.isEmpty())
                continue;
            exposed++;
            List<Body> ore = wd.deposits().bodies().ofSite(g.site()).stream()
                    .filter(o -> o.role().equals("ore") && DepositFeature.placesOre(o)).toList();
            int due = 0;
            for (int x = g.min()[0]; x <= g.max()[0]; x++)
                for (int z = g.min()[2]; z <= g.max()[2]; z++) {
                    GossanSkin.Span w = caps.window(canonical(c, x), z);
                    if (w == null)
                        continue;
                    for (int y = Math.max(minY, w.to()); y <= Math.min(maxY, w.from()); y++)
                        for (Body o : ore)
                            if (DepositFeature.materialAt(o, c, x, y, z) != null)
                                due++;
                }
            bestDue = Math.max(bestDue, due);
            if (due >= MIN_CAP_DUE && gossan == null) {
                gossan = g;
                oreBodies = ore;
            }
        }
        helper.assertTrue(gossan != null, "no exposed gossan has " + MIN_CAP_DUE + " blocks of its site's ore draw in its cap ("
                + gossans + " gossans, " + exposed + " exposed, best " + bestDue + ")");
        List<OreData.ScatterRule> rules = BackgroundScatterFeature.rules(level.registryAccess());
        Map<Long, Set<Long>> scatterPlan = new HashMap<>();
        Map<Long, GossanSkin.Caps> capsOf = new HashMap<>();
        BlockState rock = palette.block(gossan.rockClass(), StrataPass.bodyKey(gossan.id()));
        int columns = 0, rocky = 0, cells = 0, drawn = 0, ore = 0;
        List<String> problems = new ArrayList<>();
        for (int x = gossan.min()[0]; x <= gossan.max()[0]; x++) {
            int xc = canonical(c, x);
            for (int z = gossan.min()[2]; z <= gossan.max()[2]; z++) {
                long key = ChunkPos.asLong(xc >> 4, z >> 4);
                GossanSkin.Caps caps = capsOf.computeIfAbsent(key, k -> GossanSkin.Caps.of(wd,
                        wd.deposits().bodiesInChunk(xc >> 4, (int) (k >> 32), minY, maxY)));
                GossanSkin.Span w = caps.window(xc, z);
                if (w == null)
                    continue;
                ChunkAccess chunk = level.getChunkSource().getChunk(xc >> 4, z >> 4, ChunkStatus.FEATURES, true);
                columns++;
                boolean found = false;
                for (int y = Math.max(minY, w.to()); y <= Math.min(maxY, w.from()); y++) {
                    BlockState state = chunk.getBlockState(new BlockPos(xc, y, z));
                    if (!caps.holdsNoOre(w, y))
                        continue;
                    cells++;
                    found |= state == rock;
                    for (Body o : oreBodies) {
                        String m = DepositFeature.materialAt(o, c, x, y, z);
                        if (m == null)
                            continue;
                        drawn++;
                        if (!OreBlockResolver.isOreOf(state, ResourceLocation.parse(m)))
                            continue;
                        ore++;
                        if (explainedByScatter(wd, rules, scatterPlan, minY, xc, y, z, m))
                            continue;   // the background scatter
                        boolean explained = false;
                        for (Body o2 : wd.deposits().bodiesIn(xc, y, z, xc, y, z))
                            if (explainsInCap(o2, palette) && m.equals(DepositFeature.materialAt(o2, c, xc, y, z)))
                                explained = true;   // the gossan's own gold, or a placer's
                        if (explained)
                            continue;
                        if (problems.size() < 8)
                            problems.add("(" + xc + "," + y + "," + z + ") " + state.getBlock() + " for " + m + " of body " + o.id());
                    }
                }
                if (found)
                    rocky++;
            }
        }
        GossanSkin.Stats st = GossanSkin.stats();
        DeepTime.LOGGER.info("[gametest] gossanCapHoldsNoOre gossan={} site={} {} cap columns={} (gossan rock in {}) window cells={} ore drawn there={} "
                        + "ore blocks there={} problems={}; skin capped {} columns since start, the margin ({}) held {}",
                gossan.id(), gossan.site(), gossan.rockClass(), columns, rocky, cells, drawn, ore, problems, st.capped(),
                GossanSkin.GROUND_MARGIN, st.clamped());
        helper.assertTrue(columns > 0, "no cap column in gossan " + gossan.id() + "'s footprint");
        helper.assertTrue(problems.isEmpty(), "ore in the leached cap of gossan " + gossan.id() + ": " + problems);
        helper.assertTrue(st.clamped() * 4 <= st.capped(), "the margin held " + st.clamped() + " of " + st.capped()
                + " capped columns to the terrain model's ground: the model is off");
        helper.succeed();
    }

    /** Blocks of a site's ore draw inside a gossan's cap window that make it a target for {@link #gossanCapHoldsNoOre}. */
    static final int MIN_CAP_DUE = 20;

    // ---- placer gold as panning (Ben, 2026-10-16) ------------------------------------------------

    static final ResourceLocation GOLD = ResourceLocation.fromNamespaceAndPath("c", "gold");

    /**
     * True when the pack makes the placer's gold in gravel or sand something other than suspicious
     * gravel or sand: Excavated Variants is installed, or a gold ore is in {@code #c:ores_in_ground/gravel}
     * or {@code /sand}. False in the standalone profile (vanilla ores only).
     */
    static boolean packHasGravelGold(OreBlockResolver ores) {
        BlockState gravel = Blocks.GRAVEL.defaultBlockState(), sand = Blocks.SAND.defaultBlockState();
        return OreBlockResolver.evLoaded() || ores.blocks(GOLD).stream()
                .anyMatch(b -> b.is(OreBlockResolver.inGround(gravel)) || b.is(OreBlockResolver.inGround(sand)));
    }

    /**
     * The rules of the panned block, without a world: without a gold ore for gravel and sand, a
     * placer's gold in a gravel cell is suspicious gravel and in a sand cell suspicious sand; soil, rock
     * and every other material keep the ore block; the block under a suspicious block must hold it up.
     */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void placerGoldBlockRules(GameTestHelper helper) {
        OreBlockResolver ores = OreBlockResolver.get();
        BlockState gravel = Blocks.GRAVEL.defaultBlockState(), sand = Blocks.SAND.defaultBlockState();
        BlockState inGravel = ores.panned(GOLD, gravel, gravel), inSand = ores.panned(GOLD, sand, sand);
        boolean gravelGold = packHasGravelGold(ores);
        DeepTime.LOGGER.info("[gametest] placerGoldBlockRules: gold in gravel {}, in sand {}, in red sand {} (EV {}, pack has gravel gold {})",
                inGravel, inSand, ores.panned(GOLD, Blocks.RED_SAND.defaultBlockState(), sand), OreBlockResolver.evLoaded(), gravelGold);
        if (!gravelGold) {
            helper.assertTrue(inGravel != null && inGravel.is(Blocks.SUSPICIOUS_GRAVEL), "gold in gravel is " + inGravel);
            helper.assertTrue(inSand != null && inSand.is(Blocks.SUSPICIOUS_SAND), "gold in sand is " + inSand);
            BlockState inRedSand = ores.panned(GOLD, Blocks.RED_SAND.defaultBlockState(), sand);
            helper.assertTrue(inRedSand != null && inRedSand.is(Blocks.SUSPICIOUS_SAND), "gold in red sand is " + inRedSand);
        } else {
            // With Excavated Variants (or a pack's own gravel gold ore) nothing changes.
            helper.assertTrue(inGravel == null, "gold in gravel is " + inGravel + " although the pack has a gold ore for gravel");
        }
        // Not a gravel or sand cell: soil over the bar and rock keep today's ore block.
        helper.assertTrue(ores.panned(GOLD, Blocks.DIRT.defaultBlockState(), gravel) == null, "soil over the gravel is panned");
        helper.assertTrue(ores.panned(GOLD, Blocks.CLAY.defaultBlockState(), gravel) == null, "clay is panned");
        helper.assertTrue(ores.panned(GOLD, Blocks.STONE.defaultBlockState(), Blocks.STONE.defaultBlockState()) == null, "stone is panned");
        // Not gold: silver (the placer's by-product zone) and everything else has no table.
        for (String other : List.of("c:silver", "c:copper", "c:iron", "c:zinc"))
            helper.assertTrue(ores.panned(ResourceLocation.parse(other), gravel, gravel) == null, other + " in gravel is panned");
        // Support: stone, gravel and a placed suspicious block hold a suspicious block up; air, water,
        // fire and plants do not (they would let it fall).
        for (BlockState holds : List.of(Blocks.STONE.defaultBlockState(), gravel, sand, Blocks.DIRT.defaultBlockState(),
                Blocks.SUSPICIOUS_GRAVEL.defaultBlockState(), Blocks.DEEPSLATE.defaultBlockState()))
            helper.assertTrue(PlacerPanning.supported(holds), holds.getBlock() + " does not hold a suspicious block up");
        for (BlockState lets : List.of(Blocks.AIR.defaultBlockState(), Blocks.CAVE_AIR.defaultBlockState(), Blocks.WATER.defaultBlockState(),
                Blocks.LAVA.defaultBlockState(), Blocks.SHORT_GRASS.defaultBlockState(), Blocks.FIRE.defaultBlockState()))
            helper.assertTrue(!PlacerPanning.supported(lets), lets.getBlock() + " holds a suspicious block up");
        helper.succeed();
    }

    /**
     * The loot table, the roll and the block entity, in a real level:
     * {@code deeptime:archaeology/placer_gold} is loaded and {@link PlacerPanning#roll} gives one
     * stack of 2 to 5 gold nuggets whatever the seed (and both ends occur), the same stack vanilla's
     * brushing would unpack from the table and the seed ({@code LootTable.getRandomItems(params,
     * seed)}, its first stack). {@link PlacerPanning#fill} stores that stack in a suspicious
     * gravel's block entity as its item, with no loot table; brushing the block (ten strokes,
     * {@code BrushableBlockEntity.brush}) drops exactly that stack and leaves plain gravel.
     */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void placerGoldLootTable(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        LootTable table = level.getServer().reloadableRegistries().getLootTable(PlacerPanning.GOLD_TABLE);
        helper.assertTrue(table != LootTable.EMPTY, PlacerPanning.GOLD_TABLE.location() + " is not loaded");
        BlockPos at = helper.absolutePos(new BlockPos(2, 2, 2));
        LootParams params = new LootParams.Builder(level).withParameter(LootContextParams.ORIGIN, Vec3.atCenterOf(at))
                .create(LootContextParamSets.CHEST);
        // The seeds the placer really uses (a hash of the position; consecutive small seeds would
        // all start java.util.Random in nearly the same place and roll nearly the same count).
        int min = Integer.MAX_VALUE, max = 0, n = 400;
        long sum = 0;
        for (int i = 0; i < n; i++) {
            long seed = OreDraws.panSeed(level.getSeed(), 4096, i, 64, 0);
            ItemStack stack = PlacerPanning.roll(level, at, PlacerPanning.GOLD_TABLE, seed);
            helper.assertTrue(stack.is(Items.GOLD_NUGGET), "seed " + seed + " gave " + stack);
            helper.assertTrue(ItemStack.matches(stack, PlacerPanning.roll(level, at, PlacerPanning.GOLD_TABLE, seed)), "seed " + seed + " is not repeatable");
            var brushed = table.getRandomItems(params, seed);
            helper.assertTrue(brushed.size() == 1, "seed " + seed + " gave " + brushed.size() + " stacks");
            helper.assertTrue(ItemStack.matches(stack, brushed.get(0)), "seed " + seed + " rolled " + stack + " but brushing would give " + brushed.get(0));
            min = Math.min(min, stack.getCount());
            max = Math.max(max, stack.getCount());
            sum += stack.getCount();
        }
        DeepTime.LOGGER.info("[gametest] placerGoldLootTable: {} seeds gave {} to {} gold nuggets, mean {}", n, min, max, sum / (double) n);
        helper.assertTrue(min == 2 && max == 5, n + " seeds gave " + min + " to " + max + " nuggets, not 2 to 5");
        helper.assertTrue(Math.abs(sum / (double) n - 3.5) < 0.3, "mean " + sum / (double) n + " nuggets, not about 3.5");
        // The block entity holds the stack and no table (on stone: the block would fall off air).
        level.setBlock(at.below(), Blocks.STONE.defaultBlockState(), 3);
        level.setBlock(at, Blocks.SUSPICIOUS_GRAVEL.defaultBlockState(), 3);
        long seed = OreDraws.panSeed(level.getSeed(), 4096, at.getX(), at.getY(), at.getZ());
        ItemStack gold = PlacerPanning.roll(level, at, PlacerPanning.GOLD_TABLE, seed);
        helper.assertTrue(PlacerPanning.fill(level, at, PlacerPanning.GOLD_TABLE, seed), "no brushable block entity at " + at);
        CompoundTag tag = level.getBlockEntity(at).saveWithoutMetadata(level.registryAccess());
        helper.assertTrue(!tag.contains("LootTable") && ItemStack.matches(gold, itemOf(level, tag)), "block entity " + tag + ", rolled " + gold);
        // One brushing, for whoever gets there first: the stack drops once and the block is gravel.
        Player player = helper.makeMockPlayer(GameType.SURVIVAL);
        BrushableBlockEntity be = (BrushableBlockEntity) level.getBlockEntity(at);
        long t = level.getGameTime();
        boolean done = false;
        for (int i = 0; i < 10 && !done; i++)
            done = be.brush(t + 10L * i, player, Direction.UP); // a stroke every 10 ticks (the brush's cooldown)
        helper.assertTrue(done, "ten strokes did not finish brushing " + at);
        helper.assertTrue(level.getBlockState(at).is(Blocks.GRAVEL), "brushed out, " + at + " is " + level.getBlockState(at).getBlock());
        List<ItemEntity> drops = level.getEntitiesOfClass(ItemEntity.class, new AABB(at).inflate(2));
        int nuggets = drops.stream().filter(e -> e.getItem().is(Items.GOLD_NUGGET)).mapToInt(e -> e.getItem().getCount()).sum();
        drops.forEach(e -> e.discard());
        level.setBlock(at, Blocks.AIR.defaultBlockState(), 3);
        level.setBlock(at.below(), Blocks.AIR.defaultBlockState(), 3);
        helper.assertTrue(nuggets == gold.getCount(), "brushing dropped " + nuggets + " gold nuggets, the block held " + gold);
        helper.succeed();
    }

    /** The item a saved brushable block entity holds ({@code item}), or {@link ItemStack#EMPTY}. */
    static ItemStack itemOf(ServerLevel level, CompoundTag tag) {
        return tag != null && tag.contains("item", Tag.TAG_COMPOUND)
                ? ItemStack.parse(level.registryAccess(), tag.getCompound("item")).orElse(ItemStack.EMPTY) : ItemStack.EMPTY;
    }

    /**
     * What one scan of a placer body's gold cells found. {@code filled} counts suspicious blocks
     * whose block entity holds the gold rolled for their position and no loot table;
     * {@code hanging} those whose block below is free when the scan reads it; {@code converted}
     * gold cells another mod (Lootr) has replaced with its own block. {@code problems} are
     * block-entity faults and converted cells (always a failure); {@code samples} are the first
     * cells of {@code hanging} and {@code missed}.
     */
    record PlacerScan(int due, int panned, int filled, int ore, int unsupported, int missed, int hanging, int converted, int other,
                      List<String> problems, List<String> samples) {}

    /**
     * Up to {@code limit} placer bodies (by site id) of the test world that are small enough to
     * generate and have at least {@code minDue} gold cells due.
     */
    static List<Body> placerBodies(ServerLevel level, WorldData wd, int minDue, int limit) {
        DeepTimeSites sites = DeepTimeSites.of(level);
        RockPalette palette = RockPalette.get();
        int c = wd.circumference();
        List<Body> found = new ArrayList<>();
        for (Site s : sites.all()) {
            if (!s.depositType().getPath().equals("placer_au"))
                continue;
            for (Body b : wd.deposits().bodies().ofSite(s.id())) {
                if (found.size() >= limit)
                    return found;
                if (!b.role().equals("ore") || b.rockClass() == null || !palette.isLoose(b.rockClass()))
                    continue;
                if (b.max()[0] - b.min()[0] > MAX_EXTENT || b.max()[2] - b.min()[2] > MAX_EXTENT
                        || b.max()[1] < level.getMinBuildHeight() + 8)
                    continue;
                int due = 0;
                for (int x = b.min()[0]; x <= b.max()[0] && due < minDue; x++)
                    for (int y = Math.max(level.getMinBuildHeight(), b.min()[1]); y <= b.max()[1] && due < minDue; y++)
                        for (int z = b.min()[2]; z <= b.max()[2] && due < minDue; z++)
                            if (GOLD.toString().equals(DepositFeature.materialAt(b, c, x, y, z)))
                                due++;
                if (due >= minDue)
                    found.add(b);
            }
        }
        return found;
    }

    /**
     * Generates a placer body's chunks and sorts every cell the gold zone draws: suspicious gravel or
     * sand ({@code panned}; {@code filled} when its block entity holds {@link PlacerPanning#roll}'s
     * stack for its position and seed and no loot table; {@code hanging} when the block under it is
     * free now); an ore block or raw gold block ({@code ore}); plain gravel or sand over a free block
     * ({@code unsupported}); plain gravel or sand over a solid block ({@code missed}); a Lootr block
     * ({@code converted}); anything else, water, air or a rock the feature skipped ({@code other}).
     *
     * <p>The bar's chunks are read as a player gets them, in full ({@code level.getChunk}), and each
     * block entity as the chunk would save it ({@code getBlockEntityNbtForSaving}: the object, or the
     * saved tag a full chunk that has not ticked yet still holds). A chunk read back from disk below
     * full status does not show its block entities: a {@code ProtoChunk} holds them only as saved
     * tags (the first MIC gate failure of 2026-10-17, chunks saved 36 s after the porphyry scan
     * generated them), and a chunk that was saved in full is a level chunk inside an
     * {@code ImposterProtoChunk} whose block entities wait in its post-load step
     * ({@code ChunkSerializer.postLoadChunk}), which runs only when the chunk is made full again (the
     * second, where {@code coePicksSiteVein} had loaded chunk -90, 0 in full and the scan at
     * {@code FEATURES} then saw none of its 37 block entities, all 37 in its saved data).
     */
    static PlacerScan scanPlacer(ServerLevel level, WorldData wd, Body b, boolean panning) {
        int c = wd.circumference();
        OreBlockResolver ores = OreBlockResolver.get();
        BlockState raw = ores.rawBlock(GOLD);
        Map<Long, LevelChunk> chunks = new HashMap<>();
        int minY = Math.max(level.getMinBuildHeight(), b.min()[1]), maxY = Math.min(level.getMaxBuildHeight() - 1, b.max()[1]);
        int due = 0, panned = 0, filled = 0, ore = 0, unsupported = 0, missed = 0, hanging = 0, converted = 0, other = 0;
        List<String> problems = new ArrayList<>(), samples = new ArrayList<>();
        for (int x = b.min()[0]; x <= b.max()[0]; x++) {
            int xc = canonical(c, x);
            for (int z = b.min()[2]; z <= b.max()[2]; z++) {
                long key = ChunkPos.asLong(xc >> 4, z >> 4);
                LevelChunk chunk = chunks.computeIfAbsent(key, k -> level.getChunk(xc >> 4, (int) (k >> 32)));
                for (int y = minY; y <= maxY; y++) {
                    Body.Zone zone = DepositFeature.zoneAt(b, c, x, y, z);
                    if (zone == null || !GOLD.toString().equals(zone.material()))
                        continue;
                    due++;
                    BlockPos p = new BlockPos(xc, y, z);
                    BlockState state = chunk.getBlockState(p), below = chunk.getBlockState(p.below());
                    if (state.is(Blocks.SUSPICIOUS_GRAVEL) || state.is(Blocks.SUSPICIOUS_SAND)) {
                        panned++;
                        if (!PlacerPanning.supported(below)) {
                            hanging++;
                            if (samples.size() < 8)
                                samples.add(p.toShortString() + " " + state.getBlock() + " over " + below.getBlock());
                        }
                        CompoundTag tag = chunk.getBlockEntityNbtForSaving(p, level.registryAccess());
                        ItemStack gold = PlacerPanning.roll(level, p, PlacerPanning.GOLD_TABLE, OreDraws.panSeed(level.getSeed(), c, x, y, z));
                        if (tag != null && "minecraft:brushable_block".equals(tag.getString("id")) && !tag.contains("LootTable")
                                && !gold.isEmpty() && ItemStack.matches(gold, itemOf(level, tag)))
                            filled++;
                        else if (problems.size() < 8)
                            problems.add(p.toShortString() + " block entity " + tag + " (" + chunk.getClass().getSimpleName() + " "
                                    + chunk.getPersistedStatus() + "), gold due " + gold);
                    } else if (OreBlockResolver.isOreOf(state, GOLD) || (raw != null && state == raw)) {
                        ore++;
                    } else if (state.is(Tags.Blocks.GRAVELS) || state.is(Tags.Blocks.SANDS)) {
                        if (panning && !PlacerPanning.supported(below))
                            unsupported++;
                        else {
                            missed++;
                            if (samples.size() < 8)
                                samples.add(p.toShortString() + " still " + state.getBlock() + " over " + below.getBlock() + ", gold due");
                        }
                    } else if (isLootr(state)) {
                        converted++;
                        if (problems.size() < 8)
                            problems.add(p.toShortString() + " is " + BuiltInRegistries.BLOCK.getKey(state.getBlock()) + ": Lootr took it over");
                    } else {
                        other++;
                    }
                }
            }
        }
        PlacerScan s = new PlacerScan(due, panned, filled, ore, unsupported, missed, hanging, converted, other, problems, samples);
        DeepTime.LOGGER.info("[gametest] placer scan {} body={} aabb={}..{} panning={}: {}", level.dimension().location(), b.id(),
                java.util.Arrays.toString(b.min()), java.util.Arrays.toString(b.max()), panning, s);
        return s;
    }

    static final String LOOTR = "lootr";

    /** True for a block of Lootr's ({@code lootr:*}). */
    static boolean isLootr(BlockState state) {
        return BuiltInRegistries.BLOCK.getKey(state.getBlock()).getNamespace().equals(LOOTR);
    }

    /**
     * A placer bar of the test world holds its gold as the pack makes it. Without a gold ore for
     * gravel and sand (the standalone profile): suspicious gravel or sand, each holding the gold its
     * position's seed rolls from the placer gold loot table and no loot table, and no gravel or sand
     * cell left plain over solid ground; soil in the bar keeps the ore block. With Excavated Variants
     * (or a pack's own gravel gold ore): no suspicious block at all. With Lootr (MIC), the bar's
     * chunks are then loaded in full with two chunks around them, which is when Lootr replaces a
     * suspicious block whose block entity has a loot table with its own per-player block, and none
     * of the bar's blocks may be replaced ({@link #lootrLeavesPlacerGold}).
     *
     * <p>Which blocks a fixture bar's cells are (gravel the strata pass painted into stone, or the
     * soil the surface rules left on a bench) is the terrain's, not this feature's, so the test scans
     * up to four placer bodies for the first with a gold cell that is gravel or sand (with gold ore,
     * any gold ore) and, when none has one, logs that and checks nothing. A few cells may differ
     * between the moment the feature read the block below and the finished chunk (2 % are allowed in
     * {@code missed} and {@code hanging}); the block entity of every suspicious block must be right.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 6000)
    public static void placerBarHoldsPannedGold(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = data(helper, level);
        OreBlockResolver ores = OreBlockResolver.get();
        boolean panning = ores.panned(GOLD, Blocks.GRAVEL.defaultBlockState(), Blocks.GRAVEL.defaultBlockState()) != null;
        helper.assertTrue(panning != packHasGravelGold(ores), "the resolver " + (panning ? "pans" : "does not pan")
                + " but the pack " + (packHasGravelGold(ores) ? "has" : "has no") + " gold ore for gravel");
        List<Body> bodies = placerBodies(level, wd, 30, 4);
        helper.assertTrue(!bodies.isEmpty(), "no placer body with 30 gold cells among " + DeepTimeSites.of(level).all().size() + " sites");
        PlacerScan s = null;
        Body bar = null;
        for (Body b : bodies) {
            PlacerScan one = scanPlacer(level, wd, b, panning);
            helper.assertTrue(one.problems().isEmpty(), "body " + b.id() + " problems: " + one.problems());
            if (panning ? one.panned() + one.unsupported() + one.missed() > 0 : one.ore() > 0) {
                s = one;
                bar = b;
                break;
            }
        }
        if (s == null) {
            DeepTime.LOGGER.info("[gametest] placerBarHoldsPannedGold: none of {} placer bodies has a gold cell that is {}; nothing to check",
                    bodies.size(), panning ? "gravel or sand" : "ore");
            helper.succeed();
            return;
        }
        int gravelly = s.panned() + s.unsupported() + s.missed();
        helper.assertTrue(s.missed() * 50 <= gravelly, s.missed() + " of " + gravelly + " gravel or sand cells over solid ground were due gold and are plain: "
                + s.samples());
        if (panning) {
            helper.assertTrue(s.filled() == s.panned(), (s.panned() - s.filled()) + " suspicious blocks do not hold their gold: " + s.problems());
            helper.assertTrue(s.hanging() * 50 <= s.panned(), s.hanging() + " of " + s.panned() + " suspicious blocks have a free block under them: "
                    + s.samples());
            helper.assertTrue(s.panned() > 0, "no suspicious gravel or sand among " + s.due() + " gold cells: " + s);
        } else {
            helper.assertTrue(s.panned() == 0, s.panned() + " suspicious blocks in a pack with gold ore for gravel: " + s);
        }
        if (!panning || !ModList.get().isLoaded(LOOTR)) {
            if (panning)
                DeepTime.LOGGER.info("[gametest] placer lootr: Lootr is not loaded; nothing to check");
            helper.succeed();
            return;
        }
        lootrLeavesPlacerGold(helper, level, wd, bar, s);
    }

    /** One of Lootr's block tags of what it converts ({@code data/lootr/tags/block/convert/}). */
    static TagKey<Block> lootrConverts(String path) {
        return TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath(LOOTR, "convert/" + path));
    }

    /**
     * Lootr (in MIC) replaces a block in {@code #lootr:convert/gravels} or {@code /sands} whose
     * brushable block entity has a loot table with its own per-player block, once the block's
     * level chunk and the 24 around it are loaded (its {@code BlockEntityTicker}; decisions log
     * 2026-10-17). Placer gold is a finite deposit, one brushing for whoever gets there first, so
     * none of the bar's blocks may be replaced. The bar's chunks and two chunks around them are
     * forced and loaded in full, and controls go above the bar: a chest with a vanilla loot table,
     * which Lootr ({@code #lootr:convert/chests}) must turn into its own chest, or the check sees
     * nothing; and, where Lootr converts suspicious blocks in this server, a suspicious gravel (or
     * sand) with the placer gold table and a seed (the form the bar's blocks had before
     * 2026-10-17), which must become a Lootr block too. Then every gold cell is scanned again:
     * none converted, the same suspicious blocks, each still holding its gold.
     *
     * <p>The GameTest server enables every datapack it finds ({@code GameTestServer.create}), so
     * also Lootr's opt-in feature pack {@code lootr_no_suspicious_blocks}, which empties both tags:
     * there Lootr converts no suspicious block at all, the suspicious control is left out (logged),
     * and what keeps the bar's blocks from Lootr in a world is the check above that none has a loot
     * table ({@code filled == panned}). A MIC world leaves that pack off: a feature pack is not
     * selected by default, and the pack enables it nowhere (read 2026-10-17).
     */
    static void lootrLeavesPlacerGold(GameTestHelper helper, ServerLevel level, WorldData wd, Body bar, PlacerScan before) {
        int c = wd.circumference();
        int cx0 = canonical(c, bar.min()[0]) >> 4, cz0 = bar.min()[2] >> 4;
        int nx = (bar.max()[0] >> 4) - (bar.min()[0] >> 4), nz = (bar.max()[2] >> 4) - cz0;
        List<ChunkPos> forced = new ArrayList<>();
        for (int dx = -2; dx <= nx + 2; dx++)
            for (int dz = -2; dz <= nz + 2; dz++) {
                ChunkPos cp = new ChunkPos(cx0 + dx, cz0 + dz);
                if (!level.getForcedChunks().contains(cp.toLong()) && level.setChunkForced(cp.x, cp.z, true))
                    forced.add(cp);
                level.getChunk(cp.x, cp.z); // full status, now
            }
        int midX = canonical(c, (bar.min()[0] + bar.max()[0]) / 2), midZ = (bar.min()[2] + bar.max()[2]) / 2;
        BlockPos chest = new BlockPos(midX, level.getMaxBuildHeight() - 2, midZ), grain = chest.east(2);
        BlockState gravel = Blocks.SUSPICIOUS_GRAVEL.defaultBlockState(), sand = Blocks.SUSPICIOUS_SAND.defaultBlockState();
        BlockState suspicious = gravel.is(lootrConverts("gravels")) ? gravel : sand.is(lootrConverts("sands")) ? sand : null;
        List<BlockPos> controls = List.of(chest, grain, grain.below());
        for (BlockPos p : controls)
            helper.assertTrue(level.getBlockState(p).isAir(), "no room for the controls at " + p.toShortString());
        level.setBlock(chest, Blocks.CHEST.defaultBlockState(), 3);
        helper.assertTrue(level.getBlockEntity(chest) instanceof ChestBlockEntity, "no chest block entity at the control");
        ((ChestBlockEntity) level.getBlockEntity(chest)).setLootTable(BuiltInLootTables.SIMPLE_DUNGEON, level.getSeed() ^ chest.asLong());
        if (suspicious != null) {
            level.setBlock(grain.below(), Blocks.STONE.defaultBlockState(), 3);
            level.setBlock(grain, suspicious, 3);
            helper.assertTrue(level.getBlockEntity(grain) instanceof BrushableBlockEntity, "no brushable block entity at the control");
            ((BrushableBlockEntity) level.getBlockEntity(grain)).setLootTable(PlacerPanning.GOLD_TABLE,
                    OreDraws.panSeed(level.getSeed(), c, grain.getX(), grain.getY(), grain.getZ()));
        }
        long placedAt = level.getGameTime();
        Runnable cleanUp = () -> {
            for (BlockPos p : controls)
                level.setBlock(p, Blocks.AIR.defaultBlockState(), 3);
            for (ChunkPos cp : forced)
                level.setChunkForced(cp.x, cp.z, false);
        };
        helper.startSequence()
                .thenWaitUntil(() -> {
                    helper.assertTrue(isLootr(level.getBlockState(chest)), "Lootr has not replaced the chest control at " + chest.toShortString()
                            + " (" + level.getBlockState(chest).getBlock() + "): with Lootr loaded but not converting these chunks, this check would see nothing");
                    if (suspicious != null)
                        helper.assertTrue(isLootr(level.getBlockState(grain)), "Lootr converts " + suspicious.getBlock()
                                + " in this server but has not replaced the control at " + grain.toShortString());
                })
                .thenIdle(20)
                .thenExecute(() -> {
                    PlacerScan after;
                    String chestNow = BuiltInRegistries.BLOCK.getKey(level.getBlockState(chest).getBlock()).toString();
                    String grainNow = suspicious == null
                            ? "left out: Lootr converts no suspicious block in this server (#lootr:convert/gravels and /sands hold neither)"
                            : suspicious.getBlock() + " became " + BuiltInRegistries.BLOCK.getKey(level.getBlockState(grain).getBlock());
                    try {
                        after = scanPlacer(level, wd, bar, true);
                    } finally {
                        cleanUp.run();
                    }
                    DeepTime.LOGGER.info("[gametest] placer lootr: {} chunks loaded in full ({} forced here); {} ticks after placing, "
                                    + "chest control became {}; suspicious control {}; bar cells converted={} panned before={} after={} filled after={}",
                            (nx + 5) * (nz + 5), forced.size(), level.getGameTime() - placedAt, chestNow, grainNow,
                            after.converted(), before.panned(), after.panned(), after.filled());
                    helper.assertTrue(after.converted() == 0, after.converted() + " of the bar's blocks became Lootr's: " + after.problems());
                    helper.assertTrue(after.panned() == before.panned(), "suspicious blocks before " + before.panned() + ", after " + after.panned());
                    helper.assertTrue(after.filled() == after.panned() && after.problems().isEmpty(),
                            (after.panned() - after.filled()) + " suspicious blocks lost their gold: " + after.problems());
                })
                .thenSucceed();
    }

    /**
     * Without Excavated Variants the ore blocks are normal ores: the {@code #c:ores_in_ground/stone}
     * block of the material (its deepslate block below y 0). With EV, logged only.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 6000)
    public static void vanillaOreBlock(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = data(helper, level);
        Target t = target(helper, level, wd);
        Scan s = scan(helper, level, wd, t);
        OreBlockResolver ores = OreBlockResolver.get();
        List<String> wrong = new ArrayList<>();
        for (String m : t.materials()) {
            ResourceLocation mat = ResourceLocation.parse(m);
            BlockState up = ores.resolve(mat, Blocks.STONE.defaultBlockState(), 10);
            BlockState down = ores.resolve(mat, Blocks.DEEPSLATE.defaultBlockState(), -10);
            DeepTime.LOGGER.info("[gametest] vanillaOreBlock {}: above 0 {}, below 0 {} (EV {})", m, up, down, ores.usesEv());
            if (!ores.usesEv()) {
                if (up == null || !up.is(net.neoforged.neoforge.common.Tags.Blocks.ORES_IN_GROUND_STONE))
                    wrong.add(m + " above 0: " + up);
                if (down != null && ores.blocks(mat).stream().anyMatch(b -> b.is(net.neoforged.neoforge.common.Tags.Blocks.ORES_IN_GROUND_DEEPSLATE))
                        && !down.is(net.neoforged.neoforge.common.Tags.Blocks.ORES_IN_GROUND_DEEPSLATE))
                    wrong.add(m + " below 0: " + down);
            }
        }
        helper.assertTrue(wrong.isEmpty(), "ore blocks: " + wrong + "; placed " + s.blocks());
        helper.succeed();
    }

    /**
     * With Excavated Variants installed, ore in an EV stone is EV's variant in that stone; without
     * it, the resolver never uses EV.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 6000)
    public static void evVariantWhenInstalled(GameTestHelper helper) {
        OreBlockResolver ores = OreBlockResolver.get();
        boolean installed = OreBlockResolver.evLoaded();
        if (!installed) {
            DeepTime.LOGGER.info("[gametest] evVariantWhenInstalled: Excavated Variants not installed; resolver uses EV = {}", ores.usesEv());
            helper.assertTrue(!ores.usesEv(), "the resolver uses Excavated Variants but it is not installed");
            helper.succeed();
            return;
        }
        helper.assertTrue(ores.usesEv(), "Excavated Variants is installed but the resolver does not use it (see the log)");
        var ev = ores.ev().orElseThrow();
        // Direct: copper in granite and in every EV stone the palette resolves.
        ResourceLocation copper = ResourceLocation.parse("c:copper");
        BlockState granite = Blocks.GRANITE.defaultBlockState();
        BlockState inGranite = ores.resolve(copper, granite, 20);
        DeepTime.LOGGER.info("[gametest] evVariantWhenInstalled: granite stone={} copper in granite={} variant={} ({})",
                ev.stoneOf(granite), inGranite, inGranite != null && ev.isVariant(inGranite), ev.describe());
        helper.assertTrue(ev.stoneOf(granite).isPresent(), "EV does not know granite as a stone");
        // The stand-ins EV lacks, shipped by Deep Time (defaultresources/globaldata/deeptime/…).
        List<String> shipped = new ArrayList<>(), missing = new ArrayList<>();
        for (var b : List.of(Blocks.POLISHED_TUFF, Blocks.PACKED_MUD, Blocks.SMOOTH_SANDSTONE, Blocks.COBBLESTONE)) {
            var st = ev.stoneOf(b.defaultBlockState());
            BlockState v = ores.resolve(copper, b.defaultBlockState(), 20);
            shipped.add(b + " stone=" + st.map(Object::toString).orElse("none") + " copper=" + v);
            if (st.isEmpty() || v == null || !ev.isVariant(v))
                missing.add(b.toString());
        }
        DeepTime.LOGGER.info("[gametest] evVariantWhenInstalled: Deep Time's EV stones {}", shipped);
        helper.assertTrue(missing.isEmpty(), "Deep Time's EV stone configs did not load for " + missing);
        helper.assertTrue(inGranite != null && ev.isVariant(inGranite)
                && ev.stoneOfVariant(inGranite).equals(ev.stoneOf(granite)), "copper in granite is " + inGranite);
        // A block that is two EV stones (smooth basalt: EV's overworld stone and Create's nether one) still
        // gets its overworld variant (Q10).
        BlockState inBasalt = ores.resolve(copper, Blocks.SMOOTH_BASALT.defaultBlockState(), 20);
        DeepTime.LOGGER.info("[gametest] evVariantWhenInstalled: copper in smooth basalt={} variant={}",
                inBasalt, inBasalt != null && ev.isVariant(inBasalt));
        helper.assertTrue(inBasalt != null && ev.isVariant(inBasalt), "copper in smooth basalt is " + inBasalt);
        // In the world: the site's ore blocks in EV stones are variants in those stones.
        ServerLevel level = level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = data(helper, level);
        Target t = target(helper, level, wd);
        Scan s = scan(helper, level, wd, t);
        long variants = s.blocks().keySet().stream().filter(k -> k.startsWith(EV_NS)).count();
        DeepTime.LOGGER.info("[gametest] evVariantWhenInstalled: placed {} ({} variant kinds)", s.blocks(), variants);
        helper.assertTrue(variants > 0, "no Excavated Variants block among the placed ores " + s.blocks());
        helper.assertTrue(s.exact() == s.placed(), "ore blocks differ from the resolver's: " + s.problems());
        helper.succeed();
    }

    /** Excavated Variants' generated blocks' namespace. */
    static final String EV_NS = "excavated_variants:";

    /**
     * Regeneration is identical: the same body generated in a second level over the same world data
     * holds the same block at every position (the draw is a position hash, not a random stream).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 6000)
    public static void depositsDeterministic(GameTestHelper helper) {
        ServerLevel a = level(helper, GameTestSupport.TEST_DEPOSITS), b = level(helper, GameTestSupport.TEST_DEPOSITS_TWIN);
        WorldData wd = data(helper, a);
        Target t = target(helper, a, wd);
        int c = wd.circumference();
        Body body = t.body();
        int compared = 0, differ = 0, ore = 0;
        String first = "";
        for (int x = body.min()[0]; x <= body.max()[0]; x++) {
            int xc = canonical(c, x);
            for (int z = body.min()[2]; z <= body.max()[2]; z++) {
                ChunkAccess ca = a.getChunkSource().getChunk(xc >> 4, z >> 4, ChunkStatus.FEATURES, true);
                ChunkAccess cb = b.getChunkSource().getChunk(xc >> 4, z >> 4, ChunkStatus.FEATURES, true);
                for (int y = Math.max(a.getMinBuildHeight(), body.min()[1]); y <= Math.min(a.getMaxBuildHeight() - 1, body.max()[1]); y++) {
                    BlockPos p = new BlockPos(xc, y, z);
                    BlockState sa = ca.getBlockState(p), sb = cb.getBlockState(p);
                    compared++;
                    if (sa.is(net.neoforged.neoforge.common.Tags.Blocks.ORES))
                        ore++;
                    if (sa != sb) {
                        differ++;
                        if (first.isEmpty())
                            first = p.toShortString() + ": " + sa + " vs " + sb;
                    }
                }
            }
        }
        DeepTime.LOGGER.info("[gametest] depositsDeterministic site={} body={} compared={} ore={} differ={} {}", t.site().id(),
                body.id(), compared, ore, differ, first);
        helper.assertTrue(compared > 1000 && ore > 0, "compared " + compared + " blocks, " + ore + " ore");
        helper.assertTrue(differ == 0, differ + " blocks differ between the two generations, first " + first);
        helper.succeed();
    }

    /**
     * The site API (plan §5.6) and the probes (plan §7.16): {@code at} finds the site inside its
     * body, {@code inChunk}/{@code within} list it, its forming event and cause chain exist, and
     * {@code probe site}, {@code sites near} and {@code why} answer for it.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void siteApiAndProbes(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = data(helper, level);
        DeepTimeSites sites = DeepTimeSites.of(level);
        helper.assertTrue(sites.isDeepTimeLevel(), "test_deposits is not a Deep Time level");
        helper.assertTrue(!DeepTimeSites.of(helper.getLevel()).isDeepTimeLevel(), "the GameTest overworld claims to be a Deep Time level");
        Target t = target(helper, level, wd);
        int c = wd.circumference();
        Body b = t.body();
        // The body's most central block (a clipped or thin body may have none with d < 0.3).
        BlockPos inside = null;
        double best = Double.MAX_VALUE;
        for (int y = b.max()[1]; y >= b.min()[1]; y--)
            for (int x = b.min()[0]; x <= b.max()[0]; x++)
                for (int z = b.min()[2]; z <= b.max()[2]; z++) {
                    var loc = b.locate(c, x, y, z);
                    if (loc != null && loc.d() < best) {
                        best = loc.d();
                        inside = new BlockPos(canonical(c, x), y, z);
                    }
                }
        helper.assertTrue(inside != null, "no block inside body " + b.id());
        var at = sites.at(inside);
        helper.assertTrue(at.isPresent() && at.get().id() == t.site().id(), "at(" + inside.toShortString() + ") = " + at);
        helper.assertTrue(sites.inChunk(new ChunkPos(inside)).stream().anyMatch(s -> s.id() == t.site().id()), "inChunk misses the site");
        helper.assertTrue(sites.within(new AABB(inside).inflate(2)).stream().anyMatch(s -> s.id() == t.site().id()), "within misses the site");
        helper.assertTrue(sites.byId(t.site().id()).isPresent(), "byId misses the site");
        var formed = sites.formingEvent(t.site());
        helper.assertTrue(formed.isPresent(), "no forming event " + t.site().formingEvent());
        var chain = sites.causeChain(formed.get(), 4);
        DeepTime.LOGGER.info("[gametest] siteApiAndProbes site={} {} at {} formed {} ({} causes, chain {} events: {})", t.site().id(),
                t.site().depositType(), inside.toShortString(), formed.get().kind(), formed.get().causes().size(), chain.size(),
                chain.stream().map(e -> e.kind().getPath()).toList());
        helper.assertTrue(!chain.isEmpty(), "the forming event has no cause chain");
        // Far from every site nothing is found.
        var source = level.getServer().createCommandSourceStack().withLevel(level).withSuppressedOutput().withPermission(2);
        int ok = SiteCommands.probeSite(source, inside.getX(), inside.getY(), inside.getZ())
                + SiteCommands.sitesNear(source, inside.getX(), inside.getZ(), 256)
                + SiteCommands.why(source, t.site().id());
        helper.assertTrue(ok == 3, "a site command failed");
        helper.succeed();
    }

    /** Background scatter places ore away from the sites too (plan §7.8), host-gated and deterministic. */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 6000)
    public static void backgroundScatterPlaces(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = data(helper, level);
        List<OreData.ScatterRule> rules = BackgroundScatterFeature.rules(level.registryAccess());
        helper.assertTrue(!rules.isEmpty(), "no deeptime:scatter rules loaded");
        OreBlockResolver ores = OreBlockResolver.get();
        RockPalette palette = RockPalette.get();
        int planned = 0, placed = 0, wrong = 0, chunks = 0;
        String problem = "";
        // A 4 × 4 chunk patch on land far from the sites (the spawn candidate's area).
        var spawn = com.minecraftindustrialcomplex.deeptime.server.SpawnHandler.chooseSpawn(level, wd).orElse(BlockPos.ZERO);
        int cx0 = (spawn.getX() >> 4) + 8, cz0 = (spawn.getZ() >> 4) + 8;
        for (int cx = cx0; cx < cx0 + 4; cx++) {
            for (int cz = cz0; cz < cz0 + 4; cz++) {
                ChunkAccess chunk = level.getChunkSource().getChunk(cx, cz, ChunkStatus.FEATURES, true);
                chunks++;
                for (OreData.ScatterRule r : rules) {
                    if (ores.blocks(r.material()).isEmpty())
                        continue;
                    for (var p : BackgroundScatterFeature.plan(wd, r, cx, cz, level.getMinBuildHeight())) {
                        planned++;
                        BlockState s = chunk.getBlockState(new BlockPos(p.x(), p.y(), p.z()));
                        if (OreBlockResolver.isOreOf(s, r.material()))
                            placed++;
                        else if (palette.host(s) == RockPalette.Host.ROCK
                                && p.draw() < BackgroundScatterFeature.bias(wd, palette, r, s, p.x(), p.y(), p.z())
                                && wd.deposits().bodiesIn(p.x(), p.y(), p.z(), p.x(), p.y(), p.z()).isEmpty()) {
                            wrong++;
                            if (problem.isEmpty())
                                problem = "(" + p.x() + "," + p.y() + "," + p.z() + ") " + s.getBlock() + " not " + r.material();
                        }
                    }
                }
            }
        }
        DeepTime.LOGGER.info("[gametest] backgroundScatterPlaces chunks={} rules={} planned={} placed={} wrong={} {}", chunks,
                rules.size(), planned, placed, wrong, problem);
        helper.assertTrue(placed > 0, "no scatter ore in " + chunks + " chunks (" + planned + " planned blocks)");
        helper.assertTrue(wrong == 0, wrong + " scatter blocks were due and not placed: " + problem);
        helper.succeed();
    }
}
