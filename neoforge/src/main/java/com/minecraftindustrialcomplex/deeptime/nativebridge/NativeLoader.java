// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.lang.System.Logger.Level;
import java.nio.channels.Channels;
import java.nio.channels.FileChannel;
import java.nio.channels.FileLock;
import java.nio.charset.StandardCharsets;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.DirectoryStream;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.NoSuchFileException;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.StandardCopyOption;
import java.nio.file.StandardOpenOption;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.PosixFileAttributeView;
import java.nio.file.attribute.PosixFilePermission;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.HexFormat;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Finds, extracts, verifies and links the native library (plan §6.5), and classifies every failure
 * into one of the five {@link FailureClass}es (plan §6.6).
 *
 * <p><b>JDK-only.</b> No Minecraft, NeoForge, Gson, SLF4J or Guava types: {@code NativeSmoke} runs
 * this class on a bare JVM with only the mod jar on the classpath. Logging goes through a
 * {@link NativeLogSink}; {@code natives.json} is read by {@link MiniJson}.
 *
 * <p>{@link #extract} and {@link #link} are separate so tests can extract many times in one JVM
 * while the library is linked at most once per JVM; {@link #load} does both, once, and never throws.
 *
 * <p>Forcing properties for tests (plan §6.6): {@code -Ddeeptime.natives.forceFail=unsupported},
 * {@code =libc_mismatch}, {@code =io}, {@code =version_mismatch}, and
 * {@code -Ddeeptime.natives.testDeleteAfterWrite=true}. {@code -Ddeeptime.natives.path=<file>}
 * loads a user-supplied library and skips extraction.
 */
public final class NativeLoader {

    public static final String PROP_PATH = "deeptime.natives.path";
    public static final String PROP_FORCE_FAIL = "deeptime.natives.forceFail";
    public static final String PROP_DELETE_AFTER_WRITE = "deeptime.natives.testDeleteAfterWrite";

    /** Folder of the natives inside the jar. */
    public static final String RESOURCE_ROOT = "natives/deeptime/";
    /** The manifest inside the jar. */
    public static final String MANIFEST_RESOURCE = RESOURCE_ROOT + "natives.json";

    static final long SELF_TEST_TERMS = 1L << 20;
    static final long SELF_TEST_MULTIPLIER = 2_654_435_761L;

    private static volatile NativeLogSink sink = NativeLogSink.systemLogger();
    private static volatile NativeStatus status;
    /** The library this JVM linked; guarded by {@code NativeLoader.class}. */
    private static Path linkedPath;
    /** In-JVM locks per extraction directory: {@link FileChannel#lock()} only excludes other processes. */
    private static final ConcurrentHashMap<Path, Object> DIR_LOCKS = new ConcurrentHashMap<>();

    private NativeLoader() {}

    // ------------------------------------------------------------------------------------------
    // Public API
    // ------------------------------------------------------------------------------------------

    /** Routes the loader's messages and the native log lines; the mod installs an SLF4J sink. */
    public static void setLogSink(NativeLogSink newSink) {
        sink = newSink == null ? NativeLogSink.systemLogger() : newSink;
    }

    /**
     * Extracts, links and verifies the library for {@code gameDir}, once per JVM: later calls
     * return the first result. Logs the outcome once (INFO when OK, ERROR with the guidance block
     * otherwise) and never throws.
     */
    public static synchronized NativeStatus load(Path gameDir) {
        if (status != null)
            return status;
        NativeStatus result;
        try {
            ExtractResult extracted = extract(gameDir);
            result = link(extracted);
            if (result.isOk() && !extracted.isOverride())
                prune(gameDir, extracted);
        } catch (Throwable t) {
            result = NativeStatus.failure(FailureClass.IO_ERROR, safeKey(), null, t);
        }
        status = result;
        try {
            if (result.isOk())
                log(NativeLogSink.LOADER, Level.INFO, result.describe(), null);
            else
                log(NativeLogSink.LOADER, Level.ERROR, result.describe(), result.cause());
        } catch (Throwable ignored) {
            // A broken sink must not make load() throw.
        }
        return result;
    }

    /** The result of {@link #load}, if it ran. */
    public static Optional<NativeStatus> status() {
        return Optional.ofNullable(status);
    }

    /** True once a library is linked into this JVM. */
    public static synchronized boolean isLinked() {
        return linkedPath != null;
    }

    /** Runs the native self-test again (for GameTests and diagnostics). */
    public static long selfTest() {
        requireLinked();
        return DeepTimeNative.selfTest();
    }

    /** The linked library's version string, {@code <semver>+<sha>;format=<f>;target=<triple>}. */
    public static String nativeVersionString() {
        requireLinked();
        return DeepTimeNative.version();
    }

    /** The self-test sum, computed in Java: Σ (i × 2654435761) mod 2^64 for i &lt; 2^20. */
    public static long expectedSelfTest() {
        long sum = 0;
        for (long i = 0; i < SELF_TEST_TERMS; i++)
            sum += i * SELF_TEST_MULTIPLIER; // long arithmetic wraps: mod 2^64
        return sum;
    }

    /**
     * Steps 1–5 of plan §6.5: platform key, override, target directory, lock, verify or write,
     * re-verify. Never throws; a failure is returned inside the result.
     */
    public static ExtractResult extract(Path gameDir) {
        String key = safeKey();
        try {
            PlatformKey platform = PlatformKey.detect();
            key = platform.key();
            String force = forceFail();

            NativesManifest manifest;
            try {
                manifest = readManifest();
            } catch (NoSuchFileException e) {
                return fail(FailureClass.UNSUPPORTED_PLATFORM, key, null, e);
            } catch (IllegalArgumentException e) {
                return fail(FailureClass.VERSION_MISMATCH, key, null, e);
            }

            String override = System.getProperty(PROP_PATH, "").trim();
            if (!override.isEmpty()) {
                Path p = Path.of(override).toAbsolutePath();
                if (!Files.isRegularFile(p))
                    return fail(FailureClass.IO_ERROR, key, p,
                            new NoSuchFileException(p.toString(), null, "-D" + PROP_PATH + " names no file"));
                return ExtractResult.ok(key, List.of(new ExtractResult.Extracted(key, p, null, null, false)),
                        manifest.version(), manifest.formatVersion(), true);
            }

            if (force.equals("unsupported"))
                return fail(FailureClass.UNSUPPORTED_PLATFORM, key, null,
                        new UnsupportedOperationException("forced by -D" + PROP_FORCE_FAIL + "=unsupported"));

            List<NativesManifest.Library> libs = new ArrayList<>();
            for (String k : platform.candidateKeys())
                manifest.library(k).ifPresent(libs::add);
            if (libs.isEmpty()) {
                List<String> have = manifest.libraries().stream().map(NativesManifest.Library::platform).toList();
                return fail(FailureClass.UNSUPPORTED_PLATFORM, key, null, new UnsupportedOperationException(
                        "natives.json has no library for " + platform.candidateKeys() + " (it has " + have + ")"));
            }

            Path root = nativesRoot(gameDir);
            List<ExtractResult.Extracted> out = new ArrayList<>();
            for (int i = 0; i < libs.size(); i++) {
                NativesManifest.Library lib = libs.get(i);
                Path dir = root.resolve(manifest.version() + "-" + lib.sha256().substring(0, 12));
                try {
                    out.add(extractOne(dir, lib, force));
                } catch (VanishedException e) {
                    if (i == 0)
                        return fail(FailureClass.ANTIVIRUS_SUSPECTED, key, e.file, e);
                    log(NativeLogSink.LOADER, Level.WARNING, "fallback build " + lib.platform() + ": " + e, null);
                } catch (IOException | RuntimeException e) {
                    if (i == 0)
                        return fail(FailureClass.IO_ERROR, key, dir.resolve(lib.file()), e);
                    log(NativeLogSink.LOADER, Level.WARNING, "fallback build " + lib.platform() + ": " + e, null);
                }
            }
            return ExtractResult.ok(key, out, manifest.version(), manifest.formatVersion(), false);
        } catch (Throwable t) {
            return fail(FailureClass.IO_ERROR, key, null, t);
        }
    }

    /**
     * Step 6 of plan §6.5: re-checks each extracted file, {@code System.load}s the first that links
     * (trying the other libc build after a libc error), then checks {@code version()} (semver and
     * format only) and {@code selfTest()}. The library is linked at most once per JVM; a second call
     * verifies against the library already linked. Never throws.
     */
    public static NativeStatus link(ExtractResult extracted) {
        String key = extracted.platformKey();
        if (!extracted.isOk())
            return extracted.failure().orElseThrow();
        try {
            String force = forceFail();
            UnsatisfiedLinkError libcError = null;
            Path last = null;
            for (ExtractResult.Extracted c : extracted.candidates()) {
                last = c.file();
                if (c.sha256() != null) {
                    String now = Files.isRegularFile(c.file()) ? sha256(c.file()) : null;
                    if (!c.sha256().equals(now))
                        return NativeStatus.failure(FailureClass.ANTIVIRUS_SUSPECTED, key, c.file(),
                                new IOException(now == null ? c.file() + " vanished after extraction"
                                        : c.file() + " changed after extraction (sha256 now " + now + ")"));
                }
                try {
                    systemLoad(c.file(), force);
                } catch (UnsatisfiedLinkError e) {
                    if (isLibcError(e)) {
                        libcError = e;
                        log(NativeLogSink.LOADER, Level.DEBUG, "libc error linking " + c.file() + ": " + e.getMessage(),
                                null);
                        continue;
                    }
                    // Not a libc problem and the file is intact: this library cannot run here.
                    return NativeStatus.failure(FailureClass.UNSUPPORTED_PLATFORM, key, c.file(), e);
                }
                return verify(extracted, c, force);
            }
            return NativeStatus.failure(FailureClass.LIBC_MISMATCH, key, last, libcError);
        } catch (IOException e) {
            return NativeStatus.failure(FailureClass.IO_ERROR, key, extracted.directory(), e);
        } catch (Throwable t) {
            return NativeStatus.failure(FailureClass.IO_ERROR, key, extracted.directory(), t);
        }
    }

    // ------------------------------------------------------------------------------------------
    // Extraction
    // ------------------------------------------------------------------------------------------

    static Path nativesRoot(Path gameDir) {
        return gameDir.toAbsolutePath().normalize().resolve("deeptime").resolve("natives");
    }

    /** Thrown when an extracted file vanishes or changes right after being written. */
    private static final class VanishedException extends IOException {
        private static final long serialVersionUID = 1L;
        final transient Path file;

        VanishedException(Path file, String message) {
            super(message);
            this.file = file;
        }
    }

    private static ExtractResult.Extracted extractOne(Path dir, NativesManifest.Library lib, String force)
            throws IOException {
        Files.createDirectories(dir); // parents first (Sable's recurring macOS failure, plan §6.5 step 4)
        boolean forceIo = force.equals("io");
        if (forceIo)
            setWritable(dir, false);
        try {
            Object jvmLock = DIR_LOCKS.computeIfAbsent(dir, d -> new Object());
            synchronized (jvmLock) {
                try (FileChannel lockChannel = FileChannel.open(dir.resolve(".lock"), StandardOpenOption.CREATE,
                        StandardOpenOption.WRITE); FileLock ignored = lockChannel.lock()) {
                    Path file = dir.resolve(lib.file());
                    if (!forceIo && Files.isRegularFile(file) && lib.sha256().equals(sha256(file)))
                        return new ExtractResult.Extracted(lib.platform(), file, lib.sha256(), lib.rustTarget(), false);

                    Path tmp = dir.resolve(lib.file() + ".tmp");
                    try (InputStream in = openResource(RESOURCE_ROOT + lib.resource())) {
                        if (in == null)
                            throw new NoSuchFileException(RESOURCE_ROOT + lib.resource(), null, "not in the mod jar");
                        try (FileChannel ch = FileChannel.open(tmp, StandardOpenOption.CREATE,
                                StandardOpenOption.WRITE, StandardOpenOption.TRUNCATE_EXISTING)) {
                            OutputStream os = Channels.newOutputStream(ch);
                            in.transferTo(os);
                            os.flush();
                            ch.force(true); // fsync before the rename (plan §6.5 step 5)
                        }
                    }
                    if (forceIo)
                        throw new IOException("forced by -D" + PROP_FORCE_FAIL
                                + "=io, but the read-only directory was still writable (running as root?)");
                    String got = sha256(tmp);
                    if (!got.equals(lib.sha256()) || Files.size(tmp) != lib.size()) {
                        Files.deleteIfExists(tmp);
                        throw new IOException("the jar's " + lib.resource() + " has sha256 " + got
                                + ", natives.json says " + lib.sha256());
                    }
                    try {
                        Files.move(tmp, file, StandardCopyOption.ATOMIC_MOVE);
                    } catch (AtomicMoveNotSupportedException e) {
                        Files.move(tmp, file, StandardCopyOption.REPLACE_EXISTING);
                    }
                    if (Boolean.getBoolean(PROP_DELETE_AFTER_WRITE))
                        Files.deleteIfExists(file);
                    if (!Files.isRegularFile(file))
                        throw new VanishedException(file, file + " vanished right after it was written");
                    String again = sha256(file);
                    if (!again.equals(lib.sha256()))
                        throw new VanishedException(file, file + " changed right after it was written (sha256 " + again + ")");
                    return new ExtractResult.Extracted(lib.platform(), file, lib.sha256(), lib.rustTarget(), true);
                }
            }
        } finally {
            if (forceIo)
                setWritable(dir, true);
        }
    }

    /** Step 7: removes other versions' directories, best effort (a locked file is skipped). */
    private static void prune(Path gameDir, ExtractResult extracted) {
        Path root = nativesRoot(gameDir);
        Set<Path> keep = new HashSet<>();
        for (ExtractResult.Extracted c : extracted.candidates())
            keep.add(c.file().getParent().toAbsolutePath().normalize());
        try (DirectoryStream<Path> dirs = Files.newDirectoryStream(root, Files::isDirectory)) {
            for (Path d : dirs) {
                if (keep.contains(d.toAbsolutePath().normalize()))
                    continue;
                deleteTree(d);
            }
        } catch (IOException | RuntimeException e) {
            log(NativeLogSink.LOADER, Level.DEBUG, "pruning " + root + ": " + e, null);
        }
    }

    private static void deleteTree(Path dir) {
        try {
            Files.walkFileTree(dir, new SimpleFileVisitor<>() {
                @Override
                public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) {
                    try {
                        Files.delete(file);
                    } catch (IOException ignored) {
                        // locked by another running instance: leave it
                    }
                    return FileVisitResult.CONTINUE;
                }

                @Override
                public FileVisitResult postVisitDirectory(Path d, IOException exc) {
                    try {
                        Files.delete(d);
                    } catch (IOException ignored) {
                        // not empty: something inside was locked
                    }
                    return FileVisitResult.CONTINUE;
                }
            });
        } catch (IOException ignored) {
            // best effort
        }
    }

    private static void setWritable(Path dir, boolean writable) {
        try {
            PosixFileAttributeView view = Files.getFileAttributeView(dir, PosixFileAttributeView.class);
            if (view != null) {
                Set<PosixFilePermission> perms = new HashSet<>(view.readAttributes().permissions());
                if (writable) {
                    perms.add(PosixFilePermission.OWNER_WRITE);
                } else {
                    perms.remove(PosixFilePermission.OWNER_WRITE);
                    perms.remove(PosixFilePermission.GROUP_WRITE);
                    perms.remove(PosixFilePermission.OTHERS_WRITE);
                }
                view.setPermissions(perms);
            } else {
                dir.toFile().setWritable(writable);
            }
        } catch (IOException | RuntimeException ignored) {
            // forcing only; the forced write fails either way
        }
    }

    static NativesManifest readManifest() throws IOException {
        try (InputStream in = openResource(MANIFEST_RESOURCE)) {
            if (in == null)
                throw new NoSuchFileException(MANIFEST_RESOURCE, null, "not in the mod jar (a build without natives)");
            return NativesManifest.parse(new String(in.readAllBytes(), StandardCharsets.UTF_8));
        }
    }

    /**
     * Opens a resource of the mod jar. {@code Class.getResourceAsStream} first: inside NeoForge the
     * mod is a named module and the caller is in it, so encapsulation does not apply; then the class
     * loader and the context class loader for unusual launchers.
     */
    private static InputStream openResource(String name) {
        InputStream in = NativeLoader.class.getResourceAsStream("/" + name);
        if (in == null && NativeLoader.class.getClassLoader() != null)
            in = NativeLoader.class.getClassLoader().getResourceAsStream(name);
        ClassLoader ctx = Thread.currentThread().getContextClassLoader();
        if (in == null && ctx != null)
            in = ctx.getResourceAsStream(name);
        return in;
    }

    static String sha256(Path file) throws IOException {
        MessageDigest md;
        try {
            md = MessageDigest.getInstance("SHA-256");
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException("SHA-256 is mandatory in every JDK", e);
        }
        try (InputStream in = Files.newInputStream(file)) {
            byte[] buf = new byte[1 << 16];
            int n;
            while ((n = in.read(buf)) > 0)
                md.update(buf, 0, n);
        }
        return HexFormat.of().formatHex(md.digest());
    }

    // ------------------------------------------------------------------------------------------
    // Linking
    // ------------------------------------------------------------------------------------------

    private static void systemLoad(Path file, String force) {
        if (force.equals("libc_mismatch"))
            throw new UnsatisfiedLinkError(file + ": /lib64/libc.so.6: version `GLIBC_2.99' not found (required by "
                    + file + ") [forced by -D" + PROP_FORCE_FAIL + "=libc_mismatch]");
        synchronized (NativeLoader.class) {
            if (linkedPath != null) {
                if (!linkedPath.equals(file))
                    log(NativeLogSink.LOADER, Level.DEBUG,
                            "a library is already linked from " + linkedPath + "; verifying it instead of " + file, null);
                return;
            }
            System.load(file.toAbsolutePath().toString());
            linkedPath = file;
        }
    }

    static boolean isLibcError(UnsatisfiedLinkError e) {
        String m = String.valueOf(e.getMessage());
        return m.contains("GLIBC_") || m.contains("ld-linux") || m.contains("ld-musl") || m.contains("libc.musl");
    }

    /** Parsed {@code <semver>+<sha>;format=<f>;target=<triple>}. */
    record VersionInfo(String semver, String gitSha, String format, String target) {
        static Optional<VersionInfo> parse(String v) {
            if (v == null)
                return Optional.empty();
            String[] parts = v.split(";");
            String semver = parts[0];
            String sha = "";
            int plus = semver.indexOf('+');
            if (plus >= 0) {
                sha = semver.substring(plus + 1);
                semver = semver.substring(0, plus);
            }
            String format = null;
            String target = null;
            for (int i = 1; i < parts.length; i++) {
                if (parts[i].startsWith("format="))
                    format = parts[i].substring("format=".length());
                else if (parts[i].startsWith("target="))
                    target = parts[i].substring("target=".length());
            }
            if (semver.isEmpty() || format == null)
                return Optional.empty();
            return Optional.of(new VersionInfo(semver, sha, format, target));
        }
    }

    private static NativeStatus verify(ExtractResult extracted, ExtractResult.Extracted c, String force) {
        String key = c.platform();
        String versionString;
        long sum;
        try {
            versionString = DeepTimeNative.version();
            sum = DeepTimeNative.selfTest();
        } catch (LinkageError | RuntimeException e) {
            return NativeStatus.failure(FailureClass.VERSION_MISMATCH, key, c.file(), e);
        }
        Optional<VersionInfo> parsed = VersionInfo.parse(versionString);
        if (parsed.isEmpty())
            return NativeStatus.failure(FailureClass.VERSION_MISMATCH, key, c.file(),
                    new IllegalStateException("unparseable version() string: " + versionString));
        VersionInfo info = parsed.get();
        String expectVersion = extracted.expectedVersion();
        if (force.equals("version_mismatch"))
            expectVersion = "0.0.0-forced-mismatch";
        if (!info.semver().equals(expectVersion) || !info.format().equals(extracted.expectedFormat()))
            return NativeStatus.failure(FailureClass.VERSION_MISMATCH, key, c.file(), new IllegalStateException(
                    "the library reports " + info.semver() + ";format=" + info.format() + ", natives.json expects "
                            + expectVersion + ";format=" + extracted.expectedFormat()
                            + (force.equals("version_mismatch") ? " [forced by -D" + PROP_FORCE_FAIL
                                    + "=version_mismatch]" : "")));
        long expected = expectedSelfTest();
        if (sum != expected)
            return NativeStatus.failure(FailureClass.VERSION_MISMATCH, key, c.file(), new IllegalStateException(
                    "selfTest() returned " + Long.toUnsignedString(sum) + ", Java computed "
                            + Long.toUnsignedString(expected)));
        return NativeStatus.ok(key, c.file(), info.semver(), info.format(), info.target(), info.gitSha(), sum);
    }

    private static synchronized void requireLinked() {
        if (linkedPath == null)
            throw new IllegalStateException("the Deep Time native library is not linked"
                    + (status != null && !status.isOk() ? ": " + status.summary() : ""));
    }

    // ------------------------------------------------------------------------------------------
    // Logging and helpers
    // ------------------------------------------------------------------------------------------

    static void log(String logger, Level level, String message, Throwable thrown) {
        try {
            sink.log(logger, level, message, thrown);
        } catch (Throwable ignored) {
            // logging must never break loading
        }
    }

    /** Forwards drained native log lines ({@code LEVEL<TAB>message}, one per line) to the sink. */
    static void forwardNativeLog(String lines) {
        if (lines == null || lines.isEmpty())
            return;
        for (String line : lines.split("\n")) {
            if (line.isEmpty())
                continue;
            int tab = line.indexOf('\t');
            String lvl = tab < 0 ? "INFO" : line.substring(0, tab);
            String msg = tab < 0 ? line : line.substring(tab + 1);
            Level level = switch (lvl) {
                case "ERROR" -> Level.ERROR;
                case "WARN" -> Level.WARNING;
                case "DEBUG" -> Level.DEBUG;
                default -> Level.INFO;
            };
            log(NativeLogSink.NATIVE, level, msg, null);
        }
    }

    private static String forceFail() {
        return System.getProperty(PROP_FORCE_FAIL, "").trim().toLowerCase(Locale.ROOT);
    }

    private static String safeKey() {
        try {
            return PlatformKey.detect().key();
        } catch (Throwable t) {
            return "unknown";
        }
    }

    private static ExtractResult fail(FailureClass fc, String key, Path path, Throwable cause) {
        return ExtractResult.failed(NativeStatus.failure(fc, key, path, cause));
    }
}
