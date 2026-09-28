// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.Optional;

/**
 * Reads the ELF {@code PT_INTERP} (the dynamic loader path) of an executable. For
 * {@code /proc/self/exe} that is the JVM's own loader, which tells glibc ({@code ld-linux-*}) from
 * musl ({@code ld-musl-*}) without running anything (plan §6.5 step 1). Off Linux, or when anything
 * is unexpected, the result is empty and the caller falls back to trying both builds.
 */
public final class ElfInterp {

    private static final int PT_INTERP = 3;
    private static final int MAX_PHNUM = 4096;
    private static final int MAX_INTERP = 4096;

    private ElfInterp() {}

    /** The JVM's own loader: {@code PT_INTERP} of {@code /proc/self/exe}; empty off Linux. */
    public static Optional<String> ofSelf() {
        Path self = Path.of("/proc/self/exe");
        if (!Files.exists(self))
            return Optional.empty();
        return read(self);
    }

    /** {@code PT_INTERP} of {@code file}; empty if it is not an ELF file or has no interpreter. */
    public static Optional<String> read(Path file) {
        try (FileChannel ch = FileChannel.open(file, StandardOpenOption.READ)) {
            ByteBuffer ident = readAt(ch, 0, 64);
            if (ident.limit() < 52 || ident.get(0) != 0x7f || ident.get(1) != 'E' || ident.get(2) != 'L'
                    || ident.get(3) != 'F')
                return Optional.empty();
            boolean is64 = ident.get(4) == 2;
            ident.order(ident.get(5) == 2 ? ByteOrder.BIG_ENDIAN : ByteOrder.LITTLE_ENDIAN);
            long phoff = is64 ? ident.getLong(0x20) : Integer.toUnsignedLong(ident.getInt(0x1C));
            int phentsize = Short.toUnsignedInt(ident.getShort(is64 ? 0x36 : 0x2A));
            int phnum = Short.toUnsignedInt(ident.getShort(is64 ? 0x38 : 0x2C));
            if (phnum > MAX_PHNUM || phentsize < (is64 ? 56 : 32))
                return Optional.empty();
            for (int i = 0; i < phnum; i++) {
                ByteBuffer ph = readAt(ch, phoff + (long) i * phentsize, phentsize).order(ident.order());
                if (ph.limit() < phentsize)
                    return Optional.empty();
                if (ph.getInt(0) != PT_INTERP)
                    continue;
                long offset = is64 ? ph.getLong(8) : Integer.toUnsignedLong(ph.getInt(4));
                long size = is64 ? ph.getLong(32) : Integer.toUnsignedLong(ph.getInt(16));
                if (size <= 0 || size > MAX_INTERP)
                    return Optional.empty();
                ByteBuffer s = readAt(ch, offset, (int) size);
                byte[] bytes = new byte[s.limit()];
                s.get(bytes);
                int len = 0;
                while (len < bytes.length && bytes[len] != 0)
                    len++;
                return Optional.of(new String(bytes, 0, len, StandardCharsets.UTF_8));
            }
            return Optional.empty();
        } catch (IOException | RuntimeException e) {
            return Optional.empty();
        }
    }

    private static ByteBuffer readAt(FileChannel ch, long position, int length) throws IOException {
        ByteBuffer buf = ByteBuffer.allocate(length);
        while (buf.hasRemaining()) {
            int n = ch.read(buf, position + buf.position());
            if (n < 0)
                break;
        }
        return buf.flip();
    }
}
