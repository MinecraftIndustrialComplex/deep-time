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
 * <p><b>Two builds per x86-64 platform</b> (decisions log 2026-10-18; plan §6.5): the baseline
 * build and an {@code x86-64-v3} build. Before extracting, the loader asks the CPU probe
 * ({@link CpuProbe}, a separate tiny library, because the JVM binds {@link DeepTimeNative}'s methods
 * to the first library loaded that exports them) whether the CPU has the whole x86-64-v3 level, and
 * puts the v3 build first when it does. A v3 build that cannot be extracted or linked falls back to
 * the baseline build with a warning. {@code -Ddeeptime.natives.variant=baseline|x86-64-v3|auto}, or
 * the environment variable {@code DEEPTIME_NATIVES_VARIANT} (the property wins), forces the choice
 * for tests and bug reports; a forced v3 on a CPU the probe says lacks the level is refused, since
 * the library would die on its first illegal instruction.
 *
 * <p>Forcing properties for tests (plan §6.6): {@code -Ddeeptime.natives.forceFail=unsupported},
 * {@code =libc_mismatch}, {@code =io}, {@code =version_mismatch}, {@code =v3_link} (the v3 build
 * fails to link, so the baseline must load), and {@code -Ddeeptime.natives.testDeleteAfterWrite=true}.
 * {@code -Ddeeptime.natives.path=<file>} loads a user-supplied library and skips extraction.
 */
public final class NativeLoader {

    public static final String PROP_PATH = "deeptime.natives.path";
    public static final String PROP_FORCE_FAIL = "deeptime.natives.forceFail";
    public static final String PROP_DELETE_AFTER_WRITE = "deeptime.natives.testDeleteAfterWrite";
    /** Forces the build: {@code baseline}, {@code x86-64-v3} (or {@code v3}), or {@code auto}. */
    public static final String PROP_VARIANT = "deeptime.natives.variant";
    /** The environment variable of {@link #PROP_VARIANT}; the property wins when both are set. */
    public static final String ENV_VARIANT = "DEEPTIME_NATIVES_VARIANT";
    /**
     * Tests only: replaces the CPU probe's feature mask (bit {@code i} = {@code CpuProbe.FEATURES[i]},
     * decimal or {@code 0x} hex), so a v3 machine can play one without the level. The probe still runs.
     */
    public static final String PROP_TEST_CPU_FEATURES = "deeptime.natives.testCpuFeatures";

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
    /** The CPU probe's answer, once per JVM (its library links once too); guarded by {@code NativeLoader.class}. */
    private static Probe probe;
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

    /**
     * The linked library's version string,
     * {@code <semver>+<sha>;format=<f>;target=<triple>;cpu=<baseline|x86-64-v3>}.
     */
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
     * Steps 1–6 of plan §6.5: platform key, override, target directory, the build for this CPU,
     * lock, verify or write, re-verify. Never throws; a failure is returned inside the result.
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
                return ExtractResult.ok(key, List.of(new ExtractResult.Extracted(key, null, p, null, null, false)),
                        manifest.version(), manifest.formatVersion(), true, null, "-D" + PROP_PATH + " names it",
                        null);
            }

            if (force.equals("unsupported"))
                return fail(FailureClass.UNSUPPORTED_PLATFORM, key, null,
                        new UnsupportedOperationException("forced by -D" + PROP_FORCE_FAIL + "=unsupported"));

            Path root = nativesRoot(gameDir);
            VariantChoice choice = chooseVariant(platform, manifest, root);
            // Per libc (detected first): the v3 build when chosen, then the baseline build.
            List<NativesManifest.Library> libs = new ArrayList<>();
            for (String k : platform.candidateKeys()) {
                if (choice.isV3())
                    manifest.library(k, NativesManifest.X86_64_V3).ifPresent(libs::add);
                manifest.library(k, NativesManifest.BASELINE).ifPresent(libs::add);
            }
            NativesManifest.Library primary = libs.stream().filter(l -> !l.isV3()).findFirst().orElse(null);
            if (primary == null) {
                List<String> have = manifest.libraries().stream().map(l -> l.platform() + "/" + l.variant()).toList();
                return fail(FailureClass.UNSUPPORTED_PLATFORM, key, null, new UnsupportedOperationException(
                        "natives.json has no library for " + platform.candidateKeys() + " (it has " + have + ")"));
            }

            List<ExtractResult.Extracted> out = new ArrayList<>();
            String v3Error = null;
            for (NativesManifest.Library lib : libs) {
                Path dir = libraryDir(root, manifest, lib);
                try {
                    out.add(extractOne(dir, lib, force, true));
                } catch (IOException | RuntimeException e) {
                    // The detected libc's baseline build must extract; any other is a fallback.
                    if (lib == primary)
                        return e instanceof VanishedException v
                                ? fail(FailureClass.ANTIVIRUS_SUSPECTED, key, v.file, e)
                                : fail(FailureClass.IO_ERROR, key, dir.resolve(lib.file()), e);
                    log(NativeLogSink.LOADER, Level.WARNING, "the " + lib.variant() + " build for " + lib.platform()
                            + " could not be extracted (" + e + "); trying the next build", null);
                    if (lib.isV3() && v3Error == null)
                        v3Error = String.valueOf(e.getMessage());
                }
            }
            boolean v3 = out.stream().anyMatch(ExtractResult.Extracted::isV3);
            String reason = choice.reason();
            if (choice.isV3() && !v3)
                reason += "; the x86-64-v3 build could not be extracted (" + v3Error + ")";
            return ExtractResult.ok(key, out, manifest.version(), manifest.formatVersion(), false,
                    v3 ? NativesManifest.X86_64_V3 : NativesManifest.BASELINE, reason, choice.probeFile());
        } catch (Throwable t) {
            return fail(FailureClass.IO_ERROR, key, null, t);
        }
    }

    /**
     * Step 7 of plan §6.5: re-checks each extracted file, {@code System.load}s the first that links
     * (the baseline build after an x86-64-v3 build that cannot load, the other libc's builds after a
     * libc error), then checks {@code version()} (semver and format only) and {@code selfTest()}.
     * The library is linked at most once per JVM; a second call verifies against the library already
     * linked. Never throws.
     */
    public static NativeStatus link(ExtractResult extracted) {
        String key = extracted.platformKey();
        if (!extracted.isOk())
            return extracted.failure().orElseThrow();
        try {
            String force = forceFail();
            UnsatisfiedLinkError libcError = null;
            Path last = null;
            String v3Failure = null;
            for (ExtractResult.Extracted c : extracted.candidates()) {
                last = c.file();
                if (c.sha256() != null) {
                    String now = Files.isRegularFile(c.file()) ? sha256(c.file()) : null;
                    if (!c.sha256().equals(now)) {
                        IOException e = new IOException(now == null ? c.file() + " vanished after extraction"
                                : c.file() + " changed after extraction (sha256 now " + now + ")");
                        if (c.isV3()) {
                            v3Failure = v3Failed(c, "changed on disk", e);
                            continue;
                        }
                        return NativeStatus.failure(FailureClass.ANTIVIRUS_SUSPECTED, key, c.file(), e);
                    }
                }
                try {
                    systemLoad(c, force);
                } catch (UnsatisfiedLinkError e) {
                    if (isLibcError(e)) {
                        libcError = e;
                        log(NativeLogSink.LOADER, Level.DEBUG, "libc error linking " + c.file() + ": " + e.getMessage(),
                                null);
                        continue;
                    }
                    if (c.isV3()) {
                        v3Failure = v3Failed(c, "failed to link", e);
                        continue;
                    }
                    // Not a libc problem and the file is intact: this library cannot run here.
                    return NativeStatus.failure(FailureClass.UNSUPPORTED_PLATFORM, key, c.file(), e);
                }
                return verify(extracted, c, force, v3Failure);
            }
            return NativeStatus.failure(FailureClass.LIBC_MISMATCH, key, last, libcError);
        } catch (IOException e) {
            return NativeStatus.failure(FailureClass.IO_ERROR, key, extracted.directory(), e);
        } catch (Throwable t) {
            return NativeStatus.failure(FailureClass.IO_ERROR, key, extracted.directory(), t);
        }
    }

    // ------------------------------------------------------------------------------------------
    // The build for this CPU (decisions log 2026-10-18)
    // ------------------------------------------------------------------------------------------

    /** A forced build ({@link #PROP_VARIANT} or {@link #ENV_VARIANT}) and where it was forced. */
    record VariantOverride(String variant, String source) {

        /**
         * The property when set (so {@code -Ddeeptime.natives.variant=auto} cancels the environment
         * variable), else the environment variable. {@code auto}, blank and unknown values force
         * nothing (an unknown one with a warning).
         */
        static Optional<VariantOverride> parse(String property, String env) {
            String raw;
            String source;
            if (property != null && !property.isBlank()) {
                raw = property;
                source = "-D" + PROP_VARIANT;
            } else if (env != null && !env.isBlank()) {
                raw = env;
                source = ENV_VARIANT;
            } else {
                return Optional.empty();
            }
            String v = raw.trim().toLowerCase(Locale.ROOT);
            String variant = switch (v) {
                case "baseline" -> NativesManifest.BASELINE;
                case "x86-64-v3", "v3" -> NativesManifest.X86_64_V3;
                case "auto" -> null;
                default -> {
                    log(NativeLogSink.LOADER, Level.WARNING, "ignoring " + source + "=" + raw
                            + " (it takes baseline, x86-64-v3 or auto)", null);
                    yield null;
                }
            };
            return variant == null ? Optional.empty() : Optional.of(new VariantOverride(variant, source + "=" + v));
        }
    }

    /**
     * The build {@link #extract} puts first and why.
     *
     * @param probeFile the CPU probe library, when it was extracted under this game directory (so
     *                  pruning keeps it)
     */
    record VariantChoice(String variant, String reason, Path probeFile) {
        boolean isV3() {
            return NativesManifest.X86_64_V3.equals(variant);
        }
    }

    /** The CPU probe's answer: the mask of x86-64-v3 features present, or why there is none. */
    record Probe(int features, Path file, String error) {}

    /**
     * Baseline when the platform has no x86-64-v3 build or the baseline is forced; otherwise what
     * the CPU probe says (a forced v3 included: it is refused on a CPU that lacks the level, and
     * taken on trust only when the probe cannot run).
     */
    static VariantChoice chooseVariant(PlatformKey platform, NativesManifest manifest, Path root) {
        Optional<VariantOverride> forced = VariantOverride.parse(System.getProperty(PROP_VARIANT),
                System.getenv(ENV_VARIANT));
        boolean hasV3 = platform.candidateKeys().stream()
                .anyMatch(k -> manifest.library(k, NativesManifest.X86_64_V3).isPresent());
        if (!hasV3)
            return new VariantChoice(NativesManifest.BASELINE, "natives.json has no x86-64-v3 build for "
                    + platform.key() + forced.map(f -> " (" + f.source() + " has nothing to choose)").orElse(""), null);
        if (forced.isPresent() && forced.get().variant().equals(NativesManifest.BASELINE))
            return new VariantChoice(NativesManifest.BASELINE, "forced by " + forced.get().source(), null);

        Probe p = runProbe(manifest, platform, root);
        Path probeFile = p.file() != null && p.file().startsWith(root) ? p.file() : null;
        String byForce = forced.map(f -> "forced by " + f.source() + "; ").orElse("");
        if (p.error() != null)
            return new VariantChoice(forced.isPresent() ? NativesManifest.X86_64_V3 : NativesManifest.BASELINE,
                    byForce + "the CPU probe could not run (" + p.error() + ")", probeFile);
        Integer testMask = Integer.getInteger(PROP_TEST_CPU_FEATURES);
        List<String> missing = CpuProbe.missing(testMask != null ? testMask : p.features());
        if (missing.isEmpty())
            return new VariantChoice(NativesManifest.X86_64_V3, byForce + "the CPU has every x86-64-v3 feature",
                    probeFile);
        String lacks = "the CPU lacks " + String.join(", ", missing) + " of x86-64-v3";
        if (forced.isPresent()) {
            log(NativeLogSink.LOADER, Level.WARNING, forced.get().source() + " refused: " + lacks
                    + "; the baseline build loads instead", null);
            return new VariantChoice(NativesManifest.BASELINE, forced.get().source() + " refused: " + lacks, probeFile);
        }
        return new VariantChoice(NativesManifest.BASELINE, lacks, probeFile);
    }

    /**
     * Extracts and links the CPU probe of the first candidate platform that has one (the other
     * libc's after a libc error) and asks it, once per JVM. Never throws.
     */
    private static synchronized Probe runProbe(NativesManifest manifest, PlatformKey platform, Path root) {
        if (probe != null)
            return probe;
        String error = "natives.json has no CPU probe for " + platform.candidateKeys();
        Path file = null;
        for (String k : platform.candidateKeys()) {
            Optional<NativesManifest.Library> lib = manifest.probe(k);
            if (lib.isEmpty())
                continue;
            try {
                file = extractOne(libraryDir(root, manifest, lib.get()), lib.get(), "", false).file();
                System.load(file.toAbsolutePath().toString());
            } catch (UnsatisfiedLinkError e) {
                error = file + ": " + e.getMessage();
                if (isLibcError(e))
                    continue;
                break;
            } catch (IOException | RuntimeException e) {
                error = "extracting " + lib.get().resource() + ": " + e;
                continue;
            }
            try {
                long v = CpuProbe.probe();
                int abi = (int) (v >>> 32);
                if (abi != CpuProbe.ABI) {
                    error = file + " answers in probe ABI " + abi + ", this loader reads " + CpuProbe.ABI;
                    break;
                }
                probe = new Probe((int) v, file, null);
                log(NativeLogSink.LOADER, Level.DEBUG, "CPU probe " + file + ": x86-64-v3 features 0x"
                        + Integer.toHexString((int) v) + " of 0x" + Integer.toHexString(CpuProbe.ALL), null);
                return probe;
            } catch (LinkageError | RuntimeException e) {
                // Linked but unusable; another probe cannot bind CpuProbe now.
                error = file + ": " + e;
                break;
            }
        }
        probe = new Probe(0, file, error);
        log(NativeLogSink.LOADER, Level.WARNING, "the CPU probe could not run: " + error, null);
        return probe;
    }

    /** Logs why an x86-64-v3 build cannot be used and returns that for the status's reason. */
    private static String v3Failed(ExtractResult.Extracted c, String what, Throwable e) {
        log(NativeLogSink.LOADER, Level.WARNING, "the x86-64-v3 build " + c.file() + " " + what + " (" + e.getMessage()
                + "); falling back to the baseline build", null);
        return "the x86-64-v3 build " + what + " (" + e.getMessage() + ")";
    }

    // ------------------------------------------------------------------------------------------
    // Extraction
    // ------------------------------------------------------------------------------------------

    static Path nativesRoot(Path gameDir) {
        return gameDir.toAbsolutePath().normalize().resolve("deeptime").resolve("natives");
    }

    /** {@code <root>/<version>-<first 12 hex of sha256>}: each build and probe gets its own. */
    private static Path libraryDir(Path root, NativesManifest manifest, NativesManifest.Library lib) {
        return root.resolve(manifest.version() + "-" + lib.sha256().substring(0, 12));
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

    /**
     * Extracts one library (or probe) into {@code dir}, or reuses a verified copy there.
     * {@code testHooks} applies {@code forceFail=io} and {@code testDeleteAfterWrite}; the CPU probe
     * is extracted without them, so a forced failure hits the library it is aimed at.
     */
    private static ExtractResult.Extracted extractOne(Path dir, NativesManifest.Library lib, String force,
            boolean testHooks) throws IOException {
        Files.createDirectories(dir); // parents first (Sable's recurring macOS failure, plan §6.5 step 4)
        boolean forceIo = testHooks && force.equals("io");
        if (forceIo)
            setWritable(dir, false);
        try {
            Object jvmLock = DIR_LOCKS.computeIfAbsent(dir, d -> new Object());
            synchronized (jvmLock) {
                try (FileChannel lockChannel = FileChannel.open(dir.resolve(".lock"), StandardOpenOption.CREATE,
                        StandardOpenOption.WRITE); FileLock ignored = lockChannel.lock()) {
                    Path file = dir.resolve(lib.file());
                    if (!forceIo && Files.isRegularFile(file) && lib.sha256().equals(sha256(file)))
                        return new ExtractResult.Extracted(lib.platform(), lib.variant(), file, lib.sha256(),
                                lib.rustTarget(), false);

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
                    if (testHooks && Boolean.getBoolean(PROP_DELETE_AFTER_WRITE))
                        Files.deleteIfExists(file);
                    if (!Files.isRegularFile(file))
                        throw new VanishedException(file, file + " vanished right after it was written");
                    String again = sha256(file);
                    if (!again.equals(lib.sha256()))
                        throw new VanishedException(file, file + " changed right after it was written (sha256 " + again + ")");
                    return new ExtractResult.Extracted(lib.platform(), lib.variant(), file, lib.sha256(),
                            lib.rustTarget(), true);
                }
            }
        } finally {
            if (forceIo)
                setWritable(dir, true);
        }
    }

    /**
     * Step 7: removes other versions' directories, best effort (a locked file is skipped). Every
     * candidate's directory stays (both builds, both libcs) and so does the CPU probe's.
     */
    private static void prune(Path gameDir, ExtractResult extracted) {
        Path root = nativesRoot(gameDir);
        Set<Path> keep = new HashSet<>();
        for (ExtractResult.Extracted c : extracted.candidates())
            keep.add(c.file().getParent().toAbsolutePath().normalize());
        if (extracted.probeFile() != null)
            keep.add(extracted.probeFile().getParent().toAbsolutePath().normalize());
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

    private static void systemLoad(ExtractResult.Extracted c, String force) {
        Path file = c.file();
        if (force.equals("libc_mismatch"))
            throw new UnsatisfiedLinkError(file + ": /lib64/libc.so.6: version `GLIBC_2.99' not found (required by "
                    + file + ") [forced by -D" + PROP_FORCE_FAIL + "=libc_mismatch]");
        if (force.equals("v3_link") && c.isV3())
            throw new UnsatisfiedLinkError(file + ": cannot load the x86-64-v3 build [forced by -D" + PROP_FORCE_FAIL
                    + "=v3_link]");
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

    /** Parsed {@code <semver>+<sha>;format=<f>;target=<triple>;cpu=<baseline|x86-64-v3>}. */
    record VersionInfo(String semver, String gitSha, String format, String target, String cpu) {
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
            String cpu = null;
            for (int i = 1; i < parts.length; i++) {
                if (parts[i].startsWith("format="))
                    format = parts[i].substring("format=".length());
                else if (parts[i].startsWith("target="))
                    target = parts[i].substring("target=".length());
                else if (parts[i].startsWith("cpu="))
                    cpu = parts[i].substring("cpu=".length());
            }
            if (semver.isEmpty() || format == null)
                return Optional.empty();
            return Optional.of(new VersionInfo(semver, sha, format, target, cpu));
        }
    }

    private static NativeStatus verify(ExtractResult extracted, ExtractResult.Extracted c, String force,
            String v3Failure) {
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
        // The build is what the library says it is (cpu= of its version string, deeptime-jni
        // CPU_LEVEL); natives.json should agree, and build-natives.sh checks that it does.
        String variant = info.cpu() != null ? info.cpu() : c.variant();
        if (c.variant() != null && info.cpu() != null && !c.variant().equals(info.cpu()))
            log(NativeLogSink.LOADER, Level.WARNING, "natives.json lists " + c.file() + " as the " + c.variant()
                    + " build, but the library reports cpu=" + info.cpu(), null);
        String reason = extracted.variantReason();
        if (v3Failure != null)
            reason += "; " + v3Failure + ", so the baseline build";
        return NativeStatus.ok(key, c.file(), info.semver(), info.format(), info.target(), info.gitSha(), sum, variant,
                reason);
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
