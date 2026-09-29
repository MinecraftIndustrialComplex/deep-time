// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.api.Site;
import com.minecraftindustrialcomplex.deeptime.compat.SiteChunks;
import com.minecraftindustrialcomplex.deeptime.compat.cdg.CdgAccess;
import com.minecraftindustrialcomplex.deeptime.compat.cdg.CdgBridge;
import com.minecraftindustrialcomplex.deeptime.compat.coe.CoeAccess;
import com.minecraftindustrialcomplex.deeptime.compat.coe.CoeBridge;
import com.minecraftindustrialcomplex.deeptime.ore.OreData;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worldgen.LevelSites;
import net.minecraft.core.RegistryAccess;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.chunk.LevelChunk;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.TreeMap;
import java.util.TreeSet;

/**
 * The COE and CDG hooks (P1-16, plan §7.14) on the GameTest server's Deep Time level
 * {@code deeptime:test_deposits} (the synthetic deposit world of P1-15, C = 4,096; the flat
 * overworld is not a Deep Time level and serves as the control "COE and CDG decide alone"). Deep
 * Time's side of each rule ({@link CoeBridge}, {@link CdgBridge}, {@link SiteChunks}) is checked in
 * every profile; the mods' own answers (their generators with Deep Time's mixins applied) only
 * when they are installed ({@code ./gametest.sh -PwithMic}). COE's generator is asked two ways: the
 * drills' path on the level's real chunk ({@code OreDataAttachment.getData}) and, where a real
 * chunk would cost generation or have the wrong biome, a detached {@code LevelChunk} of the level
 * (all plains, which COE's vein recipes accept; COE reads only the chunk's level, position and
 * biomes). Every assertion logs its numbers on a {@code [gametest]} line.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class CompatGameTests {

    private CompatGameTests() {}

    /** The COE veins Deep Time's deposit types name (plan §5.4.3; COE 1.6.8's own ids, which MIC uses). */
    static final List<String> DEEPTIME_VEINS = List.of("createoreexcavation:ore_vein_type/coal",
            "createoreexcavation:ore_vein_type/copper", "createoreexcavation:ore_vein_type/gold",
            "createoreexcavation:ore_vein_type/iron", "createoreexcavation:ore_vein_type/zinc");

    private static ServerLevel deposits(GameTestHelper helper) {
        return DepositGameTests.level(helper, GameTestSupport.TEST_DEPOSITS);
    }

    /** Per type with a vein, the first site (by id) whose ore body has blocks. */
    static List<Site> veinSitesPerType(ServerLevel level) {
        RegistryAccess access = level.registryAccess();
        LevelSites sites = LevelSites.of(level);
        Map<ResourceLocation, Site> first = new TreeMap<>();
        for (Site s : sites.all())
            if (CoeBridge.veinOf(access, s).isPresent() && SiteChunks.footprint(sites, s).total() > 0)
                first.putIfAbsent(s.depositType(), s);
        return List.copyOf(first.values());
    }

    /**
     * The site's ore-body blocks per canonical chunk, counted here block by block (independent of
     * {@link SiteChunks}' cache and lattice).
     */
    static Map<Long, Integer> oreBlocksByChunk(LevelSites sites, Site site) {
        int c = sites.deposits().circumference();
        Map<Long, Integer> out = new HashMap<>();
        for (Body b : sites.deposits().bodies().ofSite(site.id())) {
            if (!b.role().equals("ore"))
                continue;
            for (int x = b.min()[0]; x <= b.max()[0]; x++)
                for (int z = b.min()[2]; z <= b.max()[2]; z++)
                    for (int y = b.min()[1]; y <= b.max()[1]; y++)
                        if (b.locate(c, x, y, z) != null)
                            out.merge(ChunkPos.asLong(SiteChunks.canonicalChunkX(c, x >> 4), z >> 4), 1, Integer::sum);
        }
        return out;
    }

    private static void notInstalled(GameTestHelper helper, String test, String mod, String deepTimeSide) {
        DeepTime.LOGGER.info("[gametest] {}: {} not installed, its mixins skipped; Deep Time's side: {}", test, mod, deepTimeSide);
        helper.succeed();
    }

    /**
     * A site's chunk is the site's vein: for each deposit type with a vein, the chunk its first
     * site's ore fills most is that site's vein in Deep Time's decision and in COE's stored state
     * (the drills' path, on the generated chunk), and {@code OreVeinGenerator.pick} returns it.
     * Every vein id the deposit types name is one of COE's vein recipes.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 6000)
    public static void coePicksSiteVein(GameTestHelper helper) {
        ServerLevel level = deposits(helper);
        LevelSites sites = LevelSites.of(level);
        RegistryAccess access = level.registryAccess();
        List<Site> picked = veinSitesPerType(level);
        List<String> lines = new ArrayList<>(), problems = new ArrayList<>();
        Set<String> typeVeins = new TreeSet<>();
        for (var e : OreData.entries(access, OreData.DEPOSIT_TYPE))
            e.getValue().minecraft().coeVein().ifPresent(typeVeins::add);
        if (!typeVeins.equals(new TreeSet<>(DEEPTIME_VEINS)))
            problems.add("the deposit types name veins " + typeVeins + ", expected " + DEEPTIME_VEINS);
        for (Site s : picked) {
            ResourceLocation vein = CoeBridge.veinOf(access, s).orElseThrow();
            ChunkPos best = SiteChunks.footprint(sites, s).best();
            int blocks = oreBlocksByChunk(sites, s).getOrDefault(best.toLong(), 0);
            var decided = CoeBridge.siteVein(level, best.x, best.z);
            String line = "site " + s.id() + " " + s.depositType().getPath() + " chunk " + best.x + "," + best.z
                    + " ore blocks " + blocks + " → " + decided.map(v -> "site " + v.site().id() + " " + v.vein().getPath()).orElse("none");
            if (blocks == 0)
                problems.add("site " + s.id() + ": its ore body has no block in its best chunk " + best);
            if (decided.isEmpty())
                problems.add("site " + s.id() + ": no site vein decided at " + best);
            else if (decided.get().site().id() == s.id() && !decided.get().vein().equals(vein))
                problems.add("site " + s.id() + ": vein " + decided.get().vein() + ", its type names " + vein);
            lines.add(line);
        }
        var coe = CoeAccess.load();
        if (coe.isEmpty()) {
            DeepTime.LOGGER.info("[gametest] coePicksSiteVein: {}", lines);
            helper.assertTrue(picked.size() >= 5, "only " + picked.size() + " deposit types have a vein site: " + lines);
            helper.assertTrue(problems.isEmpty(), "problems: " + problems);
            notInstalled(helper, "coePicksSiteVein", "Create Ore Excavation", picked.size() + " vein sites decided");
            return;
        }
        Set<ResourceLocation> recipes = new HashSet<>();
        coe.get().veins(level).forEach(v -> recipes.add(v.id()));
        for (String v : typeVeins)
            if (!recipes.contains(ResourceLocation.parse(v)))
                problems.add("COE has no vein recipe " + v);
        int agree = 0;
        List<String> states = new ArrayList<>();
        for (Site s : picked) {
            ChunkPos best = SiteChunks.footprint(sites, s).best();
            var decided = CoeBridge.siteVein(level, best.x, best.z).orElse(null);
            LevelChunk chunk = level.getChunk(best.x, best.z);
            var state = coe.get().chunkState(chunk);
            var pick = coe.get().pick(chunk);
            states.add("site " + s.id() + " stored " + state.vein().map(ResourceLocation::getPath).orElse("none")
                    + " mul " + state.randomMul() + " pick " + pick.map(ResourceLocation::getPath).orElse("none"));
            if (decided == null)
                continue;
            if (!state.vein().equals(Optional.of(decided.vein())) || !pick.equals(Optional.of(decided.vein())))
                problems.add("site " + s.id() + " chunk " + best + ": COE stored " + state.vein() + ", picks " + pick
                        + ", Deep Time decided " + decided.vein());
            else
                agree++;
        }
        DeepTime.LOGGER.info("[gametest] coePicksSiteVein: {} vein sites {}; COE {}; agree {} of {}; COE vein recipes {}",
                picked.size(), lines, states, agree, picked.size(), recipes.size());
        helper.assertTrue(picked.size() >= 5, "only " + picked.size() + " deposit types have a vein site: " + lines);
        helper.assertTrue(problems.isEmpty(), "problems: " + problems);
        helper.assertTrue(agree == picked.size(), "COE agreed on " + agree + " of " + picked.size());
        helper.succeed();
    }

    /**
     * COE's noise grid does not hand out a claimed vein in a Deep Time level: at the grid chunks of
     * each claimed vein that no site reaches, the same (plains) chunk gets that vein from COE in the
     * flat overworld (the control) and no claimed vein in {@code test_deposits}; unclaimed veins
     * (diamond, lapis, redstone, …) are still COE's there.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void coeDropsClaimedNoise(GameTestHelper helper) {
        ServerLevel dt = deposits(helper), ow = helper.getLevel();
        Set<ResourceLocation> claimed = CoeBridge.claimedVeins(dt);
        List<String> problems = new ArrayList<>();
        for (String v : DEEPTIME_VEINS)
            if (!claimed.contains(ResourceLocation.parse(v)))
                problems.add("not claimed: " + v);
        if (!CoeBridge.claimedVeins(ow).isEmpty())
            problems.add("the flat overworld claims " + CoeBridge.claimedVeins(ow));
        var coe = CoeAccess.load();
        if (coe.isEmpty()) {
            helper.assertTrue(problems.isEmpty(), "problems: " + problems);
            notInstalled(helper, "coeDropsClaimedNoise", "Create Ore Excavation", "claimed " + claimed);
            return;
        }
        // Grid chunks of every vein within ±W chunks that no vein site reaches.
        final int w = 512;
        record Candidate(ResourceLocation vein, ChunkPos pos) {}
        Set<Candidate> candidates = new LinkedHashSet<>();
        for (var v : coe.get().veins(dt))
            for (int gx = -w; gx < w; gx += v.spacing())
                for (int gz = -w; gz < w; gz += v.spacing())
                    coe.get().gridChunk(dt, v.id(), gx, gz).ifPresent(p -> {
                        if (CoeBridge.siteVein(dt, p.x, p.z).isEmpty())
                            candidates.add(new Candidate(v.id(), p));
                    });
        int claimedTried = 0, controls = 0, dropped = 0, unclaimedTried = 0, unclaimedKept = 0;
        Map<String, Integer> droppedBy = new TreeMap<>(), tried = new HashMap<>();
        for (Candidate cand : candidates) {
            boolean isClaimed = claimed.contains(cand.vein());
            if (tried.merge(cand.vein().toString(), 1, Integer::sum) > 6)
                continue;   // at most six grid chunks per vein
            var inDt = coe.get().pick(new LevelChunk(dt, cand.pos()));
            var inOw = coe.get().pick(new LevelChunk(ow, cand.pos()));
            var populated = coe.get().populate(new LevelChunk(dt, cand.pos())).vein();
            if (isClaimed) {
                claimedTried++;
                if (inOw.equals(Optional.of(cand.vein()))) {
                    controls++;
                    if (inDt.isEmpty() && populated.isEmpty()) {
                        dropped++;
                        droppedBy.merge(cand.vein().getPath(), 1, Integer::sum);
                    }
                }
                if (inDt.map(claimed::contains).orElse(false) || populated.map(claimed::contains).orElse(false))
                    problems.add(cand.vein().getPath() + " at " + cand.pos() + ": COE gave " + inDt + " / stored " + populated
                            + " in a Deep Time level");
            } else {
                unclaimedTried++;
                if (!inDt.equals(inOw))
                    problems.add("unclaimed " + cand.vein().getPath() + " at " + cand.pos() + ": overworld " + inOw + ", Deep Time " + inDt);
                else if (inDt.equals(Optional.of(cand.vein())))
                    unclaimedKept++;
            }
        }
        DeepTime.LOGGER.info("[gametest] coeDropsClaimedNoise: claimed {}; {} grid candidates; claimed tried {}, COE picks them in the overworld {}, "
                        + "dropped in Deep Time {} {}; unclaimed tried {}, kept {}; problems {}",
                claimed, candidates.size(), claimedTried, controls, dropped, droppedBy, unclaimedTried, unclaimedKept, problems);
        helper.assertTrue(problems.isEmpty(), "problems: " + problems);
        helper.assertTrue(controls > 0, "COE's noise picked none of " + claimedTried + " claimed grid chunks in the overworld (no control)");
        helper.assertTrue(dropped == controls, "dropped " + dropped + " of " + controls);
        helper.assertTrue(unclaimedKept > 0, "no unclaimed vein kept among " + unclaimedTried);
        helper.succeed();
    }

    /**
     * A site vein's {@code randomMul} is the site's tonnage quantile: equal to Rust's drawn quantile
     * ({@code tonnage_quantile} of the forming event) for deposits that kept all of their ore, and
     * rising with tonnage within every type; COE's populate stores it (detached chunks of the Deep
     * Time level at each site's best chunk).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void coeAmountScalesWithTonnage(GameTestHelper helper) {
        ServerLevel level = deposits(helper);
        LevelSites sites = LevelSites.of(level);
        RegistryAccess access = level.registryAccess();
        List<String> problems = new ArrayList<>();
        int rustCompared = 0;
        double worst = 0;
        Map<ResourceLocation, List<double[]>> byType = new TreeMap<>();   // [tonnage, q, randomMul]
        var coe = CoeAccess.load();
        int stored = 0;
        List<String> amounts = new ArrayList<>();
        for (Site s : sites.all()) {
            if (CoeBridge.veinOf(access, s).isEmpty())
                continue;
            ChunkPos best = SiteChunks.footprint(sites, s).best();
            if (best == null)
                continue;
            float q = CoeBridge.quantile(access, s);
            double tonnage = s.products().get(0).tonnageT();
            if (!(q > 0 && q < 1))
                problems.add("site " + s.id() + ": quantile " + q);
            var ev = sites.formingEvent(s);
            if (ev.isPresent() && ev.get().params().get("tonnage_quantile") instanceof Number rq
                    && ev.get().params().get("tonnage_mt") instanceof Number mt
                    && Math.abs(mt.doubleValue() * 1e6 - tonnage) <= 1e-6 * tonnage) {
                rustCompared++;
                worst = Math.max(worst, Math.abs(rq.doubleValue() - q));
                if (Math.abs(rq.doubleValue() - q) > 1e-4)
                    problems.add("site " + s.id() + ": quantile " + q + ", Rust drew " + rq);
            }
            float mul = q;
            if (coe.isPresent()) {
                var decided = CoeBridge.siteVein(level, best.x, best.z);
                if (decided.isPresent() && decided.get().site().id() == s.id()) {
                    var state = coe.get().populate(new LevelChunk(level, best));
                    mul = state.randomMul();
                    stored++;
                    if (mul != q)
                        problems.add("site " + s.id() + ": COE stored randomMul " + mul + ", quantile " + q);
                    amounts.add(s.id() + ":" + state.amount() + (state.infinite() ? "(inf)" : ""));
                }
            }
            byType.computeIfAbsent(s.depositType(), k -> new ArrayList<>()).add(new double[]{tonnage, q, mul});
        }
        int orderedTypes = 0;
        for (var e : byType.entrySet()) {
            List<double[]> l = e.getValue();
            l.sort(Comparator.comparingDouble(a -> a[0]));
            for (int i = 1; i < l.size(); i++)
                if (l.get(i)[1] < l.get(i - 1)[1] || l.get(i)[2] < l.get(i - 1)[2])
                    problems.add(e.getKey().getPath() + ": quantile falls from " + l.get(i - 1)[1] + " to " + l.get(i)[1]
                            + " as tonnage rises " + l.get(i - 1)[0] + " → " + l.get(i)[0]);
            if (l.size() > 1)
                orderedTypes++;
        }
        StringBuilder summary = new StringBuilder();
        byType.forEach((t, l) -> summary.append(t.getPath()).append(' ').append(l.size()).append(" sites q ")
                .append(String.format("%.3f–%.3f", l.get(0)[1], l.get(l.size() - 1)[1])).append("; "));
        DeepTime.LOGGER.info("[gametest] coeAmountScalesWithTonnage: {}compared with Rust's drawn quantile {} (worst |Δ| {}); "
                        + "types ordered {}; COE stored {} {}; amounts {}; problems {}",
                summary, rustCompared, worst, orderedTypes, stored, coe.map(CoeAccess::config).orElse("COE not installed"), amounts, problems);
        helper.assertTrue(rustCompared >= 5, "only " + rustCompared + " sites to compare with Rust's quantile");
        helper.assertTrue(orderedTypes >= 3, "only " + orderedTypes + " types with two or more vein sites");
        helper.assertTrue(problems.isEmpty(), "problems: " + problems);
        if (coe.isEmpty()) {
            notInstalled(helper, "coeAmountScalesWithTonnage", "Create Ore Excavation", rustCompared + " quantiles match Rust");
            return;
        }
        helper.assertTrue(stored >= 10, "COE stored the quantile at only " + stored + " site chunks");
        helper.succeed();
    }

    /** The traps of the level: sites that feed CDG. */
    static List<Site> traps(ServerLevel level) {
        RegistryAccess access = level.registryAccess();
        return LevelSites.of(level).all().stream().filter(s -> CdgBridge.feedsCdg(access, s)).toList();
    }

    /**
     * A trap's oil: in every chunk its ore body reaches, the base oil is the trap's charge × the
     * type's {@code cdg_scale} × the chunk's share of the body's volume (counted here block by
     * block), infinite above CDG's threshold; what a pumpjack reads ({@code getChunkOilAmount}) is
     * the same; the finite shares add up to the scaled charge. Where two traps reach one chunk,
     * the one that fills it more supplies it ({@code SiteChunks.covering}): each trap's shares
     * still add up to its scaled charge, and the per-chunk check runs on the chunks it supplies.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void cdgOilFromTrap(GameTestHelper helper) {
        ServerLevel level = deposits(helper);
        LevelSites sites = LevelSites.of(level);
        RegistryAccess access = level.registryAccess();
        var cdg = CdgAccess.load();
        int threshold = cdg.map(CdgAccess::infiniteThreshold).orElse(CdgBridge.DEFAULT_INFINITE_THRESHOLD);
        List<Site> traps = traps(level);
        List<String> problems = new ArrayList<>(), lines = new ArrayList<>();
        int chunks = 0, positive = 0, infinite = 0;
        for (Site t : traps) {
            Map<Long, Integer> counts = oreBlocksByChunk(sites, t);
            long total = counts.values().stream().mapToLong(Integer::longValue).sum();
            long charge = t.extraction().cdgChargeMb().orElseThrow();
            double scale = CdgBridge.scaleOf(access, t);
            double sum = 0;
            int trapPositive = 0, trapInfinite = 0;
            long max = 0;
            boolean exact = SiteChunks.footprint(sites, t).stride() == 1;
            for (var e : counts.entrySet()) {
                ChunkPos p = new ChunkPos(e.getKey());
                long mb = Math.round((double) charge * scale * e.getValue() / total);
                int expected = mb > threshold ? Integer.MAX_VALUE : (int) mb;
                // The trap's own share of every chunk it reaches adds up to its scaled charge,
                // whether or not it is the trap that fills the chunk.
                if (expected == Integer.MAX_VALUE)
                    trapInfinite++;
                else
                    sum += expected;
                var covering = CdgBridge.charge(level, p.x, p.z);
                if (covering.isEmpty() || covering.get().site().id() != t.id())
                    continue;   // another trap fills this chunk more: CDG sees that one there
                chunks++;
                int decided = CdgBridge.baseOilAmount(level, p, threshold).orElse(-1);
                boolean ok = exact ? decided == expected : Math.abs(decided - expected) <= 0.02 * expected + 1;
                if (!ok)
                    problems.add("trap " + t.id() + " chunk " + p + ": Deep Time " + decided + ", expected " + expected);
                if (cdg.isPresent()) {
                    int base = cdg.get().base(level, p), amount = cdg.get().amount(level, p);
                    if (base != decided || amount != decided)
                        problems.add("trap " + t.id() + " chunk " + p + ": CDG base " + base + " amount " + amount + ", Deep Time " + decided);
                }
                if (expected == Integer.MAX_VALUE)
                    infinite++;
                if (expected > 0) {
                    positive++;
                    trapPositive++;
                }
                max = Math.max(max, mb);
            }
            if (trapPositive == 0)
                problems.add("trap " + t.id() + " has no chunk with oil");
            lines.add("trap " + t.id() + " charge " + charge + " mB × " + scale + " over " + counts.size() + " chunks ("
                    + total + " blocks), max chunk " + max + " mB, finite sum " + Math.round(sum));
            double own = charge * scale;   // every chunk the trap reaches adds its share, supplied or not
            if (trapInfinite == 0 && Math.abs(sum - own) > counts.size())
                problems.add("trap " + t.id() + ": the chunks add up to " + sum + ", the scaled charge of its own chunks is " + own
                        + " over " + counts.size() + " chunks");
        }
        DeepTime.LOGGER.info("[gametest] cdgOilFromTrap: {} traps {}; chunks {}, with oil {}, infinite {} (threshold {}); CDG {}; problems {}",
                traps.size(), lines, chunks, positive, infinite, threshold, cdg.map(CdgAccess::config).orElse("not installed"), problems);
        helper.assertTrue(!traps.isEmpty(), "no trap in the deposit world");
        helper.assertTrue(problems.isEmpty(), "problems: " + problems);
        if (cdg.isEmpty()) {
            notInstalled(helper, "cdgOilFromTrap", "Create: Diesel Generators", chunks + " trap chunks decided");
            return;
        }
        helper.succeed();
    }

    /**
     * No oil outside traps in a Deep Time level: 0 at every vein site's chunk, on the ring of chunks
     * around each trap and on a grid across the world; CDG's own generator decides in the flat
     * overworld (the hook stays out, and CDG's noise finds oil there: the control).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void cdgZeroElsewhere(GameTestHelper helper) {
        ServerLevel dt = deposits(helper), ow = helper.getLevel();
        LevelSites sites = LevelSites.of(dt);
        Set<Long> trapChunks = new HashSet<>();
        for (Site t : traps(dt))
            trapChunks.addAll(oreBlocksByChunk(sites, t).keySet());
        Set<Long> probes = new LinkedHashSet<>();
        for (Site s : veinSitesPerType(dt)) {
            ChunkPos best = SiteChunks.footprint(sites, s).best();
            if (best != null)
                probes.add(best.toLong());
        }
        for (long k : trapChunks) {
            ChunkPos p = new ChunkPos(k);
            for (int dx = -1; dx <= 1; dx++)
                for (int dz = -1; dz <= 1; dz++)
                    probes.add(ChunkPos.asLong(p.x + dx, p.z + dz));
        }
        List<Long> grid = new ArrayList<>();
        for (int cx = -120; cx <= 120; cx += 30)
            for (int cz = -56; cz <= 56; cz += 16)
                grid.add(ChunkPos.asLong(cx, cz));
        probes.addAll(grid);
        probes.removeAll(trapChunks);
        var cdg = CdgAccess.load();
        int threshold = cdg.map(CdgAccess::infiniteThreshold).orElse(CdgBridge.DEFAULT_INFINITE_THRESHOLD);
        List<String> problems = new ArrayList<>();
        int owOil = 0, owChecked = 0;
        for (long k : probes) {
            ChunkPos p = new ChunkPos(k);
            if (CdgBridge.baseOilAmount(dt, p, threshold).orElse(-1) != 0)
                problems.add("Deep Time decides " + CdgBridge.baseOilAmount(dt, p, threshold) + " at " + p);
            if (cdg.isPresent()) {
                int base = cdg.get().base(dt, p), amount = cdg.get().amount(dt, p);
                if (base != 0 || amount != 0)
                    problems.add("CDG finds " + base + " / " + amount + " mB at " + p + " in the Deep Time level");
            }
        }
        for (long k : grid) {
            ChunkPos p = new ChunkPos(k);
            if (CdgBridge.baseOilAmount(ow, p, threshold).isPresent())
                problems.add("the hook decides in the flat overworld at " + p);
            if (cdg.isPresent()) {
                owChecked++;
                if (cdg.get().base(ow, p) > 0)
                    owOil++;
            }
        }
        DeepTime.LOGGER.info("[gametest] cdgZeroElsewhere: {} chunks outside {} trap chunks, all 0 in Deep Time; flat overworld (CDG's own): oil in {} of {}; CDG {}; problems {}",
                probes.size(), trapChunks.size(), owOil, owChecked, cdg.map(CdgAccess::config).orElse("not installed"), problems);
        helper.assertTrue(problems.isEmpty(), "problems: " + problems);
        if (cdg.isEmpty()) {
            notInstalled(helper, "cdgZeroElsewhere", "Create: Diesel Generators", probes.size() + " chunks decided 0");
            return;
        }
        helper.succeed();
    }
}
