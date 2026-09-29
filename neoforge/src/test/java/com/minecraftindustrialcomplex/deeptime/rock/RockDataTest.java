// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * P1-12: the data Deep Time ships in its jar. The 59 {@code rock_class} definitions (plan §5.4.1),
 * the vanilla member tags, and no fallback tags at all (plan §7.5, Q15). The game parses the same
 * files with {@link RockClass#CODEC}; this test checks the rules the codec cannot.
 */
class RockDataTest {

    private static final Set<String> FAMILIES = Set.of("intrusive_igneous", "volcanic", "clastic_sedimentary",
            "chemical_sedimentary", "metamorphic", "special_host", "unconsolidated");

    private static Path data() {
        String root = System.getProperty("deeptime.repoRoot");
        if (root == null)
            throw new IllegalStateException("system property deeptime.repoRoot is unset: run through Gradle's test task");
        return Path.of(root, "neoforge/src/main/resources/data");
    }

    private static Map<String, JsonObject> classes() throws IOException {
        Map<String, JsonObject> out = new TreeMap<>();
        try (Stream<Path> files = Files.list(data().resolve("deeptime/deeptime/rock_class"))) {
            for (Path f : files.filter(p -> p.toString().endsWith(".json")).toList()) {
                String name = f.getFileName().toString().replaceAll("\\.json$", "");
                out.put(name, JsonParser.parseString(Files.readString(f, StandardCharsets.UTF_8)).getAsJsonObject());
            }
        }
        return out;
    }

    @Test
    void fiftyNineClassesWithTheirFields() throws IOException {
        Map<String, JsonObject> classes = classes();
        assertEquals(59, classes.size());
        List<String> bad = new ArrayList<>();
        classes.forEach((name, o) -> {
            if (!name.equals(o.get("class").getAsString()))
                bad.add(name + ": class " + o.get("class"));
            if (!FAMILIES.contains(o.get("family").getAsString()))
                bad.add(name + ": family " + o.get("family"));
            for (String num : List.of("density_kg_m3", "erodibility", "diffusivity"))
                if (!o.has(num) || o.get(num).getAsDouble() <= 0)
                    bad.add(name + ": " + num);
            for (String src : List.of("density_kg_m3", "erodibility", "diffusivity", "vanilla_fallback"))
                if (!o.has("sources") || !o.getAsJsonObject("sources").has(src))
                    bad.add(name + ": no source for " + src);
            String vf = o.get("vanilla_fallback").getAsString();
            if (!vf.startsWith("minecraft:") || vf.contains("terracotta"))
                bad.add(name + ": vanilla_fallback " + vf);
            if ((vf.equals("minecraft:stone") && !name.equals("coal")) || vf.equals("minecraft:deepslate"))
                bad.add(name + ": vanilla_fallback is filler " + vf);
            List<String> refs = new ArrayList<>();
            o.getAsJsonArray("metamorphic_path").forEach(e -> refs.add(e.getAsString()));
            for (String key : List.of("weathers_to", "lithifies_to", "loose_equivalent"))
                if (o.has(key))
                    refs.add(o.get(key).getAsString());
            String family = o.get("family").getAsString();
            if (o.has("lithifies_to") && !family.equals("unconsolidated"))
                bad.add(name + ": lithifies_to on a " + family + " class");
            if (o.has("loose_equivalent") && family.equals("unconsolidated"))
                bad.add(name + ": loose_equivalent on a loose class");
            for (String r : refs)
                if (!classes.containsKey(r))
                    bad.add(name + ": unknown class " + r);
            for (JsonElement g : o.getAsJsonArray("groups"))
                if (!g.getAsString().startsWith("deeptime:rock/"))
                    bad.add(name + ": group " + g);
        });
        assertTrue(bad.isEmpty(), bad.toString());
        assertEquals("minecraft:stone", classes.get("coal").get("vanilla_fallback").getAsString());
        assertEquals("sandstone", classes.get("sand").get("lithifies_to").getAsString());
        assertEquals("sand", classes.get("sandstone").get("loose_equivalent").getAsString());
    }

    @Test
    void memberTagsAreVanillaAndNamedAfterClasses() throws IOException {
        Map<String, JsonObject> classes = classes();
        Path tags = data().resolve("deeptime/tags/block/rock");
        int blocks = 0;
        try (Stream<Path> files = Files.list(tags)) {
            for (Path f : files.toList()) {
                String name = f.getFileName().toString().replaceAll("\\.json$", "");
                assertTrue(classes.containsKey(name) || name.equals("unclassified_crust"), "tag " + name + " names no class");
                JsonObject o = JsonParser.parseString(Files.readString(f, StandardCharsets.UTF_8)).getAsJsonObject();
                assertFalse(o.has("replace") && o.get("replace").getAsBoolean(), name + " replaces");
                for (JsonElement e : o.getAsJsonArray("values")) {
                    assertTrue(e.getAsString().startsWith("minecraft:"), name + ": " + e);
                    if (!name.equals("unclassified_crust"))
                        blocks++;
                }
            }
        }
        assertEquals(18, blocks, "the README's 18 vanilla member blocks");
    }

    @Test
    void noFallbackTagsShipped() {
        assertFalse(Files.exists(data().resolve("deeptime/tags/block/rock_fallback")),
                "Deep Time ships no fallback tags (plan §7.5, Q15): a pack's own fallback order must win");
    }
}
