// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

/** The world-data fixtures Gradle's {@code testWorlds} task writes with {@code deeptime synth}. */
final class Fixtures {

    private Fixtures() {}

    static Path dir() {
        String d = System.getProperty("deeptime.testWorlds");
        if (d == null)
            throw new IllegalStateException("system property deeptime.testWorlds is unset: run through Gradle's test task");
        return Path.of(d);
    }

    static Path world(String name) {
        return dir().resolve(name);
    }

    static JsonObject samples(String name) {
        try {
            return JsonParser.parseString(Files.readString(dir().resolve(name + "-samples.json"), StandardCharsets.UTF_8))
                    .getAsJsonObject();
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
    }

    static Path repoRoot() {
        String d = System.getProperty("deeptime.repoRoot");
        if (d == null)
            throw new IllegalStateException("system property deeptime.repoRoot is unset: run through Gradle's test task");
        return Path.of(d);
    }
}
