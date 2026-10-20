// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * An essential counts only sites whose product of the material is a primary one (P1-15 on real
 * planets, ledger DEP-206): every shipped essential says so in the core's field name
 * ({@code primary_only}, which {@link OreData#contentJson} passes on), but lead, a co-product of
 * every type that makes it (ORE-273). The codec's default and the core's rule are covered by the
 * Rust tests. No essential draws the spawn toward it any more (Ben, 2026-10-05): none carries
 * {@code max_blocks_from_spawn}. An essential whose material has no ore block in the game is left
 * out of the content bundle ({@link EssentialsBundle#placeable}).
 */
class ShippedEssentialsTest {

    private static final String[] NAMES = {"coal", "copper", "gold", "iron", "lead", "zinc"};

    private static JsonObject shipped(String name) throws IOException {
        String res = "data/deeptime/deeptime/essentials/" + name + ".json";
        try (InputStream in = ShippedEssentialsTest.class.getClassLoader().getResourceAsStream(res)) {
            assertTrue(in != null, "the jar ships " + res);
            return JsonParser.parseReader(new InputStreamReader(in, StandardCharsets.UTF_8)).getAsJsonObject();
        }
    }

    @Test
    void theShippedEssentialsCountPrimaryProductsOnly() throws IOException {
        for (String name : NAMES) {
            JsonObject o = shipped(name);
            boolean primaryOnly = !name.equals("lead");
            assertTrue(o.has("primary_only") && o.get("primary_only").getAsBoolean() == primaryOnly,
                    name + " says primary_only: " + primaryOnly);
            assertTrue(!o.has("max_blocks_from_spawn"), name + " no longer sets a spawn reach");
            assertTrue(o.has("material") && o.has("min_sites_per_region") && o.has("region_blocks"), name + " has its fields");
            assertEquals("c:" + name, o.get("material").getAsString(), name + "'s material");
        }
    }

    /**
     * A game with vanilla's ores only (no Create, no lead mod): zinc and lead are left out of the
     * bundle and named; with every ore, all six go, in order.
     */
    @Test
    void anEssentialWithoutAnOreBlockIsLeftOut() throws IOException {
        JsonArray all = new JsonArray();
        for (String name : NAMES)
            all.add(shipped(name));
        Set<String> vanilla = Set.of("c:coal", "c:copper", "c:gold", "c:iron");
        List<String> skipped = new ArrayList<>();
        JsonArray kept = EssentialsBundle.placeable(all, vanilla::contains, skipped);
        assertEquals(4, kept.size());
        for (int i = 0; i < kept.size(); i++)
            assertTrue(vanilla.contains(kept.get(i).getAsJsonObject().get("material").getAsString()));
        assertEquals(List.of("c:lead", "c:zinc"), skipped);

        skipped.clear();
        assertEquals(all, EssentialsBundle.placeable(all, m -> true, skipped));
        assertTrue(skipped.isEmpty());
    }
}
