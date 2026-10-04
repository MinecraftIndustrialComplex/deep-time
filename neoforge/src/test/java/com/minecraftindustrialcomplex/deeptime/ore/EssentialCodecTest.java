// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.mojang.serialization.JsonOps;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * An essential counts only sites whose product of the material is a primary one unless it says
 * {@code primary_only: false} (P1-15 on real planets, ledger DEP-206): the codec's default and the
 * shipped files must agree with the core's {@code Essential} (Rust, same field name and default).
 */
class EssentialCodecTest {

    private static OreData.Essential parse(JsonObject o) {
        return OreData.Essential.CODEC.parse(JsonOps.INSTANCE, o).getOrThrow();
    }

    private static JsonObject json(String s) {
        return JsonParser.parseString(s).getAsJsonObject();
    }

    @Test
    void primaryOnlyDefaultsToOnAndCanBeSwitchedOff() {
        var base = "{\"material\":\"c:copper\",\"min_sites_per_region\":1,\"region_blocks\":8192";
        assertTrue(parse(json(base + "}")).primaryOnly(), "the default counts primary products only");
        assertFalse(parse(json(base + ",\"primary_only\":false}")).primaryOnly());
        assertTrue(parse(json(base + ",\"primary_only\":true}")).primaryOnly());
    }

    @Test
    void theShippedEssentialsCountPrimaryProductsOnly() throws IOException {
        for (String name : new String[] {"coal", "copper", "gold", "iron"}) {
            String res = "data/deeptime/deeptime/essentials/" + name + ".json";
            try (InputStream in = EssentialCodecTest.class.getClassLoader().getResourceAsStream(res)) {
                assertTrue(in != null, "the jar ships " + res);
                JsonObject o = JsonParser.parseReader(new InputStreamReader(in, StandardCharsets.UTF_8)).getAsJsonObject();
                assertEquals(true, o.get("primary_only").getAsBoolean(), name + " says primary_only");
                assertTrue(parse(o).primaryOnly(), name);
            }
        }
    }
}
