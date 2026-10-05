// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeCensus;
import com.minecraftindustrialcomplex.deeptime.climate.ClimatePlacement;
import com.minecraftindustrialcomplex.deeptime.climate.PlacementBundle;
import com.minecraftindustrialcomplex.deeptime.climate.PlacementPacks;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.io.IOException;
import java.util.List;
import java.util.Locale;

/**
 * Pack placement tables (Ben, decisions log 2026-10-04, "Deep Time-side fixes (Recommended)") on the real baked planet
 * {@code deeptime:test_planet_real}: the bake used exactly the tables this server's biomes choose (with -PwithMic,
 * Terralith's; gametest.sh bakes the MIC planet with it, as a MIC server's first start would), the classes Terralith's
 * table fixes reach their intended biomes, and without a pack table (vanilla, Biomes O' Plenty) the world is baked by
 * the default table, unchanged.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class PlacementPackGameTests {

    private PlacementPackGameTests() {}

    /** A check counts only with at least this many columns (the 4k test planet is small: about 9 % land). */
    static final int MIN_COLUMNS = 30;

    private static JsonObject climateNote(GameTestHelper helper, ServerLevel level) {
        var t = RealPlanetGameTests.terrain(helper, level);
        JsonObject n = t.data().manifest().notes();
        for (String k : new String[]{"bake", "climate"}) {
            if (n == null || !n.has(k) || !n.get(k).isJsonObject()) {
                helper.fail("the real planet's world data has no notes.bake.climate");
                throw new IllegalStateException();
            }
            n = n.getAsJsonObject(k);
        }
        return n;
    }

    /**
     * The pack tables the bake recorded ({@code notes.bake.climate.placement.patches}) are the ones this server's biome
     * registry chooses from its resources ({@link ClimatePlacement#chosen}): Terralith's with MIC, none in vanilla or with
     * Biomes O' Plenty.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 600)
    public static void packTablesFollowTheInstalledBiomes(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        List<String> recorded = PlacementPacks.recorded(climateNote(helper, level));
        List<String> chosen;
        try {
            var server = level.getServer();
            chosen = PlacementPacks.ids(ClimatePlacement.chosen(ClimatePlacement.packs(server.getResourceManager()), server.registryAccess()));
        } catch (IOException e) {
            helper.fail(e.toString());
            return;
        }
        DeepTime.LOGGER.info("[gametest] packTablesFollowTheInstalledBiomes recorded={} chosen={}", recorded, chosen);
        helper.assertTrue(recorded.equals(chosen), "the bake used the pack tables " + recorded + " but this server's biomes choose " + chosen);
        helper.succeed();
    }

    /**
     * Without a pack table the world is baked by the default table exactly: the table the bake recorded is the jar's
     * {@code placement.json} (comment aside) and it has no class rules, so a vanilla world's biome inputs are those
     * of master before the pack tables (the core's bit-for-bit test holds the default's answers). With a pack table,
     * logged only.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 600)
    public static void withoutAPackTableTheDefaultTableBakes(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        JsonObject note = climateNote(helper, level);
        List<String> recorded = PlacementPacks.recorded(note);
        if (!recorded.isEmpty()) {
            DeepTime.LOGGER.info("[gametest] withoutAPackTableTheDefaultTableBakes: this planet bakes with {}; nothing to check", recorded);
            helper.succeed();
            return;
        }
        JsonObject jar;
        try (var in = PlacementPackGameTests.class.getClassLoader().getResourceAsStream("data/deeptime/" + PlacementBundle.PATH)) {
            jar = JsonParser.parseString(new String(in.readAllBytes(), java.nio.charset.StandardCharsets.UTF_8)).getAsJsonObject();
        } catch (IOException | NullPointerException e) {
            helper.fail("cannot read the jar's placement table: " + e);
            return;
        }
        jar.remove("_comment");
        JsonObject table = note.getAsJsonObject("placement").getAsJsonObject("table");
        DeepTime.LOGGER.info("[gametest] withoutAPackTableTheDefaultTableBakes source={} rules={}",
                note.getAsJsonObject("placement").get("source"), table.has("class_rules"));
        helper.assertTrue(table.equals(jar), "the bake's placement table is not the jar's default table");
        helper.assertTrue(!table.has("class_rules"), "the default table has class rules");
        helper.succeed();
    }

    /**
     * With Terralith (-PwithMic): each class its table fixes reaches its intended biomes ({@link BiomeCensus#PACK_CHECKS}:
     * Am in the jungles, Cs on shrubland and steppe, BSk on the steppe and dry shrublands, ET with few snowy plains) and
     * the humid tropical mountains in the cloud forest; every check with at least {@link #MIN_COLUMNS} columns meets its
     * bound. Without Terralith every check is inactive and the test passes with a line saying so.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void packTableClassesReachTheirBiomes(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        var t = RealPlanetGameTests.terrain(helper, level);
        BiomeCensus.Result r;
        try {
            r = BiomeCensus.run(level, Math.max(4, t.circumference() / 512), 32, null);
        } catch (IOException e) {
            helper.fail(e.toString());
            return;
        }
        DeepTime.LOGGER.info("[gametest] packTableClassesReachTheirBiomes pack_checks={} cloud_forest={} by_koppen={}",
                r.json().get("pack_checks"), r.json().get("cloud_forest"), r.json().get("by_koppen"));
        if (r.packs().isEmpty()) {
            DeepTime.LOGGER.info("[gametest] packTableClassesReachTheirBiomes: no pack table's biomes in this pack; nothing to check");
            helper.succeed();
            return;
        }
        StringBuilder bad = new StringBuilder();
        for (var pc : BiomeCensus.PACK_CHECKS) {
            int n = r.packTotal(pc.name());
            if (n < MIN_COLUMNS)
                continue;
            double share = r.packShare(pc.name());
            if (pc.atMost() ? share > pc.bound() : share < pc.bound())
                bad.append(String.format(Locale.ROOT, " %s %.3f %s %.3f (%d columns)", pc.name(), share, pc.atMost() ? ">" : "<",
                        pc.bound(), n));
        }
        int[] cloud = r.cloud();
        if (cloud[0] >= MIN_COLUMNS && cloud[1] < CLOUD_FLOOR * cloud[0])
            bad.append(String.format(Locale.ROOT, " cloud_forest %d of %d candidates (< %.2f)", cloud[1], cloud[0], CLOUD_FLOOR));
        helper.assertTrue(bad.isEmpty(), "pack table classes off their biomes:" + bad);
        helper.succeed();
    }

    /** The least share of the cloud forest rule's ground on the cloud forest (the census does not check P/PET). */
    static final double CLOUD_FLOOR = 0.3; // ledger: BIO-154 (game-choice)
}
