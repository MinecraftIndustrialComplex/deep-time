// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Java↔Rust climate parity (P1-25): {@code deeptime parity} evaluated the climate at 500 blocks of
 * the real test planet with Rust's {@code ClimateWorld::sample} and wrote {@code parity/climate.json};
 * {@link ClimateLayers#sample}, which Deep Time's public climate API answers from, must give the same
 * for every one, bit for bit: the twelve months at the block's height, the mean, warmest and coldest
 * month, the annual and monthly precipitation and the Köppen class.
 */
class ClimateParityTest {

    static JsonObject fixture() throws IOException {
        return JsonParser.parseString(Files.readString(ColumnParityTest.world().resolve("parity/climate.json"),
                StandardCharsets.UTF_8)).getAsJsonObject();
    }

    private static List<String> differences(ClimateLayers.Sample s, JsonObject r) {
        List<String> out = new ArrayList<>();
        JsonArray t = r.getAsJsonArray("t_month"), p = r.getAsJsonArray("p_month");
        for (int m = 0; m < 12; m++) {
            if (s.tMonth()[m] != t.get(m).getAsDouble())
                out.add("t_month[" + m + "] " + s.tMonth()[m] + " vs " + t.get(m));
            if (s.pMonth()[m] != p.get(m).getAsDouble())
                out.add("p_month[" + m + "] " + s.pMonth()[m] + " vs " + p.get(m));
        }
        if (s.elevationM() != r.get("elevation_m").getAsDouble())
            out.add("elevation_m " + s.elevationM() + " vs " + r.get("elevation_m"));
        if (s.tMean() != r.get("t_mean").getAsDouble())
            out.add("t_mean " + s.tMean() + " vs " + r.get("t_mean"));
        if (s.tWarm() != r.get("t_warm").getAsDouble() || s.warmest() != r.get("warmest").getAsInt())
            out.add("warmest " + s.warmest() + "/" + s.tWarm() + " vs " + r.get("warmest") + "/" + r.get("t_warm"));
        if (s.tCold() != r.get("t_cold").getAsDouble() || s.coldest() != r.get("coldest").getAsInt())
            out.add("coldest " + s.coldest() + "/" + s.tCold() + " vs " + r.get("coldest") + "/" + r.get("t_cold"));
        if (s.pAnnual() != r.get("p_annual").getAsDouble())
            out.add("p_annual " + s.pAnnual() + " vs " + r.get("p_annual"));
        if (s.koppen() != r.get("koppen").getAsInt() || !s.koppenCode().equals(r.get("koppen_code").getAsString()))
            out.add("koppen " + s.koppen() + " " + s.koppenCode() + " vs " + r.get("koppen") + " " + r.get("koppen_code"));
        return out;
    }

    @Test
    void everySampleOfTheFixtureIsEqual() throws IOException {
        JsonObject doc = fixture();
        WorldData wd = WorldData.open(ColumnParityTest.world());
        assertEquals(FormatVersion.CURRENT, doc.get("format_version").getAsString());
        ClimateLayers cl = ClimateLayers.of(wd);
        assertNotNull(cl, "the real planet bakes the twelve months");
        assertEquals(doc.get("lapse_c_per_km").getAsDouble(), cl.lapseCPerKm());
        JsonArray samples = doc.getAsJsonArray("samples");
        assertEquals(500, samples.size());
        int equal = 0, land = 0, lapsed = 0;
        List<String> bad = new ArrayList<>();
        for (JsonElement e : samples) {
            JsonObject r = e.getAsJsonObject();
            long x = r.get("x").getAsLong(), z = r.get("z").getAsLong();
            int y = r.get("y").getAsInt();
            ClimateLayers.Sample s = cl.sample(x, y, z);
            List<String> diff = differences(s, r);
            if (diff.isEmpty())
                equal++;
            else if (bad.size() < 10)
                bad.add("(" + x + ", " + y + ", " + z + "): " + String.join("; ", diff));
            if (s.koppen() != ClimateLayers.SEA)
                land++;
            if (s.elevationM() > 0)
                lapsed++;
        }
        System.out.println("[junit] ClimateParityTest samples=" + samples.size() + " equal=" + equal + " land=" + land
                + " aboveSea=" + lapsed);
        assertTrue(bad.isEmpty(), (samples.size() - equal) + " samples differ:\n" + String.join("\n", bad));
        assertTrue(land > 0, "some samples are on land");
        assertTrue(lapsed > 0, "some samples are above the sea");
    }

    /** A synthetic world has no simulated climate: the API reports none rather than inventing one. */
    @Test
    void aSyntheticWorldHasNoClimate() {
        assertNull(ClimateLayers.of(WorldData.open(Fixtures.world("wrap1024"))));
    }
}
