// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The bake's {@code meta/province} on the real test planet (P1-10): USGS-style classes cut into
 * contiguous provinces, the layer's ids are the notes' ids, and the notes carry the shares that
 * tell how orogen-heavy the world is.
 */
class ProvinceBakeTest {

    @Test
    void theBakedProvincesAreUsgsClassesWithIdsThatMatchTheLayer() {
        WorldData wd = WorldData.open(ColumnParityTest.world());
        assertTrue(wd.has("meta/province"));
        JsonObject p = wd.manifest().notes().getAsJsonObject("bake").getAsJsonObject("meta").getAsJsonObject("provinces");
        int count = p.get("count").getAsInt();
        assertTrue(count >= 3, "a real planet has several provinces, got " + count);
        Set<String> classes = new HashSet<>();
        for (JsonElement e : p.getAsJsonArray("classes"))
            classes.add(e.getAsString());
        assertTrue(classes.containsAll(Set.of("Shield", "Platform", "Orogen", "Basin", "Large igneous province", "Extended crust")));
        assertTrue(classes.stream().anyMatch(c -> c.startsWith("Oceanic crust")));
        JsonArray list = p.getAsJsonArray("province");
        assertEquals(count, list.size());
        double minKm2 = p.get("min_km2").getAsDouble();
        for (JsonElement e : list) {
            JsonObject o = e.getAsJsonObject();
            assertTrue(classes.contains(o.get("class").getAsString()), "unknown class in " + o);
        }
        // Every id the layer holds is a listed province; ids are the notes' ids.
        RasterSampler layer = wd.layer("meta/province");
        int w = layer.desc().grid().w(), h = layer.desc().grid().h();
        Set<Integer> seen = new HashSet<>();
        for (int j = 0; j < h; j++)
            for (int i = 0; i < w; i++)
                seen.add((int) layer.get(i, j));
        assertEquals(count, seen.size(), "the layer holds exactly the listed provinces");
        assertTrue(seen.stream().allMatch(id -> id >= 0 && id < count));
        // Provinces are not smaller than the preset's minimum (a tiny planet that has none large
        // enough is one province, so only checked when there are several).
        for (JsonElement e : list)
            assertTrue(e.getAsJsonObject().get("km2").getAsDouble() >= minKm2 * 0.999 || count == 1,
                    "province under the minimum: " + e);
    }

    @Test
    void theNotesCarryTheClassSharesAndTheParameters() {
        WorldData wd = WorldData.open(ColumnParityTest.world());
        JsonObject p = wd.manifest().notes().getAsJsonObject("bake").getAsJsonObject("meta").getAsJsonObject("provinces");
        for (String view : new String[]{"classified", "provinces"}) {
            JsonObject shares = p.getAsJsonObject("shares").getAsJsonObject(view);
            double planet = 0, continental = 0;
            for (var e : shares.getAsJsonObject("planet").entrySet())
                planet += e.getValue().getAsDouble();
            for (var e : shares.getAsJsonObject("continental").entrySet())
                continental += e.getValue().getAsDouble();
            assertEquals(1.0, planet, 2e-3, view);
            assertEquals(1.0, continental, 2e-3, view);
        }
        assertTrue(p.getAsJsonObject("shares").get("absorbed_cells").getAsLong() >= 0);
        // The test planet's preset names its minimum (it is a 1/66-area planet); the rest are defaults.
        JsonObject params = p.getAsJsonObject("params");
        assertEquals(1500.0, params.get("min_area_km2").getAsDouble(), 1e-9);
        assertEquals(500.0, params.get("orogen_max_age_myr").getAsDouble(), 1e-9);
        assertEquals(50.0, params.get("andean_max_age_myr").getAsDouble(), 1e-9);
        assertEquals(1500.0, params.get("andean_min_elevation_m").getAsDouble(), 1e-9);
    }
}
