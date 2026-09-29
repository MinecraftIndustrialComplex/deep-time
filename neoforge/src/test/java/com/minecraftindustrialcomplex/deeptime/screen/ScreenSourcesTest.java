// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;

/**
 * Static checks over the mod's sources (P1-11):
 * <ul>
 *   <li><b>Dist separation.</b> Client-only types ({@code net.minecraft.client}, {@code com.mojang.blaze3d},
 *       NeoForge's client events) appear only in the client packages: {@code screen} (whose entry point
 *       is the {@code dist = CLIENT} mod class) and P1-18's {@code client}; nothing outside them
 *       references a screen class, and the plain-Java {@code screen.model} references no Minecraft type
 *       at all, so JUnit and GameTests run it.</li>
 *   <li><b>Language keys.</b> Every key the screens and models name exists in {@code en_us.json}.</li>
 * </ul>
 */
class ScreenSourcesTest {

    static Path src() {
        String root = System.getProperty("deeptime.repoRoot");
        if (root == null)
            throw new IllegalStateException("deeptime.repoRoot is unset: run through Gradle");
        return Path.of(root, "neoforge/src/main/java/com/minecraftindustrialcomplex/deeptime");
    }

    static List<Path> javaFiles() throws IOException {
        try (Stream<Path> s = Files.walk(src())) {
            return s.filter(p -> p.toString().endsWith(".java")).sorted().toList();
        }
    }

    static String rel(Path p) {
        return src().relativize(p).toString().replace('\\', '/');
    }

    private static final Pattern CLIENT_IMPORT = Pattern.compile(
            "^import\\s+(net\\.minecraft\\.client\\.|com\\.mojang\\.blaze3d\\.|net\\.neoforged\\.neoforge\\.client\\.)", Pattern.MULTILINE);
    private static final Pattern SCREEN_REF = Pattern.compile("com\\.minecraftindustrialcomplex\\.deeptime\\.screen\\.(?!model\\.)[A-Z]");
    private static final Pattern MINECRAFT_IMPORT = Pattern.compile("^import\\s+(net\\.minecraft|net\\.neoforged|com\\.mojang)\\.", Pattern.MULTILINE);

    @Test
    void clientCodeStaysInTheScreenPackage() throws IOException {
        List<String> bad = new ArrayList<>();
        for (Path p : javaFiles()) {
            String r = rel(p);
            String text = Files.readString(p, StandardCharsets.UTF_8);
            // Client-only packages: screen/ (P1-11, not screen/model/) and client/ (P1-18).
            boolean screen = (r.startsWith("screen/") && !r.startsWith("screen/model/")) || r.startsWith("client/");
            if (!screen && CLIENT_IMPORT.matcher(text).find())
                bad.add(r + " imports a client-only type");
            if (!screen && SCREEN_REF.matcher(text).find())
                bad.add(r + " references a client screen class");
            if (r.startsWith("screen/model/") && MINECRAFT_IMPORT.matcher(text).find())
                bad.add(r + " (screen.model is plain Java) imports a Minecraft/NeoForge type");
        }
        assertEquals(List.of(), bad);
        String setup = Files.readString(src().resolve("screen/ClientSetup.java"), StandardCharsets.UTF_8);
        assertTrue(setup.contains("dist = Dist.CLIENT"), "the screens' entry point is client-only");
    }

    private static final Pattern TR = Pattern.compile("\\btr\\(\"([a-z0-9_.]+)\"\\s*[,)]");
    private static final Pattern PROBLEM = Pattern.compile("Problem\\.(?:of|warning)\\(\"([a-z0-9_.]+)\"");

    @Test
    void everyLanguageKeyExists() throws IOException {
        Path lang = Path.of(System.getProperty("deeptime.repoRoot"), "neoforge/src/main/resources/assets/deeptime/lang/en_us.json");
        JsonObject o = JsonParser.parseString(Files.readString(lang, StandardCharsets.UTF_8)).getAsJsonObject();
        Set<String> used = new TreeSet<>();
        for (Path p : javaFiles()) {
            String text = Files.readString(p, StandardCharsets.UTF_8);
            for (Pattern pat : List.of(TR, PROBLEM)) {
                Matcher m = pat.matcher(text);
                while (m.find())
                    used.add("deeptime." + m.group(1));
            }
        }
        assertTrue(used.size() > 60, "found only " + used.size() + " keys: the patterns no longer match the sources");
        // Keys built at run time.
        for (EditorFields.Field f : EditorFields.ALL) {
            used.add("deeptime." + f.labelKey());
            for (String option : f.options())
                used.add("deeptime." + f.optionKey(option));
        }
        for (EditorFields.Section s : EditorFields.Section.values())
            used.add("deeptime.editor.section." + s.name().toLowerCase(java.util.Locale.ROOT));
        for (String s : Presets.START_CONFIGS)
            used.add("deeptime.editor.timeline.start." + s);
        for (String n : Presets.BUILTIN)
            used.add("deeptime.editor.preset." + n.replaceAll("_\\d+k$", ""));
        for (String k : List.of("16k", "32k", "other"))
            used.add("deeptime.editor.size." + k);
        for (String k : List.of("world_data", "keyframe"))
            used.add("deeptime.review.source." + k);
        used.add("generator.deeptime.planet");
        used.add("generator.deeptime.planet_32k");
        List<String> missing = used.stream().filter(k -> !o.has(k)).toList();
        assertEquals(List.of(), missing);
    }
}
