// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * An essential counts only sites whose product of the material is a primary one (P1-15 on real
 * planets, ledger DEP-206): every shipped essential says so in the core's field name
 * ({@code primary_only}, which {@link OreData#contentJson} passes on). The codec's default and the
 * core's rule are covered by the Rust tests. No essential draws the spawn toward it any more (Ben,
 * 2026-10-05): none carries {@code max_blocks_from_spawn}.
 */
class ShippedEssentialsTest {

    @Test
    void theShippedEssentialsCountPrimaryProductsOnly() throws IOException {
        for (String name : new String[] {"coal", "copper", "gold", "iron"}) {
            String res = "data/deeptime/deeptime/essentials/" + name + ".json";
            try (InputStream in = ShippedEssentialsTest.class.getClassLoader().getResourceAsStream(res)) {
                assertTrue(in != null, "the jar ships " + res);
                JsonObject o = JsonParser.parseReader(new InputStreamReader(in, StandardCharsets.UTF_8)).getAsJsonObject();
                assertTrue(o.has("primary_only") && o.get("primary_only").getAsBoolean(), name + " says primary_only: true");
                assertTrue(!o.has("max_blocks_from_spawn"), name + " no longer sets a spawn reach");
                assertTrue(o.has("material") && o.has("min_sites_per_region") && o.has("region_blocks"), name + " has its fields");
            }
        }
    }
}
