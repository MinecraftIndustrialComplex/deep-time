// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The climate class placement table (P1-09): the jar's copy is the core's default file, and a
 * datapack's table goes into the content bundle under {@code climate_placement}. The core parses
 * and checks the table (Rust {@code bake::placement} tests); Java only carries it.
 */
class PlacementBundleTest {

    private static final String RESOURCE = "data/deeptime/deeptime/climate_mapping/placement.json";

    private static JsonObject jarCopy() throws IOException {
        try (InputStream in = PlacementBundleTest.class.getClassLoader().getResourceAsStream(RESOURCE)) {
            assertTrue(in != null, "the jar ships " + RESOURCE);
            return PlacementBundle.parse(in);
        }
    }

    @Test
    void theJarShipsTheCoresDefaultTable() throws IOException {
        Path core = Path.of(System.getProperty("deeptime.repoRoot"), "crates", "deeptime-core", "data", "climate_mapping", "placement.json");
        JsonObject expected = JsonParser.parseString(Files.readString(core, StandardCharsets.UTF_8)).getAsJsonObject();
        JsonObject jar = jarCopy();
        assertEquals(expected, jar, "one file, two readers");
        assertEquals(1, jar.get("schema").getAsInt());
        assertEquals(31, jar.getAsJsonObject("classes").size(), "every land class (Af to EF)");
        assertEquals("tropical", jar.getAsJsonObject("classes").getAsJsonObject("Af").get("temperature").getAsString());
        assertEquals("data/deeptime/" + PlacementBundle.PATH, RESOURCE);
    }

    @Test
    void aPacksTableGoesIntoTheContentBundle() throws IOException {
        JsonObject table = jarCopy();
        table.getAsJsonObject("classes").getAsJsonObject("Af").addProperty("temperature", "desert");
        Optional<PlacementBundle.Found> found = Optional.of(new PlacementBundle.Found(table, "file/mypack"));

        // No bundle yet: the table alone.
        JsonObject alone = JsonParser.parseString(PlacementBundle.addTo("", found)).getAsJsonObject();
        assertEquals(table, alone.getAsJsonObject(PlacementBundle.KEY));
        assertEquals(1, alone.size());

        // With the essentials the server already put in it: both, the essentials untouched.
        String essentials = "{\"essentials\":[{\"material\":\"c:iron\",\"min_sites_per_region\":1,\"region_blocks\":4096,\"exposed\":true}]}";
        JsonObject both = JsonParser.parseString(PlacementBundle.addTo(essentials, found)).getAsJsonObject();
        assertEquals(JsonParser.parseString(essentials).getAsJsonObject().get("essentials"), both.get("essentials"));
        assertEquals("desert", both.getAsJsonObject(PlacementBundle.KEY).getAsJsonObject("classes").getAsJsonObject("Af")
                .get("temperature").getAsString());

        // No table: the bundle is passed on as it is.
        assertEquals(essentials, PlacementBundle.addTo(essentials, Optional.empty()));
        assertFalse(PlacementBundle.addTo("", Optional.empty()).contains(PlacementBundle.KEY));
    }

    @Test
    void aTableThatIsNotAnObjectIsRefused() {
        boolean refused = false;
        try {
            PlacementBundle.parse(new ByteArrayInputStream("[1, 2]".getBytes(StandardCharsets.UTF_8)));
        } catch (IllegalStateException | IOException e) {
            refused = true;
        }
        assertTrue(refused);
    }
}
