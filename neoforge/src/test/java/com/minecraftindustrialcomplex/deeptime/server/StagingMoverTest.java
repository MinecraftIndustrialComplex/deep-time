// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.FileTime;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneOffset;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * Staging folders and their move into a world (plan §7.12): a rename on one filesystem, else copy,
 * verify CRCs and delete — here on the wrap-test world data {@code deeptime synth} writes (Gradle's
 * {@code testWorlds}), which has tile packs, a manifest with CRCs and spawn candidates.
 */
class StagingMoverTest {

    @TempDir
    Path gameDir;

    static Path fixture() {
        String d = System.getProperty("deeptime.testWorlds");
        if (d == null)
            throw new IllegalStateException("system property deeptime.testWorlds is unset: run through Gradle's test task");
        return Path.of(d).resolve("wrap1024");
    }

    Path stage(String id) throws IOException {
        Path dir = Staging.dir(gameDir, id);
        copyTree(fixture(), dir);
        return dir;
    }

    static void copyTree(Path from, Path to) throws IOException {
        Files.walkFileTree(from, new SimpleFileVisitor<>() {
            @Override
            public FileVisitResult preVisitDirectory(Path d, BasicFileAttributes a) throws IOException {
                Files.createDirectories(to.resolve(from.relativize(d).toString()));
                return FileVisitResult.CONTINUE;
            }

            @Override
            public FileVisitResult visitFile(Path f, BasicFileAttributes a) throws IOException {
                Files.copy(f, to.resolve(from.relativize(f).toString()));
                return FileVisitResult.CONTINUE;
            }
        });
    }

    /** Relative path → bytes CRC, for comparing whole trees. */
    static Map<String, Long> tree(Path root) throws IOException {
        Map<String, Long> out = new TreeMap<>();
        try (var s = Files.walk(root)) {
            for (Path p : s.filter(Files::isRegularFile).toList())
                out.put(root.relativize(p).toString(), StagingMover.crc32(p));
        }
        return out;
    }

    @Test
    void idsAndPaths() {
        String id = Staging.newId();
        assertTrue(Staging.isId(id));
        assertEquals(gameDir.resolve("deeptime/staging/" + id), Staging.dir(gameDir, id));
        assertThrows(IllegalArgumentException.class, () -> Staging.dir(gameDir, "../../etc"));
        assertThrows(IllegalArgumentException.class, () -> Staging.dir(gameDir, "ABCDEF00-0000-0000-0000-000000000000"));
        assertFalse(Staging.isId(null));
    }

    @Test
    void renameOnOneFilesystem() throws Exception {
        String id = Staging.newId();
        Path staging = stage(id);
        Map<String, Long> before = tree(staging);
        Path target = gameDir.resolve("saves/New World/deeptime");
        StagingMover.Result r = StagingMover.move(staging, target, false);
        assertEquals(StagingMover.Kind.RENAMED, r.kind());
        assertFalse(Files.exists(staging));
        assertEquals(before, tree(target));
        WorldData wd = WorldData.open(target);
        wd.verify();
        assertEquals(1024, wd.circumference());
        // A restart: the data is there, nothing moves.
        assertEquals(StagingMover.Kind.ALREADY_PRESENT, StagingMover.move(staging, target, false).kind());
    }

    @Test
    void copyVerifyAndDelete() throws Exception {
        Path staging = stage(Staging.newId());
        Map<String, Long> before = tree(staging);
        Path target = gameDir.resolve("other-drive/world/deeptime");
        StagingMover.Result r = StagingMover.move(staging, target, true);
        assertEquals(StagingMover.Kind.COPIED, r.kind());
        assertEquals(before.size(), r.files());
        assertTrue(r.bytes() > 0);
        assertFalse(Files.exists(staging), "deleted after the copy was verified");
        assertFalse(Files.exists(target.resolveSibling("deeptime" + StagingMover.COPY_SUFFIX)));
        assertEquals(before, tree(target));
        WorldData.open(target).verify();
    }

    /** A copy that does not match its manifest is removed and the staged data stays as it was. */
    @Test
    void failedVerificationKeepsTheStagedData() throws Exception {
        Path staging = stage(Staging.newId());
        Path manifest = staging.resolve("manifest.json");
        String m = Files.readString(manifest, StandardCharsets.UTF_8);
        // Break the first listed CRC.
        String broken = m.replaceFirst("\"crc32\":\\s*(\\d+)", "\"crc32\": 1");
        assertTrue(!broken.equals(m), "the fixture manifest lists CRCs");
        Files.writeString(manifest, broken, StandardCharsets.UTF_8);
        Map<String, Long> before = tree(staging);
        Path target = gameDir.resolve("w/deeptime");
        StagingMover.StagingException e = assertThrows(StagingMover.StagingException.class,
                () -> StagingMover.move(staging, target, true));
        assertTrue(e.getMessage().contains("untouched"), e.getMessage());
        assertEquals(before, tree(staging));
        assertFalse(Files.exists(target));
        assertFalse(Files.exists(target.resolveSibling("deeptime" + StagingMover.COPY_SUFFIX)));
    }

    @Test
    void incompleteStagingIsRefused() throws Exception {
        Path staging = stage(Staging.newId());
        Files.delete(staging.resolve("manifest.json"));
        Path target = gameDir.resolve("w/deeptime");
        assertThrows(StagingMover.StagingException.class, () -> StagingMover.move(staging, target, false));
        assertThrows(StagingMover.StagingException.class, () -> StagingMover.move(gameDir.resolve("nope"), target, false));
        assertFalse(Files.exists(target));
    }

    /** A leftover partial target (no manifest) is replaced, never merged. */
    @Test
    void partialTargetIsReplaced() throws Exception {
        Path staging = stage(Staging.newId());
        Path target = gameDir.resolve("w/deeptime");
        Files.createDirectories(target.resolve("rasters"));
        Files.writeString(target.resolve("rasters/stale.dtr"), "stale");
        StagingMover.move(staging, target, false);
        assertFalse(Files.exists(target.resolve("rasters/stale.dtr")));
        WorldData.open(target).verify();
    }

    @Test
    void cleanupRemovesOnlyOldUnheldFolders() throws Exception {
        String oldId = Staging.newId(), heldId = Staging.newId(), freshId = Staging.newId();
        Path old = stage(oldId), held = stage(heldId), fresh = stage(freshId);
        FileTime longAgo = FileTime.from(Instant.parse("2026-01-01T00:00:00Z"));
        for (Path p : List.of(old, held)) {
            Files.setLastModifiedTime(p.resolve("manifest.json"), longAgo);
            Files.setLastModifiedTime(p, longAgo);
        }
        Clock now = Clock.fixed(Instant.parse("2026-09-28T12:00:00Z"), ZoneOffset.UTC);
        // `fresh` has today's real mtime, far in the fixed clock's future: kept.
        List<Path> removed = Staging.cleanupAbandoned(gameDir, Duration.ofHours(24), Set.of(heldId), now);
        assertEquals(List.of(old), removed);
        assertFalse(Files.exists(old));
        assertTrue(Files.exists(held), "held by a live editor");
        assertTrue(Files.exists(fresh));
        assertTrue(Staging.cleanupAbandoned(gameDir.resolve("nowhere"), Duration.ZERO, Set.of(), now).isEmpty());
    }
}
