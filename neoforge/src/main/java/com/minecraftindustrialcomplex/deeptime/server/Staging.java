// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;

import java.io.IOException;
import java.nio.file.DirectoryStream;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.NoSuchFileException;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.FileTime;
import java.time.Clock;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import java.util.UUID;
import java.util.regex.Pattern;

/**
 * The staging folders of the Create World screen (plan §7.12, glossary "Staging"):
 * {@code <gamedir>/deeptime/staging/<stagingId>/}. A singleplayer run writes its world data there
 * because the world folder does not exist yet; {@link StagingMover} moves it into
 * {@code <world>/deeptime/} when the new world's server starts. Plain file operations, no Minecraft.
 *
 * <p>Abandoned folders (a run the player never accepted, a Create World screen left with Cancel,
 * a crash) are removed by {@link #cleanupAbandoned}: a folder goes once nothing in it changed for
 * {@link #ABANDONED_AFTER}, unless it is one the caller still holds. A running simulation rewrites
 * {@code keyframes/index.json} with every frame, so another game instance's live run keeps its
 * folder fresh.
 */
public final class Staging {

    /** How long an untouched staging folder is kept. */
    public static final Duration ABANDONED_AFTER = Duration.ofHours(24);

    private static final Pattern ID = Pattern.compile("[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}");

    private Staging() {}

    /** {@code <gamedir>/deeptime/staging}. */
    public static Path root(Path gameDir) {
        return gameDir.resolve("deeptime").resolve("staging");
    }

    /** A new staging id (a random UUID). */
    public static String newId() {
        return UUID.randomUUID().toString();
    }

    /** True for a well-formed staging id (a lower-case UUID): never a path. */
    public static boolean isId(String id) {
        return id != null && ID.matcher(id).matches();
    }

    /** {@code <gamedir>/deeptime/staging/<id>}. */
    public static Path dir(Path gameDir, String id) {
        if (!isId(id))
            throw new IllegalArgumentException("not a staging id: '" + id + "'");
        return root(gameDir).resolve(id);
    }

    /** True when {@code dir} holds complete world data (its manifest is written last). */
    public static boolean complete(Path dir) {
        return Files.isRegularFile(dir.resolve(Manifest.FILE));
    }

    /**
     * Removes every staging folder under {@code gameDir} that is not in {@code keep} and in which
     * nothing changed for {@code maxAge}; returns the folders removed. Errors on one folder are
     * skipped (another instance may hold a file open on Windows); the next cleanup tries again.
     */
    public static List<Path> cleanupAbandoned(Path gameDir, Duration maxAge, Set<String> keep, Clock clock) {
        Path root = root(gameDir);
        List<Path> removed = new ArrayList<>();
        if (!Files.isDirectory(root))
            return removed;
        long cutoff = clock.millis() - maxAge.toMillis();
        try (DirectoryStream<Path> ds = Files.newDirectoryStream(root)) {
            for (Path p : ds) {
                String name = p.getFileName().toString();
                if (!Files.isDirectory(p) || keep.contains(name))
                    continue;
                try {
                    if (lastChange(p) >= cutoff)
                        continue;
                    deleteTree(p);
                    removed.add(p);
                } catch (IOException e) {
                    // skipped: tried again next time
                }
            }
        } catch (IOException e) {
            // the root vanished or is unreadable: nothing to clean
        }
        return removed;
    }

    /** The newest modification time (ms) of the folder, its manifest and its keyframe index. */
    static long lastChange(Path dir) throws IOException {
        long t = Files.getLastModifiedTime(dir).toMillis();
        for (String f : new String[]{Manifest.FILE, "keyframes/index.json", "keyframes"}) {
            try {
                FileTime ft = Files.getLastModifiedTime(dir.resolve(f));
                t = Math.max(t, ft.toMillis());
            } catch (NoSuchFileException e) {
                // not there (yet)
            }
        }
        return t;
    }

    /** Deletes a folder and everything in it; a missing folder is fine. */
    public static void deleteTree(Path root) throws IOException {
        if (!Files.exists(root))
            return;
        Files.walkFileTree(root, new SimpleFileVisitor<>() {
            @Override
            public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) throws IOException {
                Files.delete(file);
                return FileVisitResult.CONTINUE;
            }

            @Override
            public FileVisitResult postVisitDirectory(Path d, IOException exc) throws IOException {
                if (exc != null)
                    throw exc;
                Files.delete(d);
                return FileVisitResult.CONTINUE;
            }
        });
    }
}
