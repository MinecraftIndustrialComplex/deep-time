// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Loads the Deep Time native library on a bare JVM, the way the mod does, and prints one line:
//   status=OK version=<semver> format=<f> selftest=<checksum>
//   status=FAILED class=<FailureClass>
// It always exits 0: the line is the result (plan P0-07, P0-08's OS load matrix).
//
// Run with only the mod jar on the classpath (NativeLoader is JDK-only):
//   java -cp neoforge/build/libs/deeptime-1.21.1-0.0.1.jar tools/natives/NativeSmoke.java <gamedir>
// Forcing properties (plan §6.6): -Ddeeptime.natives.forceFail=unsupported|libc_mismatch|io|version_mismatch,
// -Ddeeptime.natives.testDeleteAfterWrite=true; -Ddeeptime.natives.path=<file> loads a given library.

import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeStatus;
import java.nio.file.Path;

public class NativeSmoke {
    public static void main(String[] args) {
        if (args.length != 1) {
            System.err.println("usage: java -cp <mod jar> tools/natives/NativeSmoke.java <gamedir>");
            System.out.println("status=FAILED class=USAGE");
            return;
        }
        // Loader diagnostics go to stderr; stdout carries only the result line.
        NativeLoader.setLogSink((logger, level, message, thrown) -> {
            if (level.getSeverity() >= System.Logger.Level.INFO.getSeverity()) {
                System.err.println("[" + level + "] " + logger + ": " + message);
                if (thrown != null)
                    System.err.println("    cause: " + thrown);
            }
        });
        NativeStatus status = NativeLoader.load(Path.of(args[0]));
        System.out.println(status.summary());
    }
}
