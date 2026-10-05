// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldDataException;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.channels.FileChannel;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.StandardCopyOption;
import java.nio.file.StandardOpenOption;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.zip.CRC32;

/**
 * Moves a staged world-data folder into its world (plan §7.12): {@code <gamedir>/deeptime/staging/<id>/}
 * → {@code <world>/deeptime/}, at {@code ServerAboutToStartEvent}, before {@code loadLevel()}
 * ({@link WorldDataBinder} calls it before binding). Plain file operations, no Minecraft, so JUnit
 * tests every path.
 *
 * <ol>
 *   <li>The target already holds world data (a restart, or a folder copied in): nothing to do;
 *       the staging folder, if still there, is left for {@link Staging#cleanupAbandoned}.</li>
 *   <li>An atomic rename when staging and world share a filesystem.</li>
 *   <li>Otherwise copy into {@code <target>.staging-copy}, comparing every file's CRC32 with the
 *       source's, check the copy against its manifest (every listed file's size and CRC32), rename it into
 *       place (same directory, so atomic) and only then delete the staging folder. Any failure
 *       removes the partial copy and leaves the staging folder as it was.</li>
 * </ol>
 */
public final class StagingMover {

    /** The partial copy's suffix, next to the target. */
    static final String COPY_SUFFIX = ".staging-copy";
    /** The epoch checkpoints' folder of a run (Rust `epochs::EPOCHS_DIR`). */
    static final String EPOCHS_DIR = "epochs";

    private StagingMover() {}

    /** How the data got into place. */
    public enum Kind { RENAMED, COPIED, ALREADY_PRESENT }

    public record Result(Kind kind, Path from, Path to, int files, long bytes) {}

    /** Why staged data could not be moved; the message names both folders. */
    public static final class StagingException extends Exception {
        private static final long serialVersionUID = 1L;

        public StagingException(String message) {
            super(message);
        }

        public StagingException(String message, Throwable cause) {
            super(message, cause);
        }
    }

    /**
     * Moves {@code staging} to {@code target}.
     *
     * @param forceCopy skip the rename (tests, and filesystems where a rename is known to lie)
     */
    public static Result move(Path staging, Path target, boolean forceCopy) throws StagingException {
        if (Files.isRegularFile(target.resolve(Manifest.FILE)))
            return new Result(Kind.ALREADY_PRESENT, staging, target, 0, 0);
        if (!Staging.complete(staging))
            throw new StagingException("no complete staged world data at " + staging + " (" + Manifest.FILE
                    + " missing): the Create World screen's run was not finished or its folder was removed");
        try {
            if (Files.exists(target))
                Staging.deleteTree(target); // a partial folder without a manifest: never valid data
            Files.createDirectories(target.toAbsolutePath().getParent());
            // The epoch checkpoints (pick the epoch, stop, continue; docs/design/
            // playability-steering.md) end with the editor: a world never needs them, and they are
            // the largest files of a staging folder.
            Staging.deleteTree(staging.resolve(EPOCHS_DIR));
        } catch (IOException e) {
            throw new StagingException("cannot prepare " + target + ": " + e, e);
        }
        if (!forceCopy) {
            try {
                Files.move(staging, target, StandardCopyOption.ATOMIC_MOVE);
                return new Result(Kind.RENAMED, staging, target, -1, -1);
            } catch (AtomicMoveNotSupportedException e) {
                // another filesystem: copy
            } catch (IOException e) {
                // e.g. Windows refusing a directory rename across volumes: copy
            }
        }
        return copy(staging, target);
    }

    private static Result copy(Path staging, Path target) throws StagingException {
        Path tmp = target.resolveSibling(target.getFileName() + COPY_SUFFIX);
        int[] files = {0};
        long[] bytes = {0};
        try {
            Staging.deleteTree(tmp);
            Files.walkFileTree(staging, new SimpleFileVisitor<>() {
                @Override
                public FileVisitResult preVisitDirectory(Path dir, BasicFileAttributes attrs) throws IOException {
                    Files.createDirectories(tmp.resolve(staging.relativize(dir).toString()));
                    return FileVisitResult.CONTINUE;
                }

                @Override
                public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) throws IOException {
                    Path dst = tmp.resolve(staging.relativize(file).toString());
                    long written = copyFile(file, dst);
                    long a = crc32(file), b = crc32(dst);
                    if (a != b || written != Files.size(file))
                        throw new IOException(String.format("copy of %s differs from the source (CRC32 %08x, source %08x)", file, b, a));
                    files[0]++;
                    bytes[0] += written;
                    return FileVisitResult.CONTINUE;
                }
            });
            verifyManifest(tmp);
            Files.move(tmp, target, StandardCopyOption.ATOMIC_MOVE);
        } catch (IOException | WorldDataException e) {
            try {
                Staging.deleteTree(tmp);
            } catch (IOException ignored) {
                // best effort; the next start deletes a partial copy too
            }
            throw new StagingException("cannot copy the staged world data " + staging + " into " + target + ": "
                    + e.getMessage() + " (the staged data is untouched)", e);
        }
        try {
            Staging.deleteTree(staging);
        } catch (IOException e) {
            // the world is complete; an undeletable staging folder is cleaned up later
        }
        return new Result(Kind.COPIED, staging, target, files[0], bytes[0]);
    }

    private static long copyFile(Path src, Path dst) throws IOException {
        long n;
        try (InputStream in = Files.newInputStream(src);
             OutputStream out = Files.newOutputStream(dst, StandardOpenOption.CREATE_NEW, StandardOpenOption.WRITE)) {
            n = in.transferTo(out);
        }
        try (FileChannel ch = FileChannel.open(dst, StandardOpenOption.WRITE)) {
            ch.force(true);
        }
        return n;
    }

    /**
     * Every file the manifest lists has its size and CRC32 ({@link WorldData#verify}'s check, with
     * plain reads: a memory-mapped tile pack would keep Windows from renaming the folder).
     */
    static void verifyManifest(Path folder) throws IOException {
        Manifest m = Manifest.read(folder);
        for (Manifest.LayerEntry l : m.layers())
            check(folder.resolve(l.path()), l.bytes(), l.crc32());
        for (Manifest.FileEntry f : m.files())
            check(folder.resolve(f.path()), f.bytes(), f.crc32());
    }

    private static void check(Path file, long bytes, long crc) throws IOException {
        long size = Files.size(file);
        long actual = crc32(file);
        if (size != bytes || actual != crc)
            throw new IOException(String.format("%s: %d bytes, CRC32 %08x; the manifest says %d bytes, %08x", file, size, actual,
                    bytes, crc));
    }

    static long crc32(Path file) throws IOException {
        CRC32 crc = new CRC32();
        try (InputStream in = Files.newInputStream(file)) {
            byte[] buf = new byte[1 << 16];
            int n;
            while ((n = in.read(buf)) > 0)
                crc.update(buf, 0, n);
        }
        return crc.getValue();
    }
}
